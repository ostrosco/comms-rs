//! Provides nodes for sending data to various external sources.
//!
//! Nodes will send data as bytes to the writer provided at initialization.
//! Currently the only supported receivable primitive type is Complex<i16>.
//! Complex<i16> will be written to the writer as first the real then
//! imaginary portions, with each item in host byte-order.

use crossbeam::{Receiver, Sender};
use node::Node;
use num::Complex;

use std::fs::File;
use std::io::{self, BufWriter, Write};
use std::mem;
use std::path::Path;

type IQSample = Complex<i16>;

/// Will send samples as interleaved 16-bit values in host byte-order to writer.
create_node!(
    IQOutput<W>: () where W: Write,
    [writer: W],
    [sample: IQSample],
    |node: &mut IQOutput<W>, sample: IQSample| node.run(sample)
);

impl<W: Write> IQOutput<W> {
    fn run(&mut self, samp: IQSample) {
        let bytes = self
            .writer
            .write(&complex_to_bytes(samp))
            .expect("failed to write sample to writer");
        if bytes != mem::size_of::<IQSample>() {
            panic!("did not write the expected number of bytes to writer");
        }
    }
}

/// Make an IQOutput node sending data to the given file.
///
/// # Example
///
/// ```
/// use comms_rs::output::raw_iq::iq_file_out;
///
/// let outnode = iq_file_out("/tmp/raw_iq.bin").expect("couldn't create file");
/// ```
pub fn iq_file_out<P: AsRef<Path>>(
    path: P,
) -> io::Result<IQOutput<impl Write>> {
    Ok(IQOutput::new(BufWriter::new(File::create(path)?)))
}

create_node!(
    IQBatchOutput<W>: () where W: Write,
    [writer: W],
    [samples: Vec<IQSample>],
    |node: &mut Self, samples: Vec<IQSample>| node.run(samples)
);

impl<W: Write> IQBatchOutput<W> {
    fn run(&mut self, samples: Vec<IQSample>) {
        let bytes: usize = samples
            .iter()
            .map(|samp| {
                self.writer
                    .write(&complex_to_bytes(*samp))
                    .expect("failed to write samples to file")
            }).sum();
        if bytes != mem::size_of::<IQSample>() * samples.len() {
            panic!("did not write the expected number of bytes to writer");
        }
    }
}

/// Make an IQBatchOutput node sending data to the given file.
///
/// # Example
///
/// ```
/// use comms_rs::output::raw_iq::iq_batch_file_out;
///
/// let outnode = iq_batch_file_out("/tmp/raw_iq.bin").expect("couldn't create file");
/// ```
pub fn iq_batch_file_out<P: AsRef<Path>>(
    path: P,
) -> io::Result<IQBatchOutput<impl Write>> {
    Ok(IQBatchOutput::new(File::create(path)?))
}

// copied from source of https://doc.rust-lang.org/std/primitive.i16.html#method.to_bytes
fn i16_to_bytes(i: i16) -> [u8; 2] {
    unsafe { mem::transmute(i) }
}

fn complex_to_bytes(c: Complex<i16>) -> [u8; mem::size_of::<Complex<i16>>()] {
    unsafe { mem::transmute(c) }
}

#[cfg(test)]
mod test {
    use output::raw_iq::*;

    #[test]
    /// Test that complex_to_bytes behaves in the expected manner.
    fn test_complex_to_bytes() {
        let c = Complex::new(0x1234, 0x5678);
        let bytes = complex_to_bytes(c);

        assert_eq!(bytes, [0x34, 0x12, 0x78, 0x56]);
    }

    #[test]
    /// Test that node correctly sends received data to writer.
    fn test_single_out_node() {
        let iterations = 100usize;

        let mut out: Vec<u8> = Vec::new();
        let expected: Vec<Complex<i16>> = (0..iterations as i16)
            .map(|i| Complex::new(i * 2, i * 2 + 1))
            .collect();
        {
            let mut node = IQOutput::new(&mut out);
            for item in expected.iter() {
                node.run(*item);
            }
        }

        assert_eq!(out.len(), iterations * mem::size_of::<IQSample>());
        for i in 0..iterations {
            assert_eq!(complex_to_bytes(expected[i]), out[(i * 4)..(i * 4 + 4)])
        }
    }

    #[test]
    /// Test that batch node correctly sends received data to writer.
    fn test_batch_out_node() {
        let iterations = 100usize;

        let mut out: Vec<u8> = Vec::new();
        let expected: Vec<Complex<i16>> = (0..iterations as i16)
            .map(|i| Complex::new(i * 2, i * 2 + 1))
            .collect();
        {
            let mut node = IQBatchOutput::new(&mut out);
            for _ in 0..iterations {
                node.run(expected.clone());
            }
        }

        assert_eq!(
            out.len(),
            iterations * iterations * mem::size_of::<IQSample>()
        );
        for i in 0..iterations {
            for j in 0..iterations {
                let ind = ((expected.len() * i) + j) * 4;
                assert_eq!(complex_to_bytes(expected[j]), out[ind..(ind + 4)])
            }
        }
    }
}
