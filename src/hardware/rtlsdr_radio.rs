use crate::hardware::radio::RadioRx;
use crate::hardware::rtlsdr::{self, RTLSDRDevice, RTLSDRError};

pub struct RTLSDR {
    rtlsdr: RTLSDRDevice,
}

// The default implementation for the RTLSDR in the library doesn't implement
// Send so it isn't thread-safe. We wrap it up here as Send so we can throw
// it in the nodes.
unsafe impl Send for RTLSDR {}

impl RTLSDR {
    /// Configure the radio for operation. Make sure to run this before
    /// running get_samples, otherwise get_samples will fail and output
    /// an empty vector.
    pub fn init_radio(
        &mut self,
        freq: u32,
        sample_rate: u32,
        gain: i32,
    ) -> Result<(), RTLSDRError> {
        self.rtlsdr.set_center_freq(freq)?;
        self.rtlsdr.set_sample_rate(sample_rate)?;
        self.rtlsdr.set_tuner_gain(gain)?;
        self.rtlsdr.reset_buffer()?;
        Ok(())
    }

    /// Enables or disables the AGC on the RTLSDR.
    pub fn set_agc(&mut self, agc_on: bool) -> Result<(), RTLSDRError> {
        self.rtlsdr.set_agc_mode(agc_on)?;
        Ok(())
    }

    pub fn teardown(&mut self) -> Result<(), RTLSDRError> {
        self.rtlsdr.close()
    }
}

impl RadioRx<u8> for RTLSDR {
    /// Returns samples from the RTLSDR. If the connection fails for
    /// whatever reason, an empty vector is sent out.
    fn recv_samples(&mut self, num_samples: usize, _: usize) -> Vec<u8> {
        match self.rtlsdr.read_sync(num_samples) {
            Ok(samp) => samp,
            Err(_) => {
                println!("Couldn't get samples");
                return vec![];
            }
        }
    }
}

/// Constructs a node containing an open but uninitialized connection to an
/// RTLSDR at the given index. Use RTLSDRNode::init_radio to set up the radio
/// prior to running the node.
pub fn rtlsdr(index: i32) -> Result<RTLSDR, RTLSDRError> {
    let rtlsdr = rtlsdr::open(index)?;
    Ok(RTLSDR { rtlsdr })
}

#[cfg(test)]
mod test {
    use crate::hardware::{radio, rtlsdr_radio};
    use std::time::Instant;

    use crate::prelude::*;
    use std::thread;

    #[test]
    #[cfg_attr(not(feature = "rtlsdr_node"), ignore)]
    // A quick test to check if we can read samples off the RTLSDR.
    fn test_get_samples() {
        let num_samples = 262144;
        let mut sdr = rtlsdr_radio::rtlsdr(0).unwrap();
        sdr.init_radio(88.7e6 as u32, 2.4e6 as u32, 0).unwrap();
        sdr.set_agc(true).unwrap();
        let mut sdr_node = radio::RadioRxNode::new(sdr, 0, num_samples);

        #[derive(Node)]
        #[pass_by_ref]
        struct CheckNode {
            input: NodeReceiver<Vec<u8>>,
            num_samples: usize,
        }

        impl CheckNode {
            pub fn new(num_samples: usize) -> Self {
                CheckNode {
                    num_samples,
                    input: Default::default(),
                }
            }

            pub fn run(&mut self, samples: &[u8]) -> Result<(), NodeError> {
                assert_eq!(samples.len(), self.num_samples);
                Ok(())
            }
        }

        let mut check_node = CheckNode::new(num_samples);
        connect_nodes!(sdr_node, output, check_node, input);
        start_nodes!(sdr_node);
        let check = thread::spawn(move || {
            let now = Instant::now();
            loop {
                check_node.call().unwrap();
                if now.elapsed().as_secs() >= 1 {
                    break;
                }
            }
        });
        assert!(check.join().is_ok());
    }
}
