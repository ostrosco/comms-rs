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

pub fn audio<T>(channels: u16, sample_rate: u32) -> AudioNode<T>
where
    T: Sample + Send + 'static,
{
    let device = rodio::default_output_device().unwrap();
    let sink = Sink::new(&device);
    AudioNode::new(sink, channels, sample_rate)
}

#[cfg(test)]
mod test {
    use crossbeam::{channel, Sender};
    use node::Node;
    use output::audio::{self, AudioNode};
    use rodio::source;
    use std::thread;

    #[test]
    fn play_sine() {
        let mut audio: AudioNode<f32> = audio::audio(1, 48000);

        create_node!(
            SineNode: Vec<f32>,
            [source: source::SineWave],
            [],
            |node: &mut SineNode| {
                let samp: Vec<f32> = node.source.clone().take(48000).collect();
                samp
            }
        );

        let mut sine = SineNode::new(source::SineWave::new(440));

        connect_nodes!(sine, audio, recv);
        start_nodes!(sine, audio);
    }
}
