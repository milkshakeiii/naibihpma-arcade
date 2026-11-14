//! Drills. Before a fight, each design fights each enemy design headless at equal cost, each way
//! of flying against each of the other's. What that shows is what the ships then know: how to
//! fly at that enemy, and how a fight with it is likely to go. Nothing here knows what a part
//! does. A new part shows up in the numbers because it was in the fights.

use super::*;

/// Which of the ways it drilled a ship flies at an enemy.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash, Default)]
pub enum Ways {
    /// The way its card says, whatever the enemy.
    #[default]
    Card,
    /// The way whose worst drill against that enemy was best.
    Safe,
    /// The way that did best against that enemy over all the ways the enemy flew.
    Bold,
    /// The way its side's arch-AI has decided for its design, whatever the enemy.
    Plan,
}

/// How a side's ships choose what to go for and how to fly at it.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash, Default)]
pub struct Brain {
    /// By its drills, an enemy it beats seems nearer and one that beats it further, and it goes
    /// for the one that seems nearest. Off: the nearest.
    pub targets: bool,
    pub ways: Ways,
}

/// The brains the arena's buttons step through. (The headless comparison, `--brains`, tries the
/// rest: drilled targets, and the safe ways.)
pub const BRAINS: [Brain; 3] = [Brain::CARD, Brain::DRILLED, Brain::ARCH];

impl Brain {
    pub const CARD: Brain = Brain { targets: false, ways: Ways::Card };
    /// Each ship flies at each enemy the way that did best in its drills. Of the drilled brains
    /// tried, the one that did no worse than the cards.
    pub const DRILLED: Brain = Brain { targets: false, ways: Ways::Bold };
    pub const ARCH: Brain = Brain { targets: false, ways: Ways::Plan };

    /// For a button: at most eight letters.
    pub fn short(self) -> &'static str {
        match (self.targets, self.ways) {
            (false, Ways::Card) => "CARD",
            (true, Ways::Card) => "TARGETS",
            (false, Ways::Safe) => "SAFE",
            (false, Ways::Bold) => "DRILLED",
            (true, Ways::Safe) => "T+SAFE",
            (true, Ways::Bold) => "T+BOLD",
            (false, Ways::Plan) => "ARCH",
            (true, Ways::Plan) => "T+ARCH",
        }
    }

    pub fn name(self) -> String {
        let ways = match self.ways {
            Ways::Card => "",
            Ways::Safe => "SAFE WAYS",
            Ways::Bold => "BOLD WAYS",
            Ways::Plan => "ARCH",
        };
        match (self.targets, self.ways) {
            (false, Ways::Card) => "CARD".into(),
            (true, Ways::Card) => "TARGETS".into(),
            (false, _) => ways.into(),
            (true, _) => format!("TARGETS+{ways}"),
        }
    }
}

/// What one design learned about one enemy design.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash)]
pub struct Drill {
    /// How to fly at it, safely: the style whose worst drill was best. None for a design with
    /// no choice to make (no engines, or no guns).
    pub safe: Option<Style>,
    /// How the fight went with both sides flying safe, in thousandths: the share of the enemy's
    /// pixels destroyed less the share of its own lost. From -1000 to 1000.
    pub safe_margin: i32,
    /// How to fly at it, boldly: the style that did best over all the enemy's ways.
    pub bold: Option<Style>,
    /// How the fight went with both sides flying bold.
    pub bold_margin: i32,
}

impl Drill {
    pub fn style(&self, ways: Ways) -> Option<Style> {
        match ways {
            Ways::Card | Ways::Plan => None,
            Ways::Safe => self.safe,
            Ways::Bold => self.bold,
        }
    }
    pub fn margin(&self, ways: Ways) -> i32 {
        if ways == Ways::Safe { self.safe_margin } else { self.bold_margin }
    }
}

/// What every design learned about every enemy design it was drilled against.
#[derive(Clone, PartialEq, Eq, Debug, Hash, Default)]
pub struct Drills {
    n: usize,
    cells: Vec<Option<Drill>>,
}

/// A drill is over after a minute, whoever is left.
pub const DRILL_TICKS: u32 = 15 * 60;
/// Each pairing is fought on this many rolls of the dice.
pub const DRILL_SEEDS: u64 = 2;
/// The most ships a side fields in a drill, however cheap they are.
pub const DRILL_MOST: i32 = 60;
/// How far a drilled ship's judgement bends distance, in thousandths at a margin of 1000: an
/// enemy it wipes out for nothing seems half as far off as it is, one that does that to it half
/// as far again.
pub const DRILL_PULL: i32 = 500;

/// The ways a design could fly in a drill: one for each style if it has engines and guns, and
/// otherwise the one way it has.
pub fn drill_styles(d: &Design) -> Vec<Option<Style>> {
    let st = d.stats();
    if st.flight == Flight::Still || !st.armed() {
        vec![None]
    } else {
        STYLES.iter().map(|&s| Some(s)).collect()
    }
}

/// One drill: `a` as blue on the left, `b` as red on the right, as many of each as the dearer
/// one's cost buys, each side flying one way. Blue's margin, in thousandths.
pub fn drill(a: &Design, b: &Design, sa: Option<Style>, sb: Option<Style>, seed: u64) -> i32 {
    let budget = a.stats().cost.max(b.stats().cost);
    let mut arena = Arena::seeded(seed);
    let mut px = [0i32; 2];
    for (team, d, style, cx) in [(0, a, sa, 130), (1, b, sb, 350)] {
        let n = (budget / d.stats().cost.max(1)).clamp(1, DRILL_MOST);
        let sh = d.shape();
        for (_, _, x, y) in crate::pg::block(team, team, n, cx, 180, sh.w.max(sh.h) + 4) {
            if arena.add(team, team, d, x, y) {
                let s = arena.ships.last_mut().unwrap();
                if style.is_some() {
                    s.style = style;
                }
                px[team] += s.px_full();
            }
        }
    }
    while arena.over.is_none() && arena.tick < DRILL_TICKS {
        arena.step();
    }
    let share = |t: usize| if px[t] > 0 { (arena.score[t].px_lost * 1000 / px[t]).min(1000) } else { 0 };
    share(1) - share(0)
}

/// Each side's best way to fly, given a table of blue's margins: `rows[i][j]` with blue flying
/// its way `i` and red its way `j`. Each takes the way whose worst case is best for it (the
/// first such, on a tie). Returns (blue's way, red's way).
pub fn best_ways(rows: &[Vec<i32>]) -> (usize, usize) {
    let cols = rows.first().map_or(0, |r| r.len());
    let i = (0..rows.len()).max_by_key(|&i| (rows[i].iter().copied().min().unwrap_or(0), std::cmp::Reverse(i))).unwrap_or(0);
    let j = (0..cols).min_by_key(|&j| (rows.iter().map(|r| r[j]).max().unwrap_or(0), j)).unwrap_or(0);
    (i, j)
}

/// Each side's bold way to fly, from the same table: the way that does best for it over all the
/// other's ways together (the first such, on a tie). Returns (blue's way, red's way).
pub fn bold_ways(rows: &[Vec<i32>]) -> (usize, usize) {
    let cols = rows.first().map_or(0, |r| r.len());
    let i = (0..rows.len()).max_by_key(|&i| (rows[i].iter().sum::<i32>(), std::cmp::Reverse(i))).unwrap_or(0);
    let j = (0..cols).min_by_key(|&j| (rows.iter().map(|r| r[j]).sum::<i32>(), j)).unwrap_or(0);
    (i, j)
}

impl Drills {
    /// What design `a` learned about enemy design `b`, if they were drilled.
    pub fn get(&self, a: usize, b: usize) -> Option<Drill> {
        if a < self.n && b < self.n {
            self.cells[a * self.n + b]
        } else {
            None
        }
    }

    pub fn set(&mut self, a: usize, b: usize, d: Drill) {
        let n = (a.max(b) + 1).max(self.n);
        if n > self.n {
            let mut cells = vec![None; n * n];
            for x in 0..self.n {
                for y in 0..self.n {
                    cells[x * n + y] = self.cells[x * self.n + y];
                }
            }
            (self.n, self.cells) = (n, cells);
        }
        self.cells[a * self.n + b] = Some(d);
    }

    /// Drills one pair of designs against each other, every way of flying against every way, and
    /// returns what each learned about the other: (a about b, b about a).
    pub fn pair(a: &Design, b: &Design) -> (Drill, Drill) {
        let (wa, wb) = (drill_styles(a), drill_styles(b));
        let rows: Vec<Vec<i32>> = wa.iter().map(|&sa| wb.iter().map(|&sb| (1..=DRILL_SEEDS).map(|seed| drill(a, b, sa, sb, seed)).sum::<i32>() / DRILL_SEEDS as i32).collect()).collect();
        let ((i, j), (bi, bj)) = (best_ways(&rows), bold_ways(&rows));
        (Drill { safe: wa[i], safe_margin: rows[i][j], bold: wa[bi], bold_margin: rows[bi][bj] }, Drill { safe: wb[j], safe_margin: -rows[i][j], bold: wb[bj], bold_margin: -rows[bi][bj] })
    }

    /// Drills every pair in `pairs` (slots in `lib`), each pair once whichever way round it's
    /// given. Designs that can't fly, and pairs with no guns between them, learn nothing.
    pub fn run(lib: &[Design], pairs: &[(usize, usize)]) -> Drills {
        let mut out = Drills::default();
        let mut done: Vec<(usize, usize)> = Vec::new();
        for &(a, b) in pairs {
            let key = (a.min(b), a.max(b));
            if done.contains(&key) || key.1 >= lib.len() {
                continue;
            }
            done.push(key);
            let (da, db) = (&lib[key.0], &lib[key.1]);
            if !da.check().ok() || !db.check().ok() || !(da.stats().armed() || db.stats().armed()) {
                continue;
            }
            let (x, y) = Drills::pair(da, db);
            out.set(key.0, key.1, x);
            if key.0 != key.1 {
                out.set(key.1, key.0, y);
            }
        }
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn each_side_flies_the_way_whose_worst_case_is_best() {
        // Blue's margins. Its second way is never worse than -100; its first can lose 900.
        // Red wants blue's margin low: against its first way blue can make 800, against its
        // second 300 at most.
        let rows = vec![vec![800, -900], vec![-100, 300], vec![-200, 100]];
        assert_eq!(best_ways(&rows), (1, 1));
        // A side with one way has no choice, and the other answers that way.
        assert_eq!(best_ways(&[vec![10, -50, 30]]), (0, 1));
        assert_eq!(best_ways(&[vec![10], vec![40], vec![-5]]), (1, 0));
        // On a tie, the first.
        assert_eq!(best_ways(&[vec![5, 5], vec![5, 5]]), (0, 0));
    }

    #[test]
    fn the_bold_way_is_the_one_that_does_best_over_all_the_enemys_ways() {
        // Blue's first way makes 800 or loses 900; its second makes 300 or loses 100. Over both
        // of red's ways the second adds up to more. Red's second way costs it least in all.
        let rows = vec![vec![800, -900], vec![-100, 300], vec![-200, 100]];
        assert_eq!(bold_ways(&rows), (1, 1));
        // Where safe and bold part: one way that's never bad, and one that's usually much better.
        let rows = vec![vec![0, 0, 0], vec![900, 900, -100]];
        assert_eq!((best_ways(&rows).0, bold_ways(&rows).0), (0, 1));
    }

    #[test]
    fn a_pair_drilled_twice_learns_the_same_and_one_sides_margin_is_the_others_loss() {
        let d = starter_deck();
        let (x, y) = Drills::pair(&d[0], &d[1]);
        assert_eq!((x, y), Drills::pair(&d[0], &d[1]));
        assert_eq!((x.safe_margin, x.bold_margin), (-y.safe_margin, -y.bold_margin));
        // Fighters beat lancers at equal cost, however each flies: the starter deck's triangle.
        assert!(x.safe_margin > 0 && x.bold_margin > 0 && x.bold.is_some() && y.bold.is_some(), "{x:?}");
        // A design with no engines has no way of flying to choose, and one with no guns neither.
        let ex = part_examples();
        let aegis = ex.iter().find(|d| d.name == "AEGIS").unwrap();
        assert_eq!((drill_styles(&d[0]).len(), drill_styles(aegis)), (3, vec![None]));
    }

    /// A blue fighter with a red lancer 60 px off and a red frigate 75 px off, after one tick.
    /// The table says the fighter loses to the lancer and beats the frigate.
    fn between_two(targets: bool) -> Arena {
        let d = starter_deck();
        let mut a = Arena::new();
        a.add(0, 0, &d[0], 100, 180);
        a.add(1, 1, &d[1], 160, 180);
        a.add(1, 2, &d[2], 175, 180);
        let k = |m| Drill { safe: None, safe_margin: m, bold: None, bold_margin: m };
        let mut t = Drills::default();
        t.set(0, 1, k(-1000));
        t.set(0, 2, k(1000));
        a.drills = std::sync::Arc::new(t);
        a.brain[0] = Brain { targets, ways: Ways::Card };
        a.step();
        a
    }

    #[test]
    fn a_drilled_ship_goes_for_the_enemy_it_beats_and_not_the_nearer_one_it_loses_to() {
        // By its card: the nearest, the lancer.
        assert_eq!(between_two(false).ships[0].prey, Some(1));
        // By its drills: the lancer seems half as far again as it is (90 px) and the frigate half
        // as far (37 px).
        assert_eq!(between_two(true).ships[0].prey, Some(2));
        // The other side, on its cards, is none the wiser.
        assert_eq!(between_two(true).ships[1].prey, Some(0));
    }

    /// A blue fighter 200 px from a red frigate, two seconds in: what the fighter is doing.
    /// Its drills found KITE safe against a frigate, and ORBIT bold.
    fn flying_at_a_frigate(ways: Ways) -> Doing {
        let d = starter_deck();
        let mut a = Arena::new();
        a.add(0, 0, &d[0], 100, 180);
        a.add(1, 2, &d[2], 300, 180);
        let mut t = Drills::default();
        t.set(0, 2, Drill { safe: Some(Style::Kite), safe_margin: 0, bold: Some(Style::Orbit), bold_margin: 0 });
        a.drills = std::sync::Arc::new(t);
        a.brain[0] = Brain { targets: false, ways };
        for _ in 0..30 {
            a.step();
        }
        a.ships[0].doing
    }

    #[test]
    fn a_drilled_ship_flies_at_an_enemy_the_way_its_drills_found() {
        // The fighter's card flies attack runs.
        assert!(matches!(flying_at_a_frigate(Ways::Card), Doing::RunIn | Doing::RunOut), "{:?}", flying_at_a_frigate(Ways::Card));
        assert_eq!(flying_at_a_frigate(Ways::Safe), Doing::StandOff);
        assert_eq!(flying_at_a_frigate(Ways::Bold), Doing::Orbit);
    }

    #[test]
    fn a_table_holds_what_was_put_in_it_as_it_grows() {
        let mut t = Drills::default();
        assert!(t.get(0, 0).is_none());
        let d = |m| Drill { safe: Some(Style::Kite), safe_margin: m, bold: Some(Style::Runs), bold_margin: m };
        t.set(0, 1, d(10));
        t.set(4, 2, d(-30));
        t.set(1, 0, d(-10));
        assert_eq!((t.get(0, 1), t.get(1, 0), t.get(4, 2)), (Some(d(10)), Some(d(-10)), Some(d(-30))));
        assert_eq!((t.get(2, 4), t.get(9, 9)), (None, None));
    }
}
