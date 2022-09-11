//! This node implements a basic complex NCO.  It currently provides versions
//! for sample by sample operation with and without a specified initial phase.

use crate::prelude::*;
use std::f64::consts::PI;

extern crate num; // 0.2.0

use num::Complex;

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
    /// let nco_out = nco.push(perr);
    /// ```
    pub fn push(&mut self, perr: f64) -> Complex<f64> {
        self.phase += self.dphase + perr;
        if self.phase > 2.0 * PI {
            self.phase -= 2.0 * PI;
        }
        Complex::exp(Complex::new(0.0, self.phase))
    }
}

/// A node that implements a generic NCO.
///
/// This node operates on a single sample at a time, as opposed to batch mode
/// operation.
#[derive(Node)]
pub struct NcoNode {
    pub input: NodeReceiver<f64>,
    nco: Nco,
    pub output: NodeSender<Complex<f64>>,
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
                output: Default::default(),
            },
            None => NcoNode {
                nco: Nco::new(0.0, dphase),
                input: Default::default(),
                output: Default::default(),
            },
        }
    }

    /// Runs the `NcoNode`.  Produces either the mixed `Complex<f64>` sample
    /// or a `NodeError`.
    pub fn run(&mut self, input: f64) -> Result<Complex<f64>, NodeError> {
        Ok(self.nco.push(input))
    }
}
