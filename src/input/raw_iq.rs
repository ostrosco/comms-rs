//! Provides nodes for retrieving data from sources as raw IQ data.
//!
//! Nodes will read data as bytes from the reader provided at initialization.
//! Complex<i16> will be read from the reader as first the real then
//! imaginary portions, with each item in host byte-order.

use byteorder::{NativeEndian, ReadBytesExt};
use num::Complex;

use prelude::*;

use std::fs::File;
use std::io::{self, Read, BufReader};
use std::path::Path;
use std::{thread, time};

type IQSample = Complex<i16>;

/// Will retrieve samples as interleaved 16-bit values in host byte-order from
/// reader. Panics upon reaching end of file.
create_node!(
    IQInput<R>: IQSample,
    [reader: R],
    [],
    |node: &mut IQInput<R>| node.run(),
    R: Read,
);

impl<R: Read> IQInput<R> {
    fn run(&mut self) -> IQSample {
        let re_res = self.reader
            .read_i16::<NativeEndian>();
        let im_res = self.reader
            .read_i16::<NativeEndian>();

        let (re, im) = match (re_res, im_res) {
            (Ok(re), Ok(im)) => (re, im),
            (Err(e), _) => {
                match e.kind() {
                    io::ErrorKind::UnexpectedEof => {
                        // reached eof, sleep forever
                        // TODO determine what happens if we kill the thread
                        thread::sleep(time::Duration::from_secs(100000));
                    },
                    _ => (),
                }
                panic!("Unable to read file with err: {}", e);
            },
            (_, Err(e)) => {
                match e.kind() {
                    io::ErrorKind::UnexpectedEof => {
                        thread::sleep(time::Duration::from_secs(100000));
                    },
                    _ => (),
                }
                panic!("Unable to read file with err: {}", e);
            },
        };

        Complex::new(re, im)
    }
}

/// Make an IQInput node reading data to the given file.
///
/// # Example
///
/// ```
/// use comms_rs::input::raw_iq::iq_file_in;
///
/// let innode_res = iq_file_in("/tmp/raw_iq.bin");
/// ```
pub fn iq_file_in<P: AsRef<Path>>(
    path: P,
) -> io::Result<IQInput<impl Read>> {
    Ok(IQInput::new(BufReader::new(File::open(path)?)))
}

create_node!(
    IQBatchInput<R>: Vec<IQSample>,
    [reader: R, batch_size: usize],
    [],
    |node: &mut Self| node.run(),
    R: Read,
);

/// Will retrieve samples as interleaved 16-bit values in host byte-order from
/// reader. Will only send vectors completely filled to size of buf_size.
/// Panics upon reaching end of file.
impl<R: Read> IQBatchInput<R> {
    fn run(&mut self) -> Vec<IQSample> {
        let mut buf = Vec::with_capacity(self.batch_size);
        for _ in 0..self.batch_size {
            let re_res = self.reader
                .read_i16::<NativeEndian>();
            let im_res = self.reader
                .read_i16::<NativeEndian>();

            let (re, im) = match (re_res, im_res) {
                (Ok(re), Ok(im)) => (re, im),
                (Err(e), _) => {
                    match e.kind() {
                        io::ErrorKind::UnexpectedEof => {
                            // reached eof, sleep forever
                            // TODO determine what happens if we kill the thread
                            thread::sleep(time::Duration::from_secs(1000000));
                        },
                        _ => (),
                    }
                    panic!("Unable to read file with err: {}", e);
                },
                (_, Err(e)) => {
                    match e.kind() {
                        io::ErrorKind::UnexpectedEof => {
                            thread::sleep(time::Duration::from_secs(1000000));
                        },
                        _ => (),
                    }
                    panic!("Unable to read file with err: {}", e);
                },
            };
            buf.push(Complex::new(re, im));
        }


        buf
    }
}

/// Make an IQBatchInput node sending data to the given file.
///
/// # Example
///
/// ```
/// use comms_rs::input::raw_iq::iq_batch_file_in;
///
/// let innode_res = iq_batch_file_in("/tmp/raw_iq.bin", 1024);
/// ```
pub fn iq_batch_file_in<P: AsRef<Path>>(
    path: P, buffer_size: usize
) -> io::Result<IQBatchInput<impl Read>> {
    Ok(IQBatchInput::new(File::open(path)?, buffer_size))
}

#[cfg(test)]
mod test {
    use std::io::Cursor;
    use std::mem;
    use byteorder::{ByteOrder, NativeEndian};
    use input::raw_iq::*;

    create_node!(
        CollectionNode<T>: (),
        [collection: Vec<T>],
        [recv: T],
        |node: &mut Self, val: T| {
            node.collection.push(val);
        },
        T: Sized,
    );

    fn complex_into_bytes(buf: &mut [u8], c: Complex<i16>) {
        NativeEndian::write_i16(buf, c.re);
        NativeEndian::write_i16(&mut buf[2..], c.im);
    }

    fn complex_from_bytes(buf: &[u8]) -> Complex<i16> {
        let re = NativeEndian::read_i16(buf);
        let im = NativeEndian::read_i16(&buf[2..]);

        Complex::new(re, im)
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
            complex_into_bytes(&mut input[(i*4)..], expected_out[i]);
        }
        {
            let mut node = IQInput::new(Cursor::new(input));
            for _ in 0..iterations {
                out.push(node.run());
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
            complex_into_bytes(&mut input[(i*4)..], expected_out[i]);
        }
        let input = {
            let mut tmp = Vec::with_capacity(mem::size_of::<u8>() * iterations * iterations);
            for i in 0..iterations {
                tmp.extend(&input);
            }

            tmp
        };
        {
            let mut node = IQBatchInput::new(Cursor::new(input), iterations);
            for _ in 0..iterations {
                out.push(node.run());
            }
        }

        assert_eq!(out.len(), iterations);
        for i in 0..iterations {
            for j in 0..iterations {
                assert_eq!(expected_out[j], out[i][j]);
            }
        }
    }

    // TODO add tests for thread blocking on input exhaustion
}
