use crossbeam::Sender;
use node::Node;
use rand::distributions::uniform::SampleUniform;
use rand::distributions::{Distribution, Normal, Uniform};
use rand::{FromEntropy, Rng, StdRng};

// A subset of common nodes for random data types. We will create more as
// needed.
create_node!(RandomU8Node, FnMut() -> u8);
create_node!(RandomI16Node, FnMut() -> i16);
create_node!(RandomF64Node, FnMut() -> f64);

/// Builds a closure for generating random numbers with a Normal distribution.
pub fn normal(mu: f64, std_dev: f64) -> impl FnMut() -> f64 {
    let norm = Normal::new(mu, std_dev);
    let mut rng = StdRng::from_entropy();
    move || {
        let val: f64 = rng.sample(norm);
        val
    }
}

/// Builds a closure for generating random numbers with a Uniform distribution.
pub fn uniform<T: SampleUniform>(start: T, end: T) -> impl FnMut() -> T {
    let uniform = Uniform::new(start, end);
    let mut rng = StdRng::from_entropy();
    move || {
        let val: T = uniform.sample(&mut rng);
        val
    }
}

/// Builds a closure for generating 0 or 1 with a Uniform distrubition.
pub fn random_bit() -> impl FnMut() -> u8 {
    uniform(0u8, 2u8)
}

#[cfg(test)]
mod test {
    use crossbeam::{Receiver, Sender};
    use crossbeam_channel as channel;
    use node::Node;
    use std::thread;
    use std::time::Instant;
    use util::rand_node::{self, RandomF64Node, RandomU8Node};

    #[test]
    // A basic test that just makes sure the node doesn't crash.
    fn test_normal() {
        let mut norm_node = RandomF64Node::new(rand_node::normal(0.0, 1.0));
        let check = thread::spawn(move || {
            let now = Instant::now();
            loop {
                norm_node.run_node();
                if now.elapsed().as_secs() > 1 {
                    break;
                }
            }
        });
        assert!(check.join().is_ok());
    }

    #[test]
    fn test_uniform() {
        let mut uniform_node = RandomF64Node::new(rand_node::uniform(1.0, 2.0));
        create_node!(CheckNode, Fn(f64) -> (), recv);
        let mut check_node = CheckNode::new(|x| {
            assert!(x >= 1.0 && x <= 2.0);
        });
        connect_nodes!(uniform_node, check_node, recv);
        start_nodes!(uniform_node);
        let check = thread::spawn(move || {
            let now = Instant::now();
            loop {
                check_node.run_node();
                if now.elapsed().as_secs() > 1 {
                    break;
                }
            }
        });
        assert!(check.join().is_ok());
    }

    #[test]
    fn test_random_bit() {
        let mut bit_node = RandomU8Node::new(rand_node::random_bit());
        create_node!(CheckNode, Fn(u8) -> (), recv);
        let mut check_node = CheckNode::new(|x| {
            assert!(x == 0u8 || x == 1u8);
        });
        connect_nodes!(bit_node, check_node, recv);
        start_nodes!(bit_node);
        let check = thread::spawn(move || {
            let now = Instant::now();
            loop {
                check_node.run_node();
                if now.elapsed().as_secs() > 1 {
                    break;
                }
            }
        });
        assert!(check.join().is_ok());
    }
}
