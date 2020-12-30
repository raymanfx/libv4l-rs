extern crate clap;
extern crate v4l;

use clap::{App, Arg};
use v4l::prelude::*;
use v4l::video::Capture;

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
                .help("Capture device node path or index (default: 0)")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("verbose")
                .short("v")
                .long("verbose")
                .help("Whether to output verbose framesize and frameinterval information"),
        )
        .get_matches();

    // Determine which device to use
    let mut path: String = matches
        .value_of("device")
        .unwrap_or("/dev/video0")
        .to_string();
    if path.parse::<u64>().is_ok() {
        path = format!("/dev/video{}", path);
    }
    println!("Using device: {}\n", path);

    let dev = Device::with_path(path).expect("Failed to open capture device");
    let format = dev.format().expect("Failed to get format");
    let params = dev.params().expect("Failed to get parameters");
    println!("Active format:\n{}", format);

    if matches.is_present("verbose") {
        for available_format in dev.enum_formats().expect("Failed to enumerate formats") {
            println!("Available format:\n{}", available_format);
            for available_framesize in dev
                .enum_framesizes(available_format.fourcc)
                .expect("Failed to enumerate framesizes")
            {
                println!("Available framesize:\n{}", available_framesize);
                for available_discrete_framesize in available_framesize.size.discrete_framesizes() {
                    for available_frameinterval in dev
                        .enum_frameintervals(
                            available_framesize.fourcc,
                            available_discrete_framesize.width,
                            available_discrete_framesize.height,
                        )
                        .expect("Failed to enumerate frameintervals")
                    {
                        println!("Available frameinterval:\n{}", available_frameinterval);
                    }
                }
            }
        }
    }

    println!("Active parameters:\n{}", params);
}
