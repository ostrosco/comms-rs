//! Provides an infrastructure to create processing nodes, connect nodes
//! together via crossbeam channels, and start nodes running in their own
//! independent threads.
//!
//! # Example
//!
//! ```
//! #[macro_use] extern crate comms_rs;
//! use comms_rs::prelude::*;
//! use std::thread;
//!
//! // Creates two nodes: a source and a sink node. For nodes that receive
//! // inputs, the receivers must explicitly be named.
//! #[derive(Node)]
//! struct Node1 {
//!     pub output: NodeSender<u32>,
//! }
//!
//! impl Node1 {
//!     pub fn new() -> Self {
//!         Node1 {
//!             output: Default::default(),
//!         }
//!     }
//!
//!     pub fn run(&mut self) -> Result<u32, NodeError> {
//!         Ok(1)
//!     }
//! }
//!
//! #[derive(Node)]
//! struct Node2 {
//!     pub input: NodeReceiver<u32>,
//! }
//!
//! impl Node2 {
//!     pub fn new() -> Self {
//!         Node2 {
//!             input: Default::default(),
//!         }
//!     }
//!
//!     pub fn run(&mut self, x: u32) -> Result<(), NodeError> {
//!         assert_eq!(x, 1);
//!         Ok(())
//!     }
//! }
//!
//! // Now that the structures are created, the user can now instantiate their
//! // nodes and pass in closures for the nodes to execute.
//! let mut node1 = Node1::new();
//! let mut node2 = Node2::new();
//!
//! // Create a connection between two nodes: node1 sending messages and node2
//! // receiving on the `input` receiver in the Node2 structure.
//! connect_nodes!(node1, output, node2, input);
//!
//! // Spawn threads for node1 and node2 and have them executing indefinitely.
//! start_nodes!(node1, node2);
//! ```

pub mod graph;

use std::error;
use std::fmt;

#[derive(Clone, Debug)]
pub enum NodeError {
    DataError,
    PermanentError,
    DataEnd,
    CommError,
}

impl fmt::Display for NodeError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let desc = match *self {
            NodeError::DataError => "unable to access data",
            NodeError::PermanentError => "unable to continue executing node",
            NodeError::DataEnd => "end of data source",
            NodeError::CommError => "unable to establish comm channel",
        };
        write!(f, "Node error: {}", desc)
    }
}

impl error::Error for NodeError {
    fn cause(&self) -> Option<&dyn error::Error> {
        None
    }
}

/// The trait that all nodes in the library implement.
pub trait Node: Send {
    fn start(&mut self);
    fn call(&mut self) -> Result<(), NodeError>;
    fn is_connected(&self) -> bool;
}

/// Connects two nodes together with crossbeam channels.
///
/// ```
/// # #[macro_use] extern crate comms_rs;
/// # use comms_rs::prelude::*;
/// # fn main() {
/// # #[derive(Node)]
/// # struct Node1 {
/// #     output: NodeSender<u32>,
/// # }
/// #
/// # impl Node1 {
/// #   pub fn new() -> Self {
/// #       Node1 {
/// #           output: Default::default(),
/// #       }
/// #   }
/// #
/// #   pub fn run(&mut self) -> Result<u32, NodeError> {
/// #       Ok(1)
/// #   }
/// # }
/// #
/// # #[derive(Node)]
/// # struct Node2 {
/// #   input: NodeReceiver<u32>,
/// # }
/// #
/// # impl Node2 {
/// #   pub fn new() -> Self {
/// #       Node2 {
/// #           input: Default::default(),
/// #       }
/// #   }
/// #
/// #   pub fn run(&mut self, x: u32) -> Result<(), NodeError> {
/// #       assert_eq!(x, 1);
/// #       Ok(())
/// #   }
/// # }
/// let mut node1 = Node1::new();
/// let mut node2 = Node2::new();
///
/// // node1 will now send its messages to node2. node2 will receive the
/// // message on its receiver named `input`.
/// connect_nodes!(node1, output, node2, input);
/// # }
/// ```
///
#[macro_export]
macro_rules! connect_nodes {
    ($n1:ident, $send:ident, $n2:ident, $recv:ident) => {{
        let (send, recv) = channel::unbounded();
        $n1.$send.push((send, None));
        $n2.$recv = Some(recv);
    }};
}

/// Connects two nodes together in a feedback configuration using channels.
/// When the nodes are connected in feedback, a specified value is sent
/// through the channel immediately so that the nodes don't deadlock on
/// the first iteration.
///
/// ```
/// # #[macro_use] extern crate comms_rs;
/// # use comms_rs::prelude::*;
/// # fn main() {
/// # #[derive(Node)]
/// # struct Node1 {
/// #     output: NodeSender<u32>,
/// # }
/// #
/// # impl Node1 {
/// #   pub fn new() -> Self {
/// #       Node1 {
/// #           output: Default::default(),
/// #       }
/// #   }
/// #
/// #   pub fn run(&mut self) -> Result<u32, NodeError> {
/// #       Ok(1)
/// #   }
/// # }
/// #
/// # #[derive(Node)]
/// # struct Node2 {
/// #   input: NodeReceiver<u32>,
/// # }
/// #
/// # impl Node2 {
/// #   pub fn new() -> Self {
/// #       Node2 {
/// #           input: Default::default(),
/// #       }
/// #   }
/// #
/// #   pub fn run(&mut self, x: u32) -> Result<(), NodeError> {
/// #       assert_eq!(x, 1);
/// #       Ok(())
/// #   }
/// # }
/// let mut node1 = Node1::new();
/// let mut node2 = Node2::new();
///
/// // node1 will now send its messages to node2. node2 will receive the
/// // message on its receiver named `input`. When node1 starts, it will send
/// // a 0 to node2 the first time it's run by start_nodes! and run a normal
/// // loop afterwards.
/// connect_nodes_feedback!(node1, output, node2, input, 0);
/// # }
/// ```
///
#[macro_export]
macro_rules! connect_nodes_feedback {
    ($n1:ident, $send:ident, $n2:ident, $recv:ident, $default:tt) => {{
        let (send, recv) = channel::unbounded();
        $n1.$send.push((send, Some($default)));
        $n2.$recv = Some(recv);
    }};
}

/// Spawns a thread for each node in order and starts nodes to run
/// indefinitely.
///
/// # Example
///
/// ```
/// # #[macro_use] extern crate comms_rs;
/// # use comms_rs::prelude::*;
/// # use std::thread;
/// # fn main() {
/// # #[derive(Node)]
/// # struct Node1 {
/// #     pub output: NodeSender<u32>,
/// # }
///
/// # impl Node1 {
/// #     pub fn new() -> Self {
/// #         Node1 {
/// #             output: Default::default(),
/// #         }
/// #     }
/// #
/// #     pub fn run(&mut self) -> Result<u32, NodeError> {
/// #         Ok(1)
/// #     }
/// # }
///
/// # #[derive(Node)]
/// # struct Node2 {
/// #     pub input: NodeReceiver<u32>,
/// # }
///
/// # impl Node2 {
/// #     pub fn new() -> Self {
/// #         Node2 {
/// #             input: Default::default(),
/// #         }
/// #     }
/// #
/// #     pub fn run(&mut self, x: u32) -> Result<(), NodeError> {
/// #         assert_eq!(x, 1);
/// #         Ok(())
/// #    }
/// # }
/// # let mut node1 = Node1::new();
/// # let mut node2 = Node2::new();
/// # connect_nodes!(node1, output, node2, input);
///
/// // Connect two nodes named node1 and node2. node1 will now send its
/// // messages to node2. node2 will receive the
/// // message on its receiver named `input`.
/// start_nodes!(node1, node2);
/// # }
/// ```
#[macro_export]
macro_rules! start_nodes {
    ($($node:ident),+ $(,)?) => {
        $(
            thread::spawn(move || {
                $node.start();
            });
        )*
    }
}

/// Spawns a thread from a Rayon threadpool for each node in order and starts
/// nodes to run indefinitely.
///
/// # Example
///
/// ```
/// # #[macro_use] extern crate comms_rs;
/// # extern crate rayon;
/// # use comms_rs::prelude::*;
/// # use std::thread;
///
/// # fn main() {
/// # #[derive(Node)]
/// # struct Node1 {
/// #     pub output: NodeSender<u32>,
/// # }
///
/// # impl Node1 {
/// #     pub fn new() -> Self {
/// #         Node1 {
/// #             output: Default::default(),
/// #         }
/// #     }
/// #
/// #     pub fn run(&mut self) -> Result<u32, NodeError> {
/// #         Ok(1)
/// #     }
/// # }
///
/// # #[derive(Node)]
/// # struct Node2 {
/// #     pub input: NodeReceiver<u32>,
/// # }
///
/// # impl Node2 {
/// #     pub fn new() -> Self {
/// #         Node2 {
/// #             input: Default::default(),
/// #         }
/// #     }
/// #
/// #     pub fn run(&mut self, x: u32) -> Result<(), NodeError> {
/// #         assert_eq!(x, 1);
/// #         Ok(())
/// #    }
/// # }
/// # let mut node1 = Node1::new();
/// # let mut node2 = Node2::new();
/// # connect_nodes!(node1, output, node2, input);
/// // Connect two nodes named node1 and node2. node1 will now send its
/// // messages to node2. node2 will receive the
/// // message on its receiver named `input`.
/// start_nodes_threadpool!(node1, node2);
/// # }
/// ```
#[macro_export]
macro_rules! start_nodes_threadpool {
    ($($node:ident),+ $(,)?) => {
        $(
            rayon::spawn(move || {
                $node.start();
            });
        )*
    }
}

#[cfg(test)]
mod test {
    use rand::{thread_rng, Rng};
    use std::sync::{Arc, Mutex};
    use std::thread;
    use std::time::{Duration, Instant};

    use crate::node::graph::Graph;
    use crate::prelude::*;
    use rayon;

    #[test]
    /// Constructs a simple network with two nodes: one source and one sink.
    fn test_simple_nodes() {
        #[derive(Node)]
        struct Node1 {
            pub output: NodeSender<u32>,
        }

        impl Node1 {
            pub fn new() -> Self {
                Node1 {
                    output: Default::default(),
                }
            }

            pub fn run(&mut self) -> Result<u32, NodeError> {
                Ok(1)
            }
        }

        #[derive(Node)]
        struct Node2 {
            pub input: NodeReceiver<u32>,
        }

        impl Node2 {
            pub fn new() -> Self {
                Node2 {
                    input: Default::default(),
                }
            }

            pub fn run(&mut self, x: u32) -> Result<(), NodeError> {
                assert_eq!(x, 1);
                Ok(())
            }
        }

        let mut node1 = Node1::new();
        let mut node2 = Node2::new();

        connect_nodes!(node1, output, node2, input);
        start_nodes!(node1);
        let check = thread::spawn(move || {
            let now = Instant::now();
            loop {
                node2.call().unwrap();
                if now.elapsed().as_secs() >= 1 {
                    break;
                }
            }
        });
        assert!(check.join().is_ok());
    }

    #[test]
    /// Constructs a simple network with two nodes: one source and one sink.
    fn test_simple_graph() {
        #[derive(Node)]
        struct Node1 {
            pub output: NodeSender<u32>,
        }

        impl Node1 {
            pub fn new() -> Self {
                Node1 {
                    output: Default::default(),
                }
            }

            pub fn run(&mut self) -> Result<u32, NodeError> {
                Ok(1)
            }
        }

        #[derive(Node)]
        struct Node2 {
            pub input: NodeReceiver<u32>,
            pub check: Arc<Mutex<bool>>,
        }

        impl Node2 {
            pub fn new(check: Arc<Mutex<bool>>) -> Self {
                Node2 {
                    input: Default::default(),
                    check,
                }
            }

            pub fn run(&mut self, x: u32) -> Result<(), NodeError> {
                let mut check = self.check.lock().unwrap();
                *check = x == 1;
                Ok(())
            }
        }

        let check = Arc::new(Mutex::new(false));
        let node1 = Arc::new(Mutex::new(Node1::new()));
        let node2 = Arc::new(Mutex::new(Node2::new(check.clone())));

        let mut graph = Graph::new(None);
        graph.add_node(node1.clone());
        graph.add_node(node2.clone());
        {
            let mut node1 = node1.lock().unwrap();
            let mut node2 = node2.lock().unwrap();
            graph.connect_nodes(&mut node1.output, &mut node2.input, None);
        }
        assert!(graph.is_connected());
        graph.run_graph();
        thread::sleep(Duration::from_secs(1));
        {
            let check = check.lock().unwrap();
            assert!(*check);
        }
    }

    #[test]
    /// Constructs a network with three nodes: two aggregating data and one
    /// simple node. Node1 is actually doing aggregation whereas Node2
    /// operates as a simple node but exists to ensure that there are no
    /// errors in the macro. This test also demonstrates how Arc can be
    /// used to pass around references through the channels much easier,
    /// saving on potential expensive copies.
    fn test_aggregate_nodes() {
        #[derive(Node)]
        #[aggregate]
        struct Node1 {
            agg: Vec<u32>,
            pub output: NodeSender<Arc<Vec<u32>>>,
        }

        impl Node1 {
            pub fn new() -> Self {
                Node1 {
                    agg: vec![],
                    output: Default::default(),
                }
            }

            pub fn run(&mut self) -> Result<Option<Arc<Vec<u32>>>, NodeError> {
                if self.agg.len() < 2 {
                    self.agg.push(1);
                    Ok(None)
                } else {
                    let val = self.agg.clone();
                    self.agg = vec![];
                    Ok(Some(Arc::new(val)))
                }
            }
        }

        #[derive(Node)]
        #[pass_by_ref]
        struct Node2 {
            pub input: NodeReceiver<Arc<Vec<u32>>>,
            pub output: NodeSender<Arc<Vec<u32>>>,
        }

        impl Node2 {
            pub fn new() -> Self {
                Node2 {
                    input: Default::default(),
                    output: Default::default(),
                }
            }

            pub fn run(
                &mut self,
                input: &Arc<Vec<u32>>,
            ) -> Result<Arc<Vec<u32>>, NodeError> {
                let mut y = Arc::clone(input);
                for z in Arc::make_mut(&mut y).iter_mut() {
                    *z += 1;
                }
                Ok(y)
            }
        }

        #[derive(Node)]
        #[pass_by_ref]
        struct Node3 {
            pub input: NodeReceiver<Arc<Vec<u32>>>,
        }

        impl Node3 {
            pub fn new() -> Self {
                Node3 {
                    input: Default::default(),
                }
            }

            pub fn run(
                &mut self,
                input: &Arc<Vec<u32>>,
            ) -> Result<(), NodeError> {
                assert_eq!(**input, vec![2, 2]);
                Ok(())
            }
        }

        let mut node1 = Node1::new();
        let mut node2 = Node2::new();
        let mut node3 = Node3::new();

        connect_nodes!(node1, output, node2, input);
        connect_nodes!(node2, output, node3, input);
        start_nodes!(node1, node2);
        let check = thread::spawn(move || {
            let now = Instant::now();
            loop {
                node3.call().unwrap();
                if now.elapsed().as_secs() >= 1 {
                    break;
                }
            }
        });
        assert!(check.join().is_ok());
    }

    #[test]
    /// Performs a _very_ simplistic throughput analysis. We generate
    /// 10000 random i16 values at a time and pass it through the pipeline
    /// to see if channels will handle the throughput we hope it will.
    /// Make sure to run this test with --release.
    fn test_throughput() {
        #[derive(Node)]
        struct Node1 {
            pub output: NodeSender<Arc<Vec<i16>>>,
        }

        impl Node1 {
            pub fn new() -> Self {
                Node1 {
                    output: Default::default(),
                }
            }

            pub fn run(&mut self) -> Result<Arc<Vec<i16>>, NodeError> {
                let mut random = vec![0i16; 10000];
                thread_rng().fill(random.as_mut_slice());
                Ok(Arc::new(random))
            }
        }

        #[derive(Node)]
        #[pass_by_ref]
        struct Node2 {
            pub input: NodeReceiver<Arc<Vec<i16>>>,
            pub output: NodeSender<Arc<Vec<i16>>>,
        }

        impl Node2 {
            pub fn new() -> Self {
                Node2 {
                    input: Default::default(),
                    output: Default::default(),
                }
            }

            pub fn run(
                &mut self,
                x: &Arc<Vec<i16>>,
            ) -> Result<Arc<Vec<i16>>, NodeError> {
                let mut y = Arc::clone(x);
                for z in Arc::make_mut(&mut y).iter_mut() {
                    *z = z.saturating_add(1);
                }
                Ok(y)
            }
        }

        #[derive(Node)]
        #[pass_by_ref]
        struct Node3 {
            pub input: NodeReceiver<Arc<Vec<i16>>>,
            count: u32,
        }

        impl Node3 {
            pub fn new() -> Self {
                Node3 {
                    count: 0,
                    input: Default::default(),
                }
            }

            pub fn run(
                &mut self,
                _val: &Arc<Vec<i16>>,
            ) -> Result<(), NodeError> {
                self.count += 1;
                if self.count == 40000 {
                    println!(
                        "test_throughput: Hit goal of 400 million i16 sent."
                    );
                }
                Ok(())
            }
        }

        let mut node1 = Node1::new();
        let mut node2 = Node2::new();
        let mut node3 = Node3::new();

        connect_nodes!(node1, output, node2, input);
        connect_nodes!(node2, output, node3, input);
        start_nodes!(node1, node2, node3);
        thread::sleep(Duration::from_secs(1));
    }

    #[test]
    /// Performs a _very_ simplistic throughput analysis. We generate
    /// 10000 random i16 values at a time and pass it through the pipeline
    /// to see if channels will handle the throughput we hope it will.
    /// Make sure to run this test with --release.
    fn test_threadpool_throughput() {
        #[derive(Node)]
        struct Node1 {
            pub output: NodeSender<Arc<Vec<i16>>>,
        }

        impl Node1 {
            pub fn new() -> Self {
                Node1 {
                    output: Default::default(),
                }
            }

            pub fn run(&mut self) -> Result<Arc<Vec<i16>>, NodeError> {
                let mut random = vec![0i16; 10000];
                thread_rng().fill(random.as_mut_slice());
                Ok(Arc::new(random))
            }
        }

        #[derive(Node)]
        #[pass_by_ref]
        struct Node2 {
            pub input: NodeReceiver<Arc<Vec<i16>>>,
            pub output: NodeSender<Arc<Vec<i16>>>,
        }

        impl Node2 {
            pub fn new() -> Self {
                Node2 {
                    input: Default::default(),
                    output: Default::default(),
                }
            }

            pub fn run(
                &mut self,
                x: &Arc<Vec<i16>>,
            ) -> Result<Arc<Vec<i16>>, NodeError> {
                let mut y = Arc::clone(x);
                for z in Arc::make_mut(&mut y).iter_mut() {
                    *z = z.saturating_add(1);
                }
                Ok(y)
            }
        }

        #[derive(Node)]
        #[pass_by_ref]
        struct Node3 {
            pub input: NodeReceiver<Arc<Vec<i16>>>,
            count: u32,
        }

        impl Node3 {
            pub fn new() -> Self {
                Node3 {
                    count: 0,
                    input: Default::default(),
                }
            }

            pub fn run(
                &mut self,
                _val: &Arc<Vec<i16>>,
            ) -> Result<(), NodeError> {
                self.count += 1;
                if self.count == 40000 {
                    println!(
                        "test_threadpool_throughput: Hit goal of 400 million i16 sent."
                    );
                }
                Ok(())
            }
        }

        let mut node1 = Node1::new();
        let mut node2 = Node2::new();
        let mut node3 = Node3::new();

        connect_nodes!(node1, output, node2, input);
        connect_nodes!(node2, output, node3, input);
        start_nodes!(node1, node3);
        start_nodes_threadpool!(node2,);
        thread::sleep(Duration::from_secs(1));
    }

    #[test]
    /// Constructs a network where a node receives from two different nodes.
    /// This serves to make sure that fan-in operation works as we expect
    /// it to.
    fn test_fan_in() {
        use crate::prelude::*;
        use std::thread;
        use std::time::Duration;

        // Creates a node that takes no inputs and returns a value.
        #[derive(Node)]
        struct NoInputNode {
            output: NodeSender<u32>,
        }

        impl NoInputNode {
            pub fn new() -> Self {
                NoInputNode {
                    output: Default::default(),
                }
            }

            pub fn run(&mut self) -> Result<u32, NodeError> {
                Ok(1)
            }
        }

        #[derive(Node)]
        struct AnotherNode {
            output: NodeSender<f64>,
        }

        impl AnotherNode {
            pub fn new() -> Self {
                AnotherNode {
                    output: Default::default(),
                }
            }

            pub fn run(&mut self) -> Result<f64, NodeError> {
                Ok(2.0)
            }
        }

        // Creates a node that takes a u32 and a f64, returns a f32, and names
        // the receivers recv_u and recv_f.
        #[derive(Node)]
        struct DoubleInputNode {
            input1: NodeReceiver<u32>,
            input2: NodeReceiver<f64>,
            output: NodeSender<f32>,
        }

        impl DoubleInputNode {
            pub fn new() -> Self {
                DoubleInputNode {
                    input1: Default::default(),
                    input2: Default::default(),
                    output: Default::default(),
                }
            }

            pub fn run(&mut self, x: u32, y: f64) -> Result<f32, NodeError> {
                Ok((f64::from(x) + y) as f32)
            }
        }

        #[derive(Node)]
        struct CheckNode {
            input: NodeReceiver<f32>,
        }

        impl CheckNode {
            pub fn new() -> Self {
                CheckNode {
                    input: Default::default(),
                }
            }

            pub fn run(&mut self, x: f32) -> Result<(), NodeError> {
                assert!(x - 3.0f32 < std::f32::EPSILON, "Fan-out failed.");
                Ok(())
            }
        }

        // Now, you can instantiate your nodes as usual.
        let mut node1 = NoInputNode::new();
        let mut node2 = AnotherNode::new();
        let mut node3 = DoubleInputNode::new();
        let mut node4 = CheckNode::new();

        // Once you have your nodes, you can construct receivers and senders
        // to connect the nodes to one another.
        connect_nodes!(node1, output, node3, input1);
        connect_nodes!(node2, output, node3, input2);
        connect_nodes!(node3, output, node4, input);

        // Lastly, start up your nodes.
        start_nodes!(node1, node2, node3,);
        let check = thread::spawn(move || {
            let now = Instant::now();
            loop {
                node4.call().unwrap();
                if now.elapsed().as_secs() >= 1 {
                    break;
                }
            }
        });
        thread::sleep(Duration::from_secs(1));
        assert!(check.join().is_ok());
    }

    #[test]
    /// A test to ensure that persistent state works within the nodes. Makes
    /// two nodes: one to send 1 to 10 and the other to add the number to a
    /// counter within the node.
    fn test_counter() {
        #[derive(Node)]
        struct OneNode {
            count: i32,
            output: NodeSender<i32>,
        }

        impl OneNode {
            pub fn new() -> Self {
                OneNode {
                    count: 0,
                    output: Default::default(),
                }
            }

            pub fn run(&mut self) -> Result<i32, NodeError> {
                self.count += 1;
                Ok(self.count)
            }
        }

        #[derive(Node)]
        struct CounterNode {
            input: NodeReceiver<i32>,
            count: i32,
            output: NodeSender<i32>,
        }

        impl CounterNode {
            pub fn new() -> Self {
                CounterNode {
                    count: 0,
                    input: Default::default(),
                    output: Default::default(),
                }
            }

            pub fn run(&mut self, val: i32) -> Result<i32, NodeError> {
                self.count += val;
                Ok(self.count)
            }
        }

        let mut one_node = OneNode::new();
        let mut count_node = CounterNode::new();
        connect_nodes!(one_node, output, count_node, input);

        thread::spawn(move || {
            for _ in 0..10 {
                one_node.call().unwrap();
            }
        });

        let check = thread::spawn(move || {
            for _ in 0..10 {
                count_node.call().unwrap();
            }
            assert_eq!(count_node.count, 55);
        });

        assert!(check.join().is_ok());
    }

    #[test]
    /// A test to verify that feedback will work.
    fn test_feedback() {
        #[derive(Node)]
        struct AddNode {
            input: NodeReceiver<i32>,
            count: i32,
            output: NodeSender<i32>,
        }

        impl AddNode {
            pub fn new() -> Self {
                AddNode {
                    count: 1,
                    input: Default::default(),
                    output: Default::default(),
                }
            }

            pub fn run(&mut self, val: i32) -> Result<i32, NodeError> {
                self.count += val;
                Ok(self.count)
            }
        }

        #[derive(Node)]
        struct PrintNode {
            input: NodeReceiver<i32>,
            count: i32,
            output: NodeSender<i32>,
        }

        impl PrintNode {
            pub fn new() -> Self {
                PrintNode {
                    count: 0,
                    input: Default::default(),
                    output: Default::default(),
                }
            }

            pub fn run(&mut self, val: i32) -> Result<i32, NodeError> {
                self.count = val;
                Ok(val)
            }
        }

        let mut add_node = AddNode::new();
        let mut print_node = PrintNode::new();
        connect_nodes!(add_node, output, print_node, input);
        connect_nodes_feedback!(print_node, output, add_node, input, 0);
        start_nodes!(add_node);
        let check = thread::spawn(move || {
            for (print, val) in &print_node.output {
                match val {
                    Some(v) => print.send(*v).unwrap(),
                    None => continue,
                }
            }
            for _ in 0..10 {
                print_node.call().unwrap();
            }
            assert_eq!(print_node.count, 512);
        });
        assert!(check.join().is_ok());
    }
}
