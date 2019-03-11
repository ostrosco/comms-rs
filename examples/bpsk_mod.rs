#[macro_use]
extern crate comms_rs;
extern crate zmq;

use comms_rs::filter::fir_node::BatchFirNode;
use comms_rs::io::raw_iq::IQBatchOutput;
use comms_rs::io::zmq_node::ZMQSend;
use comms_rs::prelude::*;
use comms_rs::util::math;
use comms_rs::util::rand_node;
use num::{Complex, Num, NumCast, Zero};
use std::fs::File;
use std::time::Instant;

fn main() {
    #[derive(Node, Default)]
    #[aggregate]
    struct BpskMod {
        input: NodeReceiver<u8>,
        num_samples: usize,
        state: Vec<Complex<f32>>,
        output: NodeSender<Vec<Complex<f32>>>,
    }

    impl BpskMod {
        pub fn new(num_samples: usize) -> Self {
            BpskMod {
                num_samples,
                state: vec![],
                ..Default::default()
            }
        }

        pub fn run(
            &mut self,
            input: u8,
        ) -> Result<Option<Vec<Complex<f32>>>, NodeError> {
            self.state.push(Complex::new(input as f32 * 2.0 - 1.0, 0.0));
            if self.state.len() == self.num_samples {
                let state_cl = self.state.clone();
                self.state = vec![];
                Ok(Some(state_cl))
            } else {
                Ok(None)
            }
        }
    }

    #[derive(Node)]
    struct UpsampleNode<T>
    where
        T: Zero + Clone,
    {
        input: NodeReceiver<Vec<T>>,
        upsample_factor: usize,
        output: NodeSender<Vec<T>>,
    }

    impl<T> UpsampleNode<T>
    where
        T: Zero + Clone,
    {
        pub fn new(upsample_factor: usize) -> Self {
            UpsampleNode {
                upsample_factor,
                input: Default::default(),
                output: Default::default(),
            }
        }

        pub fn run(&mut self, input: Vec<T>) -> Result<Vec<T>, NodeError> {
            let mut out = vec![T::zero(); input.len() * self.upsample_factor];
            let mut ix = 0;
            for val in input {
                out[ix] = val;
                ix += self.upsample_factor;
            }
            Ok(out)
        }
    }

    #[derive(Node)]
    struct ConvertNode<T, U>
    where
        T: Clone + Num + NumCast,
        U: Clone + Num + NumCast,
    {
        input: NodeReceiver<Vec<Complex<T>>>,
        output: NodeSender<Vec<Complex<U>>>,
    }

    impl<T, U> ConvertNode<T, U>
    where
        T: Clone + Num + NumCast,
        U: Clone + Num + NumCast,
    {
        pub fn new() -> Self {
            ConvertNode {
                input: Default::default(),
                output: Default::default(),
            }
        }

        pub fn run(
            &mut self,
            input: Vec<Complex<T>>,
        ) -> Result<Vec<Complex<U>>, NodeError> {
            let out: Vec<Complex<U>> = input
                .iter()
                .map(|x| math::cast_complex(x).unwrap())
                .collect();
            Ok(out)
        }
    }

    #[derive(Node)]
    struct DeinterleaveNode<T>
    where
        T: Clone + Num + NumCast,
    {
        input: NodeReceiver<Vec<Complex<T>>>,
        output: NodeSender<Vec<T>>,
    }

    impl<T> DeinterleaveNode<T>
    where
        T: Clone + Num + NumCast,
    {
        pub fn new() -> Self {
            DeinterleaveNode {
                input: Default::default(),
                output: Default::default(),
            }
        }

        pub fn run(
            &mut self,
            input: Vec<Complex<T>>,
        ) -> Result<Vec<T>, NodeError> {
            let out: Vec<T> = input
                .iter()
                .flat_map(|x| vec![x.re.clone(), x.im.clone()])
                .collect();
            Ok(out)
        }
    }

    let mut rand_bits = rand_node::random_bit();
    let mut bpsk_node = BpskMod::new(128);
    let mut upsample = UpsampleNode::new(4);
    let sam_per_sym = 4.0;
    let taps: Vec<Complex<f32>> =
        math::rrc_taps(32, sam_per_sym, 0.25).unwrap();
    let mut pulse_shape = BatchFirNode::new(taps, None);
    let writer = File::create("/tmp/bpsk_out.bin").unwrap();
    let mut convert = ConvertNode::new();
    let mut iq_out = IQBatchOutput::new(writer);
    let mut zmq = ZMQSend::new("tcp://*:5563", zmq::SocketType::PUB, 0);
    let mut deinterleave = DeinterleaveNode::new();

    connect_nodes!(rand_bits, sender, bpsk_node, input);
    connect_nodes!(bpsk_node, output, upsample, input);
    connect_nodes!(upsample, output, pulse_shape, input);
    connect_nodes!(pulse_shape, sender, convert, input);
    connect_nodes!(convert, output, iq_out, input);
    connect_nodes!(convert, output, deinterleave, input);
    connect_nodes!(deinterleave, output, zmq, input);
    start_nodes!(
        rand_bits,
        bpsk_node,
        upsample,
        pulse_shape,
        convert,
        iq_out,
        deinterleave,
        zmq
    );

    let start = Instant::now();
    loop {
        if start.elapsed().as_secs() > 10 {
            break;
        }
    }
}
