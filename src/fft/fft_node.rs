#[macro_use]
use node::Node;
use crossbeam::{Receiver, Sender};

create_node!(FFTNode, Fn(Vec<i16>) -> Vec<i16>, recv);

pub fn init() -> impl Node
{
    FFTNode::new(|x| x)
}

#[cfg(test)]
mod test {
    #[test]
    fn test_fft_node() {
        use node::Node;
        use crossbeam::{Receiver, Sender};
        use fft::fft_node;
        let fft_node = fft_node::init();
    }
}
