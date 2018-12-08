use num::Complex;
use num::NumCast;
use rustfft::num_complex::Complex as FFTComplex;
use rustfft::num_traits::Num;
use rustfft::num_traits::Zero;
use rustfft::FFT;
use std::sync::Arc;

pub struct BatchFFT {
    pub fft: Arc<FFT<f64>>,
    pub fft_size: usize,
}

impl BatchFFT {
    pub fn new(fft: Arc<FFT<f64>>, fft_size: usize) -> BatchFFT {
        BatchFFT { fft, fft_size }
    }

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

pub struct SampleFFT<T> {
    pub fft: Arc<FFT<f64>>,
    pub fft_size: usize,
    pub samples: Vec<Complex<T>>,
}

impl<T> SampleFFT<T>
where
    T: NumCast + Clone + Num,
{
    pub fn new(fft: Arc<FFT<f64>>, fft_size: usize) -> SampleFFT<T> {
        SampleFFT {
            fft,
            fft_size,
            samples: Vec::new(),
        }
    }

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
