// Disabling this lint as it triggers in macros which is an issue being tracked
// here: https://github.com/rust-lang-nursery/rust-clippy/issues/1553.
#![cfg_attr(feature = "cargo-clippy", allow(redundant_closure_call))]

extern crate byteorder;
extern crate crossbeam;
extern crate crossbeam_channel;
extern crate num;
extern crate num_traits;
extern crate rand;
extern crate rayon;
extern crate rodio;
extern crate rustfft;

#[macro_use]
pub mod node;
pub mod fft;
pub mod filter;
pub mod fir;
pub mod hardware;
pub mod modulation;
pub mod output;
pub mod prn;
pub mod util;

pub mod prelude;
