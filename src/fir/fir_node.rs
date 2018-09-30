use crossbeam::{Receiver, Sender};
use node::Node;

extern crate num; // 0.2.0

use num::complex::Complex;

/// A node that implements a generic FIR filter.
create_node!(
    FirNode: Complex<i16>,
    [taps: Vec<Complex<i16>>, state: Vec<Complex<i16>>],
    [input: Complex<i16>],
    |node: &mut FirNode, input: Complex<i16>| node.run(input)
);

/// Implementation of run for the FirNode.
impl FirNode {
    fn run(&mut self, input: Complex<i16>) -> Complex<i16> {
        self.state.rotate_right(1);
        self.state[0] = input;
        let output = self.taps.iter().zip(self.state.iter())
            .map(|(x, y)| *x * *y).sum();
        output
    }
}

/// Constructs a new `FirNode<T>`.
///
/// Arguments:
///     taps  - FIR filter tap Vec[T].
///     state - Initial state for the internal filter state and memory.
pub fn fir(taps: Vec<Complex<i16>>, state: Vec<Complex<i16>>) -> FirNode {
    FirNode::new(taps, state)
}

#[cfg(test)]
mod test {
    use crossbeam::{Receiver, Sender};
    use crossbeam_channel as channel;
    use node::Node;
    use num::Complex;
    use num::Zero;
    use fir::fir_node;
    use std::thread;
    use std::time::Instant;

    #[test]
    // A test to verify the FirNode correctly implements an FIR filter.
    fn test_fir_node() {
        create_node!(
            SomeSamples: Complex<i16>,
            [samples: Vec<Complex<i16>>],
            [],
            |node: &mut Self| {
                if node.samples.len() == 0 {
                    Complex::zero()
                } else {
                    node.samples.remove(0)
                }
            });

        let mut source = SomeSamples::new(vec![Complex::new(1, 2),
                                               Complex::new(3, 4),
                                               Complex::new(5, 6),
                                               Complex::new(7, 8),
                                               Complex::new(9, 0),
                                               Complex::zero(),
                                               Complex::zero(),
                                               Complex::zero(),
                                               Complex::zero(),
                                               Complex::zero()]);

        let mut mynode = fir_node::fir(vec![Complex::new(9, 0),
                                            Complex::new(8, 7),
                                            Complex::new(6, 5),
                                            Complex::new(4, 3),
                                            Complex::new(2, 1)],
                                       vec![Complex::zero(),
                                            Complex::zero(),
                                            Complex::zero(),
                                            Complex::zero(),
                                            Complex::zero()]);

        create_node!(
            CheckNode: (),
            [state: Vec<Complex<i16>>],
            [recv: Complex<i16>],
            |node: &mut CheckNode, x| {

                if node.state.len() == 9 {
                    assert_eq!(
                        node.state,
                        vec![Complex::new(9, 18),
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
            }
        );

        let mut check_node = CheckNode::new(Vec::new());

        connect_nodes!(source, mynode, input);
        connect_nodes!(mynode, check_node, recv);
        start_nodes!(source, mynode);
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
