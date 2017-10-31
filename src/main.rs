#[macro_use]
extern crate glium;

use std::fs;
use std::io::Read;
use glium::glutin;
use glium::glutin::Event;
use glium::glutin::WindowEvent;
use glium::backend;
use glium::Surface;

#[derive(Copy, Clone)]
struct Vertex
{
    position: [f32; 2],
}

implement_vertex!(Vertex, position);

fn cast_tuple((a, b): (u32, u32)) -> (f32, f32)
{
    (a as f32, b as f32)
}

fn main()
{
    let mut events_loop = glutin::EventsLoop::new();
    let window = glutin::WindowBuilder::new()
        .with_title("boids-rs")
        .with_decorations(true);
    let context = glutin::ContextBuilder::new();
    let display = backend::glutin::Display::new(window, context, &events_loop).unwrap();

    let window = &display.gl_window();
    let monitor_id = window.get_current_monitor();
    let screen_size = cast_tuple(monitor_id.get_dimensions());
    window.set_fullscreen(Some(monitor_id));

    let mut window_open = true;

    let vertex1 = Vertex{position: [0.0, 0.0]};
    let vertex2 = Vertex{position: [400.0, 0.0]};
    let vertex3 = Vertex{position: [0.0, 400.0]};
    let vertex4 = Vertex{position: [400.0, 400.0]};
    let shape = vec![vertex1, vertex2, vertex3, vertex4];

    let vertex_buffer = glium::VertexBuffer::new(&display, &shape).unwrap();
    let indices = glium::index::NoIndices(glium::index::PrimitiveType::TriangleStrip);

    let mut shader_file = fs::File::open("glsl/vertex.vert")
        .expect("Vertex shader file not found.");
    let mut vertex_shader = String::new();
    shader_file.read_to_string(&mut vertex_shader)
        .expect("Couldn't load vertex shader file.");

    let mut shader_file = fs::File::open("glsl/fragment.frag")
        .expect("Fragment shader file not found.");
    let mut fragment_shader = String::new();
    shader_file.read_to_string(&mut fragment_shader)
        .expect("Couldn't load fragment shader file.");

    let program = glium::Program::from_source(&display, &vertex_shader, &fragment_shader, None).unwrap();

    let mut translation: (f32, f32) = (0.0, 0.0);
    let mut matrix: [[f32; 3]; 3] = 
       [[1.0, 0.0, 0.0],
        [0.0, 1.0, 0.0],
        [0.0, 0.0, 1.0]];
    while window_open
    {
        let uniforms = uniform!
        {
            matrix: matrix,
            screen_size: screen_size,
        };
        matrix[2][0] = translation.0;
        matrix[2][1] = translation.1;
        translation.0 += 0.001;
        translation.1 += 0.001;

        let mut frame = display.draw();
        frame.clear_color(0.0, 0.0, 0.0, 1.0);
        frame.draw(&vertex_buffer, &indices, &program, &uniforms,
                   &Default::default()).unwrap();
        frame.finish().unwrap();

        events_loop.poll_events(|event: Event|
        {
            match event
            {
                Event::WindowEvent{event: WindowEvent::Closed, ..} => window_open = false,
                _ => (),
            }
        });
    }
}
