use crate::prelude::*;

/// A trait to capture the ability to send samples out of the hardware
/// platform on a particular output.
pub trait RadioTx<T> {
    fn send_samples(&mut self, samples: Vec<T>, output_idx: usize);
}

/// A trait to capture the ability to receive samples from the hardware
/// platform on a particular input.
pub trait RadioRx<T> {
    fn recv_samples(&mut self, num_samples: usize, input_idx: usize) -> Vec<T>;
}

create_node!(
    #[doc = "A node that takes a generic hardware platform that supports "]
    #[doc = "transmissing samples."]
    RadioTxNode<T, U>: (),
    [radio: T, output_idx: usize],
    [recv_samp: Vec<U>],
    |node: &mut RadioTxNode<T, U>, samples: Vec<U>| -> Result<(), NodeError> {
        node.radio.send_samples(samples, node.output_idx);
        Ok(())
    },
    T: RadioTx<U>, U: Clone,
);

create_node!(
    #[doc = "A node that takes a generic hardware platform that supports "]
    #[doc = "receiving samples."]
    RadioRxNode<T, U>: Vec<U>,
    [radio: T, input_idx: usize, num_samples: usize],
    [],
    |node: &mut RadioRxNode<T, U>| -> Result<Vec<U>, NodeError> {
        let res = node.radio.recv_samples(node.num_samples, node.input_idx);
        Ok(res)
    },
    T: RadioRx<U>, U: Clone,
);
