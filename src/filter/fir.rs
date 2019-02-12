//! Node for a finite impulse response (FIR) filter.
//!
//! This node implements a finite impulse response (FIR) filter.  These are
//! general purpose filters used everywhere in communications and DSP systems.
//! Some example use cases include low pass filters for anti aliasing, band
//! pass filters for band selection, and pulse shaping.
//!
//! Assume initial state of 0's. Takes in Complex<T> samples, outputs
//! Complex<T>.  Constructor takes Vec<Complex<T>> for filter taps.

use num::complex::Complex;
use num_traits::Num;

/// Implementation of run for the BatchFirNode.
pub fn fir<T>(
    input: &Complex<T>,
    taps: &[Complex<T>],
    state: &mut Vec<Complex<T>>,
) -> Complex<T>
where
    T: Num + Copy,
{
    state.rotate_right(1);
    state[0] = *input;
    taps.iter().zip(state.iter()).map(|(x, y)| *x * *y).sum()
}

/// Implementation of run for the BatchFirNode.
pub fn batch_fir<T>(
    input: &[Complex<T>],
    taps: &[Complex<T>],
    state: &mut Vec<Complex<T>>,
) -> Vec<Complex<T>>
where
    T: Num + Copy,
{
    let mut output = Vec::new();
    for sample in input {
        state.rotate_right(1);
        state[0] = *sample;
        output.push(taps.iter().zip(state.iter()).map(|(x, y)| *x * *y).sum());
    }
    output
}
