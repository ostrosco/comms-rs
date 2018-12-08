//! Nodes for general input/output support, such as file IO, audio, and ZeroMQ.

#[cfg(feature = "zmq_node")]
extern crate zmq;

#[cfg(feature = "audio_node")]
extern crate rodio;

#[cfg(feature = "audio_node")]
pub mod audio;

#[cfg(feature = "zmq_node")]
pub mod zmq_node;

pub mod raw_iq;
