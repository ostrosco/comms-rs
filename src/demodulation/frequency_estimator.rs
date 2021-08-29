use num::complex::Complex;

/// Calculates a carrier offset estimate from the input sample vector.
///
/// Result is in radians/sample, and the estimator assumes the input symbol
/// vector is oversampled by some integer oversampling factor, `m`. The input
/// samples should NOT have gone through a matched filter at this point in the
/// demodulator chain.
///
/// Reference Chp. 8.2.2 in Meyr, Moeneclaey, and Fechtel.
///
/// # Arguments
///
/// * `samples` - Input vector of samples to calculate the carrier frequency
///               offset estimate from.
/// * `m` - Input sample vector oversampling factor.
///
/// # Examples
///
/// ```
/// use comms_rs::demodulation::frequency_estimator::*;
/// use num::Complex;
///
/// let m = 4;
/// let data: Vec<_> = (0..100).map(|x| Complex::new(0.0, x as f64).exp()).collect();
///
/// let estimate = frequency_offset_estimate(&data, m);
/// ```
pub fn frequency_offset_estimate(samples: &[Complex<f64>], m: u32) -> f64 {
    let latest: Vec<_> = samples.iter().skip(1).collect();
    let delayed: Vec<_> = samples.iter().take(latest.len()).map(|x| x.conj()).collect();

    let accum = latest.iter().zip(delayed.iter()).map(|x, y| x * y).sum();
    accum.arg()
}

#[cfg(test)]
mod test {
    use crate::demodulation::frequency_estimator::*;
    use num::Complex;
    use rand::distributions::Uniform;
    use rand::prelude::*;
    use rand::rngs::SmallRng;
    use std::f64::consts::PI;

    #[test]
    fn test_frequency_estimator() {

        // 8 PSK
        let m = 8;
        let truth = 0.123456789;

        // Generate symbols
        let mut rng = SmallRng::seed_from_u64(0);
        let interval = Uniform::new(0, m);

        let symbols: Vec<_> = (0..1000)
            .map(|_| rng.sample(interval))
            .enumerate()
            .map(|i, x| {
                Complex::new(0.0, 2.0 * PI * x as f64 / (m as f64) + i * truth)
                    .exp()
            })
            .collect();

        // TODO: Oversample these symbols!
        let oversampling_factor = 4;

        // Create estimator
        let estimate = frequency_offset_estimate(&symbols, oversampling_factor);

        assert!((truth - estimate).abs() < 0.000001);
    }
}
