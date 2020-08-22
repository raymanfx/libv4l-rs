extern crate clap;
extern crate v4l;

use clap::{App, Arg};
use v4l::prelude::*;

fn main() {
    let matches = App::new("v4l device")
        .version("0.2")
        .author("Christopher N. Hesse <raymanfx@gmail.com>")
        .about("Video4Linux device example")
        .arg(
            Arg::with_name("device")
                .short("d")
                .long("device")
                .value_name("INDEX or PATH")
                .help("Output device node path or index (default: 1)")
                .takes_value(true),
        )
        .get_matches();

    // Determine which device to use
    let mut path: String = matches
        .value_of("device")
        .unwrap_or("/dev/video1")
        .to_string();
    if path.parse::<u64>().is_ok() {
        path = format!("/dev/video{}", path);
    }
    println!("Using device: {}\n", path);

    let dev = OutputDevice::with_path(path).expect("Failed to open output device");
    let controls = dev.query_controls().expect("Failed to query controls");

    for control in controls {
        println!("{}", control);
    }
}
