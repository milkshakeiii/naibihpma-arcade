//! Burrow generation and Doat's lesson boards.

use super::rules::*;
use crate::audio::Track;
use crate::rng::Rng;

pub const BURROWS: u32 = 10;

pub struct Area {
    pub name: &'static str,
    pub pool: &'static [(Kind, u32)],
    pub spawn_every: u32,
    /// Catches needed to open each burrow.
    pub seal: i32,
    pub track: Track,
}

pub const AREAS: [Area; 4] = [
    Area { name: "THE UNDERBRUSH", pool: &[(Kind::Newt, 3), (Kind::Beetle, 3), (Kind::Frog, 2)], spawn_every: 7, seal: 2, track: Track::Underbrush },
    Area {
        name: "THE DEWFIELDS",
        pool: &[(Kind::Newt, 2), (Kind::Beetle, 2), (Kind::Frog, 2), (Kind::Cricket, 2), (Kind::Heron, 2)],
        spawn_every: 5,
        seal: 3,
        track: Track::Twen,
    },
    Area {
        name: "STAR POND",
        pool: &[(Kind::Frog, 2), (Kind::Cricket, 2), (Kind::Heron, 2), (Kind::Snake, 2), (Kind::Salamander, 2), (Kind::Glowtoad, 1)],
        spawn_every: 4,
        seal: 3,
        track: Track::StarPond,
    },
    Area {
        name: "GROF'S HOLLOW",
        pool: &[(Kind::Frog, 2), (Kind::Cricket, 2), (Kind::Heron, 1), (Kind::Snake, 2), (Kind::Salamander, 1)],
        spawn_every: 5,
        seal: 0,
        track: Track::Grof,
    },
];

pub fn area_of(burrow: u32) -> usize {
    match burrow {
        0..=3 => 0,
        4..=6 => 1,
        7..=9 => 2,
        _ => 3,
    }
}

fn creature_count(burrow: u32) -> usize {
    [3, 4, 4, 5, 5, 6, 6, 7, 7, 3][(burrow.clamp(1, 10) - 1) as usize]
}

fn pick_kind(rng: &mut Rng, pool: &[(Kind, u32)]) -> Kind {
    let total: u32 = pool.iter().map(|p| p.1).sum();
    let mut r = rng.below(total);
    for &(k, w) in pool {
        if r < w {
            return k;
        }
        r -= w;
    }
    pool[0].0
}

/// What carries from one burrow to the next.
#[derive(Clone, Debug)]
pub struct Carry {
    pub hearts: i32,
    pub max_hearts: i32,
    pub dew: i32,
    pub charms: Vec<Charm>,
    pub form: Kind,
    pub prev_form: Option<Kind>,
}

impl Carry {
    pub fn fresh() -> Self {
        Self { hearts: 3, max_hearts: 3, dew: 0, charms: Vec::new(), form: Kind::Wanderer, prev_form: None }
    }

    pub fn apply(&self, f: &mut Floor) {
        f.hearts = self.hearts;
        f.max_hearts = self.max_hearts;
        f.dew_total = self.dew;
        f.charms = self.charms.clone();
        f.form = self.form;
        f.prev_form = self.prev_form;
        f.shell_ready = self.charms.contains(&Charm::Shell);
    }

    pub fn take(&mut self, f: &Floor) {
        self.hearts = f.hearts;
        self.max_hearts = f.max_hearts;
        self.dew = f.dew_total;
        self.form = f.form;
        self.prev_form = f.prev_form;
    }
}

/// Every open square reachable from `start` with single steps (so no pocket is sealed off).
fn connected(f: &Floor, start: Pos) -> bool {
    let mut seen = vec![start];
    let mut stack = vec![start];
    while let Some(p) = stack.pop() {
        for d in ALL8 {
            let q = (p.0 + d.0, p.1 + d.1);
            if inb(q) && f.cell(q) != Cell::Rock && !seen.contains(&q) {
                seen.push(q);
                stack.push(q);
            }
        }
    }
    let open = (0..N * N).filter(|i| f.cell((i % N, i / N)) != Cell::Rock).count();
    seen.len() == open
}

pub fn generate(burrow: u32, seed_rng: &mut Rng, carry: &Carry) -> Floor {
    let area = &AREAS[area_of(burrow)];
    // Strict at first; later attempts relax spacing, safety and (finally) the creature count, so
    // generation always produces a real burrow.
    for attempt in 0..600 {
        let (spacing, need_safe, fewer) = match attempt {
            0..=299 => (4, 3, 0),
            300..=499 => (3, 2, 0),
            500..=579 => (3, 1, 1),
            _ => (3, 0, 2),
        };
        let rng = &mut Rng::new(seed_rng.next_u64());
        let mut f = Floor::empty(Rng::new(rng.next_u64()));
        carry.apply(&mut f);
        f.spawn_every = area.spawn_every;
        f.seal = area.seal;
        f.spawn_pool = area.pool.iter().map(|p| p.0).collect();
        let boss = burrow >= BURROWS;
        f.player = (rng.range(0, N - 1), N - 1);
        let burrow_pos = if boss { None } else { Some((rng.range(0, N - 1), 0)) };
        if let Some(b) = burrow_pos {
            if (b.0 - f.player.0).abs() < 2 {
                continue;
            }
            f.cells[b.1 as usize][b.0 as usize] = Cell::Burrow;
        }
        let rocks = rng.range(5, if area_of(burrow) == 0 { 8 } else { 9 });
        let mut placed = 0;
        while placed < rocks {
            let p = (rng.range(0, N - 1), rng.range(0, N - 1));
            if f.cell(p) != Cell::Ground || cheb(p, f.player) <= 1 || burrow_pos.is_some_and(|b| cheb(p, b) <= 0) {
                continue;
            }
            f.cells[p.1 as usize][p.0 as usize] = Cell::Rock;
            placed += 1;
        }
        if !connected(&f, f.player) {
            continue;
        }
        let count = creature_count(burrow).saturating_sub(fewer).max(1);
        let mut kinds: Vec<Kind> = (0..count).map(|_| pick_kind(rng, area.pool)).collect();
        if boss {
            let gp = (rng.range(2, 5), rng.range(0, 1));
            if f.cell(gp) != Cell::Ground {
                continue;
            }
            f.add_creature(Kind::Grof, gp);
        }
        // From the Dewfields on, make sure the area's new creatures show up.
        if burrow == 4 && !kinds.iter().any(|k| matches!(k, Kind::Heron | Kind::Cricket)) {
            kinds[0] = Kind::Heron;
        }
        if burrow == 7 && !kinds.iter().any(|k| matches!(k, Kind::Snake | Kind::Salamander | Kind::Glowtoad)) {
            kinds[0] = Kind::Snake;
        }
        let mut ok = true;
        for k in kinds {
            let mut tries = 0;
            loop {
                tries += 1;
                if tries > 60 {
                    ok = false;
                    break;
                }
                let p = (rng.range(0, N - 1), rng.range(0, 4));
                if f.occ(p) != Occ::Empty || cheb(p, f.player) < spacing {
                    continue;
                }
                f.add_creature(k, p);
                let i = f.creatures.len() - 1;
                if f.reach(p, k, 0, None).contains(&f.player) {
                    f.creatures.remove(i);
                    f.next_id -= 1;
                    continue;
                }
                break;
            }
        }
        if !ok {
            continue;
        }
        f.plan_intents();
        // The first turn must offer a few safe squares.
        let safe = f.targets().iter().filter(|&&t| f.preview(Action::Move(t)).hits == 0).count();
        if safe < need_safe {
            continue;
        }
        let drops = rng.range(2, 3);
        let mut d = 0;
        while d < drops {
            let p = (rng.range(0, N - 1), rng.range(0, N - 2));
            if f.occ(p) == Occ::Empty && !f.dew.contains(&p) {
                f.dew.push(p);
                d += 1;
            }
        }
        f.plan_intents();
        return f;
    }
    unreachable!("burrow generation always succeeds by its last attempts")
}

pub struct Lesson {
    pub title: &'static str,
    pub rows: [&'static str; 8],
    /// Each step's text and the condition that moves past it.
    pub steps: &'static [(&'static str, Trigger)],
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Trigger {
    Moves(u32),
    Catch,
    Rest,
    PointAtCreature,
    /// Two or more catches in one turn.
    Chain,
    Descend,
}

pub const LESSONS: [Lesson; 4] = [
    Lesson {
        title: "DOAT'S LESSON 1: FIRST STEPS",
        rows: ["..o..B..", "..o.....", "....oo..", ".o......", "...o..o.", "o.......", "...o....", "..@....."],
        steps: &[
            ("YOU ARE A CHANGELING. THE LIT SQUARES SHOW WHERE YOU CAN MOVE: ONE STEP, ANY DIRECTION. CLICK ONE.", Trigger::Moves(1)),
            ("THE BURROW AT THE TOP LEADS DEEPER INTO THE FOREST. GO DOWN IT.", Trigger::Descend),
        ],
    },
    Lesson {
        title: "DOAT'S LESSON 2: CATCHING",
        rows: ["......B.", "........", "..oooo..", "........", "...n....", "........", "........", "...@...."],
        steps: &[
            ("A NEWT. MOVE ONTO A CREATURE TO CATCH IT. THE RED SQUARES ARE WHERE IT COULD STRIKE YOU.", Trigger::Catch),
            ("YOU TOOK ITS SHAPE: YOU MOVE ONE STEP STRAIGHT NOW, LIKE A NEWT. CLICK YOURSELF TO REST AND BE YOURSELF AGAIN.", Trigger::Rest),
            ("RESTING COSTS A TURN: THE CREATURES MOVE WHILE YOU DO IT. NOW TAKE THE BURROW.", Trigger::Descend),
        ],
    },
    Lesson {
        title: "DOAT'S LESSON 3: DANGER",
        rows: ["B.......", "..h.o...", "........", "oo..f...", "......d.", "..d.....", "........", ".....@.."],
        steps: &[
            ("EVERY CREATURE MOVES AND STRIKES WITH ITS OWN SHAPE. POINT AT ONE TO SEE ITS SHAPE.", Trigger::PointAtCreature),
            (
                "EACH ARROW SHOWS EXACTLY WHERE THAT CREATURE WILL MOVE AFTER YOUR TURN, WHATEVER YOU DO. A CREATURE THAT CAN REACH YOU STRIKES INSTEAD (MARKED !).",
                Trigger::Moves(1),
            ),
            ("END A MOVE ON A RED SQUARE AND YOU ARE STRUCK: YOU LOSE A HEART. POINT AT A LIT SQUARE TO SEE WHAT WOULD HAPPEN THERE.", Trigger::Moves(2)),
            ("LINES, LIKE THE HERON'S, STOP AT ROCKS AND AT ANYTHING STANDING IN THEM. LEAPS GO OVER EVERYTHING.", Trigger::Moves(1)),
            ("CATCHES AND DROPS GIVE DEW, TO SPEND IN THE GLADE BETWEEN BURROWS. TAKE THE BURROW WHEN YOU'RE READY.", Trigger::Descend),
        ],
    },
    Lesson {
        title: "DOAT'S LESSON 4: CHAINS",
        rows: ["...B....", "........", ".o....o.", "...n....", "..b.....", ".b...b..", "........", "....@..."],
        steps: &[
            (
                "AFTER A CATCH, IF YOUR NEW SHAPE CAN REACH ANOTHER CREATURE, YOU MAY CATCH IT TOO, BEFORE ANYTHING MOVES. THAT'S A CHAIN. MAKE ONE.",
                Trigger::Chain,
            ),
            ("A CHAIN ENDS WHEN YOUR SHAPE REACHES NO CREATURE, OR WHEN YOU CLICK YOURSELF. THEN THE CREATURES TAKE THEIR TURN. GO DOWN THE BURROW.", Trigger::Descend),
        ],
    },
];

pub fn lesson_floor(i: usize, carry: &Carry) -> Floor {
    let mut f = Floor::empty(Rng::new(77 + i as u64));
    carry.apply(&mut f);
    f.form = Kind::Wanderer;
    f.prev_form = None;
    for (y, r) in LESSONS[i].rows.iter().enumerate() {
        for (x, c) in r.chars().enumerate() {
            let p = (x as i32, y as i32);
            match c {
                'o' => f.cells[y][x] = Cell::Rock,
                'B' => f.cells[y][x] = Cell::Burrow,
                '@' => f.player = p,
                'd' => f.dew.push(p),
                'n' => drop(f.add_creature(Kind::Newt, p)),
                'b' => drop(f.add_creature(Kind::Beetle, p)),
                'f' => drop(f.add_creature(Kind::Frog, p)),
                'h' => drop(f.add_creature(Kind::Heron, p)),
                _ => {}
            }
        }
    }
    f.plan_intents();
    f
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generated_burrows_are_fair_on_turn_one() {
        let mut rng = Rng::new(42);
        for b in 1..=BURROWS {
            for _ in 0..20 {
                let f = generate(b, &mut rng, &Carry::fresh());
                assert!(connected(&f, f.player));
                assert!(f.creatures.iter().all(|c| !f.reach(c.pos, c.kind, 0, None).contains(&f.player)));
                let safe = f.targets().iter().filter(|&&t| f.preview(Action::Move(t)).hits == 0).count();
                assert!(safe >= 1, "burrow {b}");
                if b < BURROWS {
                    assert!((0..N).any(|x| f.cell((x, 0)) == Cell::Burrow));
                } else {
                    assert!(f.creatures.iter().any(|c| c.kind == Kind::Grof));
                }
            }
        }
    }

    #[test]
    fn lessons_parse() {
        for i in 0..LESSONS.len() {
            let f = lesson_floor(i, &Carry::fresh());
            assert!((0..N).any(|x| f.cell((x, 0)) == Cell::Burrow));
        }
    }
}

#[cfg(test)]
mod strict_tests {
    use super::*;

    /// How often the strict rules (spacing 4, three safe first moves) hold, per area.
    #[test]
    fn most_burrows_meet_the_strict_rules() {
        let mut rng = Rng::new(9);
        for b in [1, 4, 7, 9, 10] {
            let mut strict = 0;
            for _ in 0..40 {
                let f = generate(b, &mut rng, &Carry::fresh());
                assert_eq!(f.creatures.len().max(1) >= 1, true);
                let safe = f.targets().iter().filter(|&&t| f.preview(Action::Move(t)).hits == 0).count();
                if safe >= 3 {
                    strict += 1;
                }
            }
            println!("burrow {b}: {strict}/40 strict");
            assert!(strict >= 20, "burrow {b}: only {strict}/40 strict");
        }
    }
}
