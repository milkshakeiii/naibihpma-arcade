//! The arch-AI: one planner for a whole side, which decides how each of its designs flies. It
//! plays the fight forward from where things stand, once for each way a design could fly, and
//! keeps the way that came out best. Nothing here knows what a part does: it only watches what
//! happens.
//!
//! It thinks about one design at a time, and only while there's a fight. Two things keep the
//! cost of a tick about the same however big the match is. What's played forward is the fight
//! alone: copies of the ships in it in a bare arena, with no match round them (see `pocket`).
//! And the playing forward is spread over the ticks that follow, so many ticks of it each real
//! tick: fewer when the fight is big.

use super::*;

/// How far ahead it plays each way of flying.
pub const ARCH_AHEAD: u32 = 15 * 8;
/// A ship is in a fight, and is played forward, while an enemy is within this of it.
pub const ARCH_WAKE: i32 = 300;
/// The planner's work in one real tick: ticks played forward, times the square of the ships in
/// the fight plus `ARCH_FLAT`. (The cost of a tick goes about as the square of the ships in it,
/// on top of something for the tick itself.)
pub const ARCH_WORK: i32 = 24_000;
pub const ARCH_FLAT: i32 = 200;
/// The fewest and the most ticks it plays forward in one real tick.
pub const ARCH_STEPS: (i32, i32) = (2, 40);
/// With no fight on, how often it looks for one.
pub const ARCH_IDLE: u32 = 8;

/// What a side's planner has decided for a design, for the bar under a ship of it.
pub fn chosen_line(name: &str, way: Style) -> String {
    let how = match way {
        Style::Runs => "ON ATTACK RUNS",
        Style::Kite => "KITING",
        Style::Orbit => "ORBITING",
    };
    format!("ITS SIDE HAS EVERY {name} {how}.")
}

/// A side's planner in the middle of a think: one design, and each way it could fly.
#[derive(Clone)]
pub struct Think {
    /// The fight as it stood when the think began: the ships in it and nothing else. Every way
    /// is played forward from this, on the same dice.
    base: Arena,
    pub slot: usize,
    /// The ways to try: the way the design flies now first.
    ways: Vec<Option<Style>>,
    scores: Vec<i32>,
    /// The way being played forward now.
    run: Option<Arena>,
    /// Ticks played forward each real tick.
    pub steps: u32,
}

impl Think {
    /// One real tick's worth of playing forward. When every way has been played, the way to
    /// fly: the one that came out best, and the way it flies now unless another did better.
    fn advance(&mut self, team: usize) -> Option<Option<Style>> {
        let mut left = self.steps;
        while left > 0 {
            if self.run.is_none() {
                let mut a = self.base.clone();
                if a.plan[team].len() <= self.slot {
                    a.plan[team].resize(self.slot + 1, None);
                }
                a.plan[team][self.slot] = self.ways[self.scores.len()];
                self.run = Some(a);
            }
            let a = self.run.as_mut().unwrap();
            let end = self.base.tick + ARCH_AHEAD;
            while left > 0 && a.over.is_none() && a.tick < end {
                a.step();
                left -= 1;
            }
            if a.over.is_some() || a.tick >= end {
                // The pixels the enemy lost in that time, less the pixels this side lost.
                self.scores.push(a.score[1 - team].px_lost - a.score[team].px_lost);
                self.run = None;
                if self.scores.len() == self.ways.len() {
                    let best = (0..self.ways.len()).max_by_key(|&k| (self.scores[k], std::cmp::Reverse(k))).unwrap();
                    return Some(self.ways[best]);
                }
            }
        }
        None
    }

    /// What each way scored, in the order tried, so far.
    #[cfg(test)]
    pub fn scores(&self) -> Vec<(Option<Style>, i32)> {
        self.ways.iter().copied().zip(self.scores.iter().copied()).collect()
    }
}

impl Arena {
    /// The arch-AI's turn, for each side that has one. A side in the middle of a think plays it
    /// forward a little further; a side with none looks for a fight to think about. Blue goes
    /// first. A fight being played forward inside a think does no planning of its own: there,
    /// each side flies on with the plan it has.
    pub(super) fn arch_think(&mut self) {
        if self.rehearsal {
            return;
        }
        for team in 0..2 {
            if self.brain[team].ways != Ways::Plan {
                continue;
            }
            let mut think = self.arch[team].take();
            if think.is_none() && self.tick >= self.arch_next[team] {
                think = self.arch_begin(team);
                if think.is_none() {
                    self.arch_next[team] = self.tick + ARCH_IDLE;
                }
            }
            if let Some(t) = think.as_mut() {
                if let Some(way) = t.advance(team) {
                    if self.plan[team].len() <= t.slot {
                        self.plan[team].resize(t.slot + 1, None);
                    }
                    self.plan[team][t.slot] = way;
                    think = None;
                    self.arch_next[team] = self.tick + 1;
                }
            }
            self.arch[team] = think;
        }
    }

    /// Which ships are in a fight: each one that's there and has an enemy within `ARCH_WAKE`.
    pub fn in_a_fight(&self) -> Vec<bool> {
        let there: Vec<usize> = (0..self.ships.len()).filter(|&i| self.ships[i].alive && self.ships[i].warp == 0).collect();
        let mut awake = vec![false; self.ships.len()];
        for &i in &there {
            let s = &self.ships[i];
            awake[i] = there.iter().any(|&j| self.ships[j].team != s.team && cheb(self.ships[j].x, self.ships[j].y, s.x, s.y) <= ARCH_WAKE);
        }
        awake
    }

    /// The fight alone, to play forward: a bare arena the size of this one, holding copies of
    /// the ships in a fight (`awake`) and of the shots flying at them or at nobody. There's no
    /// match round it: no banks, nothing printed or mined, nobody's AI, no rock. That, and
    /// leaving out every ship that isn't in a fight, is what makes it cheap. What it costs in
    /// truth: a king that was sent somewhere stands still, a miner stays where it is, and a
    /// ship that's hurt and unarmed runs for a corner and not for its base.
    pub fn pocket(&self, awake: &[bool]) -> Arena {
        let mut a = Arena::blank(self.w, self.h, 1);
        a.rock = Vec::new();
        // Where each ship is in the copy. A seed on its way and a ship still being printed
        // aren't in it: neither fights.
        let mut at: Vec<Option<usize>> = vec![None; self.ships.len()];
        for (i, s) in self.ships.iter().enumerate() {
            if awake[i] && s.role != Role::Seed && s.maker.is_none() {
                at[i] = Some(a.ships.len());
                a.ships.push(s.clone());
            }
        }
        let moved = |i: Option<usize>| i.and_then(|i| at.get(i).copied().flatten());
        for s in &mut a.ships {
            (s.prey, s.target, s.chase, s.aim_for) = (moved(s.prey), moved(s.target), moved(s.chase), moved(s.aim_for));
            for t in &mut s.turrets {
                t.aim_for = moved(t.aim_for);
            }
            s.making = Making::None;
            if s.order == Order::Harvest {
                (s.order, s.home, s.ring0) = (Order::Hold, (s.x, s.y), 0);
            }
        }
        for shot in &self.shots {
            let mut shot = shot.clone();
            match shot.target {
                Some(t) if moved(Some(t)).is_none() => continue,
                t => shot.target = moved(t),
            }
            a.shots.push(shot);
        }
        a.tick = self.tick;
        a.started = [a.alive(0) > 0, a.alive(1) > 0];
        (a.brain, a.plan, a.drills) = (self.brain, self.plan.clone(), self.drills.clone());
        a.rehearsal = true;
        a
    }

    /// Starts a think for a side, if it has a design in a fight with a choice of how to fly:
    /// the next such design in turn.
    fn arch_begin(&mut self, team: usize) -> Option<Box<Think>> {
        let awake = self.in_a_fight();
        // (Not a miner, armed or no: it mines, and in the copy it stays where it is.)
        let chooses = |s: &Ship| s.team == team && s.role == Role::Ship && s.grow.is_none() && s.maker.is_none() && s.st.armed() && s.st.flight != Flight::Still && !s.miner();
        let mut slots: Vec<usize> = (0..self.ships.len()).filter(|&i| awake[i] && chooses(&self.ships[i])).map(|i| self.ships[i].slot).collect();
        slots.sort();
        slots.dedup();
        if slots.is_empty() {
            return None;
        }
        self.arch_turn[team] += 1;
        let slot = slots[self.arch_turn[team] % slots.len()];
        // The way it flies now, and the other ways.
        let now = self.plan[team].get(slot).copied().flatten();
        let drawn = self.ships.iter().find(|s| s.alive && s.slot == slot && chooses(s)).and_then(|s| Style::of(s.st.flight));
        let mut ways = vec![now];
        ways.extend(STYLES.iter().filter(|&&w| Some(w) != now.or(drawn)).map(|&w| Some(w)));
        let mut base = self.pocket(&awake);
        // The dice are rolled afresh, so a plan can't lean on knowing how they'll fall.
        base.rng = Rng::new(self.tick as u64 * 2 + team as u64 + 1);
        let n = base.ships.len() as i32;
        let steps = (ARCH_WORK / (n * n + ARCH_FLAT)).clamp(ARCH_STEPS.0, ARCH_STEPS.1) as u32;
        Some(Box::new(Think { base, slot, ways, scores: Vec::new(), run: None, steps }))
    }

    /// For measuring: the fight alone played forward `ticks` ticks, as a think plays it (or,
    /// with `nobody`, a bare arena with no ships at all). Returns how many ships were in it.
    pub fn pocket_forward(&self, ticks: u32, nobody: bool) -> usize {
        let awake = if nobody { vec![false; self.ships.len()] } else { self.in_a_fight() };
        let mut a = self.pocket(&awake);
        let (n, end) = (a.ships.len(), a.tick + ticks);
        while a.tick < end {
            a.step();
            if a.over.is_some() {
                break;
            }
        }
        n
    }

    /// A copy of the fight, or the match, played forward `ticks` ticks with nobody planning.
    pub fn played_forward(&self, ticks: u32) -> Arena {
        let mut a = self.clone();
        a.arch = Default::default();
        a.rehearsal = true;
        let end = a.tick + ticks;
        while a.over.is_none() && a.tick < end {
            a.step();
        }
        a
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

    /// Steps until blue's first think is over, and returns the think as it stood on its last
    /// tick but one, with every way but the last scored.
    fn first_think(a: &mut Arena) -> (Think, u32) {
        a.step();
        let mut last = *a.arch[0].clone().expect("blue began no think on the first tick");
        let mut ticks = 1;
        while let Some(t) = a.arch[0].clone() {
            last = *t;
            a.step();
            ticks += 1;
            assert!(ticks < 400, "the think never ended");
        }
        (last, ticks)
    }

    #[test]
    fn a_think_plays_each_way_forward_from_the_same_start_and_keeps_the_best() {
        let mut a = fight([Brain::ARCH, Brain::PLAIN]);
        let (think, _) = first_think(&mut a);
        // The lancers, and three ways: as they fly now (their drawing's way, attack runs), then
        // the other two.
        assert_eq!(think.slot, 1);
        assert_eq!(think.ways, vec![None, Some(Style::Kite), Some(Style::Orbit)]);
        // Each way's score is what playing the start forward with that way gives: the same
        // start, the same dice.
        let play = |way: Option<Style>| {
            let mut b = think.base.clone();
            b.plan[0] = vec![None, way];
            let end = b.tick + ARCH_AHEAD;
            while b.over.is_none() && b.tick < end {
                b.step();
            }
            b.score[1].px_lost - b.score[0].px_lost
        };
        let scores: Vec<i32> = think.ways.iter().map(|&w| play(w)).collect();
        assert_eq!(think.scores(), vec![(None, scores[0]), (Some(Style::Kite), scores[1])]);
        // The plan is the best of the three, and stays as it was unless another way did better.
        let best = (0..3).max_by_key(|&k| (scores[k], std::cmp::Reverse(k))).unwrap();
        assert_eq!(a.plan[0], vec![None, think.ways[best]], "the three ways scored {scores:?}");
        assert!(scores.iter().any(|&s| s != scores[0]), "every way scored {}: the fight tells nothing apart", scores[0]);
        // A side with no arch-AI plans nothing.
        assert!(a.plan[1].is_empty() && a.arch[1].is_none());
    }

    #[test]
    fn a_think_is_spread_over_the_ticks_that_follow_and_never_plays_more_than_its_share_in_one() {
        // Seven ships in the fight: the most a tick may play forward, 40 ticks. Three ways of 8 s
        // each are 360 ticks at most (a way stops early if the fight is over), so nine real ticks
        // at most, and never all in one.
        let mut a = fight([Brain::ARCH, Brain::PLAIN]);
        let (think, ticks) = first_think(&mut a);
        assert!(think.steps as i32 == ARCH_STEPS.1 && (4..=10).contains(&ticks), "{} steps a tick, {ticks} ticks", think.steps);
        // Fifty ships in the fight: eight ticks a tick. Two hundred: the fewest, two.
        let d = starter_deck();
        for (each, steps) in [(25, 8), (100, ARCH_STEPS.0)] {
            let mut big = Arena::blank(1200, 800, 1);
            for k in 0..each {
                big.spawn(0, 0, &d[0], 100 + 12 * (k % 10), 100 + 12 * (k / 10), 1, false);
                big.spawn(1, 0, &d[0], 380 + 12 * (k % 10), 100 + 12 * (k / 10), 3, false);
            }
            big.brain = [Brain::ARCH, Brain::PLAIN];
            big.step();
            let t = big.arch[0].as_ref().expect("no think in a big fight");
            assert_eq!(t.steps as i32, steps, "{} ships", each * 2);
        }
        // In between, the work is what's shared out: twenty ships, forty ticks, the most
        // allowed; forty ships, thirteen.
        assert_eq!((ARCH_WORK / (20 * 20 + ARCH_FLAT), ARCH_WORK / (40 * 40 + ARCH_FLAT)), (ARCH_STEPS.1, 13));
    }

    #[test]
    fn only_the_ships_in_a_fight_are_played_forward() {
        // The lancers and the frigate, and far from them a blue fighter and a red one: each
        // with no enemy within reach of a fight.
        let d = starter_deck();
        let mut a = Arena::blank(2000, 400, 1);
        for (team, slot, x, y) in crate::pg::block(0, 1, 6, 150, 180, 10).into_iter().chain(crate::pg::block(1, 2, 1, 330, 180, 0)) {
            a.spawn(team, slot, &d[slot], x, y, if team == 0 { 1 } else { 3 }, false);
        }
        let far = [a.spawn(0, 0, &d[0], 1000, 200, 1, false), a.spawn(1, 0, &d[0], 1000 + ARCH_WAKE + 400, 200, 3, false)];
        for s in a.ships.iter_mut() {
            s.order = Order::Hunt;
        }
        a.brain = [Brain::ARCH, Brain::PLAIN];
        let awake = a.in_a_fight();
        assert_eq!(awake.iter().filter(|&&w| w).count(), 7);
        assert!(!awake[far[0]] && !awake[far[1]]);
        a.step();
        // The think's copy holds the seven and not the far two, in a bare arena: no match, no
        // rock, and nobody's think.
        let think = a.arch[0].clone().unwrap();
        assert_eq!(think.base.ships.len(), 7);
        assert!(think.base.ships.iter().all(|s| s.x < 400) && think.base.eco.is_none() && think.base.rock.is_empty());
        assert!(think.base.arch[0].is_none() && think.base.arch[1].is_none() && think.base.rehearsal);
        // Played forward, the seven fight on: they move, and somebody loses pixels.
        let mut b = think.base.clone();
        for _ in 0..ARCH_AHEAD {
            b.step();
        }
        assert!(b.ships[0].x != think.base.ships[0].x && b.score[0].px_lost + b.score[1].px_lost > 0);
        // The fight itself goes on whole: the far fighters set off for their nearest enemies.
        for _ in 0..60 {
            a.step();
        }
        assert!(a.ships.len() == 9 && a.ships[far[0]].x != 1000 && a.ships[far[1]].x != 1000 + ARCH_WAKE + 400);
    }

    #[test]
    fn a_match_has_a_planner_on_each_side_and_it_plays_the_fight_alone_forward() {
        // Every side of a match plans, a person's and an AI's. A bare arena's sides don't unless
        // they're told to (the window's arena tells them).
        assert_eq!(Arena::for_match(1, [starter_deck(), starter_deck()], [false, true]).brain, [Brain::ARCH; 2]);
        assert_eq!(Arena::new().brain, [Brain::PLAIN; 2]);
        // A match with nobody's AI. In the middle of the map, far from both kings: four blue
        // fighters and two blue lancers 120 px from a red frigate and two red fighters, a blue
        // miner beside them, and a blue fighter still being printed.
        let d = starter_deck();
        let mut a = Arena::for_match(1, [d.clone(), d.clone()], [false, false]);
        // Late in the match: longer in than a fight in the bare arena is allowed to run.
        a.tick = TIME_LIMIT + 15;
        let (cx, cy) = (a.w / 2, a.h / 2);
        let kings = a.eco.as_ref().unwrap().king;
        let mut fight = Vec::new();
        for k in 0..4 {
            fight.push(a.spawn(0, 0, &d[0], cx - 60, cy - 18 + 12 * k, 1, false));
        }
        for k in 0..2 {
            fight.push(a.spawn(0, 1, &d[1], cx - 75, cy - 6 + 12 * k, 1, false));
        }
        fight.push(a.spawn(1, 2, &d[2], cx + 60, cy, 3, false));
        for k in 0..2 {
            fight.push(a.spawn(1, 0, &d[0], cx + 75, cy - 6 + 12 * k, 3, false));
        }
        let miner = a.spawn(0, 3, &d[3], cx - 90, cy + 40, 1, false);
        let unborn = a.spawn(0, 0, &d[0], cx - 90, cy - 40, 1, false);
        a.ships[unborn].maker = Some(kings[0]);
        for &i in &fight {
            a.ships[i].order = Order::Hunt;
        }
        // The fight is on: after a tick each side is in the middle of a think.
        a.step();
        let think = a.arch[0].clone().expect("blue isn't thinking");
        assert!(a.arch[1].is_some());
        // What it plays forward is the fight alone: the nine ships and the miner, in a bare
        // arena. No kings (they're far off), no ship that's still being printed, no match (no
        // banks, nobody's AI), no rock, the same clock.
        let base = &think.base;
        assert_eq!(base.ships.len(), 10);
        assert!(base.eco.is_none() && base.rock.is_empty() && base.rehearsal && base.tick == a.tick);
        assert!(base.ships.iter().all(|s| s.role == Role::Ship && s.maker.is_none()));
        assert_eq!((base.alive(0), base.alive(1)), (7, 3));
        // The miner stays where it is there: it has no rock to work.
        let m = base.ships.iter().find(|s| s.slot == 3).unwrap();
        assert_eq!((m.order, m.home, m.ring0), (Order::Hold, (m.x, m.y), 0));
        // The real match is as it was: the miner mines, and everything is where it stood.
        assert_eq!(a.ships[miner].order, Order::Harvest);
        assert!(a.eco.is_some() && !a.rock.is_empty() && a.ships.len() == 2 + 11);
        // Let them fight a while, so there are shots in the air and ships after each other, and
        // look at the next think's copy: whoever a ship or a shot there is after is in the copy.
        for _ in 0..40 {
            a.step();
        }
        while a.arch[0].is_none() {
            a.step();
        }
        let base = &a.arch[0].as_ref().unwrap().base;
        let n = base.ships.len();
        assert!(base.ships.iter().any(|s| s.prey.is_some()) && !base.shots.is_empty(), "nothing to check: nobody is after anybody");
        assert!(base.ships.iter().all(|s| s.prey.map_or(true, |p| p < n) && s.target.map_or(true, |p| p < n) && s.chase.map_or(true, |p| p < n)));
        for s in base.ships.iter().filter(|s| s.prey.is_some()) {
            assert!(base.ships[s.prey.unwrap()].team != s.team, "a ship in the copy is after its own side");
        }
        // The plan it comes to is for a design in the fight, and each way was played to a score.
        let mut ticks = 0;
        while a.plan[0].iter().all(|w| w.is_none()) && a.plan[1].iter().all(|w| w.is_none()) && ticks < 15 * 30 {
            a.step();
            ticks += 1;
        }
        // One side or the other has found a way that beat the one its drawing gave it: the
        // fights it played forward ran their 8 s, late as the match's clock is.
        assert!(a.plan[0].iter().chain(&a.plan[1]).any(|w| w.is_some()), "thirty seconds of fighting and nobody chose a way: {:?}", a.plan);
        // (The miner is armed and flies, but it isn't thought about: it mines.)
        assert!(a.plan[0].len() <= 2 && a.plan[1].len() <= 3, "a plan for a design that isn't in the fight, or for the miner: {:?}", a.plan);
        for _ in 0..15 * 20 {
            a.step();
        }
        assert!(a.plan[0].len() <= 2 && a.plan[1].len() <= 3, "{:?}", a.plan);
        // What the bar says of a choice fits the bar, for the longest name there is.
        for way in STYLES {
            assert!(chosen_line("WWWWWWWWWW", way).len() <= 49, "{}", chosen_line("WWWWWWWWWW", way));
        }
    }

    #[test]
    fn with_no_fight_on_the_arch_ai_thinks_about_nothing() {
        // Two fighters too far apart to be in a fight, and told to stay where they are.
        let d = starter_deck();
        let mut a = Arena::blank(2000, 400, 1);
        for (team, x) in [(0, 300), (1, 1500)] {
            let i = a.spawn(team, 0, &d[0], x, 200, 1, false);
            a.ships[i].order = Order::Hold;
        }
        a.brain = [Brain::ARCH, Brain::ARCH];
        for _ in 0..40 {
            a.step();
            assert!(a.arch[0].is_none() && a.arch[1].is_none() && a.plan[0].is_empty() && a.plan[1].is_empty());
        }
    }

    #[test]
    fn a_side_with_an_arch_ai_flies_the_way_its_plan_says_and_a_card_side_plans_nothing() {
        // The plan as it stands, with the thinking switched off as it is in a fight played forward.
        let mut a = fight([Brain::ARCH, Brain::PLAIN]);
        a.rehearsal = true;
        a.plan[0] = vec![None, Some(Style::Orbit)];
        for _ in 0..30 {
            a.step();
        }
        assert!(a.ships.iter().filter(|s| s.team == 0).all(|s| s.doing == Doing::Orbit), "{:?}", a.ships.iter().map(|s| s.doing).collect::<Vec<_>>());
        // The same plan on a side with no arch-AI does nothing: lancers fly attack runs.
        let mut b = fight([Brain::PLAIN, Brain::PLAIN]);
        b.plan[0] = vec![None, Some(Style::Orbit)];
        for _ in 0..30 {
            b.step();
        }
        assert!(b.ships.iter().filter(|s| s.team == 0).all(|s| s.doing != Doing::Orbit));
    }

    #[test]
    fn a_way_forced_on_a_ship_comes_before_its_sides_plan() {
        // The lancers' side has them orbiting. One of them has a way forced on it, as a drill
        // forces one: it kites, and the other five orbit.
        let mut a = fight([Brain::ARCH, Brain::PLAIN]);
        a.rehearsal = true;
        a.plan[0] = vec![None, Some(Style::Orbit)];
        a.ships[0].style = Some(Style::Kite);
        for _ in 0..30 {
            a.step();
        }
        assert!(a.ships[0].doing != Doing::Orbit, "the forced ship is {:?}", a.ships[0].doing);
        assert!(a.ships[1..6].iter().all(|s| s.doing == Doing::Orbit), "{:?}", a.ships.iter().map(|s| s.doing).collect::<Vec<_>>());
    }

    #[test]
    fn a_fight_played_forward_does_no_planning_of_its_own() {
        let mut a = fight([Brain::ARCH, Brain::ARCH]);
        a.rehearsal = true;
        let plan = [vec![None, Some(Style::Kite)], vec![None, None, Some(Style::Runs)]];
        a.plan = plan.clone();
        for _ in 0..ARCH_AHEAD {
            a.step();
            assert!(a.arch[0].is_none() && a.arch[1].is_none());
        }
        assert_eq!(a.plan, plan);
    }

    #[test]
    fn two_fights_with_arch_ais_play_out_the_same() {
        let run = || {
            let mut a = fight([Brain::ARCH, Brain::ARCH]);
            for _ in 0..100 {
                a.step();
            }
            (a.checksum(), a.plan.clone())
        };
        assert_eq!(run(), run());
    }
}
