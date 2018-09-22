//! This file is a little example program for interacting directly with the hackrf-sys package
//! without a wrapper
extern crate clap;
use clap::{Arg, App}

fn main() {
    //First, setup our command line arguments so we can bail out if they're not specified

    let matches = App::new("hackrf-sys receive")
        .version("0.1")
        .author("rfdoell")
        .about("Simple receiver program using a HackRF")
        .arg(Arg::with_name("frequency")
             .short("f")
             .long("frequency")
             .value_name("FREQ")
             .help("Set the center frequency of the receiver")
             .required(true)
             .takes_value(true))
        .arg(Arg::with_name("samp_rate")
             .short("s")
             .long("sample_rate")
             .value_name("SAMP_RATE")
             .help("Specify the sample rate to receive")
             .takes_value(true))
        .arg(Arg::with_name("lna_gain")
             .short("l")
             .long("lna_gain")
             .value_name("LNA_GAIN")
             .help("Set the gain value for the LNA, from 0-40 dB in 8dB steps")
             .takes_value(true))
        .arg(Arg::with_name("vga_gain")
             .short("v")
             .long("vga_gain")
             .value_name("VGA_GAIN")
             .help("Set the VGA gain value, from 0-62 dB in 2dB steps")
             .takes_value(true))
        .arg(Arg::with_name("output_file")
             .short("o")
             .long("output_file")
             .value_name("OUTPUT_FILE")
             .help("The name of the output file to save samples to")
             .takes_value(true))
        .get_matches();
    //TODO figure out how to do range validation
    let center_freq = matches.value_of("frequency").unwrap();
    let samp_rate = matches.value_of("samp_rate").unwrap_or(20e6);

