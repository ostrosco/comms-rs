extern crate crossbeam;
extern crate crossbeam_channel;
extern crate rand;

#[macro_use]
pub mod node;
pub use crossbeam::{channel, Receiver, Sender};
