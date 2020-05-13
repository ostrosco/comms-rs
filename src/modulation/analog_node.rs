//! Node based implementation for analog demodulation.
//!
//! Modulation is the general process of taking information and formatting it for going over an RF
//! channel - in this case, information which varies smoothly, like voice, versus discretely, like
//! digital images. For example, frequency modulation is the process of varying the center
//! frequency of a carrier correlative with the various amplitude of the input signal.
//!
//! Frequency demodulation is typically accomplished by taking the differential phase between two
//! samples, which is a direct measure of the instantaneous frequency.
use crate::modulation::analog;
use crate::prelude::*;
use num::Complex;
use num::Float;
use num::Zero;

/// This node implements a frequency demodulator node. Upon processing, it takes a batch of complex
/// samples and converts them to a vector of real, demodulated samples.
#[derive(Node, Default)]
#[pass_by_ref]
pub struct FMDemodNode<T>
where
    T: Float + Zero + Send + Default,
{
    pub input: NodeReceiver<Vec<Complex<T>>>,
    fm: analog::FM<T>,
    pub output: NodeSender<Vec<T>>,
}

impl<T> FMDemodNode<T>
where
    T: Float + Zero + Send + Default,
{
    /// Instantiates a new FM demodulation node. Takes no arguments.
    ///
    /// Examples:
    ///
    /// ```
    /// use comms_rs::modulation::analog_node::FMDemodNode;
    ///
    /// // Creates a new demodulation node to be connected to the graph
    /// let node = FMDemodNode::<f32>::new();
    /// ```
    pub fn new() -> Self {
        FMDemodNode::default()
    }

    /// Runs the FMDemodNode. Produces a batch of `Vector<T>`. Cannot actually produce a
    /// `NodeError`.
    pub fn run(&mut self, samples: &[Complex<T>]) -> Result<Vec<T>, NodeError> {
        Ok(self.fm.demod(samples))
    }
}
