//! Screens, editing, and the run state machine. Edits apply on the input edge (polled at
//! 250 Hz); the simulation and all animation advance on 15 FPS ticks.

use crate::geom::*;
use crate::levels;
use crate::model::*;
use crate::sim::*;
use std::io::Write;
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
    Flip,
    Remove,
    Run,
    Undo,
    Fast,
    /// Cycles hints: off, key parts ghosted, whole solution ghosted.
    Hint,
    Clear,
    Help,
    Back,
    Tab,
    Tool(usize),
    PrevTool,
    NextTool,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Sfx {
    Tick,
    Place,
    Rotate,
    Remove,
    Deny,
    Draw,
    RunStart,
    Emit,
    Paint,
    Switch,
    Clink(u8),
    Lock,
    Release,
    Note(u8),
    Bad,
    Won,
    Turn,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Screen {
    Select,
    Play,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Phase {
    Edit,
    Make,
    Lock(u32),
    Deliver,
    Jammed(Jam, i32, i32),
    Failed(Fail, i32, i32),
    Solved,
}

impl Phase {
    pub fn running(self) -> bool {
        matches!(self, Phase::Make | Phase::Lock(_) | Phase::Deliver)
    }
}

struct Stroke {
    path: Vec<(i32, i32)>,
    placed: Vec<bool>,
    moved: bool,
    start_was_belt: bool,
}

pub struct Game {
    pub levels: Vec<Level>,
    pub solved: Vec<bool>,
    pub screen: Screen,
    pub sel: usize,
    pub cur: usize,
    pub layout: Layout,
    pub tool: usize,
    /// Cursor column 0..16 spans both boards (8..16 is the right board); row 0..8.
    pub cursor: (i32, i32),
    undo: Vec<Layout>,
    pub phase: Phase,
    pub make: Option<MakeSim>,
    pub deliver: Option<DeliverSim>,
    sub: u32,
    /// Animation progress within the current beat: (num, den).
    pub anim: (i32, i32),
    pub fast: bool,
    /// Chute contents from the last finished Make, and the left board that made them.
    pub made: Option<(Vec<Vec<Color>>, [Option<Cell>; 64])>,
    pub trails: Vec<Vec<(i32, i32)>>,
    /// 0 none, 1 key parts of the stored solution ghosted, 2 the whole solution ghosted.
    pub hint: u8,
    pub show_help: bool,
    /// Work left in each room, so leaving and coming back keeps it.
    layouts: Vec<Layout>,
    log_path: Option<PathBuf>,
    room_started: u64,
    runs: u32,
    hints_used: u8,
    stroke: Option<Stroke>,
    act_held: bool,
    mouse_down: bool,
    last_dir: Dir,
    last_arrow: Dir,
    pub message: Option<(String, u8)>,
    pub crt: bool,
    pub ticks: u64,
    pub hall_pal: (u8, u8, u8),
    pub sfx: Vec<Sfx>,
    pub quit: bool,
    /// Mouse position in logical pixels; hidden again when the keyboard moves the cursor.
    pub pointer: Option<(i32, i32)>,
    save_path: Option<PathBuf>,
}

pub const MSG_INFO: u8 = 0;
pub const MSG_BAD: u8 = 1;
pub const MSG_GOOD: u8 = 2;

/// Tray slot rectangles on the shelf (x, y, w, h).
pub fn tray_slot_rect(i: usize) -> (i32, i32, i32, i32) {
    (16 + i as i32 * 34, 216, 28, 28)
}

impl Game {
    /// (hall index, position in hall, rooms in hall) for room `i`.
    fn room_pos(&self, i: usize) -> (usize, usize, usize) {
        let hall = self.levels[i].hall;
        let k = self.levels[..i].iter().filter(|l| l.hall == hall).count();
        let n = self.levels.iter().filter(|l| l.hall == hall).count();
        ((hall as usize).saturating_sub(1), k, n)
    }

    /// Room windows on the select screen (x, y, w, h), centered per hall.
    pub fn room_rect(&self, i: usize) -> (i32, i32, i32, i32) {
        let (hall, k, n) = self.room_pos(i);
        let x0 = 200 - (n as i32 * 40 - 8) / 2;
        (x0 + k as i32 * 40, 92 + hall as i32 * 84, 32, 40)
    }

    pub fn new(save_path: Option<PathBuf>, log_path: Option<PathBuf>) -> Result<Self, String> {
        let levels = levels::all()?;
        let mut solved = vec![false; levels.len()];
        if let Some(text) = save_path.as_ref().and_then(|p| std::fs::read_to_string(p).ok()) {
            for id in text.split_whitespace() {
                if let Some(i) = levels.iter().position(|l| l.id == id) {
                    solved[i] = true;
                }
            }
        }
        let sel = solved.iter().position(|s| !s).unwrap_or(0);
        let layouts = vec![Layout::default(); levels.len()];
        Ok(Self {
            layouts,
            log_path,
            room_started: 0,
            runs: 0,
            hints_used: 0,
            hint: 0,
            levels,
            solved,
            screen: Screen::Select,
            sel,
            cur: 0,
            layout: Layout::default(),
            tool: 0,
            cursor: (1, 3),
            undo: Vec::new(),
            phase: Phase::Edit,
            make: None,
            deliver: None,
            sub: 0,
            anim: (1, 1),
            fast: false,
            made: None,
            trails: Vec::new(),
            show_help: false,
            stroke: None,
            act_held: false,
            mouse_down: false,
            last_dir: Dir::E,
            last_arrow: Dir::E,
            message: None,
            crt: true,
            ticks: 0,
            hall_pal: (1, 1, 16),
            sfx: Vec::new(),
            quit: false,
            pointer: None,
            save_path,
        })
    }

    pub fn level(&self) -> &Level {
        &self.levels[self.cur]
    }

    pub fn hall(&self) -> u8 {
        match self.screen {
            Screen::Select => (self.sel / 8 + 1) as u8,
            Screen::Play => self.level().hall,
        }
    }

    fn set_palette(&mut self, hall: u8) {
        let current = if self.hall_pal.2 >= 8 { self.hall_pal.1 } else { self.hall_pal.0 };
        if current != hall {
            self.hall_pal = (current, hall, 0);
        }
    }

    pub fn open_level(&mut self, i: usize) {
        self.leave_room();
        self.cur = i;
        self.screen = Screen::Play;
        self.layout = self.layouts[i];
        self.undo.clear();
        self.tool = 0;
        self.cursor = (1, self.levels[i].crates[0].y);
        self.phase = Phase::Edit;
        self.make = None;
        self.deliver = None;
        self.made = None;
        self.trails.clear();
        self.hint = 0;
        self.stroke = None;
        self.message = None;
        self.room_started = self.ticks;
        self.runs = 0;
        self.hints_used = 0;
        let hall = self.levels[i].hall;
        self.set_palette(hall);
        let name = self.levels[i].name.to_uppercase();
        self.log(&format!("{} {name}: opened{}", self.levels[i].id, if self.solved[i] { " (solved before)" } else { "" }));
    }

    /// Keeps the room's work and notes how the visit went.
    fn leave_room(&mut self) {
        if self.screen != Screen::Play {
            return;
        }
        self.layouts[self.cur] = self.layout;
        let secs = (self.ticks - self.room_started) / 15;
        let state = if self.phase == Phase::Solved { "solved" } else if self.solved[self.cur] { "solved earlier" } else { "unsolved" };
        let line = format!("{} left after {}m{:02}s, {} runs, {state}, hints {}", self.level().id, secs / 60, secs % 60, self.runs, self.hints_used);
        self.log(&line);
    }

    pub fn end_session(&mut self) {
        self.leave_room();
        let done = self.solved.iter().filter(|s| **s).count();
        self.log(&format!("session end: {done} of {} rooms solved", self.levels.len()));
    }

    pub fn log(&self, line: &str) {
        if let Some(p) = &self.log_path {
            if let Ok(mut f) = std::fs::OpenOptions::new().create(true).append(true).open(p) {
                let t = self.ticks / 15;
                let _ = writeln!(f, "[{:02}:{:02}] {line}", t / 60, t % 60);
            }
        }
    }

    fn save(&self) {
        if let Some(p) = &self.save_path {
            let ids: Vec<&str> = self.levels.iter().zip(&self.solved).filter(|(_, s)| **s).map(|(l, _)| l.id.as_str()).collect();
            let _ = std::fs::write(p, ids.join("\n"));
        }
    }

    fn say(&mut self, text: &str, kind: u8) {
        self.message = Some((text.to_string(), kind));
    }

    fn deny(&mut self, text: &str) {
        self.say(text, MSG_BAD);
        self.sfx.push(Sfx::Deny);
    }

    pub fn cursor_cell(&self) -> (Board, i32, i32) {
        let (cx, cy) = self.cursor;
        if cx < 8 {
            (Board::Left, cx, cy)
        } else {
            (Board::Right, cx - 8, cy)
        }
    }

    pub fn tray_left(&self, slot: usize) -> Option<u8> {
        let s = self.level().tray[slot];
        s.count.map(|n| n.saturating_sub(self.layout.placed(s.tool)))
    }

    /// The cell as the player sees it right now (switch levers follow the run).
    pub fn cell(&self, b: Board, x: i32, y: i32) -> Cell {
        let c = self.layout.get(b, x, y).unwrap_or_else(|| self.level().fixed(b, x, y));
        match (c, b) {
            (Cell::Switch(_), Board::Left) => self.make.as_ref().map_or(c, |m| Cell::Switch(m.lever[idx(x, y)])),
            (Cell::Switch(_), Board::Right) => self.deliver.as_ref().map_or(c, |d| Cell::Switch(d.lever[idx(x, y)])),
            _ => c,
        }
    }

    pub fn is_fixed(&self, b: Board, x: i32, y: i32) -> bool {
        self.layout.get(b, x, y).is_none() && self.level().fixed(b, x, y) != Cell::Floor
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
        match self.screen {
            Screen::Select => self.select_key(k),
            Screen::Play => self.play_key(k),
        }
    }

    fn select_key(&mut self, k: Key) {
        let n = self.levels.len();
        match k {
            Key::Left => self.sel = (self.sel + n - 1) % n,
            Key::Right => self.sel = (self.sel + 1) % n,
            Key::Up | Key::Down => {
                // Same position in the other hall.
                let (hall, k, _) = self.room_pos(self.sel);
                let other: Vec<usize> = (0..n).filter(|&i| self.room_pos(i).0 != hall).collect();
                if let Some(&i) = other.get(k.min(other.len().saturating_sub(1))) {
                    self.sel = i;
                }
            }
            Key::Run | Key::Act => {
                self.sfx.push(Sfx::Place);
                self.open_level(self.sel);
                return;
            }
            Key::Back => {
                self.quit = true;
                return;
            }
            _ => return,
        }
        self.sfx.push(Sfx::Tick);
        let hall = (self.sel / 8 + 1) as u8;
        self.set_palette(hall);
    }

    fn play_key(&mut self, k: Key) {
        match k {
            Key::Up | Key::Down | Key::Left | Key::Right => {
                let (dx, dy) = match k {
                    Key::Up => (0, -1),
                    Key::Down => (0, 1),
                    Key::Left => (-1, 0),
                    _ => (1, 0),
                };
                let (cx, cy) = self.cursor;
                let next = ((cx + dx).clamp(0, 15), (cy + dy).clamp(0, 7));
                self.pointer = None;
                if next != self.cursor {
                    self.cursor = next;
                    self.sfx.push(Sfx::Tick);
                    if self.act_held && self.stroke.is_some() {
                        let (b, x, y) = self.cursor_cell();
                        if b == Board::Left {
                            self.stroke_to(x, y);
                        } else {
                            self.stroke = None;
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
                let n = self.level().tray.len();
                self.pick_tool((self.tool + n - 1) % n);
            }
            Key::NextTool => {
                let n = self.level().tray.len();
                self.pick_tool((self.tool + 1) % n);
            }
            Key::Hint => {
                self.hint = (self.hint + 1) % 3;
                self.hints_used = self.hints_used.max(self.hint);
                let text = match self.hint {
                    1 => "HINT: THE KEY PARTS ARE GHOSTED IN",
                    2 => "HINT: THE WHOLE SOLUTION IS GHOSTED IN",
                    _ => "HINTS OFF",
                };
                self.say(text, MSG_INFO);
                self.sfx.push(Sfx::Tick);
                if self.hint > 0 {
                    let line = format!("{} hint {} after {}s", self.level().id, self.hint, (self.ticks - self.room_started) / 15);
                    self.log(&line);
                }
            }
            Key::Fast => self.fast = !self.fast,
            Key::Back => {
                if self.phase.running() {
                    self.stop_run();
                } else {
                    self.leave_room();
                    self.screen = Screen::Select;
                    self.sel = self.cur;
                    self.stroke = None;
                }
            }
            Key::Run => match self.phase {
                Phase::Solved => {
                    let next = self.cur + 1;
                    if next < self.levels.len() {
                        self.open_level(next);
                    } else {
                        self.screen = Screen::Select;
                    }
                }
                p if p.running() => self.stop_run(),
                _ => self.start_run(),
            },
            Key::ActUp => {
                self.act_held = false;
                self.end_stroke();
            }
            _ if !self.editable() => {}
            Key::Act => {
                self.act_held = true;
                let (b, x, y) = self.cursor_cell();
                self.act_at(b, x, y);
            }
            Key::Rotate => {
                let (b, x, y) = self.cursor_cell();
                self.rotate_at(b, x, y, false);
            }
            Key::Flip => {
                let (b, x, y) = self.cursor_cell();
                self.rotate_at(b, x, y, true);
            }
            Key::Remove => {
                let (b, x, y) = self.cursor_cell();
                self.remove_at(b, x, y);
            }
            Key::Clear => {
                if self.layout == Layout::default() {
                    return self.deny("NOTHING TO CLEAR");
                }
                self.push_undo();
                self.layout = Layout::default();
                self.sfx.push(Sfx::Remove);
                self.say("CLEARED - BACKSPACE BRINGS IT BACK", MSG_INFO);
            }
            Key::Undo => {
                if let Some(l) = self.undo.pop() {
                    self.to_edit();
                    self.layout = l;
                    self.sfx.push(Sfx::Remove);
                    self.say("UNDONE", MSG_INFO);
                } else {
                    self.deny("NOTHING TO UNDO");
                }
            }
            Key::Help => {}
        }
    }

    /// Edits are allowed while editing and after a run stopped; they end the stop screen.
    fn editable(&self) -> bool {
        matches!(self.phase, Phase::Edit | Phase::Jammed(..) | Phase::Failed(..))
    }

    fn to_edit(&mut self) {
        if !matches!(self.phase, Phase::Edit) {
            self.phase = Phase::Edit;
            self.make = None;
            self.deliver = None;
        }
        self.trails.clear();
        self.message = None;
    }

    fn pick_tool(&mut self, i: usize) {
        if i < self.level().tray.len() {
            self.tool = i;
            self.sfx.push(Sfx::Tick);
            let name = self.level().tray[i].tool.name();
            self.say(name, MSG_INFO);
        }
    }

    pub fn mouse_cell(&self, px: i32, py: i32) -> Option<(Board, i32, i32)> {
        for b in [Board::Left, Board::Right] {
            let bx = board_x(b);
            if px >= bx && px < bx + BOARD_PX && py >= BOARD_Y && py < BOARD_Y + BOARD_PX {
                return Some((b, (px - bx) / TILE, (py - BOARD_Y) / TILE));
            }
        }
        None
    }

    pub fn mouse_move(&mut self, px: i32, py: i32) {
        match self.screen {
            Screen::Select => {
                if let Some(i) = (0..self.levels.len()).find(|&i| in_rect(self.room_rect(i), px, py)) {
                    if i != self.sel {
                        self.sel = i;
                        self.sfx.push(Sfx::Tick);
                        self.set_palette((i / 8 + 1) as u8);
                    }
                }
            }
            Screen::Play => {
                let Some((b, x, y)) = self.mouse_cell(px, py) else { return };
                let cx = x + if b == Board::Right { 8 } else { 0 };
                if (cx, y) == self.cursor {
                    return;
                }
                if self.mouse_down && self.stroke.is_some() && b == Board::Left {
                    // Walk one tile at a time so fast drags still lay a connected belt.
                    let mut at = self.cursor;
                    while at != (cx, y) {
                        if at.0 != cx {
                            at.0 += (cx - at.0).signum();
                        } else {
                            at.1 += (y - at.1).signum();
                        }
                        self.cursor = at;
                        self.stroke_to(at.0, at.1);
                    }
                } else {
                    self.cursor = (cx, y);
                }
            }
        }
    }

    pub fn mouse_button(&mut self, left: bool, down: bool, px: i32, py: i32) {
        if self.show_help {
            if down {
                self.show_help = false;
            }
            return;
        }
        match self.screen {
            Screen::Select => {
                if down && left {
                    if let Some(i) = (0..self.levels.len()).find(|&i| in_rect(self.room_rect(i), px, py)) {
                        self.sel = i;
                        self.sfx.push(Sfx::Place);
                        self.open_level(i);
                    }
                }
            }
            Screen::Play => {
                if !down {
                    if left {
                        self.mouse_down = false;
                        self.end_stroke();
                    }
                    return;
                }
                if left {
                    if let Some(i) = (0..self.level().tray.len()).find(|&i| in_rect(tray_slot_rect(i), px, py)) {
                        return self.pick_tool(i);
                    }
                }
                let Some((b, x, y)) = self.mouse_cell(px, py) else { return };
                self.cursor = (x + if b == Board::Right { 8 } else { 0 }, y);
                if !self.editable() {
                    return;
                }
                if left {
                    self.mouse_down = true;
                    self.act_at(b, x, y);
                } else {
                    self.remove_at(b, x, y);
                }
            }
        }
    }

    pub fn wheel(&mut self, clicks: i32) {
        if self.screen == Screen::Play && self.editable() && clicks != 0 {
            let (b, x, y) = self.cursor_cell();
            for _ in 0..clicks.unsigned_abs().min(3) {
                self.rotate_at(b, x, y, false);
            }
        }
    }

    // -----------------------------------------------------------------------------------------
    // Editing

    fn push_undo(&mut self) {
        self.undo.push(self.layout);
        if self.undo.len() > 300 {
            self.undo.remove(0);
        }
        self.to_edit();
    }

    fn act_at(&mut self, b: Board, x: i32, y: i32) {
        let mut tool = self.level().tray[self.tool].tool;
        if !tool.allowed(b) {
            // Pick up something that fits this board instead of refusing: the part already
            // there (so Space turns it), or the first part on the shelf that fits.
            let here = self.layout.get(b, x, y).and_then(Tool::of);
            let fits = (0..self.level().tray.len())
                .find(|&i| Some(self.level().tray[i].tool) == here)
                .or_else(|| (0..self.level().tray.len()).find(|&i| self.level().tray[i].tool.allowed(b) && self.tray_left(i) != Some(0)));
            match fits {
                Some(i) => {
                    self.tool = i;
                    tool = self.level().tray[i].tool;
                }
                None => {
                    return self.deny(if b == Board::Right { "NOTHING ON THE SHELF GOES ON THE FLOOR" } else { "NOTHING ON THE SHELF GOES HERE" });
                }
            }
        }
        if tool == Tool::Belt {
            return self.begin_stroke(x, y);
        }
        self.place(b, x, y, tool);
    }

    fn begin_stroke(&mut self, x: i32, y: i32) {
        let placed = self.layout.get(Board::Left, x, y);
        let free = placed.is_none() && self.level().fixed(Board::Left, x, y) == Cell::Floor;
        self.push_undo();
        if free {
            self.layout.set(Board::Left, x, y, Some(Cell::Belt(self.last_dir)));
            self.sfx.push(Sfx::Draw);
        }
        self.stroke = Some(Stroke { path: vec![(x, y)], placed: vec![free], moved: false, start_was_belt: matches!(placed, Some(Cell::Belt(_))) });
    }

    fn stroke_to(&mut self, x: i32, y: i32) {
        let Some(st) = self.stroke.as_mut() else { return };
        let prev = *st.path.last().unwrap();
        let Some(d) = Dir::between(prev, (x, y)) else { return };
        st.moved = true;
        if st.path.len() >= 2 && st.path[st.path.len() - 2] == (x, y) {
            // Backtracking erases what this stroke just laid.
            if *st.placed.last().unwrap() {
                self.layout.set(Board::Left, prev.0, prev.1, None);
            }
            st.path.pop();
            st.placed.pop();
            self.sfx.push(Sfx::Remove);
            return;
        }
        if let Some(Cell::Belt(_)) = self.layout.get(Board::Left, prev.0, prev.1) {
            self.layout.set(Board::Left, prev.0, prev.1, Some(Cell::Belt(d)));
        }
        let free = self.layout.get(Board::Left, x, y).is_none() && self.levels[self.cur].fixed(Board::Left, x, y) == Cell::Floor;
        if free {
            self.layout.set(Board::Left, x, y, Some(Cell::Belt(d)));
        }
        let st = self.stroke.as_mut().unwrap();
        st.path.push((x, y));
        st.placed.push(free);
        self.last_dir = d;
        self.sfx.push(Sfx::Draw);
    }

    fn end_stroke(&mut self) {
        if let Some(st) = self.stroke.take() {
            if !st.moved && st.start_was_belt {
                let (x, y) = st.path[0];
                if let Some(Cell::Belt(d)) = self.layout.get(Board::Left, x, y) {
                    self.layout.set(Board::Left, x, y, Some(Cell::Belt(d.cw())));
                    self.last_dir = d.cw();
                    self.sfx.push(Sfx::Rotate);
                }
            }
        }
    }

    fn place(&mut self, b: Board, x: i32, y: i32, tool: Tool) {
        if self.level().fixed(b, x, y) != Cell::Floor {
            return self.deny("THAT PART IS BOLTED DOWN");
        }
        let existing = self.layout.get(b, x, y);
        if existing.and_then(Tool::of) == Some(tool) {
            return self.rotate_at(b, x, y, false);
        }
        if self.tray_left(self.tool) == Some(0) {
            return self.deny("NONE LEFT ON THE SHELF");
        }
        let under = match existing {
            Some(Cell::Belt(d)) => Some(d),
            _ => None,
        };
        let cell = match tool {
            Tool::Booth(c) => Cell::Booth(c),
            Tool::Switch => Cell::Switch(if under.is_some_and(|d| d.vertical()) { Dir::E } else { Dir::N }),
            Tool::Zipper => {
                let out = under.unwrap_or(Dir::E);
                Cell::Zipper(out, out.ccw())
            }
            Tool::Arrow => Cell::Arrow(self.last_arrow),
            Tool::Belt => Cell::Belt(self.last_dir),
        };
        self.push_undo();
        self.layout.set(b, x, y, Some(cell));
        self.sfx.push(Sfx::Place);
    }

    fn rotate_at(&mut self, b: Board, x: i32, y: i32, flip: bool) {
        let Some(c) = self.layout.get(b, x, y) else {
            return self.deny(if self.is_fixed(b, x, y) { "THAT PART IS BOLTED DOWN" } else { "NOTHING TO TURN" });
        };
        let turned = match (c, flip) {
            (Cell::Belt(d), _) => Cell::Belt(d.cw()),
            (Cell::Switch(l), false) => Cell::Switch(l.cw()),
            (Cell::Switch(l), true) => Cell::Switch(l.opposite()),
            (Cell::Zipper(o, f), false) => Cell::Zipper(o.cw(), f.cw()),
            (Cell::Zipper(o, f), true) => Cell::Zipper(o, f.opposite()),
            (Cell::Arrow(d), _) => Cell::Arrow(d.cw()),
            _ => return self.deny("BOOTHS DON'T TURN"),
        };
        self.push_undo();
        if let Cell::Arrow(d) = turned {
            self.last_arrow = d;
        }
        if let Cell::Belt(d) = turned {
            self.last_dir = d;
        }
        self.layout.set(b, x, y, Some(turned));
        self.sfx.push(Sfx::Rotate);
    }

    fn remove_at(&mut self, b: Board, x: i32, y: i32) {
        if self.layout.get(b, x, y).is_none() {
            if self.is_fixed(b, x, y) {
                self.deny("THAT PART IS BOLTED DOWN");
            }
            return;
        }
        self.push_undo();
        self.layout.set(b, x, y, None);
        self.sfx.push(Sfx::Remove);
    }

    // -----------------------------------------------------------------------------------------
    // Runs

    pub fn start_run(&mut self) {
        self.stroke = None;
        self.trails.clear();
        self.message = None;
        self.sub = 0;
        self.deliver = None;
        self.sfx.push(Sfx::RunStart);
        let reuse = self.made.as_ref().is_some_and(|(_, left)| *left == self.layout.left);
        self.runs += 1;
        let line = format!("{} run {}{}", self.level().id, self.runs, if reuse { " (re-deliver)" } else { "" });
        self.log(&line);
        if reuse {
            // The left board hasn't changed since it last finished: re-deliver straight away.
            self.make = None;
            self.phase = Phase::Lock(3);
            self.say("SAME CHUTES - DELIVERING AGAIN", MSG_INFO);
        } else {
            self.made = None;
            self.make = Some(MakeSim::new(self.level(), &self.layout));
            self.phase = Phase::Make;
        }
    }

    fn stop_run(&mut self) {
        self.phase = Phase::Edit;
        self.make = None;
        self.deliver = None;
        self.trails.clear();
        self.say("STOPPED", MSG_INFO);
    }

    fn beat_ticks(&self) -> u32 {
        match (self.phase, self.fast) {
            (Phase::Make, false) => 4,
            (Phase::Make, true) => 2,
            (_, false) => 2,
            (_, true) => 1,
        }
    }

    /// Chute contents to draw at rest: during Make, what has arrived; during Deliver, what's
    /// left; while editing, the last Make if the left board still matches it.
    pub fn chute_view(&self, k: usize) -> Vec<Color> {
        if let Some(d) = &self.deliver {
            return d.queues[k].iter().copied().collect();
        }
        if let Some(m) = &self.make {
            return m.chutes[k][..m.shown[k]].to_vec();
        }
        match &self.made {
            Some((c, left)) if *left == self.layout.left => c[k].clone(),
            _ => Vec::new(),
        }
    }

    pub fn update(&mut self) {
        self.ticks += 1;
        if self.hall_pal.2 < 16 {
            self.hall_pal.2 = (self.hall_pal.2 + 2).min(16);
        }
        let b = self.beat_ticks();
        match self.phase {
            Phase::Make => {
                if self.sub == 0 {
                    let m = self.make.as_mut().unwrap();
                    if m.state == MakeState::Done {
                        m.step_settle();
                        self.made = Some((m.chutes.clone(), self.layout.left));
                        self.phase = Phase::Lock(if self.fast { 4 } else { 8 });
                        self.sfx.push(Sfx::Lock);
                        self.anim = (1, 1);
                        return;
                    }
                    m.step();
                    let evs: Vec<Ev> = m.events.drain(..).collect();
                    let state = m.state;
                    self.play_events(&evs);
                    if let MakeState::Jam(j, x, y) = state {
                        self.phase = Phase::Jammed(j, x, y);
                        self.say(&format!("JAM: {}", j.text()), MSG_BAD);
                        let line = format!("{} run {}: jam {} at left {x},{y}", self.level().id, self.runs, j.text());
                        self.log(&line);
                        self.anim = (1, 1);
                        return;
                    }
                }
                self.anim = (self.sub as i32 + 1, b as i32);
                self.sub = (self.sub + 1) % b;
            }
            Phase::Lock(t) => {
                if t <= 1 {
                    let chutes = self.made.as_ref().map(|(c, _)| c.clone()).unwrap_or_default();
                    self.deliver = Some(DeliverSim::new(self.level(), &self.layout, &chutes));
                    self.make = None;
                    self.phase = Phase::Deliver;
                    self.sub = 0;
                } else {
                    self.phase = Phase::Lock(t - 1);
                }
            }
            Phase::Deliver => {
                if self.sub == 0 {
                    let d = self.deliver.as_mut().unwrap();
                    d.step();
                    let evs: Vec<Ev> = d.events.drain(..).collect();
                    let state = d.state;
                    self.trails = d.trails.clone();
                    self.play_events(&evs);
                    match state {
                        DeliverState::Won => {
                            self.phase = Phase::Solved;
                            self.solved[self.cur] = true;
                            self.save();
                            self.say("SOLVED", MSG_GOOD);
                            self.anim = (1, 1);
                            let secs = (self.ticks - self.room_started) / 15;
                            let line = format!("{} run {}: SOLVED, {}m{:02}s in room, hints {}", self.level().id, self.runs, secs / 60, secs % 60, self.hints_used);
                            self.log(&line);
                            return;
                        }
                        DeliverState::Failed(f, x, y) => {
                            self.phase = Phase::Failed(f, x, y);
                            self.say(f.text(), MSG_BAD);
                            let line = format!("{} run {}: {} at floor {x},{y}", self.level().id, self.runs, f.text());
                            self.log(&line);
                            self.anim = (1, 1);
                            return;
                        }
                        DeliverState::Running => {}
                    }
                }
                self.anim = (self.sub as i32 + 1, b as i32);
                self.sub = (self.sub + 1) % b;
            }
            _ => {}
        }
    }

    fn play_events(&mut self, evs: &[Ev]) {
        let mut emitted = false;
        for &e in evs {
            let s = match e {
                Ev::Emit if !emitted => {
                    emitted = true;
                    Sfx::Emit
                }
                Ev::Emit => continue,
                Ev::Paint => Sfx::Paint,
                Ev::Switch => Sfx::Switch,
                Ev::IntoChute(k) => {
                    let n = self.make.as_ref().map_or(0, |m| m.chutes[k].len());
                    Sfx::Clink(n as u8)
                }
                Ev::Release(_) => Sfx::Release,
                Ev::Turn => Sfx::Turn,
                Ev::Bin(b) => {
                    let got = self.deliver.as_ref().map_or(0, |d| d.got[b].len());
                    Sfx::Note((b * 2 + got) as u8)
                }
                Ev::Bad => Sfx::Bad,
                Ev::Won => Sfx::Won,
            };
            self.sfx.push(s);
        }
    }
}

fn in_rect((x, y, w, h): (i32, i32, i32, i32), px: i32, py: i32) -> bool {
    px >= x && px < x + w && py >= y && py < y + h
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tick_until_stopped(g: &mut Game) {
        for _ in 0..5000 {
            g.update();
            if !g.phase.running() {
                return;
            }
        }
        panic!("run never finished: {:?}", g.phase);
    }

    #[test]
    fn keyboard_belt_drawing_solves_first_coat() {
        let mut g = Game::new(None, None).unwrap();
        g.open_level(0);
        // Hold Space at (1,3) and walk E, N, E, E into the booth, then lay the far side.
        g.cursor = (1, 3);
        g.key(Key::Act);
        for k in [Key::Right, Key::Up, Key::Right, Key::Right] {
            g.key(k);
        }
        g.key(Key::ActUp);
        g.cursor = (5, 2);
        g.key(Key::Act);
        for k in [Key::Right, Key::Down, Key::Right] {
            g.key(k);
        }
        g.key(Key::ActUp);
        // The last tile points onward (east) into the hatch.
        assert_eq!(g.layout.get(Board::Left, 7, 3), Some(Cell::Belt(Dir::E)));
        g.key(Key::Run);
        tick_until_stopped(&mut g);
        assert_eq!(g.phase, Phase::Solved, "{:?}", g.message);
    }

    #[test]
    fn backtracking_erases_and_tapping_a_belt_turns_it() {
        let mut g = Game::new(None, None).unwrap();
        g.open_level(0);
        g.cursor = (1, 5);
        g.key(Key::Act);
        g.key(Key::Right);
        g.key(Key::Right);
        g.key(Key::Left);
        g.key(Key::ActUp);
        assert_eq!(g.layout.get(Board::Left, 3, 5), None);
        assert_eq!(g.layout.get(Board::Left, 2, 5), Some(Cell::Belt(Dir::E)));
        g.cursor = (2, 5);
        g.key(Key::Act);
        g.key(Key::ActUp);
        assert_eq!(g.layout.get(Board::Left, 2, 5), Some(Cell::Belt(Dir::S)));
        g.key(Key::Undo);
        assert_eq!(g.layout.get(Board::Left, 2, 5), Some(Cell::Belt(Dir::E)));
    }

    #[test]
    fn tray_limits_and_board_rules_are_enforced() {
        let mut g = Game::new(None, None).unwrap();
        g.open_level(2); // Slide: one arrow
        g.pick_tool(1);
        g.cursor = (8 + 5, 2);
        g.key(Key::Act);
        g.cursor = (8 + 3, 4);
        g.key(Key::Act);
        assert_eq!(g.layout.placed(Tool::Arrow), 1);
        g.cursor = (3, 4);
        g.key(Key::Act);
        assert_eq!(g.layout.placed(Tool::Arrow), 1, "arrows don't go on the left board");
        g.cursor = (3, 2);
        g.pick_tool(0);
        g.key(Key::Remove);
        assert_eq!(g.level().fixed(Board::Left, 3, 2), Cell::Booth(RED), "fixed parts stay");
    }

    #[test]
    fn right_only_edits_re_deliver_without_making_again() {
        let mut g = Game::new(None, None).unwrap();
        g.open_level(2);
        g.layout = Layout::default();
        g.key(Key::Run);
        tick_until_stopped(&mut g);
        assert!(matches!(g.phase, Phase::Failed(Fail::HitWall, 7, 2)));
        g.pick_tool(1);
        g.cursor = (8 + 5, 2);
        g.key(Key::Act);
        g.key(Key::Rotate);
        assert_eq!(g.layout.get(Board::Right, 5, 2), Some(Cell::Arrow(Dir::S)));
        g.key(Key::Run);
        assert_eq!(g.phase, Phase::Lock(3), "skips Make");
        tick_until_stopped(&mut g);
        assert_eq!(g.phase, Phase::Solved);
    }

    #[test]
    fn acting_on_the_floor_with_the_belt_picked_grabs_an_arrow() {
        let mut g = Game::new(None, None).unwrap();
        g.open_level(2); // Slide
        assert_eq!(g.tool, 0);
        g.cursor = (8 + 5, 2);
        g.key(Key::Act);
        assert_eq!(g.layout.get(Board::Right, 5, 2), Some(Cell::Arrow(Dir::E)));
        // The shelf is empty now, but Space on the placed arrow still turns it.
        g.tool = 0;
        g.key(Key::Act);
        assert_eq!(g.layout.get(Board::Right, 5, 2), Some(Cell::Arrow(Dir::S)));
    }

    #[test]
    fn work_survives_leaving_a_room_and_clear_can_be_undone() {
        let mut g = Game::new(None, None).unwrap();
        g.open_level(0);
        g.cursor = (1, 3);
        g.key(Key::Act);
        g.key(Key::ActUp);
        g.key(Key::Back);
        assert_eq!(g.screen, Screen::Select);
        g.open_level(1);
        assert_eq!(g.layout, Layout::default());
        g.open_level(0);
        assert_eq!(g.layout.get(Board::Left, 1, 3), Some(Cell::Belt(Dir::E)));
        g.key(Key::Clear);
        assert_eq!(g.layout, Layout::default());
        g.key(Key::Undo);
        assert_eq!(g.layout.get(Board::Left, 1, 3), Some(Cell::Belt(Dir::E)));
    }

    #[test]
    fn hints_cycle_and_the_playtest_log_records_the_visit() {
        let log = std::env::temp_dir().join(format!("chuteworks_test_{}.log", std::process::id()));
        let _ = std::fs::remove_file(&log);
        let mut g = Game::new(None, Some(log.clone())).unwrap();
        g.open_level(4);
        for want in [1, 2, 0] {
            g.key(Key::Hint);
            assert_eq!(g.hint, want);
        }
        g.layout = Layout::from_solution(g.level());
        g.key(Key::Run);
        tick_until_stopped(&mut g);
        g.end_session();
        let text = std::fs::read_to_string(&log).unwrap();
        let _ = std::fs::remove_file(&log);
        for want in ["1-5 THROUGH THE BOOTH: opened", "1-5 hint 1", "1-5 run 1: SOLVED", "hints 2", "session end: 1 of 17"] {
            assert!(text.contains(want), "missing {want:?} in:\n{text}");
        }
    }

    #[test]
    fn up_and_down_hop_between_halls_of_different_sizes() {
        let mut g = Game::new(None, None).unwrap();
        g.sel = 7; // 1-8, last room of hall 1
        g.key(Key::Down);
        assert_eq!(g.levels[g.sel].id, "2-8");
        g.sel = 16; // 2-9 has no room below it in hall 1
        g.key(Key::Up);
        assert_eq!(g.levels[g.sel].id, "1-8");
        let rightmost = g.room_rect(16);
        assert!(rightmost.0 + rightmost.2 <= 392, "nine rooms fit the screen");
    }
}
