use crate::prelude::*;
use std::default::Default;

/// A trait to capture the ability to send samples out of the hardware
/// platform on a particular output.
pub trait RadioTx<T> {
    fn send_samples(&mut self, samples: &[T], output_idx: usize);
}

/// A trait to capture the ability to receive samples from the hardware
/// platform on a particular input.
pub trait RadioRx<T> {
    fn recv_samples(&mut self, num_samples: usize, input_idx: usize) -> Vec<T>;
}

/// A node that takes a generic hardware platform that supports transmitting
/// samples.
#[derive(Node)]
#[pass_by_ref]
pub struct RadioTxNode<T, U>
where
    T: RadioTx<U> + Send,
    U: Clone + Send,
{
    pub input: NodeReceiver<Vec<U>>,
    radio: T,
    output_idx: usize,
}

impl<T, U> RadioTxNode<T, U>
where
    T: RadioTx<U> + Send,
    U: Clone + Send,
{
    pub fn new(radio: T, output_idx: usize) -> Self {
        RadioTxNode {
            radio,
            output_idx,
            input: Default::default(),
        }
    }

    pub fn run(&mut self, samples: &[U]) -> Result<(), NodeError> {
        self.radio.send_samples(samples, self.output_idx);
        Ok(())
    }
}

/// A node that takes a generic hardware platform that supports receiving
/// samples.
#[derive(Node)]
pub struct RadioRxNode<T, U>
where
    T: RadioRx<U> + Send,
    U: Clone + Send,
{
    radio: T,
    input_idx: usize,
    num_samples: usize,
    pub output: NodeSender<Vec<U>>,
}

impl<T, U> RadioRxNode<T, U>
where
    T: RadioRx<U> + Send,
    U: Clone + Send,
{
    pub fn new(radio: T, input_idx: usize, num_samples: usize) -> Self {
        RadioRxNode {
            radio,
            input_idx,
            num_samples,
            output: Default::default(),
        }
    }

    pub fn run(&mut self) -> Result<Vec<U>, NodeError> {
        Ok(self.radio.recv_samples(self.num_samples, self.input_idx))
    }
}
