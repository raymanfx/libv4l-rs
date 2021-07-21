use std::io;

use v4l::prelude::*;

fn main() -> io::Result<()> {
    let path = "/dev/video0";
    println!("Using device: {}\n", path);

    let dev = Device::with_path(path)?;

    let caps = dev.query_caps()?;
    println!("Device capabilities:\n{}", caps);

    let controls = dev.query_controls()?;
    println!("Device controls:");
    let mut max_name_len = 0;
    for ctrl in &controls {
        if ctrl.name.len() > max_name_len {
            max_name_len = ctrl.name.len();
        }
    }
    for ctrl in controls {
        println!(
            "{:indent$} : [{}, {}]",
            ctrl.name,
            ctrl.minimum,
            ctrl.maximum,
            indent = max_name_len
        );
    }

    Ok(())
}
