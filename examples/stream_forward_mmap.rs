use std::io;
use std::time::Instant;

use v4l::buffer::Type;
use v4l::io::traits::{CaptureStream, OutputStream};
use v4l::memory::Mmap;
use v4l::prelude::*;
use v4l::video::{Capture, Output};

fn main() -> io::Result<()> {
    let source = "/dev/video0";
    println!("Using source device: {}\n", source);

    // Determine which device to use
    let sink = "/dev/video1";
    println!("Using sink device: {}\n", sink);

    // Capture 4 frames by default
    let count = 4;

    // Allocate 4 buffers by default
    let buffer_count = 4;

    let cap = Device::with_path(source)?;
    println!("Active cap capabilities:\n{}", cap.query_caps()?);
    println!("Active cap format:\n{}", Capture::format(&cap)?);
    println!("Active cap parameters:\n{}", Capture::params(&cap)?);

    let out = Device::with_path(sink)?;
    println!("Active out capabilities:\n{}", out.query_caps()?);
    println!("Active out format:\n{}", Output::format(&out)?);
    println!("Active out parameters:\n{}", Output::params(&out)?);

    // BEWARE OF DRAGONS
    // Buggy drivers (such as v4l2loopback) only set the v4l2 buffer size (length field) once
    // a format is set, even though a valid format appears to be available when doing VIDIOC_G_FMT!
    // In our case, we just (try to) enforce the source format on the sink device.
    let source_fmt = Capture::format(&cap)?;
    let sink_fmt = Output::set_format(&out, &source_fmt)?;
    if source_fmt.width != sink_fmt.width
        || source_fmt.height != sink_fmt.height
        || source_fmt.fourcc != sink_fmt.fourcc
    {
        return Err(io::Error::new(
            io::ErrorKind::Other,
            "failed to enforce source format on sink device",
        ));
    }
    println!("New out format:\n{}", Output::format(&out)?);

    // Setup a buffer stream and grab a frame, then print its data
    let mut cap_stream = Stream::<Mmap>::with_buffers(&cap, Type::VideoCapture, buffer_count)?;
    let mut out_stream = Stream::<Mmap>::with_buffers(&out, Type::VideoOutput, buffer_count)?;

    // warmup
    CaptureStream::next(&mut cap_stream)?;

    let start = Instant::now();
    let mut megabytes_ps: f64 = 0.0;
    for i in 0..count {
        let t0 = Instant::now();
        let buf_in = CaptureStream::next(&mut cap_stream)?;
        let buf_out = OutputStream::next(&mut out_stream)?;

        // Output devices generally cannot know the exact size of the output buffers for
        // compressed formats (e.g. MJPG). They will however allocate a size that is always
        // large enough to hold images of the format in question. We know how big a buffer we need
        // since we control the input buffer - so just enforce that size on the output buffer.
        let slice_out = &mut buf_out[0..buf_in.len()];

        slice_out.copy_from_slice(buf_in);
        buf_out.field = 0;
        buf_out.bytesused = buf_in.bytesused;
        let duration_us = t0.elapsed().as_micros();

        let cur = buf_in.len() as f64 / 1_048_576.0 * 1_000_000.0 / duration_us as f64;
        if i == 0 {
            megabytes_ps = cur;
        } else {
            // ignore the first measurement
            let prev = megabytes_ps * (i as f64 / (i + 1) as f64);
            let now = cur * (1.0 / (i + 1) as f64);
            megabytes_ps = prev + now;
        }

        println!("Buffer");
        println!("  sequence   [in] : {}", buf_in.sequence);
        println!("  sequence  [out] : {}", buf_out.sequence);
        println!("  timestamp  [in] : {}", buf_in.timestamp);
        println!("  timestamp [out] : {}", buf_out.timestamp);
        println!("  flags      [in] : {}", buf_in.flags);
        println!("  flags     [out] : {}", buf_out.flags);
        println!("  length     [in] : {}", buf_in.len());
        println!("  length    [out] : {}", buf_out.len());
    }

    println!();
    println!("FPS: {}", count as f64 / start.elapsed().as_secs_f64());
    println!("MB/s: {}", megabytes_ps);

    Ok(())
}
