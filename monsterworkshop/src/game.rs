//! Monsters, input, record/replay and the crafting rules, advanced in discrete 15 FPS ticks.
//!
//! A move updates the logical tile at once and then plays out as a short slide, so input is
//! answered on the same tick and collisions are settled before anything is drawn.

use crate::tutorial::{self, Tutorial};
use crate::world::*;

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

impl Species {
    pub fn name(self) -> &'static str {
        match self {
            Species::Goblin => "GOBLIN",
            Species::Cyclops => "CYCLOPS",
        }
    }
}

/// One recorded action. A step remembers what it pushed so replay can tell "the cotton
/// isn't here yet" (wait) from "something else is in the way" (also wait, but say so).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Act {
    Step(Dir, Option<Kind>),
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
    pub name: &'static str,
    pub species: Species,
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
    pub wait_reason: &'static str,
    pub loops: u32,
}

impl Monster {
    fn new(name: &'static str, species: Species, (x, y): (i32, i32)) -> Self {
        Self {
            name,
            species,
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
            wait_reason: "",
            loops: 0,
        }
    }

    fn idle(&self) -> bool {
        self.slide.is_none() && !self.busy
    }
}

enum StepRes {
    Moved(Option<Kind>),
    Blocked(&'static str),
}

enum UseRes {
    Done,
    Wait(&'static str),
    Nothing,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Ev {
    Walked,
    Delivered(Kind),
    Switched,
}

pub struct Game {
    pub world: World,
    pub monsters: Vec<Monster>,
    pub controlled: usize,
    pub frame: u32,
    pub crt: bool,
    pub help: bool,
    pub tutorial: Tutorial,
    pub delivered: u32,
    queued: Option<Dir>,
    queued_use: bool,
    hold: Option<(Dir, u8)>, // direction held since its press, and for how many ticks
    events: Vec<Ev>,
}

impl Game {
    pub fn new() -> Self {
        let world = World::new();
        let mut monsters = vec![
            Monster::new("MOGG", Species::Goblin, world.starts[0]),
            Monster::new("BRUNA", Species::Cyclops, world.starts[1]),
        ];
        monsters[0].bubble = Some(Bubble { text: "HI, I'M MOGG! LET'S SPIN SOME THREAD.".into(), ttl: 60, warn: false, waiting: false });
        Self {
            world,
            monsters,
            controlled: 0,
            frame: 0,
            crt: true,
            help: false,
            tutorial: Tutorial::default(),
            delivered: 0,
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
        self.tick_world(sfx);
        for m in self.monsters.iter_mut() {
            if let Some(b) = m.bubble.as_mut() {
                b.ttl = b.ttl.saturating_sub(1);
            }
            if m.bubble.as_ref().is_some_and(|b| b.ttl == 0) {
                m.bubble = None;
            }
        }
        self.advance_tutorial();
    }

    fn say(&mut self, i: usize, text: &str, warn: bool) {
        self.monsters[i].bubble = Some(Bubble { text: text.to_string(), ttl: BUBBLE_TICKS, warn, waiting: false });
    }

    pub fn monster_at(&self, x: i32, y: i32) -> Option<usize> {
        self.monsters.iter().position(|m| m.x == x && m.y == y)
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
                self.say(c, "SPINNING! TAB SWITCHES MONSTER.", false);
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
                    self.say(c, msg, true);
                    sfx.push(Sfx::Deny);
                }
                UseRes::Nothing => self.say(c, "NOTHING TO USE HERE.", false),
            }
            return;
        }

        let pressed_now = self.queued.take().or(fresh);
        let repeat = self.hold.filter(|&(_, t)| t >= HOLD_REPEAT_TICKS).map(|(d, _)| d);
        let want = pressed_now.or(repeat);
        let Some(d) = want else { return };
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
                        self.say(c, msg, true);
                    }
                }
            }
        }
    }

    fn try_step(&mut self, i: usize, d: Dir, sfx: &mut Vec<Sfx>) -> StepRes {
        let (dx, dy) = d.delta();
        let (x, y) = (self.monsters[i].x, self.monsters[i].y);
        let (tx, ty) = (x + dx, y + dy);
        self.monsters[i].facing = d;
        if dx != 0 {
            self.monsters[i].flip = dx < 0;
        }
        if !self.world.tile(tx, ty).walkable() || self.monster_at(tx, ty).is_some() {
            return StepRes::Blocked("");
        }
        let mut pushed = None;
        if let Some(it) = self.world.item_at(tx, ty) {
            let kind = self.world.items[it].kind;
            if self.world.items[it].locked {
                return StepRes::Blocked("THAT'S BEING SPUN.");
            }
            let (bx, by) = (tx + dx, ty + dy);
            if let Some(b) = self.world.bin_at(bx, by) {
                if self.world.bins[b].kind != kind {
                    return StepRes::Blocked(match self.world.bins[b].kind {
                        Kind::Cotton => "ONLY COTTON GOES IN THERE.",
                        Kind::Thread => "ONLY THREAD GOES IN THERE.",
                    });
                }
                self.world.items.remove(it);
                self.world.bins[b].count += 1;
                self.world.bins[b].bounce = 4;
                self.world.burst(bx, by, PKind::Spark, 5);
                self.events.push(Ev::Delivered(kind));
                if kind == Kind::Thread {
                    self.delivered += 1;
                }
                sfx.push(Sfx::Deposit);
            } else {
                let free = self.world.tile(bx, by).item_floor()
                    && self.world.item_at(bx, by).is_none()
                    && self.monster_at(bx, by).is_none();
                if !free {
                    return StepRes::Blocked("");
                }
                let item = &mut self.world.items[it];
                item.x = bx;
                item.y = by;
                item.hop = 0;
                item.slide = Some(Slide { dir: d, left: PUSH_TICKS, total: PUSH_TICKS });
                if let Some(p) = self.world.shop.pads.iter().position(|&pad| pad == (bx, by)) {
                    self.world.shop.pad_flash[p] = 5;
                    sfx.push(Sfx::Pad);
                }
                self.world.dust(tx, ty, d);
                sfx.push(Sfx::Push);
            }
            pushed = Some(kind);
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
        let (x, y, facing) = (self.monsters[i].x, self.monsters[i].y, self.monsters[i].facing);
        if (x, y) == self.world.shop.wheel {
            if self.world.shop.crafting.is_some() {
                return UseRes::Wait("THE WHEEL IS BUSY.");
            }
            if !self.world.pads_full() {
                return UseRes::Wait("NEED COTTON ON BOTH PADS.");
            }
            let (ox, oy) = self.world.shop.out;
            if self.world.item_at(ox, oy).is_some() || self.monster_at(ox, oy).is_some() {
                return UseRes::Wait("CLEAR THE OUTPUT MAT FIRST.");
            }
            for &(px, py) in &self.world.shop.pads {
                if let Some(it) = self.world.item_at(px, py) {
                    self.world.items[it].locked = true;
                }
            }
            self.world.shop.crafting = Some((i, 0));
            self.monsters[i].busy = true;
            sfx.push(Sfx::Spin);
            return UseRes::Done;
        }
        if let Some(p) = self.world.plant_at(x, y) {
            if !self.world.plants[p].ripe() {
                return UseRes::Wait("STILL GROWING.");
            }
            let (dx, dy) = facing.delta();
            let (fx, fy) = (x + dx, y + dy);
            let free = self.world.tile(fx, fy).item_floor()
                && self.world.item_at(fx, fy).is_none()
                && self.monster_at(fx, fy).is_none();
            if !free {
                return UseRes::Wait("NO ROOM TO DROP IT THERE.");
            }
            self.world.plants[p].growth = 0;
            let slide = Some(Slide { dir: facing, left: PUSH_TICKS, total: PUSH_TICKS });
            self.world.items.push(Item { x: fx, y: fy, kind: Kind::Cotton, slide, hop: 6, locked: false });
            self.world.burst(x, y, PKind::Leaf, 4);
            sfx.push(Sfx::Pop);
            return UseRes::Done;
        }
        UseRes::Nothing
    }

    fn replay(&mut self, i: usize, sfx: &mut Vec<Sfx>) {
        if !self.monsters[i].idle() {
            return;
        }
        let Some(act) = self.monsters[i].routine.as_ref().and_then(|r| r.acts.get(self.monsters[i].pc).copied()) else {
            self.monsters[i].playing = false;
            return;
        };
        let outcome: Result<(), &'static str> = match act {
            Act::Step(d, expect) => {
                let (dx, dy) = d.delta();
                let (tx, ty) = (self.monsters[i].x + dx, self.monsters[i].y + dy);
                let here = self.world.item_at(tx, ty).map(|k| self.world.items[k].kind);
                match (expect, here) {
                    (Some(k), Some(h)) if k == h => match self.try_step(i, d, sfx) {
                        StepRes::Moved(_) => Ok(()),
                        StepRes::Blocked(_) => Err("WAITING..."),
                    },
                    (Some(Kind::Cotton), None) => Err("WAITING FOR COTTON."),
                    (Some(Kind::Thread), None) => Err("WAITING FOR THREAD."),
                    (_, Some(_)) => Err("SOMETHING'S IN MY WAY."),
                    (None, None) => match self.try_step(i, d, sfx) {
                        StepRes::Moved(_) => Ok(()),
                        StepRes::Blocked(_) => Err("WAITING..."),
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
                m.wait_reason = "";
                m.pc += 1;
                if m.pc >= m.routine.as_ref().map_or(0, |r| r.acts.len()) {
                    m.pc = 0;
                    m.loops += 1;
                }
            }
            Err(msg) => {
                m.wait = m.wait.saturating_add(1);
                m.wait_reason = msg;
                if m.wait >= WAIT_BEFORE_SAYING {
                    m.bubble = Some(Bubble { text: msg.to_string(), ttl: 3, warn: false, waiting: true });
                }
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

    fn tick_world(&mut self, sfx: &mut Vec<Sfx>) {
        for p in self.world.plants.iter_mut() {
            p.growth = (p.growth + 1).min(REGROW_TICKS);
        }

        for b in 0..self.world.bins.len() {
            let bin = &self.world.bins[b];
            let Some((mx, my)) = bin.mouth else { continue };
            let blocked = self.world.item_at(mx, my).is_some() || self.monster_at(mx, my).is_some();
            let bin = &mut self.world.bins[b];
            bin.bounce = bin.bounce.saturating_sub(1);
            if blocked || bin.count == 0 {
                bin.timer = 0;
                continue;
            }
            bin.timer += 1;
            if bin.timer >= DISPENSE_TICKS {
                bin.timer = 0;
                bin.count -= 1;
                bin.bounce = 3;
                let dir = if my > bin.y { Dir::Down } else if my < bin.y { Dir::Up } else if mx > bin.x { Dir::Right } else { Dir::Left };
                let slide = Some(Slide { dir, left: PUSH_TICKS, total: PUSH_TICKS });
                let kind = bin.kind;
                self.world.items.push(Item { x: mx, y: my, kind, slide, hop: 4, locked: false });
                sfx.push(Sfx::Plop);
            }
        }
        for b in self.world.bins.iter_mut().filter(|b| b.mouth.is_none()) {
            b.bounce = b.bounce.saturating_sub(1);
        }

        if let Some((mi, t)) = self.world.shop.crafting {
            let t = t + 1;
            let pads = self.world.shop.pads.clone();
            if t % 3 == 0 {
                let (px, py) = pads[(t as usize / 3) % pads.len()];
                self.world.burst(px, py, PKind::Fluff, 1);
            }
            if t % 6 == 0 {
                sfx.push(Sfx::Spin);
            }
            if t >= CRAFT_TICKS {
                for &(px, py) in &pads {
                    if let Some(it) = self.world.item_at(px, py) {
                        self.world.items.remove(it);
                    }
                    self.world.burst(px, py, PKind::Fluff, 5);
                }
                let (ox, oy) = self.world.shop.out;
                self.world.items.push(Item { x: ox, y: oy, kind: Kind::Thread, slide: None, hop: 5, locked: false });
                self.world.burst(ox, oy, PKind::Spark, 10);
                self.world.shop.crafting = None;
                self.monsters[mi].busy = false;
                sfx.push(Sfx::Chime);
            } else {
                self.world.shop.crafting = Some((mi, t));
            }
        }
        for f in self.world.shop.pad_flash.iter_mut() {
            *f = f.saturating_sub(1);
        }
        self.world.tick_particles(self.frame);
    }

    fn advance_tutorial(&mut self) {
        let events = std::mem::take(&mut self.events);
        let t = &mut self.tutorial;
        t.steps += events.iter().filter(|e| **e == Ev::Walked).count() as u32;
        let thread_in_bin = self.world.bins.iter().any(|b| b.kind == Kind::Thread && b.count > 0);
        let thread_made = self.world.items.iter().any(|it| it.kind == Kind::Thread);
        let playing = self.monsters.iter().filter(|m| m.playing).count();
        let has_routine = self.monsters.iter().any(|m| m.routine.is_some());
        loop {
            let met = match t.stage {
                0 => t.steps >= 4,
                1 => self.world.pads_full() || self.world.shop.crafting.is_some() || thread_in_bin,
                2 => thread_made || thread_in_bin,
                3 => thread_in_bin,
                4 => self.monsters.iter().any(|m| m.rec.is_some()) || has_routine,
                5 => has_routine,
                6 => playing >= 1,
                7 => playing >= 1 && !self.monsters[self.controlled].playing,
                8 => events.contains(&Ev::Delivered(Kind::Cotton)) || playing >= 2,
                9 => playing >= 2,
                _ => false,
            };
            if !met || t.stage >= tutorial::DONE {
                break;
            }
            t.stage += 1;
        }
    }

    /// The controlled monster's state for the HUD, and whether it is a warning.
    pub fn status(&self, i: usize) -> (String, bool) {
        let m = &self.monsters[i];
        let who = format!("{} THE {}", m.name, m.species.name());
        let state = if m.busy {
            ("SPINNING".to_string(), false)
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
            for _ in 0..200 {
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

/// Bruna's job from the tile left of the field row: harvest, push into the cotton bin, walk back.
pub const BRUNA_JOB: &str = "RuRRRRLLLLL";

#[cfg(test)]
mod tests {
    use super::*;

    /// Mogg's job from the start tile: two cotton runs, spin, deliver, walk home.
    const MOGG_JOB: &str = "RRRRRDLLLLLU RRRRU u RRRRRRRR LLLLLLLL DDLLLLU";

    #[test]
    fn walking_into_cotton_pushes_it_one_tile() {
        let mut g = Game::new();
        let it = g.world.item_at(7, 8).expect("chute starts with a boll out");
        g.script("R");
        assert_eq!((g.world.items[it].x, g.world.items[it].y), (8, 8));
        assert_eq!((g.monsters[0].x, g.monsters[0].y), (7, 8));
    }

    #[test]
    fn a_tap_moves_one_tile_and_a_long_hold_keeps_walking() {
        let hold = |ticks: u32| {
            let mut g = Game::new();
            g.script("t"); // Bruna has open grass to the north
            let (mut req, mut sfx) = (Vec::new(), Vec::new());
            for t in 0..ticks {
                let mut inp = Input::default();
                inp.held[0] = true;
                inp.pressed[0] = t == 0;
                g.update(&inp, &mut req, &mut sfx);
            }
            g.run_ticks(10);
            g.world.starts[1].1 - g.monsters[1].y
        };
        assert_eq!(hold(1), 1);
        assert_eq!(hold(HOLD_REPEAT_TICKS as u32), 1, "a ~250 ms tap must not take a second step");
        assert!(hold(12) >= 4, "holding should keep walking");
    }

    #[test]
    fn hedges_block_and_do_not_move_anything() {
        let mut g = Game::new();
        g.script("t");
        g.script("L");
        assert_eq!((g.monsters[1].x, g.monsters[1].y), (1, 10));
    }

    #[test]
    fn manual_job_makes_and_delivers_one_thread() {
        let mut g = Game::new();
        g.script(MOGG_JOB);
        assert_eq!(g.delivered, 1);
        assert_eq!((g.monsters[0].x, g.monsters[0].y), g.world.starts[0]);
        assert_eq!(g.tutorial.stage, 4);
    }

    #[test]
    fn full_tutorial_leaves_both_loops_producing_thread() {
        let mut g = Game::new();
        g.script(MOGG_JOB);
        g.script("r");
        g.script(MOGG_JOB);
        g.script("rp");
        assert!(g.monsters[0].playing);
        g.script("tUUUr");
        g.script(BRUNA_JOB);
        g.script("rp");
        assert!(g.monsters[1].playing);
        assert_eq!(g.tutorial.stage, tutorial::DONE);

        let before = g.delivered;
        g.run_ticks(3000);
        assert!(g.delivered >= before + 8, "only {} thread in 200 s", g.delivered - before);
        assert!(g.monsters.iter().all(|m| m.playing), "a loop stopped");
    }
}
