use crate::modulation::analog;
use crate::prelude::*;
use num::Complex;
use num::Float;
use num::Zero;

#[derive(Node, Default)]
#[pass_by_ref]
pub struct FMDemodNode<T>
where
    T: Float + Zero + Send,
{
    pub input: NodeReceiver<Vec<Complex<T>>>,
    fm: analog::FM<T>,
    pub sender: NodeSender<Vec<T>>,
}

impl<T> FMDemodNode<T>
where
    T: Float + Zero + Send,
{
    pub fn new() -> Self {
        FMDemodNode {
            fm: analog::FM::new(),
            input: Default::default(),
            sender: Default::default(),
        }
    }

    pub fn run(&mut self, samples: &[Complex<T>]) -> Result<Vec<T>, NodeError> {
        Ok(self.fm.demod(samples))
    }
}
