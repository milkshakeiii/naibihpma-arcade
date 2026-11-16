//! The arch-AI: one planner for a whole side, which decides how its ships fly. It plays the
//! fight forward from where things stand, once with nothing changed and once for each change it
//! could make (some of its ships flying another way), and makes every change that came out
//! better than leaving things as they are. Nothing here knows what a part does: it only watches
//! what happens.
//!
//! It thinks only while there's a fight, about every ship of its side in it at once. How fine
//! it thinks goes by how big the fight is, so that a round of decisions takes about a second:
//! each ship for itself in a small fight, squads of a design in a bigger one, a whole design at
//! a time in the biggest (`units`).
//!
//! The playing forward is done on other threads (`pool.rs`), and what's played is the fight
//! alone: copies of the ships in it in a bare arena, with no match round them (see `pocket`).
//! What a think comes to is used on a tick that's fixed when it begins, by the size of the fight
//! alone (`arch_wait`): never sooner, however fast the machine, and a machine that hasn't
//! finished by then waits for it. So two machines playing one match always fly the same plan,
//! and every machine plays the same game: the one planned for the machine in `ARCH_CORE`.

use super::pool::{submit, Job};
use super::*;
use std::sync::{Arc, OnceLock};

/// How far ahead it plays each way of flying.
pub const ARCH_AHEAD: u32 = 15 * 8;
/// A ship is in a fight, and is played forward, while an enemy is within this of it.
pub const ARCH_WAKE: i32 = 300;
/// What a tick of playing a fight forward costs, in units of work: for each ship in it, this
/// much for each gun in the fight (`ARCH_GUN`: every gun looks at every ship) and this much
/// besides (`ARCH_SHIP`). A unit is a nanosecond on the machine this was measured on, an
/// i9-13900KF: fitted to the 6,892 thinks of four matches (2026-11-15), where the square of the
/// ships, the sum before it, was out by up to three times, and this by up to twice.
pub const ARCH_GUN: i64 = 10;
pub const ARCH_SHIP: i64 = 1200;
/// The machine every match is planned for (Henry, 2026-11-15: four cores, each about half as
/// fast as the one this was measured on): the units of work one of its three spare cores is
/// counted on for in a tick. A tick is 66.7 million ns; half of that for the slower core, and
/// two thirds of that for a sum that's sometimes short. A slower machine still plays in step:
/// it waits at the tick an answer is due.
pub const ARCH_CORE: i64 = 22_000_000;
/// What a side's planner counts on in a tick: half of the three spare cores.
pub const ARCH_SIDE: i64 = ARCH_CORE * 3 / 2;
/// How long a round of decisions should take, in ticks: it sets how fine a side thinks
/// (`units`). In the biggest fights a round takes longer, as a side never thinks coarser than a
/// design at a time.
pub const ARCH_ROUND: u32 = 15;
/// A side begins a think no sooner than this after it began its last: as often as it decided
/// for a design in a small fight when it thought on the game's own thread.
pub const ARCH_PERIOD: u32 = 8;
/// The soonest a think is used after it began.
pub const ARCH_SOONEST: u32 = 2;
/// With no fight on, how often it looks for one.
pub const ARCH_IDLE: u32 = 8;

/// Time the game's own thread spent on the planner, both sides, in nanoseconds: beginning
/// thinks, using them, and any waiting for an answer that wasn't ready. Only `--frame-cost`
/// reads it: the planner itself never does, so it changes nothing in a match.
pub static ARCH_NANOS: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);

/// The work of playing a fight forward once: `n` ships with `guns` guns among them.
pub fn arch_work(n: usize, guns: usize) -> i64 {
    ARCH_AHEAD as i64 * n as i64 * (ARCH_GUN * guns as i64 + ARCH_SHIP)
}

/// How many ticks after it begins a think is used: `plays` playings forward, each of `one`
/// work, on the machine planned for. Time for all of them on a side's share of the cores, and
/// never less than for one of them on one core.
pub fn arch_wait(one: i64, plays: usize) -> u32 {
    let all = (one * plays as i64 + ARCH_SIDE - 1) / ARCH_SIDE;
    let longest = (one + ARCH_CORE - 1) / ARCH_CORE;
    (all.max(longest) as u32).max(ARCH_SOONEST)
}

/// How fine a side's planner thinks.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash, Default)]
pub enum Grain {
    /// As fine as a round of decisions has the time for (`units`).
    #[default]
    Fine,
    /// A whole design at a time, whatever the size of the fight. Only tools ask for it, to
    /// measure what thinking finer is worth.
    Designs,
}

/// How one decision came out: the side and the design, the ways played forward (the way most of
/// those ships flew at the time first, which is the fight with nothing changed) and what each
/// scored, the way kept, how many ships were in the fight, how many of them this was a decision
/// for, and how many ticks the think took.
#[derive(Clone, Debug)]
pub struct Choice {
    pub team: usize,
    pub slot: usize,
    pub ways: Vec<Option<Style>>,
    pub scores: Vec<i32>,
    pub kept: Option<Style>,
    pub ships: usize,
    pub members: usize,
    pub took: u32,
}

thread_local! {
    /// How every decision made on this thread came out, while it's on (Some). Only `--ways`
    /// turns it on and reads it: the planner itself never does, so it changes nothing in a match.
    pub static ARCH_CHOICES: std::cell::RefCell<Option<Vec<Choice>>> = const { std::cell::RefCell::new(None) };
}

#[cfg(test)]
thread_local! {
    /// A deliberate fault, for a test to catch: a think is used the moment its answers are all
    /// in, not on the tick it's due.
    pub static ARCH_EAGER: std::cell::Cell<bool> = const { std::cell::Cell::new(false) };
}

/// What a side's planner has decided for a ship, for the bar under it.
pub fn chosen_line(way: Style) -> String {
    let how = match way {
        Style::Runs => "ON ATTACK RUNS",
        Style::Kite => "KITING",
        Style::Orbit => "ORBITING",
    };
    format!("ITS SIDE HAS IT {how}.")
}

/// How a side's planner judges a change it has played forward. Only tools change it.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum Judge {
    /// By the whole fight: the pixels the enemy lost less the pixels its side lost. The game's.
    #[default]
    Fight,
    /// By the changed ships' own account: the pixels they knocked off less the pixels they
    /// lost (`Think::given_own`). For measuring what judging nearer the ship is worth.
    Own,
    /// Not at all: the scores are thrown away and each unit is handed a way by chance
    /// (`Think::blind`). For measuring what the looking ahead is worth.
    Blind,
    /// By the whole fight, but a think makes one change only: the one that came out furthest
    /// ahead (the first of equals). A think judges each change with the rest left as they
    /// are, and this makes only what it judged.
    Best,
    /// By the whole fight, but a think makes only the half of its changes that came out
    /// furthest ahead.
    Half,
    /// By the whole fight, but a change is made only if it came out more than `MARGIN` px
    /// ahead.
    Margin,
}

/// How far ahead a change has to come out under `Judge::Margin`. A change that comes out 10 px
/// ahead or less on one roll is worth nothing on another (MATCH.md, 2026-11-16).
pub const MARGIN: i32 = 10;

/// What a fight played forward leaves beside its score: each ship's own account of it, in the
/// order the copy holds them (the pixels it has knocked off less the pixels it has lost,
/// since it was built). It's there once the fight's answer is in.
type Each = Arc<OnceLock<Vec<i32>>>;

/// Ships that decide together: one ship, or ships of one design that are near each other.
#[derive(Clone)]
pub struct Unit {
    pub slot: usize,
    /// Its ships, as they are in the fight itself.
    pub ships: Vec<usize>,
    /// And as they are in the copy.
    at: Vec<usize>,
    /// The way most of them flew when the think began.
    pub now: Option<Style>,
    /// The other ways it could fly, and the fight being played forward with each.
    pub ways: Vec<Style>,
    trials: Vec<Arc<Job>>,
    /// And each ship's own account of each of those fights.
    each: Vec<Each>,
}

/// A side's planner in the middle of a think.
#[derive(Clone)]
pub struct Think {
    pub began: u32,
    /// The tick it's used on.
    pub due: u32,
    /// When it was begun, by the clock on the wall. Only tools read it: nothing in a match may.
    pub clock: std::time::Instant,
    /// The fight as it stood when the think began: the ships in it and nothing else. Every
    /// change is played forward from this, on the same dice.
    pub base: Arc<Arena>,
    /// The fight played forward with nothing changed.
    same: Arc<Job>,
    same_each: Each,
    pub units: Vec<Unit>,
}

impl Think {
    /// How many fights it plays forward.
    pub fn plays(&self) -> usize {
        1 + self.units.iter().map(|u| u.ways.len()).sum::<usize>()
    }

    /// Whether every answer is in.
    pub fn ready(&self) -> bool {
        self.same.ready() && self.units.iter().all(|u| u.trials.iter().all(|j| j.ready()))
    }

    /// For measuring, once every answer is in: the time all its playings forward took to work
    /// out, added up, and the longest of them, in nanoseconds.
    pub fn worked(&self) -> (u64, u64) {
        use std::sync::atomic::Ordering::Relaxed;
        let each = std::iter::once(&self.same).chain(self.units.iter().flat_map(|u| u.trials.iter())).map(|j| j.nanos.load(Relaxed));
        each.fold((0, 0), |(sum, top), n| (sum + n, top.max(n)))
    }

    /// For measuring: what each change is worth on the think's own dice, and again on each of
    /// `dice` (how the dice in the fights played forward might have fallen instead), worked
    /// out here and now. The score with nothing changed, on each roll (the think's own first);
    /// then for each unit, for each other way, that way's score less the unchanged fight's,
    /// on each roll. A change that's worth something is worth about the same every time; one
    /// that only moved the dice isn't. Then the same for each change judged on its own ships'
    /// account and not the whole fight. Waits for the answers.
    pub fn rolls(&self, team: usize, dice: &[u64]) -> (Vec<i32>, Vec<Vec<Vec<i32>>>, Vec<Vec<Vec<i32>>>) {
        let mut same = vec![self.same.get()];
        let mut whole: Vec<Vec<Vec<i32>>> = self.units.iter().map(|u| u.trials.iter().map(|j| vec![j.get() - same[0]]).collect()).collect();
        let mut own: Vec<Vec<Vec<i32>>> = self.units.iter().map(|u| u.each.iter().map(|each| vec![u.own(each) - u.own(&self.same_each)]).collect()).collect();
        for &d in dice {
            let mut base = (*self.base).clone();
            base.dice = d;
            let base = Arc::new(base);
            let (work, same_each) = play(base.clone(), team, Vec::new(), None);
            let there = work();
            same.push(there);
            for (k, u) in self.units.iter().enumerate() {
                for (w, &way) in u.ways.iter().enumerate() {
                    let (work, each) = play(base.clone(), team, u.at.clone(), Some(way));
                    whole[k][w].push(work() - there);
                    own[k][w].push(u.own(&each) - u.own(&same_each));
                }
            }
        }
        (same, whole, own)
    }

    /// For measuring: the fight played forward with every unit given its way at once (`given`,
    /// one for each unit; None: it stays as it is), on the think's own dice and then on each
    /// of `dice`: that fight's score less the unchanged fight's, on each roll. A think judges
    /// each change with everything else left as it is, and then makes them all together. This
    /// is what making them together is worth.
    pub fn together(&self, team: usize, given: &[Option<Style>], dice: &[u64]) -> Vec<i32> {
        let go = |base: &Arena, changed: bool| {
            let mut a = base.clone();
            if changed {
                for (u, way) in self.units.iter().zip(given) {
                    if way.is_some() {
                        for &i in &u.at {
                            a.ships[i].way = *way;
                        }
                    }
                }
            }
            let end = a.tick + ARCH_AHEAD;
            while a.over.is_none() && a.tick < end {
                a.step();
            }
            a.score[1 - team].px_lost - a.score[team].px_lost
        };
        std::iter::once(None)
            .chain(dice.iter().copied().map(Some))
            .map(|d| {
                let mut base = (*self.base).clone();
                if let Some(d) = d {
                    base.dice = d;
                }
                go(&base, true) - go(&base, false)
            })
            .collect()
    }

    /// For measuring: what each unit is given with the scores thrown away. With equal chance it
    /// stays as it is or takes any one of its other ways, by a number made from the tick, the
    /// side and which unit it is. How much better `given` does than this is what the looking
    /// ahead is worth.
    pub fn blind(&self, tick: u32, team: usize) -> Vec<Option<Style>> {
        self.units.iter().enumerate().map(|(k, u)| {
            let pick = Rng::new(tick as u64 * 1_000_003 + team as u64 * 7919 + k as u64).below(u.ways.len() as u32 + 1) as usize;
            pick.checked_sub(1).map(|k| u.ways[k])
        }).collect()
    }

    /// For each unit, the way that beat leaving things as they are by the most (the first
    /// tried, of equals) and how far ahead it came out. None: no way did. Waits for the answers.
    fn ahead(&self) -> Vec<Option<(Style, i32)>> {
        let same = self.same.get();
        self.units.iter().map(|u| {
            let scores: Vec<i32> = u.trials.iter().map(|j| j.get()).collect();
            (0..scores.len()).filter(|&k| scores[k] > same).max_by_key(|&k| (scores[k], std::cmp::Reverse(k))).map(|k| (u.ways[k], scores[k] - same))
        }).collect()
    }

    /// What each unit is given: the way that beat leaving things as they are by the most (the
    /// first tried, of equals). None: it stays as it is. Waits for the answers.
    pub fn given(&self) -> Vec<Option<Style>> {
        self.ahead().into_iter().map(|a| a.map(|a| a.0)).collect()
    }

    /// What each unit is given by a judge (`Judge`). The game's is `Judge::Fight`, which is
    /// `given`; the rest are for measuring.
    pub fn given_by(&self, judge: Judge, tick: u32, team: usize) -> Vec<Option<Style>> {
        let ahead = || self.ahead();
        // The changes that came out at least `bar` ahead, and no others.
        let over = |bar: i32| ahead().into_iter().map(|a| a.filter(|a| a.1 >= bar).map(|a| a.0)).collect();
        match judge {
            Judge::Fight => self.given(),
            Judge::Own => self.given_own(),
            Judge::Blind => self.blind(tick, team),
            Judge::Best => {
                let ahead = ahead();
                let top = (0..ahead.len()).filter(|&k| ahead[k].is_some()).max_by_key(|&k| (ahead[k].unwrap().1, std::cmp::Reverse(k)));
                (0..ahead.len()).map(|k| ahead[k].filter(|_| Some(k) == top).map(|a| a.0)).collect()
            }
            Judge::Half => {
                let mut by: Vec<i32> = ahead().iter().flatten().map(|a| a.1).collect();
                by.sort();
                over(by.get(by.len() / 2).copied().unwrap_or(0))
            }
            Judge::Margin => over(MARGIN + 1),
        }
    }

    /// For measuring: the same, with each change judged on its own ships' account alone: what
    /// they knocked off less what they lost with the change, against the same with nothing
    /// changed. What the change did to the rest of the fight isn't counted.
    pub fn given_own(&self) -> Vec<Option<Style>> {
        self.same.get();
        self.units.iter().map(|u| {
            let same = u.own(&self.same_each);
            let scores: Vec<i32> = u.trials.iter().zip(&u.each).map(|(j, each)| {
                j.get();
                u.own(each)
            }).collect();
            (0..scores.len()).filter(|&k| scores[k] > same).max_by_key(|&k| (scores[k], std::cmp::Reverse(k))).map(|k| u.ways[k])
        }).collect()
    }
}

impl Unit {
    /// What its ships' own accounts come to in a fight played forward whose answer is in.
    fn own(&self, each: &Each) -> i32 {
        let each = each.get().expect("a fight's accounts were asked for before its answer");
        self.at.iter().map(|&i| each[i]).sum()
    }
}

/// One fight played forward: the copy, with the ships in `set` flying `way` (none: nothing
/// changed), for `ARCH_AHEAD`. What it gives is the pixels the enemy lost in that time, less the
/// pixels this side lost. It leaves each ship's own account beside that.
fn play(base: Arc<Arena>, team: usize, set: Vec<usize>, way: Option<Style>) -> (Box<dyn FnOnce() -> i32 + Send>, Each) {
    let each = Each::default();
    let left = each.clone();
    let work = Box::new(move || {
        let mut a = (*base).clone();
        for &i in &set {
            a.ships[i].way = way;
        }
        let end = a.tick + ARCH_AHEAD;
        while a.over.is_none() && a.tick < end {
            a.step();
        }
        // (A piece that broke off in the time is a ship the copy didn't hold: it's in nobody's.)
        let _ = left.set(a.ships[..base.ships.len()].iter().map(|s| s.dealt - s.lost).collect());
        a.score[1 - team].px_lost - a.score[team].px_lost
    });
    (work, each)
}

impl Arena {
    /// The arch-AI's turn, for each side that has one. A side whose think is due uses it, a
    /// side with none in hand looks for a fight to think about, and a side in the middle of one
    /// does nothing: the thinking is done elsewhere (`pool.rs`). Blue goes first. A fight being
    /// played forward inside a think does no planning of its own: there, each side flies on
    /// with the plan it has.
    pub(super) fn arch_think(&mut self) {
        if self.rehearsal {
            return;
        }
        let clock = std::time::Instant::now();
        for team in 0..2 {
            if self.brain[team].ways != Ways::Plan {
                continue;
            }
            if let Some(t) = self.arch[team].take() {
                if self.tick < t.due && !eager(&t) {
                    self.arch[team] = Some(t);
                    continue;
                }
                self.arch_use(team, &t);
                self.arch_next[team] = (t.began + ARCH_PERIOD).max(self.tick);
            }
            if self.tick >= self.arch_next[team] {
                self.arch[team] = self.arch_begin(team);
                if self.arch[team].is_none() {
                    self.arch_next[team] = self.tick + ARCH_IDLE;
                }
            }
        }
        ARCH_NANOS.fetch_add(clock.elapsed().as_nanos() as u64, std::sync::atomic::Ordering::Relaxed);
    }

    /// How ship `i` flies at what it fights: the way its side's planner gave it, or gave most
    /// of its design, or else what its drawing suggests.
    pub fn flies(&self, i: usize) -> Option<Style> {
        self.planned(i).or(Style::of(self.ships[i].st.flight))
    }

    /// What its side's planner has decided for ship `i`: for the ship itself, or else for most
    /// of its design the last time it thought about them. None: nothing yet.
    pub fn planned(&self, i: usize) -> Option<Style> {
        let s = &self.ships[i];
        s.way.or(self.plan[s.team].get(s.slot).copied().flatten())
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
        self.pocket_at(awake).0
    }

    /// The same, with where each ship is in the copy (None: it isn't in it).
    pub fn pocket_at(&self, awake: &[bool]) -> (Arena, Vec<Option<usize>>) {
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
            s.last_by = moved(s.last_by);
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
            shot.from = moved(shot.from);
            a.shots.push(shot);
        }
        a.tick = self.tick;
        a.started = [a.alive(0) > 0, a.alive(1) > 0];
        (a.brain, a.plan) = (self.brain, self.plan.clone());
        a.rehearsal = true;
        (a, at)
    }

    /// A ship that chooses how it flies: a warship of the side that's whole, armed, and has
    /// engines. (Not a miner, armed or no: it mines, and in the copy it stays where it is.)
    pub fn chooses(&self, i: usize, team: usize) -> bool {
        let s = &self.ships[i];
        s.team == team && s.role == Role::Ship && s.grow.is_none() && s.maker.is_none() && s.st.armed() && s.st.flight != Flight::Still && !s.miner()
    }

    /// The guns in a fight: every gun pixel and turret of every ship standing in it.
    pub fn guns(&self) -> usize {
        self.ships.iter().filter(|s| s.alive).map(|s| (s.st.flak + s.st.lance + s.st.cannon + s.st.tracker + s.st.beam + s.st.repeater + s.st.missile) as usize).sum()
    }

    /// A side's ships in a fight (`mine`, each one that chooses), in the units that decide
    /// together. A round of decisions should take about `ARCH_ROUND` ticks on the machine
    /// planned for, so the bigger the fight, the fewer the units: playing the fight forward
    /// once takes `one` work, and a think plays it once with nothing changed and about twice
    /// for each unit. With time for a unit a ship, each ship decides for itself.
    /// Otherwise the units are squads: each design's ships, cut into as many squads as its share
    /// of the units comes to, along the line they're most spread out on, so a squad's ships are
    /// near each other. A side never thinks coarser than a design at a time.
    pub fn units(&self, mine: &[usize], one: i64) -> Vec<Vec<usize>> {
        let mut slots: Vec<usize> = mine.iter().map(|&i| self.ships[i].slot).collect();
        slots.sort();
        slots.dedup();
        let most = match self.grain {
            Grain::Designs => slots.len(),
            Grain::Fine => {
                let plays = ARCH_ROUND as i64 * ARCH_SIDE / one.max(1);
                (((plays - 1) / 2).max(0) as usize).max(slots.len())
            }
        };
        if mine.len() <= most {
            return mine.iter().map(|&i| vec![i]).collect();
        }
        let by_design: Vec<Vec<usize>> = slots.iter().map(|&s| mine.iter().copied().filter(|&i| self.ships[i].slot == s).collect()).collect();
        // A squad each, and each squad more to the design whose squads are biggest.
        let mut squads = vec![1usize; slots.len()];
        for _ in slots.len()..most {
            let d = (0..slots.len()).max_by_key(|&d| (by_design[d].len() * 1000 / squads[d], std::cmp::Reverse(d))).unwrap();
            if by_design[d].len() <= squads[d] {
                break;
            }
            squads[d] += 1;
        }
        let mut units = Vec::new();
        for (d, ships) in by_design.iter().enumerate() {
            let spread = |f: fn(&Ship) -> i32| {
                let v: Vec<i32> = ships.iter().map(|&i| f(&self.ships[i])).collect();
                v.iter().max().unwrap() - v.iter().min().unwrap()
            };
            let mut line = ships.clone();
            if spread(|s| s.x) >= spread(|s| s.y) {
                line.sort_by_key(|&i| (self.ships[i].x, self.ships[i].y, i));
            } else {
                line.sort_by_key(|&i| (self.ships[i].y, self.ships[i].x, i));
            }
            let k = squads[d];
            for j in 0..k {
                let mut squad = line[j * line.len() / k..(j + 1) * line.len() / k].to_vec();
                squad.sort();
                units.push(squad);
            }
        }
        units
    }

    /// Begins a think for a side, if it has ships in a fight with a choice of how to fly: the
    /// fight as it stands is copied, and handed over to be played forward once as it is and
    /// once for each other way each unit could fly.
    fn arch_begin(&mut self, team: usize) -> Option<Box<Think>> {
        let awake = self.in_a_fight();
        let mine: Vec<usize> = (0..self.ships.len()).filter(|&i| awake[i] && self.chooses(i, team)).collect();
        if mine.is_empty() {
            return None;
        }
        let (mut base, at) = self.pocket_at(&awake);
        // The dice are rolled afresh, so a plan can't lean on knowing how they'll fall.
        base.dice = Rng::keyed([self.dice, self.tick as u64, team as u64, 0]).next_u64();
        let one = arch_work(base.ships.len(), base.guns());
        let base = Arc::new(base);
        let (work, same_each) = play(base.clone(), team, Vec::new(), None);
        let same = submit(work);
        let mut units = Vec::new();
        for ships in self.units(&mine, one) {
            let flown: Vec<Option<Style>> = ships.iter().map(|&i| self.flies(i)).collect();
            // The way most of them fly now (the first of equals, as `STYLES` lists them).
            let now = STYLES.iter().copied().map(Some).chain([None]).max_by_key(|w| (flown.iter().filter(|f| *f == w).count(), std::cmp::Reverse(w.map_or(3, |w| STYLES.iter().position(|&s| s == w).unwrap())))).flatten();
            // Each way that isn't how all of them fly already.
            let ways: Vec<Style> = STYLES.iter().copied().filter(|&w| flown.iter().any(|&f| f != Some(w))).collect();
            let copies: Vec<usize> = ships.iter().map(|&i| at[i].expect("a ship that chooses isn't in the copy of its fight")).collect();
            let (trials, each) = ways.iter().map(|&w| play(base.clone(), team, copies.clone(), Some(w))).map(|(work, each)| (submit(work), each)).unzip();
            units.push(Unit { slot: self.ships[ships[0]].slot, ships, at: copies, now, ways, trials, each });
        }
        let mut think = Think { began: self.tick, due: 0, clock: std::time::Instant::now(), base, same, same_each, units };
        think.due = self.tick + arch_wait(one, think.plays());
        Some(Box::new(think))
    }

    /// Uses a think: each unit takes the way that beat leaving things as they are by the most
    /// (the first tried, of equals), if any did. Whatever a ship flies after that is its own:
    /// a ship that stays as it was is given the way it was flying, so what's decided for its
    /// design later, for other ships, doesn't move it. Then each design's ships that weren't in
    /// the think (newcomers, and ships elsewhere) are given the way most of those in it now fly.
    fn arch_use(&mut self, team: usize, t: &Think) {
        let given = t.given_by(self.judge[team], self.tick, team);
        let same = t.same.get();
        for (u, &best) in t.units.iter().zip(&given) {
            for &i in &u.ships {
                // (The ship is the one it was: nothing ever takes a ship's place in the list.)
                if self.ships[i].alive && self.ships[i].team == team && self.ships[i].slot == u.slot {
                    self.ships[i].way = best.or(self.flies(i));
                }
            }
            ARCH_CHOICES.with(|c| {
                if let Some(v) = c.borrow_mut().as_mut() {
                    let ways = std::iter::once(u.now).chain(u.ways.iter().map(|&w| Some(w))).collect();
                    let scores = std::iter::once(same).chain(u.trials.iter().map(|j| j.get())).collect();
                    v.push(Choice { team, slot: u.slot, ways, scores, kept: best.or(u.now), ships: t.base.ships.len(), members: u.ships.len(), took: self.tick - t.began });
                }
            });
        }
        let mut slots: Vec<usize> = t.units.iter().map(|u| u.slot).collect();
        slots.sort();
        slots.dedup();
        for slot in slots {
            let flown: Vec<Option<Style>> = t.units.iter().filter(|u| u.slot == slot).flat_map(|u| u.ships.iter()).filter(|&&i| self.ships[i].alive).map(|&i| self.ships[i].way).collect();
            let count = |w: Style| flown.iter().filter(|f| **f == Some(w)).count();
            let top = STYLES.iter().map(|&w| count(w)).max().unwrap_or(0);
            let tops: Vec<Style> = STYLES.iter().copied().filter(|&w| count(w) == top).collect();
            // (Two ways with as many ships each: the design's way stays as it was.)
            if top > 0 && tops.len() == 1 {
                if self.plan[team].len() <= slot {
                    self.plan[team].resize(slot + 1, None);
                }
                self.plan[team][slot] = Some(tops[0]);
            }
        }
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

/// Whether a think may be used before it's due: never, outside a test that sets the fault.
fn eager(t: &Think) -> bool {
    #[cfg(test)]
    {
        if ARCH_EAGER.get() {
            return t.ready();
        }
    }
    let _ = t;
    false
}

#[cfg(test)]
mod tests {
    use super::super::pool::{Mode, MODE};
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

    /// So many fighters a side, in blocks 250 px apart on a field big enough for them, each
    /// side with a planner.
    fn fighters(each: i32) -> Arena {
        let d = starter_deck();
        let mut a = Arena::blank(1200, 800, 1);
        for k in 0..each {
            a.spawn(0, 0, &d[0], 100 + 12 * (k % 10), 100 + 12 * (k / 10), 1, false);
            a.spawn(1, 0, &d[0], 350 + 12 * (k % 10), 100 + 12 * (k / 10), 3, false);
        }
        for s in a.ships.iter_mut() {
            s.order = Order::Hunt;
        }
        a.brain = [Brain::ARCH; 2];
        a
    }

    /// Steps to the tick blue's first think is used on, and gives the think.
    fn first_think(a: &mut Arena) -> Think {
        a.step();
        let think = *a.arch[0].clone().expect("blue began no think on the first tick");
        while a.tick < think.due {
            a.step();
        }
        think
    }

    #[test]
    fn a_think_plays_each_change_forward_from_the_same_start_and_keeps_what_beat_leaving_things_be() {
        let mut a = fight([Brain::ARCH, Brain::PLAIN]);
        let think = first_think(&mut a);
        // Seven ships in the fight: each lancer decides for itself, and the ways it could fly
        // are the two it doesn't fly now (as drawn, it makes attack runs).
        assert_eq!(think.base.ships.len(), 7);
        assert_eq!(think.units.iter().map(|u| (u.slot, u.ships.clone(), u.now, u.ways.clone())).collect::<Vec<_>>(), (0..6).map(|i| (1, vec![i], Some(Style::Runs), vec![Style::Kite, Style::Orbit])).collect::<Vec<_>>());
        assert_eq!(think.plays(), 13);
        // Each score is what playing the start forward with that change gives: the same start,
        // the same dice.
        let by_hand = |set: &[usize], way: Option<Style>| {
            let mut b = (*think.base).clone();
            for &i in set {
                b.ships[i].way = way;
            }
            let end = b.tick + ARCH_AHEAD;
            while b.over.is_none() && b.tick < end {
                b.step();
            }
            b.score[1].px_lost - b.score[0].px_lost
        };
        let same = by_hand(&[], None);
        assert_eq!(think.same.get(), same);
        let mut told_apart = false;
        for u in &think.units {
            let scores: Vec<i32> = u.ways.iter().map(|&w| by_hand(&u.at, Some(w))).collect();
            assert_eq!(u.trials.iter().map(|j| j.get()).collect::<Vec<_>>(), scores);
            told_apart |= scores.iter().any(|&s| s != same);
            // The ship flies the way that beat leaving things be by the most, the first of
            // equals; if none did, the way it flew, now its own.
            let best = (0..2).filter(|&k| scores[k] > same).max_by_key(|&k| (scores[k], std::cmp::Reverse(k)));
            assert_eq!(a.ships[u.ships[0]].way, Some(best.map_or(Style::Runs, |k| u.ways[k])), "the lancer's two ways scored {scores:?} against {same}");
        }
        assert!(told_apart, "every change scored {same}: the fight tells nothing apart");
        // A side with no arch-AI plans nothing.
        assert!(a.arch[1].is_none() && a.plan[1].is_empty() && a.ships[6].way.is_none());
    }

    #[test]
    fn a_think_is_used_on_a_tick_fixed_by_the_size_of_the_fight_and_never_sooner() {
        // A fight's work: for each ship, so much for each gun in the fight and so much besides.
        // Forty fighters have four flak each.
        assert_eq!((arch_work(7, 22), arch_work(40, 160), arch_work(300, 1200)), (1_192_800, 13_440_000, 475_200_000));
        // Time for every playing forward on a side's share of the machine planned for, and for
        // the longest of them on one core; never under two ticks.
        assert_eq!((arch_wait(1_192_800, 13), arch_wait(13_440_000, 35), arch_wait(475_200_000, 3), arch_wait(475_200_000, 1)), (2, 15, 44, 22));
        // Twenty fighters a side: forty ships in seventeen units, due in fifteen ticks. The
        // answers are all in long before that (the workers have nothing else to do, and the
        // test waits for them), and nothing is used until the tick it's due.
        let mut a = fighters(20);
        a.step();
        let think = *a.arch[0].clone().unwrap();
        assert_eq!((think.base.ships.len(), think.base.guns(), think.units.len(), think.plays(), think.due - think.began), (40, 160, 17, 35, 15));
        while !think.ready() {
            std::thread::sleep(std::time::Duration::from_millis(1));
        }
        while a.tick + 1 < think.due {
            a.step();
            assert!(a.ships.iter().all(|s| s.way.is_none()) && a.plan[0].is_empty(), "a think was used at tick {}, {} before it was due", a.tick, think.due - a.tick);
            assert_eq!(a.arch[0].as_ref().map(|t| t.began), Some(think.began));
        }
        a.step();
        // On the tick it's due, every blue fighter in it has a way of its own, and the next
        // think has begun.
        assert!(think.units.iter().all(|u| u.ships.iter().all(|&i| a.ships[i].way.is_some())));
        assert_eq!(a.arch[0].as_ref().map(|t| t.began), Some(think.due));
    }

    #[test]
    fn each_ship_decides_for_itself_in_a_small_fight_squads_in_a_big_one_and_a_design_at_a_time_in_the_biggest() {
        let sizes = |a: &Arena| {
            let mut v: Vec<usize> = a.arch[0].as_ref().unwrap().units.iter().map(|u| u.ships.len()).collect();
            v.sort();
            v
        };
        // Ten a side: each for itself.
        let mut a = fighters(10);
        a.step();
        assert_eq!(sizes(&a), vec![1; 10]);
        // Fifty a side, a hundred in the fight with four hundred guns: a round has the time
        // for seven playings forward, so three units: squads of sixteen or seventeen fighters,
        // each squad next to each other along the line the fifty are most spread out on (they
        // stand ten wide and five deep).
        let mut a = fighters(50);
        a.step();
        assert_eq!(sizes(&a), vec![16, 17, 17]);
        let t = a.arch[0].as_ref().unwrap();
        let reach: Vec<(i32, i32)> = t.units.iter().map(|u| (u.ships.iter().map(|&i| a.ships[i].x).min().unwrap(), u.ships.iter().map(|&i| a.ships[i].x).max().unwrap())).collect();
        for w in reach.windows(2) {
            assert!(w[0].1 <= w[1].0, "two squads overlap along the line: {reach:?}");
        }
        assert_eq!(t.due - t.began, arch_wait(arch_work(100, 400), 7));
        // Two designs among the fifty: two squads each.
        let d = starter_deck();
        let mut a = fighters(25);
        for k in 0..25 {
            a.spawn(0, 1, &d[1], 100 + 12 * (k % 10), 200 + 12 * (k / 10), 1, false);
            a.spawn(1, 1, &d[1], 350 + 12 * (k % 10), 200 + 12 * (k / 10), 3, false);
        }
        a.step();
        let t = a.arch[0].as_ref().unwrap();
        assert_eq!(t.units.iter().map(|u| (u.slot, u.ships.len())).collect::<Vec<_>>(), vec![(0, 12), (0, 13), (1, 12), (1, 13)]);
        // A hundred and fifty a side: no time for more than the design at a time it never
        // thinks coarser than.
        let mut a = fighters(150);
        a.step();
        assert_eq!(sizes(&a), vec![150]);
        // Told to think a design at a time, it does in a small fight too.
        let mut a = fighters(10);
        a.grain = Grain::Designs;
        a.step();
        assert_eq!(sizes(&a), vec![10]);
    }

    /// The fight of forty fighters played `ticks` ticks: its checksum after every tick. With
    /// `early`, the game waits before each tick until every answer of the thinks in hand is in,
    /// as on a machine far faster than the one planned for.
    fn checksums(mode: Mode, ticks: u32, early: bool) -> Vec<u64> {
        MODE.set(mode);
        let mut a = fighters(20);
        let mut sums = Vec::new();
        for _ in 0..ticks {
            a.step();
            sums.push(a.checksum());
            while early && a.arch.iter().flatten().any(|t| !t.ready()) {
                std::thread::sleep(std::time::Duration::from_millis(1));
            }
        }
        MODE.set(Mode::Pool);
        sums
    }

    #[test]
    fn a_fight_plays_out_the_same_with_no_workers_with_many_and_with_slow_ones() {
        // Every think worked out by the game's own thread on the tick it's due; by the workers,
        // with the game not waiting for them (so some answers are in early and some are waited
        // for); by workers held up, so the game waits; and by workers with the game going
        // slowly, so every answer is in early. Tick for tick the same fight.
        let inline = checksums(Mode::Inline, 64, false);
        assert_eq!(checksums(Mode::Pool, 64, false), inline, "the workers gave another fight");
        assert_eq!(checksums(Mode::Slow, 64, false), inline, "slow workers gave another fight");
        assert_eq!(checksums(Mode::Pool, 64, true), inline, "answers that were in early gave another fight");
        // And the fault this guards against is one it catches: thinks used the moment they're
        // ready, on a machine whose answers are in early, give another fight.
        ARCH_EAGER.set(true);
        let eager = checksums(Mode::Pool, 64, true);
        ARCH_EAGER.set(false);
        assert_ne!(eager, inline, "thinks used as soon as they were ready went unnoticed");
    }

    #[test]
    fn a_copy_of_a_fight_made_in_the_middle_of_a_think_plays_on_the_same() {
        let mut a = fighters(20);
        a.step();
        a.step();
        assert!(a.arch[0].is_some() && a.arch[1].is_some());
        let mut b = a.clone();
        for _ in 0..24 {
            a.step();
            b.step();
            assert_eq!(a.checksum(), b.checksum());
        }
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
        // rock, and nobody's think. The far fighter decides nothing.
        let think = a.arch[0].clone().unwrap();
        assert_eq!(think.base.ships.len(), 7);
        assert!(think.base.ships.iter().all(|s| s.x < 400) && think.base.eco.is_none() && think.base.rock.is_empty());
        assert!(think.base.arch[0].is_none() && think.base.arch[1].is_none() && think.base.rehearsal);
        assert!(think.units.iter().all(|u| u.ships.iter().all(|&i| i < 6)));
        // Played forward, the seven fight on: they move, and somebody loses pixels.
        let mut b = (*think.base).clone();
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
        // The miner stays where it is there: it has no rock to work. It decides nothing (it's
        // armed and flies, but it mines), and nor does the fighter still being printed: the six
        // warships each decide for themselves.
        let m = base.ships.iter().find(|s| s.slot == 3).unwrap();
        assert_eq!((m.order, m.home, m.ring0), (Order::Hold, (m.x, m.y), 0));
        assert_eq!(think.units.iter().map(|u| u.ships.clone()).collect::<Vec<_>>(), fight[..6].iter().map(|&i| vec![i]).collect::<Vec<_>>());
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
        // Every warship that's been in a think has a way of its own by now, the fights played
        // forward having run their 8 s, late as the match's clock is. The miner and the kings
        // have none, and no design has a way but the fighters', the lancers' and the frigate's.
        for _ in 0..15 * 20 {
            a.step();
        }
        assert!(fight.iter().filter(|&&i| a.ships[i].alive).all(|&i| a.ships[i].way.is_some()), "a warship twenty seconds into a fight with no way of its own");
        assert!(a.ships[miner].way.is_none() && kings.iter().all(|&k| a.ships[k].way.is_none()));
        assert!(a.plan[0].len() <= 2 && a.plan[1].len() <= 3, "a way for a design that isn't in the fight, or for the miner: {:?}", a.plan);
        // What the bar says of a choice fits the bar.
        for way in STYLES {
            assert!(chosen_line(way).len() <= 49, "{}", chosen_line(way));
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
            assert!(a.arch[0].is_none() && a.arch[1].is_none() && a.plan[0].is_empty() && a.plan[1].is_empty() && a.ships.iter().all(|s| s.way.is_none()));
        }
    }

    #[test]
    fn what_one_ship_does_leaves_the_dice_of_a_fight_elsewhere_alone() {
        // Two skirmishes of fighters too far apart to meet in the time, nobody planning. Played
        // once as they are, and once with one fighter of the first skirmish kiting. The first
        // skirmish goes another way. In the second, every ship ends where it ended before, with
        // the same pixels left and its guns as far through their reloads. (On dice the whole
        // fight shared, the first skirmish's shots moved the second's rolls along: its flak
        // aimed at other pixels, and it came out another fight.)
        let d = starter_deck();
        let played = |way: Option<Style>| {
            let mut a = Arena::blank(1200, 800, 1);
            for y in [100, 700] {
                for k in 0..3 {
                    a.spawn(0, 0, &d[0], 100, y + 12 * k, 1, false);
                    a.spawn(1, 0, &d[0], 250, y + 12 * k, 3, false);
                }
            }
            for s in a.ships.iter_mut() {
                s.order = Order::Hunt;
            }
            a.ships[0].style = way;
            for _ in 0..ARCH_AHEAD {
                a.step();
            }
            let seen: Vec<_> = a.ships.iter().map(|s| (s.x, s.y, s.vx, s.vy, s.facing, s.hp.clone(), s.cool.clone())).collect();
            (seen[..6].to_vec(), seen[6..].to_vec())
        };
        let (near, far) = played(None);
        let (near_kiting, far_kiting) = played(Some(Style::Kite));
        assert!(far.iter().any(|s| s.5.contains(&0)), "the far skirmish knocked no pixel off, so it rolled no dice worth the name");
        assert!(near != near_kiting, "the kiting fighter changed nothing in its own skirmish");
        assert!(far == far_kiting, "a fighter kiting in one skirmish changed how another, 600 px away, went");
    }

    #[test]
    fn the_same_dice_pick_the_same_pixel_until_it_goes_and_every_pixel_is_as_likely() {
        let d = starter_deck();
        let mut a = Arena::new();
        assert!(a.add(1, 2, &d[2], 240, 180));
        let standing = |a: &Arena| (0..a.ships[0].hp.len()).filter(|&c| a.ships[0].hp[c] > 0).count();
        let n = standing(&a);
        // As likely: on 200 rolls for every pixel of the frigate, each pixel is picked, and
        // none under half as often as its share or over twice as often.
        let mut picked = std::collections::HashMap::new();
        for k in 0..200 * n as u64 {
            *picked.entry(a.random_cell(Rng::keyed([k, 0, 0, 0]), 0)).or_insert(0) += 1;
        }
        assert_eq!(picked.len(), n);
        assert!(picked.values().all(|&times| times > 100 && times < 400), "{:?}", picked.values().collect::<Vec<_>>());
        // The same until it goes: fifty rolls, and the frigate taken apart a pixel at a time,
        // each time a pixel one of the rolls picks. Every roll whose pixel still stands picks
        // it again, and every roll whose pixel went picks one that stands.
        let rolls = |a: &Arena| (0..50u64).map(|k| a.random_cell(Rng::keyed([k, 1, 0, 0]), 0)).collect::<Vec<_>>();
        let mut moved = 0;
        while standing(&a) > 1 {
            let before = rolls(&a);
            let gone = before[a.tick as usize % 50];
            let c = a.ships[0].cell_at(gone.0, gone.1).unwrap();
            a.kill_cell(0, c, gone, (0, 1));
            a.tick += 1;
            for (was, now) in before.iter().zip(rolls(&a)) {
                if *was == gone {
                    assert!(now != gone && a.ships[0].cell_at(now.0, now.1).is_some_and(|c| a.ships[0].hp[c] > 0));
                    moved += 1;
                } else {
                    assert_eq!(*was, now, "a roll moved to another pixel when a pixel it hadn't picked went");
                }
            }
        }
        assert!(moved > 50);
    }

    #[test]
    fn the_measuring_judges_make_some_of_the_games_changes_and_no_others() {
        // Twelve fighters a side: each decides for itself, and the first think gives several a
        // new way. The judges the tools fight beside the game's make some of those changes and
        // nothing else: the one furthest ahead, the half furthest ahead, those over the margin.
        let mut a = fighters(12);
        let think = first_think(&mut a);
        let (ahead, all) = (think.ahead(), think.given());
        let made = |g: &[Option<Style>]| g.iter().filter(|w| w.is_some()).count();
        assert!(made(&all) >= 4, "the think gives {} units a new way: too few to tell the judges apart", made(&all));
        let within = |g: &[Option<Style>]| g.iter().zip(&all).all(|(w, of)| w.is_none() || w == of);
        let top = ahead.iter().flatten().map(|a| a.1).max().unwrap();
        let best = think.given_by(Judge::Best, a.tick, 0);
        assert!(within(&best) && made(&best) == 1);
        assert_eq!(best.iter().position(|w| w.is_some()), ahead.iter().position(|a| a.is_some_and(|a| a.1 == top)));
        let half = think.given_by(Judge::Half, a.tick, 0);
        assert!(within(&half) && made(&half) >= made(&all) / 2 && made(&half) < made(&all), "{} of {}", made(&half), made(&all));
        // Every change the half leaves out came out no further ahead than any it makes.
        let (mut kept, mut left) = (Vec::new(), Vec::new());
        for (w, a) in half.iter().zip(&ahead) {
            if let Some(a) = a {
                if w.is_some() { kept.push(a.1) } else { left.push(a.1) }
            }
        }
        assert!(left.iter().max() <= kept.iter().min());
        let margin = think.given_by(Judge::Margin, a.tick, 0);
        assert!(within(&margin));
        for (w, a) in margin.iter().zip(&ahead) {
            assert_eq!(w.is_some(), a.is_some_and(|a| a.1 > MARGIN));
        }
        assert_eq!(think.given_by(Judge::Fight, a.tick, 0), all);
    }

    #[test]
    fn a_sides_score_is_the_sum_of_its_ships_own_accounts() {
        // The parts library's mixed fleet against its twin (flak, lances, shells, missiles,
        // beams, repeaters), and the starter deck's against its own, each fought to the end on
        // four rolls of the dice. Every pixel a side lost is in one of its ships' `lost`, and
        // in one enemy ship's `dealt`: also the pixels that broke away once what held them was
        // gone, and the pixels of a ship that burst. (No ship in either deck has two cores, so
        // none is cut into two ships here.)
        let mut broken = 0;
        for lib in [crate::pg::parts_library(), starter_deck()] {
            let fleet = crate::pg::fliers(&lib);
            let budget = fleet.iter().map(|&s| lib[s].stats().cost).max().unwrap();
            for seed in 1..=4 {
                let mut a = Arena::seeded(seed);
                for (team, slot, x, y) in crate::pg::fleet_side(&lib, 0, &fleet, budget).into_iter().chain(crate::pg::fleet_side(&lib, 1, &fleet, budget)) {
                    a.add(team, slot, &lib[slot], x, y);
                }
                while a.over.is_none() {
                    a.step();
                }
                broken += a.score[0].broken + a.score[1].broken;
                for t in 0..2 {
                    let lost: i32 = a.ships.iter().filter(|s| s.team == t).map(|s| s.lost).sum();
                    let dealt: i32 = a.ships.iter().filter(|s| s.team != t).map(|s| s.dealt).sum();
                    assert!(a.score[t].px_lost > 100);
                    assert_eq!((lost, dealt), (a.score[t].px_lost, a.score[t].px_lost), "seed {seed}, side {t}");
                }
            }
        }
        assert!(broken > 0, "no pixel broke away: the fights don't try it");
    }

    #[test]
    fn a_ship_flies_its_own_way_then_its_designs_then_its_drawings() {
        // The thinking switched off, as it is in a fight played forward. The lancers' design has
        // them orbiting; one has a way of its own, kiting; one has a way forced on it by a test,
        // attack runs, which comes before both.
        let mut a = fight([Brain::ARCH, Brain::PLAIN]);
        a.rehearsal = true;
        a.plan[0] = vec![None, Some(Style::Orbit)];
        a.ships[0].way = Some(Style::Kite);
        (a.ships[1].way, a.ships[1].style) = (Some(Style::Kite), Some(Style::Runs));
        assert_eq!((a.flies(0), a.flies(2), a.planned(0), a.planned(2), a.planned(6)), (Some(Style::Kite), Some(Style::Orbit), Some(Style::Kite), Some(Style::Orbit), None));
        for _ in 0..30 {
            a.step();
        }
        let doing: Vec<Doing> = a.ships.iter().map(|s| s.doing).collect();
        assert!(a.ships[0].doing != Doing::Orbit && a.ships[1].doing != Doing::Orbit, "{doing:?}");
        assert!(a.ships[2..6].iter().all(|s| s.doing == Doing::Orbit), "{doing:?}");
        // The same on a side with no arch-AI does nothing: lancers fly attack runs.
        let mut b = fight([Brain::PLAIN, Brain::PLAIN]);
        b.plan[0] = vec![None, Some(Style::Orbit)];
        b.ships[0].way = Some(Style::Orbit);
        for _ in 0..30 {
            b.step();
        }
        assert!(b.ships.iter().filter(|s| s.team == 0).all(|s| s.doing != Doing::Orbit));
    }

    #[test]
    fn a_design_takes_the_way_most_of_its_ships_in_the_think_now_fly_for_its_newcomers() {
        // Six lancers, each deciding for itself. After the think, the design's way is the one
        // most of the six fly, if one way has the most, and a lancer that arrives then, with no
        // way of its own, flies it.
        let mut a = fight([Brain::ARCH, Brain::PLAIN]);
        let think = first_think(&mut a);
        let flown: Vec<Style> = think.units.iter().map(|u| a.ships[u.ships[0]].way.unwrap()).collect();
        let count = |w: Style| flown.iter().filter(|&&f| f == w).count();
        let top = STYLES.iter().map(|&w| count(w)).max().unwrap();
        let tops: Vec<Style> = STYLES.iter().copied().filter(|&w| count(w) == top).collect();
        let want = if tops.len() == 1 { Some(tops[0]) } else { None };
        assert_eq!(a.plan[0].get(1).copied().flatten(), want, "the six fly {flown:?}");
        let d = starter_deck();
        let late = a.spawn(0, 1, &d[1], 100, 100, 1, false);
        assert_eq!((a.ships[late].way, a.flies(late)), (None, want.or(Some(Style::Runs))));
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
        assert!(a.plan == plan && a.ships.iter().all(|s| s.way.is_none()));
    }

    #[test]
    fn two_fights_with_arch_ais_play_out_the_same() {
        let run = || {
            let mut a = fight([Brain::ARCH, Brain::ARCH]);
            for _ in 0..100 {
                a.step();
            }
            (a.checksum(), a.plan.clone(), a.ships.iter().map(|s| s.way).collect::<Vec<_>>())
        };
        assert_eq!(run(), run());
    }
}
