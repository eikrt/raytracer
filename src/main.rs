extern crate sdl2;

use sdl2::pixels::Color;
use sdl2::rect::Rect;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::render::Canvas;
use std::time::Duration;
use rayon::prelude::*;
use lazy_static::lazy_static;

lazy_static! {
    static ref SCREEN_WIDTH: u32 = 160;
    static ref SCREEN_HEIGHT: u32 = 90;
    static ref PROJ_PLANE_W: u32 = 160;
    static ref PROJ_PLANE_H: u32 = 90;
    static ref FOV: f32 = 3.14 / 4.0;
    static ref PP_DIST: f32 = (*PROJ_PLANE_W as f32 / 2.0) / (*FOV / 2.0).tan();
    static ref COL_WIDTH: i32 = (*FOV / *PROJ_PLANE_W as f32) as i32;
    static ref nodes: Vec<Node> = generate_scene();
}

enum Shape {
    BALL,
}

struct Node {
    points: Vec<(f32,f32,f32)>,
    center: (f32,f32,f32),
    shape: Shape
}
impl Node {
    fn new(points: Vec<(f32,f32,f32)>, center: (f32,f32,f32), shape: Shape) -> Node {
        Node {
            points: points,
            center: center,
            shape: shape,
        }
    }
}
struct Pixel {
    rect: Rect,
    color: Color
}

impl Pixel {
    fn new(rect: Rect, color: Color) -> Pixel {
        Pixel {
            rect: rect,
            color: color,
        }
    }
}
fn ball_function(ix: f32, iy: f32, iz: f32, r: f32) -> Node {
    let mut vector = vec![];
    for i in 0..314 {
        for j in 0..628 {
            let a = i as f32 / 10.0;
            let b = j as f32 / 10.0;
            let x = a.sin() * b.cos() * r + ix;
            let y = a.sin() * b.sin() * r + iy;
            let z = a.cos() * r + iz;
            let tuple = (x.floor(),y.floor(),z.floor());
            if !vector.contains(&tuple) {
                vector.push(tuple);
            }
        }
    }
    vector.push((0.0,0.0,0.0));
    let n = Node::new(vector, (ix/2.0, iy/2.0, iz/2.0), Shape::BALL);
    return n;
}
fn generate_scene() -> Vec<Node> {
    let mut scene = vec![];
    for i in 0..3 {
        scene.push(ball_function(96.0, 32.0 + i as f32 * 18.0, 16.0, 16.0));
    }
    scene
}
fn shoot_ray(ang: f32, coords: (f32,f32,f32)) -> Option<(f32,f32,f32)>{
    let mut new_coords = (coords.0 + ang.cos(), coords.1 + ang.sin(), coords.2 + 1.0);
    if coords.0 > *PROJ_PLANE_W as f32 || coords.1 > *PROJ_PLANE_H as f32 || coords.0 < 0.0 || coords.1 < 0.0  || coords.2 < 0.0 || coords.2 > 4.0 {
        return None;
    }
    for n in &*nodes {
        if n.points.contains(&(coords.0.floor(), coords.1.floor(), coords.2.floor())) {
            return Some(coords);
        }
    }
    shoot_ray(ang, new_coords)
}
fn render_scene(pixels: &mut Vec<Pixel>) {
    let col_width_f32 = *COL_WIDTH as f32;

    // Create a thread-safe vector to collect pixels
    let pixels_par: Vec<Pixel> = (0..*SCREEN_WIDTH)
        .into_par_iter()
        .flat_map(|i| {
            let mut current_ang_x = i as f32 * col_width_f32;
            (0..*SCREEN_HEIGHT).into_par_iter().map(move |j| {
                let rect = Rect::new(i as i32, j as i32, 1, 1);
                let mut color = Color::RGB(0, 0, 0);
                let coords = shoot_ray(current_ang_x, (i as f32, j as f32, 0.0)); 
                if coords == None {
                    color = Color::RGB(0, 0, 0);
                }
                else {
                    let c = coords.unwrap();
                    let dist_from_viewer = ((c.0 - 0.0).powf(2.0) + (c.1 - 0.0).powf(2.0) + (c.2 - 0.0).powf(2.0)).sqrt();
                    color = Color::RGB((dist_from_viewer) as u8, 0, 0);
                }
                Pixel::new(rect, color)
            }).collect::<Vec<_>>() // Collect results for this column
        })
        .collect(); // Collect all pixels from all threads

    // Append all pixels to the original pixels vector
    pixels.extend(pixels_par);
}
fn main() {
    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();

    let mut window = video_subsystem.window("Raytracer", *SCREEN_WIDTH * 4, *SCREEN_HEIGHT * 4)
        .position_centered()
        .build()
        .unwrap();

    let mut canvas = window.clone().into_canvas().build().unwrap();

    canvas.set_draw_color(Color::RGB(255, 255, 255));
    canvas.clear();

    let mut pixels = Vec::new();
    render_scene(&mut pixels);
    for p in pixels.iter() {
        canvas.set_draw_color(p.color);
        let ratio_x = window.size().0 / *SCREEN_WIDTH;
        let ratio_y = window.size().1 / *SCREEN_HEIGHT;
        canvas.fill_rect(Rect::new((p.rect.x * ratio_x as i32) as i32, (p.rect.y * ratio_y as i32) as i32, (1 * ratio_x) as u32, (1 * ratio_y) as u32)).unwrap();
    }
    println!("rendered");
    canvas.present();

    // Event loop to keep the window open
    let mut is_fullscreen = false;
    let mut event_pump = sdl_context.event_pump().unwrap();
    'running: loop {
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit {..} |
                Event::KeyDown { keycode: Some(Keycode::Escape), .. } => {
                    break 'running;
                },
                Event::KeyDown { keycode: Some(Keycode::F), .. } => {
                    // Toggle fullscreen
                    if is_fullscreen {
                        window.set_fullscreen(sdl2::video::FullscreenType::Off).unwrap();
                        is_fullscreen = false;
                    } else {
                        window.set_fullscreen(sdl2::video::FullscreenType::Desktop).unwrap();
                        is_fullscreen = true;
                    }
                },
                _ => {}
            }
        }
        // Sleep for a short duration to prevent high CPU usage
        ::std::thread::sleep(Duration::from_millis(100));
    }
}

