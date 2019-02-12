use crate::modulation::analog;
use crate::prelude::*;
use num::Complex;
use num::Float;
use num::Zero;

#[derive(Node)]
pub struct FMDemodNode<T>
where
    T: Float + Zero,
{
    pub input: NodeReceiver<Vec<Complex<T>>>,
    fm: analog::FM<T>,
    pub sender: NodeSender<Vec<T>>,
}

impl<T> FMDemodNode<T>
where
    T: Float + Zero,
{
    pub fn run(&mut self, samples: &[Complex<T>]) -> Result<Vec<T>, NodeError> {
        Ok(self.fm.demod(samples))
    }
}

pub fn fm_demod_node<T>() -> FMDemodNode<T>
where
    T: Float + Zero,
{
    FMDemodNode::new(analog::FM::new())
}
