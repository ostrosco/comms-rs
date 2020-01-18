//! This node implements a basic complex NCO.  It currently provides versions
//! for sample by sample operation with and without a specified initial phase.

use crate::prelude::*;
use std::f64::consts::PI;

extern crate num; // 0.2.0

use num::complex::Complex;

/// Struct to implement a complex NCO.
///
/// This combines an input signal with a complex exponential for modulation or
/// demodulation of carrier frequencies to passband or baseband signals.
pub struct Nco {
    phase: f64,
    dphase: f64,
}

impl Nco {
    /// Creates a new `Nco` struct with parameters as specified.
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
    /// use comms_rs::demodulation::nco::Nco;
    ///
    /// let phase = PI / 4.0;
    /// let dphase = 0.1_f64;
    /// let nco = Nco::new(phase, dphase);
    /// ```
    pub fn new(phase: f64, mut dphase: f64) -> Nco {
        while dphase >= 2.0 * PI {
            dphase -= 2.0 * PI;
        }
        while dphase < 0.0 {
            dphase += 2.0 * PI;
        }
        Nco { phase, dphase }
    }

    /// Runs the input signal through the `Nco`.
    ///
    /// # Arguments
    ///
    /// * `input` - Signal with which to modulate the carrier.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::f64::consts::PI;
    /// use comms_rs::demodulation::nco::Nco;
    /// use num::Complex;
    ///
    /// let phase = PI / 4.0;
    /// let dphase = 0.1_f64;
    /// let mut nco = Nco::new(phase, dphase);
    ///
    /// let perr: f64 = -0.01;
    /// let nco_out = nco.push(&perr);
    /// ```
    pub fn push(&mut self, perr: &f64) -> Complex<f64> {
        self.phase += self.dphase + perr;
        if self.phase > 2.0 * PI {
            self.phase -= 2.0 * PI;
        }
        Complex::exp(&Complex::new(0.0, self.phase))
    }
}

/// A node that implements a generic NCO.
///
/// This node operates on a single sample at a time, as opposed to batch mode
/// operation.
#[derive(Node)]
#[pass_by_ref]
pub struct NcoNode {
    pub input: NodeReceiver<f64>,
    nco: Nco,
    pub sender: NodeSender<Complex<f64>>,
}

impl NcoNode {
    /// Constructs a new `NcoNode` with specified initial phase.
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
    /// use comms_rs::demodulation::nco::*;
    /// use std::f64::consts::PI;
    /// use num::Complex;
    ///
    /// let dphase = 0.1_f64;
    /// let phase: f64 = PI / 4.0;
    /// let node: NcoNode = NcoNode::new(dphase, Some(phase));
    /// ```
    pub fn new(dphase: f64, phase: Option<f64>) -> Self {
        match phase {
            Some(ph) => NcoNode {
                nco: Nco::new(ph, dphase),
                input: Default::default(),
                sender: Default::default(),
            },
            None => NcoNode {
                nco: Nco::new(0.0, dphase),
                input: Default::default(),
                sender: Default::default(),
            },
        }
    }

    /// Runs the `NcoNode`.  Produces either the mixed `Complex<f64>` sample
    /// or a `NodeError`.
    pub fn run(&mut self, input: &f64) -> Result<Complex<f64>, NodeError> {
        Ok(self.nco.push(input))
    }
}

#[cfg(test)]
mod test {
    use crate::demodulation::nco::*;
    use crossbeam::channel;
    use num::complex::Complex;
    use std::thread;
    use std::time::Instant;

    #[test]
    // A test to verify the sample by sample NCO node with initial phase 0.
    fn test_nco() {
        #[derive(Node)]
        struct SomeSamples {
            samples: Vec<f64>,
            pub sender: NodeSender<f64>,
        }

        impl SomeSamples {
            pub fn new(samples: Vec<f64>) -> Self {
                SomeSamples {
                    samples,
                    sender: Default::default(),
                }
            }

            pub fn run(&mut self) -> Result<f64, NodeError> {
                if self.samples.is_empty() {
                    Ok(0.0)
                } else {
                    Ok(self.samples.remove(0))
                }
            }
        }

        let mut source = SomeSamples::new(vec![
            1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 0.0,
        ]);

        let mut nco: NcoNode = NcoNode::new(0.123, None);

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

        connect_nodes!(source, sender, nco, input);
        connect_nodes!(nco, sender, check_node, input);
        start_nodes!(source, nco);
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
    // A test to verify the sample by sample NCO node with initial phase 0.1.
    fn test_nco_with_phase() {
        #[derive(Node)]
        struct SomeSamples {
            samples: Vec<f64>,
            pub sender: NodeSender<f64>,
        }

        impl SomeSamples {
            pub fn new(samples: Vec<f64>) -> Self {
                SomeSamples {
                    samples,
                    sender: Default::default(),
                }
            }

            pub fn run(&mut self) -> Result<f64, NodeError> {
                if self.samples.is_empty() {
                    Ok(0.0)
                } else {
                    Ok(self.samples.remove(0))
                }
            }
        }

        let mut source = SomeSamples::new(vec![
            1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 0.0,
        ]);

        let mut nco: NcoNode = NcoNode::new(0.123, Some(0.1));

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

        connect_nodes!(source, sender, nco, input);
        connect_nodes!(nco, sender, check_node, input);
        start_nodes!(source, nco);
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
