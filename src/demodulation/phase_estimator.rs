use num::Complex;

/// Calculates a phase estimate from the input PSK symbol vector.
///
/// Result is in radians, and the estimator assumes the input symbol vector is
/// the output from a matched filter.  It additionally  assumes the vector has
/// already gone through accurate timing recovery.
///
/// Reference Chp. 5.7.4 in Mengali.
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
/// let estimate = psk_phase_estimate(&data, m);
/// ```
pub fn psk_phase_estimate(symbols: &[Complex<f64>], m: u32) -> f64 {
    symbols
        .iter()
        .map(|x| x.powi(m as i32))
        .sum::<Complex<f64>>()
        .arg()
        / (m as f64)
}

/// Calculates a phase estimate from the input QAM symbol vector.
///
/// Result is in radians, and the estimator assumes the input symbol vector is
/// the output from a matched filter.  It additionally  assumes the vector has
/// already gone through accurate timing recovery.
///
/// Reference Chp. 5.7.5 in Mengali.
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
/// let estimate = qam_phase_estimate(&data);
/// ```
pub fn qam_phase_estimate(symbols: &[Complex<f64>]) -> f64 {
    symbols
        .iter()
        .map(|x| -1.0 * x.powi(4))
        .sum::<Complex<f64>>()
        .arg()
        / 4.0
}

#[cfg(test)]
mod test {
    use crate::demodulation::phase_estimator::*;
    use num::Complex;
    use rand::distributions::Uniform;
    use rand::prelude::*;
    use rand::rngs::SmallRng;
    use std::f64::consts::PI;

    #[test]
    fn test_psk_phase_estimator() {
        // 8 PSK
        let m = 8;
        let truth = 0.123456;

        // Generate symbols
        let mut rng = SmallRng::seed_from_u64(0);
        let interval = Uniform::new(0, m);
        let symbols: Vec<_> = (0..1000)
            .map(|_| rng.sample(interval))
            .map(|x| {
                Complex::new(0.0, 2.0 * PI * x as f64 / (m as f64) + truth)
                    .exp()
            })
            .collect();

        // Create estimator
        let estimate = psk_phase_estimate(&symbols, m);

        assert!((truth - estimate).abs() < 0.000001);
    }

    #[test]
    fn test_qam_phase_estimator() {
        // 16 QAM
        let truth = 0.123456;

        // Generate symbols
        let mut rng = SmallRng::seed_from_u64(0);
        let interval = Uniform::new(0, 16);
        let symbols: Vec<i32> =
            (0..1000).map(|_| rng.sample(interval)).collect();

        let symbols: Vec<_> = symbols
            .iter()
            .map(|x| {
                Complex::new(
                    (*x % 4) as f64 - 1.5,
                    ((*x as f64) / 4.0).trunc() - 1.5,
                )
            })
            .map(|x| 2.0 * x * Complex::new(0.0, truth).exp())
            .collect();

        // Create estimator
        let estimate = qam_phase_estimate(&symbols);

        assert!((truth - estimate).abs() < 0.01);
    }
}
