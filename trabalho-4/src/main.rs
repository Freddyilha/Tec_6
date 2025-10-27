use minifb::{Key, MouseButton, Window, WindowOptions};
use rand::Rng;

const WIDTH: usize = 400;
const HEIGHT: usize = 400;
const WHITE: u32 = 0x00FFFFFF;
const RED: u32 = 0x00FF0000;
const BLACK: u32 = 0x00080808;
const ORANGE: u32 = 0x00FF963C;

type Point = (usize, usize);
type Polygon = Vec<Point>;

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
        let radius = rng.random_range(20..=max_radius) as f32;

        let x = center_x as f32 + radius * angle.cos();
        let y = center_y as f32 + radius * angle.sin();

        let x = (x as usize).clamp(0, WIDTH - 1);
        let y = (y as usize).clamp(0, HEIGHT - 1);

        points.push((x, y));
    }

    polygons.push(points)
}

fn main() {
    let mut polygons: Vec<Polygon> = Vec::new();
    let mut polygons_expanded: Vec<Polygon> = Vec::new();
    polygons.push(vec![(20, 20), (60, 20), (60, 60), (20, 60)]);
    polygons.push(vec![(200, 20), (260, 20), (260, 60), (200, 60)]);
    polygons.push(vec![(71, 272), (91, 321), (147, 314)]);

    let robot: Polygon = vec![(200, 200), (240, 200), (240, 240), (200, 240)];

    let mut window = Window::new("Moving Box", WIDTH, HEIGHT, WindowOptions::default()).unwrap();
    let mut buffer: Vec<u32> = vec![0; WIDTH * HEIGHT];
    let mut was_pressed = false;

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
            for polygon in &polygons {
                minkowski_sum(polygon, &robot, &mut polygons_expanded);
            }
        }

        if window.is_key_pressed(Key::Space, minifb::KeyRepeat::No) {
            polygons_expanded.clear();
        }

        if let Some((x, y)) = window.get_mouse_pos(minifb::MouseMode::Clamp) {
            let mouse_x = x as usize;
            let mouse_y = y as usize;

            if is_pressed && !was_pressed {
                generate_random_obstacle(mouse_x, mouse_y, &mut polygons);
            }
        }

        was_pressed = is_pressed;
        window.update_with_buffer(&buffer, WIDTH, HEIGHT).unwrap();
    }
}
