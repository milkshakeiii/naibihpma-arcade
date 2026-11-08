//! CHANGELING: one piece on a small board. You take the shape of whatever you catch.

pub mod art;
pub mod gen;
mod render;
pub mod rules;

use crate::app::{Ev, Key, Out, Run, Summary};
use crate::audio::{Sfx, Track};
use crate::form::Question;
use crate::framebuffer::Framebuffer;
use crate::palette::Pal;
use crate::rng::Rng;
use art::Art;
use gen::{Carry, Trigger, AREAS, BURROWS, LESSONS};
use rules::*;

pub const VERSION: u32 = 2;
pub const LESSON_VERSION: u32 = 2;
pub static FORM_QUESTION: Question = Question { id: "shapes", text: "TAKING NEW SHAPES FELT...", low: "A CHORE", high: "GREAT" };

pub const TILE: i32 = 28;
pub const BX: i32 = 8;
pub const BY: i32 = 20;
const PH1: u32 = 3;
const PH2: u32 = 3;
const CARD_FRAMES: u32 = 22;

#[derive(Clone, Debug, PartialEq)]
pub enum Phase {
    Lesson,
    Burrow,
    Glade { offers: Vec<Charm>, first: bool },
    Over { won: bool },
}

pub struct Anim {
    pub prev: Floor,
    pub mid: Floor,
    pub ev_player: Vec<Event>,
    pub ev_enemy: Vec<Event>,
    pub t: u32,
}

impl Anim {
    pub fn len(&self) -> u32 {
        if self.ev_enemy.iter().any(|e| matches!(e, Event::Crept { .. } | Event::Strike { .. } | Event::Blocked { .. } | Event::Arrived { .. })) {
            PH1 + PH2
        } else {
            PH1
        }
    }
}

pub struct Particle {
    pub x: i32,
    pub y: i32,
    pub dx: i32,
    pub dy: i32,
    pub life: i32,
    pub col: u8,
}

#[derive(Default, Clone)]
pub struct Stats {
    pub turns: u32,
    pub hits: u32,
    pub catches: u32,
    pub rests: u32,
    pub deepest: u32,
    pub struck_by: Vec<Kind>,
}

pub struct Game {
    pub phase: Phase,
    pub floor: Floor,
    pub burrow: u32,
    pub lesson: Option<usize>,
    pub lesson_step: usize,
    pub lesson_moves: u32,
    pub lesson_caught: bool,
    pub lesson_rested: bool,
    pub lesson_chained: bool,
    /// Catches so far in the current turn.
    pub chain_len: u32,
    pub stats_best_chain: u32,
    pub carry: Carry,
    pub rng: Rng,
    pub art: Art,
    pub pal: Pal,
    pub mouse: (i32, i32),
    pub focus: Pos,
    pub anim: Option<Anim>,
    pub queued: Option<Action>,
    pub previews: Vec<(Pos, Preview)>,
    pub rest_preview: Preview,
    pub threatened: Vec<Pos>,
    pub message: Option<String>,
    pub tips_shown: Vec<&'static str>,
    pub show_help: bool,
    pub ended: bool,
    pub stats: Stats,
    pub think: u32,
    pub frame: u32,
    pub card: u32,
    pub particles: Vec<Particle>,
    pub hurt_flash: u32,
    pub glade_hover: Option<usize>,
    pub bought: Vec<String>,
    pub floor_turn0: u32,
    pub floor_frames: u32,
}

impl Game {
    pub fn new(seed: u64, lessons: bool) -> Self {
        let carry = Carry::fresh();
        let mut g = Self {
            phase: Phase::Burrow,
            floor: Floor::empty(Rng::new(1)),
            burrow: 0,
            lesson: None,
            lesson_step: 0,
            lesson_moves: 0,
            lesson_caught: false,
            lesson_rested: false,
            lesson_chained: false,
            chain_len: 0,
            stats_best_chain: 0,
            carry,
            rng: Rng::new(seed),
            art: Art::new(),
            pal: art::palette(),
            mouse: (-1, -1),
            focus: (0, 0),
            anim: None,
            queued: None,
            previews: Vec::new(),
            rest_preview: Preview::default(),
            threatened: Vec::new(),
            message: None,
            tips_shown: Vec::new(),
            show_help: false,
            ended: false,
            stats: Stats::default(),
            think: 0,
            frame: 0,
            card: 0,
            particles: Vec::new(),
            hurt_flash: 0,
            glade_hover: None,
            bought: Vec::new(),
            floor_turn0: 0,
            floor_frames: 0,
        };
        if lessons {
            g.start_lesson(0, &mut Out::default());
        } else {
            g.start_burrow(1, &mut Out::default());
        }
        g
    }

    pub fn area_name(&self) -> &'static str {
        if self.lesson.is_some() {
            "DOAT'S LESSONS"
        } else {
            AREAS[gen::area_of(self.burrow)].name
        }
    }

    pub fn track(&self) -> Track {
        match (&self.phase, self.lesson) {
            (Phase::Over { won: true }, _) => Track::AncientMagic,
            (_, Some(_)) => Track::Doat,
            _ if self.burrow == 0 => Track::Doat,
            _ => AREAS[gen::area_of(self.burrow)].track,
        }
    }

    fn start_lesson(&mut self, i: usize, out: &mut Out) {
        self.lesson = Some(i);
        self.lesson_step = 0;
        self.lesson_moves = 0;
        self.lesson_caught = false;
        self.lesson_rested = false;
        self.lesson_chained = false;
        self.chain_len = 0;
        let mut c = self.carry.clone();
        c.hearts = c.max_hearts;
        self.floor = gen::lesson_floor(i, &c);
        // The burrow stays shut until Doat is done talking.
        self.close_lesson_burrow(true);
        self.phase = Phase::Lesson;
        self.card = CARD_FRAMES;
        self.focus = self.floor.player;
        self.message = None;
        out.log(format!("lesson {}: {}", i + 1, LESSONS[i].title.to_lowercase()));
        out.music = Some(Track::Doat);
        self.refresh();
    }

    fn close_lesson_burrow(&mut self, closed: bool) {
        for y in 0..N {
            for x in 0..N {
                let r = LESSONS[self.lesson.unwrap()].rows[y as usize].as_bytes()[x as usize];
                if r == b'B' {
                    self.floor.cells[y as usize][x as usize] = if closed { Cell::Ground } else { Cell::Burrow };
                }
            }
        }
    }

    fn lesson_burrow_open(&self) -> bool {
        self.lesson.is_some_and(|l| LESSONS[l].steps.get(self.lesson_step).is_some_and(|s| s.1 == Trigger::Descend))
    }

    fn start_burrow(&mut self, b: u32, out: &mut Out) {
        self.lesson = None;
        self.burrow = b;
        self.stats.deepest = self.stats.deepest.max(b);
        self.floor = gen::generate(b, &mut self.rng, &self.carry);
        self.phase = Phase::Burrow;
        self.card = CARD_FRAMES;
        self.focus = self.floor.player;
        self.floor_turn0 = self.stats.turns;
        self.floor_frames = 0;
        let names: Vec<&str> = self.floor.creatures.iter().map(|c| c.kind.name()).collect();
        out.log(format!(
            "--- burrow {b} ({}): hearts {}/{}, dew {}, shape {}, creatures [{}], spawns every {}",
            self.area_name().to_lowercase(),
            self.floor.hearts,
            self.floor.max_hearts,
            self.floor.dew_total,
            self.floor.form.name().to_lowercase(),
            names.join(", ").to_lowercase(),
            self.floor.spawn_every
        ));
        out.music = Some(self.track());
        self.message = None;
        if self.floor.seal > 0 {
            let s = format!("ROOTS SEAL THIS BURROW. IT OPENS ONCE YOU'VE CAUGHT {} CREATURES HERE.", self.floor.seal);
            self.tip("seal", &s);
        }
        if b == BURROWS {
            self.tip("grof", "GROF, THE MALEVOLENT SORCERER. CATCH HIM THREE TIMES. POINT AT HIM TO SEE HOW HE MOVES.");
        }
        self.refresh();
    }

    fn tip(&mut self, id: &'static str, text: &str) {
        if !self.tips_shown.contains(&id) {
            self.tips_shown.push(id);
            self.message = Some(text.to_string());
        }
    }

    /// Recomputes the lit squares' exact outcomes and the red marks.
    pub fn refresh(&mut self) {
        let f = &self.floor;
        self.previews = f.targets().into_iter().map(|t| (t, f.preview(Action::Move(t)))).collect();
        self.rest_preview = f.preview(Action::Rest);
        self.threatened = f.threatened();
    }

    pub fn board_square(&self, x: i32, y: i32) -> Option<Pos> {
        let (gx, gy) = ((x - BX).div_euclid(TILE), (y - BY).div_euclid(TILE));
        (x >= BX && y >= BY && inb((gx, gy))).then_some((gx, gy))
    }

    pub fn pointed(&self) -> Option<Pos> {
        self.board_square(self.mouse.0, self.mouse.1).or(Some(self.focus))
    }

    fn try_act(&mut self, a: Action, out: &mut Out) {
        if self.card > 0 || self.ended || !matches!(self.phase, Phase::Lesson | Phase::Burrow) {
            return;
        }
        if self.anim.is_some() {
            self.queued = Some(a);
            return;
        }
        if let Action::Move(p) = a {
            if !self.previews.iter().any(|(t, _)| *t == p) {
                out.sfx(Sfx::Deny);
                return;
            }
        }
        let prev = self.floor.clone();
        let ev_player = self.floor.apply_player(a);
        let mid = self.floor.clone();
        let ev_enemy = if self.floor.over.is_none() && !self.floor.pouncing { self.floor.enemy_phase() } else { Vec::new() };
        self.after_action(a, &prev, &ev_player, &ev_enemy, out);
        self.anim = Some(Anim { prev, mid, ev_player, ev_enemy, t: 0 });
    }

    fn after_action(&mut self, a: Action, prev: &Floor, evp: &[Event], eve: &[Event], out: &mut Out) {
        let secs = self.think as f32 / 15.0;
        self.think = 0;
        self.stats.turns += 1;
        if self.lesson.is_some() {
            self.lesson_moves += 1;
        }
        let shape = prev.form.name().to_lowercase();
        let mut parts: Vec<String> = Vec::new();
        match a {
            Action::Rest => {
                if prev.pouncing {
                    parts.push("end chain".into());
                } else {
                    parts.push(format!("rest (was {shape})"));
                    self.stats.rests += 1;
                    self.lesson_rested = true;
                }
            }
            Action::Move(p) => parts.push(format!("{shape} {}->{}", sq(prev.player), sq(p))),
        }
        for e in evp.iter().chain(eve) {
            match e {
                Event::Caught { kind, dew, .. } => {
                    parts.push(format!("CATCH {} +{dew} dew", kind.name().to_lowercase()));
                    self.stats.catches += 1;
                    self.lesson_caught = true;
                    self.chain_len += 1;
                    self.stats_best_chain = self.stats_best_chain.max(self.chain_len);
                    if self.chain_len >= 2 {
                        self.lesson_chained = true;
                    }
                    out.sfx(Sfx::Catch);
                }
                Event::Dew { .. } => {
                    parts.push("dew +1".into());
                    out.sfx(Sfx::Dew);
                }
                Event::Rested => out.sfx(Sfx::Rest),
                Event::Toxic => {
                    parts.push("toxic -1 heart".into());
                    self.stats.hits += 1;
                }
                Event::Pounce => parts.push(format!("chain x{}", self.chain_len)),
                Event::Strike { kind, .. } => {
                    parts.push(format!("STRUCK by {}", kind.name().to_lowercase()));
                    self.stats.hits += 1;
                    self.stats.struck_by.push(*kind);
                }
                Event::Blocked { .. } => parts.push("shell blocked a strike".into()),
                Event::Rustle { kind, at } => {
                    parts.push(format!("rustle {} at {}", kind.name().to_lowercase(), sq(*at)));
                    out.sfx(Sfx::Rustle);
                    self.tip("rustle", "SOMETHING RUSTLES AT THE EDGE. A NEW CREATURE ARRIVES THERE IN 2 TURNS. POINT AT IT TO SEE WHAT.");
                }
                Event::Ward { left, .. } => {
                    parts.push(format!("grof loses a ward ({left} left)"));
                    out.sfx(Sfx::Ward);
                }
                Event::Opened => {
                    parts.push("burrow opens".into());
                    out.sfx(Sfx::Lesson);
                    self.message = Some("THE BURROW OPENS. GO DOWN WHENEVER YOU LIKE.".into());
                }
                Event::Descended => {}
                Event::Died => parts.push("DIED".into()),
                Event::Won => parts.push("WON".into()),
                _ => {}
            }
        }
        if !eve.is_empty() || !self.floor.pouncing {
            self.chain_len = 0;
        }
        parts.push(format!("hearts {}/{} dew {}", self.floor.hearts, self.floor.max_hearts, self.floor.dew_total));
        let tag = match self.lesson {
            Some(l) => format!("L{}", l + 1),
            None => format!("B{}", self.burrow),
        };
        out.log(format!("{tag} t{} ({secs:.1}s) {}", self.floor.turn, parts.join(" | ")));
        // First sightings of special rules.
        let kinds: Vec<Kind> = self.floor.creatures.iter().map(|c| c.kind).collect();
        for k in kinds {
            match k {
                Kind::Heron if self.lesson.is_none() => self.tips_shown_push_msg("heron", "HERONS ARE WARY: THEY WON'T STOP WHERE YOU COULD CATCH THEM."),
                Kind::Glowtoad => self.tips_shown_push_msg("glow", "GLOWTOADS ARE TOXIC: CATCHING ONE COSTS YOU A HEART."),
                _ => {}
            }
        }
        if self.lesson.is_some() {
            self.check_lesson(out);
        }
    }

    fn tips_shown_push_msg(&mut self, id: &'static str, text: &'static str) {
        if !self.tips_shown.contains(&id) {
            self.tips_shown.push(id);
            self.message = Some(text.to_string());
        }
    }

    fn check_lesson(&mut self, out: &mut Out) {
        let Some(l) = self.lesson else { return };
        let Some(&(_, trig)) = LESSONS[l].steps.get(self.lesson_step) else { return };
        let done = match trig {
            Trigger::Moves(n) => self.lesson_moves >= n,
            Trigger::Catch => self.lesson_caught,
            Trigger::Rest => self.lesson_rested && self.floor.form == Kind::Wanderer,
            Trigger::PointAtCreature => false,
            Trigger::Chain => self.lesson_chained || self.floor.creatures.len() <= 1,
            Trigger::Descend => false,
        };
        if done {
            self.advance_lesson(out);
        }
    }

    fn advance_lesson(&mut self, out: &mut Out) {
        self.lesson_step += 1;
        self.lesson_moves = 0;
        self.lesson_rested = false;
        out.sfx(Sfx::Lesson);
        if self.lesson_burrow_open() {
            self.close_lesson_burrow(false);
            self.refresh();
        }
    }

    /// Called once the step animation has finished.
    fn settle(&mut self, out: &mut Out) {
        let Some(anim) = self.anim.take() else { return };
        for e in &anim.ev_enemy {
            match e {
                Event::Strike { .. } => {
                    out.sfx(Sfx::Hurt);
                    self.hurt_flash = 3;
                }
                Event::Blocked { .. } => out.sfx(Sfx::Block),
                Event::Arrived { .. } => out.sfx(Sfx::Arrive),
                _ => {}
            }
        }
        if anim.ev_player.contains(&Event::Toxic) {
            out.sfx(Sfx::Hurt);
            self.hurt_flash = 3;
        }
        match self.floor.over {
            Some(false) => {
                if self.lesson.is_some() {
                    out.log("lesson: struck down, board restarts");
                    let l = self.lesson.unwrap();
                    let step = self.lesson_step;
                    self.start_lesson(l, out);
                    self.lesson_step = step;
                    if self.lesson_burrow_open() {
                        self.close_lesson_burrow(false);
                    }
                    self.message = Some("DOAT PULLS YOU BACK OUT. TRY THAT BOARD AGAIN.".into());
                    self.refresh();
                    return;
                }
                out.sfx(Sfx::Die);
                let by: Vec<String> = anim.ev_enemy.iter().filter_map(|e| if let Event::Strike { kind, .. } = e { Some(kind.name().to_lowercase()) } else { None }).collect();
                let cause = if by.is_empty() { "a toxic catch".to_string() } else { format!("struck by {}", by.join(", ")) };
                out.log(format!("DIED on burrow {} turn {} ({cause})", self.burrow, self.floor.turn));
                self.carry.take(&self.floor);
                self.phase = Phase::Over { won: false };
                self.message = None;
            }
            Some(true) if self.floor.won => {
                out.sfx(Sfx::Win);
                out.log(format!("WON: grof caught on turn {}", self.floor.turn));
                self.carry.take(&self.floor);
                self.phase = Phase::Over { won: true };
                out.music = Some(Track::AncientMagic);
            }
            Some(true) => {
                out.sfx(Sfx::Descend);
                self.finish_floor(out);
            }
            None => {
                self.refresh();
                if let Some(q) = self.queued.take() {
                    self.try_act(q, out);
                }
            }
        }
    }

    fn finish_floor(&mut self, out: &mut Out) {
        let hurt = self.floor.struck;
        self.carry.take(&self.floor);
        if let Some(l) = self.lesson {
            out.log(format!("lesson {} done in {} moves", l + 1, self.floor.turn));
            if l + 1 < LESSONS.len() {
                self.start_lesson(l + 1, out);
            } else {
                self.carry.hearts = self.carry.max_hearts;
                self.carry.form = Kind::Wanderer;
                self.carry.prev_form = None;
                self.open_glade(true, out);
            }
            return;
        }
        if !hurt && self.carry.charms.contains(&Charm::SecondWind) && self.carry.hearts < self.carry.max_hearts {
            self.carry.hearts += 1;
            out.log("second wind: +1 heart");
        }
        out.log(format!(
            "burrow {} done: {} turns, {}s, hearts {}/{}, dew {}",
            self.burrow,
            self.stats.turns - self.floor_turn0,
            self.floor_frames / 15,
            self.carry.hearts,
            self.carry.max_hearts,
            self.carry.dew
        ));
        self.open_glade(false, out);
    }

    fn open_glade(&mut self, first: bool, out: &mut Out) {
        let mut pool: Vec<Charm> = ALL_CHARMS.iter().copied().filter(|c| !self.carry.charms.contains(c)).collect();
        self.rng.shuffle(&mut pool);
        pool.truncate(3);
        out.log(format!("glade: offers [{}], dew {}", pool.iter().map(|c| c.name().to_lowercase()).collect::<Vec<_>>().join(", "), self.carry.dew));
        self.phase = Phase::Glade { offers: pool, first };
        self.glade_hover = None;
    }

    pub fn glade_items(&self) -> Vec<Option<Charm>> {
        match &self.phase {
            Phase::Glade { offers, .. } => {
                let mut v: Vec<Option<Charm>> = offers.iter().map(|c| Some(*c)).collect();
                v.push(None); // mend
                v
            }
            _ => Vec::new(),
        }
    }

    pub fn glade_cost(&self, item: Option<Charm>) -> i32 {
        item.map_or(MEND_COST, |c| c.cost())
    }

    pub fn glade_can_buy(&self, item: Option<Charm>) -> bool {
        let cost = self.glade_cost(item);
        match item {
            Some(c) => self.carry.dew >= cost && !self.carry.charms.contains(&c),
            None => self.carry.dew >= cost && self.carry.hearts < self.carry.max_hearts,
        }
    }

    fn glade_buy(&mut self, i: usize, out: &mut Out) {
        let items = self.glade_items();
        let Some(&item) = items.get(i) else { return };
        if !self.glade_can_buy(item) {
            out.sfx(Sfx::Deny);
            return;
        }
        self.carry.dew -= self.glade_cost(item);
        match item {
            Some(c) => {
                self.carry.charms.push(c);
                if c == Charm::ThickSkin {
                    self.carry.max_hearts += 1;
                    self.carry.hearts += 1;
                }
                self.bought.push(c.name().to_string());
                out.log(format!("glade: bought {} ({} dew left)", c.name().to_lowercase(), self.carry.dew));
            }
            None => {
                self.carry.hearts += 1;
                out.log(format!("glade: mended a heart ({} dew left)", self.carry.dew));
            }
        }
        out.sfx(Sfx::Buy);
    }

    fn leave_glade(&mut self, out: &mut Out) {
        out.sfx(Sfx::Click);
        let next = self.burrow + 1;
        self.start_burrow(next, out);
    }

    fn on_point(&mut self, out: &mut Out) {
        if self.lesson.is_some() {
            if let Some(&(_, Trigger::PointAtCreature)) = self.lesson.and_then(|l| LESSONS[l].steps.get(self.lesson_step)) {
                if let Some(p) = self.board_square(self.mouse.0, self.mouse.1).or(Some(self.focus)) {
                    if self.floor.creature_at(p).is_some() {
                        self.advance_lesson(out);
                    }
                }
            }
        }
    }

    fn click_square(&mut self, p: Pos, out: &mut Out) {
        if p == self.floor.player {
            self.try_act(Action::Rest, out);
        } else {
            self.try_act(Action::Move(p), out);
        }
    }
}

pub const GLADE_ITEM_Y: i32 = 92;
pub const GLADE_ITEM_H: i32 = 34;
pub fn glade_item_rect(i: usize) -> crate::ui::Rect {
    crate::ui::Rect::new(20, GLADE_ITEM_Y + i as i32 * (GLADE_ITEM_H + 4), 360, GLADE_ITEM_H)
}
pub const GLADE_GO: crate::ui::Rect = crate::ui::Rect::new(250, 268, 130, 20);
pub const OVER_GO: crate::ui::Rect = crate::ui::Rect::new(130, 250, 140, 20);

impl Run for Game {
    fn event(&mut self, ev: Ev, out: &mut Out) {
        if self.show_help {
            if matches!(ev, Ev::Key(_) | Ev::Press { .. }) {
                self.show_help = false;
                out.sfx(Sfx::Click);
            }
            return;
        }
        if let Ev::Key(Key::Help) = ev {
            self.show_help = true;
            out.sfx(Sfx::Click);
            return;
        }
        match self.phase.clone() {
            Phase::Lesson | Phase::Burrow => match ev {
                Ev::Move(x, y) => {
                    self.mouse = (x, y);
                    if let Some(p) = self.board_square(x, y) {
                        self.focus = p;
                        self.on_point(out);
                    }
                }
                Ev::Press { x, y, right: false } => {
                    if let Some(p) = self.board_square(x, y) {
                        self.focus = p;
                        self.click_square(p, out);
                    }
                }
                Ev::Key(k) => {
                    let d = match k {
                        Key::Up => Some((0, -1)),
                        Key::Down => Some((0, 1)),
                        Key::Left => Some((-1, 0)),
                        Key::Right => Some((1, 0)),
                        _ => None,
                    };
                    if let Some((dx, dy)) = d {
                        self.focus = ((self.focus.0 + dx).clamp(0, N - 1), (self.focus.1 + dy).clamp(0, N - 1));
                        self.mouse = (-1, -1);
                        self.on_point(out);
                    }
                    match k {
                        Key::Enter | Key::Space => {
                            let p = self.focus;
                            self.click_square(p, out);
                        }
                        Key::Rest => self.try_act(Action::Rest, out),
                        _ => {}
                    }
                }
                _ => {}
            },
            Phase::Glade { .. } => {
                let n = self.glade_items().len();
                match ev {
                    Ev::Move(x, y) => {
                        self.mouse = (x, y);
                        self.glade_hover = (0..n).find(|&i| glade_item_rect(i).contains(x, y)).or(GLADE_GO.contains(x, y).then_some(n));
                    }
                    Ev::Press { x, y, right: false } => {
                        if GLADE_GO.contains(x, y) {
                            self.leave_glade(out);
                        } else if let Some(i) = (0..n).find(|&i| glade_item_rect(i).contains(x, y)) {
                            self.glade_buy(i, out);
                        }
                    }
                    Ev::Key(Key::Up) => self.glade_hover = Some(self.glade_hover.map_or(0, |h| (h + n) % (n + 1))),
                    Ev::Key(Key::Down) | Ev::Key(Key::Tab) => self.glade_hover = Some(self.glade_hover.map_or(0, |h| (h + 1) % (n + 1))),
                    Ev::Key(Key::Enter) | Ev::Key(Key::Space) => match self.glade_hover {
                        Some(h) if h < n => self.glade_buy(h, out),
                        Some(_) => self.leave_glade(out),
                        None => self.glade_hover = Some(0),
                    },
                    _ => {}
                }
            }
            Phase::Over { .. } => match ev {
                Ev::Move(x, y) => self.mouse = (x, y),
                Ev::Press { x, y, right: false } if OVER_GO.contains(x, y) => {
                    out.sfx(Sfx::Click);
                    self.ended = true;
                }
                Ev::Key(Key::Enter) | Ev::Key(Key::Space) => {
                    out.sfx(Sfx::Click);
                    self.ended = true;
                }
                _ => {}
            },
        }
    }

    fn tick(&mut self, out: &mut Out) {
        self.frame += 1;
        if out.music.is_none() && self.frame == 1 {
            out.music = Some(self.track());
        }
        if self.card > 0 {
            self.card -= 1;
        }
        if matches!(self.phase, Phase::Lesson | Phase::Burrow) && self.card == 0 && self.anim.is_none() {
            self.think += 1;
            self.floor_frames += 1;
        }
        if self.hurt_flash > 0 {
            self.hurt_flash -= 1;
        }
        for p in self.particles.iter_mut() {
            p.x += p.dx;
            p.y += p.dy;
            p.life -= 1;
        }
        self.particles.retain(|p| p.life > 0);
        let mut done = false;
        if let Some(a) = self.anim.as_mut() {
            a.t += 1;
            if a.t == PH1 {
                // The player has landed: sparkle catches and dew.
                let evs = a.ev_player.clone();
                for e in evs {
                    let (at, col) = match e {
                        Event::Caught { at, .. } => (at, art::SPIRIT + 2),
                        Event::Dew { at } => (at, art::DEW + 1),
                        _ => continue,
                    };
                    let (cx, cy) = (BX + at.0 * TILE + TILE / 2, BY + at.1 * TILE + TILE / 2);
                    for (dx, dy) in [(2, 0), (-2, 0), (0, 2), (0, -2), (1, 1), (-1, 1), (1, -1), (-1, -1)] {
                        self.particles.push(Particle { x: cx, y: cy, dx, dy, life: 5, col });
                    }
                }
                if a.ev_enemy.iter().any(|e| matches!(e, Event::Crept { .. })) {
                    out.sfx(Sfx::Hop);
                }
                if a.ev_enemy.iter().any(|e| matches!(e, Event::Strike { .. })) {
                    out.sfx(Sfx::Strike);
                }
            }
            if a.t == 1 {
                out.sfx(Sfx::Step);
            }
            if a.t >= a.len() {
                done = true;
            }
        }
        if done {
            self.settle(out);
        }
    }

    fn draw(&self, fb: &mut Framebuffer) {
        render::draw(self, fb);
    }

    fn palette(&self) -> &Pal {
        &self.pal
    }

    fn ended(&self) -> bool {
        self.ended
    }

    fn summary(&self) -> Summary {
        let headline = match &self.phase {
            Phase::Over { won: true } => "GROF IS CAUGHT. THE FOREST IS QUIET.".to_string(),
            _ if self.lesson.is_some() => format!("STOPPED DURING DOAT'S LESSON {}.", self.lesson.unwrap() + 1),
            Phase::Over { won: false } => format!("FELL IN {}, BURROW {} OF {BURROWS}.", self.area_name(), self.burrow),
            _ => format!("STOPPED IN {}, BURROW {} OF {BURROWS}.", self.area_name(), self.burrow),
        };
        let charms = if self.carry.charms.is_empty() {
            "NO CHARMS".to_string()
        } else {
            format!("CHARMS: {}", self.carry.charms.iter().map(|c| c.name()).collect::<Vec<_>>().join(", "))
        };
        let detail = format!(
            "{} CATCHES (LONGEST CHAIN {}), STRUCK {} TIMES, {} TURNS. {charms}.",
            self.stats.catches, self.stats_best_chain, self.stats.hits, self.stats.turns
        );
        let won = matches!(self.phase, Phase::Over { won: true });
        Summary { headline, detail, best_key: self.stats.deepest + if won { 100 } else { 0 } }
    }

    fn takes_esc(&self) -> bool {
        self.show_help
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A cautious bot: catch when safe, otherwise step to the safest square nearer the burrow.
    fn bot_move(g: &Game) -> Action {
        let f = &g.floor;
        if f.pouncing {
            let best = g.previews.iter().filter(|(_, p)| !p.dies).min_by_key(|(_, p)| p.hits * 10 + if p.toxic { 5 } else { 0 });
            return match best {
                Some((t, p)) if p.hits <= g.rest_preview.hits => Action::Move(*t),
                _ => Action::Rest,
            };
        }
        let burrow = (0..N).flat_map(|y| (0..N).map(move |x| (x, y))).find(|&p| f.cell(p) == Cell::Burrow).filter(|_| f.seal == 0);
        let nearest = f.creatures.iter().min_by_key(|c| (c.kind != Kind::Grof, cheb(c.pos, f.player))).map(|c| c.pos);
        let goal = burrow.or(nearest).unwrap_or(f.player);
        let mut best: Option<((i32, i32, i32), Action)> = None;
        for (t, p) in &g.previews {
            // A catch that opens a chain is judged by where the chain can end, which the bot
            // treats optimistically.
            let hits = if p.pounce && !p.dies { 0 } else { p.hits };
            let score = (hits * 10 + if p.toxic { 5 } else { 0 }, if p.catch.is_some() || p.wins { -1 } else { 0 }, cheb(*t, goal));
            if best.as_ref().map_or(true, |b| score < b.0) {
                best = Some((score, Action::Move(*t)));
            }
        }
        let rest = (g.rest_preview.hits * 10, 1, 99);
        if let Some((s, _)) = &best {
            let stuck = f.turn % 9 == 8;
            if s.1 == 0 && s.2 >= cheb(f.player, goal) && f.form != Kind::Wanderer && (g.rest_preview.hits <= s.0 / 10 || stuck) {
                return Action::Rest;
            }
        }
        match best {
            Some((s, a)) if s <= rest => a,
            _ => Action::Rest,
        }
    }

    fn drive(g: &mut Game, out: &mut Out, max: usize) {
        for _ in 0..max {
            if g.ended {
                return;
            }
            match g.phase.clone() {
                Phase::Lesson | Phase::Burrow => {
                    while g.card > 0 || g.anim.is_some() {
                        g.tick(out);
                    }
                    if g.lesson.is_some() && LESSONS[g.lesson.unwrap()].steps.get(g.lesson_step).is_some_and(|s| s.1 == Trigger::PointAtCreature) {
                        let c = g.floor.creatures[0].pos;
                        g.event(Ev::Move(BX + c.0 * TILE + 5, BY + c.1 * TILE + 5), out);
                        continue;
                    }
                    let a = bot_move(g);
                    // Lesson 2 wants a rest after the catch.
                    let a = if g.lesson == Some(1) && g.lesson_step == 1 && !g.floor.pouncing { Action::Rest } else { a };
                    g.try_act(a, out);
                }
                Phase::Glade { .. } => {
                    for i in 0..4 {
                        g.glade_buy(i, out);
                    }
                    g.leave_glade(out);
                }
                Phase::Over { .. } => g.ended = true,
            }
        }
    }

    #[test]
    fn a_bot_gets_through_the_lessons_and_into_the_run() {
        let mut g = Game::new(7, true);
        let mut out = Out::default();
        drive(&mut g, &mut out, 400);
        assert!(g.burrow >= 1 || g.ended, "stuck in lessons at {:?} step {}: {:?}", g.lesson, g.lesson_step, (&out.log[out.log.len().saturating_sub(3)..], g.floor.creatures.iter().map(|c| (c.kind, c.pos, c.intent)).collect::<Vec<_>>(), g.floor.player));
        assert!(out.log.iter().any(|l| l.contains("lesson 4 done")), "{:?}", out.log);
    }

    #[test]
    fn bots_play_whole_runs_without_panicking() {
        let mut deepest = Vec::new();
        for seed in 0..30 {
            let mut g = Game::new(seed, false);
            let mut out = Out::default();
            drive(&mut g, &mut out, 3000);
            deepest.push((g.stats.deepest, matches!(g.phase, Phase::Over { won: true }), g.stats.catches, g.stats.hits, g.stats.turns));
            assert!(g.ended || matches!(g.phase, Phase::Over { .. }) || g.stats.turns > 0);
        }
        println!("bot deepest burrows: {deepest:?}");
    }
}
