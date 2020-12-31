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
            Arg::with_name("list-formats")
                .long("list-formats")
                .help("Whether to list available formats"),
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

    let dev = Device::with_path(path).unwrap();

    let format = dev.format().unwrap();
    println!("Active format:\n{}", format);

    let params = dev.params().unwrap();
    println!("Active parameters:\n{}", params);

    if matches.is_present("list-formats") {
        println!("Available formats:");
        for format in dev.enum_formats().unwrap() {
            println!("  {} ({})", format.fourcc, format.description);

            for framesize in dev.enum_framesizes(format.fourcc).unwrap() {
                for discrete in framesize.size.to_discrete() {
                    println!("    Size: {}", discrete);

                    for frameinterval in dev
                        .enum_frameintervals(framesize.fourcc, discrete.width, discrete.height)
                        .unwrap()
                    {
                        println!("      Interval:  {}", frameinterval);
                    }
                }
            }

            println!("")
        }
    }
}
