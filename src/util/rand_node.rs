use rand::distributions::uniform::SampleUniform;
use rand::distributions::{Normal, Uniform};
use rand::{FromEntropy, Rng, StdRng};

use crate::prelude::*;

create_node!(
    #[doc="A node that will generate uniformly-distributed random numbers."]
    UniformNode<T>: T,
    [rng: StdRng, dist: Uniform<T>],
    [],
    |node: &mut UniformNode<T>| -> Result<T, NodeError> {
        Ok(node.rng.sample(&node.dist))
    },
    T: SampleUniform + Clone,
);

create_node!(
    #[doc = "A node that will generate normally-distributed random numbers."]
    NormalNode: f64,
    [rng: StdRng, dist: Normal],
    [],
    |node: &mut NormalNode| -> Result<f64, NodeError> {
        Ok(node.rng.sample(&node.dist))
    }
);

/// Builds a closure for generating random numbers with a Normal distribution.
pub fn normal(mu: f64, std_dev: f64) -> NormalNode {
    let rng = StdRng::from_entropy();
    let norm = Normal::new(mu, std_dev);
    NormalNode::new(rng, norm)
}

/// Builds a closure for generating random numbers with a Uniform distribution.
pub fn uniform<T: SampleUniform + Clone>(start: T, end: T) -> UniformNode<T> {
    let rng = StdRng::from_entropy();
    let uniform = Uniform::new(start, end);
    UniformNode::new(rng, uniform)
}

/// Builds a closure for generating 0 or 1 with a Uniform distrubition.
pub fn random_bit() -> UniformNode<u8> {
    uniform(0u8, 2u8)
}

#[cfg(test)]
mod test {
    use crate::util::rand_node;
    use std::thread;
    use std::time::Instant;

    use crate::prelude::*;

    #[test]
    // A basic test that just makes sure the node doesn't crash.
    fn test_normal() {
        let mut norm_node = rand_node::normal(0.0, 1.0);
        let check = thread::spawn(move || {
            let now = Instant::now();
            loop {
                norm_node.call().unwrap();
                if now.elapsed().as_secs() > 1 {
                    break;
                }
            }
        });
        assert!(check.join().is_ok());
    }

    #[test]
    // A basic test to ensure that the uniform node can be configured
    // correctly and generates numbers within the correct range.
    fn test_uniform() {
        let mut uniform_node = rand_node::uniform(1.0, 2.0);
        create_node!(CheckNode: (), [], [recv: f64], |_,
                                                      x|
         -> Result<
            (),
            NodeError,
        > {
            assert!(x >= 1.0 && x <= 2.0);
            Ok(())
        });
        let mut check_node = CheckNode::new();
        connect_nodes!(uniform_node, check_node, recv);
        start_nodes!(uniform_node);
        let check = thread::spawn(move || {
            let now = Instant::now();
            loop {
                check_node.call().unwrap();
                if now.elapsed().as_secs() > 1 {
                    break;
                }
            }
        });
        assert!(check.join().is_ok());
    }

    #[test]
    // A basic test to ensure that the random_bit node can be configured
    // correctly and generates only 0s and 1s.
    fn test_random_bit() {
        let mut bit_node = rand_node::random_bit();
        create_node!(CheckNode: (), [], [recv: u8], |_,
                                                     x|
         -> Result<
            (),
            NodeError,
        > {
            assert!(x == 0u8 || x == 1u8);
            Ok(())
        });
        let mut check_node = CheckNode::new();
        connect_nodes!(bit_node, check_node, recv);
        start_nodes!(bit_node);
        let check = thread::spawn(move || {
            let now = Instant::now();
            loop {
                check_node.call().unwrap();
                if now.elapsed().as_secs() > 1 {
                    break;
                }
            }
        });
        assert!(check.join().is_ok());
    }
}
