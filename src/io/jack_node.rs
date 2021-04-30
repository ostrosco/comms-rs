use crate::prelude::*;
use jack::{AudioOut, Client, Control, ProcessScope, AsyncClient, ClosureProcessHandler};

pub struct JackOutputNode<F>
where
    F: 'static + Send + FnMut(&Client, &ProcessScope) -> Control,
{
    pub input: NodeReceiver<f32>,
    client: Option<AsyncClient<(), ClosureProcessHandler<F>>>,
}

impl<F> JackOutputNode<F>
where
    F: 'static + Send + FnMut(&Client, &ProcessScope) -> Control,
{
    pub fn new() -> Self {

        JackOutputNode {
            input: Default::default(),
            client: None,
        }
    }
}

impl<F> Node for JackOutputNode<F>
where
    F: 'static + Send + FnMut(&Client, &ProcessScope) -> Control,
{

    fn start(&mut self) {
        // 1. Open client
        let (client, _status) = Client::new("jack_node", jack::ClientOptions::NO_START_SERVER).unwrap();

        // 2. Register port
        let mut out_port = client.register_port("jack_node_out", jack::AudioOut::default()).unwrap();

        // 3. Callback definition
        let sample_rate = client.sample_rate();

        let xbeam_channel = self.input.as_ref().unwrap();

        let process = ClosureProcessHandler::new(

            // Callback function signature is basically non-negotiable I think...
            move |cl: &Client, ps: &ProcessScope| -> Control {

                // Get the output buffer
                let out_port = AudioOut::from(cl.port_by_name("jack_node_out").unwrap());
                let out = out_port.as_mut_slice(ps);

                // Get the crossbeam channel
                let samples = xbeam_channel.try_iter().take(out.len());

                // Write output
                for (i, o) in samples.zip(out.iter_mut()) {
                    *o = i;
                }

                // Continue as normal
                Control::Continue
            },
        );

        // 4. Activate client
        self.client = Some(client.activate_async((), process).unwrap());

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
