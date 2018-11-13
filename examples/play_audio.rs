#[macro_use]
extern crate comms_rs;
extern crate crossbeam;
extern crate rodio;
use comms_rs::io::audio::{self, AudioNode};
use comms_rs::prelude::*;
use crossbeam::{channel, Sender};
use rodio::source::{self, Source};
use std::boxed::Box;
use std::thread;

fn main() {
    let mut audio: AudioNode<f32> = audio::audio(1, 48000, 0.5);

    create_node!(
        SineNode: Vec<f32>,
        [source: Box<dyn Source<Item = f32> + Send>],
        [],
        |node: &mut SineNode| -> Result<Vec<f32>, Error> {
            let source = &mut node.source;
            let samp: Vec<f32> = source.take(48000).collect();
            Ok(samp)
        }
    );

    let mut sine = SineNode::new(Box::new(source::SineWave::new(440)));

    connect_nodes!(sine, audio, recv);
    start_nodes!(sine, audio);
    loop {}
}
