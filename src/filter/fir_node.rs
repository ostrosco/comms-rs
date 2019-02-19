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
/// let node = FirNode::new(taps, state);
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
/// let node = BatchFirNode::new(taps, state);
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
    /// Runs the `BatchFirNode<T>`.  Produces either a new `Vec<Complex<T>>`
    /// batch of samples or a `NodeError`.
    pub fn run(
        &mut self,
        input: &[Complex<T>],
    ) -> Result<Vec<Complex<T>>, NodeError> {
        Ok(batch_fir(input, &self.taps, &mut self.state))
    }
}

/// Constructs a new `FirNode<T>` with initial state set to zeros.
///
/// # Arguments
///
/// * `taps` - FIR filter tap Vec[Complex<T>].
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
/// let node = fir_node(taps);
/// ```
pub fn fir_node<T>(taps: Vec<Complex<T>>) -> FirNode<T>
where
    T: Num + Copy,
{
    let len = taps.len();
    FirNode::new(taps, vec![Complex::zero(); len])
}

/// Constructs a new `FirNode<T>` with user defined initial state.
///
/// # Arguments
///
/// * `taps` - FIR filter tap Vec[Complex<T>].
/// * `state` - Initial state for the internal filter state and memory.
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
/// let node = fir_node_with_state(taps, state);
/// ```
pub fn fir_node_with_state<T>(
    taps: Vec<Complex<T>>,
    state: Vec<Complex<T>>,
) -> FirNode<T>
where
    T: Num + Copy,
{
    FirNode::new(taps, state)
}

/// Constructs a new `BatchFirNode<T>` with initial state set to zeros.
///
/// # Arguments
///
/// * `taps` - FIR filter tap Vec[Complex<T>].
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
/// let node = batch_fir_node(taps);
/// ```
pub fn batch_fir_node<T>(taps: Vec<Complex<T>>) -> BatchFirNode<T>
where
    T: Num + Copy,
{
    let len = taps.len();
    BatchFirNode::new(taps, vec![Complex::zero(); len])
}

/// Constructs a new `BatchFirNode<T>` with user defined initial state.
///
/// # Arguments
///
/// * `taps` - FIR filter tap Vec[Complex<T>].
/// * `state` - Initial state for the internal filter state and memory.
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
/// let node = batch_fir_node_with_state(taps, state);
/// ```
pub fn batch_fir_node_with_state<T>(
    taps: Vec<Complex<T>>,
    state: Vec<Complex<T>>,
) -> BatchFirNode<T>
where
    T: Num + Copy,
{
    BatchFirNode::new(taps, state)
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

        let mut mynode = fir_node::fir_node(vec![
            Complex::new(9, 0),
            Complex::new(8, 7),
            Complex::new(6, 5),
            Complex::new(4, 3),
            Complex::new(2, 1),
        ]);

        #[derive(Node)]
        pub struct CheckNode {
            pub input: NodeReceiver<Complex<i16>>,
            state: Vec<Complex<i16>>,
        }

        impl CheckNode {
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

        let mut check_node = CheckNode::new(Vec::new());

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

        let mut mynode = fir_node::batch_fir_node_with_state(
            vec![
                Complex::new(9, 0),
                Complex::new(8, 7),
                Complex::new(6, 5),
                Complex::new(4, 3),
                Complex::new(2, 1),
            ],
            vec![Complex::zero(); 5],
        );

        #[derive(Node)]
        #[pass_by_ref]
        pub struct CheckNode {
            pub input: NodeReceiver<Vec<Complex<i16>>>,
            state: Vec<Complex<i16>>,
        }

        impl CheckNode {
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

        let mut check_node = CheckNode::new(Vec::new());

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
