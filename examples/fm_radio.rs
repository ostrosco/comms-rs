#[macro_use]
extern crate comms_rs;
extern crate num;

use comms_rs::filter::fir_node::{batch_fir, BatchFirNode};
use comms_rs::hardware::{self, radio};
use comms_rs::io::audio;
use comms_rs::modulation::fm;
use comms_rs::prelude::*;
use num::Complex;
use num::Zero;
use std::thread;

fn main() {
    // Get the radio frequency (assumed to be MHz as an input) and convert to
    // Hz. If no input is specified, default to 88.7 MHz.
    let radio_mhz = std::env::args()
        .nth(1)
        .and_then(|s| str::parse::<f32>(&s).ok());
    let radio_freq = match radio_mhz {
        Some(f) => (f * 1e6) as u32,
        None => {
            println!("No frequency specified, defaulting to 88.7.");
            88.7e6 as u32
        }
    };

    // The taps for the first low pass filter before FM demodulation.
    #[cfg_attr(rustfmt, rustfmt_skip)]
    let taps = [
        -0.01801270027742274, -0.004656920885448867, -0.002648852132912597,
        0.0008677368918448623, 0.005009212152225975, 0.008526175375849215,
        0.010172968340398776, 0.00912437509989248, 0.005334905990231011,
        -0.0003335859703032652, -0.0063014158102353, -0.01064075999239304,
        -0.011581891677991056, -0.008341954525032592, -0.0012824780121151447,
        0.007845515892673058, 0.016328062816332187, 0.021185546181771774,
        0.02007654361670823, 0.01217403940591024, -0.0013140567851934943,
        -0.017152074443356792, -0.030621606809715814, -0.03659663988110718,
        -0.030901697984472332, -0.01147126195667417, 0.02079513703320541,
        0.06194329755943689, 0.10559594630001239, 0.14421303245485026,
        0.17074726962322123, 0.18019648556329151, 0.17074726962322123,
        0.14421303245485026, 0.10559594630001239, 0.06194329755943689,
        0.02079513703320541, -0.01147126195667417, -0.030901697984472332,
        -0.03659663988110718, -0.030621606809715814, -0.017152074443356792,
        -0.0013140567851934943, 0.01217403940591024, 0.02007654361670823,
        0.021185546181771774, 0.016328062816332187, 0.007845515892673058,
        -0.0012824780121151447, -0.008341954525032592, -0.011581891677991056,
        -0.01064075999239304, -0.0063014158102353, -0.0003335859703032652,
        0.005334905990231011, 0.00912437509989248, 0.010172968340398776,
        0.008526175375849215, 0.005009212152225975, 0.0008677368918448623,
        -0.002648852132912597, -0.004656920885448867, -0.01801270027742274,
    ];
    let taps_samples: Vec<Complex<f32>> =
        taps.iter().map(|&x| Complex::new(x, 0.0)).collect();

    // The taps for the second low pass filter after FM demodulation.
    #[cfg_attr(rustfmt, rustfmt_skip)]
    let taps_audio = [
       -7.80759301e-19, -4.70562469e-04, -7.18601045e-04, -5.64787938e-04,
        1.00048451e-18,  7.40711911e-04,  1.22010776e-03,  1.00943714e-03,
       -1.63815191e-18, -1.38305804e-03, -2.28165286e-03, -1.87515712e-03,
        2.63134217e-18,  2.50157079e-03,  4.06019205e-03,  3.28157003e-03,
       -3.88283490e-18, -4.23970064e-03, -6.78167143e-03, -5.40871357e-03,
        5.27012528e-18,  6.83437728e-03,  1.08374410e-02,  8.58370816e-03,
       -6.65741567e-18, -1.07597935e-02, -1.70504083e-02, -1.35302879e-02,
        7.90890840e-18,  1.71869634e-02,  2.75821124e-02,  2.22845949e-02,
       -8.90209866e-18, -3.00178183e-02, -5.04645076e-02, -4.35022473e-02,
        9.53976606e-18,  7.41796666e-02,  1.58481935e-01,  2.25084217e-01,
        2.50360725e-01,  2.25084217e-01,  1.58481935e-01,  7.41796666e-02,
        9.53976606e-18, -4.35022473e-02, -5.04645076e-02, -3.00178183e-02,
       -8.90209866e-18,  2.22845949e-02,  2.75821124e-02,  1.71869634e-02,
        7.90890840e-18, -1.35302879e-02, -1.70504083e-02, -1.07597935e-02,
       -6.65741567e-18,  8.58370816e-03,  1.08374410e-02,  6.83437728e-03,
        5.27012528e-18, -5.40871357e-03, -6.78167143e-03, -4.23970064e-03,
       -3.88283490e-18,  3.28157003e-03,  4.06019205e-03,  2.50157079e-03,
        2.63134217e-18, -1.87515712e-03, -2.28165286e-03, -1.38305804e-03,
       -1.63815191e-18,  1.00943714e-03,  1.22010776e-03,  7.40711911e-04,
        1.00048451e-18, -5.64787938e-04, -7.18601045e-04, -4.70562469e-04,
       -7.80759301e-19];
    let taps_audio: Vec<Complex<f32>> =
        taps_audio.iter().map(|&x| Complex::new(x, 0.0)).collect();

    let mut rtlsdr = hardware::rtlsdr_radio::rtlsdr(0).unwrap();
    rtlsdr.init_radio(radio_freq, 1140000, 496).unwrap();
    rtlsdr.set_agc(true).unwrap();

    // Since we don't have anything fancy yet for type conversion, we're
    // gonna make a node to do it for us.
    create_node!(
        ConvertNode: Vec<Complex<f32>>,
        [],
        [recv: Vec<u8>],
        |_, samples: Vec<u8>| samples
            .chunks(2)
            .map(|x| Complex::new(
                (x[0] as f32 - 127.5) / 127.5,
                (x[1] as f32 - 127.5) / 127.5
            )).collect()
    );

    // A simple node to decimate the input by dec_rate. Once decimation makes
    // it to the standard libary, this node should go away.
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

    // Since the FIR filter currently only supports Complex samples, we need
    // to transform our real data after demodulation into Complex samples
    // so we can filter again.
    create_node!(
        Convert2Node: Vec<Complex<f32>>,
        [],
        [recv: Vec<f32>],
        |_, samples: Vec<f32>| -> Vec<Complex<f32>> {
            samples.iter().map(|&x| Complex::new(x, 0.0)).collect()
        },
    );

    // After the filter, we need to convert the data bact to real samples
    // to pass through decimation and to the audio sink.
    create_node!(
        Convert3Node: Vec<f32>,
        [],
        [recv: Vec<Complex<f32>>],
        |_, samples: Vec<Complex<f32>>| -> Vec<f32> {
            samples.iter().map(|&x| x.re).collect()
        },
    );

    let mut sdr = radio::RadioRxNode::new(rtlsdr, 0, 262144);
    let mut convert = ConvertNode::new();
    let mut dec1: DecimateNode<Complex<f32>> = DecimateNode::new(5);
    let mut filt1: BatchFirNode<f32> = batch_fir(taps_samples);
    let mut fm = fm::FMDemodNode::new(Complex::zero());
    let mut convert2 = Convert2Node::new();
    let mut filt2: BatchFirNode<f32> = batch_fir(taps_audio);
    let mut convert3 = Convert3Node::new();
    let mut dec2: DecimateNode<f32> = DecimateNode::new(5);
    let mut audio: audio::AudioNode<f32> = audio::audio(1, 44100, 0.1);

    connect_nodes!(sdr, convert, recv);
    connect_nodes!(convert, filt1, input);
    connect_nodes!(filt1, dec1, recv);
    connect_nodes!(dec1, fm, recv);
    connect_nodes!(fm, convert2, recv);
    connect_nodes!(convert2, filt2, input);
    connect_nodes!(filt2, convert3, recv);
    connect_nodes!(convert3, dec2, recv);
    connect_nodes!(dec2, audio, recv);
    start_nodes!(
        sdr, convert, filt1, dec1, fm, convert2, filt2, dec2, convert3, audio
    );
    loop {}
}