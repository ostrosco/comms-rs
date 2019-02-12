use crate::io::rodio::buffer;
use crate::io::rodio::queue::{queue, SourcesQueueInput};
use crate::io::rodio::{self, Sample, Sink};
use crate::prelude::*;
use std::sync::Arc;

/// A node that can play received samples out on audio. Currently this only
/// uses the default output device on the system.
#[derive(Node)]
pub struct AudioNode<T>
where
    T: Sample + Send + 'static,
{
    pub input: NodeReceiver<Vec<T>>,
    _sink: Sink,
    in_queue: Arc<SourcesQueueInput<T>>,
    channels: u16,
    sample_rate: u32,
}

impl<T> AudioNode<T>
where
    T: Sample + Send + 'static,
{
    /// Tosses the received samples into the sink for output.
    pub fn run(&mut self, samples: &[T]) -> Result<(), NodeError> {
        let samplebuffer = buffer::SamplesBuffer::new(
            self.channels,
            self.sample_rate,
            samples,
        );
        self.in_queue.append(samplebuffer);
        Ok(())
    }
}

/// Creates an AudioNode with the given parameters.
pub fn audio<T>(channels: u16, sample_rate: u32, volume: f32) -> AudioNode<T>
where
    T: Sample + Send + 'static,
{
    let device = rodio::default_output_device().unwrap();
    let mut sink = Sink::new(&device);
    let (in_queue, out_queue) = queue::<T>(true);
    sink.set_volume(volume);
    sink.append(out_queue);
    AudioNode::new(sink, in_queue, channels, sample_rate)
}
