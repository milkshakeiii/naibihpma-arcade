//! How a looping monster gets where its goal takes it: walking to a spot, leading a wagon. (How
//! it moves a good is `carry.rs`'s planning, on top of this.)
//!
//! This is the one place where looping monsters' moves meet. Each takes the next step of its
//! own shortest way there, around walls, goods and parked wagons, and nobody ever moves onto a
//! tile someone else is on, so nobody passes through anybody. On top of that greedy baseline,
//! coordination is added one principle at a time, each with what it guarantees, measured by
//! the tangle tests:
//!
//! 1. **Go around anyone standing still.** A monster standing still (one you've left standing,
//!    one working at a spot, a loop that's waiting, or a wagon whose leader is any of those) is
//!    planned around like a wall, when there's a way around; one on the move is waited for, as it
//!    moves on. While waiting behind one standing still, a monster looks again for a way around
//!    every second. Guarantee: nobody waits for ever behind someone who isn't going anywhere when
//!    there's another way.
//! 2. **A loop that's only waiting gives way.** When a monster's only way is through a loop
//!    that's waiting (not working, not leading a wagon), that one gives way: it goes, through no
//!    one, to the nearest tile off the other's way, says "AFTER YOU.", and waits there until the
//!    other has gone by (or has stopped, or ten seconds have passed), then carries on with its own
//!    goal. Of two waiting on each other, each in the other's only way (head-on in a lane, one
//!    going into a dead end as the other comes out), the one later in line gives way: whoever has
//!    gone longest without finishing a goal goes first. Guarantee: two that are each in the
//!    other's way get past each other whenever the one later in line has somewhere off the
//!    other's way it can reach; when it hasn't (boxed in by others, or in a dead end), that's a
//!    knot, for the next principle, which keeps to the same order.
//! 3. **Knots get searched.** When those two can't do it (a third monster boxing in the one that
//!    has to give way, lines meeting head-on, a crowded dead end), the monsters stuck in it search
//!    real moves together and make them: see `knot.rs`. Guarantee: a way out is found whenever
//!    one exists among them within the search's reach; when the search covered everything they
//!    could reach and found none, they say "WE NEED MORE ROOM HERE."

use crate::game::{Game, Sfx, StepRes};
use crate::world::{Dir, DIRS};
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;

/// What a monster says when no path at all leads where it's going.
pub(crate) const NO_WAY: &str = "I CAN'T FIND A WAY THERE.";
/// What a monster waiting on another in its way says.
pub(crate) const EXCUSE_ME: &str = "EXCUSE ME!";
/// Ticks between looks for a way around someone standing still, while waiting behind them.
pub(crate) const LOOK_AROUND_EVERY: u16 = 15;
/// What a monster giving way says.
const AFTER_YOU: &str = "AFTER YOU.";
/// Ticks held up by a waiting loop, with no way around, before one of them gives way.
pub(crate) const SETTLE_AFTER: u16 = 3;
/// The longest a monster gives way for, in ticks.
const GIVE_WAY_MAX: u16 = 150;
/// Ticks the one given way to can stand still before the one giving way stops waiting for it.
const PASSER_STOPPED: u16 = 10;

/// A monster giving way: to whom (by id), the tile off their way it's waiting on, for how long
/// it has been giving way, and how long it has been waiting on that tile.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct GiveWay {
    pub to: u32,
    pub spot: (i32, i32),
    pub ticks: u16,
    #[serde(default)]
    pub at_spot: u16,
}

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
    pub(crate) fn open(&self, x: i32, y: i32) -> bool {
        self.world.walkable(x, y) && self.world.item_at(x, y).is_none() && !self.applicant_at(x, y) && !self.world.cart_at(x, y)
    }

    /// What a monster held up by another says: having found there's no way through at all, that.
    pub(crate) fn excuse_me(&self, i: usize) -> String {
        if self.monsters[i].no_room > 0 { crate::knot::NO_ROOM.into() } else { EXCUSE_ME.into() }
    }

    /// Is monster `k` standing still: not on its way anywhere this moment? One you've left
    /// standing, one working at a spot, or a loop that's waiting.
    pub(crate) fn standing_still(&self, k: usize) -> bool {
        let m = &self.monsters[k];
        m.slide.is_none() && (!m.playing || m.busy || m.wait > 0)
    }

    /// The tiles of everyone but monster `i` standing still, and of wagons whose leaders are.
    pub(crate) fn still_tiles(&self, i: usize) -> Vec<(i32, i32)> {
        let still = |k: usize| k != i && self.standing_still(k);
        let mut tiles: Vec<(i32, i32)> = (0..self.monsters.len()).filter(|&k| still(k)).map(|k| (self.monsters[k].x, self.monsters[k].y)).collect();
        for w in &self.world.wagons {
            if w.leader.and_then(|id| self.monsters.iter().position(|m| m.id == id)).is_some_and(still) {
                tiles.extend([w.pumpkin, w.at]);
            }
        }
        tiles
    }

    /// A way for monster `i` from `from` to `to` around everyone standing still if there is
    /// one, otherwise straight through them (to wait for them).
    fn plan_for(&self, i: usize, from: (i32, i32), to: (i32, i32)) -> Option<Vec<(i32, i32)>> {
        self.plan_avoiding(from, to, &self.still_tiles(i)).or_else(|| self.plan(from, to))
    }

    /// Is someone standing still on the next tile of monster `i`'s way, and is it time to look
    /// again for a way around them?
    fn look_around(&self, i: usize) -> bool {
        let m = &self.monsters[i];
        m.wait % LOOK_AROUND_EVERY == 0 && m.path.first().and_then(|&(x, y)| self.monster_at(x, y)).is_some_and(|o| o != i && self.standing_still(o))
    }

    /// A shortest path, not counting `from`, through other monsters. Breadth-first in a fixed
    /// direction order, so the same world always gives the same path (saves replay exactly).
    pub(crate) fn plan(&self, from: (i32, i32), to: (i32, i32)) -> Option<Vec<(i32, i32)>> {
        self.plan_avoiding(from, to, &[])
    }

    /// The same, never through the tiles in `avoid`.
    fn plan_avoiding(&self, from: (i32, i32), to: (i32, i32), avoid: &[(i32, i32)]) -> Option<Vec<(i32, i32)>> {
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
                if n.0 < 0 || n.1 < 0 || n.0 >= w || n.1 >= h || prev[idx(n)] != usize::MAX || !self.open(n.0, n.1) || avoid.contains(&n) {
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
            if self.monsters[i].wait >= SETTLE_AFTER && self.standing_still(o) {
                self.settle(i, o);
            }
            return Going::Waiting(self.excuse_me(i));
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
        if path.last() != Some(&to) || !path.first().is_some_and(|&n| adjacent(here, n)) || self.look_around(i) {
            match self.plan_for(i, here, to) {
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
        if !path.first().is_some_and(|&n| adjacent(here, n)) || self.look_around(i) {
            let still = self.still_tiles(i);
            match self.plan_lead([here, p, at], bed, pumpkin, &still).or_else(|| self.plan_lead([here, p, at], bed, pumpkin, &[])) {
                Some(path) if !path.is_empty() => self.monsters[i].path = path,
                _ => return Going::Stuck("I CAN'T GET THE WAGON THERE.".into()),
            }
        }
        self.step_on(i, sfx)
    }

    /// The tiles monster `k` is about to pass through: its planned walk, and where the good it's
    /// moving and it will go.
    pub(crate) fn way_of(&self, k: usize) -> Vec<(i32, i32)> {
        let m = &self.monsters[k];
        let mut way = m.path.clone();
        if let Some(mut b) = m.carry {
            for &(d, pull) in &m.carry_plan {
                let (dx, dy) = d.delta();
                let next = (b.0 + dx, b.1 + dy);
                // Pushing, it stands behind the good; pulling, it stands ahead and backs away.
                way.extend(if pull { [b, next, (b.0 + 2 * dx, b.1 + 2 * dy)] } else { [(b.0 - dx, b.1 - dy), b, next] });
                b = next;
            }
        }
        way
    }

    /// Monster `i` is held up by `o`, standing still in its only way. If `o` is a loop that's
    /// only waiting, it gives way. Of two waiting on each other, the one later in line gives way,
    /// or the other if that one has nowhere to go.
    pub(crate) fn settle(&mut self, i: usize, o: usize) {
        if self.monsters[i].giving_way.is_some() || self.monsters[o].giving_way.is_some() || self.in_knot(i) || self.in_knot(o) {
            return;
        }
        // One working stays at it, one leading a wagon can't step aside, and one you've left
        // standing is yours to move.
        let movable = |k: usize| {
            let m = &self.monsters[k];
            m.playing && !m.busy && self.led_wagon(k).is_none()
        };
        if !movable(o) || self.monsters[o].wait == 0 {
            return;
        }
        // Each in the other's way: not just waiting on something the other has reserved. Then the
        // one later in line gives way to the one first in line.
        let mutual = self.monsters[o].waiting_on == Some(i) && self.way_of(o).contains(&(self.monsters[i].x, self.monsters[i].y));
        let first = |k: usize| (self.monsters[k].since, k);
        let (lead, giver) = if mutual && movable(i) && first(o) < first(i) { (o, i) } else { (i, o) };
        if self.start_giving_way(giver, lead) {
            return;
        }
        // The one that should give way is boxed in: that's a knot. Search it for moves that clear
        // the way of the one first in line (now, then now and then while still held up), rather
        // than have that one give way to someone later, which could happen over and over.
        let w = self.monsters[i].wait;
        if w == SETTLE_AFTER || (w > SETTLE_AFTER && (w - SETTLE_AFTER) % crate::knot::KNOT_RETRY == 0) {
            self.try_knot(lead);
        }
    }

    /// Monster `g` starts giving way to `p`: off to the nearest tile off `p`'s way it can reach
    /// through no one. False if there's none.
    fn start_giving_way(&mut self, g: usize, p: usize) -> bool {
        let mut keep = self.way_of(p);
        keep.push((self.monsters[p].x, self.monsters[p].y));
        let Some(path) = self.retreat(g, &keep) else { return false };
        let spot = path[path.len() - 1];
        let to = self.monsters[p].id;
        let m = &mut self.monsters[g];
        m.giving_way = Some(GiveWay { to, spot, ticks: 0, at_spot: 0 });
        m.path = path;
        self.say(g, AFTER_YOU, false);
        true
    }

    /// The way for monster `g`, through no one, to the nearest tile not in `keep`.
    fn retreat(&self, g: usize, keep: &[(i32, i32)]) -> Option<Vec<(i32, i32)>> {
        let (w, h) = (self.world.w, self.world.h);
        let idx = |(x, y): (i32, i32)| (y * w + x) as usize;
        let from = (self.monsters[g].x, self.monsters[g].y);
        let mut prev = vec![usize::MAX; (w * h) as usize];
        prev[idx(from)] = idx(from);
        let mut queue = VecDeque::from([from]);
        while let Some(q) = queue.pop_front() {
            if q != from && !keep.contains(&q) {
                let mut path = vec![q];
                let mut at = idx(q);
                while prev[at] != idx(from) {
                    at = prev[at];
                    path.push(((at as i32) % w, (at as i32) / w));
                }
                path.reverse();
                return Some(path);
            }
            for d in DIRS {
                let n = (q.0 + d.delta().0, q.1 + d.delta().1);
                if n.0 < 0 || n.1 < 0 || n.0 >= w || n.1 >= h || prev[idx(n)] != usize::MAX || !self.open(n.0, n.1) || self.monster_at(n.0, n.1).is_some() {
                    continue;
                }
                prev[idx(n)] = idx(q);
                queue.push_back(n);
            }
        }
        None
    }

    /// Has monster `p` gone by one giving way to it: nothing it's about to pass is next to the
    /// spot any more? Or, with the way clear for a while, has it not moved, so it isn't coming?
    fn gone_by(&self, p: usize, gw: GiveWay) -> bool {
        let near = |t: &(i32, i32)| (t.0 - gw.spot.0).abs() + (t.1 - gw.spot.1).abs() <= 1;
        let stopped = gw.at_spot >= PASSER_STOPPED && self.standing_still(p) && self.monsters[p].wait >= gw.at_spot;
        stopped || !self.way_of(p).iter().any(near)
    }

    /// One tick of monster `g` giving way: off to its spot, then waiting there until the one it
    /// gave way to has gone by.
    pub(crate) fn give_way_tick(&mut self, g: usize, sfx: &mut Vec<Sfx>) -> Going {
        let Some(mut gw) = self.monsters[g].giving_way else { return Going::Moving };
        gw.ticks = gw.ticks.saturating_add(1);
        if (self.monsters[g].x, self.monsters[g].y) == gw.spot {
            gw.at_spot = gw.at_spot.saturating_add(1);
        }
        self.monsters[g].giving_way = Some(gw);
        let passer = self.monsters.iter().position(|m| m.id == gw.to);
        if gw.ticks >= GIVE_WAY_MAX || passer.is_none_or(|p| self.gone_by(p, gw)) {
            self.end_give_way(g);
            return Going::Moving;
        }
        if (self.monsters[g].x, self.monsters[g].y) == gw.spot {
            return Going::Waiting(AFTER_YOU.into());
        }
        match self.go_toward(g, gw.spot, None, sfx) {
            Going::Stuck(_) => {
                self.end_give_way(g);
                Going::Moving
            }
            Going::Waiting(_) => Going::Waiting(AFTER_YOU.into()),
            _ => Going::Moving,
        }
    }

    pub(crate) fn end_give_way(&mut self, g: usize) {
        let m = &mut self.monsters[g];
        m.giving_way = None;
        m.path.clear();
    }

    /// The leader's way, for a wagon train standing at `[leader, pumpkin, bed]`, to end with its
    /// bed at `bed` and its pumpkin at `pumpkin`: each step, the pumpkin takes the tile the
    /// leader left and the bed the pumpkin's. Breadth-first over where the leader is and which
    /// way the pumpkin and bed trail behind it (a train can't back up, so turning round takes
    /// room), only over tiles wagons fit and not through the tiles in `avoid`.
    fn plan_lead(&self, [leader, pumpkin, bed]: [(i32, i32); 3], to_bed: (i32, i32), to_pumpkin: (i32, i32), avoid: &[(i32, i32)]) -> Option<Vec<(i32, i32)>> {
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
        let rolls = |t: (i32, i32)| inside(t) && self.world.wagon_way(t.0, t.1) && self.world.item_at(t.0, t.1).is_none() && !self.applicant_at(t.0, t.1) && (!self.world.cart_at(t.0, t.1) || t == pumpkin || t == bed) && !avoid.contains(&t);
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
