//! The arch-AI: one planner for a whole side. It plays the fight forward from where things
//! stand, once for each change it could make to how its designs fly, and keeps the plan that
//! came out best. It thinks the whole plan through before the first move, and one design again
//! every couple of seconds after. Nothing here knows what a part does: it only watches what
//! happens.

use super::*;

/// How often a side's arch-AI thinks again, after its first plan.
pub const ARCH_EVERY: u32 = 15 * 2;
/// How far ahead it plays each plan.
pub const ARCH_AHEAD: u32 = 15 * 8;

impl Arena {
    /// The arch-AI's turn, for each side that has one. Blue plans first and red answers what
    /// blue just chose. A fight being played forward inside a plan does no planning of its own:
    /// there, each side flies on with the plan it has.
    pub(super) fn arch_think(&mut self) {
        if self.rehearsal || (self.tick - 1) % ARCH_EVERY != 0 {
            return;
        }
        for team in 0..2 {
            if self.brain[team].ways == Ways::Plan {
                self.plan[team] = self.arch_plan(team);
            }
        }
    }

    /// This side's designs that have a choice of how to fly: still flying, with guns and engines.
    fn arch_slots(&self, team: usize) -> Vec<usize> {
        let mut slots: Vec<usize> = self.ships.iter().filter(|s| s.alive && s.team == team && s.warp == 0 && s.role == Role::Ship && s.grow.is_none() && s.st.armed() && s.st.flight != Flight::Still).map(|s| s.slot).collect();
        slots.sort();
        slots.dedup();
        slots
    }

    /// The side's plan after thinking: how each of its designs flies, by slot (None: as its card
    /// says). One design at a time, it plays the fight forward with that design flying each way
    /// and the rest as planned, and keeps a way only if it comes out better than the plan in
    /// hand. The first time, every design gets its turn; after that one design a time, in turn.
    fn arch_plan(&mut self, team: usize) -> Vec<Option<Style>> {
        let slots = self.arch_slots(team);
        let mut plan = self.plan[team].clone();
        let Some(&top) = slots.last() else { return plan };
        if plan.len() <= top {
            plan.resize(top + 1, None);
        }
        let turn: Vec<usize> = if self.tick == 1 {
            slots
        } else {
            self.arch_turn[team] += 1;
            vec![slots[self.arch_turn[team] % slots.len()]]
        };
        let mut best = self.rehearse(team, &plan);
        for slot in turn {
            for style in STYLES {
                if plan[slot] == Some(style) {
                    continue;
                }
                let mut trial = plan.clone();
                trial[slot] = Some(style);
                let score = self.rehearse(team, &trial);
                if score > best {
                    (best, plan) = (score, trial);
                }
            }
        }
        plan
    }

    /// The fight played forward `ARCH_AHEAD` ticks with this side flying `plan` and the other
    /// flying on as it is: the pixels the enemy loses in that time, less the pixels this side
    /// loses. The dice are rolled afresh, so a plan can't lean on knowing how they'll fall.
    fn rehearse(&self, team: usize, plan: &[Option<Style>]) -> i32 {
        let mut a = self.clone();
        a.rehearsal = true;
        a.plan[team] = plan.to_vec();
        a.rng = Rng::new(self.tick as u64 * 2 + team as u64 + 1);
        let before = [a.score[0].px_lost, a.score[1].px_lost];
        let end = a.tick + ARCH_AHEAD;
        while a.over.is_none() && a.tick < end {
            a.step();
        }
        let lost = |t: usize| a.score[t].px_lost - before[t];
        lost(1 - team) - lost(team)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Six lancers (slot 1) against a frigate (slot 2), from the starter deck.
    fn fight(brain: [Brain; 2]) -> Arena {
        let d = starter_deck();
        let mut a = Arena::new();
        for (team, slot, x, y) in crate::pg::block(0, 1, 6, 150, 180, 10).into_iter().chain(crate::pg::block(1, 2, 1, 330, 180, 0)) {
            assert!(a.add(team, slot, &d[slot], x, y));
        }
        a.brain = brain;
        a
    }

    #[test]
    fn the_arch_ais_plan_plays_forward_no_worse_than_the_cards() {
        let mut a = fight([Brain::ARCH, Brain::CARD]);
        a.tick = 1;
        let plan = a.arch_plan(0);
        assert_eq!(plan.len(), 2, "one place for each slot up to the lancer's");
        assert!(a.rehearse(0, &plan) >= a.rehearse(0, &[]), "{plan:?}: {} against {}", a.rehearse(0, &plan), a.rehearse(0, &[]));
        // Whatever it kept for the lancers is better than each of the other ways.
        for style in STYLES {
            assert!(a.rehearse(0, &[None, Some(style)]) <= a.rehearse(0, &plan), "{style:?} was better than {plan:?}");
        }
    }

    #[test]
    fn a_side_with_an_arch_ai_flies_the_way_its_plan_says_and_a_card_side_plans_nothing() {
        // The plan as it stands, with the thinking switched off as it is in a fight played forward.
        let mut a = fight([Brain::ARCH, Brain::CARD]);
        a.rehearsal = true;
        a.plan[0] = vec![None, Some(Style::Orbit)];
        for _ in 0..30 {
            a.step();
        }
        assert!(a.ships.iter().filter(|s| s.team == 0).all(|s| s.doing == Doing::Orbit), "{:?}", a.ships.iter().map(|s| s.doing).collect::<Vec<_>>());
        // The same plan on a side with no arch-AI does nothing: lancers fly attack runs.
        let mut b = fight([Brain::CARD, Brain::CARD]);
        b.plan[0] = vec![None, Some(Style::Orbit)];
        for _ in 0..30 {
            b.step();
        }
        assert!(b.ships.iter().filter(|s| s.team == 0).all(|s| s.doing != Doing::Orbit));
        // In the real fight the arch side has a plan after the first tick, and the card side none.
        let mut c = fight([Brain::ARCH, Brain::CARD]);
        c.step();
        assert_eq!((c.plan[0].len(), c.plan[1].len()), (2, 0));
    }

    #[test]
    fn a_fight_played_forward_does_no_planning_of_its_own() {
        let mut a = fight([Brain::ARCH, Brain::ARCH]);
        a.rehearsal = true;
        let plan = [vec![None, Some(Style::Kite)], vec![None, None, Some(Style::Runs)]];
        a.plan = plan.clone();
        for _ in 0..ARCH_EVERY * 3 {
            a.step();
        }
        assert_eq!(a.plan, plan);
    }

    #[test]
    fn two_fights_with_arch_ais_play_out_the_same() {
        let run = || {
            let mut a = fight([Brain::ARCH, Brain::ARCH]);
            for _ in 0..ARCH_EVERY * 3 + 5 {
                a.step();
            }
            a.checksum()
        };
        assert_eq!(run(), run());
    }
}
