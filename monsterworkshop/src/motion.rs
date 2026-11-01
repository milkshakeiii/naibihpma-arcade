//! How a looping monster gets where its goal takes it: walking to a spot, leading a wagon. (How
//! it moves a good is `carry.rs`'s planning, on top of this.)
//!
//! This is the one place where looping monsters' moves meet, and for now they meet greedily:
//! each takes the next step of its own shortest way there, around walls, goods and parked
//! wagons but not around other monsters, and when another monster stands on that tile, it
//! waits. Nobody ever moves onto a tile someone else is on, so nobody passes through anybody.
//! It's a baseline, and it jams when two meet head-on in a lane; coordination gets added on top
//! of it one principle at a time, each with what it guarantees, measured by the tangle tests.

use crate::game::{Game, Sfx, StepRes};
use crate::world::{Dir, DIRS};
use std::collections::VecDeque;

/// What a monster says when no path at all leads where it's going.
pub(crate) const NO_WAY: &str = "I CAN'T FIND A WAY THERE.";
/// What a monster waiting on another in its way says.
pub(crate) const EXCUSE_ME: &str = "EXCUSE ME!";

pub(crate) enum Going {
    Arrived,
    Moving,
    Waiting(String),
    /// It can't be done from here, now: no way there.
    Stuck(String),
}

pub(crate) fn dir_to((x, y): (i32, i32), (nx, ny): (i32, i32)) -> Dir {
    match (nx - x, ny - y) {
        (1, _) => Dir::Right,
        (-1, _) => Dir::Left,
        (_, 1) => Dir::Down,
        _ => Dir::Up,
    }
}

fn adjacent(a: (i32, i32), b: (i32, i32)) -> bool {
    (a.0 - b.0).abs() + (a.1 - b.1).abs() == 1
}

impl Game {
    /// Can a walking monster pass this tile? Never through items (it would push them), wagons
    /// or applicants. Other monsters don't count: they move on.
    fn open(&self, x: i32, y: i32) -> bool {
        self.world.walkable(x, y) && self.world.item_at(x, y).is_none() && !self.applicant_at(x, y) && !self.world.cart_at(x, y)
    }

    /// A shortest path, not counting `from`. Breadth-first in a fixed direction order, so the
    /// same world always gives the same path (saves replay exactly).
    pub(crate) fn plan(&self, from: (i32, i32), to: (i32, i32)) -> Option<Vec<(i32, i32)>> {
        let (w, h) = (self.world.w, self.world.h);
        let idx = |(x, y): (i32, i32)| (y * w + x) as usize;
        let mut prev = vec![usize::MAX; (w * h) as usize];
        prev[idx(from)] = idx(from);
        let mut queue = VecDeque::from([from]);
        while let Some((x, y)) = queue.pop_front() {
            if (x, y) == to {
                break;
            }
            for d in DIRS {
                let (dx, dy) = d.delta();
                let n = (x + dx, y + dy);
                if n.0 < 0 || n.1 < 0 || n.0 >= w || n.1 >= h || prev[idx(n)] != usize::MAX || !self.open(n.0, n.1) {
                    continue;
                }
                prev[idx(n)] = idx((x, y));
                queue.push_back(n);
            }
        }
        if prev[idx(to)] == usize::MAX {
            return None;
        }
        let mut path = vec![to];
        let mut at = idx(to);
        while prev[at] != idx(from) {
            at = prev[at];
            path.push(((at as i32) % w, (at as i32) / w));
        }
        path.reverse();
        Some(path)
    }

    /// Takes the next step of a planned way (`self.monsters[i].path`), or waits for whoever is on
    /// that tile.
    fn step_on(&mut self, i: usize, sfx: &mut Vec<Sfx>) -> Going {
        let here = (self.monsters[i].x, self.monsters[i].y);
        let next = self.monsters[i].path[0];
        if let Some(o) = self.monster_at(next.0, next.1) {
            self.monsters[i].waiting_on = Some(o);
            return Going::Waiting(EXCUSE_ME.into());
        }
        match self.try_step(i, dir_to(here, next), sfx) {
            StepRes::Moved(_) => {
                self.monsters[i].path.remove(0);
                Going::Moving
            }
            StepRes::Blocked(msg) => {
                // Something landed on the way since it was planned: plan again next time.
                self.monsters[i].path.clear();
                Going::Waiting(if msg.is_empty() { "SOMETHING'S IN MY WAY.".into() } else { msg })
            }
        }
    }

    fn face(&mut self, i: usize, d: Dir) {
        let m = &mut self.monsters[i];
        m.facing = d;
        if matches!(d, Dir::Left | Dir::Right) {
            m.flip = d == Dir::Left;
        }
    }

    /// One step of a looping monster's walk to `to`, ending facing `facing` if it matters.
    pub(crate) fn go_toward(&mut self, i: usize, to: (i32, i32), facing: Option<Dir>, sfx: &mut Vec<Sfx>) -> Going {
        let here = (self.monsters[i].x, self.monsters[i].y);
        if here == to {
            if let Some(d) = facing {
                self.face(i, d);
            }
            self.monsters[i].path.clear();
            return Going::Arrived;
        }
        let path = &self.monsters[i].path;
        if path.last() != Some(&to) || !path.first().is_some_and(|&n| adjacent(here, n)) {
            match self.plan(here, to) {
                Some(p) => self.monsters[i].path = p,
                None => return Going::Stuck(NO_WAY.into()),
            }
        }
        self.step_on(i, sfx)
    }

    /// One step of leading wagon `w` until its bed stands at `bed` with its pumpkin at
    /// `pumpkin`: up to the pumpkin first to take its lead, then along a way the three of them
    /// fit, and letting go once it's parked.
    pub(crate) fn lead_toward(&mut self, i: usize, w: usize, bed: (i32, i32), pumpkin: (i32, i32), sfx: &mut Vec<Sfx>) -> Going {
        let (at, p, leader) = (self.world.wagons[w].at, self.world.wagons[w].pumpkin, self.world.wagons[w].leader);
        let leading = self.led_wagon(i) == Some(w);
        if (at, p) == (bed, pumpkin) {
            if leading {
                self.let_go(w, sfx);
            }
            return Going::Arrived;
        }
        let here = (self.monsters[i].x, self.monsters[i].y);
        if !leading {
            if leader.is_some() {
                return Going::Waiting("SOMEONE ELSE IS LEADING THE WAGON.".into());
            }
            if adjacent(here, p) {
                self.face(i, dir_to(here, p));
                return match self.try_use(i, sfx) {
                    crate::game::UseRes::Done => Going::Moving,
                    crate::game::UseRes::Wait(msg) | crate::game::UseRes::Full(msg) | crate::game::UseRes::Missing(msg) => Going::Waiting(msg),
                    crate::game::UseRes::Nothing => Going::Waiting("WAITING FOR THE WAGON.".into()),
                };
            }
            // Up to the nearest side of the pumpkin it can reach.
            let sides = DIRS.iter().map(|d| (p.0 + d.delta().0, p.1 + d.delta().1)).filter(|&s| s != at && self.open(s.0, s.1));
            let Some((_, side)) = sides.filter_map(|s| self.plan(here, s).map(|path| (path.len(), s))).min() else {
                return Going::Stuck(NO_WAY.into());
            };
            return match self.go_toward(i, side, Some(dir_to(side, p)), sfx) {
                Going::Arrived => Going::Moving,
                other => other,
            };
        }
        let path = &self.monsters[i].path;
        if !path.first().is_some_and(|&n| adjacent(here, n)) {
            match self.plan_lead([here, p, at], bed, pumpkin) {
                Some(path) if !path.is_empty() => self.monsters[i].path = path,
                _ => return Going::Stuck("I CAN'T GET THE WAGON THERE.".into()),
            }
        }
        self.step_on(i, sfx)
    }

    /// The leader's way, for a wagon train standing at `[leader, pumpkin, bed]`, to end with its
    /// bed at `bed` and its pumpkin at `pumpkin`: each step, the pumpkin takes the tile the
    /// leader left and the bed the pumpkin's. Breadth-first over where the leader is and which
    /// way the pumpkin and bed trail behind it (a train can't back up, so turning round takes
    /// room), only over tiles wagons fit, and not counting other monsters.
    fn plan_lead(&self, [leader, pumpkin, bed]: [(i32, i32); 3], to_bed: (i32, i32), to_pumpkin: (i32, i32)) -> Option<Vec<(i32, i32)>> {
        let (w, h) = (self.world.w, self.world.h);
        let inside = |(x, y): (i32, i32)| x >= 0 && y >= 0 && x < w && y < h;
        let side = |a: (i32, i32), b: (i32, i32)| DIRS.iter().position(|d| (a.0 + d.delta().0, a.1 + d.delta().1) == b);
        let key = |l: (i32, i32), p: (i32, i32), b: (i32, i32)| Some(((l.1 * w + l.0) as usize * 16 + side(l, p)? * 4 + side(p, b)?) as u32);
        let unkey = |k: u32| {
            let cell = (k / 16) as i32;
            let l = (cell % w, cell / w);
            let dp = DIRS[(k % 16 / 4) as usize].delta();
            let p = (l.0 + dp.0, l.1 + dp.1);
            let db = DIRS[(k % 4) as usize].delta();
            (l, p, (p.0 + db.0, p.1 + db.1))
        };
        // The wagon's own tiles move with it; anything else in the way stays put.
        let rolls = |t: (i32, i32)| inside(t) && self.world.wagon_way(t.0, t.1) && self.world.item_at(t.0, t.1).is_none() && !self.applicant_at(t.0, t.1) && (!self.world.cart_at(t.0, t.1) || t == pumpkin || t == bed);
        let start = key(leader, pumpkin, bed)?;
        let mut prev = vec![u32::MAX; (w * h) as usize * 16];
        prev[start as usize] = start;
        let mut queue = VecDeque::from([start]);
        while let Some(k) = queue.pop_front() {
            let (l, p, b) = unkey(k);
            if (p, b) == (to_pumpkin, to_bed) {
                let mut way = Vec::new();
                let mut at = k;
                while at != start {
                    way.push(unkey(at).0);
                    at = prev[at as usize];
                }
                way.reverse();
                return Some(way);
            }
            // The pumpkin will stand where the leader is now.
            if !self.world.wagon_way(l.0, l.1) {
                continue;
            }
            for d in DIRS {
                let n = (l.0 + d.delta().0, l.1 + d.delta().1);
                if n == p || !rolls(n) {
                    continue;
                }
                let Some(nk) = key(n, l, p) else { continue };
                if prev[nk as usize] == u32::MAX {
                    prev[nk as usize] = k;
                    queue.push_back(nk);
                }
            }
        }
        None
    }
}
