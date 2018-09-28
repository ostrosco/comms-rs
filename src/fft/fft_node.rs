use node::Node;
use crossbeam::{Receiver, Sender};
use num::NumCast;
use rustfft::num_complex::Complex;
use rustfft::num_traits::Zero;
use rustfft::{FFTplanner, FFT};
use std::sync::Arc;

create_node!(
    FFTBatchNode<T>: Vec<T> where T: NumCast + Clone,
    [fft: Arc<FFT<f32>>, fft_size: usize],
    [recv: Vec<T>],
    |node: &mut FFTBatchNode<T>, data: Vec<T>| {
        node.run_fft(&data)
    }
);

create_node!(
    FFTSampleNode<T>: Option<Vec<T>> where T: NumCast + Clone,
    [fft: Arc<FFT<f32>>, fft_size: usize, samples: Vec<T>],
    [recv: T],
    |node: &mut FFTSampleNode<T>, sample: T| {
        if node.samples.len() == node.fft_size {
            let results = node.run_fft();
            node.samples = vec![];
            Some(results)
        } else {
            node.samples.push(sample);
            None
        }
    }
);

impl<T> FFTSampleNode<T>
where
    T: NumCast + Clone,
{
    fn run_fft(&mut self) -> Vec<T> {
        // Convert the input type from interleaved values to Complex<f32>.
        let mut input: Vec<Complex<f32>> = self
            .samples
            .chunks(2)
            .map(|ref x| {
                Complex::new(x[0].to_f32().unwrap(), x[1].to_f32().unwrap())
            }).collect();
        let mut output: Vec<Complex<f32>> =
            vec![Complex::zero(); self.fft_size];
        self.fft.process(&mut input[..], &mut output[..]);

        // After the FFT, convert back to interleaved values.
        let res: Vec<T> = output.iter().fold(vec![], |mut acc, cmplx| {
            acc.push(T::from(cmplx.re).unwrap());
            acc.push(T::from(cmplx.im).unwrap());
            acc
        });
        res
    }
}

impl<T> FFTBatchNode<T>
where
    T: NumCast + Clone,
{
    fn run_fft(&mut self, data: &[T]) -> Vec<T> {
        // Convert the input type from interleaved values to Complex<f32>.
        let mut input: Vec<Complex<f32>> = data
            .chunks(2)
            .map(|ref x| {
                Complex::new(x[0].to_f32().unwrap(), x[1].to_f32().unwrap())
            }).collect();
        let mut output: Vec<Complex<f32>> =
            vec![Complex::zero(); self.fft_size];
        self.fft.process(&mut input[..], &mut output[..]);

        // After the FFT, convert back to interleaved values.
        let res: Vec<T> = output.iter().fold(vec![], |mut acc, cmplx| {
            acc.push(T::from(cmplx.re).unwrap());
            acc.push(T::from(cmplx.im).unwrap());
            acc
        });
        res
    }
}

pub fn fft_batch_node<T: NumCast + Clone>(
    fft_size: usize,
    ifft: bool,
) -> FFTBatchNode<T> {
    let mut planner = FFTplanner::new(ifft);
    let fft = planner.plan_fft(fft_size);
    FFTBatchNode::new(fft, fft_size)
}

pub fn fft_sample_node<T: NumCast + Clone>(
    fft_size: usize,
    ifft: bool,
) -> FFTSampleNode<T> {
    let mut planner = FFTplanner::new(ifft);
    let fft = planner.plan_fft(fft_size);
    FFTSampleNode::new(fft, fft_size, vec![])
}
