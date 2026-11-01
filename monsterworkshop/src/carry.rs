//! Moving a good where a loop wants it (phase 2 of routines that scale). A recorded run of
//! pushes (and pulls) of one good is saved as its goal: "move the thread from the spinnery's
//! output mat to the loom house's pad". On replay the monster plans its own pushes from
//! wherever it stands, around whatever is in the way, pulling only when nothing else works. A
//! good already where it belongs needs nothing; a good not there yet is waited for. If the goal
//! is still taken (the loom busy with the last thread), the good is brought up next to it and
//! goes in the moment it's free.

use crate::econ::Claim;
use crate::game::{Game, Sfx, StepRes, PULL_TICKS, PUSH_TICKS, WALK_TICKS};
use crate::walk::{Going, REPLAN_AFTER};
use crate::world::{Dir, DIRS};
use std::cmp::Reverse;
use std::collections::{BinaryHeap, VecDeque};

/// After a plan fails, ticks before trying again (the world has to change first anyway).
const RETRY_TICKS: u16 = 15;

/// One move of the good: the way it goes, and whether it is pulled rather than pushed.
pub(crate) type CarryStep = (Dir, bool);

impl Game {
    /// Is this tile where the good is meant to end up a bin, board or wagon that takes it?
    pub(crate) fn takes_into(&self, (x, y): (i32, i32), good: &str) -> bool {
        self.world.bin_at(x, y).is_some_and(|b| self.world.bins[b].takes(good))
            || self.world.board_at(x, y).is_some_and(|b| !self.world.boards[b].done() && self.board_takes(b, good))
            || self.world.wagon_at(x, y).is_some_and(|w| self.world.wagons[w].good().is_none_or(|g| g == good))
    }

    /// The cheapest pushes and pulls that bring the good at `from` to `to`, for a monster
    /// standing at `me`, and what they cost. Costs are ticks: walking between moves, pushes, and
    /// very slow pulls. A* over (where the good is, which side of it the monster stands), over
    /// the whole map, so a route that has to go a long way round is still found. Other monsters
    /// are ignored (most move on) unless `around_monsters`. The goal counts as free: anything on
    /// it when this is asked is being worked, so about to be gone. Ties break in a fixed order,
    /// so the same world always gives the same plan.
    pub(crate) fn plan_carry(&self, from: (i32, i32), to: (i32, i32), me: (i32, i32), good: &str, around_monsters: bool) -> Option<(Vec<CarryStep>, u32)> {
        if from == to {
            return Some((Vec::new(), 0));
        }
        let into = self.takes_into(to, good);
        let (w, h) = (self.world.w, self.world.h);
        let cells = (w * h) as usize;
        let inside = |(x, y): (i32, i32)| x >= 0 && y >= 0 && x < w && y < h;
        let idx = |(x, y): (i32, i32)| (y * w + x) as usize;
        let at = |i: usize| (i as i32 % w, i as i32 / w);
        // What each tile allows, worked out once. The good itself has left `from` once it moves.
        let mut taken = vec![false; cells];
        for p in self.world.items.iter().map(|it| (it.x, it.y)).filter(|&p| p != from) {
            taken[idx(p)] = true;
        }
        let mut blocked = vec![false; cells];
        let monsters = self.monsters.iter().map(|m| (m.x, m.y)).filter(|&p| around_monsters && p != me);
        // A parked wagon stays put; one being led moves on, like a monster.
        let carts = self.world.wagons.iter().filter(|wg| around_monsters || wg.leader.is_none()).flat_map(|wg| [wg.at, wg.pumpkin]);
        let stands = self.world.stands.iter().zip(&self.applicants).filter(|(_, a)| a.is_some()).map(|(&s, _)| s);
        for p in monsters.chain(carts).chain(stands).filter(|&p| inside(p)) {
            blocked[idx(p)] = true;
        }
        let mut walk = vec![false; cells];
        let mut rest = vec![false; cells];
        for c in 0..cells {
            let p = at(c);
            let clear = !blocked[c] && !self.world.on_deck(p.0, p.1);
            walk[c] = !taken[c] && clear && self.world.walkable(p.0, p.1);
            rest[c] = (!taken[c] || p == to) && clear && self.world.item_floor(p.0, p.1);
        }
        let walks = |p: (i32, i32), good_at: (i32, i32)| inside(p) && p != good_at && walk[idx(p)];
        let rests = |p: (i32, i32)| inside(p) && rest[idx(p)];
        // A state is the good's tile and the side of it the monster stands on (4: anywhere, at the start).
        let side_of = |d: Dir| DIRS.iter().position(|&x| x == d).unwrap_or(0);
        let player = |b: (i32, i32), side: usize| if side < 4 { (b.0 + DIRS[side].delta().0, b.1 + DIRS[side].delta().1) } else { me };
        let state = |b: (i32, i32), side: usize| idx(b) * 5 + side;
        let est = |b: (i32, i32)| ((b.0 - to.0).abs() + (b.1 - to.1).abs()) as u32 * PUSH_TICKS as u32;
        let mut best = vec![u32::MAX; cells * 5];
        let mut came: Vec<Option<(usize, CarryStep)>> = vec![None; cells * 5];
        let mut heap = BinaryHeap::new();
        let mut seq = 0u32;
        let start = state(from, 4);
        best[start] = 0;
        heap.push(Reverse((est(from), 0u32, 0u32, start)));
        // Walking distances from the monster to the good's sides, reused between states.
        let mut dist = vec![u32::MAX; cells];
        let mut touched: Vec<usize> = Vec::new();
        let mut queue = VecDeque::new();
        let mut goal = None;
        while let Some(Reverse((_, _, cost, s))) = heap.pop() {
            if cost > best[s] {
                continue;
            }
            let (b, side) = (at(s / 5), s % 5);
            if b == to {
                goal = Some(s);
                break;
            }
            for &t in &touched {
                dist[t] = u32::MAX;
            }
            touched.clear();
            let p = player(b, side);
            if !inside(p) {
                continue;
            }
            // Only the good's four sides matter, so stop as soon as every one that can be stood
            // on has been reached; a side cut off from the monster still costs a full search.
            let beside = |q: (i32, i32)| (q.0 - b.0).abs() + (q.1 - b.1).abs() == 1;
            let mut left = DIRS.iter().filter(|d| walks((b.0 + d.delta().0, b.1 + d.delta().1), b)).count();
            dist[idx(p)] = 0;
            touched.push(idx(p));
            left -= (beside(p) && walks(p, b)) as usize;
            queue.clear();
            queue.push_back(p);
            while left > 0 {
                let Some(q) = queue.pop_front() else { break };
                for d in DIRS {
                    let n = (q.0 + d.delta().0, q.1 + d.delta().1);
                    if walks(n, b) && dist[idx(n)] == u32::MAX {
                        dist[idx(n)] = dist[idx(q)] + 1;
                        touched.push(idx(n));
                        queue.push_back(n);
                        left -= beside(n) as usize;
                    }
                }
            }
            let reach = |p: (i32, i32)| if inside(p) && dist[idx(p)] != u32::MAX { Some(dist[idx(p)]) } else { None };
            for d in DIRS {
                let (dx, dy) = d.delta();
                let ahead = (b.0 + dx, b.1 + dy);
                let mut try_move = |next: (i32, i32), side: usize, extra: u32, step: CarryStep| {
                    let ns = state(next, side);
                    let c = cost + extra;
                    if c < best[ns] {
                        best[ns] = c;
                        came[ns] = Some((s, step));
                        seq += 1;
                        heap.push(Reverse((c + est(next), seq, c, ns)));
                    }
                };
                // Push: stand behind it, and it goes ahead (into the goal itself, if that takes it).
                if let Some(w) = reach((b.0 - dx, b.1 - dy)) {
                    if (ahead == to && into) || rests(ahead) {
                        try_move(ahead, side_of(d.opposite()), w * WALK_TICKS as u32 + PUSH_TICKS as u32, (d, false));
                    }
                }
                // Pull: stand ahead of it, back away, and it follows into the tile left.
                if let Some(w) = reach(ahead) {
                    if rests(ahead) && walks((b.0 + 2 * dx, b.1 + 2 * dy), b) {
                        try_move(ahead, side_of(d), w * WALK_TICKS as u32 + PULL_TICKS as u32, (d, true));
                    }
                }
            }
        }
        let goal = goal?;
        let mut s = goal;
        let mut steps = Vec::new();
        while let Some((prev, step)) = came[s] {
            steps.push(step);
            s = prev;
        }
        steps.reverse();
        Some((steps, best[goal]))
    }

    /// One step toward a "move this good" goal: walk to where the next push or pull is made
    /// from, then make it.
    pub(crate) fn carry_toward(&mut self, i: usize, good: &str, from: (i32, i32), to: (i32, i32), sfx: &mut Vec<Sfx>) -> Going {
        let started = self.monsters[i].carry.is_some();
        let at = self.monsters[i].carry.unwrap_or(from);
        let step = self.monsters[i].pc;
        // Someone else put it where it belongs (pushed it into the bin, say): done.
        if started && at == to {
            let m = &mut self.monsters[i];
            (m.carry, m.carry_plan) = (None, Vec::new());
            if let Some(it) = self.world.item_at(to.0, to.1) {
                self.pass_on(i, step, it);
            }
            return Going::Arrived;
        }
        // Already where it belongs (and not another loop's): nothing to do but hand it on. Not
        // if a workshop is busy using that one, though: it's about to be gone, so bring this
        // one up ready to go in after it.
        let met = self.world.item_at(to.0, to.1).filter(|&k| {
            let it = &self.world.items[k];
            it.good() == good && !it.locked && self.usable(i, &it.data)
        });
        if let (false, Some(it)) = (started, met) {
            self.pass_on(i, step, it);
            return Going::Arrived;
        }
        let here_now = self.world.item_at(at.0, at.1).filter(|&k| self.world.items[k].good() == good);
        let Some(k) = here_now.filter(|&k| started || self.usable(i, &self.world.items[k].data)) else {
            // Not there yet, or used up or put away somewhere else (pushed elsewhere, it would
            // have been followed), or set aside for another loop.
            let m = &mut self.monsters[i];
            (m.carry, m.carry_plan) = (None, Vec::new());
            // If nothing is bringing one and an earlier step of this loop makes it, go back and
            // make it. Otherwise wait for one where the loop takes it from.
            if here_now.is_none() && !self.coming_for(i, from, good) {
                if let Some(j) = self.producer_of(i, step, &(Some(good.to_string()), from)) {
                    return Going::Redo(j);
                }
            }
            return Going::Waiting(format!("WAITING FOR {}.", self.good_name(good)));
        };
        if self.world.items[k].locked {
            return Going::Waiting("THAT'S BEING WORKED ON.".into());
        }
        // Something else sits where it's going and isn't being used up.
        if let Some(o) = self.world.item_at(to.0, to.1).filter(|&o| o != k && !self.world.items[o].locked) {
            // One of the kind this loop may use, that a later step of it takes from there:
            // that step clears the way, so do it first, then come back and bring this one.
            let ours = self.world.items[o].good() == good && self.usable(i, &self.world.items[o].data);
            if let Some(u) = self.fed_by(i, step, good).into_iter().next().filter(|_| ours) {
                self.world.items[o].data.claim = Some(Claim { monster: self.monsters[i].id, step: u });
                return Going::First(u);
            }
            // Otherwise wait for it to move on.
            return Going::Waiting(format!("THE {} IS IN THE WAY.", self.good_name(self.world.items[o].good())));
        }
        let here = (self.monsters[i].x, self.monsters[i].y);
        if self.monsters[i].carry_plan.is_empty() {
            if self.monsters[i].carry_retry > 0 {
                self.monsters[i].carry_retry -= 1;
                return Going::Waiting(format!("I CAN'T GET THE {} THERE.", self.good_name(good)));
            }
            match self.plan_carry(at, to, here, good, false) {
                Some((plan, _)) if plan.is_empty() => return Going::Arrived,
                Some((plan, _)) => self.monsters[i].carry_plan = plan,
                None => {
                    self.monsters[i].carry_retry = RETRY_TICKS;
                    return Going::Waiting(format!("I CAN'T GET THE {} THERE.", self.good_name(good)));
                }
            }
        }
        let (d, pull) = self.monsters[i].carry_plan[0];
        let (dx, dy) = d.delta();
        let next = (at.0 + dx, at.1 + dy);
        let stand = if pull { next } else { (at.0 - dx, at.1 - dy) };
        // Where the monster or the good ends up after this move: pushed ahead, or pulled
        // along behind a monster backing away.
        let dest = if pull { (at.0 + 2 * dx, at.1 + 2 * dy) } else { next };
        // Another monster, or a wagon being led, where this move is made from (once next to it)
        // or where it goes (once making it) holds it up. Those move on, mostly; going around
        // costs extra, so wait about that long first, then go around. (Waiting it out never
        // costs more than twice the best choice in hindsight.)
        let in_way = |(x, y): (i32, i32)| self.monster_at(x, y).is_some_and(|o| o != i) || self.world.wagons.iter().any(|w| w.leader.is_some() && (w.at == (x, y) || w.pumpkin == (x, y)));
        let near = (here.0 - stand.0).abs() + (here.1 - stand.1).abs() <= 1;
        let blocking = if near && in_way(stand) { Some(stand) } else if here == stand && in_way(dest) { Some(dest) } else { None };
        // One that's only waiting there, and doesn't need to be, steps aside.
        if let Some(o) = blocking.and_then(|(x, y)| self.monster_at(x, y)).filter(|&o| o != i) {
            if self.make_way(o, &[here, stand, dest, at], i) {
                return Going::Waiting("EXCUSE ME!".into());
            }
        }
        if blocking.is_some() {
            let m = &mut self.monsters[i];
            m.blocked = m.blocked.saturating_add(1);
            if m.blocked % REPLAN_AFTER == 0 {
                let waited = self.monsters[i].blocked as u32;
                let through = self.plan_carry(at, to, here, good, false).map_or(u32::MAX, |(_, c)| c);
                if let Some((plan, around)) = self.plan_carry(at, to, here, good, true).filter(|(p, _)| !p.is_empty()) {
                    if around <= through.saturating_add(waited) {
                        self.monsters[i].carry_plan = plan;
                    }
                }
            }
            return Going::Waiting("EXCUSE ME!".into());
        }
        // Something landed where this move is made from since planning: plan again.
        if self.world.item_at(stand.0, stand.1).is_some_and(|o| o != k) {
            self.monsters[i].carry_plan.clear();
            return Going::Waiting("SOMETHING'S IN THE WAY.".into());
        }
        if here != stand {
            return match self.go_toward(i, stand, if pull { d.opposite() } else { d }, sfx) {
                Going::Arrived => Going::Moving,
                other => other,
            };
        }
        // One move from done, but the goal's still busy with the last one: wait right here,
        // ready to put this one in the moment it's used up.
        if next == to {
            if let Some(o) = self.world.item_at(to.0, to.1).filter(|&o| o != k) {
                return Going::Waiting(format!("READY. WAITING FOR THE {} TO BE USED.", self.good_name(self.world.items[o].good())));
            }
        }
        let res = if pull { self.try_pull(i, d, sfx) } else { self.try_step(i, d, sfx) };
        let m = &mut self.monsters[i];
        match res {
            StepRes::Moved(_) => {
                m.carry_plan.remove(0);
                m.blocked = 0;
                let moved = self.world.item_at(next.0, next.1);
                if next == to {
                    let m = &mut self.monsters[i];
                    (m.carry, m.carry_plan) = (None, Vec::new());
                    // Put down: on to the step that takes it from here, if this loop has one.
                    if let Some(it) = moved {
                        self.pass_on(i, step, it);
                    }
                    return Going::Arrived;
                }
                self.monsters[i].carry = Some(next);
                // Set aside while it's being moved, so no other loop takes it.
                let claim = self.loop_step(i).map(|step| Claim { monster: self.monsters[i].id, step });
                if let Some(it) = moved {
                    self.world.items[it].data.claim = claim;
                }
                Going::Moving
            }
            StepRes::Blocked(msg) => {
                // Something changed since planning: plan again next time.
                m.carry_plan.clear();
                Going::Waiting(if msg.is_empty() { "SOMETHING'S IN THE WAY.".into() } else { msg })
            }
        }
    }
}
