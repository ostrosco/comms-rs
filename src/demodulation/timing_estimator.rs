use crate::prelude::*;
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
    ///         This effectively sets the pulse filter length to 2 * N * D in
    ///         samples.
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
    pub fn new(n: u32, d: u32, alpha: f64) -> Result<TimingEstimator, &'static str> {

        // Generate Mengali's q(t)
        let taps = qfilt_taps(2 * n * d, alpha, n, 1.0)?;
        let taps = taps.iter().map(|x| Complex::new(*x as f64, 0.0)).collect();
        let mut delay = vec![0.0; ((n * d) as i32 - 1) as usize];
        delay.push(1.0);
        let delay = delay.iter().map(|x| Complex::new(*x as f64, 0.0)).collect();

        Ok(TimingEstimator {
            qfilt: taps,
            delay: delay,
            n,
            d,
        })
    }

    /// Calculates a new timing estimate from the input sample vector.
    ///
    /// Result is in units normalized to the input vector sample rate.
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
    /// let data = (0..100).map(|x| Complex::new(x as f64, 0.0)).collect();
    ///
    /// let estimate = estimator.push(&data);
    /// ```
    pub fn push(&mut self, samples: &Vec<Complex<f64>>) -> f64 {

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
        let mut initial_qfilt_state = vec![Complex::new(0.0, 0.0); (2 * self.n * self.d) as usize];
        let mut initial_delay_state = vec![Complex::new(0.0, 0.0); (self.n * self.d) as usize];
        let qout = batch_fir(&qin, &self.qfilt, &mut initial_qfilt_state);
        let dout = batch_fir(&din, &self.delay, &mut initial_delay_state);

        // Multiply input samples with mixing vector and delay by ND samples to
        // ensure it lines up with qfilt output, then multiply by qfilt output
        // and sum.
        let sum_value: Complex<f64> = qout.iter().zip(dout.iter()).map(|(q, d)| q * d).sum();

        -sum_value.arg() / (2.0 * PI)
    }
}
