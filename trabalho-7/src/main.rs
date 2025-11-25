use minifb::{Key, MouseButton, Window, WindowOptions};
use rand::Rng;
use std::cmp::Ordering;
use std::collections::HashSet;
use std::collections::{BinaryHeap, HashMap};

const WIDTH: usize = 1000;
const HEIGHT: usize = 1000;
const ROWS: usize = 20;
const COLUMNS: usize = 20;
const WHITE: u32 = 0x00FFFFFF;
const RED: u32 = 0x00FF0000;
const BLACK: u32 = 0x00080808;
const ORANGE: u32 = 0x00FF963C;
const CELL_WIDTH: usize = WIDTH / COLUMNS;
const CELL_HEIGHT: usize = HEIGHT / ROWS;

struct LineParams {
    pub x0: usize,
    pub y0: usize,
    pub x1: usize,
    pub y1: usize,
    pub color: u32,
}

struct SquareParams {
    pub x: usize,
    pub y: usize,
    pub color: u32,
}

struct CircleParams {
    pub x: usize,
    pub y: usize,
    pub radius: usize,
    pub color: u32,
}

struct PixelArtist;
struct ArtistFactory;

enum DrawType {
    Line(LineParams),
    Square(SquareParams),
    Circle(CircleParams),
}

enum ArtistType {
    Normal,
}

trait Artist {
    fn draw(&self, buffer: &mut [u32], item: &DrawType);
}

impl Artist for PixelArtist {
    fn draw(&self, buffer: &mut [u32], item: &DrawType) {
        match item {
            DrawType::Line(p) => draw_line(buffer, p),
            DrawType::Square(p) => draw_square(buffer, p),
            DrawType::Circle(p) => draw_circle(buffer, p),
        }
    }
}

impl ArtistFactory {
    fn create(kind: ArtistType) -> Box<dyn Artist> {
        match kind {
            ArtistType::Normal => Box::new(PixelArtist),
        }
    }
}

fn draw_line(buffer: &mut [u32], p: &LineParams) {
    let (mut x0, mut y0, x1, y1) = (p.x0 as i32, p.y0 as i32, p.x1 as i32, p.y1 as i32);
    let dx = (x1 - x0).abs();
    let sx = if x0 < x1 { 1 } else { -1 };
    let dy = -(y1 - y0).abs();
    let sy = if y0 < y1 { 1 } else { -1 };
    let mut err = dx + dy;
    loop {
        if x0 >= 0 && y0 >= 0 && (x0 as usize) < WIDTH && (y0 as usize) < HEIGHT {
            buffer[y0 as usize * WIDTH + x0 as usize] = p.color;
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

fn draw_circle(buffer: &mut [u32], p: &CircleParams) {
    let cx = p.x * CELL_HEIGHT + ((WIDTH / ROWS) / 2);
    let cy = p.y * CELL_WIDTH + ((HEIGHT / COLUMNS) / 2);

    let r2 = (p.radius * p.radius) as isize;

    for y in (cy.saturating_sub(p.radius))..=(cy + p.radius).min(HEIGHT - 1) {
        for x in (cx.saturating_sub(p.radius))..=(cx + p.radius).min(WIDTH - 1) {
            let dx = x as isize - cx as isize;
            let dy = y as isize - cy as isize;

            if dx * dx + dy * dy <= r2 {
                let idx = y * WIDTH + x;
                buffer[idx] = p.color;
            }
        }
    }
}

fn draw_square(buffer: &mut [u32], p: &SquareParams) {
    let top_left = (p.y * CELL_WIDTH) * WIDTH + p.x * CELL_HEIGHT;
    for i in 0..CELL_WIDTH {
        let row_start = top_left + (i * WIDTH);
        let row_end = row_start + CELL_HEIGHT;
        buffer[row_start..row_end].fill(p.color);
    }
}

struct DrawFactory;

#[derive(Eq, PartialEq)]
enum Steps {
    Obstacles,
    Start,
    End,
}

#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
struct Node {
    x: i32,
    y: i32,
}

impl Node {
    fn ux(&self) -> usize {
        self.x as usize
    }
    fn uy(&self) -> usize {
        self.y as usize
    }
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

trait MovementStrategy {
    fn get_neighbors(&self, node: Node, rows: usize, columns: usize) -> Vec<Node>;
    fn name(&self) -> &str;
}

struct OrthogonalMovement;

impl MovementStrategy for OrthogonalMovement {
    fn get_neighbors(&self, node: Node, rows: usize, columns: usize) -> Vec<Node> {
        // 4 directions: up, down, left, right
        let deltas = [(1, 0), (-1, 0), (0, 1), (0, -1)];
        let mut result = Vec::with_capacity(4);

        for (dx, dy) in deltas {
            let nx = node.x + dx;
            let ny = node.y + dy;

            if nx >= 0 && ny >= 0 && nx < columns as i32 && ny < rows as i32 {
                result.push(Node { x: nx, y: ny });
            }
        }

        result
    }

    fn name(&self) -> &str {
        "Orthogonal"
    }
}

struct DiagonalMovement;

impl MovementStrategy for DiagonalMovement {
    fn get_neighbors(&self, node: Node, rows: usize, columns: usize) -> Vec<Node> {
        // 8 directions: includes diagonals
        let deltas = [
            (1, 0),
            (-1, 0),
            (0, 1),
            (0, -1), // orthogonal
            (1, 1),
            (1, -1),
            (-1, 1),
            (-1, -1), // diagonal
        ];
        let mut result = Vec::with_capacity(8);

        for (dx, dy) in deltas {
            let nx = node.x + dx;
            let ny = node.y + dy;

            if nx >= 0 && ny >= 0 && nx < columns as i32 && ny < rows as i32 {
                result.push(Node { x: nx, y: ny });
            }
        }

        result
    }

    fn name(&self) -> &str {
        "Diagonal"
    }
}

fn heuristic(a: Node, b: Node) -> i32 {
    (a.x - b.x).abs() + (a.y - b.y).abs()
}

fn draw_matrix(buffer: &mut Vec<u32>, artist: &dyn Artist) {
    for i in 1..ROWS {
        artist.draw(
            buffer,
            &DrawType::Line(LineParams {
                x0: (WIDTH / ROWS) * i,
                y0: 0,
                x1: (WIDTH / ROWS) * i,
                y1: HEIGHT,
                color: BLACK,
            }),
        );
    }

    for i in 1..COLUMNS {
        artist.draw(
            buffer,
            &DrawType::Line(LineParams {
                x0: 0,
                y0: (HEIGHT / COLUMNS) * i,
                x1: WIDTH,
                y1: (HEIGHT / COLUMNS) * i,
                color: BLACK,
            }),
        );
    }
}

fn neighbors(node: Node, walls: &HashSet<Node>) -> Vec<Node> {
    let deltas = [(1, 0), (-1, 0), (0, 1), (0, -1)];
    let mut result = Vec::with_capacity(4);

    for (dx, dy) in deltas {
        let nx = node.x + dx;
        let ny = node.y + dy;

        if nx < 0 || ny < 0 || nx >= ROWS as i32 || ny >= COLUMNS as i32 {
            continue;
        }

        let next = Node { x: nx, y: ny };

        if !walls.contains(&next) {
            result.push(next);
        }
    }

    result
}

fn a_star(
    start: Node,
    goal: Node,
    walls: &HashSet<Node>,
    movement: &dyn MovementStrategy,
) -> Option<Vec<Node>> {
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

        // Use the movement strategy instead of hardcoded neighbors
        for neighbor in movement.get_neighbors(position, ROWS, COLUMNS) {
            if walls.contains(&neighbor) {
                continue;
            }

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

struct GameState {
    was_pressed: bool,
    start_points: Vec<(usize, usize)>,
    end_points: Vec<(usize, usize)>,
    currect_step: Steps,
    walls: HashSet<Node>,
    lines: Vec<Vec<(usize, usize)>>,
    movement_strategy: Box<dyn MovementStrategy>,
}

fn main() {
    let mut window =
        Window::new("Navigation grid", WIDTH, HEIGHT, WindowOptions::default()).unwrap();
    let mut buffer: Vec<u32> = vec![0; WIDTH * HEIGHT];

    let mut game_state = GameState {
        was_pressed: false,
        start_points: Vec::new(),
        end_points: Vec::new(),
        currect_step: Steps::Obstacles,
        walls: HashSet::new(),
        lines: Vec::new(),
        movement_strategy: Box::new(OrthogonalMovement),
    };

    game_loop(&mut window, &mut buffer, &mut game_state);
}

fn game_loop(window: &mut Window, buffer: &mut Vec<u32>, state: &mut GameState) {
    let artist = ArtistFactory::create(ArtistType::Normal);

    while window.is_open() && !window.is_key_down(minifb::Key::Escape) {
        buffer.fill(WHITE);
        let is_pressed = window.get_mouse_down(MouseButton::Left);

        if window.is_key_pressed(Key::Space, minifb::KeyRepeat::No) {
            state.currect_step = Steps::Start;
        }

        if window.is_key_pressed(Key::O, minifb::KeyRepeat::No) {
            state.currect_step = Steps::Obstacles;
        }

        if window.is_key_pressed(Key::Q, minifb::KeyRepeat::No) {
            state.start_points.clear();
            state.end_points.clear();
            state.walls.clear();
            state.lines.clear();
        }

        if window.is_key_pressed(Key::M, minifb::KeyRepeat::No) {
            if state.movement_strategy.name() == "Orthogonal" {
                state.movement_strategy = Box::new(DiagonalMovement);
            } else {
                state.movement_strategy = Box::new(OrthogonalMovement)
            }
            state.lines.clear();
        }

        if window.is_key_pressed(Key::R, minifb::KeyRepeat::No) {
            let mut rng = rand::rng();
            let how_many = rng.random_range(3..=12);

            for _ in 0..how_many {
                let random_x_start = rng.random_range(0..ROWS);
                let random_y_start = rng.random_range(0..ROWS);
                state.start_points.push((random_x_start, random_y_start));

                let random_x_end = rng.random_range(0..COLUMNS);
                let random_y_end = rng.random_range(0..COLUMNS);
                state.end_points.push((random_x_end, random_y_end));
            }
        }

        if window.is_key_pressed(Key::A, minifb::KeyRepeat::No) {
            if state.currect_step == Steps::Start || state.currect_step == Steps::Obstacles {
                state.lines.clear();

                for (x, y) in state.start_points.iter().zip(state.end_points.iter()) {
                    let start = Node {
                        x: x.0 as i32,
                        y: x.1 as i32,
                    };
                    let goal = Node {
                        x: y.0 as i32,
                        y: y.1 as i32,
                    };

                    if let Some(path) =
                        a_star(start, goal, &state.walls, state.movement_strategy.as_ref())
                    {
                        let mut temp_vec: Vec<(usize, usize)> = Vec::new();
                        for p in path {
                            temp_vec.push((p.x as usize, p.y as usize));
                        }

                        state.lines.push(temp_vec);
                    } else {
                        println!("No path found — goal is blocked.");
                    }
                }
            }
        }

        draw_matrix(buffer, artist.as_ref());

        for node in &state.walls {
            artist.draw(
                buffer,
                &DrawType::Square(SquareParams {
                    x: node.ux(),
                    y: node.uy(),
                    color: BLACK,
                }),
            );
        }

        for (x, y) in &state.start_points {
            let (x, y) = (*x as usize, *y as usize);

            artist.draw(
                buffer,
                &DrawType::Circle(CircleParams {
                    x: x,
                    y: y,
                    radius: 10,
                    color: RED,
                }),
            );
        }

        for (x, y) in &state.end_points {
            let (x, y) = (*x as usize, *y as usize);

            artist.draw(
                buffer,
                &DrawType::Circle(CircleParams {
                    x: x,
                    y: y,
                    radius: 10,
                    color: ORANGE,
                }),
            );
        }

        for line in &state.lines {
            for i in 1..line.len() {
                artist.draw(
                    buffer,
                    &DrawType::Line(LineParams {
                        x0: line[i - 1].0 * CELL_HEIGHT + ((WIDTH / ROWS) / 2),
                        y0: line[i - 1].1 * CELL_WIDTH + ((HEIGHT / COLUMNS) / 2),
                        x1: line[i].0 * CELL_HEIGHT + ((WIDTH / ROWS) / 2),
                        y1: line[i].1 * CELL_WIDTH + ((HEIGHT / COLUMNS) / 2),
                        color: BLACK,
                    }),
                );
            }
        }

        if let Some((x, y)) = window.get_mouse_pos(minifb::MouseMode::Clamp) {
            if is_pressed && !state.was_pressed {
                let mouse_x = x as usize;
                let mouse_y = y as usize;
                let mod_x = mouse_x / (WIDTH / ROWS);
                let mod_y = mouse_y / (HEIGHT / COLUMNS);

                match state.currect_step {
                    Steps::Obstacles => {
                        if !state.start_points.contains(&(mod_x, mod_y))
                            & !state.end_points.contains(&(mod_x, mod_y))
                        {
                            state.walls.insert(Node {
                                x: mod_x as i32,
                                y: mod_y as i32,
                            });
                        }
                    }
                    Steps::Start => {
                        if !state.walls.contains(&Node {
                            x: mod_x as i32,
                            y: mod_y as i32,
                        }) {
                            state.start_points.push((mod_x, mod_y));
                            state.currect_step = Steps::End;
                        }
                    }
                    Steps::End => {
                        if !state.walls.contains(&Node {
                            x: mod_x as i32,
                            y: mod_y as i32,
                        }) {
                            state.end_points.push((mod_x, mod_y));
                            state.currect_step = Steps::Start;
                        }
                    }
                }
            }
        }

        state.was_pressed = is_pressed;
        window.update_with_buffer(&buffer, WIDTH, HEIGHT).unwrap();
    }
}
