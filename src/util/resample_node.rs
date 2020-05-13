use crate::prelude::*;
use num::Zero;

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
    pub output: NodeSender<Vec<T>>,
}

impl<T> DecimateNode<T>
where
    T: Copy + Send,
{
    pub fn new(dec_rate: usize) -> Self {
        DecimateNode {
            dec_rate,
            input: Default::default(),
            output: Default::default(),
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

/// A simple node to upsample the input signal.
///
/// This node will upsample the input stream by a factor of `ups_rate`, meaning
/// that for every input sample there will be `ups_rate` output sample.
#[derive(Node)]
#[pass_by_ref]
pub struct UpsampleNode<T>
where
    T: Copy + Send + Zero,
{
    pub input: NodeReceiver<Vec<T>>,
    ups_rate: usize,
    pub output: NodeSender<Vec<T>>,
}

impl<T> UpsampleNode<T>
where
    T: Copy + Send + Zero,
{
    pub fn new(ups_rate: usize) -> Self {
        UpsampleNode {
            ups_rate,
            input: Default::default(),
            output: Default::default(),
        }
    }

    pub fn run(&mut self, signal: &[T]) -> Result<Vec<T>, NodeError> {
        Ok(self.upsample(signal))
    }

    /// This is the decimation function.
    ///
    /// A sample of `data` will be zero-padded by a factor of `ups_rate`, to have total samples
    /// equal to ups_rate * data.len()
    ///
    /// If the upsampling rate is equal to zero or one, the original data is returned as-is.
    ///
    /// # Arguments
    ///
    /// * `data` - The input data to be zero padded
    ///
    /// # Examples
    ///
    /// ```
    /// use comms_rs::util::resample_node::UpsampleNode;
    ///
    /// let node = UpsampleNode::new(3);
    ///
    /// let data = vec![1, 2, 3];
    /// assert_eq!(node.upsample(&data), vec![1, 0, 0, 2, 0, 0, 3, 0, 0]);
    /// ```
    pub fn upsample(&self, data: &[T]) -> Vec<T> {
        if self.ups_rate == 0 || self.ups_rate == 1 {
            return data.to_vec();
        }
        data.iter()
            .flat_map(|sample| {
                let mut tmp_vec = vec![T::zero(); self.ups_rate];
                tmp_vec[0] = *sample;
                tmp_vec
            })
            .collect()
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

    #[test]
    fn test_upsample() {
        let v1 = vec![1, 2, 3, 4];
        // Simple upsample test
        let ups_node = UpsampleNode::new(4);
        assert_eq!(
            ups_node.upsample(&v1),
            vec![1, 0, 0, 0, 2, 0, 0, 0, 3, 0, 0, 0, 4, 0, 0, 0]
        );

        // Check a rate that's zero; the data should be unchanged
        let ups_node = UpsampleNode::new(0);
        assert_eq!(ups_node.upsample(&v1), v1);

        // Check a rate that's one; the data should be unchanged
        let ups_node = UpsampleNode::new(1);
        assert_eq!(ups_node.upsample(&v1), v1);
    }
}
