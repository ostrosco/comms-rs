use crate::prelude::*;
use jack::{
    AsyncClient, AudioOut, Client, ClosureProcessHandler, Control, Port,
    ProcessHandler, ProcessScope,
};
use std::sync::Arc;

pub struct MyHandler {
    out_port: Port<AudioOut>,
    rx: crossbeam::Receiver<f32>,
    cb_tx: crossbeam::Sender<String>,
    time: f32,
    frame_t: f32,
}

impl MyHandler {
    fn new(
        out_port: Port<AudioOut>,
        rx: crossbeam::Receiver<f32>,
        cb_tx: crossbeam::Sender<String>,
        frame_t: f32,
    ) -> Self {
        MyHandler {
            out_port,
            rx,
            cb_tx,
            frame_t,
            time: 0.0,
        }
    }
}

impl ProcessHandler for MyHandler {
    fn process(&mut self, _: &Client, ps: &ProcessScope) -> Control {
        if self.time >= 1.0 {
            let msg = "ASDF".to_string();
            self.cb_tx.try_send(msg).unwrap();
            self.time = 0.0;
        }

        // Get the output buffer
        let out = self.out_port.as_mut_slice(ps);

        // Write output
        for v in out.iter_mut() {
            if let Ok(y) = self.rx.recv() {
                *v = y;
                self.time += self.frame_t;
            }
        }

        // Get here if out_len > # of samples ready at input...
        // Continue as normal
        Control::Continue
    }
}

#[pass_by_ref]
#[derive(Node)]
pub struct JackOutputNode {
    pub input: NodeReceiver<f32>,
    pub sample_rate: usize,
    pub active_client: AsyncClient<(), MyHandler>,
    pub cb_rx: crossbeam::Receiver<String>,
    pub jack_tx: crossbeam::Sender<f32>,
}

impl JackOutputNode {
    pub fn new() -> Self {
        // 1. Open client
        let (client, _status) =
            Client::new("comms_rs", jack::ClientOptions::NO_START_SERVER)
                .unwrap();

        // 2. Register port
        let out_port = client
            .register_port("comms_rs_out", jack::AudioOut::default())
            .unwrap();

        // 3. Callback definition
        let sample_rate = client.sample_rate();
        let frame_t = 1.0 / sample_rate as f32;
        let (jack_tx, jack_rx) = crossbeam::bounded::<f32>(4096);

        // Callback function signature is basically non-negotiable I think...

        // 4. Activate client
        let (cb_tx, cb_rx) = crossbeam::unbounded();
        let active_client = client
            .activate_async(
                (),
                MyHandler::new(out_port, jack_rx, cb_tx, frame_t),
            )
            .unwrap();

        JackOutputNode {
            input: Default::default(),
            sample_rate,
            active_client,
            cb_rx,
            jack_tx,
        }
    }

    fn run(&mut self, input: &f32) -> Result<(), NodeError> {
        // JACK drives the sample copying with the callback we register in the
        // JACK server, meaning comms-rs really doesn't do anything in it's run
        // handling
        self.jack_tx.send(*input).unwrap();
        Ok(())
    }
}

impl Default for JackOutputNode {
    fn default() -> Self {
        Self::new()
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
        let xbeam_channel: Arc<_> = self.output.clone();

        let process = ClosureProcessHandler::new(
            // Callback function signature is basically non-negotiable I think...
            move |_cl: &Client, ps: &ProcessScope| -> Control {
                // Get the output buffer
                let input = in_port.as_slice(ps);

                // Get the crossbeam channel
                for sample in input {
                    for (channel, _feedback) in &*xbeam_channel {
                        // Write input
                        channel.send(*sample).unwrap();
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
