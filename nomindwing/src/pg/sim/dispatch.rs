//! The dispatcher: where a side's warships go, for a side the AI plays. Every so often it
//! plays the match itself forward once for each thing it could do (keep everyone home, send
//! everyone to one of the enemy's places, send everyone to a place of its own that's being
//! raided) and does what left it best off. It replaces the AI's rules of thumb (pixel counts and
//! timers) for when a wave leaves, where it goes and when it comes home.
//!
//! Nothing here knows what a part does. It only watches what happens.
//!
//! (Two earlier tries played out a fight off the map, the group against what stood at the
//! place. Counting only what stood there, the answer was rosy and changed by the second.
//! Counting the enemy's whole fleet, no attack on a defended place ever looked worth it, and a
//! side that never attacks is out-grown. See MATCH.md, 2026-11-13.)

use super::*;

/// How often a side's dispatcher thinks.
pub const DISPATCH_EVERY: u32 = 15 * 20;
/// How far ahead it plays the match for each thing it could do.
pub const DISPATCH_AHEAD: u32 = 15 * 60;
/// How much better off (in pixels) another plan has to leave it than the plan in hand, to be
/// taken instead.
pub const SWITCH: i32 = 100;
/// Once it has changed its plan it keeps to it this long, whatever else looks better. Only a
/// raid on one of its own places is weighed against it in that time. (Without this it changed
/// its mind every other thought, and lost its ships on the way there and back.)
pub const DISPATCH_HOLD: u32 = 15 * 60;
/// How near one of its places an enemy has to be for the place to count as raided.
pub const SITE_R: i32 = 200;
/// A ship sent to a place fights what comes this near it on the way.
pub const SENT_NEAR: i32 = 130;
/// The enemy's places it weighs going to (the nearest to its fleet), and its own raided places
/// (the hardest pressed).
const PLACES: usize = 2;
const RAIDED: usize = 1;
/// What a king's fall is worth, in pixels.
const KING: i32 = 100_000;

/// What a side's dispatcher remembers between thoughts.
#[derive(Clone, Debug, Default, PartialEq, Eq, Hash)]
pub struct Dispatch {
    next: u32,
    /// The plan in hand: the thing (a place of the enemy's, or one of its own) that every
    /// warship is sent to. None: everyone is home.
    pub plan: Option<usize>,
    /// When it last changed its plan.
    pub since: u32,
    /// Times it has thought, times it has changed its plan, and matches it has played forward.
    pub thoughts: u32,
    pub switches: u32,
    pub looked: u32,
}

/// The plan to follow, from how each plan came out when played forward (the pixels it left the
/// side ahead by): the best of them, if it beats the plan in hand by `SWITCH` or the plan in
/// hand isn't among them any more. Otherwise the plan in hand. On a tie, the first.
pub fn choose(current: Option<usize>, scored: &[(Option<usize>, i32)]) -> Option<usize> {
    let Some(&(best, top)) = scored.iter().rev().max_by_key(|s| s.1) else { return None };
    match scored.iter().find(|s| s.0 == current) {
        Some(&(_, now)) if top < now + SWITCH => current,
        _ => best,
    }
}

impl Arena {
    /// From now on side `t`'s AI sends its warships by playing the match forward, not by its
    /// rules of thumb. (Only a side an AI plays has anyone to do the sending.)
    pub fn send_by_looking(&mut self, t: usize) {
        if let Some(e) = self.eco.as_mut() {
            e.dispatch[t] = Some(Dispatch::default());
        }
    }

    /// Ships sent after a thing: the place they're sent to moves to where the thing is now. A
    /// thing that's gone leaves the place where it last was.
    pub(super) fn chase_step(&mut self) {
        for i in 0..self.ships.len() {
            if let Some(t) = self.ships[i].chase {
                if self.ships.get(t).is_some_and(|o| o.alive) {
                    self.ships[i].sent = Some((self.ships[t].x, self.ships[t].y));
                } else {
                    self.ships[i].chase = None;
                }
            }
        }
    }

    /// A side's warships that a dispatcher sends: finished, flying, and not miners.
    fn dispatchable(&self, t: usize) -> Vec<usize> {
        let cards = &self.eco.as_ref().unwrap().cards[t];
        (0..self.ships.len())
            .filter(|&i| {
                let s = &self.ships[i];
                s.alive && s.team == t && s.role == Role::Ship && s.grow.is_none() && s.warp == 0 && s.maker.is_none() && s.st.thrust > 0 && cards.get(s.slot).is_some_and(|c| c.warship())
            })
            .collect()
    }

    /// A side's places: its king, and everything it has planted.
    fn sites(&self, t: usize) -> Vec<usize> {
        (0..self.ships.len()).filter(|&i| self.ships[i].alive && self.ships[i].team == t && (self.ships[i].role == Role::King || self.ships[i].rooted)).collect()
    }

    /// The pixels of the enemy's armed ships that have come within `SITE_R` of a place.
    fn raid_on(&self, site: usize) -> i32 {
        let p = &self.ships[site];
        self.ships
            .iter()
            .filter(|s| s.alive && s.team != p.team && s.role != Role::Seed && s.warp == 0 && s.maker.is_none() && s.grow.is_none() && s.st.armed() && (s.st.thrust > 0 || s.role == Role::King) && cheb(s.x, s.y, p.x, p.y) <= SITE_R)
            .map(|s| s.px_now())
            .sum()
    }

    /// What a side has: every pixel standing (its king's too) and every pixel in its banks.
    pub fn material(&self, t: usize) -> i32 {
        self.ships.iter().filter(|s| s.alive && s.team == t).map(|s| s.px_now() + s.store).sum()
    }

    /// Puts a plan into effect: every warship of the side is sent to where the thing stands, at
    /// the pace of the slowest of them, or let go home.
    fn send_all(&mut self, t: usize, plan: Option<usize>) {
        let all = self.dispatchable(t);
        let to = plan.filter(|&p| self.ships[p].alive).map(|p| (self.ships[p].x, self.ships[p].y));
        let pace = all.iter().map(|&i| self.ships[i].st.vmax).min().unwrap_or(0);
        for i in all {
            (self.ships[i].sent, self.ships[i].pace) = (to, if to.is_some() { pace } else { 0 });
            self.ships[i].chase = None;
        }
    }

    /// The match played forward `DISPATCH_AHEAD` ticks with side `t` following `plan`, and
    /// everything else going on as it is: the other side's AI plays, both sides print and mine,
    /// and no dispatcher thinks again. How far ahead that leaves the side, in pixels (what it
    /// has less what the enemy has), with a king's fall on top. The dice are rolled afresh.
    pub fn look_ahead(&self, t: usize, plan: Option<usize>) -> i32 {
        let mut a = self.clone();
        a.rehearsal = true;
        a.rng = Rng::new(self.tick as u64 * 2 + t as u64 + 7);
        a.send_all(t, plan);
        let end = a.tick + DISPATCH_AHEAD;
        while a.over.is_none() && a.tick < end {
            a.step();
        }
        let won = match a.over {
            Some(Some(w)) if w == t => KING,
            Some(Some(_)) => -KING,
            _ => 0,
        };
        a.material(t) - a.material(1 - t) + won
    }

    /// The dispatcher's turn for side `t`, every `DISPATCH_EVERY` ticks (and at once when what it
    /// was sent against is gone). A match being played forward does no thinking of its own.
    pub(super) fn dispatch(&mut self, t: usize) {
        if self.rehearsal {
            return;
        }
        let Some(mut d) = self.eco.as_ref().unwrap().dispatch[t].clone() else { return };
        let gone = d.plan.is_some_and(|p| !self.ships[p].alive);
        if gone {
            d.plan = None;
        }
        if self.tick < d.next && !gone {
            return;
        }
        d.next = self.tick + DISPATCH_EVERY;
        d.thoughts += 1;
        let all = self.dispatchable(t);
        // Keeping to a plan it changed to lately: only its own raided places are weighed
        // against it.
        let held = !gone && d.thoughts > 1 && self.tick < d.since + DISPATCH_HOLD;
        let mut plans: Vec<Option<usize>> = if held { Vec::new() } else { vec![None] };
        if !all.is_empty() {
            // The enemy's places nearest its fleet.
            let n = all.len() as i32;
            let mid = (all.iter().map(|&i| self.ships[i].x).sum::<i32>() / n, all.iter().map(|&i| self.ships[i].y).sum::<i32>() / n);
            let mut theirs = self.sites(1 - t);
            theirs.sort_by_key(|&s| (dist2(self.ships[s].x, self.ships[s].y, mid.0, mid.1), s));
            if !held {
                plans.extend(theirs.into_iter().take(PLACES).map(Some));
            }
            // Its own places that are being raided, the hardest pressed first.
            let mut raided: Vec<(i32, usize)> = self.sites(t).into_iter().map(|s| (self.raid_on(s), s)).filter(|r| r.0 > 0).collect();
            raided.sort_by_key(|&(px, s)| (std::cmp::Reverse(px), s));
            plans.extend(raided.into_iter().take(RAIDED).map(|r| Some(r.1)));
        }
        // The plan in hand is always weighed again.
        if !plans.contains(&d.plan) {
            plans.push(d.plan);
        }
        let scored: Vec<(Option<usize>, i32)> = plans.iter().map(|&p| (p, self.look_ahead(t, p))).collect();
        d.looked += scored.len() as u32;
        let plan = choose(d.plan, &scored);
        if std::env::var("NMW_TRACE").is_ok() {
            let name = |p: Option<usize>| p.map_or("HOME".to_string(), |p| format!("{} {}", if self.ships[p].team == t { "OWN" } else { "THEIR" }, self.ships[p].name));
            let list: Vec<String> = scored.iter().map(|&(p, s)| format!("{} {s}", name(p))).collect();
            eprintln!("{:5} team {t}: {} ships; {} -> {}", self.tick / 15, all.len(), list.join(", "), name(plan));
        }
        if plan != d.plan {
            d.switches += 1;
            d.since = self.tick;
        }
        d.plan = plan;
        self.send_all(t, plan);
        self.eco.as_mut().unwrap().dispatch[t] = Some(d);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A match with nobody playing either side: nothing moves but what a test moves.
    fn quiet_match() -> Arena {
        Arena::for_bots(1, [starter_deck(), starter_deck()], [Bot::None, Bot::None])
    }

    /// `n` ships of the starter deck's slot `slot` for a side, in rows of eight from (x, y).
    fn fleet(a: &mut Arena, team: usize, slot: usize, n: i32, x: i32, y: i32) -> Vec<usize> {
        let d = starter_deck();
        (0..n).map(|k| a.spawn(team, slot, &d[slot], x + 9 * (k % 8), y + 9 * (k / 8), if team == 0 { 1 } else { 3 }, false)).collect()
    }

    fn state(a: &Arena, t: usize) -> Dispatch {
        a.eco.as_ref().unwrap().dispatch[t].clone().unwrap()
    }

    fn kings(a: &Arena) -> (usize, usize) {
        (a.eco.as_ref().unwrap().king[0], a.eco.as_ref().unwrap().king[1])
    }

    /// A blue fighter at (100, 100) and two red outposts, which can't move: one 200 px off to the
    /// side, the nearer; one 300 px ahead.
    fn fighter_and_two_outposts() -> Arena {
        let d = starter_deck();
        let mut a = Arena::new();
        assert!(a.add(0, 0, &d[0], 100, 100) && a.add(1, 4, &d[4], 100, 300) && a.add(1, 4, &d[4], 400, 100));
        a
    }

    #[test]
    fn a_ship_sent_to_a_place_flies_there_and_fights_what_is_round_it() {
        // On its card it goes for the nearest enemy: the outpost to the side.
        let mut a = fighter_and_two_outposts();
        a.step();
        assert_eq!(a.ships[0].prey, Some(1));
        // Sent to where the far one stands, it sets off for the place and leaves the near one be.
        let mut a = fighter_and_two_outposts();
        a.ships[0].sent = Some((400, 100));
        a.step();
        assert_eq!((a.ships[0].doing, a.ships[0].prey), (Doing::Sent, None));
        // It takes up the fight when it's within reach of the place, with what's round the place.
        let mut ticks = 0;
        while a.ships[0].prey.is_none() && ticks < 300 {
            a.step();
            ticks += 1;
        }
        assert_eq!(a.ships[0].prey, Some(2));
        assert!(cheb(a.ships[0].x, a.ships[0].y, 400, 100) <= GUARD_RADIUS + 50 && a.ships[0].x > 180, "it took the fight up at {}", a.ships[0].x);
        // A thing with no engines sent somewhere stays where it is.
        let mut a = fighter_and_two_outposts();
        a.ships[1].sent = Some((50, 50));
        for _ in 0..10 {
            a.step();
        }
        assert_eq!((a.ships[1].x, a.ships[1].y), (100, 300));
    }

    #[test]
    fn on_its_way_a_sent_ship_fights_what_comes_close_and_keeps_its_groups_pace() {
        // An outpost 110 px from where it starts, off its path: inside `SENT_NEAR`.
        let d = starter_deck();
        let mut a = Arena::new();
        assert!(a.add(0, 0, &d[0], 100, 100) && a.add(1, 4, &d[4], 200, 150) && a.add(1, 4, &d[4], 440, 100));
        a.ships[0].sent = Some((440, 100));
        a.step();
        assert_eq!(a.ships[0].prey, Some(1));
        // With nothing near, how far it gets in two seconds: flat out, and held to a pace of a
        // pixel a tick.
        let far = |pace: i32| {
            let mut a = Arena::new();
            assert!(a.add(0, 0, &d[0], 60, 40));
            (a.ships[0].sent, a.ships[0].pace) = (Some((460, 40)), pace);
            for _ in 0..30 {
                a.step();
            }
            a.ships[0].x - 60
        };
        assert!(far(4) <= 31 && far(4) >= 20 && far(0) >= far(4) + 15, "{} px at a pace, {} flat out", far(4), far(0));
    }

    #[test]
    fn ships_sent_to_one_point_each_take_a_clear_spot_by_it_and_stand_still() {
        // Eight fighters and two lancers a little apart, all sent to the one point, 150 px off.
        let mut a = quiet_match();
        let mut ships = fleet(&mut a, 0, 0, 8, 300, 800);
        ships.extend(fleet(&mut a, 0, 1, 2, 300, 840));
        let to = (450, 760);
        for &ship in &ships {
            a.command(Cmd::Send { team: 0, ship, at: to, pace: 0 });
        }
        for _ in 0..15 * 20 {
            a.step();
        }
        // The first of them stands on the point itself. The rest are round it, each clear of
        // every other by its own size and two pixels, and none is far off.
        let at = |a: &Arena, i: usize| (a.ships[i].x, a.ships[i].y);
        assert_eq!(at(&a, ships[0]), to);
        for (n, &i) in ships.iter().enumerate() {
            assert!(a.ships[i].parked && a.ships[i].doing == Doing::Sent && a.ships[i].goal == Some(at(&a, i)), "ship {i} hasn't settled: {:?} at {:?}", a.ships[i].goal, at(&a, i));
            assert!(cheb(a.ships[i].x, a.ships[i].y, to.0, to.1) <= 24, "ship {i} stands {} px off", cheb(a.ships[i].x, a.ships[i].y, to.0, to.1));
            for &j in &ships[..n] {
                let need = a.ships[i].half() + a.ships[j].half() + 2;
                assert!(cheb(a.ships[i].x, a.ships[i].y, a.ships[j].x, a.ships[j].y) >= need, "ships {i} and {j} stand on each other");
            }
        }
        // And there they stay.
        let then: Vec<(i32, i32)> = ships.iter().map(|&i| at(&a, i)).collect();
        for _ in 0..30 {
            a.step();
            assert_eq!(ships.iter().map(|&i| at(&a, i)).collect::<Vec<_>>(), then, "ships sent to a point kept moving");
        }
        // Let go, they're no longer on a spot of their own.
        a.command(Cmd::Home { team: 0, ship: ships[0] });
        a.step();
        assert!(!a.ships[ships[0]].parked && a.ships[ships[1]].parked);
    }

    #[test]
    fn a_sent_ship_flies_through_a_rank_already_standing_to_its_own_spot_beyond() {
        // Three fighters standing in a rank on the spots they were sent to, ten pixels apart.
        let mut a = quiet_match();
        let d = starter_deck();
        let rank: Vec<usize> = [790, 800, 810].iter().map(|&y| a.spawn(0, 0, &d[0], 400, y, 1, false)).collect();
        for (&ship, y) in rank.iter().zip([790, 800, 810]) {
            a.command(Cmd::Send { team: 0, ship, at: (400, y), pace: 0 });
        }
        // A fourth behind them, sent to a spot beyond, at a slow group's pace: its way lies
        // between two of them.
        let late = a.spawn(0, 0, &d[0], 350, 795, 1, false);
        a.command(Cmd::Send { team: 0, ship: late, at: (440, 795), pace: 3 });
        let stood: Vec<(i32, i32)> = rank.iter().map(|&i| (a.ships[i].x, a.ships[i].y)).collect();
        for _ in 0..15 * 20 {
            a.step();
            assert_eq!(rank.iter().map(|&i| (a.ships[i].x, a.ships[i].y)).collect::<Vec<_>>(), stood, "a ship flying by pushed the rank about");
        }
        // It got through, and stands on its spot. (Pushed like any other ship, it never did: two
        // friends pushing it back were more than its pace.)
        assert_eq!((a.ships[late].x, a.ships[late].y), (440, 795));
    }

    #[test]
    fn a_send_command_sends_only_a_finished_ship_of_the_side_that_gave_it() {
        let mut a = quiet_match();
        let (mk, _) = kings(&a);
        let mine = fleet(&mut a, 0, 0, 2, 300, 800);
        let theirs = fleet(&mut a, 1, 0, 1, 900, 300)[0];
        let post = fleet(&mut a, 0, 4, 1, 400, 800)[0];
        // Blue sends: its own fighter, the enemy's, its outpost (no engines), the enemy's king, a
        // ship that isn't there. And red sends blue's other fighter.
        let (_, ek) = kings(&a);
        for (team, ship) in [(0, mine[0]), (0, theirs), (0, post), (0, ek), (0, 99_999), (1, mine[1])] {
            a.command(Cmd::Send { team, ship, at: (5000, -20), pace: 7 });
        }
        a.step();
        // Its own fighter goes, to the nearest place on the map. Nothing else does.
        assert_eq!((a.ships[mine[0]].sent, a.ships[mine[0]].pace), (Some((MW - 1, 0)), 7));
        assert!([theirs, post, ek, mine[1]].iter().all(|&i| a.ships[i].sent.is_none()));
        // Its own king can be sent: it flies there along both axes, keeps the pace it was given,
        // and stops when it arrives.
        let from = (a.ships[mk].x, a.ships[mk].y);
        a.command(Cmd::Send { team: 0, ship: mk, at: (from.0 + 40, from.1 - 10), pace: 4 });
        a.step();
        a.step();
        assert_eq!((a.ships[mk].sent, a.ships[mk].x > from.0), (Some((from.0 + 40, from.1 - 10)), true));
        // A pixel a tick at a pace of 4, where its own speed is two and a half.
        let x0 = a.ships[mk].x;
        for _ in 0..8 {
            a.step();
        }
        assert!((7..=9).contains(&(a.ships[mk].x - x0)), "{} px in 8 ticks", a.ships[mk].x - x0);
        for _ in 0..60 {
            a.step();
        }
        assert!((a.ships[mk].x - from.0 - 40).abs() <= 1 && (a.ships[mk].y - from.1 + 10).abs() <= 1 && a.ships[mk].sent.is_none(), "{:?}", (a.ships[mk].x, a.ships[mk].y, a.ships[mk].sent));
        // Sent again, its keys take it back.
        a.command(Cmd::Send { team: 0, ship: mk, at: (from.0, from.1), pace: 0 });
        a.step();
        assert!(a.ships[mk].sent.is_some());
        a.command(Cmd::Steer { team: 0, dx: 0, dy: 1 });
        a.step();
        a.step();
        assert!(a.ships[mk].sent.is_none());
        a.command(Cmd::Steer { team: 0, dx: 0, dy: 0 });
        a.step();
        // Only its own side can let it go.
        a.command(Cmd::Home { team: 1, ship: mine[0] });
        a.step();
        assert!(a.ships[mine[0]].sent.is_some());
        a.command(Cmd::Home { team: 0, ship: mine[0] });
        a.step();
        assert_eq!((a.ships[mine[0]].sent, a.ships[mine[0]].pace), (None, 0));
    }

    #[test]
    fn a_ship_sent_after_a_thing_is_sent_wherever_it_is_and_then_where_it_last_was() {
        let mut a = quiet_match();
        let (mk, _) = kings(&a);
        let mine = fleet(&mut a, 0, 0, 1, 300, 800)[0];
        let prey = fleet(&mut a, 1, 2, 1, 900, 300)[0];
        let state = |a: &Arena, i: usize| (a.ships[i].chase, a.ships[i].sent);
        a.command(Cmd::Follow { team: 0, ship: mine, target: prey, pace: 0 });
        a.step();
        assert_eq!(state(&a, mine), (Some(prey), Some((900, 300))));
        // It's somewhere else: so is the place.
        (a.ships[prey].x, a.ships[prey].y) = (700, 500);
        a.step();
        assert_eq!(state(&a, mine), (Some(prey), Some((700, 500))));
        // After itself, after a thing that isn't there, or told by the other side: nothing changes.
        for c in [Cmd::Follow { team: 0, ship: mine, target: mine, pace: 0 }, Cmd::Follow { team: 0, ship: mine, target: 99_999, pace: 0 }, Cmd::Follow { team: 1, ship: mine, target: mk, pace: 0 }] {
            a.command(c);
        }
        a.step();
        assert_eq!(a.ships[mine].chase, Some(prey));
        // Sent to a place, it's after nothing. Let go, the same.
        a.command(Cmd::Send { team: 0, ship: mine, at: (400, 400), pace: 0 });
        a.step();
        assert_eq!(state(&a, mine), (None, Some((400, 400))));
        a.command(Cmd::Follow { team: 0, ship: mine, target: prey, pace: 0 });
        a.step();
        a.command(Cmd::Home { team: 0, ship: mine });
        a.step();
        assert_eq!(state(&a, mine), (None, None));
        // After it once more, and it dies: the order stands where it last was.
        a.command(Cmd::Follow { team: 0, ship: mine, target: prey, pace: 0 });
        a.step();
        (a.ships[prey].x, a.ships[prey].y) = (650, 450);
        a.step();
        a.ships[prey].alive = false;
        a.step();
        assert_eq!(state(&a, mine), (None, Some((650, 450))));
        // A king sent after a thing stays sent when it gets there: it goes on with it.
        let by = (a.ships[mk].x + 30, a.ships[mk].y);
        let friend = fleet(&mut a, 0, 2, 1, by.0, by.1)[0];
        a.command(Cmd::Follow { team: 0, ship: mk, target: friend, pace: 0 });
        for _ in 0..40 {
            a.step();
        }
        assert!(a.ships[mk].chase == Some(friend) && a.ships[mk].sent.is_some());
    }

    #[test]
    fn a_ship_with_no_guns_that_is_sent_goes_too() {
        // The AEGIS has engines and no guns. Sent across the arena it sets off, where with no
        // order from anyone it would stay beside an armed friend (and with none, stay put).
        let d = starter_deck();
        let ex = part_examples();
        let aegis = ex.iter().find(|d| d.name == "AEGIS").unwrap();
        assert!(!aegis.stats().armed() && aegis.stats().thrust > 0);
        let fly = |sent: Option<(i32, i32)>| {
            let mut a = Arena::new();
            assert!(a.add(0, 0, aegis, 60, 40) && a.add(1, 4, &d[4], 440, 320));
            a.ships[0].sent = sent;
            for _ in 0..30 {
                a.step();
            }
            (a.ships[0].doing, a.ships[0].x)
        };
        let (doing, x) = fly(Some((400, 40)));
        assert!(doing == Doing::Sent && x > 80, "{doing:?} at {x}");
        assert_eq!(fly(None).1, 60);
    }

    #[test]
    fn the_plan_in_hand_is_kept_unless_another_beats_it_by_a_margin() {
        let (home, a, b) = (None, Some(4), Some(9));
        // Another plan better by less than the margin: no change. By the margin: taken.
        assert_eq!(choose(home, &[(home, 0), (a, SWITCH - 1), (b, -50)]), home);
        assert_eq!(choose(home, &[(home, 0), (a, SWITCH), (b, -50)]), a);
        assert_eq!(choose(a, &[(home, 0), (a, -SWITCH + 1), (b, -500)]), a);
        assert_eq!(choose(a, &[(home, 0), (a, -SWITCH), (b, -500)]), home);
        // The plan in hand isn't among those weighed (what it was sent against is gone): the best.
        assert_eq!(choose(Some(7), &[(home, 0), (a, 5), (b, 3)]), a);
        // On a tie, the first. With nothing weighed, home.
        assert_eq!(choose(Some(7), &[(home, 5), (a, 5)]), home);
        assert_eq!(choose(a, &[]), None);
    }

    #[test]
    fn a_match_played_forward_gives_the_same_answer_every_time_and_changes_nothing() {
        let mut a = quiet_match();
        let (_, ek) = kings(&a);
        fleet(&mut a, 0, 1, 6, 400, 500);
        let before = a.checksum();
        let (home, go) = (a.look_ahead(0, None), a.look_ahead(0, Some(ek)));
        assert_eq!((a.look_ahead(0, None), a.look_ahead(0, Some(ek)), a.checksum()), (home, go, before));
        // What a side has is every pixel standing, its king's too, and every pixel banked.
        let standing: i32 = a.ships.iter().filter(|s| s.team == 0).map(|s| s.px_now()).sum();
        assert_eq!(a.material(0), standing + START_BANK);
    }

    #[test]
    fn a_dispatcher_sends_everyone_where_playing_the_match_forward_says_it_pays() {
        let mut a = quiet_match();
        a.send_by_looking(0);
        let (_, ek) = kings(&a);
        let at = (a.ships[ek].x, a.ships[ek].y);
        // Forty lancers 300 px from a king with nothing round it: played forward, the king falls.
        let lancers = fleet(&mut a, 0, 1, 40, at.0 - 300, at.1 + 60);
        assert!(a.look_ahead(0, Some(ek)) > a.look_ahead(0, None) + KING / 2);
        a.dispatch(0);
        let d = state(&a, 0);
        assert_eq!((d.plan, d.thoughts, d.switches, d.looked), (Some(ek), 1, 1, 2));
        let slowest = lancers.iter().map(|&i| a.ships[i].st.vmax).min().unwrap();
        assert!(lancers.iter().all(|&i| a.ships[i].sent == Some(at) && a.ships[i].pace == slowest));
        // It doesn't think again until its next turn.
        a.dispatch(0);
        assert_eq!(state(&a, 0).thoughts, 1);
        // The other side has no dispatcher.
        assert!(a.eco.as_ref().unwrap().dispatch[1].is_none());
    }

    #[test]
    fn its_own_kings_fall_counts_against_a_plan_as_the_enemys_counts_for_it() {
        // Forty red lancers set down beside blue's king, guarding where they stand: played
        // forward, blue's king falls, whatever blue's two fighters far away are told.
        let mut a = quiet_match();
        let (mk, _) = kings(&a);
        fleet(&mut a, 0, 0, 2, 700, 500);
        let home = (a.ships[mk].x, a.ships[mk].y);
        fleet(&mut a, 1, 1, 40, home.0 + 60, home.1 - 60);
        assert!(a.look_ahead(0, None) < -KING / 2, "{}", a.look_ahead(0, None));
        // Seen from red's side, the same match is that much to the good.
        assert!(a.look_ahead(1, None) > KING / 2);
    }

    #[test]
    fn a_side_with_a_dispatcher_keeps_its_cards_on_guard_where_the_ai_would_turn_them_out() {
        // The AI plays blue. Red fighters at blue's king are a raid: the AI's own answer is to
        // turn every warship card out on HUNT. With a dispatcher the cards stay on GUARD, and
        // the dispatcher does the sending.
        let orders = |looking: bool| {
            let mut a = Arena::for_bots(1, [starter_deck(), starter_deck()], [Bot::Ai, Bot::None]);
            if looking {
                a.send_by_looking(0);
            }
            let (mk, _) = kings(&a);
            let home = (a.ships[mk].x, a.ships[mk].y);
            fleet(&mut a, 1, 0, 3, home.0 + 100, home.1 - 20);
            for _ in 0..20 {
                a.step();
            }
            let e = a.eco.as_ref().unwrap();
            let warships: Vec<Order> = (0..e.cards[0].len()).filter(|&s| e.cards[0][s].warship()).map(|s| e.orders[0][s]).collect();
            (warships, e.dispatch[0].as_ref().map(|d| d.thoughts))
        };
        let (cards, thoughts) = orders(false);
        assert!(!cards.is_empty() && cards.iter().all(|&o| o == Order::Hunt) && thoughts.is_none(), "{cards:?}");
        let (cards, thoughts) = orders(true);
        assert!(cards.iter().all(|&o| o == Order::Guard) && thoughts.is_some_and(|n| n > 0), "{cards:?} {thoughts:?}");
    }

    #[test]
    fn a_dispatcher_keeps_everyone_home_from_a_fight_that_plays_out_badly() {
        let mut a = quiet_match();
        a.send_by_looking(0);
        let (_, ek) = kings(&a);
        let at = (a.ships[ek].x, a.ships[ek].y);
        // Two fighters, and forty frigates round the king they'd be sent against.
        let mine = fleet(&mut a, 0, 0, 2, at.0 - 300, at.1 + 60);
        fleet(&mut a, 1, 2, 40, at.0 - 150, at.1 - 40);
        a.dispatch(0);
        assert_eq!(state(&a, 0).plan, None);
        assert!(mine.iter().all(|&i| a.ships[i].sent.is_none() && a.ships[i].pace == 0));
    }

    #[test]
    fn a_plan_just_taken_is_kept_to_with_only_a_raid_at_home_weighed_against_it() {
        let mut a = quiet_match();
        a.send_by_looking(0);
        let (mk, ek) = kings(&a);
        let (home, at) = ((a.ships[mk].x, a.ships[mk].y), (a.ships[ek].x, a.ships[ek].y));
        // Thirty fighters 350 px from home, sent a moment ago against a king that now has forty
        // frigates round it (frigates beat fighters). Played forward, going on is far worse than
        // going home.
        let mine = fleet(&mut a, 0, 0, 30, home.0 + 300, home.1 - 180);
        fleet(&mut a, 1, 2, 40, at.0 - 200, at.1 - 60);
        a.tick = 15 * 600;
        a.eco.as_mut().unwrap().dispatch[0] = Some(Dispatch { plan: Some(ek), since: a.tick, thoughts: 3, ..Dispatch::default() });
        assert!(a.look_ahead(0, None) >= a.look_ahead(0, Some(ek)) + SWITCH, "home {} going {}", a.look_ahead(0, None), a.look_ahead(0, Some(ek)));
        // It keeps to the plan all the same: going home isn't weighed.
        a.dispatch(0);
        assert_eq!((state(&a, 0).plan, state(&a, 0).looked), (Some(ek), 1));
        // Twelve red lancers at its own king: going there is weighed against going on, and
        // nothing else is. Whichever of the two came out better, everyone is sent there.
        fleet(&mut a, 1, 1, 12, home.0 + 100, home.1 - 40);
        a.tick += DISPATCH_EVERY;
        a.dispatch(0);
        let d = state(&a, 0);
        assert_eq!(d.looked, 3);
        let want = choose(Some(ek), &[(Some(mk), a.look_ahead(0, Some(mk))), (Some(ek), a.look_ahead(0, Some(ek)))]);
        assert_eq!(d.plan, want);
        let to = if want == Some(mk) { home } else { at };
        assert!(mine.iter().all(|&i| a.ships[i].sent == Some(to)));
    }

    #[test]
    fn once_it_has_kept_to_a_plan_long_enough_a_better_one_is_taken() {
        let mut a = quiet_match();
        a.send_by_looking(0);
        let (mk, ek) = kings(&a);
        let (home, at) = ((a.ships[mk].x, a.ships[mk].y), (a.ships[ek].x, a.ships[ek].y));
        let mine = fleet(&mut a, 0, 0, 30, home.0 + 300, home.1 - 180);
        fleet(&mut a, 1, 2, 40, at.0 - 200, at.1 - 60);
        a.tick = 15 * 600;
        a.eco.as_mut().unwrap().dispatch[0] = Some(Dispatch { plan: Some(ek), since: a.tick - DISPATCH_HOLD, thoughts: 3, ..Dispatch::default() });
        a.dispatch(0);
        assert_eq!((state(&a, 0).plan, state(&a, 0).since), (None, a.tick));
        assert!(mine.iter().all(|&i| a.ships[i].sent.is_none()));
    }

    #[test]
    fn a_plan_whose_target_is_gone_is_thought_about_again_at_once() {
        let mut a = quiet_match();
        a.send_by_looking(0);
        let mine = fleet(&mut a, 0, 0, 2, 700, 500);
        // Sent against a red outpost, which then dies; its next turn isn't due yet.
        let post = fleet(&mut a, 1, 4, 1, 900, 500)[0];
        a.ships[post].rooted = true;
        a.tick = 15 * 600;
        a.eco.as_mut().unwrap().dispatch[0] = Some(Dispatch { plan: Some(post), since: a.tick, next: a.tick + 100, thoughts: 3, ..Dispatch::default() });
        a.dispatch(0);
        assert_eq!(state(&a, 0).thoughts, 3, "not due, and its target stands");
        a.ships[post].alive = false;
        a.dispatch(0);
        assert_eq!(state(&a, 0).thoughts, 4);
        assert!(state(&a, 0).plan != Some(post) && mine.iter().all(|&i| a.ships[i].sent != Some((900, 500))));
    }

    #[test]
    fn a_match_being_played_forward_does_no_thinking_of_its_own() {
        let mut a = quiet_match();
        a.send_by_looking(0);
        fleet(&mut a, 0, 1, 6, 400, 500);
        a.rehearsal = true;
        a.dispatch(0);
        assert_eq!(state(&a, 0), Dispatch::default());
    }

    #[test]
    fn a_match_with_a_dispatcher_plays_the_same_twice_and_stays_sound() {
        let run = || {
            let mut a = Arena::for_bots(3, [starter_deck(), starter_deck()], [Bot::Ai, Bot::Ai]);
            a.send_by_looking(0);
            for _ in 0..15 * 60 * 3 {
                a.step();
                if a.tick % 15 == 0 {
                    a.audit().unwrap_or_else(|e| panic!("at tick {}: {e}", a.tick));
                }
                if a.over.is_some() {
                    break;
                }
            }
            (a.checksum(), state(&a, 0).looked)
        };
        let first = run();
        assert_eq!(first, run());
        assert!(first.1 > 0, "the dispatcher never played the match forward");
    }
}
