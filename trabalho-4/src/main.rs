use chrono::prelude::*;
use csv::Writer;
use minifb::{Key, MouseButton, Window, WindowOptions};
use rand::Rng;
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

type Point = (usize, usize);
type Polygon = Vec<Point>;

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

    fill_polygon(buffer, polygon, color);
}

fn convex_hull(points: &Vec<Point>) -> Polygon {
    let mut pts = points.clone();
    pts.sort_by_key(|&(x, y)| (x, y));

    fn cross(o: Point, a: Point, b: Point) -> isize {
        (a.0 as isize - o.0 as isize) * (b.1 as isize - o.1 as isize)
            - (a.1 as isize - o.1 as isize) * (b.0 as isize - o.0 as isize)
    }

    let mut lower = Vec::new();
    for &p in &pts {
        while lower.len() >= 2 && cross(lower[lower.len() - 2], lower[lower.len() - 1], p) <= 0 {
            lower.pop();
        }
        lower.push(p);
    }

    let mut upper = Vec::new();
    for &p in pts.iter().rev() {
        while upper.len() >= 2 && cross(upper[upper.len() - 2], upper[upper.len() - 1], p) <= 0 {
            upper.pop();
        }
        upper.push(p);
    }

    lower.pop();
    upper.pop();
    lower.extend(upper);
    lower
}

fn minkowski_sum(a: &Polygon, b: &Polygon, polygons_expanded: &mut Vec<Polygon>) {
    let robot_center_x = b.iter().map(|&(x, _)| x as isize).sum::<isize>() / b.len() as isize;
    let robot_center_y = b.iter().map(|&(_, y)| y as isize).sum::<isize>() / b.len() as isize;

    let robot_reflected: Vec<(isize, isize)> = b
        .iter()
        .map(|&(x, y)| {
            let rel_x = x as isize - robot_center_x;
            let rel_y = y as isize - robot_center_y;
            (-rel_x, -rel_y)
        })
        .collect();

    let mut sum: Vec<Point> = Vec::new();
    for &(ox, oy) in a {
        for &(rx, ry) in &robot_reflected {
            let x_result = ox as isize + rx;
            let y_result = oy as isize + ry;

            let x_clamped = x_result.clamp(0, WIDTH as isize - 1) as usize;
            let y_clamped = y_result.clamp(0, HEIGHT as isize - 1) as usize;

            sum.push((x_clamped, y_clamped));
        }
    }

    let hull = convex_hull(&sum);
    polygons_expanded.push(hull);
}

fn generate_random_obstacle(center_x: usize, center_y: usize, polygons: &mut Vec<Polygon>) {
    let mut rng = rand::rng();
    let num_vertices = rng.random_range(3..=8);
    let max_radius = 50;

    let mut points: Vec<Point> = Vec::new();
    for i in 0..num_vertices {
        let angle = (i as f32 / num_vertices as f32) * 2.0 * std::f32::consts::PI;
        let radius = rng.random_range(10..=max_radius) as f32;

        let x = center_x as f32 + radius * angle.cos();
        let y = center_y as f32 + radius * angle.sin();

        let x = (x as usize).clamp(0, WIDTH - 1);
        let y = (y as usize).clamp(0, HEIGHT - 1);

        points.push((x, y));
    }

    polygons.push(points)
}

fn point_to_segment_distance(px: f32, py: f32, x1: f32, y1: f32, x2: f32, y2: f32) -> f32 {
    let dx = x2 - x1;
    let dy = y2 - y1;
    let len_sq = dx * dx + dy * dy;

    if len_sq == 0.0 {
        return ((px - x1) * (px - x1) + (py - y1) * (py - y1)).sqrt();
    }

    let t = ((px - x1) * dx + (py - y1) * dy) / len_sq;
    let t = t.clamp(0.0, 1.0);

    let closest_x = x1 + t * dx;
    let closest_y = y1 + t * dy;

    ((px - closest_x) * (px - closest_x) + (py - closest_y) * (py - closest_y)).sqrt()
}

fn min_distance_to_polygon_edges(point: Point, polygon: &Polygon) -> f32 {
    let mut min_dist = f32::MAX;
    let (px, py) = point;

    for i in 0..polygon.len() {
        let (x1, y1) = polygon[i];
        let (x2, y2) = polygon[(i + 1) % polygon.len()];

        let dist = point_to_segment_distance(
            px as f32, py as f32, x1 as f32, y1 as f32, x2 as f32, y2 as f32,
        );

        min_dist = min_dist.min(dist);
    }

    min_dist
}

fn min_distance_polygon_to_expanded(polygon: &Polygon, expanded: &Polygon) -> f32 {
    let mut min_dist = f32::MAX;

    for &vertex in polygon {
        let dist = min_distance_to_polygon_edges(vertex, expanded);
        min_dist = min_dist.min(dist);
    }

    min_dist
}

fn main() {
    let mut stats = Statistics::new();
    let mut polygons: Vec<Polygon> = Vec::new();
    let mut polygons_expanded: Vec<Polygon> = Vec::new();
    let mut last_log_time = Instant::now();
    let robot: Polygon = vec![(200, 200), (240, 200), (240, 240), (200, 240)];
    let mut window = Window::new("Moving Box", WIDTH, HEIGHT, WindowOptions::default()).unwrap();
    let mut buffer: Vec<u32> = vec![0; WIDTH * HEIGHT];
    let mut was_pressed = false;
    let mut distance_table: Vec<(usize, usize)> = Vec::new();

    polygons.push(vec![(20, 20), (60, 20), (60, 60), (20, 60)]);
    polygons.push(vec![(200, 20), (260, 20), (260, 60), (200, 60)]);
    polygons.push(vec![(71, 272), (91, 321), (147, 314)]);

    stats.obstacles_amount = 3;
    stats.points_amount = 22;

    while window.is_open() && !window.is_key_down(minifb::Key::Escape) {
        buffer.fill(WHITE);
        let is_pressed = window.get_mouse_down(MouseButton::Left);

        for expanded in &polygons_expanded {
            draw_polygon(&mut buffer, expanded, RED);
        }

        draw_polygon(&mut buffer, &robot, ORANGE);

        for polygon in &polygons {
            draw_polygon(&mut buffer, polygon, BLACK);
        }

        if window.is_key_pressed(Key::M, minifb::KeyRepeat::No) {
            let start_time = Instant::now();
            for polygon in &polygons {
                minkowski_sum(polygon, &robot, &mut polygons_expanded);
            }
            let duration = start_time.elapsed();

            stats.time_to_finish_in_micros = duration.as_micros() as usize;

            for i in 0..polygons.len() {
                let smallest_distance =
                    min_distance_polygon_to_expanded(&polygons[i], &polygons_expanded[i]);

                if !distance_table.iter().any(|(id, _)| *id == i) {
                    distance_table.push((i, smallest_distance as usize));
                }
            }

            for x in &distance_table {
                println!("obstacle:{}, min_distance:{}", x.0, x.1);
            }
        }

        if window.is_key_pressed(Key::Space, minifb::KeyRepeat::No) {
            polygons_expanded.clear();
        }

        if let Some((x, y)) = window.get_mouse_pos(minifb::MouseMode::Clamp) {
            let mouse_x = x as usize;
            let mouse_y = y as usize;

            if is_pressed && !was_pressed {
                let mut points_amount = 0;

                generate_random_obstacle(mouse_x, mouse_y, &mut polygons);
                stats.obstacles_amount += 1;

                for polygon in &polygons {
                    for _ in polygon {
                        points_amount += 2;
                    }
                }

                stats.points_amount = points_amount;
            }
        }

        if last_log_time.elapsed() >= Duration::from_secs(1) {
            save_statistics(&stats).unwrap();
            last_log_time = Instant::now();
        }
        was_pressed = is_pressed;
        window.update_with_buffer(&buffer, WIDTH, HEIGHT).unwrap();
    }
}
