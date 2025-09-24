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

fn draw_square(side: usize, buffer: &mut Vec<u32>, top_left: usize) {
    for i in 0..side {
        let row_start = top_left + (i * WIDTH);
        let row_end = row_start + side;
        buffer[row_start..row_end].fill(RED);

        if i > 0 {
            buffer[row_start - 1] = WHITE;
        }
    }
}

fn draw_line(thickness: usize, size: usize, buffer: &mut Vec<u32>, top_left: usize) {
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

    buffer.fill(WHITE);
    while window.is_open() && !window.is_key_down(minifb::Key::Escape) {
        let is_pressed = window.get_mouse_down(MouseButton::Left);

        draw_square(red_square_size, &mut buffer, x);

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
            x = (x + 1) % (WIDTH - red_square_size);
            last_move = Instant::now();
        }

        // TODO: This need to me move over to draw square since it's a fix for it
        if x == 0 {
            buffer.fill(WHITE);
        }

        draw_line(5, 100, &mut buffer, 100);

        was_pressed = is_pressed;
        window.update_with_buffer(&buffer, WIDTH, HEIGHT).unwrap();
    }
}
