//! TURNCOAT: place your band, then watch. Whoever runs out of resolve joins the side that won
//! them over.

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
use std::collections::VecDeque;

pub const VERSION: u32 = 1;
pub const LESSON_VERSION: u32 = 1;
pub static FORM_QUESTION: Question = Question { id: "watching", text: "WATCHING THE BATTLES WAS...", low: "DULL", high: "GRIPPING" };

pub const TILE: i32 = 28;
pub const FX: i32 = 8;
pub const FY: i32 = 20;
pub const BATTLES: usize = 13;
pub const BENCH_Y: i32 = 204;
pub const START_BTN: Rect = Rect::new(274, 200, 118, 30);
pub const SPEED_BTN: Rect = Rect::new(274, 200, 56, 30);
pub const SKIP_BTN: Rect = Rect::new(336, 200, 56, 30);

#[derive(Clone, Debug, PartialEq)]
pub struct Member {
    pub uid: u32,
    pub kind: Kind,
    pub career: i32,
    pub veteran: bool,
}

#[derive(Clone, Debug)]
pub struct Encounter {
    pub rivals: Vec<(Kind, Pos, bool)>,
    pub leader: bool,
}

#[derive(Clone, Debug, PartialEq)]
pub enum Phase {
    Route { options: Vec<usize> },
    Deploy,
    Battle,
    After { outcome: Outcome, t: u32 },
    Keep { cands: Vec<(Member, bool)>, keep: Vec<bool> },
    Idea { offers: Vec<Idea> },
    Over { won: bool },
}

pub struct Drag {
    pub member: usize,
    pub from: Option<Pos>,
}

/// What the screen shows for each person while acts play out (it trails the battle state).
#[derive(Clone, Debug)]
pub struct Shown {
    pub side: Side,
    pub resolve: i32,
    pub pos: Pos,
}

pub struct Playing {
    pub act: Act,
    pub t: u32,
    pub len: u32,
}

#[derive(Default, Clone)]
pub struct Stats {
    pub won: u32,
    pub lost: u32,
    pub drawn: u32,
    pub turned_to_us: u32,
    pub turned_from_us: u32,
    pub battle_frames: u32,
}

pub struct Game {
    pub phase: Phase,
    pub band: Vec<Member>,
    pub ideas: Vec<Idea>,
    pub lives: i32,
    pub index: usize,
    pub leaders_beaten: i32,
    pub encounters: Vec<Encounter>,
    pub encounter: Option<Encounter>,
    pub placed: Vec<(usize, Pos)>,
    pub battle: Option<Battle>,
    pub shown: Vec<Shown>,
    pub queue: VecDeque<Act>,
    pub playing: Option<Playing>,
    pub speed: u32,
    pub drag: Option<Drag>,
    pub mouse: (i32, i32),
    pub hover_route: Option<usize>,
    pub hover_item: Option<usize>,
    pub lesson: bool,
    pub lesson_step: usize,
    pub note: Option<String>,
    pub tips: Vec<&'static str>,
    pub message: Option<String>,
    pub rng: Rng,
    pub next_uid: u32,
    pub art: art::Art,
    pub pals: Vec<Pal>,
    pub stats: Stats,
    pub ended: bool,
    pub show_help: bool,
    pub frame: u32,
    pub think: u32,
    pub particles: Vec<(i32, i32, i32, i32, i32, u8)>,
}

pub const LESSON_STEPS: [&str; 3] = [
    "THESE ARE YOUR PEOPLE, IN BLUE, ON THE BENCH BELOW. DRAG ONE ONTO A LIT SQUARE OF THE FIELD.",
    "POINT AT ANYONE, YOURS OR THEIRS: THE MARKED SQUARES ARE WHERE THEIR VOICE REACHES. THE BAR UNDER THEM IS THEIR RESOLVE.",
    "PLACE THE REST WHERE YOU LIKE (DRAG THEM AGAIN TO MOVE THEM), THEN PRESS START.",
];

impl Game {
    pub fn new(seed: u64, lesson: bool) -> Self {
        let mut g = Self {
            phase: Phase::Deploy,
            band: Vec::new(),
            ideas: Vec::new(),
            lives: 3,
            index: 0,
            leaders_beaten: 0,
            encounters: Vec::new(),
            encounter: None,
            placed: Vec::new(),
            battle: None,
            shown: Vec::new(),
            queue: VecDeque::new(),
            playing: None,
            speed: 1,
            drag: None,
            mouse: (-1, -1),
            hover_route: None,
            hover_item: None,
            lesson,
            lesson_step: 0,
            note: None,
            tips: Vec::new(),
            message: None,
            rng: Rng::new(seed),
            next_uid: 1,
            art: art::Art::new(),
            pals: (0..5).map(art::palette).collect(),
            stats: Stats::default(),
            ended: false,
            show_help: false,
            frame: 0,
            think: 0,
            particles: Vec::new(),
        };
        for k in [Kind::Gatherer, Kind::Gatherer, Kind::Hunter, Kind::Elder] {
            let uid = g.uid();
            g.band.push(Member { uid, kind: k, career: 0, veteran: false });
        }
        if lesson {
            g.encounter = Some(Encounter {
                rivals: vec![(Kind::Gatherer, (2, 2), false), (Kind::Gatherer, (4, 2), false), (Kind::Hunter, (3, 1), false)],
                leader: false,
            });
            g.phase = Phase::Deploy;
        } else {
            g.open_route(&mut Out::default());
        }
        g
    }

    fn uid(&mut self) -> u32 {
        self.next_uid += 1;
        self.next_uid
    }

    pub fn age(&self) -> usize {
        if self.lesson {
            0
        } else {
            (self.index / 3).min(4)
        }
    }

    pub fn track(&self) -> Track {
        match self.phase {
            Phase::Over { won: true } => Track::Posthistory,
            _ => [Track::Prehistory, Track::IronAge, Track::GoldenAge, Track::SpaceAge, Track::AMyth][self.age()],
        }
    }

    pub fn deploy_limit(&self) -> usize {
        if self.lesson {
            return 4;
        }
        (4 + self.leaders_beaten + if self.ideas.contains(&Idea::Banners) { 1 } else { 0 }).min(8) as usize
    }

    pub fn band_limit(&self) -> usize {
        7 + if self.ideas.contains(&Idea::BigTent) { 2 } else { 0 }
    }

    pub fn is_leader_battle(&self) -> bool {
        !self.lesson && (self.index % 3 == 2 || self.index >= 12)
    }

    fn tip(&mut self, id: &'static str, text: &str) {
        if !self.tips.contains(&id) {
            self.tips.push(id);
            self.message = Some(text.to_string());
        }
    }

    fn gen_encounter(&mut self, heavy: bool) -> Encounter {
        let age = self.age();
        let slot = self.index % 3;
        let leader = self.is_leader_battle();
        let base = [3, 4, 5, 6, 6][age] + if slot == 1 { 1 } else { 0 };
        let count = if leader { base } else { base + heavy as usize };
        let pool = age_pool(age);
        let total: u32 = pool.iter().map(|p| p.1).sum();
        let mut kinds: Vec<(Kind, bool)> = Vec::new();
        if leader {
            kinds.push((LEADERS[age], false));
        }
        while kinds.len() < count {
            let mut r = self.rng.below(total);
            let mut pick = pool[0].0;
            for &(k, w) in &pool {
                if r < w {
                    pick = k;
                    break;
                }
                r -= w;
            }
            let vet = (heavy && kinds.iter().all(|k| !k.1)) || (age >= 2 && self.rng.below(100) < (age as u32 * 12));
            kinds.push((pick, vet));
        }
        // Short voices in front, long voices behind, the leader in the middle.
        let row_of = |k: Kind| match k.info().reach {
            _ if k.info().leader => 1,
            Reach::Radius(_) | Reach::Column | Reach::Ahead(_) => 0,
            Reach::Lines(_, n) if n > 2 => 0,
            Reach::Lines(..) => 1,
            _ => 2,
        };
        let mut rivals: Vec<(Kind, Pos, bool)> = Vec::new();
        for (k, vet) in kinds {
            let mut row = row_of(k);
            for _ in 0..30 {
                let x = self.rng.range(0, W - 1);
                if rivals.iter().all(|r| r.1 != (x, row)) {
                    rivals.push((k, (x, row), vet));
                    break;
                }
                row = (row + 1) % 3;
            }
        }
        Encounter { rivals, leader }
    }

    fn open_route(&mut self, out: &mut Out) {
        self.encounters.clear();
        if self.is_leader_battle() {
            let e = self.gen_encounter(false);
            self.encounters.push(e);
        } else {
            let a = self.gen_encounter(false);
            let b = self.gen_encounter(true);
            self.encounters.push(a);
            self.encounters.push(b);
        }
        self.phase = Phase::Route { options: (0..self.encounters.len()).collect() };
        self.hover_route = None;
        out.music = Some(self.track());
        if self.index == 0 {
            self.tip("route", "CHOOSE WHO TO MEET NEXT. ANYONE YOU WIN OVER CAN JOIN YOUR BAND.");
        }
        if self.is_leader_battle() && self.index < 12 {
            self.tip("leader", "THE AGE'S LEADER. WIN, AND YOU GAIN AN IDEA AND ONE MORE PLACE ON THE FIELD.");
        }
        if self.index == 12 {
            self.message = Some("A MYTH. THE LAST CONTEST.".into());
        }
    }

    fn choose_route(&mut self, i: usize, out: &mut Out) {
        let Some(e) = self.encounters.get(i).cloned() else { return };
        out.sfx(Sfx::Click);
        let names: Vec<String> = e.rivals.iter().map(|r| format!("{}{}", r.0.info().name.to_lowercase(), if r.2 { "*" } else { "" })).collect();
        out.log(format!(
            "battle {} ({}): chose option {} of {} [{}]",
            self.index + 1,
            AGES[self.age()].to_lowercase(),
            i + 1,
            self.encounters.len(),
            names.join(", ")
        ));
        self.encounter = Some(e);
        self.placed.retain(|(m, _)| *m < self.band.len());
        self.phase = Phase::Deploy;
        self.message = None;
        self.think = 0;
    }

    pub fn field_square(&self, x: i32, y: i32) -> Option<Pos> {
        let (gx, gy) = ((x - FX).div_euclid(TILE), (y - FY).div_euclid(TILE));
        (x >= FX && y >= FY && inb((gx, gy))).then_some((gx, gy))
    }

    pub fn bench_rect(i: usize) -> Rect {
        Rect::new(8 + i as i32 * 29, BENCH_Y, 28, 28)
    }

    pub fn bench(&self) -> Vec<usize> {
        (0..self.band.len()).filter(|m| !self.placed.iter().any(|(p, _)| p == m)).collect()
    }

    pub fn can_stand(&self, member: usize, p: Pos) -> bool {
        let rival = self.encounter.as_ref().is_some_and(|e| e.rivals.iter().any(|r| r.1 == p));
        let ours = self.placed.iter().any(|(m, q)| *q == p && *m != member);
        if rival || ours {
            return false;
        }
        p.1 >= 3 || self.band[member].kind.has(Trait::DropIn)
    }

    fn place(&mut self, member: usize, p: Pos, out: &mut Out) {
        if !self.can_stand(member, p) {
            out.sfx(Sfx::Deny);
            return;
        }
        let already = self.placed.iter().any(|(m, _)| *m == member);
        if !already && self.placed.len() >= self.deploy_limit() {
            out.sfx(Sfx::Deny);
            self.message = Some(format!("ONLY {} OF YOUR PEOPLE FIT ON THE FIELD THIS TIME.", self.deploy_limit()));
            return;
        }
        self.placed.retain(|(m, _)| *m != member);
        self.placed.push((member, p));
        out.sfx(Sfx::Place);
        if self.lesson && self.lesson_step == 0 {
            self.lesson_step = 1;
            out.sfx(Sfx::Lesson);
        }
    }

    /// The person under the pointer: on the field (yours or theirs) or on the bench.
    pub fn hovered(&self) -> Option<(Kind, Side, Option<Pos>, Option<usize>)> {
        let (x, y) = self.mouse;
        if let Some(p) = self.field_square(x, y) {
            if let (Phase::Battle | Phase::After { .. }, Some(b)) = (&self.phase, &self.battle) {
                if let Some(i) = self.shown.iter().position(|s| s.pos == p) {
                    return Some((b.people[i].kind, self.shown[i].side, Some(p), Some(i)));
                }
                return None;
            }
            if let Some((m, _)) = self.placed.iter().find(|(_, q)| *q == p) {
                return Some((self.band[*m].kind, Side::Blue, Some(p), None));
            }
            if let Some(r) = self.encounter.as_ref().and_then(|e| e.rivals.iter().find(|r| r.1 == p)) {
                return Some((r.0, Side::Red, Some(p), None));
            }
        }
        for (slot, &m) in self.bench().iter().enumerate() {
            if Self::bench_rect(slot).contains(x, y) {
                return Some((self.band[m].kind, Side::Blue, None, None));
            }
        }
        None
    }

    /// A battle as it would stand at the start, from the current placement.
    pub fn build_battle(&self) -> Battle {
        let mut people = Vec::new();
        for &(m, p) in &self.placed {
            let mb = &self.band[m];
            let max = max_resolve(mb.kind, mb.veteran, Side::Blue, &self.ideas);
            people.push(Person { uid: mb.uid, kind: mb.kind, side: Side::Blue, origin: Side::Blue, pos: p, resolve: max, max, veteran: mb.veteran, career: mb.career, turned_here: 0 });
        }
        if let Some(e) = &self.encounter {
            for (k, (kind, p, vet)) in e.rivals.iter().enumerate() {
                let max = max_resolve(*kind, *vet, Side::Red, &[]);
                people.push(Person {
                    uid: 10_000 + k as u32,
                    kind: *kind,
                    side: Side::Red,
                    origin: Side::Red,
                    pos: *p,
                    resolve: max,
                    max,
                    veteran: *vet,
                    career: if *vet { 3 } else { 0 },
                    turned_here: 0,
                });
            }
        }
        Battle::new(people, self.ideas.clone())
    }

    fn start_battle(&mut self, out: &mut Out) {
        if self.placed.is_empty() {
            out.sfx(Sfx::Deny);
            self.message = Some("DRAG AT LEAST ONE OF YOUR PEOPLE ONTO THE FIELD FIRST.".into());
            return;
        }
        let b = self.build_battle();
        self.shown = b.people.iter().map(|p| Shown { side: p.side, resolve: p.resolve, pos: p.pos }).collect();
        let ours: Vec<String> = self.placed.iter().map(|(m, p)| format!("{}@{}{}", self.band[*m].kind.info().name.to_lowercase(), (b'a' + p.0 as u8) as char, H - p.1)).collect();
        out.log(format!(
            "battle {} start ({:.0}s deploying): placed [{}], bench {}",
            self.index + 1,
            self.think as f32 / 15.0,
            ours.join(", "),
            self.band.len() - self.placed.len()
        ));
        self.battle = Some(b);
        self.queue.clear();
        self.playing = None;
        self.phase = Phase::Battle;
        self.message = None;
        if self.lesson {
            self.lesson_step = 3;
        }
        out.sfx(Sfx::BattleStart);
    }

    fn act_len(&self, a: &Act) -> u32 {
        let base = match a {
            Act::Round(_) => 1,
            Act::Speak { .. } => 5,
            Act::Turned { .. } => 6,
            Act::Held { .. } => 4,
            Act::Walk { .. } => 2,
            Act::Veteran { .. } => 5,
            Act::Mend { .. } => 2,
        };
        match self.speed {
            1 => base,
            2 => (base + 1) / 2,
            _ => 1,
        }
    }

    /// Applies an act's visible effect when it starts playing.
    fn show_act(&mut self, a: &Act, out: &mut Out) {
        let Some(b) = &self.battle else { return };
        match a {
            Act::Speak { actor, hits } => {
                out.sfx(Sfx::Persuade);
                let side = self.shown[*actor].side;
                for &(t, d) in hits {
                    if self.shown[t].side != side {
                        self.shown[t].resolve = (self.shown[t].resolve - d).max(0);
                    }
                }
                if self.lesson {
                    self.lesson_note("speak", "EACH TURN, A PERSON PERSUADES THE RIVAL IN REACH WITH THE LEAST RESOLVE LEFT, OR WALKS CLOSER. THE FASTEST (HIGHEST PACE) GO FIRST.");
                }
            }
            Act::Turned { who, to, .. } => {
                out.sfx(Sfx::Turn);
                self.shown[*who].side = *to;
                self.shown[*who].resolve = b.people[*who].resolve;
                let (x, y) = (FX + self.shown[*who].pos.0 * TILE + TILE / 2, FY + self.shown[*who].pos.1 * TILE + TILE / 2);
                for (dx, dy) in [(2, 0), (-2, 0), (0, 2), (0, -2), (1, 1), (-1, 1), (1, -1), (-1, -1)] {
                    self.particles.push((x, y, dx, dy, 6, art::GOLD_R + 2));
                }
                if *to == Side::Blue {
                    self.stats.turned_to_us += 1;
                } else {
                    self.stats.turned_from_us += 1;
                }
                if self.lesson {
                    self.lesson_note(
                        "turn",
                        "OUT OF RESOLVE: THEY JOIN THE SIDE THAT WON THEM OVER, WITH HALF THEIR RESOLVE BACK. WIN EVERYONE OVER TO WIN THE BATTLE.",
                    );
                }
            }
            Act::Held { who } => {
                out.sfx(Sfx::Block);
                self.shown[*who].resolve = 1;
            }
            Act::Walk { .. } => out.sfx(Sfx::Hop),
            Act::Veteran { who } => {
                out.sfx(Sfx::Lesson);
                self.shown[*who].resolve = b.people[*who].resolve;
                self.tip("veteran", "A VETERAN: SOMEONE WHO TURNS THREE PEOPLE SPEAKS 1 LOUDER AND HAS 2 MORE RESOLVE, FOR GOOD.");
            }
            Act::Mend { who } => self.shown[*who].resolve += 1,
            Act::Round(_) => {}
        }
    }

    fn finish_act(&mut self, a: &Act) {
        if let Act::Walk { who, to, .. } = a {
            self.shown[*who].pos = *to;
        }
    }

    fn lesson_note(&mut self, id: &'static str, text: &str) {
        if !self.tips.contains(&id) {
            self.tips.push(id);
            self.note = Some(text.to_string());
        }
    }

    fn sync_shown(&mut self) {
        if let Some(b) = &self.battle {
            self.shown = b.people.iter().map(|p| Shown { side: p.side, resolve: p.resolve, pos: p.pos }).collect();
        }
    }

    fn battle_tick(&mut self, out: &mut Out) {
        if self.note.is_some() {
            return;
        }
        self.stats.battle_frames += 1;
        // Several short acts can fit in one frame at high speed.
        let mut budget = if self.speed >= 4 { 3 } else { 1 };
        while budget > 0 {
            if let Some(p) = self.playing.as_mut() {
                p.t += 1;
                if p.t < p.len {
                    return;
                }
                let a = self.playing.take().unwrap().act;
                self.finish_act(&a);
                budget -= 1;
                if self.note.is_some() {
                    return;
                }
                continue;
            }
            if self.queue.is_empty() {
                let over = self.battle.as_ref().map_or(true, |b| b.over.is_some());
                if over {
                    self.sync_shown();
                    self.end_battle(out);
                    return;
                }
                let acts = self.battle.as_mut().unwrap().step();
                self.queue.extend(acts);
                if self.queue.is_empty() {
                    continue;
                }
            }
            let a = self.queue.pop_front().unwrap();
            let len = self.act_len(&a);
            self.show_act(&a, out);
            if self.queue.is_empty() && !matches!(a, Act::Walk { .. }) {
                // End of one person's action: make sure the screen matches the battle.
            }
            self.playing = Some(Playing { act: a, t: 0, len });
        }
    }

    fn skip_battle(&mut self, out: &mut Out) {
        if let Some(b) = self.battle.as_mut() {
            let acts = b.run();
            for a in acts {
                if let Act::Turned { to, .. } = a {
                    if to == Side::Blue {
                        self.stats.turned_to_us += 1;
                    } else {
                        self.stats.turned_from_us += 1;
                    }
                }
            }
        }
        self.queue.clear();
        self.playing = None;
        self.note = None;
        self.sync_shown();
        out.log("battle skipped to the end");
        self.end_battle(out);
    }

    fn end_battle(&mut self, out: &mut Out) {
        let Some(b) = &self.battle else { return };
        let outcome = b.over.unwrap_or(Outcome::Draw);
        let blue = b.people.iter().filter(|p| p.side == Side::Blue).count();
        out.log(format!(
            "battle {} {:?} in {} rounds: {} of {} on our side at the end",
            self.index + 1,
            outcome,
            b.round.min(ROUND_CAP),
            blue,
            b.people.len()
        ));
        match outcome {
            Outcome::Won => {
                out.sfx(Sfx::BattleWin);
                self.stats.won += 1;
            }
            Outcome::Lost => {
                out.sfx(Sfx::BattleLose);
                self.stats.lost += 1;
            }
            Outcome::Draw => {
                out.sfx(Sfx::BattleLose);
                self.stats.drawn += 1;
            }
        }
        self.phase = Phase::After { outcome, t: 0 };
    }

    fn after_done(&mut self, out: &mut Out) {
        let Phase::After { outcome, .. } = self.phase else { return };
        let b = self.battle.take().unwrap();
        let was_leader = self.is_leader_battle();
        if self.lesson {
            self.lesson = false;
            self.lesson_step = 0;
            // The lesson doesn't count as battle 1.
            self.encounter = None;
            out.log("lesson battle done");
            if outcome != Outcome::Won {
                // The lesson can't be lost for good: back to the start of the run proper.
                self.placed.clear();
                self.index = 0;
                self.open_route(out);
                return;
            }
        }
        match outcome {
            Outcome::Won => {
                // Carry careers back to the band, then gather everyone now on our side.
                let mut cands: Vec<(Member, bool)> = Vec::new();
                for p in b.people.iter().filter(|p| p.side == Side::Blue) {
                    let member = if p.origin == Side::Blue {
                        let mut m = self.band.iter().find(|m| m.uid == p.uid).unwrap().clone();
                        m.career = p.career;
                        m.veteran = p.veteran;
                        m
                    } else {
                        let uid = self.uid();
                        Member { uid, kind: p.kind, career: p.career, veteran: p.veteran }
                    };
                    cands.push((member, p.origin == Side::Red));
                }
                for (i, m) in self.band.iter().enumerate() {
                    if !self.placed.iter().any(|(pm, _)| *pm == i) {
                        cands.push((m.clone(), false));
                    }
                }
                let limit = self.band_limit();
                let mut order: Vec<usize> = (0..cands.len()).collect();
                order.sort_by_key(|&i| {
                    let (m, new) = &cands[i];
                    (*new, !m.kind.info().leader, !m.veteran, -(m.kind.info().resolve + m.kind.info().voice * 2))
                });
                let mut keep = vec![false; cands.len()];
                for &i in order.iter().take(limit) {
                    keep[i] = true;
                }
                if was_leader {
                    self.leaders_beaten += 1;
                }
                self.phase = Phase::Keep { cands, keep };
                self.hover_item = None;
                self.tip("keep", "EVERYONE ON YOUR SIDE MAY STAY, BUT YOUR BAND HOLDS ONLY SO MANY. CLICK PEOPLE TO CHOOSE WHO STAYS.");
            }
            Outcome::Lost | Outcome::Draw => {
                if outcome == Outcome::Lost {
                    self.lives -= 1;
                    self.tip("lost", "A LOST CONTEST COSTS ONE OF YOUR CHANCES. YOUR BAND IS AS IT WAS BEFORE.");
                } else {
                    self.tip("draw", "A DRAW: 30 ROUNDS PASSED WITH THE SIDES EVEN. NOBODY JOINS, NOTHING IS LOST.");
                }
                if self.lives <= 0 {
                    out.log("OUT OF CHANCES");
                    self.phase = Phase::Over { won: false };
                    out.music = Some(self.track());
                } else if self.index >= 12 {
                    // The myth stands; try again.
                    self.phase = Phase::Deploy;
                } else {
                    self.next_battle(false, out);
                }
            }
        }
    }

    fn finish_keep(&mut self, out: &mut Out) {
        let Phase::Keep { cands, keep } = self.phase.clone() else { return };
        if !keep.iter().any(|k| *k) {
            out.sfx(Sfx::Deny);
            self.message = Some("KEEP AT LEAST ONE PERSON.".into());
            return;
        }
        let old: Vec<u32> = self.band.iter().map(|m| m.uid).collect();
        self.band = cands.iter().zip(&keep).filter(|(_, k)| **k).map(|((m, _), _)| m.clone()).collect();
        let joined: Vec<String> = self.band.iter().filter(|m| !old.contains(&m.uid)).map(|m| m.kind.info().name.to_lowercase()).collect();
        let left: Vec<String> = cands.iter().zip(&keep).filter(|(_, k)| !**k).map(|((m, _), _)| m.kind.info().name.to_lowercase()).collect();
        out.log(format!("who stays: joined [{}], left [{}], band now {}", joined.join(", "), left.join(", "), self.band.len()));
        // Keep the placement for people who stayed (so the next battle starts from it).
        let uid_pos: Vec<(u32, Pos)> = self.placed.iter().filter_map(|(m, p)| old.get(*m).map(|u| (*u, *p))).collect();
        self.placed = uid_pos.iter().filter_map(|(u, p)| self.band.iter().position(|m| m.uid == *u).map(|i| (i, *p))).filter(|(_, p)| p.1 >= 3).collect();
        out.sfx(Sfx::Click);
        let was_leader = self.is_leader_battle();
        if self.index >= 12 {
            self.phase = Phase::Over { won: true };
            out.music = Some(Track::Posthistory);
            out.sfx(Sfx::Win);
            out.log("WON: the myth is ours");
            return;
        }
        if was_leader {
            let mut pool: Vec<Idea> = ALL_IDEAS.iter().copied().filter(|i| !self.ideas.contains(i)).collect();
            self.rng.shuffle(&mut pool);
            pool.truncate(3);
            self.phase = Phase::Idea { offers: pool };
            self.hover_item = None;
            return;
        }
        self.next_battle(true, out);
    }

    fn next_battle(&mut self, _won: bool, out: &mut Out) {
        if self.encounter.is_some() && !self.lesson {
            self.index += 1;
        }
        self.placed.retain(|(m, _)| *m < self.band.len());
        while self.placed.len() > self.deploy_limit() {
            self.placed.pop();
        }
        self.open_route(out);
    }

    fn pick_idea(&mut self, i: usize, out: &mut Out) {
        let Phase::Idea { offers } = &self.phase else { return };
        let Some(&idea) = offers.get(i) else { return };
        self.ideas.push(idea);
        out.log(format!("idea: {}", idea.name().to_lowercase()));
        out.sfx(Sfx::Buy);
        self.next_battle(true, out);
    }
}

pub fn route_rect(i: usize, n: usize) -> Rect {
    if n == 1 {
        Rect::new(100, 70, 200, 150)
    } else {
        Rect::new(14 + i as i32 * 192, 70, 180, 150)
    }
}

pub fn keep_rect(i: usize) -> Rect {
    let (col, row) = (i % 6, i / 6);
    Rect::new(14 + col as i32 * 63, 64 + row as i32 * 62, 58, 56)
}
pub const KEEP_DONE: Rect = Rect::new(270, 268, 120, 22);
pub fn idea_rect(i: usize) -> Rect {
    Rect::new(30, 70 + i as i32 * 52, 340, 46)
}
pub const OVER_GO: Rect = Rect::new(140, 262, 120, 22);

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
            Phase::Route { options } => match ev {
                Ev::Move(x, y) => self.hover_route = (0..options.len()).find(|&i| route_rect(i, options.len()).contains(x, y)),
                Ev::Press { x, y, right: false } => {
                    if let Some(i) = (0..options.len()).find(|&i| route_rect(i, options.len()).contains(x, y)) {
                        self.choose_route(i, out);
                    }
                }
                Ev::Key(Key::Left) | Ev::Key(Key::Right) | Ev::Key(Key::Tab) => {
                    self.hover_route = Some(self.hover_route.map_or(0, |h| (h + 1) % options.len()));
                }
                Ev::Key(Key::Enter) | Ev::Key(Key::Space) => {
                    let h = self.hover_route.unwrap_or(0);
                    self.choose_route(h, out);
                }
                _ => {}
            },
            Phase::Deploy => match ev {
                Ev::Move(..) => {
                    if self.lesson && self.lesson_step == 1 && self.hovered().is_some_and(|h| h.2.is_some()) {
                        self.lesson_step = 2;
                        out.sfx(Sfx::Lesson);
                    }
                }
                Ev::Press { x, y, right } => {
                    if START_BTN.contains(x, y) && !right {
                        if self.lesson && self.lesson_step < 2 {
                            out.sfx(Sfx::Deny);
                        } else {
                            self.start_battle(out);
                        }
                        return;
                    }
                    if let Some(p) = self.field_square(x, y) {
                        if let Some(&(m, _)) = self.placed.iter().find(|(_, q)| *q == p) {
                            if right {
                                self.placed.retain(|(mm, _)| *mm != m);
                                out.sfx(Sfx::Lift);
                            } else {
                                self.drag = Some(Drag { member: m, from: Some(p) });
                                out.sfx(Sfx::Lift);
                            }
                        } else if let Some(d) = self.drag.take() {
                            self.place(d.member, p, out);
                        }
                        return;
                    }
                    for (slot, m) in self.bench().into_iter().enumerate() {
                        if Self::bench_rect(slot).contains(x, y) && !right {
                            self.drag = Some(Drag { member: m, from: None });
                            out.sfx(Sfx::Lift);
                            return;
                        }
                    }
                }
                Ev::Release { x, y, right: false } => {
                    if let Some(d) = self.drag.take() {
                        match self.field_square(x, y) {
                            Some(p) if Some(p) != d.from => self.place(d.member, p, out),
                            Some(_) => self.drag = Some(d), // a click without moving: keep holding
                            None => {
                                if d.from.is_some() && y >= BENCH_Y - 6 {
                                    self.placed.retain(|(m, _)| *m != d.member);
                                    out.sfx(Sfx::Place);
                                } else if d.from.is_none() && y >= BENCH_Y - 6 {
                                    // Clicked a bench person: keep holding until a square is clicked.
                                    self.drag = Some(d);
                                }
                            }
                        }
                    }
                }
                Ev::Key(Key::Enter) => {
                    if !(self.lesson && self.lesson_step < 2) {
                        self.start_battle(out);
                    }
                }
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
                        self.skip_battle(out);
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
            Phase::Keep { cands, mut keep } => match ev {
                Ev::Move(x, y) => self.hover_item = (0..cands.len()).find(|&i| keep_rect(i).contains(x, y)),
                Ev::Press { x, y, right: false } => {
                    if KEEP_DONE.contains(x, y) {
                        self.finish_keep(out);
                        return;
                    }
                    if let Some(i) = (0..cands.len()).find(|&i| keep_rect(i).contains(x, y)) {
                        let count = keep.iter().filter(|k| **k).count();
                        if keep[i] {
                            keep[i] = false;
                            out.sfx(Sfx::Lift);
                        } else if count < self.band_limit() {
                            keep[i] = true;
                            out.sfx(Sfx::Place);
                        } else {
                            out.sfx(Sfx::Deny);
                            self.message = Some(format!("YOUR BAND HOLDS {}. LET SOMEONE GO FIRST.", self.band_limit()));
                        }
                        self.phase = Phase::Keep { cands, keep };
                    }
                }
                Ev::Key(Key::Enter) => self.finish_keep(out),
                _ => {}
            },
            Phase::Idea { offers } => match ev {
                Ev::Move(x, y) => self.hover_item = (0..offers.len()).find(|&i| idea_rect(i).contains(x, y)),
                Ev::Press { x, y, right: false } => {
                    if let Some(i) = (0..offers.len()).find(|&i| idea_rect(i).contains(x, y)) {
                        self.pick_idea(i, out);
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
        if self.phase == Phase::Deploy {
            self.think += 1;
        }
        for p in self.particles.iter_mut() {
            p.0 += p.2;
            p.1 += p.3;
            p.4 -= 1;
        }
        self.particles.retain(|p| p.4 > 0);
        match &mut self.phase {
            Phase::Battle => self.battle_tick(out),
            Phase::After { t, .. } => *t += 1,
            _ => {}
        }
    }

    fn draw(&self, fb: &mut Framebuffer) {
        render::draw(self, fb);
    }

    fn palette(&self) -> &Pal {
        &self.pals[self.age()]
    }

    fn ended(&self) -> bool {
        self.ended
    }

    fn summary(&self) -> Summary {
        let headline = match self.phase {
            Phase::Over { won: true } => "A MYTH IS WON OVER. HISTORY CARRIES YOUR BANNER.".to_string(),
            _ if self.lesson => "STOPPED DURING THE LESSON BATTLE.".to_string(),
            Phase::Over { won: false } => format!("OUT OF CHANCES IN {}, BATTLE {} OF {BATTLES}.", AGES[self.age()], self.index + 1),
            _ => format!("STOPPED IN {}, BATTLE {} OF {BATTLES}.", AGES[self.age()], self.index + 1),
        };
        let s = &self.stats;
        let ideas = if self.ideas.is_empty() { "NO IDEAS".to_string() } else { format!("IDEAS: {}", self.ideas.iter().map(|i| i.name()).collect::<Vec<_>>().join(", ")) };
        let detail = format!(
            "WON {}, LOST {}, DREW {}. {} TURNED TO YOU, {} TURNED AWAY. BAND OF {}. {ideas}.",
            s.won,
            s.lost,
            s.drawn,
            s.turned_to_us,
            s.turned_from_us,
            self.band.len()
        );
        let won = matches!(self.phase, Phase::Over { won: true });
        Summary { headline, detail, best_key: self.index as u32 + 1 + if won { 100 } else { 0 } }
    }

    fn takes_esc(&self) -> bool {
        self.show_help
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Places the band front-first across the lit rows and plays every battle out.
    fn drive(g: &mut Game, out: &mut Out, max: usize) {
        for _ in 0..max {
            if g.ended {
                return;
            }
            match g.phase.clone() {
                Phase::Route { .. } => {
                    let i = g.rng.below(g.encounters.len() as u32) as usize;
                    g.choose_route(i, out);
                }
                Phase::Deploy => {
                    if g.lesson {
                        g.lesson_step = 2;
                    }
                    g.placed.clear();
                    let mut spots: Vec<Pos> = (0..W).flat_map(|x| [(x, 3), (x, 4)]).collect();
                    g.rng.shuffle(&mut spots);
                    let n = g.deploy_limit().min(g.band.len());
                    let mut order: Vec<usize> = (0..g.band.len()).collect();
                    order.sort_by_key(|&m| -(g.band[m].kind.info().resolve + g.band[m].kind.info().voice * 3));
                    for (k, &m) in order.iter().take(n).enumerate() {
                        g.placed.push((m, spots[k]));
                    }
                    g.start_battle(out);
                }
                Phase::Battle => {
                    g.note = None;
                    g.speed = 4;
                    g.tick(out);
                }
                Phase::After { .. } => g.after_done(out),
                Phase::Keep { .. } => g.finish_keep(out),
                Phase::Idea { .. } => g.pick_idea(0, out),
                Phase::Over { .. } => g.ended = true,
            }
        }
    }

    #[test]
    fn lesson_then_runs_play_to_an_end() {
        let mut results = Vec::new();
        for seed in 0..10 {
            let mut g = Game::new(seed, seed == 0);
            let mut out = Out::default();
            drive(&mut g, &mut out, 200_000);
            assert!(g.ended, "seed {seed} stuck in {:?}", g.phase);
            results.push((g.index + 1, g.stats.won, g.stats.lost, matches!(g.phase, Phase::Over { won: true })));
        }
        println!("turncoat bot results (battle reached, won, lost, victory): {results:?}");
    }

    #[test]
    fn rivals_never_overlap() {
        let mut g = Game::new(3, false);
        for i in 0..13 {
            g.index = i;
            for heavy in [false, true] {
                let e = g.gen_encounter(heavy);
                let mut seen = std::collections::HashSet::new();
                assert!(e.rivals.iter().all(|r| seen.insert(r.1) && r.1 .1 <= 2));
            }
        }
    }
}
