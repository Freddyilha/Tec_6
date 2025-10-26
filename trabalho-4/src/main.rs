use minifb::{Key, Window, WindowOptions};
use rayon::prelude::*;

const WIDTH: usize = 400;
const HEIGHT: usize = 400;
const WHITE: u32 = 0x00FFFFFF;
const RED: u32 = 0x00FF0000;
const BLACK: u32 = 0x00080808;
const ROBOT_SIZE: usize = 40;

type Point = (usize, usize);
type Polygon = Vec<Point>;

fn calculate_minkowski_addition(
    squares_excess: &mut Vec<(usize, usize, usize)>,
    squares: &[(usize, usize, usize)],
) {
    for &(x, y, side) in squares {
        let expanded = (
            x.saturating_sub(ROBOT_SIZE / 2),
            y.saturating_sub(ROBOT_SIZE / 2),
            side + ROBOT_SIZE,
        );
        squares_excess.push(expanded);
    }
}

fn draw_line(buffer: &mut [u32], x0: usize, y0: usize, x1: usize, y1: usize, color: u32) {
    let (mut x0, mut y0, x1, y1) = (x0 as i32, y0 as i32, x1 as i32, y1 as i32);
    let dx = (x1 - x0).abs();
    let sx = if x0 < x1 { 1 } else { -1 };
    let dy = -(y1 - y0).abs();
    let sy = if y0 < y1 { 1 } else { -1 };
    let mut err = dx + dy;
    loop {
        if x0 >= 0 && y0 >= 0 && (x0 as usize) < WIDTH && (y0 as usize) < HEIGHT {
            buffer[y0 as usize * WIDTH + x0 as usize] = color;
        }
        if x0 == x1 && y0 == y1 {
            break;
        }
        let e2 = 2 * err;
        if e2 >= dy {
            err += dy;
            x0 += sx;
        }
        if e2 <= dx {
            err += dx;
            y0 += sy;
        }
    }
}

fn fill_polygon(buffer: &mut [u32], polygon: &Polygon, color: u32) {
    let min_y = polygon.iter().map(|&(_, y)| y).min().unwrap_or(0);
    let max_y = polygon.iter().map(|&(_, y)| y).max().unwrap_or(HEIGHT - 1);

    for y in min_y..=max_y {
        let mut intersections = Vec::new();

        for i in 0..polygon.len() {
            let (x0, y0) = polygon[i];
            let (x1, y1) = polygon[(i + 1) % polygon.len()];

            if (y0 <= y && y1 > y) || (y1 <= y && y0 > y) {
                let dy = y1 as f32 - y0 as f32;
                let dx = x1 as f32 - x0 as f32;
                let t = (y as f32 - y0 as f32) / dy;
                let x_int = x0 as f32 + t * dx;
                intersections.push(x_int as usize);
            }
        }

        intersections.sort_unstable();
        for pair in intersections.chunks(2) {
            if pair.len() == 2 {
                let (x_start, x_end) = (pair[0], pair[1]);
                if x_end > x_start && y < HEIGHT {
                    let start = y * WIDTH + x_start.min(WIDTH - 1);
                    let end = y * WIDTH + x_end.min(WIDTH - 1);
                    for px in start..end {
                        buffer[px] = color;
                    }
                }
            }
        }
    }
}

fn draw_polygon(buffer: &mut [u32], polygon: &Polygon, color: u32) {
    for i in 0..polygon.len() {
        let (x0, y0) = polygon[i];
        let (x1, y1) = polygon[(i + 1) % polygon.len()];
        draw_line(buffer, x0, y0, x1, y1, color);
    }

    fill_polygon(buffer, polygon, BLACK);
}

fn main() {
    let mut polygons: Vec<Polygon> = Vec::new();
    polygons.push(vec![(20, 20), (60, 20), (60, 60), (20, 60)]);

    let mut window = Window::new("Moving Box", WIDTH, HEIGHT, WindowOptions::default()).unwrap();
    let mut buffer: Vec<u32> = vec![0; WIDTH * HEIGHT];

    while window.is_open() && !window.is_key_down(minifb::Key::Escape) {
        buffer.fill(WHITE);

        for polygon in &polygons {
            draw_polygon(&mut buffer, polygon, BLACK)
        }

        if window.is_key_pressed(Key::M, minifb::KeyRepeat::No) {}

        if window.is_key_pressed(Key::Space, minifb::KeyRepeat::No) {
            polygons.clear();
        }

        window.update_with_buffer(&buffer, WIDTH, HEIGHT).unwrap();
    }
}
