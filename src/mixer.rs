//! This node implements a basic complex mixer.  It currently provides versions
//! for sample by sample operation with and without a specified initial phase.

use crate::prelude::*;
use std::f64::consts::PI;

extern crate num; // 0.2.0

use num::complex::Complex;
use num::Num;
use num_traits::NumCast;

use crate::util::math;

/// Struct to implement a complex mixer.
///
/// This combines an input signal with a complex exponential for modulation or
/// demodulation of carrier frequencies to passband or baseband signals.
pub struct Mixer {
    phase: f64,
    dphase: f64,
}

impl Mixer {
    /// Creates a new `Mixer` struct with parameters as specified.
    ///
    /// The `dphase` parameter is automatically adjusted to the interval
    /// [0.0, 2 * PI).
    ///
    /// # Arguments
    ///
    /// * `phase` - Intial phase state in radians of complex exponential.
    /// * `dphase` - Time derivative of phase in radians per sample.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::f64::consts::PI;
    /// use comms_rs::mixer::Mixer;
    ///
    /// let phase = PI / 4.0;
    /// let dphase = 0.1_f64;
    /// let mixer = Mixer::new(phase, dphase);
    /// ```
    pub fn new(phase: f64, mut dphase: f64) -> Mixer {
        while dphase >= 2.0 * PI {
            dphase -= 2.0 * PI;
        }
        while dphase < 0.0 {
            dphase += 2.0 * PI;
        }
        Mixer { phase, dphase }
    }

    /// Runs the input signal through the `Mixer`.
    ///
    /// # Arguments
    ///
    /// * `input` - Signal with which to modulate the carrier.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::f64::consts::PI;
    /// use comms_rs::mixer::Mixer;
    /// use num::Complex;
    ///
    /// let phase = PI / 4.0;
    /// let dphase = 0.1_f64;
    /// let mut mixer = Mixer::new(phase, dphase);
    ///
    /// let input: Complex<f64> = Complex::new(12.345_f64.cos(), 0.0);
    /// let passband = mixer.mix(&input);
    /// ```
    pub fn mix<T>(&mut self, input: &Complex<T>) -> Complex<T>
    where
        T: NumCast + Copy + Num,
    {
        let inp: Complex<f64> = math::cast_complex(input).unwrap();
        let res = inp * Complex::exp(&Complex::new(0.0, self.phase));
        self.phase += self.dphase;
        if self.phase > 2.0 * PI {
            self.phase -= 2.0 * PI;
        }
        math::cast_complex(&res).unwrap()
    }
}

/// A node that implements a generic mixer.
///
/// This node operates on a single sample at a time, as opposed to batch mode
/// operation.
#[derive(Node)]
#[pass_by_ref]
pub struct MixerNode<T>
where
    T: Copy + Num + NumCast,
{
    pub input: NodeReceiver<Complex<T>>,
    mixer: Mixer,
    pub sender: NodeSender<Complex<T>>,
}

impl<T> MixerNode<T>
where
    T: Copy + Num + NumCast,
{
    /// Constructs a new `MixerNode<T>` with specified initial phase.
    ///
    /// Any frequency above Nyquist will not be supported, ie, dphase will be
    /// limited to the range [0, 2*Pi).
    ///
    /// # Arguments
    ///
    /// * `dphase` - The change in phase (radians) per sampling period. This should
    /// be dphase = 2 * PI * freq(Hz) * Ts.
    /// * `phase` - The initial phase of the oscillator.
    ///
    /// # Examples
    ///
    /// ```
    /// use comms_rs::mixer::*;
    /// use std::f64::consts::PI;
    /// use num::Complex;
    ///
    /// let dphase = 0.1_f64;
    /// let phase: f64 = PI / 4.0;
    /// let node: MixerNode<Complex<f64>> = MixerNode::new(dphase, Some(phase));
    /// ```
    pub fn new(dphase: f64, phase: Option<f64>) -> Self {
        match phase {
            Some(ph) => MixerNode {
                mixer: Mixer::new(ph, dphase),
                input: Default::default(),
                sender: Default::default(),
            },
            None => MixerNode {
                mixer: Mixer::new(0.0, dphase),
                input: Default::default(),
                sender: Default::default(),
            },
        }
    }

    /// Runs the `MixerNode<T>`.  Produces either the mixed `Complex<T>` sample
    /// or a `NodeError`.
    pub fn run(&mut self, input: &Complex<T>) -> Result<Complex<T>, NodeError> {
        Ok(self.mixer.mix(input))
    }
}

#[cfg(test)]
mod test {
    use crate::mixer::*;
    use crossbeam::channel;
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
            pub fn new(samples: Vec<Complex<f64>>) -> Self {
                SomeSamples {
                    samples,
                    sender: Default::default(),
                }
            }

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

        let mut mixer: MixerNode<f64> = MixerNode::new(0.123, None);

        #[derive(Node)]
        struct CheckNode {
            pub input: NodeReceiver<Complex<f64>>,
            state: Vec<Complex<f64>>,
        }

        impl CheckNode {
            pub fn new() -> Self {
                CheckNode {
                    state: vec![],
                    input: Default::default(),
                }
            }

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

        let mut check_node = CheckNode::new();

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
            pub fn new(samples: Vec<Complex<f64>>) -> Self {
                SomeSamples {
                    samples,
                    sender: Default::default(),
                }
            }

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

        let mut mixer: MixerNode<f64> = MixerNode::new(0.123, Some(0.1));

        #[derive(Node)]
        struct CheckNode {
            pub input: NodeReceiver<Complex<f64>>,
            state: Vec<Complex<f64>>,
        }

        impl CheckNode {
            pub fn new() -> Self {
                CheckNode {
                    state: vec![],
                    input: Default::default(),
                }
            }

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

        let mut check_node = CheckNode::new();

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
