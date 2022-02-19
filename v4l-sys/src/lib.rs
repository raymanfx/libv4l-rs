#![allow(clippy::all)]
#![allow(deref_nullptr)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]
// https://github.com/rust-lang/rust-bindgen/issues/1651
#![allow(unaligned_references)]

include!(concat!(env!("OUT_DIR"), "/libv4l_bindings.rs"));
