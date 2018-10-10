extern crate bindgen;

use std::env;
use std::path::PathBuf;

// Code lifted from bindgen tutorial
fn main() {
    println!("cargo:rustc-link-lib=hackrf");
    let bindings = bindgen::Builder::default()
        .header("wrapper.h")
        .clang_arg("-I/usr/local/include/libhackrf/")
        .generate()
        .expect("Unable to generate bindings");

    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .expect("Couldn't write bindings!");
}