use crossbeam::Sender;
use node::Node;

extern crate num; // 0.2.0

use num::PrimInt;
use std::mem::size_of;

/// A node that implements a generic LFSR based PRNS generator.
create_node!(
    PrnsNode<T>: u8 where T: PrimInt,
    [poly_mask: T, state: T],
    [],
    |node: &mut PrnsNode<T>| node.run()
);

/// Implementation of run for the PrnsNode.
impl<T: PrimInt> PrnsNode<T> {
    fn run(&mut self) -> u8 {
        let fb_bit =
            T::from((self.state & self.poly_mask).count_ones() % 2).unwrap();
        let output = self.state >> (size_of::<T>() * 8 - 1);
        self.state = self.state << 1;
        self.state = self.state | fb_bit;
        output.to_u8().unwrap()
    }
}

/// Constructs a new `PrnsNode<T: PrimInt>`.
///
/// Arguments:
///  poly_mask - Polynomial bit mask to define the feedback taps on the LFSR. A
///              1 designates that the state bit present should be part of the
///              xor operation when creating the next bit in the sequence.
///  state     - Initial state of the LFSR.
pub fn prns<T: PrimInt>(poly_mask: T, state: T) -> PrnsNode<T> {
    PrnsNode::new(poly_mask, state)
}

#[cfg(test)]
mod test {
    use crossbeam::{Receiver, Sender};
    use crossbeam_channel as channel;
    use node::Node;
    use num::PrimInt;
    use prn::prn_node;
    use std::collections::HashMap;
    use std::hash::Hash;
    use std::mem::size_of;
    use std::thread;
    use std::time::Instant;

    #[test]
    // A test to verify the correctness of a maximum length PRBS8.
    fn test_prns8_correctness() {
        struct TestPrnsGenerator<T: PrimInt + Hash> {
            poly_mask: T,
            state: T,
            statemap: HashMap<T, u8>,
        }

        impl<T: PrimInt + Hash> TestPrnsGenerator<T> {
            fn run(&mut self) -> Option<u8> {
                if self.statemap.contains_key(&self.state) {
                    println!("\nSize of <T>: {}", size_of::<T>());
                    println!("\n\nWrapped, size = {}!", self.statemap.len());
                    assert_eq!(self.statemap.len(), 255);
                    return None;
                } else {
                    self.statemap.insert(self.state, 1 as u8);
                }

                let fb_bit =
                    T::from((self.state & self.poly_mask).count_ones() % 2)
                        .unwrap();
                let output = self.state >> (size_of::<T>() * 8 - 1);
                self.state = self.state << 1;
                self.state = self.state | fb_bit;
                output.to_u8()
            }
        }

        let mut prngen = TestPrnsGenerator {
            poly_mask: 0xB8 as u8,
            state: 0x01,
            statemap: HashMap::new(),
        };

        loop {
            match prngen.run() {
                Some(x) => print!("{:x}", x),
                None => break,
            }
        }
    }

    #[test]
    // A test to verify the PrnsNode matches the PRBS7 output.
    fn test_prns_node() {
        let mut mynode = prn_node::prns(0xC0 as u8, 0x01);
        create_node!(
            CheckNode: (),
            [state: Vec<u8>],
            [recv: u8],
            |node: &mut CheckNode, x| if node.state.len() == 128 {
                assert_eq!(
                    node.state,
                    vec![
                        0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 1, 1, 0, 0,
                        0, 0, 0, 1, 0, 1, 0, 0, 0, 0, 1, 1, 1, 1, 0, 0, 0, 1,
                        0, 0, 0, 1, 0, 0, 1, 1, 0, 0, 1, 1, 0, 1, 0, 1, 0, 1,
                        0, 1, 1, 1, 1, 1, 1, 1, 1, 0, 0, 0, 0, 0, 0, 0, 1, 0,
                        0, 0, 0, 0, 0, 1, 1, 0, 0, 0, 0, 0, 1, 0, 1, 0, 0, 0,
                        0, 1, 1, 1, 1, 0, 0, 0, 1, 0, 0, 0, 1, 0, 0, 1, 1, 0,
                        0, 1, 1, 0, 1, 0, 1, 0, 1, 0, 1, 1, 1, 1, 1, 1, 1, 1,
                        0, 0
                    ]
                );
            } else {
                node.state.push(x);
            }
        );

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
