#[macro_use]
extern crate node_derive;
#[macro_use]
extern crate comms_rs;

use comms_rs::prelude::*;
use std::thread;

node_derive!(
    pub struct Node1 {
        sender: Vec<(Sender<u32>, Option<u32>)>,
    }
);

impl Node1 {
    fn run(&mut self) -> u32 {
        1
    }
}

node_derive!(
    pub struct Node2 {
        recv_u32: Option<Receiver<u32>>,
    }
);

impl Node2 {
    fn run(&mut self, x: u32) -> () {
        assert_eq!(x, 1);
    }
}

#[test]
fn test_macro() {
    let mut node1 = Node1 { sender: vec![] };

    let mut node2 = Node2 { recv_u32: None };

    connect_nodes!(node1, node2, recv_u32);

    thread::spawn(move || {
        node1.call();
    });

    node2.call();
}
