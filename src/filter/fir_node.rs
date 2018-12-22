use crate::prelude::*;

use crate::filter::fir::*;
use num::complex::Complex;
use num::Zero;
use num_traits::Num;

/// A node that implements a generic FIR filter.
create_node!(
    FirNode<T>: Complex<T>,
    [taps: Vec<Complex<T>>, state: Vec<Complex<T>>],
    [input: Complex<T>],
    |node: &mut FirNode<T>, input: Complex<T>| {
        Ok(fir(input, &node.taps, &mut node.state))
    },
    T: Num + Copy,
);

/// A node that implements a generic FIR filter.  Operates on a batch of
/// samples at a time.
create_node!(
    BatchFirNode<T>: Vec<Complex<T>>,
    [taps: Vec<Complex<T>>, state: Vec<Complex<T>>],
    [input: Vec<Complex<T>>],
    |node: &mut BatchFirNode<T>, input: Vec<Complex<T>>| {
        Ok(batch_fir(input, &node.taps, &mut node.state))
    },
    T: Num + Copy,
);

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
    use crossbeam::{Receiver, Sender};
    use crossbeam_channel as channel;
    use crate::filter::fir_node;
    use num::Complex;
    use num::Zero;
    use crate::prelude::*;
    use std::thread;
    use std::time::Instant;

    #[test]
    // A test to verify the FirNode correctly implements an FIR filter.
    fn test_fir_node() {
        create_node!(
            SomeSamples: Complex<i16>,
            [samples: Vec<Complex<i16>>],
            [],
            |node: &mut Self| -> Result<Complex<i16>, NodeError> {
                if node.samples.len() == 0 {
                    Ok(Complex::zero())
                } else {
                    Ok(node.samples.remove(0))
                }
            }
        );

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

        create_node!(
            CheckNode: (),
            [state: Vec<Complex<i16>>],
            [recv: Complex<i16>],
            |node: &mut CheckNode, x| -> Result<(), NodeError> {
                if node.state.len() == 9 {
                    assert_eq!(
                        node.state,
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
                    node.state.push(x);
                }
                Ok(())
            }
        );

        let mut check_node = CheckNode::new(Vec::new());

        connect_nodes!(source, mynode, input);
        connect_nodes!(mynode, check_node, recv);
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
        create_node!(
            SomeSamples: Vec<Complex<i16>>,
            [samples: Vec<Complex<i16>>],
            [],
            |node: &mut Self| -> Result<Vec<Complex<i16>>, NodeError> {
                if node.samples.len() == 0 {
                    Ok(vec![Complex::zero(), Complex::zero()])
                } else {
                    let s1 = node.samples.remove(0);
                    let s2 = node.samples.remove(0);
                    Ok(vec![s1, s2])
                }
            }
        );

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

        create_node!(
            CheckNode: (),
            [state: Vec<Complex<i16>>],
            [recv: Vec<Complex<i16>>],
            |node: &mut CheckNode, mut x| -> Result<(), NodeError> {
                if node.state.len() == 10 {
                    assert_eq!(
                        node.state,
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
                            Complex::new(0, 0)
                        ]
                    );
                } else {
                    node.state.append(&mut x);
                }
                Ok(())
            }
        );

        let mut check_node = CheckNode::new(Vec::new());

        connect_nodes!(source, mynode, input);
        connect_nodes!(mynode, check_node, recv);
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
