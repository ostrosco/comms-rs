
//! [![Build Status](https://travis-ci.com/ostrosco/comms-rs.svg?branch=develop)](https://travis-ci.com/ostrosco/comms-rs)
//! 
//! A library for building DSP communication pipelines. The goal of this
//! experimental project is explore the use of Rust for the generation of signals
//! in real time.  The memory model and ease of concurrency in rust are potentially
//! very useful for a data flow graph style of processing, with each node running
//! in its own thread.
//! 
//! Currently macros are being worked on that allow a user to easily drop an
//! arbitrary defined function into a graph node, and ease the development of
//! potentially complicated processing pipelines.
//! 
//! There will be node functions defined for several common DSP and communications
//! oriented tasks (such as mixing, filtering, FFTs, etc...) and provided directly
//! in the project for immediate use.  Additionally hardware interfaces will be
//! provided to work with a few common SDR platforms.

// Disabling this lint as it triggers in macros which is an issue being tracked
// here: https://github.com/rust-lang-nursery/rust-clippy/issues/1553.
#![cfg_attr(feature = "cargo-clippy", allow(redundant_closure_call))]

extern crate bincode;
extern crate byteorder;
extern crate crossbeam;
extern crate crossbeam_channel;
extern crate num;
extern crate num_traits;
extern crate rand;
extern crate rayon;
extern crate rodio;
extern crate rustfft;
extern crate serde;

#[macro_use]
pub mod node;
pub mod fft;
pub mod filter;
pub mod fir;
pub mod hardware;
pub mod io;
pub mod mixer;
pub mod modulation;
pub mod prn;
pub mod util;

#[cfg(test)]
#[macro_use]
extern crate assert_approx_eq; // 1.0.0

pub mod prelude;
