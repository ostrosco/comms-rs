use crossbeam::{Receiver, Sender};
use node::Node;

pub trait RadioTx<T> {
    fn send_samples(&mut self, samples: Vec<T>, output_idx: usize);
}

pub trait RadioRx<T> {
    fn recv_samples(&mut self, num_samples: usize, input_idx: usize) -> Vec<T>;
}

create_node!(
    RadioTxNode<T, U>: (),
    [radio: T, output_idx: usize],
    [recv_samp: Vec<U>],
    |node: &mut RadioTxNode<T, U>, samples: Vec<U>| {
        node.radio.send_samples(samples, node.output_idx);
    },
    T: RadioTx<U>, U: Clone,
);

create_node!(
    RadioRxNode<T, U>: Vec<U>,
    [radio: T, input_idx: usize, num_samples: usize],
    [],
    |node: &mut RadioRxNode<T, U>| {
        node.radio.recv_samples(node.num_samples, node.input_idx)
    },
    T: RadioRx<U>, U: Clone,
);
