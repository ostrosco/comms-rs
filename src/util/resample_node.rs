use prelude::*;

// A simple node to decimate the input.
create_node!(
    DecimateNode<T>: Vec<T>,
    [dec_rate: usize],
    [recv: Vec<T>],
    |node: &mut DecimateNode<T>, signal: Vec<T>| {
        Ok(node.decimate(&signal))
    },
    T: Copy,
);

impl<T> DecimateNode<T>
where
    T: Copy,
{
    fn decimate(&self, data: &[T]) -> Vec<T> {
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
