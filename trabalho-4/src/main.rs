use minifb::{Key, Window, WindowOptions};
use rayon::prelude::*;

const WIDTH: usize = 400;
const HEIGHT: usize = 400;
const WHITE: u32 = 0x00FFFFFF;
const RED: u32 = 0x00FF0000;
const BLACK: u32 = 0x00080808;
const ROBOT_SIZE: usize = 40;

fn draw_square(buffer: &mut Vec<u32>, x: usize, y: usize, side: usize, color: u32) {
    buffer
        .par_chunks_mut(WIDTH)
        .enumerate()
        .skip(y)
        .take(side)
        .for_each(|(_, row)| {
            row[x..x + side].fill(color);
        });
}

fn calculate_minkowski_addition(
    squares_excess: &mut Vec<(usize, usize, usize)>,
    squares: &[(usize, usize, usize)],
) {
    for &(x, y, side) in squares {
        let expanded = (
            x.saturating_sub(ROBOT_SIZE / 2),
            y.saturating_sub(ROBOT_SIZE / 2),
            side + ROBOT_SIZE,
        );
        squares_excess.push(expanded);
    }
}

fn main() {
    let mut triangles: Vec<(usize, usize, usize)> = Vec::new();
    let mut squares: Vec<(usize, usize, usize)> = Vec::new();
    let mut squares_excess: Vec<(usize, usize, usize)> = Vec::new();

    triangles.push((3, 5, 7));
    squares.push((50, 50, 50));

    let mut window = Window::new("Moving Box", WIDTH, HEIGHT, WindowOptions::default()).unwrap();
    let mut buffer: Vec<u32> = vec![0; WIDTH * HEIGHT];

    while window.is_open() && !window.is_key_down(minifb::Key::Escape) {
        buffer.fill(WHITE);

        for (x, y, side) in &squares_excess {
            draw_square(&mut buffer, *x, *y, *side, RED);
        }

        for (x, y, side) in &squares {
            draw_square(&mut buffer, *x, *y, *side, BLACK);
        }

        if window.is_key_pressed(Key::Space, minifb::KeyRepeat::No) {
            calculate_minkowski_addition(&mut squares_excess, &squares);
        }

        window.update_with_buffer(&buffer, WIDTH, HEIGHT).unwrap();
    }
}
