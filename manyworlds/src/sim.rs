//! The rules of one leg as a deterministic integer simulation, with no graphics or audio.
//!
//! One tick is one 15 FPS frame. Rail is a 4-bit arm mask per tile (N, E, S, W). A tile with
//! three arms is a switch: its two lit arms are the route and the third arm is closed. A tile
//! with four arms is a crossing that only goes straight. The train moves one tile every
//! `tile_ticks` ticks (`express_ticks` while express is held) and never reverses; its cars
//! follow the engine's trail. DESIGN.md "Rules" describes the same rules for players.

use crate::geom::*;
use crate::map::Map;
use crate::rng::Rng;
use std::collections::VecDeque;

/// Which passengers can get off at a station.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Unload {
    /// Anyone aboard whose shape matches.
    Any,
    /// Only the rear car, then the next rear car, while they match: boarding order decides
    /// delivery order, last on first off (a slip coach dropped from a moving train).
    Rear,
    /// Only the front car, then the next, while they match: first on, first off.
    Front,
}

/// Take the passengers who get off at a station of `shape` out of `cars`; returns how many.
pub fn unload(cars: &mut Vec<u8>, shape: u8, rule: Unload) -> usize {
    let before = cars.len();
    match rule {
        Unload::Any => cars.retain(|&c| c != shape),
        Unload::Rear => {
            while cars.last() == Some(&shape) {
                cars.pop();
            }
        }
        Unload::Front => {
            let k = cars.iter().take_while(|&&c| c == shape).count();
            cars.drain(..k);
        }
    }
    before - cars.len()
}

#[derive(Clone, Debug)]
pub struct Params {
    /// Length of the leg: the track's length.
    pub ticks: i32,
    /// When the terminus opens: the start of the fade-out.
    pub fade: i32,
    pub tile_ticks: i32,
    pub express_ticks: i32,
    /// Most cars the engine can pull.
    pub capacity: usize,
    /// Rail pieces for the leg; one piece per tile that has rail.
    pub stock: i32,
    /// Ticks before an erased piece can be drawn again.
    pub erase_delay: i32,
    /// Ticks the train stands still after running into its own cars.
    pub crash_ticks: i32,
    /// Ticks between new passengers at one station, at the start and at the end of the leg.
    pub spawn_start: i32,
    pub spawn_end: i32,
    /// Passengers waiting at a station the moment it opens.
    pub opening_burst: usize,
    pub queue_cap: usize,
    /// Deliveries per step of the streak multiplier, and its ceiling.
    pub streak_step: i32,
    pub max_mult: i32,
    /// Arriving at the open terminus scores this times the multiplier.
    pub terminus_bonus: i32,
    /// An express train runs through stations without stopping.
    pub express_skips: bool,
    /// The n-th passenger off at one stop scores n (times the multiplier), so unloading
    /// several at once pays more than one at a time.
    pub batch_scoring: bool,
    /// Every station gets its own shape, so each passenger has exactly one destination.
    pub distinct_stations: bool,
    pub unload: Unload,
    /// Plain rail the tail leaves behind is lifted and its piece returned, unless the train's
    /// route comes back over it. Junctions stay.
    pub auto_lift: bool,
    /// When positive, the streak also ends if this many ticks pass without a delivery.
    pub combo_window: i32,
}

impl Params {
    /// A leg as long as a track of `seconds`; the fade-out (last eighth) opens the terminus.
    pub fn for_track(seconds: i32) -> Params {
        let ticks = seconds * 15;
        Params {
            ticks,
            fade: ticks - ticks / 8,
            tile_ticks: 9,
            express_ticks: 4,
            capacity: 6,
            stock: 40,
            erase_delay: 0,
            crash_ticks: 30,
            spawn_start: 270,
            spawn_end: 135,
            opening_burst: 2,
            queue_cap: 12,
            streak_step: 4,
            max_mult: 4,
            terminus_bonus: 10,
            express_skips: true,
            batch_scoring: false,
            distinct_stations: false,
            unload: Unload::Rear,
            auto_lift: true,
            combo_window: 0,
        }
    }
}

#[derive(Clone, Debug)]
pub struct Station {
    pub pos: Pos,
    pub shape: u8,
    pub appear: i32,
    /// Waiting passengers, first in line first; each is the shape they want.
    pub queue: VecDeque<u8>,
    pub next_spawn: i32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Ev {
    Deliver,
    /// An express train ran through an open station without stopping.
    Skip,
    Stall,
    Crash,
    Park,
}

#[derive(Clone, Debug, Default)]
pub struct Stats {
    pub score: i32,
    pub delivered: i32,
    pub boarded: i32,
    pub stalls: i32,
    pub stall_ticks: i32,
    pub crashes: i32,
    /// Passengers aboard when the train crashed.
    pub lost: i32,
    pub moves: i32,
    pub express_moves: i32,
    pub drawn: i32,
    pub erased: i32,
    pub flips: i32,
    pub bonus: i32,
    pub best_streak: i32,
    /// Pieces picked up automatically behind the train.
    pub lifted: i32,
}

#[derive(Clone)]
pub struct Sim {
    pub p: Params,
    pub tick: i32,
    pub blocked: Vec<bool>,
    pub mask: Vec<u8>,
    /// For three-arm tiles: the two lit arms.
    pub lit: Vec<u8>,
    /// The depot's rail: always there, never costs or refunds a piece.
    pub fixed: Vec<bool>,
    pub stations: Vec<Station>,
    pub station_at: Vec<Option<u8>>,
    pub terminus: Pos,
    /// Pieces on the board.
    pub used: i32,
    /// Erased pieces not yet returned: the tick each returns.
    pub refunds: VecDeque<i32>,
    /// Tiles under the train, engine first.
    pub body: VecDeque<Pos>,
    /// Tiles the tail has left, most recent first: where cars that have boarded but not yet
    /// pulled onto the line are drawn.
    pub trail: VecDeque<Pos>,
    /// The side of the engine's tile it came in through.
    pub entry: Dir,
    /// Passengers aboard, front car first; each is the shape they want.
    pub cars: Vec<u8>,
    pub progress: i32,
    pub express: bool,
    pub departed: bool,
    pub stalled: bool,
    pub crash_timer: i32,
    pub parked: bool,
    /// Held at a signal: the train waits without it counting as a stall (the tutorial uses it).
    pub hold: bool,
    /// New passengers arrive (the tutorial turns this off while it scripts the queues).
    pub spawning: bool,
    pub streak: i32,
    pub last_delivery: i32,
    dropped_tiles: Vec<Pos>,
    pub rng: Rng,
    pub stats: Stats,
    pub log: Vec<(i32, Ev)>,
}

impl Sim {
    pub fn new(map: &Map, p: Params, seed: u64) -> Sim {
        let mut mask = vec![0u8; NT];
        let mut fixed = vec![false; NT];
        let toward = map
            .buffer
            .dir_to(map.depot)
            .expect("the buffer must touch the depot");
        mask[map.buffer.idx()] |= toward.bit();
        mask[map.depot.idx()] |= toward.opp().bit();
        fixed[map.buffer.idx()] = true;
        fixed[map.depot.idx()] = true;
        let mut station_at = vec![None; NT];
        let stations = map
            .stations
            .iter()
            .enumerate()
            .map(|(i, s)| {
                station_at[s.pos.idx()] = Some(i as u8);
                let appear = s.appear * p.ticks / 1000;
                let shape = if p.distinct_stations {
                    i as u8
                } else {
                    s.shape
                };
                Station {
                    pos: s.pos,
                    shape,
                    appear,
                    queue: VecDeque::new(),
                    next_spawn: appear,
                }
            })
            .collect();
        Sim {
            p,
            tick: 0,
            blocked: map.blocked.clone(),
            mask,
            lit: vec![0; NT],
            fixed,
            stations,
            station_at,
            terminus: map.terminus,
            used: 0,
            refunds: VecDeque::new(),
            body: VecDeque::from([map.depot]),
            trail: VecDeque::from([map.buffer]),
            entry: toward.opp(),
            cars: vec![],
            progress: 0,
            express: false,
            departed: false,
            stalled: false,
            crash_timer: 0,
            parked: false,
            hold: false,
            spawning: true,
            streak: 0,
            last_delivery: 0,
            dropped_tiles: vec![],
            rng: Rng::new(seed),
            stats: Stats::default(),
            log: vec![],
        }
    }

    // ----------------------------------------------------------------------------------------
    // Reading the state

    /// Pieces the player can draw right now.
    pub fn free(&self) -> i32 {
        self.p.stock - self.used - self.refunds.len() as i32
    }

    pub fn engine(&self) -> Pos {
        self.body[0]
    }

    pub fn on_body(&self, t: Pos) -> bool {
        self.body.contains(&t)
    }

    /// The open station on tile `t`, if any.
    pub fn station(&self, t: Pos) -> Option<usize> {
        if !t.ok() {
            return None;
        }
        self.station_at[t.idx()]
            .map(|i| i as usize)
            .filter(|&i| self.tick >= self.stations[i].appear)
    }

    pub fn terminus_open(&self) -> bool {
        self.tick >= self.p.fade
    }

    pub fn mult(&self) -> i32 {
        (1 + self.streak / self.p.streak_step).min(self.p.max_mult)
    }

    pub fn finished(&self) -> bool {
        self.parked || self.tick >= self.p.ticks
    }

    /// Which way a train leaves tile `t` after coming in through side `entry`.
    pub fn exit_of(&self, t: Pos, entry: Dir) -> Option<Dir> {
        let m = self.mask[t.idx()];
        match m.count_ones() {
            4 => Some(entry.opp()),
            3 => {
                let l = self.lit[t.idx()];
                if l & entry.bit() == 0 {
                    None
                } else {
                    arms(l & !entry.bit()).next()
                }
            }
            _ => {
                let rest = m & !entry.bit();
                if rest.count_ones() == 1 {
                    arms(rest).next()
                } else {
                    None
                }
            }
        }
    }

    /// Whether a train can come into tile `n` through side `side`.
    pub fn can_enter(&self, n: Pos, side: Dir) -> bool {
        if !n.ok() {
            return false;
        }
        let m = self.mask[n.idx()];
        if m & side.bit() == 0 {
            return false;
        }
        m.count_ones() != 3 || self.lit[n.idx()] & side.bit() != 0
    }

    /// The tile the engine moves into next, if the rail allows it now. The game highlights it.
    pub fn next_tile(&self) -> Option<Pos> {
        let e = self.engine();
        let d = self.exit_of(e, self.entry)?;
        let n = e.step(d);
        self.can_enter(n, d.opp()).then_some(n)
    }

    // ----------------------------------------------------------------------------------------
    // Player actions

    /// One step of a drag: connect tile `a` to its neighbour `b`. `came` is the side of `a` the
    /// drag arrived through, when the drag passed through `a` rather than starting there.
    /// Fails, changing nothing, off the board, on blocked ground, or without enough pieces.
    pub fn draw_seg(&mut self, came: Option<Dir>, a: Pos, b: Pos) -> bool {
        if !a.ok() || !b.ok() {
            return false;
        }
        let Some(d) = a.dir_to(b) else { return false };
        let (ia, ib) = (a.idx(), b.idx());
        if self.blocked[ia] || self.blocked[ib] {
            return false;
        }
        let cost = (self.mask[ia] == 0) as i32 + (self.mask[ib] == 0) as i32;
        if cost > self.free() {
            return false;
        }
        let fresh_a = self.mask[ia] & d.bit() == 0;
        let fresh_b = self.mask[ib] & d.opp().bit() == 0;
        // A branch drawn off the line the train is following leaves the train's way lit.
        let ways = if fresh_a || fresh_b {
            self.route_ways()
        } else {
            vec![]
        };
        let way = |t: Pos| ways.iter().find(|w| w.0 == t).map(|w| w.1);
        let (keep_a, keep_b) = (way(a), way(b));
        self.used += cost;
        self.stats.drawn += cost;
        self.mask[ia] |= d.bit();
        self.mask[ib] |= d.opp().bit();
        match came.filter(|&c| c != d && self.mask[ia] & c.bit() != 0) {
            // A drag through a switch lights the way it went.
            Some(c) if self.mask[ia].count_ones() == 3 => self.lit[ia] = c.bit() | d.bit(),
            _ => self.settle_new(a, fresh_a.then_some(d), keep_a),
        }
        self.settle_new(b, fresh_b.then_some(d.opp()), keep_b);
        true
    }

    /// Remove all rail from tile `t`. Not under the train, and never the depot's rail.
    pub fn erase(&mut self, t: Pos) -> bool {
        if !t.ok() {
            return false;
        }
        let i = t.idx();
        if self.fixed[i] || self.mask[i] == 0 || self.on_body(t) {
            return false;
        }
        for d in arms(self.mask[i]) {
            let n = t.step(d);
            let j = n.idx();
            if n == self.engine() && self.entry == d.opp() {
                continue; // the engine keeps the arm it came in by
            }
            if self.mask[j] == 0 {
                continue; // already empty: an arm the engine kept pointed here
            }
            self.mask[j] &= !d.opp().bit();
            self.settle(n, None);
            if self.mask[j] == 0 && !self.fixed[j] {
                self.refund();
            }
        }
        self.mask[i] = 0;
        self.lit[i] = 0;
        self.refund();
        self.stats.erased += 1;
        true
    }

    /// Cycle a switch's lit pair. Points are locked while the train is on them.
    pub fn flip(&mut self, t: Pos) -> bool {
        if !t.ok() {
            return false;
        }
        let i = t.idx();
        let m = self.mask[i];
        if m.count_ones() != 3 || self.on_body(t) {
            return false;
        }
        let a: Vec<u8> = arms(m).map(Dir::bit).collect();
        let pairs = [a[0] | a[1], a[1] | a[2], a[0] | a[2]];
        let k = pairs.iter().position(|&p| p == self.lit[i]).unwrap_or(2);
        self.lit[i] = pairs[(k + 1) % 3];
        self.stats.flips += 1;
        true
    }

    pub fn set_express(&mut self, on: bool) {
        self.express = on;
    }

    // ----------------------------------------------------------------------------------------
    // Time

    pub fn step(&mut self) {
        if self.finished() {
            return;
        }
        while self.refunds.front().is_some_and(|&r| r <= self.tick) {
            self.refunds.pop_front();
        }
        self.spawn();
        self.drive();
        self.tick += 1;
    }

    fn refund(&mut self) {
        self.used -= 1;
        if self.p.erase_delay > 0 {
            self.refunds.push_back(self.tick + self.p.erase_delay);
        }
    }

    /// Keep tile `t`'s lit pair valid after its arms change. `fresh` is an arm a drag just added
    /// that started or ended here: a new switch lights it.
    /// Each tile of the route ahead, from the engine's own, with the pair of arms the train
    /// uses there.
    fn route_ways(&self) -> Vec<(Pos, u8)> {
        let mut out = vec![];
        let mut seen = vec![0u8; NT];
        let (mut cur, mut e) = (self.engine(), self.entry);
        while let Some(d) = self.exit_of(cur, e) {
            out.push((cur, e.bit() | d.bit()));
            let n = cur.step(d);
            if !self.can_enter(n, d.opp()) || seen[n.idx()] & d.opp().bit() != 0 {
                break;
            }
            seen[n.idx()] |= d.opp().bit();
            cur = n;
            e = d.opp();
        }
        out
    }

    fn settle_new(&mut self, t: Pos, fresh: Option<Dir>, keep: Option<u8>) {
        let i = t.idx();
        if fresh.is_some() && self.mask[i].count_ones() == 3 {
            if let Some(k) = keep.filter(|k| k & !self.mask[i] == 0) {
                self.lit[i] = k;
                return;
            }
        }
        self.settle(t, fresh);
    }

    fn settle(&mut self, t: Pos, fresh: Option<Dir>) {
        let i = t.idx();
        let m = self.mask[i];
        if m.count_ones() != 3 {
            self.lit[i] = 0;
            return;
        }
        let engine_entry = (self.body[0] == t && m & self.entry.bit() != 0).then_some(self.entry);
        if let Some(c) = fresh {
            let partner = engine_entry
                .filter(|&e| e != c)
                .or(Some(c.opp()).filter(|o| m & o.bit() != 0))
                .unwrap_or_else(|| arms(m & !c.bit()).next().unwrap());
            self.lit[i] = c.bit() | partner.bit();
            return;
        }
        let l = self.lit[i];
        if l.count_ones() == 2 && l & !m == 0 {
            return;
        }
        // A three-arm tile always has exactly one straight pair; light it, unless that would
        // shut the engine in.
        let straight = arms(m).find(|d| m & d.opp().bit() != 0).unwrap();
        self.lit[i] = straight.bit() | straight.opp().bit();
        if let Some(e) = engine_entry {
            if self.lit[i] & e.bit() == 0 {
                self.lit[i] = e.bit() | arms(m & !e.bit()).next().unwrap().bit();
            }
        }
    }

    fn spawn(&mut self) {
        if !self.spawning {
            return;
        }
        let t = self.tick;
        let mut shapes = 0u8;
        for s in &self.stations {
            if t >= s.appear {
                shapes |= 1 << s.shape;
            }
        }
        let interval =
            self.p.spawn_start + (self.p.spawn_end - self.p.spawn_start) * t / self.p.ticks.max(1);
        for k in 0..self.stations.len() {
            let (appear, next, shape) = (
                self.stations[k].appear,
                self.stations[k].next_spawn,
                self.stations[k].shape,
            );
            if t < appear || t < next {
                continue;
            }
            let others = shapes & !(1u8 << shape);
            let burst = if t == appear { self.p.opening_burst } else { 1 };
            for _ in 0..burst {
                if others == 0 || self.stations[k].queue.len() >= self.p.queue_cap {
                    break;
                }
                let pick = self.rng.below(others.count_ones()) as usize;
                let dest = (0..8u8)
                    .filter(|b| others & (1 << b) != 0)
                    .nth(pick)
                    .unwrap();
                self.stations[k].queue.push_back(dest);
            }
            let jitter = interval * 3 / 10;
            self.stations[k].next_spawn = t + interval + self.rng.range(-jitter, jitter);
        }
    }

    fn drive(&mut self) {
        if self.parked {
            return;
        }
        if self.crash_timer > 0 {
            self.crash_timer -= 1;
            return;
        }
        if self.hold {
            return;
        }
        let tt = if self.express {
            self.p.express_ticks
        } else {
            self.p.tile_ticks
        };
        if self.progress < tt {
            self.progress += 1;
        }
        if self.progress < tt {
            return;
        }
        let e = self.engine();
        let way = self
            .exit_of(e, self.entry)
            .map(|d| (d, e.step(d)))
            .filter(|&(d, n)| self.can_enter(n, d.opp()));
        let Some((d, n)) = way else {
            if self.departed {
                if !self.stalled {
                    self.stalled = true;
                    self.stats.stalls += 1;
                    self.streak = 0;
                    self.log.push((self.tick, Ev::Stall));
                }
                self.stats.stall_ticks += 1;
            }
            return;
        };
        let grow = self.body.len() < 1 + self.cars.len();
        let last = self.body.len() - 1;
        if self
            .body
            .iter()
            .enumerate()
            .any(|(k, &b)| b == n && (grow || k != last))
        {
            self.stats.crashes += 1;
            self.stats.lost += self.cars.len() as i32;
            self.cars.clear();
            self.body.truncate(1);
            self.crash_timer = self.p.crash_ticks;
            self.streak = 0;
            self.progress = 0;
            self.stalled = false;
            self.log.push((self.tick, Ev::Crash));
            return;
        }
        self.body.push_front(n);
        let mut left = vec![];
        if !grow {
            let t = self.body.pop_back().unwrap();
            left.push(t);
            self.remember(t);
        }
        self.entry = d.opp();
        self.progress = 0;
        self.departed = true;
        self.stalled = false;
        self.stats.moves += 1;
        if self.express {
            self.stats.express_moves += 1;
        }
        let before = self.body.len();
        self.arrive(n);
        // Cars that got off shorten the train from the back.
        let dropped = before.saturating_sub(self.body.len());
        left.extend(
            self.dropped_tiles
                .drain(..dropped.min(self.dropped_tiles.len())),
        );
        self.lift(&left);
    }

    /// The tiles of the route ahead of the engine, until it would stop or come round again.
    pub fn route_tiles(&self) -> Vec<Pos> {
        let mut seen = vec![0u8; NT];
        let (mut cur, mut e) = (self.engine(), self.entry);
        let mut out = vec![];
        while let Some(d) = self.exit_of(cur, e) {
            let n = cur.step(d);
            if !self.can_enter(n, d.opp()) || seen[n.idx()] & d.opp().bit() != 0 {
                break;
            }
            seen[n.idx()] |= d.opp().bit();
            out.push(n);
            cur = n;
            e = d.opp();
        }
        out
    }

    fn remember(&mut self, t: Pos) {
        self.trail.push_front(t);
        self.trail.truncate(self.p.capacity + 2);
    }

    /// With `auto_lift`, pick up plain rail from tiles the train has just left.
    fn lift(&mut self, tiles: &[Pos]) {
        if !self.p.auto_lift || tiles.is_empty() {
            return;
        }
        let ahead = self.route_tiles();
        for &t in tiles {
            let m = self.mask[t.idx()];
            if m != 0 && m.count_ones() <= 2 && !ahead.contains(&t) && self.erase(t) {
                self.stats.erased -= 1;
                self.stats.lifted += 1;
            }
        }
    }

    fn arrive(&mut self, n: Pos) {
        if n == self.terminus && self.terminus_open() {
            let b = self.p.terminus_bonus * self.mult();
            self.stats.score += b;
            self.stats.bonus += b;
            self.parked = true;
            self.log.push((self.tick, Ev::Park));
            return;
        }
        let Some(si) = self.station(n) else { return };
        if self.express && self.p.express_skips {
            self.log.push((self.tick, Ev::Skip));
            return;
        }
        let shape = self.stations[si].shape;
        let before = self.cars.len();
        let late = self.p.combo_window > 0 && self.tick - self.last_delivery > self.p.combo_window;
        let mut after = self.cars.clone();
        if unload(&mut after, shape, self.p.unload) > 0 && late {
            self.streak = 0;
        }
        self.cars = after;
        for n in 1..=(before - self.cars.len()) as i32 {
            self.stats.score += self.mult() * if self.p.batch_scoring { n } else { 1 };
            self.streak += 1;
            self.stats.delivered += 1;
            self.log.push((self.tick, Ev::Deliver));
        }
        if self.cars.len() < before {
            self.last_delivery = self.tick;
        }
        self.stats.best_streak = self.stats.best_streak.max(self.streak);
        while self.body.len() > 1 + self.cars.len() {
            let t = self.body.pop_back().unwrap();
            self.dropped_tiles.push(t);
            self.remember(t);
        }
        while self.cars.len() < self.p.capacity {
            let Some(c) = self.stations[si].queue.pop_front() else {
                break;
            };
            self.cars.push(c);
            self.stats.boarded += 1;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::map::parse;

    /// An empty 25 x 17 field with the depot at (1, 8) facing east, plus `extra` rows patched in.
    fn field(extra: &[(usize, &str)], defs: &[(char, u8, i32)]) -> Map {
        let mut rows: Vec<String> = vec![".".repeat(25); 17];
        rows[8] = format!("<@{}", ".".repeat(23));
        rows[16] = format!("{}T", ".".repeat(24));
        for &(y, r) in extra {
            rows[y] = r.to_string();
        }
        let refs: Vec<&str> = rows.iter().map(|s| s.as_str()).collect();
        parse("test", &refs, defs)
    }

    fn quiet() -> Params {
        let mut p = Params::for_track(60);
        p.spawn_start = 1_000_000;
        p.spawn_end = 1_000_000;
        p.opening_burst = 0;
        p.combo_window = 0;
        p
    }

    fn line(sim: &mut Sim, pts: &[(i32, i32)]) {
        for w in pts.windows(2) {
            let (a, b) = (Pos::new(w[0].0, w[0].1), Pos::new(w[1].0, w[1].1));
            assert!(sim.draw_seg(None, a, b), "draw {a:?} -> {b:?}");
        }
    }

    fn run(sim: &mut Sim, ticks: i32) {
        for _ in 0..ticks {
            sim.step();
        }
    }

    #[test]
    fn train_follows_rail_then_stalls_at_the_end() {
        let mut sim = Sim::new(&field(&[], &[]), quiet(), 1);
        run(&mut sim, 30);
        assert_eq!(
            sim.stats.stalls, 0,
            "waiting before departure is not a stall"
        );
        line(&mut sim, &[(1, 8), (2, 8), (3, 8), (4, 8)]);
        assert_eq!(sim.used, 3);
        run(&mut sim, 9 * 3);
        assert_eq!(sim.engine(), Pos::new(4, 8));
        run(&mut sim, 10);
        assert_eq!(sim.stats.stalls, 1);
    }

    #[test]
    fn delivery_scores_with_the_streak_multiplier() {
        let rows = [(8, "<@..a....b...............")];
        let mut sim = Sim::new(&field(&rows, &[('a', 0, 0), ('b', 1, 0)]), quiet(), 1);
        sim.stations[0].queue.extend([1, 1, 1, 1, 1, 1, 1]);
        line(&mut sim, &(1..12).map(|x| (x, 8)).collect::<Vec<_>>());
        run(&mut sim, 9 * 3 + 1);
        assert_eq!(sim.cars.len(), 6, "boards up to capacity");
        assert_eq!(sim.stations[0].queue.len(), 1);
        run(&mut sim, 9 * 5);
        assert_eq!(sim.stats.delivered, 6);
        // four deliveries at x1, then two at x2
        assert_eq!(sim.stats.score, 4 + 2 * 2);
        assert_eq!(sim.body.len(), 1);
    }

    #[test]
    fn only_the_rear_car_gets_off() {
        let rows = [(8, "<@a....b.................")];
        let mut sim = Sim::new(&field(&rows, &[('a', 0, 0), ('b', 1, 0)]), quiet(), 1);
        // boarded in this order, so the rear car wants shape 2 and buries the two shape-1 cars
        sim.stations[0].queue.extend([1, 1, 2]);
        line(&mut sim, &(1..12).map(|x| (x, 8)).collect::<Vec<_>>());
        run(&mut sim, 9 * 7);
        assert_eq!(sim.stats.delivered, 0, "the rear car wants another station");
        assert_eq!(sim.cars, vec![1, 1, 2]);
        let mut any = Sim::new(&field(&rows, &[('a', 0, 0), ('b', 1, 0)]), quiet(), 1);
        any.p.unload = Unload::Any;
        any.stations[0].queue.extend([1, 1, 2]);
        line(&mut any, &(1..12).map(|x| (x, 8)).collect::<Vec<_>>());
        run(&mut any, 9 * 7);
        assert_eq!(any.stats.delivered, 2);
    }

    #[test]
    fn rail_behind_the_train_is_lifted() {
        let mut sim = Sim::new(&field(&[], &[]), quiet(), 1);
        line(&mut sim, &[(1, 8), (2, 8), (3, 8), (4, 8), (5, 8)]);
        assert_eq!(sim.used, 4);
        run(&mut sim, 9 * 3 + 1);
        assert_eq!(sim.engine(), Pos::new(4, 8));
        assert_eq!(sim.mask[Pos::new(2, 8).idx()], 0, "left behind and lifted");
        assert_eq!(sim.used, 2, "the engine's tile and the one ahead remain");
        sim.p.auto_lift = false;
        run(&mut sim, 10);
        assert_ne!(sim.mask[Pos::new(4, 8).idx()], 0);
    }

    #[test]
    fn express_runs_through_stations() {
        let rows = [(8, "<@..a....................")];
        let mut sim = Sim::new(&field(&rows, &[('a', 0, 0)]), quiet(), 1);
        sim.stations[0].queue.extend([1, 1]);
        sim.set_express(true);
        line(&mut sim, &(1..10).map(|x| (x, 8)).collect::<Vec<_>>());
        run(&mut sim, 4 * 4);
        assert!(sim.engine().x > 4);
        assert!(sim.cars.is_empty());
    }

    #[test]
    fn running_into_your_own_cars_crashes() {
        let rows = [(8, "<@a......................")];
        let mut sim = Sim::new(&field(&rows, &[('a', 0, 0)]), quiet(), 1);
        sim.stations[0].queue.extend([1; 6]);
        // out along row 8, round a loop and straight across (3,8) while the tail is still on it
        line(
            &mut sim,
            &[
                (1, 8),
                (2, 8),
                (3, 8),
                (4, 8),
                (5, 8),
                (5, 9),
                (4, 9),
                (3, 9),
                (3, 8),
                (3, 7),
            ],
        );
        assert_eq!(sim.mask[Pos::new(3, 8).idx()].count_ones(), 4, "a crossing");
        run(&mut sim, 9 * 12);
        assert_eq!(sim.stats.crashes, 1);
        assert_eq!(sim.stats.lost, 6);
        assert!(sim.cars.is_empty());
    }

    #[test]
    fn the_unlit_arm_of_a_switch_is_closed() {
        let mut sim = Sim::new(&field(&[], &[]), quiet(), 1);
        line(&mut sim, &[(1, 8), (2, 8), (3, 8), (4, 8)]);
        // a drag that arrives at (3,8) from the west and turns south lights west-south
        assert!(sim.draw_seg(Some(Dir::W), Pos::new(3, 8), Pos::new(3, 9)));
        assert_eq!(sim.mask[Pos::new(3, 8).idx()].count_ones(), 3);
        assert_eq!(sim.lit[Pos::new(3, 8).idx()], Dir::W.bit() | Dir::S.bit());
        run(&mut sim, 9 * 3 + 1);
        assert_eq!(
            sim.engine(),
            Pos::new(3, 9),
            "took the lit branch, not the closed east arm"
        );
        assert!(sim.flip(Pos::new(3, 8)), "the train has left the switch");
        // a switch under the engine is locked
        line(&mut sim, &[(3, 9), (4, 9)]);
        line(&mut sim, &[(3, 9), (2, 9)]);
        assert_eq!(sim.mask[Pos::new(3, 9).idx()].count_ones(), 3);
        assert!(!sim.flip(Pos::new(3, 9)));
    }

    #[test]
    fn a_branch_off_the_trains_line_leaves_its_way_lit() {
        let mut sim = Sim::new(&field(&[], &[]), quiet(), 1);
        line(&mut sim, &[(1, 8), (2, 8), (3, 8), (4, 8), (5, 8)]);
        // a drag that starts beside the line and ends on it makes a switch at (4,8)
        line(&mut sim, &[(4, 10), (4, 9), (4, 8)]);
        assert_eq!(sim.mask[Pos::new(4, 8).idx()].count_ones(), 3);
        assert_eq!(sim.lit[Pos::new(4, 8).idx()], Dir::W.bit() | Dir::E.bit());
        run(&mut sim, 9 * 4 + 1);
        assert_eq!(sim.engine(), Pos::new(5, 8), "kept going straight");
    }

    #[test]
    fn erasing_returns_pieces_after_the_delay() {
        let mut p = quiet();
        p.stock = 3;
        p.erase_delay = 10;
        let mut sim = Sim::new(&field(&[], &[]), p, 1);
        line(&mut sim, &[(5, 2), (6, 2), (7, 2)]);
        assert_eq!(sim.free(), 0);
        assert!(
            !sim.draw_seg(None, Pos::new(7, 2), Pos::new(8, 2)),
            "out of pieces"
        );
        assert!(sim.erase(Pos::new(6, 2)));
        assert_eq!(
            sim.mask[Pos::new(5, 2).idx()],
            0,
            "stranded neighbours lose their rail too"
        );
        assert_eq!(sim.used, 0);
        assert_eq!(sim.free(), 0, "pieces are still on their way back");
        run(&mut sim, 11);
        assert_eq!(sim.free(), 3);
    }

    #[test]
    fn the_open_terminus_ends_the_leg_with_a_bonus() {
        let mut sim = Sim::new(&field(&[], &[]), quiet(), 1);
        sim.p.fade = 0;
        line(
            &mut sim,
            &[
                (1, 8),
                (1, 9),
                (1, 10),
                (1, 11),
                (1, 12),
                (1, 13),
                (1, 14),
                (1, 15),
                (1, 16),
            ],
        );
        let mut path: Vec<(i32, i32)> = (1..=24).map(|x| (x, 16)).collect();
        path.insert(0, (1, 16));
        path.dedup();
        line(&mut sim, &path);
        run(&mut sim, 9 * 40);
        assert!(sim.parked);
        assert_eq!(sim.stats.bonus, 10);
    }
}
