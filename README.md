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
