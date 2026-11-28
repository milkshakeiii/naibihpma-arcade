//! The game around the rules (document 04): the title, the play screen and its panel, the stock
//! window, building, rewards, the Ancient Magic, the information screens, the guided first
//! minute and hints (document 05), feedback and the end card. Everything is drawn into the
//! 400 x 300 indexed framebuffer at 15 FPS; input arrives latched, in order.
//!
//! Majesty-style mouse controls, and every button also has a key: its yellow letter. Tab steps
//! through what's on screen, arrows scroll (or move a map cursor while placing), Enter picks,
//! Esc or right-click backs out.

use crate::art::*;
use crate::data::*;
use crate::fb::{Fb, FB_W};
use crate::terrain::{dist, idx, on_map, Tile, MH, MW, SQ};
use crate::world::elder::Answer;
use crate::world::marsh::hint;
use crate::world::{clock, day_tick, is_night, Ev, Id, RKind, RTarget, Task, World};

pub const K_TAB: u8 = 1;
pub const K_ENTER: u8 = 2;
pub const K_ESC: u8 = 3;
pub const K_BACK: u8 = 4;
pub const K_UP: u8 = 5;
pub const K_DOWN: u8 = 6;
pub const K_LEFT: u8 = 7;
pub const K_RIGHT: u8 = 8;
pub const K_F2: u8 = 9;
pub const K_F3: u8 = 10;
pub const K_F4: u8 = 11;
pub const K_F5: u8 = 12;
pub const K_F6: u8 = 13;
pub const K_F7: u8 = 14;
pub const K_F8: u8 = 15;
pub const K_F9: u8 = 16;
pub const K_HOME: u8 = 17;
/// W, A, S, D held down (18 to 21) and let go (22 to 25): they scroll the view.
pub const K_HOLD_W: u8 = 18;
pub const K_LET_W: u8 = 22;

pub const VIEW_H: i32 = 240;

/// Sound effects raised this frame, as bits (the page plays them).
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
    C(Id),
    B(Id),
    Flag(Id),
    Husk(Id),
    Ground(usize),
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Info {
    Heroes,
    Folk,
    Rewards,
    Log,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Mode {
    Normal,
    BuildMenu(Id),
    Place(Id, BKind),
    Route(Id),
    Puddle,
    Explore,
    Spells,
    Aim(Spell),
    Stock(Id),
    Info(Info),
    Menu,
    Feedback,
    Note,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Act {
    Reward,
    More,
    Less,
    Withdraw,
    Build,
    Choose(BKind),
    Hatch,
    Stock,
    Route,
    Unroute,
    Puddle,
    Explore,
    CancelSite,
    Pause,
    Fast,
    Magic,
    Screens,
    Spell(Spell),
    Plus,
    Minus,
    Defaults,
    Keep,
    Change,
    Drop,
    Note,
    Back,
    Resume,
    Restart,
    Hints,
    Again,
    Start,
    FlagOf,
}

#[derive(Clone, Debug)]
struct Button {
    label: String,
    key: char,
    x: i32,
    y: i32,
    w: i32,
    h: i32,
    act: Act,
    enabled: bool,
    icon: Option<UiLook>,
}

/// The guided first minute (document 05).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Lesson {
    Elder,
    Offer,
    Weigh,
    Raise,
    Cave,
    Wait,
    Done,
}

pub struct Game {
    pub world: World,
    pub fb: Fb,
    art: Art,
    out: Vec<u8>,
    cursor_px: Vec<u8>,
    pub screen: Screen,
    inputs: Vec<In>,
    pub sfx: u32,
    pub outbox: Vec<String>,
    pub mouse: (i32, i32),
    pub cam: (i32, i32),
    pub sel: Sel,
    pub mode: Mode,
    pub paused: bool,
    fast: bool,
    scanlines: bool,
    pub hints_on: bool,
    pub lesson: Lesson,
    lesson_t: u32,
    line: Option<(String, u32)>,
    buttons: Vec<Button>,
    stock_line: usize,
    note: String,
    cursor_sq: Option<(i32, i32)>,
    tick: u32,
    seed: u64,
    message: Option<(String, u32)>,
    cam_goal: Option<(i32, i32)>,
    info_scroll: usize,
    pub frames: u32,
    shots: Vec<((i32, i32), (i32, i32), u8, u32)>,
    dew_hint: bool,
    /// W, A, S, D held.
    held: [bool; 4],
}

fn text_w(s: &str) -> i32 {
    Fb::text_width(s, 1)
}

impl Game {
    pub fn new(seed: u64) -> Game {
        let mut g = Game {
            world: World::new(seed),
            fb: Fb::default(),
            art: Art::new(),
            out: Vec::new(),
            cursor_px: Vec::new(),
            screen: Screen::Title,
            inputs: Vec::new(),
            sfx: 0,
            outbox: Vec::new(),
            mouse: (200, 150),
            cam: (0, 0),
            sel: Sel::None,
            mode: Mode::Normal,
            paused: false,
            fast: false,
            scanlines: true,
            hints_on: true,
            lesson: Lesson::Elder,
            lesson_t: 0,
            line: None,
            buttons: Vec::new(),
            stock_line: 0,
            note: String::new(),
            cursor_sq: None,
            tick: 0,
            seed,
            message: None,
            cam_goal: None,
            info_scroll: 0,
            frames: 0,
            shots: Vec::new(),
            dew_hint: false,
            held: [false; 4],
        };
        g.centre_on_hall();
        g.draw();
        g
    }

    fn restart(&mut self, lesson: bool) {
        self.seed = self.seed.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
        self.world = World::new(self.seed);
        self.sel = Sel::None;
        self.mode = Mode::Normal;
        self.paused = false;
        self.fast = false;
        self.line = None;
        self.message = None;
        self.lesson = if lesson { Lesson::Elder } else { Lesson::Done };
        self.world.lesson = lesson;
        self.lesson_t = 0;
        self.screen = Screen::Play;
        self.centre_on_hall();
    }

    pub fn start(&mut self, lesson: bool) {
        self.screen = Screen::Play;
        self.lesson = if lesson { Lesson::Elder } else { Lesson::Done };
        self.world.lesson = lesson;
        self.lesson_t = 0;
        self.centre_on_hall();
    }

    pub fn push(&mut self, i: In) {
        self.inputs.push(i);
    }

    pub fn pause(&mut self) {
        if self.screen == Screen::Play {
            self.paused = true;
        }
    }

    pub fn wants_music(&self) -> bool {
        self.screen != Screen::Title || self.frames > 0
    }

    pub fn typing(&self) -> bool {
        self.mode == Mode::Note
    }

    fn centre_on_hall(&mut self) {
        if let Some(h) = self.world.hall() {
            let c = self.world.b(h).centre();
            self.centre_on(c);
        }
    }

    fn centre_on(&mut self, sq: (i32, i32)) {
        self.cam = (sq.0 * SQ + 8 - FB_W / 2, sq.1 * SQ + 8 - VIEW_H / 2);
        self.clamp_cam();
    }

    fn clamp_cam(&mut self) {
        self.cam.0 = self.cam.0.clamp(0, MW * SQ - FB_W);
        self.cam.1 = self.cam.1.clamp(0, MH * SQ - VIEW_H);
    }

    fn say(&mut self, s: impl Into<String>) {
        self.line = Some((s.into(), self.tick + 75));
    }

    fn nope(&mut self, s: impl Into<String>) {
        self.message = Some((s.into(), self.tick + 45));
        self.sfx |= sfx::NOPE;
    }

    // ---- the frame -----------------------------------------------------------------------------

    pub fn frame(&mut self) {
        self.frames += 1;
        self.tick += 1;
        let inputs = std::mem::take(&mut self.inputs);
        for i in inputs {
            self.input(i);
        }
        if self.screen == Screen::Play {
            self.scroll();
            let steps = if self.paused || matches!(self.mode, Mode::Menu | Mode::Note | Mode::Feedback) {
                0
            } else if self.fast {
                2
            } else {
                1
            };
            for _ in 0..steps {
                self.world.tick();
                self.take_events();
            }
            self.lesson_step();
            if self.lesson == Lesson::Done && self.hints_on && !self.dew_hint {
                if let Some(h) = self.world.hall() {
                    if self.world.b(h).has(Item::Dew) == 0 && self.world.t > 2 * MINUTE {
                        self.dew_hint = true;
                        self.say("THE HALL'S DEW GOES TO BUILDING AND HATCHING. HATCH:N SAVES IT.");
                    }
                }
            }
            if let Some(won) = self.world.over {
                if self.screen == Screen::Play && self.world.t > 0 {
                    self.screen = Screen::End;
                    self.sfx |= if won { sfx::WON } else { sfx::FELL };
                }
            }
            self.validate_sel();
        }
        self.draw();
    }

    fn take_events(&mut self) {
        let evs = std::mem::take(&mut self.world.events);
        for e in evs {
            match e {
                Ev::Say(s) => {
                    if self.lesson == Lesson::Done || !s.starts_with("A SITE") {
                        self.line = Some((s, self.tick + 75));
                    }
                }
                Ev::Hit => self.sfx |= sfx::HIT,
                Ev::Bolt => self.sfx |= sfx::BOLT,
                Ev::Died(_, hold) => self.sfx |= if hold { sfx::FELL } else { sfx::DOWN },
                Ev::Paid(_) => self.sfx |= sfx::PAID,
                Ev::Built(_) => self.sfx |= sfx::BUILT,
                Ev::Hatched(_) => self.sfx |= sfx::HIRE,
                Ev::Raid(_) => self.sfx |= sfx::RAID,
                Ev::Took(..) => self.sfx |= sfx::ACCEPT,
                Ev::Refused => self.sfx |= sfx::REFUSE,
                Ev::Won | Ev::Lost => {}
                Ev::Shot(a, b, k) => {
                    if self.shots.len() < 60 {
                        self.shots.push((a, b, k, self.tick));
                    }
                }
                Ev::Hint(h) => {
                    if self.hints_on && self.lesson == Lesson::Done {
                        let s = match h {
                            hint::NOON => Some("NOON. DEW ON OPEN FERNS DRIES."),
                            hint::WATER => Some("TOO DRY. AMPHIBIANS NEED WATER."),
                            hint::RAID => Some("HERONS WILL COME FROM THE EAST."),
                            hint::SNAKE => Some("A SNAKE. NEWT POISON WILL NOT STOP IT."),
                            hint::SUN_ORB => Some("A SUN ORB. THE STAR POND CAN BE RAISED."),
                            hint::STONE => Some("GORF'S STONE. ITS CURSE WILL SPREAD."),
                            _ => None,
                        };
                        if let Some(s) = s {
                            self.line = Some((s.into(), self.tick + 90));
                        }
                    }
                }
            }
        }
    }

    fn validate_sel(&mut self) {
        let w = &self.world;
        let ok = match self.sel {
            Sel::None => true,
            Sel::C(c) => w.ci(c).is_some_and(|k| !w.cs[k].dead),
            Sel::B(b) => w.bi(b).is_some_and(|k| !w.bs[k].dead),
            Sel::Flag(f) => w.flag(f).is_some(),
            Sel::Husk(h) => w.husks.iter().any(|x| x.id == h && !x.gone),
            Sel::Ground(g) => w.ground.get(g).is_some_and(|x| !x.gone),
        };
        if !ok {
            self.sel = Sel::None;
            if matches!(self.mode, Mode::Stock(_) | Mode::BuildMenu(_) | Mode::Route(_)) {
                self.mode = Mode::Normal;
            }
        }
    }

    fn scroll(&mut self) {
        if let Some(goal) = self.cam_goal {
            let dx = (goal.0 - self.cam.0).clamp(-12, 12);
            let dy = (goal.1 - self.cam.1).clamp(-12, 12);
            self.cam.0 += dx;
            self.cam.1 += dy;
            self.clamp_cam();
            if dx == 0 && dy == 0 || (self.cam.0 - goal.0).abs() + (self.cam.1 - goal.1).abs() < 3 {
                self.cam_goal = None;
            }
            return;
        }
        if matches!(self.mode, Mode::Info(_) | Mode::Menu | Mode::Note | Mode::Feedback) {
            return;
        }
        // W, A, S, D held.
        let (dx, dy) = ((self.held[3] as i32) - (self.held[1] as i32), (self.held[2] as i32) - (self.held[0] as i32));
        if dx != 0 || dy != 0 {
            self.cam.0 += dx * 8;
            self.cam.1 += dy * 8;
            self.clamp_cam();
            self.cam_goal = None;
        }
        let (mx, my) = self.mouse;
        if mx < 0 || my < 0 || mx >= FB_W || my >= VIEW_H {
            return;
        }
        let edge = 4;
        if mx < edge {
            self.cam.0 -= 8;
        }
        if mx >= FB_W - edge {
            self.cam.0 += 8;
        }
        if my < edge {
            self.cam.1 -= 8;
        }
        if my >= VIEW_H - edge {
            self.cam.1 += 8;
        }
        self.clamp_cam();
    }

    fn pan_to(&mut self, sq: (i32, i32)) {
        let goal = (sq.0 * SQ + 8 - FB_W / 2, sq.1 * SQ + 8 - VIEW_H / 2);
        self.cam_goal = Some((goal.0.clamp(0, MW * SQ - FB_W), goal.1.clamp(0, MH * SQ - VIEW_H)));
    }

    // ---- the guided start ----------------------------------------------------------------------

    fn lesson_nest(&self) -> Option<Id> {
        self.world.bs.iter().find(|b| b.kind == BKind::LoneNest && !b.dead).map(|b| b.id)
    }

    pub fn lesson_line(&self) -> Option<&'static str> {
        let since = self.tick.saturating_sub(self.lesson_t);
        Some(match self.lesson {
            Lesson::Elder if since < 70 => "YOUR GOAL: BREAK GORF'S HOLLOW, FAR TO THE EAST.",
            Lesson::Elder => "YOU ARE THE ELDER. YOU CANNOT ORDER ANYONE.",
            Lesson::Offer => "A HERON NESTS CLOSE. OFFER DEW FOR IT.",
            Lesson::Weigh => "PIP WILL GO. BUFO WANTS MORE.",
            Lesson::Raise => "RAISE IT, IF BUFO IS WORTH IT.",
            Lesson::Cave => "THE CHORUS LOG CAN BUILD AN IRONCAP CAVE, UNDER THE CLIFFS.",
            Lesson::Wait => "IT WAITS FOR 20 DEW AND 4 FUNGIWOOD. NEWTS WILL CARRY THEM.",
            Lesson::Done => return None,
        })
    }

    fn lesson_step(&mut self) {
        let t = self.tick;
        if self.lesson == Lesson::Done {
            return;
        }
        if self.lesson_t == 0 {
            self.lesson_t = t;
        }
        let since = t - self.lesson_t;
        let nest = self.lesson_nest();
        let flag = nest.and_then(|n| self.world.flags.iter().find(|f| !f.done && f.target == RTarget::Bldg(n)).map(|f| f.id));
        let next = match self.lesson {
            Lesson::Elder => since > 140,
            Lesson::Offer => {
                if since == 1 {
                    if let Some(n) = nest {
                        let c = self.world.b(n).centre();
                        self.pan_to(c);
                    }
                }
                flag.is_some() || nest.is_none() || since > 1350
            }
            Lesson::Weigh => {
                let taken = flag.and_then(|f| self.world.flag(f)).is_some_and(|f| !f.takers.is_empty());
                (taken && since > 45) || since > 150 || flag.is_none()
            }
            Lesson::Raise => {
                let bufo = self.world.cs.iter().find(|c| c.name == "BUFO").map(|c| c.id);
                let took = match (flag.and_then(|f| self.world.flag(f)), bufo) {
                    (Some(f), Some(b)) => f.takers.contains(&b),
                    _ => true,
                };
                if took || since > 300 {
                    if let Some(c) = self.world.find_bld(BKind::ChorusLog) {
                        let p = self.world.b(c).centre();
                        // Show the Chorus Log and the cliffs where the cave can go.
                        let site = crate::bot::Bot::site_for(&self.world, BKind::IroncapCave, p).unwrap_or((p.0, p.1 - 4));
                        self.pan_to((site.0 + 1, site.1 + 4));
                        self.sel = Sel::B(c);
                    }
                    true
                } else {
                    false
                }
            }
            Lesson::Cave => self.world.bs.iter().any(|b| b.kind == BKind::IroncapCave && !b.dead) || since > 600,
            Lesson::Wait => since > 120,
            Lesson::Done => false,
        };
        if next {
            self.lesson = match self.lesson {
                Lesson::Elder => Lesson::Offer,
                Lesson::Offer => Lesson::Weigh,
                Lesson::Weigh => Lesson::Raise,
                Lesson::Raise => Lesson::Cave,
                Lesson::Cave => Lesson::Wait,
                _ => Lesson::Done,
            };
            self.lesson_t = t;
            if self.lesson == Lesson::Done {
                self.world.lesson = false;
                self.say("THE MARSH WAKES. HERONS AND WORSE WILL COME.");
            }
        }
    }

    // ---- input ---------------------------------------------------------------------------------

    fn input(&mut self, i: In) {
        match i {
            In::Move(x, y) => self.mouse = (x, y),
            In::Down(b, x, y) => {
                self.mouse = (x, y);
                self.click(b, x, y);
            }
            In::Up(..) => {}
            In::Key(k, shift) => self.key(k, shift),
            In::Char(c) => self.char(c),
        }
    }

    fn click(&mut self, b: u8, x: i32, y: i32) {
        if self.screen != Screen::Play {
            if b == 0 {
                self.buttons_for_screen();
                if let Some(bt) = self.buttons.iter().find(|bt| bt.enabled && x >= bt.x && x < bt.x + bt.w && y >= bt.y && y < bt.y + bt.h).cloned() {
                    self.act(bt.act);
                }
            }
            return;
        }
        if b == 2 {
            self.back();
            return;
        }
        if b != 0 {
            return;
        }
        // Buttons first.
        if let Some(bt) = self.buttons.iter().find(|bt| x >= bt.x && x < bt.x + bt.w && y >= bt.y && y < bt.y + bt.h).cloned() {
            if bt.enabled {
                self.sfx |= sfx::CLICK;
                self.act(bt.act);
            } else {
                self.sfx |= sfx::NOPE;
            }
            return;
        }
        // The stock window's lines.
        if let Mode::Stock(_) = self.mode {
            if let Some(line) = self.stock_line_at(x, y) {
                self.stock_line = line;
                return;
            }
            let n = self.stock_items().len() as i32;
            if (100..300).contains(&x) && (30..30 + 28 + n * 10).contains(&y) {
                return;
            }
        }
        // The build menu: click a building's name to choose it.
        if let Mode::BuildMenu(src) = self.mode {
            let opts = World::buildable(self.world.b(src).kind);
            let (wx, wy, ww) = (80, 20, 240);
            let h = 14 + opts.len().max(1) as i32 * 29 + 6;
            if x >= wx && x < wx + ww && y >= wy && y < wy + h {
                let row = (y - wy - 14) / 29;
                if y >= wy + 14 && (row as usize) < opts.len() {
                    self.sfx |= sfx::CLICK;
                    self.act(Act::Choose(opts[row as usize]));
                }
                return;
            }
        }
        // The spell list: click a spell to choose it.
        if self.mode == Mode::Spells {
            let (wx, wy, ww) = (20, 140, 360);
            if x >= wx && x < wx + ww && y >= wy && y < wy + 68 {
                let row = (y - wy - 14) / 10;
                if y >= wy + 14 && (0..5).contains(&row) {
                    self.sfx |= sfx::CLICK;
                    self.act(Act::Spell(SPELLS[row as usize]));
                }
                return;
            }
        }
        if let Mode::Info(info) = self.mode {
            if y < VIEW_H {
                let row = ((y - 30) / 10).max(0) as usize + self.info_scroll;
                self.pick_info_row(info, row);
            }
            return;
        }
        // The minimap.
        if (299..399).contains(&x) && (243..288).contains(&y) {
            let sq = (x - 299, y - 243);
            self.centre_on(sq);
            return;
        }
        if y >= VIEW_H {
            return;
        }
        let sq = ((x + self.cam.0) / SQ, (y + self.cam.1) / SQ);
        self.map_click(sq, (x + self.cam.0, y + self.cam.1));
    }

    fn map_click(&mut self, sq: (i32, i32), px: (i32, i32)) {
        match self.mode {
            Mode::Place(src, k) => {
                let fp = bld(k).fp;
                let pos = (sq.0 - fp.0 / 2, sq.1 - fp.1 / 2);
                match self.world.order_build(src, k, pos) {
                    Ok(id) => {
                        self.sfx |= sfx::POST;
                        self.mode = Mode::Normal;
                        self.sel = Sel::B(id);
                    }
                    Err(e) => self.nope(e),
                }
            }
            Mode::Route(a) => {
                if let Some(b) = self.world.bld_at(sq).filter(|b| self.world.b(*b).team == Team::Hold) {
                    self.world.route(a, b);
                    self.sfx |= sfx::POST;
                    self.say(format!("THE {} NOW FEEDS THE {}.", self.bname(a), self.bname(b)));
                    self.mode = Mode::Normal;
                } else {
                    self.nope("CLICK ONE OF YOUR BUILDINGS");
                }
            }
            Mode::Puddle => match self.world.mark_puddle(sq) {
                Ok(()) => self.sfx |= sfx::POST,
                Err(e) => self.nope(e),
            },
            Mode::Explore => {
                let dew = REWARD_FIRST;
                match self.world.post_reward(RKind::Explore, RTarget::Square(sq), dew) {
                    Ok(f) => {
                        self.sfx |= sfx::POST;
                        self.sel = Sel::Flag(f);
                        self.mode = Mode::Normal;
                    }
                    Err(e) => self.nope(e),
                }
            }
            Mode::Aim(s) => match self.world.cast(s, sq) {
                Ok(()) => {
                    self.sfx |= sfx::BOLT;
                }
                Err(e) => {
                    self.nope(e);
                    self.mode = Mode::Normal;
                }
            },
            _ => {
                self.sel = self.pick_at(sq, px);
                if self.sel != Sel::None {
                    self.sfx |= sfx::CLICK;
                }
                if !matches!(self.mode, Mode::Stock(_)) {
                    self.mode = Mode::Normal;
                } else if let Sel::B(b) = self.sel {
                    self.mode = Mode::Stock(b);
                } else {
                    self.mode = Mode::Normal;
                }
            }
        }
    }

    /// What's under the pointer: a creature, a reward flag, a husk, an item, then a building.
    fn pick_at(&self, sq: (i32, i32), px: (i32, i32)) -> Sel {
        let w = &self.world;
        if !self.seen(sq) {
            return Sel::None;
        }
        // Flags sit on their targets: a click on the flag itself selects the reward.
        for f in w.flags.iter().filter(|f| !f.done) {
            if let Some(p) = w.target_pos(f.target) {
                let fx = p.0 * SQ + 10;
                let fy = p.1 * SQ - 14;
                if px.0 >= fx && px.0 < fx + 8 && px.1 >= fy && px.1 < fy + 12 {
                    return Sel::Flag(f.id);
                }
            }
        }
        let mut best: Option<(i32, Id)> = None;
        for c in w.cs.iter().filter(|c| c.alive() && self.shown(c)) {
            let (cx, cy) = c.px();
            let s = self.creature_sprite(c);
            let x0 = cx + 8 - s.w / 2;
            let y0 = cy + 16 - s.h;
            if px.0 >= x0 - 2 && px.0 < x0 + s.w + 2 && px.1 >= y0 - 2 && px.1 < y0 + s.h + 2 {
                let d = (px.0 - (x0 + s.w / 2)).abs() + (px.1 - (y0 + s.h / 2)).abs();
                if best.is_none_or(|b| d < b.0) {
                    best = Some((d, c.id));
                }
            }
        }
        if let Some((_, c)) = best {
            return Sel::C(c);
        }
        if let Some(h) = w.husks.iter().find(|h| !h.gone && h.pos == sq) {
            return Sel::Husk(h.id);
        }
        if let Some(g) = w.ground.iter().position(|g| !g.gone && g.pos == sq) {
            return Sel::Ground(g);
        }
        if let Some(b) = w.bld_at(sq) {
            return Sel::B(b);
        }
        for b in w.bs.iter().filter(|b| !b.dead && self.seen(b.centre())) {
            let s = self.art.building(Self::blook(b.kind));
            let (x0, y1) = (b.pos.0 * SQ + b.fp.0 * SQ / 2 - s.w / 2, (b.pos.1 + b.fp.1) * SQ);
            if px.0 >= x0 && px.0 < x0 + s.w && px.1 >= y1 - s.h && px.1 < y1 {
                return Sel::B(b.id);
            }
        }
        Sel::None
    }

    fn back(&mut self) {
        self.message = None;
        match self.mode {
            Mode::Normal => {
                if self.sel != Sel::None {
                    self.sel = Sel::None;
                } else {
                    self.mode = Mode::Menu;
                }
            }
            Mode::Note | Mode::Feedback => self.mode = Mode::Normal,
            _ => self.mode = Mode::Normal,
        }
        self.cursor_sq = None;
    }

    fn key(&mut self, k: u8, shift: bool) {
        if self.mode == Mode::Note {
            self.held = [false; 4];
            match k {
                K_ENTER => {
                    let n = std::mem::take(&mut self.note);
                    if !n.trim().is_empty() {
                        self.send("note", &n);
                    }
                    self.mode = Mode::Normal;
                }
                K_ESC => {
                    self.note.clear();
                    self.mode = Mode::Normal;
                }
                K_BACK => {
                    self.note.pop();
                }
                _ => {}
            }
            return;
        }
        if self.screen != Screen::Play {
            if k == K_ENTER {
                self.act(if self.screen == Screen::Title { Act::Start } else { Act::Again });
            }
            if k == K_F2 {
                self.scanlines = !self.scanlines;
            }
            return;
        }
        if (K_HOLD_W..K_HOLD_W + 4).contains(&k) {
            self.held[(k - K_HOLD_W) as usize] = true;
            return;
        }
        if (K_LET_W..K_LET_W + 4).contains(&k) {
            self.held[(k - K_LET_W) as usize] = false;
            return;
        }
        match k {
            K_ESC => self.back(),
            K_F2 => self.scanlines = !self.scanlines,
            K_F3 => self.act(Act::Fast),
            K_F4 => self.act(Act::Magic),
            K_F5 => self.mode = Mode::Info(Info::Heroes),
            K_F6 => self.mode = Mode::Info(Info::Folk),
            K_F7 => self.mode = Mode::Info(Info::Rewards),
            K_F8 => self.mode = Mode::Info(Info::Log),
            K_F9 => self.mode = Mode::Feedback,
            K_HOME => self.centre_on_hall(),
            K_TAB => self.tab(shift),
            K_ENTER if self.lesson != Lesson::Done && self.cursor_sq.is_none() && matches!(self.mode, Mode::Normal) => {
                self.lesson_t = 0;
                self.lesson = match self.lesson {
                    Lesson::Elder => Lesson::Offer,
                    Lesson::Offer => Lesson::Weigh,
                    Lesson::Weigh => Lesson::Raise,
                    Lesson::Raise => Lesson::Cave,
                    Lesson::Cave => Lesson::Wait,
                    _ => Lesson::Done,
                };
                if self.lesson == Lesson::Done {
                    self.world.lesson = false;
                }
            }
            K_ENTER => {
                if let Some(c) = self.cursor_sq {
                    let px = (c.0 * SQ + 8, c.1 * SQ + 8);
                    self.map_click(c, px);
                }
            }
            K_UP | K_DOWN | K_LEFT | K_RIGHT => {
                let (dx, dy) = match k {
                    K_UP => (0, -1),
                    K_DOWN => (0, 1),
                    K_LEFT => (-1, 0),
                    _ => (1, 0),
                };
                if let Mode::Stock(_) = self.mode {
                    if dy != 0 {
                        let n = self.stock_items().len().max(1);
                        self.stock_line = (self.stock_line as i32 + dy).rem_euclid(n as i32) as usize;
                        return;
                    }
                }
                if let Mode::Info(_) = self.mode {
                    self.info_scroll = (self.info_scroll as i32 + dy).max(0) as usize;
                    return;
                }
                let aiming = matches!(self.mode, Mode::Place(..) | Mode::Route(_) | Mode::Puddle | Mode::Explore | Mode::Aim(_));
                if aiming {
                    let c = self.cursor_sq.unwrap_or(((self.cam.0 + FB_W / 2) / SQ, (self.cam.1 + VIEW_H / 2) / SQ));
                    let c = ((c.0 + dx).clamp(0, MW - 1), (c.1 + dy).clamp(0, MH - 1));
                    self.cursor_sq = Some(c);
                    self.keep_in_view(c);
                } else {
                    self.cam.0 += dx * 16;
                    self.cam.1 += dy * 16;
                    self.clamp_cam();
                }
            }
            _ => {}
        }
    }

    fn keep_in_view(&mut self, c: (i32, i32)) {
        let (px, py) = (c.0 * SQ, c.1 * SQ);
        if px < self.cam.0 + 16 {
            self.cam.0 = px - 16;
        }
        if px > self.cam.0 + FB_W - 32 {
            self.cam.0 = px - FB_W + 32;
        }
        if py < self.cam.1 + 16 {
            self.cam.1 = py - 16;
        }
        if py > self.cam.1 + VIEW_H - 32 {
            self.cam.1 = py - VIEW_H + 32;
        }
        self.clamp_cam();
    }

    fn char(&mut self, c: char) {
        if self.mode == Mode::Note {
            if self.note.len() < 120 && (c.is_ascii_graphic() || c == ' ') {
                self.note.push(c.to_ascii_uppercase());
            }
            return;
        }
        if self.screen != Screen::Play {
            let c = c.to_ascii_uppercase();
            self.buttons_for_screen();
            if let Some(bt) = self.buttons.iter().find(|b| b.key == c && b.enabled).cloned() {
                self.act(bt.act);
            }
            return;
        }
        if c == ' ' {
            self.act(Act::Pause);
            return;
        }
        if matches!(c, '+' | '=') {
            if let Mode::Stock(_) = self.mode {
                self.act(Act::Plus);
            } else {
                self.act(Act::More);
            }
            return;
        }
        if c == '-' {
            if let Mode::Stock(_) = self.mode {
                self.act(Act::Minus);
            } else {
                self.act(Act::Less);
            }
            return;
        }
        if let Some(d) = c.to_digit(10) {
            if self.mode == Mode::Spells && (1..=5).contains(&d) {
                self.act(Act::Spell(SPELLS[d as usize - 1]));
                return;
            }
        }
        let c = c.to_ascii_uppercase();
        if let Some(bt) = self.buttons.iter().find(|b| b.key == c).cloned() {
            if bt.enabled {
                self.sfx |= sfx::CLICK;
                self.act(bt.act);
            } else {
                self.sfx |= sfx::NOPE;
            }
            if matches!(c, 'W' | 'A' | 'S' | 'D') {
                // A button took the letter: don't scroll as well.
                self.held = [false; 4];
            }
            return;
        }
        if c == 'P' {
            self.act(Act::Pause);
        }
    }

    /// Tab steps through what's on screen: buildings, then creatures, nearest the centre first.
    fn tab(&mut self, back: bool) {
        let w = &self.world;
        let mut list: Vec<(i32, Sel)> = Vec::new();
        let centre = (self.cam.0 + FB_W / 2, self.cam.1 + VIEW_H / 2);
        for b in w.bs.iter().filter(|b| !b.dead) {
            let c = b.centre();
            let p = (c.0 * SQ, c.1 * SQ);
            if self.on_screen(p) && self.seen(c) {
                list.push(((p.0 - centre.0).abs() + (p.1 - centre.1).abs(), Sel::B(b.id)));
            }
        }
        for c in w.cs.iter().filter(|c| c.alive() && self.shown(c)) {
            let p = c.px();
            if self.on_screen(p) {
                list.push(((p.0 - centre.0).abs() + (p.1 - centre.1).abs() + 10_000, Sel::C(c.id)));
            }
        }
        list.sort_by_key(|x| x.0);
        if list.is_empty() {
            return;
        }
        let at = list.iter().position(|x| x.1 == self.sel);
        let next = match at {
            Some(k) if back => (k + list.len() - 1) % list.len(),
            Some(k) => (k + 1) % list.len(),
            None => 0,
        };
        self.sel = list[next].1;
        self.sfx |= sfx::CLICK;
    }

    fn on_screen(&self, p: (i32, i32)) -> bool {
        p.0 >= self.cam.0 - 8 && p.0 < self.cam.0 + FB_W && p.1 >= self.cam.1 - 8 && p.1 < self.cam.1 + VIEW_H
    }

    fn seen(&self, sq: (i32, i32)) -> bool {
        on_map(sq.0, sq.1) && self.world.map.explored[idx(sq.0, sq.1)]
    }

    /// Whether to draw a creature: always our own; others only where the Hold can see now.
    fn shown(&self, c: &crate::world::Creature) -> bool {
        c.team == Team::Hold || self.world.visible(c.pos)
    }

    fn bname(&self, b: Id) -> String {
        bld(self.world.b(b).kind).name.trim_start_matches("THE ").to_string()
    }

    // ---- actions -------------------------------------------------------------------------------

    fn act(&mut self, a: Act) {
        match a {
            Act::Start => {
                self.frames = 1;
                self.start(true);
            }
            Act::Again => self.restart(false),
            Act::Restart => self.restart(false),
            Act::Resume => self.mode = Mode::Normal,
            Act::Hints => self.hints_on = !self.hints_on,
            Act::Pause => self.paused = !self.paused,
            Act::Fast => self.fast = !self.fast,
            Act::Magic => self.mode = if self.mode == Mode::Spells { Mode::Normal } else { Mode::Spells },
            Act::Screens => {
                self.mode = match self.mode {
                    Mode::Info(Info::Heroes) => Mode::Info(Info::Folk),
                    Mode::Info(Info::Folk) => Mode::Info(Info::Rewards),
                    Mode::Info(Info::Rewards) => Mode::Info(Info::Log),
                    Mode::Info(Info::Log) => Mode::Normal,
                    _ => Mode::Info(Info::Heroes),
                };
                self.info_scroll = 0;
            }
            Act::Spell(s) => match self.world.spell_state(s) {
                Ok(()) => {
                    if spell(s).aimed {
                        self.mode = Mode::Aim(s);
                    } else {
                        let at = self.world.hall().map(|h| self.world.b(h).centre()).unwrap_or((0, 0));
                        if let Err(e) = self.world.cast(s, at) {
                            self.nope(e);
                        } else {
                            self.sfx |= sfx::BOLT;
                        }
                        self.mode = Mode::Normal;
                    }
                }
                Err(e) => self.nope(e),
            },
            Act::Reward => self.post_reward(),
            Act::FlagOf => {
                if let Some(f) = self.flag_on_sel() {
                    self.sel = Sel::Flag(f);
                }
            }
            Act::More | Act::Less => {
                let f = match self.sel {
                    Sel::Flag(f) => Some(f),
                    _ => self.flag_on_sel(),
                };
                if let Some(f) = f {
                    let by = if a == Act::More { REWARD_STEP } else { -REWARD_STEP };
                    if let Err(e) = self.world.change_reward(f, by) {
                        self.nope(e);
                    }
                }
            }
            Act::Withdraw => {
                let f = match self.sel {
                    Sel::Flag(f) => Some(f),
                    _ => self.flag_on_sel(),
                };
                if let Some(f) = f {
                    self.world.cancel_reward(f);
                    self.sel = Sel::None;
                }
            }
            Act::Build => {
                if let Sel::B(b) = self.sel {
                    self.mode = Mode::BuildMenu(b);
                }
            }
            Act::Choose(k) => {
                if let Mode::BuildMenu(src) = self.mode {
                    match self.world.build_blocker(src, k) {
                        None => {
                            self.mode = Mode::Place(src, k);
                            self.cursor_sq = None;
                        }
                        Some(e) => self.nope(e),
                    }
                }
            }
            Act::Hatch => {
                if let Sel::B(b) = self.sel {
                    self.world.toggle_auto_hatch(b);
                }
            }
            Act::Stock => {
                if let Sel::B(b) = self.sel {
                    self.mode = if self.mode == Mode::Stock(b) { Mode::Normal } else { Mode::Stock(b) };
                    self.stock_line = 0;
                }
            }
            Act::Plus | Act::Minus => {
                if let Mode::Stock(b) = self.mode {
                    let items = self.stock_items();
                    if let Some(&it) = items.get(self.stock_line) {
                        let cur = self.world.b(b).keep(it);
                        let step = if it == Item::Dew { 5 } else { 1 };
                        let n = if a == Act::Plus { cur + step } else { cur - step };
                        self.world.set_keep(b, it, n);
                    }
                }
            }
            Act::Defaults => {
                if let Mode::Stock(b) = self.mode {
                    let k = self.world.b(b).kind;
                    for (it, _) in self.world.b(b).keeps.clone() {
                        let d = bld(k).keeps.iter().find(|x| x.0 == it).map_or(0, |x| x.1);
                        self.world.set_keep(b, it, d);
                    }
                }
            }
            Act::Route => {
                if let Sel::B(b) = self.sel {
                    self.mode = Mode::Route(b);
                }
            }
            Act::Unroute => {
                if let Sel::B(b) = self.sel {
                    self.world.unroute(b);
                }
            }
            Act::Puddle => self.mode = if self.mode == Mode::Puddle { Mode::Normal } else { Mode::Puddle },
            Act::Explore => self.mode = Mode::Explore,
            Act::CancelSite => {
                if let Sel::B(b) = self.sel {
                    self.world.cancel_site(b);
                    self.sel = Sel::None;
                }
            }
            Act::Keep => self.send("keep", ""),
            Act::Change => self.send("change", ""),
            Act::Drop => self.send("drop", ""),
            Act::Note => {
                self.note.clear();
                self.mode = Mode::Note;
            }
            Act::Back => self.back(),
        }
    }

    fn flag_on_sel(&self) -> Option<Id> {
        let t = match self.sel {
            Sel::C(c) => RTarget::Creature(c),
            Sel::B(b) => RTarget::Bldg(b),
            Sel::Husk(h) => RTarget::Husk(h),
            Sel::Ground(g) => RTarget::Ground(g),
            _ => return None,
        };
        self.world.flags.iter().find(|f| !f.done && f.target == t).map(|f| f.id)
    }

    fn post_reward(&mut self) {
        let w = &self.world;
        let (kind, target) = match self.sel {
            Sel::C(c) => (RKind::Attack, RTarget::Creature(c)),
            Sel::B(b) => {
                if w.b(b).team == Team::Hold {
                    (RKind::Guard, RTarget::Bldg(b))
                } else {
                    (RKind::Attack, RTarget::Bldg(b))
                }
            }
            Sel::Husk(h) => (RKind::Fetch, RTarget::Husk(h)),
            Sel::Ground(g) => (RKind::Fetch, RTarget::Ground(g)),
            _ => return,
        };
        match self.world.post_reward(kind, target, REWARD_FIRST) {
            Ok(f) => {
                self.sfx |= sfx::POST;
                self.sel = Sel::Flag(f);
            }
            Err(e) => self.nope(e),
        }
    }

    fn send(&mut self, kind: &str, text: &str) {
        let w = &self.world;
        let heroes = w.cs.iter().filter(|c| c.alive() && c.hero()).count();
        let dew = w.hall().map_or(0, |h| w.b(h).has(Item::Dew));
        let state = format!(
            "{} dew, {} heroes, {} buildings, {} lairs left, lesson {:?}, selected {:?}",
            dew,
            heroes,
            w.hold_count(),
            w.bs.iter().filter(|b| !b.dead && b.team == Team::Gorf).count(),
            self.lesson,
            self.sel
        );
        let esc = |s: &str| s.replace('\\', "\\\\").replace('"', "\\\"");
        self.outbox.push(format!(
            "{{\"kind\":\"{}\",\"note\":\"{}\",\"game_time\":\"{}\",\"state\":\"{}\"}}",
            esc(kind),
            esc(text),
            clock(w.t),
            esc(&state)
        ));
        self.say(format!("NOTED: {}", kind.to_uppercase()));
        self.mode = Mode::Normal;
    }

    // ---- what the stock window lists

    fn stock_items(&self) -> Vec<Item> {
        let Mode::Stock(b) = self.mode else { return Vec::new() };
        let bb = self.world.b(b);
        let k = bb.kind;
        let mut v: Vec<Item> = ITEMS.iter().copied().filter(|it| limit(k, *it) > 0 || bb.keep(*it) > 0).collect();
        if k == BKind::Hall {
            v.retain(|i| *i != Item::Dew);
        }
        v
    }

    fn stock_line_at(&self, x: i32, y: i32) -> Option<usize> {
        let n = self.stock_items().len() as i32;
        let (wx, wy) = (100, 30);
        if x >= wx && x < wx + 200 && y >= wy + 14 && y < wy + 14 + n * 10 {
            return Some(((y - wy - 14) / 10) as usize);
        }
        None
    }

    fn pick_info_row(&mut self, info: Info, row: usize) {
        let ids: Vec<Id> = match info {
            Info::Heroes => self.world.cs.iter().filter(|c| c.alive() && c.hero() && c.team == Team::Hold).map(|c| c.id).collect(),
            Info::Folk => self.world.cs.iter().filter(|c| c.alive() && c.folk() && c.team == Team::Hold).map(|c| c.id).collect(),
            Info::Rewards => {
                if let Some(f) = self.world.flags.iter().filter(|f| !f.done).nth(row) {
                    self.sel = Sel::Flag(f.id);
                    if let Some(p) = self.world.target_pos(f.target) {
                        self.centre_on(p);
                    }
                    self.mode = Mode::Normal;
                }
                return;
            }
            Info::Log => return,
        };
        if let Some(&c) = ids.get(row) {
            self.sel = Sel::C(c);
            let p = self.world.c(c).pos;
            self.centre_on(p);
            self.mode = Mode::Normal;
        }
    }

    // ---- drawing -------------------------------------------------------------------------------

    fn sky(&self) -> Sky {
        let dt = day_tick(self.world.t);
        let dark = if dt < 300 {
            1
        } else if dt < DUSK {
            0
        } else if dt < DUSK + 300 {
            1
        } else {
            2
        };
        Sky { dark, rain: self.world.weather.rain, dry: self.world.weather.dry }
    }

    pub fn present(&mut self, n: usize) -> &[u8] {
        let pal = palette(self.tick, self.sky());
        self.fb.present(&mut self.out, n, self.scanlines, &pal);
        &self.out
    }

    /// The pointer as RGBA at scale `n`: each logical pixel an exact n x n block, index 0 fully
    /// transparent, everything else fully opaque. Returns (pixels, w, h).
    pub fn cursor(&mut self, n: usize) -> (&[u8], usize, usize) {
        let s = self.art.pointer();
        let (w, h) = (s.w as usize * n, s.h as usize * n);
        let mut px = vec![0u8; w * h * 4];
        for y in 0..h {
            for x in 0..w {
                let c = s.px[(y / n) * s.w as usize + x / n];
                if c != 0 {
                    let v = rgb(c);
                    px[(y * w + x) * 4..(y * w + x) * 4 + 4].copy_from_slice(&[(v >> 16) as u8, (v >> 8) as u8, v as u8, 255]);
                }
            }
        }
        self.cursor_px = px;
        (&self.cursor_px, w, h)
    }

    fn creature_sprite(&self, c: &crate::world::Creature) -> &Sprite {
        let look = match c.kind {
            Kind::Newt => Look::Newt,
            Kind::Caecilian => Look::Caecilian,
            Kind::TreeFrog => Look::TreeFrog,
            Kind::Salamander => Look::Salamander,
            Kind::ChorusToad => Look::ChorusToad,
            Kind::Toad => Look::Toad,
            Kind::Axolotl => Look::Axolotl,
            Kind::Bullfrog => Look::Bullfrog,
            Kind::Goliath => Look::Goliath,
            Kind::Olm => Look::Olm,
            Kind::Daot => Look::Daot,
            Kind::Heron => Look::Heron,
            Kind::Snake => Look::Snake,
            Kind::Crow => Look::Crow,
            Kind::Wisp => Look::Wisp,
            Kind::Turtle => Look::Turtle,
            Kind::Gorf => Look::Gorf,
        };
        let age = match c.age() {
            0 => Age::Young,
            1 => Age::Adult,
            _ => Age::Old,
        };
        let moving = c.step_t < c.step_len;
        let frame = (self.tick / 4) % 2 == 0;
        let pose = if c.dug_in {
            Pose::DugIn
        } else if c.stacks(Cond::Dreaming) >= 1.0 {
            Pose::Dream
        } else if c.info().flies && moving {
            if frame {
                Pose::Fly0
            } else {
                Pose::Fly1
            }
        } else if moving {
            if frame {
                Pose::Walk0
            } else {
                Pose::Walk1
            }
        } else if c.used.iter().any(|u| self.world.t < u.1 + 6 && ability(u.0).kind == AKind::Attack) {
            Pose::Strike
        } else if matches!(c.task, Task::Harvest { .. } | Task::Plant { .. } | Task::Tend { .. } | Task::Craft { .. } | Task::Repair { .. } | Task::DigPuddle { .. } | Task::Widen { .. }) {
            if (self.tick / 6) % 2 == 0 {
                Pose::Work0
            } else {
                Pose::Work1
            }
        } else {
            Pose::Walk0
        };
        let left = c.pos.0 < c.from.0 || (c.pos.0 == c.from.0 && c.id % 2 == 0);
        self.art.creature(look, age, pose, left)
    }

    fn blook(k: BKind) -> BLook {
        use BKind::*;
        match k {
            Hall => BLook::Hall,
            Burrow => BLook::Burrow,
            TwigLoft => BLook::TwigLoft,
            MorelCave => BLook::MorelCave,
            FairyRing => BLook::FairyRing,
            ChorusLog => BLook::ChorusLog,
            EmberLog => BLook::EmberLog,
            IroncapCave => BLook::IroncapCave,
            ToadHall => BLook::ToadHall,
            ShellForge => BLook::ShellForge,
            Cauldron => BLook::Cauldron,
            StarPond => BLook::StarPond,
            NewtHouse => BLook::NewtHouse,
            SpawningPool => BLook::SpawningPool,
            DaotsGrotto => BLook::DaotsGrotto,
            OlmsCave => BLook::OlmsCave,
            Nest => BLook::Nest,
            LoneNest => BLook::LoneNest,
            GorfStone => BLook::GorfStone,
            LeechPool => BLook::LeechPool,
            CrowRoost => BLook::CrowRoost,
            GorfHollow => BLook::GorfHollow,
        }
    }

    pub fn item_look(i: Item) -> ItemLook {
        use Item::*;
        match i {
            Dew => ItemLook::Dew,
            Fungiwood => ItemLook::Fungiwood,
            CapIngot => ItemLook::CapIngot,
            SpottedCap => ItemLook::SpottedCap,
            GlowSpores => ItemLook::GlowSpores,
            SunOrb => ItemLook::SunOrb,
            ClargentOrb => ItemLook::ClargentOrb,
            MeteorOrb => ItemLook::MeteorOrb,
            WoodenShovel => ItemLook::WoodenShovel,
            IronShovel => ItemLook::IronShovel,
            VenomwoodShovel => ItemLook::VenomwoodShovel,
            Slingshot => ItemLook::Slingshot,
            Catapult => ItemLook::Catapult,
            IronSpear => ItemLook::IronSpear,
            VenomwoodSpear => ItemLook::VenomwoodSpear,
            VenomironSpear => ItemLook::VenomironSpear,
            Chanfron => ItemLook::Chanfron,
            Peytral => ItemLook::Peytral,
            Croupiere => ItemLook::Croupiere,
            VenomironChanfron => ItemLook::VenomironChanfron,
            DewBead => ItemLook::DewBead,
            GlowCharm => ItemLook::GlowCharm,
            StarPearl => ItemLook::StarPearl,
        }
    }

    fn cond_look(c: Cond) -> CondLook {
        match c {
            Cond::Bolstered => CondLook::Bolstered,
            Cond::Poisoned => CondLook::Poisoned,
            Cond::Blinded => CondLook::Blinded,
            Cond::Regenerating => CondLook::Regenerating,
            Cond::Altered => CondLook::Altered,
            Cond::Dry => CondLook::Dry,
            Cond::Dreaming => CondLook::Dreaming,
            Cond::Leeched => CondLook::Leeched,
            Cond::Frightened => CondLook::Frightened,
        }
    }

    fn draw(&mut self) {
        self.fb.clear(INK);
        match self.screen {
            Screen::Title => self.draw_title(),
            Screen::Play => {
                if let Mode::Info(info) = self.mode {
                    self.draw_info(info);
                } else {
                    self.draw_map();
                }
                self.draw_panel();
                self.draw_overlays();
            }
            Screen::End => self.draw_end(),
        }
    }

    fn draw_map(&mut self) {
        let (cx, cy) = self.cam;
        let w = &self.world;
        let x0 = cx / SQ;
        let y0 = cy / SQ;
        // Ground.
        for sy in y0..=(y0 + VIEW_H / SQ + 1) {
            for sx in x0..=(x0 + FB_W / SQ + 1) {
                if !on_map(sx, sy) {
                    continue;
                }
                let i = idx(sx, sy);
                let px = sx * SQ - cx;
                let py = sy * SQ - cy;
                if !w.map.explored[i] {
                    continue;
                }
                let t = if w.map.curse[i] > 0 {
                    TileLook::Cursed
                } else if w.map.puddle[i] > 0 {
                    TileLook::Puddle
                } else {
                    match w.map.t[i] {
                        Tile::Water => TileLook::Water,
                        Tile::Deep => TileLook::Deep,
                        Tile::Moss => TileLook::Moss,
                        Tile::Fern => TileLook::Fern,
                        Tile::Mud => TileLook::Mud,
                        Tile::Path => TileLook::Path,
                        Tile::Sand => TileLook::Sand,
                        Tile::Rock => TileLook::Rock,
                        Tile::Reeds => TileLook::Reeds,
                    }
                };
                let v = ((((sx as u32).wrapping_mul(2654435761) ^ (sy as u32).wrapping_mul(40503)) >> 7) % TILE_VARIANTS as u32) as u8;
                let s = self.art.tile(t, v);
                self.fb.blit(s, px, py);
                if w.map.leech[i] != 0 && w.map.wet(sx, sy) {
                    for (dx, dy) in [(3, 4), (10, 8), (6, 12)] {
                        self.fb.rect(px + dx, py + dy, 3, 1, HP_LOW);
                        self.fb.pset(px + dx + 1, py + dy + 1, HP_LOW);
                    }
                }
            }
        }
        // Things, sorted by their bottom edge.
        let mut draws: Vec<(i32, i32, u8, u32)> = Vec::new(); // (bottom y, x, kind, id/index)
        for sy in y0 - 3..=(y0 + VIEW_H / SQ + 4) {
            for sx in x0 - 2..=(x0 + FB_W / SQ + 2) {
                if on_map(sx, sy) && w.map.deco[idx(sx, sy)].is_some() && w.map.explored[idx(sx, sy)] {
                    draws.push((sy * SQ + 15, sx, 0, idx(sx, sy) as u32));
                }
            }
        }
        for (k, b) in w.bs.iter().enumerate() {
            let c = b.centre();
            if (b.pos.0 + b.fp.0) * SQ < cx - 32 || b.pos.0 * SQ > cx + FB_W + 32 || !self.seen(c) {
                continue;
            }
            if b.dead && b.team != Team::Hold {
                continue;
            }
            draws.push(((b.pos.1 + b.fp.1) * SQ - 1, b.pos.0, 1, k as u32));
        }
        for (k, s) in w.spots.iter().enumerate() {
            if self.seen(s.pos) && w.bi(s.owner).is_some_and(|bi| !w.bs[bi].dead && !w.bs[bi].site) {
                draws.push((s.pos.1 * SQ + 14, s.pos.0, 2, k as u32));
            }
        }
        for (k, h) in w.husks.iter().enumerate() {
            if !h.gone && self.seen(h.pos) {
                draws.push((h.pos.1 * SQ + 13, h.pos.0, 3, k as u32));
            }
        }
        for (k, g) in w.ground.iter().enumerate() {
            if !g.gone && self.seen(g.pos) {
                draws.push((g.pos.1 * SQ + 12, g.pos.0, 4, k as u32));
            }
        }
        for (k, c) in w.cs.iter().enumerate() {
            if c.alive() && self.shown(c) {
                let p = c.px();
                draws.push((p.1 + 15, p.0 / SQ, 5, k as u32));
            }
        }
        draws.sort_by_key(|d| (d.0, d.2));
        for (bottom, _, kind, id) in draws {
            match kind {
                0 => {
                    let i = id as usize;
                    let sx = (i as i32) % MW;
                    let d = match self.world.map.deco[i].unwrap() {
                        crate::terrain::Deco::BigTree => Deco::BigTree,
                        crate::terrain::Deco::Tree => Deco::Tree,
                        crate::terrain::Deco::Stump => Deco::Stump,
                        crate::terrain::Deco::Stone => Deco::Stone,
                        crate::terrain::Deco::LeyStone => Deco::LeyStone,
                        crate::terrain::Deco::Lily => Deco::Lily,
                        crate::terrain::Deco::ReedClump => Deco::ReedClump,
                        crate::terrain::Deco::Mushroom => Deco::Mushroom,
                        crate::terrain::Deco::Log => Deco::Log,
                    };
                    let s = self.art.deco(d);
                    self.fb.blit(s, sx * SQ + 8 - s.w / 2 - cx, bottom + 1 - s.h - cy);
                }
                1 => self.draw_bld(id as usize),
                2 => {
                    let sp = &self.world.spots[id as usize];
                    let stage = match sp.state {
                        0 | 3 => 0,
                        1 => {
                            if sp.progress * 2 < spot(sp.kind).ripen {
                                1
                            } else {
                                2
                            }
                        }
                        _ => 3,
                    };
                    let look = match sp.kind {
                        SpotKind::Fern => SpotLook::Fern,
                        SpotKind::Red => SpotLook::RedFungus,
                        SpotKind::White => SpotLook::WhiteFungus,
                        SpotKind::Hybrid => SpotLook::HybridFungus,
                        SpotKind::Puff => SpotLook::Puffball,
                    };
                    let s = self.art.spot(look, stage);
                    self.fb.blit(s, sp.pos.0 * SQ + 8 - s.w / 2 - cx, sp.pos.1 * SQ + 16 - s.h - cy);
                }
                3 => {
                    let h = &self.world.husks[id as usize];
                    let s = self.art.husk();
                    self.fb.blit(s, h.pos.0 * SQ + 8 - s.w / 2 - cx, h.pos.1 * SQ + 15 - s.h - cy);
                }
                4 => {
                    let g = &self.world.ground[id as usize];
                    let s = self.art.item(Self::item_look(g.item));
                    self.fb.blit(s, g.pos.0 * SQ + 4 - cx, g.pos.1 * SQ + 6 - cy);
                }
                _ => self.draw_creature(id as usize),
            }
        }
        let w = &self.world;
        // Reward flags on their targets.
        for f in w.flags.iter().filter(|f| !f.done) {
            if let Some(p) = w.target_pos(f.target) {
                let (fx, fy) = (p.0 * SQ + 10 - cx, p.1 * SQ - 14 - cy);
                let s = self.art.flag();
                self.fb.blit(s, fx, fy);
                let t = format!("{}", f.dew);
                self.fb.text_shadow(&t, fx + 9, fy, KEY, 1);
                if self.sel == Sel::Flag(f.id) {
                    self.fb.frame_dotted(fx - 2, fy - 2, s.w + 4 + text_w(&t) + 2, s.h + 4, HILITE);
                }
            }
        }
        // Blows: a pebble in flight, a spark where it lands, lightning, a ring of pond light.
        let tick = self.tick;
        self.shots.retain(|s| tick < s.3 + 8);
        for &(a, b, k, born) in self.shots.clone().iter() {
            let age = (tick - born) as i32;
            let (ax, ay) = (a.0 * SQ + 8 - cx, a.1 * SQ + 6 - cy);
            let (bx, by) = (b.0 * SQ + 8 - cx, b.1 * SQ + 6 - cy);
            match k {
                1 if age < 4 => {
                    let (px, py) = (ax + (bx - ax) * (age + 1) / 4, ay + (by - ay) * (age + 1) / 4);
                    self.fb.rect(px, py, 2, 2, STONE2);
                }
                2 if age < 3 => {
                    let steps = 6;
                    let mut last = (ax, ay - 4);
                    for s in 1..=steps {
                        let jig = if s % 2 == 0 { 2 } else { -2 };
                        let p = (ax + (bx - ax) * s / steps + if s < steps { jig } else { 0 }, ay - 4 + (by - ay + 4) * s / steps);
                        self.fb.line(last.0, last.1, p.0, p.1, DEW);
                        last = p;
                    }
                }
                3 if age < 5 => {
                    let r = 8 + age * 6;
                    for t in 0..32 {
                        let (dx, dy) = ([0, 2, 4, 5, 6, 5, 4, 2][t % 8], [6, 5, 4, 2, 0, -2, -4, -5][t % 8]);
                        let q = t / 8;
                        let (sx, sy) = match q {
                            0 => (dx, dy),
                            1 => (dy, -dx),
                            2 => (-dx, -dy),
                            _ => (-dy, dx),
                        };
                        self.fb.pset(bx + sx * r / 6, by + sy * r / 6, GLOW);
                    }
                }
                _ => {}
            }
            // The spark where it lands.
            let land = match k {
                1 => 3,
                _ => 0,
            };
            if age >= land && age < land + 3 {
                for (dx, dy) in [(0, 0), (-1, -1), (1, 1), (-1, 1), (1, -1)] {
                    self.fb.pset(bx + dx * (1 + age - land), by + dy * (1 + age - land), if k == 0 { TEXT } else { KEY });
                }
            }
        }
        // Fog: seen-but-not-now is dithered; never seen is black.
        for sy in y0..=(y0 + VIEW_H / SQ + 1) {
            for sx in x0..=(x0 + FB_W / SQ + 1) {
                if !on_map(sx, sy) {
                    continue;
                }
                let i = idx(sx, sy);
                let (px, py) = (sx * SQ - cx, sy * SQ - cy);
                if !w.map.explored[i] {
                    self.fb.rect(px, py, SQ, SQ, INK);
                } else if !w.map.visible[i] {
                    self.fb.checker(px, py, SQ, SQ, SHADE);
                }
            }
        }
        // The selection, drawn over the fog so its dots are never hidden by the fog's dither.
        match self.sel {
            Sel::B(b) => {
                if let Some(k) = self.world.bi(b) {
                    let bb = &self.world.bs[k];
                    let (x0, y0, w, h) = (bb.pos.0 * SQ - cx, bb.pos.1 * SQ - cy, bb.fp.0 * SQ, bb.fp.1 * SQ);
                    self.fb.frame(x0 - 1, y0 - 1, w + 2, h + 2, HILITE);
                }
            }
            Sel::C(c) => {
                if let Some(k) = self.world.ci(c) {
                    let cr = self.world.cs[k].clone();
                    let (px, py) = cr.px();
                    let s = self.creature_sprite(&cr);
                    let (sw, sh) = (s.w, s.h);
                    let (x, y) = (px + 8 - sw / 2 - cx, py + 16 - sh - cy);
                    for (dx, dy) in [(-2, -2), (sw + 1, -2), (-2, sh + 1), (sw + 1, sh + 1)] {
                        self.fb.pset(x + dx, y + dy, HILITE);
                        self.fb.pset(x + dx + if dx < 0 { 1 } else { -1 }, y + dy, HILITE);
                        self.fb.pset(x + dx, y + dy + if dy < 0 { 1 } else { -1 }, HILITE);
                    }
                }
            }
            _ => {}
        }
        // The placing ghost, and the keyboard cursor.
        let ptr_sq = self.cursor_sq.unwrap_or(((self.mouse.0 + cx) / SQ, (self.mouse.1 + cy) / SQ));
        if let Mode::Place(_, k) = self.mode {
            // Mark every place in view where it can go.
            let info = bld(k);
            for sy in y0..=(y0 + VIEW_H / SQ) {
                for sx in x0..=(x0 + FB_W / SQ) {
                    if w.map.explored[idx(sx.clamp(0, MW - 1), sy.clamp(0, MH - 1))] && w.map.can_place(sx, sy, info.fp.0, info.fp.1, info.place) && w.hold_blds().any(|b| b.dist_to((sx, sy)) <= 20) {
                        let (px, py) = ((sx + info.fp.0 / 2) * SQ + 5 - cx, (sy + info.fp.1 / 2) * SQ + 5 - cy);
                        self.fb.frame(px, py, 6, 6, GHOST_OK);
                        self.fb.rect(px + 2, py + 2, 2, 2, GHOST_OK);
                    }
                }
            }
        }
        match self.mode {
            Mode::Place(_, k) => {
                let info = bld(k);
                let pos = (ptr_sq.0 - info.fp.0 / 2, ptr_sq.1 - info.fp.1 / 2);
                let ok = w.map.can_place(pos.0, pos.1, info.fp.0, info.fp.1, info.place);
                let col = if ok { GHOST_OK } else { GHOST_NO };
                self.fb.checker(pos.0 * SQ - cx, pos.1 * SQ - cy, info.fp.0 * SQ, info.fp.1 * SQ, col);
                self.fb.frame(pos.0 * SQ - cx, pos.1 * SQ - cy, info.fp.0 * SQ, info.fp.1 * SQ, col);
            }
            Mode::Puddle | Mode::Explore | Mode::Aim(_) | Mode::Route(_) => {
                let r = if let Mode::Aim(s) = self.mode { spell(s).radius } else { 0 };
                let col = if let Mode::Aim(_) = self.mode { DEW } else { HILITE };
                self.fb.frame_dotted((ptr_sq.0 - r) * SQ - cx, (ptr_sq.1 - r) * SQ - cy, (2 * r + 1) * SQ, (2 * r + 1) * SQ, col);
            }
            _ => {}
        }
        if let Some(c) = self.cursor_sq {
            self.fb.frame(c.0 * SQ - cx, c.1 * SQ - cy, SQ, SQ, KEY);
        }
        // Routes from the selected building.
        if let Sel::B(b) = self.sel {
            let bb = w.b(b).clone();
            for r in bb.routes.iter().filter_map(|r| w.bi(*r)) {
                let a = bb.centre();
                let c = w.bs[r].centre();
                let steps = dist(a, c).max(1) * 4;
                for s in 0..=steps {
                    if s % 2 == 0 {
                        let x = (a.0 * SQ + 8) + ((c.0 - a.0) * SQ) * s / steps - cx;
                        let y = (a.1 * SQ + 8) + ((c.1 - a.1) * SQ) * s / steps - cy;
                        self.fb.pset(x, y, KEY);
                    }
                }
            }
        }
    }

    fn draw_bld(&mut self, k: usize) {
        let (cx, cy) = self.cam;
        let b = self.world.bs[k].clone();
        let (x0, y0) = (b.pos.0 * SQ - cx, b.pos.1 * SQ - cy);
        let (w, h) = (b.fp.0 * SQ, b.fp.1 * SQ);
        let s = if b.dead {
            self.art.ruin(b.fp.0, b.fp.1)
        } else if b.site || (b.grown == 0 && b.hp < b.max_hp && b.team == Team::Hold) {
            self.art.site(b.fp.0, b.fp.1)
        } else {
            self.art.building(Self::blook(b.kind))
        };
        self.fb.blit(s, x0 + w / 2 - s.w / 2, y0 + h - s.h);
        if !b.dead {
            if b.hatching.is_some() {
                let stage = (self.world.t % 900 / 300) as u8;
                let e = self.art.eggs(stage);
                self.fb.blit(e, x0 + w / 2 - e.w / 2, y0 + h - 2);
            }
            let show_bar = b.hp < b.max_hp && (b.grown != 0 || b.team != Team::Hold) || self.sel == Sel::B(b.id) || b.site;
            if show_bar && !b.site {
                let fill = (w - 4) * b.hp / b.max_hp.max(1);
                self.fb.rect(x0 + 2, y0 - 4, w - 4, 3, INK);
                self.fb.rect(x0 + 2, y0 - 4, fill, 3, if b.hp * 3 < b.max_hp { HP_LOW } else { HP_OK });
            }
        }
    }

    fn draw_creature(&mut self, k: usize) {
        let (cx, cy) = self.cam;
        let c = self.world.cs[k].clone();
        let (px, py) = c.px();
        let s = self.creature_sprite(&c);
        let (sw, sh) = (s.w, s.h);
        let x = px + 8 - sw / 2 - cx;
        let y = py + 16 - sh - cy;
        let s = self.creature_sprite(&c).clone();
        self.fb.blit(&s, x, y);
        if let Some((it, n)) = c.carry {
            if n > 0 {
                let ic = self.art.item(Self::item_look(it));
                self.fb.blit(ic, x + sw / 2 - 3, y - 8);
            }
        }
        if c.dmg > 0 && (self.world.t < c.last_hit + 120 || self.sel == Sel::C(c.id)) {
            let bw = 12;
            let fill = bw * c.hp() / c.max_hp().max(1);
            self.fb.rect(x + sw / 2 - bw / 2, y - 3, bw, 2, INK);
            self.fb.rect(x + sw / 2 - bw / 2, y - 3, fill.max(1), 2, if c.hp() * 3 < c.max_hp() { HP_LOW } else { HP_OK });
        }
        if c.stacks(Cond::Dry) >= 3.0 && c.team == Team::Hold {
            let d = self.art.cond(CondLook::Dry);
            self.fb.blit(d, x + sw, y - 2);
        }
    }

    fn draw_overlays(&mut self) {
        // The event line (or the lesson), above the panel.
        let lesson = self.lesson_line();
        let line = lesson.map(|s| s.to_string()).or_else(|| self.line.as_ref().filter(|l| self.tick < l.1).map(|l| l.0.clone()));
        if let Some(text) = line {
            if !matches!(self.mode, Mode::Info(_)) {
                self.fb.rect(0, 0, FB_W, 14, SHADE);
                self.fb.rect(0, 14, FB_W, 1, INK);
                let col = if lesson.is_some() { KEY } else { TEXT };
                self.fb.text_centered(&text, FB_W / 2, 4, col, 1);
                if lesson.is_some() {
                    self.fb.text_shadow("ENTER: NEXT", FB_W - 70, 18, TEXT_DIM, 1);
                }
            }
        }
        if let Some((m, until)) = self.message.clone() {
            if self.tick < until {
                let wd = text_w(&m) + 12;
                self.fb.bevel(FB_W / 2 - wd / 2, 100, wd, 15, PANEL, PANEL_HI, PANEL_LO);
                self.fb.text_centered(&m, FB_W / 2, 104, WARN, 1);
            }
        }
        if self.paused && !matches!(self.mode, Mode::Menu) {
            self.fb.bevel(FB_W / 2 - 30, 18, 60, 13, PANEL, PANEL_HI, PANEL_LO);
            self.fb.text_centered("PAUSED", FB_W / 2, 21, KEY, 1);
        }
        match self.mode {
            Mode::Stock(b) => self.draw_stock(b),
            Mode::BuildMenu(src) => self.draw_build_menu(src),
            Mode::Spells => self.draw_spells(),
            Mode::Menu => self.draw_menu(),
            Mode::Feedback | Mode::Note => self.draw_feedback(),
            _ => {}
        }
        let hint = match self.mode {
            Mode::Place(_, k) => Some(format!("PLACE THE {}: CLICK, OR ARROWS AND ENTER", bld(k).name.trim_start_matches("THE "))),
            Mode::Route(_) => Some("CLICK THE BUILDING IT SHOULD FEED".into()),
            Mode::Puddle => Some("CLICK DRY GROUND FOR A PUDDLE (5 DEW). ESC TO STOP".into()),
            Mode::Explore => Some("CLICK WHERE A HERO SHOULD GO".into()),
            Mode::Aim(s) => Some(format!("{}: CLICK WHERE. ESC TO STOP", spell(s).name)),
            _ => None,
        };
        if let Some(h) = hint {
            self.fb.text_shadow(&h, 4, VIEW_H - 11, KEY, 1);
        }
    }

    fn window(&mut self, x: i32, y: i32, w: i32, h: i32, title: &str) {
        self.fb.bevel(x, y, w, h, PANEL, PANEL_HI, PANEL_LO);
        self.fb.text(title, x + 5, y + 4, KEY, 1);
        self.fb.rect(x + 3, y + 12, w - 6, 1, PANEL_LO);
    }

    fn draw_stock(&mut self, b: Id) {
        let items = self.stock_items();
        let (wx, wy, ww) = (100, 30, 200);
        let h = 14 + items.len() as i32 * 10 + 14;
        let title = format!("{}: STOCK", self.bname(b));
        self.window(wx, wy, ww, h, &title);
        let bb = self.world.b(b).clone();
        for (n, it) in items.iter().enumerate() {
            let y = wy + 15 + n as i32 * 10;
            if n == self.stock_line {
                self.fb.rect(wx + 3, y - 1, ww - 6, 9, BTN_ON);
            }
            let ic = self.art.item(Self::item_look(*it));
            self.fb.blit(ic, wx + 6, y);
            self.fb.text(item(*it).name, wx + 16, y + 1, TEXT, 1);
            let t = format!("{}/{}", bb.has(*it), bb.keep(*it));
            self.fb.text(&t, wx + ww - 8 - text_w(&t), y + 1, if bb.has(*it) < bb.keep(*it) { WARN } else { TEXT }, 1);
        }
        let why = items.get(self.stock_line).map(|it| self.why_short(&bb, *it)).unwrap_or_default();
        self.fb.text(&why, wx + 6, wy + h - 11, TEXT_DIM, 1);
    }

    /// Why a stock line is short, in plain words (documents 02 and 04).
    fn why_short(&self, b: &crate::world::Building, it: Item) -> String {
        let w = &self.world;
        if b.has(it) >= b.keep(it) {
            return "NEWTS FETCH UP TO WHAT IT KEEPS".into();
        }
        if let Some(maker) = HOLD_BUILDINGS.iter().find(|k| bld(**k).makes.contains(&it)) {
            let ii = item(it);
            if *maker == b.kind {
                let skill = ii.craft.unwrap();
                let best = w.cs.iter().filter(|c| c.alive()).map(|c| c.skill(skill)).max().unwrap_or(-9);
                if best < ii.craft_lvl {
                    return "NOBODY SKILLED ENOUGH YET".into();
                }
                if let Some((inp, _)) = ii.inputs.iter().find(|(inp, n)| b.has(*inp) < *n) {
                    return format!("WAITING FOR {}", item(*inp).name);
                }
                return "BEING MADE".into();
            }
        }
        if w.total_want(it) > 0 && !w.bs.iter().any(|o| o.id != b.id && w.spare(o, it) > 0) && !w.ground.iter().any(|g| !g.gone && g.item == it) {
            let ripening = w.spots.iter().any(|s| spot(s.kind).gives == it && s.state == 1);
            return if ripening { "NONE SPARE. IT'S RIPENING".into() } else { "NONE SPARE".into() };
        }
        "NEWTS WILL CARRY IT".into()
    }

    fn draw_build_menu(&mut self, src: Id) {
        let k = self.world.b(src).kind;
        let opts = World::buildable(k);
        let (wx, wy, ww) = (80, 20, 240);
        let h = 14 + opts.len().max(1) as i32 * 29 + 6;
        let title = format!("BUILD FROM THE {}", self.bname(src));
        self.window(wx, wy, ww, h, &title);
        if opts.is_empty() {
            self.fb.text("NOTHING IS BUILT FROM HERE", wx + 6, wy + 18, TEXT_DIM, 1);
        }
        for (n, o) in opts.iter().enumerate() {
            let y = wy + 16 + n as i32 * 29;
            let block = self.world.build_blocker(src, *o);
            let name = bld(*o).name.trim_start_matches("THE ");
            let key = (b'1' + n as u8) as char;
            self.fb.text(&format!("{key}"), wx + 6, y, KEY, 1);
            self.fb.text(name, wx + 16, y, if block.is_none() { TEXT } else { TEXT_DIM }, 1);
            self.fb.text(purpose(*o), wx + 16, y + 9, GOOD, 1);
            let cost = bld(*o).cost.iter().map(|(i, n)| format!("{n} {}", item(*i).name)).collect::<Vec<_>>().join(", ");
            let sub = block.unwrap_or(cost);
            self.fb.text(&sub, wx + 16, y + 18, TEXT_DIM, 1);
        }
    }

    fn draw_spells(&mut self) {
        let (wx, wy, ww) = (20, 140, 360);
        self.window(wx, wy, ww, 14 + 5 * 10 + 4, "THE ANCIENT MAGIC (1-5)");
        for (n, s) in SPELLS.iter().enumerate() {
            let y = wy + 15 + n as i32 * 10;
            let state = self.world.spell_state(*s);
            let si = spell(*s);
            let ic = self.art.ui(match s {
                Spell::RainCall => UiLook::RainCall,
                Spell::OldSong => UiLook::OldSong,
                Spell::Starlight => UiLook::Starlight,
                Spell::LeyLight => UiLook::LeyLight,
                Spell::DeepCalling => UiLook::DeepCalling,
            });
            let _ = ic;
            self.fb.text(&format!("{}", n + 1), wx + 6, y, KEY, 1);
            self.fb.text(si.name, wx + 16, y, if state.is_ok() { TEXT } else { TEXT_DIM }, 1);
            let right = match state {
                Ok(()) => format!("{} DEW", si.cost),
                Err(e) => e,
            };
            self.fb.text(&right, wx + ww - 6 - text_w(&right), y, TEXT_DIM, 1);
        }
    }

    fn draw_menu(&mut self) {
        let (wx, wy, ww, wh) = (130, 70, 140, 80);
        self.window(wx, wy, ww, wh, "THE HOLD");
        self.fb.text(&format!("DAY {}  {}", self.world.t / DAY + 1, clock(self.world.t)), wx + 6, wy + 18, TEXT_DIM, 1);
    }

    fn draw_feedback(&mut self) {
        let (wx, wy, ww, wh) = (50, 80, 300, 60);
        self.window(wx, wy, ww, wh, "FEEDBACK FOR HENRY'S PLAYTEST");
        if self.mode == Mode::Note {
            self.fb.text("TYPE A NOTE, THEN ENTER:", wx + 6, wy + 18, TEXT_DIM, 1);
            let n = format!("{}_", self.note);
            let shown: String = n.chars().rev().take(46).collect::<Vec<_>>().into_iter().rev().collect();
            self.fb.text(&shown, wx + 6, wy + 30, TEXT, 1);
        } else {
            self.fb.text("KEEP, CHANGE, DROP OR A NOTE:", wx + 6, wy + 18, TEXT_DIM, 1);
        }
    }

    fn draw_info(&mut self, info: Info) {
        self.fb.rect(0, 0, FB_W, VIEW_H, PANEL_LO);
        let title = match info {
            Info::Heroes => "HEROES (F5)",
            Info::Folk => "FOLK (F6)",
            Info::Rewards => "REWARDS (F7)",
            Info::Log => "LOG (F8)",
        };
        self.fb.text(title, 6, 6, KEY, 1);
        let help = if matches!(info, Info::Log) { "ESC CLOSES." } else { "ESC CLOSES. CLICK A LINE TO GO THERE." };
        self.fb.text(help, 120, 6, TEXT_DIM, 1);
        let w = &self.world;
        let mut lines: Vec<String> = Vec::new();
        match info {
            Info::Heroes | Info::Folk => {
                for c in w.cs.iter().filter(|c| c.alive() && c.team == Team::Hold && (c.hero() == (info == Info::Heroes))) {
                    let purse = if c.hero() { format!("{:>4}", c.purse) } else { "    ".into() };
                    let mut s = format!("{:<11} {:<11} L{:<2} {:<7} {} {}", c.name, c.info().name, c.level, c.trait_.name(), purse, w.doing(c));
                    s.truncate(64);
                    lines.push(s);
                }
            }
            Info::Rewards => {
                for f in w.flags.iter().filter(|f| !f.done) {
                    let what = match f.target {
                        RTarget::Creature(c) => w.ci(c).map_or("?", |k| w.cs[k].info().name).to_string(),
                        RTarget::Bldg(b) => bld(w.b(b).kind).name.to_string(),
                        RTarget::Square(p) => format!("SQUARE {},{}", p.0, p.1),
                        RTarget::Husk(_) => "A HUSK".into(),
                        RTarget::Ground(g) => item(w.ground[g].item).name.to_string(),
                    };
                    let mut s = format!("{:<8} {:<18} {:>4} DEW  {}", f.kind.name(), what, f.dew, w.candidate_summary(f.id));
                    s.truncate(64);
                    lines.push(s);
                }
            }
            Info::Log => {
                for (t, s) in w.log.iter().rev() {
                    let mut l = format!("{:>6} {}", clock(*t), s);
                    l.truncate(64);
                    lines.push(l);
                }
            }
        }
        let start = self.info_scroll.min(lines.len().saturating_sub(1));
        for (n, l) in lines.iter().skip(start).take(20).enumerate() {
            self.fb.text(l, 6, 30 + n as i32 * 10, TEXT, 1);
        }
        if lines.is_empty() {
            self.fb.text("NOTHING YET.", 6, 30, TEXT_DIM, 1);
        }
    }

    fn draw_title(&mut self) {
        self.fb.rect(0, 0, FB_W, 300, NIGHT);
        let hall = self.art.building(BLook::Hall).clone();
        self.fb.blit(&hall, 200 - hall.w / 2, 170 - hall.h);
        self.fb.text_centered("NAIBIHPMA HOLD", 200, 60, KEY, 3);
        self.fb.text_centered("THE ENCHANTED FOREST OF NAIBIHPMA", 200, 92, TEXT, 1);
        self.fb.text_centered("YOU ARE THE ELDER. THE HOLD DECIDES FOR ITSELF.", 200, 200, TEXT_DIM, 1);
        self.buttons_for_screen();
        let bts = self.buttons.clone();
        for b in bts {
            self.draw_button(&b);
        }
    }

    fn draw_end(&mut self) {
        self.fb.rect(0, 0, FB_W, 300, NIGHT);
        let won = self.world.over == Some(true);
        self.fb.text_centered(if won { "THE MARSH IS FREE" } else { "THE HALL HAS FALLEN" }, 200, 70, if won { GOOD } else { BAD }, 2);
        let w = &self.world;
        let lines = [
            format!("TIME {}", clock(w.t)),
            format!("FALLEN {}   ENEMIES DEFEATED {}", w.stats.deaths, w.stats.kills),
            format!("BUILDINGS {}", w.hold_count()),
        ];
        for (n, l) in lines.iter().enumerate() {
            self.fb.text_centered(l, 200, 110 + n as i32 * 12, TEXT, 1);
        }
        self.buttons_for_screen();
        let bts = self.buttons.clone();
        for b in bts {
            self.draw_button(&b);
        }
    }

    fn buttons_for_screen(&mut self) {
        let mut v = Vec::new();
        match self.screen {
            Screen::Title => {
                v.push(Button { label: "START".into(), key: 'S', x: 160, y: 220, w: 80, h: 15, act: Act::Start, enabled: true, icon: None });
                let hl = if self.hints_on { "HINTS ON" } else { "HINTS OFF" };
                v.push(Button { label: hl.into(), key: 'H', x: 160, y: 240, w: 80, h: 15, act: Act::Hints, enabled: true, icon: None });
            }
            Screen::End => {
                v.push(Button { label: "AGAIN".into(), key: 'A', x: 160, y: 200, w: 80, h: 15, act: Act::Again, enabled: true, icon: None });
                v.push(Button { label: "FEEDBACK".into(), key: 'F', x: 160, y: 220, w: 80, h: 15, act: Act::Note, enabled: true, icon: None });
            }
            Screen::Play => {}
        }
        self.buttons = v;
    }

    fn draw_button(&mut self, b: &Button) {
        let fill = if b.enabled { BTN } else { BTN_LO };
        self.fb.bevel(b.x, b.y, b.w, b.h, fill, BTN_HI, BTN_LO);
        if let Some(ic) = b.icon {
            let s = self.art.ui(ic).clone();
            self.fb.blit(&s, b.x + 1, b.y + 1);
            return;
        }
        let tw = text_w(&b.label);
        let x = b.x + b.w / 2 - tw / 2;
        let y = b.y + (b.h - 7) / 2;
        let col = if b.enabled { TEXT } else { TEXT_DIM };
        self.fb.text(&b.label, x, y, col, 1);
        if let Some(k) = b.label.chars().position(|c| c == b.key) {
            let ch: String = b.label.chars().nth(k).unwrap().to_string();
            self.fb.text(&ch, x + k as i32 * 6, y, if b.enabled { KEY } else { TEXT_DIM }, 1);
        }
    }

    // ---- the panel -----------------------------------------------------------------------------

    fn panel_buttons(&self) -> Vec<(String, char, Act, bool)> {
        let w = &self.world;
        let mut v: Vec<(String, char, Act, bool)> = Vec::new();
        match self.mode {
            Mode::Stock(_) => {
                v.push(("MORE +".into(), '+', Act::Plus, true));
                v.push(("LESS -".into(), '-', Act::Minus, true));
                v.push(("RESET".into(), 'R', Act::Defaults, true));
                v.push(("CLOSE".into(), 'C', Act::Back, true));
                return v;
            }
            Mode::BuildMenu(src) => {
                for (n, o) in World::buildable(w.b(src).kind).iter().enumerate() {
                    let key = (b'1' + n as u8) as char;
                    let ok = w.build_blocker(src, *o).is_none();
                    v.push((format!("{key}"), key, Act::Choose(*o), ok));
                }
                v.push(("BACK".into(), 'B', Act::Back, true));
                return v;
            }
            Mode::Feedback => {
                v.push(("KEEP".into(), 'K', Act::Keep, true));
                v.push(("CHANGE".into(), 'C', Act::Change, true));
                v.push(("DROP".into(), 'D', Act::Drop, true));
                v.push(("NOTE".into(), 'N', Act::Note, true));
                v.push(("BACK".into(), 'B', Act::Back, true));
                return v;
            }
            Mode::Menu => {
                v.push(("RESUME".into(), 'R', Act::Resume, true));
                v.push(("RESTART".into(), 'E', Act::Restart, true));
                let hl = if self.hints_on { "HINTS ON" } else { "HINTS OFF" };
                v.push((hl.into(), 'H', Act::Hints, true));
                v.push(("FEEDBACK".into(), 'F', Act::Note, true));
                return v;
            }
            Mode::Place(..) | Mode::Route(_) | Mode::Puddle | Mode::Explore | Mode::Aim(_) | Mode::Spells => {
                v.push(("BACK".into(), 'B', Act::Back, true));
                return v;
            }
            _ => {}
        }
        let flag = self.flag_on_sel();
        match self.sel {
            Sel::Flag(_) => {
                v.push(("MORE".into(), 'M', Act::More, true));
                v.push(("LESS".into(), 'L', Act::Less, true));
                v.push(("CANCEL".into(), 'C', Act::Withdraw, true));
            }
            Sel::C(c) => {
                let cr = w.c(c);
                if cr.team != Team::Hold {
                    if let Some(_f) = flag {
                        v.push(("REWARD".into(), 'R', Act::FlagOf, true));
                        v.push(("MORE".into(), 'M', Act::More, true));
                        v.push(("LESS".into(), 'L', Act::Less, true));
                    } else {
                        v.push(("REWARD".into(), 'R', Act::Reward, true));
                    }
                }
            }
            Sel::Husk(_) | Sel::Ground(_) => {
                if flag.is_some() {
                    v.push(("REWARD".into(), 'R', Act::FlagOf, true));
                } else {
                    v.push(("FETCH IT".into(), 'R', Act::Reward, true));
                }
            }
            Sel::B(b) => {
                let bb = w.b(b);
                if bb.team != Team::Hold {
                    if flag.is_some() {
                        v.push(("REWARD".into(), 'R', Act::FlagOf, true));
                        v.push(("MORE".into(), 'M', Act::More, true));
                        v.push(("LESS".into(), 'L', Act::Less, true));
                    } else {
                        v.push(("REWARD".into(), 'R', Act::Reward, true));
                    }
                } else if bb.site {
                    v.push(("CANCEL".into(), 'C', Act::CancelSite, true));
                } else {
                    if !World::buildable(bb.kind).is_empty() {
                        v.push(("BUILD".into(), 'B', Act::Build, bb.done()));
                    }
                    v.push(("STOCK".into(), 'T', Act::Stock, true));
                    if !bb.info().houses.is_empty() {
                        let l = if bb.auto_hatch { "HATCH:Y" } else { "HATCH:N" };
                        v.push((l.into(), 'H', Act::Hatch, true));
                    }
                    if bb.routes.is_empty() {
                        v.push(("ROUTE".into(), 'O', Act::Route, true));
                    } else {
                        v.push(("UNROUTE".into(), 'N', Act::Unroute, true));
                    }
                    if flag.is_some() {
                        v.push(("GUARD".into(), 'G', Act::FlagOf, true));
                    } else {
                        v.push(("GUARD".into(), 'G', Act::Reward, true));
                    }
                    if bb.kind == BKind::Hall {
                        v.push(("PUDDLE".into(), 'U', Act::Puddle, true));
                        v.push(("EXPLORE".into(), 'X', Act::Explore, true));
                    }
                }
            }
            Sel::None => {
                v.push(("EXPLORE".into(), 'X', Act::Explore, true));
                v.push(("PUDDLE".into(), 'U', Act::Puddle, true));
            }
        }
        v
    }

    fn draw_panel(&mut self) {
        let y0 = VIEW_H;
        self.fb.rect(0, y0, FB_W, 300 - y0, PANEL);
        self.fb.rect(0, y0, FB_W, 1, PANEL_HI);
        self.fb.rect(0, y0 + 1, FB_W, 1, INK);
        // Selection box.
        self.fb.rect(2, 243, 133, 55, PANEL_LO);
        self.fb.frame(2, 243, 133, 55, PANEL_HI);
        self.draw_selection();
        // Buttons: two or three rows of three.
        let mut bts = Vec::new();
        for (n, (label, key, act, ok)) in self.panel_buttons().into_iter().enumerate() {
            let (col, row) = ((n % 3) as i32, (n / 3) as i32);
            if row > 2 {
                break;
            }
            bts.push(Button { label, key, x: 138 + col * 48, y: 244 + row * 18, w: 46, h: 15, act, enabled: ok, icon: None });
        }
        // The icon column.
        let icons = [(UiLook::Pause, 'P', Act::Pause), (UiLook::Fast, '\u{1}', Act::Fast), (UiLook::Magic, '\u{2}', Act::Magic), (UiLook::Screens, '\u{3}', Act::Screens)];
        for (n, (ic, key, act)) in icons.iter().enumerate() {
            let look = if *ic == UiLook::Pause && self.paused { UiLook::Play } else { *ic };
            bts.push(Button { label: String::new(), key: *key, x: 284, y: 243 + n as i32 * 14, w: 13, h: 13, act: *act, enabled: true, icon: Some(look) });
        }
        for b in &bts {
            self.draw_button(b);
        }
        if self.fast {
            self.fb.frame(284, 257, 13, 13, KEY);
        }
        self.buttons = bts;
        // The minimap and the status line.
        self.draw_minimap();
        let w = &self.world;
        let dew = w.hall().map_or(0, |h| w.b(h).has(Item::Dew));
        let night = is_night(w.t);
        let s = format!("{} {}", if night { "NIGHT" } else { "DAY" }, w.t / DAY + 1);
        let ic = self.art.item(ItemLook::Dew).clone();
        self.fb.text(&s, 300, 290, TEXT_DIM, 1);
        let x = 300 + text_w(&s) + 6;
        self.fb.blit(&ic, x, 289);
        self.fb.text(&format!("{dew}"), x + ic.w + 1, 290, TEXT, 1);
    }

    fn draw_minimap(&mut self) {
        let (mx, my) = (299, 243);
        let w = &self.world;
        for y in 0..MH {
            for x in 0..MW {
                let i = idx(x, y);
                if !w.map.explored[i] {
                    self.fb.pset(mx + x, my + y, INK);
                    continue;
                }
                let c = if w.map.curse[i] > 0 {
                    CURSE1
                } else {
                    match w.map.t[i] {
                        Tile::Water | Tile::Deep => WATER2,
                        Tile::Rock => STONE1,
                        Tile::Sand | Tile::Path => PATH1,
                        Tile::Reeds => LEAF0,
                        Tile::Mud => PATH0,
                        _ => MOSS3,
                    }
                };
                self.fb.pset(mx + x, my + y, c);
            }
        }
        let blds: Vec<((i32, i32), (i32, i32), u8)> = w
            .bs
            .iter()
            .filter(|b| !b.dead && w.map.explored[idx(b.centre().0, b.centre().1)])
            .map(|b| (b.pos, b.fp, if b.team == Team::Hold { CREAM } else { BAD }))
            .collect();
        let foes: Vec<(i32, i32)> = w.cs.iter().filter(|c| c.alive() && c.team != Team::Hold && w.visible(c.pos)).map(|c| c.pos).collect();
        for (p, fp, col) in blds {
            self.fb.rect(mx + p.0, my + p.1, fp.0.min(3), fp.1.min(2), col);
        }
        for p in foes {
            self.fb.pset(mx + p.0, my + p.1, WARN);
        }
        let (vx, vy) = (mx + self.cam.0 / SQ, my + self.cam.1 / SQ);
        self.fb.frame(vx, vy, FB_W / SQ, VIEW_H / SQ, TEXT);
    }

    fn draw_selection(&mut self) {
        let (x, y) = (6, 247);
        let w = &self.world;
        match self.sel {
            Sel::None => {
                let lines = ["GOAL: BREAK GORF'S", "HOLLOW, FAR EAST.", "CLICK THINGS, OR TAB.", "F5 HEROES F7 REWARDS", "F4 ANCIENT MAGIC"];
                for (n, l) in lines.iter().enumerate() {
                    self.fb.text(l, x, y + n as i32 * 10, if n < 2 { KEY } else { TEXT_DIM }, 1);
                }
            }
            Sel::C(c) => {
                let cr = w.c(c).clone();
                self.fb.text(&cr.name, x, y, KEY, 1);
                let sub = if cr.role() == crate::data::Role::Enemy {
                    format!("{} L{}", cr.info().name, cr.level)
                } else {
                    format!("{} L{} {}", cr.info().name, cr.level, cr.trait_.name())
                };
                self.fb.text(&sub, x, y + 9, TEXT_DIM, 1);
                // Health.
                let bw = 60;
                self.fb.rect(x, y + 19, bw, 3, INK);
                self.fb.rect(x, y + 19, bw * cr.hp().max(0) / cr.max_hp().max(1), 3, HP_OK);
                if cr.hero() {
                    let p = format!("{}", cr.purse);
                    let ic = self.art.item(ItemLook::Dew).clone();
                    self.fb.blit(&ic, x + 66, y + 17);
                    self.fb.text(&p, x + 75, y + 17, TEXT, 1);
                }
                // Gear and conditions.
                let mut gx = x;
                for (_, it) in &cr.gear {
                    let s = self.art.item(Self::item_look(*it)).clone();
                    self.fb.blit(&s, gx, y + 24);
                    gx += s.w + 1;
                }
                let mut cx = x + 70;
                for (cnd, st, _) in &cr.conds {
                    if *st >= 1.0 {
                        let s = self.art.cond(Self::cond_look(*cnd)).clone();
                        self.fb.blit(&s, cx, y + 25);
                        cx += s.w + 2;
                    }
                }
                let mut doing = w.doing(&cr);
                if let Some(o) = cr.orb.filter(|_| w.cs.iter().any(|h| h.dreamt == Some(crate::world::Vision::Orb(cr.id)))) {
                    doing = format!("CARRIES A {}", item(o).name);
                }
                self.wrap(&doing, x, y + 34, 21, 2, TEXT);
            }
            Sel::B(b) => {
                let bb = w.b(b).clone();
                self.fb.text(bld(bb.kind).name, x, y, KEY, 1);
                let bw = 60;
                self.fb.rect(x, y + 10, bw, 3, INK);
                self.fb.rect(x, y + 10, bw * bb.hp / bb.max_hp.max(1), 3, HP_OK);
                let houses = bb.info().houses;
                if !houses.is_empty() && bb.team == Team::Hold {
                    let n = w.cs.iter().filter(|c| c.home == Some(b) && c.alive()).count();
                    let cap: i32 = houses.iter().map(|h| h.1).sum();
                    self.fb.text(&format!("{n}/{cap}"), x + 66, y + 8, TEXT_DIM, 1);
                }
                let status = if bb.site {
                    let cost = bld(bb.kind).cost;
                    let src = bb.src.and_then(|s| w.bi(s)).map(|k| w.bs[k].clone());
                    match src {
                        Some(s) => {
                            let missing: Vec<String> = cost.iter().filter(|(i, n)| s.has(*i) < *n).map(|(i, n)| format!("{} {}", n - s.has(*i), item(*i).name)).collect();
                            if missing.is_empty() {
                                "ITS MATERIALS ARE THERE".into()
                            } else {
                                format!("WAITING FOR {}", missing.join(", "))
                            }
                        }
                        None => "WAITING".into(),
                    }
                } else if bb.grown == 0 && bb.hp < bb.max_hp && bb.team == Team::Hold {
                    format!("BEING DUG UP: {}%", 100 * bb.hp / bb.max_hp)
                } else if let Some((k, ready)) = bb.hatching {
                    format!("HATCHING A {} ({})", kind(k).name, clock(ready.saturating_sub(w.t)))
                } else if bb.team != Team::Hold {
                    let n = w.cs.iter().filter(|c| c.home == Some(b) && c.alive() && dist(c.pos, bb.centre()) <= 3).count();
                    format!("{n} AT HOME")
                } else {
                    let st: Vec<String> = bb.stock.iter().filter(|s| s.1 > 0).take(4).map(|(i, n)| format!("{n} {}", item(*i).name)).collect();
                    if st.is_empty() {
                        purpose(bb.kind).into()
                    } else {
                        st.join(", ")
                    }
                };
                self.wrap(&status, x, y + 18, 21, 3, TEXT);
            }
            Sel::Flag(f) => {
                if let Some(fl) = w.flag(f).cloned() {
                    self.fb.text(&format!("{} REWARD", fl.kind.name()), x, y, KEY, 1);
                    self.fb.text(&format!("{} DEW SET ASIDE", fl.dew), x, y + 10, TEXT, 1);
                    let taken = fl.takers.iter().filter_map(|t| w.ci(*t)).map(|k| w.cs[k].name.clone()).collect::<Vec<_>>().join(", ");
                    let s = if taken.is_empty() { w.candidate_summary(f) } else { format!("TAKEN BY {taken}") };
                    let s = if s.is_empty() { "NOBODY HAS SEEN IT YET".into() } else { s };
                    self.wrap(&s, x, y + 20, 21, 3, TEXT_DIM);
                    // What each hero wants, for the lesson's Bufo and the like.
                    let _ = Answer::Taken;
                }
            }
            Sel::Husk(h) => {
                if let Some(hk) = w.husks.iter().find(|x| x.id == h).cloned() {
                    self.fb.text(&format!("{}'S HUSK", hk.was.name), x, y, KEY, 1);
                    let s = if hk.home {
                        format!("AT THE STAR POND. REGROWING COSTS {} DEW.", 10 * hk.was.level)
                    } else {
                        let left = (crate::data::HUSK_TICKS + hk.was.level as u32 * 300 + hk.since).saturating_sub(w.t);
                        format!("CARRY IT TO THE STAR POND WITHIN {}", clock(left))
                    };
                    self.wrap(&s, x, y + 12, 21, 3, TEXT);
                }
            }
            Sel::Ground(g) => {
                if let Some(gr) = w.ground.get(g).cloned() {
                    self.fb.text(&format!("{} {}", gr.n, item(gr.item).name), x, y, KEY, 1);
                    self.wrap("LYING ON THE GROUND. FOLK CARRY WHAT'S WANTED.", x, y + 12, 21, 3, TEXT_DIM);
                }
            }
        }
    }

    /// Text wrapped to `width` characters, at most `lines` lines.
    fn wrap(&mut self, s: &str, x: i32, y: i32, width: usize, lines: usize, col: u8) {
        let mut out: Vec<String> = Vec::new();
        let mut cur = String::new();
        for word in s.split(' ') {
            if !cur.is_empty() && cur.len() + 1 + word.len() > width {
                out.push(std::mem::take(&mut cur));
            }
            if !cur.is_empty() {
                cur.push(' ');
            }
            cur.push_str(word);
        }
        if !cur.is_empty() {
            out.push(cur);
        }
        for (n, l) in out.iter().take(lines).enumerate() {
            self.fb.text(l, x, y + n as i32 * 9, col, 1);
        }
    }
}

/// What a building is for, in a line (for the build menu and the panel).
pub fn purpose(k: BKind) -> &'static str {
    use BKind::*;
    match k {
        Hall => "THE ELDER'S POOL. NEWTS LIVE HERE.",
        Burrow => "CAECILIANS, WHO GROW FUNGUS",
        TwigLoft => "TREE FROGS: WOODEN SHOVELS, SLINGS",
        MorelCave => "GROWS MORELS, FOR FUNGIWOOD",
        FairyRing => "TOADSTOOLS, FOR SPOTTED CAPS",
        ChorusLog => "THE CHORUS TOAD. LEADS TO IRON",
        EmberLog => "SALAMANDERS: IRON SHOVELS",
        IroncapCave => "GROWS IRONCAPS, FOR CAP INGOTS",
        ToadHall => "TOAD HEROES. SLINGS AND SPEARS",
        ShellForge => "PLATES FOR THE BULLFROGS",
        Cauldron => "VENOMWOOD SPEARS AND SHOVELS",
        StarPond => "AXOLOTL MAGES. REGROWS THE FALLEN",
        NewtHouse => "FOUR MORE NEWTS, TWO MORE FERNS",
        SpawningPool => "BULLFROGS AND GOLIATH FROGS",
        DaotsGrotto => "DAOT: VENOMIRON, STAR PEARLS",
        OlmsCave => "THE OLM, OLDEST OF ALL",
        _ => "",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn frames(g: &mut Game, n: u32) {
        for _ in 0..n {
            g.frame();
        }
    }

    fn click_world(g: &mut Game, sq: (i32, i32)) {
        let (x, y) = (sq.0 * SQ + 8 - g.cam.0, sq.1 * SQ + 8 - g.cam.1);
        g.push(In::Down(0, x, y));
        g.push(In::Up(0, x, y));
        frames(g, 1);
    }

    #[test]
    fn the_guided_start_plays_with_the_mouse() {
        let mut g = Game::new(7);
        g.push(In::Char('s'));
        frames(&mut g, 2);
        assert_eq!(g.screen, Screen::Play);
        frames(&mut g, 160);
        assert_eq!(g.lesson, Lesson::Offer);
        // Select the lone nest and post a reward on it.
        let nest = g.lesson_nest().unwrap();
        let c = g.world.b(nest).centre();
        g.centre_on(c);
        click_world(&mut g, c);
        assert_eq!(g.sel, Sel::B(nest));
        g.push(In::Char('r'));
        frames(&mut g, 2);
        assert!(g.world.flags.iter().any(|f| f.target == RTarget::Bldg(nest)), "a reward on the nest");
        frames(&mut g, 400);
        assert!(matches!(g.lesson, Lesson::Raise | Lesson::Cave), "lesson at {:?}", g.lesson);
        // Raise it a couple of times.
        g.push(In::Char('m'));
        g.push(In::Char('m'));
        frames(&mut g, 400);
        assert_eq!(g.lesson, Lesson::Cave);
        // Build the Ironcap Cave from the Chorus Log, by keys.
        let chorus = g.world.find_bld(BKind::ChorusLog).unwrap();
        g.sel = Sel::B(chorus);
        g.mode = Mode::Normal;
        frames(&mut g, 1);
        g.push(In::Char('b'));
        frames(&mut g, 1);
        assert_eq!(g.mode, Mode::BuildMenu(chorus));
        let n = World::buildable(BKind::ChorusLog).iter().position(|k| *k == BKind::IroncapCave).unwrap();
        g.push(In::Char((b'1' + n as u8) as char));
        frames(&mut g, 1);
        assert!(matches!(g.mode, Mode::Place(_, BKind::IroncapCave)));
        let site = crate::bot::Bot::site_for(&g.world, BKind::IroncapCave, g.world.b(chorus).centre()).expect("a cave site");
        let fp = bld(BKind::IroncapCave).fp;
        click_world(&mut g, (site.0 + fp.0 / 2, site.1 + fp.1 / 2));
        assert!(g.world.bs.iter().any(|b| b.kind == BKind::IroncapCave), "the cave was ordered: {:?}", g.message);
        frames(&mut g, 200);
        assert_eq!(g.lesson, Lesson::Done);
        assert!(!g.world.lesson);
    }

    #[test]
    fn clicking_a_name_in_the_build_menu_chooses_it() {
        let mut g = Game::new(4);
        g.push(In::Char('s'));
        frames(&mut g, 2);
        g.lesson = Lesson::Done;
        g.world.lesson = false;
        let chorus = g.world.find_bld(BKind::ChorusLog).unwrap();
        g.sel = Sel::B(chorus);
        g.mode = Mode::BuildMenu(chorus);
        frames(&mut g, 1);
        let n = World::buildable(BKind::ChorusLog).iter().position(|k| *k == BKind::IroncapCave).unwrap() as i32;
        let y = 20 + 16 + n * 29 + 3;
        g.push(In::Down(0, 150, y));
        g.push(In::Up(0, 150, y));
        frames(&mut g, 1);
        assert_eq!(g.mode, Mode::Place(chorus, BKind::IroncapCave));
    }

    #[test]
    fn holding_wasd_scrolls_the_view() {
        let mut g = Game::new(4);
        g.push(In::Char('s'));
        frames(&mut g, 2);
        g.lesson = Lesson::Done;
        g.world.lesson = false;
        g.mouse = (200, 120);
        g.cam = (200, 200);
        g.push(In::Key(K_HOLD_W + 3, false));
        g.push(In::Char('d'));
        frames(&mut g, 5);
        assert_eq!(g.cam.0, 240, "D held scrolls right");
        g.push(In::Key(K_LET_W + 3, false));
        frames(&mut g, 5);
        assert_eq!(g.cam.0, 240, "and stops when let go");
        g.push(In::Key(K_HOLD_W, false));
        frames(&mut g, 3);
        assert_eq!(g.cam.1, 176, "W held scrolls up");
    }

    #[test]
    fn every_key_and_button_works_without_a_panic() {
        let mut g = Game::new(3);
        g.push(In::Char('s'));
        frames(&mut g, 2);
        g.lesson = Lesson::Done;
        g.world.lesson = false;
        for k in [K_F3, K_F4, K_ESC, K_F5, K_DOWN, K_ESC, K_F6, K_ESC, K_F7, K_ESC, K_F8, K_ESC, K_F9, K_ESC, K_HOME, K_TAB, K_TAB, K_LEFT, K_RIGHT, K_UP, K_DOWN] {
            g.push(In::Key(k, false));
            frames(&mut g, 1);
        }
        for c in "pbsohgux+-mlcpdrn123 ".chars() {
            g.push(In::Char(c));
            frames(&mut g, 2);
        }
        frames(&mut g, 3000);
    }
}
