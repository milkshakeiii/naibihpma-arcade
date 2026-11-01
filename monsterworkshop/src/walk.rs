//! How a looping monster gets somewhere: a recorded stretch of plain walking replays as "go to
//! this tile", and the monster finds its own way there around items and other monsters. Two
//! monsters walking straight at each other squeeze past; one blocked by a monster that isn't
//! moving waits a moment, then looks for a way around. Pushes, pulls and the ferry still
//! replay step by step.

use crate::game::{Act, Game, Sfx, StepRes, WALK_TICKS};
use crate::world::{Dir, Slide, DIRS};
use std::collections::VecDeque;

/// Ticks a monster waits behind another before looking for a way around it.
const REPLAN_AFTER: u16 = 6;

pub(crate) enum Going {
    Arrived,
    Moving,
    Waiting(String),
}

fn dir_to((x, y): (i32, i32), (nx, ny): (i32, i32)) -> Dir {
    match (nx - x, ny - y) {
        (1, _) => Dir::Right,
        (-1, _) => Dir::Left,
        (_, 1) => Dir::Down,
        _ => Dir::Up,
    }
}

impl Game {
    /// Can a walking monster pass this tile? Never through items (it would push them), and
    /// never onto the ferry: crossing stays a recorded step.
    fn open(&self, x: i32, y: i32, around_monsters: bool) -> bool {
        self.world.walkable(x, y)
            && !self.world.on_deck(x, y)
            && self.world.item_at(x, y).is_none()
            && !self.applicant_at(x, y)
            && (!around_monsters || self.monster_at(x, y).is_none())
    }

    /// A shortest path, not counting `from`. Breadth-first in a fixed direction order, so the
    /// same world always gives the same path (saves replay exactly).
    pub(crate) fn plan(&self, from: (i32, i32), to: (i32, i32), around_monsters: bool) -> Option<Vec<(i32, i32)>> {
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
                if n.0 < 0 || n.1 < 0 || n.0 >= w || n.1 >= h || prev[idx(n)] != usize::MAX || !self.open(n.0, n.1, around_monsters) {
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

    /// One step of a looping monster's walk to `to`, ending facing `facing`.
    pub(crate) fn go_toward(&mut self, i: usize, to: (i32, i32), facing: Dir, sfx: &mut Vec<Sfx>) -> Going {
        let here = (self.monsters[i].x, self.monsters[i].y);
        if here == to {
            let m = &mut self.monsters[i];
            m.facing = facing;
            if matches!(facing, Dir::Left | Dir::Right) {
                m.flip = facing == Dir::Left;
            }
            m.path.clear();
            m.blocked = 0;
            return Going::Arrived;
        }
        let adjacent = |p: (i32, i32)| (p.0 - here.0).abs() + (p.1 - here.1).abs() == 1;
        let path_ok = self.monsters[i].path.last() == Some(&to) && self.monsters[i].path.first().is_some_and(|&n| adjacent(n));
        if !path_ok {
            match self.plan(here, to, false) {
                Some(p) => self.monsters[i].path = p,
                None => return Going::Waiting("I CAN'T FIND A WAY THERE.".into()),
            }
        }
        let next = self.monsters[i].path[0];
        if let Some(o) = self.monster_at(next.0, next.1) {
            // Walking straight at each other: squeeze past.
            let other = &self.monsters[o];
            let other_going = other.routine.as_ref().and_then(|r| r.acts.get(other.pc)).is_some_and(|a| matches!(a, Act::Go { .. }));
            if other.playing && other.idle() && other_going && other.path.first() == Some(&here) {
                self.squeeze_past(i, o);
                return Going::Moving;
            }
            let m = &mut self.monsters[i];
            m.blocked += 1;
            if m.blocked >= REPLAN_AFTER {
                m.blocked = 0;
                if let Some(p) = self.plan(here, to, true) {
                    self.monsters[i].path = p;
                }
            }
            return Going::Waiting("EXCUSE ME!".into());
        }
        if !self.open(next.0, next.1, false) {
            // Something landed on the way since the path was planned.
            self.monsters[i].path.clear();
            return Going::Waiting("SOMETHING'S IN MY WAY.".into());
        }
        match self.try_step(i, dir_to(here, next), sfx) {
            StepRes::Moved(_) => {
                self.monsters[i].path.remove(0);
                self.monsters[i].blocked = 0;
                Going::Moving
            }
            StepRes::Blocked(msg) => {
                self.monsters[i].path.clear();
                Going::Waiting(if msg.is_empty() { "WAITING...".into() } else { msg })
            }
        }
    }

    /// Two walking monsters trade places.
    fn squeeze_past(&mut self, a: usize, b: usize) {
        let (pa, pb) = ((self.monsters[a].x, self.monsters[a].y), (self.monsters[b].x, self.monsters[b].y));
        for (k, from, to) in [(a, pa, pb), (b, pb, pa)] {
            let d = dir_to(from, to);
            let m = &mut self.monsters[k];
            (m.x, m.y) = to;
            m.facing = d;
            if matches!(d, Dir::Left | Dir::Right) {
                m.flip = d == Dir::Left;
            }
            m.slide = Some(Slide { dir: d, left: WALK_TICKS, total: WALK_TICKS });
            m.bump = None;
            m.blocked = 0;
            if m.path.first() == Some(&to) {
                m.path.remove(0);
            }
        }
    }
}
