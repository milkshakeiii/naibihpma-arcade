//! CONDUIT: gear up a crew of monsters part by part (held, worn and inner slots, within each
//! body's body and mind capacity), see the rival crew, then watch the contest for the conduits.

pub mod art;
pub mod battle;
pub mod data;
mod render;

use crate::app::{Ev, Key, Out, Run, Summary};
use crate::audio::{Sfx, Track};
use crate::form::Question;
use crate::framebuffer::Framebuffer;
use crate::palette::Pal;
use crate::rng::Rng;
use crate::ui::Rect;
use battle::*;
use data::*;

pub const VERSION: u32 = 1;
pub const LESSON_VERSION: u32 = 1;
pub static FORM_QUESTION: Question = Question { id: "gearing", text: "GEARING UP YOUR CREW WAS...", low: "BUSYWORK", high: "THE BEST PART" };

pub const MAX_CREW: usize = 4;
pub const FUSES: i32 = 3;

// Workshop layout.
pub fn card_rect(i: usize) -> Rect {
    Rect::new(4 + (i % 2) as i32 * 122, 18 + (i / 2) as i32 * 96, 118, 92)
}
pub fn slot_rect(i: usize, s: usize) -> Rect {
    let c = card_rect(i);
    Rect::new(c.x + 3, c.y + 47 + s as i32 * 10, c.w - 6, 10)
}
pub const SHELF: Rect = Rect::new(248, 18, 148, 188);
pub fn shelf_rect(k: usize) -> Rect {
    Rect::new(SHELF.x + 3, SHELF.y + 13 + k as i32 * 11, SHELF.w - 6, 11)
}
pub const SHELF_ROWS: usize = 15;
pub const INFO: Rect = Rect::new(4, 210, 262, 88);
pub const RIVAL_BOX: Rect = Rect::new(270, 210, 126, 60);
pub fn rival_rect(k: usize) -> Rect {
    Rect::new(RIVAL_BOX.x + 4 + k as i32 * 30, RIVAL_BOX.y + 24, 28, 32)
}
pub const FIGHT_BTN: Rect = Rect::new(270, 274, 126, 22);

// Battle layout.
pub const AT: i32 = 24;
pub const AX: i32 = 20;
pub const AY: i32 = 18;
pub const SPEED_BTN: Rect = Rect::new(276, 270, 56, 24);
pub const SKIP_BTN: Rect = Rect::new(336, 270, 56, 24);

pub fn reward_rect(i: usize) -> Rect {
    Rect::new(24, 74 + i as i32 * 56, 352, 50)
}
pub const OVER_GO: Rect = Rect::new(140, 262, 120, 22);

#[derive(Clone, Debug, PartialEq)]
pub enum Reward {
    Part(Part),
    Monster(Body),
    Crate(Vec<Part>),
}

#[derive(Clone, Debug, PartialEq)]
pub enum Phase {
    Workshop,
    Battle,
    After { outcome: Outcome, t: u32 },
    Reward { offers: Vec<Reward> },
    Over { won: bool },
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum From {
    Shelf(usize),
    Slot(usize, usize),
}

#[derive(Clone, Debug)]
pub struct Rival {
    pub name: &'static str,
    pub text: &'static str,
    pub crew: Vec<Build>,
}

struct Theme {
    name: &'static str,
    text: &'static str,
    bodies: &'static [Body],
    parts: &'static [Part],
}

const THEMES: [Theme; 5] = [
    Theme {
        name: "THE TIN CANS",
        text: "HEAVY ARMOR AND SHORT REACH.",
        bodies: &[Body::Orc, Body::Cyclops, Body::Troll],
        parts: &[Part::Club, Part::Shield, Part::Leather, Part::Plate, Part::Spear, Part::SecondHeart],
    },
    Theme {
        name: "THE QUICKSILVER CLUB",
        text: "CLOAKS, SLINGS AND BOWS.",
        bodies: &[Body::Kobold, Body::Elf],
        parts: &[Part::Sling, Part::Bow, Part::Cloak, Part::Boots, Part::QuickNerves, Part::HunterMind],
    },
    Theme {
        name: "THE HEX SOCIETY",
        text: "WARDS, WANDS AND MENDING.",
        bodies: &[Body::Elf, Body::Kobold],
        parts: &[Part::SparkWand, Part::MendingStaff, Part::WardCharm, Part::Cloak, Part::Fang],
    },
    Theme {
        name: "THE RAIDING PARTY",
        text: "FAST FEET, STRAIGHT FOR YOUR CONDUIT.",
        bodies: &[Body::Kobold, Body::Orc],
        parts: &[Part::Club, Part::Boots, Part::RaiderMind, Part::Leather, Part::Spear],
    },
    Theme {
        name: "THE BRUISERS",
        text: "BIG BODIES, AXES AND SPIKES.",
        bodies: &[Body::Cyclops, Body::Troll, Body::Orc],
        parts: &[Part::Axe, Part::Spikes, Part::StoneGut, Part::SecondHeart, Part::Torch],
    },
];

#[derive(Default, Clone)]
pub struct Stats {
    pub won: u32,
    pub lost: u32,
    pub drawn: u32,
}

pub const LESSON_STEPS: [&str; 3] = [
    "THIS IS YOUR CREW. GEAR GOES IN SLOTS: H HELD, W WORN, I INNER. DRAG THE CLUB FROM THE SPARE PARTS ONTO THE ORC'S H SLOT.",
    "EACH PART USES SOME OF A MONSTER'S BODY (B) OR MIND (M). THE BARS ON EACH CARD SHOW WHAT'S LEFT. NOW GEAR UP THE KOBOLD.",
    "POINT AT THE RIVALS (BOTTOM RIGHT) TO SEE THEIR GEAR. PRESS FIGHT WHEN YOU'RE READY.",
];

pub struct Game {
    pub phase: Phase,
    pub crew: Vec<Build>,
    pub shelf: Vec<Part>,
    pub fuses: i32,
    pub contest: usize,
    pub rival: Rival,
    pub battle: Option<Battle>,
    pub speed: u32,
    pub held: Option<(Part, From)>,
    pub press_at: (i32, i32),
    pub mouse: (i32, i32),
    pub hover_item: Option<usize>,
    pub lesson: bool,
    pub lesson_step: usize,
    pub note: Option<String>,
    pub tips: Vec<&'static str>,
    pub message: Option<String>,
    pub rng: Rng,
    pub art: art::Art,
    pub pals: Vec<Pal>,
    pub stats: Stats,
    pub ended: bool,
    pub show_help: bool,
    pub frame: u32,
    pub think: u32,
    pub from_lesson: bool,
}

impl Game {
    pub fn new(seed: u64, lesson: bool) -> Self {
        let mut g = Self {
            phase: Phase::Workshop,
            crew: vec![Build::new(Body::Orc), Build::new(Body::Kobold)],
            shelf: vec![Part::Club, Part::Sling, Part::Leather, Part::SecondHeart, Part::Boots],
            fuses: FUSES,
            contest: 0,
            rival: Rival { name: "", text: "", crew: Vec::new() },
            battle: None,
            speed: 1,
            held: None,
            press_at: (-1, -1),
            mouse: (-1, -1),
            hover_item: None,
            lesson,
            lesson_step: 0,
            note: None,
            tips: Vec::new(),
            message: None,
            rng: Rng::new(seed),
            art: art::Art::new(),
            pals: (0..5).map(art::palette).collect(),
            stats: Stats::default(),
            ended: false,
            show_help: false,
            frame: 0,
            think: 0,
            from_lesson: false,
        };
        if lesson {
            let mut k = Build::new(Body::Kobold);
            k.parts[1] = Some(Part::Leather);
            g.rival = Rival { name: "THE PRACTICE DUMMIES", text: "TWO KOBOLDS, BARELY GEARED.", crew: vec![Build::new(Body::Kobold), k] };
        } else {
            g.rival = g.make_rival();
            g.tip("start", "GEAR UP YOUR CREW AGAINST THE RIVALS YOU SEE BELOW, THEN PRESS FIGHT. POINT AT ANYTHING FOR DETAILS.");
        }
        g
    }

    pub fn district(&self) -> usize {
        district_of(self.contest)
    }

    pub fn track(&self) -> Track {
        if self.lesson {
            return Track::LateShuttle;
        }
        [Track::MapleStreet, Track::InventorMuseum, Track::TransformerJunction, Track::Starbridge, Track::Railway][self.district()]
    }

    fn tip(&mut self, id: &'static str, text: &str) {
        if !self.tips.contains(&id) {
            self.tips.push(id);
            self.message = Some(text.to_string());
        }
    }

    fn fit_random(&mut self, b: &mut Build, pool: &[Part], count: usize) {
        let mut ps: Vec<Part> = pool.to_vec();
        for _ in 0..3 {
            self.rng.shuffle(&mut ps);
            for &p in &ps {
                if b.fitted().len() >= count {
                    return;
                }
                if let Some(i) = (0..b.parts.len()).find(|&i| b.parts[i].is_none() && b.refusal(i, p).is_none()) {
                    b.parts[i] = Some(p);
                }
            }
        }
    }

    pub fn make_rival(&mut self) -> Rival {
        let c = self.contest.min(CONTESTS - 1);
        let size = [2, 2, 3, 3, 3, 4, 4, 4][c];
        let gear = [1, 2, 2, 2, 3, 3, 3, 4][c];
        let t = self.rng.below(THEMES.len() as u32) as usize;
        let theme = &THEMES[t];
        let mut crew = Vec::new();
        for k in 0..size {
            // The final crew mixes two themes.
            let th = if c == CONTESTS - 1 && k % 2 == 1 { &THEMES[(t + 2) % THEMES.len()] } else { theme };
            let body = *self.rng.pick(th.bodies);
            let mut b = Build::new(body);
            let mut pool = th.parts.to_vec();
            if c >= 3 {
                pool.push(*self.rng.pick(&ALL_PARTS));
            }
            self.fit_random(&mut b, &pool, gear);
            crew.push(b);
        }
        if c == CONTESTS - 1 {
            return Rival { name: "THE RAILWAY BARONS", text: "THE BEST CREW ON THE LINE: A BIT OF EVERYTHING.", crew };
        }
        Rival { name: theme.name, text: theme.text, crew }
    }

    pub fn part_at(&self, f: From) -> Option<Part> {
        match f {
            From::Shelf(k) => self.shelf.get(k).copied(),
            From::Slot(m, s) => self.crew.get(m).and_then(|b| b.parts.get(s).copied().flatten()),
        }
    }

    /// The slot or shelf row under a point.
    pub fn spot_at(&self, x: i32, y: i32) -> Option<From> {
        for m in 0..self.crew.len() {
            for s in 0..self.crew[m].parts.len() {
                if slot_rect(m, s).contains(x, y) {
                    return Some(From::Slot(m, s));
                }
            }
        }
        if SHELF.contains(x, y) {
            let k = ((y - SHELF.y - 13).max(0) / 11) as usize;
            return Some(From::Shelf(k));
        }
        None
    }

    fn take(&mut self, f: From) -> Option<Part> {
        match f {
            From::Shelf(k) if k < self.shelf.len() => Some(self.shelf.remove(k)),
            From::Slot(m, s) => self.crew.get_mut(m).and_then(|b| b.parts.get_mut(s)).and_then(|p| p.take()),
            _ => None,
        }
    }

    /// Drops the held part on `to`. Returns true if it moved.
    fn drop_on(&mut self, to: From, out: &mut Out) -> bool {
        let Some((part, from)) = self.held else { return false };
        match to {
            From::Shelf(_) => {
                if let From::Slot(..) = from {
                    self.take(from);
                    self.shelf.push(part);
                    self.shelf.sort();
                    out.sfx(Sfx::Lift);
                    out.log(format!("workshop: {} back to the shelf", part.info().name.to_lowercase()));
                }
                self.held = None;
                true
            }
            From::Slot(m, s) => {
                if from == to {
                    return false;
                }
                // Check the fit as if the part had already left where it was.
                let mut probe = self.crew[m].clone();
                if let From::Slot(fm, fs) = from {
                    if fm == m {
                        probe.parts[fs] = None;
                    }
                }
                if let Some(why) = probe.refusal(s, part) {
                    self.message = Some(why);
                    out.sfx(Sfx::Deny);
                    self.held = None;
                    return false;
                }
                self.take(from);
                if let Some(old) = self.crew[m].parts[s].replace(part) {
                    self.shelf.push(old);
                    self.shelf.sort();
                }
                self.held = None;
                self.message = None;
                out.sfx(Sfx::Place);
                out.log(format!("workshop: {} on the {} ({})", part.info().name.to_lowercase(), self.crew[m].body.info().name.to_lowercase(), m + 1));
                if self.lesson {
                    if self.lesson_step == 0 && self.crew[0].fitted().contains(&Part::Club) {
                        self.lesson_step = 1;
                        out.sfx(Sfx::Lesson);
                    } else if self.lesson_step == 1 && self.crew.get(1).is_some_and(|k| !k.fitted().is_empty()) {
                        self.lesson_step = 2;
                        out.sfx(Sfx::Lesson);
                    }
                }
                if part_is_mind(part) {
                    self.tip("mind", "A MIND PART CHANGES WHAT A MONSTER GOES FOR. WITHOUT ONE IT FIGHTS THE NEAREST ENEMY, THEN THE CONDUIT.");
                }
                true
            }
        }
    }

    fn start_fight(&mut self, out: &mut Out) {
        if self.lesson && self.lesson_step < 2 {
            out.sfx(Sfx::Deny);
            return;
        }
        let describe = |crew: &[Build]| -> String {
            crew.iter()
                .map(|b| {
                    let parts: Vec<String> = b.fitted().iter().map(|p| p.info().name.to_lowercase()).collect();
                    format!("{}[{}]", b.body.info().name.to_lowercase(), parts.join("+"))
                })
                .collect::<Vec<_>>()
                .join(" ")
        };
        out.log(format!(
            "contest {} vs {} ({:.0}s in the workshop): ours {} | theirs {} | shelf [{}]",
            self.contest + 1,
            self.rival.name.to_lowercase(),
            self.think as f32 / 15.0,
            describe(&self.crew),
            describe(&self.rival.crew),
            self.shelf.iter().map(|p| p.info().name.to_lowercase()).collect::<Vec<_>>().join(", ")
        ));
        self.battle = Some(Battle::new(&self.crew, &self.rival.crew));
        self.phase = Phase::Battle;
        self.held = None;
        self.message = None;
        out.sfx(Sfx::BattleStart);
    }

    fn battle_tick(&mut self, out: &mut Out) {
        if self.note.is_some() {
            return;
        }
        let Some(b) = self.battle.as_mut() else { return };
        let n = match self.speed {
            1 => 1,
            2 => 2,
            _ => 4,
        };
        for _ in 0..n {
            b.step();
            let t = b.tick;
            let mut hit = false;
            let mut conduit = false;
            let mut died = false;
            for (ft, fx) in &b.fx {
                if *ft != t {
                    continue;
                }
                match fx {
                    Fx::Hit { .. } => hit = true,
                    Fx::ConduitHit { .. } => conduit = true,
                    Fx::Died { .. } => died = true,
                    _ => {}
                }
            }
            if died {
                out.sfx(Sfx::Hurt);
            } else if conduit {
                out.sfx(Sfx::Ward);
            } else if hit {
                out.sfx(Sfx::Strike);
            }
            if self.lesson {
                if hit && !self.tips.contains(&"first_hit") {
                    self.tips.push("first_hit");
                    self.note = Some(
                        "A HIT DOES ITS DAMAGE MINUS THE MATCHING DEFENSE: MELEE AGAINST ARMOR, RANGED AGAINST DODGE, MAGIC AGAINST WARD. ALWAYS AT LEAST 1."
                            .into(),
                    );
                    return;
                }
                if conduit && !self.tips.contains(&"first_conduit") {
                    self.tips.push("first_conduit");
                    self.note = Some("BREAK THE RIVAL CONDUIT TO WIN, AND KEEP YOURS STANDING. EACH HAS 30 HP.".into());
                    return;
                }
            }
            if b.over.is_some() {
                self.end_battle(out);
                return;
            }
        }
    }

    fn end_battle(&mut self, out: &mut Out) {
        let Some(b) = &self.battle else { return };
        let outcome = b.over.unwrap_or(Outcome::Draw);
        let alive = |s: Side| b.units.iter().filter(|u| u.alive && u.side == s).count();
        let dealt: Vec<String> = b.units.iter().filter(|u| u.side == Side::Blue).map(|u| format!("{} {}", u.build.body.info().name.to_lowercase(), u.dealt)).collect();
        out.log(format!(
            "contest {} {:?} after {} ticks: conduits ours {} theirs {}, standing ours {} theirs {}, damage dealt [{}]",
            self.contest + 1,
            outcome,
            b.tick,
            b.conduit_hp[0].max(0),
            b.conduit_hp[1].max(0),
            alive(Side::Blue),
            alive(Side::Red),
            dealt.join(", ")
        ));
        match outcome {
            Outcome::Won => {
                self.stats.won += 1;
                out.sfx(Sfx::BattleWin);
            }
            Outcome::Lost => {
                self.stats.lost += 1;
                out.sfx(Sfx::BattleLose);
            }
            Outcome::Draw => {
                self.stats.drawn += 1;
                out.sfx(Sfx::BattleLose);
            }
        }
        self.phase = Phase::After { outcome, t: 0 };
    }

    fn after_done(&mut self, out: &mut Out) {
        let Phase::After { outcome, .. } = self.phase else { return };
        self.battle = None;
        if self.lesson {
            self.lesson = false;
            out.log("lesson contest done");
            self.contest = 0;
            self.stats = Stats::default();
            self.from_lesson = true;
            self.offer_rewards(out);
            self.tip("reward", "AFTER EACH WIN, KEEP ONE THING: A PART, A CRATE OF PARTS, OR A NEW MONSTER.");
            return;
        }
        match outcome {
            Outcome::Won => {
                if self.contest + 1 >= CONTESTS {
                    out.log("WON: the conduit league is ours");
                    out.sfx(Sfx::Win);
                    self.phase = Phase::Over { won: true };
                    out.music = Some(Track::Railway);
                    return;
                }
                self.offer_rewards(out);
            }
            Outcome::Lost | Outcome::Draw => {
                if outcome == Outcome::Lost {
                    self.fuses -= 1;
                    self.tip("lost", "A LOST CONTEST BLOWS ONE OF YOUR 3 FUSES. YOUR CREW AND PARTS ARE KEPT.");
                } else {
                    self.tip("draw", "A DRAW: TIME RAN OUT WITH THE CONDUITS EVEN. NOTHING GAINED, NOTHING LOST.");
                }
                if self.fuses <= 0 {
                    out.log("OUT OF FUSES");
                    self.phase = Phase::Over { won: false };
                    return;
                }
                if self.contest + 1 >= CONTESTS {
                    // The final has to be won: same rivals, try again.
                    self.phase = Phase::Workshop;
                    self.think = 0;
                } else {
                    self.next_contest(out);
                }
            }
        }
    }

    fn offer_rewards(&mut self, out: &mut Out) {
        let mut offers = Vec::new();
        let mut pool: Vec<Part> = ALL_PARTS.to_vec();
        self.rng.shuffle(&mut pool);
        offers.push(Reward::Part(pool[0]));
        offers.push(Reward::Part(pool[1]));
        if self.crew.len() < MAX_CREW && self.rng.below(100) < 60 {
            offers.push(Reward::Monster(*self.rng.pick(&ALL_BODIES)));
        } else {
            offers.push(Reward::Crate(vec![pool[2], pool[3]]));
        }
        out.log(format!("reward offers: {}", offers.iter().map(reward_name).collect::<Vec<_>>().join(", ")));
        self.phase = Phase::Reward { offers };
        self.hover_item = None;
    }

    fn take_reward(&mut self, i: usize, out: &mut Out) {
        let Phase::Reward { offers } = self.phase.clone() else { return };
        let Some(r) = offers.get(i) else { return };
        match r {
            Reward::Part(p) => self.shelf.push(*p),
            Reward::Crate(ps) => self.shelf.extend(ps.iter().copied()),
            Reward::Monster(b) => self.crew.push(Build::new(*b)),
        }
        self.shelf.sort();
        out.log(format!("reward taken: {}", reward_name(r)));
        out.sfx(Sfx::Buy);
        self.next_contest(out);
    }

    fn next_contest(&mut self, out: &mut Out) {
        if self.from_lesson {
            // From the lesson straight into contest 1.
            self.from_lesson = false;
            self.tip("start", "GEAR UP YOUR CREW AGAINST THE RIVALS YOU SEE BELOW, THEN PRESS FIGHT. POINT AT ANYTHING FOR DETAILS.");
        } else {
            self.contest += 1;
        }
        self.rival = self.make_rival();
        self.phase = Phase::Workshop;
        self.think = 0;
        out.music = Some(self.track());
        if self.contest >= CONTESTS - 1 {
            self.message = Some("THE FINAL. WIN THIS ONE TO TAKE THE LEAGUE.".into());
        }
    }
}

pub fn reward_name(r: &Reward) -> String {
    match r {
        Reward::Part(p) => p.info().name.to_lowercase(),
        Reward::Monster(b) => format!("monster {}", b.info().name.to_lowercase()),
        Reward::Crate(ps) => format!("crate [{}]", ps.iter().map(|p| p.info().name.to_lowercase()).collect::<Vec<_>>().join(", ")),
    }
}

impl Run for Game {
    fn event(&mut self, ev: Ev, out: &mut Out) {
        if let Ev::Move(x, y) | Ev::Press { x, y, .. } | Ev::Release { x, y, .. } = ev {
            self.mouse = (x, y);
        }
        if self.show_help {
            if matches!(ev, Ev::Key(_) | Ev::Press { .. }) {
                self.show_help = false;
                out.sfx(Sfx::Click);
            }
            return;
        }
        if ev == Ev::Key(Key::Help) {
            self.show_help = true;
            out.sfx(Sfx::Click);
            return;
        }
        match self.phase.clone() {
            Phase::Workshop => match ev {
                Ev::Press { x, y, right } => {
                    if FIGHT_BTN.contains(x, y) && !right {
                        self.start_fight(out);
                        return;
                    }
                    let spot = self.spot_at(x, y);
                    if right {
                        // Right-click a fitted part: back to the shelf.
                        if let Some(f @ From::Slot(..)) = spot {
                            if let Some(p) = self.part_at(f) {
                                self.held = Some((p, f));
                                self.drop_on(From::Shelf(0), out);
                            }
                        }
                        return;
                    }
                    match (self.held, spot) {
                        (Some(_), Some(to)) => {
                            self.drop_on(to, out);
                        }
                        (None, Some(f)) => {
                            if let Some(p) = self.part_at(f) {
                                self.held = Some((p, f));
                                self.press_at = (x, y);
                                out.sfx(Sfx::Lift);
                            }
                        }
                        (Some(_), None) => self.held = None,
                        _ => {}
                    }
                }
                Ev::Release { x, y, right: false } => {
                    if let Some((_, from)) = self.held {
                        let moved = (x - self.press_at.0).abs() + (y - self.press_at.1).abs() > 3;
                        match self.spot_at(x, y) {
                            Some(to) if to != from && moved => {
                                self.drop_on(to, out);
                            }
                            Some(_) => {} // a click: keep holding until the next click
                            None if moved => self.held = None,
                            None => {}
                        }
                    }
                }
                Ev::Key(Key::Enter) => self.start_fight(out),
                _ => {}
            },
            Phase::Battle => match ev {
                Ev::Press { x, y, right: false } => {
                    if self.note.is_some() {
                        self.note = None;
                        out.sfx(Sfx::Click);
                    } else if SPEED_BTN.contains(x, y) {
                        self.speed = match self.speed {
                            1 => 2,
                            2 => 4,
                            _ => 1,
                        };
                        out.sfx(Sfx::Click);
                    } else if SKIP_BTN.contains(x, y) && !self.lesson {
                        out.sfx(Sfx::Click);
                        if let Some(b) = self.battle.as_mut() {
                            b.run();
                        }
                        out.log("contest skipped to the end");
                        self.end_battle(out);
                    }
                }
                Ev::Key(Key::Enter) | Ev::Key(Key::Space) if self.note.is_some() => {
                    self.note = None;
                    out.sfx(Sfx::Click);
                }
                Ev::Key(Key::Fast) => {
                    self.speed = match self.speed {
                        1 => 2,
                        2 => 4,
                        _ => 1,
                    };
                    out.sfx(Sfx::Click);
                }
                _ => {}
            },
            Phase::After { t, .. } => {
                if t > 8 && matches!(ev, Ev::Press { right: false, .. } | Ev::Key(Key::Enter) | Ev::Key(Key::Space)) {
                    out.sfx(Sfx::Click);
                    self.after_done(out);
                }
            }
            Phase::Reward { offers } => match ev {
                Ev::Move(x, y) => self.hover_item = (0..offers.len()).find(|&i| reward_rect(i).contains(x, y)),
                Ev::Press { x, y, right: false } => {
                    if let Some(i) = (0..offers.len()).find(|&i| reward_rect(i).contains(x, y)) {
                        self.take_reward(i, out);
                    }
                }
                _ => {}
            },
            Phase::Over { .. } => {
                if matches!(ev, Ev::Press { right: false, .. } | Ev::Key(Key::Enter)) {
                    out.sfx(Sfx::Click);
                    self.ended = true;
                }
            }
        }
    }

    fn tick(&mut self, out: &mut Out) {
        self.frame += 1;
        if self.frame == 1 {
            out.music = Some(self.track());
        }
        match &mut self.phase {
            Phase::Workshop => self.think += 1,
            Phase::After { t, .. } => *t += 1,
            Phase::Battle => self.battle_tick(out),
            _ => {}
        }
    }

    fn draw(&self, fb: &mut Framebuffer) {
        render::draw(self, fb);
    }

    fn palette(&self) -> &Pal {
        &self.pals[if self.lesson { 0 } else { self.district() }]
    }

    fn ended(&self) -> bool {
        self.ended
    }

    fn summary(&self) -> Summary {
        let headline = match self.phase {
            Phase::Over { won: true } => "THE CONDUIT LEAGUE IS YOURS.".to_string(),
            _ if self.lesson => "STOPPED DURING THE PRACTICE CONTEST.".to_string(),
            Phase::Over { won: false } => format!("OUT OF FUSES AT {}, CONTEST {} OF {CONTESTS}.", DISTRICTS[self.district()], self.contest + 1),
            _ => format!("STOPPED AT {}, CONTEST {} OF {CONTESTS}.", DISTRICTS[self.district()], self.contest + 1),
        };
        let crew: Vec<String> = self
            .crew
            .iter()
            .map(|b| format!("{} ({})", b.body.info().name, b.fitted().iter().map(|p| p.info().name).collect::<Vec<_>>().join(", ")))
            .collect();
        let s = &self.stats;
        let detail = format!("WON {}, LOST {}, DREW {}. CREW: {}.", s.won, s.lost, s.drawn, crew.join("; "));
        let won = matches!(self.phase, Phase::Over { won: true });
        Summary { headline, detail, best_key: self.contest as u32 + 1 + if won { 100 } else { 0 } }
    }

    fn takes_esc(&self) -> bool {
        self.show_help
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Greedy fitter: tries every shelf part in every slot, attacks first.
    fn gear_up(g: &mut Game, out: &mut Out) {
        for _ in 0..3 {
            for m in 0..g.crew.len() {
                for s in 0..g.crew[m].parts.len() {
                    if g.crew[m].parts[s].is_some() {
                        continue;
                    }
                    let mut order: Vec<usize> = (0..g.shelf.len()).collect();
                    order.sort_by_key(|&k| if g.shelf[k].attack().is_some() { 0 } else { 1 });
                    for k in order {
                        if k < g.shelf.len() && g.crew[m].refusal(s, g.shelf[k]).is_none() {
                            g.held = Some((g.shelf[k], From::Shelf(k)));
                            g.drop_on(From::Slot(m, s), out);
                            break;
                        }
                    }
                }
            }
        }
    }

    fn drive(g: &mut Game, out: &mut Out) {
        for _ in 0..20_000 {
            if g.ended {
                return;
            }
            match g.phase.clone() {
                Phase::Workshop => {
                    gear_up(g, out);
                    if g.lesson {
                        g.lesson_step = 2;
                    }
                    g.start_fight(out);
                }
                Phase::Battle => {
                    g.note = None;
                    g.speed = 4;
                    g.tick(out);
                }
                Phase::After { .. } => g.after_done(out),
                Phase::Reward { .. } => g.take_reward(2, out),
                Phase::Over { .. } => g.ended = true,
            }
        }
    }

    #[test]
    fn lesson_then_whole_runs_reach_an_end() {
        let mut results = Vec::new();
        for seed in 0..16 {
            let mut g = Game::new(seed, seed == 0);
            let mut out = Out::default();
            drive(&mut g, &mut out);
            assert!(g.ended, "seed {seed} stuck in {:?}", g.phase);
            results.push((g.contest + 1, g.stats.won, g.stats.lost, matches!(g.phase, Phase::Over { won: true })));
        }
        println!("conduit bot results (contest reached, won, lost, victory): {results:?}");
    }

    #[test]
    fn dragging_respects_capacity_and_returns_swapped_parts() {
        let mut g = Game::new(1, false);
        let mut out = Out::default();
        let club = g.shelf.iter().position(|p| *p == Part::Club).unwrap();
        g.held = Some((Part::Club, From::Shelf(club)));
        assert!(g.drop_on(From::Slot(0, 0), &mut out));
        assert_eq!(g.crew[0].parts[0], Some(Part::Club));
        // The kobold's held slot takes the sling, not plate armor in the wrong slot.
        let sling = g.shelf.iter().position(|p| *p == Part::Sling).unwrap();
        g.held = Some((Part::Sling, From::Shelf(sling)));
        assert!(!g.drop_on(From::Slot(1, 1), &mut out));
        assert!(g.message.as_ref().unwrap().contains("HELD SLOT"));
        // Moving the club from the orc to the kobold.
        g.held = Some((Part::Club, From::Slot(0, 0)));
        assert!(g.drop_on(From::Slot(1, 0), &mut out));
        assert_eq!(g.crew[0].parts[0], None);
        assert_eq!(g.crew[1].parts[0], Some(Part::Club));
    }
}
