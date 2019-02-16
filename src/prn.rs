//! Generates a PRN using a linear-feedback shift register.
//!
//! This node implements a PRNS generator using an linear-feedback shift register
//! (LFSR).  These are often used in communications systems for various tasks which
//! require a merely pseudorandom sequence as they are very cheap and easy to
//! implement in hardware with the use of a single LFSR.  These tasks can include
//! things such as frequency hopping and spread spectrum waveform spreading codes.
//!
//! As usual,
//! [the Wiki](https://en.wikipedia.org/wiki/Linear-feedback_shift_register#Fibonacci_LFSRs)
//! is an excellent reference for farther details, particularly regarding what
//! exactly the polynomial bitmask is all about.  Note that the implementation of
//! an LFSR in this code has the register shifting to the left rather than right.
//! This is different than the Wiki graphics, and I chose it because it matches the
//! feedback polynomials in standard form better in my opinion.
//!
//! A final note regarding the arguments to the constructor: be careful to size the
//! input type as the type with the desired LFSR length.  If you simply do something
//! like `let mut node = prns(0xC0, 1);` you'll get a 32 bit LFSR, which may not be
//! what you want.  Doing `let mut node = prns(0xC0 as u8, 1);` indicates to the
//! node internals that you want an 8 bit LSFR implementation.

use crate::prelude::*;

extern crate num; // 0.2.0

use num::PrimInt;
use std::mem::size_of;

pub struct PrnGen<T> {
    poly_mask: T,
    state: T,
}

/// Implementation of run for the PrnsNode.
impl<T: PrimInt> PrnGen<T> {
    pub fn new(poly_mask: T, state: T) -> PrnGen<T> {
        PrnGen { poly_mask, state }
    }

    pub fn next_byte(&mut self) -> u8 {
        let fb_bit =
            T::from((self.state & self.poly_mask).count_ones() % 2).unwrap();
        let output = self.state >> (size_of::<T>() * 8 - 1);
        self.state = self.state << 1;
        self.state = self.state | fb_bit;
        output.to_u8().unwrap()
    }
}

/// A node that implements a generic LFSR based PRNS generator.
///
/// This node provides bits from a standard LFSR based PRNS generator, and the
/// polynomial and initial state defining the LFSR can be specified by the
/// provided `PrnGen` struct.
///
/// # Arguments
///
/// * `prngen` - `PrnGen` struct defining the LFSR used to generate bits
///
/// # Examples
///
/// ```
/// use comms_rs::prn::{PrnGen, PrnsNode};
///
/// let poly_mask = 0xC0_u8;
/// let state = 0xFF_u8;
/// let prn_gen = PrnGen::new(poly_mask, state);
/// let node = PrnsNode::new(prn_gen);
/// ```
#[derive(Node)]
pub struct PrnsNode<T>
where
    T: PrimInt,
{
    prngen: PrnGen<T>,
    pub sender: NodeSender<u8>,
}

impl<T> PrnsNode<T>
where
    T: PrimInt,
{
    pub fn run(&mut self) -> Result<u8, NodeError> {
        Ok(self.prngen.next_byte())
    }
}

/// Constructs a new `PrnsNode<T: PrimInt>`.
///
/// # Arguments
///
/// * `poly_mask` - Polynomial bit mask to define the feedback taps on the
/// LFSR. A 1 designates that the state bit present should be part of the xor
/// operation when creating the next bit in the sequence.
/// * `state` - Initial state of the LFSR.
///
/// # Examples
///
/// ```
/// use comms_rs::prn::prns_node;
///
/// let poly_mask = 0xC0_u8;
/// let state = 0xFF_u8;
/// let node = prns_node(poly_mask, state);
/// ```
pub fn prns_node<T: PrimInt>(poly_mask: T, state: T) -> PrnsNode<T> {
    PrnsNode::new(PrnGen::new(poly_mask, state))
}

#[cfg(test)]
mod test {
    use crate::prelude::*;
    use crate::prn::*;
    use num::PrimInt;
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
            #[allow(clippy::map_entry)]
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

        while let Some(x) = prngen.run() {
            print!("{:x}", x);
        }
    }

    #[test]
    // A test to verify the PrnsNode matches the PRBS7 output.
    fn test_prns_node() {
        let mut mynode = prns_node(0xC0 as u8, 0x01);
        #[derive(Node)]
        struct CheckNode {
            recv: NodeReceiver<u8>,
            state: Vec<u8>,
        }

        impl CheckNode {
            pub fn run(&mut self, x: u8) -> Result<(), NodeError> {
                if self.state.len() == 128 {
                    assert_eq!(
                        self.state,
                        vec![
                            0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 1, 1, 0,
                            0, 0, 0, 0, 1, 0, 1, 0, 0, 0, 0, 1, 1, 1, 1, 0, 0,
                            0, 1, 0, 0, 0, 1, 0, 0, 1, 1, 0, 0, 1, 1, 0, 1, 0,
                            1, 0, 1, 0, 1, 1, 1, 1, 1, 1, 1, 1, 0, 0, 0, 0, 0,
                            0, 0, 1, 0, 0, 0, 0, 0, 0, 1, 1, 0, 0, 0, 0, 0, 1,
                            0, 1, 0, 0, 0, 0, 1, 1, 1, 1, 0, 0, 0, 1, 0, 0, 0,
                            1, 0, 0, 1, 1, 0, 0, 1, 1, 0, 1, 0, 1, 0, 1, 0, 1,
                            1, 1, 1, 1, 1, 1, 1, 0, 0
                        ]
                    );
                } else {
                    self.state.push(x);
                }
                Ok(())
            }
        }

        let mut check_node = CheckNode::new(Vec::new());

        connect_nodes!(mynode, sender, check_node, recv);
        start_nodes!(mynode);
        let check = thread::spawn(move || {
            let now = Instant::now();
            loop {
                check_node.call().unwrap();
                if now.elapsed().as_secs() > 1 {
                    break;
                }
            }
        });
        assert!(check.join().is_ok());
    }
}
