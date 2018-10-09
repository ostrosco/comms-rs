#[macro_use]
extern crate comms_rs;
extern crate num;

use comms_rs::hardware::{self, radio};
use comms_rs::modulation::fm;
use comms_rs::output::audio;
use comms_rs::prelude::*;
use num::Complex;
use num::Zero;
use std::thread;

fn main() {
    let mut rtlsdr = hardware::rtlsdr_radio::rtlsdr(0).unwrap();
    rtlsdr.init_radio(98.9e6 as u32, 2.4e6 as u32, 0).unwrap();
    rtlsdr.set_agc(true).unwrap();

    // Since we don't have anything fancy yet for type conversion, we're
    // gonna make a node to do it for us.
    create_node!(
        ConvertNode: Vec<Complex<f32>>,
        [],
        [recv: Vec<u8>],
        |_, samples: Vec<u8>| {
            samples
            .chunks(2)
            .map(|x| Complex::new(x[0] as f32, x[1] as f32))
            .collect()
        }
    );

    create_node!(
        DecimateNode<T>: Vec<T>,
        [dec_rate: usize],
        [recv: Vec<T>],
        |node: &mut DecimateNode<T>, signal: Vec<T>| {
            let mut ix = 0;
            let new_size = (signal.len() / node.dec_rate + 1) as usize;
            let mut signal_dec = Vec::<T>::with_capacity(new_size);
            while ix < signal.len() {
                signal_dec.push(signal[ix]);
                ix += node.dec_rate;
            }
            signal_dec
        },
        T: Copy,
    );

    let mut sdr = radio::RadioRxNode::new(rtlsdr, 0, 262144);
    let mut convert = ConvertNode::new();
    let mut dec1: DecimateNode<Complex<f32>> = DecimateNode::new(6);
    let mut fm = fm::FMDemodNode::new(Complex::zero());
    let mut dec2: DecimateNode<f32> = DecimateNode::new(4);
    let mut audio: audio::AudioNode<f32> = audio::audio(2, 48000, 1.0);

    connect_nodes!(sdr, convert, recv);
    connect_nodes!(convert, dec1, recv);
    connect_nodes!(dec1, fm, recv);
    connect_nodes!(fm, dec2, recv);
    connect_nodes!(dec2, audio, recv);
    start_nodes!(sdr, convert, dec1, fm, dec2, audio);
    loop {}
}
