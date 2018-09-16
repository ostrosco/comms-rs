pub trait Node {
    fn run_node(&mut self);
}

/// Creates a base node with a variable number of receivers, a variable
/// number of senders, and a transformation function that maps the inputs
/// into the outputs. Channels are assumed to use the crossbeam crate.
///
/// # Examples
///
/// ```
/// #[macro_use] extern crate comms_rs;
/// use comms_rs::node::Node;
/// use comms_rs::{channel, Receiver, Sender};
/// # fn main() {
///
/// // Creates a node that takes no inputs and returns a value.
/// create_node!(NoInputNode, Fn() -> u32);
///
/// // Creates a node that takes a u32 and a f64, returns a f32, and names
/// // the receivers recv_u and recv_f.
/// create_node!(DoubleInputNode, FnMut(u32, f64) -> f32, recv_u, recv_f);
///
/// // Creates a node that takes one input and returns nothing.
/// create_node!(NoOutputNode, Fn(u32) -> (), recv_u);
/// # }
/// ```
#[macro_export]
macro_rules! create_node {
    ($name:ident, FnMut() -> $out:ty) => {
        struct $name<F>
        where
            F: FnMut() -> $out,
        {
            sender: Vec<Sender<$out>>,
            func: F,
        }

        impl<F> $name<F>
        where
            F: FnMut() -> $out,
        {
            fn new(func: F) -> $name<F> {
                $name {
                    sender: vec![],
                    func
                }
            }
        }

        impl<F> Node for $name<F> 
        where
            F: FnMut() -> $out,
        {
            fn run_node(&mut self) {
                let res = (self.func)();
                for send in &self.sender {
                    send.send(res);
                }
            }
        }
    };
    ($name:ident, Fn() -> $out:ty) => {
        struct $name<F>
        where
            F: Fn() -> $out,
        {
            sender: Vec<Sender<$out>>,
            func: F,
        }

        impl<F> $name<F>
        where
            F: Fn() -> $out,
        {
            fn new(func: F) -> $name<F> {
                $name {
                    sender: vec![],
                    func
                }
            }
        }

        impl<F> Node for $name<F> 
        where
            F: Fn() -> $out,
        {
            fn run_node(&mut self) {
                let res = (self.func)();
                for send in &self.sender {
                    send.send(res);
                }
            }
        }
    };
    ($name:ident, FnMut($($in:ty),+) -> $out:ty, $($recv:ident),+) => {
        struct $name<F>
        where
            F: FnMut($($in),+) -> $out,
        {
            $(
                $recv: Option<Receiver<$in>>,
            )*
            sender: Vec<Sender<$out>>,
            func: F,
        }

        impl<F> $name<F>
        where
            F: FnMut($($in),+) -> $out,
        {
            fn new(func: F) -> $name<F> {
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
            F: FnMut($($in),+) -> $out,
        {
            fn run_node(&mut self) {
                $(
                    let $recv = match self.$recv {
                        Some(ref r) => r.recv().unwrap(),
                        None => return,
                    };
                )*
                let res = (self.func)($($recv,)+);
                for send in &self.sender {
                    send.send(res);
                }
            }
        }
    };
    ($name:ident, Fn($($in:ty),+) -> $out:ty, $($recv:ident),+) => {
        struct $name<F>
        where
            F: Fn($($in),+) -> $out,
        {
            $(
                $recv: Option<Receiver<$in>>,
            )*
            sender: Vec<Sender<$out>>,
            func: F,
        }

        impl<F> $name<F>
        where
            F: Fn($($in),+) -> $out,
        {
            fn new(func: F) -> $name<F> {
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
            F: Fn($($in),+) -> $out,
        {
            fn run_node(&mut self) {
                $(
                    let $recv = match self.$recv {
                        Some(ref r) => r.recv().unwrap(),
                        None => return,
                    };
                )*
                let res = (self.func)($($recv,)+);
                for send in &self.sender {
                    send.send(res);
                }
            }
        }
    };
}

/// Connects two nodes together with crossbeam channels.
///
/// # Exapmles
///
/// ```
/// #[macro_use] extern crate comms_rs;
/// use comms_rs::node::Node;
/// use comms_rs::{channel, Receiver, Sender};
/// # fn main() {
/// create_node!(Node1, Fn() -> u32);
/// create_node!(Node2, Fn(u32) -> (), recv);
///
/// let mut node1 = Node1::new(|| { 1 });
/// let mut node2 = Node2::new(|x| { assert_eq!(x, 1) });
/// connect_nodes!(node1, node2, recv);
/// # }
/// ```
///
#[macro_export]
macro_rules! connect_nodes {
    ($n1:ident, $n2:ident, $recv:ident) => {
        {
            let (send, recv) = channel::unbounded();
            $n1.sender.push(send);
            $n2.$recv = Some(recv);
        }
    }
}

/// Spawns a thread for each node in order and starts nodes to run
/// indefinitely.
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
    fn test_simple_nodes() {
        use node::Node;
        use crossbeam_channel as channel;
        use std::thread;
        use crossbeam::{Sender, Receiver};

        create_node!(Node1, Fn() -> u32);
        create_node!(Node2, Fn(u32) -> (), recv1);

        let mut node1 = Node1::new(|| { 1 });
        let mut node2 = Node2::new(|x| { assert_eq!(x, 1); });

        connect_nodes!(node1, node2, recv1);
        start_nodes!(node1, node2);
    }

    #[test]
    fn test_fan_in() {
        use crossbeam::{Receiver, Sender};
        use crossbeam_channel as channel;
        use std::thread;
        use node::Node;

        // Creates a node that takes no inputs and returns a value.
        create_node!(NoInputNode, Fn() -> u32);
        create_node!(AnotherNode, Fn() -> f64);

        // Creates a node that takes a u32 and a f64, returns a f32, and names
        // the receivers recv_u and recv_f.
        create_node!(DoubleInputNode, FnMut(u32, f64) -> f32, recv1, recv2);

        // Create a node to check the value.
        create_node!(CheckNode, Fn(f32) -> (), recv);

        // Now, you can instantiate your nodes as usual.
        let mut node1 = NoInputNode::new(|| 1);
        let mut node2 = AnotherNode::new(|| 2.0);
        let mut node3 = DoubleInputNode::new(|x, y| (x as f64 + y) as f32);
        let mut node4 = CheckNode::new(|x| {
            assert_eq!(x, 3.0, "Node didn't work!");
        });

        // Once you have your nodes, you can construct receivers and senders
        // to connect the nodes to one another.
        connect_nodes!(node1, node3, recv1);
        connect_nodes!(node2, node3, recv2);
        connect_nodes!(node3, node4, recv);

        // Lastly, start up your nodes.
        start_nodes!(node1, node2, node3, node4);
    }
}
