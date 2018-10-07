use crossbeam::{Receiver, Sender};
use node::Node;
use rodio::buffer;
use rodio::{self, Sample, Sink};

create_node!(
    #[doc = "A node that can play received samples out on audio."]
    AudioNode<T>: (),
    [sink: Sink, channels: u16, sample_rate: u32],
    [recv: Vec<T>],
    |node: &mut AudioNode<T>, samples: Vec<T>| {
        node.play(samples);
    },
    T: Sample + Send + 'static,
);

impl<T> AudioNode<T>
where
    T: Sample + Send + 'static,
{
    pub fn play(&mut self, samples: Vec<T>) {
        let samplebuffer = buffer::SamplesBuffer::new(
            self.channels,
            self.sample_rate,
            samples,
        );
        self.sink.append(samplebuffer);
    }
}

pub fn audio<T>(channels: u16, sample_rate: u32, volume: f32) -> AudioNode<T>
where
    T: Sample + Send + 'static,
{
    let device = rodio::default_output_device().unwrap();
    let mut sink = Sink::new(&device);
    sink.set_volume(volume);
    AudioNode::new(sink, channels, sample_rate)
}
