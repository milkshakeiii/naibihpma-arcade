//! The loop scheduler: which goal of its loop a monster works on next.
//!
//! A loop is its goals, in the order you showed them: activate a thing (harvest a plant, work a
//! workshop, open or shut a wagon's tailgate), move a good from one tile to another, or lead a
//! wagon somewhere. They're linked by what they make and take: a harvest or a workshop's work
//! puts goods on a tile, a move takes a good from a tile, a workshop takes what's on its pads.
//! Whenever a monster is free it works on the first goal of its loop that is both wanted and
//! possible right now. A goal is wanted unless where it puts things is already full of what it
//! would put there: a tile holds one good, so a harvest isn't wanted while its last harvest
//! lies on its drop tile, nor a move while one of its good lies where it goes; a workshop's
//! output holds its mat and nine more waiting inside, so it keeps working until that's all
//! taken up with what it makes. The same for wagons: not wanted if it's already parked there,
//! or its tailgate already open (or shut). So nothing piles up past where it can go, a short
//! harvest is made up by harvesting again, work further up the chain carries on while a
//! workshop is busy, and a loop never waits on one of its own later goals: while one can't be
//! done, it does another. A loop that leads a wagon goes through its
//! goals strictly in order instead: it's a journey.
//!
//! A goal reserves the good it's going for, the tiles it's about to use and the wagon it leads,
//! from the moment it's picked until it's done, so two monsters never go for the same good or
//! pad. Monsters wanting the same plant, work spot or good take turns: whoever has waited
//! longest goes first. How monsters get where their goals take them is `motion.rs`'s business.

use crate::game::{Game, Goal, Sfx, Thing, UseRes};
use crate::motion::Going;
use serde::{Deserialize, Serialize};

/// Ticks a monster waits before saying what for.
pub(crate) const WAIT_BEFORE_SAYING: u16 = 12;
/// Ticks before trying a goal again that couldn't be done when it was picked.
const RETRY_TICKS: u16 = 15;

/// What a goal has reserved until it's done.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub enum Hold {
    /// The good it's going for, by its id.
    Item(u32),
    /// A tile it's about to use: where it delivers, the work spot it's walking to.
    Tile((i32, i32)),
    /// The wagon it's leading.
    Wagon(usize),
}

/// Why a goal can't be worked on right now, and the monster it waits on, if any.
pub(crate) struct Why {
    pub msg: String,
    pub on: Option<usize>,
}

impl Why {
    fn world(msg: impl Into<String>) -> Self {
        Why { msg: msg.into(), on: None }
    }
}

/// What picking a goal would reserve: goods by their index for now, tiles, wagons.
enum Want {
    Item(usize),
    Tile((i32, i32)),
    Wagon(usize),
}

/// How a goal went this tick.
pub(crate) enum Outcome {
    Done,
    Moving,
    Waiting(String),
    /// It can't be done after all (the world changed since it was picked): give it up.
    GiveUp(String),
}

impl Game {
    fn goals(&self, i: usize) -> &[Goal] {
        self.monsters[i].routine.as_ref().map_or(&[], |r| &r.goals)
    }

    /// A loop that leads a wagon goes through its goals in order.
    fn in_order(&self, i: usize) -> bool {
        self.goals(i).iter().any(|g| matches!(g, Goal::Lead { .. }))
    }

    /// What a goal puts down, and where.
    fn puts(&self, g: &Goal) -> Vec<(String, (i32, i32))> {
        match g {
            Goal::Move { good, to, .. } if !self.takes_into(*to, good) => vec![(good.clone(), *to)],
            Goal::Activate(Thing::Plant { at, facing }) => match self.world.plant_at(at.0, at.1) {
                Some(pl) => {
                    let (dx, dy) = facing.delta();
                    vec![(self.world.plants[pl].good.clone(), (at.0 + dx, at.1 + dy))]
                }
                None => Vec::new(),
            },
            Goal::Activate(Thing::Shop(s)) => {
                let shop = &self.world.shops[*s];
                shop.kind.recipes().iter().flat_map(|r| shop.outs.iter().map(move |&o| (r.to_string(), o))).collect()
            }
            _ => Vec::new(),
        }
    }

    /// What a goal takes, and from where (no good named: whatever lies on a workshop's pads).
    fn takes(&self, g: &Goal) -> Vec<(Option<String>, (i32, i32))> {
        match g {
            Goal::Move { good, from, .. } => vec![(Some(good.clone()), *from)],
            Goal::Activate(Thing::Shop(s)) => self.world.shops[*s].pads.iter().map(|&pad| (None, pad)).collect(),
            _ => Vec::new(),
        }
    }

    /// Which monster holds this good, tile or wagon, other than `i`.
    fn holder(&self, i: usize, hold: Hold) -> Option<usize> {
        (0..self.monsters.len()).find(|&k| k != i && self.monsters[k].holds.contains(&hold))
    }

    /// May monster `i` use item `k`: nobody else has reserved it, or `i` isn't looping (a
    /// monster you steer takes anything, and the loops make it again)?
    pub(crate) fn may_take(&self, i: usize, k: usize) -> bool {
        let id = self.world.items[k].id;
        !self.monsters[i].playing || id == 0 || self.holder(i, Hold::Item(id)).is_none()
    }

    /// Which goals of a loop are wanted right now: every one whose destination isn't already
    /// full of what it would put there. A tile holds one good: a harvest isn't wanted while one
    /// of its goods lies unused on its drop tile, nor a move while one of its good lies where it
    /// goes (one being worked there doesn't count: the next is brought up ready beside it). A
    /// workshop's output holds more (see `output_full`). Nor is leading a wagon already parked
    /// there wanted, nor a tailgate already as it should be. Deliveries always are.
    fn wanted(&self, goals: &[Goal]) -> Vec<bool> {
        let lying = |(g, t): &(String, (i32, i32))| self.world.item_at(t.0, t.1).is_some_and(|k| self.world.items[k].good() == g && !self.world.items[k].locked);
        goals
            .iter()
            .map(|g| match g {
                Goal::Lead { wagon, bed, pumpkin } => self.world.wagons.get(*wagon).is_some_and(|w| (w.at, w.pumpkin) != (*bed, *pumpkin)),
                Goal::Activate(Thing::Tailgate { wagon, open }) => self.world.wagons.get(*wagon).is_some_and(|w| w.open != *open),
                Goal::Activate(Thing::Shop(s)) => !self.output_full(*s),
                _ => !self.puts(g).iter().any(lying),
            })
            .collect()
    }

    /// Is workshop `s`'s output full of what it would make next? Its output is its mat and up to
    /// `OUTPUT_BUFFER` more waiting inside to be set out on it; full of something else (made from
    /// what was on the pads before), it isn't, and a loop waits for it to be cleared.
    fn output_full(&self, s: usize) -> bool {
        let shop = &self.world.shops[s];
        if shop.pending.len() < crate::world::OUTPUT_BUFFER {
            return false;
        }
        match self.best_recipe(s, shop.kind.recipes(), &|_| true) {
            Some((making, _)) => shop.pending.iter().any(|d| d.good == making),
            None => shop.kind.recipes().iter().any(|r| shop.pending.iter().any(|d| d.good == *r)),
        }
    }

    /// Can goal `g` be worked on right now, and what would it reserve? Cheap checks only:
    /// whether a good can actually be pushed there shows when the monster tries.
    fn possible(&self, i: usize, g: &Goal) -> Result<Vec<Want>, Why> {
        let name = |g: &str| self.good_name(g);
        let tile_free = |t: (i32, i32)| match self.holder(i, Hold::Tile(t)) {
            Some(m) => Err(Why { msg: crate::motion::EXCUSE_ME.into(), on: Some(m) }),
            None => Ok(()),
        };
        match g {
            Goal::Move { good, from, to } => {
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
            &Goal::Activate(Thing::Plant { at: p, facing }) => {
                tile_free(p)?;
                let Some(pl) = self.world.plant_at(p.0, p.1) else { return Err(Why::world("NOTHING TO HARVEST HERE.")) };
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
                Ok(vec![Want::Tile(p), Want::Tile(drop)])
            }
            &Goal::Activate(Thing::Shop(s)) => {
                let shop = &self.world.shops[s];
                tile_free(shop.spot)?;
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
                let mut wants = vec![Want::Tile(shop.spot)];
                wants.extend(inputs.into_iter().map(Want::Item));
                Ok(wants)
            }
            &Goal::Activate(Thing::Tailgate { wagon, .. }) => {
                let w = self.wagon_free(i, wagon)?;
                if w.leader.is_some() {
                    return Err(Why::world("WAITING FOR THE WAGON."));
                }
                let back = w.behind();
                tile_free(back)?;
                Ok(vec![Want::Tile(back)])
            }
            &Goal::Lead { wagon, .. } => {
                let w = self.wagon_free(i, wagon)?;
                if w.leader.is_some_and(|id| id != self.monsters[i].id) {
                    return Err(Why::world("SOMEONE ELSE IS LEADING THE WAGON."));
                }
                if self.led_wagon(i).is_some_and(|x| x != wagon) {
                    return Err(Why::world("I'M ALREADY LEADING A WAGON."));
                }
                Ok(vec![Want::Wagon(wagon)])
            }
        }
    }

    /// Wagon `w`, if no other monster has it in hand.
    fn wagon_free(&self, i: usize, w: usize) -> Result<&crate::world::Wagon, Why> {
        let Some(wagon) = self.world.wagons.get(w) else { return Err(Why::world("WAITING FOR THE WAGON.")) };
        match self.holder(i, Hold::Wagon(w)) {
            Some(m) => Err(Why { msg: "WAITING FOR THE WAGON.".into(), on: Some(m) }),
            None => Ok(wagon),
        }
    }

    /// What monsters take turns at for this goal: the plant or work spot it's activated at, or
    /// the tile a move takes its good from.
    fn turn_tile(&self, g: &Goal) -> Option<(i32, i32)> {
        match *g {
            Goal::Activate(Thing::Plant { at, .. }) => Some(at),
            Goal::Activate(Thing::Shop(s)) => Some(self.world.shops[s].spot),
            Goal::Move { from, .. } => Some(from),
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
            .filter(|&k| k != i && self.free_to_choose(k))
            .filter_map(|k| since(k).filter(|&s| s < mine || (s == mine && k < i)).map(|s| (s, k)))
            .filter(|&(_, k)| {
                let goals = self.goals(k);
                let want = self.wanted(goals);
                let resting = |n: usize| self.monsters[k].cooldown.iter().any(|&(c, _)| c == n);
                goals.iter().zip(want).enumerate().any(|(n, (g, w))| w && !resting(n) && self.turn_tile(g) == Some(tile) && self.possible(k, g).is_ok())
            })
            .min()
            .map(|(_, k)| k)
    }

    /// Could looping monster `k` pick a goal this very tick: between goals, and not busy working,
    /// mid-step, giving way or in a knot? (A place in line held by one working across the
    /// village doesn't make anyone wait for it.)
    fn free_to_choose(&self, k: usize) -> bool {
        let m = &self.monsters[k];
        m.playing && m.task.is_none() && m.idle() && m.giving_way.is_none() && !self.in_knot(k)
    }

    /// Picks the goal monster `i` works on next, reserving what it needs. If there's nothing it
    /// can do yet, says what it's waiting for.
    fn choose(&mut self, i: usize) -> Option<Why> {
        let goals = self.goals(i).to_vec();
        if goals.is_empty() {
            return Some(Why::world("NOTHING TO DO."));
        }
        let n = goals.len();
        let start = self.monsters[i].pc.min(n - 1);
        let (order, want): (Vec<usize>, Vec<bool>) = if self.in_order(i) { (vec![start], vec![true; n]) } else { ((0..n).map(|d| (start + d) % n).collect(), self.wanted(&goals)) };
        // If nothing can be done, the wait worth telling is at the start of the chain of
        // needs: a wanted goal that no wanted goal of the loop is making things for.
        let puts: Vec<_> = goals.iter().map(|g| self.puts(g)).collect();
        let fed = |u: usize| self.takes(&goals[u]).iter().any(|t| (0..n).any(|p| p != u && want[p] && puts[p].iter().any(|q| q.1 == t.1 && t.0.as_ref().is_none_or(|g| *g == q.0))));
        let mut why: Option<Why> = None;
        let mut root_why: Option<Why> = None;
        // What it's waiting its turn for this time round.
        let mut waiting_for: Vec<(i32, i32)> = Vec::new();
        for u in order {
            if !want[u] || self.monsters[i].cooldown.iter().any(|&(k, _)| k == u) {
                continue;
            }
            let tile = self.turn_tile(&goals[u]);
            waiting_for.extend(tile);
            let turn = self.possible(i, &goals[u]).and_then(|wants| match tile.and_then(|t| self.whose_turn(i, t)) {
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
                            Want::Wagon(w) => Hold::Wagon(w),
                        })
                        .collect();
                    let m = &mut self.monsters[i];
                    // Past the end of the loop's order and round to its start: another lap.
                    if u < m.pc {
                        m.lap();
                    }
                    (m.task, m.holds, m.pc) = (Some(u), holds, u);
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
        // it leaves its loop), even while that goal isn't wanted for a moment.
        let (m, now) = (&mut self.monsters[i], self.frame);
        for t in waiting_for {
            if !m.turns.iter().any(|&(x, _)| x == t) {
                m.turns.push((t, now));
            }
        }
        Some(root_why.or(why).unwrap_or_else(|| Why::world("NOTHING TO DO.")))
    }

    /// Ends monster `i`'s goal: done, or given up for a while.
    fn end_step(&mut self, i: usize, done: bool) {
        let (n, frame) = (self.goals(i).len(), self.frame);
        let m = &mut self.monsters[i];
        let Some(key) = m.task.take() else { return };
        m.holds.clear();
        (m.carry, m.carry_plan, m.shove) = (None, Vec::new(), None);
        if done {
            m.pc = key + 1;
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

    /// Leaving its loop, monster `i` gives up what it reserved.
    pub(crate) fn leave_loop(&mut self, i: usize) {
        let m = &mut self.monsters[i];
        (m.task, m.carry, m.shove) = (None, None, None);
        m.holds.clear();
        m.turns.clear();
        m.cooldown.clear();
        (m.carry_plan, m.path) = (Vec::new(), Vec::new());
        (m.waiting_on, m.giving_way, m.knot) = (None, None, None);
    }

    /// One tick of a looping monster.
    pub(crate) fn run_loop(&mut self, i: usize, sfx: &mut Vec<Sfx>) {
        // (With no loop at all, something else is moving it: a test's walker. In a knot, the
        // knot's moves are.)
        if !self.monsters[i].idle() || self.monsters[i].routine.is_none() || self.in_knot(i) {
            return;
        }
        let m = &mut self.monsters[i];
        m.waiting_on = None;
        for c in m.cooldown.iter_mut() {
            c.1 = c.1.saturating_sub(1);
        }
        m.cooldown.retain(|&(_, t)| t > 0);
        // Giving way: its goal waits until the one it gave way to has gone by.
        if m.giving_way.is_some() {
            match self.give_way_tick(i, sfx) {
                Going::Waiting(msg) => self.waiting(i, msg),
                _ => self.not_waiting(i),
            }
            return;
        }
        let m = &mut self.monsters[i];
        if m.task.is_none() {
            if let Some(why) = self.choose(i) {
                self.monsters[i].waiting_on = why.on;
                return self.waiting(i, why.msg);
            }
        }
        match self.do_step(i, sfx) {
            Outcome::Done => {
                self.end_step(i, true);
                self.not_waiting(i);
            }
            Outcome::Moving => self.not_waiting(i),
            Outcome::Waiting(msg) => self.waiting(i, msg),
            Outcome::GiveUp(msg) => {
                self.end_step(i, false);
                // It couldn't be done: to the back of the line, as if it had finished one, so a
                // goal that keeps failing can't keep putting it ahead of the others.
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
        m.wait = 0;
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

    /// One tick of the goal monster `i` picked.
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
        let Some(goal) = self.goals(i).get(key).cloned() else { return Outcome::GiveUp(String::new()) };
        match goal {
            Goal::Move { to, .. } => {
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
            // A journey: anything in the way is waited out.
            Goal::Lead { wagon, bed, pumpkin } => match self.lead_toward(i, wagon, bed, pumpkin, sfx) {
                Going::Arrived => Outcome::Done,
                Going::Moving => Outcome::Moving,
                Going::Waiting(msg) | Going::Stuck(msg) => Outcome::Waiting(msg),
            },
            Goal::Activate(t) => {
                if let Thing::Tailgate { wagon, open } = t {
                    if self.world.wagons.get(wagon).is_some_and(|w| w.open == open) {
                        return Outcome::Done;
                    }
                }
                let Some((spot, facing)) = self.activation_spot(t) else { return Outcome::GiveUp(String::new()) };
                match self.go_toward(i, spot, facing, sfx) {
                    Going::Arrived => {}
                    Going::Moving => return Outcome::Moving,
                    Going::Waiting(msg) => return Outcome::Waiting(msg),
                    Going::Stuck(msg) => return self.shove_or_give_up(i, spot, msg),
                }
                match self.try_use(i, sfx) {
                    UseRes::Done => Outcome::Done,
                    UseRes::Nothing => Outcome::GiveUp("NOTHING TO USE HERE.".into()),
                    // In a loop gone through in order, wait it out; otherwise something changed
                    // since the goal was picked, so pick again.
                    UseRes::Wait(msg) | UseRes::Full(msg) | UseRes::Missing(msg) if self.in_order(i) => Outcome::Waiting(msg),
                    UseRes::Wait(msg) | UseRes::Full(msg) | UseRes::Missing(msg) => Outcome::GiveUp(msg),
                }
            }
        }
    }

    /// No way to `to`: if strays are what's in the way, shove the first aside; otherwise give
    /// the goal up.
    fn shove_or_give_up(&mut self, i: usize, to: (i32, i32), msg: String) -> Outcome {
        match self.stray_on_walk(i, to) {
            Some((s, path)) if self.start_shove(i, s, &path) => Outcome::Moving,
            _ => Outcome::GiveUp(msg),
        }
    }
}
