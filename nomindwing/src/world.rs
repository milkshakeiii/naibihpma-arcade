//! The simulation. Rocks are made of pixels and so are ships, where each colour is a part.
//! Factories print ships one pixel at a time out of the team's bank, seeds fly to rocks and eat
//! them into new factories, and shots knock pixels loose for anyone to collect.
//! Everything is integer and advances in discrete 15 FPS ticks.

use crate::design::{info, Design, Kind, Part, CARDS};
use crate::geom::{cheb, dist2, on_circle, step_toward, Line};
use crate::rng::Rng;
use std::collections::{BTreeSet, VecDeque};

pub const WORLD_W: i32 = 960;
pub const WORLD_H: i32 = 720;
/// Minimap: one pixel per 12x12 block.
pub const MINI: i32 = 12;
pub const MINI_W: i32 = WORLD_W / MINI;
pub const MINI_H: i32 = WORLD_H / MINI;
/// Tactical view: one pixel per 4x4 block.
pub const TAC: i32 = 4;
pub const TAC_W: i32 = WORLD_W / TAC;
pub const TAC_H: i32 = WORLD_H / TAC;
pub const QUEUE_MAX: usize = 6;
/// Ticks per printed pixel.
pub const PRINT_EVERY: u32 = 1;
/// Ticks a collector spends biting off one pixel (with two or more scoops; one scoop is slower).
pub const CHEW: i32 = 3;
pub const START_BANK: i32 = 30;
/// Rock pixels a seed needs to land.
pub const PLANT_MIN: i32 = 60;
pub const SHOT_SPEED: i32 = 6;
pub const DEBRIS_MAX: usize = 2500;
/// How far a base looks for the next rock when its build order includes a seed.
pub const SPREAD_RANGE: i32 = 220;
/// Kings start in opposite corners; the map is point-symmetric through its centre.
pub const KING_STARTS: [(i32, i32); 2] = [(120, WORLD_H - 120), (WORLD_W - 1 - 120, 119)];

pub struct Rock {
    pub cx: i32,
    pub cy: i32,
    pub r: i32,
    pub count: i32,
    pub x0: i32,
    pub y0: i32,
    pub x1: i32,
    pub y1: i32,
}

#[derive(Clone, Copy, Debug)]
pub struct Job {
    pub kind: Kind,
    pub printed: usize,
    /// The rock a seed is meant for.
    pub dest: Option<usize>,
    /// Which pinned slot this job refills.
    pub slot: Option<usize>,
    /// Re-queue the same card when it finishes (the king's queue only).
    pub repeat: bool,
}

#[derive(Clone, Copy, Debug)]
pub struct Slot {
    pub kind: Kind,
    pub child: Option<usize>,
}

#[derive(Clone, Debug)]
pub struct Ship {
    pub id: usize,
    pub team: usize,
    pub kind: Kind,
    pub x: i32,
    pub y: i32,
    /// 0 north, 1 east, 2 south, 3 west. Rotations are exact quarter turns.
    pub facing: u8,
    /// Movement this tick, for gunners leading their shots.
    pub vx: i32,
    pub vy: i32,
    /// Hit points per design cell; 0 means the pixel is gone.
    pub hp: Vec<u8>,
    pub alive: bool,
    pub parent: Option<usize>,
    pub cool: i32,
    pub carry: i32,
    pub returning: bool,
    pub tpx: Option<(i32, i32)>,
    pub chew: i32,
    pub orbit: usize,
    /// Raiders wait at their factory until their pack is complete.
    pub hunting: bool,
    pub dest: Option<usize>,
    /// Cells grown so far (planted bases grow from nothing).
    pub grown: usize,
    pub slots: Vec<Slot>,
    pub queue: VecDeque<Job>,
    pub job: Option<Job>,
    pub print_t: u32,
    pub rock: Option<usize>,
    pub last_hit: u32,
    pub repair_t: u32,
}

/// Where cell `i` of a design lands when centred on (x, y) with a facing.
pub fn cell_pos(d: &Design, x: i32, y: i32, facing: u8, i: usize) -> (i32, i32) {
    let (dx, dy) = (i as i32 % d.w, i as i32 / d.w);
    let (w, h, rx, ry) = match facing & 3 {
        0 => (d.w, d.h, dx, dy),
        1 => (d.h, d.w, d.h - 1 - dy, dx),
        2 => (d.w, d.h, d.w - 1 - dx, d.h - 1 - dy),
        _ => (d.h, d.w, dy, d.w - 1 - dx),
    };
    (x - w / 2 + rx, y - h / 2 + ry)
}

pub fn dims(d: &Design, facing: u8) -> (i32, i32) {
    if facing & 1 == 1 {
        (d.h, d.w)
    } else {
        (d.w, d.h)
    }
}

impl Ship {
    pub fn design(&self) -> &'static Design {
        &info(self.kind).design
    }
    pub fn dims(&self) -> (i32, i32) {
        dims(self.design(), self.facing)
    }
    pub fn half(&self) -> i32 {
        let (w, h) = self.dims();
        w.max(h) / 2
    }
    pub fn bbox(&self) -> (i32, i32, i32, i32) {
        let (w, h) = self.dims();
        let (l, t) = (self.x - w / 2, self.y - h / 2);
        (l, t, l + w - 1, t + h - 1)
    }
    pub fn cell_pos(&self, i: usize) -> (i32, i32) {
        cell_pos(self.design(), self.x, self.y, self.facing, i)
    }
    /// The design cell under a world pixel, if the design has one there.
    pub fn cell_at(&self, px: i32, py: i32) -> Option<usize> {
        let d = self.design();
        let (w, h) = self.dims();
        let (rx, ry) = (px - (self.x - w / 2), py - (self.y - h / 2));
        if rx < 0 || ry < 0 || rx >= w || ry >= h {
            return None;
        }
        let (dx, dy) = match self.facing & 3 {
            0 => (rx, ry),
            1 => (ry, d.h - 1 - rx),
            2 => (d.w - 1 - rx, d.h - 1 - ry),
            _ => (d.w - 1 - ry, rx),
        };
        let i = (dy * d.w + dx) as usize;
        d.cells[i].map(|_| i)
    }
    pub fn parts_alive(&self, p: Part) -> i32 {
        self.design().cells.iter().zip(&self.hp).filter(|(c, &h)| **c == Some(p) && h > 0).count() as i32
    }
    pub fn complete(&self) -> bool {
        self.grown >= self.design().count
    }
    pub fn hp_now(&self) -> u32 {
        self.hp.iter().map(|&h| h as u32).sum()
    }
    pub fn hp_pct(&self) -> u32 {
        self.hp_now() * 100 / self.design().full_hp.max(1)
    }
    pub fn speed(&self) -> i32 {
        let base = info(self.kind).speed;
        if base == 0 {
            0
        } else if self.parts_alive(Part::Engine) > 0 {
            base
        } else {
            1
        }
    }
}

fn turn(s: &mut Ship, dx: i32, dy: i32) {
    if dx.abs() > dy.abs() {
        s.facing = if dx > 0 { 1 } else { 3 };
    } else if dy.abs() > dx.abs() {
        s.facing = if dy > 0 { 2 } else { 0 };
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Shot {
    pub team: usize,
    pub line: Line,
    pub left: i32,
    pub prev: (i32, i32),
}

/// A rock pixel flying into a growing base (cosmetic: the base cell is already built).
#[derive(Clone, Copy, Debug)]
pub struct Mote {
    pub x: i32,
    pub y: i32,
    pub tx: i32,
    pub ty: i32,
    pub team: usize,
}

#[derive(Clone, Copy, Debug)]
pub struct Debris {
    pub x: i32,
    pub y: i32,
    pub vx: i32,
    pub vy: i32,
    pub drift: i32,
    pub taken: bool,
}

#[derive(Clone, Debug, Default)]
pub struct Team {
    pub bank: i32,
    pub king: usize,
    pub mined: i32,
    pub spent: i32,
    pub planted: i32,
    pub printed: i32,
    pub kills: i32,
    pub lost: i32,
}

#[derive(Clone, Copy, Debug)]
pub enum Event {
    Planted { team: usize, kind: Kind, x: i32, y: i32, by_king: bool },
    Lost { team: usize, kind: Kind, x: i32, y: i32 },
}

#[derive(Clone, Copy, Debug)]
pub enum Cmd {
    Queue { team: usize, kind: Kind, dest: Option<usize> },
    Unqueue { team: usize },
    /// Toggle repeat on the king's nth job (0 = the one printing now).
    ToggleRepeat { team: usize, n: usize },
}

/// A read-only copy of the living ships, taken at the start of each tick.
#[derive(Clone, Copy, Debug)]
pub struct Snap {
    pub id: usize,
    pub team: usize,
    pub kind: Kind,
    pub x: i32,
    pub y: i32,
    pub vx: i32,
    pub vy: i32,
    pub half: i32,
    pub complete: bool,
}

pub struct World {
    pub tick: u32,
    /// Rock id + 1 per world pixel (0 = empty space).
    pub rock: Vec<u8>,
    pub rocks: Vec<Rock>,
    pub mini: Vec<u16>,
    pub tac: Vec<u8>,
    pub ships: Vec<Ship>,
    pub shots: Vec<Shot>,
    pub debris: Vec<Debris>,
    pub motes: Vec<Mote>,
    pub teams: [Team; 2],
    /// Each king's flight direction this tick, -1..1 on each axis.
    pub steer: [(i32, i32); 2],
    pub rng: Rng,
    /// Parallax stars: position on the half-speed plane and shade 0..2.
    pub stars: Vec<(i32, i32, u8)>,
    pub events: Vec<Event>,
    pub winner: Option<usize>,
}

impl World {
    pub fn new(seed: u64) -> World {
        let mut w = World {
            tick: 0,
            rock: vec![0; (WORLD_W * WORLD_H) as usize],
            rocks: Vec::new(),
            mini: vec![0; (MINI_W * MINI_H) as usize],
            tac: vec![0; (TAC_W * TAC_H) as usize],
            ships: Vec::new(),
            shots: Vec::new(),
            debris: Vec::new(),
            motes: Vec::new(),
            teams: [Team::default(), Team::default()],
            steer: [(0, 0); 2],
            rng: Rng::new(seed),
            stars: Vec::new(),
            events: Vec::new(),
            winner: None,
        };
        w.gen_rocks();
        let (sw, sh) = (WORLD_W / 2 + 304, WORLD_H / 2 + 300);
        for _ in 0..260 {
            let x = w.rng.range(0, sw - 1);
            let y = w.rng.range(0, sh - 1);
            let c = match w.rng.below(10) {
                0 => 2,
                1..=3 => 1,
                _ => 0,
            };
            w.stars.push((x, y, c));
        }
        for t in 0..2 {
            let (x, y) = KING_STARTS[t];
            let k = w.spawn(t, Kind::King, x, y, None);
            w.ships[k].facing = if t == 0 { 0 } else { 2 };
            w.teams[t].king = k;
            w.teams[t].bank = START_BANK;
            for dx in [-14, 14] {
                let dy = if t == 0 { 16 } else { -16 };
                let c = w.spawn(t, Kind::Collector, x + dx, y + dy, Some(k));
                w.ships[c].facing = w.ships[k].facing;
            }
        }
        w
    }

    pub fn king(&self, team: usize) -> &Ship {
        &self.ships[self.teams[team].king]
    }

    // ----- Rocks -----

    fn gen_rocks(&mut self) {
        let mut placed: Vec<(i32, i32, i32)> = Vec::new();
        let fits = |placed: &Vec<(i32, i32, i32)>, cx: i32, cy: i32, r: i32| -> bool {
            let e = r * 6 / 5;
            if cx - e < 10 || cy - e < 10 || cx + e >= WORLD_W - 10 || cy + e >= WORLD_H - 10 {
                return false;
            }
            if KING_STARTS.iter().any(|&(kx, ky)| dist2(cx, cy, kx, ky) < (r + 40) * (r + 40)) {
                return false;
            }
            placed.iter().all(|&(x, y, r2)| dist2(cx, cy, x, y) >= (r + r2 + 22) * (r + r2 + 22))
        };
        // Every rock has a mirror twin, so neither king is ever a pixel closer to anything.
        // Two home rocks for each king.
        for (cx, cy, r) in [(196, WORLD_H - 104, 14), (104, WORLD_H - 214, 11)] {
            self.add_pair(cx, cy, r, &mut placed);
        }
        let (mut pairs, mut tries) = (0, 0);
        while pairs < 10 && tries < 4000 {
            tries += 1;
            let r = self.rng.range(8, 15);
            let cx = self.rng.range(r + 12, WORLD_W - r - 13);
            let cy = self.rng.range(r + 12, WORLD_H - r - 13);
            let (mx, my) = (WORLD_W - 1 - cx, WORLD_H - 1 - cy);
            if !fits(&placed, cx, cy, r) || !fits(&placed, mx, my, r) || dist2(cx, cy, mx, my) < (2 * r + 24) * (2 * r + 24) {
                continue;
            }
            self.add_pair(cx, cy, r, &mut placed);
            pairs += 1;
        }
    }

    fn add_pair(&mut self, cx: i32, cy: i32, r: i32, placed: &mut Vec<(i32, i32, i32)>) {
        let shape = self.blob(r);
        self.add_rock(cx, cy, r, &shape);
        let mirrored: Vec<(i32, i32)> = shape.iter().map(|&(x, y)| (-x, -y)).collect();
        let (mx, my) = (WORLD_W - 1 - cx, WORLD_H - 1 - cy);
        self.add_rock(mx, my, r, &mirrored);
        placed.push((cx, cy, r));
        placed.push((mx, my, r));
    }

    /// A lumpy rock: three overlapping circles with a cragged edge.
    fn blob(&mut self, r: i32) -> Vec<(i32, i32)> {
        let a = self.rng.below(16) as usize;
        let (l1x, l1y) = on_circle(0, 0, r / 2, a);
        let (l2x, l2y) = on_circle(0, 0, r * 3 / 5, a + 7);
        let lumps = [(0, 0, r), (l1x, l1y, r * 2 / 3), (l2x, l2y, r / 2)];
        let ext = r * 3 / 2;
        let mut set = BTreeSet::new();
        for y in -ext..=ext {
            for x in -ext..=ext {
                if lumps.iter().any(|&(lx, ly, lr)| dist2(x, y, lx, ly) <= lr * lr) {
                    set.insert((x, y));
                }
            }
        }
        let edge: Vec<(i32, i32)> = set
            .iter()
            .filter(|&&(x, y)| [(1, 0), (-1, 0), (0, 1), (0, -1)].iter().any(|&(dx, dy)| !set.contains(&(x + dx, y + dy))))
            .copied()
            .collect();
        for p in edge {
            if self.rng.chance(40) {
                set.remove(&p);
            }
        }
        set.into_iter().collect()
    }

    fn add_rock(&mut self, cx: i32, cy: i32, r: i32, shape: &[(i32, i32)]) {
        let tag = (self.rocks.len() + 1) as u8;
        let mut rk = Rock { cx, cy, r, count: 0, x0: i32::MAX, y0: i32::MAX, x1: i32::MIN, y1: i32::MIN };
        for &(dx, dy) in shape {
            let (x, y) = (cx + dx, cy + dy);
            if x < 0 || y < 0 || x >= WORLD_W || y >= WORLD_H {
                continue;
            }
            let i = (y * WORLD_W + x) as usize;
            if self.rock[i] != 0 {
                continue;
            }
            self.rock[i] = tag;
            rk.count += 1;
            rk.x0 = rk.x0.min(x);
            rk.y0 = rk.y0.min(y);
            rk.x1 = rk.x1.max(x);
            rk.y1 = rk.y1.max(y);
            self.mini[((y / MINI) * MINI_W + x / MINI) as usize] += 1;
            self.tac[((y / TAC) * TAC_W + x / TAC) as usize] += 1;
        }
        self.rocks.push(rk);
    }

    pub fn rock_at(&self, x: i32, y: i32) -> u8 {
        if x < 0 || y < 0 || x >= WORLD_W || y >= WORLD_H {
            0
        } else {
            self.rock[(y * WORLD_W + x) as usize]
        }
    }

    fn take_rock(&mut self, x: i32, y: i32) -> bool {
        let v = self.rock_at(x, y);
        if v == 0 {
            return false;
        }
        self.rock[(y * WORLD_W + x) as usize] = 0;
        self.rocks[v as usize - 1].count -= 1;
        self.mini[((y / MINI) * MINI_W + x / MINI) as usize] -= 1;
        self.tac[((y / TAC) * TAC_W + x / TAC) as usize] -= 1;
        true
    }

    pub fn nearest_rock_px(&self, a: usize, x: i32, y: i32) -> Option<(i32, i32)> {
        let r = &self.rocks[a];
        if r.count <= 0 {
            return None;
        }
        let tag = (a + 1) as u8;
        let mut best = None;
        let mut bd = i32::MAX;
        for py in r.y0..=r.y1 {
            for px in r.x0..=r.x1 {
                if self.rock[(py * WORLD_W + px) as usize] == tag {
                    let d = dist2(px, py, x, y);
                    if d < bd {
                        bd = d;
                        best = Some((px, py));
                    }
                }
            }
        }
        best
    }

    /// The base standing on a rock, if any.
    pub fn base_on(&self, a: usize) -> Option<&Ship> {
        self.ships.iter().find(|s| s.alive && s.kind.is_base() && s.rock == Some(a))
    }

    /// A rock can take a seed if it is big enough, nobody stands on it, and no other seed of the
    /// same team is already flying there.
    pub fn rock_free(&self, a: usize, team: usize, except: Option<usize>) -> bool {
        self.rocks[a].count >= PLANT_MIN
            && !self.ships.iter().any(|s| {
                s.alive && Some(s.id) != except && ((s.kind.is_base() && s.rock == Some(a)) || (s.kind.is_seed() && s.team == team && s.dest == Some(a)))
            })
    }

    pub fn free_rock_near(&self, x: i32, y: i32, maxd: i32, team: usize, except: Option<usize>) -> Option<usize> {
        (0..self.rocks.len())
            .filter(|&a| dist2(self.rocks[a].cx, self.rocks[a].cy, x, y) <= maxd * maxd && self.rock_free(a, team, except))
            .min_by_key(|&a| dist2(self.rocks[a].cx, self.rocks[a].cy, x, y))
    }

    /// The rock nearest a point, within its radius plus `slack`.
    pub fn rock_near_point(&self, x: i32, y: i32, slack: i32) -> Option<usize> {
        (0..self.rocks.len())
            .filter(|&a| {
                let r = &self.rocks[a];
                r.count > 0 && dist2(r.cx, r.cy, x, y) <= (r.r + slack) * (r.r + slack)
            })
            .min_by_key(|&a| dist2(self.rocks[a].cx, self.rocks[a].cy, x, y))
    }

    // ----- Ships -----

    pub fn spawn(&mut self, team: usize, kind: Kind, x: i32, y: i32, parent: Option<usize>) -> usize {
        let d = &info(kind).design;
        let id = self.ships.len();
        let hp = d.cells.iter().map(|c| c.map_or(0, |p| p.hp())).collect();
        let orbit = self.rng.below(16) as usize;
        self.ships.push(Ship {
            id,
            team,
            kind,
            x,
            y,
            facing: 0,
            vx: 0,
            vy: 0,
            hp,
            alive: true,
            parent,
            cool: 0,
            carry: 0,
            returning: false,
            tpx: None,
            chew: 0,
            orbit,
            hunting: false,
            dest: None,
            grown: d.count,
            slots: Vec::new(),
            queue: VecDeque::new(),
            job: None,
            print_t: 0,
            rock: None,
            last_hit: 0,
            repair_t: 0,
        });
        id
    }

    fn snapshot(&self) -> Vec<Snap> {
        self.ships
            .iter()
            .filter(|s| s.alive)
            .map(|s| Snap { id: s.id, team: s.team, kind: s.kind, x: s.x, y: s.y, vx: s.vx, vy: s.vy, half: s.half(), complete: s.complete() })
            .collect()
    }

    /// Where a factory's next ship appears (behind a king, below a base) and which way it faces.
    pub fn print_spot(&self, i: usize, kind: Kind) -> (i32, i32, u8) {
        let s = &self.ships[i];
        let child = &info(kind).design;
        let (fw, fh) = s.dims();
        let gap = child.w.max(child.h) / 2 + 3;
        let (ox, oy) = match s.facing & 3 {
            0 => (0, fh / 2 + gap),
            1 => (-(fw / 2) - gap, 0),
            2 => (0, -(fh / 2) - gap),
            _ => (fw / 2 + gap, 0),
        };
        let f = if s.kind == Kind::King { s.facing } else { 2 };
        ((s.x + ox).clamp(4, WORLD_W - 5), (s.y + oy).clamp(4, WORLD_H - 5), f)
    }

    // ----- The tick -----

    pub fn tick(&mut self, cmds: &[Cmd]) {
        if self.winner.is_some() {
            return;
        }
        self.tick += 1;
        for &c in cmds {
            self.apply(c);
        }
        let snap = self.snapshot();
        for s in self.ships.iter_mut() {
            s.vx = 0;
            s.vy = 0;
        }
        for t in 0..2 {
            let k = self.teams[t].king;
            if self.ships[k].alive {
                self.steer_king(k, self.steer[t]);
            }
        }
        let n = self.ships.len();
        for i in 0..n {
            if !self.ships[i].alive {
                continue;
            }
            match self.ships[i].kind {
                Kind::King => {
                    self.factory_tick(i);
                    self.gun_tick(i, &snap);
                    self.repair_tick(i);
                }
                Kind::Outpost | Kind::Forward => {
                    if self.ships[i].complete() {
                        self.factory_tick(i);
                        self.gun_tick(i, &snap);
                        self.repair_tick(i);
                    } else {
                        self.grow_tick(i);
                    }
                }
                Kind::Collector => self.collector_tick(i, &snap),
                Kind::Escort => {
                    self.escort_tick(i, &snap);
                    self.gun_tick(i, &snap);
                }
                Kind::Raider => {
                    self.raider_tick(i, &snap);
                    self.gun_tick(i, &snap);
                }
                Kind::OutpostSeed | Kind::ForwardSeed => self.seed_tick(i),
            }
        }
        self.shots_tick();
        self.debris_tick();
        self.deaths();
    }

    fn apply(&mut self, c: Cmd) {
        match c {
            Cmd::Queue { team, kind, dest } => {
                let k = self.teams[team].king;
                if !CARDS.contains(&kind) || (kind.is_seed() && dest.is_none()) {
                    return;
                }
                let s = &mut self.ships[k];
                if s.alive && s.queue.len() < QUEUE_MAX {
                    s.queue.push_back(Job { kind, printed: 0, dest, slot: None, repeat: false });
                }
            }
            Cmd::ToggleRepeat { team, n } => {
                let k = self.teams[team].king;
                let s = &mut self.ships[k];
                let j = if s.job.is_some() {
                    if n == 0 {
                        s.job.as_mut()
                    } else {
                        s.queue.get_mut(n - 1)
                    }
                } else {
                    s.queue.get_mut(n)
                };
                if let Some(j) = j {
                    if !j.kind.is_seed() {
                        j.repeat = !j.repeat;
                    }
                }
            }
            Cmd::Unqueue { team } => {
                let k = self.teams[team].king;
                if self.ships[k].queue.pop_back().is_none() {
                    if let Some(j) = self.ships[k].job.take() {
                        self.teams[team].bank += j.printed as i32;
                        self.teams[team].spent -= j.printed as i32;
                    }
                }
            }
        }
    }

    fn move_to(&mut self, i: usize, tx: i32, ty: i32) {
        let sp = self.ships[i].speed();
        if sp == 0 {
            return;
        }
        let s = &mut self.ships[i];
        let (nx, ny) = step_toward(s.x, s.y, tx, ty, sp);
        let h = s.half();
        let (nx, ny) = (nx.clamp(h, WORLD_W - 1 - h), ny.clamp(h, WORLD_H - 1 - h));
        let (dx, dy) = (nx - s.x, ny - s.y);
        s.vx = dx;
        s.vy = dy;
        turn(s, dx, dy);
        s.x = nx;
        s.y = ny;
    }

    fn steer_king(&mut self, k: usize, (dx, dy): (i32, i32)) {
        let sp = self.ships[k].speed();
        let s = &mut self.ships[k];
        let h = s.half();
        let nx = (s.x + dx.signum() * sp).clamp(h, WORLD_W - 1 - h);
        let ny = (s.y + dy.signum() * sp).clamp(h, WORLD_H - 1 - h);
        let (mx, my) = (nx - s.x, ny - s.y);
        s.vx = mx;
        s.vy = my;
        turn(s, mx, my);
        s.x = nx;
        s.y = ny;
    }

    fn factory_tick(&mut self, i: usize) {
        let team = self.ships[i].team;
        if self.ships[i].job.is_none() {
            let next = if self.ships[i].kind == Kind::King { self.ships[i].queue.pop_front() } else { self.next_slot_job(i) };
            self.ships[i].job = next;
            self.ships[i].print_t = 0;
        }
        let Some(job) = self.ships[i].job else { return };
        let need = info(job.kind).design.count;
        let mut printed = job.printed;
        if printed < need {
            self.ships[i].print_t += 1;
            if self.ships[i].print_t >= PRINT_EVERY && self.teams[team].bank > 0 {
                self.ships[i].print_t = 0;
                self.teams[team].bank -= 1;
                self.teams[team].spent += 1;
                printed += 1;
                if let Some(j) = self.ships[i].job.as_mut() {
                    j.printed = printed;
                }
            }
        }
        if printed >= need {
            let (px, py, f) = self.print_spot(i, job.kind);
            let id = self.spawn(team, job.kind, px, py, Some(i));
            self.ships[id].facing = f;
            self.ships[id].dest = job.dest;
            if let Some(sl) = job.slot {
                self.ships[i].slots[sl].child = Some(id);
            }
            self.ships[i].job = None;
            if job.repeat && self.ships[i].queue.len() < QUEUE_MAX {
                self.ships[i].queue.push_back(Job { printed: 0, ..job });
            }
            self.teams[team].printed += 1;
        }
    }

    /// A base refills the first empty slot of its pinned build order.
    fn next_slot_job(&self, i: usize) -> Option<Job> {
        let s = &self.ships[i];
        for (k, slot) in s.slots.iter().enumerate() {
            let filled = slot.child.map_or(false, |c| self.ships[c].alive && self.ships[c].team == s.team);
            if filled {
                continue;
            }
            if slot.kind.is_seed() {
                // Looking for a rock is the costly part, so a base only looks once a second.
                if (self.tick + i as u32) % 15 != 0 {
                    continue;
                }
                match self.free_rock_near(s.x, s.y, SPREAD_RANGE, s.team, None) {
                    Some(r) => return Some(Job { kind: slot.kind, printed: 0, dest: Some(r), slot: Some(k), repeat: false }),
                    None => continue,
                }
            }
            return Some(Job { kind: slot.kind, printed: 0, dest: None, slot: Some(k), repeat: false });
        }
        None
    }

    fn grow_tick(&mut self, i: usize) {
        let (cell, wx, wy, team, rock) = {
            let s = &self.ships[i];
            let d = s.design();
            if s.grown >= d.count {
                return;
            }
            let cell = d.order[s.grown];
            let (wx, wy) = s.cell_pos(cell);
            (cell, wx, wy, s.team, s.rock)
        };
        // Rock under the footprint turns straight into factory; after that it pulls from its rock,
        // and only when the rock is gone does it draw on the bank.
        let ok = if self.take_rock(wx, wy) {
            true
        } else if let Some((px, py)) = rock.and_then(|a| self.nearest_rock_px(a, wx, wy)) {
            self.motes.push(Mote { x: px, y: py, tx: wx, ty: wy, team });
            self.take_rock(px, py)
        } else if self.teams[team].bank > 0 {
            self.teams[team].bank -= 1;
            self.teams[team].spent += 1;
            true
        } else {
            false
        };
        if ok {
            let s = &mut self.ships[i];
            let part = s.design().cells[cell].unwrap_or(Part::Hull);
            s.hp[cell] = part.hp();
            s.grown += 1;
        }
    }

    fn repair_tick(&mut self, i: usize) {
        let tick = self.tick;
        let team = self.ships[i].team;
        if tick.saturating_sub(self.ships[i].last_hit) < 45 || self.teams[team].bank <= 0 {
            return;
        }
        let s = &mut self.ships[i];
        s.repair_t += 1;
        if s.repair_t < 6 {
            return;
        }
        let d = s.design();
        if let Some(&c) = d.order.iter().find(|&&c| s.hp[c] < d.cells[c].map_or(0, |p| p.hp())) {
            s.hp[c] = d.cells[c].map_or(0, |p| p.hp());
            s.repair_t = 0;
            self.teams[team].bank -= 1;
            self.teams[team].spent += 1;
        }
    }

    fn gun_tick(&mut self, i: usize, snap: &[Snap]) {
        if self.ships[i].cool > 0 {
            self.ships[i].cool -= 1;
            return;
        }
        let s = &self.ships[i];
        let guns = s.parts_alive(Part::Gun);
        if guns == 0 {
            return;
        }
        let range = info(s.kind).range;
        let (team, x, y) = (s.team, s.x, s.y);
        let tgt = snap.iter().filter(|e| e.team != team && cheb(e.x, e.y, x, y) <= range + e.half).min_by_key(|e| dist2(e.x, e.y, x, y));
        if let Some(e) = tgt {
            // Aim at a random surviving pixel (aiming at the centre drills a tunnel that later
            // shots fly straight through), led by the target's last step so only a change of
            // course dodges.
            let (px, py) = {
                let ts = &self.ships[e.id];
                let alive: Vec<usize> = (0..ts.hp.len()).filter(|&c| ts.hp[c] > 0).collect();
                if alive.is_empty() {
                    (e.x, e.y)
                } else {
                    let c = alive[self.rng.below(alive.len() as u32) as usize];
                    ts.cell_pos(c)
                }
            };
            let t = cheb(e.x, e.y, x, y) / SHOT_SPEED;
            let (mut ax, mut ay) = (px + e.vx * t, py + e.vy * t);
            if (ax, ay) == (x, y) {
                (ax, ay) = (e.x, e.y + 1);
            }
            self.shots.push(Shot { team, line: Line::new(x, y, ax, ay), left: range + 12, prev: (x, y) });
            self.ships[i].cool = (20 / guns).max(3);
        }
    }

    fn collector_tick(&mut self, i: usize, snap: &[Snap]) {
        let (team, x, y, parent, carry) = {
            let s = &self.ships[i];
            (s.team, s.x, s.y, s.parent, s.carry)
        };
        let scoops = self.ships[i].parts_alive(Part::Scoop);
        let cap = scoops * 2;
        if self.ships[i].returning || carry >= cap {
            self.ships[i].returning = true;
            let drop = snap.iter().filter(|o| o.team == team && o.kind.is_factory() && o.complete).min_by_key(|o| dist2(o.x, o.y, x, y)).copied();
            if let Some(o) = drop {
                if cheb(x, y, o.x, o.y) <= o.half + 3 {
                    self.teams[team].bank += carry;
                    self.teams[team].mined += carry;
                    let s = &mut self.ships[i];
                    s.carry = 0;
                    s.returning = cap == 0;
                    s.tpx = None;
                } else {
                    self.move_to(i, o.x, o.y);
                }
            }
            return;
        }
        // Loose pixels nearby come first.
        let near = self
            .debris
            .iter()
            .enumerate()
            .filter(|(_, d)| !d.taken && cheb(d.x, d.y, x, y) <= 40)
            .min_by_key(|(_, d)| dist2(d.x, d.y, x, y))
            .map(|(k, d)| (k, d.x, d.y));
        if let Some((k, dx, dy)) = near {
            if cheb(x, y, dx, dy) <= 1 {
                self.debris[k].taken = true;
                self.ships[i].carry += 1;
            } else {
                self.move_to(i, dx, dy);
            }
            return;
        }
        let valid = self.ships[i].tpx.map_or(false, |(px, py)| self.rock_at(px, py) != 0);
        if !valid {
            let (hx, hy) = parent.filter(|&p| self.ships[p].alive && self.ships[p].team == team).map(|p| (self.ships[p].x, self.ships[p].y)).unwrap_or((x, y));
            let a = (0..self.rocks.len()).filter(|&a| self.rocks[a].count > 0).min_by_key(|&a| dist2(self.rocks[a].cx, self.rocks[a].cy, hx, hy));
            self.ships[i].tpx = a.and_then(|a| self.nearest_rock_px(a, x, y));
        }
        match self.ships[i].tpx {
            Some((px, py)) => {
                if cheb(x, y, px, py) <= 2 {
                    let bite = {
                        let s = &mut self.ships[i];
                        s.chew += 1;
                        if s.chew >= if scoops >= 2 { CHEW } else { CHEW * 2 } {
                            s.chew = 0;
                            s.tpx = None;
                            true
                        } else {
                            false
                        }
                    };
                    if bite && self.take_rock(px, py) {
                        self.ships[i].carry += 1;
                    }
                } else {
                    self.move_to(i, px, py);
                }
            }
            None => {
                if carry > 0 {
                    self.ships[i].returning = true;
                }
            }
        }
    }

    fn escort_tick(&mut self, i: usize, snap: &[Snap]) {
        let (team, x, y, parent, orbit) = {
            let s = &self.ships[i];
            (s.team, s.x, s.y, s.parent, s.orbit)
        };
        let anchor = parent
            .filter(|&p| self.ships[p].alive && self.ships[p].team == team && self.ships[p].kind.is_factory())
            .map(|p| (self.ships[p].x, self.ships[p].y, self.ships[p].half()))
            .or_else(|| snap.iter().filter(|o| o.team == team && o.kind.is_factory()).min_by_key(|o| dist2(o.x, o.y, x, y)).map(|o| (o.x, o.y, o.half)));
        let Some((ax, ay, ah)) = anchor else {
            self.raider_tick(i, snap);
            return;
        };
        let range = info(Kind::Escort).range;
        let threat = snap.iter().filter(|e| e.team != team && cheb(e.x, e.y, ax, ay) <= 72 + ah).min_by_key(|e| dist2(e.x, e.y, x, y)).copied();
        if let Some(e) = threat {
            if cheb(x, y, e.x, e.y) > range - 6 + e.half {
                self.move_to(i, e.x, e.y);
            }
        } else {
            let slot = (orbit + (self.tick / 5) as usize) % 16;
            let (tx, ty) = on_circle(ax, ay, ah + 9, slot);
            self.move_to(i, tx, ty);
        }
    }

    fn raider_tick(&mut self, i: usize, snap: &[Snap]) {
        let (team, x, y, parent, hunting, orbit) = {
            let s = &self.ships[i];
            (s.team, s.x, s.y, s.parent, s.hunting, s.orbit)
        };
        let range = info(Kind::Raider).range;
        let close = snap.iter().filter(|e| e.team != team && cheb(e.x, e.y, x, y) <= 56).min_by_key(|e| dist2(e.x, e.y, x, y)).copied();
        let home = parent.filter(|&p| self.ships[p].alive && self.ships[p].team == team && self.ships[p].kind.is_factory());
        if !hunting && close.is_none() {
            match home {
                Some(p) => {
                    let (px, py, ph) = (self.ships[p].x, self.ships[p].y, self.ships[p].half());
                    let pack: Vec<usize> = self
                        .ships
                        .iter()
                        .filter(|s| s.alive && s.kind == Kind::Raider && s.parent == Some(p) && !s.hunting && cheb(s.x, s.y, px, py) <= ph + 24)
                        .map(|s| s.id)
                        .collect();
                    if pack.len() >= 3 {
                        for id in pack {
                            self.ships[id].hunting = true;
                        }
                    } else {
                        let slot = (orbit + (self.tick / 7) as usize) % 16;
                        let (tx, ty) = on_circle(px, py, ph + 14, slot);
                        self.move_to(i, tx, ty);
                        return;
                    }
                }
                None => self.ships[i].hunting = true,
            }
        }
        let target = close.or_else(|| snap.iter().filter(|e| e.team != team && e.kind.is_factory()).min_by_key(|e| dist2(e.x, e.y, x, y)).copied());
        if let Some(e) = target {
            if cheb(x, y, e.x, e.y) > range - 6 + e.half {
                self.move_to(i, e.x, e.y);
            }
        }
    }

    fn seed_tick(&mut self, i: usize) {
        let (team, x, y, dest) = {
            let s = &self.ships[i];
            (s.team, s.x, s.y, s.dest)
        };
        let dest = if dest.map_or(false, |a| self.rock_free(a, team, Some(i))) { dest } else { self.free_rock_near(x, y, 2000, team, Some(i)) };
        self.ships[i].dest = dest;
        let Some(a) = dest else { return };
        let (tx, ty) = (self.rocks[a].cx, self.rocks[a].cy);
        self.move_to(i, tx, ty);
        let s = &self.ships[i];
        if cheb(s.x, s.y, tx, ty) <= 1 {
            self.plant(i, a);
        }
    }

    fn plant(&mut self, i: usize, a: usize) {
        let Some(base) = self.ships[i].kind.grows_into() else { return };
        let by_king = self.ships[i].parent == Some(self.teams[self.ships[i].team].king);
        let d = &info(base).design;
        let (cx, cy) = (self.rocks[a].cx, self.rocks[a].cy);
        let s = &mut self.ships[i];
        s.kind = base;
        s.hp = vec![0; d.cells.len()];
        s.grown = 0;
        s.facing = 0;
        s.x = cx;
        s.y = cy;
        s.rock = Some(a);
        s.dest = None;
        s.slots = base.pinned().iter().map(|&k| Slot { kind: k, child: None }).collect();
        s.job = None;
        let team = s.team;
        self.teams[team].planted += 1;
        self.events.push(Event::Planted { team, kind: base, x: cx, y: cy, by_king });
    }

    fn shots_tick(&mut self) {
        let mut shots = std::mem::take(&mut self.shots);
        let boxes: Vec<(usize, usize, (i32, i32, i32, i32))> = self.ships.iter().filter(|s| s.alive).map(|s| (s.id, s.team, s.bbox())).collect();
        shots.retain_mut(|sh| {
            for _ in 0..SHOT_SPEED {
                sh.prev = (sh.line.x, sh.line.y);
                sh.line.step();
                sh.left -= 1;
                let (x, y) = (sh.line.x, sh.line.y);
                if sh.left <= 0 || x < 0 || y < 0 || x >= WORLD_W || y >= WORLD_H {
                    return false;
                }
                for &(id, team, (l, t, r, b)) in &boxes {
                    if team == sh.team || x < l || y < t || x > r || y > b {
                        continue;
                    }
                    if let Some(c) = self.ships[id].cell_at(x, y) {
                        if self.ships[id].hp[c] > 0 && self.ships[id].alive {
                            let s = &mut self.ships[id];
                            s.hp[c] -= 1;
                            s.last_hit = self.tick;
                            if s.hp[c] == 0 {
                                let (vx, vy) = sh.line.dir();
                                let drift = 2 + self.rng.below(4) as i32;
                                self.debris.push(Debris { x, y, vx, vy, drift, taken: false });
                            }
                            return false;
                        }
                    }
                }
            }
            true
        });
        self.shots.append(&mut shots);
    }

    fn debris_tick(&mut self) {
        for m in &mut self.motes {
            (m.x, m.y) = step_toward(m.x, m.y, m.tx, m.ty, 3);
        }
        self.motes.retain(|m| (m.x, m.y) != (m.tx, m.ty));
        self.debris.retain(|d| !d.taken);
        for d in &mut self.debris {
            if d.drift > 0 {
                d.x = (d.x + d.vx).clamp(0, WORLD_W - 1);
                d.y = (d.y + d.vy).clamp(0, WORLD_H - 1);
                d.drift -= 1;
            }
        }
        if self.debris.len() > DEBRIS_MAX {
            let n = self.debris.len() - DEBRIS_MAX;
            self.debris.drain(0..n);
        }
    }

    /// A ship whose core pixels are all gone bursts into loose pixels.
    fn deaths(&mut self) {
        for i in 0..self.ships.len() {
            let s = &self.ships[i];
            if !s.alive || s.grown == 0 {
                continue;
            }
            let d = s.design();
            if d.cells.iter().zip(&s.hp).any(|(c, &h)| *c == Some(Part::Core) && h > 0) {
                continue;
            }
            let (team, kind, x, y, carry) = (s.team, s.kind, s.x, s.y, s.carry);
            let cells: Vec<(i32, i32)> = (0..d.cells.len()).filter(|&c| s.hp[c] > 0).map(|c| s.cell_pos(c)).collect();
            self.ships[i].alive = false;
            for (px, py) in cells {
                let drift = 2 + self.rng.below(5) as i32;
                self.debris.push(Debris { x: px, y: py, vx: (px - x).signum(), vy: (py - y).signum(), drift, taken: false });
            }
            for _ in 0..carry {
                let (vx, vy) = (self.rng.range(-1, 1), self.rng.range(-1, 1));
                self.debris.push(Debris { x, y, vx, vy, drift: 1 + self.rng.below(3) as i32, taken: false });
            }
            self.teams[team].lost += 1;
            self.teams[1 - team].kills += 1;
            self.events.push(Event::Lost { team, kind, x, y });
            if kind == Kind::King {
                self.winner = Some(1 - team);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::design::ALL;

    #[test]
    fn rotated_cells_map_back() {
        let mut w = World::new(1);
        for k in ALL {
            let id = w.spawn(0, k, 300, 300, None);
            for f in 0..4 {
                w.ships[id].facing = f;
                let s = &w.ships[id];
                for c in 0..s.design().cells.len() {
                    if s.design().cells[c].is_some() {
                        let (x, y) = s.cell_pos(c);
                        assert_eq!(s.cell_at(x, y), Some(c), "{k:?} facing {f}");
                        let (l, t, r, b) = s.bbox();
                        assert!(x >= l && x <= r && y >= t && y <= b);
                    }
                }
            }
        }
    }

    #[test]
    fn map_is_point_symmetric() {
        let w = World::new(9);
        for y in 0..WORLD_H {
            for x in 0..WORLD_W {
                let a = w.rock_at(x, y) != 0;
                let b = w.rock_at(WORLD_W - 1 - x, WORLD_H - 1 - y) != 0;
                assert_eq!(a, b, "asymmetric rock pixel at {x},{y}");
            }
        }
        assert!(w.rocks.len() >= 15, "only {} rocks", w.rocks.len());
    }

    #[test]
    fn collectors_fill_the_bank() {
        let mut w = World::new(2);
        let before = w.teams[0].bank;
        for _ in 0..15 * 40 {
            w.tick(&[]);
        }
        assert!(w.teams[0].mined > 20, "mined {}", w.teams[0].mined);
        assert!(w.teams[0].bank > before);
    }

    #[test]
    fn printing_spends_one_pixel_at_a_time() {
        let mut w = World::new(3);
        w.teams[0].bank = 100;
        w.tick(&[Cmd::Queue { team: 0, kind: Kind::Raider, dest: None }]);
        let n = w.ships.len();
        for _ in 0..2 * 18 + 2 {
            w.tick(&[]);
        }
        assert_eq!(w.ships.len(), n + 1);
        assert_eq!(w.ships[n].kind, Kind::Raider);
        assert!(w.teams[0].spent >= 18);
    }

    #[test]
    fn a_seed_grows_an_outpost_that_fills_its_slots() {
        let mut w = World::new(4);
        w.teams[0].bank = 400;
        let (kx, ky) = (w.king(0).x, w.king(0).y);
        let rock = w.free_rock_near(kx, ky, 2000, 0, None).expect("a free rock");
        w.tick(&[Cmd::Queue { team: 0, kind: Kind::OutpostSeed, dest: Some(rock) }]);
        let mut grown = None;
        for _ in 0..15 * 60 {
            w.tick(&[]);
            if let Some(b) = w.ships.iter().find(|s| s.alive && s.team == 0 && s.kind == Kind::Outpost && s.complete()) {
                grown = Some(b.id);
                break;
            }
        }
        let b = grown.expect("outpost grew");
        assert_eq!(w.ships[b].rock, Some(rock));
        for _ in 0..15 * 30 {
            w.tick(&[]);
        }
        let kids = w.ships.iter().filter(|s| s.alive && s.parent == Some(b)).count();
        assert!(kids >= 3, "outpost printed {kids}");
    }

    #[test]
    fn raiders_wear_down_a_still_king() {
        let mut w = World::new(1);
        let k = w.teams[0].king;
        for s in w.ships.iter_mut() {
            if s.id != k {
                s.alive = false;
            }
        }
        w.teams[0].bank = 0;
        let (kx, ky) = (w.ships[k].x, w.ships[k].y);
        for n in 0..3 {
            let r = w.spawn(1, Kind::Raider, kx + 43, ky - 31 + n * 6, None);
            w.ships[r].hunting = true;
        }
        let before = w.ships[k].hp_now();
        for _ in 0..60 {
            w.tick(&[]);
        }
        let after = w.ships[k].hp_now();
        assert!(after + 10 < before, "king hp {before} -> {after}");
    }

    #[test]
    fn a_shot_knocks_a_pixel_loose() {
        let mut w = World::new(5);
        // A finished outpost doesn't move, so the shot can't miss. Its top row is armour (2 hp),
        // so two shots knock one pixel loose.
        let id = w.spawn(1, Kind::Outpost, 500, 300, None);
        for k in 0..2 {
            w.shots.push(Shot { team: 0, line: Line::new(500, 250 - k * 3, 500, 300), left: 80, prev: (500, 250) });
        }
        let before = w.ships[id].hp_now();
        for _ in 0..12 {
            w.tick(&[]);
        }
        assert_eq!(w.ships[id].hp_now() + 2, before);
        assert_eq!(w.debris.len(), 1);
    }
}
