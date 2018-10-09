use crossbeam::{Receiver, Sender};
use node::Node;
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

impl<T> FMDemodNode<T> where T: Float {
    pub fn demod(&mut self, samples: Vec<Complex<T>>) -> Vec<T> {
        let mut prev = self.prev;
        let mut demod_queue: Vec<T> = Vec::with_capacity(samples.len());

        for samp in samples {
            let conj = prev.conj() * samp;
            demod_queue.push(conj.arg());
            prev = samp;
        }
        self.prev = prev;

        demod_queue
    }
}
