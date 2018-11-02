#[macro_use]
extern crate comms_rs;
extern crate num;

use comms_rs::fir::fir_node::batch_fir;
use comms_rs::hardware::{self, radio};
use comms_rs::modulation::fm;
use comms_rs::io::audio;
use comms_rs::prelude::*;
use num::Complex;
use num::Zero;
use std::thread;

fn main() {
    let mut rtlsdr = hardware::rtlsdr_radio::rtlsdr(0).unwrap();
    rtlsdr.init_radio(88.7e6 as u32, 240e3 as u32, 0).unwrap();
    rtlsdr.set_agc(true).unwrap();
    let taps = vec![
        Complex::new(-0.008037050554982349, 0.0),
        Complex::new(-0.005537827292176266, 0.0),
        Complex::new(0.0035793930622277847, 0.0),
        Complex::new(0.0217809799077926, 0.0),
        Complex::new(0.04144442493103674, 0.0),
        Complex::new(0.049392833077048684, 0.0),
        Complex::new(0.03486325788981191, 0.0),
        Complex::new(-0.0009854161558589221, 0.0),
        Complex::new(-0.04082230270860769, 0.0),
        Complex::new(-0.05721207557466388, 0.0),
        Complex::new(-0.027337093993023753, 0.0),
        Complex::new(0.051477138420681204, 0.0),
        Complex::new(0.1556993466946653, 0.0),
        Complex::new(0.24436629232087692, 0.0),
        Complex::new(0.27911529640745075, 0.0),
        Complex::new(0.24436629232087692, 0.0),
        Complex::new(0.1556993466946653, 0.0),
        Complex::new(0.051477138420681204, 0.0),
        Complex::new(-0.027337093993023753, 0.0),
        Complex::new(-0.05721207557466388, 0.0),
        Complex::new(-0.04082230270860769, 0.0),
        Complex::new(-0.0009854161558589221, 0.0),
        Complex::new(0.03486325788981191, 0.0),
        Complex::new(0.049392833077048684, 0.0),
        Complex::new(0.04144442493103674, 0.0),
        Complex::new(0.0217809799077926, 0.0),
        Complex::new(0.0035793930622277847, 0.0),
        Complex::new(-0.005537827292176266, 0.0),
        Complex::new(-0.008037050554982349, 0.0),
    ];

    // Since we don't have anything fancy yet for type conversion, we're
    // gonna make a node to do it for us.
    create_node!(
        ConvertNode: Vec<Complex<f32>>,
        [],
        [recv: Vec<u8>],
        |_, samples: Vec<u8>| samples
            .chunks(2)
            .map(|x| Complex::new(x[0] as f32, x[1] as f32))
            .collect()
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
    let mut filt = batch_fir::<f32>(taps);
    let mut dec1: DecimateNode<Complex<f32>> = DecimateNode::new(2);
    let mut fm = fm::FMDemodNode::new(Complex::zero());
    let mut dec2: DecimateNode<f32> = DecimateNode::new(1);
    let mut audio: audio::AudioNode<f32> = audio::audio(2, 48000, 1.0);

    connect_nodes!(sdr, convert, recv);
    connect_nodes!(convert, filt, input);
    connect_nodes!(filt, dec1, recv);
    connect_nodes!(dec1, fm, recv);
    connect_nodes!(fm, dec2, recv);
    connect_nodes!(dec2, audio, recv);
    start_nodes!(sdr, convert, filt, dec1, fm, dec2, audio);
    loop {}
}
