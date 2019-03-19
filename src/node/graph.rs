use crate::prelude::*;
use hashbrown::HashMap;
use std::sync::{Arc, Mutex};
use std::thread;
use std::thread::JoinHandle;
use uuid::Uuid;

/// The basics of a data structure to hold nodes and their thread handles
/// after starting the graph. Currently, this does not support connecting the
/// nodes; nodes need to be connected before passing them to the graph at the
/// moment.
#[derive(Default)]
pub struct Graph {
    nodes: HashMap<Uuid, Arc<Mutex<dyn Node>>>,
    handles: Vec<JoinHandle<()>>,
}

impl Graph {
    pub fn new() -> Self {
        Graph {
            nodes: HashMap::new(),
            handles: vec![],
        }
    }

    pub fn add_node(&mut self, node: Arc<Mutex<dyn Node>>) {
        self.nodes.insert(Uuid::new_v4(), node);
    }

    pub fn connect_nodes<T>(
        &self,
        sender: &mut NodeSender<T>,
        receiver: &mut NodeReceiver<T>,
        default: Option<T>,
    ) {
        let (send, recv) = channel::bounded(0);
        sender.push((send, default));
        *receiver = Some(recv);
    }

    pub fn is_connected(&self) -> bool {
        for (_, node) in self.nodes.iter() {
            let lock = node.clone();
            let node = lock.lock().unwrap();
            if !node.is_connected() {
                return false;
            }
        }
        return true;
    }

    /// Start up all of the nodes in the graph one by one and keep track of
    /// the handles.
    pub fn run_graph(&mut self) {
        for (_, node) in self.nodes.iter() {
            let lock = node.clone();
            self.handles.push(thread::spawn(move || {
                let mut node = lock.lock().unwrap();
                node.start();
            }));
        }
    }
}
