#[macro_use]
extern crate comms_rs;

use comms_rs::prelude::*;
use std::thread;
use std::time::Duration;

#[test]
fn simple_nodes() {
    fn gen_stuff() -> Result<u32, Error> {
        Ok(1)
    }

    fn do_thingies(x: u32) -> Result<(), Error> {
        assert_eq!(x, 1);
        Ok(())
    }

    create_node!(SourceNode: u32, [], [], { |_| gen_stuff() });
    create_node!(SinkNode: (), [], [recv1: u32], { |_, x| do_thingies(x) });

    let mut node = SourceNode::new();
    let mut node2 = SinkNode::new();
    connect_nodes!(node, node2, recv1);
    start_nodes!(node, node2);
    thread::sleep(Duration::from_secs(1));
}
