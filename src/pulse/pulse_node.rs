use crate::filter::fir::*;
use crate::prelude::*;

use num::complex::Complex;
use num::Zero;
use num_traits::Num;

/// A node that implements a pulse shaping filter of some sort.
#[derive(Node)]
#[pass_by_ref]
pub struct PulseNode<T>
where
    T: Num + Copy,
{
    pub input: NodeReceiver<Complex<T>>,
    taps: Vec<Complex<T>>,
    sam_per_sym: usize,
    state: Vec<Complex<T>>,
    pub sender: NodeSender<Vec<Complex<T>>>,
}

impl<T> PulseNode<T>
where
    T: Num + Copy,
{
    pub fn run(
        &mut self,
        input: &Complex<T>,
    ) -> Result<Vec<Complex<T>>, NodeError> {
        let mut output = Vec::new();
        output.push(fir(input, &self.taps, &mut self.state));
        for _ in 0..&self.sam_per_sym - 1 {
            output.push(fir(&Complex::zero(), &self.taps, &mut self.state));
        }
        Ok(output)
    }
}

/// Constructs a new `PulseNode<T>` with initial state set to zeros.
///
/// Arguments:
///     taps        - FIR filter tap Vec[Complex<T>].
///     sam_per_sym - Number of samples per symbol in the output.
///     state       - Initial state for the internal filter state and memory.
pub fn pulse_node<T>(taps: Vec<Complex<T>>, sam_per_sym: usize) -> PulseNode<T>
where
    T: Num + Copy,
{
    let len = taps.len();
    PulseNode::new(taps, sam_per_sym, vec![Complex::zero(); len])
}

#[cfg(test)]
mod test {
    use crate::prelude::*;
    use crate::pulse::pulse_node::*;
    use crate::util::math::*;
    use num::Complex;
    use std::thread;
    use std::time::Instant;

    #[test]
    fn test_rect_node() {
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
            Complex::new(-1, -1),
            Complex::new(1, -1),
            Complex::new(1, -1),
            Complex::new(1, 1),
            Complex::new(-1, 1),
        ]);

        let sam_per_sym = 4;
        let taps = rect_taps(sam_per_sym).unwrap();
        let mut mynode = pulse_node(taps, sam_per_sym);

        #[derive(Node)]
        pub struct CheckNode {
            pub input: NodeReceiver<Vec<Complex<i16>>>,
            state: Vec<Complex<i16>>,
        }

        impl CheckNode {
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
