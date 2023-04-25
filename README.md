# Safe video4linux (v4l) bindings

[![crates.io](https://img.shields.io/crates/v/v4l.svg?style=for-the-badge)](https://crates.io/crates/v4l)
[![license](https://img.shields.io/github/license/raymanfx/libv4l-rs?style=for-the-badge)](https://github.com/raymanfx/libv4l-rs/blob/master/LICENSE.txt)
[![Build Status](https://img.shields.io/travis/raymanfx/libv4l-rs/master.svg?style=for-the-badge&logo=travis)](https://travis-ci.org/raymanfx/libv4l-rs)

This crate provides safe bindings to the Video for Linux (V4L) stack. Modern device drivers will usually implement the `v4l2` API while older ones may depend on the legacy `v4l` API. Such legacy devices may be used with this crate by choosing the `libv4l` feature for this crate.

## Goals
This crate shall provide the v4l-sys package to enable full (but unsafe) access to libv4l\*.
On top of that, there will be a high level, more idiomatic API to use video capture devices in Linux.

There will be simple utility applications to list devices and capture frames.
A minimalistic OpenGL/Vulkan viewer to display frames is planned for the future.

## Changelog
See [CHANGELOG.md](https://github.com/raymanfx/libv4l-rs/blob/master/CHANGELOG.md)

## Dependencies
You have the choice between two dependencies (both provided by this crate internally):
 * libv4l-sys
   > Link against the libv4l* stack including libv4l1, libv4l2, libv4lconvert.
   > This has the advantage of emulating common capture formats such as RGB3 in userspace through libv4lconvert and more.
   > However, some features like userptr buffers are not supported in libv4l.
 * v4l2-sys
   > Use only the Linux kernel provided v4l2 API provided by videodev2.h.
   > You get support for all v4l2 features such as userptr buffers, but may need to do format conversion yourself if you require e.g. RGB/BGR buffers which may not be supported by commodity devices such as webcams.

Enable either the `libv4l` or the `v4l2` backend by choosing the it as feature for this crate.

## Usage
Below you can find a quick example usage of this crate. It introduces the basics necessary to do frame capturing from a streaming device (e.g. webcam).

```rust
use v4l::buffer::Type;
use v4l::io::mmap::Stream;
use v4l::io::traits::CaptureStream;
use v4l::video::Capture;
use v4l::Device;
use v4l::FourCC;

fn main() {
    // Create a new capture device with a few extra parameters
    let mut dev = Device::new(0).expect("Failed to open device");

    // Let's say we want to explicitly request another format
    let mut fmt = dev.format().expect("Failed to read format");
    fmt.width = 1280;
    fmt.height = 720;
    fmt.fourcc = FourCC::new(b"YUYV");
    let fmt = dev.set_format(&fmt).expect("Failed to write format");

    // The actual format chosen by the device driver may differ from what we
    // requested! Print it out to get an idea of what is actually used now.
    println!("Format in use:\n{}", fmt);

    // Now we'd like to capture some frames!
    // First, we need to create a stream to read buffers from. We choose a
    // mapped buffer stream, which uses mmap to directly access the device
    // frame buffer. No buffers are copied nor allocated, so this is actually
    // a zero-copy operation.

    // To achieve the best possible performance, you may want to use a
    // UserBufferStream instance, but this is not supported on all devices,
    // so we stick to the mapped case for this example.
    // Please refer to the rustdoc docs for a more detailed explanation about
    // buffer transfers.

    // Create the stream, which will internally 'allocate' (as in map) the
    // number of requested buffers for us.
    let mut stream = Stream::with_buffers(&mut dev, Type::VideoCapture, 4)
        .expect("Failed to create buffer stream");

    // At this point, the stream is ready and all buffers are setup.
    // We can now read frames (represented as buffers) by iterating through
    // the stream. Once an error condition occurs, the iterator will return
    // None.
    loop {
        let (buf, meta) = stream.next().unwrap();
        println!(
            "Buffer size: {}, seq: {}, timestamp: {}",
            buf.len(),
            meta.sequence,
            meta.timestamp
        );

        // To process the captured data, you can pass it somewhere else.
        // If you want to modify the data or extend its lifetime, you have to
        // copy it. This is a best-effort tradeoff solution that allows for
        // zero-copy readers while enforcing a full clone of the data for
        // writers.
    }
}
```

Have a look at the provided `examples` for more sample applications.
