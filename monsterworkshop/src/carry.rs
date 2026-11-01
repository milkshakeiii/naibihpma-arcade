//! Moving a good where a loop wants it (phase 2 of routines that scale). A recorded run of
//! pushes (and pulls) of one good is saved as its goal: "move the thread from the spinnery's
//! output mat to the loom house's pad". On replay the monster plans its own pushes from
//! wherever it stands, around whatever is in the way, pulling only when nothing else works. If
//! the goal is still taken (the loom busy with the last thread), the good is brought up next to
//! it and goes in the moment it's free. A stray good in the way there is shoved aside first.

use crate::game::{Game, Sfx, StepRes, PULL_TICKS, PUSH_TICKS, WALK_TICKS};
use crate::sched::Hold;
use crate::walk::{Going, REPLAN_AFTER};
use crate::world::{Dir, Tile, DIRS};
use std::cmp::Reverse;
use std::collections::{BinaryHeap, VecDeque};

/// How far a stray is shoved to get it out of the way, at most, in tiles.
const SHOVE_REACH: i32 = 4;

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
        self.plan_carry_with(from, to, me, good, around_monsters, false)
    }

    /// The same, but as if strays weren't there (for finding which to shove aside).
    fn plan_carry_with(&self, from: (i32, i32), to: (i32, i32), me: (i32, i32), good: &str, around_monsters: bool, through_strays: bool) -> Option<(Vec<CarryStep>, u32)> {
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
        for it in self.world.items.iter().filter(|it| (it.x, it.y) != from && !(through_strays && self.is_stray(it))) {
            taken[idx((it.x, it.y))] = true;
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

    /// One step of moving good `id` (reserved for it) to `to`: walk to where the next push or
    /// pull is made from, then make it. A stray (a good nobody is moving or using) lying where it
    /// goes is shoved aside first.
    pub(crate) fn move_good(&mut self, i: usize, id: u32, to: (i32, i32), sfx: &mut Vec<Sfx>) -> Going {
        // Used up or put away by someone else: this step is over.
        let Some(k) = self.world.find_item(id) else { return Going::Stuck(String::new()) };
        if (self.world.items[k].x, self.world.items[k].y) == to {
            return Going::Arrived;
        }
        if self.world.items[k].locked {
            return Going::Waiting("THAT'S BEING WORKED ON.".into());
        }
        if let Some(o) = self.world.item_at(to.0, to.1).filter(|&o| o != k && !self.world.items[o].locked) {
            let name = self.good_name(self.world.items[o].good());
            let oid = self.world.items[o].id;
            if let Some(m) = (oid != 0).then(|| (0..self.monsters.len()).find(|&m| m != i && self.monsters[m].holds.contains(&Hold::Item(oid)))).flatten() {
                self.monsters[i].waiting_on = Some(m);
                return Going::Waiting(format!("THE {name} IS IN THE WAY."));
            }
            return if self.start_shove(i, o, &[to]) { Going::Moving } else { Going::Stuck(format!("THE {name} IS IN THE WAY.")) };
        }
        self.push_good(i, k, to, true, sfx)
    }

    /// Is this a stray: a good nobody is moving or using?
    pub(crate) fn is_stray(&self, it: &crate::world::Item) -> bool {
        !it.locked && (it.id == 0 || !self.monsters.iter().any(|m| m.holds.contains(&Hold::Item(it.id))))
    }

    /// Starts shoving stray item `o` aside, off the tiles in `avoid`. False if there's nowhere
    /// it can be pushed.
    pub(crate) fn start_shove(&mut self, i: usize, o: usize, avoid: &[(i32, i32)]) -> bool {
        let Some(target) = self.shove_spot(i, o, avoid) else { return false };
        let sid = self.world.item_id(o);
        let m = &mut self.monsters[i];
        m.holds.extend([Hold::Item(sid), Hold::Tile(target)]);
        (m.shove, m.carry, m.carry_plan) = (Some((sid, target)), None, Vec::new());
        true
    }

    /// A walk from `from` to `to` as if strays weren't there (other goods and `avoid` are in
    /// the way), not counting `from`.
    fn walk_through_strays(&self, from: (i32, i32), to: (i32, i32), avoid: (i32, i32)) -> Option<Vec<(i32, i32)>> {
        let (w, h) = (self.world.w, self.world.h);
        let idx = |(x, y): (i32, i32)| (y * w + x) as usize;
        let open = |(x, y): (i32, i32)| {
            x >= 0 && y >= 0 && x < w && y < h && (x, y) != avoid && self.world.walkable(x, y) && !self.world.on_deck(x, y) && !self.world.cart_at(x, y) && self.world.item_at(x, y).is_none_or(|k| self.is_stray(&self.world.items[k]))
        };
        let mut prev = vec![usize::MAX; (w * h) as usize];
        prev[idx(from)] = idx(from);
        let mut queue = VecDeque::from([from]);
        while let Some(q) = queue.pop_front() {
            if q == to {
                let mut path = vec![to];
                let mut at = idx(to);
                while prev[at] != idx(from) {
                    at = prev[at];
                    path.push(((at as i32) % w, (at as i32) / w));
                }
                path.reverse();
                return Some(path);
            }
            for d in DIRS {
                let n = (q.0 + d.delta().0, q.1 + d.delta().1);
                if open(n) && prev[idx(n)] == usize::MAX {
                    prev[idx(n)] = idx(q);
                    queue.push_back(n);
                }
            }
        }
        None
    }

    /// The first stray on the way if monster `i` walks to `to`, and that way.
    pub(crate) fn stray_on_walk(&self, i: usize, to: (i32, i32)) -> Option<(usize, Vec<(i32, i32)>)> {
        let path = self.walk_through_strays((self.monsters[i].x, self.monsters[i].y), to, (-1, -1))?;
        let first = path.iter().find_map(|&(x, y)| self.world.item_at(x, y))?;
        Some((first, path))
    }

    /// When item `k` can't be got to `to`: the first stray on the way it could go if strays
    /// weren't there (where it's pushed, where pushes are made from, the walks between), and
    /// all of that way.
    fn stray_on_route(&self, i: usize, k: usize, to: (i32, i32)) -> Option<(usize, Vec<(i32, i32)>)> {
        let it = &self.world.items[k];
        let (at, good) = ((it.x, it.y), it.good().to_string());
        let me = (self.monsters[i].x, self.monsters[i].y);
        let (plan, _) = self.plan_carry_with(at, to, me, &good, false, true)?;
        let (mut b, mut pos, mut route) = (at, me, Vec::new());
        for (d, pull) in plan {
            let (dx, dy) = d.delta();
            let stand = if pull { (b.0 + dx, b.1 + dy) } else { (b.0 - dx, b.1 - dy) };
            route.extend(self.walk_through_strays(pos, stand, b).unwrap_or_default());
            let next = (b.0 + dx, b.1 + dy);
            route.extend([stand, next]);
            pos = if pull { (b.0 + 2 * dx, b.1 + 2 * dy) } else { b };
            b = next;
        }
        let first = route.iter().find_map(|&(x, y)| self.world.item_at(x, y).filter(|&s| s != k && self.is_stray(&self.world.items[s])))?;
        Some((first, route))
    }

    /// The shove is over: back to the good the step is for.
    pub(crate) fn end_shove(&mut self, i: usize) {
        let m = &mut self.monsters[i];
        if let Some((sid, target)) = m.shove.take() {
            m.holds.retain(|&h| h != Hold::Item(sid) && h != Hold::Tile(target));
        }
        (m.carry, m.carry_plan) = (None, Vec::new());
    }

    /// Where to shove stray item `o` so it's out of the way: the nearest free floor it can be
    /// pushed to (not a pad, mat, work spot, peg or doorway, nor any tile in `avoid`).
    fn shove_spot(&self, i: usize, o: usize, avoid: &[(i32, i32)]) -> Option<(i32, i32)> {
        let it = &self.world.items[o];
        let (from, good) = ((it.x, it.y), it.good().to_string());
        let me = (self.monsters[i].x, self.monsters[i].y);
        let held = |t: (i32, i32)| self.monsters.iter().any(|m| m.holds.contains(&Hold::Tile(t)));
        let mut spots: Vec<(i32, i32)> = (-SHOVE_REACH..=SHOVE_REACH)
            .flat_map(|dy| (-SHOVE_REACH..=SHOVE_REACH).map(move |dx| (from.0 + dx, from.1 + dy)))
            .filter(|&t| t != from && !avoid.contains(&t) && !held(t))
            .filter(|&(x, y)| matches!(self.world.tile(x, y), Tile::Floor | Tile::Grass | Tile::Path) && self.world.item_floor(x, y))
            .filter(|&(x, y)| self.world.item_at(x, y).is_none() && self.monster_at(x, y).is_none())
            .collect();
        spots.sort_by_key(|&(x, y)| ((x - from.0).abs() + (y - from.1).abs(), y, x));
        spots.into_iter().take(12).filter_map(|t| self.plan_carry(from, t, me, &good, false).map(|(_, cost)| (cost, t))).min().map(|(_, t)| t)
    }

    /// One push or pull, or a step of the walk to make it, of item `k` toward `to`. With
    /// `may_shove`, strays in the way get shoved aside.
    pub(crate) fn push_good(&mut self, i: usize, k: usize, to: (i32, i32), may_shove: bool, sfx: &mut Vec<Sfx>) -> Going {
        let at = (self.world.items[k].x, self.world.items[k].y);
        let good = self.world.items[k].good().to_string();
        // Moved by someone else since the plan was made: plan again from where it is now.
        if self.monsters[i].carry != Some(at) {
            let m = &mut self.monsters[i];
            (m.carry, m.carry_plan) = (Some(at), Vec::new());
        }
        let here = (self.monsters[i].x, self.monsters[i].y);
        if self.monsters[i].carry_plan.is_empty() {
            match self.plan_carry(at, to, here, &good, false) {
                Some((plan, _)) if plan.is_empty() => return Going::Arrived,
                Some((plan, _)) => self.monsters[i].carry_plan = plan,
                None => {
                    // Strays in the way: shove the first aside, off the way it would go.
                    if let Some((s, route)) = self.stray_on_route(i, k, to).filter(|_| may_shove) {
                        if self.start_shove(i, s, &route) {
                            return Going::Moving;
                        }
                    }
                    return Going::Stuck(format!("I CAN'T GET THE {} THERE.", self.good_name(&good)));
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
        if let Some(o) = blocking.and_then(|(x, y)| self.monster_at(x, y)).filter(|&o| o != i) {
            self.monsters[i].waiting_on = Some(o);
            // One that's only waiting there, and doesn't need to be, steps aside.
            if self.make_way(o, &[here, stand, dest, at], i) {
                return Going::Waiting("EXCUSE ME!".into());
            }
        }
        if blocking.is_some() {
            let m = &mut self.monsters[i];
            m.blocked = m.blocked.saturating_add(1);
            if m.blocked % REPLAN_AFTER == 0 {
                let waited = self.monsters[i].blocked as u32;
                let through = self.plan_carry(at, to, here, &good, false).map_or(u32::MAX, |(_, c)| c);
                if let Some((plan, around)) = self.plan_carry(at, to, here, &good, true).filter(|(p, _)| !p.is_empty()) {
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
                // No way there for strays: shove the first aside.
                Going::Stuck(msg) => match self.stray_on_walk(i, stand).filter(|_| may_shove) {
                    Some((s, path)) if self.start_shove(i, s, &path) => Going::Moving,
                    _ => Going::Stuck(msg),
                },
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
                if next == to {
                    (m.carry, m.carry_plan) = (None, Vec::new());
                    return Going::Arrived;
                }
                m.carry = Some(next);
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
