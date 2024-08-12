extern crate bindgen;

use std::env;
use std::path::{Path, PathBuf};

fn main() {
    let extra_include_paths = if cfg!(target_os = "freebsd") {
        assert!(
            Path::new("/usr/local/include/linux/videodev2.h").exists(),
            "Video4Linux `videodev2.h` UAPI header is required to generate bindings \
            against `libv4l2` and the header file is missing.\n\
            Consider installing `multimedia/v4l_compat` FreeBSD package."
        );
        vec!["-I/usr/local/include"]
    } else {
        vec![]
    };

    let bindings = bindgen::Builder::default()
        .header("wrapper.h")
        .clang_args(extra_include_paths)
        .generate()
        .expect("Failed to generate bindings");

    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    bindings
        .write_to_file(out_path.join("v4l2_bindings.rs"))
        .expect("Failed to write bindings");
}
