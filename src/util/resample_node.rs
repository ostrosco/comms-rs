use crate::prelude::*;

/// A simple node to decimate the input signal.
///
/// This node will decimate the input stream by a factor of `dec_rate`, meaning
/// that for every `dec_rate` input samples there will be 1 output sample.
#[derive(Node)]
#[pass_by_ref]
pub struct DecimateNode<T>
where
    T: Copy + Send,
{
    pub input: NodeReceiver<Vec<T>>,
    dec_rate: usize,
    pub sender: NodeSender<Vec<T>>,
}

impl<T> DecimateNode<T>
where
    T: Copy + Send,
{
    pub fn new(dec_rate: usize) -> Self {
        DecimateNode {
            dec_rate,
            input: Default::default(),
            sender: Default::default(),
        }
    }

    pub fn run(&mut self, signal: &[T]) -> Result<Vec<T>, NodeError> {
        Ok(self.decimate(signal))
    }

    /// This is the decimation function.
    ///
    /// A slice of `data` will be reduced by a factor of `dec_rate`.
    ///
    /// # Arguments
    ///
    /// * `data` - The input data to be reduced down
    ///
    /// # Examples
    ///
    /// ```
    /// use comms_rs::util::resample_node::DecimateNode;
    ///
    /// let node = DecimateNode::new(3);
    ///
    /// let data = vec![1, 2, 3, 4, 5, 6, 7, 8];
    /// assert_eq!(node.decimate(&data), vec![1, 4, 7]);
    /// ```
    pub fn decimate(&self, data: &[T]) -> Vec<T> {
        let mut ix = 0;
        if self.dec_rate == 0 || self.dec_rate == 1 {
            return data.to_vec();
        }
        let new_size = (data.len() / self.dec_rate + 1) as usize;
        let mut data_dec = Vec::<T>::with_capacity(new_size);
        while ix < data.len() {
            data_dec.push(data[ix]);
            ix += self.dec_rate;
        }
        data_dec
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_decimate() {
        let v1 = vec![1, 2, 3, 4, 5, 6];
        // First, check a simple decimate.
        let dec_node = DecimateNode::new(2);
        assert_eq!(dec_node.decimate(&v1), vec![1, 3, 5]);

        // Next, check a decimate with a rate longer than the data.
        let dec_node = DecimateNode::new(100);
        assert_eq!(dec_node.decimate(&v1), vec![1]);

        // Check a decimate with 0. The data should be unchanged.
        let dec_node = DecimateNode::new(0);
        assert_eq!(dec_node.decimate(&v1), v1);

        // Check a decimate with 1. The data should be unchanged.
        let dec_node = DecimateNode::new(1);
        assert_eq!(dec_node.decimate(&v1), v1);
    }
}
