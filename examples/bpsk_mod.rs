use comms_rs::filter::fir_node::BatchFirNode;
use comms_rs::io::raw_iq::IQBatchOutput;
use comms_rs::node::graph::Graph;
use comms_rs::prelude::*;
use comms_rs::util::math;
use comms_rs::util::rand_node;
use num::{Complex, Num, NumCast, Zero};
use std::fs::File;
use std::io::BufWriter;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

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
        T: Zero + Copy + Send,
    {
        input: NodeReceiver<Vec<T>>,
        upsample_factor: usize,
        output: NodeSender<Vec<T>>,
    }

    impl<T> UpsampleNode<T>
    where
        T: Zero + Copy + Send,
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
        T: Copy + Num + NumCast + Send,
        U: Copy + Num + NumCast + Send,
    {
        input: NodeReceiver<Vec<Complex<T>>>,
        output: NodeSender<Vec<Complex<U>>>,
    }

    impl<T, U> ConvertNode<T, U>
    where
        T: Copy + Num + NumCast + Send,
        U: Copy + Num + NumCast + Send,
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

    let mut graph = Graph::new();
    let rand_bits = Arc::new(Mutex::new(rand_node::random_bit()));
    let bpsk_node = Arc::new(Mutex::new(BpskMod::new(4096)));
    let upsample = Arc::new(Mutex::new(UpsampleNode::new(4)));
    let sam_per_sym = 4.0;
    let taps: Vec<Complex<f32>> =
        math::rrc_taps(32, sam_per_sym, 0.25).unwrap();
    let pulse_shape = Arc::new(Mutex::new(BatchFirNode::new(taps, None)));
    let writer = BufWriter::new(File::create("./bpsk_out.bin").unwrap());
    let convert = Arc::new(Mutex::new(ConvertNode::new()));
    let iq_out = Arc::new(Mutex::new(IQBatchOutput::new(writer)));
    let nodes: Vec<Arc<Mutex<dyn Node>>> = vec![
        rand_bits.clone(),
        bpsk_node.clone(),
        pulse_shape.clone(),
        upsample.clone(),
        convert.clone(),
        iq_out.clone(),
    ];
    graph.add_nodes(nodes);

    {
        let mut rand_bits = rand_bits.lock().unwrap();
        let mut bpsk_node = bpsk_node.lock().unwrap();
        let mut pulse_shape = pulse_shape.lock().unwrap();
        let mut convert = convert.lock().unwrap();
        let mut iq_out = iq_out.lock().unwrap();
        let mut upsample = upsample.lock().unwrap();
        graph.connect_nodes(&mut rand_bits.sender, &mut bpsk_node.input, None);
        graph.connect_nodes(&mut bpsk_node.output, &mut upsample.input, None);
        graph.connect_nodes(&mut upsample.output, &mut pulse_shape.input, None);
        graph.connect_nodes(&mut pulse_shape.sender, &mut convert.input, None);
        graph.connect_nodes(&mut convert.output, &mut iq_out.input, None);
    }

    assert!(graph.is_connected());
    graph.run_graph();
    thread::sleep(Duration::from_secs(10));
}
