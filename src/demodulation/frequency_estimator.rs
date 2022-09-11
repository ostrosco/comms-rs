use num::Complex;

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
///
/// # Examples
///
/// ```
/// use comms_rs::demodulation::frequency_estimator::*;
/// use num::Complex;
///
/// let data: Vec<_> = (0..100).map(|x| Complex::new(0.0, x as f64).exp()).collect();
///
/// let estimate = frequency_offset_estimate(&data);
/// ```
pub fn frequency_offset_estimate(samples: &[Complex<f64>]) -> f64 {
    let latest: Vec<_> = samples.iter().skip(1).collect();
    let delayed: Vec<_> = samples
        .iter()
        .take(latest.len())
        .map(|x| x.conj())
        .collect();

    let mult: Vec<_> = latest
        .iter()
        .zip(delayed.iter())
        .map(|(x, y)| *x * y)
        .collect();
    let accum: Complex<f64> = mult.iter().sum();
    accum.arg()
}

#[cfg(test)]
mod test {
    use crate::demodulation::frequency_estimator::*;
    use crate::filter::fir;
    use crate::num::Zero;
    use crate::util::math::*;
    use num::Complex;
    use rand::distributions::Uniform;
    use rand::prelude::*;
    use rand::rngs::SmallRng;
    use std::f64::consts::PI;

    #[test]
    fn test_frequency_estimator() {
        // 4 PSK
        let m = 4;

        // Generate symbols
        let mut rng = SmallRng::seed_from_u64(0);
        let interval = Uniform::new(0, m);

        let symbols: Vec<_> = (0..4096)
            .map(|_| rng.sample(interval))
            .map(|x| Complex::new(0.0, 2.0 * PI * x as f64 / (m as f64)).exp())
            .collect();

        // Oversample these symbols!
        let sam_per_sym = 4;
        let mut upsample = vec![Complex::zero(); symbols.len() * sam_per_sym];
        let mut ix = 0;
        for samp in symbols {
            upsample[ix] = samp;
            ix += sam_per_sym;
        }
        let taps: Vec<Complex<f64>> =
            rrc_taps(16, sam_per_sym as f64, 0.75).unwrap();
        let mut state: Vec<Complex<f64>> = vec![Complex::zero(); 16];
        let data = fir::batch_fir(&upsample, &taps, &mut state);

        // Add in frequency shift
        let truth = 0.123456789;
        let data: Vec<_> = data
            .iter()
            .enumerate()
            .map(|(i, x)| x * (Complex::new(0.0, truth * (i as f64))).exp())
            .collect();

        // Create estimator
        let estimate = frequency_offset_estimate(&data);

        assert!((truth - estimate).abs() < 0.01);
    }
}
