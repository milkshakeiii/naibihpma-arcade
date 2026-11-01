//! Claims: a loop sets aside the goods it makes for its own later steps, so other loops leave
//! them alone. Each claim names the step the good is for: a harvest's bolls go to the pushes
//! that follow it, one each, and a workshop's output to the step that hauls it away. A step
//! takes the good set aside for it, or an unclaimed one; if neither is there and an earlier
//! step of the same loop makes it, the loop goes back, makes it again, and carries on. A
//! producing step makes nothing while one of the kind already lies unclaimed where it would
//! put it, or while every step it feeds has its good. Leaving a loop gives its claims up.
//! Goods that come from outside the loop are waited for, as before.

use crate::econ::{Claim, ItemData};
use crate::game::{Act, Game};
use crate::world::Dir;

/// A tile a step takes from, and the good it takes there (any, for a workshop's pads).
pub(crate) type Take = (Option<String>, (i32, i32));

/// Where an unclaimed good lies or waits to come out.
enum Spot {
    Item(usize),
    Plant(usize, usize),
    Shop(usize, usize),
}

impl Game {
    /// The step of monster `i`'s loop it's doing now, if it's looping.
    pub(crate) fn loop_step(&self, i: usize) -> Option<usize> {
        self.monsters[i].playing.then_some(self.monsters[i].pc)
    }

    fn acts(&self, i: usize) -> &[Act] {
        self.monsters[i].routine.as_ref().map_or(&[], |r| &r.acts)
    }

    /// Where the Space at step `j` is pressed, facing which way: the "go to" just before it
    /// (for the first step, the loop's last).
    pub(crate) fn use_spot(&self, i: usize, j: usize) -> Option<((i32, i32), Dir)> {
        let acts = self.acts(i);
        let before = (j + acts.len().max(1) - 1) % acts.len().max(1);
        match (acts.get(j), acts.get(before).filter(|_| before != j)) {
            (Some(Act::Use), Some(&Act::Go { to, facing })) => Some((to, facing)),
            _ => None,
        }
    }

    /// What step `j` of monster `i`'s loop puts down, and where: a plant's good on the tile
    /// it faces, what a workshop makes on its output mats, a pushed good where it ends up.
    pub(crate) fn step_puts(&self, i: usize, j: usize) -> Vec<(String, (i32, i32))> {
        match self.acts(i).get(j) {
            Some(Act::Move { good, to, .. }) if !self.takes_into(*to, good) => vec![(good.clone(), *to)],
            Some(Act::Use) => {
                let Some((p, facing)) = self.use_spot(i, j) else { return Vec::new() };
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

    /// What step `k` takes, and from where: a move its good from where it starts, a
    /// workshop's Space whatever lies on its pads.
    pub(crate) fn step_takes(&self, i: usize, k: usize) -> Vec<Take> {
        match self.acts(i).get(k) {
            Some(Act::Move { good, from, .. }) => vec![(Some(good.clone()), *from)],
            Some(Act::Use) => match self.use_spot(i, k).and_then(|(p, _)| self.world.shop_at_spot(p.0, p.1)) {
                Some(s) => self.world.shops[s].pads.iter().map(|&p| (None, p)).collect(),
                None => Vec::new(),
            },
            _ => Vec::new(),
        }
    }

    fn feeds(put: &(String, (i32, i32)), take: &Take) -> bool {
        put.1 == take.1 && take.0.as_ref().is_none_or(|g| *g == put.0)
    }

    /// The steps that take the `good` producing step `j` puts down, in loop order after it, up
    /// to the next step that puts the same down again.
    pub(crate) fn fed_by(&self, i: usize, j: usize, good: &str) -> Vec<usize> {
        let n = self.acts(i).len();
        let puts: Vec<_> = self.step_puts(i, j).into_iter().filter(|p| p.0 == good).collect();
        let mut fed = Vec::new();
        for d in 1..n {
            let k = (j + d) % n;
            if self.step_takes(i, k).iter().any(|t| puts.iter().any(|p| Self::feeds(p, t))) {
                fed.push(k);
            }
            if self.step_puts(i, k).iter().any(|q| puts.contains(q)) {
                break;
            }
        }
        fed
    }

    /// The nearest step before `k` that puts down what `k` takes there.
    pub(crate) fn producer_of(&self, i: usize, k: usize, take: &Take) -> Option<usize> {
        let n = self.acts(i).len();
        (1..n).map(|d| (k + n - d) % n).find(|&j| self.step_puts(i, j).iter().any(|p| Self::feeds(p, take)))
    }

    /// Can monster `i` take this good: unclaimed, set aside by its own loop, or it isn't
    /// looping at all (a monster you steer takes anything, and the loops make it again).
    pub(crate) fn usable(&self, i: usize, d: &ItemData) -> bool {
        let m = &self.monsters[i];
        !m.playing || d.claim.is_none_or(|c| c.monster == m.id)
    }

    /// Every good there is: lying about, waiting in a plant or workshop, riding in a wagon.
    fn all_goods(&self) -> impl Iterator<Item = &ItemData> {
        let w = &self.world;
        w.items.iter().map(|it| &it.data).chain(w.plants.iter().flat_map(|p| &p.stock)).chain(w.shops.iter().flat_map(|s| &s.pending)).chain(w.wagons.iter().flat_map(|wg| wg.stock.iter().map(|(d, _)| d)))
    }

    /// Does step `k` of monster `i`'s loop have a good set aside for it?
    pub(crate) fn has_claim(&self, i: usize, k: usize) -> bool {
        let c = Claim { monster: self.monsters[i].id, step: k };
        self.all_goods().any(|d| d.claim == Some(c))
    }

    /// The steps producing step `j` feeds with `good` that have nothing set aside yet and are
    /// still to come this lap, the next first. (Making something again partway through a lap,
    /// the steps already passed get nothing: anything set aside for next lap would sit in
    /// everyone's way until then.)
    pub(crate) fn unfed(&self, i: usize, j: usize, good: &str) -> Vec<usize> {
        let (n, pc) = (self.acts(i).len().max(1), self.monsters[i].pc);
        let after_j = |k: usize| (k + n - j) % n;
        self.fed_by(i, j, good).into_iter().filter(|&k| after_j(k) >= after_j(pc) && !self.has_claim(i, k)).collect()
    }

    /// Step `k` of monster `i`'s loop has run: whatever was set aside for it, used or not,
    /// isn't any more.
    pub(crate) fn step_done(&mut self, i: usize, k: usize) {
        let c = Some(Claim { monster: self.monsters[i].id, step: k });
        let w = &mut self.world;
        let goods = w.items.iter_mut().map(|it| &mut it.data).chain(w.plants.iter_mut().flat_map(|p| p.stock.iter_mut())).chain(w.shops.iter_mut().flat_map(|s| s.pending.iter_mut())).chain(w.wagons.iter_mut().flat_map(|wg| wg.stock.iter_mut().map(|(d, _)| d)));
        for d in goods.filter(|d| d.claim == c) {
            d.claim = None;
        }
    }

    /// Claims for newly made `good` from producing step `j`, one per step it feeds that has
    /// none yet. Any more stay unclaimed.
    pub(crate) fn claims_for_made(&self, i: usize, j: usize, good: &str) -> Vec<Claim> {
        let id = self.monsters[i].id;
        self.unfed(i, j, good).into_iter().map(|step| Claim { monster: id, step }).collect()
    }

    /// Hands a good on to the next step of the loop that takes it where it now lies, or lets
    /// it go if none does. Called when step `k` has put it down.
    pub(crate) fn pass_on(&mut self, i: usize, k: usize, item: usize) {
        let good = self.world.items[item].good().to_string();
        let next = self.claims_for_made(i, k, &good).into_iter().next();
        self.world.items[item].data.claim = next;
    }

    /// Monster `i` has left its loop: everything it set aside is free again.
    pub(crate) fn release_claims(&mut self, i: usize) {
        let id = self.monsters[i].id;
        let w = &mut self.world;
        let goods = w.items.iter_mut().map(|it| &mut it.data).chain(w.plants.iter_mut().flat_map(|p| p.stock.iter_mut())).chain(w.shops.iter_mut().flat_map(|s| s.pending.iter_mut())).chain(w.wagons.iter_mut().flat_map(|wg| wg.stock.iter_mut().map(|(d, _)| d)));
        for d in goods.filter(|d| d.claim.is_some_and(|c| c.monster == id)) {
            d.claim = None;
        }
    }

    /// Is a good of this kind on its way to this tile that monster `i` could take: a plant or
    /// workshop setting one out there, or its own harvest or job there still going?
    pub(crate) fn coming_for(&self, i: usize, tile: (i32, i32), good: &str) -> bool {
        let w = &self.world;
        let plant = w.plants.iter().filter(|p| p.drop.is_some_and(|(d, _)| d == tile)).any(|p| p.stock.iter().any(|d| d.good == good && self.usable(i, d)) || (p.good == good && p.harvest.as_ref().is_some_and(|j| j.monster == i)));
        let shop = w.shops.iter().filter(|s| s.outs.contains(&tile)).any(|s| s.pending.iter().any(|d| d.good == good && self.usable(i, d)) || s.job.as_ref().is_some_and(|j| j.monster == i && j.recipe == good));
        plant || shop
    }

    /// Before the producing Space at step `j`: sets aside unclaimed goods of the kind it makes,
    /// lying on its output tile or about to come out there, for the steps it feeds that have
    /// nothing yet. True when there's no need to make more: one of the kind lies there
    /// unclaimed anyway, or every step it feeds has its good.
    pub(crate) fn already_made(&mut self, i: usize, j: usize) -> bool {
        let puts = self.step_puts(i, j);
        // What this loop wants from here: what its later steps take, or failing that, what
        // the step would make right now.
        let mut wanted: Vec<String> = puts.iter().map(|p| p.0.clone()).filter(|g| !self.fed_by(i, j, g).is_empty()).collect();
        wanted.dedup();
        if wanted.is_empty() {
            if let Some((p, _)) = self.use_spot(i, j) {
                if let Some(pl) = self.world.plant_at(p.0, p.1) {
                    wanted.push(self.world.plants[pl].good.clone());
                } else if let Some(s) = self.world.shop_at_spot(p.0, p.1) {
                    let ok = |it: usize| self.usable(i, &self.world.items[it].data);
                    wanted.extend(self.best_recipe(s, self.world.shops[s].kind.recipes(), &ok).map(|(r, _)| r));
                }
            }
        }
        let id = self.monsters[i].id;
        let mut skip = false;
        for good in wanted {
            let tiles: Vec<(i32, i32)> = puts.iter().filter(|p| p.0 == good).map(|p| p.1).collect();
            let w = &self.world;
            let lying = w.items.iter().enumerate().filter(|(_, it)| it.good() == good && it.data.claim.is_none() && !it.locked && tiles.contains(&(it.x, it.y))).map(|(k, _)| Spot::Item(k));
            let in_plants = w.plants.iter().enumerate().filter(|(_, p)| p.drop.is_some_and(|(d, _)| tiles.contains(&d))).flat_map(|(p, pl)| pl.stock.iter().enumerate().filter(|(_, d)| d.good == good && d.claim.is_none()).map(move |(k, _)| Spot::Plant(p, k)));
            let in_shops = w.shops.iter().enumerate().filter(|(_, s)| s.outs.iter().any(|o| tiles.contains(o))).flat_map(|(s, sh)| sh.pending.iter().enumerate().filter(|(_, d)| d.good == good && d.claim.is_none()).map(move |(k, _)| Spot::Shop(s, k)));
            // Lying there first, then what's about to come out.
            let mut spots: Vec<Spot> = lying.chain(in_plants).chain(in_shops).collect::<Vec<_>>().into_iter().rev().collect();
            for step in self.unfed(i, j, &good) {
                let Some(spot) = spots.pop() else { break };
                let d = match spot {
                    Spot::Item(k) => &mut self.world.items[k].data,
                    Spot::Plant(p, k) => &mut self.world.plants[p].stock[k],
                    Spot::Shop(s, k) => &mut self.world.shops[s].pending[k],
                };
                d.claim = Some(Claim { monster: id, step });
            }
            // One still lying there unclaimed: there's no need to make another. (Ones still
            // inside behind something else on the mat don't count: that's waited out.)
            let lying_left = spots.iter().any(|s| matches!(s, Spot::Item(_)));
            let fed = self.fed_by(i, j, &good);
            skip |= lying_left || (!fed.is_empty() && fed.iter().all(|&k| self.has_claim(i, k)));
        }
        skip
    }

    /// Goes back to step `j` to make again the `good` the current step needs, then comes back
    /// to it. False if the loop has already gone back as many times as it has steps: then it
    /// waits instead.
    pub(crate) fn redo(&mut self, i: usize, j: usize, good: &str) -> bool {
        let name = self.good_name(good);
        if !self.detour(i, j) {
            return false;
        }
        let m = &mut self.monsters[i];
        m.carry = None;
        self.say(i, &format!("GOING BACK FOR MORE {name}."), false);
        true
    }

    /// Does step `j` now, then comes back to the current step. False if the loop is already
    /// as many steps away from where it was as it has steps.
    pub(crate) fn detour(&mut self, i: usize, j: usize) -> bool {
        let n = self.acts(i).len();
        let m = &mut self.monsters[i];
        if m.detour.len() >= n {
            return false;
        }
        m.detour.push(m.pc);
        m.pc = j;
        (m.carry_plan, m.wait) = (Vec::new(), 0);
        m.wait_reason.clear();
        true
    }

    /// For workshop step `k` missing its inputs: the earlier step of the loop that brings one
    /// to a pad with nothing there this loop may use, and the good it brings.
    pub(crate) fn missing_input_step(&self, i: usize, k: usize) -> Option<(usize, String)> {
        let empty = |t: &&Take| self.world.item_at(t.1 .0, t.1 .1).is_none_or(|it| !self.usable(i, &self.world.items[it].data));
        self.step_takes(i, k).iter().filter(empty).find_map(|t| match self.producer_of(i, k, t).map(|j| (j, self.acts(i).get(j))) {
            Some((j, Some(Act::Move { good, .. }))) => Some((j, good.clone())),
            _ => None,
        })
    }

    /// Does any step producing step `j` feeds already have its good set aside?
    pub(crate) fn feeds_any_claimed(&self, i: usize, j: usize) -> bool {
        let mut goods: Vec<String> = self.step_puts(i, j).into_iter().map(|p| p.0).collect();
        goods.dedup();
        goods.iter().any(|g| self.fed_by(i, j, g).iter().any(|&k| self.has_claim(i, k)))
    }
}
