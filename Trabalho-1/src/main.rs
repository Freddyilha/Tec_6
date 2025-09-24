use minifb::{MouseButton, MouseMode, Window, WindowOptions};
use std::time::{Duration, Instant};

const WIDTH: usize = 200;
const HEIGHT: usize = 200;
const WHITE: u32 = 0x00FFFFFF;
const RED: u32 = 0x00FF0000;
const BLACK: u32 = 0x00080808;

fn draw_square(side: usize, buffer: &mut [u32], top_left: usize) {
    for i in 0..side {
        let row_start = top_left + i * WIDTH;
        let row_end = row_start + side;
        buffer[row_start..row_end].fill(RED);
        // There is still an issue here I think this is reducing the square to 19x20
        buffer[row_start] = WHITE;
    }
}

fn main() {
    let move_interval = Duration::from_millis(25);

    let mut buffer: Vec<u32> = vec![0; WIDTH * HEIGHT];
    let mut last_move = Instant::now();
    let mut window = Window::new("Moving Box", WIDTH, HEIGHT, WindowOptions::default()).unwrap();
    let mut x = 0;
    let mut was_pressed = false;

    buffer.fill(WHITE);
    while window.is_open() && !window.is_key_down(minifb::Key::Escape) {
        let is_pressed = window.get_mouse_down(MouseButton::Left);

        draw_square(20, &mut buffer, x);

        if x == 0 {
            buffer.fill(WHITE);
        }

        if is_pressed && !was_pressed {
            if let Some((mx, my)) = window.get_mouse_pos(MouseMode::Clamp) {
                let (x, y) = (mx as usize, my as usize);

                let idx = y * WIDTH + x;

                if buffer[idx] != WHITE {
                    println!("CLICKED ON SOMETHING");
                }
            }
        }

        if last_move.elapsed() >= move_interval {
            x = (x + 1) % (WIDTH - 20); // move square horizontally
            last_move = Instant::now();
        }

        for i in 0..10 {
            let start_index = (100 + i) * WIDTH + 0;
            let end_index = start_index + 100;
            buffer[start_index..end_index].fill(BLACK);
        }

        was_pressed = is_pressed;
        window.update_with_buffer(&buffer, WIDTH, HEIGHT).unwrap();
    }
}
