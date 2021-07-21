use std::io;

use v4l::prelude::*;

fn main() -> io::Result<()> {
    let path = "/dev/video0";
    println!("Using device: {}\n", path);

    let dev = Device::with_path(path)?;
    let controls = dev.query_controls()?;

    for control in controls {
        println!("{}", control);
    }

    Ok(())
}
