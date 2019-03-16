use crate::node::Node;
use hashbrown::HashMap;
use std::sync::{Arc, Mutex};
use std::thread;
use std::thread::JoinHandle;

/// The basics of a data structure to hold nodes and their thread handles
/// after starting the graph. Currently, this does not support connecting the
/// nodes; nodes need to be connected before passing them to the graph at the
/// moment.
pub struct Graph {
    nodes: HashMap<String, Arc<Mutex<dyn Node>>>,
    handles: Vec<JoinHandle<()>>,
}

impl Graph {
    pub fn new() -> Self {
        Graph {
            nodes: HashMap::new(),
            handles: vec![],
        }
    }

    pub fn add_node(&mut self, name: String, node: Arc<Mutex<dyn Node>>) {
        self.nodes.insert(name, node);
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
