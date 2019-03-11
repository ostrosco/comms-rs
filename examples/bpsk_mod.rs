#[macro_use]
extern crate comms_rs;

use comms_rs::filter::fir_node::BatchFirNode;
use comms_rs::io::raw_iq::IQBatchOutput;
use comms_rs::io::zmq_node::ZMQSend;
use comms_rs::prelude::*;
use comms_rs::util::math;
use comms_rs::util::rand_node;
use num::{Complex, Num, NumCast, Zero};
use std::fs::File;
use std::io::BufWriter;
use std::thread;
use std::time::Duration;
use zmq;

/// An example that will generate random numbers, pass them through a BPSK
/// modulation and a pulse shaper, then broadcast them out to a file and
/// via ZeroMQ for visualization.
fn main() {
    // A simple node to perform BPSK modulation. Only broadcasts a message
    // once `num_samples` samples have been received and modulated.
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

    // A simple node to perform upsampling of data. Zeros are injected here;
    // filtering takes place elsewhere.
    #[derive(Node)]
    struct UpsampleNode<T>
    where
        T: Zero + Copy,
    {
        input: NodeReceiver<Vec<T>>,
        upsample_factor: usize,
        output: NodeSender<Vec<T>>,
    }

    impl<T> UpsampleNode<T>
    where
        T: Zero + Copy,
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

    // A generic node to convert from one complex type to another.
    #[derive(Node)]
    struct ConvertNode<T, U>
    where
        T: Copy + Num + NumCast,
        U: Copy + Num + NumCast,
    {
        input: NodeReceiver<Vec<Complex<T>>>,
        output: NodeSender<Vec<Complex<U>>>,
    }

    impl<T, U> ConvertNode<T, U>
    where
        T: Copy + Num + NumCast,
        U: Copy + Num + NumCast,
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

    // A node to convert a vector of complex numbers into a vector of
    // numbers which alternate between real and imaginary.
    #[derive(Node)]
    struct DeinterleaveNode<T>
    where
        T: Copy + Num + NumCast,
    {
        input: NodeReceiver<Vec<Complex<T>>>,
        output: NodeSender<Vec<T>>,
    }

    impl<T> DeinterleaveNode<T>
    where
        T: Clone + Num + NumCast + Copy,
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
            let out: Vec<T> =
                input.iter().flat_map(|x| vec![x.re, x.im]).collect();
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
    let writer = BufWriter::new(File::create("/tmp/bpsk_out.bin").unwrap());
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
        pulse_shape,
        iq_out,
        zmq,
        upsample,
        convert,
        deinterleave
    );

    thread::sleep(Duration::from_secs(10));
}
