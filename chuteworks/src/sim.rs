//! The two halves of a run.
//!
//! **Make** (left board): crates emit items onto belts, one tile per beat; items queue behind
//! each other. Order into a chute depends only on the layout, never on timing: lines can't
//! touch, a switch deals in turn, and a zipper waits for its turn (it knows how many items each
//! input will ever receive, so it only skips a side that has run dry).
//!
//! **Deliver** (right board): chutes take turns from the top, one item on the floor at a time.

use crate::geom::*;
use crate::model::*;
use std::collections::{HashMap, VecDeque};

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Jam {
    DeadEnd,
    Merge,
    OffBoard,
    ChuteFull,
    Mud,
    WrongWay,
    Stuck,
    Loop,
}

impl Jam {
    pub fn text(self) -> &'static str {
        match self {
            Jam::DeadEnd => "DEAD END",
            Jam::Merge => "LINES CAN'T MERGE",
            Jam::OffBoard => "OFF THE BOARD",
            Jam::ChuteFull => "CHUTE IS FULL",
            Jam::Mud => "MUD! THREE COATS",
            Jam::WrongWay => "WRONG WAY IN",
            Jam::Stuck => "STUCK",
            Jam::Loop => "GOING IN CIRCLES",
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Fail {
    HitWall,
    WrongBin,
    BinFull,
    Mud,
    Lost,
    NotFull,
}

impl Fail {
    pub fn text(self) -> &'static str {
        match self {
            Fail::HitWall => "HIT A WALL",
            Fail::WrongBin => "WRONG BIN",
            Fail::BinFull => "BIN IS FULL",
            Fail::Mud => "MUD! THREE COATS",
            Fail::Lost => "GOING IN CIRCLES",
            Fail::NotFull => "BINS NOT FULL",
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Ev {
    Emit,
    Paint,
    Switch,
    IntoChute(usize),
    Release(usize),
    Turn,
    Bin(usize),
    Bad,
    Won,
}

// ---------------------------------------------------------------------------------------------
// Flow counts: how many items will ever leave a cell in a direction. Zippers use this to tell a
// side that has run dry from one whose next item is still on its way.

struct Flow<'a> {
    grid: &'a Grid,
    crates: &'a [Crate],
    memo: HashMap<(i32, i32, Dir), u32>,
    visiting: Vec<(i32, i32, Dir)>,
}

impl Flow<'_> {
    fn inflow(&mut self, x: i32, y: i32, side: Dir) -> u32 {
        self.outflow(x + side.dx(), y + side.dy(), side.opposite())
    }

    fn outflow(&mut self, x: i32, y: i32, d: Dir) -> u32 {
        if !inside(x, y) {
            return 0;
        }
        let key = (x, y, d);
        if let Some(&v) = self.memo.get(&key) {
            return v;
        }
        if self.visiting.contains(&key) {
            return 0; // a belt loop; the run will report it
        }
        self.visiting.push(key);
        let v = match self.grid[idx(x, y)] {
            Cell::Crate(k) => {
                let c = &self.crates[k as usize];
                if c.dir == d {
                    c.items.len() as u32
                } else {
                    0
                }
            }
            Cell::Belt(o) if o == d => Dir::ALL.into_iter().filter(|&s| s != d).map(|s| self.inflow(x, y, s)).sum(),
            Cell::Booth(_) => self.inflow(x, y, d.opposite()),
            Cell::Switch(lever) if lever.vertical() == d.vertical() => {
                let side = lever.cw();
                let t = self.inflow(x, y, side) + self.inflow(x, y, side.opposite());
                if d == lever {
                    t.div_ceil(2)
                } else {
                    t / 2
                }
            }
            Cell::Zipper(out, first) if out == d => self.inflow(x, y, first) + self.inflow(x, y, first.opposite()),
            _ => 0,
        };
        self.visiting.pop();
        self.memo.insert(key, v);
        v
    }
}

// ---------------------------------------------------------------------------------------------
// Make

#[derive(Clone, Debug)]
pub struct MItem {
    pub color: Color,
    pub x: i32,
    pub y: i32,
    pub dir: Dir,
    /// Exit assigned by a switch when the item entered it.
    out: Dir,
    pub from: (i32, i32),
    pub to: (i32, i32),
    pub alive: bool,
    /// Set on the beat it drops into a chute (chute index).
    pub chute: Option<usize>,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum MakeState {
    Running,
    Done,
    Jam(Jam, i32, i32),
}

#[derive(Clone)]
pub struct MakeSim {
    pub grid: Grid,
    rows: Vec<i32>,
    cap: usize,
    crates: Vec<Crate>,
    crate_next: Vec<usize>,
    pub items: Vec<MItem>,
    occ: [Option<usize>; 64],
    belt_in: [Option<Dir>; 64],
    booth: [[Option<usize>; 2]; 64],
    booth_in: [[Option<Dir>; 2]; 64],
    zip: [[Option<usize>; 4]; 64],
    pub zip_next: [Dir; 64],
    zip_left: [[u32; 4]; 64],
    pub lever: [Dir; 64],
    pub chutes: Vec<Vec<Color>>,
    /// Minis drawn at rest in each chute (the newest may still be sliding in).
    pub shown: Vec<usize>,
    pub beat: u32,
    pub state: MakeState,
    pub events: Vec<Ev>,
}

enum Enter {
    Ok,
    Wait,
    Jam(Jam, i32, i32),
}

impl MakeSim {
    pub fn new(level: &Level, layout: &Layout) -> Self {
        let grid = level.combined(layout, Board::Left);
        let mut lever = [Dir::N; 64];
        let mut zip_next = [Dir::N; 64];
        let mut zip_left = [[0u32; 4]; 64];
        let mut flow = Flow { grid: &grid, crates: &level.crates, memo: HashMap::new(), visiting: Vec::new() };
        for y in 0..N {
            for x in 0..N {
                match grid[idx(x, y)] {
                    Cell::Switch(l) => lever[idx(x, y)] = l,
                    Cell::Zipper(_, first) => {
                        zip_next[idx(x, y)] = first;
                        for s in [first, first.opposite()] {
                            zip_left[idx(x, y)][s.index()] = flow.inflow(x, y, s);
                        }
                    }
                    _ => {}
                }
            }
        }
        Self {
            grid,
            rows: level.chutes.clone(),
            cap: level.chute_cap,
            crates: level.crates.clone(),
            crate_next: vec![0; level.crates.len()],
            items: Vec::new(),
            occ: [None; 64],
            belt_in: [None; 64],
            booth: [[None; 2]; 64],
            booth_in: [[None; 2]; 64],
            zip: [[None; 4]; 64],
            zip_next,
            zip_left,
            lever,
            chutes: vec![Vec::new(); level.chutes.len()],
            shown: vec![0; level.chutes.len()],
            beat: 0,
            state: MakeState::Running,
            events: Vec::new(),
        }
    }

    /// Items left in crate `k`, in emission order.
    pub fn crate_left(&self, k: usize) -> &[Color] {
        &self.crates[k].items[self.crate_next[k]..]
    }

    fn cell(&self, x: i32, y: i32) -> Cell {
        if inside(x, y) {
            self.grid[idx(x, y)]
        } else {
            Cell::Wall
        }
    }

    /// Checks whether an item moving `d` may enter (tx, ty) from (sx, sy) right now.
    fn can_enter(&self, sx: i32, sy: i32, tx: i32, ty: i32, d: Dir) -> Enter {
        let side = d.opposite();
        if !inside(tx, ty) {
            if d == Dir::E && tx == N {
                if let Some(k) = self.rows.iter().position(|&r| r == ty) {
                    return if self.chutes[k].len() >= self.cap { Enter::Jam(Jam::ChuteFull, sx, sy) } else { Enter::Ok };
                }
            }
            return Enter::Jam(Jam::OffBoard, sx, sy);
        }
        let t = idx(tx, ty);
        match self.grid[t] {
            Cell::Belt(o) => {
                if o == side {
                    Enter::Jam(Jam::DeadEnd, sx, sy)
                } else if self.belt_in[t].is_some_and(|s| s != side) {
                    Enter::Jam(Jam::Merge, tx, ty)
                } else if self.occ[t].is_some() {
                    Enter::Wait
                } else {
                    Enter::Ok
                }
            }
            Cell::Booth(_) => {
                let a = d.vertical() as usize;
                if self.booth_in[t][a].is_some_and(|s| s != side) {
                    Enter::Jam(Jam::Merge, tx, ty)
                } else if self.booth[t][a].is_some() {
                    Enter::Wait
                } else {
                    Enter::Ok
                }
            }
            Cell::Switch(_) => {
                if self.lever[t].vertical() == d.vertical() {
                    Enter::Jam(Jam::WrongWay, tx, ty)
                } else if self.belt_in[t].is_some_and(|s| s != side) {
                    Enter::Jam(Jam::Merge, tx, ty)
                } else if self.occ[t].is_some() {
                    Enter::Wait
                } else {
                    Enter::Ok
                }
            }
            Cell::Zipper(out, _) => {
                if side == out || side == out.opposite() {
                    Enter::Jam(Jam::WrongWay, tx, ty)
                } else if self.zip[t][side.index()].is_some() {
                    Enter::Wait
                } else {
                    Enter::Ok
                }
            }
            _ => Enter::Jam(Jam::DeadEnd, sx, sy),
        }
    }

    /// Moves item `k` (already allowed by `can_enter`) into (tx, ty).
    fn enter(&mut self, k: usize, tx: i32, ty: i32, d: Dir) -> Option<(Jam, i32, i32)> {
        let side = d.opposite();
        // Leave the old cell.
        let (ox, oy) = (self.items[k].x, self.items[k].y);
        if inside(ox, oy) {
            let o = idx(ox, oy);
            match self.grid[o] {
                Cell::Booth(_) => self.booth[o][self.items[k].dir.vertical() as usize] = None,
                Cell::Zipper(..) => {
                    let from_side = self.items[k].dir.opposite();
                    self.zip[o][from_side.index()] = None;
                    self.zip_next[o] = from_side.opposite();
                }
                Cell::Crate(_) => {}
                _ => self.occ[o] = None,
            }
        }
        let it = &mut self.items[k];
        it.x = tx;
        it.y = ty;
        it.dir = d;
        if !inside(tx, ty) {
            let c = self.rows.iter().position(|&r| r == ty).unwrap();
            let slot = self.chutes[c].len();
            self.chutes[c].push(it.color);
            it.chute = Some(c);
            it.to = chute_slot(ty, slot);
            self.events.push(Ev::IntoChute(c));
            return None;
        }
        let t = idx(tx, ty);
        it.to = cell_center(Board::Left, tx, ty);
        match self.grid[t] {
            Cell::Belt(_) => {
                self.occ[t] = Some(k);
                self.belt_in[t] = Some(side);
            }
            Cell::Booth(c) => {
                let a = d.vertical() as usize;
                self.booth[t][a] = Some(k);
                self.booth_in[t][a] = Some(side);
                if it.color | c != it.color {
                    it.color |= c;
                    self.events.push(Ev::Paint);
                }
                if it.color == MUD {
                    return Some((Jam::Mud, tx, ty));
                }
            }
            Cell::Switch(_) => {
                self.occ[t] = Some(k);
                self.belt_in[t] = Some(side);
                it.out = self.lever[t];
                self.lever[t] = self.lever[t].opposite();
                self.events.push(Ev::Switch);
            }
            Cell::Zipper(..) => {
                self.zip[t][side.index()] = Some(k);
                self.zip_left[t][side.index()] = self.zip_left[t][side.index()].saturating_sub(1);
                it.to = (it.to.0 + side.dx() * 5, it.to.1 + side.dy() * 5);
            }
            _ => {}
        }
        None
    }

    fn wanted(&self, k: usize, picks: &[Option<usize>; 64]) -> Option<(i32, i32, Dir)> {
        let it = &self.items[k];
        let d = match self.cell(it.x, it.y) {
            Cell::Belt(o) => o,
            Cell::Booth(_) => it.dir,
            Cell::Switch(_) => it.out,
            Cell::Zipper(out, _) if picks[idx(it.x, it.y)] == Some(k) => out,
            _ => return None,
        };
        Some((it.x + d.dx(), it.y + d.dy(), d))
    }

    pub fn step(&mut self) {
        if self.state != MakeState::Running {
            return;
        }
        self.beat += 1;
        for k in 0..self.items.len() {
            let it = &mut self.items[k];
            if it.alive && it.chute.is_some() {
                it.alive = false;
                self.shown[it.chute.unwrap()] += 1;
            }
            it.from = it.to;
        }

        // Each zipper offers at most one item this beat: its turn's side, or the other side
        // once the turn's side will never receive anything again.
        let mut picks = [None; 64];
        for i in 0..64 {
            if let Cell::Zipper(..) = self.grid[i] {
                let next = self.zip_next[i];
                picks[i] = match (self.zip[i][next.index()], self.zip_left[i][next.index()]) {
                    (Some(k), _) => Some(k),
                    (None, 0) => self.zip[i][next.opposite().index()],
                    _ => None,
                };
            }
        }

        let mut moved = vec![false; self.items.len()];
        let mut emitted = vec![false; self.crates.len()];
        let mut any = false;
        loop {
            let mut progress = false;
            for k in 0..self.items.len() {
                if moved[k] || !self.items[k].alive || self.items[k].chute.is_some() {
                    continue;
                }
                let Some((tx, ty, d)) = self.wanted(k, &picks) else { continue };
                let (sx, sy) = (self.items[k].x, self.items[k].y);
                match self.can_enter(sx, sy, tx, ty, d) {
                    Enter::Ok => {
                        if let Some(j) = self.enter(k, tx, ty, d) {
                            return self.jam(j);
                        }
                        moved[k] = true;
                        progress = true;
                    }
                    Enter::Wait => {}
                    Enter::Jam(j, x, y) => return self.jam((j, x, y)),
                }
            }
            for c in 0..self.crates.len() {
                if emitted[c] || self.crate_next[c] >= self.crates[c].items.len() {
                    continue;
                }
                let cr = &self.crates[c];
                let (d, sx, sy) = (cr.dir, cr.x, cr.y);
                let (tx, ty) = (sx + d.dx(), sy + d.dy());
                match self.can_enter(sx, sy, tx, ty, d) {
                    Enter::Ok => {
                        let color = cr.items[self.crate_next[c]];
                        self.crate_next[c] += 1;
                        let at = cell_center(Board::Left, sx, sy);
                        self.items.push(MItem { color, x: sx, y: sy, dir: d, out: d, from: at, to: at, alive: true, chute: None });
                        moved.push(true);
                        let k = self.items.len() - 1;
                        if let Some(j) = self.enter(k, tx, ty, d) {
                            return self.jam(j);
                        }
                        self.events.push(Ev::Emit);
                        emitted[c] = true;
                        progress = true;
                    }
                    Enter::Wait => {}
                    Enter::Jam(j, x, y) => return self.jam((j, x, y)),
                }
            }
            any |= progress;
            if !progress {
                break;
            }
        }

        let on_board = self.items.iter().position(|it| it.alive && it.chute.is_none());
        let crates_left = (0..self.crates.len()).any(|c| self.crate_next[c] < self.crates[c].items.len());
        if on_board.is_none() && !crates_left {
            self.state = MakeState::Done;
        } else if !any {
            let (x, y) = match on_board {
                Some(k) => (self.items[k].x, self.items[k].y),
                None => (0..self.crates.len()).find(|&c| self.crate_next[c] < self.crates[c].items.len()).map_or((0, 0), |c| (self.crates[c].x, self.crates[c].y)),
            };
            self.jam((Jam::Stuck, x, y));
        } else if self.beat > 400 {
            let k = on_board.unwrap_or(0);
            let (x, y) = self.items.get(k).map_or((0, 0), |it| (it.x, it.y));
            self.jam((Jam::Loop, x, y));
        }
    }

    /// After the last beat's animation: every item that dropped into a chute is now at rest.
    pub fn step_settle(&mut self) {
        for it in self.items.iter_mut() {
            if it.alive && it.chute.is_some() {
                it.alive = false;
                self.shown[it.chute.unwrap()] += 1;
            }
        }
    }

    fn jam(&mut self, (j, x, y): (Jam, i32, i32)) {
        self.state = MakeState::Jam(j, x, y);
        self.events.push(Ev::Bad);
    }

    /// Runs to the end (tests).
    #[cfg(test)]
    pub fn finish(&mut self) -> MakeState {
        while self.state == MakeState::Running {
            self.step();
        }
        self.state
    }
}

// ---------------------------------------------------------------------------------------------
// Deliver

#[derive(Clone, Debug)]
pub struct DItem {
    pub color: Color,
    pub x: i32,
    pub y: i32,
    pub dir: Dir,
    pub from: (i32, i32),
    pub to: (i32, i32),
    moves: u32,
    pub landed: Option<usize>,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum DeliverState {
    Running,
    Won,
    Failed(Fail, i32, i32),
}

#[derive(Clone)]
pub struct DeliverSim {
    pub grid: Grid,
    pub bins: Vec<Bin>,
    pub lever: [Dir; 64],
    pub rows: Vec<i32>,
    pub queues: Vec<VecDeque<Color>>,
    /// The chute that fires next.
    pub next: usize,
    pub item: Option<DItem>,
    pub got: Vec<Vec<Color>>,
    /// Cell centers each item passed, for chalk trails.
    pub trails: Vec<Vec<(i32, i32)>>,
    pub state: DeliverState,
    pub events: Vec<Ev>,
}

impl DeliverSim {
    pub fn new(level: &Level, layout: &Layout, chutes: &[Vec<Color>]) -> Self {
        let grid = level.combined(layout, Board::Right);
        let mut lever = [Dir::N; 64];
        for (i, c) in grid.iter().enumerate() {
            if let Cell::Switch(l) = c {
                lever[i] = *l;
            }
        }
        Self {
            grid,
            bins: level.bins.clone(),
            lever,
            rows: level.chutes.clone(),
            queues: chutes.iter().map(|c| c.iter().copied().collect()).collect(),
            next: 0,
            item: None,
            got: vec![Vec::new(); level.bins.len()],
            trails: Vec::new(),
            state: DeliverState::Running,
            events: Vec::new(),
        }
    }

    fn fail(&mut self, f: Fail, x: i32, y: i32) {
        self.state = DeliverState::Failed(f, x, y);
        self.events.push(Ev::Bad);
    }

    pub fn step(&mut self) {
        if self.state != DeliverState::Running {
            return;
        }
        if self.item.as_ref().is_some_and(|it| it.landed.is_some()) {
            self.item = None;
        }
        if self.item.is_none() {
            let n = self.queues.len();
            let Some(k) = (0..n).map(|o| (self.next + o) % n).find(|&k| !self.queues[k].is_empty()) else {
                if self.bins.iter().zip(&self.got).all(|(b, g)| g.len() == b.need as usize) {
                    self.state = DeliverState::Won;
                    self.events.push(Ev::Won);
                } else {
                    self.fail(Fail::NotFull, -1, -1);
                }
                return;
            };
            let color = self.queues[k].pop_front().unwrap();
            self.next = (k + 1) % n;
            let row = self.rows[k];
            self.item = Some(DItem { color, x: 0, y: row, dir: Dir::E, from: chute_gate(row), to: cell_center(Board::Right, 0, row), moves: 0, landed: None });
            self.trails.push(Vec::new());
            self.events.push(Ev::Release(k));
            self.arrive();
            return;
        }
        let it = self.item.as_mut().unwrap();
        let (nx, ny) = (it.x + it.dir.dx(), it.y + it.dir.dy());
        it.from = it.to;
        if !inside(nx, ny) {
            let (cx, cy) = cell_center(Board::Right, it.x, it.y);
            it.to = (cx + it.dir.dx() * 8, cy + it.dir.dy() * 8);
            let (x, y) = (it.x, it.y);
            return self.fail(Fail::HitWall, x, y);
        }
        it.x = nx;
        it.y = ny;
        it.to = cell_center(Board::Right, nx, ny);
        self.arrive();
    }

    /// Applies the cell the item just entered.
    fn arrive(&mut self) {
        let it = self.item.as_mut().unwrap();
        let (x, y) = (it.x, it.y);
        let i = idx(x, y);
        it.moves += 1;
        if let Some(t) = self.trails.last_mut() {
            t.push(cell_center(Board::Right, x, y));
        }
        match self.grid[i] {
            Cell::Wall | Cell::Crate(_) | Cell::Belt(_) | Cell::Zipper(..) => {
                it.to = lerp(it.from, it.to, 1, 2);
                return self.fail(Fail::HitWall, x, y);
            }
            Cell::Arrow(d) => {
                if d != it.dir {
                    self.events.push(Ev::Turn);
                }
                it.dir = d;
            }
            Cell::Booth(c) => {
                if it.color | c != it.color {
                    it.color |= c;
                    self.events.push(Ev::Paint);
                }
                if it.color == MUD {
                    return self.fail(Fail::Mud, x, y);
                }
            }
            Cell::Switch(_) => {
                it.dir = self.lever[i];
                self.lever[i] = self.lever[i].opposite();
                self.events.push(Ev::Switch);
            }
            Cell::Bin(b) => {
                let b = b as usize;
                let bin = self.bins[b];
                if bin.want != it.color {
                    return self.fail(Fail::WrongBin, x, y);
                }
                if self.got[b].len() >= bin.need as usize {
                    return self.fail(Fail::BinFull, x, y);
                }
                self.got[b].push(it.color);
                it.landed = Some(b);
                self.events.push(Ev::Bin(b));
            }
            Cell::Floor => {}
        }
        if self.item.as_ref().unwrap().moves > 100 {
            self.fail(Fail::Lost, x, y);
        }
    }

    #[cfg(test)]
    pub fn finish(&mut self) -> DeliverState {
        while self.state == DeliverState::Running {
            self.step();
        }
        self.state
    }
}


#[cfg(test)]
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Outcome {
    Jam(Jam, i32, i32),
    Failed(Fail, i32, i32),
    Won,
}

/// Runs both halves headlessly.
#[cfg(test)]
pub fn run(level: &Level, layout: &Layout) -> (Outcome, Vec<Vec<Color>>) {
    let mut m = MakeSim::new(level, layout);
    if let MakeState::Jam(j, x, y) = m.finish() {
        return (Outcome::Jam(j, x, y), m.chutes);
    }
    let mut d = DeliverSim::new(level, layout, &m.chutes);
    let out = match d.finish() {
        DeliverState::Won => Outcome::Won,
        DeliverState::Failed(f, x, y) => Outcome::Failed(f, x, y),
        DeliverState::Running => unreachable!(),
    };
    (out, m.chutes)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn level(src: &str) -> Level {
        Level::parse("test", src).unwrap()
    }

    const LINE: &str = "name: t
left:
........
........
........
@.......
........
........
........
........
right:
........
........
........
.......R
........
........
........
........
crate 0 3 E red blue red blue
chute 3
bin 7 3 red 2
";

    #[test]
    fn straight_line_fills_chute_in_crate_order() {
        let lv = level(LINE);
        let mut l = Layout::default();
        for x in 1..8 {
            l.set(Board::Left, x, 3, Some(Cell::Belt(Dir::E)));
        }
        let mut m = MakeSim::new(&lv, &l);
        assert_eq!(m.finish(), MakeState::Done);
        assert_eq!(m.chutes[0], vec![RED, BLUE, RED, BLUE]);
    }

    #[test]
    fn dead_end_and_off_board_jam() {
        let lv = level(LINE);
        let mut l = Layout::default();
        l.set(Board::Left, 1, 3, Some(Cell::Belt(Dir::E)));
        assert!(matches!(run(&lv, &l).0, Outcome::Jam(Jam::DeadEnd, 1, 3)));
        l.set(Board::Left, 1, 3, Some(Cell::Belt(Dir::N)));
        for y in 0..3 {
            l.set(Board::Left, 1, y, Some(Cell::Belt(Dir::N)));
        }
        assert!(matches!(run(&lv, &l).0, Outcome::Jam(Jam::OffBoard, 1, 0)));
    }

    #[test]
    fn switch_deals_in_turn_and_zipper_weaves_back() {
        // switch at (2,3) lever N: items 1,3 north, 2,4 south; zipper at (5,3) first S.
        let lv = level(LINE);
        let mut l = Layout::default();
        let mut put = |x, y, c| l.set(Board::Left, x, y, Some(c));
        put(1, 3, Cell::Belt(Dir::E));
        put(2, 3, Cell::Switch(Dir::N));
        put(2, 2, Cell::Belt(Dir::E));
        put(3, 2, Cell::Belt(Dir::E));
        put(4, 2, Cell::Belt(Dir::E));
        put(5, 2, Cell::Belt(Dir::S));
        put(2, 4, Cell::Belt(Dir::E));
        put(3, 4, Cell::Belt(Dir::E));
        put(4, 4, Cell::Belt(Dir::E));
        put(5, 4, Cell::Belt(Dir::N));
        put(5, 3, Cell::Zipper(Dir::E, Dir::S));
        put(6, 3, Cell::Belt(Dir::E));
        put(7, 3, Cell::Belt(Dir::E));
        let mut m = MakeSim::new(&lv, &l);
        assert_eq!(m.finish(), MakeState::Done);
        assert_eq!(m.chutes[0], vec![BLUE, RED, BLUE, RED], "pairs swap");
        l.set(Board::Left, 5, 3, Some(Cell::Zipper(Dir::E, Dir::N)));
        let mut m = MakeSim::new(&lv, &l);
        m.finish();
        assert_eq!(m.chutes[0], vec![RED, BLUE, RED, BLUE], "order restored");
    }

    #[test]
    fn zipper_drains_the_other_side_when_one_runs_dry() {
        let src = LINE.replace("crate 0 3 E red blue red blue", "crate 0 1 E red\ncrate 0 5 E blue blue blue").replace("bin 7 3 red 2", "bin 7 3 red 1");
        let lv = level(&src);
        let mut l = Layout::default();
        let mut put = |x, y, c| l.set(Board::Left, x, y, Some(c));
        for x in 1..4 {
            put(x, 1, Cell::Belt(Dir::E));
            put(x, 5, Cell::Belt(Dir::E));
        }
        put(4, 1, Cell::Belt(Dir::S));
        put(4, 2, Cell::Belt(Dir::S));
        put(4, 5, Cell::Belt(Dir::N));
        put(4, 4, Cell::Belt(Dir::N));
        put(4, 3, Cell::Zipper(Dir::E, Dir::S));
        for x in 5..8 {
            put(x, 3, Cell::Belt(Dir::E));
        }
        let mut m = MakeSim::new(&lv, &l);
        assert_eq!(m.finish(), MakeState::Done);
        assert_eq!(m.chutes[0], vec![BLUE, RED, BLUE, BLUE]);
    }

    #[test]
    fn belts_feeding_one_belt_is_a_merge_jam() {
        let src = LINE.replace("crate 0 3 E red blue red blue", "crate 0 1 E red\ncrate 0 5 E blue");
        let lv = level(&src);
        let mut l = Layout::default();
        let mut put = |x, y, c| l.set(Board::Left, x, y, Some(c));
        put(1, 1, Cell::Belt(Dir::E));
        put(2, 1, Cell::Belt(Dir::S));
        put(2, 2, Cell::Belt(Dir::S));
        put(1, 5, Cell::Belt(Dir::E));
        put(2, 5, Cell::Belt(Dir::N));
        put(2, 4, Cell::Belt(Dir::N));
        put(2, 3, Cell::Belt(Dir::E));
        for x in 3..8 {
            put(x, 3, Cell::Belt(Dir::E));
        }
        assert!(matches!(run(&lv, &l).0, Outcome::Jam(Jam::Merge, 2, 3)));
    }

    #[test]
    fn chutes_take_turns_from_the_top_and_flip_flop_alternates() {
        let src = "name: t
left:
........
@.......
........
........
........
@.......
........
........
right:
....R...
........
........
........
........
........
........
....B...
crate 0 1 E red red
crate 0 5 E blue blue
chute 1
chute 5
bin 4 0 red 2
bin 4 7 blue 2
";
        let lv = level(src);
        let mut l = Layout::default();
        for x in 1..8 {
            l.set(Board::Left, x, 1, Some(Cell::Belt(Dir::E)));
            l.set(Board::Left, x, 5, Some(Cell::Belt(Dir::E)));
        }
        // Both inlets meet at (4,3) where a flip-flop sends red up and blue down.
        l.set(Board::Right, 2, 1, Some(Cell::Arrow(Dir::S)));
        l.set(Board::Right, 2, 5, Some(Cell::Arrow(Dir::N)));
        l.set(Board::Right, 2, 3, Some(Cell::Arrow(Dir::E)));
        l.set(Board::Right, 4, 3, Some(Cell::Switch(Dir::N)));
        assert_eq!(run(&lv, &l).0, Outcome::Won);
        l.set(Board::Right, 4, 3, Some(Cell::Switch(Dir::S)));
        assert!(matches!(run(&lv, &l).0, Outcome::Failed(Fail::WrongBin, ..)));
    }

    #[test]
    fn booths_mix_and_cross() {
        let src = "name: t
left:
........
@.......
........
........
...r....
........
@.......
........
right:
........
.......O
........
........
........
........
.......R
........
crate 0 1 E grey grey
crate 0 6 E yellow yellow
chute 1
chute 6
bin 7 1 orange 2
bin 7 6 red 2
solve: L belt 1 1 EESSS
solve: L belt 3 5 SEEEEE
solve: L belt 1 6 NNEE
solve: L belt 4 4 ENNNEEE
";
        let lv = level(src);
        let (out, chutes) = run(&lv, &Layout::from_solution(&lv));
        assert_eq!(out, Outcome::Won);
        assert_eq!(chutes, vec![vec![RED | YELLOW; 2], vec![RED; 2]]);
    }
}
