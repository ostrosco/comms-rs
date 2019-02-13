//! Provides nodes for retrieving data from sources as raw IQ data.
//!
//! Nodes will read data as bytes from the reader provided at initialization.
//! Complex<i16> will be read from the reader as first the real then
//! imaginary portions, with each item in host byte-order.

use byteorder::{NativeEndian, ReadBytesExt, WriteBytesExt};
use num::Complex;

use crate::prelude::*;

use std::fs::File;
use std::io::{self, BufReader, BufWriter, Read, Write};
use std::path::Path;
use std::{thread, time};

type IQSample = Complex<i16>;

/// Will retrieve samples as interleaved 16-bit values in host byte-order from
/// reader. Panics upon reaching end of file.
#[derive(Node)]
pub struct IQInput<R>
where
    R: Read,
{
    reader: R,
    pub sender: NodeSender<IQSample>,
}

impl<R: Read> IQInput<R> {
    pub fn run(&mut self) -> Result<IQSample, NodeError> {
        let re_res = self.reader.read_i16::<NativeEndian>();
        let im_res = self.reader.read_i16::<NativeEndian>();

        let (re, im) = match (re_res, im_res) {
            (Ok(re), Ok(im)) => (re, im),
            (Err(e), _) => {
                if let io::ErrorKind::UnexpectedEof = e.kind() {
                    // reached eof, sleep forever
                    // TODO determine what happens if we kill the thread
                    thread::sleep(time::Duration::from_secs(100_000));
                }
                panic!("Unable to read file with err: {}", e);
            }
            (_, Err(e)) => {
                if let io::ErrorKind::UnexpectedEof = e.kind() {
                    // reached eof, sleep forever
                    // TODO determine what happens if we kill the thread
                    thread::sleep(time::Duration::from_secs(100_000));
                }
                panic!("Unable to read file with err: {}", e);
            }
        };

        Ok(Complex::new(re, im))
    }
}

/// Make an IQInput node reading data to the given file.
///
/// # Example
///
/// ```
/// use comms_rs::io::raw_iq::iq_file_in;
///
/// let innode_res = iq_file_in("/tmp/raw_iq.bin");
/// ```
pub fn iq_file_in<P: AsRef<Path>>(path: P) -> io::Result<IQInput<impl Read>> {
    Ok(IQInput::new(BufReader::new(File::open(path)?)))
}

#[derive(Node)]
pub struct IQBatchInput<R>
where
    R: Read,
{
    reader: R,
    batch_size: usize,
    pub sender: NodeSender<Vec<IQSample>>,
}

/// Will retrieve samples as interleaved 16-bit values in host byte-order from
/// reader. Will only send vectors completely filled to size of buf_size.
/// Panics upon reaching end of file.
impl<R: Read> IQBatchInput<R> {
    pub fn run(&mut self) -> Result<Vec<IQSample>, NodeError> {
        let mut buf = Vec::with_capacity(self.batch_size);
        for _ in 0..self.batch_size {
            let re_res = self.reader.read_i16::<NativeEndian>();
            let im_res = self.reader.read_i16::<NativeEndian>();

            let (re, im) = match (re_res, im_res) {
                (Ok(re), Ok(im)) => (re, im),
                (Err(e), _) => {
                    if let io::ErrorKind::UnexpectedEof = e.kind() {
                        // reached eof, sleep forever
                        // TODO determine what happens if we kill the thread
                        thread::sleep(time::Duration::from_secs(1_000_000));
                    }
                    panic!("Unable to read file with err: {}", e);
                }
                (_, Err(e)) => {
                    if let io::ErrorKind::UnexpectedEof = e.kind() {
                        // reached eof, sleep forever
                        // TODO determine what happens if we kill the thread
                        thread::sleep(time::Duration::from_secs(1_000_000));
                    }
                    panic!("Unable to read file with err: {}", e);
                }
            };
            buf.push(Complex::new(re, im));
        }

        Ok(buf)
    }
}

/// Make an IQBatchInput node sending data to the given file.
///
/// # Example
///
/// ```
/// use comms_rs::io::raw_iq::iq_batch_file_in;
///
/// let innode_res = iq_batch_file_in("/tmp/raw_iq.bin", 1024);
/// ```
pub fn iq_batch_file_in<P: AsRef<Path>>(
    path: P,
    buffer_size: usize,
) -> io::Result<IQBatchInput<impl Read>> {
    Ok(IQBatchInput::new(File::open(path)?, buffer_size))
}

/// Will send samples as interleaved 16-bit values in host byte-order to writer.
#[derive(Node)]
pub struct IQOutput<W>
where
    W: Write,
{
    pub input: NodeReceiver<IQSample>,
    writer: W,
}

impl<W: Write> IQOutput<W> {
    pub fn run(&mut self, samp: IQSample) -> Result<(), NodeError> {
        self.writer
            .write_i16::<NativeEndian>(samp.re)
            .expect("failed to write sample to writer");
        self.writer
            .write_i16::<NativeEndian>(samp.im)
            .expect("failed to write sample to writer");
        Ok(())
    }
}

/// Make an IQOutput node sending data to the given file.
///
/// # Example
///
/// ```
/// use comms_rs::io::raw_iq::iq_file_out;
///
/// let outnode = iq_file_out("/tmp/raw_iq.bin").expect("couldn't create file");
/// ```
pub fn iq_file_out<P: AsRef<Path>>(
    path: P,
) -> io::Result<IQOutput<impl Write>> {
    Ok(IQOutput::new(BufWriter::new(File::create(path)?)))
}

#[derive(Node)]
#[pass_by_ref]
pub struct IQBatchOutput<W>
where
    W: Write,
{
    pub input: NodeReceiver<Vec<IQSample>>,
    writer: W,
}

impl<W: Write> IQBatchOutput<W> {
    pub fn run(&mut self, samples: &[IQSample]) -> Result<(), NodeError> {
        samples.iter().for_each(|samp| {
            self.writer
                .write_i16::<NativeEndian>(samp.re)
                .expect("failed to write sample to writer");
            self.writer
                .write_i16::<NativeEndian>(samp.im)
                .expect("failed to write sample to writer");
        });
        Ok(())
    }
}

/// Make an IQBatchOutput node sending data to the given file.
///
/// # Example
///
/// ```
/// use comms_rs::io::raw_iq::iq_batch_file_out;
///
/// let outnode = iq_batch_file_out("/tmp/raw_iq.bin").expect("couldn't create file");
/// ```
pub fn iq_batch_file_out<P: AsRef<Path>>(
    path: P,
) -> io::Result<IQBatchOutput<impl Write>> {
    Ok(IQBatchOutput::new(File::create(path)?))
}

#[cfg(test)]
mod test {
    use crate::io::raw_iq::*;
    use byteorder::{ByteOrder, NativeEndian};
    use std::io::Cursor;
    use std::mem;

    fn complex_into_bytes(buf: &mut [u8], c: Complex<i16>) {
        NativeEndian::write_i16(buf, c.re);
        NativeEndian::write_i16(&mut buf[2..], c.im);
    }

    #[test]
    /// Test that node correctly sends received data to writer.
    fn test_single_in_node() {
        let iterations = 100usize;

        let mut out: Vec<Complex<i16>> = Vec::new();
        let expected_out: Vec<Complex<i16>> = (0..iterations as i16)
            .map(|i| Complex::new(i * 2, i * 2 + 1))
            .collect();
        let mut input = vec![0u8; iterations * 2 * 2];
        for i in 0..iterations {
            complex_into_bytes(&mut input[(i * 4)..], expected_out[i]);
        }
        {
            let mut node = IQInput::new(Cursor::new(input));
            for _ in 0..iterations {
                out.push(node.run().unwrap());
            }
        }

        assert_eq!(out.len(), iterations);
        for i in 0..iterations {
            assert_eq!(expected_out[i], out[i]);
        }
    }

    #[test]
    /// Test that node correctly sends received data to writer.
    fn test_batch_in_node() {
        let iterations = 100usize;

        let mut out: Vec<Vec<Complex<i16>>> = Vec::new();
        let expected_out: Vec<Complex<i16>> = (0..iterations as i16)
            .map(|i| Complex::new(i * 2, i * 2 + 1))
            .collect();
        let mut input = vec![0u8; iterations * 2 * 2];
        for i in 0..iterations {
            complex_into_bytes(&mut input[(i * 4)..], expected_out[i]);
        }
        let input = {
            let mut tmp = Vec::with_capacity(
                mem::size_of::<u8>() * iterations * iterations,
            );
            for _i in 0..iterations {
                tmp.extend(&input);
            }

            tmp
        };
        {
            let mut node = IQBatchInput::new(Cursor::new(input), iterations);
            for _ in 0..iterations {
                out.push(node.run().unwrap());
            }
        }

        assert_eq!(out.len(), iterations);
        for out in out.iter() {
            for j in 0..iterations {
                assert_eq!(expected_out[j], out[j]);
            }
        }
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
                node.run(*item).unwrap();
            }
        }

        assert_eq!(out.len(), iterations * mem::size_of::<IQSample>());
        let mut buf = vec![0u8; 4];
        for i in 0..iterations {
            complex_into_bytes(&mut buf, expected[i]);
            assert_eq!(*buf, out[(i * 4)..(i * 4 + 4)])
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
                node.run(&expected).unwrap();
            }
        }

        assert_eq!(
            out.len(),
            iterations * iterations * mem::size_of::<IQSample>()
        );
        let mut buf = vec![0u8; 4];
        for i in 0..iterations {
            for j in 0..iterations {
                let ind = ((expected.len() * i) + j) * 4;
                complex_into_bytes(&mut buf, expected[j]);
                assert_eq!(*buf, out[ind..(ind + 4)])
            }
        }
    }

    // TODO add tests for thread blocking on input exhaustion
}
