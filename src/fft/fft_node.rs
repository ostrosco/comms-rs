use crossbeam::{Receiver, Sender};
use node::Node;
use num::NumCast;
use rustfft::num_traits::Num;
use rustfft::num_complex::Complex;
use rustfft::num_traits::Zero;
use rustfft::{FFTplanner, FFT};
use std::sync::Arc;

create_node!(
    FFTBatchNode<T>: Vec<Complex<T>> where T: NumCast + Clone + Num,
    [fft: Arc<FFT<f64>>, fft_size: usize],
    [recv: Vec<Complex<T>>],
    |node: &mut FFTBatchNode<T>, data: Vec<Complex<T>>| {
        node.run_fft(&data)
    }
);

impl<T> FFTBatchNode<T>
where
    T: NumCast + Clone + Num,
{
    fn run_fft(&mut self, data: &[Complex<T>]) -> Vec<Complex<T>> {
        // Convert the input type from interleaved values to Complex<f32>.
        let mut input: Vec<Complex<f64>> = data
            .iter()
            .map(|x| {
                Complex::new(x.re.to_f64().unwrap(), x.im.to_f64().unwrap())
            }).collect();
        let mut output: Vec<Complex<f64>> =
            vec![Complex::zero(); self.fft_size];
        self.fft.process(&mut input[..], &mut output[..]);

        // After the FFT, convert back to interleaved values.
        let res: Vec<Complex<T>> = output
            .iter()
            .map(|x| Complex::new(T::from(x.re).unwrap(), T::from(x.im).unwrap()))
            .collect();
        res
    }
}

pub fn fft_batch_node<T: NumCast + Clone + Num>(
    fft_size: usize,
    ifft: bool,
) -> FFTBatchNode<T> {
    let mut planner = FFTplanner::new(ifft);
    let fft = planner.plan_fft(fft_size);
    FFTBatchNode::new(fft, fft_size)
}

create_node!(
    FFTSampleNode<T>: Option<Vec<Complex<T>>> where T: NumCast + Clone + Num,
    [fft: Arc<FFT<f64>>, fft_size: usize, samples: Vec<Complex<T>>],
    [recv: Complex<T>],
    |node: &mut FFTSampleNode<T>, sample: Complex<T>| {
        node.samples.push(sample);
        if node.samples.len() == node.fft_size {
            let results = node.run_fft();
            node.samples = vec![];
            Some(results)
        } else {
            None
        }
    }
);

impl<T> FFTSampleNode<T>
where
    T: NumCast + Clone + Num,
{
    fn run_fft(&mut self) -> Vec<Complex<T>> {
        let mut input: Vec<Complex<f64>> = self.samples
            .iter()
            .map(|x| {
                Complex::new(x.re.to_f64().unwrap(), x.im.to_f64().unwrap())
            }).collect();
        let mut output: Vec<Complex<f64>> =
            vec![Complex::zero(); self.fft_size];
        self.fft.process(&mut input[..], &mut output[..]);

        // After the FFT, convert back to interleaved values.
        let res: Vec<Complex<T>> = output
            .iter()
            .map(|x| Complex::new(T::from(x.re).unwrap(), T::from(x.im).unwrap()))
            .collect();
        res
    }
}

pub fn fft_sample_node<T: NumCast + Clone + Num>(
    fft_size: usize,
    ifft: bool,
) -> FFTSampleNode<T> {
    let mut planner = FFTplanner::new(ifft);
    let fft = planner.plan_fft(fft_size);
    FFTSampleNode::new(fft, fft_size, vec![])
}

#[cfg(test)]
mod test {
    use crossbeam::{Receiver, Sender};
    use crossbeam_channel as channel;
    use fft::fft_node;
    use node::Node;
    use std::thread;
    use std::time::Instant;
    use rustfft::num_complex::Complex;

    #[test]
    fn test_fft_batch() {
        create_node!(
            SendNode: Vec<Complex<f32>>,
            [],
            [],
            |_| {
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
                input
            }
        );
        let mut send_node = SendNode::new();

        let mut fft_node = fft_node::fft_batch_node(10, false);

        create_node!(
            CheckNode: (),
            [],
            [recv: Vec<Complex<f32>>],
            |_, val: Vec<Complex<f32>>| {
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
            }
        );
        let mut check_node = CheckNode::new();

        connect_nodes!(send_node, fft_node, recv);
        connect_nodes!(fft_node, check_node, recv);
        start_nodes!(send_node, fft_node);
        let check = thread::spawn(move || {
            let now = Instant::now();
            loop {
                check_node.call();
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
            |node: &mut SendNode| {
                node.input.pop()
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
            |_, val: Vec<Complex<f32>>| {
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
            }
        );
        let mut check_node = CheckNode::new();

        connect_nodes!(send_node, fft_node, recv);
        connect_nodes!(fft_node, check_node, recv);
        start_nodes!(send_node, fft_node);
        let check = thread::spawn(move || {
            check_node.call();
        });
        assert!(check.join().is_ok());
    }
}
