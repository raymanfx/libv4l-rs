use std::io;
use std::sync::{mpsc, RwLock};
use std::thread;
use std::time::Instant;

use glium::index::PrimitiveType;
use glium::{glutin, Surface};
use glium::{implement_vertex, program, uniform};

use jpeg_decoder as jpeg;

use v4l::buffer::Type;
use v4l::io::traits::CaptureStream;
use v4l::prelude::*;
use v4l::video::capture::Parameters;
use v4l::video::Capture;
use v4l::{Format, FourCC};

fn main() -> io::Result<()> {
    let path = "/dev/video0";
    println!("Using device: {}\n", path);

    // Allocate 4 buffers by default
    let buffer_count = 4;

    let mut format: Format;
    let params: Parameters;

    let dev = RwLock::new(Device::with_path(path)?);
    {
        let dev = dev.write().unwrap();
        format = dev.format()?;
        params = dev.params()?;

        // try RGB3 first
        format.fourcc = FourCC::new(b"RGB3");
        format = dev.set_format(&format)?;

        if format.fourcc != FourCC::new(b"RGB3") {
            // fallback to Motion-JPEG
            format.fourcc = FourCC::new(b"MJPG");
            format = dev.set_format(&format)?;

            if format.fourcc != FourCC::new(b"MJPG") {
                return Err(io::Error::new(
                    io::ErrorKind::Other,
                    "neither RGB3 nor MJPG supported by the device, but required by this example!",
                ));
            }
        }
    }

    println!("Active format:\n{}", format);
    println!("Active parameters:\n{}", params);

    // Setup the GL display stuff
    let event_loop = glutin::event_loop::EventLoop::new();
    let wb = glutin::window::WindowBuilder::new();
    let cb = glutin::ContextBuilder::new().with_vsync(true);
    let display = glium::Display::new(wb, cb, &event_loop).unwrap();

    // building the vertex buffer, which contains all the vertices that we will draw
    let vertex_buffer = {
        #[derive(Copy, Clone)]
        struct Vertex {
            position: [f32; 2],
            tex_coords: [f32; 2],
        }

        implement_vertex!(Vertex, position, tex_coords);

        glium::VertexBuffer::new(
            &display,
            &[
                Vertex {
                    position: [-1.0, -1.0],
                    tex_coords: [0.0, 0.0],
                },
                Vertex {
                    position: [-1.0, 1.0],
                    tex_coords: [0.0, 1.0],
                },
                Vertex {
                    position: [1.0, 1.0],
                    tex_coords: [1.0, 1.0],
                },
                Vertex {
                    position: [1.0, -1.0],
                    tex_coords: [1.0, 0.0],
                },
            ],
        )
        .unwrap()
    };

    // building the index buffer
    let index_buffer =
        glium::IndexBuffer::new(&display, PrimitiveType::TriangleStrip, &[1u16, 2, 0, 3]).unwrap();

    // compiling shaders and linking them together
    let program = program!(&display,
        140 => {
            vertex: "
                #version 140
                uniform mat4 matrix;
                in vec2 position;
                in vec2 tex_coords;
                out vec2 v_tex_coords;
                void main() {
                    gl_Position = matrix * vec4(position, 0.0, 1.0);
                    v_tex_coords = tex_coords;
                }
            ",

            fragment: "
                #version 140
                uniform sampler2D tex;
                in vec2 v_tex_coords;
                out vec4 f_color;
                void main() {
                    f_color = texture(tex, v_tex_coords);
                }
            "
        },
    )
    .unwrap();

    let (tx, rx) = mpsc::channel();

    thread::spawn(move || {
        let dev = dev.write().unwrap();

        // Setup a buffer stream
        let mut stream = MmapStream::with_buffers(&dev, Type::VideoCapture, buffer_count).unwrap();

        loop {
            let (buf, _) = stream.next().unwrap();
            let data = match &format.fourcc.repr {
                b"RGB3" => buf.to_vec(),
                b"MJPG" => {
                    // Decode the JPEG frame to RGB
                    let mut decoder = jpeg::Decoder::new(buf);
                    decoder.decode().expect("failed to decode JPEG")
                }
                _ => panic!("invalid buffer pixelformat"),
            };
            tx.send(data).unwrap();
        }
    });

    event_loop.run(move |event, _, control_flow| {
        let t0 = Instant::now();
        let data = rx.recv().unwrap();
        let t1 = Instant::now();

        let image =
            glium::texture::RawImage2d::from_raw_rgb_reversed(&data, (format.width, format.height));
        let opengl_texture = glium::texture::Texture2d::new(&display, image).unwrap();

        // building the uniforms
        let uniforms = uniform! {
            matrix: [
                [1.0, 0.0, 0.0, 0.0],
                [0.0, 1.0, 0.0, 0.0],
                [0.0, 0.0, 1.0, 0.0],
                [0.0, 0.0, 0.0, 1.0f32]
            ],
            tex: &opengl_texture
        };

        // drawing a frame
        let mut target = display.draw();
        target.clear_color(0.0, 0.0, 0.0, 0.0);
        target
            .draw(
                &vertex_buffer,
                &index_buffer,
                &program,
                &uniforms,
                &Default::default(),
            )
            .unwrap();
        target.finish().unwrap();

        // polling and handling the events received by the window
        if let glutin::event::Event::WindowEvent {
            event: glutin::event::WindowEvent::CloseRequested,
            ..
        } = event
        {
            *control_flow = glutin::event_loop::ControlFlow::Exit;
        }

        print!(
            "\rms: {}\t (buffer) + {}\t (UI)",
            t1.duration_since(t0).as_millis(),
            t0.elapsed().as_millis()
        );
    });
}
