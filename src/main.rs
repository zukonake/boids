extern crate rand;
extern crate cgmath;
#[macro_use]
extern crate glium;

use std::fs;
use std::io::Read;
use cgmath::Point2;
use cgmath::Vector2;
use cgmath::MetricSpace;
use cgmath::EuclideanSpace;
use cgmath::InnerSpace;
use cgmath::Rad;
use cgmath::Angle;
use glium::glutin;
use glium::glutin::Event;
use glium::glutin::WindowEvent;
use glium::backend;
use glium::Surface;

#[derive(Copy, Clone)]
struct Boid
{
    position: Point2<f32>,
    direction: Vector2<f32>,
}

impl Default for Boid
{
    fn default() -> Boid
    {
        Boid
        {
            position: Point2::new(0.0, 0.0),
            direction: Vector2::new(0.0, 0.0),
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
const BOIDS_NUMBER: usize = 500;
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
            position: (0f32, 0f32)
        }
    }
}

implement_vertex!(Vertex, position);

fn get_group(boids: &[Boid; BOIDS_NUMBER], center: Point2<f32>, range: f32) -> Vec<&Boid>
{
    let mut result = Vec::new();
    for i in boids.iter()
    {
        if i.position.distance(center) <= range
        {
            result.push(i);
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
        if group_density > BOIDS_MAX_DENSITY
        {
            let mut average_position = Point2::new(0.0, 0.0);
            for i in &group
            {
                average_position += i.position.to_vec();
            }
            average_position /= group.len() as f32;
            let mut delta = average_position - boids[i].position;
            delta = delta.normalize();
            if delta.x.is_nan() || delta.y.is_nan()
            {
                delta = Vector2::new(0.0, 0.0);
            }
            new_boids[i].direction -= delta * BOIDS_SEPARATION_RATE;
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
        let mut average_position = Point2::new(0.0, 0.0);
        for i in &group
        {
            average_position += i.position.to_vec();
        }
        average_position /= group.len() as f32;
        let mut delta = average_position - boids[i].position;
        delta = delta.normalize();
        if delta.x.is_nan() || delta.y.is_nan()
        {
            delta = Vector2::new(0.0, 0.0);
        }
        new_boids[i].direction += delta * BOIDS_COHESION_RATE;
    }
    std::mem::swap(boids, &mut new_boids);
}

fn alignment(boids: &mut [Boid; BOIDS_NUMBER]) -> ()
{
    let mut new_boids = *boids;
    for i in 0..BOIDS_NUMBER
    {
        let group = get_group(boids, boids[i].position, BOIDS_ALIGNMENT_RANGE);
        let mut average_direction = Vector2::new(0.0, 0.0);
        for i in &group
        {
            average_direction += i.direction;
        }
        average_direction /= group.len() as f32;
        average_direction = average_direction.normalize();
        if average_direction.x.is_nan() || average_direction.y.is_nan()
        {
            average_direction = Vector2::new(0.0, 0.0);
        }
        let delta = average_direction - boids[i].direction;
        new_boids[i].direction += delta * BOIDS_ALIGNMENT_RATE;
    }
    std::mem::swap(boids, &mut new_boids);
}

fn simulate(boids: &mut [Boid; BOIDS_NUMBER], map_size: &Vector2<f32>) -> ()
{
    let mut new_boids = *boids;
    for i in 0..BOIDS_NUMBER
    {
        new_boids[i].direction.x += ((rand::random::<f32>() * 2.0) - 1.0) * BOIDS_CHAOS;
        new_boids[i].direction.y += ((rand::random::<f32>() * 2.0) - 1.0) * BOIDS_CHAOS;
        new_boids[i].direction.normalize();
        new_boids[i].position = boids[i].position + (boids[i].direction * BOIDS_VELOCITY);
    }
    std::mem::swap(boids, &mut new_boids);
    for i in 0..BOIDS_NUMBER
    {
        if boids[i].position.x < 0.0
        {
            boids[i].position.x = map_size.x - 1.0;
        }
        if boids[i].position.y < 0.0
        {
            boids[i].position.y = map_size.y - 1.0;
        }
        if boids[i].position.x >= map_size.x
        {
            boids[i].position.x = 0.0;
        }
        if boids[i].position.y >= map_size.y
        {
            boids[i].position.y = 0.0;
        }
    }
}

fn rotate_point(point: Point2<f32>, center: Point2<f32>, angle: Rad<f32>) -> Point2<f32>
{
    let (px, py) = point.into();
    let (cx, cy) = center.into();
    let x = angle.cos() * (px - cx) - angle.sin() * (py - cy) + cx;
    let y = angle.sin() * (px - cx) + angle.cos() * (py - cy) + cy;
    Point2
    {
        x: x,
        y: y,
    }
}

fn update_vertices(boids: &[Boid; BOIDS_NUMBER], vertices: &mut [Vertex]) -> ()
{
    let mut iv = 0usize;
    for i in boids.iter()
    {
        let center = i.position + Vector2::new(-(3f32.sqrt() / 2.0), 0.0);
        let angle = i.direction.angle(Vector2::new(1.0, 0.0));
        let vertex0 = i.position;
        let vertex1 = i.position + Vector2::new(BOIDS_SIZE, -(BOIDS_SIZE / 2.0));
        let vertex2 = i.position + Vector2::new(BOIDS_SIZE, (BOIDS_SIZE / 2.0));
        let rotated0 = rotate_point(vertex0, center, angle);
        let rotated1 = rotate_point(vertex1, center, angle);
        let rotated2 = rotate_point(vertex2, center, angle);
        vertices[(iv * 3)].position = rotated0.into();
        vertices[(iv * 3) + 1].position = rotated1.into();
        vertices[(iv * 3) + 2].position = rotated2.into();
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
    let screen_size = (monitor_id.get_dimensions().0 as f32, monitor_id.get_dimensions().1 as f32);
    window.set_fullscreen(Some(monitor_id));

    let mut window_open = true;

    let mut boids: [Boid; BOIDS_NUMBER] = [Boid::default(); BOIDS_NUMBER];
    for i in 0..BOIDS_NUMBER
    {
        boids[i].position.x = rand::random::<f32>() * screen_size.0;
        boids[i].position.y = rand::random::<f32>() * screen_size.1;
        boids[i].direction.x = (rand::random::<f32>() * 2.0) - 1.0;
        boids[i].direction.y = (rand::random::<f32>() * 2.0) - 1.0;
    }

    let vertices: [Vertex; BOIDS_NUMBER * 3] = [Vertex::default(); BOIDS_NUMBER * 3];

    let mut vertex_buffer = glium::VertexBuffer::new(&display, &vertices).unwrap();
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
    let uniforms = uniform!
    {
        matrix: matrix,
        screen_size: screen_size,
    };
    while window_open
    {
        separation(&mut boids);
        alignment(&mut boids);
        cohesion(&mut boids);
        simulate(&mut boids, &Vector2::from(screen_size));
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
