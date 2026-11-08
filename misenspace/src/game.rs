//! Editing, the evening's phases, and what the player hears. Edits apply on the input edge
//! (polled at 250 Hz); the restaurant and all animation advance on 15 FPS ticks.

use crate::geom::*;
use crate::model::*;
use crate::sim::*;
use std::path::PathBuf;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Key {
    Up,
    Down,
    Left,
    Right,
    Act,
    ActUp,
    Rotate,
    Remove,
    Run,
    Restart,
    Undo,
    Fast,
    Tab,
    Tool(usize),
    PrevTool,
    NextTool,
    Clear,
    Help,
    Back,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Sfx {
    Tick,
    Place,
    Rotate,
    Remove,
    Deny,
    Draw,
    Open,
    Slice,
    Bake,
    Plate,
    Arm,
    ChuteUp,
    Arrive,
    Coin,
    Hungry,
    Closed,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Phase {
    /// Doors closed: build, then open.
    Prep,
    Running,
    Paused,
    /// The evening is over; the tally card is up.
    Closed,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Button {
    Run,
    Restart,
    Speed,
}

/// The header buttons (x, y, w, h).
pub const BUTTONS: [(Button, (i32, i32, i32, i32)); 3] =
    [(Button::Run, (244, 6, 48, 20)), (Button::Restart, (296, 6, 56, 20)), (Button::Speed, (356, 6, 38, 20))];

pub fn shelf_rect(i: usize) -> (i32, i32, i32, i32) {
    (10 + i as i32 * 29, 211, 26, 26)
}

/// A floating "+4" over a table when a guest pays, or a sad cloud when one leaves.
#[derive(Clone, Debug)]
pub struct Pop {
    pub x: i32,
    pub y: i32,
    pub coins: u32,
    pub ttl: u8,
}

struct Stroke {
    path: Vec<(i32, i32)>,
    placed: Vec<bool>,
    moved: bool,
    start_was_belt: bool,
}

pub const MSG_INFO: u8 = 0;
pub const MSG_BAD: u8 = 1;
pub const MSG_GOOD: u8 = 2;

pub struct Game {
    pub r: Restaurant,
    pub layout: [Option<KCell>; 64],
    pub sim: Sim,
    pub phase: Phase,
    pub speed: u8,
    sub: u32,
    pub anim: (i32, i32),
    pub tool: usize,
    /// Columns 0..8 are the kitchen, 8..16 the dining room.
    pub cursor: (i32, i32),
    undo: Vec<[Option<KCell>; 64]>,
    stroke: Option<Stroke>,
    act_held: bool,
    mouse_down: bool,
    last_dir: Dir,
    pub message: Option<(String, u8)>,
    message_until: u64,
    pub crt: bool,
    pub ticks: u64,
    pub sfx: Vec<Sfx>,
    pub quit: bool,
    pub show_help: bool,
    pub pops: Vec<Pop>,
    pub best: u32,
    pub nights: u32,
    save_path: Option<PathBuf>,
}

impl Game {
    pub fn new(save_path: Option<PathBuf>) -> Self {
        let r = object25();
        let best = save_path.as_ref().and_then(|p| std::fs::read_to_string(p).ok()).and_then(|t| t.trim().parse().ok()).unwrap_or(0);
        let sim = Sim::new(&r, &r.kitchen);
        let mut g = Self {
            r,
            layout: [None; 64],
            sim,
            phase: Phase::Prep,
            speed: 1,
            sub: 0,
            anim: (1, 1),
            tool: 0,
            cursor: (1, 0),
            undo: Vec::new(),
            stroke: None,
            act_held: false,
            mouse_down: false,
            last_dir: Dir::E,
            message: None,
            message_until: 0,
            crt: true,
            ticks: 0,
            sfx: Vec::new(),
            quit: false,
            show_help: false,
            pops: Vec::new(),
            best,
            nights: 0,
            save_path,
        };
        g.say("PREP: BUILD THE KITCHEN, THEN PRESS ENTER TO OPEN THE DOORS", MSG_INFO);
        g.message_until = u64::MAX;
        g
    }

    pub fn kitchen(&self) -> [KCell; 64] {
        let mut k = self.r.kitchen;
        for i in 0..64 {
            if let Some(c) = self.layout[i] {
                k[i] = c;
            }
        }
        k
    }

    pub fn say(&mut self, text: &str, kind: u8) {
        self.message = Some((text.to_string(), kind));
        self.message_until = self.ticks + if kind == MSG_INFO { 60 } else { 75 };
    }

    fn deny(&mut self, text: &str) {
        self.say(text, MSG_BAD);
        self.sfx.push(Sfx::Deny);
    }

    pub fn cursor_cell(&self) -> (Board, i32, i32) {
        let (cx, cy) = self.cursor;
        if cx < 8 {
            (Board::Kitchen, cx, cy)
        } else {
            (Board::Dining, cx - 8, cy)
        }
    }

    pub fn shelf_left(&self, slot: usize) -> Option<u8> {
        let (tool, count) = self.r.shelf[slot];
        count.map(|n| n.saturating_sub(self.layout.iter().filter(|c| c.and_then(Tool::of) == Some(tool)).count() as u8))
    }

    pub fn is_fixed(&self, x: i32, y: i32) -> bool {
        self.r.kitchen[idx(x, y)] != KCell::Floor
    }

    // -----------------------------------------------------------------------------------------
    // Input

    pub fn key(&mut self, k: Key) {
        if k == Key::Help {
            self.show_help = !self.show_help;
            return;
        }
        if self.show_help {
            if matches!(k, Key::Back | Key::Run) {
                self.show_help = false;
            }
            return;
        }
        match k {
            Key::Up | Key::Down | Key::Left | Key::Right => {
                let (dx, dy) = match k {
                    Key::Up => (0, -1),
                    Key::Down => (0, 1),
                    Key::Left => (-1, 0),
                    _ => (1, 0),
                };
                let next = ((self.cursor.0 + dx).clamp(0, 15), (self.cursor.1 + dy).clamp(0, 7));
                if next != self.cursor {
                    self.cursor = next;
                    self.sfx.push(Sfx::Tick);
                    if self.act_held && self.stroke.is_some() {
                        match self.cursor_cell() {
                            (Board::Kitchen, x, y) => self.stroke_to(x, y),
                            _ => self.stroke = None,
                        }
                    }
                }
            }
            Key::Tab => {
                self.cursor.0 = (self.cursor.0 + 8) % 16;
                self.sfx.push(Sfx::Tick);
            }
            Key::Tool(i) => self.pick_tool(i),
            Key::PrevTool => {
                let n = self.r.shelf.len();
                self.pick_tool((self.tool + n - 1) % n);
            }
            Key::NextTool => {
                let n = self.r.shelf.len();
                self.pick_tool((self.tool + 1) % n);
            }
            Key::Fast => {
                self.speed = match self.speed {
                    1 => 2,
                    2 => 4,
                    _ => 1,
                };
                self.sfx.push(Sfx::Tick);
                let t = format!("SPEED {}X", self.speed);
                self.say(&t, MSG_INFO);
            }
            Key::Run => self.run_button(),
            Key::Restart => self.restart(),
            Key::Back => {
                if self.phase == Phase::Running {
                    self.phase = Phase::Paused;
                } else {
                    self.quit = true;
                }
            }
            Key::ActUp => {
                self.act_held = false;
                self.end_stroke();
            }
            Key::Act => {
                self.act_held = true;
                if self.phase == Phase::Closed {
                    return;
                }
                let (b, x, y) = self.cursor_cell();
                self.act_at(b, x, y);
            }
            Key::Rotate => {
                let (b, x, y) = self.cursor_cell();
                if b == Board::Kitchen {
                    self.rotate_at(x, y);
                }
            }
            Key::Remove => {
                let (b, x, y) = self.cursor_cell();
                if b == Board::Kitchen {
                    self.remove_at(x, y);
                }
            }
            Key::Clear => {
                if self.layout.iter().all(|c| c.is_none()) {
                    return self.deny("NOTHING TO CLEAR");
                }
                self.push_undo();
                self.layout = [None; 64];
                self.rebuilt();
                self.sfx.push(Sfx::Remove);
                self.say("CLEARED - BACKSPACE BRINGS IT BACK", MSG_INFO);
            }
            Key::Undo => {
                if let Some(l) = self.undo.pop() {
                    self.layout = l;
                    self.rebuilt();
                    self.sfx.push(Sfx::Remove);
                } else {
                    self.deny("NOTHING TO UNDO");
                }
            }
            Key::Help => {}
        }
    }

    fn run_button(&mut self) {
        match self.phase {
            Phase::Prep => {
                self.phase = Phase::Running;
                self.sub = 0;
                self.sfx.push(Sfx::Open);
                self.say("DOORS OPEN", MSG_GOOD);
            }
            Phase::Running => {
                self.phase = Phase::Paused;
                self.say("PAUSED - KEEP BUILDING IF YOU LIKE", MSG_INFO);
            }
            Phase::Paused => {
                self.phase = Phase::Running;
                self.sub = 0;
            }
            Phase::Closed => self.restart(),
        }
    }

    /// Back to prep with a fresh evening; the kitchen stays as built.
    pub fn restart(&mut self) {
        self.sim = Sim::new(&self.r, &self.kitchen());
        self.phase = Phase::Prep;
        self.pops.clear();
        self.anim = (1, 1);
        self.sfx.push(Sfx::Remove);
        self.say("A FRESH EVENING - YOUR KITCHEN STAYS. ENTER TO OPEN", MSG_INFO);
    }

    fn pick_tool(&mut self, i: usize) {
        if i < self.r.shelf.len() {
            self.tool = i;
            self.sfx.push(Sfx::Tick);
        }
    }

    pub fn mouse_cell(px: i32, py: i32) -> Option<(Board, i32, i32)> {
        for b in [Board::Kitchen, Board::Dining] {
            let bx = board_x(b);
            if px >= bx && px < bx + BOARD_PX && py >= BOARD_Y && py < BOARD_Y + BOARD_PX {
                return Some((b, (px - bx) / TILE, (py - BOARD_Y) / TILE));
            }
        }
        None
    }

    pub fn mouse_move(&mut self, px: i32, py: i32) {
        let Some((b, x, y)) = Self::mouse_cell(px, py) else { return };
        let cx = x + if b == Board::Dining { 8 } else { 0 };
        if (cx, y) == self.cursor {
            return;
        }
        if self.mouse_down && self.stroke.is_some() && b == Board::Kitchen {
            let mut at = self.cursor;
            while at != (cx, y) {
                if at.0 != cx {
                    at.0 += (cx - at.0).signum();
                } else {
                    at.1 += (y - at.1).signum();
                }
                self.cursor = at;
                if at.0 < 8 {
                    self.stroke_to(at.0, at.1);
                }
            }
        } else {
            self.cursor = (cx, y);
        }
    }

    pub fn mouse_button(&mut self, left: bool, down: bool, px: i32, py: i32) {
        if self.show_help {
            if down {
                self.show_help = false;
            }
            return;
        }
        if !down {
            if left {
                self.mouse_down = false;
                self.end_stroke();
            }
            return;
        }
        if left {
            let inside_rect = |(x, y, w, h): (i32, i32, i32, i32)| px >= x && px < x + w && py >= y && py < y + h;
            if let Some(&(b, _)) = BUTTONS.iter().find(|(_, r)| inside_rect(*r)) {
                return self.key(match b {
                    Button::Run => Key::Run,
                    Button::Restart => Key::Restart,
                    Button::Speed => Key::Fast,
                });
            }
            if let Some(i) = (0..self.r.shelf.len()).find(|&i| inside_rect(shelf_rect(i))) {
                return self.pick_tool(i);
            }
        }
        let Some((b, x, y)) = Self::mouse_cell(px, py) else { return };
        self.cursor = (x + if b == Board::Dining { 8 } else { 0 }, y);
        if self.phase == Phase::Closed || b == Board::Dining {
            return;
        }
        if left {
            self.mouse_down = true;
            self.act_at(b, x, y);
        } else {
            self.remove_at(x, y);
        }
    }

    pub fn wheel(&mut self, clicks: i32) {
        if let (Board::Kitchen, x, y) = self.cursor_cell() {
            for _ in 0..clicks.unsigned_abs().min(3) {
                self.rotate_at(x, y);
            }
        }
    }

    // -----------------------------------------------------------------------------------------
    // Building

    fn push_undo(&mut self) {
        self.undo.push(self.layout);
        if self.undo.len() > 300 {
            self.undo.remove(0);
        }
    }

    /// The kitchen changed: the live restaurant adopts it immediately.
    fn rebuilt(&mut self) {
        let k = self.kitchen();
        self.sim.rebuild(&k);
    }

    fn act_at(&mut self, b: Board, x: i32, y: i32) {
        if b == Board::Dining {
            return self.deny("THE DINING ROOM IS SET. BUILD IN THE KITCHEN (LEFT)");
        }
        let tool = self.r.shelf[self.tool].0;
        if tool == Tool::Belt {
            return self.begin_stroke(x, y);
        }
        self.place(x, y, tool);
    }

    fn begin_stroke(&mut self, x: i32, y: i32) {
        let placed = self.layout[idx(x, y)];
        let free = placed.is_none() && !self.is_fixed(x, y);
        self.push_undo();
        if free {
            self.layout[idx(x, y)] = Some(KCell::Belt(self.last_dir));
            self.sfx.push(Sfx::Draw);
            self.rebuilt();
        }
        self.stroke = Some(Stroke { path: vec![(x, y)], placed: vec![free], moved: false, start_was_belt: matches!(placed, Some(KCell::Belt(_))) });
    }

    fn stroke_to(&mut self, x: i32, y: i32) {
        let Some(st) = self.stroke.as_mut() else { return };
        let prev = *st.path.last().unwrap();
        let Some(d) = Dir::between(prev, (x, y)) else { return };
        st.moved = true;
        if st.path.len() >= 2 && st.path[st.path.len() - 2] == (x, y) {
            if *st.placed.last().unwrap() {
                self.layout[idx(prev.0, prev.1)] = None;
            }
            st.path.pop();
            st.placed.pop();
            self.sfx.push(Sfx::Remove);
            self.rebuilt();
            return;
        }
        if let Some(KCell::Belt(_)) = self.layout[idx(prev.0, prev.1)] {
            self.layout[idx(prev.0, prev.1)] = Some(KCell::Belt(d));
        }
        let free = self.layout[idx(x, y)].is_none() && !self.is_fixed(x, y);
        if free {
            self.layout[idx(x, y)] = Some(KCell::Belt(d));
        }
        let st = self.stroke.as_mut().unwrap();
        st.path.push((x, y));
        st.placed.push(free);
        self.last_dir = d;
        self.sfx.push(Sfx::Draw);
        self.rebuilt();
    }

    fn end_stroke(&mut self) {
        if let Some(st) = self.stroke.take() {
            if !st.moved && st.start_was_belt {
                let (x, y) = st.path[0];
                if let Some(KCell::Belt(d)) = self.layout[idx(x, y)] {
                    self.layout[idx(x, y)] = Some(KCell::Belt(d.cw()));
                    self.last_dir = d.cw();
                    self.sfx.push(Sfx::Rotate);
                    self.rebuilt();
                }
            }
        }
    }

    fn place(&mut self, x: i32, y: i32, tool: Tool) {
        if self.is_fixed(x, y) {
            return self.deny("THAT'S PART OF THE STATION");
        }
        if tool == Tool::Chute && !self.r.table_at(x, y) {
            return self.deny("A CHUTE ONLY HELPS UNDER A TABLE - SEE THE DINING ROOM");
        }
        let existing = self.layout[idx(x, y)];
        if existing.and_then(Tool::of) == Some(tool) {
            return self.rotate_at(x, y);
        }
        if self.shelf_left(self.tool) == Some(0) {
            return self.deny("NONE LEFT ON THE SHELF");
        }
        let d = match existing {
            Some(KCell::Belt(d)) => d,
            _ => self.last_dir,
        };
        self.push_undo();
        self.layout[idx(x, y)] = Some(tool.cell(d));
        self.sfx.push(Sfx::Place);
        self.rebuilt();
    }

    fn rotate_at(&mut self, x: i32, y: i32) {
        let Some(c) = self.layout[idx(x, y)] else {
            return self.deny(if self.is_fixed(x, y) { "THAT'S PART OF THE STATION" } else { "NOTHING TO TURN" });
        };
        let turned = match c {
            KCell::Belt(d) => KCell::Belt(d.cw()),
            KCell::Arm(d) => KCell::Arm(d.cw()),
            KCell::Machine(m, d) => KCell::Machine(m, d.cw()),
            KCell::Counter(i, d) => KCell::Counter(i, d.cw()),
            _ => return self.deny("CHUTES DON'T TURN"),
        };
        self.push_undo();
        self.layout[idx(x, y)] = Some(turned);
        if let KCell::Belt(d) | KCell::Arm(d) | KCell::Machine(_, d) | KCell::Counter(_, d) = turned {
            self.last_dir = d;
        }
        self.sfx.push(Sfx::Rotate);
        self.rebuilt();
    }

    fn remove_at(&mut self, x: i32, y: i32) {
        if self.layout[idx(x, y)].is_none() {
            if self.is_fixed(x, y) {
                self.deny("THAT'S PART OF THE STATION");
            }
            return;
        }
        self.push_undo();
        self.layout[idx(x, y)] = None;
        self.sfx.push(Sfx::Remove);
        self.rebuilt();
    }

    // -----------------------------------------------------------------------------------------
    // The evening

    fn beat_ticks(&self) -> u32 {
        (4 / self.speed.max(1) as u32).max(1)
    }

    pub fn update(&mut self) {
        self.ticks += 1;
        if self.ticks > self.message_until {
            self.message = None;
        }
        for p in self.pops.iter_mut() {
            p.ttl = p.ttl.saturating_sub(1);
        }
        self.pops.retain(|p| p.ttl > 0);
        if self.phase != Phase::Running {
            self.anim = (1, 1);
            return;
        }
        let b = self.beat_ticks();
        if self.sub == 0 {
            self.sim.step();
            let evs: Vec<Ev> = self.sim.events.drain(..).collect();
            self.play(&evs);
            for g in self.sim.guests.iter() {
                if g.state == GState::Eating && g.timer == self.r.service.eat_beats {
                    let (x, y) = g.to;
                    self.pops.push(Pop { x, y: y - 14, coins: g.paid, ttl: 24 });
                }
            }
            if self.sim.over() {
                self.phase = Phase::Closed;
                self.nights += 1;
                let stars = self.sim.stars();
                if stars > self.best {
                    self.best = stars;
                    if let Some(p) = &self.save_path {
                        let _ = std::fs::write(p, stars.to_string());
                    }
                }
                self.sfx.push(Sfx::Closed);
                self.anim = (1, 1);
                return;
            }
        }
        self.anim = (self.sub as i32 + 1, b as i32);
        self.sub = (self.sub + 1) % b;
    }

    fn play(&mut self, evs: &[Ev]) {
        let mut seen: Vec<Sfx> = Vec::new();
        for &e in evs {
            let s = match e {
                Ev::Hatch => continue,
                Ev::Slice => Sfx::Slice,
                Ev::Bake => Sfx::Bake,
                Ev::Plate => Sfx::Plate,
                Ev::Arm => Sfx::Arm,
                Ev::ChuteUp => Sfx::ChuteUp,
                Ev::Arrive => Sfx::Arrive,
                Ev::Served(_) => Sfx::Coin,
                Ev::Hungry => Sfx::Hungry,
            };
            if !seen.contains(&s) {
                seen.push(s);
                self.sfx.push(s);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn building_the_reference_by_placement_and_serving_an_evening() {
        let mut g = Game::new(None);
        let k = crate::sim::lab::reference_kitchen(&g.r);
        for i in 0..64 {
            if k[i] != g.r.kitchen[i] {
                g.layout[i] = Some(k[i]);
            }
        }
        g.restart();
        g.key(Key::Run);
        g.speed = 4;
        for _ in 0..20000 {
            g.update();
            if g.phase == Phase::Closed {
                break;
            }
        }
        assert_eq!(g.phase, Phase::Closed);
        assert_eq!(g.sim.stars(), 3);
    }

    #[test]
    fn belts_draw_by_holding_act_and_chutes_only_go_under_tables() {
        let mut g = Game::new(None);
        g.cursor = (1, 1);
        g.key(Key::Act);
        g.key(Key::Right);
        g.key(Key::Right);
        g.key(Key::ActUp);
        assert_eq!(g.layout[idx(3, 1)], Some(KCell::Belt(Dir::E)));
        let chute = g.r.shelf.iter().position(|s| s.0 == Tool::Chute).unwrap();
        g.key(Key::Tool(chute));
        g.cursor = (5, 5);
        g.key(Key::Act);
        g.key(Key::ActUp);
        assert_eq!(g.layout[idx(5, 5)], None, "no table above F6");
        g.cursor = (6, 1);
        g.key(Key::Act);
        g.key(Key::ActUp);
        assert_eq!(g.layout[idx(6, 1)], Some(KCell::Chute), "G2 has a table");
    }

    #[test]
    fn building_during_service_changes_the_live_kitchen() {
        let mut g = Game::new(None);
        g.key(Key::Run);
        for _ in 0..40 {
            g.update();
        }
        g.cursor = (1, 0);
        g.key(Key::Act);
        g.key(Key::ActUp);
        assert!(matches!(g.sim.cells[idx(1, 0)], KCell::Belt(_)));
    }
}
