#[macro_use]
extern crate comms_rs;
extern crate rodio;
use comms_rs::io::jack_node::JackOutputNode;
use comms_rs::prelude::*;
use rodio::source::Source;
use std::f32::consts::PI;
use std::time::Duration;

#[derive(Clone, Debug)]
pub struct SineWaveArb {
    freq: f32,
    sample_rate: u32,
    num_sample: usize,
}

impl SineWaveArb {
    #[inline]
    pub fn new(freq: u32, sample_rate: u32) -> SineWaveArb {
        SineWaveArb {
            freq: freq as f32,
            sample_rate: sample_rate,
            num_sample: 0,
        }
    }
}

impl Iterator for SineWaveArb {
    type Item = f32;

    #[inline]
    fn next(&mut self) -> Option<f32> {
        self.num_sample = self.num_sample.wrapping_add(1);
        let value = 2.0 * PI * self.freq * self.num_sample as f32
            / (self.sample_rate as f32);
        Some(value.sin())
    }
}

impl Source for SineWaveArb {
    #[inline]
    fn current_frame_len(&self) -> Option<usize> {
        None
    }

    #[inline]
    fn channels(&self) -> u16 {
        1
    }

    #[inline]
    fn sample_rate(&self) -> u32 {
        self.sample_rate
    }

    #[inline]
    fn total_duration(&self) -> Option<Duration> {
        None
    }
}

fn main() {
    let mut jack_output: JackOutputNode = JackOutputNode::new();

    #[derive(Node)]
    struct SineNode {
        source: Box<dyn Source<Item = f32> + Send>,
        pub output: NodeSender<f32>,
    }

    impl SineNode {
        pub fn new(source: Box<dyn Source<Item = f32> + Send>) -> Self {
            SineNode {
                source,
                output: Default::default(),
            }
        }

        pub fn run(&mut self) -> Result<f32, NodeError> {
            let source = &mut self.source;
            if let Some(samp) = source.next() {
                Ok(samp)
            } else {
                Ok(0.0)
            }
        }
    }

    let mut sine = SineNode::new(Box::new(SineWaveArb::new(440, 44100)));

    connect_nodes!(sine, output, jack_output, input);
    start_nodes!(sine, jack_output);
    loop {}
}
