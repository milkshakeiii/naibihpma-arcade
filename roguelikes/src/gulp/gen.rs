//! Pond generation, areas and the lessons.

use super::rules::*;
use crate::audio::Track;
use crate::rng::Rng;

pub const PONDS: u32 = 10;

pub struct Area {
    pub name: &'static str,
    pub kinds: &'static [Kind],
    pub track: Track,
}

pub const AREAS: [Area; 4] = [
    Area { name: "THE UNDERBRUSH", kinds: &[Kind::Beetle, Kind::Newt], track: Track::Underbrush },
    Area { name: "STAR POND", kinds: &[Kind::Beetle, Kind::Newt, Kind::Dragonfly, Kind::Snake], track: Track::StarPond },
    Area { name: "THE DEEP POOL", kinds: &[Kind::Newt, Kind::Dragonfly, Kind::Snake, Kind::Crayfish, Kind::Heron], track: Track::AncientMagic },
    Area { name: "GROF'S HOLLOW", kinds: &[Kind::Newt, Kind::Beetle], track: Track::Grof },
];

pub fn area_of(pond: u32) -> usize {
    match pond {
        0..=3 => 0,
        4..=6 => 1,
        7..=9 => 2,
        _ => 3,
    }
}

fn starters(pond: u32) -> usize {
    [3, 3, 4, 4, 5, 5, 5, 6, 6, 2][(pond.clamp(1, 10) - 1) as usize]
}

fn reinforcements(pond: u32) -> usize {
    [1, 2, 2, 3, 3, 3, 4, 4, 5, 4][(pond.clamp(1, 10) - 1) as usize]
}

/// What carries from pond to pond: the frog itself.
pub fn carry(frog: &Frog, into: &mut Pond) {
    let pos = into.frog.pos;
    into.frog = frog.clone();
    into.frog.pos = pos;
    into.frog.hop_left = true;
    into.frog.act_left = true;
    into.frog.croak_cd = 0;
    into.frog.camo_ready = frog.has(Mutation::Camouflage);
}

pub fn generate(pond: u32, seed: &mut Rng, frog: &Frog) -> Pond {
    let area = &AREAS[area_of(pond)];
    for attempt in 0..300 {
        let rng = &mut Rng::new(seed.next_u64());
        let mut p = Pond::empty(Rng::new(rng.next_u64()));
        // Water: one or two rounded pools, some of it lily pads.
        for _ in 0..rng.range(1, 2) {
            let c = (rng.range(1, N - 2), rng.range(2, N - 3));
            let r = rng.range(1, 2);
            for y in 0..N {
                for x in 0..N {
                    let (dx, dy) = (x - c.0, y - c.1);
                    if dx * dx + dy * dy <= r * r + 1 {
                        p.set((x, y), if rng.below(100) < 28 { Tile::Lily } else { Tile::Water });
                    }
                }
            }
        }
        for _ in 0..rng.range(2, 4) {
            let at = (rng.range(0, N - 1), rng.range(1, N - 2));
            if p.tile(at) == Tile::Bank {
                p.set(at, if rng.below(2) == 0 { Tile::Reeds } else { Tile::Stone });
            }
        }
        for _ in 0..rng.range(0, 2) {
            let at = (rng.range(0, N - 1), rng.range(1, N - 2));
            if p.tile(at) == Tile::Bank {
                p.set(at, Tile::Thorns);
            }
        }
        p.frog.pos = (rng.range(0, N - 1), N - 1);
        p.exit = (rng.range(0, N - 1), 0);
        if (p.exit.0 - p.frog.pos.0).abs() < 2 {
            continue;
        }
        for at in [p.frog.pos, p.exit] {
            p.set(at, Tile::Bank);
        }
        // Keep the start's neighbours walkable so the first turn has choices.
        for d in DIRS8 {
            let q = (p.frog.pos.0 + d.0, p.frog.pos.1 + d.1);
            if inb(q) && p.tile(q).solid() {
                p.set(q, Tile::Bank);
            }
        }
        // The frog's spawn, in the lower half where the frog can look after it.
        let clutches = if pond <= 3 { 1 } else { 2 };
        for _ in 0..clutches {
            for _ in 0..40 {
                let at = (rng.range(1, N - 2), rng.range(3, N - 2));
                if !p.tile(at).solid() && p.tile(at) != Tile::Thorns && cheb(at, p.frog.pos) >= 2 && !p.eggs.contains(&at) {
                    p.eggs.push(at);
                    break;
                }
            }
        }
        let boss = pond >= PONDS;
        if boss {
            p.add_creature(Kind::Grof, (rng.range(2, 5), rng.range(1, 2)));
            p.exit = (-9, -9);
        }
        let mut ok = true;
        for _ in 0..starters(pond) {
            let kind = *rng.pick(area.kinds);
            let mut placed = false;
            for _ in 0..80 {
                let at = (rng.range(0, N - 1), rng.range(0, 4));
                let spaced = cheb(at, p.frog.pos) >= 3 + (attempt < 200) as i32;
                if spaced && p.at(at).is_none() && !p.tile(at).solid() && (kind.info().swims || kind.info().flies || p.tile(at) != Tile::Water) {
                    p.add_creature(kind, at);
                    placed = true;
                    break;
                }
            }
            ok &= placed;
        }
        if !ok {
            continue;
        }
        for _ in 0..rng.range(2, 3) {
            for _ in 0..40 {
                let at = (rng.range(0, N - 1), rng.range(0, N - 2));
                if p.at(at).is_none() && !p.tile(at).solid() && at != p.frog.pos {
                    p.add_creature(Kind::Fly, at);
                    break;
                }
            }
        }
        let mut t = 2;
        for _ in 0..reinforcements(pond) {
            t += rng.range(2, 4) as u32;
            p.waves.push((t, *rng.pick(area.kinds)));
        }
        if rng.below(2) == 0 {
            p.waves.push((t + 1, Kind::Fly));
        }
        // Let the creatures take aim before the frog's first turn.
        let ids: Vec<u32> = p.creatures.iter().map(|c| c.id).collect();
        let _ = ids;
        p.creatures_turn_aim_only();
        carry(frog, &mut p);
        if p.hop_targets().iter().filter(|&&h| p.preview_hop(h) == 0).count() < 2 {
            continue;
        }
        // No clutch under a strike on turn one.
        if p.danger().iter().any(|(q, _)| p.eggs.contains(q)) && attempt < 250 {
            continue;
        }
        return p;
    }
    unreachable!("pond generation always finds a fair pond")
}

impl Pond {
    /// Creatures move and aim without anything resolving (the start of a pond).
    pub fn creatures_turn_aim_only(&mut self) {
        let saved = (self.turn, self.frog.hop_left, self.frog.act_left);
        let mut ev = Vec::new();
        let ids: Vec<u32> = self.creatures.iter().map(|c| c.id).collect();
        for id in ids {
            if let Some(i) = self.creatures.iter().position(|c| c.id == id) {
                if self.creatures[i].kind != Kind::Fly {
                    self.aim_only(id, &mut ev);
                }
            }
        }
        (self.turn, self.frog.hop_left, self.frog.act_left) = saved;
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Trigger {
    Turns(u32),
    Swallow,
    Gulp,
    Grow,
    Spit,
    Croak,
    Cleared,
    Exit,
}

pub struct Lesson {
    pub title: &'static str,
    pub rows: [&'static str; 8],
    pub steps: &'static [(&'static str, Trigger)],
}

pub const LESSONS: [Lesson; 4] = [
    Lesson {
        title: "LESSON 1: HOPPING",
        rows: ["...E....", "..|.....", ".~~~o...", ".~o~~...", "..~~..#.", "........", ".|......", "..@....."],
        steps: &[
            ("CLICK A LIT SQUARE TO HOP ONE SQUARE, ANY WAY. EACH TURN YOU GET ONE HOP AND ONE ACTION; CLICK YOURSELF TO END THE TURN.", Trigger::Turns(1)),
            ("YOU SWIM, SO WATER IS FINE FOR YOU. REEDS AND STONES BLOCK YOU. HOP TO THE EXIT AT THE TOP.", Trigger::Exit),
        ],
    },
    Lesson {
        title: "LESSON 2: TONGUE AND GULP",
        rows: ["....E...", "........", ".f....f.", "........", "...f....", "........", "........", "...@...."],
        steps: &[
            ("EACH TURN YOU HOP ONCE AND ACT ONCE. PRESS T (OR TONGUE), THEN CLICK A SQUARE IN ONE OF THE 8 LINES: IT GRABS THE FIRST THING THERE.", Trigger::Swallow),
            ("A FLY IS SMALLER THAN YOU, SO YOU SWALLOWED IT. PRESS G (GULP) TO DIGEST IT AND GROW.", Trigger::Gulp),
            ("EAT 3 AND YOU GROW A SIZE: BIGGER FROGS SWALLOW BIGGER THINGS. KEEP EATING.", Trigger::Grow),
            ("YOU'RE MEDIUM NOW. SPACE ENDS YOUR TURN EARLY. HOP TO THE EXIT.", Trigger::Exit),
        ],
    },
    Lesson {
        title: "LESSON 3: STRIKES",
        rows: ["......E.", "........", ".b......", "........", ".~~~....", "......n.", "...g....", ".@......"],
        steps: &[
            ("RED SQUARES ARE WHERE CREATURES WILL STRIKE WHEN YOUR TURN ENDS. DON'T END YOUR TURN IN THEM. POINT AT A CREATURE TO SEE IT.", Trigger::Turns(1)),
            ("THE CLUTCH OF EGGS IS YOURS. PREDATORS GO FOR IT TOO: A STRIKE ON IT COSTS YOU A HEART. EACH CLUTCH THAT SURVIVES A POND HEALS YOU 1.", Trigger::Turns(2)),
            ("A BEETLE CHARGES STRAIGHT UNTIL IT HITS SOMETHING. MOST CREATURES THAT END UP IN WATER DROWN. CLEAR THE POND.", Trigger::Cleared),
            ("THE POND IS CLEAR. HOP TO THE EXIT.", Trigger::Exit),
        ],
    },
    Lesson {
        title: "LESSON 4: SPIT AND CROAK",
        rows: ["E.......", "........", "..~~~...", "..~n~...", "...f..h.", "........", "..g.f...", "....@..."],
        steps: &[
            ("SPIT (S) SHOOTS WHAT'S IN YOUR THROAT: IT HITS HARD AND SHOVES. CROAK (C) SHOVES EVERYTHING NEXT TO YOU A SQUARE.", Trigger::Turns(1)),
            ("A SHOVED CREATURE STRIKES FROM WHERE IT ENDS UP. SOMETIMES THAT'S ANOTHER CREATURE. CLEAR THE POND.", Trigger::Cleared),
            ("BETWEEN PONDS YOU'LL CHOOSE A MUTATION. HOP TO THE EXIT.", Trigger::Exit),
        ],
    },
];

pub fn lesson_pond(i: usize, frog: &Frog) -> Pond {
    let mut p = Pond::empty(Rng::new(101 + i as u64));
    for (y, r) in LESSONS[i].rows.iter().enumerate() {
        for (x, c) in r.chars().enumerate() {
            let at = (x as i32, y as i32);
            match c {
                '~' => p.set(at, Tile::Water),
                'o' => p.set(at, Tile::Lily),
                '|' => p.set(at, Tile::Reeds),
                '#' => p.set(at, Tile::Stone),
                '^' => p.set(at, Tile::Thorns),
                '@' => p.frog.pos = at,
                'E' => p.exit = at,
                'g' => p.eggs.push(at),
                'f' => drop(p.add_creature(Kind::Fly, at)),
                'b' => drop(p.add_creature(Kind::Beetle, at)),
                'n' => drop(p.add_creature(Kind::Newt, at)),
                'h' => drop(p.add_creature(Kind::Heron, at)),
                _ => {}
            }
        }
    }
    p.creatures_turn_aim_only();
    carry(frog, &mut p);
    if !p.predators_left() {
        p.cleared = true;
    }
    p
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ponds_generate_fairly() {
        let mut rng = Rng::new(8);
        for pond in 1..=PONDS {
            for _ in 0..15 {
                let p = generate(pond, &mut rng, &Frog::new());
                assert!(p.hop_targets().iter().filter(|&&h| p.preview_hop(h) == 0).count() >= 2);
                if pond == PONDS {
                    assert!(p.creatures.iter().any(|c| c.kind == Kind::Grof));
                }
            }
        }
    }

    #[test]
    fn lessons_parse() {
        for i in 0..LESSONS.len() {
            let p = lesson_pond(i, &Frog::new());
            assert!(inb(p.exit));
        }
    }
}
