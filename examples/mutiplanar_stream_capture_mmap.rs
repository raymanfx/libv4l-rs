use std::io;

use v4l::buffer::Type;
use v4l::capability::Flags;
use v4l::format::FieldOrder;
use v4l::{prelude::*, Format, FourCC};
use v4l::io::mmap::MPlaneStream;
use v4l::video::MultiPlanarCapture;
use v4l::io::traits::CaptureStream; 

fn main() -> io::Result<()> {
    let path = "/dev/video22";
    println!("Using device: {}\n", path);

    let dev = Device::with_path(path)?;

    let caps = dev.query_caps()?;
    if !caps.capabilities.contains(Flags::VIDEO_CAPTURE_MPLANE) {
        println!("{path} is no Video capture mplane device");
        return Err(io::Error::last_os_error());
    }

    if !caps.capabilities.contains(Flags::STREAMING) {
        println!("{path} does not support streaming i/o");
        return Err(io::Error::last_os_error());
    }

    let format = dev.format()?;
    println!("Active format:\n{}", format);
    // let mut format = Format::new(640, 480, FourCC::new(b"NV12"));
    // format.field_order = FieldOrder::Interlaced;
    // dev.set_format(&format)?;

    // let mut mplane_stream = MPlaneStream::new(&dev, Type::VideoCaptureMplane, 1)?;

    // mplane_stream.next()?;

    
    Ok(())
}