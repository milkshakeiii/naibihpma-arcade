//! GULP: a frog in the forest of Naibihpma. Hop, tongue, spit, croak; eat to grow.

pub mod art;
pub mod gen;
mod render;
pub mod rules;

use crate::app::{Ev as In, Key, Out, Run, Summary};
use crate::audio::{Sfx, Track};
use crate::form::Question;
use crate::framebuffer::Framebuffer;
use crate::palette::Pal;
use crate::rng::Rng;
use crate::ui::Rect;
use gen::{Trigger, AREAS, LESSONS, PONDS};
use rules::*;

pub const VERSION: u32 = 1;
pub const LESSON_VERSION: u32 = 1;
pub static FORM_QUESTION: Question = Question { id: "verbs", text: "TONGUE, SPIT AND CROAK FELT...", low: "FIDDLY", high: "DELICIOUS" };

pub const TILE: i32 = 28;
pub const BX: i32 = 8;
pub const BY: i32 = 20;
pub const PX: i32 = 238;
pub const PW: i32 = 158;
const CARD_FRAMES: u32 = 20;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Mode {
    Hop,
    Tongue,
    Spit,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Button {
    Tongue,
    Spit,
    Gulp,
    Croak,
    End,
}

pub const BUTTONS: [(Button, &str); 5] = [(Button::Tongue, "T TONGUE"), (Button::Spit, "S SPIT"), (Button::Gulp, "G GULP"), (Button::Croak, "C CROAK"), (Button::End, "END TURN")];

pub fn button_rect(i: usize) -> Rect {
    if i == 4 {
        Rect::new(PX, 220, PW, 22)
    } else {
        Rect::new(PX + (i % 2) as i32 * 80, 172 + (i / 2) as i32 * 24, 78, 22)
    }
}

pub fn mutation_rect(i: usize) -> Rect {
    Rect::new(30, 84 + i as i32 * 52, 340, 46)
}

pub const OVER_GO: Rect = Rect::new(140, 262, 120, 22);

#[derive(Clone, Debug, PartialEq)]
pub enum Phase {
    Lesson,
    Pond,
    Bank { offers: Vec<Option<Mutation>> },
    Over { won: bool },
}

pub struct Anim {
    pub prev: Pond,
    pub mid: Pond,
    pub mine: Vec<Ev>,
    pub theirs: Vec<Ev>,
    pub t: u32,
}

impl Anim {
    pub const MINE: u32 = 4;
    pub const THEIRS: u32 = 5;
    pub fn len(&self) -> u32 {
        Self::MINE + if self.theirs.is_empty() { 0 } else { Self::THEIRS }
    }
}

#[derive(Default, Clone)]
pub struct Stats {
    pub turns: u32,
    pub hurt: i32,
    pub eaten: u32,
    pub defeated: u32,
    pub ponds: u32,
    pub eggs_lost: u32,
    pub eggs_saved: u32,
}

pub struct Game {
    pub phase: Phase,
    pub pond: Pond,
    pub frog: Frog,
    pub pond_no: u32,
    pub lesson: Option<usize>,
    pub lesson_step: usize,
    pub lesson_turns: u32,
    pub flags: Vec<Trigger>,
    pub mode: Mode,
    pub mouse: (i32, i32),
    pub rng: Rng,
    pub art: art::Art,
    pub pal: Pal,
    pub message: Option<String>,
    pub tips: Vec<&'static str>,
    pub show_help: bool,
    pub ended: bool,
    pub stats: Stats,
    pub think: u32,
    pub frame: u32,
    pub card: u32,
    pub anim: Option<Anim>,
    pub hurt_flash: u32,
    pub hover_item: Option<usize>,
}

impl Game {
    pub fn new(seed: u64, lessons: bool) -> Self {
        let mut g = Self {
            phase: Phase::Pond,
            pond: Pond::empty(Rng::new(1)),
            frog: Frog::new(),
            pond_no: 0,
            lesson: None,
            lesson_step: 0,
            lesson_turns: 0,
            flags: Vec::new(),
            mode: Mode::Hop,
            mouse: (-1, -1),
            rng: Rng::new(seed),
            art: art::Art::new(),
            pal: art::palette(),
            message: None,
            tips: Vec::new(),
            show_help: false,
            ended: false,
            stats: Stats::default(),
            think: 0,
            frame: 0,
            card: 0,
            anim: None,
            hurt_flash: 0,
            hover_item: None,
        };
        let mut out = Out::default();
        if lessons {
            g.start_lesson(0, &mut out);
        } else {
            g.start_pond(1, &mut out);
        }
        g
    }

    pub fn area_name(&self) -> &'static str {
        if self.lesson.is_some() {
            "LESSONS"
        } else {
            AREAS[gen::area_of(self.pond_no)].name
        }
    }

    pub fn track(&self) -> Track {
        match (&self.phase, self.lesson) {
            (Phase::Over { won: true }, _) => Track::AncientMagic,
            (_, Some(_)) => Track::Underbrush,
            _ if self.pond_no == 0 => Track::Underbrush,
            _ => AREAS[gen::area_of(self.pond_no)].track,
        }
    }

    fn tip(&mut self, id: &'static str, text: &str) {
        if !self.tips.contains(&id) {
            self.tips.push(id);
            self.message = Some(text.to_string());
        }
    }

    fn start_lesson(&mut self, i: usize, out: &mut Out) {
        self.lesson = Some(i);
        self.lesson_step = 0;
        self.lesson_turns = 0;
        self.flags.clear();
        let mut f = self.frog.clone();
        if i == 0 {
            f = Frog::new();
        }
        f.hearts = f.max_hearts;
        self.pond = gen::lesson_pond(i, &f);
        self.pond.exit_locked = true;
        self.phase = Phase::Lesson;
        self.card = CARD_FRAMES;
        self.mode = Mode::Hop;
        self.message = None;
        out.log(format!("lesson {}: {}", i + 1, LESSONS[i].title.to_lowercase()));
        out.music = Some(Track::Underbrush);
    }

    fn start_pond(&mut self, n: u32, out: &mut Out) {
        self.lesson = None;
        self.pond_no = n;
        self.pond = gen::generate(n, &mut self.rng, &self.frog);
        self.phase = Phase::Pond;
        self.card = CARD_FRAMES;
        self.mode = Mode::Hop;
        self.message = None;
        let names: Vec<String> = self.pond.creatures.iter().filter(|c| c.kind != Kind::Fly).map(|c| c.kind.info().name.to_lowercase()).collect();
        let waves: Vec<String> = self.pond.waves.iter().map(|w| w.1.info().name.to_lowercase()).collect();
        out.log(format!(
            "--- pond {n} ({}): frog {} {}/{} hearts, mutations [{}], creatures [{}], later [{}]",
            self.area_name().to_lowercase(),
            self.frog.size_name().to_lowercase(),
            self.frog.hearts,
            self.frog.max_hearts,
            self.frog.muts.iter().map(|m| m.name().to_lowercase()).collect::<Vec<_>>().join(", "),
            names.join(", "),
            waves.join(", ")
        ));
        out.music = Some(self.track());
        if n == PONDS {
            self.message = Some("GROF. HE'S TOO BIG TO SWALLOW. SPIT, SHOVE AND STEER STRIKES INTO HIM.".into());
        } else if n == 1 {
            self.tip("pond1", "CLEAR THE POND OF PREDATORS (FLIES DON'T COUNT), THEN HOP TO THE EXIT. KEEP YOUR EGGS SAFE: EACH CLUTCH THAT SURVIVES HEALS YOU 1.");
        }
    }

    pub fn board_square(&self, x: i32, y: i32) -> Option<Pos> {
        let (gx, gy) = ((x - BX).div_euclid(TILE), (y - BY).div_euclid(TILE));
        (x >= BX && y >= BY && inb((gx, gy))).then_some((gx, gy))
    }

    /// The direction from the frog to `p`, if `p` lies on one of its 8 lines.
    pub fn line_dir(&self, p: Pos) -> Option<Pos> {
        let f = self.pond.frog.pos;
        let (dx, dy) = (p.0 - f.0, p.1 - f.1);
        if (dx, dy) == (0, 0) || !(dx == 0 || dy == 0 || dx.abs() == dy.abs()) {
            return None;
        }
        Some((dx.signum(), dy.signum()))
    }

    pub fn can(&self, b: Button) -> bool {
        let f = &self.pond.frog;
        let free = self.anim.is_none() && self.card == 0 && matches!(self.phase, Phase::Lesson | Phase::Pond);
        free && match b {
            Button::Tongue => f.act_left,
            Button::Spit | Button::Gulp => f.act_left && !f.throat.is_empty(),
            Button::Croak => f.act_left && f.croak_cd == 0,
            Button::End => true,
        }
    }

    fn press(&mut self, b: Button, out: &mut Out) {
        if !self.can(b) {
            out.sfx(Sfx::Deny);
            return;
        }
        out.sfx(Sfx::Click);
        match b {
            Button::Tongue => self.mode = if self.mode == Mode::Tongue { Mode::Hop } else { Mode::Tongue },
            Button::Spit => self.mode = if self.mode == Mode::Spit { Mode::Hop } else { Mode::Spit },
            Button::Gulp => self.do_act(Act::Gulp, out),
            Button::Croak => self.do_act(Act::Croak, out),
            Button::End => self.do_act(Act::End, out),
        }
    }

    fn click_square(&mut self, p: Pos, right: bool, out: &mut Out) {
        if self.anim.is_some() || self.card > 0 {
            return;
        }
        if p == self.pond.frog.pos {
            self.do_act(Act::End, out);
            return;
        }
        let dir = self.line_dir(p);
        let wants_line = right || matches!(self.mode, Mode::Tongue | Mode::Spit);
        if let (true, Some(d)) = (wants_line, dir) {
            if self.pond.frog.act_left {
                let a = if self.mode == Mode::Spit && !right { Act::Spit(d) } else { Act::Tongue(d) };
                if matches!(a, Act::Spit(_)) && self.pond.frog.throat.is_empty() {
                    out.sfx(Sfx::Deny);
                    return;
                }
                self.do_act(a, out);
                return;
            }
        }
        if self.pond.hop_targets().contains(&p) {
            self.do_act(Act::Hop(p), out);
        } else {
            out.sfx(Sfx::Deny);
        }
    }

    fn do_act(&mut self, a: Act, out: &mut Out) {
        let prev = self.pond.clone();
        let (mine, done) = self.pond.player_act(a);
        if mine.is_empty() && !done && a != Act::End {
            out.sfx(Sfx::Deny);
            return;
        }
        let mid = self.pond.clone();
        let turn_over = done && self.pond.over.is_none();
        let theirs = if turn_over { self.pond.creatures_turn() } else { Vec::new() };
        self.after(a, &prev, &mine, &theirs, turn_over, out);
        self.anim = Some(Anim { prev, mid, mine, theirs, t: 0 });
        self.mode = Mode::Hop;
    }

    fn after(&mut self, a: Act, prev: &Pond, mine: &[Ev], theirs: &[Ev], turn_over: bool, out: &mut Out) {
        let secs = self.think as f32 / 15.0;
        self.think = 0;
        let mut parts = vec![match a {
            Act::Hop(p) => format!("hop {}->{}", sq(prev.frog.pos), sq(p)),
            Act::Tongue(d) => format!("tongue {:?}", d),
            Act::Spit(d) => format!("spit {:?}", d),
            Act::Gulp => "gulp".into(),
            Act::Croak => "croak".into(),
            Act::End => "end turn".into(),
        }];
        for e in mine.iter().chain(theirs) {
            match e {
                Ev::Swallow { kind } => {
                    parts.push(format!("swallow {}", kind.info().name.to_lowercase()));
                    self.flags.push(Trigger::Swallow);
                }
                Ev::Yank { .. } => parts.push("yank".into()),
                Ev::Grapple { .. } => parts.push("grapple".into()),
                Ev::Spit { kind, .. } => {
                    parts.push(format!("spit {}", kind.info().name.to_lowercase()));
                    self.flags.push(Trigger::Spit);
                }
                Ev::Gulp { kind, grew } => {
                    parts.push(format!("digest {}{}", kind.info().name.to_lowercase(), if *grew { " GREW" } else { "" }));
                    self.stats.eaten += 1;
                    self.flags.push(Trigger::Gulp);
                    if *grew {
                        self.flags.push(Trigger::Grow);
                    }
                }
                Ev::Croak => self.flags.push(Trigger::Croak),
                Ev::Hurt { dmg, by } => {
                    parts.push(format!("HURT {dmg} by {}", by.to_lowercase()));
                    self.stats.hurt += dmg;
                }
                Ev::Dodged => parts.push("camouflage dodge".into()),
                Ev::Died { kind, how, .. } => {
                    parts.push(format!("{} {}", kind.info().name.to_lowercase(), how.to_lowercase()));
                    if *kind != Kind::Fly {
                        self.stats.defeated += 1;
                    }
                }
                Ev::Ripple { kind, .. } => {
                    parts.push(format!("ripple {}", kind.info().name.to_lowercase()));
                    self.tip("ripple", "RIPPLES AT THE EDGE: A CREATURE ARRIVES THERE NEXT TURN. POINT AT THEM TO SEE WHAT.");
                }
                Ev::Cleared => {
                    parts.push("POND CLEAR".into());
                    self.flags.push(Trigger::Cleared);
                    if self.lesson.is_none() && self.pond_no < PONDS {
                        self.message = Some("THE POND IS CLEAR. EAT THE FLIES IF YOU LIKE, THEN HOP TO THE EXIT.".into());
                    }
                }
                Ev::Exited => self.flags.push(Trigger::Exit),
                Ev::EggLost { .. } => {
                    parts.push("EGGS LOST".into());
                    self.stats.eggs_lost += 1;
                    self.tip("eggs", "A CLUTCH OF YOUR EGGS WAS LOST, AND A HEART WITH IT. SHOVE, YANK OR KILL WHATEVER IS AIMING AT THEM.");
                }
                Ev::Hatched { clutches } => {
                    parts.push(format!("{clutches} clutch(es) hatched"));
                    self.stats.eggs_saved += *clutches as u32;
                }
                _ => {}
            }
        }
        if turn_over {
            self.stats.turns += 1;
            self.lesson_turns += 1;
            if self.lesson.is_some() {
                self.flags.push(Trigger::Turns(self.lesson_turns));
            }
        }
        let f = &self.pond.frog;
        parts.push(format!("{} {}/{} growth {} throat [{}]", f.size_name().to_lowercase(), f.hearts, f.max_hearts, f.growth, f.throat.iter().map(|k| k.info().name.to_lowercase()).collect::<Vec<_>>().join(",")));
        let tag = match self.lesson {
            Some(l) => format!("L{}", l + 1),
            None => format!("P{}", self.pond_no),
        };
        out.log(format!("{tag} t{} ({secs:.1}s) {}", self.pond.turn, parts.join(" | ")));
        self.check_lesson(out);
        // First sightings.
        let kinds: Vec<Kind> = self.pond.creatures.iter().map(|c| c.kind).collect();
        if self.lesson.is_none() {
            for k in kinds {
                let t = match k {
                    Kind::Dragonfly => Some(("dragonfly", "A DRAGONFLY: IT FLIES, SO WATER WON'T DROWN IT. IT DIVES TWO SQUARES DIAGONALLY.")),
                    Kind::Snake => Some(("snake", "A SNAKE: ITS LUNGE HITS TWO SQUARES IN A ROW, AND IT SWIMS.")),
                    Kind::Heron => Some(("heron", "A HERON: TOO BIG TO SWALLOW. YOUR TONGUE PULLS YOU TO IT INSTEAD.")),
                    Kind::Crayfish => Some(("crayfish", "A CRAYFISH: ARMORED AGAINST SPIT, AND IT PINCHES THREE SQUARES.")),
                    _ => None,
                };
                if let Some((id, text)) = t {
                    self.tip(id, text);
                }
            }
        }
    }

    fn check_lesson(&mut self, out: &mut Out) {
        let Some(l) = self.lesson else { return };
        loop {
            let Some(&(_, trig)) = LESSONS[l].steps.get(self.lesson_step) else { return };
            let done = match trig {
                Trigger::Turns(n) => self.lesson_turns >= n,
                Trigger::Cleared => self.pond.cleared,
                t => self.flags.contains(&t),
            };
            if !done {
                return;
            }
            self.lesson_step += 1;
            out.sfx(Sfx::Lesson);
            if LESSONS[l].steps.get(self.lesson_step).is_some_and(|s| s.1 == Trigger::Exit) {
                self.pond.exit_locked = false;
            }
        }
    }

    fn settle(&mut self, out: &mut Out) {
        self.anim = None;
        match self.pond.over {
            Some(false) => {
                if self.lesson.is_some() {
                    let l = self.lesson.unwrap();
                    out.log("lesson: knocked out, the pond restarts");
                    let step = self.lesson_step;
                    self.start_lesson(l, out);
                    self.lesson_step = step;
                    self.pond.exit_locked = !LESSONS[l].steps.get(step).is_some_and(|s| s.1 == Trigger::Exit);
                    self.message = Some("OUCH. TRY THAT POND AGAIN.".into());
                    return;
                }
                out.sfx(Sfx::Lose);
                out.log(format!("DIED in pond {} turn {}", self.pond_no, self.pond.turn));
                self.frog = self.pond.frog.clone();
                self.phase = Phase::Over { won: false };
            }
            Some(true) => {
                self.frog = self.pond.frog.clone();
                if let Some(l) = self.lesson {
                    out.log(format!("lesson {} done", l + 1));
                    if l + 1 < LESSONS.len() {
                        self.start_lesson(l + 1, out);
                    } else {
                        // The run proper starts from a fresh small frog, with one mutation.
                        self.frog = Frog::new();
                        self.open_bank(out);
                        self.tip("bank", "A MUTATION. PICK ONE: IT CHANGES YOU FOR THE WHOLE RUN.");
                    }
                    return;
                }
                out.sfx(Sfx::Clear);
                self.stats.ponds += 1;
                out.log(format!("pond {} done in {} turns", self.pond_no, self.pond.turn));
                self.open_bank(out);
            }
            None => {
                if self.pond_no == PONDS && self.lesson.is_none() && !self.pond.creatures.iter().any(|c| c.kind == Kind::Grof) {
                    out.sfx(Sfx::Win);
                    out.log(format!("WON: grof defeated on turn {}", self.pond.turn));
                    self.frog = self.pond.frog.clone();
                    self.phase = Phase::Over { won: true };
                    out.music = Some(Track::AncientMagic);
                }
            }
        }
    }

    fn open_bank(&mut self, out: &mut Out) {
        let mut pool: Vec<Mutation> = ALL_MUTATIONS.iter().copied().filter(|m| !self.frog.has(*m)).collect();
        self.rng.shuffle(&mut pool);
        let mut offers: Vec<Option<Mutation>> = pool.into_iter().take(3).map(Some).collect();
        if self.frog.hearts < self.frog.max_hearts {
            // Mend replaces one mutation when hurt.
            offers.truncate(2);
            offers.push(None);
        }
        out.log(format!("bank offers [{}]", offers.iter().map(|o| o.map_or("mend".to_string(), |m| m.name().to_lowercase())).collect::<Vec<_>>().join(", ")));
        self.phase = Phase::Bank { offers };
        self.hover_item = None;
    }

    fn pick(&mut self, i: usize, out: &mut Out) {
        let Phase::Bank { offers } = self.phase.clone() else { return };
        let Some(o) = offers.get(i) else { return };
        match o {
            Some(m) => {
                self.frog.muts.push(*m);
                out.log(format!("mutation: {}", m.name().to_lowercase()));
            }
            None => {
                self.frog.hearts = self.frog.max_hearts;
                out.log("mended to full");
            }
        }
        out.sfx(Sfx::Pick);
        let next = self.pond_no + 1;
        self.start_pond(next, out);
    }
}

impl Run for Game {
    fn event(&mut self, ev: In, out: &mut Out) {
        if let In::Move(x, y) | In::Press { x, y, .. } | In::Release { x, y, .. } = ev {
            self.mouse = (x, y);
        }
        if self.show_help {
            if matches!(ev, In::Key(_) | In::Press { .. }) {
                self.show_help = false;
                out.sfx(Sfx::Click);
            }
            return;
        }
        if ev == In::Key(Key::Help) {
            self.show_help = true;
            out.sfx(Sfx::Click);
            return;
        }
        match self.phase.clone() {
            Phase::Lesson | Phase::Pond => match ev {
                In::Press { x, y, right } => {
                    if let Some(p) = self.board_square(x, y) {
                        self.click_square(p, right, out);
                    } else if let Some(i) = (0..BUTTONS.len()).find(|&i| button_rect(i).contains(x, y)) {
                        self.press(BUTTONS[i].0, out);
                    }
                }
                In::Char(c) => match c.to_ascii_lowercase() {
                    't' => self.press(Button::Tongue, out),
                    's' => self.press(Button::Spit, out),
                    'g' => self.press(Button::Gulp, out),
                    'c' => self.press(Button::Croak, out),
                    _ => {}
                },
                In::Key(Key::Space) | In::Key(Key::Enter) => self.press(Button::End, out),
                _ => {}
            },
            Phase::Bank { offers } => match ev {
                In::Move(x, y) => self.hover_item = (0..offers.len()).find(|&i| mutation_rect(i).contains(x, y)),
                In::Press { x, y, right: false } => {
                    if let Some(i) = (0..offers.len()).find(|&i| mutation_rect(i).contains(x, y)) {
                        self.pick(i, out);
                    }
                }
                _ => {}
            },
            Phase::Over { .. } => {
                if matches!(ev, In::Press { right: false, .. } | In::Key(Key::Enter)) {
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
        if self.card > 0 {
            self.card -= 1;
        }
        if self.hurt_flash > 0 {
            self.hurt_flash -= 1;
        }
        if matches!(self.phase, Phase::Lesson | Phase::Pond) && self.anim.is_none() && self.card == 0 {
            self.think += 1;
        }
        let mut done = false;
        if let Some(a) = self.anim.as_mut() {
            if a.t == 0 {
                for e in &a.mine {
                    let s = match e {
                        Ev::Hop { .. } => Some(Sfx::Hop),
                        Ev::Tongue { .. } => Some(Sfx::Tongue),
                        Ev::Spit { .. } => Some(Sfx::Spit),
                        Ev::Gulp { grew: false, .. } => Some(Sfx::Gulp),
                        Ev::Gulp { grew: true, .. } => Some(Sfx::Grow),
                        Ev::Croak => Some(Sfx::Croak),
                        Ev::Sank { .. } => Some(Sfx::Splash),
                        _ => None,
                    };
                    if let Some(s) = s {
                        out.sfx(s);
                    }
                }
            }
            if a.t == 2 {
                for e in &a.mine {
                    match e {
                        Ev::Swallow { .. } => out.sfx(Sfx::Gulp),
                        Ev::Died { how, .. } if how.contains("WATER") || *how == "DROWNED" => out.sfx(Sfx::Drown),
                        Ev::Died { .. } => out.sfx(Sfx::Fall),
                        Ev::Damage { .. } => out.sfx(Sfx::Thud),
                        Ev::Hurt { .. } => out.sfx(Sfx::Hurt),
                        Ev::Cleared => out.sfx(Sfx::Lesson),
                        _ => {}
                    }
                }
            }
            if a.t == Anim::MINE + 1 {
                let mut hurt = false;
                for e in &a.theirs {
                    match e {
                        Ev::Hurt { .. } => hurt = true,
                        Ev::Died { how, .. } if how.contains("WATER") => out.sfx(Sfx::Drown),
                        Ev::Died { .. } => out.sfx(Sfx::Fall),
                        Ev::Damage { .. } => out.sfx(Sfx::Thud),
                        Ev::Ripple { .. } => out.sfx(Sfx::Rustle),
                        _ => {}
                    }
                }
                if hurt {
                    out.sfx(Sfx::Hurt);
                    self.hurt_flash = 3;
                }
            }
            a.t += 1;
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
        let headline = match self.phase {
            Phase::Over { won: true } => "GROF IS DEFEATED. THE POND IS YOURS.".to_string(),
            _ if self.lesson.is_some() => format!("STOPPED DURING LESSON {}.", self.lesson.unwrap() + 1),
            Phase::Over { won: false } => format!("THE FROG FELL IN {}, POND {} OF {PONDS}.", self.area_name(), self.pond_no),
            _ => format!("STOPPED IN {}, POND {} OF {PONDS}.", self.area_name(), self.pond_no),
        };
        let f = &self.frog;
        let muts = if f.muts.is_empty() { "NO MUTATIONS".to_string() } else { f.muts.iter().map(|m| m.name()).collect::<Vec<_>>().join(", ") };
        let s = &self.stats;
        let detail = format!(
            "A {} FROG. ATE {}, DEFEATED {}, HURT {} TIMES OVER {} TURNS. EGG CLUTCHES SAVED {}, LOST {}. {muts}.",
            f.size_name(),
            s.eaten,
            s.defeated,
            s.hurt,
            s.turns,
            s.eggs_saved,
            s.eggs_lost
        );
        let won = matches!(self.phase, Phase::Over { won: true });
        Summary { headline, detail, best_key: self.pond_no + if won { 100 } else { 0 } }
    }

    fn takes_esc(&self) -> bool {
        self.show_help
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A greedy bot: eat when it can, spit at predators, otherwise hop to the safest square
    /// nearest the most useful thing.
    fn bot(g: &mut Game, out: &mut Out) {
        let p = &g.pond;
        let f = &p.frog;
        if f.act_left {
            let threatened: Vec<u32> = p.danger().iter().filter(|(q, _)| p.eggs.contains(q)).map(|(_, id)| *id).collect();
            if !threatened.is_empty() {
                // Croak if a raider is next to us; otherwise tongue it (yank or swallow).
                if f.croak_cd == 0 && p.creatures.iter().any(|c| threatened.contains(&c.id) && cheb(c.pos, f.pos) == 1) {
                    g.do_act(Act::Croak, out);
                    return;
                }
                for d in DIRS8 {
                    let (_, who, _) = p.line_hit(d, f.tongue_range());
                    if who.is_some_and(|i| threatened.contains(&p.creatures[i].id)) {
                        g.do_act(Act::Tongue(d), out);
                        return;
                    }
                }
            }
            // A small frog digests right away: growing is how it gets teeth.
            if !f.throat.is_empty() && f.size == 1 {
                g.do_act(Act::Gulp, out);
                return;
            }
            // Digest when full or hurt.
            if !f.throat.is_empty() && (f.throat.len() >= f.throat_cap() || f.hearts < f.max_hearts) && p.creatures.iter().all(|c| c.kind == Kind::Fly) {
                g.do_act(Act::Gulp, out);
                return;
            }
            for d in DIRS8 {
                let (_, who, _) = p.line_hit(d, f.tongue_range());
                if let Some(i) = who {
                    let c = &p.creatures[i];
                    if c.kind.info().size < f.size && f.throat.len() < f.throat_cap() {
                        g.do_act(Act::Tongue(d), out);
                        return;
                    }
                }
                if !f.throat.is_empty() {
                    let (_, who, _) = p.line_hit(d, 4);
                    if let Some(i) = who {
                        if p.creatures[i].kind != Kind::Fly {
                            g.do_act(Act::Spit(d), out);
                            return;
                        }
                    }
                }
            }
            if !f.throat.is_empty() {
                g.do_act(Act::Gulp, out);
                return;
            }
            if f.croak_cd == 0 && p.creatures.iter().any(|c| cheb(c.pos, f.pos) == 1 && c.kind != Kind::Fly) {
                g.do_act(Act::Croak, out);
                return;
            }
        }
        if f.hop_left {
            let goal = if p.cleared {
                p.exit
            } else {
                p.creatures.iter().filter(|c| c.kind != Kind::Fly).min_by_key(|c| cheb(c.pos, f.pos)).map(|c| c.pos).unwrap_or(p.exit)
            };
            let mut opts: Vec<(i32, i32, Pos)> = p.hop_targets().iter().map(|&h| (p.preview_hop(h), cheb(h, goal), h)).collect();
            opts.sort();
            let stay = p.preview_hop(f.pos);
            if let Some(&(hits, _, h)) = opts.first() {
                if hits <= stay {
                    g.do_act(Act::Hop(h), out);
                    return;
                }
            }
        }
        g.do_act(Act::End, out);
    }

    fn drive(g: &mut Game, out: &mut Out, max: usize) {
        for _ in 0..max {
            if g.ended {
                return;
            }
            while g.card > 0 || g.anim.is_some() {
                g.tick(out);
            }
            match g.phase.clone() {
                Phase::Lesson | Phase::Pond => bot(g, out),
                Phase::Bank { .. } => g.pick(0, out),
                Phase::Over { .. } => g.ended = true,
            }
        }
    }

    #[test]
    fn a_bot_gets_through_the_lessons() {
        let mut g = Game::new(4, true);
        let mut out = Out::default();
        drive(&mut g, &mut out, 1500);
        assert!(g.lesson.is_none(), "stuck in lesson {:?} step {}: {:?}", g.lesson, g.lesson_step, &out.log[out.log.len().saturating_sub(4)..]);
    }

    #[test]
    fn bots_play_whole_runs() {
        let mut res = Vec::new();
        for seed in 0..20 {
            let mut g = Game::new(seed, false);
            let mut out = Out::default();
            drive(&mut g, &mut out, 4000);
            res.push((g.pond_no, matches!(g.phase, Phase::Over { won: true }), g.stats.hurt, g.stats.eggs_lost, g.stats.eggs_saved, g.stats.turns));
            assert!(g.pond_no <= PONDS);
        }
        println!("gulp bot (pond, won, size, turns): {res:?}");
    }
}
