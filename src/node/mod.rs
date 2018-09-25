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
//! // inputs, the receivers must explicitly be named.
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

/// The trait that all nodes in the library implement. Only contains a single
/// function: `call(&mut self)` which executes the function in the node once.
pub trait Node {
    fn call(&mut self);
}

/// Creates a structure with crossbeam senders and receivers automatically and
/// auto-implements the Node trait.
///
/// # Arguments
///
/// create_node!(name: out_type, [fields: field_type], [recv: recv_type], func);
///
/// - name: The name of the node to construct.
/// - out_type: The type the node outputs, can be () if the node doesn't send
///   anything to another node.
/// - [fields: field_type]: A list of fields with their types to add to the
///   structure. This is useful for maintaining state within the structure.
/// - [recv: recv_type]: A list of receiver field names to add to the structure
///   along with the type.
/// - func: The function this node executes upon executing `call()`. The
///   function must accept a mutable reference to the node being constructed as
///   its first parameter, but if the function doesn't need to access state the
///   parameter can be safely be ignored.
///
/// Generics can be passed along with the following format:
///
/// create_generic_node!(name<generic>: out_type (where generic: Trait + ...,), [fields: field_type], [recv: recv_type], func);
/// - generic: any generic variables to add to the structure
/// - where generic: Trait + ...,: a list of trait bounds for the structure,
///   trait bounds are optional
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
/// // Create a node named CounterNode that receives an i32 input on the
/// // receiver named `recv`, and adds it to a `count` field in the structure.
/// // The current count is outputted from the node.
/// create_node!(CounterNode: i32,
///     [count: i32],
///     [recv: i32],
///     |node: &mut CounterNode, val: i32| {
///         node.count += val;
///         node.count
///     }
/// );
/// # }
/// ```
#[macro_export]
macro_rules! create_node {
    ($name:ident: Option<$out:ty>, [$($state:ident: $type:ty),*], [$($recv:ident: $in:ty),*], $func:expr) => {
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
            generate_new!($name, [$($state: $type),*], [$($recv),*]);
        }

        impl Node for $name
        {
            generate_aggregate_call!($func, $($recv),*);
        }
    };

    ($name:ident<$($gen:ident),+>: Option<$out:ty>, [$($state:ident: $type:ty),*],
     [$($recv:ident: $in:ty),*], $func:expr) => {
        pub struct $name<$($gen,)+> {
            $(
                pub $recv: Option<Receiver<$in>>,
            )*
            pub sender: Vec<Sender<$out>>,
            $(
                pub $state: $type,
            )*
        }

        impl<$($gen,)*> $name<$($gen,)+> {
            generate_new!($name<$($gen),+>, [$($state: $type),*], [$($recv),*]);
        }

        impl<$($gen,)*> Node for $name<$($gen,)+>
        {
            generate_aggregate_call!($func, $($recv),*);
        }
    };

    ($name:ident<$($gen:ident),+>: Option<$out:ty> where $($gen_t:ident: $where:ident $(+ $where_rep:ident),*),+,
     [$($state:ident: $type:ty),*], [$($recv:ident: $in:ty),*], $func:expr) => {
        pub struct $name<$($gen,)+>
        where $( $gen_t: $where $(+ ($where_rep))*, )+
        {
            $(
                pub $recv: Option<Receiver<$in>>,
            )*
            pub sender: Vec<Sender<$out>>,
            $(
                pub $state: $type,
            )*
        }

        impl<$($gen,)*> $name<$($gen,)+>
        where $( $gen_t: $where $(+ ($where_rep))*, )+
        {
            generate_new!($name<$($gen),+>, [$($state: $type),*], [$($recv),*]);
        }

        impl<$($gen,)*> Node for $name<$($gen,)+>
        where $( $gen_t: $where $(+ ($where_rep))*, )+
        {
            generate_aggregate_call!($func, $($recv),*);
        }
    };
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
            generate_new!($name, [$($state: $type),*], [$($recv),*]);
        }

        impl Node for $name
        {
            generate_call!($func, $($recv),*);
        }
    };

    ($name:ident<$($gen:ident),+>: $out:ty, [$($state:ident: $type:ty),*],
     [$($recv:ident: $in:ty),*], $func:expr) => {
        pub struct $name<$($gen,)+> {
            $(
                pub $recv: Option<Receiver<$in>>,
            )*
            pub sender: Vec<Sender<$out>>,
            $(
                pub $state: $type,
            )*
        }

        impl<$($gen,)*> $name<$($gen,)+> {
            generate_new!($name<$($gen),+>, [$($state: $type),*], [$($recv),*]);
        }

        impl<$($gen,)*> Node for $name<$($gen,)+>
        {
            generate_call!($func, $($recv),*);
        }
    };

    ($name:ident<$($gen:ident),+>: $out:ty where $($gen_t:ident: $where:ident $(+ $where_rep:ident),*),+,
     [$($state:ident: $type:ty),*], [$($recv:ident: $in:ty),*], $func:expr) => {
        pub struct $name<$($gen,)+>
        where $( $gen_t: $where $(+ ($where_rep))*, )+
        {
            $(
                pub $recv: Option<Receiver<$in>>,
            )*
            pub sender: Vec<Sender<$out>>,
            $(
                pub $state: $type,
            )*
        }

        impl<$($gen,)*> $name<$($gen,)+>
        where $( $gen_t: $where $(+ ($where_rep))*, )+
        {
            generate_new!($name<$($gen),+>, [$($state: $type),*], [$($recv),*]);
        }

        impl<$($gen,)*> Node for $name<$($gen,)+>
        where $( $gen_t: $where $(+ ($where_rep))*, )+
        {
            generate_call!($func, $($recv),*);
        }
    };
}

#[doc(hidden)]
#[macro_export]
macro_rules! generate_call {
    ($func:expr, $($recv:ident),*) => {
        fn call(&mut self) {
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
}

#[doc(hidden)]
#[macro_export]
macro_rules! generate_aggregate_call {
    ($func:expr, $($recv:ident),*) => {
        fn call(&mut self) {
            $(
                let $recv = match self.$recv {
                    Some(ref r) => r.recv().unwrap(),
                    None => return,
                };
            )*
            if let Some(res) = ($func)(&mut *self, $($recv,)*) {
                for send in &self.sender {
                    send.send(res.clone());
                }
            }
        }
    }
}

#[doc(hidden)]
#[macro_export]
macro_rules! generate_new {
    ($name:ident, [$($state:ident: $type:ty),*], [$($recv:ident),*]) => {
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
    };
    ($name:ident<$($gen:ident),*>, [$($state:ident: $type:ty),*], [$($recv:ident),*]) => {
        pub fn new($($state: $type,)*) -> $name<$($gen,)*> {
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
                    $node.call();
                }
            });
        )*
    }
}

#[cfg(test)]
mod test {
    use crossbeam::{Receiver, Sender};
    use crossbeam_channel as channel;
    use node::Node;
    use rand::{thread_rng, Rng};
    use std::sync::Arc;
    use std::thread;
    use std::time::{Duration, Instant};

    #[test]
    /// Constructs a simple network with two nodes: one source and one sink.
    fn test_simple_nodes() {
        create_node!(Node1: u32, [], [], { |_| 1 });
        create_node!(Node2: (), [], [recv1: u32], { |_, x| assert_eq!(x, 1) });

        let mut node1 = Node1::new();
        let mut node2 = Node2::new();

        connect_nodes!(node1, node2, recv1);
        start_nodes!(node1);
        let check = thread::spawn(move || {
            let now = Instant::now();
            loop {
                node2.call();
                if now.elapsed().as_secs() >= 1 {
                    break;
                }
            }
        });
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
        create_node!(
            Node1: Option<Arc<Vec<u32>>>,
            [agg: Vec<u32>],
            [],
            |node: &mut Node1| if node.agg.len() < 2 {
                node.agg.push(1);
                None
            } else {
                let val = node.agg.clone();
                node.agg = vec![];
                Some(Arc::new(val))
            }
        );
        create_node!(
            Node2: Option<Arc<Vec<u32>>>,
            [],
            [recv1: Arc<Vec<u32>>],
            |_, x: Arc<Vec<u32>>| {
                let mut y = Arc::clone(&x);
                for z in Arc::make_mut(&mut y).iter_mut() {
                    *z = *z + 1;
                }
                Some(y)
            }
        );
        create_node!(
            Node3: (),
            [],
            [recv2: Arc<Vec<u32>>],
            |_, x: Arc<Vec<u32>>| {
                assert_eq!(*x, vec![2, 2]);
            }
        );

        let mut node1 = Node1::new(Vec::new());
        let mut node2 = Node2::new();
        let mut node3 = Node3::new();

        connect_nodes!(node1, node2, recv1);
        connect_nodes!(node2, node3, recv2);
        start_nodes!(node1, node2);
        let check = thread::spawn(move || {
            let now = Instant::now();
            loop {
                node3.call();
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
        create_node!(Node1: Arc<Vec<i16>>, [], [], |_| {
            let mut random = vec![0i16; 10000];
            thread_rng().fill(random.as_mut_slice());
            Arc::new(random)
        });
        create_node!(
            Node2: Arc<Vec<i16>>,
            [],
            [recv1: Arc<Vec<i16>>],
            |_, x: Arc<Vec<i16>>| {
                let mut y = Arc::clone(&x);
                for z in Arc::make_mut(&mut y).iter_mut() {
                    *z = z.saturating_add(1);
                }
                y
            }
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

        let mut node1 = Node1::new();
        let mut node2 = Node2::new();
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
            |_, x: u32, y: f64| (x as f64 + y) as f32
        );

        // Create a node to check the value.
        create_node!(CheckNode: (), [], [recv: f32], |_, x: f32| {
            assert_eq!(x, 3.0, "Node didn't work!");
        });

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
            let now = Instant::now();
            loop {
                node4.call();
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
        create_node!(OneNode: i32, [count: i32], [], |node: &mut OneNode| {
            node.count += 1;
            node.count
        });

        create_node!(
            CounterNode: i32,
            [count: i32],
            [recv: i32],
            |node: &mut CounterNode, val: i32| {
                node.count = node.count + val;
                node.count
            }
        );

        let mut one_node = OneNode::new(0);
        let mut count_node = CounterNode::new(0);
        connect_nodes!(one_node, count_node, recv);

        thread::spawn(move || {
            for _ in 0..10 {
                one_node.call();
            }
        });

        let check = thread::spawn(move || {
            for _ in 0..10 {
                count_node.call();
            }
            assert_eq!(count_node.count, 55);
        });

        assert!(check.join().is_ok());
    }
}
