//! This node implements a basic mixer.  It provides a sample by sample and
//! batch based mixer, and versions of each with and inital phase value.

use crate::prelude::*;

extern crate num; // 0.2.0

use crate::mixer::Mixer;
use num::complex::Complex;
use num::Num;
use num_traits::NumCast;

/// A node that implements a generic mixer.
#[derive(Node)]
#[pass_by_ref]
pub struct MixerNode<T>
where
    T: Clone + Num + NumCast,
{
    pub input: NodeReceiver<Complex<T>>,
    mixer: Mixer,
    pub sender: NodeSender<Complex<T>>,
}

impl<T> MixerNode<T>
where
    T: Clone + Num + NumCast,
{
    pub fn run(&mut self, input: &Complex<T>) -> Result<Complex<T>, NodeError> {
        Ok(self.mixer.mix(input))
    }
}

/// Constructs a new `MixerNode<T>`.  Assumes 0 initial phase in the local
/// oscillator.  Any frequency above Nyquist will not be supported, ie, dphase
/// will be limited to the range [0, 2*Pi).
///
/// Arguments:
///     dphase - The change in phase (radians) per sampling period. This should
///              be dphase = 2 * PI * freq(Hz) * Ts.
pub fn mixer_node<T>(dphase: f64) -> MixerNode<T>
where
    T: NumCast + Clone + Num,
{
    MixerNode::new(Mixer::new(0.0, dphase))
}

/// Constructs a new `MixerNode<T>`.  User defined initial phase in the local
/// oscillator.  Any frequency above Nyquist will not be supported, ie, dphase
/// will be limited to the range [0, 2*Pi).
///
/// Arguments:
///     dphase - The change in phase (radians) per sampling period. This should
///              be dphase = 2 * PI * freq(Hz) * Ts.
///     phase  - The initial phase of the oscillator.
pub fn mixer_node_with_phase<T>(dphase: f64, phase: f64) -> MixerNode<T>
where
    T: NumCast + Clone + Num,
{
    MixerNode::new(Mixer::new(phase, dphase))
}

#[cfg(test)]
mod test {
    use crate::mixer::mixer_node;
    use crate::prelude::*;
    use crossbeam_channel as channel;
    use num::complex::Complex;
    use num::Zero;
    use std::thread;
    use std::time::Instant;

    #[test]
    // A test to verify the sample by sample mixer node with initial phase 0.
    fn test_mixer() {
        #[derive(Node)]
        struct SomeSamples {
            samples: Vec<Complex<f64>>,
            pub sender: NodeSender<Complex<f64>>,
        }

        impl SomeSamples {
            pub fn run(&mut self) -> Result<Complex<f64>, NodeError> {
                if self.samples.is_empty() {
                    Ok(Complex::zero())
                } else {
                    Ok(self.samples.remove(0))
                }
            }
        }

        let mut source = SomeSamples::new(vec![
            Complex::new(1.0, 2.0),
            Complex::new(3.0, 4.0),
            Complex::new(5.0, 6.0),
            Complex::new(7.0, 8.0),
            Complex::new(9.0, 0.0),
        ]);

        let mut mixer = mixer_node::mixer_node::<f64>(0.123);

        #[derive(Node)]
        struct CheckNode {
            pub input: NodeReceiver<Complex<f64>>,
            state: Vec<Complex<f64>>,
        }

        impl CheckNode {
            pub fn run(
                &mut self,
                input: Complex<f64>,
            ) -> Result<(), NodeError> {
                if self.state.len() == 5 {
                    let truth = vec![
                        Complex::new(1.0, 2.0),
                        Complex::new(2.486574736, 4.337850399),
                        Complex::new(3.388313374, 7.036997405),
                        Complex::new(3.643356072, 9.986288426),
                        Complex::new(7.932508585, 4.251506503),
                    ];
                    for (i, truth) in truth.iter().enumerate() {
                        assert_approx_eq!(self.state[i].re, truth.re);
                        assert_approx_eq!(self.state[i].im, truth.im);
                    }
                } else {
                    self.state.push(input);
                }
                Ok(())
            }
        }

        let mut check_node = CheckNode::new(Vec::new());

        connect_nodes!(source, sender, mixer, input);
        connect_nodes!(mixer, sender, check_node, input);
        start_nodes!(source, mixer);
        let check = thread::spawn(move || {
            let now = Instant::now();
            loop {
                check_node.call().unwrap();
                if now.elapsed().subsec_millis() > 10 {
                    break;
                }
            }
        });
        assert!(check.join().is_ok());
    }

    #[test]
    // A test to verify the sample by sample mixer node with initial phase 0.1.
    fn test_mixer_with_phase() {
        #[derive(Node)]
        struct SomeSamples {
            samples: Vec<Complex<f64>>,
            pub sender: NodeSender<Complex<f64>>,
        }

        impl SomeSamples {
            pub fn run(&mut self) -> Result<Complex<f64>, NodeError> {
                if self.samples.is_empty() {
                    Ok(Complex::zero())
                } else {
                    Ok(self.samples.remove(0))
                }
            }
        }

        let mut source = SomeSamples::new(vec![
            Complex::new(1.0, 2.0),
            Complex::new(3.0, 4.0),
            Complex::new(5.0, 6.0),
            Complex::new(7.0, 8.0),
            Complex::new(9.0, 0.0),
        ]);

        let mut mixer = mixer_node::mixer_node_with_phase::<f64>(0.123, 0.1);

        #[derive(Node)]
        struct CheckNode {
            pub input: NodeReceiver<Complex<f64>>,
            state: Vec<Complex<f64>>,
        }

        impl CheckNode {
            pub fn run(
                &mut self,
                input: Complex<f64>,
            ) -> Result<(), NodeError> {
                if self.state.len() == 5 {
                    let truth = vec![
                        Complex::new(0.795337332, 2.089841747),
                        Complex::new(2.041089794, 4.564422467),
                        Complex::new(2.668858427, 7.340108630),
                        Complex::new(2.628189174, 10.300127265),
                        Complex::new(7.468436663, 5.022196114),
                    ];
                    for (i, truth) in truth.iter().enumerate() {
                        assert_approx_eq!(self.state[i].re, truth.re);
                        assert_approx_eq!(self.state[i].im, truth.im);
                    }
                } else {
                    self.state.push(input);
                }
                Ok(())
            }
        }

        let mut check_node = CheckNode::new(Vec::new());

        connect_nodes!(source, sender, mixer, input);
        connect_nodes!(mixer, sender, check_node, input);
        start_nodes!(source, mixer);
        let check = thread::spawn(move || {
            let now = Instant::now();
            loop {
                check_node.call().unwrap();
                if now.elapsed().subsec_millis() > 10 {
                    break;
                }
            }
        });
        assert!(check.join().is_ok());
    }
}
