use chrono::prelude::*;
use csv::Writer;
use minifb::{Key, MouseButton, Window, WindowOptions};
use rand::Rng;
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

fn distance_from_line(
    line_start: &(usize, usize),
    line_end: &(usize, usize),
    point: &(usize, usize),
) -> f64 {
    let (x0, y0) = (line_start.0 as isize, line_start.1 as isize);
    let (x1, y1) = (line_end.0 as isize, line_end.1 as isize);
    let (px, py) = (point.0 as isize, point.1 as isize);

    let num = ((y1 - y0) * px - (x1 - x0) * py + x1 * y0 - y1 * x0).abs() as f64;
    let den = (((y1 - y0).pow(2) + (x1 - x0).pow(2)) as f64).sqrt();

    if den == 0.0 { 0.0 } else { num / den }
}

fn cross_product(
    line_start: &(usize, usize),
    line_end: &(usize, usize),
    point: &(usize, usize),
) -> isize {
    let (x1, y1) = (line_start.0 as isize, line_start.1 as isize);
    let (x2, y2) = (line_end.0 as isize, line_end.1 as isize);
    let (px, py) = (point.0 as isize, point.1 as isize);

    (x2 - x1) * (py - y1) - (y2 - y1) * (px - x1)
}

fn quick_hull(dots: &Vec<(usize, usize)>) -> Vec<(usize, usize)> {
    let mut convex_hull: Vec<(usize, usize)> = Vec::new();
    let mut sorted_by_x_dots = dots.clone();
    sorted_by_x_dots.sort_by_key(|&(x, _y)| x);

    let left_most = sorted_by_x_dots.first().unwrap();
    let right_most = sorted_by_x_dots.last().unwrap();

    let mut upper: Vec<(usize, usize)> = Vec::new();
    let mut lower: Vec<(usize, usize)> = Vec::new();

    for dot in &sorted_by_x_dots {
        let cross_result = cross_product(&left_most, &right_most, &dot);

        if cross_result > 0 {
            upper.push(dot.clone());
        } else if cross_result < 0 {
            lower.push(dot.clone());
        }
    }

    convex_hull.push(left_most.clone());
    find_hull(&upper, left_most, right_most, &mut convex_hull);
    convex_hull.push(right_most.clone());
    find_hull(&lower, right_most, left_most, &mut convex_hull);

    convex_hull
}

fn find_hull(
    half_dots: &Vec<(usize, usize)>,
    start_node: &(usize, usize),
    end_node: &(usize, usize),
    convex_hull: &mut Vec<(usize, usize)>,
) {
    if half_dots.is_empty() {
        return;
    }

    let mut max_distance = 0.0;
    let mut furthest_point = half_dots[0];

    for &dot in half_dots.iter() {
        let distance = distance_from_line(start_node, end_node, &dot);
        if distance > max_distance {
            max_distance = distance;
            furthest_point = dot;
        }
    }

    convex_hull.push(furthest_point.clone());

    let mut left_upper: Vec<(usize, usize)> = Vec::new();
    let mut right_upper: Vec<(usize, usize)> = Vec::new();

    for &p in half_dots.iter() {
        if cross_product(start_node, &furthest_point, &p) > 0 {
            left_upper.push(p);
        } else if cross_product(&furthest_point, end_node, &p) > 0 {
            right_upper.push(p);
        }
    }

    find_hull(&left_upper, start_node, &furthest_point, convex_hull);
    find_hull(&right_upper, &furthest_point, end_node, convex_hull);
}

fn sort_hull_points(hull: &mut Vec<(usize, usize)>) {
    let (sum_x, sum_y): (f64, f64) = hull
        .iter()
        .map(|&(x, y)| (x as f64, y as f64))
        .fold((0.0, 0.0), |(sx, sy), (x, y)| (sx + x, sy + y));

    let len = hull.len() as f64;
    let center = (sum_x / len, sum_y / len);

    hull.sort_by(|a, b| {
        let ang_a = (a.1 as f64 - center.1).atan2(a.0 as f64 - center.0);
        let ang_b = (b.1 as f64 - center.1).atan2(b.0 as f64 - center.0);
        ang_a.partial_cmp(&ang_b).unwrap()
    });
}

fn generate_random_points(dots: &mut Vec<(usize, usize)>, quantity: usize) {
    println!("Generating {} random points", quantity);

    let mut rng = rand::rng();

    for _ in 0..quantity {
        let random_x: usize = rng.random_range(0..WIDTH);
        let random_y: usize = rng.random_range(0..HEIGHT);

        dots.push((random_x, random_y));
    }
}

fn main() {
    let mut stats = Statistics::new();
    let mut buffer: Vec<u32> = vec![0; WIDTH * HEIGHT];
    let mut window = Window::new("Moving Box", WIDTH, HEIGHT, WindowOptions::default()).unwrap();
    let mut was_pressed = false;

    let mut dots: Vec<(usize, usize)> = Vec::new();

    let mut lines: Vec<(usize, usize, usize, usize)> = Vec::new();

    while window.is_open() && !window.is_key_down(minifb::Key::Escape) {
        buffer.fill(WHITE);
        stats.increment_frames();
        let is_pressed = window.get_mouse_down(MouseButton::Left);

        for (x, y) in &dots {
            draw_circle(&mut buffer, *x, *y, 5);
        }

        for (x0, y0, x1, y1) in &lines {
            draw_line(&mut buffer, *x0, *y0, *x1, *y1);
        }

        if window.is_key_pressed(Key::Space, minifb::KeyRepeat::No) {
            generate_random_points(&mut dots, 10);
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

                    let mut hull = quick_hull(&dots);
                    sort_hull_points(&mut hull);
                    lines.clear();

                    for i in 1..hull.len() {
                        lines.push((hull[i - 1].0, hull[i - 1].1, hull[i].0, hull[i].1));
                    }

                    lines.push((
                        hull[hull.len() - 1].0,
                        hull[hull.len() - 1].1,
                        hull[0].0,
                        hull[0].1,
                    ));
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
