//! Provides nodes for executing forward and reverse FFTs.

use crate::fft::{BatchFFT, SampleFFT};
use crate::prelude::*;
use num::Complex;
use num::NumCast;
use rustfft::num_traits::Num;
use rustfft::FFTplanner;
use std::default::Default;

/// A node that supports batch execution of FFTs and IFFTs.
///
/// The node expects that input data matching the specified FFT size is
/// provided.
///
/// # Examples
///
/// ```
/// use comms_rs::fft::*;
/// use comms_rs::fft::fft_node::*;
/// use rustfft::FFTplanner;
///
/// let fft_size = 1024;
/// let node: FFTBatchNode<f64> = FFTBatchNode::new(fft_size, false);
/// ```
#[derive(Node)]
#[pass_by_ref]
pub struct FFTBatchNode<T>
where
    T: NumCast + Copy + Num + Send,
{
    pub input: NodeReceiver<Vec<Complex<T>>>,
    batch_fft: BatchFFT,
    pub output: NodeSender<Vec<Complex<T>>>,
}

impl<T> FFTBatchNode<T>
where
    T: NumCast + Copy + Num + Send,
{
    /// Constructs a node that performs FFT or IFFTs in batches.
    ///
    /// # Arguments
    ///
    /// * `fft_size` - The size of the FFT to be performed
    /// * `ifft` - `true` to perform an inverse FFT, `false` for a normal forward
    /// FFT.
    ///
    /// # Examples
    ///
    /// ```
    /// # extern crate comms_rs;
    /// # #[macro_use] use comms_rs::node::Node;
    /// # use comms_rs::prelude::*;
    /// use comms_rs::fft::fft_node::{self, FFTBatchNode};
    ///
    /// // Sets up an FFT that receives 1024 Complex<i16> samples and performs
    /// // an FFT on those samples.
    /// let mut fft_node: FFTBatchNode<i16> = FFTBatchNode::new(1024, false);
    ///
    /// // Sets up an IFFT that receives 1024 Complex<f32> complex samples and performs
    /// // an IFFT on those samples.
    /// let mut ifft_node: FFTBatchNode<f32> = FFTBatchNode::new(1024, true);
    /// ```
    pub fn new(fft_size: usize, ifft: bool) -> Self {
        let mut planner = FFTplanner::new(ifft);
        let fft = planner.plan_fft(fft_size);
        let batch_fft = BatchFFT::new(fft, fft_size);
        FFTBatchNode {
            batch_fft,
            input: Default::default(),
            output: Default::default(),
        }
    }

    /// Runs the `FFTBatchNode<T>` on passed batch of samples.  Produces either
    /// a new `Vec<Complex<T>>` batch of samples or a `NodeError`.
    pub fn run(
        &mut self,
        data: &[Complex<T>],
    ) -> Result<Vec<Complex<T>>, NodeError> {
        Ok(self.batch_fft.run_fft(data))
    }
}

/// A node that supports sample by sample execution of FFTs and IFFTs.
///
/// This node expects data to be provided sample by sample and will only
/// perform the FFT once it has received enough samples specified by fft_size.
///
/// # Examples
///
/// ```
/// use comms_rs::fft::*;
/// use comms_rs::fft::fft_node::*;
/// use rustfft::FFTplanner;
///
/// let fft_size = 1024;
/// let node: FFTSampleNode<f64> = FFTSampleNode::new(fft_size, false);
/// ```
#[derive(Node)]
#[aggregate]
#[pass_by_ref]
pub struct FFTSampleNode<T>
where
    T: NumCast + Copy + Num + Send,
{
    pub input: NodeReceiver<Complex<T>>,
    sample_fft: SampleFFT<T>,
    pub output: NodeSender<Vec<Complex<T>>>,
}

impl<T> FFTSampleNode<T>
where
    T: NumCast + Copy + Num + Send,
{
    /// Constructs a node that performs FFT or IFFTs, receiving a sample at a time
    /// versus a batch of samples.
    ///
    /// # Arguments
    ///
    /// * `fft_size` - The size of the FFT to be performed.
    /// * `ifft` - `true` to perform an inverse FFT, `false` for a normal forward
    /// FFT.
    ///
    /// # Example:
    ///
    /// ```
    /// # extern crate comms_rs;
    /// # #[macro_use] use comms_rs::node::Node;
    /// # use comms_rs::prelude::*;
    /// use comms_rs::fft::fft_node::{self, FFTSampleNode};
    ///
    /// // Sets up an FFT that receives 1024 Complex<i16> samples and performs
    /// // an FFT on those samples.
    /// let mut fft_node: FFTSampleNode<i16> = FFTSampleNode::new(1024, false);
    ///
    /// // Sets up an IFFT that receives 1024 Complex<f32> complex samples and performs
    /// // an IFFT on those samples.
    /// let mut ifft_node: FFTSampleNode<f32> = FFTSampleNode::new(1024, true);
    /// ```
    pub fn new(fft_size: usize, ifft: bool) -> Self {
        let mut planner = FFTplanner::new(ifft);
        let fft = planner.plan_fft(fft_size);
        let sample_fft = SampleFFT::new(fft, fft_size);
        FFTSampleNode {
            sample_fft,
            input: Default::default(),
            output: Default::default(),
        }
    }

    /// Runs the `FFTSampleNode<T>` on passed sample.  Produces either a new
    /// `Complex<T>` sample or a `NodeError`.
    pub fn run(
        &mut self,
        sample: &Complex<T>,
    ) -> Result<Option<Vec<Complex<T>>>, NodeError> {
        self.sample_fft.samples.push(*sample);
        if self.sample_fft.samples.len() == self.sample_fft.fft_size {
            let results = self.sample_fft.run_fft();
            self.sample_fft.samples = vec![];
            Ok(Some(results))
        } else {
            Ok(None)
        }
    }
}

#[cfg(test)]
mod test {
    use crate::fft::fft_node;
    use num::Complex;
    use std::thread;
    use std::time::Instant;

    use crate::prelude::*;

    #[test]
    fn test_fft_batch() {
        #[derive(Node)]
        struct SendNode {
            pub output: NodeSender<Vec<Complex<f32>>>,
        }

        impl SendNode {
            pub fn new() -> Self {
                SendNode {
                    output: Default::default(),
                }
            }

            pub fn run(&mut self) -> Result<Vec<Complex<f32>>, NodeError> {
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
            }
        }

        let mut send_node = SendNode::new();
        let mut fft_node = fft_node::FFTBatchNode::new(10, false);

        #[derive(Node)]
        #[pass_by_ref]
        struct CheckNode {
            pub input: NodeReceiver<Vec<Complex<f32>>>,
        }

        impl CheckNode {
            pub fn new() -> Self {
                CheckNode {
                    input: Default::default(),
                }
            }

            pub fn run(
                &mut self,
                input: &[Complex<f32>],
            ) -> Result<(), NodeError> {
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
                for (actual, expected) in input.iter().zip(expected_out) {
                    assert!((actual - expected).norm() < 1e-5);
                }
                Ok(())
            }
        }
        let mut check_node = CheckNode::new();

        connect_nodes!(send_node, output, fft_node, input);
        connect_nodes!(fft_node, output, check_node, input);
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
        #[derive(Node)]
        struct SendNode {
            state: Vec<Complex<f32>>,
            pub output: NodeSender<Complex<f32>>,
        }

        impl SendNode {
            pub fn new(state: Vec<Complex<f32>>) -> Self {
                SendNode {
                    state,
                    output: Default::default(),
                }
            }

            pub fn run(&mut self) -> Result<Complex<f32>, NodeError> {
                Ok(self.state.pop().unwrap_or_else(|| Complex::new(0.0, 0.0)))
            }
        }

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
        let mut fft_node = fft_node::FFTSampleNode::new(10, false);
        #[derive(Node)]
        #[pass_by_ref]
        struct CheckNode {
            pub input: NodeReceiver<Vec<Complex<f32>>>,
        }

        impl CheckNode {
            pub fn new() -> Self {
                CheckNode {
                    input: Default::default(),
                }
            }

            pub fn run(
                &mut self,
                input: &[Complex<f32>],
            ) -> Result<(), NodeError> {
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
                for (actual, expected) in input.iter().zip(expected_out) {
                    assert!((actual - expected).norm() < 1e-5);
                }
                Ok(())
            }
        }
        let mut check_node = CheckNode::new();

        connect_nodes!(send_node, output, fft_node, input);
        connect_nodes!(fft_node, output, check_node, input);
        start_nodes!(send_node, fft_node);
        let check = thread::spawn(move || {
            check_node.call().unwrap();
        });
        assert!(check.join().is_ok());
    }
}
