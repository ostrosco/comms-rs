use prelude::*;
use num::Complex;
use num::Float;

create_node!(
    FMDemodNode<T>: Vec<T>,
    [prev: Complex<T>],
    [recv: Vec<Complex<T>>],
    |node: &mut FMDemodNode<T>, samples: Vec<Complex<T>>| {
        node.demod(samples)
    },
    T: Float,
);

impl<T> FMDemodNode<T>
where
    T: Float,
{
    pub fn demod(&mut self, samples: Vec<Complex<T>>) -> Result<Vec<T>, NodeError> {
        let mut prev = self.prev;
        let mut demod_queue: Vec<T> = Vec::with_capacity(samples.len());

        for samp in samples {
            let theta = samp * prev.conj();
            demod_queue.push(theta.arg());
            prev = samp;
        }
        self.prev = prev;

        Ok(demod_queue)
    }
}
