extern crate clap;
extern crate v4l;

use clap::{App, Arg};
use glium::index::PrimitiveType;
use glium::{glutin, Surface};
use glium::{implement_vertex, program, uniform};
use std::sync::{mpsc, RwLock};
use std::thread;
use std::time::Instant;
use v4l::capture;
use v4l::prelude::*;
use v4l::FourCC;

fn main() {
    let matches = App::new("v4l capture")
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

    // Allocate 4 buffers by default
    let buffers = matches.value_of("buffers").unwrap_or("4").to_string();
    let buffers = buffers.parse::<u32>().unwrap();

    let mut format: capture::Format;
    let params: capture::Parameters;

    let dev = RwLock::new(CaptureDevice::with_path(path.clone()).expect("Failed to open device"));
    {
        let mut dev = dev.write().unwrap();
        format = dev.format().expect("Failed to get format");
        params = dev.params().expect("Failed to get parameters");

        // enforce RGB3
        format.fourcc = FourCC::new(b"RGB3");
        format = dev.set_format(&format).expect("Failed to set format");
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
        glium::IndexBuffer::new(&display, PrimitiveType::TriangleStrip, &[1 as u16, 2, 0, 3])
            .unwrap();

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
        let mut dev = dev.write().unwrap();

        // Setup a buffer stream
        let mut stream =
            MmapStream::with_buffers(&mut *dev, buffers).expect("Failed to create buffer stream");

        loop {
            let buf = stream.next().expect("Failed to capture buffer");
            let data = buf.data().to_vec();
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
        match event {
            glutin::event::Event::WindowEvent { event, .. } => match event {
                glutin::event::WindowEvent::CloseRequested => {
                    *control_flow = glutin::event_loop::ControlFlow::Exit;
                }
                _ => {}
            },
            _ => {}
        }

        print!(
            "\rms: {}\t (buffer) + {}\t (UI)",
            t1.duration_since(t0).as_millis(),
            t0.elapsed().as_millis()
        );
    });
}
