//! Implementation of a finite impulse response (FIR) filter.
//!
//! FIR filters are general purpose filters used everywhere in communications
//! and DSP systems.  Some example use cases include low pass filters for
//! anti-aliasing, band pass filters for band selection, and pulse shaping.
//!
//! Assume initial state of 0's. Takes in `Complex<T>` samples, outputs
//! `Complex<T>`.  Constructor takes `Vec<Complex<T>>` for filter taps.

use num::complex::Complex;
use num_traits::Num;

/// Implementation of run for the FirNode.
///
/// # Arguments
///
/// * `input` - Input sample to be filtered.
/// * `taps` - FIR filter taps.
/// * `state` - FIR filter initial internal state.
///
/// # Examples
///
/// ```
/// use comms_rs::filter::fir::*;
/// use num::Complex;
///
/// let input = Complex::new(1.2_f64, -0.747_f64);
/// let taps = vec![
///     Complex::new(0.2, 0.0),
///     Complex::new(0.6, 0.0),
///     Complex::new(0.6, 0.0),
///     Complex::new(0.2, 0.0),
/// ];
///
/// let mut state = vec![
///     Complex::new(1.0, 0.0),
///     Complex::new(0.5, 0.0),
///     Complex::new(0.25, 0.0),
///     Complex::new(0.125, 0.0),
/// ];
///
/// let output = fir(&input, &taps, &mut state);
/// ```
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
///
/// # Arguments
///
/// * `input` - Input batch of samples to be filtered.
/// * `taps` - FIR filter taps.
/// * `state` - FIR filter initial internal state.
///
/// # Examples
///
/// ```
/// use comms_rs::filter::fir::*;
/// use num::Complex;
///
/// let input: Vec<Complex<f64>> = (0..100).map(|x| Complex::new((x as f64).cos(), 0.0)).collect();
/// let taps = vec![
///     Complex::new(0.2, 0.0),
///     Complex::new(0.6, 0.0),
///     Complex::new(0.6, 0.0),
///     Complex::new(0.2, 0.0),
/// ];
///
/// let mut state = vec![
///     Complex::new(1.0, 0.0),
///     Complex::new(0.5, 0.0),
///     Complex::new(0.25, 0.0),
///     Complex::new(0.125, 0.0),
/// ];
///
/// let output = batch_fir(&input, &taps, &mut state);
/// ```
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
