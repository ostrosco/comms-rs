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
/// # fn main() {
/// use std::sync::mpsc::{channel, Receiver, Sender};
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

#[macro_export]
macro_rules! connect_nodes {
    ($n1:ident, $n2:ident, $recv:ident) => {
        {
            let (send, recv) = crossbeam_channel::unbounded();
            $n1.sender.push(send);
            $n2.$recv = Some(recv);
        }
    }
}

#[cfg(test)]
mod test {
    #[test]
    fn test_connect_nodes() {
        use node::Node;
        use crossbeam_channel;
        use std::thread;
        use crossbeam::{Sender, Receiver};

        create_node!(Node1, Fn() -> u32);
        create_node!(Node2, Fn(u32) -> (), recv1);

        let mut node1 = Node1::new(|| { 1 });
        let mut node2 = Node2::new(|x| { assert_eq!(x, 1); });

        connect_nodes!(node1, node2, recv1);
        let n1_handle = thread::spawn(move || {
            node1.run_node();
        });

        let n2_handle = thread::spawn(move || {
            node2.run_node();
        });

        n1_handle.join().unwrap();
        n2_handle.join().unwrap();
    }

    #[test]
    fn test_multi_node() {
        use crossbeam::{Receiver, Sender};
        use crossbeam_channel;
        use std::thread;
        use node::Node;

        // Creates a node that takes no inputs and returns a value.
        create_node!(NoInputNode, Fn() -> u32);
        create_node!(AnotherNode, Fn() -> f64);

        // Creates a node that takes a u32 and a f64, returns a f32, and names
        // the receivers recv_u and recv_f.
        create_node!(DoubleInputNode, FnMut(u32, f64) -> f32, recv_u, recv_f);

        // Create a node to check the value.
        create_node!(CheckNode, Fn(f32) -> (), recv_c);

        // Now, you can instantiate your nodes as usual.
        let mut node1 = NoInputNode::new(|| 1);
        let mut node2 = AnotherNode::new(|| 2.0);
        let mut node3 = DoubleInputNode::new(|x, y| (x as f64 + y) as f32);
        let mut node4 = CheckNode::new(|x| {
            assert_eq!(x, 3.0, "Node didn't work!");
        });

        // Once you have your nodes, you can construct receivers and senders
        // to connect the nodes to one another.
        connect_nodes!(node1, node3, recv_u);
        connect_nodes!(node2, node3, recv_f);
        connect_nodes!(node3, node4, recv_c);

        // Lastly, start up your nodes.
        let node1_handle = thread::spawn(move || {
            node1.run_node();
        });

        let node2_handle = thread::spawn(move || {
            node2.run_node();
        });

        let node3_handle = thread::spawn(move || {
            node3.run_node();
        });

        let node4_handle = thread::spawn(move || {
            node4.run_node();
        });

        node1_handle.join().unwrap();
        node2_handle.join().unwrap();
        node3_handle.join().unwrap();
        node4_handle.join().unwrap();
    }
}
