use chrono::prelude::*;
use csv::Writer;
use minifb::{MouseButton, Window, WindowOptions};
use std::error::Error;
use std::fs::OpenOptions;
use std::path::Path;
use std::time::{Duration, Instant};

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

fn draw_square(buffer: &mut Vec<u32>, side: usize, top_left: usize) {
    for i in 0..side {
        let row_start = top_left + (i * WIDTH);
        let row_end = row_start + side;
        buffer[row_start..row_end].fill(RED);
    }
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

fn draw_line(buffer: &mut Vec<u32>, thickness: usize, size: usize, top_left: usize, offset: usize) {
    for i in 0..thickness {
        let start_index = (top_left + i) * WIDTH + offset;
        let end_index = start_index + size;
        buffer[start_index..end_index].fill(BLACK);
    }
}

fn main() {
    let move_interval = Duration::from_millis(25);
    let red_square_size = 20;

    let mut stats = Statistics::new();
    let mut buffer: Vec<u32> = vec![0; WIDTH * HEIGHT];
    let mut last_move = Instant::now();
    let mut window = Window::new("Moving Box", WIDTH, HEIGHT, WindowOptions::default()).unwrap();
    let mut x = 0;
    let mut was_pressed = false;
    let mut dots: Vec<(usize, usize)> = Vec::new();

    let mut lines: Vec<(usize, usize, usize)> = Vec::new();
    lines.push((WIDTH, 100, 0));
    lines.push((WIDTH / 2, 50, 50));

    while window.is_open() && !window.is_key_down(minifb::Key::Escape) {
        buffer.fill(WHITE);
        stats.increment_frames();
        let is_pressed = window.get_mouse_down(MouseButton::Left);

        if last_move.elapsed() >= move_interval {
            x = (x + 1) % (WIDTH - red_square_size);
            last_move = Instant::now();
        }

        draw_square(&mut buffer, red_square_size, x);

        for (size, top_left, offset) in &lines {
            draw_line(&mut buffer, 5, *size, *top_left, *offset);
        }

        for (x, y) in &dots {
            draw_circle(&mut buffer, *x, *y, 5);
        }

        if let Some((mx, my)) = window.get_mouse_pos(minifb::MouseMode::Clamp) {
            let (x, y) = (mx as usize, my as usize);

            stats.set_mouse_x(x);
            stats.set_mouse_y(y);

            if is_pressed && !was_pressed {
                stats.increment_clicks();

                let idx = y * WIDTH + x;

                if buffer[idx] == RED {
                    stats.increment_click_on_dots();
                }

                if buffer[idx] == BLACK {
                    stats.increment_click_on_lines();
                }

                if buffer[idx] == WHITE {
                    dots.push((x, y));
                }
            }
        }

        save_statistics(&stats).unwrap();
        was_pressed = is_pressed;
        window.update_with_buffer(&buffer, WIDTH, HEIGHT).unwrap();
    }
}
