//! Untangling: looping monsters stuck in each other's way where nobody can just step aside (a
//! dead end, a side pocket, a doorway crowded from both sides, a good in the way). A search over
//! real moves of only the monsters stuck there and the goods near them finds the cheapest that
//! get the one first in line a tile nearer where it's going: someone backs out of a dead end,
//! ducks into a pocket, or pushes a good aside. They make those moves in order, then carry on
//! with their loops.
//!
//! Nobody passes through anybody: every move, here as everywhere, is onto a tile that's free at
//! that moment, so no two monsters trade tiles and no ring of them turns at once. The search is
//! exact within its window: if a way exists there, it's found. When none does (a one-tile
//! tunnel or dead end packed from both ends, a room filled solid), the monsters say so and wait
//! for room.

use crate::game::{Act, Game, Sfx, StepRes, PULL_TICKS, PUSH_TICKS, WALK_TICKS};
use crate::world::{Dir, Tile, DIRS};
use serde::{Deserialize, Serialize};
use std::cmp::Reverse;
use std::collections::{BinaryHeap, HashMap, VecDeque};

/// What a monster says when there's no way through without passing through someone.
pub(crate) const NO_ROOM: &str = "WE NEED MORE ROOM HERE.";
/// Ticks held up by another monster before looking for moves that get it by.
const UNTANGLE_AFTER: u16 = 10;
/// Ticks before looking again, having found none.
const ROOM_RETRY: u16 = 45;
/// Ticks a planned move may wait on someone outside the plan before the plan is dropped.
const STALL_TICKS: u16 = 20;
/// The windows searched in turn: tiles round the tangle, how many monsters and goods may move
/// in it, and how many arrangements to look at, at most.
const WINDOWS: [(i32, usize, usize, usize); 2] = [(5, 3, 2, 20_000), (9, 4, 3, 100_000)];
/// The most monsters and goods one search moves.
const MOST: usize = 7;

/// One move of an untangling.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Move {
    Walk,
    /// Into a good, pushing it ahead.
    Push,
    /// Backing away from a good, dragging it along.
    Pull,
}

/// Real moves under way so a monster can get by.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Untangle {
    /// Who moves, which way, and how, in order.
    pub moves: VecDeque<(usize, Dir, Move)>,
    /// Everyone it may move, who leave their loops be until it's done.
    pub members: Vec<usize>,
    /// Ticks the next move has waited on someone outside it.
    pub stalled: u16,
}

/// What getting by means for the monster first in line.
enum Goal {
    /// Walking: standing on any tile nearer where it's going (around items, monsters aside).
    Nearer { dist: Vec<u32>, now: u32 },
    /// Moving a good: that good on this tile, its next move made.
    Good((i32, i32)),
    /// On steps played as recorded: these tiles clear of monsters, without it moving at all.
    Clear(Vec<(i32, i32)>),
}

fn apart(a: (i32, i32), b: (i32, i32)) -> i32 {
    (a.0 - b.0).abs() + (a.1 - b.1).abs()
}

/// An arrangement searched: where each monster that may move stands, then where each good that
/// may move lies (as tiles of the window; unused slots are `u16::MAX`).
type State = [u16; MOST];

impl Game {
    /// Where monster `k` comes in line when monsters are in each other's way (least first).
    /// Whoever can't step aside goes first: a looping monster on steps played as recorded,
    /// which can't go around anyone either. Then one with a step in hand, and last one with
    /// nothing to do. Within each, whoever has gone longest without finishing a step (or since
    /// it last gave one up as impossible); then by roster order.
    ///
    /// So the first in line is never moved back by anyone, and who's first changes only when
    /// it finishes its step, or when one that has waited longer takes up a step; one that then
    /// gives that up goes to the back. Nobody can keep undoing the first one's progress.
    pub(crate) fn rank(&self, k: usize) -> (u8, u32, usize) {
        let m = &self.monsters[k];
        let class = if self.on_recorded_steps(k) {
            0
        } else if m.task.is_some() {
            1
        } else {
            2
        };
        (class, m.since, k)
    }

    /// Does monster `a` go before monster `b`? One walking itself home goes before any loop.
    pub(crate) fn outranks(&self, a: usize, b: usize) -> bool {
        !self.monsters[a].playing || self.rank(a) < self.rank(b)
    }

    /// Is looping monster `k` leading a wagon, on the ferry, or partway through steps played as
    /// recorded? Then it can't step aside or go around anyone.
    pub(crate) fn on_recorded_steps(&self, k: usize) -> bool {
        let m = &self.monsters[k];
        let literal = m.task.is_some() && matches!(m.routine.as_ref().and_then(|r| r.acts.get(m.pc)), Some(Act::Step(..) | Act::Pull(..)));
        m.playing && (literal || self.led_wagon(k).is_some() || self.world.on_deck(m.x, m.y))
    }

    /// Can looping monster `k` be moved out of someone's way right now? Not while it's moving,
    /// working, on steps played as recorded, or already making room for someone.
    pub(crate) fn free_to_move(&self, k: usize) -> bool {
        let m = &self.monsters[k];
        m.playing && m.idle() && !self.on_recorded_steps(k) && !self.in_untangle(k)
    }

    /// Is monster `k` part of an untangling under way?
    pub(crate) fn in_untangle(&self, k: usize) -> bool {
        self.monsters.iter().any(|m| m.untangle.as_ref().is_some_and(|u| u.members.contains(&k)))
    }

    /// The first in line of the looping monsters held up a while by one that comes after it,
    /// is waiting too, and can't just step aside, looks for moves that get it by. Finding none,
    /// it and the one in its way say they need more room, and it looks again a while later.
    pub(crate) fn untangle_if_held(&mut self) {
        for m in self.monsters.iter_mut() {
            m.no_room = m.no_room.saturating_sub(1);
        }
        // One that can't be moved at all (on recorded steps too) is a wall to the search.
        let waiting = |k: usize| {
            let m = &self.monsters[k];
            m.playing && m.idle() && m.untangle.is_none() && !self.in_untangle(k)
        };
        let stuck_in_way = |k: usize, o: usize| o != k && waiting(o) && self.monsters[o].wait > 0 && self.outranks(k, o);
        let held = |k: usize| {
            let m = &self.monsters[k];
            m.held_up && m.wait >= UNTANGLE_AFTER && m.no_room == 0 && waiting(k) && m.waiting_on.is_some_and(|o| stuck_in_way(k, o))
        };
        let Some(p) = (0..self.monsters.len()).filter(|&k| held(k)).min_by_key(|&k| self.rank(k)) else { return };
        if !self.start_untangle(p) {
            for k in std::iter::once(p).chain(self.monsters[p].waiting_on) {
                self.monsters[k].no_room = ROOM_RETRY;
            }
        }
    }

    /// Looks for the cheapest real moves that get monster `p` by, of `p`, the monsters round it
    /// that come after it in line, and the goods near it, in a small window and then a larger
    /// one, and sets them going. False if there are none.
    fn start_untangle(&mut self, p: usize) -> bool {
        let Some(goal) = self.untangle_goal(p) else { return false };
        for &(reach, monsters, goods, budget) in &WINDOWS {
            let Some((members, moves)) = self.search(p, &goal, reach, monsters, goods, budget) else { continue };
            let name = self.monsters[p].name.clone();
            for &k in members.iter().filter(|&&k| k != p) {
                self.say(k, &format!("AFTER YOU, {name}."), false);
            }
            self.monsters[p].untangle = Some(Untangle { moves, members, stalled: 0 });
            return true;
        }
        false
    }

    /// What getting by means for monster `p`: on steps played as recorded, the way its next one
    /// needs cleared; moving a good, the good's next move made; otherwise a tile nearer where
    /// it's walking.
    fn untangle_goal(&self, p: usize) -> Option<Goal> {
        let m = &self.monsters[p];
        if self.on_recorded_steps(p) {
            let way = self.recorded_way(p);
            return (!way.is_empty()).then_some(Goal::Clear(way));
        }
        if let (Some(at), Some(&(d, _))) = (m.carry, m.carry_plan.first()) {
            self.world.item_at(at.0, at.1)?;
            return Some(Goal::Good((at.0 + d.delta().0, at.1 + d.delta().1)));
        }
        let &to = m.path.last()?;
        let dist = self.distances_to(to);
        let now = dist[(m.y * self.world.w + m.x) as usize];
        (now != u32::MAX && now > 0).then_some(Goal::Nearer { dist, now })
    }

    /// Walking distances to `to` from every tile, around items (monsters aside).
    fn distances_to(&self, to: (i32, i32)) -> Vec<u32> {
        let (w, h) = (self.world.w, self.world.h);
        let idx = |(x, y): (i32, i32)| (y * w + x) as usize;
        let mut dist = vec![u32::MAX; (w * h) as usize];
        dist[idx(to)] = 0;
        let mut queue = VecDeque::from([to]);
        while let Some(q) = queue.pop_front() {
            for d in DIRS {
                let n = (q.0 + d.delta().0, q.1 + d.delta().1);
                if n.0 >= 0 && n.1 >= 0 && n.0 < w && n.1 < h && dist[idx(n)] == u32::MAX && self.open(n.0, n.1, false) {
                    dist[idx(n)] = dist[idx(q)] + 1;
                    queue.push_back(n);
                }
            }
        }
        dist
    }

    /// The search: A* over where the monsters and goods that may move are, one move at a time
    /// (so every move is onto a tile free at that moment), costed in ticks (walks, pushes, very
    /// slow pulls), in a fixed order so the same tangle always gets the same moves. Everything
    /// else stays where it is. Gives everyone that may move (all of them wait until it's done)
    /// and the moves, or nothing if there's no way within `budget` arrangements.
    fn search(&self, p: usize, goal: &Goal, reach: i32, most_monsters: usize, most_goods: usize, budget: usize) -> Option<(Vec<usize>, VecDeque<(usize, Dir, Move)>)> {
        let (w, h) = (self.world.w, self.world.h);
        let pos = |k: usize| (self.monsters[k].x, self.monsters[k].y);
        // The window: round `p` and the monsters it's waiting on, one waiting on the next.
        let mut chain = vec![p];
        while chain.len() < most_monsters {
            match self.monsters[chain[chain.len() - 1]].waiting_on {
                Some(o) if !chain.contains(&o) => chain.push(o),
                _ => break,
            }
        }
        let (mut x0, mut y0, mut x1, mut y1) = (i32::MAX, i32::MAX, i32::MIN, i32::MIN);
        for &k in &chain {
            let (x, y) = pos(k);
            (x0, y0, x1, y1) = (x0.min(x), y0.min(y), x1.max(x), y1.max(y));
        }
        let (x0, y0, x1, y1) = ((x0 - reach).max(0), (y0 - reach).max(0), (x1 + reach).min(w - 1), (y1 + reach).min(h - 1));
        let inside = |(x, y): (i32, i32)| x >= x0 && y >= y0 && x <= x1 && y <= y1;
        let ww = x1 - x0 + 1;
        let cell = |(x, y): (i32, i32)| ((y - y0) * ww + (x - x0)) as u16;
        let at_cell = |c: u16| (x0 + c as i32 % ww, y0 + c as i32 / ww);
        let cells = (ww * (y1 - y0 + 1)) as usize;

        // Who may move: `p` (unless its steps are played as recorded), then those it's waiting
        // on, then the nearest others in the window; all of them after `p` in line and free to
        // move.
        let clear = match goal {
            Goal::Clear(tiles) => tiles.as_slice(),
            _ => &[],
        };
        let may = |k: usize| if k == p { clear.is_empty() } else { self.free_to_move(k) && self.outranks(p, k) };
        let mut movers: Vec<usize> = chain.iter().copied().filter(|&k| may(k)).collect();
        let mut others: Vec<(i32, usize)> = (0..self.monsters.len()).filter(|&k| !movers.contains(&k) && inside(pos(k)) && may(k)).map(|k| (apart(pos(k), pos(p)), k)).collect();
        others.sort();
        movers.extend(others.into_iter().map(|(_, k)| k));
        movers.truncate(most_monsters);

        // Clearing a way for steps played as recorded: one that won't move is in the way for good.
        if clear.iter().any(|&t| !inside(t) || self.monster_at(t.0, t.1).is_some_and(|k| !movers.contains(&k))) {
            return None;
        }

        // Goods that may move: `p`'s own first, then the nearest that the others are moving or
        // that lie loose on plain ground; none being worked. Each is moved only by whoever is
        // moving it, a loose one by anyone. None for steps played as recorded, which push just
        // what they push.
        let plain = |(x, y): (i32, i32)| matches!(self.world.tile(x, y), Tile::Floor | Tile::Grass | Tile::Path | Tile::Door | Tile::Bridge);
        let item_at = |k: usize| (self.world.items[k].x, self.world.items[k].y);
        let carrier = |k: usize| movers.iter().position(|&m| self.monsters[m].carry == Some(item_at(k)));
        let mine = self.monsters[p].carry.and_then(|(x, y)| self.world.item_at(x, y));
        let mut goods: Vec<usize> = Vec::new();
        if let Goal::Good(to) = goal {
            let k = mine?;
            if self.world.items[k].locked || !inside(item_at(k)) || !inside(*to) {
                return None;
            }
            goods.push(k);
        }
        let mut loose: Vec<(i32, usize)> = (0..self.world.items.len())
            .filter(|&k| !goods.contains(&k) && inside(item_at(k)) && !self.world.items[k].locked)
            .filter(|&k| carrier(k).is_some() || (self.is_stray(&self.world.items[k]) && plain(item_at(k))))
            .map(|k| (apart(item_at(k), pos(p)), k))
            .collect();
        loose.sort();
        goods.extend(loose.into_iter().map(|(_, k)| k));
        goods.truncate(if clear.is_empty() { most_goods } else { 0 });
        let owner: Vec<Option<usize>> = goods.iter().map(|&k| carrier(k)).collect();
        let (nm, ng) = (movers.len(), goods.len());

        // What each tile of the window allows, with everything not searched held where it is.
        let goal_cell = match goal {
            Goal::Good(to) => Some(cell(*to)),
            _ => None,
        };
        let clear: Vec<u16> = clear.iter().map(|&t| cell(t)).collect();
        let mut stand = vec![false; cells];
        let mut lie = vec![false; cells];
        for c in 0..cells {
            let t = at_cell(c as u16);
            let monster = self.monster_at(t.0, t.1).is_some_and(|k| !movers.contains(&k));
            let item = self.world.item_at(t.0, t.1).is_some_and(|k| !goods.contains(&k));
            let clear = !monster && !item && !self.world.on_deck(t.0, t.1) && !self.world.cart_at(t.0, t.1) && !self.applicant_at(t.0, t.1);
            stand[c] = clear && self.world.walkable(t.0, t.1);
            // A good goes where it's meant to (a pad, a bin), and otherwise only onto plain ground.
            lie[c] = clear && (goal_cell == Some(c as u16) || (self.world.item_floor(t.0, t.1) && plain(t)));
        }

        let done = |s: &State| match goal {
            Goal::Nearer { dist, now } => {
                let (x, y) = at_cell(s[0]);
                dist[(y * w + x) as usize] < *now
            }
            Goal::Good(_) => Some(s[nm]) == goal_cell,
            Goal::Clear(_) => !s[..nm].iter().any(|c| clear.contains(c)),
        };
        let guess = |s: &State| match goal {
            Goal::Nearer { dist, now } => {
                let (x, y) = at_cell(s[0]);
                dist[(y * w + x) as usize].saturating_sub(now - 1).min(1000) * WALK_TICKS as u32
            }
            Goal::Good(to) => apart(at_cell(s[nm]), *to) as u32 * PUSH_TICKS as u32,
            Goal::Clear(_) => s[..nm].iter().filter(|c| clear.contains(c)).count() as u32 * WALK_TICKS as u32,
        };

        let mut start: State = [u16::MAX; MOST];
        for (j, &k) in movers.iter().enumerate() {
            start[j] = cell(pos(k));
        }
        for (j, &k) in goods.iter().enumerate() {
            start[nm + j] = cell(item_at(k));
        }
        // Each arrangement found: how, its cost so far, and the one it came from.
        let mut nodes: Vec<(State, u32, usize, (usize, Dir, Move))> = vec![(start, 0, usize::MAX, (p, Dir::Up, Move::Walk))];
        let mut best: HashMap<State, u32> = HashMap::from([(start, 0)]);
        let mut heap = BinaryHeap::from([Reverse((guess(&start), 0usize))]);
        let mut looked = 0;
        while let Some(Reverse((_, id))) = heap.pop() {
            let (s, cost) = (nodes[id].0, nodes[id].1);
            if best.get(&s).is_some_and(|&b| b < cost) {
                continue;
            }
            if done(&s) {
                let mut moves = VecDeque::new();
                let mut at = id;
                while nodes[at].2 != usize::MAX {
                    moves.push_front(nodes[at].3);
                    at = nodes[at].2;
                }
                return Some((movers, moves));
            }
            looked += 1;
            if looked > budget {
                return None;
            }
            let who = |c: u16| (0..nm + ng).find(|&e| s[e] == c);
            let moves_good = |e: usize, j: usize| e >= nm && owner[e - nm].is_none_or(|o| o == j);
            let mut next: Vec<(State, u32, (usize, Dir, Move))> = Vec::new();
            for j in 0..nm {
                let q = at_cell(s[j]);
                for d in DIRS {
                    let (dx, dy) = d.delta();
                    let n = (q.0 + dx, q.1 + dy);
                    if !inside(n) || !stand[cell(n) as usize] {
                        continue;
                    }
                    match who(cell(n)) {
                        None => {
                            let mut t = s;
                            t[j] = cell(n);
                            next.push((t, WALK_TICKS as u32, (movers[j], d, Move::Walk)));
                            // Or back into it, dragging the good in front along.
                            let front = (q.0 - dx, q.1 - dy);
                            if let Some(e) = Some(front).filter(|&f| inside(f)).and_then(|f| who(cell(f))).filter(|&e| moves_good(e, j)) {
                                if lie[s[j] as usize] {
                                    let mut t = s;
                                    (t[j], t[e]) = (cell(n), s[j]);
                                    next.push((t, PULL_TICKS as u32, (movers[j], d, Move::Pull)));
                                }
                            }
                        }
                        Some(e) if moves_good(e, j) => {
                            let beyond = (n.0 + dx, n.1 + dy);
                            if inside(beyond) && lie[cell(beyond) as usize] && who(cell(beyond)).is_none() {
                                let mut t = s;
                                (t[j], t[e]) = (cell(n), cell(beyond));
                                next.push((t, PUSH_TICKS as u32, (movers[j], d, Move::Push)));
                            }
                        }
                        Some(_) => {}
                    }
                }
            }
            for (t, step, how) in next {
                let c = cost + step;
                if best.get(&t).is_none_or(|&b| c < b) {
                    best.insert(t, c);
                    nodes.push((t, c, id, how));
                    heap.push(Reverse((c + guess(&t), nodes.len() - 1)));
                }
            }
        }
        None
    }

    /// Moves under way: each is made as soon as its monster is free; one held up too long by
    /// someone outside them, or one of them stopped partway, drops the rest, and everyone goes
    /// back to its loop.
    pub(crate) fn run_untangles(&mut self, sfx: &mut Vec<Sfx>) {
        for p in 0..self.monsters.len() {
            let Some(mut u) = self.monsters[p].untangle.take() else { continue };
            let mut off = u.members.iter().any(|&k| !self.monsters[k].playing);
            while let Some(&(k, d, how)) = u.moves.front().filter(|_| !off) {
                if !self.monsters[k].idle() {
                    break;
                }
                let (dx, dy) = d.delta();
                let ahead = (self.monsters[k].x + dx, self.monsters[k].y + dy);
                let as_planned = match how {
                    Move::Walk => self.world.item_at(ahead.0, ahead.1).is_none(),
                    Move::Push => self.world.item_at(ahead.0, ahead.1).is_some(),
                    Move::Pull => self.pulled_item(k, d).is_some(),
                };
                let res = match how {
                    _ if !as_planned => StepRes::Blocked(String::new()),
                    Move::Pull => self.try_pull(k, d, sfx),
                    _ => self.try_step(k, d, sfx),
                };
                if let StepRes::Moved(_) = res {
                    u.moves.pop_front();
                    u.stalled = 0;
                    continue;
                }
                u.stalled += 1;
                off = u.stalled > STALL_TICKS;
                break;
            }
            if off || u.moves.is_empty() {
                self.end_untangle(&u);
            } else {
                self.monsters[p].untangle = Some(u);
            }
        }
    }

    /// Everyone goes back to its loop, planning its way afresh from where it now is.
    fn end_untangle(&mut self, u: &Untangle) {
        for &k in &u.members {
            let m = &mut self.monsters[k];
            (m.path, m.carry_plan, m.blocked, m.wait) = (Vec::new(), Vec::new(), 0, 0);
            (m.waiting_on, m.held_up) = (None, false);
        }
    }
}
