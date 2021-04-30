use crate::prelude::*;

#[derive(Node)]
#[pass_by_ref]
pub struct JackOutputNode
{
    pub input: NodeReceiver<f32>,
    client: jack::Client,
}

impl JackOutputNode
{
    pub fn new() -> Self {
        // 1. Open client
        let (client, _status) = jack::Client::new("jack_node", jack::ClientOptions::NO_START_SERVER).unwrap();


        // 2. Register port
        let mut out_port = client.register_port("jack_node_out", jack::AudioOut::default()).unwrap();

        JackOutputNode {
            input: Default::default(),
            client,
        }
    }

    pub fn run(&mut self, _input: &f32) -> Result<(), NodeError> {
        // JACK drives the sample copying with the callback we register in the
        // JACK server, meaning comms-rs really doesn't do anything in it's run
        // handling

        // 3. Callback definition
        //let sample_rate = client.sample_rate();

        let process = jack::ClosureProcessHandler::new(

            // Callback function signature is basically non-negotiable I think...
            move |_: & jack::Client, ps: &jack::ProcessScope| -> jack::Control {

                // Get the output buffer
                let out = out_port.as_mut_slice(ps);

                // Get the crossbeam channel
                let samples = input.as_ref().unwrap().try_iter().take(out.len());

                // Write output
                for (i, o) in samples.zip(out.iter_mut()) {
                    *o = i;
                }

                // Continue as normal
                jack::Control::Continue
            },
        );

        // 4. Activate client
        let _active_client = client.activate_async((), process).unwrap();

        // 5. Processing...

        // 6. Optional client deactivate
        //active_client.deactivate().uwrap();
        //
        Ok(())
    }
}
