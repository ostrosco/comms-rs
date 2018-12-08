//! Nodes for generated pseudo-random numbers.

pub mod prn_node;
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
