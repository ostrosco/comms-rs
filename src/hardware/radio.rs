use crossbeam::{channel, Receiver, Sender};
use node::Node;

pub trait RadioConfig {
    fn init_radio(&mut self);
    fn teardown(&mut self);
    fn set_center_freq(&mut self);
    fn get_center_freq(&mut self);
    fn set_sample_rate(&mut self);
    fn get_sample_rate(&mut self);
    fn set_gain(&mut self);
    fn get_gain(&mut self);
}

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
    T: RadioConfig + RadioTx<U>, U: Clone,
);

create_node!(
    RadioRxNode<T, U>: Vec<U>,
    [radio: T, input_idx: usize, num_samples: usize],
    [],
    |node: &mut RadioRxNode<T, U>| {
        node.radio.recv_samples(node.num_samples, node.input_idx)
    },
    T: RadioConfig + RadioRx<U>, U: Clone,
);
