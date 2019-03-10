#[macro_use]
extern crate comms_rs;
extern crate crossbeam;
extern crate rodio;
use comms_rs::io::audio::AudioNode;
use comms_rs::prelude::*;
use crossbeam::channel;
use rodio::source::{self, Source};
use std::boxed::Box;
use std::thread;

fn main() {
    let mut audio: AudioNode<f32> = AudioNode::new(1, 48000, 0.5);

    #[derive(Node)]
    struct SineNode {
        source: Box<dyn Source<Item = f32> + Send>,
        pub sender: NodeSender<Vec<f32>>,
    }

    impl SineNode {
        pub fn new(source: Box<dyn Source<Item = f32> + Send>) -> Self {
            SineNode {
                source,
                sender: Default::default(),
            }
        }

        pub fn run(&mut self) -> Result<Vec<f32>, NodeError> {
            let source = &mut self.source;
            let samp: Vec<f32> = source.take(48000).collect();
            Ok(samp)
        }
    }

    let mut sine = SineNode::new(Box::new(source::SineWave::new(440)));

    connect_nodes!(sine, sender, audio, input);
    start_nodes!(sine, audio);
    loop {}
}
