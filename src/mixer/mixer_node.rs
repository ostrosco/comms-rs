use crossbeam::{Receiver, Sender};
use node::Node;
use std::f64::consts::PI;

extern crate num; // 0.2.0

use num::complex::Complex;
use num::Num;
use num_traits::NumCast;
use util::math;

/// A node that implements a generic mixer.
create_node!(
    MixerNode<T>: Complex<T>,
    [phase: f64, dphase: f64],
    [input: Complex<T>],
    |node: &mut MixerNode<T>, input: Complex<T>| node.run(&input),
    T: Clone + Num + NumCast,
);

/// Implementation of run for the MixerNode.
impl<T> MixerNode<T>
where
    T: NumCast + Clone + Num,
{
    fn run(&mut self, input: &Complex<T>) -> Complex<T> {
        self.phase += self.dphase;
        if self.phase > 2.0 * PI {
            self.phase -= 2.0 * PI;
        }
        let inp: Complex<f64> = math::cast_complex(input).unwrap();
        let res = inp * Complex::exp(&Complex::new(0.0, self.phase));
        math::cast_complex(&res).unwrap()
    }
}

/// Constructs a new `MixerNode<T>`.  Assumes 0 initial phase in the local
/// oscillator.  Any frequency above Nyquist will not be supported, ie, dphase
/// will be limited to the range [0, 2*Pi).
///
/// Arguments:
///     dphase - The change in phase (radians) per sampling period. This should
///              be dphase = 2 * PI * freq(Hz) * Ts.
pub fn mixer<T>(mut dphase: f64) -> MixerNode<T>
where
    T: NumCast + Clone + Num,
{
    while dphase >= 2.0 * PI {
        dphase -= 2.0 * PI;
    }
    while dphase < 0.0 {
        dphase += 2.0 * PI;
    }
    MixerNode::new(0.0, dphase)
}

/// Constructs a new `MixerNode<T>`.  User defined initial phase in the local
/// oscillator.  Any frequency above Nyquist will not be supported, ie, dphase
/// will be limited to the range [0, 2*Pi).
///
/// Arguments:
///     dphase - The change in phase (radians) per sampling period. This should
///              be dphase = 2 * PI * freq(Hz) * Ts.
///     phase  - The initial phase of the oscillator.
pub fn mixer_with_phase<T>(mut dphase: f64, phase: f64) -> MixerNode<T>
where
    T: NumCast + Clone + Num,
{
    while dphase >= 2.0 * PI {
        dphase -= 2.0 * PI;
    }
    while dphase < 0.0 {
        dphase += 2.0 * PI;
    }
    MixerNode::new(phase, dphase)
}

#[cfg(test)]
mod test {
    use crossbeam::{Receiver, Sender};
    use crossbeam_channel as channel;
    use mixer::mixer_node;
    use node::Node;
    use num::complex::Complex;
    use num::Zero;
    use std::thread;
    use std::time::Instant;

    #[test]
    // A test to verify the sample by sample mixer node with initial phase 0.
    fn test_mixer() {
        create_node!(
            SomeSamples: Complex<f64>,
            [samples: Vec<Complex<f64>>],
            [],
            |node: &mut Self| if node.samples.len() == 0 {
                Complex::zero()
            } else {
                node.samples.remove(0)
            }
        );

        let mut source = SomeSamples::new(vec![
            Complex::new(1.0, 2.0),
            Complex::new(3.0, 4.0),
            Complex::new(5.0, 6.0),
            Complex::new(7.0, 8.0),
            Complex::new(9.0, 0.0),
        ]);

        let mut mixer = mixer_node::mixer::<f64>(0.123);

        create_node!(
            CheckNode: (),
            [state: Vec<Complex<f64>>],
            [recv: Complex<f64>],
            |node: &mut CheckNode, x| if node.state.len() == 5 {
                let truth = vec![
                    Complex::new(1.0, 2.0),
                    Complex::new(3.39265326, 4.52353768),
                    Complex::new(6.39449787, 7.67339744),
                    Complex::new(10.12401323, 11.57030083),
                    Complex::new(14.72025707, 0.0),
                ];
                for i in 0..node.state.len() {
                    assert_approx_eq!(node.state[i].re, truth[i].re);
                    assert_approx_eq!(node.state[i].im, truth[i].im);
                }
            } else {
                node.state.push(x);
            }
        );

        let mut check_node = CheckNode::new(Vec::new());

        connect_nodes!(source, mixer, input);
        connect_nodes!(mixer, check_node, recv);
        start_nodes!(source, mixer);
        let check = thread::spawn(move || {
            let now = Instant::now();
            loop {
                check_node.call();
                if now.elapsed().subsec_millis() > 10 {
                    break;
                }
            }
        });
        assert!(check.join().is_ok());
    }
}
