use crossbeam::{Receiver, Sender};
use node::Node;
use num::Complex;

use std::fs::File;
use std::io::{self, BufWriter, Write};
use std::mem;
use std::path::Path;

type IQSample = Complex<i16>;

create_node!(
    IQFileOutput<W>: () where W: Write,
    [writer: W],
    [sample: IQSample],
    |node: &mut IQFileOutput<W>, sample: IQSample| node.run(sample)
);

impl<W: Write> IQFileOutput<W> {
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

pub fn iq_file_out<P: AsRef<Path>>(
    path: P,
) -> io::Result<IQFileOutput<impl Write>> {
    Ok(IQFileOutput::new(BufWriter::new(File::create(path)?)))
}

create_node!(
    IQFileBatchOutput<W>: () where W: Write,
    [writer: W],
    [samples: Vec<IQSample>],
    |node: &mut Self, samples: Vec<IQSample>| node.run(samples)
);

impl<W: Write> IQFileBatchOutput<W> {
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

pub fn iq_batch_file_out<P: AsRef<Path>>(
    path: P,
) -> io::Result<IQFileBatchOutput<impl Write>> {
    Ok(IQFileBatchOutput::new(File::create(path)?))
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
    fn test_complex_to_bytes() {
        let c = Complex::new(0x1234, 0x5678);
        let bytes = complex_to_bytes(c);

        assert_eq!(bytes, [0x34, 0x12, 0x78, 0x56]);
    }
}
