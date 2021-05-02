use crate::prelude::*;
use jack::{Client, ClosureProcessHandler, Control, ProcessScope};
use std::sync::Arc;
use crossbeam::crossbeam_channel::{unbounded};
use std::fs::File;

#[derive(Default)]
pub struct JackOutputNode<P> {
    pub input: Arc<NodeReceiver<f32>>,
    pub sample_rate: usize,
    pub cb_rx: Option<Receiver<String>>,
    pub active_client: Option<jack::AsyncClient<(), P>>,
}

impl<P> JackOutputNode<P> {
    pub fn new() -> Self {
        JackOutputNode {
            input: Default::default(),
            sample_rate: 0,
            cb_rx: None,
            active_client: None,
        }
    }
}

impl<P: Send> Node for JackOutputNode<P>
{
    fn start(&mut self) {
        // 1. Open client
        let (client, _status) =
            Client::new("jack_node", jack::ClientOptions::NO_START_SERVER)
                .unwrap();

        // 2. Register port
        let mut out_port = client
            .register_port("jack_node_out", jack::AudioOut::default())
            .unwrap();

        // 3. Callback definition
        self.sample_rate = client.sample_rate();
        let xbeam_channel: Arc<_> = self.input.clone();
        let (cb_send, cb_rx) = unbounded();
        self.cb_rx = Some(cb_rx);

        // Callback function signature is basically non-negotiable I think...
        let process_cb = move |_cl: &Client, ps: &ProcessScope| -> Control {
            // Get the output buffer
            let out = out_port.as_mut_slice(ps);
            let mut out_iter = out.iter_mut();

            // TODO: Do the samples at the input get consumed properly as
            // we iterate over them here?
            // Get the crossbeam channel
            let mut file = File::create("/home/styty/git/comms-rs/i_am_here").unwrap();
            let msg = format!("xbeam_channel len: {:?}", (*xbeam_channel).as_ref().unwrap().len());
            cb_send.send(String::from("herp")).unwrap();
            for sample in (*xbeam_channel).as_ref().unwrap() {
                // Write output
                if let Some(o) = out_iter.next() {
                    *o = sample;
                } else {
                    // Get here if # of samples ready at input > out_len...
                    // Continue as normal
                    return Control::Continue;
                }
            }

            // Get here if out_len > # of samples ready at input...
            // Continue as normal
            Control::Continue
        };

        // 4. Activate client
        let process = jack::ClosureProcessHandler::new(process_cb);
        self.active_client = Some(client.activate_async((), process).unwrap());

        // 5. Processing...

        // 6. Optional client deactivate
        //active_client.deactivate().uwrap();

        println!("Sample Rate: {:?} Hz", self.sample_rate);
    }

    fn call(&mut self) -> Result<(), NodeError> {
        // JACK drives the sample copying with the callback we register in the
        // JACK server, meaning comms-rs really doesn't do anything in it's run
        // handling
        if let Some(cb_rx) = &self.cb_rx {
            if let Ok(msg) = cb_rx.try_recv() {
                println!("msg from cb: {:?}", msg);
            }
        }
        Ok(())
    }

    fn is_connected(&self) -> bool {
        self.input.is_some()
    }
}

#[derive(Default)]
pub struct JackInputNode {
    pub output: Arc<NodeSender<f32>>,
    pub sample_rate: usize,
}

impl JackInputNode {
    pub fn new() -> Self {
        JackInputNode {
            output: Default::default(),
            sample_rate: 0,
        }
    }
}

impl Node for JackInputNode {
    fn start(&mut self) {
        // 1. Open client
        let (client, _status) = Client::new(
            "jack_input_node",
            jack::ClientOptions::NO_START_SERVER,
        )
        .unwrap();

        // 2. Register port
        let in_port = client
            .register_port("jack_node_in", jack::AudioIn::default())
            .unwrap();

        // 3. Callback definition
        self.sample_rate = client.sample_rate();
        println!("Sample Rate: {:?} Hz", self.sample_rate);
        let xbeam_channel: Arc<_> = self.output.clone();

        let process = ClosureProcessHandler::new(
            // Callback function signature is basically non-negotiable I think...
            move |_cl: &Client, ps: &ProcessScope| -> Control {
                // Get the output buffer
                let input = in_port.as_slice(ps);

                // TODO: Do the samples at the input get consumed properly as
                // we iterate over them here?
                // Get the crossbeam channel
                for sample in input {
                    for (channel, _feedback) in &*xbeam_channel {
                        // Write input
                        let _result = channel.send(*sample);
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
        (*self.output).get(0).is_some()
    }
}
