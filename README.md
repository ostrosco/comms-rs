# comms-rs
A library for building DSP communication pipelines. The goal of this experimental project is explore the use of rust for the generation of signals in real time.  The memory model and ease of concurrency in rust are potentially very useful for a data flow graph style of processing, with each node running in its own thread.

Currently macros are being worked on that allow a user to easily drop an arbitrary defined function into a graph node, and ease the development of potentially complicated processing pipelines.

There will be node functions defined for several common DSP and communications oriented tasks (such as mixing, filtering, ffts, ect...) and provided directly in the project for immediate use.  Additionally hardware interfaces will be provided to work with a few common SDR platforms.

Initial Goals:
- Construct node functions necessary to develop a BPSK modulated PRN sequence.
- Functions for basic FIR filter, decimation, upsampling, FFT, IFFT.
- Construct several sink and source node functions to allow initial testing (write to file, read from file, plot array, ect...)
- Hardware support for BladeRF, HackRF, rtl-sdr.
