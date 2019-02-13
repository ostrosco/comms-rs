extern crate node_derive;
#[macro_use]
extern crate comms_rs;

use comms_rs::prelude::*;
use std::thread;

#[derive(Node)]
pub struct Node1 {
    sender: NodeSender<u32>,
}

impl Node1 {
    fn run(&mut self) -> Result<u32, NodeError> {
        Ok(1)
    }
}

#[derive(Node)]
pub struct Node2 {
    recv_input: NodeReceiver<u32>,
    stuff: u32,
    sender: NodeSender<u32>,
}

impl Node2 {
    fn run(&mut self, x: u32) -> Result<u32, NodeError> {
        assert_eq!(x, 1);
        Ok(x + self.stuff)
    }
}

#[derive(Node)]
pub struct Node3 {
    recv_input: NodeReceiver<u32>,
}

impl Node3 {
    fn run(&mut self, x: u32) -> Result<(), NodeError> {
        assert_eq!(x, 6);
        Ok(())
    }
}

#[test]
fn test_macro() {
    let mut node1 = Node1::new();

    let mut node2 = Node2::new(5);

    let mut node3 = Node3::new();

    connect_nodes!(node1, sender, node2, recv_input);
    connect_nodes!(node2, sender, node3, recv_input);

    thread::spawn(move || {
        node1.call().unwrap();
    });

    thread::spawn(move || {
        node2.call().unwrap();
    });

    node3.call().unwrap();
}
