//! This module implements frequency demodulation
use num::Complex;
use num::Float;
use num::Zero;

pub struct FM<T> {
    prev: Complex<T>,
}

impl<T> FM<T>
where
    T: Float + Zero,
{
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
    fn default() -> Self {
        FM {
            prev: Complex::<T>::zero(),
        }
    }
}
