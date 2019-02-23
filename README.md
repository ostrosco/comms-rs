# comms-rs

[![Build Status](https://travis-ci.com/ostrosco/comms-rs.svg?branch=develop)](https://travis-ci.com/ostrosco/comms-rs)

A library for building DSP communication pipelines. The goal of this
experimental project is explore the use of Rust for the generation of signals
in real time.  The memory model and ease of concurrency in rust are potentially
very useful for a data flow graph style of processing, with each node running
in its own thread.

Currently macros are being worked on that allow a user to easily drop an
arbitrary defined function into a graph node, and ease the development of
potentially complicated processing pipelines.

There will be node functions defined for several common DSP and communications
oriented tasks (such as mixing, filtering, FFTs, etc...) and provided directly
in the project for immediate use.  Additionally hardware interfaces will be
provided to work with a few common SDR platforms.

Initial Goals:
- [x] Node based architecture
- [x] FFT/IFFT
- [x] FIR filter
- [X] Decimation
- [x] Upsampling
- [x] Pulse shaping
- [X] Mixer
- [x] BPSK modulation
- [X] Write/read to/from file
- [ ] Hardware support for HackRF
- [X] Hardware support for rtl-sdr
- [ ] Hardware support for BladeRF

# Creating Your Own Nodes

The easiest way to create your own nodes is to derive the Node trait for your
structure. The macro used to derive the Node trait is a bit magical, so there
are certain assumptions it makes in order for the derivation to work properly.

To derive the trait, there are a few requirements that much be met.

* Any inputs to the node must be of the type NodeReceiver<T>.
* Any outputs from the node must be of the type NodeSender<T>.
* The structure must implement a method named run(). The signature of run() is
  currently expected to take &mut self followed by every input type received
  in order defined in the structure. It is expected to return a
  Result<U, NodeError>.

Example:

```
#[derive(Node)]
pub struct ExampleNode {
    input1: NodeReceiver<u32>,
    input2: NodeReceiver<f64>,
    output: NodeSender<u8>,
}

impl ExampleNode {
    pub fn run(&mut self, input1: u32, input2: f64) -> Result<u8, NodeError> {
        ...
    }
}
```
