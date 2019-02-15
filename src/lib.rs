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

#![allow(clippy::unreadable_literal)]

extern crate bincode;
extern crate byteorder;
extern crate crossbeam;
extern crate crossbeam_channel;
extern crate node_derive;
extern crate num;
extern crate num_traits;
extern crate rand;
extern crate rayon;
extern crate rustfft;
extern crate serde;

#[macro_use]
pub mod node;
pub mod fft;
pub mod filter;
pub mod hardware;
pub mod io;
pub mod mixer;
pub mod modulation;
pub mod prn;
pub mod pulse;
pub mod util;

#[cfg(test)]
#[macro_use]
extern crate assert_approx_eq; // 1.0.0

pub mod prelude;
