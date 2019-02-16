use rand::distributions::uniform::SampleUniform;
use rand::distributions::{Normal, Uniform};
use rand::{FromEntropy, Rng, StdRng};

use crate::prelude::*;

/// A node that will generate uniformly-distributed random numbers.
///
/// This node can generate uniformly distributed random numbers over a given
/// range of values, using a given random number generator.  It will produce a
/// single value at a time.
///
/// # Arguments
///
/// * `rng` - Random number generator to use for sampling
/// * `dist` - `Uniform` distribution to draw samples from
///
/// # Examples
///
/// ```
/// use rand::rngs::StdRng;
/// use rand::FromEntropy;
/// use rand::distributions::Uniform;
/// use comms_rs::util::rand_node::UniformNode;
///
/// let rng = StdRng::from_entropy();
/// let dist = Uniform::new(0, 2);
/// let node = UniformNode::new(rng, dist);
/// ```
#[derive(Node)]
pub struct UniformNode<T>
where
    T: SampleUniform + Clone,
{
    rng: StdRng,
    dist: Uniform<T>,
    pub sender: NodeSender<T>,
}

impl<T> UniformNode<T>
where
    T: SampleUniform + Clone,
{
    /// Runs the `UniformNode`.  Produces either a new `f64` sample drawn from
    /// the stored random number generator or produces a `NodeError`.
    pub fn run(&mut self) -> Result<T, NodeError> {
        Ok(self.rng.sample(&self.dist))
    }
}

/// A node that will generate normally-distributed random numbers.
///
/// This node can generate normally distributed random numbers using a passed
/// Normal distribution parameter set using a given random number generator.
/// It produces a single value at a time.
///
/// # Arguments
///
/// * `rng` - Random number generator to use for sampling
/// * `dist` - `Normal` distribution to draw samples from
///
/// # Examples
///
/// ```
/// use rand::rngs::StdRng;
/// use rand::FromEntropy;
/// use rand::distributions::Normal;
/// use comms_rs::util::rand_node::NormalNode;
///
/// let rng = StdRng::from_entropy();
/// let mean_value = 0.0;
/// let standard_deviation = 1.0;
/// let dist = Normal::new(mean_value, standard_deviation);
/// let node = NormalNode::new(rng, dist);
/// ```
#[derive(Node)]
pub struct NormalNode {
    rng: StdRng,
    dist: Normal,
    pub sender: NodeSender<f64>,
}

impl NormalNode {
    /// Runs the `NormalNode`.  Produces either a new `f64` sample drawn from
    /// the stored random number generator or produces a `NodeError`.
    pub fn run(&mut self) -> Result<f64, NodeError> {
        Ok(self.rng.sample(&self.dist))
    }
}

/// Builds a closure for generating random numbers with a Normal distribution.
///
/// Provides a shorthand for getting a node that produces normally distributed
/// random numbers with a given mean and standard deviation.
///
/// # Arguments
///
/// * `mu` - Mean value for `Normal` distribution
/// * `std_dev` - Standard deviation for `Normal` distribution
///
/// # Examples
///
/// ```
/// use comms_rs::util::rand_node::normal;
///
/// let mu = 0.0_f64;
/// let std_dev = 1.0_f64;
/// let node = normal(mu, std_dev);
/// ```
pub fn normal(mu: f64, std_dev: f64) -> NormalNode {
    let rng = StdRng::from_entropy();
    let norm = Normal::new(mu, std_dev);
    NormalNode::new(rng, norm)
}

/// Builds a closure for generating random numbers with a Uniform distribution.
///
/// Provides a shorthand for getting a node that produces uniformly distributed
/// random numbers with a given start and end range.
///
/// # Arguments
///
/// * `start` - Lower bound (inclusive) of `Uniform` range
/// * `end` - Upper bound (exclusive) of `Uniform` range
///
/// # Examples
///
/// ```
/// use comms_rs::util::rand_node::uniform;
///
/// let start = 0.0_f64;
/// let end = 1.0_f64;
/// let node = uniform(start, end);
/// ```
pub fn uniform<T: SampleUniform + Clone>(start: T, end: T) -> UniformNode<T> {
    let rng = StdRng::from_entropy();
    let uniform = Uniform::new(start, end);
    UniformNode::new(rng, uniform)
}

/// Builds a closure for generating 0 or 1 with a Uniform distrubition.
///
/// # Examples
///
/// ```
/// use comms_rs::util::rand_node::random_bit;
///
/// let node = random_bit();
/// ```
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
        #[derive(Node)]
        struct CheckNode {
            recv: NodeReceiver<f64>,
        }

        impl CheckNode {
            pub fn run(&mut self, x: f64) -> Result<(), NodeError> {
                assert!(x >= 1.0 && x <= 2.0);
                Ok(())
            }
        }
        let mut check_node = CheckNode::new();
        connect_nodes!(uniform_node, sender, check_node, recv);
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
        #[derive(Node)]
        struct CheckNode {
            recv: NodeReceiver<u8>,
        }

        impl CheckNode {
            pub fn run(&mut self, x: u8) -> Result<(), NodeError> {
                assert!(x == 0u8 || x == 1u8);
                Ok(())
            }
        }
        let mut check_node = CheckNode::new();
        connect_nodes!(bit_node, sender, check_node, recv);
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
