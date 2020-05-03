use crate::filter::fir::batch_fir;
use crate::util::math::qfilt_taps;
use std::f64::consts::PI;

extern crate num; // 0.2.0

use num::complex::Complex;

// Reference Chp. 8.4 in Mengali
//
pub struct TimingEstimator {
    qfilt: Vec<Complex<f64>>,
    delay: Vec<Complex<f64>>,
    n: u32,
    d: u32,
}

impl TimingEstimator {
    /// Create a new TimingEstimator struct.
    ///
    /// # Arguments
    ///
    /// * `n` - Samples per symbol of input signal.
    /// * `d` - Delay of filters internal to the TimingEstimator in symbols.
    ///         This effectively sets the pulse filter length to 2 * N * D + 1
    ///         in samples.
    /// * `alpha` - Rolloff factor for internal filtering.  Should be on
    ///             interval [0, 1].
    ///
    /// # Examples
    ///
    /// ```
    /// use comms_rs::demodulation::timing_estimator::*;
    ///
    /// let n = 2;
    /// let d = 5;
    /// let alpha = 0.25;
    /// let estimator = TimingEstimator::new(n, d, alpha);
    /// ```
    pub fn new(
        n: u32,
        d: u32,
        alpha: f64,
    ) -> Result<TimingEstimator, &'static str> {
        // Generate Mengali's q(t)
        let taps = qfilt_taps(2 * n * d + 1, alpha, n)?;
        let taps = taps.iter().map(|x| Complex::new(*x as f64, 0.0)).collect();
        let mut delay = vec![0.0; (n * d) as usize];
        delay.push(1.0);
        let delay =
            delay.iter().map(|x| Complex::new(*x as f64, 0.0)).collect();

        Ok(TimingEstimator {
            qfilt: taps,
            delay,
            n,
            d,
        })
    }

    /// Calculates a new timing estimate from the input sample vector.
    ///
    /// Result is in samples.
    ///
    /// # Arguments
    ///
    /// * `samples` - Input vector of samples to calculate the timing
    ///               estimate from.
    ///
    /// # Examples
    ///
    /// ```
    /// use comms_rs::demodulation::timing_estimator::*;
    /// use num::Complex;
    ///
    /// let n = 2;
    /// let d = 5;
    /// let alpha = 0.25;
    /// let mut estimator = TimingEstimator::new(n, d, alpha).unwrap();
    ///
    /// let data: Vec<_> = (0..100).map(|x| Complex::new(x as f64, 0.0)).collect();
    ///
    /// let estimate = estimator.push(&data);
    /// ```
    pub fn push(&mut self, samples: &[Complex<f64>]) -> f64 {
        let mut qin = vec![];
        let mut din = vec![];
        for (i, s) in samples.iter().enumerate() {
            // Complex exponential for mixing
            let r = Complex::new(0.0, -PI * i as f64 / self.n as f64).exp();

            // Prepare inputs to parallel filters
            qin.push(s.conj() * r);
            din.push(s * r);
        }

        // Execute the parallel FIR filter
        let mut initial_qfilt_state =
            vec![Complex::new(0.0, 0.0); (2 * self.n * self.d + 1) as usize];
        let mut initial_delay_state =
            vec![Complex::new(0.0, 0.0); (self.n * self.d + 1) as usize];
        let qout = batch_fir(&qin, &self.qfilt, &mut initial_qfilt_state);
        let dout = batch_fir(&din, &self.delay, &mut initial_delay_state);

        // Multiply input samples with mixing vector and delay by ND samples to
        // ensure it lines up with qfilt output, then multiply by qfilt output
        // and sum.
        let sum_value: Complex<f64> =
            qout.iter().zip(dout.iter()).map(|(q, d)| q * d).sum();

        -(self.n as f64) * sum_value.arg() / (2.0 * PI)
    }
}

#[cfg(test)]
mod test {
    use crate::demodulation::timing_estimator::*;
    use crate::util::math::rrc_taps;
    use num::Complex;
    use rand::distributions::Uniform;
    use rand::prelude::*;
    use rand::rngs::SmallRng;
    use std::f64::consts::PI;

    #[test]
    fn test_timing_estimator() {
        let alpha = 0.5;
        let sam_per_sym = 10;

        // Generate QPSK signal
        let mut rng = SmallRng::seed_from_u64(0);
        let interval = Uniform::new(0, 4);
        let data: Vec<_> = (0..1000)
            .map(|_| rng.sample(interval))
            .map(|x| {
                Complex::new(0.0, 2.0 * PI * x as f64 / 4.0 + PI / 4.0).exp()
            })
            .collect();

        let mut symbols = vec![];
        for pt in data {
            symbols.push(pt);
            for _ in 1..sam_per_sym {
                symbols.push(Complex::new(0.0, 0.0));
            }
        }

        let n_taps = sam_per_sym * 10 + 1;
        let rrctaps = rrc_taps(n_taps, sam_per_sym as f64, alpha).unwrap();
        let mut state = vec![Complex::new(0.0, 0.0); n_taps as usize];
        let samples = batch_fir(&symbols, &rrctaps, &mut state);

        // Create estimator
        let truth = 2;
        let n = sam_per_sym;
        let d = 5;
        let mut estimator = TimingEstimator::new(n, d, alpha).unwrap();
        let estimate = estimator.push(&samples[truth..]);
        println!("{}", estimate);

        assert!((truth as f64 + estimate).abs() < 0.01);
    }
}
