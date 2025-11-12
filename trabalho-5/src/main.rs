use chrono::prelude::*;
use csv::Writer;
use minifb::{Key, MouseButton, Window, WindowOptions};
use rand::Rng;
use std::collections::HashSet;
use std::error::Error;
use std::fs::OpenOptions;
use std::path::Path;
use std::time::{Duration, Instant};

const WIDTH: usize = 1000;
const HEIGHT: usize = 1000;
const WHITE: u32 = 0x00FFFFFF;
const RED: u32 = 0x00FF0000;
const BLACK: u32 = 0x00080808;
const ORANGE: u32 = 0x00FF963C;

struct Statistics {
    obstacles_amount: usize,
    points_amount: usize,
    time_to_finish_in_micros: usize,
}

impl Statistics {
    fn new() -> Self {
        Statistics {
            obstacles_amount: 0,
            points_amount: 0,
            time_to_finish_in_micros: 0,
        }
    }
}

fn save_statistics(stats: &Statistics) -> Result<(), Box<dyn Error>> {
    let path = "stats.csv";
    let file_exists = Path::new(path).exists();

    let file = OpenOptions::new().append(true).create(true).open(path)?;

    let mut wtr = Writer::from_writer(file);

    if !file_exists {
        wtr.write_record(&[
            "timestamp",
            "obstacles_amount",
            "points_amount",
            "time_to_finish_in_micros",
        ])?;
    }

    wtr.write_record(&[
        Local::now().to_string(),
        stats.obstacles_amount.to_string(),
        stats.points_amount.to_string(),
        stats.time_to_finish_in_micros.to_string(),
    ])?;

    wtr.flush()?;
    Ok(())
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

fn draw_square(buffer: &mut Vec<u32>, length: usize, top_left: usize) {
    for i in 0..length {
        let row_start = top_left + (i * WIDTH);
        let row_end = row_start + length;
        buffer[row_start..row_end].fill(BLACK);
    }
}

fn draw_matrix(buffer: &mut Vec<u32>, columns: usize, rows: usize) {
    for i in 1..rows {
        draw_line(
            buffer,
            (WIDTH / rows) * i,
            0,
            (WIDTH / rows) * i,
            HEIGHT,
            BLACK,
        );
    }

    for i in 1..columns {
        draw_line(
            buffer,
            0,
            (HEIGHT / columns) * i,
            WIDTH,
            (HEIGHT / columns) * i,
            BLACK,
        );
    }
}

fn main() {
    let mut stats = Statistics::new();
    let mut window =
        Window::new("Navigation grid", WIDTH, HEIGHT, WindowOptions::default()).unwrap();
    let mut buffer: Vec<u32> = vec![0; WIDTH * HEIGHT];
    let mut was_pressed = false;
    let rows: usize = 10;
    let columns: usize = 10;
    let mut grid: HashSet<(usize, usize)> = HashSet::new();

    while window.is_open() && !window.is_key_down(minifb::Key::Escape) {
        buffer.fill(WHITE);
        let is_pressed = window.get_mouse_down(MouseButton::Left);

        if window.is_key_pressed(Key::Space, minifb::KeyRepeat::No) {
            println!("banana");
        }

        draw_matrix(&mut buffer, rows, columns);

        for (x, y) in &grid {
            draw_square(&mut buffer, WIDTH / rows, ((y * 100) * WIDTH) + (x * 100))
        }

        if let Some((x, y)) = window.get_mouse_pos(minifb::MouseMode::Clamp) {
            if is_pressed && !was_pressed {
                let mouse_x = x as usize;
                let mouse_y = y as usize;

                let mod_x = mouse_x / (WIDTH / rows);
                let mod_y = mouse_y / (HEIGHT / columns);

                println!(
                    "X:{}, Y:{}, mod_x:{}, mod_y:{}",
                    mouse_x, mouse_y, mod_x, mod_y
                );

                grid.insert((mod_x, mod_y));
            }
        }

        was_pressed = is_pressed;
        window.update_with_buffer(&buffer, WIDTH, HEIGHT).unwrap();
    }
}
