//! Scripted Elders, for balance checks (document 05's build step 1). They use only the commands
//! the player has. Bots show whether a strategy wins or stalls; they never show whether it's fun.

use crate::data::*;
use crate::terrain::{dist, places, MH, MW};
use crate::world::{RKind, RTarget, Task, World};
#[allow(unused_imports)]
use crate::data::MINUTE;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Style {
    /// Builds in document 02's order and posts rewards on the nearest lairs.
    Steady,
    /// Builds the same, but never posts a reward.
    NoRewards,
    /// Does nothing at all.
    Idle,
}

pub const ORDER: [BKind; 11] = [
    BKind::MorelCave,
    BKind::IroncapCave,
    BKind::EmberLog,
    BKind::FairyRing,
    BKind::StarPond,
    BKind::NewtHouse,
    BKind::ShellForge,
    BKind::Cauldron,
    BKind::DaotsGrotto,
    BKind::SpawningPool,
    BKind::OlmsCave,
];

pub struct Bot {
    pub style: Style,
    next: usize,
    last_raise: u32,
}

impl Bot {
    pub fn new(style: Style) -> Bot {
        Bot { style, next: 0, last_raise: 0 }
    }

    /// Find a place for a building near the Hold.
    pub fn site_for(w: &World, k: BKind, near: (i32, i32)) -> Option<(i32, i32)> {
        let info = bld(k);
        let mut best: Option<(i32, (i32, i32))> = None;
        for y in 0..MH {
            for x in 0..MW {
                let d = dist((x, y), near);
                if d > 20 || best.is_some_and(|b| d >= b.0) {
                    continue;
                }
                if !w.map.can_place(x, y, info.fp.0, info.fp.1, info.place) {
                    continue;
                }
                // Leave a square free round it, so paths stay open.
                let mut roomy = true;
                for yy in y - 1..=y + info.fp.1 {
                    for xx in x - 1..=x + info.fp.0 {
                        if w.bld_at((xx, yy)).is_some() {
                            roomy = false;
                        }
                    }
                }
                if roomy && w.hold_blds().any(|b| b.dist_to((x, y)) <= 18) {
                    best = Some((d, (x, y)));
                }
            }
        }
        best.map(|b| b.1)
    }

    pub fn step(&mut self, w: &mut World) {
        if self.style == Style::Idle || w.t % 150 != 0 {
            return;
        }
        let hall = match w.hall() {
            Some(h) => h,
            None => return,
        };
        let hc = w.b(hall).centre();
        // Build in order, one site at a time, and rebuild whatever's been destroyed.
        let pending = w.bs.iter().any(|b| b.team == Team::Hold && !b.dead && (b.site || b.hp < b.max_hp && b.grown == 0));
        if !pending {
            let start = [BKind::Burrow, BKind::TwigLoft, BKind::ChorusLog, BKind::ToadHall];
            let mut want: Vec<BKind> = start.to_vec();
            want.extend(ORDER.iter().take(self.next + 1));
            for k in want {
                if w.bs.iter().any(|b| b.kind == k && !b.dead) {
                    if ORDER.get(self.next) == Some(&k) {
                        self.next += 1;
                    }
                    continue;
                }
                let Some(src) = bld(k).src.and_then(|s| w.find_bld(s)) else { continue };
                if w.build_blocker(src, k).is_some() {
                    continue;
                }
                if let Some(p) = Bot::site_for(w, k, hc) {
                    if w.order_build(src, k, p).is_ok() {
                        if ORDER.get(self.next) == Some(&k) {
                            self.next += 1;
                        }
                        break;
                    }
                }
            }
        }
        if self.style == Style::NoRewards {
            return;
        }
        // Rewards: the raiders first, then the nearest lairs it has seen, a few at a time.
        let open = w.flags.iter().filter(|f| !f.done).count();
        let raider = w.cs.iter().filter(|c| c.alive() && c.team == Team::Gorf && c.wave && w.hold_blds().any(|b| b.dist_to(c.pos) <= 12)).map(|c| c.id).next();
        if let Some(r) = raider {
            if open < MAX_REWARDS && !w.flags.iter().any(|f| !f.done && f.target == RTarget::Creature(r)) {
                let _ = w.post_reward(RKind::Attack, RTarget::Creature(r), 30);
            }
        }
        let dew = w.b(hall).has(Item::Dew);
        let mut lairs: Vec<(i32, u32)> = w
            .bs
            .iter()
            .filter(|b| !b.dead && b.team == Team::Gorf && b.kind != BKind::GorfStone || !b.dead && b.kind == BKind::GorfStone)
            .filter(|b| b.team != Team::Hold && b.kind != BKind::LeechPool)
            .filter(|b| w.map.explored[crate::terrain::idx(b.centre().0, b.centre().1)] || b.kind == BKind::LoneNest)
            .map(|b| (b.dist_to(hc) + if b.kind == BKind::GorfHollow { 1000 } else { 0 }, b.id))
            .collect();
        lairs.sort();
        // Nothing left in sight: send someone to look further east.
        let seen_lairs = lairs.iter().filter(|(_, b)| w.b(*b).kind != BKind::GorfHollow).count();
        let heroes_now = w.cs.iter().filter(|c| c.alive() && c.hero()).count();
        if seen_lairs < 2 && heroes_now >= 5 && w.t > 12 * MINUTE && !w.flags.iter().any(|f| !f.done && f.kind == RKind::Explore) && dew >= 40 {
            let mut east = 0;
            for y in 0..MH {
                for x in 0..MW {
                    if w.map.explored[crate::terrain::idx(x, y)] {
                        east = east.max(x);
                    }
                }
            }
            let p = ((east + 8).min(MW - 2), 21);
            let _ = w.post_reward(RKind::Explore, RTarget::Square(p), 30);
        }
        // The road to Gorf: once the Hold is strong, rewards on the turtles, then the Hollow.
        let heroes = w.cs.iter().filter(|c| c.alive() && c.hero()).count();
        if heroes >= 8 && w.t > 30 * MINUTE && dew >= 80 {
            if let Some(t) = w.cs.iter().filter(|c| c.alive() && c.kind == Kind::Turtle).map(|c| c.id).next() {
                if !w.flags.iter().any(|f| !f.done && f.target == RTarget::Creature(t)) && w.flags.iter().filter(|f| !f.done).count() < MAX_REWARDS {
                    let _ = w.post_reward(RKind::Attack, RTarget::Creature(t), 80);
                }
            }
        }
        let posted = w.flags.iter().filter(|f| !f.done && f.kind == RKind::Attack && matches!(f.target, RTarget::Bldg(_))).count();
        let in_rewards: i32 = w.flags.iter().filter(|f| !f.done).map(|f| f.dew).sum();
        if posted < 2 && dew >= 60 && in_rewards * 2 < dew {
            for (_, b) in lairs.iter().take(3) {
                if w.flags.iter().any(|f| !f.done && f.target == RTarget::Bldg(*b)) {
                    continue;
                }
                let is_hollow = w.b(*b).kind == BKind::GorfHollow;
                let turtles = w.cs.iter().filter(|c| c.alive() && c.kind == Kind::Turtle).count();
                if is_hollow && (heroes < 8 || w.t < 30 * MINUTE) {
                    continue;
                }
                let _ = turtles;
                let dew = if is_hollow { 150 } else { 30 };
                let _ = w.post_reward(RKind::Attack, RTarget::Bldg(*b), dew);
                break;
            }
        }
        // Nobody's taking it: raise it, now and then.
        if w.t >= self.last_raise + 900 {
            self.last_raise = w.t;
            let ids: Vec<u32> = w
                .flags
                .iter()
                .filter(|f| !f.done && f.takers.is_empty() && f.dew < 80 && w.candidate_summary(f.id).contains("WANT MORE"))
                .map(|f| f.id)
                .collect();
            for f in ids {
                let _ = w.change_reward(f, REWARD_STEP * 2);
            }
        }
        // Fetch husks home once the Star Pond stands.
        if w.has_done(BKind::StarPond) {
            let husks: Vec<u32> = w.husks.iter().filter(|h| !h.gone && !h.home && h.was.level >= 2).map(|h| h.id).collect();
            for h in husks {
                if w.flags.iter().filter(|f| !f.done).count() < MAX_REWARDS && !w.flags.iter().any(|f| !f.done && f.target == RTarget::Husk(h)) {
                    let _ = w.post_reward(RKind::Fetch, RTarget::Husk(h), 20);
                }
            }
        }
        let _ = (places::HALL, Task::Sleep);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The balance gate: a steady Elder keeps the Hall standing for the first half hour on most
    /// seeds. (It shows the opening isn't a death spiral; it can't show the game is fun.)
    #[test]
    fn a_steady_elder_survives_the_first_half_hour() {
        let mut alive = 0;
        for seed in [1, 2, 3, 5, 6] {
            let mut w = World::new(seed);
            let mut bot = Bot::new(Style::Steady);
            while w.over.is_none() && w.t < 30 * MINUTE {
                bot.step(&mut w);
                w.tick();
                w.events.clear();
            }
            if w.over.is_none() || w.over == Some(true) {
                alive += 1;
            }
        }
        assert!(alive >= 4, "only {alive} of 5 seeds kept the Hall for 30 minutes");
    }

    /// With nobody posting rewards, nothing ever crashes over a long game.
    #[test]
    fn a_long_game_with_no_rewards_runs_clean() {
        let mut w = World::new(9);
        let mut bot = Bot::new(Style::NoRewards);
        while w.over.is_none() && w.t < 40 * MINUTE {
            bot.step(&mut w);
            w.tick();
            w.events.clear();
        }
    }
}
