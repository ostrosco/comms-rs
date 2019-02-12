use crate::prelude::*;

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
pub struct RadioTxNode<T, U>
where
    T: RadioTx<U>,
    U: Clone,
{
    pub input: NodeReceiver<Vec<U>>,
    radio: T,
    output_idx: usize,
}

impl<T, U> RadioTxNode<T, U>
where
    T: RadioTx<U>,
    U: Clone,
{
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
    T: RadioRx<U>,
    U: Clone,
{
    radio: T,
    input_idx: usize,
    num_samples: usize,
    pub sender: NodeSender<Vec<U>>,
}

impl<T, U> RadioRxNode<T, U>
where
    T: RadioRx<U>,
    U: Clone,
{
    pub fn run(&mut self) -> Result<Vec<U>, NodeError> {
        Ok(self.radio.recv_samples(self.num_samples, self.input_idx))
    }
}
