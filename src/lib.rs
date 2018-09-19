extern crate crossbeam;
extern crate crossbeam_channel;

#[macro_use]
pub mod node;
pub mod modulation;
pub use crossbeam::{channel, Receiver, Sender};
