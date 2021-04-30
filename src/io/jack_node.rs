use std::sync::Arc;
use crate::prelude::*;
use jack::{AudioOut, Client, Control, ProcessScope, AsyncClient, ClosureProcessHandler};

pub struct JackOutputNode {
    pub input: Arc<NodeReceiver<Vec<f32>>>,
    pub sample_rate: usize,
}

impl JackOutputNode {
    pub fn new() -> Self {

        JackOutputNode {
            input: Default::default(),
            sample_rate: 0,
        }
    }
}

impl Node for JackOutputNode {

    fn start(&mut self) {
        // 1. Open client
        let (client, _status) = Client::new("jack_node", jack::ClientOptions::NO_START_SERVER).unwrap();

        // 2. Register port
        let mut out_port = client.register_port("jack_node_out", jack::AudioOut::default()).unwrap();

        // 3. Callback definition
        self.sample_rate = client.sample_rate();
        println!("Sample Rate: {:?} Hz", self.sample_rate);
        let xbeam_channel: Arc<_> = self.input.clone();

        let process = ClosureProcessHandler::new(

            // Callback function signature is basically non-negotiable I think...
            move |_cl: &Client, ps: &ProcessScope| -> Control {

                // Get the output buffer
                let out = out_port.as_mut_slice(ps);
                let mut out_iter = out.iter_mut();

                // TODO: Do the samples at the input get consumed properly as
                // we iterate over them here?
                // Get the crossbeam channel
                let mut cntr = 0;
                for sample_vec in (*xbeam_channel).as_ref().unwrap() {
                    for sample in sample_vec {

                        // Write output
                        if let Some(o) = out_iter.next() {
                            *o = sample;
                        } else {
                            // Get here if # of samples ready at input > out_len...
                            // Continue as normal
                            return Control::Continue
                        }
                    }
                }

                // Get here if out_len > # of samples ready at input...
                // Continue as normal
                Control::Continue
            },
        );

        // 4. Activate client
        let _active_client = client.activate_async((), process).unwrap();

        // 5. Processing...

        // 6. Optional client deactivate
        //active_client.deactivate().uwrap();
        //
    }

    fn call(&mut self) -> Result<(), NodeError> {
        // JACK drives the sample copying with the callback we register in the
        // JACK server, meaning comms-rs really doesn't do anything in it's run
        // handling
        Ok(())
    }

    fn is_connected(&self) -> bool {
        self.input.is_some()
    }
}
