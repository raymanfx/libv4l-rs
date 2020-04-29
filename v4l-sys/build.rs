extern crate bindgen;

use std::env;
use std::path::PathBuf;

fn main() {
    println!("cargo:rustc-link-lib=v4l1");
    println!("cargo:rustc-link-lib=v4l2");
    println!("cargo:rustc-link-lib=v4lconvert");

    let bindings = bindgen::Builder::default()
        .header("wrapper.h")
        .generate()
        .expect("Failed to generate bindings");

    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    bindings
        .write_to_file(out_path.join("libv4l_bindings.rs"))
        .expect("Failed to write bindings");
}
