//! How a looping monster gets somewhere: a recorded stretch of plain walking replays as "go to
//! this tile", and the monster finds its own way there around items and other monsters. Two
//! monsters walking straight at each other squeeze past; one blocked by a monster that isn't
//! moving waits a moment, then looks for a way around. Pushes, pulls and the ferry still
//! replay step by step.

use crate::game::{Act, Game, Sfx, StepRes, WALK_TICKS};
use crate::world::{Dir, Slide, DIRS};
use std::collections::VecDeque;

/// Ticks a monster waits behind another before looking for a way around it.
pub(crate) const REPLAN_AFTER: u16 = 6;
/// What a monster says when no path at all leads where it's going.
pub(crate) const NO_WAY: &str = "I CAN'T FIND A WAY THERE.";

pub(crate) enum Going {
    Arrived,
    Moving,
    Waiting(String),
    /// It can't be done from here, now: no way there.
    Stuck(String),
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
            && !self.world.cart_at(x, y)
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
                None => return Going::Stuck(NO_WAY.into()),
            }
        }
        let next = self.monsters[i].path[0];
        if let Some(o) = self.monster_at(next.0, next.1) {
            // Walking straight at each other: squeeze past.
            let other = &self.monsters[o];
            let other_going = other.routine.as_ref().and_then(|r| r.acts.get(other.pc)).is_some_and(|a| matches!(a, Act::Go { .. }));
            let free_hands = self.led_wagon(i).is_none() && self.led_wagon(o).is_none();
            if other.playing && other.idle() && other_going && other.path.first() == Some(&here) && free_hands {
                self.squeeze_past(i, o);
                return Going::Moving;
            }
            // One that's only waiting there, and doesn't need to be, steps aside.
            let keep: Vec<(i32, i32)> = std::iter::once(here).chain(self.monsters[i].path.iter().copied()).collect();
            self.monsters[i].waiting_on = Some(o);
            if self.make_way(o, &keep, i) {
                return Going::Waiting("EXCUSE ME!".into());
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

    /// A looping monster that's only waiting, on a tile its current step doesn't need it on,
    /// steps aside for one that needs to get by: onto a free tile next to it, clear of
    /// `keep_clear`. (Waiting at its own work spot, ready to push a good, or on a recorded
    /// step, it stays put.)
    /// Nor does it block `for_whom` if that's who it's waiting on, ready or not. With nowhere
    /// to step, it trades places with `for_whom` if that one is right beside it.
    pub(crate) fn make_way(&mut self, o: usize, keep_clear: &[(i32, i32)], for_whom: usize) -> bool {
        let m = &self.monsters[o];
        if !m.playing || !m.idle() || m.wait == 0 || self.led_wagon(o).is_some() {
            return false;
        }
        let here = (m.x, m.y);
        // With no step in hand it needs nowhere.
        let needs_here = m.task.is_some()
            && match m.routine.as_ref().and_then(|r| r.acts.get(m.pc)) {
                Some(Act::Use) => self.use_spot(o, m.pc).is_none_or(|(p, _)| p == here),
                // Moving a good, only where it's about to push or pull from (or waits, ready).
                Some(Act::Move { .. }) => match (m.carry, m.carry_plan.first()) {
                    (Some(at), Some(&(d, pull))) => here == if pull { (at.0 + d.delta().0, at.1 + d.delta().1) } else { (at.0 - d.delta().0, at.1 - d.delta().1) },
                    _ => false,
                },
                Some(Act::Go { .. }) => false,
                _ => true,
            };
        if needs_here && self.monsters[o].waiting_on != Some(for_whom) {
            return false;
        }
        if self.step_aside(o, keep_clear) {
            return true;
        }
        let w = &self.monsters[for_whom];
        let beside = (here.0 - w.x).abs() + (here.1 - w.y).abs() == 1;
        if beside && w.idle() && self.led_wagon(for_whom).is_none() {
            self.squeeze_past(o, for_whom);
            return true;
        }
        false
    }

    /// Monster `o` steps onto a free tile next to it, clear of `keep_clear`, if there is one.
    pub(crate) fn step_aside(&mut self, o: usize, keep_clear: &[(i32, i32)]) -> bool {
        let here = (self.monsters[o].x, self.monsters[o].y);
        let beside = |d: Dir| (here.0 + d.delta().0, here.1 + d.delta().1);
        let Some(d) = DIRS.iter().copied().find(|&d| self.open(beside(d).0, beside(d).1, true) && !keep_clear.contains(&beside(d))) else {
            return false;
        };
        let m = &mut self.monsters[o];
        (m.x, m.y) = beside(d);
        m.facing = d;
        if matches!(d, Dir::Left | Dir::Right) {
            m.flip = d == Dir::Left;
        }
        m.slide = Some(Slide { dir: d, left: WALK_TICKS, total: WALK_TICKS });
        m.path.clear();
        true
    }

    /// Two walking monsters trade places.
    pub(crate) fn squeeze_past(&mut self, a: usize, b: usize) {
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
