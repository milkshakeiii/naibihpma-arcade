//! Movement principle 3 (see `motion.rs`): knots. When going around and giving way can't get
//! monsters past each other (the one that has to give way is boxed in by others, a line of them
//! meets another head-on in a lane, a dead end crowded from both ends), the monsters stuck in it
//! search real moves together: an exact search over where each of them stands, one step of one
//! of them at a time, never through anyone, with everyone else and every good as walls, for moves
//! that clear the way of the one first in line. They make those moves at their usual pace, then
//! wait off its way until it has gone by. If the search covers every tile they could reach and
//! finds no way, there is none without passing through: they say so ("WE NEED MORE ROOM HERE.")
//! and look again now and then. A search that runs out of looks before it's sure just tries
//! again later.

use crate::game::{Game, Sfx, StepRes};
use crate::motion::{dir_to, GiveWay};
use crate::world::DIRS;
use serde::{Deserialize, Serialize};
use std::cmp::Reverse;
use std::collections::{BinaryHeap, HashMap, VecDeque};

/// What monsters in a knot with no way out say.
pub(crate) const NO_ROOM: &str = "WE NEED MORE ROOM HERE.";
/// Ticks held up by another monster before searching for moves that get past.
const KNOT_AFTER: u16 = 45;
/// Ticks between searches while still held up.
pub(crate) const KNOT_RETRY: u16 = 45;
/// The most monsters one search moves.
const MOST_MOVERS: usize = 6;
/// The most tiles one search covers.
const MOST_TILES: usize = 400;
/// The most arrangements one search looks at, so it never takes long.
const MOST_LOOKS: usize = 6000;
/// Ticks a planned move may wait on someone outside the knot before the plan is dropped.
const STALL_TICKS: u16 = 30;

/// A knot being untangled: who's in it (by id; the first is the one whose way is being
/// cleared), the moves found (who steps onto which tile, in order), how far along they are, and
/// how long the next one has been held up.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Knot {
    pub members: Vec<u32>,
    pub moves: Vec<(u32, (i32, i32))>,
    pub next: usize,
    pub stalled: u16,
}

/// What a search found.
enum Found {
    Moves(Vec<(u32, (i32, i32))>),
    /// No way at all: every tile they could reach was searched.
    NoWay,
    /// It ran out of looks, or the area was too big to be sure.
    NotSure,
}

impl Game {
    fn by_id(&self, id: u32) -> Option<usize> {
        self.monsters.iter().position(|m| m.id == id)
    }

    /// Is monster `k` in a knot being untangled? Its loop waits until that's done.
    pub(crate) fn in_knot(&self, k: usize) -> bool {
        let id = self.monsters[k].id;
        self.monsters.iter().any(|m| m.knot.as_ref().is_some_and(|kn| kn.members.contains(&id)))
    }

    /// Can a search move monster `k`: a loop that isn't working, leading a wagon, giving way, or
    /// in another knot? (One you've left standing is yours to move.)
    fn movable(&self, k: usize) -> bool {
        let m = &self.monsters[k];
        m.playing && !m.busy && self.led_wagon(k).is_none() && m.giving_way.is_none() && !self.in_knot(k)
    }

    /// Is monster `k` held up by another monster standing on its way?
    fn held_up(&self, k: usize) -> bool {
        self.monsters[k].waiting_on.is_some_and(|o| self.way_of(k).contains(&(self.monsters[o].x, self.monsters[o].y)))
    }

    /// Each tick: knots being untangled make their next moves, and a monster held up long
    /// enough, first in line of those stuck with it, searches for a way.
    pub(crate) fn untangle(&mut self, sfx: &mut Vec<Sfx>) {
        for i in 0..self.monsters.len() {
            if self.monsters[i].knot.is_some() {
                self.run_knot(i, sfx);
            }
            let m = &mut self.monsters[i];
            m.no_room = m.no_room.saturating_sub(1);
        }
        for i in 0..self.monsters.len() {
            let wait = self.monsters[i].wait;
            if wait < KNOT_AFTER || (wait - KNOT_AFTER) % KNOT_RETRY != 0 || !self.movable(i) || !self.held_up(i) {
                continue;
            }
            let members = self.knot_members(i);
            let first = |k: usize| (self.monsters[k].since, k);
            if members.iter().any(|&k| k != i && self.held_up(k) && self.monsters[k].wait >= KNOT_AFTER && first(k) < first(i)) {
                continue;
            }
            self.try_knot(i);
        }
    }

    /// Searches the knot monster `lead` is stuck in, for moves that clear its way, and starts
    /// making them; with no way at all, everyone in it says so.
    pub(crate) fn try_knot(&mut self, lead: usize) {
        if !self.movable(lead) {
            return;
        }
        let members = self.knot_members(lead);
        if members.len() < 2 {
            return;
        }
        match self.search_knot(&members) {
            Found::Moves(moves) => {
                let members = members.iter().map(|&k| self.monsters[k].id).collect();
                self.monsters[lead].knot = Some(Knot { members, moves, next: 0, stalled: 0 });
            }
            Found::NoWay => {
                for &k in &members {
                    self.monsters[k].no_room = KNOT_RETRY + 1;
                }
            }
            Found::NotSure => {}
        }
    }

    /// Monster `i` and those stuck with it: whoever it waits on and whoever waits on it, and so
    /// on, as long as they can be moved. `i` comes first.
    fn knot_members(&self, i: usize) -> Vec<usize> {
        let mut members = vec![i];
        let mut queue = VecDeque::from([i]);
        while let Some(k) = queue.pop_front() {
            for j in 0..self.monsters.len() {
                if members.len() >= MOST_MOVERS {
                    return members;
                }
                // Held up in each other's way, not just waiting on something reserved.
                let linked = (self.monsters[k].waiting_on == Some(j) && self.held_up(k)) || (self.monsters[j].waiting_on == Some(k) && self.held_up(j));
                if linked && !members.contains(&j) && self.movable(j) {
                    members.push(j);
                    queue.push_back(j);
                }
            }
        }
        members
    }

    /// The search: over where each member stands, one step of one of them at a time onto a free
    /// tile, for an arrangement with no member but the first on the first's way (or where it
    /// stands now). Weighted A*, most promising first: the cost is steps taken, the estimate how
    /// far the others are from tiles off that way. Ties go in a fixed order, so the same world
    /// always gives the same moves.
    fn search_knot(&self, members: &[usize]) -> Found {
        let lead = members[0];
        let pos = |k: usize| (self.monsters[k].x, self.monsters[k].y);
        let mut keep = self.way_of(lead);
        keep.push(pos(lead));
        // The area: tiles they can reach through free tiles; anyone else there is a wall.
        let others: Vec<(i32, i32)> = (0..self.monsters.len()).filter(|k| !members.contains(k)).map(pos).collect();
        let free = |t: (i32, i32)| self.open(t.0, t.1) && !others.contains(&t);
        let mut tiles: Vec<(i32, i32)> = members.iter().map(|&k| pos(k)).collect();
        let mut index: HashMap<(i32, i32), u16> = tiles.iter().enumerate().map(|(n, &t)| (t, n as u16)).collect();
        let mut queue: VecDeque<(i32, i32)> = tiles.iter().copied().collect();
        let mut capped = false;
        while let Some(t) = queue.pop_front() {
            for d in DIRS {
                let n = (t.0 + d.delta().0, t.1 + d.delta().1);
                if index.contains_key(&n) || !free(n) {
                    continue;
                }
                if tiles.len() >= MOST_TILES {
                    capped = true;
                    continue;
                }
                index.insert(n, tiles.len() as u16);
                tiles.push(n);
                queue.push_back(n);
            }
        }
        let next_to: Vec<[u16; 4]> = tiles.iter().map(|&t| DIRS.map(|d| index.get(&(t.0 + d.delta().0, t.1 + d.delta().1)).copied().unwrap_or(u16::MAX))).collect();
        let kept: Vec<bool> = tiles.iter().map(|t| keep.contains(t)).collect();
        // How far each tile is from the nearest tile off the way.
        let mut off = vec![u32::MAX; tiles.len()];
        let mut queue: VecDeque<usize> = (0..tiles.len()).filter(|&n| !kept[n]).collect();
        for &n in &queue {
            off[n] = 0;
        }
        while let Some(n) = queue.pop_front() {
            for &m in &next_to[n] {
                if m != u16::MAX && off[m as usize] == u32::MAX {
                    off[m as usize] = off[n] + 1;
                    queue.push_back(m as usize);
                }
            }
        }
        let k = members.len();
        let get = |s: u128, j: usize| ((s >> (16 * j)) & 0xFFFF) as usize;
        let set = |s: u128, j: usize, v: usize| (s & !(0xFFFFu128 << (16 * j))) | ((v as u128) << (16 * j));
        let estimate = |s: u128| (1..k).map(|j| off[get(s, j)].min(1000)).sum::<u32>();
        let done = |s: u128| (1..k).all(|j| !kept[get(s, j)]);
        let start = (0..k).fold(0u128, |s, j| set(s, j, j));
        // Each arrangement seen: the one it came from, and the step taken (who, onto which tile).
        let mut seen: HashMap<u128, (u128, u8, u16)> = HashMap::from([(start, (start, 0, 0))]);
        let mut heap = BinaryHeap::from([Reverse((3 * estimate(start), 0u32, 0u32, start))]);
        let mut order = 0u32;
        let mut looks = 0;
        while let Some(Reverse((_, steps, _, s))) = heap.pop() {
            if done(s) {
                let mut moves = Vec::new();
                let mut at = s;
                while at != start {
                    let (prev, j, to) = seen[&at];
                    moves.push((self.monsters[members[j as usize]].id, tiles[to as usize]));
                    at = prev;
                }
                moves.reverse();
                return Found::Moves(moves);
            }
            looks += 1;
            if looks > MOST_LOOKS {
                return Found::NotSure;
            }
            for j in 0..k {
                let from = get(s, j);
                for &to in &next_to[from] {
                    if to == u16::MAX || (0..k).any(|o| get(s, o) == to as usize) {
                        continue;
                    }
                    let n = set(s, j, to as usize);
                    if seen.contains_key(&n) {
                        continue;
                    }
                    seen.insert(n, (s, j as u8, to));
                    order += 1;
                    heap.push(Reverse((steps + 1 + 3 * estimate(n), steps + 1, order, n)));
                }
            }
        }
        if capped {
            Found::NotSure
        } else {
            Found::NoWay
        }
    }

    /// The knot monster `i` is untangling makes its next moves, each by a monster that isn't
    /// mid-step (so nobody moves twice in a tick). Once they're all made, everyone but `i` waits
    /// off its way until it has gone by. A move held up by someone outside the knot for too
    /// long, or no longer possible, drops the plan.
    fn run_knot(&mut self, i: usize, sfx: &mut Vec<Sfx>) {
        let Some(mut knot) = self.monsters[i].knot.clone() else { return };
        let still_on = knot.members.iter().all(|&id| self.by_id(id).is_some_and(|k| self.monsters[k].playing));
        while still_on && knot.next < knot.moves.len() {
            let (id, to) = knot.moves[knot.next];
            let Some(k) = self.by_id(id) else { break };
            if !self.monsters[k].idle() {
                break;
            }
            let here = (self.monsters[k].x, self.monsters[k].y);
            let blocked = self.monster_at(to.0, to.1).is_some() || self.world.item_at(to.0, to.1).is_some();
            if (here.0 - to.0).abs() + (here.1 - to.1).abs() != 1 || blocked {
                knot.stalled += 1;
                break;
            }
            match self.try_step(k, dir_to(here, to), sfx) {
                StepRes::Moved(_) => {
                    knot.next += 1;
                    knot.stalled = 0;
                }
                StepRes::Blocked(_) => {
                    knot.stalled += 1;
                    break;
                }
            }
        }
        let lead = self.monsters[i].id;
        if knot.next >= knot.moves.len() && still_on {
            for &id in &knot.members[1..] {
                if let Some(k) = self.by_id(id) {
                    let m = &mut self.monsters[k];
                    m.giving_way = Some(GiveWay { to: lead, spot: (m.x, m.y), ticks: 0, at_spot: 0 });
                    m.path.clear();
                }
            }
            self.end_knot(i);
        } else if !still_on || knot.stalled > STALL_TICKS {
            self.end_knot(i);
        } else {
            self.monsters[i].knot = Some(knot);
        }
    }

    fn end_knot(&mut self, i: usize) {
        let ids = self.monsters[i].knot.take().map(|k| k.members).unwrap_or_default();
        for id in ids {
            if let Some(k) = self.by_id(id) {
                self.monsters[k].path.clear();
            }
        }
    }
}
