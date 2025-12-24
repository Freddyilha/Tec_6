use minifb::{Key, MouseButton, Window, WindowOptions};
use rand::Rng;
use std::cmp::Ordering;
use std::collections::HashSet;
use std::collections::{BinaryHeap, HashMap};
use std::rc::Rc;

const WIDTH: usize = 1000;
const HEIGHT: usize = 1000;
const ROWS: usize = 20;
const COLUMNS: usize = 20;
const WHITE: u32 = 0x00FFFFFF;
const RED: u32 = 0x00FF0000;
const PALE_RED: u32 = 0x00FFF0F0;
const BLACK: u32 = 0x00080808;
const ORANGE: u32 = 0x00FF963C;
const LIGHT_BLUE: u32 = 0x00ADD8E6;
const CELL_WIDTH: usize = WIDTH / COLUMNS;
const CELL_HEIGHT: usize = HEIGHT / ROWS;

// Structs
struct PixelArtist;
struct ArtistFactory;
struct WindowInitHandler;
struct BufferInitHandler;
struct GameStateInitHandler;
struct CollisionLogger;
struct CollisionAssistant;

#[derive(Debug)]
struct OrthogonalMovement;
#[derive(Debug)]
struct DiagonalMovement;

#[derive(Clone, Eq, PartialEq, Debug)]
struct Agent {
    id: usize,
    start_point: Node,
    end_point: Option<Node>,
    current_point: Node,
    final_path: Option<Vec<Node>>,
    current_path_index: usize,
    collision_radius: Vec<Node>,
    forward_path: Vec<Node>,
}

#[derive(Debug, Clone)]
enum CollisionType {
    Proximity,
    Direct,
}

struct LineParams {
    pub x0: i32,
    pub y0: i32,
    pub x1: i32,
    pub y1: i32,
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

struct GameState {
    was_pressed: bool,
    start_points: Vec<Node>,
    end_points: Vec<Node>,
    currect_step: Steps,
    walls: HashSet<Node>,
    movement_strategy: Box<dyn MovementStrategy>,
}

#[derive(Debug)]
struct PathMovement {
    steps: Vec<Vec<Node>>,
}

struct WriteCommand {
    step: Vec<Node>,
}

struct DeleteCommand {
    deleted_steps: Vec<Vec<Node>>,
    count: usize,
}

struct CommandHistory {
    history: Vec<Box<dyn Command>>,
}

struct InitContext {
    window: Option<Window>,
    buffer: Option<Vec<u32>>,
    game_state: Option<GameState>,
}

#[derive(Debug, Clone)]
struct Line {
    start: Node,
    end: Node,
}

struct CollisionEvent {
    agent1_id: usize,
    agent2_id: usize,
    collision_type: CollisionType,
    collision_point: Node,
}

struct CollisionDetector {
    observers: Vec<Rc<dyn CollisionObserver>>,
}

// TRAITS
trait Artist {
    fn draw(&self, buffer: &mut [u32], item: &DrawType);
}

trait MovementStrategy {
    fn get_neighbors(&self, node: Node, rows: usize, columns: usize) -> Vec<Node>;
    fn name(&self) -> &str;
}

trait Command {
    fn execute(&mut self, movement: &mut PathMovement);
    fn undo(&mut self, movement: &mut PathMovement);
}

trait InitHandler {
    fn initialize(&mut self, context: &mut InitContext) -> Result<(), String>;
}

trait CollisionObserver {
    fn on_collision(&self, event: &CollisionEvent);
}

trait CollisionSubject {
    fn register_observer(&mut self, observer: Rc<dyn CollisionObserver>);
    fn remove_observer(&mut self, observer: Rc<dyn CollisionObserver>);
    fn notify_observers(&self, event: &CollisionEvent);
}

// ENUMS
enum DrawType {
    Line(LineParams),
    Square(SquareParams),
    Circle(CircleParams),
}

enum ArtistType {
    Normal,
}

#[derive(Eq, PartialEq)]
enum Steps {
    Obstacles,
    Start,
    End,
}

// IMPLEMENTATIONS
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

impl Node {
    fn ux(&self) -> usize {
        self.x as usize
    }
    fn uy(&self) -> usize {
        self.y as usize
    }

    fn fx(&self) -> f32 {
        self.x as f32
    }
    fn fy(&self) -> f32 {
        self.y as f32
    }
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

impl MovementStrategy for OrthogonalMovement {
    fn get_neighbors(&self, node: Node, rows: usize, columns: usize) -> Vec<Node> {
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

impl MovementStrategy for DiagonalMovement {
    fn get_neighbors(&self, node: Node, rows: usize, columns: usize) -> Vec<Node> {
        let deltas = [
            (1, 0),
            (-1, 0),
            (0, 1),
            (0, -1),
            (1, 1),
            (1, -1),
            (-1, 1),
            (-1, -1),
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

impl PathMovement {
    fn new() -> Self {
        PathMovement { steps: Vec::new() }
    }

    fn write(&mut self, step: &Vec<Node>) {
        self.steps.push(step.to_vec());
    }

    fn delete(&mut self, count: usize) {
        for _ in 0..count {
            self.steps.pop();
        }
    }

    fn get_steps(&self) -> &Vec<Vec<Node>> {
        &self.steps
    }
}

impl WriteCommand {
    fn new(step: Vec<Node>) -> Self {
        WriteCommand { step }
    }
}

impl Command for WriteCommand {
    fn execute(&mut self, movement: &mut PathMovement) {
        movement.write(&self.step);
    }

    fn undo(&mut self, movement: &mut PathMovement) {
        movement.delete(1);
    }
}

impl DeleteCommand {
    fn new(count: usize) -> Self {
        DeleteCommand {
            deleted_steps: Vec::new(),
            count,
        }
    }
}

impl Command for DeleteCommand {
    fn execute(&mut self, movement: &mut PathMovement) {
        let steps = movement.get_steps();
        let start = steps.len().saturating_sub(self.count);
        for i in start..steps.len() {
            let latest_step = steps[i].clone();
            self.deleted_steps.push(latest_step);
        }
        movement.delete(self.count);
    }

    fn undo(&mut self, movement: &mut PathMovement) {
        for step in &self.deleted_steps {
            movement.write(step);
        }
    }
}

impl CommandHistory {
    fn new() -> Self {
        CommandHistory {
            history: Vec::new(),
        }
    }

    fn execute(&mut self, mut command: Box<dyn Command>, movement: &mut PathMovement) {
        command.execute(movement);
        self.history.push(command);
    }

    fn undo(&mut self, movement: &mut PathMovement) {
        if let Some(mut command) = self.history.pop() {
            command.undo(movement);
        }
    }
}

impl InitHandler for WindowInitHandler {
    fn initialize(&mut self, context: &mut InitContext) -> Result<(), String> {
        let window = Window::new("Navigation grid", WIDTH, HEIGHT, WindowOptions::default())
            .map_err(|e| format!("Failed to create window: {:?}", e))?;
        context.window = Some(window);
        Ok(())
    }
}

impl InitHandler for BufferInitHandler {
    fn initialize(&mut self, context: &mut InitContext) -> Result<(), String> {
        context.buffer = Some(vec![0; WIDTH * HEIGHT]);
        Ok(())
    }
}

impl InitHandler for GameStateInitHandler {
    fn initialize(&mut self, context: &mut InitContext) -> Result<(), String> {
        let game_state = GameState {
            was_pressed: false,
            start_points: Vec::new(),
            end_points: Vec::new(),
            currect_step: Steps::Obstacles,
            walls: HashSet::new(),
            movement_strategy: Box::new(OrthogonalMovement),
        };
        context.game_state = Some(game_state);
        Ok(())
    }
}

impl CollisionDetector {
    fn new() -> Self {
        CollisionDetector {
            observers: Vec::new(),
        }
    }

    fn check_agents(&mut self, agents: &[Agent]) {
        for i in 0..agents.len() {
            for j in (i + 1)..agents.len() {
                let agent1 = &agents[i];
                let agent2 = &agents[j];

                if agent1.current_point == agent2.current_point {
                    let event = CollisionEvent {
                        agent1_id: agent1.id,
                        agent2_id: agent2.id,
                        collision_type: CollisionType::Direct,
                        collision_point: agent1.current_point,
                    };
                    self.notify_observers(&event);
                } else if self.check_path_collision(agent1, agent2) {
                    if let Some(collision_point) = self.find_collision_path(agent1, agent2) {
                        let event = CollisionEvent {
                            agent1_id: agent1.id,
                            agent2_id: agent2.id,
                            collision_type: CollisionType::Proximity,
                            collision_point,
                        };
                        self.notify_observers(&event);
                    }
                }
            }
        }
    }

    fn check_proximity_collision(&self, agent1: &Agent, agent2: &Agent) -> bool {
        for radius1 in &agent1.collision_radius {
            for radius2 in &agent2.collision_radius {
                if radius1 == radius2 {
                    return true;
                }
            }

            if *radius1 == agent2.current_point {
                return true;
            }
        }

        for radius2 in &agent2.collision_radius {
            if *radius2 == agent1.current_point {
                return true;
            }
        }

        false
    }

    fn check_path_collision(&self, agent1: &Agent, agent2: &Agent) -> bool {
        for path1 in &agent1.forward_path {
            for path2 in &agent2.forward_path {
                if path1 == path2 {
                    return true;
                }
            }

            if *path1 == agent2.current_point {
                return true;
            }
        }

        for path2 in &agent2.forward_path {
            if *path2 == agent1.current_point {
                return true;
            }
        }

        false
    }

    fn find_collision_point(&self, agent1: &Agent, agent2: &Agent) -> Option<Node> {
        for radius1 in &agent1.collision_radius {
            for radius2 in &agent2.collision_radius {
                if radius1 == radius2 {
                    return Some(*radius1);
                }
            }
            if *radius1 == agent2.current_point {
                return Some(*radius1);
            }
        }

        for radius2 in &agent2.collision_radius {
            if *radius2 == agent1.current_point {
                return Some(*radius2);
            }
        }

        None
    }

    fn find_collision_path(&self, agent1: &Agent, agent2: &Agent) -> Option<Node> {
        for path1 in &agent1.forward_path {
            for path2 in &agent2.forward_path {
                if path1 == path2 {
                    return Some(*path1);
                }
            }
            if *path1 == agent2.current_point {
                return Some(*path1);
            }
        }

        for path2 in &agent2.forward_path {
            if *path2 == agent1.current_point {
                return Some(*path2);
            }
        }

        None
    }
}

impl CollisionSubject for CollisionDetector {
    fn register_observer(&mut self, observer: Rc<dyn CollisionObserver>) {
        self.observers.push(observer);
    }

    fn remove_observer(&mut self, observer: Rc<dyn CollisionObserver>) {
        self.observers.retain(|obs| !Rc::ptr_eq(obs, &observer));
    }

    fn notify_observers(&self, event: &CollisionEvent) {
        for observer in &self.observers {
            observer.on_collision(event);
        }
    }
}

impl CollisionObserver for CollisionLogger {
    fn on_collision(&self, event: &CollisionEvent) {
        match event.collision_type {
            CollisionType::Direct => {}
            CollisionType::Proximity => {
                println!(
                    "Região em volta dos agentes {} e {} encostou com outro agente na posição: ({}, {})",
                    event.agent1_id,
                    event.agent2_id,
                    event.collision_point.x,
                    event.collision_point.y
                );
            }
        }
    }
}

impl CollisionObserver for CollisionAssistant {
    fn on_collision(&self, event: &CollisionEvent) {
        match event.collision_type {
            CollisionType::Direct => {
                println!(
                    "Colisão entre agentes {} e {}",
                    event.agent1_id, event.agent2_id
                );
            }
            CollisionType::Proximity => {
                println!(
                    "Perigo entre agentes {} e {}",
                    event.agent1_id, event.agent2_id
                );
            }
        }
    }
}

impl CollisionAssistant {
    fn new() -> Self {
        CollisionAssistant {}
    }
}

impl Agent {
    fn calculate_radius(&mut self) -> Vec<Node> {
        let deltas = [
            (1, 0),
            (-1, 0),
            (0, 1),
            (0, -1),
            (1, 1),
            (1, -1),
            (-1, 1),
            (-1, -1),
        ];

        let mut temp_radius: Vec<Node> = Vec::with_capacity(8);
        for (dx, dy) in deltas {
            if self.current_point.x + dx >= HEIGHT as i32 || self.current_point.x + dx < 0 {
                continue;
            };
            if self.current_point.y + dy >= WIDTH as i32 || self.current_point.y + dy < 0 {
                continue;
            };

            temp_radius.push(Node {
                x: (self.current_point.x + dx),
                y: (self.current_point.y + dy),
            });
        }

        temp_radius
    }

    fn calculate_forward(&self) -> Vec<Node> {
        let mut temp_forward = Vec::with_capacity(3);

        if let Some(path) = &self.final_path {
            for node in path.iter().skip(self.current_path_index + 1).take(3) {
                if *node == self.end_point.unwrap() {
                    break;
                }
                temp_forward.push(Node {
                    x: node.x,
                    y: node.y,
                });
            }
        }

        temp_forward
    }
}

// FUNCTIONS
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

fn heuristic(a: Node, b: Node) -> i32 {
    (a.x - b.x).abs() + (a.y - b.y).abs()
}

fn draw_matrix(buffer: &mut Vec<u32>, artist: &dyn Artist) {
    for i in 1..ROWS {
        artist.draw(
            buffer,
            &DrawType::Line(LineParams {
                x0: ((WIDTH / ROWS) * i) as i32,
                y0: 0,
                x1: ((WIDTH / ROWS) * i) as i32,
                y1: HEIGHT as i32,
                color: BLACK,
            }),
        );
    }

    for i in 1..COLUMNS {
        artist.draw(
            buffer,
            &DrawType::Line(LineParams {
                x0: 0,
                y0: ((HEIGHT / COLUMNS) * i) as i32,
                x1: WIDTH as i32,
                y1: ((HEIGHT / COLUMNS) * i) as i32,
                color: BLACK,
            }),
        );
    }
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

fn game_loop(window: &mut Window, buffer: &mut Vec<u32>, state: &mut GameState) {
    let artist = ArtistFactory::create(ArtistType::Normal);
    let mut movement = PathMovement::new();
    let mut history = CommandHistory::new();
    let mut agents: Vec<Agent> = Vec::new();

    let mut collision_detector = CollisionDetector::new();
    let logger = Rc::new(CollisionLogger);
    let assistant = Rc::new(CollisionAssistant::new());
    collision_detector.register_observer(logger);
    collision_detector.register_observer(assistant);

    while window.is_open() && !window.is_key_down(minifb::Key::Escape) {
        buffer.fill(WHITE);
        let is_pressed = window.get_mouse_down(MouseButton::Left);

        if window.is_key_pressed(Key::Space, minifb::KeyRepeat::No) {
            state.currect_step = Steps::Start;
        }

        if window.is_key_pressed(Key::O, minifb::KeyRepeat::No) {
            state.currect_step = Steps::Obstacles;
        }

        if window.is_key_pressed(Key::N, minifb::KeyRepeat::No) {
            history.undo(&mut movement);
        }

        if window.is_key_pressed(Key::B, minifb::KeyRepeat::No) {
            history.execute(Box::new(DeleteCommand::new(1)), &mut movement);
        }

        if window.is_key_pressed(Key::W, minifb::KeyRepeat::No) {
            for agent in &mut agents {
                if let Some(path) = &agent.final_path {
                    if agent.current_path_index + 1 < path.len() {
                        agent.current_path_index += 1;
                        agent.current_point = path[agent.current_path_index].clone();
                        agent.collision_radius = agent.calculate_radius();
                        agent.forward_path = agent.calculate_forward();
                    }
                }
            }
            collision_detector.check_agents(&agents);
        }

        if window.is_key_pressed(Key::N, minifb::KeyRepeat::No) {
            history.undo(&mut movement);
        }

        if window.is_key_pressed(Key::M, minifb::KeyRepeat::No) {
            if state.movement_strategy.name() == "Orthogonal" {
                state.movement_strategy = Box::new(DiagonalMovement);
            } else {
                state.movement_strategy = Box::new(OrthogonalMovement)
            }
        }

        if window.is_key_pressed(Key::R, minifb::KeyRepeat::No) {
            let mut rng = rand::rng();
            let how_many = rng.random_range(3..=12);

            for _ in 0..how_many {
                let temp_start = Node {
                    x: rng.random_range(0..ROWS) as i32,
                    y: rng.random_range(0..ROWS) as i32,
                };

                let temp_end = Node {
                    x: rng.random_range(0..COLUMNS) as i32,
                    y: rng.random_range(0..COLUMNS) as i32,
                };

                agents.push(Agent {
                    id: agents.len(),
                    start_point: temp_start,
                    end_point: Some(temp_end),
                    current_point: temp_start,
                    final_path: None,
                    current_path_index: 0,
                    collision_radius: Vec::with_capacity(8),
                    forward_path: Vec::with_capacity(3),
                });
            }
        }

        if window.is_key_pressed(Key::A, minifb::KeyRepeat::No) {
            if state.currect_step == Steps::Start || state.currect_step == Steps::Obstacles {
                movement.steps.clear();
                history.history.clear();

                for agent in &mut agents {
                    if let Some(path) = a_star(
                        agent.start_point,
                        agent.end_point.unwrap(),
                        &state.walls,
                        state.movement_strategy.as_ref(),
                    ) {
                        agent.final_path = Some(path);

                        agent.current_point = agent.start_point;
                        agent.current_path_index = 0;
                        agent.collision_radius = agent.calculate_radius();
                        agent.forward_path = agent.calculate_forward();
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

        for agent in &agents {
            artist.draw(
                buffer,
                &DrawType::Circle(CircleParams {
                    x: agent.current_point.ux(),
                    y: agent.current_point.uy(),
                    radius: 10,
                    color: RED,
                }),
            );

            // for radius in &agent.collision_radius {
            //     artist.draw(
            //         buffer,
            //         &DrawType::Circle(CircleParams {
            //             x: radius.x as usize,
            //             y: radius.y as usize,
            //             radius: 10,
            //             color: PALE_RED,
            //         }),
            //     );
            // }

            for radius in &agent.forward_path {
                artist.draw(
                    buffer,
                    &DrawType::Circle(CircleParams {
                        x: radius.x as usize,
                        y: radius.y as usize,
                        radius: 10,
                        color: LIGHT_BLUE,
                    }),
                );
            }

            if let Some(point) = &agent.end_point {
                artist.draw(
                    buffer,
                    &DrawType::Circle(CircleParams {
                        x: point.ux(),
                        y: point.uy(),
                        radius: 10,
                        color: ORANGE,
                    }),
                );
            }

            if let Some(path) = &agent.final_path {
                for i in 1..path.len() {
                    artist.draw(
                        buffer,
                        &DrawType::Line(LineParams {
                            x0: path[i - 1].x * CELL_HEIGHT as i32 + ((WIDTH / ROWS) / 2) as i32,
                            y0: path[i - 1].y * CELL_WIDTH as i32 + ((HEIGHT / COLUMNS) / 2) as i32,
                            x1: path[i].x * CELL_HEIGHT as i32 + ((WIDTH / ROWS) / 2) as i32,
                            y1: path[i].y * CELL_WIDTH as i32 + ((HEIGHT / COLUMNS) / 2) as i32,
                            color: BLACK,
                        }),
                    );
                }
            }
        }

        if let Some((x, y)) = window.get_mouse_pos(minifb::MouseMode::Clamp) {
            if is_pressed && !state.was_pressed {
                let mod_x = x as usize / (WIDTH / ROWS);
                let mod_y = y as usize / (HEIGHT / COLUMNS);

                let temp_node = Node {
                    x: mod_x as i32,
                    y: mod_y as i32,
                };

                match state.currect_step {
                    Steps::Obstacles => {
                        if !state.start_points.contains(&temp_node)
                            & !state.end_points.contains(&temp_node)
                        {
                            state.walls.insert(temp_node);
                        }
                    }
                    Steps::Start => {
                        if !state.walls.contains(&temp_node) {
                            agents.push(Agent {
                                id: agents.len(),
                                start_point: temp_node,
                                end_point: None,
                                current_point: temp_node,
                                final_path: None,
                                current_path_index: 0,
                                collision_radius: Vec::with_capacity(8),
                                forward_path: Vec::with_capacity(3),
                            });
                            state.currect_step = Steps::End;
                        }
                    }
                    Steps::End => {
                        if !state.walls.contains(&temp_node) {
                            let last_agent = agents.last_mut().unwrap();
                            last_agent.end_point = Some(temp_node);
                            last_agent.collision_radius = last_agent.calculate_radius();
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

fn main() {
    let mut handlers: Vec<Box<dyn InitHandler>> = vec![
        Box::new(WindowInitHandler),
        Box::new(BufferInitHandler),
        Box::new(GameStateInitHandler),
    ];

    let mut context = InitContext {
        window: None,
        buffer: None,
        game_state: None,
    };

    for handler in handlers.iter_mut() {
        if let Err(e) = handler.initialize(&mut context) {
            eprintln!("Initialization failed: {}", e);
            return;
        }
    }

    let mut window = context.window.unwrap();
    let mut buffer = context.buffer.unwrap();
    let mut game_state = context.game_state.unwrap();

    game_loop(&mut window, &mut buffer, &mut game_state);
}
