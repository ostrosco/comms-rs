use node::Node;
use crossbeam::{Receiver, Sender};

extern crate num; // 0.2.0

use num::{PrimInt, ToPrimitive};

create_generic_node!(
    PrnsNode<T>: u8 where T: PrimInt,
    [poly_mask: T, state: T],
    [],
    |node: &mut PrnsNode<T>| node.run()
);

impl<T: PrimInt> PrnsNode<T> {
    fn run(&mut self) -> u8 {
        let new_bit = T::from((self.state & self.poly_mask).count_ones() % 2).unwrap();
        self.state = (self.state << 1) | new_bit;
        new_bit.to_u8().unwrap()
    }

    fn set_state(&mut self, new_state: T) -> () {
        self.state = new_state;
    }
}

pub fn prns<T: PrimInt>(poly_mask: T, state: T) -> PrnsNode<T> {
    PrnsNode::new(poly_mask, state)
}

#[cfg(test)]
mod test {
    use crossbeam::{Receiver, Sender};
    use crossbeam_channel as channel;
    use node::Node;
    use std::thread;
    use std::time::Instant;
    use prn::prn_node;

    #[test]
    fn test_prns_generator() {
        let mut mynode = prn_node::prns(0xC0, 0xFF);
        create_node!(CheckNode: (),
                     [state: Vec<u8>],
                     [recv: u8],
                     |node: &mut CheckNode, x| {
            if node.state.len() == 256 {
                assert_eq!(node.state, vec![0,0,0,0,0,0,0,1,0,0,0,0,0,0,1,1,0,0,0,0,0,1,0,1,0,0,0,0,1,1,1,1,0,0,0,1,0,0,0,1,0,0,1,1,0,0,1,1,0,1,0,1,0,1,0,1,1,1,1,1,1,1,1,0,0,0,0,0,0,0,1,0,0,0,0,0,0,1,1,0,0,0,0,0,1,0,1,0,0,0,0,1,1,1,1,0,0,0,1,0,0,0,1,0,0,1,1,0,0,1,1,0,1,0,1,0,1,0,1,1,1,1,1,1,1,1,0,0,0,0,0,0,0,1,0,0,0,0,0,0,1,1,0,0,0,0,0,1,0,1,0,0,0,0,1,1,1,1,0,0,0,1,0,0,0,1,0,0,1,1,0,0,1,1,0,1,0,1,0,1,0,1,1,1,1,1,1,1,1,0,0,0,0,0,0,0,1,0,0,0,0,0,0,1,1,0,0,0,0,0,1,0,1,0,0,0,0,1,1,1,1,0,0,0,1,0,0,0,1,0,0,1,1,0,0,1,1,0,1,0,1,0,1,0,1,1,1,1,1,1,1,1,0,0,0,0]);
            } else {
                node.state.push(x);
            }
        });

        let mut check_node = CheckNode::new(Vec::new());

        connect_nodes!(mynode, check_node, recv);
        start_nodes!(mynode);
        let check = thread::spawn(move || {
            let now = Instant::now();
            loop {
                check_node.call();
                if now.elapsed().as_secs() > 1 {
                    break;
                }
            }
        });
        assert!(check.join().is_ok());
    }
}
