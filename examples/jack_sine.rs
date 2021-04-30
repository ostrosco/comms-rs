#[macro_use]
extern crate comms_rs;
extern crate rodio;
use comms_rs::io::jack_node::JackOutputNode;
use comms_rs::prelude::*;
use rodio::source::{self, Source};
use std::sync::Arc;

fn main() {

    let mut jack_output: JackOutputNode = JackOutputNode::new();

    #[derive(Node)]
    struct SineNode {
        source: Box<dyn Source<Item = f32> + Send>,
        pub output: NodeSender<Vec<f32>>,
    }

    impl SineNode {
        pub fn new(source: Box<dyn Source<Item = f32> + Send>) -> Self {
            SineNode {
                source,
                output: Default::default(),
            }
        }

        pub fn run(&mut self) -> Result<Vec<f32>, NodeError> {
            let source = &mut self.source;
            let samp: Vec<f32> = source.take(44100).collect();
            Ok(samp)
        }
    }

    let mut sine = SineNode::new(Box::new(source::SineWave::new(440)));

    //connect_nodes!(sine, output, jack_output, input);
    let (send, recv) = channel::unbounded();
    sine.output.push((send, None));
    jack_output.input = Arc::new(Some(recv));
    start_nodes!(sine, jack_output);
    loop {}
}
