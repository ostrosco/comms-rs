//! Provides an infrastructure to create processing nodes, connect nodes
//! together via crossbeam channels, and start nodes running in their own
//! independent threads.
//!
//! # Example
//!
//! ```
//! #[macro_use] extern crate comms_rs;
//! use comms_rs::node::Node;
//! use comms_rs::{channel, Receiver, Sender};
//! use std::thread;
//!
//! # fn main() {
//! // Creates two nodes: a source and a sink node. For nodes that receive
//! // inputs, the inputs must explicitly be named.
//! create_node!(Node1: u32, [], [], { |_| 1 });
//! create_node!(Node2: (), [], [recv: u32], { |_, x| assert_eq!(x, 1) });
//!
//! // Now that the structures are created, the user can now instantiate their
//! // nodes and pass in closures for the nodes to execute.
//! let mut node1 = Node1::new();
//! let mut node2 = Node2::new();
//!
//! // Create a connection between two nodes: node1 sending messages and node2
//! // receiving on the `recv` receiver in the Node2 structure.
//! connect_nodes!(node1, node2, recv);
//!
//! // Spawn threads for node1 and node2 and have them executing indefinitely.
//! start_nodes!(node1, node2);
//! # }
//! ```

pub trait Node {
    fn run_node(&mut self);
}

/// Creates a structure with crossbeam senders and receivers automatically and
/// auto-implements the Node trait.
///
/// This macro takes in the following:
/// - the name of the structure to create along with the return type of the node
/// - a list of identifiers with types to add ad fields to the structure
/// - the names of the crossbeam receivers along with receive types
/// - the function to run when `run_node` is called
///
/// Note that the input function defined for this node must take n+1 parameters
/// as &mut self is passed into each function to handle potential state.
///
/// # Examples
///
/// ```
/// # #[macro_use] extern crate comms_rs;
/// # use comms_rs::node::Node;
/// # use comms_rs::{channel, Receiver, Sender};
/// # fn main() {
/// // Creates a node that takes no inputs and outputs a u32.
/// create_node!(
///     NoInputNode: u32,
///     [],
///     [],
///     |_| 1
/// );
///
/// // Creates a node that outputs a f32 and receives a u32 and an f64.
/// create_node!(
///     DoubleInputNode: f32,
///     [],
///     [recv_u: u32, recv_v: f64],
///     |_, x, y| (x as f64 + y) as f32
/// );
///
/// // Creates a node that takes one u32 input and outputs nothing.
/// create_node!(
///     NoOutputNode: (),
///     [],
///     [recv_u: u32],
///     |_, x| println!("{}", x)
/// );
///
/// // Creates a node with a field named count of type i32 that receives
/// // nothing and outputs an i32.
/// create_node!(NewFieldNode: i32,
///     [count: i32],
///     [],
///     |node: &mut NewFieldNode| {
///         node.count += 1;
///         node.count
///     }
/// );
/// # }
/// ```
#[macro_export]
macro_rules! create_node {
    ($name:ident: $out:ty, [$($state:ident: $type:ty),*], [$($recv:ident: $in:ty),*], $func:expr) => {
        pub struct $name {
            $(
                pub $recv: Option<Receiver<$in>>,
            )*
            pub sender: Vec<Sender<$out>>,
            $(
                pub $state: $type,
            )*
        }

        impl $name {
            pub fn new($($state: $type,)*) -> $name {
                $name {
                    $(
                        $recv: None,
                    )*
                    $(
                        $state,
                    )*
                    sender: vec![],
                }
            }
        }

        impl Node for $name
        {
            fn run_node(&mut self) {
                $(
                    let $recv = match self.$recv {
                        Some(ref r) => r.recv().unwrap(),
                        None => return,
                    };
                )*
                let res = ($func)(&mut *self, $($recv,)*);
                for send in &self.sender {
                    send.send(res.clone());
                }
            }
        }
    };
}

/// An aggregate node is a node that does not generate output each time it
/// receives an input. Instead, an aggregate node will generate output after
/// receiving multiple inputs. Output is only sent along a channel when
/// the output of the function is not None.
#[macro_export]
macro_rules! create_aggregate_node {
    ($name:ident, FnMut() -> Option<$out:ty>) => {
        pub struct $name<F>
        where
            F: FnMut() -> Option<$out>,
        {
            pub sender: Vec<Sender<$out>>,
            func: F,
        }

        impl<F> $name<F>
        where
            F: FnMut() -> Option<$out>,
        {
            pub fn new(func: F) -> $name<F> {
                $name {
                    sender: vec![],
                    func
                }
            }
        }

        impl<F> Node for $name<F>
        where
            F: FnMut() -> Option<$out>,
        {
            fn run_node(&mut self) {
                if let Some(res) = (self.func)() {
                    for send in &self.sender {
                        send.send(res.clone());
                    }
                }
            }
        }
    };
    ($name:ident, FnMut($($in:ty),+) -> Option<$out:ty>, $($recv:ident),+) => {
        pub struct $name<F>
        where
            F: FnMut($($in),+) -> Option<$out>,
        {
            $(
                pub $recv: Option<Receiver<$in>>,
            )*
            pub sender: Vec<Sender<$out>>,
            func: F,
        }

        impl<F> $name<F>
        where
            F: FnMut($($in),+) -> Option<$out>,
        {
            pub fn new(func: F) -> $name<F> {
                $name {
                    $(
                        $recv: None,
                    )*
                    sender: vec![],
                    func
                }
            }
        }

        impl<F> Node for $name<F>
        where
            F: FnMut($($in),+) -> Option<$out>,
        {
            fn run_node(&mut self) {
                $(
                    let $recv = match self.$recv {
                        Some(ref r) => r.recv().unwrap(),
                        None => return,
                    };
                )*
                if let Some(res) = (self.func)($($recv,)+) {
                    for send in &self.sender {
                        send.send(res.clone());
                    }
                }
            }
        }
    };
}

/// Connects two nodes together with crossbeam channels.
///
/// ```
/// # #[macro_use] extern crate comms_rs;
/// # use comms_rs::node::Node;
/// # use comms_rs::{channel, Receiver, Sender};
/// # fn main() {
/// # create_node!(Node1: u32, [], [], |_| 1);
/// # create_node!(Node2: (), [], [recv: u32], { |_, x| assert_eq!(x, 1) });
/// let mut node1 = Node1::new();
/// let mut node2 = Node2::new();
///
/// // node1 will now send its messages to node2. node2 will receive the
/// // message on its receiver named `recv`.
/// connect_nodes!(node1, node2, recv);
/// # }
/// ```
///
#[macro_export]
macro_rules! connect_nodes {
    ($n1:ident, $n2:ident, $recv:ident) => {{
        let (send, recv) = channel::bounded(0);
        $n1.sender.push(send);
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
/// # use comms_rs::node::Node;
/// # use comms_rs::{channel, Receiver, Sender};
/// # use std::thread;
/// # fn main() {
/// # create_node!(Node1: u32, [], [], |_| 1);
/// # create_node!(Node2: (), [], [recv: u32], |_, x| assert_eq!(x, 1));
/// # let mut node1 = Node1::new();
/// # let mut node2 = Node2::new();
/// # connect_nodes!(node1, node2, recv);
/// // Connect two nodes named node1 and node2. node1 will now send its
/// // messages to node2. node2 will receive the
/// // message on its receiver named `recv`.
/// start_nodes!(node1, node2);
/// # }
/// ```
#[macro_export]
macro_rules! start_nodes {
    ($($node:ident),+) => {
        $(
            thread::spawn(move || {
                loop {
                    $node.run_node();
                }
            });
        )*
    }
}

#[cfg(test)]
mod test {
    #[test]
    /// Constructs a simple network with two nodes: one source and one sink.
    fn test_simple_nodes() {
        use crossbeam::{Receiver, Sender};
        use crossbeam_channel as channel;
        use node::Node;
        use std::thread;
        use std::time::Duration;

        create_node!(Node1: u32, [], [], { |_| 1 });
        create_node!(Node2: (), [], [recv1: u32], { |_, x| assert_eq!(x, 1) });

        let mut node1 = Node1::new();
        let mut node2 = Node2::new();

        connect_nodes!(node1, node2, recv1);
        start_nodes!(node1);
        let check = thread::spawn(move || {
            node2.run_node();
        });
        thread::sleep(Duration::from_secs(1));
        assert!(check.join().is_ok());
    }

    #[test]
    /// Constructs a network with three nodes: two aggregating data and one
    /// simple node. Node1 is actually doing aggregation whereas Node2
    /// operates as a simple node but exists to ensure that there are no
    /// errors in the macro. This test also demonstrates how Arc can be
    /// used to pass around references through the channels much easier,
    /// saving on potential expensive copies.
    fn test_aggregate_nodes() {
        use crossbeam::{Receiver, Sender};
        use crossbeam_channel as channel;
        use node::Node;
        use std::sync::Arc;
        use std::thread;
        use std::time::Duration;

        create_aggregate_node!(Node1, FnMut() -> Option<Arc<Vec<u32>>>);
        create_aggregate_node!(
            Node2,
            FnMut(Arc<Vec<u32>>) -> Option<Arc<Vec<u32>>>,
            recv1
        );
        create_node!(
            Node3: (),
            [],
            [recv2: Arc<Vec<u32>>],
            {
                |_, x: Arc<Vec<u32>>| {
                    assert_eq!(*x, vec![2, 2]);
                }
            }
        );

        let mut agg = Vec::new();
        let mut node1 = Node1::new(move || {
            if agg.len() < 2 {
                agg.push(1);
                None
            } else {
                let val = agg.clone();
                agg = vec![];
                Some(Arc::new(val))
            }
        });
        let mut node2 = Node2::new(|x| {
            let mut y = Arc::clone(&x);
            for z in Arc::make_mut(&mut y).iter_mut() {
                *z = *z + 1;
            }
            Some(y)
        });
        let mut node3 = Node3::new();

        connect_nodes!(node1, node2, recv1);
        connect_nodes!(node2, node3, recv2);
        start_nodes!(node1, node2);
        let check = thread::spawn(move || {
            node3.run_node();
        });
        thread::sleep(Duration::from_secs(1));
        assert!(check.join().is_ok());
    }

    #[test]
    /// Performs a _very_ simplistic throughput analysis. We generate
    /// 10000 random i16 values at a time and pass it through the pipeline
    /// to see if channels will handle the throughput we hope it will.
    /// Make sure to run this test with --release.
    fn test_throughput() {
        use crossbeam::{Receiver, Sender};
        use crossbeam_channel as channel;
        use node::Node;
        use rand::{thread_rng, Rng};
        use std::sync::Arc;
        use std::thread;
        use std::time::Duration;

        create_aggregate_node!(Node1, FnMut() -> Option<Arc<Vec<i16>>>);
        create_aggregate_node!(
            Node2,
            FnMut(Arc<Vec<i16>>) -> Option<Arc<Vec<i16>>>,
            recv1
        );
        create_node!(
            Node3: (),
            [count: u32],
            [recv2: Arc<Vec<i16>>],
            |node: &mut Node3, _val: Arc<Vec<i16>>| {
                node.count = node.count + 1;
                if node.count == 40000 {
                    println!("Hit goal of 400 million i16 sent.");
                }
            }
        );

        let mut node1 = Node1::new(move || {
            let mut random = vec![0i16; 10000];
            thread_rng().fill(random.as_mut_slice());
            Some(Arc::new(random))
        });
        let mut node2 = Node2::new(|x| {
            let mut y = Arc::clone(&x);
            for z in Arc::make_mut(&mut y).iter_mut() {
                *z = z.saturating_add(1);
            }
            Some(y)
        });
        let mut node3 = Node3::new(0);

        connect_nodes!(node1, node2, recv1);
        connect_nodes!(node2, node3, recv2);
        start_nodes!(node1, node2, node3);
        thread::sleep(Duration::from_secs(1));
    }

    #[test]
    /// Constructs a network where a node receives from two different nodes.
    /// This serves to make sure that fan-in operation works as we expect
    /// it to.
    fn test_fan_in() {
        use crossbeam::{Receiver, Sender};
        use crossbeam_channel as channel;
        use node::Node;
        use std::thread;
        use std::time::Duration;

        // Creates a node that takes no inputs and returns a value.
        create_node!(NoInputNode: u32, [], [], { |_| 1 });
        create_node!(AnotherNode: f64, [], [], { |_| 2.0 });

        // Creates a node that takes a u32 and a f64, returns a f32, and names
        // the receivers recv_u and recv_f.
        create_node!(
            DoubleInputNode: f32,
            [],
            [recv1: u32, recv2: f64],
            |_, x: u32, y: f64| {
                (x as f64 + y) as f32
            }
        );

        // Create a node to check the value.
        create_node!(
            CheckNode: (),
            [],
            [recv: f32],
            |_, x: f32| {
                assert_eq!(x, 3.0, "Node didn't work!");
            }
        );

        // Now, you can instantiate your nodes as usual.
        let mut node1 = NoInputNode::new();
        let mut node2 = AnotherNode::new();
        let mut node3 = DoubleInputNode::new();
        let mut node4 = CheckNode::new();

        // Once you have your nodes, you can construct receivers and senders
        // to connect the nodes to one another.
        connect_nodes!(node1, node3, recv1);
        connect_nodes!(node2, node3, recv2);
        connect_nodes!(node3, node4, recv);

        // Lastly, start up your nodes.
        start_nodes!(node1, node2, node3);
        let check = thread::spawn(move || {
            node4.run_node();
        });
        thread::sleep(Duration::from_secs(1));
        assert!(check.join().is_ok());
    }
}
