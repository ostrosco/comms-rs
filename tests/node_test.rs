#[macro_use]
extern crate comms_rs;

use comms_rs::prelude::*;
use std::thread;
use std::time::Duration;

#[test]
fn simple_nodes() {
    #[derive(Node)]
    struct SourceNode {
        pub sender: NodeSender<u32>,
    }

    impl SourceNode {
        pub fn run(&mut self) -> Result<u32, NodeError> {
            Ok(1)
        }
    }

    #[derive(Node)]
    struct SinkNode {
        pub input: NodeReceiver<u32>,
    }

    impl SinkNode {
        pub fn run(&mut self, input: u32) -> Result<(), NodeError> {
            assert_eq!(input, 1);
            Ok(())
        }
    }

    let mut node = SourceNode::new();
    let mut node2 = SinkNode::new();
    connect_nodes!(node, sender, node2, input);
    start_nodes!(node, node2);
    thread::sleep(Duration::from_secs(1));
}
