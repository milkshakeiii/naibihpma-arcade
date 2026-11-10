//! The match, on top of the arena: kings, seeds that grow factories out of rocks pixel by pixel,
//! factories printing their build orders a pixel at a time, and the networks that pool what's
//! mined. None of it has its own state about rocks: a rock is just pixels, and a planted factory
//! is the claim.

use super::*;

pub const MW: i32 = 1200;
pub const MH: i32 = 900;
pub const KING_STARTS: [(i32, i32); 2] = [(150, MH - 150), (MW - 1 - 150, 149)];
pub const START_BANK: i32 = 300;
/// Pixels a second the king prints.
pub const KING_RATE: i32 = 6;
/// The king flies like Toy 1's: straight from the keys, this many pixels a tick.
pub const KING_SPEED: i32 = 3;
pub const SEED_COST: i32 = 20;
/// Pixels a second a seed eats into its factory, before the factory pixels it has grown join in.
pub const GROW_RATE: i32 = 4;
pub const WARP_TICKS: u32 = 30;
/// How far a node reaches: two of a team's nodes link when their reaches overlap.
pub const NODE_RANGE: i32 = 80;
pub const KING_RANGE: i32 = 120;
/// Most loose pixels a destroyed node spills.
pub const SPILL_CAP: i32 = 300;
/// A match still going after this long is a draw.
pub const MATCH_LIMIT: u32 = 15 * 60 * 40;
pub const KING_SLOT: usize = 100;
pub const SEED_SLOT: usize = 101;
/// Accumulators count in 60ths of a pixel per tick: a factory pixel adds 1 a tick, so it prints a
/// pixel every 4 s.
const UNIT: i32 = PRINT_TICKS;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Role {
    Ship,
    King,
    Seed,
}

/// What a factory is printing: nothing, the next thing on its list (no pixel paid yet), or the
/// ship it's partway through.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Making {
    None,
    Pending(usize),
    Child(usize),
}

/// Something a side tells its king to do. Commands are the only way input reaches a match, the
/// AI's included: they're queued and applied at the start of the next tick, and every one applied
/// is logged with its tick. The same commands on the same ticks replay the same match, which is
/// what lockstep multiplayer and replays need.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Cmd {
    /// The king's keys: -1, 0 or 1 on each axis, held until the next one.
    Steer { team: usize, dx: i32, dy: i32 },
    /// The king prints a deck slot. A seeded design's seed flies to the rock pixel `target`.
    Print { team: usize, slot: usize, target: Option<(i32, i32)> },
}

/// Whoever is printing this tick: a king working through its queue, or a factory (by ship index).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Printer {
    King(usize),
    Factory(usize),
}

/// What growing the next pixel of a ship would do: grow this pending cell, wait for an enemy to
/// move out of the way, or nothing (it's as grown as it can get).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(super) enum Next {
    Cell(usize),
    Blocked,
    Done,
}

#[derive(Clone, Debug)]
pub struct Grow {
    /// Cells still to grow, in growth order (outward from the core).
    pub pending: Vec<usize>,
    /// Rock pixels left to eat, nearest last. Empty for a ship being printed.
    food: Vec<(i32, i32)>,
    /// Growing out of a rock (a planted design) rather than being printed.
    pub eats: bool,
    acc: i32,
}

/// Something the king is printing: a deck slot, and for a seeded design the rock pixel its seed
/// flies to.
#[derive(Clone, Copy, Debug)]
pub struct Job {
    pub slot: usize,
    pub target: Option<(i32, i32)>,
    pub paid: i32,
    pub cost: i32,
}

#[derive(Clone, Copy, Debug, Default)]
pub struct Tally {
    pub planted: i32,
    pub printed: i32,
    pub summoned: i32,
    pub bank_peak: i32,
}

/// A rock as generated: its middle and its pixels (some may since have been eaten or mined).
#[derive(Clone, Debug)]
pub struct Rock {
    pub cx: i32,
    pub cy: i32,
    pub px: Vec<(i32, i32)>,
}

/// What the match needs to know about each deck slot, worked out once.
#[derive(Clone, Copy, Debug)]
pub struct Card {
    pub ok: bool,
    pub count: i32,
    pub cost: i32,
    pub seeded: bool,
}

pub struct Eco {
    pub deck: [Vec<Design>; 2],
    pub cards: [Vec<Card>; 2],
    pub king: [usize; 2],
    pub queue: [Vec<Job>; 2],
    king_acc: [i32; 2],
    /// The king's keys this tick: -1, 0 or 1 on each axis.
    steer: [(i32, i32); 2],
    /// Commands waiting for the next tick, and every command applied so far with its tick.
    inbox: Vec<Cmd>,
    pub log: Vec<(u32, Cmd)>,
    pub ai: [bool; 2],
    ai_next: [u32; 2],
    ai_big: [u32; 2],
    /// Network id per ship (usize::MAX: not a node).
    pub net: Vec<usize>,
    /// The tick each team's network last split (0: never).
    pub split_at: [u32; 2],
    pub tally: [Tally; 2],
    pub rocks: Vec<Rock>,
    /// Pixels each side has taken out of its banks to print with, and pixels lost with destroyed
    /// nodes (spilled or not). With what's mined and what's stored, these must always add up.
    pub spent: [i32; 2],
    pub spilt: [i32; 2],
}

impl Card {
    fn of(d: &Design) -> Card {
        let st = d.stats();
        Card { ok: d.check().ok(), count: st.count, cost: st.cost, seeded: d.seeded }
    }
    /// What the king pays to print it: a seed for a seeded design, the ship itself otherwise.
    pub fn price(&self) -> i32 {
        if self.seeded {
            SEED_COST
        } else {
            self.cost
        }
    }
}

impl Arena {
    /// A match on a mirrored map: each side starts with its king and a bank, nothing else.
    pub fn for_match(seed: u64, deck: [Vec<Design>; 2], ai: [bool; 2]) -> Arena {
        let mut a = Arena::blank(MW, MH, seed);
        let rocks = a.match_rocks();
        for _ in 0..900 {
            let (x, y) = (a.rng.range(0, MW - 1), a.rng.range(0, MH - 1));
            let c = if a.rng.chance(15) { 2 } else if a.rng.chance(40) { 1 } else { 0 };
            a.stars.push((x, y, c));
        }
        let k = king();
        let mut kings = [0; 2];
        for t in 0..2 {
            let (x, y) = KING_STARTS[t];
            let i = a.spawn(t, KING_SLOT, &k, x, y, if t == 0 { 0 } else { 2 }, false);
            let s = &mut a.ships[i];
            s.role = Role::King;
            s.store = START_BANK;
            s.node = true;
            a.score[t].placed += 1;
            kings[t] = i;
        }
        let cards = [deck[0].iter().map(Card::of).collect(), deck[1].iter().map(Card::of).collect()];
        a.eco = Some(Eco {
            deck,
            cards,
            king: kings,
            queue: [Vec::new(), Vec::new()],
            king_acc: [0; 2],
            steer: [(0, 0); 2],
            inbox: Vec::new(),
            log: Vec::new(),
            ai,
            ai_next: [0; 2],
            ai_big: [0; 2],
            net: Vec::new(),
            split_at: [0; 2],
            tally: [Tally::default(); 2],
            rocks,
            spent: [0; 2],
            spilt: [0; 2],
        });
        a
    }

    fn eco(&self) -> &Eco {
        self.eco.as_ref().expect("a match")
    }
    fn eco_mut(&mut self) -> &mut Eco {
        self.eco.as_mut().expect("a match")
    }

    // ----- The map -----

    /// Rocks in mirrored pairs (every rock has a twin through the centre, so neither king is ever
    /// a pixel closer to anything), one giant in the middle, and two home rocks by each king.
    fn match_rocks(&mut self) -> Vec<Rock> {
        let (w, h) = (self.w, self.h);
        let mut rocks = Vec::new();
        let mut placed: Vec<(i32, i32, i32)> = Vec::new();
        // The giant: point-symmetric itself, so it's its own twin.
        let (cx, cy, r) = (w / 2, h / 2, 30);
        let half: Vec<(i32, i32)> = self.blob(cx, cy, r).into_iter().filter(|&(x, y)| y < cy || (y == cy && x <= cx)).collect();
        let mut giant = half.clone();
        giant.extend(half.iter().map(|&(x, y)| (w - 1 - x, h - 1 - y)));
        giant.sort();
        giant.dedup();
        self.put_rock(giant, &mut rocks);
        placed.push((cx, cy, r));
        let (kx, ky) = KING_STARTS[0];
        let mut pairs: Vec<(i32, i32, i32)> = vec![(kx + 125, ky + 35, 12), (kx - 15, ky - 150, 9)];
        // Two big contested pairs, then the rest.
        let mut tries = 0;
        let mut want: Vec<(i32, i32)> = vec![(20, 24), (20, 24)];
        want.extend(std::iter::repeat((6, 16)).take(16));
        for (lo, hi) in want {
            loop {
                tries += 1;
                if tries > 20000 {
                    break;
                }
                let r = self.rng.range(lo, hi);
                let (x, y) = (self.rng.range(r + 14, w - r - 15), self.rng.range(r + 14, h - r - 15));
                let (mx, my) = (w - 1 - x, h - 1 - y);
                let clear = |px: i32, py: i32, all: &Vec<(i32, i32, i32)>| {
                    KING_STARTS.iter().all(|&(a, b)| dist2(px, py, a, b) >= (r + 90) * (r + 90)) && all.iter().all(|&(a, b, r2)| dist2(px, py, a, b) >= (r + r2 + 26) * (r + r2 + 26))
                };
                let mut all = placed.clone();
                all.extend(pairs.iter().copied());
                if clear(x, y, &all) && clear(mx, my, &all) && dist2(x, y, mx, my) >= (2 * r + 26) * (2 * r + 26) {
                    pairs.push((x, y, r));
                    break;
                }
            }
        }
        for (x, y, r) in pairs {
            let px = self.blob(x, y, r);
            let twin: Vec<(i32, i32)> = px.iter().map(|&(a, b)| (w - 1 - a, h - 1 - b)).collect();
            self.put_rock(px, &mut rocks);
            self.put_rock(twin, &mut rocks);
            placed.push((x, y, r));
        }
        rocks
    }

    /// A rough disc of rock: a ragged edge, solid inside.
    fn blob(&mut self, cx: i32, cy: i32, r: i32) -> Vec<(i32, i32)> {
        let mut px = Vec::new();
        for y in -r..=r {
            for x in -r..=r {
                let d = dist2(x, y, 0, 0);
                let edge = d > (r - 2) * (r - 2);
                if d <= r * r && !(edge && self.rng.chance(35)) {
                    px.push((cx + x, cy + y));
                }
            }
        }
        px
    }

    fn put_rock(&mut self, px: Vec<(i32, i32)>, rocks: &mut Vec<Rock>) {
        let px: Vec<(i32, i32)> = px.into_iter().filter(|&(x, y)| (0..self.w).contains(&x) && (0..self.h).contains(&y)).collect();
        if px.is_empty() {
            return;
        }
        for &(x, y) in &px {
            self.rock[(y * self.w + x) as usize] = true;
            self.rock_px.push((x, y));
        }
        let n = px.len() as i32;
        let (cx, cy) = (px.iter().map(|p| p.0).sum::<i32>() / n, px.iter().map(|p| p.1).sum::<i32>() / n);
        rocks.push(Rock { cx, cy, px });
    }

    /// The pixels of a rock still standing.
    pub fn rock_left(&self, r: &Rock) -> i32 {
        r.px.iter().filter(|&&(x, y)| self.rock_at(x, y)).count() as i32
    }

    /// The rock pixel nearest a point, within `reach` (a square search).
    pub fn rock_near(&self, (x, y): (i32, i32), reach: i32) -> Option<(i32, i32)> {
        let mut best: Option<(i32, (i32, i32))> = None;
        for py in y - reach..=y + reach {
            for px in x - reach..=x + reach {
                if self.rock_at(px, py) {
                    let d = dist2(px, py, x, y);
                    if best.map_or(true, |(b, _)| d < b) {
                        best = Some((d, (px, py)));
                    }
                }
            }
        }
        best.map(|(_, p)| p)
    }

    /// The rock pixel nearest a point that no enemy pixel is sitting on, within `reach` if given.
    pub fn free_rock(&self, (x, y): (i32, i32), team: usize, reach: Option<i32>) -> Option<(i32, i32)> {
        let mut best: Option<((i32, i32, i32), (i32, i32))> = None;
        for &(px, py) in &self.rock_px {
            if !self.rock_at(px, py) || reach.is_some_and(|r| cheb(px, py, x, y) > r) {
                continue;
            }
            let key = (dist2(px, py, x, y), py, px);
            if best.map_or(true, |(b, _)| key < b) && self.room_to_grow(team, None, px, py) {
                best = Some((key, (px, py)));
            }
        }
        best.map(|(_, p)| p)
    }

    /// Every pixel of the rock touching `start`, edge to edge.
    pub fn rock_blob(&self, start: (i32, i32)) -> Vec<(i32, i32)> {
        let mut seen = std::collections::HashSet::new();
        let mut todo = vec![start];
        let mut out = Vec::new();
        seen.insert(start);
        while let Some((x, y)) = todo.pop() {
            out.push((x, y));
            for (dx, dy) in [(1, 0), (-1, 0), (0, 1), (0, -1)] {
                let n = (x + dx, y + dy);
                if self.rock_at(n.0, n.1) && seen.insert(n) {
                    todo.push(n);
                }
            }
        }
        out
    }

    // ----- The tick -----

    pub(super) fn eco_pre(&mut self) {
        for t in 0..2 {
            if self.eco().ai[t] && self.tick >= self.eco().ai_next[t] {
                self.ai_think(t);
                self.eco_mut().ai_next[t] = self.tick + 15;
            }
            self.apply_commands();
            self.steer_king(t);
        }
    }

    /// Queues a command for the next tick. A steer that changes nothing is dropped.
    pub fn command(&mut self, c: Cmd) {
        let e = self.eco_mut();
        if let Cmd::Steer { team, dx, dy } = c {
            let last = e.inbox.iter().rev().find_map(|q| match *q {
                Cmd::Steer { team: t, dx, dy } if t == team => Some((dx, dy)),
                _ => None,
            });
            if last.unwrap_or(e.steer[team]) == (dx.signum(), dy.signum()) {
                return;
            }
        }
        e.inbox.push(c);
    }

    /// Applies everything queued, in order, and logs it against this tick.
    fn apply_commands(&mut self) {
        let cmds = std::mem::take(&mut self.eco_mut().inbox);
        let tick = self.tick;
        for c in cmds {
            match c {
                Cmd::Steer { team, dx, dy } => self.eco_mut().steer[team] = (dx.signum(), dy.signum()),
                Cmd::Print { team, slot, target } => {
                    // One that's no longer allowed (it was checked when it was given) is dropped.
                    let Ok(cost) = self.can_print(team, slot, target) else { continue };
                    self.eco_mut().queue[team].push(Job { slot, target, paid: 0, cost });
                }
            }
            self.eco_mut().log.push((tick, c));
        }
    }

    pub(super) fn eco_post(&mut self) {
        self.networks();
        self.spills();
        // Anything delivered or printed this tick is dealt with from next tick on.
        let n = self.ships.len();
        // Everyone with something to print: kings with a queue, factories with a build order.
        let mut printers: Vec<Printer> = Vec::new();
        for t in 0..2 {
            let k = self.eco().king[t];
            if self.ships[k].alive && !self.eco().queue[t].is_empty() {
                self.eco_mut().king_acc[t] += KING_RATE * UNIT / 15;
                printers.push(Printer::King(t));
            }
        }
        for i in 0..n {
            if !self.ships[i].alive {
                continue;
            }
            if self.ships[i].warp > 0 {
                // It can't come out of hyperspace into another ship's pixels: it waits, still
                // beside its king, until its spot is clear.
                if self.ships[i].warp > 1 || !self.in_enemy(i) {
                    self.ships[i].warp -= 1;
                }
                continue;
            }
            // A half-printed ship whose factory died flies on as it is.
            if let Some(m) = self.ships[i].maker {
                if !self.ships[m].alive {
                    let s = &mut self.ships[i];
                    s.maker = None;
                    s.grow = None;
                    s.home = (s.x, s.y);
                }
                continue;
            }
            if self.ships[i].role == Role::Seed {
                self.seed_step(i);
            } else if self.ships[i].grow.as_ref().is_some_and(|g| g.eats) {
                // Its factory pixels grow it first. Wedged against an enemy with nowhere to grow,
                // they print instead.
                if self.growth_step(i) && self.factory_ready(i) {
                    printers.push(Printer::Factory(i));
                }
            } else if self.factory_ready(i) {
                printers.push(Printer::Factory(i));
            }
        }
        self.take_turns(printers);
        for t in 0..2 {
            let k = self.eco().king[t];
            let bank = self.bank(k);
            let tl = &mut self.eco_mut().tally[t];
            tl.bank_peak = tl.bank_peak.max(bank);
        }
        let k = self.eco().king;
        let dead = [!self.ships[k[0]].alive, !self.ships[k[1]].alive];
        if dead[0] || dead[1] {
            self.over = Some(match dead {
                [true, true] => None,
                [true, false] => Some(1),
                _ => Some(0),
            });
        } else if self.tick >= MATCH_LIMIT {
            self.over = Some(None);
        }
    }

    /// The king flies straight from its keys, like Toy 1's. It moves and turns by the same rules as
    /// every other ship: it can't fly through a bigger enemy, or through anything planted.
    fn steer_king(&mut self, t: usize) {
        let (k, (dx, dy)) = (self.eco().king[t], self.eco().steer[t]);
        if !self.ships[k].alive {
            return;
        }
        let (dx, dy) = (dx.signum(), dy.signum());
        let (x0, y0, h) = (self.ships[k].x, self.ships[k].y, self.ships[k].half());
        let nx = (x0 + dx * KING_SPEED).clamp(h, self.w - 1 - h);
        let ny = (y0 + dy * KING_SPEED).clamp(h, self.h - 1 - h);
        let (x, y) = self.shift(k, nx, ny);
        // Its speed, in the quarter-pixels the rest of the sim uses, so guns lead it properly.
        let s = &mut self.ships[k];
        s.vx = (x - x0) * 4;
        s.vy = (y - y0) * 4;
        if let Some(f) = axis_face(dx, dy) {
            if f != self.ships[k].facing {
                self.try_turn(k, f);
            }
        }
    }

    // ----- Networks and banks -----

    fn reach(&self, i: usize) -> i32 {
        if self.ships[i].role == Role::King {
            KING_RANGE
        } else {
            NODE_RANGE
        }
    }

    pub fn is_node(&self, i: usize) -> bool {
        let s = &self.ships[i];
        s.alive && s.node && s.warp == 0 && s.maker.is_none()
    }

    /// Links every pair of a team's nodes whose reaches overlap, and notes when a network that was
    /// one piece last tick is in two now.
    fn networks(&mut self) {
        let n = self.ships.len();
        let ids: Vec<usize> = (0..n).filter(|&i| self.is_node(i)).collect();
        let mut parent: Vec<usize> = (0..n).collect();
        fn find(p: &mut [usize], mut x: usize) -> usize {
            while p[x] != x {
                p[x] = p[p[x]];
                x = p[x];
            }
            x
        }
        for a in 0..ids.len() {
            for b in a + 1..ids.len() {
                let (i, j) = (ids[a], ids[b]);
                let (si, sj) = (&self.ships[i], &self.ships[j]);
                let r = self.reach(i) + self.reach(j);
                if si.team == sj.team && dist2(si.x, si.y, sj.x, sj.y) <= r * r {
                    let (ri, rj) = (find(&mut parent, i), find(&mut parent, j));
                    parent[ri] = rj;
                }
            }
        }
        let mut net = vec![usize::MAX; n];
        for &i in &ids {
            net[i] = find(&mut parent, i);
        }
        let old = &self.eco().net;
        let mut split = [false; 2];
        for a in 0..ids.len() {
            for b in a + 1..ids.len() {
                let (i, j) = (ids[a], ids[b]);
                let was = old.get(i).copied().unwrap_or(usize::MAX);
                if self.ships[i].team == self.ships[j].team && was != usize::MAX && old.get(j) == Some(&was) && net[i] != net[j] {
                    split[self.ships[i].team] = true;
                }
            }
        }
        let tick = self.tick;
        let e = self.eco_mut();
        for t in 0..2 {
            if split[t] {
                e.split_at[t] = tick;
            }
        }
        e.net = net;
    }

    /// Everything stored in the network a ship belongs to (just its own store if it isn't a node).
    pub fn bank(&self, i: usize) -> i32 {
        match self.eco().net.get(i).copied() {
            Some(id) if id != usize::MAX => (0..self.ships.len()).filter(|&j| self.eco().net.get(j) == Some(&id)).map(|j| self.ships[j].store).sum(),
            _ => self.ships[i].store,
        }
    }

    /// Takes a pixel for printing: from the printer's own store first, then from the nearest node
    /// in its network that has one.
    fn take_px(&mut self, i: usize) -> bool {
        let team = self.ships[i].team;
        if self.ships[i].store > 0 {
            self.ships[i].store -= 1;
            self.eco_mut().spent[team] += 1;
            return true;
        }
        let id = match self.eco().net.get(i).copied() {
            Some(id) if id != usize::MAX => id,
            _ => return false,
        };
        let (x, y) = (self.ships[i].x, self.ships[i].y);
        let from = (0..self.ships.len()).filter(|&j| self.eco().net.get(j) == Some(&id) && self.ships[j].store > 0).min_by_key(|&j| dist2(self.ships[j].x, self.ships[j].y, x, y));
        match from {
            Some(j) => {
                self.ships[j].store -= 1;
                self.eco_mut().spent[team] += 1;
                true
            }
            None => false,
        }
    }

    /// A destroyed node spills what it held as loose pixels for anyone to scoop.
    fn spills(&mut self) {
        for i in 0..self.ships.len() {
            if self.ships[i].alive || self.ships[i].store == 0 {
                continue;
            }
            let (x, y, h) = (self.ships[i].x, self.ships[i].y, self.ships[i].half().max(2));
            let n = self.ships[i].store.min(SPILL_CAP);
            let team = self.ships[i].team;
            self.eco_mut().spilt[team] += self.ships[i].store;
            self.ships[i].store = 0;
            for _ in 0..n {
                let (dx, dy) = (self.rng.range(-h, h), self.rng.range(-h, h));
                let drift = 2 + self.rng.below(6) as i32;
                self.debris.push(Debris { x: (x + dx).clamp(0, self.w - 1), y: (y + dy).clamp(0, self.h - 1), vx: dx.signum(), vy: dy.signum(), drift, taken: false });
            }
        }
    }

    // ----- Growing and printing -----

    /// The next pixel a growing or half-printed ship would grow: the first pending cell that touches
    /// one already there (the core, to start) with room where it would go (see `room_to_grow`).
    /// Looked up before anything is eaten or paid for, so nothing is spent on a pixel that can't be
    /// placed.
    pub(super) fn next_cell(&self, i: usize) -> Next {
        let s = &self.ships[i];
        let Some(g) = s.grow.as_ref() else { return Next::Done };
        let (w, h) = (s.shape.w, s.shape.h);
        let started = s.hp.iter().any(|&p| p > 0);
        let mut blocked = false;
        for (k, &c) in g.pending.iter().enumerate() {
            let touches = if started {
                let (x, y) = (c as i32 % w, c as i32 / w);
                [(1, 0), (-1, 0), (0, 1), (0, -1)].iter().any(|&(dx, dy)| {
                    let (nx, ny) = (x + dx, y + dy);
                    nx >= 0 && ny >= 0 && nx < w && ny < h && s.hp[(ny * w + nx) as usize] > 0
                })
            } else {
                s.shape.is_core(c)
            };
            if !touches {
                continue;
            }
            // A factory growing out of a rock needs room; a ship being printed only needs no enemy
            // there (its own side's things are under it, and it flies off when it's done).
            let (x, y) = s.cell_xy(c);
            let room = if g.eats { self.room_to_grow(s.team, Some(i), x, y) } else { !self.enemy_pixel(s.team, x, y) };
            if !room {
                blocked = true;
                continue;
            }
            return Next::Cell(k);
        }
        if blocked {
            Next::Blocked
        } else {
            Next::Done
        }
    }

    fn grow_cell(&mut self, i: usize, k: usize) {
        if cfg!(test) {
            let s = &self.ships[i];
            let g = s.grow.as_ref().unwrap();
            let (x, y) = s.cell_xy(g.pending[k]);
            assert!(!self.enemy_pixel(s.team, x, y), "{} grew a pixel onto an enemy pixel at ({x},{y})", s.name);
            assert!(!g.eats || !self.own_standing_pixel(s.team, Some(i), x, y), "{} grew a pixel into a factory of its own side at ({x},{y})", s.name);
        }
        let s = &mut self.ships[i];
        let c = s.grow.as_mut().unwrap().pending.remove(k);
        s.hp[c] = 1;
        s.dirty = true;
    }

    fn pending(&self, i: usize) -> usize {
        self.ships[i].grow.as_ref().map_or(0, |g| g.pending.len())
    }

    /// A factory growing out of its rock: the seed eats at a steady rate and every factory pixel
    /// grown so far helps, one rock pixel for each pixel grown. It stops when it's whole or the
    /// rock runs out, and waits while an enemy sits where its next pixels would go (returning true
    /// for a tick it spent waiting).
    fn growth_step(&mut self, i: usize) -> bool {
        let f = self.ships[i].st.factory;
        let mut food = {
            let g = self.ships[i].grow.as_mut().unwrap();
            g.acc += GROW_RATE * UNIT / 15 + f;
            std::mem::take(&mut g.food)
        };
        let (mut done, mut waiting) = (false, false);
        while self.ships[i].grow.as_ref().is_some_and(|g| g.acc >= UNIT) {
            // The next rock pixel to eat (any mined away since it landed are skipped).
            while food.last().is_some_and(|&(x, y)| !self.rock_at(x, y)) {
                food.pop();
            }
            let Some(&(x, y)) = food.last() else {
                done = true;
                break;
            };
            match self.next_cell(i) {
                Next::Cell(k) => {
                    food.pop();
                    self.rock[(y * self.w + x) as usize] = false;
                    self.grow_cell(i, k);
                    self.ships[i].grow.as_mut().unwrap().acc -= UNIT;
                    if self.pending(i) == 0 {
                        done = true;
                        break;
                    }
                }
                Next::Blocked => {
                    self.ships[i].grow.as_mut().unwrap().acc = UNIT;
                    waiting = true;
                    break;
                }
                Next::Done => {
                    done = true;
                    break;
                }
            }
        }
        if done {
            self.ships[i].grow = None;
        } else if let Some(g) = self.ships[i].grow.as_mut() {
            g.food = food;
        }
        waiting
    }

    /// Where something printed beside `m` goes: just outside it, on the side facing the enemy king,
    /// facing that way.
    fn beside(&self, m: usize, sh: &Shape) -> (i32, i32, u8) {
        let s = &self.ships[m];
        let ek = &self.ships[self.eco().king[1 - s.team]];
        let f = axis_face(ek.x - s.x, ek.y - s.y).unwrap_or(1);
        let (w, h) = sh.dims(f);
        let (l, t, r, b) = s.bbox();
        let (x, y) = match f {
            1 => (r + 3 + w / 2, s.y),
            3 => (l - 3 - (w - 1 - w / 2), s.y),
            2 => (s.x, b + 3 + h / 2),
            _ => (s.x, t - 3 - (h - 1 - h / 2)),
        };
        (x, y, f)
    }

    /// A factory (or carrier) works through its build order: the next thing it's big enough for.
    /// This settles what it's printing and gives it this tick's printing; false if it has nothing
    /// it can build.
    fn factory_ready(&mut self, i: usize) -> bool {
        if self.ships[i].st.factory == 0 || self.ships[i].build.is_empty() {
            return false;
        }
        let team = self.ships[i].team;
        if let Making::Child(c) = self.ships[i].making {
            if !self.ships[c].alive {
                // Shot while printing: start it again.
                self.ships[i].making = Making::None;
            }
        }
        if self.ships[i].making == Making::None {
            let s = &self.ships[i];
            let cards = &self.eco().cards[team];
            let n = s.build.len();
            let max = s.st.max_build();
            let fits = |slot: usize| cards.get(slot).is_some_and(|c| c.ok && !c.seeded && c.count <= max);
            let Some(k) = (0..n).map(|k| (s.build_i + k) % n).find(|&k| fits(s.build[k])) else { return false };
            let slot = s.build[k];
            self.ships[i].build_i = k;
            self.ships[i].making = Making::Pending(slot);
        }
        let f = self.ships[i].st.factory;
        self.ships[i].print_acc += f;
        true
    }

    /// Everyone printing takes turns, a pixel each, until each has printed what its rate allows or
    /// can't get a pixel. Who goes first moves round every tick, so when the bank is short it's
    /// shared out evenly, not handed to whoever was built first.
    fn take_turns(&mut self, mut printers: Vec<Printer>) {
        if printers.is_empty() {
            return;
        }
        let first = self.tick as usize % printers.len();
        printers.rotate_left(first);
        while !printers.is_empty() {
            let mut again = Vec::new();
            for p in printers {
                if self.print_turn(p) {
                    again.push(p);
                }
            }
            printers = again;
        }
    }

    /// One printer's turn: one pixel, paid for from its network as it goes down. True if it printed
    /// and is due another this tick. Short of pixels, or with an enemy where the pixel would go, it
    /// waits.
    fn print_turn(&mut self, p: Printer) -> bool {
        match p {
            Printer::King(t) => {
                let k = self.eco().king[t];
                if self.eco().king_acc[t] < UNIT || self.eco().queue[t].is_empty() {
                    return false;
                }
                if !self.take_px(k) {
                    self.eco_mut().king_acc[t] = UNIT;
                    return false;
                }
                let e = self.eco_mut();
                e.king_acc[t] -= UNIT;
                e.queue[t][0].paid += 1;
                if e.queue[t][0].paid >= e.queue[t][0].cost {
                    let job = e.queue[t].remove(0);
                    self.deliver(t, job);
                    return false;
                }
                self.eco().king_acc[t] >= UNIT
            }
            Printer::Factory(i) => {
                if self.ships[i].print_acc < UNIT {
                    return false;
                }
                // Is there a pixel to print? Look first, so nothing is paid for one that can't go down.
                let making = self.ships[i].making;
                let next = match making {
                    Making::Child(c) => self.next_cell(c),
                    Making::Pending(slot) if self.print_spot_clear(i, slot) => Next::Cell(0),
                    Making::Pending(_) => Next::Blocked,
                    Making::None => return false,
                };
                match (next, making) {
                    (Next::Cell(k), _) => {
                        if !self.take_px(i) {
                            self.ships[i].print_acc = UNIT;
                            return false;
                        }
                        self.ships[i].print_acc -= UNIT;
                        match making {
                            Making::Pending(slot) => self.start_print(i, slot),
                            Making::Child(c) => {
                                self.grow_cell(c, k);
                                if self.pending(c) == 0 {
                                    self.launch(i, c);
                                }
                            }
                            Making::None => {}
                        }
                        self.ships[i].making != Making::None && self.ships[i].print_acc >= UNIT
                    }
                    // An enemy sits where the next pixel goes: wait.
                    (Next::Blocked, _) => {
                        self.ships[i].print_acc = UNIT;
                        false
                    }
                    // Nothing more of it can grow (pieces were shot off): it launches as it is.
                    (Next::Done, Making::Child(c)) => {
                        self.launch(i, c);
                        false
                    }
                    (Next::Done, _) => false,
                }
            }
        }
    }

    /// Whether the first pixel of the next ship (its core) has a free spot beside the factory.
    fn print_spot_clear(&self, i: usize, slot: usize) -> bool {
        let team = self.ships[i].team;
        let d = &self.eco().deck[team][slot];
        let sh = d.shape();
        let (x, y, f) = self.beside(i, &sh);
        let (cx, cy) = sh.cell_xy(x, y, f, d.start_cell().unwrap_or(0));
        !self.enemy_pixel(team, cx, cy)
    }

    /// Puts down the first pixel of the next ship on the list: its core, beside the factory.
    fn start_print(&mut self, i: usize, slot: usize) {
        let team = self.ships[i].team;
        let d = self.eco().deck[team][slot].clone();
        let sh = d.shape();
        let (x, y, f) = self.beside(i, &sh);
        let c = self.spawn(team, slot, &d, x, y, f, true);
        let start = d.start_cell().unwrap_or(0);
        self.ships[c].grow = Some(Grow { pending: Design::growth_order(&sh, start), food: Vec::new(), eats: false, acc: 0 });
        self.ships[c].maker = Some(i);
        if let Some(k) = self.ships[c].grow.as_ref().unwrap().pending.iter().position(|&p| p == start) {
            self.grow_cell(c, k);
        }
        let off = (self.ships[c].x - self.ships[i].x, self.ships[c].y - self.ships[i].y);
        self.ships[i].make_off = off;
        self.ships[i].making = Making::Child(c);
        if self.pending(c) == 0 {
            self.launch(i, c);
        }
    }

    /// A ship being printed stays beside its factory. Carriers move, so this runs with the rest of
    /// the movement, before shots are resolved: shots then see it move like any other ship.
    pub(super) fn eco_follow(&mut self) {
        // A ship still arriving comes out beside its king, wherever the king has got to.
        for i in 0..self.ships.len() {
            if self.ships[i].alive && self.ships[i].warp > 0 {
                let k = self.eco().king[self.ships[i].team];
                if self.ships[k].alive {
                    let (x, y, f) = self.beside(k, &self.ships[i].shape);
                    self.carry(i, x, y, Some(f));
                    let s = &mut self.ships[i];
                    s.home = (s.x, s.y);
                    s.pose0 = (s.x, s.y, f);
                }
            }
        }
        for i in 0..self.ships.len() {
            if let (true, Making::Child(c)) = (self.ships[i].alive, self.ships[i].making) {
                if self.ships[c].alive {
                    let (mx, my, off) = (self.ships[i].x, self.ships[i].y, self.ships[i].make_off);
                    self.carry(c, mx + off.0, my + off.1, None);
                }
            }
        }
    }

    /// A printed ship is finished: it's on its own now.
    fn launch(&mut self, i: usize, c: usize) {
        let s = &mut self.ships[c];
        s.grow = None;
        s.maker = None;
        s.home = (s.x, s.y);
        let team = s.team;
        let f = &mut self.ships[i];
        f.making = Making::None;
        f.build_i = (f.build_i + 1) % f.build.len().max(1);
        self.score[team].placed += 1;
        self.eco_mut().tally[team].printed += 1;
    }

    /// What a king finishes printing arrives by hyperspace beside it.
    fn deliver(&mut self, t: usize, job: Job) {
        let k = self.eco().king[t];
        let seeded = self.eco().cards[t][job.slot].seeded;
        let d = if seeded { seed() } else { self.eco().deck[t][job.slot].clone() };
        let (x, y, f) = self.beside(k, &d.shape());
        let i = self.spawn(t, if seeded { SEED_SLOT } else { job.slot }, &d, x, y, f, false);
        let s = &mut self.ships[i];
        s.warp = WARP_TICKS;
        if seeded {
            s.role = Role::Seed;
            s.seed_for = Some((job.slot, job.target.unwrap_or((x, y))));
        }
        self.score[t].placed += 1;
        if !seeded {
            self.eco_mut().tally[t].summoned += 1;
        }
    }

    /// Whether a king may print a deck slot, and what it costs. A seeded design needs the rock pixel
    /// its seed should fly to.
    pub fn can_print(&self, t: usize, slot: usize, target: Option<(i32, i32)>) -> Result<i32, &'static str> {
        let card = *self.eco().cards.get(t).and_then(|c| c.get(slot)).ok_or("NO SUCH DESIGN")?;
        if !card.ok {
            return Err("THAT DESIGN CAN'T FLY YET");
        }
        if card.seeded && target.is_none() {
            return Err("A SEED NEEDS A ROCK");
        }
        Ok(card.price())
    }

    // ----- Seeds -----

    pub(super) fn seed_plan(&mut self, i: usize) -> Plan {
        let s = &self.ships[i];
        match s.seed_for {
            Some((_, target)) => self.go(i, target, Doing::ToRock),
            None => self.go(i, (s.x, s.y), Doing::Idle),
        }
    }

    /// A seed that has reached its rock comes down on it and starts growing its design out of it,
    /// from the seed's own core outward.
    fn seed_step(&mut self, i: usize) {
        let Some((slot, target)) = self.ships[i].seed_for else { return };
        let (x, y, team) = (self.ships[i].x, self.ships[i].y, self.ships[i].team);
        // It comes down when it reaches its rock pixel, or when it's run up against a bigger enemy
        // and the pixel it was after is gone or under an enemy (so it can never get there).
        let there = cheb(x, y, target.0, target.1) <= 2;
        let shut_out = self.ships[i].bumped && (!self.rock_at(target.0, target.1) || self.enemy_pixel(team, target.0, target.1));
        if !there && !shut_out {
            return;
        }
        // It lands on the nearest rock pixel no enemy is sitting on.
        let Some(start) = self.free_rock((x, y), team, Some(10)) else {
            // None in reach (its rock was mined or eaten away, or is covered): it heads for the
            // nearest free rock left anywhere, or waits where it is if there's none.
            if there {
                self.ships[i].seed_for = self.free_rock((x, y), team, None).map(|p| (slot, p));
            }
            return;
        };
        let mut food = self.rock_blob(start);
        food.sort_by_key(|&(px, py)| std::cmp::Reverse((dist2(px, py, start.0, start.1), py, px)));
        let d = self.eco().deck[team][slot].clone();
        let sh = d.shape();
        let Some(c0) = d.start_cell() else { return };
        let (ox, oy) = sh.cell_xy(0, 0, 0, c0);
        let f = self.spawn(team, slot, &d, start.0 - ox, start.1 - oy, 0, true);
        self.ships[f].rooted = true;
        self.ships[f].grow = Some(Grow { pending: Design::growth_order(&sh, c0), food, eats: true, acc: 0 });
        // The seed's own core is the factory's first pixel.
        if let Some(k) = self.ships[f].grow.as_ref().unwrap().pending.iter().position(|&p| p == c0) {
            self.grow_cell(f, k);
        }
        self.ships[i].alive = false;
        self.score[team].placed += 1;
        self.eco_mut().tally[team].planted += 1;
    }

    // ----- The AI -----

    /// A scripted opponent: it keeps its king home and out of reach, plants a factory on the best
    /// free rock every so often (an outpost first, shipyards once it has two), and later has its
    /// king summon its biggest ship now and then.
    fn ai_think(&mut self, t: usize) {
        let k = self.eco().king[t];
        if !self.ships[k].alive {
            return;
        }
        let (kx, ky) = (self.ships[k].x, self.ships[k].y);
        let home = KING_STARTS[t];
        let threat = self.ships.iter().filter(|s| s.alive && s.team != t && s.st.armed() && cheb(s.x, s.y, kx, ky) < 90).min_by_key(|s| dist2(s.x, s.y, kx, ky));
        let steer = match threat {
            Some(s) => ((kx - s.x).signum(), (ky - s.y).signum()),
            None if cheb(kx, ky, home.0, home.1) > 6 => ((home.0 - kx).signum(), (home.1 - ky).signum()),
            None => (0, 0),
        };
        self.command(Cmd::Steer { team: t, dx: steer.0, dy: steer.1 });
        if !self.eco().queue[t].is_empty() {
            return;
        }
        let cards = self.eco().cards[t].clone();
        let mut seeded: Vec<usize> = (0..cards.len()).filter(|&s| cards[s].ok && cards[s].seeded).collect();
        seeded.sort_by_key(|&s| cards[s].count);
        let factories = self.ships.iter().filter(|s| s.alive && s.team == t && s.role == Role::Ship && s.node).count();
        let seeds = self.ships.iter().filter(|s| s.alive && s.team == t && s.role == Role::Seed && s.seed_for.is_some()).count();
        let want = 1 + (self.tick / (15 * 75)) as usize;
        if seeds == 0 && factories < want.min(9) && !seeded.is_empty() {
            let big = *seeded.last().unwrap();
            let pick = if factories >= 2 && seeded.len() > 1 { [big, seeded[0]] } else { [seeded[0], seeded[0]] };
            for slot in pick {
                if let Some(target) = self.ai_rock(t, cards[slot].count) {
                    self.command(Cmd::Print { team: t, slot, target: Some(target) });
                    return;
                }
            }
        }
        // From four minutes on, the king summons its biggest built ship every two and a half
        // minutes, paying for it a pixel at a time alongside the factories.
        if self.tick > 15 * 240 && self.tick >= self.eco().ai_big[t] {
            let best = (0..cards.len()).filter(|&s| cards[s].ok && !cards[s].seeded).max_by_key(|&s| cards[s].cost);
            if let Some(s) = best {
                self.command(Cmd::Print { team: t, slot: s, target: None });
                self.eco_mut().ai_big[t] = self.tick + 15 * 150;
            }
        }
    }

    /// The free rock nearest the AI's king that's big enough, nearer to it than to the enemy king:
    /// nobody's growing on it and none of our seeds is headed there. Returns the rock pixel nearest
    /// its middle.
    fn ai_rock(&self, t: usize, need: i32) -> Option<(i32, i32)> {
        let (k, ek) = (self.eco().king[t], self.eco().king[1 - t]);
        let (kx, ky) = (self.ships[k].x, self.ships[k].y);
        let (ex, ey) = (self.ships[ek].x, self.ships[ek].y);
        let rocks = &self.eco().rocks;
        let mut best: Option<(i32, (i32, i32))> = None;
        for r in rocks {
            let left = self.rock_left(r);
            if left < need {
                continue;
            }
            let rad = isqrt(left / 3) + 4;
            let taken = self.ships.iter().any(|s| s.alive && s.role == Role::Ship && (s.node || s.grow.as_ref().is_some_and(|g| g.eats)) && cheb(s.x, s.y, r.cx, r.cy) <= rad + s.half())
                || self.ships.iter().any(|s| s.alive && s.seed_for.is_some_and(|(_, p)| cheb(p.0, p.1, r.cx, r.cy) <= rad));
            let ours = dist2(r.cx, r.cy, kx, ky) < dist2(r.cx, r.cy, ex, ey);
            if taken || !ours {
                continue;
            }
            let d = dist2(r.cx, r.cy, kx, ky);
            if best.map_or(true, |(b, _)| d < b) {
                let target = r.px.iter().copied().filter(|&(x, y)| self.rock_at(x, y)).min_by_key(|&(x, y)| dist2(x, y, r.cx, r.cy))?;
                best = Some((d, target));
            }
        }
        best.map(|(_, p)| p)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn quiet() -> Arena {
        Arena::for_match(1, [starter_deck(), starter_deck()], [false, false])
    }

    /// Blue's home rock (the one nearest its king) and the rock pixel nearest its middle.
    fn home_rock(a: &Arena) -> (usize, (i32, i32)) {
        let (kx, ky) = KING_STARTS[0];
        let e = a.eco.as_ref().unwrap();
        let r = (0..e.rocks.len()).min_by_key(|&r| dist2(e.rocks[r].cx, e.rocks[r].cy, kx, ky)).unwrap();
        let rock = &e.rocks[r];
        let p = rock.px.iter().copied().min_by_key(|&(x, y)| dist2(x, y, rock.cx, rock.cy)).unwrap();
        (r, p)
    }

    fn factory(a: &Arena) -> Option<usize> {
        (0..a.ships.len()).find(|&i| a.ships[i].alive && a.ships[i].team == 0 && a.ships[i].role == Role::Ship && a.ships[i].node)
    }

    #[test]
    fn the_map_is_a_mirror_image() {
        let a = quiet();
        for y in 0..a.h {
            for x in 0..a.w {
                assert_eq!(a.rock_at(x, y), a.rock_at(a.w - 1 - x, a.h - 1 - y), "({x},{y})");
            }
        }
        let k = a.eco.as_ref().unwrap().king;
        assert_eq!((a.ships[k[0]].x, a.ships[k[0]].y), (a.w - 1 - a.ships[k[1]].x, a.h - 1 - a.ships[k[1]].y));
    }

    #[test]
    fn a_seed_grows_its_factory_out_of_the_rock_pixel_by_pixel() {
        let mut a = quiet();
        let (r, target) = home_rock(&a);
        let rock = a.eco.as_ref().unwrap().rocks[r].clone();
        let before = a.rock_left(&rock);
        a.command(Cmd::Print { team: 0, slot: 4, target: Some(target) });
        // The king prints the seed (20 px at 6 px/s), it arrives by hyperspace, flies and lands.
        let mut last = 0;
        let mut grew = false;
        for _ in 0..15 * 120 {
            a.step();
            if let Some(f) = factory(&a) {
                let now = a.ships[f].px_now();
                assert!(now >= last && now <= last + 3, "it jumped from {last} to {now} px in a tick");
                last = now;
                if a.ships[f].grow.is_none() {
                    grew = true;
                    break;
                }
            }
        }
        assert!(grew, "the outpost never finished growing");
        let f = factory(&a).unwrap();
        let count = starter_deck()[4].stats().count;
        assert_eq!(a.ships[f].px_now(), count, "it should finish whole on a rock this big");
        // One rock pixel eaten for every pixel grown, bar the core the seed brought.
        assert_eq!(before - a.rock_left(&rock), count - 1);
        assert_eq!(a.eco.as_ref().unwrap().tally[0].planted, 1);
    }

    #[test]
    fn factories_print_a_pixel_at_a_time_and_wait_when_broke() {
        let mut a = quiet();
        let (_, target) = home_rock(&a);
        a.command(Cmd::Print { team: 0, slot: 4, target: Some(target) });
        while factory(&a).map_or(true, |f| a.ships[f].grow.is_some()) {
            a.step();
        }
        let f = factory(&a).unwrap();
        let k = a.eco.as_ref().unwrap().king[0];
        while !matches!(a.ships[f].making, Making::Child(_)) {
            a.step();
        }
        let Making::Child(c) = a.ships[f].making else { unreachable!() };
        assert_eq!(a.ships[c].name, "MINER");
        assert!(a.ships[c].maker == Some(f) && a.ships[c].px_now() >= 1);
        // Empty the bank: printing stops where it is.
        for i in 0..a.ships.len() {
            a.ships[i].store = 0;
        }
        let stuck = a.ships[c].px_now();
        for _ in 0..15 * 10 {
            a.step();
        }
        assert_eq!(a.ships[c].px_now(), stuck, "it printed with an empty bank");
        assert_eq!(a.ships[f].making, Making::Child(c));
        // Pixels arrive: it carries on, then launches the finished miner.
        a.ships[k].store = 100;
        for _ in 0..15 * 30 {
            a.step();
        }
        assert!(a.ships[c].maker.is_none() && a.ships[c].px_now() == a.ships[c].px_full(), "the miner never launched");
        assert!(a.eco.as_ref().unwrap().tally[0].printed >= 1);
    }

    #[test]
    fn a_factory_builds_only_what_its_factory_pixels_allow() {
        // A tiny factory (2 factory pixels: ships up to 20 px) with a frigate (44 px) first on its
        // list skips straight to the fighter.
        let mut deck = starter_deck();
        let mut tiny = Design::from_rows("TINY", Order::Hold, &["hfh", "hch", "hfh"]);
        tiny.seeded = true;
        tiny.build = vec![2, 0];
        deck[4] = tiny;
        let mut a = Arena::for_match(1, [deck, starter_deck()], [false, false]);
        let (_, target) = home_rock(&a);
        a.command(Cmd::Print { team: 0, slot: 4, target: Some(target) });
        while factory(&a).map_or(true, |f| !matches!(a.ships[f].making, Making::Child(_))) {
            a.step();
        }
        let f = factory(&a).unwrap();
        let Making::Child(c) = a.ships[f].making else { unreachable!() };
        assert_eq!(a.ships[c].name, "FIGHTER");
    }

    #[test]
    fn linked_nodes_share_a_bank_and_split_where_the_pixels_sit() {
        let mut a = quiet();
        let (_, target) = home_rock(&a);
        a.command(Cmd::Print { team: 0, slot: 4, target: Some(target) });
        while factory(&a).is_none() {
            a.step();
        }
        let f = factory(&a).unwrap();
        let k = a.eco.as_ref().unwrap().king[0];
        a.step();
        a.ships[f].store = 50;
        let both = a.ships[k].store + 50;
        assert_eq!(a.bank(k), both, "the king and its outpost should be one network");
        // Fly the king far off: the network splits, each half keeping what it holds.
        for _ in 0..15 * 8 {
            a.command(Cmd::Steer { team: 0, dx: 1, dy: -1 });
            a.step();
        }
        assert!(a.eco.as_ref().unwrap().split_at[0] > 0, "the split went unnoticed");
        assert_eq!(a.bank(f), a.ships[f].store);
        assert_eq!(a.bank(k), a.ships[k].store);
    }

    #[test]
    fn a_destroyed_node_spills_what_it_held() {
        let mut a = quiet();
        let k = a.eco.as_ref().unwrap().king[1];
        let store = a.ships[k].store;
        let n = a.debris.len();
        a.ships[k].hp.iter_mut().for_each(|h| *h = 0);
        a.step();
        assert!(!a.ships[k].alive);
        assert!(a.debris.len() - n >= store.min(SPILL_CAP) as usize);
        assert_eq!(a.over, Some(Some(0)), "killing the red king wins for blue");
    }

    #[test]
    fn the_king_flies_straight_from_its_keys() {
        let mut a = quiet();
        let k = a.eco.as_ref().unwrap().king[0];
        let (x, y) = (a.ships[k].x, a.ships[k].y);
        a.command(Cmd::Steer { team: 0, dx: 1, dy: 0 });
        for _ in 0..10 {
            a.step();
        }
        assert_eq!((a.ships[k].x, a.ships[k].y), (x + 10 * KING_SPEED, y));
    }

    #[test]
    fn the_kings_builds_arrive_by_hyperspace() {
        let mut a = quiet();
        a.command(Cmd::Print { team: 0, slot: 0, target: None });
        let n = a.ships.len();
        while a.ships.len() == n {
            a.step();
        }
        let i = n;
        assert_eq!(a.ships[i].name, "FIGHTER");
        assert_eq!(a.ships[i].warp, WARP_TICKS);
        assert_eq!(a.eco.as_ref().unwrap().tally[0].summoned, 1);
        for _ in 0..WARP_TICKS {
            a.step();
        }
        assert_eq!(a.ships[i].warp, 0);
    }

    #[test]
    fn ai_matches_end_and_replay_exactly() {
        let run = || {
            let mut a = Arena::for_match(2, [starter_deck(), starter_deck()], [true, true]);
            while a.over.is_none() {
                a.step();
            }
            (a.over, a.tick, a.score[0].mined, a.score[1].mined)
        };
        let first = run();
        assert!(first.0.is_some() && first.1 < MATCH_LIMIT, "{first:?}");
        assert_eq!(run(), first, "the same seed played differently");
    }

    /// Living pixels two ships have in the same place.
    fn shared(a: &Arena, i: usize, j: usize) -> usize {
        let (si, sj) = (&a.ships[i], &a.ships[j]);
        (0..si.hp.len()).filter(|&c| si.hp[c] > 0 && { let (x, y) = si.cell_xy(c); sj.cell_at(x, y).is_some_and(|d| sj.hp[d] > 0) }).count()
    }

    #[test]
    fn one_side_cant_plant_a_factory_on_top_of_another() {
        // Blue grows a yard, then sends a second seed to the very same rock pixel. The second
        // comes down on the nearest rock the first isn't on and grows up against it, not through
        // it: stacking factories on one spot would otherwise be free armour.
        let mut yard = Design::from_rows("YARD", Order::Hold, &["hhhhhhh", "hfffffh", "hfhhhfh", "hfhchfh", "hfhhhfh", "hfffffh", "hhhhhhh"]);
        yard.seeded = true;
        let mut blue = starter_deck();
        blue[4] = yard;
        let mut a = Arena::for_match(1, [blue, starter_deck()], [false, false]);
        let (_, target) = home_rock(&a);
        let yards = |a: &Arena| -> Vec<usize> { (0..a.ships.len()).filter(|&i| a.ships[i].alive && a.ships[i].team == 0 && a.ships[i].role == Role::Ship && a.ships[i].name == "YARD").collect() };
        a.command(Cmd::Print { team: 0, slot: 4, target: Some(target) });
        while yards(&a).first().map_or(true, |&y| a.ships[y].px_now() < 49) {
            a.step();
            assert!(a.tick < 15 * 120, "the first yard never finished growing");
        }
        a.command(Cmd::Print { team: 0, slot: 4, target: Some(target) });
        let mut seen = 0;
        for _ in 0..15 * 90 {
            a.step();
            let y = yards(&a);
            if y.len() == 2 {
                seen += 1;
                assert_eq!(shared(&a, y[0], y[1]), 0, "the two yards share pixels at tick {}", a.tick);
            }
        }
        assert!(seen > 15 * 20, "the second seed never came down");
        let y = yards(&a);
        let (first, second) = (&a.ships[y[0]], &a.ships[y[1]]);
        assert_eq!(first.px_now(), 49, "the first yard should be whole");
        assert!(cheb(first.x, first.y, second.x, second.y) <= 12, "the second yard should have come down beside the first");
        assert!(second.px_now() > 5, "the second should have grown what it had room for: {} px", second.px_now());
        println!("second yard: {} of 49 px, {} px from the first", second.px_now(), cheb(first.x, first.y, second.x, second.y));
    }

    #[test]
    fn a_bigger_ship_flying_over_a_hulk_doesnt_push_it() {
        // Red's king, far bigger than a little plated hulk with no engines, is steered straight
        // over it. The hulk can't move itself, so it stays exactly where it is while the king
        // passes over. It used to be pushed out from under the king a pixel or two a tick.
        let hulk = Design::from_rows("HULK", Order::Hold, &["ppppp", "pfffp", "pfcfp", "pfffp", "ppppp"]);
        let mut a = Arena::for_match(1, [starter_deck(), starter_deck()], [false, false]);
        let k = a.eco.as_ref().unwrap().king[1];
        let (kx, ky) = (a.ships[k].x, a.ships[k].y);
        let y = a.spawn(0, 0, &hulk, kx - 40, ky, 0, false);
        let at = (a.ships[y].x, a.ships[y].y);
        a.command(Cmd::Steer { team: 1, dx: -1, dy: 0 });
        let mut over = 0;
        for _ in 0..40 {
            a.step();
            if !a.ships[y].alive {
                break;
            }
            assert_eq!((a.ships[y].x, a.ships[y].y), at, "the hulk was moved at tick {}", a.tick);
            over += (shared(&a, y, k) > 0) as i32;
        }
        assert!(over >= 3, "the king never flew over the hulk");
    }

    #[test]
    fn nothing_flies_through_a_planted_factory() {
        // The same little yard, planted this time. Red's king is steered straight at it and stops
        // against its plate: it never shares a pixel with it, however much bigger it is, and the
        // yard doesn't move. Steered diagonally instead, the king slides along the plate and past.
        let yard = Design::from_rows("YARD", Order::Hold, &["ppppp", "pfffp", "pfcfp", "pfffp", "ppppp"]);
        for slide in [false, true] {
            let mut a = Arena::for_match(1, [starter_deck(), starter_deck()], [false, false]);
            let k = a.eco.as_ref().unwrap().king[1];
            let (kx, ky) = (a.ships[k].x, a.ships[k].y);
            let y = a.spawn(0, 0, &yard, kx - 40, ky, 0, false);
            a.ships[y].rooted = true;
            let at = (a.ships[y].x, a.ships[y].y);
            a.command(Cmd::Steer { team: 1, dx: -1, dy: if slide { -1 } else { 0 } });
            let (mut held, mut last) = (0, (kx, ky));
            for _ in 0..40 {
                a.step();
                if !a.ships[y].alive {
                    break;
                }
                assert_eq!((a.ships[y].x, a.ships[y].y), at, "the yard was moved at tick {}", a.tick);
                assert_eq!(shared(&a, y, k), 0, "the king is inside the yard at tick {}", a.tick);
                let now = (a.ships[k].x, a.ships[k].y);
                held += (now == last) as i32;
                last = now;
            }
            if slide {
                assert!(a.ships[k].x < at.0, "the king should have slid past the yard: it's at {:?}, the yard at {at:?}", last);
            } else {
                assert!(a.ships[y].alive && held >= 10, "the king should have been held at the yard: held {held} ticks");
                assert!(a.ships[k].x > at.0 && a.ships[k].x - at.0 <= 14, "the king should be right up against the yard: {} px off", a.ships[k].x - at.0);
            }
        }
    }

    #[test]
    fn a_seeded_warship_grows_out_of_a_rock_and_flies_off() {
        // A seeded design may have engines and guns. It grows out of its rock like a factory,
        // planted and solid while it grows; once it's whole it lifts off and is a ship like any
        // other.
        let mut blue = starter_deck();
        let mut d = blue[0].clone();
        d.name = "SPROUT".into();
        d.seeded = true;
        assert!(d.stats().thrust > 0, "the test needs a design that can fly");
        blue[4] = d;
        let mut a = Arena::for_match(1, [blue, starter_deck()], [false, false]);
        let (_, target) = home_rock(&a);
        a.command(Cmd::Print { team: 0, slot: 4, target: Some(target) });
        let sprout = |a: &Arena| (0..a.ships.len()).find(|&i| a.ships[i].alive && a.ships[i].role == Role::Ship && a.ships[i].name == "SPROUT");
        let (mut landed, mut flew) = (None, false);
        for _ in 0..15 * 180 {
            a.step();
            let Some(i) = sprout(&a) else { continue };
            let s = &a.ships[i];
            if s.grow.is_some() {
                assert!(s.rooted, "growing but not planted");
                let at = *landed.get_or_insert((s.x, s.y));
                assert_eq!((s.x, s.y), at, "it moved while it was growing");
            } else if let Some(at) = landed {
                if cheb(s.x, s.y, at.0, at.1) > 10 {
                    assert!(!s.rooted, "flying but still planted");
                    assert_eq!(s.px_now(), s.px_full(), "it should have flown off whole");
                    flew = true;
                    break;
                }
            }
        }
        assert!(landed.is_some(), "the seed never came down");
        assert!(flew, "the grown ship never flew off");
    }

    #[test]
    fn two_sides_planting_on_one_rock_never_share_a_pixel() {
        // Both sides send a seed to the same rock pixel, with unarmed factories so nothing shoots
        // the seeds down. Blue gets there first. Red can't reach its pixel (blue's factory is on
        // it), comes down on the nearest free rock, grows round blue's factory, not through it,
        // and prints with what it managed to grow.
        let yard = |build: Vec<usize>| {
            let mut d = Design::from_rows("YARD", Order::Hold, &["hhhhh", "hfffh", "hfcfh", "hfffh", "hhhhh"]);
            d.seeded = true;
            d.build = build;
            d
        };
        let (mut blue, mut red) = (starter_deck(), starter_deck());
        blue[4] = yard(Vec::new());
        red[4] = yard(vec![0]);
        let mut a = Arena::for_match(1, [blue, red], [false, false]);
        let (_, target) = home_rock(&a);
        a.command(Cmd::Print { team: 0, slot: 4, target: Some(target) });
        a.command(Cmd::Print { team: 1, slot: 4, target: Some(target) });
        let both = |a: &Arena| -> Option<(usize, usize)> {
            let f = |t: usize| (0..a.ships.len()).find(|&i| a.ships[i].alive && a.ships[i].team == t && a.ships[i].role == Role::Ship && a.ships[i].node);
            Some((f(0)?, f(1)?))
        };
        let mut seen = 0;
        for _ in 0..15 * 120 {
            a.step();
            if let Some((b, r)) = both(&a) {
                seen += 1;
                assert_eq!(shared(&a, b, r), 0, "the two factories share pixels at tick {}", a.tick);
            }
        }
        assert!(seen > 15 * 30, "red's seed never came down beside blue's factory");
        let (b, r) = both(&a).unwrap();
        assert_eq!(a.ships[b].px_now(), a.ships[b].px_full(), "blue got there first and should be whole");
        let red = &a.ships[r];
        assert!(red.px_now() > 1 && red.px_now() < red.px_full(), "red should be wedged in part-grown: {} of {}", red.px_now(), red.px_full());
        assert!(cheb(red.x, red.y, a.ships[b].x, a.ships[b].y) <= 8, "red should be right beside blue");
        // Wedged with nowhere to grow, its factory pixels print instead of waiting for ever.
        assert!(red.st.factory > 0 && red.making != Making::None, "red's wedged factory never started on its build order: {:?}", red.making);
    }

    #[test]
    fn a_seed_whose_rock_is_gone_finds_another() {
        let mut a = quiet();
        let (r, target) = home_rock(&a);
        a.command(Cmd::Print { team: 0, slot: 4, target: Some(target) });
        // The seed is out and flying: its rock vanishes (mined or eaten by someone else).
        while !a.ships.iter().any(|s| s.alive && s.role == Role::Seed && s.warp == 0) {
            a.step();
        }
        let rock = a.eco.as_ref().unwrap().rocks[r].clone();
        for &(x, y) in &rock.px {
            let k = (y * a.w + x) as usize;
            a.rock[k] = false;
        }
        for _ in 0..15 * 60 {
            a.step();
        }
        assert_eq!(a.eco.as_ref().unwrap().tally[0].planted, 1, "the seed never came down anywhere");
        let f = factory(&a).unwrap();
        assert!(cheb(a.ships[f].x, a.ships[f].y, rock.cx, rock.cy) > 30, "it planted on the rock that had gone");
        assert!(!a.ships.iter().any(|s| s.alive && s.role == Role::Seed), "a seed is still hanging about");
    }

    #[test]
    fn the_books_balance_and_structures_keep_apart() {
        // Six minutes of AI against AI: every tick, what each side has mined plus its starting
        // bank equals what it holds, has spent on printing and lost with destroyed nodes. No
        // ship is off the map, and no two sides' structures share a pixel.
        let mut a = Arena::for_match(3, [starter_deck(), starter_deck()], [true, true]);
        while a.over.is_none() && a.tick < 15 * 360 {
            a.step();
            let e = a.eco.as_ref().unwrap();
            for t in 0..2 {
                let stored: i32 = a.ships.iter().filter(|s| s.team == t).map(|s| s.store).sum();
                assert_eq!(START_BANK + a.score[t].mined, stored + e.spent[t] + e.spilt[t], "team {t}'s books are off at tick {}", a.tick);
            }
            for s in a.ships.iter().filter(|s| s.alive) {
                let (l, t, r, b) = s.bbox();
                assert!(l >= 0 && t >= 0 && r < a.w && b < a.h && s.store >= 0, "{} is off the map or overdrawn", s.name);
            }
            if a.tick % 15 == 0 {
                let st: Vec<usize> = (0..a.ships.len()).filter(|&i| a.ships[i].alive && a.ships[i].role == Role::Ship && (a.ships[i].node || a.ships[i].grow.is_some())).collect();
                for (k, &i) in st.iter().enumerate() {
                    for &j in &st[k + 1..] {
                        if a.ships[i].team != a.ships[j].team {
                            assert_eq!(shared(&a, i, j), 0, "{} and {} share pixels", a.ships[i].name, a.ships[j].name);
                        }
                    }
                }
            }
        }
    }

    #[test]
    fn factories_short_of_pixels_take_turns() {
        // Two small yards in one network, one printing fighters and one lancers (7 px each), fed
        // one pixel a second when between them they could print four. Each should get half: before
        // turns, whichever was built first took the lot and the other printed nothing.
        let yard = |name: &str, build: Vec<usize>| {
            let mut d = Design::from_rows(name, Order::Hold, &["hhhhh", "hfffh", "hfcfh", "hfffh", "hhhhh"]);
            d.seeded = true;
            d.build = build;
            d
        };
        let mut deck = starter_deck();
        deck[4] = yard("YARD A", vec![0]);
        deck[5] = yard("YARD B", vec![1]);
        // Hold-order copies, so what's printed stays put and nothing wanders off to fight.
        deck[0].order = Order::Hold;
        deck[1].order = Order::Hold;
        let mut a = Arena::for_match(1, [deck, starter_deck()], [false, false]);
        let e = a.eco.as_ref().unwrap();
        let (kx, ky) = KING_STARTS[0];
        let mut near: Vec<usize> = (0..e.rocks.len()).collect();
        near.sort_by_key(|&r| dist2(e.rocks[r].cx, e.rocks[r].cy, kx, ky));
        let spot = |r: usize| e.rocks[r].px.iter().copied().min_by_key(|&(x, y)| dist2(x, y, e.rocks[r].cx, e.rocks[r].cy)).unwrap();
        let (t0, t1) = (spot(near[0]), spot(near[1]));
        a.command(Cmd::Print { team: 0, slot: 4, target: Some(t0) });
        a.command(Cmd::Print { team: 0, slot: 5, target: Some(t1) });
        let yards = |a: &Arena| a.ships.iter().filter(|s| s.alive && s.team == 0 && s.node && s.role == Role::Ship && s.grow.is_none()).count();
        while yards(&a) < 2 {
            a.step();
            assert!(a.tick < 15 * 120, "the two yards never finished growing");
        }
        let k = a.eco.as_ref().unwrap().king[0];
        let made = |a: &Arena, name: &str| a.ships.iter().filter(|s| s.team == 0 && s.name == name && s.maker.is_none() && s.grow.is_none()).count() as i32;
        let (f0, l0) = (made(&a, "FIGHTER"), made(&a, "LANCER"));
        for i in 0..a.ships.len() {
            a.ships[i].store = 0;
        }
        for t in 0..15 * 180 {
            if t % 15 == 0 {
                a.ships[k].store += 1;
            }
            a.step();
        }
        let (f, l) = (made(&a, "FIGHTER") - f0, made(&a, "LANCER") - l0);
        println!("two yards fed 180 px: {f} fighters, {l} lancers");
        assert!(f >= 8 && l >= 8 && (f - l).abs() <= 2, "180 px should make about 12 of each: {f} fighters, {l} lancers");
    }

    #[test]
    fn nothing_comes_out_of_hyperspace_into_an_enemy() {
        // A slab of enemy plate sits right where the king's fighter would arrive. The fighter
        // stays in hyperspace until the king moves and its spot is clear.
        let mut rows = vec!["ppppppppp"; 9];
        rows[4] = "ppppcpppp";
        let slab = Design::from_rows("SLAB", Order::Hold, &rows);
        let mut a = quiet();
        let k = a.eco.as_ref().unwrap().king[0];
        let (_, _, r, _) = a.ships[k].bbox();
        let slab_i = a.spawn(1, 0, &slab, r + 6, a.ships[k].y, 0, false);
        a.command(Cmd::Print { team: 0, slot: 0, target: None });
        let n = a.ships.len();
        for _ in 0..15 * 12 {
            a.step();
        }
        assert!(a.ships.len() > n, "the fighter was never delivered");
        let f = n;
        assert_eq!(a.ships[f].name, "FIGHTER");
        assert!(a.ships[f].warp > 0, "it came out of hyperspace inside the slab");
        // The king flies off; the fighter comes out beside it, clear of the slab.
        for _ in 0..15 * 4 {
            a.command(Cmd::Steer { team: 0, dx: 0, dy: 1 });
            a.step();
        }
        assert_eq!(a.ships[f].warp, 0, "with its spot clear it should have arrived");
        assert_eq!(shared(&a, f, slab_i), 0);
    }

    /// Enough of a match's state to tell two runs apart.
    fn fingerprint(a: &Arena) -> (u32, Option<Option<usize>>, usize, i64, [i32; 2]) {
        let sum: i64 = a.ships.iter().map(|s| (s.x as i64 * 31 + s.y as i64) * 7 + s.px_now() as i64 + s.store as i64 * 3 + s.alive as i64).sum();
        (a.tick, a.over, a.ships.len(), sum, [a.score[0].mined, a.score[1].mined])
    }

    #[test]
    fn a_match_replays_from_its_commands() {
        // Five minutes of AI against AI, keeping the log of every command. The same commands fed
        // on the same ticks into a match with no AI at all give the same match: commands are the
        // only thing that drives one.
        let mut a = Arena::for_match(4, [starter_deck(), starter_deck()], [true, true]);
        while a.over.is_none() && a.tick < 15 * 300 {
            a.step();
        }
        let log = a.eco.as_ref().unwrap().log.clone();
        assert!(log.len() >= 6, "only {} commands in five minutes", log.len());
        let mut b = Arena::for_match(4, [starter_deck(), starter_deck()], [false, false]);
        let mut k = 0;
        while b.tick < a.tick {
            while k < log.len() && log[k].0 == b.tick + 1 {
                b.command(log[k].1);
                k += 1;
            }
            b.step();
        }
        assert_eq!(k, log.len());
        assert_eq!(fingerprint(&a), fingerprint(&b), "the replay went differently");
        assert_eq!(b.eco.as_ref().unwrap().log, log, "the replay's own log should match");
    }
}
