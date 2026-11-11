//! A standing check on the sim's own rules. `audit` looks at a fight or a match as it stands and
//! says what, if anything, is wrong: a ship in two pieces, stats that don't match the pixels, a
//! bank that doesn't add up. The soak tests below run whole arena fights and whole matches (the
//! AI's, and ones driven by random commands that do what no AI would) and audit every tick.
//!
//! Rules about a moment rather than a state are asserted where they happen, in tests only: a pixel
//! growing onto an enemy (`grow_cell`), a move into a bigger enemy (`shift`), a shot left sitting
//! inside a pixel (`shots_step`).
//!
//! Every bug of the kinds found by eye so far (fighters inside the capital, flak past plate,
//! factories through each other, a bank handed to one factory) breaks one of these. A new
//! mechanic that breaks one fails here before anyone has to notice it on screen.

use super::*;

impl Arena {
    /// What is wrong with the state of things, if anything.
    #[cfg_attr(not(test), allow(dead_code))]
    pub fn audit(&self) -> Result<(), String> {
        for (i, s) in self.ships.iter().enumerate() {
            if !s.alive {
                continue;
            }
            let n = s.shape.cells.len();
            let bad = |what: &str| Err(format!("ship {i} ({}, team {}) {what}", s.name, s.team));
            if s.hp.len() != n || s.cool.len() != n {
                return bad("has pixel lists of different lengths");
            }
            if (0..n).any(|c| s.hp[c] > 0 && s.shape.cells[c].is_none()) {
                return bad("has a living pixel where its design has none");
            }
            if !s.has_core() {
                return bad("is alive with no core");
            }
            let (l, t, r, b) = s.bbox();
            if l < 0 || t < 0 || r >= self.w || b >= self.h {
                return bad("is off the map");
            }
            // The quarter-pixels a move leaves over are kept 0 to 3 by everything that moves a
            // ship. A piece shot off takes them along, and out of range they drift it.
            if !(0..4).contains(&s.sx) || !(0..4).contains(&s.sy) {
                return bad("carries a sub-pixel remainder outside 0 to 3");
            }
            // One piece: every living pixel joined to the rest edge to edge.
            if let Some(first) = (0..n).find(|&c| s.hp[c] > 0) {
                let seen = reach(s.shape.w, s.shape.h, first, |k| s.hp[k] > 0);
                if (0..n).any(|c| s.hp[c] > 0 && !seen[c]) {
                    return bad("is in more than one piece");
                }
            }
            let cannons = s.turrets.iter().filter(|t| s.hp[t.cell] > 0).count() as i32;
            if !s.dirty && s.st != stats_of(&s.alive_cells(), cannons) {
                return bad("has stats that don't match its pixels");
            }
            if s.turrets.iter().any(|t| s.shape.cells.get(t.cell) != Some(&Some(Part::Cannon))) {
                return bad("has a turret that isn't on a cannon");
            }
            if s.store < 0 {
                return bad("has a store below zero");
            }
            if s.store > 0 && !s.node {
                return bad("holds pixels but isn't a node");
            }
            if let Some(g) = &s.grow {
                if g.pending.iter().any(|&c| s.hp[c] > 0 || s.shape.cells[c].is_none()) {
                    return bad("has a pixel that's both grown and still to grow");
                }
            }
            if let Some(m) = s.maker {
                if s.grow.is_none() {
                    return bad("is being printed but has nothing left to grow");
                }
                if self.ships[m].alive && self.ships[m].making != Making::Child(i) {
                    return bad("thinks it's being printed by a factory that's printing something else");
                }
            }
            if let Making::Child(c) = s.making {
                if self.ships[c].alive && self.ships[c].maker != Some(i) {
                    return bad("is printing a ship that doesn't know it");
                }
            }
        }
        if let Some(e) = &self.eco {
            for t in 0..2 {
                if self.ships.get(e.king[t]).map_or(true, |s| s.role != Role::King || s.team != t) {
                    return Err(format!("team {t}'s king isn't a king of team {t}"));
                }
                // The books: what's mined and what the king has made, plus the starting bank, is
                // what's held, spent or lost.
                let held: i32 = self.ships.iter().filter(|s| s.team == t).map(|s| s.store).sum();
                let (had, has) = (START_BANK + self.score[t].mined + e.made[t], held + e.spent[t] + e.spilt[t]);
                if had != has {
                    return Err(format!("team {t}'s books are off: {had} px mined, made and started with, {has} held, spent and lost"));
                }
                if e.queue[t].iter().any(|j| j.paid >= j.cost || j.slot >= e.cards[t].len()) {
                    return Err(format!("team {t}'s king has a finished or impossible job queued"));
                }
            }
            for (i, &id) in e.net.iter().enumerate() {
                if id != usize::MAX && self.ships[id].team != self.ships[i].team {
                    return Err(format!("ships {i} and {id} of different sides share a network"));
                }
            }
        }
        // Nothing planted moves, nothing grows onto an enemy pixel or into something of its own
        // side that's standing there, and no enemy ship can move, turn, arrive or be printed into
        // a planted pixel. So a planted thing never shares a pixel with another planted thing,
        // whoever's, nor with any enemy ship that's on the map.
        for (i, p) in self.ships.iter().enumerate() {
            if !p.alive || !p.rooted {
                continue;
            }
            let (pl, pt, pr, pb) = p.bbox();
            for (j, o) in self.ships.iter().enumerate() {
                if j == i || !o.alive || o.warp > 0 || !(o.team != p.team || (o.rooted && j > i)) {
                    continue;
                }
                let (ol, ot, or, ob) = o.bbox();
                if pl > or || ol > pr || pt > ob || ot > pb {
                    continue;
                }
                let shares = (0..p.hp.len()).any(|c| {
                    p.hp[c] > 0 && {
                        let (x, y) = p.cell_xy(c);
                        o.cell_at(x, y).is_some_and(|d| o.hp[d] > 0)
                    }
                });
                if shares && o.rooted {
                    return Err(format!("factories {i} ({}, team {}) and {j} ({}, team {}) share a pixel", p.name, p.team, o.name, o.team));
                }
                if shares {
                    return Err(format!("ship {j} ({}, team {}) got into something planted: {i} ({}, team {})", o.name, o.team, p.name, p.team));
                }
            }
        }
        Ok(())
    }

    /// After shots are resolved, none is left sitting inside a living enemy pixel: it would have
    /// hit it.
    pub(super) fn audit_shots(&self) -> Result<(), String> {
        for sh in &self.shots {
            let (x, y) = (sh.line.x, sh.line.y);
            for (i, s) in self.ships.iter().enumerate() {
                if s.alive && s.team != sh.team && s.warp == 0 && s.cell_at(x, y).is_some_and(|c| s.hp[c] > 0) {
                    return Err(format!("a {:?} shot of team {} is inside a living pixel of ship {i} ({}) at ({x},{y})", sh.kind, sh.team, s.name));
                }
            }
        }
        Ok(())
    }

    /// No shot that flew on this tick passed through a pixel that is still alive. Every hit either
    /// stops the shot or kills the pixel, so a living pixel on the path of a shot that flew on is
    /// a pixel the shot skipped. Checked against ships that held still this tick (factories,
    /// parked ships, kings at rest): past a moving ship the path is worked out in the frame of
    /// the ship, which this plain check does not redo.
    pub(super) fn audit_paths(&self, flown: &[(Vec<(i32, i32)>, usize, u32)]) -> Result<(), String> {
        for (path, team, born) in flown {
            let (pl, pr) = (path.iter().map(|p| p.0).min().unwrap(), path.iter().map(|p| p.0).max().unwrap());
            let (pt, pb) = (path.iter().map(|p| p.1).min().unwrap(), path.iter().map(|p| p.1).max().unwrap());
            for (i, s) in self.ships.iter().enumerate() {
                if !s.alive || s.team == *team || s.warp > 0 || (*born != self.tick && s.pose0 != (s.x, s.y, s.facing)) {
                    continue;
                }
                let (l, t, r, b) = s.bbox();
                if pl > r || pr < l || pt > b || pb < t {
                    continue;
                }
                if let Some(&(x, y)) = path.iter().find(|&&(x, y)| s.cell_at(x, y).is_some_and(|c| s.hp[c] > 0)) {
                    return Err(format!("a shot of team {team} flew through a living pixel of ship {i} ({}) at ({x},{y})", s.name));
                }
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Runs a fight for up to `ticks`, auditing every tick. `drive` gets to give commands first.
    /// Also checks that nothing comes out of hyperspace inside an enemy, and that nothing without
    /// thrust (a factory, a hulk) changes position. Returns the longest any ship that can fly
    /// spent inside a bigger enemy, in ticks.
    fn soak(a: &mut Arena, ticks: u32, mut drive: impl FnMut(&mut Arena)) -> u32 {
        let mut run: Vec<u32> = Vec::new();
        let mut warp = Vec::new();
        let mut was: Vec<Option<(i32, i32)>> = Vec::new();
        let mut longest = 0;
        for _ in 0..ticks {
            if a.over.is_some() {
                break;
            }
            drive(a);
            a.step();
            if let Err(what) = a.audit() {
                panic!("tick {}: {what}", a.tick);
            }
            run.resize(a.ships.len(), 0);
            warp.resize(a.ships.len(), 0);
            was.resize(a.ships.len(), None);
            for i in 0..a.ships.len() {
                let s = &a.ships[i];
                let still = s.alive && s.role == Role::Ship && s.warp == 0 && s.maker.is_none() && (s.grow.is_some() || s.st.thrust == 0);
                if let (true, Some(at)) = (still, was[i]) {
                    assert_eq!((s.x, s.y), at, "tick {}: {} has no thrust but moved", a.tick, s.name);
                }
                was[i] = if still { Some((s.x, s.y)) } else { None };
            }
            for i in 0..a.ships.len() {
                let s = &a.ships[i];
                if s.alive && warp[i] > 0 && s.warp == 0 {
                    assert!(!a.in_enemy(i), "tick {}: {} came out of hyperspace inside an enemy", a.tick, s.name);
                }
                warp[i] = s.warp;
                let flies = s.alive && s.warp == 0 && s.grow.is_none() && s.st.thrust > 0;
                if flies && a.overlap(i, s.x, s.y, s.facing).0 > 0 {
                    run[i] += 1;
                    longest = longest.max(run[i]);
                } else {
                    run[i] = 0;
                }
            }
        }
        longest
    }

    #[test]
    fn every_arena_test_stays_sound() {
        // The fights made for the starter deck, and the proving ground's own fleet and fights.
        let mut longest = 0;
        for (lib, fights) in [(starter_deck(), crate::pg::tests()), (defaults(), crate::pg::reference_tests())] {
            for t in fights.iter().filter(|t| !t.place.is_empty()) {
                let mut a = Arena::new();
                for &(team, slot, x, y) in &t.place {
                    a.add(team, slot, &lib[slot], x, y);
                }
                longest = longest.max(soak(&mut a, TIME_LIMIT, |_| {}));
            }
        }
        println!("arena tests: longest stay inside a bigger enemy {longest} ticks");
        // The longest stays come from a big ship turning: a titan's quarter turn swings its whole
        // hull round in one tick and can leave a lancer 12 px deep, which then takes 6 to 9 ticks
        // to get out at 2 px a tick.
        assert!(longest <= 10, "a ship that could fly spent {longest} ticks inside a bigger enemy");
    }

    #[test]
    fn an_ai_match_stays_sound() {
        let mut a = Arena::for_match(5, [starter_deck(), starter_deck()], [true, true]);
        let longest = soak(&mut a, MATCH_LIMIT, |_| {});
        println!("AI match: over {:?} at tick {}, longest stay inside a bigger enemy {longest} ticks", a.over, a.tick);
        assert!(a.over.is_some());
        assert!(longest <= 10, "a ship that could fly spent {longest} ticks inside a bigger enemy");
    }

    /// A carrier: a built ship with factory pixels and engines, printing fighters as it flies.
    fn carrier() -> Design {
        let mut d = Design::from_rows("CARRIER", Order::Hunt, &[".hhh.", "hfffh", "hfcfh", "hfffh", ".e.e."]);
        d.build = vec![0];
        d
    }

    /// A small factory with no guns, so a seed planted beside an enemy one lives to grow.
    fn yard() -> Design {
        let mut d = Design::from_rows("YARD", Order::Hold, &["hhhhhhh", "hfffffh", "hfhhhfh", "hfhchfh", "hfhhhfh", "hfffffh", "hhhhhhh"]);
        d.seeded = true;
        d.build = vec![0];
        d
    }

    #[test]
    fn matches_driven_by_random_commands_stay_sound() {
        // Both sides do what no AI would. Their kings charge each other and sit on top of one
        // another, so what they print arrives in a crowd. They plant factory after factory, half
        // of them on the same three rocks in the middle of the map, so the two sides grow up
        // against each other. They print anything, a carrier included. Half the matches swap the
        // outpost for a yard with no guns. Each side is handed two pixels a tick (counted as
        // mined, so the books still have to balance) to keep the kings printing. Four matches of
        // five minutes, audited every tick. The test also counts how often the rules had
        // something to stop, and fails if they never did.
        let mut longest = 0;
        let (mut planted, mut printed, mut held, mut walled, mut pressed) = (0, 0, 0, 0, 0);
        for seed in 1..=4u64 {
            let mut decks = [starter_deck(), starter_deck()];
            decks[(seed % 2) as usize][1] = carrier();
            if seed > 2 {
                decks[0][4] = yard();
                decks[1][4] = yard();
            }
            let mut a = Arena::for_match(seed, decks, [false, false]);
            let mut rng = Rng::new(seed * 7919);
            let charge = if seed % 2 == 0 { 80 } else { 30 };
            let hot: Vec<(i32, i32)> = [(600, 450), (450, 300), (750, 600)].iter().filter_map(|&p| a.rock_near(p, 300)).collect();
            let run = soak(&mut a, 15 * 300, |a| {
                // How often the rules under test had something to stop.
                let smallest = |team: usize| a.ships.iter().filter(|p| p.alive && p.rooted && p.team != team).map(|p| p.px_full()).min().unwrap_or(i32::MAX);
                let smallest = [smallest(0), smallest(1)];
                for i in 0..a.ships.len() {
                    let s = &a.ships[i];
                    // A ship at least as big as something the enemy planted, right up against it:
                    // before planted things were solid, it would have flown over.
                    if s.alive && s.warp == 0 && s.maker.is_none() && (s.st.thrust > 0 || s.role == Role::King) && s.px_full() >= smallest[s.team] {
                        let big = s.px_full();
                        if [(1, 0), (-1, 0), (0, 1), (0, -1)].iter().any(|&(dx, dy)| a.clash(i, s.x + dx, s.y + dy, s.facing, space::Solid::Planted).1.is_some_and(|p| a.ships[p].px_full() <= big)) {
                            pressed += 1;
                        }
                    }
                    if s.alive && s.warp == 1 && a.in_enemy(i) {
                        held += 1;
                    }
                    if s.alive && s.warp == 0 && s.grow.is_some() && a.next_cell(i) == Next::Blocked {
                        walled += 1;
                    }
                }
                for t in 0..2 {
                    let k = a.eco.as_ref().unwrap().king[t];
                    if a.ships[k].alive {
                        a.ships[k].store += 2;
                        a.score[t].mined += 2;
                    }
                }
                // Every half minute both sides send a factory seed to the same rock.
                if a.tick % 450 == 0 && !hot.is_empty() {
                    let target = a.rock_near(hot[(a.tick / 450) as usize % hot.len()], 30);
                    for team in 0..2 {
                        a.command(Cmd::Print { team, slot: 4, target });
                    }
                }
                let team = rng.below(2) as usize;
                let e = a.eco.as_ref().unwrap();
                let (king, foe, queued) = (e.king[team], e.king[1 - team], e.queue[team].len());
                if rng.chance(5) {
                    let (mut dx, mut dy) = (rng.range(-1, 1), rng.range(-1, 1));
                    if rng.chance(charge) {
                        dx = (a.ships[foe].x - a.ships[king].x).signum();
                        dy = (a.ships[foe].y - a.ships[king].y).signum();
                    }
                    a.command(Cmd::Steer { team, dx, dy });
                }
                // Orders for any design, sensible or not; repeats; and cancelled prints.
                if rng.chance(3) {
                    a.command(Cmd::Order { team, slot: rng.below(7) as usize, order: ORDERS[rng.below(ORDERS.len() as u32) as usize] });
                }
                if rng.chance(1) {
                    a.command(Cmd::Repeat { team, slot: rng.below(7) as usize });
                }
                if rng.chance(1) {
                    // Mostly a job that's there; now and then one the queue has moved up from.
                    let at = rng.below(4) as usize;
                    let slot = a.eco.as_ref().unwrap().queue[team].get(at).map_or(rng.below(7) as usize, |j| j.slot);
                    a.command(Cmd::Cancel { team, at, slot });
                }
                if rng.chance(6) && queued < 4 {
                    // Mostly factories, and miners to pay for them.
                    let slot = match rng.below(100) {
                        0..=44 => 4 + rng.below(2) as usize,
                        45..=64 => 3,
                        _ => rng.below(7) as usize,
                    };
                    // A seed goes to one of the rocks both sides fight over, to a rock beside a
                    // factory of either side, or to any rock at all.
                    let mut target = None;
                    for _ in 0..20 {
                        let p = a.rock_px[rng.below(a.rock_px.len() as u32) as usize];
                        if a.rock_at(p.0, p.1) {
                            target = Some(p);
                            break;
                        }
                    }
                    let pick = rng.below(100);
                    if pick < 50 && !hot.is_empty() {
                        let h = hot[rng.below(hot.len() as u32) as usize];
                        target = a.rock_near((h.0 + rng.range(-8, 8), h.1 + rng.range(-8, 8)), 30).or(target);
                    } else if pick < 75 {
                        let nodes: Vec<(i32, i32)> = a.ships.iter().filter(|s| s.alive && s.node && s.role == Role::Ship).map(|s| (s.x, s.y)).collect();
                        if !nodes.is_empty() {
                            target = a.rock_near(nodes[rng.below(nodes.len() as u32) as usize], 40).or(target);
                        }
                    }
                    let seeded = a.eco.as_ref().unwrap().cards[team][slot].seeded;
                    a.command(Cmd::Print { team, slot, target: if seeded { target } else { None } });
                }
            });
            longest = longest.max(run);
            let e = a.eco.as_ref().unwrap();
            planted += e.tally[0].planted + e.tally[1].planted;
            printed += e.tally[0].printed + e.tally[1].printed;
            println!("fuzzed match {seed}: tick {}, over {:?}, {} ships", a.tick, a.over, a.ships.iter().filter(|s| s.alive).count());
        }
        println!("fuzzed matches: {planted} factories planted, {printed} ships printed; arrivals held back for {held} ticks, growth walled off for {walled}, ships stopped at a smaller planted enemy for {pressed}; longest stay inside a bigger enemy {longest} ticks");
        assert!(planted >= 20 && printed >= 20, "the random commands barely did anything: {planted} planted, {printed} printed");
        assert!(held > 0, "nothing ever tried to arrive inside an enemy, so that rule went untested");
        assert!(walled > 0, "no factory ever grew up against an enemy, so that rule went untested");
        assert!(pressed > 0, "no ship ever ran up against a planted enemy smaller than itself, so that rule went untested");
        // A ship leaves a bigger enemy at 2 px a tick. Since 2026-11-11 a king flies at 2.5, not
        // 3, so one that comes over a small ship going its way takes longer to pass: 15 ticks
        // seen, along the edge of the map.
        assert!(longest <= 20, "a ship that could fly spent {longest} ticks inside a bigger enemy");
    }
}
