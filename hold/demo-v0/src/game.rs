//! The game around the simulation: the title, the play screen and its panel, the guided first
//! minute, the feedback bar and the end card. Everything is drawn into the 400 x 300 indexed
//! framebuffer at 15 FPS; input arrives latched, in order, and is applied at the next frame.
//!
//! Controls work Majesty-style with the mouse (click to select, buttons in the panel, click the
//! map to place), and every action also has a key: the yellow first letter of each button, Tab to
//! step through things, arrows to move a map cursor, Enter to choose, Esc to back out.

use crate::art::*;
use crate::fb::{Fb, FB_W};
use crate::map::{PLAY_H, TILE};
use crate::sim::*;

pub const K_TAB: u8 = 1;
pub const K_ENTER: u8 = 2;
pub const K_ESC: u8 = 3;
pub const K_BACK: u8 = 4;
pub const K_UP: u8 = 5;
pub const K_DOWN: u8 = 6;
pub const K_LEFT: u8 = 7;
pub const K_RIGHT: u8 = 8;
pub const K_F2: u8 = 9;

/// Sound effects raised this frame, as bits.
pub mod sfx {
    pub const CLICK: u32 = 1;
    pub const POST: u32 = 2;
    pub const ACCEPT: u32 = 4;
    pub const REFUSE: u32 = 8;
    pub const HIT: u32 = 16;
    pub const PAID: u32 = 32;
    pub const HIRE: u32 = 64;
    pub const RAID: u32 = 128;
    pub const DOWN: u32 = 256;
    pub const BUILT: u32 = 512;
    pub const FELL: u32 = 1024;
    pub const WON: u32 = 2048;
    pub const CURSE: u32 = 4096;
    pub const LEVEL: u32 = 8192;
    pub const BOLT: u32 = 16384;
    pub const NOPE: u32 = 32768;
}

#[derive(Clone, Copy, Debug)]
pub enum In {
    Move(i32, i32),
    Down(u8, i32, i32),
    Up(u8, i32, i32),
    Key(u8, bool),
    Char(char),
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Screen {
    Title,
    Play,
    End,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Sel {
    None,
    Unit(u32),
    Bldg(u32),
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Mode {
    Normal,
    BuildMenu,
    Place(BKind),
    Target,
    Feedback,
    Note,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Act {
    Build,
    Hire,
    Reward,
    Pause,
    Feedback,
    Place(BKind),
    More,
    Less,
    Cancel,
    Keep,
    Change,
    Drop,
    Note,
    Back,
    Again,
}

struct Button {
    label: String,
    key: char,
    x: i32,
    y: i32,
    w: i32,
    act: Act,
    enabled: bool,
}

const BTN_H: i32 = 13;

/// The guided first minute.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Lesson {
    Goal,
    Elder,
    FirstReward,
    Watch,
    Hire,
    SecondReward,
    OfferMore,
    Fight,
    Done,
}

struct Float {
    text: String,
    x: i32,
    y: i32,
    t: i32,
    col: u8,
}

/// Contextual hints shown once each.
mod hint {
    pub const RAID: u32 = 1;
    pub const HEAL: u32 = 2;
    pub const CURSE: u32 = 4;
    pub const FELL: u32 = 8;
    pub const STILL: u32 = 16;
    pub const WAIT: u32 = 32;
}

pub struct Game {
    pub world: World,
    art: Art,
    bg: Vec<u8>,
    bg_curse: bool,
    fb: Fb,
    out: Vec<u8>,
    cursor_px: Vec<u8>,
    pub screen: Screen,
    mode: Mode,
    pub sel: Sel,
    inputs: Vec<In>,
    mouse: (i32, i32),
    kb: (i32, i32),
    kb_on: bool,
    pub tick: u32,
    pub paused: bool,
    pub scanlines: bool,
    lesson: Lesson,
    lesson_t: u32,
    caption: Option<(String, u32)>,
    /// Hints and news wait their turn behind whatever caption is showing.
    queue: Vec<String>,
    hints: u32,
    floats: Vec<Float>,
    bolts: Vec<(i32, i32, i32, i32, i32)>,
    sparks: Vec<(i32, i32, i32)>,
    toast: Option<(String, u32)>,
    note: String,
    /// Feedback entries waiting to be sent, one JSON object each.
    pub outbox: Vec<String>,
    pub sfx: u32,
    seed: u64,
    play_ticks: u32,
}

impl Game {
    pub fn new(seed: u64) -> Game {
        let world = World::new(seed);
        let art = Art::new();
        let bg = world.map.paint(&art);
        Game {
            world,
            art,
            bg,
            bg_curse: false,
            fb: Fb::default(),
            out: vec![],
            cursor_px: vec![],
            screen: Screen::Title,
            mode: Mode::Normal,
            sel: Sel::None,
            inputs: vec![],
            mouse: (200, 150),
            kb: (6, 8),
            kb_on: false,
            tick: 0,
            paused: false,
            scanlines: true,
            lesson: Lesson::Goal,
            lesson_t: 0,
            caption: None,
            queue: vec![],
            hints: 0,
            floats: vec![],
            bolts: vec![],
            sparks: vec![],
            toast: None,
            note: String::new(),
            outbox: vec![],
            sfx: 0,
            seed,
            play_ticks: 0,
        }
    }

    pub fn push(&mut self, i: In) {
        self.inputs.push(i);
    }

    /// Pause from outside (the page was hidden).
    pub fn pause(&mut self) {
        if self.screen == Screen::Play {
            self.paused = true;
        }
    }

    /// True while the soundtrack should play.
    pub fn wants_music(&self) -> bool {
        self.screen != Screen::Title
    }

    /// Leave the title and skip the guided minute (for Again, and for headless checks).
    pub fn skip_lesson(&mut self) {
        self.screen = Screen::Play;
        self.lesson = Lesson::Done;
        self.caption = None;
        self.queue.clear();
        self.world.end_calm();
    }

    fn restart(&mut self, lesson: bool) {
        self.seed = self.seed.wrapping_mul(6364136223846793005).wrapping_add(1);
        let scan = self.scanlines;
        let outbox = std::mem::take(&mut self.outbox);
        *self = Game::new(self.seed);
        self.scanlines = scan;
        self.outbox = outbox;
        self.screen = Screen::Play;
        if !lesson {
            self.lesson = Lesson::Done;
            self.world.end_calm();
            self.say("DRIVE THE HERONS FROM THE MARSH.", 60);
        }
    }

    // ---- the frame -----------------------------------------------------------------------------

    pub fn frame(&mut self) {
        self.tick += 1;
        let inputs = std::mem::take(&mut self.inputs);
        for i in inputs {
            self.input(i);
        }
        if self.screen == Screen::Play {
            self.lesson_step();
            let hold = matches!(
                self.lesson,
                Lesson::Goal
                    | Lesson::Elder
                    | Lesson::FirstReward
                    | Lesson::Hire
                    | Lesson::SecondReward
            );
            if !self.paused && !hold && !matches!(self.mode, Mode::Feedback | Mode::Note) {
                self.world.step();
                self.play_ticks += 1;
            }
            self.consume_events();
            if let Some(won) = self.world.over {
                if self.screen == Screen::Play {
                    self.screen = Screen::End;
                    self.mode = Mode::Normal;
                    self.sfx |= if won { sfx::WON } else { sfx::FELL };
                }
            }
            if let Sel::Unit(id) = self.sel {
                if self.world.unit(id).is_none() {
                    self.sel = Sel::None;
                }
            }
            if let Sel::Bldg(id) = self.sel {
                if self.world.bldg(id).is_none() {
                    self.sel = Sel::None;
                }
            }
        }
        for f in self.floats.iter_mut() {
            f.t -= 1;
            if f.t % 2 == 0 {
                f.y -= 1;
            }
        }
        self.floats.retain(|f| f.t > 0);
        self.bolts.iter_mut().for_each(|b| b.4 -= 1);
        self.bolts.retain(|b| b.4 > 0);
        self.sparks.iter_mut().for_each(|s| s.2 -= 1);
        self.sparks.retain(|s| s.2 > 0);
        if let Some((_, t)) = &mut self.caption {
            *t = t.saturating_sub(1);
        }
        if matches!(self.caption, Some((_, 0))) {
            self.caption = None;
        }
        if self.caption.is_none() && !self.queue.is_empty() {
            let next = self.queue.remove(0);
            self.say(&next, 75);
        }
        if let Some((_, t)) = &mut self.toast {
            *t = t.saturating_sub(1);
        }
        if matches!(self.toast, Some((_, 0))) {
            self.toast = None;
        }
        self.draw();
    }

    fn say(&mut self, s: &str, ticks: u32) {
        self.caption = Some((s.to_string(), ticks));
    }

    /// A hint or a piece of news: shown after the current caption, never over it.
    fn tell(&mut self, s: &str) {
        if self.caption.is_none() {
            self.say(s, 75);
        } else if self.queue.len() < 3 && !self.queue.iter().any(|q| q == s) {
            self.queue.push(s.to_string());
        }
    }

    fn hint(&mut self, bit: u32, s: &str) {
        if self.lesson == Lesson::Done && self.hints & bit == 0 {
            self.hints |= bit;
            self.tell(s);
        }
    }

    /// Why something costing `cost` can't be paid for right now.
    fn short_of_dew(&self) -> &'static str {
        if self.world.reserved() > 0 && self.world.dew >= 10 {
            "DEW OFFERED AS REWARDS IS SET ASIDE UNTIL PAID."
        } else {
            "NOT ENOUGH DEW."
        }
    }

    fn consume_events(&mut self) {
        let events = std::mem::take(&mut self.world.events);
        for e in events {
            match e {
                Ev::Accept { hero, .. } => {
                    self.sfx |= sfx::ACCEPT;
                    if let Some(u) = self.world.unit(hero) {
                        let (x, y) = (u.x, u.y);
                        self.float("!", x, y - 22, KEY);
                    }
                }
                Ev::Refuse { hero, wants, .. } => {
                    self.sfx |= sfx::REFUSE;
                    if let Some(u) = self.world.unit(hero) {
                        let (x, y) = (u.x, u.y);
                        self.float(&format!("{wants}?"), x, y - 22, TEXT_DIM);
                    }
                }
                Ev::Paid { hero, amount } => {
                    self.sfx |= sfx::PAID;
                    if let Some(u) = self.world.unit(hero) {
                        let (x, y) = (u.x, u.y);
                        self.float(&format!("+{amount}"), x, y - 24, DEW);
                    }
                }
                Ev::Hit { x, y, .. } => {
                    self.sfx |= sfx::HIT;
                    self.sparks.push((x, y, 3));
                }
                Ev::Bolt { x0, y0, x1, y1 } => {
                    self.sfx |= sfx::BOLT;
                    self.bolts.push((x0, y0, x1, y1, 3));
                }
                Ev::Died { .. } => {}
                Ev::Fell { id } => {
                    self.sfx |= sfx::FELL;
                    let name = self
                        .world
                        .units
                        .iter()
                        .find(|u| u.id == id)
                        .map_or("A HERO", |u| u.name);
                    if let Some(&(x, y, _)) = self.world.graves.last() {
                        self.float(&format!("{name} FELL"), x, y - 20, BAD);
                    }
                    let s = format!("{name} HAS FALLEN. HIRE ANOTHER AT THE LODGE OR HALL.");
                    self.hint(hint::FELL, &s);
                }
                Ev::Down { kind, .. } => {
                    self.sfx |= sfx::DOWN;
                    if kind == BKind::Glade {
                        self.tell("GORF'S STONE IS BROKEN. THE CURSE LIFTS.");
                    }
                    if kind == BKind::Nest && self.lesson == Lesson::Done {
                        let n = self.world.nests_left();
                        if n > 0 {
                            self.tell(&format!("A NEST IS DOWN. {n} TO GO."));
                        } else {
                            self.tell("THE NESTS ARE GONE. NOW BREAK GORF'S STONE.");
                        }
                    }
                }
                Ev::Raid { .. } => {
                    self.sfx |= sfx::RAID;
                    self.hint(
                        hint::RAID,
                        "A HERON IS RAIDING THE HOLD! A REWARD ON IT DRAWS DEFENDERS.",
                    );
                }
                Ev::Built { .. } => self.sfx |= sfx::BUILT,
                Ev::Hired { .. } => self.sfx |= sfx::HIRE,
                Ev::Healing { .. } => self.hint(hint::HEAL, "HURT HEROES HEAL AT THE STAR POND."),
                Ev::LevelUp { id } => {
                    self.sfx |= sfx::LEVEL;
                    if let Some(u) = self.world.unit(id) {
                        let (x, y) = (u.x, u.y);
                        self.float("LEVEL UP", x, y - 30, KEY);
                    }
                }
                Ev::Curse => {
                    self.sfx |= sfx::CURSE;
                    self.hint(
                        hint::CURSE,
                        "GORF'S STONE RISES. BREAK IT TOO: ITS WISPS DRAIN STILLS.",
                    );
                }
                Ev::Lapsed { .. } => {
                    self.toast("THE RAIDER GOT AWAY. ITS REWARD IS FREE AGAIN.");
                }
                Ev::Won | Ev::Lost => {}
            }
        }
        // A quiet nudge toward more dew, once.
        let stills = self
            .world
            .bldgs
            .iter()
            .filter(|b| b.alive && b.kind == BKind::Still)
            .count();
        let building = self.world.bldgs.iter().any(|b| b.alive && b.built < 100);
        if self.lesson == Lesson::Done
            && self.world.tick > self.world.calm_ended + 900
            && self.world.dew - self.world.reserved() < 20
            && stills <= 1
            && !building
        {
            self.hint(
                hint::STILL,
                "SHORT OF DEW? BUILD A DEW STILL, OR LOWER A REWARD.",
            );
        }
        let waiting = self
            .world
            .units
            .iter()
            .any(|u| u.kind.hero() && self.world.status(u).contains("WAITING"));
        if waiting {
            self.hint(
                hint::WAIT,
                "HEROES WAIT AT THE BRIDGE UNTIL ENOUGH OTHERS TAKE THE REWARD.",
            );
        }
    }

    fn float(&mut self, s: &str, x: i32, mut y: i32, col: u8) {
        while self
            .floats
            .iter()
            .any(|f| (f.x - x).abs() < 28 && (f.y - y).abs() < 8)
        {
            y -= 8;
        }
        self.floats.push(Float {
            text: s.to_string(),
            x,
            y,
            t: 24,
            col,
        });
    }

    // ---- the lesson ----------------------------------------------------------------------------

    fn weak_nest(&self) -> Option<u32> {
        self.world
            .bldgs
            .iter()
            .find(|b| b.weak && b.alive)
            .map(|b| b.id)
    }

    fn second_nest(&self) -> Option<u32> {
        self.world
            .bldgs
            .iter()
            .find(|b| b.alive && b.kind == BKind::Nest && !b.weak && b.ty < 8)
            .map(|b| b.id)
    }

    fn toad_hall(&self) -> Option<u32> {
        self.world
            .bldgs
            .iter()
            .find(|b| b.alive && b.kind == BKind::ToadHall)
            .map(|b| b.id)
    }

    fn first_toad(&self) -> Option<u32> {
        self.world
            .units
            .iter()
            .find(|u| u.kind == Kind::Toad)
            .map(|u| u.id)
    }

    fn lesson_step(&mut self) {
        let t = self.tick - self.lesson_t;
        let next = match self.lesson {
            Lesson::Goal => {
                if t == 1 {
                    self.say("DRIVE THE HERONS FROM THE MARSH.", 999);
                }
                (t > 40).then_some(Lesson::Elder)
            }
            Lesson::Elder => {
                if t == 1 {
                    self.say(
                        "YOU ARE THE ELDER. YOU CAN'T GIVE ORDERS, ONLY REWARDS.",
                        999,
                    );
                }
                (t > 50).then_some(Lesson::FirstReward)
            }
            Lesson::FirstReward => {
                if t == 1 {
                    self.say(
                        "CLICK THE MARKED NEST, THEN REWARD. (OR TAB TO IT, THEN R)",
                        9999,
                    );
                }
                let n = self.weak_nest();
                n.is_some_and(|n| self.world.flag_on(Tgt::Bldg(n)).is_some())
                    .then_some(Lesson::Watch)
            }
            Lesson::Watch => {
                if t == 1 {
                    self.say("HEROES DECIDE FOR THEMSELVES.", 9999);
                    self.sel = Sel::None;
                }
                self.weak_nest().is_none().then_some(Lesson::Hire)
            }
            Lesson::Hire => {
                if t == 1 {
                    self.say("PAID! NOW CLICK THE TOAD HALL AND HIRE A TOAD.", 9999);
                }
                self.first_toad().is_some().then_some(Lesson::SecondReward)
            }
            Lesson::SecondReward => {
                if t == 1 {
                    self.say("NOW PUT A REWARD ON THE MARKED NEST.", 9999);
                }
                let n = self.second_nest();
                n.is_some_and(|n| self.world.flag_on(Tgt::Bldg(n)).is_some())
                    .then_some(Lesson::OfferMore)
            }
            Lesson::OfferMore => {
                let toad = self.first_toad();
                let n = self.second_nest();
                let flag = n.and_then(|n| self.world.flag_on(Tgt::Bldg(n)));
                let taken = flag.is_some_and(|f| toad.is_some_and(|t| f.takers.contains(&t)));
                let flag = flag.is_some();
                if t == 1 {
                    if let Some(n) = n {
                        self.sel = Sel::Bldg(n);
                    }
                }
                let name = toad
                    .and_then(|t| self.world.unit(t))
                    .map_or("THE TOAD", |u| u.name)
                    .to_string();
                // What the toad would take right now, with whoever has already signed up.
                let wants = match (
                    toad.and_then(|t| self.world.unit(t)),
                    n.and_then(|n| self.world.flag_on(Tgt::Bldg(n))),
                ) {
                    (Some(u), Some(f)) => {
                        let ask = self.world.asking(u, f);
                        (ask + REWARD_STEP - 1) / REWARD_STEP * REWARD_STEP
                    }
                    _ => 0,
                };
                let refused = toad
                    .and_then(|t| self.world.unit(t))
                    .is_some_and(|u| u.wants > 0);
                if refused && !taken {
                    let _ = wants;
                    let s = format!("{name} WANTS MORE FOR THAT RISK. PRESS MORE.");
                    if self.caption.as_ref().map(|c| c.0 != s).unwrap_or(true) {
                        self.say(&s, 9999);
                    }
                } else if t == 1 {
                    self.say("WATCH WHO TAKES IT.", 9999);
                }
                if !flag && n.is_some() && t > 1 {
                    // They cancelled it: ask again.
                    Some(Lesson::SecondReward)
                } else {
                    taken.then_some(Lesson::Fight)
                }
            }
            Lesson::Fight => {
                if t == 1 {
                    self.world.end_calm();
                    self.say("MORE DEW, MORE TAKERS. NOW THE HERONS FIGHT BACK.", 75);
                }
                (t > 75).then_some(Lesson::Done)
            }
            Lesson::Done => None,
        };
        if let Some(n) = next {
            self.lesson = n;
            self.lesson_t = self.tick;
        }
    }

    /// What the lesson wants the player to touch now: a map target and a button.
    fn lesson_marks(&self) -> (Option<Tgt>, Option<Act>) {
        match self.lesson {
            Lesson::FirstReward => (self.weak_nest().map(Tgt::Bldg), Some(Act::Reward)),
            Lesson::Hire => (self.toad_hall().map(Tgt::Bldg), Some(Act::Hire)),
            Lesson::SecondReward => (self.second_nest().map(Tgt::Bldg), Some(Act::Reward)),
            Lesson::OfferMore => {
                let refused = self
                    .first_toad()
                    .and_then(|t| self.world.unit(t))
                    .is_some_and(|u| u.wants > 0);
                (None, refused.then_some(Act::More))
            }
            _ => (None, None),
        }
    }

    /// During the lesson, rewards go only where it asks.
    fn lesson_allows(&self, t: Tgt) -> bool {
        match self.lesson {
            Lesson::Goal | Lesson::Elder | Lesson::Watch | Lesson::Hire => false,
            Lesson::FirstReward => Some(t) == self.weak_nest().map(Tgt::Bldg),
            Lesson::SecondReward | Lesson::OfferMore => {
                Some(t) == self.second_nest().map(Tgt::Bldg)
            }
            _ => true,
        }
    }

    // ---- input ---------------------------------------------------------------------------------

    fn input(&mut self, i: In) {
        match self.screen {
            Screen::Title => {
                if matches!(i, In::Down(..) | In::Key(K_ENTER, _) | In::Char(' ')) {
                    self.screen = Screen::Play;
                    self.lesson_t = self.tick;
                    self.sfx |= sfx::CLICK;
                }
                if let In::Key(K_F2, _) = i {
                    self.scanlines = !self.scanlines;
                }
                if let In::Move(x, y) = i {
                    self.mouse = (x, y);
                }
            }
            Screen::Play | Screen::End => self.play_input(i),
        }
    }

    fn play_input(&mut self, i: In) {
        match i {
            In::Move(x, y) => {
                self.mouse = (x, y);
                if matches!(self.mode, Mode::Place(_)) && y < PLAY_H {
                    self.kb_on = false;
                }
            }
            In::Up(..) => {}
            In::Down(b, x, y) => {
                self.mouse = (x, y);
                if b == 2 {
                    self.back();
                    return;
                }
                if let Some(act) = self.button_at(x, y) {
                    self.act(act);
                    return;
                }
                if self.screen == Screen::Play
                    && y < PLAY_H
                    && !matches!(self.mode, Mode::Feedback | Mode::Note)
                {
                    self.kb_on = false;
                    self.choose_at(x, y);
                }
            }
            In::Key(code, shift) => self.key(code, shift),
            In::Char(c) => self.char(c),
        }
    }

    fn key(&mut self, code: u8, shift: bool) {
        if self.mode == Mode::Note {
            match code {
                K_ENTER => {
                    let text = std::mem::take(&mut self.note);
                    if !text.trim().is_empty() {
                        self.send("note", &text);
                    }
                    self.mode = Mode::Normal;
                }
                K_ESC => {
                    self.note.clear();
                    self.mode = Mode::Feedback;
                }
                K_BACK => {
                    self.note.pop();
                }
                _ => {}
            }
            return;
        }
        match code {
            K_F2 => self.scanlines = !self.scanlines,
            K_ESC => self.back(),
            K_TAB if self.screen == Screen::Play => self.cycle(if shift { -1 } else { 1 }),
            K_UP | K_DOWN | K_LEFT | K_RIGHT if self.screen == Screen::Play => {
                if !self.kb_on {
                    self.kb_on = true;
                    if !matches!(self.mode, Mode::Place(_)) {
                        self.kb = self.focus_tile();
                    }
                } else {
                    let (dx, dy) = match code {
                        K_UP => (0, -1),
                        K_DOWN => (0, 1),
                        K_LEFT => (-1, 0),
                        _ => (1, 0),
                    };
                    self.kb = ((self.kb.0 + dx).clamp(0, 24), (self.kb.1 + dy).clamp(0, 14));
                }
            }
            K_ENTER => {
                if self.screen == Screen::End {
                    self.act(Act::Again);
                } else if self.kb_on && !matches!(self.mode, Mode::Feedback) {
                    let (x, y) = (self.kb.0 * TILE + 8, self.kb.1 * TILE + 8);
                    self.choose_at(x, y);
                }
            }
            _ => {}
        }
    }

    fn char(&mut self, c: char) {
        if self.mode == Mode::Note {
            if self.note.chars().count() < 120 && (c.is_ascii_graphic() || c == ' ') {
                self.note.push(c);
            }
            return;
        }
        if self.screen == Screen::Play && c == ' ' && !matches!(self.mode, Mode::Feedback) {
            self.act(Act::Pause);
            return;
        }
        if matches!(c, '+' | '=') {
            self.act(Act::More);
            return;
        }
        if c == '-' {
            self.act(Act::Less);
            return;
        }
        let up = c.to_ascii_uppercase();
        let hit = self
            .buttons()
            .into_iter()
            .find(|b| b.key == up)
            .map(|b| b.act);
        if let Some(a) = hit {
            self.act(a);
        }
    }

    /// Esc or right-click: step back out of whatever is open.
    fn back(&mut self) {
        self.mode = match self.mode {
            Mode::Note => Mode::Feedback,
            Mode::Place(_) => Mode::BuildMenu,
            Mode::Normal => {
                self.sel = Sel::None;
                self.kb_on = false;
                Mode::Normal
            }
            _ => Mode::Normal,
        };
    }

    fn focus_tile(&self) -> (i32, i32) {
        match self.sel {
            Sel::Unit(id) => self
                .world
                .unit(id)
                .map_or(self.kb, |u| (u.x / TILE, (u.y - 4) / TILE)),
            Sel::Bldg(id) => self.world.bldg(id).map_or(self.kb, |b| (b.tx, b.ty)),
            Sel::None => (6, 8),
        }
    }

    /// Click or Enter on the map at logical pixel (x, y).
    fn choose_at(&mut self, x: i32, y: i32) {
        match self.mode {
            Mode::Place(kind) => {
                let (tx, ty) = self.place_tile(x, y);
                if self.world.build(kind, tx, ty).is_some() {
                    self.sfx |= sfx::CLICK;
                    self.mode = Mode::Normal;
                } else {
                    self.sfx |= sfx::NOPE;
                }
            }
            Mode::Target => {
                let v = self.things_at(x, y);
                let pick = v
                    .iter()
                    .copied()
                    .find(|&t| matches!(t, Tgt::Bldg(_)) && self.world.rewardable(t))
                    .or_else(|| v.iter().copied().find(|&t| self.world.rewardable(t)));
                if let Some(t) = pick {
                    if self.world.rewardable(t) {
                        self.sel = sel_of(t);
                        self.mode = Mode::Normal;
                        self.post(t);
                        return;
                    }
                }
                self.sfx |= sfx::NOPE;
            }
            _ => {
                self.sel = self.thing_at(x, y).map_or(Sel::None, sel_of);
                if self.sel != Sel::None {
                    self.sfx |= sfx::CLICK;
                }
                if self.mode == Mode::BuildMenu {
                    self.mode = Mode::Normal;
                }
            }
        }
    }

    fn place_tile(&self, x: i32, y: i32) -> (i32, i32) {
        if self.kb_on {
            self.kb
        } else {
            ((x - 8).div_euclid(TILE), (y - 8).div_euclid(TILE))
        }
    }

    /// The unit or building under a point: units first (their sprite), then footprints.
    /// Everything under a point: creatures nearest first (with a forgiving margin), then the
    /// building whose footprint holds it.
    fn things_at(&self, x: i32, y: i32) -> Vec<Tgt> {
        let mut units: Vec<&Unit> = self
            .world
            .units
            .iter()
            .filter(|u| !u.dead)
            .filter(|u| {
                let (w, h) = if u.kind == Kind::Heron {
                    (14, 20)
                } else {
                    (12, 14)
                };
                x >= u.x - w / 2 - 4 && x <= u.x + w / 2 + 4 && y >= u.y - h - 4 && y <= u.y + 4
            })
            .collect();
        units.sort_by_key(|u| (u.x - x).abs() + (u.y - 6 - y).abs());
        let mut v: Vec<Tgt> = units.iter().map(|u| Tgt::Unit(u.id)).collect();
        v.extend(
            self.world
                .bldgs
                .iter()
                .filter(|b| b.alive)
                .filter(|b| {
                    let (bx, by, bw, bh) = b.rect();
                    x >= bx && x < bx + bw && y >= by - 6 && y < by + bh
                })
                .map(|b| Tgt::Bldg(b.id)),
        );
        v
    }

    /// What a click picks: clicking the same spot again steps on to the next thing there.
    fn thing_at(&self, x: i32, y: i32) -> Option<Tgt> {
        let v = self.things_at(x, y);
        let cur = tgt_of(self.sel);
        match v.iter().position(|&t| Some(t) == cur) {
            Some(i) => Some(v[(i + 1) % v.len()]),
            None => v.first().copied(),
        }
    }

    /// Everything Tab steps through: heroes, then creatures, then enemy lairs, then your buildings.
    fn selectables(&self) -> Vec<Sel> {
        let w = &self.world;
        let mut v: Vec<Sel> = w
            .units
            .iter()
            .filter(|u| !u.dead && u.kind.hero())
            .map(|u| Sel::Unit(u.id))
            .collect();
        v.extend(
            w.units
                .iter()
                .filter(|u| !u.dead && !u.kind.hero())
                .map(|u| Sel::Unit(u.id)),
        );
        v.extend(
            w.bldgs
                .iter()
                .filter(|b| b.alive && b.kind.enemy())
                .map(|b| Sel::Bldg(b.id)),
        );
        v.extend(
            w.bldgs
                .iter()
                .filter(|b| b.alive && !b.kind.enemy())
                .map(|b| Sel::Bldg(b.id)),
        );
        v
    }

    fn cycle(&mut self, dir: i32) {
        let v = self.selectables();
        if v.is_empty() {
            return;
        }
        let at = v.iter().position(|&s| s == self.sel);
        let n = v.len() as i32;
        let next = match at {
            Some(i) => (i as i32 + dir).rem_euclid(n),
            None => {
                if dir > 0 {
                    0
                } else {
                    n - 1
                }
            }
        };
        let picked = v[next as usize];
        self.sfx |= sfx::CLICK;
        if self.mode == Mode::Target {
            if let Some(t) = tgt_of(picked) {
                if self.world.rewardable(t) {
                    self.sel = picked;
                    return;
                }
            }
        }
        self.sel = picked;
        self.kb = self.focus_tile();
    }

    fn post(&mut self, t: Tgt) {
        if !self.lesson_allows(t) {
            self.toast("THE LESSON ASKS FOR THE MARKED ONE FIRST.");
            self.sfx |= sfx::NOPE;
            return;
        }
        if self.world.flag_on(t).is_some() {
            return;
        }
        if self.world.flags.len() >= MAX_FLAGS {
            self.toast("FOUR REWARDS AT MOST.");
            self.sfx |= sfx::NOPE;
            return;
        }
        if self.world.post(t).is_some() {
            self.sfx |= sfx::POST;
        } else {
            self.toast(if self.world.reserved() > 0 {
                "A REWARD NEEDS 20 FREE DEW. OFFERS ARE SET ASIDE."
            } else {
                "A REWARD STARTS AT 20 DEW."
            });
            self.sfx |= sfx::NOPE;
        }
    }

    fn toast(&mut self, s: &str) {
        self.toast = Some((s.to_string(), 30));
    }

    fn selected_flag(&self) -> Option<u32> {
        tgt_of(self.sel)
            .and_then(|t| self.world.flag_on(t))
            .map(|f| f.id)
    }

    fn act(&mut self, a: Act) {
        match a {
            Act::Build => {
                self.mode = if self.mode == Mode::BuildMenu {
                    Mode::Normal
                } else {
                    Mode::BuildMenu
                };
                self.sfx |= sfx::CLICK;
            }
            Act::Place(kind) => {
                if self.world.dew - self.world.reserved() >= kind.cost() {
                    self.mode = Mode::Place(kind);
                    self.kb = (4, 7);
                    self.sfx |= sfx::CLICK;
                } else {
                    self.toast(self.short_of_dew());
                    self.sfx |= sfx::NOPE;
                }
            }
            Act::Hire => {
                let guild = match self.sel {
                    Sel::Bldg(b) => self
                        .world
                        .bldg(b)
                        .filter(|x| matches!(x.kind, BKind::Lodge | BKind::ToadHall))
                        .map(|x| x.id),
                    _ => None,
                };
                match guild {
                    None => {
                        self.toast("SELECT A LODGE OR THE TOAD HALL FIRST.");
                        self.sfx |= sfx::NOPE;
                    }
                    Some(b) => {
                        if self.world.hire(b).is_none() {
                            let msg = if self.world.can_hire(b).is_none() {
                                "NO ROOM FOR MORE HEROES THERE."
                            } else {
                                self.short_of_dew()
                            };
                            self.toast(msg);
                            self.sfx |= sfx::NOPE;
                        }
                    }
                }
            }
            Act::Reward => match tgt_of(self.sel) {
                Some(t) if self.world.rewardable(t) => self.post(t),
                _ => {
                    self.mode = Mode::Target;
                    self.sfx |= sfx::CLICK;
                }
            },
            Act::More => {
                if let Some(f) = self.selected_flag() {
                    if self.world.raise(f) {
                        self.sfx |= sfx::POST;
                    } else {
                        let msg = if self.world.flag(f).is_some_and(|x| x.reward >= 200) {
                            "200 DEW IS THE MOST."
                        } else {
                            "NOT ENOUGH FREE DEW."
                        };
                        self.toast(msg);
                        self.sfx |= sfx::NOPE;
                    }
                }
            }
            Act::Less => {
                if let Some(f) = self.selected_flag() {
                    if self.world.lower(f) {
                        self.sfx |= sfx::CLICK;
                    }
                }
            }
            Act::Cancel => {
                if let Some(f) = self.selected_flag() {
                    self.world.cancel(f);
                    self.sfx |= sfx::CLICK;
                }
            }
            Act::Pause => {
                self.paused = !self.paused;
                self.sfx |= sfx::CLICK;
            }
            Act::Feedback => {
                self.mode = Mode::Feedback;
                self.sfx |= sfx::CLICK;
            }
            Act::Keep => self.send("keep", ""),
            Act::Change => self.send("change", ""),
            Act::Drop => self.send("drop", ""),
            Act::Note => {
                self.mode = Mode::Note;
                self.note.clear();
            }
            Act::Back => self.back(),
            Act::Again => {
                let lesson = false;
                self.restart(lesson);
            }
        }
    }

    /// Queue one feedback entry (sent to the feedback file by the page).
    fn send(&mut self, kind: &str, text: &str) {
        let w = &self.world;
        let heroes = w.units.iter().filter(|u| u.kind.hero()).count();
        let sel = match self.sel {
            Sel::Unit(id) => w.unit(id).map_or(String::new(), |u| u.name.to_string()),
            Sel::Bldg(id) => w
                .bldg(id)
                .map_or(String::new(), |b| b.kind.name().to_string()),
            Sel::None => String::new(),
        };
        let state = format!(
            "{} dew, {} offered, {} nests left, {} heroes, {} fallen, lesson {:?}, selected {}",
            w.dew,
            w.reserved(),
            w.nests_left(),
            heroes,
            w.fallen,
            self.lesson,
            if sel.is_empty() { "nothing" } else { &sel }
        );
        let time = self.clock();
        self.outbox.push(format!(
            "{{\"kind\":\"{}\",\"text\":\"{}\",\"time\":\"{}\",\"state\":\"{}\"}}",
            kind,
            escape(text),
            time,
            escape(&state)
        ));
        self.toast(match kind {
            "note" => "NOTE SAVED. THANK YOU.",
            _ => "SAVED. THANK YOU.",
        });
        self.sfx |= sfx::PAID;
        self.mode = Mode::Normal;
    }

    fn clock(&self) -> String {
        let s = self.play_ticks / 15;
        format!("{}:{:02}", s / 60, s % 60)
    }

    // ---- the panel -----------------------------------------------------------------------------

    fn buttons(&self) -> Vec<Button> {
        let mut v = vec![];
        let row = |v: &mut Vec<Button>, y: i32, x0: i32, items: Vec<(String, Act, bool)>| {
            let mut x = x0;
            for (label, act, enabled) in items {
                let w = Fb::text_width(&label, 1) + 8;
                let key = label.chars().next().unwrap_or(' ').to_ascii_uppercase();
                v.push(Button {
                    label,
                    key,
                    x,
                    y,
                    w,
                    act,
                    enabled,
                });
                x += w + 3;
            }
        };
        if self.screen == Screen::End && self.mode == Mode::Note {
            return v;
        }
        if self.screen == Screen::End {
            row(
                &mut v,
                196,
                112,
                vec![
                    ("KEEP".into(), Act::Keep, true),
                    ("CHANGE".into(), Act::Change, true),
                    ("DROP".into(), Act::Drop, true),
                    ("NOTE".into(), Act::Note, true),
                    ("AGAIN".into(), Act::Again, true),
                ],
            );
            return v;
        }
        if self.screen != Screen::Play {
            return v;
        }
        if matches!(self.mode, Mode::Feedback | Mode::Note) {
            if self.mode == Mode::Feedback {
                row(
                    &mut v,
                    215,
                    100,
                    vec![
                        ("KEEP".into(), Act::Keep, true),
                        ("CHANGE".into(), Act::Change, true),
                        ("DROP".into(), Act::Drop, true),
                        ("NOTE".into(), Act::Note, true),
                        ("BACK".into(), Act::Back, true),
                    ],
                );
            }
            return v;
        }
        let w = &self.world;
        let free = w.dew - w.reserved();
        let guild = match self.sel {
            Sel::Bldg(b) => w.can_hire(b),
            _ => None,
        };
        row(
            &mut v,
            244,
            137,
            vec![
                ("BUILD".into(), Act::Build, true),
                ("HIRE".into(), Act::Hire, guild.is_some_and(|c| free >= c)),
                ("REWARD".into(), Act::Reward, w.flags.len() < MAX_FLAGS),
                (
                    if self.paused { "PLAY" } else { "PAUSE" }.into(),
                    Act::Pause,
                    true,
                ),
            ],
        );
        if v.last().is_some_and(|b| b.act == Act::Pause) {
            // "Play" and "Pause" both answer to P.
            v.last_mut().unwrap().key = 'P';
        }
        if matches!(self.mode, Mode::BuildMenu | Mode::Place(_)) {
            let opt = |k: BKind, label: &str| {
                (
                    format!("{label} {}", k.cost()),
                    Act::Place(k),
                    free >= k.cost(),
                )
            };
            row(
                &mut v,
                259,
                137,
                vec![
                    opt(BKind::Lodge, "LODGE"),
                    opt(BKind::ToadHall, "TOAD HALL"),
                ],
            );
            row(
                &mut v,
                274,
                137,
                vec![opt(BKind::Still, "STILL"), opt(BKind::Post, "WATCH POST")],
            );
        } else if let Some(f) = self.selected_flag().and_then(|f| w.flag(f)) {
            row(
                &mut v,
                259,
                137,
                vec![
                    (
                        format!("MORE +{REWARD_STEP}"),
                        Act::More,
                        free >= REWARD_STEP && f.reward < 200,
                    ),
                    (
                        format!("LESS -{REWARD_STEP}"),
                        Act::Less,
                        f.reward > REWARD_STEP,
                    ),
                    ("CANCEL".into(), Act::Cancel, true),
                ],
            );
        }
        row(
            &mut v,
            284,
            336,
            vec![("FEEDBACK".into(), Act::Feedback, true)],
        );
        v
    }

    fn button_at(&self, x: i32, y: i32) -> Option<Act> {
        self.buttons()
            .into_iter()
            .find(|b| x >= b.x && x < b.x + b.w && y >= b.y && y < b.y + BTN_H)
            .map(|b| b.act)
    }

    // ---- drawing -------------------------------------------------------------------------------

    fn draw(&mut self) {
        match self.screen {
            Screen::Title => self.draw_title(),
            _ => {
                self.draw_map();
                self.draw_panel();
                if self.screen == Screen::End {
                    self.draw_end();
                }
                if matches!(self.mode, Mode::Feedback | Mode::Note) {
                    self.draw_feedback();
                }
            }
        }
    }

    fn draw_title(&mut self) {
        let fb = &mut self.fb;
        fb.clear(NIGHT);
        for k in 0..60 {
            let h = crate::map::hash(k, 7);
            fb.pset(
                (h % 400) as i32,
                (h / 400 % 150) as i32,
                if k % 5 == 0 { STAR } else { TEXT_DIM },
            );
        }
        fb.rect(0, 210, 400, 90, TREE0);
        for k in 0..26 {
            let x = k * 16 + (k % 3) * 3;
            for r in 0..3 {
                let rr = 14 - r * 3;
                let col = [TREE0, TREE1, TREE2][r as usize];
                for yy in -rr..=rr {
                    for xx in -rr..=rr {
                        if xx * xx + yy * yy <= rr * rr {
                            fb.pset(x + xx - r, 206 + yy - r * 2, col);
                        }
                    }
                }
            }
        }
        let hall = &self.art.hall;
        fb.blit(hall, 200 - hall.w / 2, 232 - hall.h);
        let newt = &self.art.newt.right[0];
        fb.blit(newt, 160, 232 - newt.h);
        let toad = &self.art.toad.left[0];
        fb.blit(toad, 226, 232 - toad.h);
        let heron = &self.art.heron_fly.left[0];
        fb.blit(heron, 300, 120);
        fb.text_centered("NAIBIHPMA HOLD", FB_W / 2, 50, KEY, 3);
        fb.text_centered("THE ENCHANTED FOREST OF NAIBIHPMA", FB_W / 2, 90, TEXT, 1);
        fb.text_centered("MUSIC BY HENRY SOLBERG", FB_W / 2, 102, TEXT_DIM, 1);
        fb.text_centered("CLICK OR PRESS ENTER", FB_W / 2, 150, TEXT, 1);
        fb.text_centered("F2 SCANLINES", FB_W / 2, 280, TEXT_DIM, 1);
    }

    fn draw_map(&mut self) {
        if self.world.bldgs.iter().any(|b| b.kind == BKind::Glade) && !self.bg_curse {
            self.bg = self.world.map.paint(&self.art);
            self.bg_curse = true;
        }
        let (mark, _) = self.lesson_marks();
        let place = match self.mode {
            Mode::Place(k) => Some((k, self.place_tile(self.mouse.0, self.mouse.1))),
            _ => None,
        };
        let caption = if self.mode == Mode::Target {
            Some("CHOOSE A NEST OR A CREATURE FOR THE REWARD.".to_string())
        } else if let Mode::Place(k) = self.mode {
            Some(format!("WHERE SHOULD THE {} GO?", k.name()))
        } else {
            self.caption.as_ref().map(|c| c.0.clone())
        };
        let fb = &mut self.fb;
        fb.px[..(FB_W * PLAY_H) as usize].copy_from_slice(&self.bg);
        let w = &self.world;
        let art = &self.art;
        let tick = self.tick;
        let sel = self.sel;

        // Everything standing, back to front.
        enum D<'a> {
            B(&'a Bldg),
            U(&'a Unit),
            G(i32, i32),
        }
        let mut list: Vec<(i32, D)> = vec![];
        for b in w.bldgs.iter() {
            if b.alive || !b.kind.enemy() || b.kind == BKind::Nest {
                list.push((b.ty * TILE + 2 * TILE, D::B(b)));
            }
        }
        for u in w.units.iter().filter(|u| !u.dead) {
            let flying = u.kind == Kind::Heron && u.moving && !matches!(u.job, Job::Guard);
            list.push((if flying { 10_000 + u.y } else { u.y }, D::U(u)));
        }
        for &(x, y, _) in w.graves.iter() {
            list.push((y, D::G(x, y)));
        }
        list.sort_by_key(|e| e.0);
        for (_, d) in list.iter() {
            match d {
                D::B(b) => {
                    let (bx, by, _, bh) = b.rect();
                    let s = if !b.alive {
                        &art.rubble
                    } else {
                        match b.kind {
                            BKind::Hall => &art.hall,
                            BKind::Lodge => &art.lodge,
                            BKind::ToadHall => &art.toad_hall,
                            BKind::Still => &art.still,
                            BKind::Post => &art.post,
                            BKind::Nest => &art.nest,
                            BKind::Glade => &art.totem,
                        }
                    };
                    let (sx, sy) = (bx + TILE - s.w / 2, by + bh - s.h + 1);
                    if b.alive && b.built < 100 {
                        // Rising: the built share drawn from the ground up, the rest as a frame.
                        let rows = s.h * b.built / 100;
                        for yy in s.h - rows..s.h {
                            for xx in 0..s.w {
                                let c = s.get(xx, yy);
                                if c != 0 {
                                    fb.pset(sx + xx, sy + yy, c);
                                }
                            }
                        }
                        fb.frame_dotted(sx, sy, s.w, s.h - rows, WOOD2);
                    } else {
                        fb.blit(s, sx, sy);
                    }
                    if b.alive && b.hp < b.max_hp && b.built >= 100 {
                        bar(fb, bx + 4, sy - 3, 24, b.hp, b.max_hp);
                    }
                }
                D::U(u) => {
                    let frame = if u.moving {
                        ((tick / 3 + u.id) % 2) as usize
                    } else {
                        0
                    };
                    let s = match u.kind {
                        Kind::Newt => pick(&art.newt, u.left, frame),
                        Kind::Toad => pick(&art.toad, u.left, frame),
                        Kind::Heron => {
                            if u.moving && !matches!(u.job, Job::Guard) {
                                pick(&art.heron_fly, u.left, ((tick / 2 + u.id) % 2) as usize)
                            } else {
                                pick(&art.heron, u.left, 0)
                            }
                        }
                        Kind::Wisp => &art.wisp[((tick / 4 + u.id) % 2) as usize],
                    };
                    let (sx, sy) = (u.x - s.w / 2, u.y - s.h + 1);
                    if u.hurt_t > 0 {
                        blit_tint(fb, s, sx, sy, TEXT);
                    } else {
                        fb.blit(s, sx, sy);
                    }
                    let chosen = sel == Sel::Unit(u.id);
                    if u.hp < u.max_hp
                        && (u.hit_t > 0 || chosen || (u.kind.hero() && u.hp * 3 < u.max_hp))
                    {
                        bar(fb, u.x - 5, sy - 3, 10, u.hp, u.max_hp);
                    }
                    let icon = match u.bubble {
                        Bubble::None => None,
                        Bubble::Yes => Some(&art.icon_yes),
                        Bubble::No => Some(&art.icon_no),
                        Bubble::Dew => Some(&art.icon_dew),
                        Bubble::Heart => Some(&art.icon_heart),
                        Bubble::Home => Some(&art.icon_home),
                        Bubble::Zz => Some(&art.icon_zz),
                    };
                    if let Some(i) = icon {
                        fb.blit(i, u.x - i.w / 2, sy - 11);
                    }
                }
                D::G(x, y) => {
                    let g = &art.grave;
                    fb.blit(g, x - g.w / 2, y - g.h + 1);
                }
            }
        }

        // Reward flags with their amounts.
        for f in w.flags.iter() {
            let Some((x, y)) = flag_spot(w, f.tgt) else {
                continue;
            };
            let fl = &art.flag;
            fb.blit(fl, x, y - fl.h);
            let label = format!("{}", f.reward);
            fb.text_shadow(&label, x + fl.w + 1, y - fl.h, FLAG, 1);
        }

        // Toad bolts, watch-post spores, and blows landing.
        for &(x0, y0, x1, y1, _) in self.bolts.iter() {
            fb.line(x0, y0, x1, y1, DEW);
        }
        for &(x, y, _) in self.sparks.iter() {
            for (dx, dy) in [(0, 0), (-1, 0), (1, 0), (0, -1), (0, 1)] {
                fb.pset(x + dx, y + dy, TEXT);
            }
        }

        // Selection, lesson marks, the keyboard cursor and the placement ghost.
        if let Some(t) = tgt_of(self.sel) {
            if let Some((x, y, wd, ht)) = bounds(w, art, t) {
                brackets(fb, x - 2, y - 2, wd + 4, ht + 4, HILITE);
            }
        }
        if let Some(t) = mark {
            if let Some((x, y, wd, ht)) = bounds(w, art, t) {
                fb.frame(x - 4, y - 4, wd + 8, ht + 8, KEY);
                fb.frame(x - 5, y - 5, wd + 10, ht + 10, INK);
            }
        }
        if let Some((kind, (tx, ty))) = place {
            let ok = w.map.buildable(tx, ty) && w.dew - w.reserved() >= kind.cost();
            fb.checker(
                tx * TILE,
                ty * TILE,
                2 * TILE,
                2 * TILE,
                if ok { GHOST_OK } else { GHOST_NO },
            );
            fb.frame(
                tx * TILE,
                ty * TILE,
                2 * TILE,
                2 * TILE,
                if ok { GOOD } else { BAD },
            );
        } else if self.kb_on {
            brackets(fb, self.kb.0 * TILE, self.kb.1 * TILE, TILE, TILE, TEXT);
        }

        for f in self.floats.iter() {
            fb.text_centered(&f.text, f.x, f.y, f.col, 1);
        }

        // The caption band, the toast and the pause notice.
        if let Some(c) = caption {
            // Along the bottom of the marsh, over the tree line, where nothing happens.
            fb.rect(0, PLAY_H - 15, FB_W, 1, INK);
            fb.rect(0, PLAY_H - 14, FB_W, 14, SHADE);
            fb.text_centered(&c, FB_W / 2, PLAY_H - 11, TEXT, 1);
        }
        if let Some((s, _)) = &self.toast {
            let wd = Fb::text_width(s, 1) + 10;
            fb.bevel(FB_W / 2 - wd / 2, 200, wd, 15, PANEL, PANEL_HI, PANEL_LO);
            fb.text(s, FB_W / 2 - wd / 2 + 5, 204, TEXT, 1);
        }
        if self.paused && self.screen == Screen::Play {
            fb.bevel(FB_W / 2 - 34, 108, 68, 17, PANEL, PANEL_HI, PANEL_LO);
            fb.text_centered("PAUSED", FB_W / 2, 113, KEY, 1);
        }
    }

    fn draw_panel(&mut self) {
        let fb = &mut self.fb;
        fb.rect(0, PLAY_H, FB_W, 300 - PLAY_H, PANEL);
        fb.rect(0, PLAY_H, FB_W, 1, PANEL_HI);
        fb.rect(0, PLAY_H + 1, FB_W, 1, INK);
        // Info box.
        fb.rect(2, 243, 131, 55, PANEL_LO);
        fb.frame(2, 243, 131, 55, PANEL_HI);
        // Right box.
        fb.rect(334, 243, 64, 40, PANEL_LO);
        fb.frame(334, 243, 64, 40, PANEL_HI);
        let w = &self.world;
        let art = &self.art;
        fb.blit(&art.icon_dew, 338, 246);
        fb.text(&format!("FREE {}", w.dew - w.reserved()), 347, 247, TEXT, 1);
        if w.reserved() > 0 {
            fb.text(&format!("OFFER {}", w.reserved()), 347, 256, FLAG, 1);
        }
        fb.blit(&art.icon_nest, 338, 265);
        if w.glade_up() {
            fb.text(&format!("{}", w.nests_left()), 347, 266, TEXT, 1);
            fb.blit(&art.icon_stone, 357, 264);
            fb.text("STONE", 366, 266, WISP2, 1);
        } else {
            fb.text(&format!("NESTS {}", w.nests_left()), 347, 266, TEXT, 1);
        }
        let time = if self.lesson != Lesson::Done && self.lesson != Lesson::Fight {
            "LESSON".to_string()
        } else {
            format!("{}:{:02}", self.play_ticks / 900, self.play_ticks / 15 % 60)
        };
        fb.text(&time, 347, 275, TEXT_DIM, 1);

        // What's selected.
        let lines: Vec<(String, u8)> = match self.sel {
            Sel::Unit(id) => match w.unit(id) {
                Some(u) => {
                    let s = match u.kind {
                        Kind::Newt => pick(&art.newt, false, 0),
                        Kind::Toad => pick(&art.toad, false, 0),
                        Kind::Heron => pick(&art.heron, false, 0),
                        Kind::Wisp => &art.wisp[0],
                    };
                    if !matches!(self.mode, Mode::BuildMenu | Mode::Place(_)) {
                        fb.blit(s, 6, 262 - s.h);
                        bar(fb, 26, 265, 100, u.hp, u.max_hp);
                    }
                    let mut v = vec![(u.name.to_string(), TEXT)];
                    if u.kind.hero() {
                        let kind = if u.kind == Kind::Newt { "NEWT" } else { "TOAD" };
                        v.push((
                            format!("{} {} L{}", u.trait_.name(), kind, u.level),
                            TEXT_DIM,
                        ));
                        v.push((
                            format!("HP {}/{}  PURSE {}", u.hp, u.max_hp, u.purse),
                            TEXT_DIM,
                        ));
                    } else {
                        v.push((format!("HP {}/{}", u.hp, u.max_hp), TEXT_DIM));
                        if let Some(f) = w.flag_on(Tgt::Unit(id)) {
                            v.push((format!("REWARD {} DEW", f.reward), FLAG));
                        }
                    }
                    for l in wrap(&w.status(u), 21) {
                        v.push((l, TEXT));
                    }
                    v
                }
                None => vec![],
            },
            Sel::Bldg(id) => match w.bldg(id) {
                Some(b) => {
                    let mut v = vec![
                        (b.kind.name().to_string(), TEXT),
                        (format!("HP {}/{}", b.hp.max(0), b.max_hp), TEXT_DIM),
                    ];
                    if b.built < 100 {
                        v.push((format!("BUILDING {}%", b.built), WARN));
                    }
                    match b.kind {
                        BKind::Lodge | BKind::ToadHall => {
                            let n = w
                                .units
                                .iter()
                                .filter(|u| !u.dead && u.home == b.id && u.kind.hero())
                                .count();
                            let cost = if b.kind == BKind::Lodge {
                                HIRE_NEWT
                            } else {
                                HIRE_TOAD
                            };
                            let who = if b.kind == BKind::Lodge {
                                "NEWTS"
                            } else {
                                "TOADS"
                            };
                            v.push((format!("{who} {n}/3, HIRE {cost}"), TEXT_DIM));
                        }
                        BKind::Still => v.push(("MAKES DEW".into(), TEXT_DIM)),
                        BKind::Hall => v.push(("MAKES DEW. KEEP IT!".into(), TEXT_DIM)),
                        BKind::Post => v.push(("SHOOTS AT RAIDERS".into(), TEXT_DIM)),
                        BKind::Nest | BKind::Glade => {
                            let guards =
                                w.units.iter().filter(|u| !u.dead && u.home == b.id).count();
                            if b.kind == BKind::Glade {
                                v.push((format!("WISPS {guards}, DRAIN STILLS"), TEXT_DIM));
                            } else {
                                v.push((format!("GUARDS {guards}"), TEXT_DIM));
                            }
                            match w.flag_on(Tgt::Bldg(id)) {
                                Some(f) => {
                                    v.push((format!("REWARD {} DEW", f.reward), FLAG));
                                    let names: Vec<&str> = f
                                        .takers
                                        .iter()
                                        .filter_map(|&t| w.unit(t))
                                        .map(|u| u.name)
                                        .collect();
                                    let s = if names.is_empty() {
                                        "NO TAKERS YET".to_string()
                                    } else {
                                        let shown: Vec<&str> =
                                            names.iter().take(3).copied().collect();
                                        let more = names.len().saturating_sub(3);
                                        if more > 0 {
                                            format!("TAKEN BY {} +{more}", shown.join(", "))
                                        } else {
                                            format!("TAKEN BY {}", shown.join(", "))
                                        }
                                    };
                                    for l in wrap(&s, 20) {
                                        v.push((l, TEXT));
                                    }
                                }
                                None => v.push(("NO REWARD ON IT".into(), TEXT_DIM)),
                            }
                        }
                    }
                    v
                }
                None => vec![],
            },
            Sel::None => vec![
                ("NOTHING SELECTED".into(), TEXT_DIM),
                ("CLICK SOMETHING,".into(), TEXT_DIM),
                ("OR PRESS TAB.".into(), TEXT_DIM),
            ],
        };
        let lines = if matches!(self.mode, Mode::BuildMenu | Mode::Place(_)) {
            vec![
                ("BUILD WHAT?".to_string(), TEXT),
                ("LODGE: HIRES NEWTS".to_string(), TEXT_DIM),
                ("TOAD HALL: TOADS".to_string(), TEXT_DIM),
                ("STILL: MAKES DEW".to_string(), TEXT_DIM),
                ("POST: SHOOTS RAIDERS".to_string(), TEXT_DIM),
                ("ON THE HOLD'S BANK".to_string(), TEXT_DIM),
            ]
        } else {
            lines
        };
        let unit = matches!(self.sel, Sel::Unit(_))
            && !matches!(self.mode, Mode::BuildMenu | Mode::Place(_));
        for (k, (s, c)) in lines.iter().take(6).enumerate() {
            // A unit's name and kind sit beside its portrait, above its health bar.
            let (x, y) = if unit {
                if k < 2 {
                    (26, 247 + k as i32 * 8)
                } else {
                    (6, 270 + (k as i32 - 2) * 8)
                }
            } else {
                (6, 247 + k as i32 * 8)
            };
            if y > 290 {
                break;
            }
            let s: String = s.chars().take(((132 - x) / 6) as usize).collect();
            fb.text(&s, x, y, *c, 1);
        }

        // Buttons.
        let (_, mark) = self.lesson_marks();
        for b in self.buttons() {
            draw_button(&mut self.fb, &b, mark == Some(b.act), self.mode);
        }
    }

    fn draw_feedback(&mut self) {
        let fb = &mut self.fb;
        fb.bevel(90, 188, 220, 46, PANEL, PANEL_HI, PANEL_LO);
        if self.mode == Mode::Note {
            fb.text("YOUR NOTE (ENTER SAVES, ESC BACK)", 96, 193, TEXT_DIM, 1);
            fb.rect(96, 205, 208, 22, PANEL_LO);
            let shown: String = {
                let n = self.note.chars().count();
                self.note.chars().skip(n.saturating_sub(33)).collect()
            };
            fb.text(&shown, 99, 209, TEXT, 1);
            let cx = 99 + Fb::text_width(&shown, 1) + 2;
            fb.rect(
                cx,
                209,
                5,
                7,
                if self.tick / 8 % 2 == 0 {
                    KEY
                } else {
                    PANEL_LO
                },
            );
            if self.screen == Screen::Play {
                fb.text("THE GAME IS PAUSED", 99, 218, TEXT_DIM, 1);
            }
        } else {
            fb.text("WHAT DO YOU THINK SO FAR?", 96, 193, TEXT, 1);
            fb.text("(THE GAME IS PAUSED)", 96, 202, TEXT_DIM, 1);
        }
        for b in self.buttons() {
            draw_button(&mut self.fb, &b, false, self.mode);
        }
    }

    fn draw_end(&mut self) {
        let won = self.world.over == Some(true);
        let fb = &mut self.fb;
        fb.checker(0, 0, FB_W, PLAY_H, SHADE);
        fb.bevel(70, 56, 260, 160, PANEL, PANEL_HI, PANEL_LO);
        fb.text_centered(
            if won {
                "THE MARSH IS SAFE TONIGHT"
            } else {
                "THE HOLD HAS FALLEN"
            },
            FB_W / 2,
            68,
            KEY,
            1,
        );
        let w = &self.world;
        let s = self.play_ticks / 15;
        let rows = [
            format!("TIME {}:{:02}", s / 60, s % 60),
            format!("REWARDS PAID {} DEW", w.paid_total),
            format!("HEROES LOST {}", w.fallen),
        ];
        for (k, r) in rows.iter().enumerate() {
            fb.text_centered(r, FB_W / 2, 92 + k as i32 * 12, TEXT, 1);
        }
        fb.text_centered("WHAT DID YOU THINK?", FB_W / 2, 140, TEXT, 1);
        fb.text_centered(
            "KEEP, CHANGE OR DROP THIS IDEA,",
            FB_W / 2,
            154,
            TEXT_DIM,
            1,
        );
        fb.text_centered(
            "OR LEAVE A NOTE. AGAIN PLAYS ANOTHER.",
            FB_W / 2,
            164,
            TEXT_DIM,
            1,
        );
        if let Some((s, _)) = &self.toast {
            fb.text_centered(s, FB_W / 2, 180, GOOD, 1);
        }
        let mode = self.mode;
        for b in self.buttons() {
            draw_button(&mut self.fb, &b, false, mode);
        }
    }

    // ---- presentation --------------------------------------------------------------------------

    pub fn present(&mut self, n: usize) -> &[u8] {
        let pal = palette(self.tick);
        self.fb.present(&mut self.out, n, self.scanlines, &pal);
        &self.out
    }

    /// The pointer as RGBA at scale `n`: each logical pixel an exact n x n block, index 0 fully
    /// transparent, everything else fully opaque. Returns (pixels, w, h).
    pub fn cursor(&mut self, n: usize) -> (&[u8], usize, usize) {
        let s = &self.art.pointer;
        let (w, h) = (s.w as usize * n, s.h as usize * n);
        self.cursor_px.clear();
        self.cursor_px.resize(w * h * 4, 0);
        for y in 0..h {
            for x in 0..w {
                let c = s.px[(y / n) * s.w as usize + x / n];
                if c != 0 {
                    let v = rgb(c);
                    self.cursor_px[(y * w + x) * 4..(y * w + x) * 4 + 4].copy_from_slice(&[
                        (v >> 16) as u8,
                        (v >> 8) as u8,
                        v as u8,
                        255,
                    ]);
                }
            }
        }
        (&self.cursor_px, w, h)
    }

    /// Is the game waiting for typed text (so the page should pass every key through)?
    pub fn typing(&self) -> bool {
        self.mode == Mode::Note
    }
}

// ---- helpers ------------------------------------------------------------------------------------

fn sel_of(t: Tgt) -> Sel {
    match t {
        Tgt::Unit(id) => Sel::Unit(id),
        Tgt::Bldg(id) => Sel::Bldg(id),
    }
}

fn tgt_of(s: Sel) -> Option<Tgt> {
    match s {
        Sel::Unit(id) => Some(Tgt::Unit(id)),
        Sel::Bldg(id) => Some(Tgt::Bldg(id)),
        Sel::None => None,
    }
}

fn pick(w: &Walk, left: bool, frame: usize) -> &Sprite {
    if left {
        &w.left[frame]
    } else {
        &w.right[frame]
    }
}

/// On-screen bounds of a target: x, y, w, h.
fn bounds(w: &World, art: &Art, t: Tgt) -> Option<(i32, i32, i32, i32)> {
    match t {
        Tgt::Unit(id) => w.unit(id).map(|u| {
            let s = match u.kind {
                Kind::Newt => &art.newt.right[0],
                Kind::Toad => &art.toad.right[0],
                Kind::Heron => &art.heron.right[0],
                Kind::Wisp => &art.wisp[0],
            };
            (u.x - s.w / 2, u.y - s.h + 1, s.w, s.h)
        }),
        Tgt::Bldg(id) => w.bldg(id).map(|b| {
            let (bx, by, bw, bh) = b.rect();
            (bx, by - 2, bw, bh + 2)
        }),
    }
}

/// Where a reward flag stands: the top-right of the target.
fn flag_spot(w: &World, t: Tgt) -> Option<(i32, i32)> {
    match t {
        Tgt::Unit(id) => w.unit(id).map(|u| (u.x + 6, u.y - 14)),
        Tgt::Bldg(id) => w.bldg(id).map(|b| {
            let (bx, by, _, _) = b.rect();
            (bx + 22, by + 8)
        }),
    }
}

fn bar(fb: &mut Fb, x: i32, y: i32, w: i32, hp: i32, max: i32) {
    let fill = (w * hp.max(0) / max.max(1)).clamp(0, w);
    fb.rect(x - 1, y - 1, w + 2, 3, INK);
    fb.rect(x, y, w, 1, PANEL_LO);
    fb.rect(x, y, fill, 1, if hp * 3 < max { HP_LOW } else { HP_OK });
}

fn brackets(fb: &mut Fb, x: i32, y: i32, w: i32, h: i32, c: u8) {
    let l = 4;
    for (cx, cy, dx, dy) in [
        (x, y, 1, 1),
        (x + w - 1, y, -1, 1),
        (x, y + h - 1, 1, -1),
        (x + w - 1, y + h - 1, -1, -1),
    ] {
        for k in 0..l {
            fb.pset(cx + k * dx, cy, c);
            fb.pset(cx, cy + k * dy, c);
        }
    }
}

fn blit_tint(fb: &mut Fb, s: &Sprite, x: i32, y: i32, c: u8) {
    for sy in 0..s.h {
        for sx in 0..s.w {
            let p = s.get(sx, sy);
            if p != 0 {
                fb.pset(x + sx, y + sy, if p == INK { INK } else { c });
            }
        }
    }
}

fn draw_button(fb: &mut Fb, b: &Button, marked: bool, mode: Mode) {
    let active = matches!(
        (b.act, mode),
        (Act::Build, Mode::BuildMenu) | (Act::Build, Mode::Place(_)) | (Act::Reward, Mode::Target)
    ) || matches!((b.act, mode), (Act::Place(k), Mode::Place(m)) if k == m);
    let fill = if active { BTN_ON } else { BTN };
    fb.bevel(b.x, b.y, b.w, BTN_H, fill, BTN_HI, BTN_LO);
    let mut chars = b.label.chars();
    let first: String = chars.next().map(|c| c.to_string()).unwrap_or_default();
    let rest: String = chars.collect();
    let (kc, tc) = if b.enabled {
        (KEY, TEXT)
    } else {
        (TEXT_DIM, TEXT_DIM)
    };
    fb.text(&first, b.x + 4, b.y + 3, kc, 1);
    fb.text(&rest, b.x + 10, b.y + 3, tc, 1);
    if marked {
        fb.frame(b.x - 2, b.y - 2, b.w + 4, BTN_H + 4, KEY);
    }
}

fn wrap(s: &str, width: usize) -> Vec<String> {
    let mut out = vec![];
    let mut line = String::new();
    for word in s.split(' ') {
        if !line.is_empty() && line.len() + 1 + word.len() > width {
            out.push(std::mem::take(&mut line));
        }
        if !line.is_empty() {
            line.push(' ');
        }
        line.push_str(word);
    }
    if !line.is_empty() {
        out.push(line);
    }
    out
}

fn escape(s: &str) -> String {
    s.chars()
        .flat_map(|c| match c {
            '"' => vec!['\\', '"'],
            '\\' => vec!['\\', '\\'],
            c if (c as u32) < 0x20 => vec![' '],
            c => vec![c],
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn frames(g: &mut Game, n: u32) {
        for _ in 0..n {
            g.frame();
        }
    }

    /// Click the centre of a target.
    fn click(g: &mut Game, t: Tgt) {
        let (x, y, w, h) = bounds(&g.world, &g.art, t).unwrap();
        g.push(In::Down(0, x + w / 2, y + h / 2));
        g.frame();
    }

    fn press(g: &mut Game, c: char) {
        g.push(In::Char(c));
        g.frame();
    }

    #[test]
    fn the_lesson_can_be_played_with_the_mouse() {
        let mut g = Game::new(1);
        g.push(In::Down(0, 200, 150));
        frames(&mut g, 100);
        assert_eq!(g.lesson, Lesson::FirstReward);
        let n1 = g.weak_nest().unwrap();
        click(&mut g, Tgt::Bldg(n1));
        let (bx, by) = g
            .buttons()
            .iter()
            .find(|b| b.act == Act::Reward)
            .map(|b| (b.x + 3, b.y + 3))
            .unwrap();
        g.push(In::Down(0, bx, by));
        frames(&mut g, 2);
        assert_eq!(g.lesson, Lesson::Watch);
        frames(&mut g, 900);
        assert_eq!(g.lesson, Lesson::Hire);
        let th = g.toad_hall().unwrap();
        click(&mut g, Tgt::Bldg(th));
        press(&mut g, 'h');
        frames(&mut g, 2);
        assert_eq!(g.lesson, Lesson::SecondReward);
        let n2 = g.second_nest().unwrap();
        click(&mut g, Tgt::Bldg(n2));
        press(&mut g, 'r');
        frames(&mut g, 60);
        assert_eq!(g.lesson, Lesson::OfferMore);
        for _ in 0..4 {
            press(&mut g, 'm');
            frames(&mut g, 30);
        }
        frames(&mut g, 100);
        assert!(
            matches!(g.lesson, Lesson::Fight | Lesson::Done),
            "lesson at {:?}",
            g.lesson
        );
        assert!(!g.world.calm);
    }

    #[test]
    fn tab_and_letters_can_play_without_a_mouse() {
        let mut g = Game::new(2);
        g.push(In::Key(K_ENTER, false));
        frames(&mut g, 100);
        // Tab to the weak nest, then R.
        for _ in 0..20 {
            g.push(In::Key(K_TAB, false));
            g.frame();
            if tgt_of(g.sel) == g.weak_nest().map(Tgt::Bldg) {
                break;
            }
        }
        assert_eq!(tgt_of(g.sel), g.weak_nest().map(Tgt::Bldg));
        press(&mut g, 'r');
        frames(&mut g, 2);
        assert_eq!(g.lesson, Lesson::Watch);
    }

    #[test]
    fn feedback_is_queued_as_json() {
        let mut g = Game::new(3);
        g.push(In::Key(K_ENTER, false));
        frames(&mut g, 5);
        press(&mut g, 'f');
        press(&mut g, 'n');
        for c in "too \"slow\"".chars() {
            g.push(In::Char(c));
        }
        g.push(In::Key(K_ENTER, false));
        g.frame();
        assert_eq!(g.outbox.len(), 1);
        assert!(g.outbox[0].contains("\"kind\":\"note\""));
        assert!(g.outbox[0].contains("too \\\"slow\\\""));
        press(&mut g, 'f');
        press(&mut g, 'k');
        assert!(g.outbox[1].contains("\"kind\":\"keep\""));
    }

    #[test]
    fn building_with_the_keyboard_cursor() {
        let mut g = Game::new(4);
        g.restart(false);
        g.world.dew = 200;
        press(&mut g, 'b');
        press(&mut g, 's');
        g.push(In::Key(K_RIGHT, false));
        g.frame();
        g.push(In::Key(K_ENTER, false));
        g.frame();
        assert_eq!(
            g.world
                .bldgs
                .iter()
                .filter(|b| b.kind == BKind::Still)
                .count(),
            2
        );
    }
}
