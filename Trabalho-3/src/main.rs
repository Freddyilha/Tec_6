use chrono::prelude::*;
use csv::Writer;
use minifb::{Key, MouseButton, Window, WindowOptions};
use std::error::Error;
use std::fs::OpenOptions;
use std::path::Path;

/*
* The calculation for the position on a 1D array on the screen is
* ROW = (vertical * WIDTH) + COLUMN = horizontal
*/

const WIDTH: usize = 200;
const HEIGHT: usize = 200;
const WHITE: u32 = 0x00FFFFFF;
const RED: u32 = 0x00FF0000;
const BLACK: u32 = 0x00080808;

struct Statistics {
    clicks_on_dots: usize,
    clicks_on_lines: usize,
    number_of_clicks: usize,
    mouse_x: usize,
    mouse_y: usize,
    frames_count: usize,
}

impl Statistics {
    fn new() -> Self {
        Statistics {
            clicks_on_dots: 0,
            clicks_on_lines: 0,
            number_of_clicks: 0,
            mouse_x: 0,
            mouse_y: 0,
            frames_count: 0,
        }
    }

    fn increment_frames(&mut self) {
        self.frames_count += 1;
    }

    fn increment_clicks(&mut self) {
        self.number_of_clicks += 1;
    }

    fn increment_click_on_dots(&mut self) {
        self.clicks_on_dots += 1;
    }

    fn increment_click_on_lines(&mut self) {
        self.clicks_on_lines += 1;
    }

    fn set_mouse_x(&mut self, x: usize) {
        self.mouse_x = x;
    }

    fn set_mouse_y(&mut self, y: usize) {
        self.mouse_y = y;
    }
}

fn save_statistics(stats: &Statistics) -> Result<(), Box<dyn Error>> {
    let path = "stats.csv";
    let file_exists = Path::new(path).exists();

    let file = OpenOptions::new().append(true).create(true).open(path)?;

    let mut wtr = Writer::from_writer(file);

    if !file_exists {
        wtr.write_record(&[
            "clicks_on_dots",
            "clicks_on_lines",
            "number_of_clicks",
            "mouse_x",
            "mouse_y",
            "frames_count",
            "timestamp",
        ])?;
    }

    wtr.write_record(&[
        stats.clicks_on_dots.to_string(),
        stats.clicks_on_lines.to_string(),
        stats.number_of_clicks.to_string(),
        stats.mouse_x.to_string(),
        stats.mouse_y.to_string(),
        stats.frames_count.to_string(),
        Local::now().to_string(),
    ])?;

    wtr.flush()?;
    Ok(())
}

fn draw_circle(buffer: &mut [u32], cx: usize, cy: usize, radius: usize) {
    let r2 = (radius * radius) as isize;

    for y in (cy.saturating_sub(radius))..=(cy + radius).min(HEIGHT - 1) {
        for x in (cx.saturating_sub(radius))..=(cx + radius).min(WIDTH - 1) {
            let dx = x as isize - cx as isize;
            let dy = y as isize - cy as isize;

            if dx * dx + dy * dy <= r2 {
                let idx = y * WIDTH + x;
                buffer[idx] = RED;
            }
        }
    }
}

fn draw_line(buffer: &mut [u32], x0: usize, y0: usize, x1: usize, y1: usize) {
    let mut x0 = x0 as isize;
    let mut y0 = y0 as isize;
    let x1 = x1 as isize;
    let y1 = y1 as isize;

    let dx = (x1 - x0).abs();
    let sx = if x0 < x1 { 1 } else { -1 };
    let dy = -(y1 - y0).abs();
    let sy = if y0 < y1 { 1 } else { -1 };
    let mut err = dx + dy;

    loop {
        if x0 >= 0 && y0 >= 0 && (x0 as usize) < WIDTH && (y0 as usize) < HEIGHT {
            let idx = y0 as usize * WIDTH + x0 as usize;
            buffer[idx] = BLACK;
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

fn is_point_on_dot(mx: usize, my: usize, dot: (usize, usize), radius: usize) -> bool {
    let (dx, dy) = (mx as isize - dot.0 as isize, my as isize - dot.1 as isize);
    dx * dx + dy * dy <= (radius as isize).pow(2)
}

fn cross_product(o: &(usize, usize), a: &(usize, usize), b: &(usize, usize)) -> isize {
    let (ox, oy) = (o.0 as isize, o.1 as isize);
    let (ax, ay) = (a.0 as isize, a.1 as isize);
    let (bx, by) = (b.0 as isize, b.1 as isize);
    (ax - ox) * (by - oy) - (ay - oy) * (bx - ox)
}

fn convex_hull(dots: &Vec<(usize, usize)>) {
    println!("Running Convex Hull");

    let mut sorted_by_x_dots = dots.clone();
    // let mut sorted_by_y_dots = dots.clone();

    for (i, dot) in dots.iter().enumerate() {
        println!("dot-{} x:{} y:{}", i, dot.0, dot.1);
    }

    println!("-----------------------------------------------------------------");

    sorted_by_x_dots.sort_by_key(|&(x, _y)| x);

    let left_most = sorted_by_x_dots.first().unwrap();
    let right_most = sorted_by_x_dots.last().unwrap();
    println!("Left_most: {}.{}", left_most.0, left_most.1);
    println!("Right_most: {}.{}", right_most.0, right_most.1);

    println!("-----------------------------------------------------------------");
    let mut upper: Vec<(usize, usize)> = Vec::new();
    let mut lower: Vec<(usize, usize)> = Vec::new();

    for dot in &sorted_by_x_dots {
        let cross_result = cross_product(&right_most, &left_most, &dot);

        if cross_result > 0 {
            upper.push(dot.clone());
        } else if cross_result < 0 {
            lower.push(dot.clone());
        }
    }

    for (i, dot) in upper.iter().enumerate() {
        println!("dot-{} x:{} y:{}", i, dot.0, dot.1);
    }

    println!("-----------------------------------------------------------------");
    for (i, dot) in lower.iter().enumerate() {
        println!("dot-{} x:{} y:{}", i, dot.0, dot.1);
    }
}

fn main() {
    let mut stats = Statistics::new();
    let mut buffer: Vec<u32> = vec![0; WIDTH * HEIGHT];
    let mut window = Window::new("Moving Box", WIDTH, HEIGHT, WindowOptions::default()).unwrap();
    let mut was_pressed = false;

    let mut dots: Vec<(usize, usize)> = Vec::new();
    dots.push((25, 40));
    dots.push((100, 40));
    dots.push((160, 170));

    let mut lines: Vec<(usize, usize, usize, usize)> = Vec::new();
    lines.push((0, 0, WIDTH - 1, HEIGHT - 1));
    lines.push((0, HEIGHT / 2, WIDTH - 1, HEIGHT / 2));

    while window.is_open() && !window.is_key_down(minifb::Key::Escape) {
        buffer.fill(WHITE);
        stats.increment_frames();
        let is_pressed = window.get_mouse_down(MouseButton::Left);

        for (x, y) in &dots {
            draw_circle(&mut buffer, *x, *y, 5);
        }

        if window.is_key_pressed(Key::Space, minifb::KeyRepeat::No) {
            convex_hull(&dots)
        }

        if let Some((mx, my)) = window.get_mouse_pos(minifb::MouseMode::Clamp) {
            let (x, y) = (mx as usize, my as usize);

            stats.set_mouse_x(x);
            stats.set_mouse_y(y);

            if is_pressed && !was_pressed {
                stats.increment_clicks();

                let idx = y * WIDTH + x;

                if buffer[idx] == WHITE {
                    dots.push((x, y));
                }

                if buffer[idx] == RED {
                    stats.increment_click_on_dots();

                    for (i, dot) in dots.iter().enumerate() {
                        if is_point_on_dot(x, y, *dot, 5) {
                            println!("Clicked on dot {}", i);
                        }
                    }
                }
            }
        }

        save_statistics(&stats).unwrap();
        was_pressed = is_pressed;
        window.update_with_buffer(&buffer, WIDTH, HEIGHT).unwrap();
    }
}
