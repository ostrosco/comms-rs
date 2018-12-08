use fft::{BatchFFT, SampleFFT};
use num::Complex;
use num::NumCast;
use prelude::*;
use rustfft::num_traits::Num;
use rustfft::FFTplanner;

create_node!(
    #[doc="A node that supports FFTs and IFFTs. FFTs are done in batch: the "]
    #[doc="node expects that input data matching the specified FFT size is "]
    #[doc="provided."]
    FFTBatchNode<T>: Vec<Complex<T>>,
    [batch_fft: BatchFFT],
    [recv: Vec<Complex<T>>],
    |node: &mut FFTBatchNode<T>, data: Vec<Complex<T>>| {
        Ok(node.batch_fft.run_fft(&data))
    },
    T: NumCast + Clone + Num,
);

/// Constructs a node that performs FFT or IFFTs in batches.
///
/// Example:
/// ```
/// # extern crate comms_rs;
/// # #[macro_use] use comms_rs::node::Node;
/// # use comms_rs::prelude::*;
/// # fn main() {
/// use comms_rs::fft::fft_node::{self, FFTBatchNode};
///
/// // Sets up an FFT that receives 1024 Complex<i16> samples and performs
/// // an FFT on those samples.
/// let mut fft_node: FFTBatchNode<i16> = fft_node::fft_batch_node(1024, false);
///
/// // Sets up an IFFT that receives 1024 Complex<f32> complex samples and performs
/// // an IFFT on those samples.
/// let mut ifft_node: FFTBatchNode<f32> = fft_node::fft_batch_node(1024, true);
/// # }
///
pub fn fft_batch_node<T: NumCast + Clone + Num>(
    fft_size: usize,
    ifft: bool,
) -> FFTBatchNode<T> {
    let mut planner = FFTplanner::new(ifft);
    let fft = planner.plan_fft(fft_size);
    let batch_fft = BatchFFT::new(fft, fft_size);
    FFTBatchNode::new(batch_fft)
}

create_node!(
    #[doc="A node that supports FFTs and IFFTs. This node expects data to be "]
    #[doc="provided sample by sample and will only perform the FFT once it "]
    #[doc="has received enough samples specified by fft_size."]
    FFTSampleNode<T>: Option<Vec<Complex<T>>>,
    [sample_fft: SampleFFT<T>],
    [recv: Complex<T>],
    |node: &mut FFTSampleNode<T>, sample: Complex<T>| -> Result<Option<Vec<Complex<T>>>, NodeError> {
        node.sample_fft.samples.push(sample);
        if node.sample_fft.samples.len() == node.sample_fft.fft_size {
            let results = node.sample_fft.run_fft();
            node.sample_fft.samples = vec![];
            Ok(Some(results))
        } else {
            Ok(None)
        }
    },
    T: NumCast + Clone + Num,
);

/// Constructs a node that performs FFT or IFFTs, but only receives a sample
/// at a time versus a batch of samples.
///
/// Example:
/// ```
/// # extern crate comms_rs;
/// # #[macro_use] use comms_rs::node::Node;
/// # use comms_rs::prelude::*;
/// # fn main() {
/// use comms_rs::fft::fft_node::{self, FFTSampleNode};
///
/// // Sets up an FFT that receives 1024 Complex<i16> samples and performs
/// // an FFT on those samples.
/// let mut fft_node: FFTSampleNode<i16> = fft_node::fft_sample_node(1024, false);
///
/// // Sets up an IFFT that receives 1024 Complex<f32> complex samples and performs
/// // an IFFT on those samples.
/// let mut ifft_node: FFTSampleNode<f32> = fft_node::fft_sample_node(1024, true);
/// # }
pub fn fft_sample_node<T: NumCast + Clone + Num>(
    fft_size: usize,
    ifft: bool,
) -> FFTSampleNode<T> {
    let mut planner = FFTplanner::new(ifft);
    let fft = planner.plan_fft(fft_size);
    let sample_fft = SampleFFT::new(fft, fft_size);
    FFTSampleNode::new(sample_fft)
}

#[cfg(test)]
mod test {
    use fft::fft_node;
    use num::Complex;
    use std::thread;
    use std::time::Instant;

    use prelude::*;

    #[test]
    fn test_fft_batch() {
        create_node!(SendNode: Vec<Complex<f32>>, [], [], |_| -> Result<
            Vec<Complex<f32>>,
            NodeError,
        > {
            let input = vec![
                Complex::new(0.1, 0.1),
                Complex::new(0.2, 0.2),
                Complex::new(0.3, 0.3),
                Complex::new(0.4, 0.4),
                Complex::new(0.5, 0.5),
                Complex::new(0.6, 0.6),
                Complex::new(0.7, 0.7),
                Complex::new(0.8, 0.8),
                Complex::new(0.9, 0.9),
                Complex::new(1.0, 1.0),
            ];
            Ok(input)
        });
        let mut send_node = SendNode::new();

        let mut fft_node = fft_node::fft_batch_node(10, false);

        create_node!(
            CheckNode: (),
            [],
            [recv: Vec<Complex<f32>>],
            |_, val: Vec<Complex<f32>>| -> Result<(), NodeError> {
                let expected_out = vec![
                    Complex::new(5.5, 5.5),
                    Complex::new(-2.03884, 1.03884),
                    Complex::new(-1.18819, 0.18819),
                    Complex::new(-0.86327, -0.13673),
                    Complex::new(-0.66246, -0.33754),
                    Complex::new(-0.5, -0.5),
                    Complex::new(-0.33754, -0.66246),
                    Complex::new(-0.13673, -0.86327),
                    Complex::new(0.18819, -1.18819),
                    Complex::new(1.03884, -2.03884),
                ];
                for (input, out) in val.iter().zip(expected_out) {
                    assert!((input - out).norm() < 1e-5);
                }
                Ok(())
            }
        );
        let mut check_node = CheckNode::new();

        connect_nodes!(send_node, fft_node, recv);
        connect_nodes!(fft_node, check_node, recv);
        start_nodes!(send_node, fft_node);
        let check = thread::spawn(move || {
            let now = Instant::now();
            loop {
                check_node.call().unwrap();
                if now.elapsed().as_secs() >= 1 {
                    break;
                }
            }
        });
        assert!(check.join().is_ok());
    }

    #[test]
    fn test_fft_sample() {
        create_node!(
            SendNode: Option<Complex<f32>>,
            [input: Vec<Complex<f32>>],
            [],
            |node: &mut SendNode| -> Result<Option<Complex<f32>>, NodeError> {
                Ok(node.input.pop())
            }
        );

        let input = vec![
            Complex::new(1.0, 1.0),
            Complex::new(0.9, 0.9),
            Complex::new(0.8, 0.8),
            Complex::new(0.7, 0.7),
            Complex::new(0.6, 0.6),
            Complex::new(0.5, 0.5),
            Complex::new(0.4, 0.4),
            Complex::new(0.3, 0.3),
            Complex::new(0.2, 0.2),
            Complex::new(0.1, 0.1),
        ];

        let mut send_node = SendNode::new(input);
        let mut fft_node = fft_node::fft_sample_node(10, false);

        create_node!(
            CheckNode: (),
            [],
            [recv: Vec<Complex<f32>>],
            |_, val: Vec<Complex<f32>>| -> Result<(), NodeError> {
                let expected_out = vec![
                    Complex::new(5.5, 5.5),
                    Complex::new(-2.03884, 1.03884),
                    Complex::new(-1.18819, 0.18819),
                    Complex::new(-0.86327, -0.13673),
                    Complex::new(-0.66246, -0.33754),
                    Complex::new(-0.5, -0.5),
                    Complex::new(-0.33754, -0.66246),
                    Complex::new(-0.13673, -0.86327),
                    Complex::new(0.18819, -1.18819),
                    Complex::new(1.03884, -2.03884),
                ];
                for (input, out) in val.iter().zip(expected_out) {
                    assert!((input - out).norm() < 1e-5);
                }
                Ok(())
            }
        );
        let mut check_node = CheckNode::new();

        connect_nodes!(send_node, fft_node, recv);
        connect_nodes!(fft_node, check_node, recv);
        start_nodes!(send_node, fft_node);
        let check = thread::spawn(move || {
            check_node.call().unwrap();
        });
        assert!(check.join().is_ok());
    }
}
