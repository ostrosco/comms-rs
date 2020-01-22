#[macro_use]
extern crate comms_rs;
extern crate zmq;
use byteorder::{NativeEndian, WriteBytesExt};
use comms_rs::filter::fir;
use comms_rs::io::zmq_node::ZMQSend;
use comms_rs::prelude::*;
use comms_rs::util::math;
use num::{Complex, Zero};
use rand::distributions::Uniform;
use rand::Rng;
use std::fs::File;
use std::time::Instant;
use std::{thread, time};

/// An example that will generate random numbers, pass them through a QPSK
/// modulation and a pulse shaper, then broadcast them out to a file and
/// via ZeroMQ for visualization.
fn main() {
    let now = Instant::now();

    #[derive(Node)]
    struct QpskMod {
        pub sender: NodeSender<Vec<Complex<f32>>>,
    }
    impl QpskMod {
        pub fn new() -> Self {
            QpskMod {
                sender: Default::default(),
            }
        }

        pub fn run(&mut self) -> Result<Vec<Complex<f32>>, NodeError> {
            let dist = Uniform::new(0u8, 2u8);
            let mut rng = rand::thread_rng();
            let mut bits: Vec<u8> = Vec::new();
            for _ in 0..4096 {
                bits.push(rng.sample(&dist));
            }
            let qpsk_mod: Vec<Complex<f32>> = bits
                .iter()
                .step_by(2)
                .zip(bits.iter().skip(1).step_by(2))
                .map(|(x, y)| {
                    Complex::new(
                        f32::from(*x) * 2.0 - 1.0,
                        *y as f32 * 2.0 - 1.0,
                    )
                })
                .collect();
            let mut upsample = vec![Complex::zero(); 4096 * 2];
            let mut ix = 0;
            for samp in qpsk_mod {
                upsample[ix] = samp;
                ix += 4;
            }
            let sam_per_sym = 4.0;
            let taps: Vec<Complex<f32>> =
                math::rrc_taps(32, sam_per_sym, 0.25).unwrap();
            let mut state: Vec<Complex<f32>> = vec![Complex::zero(); 32];
            let data = fir::batch_fir(&upsample, &taps, &mut state);
            Ok(data)
        }
    }
    let mut qpsk_mod_node = QpskMod::new();
    let mut zmq_out =
        ZMQSend::new("ipc://127.0.0.1:57324", zmq::SocketType::PUSH, 0);

    connect_nodes!(qpsk_mod_node, sender, zmq_out, input);
    start_nodes!(qpsk_mod_node, zmq_out);

    thread::sleep(time::Duration::from_millis(1000));
}
