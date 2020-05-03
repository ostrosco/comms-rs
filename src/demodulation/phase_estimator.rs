use std::f64::consts::PI;

extern crate num; // 0.2.0

use num::complex::Complex;

// Reference Chp. 5.7.4 in Mengali
//

/// Calculates a phase estimate from the input PSK symbol vector.
///
/// Result is in radians, and the estimator assumes the input symbol vector is
/// the output from a matched filter.  It additionally  assumes the vector has
/// already gone through accurate timing recovery.
///
/// # Arguments
///
/// * `symbols` - Input vector of symbols to calculate the phase estimate from.
///
/// # Examples
///
/// ```
/// use comms_rs::demodulation::phase_estimator::*;
/// use num::Complex;
///
/// let m = 4;
/// let data: Vec<_> = (0..100).map(|x| Complex::new(0.0, x as f64).exp()).collect();
///
/// let estimate = psk_phase_estimate(&data);
/// ```
pub fn psk_phase_estimate(m: u32, symbols: &[Complex<f64>]) -> f64 {
    symbols.iter().map(|x| x.powi(m)).sum().arg() / (m as f64)
}

#[cfg(test)]
mod test {
    use crate::demodulation::timing_estimator::*;
    use num::Complex;
    use rand::distributions::Uniform;
    use rand::prelude::*;
    use rand::rngs::SmallRng;
    use std::f64::consts::PI;

    #[test]
    fn test_phase_estimator() {
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
