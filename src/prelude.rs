//! This module provides an easy single import for those using this crate.

pub use crossbeam::{channel, Receiver, Sender};
pub use crate::node::Node;
pub use crate::node::NodeError;
pub use std::thread;
