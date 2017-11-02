extern crate rand;
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
struct Boid
{
    position: (f32, f32),
    direction: (f32, f32),
}

impl Default for Boid
{
    fn default() -> Boid
    {
        Boid
        {
            position: (0f32, 0f32),
            direction: (0f32, 0f32),
        }
    }
}

const BOIDS_VELOCITY: f32 = 2.0;
const BOIDS_ALIGNMENT_RANGE: f32 = 25.0;
const BOIDS_COHESION_RANGE: f32 = 100.0;
const BOIDS_SEPARATION_RANGE: f32 = 20.0;
const BOIDS_SEPARATION_RATE: f32 = 0.25;
const BOIDS_COHESION_RATE: f32 = 0.005;
const BOIDS_ALIGNMENT_RATE: f32 = 0.1;
const BOIDS_NUMBER: usize = 300;
const BOIDS_SIZE: f32 = 10.0;
const BOIDS_SEPARATION_AREA: f32 =
    std::f32::consts::PI * BOIDS_SEPARATION_RANGE * BOIDS_SEPARATION_RANGE;
const BOIDS_MAX_DENSITY: f32 = 5.0 / BOIDS_SEPARATION_AREA;
const BOIDS_CHAOS: f32 = 0.01;

#[derive(Copy, Clone)]
struct Vertex
{
    position: (f32, f32),
}

impl Default for Vertex
{
    fn default() -> Vertex
    {
        Vertex
        {
            position: (0f32, 0f32),
        }
    }
}

implement_vertex!(Vertex, position);

fn cast_tuple((a, b): (u32, u32)) -> (f32, f32)
{
    (a as f32, b as f32)
}

fn normalize((x, y): (f32, f32)) -> (f32, f32)
{
    let length = length((x, y));
    if length != 0.0
    {
        (x / length, y / length)
    }
    else
    {
        (0.0, 0.0)
    }
}

fn distance((x1, y1): (f32, f32), (x2, y2): (f32, f32)) -> f32
{
    ((x1 - x2).powi(2) + (y1 - y2).powi(2)).sqrt()
}

fn length(vector: (f32, f32)) -> f32
{
    distance(vector, (0f32, 0f32))
}

fn get_group(boids: &[Boid; BOIDS_NUMBER], center: (f32, f32), range: f32) -> Vec<&Boid>
{
    let mut result = Vec::new();
    for i in 0..BOIDS_NUMBER
    {
        if distance(boids[i].position, center) <= range
        {
            result.push(&boids[i]);
        }
    }
    result
}

fn separation(boids: &mut [Boid; BOIDS_NUMBER]) -> ()
{
    let mut new_boids = *boids;
    for i in 0..BOIDS_NUMBER
    {
        let group = get_group(boids, boids[i].position, BOIDS_SEPARATION_RANGE);
        let group_density = group.len() as f32 / BOIDS_SEPARATION_AREA;
        if group_density > BOIDS_MAX_DENSITY && group_density > 0.0
        {
            let mut average_position = (0f32, 0f32);
            for i in &group
            {
                average_position.0 += i.position.0;
                average_position.1 += i.position.1;
            }
            average_position.0 /= group.len() as f32;
            average_position.1 /= group.len() as f32;
            let mut delta = (average_position.0 - boids[i].position.0, average_position.1 - boids[i].position.1);
            delta = normalize(delta);
            new_boids[i].direction.0 -= delta.0 * BOIDS_SEPARATION_RATE;
            new_boids[i].direction.1 -= delta.1 * BOIDS_SEPARATION_RATE;
        }
    }
    std::mem::swap(boids, &mut new_boids);
}

fn cohesion(boids: &mut [Boid; BOIDS_NUMBER]) -> ()
{
    let mut new_boids = *boids;
    for i in 0..BOIDS_NUMBER
    {
        let group = get_group(boids, boids[i].position, BOIDS_COHESION_RANGE);
        let mut average_position = (0f32, 0f32);
        for i in &group
        {
            average_position.0 += i.position.0;
            average_position.1 += i.position.1;
        }
        average_position.0 /= group.len() as f32;
        average_position.1 /= group.len() as f32;
        let mut delta = (average_position.0 - boids[i].position.0, average_position.1 - boids[i].position.1);
        delta = normalize(delta);
        new_boids[i].direction.0 += delta.0 * BOIDS_COHESION_RATE;
        new_boids[i].direction.1 += delta.1 * BOIDS_COHESION_RATE;
    }
    std::mem::swap(boids, &mut new_boids);
}

fn alignment(boids: &mut [Boid; BOIDS_NUMBER]) -> ()
{
    let mut new_boids = *boids;
    for i in 0..BOIDS_NUMBER
    {
        let group = get_group(boids, boids[i].position, BOIDS_ALIGNMENT_RANGE);
        let mut average_direction = (0f32, 0f32);
        for i in &group
        {
            average_direction.0 += i.direction.0;
            average_direction.1 += i.direction.1;
        }
        average_direction.0 /= group.len() as f32;
        average_direction.1 /= group.len() as f32;
        average_direction = normalize(average_direction);
        let delta = (average_direction.0 - boids[i].direction.0, average_direction.1 - boids[i].direction.1);
        new_boids[i].direction.0 += delta.0 * BOIDS_ALIGNMENT_RATE;
        new_boids[i].direction.1 += delta.1 * BOIDS_ALIGNMENT_RATE;
    }
    std::mem::swap(boids, &mut new_boids);
}

fn simulate(boids: &mut [Boid; BOIDS_NUMBER], map_size: &(f32, f32)) -> ()
{
    let mut new_boids = *boids;
    for i in 0..BOIDS_NUMBER
    {
        new_boids[i].direction.0 += ((rand::random::<f32>() * 2.0) - 1.0) * BOIDS_CHAOS;
        new_boids[i].direction.1 += ((rand::random::<f32>() * 2.0) - 1.0) * BOIDS_CHAOS;
        normalize(new_boids[i].direction);
        new_boids[i].position.0 = boids[i].position.0 + (boids[i].direction.0 * BOIDS_VELOCITY);
        new_boids[i].position.1 = boids[i].position.1 + (boids[i].direction.1 * BOIDS_VELOCITY);
    }
    std::mem::swap(boids, &mut new_boids);
    for i in 0..BOIDS_NUMBER
    {
        if boids[i].position.0 < 0.0
        {
            boids[i].position.0 = map_size.0 - 1.0;
        }
        if boids[i].position.1 < 0.0
        {
            boids[i].position.1 = map_size.1 - 1.0;
        }
        if boids[i].position.0 >= map_size.0
        {
            boids[i].position.0 = 0.0;
        }
        if boids[i].position.1 >= map_size.1
        {
            boids[i].position.1 = 0.0;
        }
    }
}

fn rotate_point((px, py): (f32, f32), (cx, cy): (f32, f32), angle: f32) -> (f32, f32)
{
    let x = angle.cos() * (px - cx) - angle.sin() * (py - cy) + cx;
    let y = angle.sin() * (px - cx) + angle.cos() * (py - cy) + cy;
    (x, y)
}

fn update_vertices(boids: &[Boid; BOIDS_NUMBER], vertices: &mut [Vertex]) -> ()
{
    let mut iv = 0usize;
    for i in boids.iter()
    {
        let center = (i.position.0 - (3f32.sqrt() / 2.0), i.position.1);
        let angle = i.direction.1.atan2(i.direction.0);
        let vertex0 = i.position;
        let vertex1 = (i.position.0 - BOIDS_SIZE, i.position.1 - (BOIDS_SIZE / 2.0));
        let vertex2 = (i.position.0 - BOIDS_SIZE, i.position.1 + (BOIDS_SIZE / 2.0));
        let rotated0 = rotate_point(vertex0, center, angle);
        let rotated1 = rotate_point(vertex1, center, angle);
        let rotated2 = rotate_point(vertex2, center, angle);
        vertices[(iv * 3)].position = rotated0;
        vertices[(iv * 3) + 1].position = rotated1;
        vertices[(iv * 3) + 2].position = rotated2;
        iv += 1;
    }
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

    let mut boids: [Boid; BOIDS_NUMBER] = [Boid::default(); BOIDS_NUMBER];
    for i in 0..BOIDS_NUMBER
    {
        boids[i].position.0 = rand::random::<f32>() * screen_size.0;
        boids[i].position.1 = rand::random::<f32>() * screen_size.1;
        boids[i].direction.0 = (rand::random::<f32>() * 2.0) - 1.0;
        boids[i].direction.1 = (rand::random::<f32>() * 2.0) - 1.0;
    }

    let vertices: [Vertex; BOIDS_NUMBER * 3] = [Vertex::default(); BOIDS_NUMBER * 3];

    let mut vertex_buffer = glium::VertexBuffer::dynamic(&display, &vertices).unwrap();
    let indices = glium::index::NoIndices(glium::index::PrimitiveType::TrianglesList);

    let mut shader_file = fs::File::open("glsl/vertex.vert")
        .expect("Vertex shader file not found.");
    let mut vertex_shader = String::new();
    shader_file.read_to_string(&mut vertex_shader)
        .expect("Couldn't load vertex shader.");

    let mut shader_file = fs::File::open("glsl/fragment.frag")
        .expect("Fragment shader file not found.");
    let mut fragment_shader = String::new();
    shader_file.read_to_string(&mut fragment_shader)
        .expect("Couldn't load fragment shader.");

    let program = glium::Program::from_source(&display, &vertex_shader, &fragment_shader, None).unwrap();

    let matrix: [[f32; 3]; 3] =
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
        separation(&mut boids);
        alignment(&mut boids);
        cohesion(&mut boids);
        simulate(&mut boids, &screen_size);
        update_vertices(&boids, &mut vertex_buffer.map());

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
