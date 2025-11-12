use chrono::prelude::*;
use csv::Writer;
use minifb::{Key, MouseButton, Window, WindowOptions};
use rand::Rng;
use std::cmp::Ordering;
use std::collections::HashSet;
use std::collections::{BinaryHeap, HashMap};
use std::error::Error;
use std::fs::OpenOptions;
use std::path::Path;
use std::time::{Duration, Instant};

const WIDTH: usize = 1000;
const HEIGHT: usize = 1000;
const ROWS: usize = 10;
const COLUMNS: usize = 10;
const WHITE: u32 = 0x00FFFFFF;
const RED: u32 = 0x00FF0000;
const BLACK: u32 = 0x00080808;
const ORANGE: u32 = 0x00FF963C;

struct Statistics {
    obstacles_amount: usize,
    start_points: usize,
    end_points: usize,
    time_to_finish_in_micros: usize,
}

#[derive(Eq, PartialEq)]
enum Steps {
    Obstacles,
    Start,
    End,
}

impl Statistics {
    fn new() -> Self {
        Statistics {
            obstacles_amount: 0,
            start_points: 0,
            end_points: 0,
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
            "start_points",
            "end_points",
            "time_to_finish_in_micros",
        ])?;
    }

    wtr.write_record(&[
        Local::now().to_string(),
        stats.obstacles_amount.to_string(),
        stats.start_points.to_string(),
        stats.end_points.to_string(),
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

fn draw_matrix(buffer: &mut Vec<u32>) {
    for i in 1..ROWS {
        draw_line(
            buffer,
            (WIDTH / ROWS) * i,
            0,
            (WIDTH / ROWS) * i,
            HEIGHT,
            BLACK,
        );
    }

    for i in 1..COLUMNS {
        draw_line(
            buffer,
            0,
            (HEIGHT / COLUMNS) * i,
            WIDTH,
            (HEIGHT / COLUMNS) * i,
            BLACK,
        );
    }
}

fn draw_circle(buffer: &mut [u32], cx: usize, cy: usize, radius: usize, color: u32) {
    let r2 = (radius * radius) as isize;

    for y in (cy.saturating_sub(radius))..=(cy + radius).min(HEIGHT - 1) {
        for x in (cx.saturating_sub(radius))..=(cx + radius).min(WIDTH - 1) {
            let dx = x as isize - cx as isize;
            let dy = y as isize - cy as isize;

            if dx * dx + dy * dy <= r2 {
                let idx = y * WIDTH + x;
                buffer[idx] = color;
            }
        }
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
struct Node {
    x: i32,
    y: i32,
}

#[derive(Copy, Clone, PartialEq, Eq)]
struct State {
    cost: i32,
    position: Node,
}

impl Ord for State {
    fn cmp(&self, other: &Self) -> Ordering {
        other.cost.cmp(&self.cost)
    }
}

impl PartialOrd for State {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

fn heuristic(a: Node, b: Node) -> i32 {
    (a.x - b.x).abs() + (a.y - b.y).abs()
}

fn in_bounds(n: Node) -> bool {
    (0..WIDTH).contains(&(n.x as usize)) && (0..HEIGHT).contains(&(n.y as usize))
}

fn neighbors(node: Node, walls: &HashSet<Node>) -> Vec<Node> {
    let deltas = [(1, 0), (-1, 0), (0, 1), (0, -1)];
    let mut result = Vec::with_capacity(4);

    for (dx, dy) in deltas {
        let nx = node.x + dx;
        let ny = node.y + dy;

        if nx < 0 || ny < 0 || nx >= COLUMNS as i32 || ny >= ROWS as i32 {
            continue;
        }

        let next = Node { x: nx, y: ny };

        if !walls.contains(&next) {
            result.push(next);
        }
    }

    result
}

fn a_star(start: Node, goal: Node, walls: &HashSet<Node>) -> Option<Vec<Node>> {
    let mut open_set = BinaryHeap::new();
    let mut came_from: HashMap<Node, Node> = HashMap::new();
    let mut g_score: HashMap<Node, i32> = HashMap::new();

    g_score.insert(start, 0);
    open_set.push(State {
        cost: heuristic(start, goal),
        position: start,
    });

    while let Some(State { cost: _, position }) = open_set.pop() {
        if position == goal {
            let mut path = vec![position];
            let mut current = position;
            while let Some(&prev) = came_from.get(&current) {
                path.push(prev);
                current = prev;
            }
            path.reverse();
            return Some(path);
        }

        for neighbor in neighbors(position, walls) {
            let tentative_g = g_score.get(&position).unwrap_or(&i32::MAX) + 1;

            if tentative_g < *g_score.get(&neighbor).unwrap_or(&i32::MAX) {
                came_from.insert(neighbor, position);
                g_score.insert(neighbor, tentative_g);

                let f = tentative_g + heuristic(neighbor, goal);
                open_set.push(State {
                    cost: f,
                    position: neighbor,
                });
            }
        }
    }

    None
}

fn main() {
    let mut stats = Statistics::new();
    let mut last_log_time = Instant::now();
    let mut window =
        Window::new("Navigation grid", WIDTH, HEIGHT, WindowOptions::default()).unwrap();
    let mut buffer: Vec<u32> = vec![0; WIDTH * HEIGHT];
    let mut was_pressed = false;
    let mut start_points: Vec<(usize, usize)> = Vec::new();
    let mut end_points: Vec<(usize, usize)> = Vec::new();
    let mut currect_step = Steps::Obstacles;
    let mut walls: HashSet<Node> = HashSet::new();
    let mut lines: Vec<Vec<(usize, usize)>> = Vec::new();

    while window.is_open() && !window.is_key_down(minifb::Key::Escape) {
        buffer.fill(WHITE);
        let is_pressed = window.get_mouse_down(MouseButton::Left);

        if window.is_key_pressed(Key::Space, minifb::KeyRepeat::No) {
            currect_step = Steps::Start;
        }

        if window.is_key_pressed(Key::O, minifb::KeyRepeat::No) {
            currect_step = Steps::Obstacles;
        }

        if window.is_key_pressed(Key::Q, minifb::KeyRepeat::No) {
            start_points.clear();
            end_points.clear();
            walls.clear();
            lines.clear();
        }

        if window.is_key_pressed(Key::R, minifb::KeyRepeat::No) {
            let mut rng = rand::rng();
            let how_many = rng.random_range(3..=12);

            for _ in 0..how_many {
                let random_x_start = rng.random_range(0..=ROWS);
                let random_y_start = rng.random_range(0..=ROWS);
                start_points.push((random_x_start, random_y_start));

                let random_x_end = rng.random_range(0..=COLUMNS);
                let random_y_end = rng.random_range(0..=COLUMNS);
                end_points.push((random_x_end, random_y_end));
            }
        }

        if window.is_key_pressed(Key::A, minifb::KeyRepeat::No) {
            if currect_step == Steps::Start || currect_step == Steps::Obstacles {
                lines.clear();

                let start_time = Instant::now();
                for (x, y) in start_points.iter().zip(end_points.iter()) {
                    let start = Node {
                        x: x.0 as i32,
                        y: x.1 as i32,
                    };
                    let goal = Node {
                        x: y.0 as i32,
                        y: y.1 as i32,
                    };

                    if let Some(path) = a_star(start, goal, &walls) {
                        let mut temp_vec: Vec<(usize, usize)> = Vec::new();
                        for p in path {
                            temp_vec.push((p.x as usize, p.y as usize));
                        }

                        lines.push(temp_vec);
                    } else {
                        println!("No path found — goal is blocked.");
                    }
                }
                let duration = start_time.elapsed();

                stats.time_to_finish_in_micros = duration.as_micros() as usize;
            }
        }

        draw_matrix(&mut buffer);

        for wall in &walls {
            draw_square(
                &mut buffer,
                WIDTH / ROWS,
                ((wall.y as usize * 100) * WIDTH) + (wall.x as usize * 100),
            )
        }

        for (x, y) in &start_points {
            let x_new = x * 100 + ((WIDTH / ROWS) / 2);
            let y_new = y * 100 + ((HEIGHT / COLUMNS) / 2);

            draw_circle(&mut buffer, x_new, y_new, 10, RED);
        }

        for (x, y) in &end_points {
            let x_new = x * 100 + ((WIDTH / ROWS) / 2);
            let y_new = y * 100 + ((HEIGHT / COLUMNS) / 2);

            draw_circle(&mut buffer, x_new, y_new, 10, ORANGE);
        }

        for line in &lines {
            for i in 1..line.len() {
                draw_line(
                    &mut buffer,
                    line[i - 1].0 * 100 + ((WIDTH / ROWS) / 2),
                    line[i - 1].1 * 100 + ((HEIGHT / COLUMNS) / 2),
                    line[i].0 * 100 + ((WIDTH / ROWS) / 2),
                    line[i].1 * 100 + ((HEIGHT / COLUMNS) / 2),
                    BLACK,
                );
            }
        }

        if let Some((x, y)) = window.get_mouse_pos(minifb::MouseMode::Clamp) {
            if is_pressed && !was_pressed {
                let mouse_x = x as usize;
                let mouse_y = y as usize;
                let mod_x = mouse_x / (WIDTH / ROWS);
                let mod_y = mouse_y / (HEIGHT / COLUMNS);

                match currect_step {
                    Steps::Obstacles => {
                        if !start_points.contains(&(mod_x, mod_y))
                            & !end_points.contains(&(mod_x, mod_y))
                        {
                            stats.obstacles_amount += 1;
                            walls.insert(Node {
                                x: mod_x as i32,
                                y: mod_y as i32,
                            });
                        }
                    }
                    Steps::Start => {
                        if !walls.contains(&Node {
                            x: mod_x as i32,
                            y: mod_y as i32,
                        }) {
                            stats.start_points += 1;
                            start_points.push((mod_x, mod_y));
                            currect_step = Steps::End;
                        }
                    }
                    Steps::End => {
                        if !walls.contains(&Node {
                            x: mod_x as i32,
                            y: mod_y as i32,
                        }) {
                            stats.end_points += 1;
                            end_points.push((mod_x, mod_y));
                            currect_step = Steps::Start;
                        }
                    }
                }
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
