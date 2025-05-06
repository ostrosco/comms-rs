//! Module to provide pulse shaping features.
use crate::filter::fir::*;
use crate::prelude::*;

use num::{Complex, Num, Zero};

/// A node that implements a pulse shaping filter of a specified sort.
///
/// This node type is really a combination of a couple of the other existing
/// nodes.  Essentially the pulse shaping is accomplished by upsampling the
/// input signal through the insertion of zero samples, and then running the
/// upsampled signal through an FIR filter whose taps determine the pulse
/// shape.
///
/// Several options for pulse shape tap generators are provided in the `util`
/// module, including (but not limited to) root raised cosine, raised cosine,
/// and rectangular window pulse shapings.
///
/// # Arguments
///
/// * `taps` - FIR filter taps to implement pulse shaping
/// * `sam_per_sym` - The number of output samples per input symbol
/// * `state` - The initial state for the internal FIR filter
///
/// # Examples
///
/// ```
/// use comms_rs::pulse::PulseNode;
/// use comms_rs::util::math::rect_taps;
/// use num::{Zero, Complex};
///
/// let sam_per_sym = 4_usize;
/// let taps: Vec<Complex<i16>> = rect_taps(sam_per_sym).unwrap();
/// let node = PulseNode::new(taps, sam_per_sym);
/// ```
#[derive(Node)]
#[pass_by_ref]
pub struct PulseNode<T>
where
    T: Num + Copy + Send,
{
    pub input: NodeReceiver<Complex<T>>,
    taps: Vec<Complex<T>>,
    sam_per_sym: usize,
    state: Vec<Complex<T>>,
    pub output: NodeSender<Vec<Complex<T>>>,
}

impl<T> PulseNode<T>
where
    T: Num + Copy + Send,
{
    /// Constructs a new `PulseNode<T>` with initial state set to zeros.
    ///
    /// # Arguments
    ///
    /// * `taps` - FIR filter tap Vec[Complex<T>]
    /// * `sam_per_sym` - Number of samples per symbol in the output
    ///
    /// # Examples
    ///
    /// ```
    /// use comms_rs::pulse::PulseNode;
    /// use comms_rs::util::math::rect_taps;
    /// use num::{Zero, Complex};
    ///
    /// let sam_per_sym = 4_usize;
    /// let taps: Vec<Complex<i16>> = rect_taps(sam_per_sym).unwrap();
    /// let node = PulseNode::new(taps, sam_per_sym);
    /// ```
    pub fn new(taps: Vec<Complex<T>>, sam_per_sym: usize) -> Self {
        let len = taps.len();
        PulseNode {
            taps,
            sam_per_sym,
            state: vec![Complex::zero(); len],
            input: Default::default(),
            output: Default::default(),
        }
    }

    pub fn run(
        &mut self,
        input: &Complex<T>,
    ) -> Result<Vec<Complex<T>>, NodeError> {
        let mut output = vec![fir(input, &self.taps, &mut self.state)];
        for _ in 0..&self.sam_per_sym - 1 {
            output.push(fir(&Complex::zero(), &self.taps, &mut self.state));
        }
        Ok(output)
    }
}

#[cfg(test)]
mod test {
    use crate::prelude::*;
    use crate::pulse::*;
    use crate::util::math::*;
    use num::Complex;
    use std::thread;
    use std::time::Instant;

    #[test]
    fn test_rect_node() {
        #[derive(Node)]
        struct SomeSamples {
            samples: Vec<Complex<i16>>,
            output: NodeSender<Complex<i16>>,
        }

        impl SomeSamples {
            pub fn new(samples: Vec<Complex<i16>>) -> Self {
                SomeSamples {
                    samples,
                    output: Default::default(),
                }
            }

            pub fn run(&mut self) -> Result<Complex<i16>, NodeError> {
                if self.samples.is_empty() {
                    Ok(Complex::zero())
                } else {
                    Ok(self.samples.remove(0))
                }
            }
        }

        let mut source = SomeSamples::new(vec![
            Complex::new(-1, -1),
            Complex::new(1, -1),
            Complex::new(1, -1),
            Complex::new(1, 1),
            Complex::new(-1, 1),
        ]);

        let sam_per_sym = 4;
        let taps = rect_taps(sam_per_sym).unwrap();
        let mut mynode = PulseNode::new(taps, sam_per_sym);

        #[derive(Node)]
        pub struct CheckNode {
            pub input: NodeReceiver<Vec<Complex<i16>>>,
            state: Vec<Complex<i16>>,
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
                input: Vec<Complex<i16>>,
            ) -> Result<(), NodeError> {
                if self.state.len() == 20 {
                    assert_eq!(
                        self.state,
                        vec![
                            Complex::new(-1, -1),
                            Complex::new(-1, -1),
                            Complex::new(-1, -1),
                            Complex::new(-1, -1),
                            Complex::new(1, -1),
                            Complex::new(1, -1),
                            Complex::new(1, -1),
                            Complex::new(1, -1),
                            Complex::new(1, -1),
                            Complex::new(1, -1),
                            Complex::new(1, -1),
                            Complex::new(1, -1),
                            Complex::new(1, 1),
                            Complex::new(1, 1),
                            Complex::new(1, 1),
                            Complex::new(1, 1),
                            Complex::new(-1, 1),
                            Complex::new(-1, 1),
                            Complex::new(-1, 1),
                            Complex::new(-1, 1),
                        ]
                    );
                } else {
                    for value in input {
                        self.state.push(value);
                    }
                }
                Ok(())
            }
        }

        let mut check_node = CheckNode::new();

        connect_nodes!(source, output, mynode, input);
        connect_nodes!(mynode, output, check_node, input);
        start_nodes!(source, mynode);
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
