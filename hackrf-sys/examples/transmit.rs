//! This file is a little example program for interacting directly with the hackrf-sys package
//! without a wrapper
#[macro_use]
extern crate log;
extern crate clap;
extern crate hackrf_sys;
extern crate simple_logger;

use clap::{App, Arg};
use log::Level;
use std::boxed::Box;
use std::process::exit;

use std::fs::File;
use std::io::Write;
use std::os::raw::c_void;
use std::ptr;
use std::thread;
use std::time;
use std::vec;

use hackrf_sys::{
    hackrf_device, hackrf_device_list_t, hackrf_error, hackrf_transfer,
};

fn cleanup(cleanup_stack: &mut vec::Vec<Box<FnMut() -> ()>>) {
    let mut next_item = cleanup_stack.pop();
    while let Some(mut item) = next_item {
        item();
        next_item = cleanup_stack.pop();
    }
    debug!("Finished executing all items on the cleanup stack");
}

//This method expects a message which can display the code at the end
fn catch_hackrf_code_and_quit(
    code: i32,
    message: &str,
    cleanup_stack: &mut vec::Vec<Box<FnMut() -> ()>>,
) {
    match code {
        hackrf_sys::hackrf_error_HACKRF_SUCCESS => (),
        hackrf_sys::hackrf_error_HACKRF_TRUE => (),
        _ => {
            error!("Got failure when calling {}: {}", message, code);
            cleanup(cleanup_stack);
            exit(10);
        }
    }
}

struct StateHolder<'a> {
    file_idx: usize,
    file_ref: &'a mut File,
}

extern "C" fn reader(xfer: *mut hackrf_transfer) -> i32 {
    trace!("reader method called");
    unsafe {
        let num_bytes = (*xfer).valid_length as usize;
        let state_ptr = (*xfer).rx_ctx as *mut StateHolder;

        let mut data_buffer: vec::Vec<u8> = vec::Vec::with_capacity(num_bytes);
        for idx in 0..num_bytes {
            data_buffer.push(*((*xfer).buffer.offset(idx as isize)));
        }
        trace!("Pushed {} bytes into the vector", data_buffer.len());
        match (*state_ptr).file_ref.write_all(&data_buffer) {
            Ok(_) => (),
            Err(_exc) => {
                error!("Could not write to file!");
                return 1;
            }
        };
    }
    0
}

fn main() {
    //First, initalize logging
    simple_logger::init_with_level(Level::Warn).unwrap();

    //Next, setup our command line arguments so we can bail out if they're not specified

    let min_samp_rate = 2e6;
    let max_samp_rate = 20e6;

    let min_freq = 1e6;
    let max_freq = 6e9;

    let min_vga_gain = 0;
    let max_vga_gain = 47;
    let vga_gain_step = 1;

    let matches = App::new("hackrf-sys receive")
        .version("0.1")
        .author("rfdoell")
        .about("Simple transmitter program using a HackRF")
        .arg(
            Arg::with_name("frequency")
                .short("f")
                .long("frequency")
                .value_name("FREQ")
                .help("Set the center frequency of the transmitter")
                .required(true)
                .takes_value(true),
        ).arg(
            Arg::with_name("samp_rate")
                .short("s")
                .long("sample_rate")
                .value_name("SAMP_RATE")
                .help("Specify the sample rate to transmit at")
                .takes_value(true),
        ).arg(
            Arg::with_name("vga_gain")
                .short("v")
                .long("vga_gain")
                .value_name("VGA_GAIN")
                .help("Set the Tx VGA gain value, from 0-47 dB in 1dB steps")
                .takes_value(true),
        ).arg(
            Arg::with_name("input_file")
                .short("i")
                .long("input_file")
                .value_name("INPUT_FILE")
                .help("The name of the input file to transmit samples from")
                .takes_value(true),
        ).get_matches();

    let center_freq: f32 = matches
        .value_of("frequency")
        .unwrap_or("1e9")
        .parse()
        .unwrap();
    let samp_rate: f64 = matches
        .value_of("samp_rate")
        .unwrap_or("20e6")
        .parse()
        .unwrap();
    let vga_gain: u32 =
        matches.value_of("vga_gain").unwrap_or("0").parse().unwrap();
    let input_fname =
        matches.value_of("input_file").unwrap_or("tmp_samps.dat");

    if samp_rate > max_samp_rate {
        error!(
            "Sample rate greater than maximum supported value: {} > {}",
            samp_rate, max_samp_rate
        );
        exit(1);
    }
    if samp_rate < min_samp_rate {
        error!(
            "Sample rate less than minimum supported value: {} < {}",
            samp_rate, min_samp_rate
        );
        exit(1);
    }

    if center_freq > max_freq {
        error!(
            "Center frequency greater than maximum supported value: {} > {}",
            center_freq, max_freq
        );
        exit(2);
    }
    if center_freq < min_freq {
        error!(
            "Center frequency less than minimum supported value: {} < {}",
            center_freq, min_freq
        );
        exit(2);
    }

    if vga_gain > max_vga_gain {
        error!(
            "VGA gain is greater than maximum supported value: {} > {}",
            vga_gain, max_vga_gain
        );
        exit(3);
    }
    if vga_gain < min_vga_gain {
        error!(
            "VGA gain is less than minimum supported value: {} < {}",
            vga_gain, min_vga_gain
        );
        exit(3);
    }
    if vga_gain % vga_gain_step != 0 {
        error!(
            "Given VGA gain is not a multiple of the gain step: {}",
            vga_gain_step
        );
        exit(3);
    }

    //Now that we're out of the woods, parameter-wise, create a stack to hold cleanup functions:
    let mut cleanup_stack: vec::Vec<Box<FnMut() -> ()>> = vec::Vec::new();
    debug!("About to initialize the hackrf subsystem");
    unsafe {
        let code: hackrf_error = hackrf_sys::hackrf_init();
        match code {
            hackrf_sys::hackrf_error_HACKRF_SUCCESS => (),
            hackrf_sys::hackrf_error_HACKRF_TRUE => (),
            _ => {
                error!(
                    "Got value of {} when initializing the hackrf subsystem",
                    code
                );
                exit(4);
            }
        }
        // If went well, add the de-init to the stack, to be called later:
        cleanup_stack.push(Box::new(|| {
            trace!("About to call hackrf_exit()");
            let code = hackrf_sys::hackrf_exit();
            match code {
                hackrf_sys::hackrf_error_HACKRF_SUCCESS => (),
                hackrf_sys::hackrf_error_HACKRF_TRUE => (),
                _ => {
                    error!(
                        "Got value of {} when deinitializing hackrf subsystem",
                        code
                    );
                }
            }
            trace!("Called hackrf_exit()");
        }));
        debug!("Initialized hackrf subsystem");
        // The reason for the extra {} and ; is to return "unit"

        //Next, list out any HackRFs that are present:
        let device_list: *mut hackrf_device_list_t =
            hackrf_sys::hackrf_device_list();

        //Check and see if there are any items in the list:
        let num_devices = (*device_list).devicecount;
        debug!(
            "Found {} device(s) when querying hackrf library",
            num_devices
        );
        if num_devices <= 0 {
            println!("To use this program, please connect a HackRF device and have the correct permissions.");
            debug!("Cleaning up and exiting...");
            hackrf_sys::hackrf_device_list_free(device_list);
            cleanup(&mut cleanup_stack);
            exit(5);
        }

        let mut hackrf_dev: *mut hackrf_device = ptr::null_mut();
        debug!("About to try and open the first device in the list");
        let code = hackrf_sys::hackrf_device_list_open(
            device_list,
            0,
            &mut hackrf_dev,
        );
        hackrf_sys::hackrf_device_list_free(device_list);

        match code {
            hackrf_sys::hackrf_error_HACKRF_SUCCESS => (),
            _ => {
                error!(
                    "Got value of {} when deinitializing hackrf subsystem",
                    code
                );
                cleanup(&mut cleanup_stack);
                exit(6);
            }
        }

        cleanup_stack.push(Box::new(move || {
            trace!("About to call hackrf_close()");
            let code = hackrf_sys::hackrf_close(hackrf_dev);
            match code {
                hackrf_sys::hackrf_error_HACKRF_SUCCESS => (),
                hackrf_sys::hackrf_error_HACKRF_TRUE => (),
                _ => {
                    error!("Got value of {} when closing hackrf device", code);
                }
            }
            trace!("Called hackrf_close()");
        }));

        //Next, we should be able to tune the radio using our center_freq
        debug!("About to set the center frequency");

        let code = hackrf_sys::hackrf_set_freq(hackrf_dev, center_freq as u64);
        catch_hackrf_code_and_quit(code, "set_freq", &mut cleanup_stack);

        debug!("About to set the sample rate");
        let code = hackrf_sys::hackrf_set_sample_rate(hackrf_dev, samp_rate);
        catch_hackrf_code_and_quit(code, "set_sample_rate", &mut cleanup_stack);

        debug!("About to set the VGA gain");
        let code = hackrf_sys::hackrf_set_txvga_gain(hackrf_dev, vga_gain);
        catch_hackrf_code_and_quit(code, "set_txvga_gain", &mut cleanup_stack);

        debug!("About to open a file");
        let in_file_result = File::open(input_fname);

        let mut in_file = match in_file_result {
            Ok(file) => file,
            Err(_exc) => {
                error!("Couldn't open file for writing: {}", input_fname);
                cleanup(&mut cleanup_stack);
                exit(11);
            }
        };
        //Assuming the file gets cleaned up when we leave scope. May cause errors
        let mut state_ball = StateHolder { file_idx:0, file_ref:&mut in_file };
        let state_ptr: *mut StateHolder = &mut state_ball;

        debug!("About to set up the transmitter");
        let code = hackrf_sys::hackrf_start_rx(
            hackrf_dev,
            Some(reader),
            state_ptr as *mut c_void,
        );
        catch_hackrf_code_and_quit(code, "start_tx", &mut cleanup_stack);

        debug!("Going to sleep for ten seconds");
        thread::sleep(time::Duration::from_secs(1));

        debug!("About to stop transmitting");
        let code = hackrf_sys::hackrf_stop_rx(hackrf_dev);
        catch_hackrf_code_and_quit(code, "stop_rx", &mut cleanup_stack);

        // Now that we're done, go through each item in the stack and call it
        cleanup(&mut cleanup_stack);
    }
}
