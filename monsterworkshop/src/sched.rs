//! The loop scheduler: which step of its loop a monster does next.
//!
//! A loop's steps are linked by what they make and take: a Space at a plant or workshop puts
//! goods down on a tile, a move takes a good from one tile to another, a workshop's Space takes
//! what's on its pads. Whenever a monster is free it does the first step of its loop, in the
//! order you recorded it, that is both wanted and possible right now. A step is wanted unless
//! its work is already done where it would be: what a Space makes already lies on its output
//! tile, or the good a move brings already lies where it goes. So nothing piles up, a short
//! harvest is made up by harvesting again, work further up the chain carries on while a
//! workshop is busy, and a loop never waits on one of its own later steps: while one step can't
//! be done, it does another that can.
//!
//! A step reserves the good it's going for and the tiles it's about to use from the moment it
//! picks the step until the step ends, so two monsters never go for the same good or pad.
//! Loops with steps played exactly as recorded (the ferry, leading a wagon) run in order.

use crate::game::{Act, Game, Sfx, StepRes, UseRes};
use crate::walk::Going;
use crate::world::Dir;
use serde::{Deserialize, Serialize};

/// Ticks a monster waits before saying what for.
pub(crate) const WAIT_BEFORE_SAYING: u16 = 12;
/// Ticks before trying a step again that couldn't be done when it was picked.
const RETRY_TICKS: u16 = 15;
/// Ticks of waiting on each other before one gives way.
const GIVE_WAY_AFTER: u16 = 8;
/// Ticks before a monster that gave way tries that step again.
const GAVE_WAY_TICKS: u16 = 45;

/// What a step has reserved until it ends.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub enum Hold {
    /// The good it's going for, by its id.
    Item(u32),
    /// A tile it's about to use: where it delivers, the work spot it's walking to.
    Tile((i32, i32)),
}

/// A step of a loop as the scheduler sees it: one recorded act, or a few that go together.
#[derive(Clone, Debug, PartialEq)]
pub(crate) enum Unit {
    /// A Space, with the walk to where it was pressed when that was recorded.
    Use { act: usize, spot: Option<((i32, i32), Dir)> },
    /// One good from a tile to another.
    Move { act: usize, good: String, from: (i32, i32), to: (i32, i32) },
    /// Steps played exactly as recorded (the ferry, leading a wagon), acts `first..=last`.
    Run { first: usize, last: usize },
    /// Plain walking, attached to nothing.
    Walk { act: usize, to: (i32, i32), facing: Dir },
}

impl Unit {
    /// Where it comes in the recorded order.
    pub(crate) fn key(&self) -> usize {
        match *self {
            Unit::Use { act, .. } | Unit::Move { act, .. } | Unit::Walk { act, .. } => act,
            Unit::Run { first, .. } => first,
        }
    }

    /// Its last act.
    fn last(&self) -> usize {
        match *self {
            Unit::Run { last, .. } => last,
            _ => self.key(),
        }
    }
}

/// Why a step can't be done right now, and the monster it waits on, if any.
pub(crate) struct Why {
    pub msg: String,
    pub on: Option<usize>,
}

impl Why {
    fn world(msg: impl Into<String>) -> Self {
        Why { msg: msg.into(), on: None }
    }
}

/// What picking a step would reserve: goods by their index for now, tiles.
enum Want {
    Item(usize),
    Tile((i32, i32)),
}

/// How a step went this tick.
pub(crate) enum Outcome {
    Done,
    Moving,
    Waiting(String),
    /// It can't be done after all (the world changed since it was picked): give it up.
    GiveUp(String),
}

impl Game {
    fn acts(&self, i: usize) -> &[Act] {
        self.monsters[i].routine.as_ref().map_or(&[], |r| &r.acts)
    }

    /// Loops with steps played exactly as recorded run in order.
    fn in_order(&self, i: usize) -> bool {
        self.acts(i).iter().any(|a| matches!(a, Act::Step(..) | Act::Pull(..)))
    }

    /// Where the Space at act `j` is pressed, facing which way: the "go to" just before it (for
    /// the first act, the loop's last).
    pub(crate) fn use_spot(&self, i: usize, j: usize) -> Option<((i32, i32), Dir)> {
        let acts = self.acts(i);
        let n = acts.len().max(1);
        let before = (j + n - 1) % n;
        match (acts.get(j), acts.get(before).filter(|_| before != j)) {
            (Some(Act::Use), Some(&Act::Go { to, facing })) => Some((to, facing)),
            _ => None,
        }
    }

    /// Monster `i`'s loop as steps.
    pub(crate) fn units(&self, i: usize) -> Vec<Unit> {
        let acts = self.acts(i);
        let n = acts.len();
        let literal = |a: &Act| matches!(a, Act::Step(..) | Act::Pull(..));
        let run_end = |mut k: usize| {
            while k + 1 < n && literal(&acts[k + 1]) {
                k += 1;
            }
            k
        };
        let mut out = Vec::new();
        let mut k = 0;
        while k < n {
            match &acts[k] {
                // A "go to" just before a Space is where it's pressed; just before steps played as
                // recorded, where they start.
                Act::Go { .. } if matches!(acts.get(k + 1), Some(Act::Use)) => {}
                Act::Go { .. } if k + 1 == n && n > 1 && matches!(acts[0], Act::Use) => {}
                Act::Go { .. } if acts.get(k + 1).is_some_and(literal) => {
                    let last = run_end(k + 1);
                    out.push(Unit::Run { first: k, last });
                    k = last;
                }
                &Act::Go { to, facing } => out.push(Unit::Walk { act: k, to, facing }),
                Act::Use => out.push(Unit::Use { act: k, spot: self.use_spot(i, k) }),
                Act::Move { good, from, to } => out.push(Unit::Move { act: k, good: good.clone(), from: *from, to: *to }),
                Act::Step(..) | Act::Pull(..) => {
                    let last = run_end(k);
                    out.push(Unit::Run { first: k, last });
                    k = last;
                }
            }
            k += 1;
        }
        out
    }

    /// What a step puts down, and where.
    fn puts(&self, u: &Unit) -> Vec<(String, (i32, i32))> {
        match u {
            Unit::Move { good, to, .. } if !self.takes_into(*to, good) => vec![(good.clone(), *to)],
            Unit::Use { spot: Some((p, facing)), .. } => {
                if let Some(pl) = self.world.plant_at(p.0, p.1) {
                    let (dx, dy) = facing.delta();
                    return vec![(self.world.plants[pl].good.clone(), (p.0 + dx, p.1 + dy))];
                }
                let Some(s) = self.world.shop_at_spot(p.0, p.1) else { return Vec::new() };
                let shop = &self.world.shops[s];
                shop.kind.recipes().iter().flat_map(|r| shop.outs.iter().map(move |&o| (r.to_string(), o))).collect()
            }
            _ => Vec::new(),
        }
    }

    /// What a step takes, and from where (no good named: whatever lies on a workshop's pads).
    fn takes(&self, u: &Unit) -> Vec<(Option<String>, (i32, i32))> {
        match u {
            Unit::Move { good, from, .. } => vec![(Some(good.clone()), *from)],
            Unit::Use { spot: Some((p, _)), .. } => match self.world.shop_at_spot(p.0, p.1) {
                Some(s) => self.world.shops[s].pads.iter().map(|&pad| (None, pad)).collect(),
                None => Vec::new(),
            },
            _ => Vec::new(),
        }
    }

    /// Which monster holds this good or tile, other than `i`.
    fn holder(&self, i: usize, hold: Hold) -> Option<usize> {
        (0..self.monsters.len()).find(|&k| k != i && self.monsters[k].holds.contains(&hold))
    }

    /// May monster `i` use item `k`: nobody else has reserved it, or `i` isn't looping (a
    /// monster you steer takes anything, and the loops make it again)?
    pub(crate) fn may_take(&self, i: usize, k: usize) -> bool {
        let id = self.world.items[k].id;
        !self.monsters[i].playing || id == 0 || self.holder(i, Hold::Item(id)).is_none()
    }

    /// Which steps of a loop are wanted right now: every one whose work isn't already done where
    /// it would be. A Space isn't wanted while something it makes lies unused on its output tile,
    /// nor a move while one of its good already lies where it goes (one being worked there
    /// doesn't count: the next is brought up ready beside it). Deliveries always are. Plain walks
    /// are only wanted in a loop that does nothing but walk.
    fn wanted(&self, units: &[Unit]) -> Vec<bool> {
        let lying = |(g, t): &(String, (i32, i32))| self.world.item_at(t.0, t.1).is_some_and(|k| self.world.items[k].good() == g && !self.world.items[k].locked);
        let only_walks = units.iter().all(|u| matches!(u, Unit::Walk { .. }));
        units
            .iter()
            .map(|u| match u {
                Unit::Walk { .. } => only_walks,
                _ => !self.puts(u).iter().any(lying),
            })
            .collect()
    }

    /// Can step `u` be done right now, and what would it reserve? Cheap checks only: whether a
    /// good can actually be pushed there shows when the monster tries.
    fn possible(&self, i: usize, u: &Unit) -> Result<Vec<Want>, Why> {
        let name = |g: &str| self.good_name(g);
        let tile_free = |t: (i32, i32)| match self.holder(i, Hold::Tile(t)) {
            Some(m) => Err(Why { msg: "EXCUSE ME!".into(), on: Some(m) }),
            None => Ok(()),
        };
        match u {
            Unit::Move { good, from, to, .. } => {
                let Some(k) = self.world.item_at(from.0, from.1).filter(|&k| self.world.items[k].good() == good) else {
                    return Err(Why::world(format!("WAITING FOR {}.", name(good))));
                };
                let it = &self.world.items[k];
                if it.locked {
                    return Err(Why::world("THAT'S BEING WORKED ON."));
                }
                if let Some(m) = (it.id != 0).then(|| self.holder(i, Hold::Item(it.id))).flatten() {
                    return Err(Why { msg: format!("WAITING FOR {}.", name(good)), on: Some(m) });
                }
                let mut wants = vec![Want::Item(k)];
                if !self.takes_into(*to, good) {
                    tile_free(*to)?;
                    // Another monster's good on the way there holds this up; a good being worked
                    // there is waited for right beside it; a stray gets shoved aside.
                    if let Some(o) = self.world.item_at(to.0, to.1).filter(|&o| o != k) {
                        let other = &self.world.items[o];
                        if let Some(m) = (other.id != 0).then(|| self.holder(i, Hold::Item(other.id))).flatten() {
                            return Err(Why { msg: format!("THE {} IS IN THE WAY.", name(other.good())), on: Some(m) });
                        }
                    }
                    wants.push(Want::Tile(*to));
                }
                Ok(wants)
            }
            Unit::Use { spot: Some((p, facing)), .. } => {
                tile_free(*p)?;
                if let Some(pl) = self.world.plant_at(p.0, p.1) {
                    let plant = &self.world.plants[pl];
                    let (dx, dy) = facing.delta();
                    let drop = (p.0 + dx, p.1 + dy);
                    if plant.harvest.is_some() {
                        return Err(Why::world("SOMEONE IS HARVESTING IT."));
                    }
                    if !plant.stock.is_empty() {
                        return Err(Why::world("IT'S STILL SETTING OUT ITS LAST HARVEST."));
                    }
                    if !plant.ripe() {
                        return Err(Why::world("STILL GROWING."));
                    }
                    tile_free(drop)?;
                    if let Some(o) = self.world.item_at(drop.0, drop.1) {
                        let on = self.holder(i, Hold::Item(self.world.items[o].id)).filter(|_| self.world.items[o].id != 0);
                        return Err(Why { msg: "NO ROOM TO DROP IT THERE.".into(), on });
                    }
                    return Ok(vec![Want::Tile(*p), Want::Tile(drop)]);
                }
                if let Some(s) = self.world.shop_at_spot(p.0, p.1) {
                    let shop = &self.world.shops[s];
                    if shop.job.is_some() {
                        return Err(Why::world(format!("THE {} IS BUSY.", shop.kind.label())));
                    }
                    if shop.pending.len() >= crate::world::OUTPUT_BUFFER {
                        return Err(Why::world(format!("THE {} IS FULL. CLEAR ITS OUTPUT MAT.", shop.kind.label())));
                    }
                    let ok = |it: usize| {
                        let item = &self.world.items[it];
                        !item.locked && (item.id == 0 || self.holder(i, Hold::Item(item.id)).is_none())
                    };
                    let Some((recipe, inputs)) = self.best_recipe(s, shop.kind.recipes(), &ok) else {
                        let first = self.data.good(shop.default_recipe(&self.data));
                        let want = first.input_groups().next().map_or_else(String::new, |(_, g)| self.describe_group(g));
                        return Err(Why::world(format!("PUT {want} ON THE PAD.")));
                    };
                    if let Err(missing) = self.missing_tool(s, &recipe) {
                        return Err(Why::world(format!("MISSING A TOOL: {missing}.")));
                    }
                    let mut wants = vec![Want::Tile(*p)];
                    wants.extend(inputs.into_iter().map(Want::Item));
                    return Ok(wants);
                }
                Ok(vec![Want::Tile(*p)])
            }
            _ => Ok(Vec::new()),
        }
    }

    /// What monsters take turns at for this step: the plant or work spot a Space is pressed at,
    /// or the tile a move takes its good from.
    fn turn_tile(u: &Unit) -> Option<(i32, i32)> {
        match u {
            Unit::Use { spot: Some((p, _)), .. } => Some(*p),
            Unit::Move { from, .. } => Some(*from),
            _ => None,
        }
    }

    /// Whose turn it is at `tile`, if not monster `i`'s: another looping monster that has
    /// been waiting for it longer and could take it right now. (Only one that could, so a
    /// turn never holds anyone up.)
    fn whose_turn(&self, i: usize, tile: (i32, i32)) -> Option<usize> {
        let since = |k: usize| self.monsters[k].turns.iter().find(|(t, _)| *t == tile).map(|&(_, s)| s);
        let mine = since(i).unwrap_or(self.frame);
        (0..self.monsters.len())
            .filter(|&k| k != i && self.monsters[k].playing && self.monsters[k].task.is_none())
            .filter_map(|k| since(k).filter(|&s| s < mine || (s == mine && k < i)).map(|s| (s, k)))
            .filter(|&(_, k)| {
                let units = self.units(k);
                let want = self.wanted(&units);
                units.iter().zip(want).any(|(u, w)| w && Self::turn_tile(u) == Some(tile) && self.possible(k, u).is_ok())
            })
            .min()
            .map(|(_, k)| k)
    }

    /// Picks the step monster `i` does next, reserving what it needs. If there's nothing it
    /// can do yet, says what it's waiting for. Monsters wanting the same plant, work spot or
    /// good take turns: whoever has waited longest for it goes first.
    fn choose(&mut self, i: usize) -> Option<Why> {
        let units = self.units(i);
        if units.is_empty() {
            return Some(Why::world("NOTHING TO DO."));
        }
        let n = units.len();
        let pc = self.monsters[i].pc;
        let start = units.iter().position(|u| u.key() >= pc).unwrap_or(0);
        let (order, want): (Vec<usize>, Vec<bool>) = if self.in_order(i) { (vec![start], vec![true; n]) } else { ((0..n).map(|d| (start + d) % n).collect(), self.wanted(&units)) };
        // If nothing can be done, the wait worth telling is at the start of the chain of
        // needs: a wanted step that no wanted step of the loop is making things for.
        let puts: Vec<_> = units.iter().map(|u| self.puts(u)).collect();
        let fed = |u: usize| self.takes(&units[u]).iter().any(|t| (0..n).any(|p| p != u && want[p] && puts[p].iter().any(|q| q.1 == t.1 && t.0.as_ref().is_none_or(|g| *g == q.0))));
        let mut why: Option<Why> = None;
        let mut root_why: Option<Why> = None;
        // What it's waiting its turn for this time round.
        let mut waiting_for: Vec<(i32, i32)> = Vec::new();
        for u in order {
            let key = units[u].key();
            if !want[u] || self.monsters[i].cooldown.iter().any(|&(k, _)| k == key) {
                continue;
            }
            let tile = Self::turn_tile(&units[u]);
            waiting_for.extend(tile);
            let turn = self.possible(i, &units[u]).and_then(|wants| match tile.and_then(|t| self.whose_turn(i, t)) {
                Some(k) => Err(Why { msg: format!("AFTER YOU, {}.", self.monsters[k].name), on: Some(k) }),
                None => Ok(wants),
            });
            match turn {
                Ok(wants) => {
                    let holds: Vec<Hold> = wants
                        .into_iter()
                        .map(|w| match w {
                            Want::Item(k) => Hold::Item(self.world.item_id(k)),
                            Want::Tile(t) => Hold::Tile(t),
                        })
                        .collect();
                    let m = &mut self.monsters[i];
                    // Past the end of the recorded order and round to its start: another lap.
                    if key < m.pc {
                        m.lap();
                    }
                    (m.task, m.holds, m.pc) = (Some(key), holds, key);
                    (m.carry_plan, m.path) = (Vec::new(), Vec::new());
                    // Its turn at this is used; its place in line for anything else stays.
                    m.turns.retain(|&(t, _)| Some(t) != tile);
                    return None;
                }
                Err(w) if root_why.is_none() && !fed(u) => root_why = Some(w),
                Err(w) => {
                    why.get_or_insert(w);
                }
            }
        }
        // Join the line for anything new; a place already held stays until its turn comes (or
        // it leaves its loop), even while that step isn't wanted for a moment.
        let (m, now) = (&mut self.monsters[i], self.frame);
        for t in waiting_for {
            if !m.turns.iter().any(|&(x, _)| x == t) {
                m.turns.push((t, now));
            }
        }
        Some(root_why.or(why).unwrap_or_else(|| Why::world("NOTHING TO DO.")))
    }

    /// Ends monster `i`'s step: done, or given up for a while.
    fn end_step(&mut self, i: usize, done: bool) {
        let units = self.units(i);
        let (n, frame) = (self.acts(i).len(), self.frame);
        let m = &mut self.monsters[i];
        let Some(key) = m.task.take() else { return };
        m.holds.clear();
        (m.carry, m.carry_plan, m.shove) = (None, Vec::new(), None);
        if done {
            let last = units.iter().find(|u| u.key() == key).map_or(key, |u| u.last());
            m.pc = last + 1;
            if m.pc >= n {
                m.pc = 0;
                m.lap();
            }
            m.steps += 1;
            m.since = frame;
            m.cooldown.retain(|&(k, _)| k != key);
        } else {
            m.cooldown.push((key, RETRY_TICKS));
        }
    }

    /// Monster `i` gives up its step and everything it reserved, and doesn't try that step again
    /// for `ticks`.
    pub(crate) fn give_up(&mut self, i: usize, ticks: u16) {
        if let Some(key) = self.monsters[i].task {
            self.end_step(i, false);
            if let Some(c) = self.monsters[i].cooldown.iter_mut().find(|(k, _)| *k == key) {
                c.1 = ticks;
            }
        }
    }

    /// Leaving its loop, monster `i` gives up what it reserved.
    pub(crate) fn leave_loop(&mut self, i: usize) {
        let m = &mut self.monsters[i];
        (m.task, m.carry, m.shove) = (None, None, None);
        m.holds.clear();
        m.turns.clear();
        m.cooldown.clear();
        (m.carry_plan, m.path) = (Vec::new(), Vec::new());
    }

    /// One tick of a looping monster.
    pub(crate) fn run_loop(&mut self, i: usize, sfx: &mut Vec<Sfx>) {
        // Making room so someone can get by: its loop waits until that's done.
        if !self.monsters[i].idle() || self.in_untangle(i) {
            return;
        }
        let m = &mut self.monsters[i];
        (m.waiting_on, m.held_up) = (None, false);
        for c in m.cooldown.iter_mut() {
            c.1 = c.1.saturating_sub(1);
        }
        m.cooldown.retain(|&(_, t)| t > 0);
        if m.task.is_none() {
            if let Some(why) = self.choose(i) {
                self.monsters[i].waiting_on = why.on;
                return self.waiting(i, why.msg);
            }
        }
        let outcome = self.do_step(i, sfx);
        match outcome {
            Outcome::Done => {
                self.end_step(i, true);
                self.not_waiting(i);
            }
            Outcome::Moving => self.not_waiting(i),
            Outcome::Waiting(msg) => self.waiting(i, msg),
            Outcome::GiveUp(msg) => {
                self.end_step(i, false);
                // It couldn't be done: to the back of the line, as if it had finished one, so a
                // step that keeps failing can't keep putting it ahead of the others.
                self.monsters[i].since = self.frame;
                self.waiting(i, msg);
            }
        }
    }

    fn not_waiting(&mut self, i: usize) {
        let m = &mut self.monsters[i];
        if m.wait >= WAIT_BEFORE_SAYING {
            m.bubble = None;
        }
        (m.wait, m.no_room) = (0, 0);
        m.wait_reason.clear();
    }

    fn waiting(&mut self, i: usize, msg: String) {
        let m = &mut self.monsters[i];
        m.wait = m.wait.saturating_add(1);
        if m.wait >= WAIT_BEFORE_SAYING {
            m.bubble = Some(crate::game::Bubble { text: msg.clone(), ttl: 3, warn: false, waiting: true });
        }
        m.wait_reason = msg;
    }

    /// Looping monsters waiting on each other in a circle: the last of them in line (see
    /// `outranks`) gives way to the one waiting on it, so whoever has waited longest gets
    /// through, and in time everyone does. If something it reserved is part of the circle, it
    /// gives up its step and all it reserved; either way, if it's in the way, it makes way. Then
    /// whoever is first in line of those still held up by a monster that can't just step aside
    /// looks for real moves that get it by (see `untangle_if_held`).
    pub(crate) fn give_way(&mut self) {
        let n = self.monsters.len();
        let stuck = |k: usize| {
            let m = &self.monsters[k];
            m.playing && m.idle() && m.wait >= GIVE_WAY_AFTER && !self.in_untangle(k)
        };
        let mut seen = vec![false; n];
        let mut circles = Vec::new();
        for a in 0..n {
            let mut chain: Vec<usize> = Vec::new();
            let mut x = a;
            while stuck(x) && !seen[x] {
                if let Some(p) = chain.iter().position(|&c| c == x) {
                    circles.push(chain[p..].to_vec());
                    break;
                }
                chain.push(x);
                match self.monsters[x].waiting_on {
                    Some(y) if y < n => x = y,
                    _ => break,
                }
            }
            for c in chain {
                seen[c] = true;
            }
        }
        for circle in circles {
            let Some(&y) = circle.iter().max_by_key(|&&k| self.rank(k)) else { continue };
            let p = circle.iter().position(|&k| k == y).unwrap_or(0);
            let w = circle[(p + circle.len() - 1) % circle.len()];
            // Only in each other's way, nobody need give anything up: that's a tangle.
            let reserved = circle.iter().any(|&k| !self.monsters[k].held_up);
            self.give_way_to(y, w, reserved);
        }
        self.untangle_if_held();
    }

    /// Monster `y` gives way to monster `w`, which is waiting on it: gives up its step if
    /// `give_up`, and makes way if it's in the way. Steps played as recorded are never given up
    /// partway: they couldn't be taken up again from anywhere else.
    fn give_way_to(&mut self, y: usize, w: usize, give_up: bool) {
        if give_up && !self.on_recorded_steps(y) {
            self.give_up(y, GAVE_WAY_TICKS);
        }
        let wm = &self.monsters[w];
        let keep: Vec<(i32, i32)> = std::iter::once((wm.x, wm.y)).chain(wm.path.iter().copied()).collect();
        let (yp, wp) = ((self.monsters[y].x, self.monsters[y].y), (wm.x, wm.y));
        let in_the_way = keep.contains(&yp) || (yp.0 - wp.0).abs() + (yp.1 - wp.1).abs() == 1;
        let moved = in_the_way && self.move_over(y, &keep, w, &mut vec![w]);
        if give_up || moved {
            self.say(y, "AFTER YOU.", false);
        }
    }

    /// One tick of the step monster `i` picked.
    fn do_step(&mut self, i: usize, sfx: &mut Vec<Sfx>) -> Outcome {
        let Some(key) = self.monsters[i].task else { return Outcome::Done };
        // Shoving a stray out of the way first.
        if let Some((sid, target)) = self.monsters[i].shove {
            match self.world.find_item(sid).filter(|&s| (self.world.items[s].x, self.world.items[s].y) != target) {
                Some(s) => match self.push_good(i, s, target, false, sfx) {
                    Going::Arrived => self.end_shove(i),
                    Going::Moving => return Outcome::Moving,
                    Going::Waiting(msg) => return Outcome::Waiting(msg),
                    Going::Stuck(msg) => {
                        self.end_shove(i);
                        return Outcome::GiveUp(msg);
                    }
                },
                None => self.end_shove(i),
            }
            return Outcome::Moving;
        }
        let Some(unit) = self.units(i).into_iter().find(|u| u.key() == key) else { return Outcome::GiveUp(String::new()) };
        match unit {
            Unit::Move { to, .. } => {
                let id = self.monsters[i].holds.iter().find_map(|h| match h {
                    Hold::Item(id) => Some(*id),
                    _ => None,
                });
                match id.map(|id| self.move_good(i, id, to, sfx)) {
                    Some(Going::Arrived) => Outcome::Done,
                    Some(Going::Moving) => Outcome::Moving,
                    Some(Going::Waiting(msg)) => Outcome::Waiting(msg),
                    Some(Going::Stuck(msg)) => Outcome::GiveUp(msg),
                    None => Outcome::GiveUp(String::new()),
                }
            }
            Unit::Use { spot, .. } => {
                if let Some((p, facing)) = spot {
                    match self.go_toward(i, p, facing, sfx) {
                        Going::Arrived => {}
                        Going::Moving => return Outcome::Moving,
                        Going::Waiting(msg) => return Outcome::Waiting(msg),
                        Going::Stuck(msg) => return self.shove_or_give_up(i, p, msg),
                    }
                }
                match self.try_use(i, sfx) {
                    UseRes::Done => Outcome::Done,
                    // At the very spot it was pressed, nothing to use means what was used has
                    // moved, and only wagons move: wait for it.
                    UseRes::Nothing => Outcome::Waiting("WAITING FOR THE WAGON.".into()),
                    // In a loop played in order, wait it out; otherwise something changed since
                    // the step was picked, so pick again.
                    UseRes::Wait(msg) | UseRes::Full(msg) | UseRes::Missing(msg) if self.in_order(i) => Outcome::Waiting(msg),
                    UseRes::Wait(msg) | UseRes::Full(msg) | UseRes::Missing(msg) => Outcome::GiveUp(msg),
                }
            }
            Unit::Walk { to, facing, .. } => match self.go_toward(i, to, facing, sfx) {
                Going::Arrived => Outcome::Done,
                Going::Moving => Outcome::Moving,
                Going::Waiting(msg) => Outcome::Waiting(msg),
                Going::Stuck(msg) => self.shove_or_give_up(i, to, msg),
            },
            Unit::Run { first, last } => {
                let pc = self.monsters[i].pc.clamp(first, last);
                self.monsters[i].pc = pc;
                match self.do_literal(i, pc, sfx) {
                    Outcome::Done if pc < last => {
                        self.monsters[i].pc = pc + 1;
                        Outcome::Moving
                    }
                    other => other,
                }
            }
        }
    }

    /// No way to `to`: if strays are what's in the way, shove the first aside; otherwise give
    /// the step up.
    fn shove_or_give_up(&mut self, i: usize, to: (i32, i32), msg: String) -> Outcome {
        match self.stray_on_walk(i, to) {
            Some((s, path)) if self.start_shove(i, s, &path) => Outcome::Moving,
            _ => Outcome::GiveUp(msg),
        }
    }

    /// The tiles monster `i`'s next step played as recorded needs clear of other monsters: where
    /// it steps (or backs away to), and where a good it pushes goes.
    pub(crate) fn recorded_way(&self, i: usize) -> Vec<(i32, i32)> {
        let m = &self.monsters[i];
        let ahead = |k: i32, d: Dir| (m.x + k * d.delta().0, m.y + k * d.delta().1);
        match self.acts(i).get(m.pc) {
            Some(&Act::Step(d, Some(_))) => vec![ahead(1, d), ahead(2, d)],
            Some(&Act::Step(d, None) | &Act::Pull(d, _)) => vec![ahead(1, d)],
            _ => Vec::new(),
        }
    }

    /// One act played exactly as recorded.
    fn do_literal(&mut self, i: usize, pc: usize, sfx: &mut Vec<Sfx>) -> Outcome {
        let Some(act) = self.acts(i).get(pc).cloned() else { return Outcome::Done };
        // Steps played as recorded can't go around anyone: one waiting in the way makes way.
        let way = self.recorded_way(i);
        if let Some(o) = way.iter().find_map(|&(x, y)| self.monster_at(x, y)).filter(|&o| o != i) {
            (self.monsters[i].waiting_on, self.monsters[i].held_up) = (Some(o), true);
            let keep: Vec<(i32, i32)> = way.iter().copied().chain([(self.monsters[i].x, self.monsters[i].y)]).collect();
            self.make_way(o, &keep, i);
            return Outcome::Waiting(self.excuse_me(i));
        }
        let stepped = |res: StepRes| match res {
            StepRes::Moved(_) => Outcome::Done,
            StepRes::Blocked(msg) => Outcome::Waiting(if msg.is_empty() { "WAITING...".into() } else { msg }),
        };
        match act {
            Act::Go { to, facing } => match self.go_toward(i, to, facing, sfx) {
                Going::Arrived => Outcome::Done,
                Going::Moving => Outcome::Moving,
                Going::Waiting(msg) | Going::Stuck(msg) => Outcome::Waiting(msg),
            },
            Act::Step(d, expect) => {
                let (dx, dy) = d.delta();
                let (tx, ty) = (self.monsters[i].x + dx, self.monsters[i].y + dy);
                let here = self.world.item_at(tx, ty).map(|k| self.world.items[k].good().to_string());
                match (expect, here) {
                    (Some(k), Some(h)) if k == h => stepped(self.try_step(i, d, sfx)),
                    (Some(k), None) => Outcome::Waiting(format!("WAITING FOR {}.", self.good_name(&k))),
                    (_, Some(_)) => Outcome::Waiting("SOMETHING'S IN MY WAY.".into()),
                    (None, None) => stepped(self.try_step(i, d, sfx)),
                }
            }
            Act::Pull(d, expect) => match self.pulled_item(i, d).map(|k| self.world.items[k].good().to_string()) {
                Some(h) if h == expect => stepped(self.try_pull(i, d, sfx)),
                None => Outcome::Waiting(format!("WAITING FOR {}.", self.good_name(&expect))),
                Some(_) => Outcome::Waiting("THAT'S NOT WHAT I PULL.".into()),
            },
            Act::Use => match self.try_use(i, sfx) {
                UseRes::Done => Outcome::Done,
                UseRes::Nothing => Outcome::Waiting("WAITING FOR THE WAGON.".into()),
                UseRes::Wait(msg) | UseRes::Full(msg) | UseRes::Missing(msg) => Outcome::Waiting(msg),
            },
            Act::Move { .. } => Outcome::Done,
        }
    }
}
