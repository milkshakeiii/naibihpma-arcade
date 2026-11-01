//! Wagons and their pumpkins. A monster leads the pumpkin (Space facing it) and the wagon rolls
//! along behind, the three of them in a line like a snake. The wagon is a dispenser: goods
//! pushed into it go into its bed, one kind of good at a time; parked with its tailgate open
//! (Space at its back), it sets them out behind it one at a time, like a chute. How fast it
//! goes depends on its load against the wagon's power, the leader's CHA and wagon driving.

use crate::econ::{self, ItemData};
use crate::game::{Ev, Game, Sfx, UseRes, PUSH_TICKS};
use crate::world::*;

/// The most a heavy load or a heavy good can slow a move, in extra ticks per tile.
const MAX_SLOWDOWN: u8 = 9;

/// `PUSH_TICKS` up to `cap`, then a tick slower for every half `cap` beyond it.
pub(crate) fn heavy_ticks(weight: f64, cap: f64) -> u8 {
    if weight <= cap || cap <= 0.0 {
        return PUSH_TICKS;
    }
    PUSH_TICKS + (((weight - cap) / cap * 2.0).ceil() as u8).min(MAX_SLOWDOWN)
}

fn dir_to(from: (i32, i32), to: (i32, i32)) -> Dir {
    match (to.0 - from.0, to.1 - from.1) {
        (1, _) => Dir::Right,
        (-1, _) => Dir::Left,
        (_, 1) => Dir::Down,
        _ => Dir::Up,
    }
}

impl Game {
    /// The wagon this monster is leading.
    pub(crate) fn led_wagon(&self, i: usize) -> Option<usize> {
        let id = self.monsters[i].id;
        self.world.wagons.iter().position(|w| w.leader == Some(id))
    }

    /// How much weight a monster's wagon pulls at full speed.
    pub(crate) fn pull_capacity(&self, i: usize, w: usize) -> f64 {
        let m = &self.monsters[i];
        self.world.wagons[w].power * m.stats.cha as f64 * (1.0 + m.skills.applied("wagon_driving"))
    }

    pub(crate) fn load_weight(&self, w: usize) -> f64 {
        self.world.wagons[w].stock.iter().map(|(d, _)| econ::weight(self.data.good(&d.good), d)).sum()
    }

    pub(crate) fn wagon_ticks(&self, i: usize, w: usize) -> u8 {
        heavy_ticks(self.load_weight(w), self.pull_capacity(i, w))
    }

    /// Ticks to push a good: anything heavier than the monster moves easily goes slower.
    pub(crate) fn push_ticks(&self, i: usize, item: &ItemData) -> u8 {
        let m = &self.monsters[i];
        let cap = 10.0 * m.stats.str as f64 * (1.0 + m.skills.applied("hauling"));
        heavy_ticks(econ::weight(self.data.good(&item.good), item), cap)
    }

    /// Practice at hauling or wagon driving: the move's own game time.
    pub(crate) fn practice(&mut self, i: usize, skill: &str, ticks: u8) {
        let m = &mut self.monsters[i];
        econ::practice(&mut m.skills, &m.stats, skill, &self.data, ticks as f64 * crate::work::GAME_SECONDS_PER_TICK);
    }

    /// Space facing a wagon's pumpkin takes its lead; facing a wagon from behind, opens or shuts
    /// its tailgate. `None` if there's no wagon or pumpkin in front.
    pub(crate) fn use_wagon_ahead(&mut self, i: usize, sfx: &mut Vec<Sfx>) -> Option<UseRes> {
        let m = &self.monsters[i];
        let (dx, dy) = m.facing.delta();
        let (here, ahead) = ((m.x, m.y), (m.x + dx, m.y + dy));
        if let Some(w) = self.world.pumpkin_at(ahead.0, ahead.1) {
            match self.led_wagon(i) {
                Some(mine) if mine == w => {
                    self.let_go(w, sfx);
                    return Some(UseRes::Done);
                }
                Some(_) => return Some(UseRes::Wait("I'M ALREADY LEADING A WAGON.".into())),
                None => {}
            }
            if self.world.wagons[w].leader.is_some() {
                return Some(UseRes::Wait("SOMEONE ELSE IS LEADING IT.".into()));
            }
            let id = self.monsters[i].id;
            let wagon = &mut self.world.wagons[w];
            (wagon.leader, wagon.last_leader) = (Some(id), Some(id));
            sfx.push(Sfx::RecOn);
            return Some(UseRes::Done);
        }
        if let Some(w) = self.world.wagon_at(ahead.0, ahead.1) {
            let wagon = &mut self.world.wagons[w];
            if wagon.behind() != here {
                return Some(UseRes::Wait("ITS TAILGATE IS AT THE BACK.".into()));
            }
            wagon.open = !wagon.open;
            wagon.timer = 0;
            sfx.push(Sfx::Switch);
            return Some(UseRes::Done);
        }
        None
    }

    /// Space with nothing usable in front turns toward a neighbor that is: a pumpkin, a wagon's
    /// tailgate, or an applicant. Standing on a work spot or a plant, Space uses that instead.
    pub(crate) fn face_something_usable(&mut self, i: usize) {
        let m = &self.monsters[i];
        let here = (m.x, m.y);
        if self.world.shop_at_spot(here.0, here.1).is_some() || self.world.plant_at(here.0, here.1).is_some() {
            return;
        }
        let usable = |(x, y): (i32, i32)| {
            self.world.pumpkin_at(x, y).is_some() || self.world.wagon_at(x, y).is_some_and(|w| self.world.wagons[w].behind() == here) || self.applicant_at(x, y)
        };
        let ahead = |d: Dir| (here.0 + d.delta().0, here.1 + d.delta().1);
        if usable(ahead(m.facing)) {
            return;
        }
        if let Some(d) = DIRS.iter().copied().find(|&d| usable(ahead(d))) {
            let m = &mut self.monsters[i];
            m.facing = d;
            if matches!(d, Dir::Left | Dir::Right) {
                m.flip = d == Dir::Left;
            }
        }
    }

    pub(crate) fn let_go(&mut self, w: usize, sfx: &mut Vec<Sfx>) {
        self.world.wagons[w].leader = None;
        sfx.push(Sfx::RecSave);
    }

    /// After its leader steps away from `from`: the pumpkin takes that tile, the wagon the
    /// pumpkin's, both sliding in step with the leader.
    pub(crate) fn follow(&mut self, w: usize, from: (i32, i32), ticks: u8) {
        let wagon = &mut self.world.wagons[w];
        let (pumpkin, bed) = (wagon.pumpkin, wagon.at);
        wagon.pumpkin_slide = Some(Slide { dir: dir_to(pumpkin, from), left: ticks, total: ticks });
        wagon.slide = Some(Slide { dir: dir_to(bed, pumpkin), left: ticks, total: ticks });
        (wagon.pumpkin, wagon.at) = (from, pumpkin);
    }

    /// A good pushed into a wagon's bed, if it fits: the same kind of good as already there,
    /// and room for it.
    pub(crate) fn load_wagon(&mut self, w: usize, it: usize, hauler: u32) -> Result<(), String> {
        let good = self.world.items[it].good().to_string();
        let wagon = &self.world.wagons[w];
        if let Some(have) = wagon.good().filter(|g| *g != good) {
            return Err(format!("THE WAGON IS CARRYING {}.", self.good_name(have)));
        }
        let used: f64 = wagon.stock.iter().map(|(d, _)| self.data.good(&d.good).storage_volume).sum();
        if used + self.data.good(&good).storage_volume > wagon.room + 1e-9 {
            return Err("THE WAGON IS FULL.".into());
        }
        let mut item = self.world.items.remove(it);
        item.data.hauled_by(hauler);
        let wagon = &mut self.world.wagons[w];
        wagon.stock.push((item.data, area_name(wagon.at.0).to_string()));
        wagon.bounce = 4;
        Ok(())
    }

    /// Wagons slide along, and a parked wagon with its tailgate open sets its goods out behind
    /// it, one at a time, straight into a bin there if the bin takes them.
    pub(crate) fn tick_wagons(&mut self, sfx: &mut Vec<Sfx>) {
        for w in 0..self.world.wagons.len() {
            let wagon = &mut self.world.wagons[w];
            for s in [&mut wagon.slide, &mut wagon.pumpkin_slide] {
                if let Some(sl) = s.as_mut() {
                    sl.left -= 1;
                    if sl.left == 0 {
                        *s = None;
                    }
                }
            }
            wagon.bounce = wagon.bounce.saturating_sub(1);
            if wagon.leader.is_some() || !wagon.open || wagon.stock.is_empty() || wagon.slide.is_some() {
                wagon.timer = 0;
                continue;
            }
            wagon.timer += 1;
            if wagon.timer < DISPENSE_TICKS {
                continue;
            }
            wagon.timer = 0;
            let (bx, by) = wagon.behind();
            let (good, from) = (wagon.stock[0].0.good.clone(), wagon.stock[0].1.clone());
            let bin = self.world.bin_at(bx, by).filter(|&b| self.world.bins[b].takes(&good));
            let free = self.world.item_floor(bx, by) && self.world.item_at(bx, by).is_none() && self.monster_at(bx, by).is_none() && !self.world.cart_at(bx, by);
            if bin.is_none() && !free {
                continue;
            }
            let wagon = &mut self.world.wagons[w];
            let (mut data, _) = wagon.stock.remove(0);
            // Brought here from somewhere else: the wagon's leader earns the journey's credit.
            if from != area_name(wagon.at.0) {
                data.carted = wagon.last_leader;
            }
            let back = dir_to(wagon.pumpkin, wagon.at);
            self.events.push(Ev::Unloaded);
            if let Some(b) = bin {
                if self.world.bins[b].scores {
                    econ::pay_haul(&mut data);
                    let (value, shares) = (data.value, data.shares.clone());
                    self.score(value, &shares, bx, by);
                }
                data.claim = None;
                self.world.bins[b].stock.push(data);
                self.world.bins[b].bounce = 4;
                self.world.burst(bx, by, PKind::Spark, 4);
                sfx.push(Sfx::Deposit);
            } else {
                let slide = Some(Slide { dir: back, left: PUSH_TICKS, total: PUSH_TICKS });
                self.world.items.push(Item { slide, hop: 6, ..Item::new(bx, by, data) });
                sfx.push(Sfx::Pop);
            }
        }
    }
}
