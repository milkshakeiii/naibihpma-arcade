//! Monsters, input, pushing and record/replay, advanced in discrete 15 FPS ticks. Crafting,
//! harvesting and bins live in `work.rs`.
//!
//! A move updates the logical tile at once and then plays out as a short slide, so input is
//! answered on the same tick and collisions are settled before anything is drawn.

use crate::data::{Data, Stats};
use crate::econ::{Learned, Rng, Skills};
use crate::tutorial::{self, Tutorial};
use crate::world::*;
use std::collections::BTreeSet;

pub const WALK_TICKS: u8 = 2;
pub const PUSH_TICKS: u8 = 3;
const BUBBLE_TICKS: u16 = 45;
const WAIT_BEFORE_SAYING: u16 = 12;
/// A held direction starts walking on its own only after ~267 ms, so a tap is one tile.
const HOLD_REPEAT_TICKS: u8 = 4;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Sfx {
    Step,
    Push,
    Bump,
    Pop,
    Plop,
    Pad,
    Spin,
    Chime,
    Deposit,
    RecOn,
    RecSave,
    LoopOn,
    LoopOff,
    Switch,
    Deny,
}

impl Sfx {
    #[cfg(test)]
    pub const ALL: [Sfx; 15] = [
        Sfx::Step,
        Sfx::Push,
        Sfx::Bump,
        Sfx::Pop,
        Sfx::Plop,
        Sfx::Pad,
        Sfx::Spin,
        Sfx::Chime,
        Sfx::Deposit,
        Sfx::RecOn,
        Sfx::RecSave,
        Sfx::LoopOn,
        Sfx::LoopOff,
        Sfx::Switch,
        Sfx::Deny,
    ];
}

pub enum Request {
    Quit,
}

/// One tick of latched input. Direction arrays are indexed like `DIRS`.
#[derive(Default, Clone, Copy)]
pub struct Input {
    pub held: [bool; 4],
    pub pressed: [bool; 4],
    pub use_pressed: bool,
    pub rec_pressed: bool,
    pub play_pressed: bool,
    pub tab_pressed: bool,
    pub cancel_pressed: bool,
    pub help_pressed: bool,
    pub quit_pressed: bool,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Species {
    Goblin,
    Cyclops,
}

/// One recorded action. A step remembers which good it pushed, so replay can tell "it isn't
/// here yet" (wait) from "something else is in the way" (also wait, but say so).
#[derive(Clone, PartialEq, Debug)]
pub enum Act {
    Step(Dir, Option<String>),
    Use,
}

#[derive(Clone, Debug)]
pub struct Routine {
    pub flag: (i32, i32),
    pub acts: Vec<Act>,
}

pub struct Bubble {
    pub text: String,
    pub ttl: u16,
    pub warn: bool,
    pub waiting: bool, // a looping monster stuck on a step; drawn as a small icon when not controlled
}

pub struct Monster {
    pub id: u32,
    pub name: &'static str,
    pub kind: &'static str,
    pub species: Species,
    pub stats: Stats,
    pub skills: Skills,
    pub x: i32,
    pub y: i32,
    pub facing: Dir,
    pub flip: bool,
    pub slide: Option<Slide>,
    pub bump: Option<(Dir, u8)>,
    pub busy: bool,
    pub walk_frame: u8,
    pub bubble: Option<Bubble>,
    pub rec: Option<Routine>,
    pub routine: Option<Routine>,
    pub playing: bool,
    pub pc: usize,
    pub paused_at: Option<(i32, i32)>,
    pub wait: u16,
    pub wait_reason: String,
    pub loops: u32,
    /// What the last finished job taught, for skill popups.
    pub learned: Option<(String, Learned, u16)>,
}

impl Monster {
    fn new(id: u32, name: &'static str, kind: &'static str, data: &Data, (x, y): (i32, i32), skills: Skills) -> Self {
        let species = if kind == "goblin" { Species::Goblin } else { Species::Cyclops };
        Self {
            id,
            name,
            kind,
            species,
            stats: data.monster_types[kind].stats.clone(),
            skills,
            x,
            y,
            facing: Dir::Right,
            flip: false,
            slide: None,
            bump: None,
            busy: false,
            walk_frame: 0,
            bubble: None,
            rec: None,
            routine: None,
            playing: false,
            pc: 0,
            paused_at: None,
            wait: 0,
            wait_reason: String::new(),
            loops: 0,
            learned: None,
        }
    }

    pub fn idle(&self) -> bool {
        self.slide.is_none() && !self.busy
    }

}

fn starting_skills(transferable: &[&str], applied: &[(&str, f64)]) -> Skills {
    Skills {
        transferable: transferable.iter().map(|s| s.to_string()).collect(),
        applied: applied.iter().map(|(k, v)| (k.to_string(), *v)).collect(),
        ..Skills::default()
    }
}

pub(crate) enum StepRes {
    Moved(Option<String>),
    Blocked(String),
}

pub(crate) enum UseRes {
    Done,
    Wait(String),
    Nothing,
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub(crate) enum Ev {
    Walked,
    Made(String),
    Delivered(String),
    Switched,
}

pub struct Game {
    pub data: Data,
    pub world: World,
    pub monsters: Vec<Monster>,
    pub controlled: usize,
    pub frame: u32,
    pub crt: bool,
    pub help: bool,
    pub tutorial: Tutorial,
    pub rng: Rng,
    /// Goods made at least once (tutorial progress).
    pub made: BTreeSet<String>,
    queued: Option<Dir>,
    queued_use: bool,
    hold: Option<(Dir, u8)>, // direction held since its press, and for how many ticks
    pub(crate) events: Vec<Ev>,
}

impl Game {
    pub fn new() -> Self {
        let data = Data::load();
        let world = World::new(&data);
        let mogg = starting_skills(&["handcrafts", "science", "engineering"], &[("spinning", 0.3), ("textiles", 0.3), ("weaving", 0.2)]);
        let bruna = starting_skills(&["outdoorsmonstership", "athletics", "social"], &[("harvesting", 0.3), ("gathering", 0.3)]);
        let mut monsters = vec![
            Monster::new(1, "MOGG", "goblin", &data, world.starts[0], mogg),
            Monster::new(2, "BRUNA", "cyclops", &data, world.starts[1], bruna),
        ];
        monsters[0].bubble = Some(Bubble { text: "HI, I'M MOGG! LET'S MAKE SOME CLOTH.".into(), ttl: 60, warn: false, waiting: false });
        Self {
            data,
            world,
            monsters,
            controlled: 0,
            frame: 0,
            crt: true,
            help: false,
            tutorial: Tutorial::default(),
            rng: Rng::new(0x5EED_2026),
            made: BTreeSet::new(),
            queued: None,
            queued_use: false,
            hold: None,
            events: Vec::new(),
        }
    }

    pub fn update(&mut self, inp: &Input, req: &mut Vec<Request>, sfx: &mut Vec<Sfx>) {
        self.frame = self.frame.wrapping_add(1);
        if inp.quit_pressed {
            req.push(Request::Quit);
            return;
        }
        if inp.help_pressed {
            self.help = !self.help;
        }
        if inp.tab_pressed {
            self.controlled = (self.controlled + 1) % self.monsters.len();
            self.queued = None;
            self.queued_use = false;
            self.events.push(Ev::Switched);
            sfx.push(Sfx::Switch);
        }
        self.player_input(inp, sfx);
        for i in 0..self.monsters.len() {
            if self.monsters[i].playing {
                self.replay(i, sfx);
            }
        }
        self.advance_motion();
        self.tick_work(sfx);
        self.world.tick_particles(self.frame);
        for m in self.monsters.iter_mut() {
            if let Some(b) = m.bubble.as_mut() {
                b.ttl = b.ttl.saturating_sub(1);
            }
            if m.bubble.as_ref().is_some_and(|b| b.ttl == 0) {
                m.bubble = None;
            }
            if let Some((_, _, t)) = m.learned.as_mut() {
                *t = t.saturating_sub(1);
            }
            if m.learned.as_ref().is_some_and(|l| l.2 == 0) {
                m.learned = None;
            }
        }
        self.advance_tutorial();
    }

    pub(crate) fn say(&mut self, i: usize, text: &str, warn: bool) {
        self.monsters[i].bubble = Some(Bubble { text: text.to_string(), ttl: BUBBLE_TICKS, warn, waiting: false });
    }

    pub fn monster_at(&self, x: i32, y: i32) -> Option<usize> {
        self.monsters.iter().position(|m| m.x == x && m.y == y)
    }

    pub fn good_name(&self, id: &str) -> String {
        self.data.try_good(id).map_or_else(|| id.to_uppercase(), |g| g.name.to_uppercase())
    }

    fn player_input(&mut self, inp: &Input, sfx: &mut Vec<Sfx>) {
        let c = self.controlled;
        let held = |d: Dir| inp.held[DIRS.iter().position(|&x| x == d).unwrap_or(0)];
        let mut fresh = None;
        for (k, d) in DIRS.iter().enumerate() {
            if inp.pressed[k] {
                fresh = Some(*d);
            }
        }
        self.hold = match (fresh, self.hold) {
            (Some(d), _) => Some((d, 0)),
            (None, Some((d, t))) if held(d) => Some((d, t.saturating_add(1))),
            // Released, but another direction is still down: keep walking that way.
            (None, Some((_, t))) => DIRS.iter().copied().find(|&d| held(d)).map(|d| (d, t)),
            (None, None) => None,
        };
        if inp.rec_pressed {
            self.toggle_record(c, sfx);
        }
        if inp.cancel_pressed && self.monsters[c].rec.take().is_some() {
            self.say(c, "RECORDING CANCELLED.", false);
            sfx.push(Sfx::Deny);
        }
        if inp.play_pressed {
            self.toggle_play(c, sfx);
        }

        let m = &self.monsters[c];
        if m.playing {
            if fresh.is_some() || inp.use_pressed {
                self.say(c, "I'M ON A LOOP. P STOPS ME.", true);
            }
            return;
        }
        if m.busy {
            if fresh.is_some() || inp.use_pressed {
                self.say(c, "WORKING! TAB SWITCHES MONSTER.", false);
            }
            return;
        }
        if m.slide.is_some() {
            if fresh.is_some() {
                self.queued = fresh;
            }
            self.queued_use |= inp.use_pressed;
            return;
        }

        if inp.use_pressed || std::mem::take(&mut self.queued_use) {
            self.queued = None;
            match self.try_use(c, sfx) {
                UseRes::Done => {
                    if let Some(r) = self.monsters[c].rec.as_mut() {
                        r.acts.push(Act::Use);
                    }
                }
                UseRes::Wait(msg) => {
                    self.say(c, &msg, true);
                    sfx.push(Sfx::Deny);
                }
                UseRes::Nothing => self.say(c, "NOTHING TO USE HERE.", false),
            }
            return;
        }

        let pressed_now = self.queued.take().or(fresh);
        let repeat = self.hold.filter(|&(_, t)| t >= HOLD_REPEAT_TICKS).map(|(d, _)| d);
        let Some(d) = pressed_now.or(repeat) else { return };
        match self.try_step(c, d, sfx) {
            StepRes::Moved(pushed) => {
                if let Some(r) = self.monsters[c].rec.as_mut() {
                    r.acts.push(Act::Step(d, pushed));
                }
                self.events.push(Ev::Walked);
            }
            StepRes::Blocked(msg) => {
                // Holding into a wall bumps once; each fresh press bumps again.
                if pressed_now.is_some() {
                    self.monsters[c].bump = Some((d, 3));
                    sfx.push(Sfx::Bump);
                    if !msg.is_empty() {
                        self.say(c, &msg, true);
                    }
                }
            }
        }
    }

    pub(crate) fn try_step(&mut self, i: usize, d: Dir, sfx: &mut Vec<Sfx>) -> StepRes {
        let (dx, dy) = d.delta();
        let (x, y) = (self.monsters[i].x, self.monsters[i].y);
        let (tx, ty) = (x + dx, y + dy);
        self.monsters[i].facing = d;
        if dx != 0 {
            self.monsters[i].flip = dx < 0;
        }
        if !self.world.tile(tx, ty).walkable() || self.monster_at(tx, ty).is_some() {
            return StepRes::Blocked(String::new());
        }
        let mut pushed = None;
        if let Some(it) = self.world.item_at(tx, ty) {
            let good = self.world.items[it].good().to_string();
            if self.world.items[it].locked {
                return StepRes::Blocked("THAT'S BEING WORKED ON.".into());
            }
            let (bx, by) = (tx + dx, ty + dy);
            if let Some(b) = self.world.bin_at(bx, by) {
                if self.world.bins[b].accepts != good {
                    return StepRes::Blocked(format!("ONLY {} GOES IN THERE.", self.good_name(&self.world.bins[b].accepts)));
                }
                let item = self.world.items.remove(it);
                let bin = &mut self.world.bins[b];
                bin.stock.push(item.data);
                bin.bounce = 4;
                self.world.burst(bx, by, PKind::Spark, 5);
                self.events.push(Ev::Delivered(good.clone()));
                sfx.push(Sfx::Deposit);
            } else {
                let free = self.world.tile(bx, by).item_floor() && self.world.item_at(bx, by).is_none() && self.monster_at(bx, by).is_none();
                if !free {
                    return StepRes::Blocked(String::new());
                }
                let item = &mut self.world.items[it];
                item.x = bx;
                item.y = by;
                item.hop = 0;
                item.slide = Some(Slide { dir: d, left: PUSH_TICKS, total: PUSH_TICKS });
                if let Some((s, k)) = self.world.pad_owner(bx, by) {
                    self.world.shops[s].pad_flash[k] = 5;
                    sfx.push(Sfx::Pad);
                }
                self.world.dust(tx, ty, d);
                sfx.push(Sfx::Push);
            }
            pushed = Some(good);
        } else if i == self.controlled {
            sfx.push(Sfx::Step);
        }
        let m = &mut self.monsters[i];
        m.x = tx;
        m.y = ty;
        let t = if pushed.is_some() { PUSH_TICKS } else { WALK_TICKS };
        m.slide = Some(Slide { dir: d, left: t, total: t });
        m.bump = None;
        StepRes::Moved(pushed)
    }

    fn try_use(&mut self, i: usize, sfx: &mut Vec<Sfx>) -> UseRes {
        let (x, y) = (self.monsters[i].x, self.monsters[i].y);
        if let Some(s) = self.world.shop_at_spot(x, y) {
            return self.use_shop(i, s, sfx);
        }
        if let Some(p) = self.world.plant_at(x, y) {
            return self.use_plant(i, p, sfx);
        }
        UseRes::Nothing
    }

    fn replay(&mut self, i: usize, sfx: &mut Vec<Sfx>) {
        if !self.monsters[i].idle() {
            return;
        }
        let Some(act) = self.monsters[i].routine.as_ref().and_then(|r| r.acts.get(self.monsters[i].pc).cloned()) else {
            self.monsters[i].playing = false;
            return;
        };
        let outcome: Result<(), String> = match act {
            Act::Step(d, expect) => {
                let (dx, dy) = d.delta();
                let (tx, ty) = (self.monsters[i].x + dx, self.monsters[i].y + dy);
                let here = self.world.item_at(tx, ty).map(|k| self.world.items[k].good().to_string());
                match (expect, here) {
                    (Some(k), Some(h)) if k == h => match self.try_step(i, d, sfx) {
                        StepRes::Moved(_) => Ok(()),
                        StepRes::Blocked(_) => Err("WAITING...".into()),
                    },
                    (Some(k), None) => Err(format!("WAITING FOR {}.", self.good_name(&k))),
                    (_, Some(_)) => Err("SOMETHING'S IN MY WAY.".into()),
                    (None, None) => match self.try_step(i, d, sfx) {
                        StepRes::Moved(_) => Ok(()),
                        StepRes::Blocked(_) => Err("WAITING...".into()),
                    },
                }
            }
            Act::Use => match self.try_use(i, sfx) {
                UseRes::Done => Ok(()),
                UseRes::Wait(msg) => Err(msg),
                UseRes::Nothing => {
                    let m = &mut self.monsters[i];
                    m.playing = false;
                    m.paused_at = Some((m.x, m.y));
                    self.say(i, "I GOT LOST. TAKE ME TO MY FLAG.", true);
                    sfx.push(Sfx::LoopOff);
                    return;
                }
            },
        };
        let m = &mut self.monsters[i];
        match outcome {
            Ok(()) => {
                if m.wait >= WAIT_BEFORE_SAYING {
                    m.bubble = None;
                }
                m.wait = 0;
                m.wait_reason.clear();
                m.pc += 1;
                if m.pc >= m.routine.as_ref().map_or(0, |r| r.acts.len()) {
                    m.pc = 0;
                    m.loops += 1;
                }
            }
            Err(msg) => {
                m.wait = m.wait.saturating_add(1);
                if m.wait >= WAIT_BEFORE_SAYING {
                    m.bubble = Some(Bubble { text: msg.clone(), ttl: 3, warn: false, waiting: true });
                }
                m.wait_reason = msg;
            }
        }
    }

    fn toggle_record(&mut self, i: usize, sfx: &mut Vec<Sfx>) {
        let m = &mut self.monsters[i];
        if m.playing {
            self.say(i, "STOP MY LOOP FIRST (P).", true);
            sfx.push(Sfx::Deny);
            return;
        }
        let pos = (m.x, m.y);
        match m.rec.take() {
            None => {
                m.rec = Some(Routine { flag: pos, acts: Vec::new() });
                self.say(i, "RECORDING! BRING ME BACK HERE, THEN R.", false);
                sfx.push(Sfx::RecOn);
            }
            Some(r) if r.flag != pos => {
                m.rec = Some(r);
                self.say(i, "STAND ON THE FLAG TO SAVE.", true);
                sfx.push(Sfx::Deny);
            }
            Some(r) if r.acts.is_empty() => self.say(i, "NOTHING RECORDED YET.", false),
            Some(r) => {
                m.routine = Some(r);
                m.pc = 0;
                m.paused_at = None;
                self.say(i, "GOT IT! PRESS P TO LOOP.", false);
                sfx.push(Sfx::RecSave);
            }
        }
    }

    fn toggle_play(&mut self, i: usize, sfx: &mut Vec<Sfx>) {
        let m = &mut self.monsters[i];
        if m.rec.is_some() {
            self.say(i, "FINISH RECORDING FIRST (R).", true);
            sfx.push(Sfx::Deny);
            return;
        }
        if m.playing {
            m.playing = false;
            m.paused_at = Some((m.x, m.y));
            self.say(i, "TAKING A BREAK.", false);
            sfx.push(Sfx::LoopOff);
            return;
        }
        let pos = (m.x, m.y);
        let Some(flag) = m.routine.as_ref().map(|r| r.flag) else {
            self.say(i, "NO ROUTINE YET. R RECORDS ONE.", true);
            sfx.push(Sfx::Deny);
            return;
        };
        if m.paused_at == Some(pos) && m.pc > 0 {
            // Resume where the loop was stopped.
        } else if pos == flag {
            m.pc = 0;
        } else {
            self.say(i, "TAKE ME TO MY FLAG FIRST.", true);
            sfx.push(Sfx::Deny);
            return;
        }
        m.playing = true;
        m.wait = 0;
        self.say(i, "ON IT!", false);
        sfx.push(Sfx::LoopOn);
    }

    fn advance_motion(&mut self) {
        for m in self.monsters.iter_mut() {
            if let Some(s) = m.slide.as_mut() {
                s.left -= 1;
                m.walk_frame ^= 1;
                if s.left == 0 {
                    m.slide = None;
                }
            }
            if let Some((_, t)) = m.bump.as_mut() {
                *t -= 1;
                if *t == 0 {
                    m.bump = None;
                }
            }
        }
        for it in self.world.items.iter_mut() {
            if let Some(s) = it.slide.as_mut() {
                s.left -= 1;
                if s.left == 0 {
                    it.slide = None;
                }
            }
            it.hop = it.hop.saturating_sub(1);
        }
    }

    fn advance_tutorial(&mut self) {
        let events = std::mem::take(&mut self.events);
        for e in &events {
            if let Ev::Made(g) = e {
                self.made.insert(g.clone());
            }
        }
        let t = &mut self.tutorial;
        t.steps += events.iter().filter(|e| **e == Ev::Walked).count() as u32;
        let stored = |good: &str| self.world.bins.iter().any(|b| b.accepts == good && !b.stock.is_empty());
        let playing = self.monsters.iter().filter(|m| m.playing).count();
        let has_routine = self.monsters.iter().any(|m| m.routine.is_some());
        loop {
            let met = match t.stage {
                0 => t.steps >= 4,
                1 => self.made.contains("cotton_bolls"),
                2 => stored("cotton_bolls") || self.made.contains("cotton_slivers"),
                3 => self.made.contains("cotton_slivers"),
                4 => self.made.contains("thread"),
                5 => self.made.contains("fabric"),
                6 => stored("fabric"),
                7 => has_routine,
                8 => playing >= 2,
                _ => false,
            };
            if !met || t.stage >= tutorial::DONE {
                break;
            }
            t.stage += 1;
        }
    }

    /// Fabric in the cloth store: the HUD's running tally for now.
    pub fn cloth_stored(&self) -> usize {
        self.world.bins.iter().filter(|b| b.accepts == "fabric").map(|b| b.stock.len()).sum()
    }

    /// The controlled monster's state for the HUD, and whether it is a warning.
    pub fn status(&self, i: usize) -> (String, bool) {
        let m = &self.monsters[i];
        let who = format!("{} THE {}", m.name, m.kind.to_uppercase());
        let state = if m.busy {
            ("WORKING".to_string(), false)
        } else if let Some(r) = &m.rec {
            (format!("RECORDING ({} ACTIONS)", r.acts.len()), false)
        } else if m.playing && m.wait >= WAIT_BEFORE_SAYING {
            (m.wait_reason.trim_end_matches('.').to_string(), true)
        } else if m.playing {
            (format!("LOOPING, {} DONE", m.loops), false)
        } else if m.routine.is_some() {
            ("IDLE. P RUNS THE SAVED LOOP".to_string(), false)
        } else {
            ("IDLE".to_string(), false)
        };
        (format!("{who}: {}", state.0), state.1)
    }

    /// Scripted play for tests and screenshots: `U D L R` move, `u` use, `r` record,
    /// `p` play, `t` tab, `x` cancel, `h` help. Waits for each action to finish.
    pub fn script(&mut self, keys: &str) {
        let (mut req, mut sfx) = (Vec::new(), Vec::new());
        for ch in keys.chars() {
            let mut inp = Input::default();
            match ch {
                'U' | 'D' | 'L' | 'R' => {
                    let k = "UDLR".find(ch).unwrap_or(0);
                    inp.pressed[k] = true;
                    inp.held[k] = true;
                }
                'u' => inp.use_pressed = true,
                'r' => inp.rec_pressed = true,
                'p' => inp.play_pressed = true,
                't' => inp.tab_pressed = true,
                'x' => inp.cancel_pressed = true,
                'h' => inp.help_pressed = true,
                _ => continue,
            }
            self.update(&inp, &mut req, &mut sfx);
            for _ in 0..2000 {
                if self.monsters[self.controlled].idle() {
                    break;
                }
                self.update(&Input::default(), &mut req, &mut sfx);
            }
        }
    }

    pub fn run_ticks(&mut self, n: u32) {
        let (mut req, mut sfx) = (Vec::new(), Vec::new());
        for _ in 0..n {
            self.update(&Input::default(), &mut req, &mut sfx);
        }
    }
}

/// Scripted routes on the Pinewater map, for tests.
#[cfg(test)]
pub mod routes {
    /// Mogg, from the start west of the fields: harvest the east end of the bottom row and
    /// push the boll into the cotton bin.
    pub const MOGG_HARVEST: &str = "RRRRRRu RRR";
    pub const MOGG_HOME: &str = "LLLLLLLLL";

    /// Bruna, from beside the chute: workbench, spinnery, loom house, cloth store.
    pub fn bruna_chain() -> String {
        format!("RDRRUu{}URuD{}UR{}LDuR{}", "R".repeat(15), "R".repeat(13), "D".repeat(12), "D".repeat(6))
    }

    /// From the cloth store back to the flag beside the chute, staying clear of the chute.
    pub fn bruna_home() -> String {
        format!("{}{}D{}U", "U".repeat(18), "L".repeat(28), "L".repeat(5))
    }
}

#[cfg(test)]
#[path = "game_tests.rs"]
mod tests;
