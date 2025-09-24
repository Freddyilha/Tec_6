use minifb::{MouseButton, MouseMode, Window, WindowOptions};
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

fn draw_line(buffer: &mut Vec<u32>, thickness: usize, size: usize, top_left: usize) {
    for i in 0..thickness {
        let start_index = (top_left + i) * WIDTH;
        let end_index = start_index + size;
        buffer[start_index..end_index].fill(BLACK);
    }
}

fn main() {
    let move_interval = Duration::from_millis(25);
    let red_square_size = 20;

    let mut buffer: Vec<u32> = vec![0; WIDTH * HEIGHT];
    let mut last_move = Instant::now();
    let mut window = Window::new("Moving Box", WIDTH, HEIGHT, WindowOptions::default()).unwrap();
    let mut x = 0;
    let mut was_pressed = false;
    let mut dots: Vec<(usize, usize)> = Vec::new();

    while window.is_open() && !window.is_key_down(minifb::Key::Escape) {
        buffer.fill(WHITE);
        let is_pressed = window.get_mouse_down(MouseButton::Left);

        if last_move.elapsed() >= move_interval {
            x = (x + 1) % (WIDTH - red_square_size);
            last_move = Instant::now();
        }

        draw_square(&mut buffer, red_square_size, x);
        draw_line(&mut buffer, 5, WIDTH, 100);
        draw_circle(&mut buffer, 150, 150, 10);

        for (x, y) in &dots {
            draw_circle(&mut buffer, *x, *y, 5);
        }

        if is_pressed && !was_pressed {
            if let Some((mx, my)) = window.get_mouse_pos(MouseMode::Clamp) {
                let (x, y) = (mx as usize, my as usize);

                let idx = y * WIDTH + x;

                if buffer[idx] != WHITE {
                    println!("CLICKED ON SOMETHING");
                }

                if buffer[idx] == WHITE {
                    dots.push((x, y));
                }
            }
        }

        was_pressed = is_pressed;
        window.update_with_buffer(&buffer, WIDTH, HEIGHT).unwrap();
    }
}
