use crossbeam::Sender;
use hardware::rtlsdr::{self, RTLSDRDevice, RTLSDRError};
use node::Node;

pub struct RTLSDR {
    rtlsdr: RTLSDRDevice,
}

// The default implementation for the RTLSDR in the library doesn't implement
// Send so it isn't thread-safe. We wrap it up here as Send so we can throw
// it in the nodes.
unsafe impl Send for RTLSDR {}

create_node!(
    #[doc = "An interface for the RTL-SDR."]
    RTLSDRNode: Vec<u8>,
    [sdr: RTLSDR, num_samples: usize],
    [],
    |node: &mut RTLSDRNode| node.get_samples()
);

impl RTLSDRNode {
    /// Configure the radio for operation. Make sure to run this before
    /// running get_samples, otherwise get_samples will fail and output
    /// an empty vector.
    pub fn init_radio(
        &mut self,
        freq: u32,
        sample_rate: u32,
        gain: i32,
    ) -> Result<(), RTLSDRError> {
        self.sdr.rtlsdr.set_center_freq(freq)?;
        self.sdr.rtlsdr.set_sample_rate(sample_rate)?;
        self.sdr.rtlsdr.set_tuner_gain(gain)?;
        self.sdr.rtlsdr.reset_buffer()?;
        Ok(())
    }

    /// Enables or disables the AGC on the RTLSDR.
    pub fn set_agc(&mut self, agc_on: bool) -> Result<(), RTLSDRError> {
        self.sdr.rtlsdr.set_agc_mode(agc_on)?;
        Ok(())
    }

    /// Returns samples from the RTLSDR. If the connection fails for
    /// whatever reason, an empty vector is sent out.
    pub fn get_samples(&mut self) -> Vec<u8> {
        match self.sdr.rtlsdr.read_sync(self.num_samples) {
            Ok(samp) => samp,
            Err(_) => vec![],
        }
    }
}

/// Constructs a node containing an open but uninitialized connection to an
/// RTLSDR at the given index. Use RTLSDRNode::init_radio to set up the radio
/// prior to running the node.
pub fn rtlsdr(
    index: i32,
    num_samples: usize,
) -> Result<RTLSDRNode, RTLSDRError> {
    let rtlsdr = rtlsdr::open(index)?;
    let sdr = RTLSDR { rtlsdr };
    Ok(RTLSDRNode::new(sdr, num_samples))
}

#[cfg(test)]
mod test {
    use crossbeam::{channel, Receiver, Sender};
    use hardware::rtlsdr_node;
    use node::Node;
    use std::thread;
    use std::time::Instant;

    #[test]
    #[cfg_attr(not(feature = "rtlsdr_support"), ignore)]
    // A quick test to check if we can read samples off the RTLSDR.
    fn test_get_samples() {
        let num_samples = 262144;
        if let Ok(mut sdr_node) = rtlsdr_node::rtlsdr(0, num_samples) {
            sdr_node.init_radio(88.7e6 as u32, 2.4e6 as u32, 0).unwrap();
            sdr_node.set_agc(true).unwrap();
            create_node!(
                CheckNode: (),
                [num_samples: usize],
                [recv: Vec<u8>],
                |node: &mut CheckNode, samples: Vec<u8>| {
                    assert_eq!(samples.len(), node.num_samples);
                }
            );
            let mut check_node = CheckNode::new(num_samples);
            connect_nodes!(sdr_node, check_node, recv);
            start_nodes!(sdr_node);
            let check = thread::spawn(move || {
                let now = Instant::now();
                loop {
                    check_node.call();
                    if now.elapsed().as_secs() >= 1 {
                        break;
                    }
                }
            });
            assert!(check.join().is_ok());
        } else {
            panic!("Couldn't connect to SDR.");
        }
    }
}
