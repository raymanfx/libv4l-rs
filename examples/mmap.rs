extern crate clap;
extern crate v4l;

use clap::{App, Arg};
use std::time::Instant;
use v4l::{Buffer, CaptureDevice, MappedBufferStream};

fn main() {
    let matches = App::new("v4l mmap")
        .version("0.2")
        .author("Christopher N. Hesse <raymanfx@gmail.com>")
        .about("Video4Linux device example")
        .arg(
            Arg::with_name("device")
                .short("d")
                .long("device")
                .value_name("INDEX or PATH")
                .help("Device node path or index (default: 0)")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("count")
                .short("c")
                .long("count")
                .value_name("INT")
                .help("Number of frames to capture (default: 4)")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("buffers")
                .short("b")
                .long("buffers")
                .value_name("INT")
                .help("Number of buffers to allocate (default: 4)")
                .takes_value(true),
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

    // Capture 4 frames by default
    let count = matches.value_of("count").unwrap_or("4").to_string();
    let count = count.parse::<u32>().unwrap();

    // Allocate 4 buffers by default
    let buffers = matches.value_of("buffers").unwrap_or("4").to_string();
    let buffers = buffers.parse::<u32>().unwrap();

    let mut dev = CaptureDevice::with_path(path).expect("Failed to open device");
    let format = dev.get_format().expect("Failed to get format");
    let params = dev.get_params().expect("Failed to get parameters");
    println!("Active format:\n{}", format);
    println!("Active parameters:\n{}", params);

    // Setup a buffer stream and grab a frame, then print its data
    let mut stream = MappedBufferStream::with_buffers(&mut dev, buffers)
        .expect("Failed to create buffer stream");

    // warmup
    stream.next().expect("Failed to capture buffer");

    let start = Instant::now();
    let mut megabytes_ps: f64 = 0.0;
    for i in 0..count {
        let t0 = Instant::now();
        let buf = stream.next().expect("Failed to capture buffer");
        let duration_us = t0.elapsed().as_micros();

        let cur = buf.len() as f64 / 1_048_576.0 * 1_000_000.0 / duration_us as f64;
        if i == 0 {
            megabytes_ps = cur;
        } else {
            // ignore the first measurement
            let prev = megabytes_ps * (i as f64 / (i + 1) as f64);
            let now = cur * (1.0 / (i + 1) as f64);
            megabytes_ps = prev + now;
        }

        println!("Buffer");
        println!("  sequence  : {}", buf.seq());
        println!("  timestamp : {}", buf.timestamp());
        println!("  flags     : {}", buf.flags());
        println!("  length    : {}", buf.len());
    }

    println!();
    println!("FPS: {}", count as f64 / start.elapsed().as_secs_f64());
    println!("MB/s: {}", megabytes_ps);
}
