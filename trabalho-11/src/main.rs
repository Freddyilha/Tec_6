use chrono::prelude::*;
use csv::Writer;
use minifb::{Key, MouseButton, Window, WindowOptions};
use rand::Rng;
use std::cell::RefCell;
use std::cmp::Ordering;
use std::collections::{BinaryHeap, HashMap, HashSet};
use std::error::Error;
use std::fs::OpenOptions;
use std::path::Path;
use std::rc::Rc;
use std::time::{Duration, Instant};

// ---------------------------------------------------------------------------
// Constants
// ---------------------------------------------------------------------------

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

// ---------------------------------------------------------------------------
// Vector / grid helpers
// ---------------------------------------------------------------------------

#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
struct Node {
    x: i32,
    y: i32,
}

impl Node {
    fn ux(self) -> usize { self.x as usize }
    fn uy(self) -> usize { self.y as usize }
}

fn move_dir(a: Node, b: Node) -> Node {
    Node { x: b.x - a.x, y: b.y - a.y }
}

fn dot(a: Node, b: Node) -> i32 {
    a.x * b.x + a.y * b.y
}

fn is_zero_dir(d: Node) -> bool {
    d.x == 0 && d.y == 0
}

fn rotate_right(d: Node) -> Node { Node { x: d.y, y: -d.x } }
fn rotate_left(d: Node) -> Node  { Node { x: -d.y, y: d.x } }
fn negate(d: Node) -> Node       { Node { x: -d.x, y: -d.y } }

fn in_bounds(n: Node) -> bool {
    n.x >= 0 && n.y >= 0 && (n.x as usize) < COLUMNS && (n.y as usize) < ROWS
}

// ---------------------------------------------------------------------------
// Statistics & CSV logging
// ---------------------------------------------------------------------------

struct Statistics {
    recalculations: usize,
    collisions: usize,
    detections: usize,
    total_path_length: usize,
    total_steps: usize,
}

impl Statistics {
    fn new() -> Self {
        Statistics {
            recalculations: 0,
            collisions: 0,
            detections: 0,
            total_path_length: 0,
            total_steps: 0,
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
            "recalculations",
            "collisions",
            "detections",
            "total_path_length",
            "total_steps",
        ])?;
    }

    wtr.write_record(&[
        Local::now().to_string(),
        stats.recalculations.to_string(),
        stats.collisions.to_string(),
        stats.detections.to_string(),
        stats.total_path_length.to_string(),
        stats.total_steps.to_string(),
    ])?;

    wtr.flush()?;
    Ok(())
}

// ---------------------------------------------------------------------------
// Drawing primitives
// ---------------------------------------------------------------------------

struct LineParams   { x0: i32, y0: i32, x1: i32, y1: i32, color: u32 }
struct SquareParams { x: usize, y: usize, color: u32 }
struct CircleParams { x: usize, y: usize, radius: usize, color: u32 }

enum DrawType {
    Line(LineParams),
    Square(SquareParams),
    Circle(CircleParams),
}

fn draw(buffer: &mut [u32], item: &DrawType) {
    match item {
        DrawType::Line(p)   => draw_line(buffer, p),
        DrawType::Square(p) => draw_square(buffer, p),
        DrawType::Circle(p) => draw_circle(buffer, p),
    }
}

fn draw_line(buffer: &mut [u32], p: &LineParams) {
    let (mut x0, mut y0) = (p.x0, p.y0);
    let (x1, y1)         = (p.x1, p.y1);
    let dx  = (x1 - x0).abs();
    let sx  = if x0 < x1 { 1 } else { -1 };
    let dy  = -(y1 - y0).abs();
    let sy  = if y0 < y1 { 1 } else { -1 };
    let mut err = dx + dy;

    loop {
        if x0 >= 0 && y0 >= 0 && (x0 as usize) < WIDTH && (y0 as usize) < HEIGHT {
            buffer[y0 as usize * WIDTH + x0 as usize] = p.color;
        }
        if x0 == x1 && y0 == y1 { break; }

        let e2 = 2 * err;
        if e2 >= dy { err += dy; x0 += sx; }
        if e2 <= dx { err += dx; y0 += sy; }
    }
}

fn draw_circle(buffer: &mut [u32], p: &CircleParams) {
    let cx = p.x * CELL_HEIGHT + CELL_WIDTH / 2;
    let cy = p.y * CELL_WIDTH  + CELL_HEIGHT / 2;
    let r2 = (p.radius * p.radius) as isize;

    let y_lo = cy.saturating_sub(p.radius);
    let y_hi = (cy + p.radius).min(HEIGHT - 1);
    let x_lo = cx.saturating_sub(p.radius);
    let x_hi = (cx + p.radius).min(WIDTH - 1);

    for y in y_lo..=y_hi {
        let dy = y as isize - cy as isize;
        for x in x_lo..=x_hi {
            let dx = x as isize - cx as isize;
            if dx * dx + dy * dy <= r2 {
                buffer[y * WIDTH + x] = p.color;
            }
        }
    }
}

fn draw_square(buffer: &mut [u32], p: &SquareParams) {
    let top_left = (p.y * CELL_WIDTH) * WIDTH + p.x * CELL_HEIGHT;
    for i in 0..CELL_WIDTH {
        let row_start = top_left + i * WIDTH;
        buffer[row_start..row_start + CELL_HEIGHT].fill(p.color);
    }
}

fn draw_matrix(buffer: &mut [u32]) {
    for i in 1..ROWS {
        let px = (WIDTH / ROWS) * i;
        draw(buffer, &DrawType::Line(LineParams {
            x0: px as i32, y0: 0, x1: px as i32, y1: HEIGHT as i32, color: BLACK,
        }));
    }
    for i in 1..COLUMNS {
        let py = (HEIGHT / COLUMNS) * i;
        draw(buffer, &DrawType::Line(LineParams {
            x0: 0, y0: py as i32, x1: WIDTH as i32, y1: py as i32, color: BLACK,
        }));
    }
}

// ---------------------------------------------------------------------------
// Movement strategy (Strategy pattern)
// ---------------------------------------------------------------------------

trait MovementStrategy {
    fn get_neighbors(&self, node: Node) -> Vec<Node>;
    fn name(&self) -> &str;
}

struct OrthogonalMovement;
struct DiagonalMovement;

impl MovementStrategy for OrthogonalMovement {
    fn get_neighbors(&self, node: Node) -> Vec<Node> {
        const DELTAS: [(i32, i32); 4] = [(1,0), (-1,0), (0,1), (0,-1)];
        DELTAS.iter()
            .map(|&(dx, dy)| Node { x: node.x + dx, y: node.y + dy })
            .filter(|n| in_bounds(*n))
            .collect()
    }
    fn name(&self) -> &str { "Orthogonal" }
}

impl MovementStrategy for DiagonalMovement {
    fn get_neighbors(&self, node: Node) -> Vec<Node> {
        const DELTAS: [(i32, i32); 8] = [
            (1,0), (-1,0), (0,1), (0,-1),
            (1,1), (1,-1), (-1,1), (-1,-1),
        ];
        DELTAS.iter()
            .map(|&(dx, dy)| Node { x: node.x + dx, y: node.y + dy })
            .filter(|n| in_bounds(*n))
            .collect()
    }
    fn name(&self) -> &str { "Diagonal" }
}

// ---------------------------------------------------------------------------
// Command pattern for wall/step history (undo / redo)
// ---------------------------------------------------------------------------

trait Command {
    fn execute(&mut self, steps: &mut Vec<Vec<Node>>);
    fn undo(&mut self, steps: &mut Vec<Vec<Node>>);
}

struct WriteCommand { step: Vec<Node> }

impl Command for WriteCommand {
    fn execute(&mut self, steps: &mut Vec<Vec<Node>>) {
        steps.push(self.step.clone());
    }
    fn undo(&mut self, steps: &mut Vec<Vec<Node>>) {
        steps.pop();
    }
}

struct DeleteCommand {
    count: usize,
    deleted: Vec<Vec<Node>>,
}

impl DeleteCommand {
    fn new(count: usize) -> Self {
        DeleteCommand { count, deleted: Vec::new() }
    }
}

impl Command for DeleteCommand {
    fn execute(&mut self, steps: &mut Vec<Vec<Node>>) {
        let start = steps.len().saturating_sub(self.count);
        self.deleted = steps[start..].to_vec();
        steps.truncate(start);
    }
    fn undo(&mut self, steps: &mut Vec<Vec<Node>>) {
        steps.append(&mut self.deleted);
    }
}

struct CommandHistory {
    history: Vec<Box<dyn Command>>,
}

impl CommandHistory {
    fn new() -> Self { CommandHistory { history: Vec::new() } }

    fn execute(&mut self, mut cmd: Box<dyn Command>, steps: &mut Vec<Vec<Node>>) {
        cmd.execute(steps);
        self.history.push(cmd);
    }

    fn undo(&mut self, steps: &mut Vec<Vec<Node>>) {
        if let Some(mut cmd) = self.history.pop() {
            cmd.undo(steps);
        }
    }
}

// ---------------------------------------------------------------------------
// Agent
// ---------------------------------------------------------------------------

#[derive(Clone, Eq, PartialEq, Debug)]
struct Agent {
    id: usize,
    start_point: Node,
    end_point: Option<Node>,
    current_point: Node,
    path: Option<Vec<Node>>,
    path_index: usize,
    collision_radius: Vec<Node>,
    forward_path: Vec<Node>,
}

impl Agent {
    fn new(id: usize, start: Node, end: Option<Node>) -> Self {
        let mut agent = Agent {
            id,
            start_point: start,
            end_point: end,
            current_point: start,
            path: None,
            path_index: 0,
            collision_radius: Vec::with_capacity(8),
            forward_path: Vec::with_capacity(2),
        };
        agent.collision_radius = agent.calc_radius();
        agent
    }

    fn calc_radius(&self) -> Vec<Node> {
        const DELTAS: [(i32, i32); 8] = [
            (1,0), (-1,0), (0,1), (0,-1),
            (1,1), (1,-1), (-1,1), (-1,-1),
        ];
        DELTAS.iter()
            .map(|&(dx, dy)| Node {
                x: self.current_point.x + dx,
                y: self.current_point.y + dy,
            })
            .filter(|n| in_bounds(*n))
            .collect()
    }

    fn calc_forward(&self) -> Vec<Node> {
        let Some(path) = &self.path else { return Vec::new() };
        let goal = match self.end_point { Some(g) => g, None => return Vec::new() };

        path.iter()
            .skip(self.path_index + 1)
            .take(2)
            .take_while(|&&n| n != goal)
            .copied()
            .collect()
    }

    fn direction(&self) -> Node {
        if let Some(path) = &self.path {
            if self.path_index + 1 < path.len() {
                return move_dir(self.current_point, path[self.path_index + 1]);
            }
        }
        if let Some(goal) = self.end_point {
            return Node {
                x: (goal.x - self.current_point.x).signum(),
                y: (goal.y - self.current_point.y).signum(),
            };
        }
        Node { x: 0, y: 0 }
    }

    fn refresh_cache(&mut self) {
        self.collision_radius = self.calc_radius();
        self.forward_path = self.calc_forward();
    }
}

// ---------------------------------------------------------------------------
// Collision detection (Observer pattern)
// ---------------------------------------------------------------------------

#[derive(Debug, Clone)]
enum CollisionType { Direct, Proximity }

struct CollisionEvent {
    agent1_id: usize,
    agent2_id: usize,
    collision_type: CollisionType,
    collision_point: Node,
}

trait CollisionObserver {
    fn on_collision(&self, event: &CollisionEvent);
}

#[derive(Hash, Eq, PartialEq, Copy, Clone)]
struct AgentPair(usize, usize);

impl AgentPair {
    fn new(a: usize, b: usize) -> Self {
        if a < b { AgentPair(a, b) } else { AgentPair(b, a) }
    }
}

struct CollisionDetector {
    observers: Vec<Rc<dyn CollisionObserver>>,
    ignored_pairs: HashSet<AgentPair>,
}

impl CollisionDetector {
    fn new() -> Self {
        CollisionDetector {
            observers: Vec::new(),
            ignored_pairs: HashSet::new(),
        }
    }

    fn register_observer(&mut self, obs: Rc<dyn CollisionObserver>) {
        self.observers.push(obs);
    }

    fn notify(&self, event: &CollisionEvent) {
        for obs in &self.observers {
            obs.on_collision(event);
        }
    }

    fn check_agents(&mut self, agents: &[Agent], stats: &mut Statistics) {
        for i in 0..agents.len() {
            for j in (i + 1)..agents.len() {
                let pair = AgentPair::new(agents[i].id, agents[j].id);
                if self.ignored_pairs.contains(&pair) { continue; }

                let (a, b) = (&agents[i], &agents[j]);

                if a.current_point == b.current_point {
                    self.notify(&CollisionEvent {
                        agent1_id: a.id,
                        agent2_id: b.id,
                        collision_type: CollisionType::Direct,
                        collision_point: a.current_point,
                    });
                    self.ignored_pairs.insert(pair);
                    stats.collisions += 1;
                } else if let Some(point) = Self::find_forward_collision(a, b) {
                    self.notify(&CollisionEvent {
                        agent1_id: a.id,
                        agent2_id: b.id,
                        collision_type: CollisionType::Proximity,
                        collision_point: point,
                    });
                    self.ignored_pairs.insert(pair);
                    stats.detections += 1;
                }
            }
        }
    }

    fn find_forward_collision(a: &Agent, b: &Agent) -> Option<Node> {
        let b_set: HashSet<Node> = b.forward_path.iter().copied().collect();

        for &node in &a.forward_path {
            if b_set.contains(&node) || node == b.current_point {
                return Some(node);
            }
        }
        for &node in &b.forward_path {
            if node == a.current_point {
                return Some(node);
            }
        }
        None
    }
}

// ---------------------------------------------------------------------------
// Collision observers
// ---------------------------------------------------------------------------

struct CollisionLogger;

impl CollisionObserver for CollisionLogger {
    fn on_collision(&self, event: &CollisionEvent) {
        match event.collision_type {
            CollisionType::Direct => {
                println!(
                    "DIRECT COLLISION: agents {} and {} at ({}, {})",
                    event.agent1_id, event.agent2_id,
                    event.collision_point.x, event.collision_point.y,
                );
            }
            CollisionType::Proximity => {
            }
        }
    }
}

struct CollisionAssistant {
    requests: RefCell<Vec<RerouteRequest>>,
}

#[derive(Debug, Clone)]
struct RerouteRequest {
    agent_id: usize,
    avoid_point: Node,
}

impl CollisionAssistant {
    fn new() -> Self { CollisionAssistant { requests: RefCell::new(Vec::new()) } }

    fn take_requests(&self) -> Vec<RerouteRequest> {
        std::mem::take(&mut *self.requests.borrow_mut())
    }

    fn has_requests(&self) -> bool {
        !self.requests.borrow().is_empty()
    }
}

impl CollisionObserver for CollisionAssistant {
    fn on_collision(&self, event: &CollisionEvent) {
        if let CollisionType::Proximity = event.collision_type {
            let mut reqs = self.requests.borrow_mut();
            reqs.push(RerouteRequest { agent_id: event.agent1_id, avoid_point: event.collision_point });
            reqs.push(RerouteRequest { agent_id: event.agent2_id, avoid_point: event.collision_point });
        }
    }
}

// ---------------------------------------------------------------------------
// A* pathfinding
// ---------------------------------------------------------------------------

#[derive(Copy, Clone, PartialEq, Eq)]
struct State { cost: i32, position: Node }

impl Ord for State {
    fn cmp(&self, other: &Self) -> Ordering { other.cost.cmp(&self.cost) }
}
impl PartialOrd for State {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> { Some(self.cmp(other)) }
}

fn heuristic(a: Node, b: Node) -> i32 {
    (a.x - b.x).abs() + (a.y - b.y).abs()
}

fn a_star(
    start: Node, goal: Node,
    walls: &HashSet<Node>, movement: &dyn MovementStrategy,
) -> Option<Vec<Node>> {
    a_star_inner(start, goal, walls, &HashSet::new(), None, movement)
}

fn a_star_with_avoidance(
    start: Node, goal: Node,
    walls: &HashSet<Node>, avoid: &HashSet<Node>,
    preferred_dir: Option<Node>, movement: &dyn MovementStrategy,
) -> Option<Vec<Node>> {
    a_star_inner(start, goal, walls, avoid, preferred_dir, movement)
}

fn a_star_inner(
    start: Node, goal: Node,
    walls: &HashSet<Node>, avoid: &HashSet<Node>,
    preferred_dir: Option<Node>, movement: &dyn MovementStrategy,
) -> Option<Vec<Node>> {
    let mut open = BinaryHeap::new();
    let mut came_from = HashMap::new();
    let mut g_score: HashMap<Node, i32> = HashMap::new();

    g_score.insert(start, 0);
    open.push(State { cost: heuristic(start, goal), position: start });

    while let Some(State { position, .. }) = open.pop() {
        if position == goal {
            let mut path = vec![position];
            let mut cur = position;
            while let Some(&prev) = came_from.get(&cur) {
                path.push(prev);
                cur = prev;
            }
            path.reverse();
            return Some(path);
        }

        let base_g = *g_score.get(&position).unwrap_or(&i32::MAX);

        for neighbor in movement.get_neighbors(position) {
            if walls.contains(&neighbor) || avoid.contains(&neighbor) { continue; }

            let mut tentative_g = base_g.saturating_add(1);

            if let Some(pref) = preferred_dir {
                let mv = move_dir(position, neighbor);
                if mv == pref {
                    tentative_g -= 4;
                } else if mv == negate(pref) {
                    tentative_g += 8;
                } else if dot(mv, pref) == 0 {
                    tentative_g -= 1;
                }
            }

            if tentative_g < *g_score.get(&neighbor).unwrap_or(&i32::MAX) {
                came_from.insert(neighbor, position);
                g_score.insert(neighbor, tentative_g);
                open.push(State {
                    cost: tentative_g + heuristic(neighbor, goal),
                    position: neighbor,
                });
            }
        }
    }
    None
}

// ---------------------------------------------------------------------------
// Reroute logic
// ---------------------------------------------------------------------------

fn process_reroute_requests(
    agents: &mut [Agent],
    requests: &[RerouteRequest],
    walls: &HashSet<Node>,
    movement: &dyn MovementStrategy,
    stats: &mut Statistics,
) {
    let mut by_point: HashMap<Node, Vec<usize>> = HashMap::new();
    for req in requests {
        by_point.entry(req.avoid_point).or_default().push(req.agent_id);
    }

    for (collision_point, agent_ids) in &by_point {
        let per_agent = compute_avoidance_plan(agents, agent_ids, *collision_point);

        for (agent_id, avoid_set, pref_dir) in per_agent {
            let agent = &agents[agent_id];
            let Some(goal) = agent.end_point else { continue };

            let pref = if is_zero_dir(pref_dir) { None } else { Some(pref_dir) };

            if let Some(new_path) = a_star_with_avoidance(
                agent.current_point, goal, walls, &avoid_set, pref, movement,
            ) {
                stats.recalculations += 1;
                let agent = &mut agents[agent_id];
                agent.path = Some(new_path);
                agent.path_index = 0;
                agent.refresh_cache();
            }
        }
    }
}

fn compute_avoidance_plan(
    agents: &[Agent],
    agent_ids: &[usize],
    collision_point: Node,
) -> Vec<(usize, HashSet<Node>, Node)> {
    let dirs: Vec<(usize, Node)> = agent_ids.iter()
        .filter_map(|&id| {
            let agent = &agents[id];
            let d = agent.direction();
            let final_dir = if is_zero_dir(d) {
                agent.end_point.map(|g| Node {
                    x: (g.x - agent.current_point.x).signum(),
                    y: (g.y - agent.current_point.y).signum(),
                }).unwrap_or(d)
            } else { d };
            Some((id, final_dir))
        })
        .collect();

    let mut plan = Vec::with_capacity(dirs.len());

    if dirs.len() >= 2 {
        let (a_id, a_dir) = dirs[0];
        let (b_id, _b_dir) = dirs[1];

        let axis = rotate_right(a_dir);

        let (steer_a, steer_b) = if a_id <= b_id {
            (axis, negate(axis))
        } else {
            (negate(axis), axis)
        };

        plan.push(make_avoid_entry(a_id, collision_point, steer_a));
        plan.push(make_avoid_entry(b_id, collision_point, steer_b));

        for &(id, dir) in &dirs[2..] {
            plan.push(make_avoid_entry(id, collision_point, rotate_right(dir)));
        }
    } else {
        for &(id, _) in &dirs {
            let mut avoid = HashSet::new();
            avoid.insert(collision_point);
            plan.push((id, avoid, Node { x: 0, y: 0 }));
        }
    }

    plan
}

fn make_avoid_entry(id: usize, collision_point: Node, avoid_dir: Node) -> (usize, HashSet<Node>, Node) {
    let mut avoid = HashSet::new();
    avoid.insert(collision_point);
    let nudge = Node {
        x: collision_point.x + avoid_dir.x,
        y: collision_point.y + avoid_dir.y,
    };
    if in_bounds(nudge) { avoid.insert(nudge); }
    (id, avoid, avoid_dir)
}

// ---------------------------------------------------------------------------
// Game state & initialization (Chain of Responsibility)
// ---------------------------------------------------------------------------

#[derive(Eq, PartialEq)]
enum Step { Obstacles, Start, End }

struct GameState {
    was_pressed: bool,
    current_step: Step,
    walls: HashSet<Node>,
    movement_strategy: Box<dyn MovementStrategy>,
    step_history: Vec<Vec<Node>>,
}

struct InitContext {
    window: Option<Window>,
    buffer: Option<Vec<u32>>,
    game_state: Option<GameState>,
}

trait InitHandler {
    fn initialize(&mut self, ctx: &mut InitContext) -> Result<(), String>;
}

struct WindowInitHandler;
struct BufferInitHandler;
struct GameStateInitHandler;

impl InitHandler for WindowInitHandler {
    fn initialize(&mut self, ctx: &mut InitContext) -> Result<(), String> {
        ctx.window = Some(
            Window::new("Navigation grid", WIDTH, HEIGHT, WindowOptions::default())
                .map_err(|e| format!("Window creation failed: {:?}", e))?,
        );
        Ok(())
    }
}

impl InitHandler for BufferInitHandler {
    fn initialize(&mut self, ctx: &mut InitContext) -> Result<(), String> {
        ctx.buffer = Some(vec![0; WIDTH * HEIGHT]);
        Ok(())
    }
}

impl InitHandler for GameStateInitHandler {
    fn initialize(&mut self, ctx: &mut InitContext) -> Result<(), String> {
        ctx.game_state = Some(GameState {
            was_pressed: false,
            current_step: Step::Obstacles,
            walls: HashSet::new(),
            movement_strategy: Box::new(OrthogonalMovement),
            step_history: Vec::new(),
        });
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// Input handling
// ---------------------------------------------------------------------------

fn handle_input(
    window: &Window,
    state: &mut GameState,
    agents: &mut Vec<Agent>,
    history: &mut CommandHistory,
    collision_detector: &mut CollisionDetector,
    stats: &mut Statistics,
) {
    // --- mode switches ---
    if window.is_key_pressed(Key::Space, minifb::KeyRepeat::No) {
        state.current_step = Step::Start;
    }
    if window.is_key_pressed(Key::O, minifb::KeyRepeat::No) {
        state.current_step = Step::Obstacles;
    }
    if window.is_key_pressed(Key::M, minifb::KeyRepeat::No) {
        state.movement_strategy = if state.movement_strategy.name() == "Orthogonal" {
            Box::new(DiagonalMovement)
        } else {
            Box::new(OrthogonalMovement)
        };
    }

    // --- undo / delete ---
    if window.is_key_pressed(Key::N, minifb::KeyRepeat::No) {
        history.undo(&mut state.step_history);
    }
    if window.is_key_pressed(Key::B, minifb::KeyRepeat::No) {
        history.execute(Box::new(DeleteCommand::new(1)), &mut state.step_history);
    }

    // --- step agents forward one tick ---
    if window.is_key_pressed(Key::W, minifb::KeyRepeat::No) {
        for agent in agents.iter_mut() {
            if let Some(path) = &agent.path {
                if agent.path_index + 1 < path.len() {
                    agent.path_index += 1;
                    agent.current_point = path[agent.path_index];
                    agent.refresh_cache();
                }
            }
            stats.total_steps += 1;
        }
        collision_detector.ignored_pairs.clear();
    }

    // --- spawn random agents ---
    if window.is_key_pressed(Key::R, minifb::KeyRepeat::No) {
        let mut rng = rand::rng();
        let count = rng.random_range(3..=12);
        for _ in 0..count {
            let id = agents.len();
            let start = Node {
                x: rng.random_range(0..COLUMNS) as i32,
                y: rng.random_range(0..ROWS) as i32,
            };
            let end = Node {
                x: rng.random_range(0..COLUMNS) as i32,
                y: rng.random_range(0..ROWS) as i32,
            };
            agents.push(Agent::new(id, start, Some(end)));
        }
    }

    // --- compute / recompute all paths ---
    if window.is_key_pressed(Key::A, minifb::KeyRepeat::No) {
        state.step_history.clear();
        history.history.clear();

        let mut total_len = 0;
        for agent in agents.iter_mut() {
            let Some(goal) = agent.end_point else { continue };
            if let Some(path) = a_star(agent.start_point, goal, &state.walls, state.movement_strategy.as_ref()) {
                total_len += path.len();
                agent.path = Some(path);
                agent.current_point = agent.start_point;
                agent.path_index = 0;
                agent.refresh_cache();
            } else {
                println!("No path found for agent {} — goal may be blocked.", agent.id);
            }
        }
        stats.total_path_length += total_len;
    }

    // --- mouse click: place walls or agents ---
    let is_pressed = window.get_mouse_down(MouseButton::Left);
    if is_pressed && !state.was_pressed {
        if let Some((mx, my)) = window.get_mouse_pos(minifb::MouseMode::Clamp) {
            let cell = Node {
                x: (mx as usize / (WIDTH / COLUMNS)) as i32,
                y: (my as usize / (HEIGHT / ROWS)) as i32,
            };

            match state.current_step {
                Step::Obstacles => {
                    state.walls.insert(cell);
                }
                Step::Start => {
                    if !state.walls.contains(&cell) {
                        let id = agents.len();
                        agents.push(Agent::new(id, cell, None));
                        state.current_step = Step::End;
                    }
                }
                Step::End => {
                    if !state.walls.contains(&cell) {
                        let last = agents.last_mut().unwrap();
                        last.end_point = Some(cell);
                        last.refresh_cache();
                        state.current_step = Step::Start;
                    }
                }
            }
        }
    }
    state.was_pressed = is_pressed;
}

// ---------------------------------------------------------------------------
// Rendering
// ---------------------------------------------------------------------------

fn render(buffer: &mut Vec<u32>, state: &GameState, agents: &[Agent], draw_radius: bool) {
    buffer.fill(WHITE);
    draw_matrix(buffer);

    for node in &state.walls {
        draw(buffer, &DrawType::Square(SquareParams { x: node.ux(), y: node.uy(), color: BLACK }));
    }

    for agent in agents {
        if let Some(path) = &agent.path {
            for w in path.windows(2) {
                let (a, b) = (w[0], w[1]);
                draw(buffer, &DrawType::Line(LineParams {
                    x0: a.x * CELL_HEIGHT as i32 + (CELL_WIDTH / 2) as i32,
                    y0: a.y * CELL_WIDTH  as i32 + (CELL_HEIGHT / 2) as i32,
                    x1: b.x * CELL_HEIGHT as i32 + (CELL_WIDTH / 2) as i32,
                    y1: b.y * CELL_WIDTH  as i32 + (CELL_HEIGHT / 2) as i32,
                    color: BLACK,
                }));
            }
        }

        if let Some(goal) = agent.end_point {
            draw(buffer, &DrawType::Circle(CircleParams {
                x: goal.ux(), y: goal.uy(), radius: 10, color: ORANGE,
            }));
        }

        for &node in &agent.forward_path {
            draw(buffer, &DrawType::Circle(CircleParams {
                x: node.ux(), y: node.uy(), radius: 10, color: LIGHT_BLUE,
            }));
        }

        if draw_radius {
            for &node in &agent.collision_radius {
                draw(buffer, &DrawType::Circle(CircleParams {
                    x: node.ux(), y: node.uy(), radius: 10, color: PALE_RED,
                }));
            }
        }

        draw(buffer, &DrawType::Circle(CircleParams {
            x: agent.current_point.ux(), y: agent.current_point.uy(), radius: 10, color: RED,
        }));
    }
}

// ---------------------------------------------------------------------------
// Main game loop
// ---------------------------------------------------------------------------

fn game_loop(window: &mut Window, buffer: &mut Vec<u32>, state: &mut GameState) {
    let mut stats = Statistics::new();
    let mut history = CommandHistory::new();
    let mut agents: Vec<Agent> = Vec::new();
    let mut last_log = Instant::now();

    let mut detector = CollisionDetector::new();
    let logger    = Rc::new(CollisionLogger);
    let assistant = Rc::new(CollisionAssistant::new());
    detector.register_observer(logger);
    detector.register_observer(assistant.clone());

    while window.is_open() && !window.is_key_down(Key::Escape) {
        handle_input(window, state, &mut agents, &mut history, &mut detector, &mut stats);
        render(buffer, state, &agents, false);

        detector.check_agents(&agents, &mut stats);
        if assistant.has_requests() {
            let requests = assistant.take_requests();
            process_reroute_requests(
                &mut agents, &requests,
                &state.walls, state.movement_strategy.as_ref(), &mut stats,
            );
        }

        if last_log.elapsed() >= Duration::from_secs(1) {
            save_statistics(&stats).unwrap();
            last_log = Instant::now();
        }

        window.update_with_buffer(buffer, WIDTH, HEIGHT).unwrap();
    }
}

// ---------------------------------------------------------------------------
// Entry point
// ---------------------------------------------------------------------------

fn main() {
    let mut handlers: Vec<Box<dyn InitHandler>> = vec![
        Box::new(WindowInitHandler),
        Box::new(BufferInitHandler),
        Box::new(GameStateInitHandler),
    ];

    let mut ctx = InitContext { window: None, buffer: None, game_state: None };

    for handler in handlers.iter_mut() {
        if let Err(e) = handler.initialize(&mut ctx) {
            eprintln!("Initialization failed: {}", e);
            return;
        }
    }

    let mut window    = ctx.window.unwrap();
    let mut buffer    = ctx.buffer.unwrap();
    let mut game_state = ctx.game_state.unwrap();

    game_loop(&mut window, &mut buffer, &mut game_state);
}
