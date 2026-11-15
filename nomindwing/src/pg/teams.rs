//! The ready-made teams: the starter deck, and five teams drawn for a strategy each. The AI plays
//! any of them against you (pick one in the match menu) or against itself (`--teams`), and you can
//! load any of them from the library to play it or change it. Each team's factories build from its
//! own slots, and every card spends all the power its hull makes.
//!
//! A strategy is the AI's (`Strategy`): a person playing one of these teams plays it however they like.
//! The new drawings are of 2026-11-14; the other cards are the starter deck's and the part examples.

use super::design::*;
use super::sim::{Strategy, AH, AW};
use super::{block, Sides, Test};

pub struct Ready {
    pub name: &'static str,
    /// What the AI does with it, in a sentence or two, for the match menu.
    pub about: &'static str,
    pub designs: Vec<Design>,
    /// How the AI plays it. None: as the AI always has, which is the starter deck's way.
    pub strategy: Option<Strategy>,
}

/// A card with a build order of its own for this team.
fn builds(mut d: Design, build: &[usize]) -> Design {
    d.build = build.to_vec();
    d
}

fn example(name: &str) -> Design {
    part_examples().into_iter().find(|d| d.name == name).unwrap_or_else(|| panic!("no example called {name}"))
}

/// The fastest ship a drawing can make, with a three-pixel repeater: 84 px of reach and two shots
/// a second, and nothing but hull round its core.
pub fn wasp() -> Design {
    drawn("WASP", (48, 46), &[".r.", ".r.", "hrh", "hch", "ehe", ".e."], None)
}

/// The cheapest factory worth planting: twelve factory pixels round the core and a flak gun on each
/// side, with no plate. It builds anything up to 120 px, a fighter in 20 s.
pub fn nest(build: &[usize]) -> Design {
    drawn("NEST", (46, 46), &[".hhghh.", ".hhfhh.", "hhfffhh", "gffcffg", "hhfffhh", ".hhfhh.", ".hhghh."], Some(build))
}

/// Three lances in a column behind a plated nose, two flak guns and four engines.
pub fn corvette() -> Design {
    drawn("CORVETTE", (47, 46), &["..p..", ".plp.", "hhlhh", "hhchh", "ghlhg", "eehee"], None)
}

/// A battle-line ship: seventeen lances in five columns down a plated hull, so a broadside isn't
/// needed: it points at what it's fighting and every column fires.
pub fn cruiser() -> Design {
    drawn(
        "CRUISER",
        (43, 42),
        &[
            "......p......",
            ".....plp.....",
            "....phlhp....",
            "...phhlhhp...",
            "..phlhlhlhp..",
            "..phlhlhlhp..",
            ".phlhhlhhlhp.",
            ".phlhhlhhlhp.",
            "gphlhhhhhlhpg",
            ".phhhhchhhhp.",
            "gphhhhhhhhhpg",
            ".phhhhhhhhhp.",
            ".ppppphppppp.",
            "..eeeeeeeee..",
        ],
        None,
    )
}

/// The step between a frigate and a capital: five cannons, six lances in the prow and on the
/// flanks, ten flak, and engines all across the stern so it keeps up with a frigate.
pub fn battleship() -> Design {
    drawn(
        "BATTLESHIP",
        (41, 41),
        &[
            "........p........",
            ".......plp.......",
            "......phlhp......",
            ".....phhlhhp.....",
            "....pnnnlnnnp....",
            "...ppnnngnnnpp...",
            "..phhnnnhnnnhhp..",
            ".gplhhhhphhhhlpg.",
            ".phnnnhhghhnnnhp.",
            ".phnnnhhghhnnnhp.",
            "gphnnnhhchhnnnhpg",
            ".phhhhhnnnhhhhhp.",
            ".phhhhhnnnhhhhhp.",
            "gphhhhhnnnhhhhhpg",
            ".phhhhhhghhhhhhp.",
            ".eeeeeeeeeeeeeee.",
        ],
        None,
    )
}

/// A mid-sized factory between the outpost and the shipyard: two rings of factory pixels and a
/// cross, 72 in all, inside a thin plate. It builds anything up to 720 px.
pub fn dock(build: &[usize]) -> Design {
    drawn(
        "DOCK",
        (40, 40),
        &[
            ".......ppppp.......",
            ".....pphhhhhpp.....",
            "....phhhhghhhhp....",
            "...phhhhhhhhhhhp...",
            "..phfffffffffffhp..",
            ".phhfhhhhfhhhhfhhp.",
            ".phhfhfffffffhfhhp.",
            "phhhfhfhhhhhfhfhhhp",
            "phhhfhfhhfhhfhfhhhp",
            "pghhffffhchffffhhgp",
            "phhhfhfhhfhhfhfhhhp",
            "phhhfhfhhhhhfhfhhhp",
            ".phhfhfffffffhfhhp.",
            ".phhfhhhhfhhhhfhhp.",
            "..phfffffffffffhp..",
            "...phhhhhhhhhhhp...",
            "....phhhhghhhhp....",
            ".....pphhhhhpp.....",
            ".......ppppp.......",
        ],
        Some(build),
    )
}

/// A siege ship: two fourteen-pixel beams down its length reach 208 px, further than any other
/// gun, and cut 56 pixels each. Slow, and plated all round.
pub fn lancet() -> Design {
    drawn(
        "LANCET",
        (43, 40),
        &[
            "...b.....b...",
            "..pbp...pbp..",
            "..hbh...hbh..",
            ".phbhp.phbhp.",
            ".phbhhphhbhp.",
            ".phbhhhhhbhp.",
            "gphbhhhhhbhpg",
            ".phbhhhhhbhp.",
            ".phbhhhhhbhp.",
            ".phbhhhhhbhp.",
            "gphbhhchhbhpg",
            ".phbhhhhhbhp.",
            ".phbhhhhhbhp.",
            ".phbhhhhhbhp.",
            "gphhhhhhhhhpg",
            ".phhhhhhhhhp.",
            "..pppphpppp..",
            "..eeeeeeeee..",
        ],
        None,
    )
}

/// Every ready-made team with a strategy, the starter deck first. The match menu offers them in
/// this order, and `--teams` plays each against each.
pub fn ready_teams() -> Vec<Ready> {
    let s = starter_deck();
    let card = |i: usize| s[i].clone();
    let (fighter, lancer, frigate, miner, outpost, shipyard, capital) = (card(0), card(1), card(2), card(3), card(4), card(5), card(6));
    vec![
        Ready { name: "STARTER DECK", about: "TWO OUTPOSTS, THEN SHIPYARDS. IT PRINTS WHAT BEATS YOUR FLEET AND ATTACKS WHEN IT HAS THE EDGE.", designs: s.clone(), strategy: None },
        // A rush. Slots: 0 fighter, 1 lancer, 2 miner, 3 nest, 4 outpost.
        Ready {
            name: "SWARM",
            about: "A RUSH: CHEAP NESTS AND A KING PRINTING FIGHTERS AND LANCERS, AN ATTACK AT 2:30. THEN IT BOOMS.",
            designs: vec![fighter.clone(), lancer, miner.clone(), nest(&[2, 0, 1, 0, 1]), builds(outpost.clone(), &[2, 2, 0, 1, 0, 1])],
            strategy: Some(Strategy { plant: &[(3, 0), (3, 100), (3, 200), (4, 330)], most: 6, lean: 150, prints: &[0, 1], keep: 0, first: 150, wave: 54, bold: true, raid: &[] }),
        },
        // A timing attack. Slots: 0 fighter, 1 corvette, 2 cruiser, 3 miner, 4 outpost, 5 dock.
        Ready {
            name: "LANCE LINE",
            about: "A TIMING ATTACK. OUTPOSTS AND A DOCK MASS CORVETTES AND CRUISERS FOR ONE BIG PUSH AT 7:00.",
            designs: vec![fighter.clone(), corvette(), cruiser(), miner.clone(), builds(outpost.clone(), &[3, 3, 0, 1, 1, 2]), dock(&[3, 2, 1, 2])],
            strategy: Some(Strategy { plant: &[(4, 0), (4, 0), (5, 0), (4, 0)], most: 5, lean: 0, prints: &[1], keep: 60, first: 420, wave: 400, bold: true, raid: &[] }),
        },
        // A boom into big ships. Slots: 0 fighter, 1 frigate, 2 miner, 3 outpost, 4 shipyard,
        // 5 battleship, 6 capital.
        Ready {
            name: "BIG GUNS",
            about: "A BOOM. FOUR OUTPOSTS AND MANY MINERS, THEN BATTLESHIPS AND CAPITALS FROM A SHIPYARD. IT ATTACKS LATE.",
            designs: vec![fighter, frigate, miner.clone(), builds(outpost.clone(), &[2, 2, 2, 0, 0, 1]), builds(shipyard.clone(), &[2, 5, 6, 1]), battleship(), capital],
            strategy: Some(Strategy { plant: &[(3, 0), (3, 0), (3, 0), (3, 0), (4, 0), (3, 0)], most: 7, lean: 0, prints: &[1, 0, 0], keep: 120, first: 720, wave: 900, bold: false, raid: &[] }),
        },
        // Harassment. Slots: 0 wasp, 1 archer, 2 raker, 3 miner, 4 outpost.
        Ready {
            name: "RAIDERS",
            about: "HARASSMENT. FAST WASPS GO FOR YOUR MINERS AND FACTORIES FROM THE START. ARCHERS AND RAKERS FOLLOW.",
            designs: vec![wasp(), example("ARCHER"), example("RAKER"), miner.clone(), builds(outpost.clone(), &[3, 3, 0, 1, 0, 2])],
            strategy: Some(Strategy { plant: &[(4, 0)], most: 5, lean: 0, prints: &[0, 0, 1, 2], keep: 40, first: 300, wave: 150, bold: false, raid: &[0] }),
        },
        // A turtle. Slots: 0 beamer, 1 warden, 2 aegis, 3 miner, 4 outpost, 5 shipyard, 6 lancet.
        Ready {
            name: "SIEGE",
            about: "A TURTLE. BEAMERS AND WARDENS HOLD HOME, THEN LANCETS THAT OUTRANGE EVERYTHING PUSH SLOWLY.",
            designs: vec![example("BEAMER"), example("WARDEN"), example("AEGIS"), miner, builds(outpost, &[3, 3, 1, 0]), builds(shipyard, &[3, 6, 2, 1, 0]), lancet()],
            strategy: Some(Strategy { plant: &[(4, 0), (4, 0), (5, 0), (4, 0)], most: 5, lean: 0, prints: &[1, 0], keep: 120, first: 540, wave: 700, bold: false, raid: &[] }),
        },
    ]
}

/// Each team's army as it might be in the middle of a match, at about 700 px a side, in the
/// shape its strategy builds: so many of each of its cards, by name. Miners and factories stay
/// home.
fn army(team: &str) -> Vec<(&'static str, i32)> {
    match team {
        "STARTER DECK" => vec![("FIGHTER", 12), ("LANCER", 12), ("FRIGATE", 9)],
        "SWARM" => vec![("FIGHTER", 40), ("LANCER", 39)],
        "LANCE LINE" => vec![("FIGHTER", 6), ("CORVETTE", 12), ("CRUISER", 3)],
        "BIG GUNS" => vec![("FIGHTER", 6), ("FRIGATE", 5), ("BATTLESHIP", 2)],
        "RAIDERS" => vec![("WASP", 16), ("ARCHER", 13), ("RAKER", 9)],
        "SIEGE" => vec![("LANCET", 2), ("BEAMER", 3), ("WARDEN", 1), ("AEGIS", 1)],
        _ => panic!("no army for {team}"),
    }
}

/// A fight between two ready-made teams: so many of each of blue's cards against so many of
/// red's, by name. Each side's slots are its whole team. Each side stands in a column of blocks,
/// a card to a block, as many columns as it takes to stay on the field.
fn fight(name: &'static str, blue: (&'static str, &[(&str, i32)]), red: (&'static str, &[(&str, i32)])) -> Test {
    let all = ready_teams();
    let team = |n: &str| team_of(all.iter().find(|t| t.name == n).unwrap_or_else(|| panic!("no team {n}")).designs.clone());
    let sides = Sides { names: [blue.0, red.0], teams: [team(blue.0), team(red.0)] };
    let mut place = Vec::new();
    for (t, (_, ships)) in [blue, red].into_iter().enumerate() {
        // Each card's block: its slot, how many, how far apart, and its size.
        let blocks: Vec<(usize, i32, i32, i32, i32)> = ships
            .iter()
            .map(|&(card, n)| {
                let slot = sides.teams[t].iter().position(|d| d.name == card).unwrap_or_else(|| panic!("{} has no {card}", sides.names[t]));
                let sh = sides.teams[t][slot].shape();
                let gap = sh.w.max(sh.h) + 4;
                let rows = n.clamp(1, 9);
                (slot, n, gap, (n + rows - 1) / rows * gap, rows * gap)
            })
            .collect();
        // Columns of blocks no taller than the field allows, the first nearest the enemy.
        let mut columns: Vec<Vec<(usize, i32, i32, i32, i32)>> = vec![Vec::new()];
        for b in blocks {
            let high: i32 = columns.last().unwrap().iter().map(|c| c.4 + 8).sum();
            if high + b.4 > AH - 40 && !columns.last().unwrap().is_empty() {
                columns.push(Vec::new());
            }
            columns.last_mut().unwrap().push(b);
        }
        let mut x = 150;
        for column in &columns {
            let wide = column.iter().map(|c| c.3).max().unwrap_or(0);
            let high: i32 = column.iter().map(|c| c.4 + 8).sum::<i32>() - 8;
            let mut y = AH / 2 - high / 2;
            for &(slot, n, gap, _, h) in column {
                let cx = if t == 0 { x } else { AW - x };
                place.extend(block(t, slot, n, cx, y + h / 2, gap));
                y += h + 8;
            }
            x -= wide + 12;
        }
    }
    Test { name, place, ships: Some(sides) }
}

/// The ready-made teams' fights, after the arena's first (`arena_fights`). For each team with a
/// strategy, its army against the starter deck's, and a fight or two for what its new drawings
/// are for; then every two of the five teams' armies against each other. Blue is the team being
/// shown, so its whole team is what the arena lists. Armies are about 700 px a side, and the
/// other fights about level on cost too.
pub fn team_fights() -> Vec<Test> {
    let (start, swarm, line, guns, raid, siege) = ("STARTER DECK", "SWARM", "LANCE LINE", "BIG GUNS", "RAIDERS", "SIEGE");
    let armies = |a: &'static str, b: &'static str, name: &'static str| fight(name, (a, &army(a)), (b, &army(b)));
    let mut fights = vec![
        armies(swarm, start, "SWARM VS STARTER DECK"),
        fight("5 LANCERS VS 1 NEST", (swarm, &[("LANCER", 5)]), (swarm, &[("NEST", 1)])),
        armies(line, start, "LANCE LINE VS STARTER DECK"),
        fight("6 CORVETTES VS 16 LANCERS", (line, &[("CORVETTE", 6)]), (start, &[("LANCER", 16)])),
        fight("1 CRUISER VS 2 FRIGATES", (line, &[("CRUISER", 1)]), (start, &[("FRIGATE", 2)])),
        fight("2 CRUISERS VS 1 DOCK", (line, &[("CRUISER", 2)]), (line, &[("DOCK", 1)])),
        armies(guns, start, "BIG GUNS VS STARTER DECK"),
        fight("1 BATTLESHIP VS 21 LANCERS", (guns, &[("BATTLESHIP", 1)]), (start, &[("LANCER", 21)])),
        armies(raid, start, "RAIDERS VS STARTER DECK"),
        fight("8 WASPS VS 10 MINERS", (raid, &[("WASP", 8)]), (start, &[("MINER", 10)])),
        fight("8 WASPS VS 10 FIGHTERS", (raid, &[("WASP", 8)]), (start, &[("FIGHTER", 10)])),
        armies(siege, start, "SIEGE VS STARTER DECK"),
        fight("1 LANCET VS 1 BATTLESHIP", (siege, &[("LANCET", 1)]), (guns, &[("BATTLESHIP", 1)])),
        fight("2 LANCETS VS 40 FIGHTERS", (siege, &[("LANCET", 2)]), (swarm, &[("FIGHTER", 40)])),
    ];
    for (a, b, name) in [
        (swarm, line, "SWARM VS LANCE LINE"),
        (swarm, guns, "SWARM VS BIG GUNS"),
        (swarm, raid, "SWARM VS RAIDERS"),
        (swarm, siege, "SWARM VS SIEGE"),
        (line, guns, "LANCE LINE VS BIG GUNS"),
        (line, raid, "LANCE LINE VS RAIDERS"),
        (line, siege, "LANCE LINE VS SIEGE"),
        (guns, raid, "BIG GUNS VS RAIDERS"),
        (guns, siege, "BIG GUNS VS SIEGE"),
        (raid, siege, "RAIDERS VS SIEGE"),
    ] {
        fights.push(armies(a, b, name));
    }
    fights
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pg::sim::{Arena, Card, START_BANK};

    #[test]
    fn every_ready_team_is_a_team_its_ai_can_play() {
        let teams = ready_teams();
        assert_eq!(teams[0].designs.iter().map(|d| d.to_line()).collect::<Vec<_>>(), starter_deck().iter().map(|d| d.to_line()).collect::<Vec<_>>(), "the first is the starter deck");
        assert!(teams[0].strategy.is_none() && teams[1..].iter().all(|t| t.strategy.is_some()));
        for t in &teams {
            assert!(t.name.len() <= 12 && !t.about.is_empty(), "{}", t.name);
            assert!(t.designs.len() <= SLOTS, "{} has {} cards", t.name, t.designs.len());
            let cards: Vec<Card> = t.designs.iter().map(Card::of).collect();
            for (d, c) in t.designs.iter().zip(&cards) {
                let st = d.stats();
                assert!(c.ok, "{} {}: {:?}", t.name, d.name, d.check().problem());
                // Every card spends its power: none short of it, and none with more than a
                // fiftieth to spare (the starter capital has 5 of 264).
                assert!(st.used <= st.made && (st.made - st.used) * 50 <= st.made, "{} {}: power {}/{}", t.name, d.name, st.made, st.used);
                // A factory builds from this team's slots, only what it can build.
                for &b in &d.build {
                    assert!(b < t.designs.len() && !t.designs[b].seeded && t.designs[b].stats().count <= st.max_build(), "{} {} can't build slot {b}", t.name, d.name);
                }
            }
            // It can mine, and the bank it starts with buys a factory and a miner.
            let miner = cards.iter().filter(|c| c.miner()).map(|c| c.cost).min().unwrap_or_else(|| panic!("{} has no miner", t.name));
            let factory = cards.iter().filter(|c| c.factory()).map(|c| c.cost).min().unwrap_or_else(|| panic!("{} has no factory", t.name));
            assert!(factory + miner <= START_BANK, "{}: a factory and a miner cost {}", t.name, factory + miner);
            assert!(t.designs.iter().any(|d| d.build.iter().any(|&b| cards[b].miner())), "{}: no factory builds a miner", t.name);
            if let Some(p) = t.strategy {
                assert!(p.plant.iter().all(|&(s, _)| cards.get(s).is_some_and(|c| c.factory())), "{} plants something that isn't a factory", t.name);
                assert!(p.prints.iter().all(|&s| cards.get(s).is_some_and(|c| c.warship())), "{}'s king prints something that isn't a warship", t.name);
                assert!(p.raid.iter().all(|&s| cards.get(s).is_some_and(|c| c.stanced())), "{} raids with something that doesn't fly", t.name);
                assert!(p.plant.windows(2).all(|w| w[0].1 <= w[1].1) && p.most >= p.plant.len().min(p.most));
            }
        }
    }

    /// Each strategy against the AI as it always was: the match ends, replays exactly from the same
    /// seed, and the strategy's first wave doesn't go before its time (its raiders may).
    #[test]
    fn every_strategy_plays_a_match_and_replays_it() {
        let teams = ready_teams();
        for t in &teams[1..] {
            let p = t.strategy.unwrap();
            let run = || {
                let mut a = Arena::for_match(3, [t.designs.clone(), starter_deck()], [true, true]);
                a.eco.as_mut().unwrap().strategy[0] = t.strategy;
                while a.over.is_none() {
                    a.step();
                    // Raiders stay on SAB whatever the waves and raids are doing.
                    let e = a.eco.as_ref().unwrap();
                    assert!(a.tick < 30 || p.raid.iter().all(|&s| e.orders[0][s] == Order::Sab), "{}'s raiders came off SAB at {}", t.name, a.tick);
                }
                let tally = a.eco.as_ref().unwrap().tally[0];
                (a.tick, a.over, tally.first_wave, tally.planted)
            };
            let (tick, over, wave, planted) = run();
            assert_eq!(run(), (tick, over, wave, planted), "{} didn't replay", t.name);
            assert!(tick < crate::pg::sim::MATCH_LIMIT, "{} drew on time", t.name);
            assert!(planted >= 1, "{} planted nothing", t.name);
            assert!(wave == 0 || wave >= p.first * 15, "{}'s first wave went at {wave}, before {} s", t.name, p.first);
        }
    }
}

#[cfg(test)]
mod fight_tests {
    use super::*;
    use crate::framebuffer::Framebuffer;
    use crate::pg::sim::Arena;
    use crate::app::App;
    use crate::pg::{arena_fights, Pg, Screen, FIELD, YOUR_TEAM};

    #[test]
    fn every_team_fight_stands_on_the_field_level_on_cost_and_says_what_it_is() {
        let teams: Vec<&str> = ready_teams().iter().map(|t| t.name).collect();
        let fights = team_fights();
        assert!(fights.len() >= 20);
        for t in &fights {
            let sides = t.ships.as_ref().expect("every team fight brings its two teams");
            let lib = [&sides.teams[0][..], &sides.teams[1][..]];
            // Each side's slots are its whole team.
            for k in 0..2 {
                let whole = team_of(ready_teams().into_iter().find(|r| r.name == sides.names[k]).unwrap().designs);
                assert_eq!(lib[k].iter().map(|d| d.to_line()).collect::<Vec<_>>(), whole.iter().map(|d| d.to_line()).collect::<Vec<_>>(), "{}: {}", t.name, sides.names[k]);
            }
            let mut a = Arena::new();
            let mut cost = [0; 2];
            for &(team, slot, x, y) in &t.place {
                let d = &lib[team][slot];
                assert!(a.add(team, slot, d, x, y), "{}: a {} wasn't placed", t.name, d.name);
                let half = d.shape().w.max(d.shape().h) / 2;
                assert!(x - half >= 0 && x + half < AW && y - half >= 0 && y + half < AH, "{} places a {} off the arena at {x},{y}", t.name, d.name);
                cost[team] += d.stats().cost;
            }
            // Level on cost: neither side a fifth dearer than the other.
            assert!(cost[0] * 5 <= cost[1] * 6 && cost[1] * 5 <= cost[0] * 6, "{} costs {} against {}", t.name, cost[0], cost[1]);
            // Named for the two teams in it, or for what it places.
            let (blue, red) = t.name.split_once(" VS ").unwrap();
            if blue == sides.names[0] && red == sides.names[1] {
                assert!(teams.contains(&blue) && teams.contains(&red));
            } else {
                assert_eq!(t.counted(lib), t.name);
            }
            assert!(4 + Framebuffer::text_width(&t.title(lib[0]), 1) <= FIELD.w - 40, "{}", t.name);
        }
        // Every new drawing is in a fight, and every team's army; and blue is each new team in
        // a fight of its own, so its whole team is seen.
        for name in ["WASP", "NEST", "CORVETTE", "CRUISER", "BATTLESHIP", "DOCK", "LANCET"] {
            assert!(fights.iter().any(|t| t.place.iter().any(|p| t.ships.as_ref().unwrap().teams[p.0][p.1].name == name)), "no fight has a {name}");
        }
        for name in &teams[1..] {
            assert!(fights.iter().any(|t| t.name == format!("{name} VS STARTER DECK") && t.ships.as_ref().unwrap().names[0] == *name), "no fight has {name}'s army against the starter deck's");
        }
    }

    #[test]
    fn the_first_fight_is_your_team_and_the_rest_leave_it_alone() {
        let dir = std::env::temp_dir().join(format!("nmw_team_fights_{}", std::process::id()));
        let _ = std::fs::create_dir_all(&dir);
        let save = dir.join("proving_ground_designs.txt");
        let mut pg = Pg::new(Some(save.clone()), None);
        pg.help = false;
        // A team of yours that isn't the starter deck.
        pg.lib[8] = wasp();
        let mine: Vec<String> = pg.lib.iter().map(|d| d.to_line()).collect();
        let lines = |l: &[Design]| l.iter().map(|d| d.to_line()).collect::<Vec<_>>();
        // The game opens on the first: an open field, your team in the slots.
        assert_eq!((pg.screen, pg.test, pg.arena.ships.len()), (Screen::Arena, Some(0), 0));
        assert_eq!(arena_fights()[0].name, YOUR_TEAM);
        assert_eq!(lines(pg.side(0)), mine);
        let fights = arena_fights();
        for t in 1..fights.len() {
            pg.key(crate::app::Key::Letter('N'));
            assert_eq!(pg.test, Some(t));
            let sides = fights[t].ships.as_ref().unwrap();
            // Each side's slots are its team's; the list shows blue's, and red's once you place
            // for red.
            assert_eq!((lines(pg.side(0)), lines(pg.side(1))), (lines(&sides.teams[0]), lines(&sides.teams[1])), "{}", fights[t].name);
            assert_eq!(pg.arena.ships.len(), fights[t].place.len(), "{} placed {} of {}", fights[t].name, pg.arena.ships.len(), fights[t].place.len());
            // Your team is as it was.
            assert_eq!(lines(&pg.lib), mine, "{} touched your team", fights[t].name);
        }
        // A ship placed in a team fight is that side's card.
        pg.load_named("SWARM VS STARTER DECK");
        pg.key(crate::app::Key::Letter('T'));
        pg.slot = 2;
        let n = pg.arena.ships.len();
        pg.event(crate::app::Ev::Press { x: 200, y: 30, right: false });
        assert_eq!(pg.arena.ships.len(), n + 1);
        assert_eq!(pg.arena.ships.last().unwrap().name, "FRIGATE");
        // To the yard and back: your team, in its own fight; your team in the yard all along.
        pg.key(crate::app::Key::Enter);
        assert_eq!((pg.screen, lines(&pg.lib)), (Screen::Yard, mine.clone()));
        pg.key(crate::app::Key::Enter);
        assert_eq!((pg.screen, pg.test), (Screen::Arena, Some(0)));
        assert_eq!(lines(pg.side(1)), mine);
        // Nothing a team fight did was saved over your team.
        let saved = std::fs::read_to_string(&save).unwrap_or_default();
        assert!(!["NEST", "CORVETTE", "LANCET"].iter().any(|n| saved.contains(&format!("\n{n}|"))), "a team fight's card was saved as yours");
        let _ = std::fs::remove_dir_all(&dir);
    }
}
