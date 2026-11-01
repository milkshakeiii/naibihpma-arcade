//! How a looping monster gets somewhere: a recorded stretch of plain walking replays as "go to
//! this tile", and the monster finds its own way there around items and other monsters. One in
//! the way of a monster that goes before it makes way: steps aside, or backs up ahead of it
//! until it can. One blocked by a monster that isn't moving waits a moment, then looks for a
//! way around. Nobody passes through anybody: every move is onto a tile that's free right then.
//! Pushes, pulls and the ferry still replay step by step.

use crate::game::{Game, Sfx, StepRes, WALK_TICKS};
use crate::world::{Dir, Slide, DIRS};
use std::collections::VecDeque;

/// Ticks a monster waits behind another before looking for a way around it.
pub(crate) const REPLAN_AFTER: u16 = 6;
/// What a monster says when no path at all leads where it's going.
pub(crate) const NO_WAY: &str = "I CAN'T FIND A WAY THERE.";
/// How long a line of monsters, the one that needs the tile included, can make way in turn.
const MAKE_WAY_DEPTH: usize = 4;

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
    pub(crate) fn open(&self, x: i32, y: i32, around_monsters: bool) -> bool {
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
            // One that's waiting there and comes after this one in line makes way.
            let keep: Vec<(i32, i32)> = std::iter::once(here).chain(self.monsters[i].path.iter().copied()).collect();
            (self.monsters[i].waiting_on, self.monsters[i].held_up) = (Some(o), true);
            if self.make_way(o, &keep, i) {
                return Going::Waiting(self.excuse_me(i));
            }
            let m = &mut self.monsters[i];
            m.blocked += 1;
            if m.blocked >= REPLAN_AFTER {
                m.blocked = 0;
                if let Some(p) = self.plan(here, to, true) {
                    self.monsters[i].path = p;
                }
            }
            return Going::Waiting(self.excuse_me(i));
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

    /// What a monster held up by another says: having found no way through at all, that.
    pub(crate) fn excuse_me(&self, i: usize) -> String {
        if self.monsters[i].no_room > 0 { crate::tangle::NO_ROOM.into() } else { "EXCUSE ME!".into() }
    }

    /// Looping monster `o`, waiting in the way of `for_whom`, makes way if `for_whom` goes
    /// before it in line (see `outranks`).
    pub(crate) fn make_way(&mut self, o: usize, keep_clear: &[(i32, i32)], for_whom: usize) -> bool {
        self.outranks(for_whom, o) && self.move_over(o, keep_clear, for_whom, &mut vec![for_whom])
    }

    /// Monster `o`, waiting, moves off its tile for `for_whom`: onto a free tile beside it, clear
    /// of `keep_clear` if it can be (else further along it, backing up ahead of `for_whom` until
    /// it can step aside), or onto the tile of one beside it that makes way in turn (if that
    /// one, too, comes after `for_whom`). Never onto a tile anyone is on, so nobody passes
    /// through anybody. `chain` is who is making way already.
    pub(crate) fn move_over(&mut self, o: usize, keep_clear: &[(i32, i32)], for_whom: usize, chain: &mut Vec<usize>) -> bool {
        if !self.free_to_move(o) || self.monsters[o].wait == 0 {
            return false;
        }
        chain.push(o);
        let here = (self.monsters[o].x, self.monsters[o].y);
        // Off the way first, then onto one that makes way; free tiles before occupied ones.
        let mut options: Vec<(bool, bool, usize, (i32, i32))> = Vec::new();
        for (k, d) in DIRS.iter().enumerate() {
            let n = (here.0 + d.delta().0, here.1 + d.delta().1);
            if !self.open(n.0, n.1, false) {
                continue;
            }
            match self.monster_at(n.0, n.1) {
                None => options.push((keep_clear.contains(&n), false, k, n)),
                Some(p) if chain.len() < MAKE_WAY_DEPTH && !chain.contains(&p) && self.outranks(for_whom, p) => options.push((keep_clear.contains(&n), true, k, n)),
                Some(_) => {}
            }
        }
        options.sort();
        for (_, taken, _, n) in options {
            if taken && !self.monster_at(n.0, n.1).is_some_and(|p| self.move_over(p, keep_clear, for_whom, chain)) {
                continue;
            }
            self.shift(o, n);
            return true;
        }
        chain.pop();
        false
    }

    /// Monster `o` steps onto the free tile `to` beside it, out of someone's way.
    fn shift(&mut self, o: usize, to: (i32, i32)) {
        let m = &mut self.monsters[o];
        let d = dir_to((m.x, m.y), to);
        (m.x, m.y) = to;
        m.facing = d;
        if matches!(d, Dir::Left | Dir::Right) {
            m.flip = d == Dir::Left;
        }
        m.slide = Some(Slide { dir: d, left: WALK_TICKS, total: WALK_TICKS });
        m.bump = None;
        m.path.clear();
        (m.waiting_on, m.held_up) = (None, false);
    }
}
