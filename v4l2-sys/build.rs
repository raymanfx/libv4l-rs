extern crate bindgen;

use std::env;
use std::path::PathBuf;

fn main() {
    let pkg_conf = pkg_config::Config::new()
        .probe("libv4l2")
        .expect("pkg-config has failed to find `libv4l2`");

    let bindings = bindgen::Builder::default()
        .header("wrapper.h")
        .clang_args(
            pkg_conf
                .include_paths
                .into_iter()
                .map(|path| format!("-I{}", path.to_string_lossy())),
        )
        .generate()
        .expect("Failed to generate bindings");

    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    bindings
        .write_to_file(out_path.join("v4l2_bindings.rs"))
        .expect("Failed to write bindings");
}
