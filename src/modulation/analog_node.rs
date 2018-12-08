use modulation::analog;
use num::Complex;
use num::Float;
use num::Zero;
use prelude::*;

create_node!(
    FMDemodNode<T>: Vec<T>,
    [fm: analog::FM<T>],
    [recv: Vec<Complex<T>>],
    |node: &mut FMDemodNode<T>, samples: Vec<Complex<T>>| {
        Ok(node.fm.demod(samples))
    },
    T: Float + Zero,
);

pub fn fm_demod_node<T>() -> FMDemodNode<T>
where
    T: Float + Zero,
{
    FMDemodNode::new(analog::FM::new())
}
