use crate::prelude::*;

use crate::filter::fir::*;
use num::complex::Complex;
use num::Zero;
use num_traits::Num;

/// A node that implements a generic FIR filter.
#[derive(Node)]
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
    pub fn run(&mut self, input: &Complex<T>) -> Result<Complex<T>, NodeError> {
        Ok(fir(input, &self.taps, &mut self.state))
    }
}

/// A node that implements a generic FIR filter.  Operates on a batch of
/// samples at a time.
#[derive(Node)]
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
    pub fn run(
        &mut self,
        input: &[Complex<T>],
    ) -> Result<Vec<Complex<T>>, NodeError> {
        Ok(batch_fir(input, &self.taps, &mut self.state))
    }
}

/// Constructs a new `FirNode<T>` with initial state set to zeros.
///
/// Arguments:
///     taps  - FIR filter tap Vec[Complex<T>].
///     state - Initial state for the internal filter state and memory.
pub fn fir_node<T>(taps: Vec<Complex<T>>) -> FirNode<T>
where
    T: Num + Copy,
{
    let len = taps.len();
    FirNode::new(taps, vec![Complex::zero(); len])
}

/// Constructs a new `FirNode<T>` with user defined initial state.
///
/// Arguments:
///     taps  - FIR filter tap Vec[Complex<T>].
///     state - Initial state for the internal filter state and memory.
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
/// Arguments:
///     taps  - FIR filter tap Vec[Complex<T>].
///     state - Initial state for the internal filter state and memory.
pub fn batch_fir_node<T>(taps: Vec<Complex<T>>) -> BatchFirNode<T>
where
    T: Num + Copy,
{
    let len = taps.len();
    BatchFirNode::new(taps, vec![Complex::zero(); len])
}

/// Constructs a new `BatchFirNode<T>` with user defined initial state.
///
/// Arguments:
///     taps  - FIR filter tap Vec[Complex<T>].
///     state - Initial state for the internal filter state and memory.
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
                if self.samples.len() == 0 {
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
                input: &Complex<i16>,
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
                    self.state.push(*input);
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
                if self.samples.len() == 0 {
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
                    self.state.append(&mut input.clone().to_vec());
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
