//! ========
//! This node implements a finite impulse response (FIR) filter.  These are
//! general purpose filters used everywhere in communications and DSP systems.
//! Some example use cases include low pass filters for anti aliasing, band
//! pass filters for band selection, and pulse shaping.
//!
//! Assume initial state of 0's. Takes in Complex<T> samples, outputs
//! Complex<T>.  Constructor takes Vec<Complex<T>> for filter taps.

pub mod fir_node;
