//! Node based implementation of FIR filters.
//!
//! Currently provides implementations that can handle input on a sample by
//! sample basis, or a whole batch at once.  Also each of these have versions
//! which allow the user to specify an initial internal filter state, or a
//! version which just assumes the initial internal state to be a vector of
//! zeroes.
use crate::prelude::*;

use crate::filter::fir::*;
use num::complex::Complex;
use num::Zero;
use num_traits::Num;

/// A node that implements a generic FIR filter which operates on a sample at a
/// time.
///
/// # Arguments
///
/// * `taps` - FIR filter tap `Vec<Complex<T>>`.
/// * `state` - FIR filter initial state `Vec<Complex<T>>`.
///
/// # Examples
///
/// ```
/// use comms_rs::filter::fir_node::*;
/// use num::Complex;
///
/// let taps = vec![
///     Complex::new(0.2, 0.0),
///     Complex::new(0.6, 0.0),
///     Complex::new(0.6, 0.0),
///     Complex::new(0.2, 0.0),
/// ];
///
/// let mut state = vec![
///     Complex::new(1.0, 0.0),
///     Complex::new(0.5, 0.0),
///     Complex::new(0.25, 0.0),
///     Complex::new(0.125, 0.0),
/// ];
///
/// let node = FirNode::new(taps, Some(state));
/// ```
#[derive(Node)]
#[pass_by_ref]
pub struct FirNode<T>
where
    T: Num + Copy,
{
    pub input: NodeReceiver<Complex<T>>,
    taps: Vec<Complex<T>>,
    state: Vec<Complex<T>>,
    pub sender: NodeSender<Complex<T>>,
}

impl<T> FirNode<T>
where
    T: Num + Copy,
{
    /// Constructs a new `FirNode<T>` with optional user defined initial state.
    ///
    /// # Arguments
    ///
    /// * `taps` - FIR filter tap Vec[Complex<T>].
    /// * `state` - Initial state for the internal filter state and memory. If
    ///    set to None, defaults to zeros.
    ///
    /// # Examples
    ///
    /// ```
    /// use comms_rs::filter::fir_node::*;
    /// use num::Complex;
    ///
    /// let taps = vec![
    ///     Complex::new(0.2, 0.0),
    ///     Complex::new(0.6, 0.0),
    ///     Complex::new(0.6, 0.0),
    ///     Complex::new(0.2, 0.0),
    /// ];
    ///
    /// let mut state = vec![
    ///     Complex::new(1.0, 0.0),
    ///     Complex::new(0.5, 0.0),
    ///     Complex::new(0.25, 0.0),
    ///     Complex::new(0.125, 0.0),
    /// ];
    ///
    /// let node = FirNode::new(taps, Some(state));
    /// ```
    pub fn new(taps: Vec<Complex<T>>, state: Option<Vec<Complex<T>>>) -> Self {
        match state {
            Some(st) => FirNode {
                taps,
                state: st,
                input: Default::default(),
                sender: Default::default(),
            },
            None => {
                let len = taps.len();
                FirNode {
                    taps,
                    state: vec![Complex::zero(); len],
                    input: Default::default(),
                    sender: Default::default(),
                }
            }
        }
    }

    /// Runs the `FirNode<T>`.  Produces either a new `Complex<T>` sample or
    /// a `NodeError`.
    pub fn run(&mut self, input: &Complex<T>) -> Result<Complex<T>, NodeError> {
        Ok(fir(input, &self.taps, &mut self.state))
    }
}

/// A node that implements a generic FIR filter which operates on a batch of
/// samples at a time.
///
/// # Arguments
///
/// * `taps` - FIR filter tap `Vec<Complex<T>>`.
/// * `state` - FIR filter initial state `Vec<Complex<T>>`.
///
/// # Examples
///
/// ```
/// use comms_rs::filter::fir_node::*;
/// use num::Complex;
///
/// let taps = vec![
///     Complex::new(0.2, 0.0),
///     Complex::new(0.6, 0.0),
///     Complex::new(0.6, 0.0),
///     Complex::new(0.2, 0.0),
/// ];
///
/// let mut state = vec![
///     Complex::new(1.0, 0.0),
///     Complex::new(0.5, 0.0),
///     Complex::new(0.25, 0.0),
///     Complex::new(0.125, 0.0),
/// ];
///
/// let node = BatchFirNode::new(taps, Some(state));
/// ```
#[derive(Node)]
#[pass_by_ref]
pub struct BatchFirNode<T>
where
    T: Num + Copy,
{
    pub input: NodeReceiver<Vec<Complex<T>>>,
    taps: Vec<Complex<T>>,
    state: Vec<Complex<T>>,
    pub sender: NodeSender<Vec<Complex<T>>>,
}

impl<T> BatchFirNode<T>
where
    T: Num + Copy,
{
    /// Constructs a new `BatchFirNode<T>` with optional user defined initial
    /// state.
    ///
    /// # Arguments
    ///
    /// * `taps` - FIR filter tap Vec[Complex<T>].
    /// * `state` - Initial state for the internal filter state and memory. If
    ///    set to None, defaults to zeros.
    ///
    /// # Examples
    ///
    /// ```
    /// use comms_rs::filter::fir_node::*;
    /// use num::Complex;
    ///
    /// let taps = vec![
    ///     Complex::new(0.2, 0.0),
    ///     Complex::new(0.6, 0.0),
    ///     Complex::new(0.6, 0.0),
    ///     Complex::new(0.2, 0.0),
    /// ];
    ///
    /// let mut state = vec![
    ///     Complex::new(1.0, 0.0),
    ///     Complex::new(0.5, 0.0),
    ///     Complex::new(0.25, 0.0),
    ///     Complex::new(0.125, 0.0),
    /// ];
    ///
    /// let node = BatchFirNode::new(taps, Some(state));
    /// ```
    pub fn new(taps: Vec<Complex<T>>, state: Option<Vec<Complex<T>>>) -> Self {
        match state {
            Some(st) => BatchFirNode {
                taps,
                state: st,
                input: Default::default(),
                sender: Default::default(),
            },
            None => {
                let len = taps.len();
                BatchFirNode {
                    taps,
                    state: vec![Complex::zero(); len],
                    input: Default::default(),
                    sender: Default::default(),
                }
            }
        }
    }

    /// Runs the `BatchFirNode<T>`.  Produces either a new `Vec<Complex<T>>`
    /// batch of samples or a `NodeError`.
    pub fn run(
        &mut self,
        input: &[Complex<T>],
    ) -> Result<Vec<Complex<T>>, NodeError> {
        Ok(batch_fir(input, &self.taps, &mut self.state))
    }
}

#[cfg(test)]
mod test {
    use crate::filter::fir_node;
    use crate::prelude::*;
    use crossbeam_channel as channel;
    use num::Complex;
    use num::Zero;
    use std::thread;
    use std::time::Instant;

    #[test]
    // A test to verify the FirNode correctly implements an FIR filter.
    fn test_fir_node() {
        #[derive(Node)]
        struct SomeSamples {
            samples: Vec<Complex<i16>>,
            sender: NodeSender<Complex<i16>>,
        }

        impl SomeSamples {
            pub fn new(samples: Vec<Complex<i16>>) -> Self {
                SomeSamples {
                    samples,
                    sender: Default::default(),
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
            Complex::new(1, 2),
            Complex::new(3, 4),
            Complex::new(5, 6),
            Complex::new(7, 8),
            Complex::new(9, 0),
            Complex::zero(),
            Complex::zero(),
            Complex::zero(),
            Complex::zero(),
            Complex::zero(),
        ]);

        let mut mynode = fir_node::FirNode::new(
            vec![
                Complex::new(9, 0),
                Complex::new(8, 7),
                Complex::new(6, 5),
                Complex::new(4, 3),
                Complex::new(2, 1),
            ],
            None,
        );

        #[derive(Node)]
        pub struct CheckNode {
            pub input: NodeReceiver<Complex<i16>>,
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
                input: Complex<i16>,
            ) -> Result<(), NodeError> {
                if self.state.len() == 9 {
                    assert_eq!(
                        self.state,
                        vec![
                            Complex::new(9, 18),
                            Complex::new(21, 59),
                            Complex::new(37, 124),
                            Complex::new(57, 205),
                            Complex::new(81, 204),
                            Complex::new(78, 196),
                            Complex::new(62, 115),
                            Complex::new(42, 50),
                            Complex::new(18, 9)
                        ]
                    );
                } else {
                    self.state.push(input);
                }
                Ok(())
            }
        }

        let mut check_node = CheckNode::new();

        connect_nodes!(source, sender, mynode, input);
        connect_nodes!(mynode, sender, check_node, input);
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

    #[test]
    // A test to verify the BatchFirNode correctly implements an FIR filter.
    fn test_batch_fir_node() {
        #[derive(Node)]
        struct SomeSamples {
            samples: Vec<Complex<i16>>,
            pub sender: NodeSender<Vec<Complex<i16>>>,
        }

        impl SomeSamples {
            pub fn new(samples: Vec<Complex<i16>>) -> Self {
                SomeSamples {
                    samples,
                    sender: Default::default(),
                }
            }

            pub fn run(&mut self) -> Result<Vec<Complex<i16>>, NodeError> {
                if self.samples.is_empty() {
                    Ok(vec![Complex::zero(), Complex::zero()])
                } else {
                    let s1 = self.samples.remove(0);
                    let s2 = self.samples.remove(0);
                    Ok(vec![s1, s2])
                }
            }
        }

        let mut source = SomeSamples::new(vec![
            Complex::new(1, 2),
            Complex::new(3, 4),
            Complex::new(5, 6),
            Complex::new(7, 8),
            Complex::new(9, 0),
            Complex::zero(),
            Complex::zero(),
            Complex::zero(),
            Complex::zero(),
            Complex::zero(),
        ]);

        let mut mynode = fir_node::BatchFirNode::new(
            vec![
                Complex::new(9, 0),
                Complex::new(8, 7),
                Complex::new(6, 5),
                Complex::new(4, 3),
                Complex::new(2, 1),
            ],
            Some(vec![Complex::zero(); 5]),
        );

        #[derive(Node)]
        #[pass_by_ref]
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
                input: &[Complex<i16>],
            ) -> Result<(), NodeError> {
                if self.state.len() == 9 {
                    assert_eq!(
                        self.state,
                        vec![
                            Complex::new(9, 18),
                            Complex::new(21, 59),
                            Complex::new(37, 124),
                            Complex::new(57, 205),
                            Complex::new(81, 204),
                            Complex::new(78, 196),
                            Complex::new(62, 115),
                            Complex::new(42, 50),
                            Complex::new(18, 9),
                            Complex::new(0, 0),
                        ]
                    );
                } else {
                    self.state.append(&mut (*input).to_vec());
                }
                Ok(())
            }
        }

        let mut check_node = CheckNode::new();

        connect_nodes!(source, sender, mynode, input);
        connect_nodes!(mynode, sender, check_node, input);
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
