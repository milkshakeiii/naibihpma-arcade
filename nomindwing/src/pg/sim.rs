//! The proving ground's arena. Ships are built from parts and fly with integer inertia. Each one
//! has a three-layer brain:
//! - the order picks where to go,
//! - the parts fire on reflex,
//! - mass and thrust decide how to fly.
//! Nothing here knows what kind of ship it is flying.

use super::design::*;
use crate::geom::{cheb, dist2, Line};
use crate::rng::Rng;

pub const AW: i32 = 304;
pub const AH: i32 = 300;
pub const TIME_LIMIT: u32 = 15 * 180;
/// Each team's home station: where hauls are delivered and the hurt run to.
pub const STATIONS: [(i32, i32); 2] = [(10, AH / 2), (AW - 11, AH / 2)];
const FLAK_SPEED: i32 = 6;
const LANCE_SPEED: i32 = 4;
const GUARD_RADIUS: i32 = 100;
/// Ticks after a hit during which an unarmed ship keeps fleeing.
const SCARED: u32 = 30;

pub fn isqrt(n: i32) -> i32 {
    if n < 2 {
        return n.max(0);
    }
    let (mut x, mut y) = (n, (n + 1) / 2);
    while y < x {
        x = y;
        y = (x + n / x) / 2;
    }
    x
}

/// A vector of length `speed` from (x, y) toward (tx, ty).
fn toward(x: i32, y: i32, tx: i32, ty: i32, speed: i32) -> (i32, i32) {
    let (dx, dy) = (tx - x, ty - y);
    let d = isqrt(dx * dx + dy * dy);
    if d == 0 {
        (0, 0)
    } else {
        (dx * speed / d, dy * speed / d)
    }
}

/// The facing (0 N, 1 E, 2 S, 3 W) nearest a direction.
fn axis_face(dx: i32, dy: i32) -> Option<u8> {
    if dx == 0 && dy == 0 {
        None
    } else if dx.abs() >= dy.abs() {
        Some(if dx > 0 { 1 } else { 3 })
    } else {
        Some(if dy > 0 { 2 } else { 0 })
    }
}

pub fn facing_vec(f: u8) -> (i32, i32) {
    [(0, -1), (1, 0), (0, 1), (-1, 0)][(f & 3) as usize]
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Doing {
    Idle,
    RunIn,
    RunOut,
    StandOff,
    Flee,
    Mine,
    Haul,
    Guard,
    Hold,
}

impl Doing {
    pub fn name(self) -> &'static str {
        match self {
            Doing::Idle => "IDLE",
            Doing::RunIn => "ATTACK RUN IN",
            Doing::RunOut => "ATTACK RUN OUT",
            Doing::StandOff => "HOLDING AT RANGE",
            Doing::Flee => "FLEEING HOME",
            Doing::Mine => "MINING",
            Doing::Haul => "HAULING HOME",
            Doing::Guard => "ON GUARD",
            Doing::Hold => "HOLDING",
        }
    }
}

#[derive(Clone, Debug)]
pub struct Ship {
    pub team: usize,
    pub slot: usize,
    pub name: String,
    pub order: Order,
    pub shape: Shape,
    /// Stats from the pixels still alive, so shooting off engines slows a ship.
    pub st: Stats,
    pub x: i32,
    pub y: i32,
    /// Sub-pixel remainders (quarter pixels) for the integer DDA; positions stay whole pixels.
    sx: i32,
    sy: i32,
    /// Velocity in quarter pixels per tick.
    pub vx: i32,
    pub vy: i32,
    acc: i32,
    pub facing: u8,
    turn_t: i32,
    pub hp: Vec<u8>,
    cool: Vec<i32>,
    pub alive: bool,
    pub doing: Doing,
    outbound: bool,
    run_t: i32,
    pub home: (i32, i32),
    pub carry: i32,
    hauling: bool,
    tpx: Option<(i32, i32)>,
    chew: i32,
    pub last_hit: u32,
    pub target: Option<usize>,
    pub goal: Option<(i32, i32)>,
    pub trail: Vec<(i32, i32)>,
    dirty: bool,
    /// The target pixel (relative to the target's centre) a lance ship is lining up on.
    aim_off: (i32, i32),
    aim_for: Option<usize>,
    aim_t: i32,
}

impl Ship {
    pub fn dims(&self) -> (i32, i32) {
        self.shape.dims(self.facing)
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
    pub fn cell_xy(&self, i: usize) -> (i32, i32) {
        self.shape.cell_xy(self.x, self.y, self.facing, i)
    }
    pub fn cell_at(&self, px: i32, py: i32) -> Option<usize> {
        self.shape.cell_at(self.x, self.y, self.facing, px, py)
    }
    pub fn alive_cells(&self) -> Vec<Option<Part>> {
        self.shape.cells.iter().zip(&self.hp).map(|(c, &h)| if h > 0 { *c } else { None }).collect()
    }
    pub fn hp_now(&self) -> i32 {
        self.hp.iter().map(|&h| h as i32).sum()
    }
    pub fn hp_full(&self) -> i32 {
        self.shape.cells.iter().flatten().map(|p| p.hp() as i32).sum()
    }
    pub fn speed_px_s(&self) -> i32 {
        isqrt(self.vx * self.vx + self.vy * self.vy) * 15 / 4
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Shot {
    pub team: usize,
    pub line: Line,
    pub left: i32,
    pub speed: i32,
    pub pen: i32,
    pub lance: bool,
    pub prev: (i32, i32),
    /// Counted once in the shooter's hits, however many pixels it punches.
    connected: bool,
}

#[derive(Clone, Copy, Debug)]
pub struct Debris {
    pub x: i32,
    pub y: i32,
    vx: i32,
    vy: i32,
    drift: i32,
    taken: bool,
}

#[derive(Clone, Copy, Debug, Default)]
pub struct Score {
    pub placed: i32,
    pub lost: i32,
    pub px_lost: i32,
    pub mined: i32,
    pub fired: i32,
    pub hits: i32,
}

#[derive(Clone, Copy, Debug)]
struct Snap {
    idx: usize,
    team: usize,
    x: i32,
    y: i32,
    vx: i32,
    vy: i32,
    half: i32,
}

/// What a brain wants this tick.
struct Plan {
    dv: (i32, i32),
    face: Option<u8>,
    doing: Doing,
    target: Option<usize>,
    goal: Option<(i32, i32)>,
}

pub struct Arena {
    pub tick: u32,
    pub ships: Vec<Ship>,
    pub shots: Vec<Shot>,
    pub debris: Vec<Debris>,
    pub rock: Vec<bool>,
    rock_px: Vec<(i32, i32)>,
    pub stars: Vec<(i32, i32, u8)>,
    rng: Rng,
    pub score: [Score; 2],
    /// Some(Some(team)) when a team won, Some(None) for a draw.
    pub over: Option<Option<usize>>,
    started: [bool; 2],
}

impl Arena {
    /// Always the same arena, so the same setup replays the same fight.
    pub fn new() -> Arena {
        let mut a = Arena {
            tick: 0,
            ships: Vec::new(),
            shots: Vec::new(),
            debris: Vec::new(),
            rock: vec![false; (AW * AH) as usize],
            rock_px: Vec::new(),
            stars: Vec::new(),
            rng: Rng::new(1),
            score: [Score::default(); 2],
            over: None,
            started: [false; 2],
        };
        for (cx, cy, r) in [(AW / 2, AH / 2, 14), (AW / 2, 58, 9), (AW / 2, AH - 59, 9)] {
            for y in -r..=r {
                for x in -r..=r {
                    let d = dist2(x, y, 0, 0);
                    let edge = d > (r - 2) * (r - 2);
                    if d <= r * r && !(edge && a.rng.chance(35)) {
                        a.rock[((cy + y) * AW + cx + x) as usize] = true;
                        a.rock_px.push((cx + x, cy + y));
                    }
                }
            }
        }
        for _ in 0..90 {
            let (x, y) = (a.rng.range(0, AW - 1), a.rng.range(0, AH - 1));
            let c = if a.rng.chance(15) { 2 } else if a.rng.chance(40) { 1 } else { 0 };
            a.stars.push((x, y, c));
        }
        a
    }

    pub fn rock_at(&self, x: i32, y: i32) -> bool {
        (0..AW).contains(&x) && (0..AH).contains(&y) && self.rock[(y * AW + x) as usize]
    }

    /// Places a ship; refuses a design with no core.
    pub fn add(&mut self, team: usize, slot: usize, d: &Design, x: i32, y: i32) -> bool {
        let st = d.stats();
        if st.cores == 0 {
            return false;
        }
        let shape = d.shape();
        let n = shape.cells.len();
        let hp = shape.cells.iter().map(|c| c.map_or(0, |p| p.hp())).collect();
        // Stagger the guns so a broadside ripples instead of firing as one.
        let cool = (0..n).map(|i| (i as i32 * 7) % FLAK_PERIOD).collect();
        let mut s = Ship {
            team,
            slot,
            name: d.name.clone(),
            order: d.order,
            shape,
            st,
            x,
            y,
            sx: 0,
            sy: 0,
            vx: 0,
            vy: 0,
            acc: 0,
            facing: if team == 0 { 1 } else { 3 },
            turn_t: 0,
            hp,
            cool,
            alive: true,
            doing: Doing::Idle,
            outbound: false,
            run_t: 0,
            home: (x, y),
            carry: 0,
            hauling: false,
            tpx: None,
            chew: 0,
            last_hit: 0,
            target: None,
            goal: None,
            trail: Vec::new(),
            dirty: false,
            aim_off: (0, 0),
            aim_for: None,
            aim_t: 0,
        };
        let h = s.half();
        s.x = x.clamp(h, AW - 1 - h);
        s.y = y.clamp(h, AH - 1 - h);
        s.home = (s.x, s.y);
        self.ships.push(s);
        self.score[team].placed += 1;
        true
    }

    /// The living ship under a point (with a pixel of slack).
    pub fn ship_at(&self, x: i32, y: i32) -> Option<usize> {
        self.ships
            .iter()
            .enumerate()
            .filter(|(_, s)| {
                let (l, t, r, b) = s.bbox();
                s.alive && x >= l - 2 && y >= t - 2 && x <= r + 2 && y <= b + 2
            })
            .min_by_key(|(_, s)| dist2(s.x, s.y, x, y))
            .map(|(i, _)| i)
    }

    pub fn alive(&self, team: usize) -> usize {
        self.ships.iter().filter(|s| s.alive && s.team == team).count()
    }

    pub fn step(&mut self) {
        if self.over.is_some() {
            return;
        }
        if self.tick == 0 {
            self.started = [self.alive(0) > 0, self.alive(1) > 0];
        }
        self.tick += 1;
        let snap: Vec<Snap> = self
            .ships
            .iter()
            .enumerate()
            .filter(|(_, s)| s.alive)
            .map(|(idx, s)| Snap { idx, team: s.team, x: s.x, y: s.y, vx: s.vx, vy: s.vy, half: s.half() })
            .collect();
        let n = self.ships.len();
        for i in 0..n {
            if self.ships[i].alive {
                let plan = self.brain(i, &snap);
                self.fly(i, plan);
            }
        }
        for i in 0..n {
            if self.ships[i].alive {
                self.weapons(i, &snap);
            }
        }
        self.shots_step();
        self.debris_step();
        self.deaths();
        for s in self.ships.iter_mut().filter(|s| s.alive) {
            if s.dirty {
                s.dirty = false;
                s.st = stats_of(&s.alive_cells());
            }
            if self.tick % 2 == 0 {
                s.trail.push((s.x, s.y));
                if s.trail.len() > 10 {
                    s.trail.remove(0);
                }
            }
        }
        let (a, b) = (self.alive(0), self.alive(1));
        if self.started[0] && self.started[1] {
            if a == 0 && b == 0 {
                self.over = Some(None);
            } else if a == 0 {
                self.over = Some(Some(1));
            } else if b == 0 {
                self.over = Some(Some(0));
            }
        }
        if self.over.is_none() && self.tick >= TIME_LIMIT {
            self.over = Some(None);
        }
    }

    // ----- Layer 1: the order picks where to go -----

    fn brain(&mut self, i: usize, snap: &[Snap]) -> Plan {
        let (team, x, y, order, home, st, last_hit) = {
            let s = &self.ships[i];
            (s.team, s.x, s.y, s.order, s.home, s.st, s.last_hit)
        };
        let armed = st.flak + st.lance > 0;
        let hurt = last_hit > 0 && self.tick.saturating_sub(last_hit) < SCARED;
        let station = STATIONS[team];
        let nearest = |maxd: i32, from: (i32, i32)| {
            snap.iter().filter(|e| e.team != team && cheb(e.x, e.y, from.0, from.1) <= maxd).min_by_key(|e| dist2(e.x, e.y, x, y)).copied()
        };
        let mut plan = match order {
            Order::Hunt => {
                if !armed {
                    if hurt {
                        self.flee(i, station)
                    } else {
                        self.go(i, home, Doing::Idle)
                    }
                } else {
                    match nearest(i32::MAX / 4, (x, y)) {
                        Some(e) => self.engage(i, &e, snap),
                        None => self.go(i, home, Doing::Idle),
                    }
                }
            }
            Order::Guard => match nearest(GUARD_RADIUS, home) {
                Some(e) if armed && cheb(x, y, home.0, home.1) <= GUARD_RADIUS + 50 => self.engage(i, &e, snap),
                _ if !armed && hurt => self.flee(i, station),
                _ => self.go(i, home, Doing::Guard),
            },
            Order::Harvest => {
                if !armed && hurt {
                    self.flee(i, station)
                } else {
                    self.harvest(i)
                }
            }
            Order::Hold => {
                let mut p = self.go(i, home, Doing::Hold);
                if st.lance > 0 {
                    if let Some(e) = nearest(LANCE_RANGE + 10, (x, y)) {
                        p.face = axis_face(e.x - x, e.y - y);
                        p.target = Some(e.idx);
                    }
                } else {
                    p.face = None;
                }
                p
            }
        };
        // Friends keep a pixel or two apart instead of stacking into one sprite.
        if st.thrust > 0 {
            let h = self.ships[i].half();
            for f in snap.iter().filter(|f| f.team == team && f.idx != i && cheb(f.x, f.y, x, y) < f.half + h + 2) {
                let (ax, ay) = if (f.x, f.y) == (x, y) { (if i < f.idx { -4 } else { 4 }, 0) } else { toward(f.x, f.y, x, y, 4) };
                plan.dv.0 += ax;
                plan.dv.1 += ay;
            }
        }
        plan
    }

    /// Arrive at a point, braking in time for this ship's thrust and weight.
    fn arrive(&self, i: usize, gx: i32, gy: i32) -> (i32, i32) {
        let s = &self.ships[i];
        let d = isqrt(dist2(s.x, s.y, gx, gy));
        let brake = isqrt(8 * s.st.thrust * d / s.st.mass.max(1));
        toward(s.x, s.y, gx, gy, s.st.vmax.min(brake).min(d * 4))
    }

    fn go(&self, i: usize, (gx, gy): (i32, i32), doing: Doing) -> Plan {
        let dv = self.arrive(i, gx, gy);
        let face = if dv.0.abs() + dv.1.abs() > 2 { axis_face(dv.0, dv.1) } else { None };
        Plan { dv, face, doing, target: None, goal: Some((gx, gy)) }
    }

    fn flee(&self, i: usize, (gx, gy): (i32, i32)) -> Plan {
        let s = &self.ships[i];
        let dv = toward(s.x, s.y, gx, gy, s.st.vmax);
        Plan { dv, face: axis_face(dv.0, dv.1), doing: Doing::Flee, target: None, goal: Some((gx, gy)) }
    }

    // ----- Layer 3: mass and thrust decide how to fly -----

    fn engage(&mut self, i: usize, e: &Snap, snap: &[Snap]) -> Plan {
        match self.ships[i].st.flight {
            Flight::Runs => self.attack_run(i, e),
            Flight::StandOff => self.stand_off(i, e, snap),
            Flight::Still => Plan { dv: (0, 0), face: None, doing: Doing::Hold, target: Some(e.idx), goal: None },
        }
    }

    /// Dive at the target (leading it), fly through, keep going a little, then come about.
    fn attack_run(&mut self, i: usize, e: &Snap) -> Plan {
        let (x, y, vx, vy, vmax) = {
            let s = &self.ships[i];
            (s.x, s.y, s.vx, s.vy, s.st.vmax)
        };
        let d = isqrt(dist2(x, y, e.x, e.y));
        let t = d * 4 / vmax.max(1);
        let (tx, ty) = ((e.x + e.vx * t / 4).clamp(0, AW - 1), (e.y + e.vy * t / 4).clamp(0, AH - 1));
        let s = &mut self.ships[i];
        s.run_t += 1;
        if !s.outbound {
            let dv = toward(x, y, tx, ty, vmax);
            let closing = (e.x - x) * vx + (e.y - y) * vy;
            if d <= e.half + 6 || (d < 36 && closing < 0 && s.run_t > 8) {
                s.outbound = true;
                s.run_t = 0;
            }
            Plan { dv, face: axis_face(dv.0, dv.1), doing: Doing::RunIn, target: Some(e.idx), goal: Some((tx, ty)) }
        } else {
            let dv = if vx == 0 && vy == 0 { toward(e.x, e.y, x, y, vmax) } else { toward(0, 0, vx, vy, vmax) };
            let near_wall = x < 16 || y < 16 || x > AW - 17 || y > AH - 17;
            if near_wall || (s.run_t > 8 && d > 48) || s.run_t > 30 {
                s.outbound = false;
                s.run_t = 0;
            }
            Plan { dv, face: axis_face(dv.0, dv.1), doing: Doing::RunOut, target: Some(e.idx), goal: None }
        }
    }

    /// Hold at weapon range. Ships with lances line up on the target's row or column so the lances
    /// bear (they only fire straight ahead); flak-only ships close to flak range from wherever they are.
    fn stand_off(&mut self, i: usize, e: &Snap, snap: &[Snap]) -> Plan {
        let (x, y, h, st, team) = {
            let s = &self.ships[i];
            (s.x, s.y, s.half(), s.st, s.team)
        };
        let pref = if st.lance > 0 { LANCE_RANGE * 3 / 4 } else { FLAK_RANGE * 2 / 3 } + e.half;
        let inside = |(px, py): (i32, i32)| px >= h + 2 && py >= h + 2 && px <= AW - 3 - h && py <= AH - 3 - h;
        // Spots in line with the target, at a few distances; a spot a friend already holds costs
        // extra, so a group spreads into a firing line instead of stacking (own shots pass through
        // own ships, so a line behind a line still fires).
        let crowd = |(px, py): (i32, i32)| snap.iter().filter(|f| f.team == team && f.idx != i && cheb(f.x, f.y, px, py) <= f.half + h + 2).count() as i32;
        let lined_up = if st.lance > 0 {
            // Line the lance up on one surviving pixel of the target, picked at random and
            // re-picked after every shot, so a group works across the hull instead of drilling
            // one tunnel that later shots fly straight through.
            let stale = {
                let s = &self.ships[i];
                let ts = &self.ships[e.idx];
                s.aim_for != Some(e.idx) || s.aim_t <= 0 || ts.cell_at(e.x + s.aim_off.0, e.y + s.aim_off.1).map_or(true, |c| ts.hp[c] == 0)
            };
            if stale {
                let (px, py) = self.random_cell(e.idx);
                let s = &mut self.ships[i];
                s.aim_off = (px - e.x, py - e.y);
                s.aim_for = Some(e.idx);
                s.aim_t = LANCE_PERIOD;
            }
            let s = &mut self.ships[i];
            s.aim_t -= 1;
            let aim = s.aim_off;
            let lance = (0..s.shape.cells.len()).find(|&c| s.shape.cells[c] == Some(Part::Lance) && s.hp[c] > 0);
            // Where the lance pixel sits relative to the ship's centre when facing `f`.
            let lance_off = |f: u8| lance.map_or((0, 0), |c| s.shape.cell_xy(0, 0, f, c));
            let mut spots = Vec::new();
            for d in [pref, pref - 14, pref + 14] {
                let (ew, we) = (lance_off(3), lance_off(1));
                let (so, no) = (lance_off(0), lance_off(2));
                spots.push((e.x + d, e.y + aim.1 - ew.1));
                spots.push((e.x - d, e.y + aim.1 - we.1));
                spots.push((e.x + aim.0 - so.0, e.y + d));
                spots.push((e.x + aim.0 - no.0, e.y - d));
            }
            spots.into_iter().filter(|&p| inside(p)).min_by_key(|&p| dist2(p.0, p.1, x, y) + crowd(p) * 20_000)
        } else {
            None
        };
        let point = lined_up.unwrap_or_else(|| {
            let (ux, uy) = toward(e.x, e.y, x, y, 64);
            let (ux, uy) = if (ux, uy) == (0, 0) { (-64, 0) } else { (ux, uy) };
            ((e.x + ux * pref / 64).clamp(h + 2, AW - 3 - h), (e.y + uy * pref / 64).clamp(h + 2, AH - 3 - h))
        });
        let dv = self.arrive(i, point.0, point.1);
        Plan { dv, face: axis_face(e.x - x, e.y - y), doing: Doing::StandOff, target: Some(e.idx), goal: Some(point) }
    }

    fn harvest(&mut self, i: usize) -> Plan {
        let (team, x, y, carry, cap, home) = {
            let s = &self.ships[i];
            (s.team, s.x, s.y, s.carry, s.st.scoops * SCOOP_CARRY, s.home)
        };
        if cap == 0 {
            return self.go(i, home, Doing::Idle);
        }
        let station = STATIONS[team];
        if self.ships[i].hauling || carry >= cap {
            self.ships[i].hauling = true;
            if cheb(x, y, station.0, station.1) <= 10 {
                self.score[team].mined += carry;
                let s = &mut self.ships[i];
                s.carry = 0;
                s.hauling = false;
            }
            let mut p = self.flee(i, station);
            p.doing = Doing::Haul;
            return p;
        }
        if let Some(d) = self.debris.iter().filter(|d| !d.taken && cheb(d.x, d.y, x, y) <= 30).min_by_key(|d| dist2(d.x, d.y, x, y)) {
            let (dx, dy) = (d.x, d.y);
            return self.go(i, (dx, dy), Doing::Mine);
        }
        let valid = self.ships[i].tpx.map_or(false, |(px, py)| self.rock_at(px, py));
        if !valid {
            self.ships[i].tpx = self.rock_px.iter().copied().filter(|&(px, py)| self.rock_at(px, py)).min_by_key(|&(px, py)| dist2(px, py, x, y));
        }
        match self.ships[i].tpx {
            Some((px, py)) if cheb(x, y, px, py) <= 2 => {
                let scoops = self.ships[i].st.scoops;
                let s = &mut self.ships[i];
                s.chew += 1;
                if s.chew >= if scoops >= 2 { 3 } else { 6 } {
                    s.chew = 0;
                    s.tpx = None;
                    s.carry += 1;
                    self.rock[(py * AW + px) as usize] = false;
                }
                Plan { dv: (0, 0), face: None, doing: Doing::Mine, target: None, goal: Some((px, py)) }
            }
            Some(p) => self.go(i, p, Doing::Mine),
            None => {
                if carry > 0 {
                    self.ships[i].hauling = true;
                }
                self.go(i, home, Doing::Idle)
            }
        }
    }

    /// Integer inertia: thrust over weight buys a quarter-pixel of velocity change per tick in
    /// whole steps; facing turns a quarter at a time, as fast as the weight allows.
    fn fly(&mut self, i: usize, p: Plan) {
        let s = &mut self.ships[i];
        s.doing = p.doing;
        s.target = p.target;
        s.goal = p.goal;
        let st = s.st;
        if st.thrust > 0 {
            s.acc += st.thrust;
            let budget = (s.acc / st.mass.max(1)).min(8);
            s.acc %= st.mass.max(1);
            s.vx += (p.dv.0 - s.vx).clamp(-budget, budget);
            s.vy += (p.dv.1 - s.vy).clamp(-budget, budget);
        } else {
            s.vx = 0;
            s.vy = 0;
        }
        let v = isqrt(s.vx * s.vx + s.vy * s.vy);
        if v > st.vmax && v > 0 {
            s.vx = s.vx * st.vmax / v;
            s.vy = s.vy * st.vmax / v;
        }
        match p.face {
            Some(f) if f != s.facing && st.turn > 0 => {
                s.turn_t += 1;
                if s.turn_t >= st.turn {
                    s.turn_t = 0;
                    s.facing = if (f + 4 - s.facing) % 4 == 3 { (s.facing + 3) % 4 } else { (s.facing + 1) % 4 };
                }
            }
            _ => s.turn_t = 0,
        }
        s.sx += s.vx;
        s.sy += s.vy;
        let (dx, dy) = (s.sx.div_euclid(4), s.sy.div_euclid(4));
        s.sx = s.sx.rem_euclid(4);
        s.sy = s.sy.rem_euclid(4);
        let h = s.half();
        let (nx, ny) = (s.x + dx, s.y + dy);
        let (cx, cy) = (nx.clamp(h, AW - 1 - h), ny.clamp(h, AH - 1 - h));
        if cx != nx {
            s.vx = 0;
            s.sx = 0;
        }
        if cy != ny {
            s.vy = 0;
            s.sy = 0;
        }
        s.x = cx;
        s.y = cy;
    }

    // ----- Layer 2: the parts fire on reflex -----

    fn random_cell(&mut self, idx: usize) -> (i32, i32) {
        let alive: Vec<usize> = (0..self.ships[idx].hp.len()).filter(|&c| self.ships[idx].hp[c] > 0).collect();
        if alive.is_empty() {
            return (self.ships[idx].x, self.ships[idx].y);
        }
        let c = alive[self.rng.below(alive.len() as u32) as usize];
        self.ships[idx].cell_xy(c)
    }

    fn weapons(&mut self, i: usize, snap: &[Snap]) {
        let team = self.ships[i].team;
        for c in 0..self.ships[i].shape.cells.len() {
            let part = self.ships[i].shape.cells[c];
            if self.ships[i].hp[c] == 0 || !matches!(part, Some(Part::Flak) | Some(Part::Lance)) {
                continue;
            }
            if self.ships[i].cool[c] > 0 {
                self.ships[i].cool[c] -= 1;
                continue;
            }
            let (gx, gy) = self.ships[i].cell_xy(c);
            if part == Some(Part::Flak) {
                let tgt = snap.iter().filter(|e| e.team != team && cheb(e.x, e.y, gx, gy) <= FLAK_RANGE + e.half).min_by_key(|e| dist2(e.x, e.y, gx, gy)).copied();
                if let Some(e) = tgt {
                    // Aim at a random surviving pixel (never the centre: that drills a tunnel later
                    // shots fly straight through), led by the target's velocity.
                    let (px, py) = self.random_cell(e.idx);
                    let t = cheb(px, py, gx, gy) / FLAK_SPEED;
                    let mut aim = (px + e.vx * t / 4, py + e.vy * t / 4);
                    if aim == (gx, gy) {
                        aim = (px, py + 1);
                    }
                    self.shots.push(Shot { team, line: Line::new(gx, gy, aim.0, aim.1), left: FLAK_RANGE + 12, speed: FLAK_SPEED, pen: 1, lance: false, prev: (gx, gy), connected: false });
                    self.ships[i].cool[c] = FLAK_PERIOD + self.rng.below(4) as i32;
                    self.score[team].fired += 1;
                }
            } else {
                // A lance only fires straight ahead, and only when its line would cross a pixel
                // still standing where the slow shot will meet it.
                let (dx, dy) = facing_vec(self.ships[i].facing);
                let ships = &self.ships;
                let in_line = snap.iter().any(|e| {
                    if e.team == team {
                        return false;
                    }
                    let along = (e.x - gx) * dx + (e.y - gy) * dy;
                    if along <= 0 || along > LANCE_RANGE + e.half {
                        return false;
                    }
                    let t = along / LANCE_SPEED;
                    let (ox, oy) = (e.vx * t / 4, e.vy * t / 4);
                    let ts = &ships[e.idx];
                    ((along - e.half - 1).max(1)..=along + e.half + 1).any(|k| ts.cell_at(gx + dx * k - ox, gy + dy * k - oy).map_or(false, |c| ts.hp[c] > 0))
                });
                if in_line {
                    self.ships[i].aim_t = 0;
                    self.shots.push(Shot {
                        team,
                        line: Line::new(gx, gy, gx + dx * 400, gy + dy * 400),
                        left: LANCE_RANGE + 12,
                        speed: LANCE_SPEED,
                        pen: LANCE_PEN,
                        lance: true,
                        prev: (gx, gy),
                        connected: false,
                    });
                    self.ships[i].cool[c] = LANCE_PERIOD;
                    self.score[team].fired += 1;
                }
            }
        }
    }

    fn shots_step(&mut self) {
        let mut shots = std::mem::take(&mut self.shots);
        let boxes: Vec<(usize, usize, (i32, i32, i32, i32))> = self.ships.iter().enumerate().filter(|(_, s)| s.alive).map(|(i, s)| (i, s.team, s.bbox())).collect();
        shots.retain_mut(|sh| {
            for _ in 0..sh.speed {
                sh.prev = (sh.line.x, sh.line.y);
                sh.line.step();
                sh.left -= 1;
                let (x, y) = (sh.line.x, sh.line.y);
                if sh.left <= 0 || x < 0 || y < 0 || x >= AW || y >= AH {
                    return false;
                }
                for &(id, team, (l, t, r, b)) in &boxes {
                    if team == sh.team || x < l || y < t || x > r || y > b {
                        continue;
                    }
                    let Some(c) = self.ships[id].cell_at(x, y) else { continue };
                    while self.ships[id].hp[c] > 0 && sh.pen > 0 {
                        let s = &mut self.ships[id];
                        s.hp[c] -= 1;
                        s.last_hit = self.tick;
                        s.dirty = true;
                        sh.pen -= 1;
                        if !sh.connected {
                            sh.connected = true;
                            self.score[sh.team].hits += 1;
                        }
                        if s.hp[c] == 0 {
                            self.score[team].px_lost += 1;
                            let (vx, vy) = sh.line.dir();
                            let drift = 2 + self.rng.below(4) as i32;
                            self.debris.push(Debris { x, y, vx, vy, drift, taken: false });
                        }
                    }
                    if sh.pen == 0 {
                        return false;
                    }
                }
            }
            true
        });
        self.shots.append(&mut shots);
    }

    fn debris_step(&mut self) {
        for d in &mut self.debris {
            if d.drift > 0 {
                d.x = (d.x + d.vx).clamp(0, AW - 1);
                d.y = (d.y + d.vy).clamp(0, AH - 1);
                d.drift -= 1;
            }
        }
        // Scoops grab loose pixels they brush past, whatever the ship's order.
        for s in self.ships.iter_mut().filter(|s| s.alive && s.st.scoops > 0) {
            let cap = s.st.scoops * SCOOP_CARRY;
            for d in self.debris.iter_mut().filter(|d| !d.taken) {
                if s.carry < cap && cheb(d.x, d.y, s.x, s.y) <= s.half() + 1 {
                    d.taken = true;
                    s.carry += 1;
                }
            }
        }
        self.debris.retain(|d| !d.taken);
        if self.debris.len() > 1500 {
            let n = self.debris.len() - 1500;
            self.debris.drain(0..n);
        }
    }

    /// A ship whose core pixels are all gone bursts into loose pixels.
    fn deaths(&mut self) {
        for i in 0..self.ships.len() {
            let s = &self.ships[i];
            if !s.alive || s.shape.cells.iter().zip(&s.hp).any(|(c, &h)| *c == Some(Part::Core) && h > 0) {
                continue;
            }
            let (team, x, y) = (s.team, s.x, s.y);
            let cells: Vec<(i32, i32)> = (0..s.hp.len()).filter(|&c| s.hp[c] > 0).map(|c| s.cell_xy(c)).collect();
            self.ships[i].alive = false;
            self.score[team].lost += 1;
            self.score[team].px_lost += cells.len() as i32;
            for (px, py) in cells {
                let drift = 2 + self.rng.below(5) as i32;
                self.debris.push(Debris { x: px, y: py, vx: (px - x).signum(), vy: (py - y).signum(), drift, taken: false });
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fight(blue: &[(usize, i32, i32)], red: &[(usize, i32, i32)]) -> Arena {
        let lib = defaults();
        let mut a = Arena::new();
        for &(slot, x, y) in blue {
            a.add(0, slot, &lib[slot], x, y);
        }
        for &(slot, x, y) in red {
            a.add(1, slot, &lib[slot], x, y);
        }
        while a.over.is_none() {
            a.step();
        }
        a
    }

    #[test]
    fn fights_end() {
        let a = fight(&[(0, 60, 150), (0, 60, 170)], &[(2, 240, 150)]);
        assert!(a.tick < TIME_LIMIT, "two fighters vs a frigate timed out");
    }

    #[test]
    fn heavy_ships_turn_and_accelerate_slower() {
        let lib = defaults();
        let mut a = Arena::new();
        a.add(0, 0, &lib[0], 40, 40);
        a.add(0, 3, &lib[3], 40, 200);
        a.ships[0].order = Order::Hold;
        a.ships[1].order = Order::Hold;
        a.ships[0].home = (200, 40);
        a.ships[1].home = (200, 200);
        for _ in 0..15 {
            a.step();
        }
        assert!(a.ships[0].x - 40 > 3 * (a.ships[1].x - 40), "fighter {} capital {}", a.ships[0].x, a.ships[1].x);
    }

    #[test]
    fn an_armed_miner_shoots_back() {
        let a = fight(&[(0, 120, 150)], &[(5, 170, 150)]);
        assert!(a.score[1].hits > 0, "the gunminer never hit the fighter");
    }
}
