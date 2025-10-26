use minifb::{Key, MouseButton, Window, WindowOptions};
use rayon::prelude::*;

const WIDTH: usize = 400;
const HEIGHT: usize = 400;
const WHITE: u32 = 0x00FFFFFF;
const RED: u32 = 0x00FF0000;
const BLACK: u32 = 0x00080808;

fn draw_square(buffer: &mut Vec<u32>, x: usize, y: usize, side: usize) {
    buffer
        .par_chunks_mut(WIDTH)
        .enumerate()
        .skip(y)
        .take(side)
        .for_each(|(_, row)| {
            row[x..x + side].fill(BLACK);
        });
}

fn main() {
    let mut triangles: Vec<(usize, usize, usize)> = Vec::new();
    let mut squares: Vec<(usize, usize, usize)> = Vec::new();
    let robot_size: u8 = 4;

    triangles.push((3, 5, 7));
    squares.push((50, 10, 50));

    let mut window = Window::new("Moving Box", WIDTH, HEIGHT, WindowOptions::default()).unwrap();
    let mut buffer: Vec<u32> = vec![0; WIDTH * HEIGHT];

    while window.is_open() && !window.is_key_down(minifb::Key::Escape) {
        buffer.fill(WHITE);

        for (x, y, side) in &squares {
            draw_square(&mut buffer, *x, *y, *side);
        }

        if window.is_key_pressed(Key::Space, minifb::KeyRepeat::No) {
            calculate_minkowski_addition()
        }

        window.update_with_buffer(&buffer, WIDTH, HEIGHT).unwrap();
    }
}
