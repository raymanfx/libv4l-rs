use minifb::{Window, WindowOptions};
use v4l::buffer::Type;
use v4l::format::fourcc::FourCC;
use v4l::io::traits::CaptureStream;
use v4l::prelude::*;
use v4l::video::traits::Capture;

fn main() {
    // Choose first caputre device
    let mut dev = Device::new(0).expect("Failed to open device");
    // Read device current format
    let mut fmt = dev.format().expect("Failed to read format");
    let width = fmt.width as usize;
    let height = fmt.height as usize;
    // Set device format to YUUV
    fmt.fourcc = FourCC::new(b"YUYV");
    dev.set_format(&fmt).expect("Failed to write format");

    // create a window with specifed width and height and default options
    let mut window =
        Window::new("Camera", width, height, WindowOptions::default()).unwrap_or_else(|e| {
            panic!("{}", e);
        });

    // Limit to max ~30 fps update rate
    window.limit_update_rate(Some(std::time::Duration::from_micros(8300)));

    // Create the capture stream
    let mut stream = MmapStream::with_buffers(&mut dev, Type::VideoCapture, 4)
        .expect("Failed to create buffer stream");

    loop {
        // Read next video frame
        let (frame, _m) = stream.next().unwrap();
        // The video is encoded as Vec<u32> but we have a Vec<u8>
        // We read it 4 bytes at a time, in order to be processed correctly
        #[allow(non_snake_case)]
        let buffer: Vec<u32> = frame
            .chunks(4)
            .map(|v| {
                // convert form YUYV to RGB
                let [Y, U, _, V]: [u8; 4] = std::convert::TryFrom::try_from(v).unwrap();
                let Y = Y as f32;
                let U = U as f32;
                let V = V as f32;

                let B = 1.164 * (Y - 16.) + 2.018 * (U - 128.);

                let G = 1.164 * (Y - 16.) - 0.813 * (V - 128.) - 0.391 * (U - 128.);

                let R = 1.164 * (Y - 16.) + 1.596 * (V - 128.);
                let v = [0, R as u8, G as u8, B as u8];

                u32::from_be_bytes(v)
            })
            .collect();
        assert_eq!(buffer.len(), width / 2 * height);

        // We unwrap here as we want this code to exit if it fails. Real applications may want to handle this in a different way
        window
            .update_with_buffer(&buffer, width / 2, height)
            .unwrap();
    }
}
