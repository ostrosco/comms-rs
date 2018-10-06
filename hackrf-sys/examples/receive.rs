//! This file is a little example program for interacting directly with the hackrf-sys package
//! without a wrapper
extern crate clap;
extern crate hackrf_

use std::collections::HashSet;
use std::process::exit;
use clap::{Arg, App}


fn main() {
    //First, setup our command line arguments so we can bail out if they're not specified

    let min_samp_rate = 2e6;
    let max_samp_rate = 20e6;

    let min_freq = 1e6;
    let max_freq = 6e9;

    let min_vga_gain = 0;
    let max_vga_gain = 62;
    let vga_gain_step = 2;
    
    let min_lna_gain = 0;
    let max_lna_gain = 40;
    let lna_gain_step = 8;

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

    let center_freq = matches.value_of("frequency").unwrap_or(1e9);
    let samp_rate = matches.value_of("samp_rate").unwrap_or(20e6);
    let vga_gain = matches.value_of("vga_gain").unwrap_or(0);
    let lna_gain = matches.value_of("vga_gain").unwrap_or(0);
    let output_fname = matches.value_of("output_file").unwrap_or("tmp_samps.dat");

    if samp_rate > max_samp_rate {
        println!("Sample rate greater than maximum supported value: {} > {}", samp_rate, max_samp_rate);
        exit(1);
    }
    if samp_rate < min_samp_rate {
        println!("Sample rate less than minimum supported value: {} < {}", samp_rate, min_samp_rate);
        exit(1);
    }

    if center_freq > max_freq {
        println!("Center frequency greater than maximum supported value: {} > {}", center_freq, max_freq);
        exit(2);
    }
    if center_freq > max_freq {
        println!("Center frequency less than minimum supported value: {} < {}", center_freq, min_freq);
        exit(2);
    }

    if vga_gain > max_vga_gain {
        println!("VGA gain is greater than maximum supported value: {} > {}", vga_gain, max_vga_gain);
        exit(3);
    }
    if vga_gain > min_vga_gain {
        println!("VGA gain is less than minimum supported value: {} < {}", vga_gain, min_vga_gain);
        exit(3);
    }
    if vga_gain % vga_gain_step != 0 {
        println!("Given VGA gain is not a multiple of the gain step: {}", vga_gain_step);
        exit(3);
    }

    if lna_gain > max_lna_gain {
        println!("VGA gain is greater than maximum supported value: {} > {}", lna_gain, max_lna_gain);
        exit(3);
    }
    if lna_gain > min_lna_gain {
        println!("VGA gain is less than minimum supported value: {} < {}", lna_gain, min_lna_gain);
        exit(3);
    }
    if lna_gain % lna_gain_step != 0 {
        println!("Given VGA gain is not a multiple of the gain step: {}", lna_gain_step);
        exit(3);
    }
}
