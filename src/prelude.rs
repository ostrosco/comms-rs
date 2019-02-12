//! This module provides an easy single import for those using this crate.

pub use crate::node::Node;
pub use crate::node::NodeError;
pub use crossbeam::{channel, Receiver, Sender};
pub use std::thread;
pub use node_derive::node_derive;

pub type NodeReceiver<T> = Option<Receiver<T>>;
pub type NodeSender<T> = Vec<(Sender<T>, Option<T>)>;
