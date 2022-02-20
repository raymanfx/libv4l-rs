use std::io;

use v4l::prelude::*;
use v4l::video::Capture;

fn main() -> io::Result<()> {
    let path = "/dev/video0";
    println!("Using device: {}\n", path);

    let dev = Device::with_path(path)?;

    let format = dev.format()?;
    println!("Active format:\n{}", format);

    let params = dev.params()?;
    println!("Active parameters:\n{}", params);

    println!("Available formats:");
    for format in dev.enum_formats()? {
        println!("  {} ({})", format.fourcc, format.description);

        for framesize in dev.enum_framesizes(format.fourcc)? {
            for discrete in framesize.size.to_discrete() {
                println!("    Size: {}", discrete);

                for frameinterval in
                    dev.enum_frameintervals(framesize.fourcc, discrete.width, discrete.height)?
                {
                    println!("      Interval:  {}", frameinterval);
                }
            }
        }

        println!()
    }

    Ok(())
}
