//! This module implements frequency demodulation routines for analog signals.
use num::Complex;
use num::Float;
use num::Zero;

/// This struct enables state persistence for frequency demodulation and instantaneous frequency
/// estimation.
pub struct FM<T> {
    prev: Complex<T>,
}

impl<T> FM<T>
where
    T: Float + Zero,
{
    /// This function implements batch frequency demodulation for complex samples and returns a
    /// batch of demodulated real samples.
    ///
    /// # Arguments
    ///
    /// * `samples` - The set of complex, frequency modulated samples to demodulate
    pub fn demod(&mut self, samples: &[Complex<T>]) -> Vec<T> {
        let mut prev = self.prev;
        let mut demod_queue: Vec<T> = Vec::with_capacity(samples.len());

        for samp in samples {
            let theta = samp * prev.conj();
            demod_queue.push(theta.arg());
            prev = *samp;
        }
        self.prev = prev;

        demod_queue
    }
}

impl<T> Default for FM<T>
where
    T: Float + Zero,
{
    /// Implements the default method for instantiating the FM node, initializing the previous
    /// sample to zero. A user should never have to modify this value.
    fn default() -> Self {
        FM {
            prev: Complex::<T>::zero(),
        }
    }
}
