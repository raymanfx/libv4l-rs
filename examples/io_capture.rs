use std::io;
use std::time::Instant;

use v4l::buffer::Type;
use v4l::io::Queue;
use v4l::prelude::*;
use v4l::video::Capture;

fn main() -> io::Result<()> {
    let path = "/dev/video0";
    println!("Using device: {}\n", path);

    // Capture 4 frames by default
    let count = 4;

    let dev = Device::with_path(path)?;
    let format = dev.format()?;
    let params = dev.params()?;
    println!("Active format:\n{}", format);
    println!("Active parameters:\n{}", params);

    // set up a queue
    let mut queue = Queue::with_mmap(dev.handle(), Type::VideoCapture, 4)?;

    // enqueue all buffers once
    for i in 0..queue.len() {
        let buf = queue.query_buf(i as u32)?;
        queue.enqueue(&buf)?;
    }

    // start streaming
    let mut queue = queue.start_stream()?;

    // warm up
    let buf = queue.dequeue()?;
    let _ = queue.enqueue(&buf)?;

    let start = Instant::now();
    let mut megabytes_ps: f64 = 0.0;
    for i in 0..count {
        let t0 = Instant::now();

        // check whether there's something for us to do
        dev.handle().poll(libc::POLLIN, -1)?;

        // remove a buffer from the drivers' outgoing queue
        let buf = queue.dequeue()?;

        let duration_us = t0.elapsed().as_micros();

        let cur = buf.bytesused as f64 / 1_048_576.0 * 1_000_000.0 / duration_us as f64;
        if i == 0 {
            megabytes_ps = cur;
        } else {
            // ignore the first measurement
            let prev = megabytes_ps * (i as f64 / (i + 1) as f64);
            let now = cur * (1.0 / (i + 1) as f64);
            megabytes_ps = prev + now;
        }

        println!("Buffer");
        println!("  sequence  : {}", buf.sequence);
        println!("  timestamp : {}", buf.timestamp);
        println!("  flags     : {}", buf.flags);
        println!("  length    : {}", buf.bytesused);

        // once we're done with it, add the buffer to the drivers' incoming queue
        queue.enqueue(&buf)?;
    }

    println!();
    println!("FPS: {}", count as f64 / start.elapsed().as_secs_f64());
    println!("MB/s: {}", megabytes_ps);

    Ok(())
}
