//! Nodes for performing FFTs and IFFTs.

pub mod fft_node;

use num::Complex;
use num::NumCast;
use rustfft::num_complex::Complex as FFTComplex;
use rustfft::num_traits::Num;
use rustfft::num_traits::Zero;
use rustfft::FFT;
use std::sync::Arc;

/// Batch based wrapper of FFT implementation provided by
/// [RustFFT](https://github.com/awelkie/RustFFT).
///
/// This implementation acts on a batch of samples at a time.  The underlying
/// library only provides an FFT for type `f64`, so this wrapper will
/// automatically cast any provided input appropriately, although it does
/// expect the general form of `Complex<T>`.
pub struct BatchFFT {
    pub fft: Arc<FFT<f64>>,
    pub fft_size: usize,
}

impl BatchFFT {
    /// Creates a new `BatchFFT`.
    ///
    /// Requires an FFT plan from the RustFFT crate.
    ///
    /// # Arguments
    ///
    /// * `fft` - FFT plan to be executed in this implementation.
    /// * `fft_size` - Size of the FFT to be performed.
    ///
    /// # Examples
    ///
    /// ```
    /// use comms_rs::fft::*;
    /// use rustfft::FFTplanner;
    ///
    /// let fft_size = 1024;
    /// let mut planner = FFTplanner::new(false);
    /// let fft = planner.plan_fft(fft_size);
    /// let batch_fft = BatchFFT::new(fft, fft_size);
    /// ```
    pub fn new(fft: Arc<FFT<f64>>, fft_size: usize) -> BatchFFT {
        BatchFFT { fft, fft_size }
    }

    /// Runs the `BatchFFT`.
    ///
    /// Takes input `Complex<T>` and performs the FFT specified in
    /// construction, and returns the resulting output.
    ///
    /// # Arguments
    ///
    /// * `data` - Complex samples on which to perform the FFT.
    ///
    /// # Examples
    ///
    /// ```
    /// use comms_rs::fft::*;
    /// use num::{Complex, Zero};
    /// use rustfft::FFTplanner;
    ///
    /// let fft_size = 1024;
    /// let mut planner = FFTplanner::new(false);
    /// let fft = planner.plan_fft(fft_size);
    /// let mut batch_fft = BatchFFT::new(fft, fft_size);
    ///
    /// let result: Vec<Complex<f64>> = batch_fft.run_fft(&vec![Complex::zero(); fft_size][..]);
    /// ```
    pub fn run_fft<T>(&mut self, data: &[Complex<T>]) -> Vec<Complex<T>>
    where
        T: NumCast + Clone + Num,
    {
        // Convert the input type from interleaved values to Complex<f32>.
        let mut input: Vec<FFTComplex<f64>> = data
            .iter()
            .map(|x| {
                FFTComplex::new(x.re.to_f64().unwrap(), x.im.to_f64().unwrap())
            })
            .collect();
        let mut output: Vec<FFTComplex<f64>> =
            vec![FFTComplex::zero(); self.fft_size];
        self.fft.process(&mut input[..], &mut output[..]);

        // After the FFT, convert back to interleaved values.
        let res: Vec<Complex<T>> = output
            .iter()
            .map(|x| {
                Complex::new(T::from(x.re).unwrap(), T::from(x.im).unwrap())
            })
            .collect();
        res
    }
}

/// Sample based wrapper of FFT implementation provided by
/// [RustFFT](https://github.com/awelkie/RustFFT).
///
/// This implementation acts on a sample at a time.  The underlying library
/// only provides an FFT for type `f64`, so this wrapper will automatically
/// cast any provided input appropriately, although it does expect the general
/// form of `Complex<T>`.
pub struct SampleFFT<T> {
    pub fft: Arc<FFT<f64>>,
    pub fft_size: usize,
    pub samples: Vec<Complex<T>>,
}

impl<T> SampleFFT<T>
where
    T: NumCast + Clone + Num,
{
    /// Creates a new `SampleFFT`.
    ///
    /// Requires an FFT plan from the RustFFT crate.
    ///
    /// # Arguments
    ///
    /// * `fft` - FFT plan to be executed in this implementation.
    /// * `fft_size` - Size of the FFT to be performed.
    ///
    /// # Examples
    ///
    /// ```
    /// use comms_rs::fft::*;
    /// use rustfft::FFTplanner;
    ///
    /// let fft_size = 1024;
    /// let mut planner = FFTplanner::new(false);
    /// let fft = planner.plan_fft(fft_size);
    /// let sample_fft: SampleFFT<f64> = SampleFFT::new(fft, fft_size);
    /// ```
    pub fn new(fft: Arc<FFT<f64>>, fft_size: usize) -> SampleFFT<T> {
        SampleFFT {
            fft,
            fft_size,
            samples: Vec::new(),
        }
    }

    /// Runs the `SampleFFT`.
    ///
    /// Takes input `Complex<T>` and performs the FFT specified in
    /// construction, and returns the resulting output.
    ///
    /// # Examples
    ///
    /// ```
    /// use comms_rs::fft::*;
    /// use num::{Complex, Zero};
    /// use rustfft::FFTplanner;
    ///
    /// let fft_size = 1024;
    /// let mut planner = FFTplanner::new(false);
    /// let fft = planner.plan_fft(fft_size);
    /// let mut sample_fft: SampleFFT<f64> = SampleFFT::new(fft, fft_size);
    /// sample_fft.samples = vec![Complex::zero(); fft_size];
    ///
    /// let result: Vec<Complex<f64>> = sample_fft.run_fft();
    /// ```
    pub fn run_fft(&mut self) -> Vec<Complex<T>> {
        let mut input: Vec<FFTComplex<f64>> = self
            .samples
            .iter()
            .map(|x| {
                FFTComplex::new(x.re.to_f64().unwrap(), x.im.to_f64().unwrap())
            })
            .collect();
        let mut output: Vec<FFTComplex<f64>> =
            vec![FFTComplex::zero(); self.fft_size];
        self.fft.process(&mut input[..], &mut output[..]);

        // After the FFT, convert back to interleaved values.
        let res: Vec<Complex<T>> = output
            .iter()
            .map(|x| {
                Complex::new(T::from(x.re).unwrap(), T::from(x.im).unwrap())
            })
            .collect();
        res
    }
}
