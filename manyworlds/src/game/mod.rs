//! Leg 1 as a playable gray-box: input, the interactive tutorial, the HUD, and drawing into the
//! indexed framebuffer. Platform code (the browser shell in `web/`) feeds it input events and the
//! song's clock, calls `frame()` at 15 FPS, and shows `present()`.
//!
//! The song is the clock: the simulation advances to `song_tick`, which the shell takes from the
//! audio's playback position, so the leg ends exactly when the track does.

pub mod art;
pub mod fb;

use crate::geom::*;
use crate::map::{self, Map};
use crate::plan;
use crate::sim::{Ev, Params, Sim};
use art::*;
use fb::*;

pub const TILE: i32 = 16;
pub const STRIP_Y: i32 = GH * TILE;

pub const TRACK: &str = "THE CORNER OF MAPLE ST. AND THE FUTURE";
pub const ALBUM: &str = "THE MARVELOUS ELECTRICAL WORLD OF TOMORROW";

/// Lamp thresholds for this map at these rules: 30%, 60% and 85% of the planner bot's mean
/// score (`cargo run --release --bin gate -- lamps`).
pub const LAMPS: [i32; 3] = [112, 223, 316];

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Key {
    Space,
    E,
    P,
    R,
    F2,
    M,
    L,
    Esc,
    Enter,
}

impl Key {
    pub fn from_code(c: i32) -> Option<Key> {
        Some(match c {
            1 => Key::Space,
            2 => Key::E,
            3 => Key::P,
            4 => Key::R,
            5 => Key::F2,
            6 => Key::M,
            7 => Key::L,
            8 => Key::Esc,
            9 => Key::Enter,
            _ => return None,
        })
    }
}

/// Input, latched by the shell as it happens and handled in order on the next frame.
#[derive(Clone, Copy, Debug)]
pub enum In {
    Move(i32, i32),
    /// Button 0 is left, 2 is right.
    Down(u8, i32, i32),
    Up(u8, i32, i32),
    Key(Key, bool),
}

/// Sound effects the shell should play, as bits.
pub mod sfx {
    pub const DELIVER: u32 = 1;
    pub const BOARD: u32 = 2;
    pub const STALL: u32 = 4;
    pub const CRASH: u32 = 8;
    pub const LAMP: u32 = 16;
    pub const CLICK: u32 = 32;
    pub const PARK: u32 = 64;
    pub const DRAW: u32 = 128;
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Screen {
    Title,
    Play,
    End,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Btn {
    Erase,
    Express,
    Lift,
    Scan,
    Sound,
    Pause,
    Restart,
}

const BUTTONS: [Btn; 7] = [
    Btn::Erase,
    Btn::Express,
    Btn::Lift,
    Btn::Scan,
    Btn::Sound,
    Btn::Pause,
    Btn::Restart,
];

fn button_at(x: i32, y: i32) -> Option<Btn> {
    if !(278..294).contains(&y) {
        return None;
    }
    BUTTONS
        .iter()
        .enumerate()
        .find(|(i, _)| (button_x(*i)..button_x(*i) + 16).contains(&x))
        .map(|(_, b)| *b)
}

fn button_x(i: usize) -> i32 {
    272 + i as i32 * 18
}

fn tile_at(x: i32, y: i32) -> Option<Pos> {
    (x >= 0 && y >= 0 && x < GW * TILE && y < STRIP_Y).then(|| Pos::new(x / TILE, y / TILE))
}

struct Stroke {
    start: Pos,
    last: Pos,
    prev: Option<Pos>,
    moved: bool,
    erase: bool,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Step {
    Draw,
    Match,
    Rear,
    Express,
    Free,
    LastStop,
    Done,
}

/// The interactive tutorial for leg 1: a caption of at most three words, a ghost hand showing the
/// gesture on the real board, and a check that the player has done it.
struct Tutorial {
    step: Step,
    demo: Vec<Pos>,
    target: Option<Pos>,
    express_from: i32,
}

pub struct Game {
    pub sim: Sim,
    map: Map,
    seconds: i32,
    pub screen: Screen,
    pub paused: bool,
    pub scanlines: bool,
    pub sound: bool,
    erase_mode: bool,
    express_key: bool,
    express_btn: bool,
    events: Vec<In>,
    mouse: (i32, i32),
    stroke: Option<Stroke>,
    held_btn: Option<Btn>,
    pub frame: u32,
    song_tick: i32,
    /// Sound effects since the shell last asked.
    pub sfx: u32,
    /// Bumped whenever the song should start again from the top.
    pub audio_gen: u32,
    tut: Option<Tutorial>,
    tutorial_done: bool,
    release_hint: i32,
    told_release: bool,
    play_frame: u32,
    lamps_lit: usize,
    /// The share of the leg left when free play began; lamp thresholds shrink to match after
    /// the tutorial has used part of the song.
    lamp_share: (i32, i32),
    fb: Fb,
    art: Art,
    out: Vec<u8>,
    cursor: Vec<u8>,
}

impl Game {
    pub fn new(seconds: i32) -> Game {
        let map = map::town();
        let sim = Sim::new(&map, Params::for_track(seconds), 1);
        Game {
            sim,
            map,
            seconds,
            screen: Screen::Title,
            paused: false,
            scanlines: true,
            sound: true,
            erase_mode: false,
            express_key: false,
            express_btn: false,
            events: vec![],
            mouse: (-1, -1),
            stroke: None,
            held_btn: None,
            frame: 0,
            song_tick: 0,
            sfx: 0,
            audio_gen: 0,
            tut: None,
            tutorial_done: false,
            release_hint: 0,
            told_release: false,
            play_frame: 0,
            lamps_lit: 0,
            lamp_share: (1, 1),
            fb: Fb::default(),
            art: Art::new(),
            out: vec![],
            cursor: vec![],
        }
    }

    /// The song's length, once the shell knows it.
    pub fn set_length(&mut self, seconds: i32) {
        if self.screen == Screen::Title && seconds > 30 {
            self.seconds = seconds;
            self.sim = Sim::new(&self.map, Params::for_track(seconds), 1);
        }
    }

    pub fn push(&mut self, ev: In) {
        self.events.push(ev);
    }

    pub fn set_song_tick(&mut self, t: i32) {
        self.song_tick = t;
    }

    /// Pause, if a leg is being played (the shell does this when the page is hidden).
    pub fn pause(&mut self) {
        if self.screen == Screen::Play {
            self.paused = true;
        }
    }

    /// Lamp thresholds for this leg.
    pub fn lamps(&self) -> [i32; 3] {
        LAMPS.map(|t| t * self.lamp_share.0 / self.lamp_share.1.max(1))
    }

    pub fn wants_audio(&self) -> bool {
        match self.screen {
            Screen::Title => false,
            Screen::Play => !self.paused,
            Screen::End => self.sim.tick < self.sim.p.ticks,
        }
    }

    fn start(&mut self) {
        let mut p = Params::for_track(self.seconds);
        p.auto_lift = self.sim.p.auto_lift;
        self.sim = Sim::new(&self.map, p, 1 + self.audio_gen as u64);
        self.screen = Screen::Play;
        self.paused = false;
        self.stroke = None;
        self.song_tick = 0;
        self.audio_gen += 1;
        self.play_frame = 0;
        self.lamps_lit = 0;
        self.lamp_share = (1, 1);
        self.tut = None;
        if !self.tutorial_done {
            self.sim.hold = true;
            self.sim.spawning = false;
            // a: one passenger for ◆. b: one for ■, then one for ● (who will be the rear car).
            self.sim.stations[0].queue.extend([1]);
            self.sim.stations[1].queue.extend([2, 0]);
            self.tut = Some(Tutorial {
                step: Step::Draw,
                demo: vec![],
                target: None,
                express_from: 0,
            });
        }
    }

    /// One 15 FPS frame: handle latched input, advance the leg to the song's clock, draw.
    pub fn frame(&mut self) {
        self.frame += 1;
        for ev in std::mem::take(&mut self.events) {
            self.handle(ev);
        }
        if self.screen == Screen::Play && !self.paused {
            self.play_frame += 1;
            self.sim.set_express(self.express_key || self.express_btn);
            let before = self.sim.stats.clone();
            let log_from = self.sim.log.len();
            let mut steps = 0;
            while self.sim.tick < self.song_tick && steps < 8 && !self.sim.finished() {
                self.sim.step();
                steps += 1;
            }
            let after = &self.sim.stats;
            if after.delivered > before.delivered {
                self.sfx |= sfx::DELIVER;
            }
            if after.boarded > before.boarded {
                self.sfx |= sfx::BOARD;
            }
            if after.stalls > before.stalls {
                self.sfx |= sfx::STALL;
            }
            if after.crashes > before.crashes {
                self.sfx |= sfx::CRASH;
            }
            let lit = self
                .lamps()
                .iter()
                .filter(|&&t| self.sim.stats.score >= t)
                .count();
            if lit > self.lamps_lit {
                self.sfx |= sfx::LAMP;
                self.lamps_lit = lit;
            }
            if self.sim.log[log_from..].iter().any(|e| e.1 == Ev::Skip) && !self.told_release {
                self.told_release = true;
                self.release_hint = 60;
            }
            self.release_hint = (self.release_hint - 1).max(0);
            self.tutorial();
            if self.sim.finished() {
                self.screen = Screen::End;
                self.stroke = None;
                if self.sim.parked {
                    self.sfx |= sfx::PARK;
                }
                if self.tut.is_some() {
                    self.tutorial_done = true;
                }
            }
        }
        self.render();
    }

    fn handle(&mut self, ev: In) {
        match ev {
            In::Move(x, y) => {
                self.mouse = (x, y);
                if let Some(t) = tile_at(x, y) {
                    self.drag_to(t);
                }
            }
            In::Down(b, x, y) => {
                self.mouse = (x, y);
                match self.screen {
                    Screen::Title => self.start(),
                    Screen::End => {
                        if end_button(x, y) {
                            self.start();
                        }
                    }
                    Screen::Play if self.paused => {
                        if let Some(btn) = button_at(x, y) {
                            self.press(btn);
                        } else {
                            self.paused = false;
                        }
                    }
                    Screen::Play => {
                        if let Some(btn) = button_at(x, y) {
                            self.press(btn);
                        } else if let Some(t) = tile_at(x, y) {
                            let erase = b == 2 || self.erase_mode;
                            self.stroke = Some(Stroke {
                                start: t,
                                last: t,
                                prev: None,
                                moved: false,
                                erase,
                            });
                            if erase && self.sim.erase(t) {
                                self.sfx |= sfx::CLICK;
                            }
                        }
                    }
                }
            }
            In::Up(_, x, y) => {
                self.mouse = (x, y);
                if let Some(btn) = self.held_btn.take() {
                    if btn == Btn::Express {
                        self.express_btn = false;
                    }
                }
                if let Some(s) = self.stroke.take() {
                    if !s.erase && !s.moved && self.sim.flip(s.start) {
                        self.sfx |= sfx::CLICK;
                    }
                }
            }
            In::Key(k, down) => self.key(k, down),
        }
    }

    fn key(&mut self, k: Key, down: bool) {
        if k == Key::Space {
            self.express_key = down;
            return;
        }
        if !down {
            return;
        }
        match k {
            Key::E => self.erase_mode = !self.erase_mode,
            Key::P | Key::Esc if self.screen == Screen::Play => self.paused = !self.paused,
            Key::R if self.screen != Screen::Title => self.start(),
            Key::Enter if self.screen != Screen::Play => self.start(),
            Key::F2 => self.scanlines = !self.scanlines,
            Key::M => self.sound = !self.sound,
            Key::L => self.sim.p.auto_lift = !self.sim.p.auto_lift,
            _ => {}
        }
    }

    fn press(&mut self, b: Btn) {
        self.held_btn = Some(b);
        self.sfx |= sfx::CLICK;
        match b {
            Btn::Erase => self.erase_mode = !self.erase_mode,
            Btn::Express => self.express_btn = true,
            Btn::Lift => self.sim.p.auto_lift = !self.sim.p.auto_lift,
            Btn::Scan => self.scanlines = !self.scanlines,
            Btn::Sound => self.sound = !self.sound,
            Btn::Pause => self.paused = !self.paused,
            Btn::Restart => self.start(),
        }
    }

    /// Extend the drag to tile `t`, through every tile in between so fast strokes leave no gaps.
    fn drag_to(&mut self, t: Pos) {
        let Some(mut s) = self.stroke.take() else {
            return;
        };
        while s.last != t {
            let (dx, dy) = (t.x - s.last.x, t.y - s.last.y);
            let d = if dx.abs() >= dy.abs() {
                if dx > 0 {
                    Dir::E
                } else {
                    Dir::W
                }
            } else if dy > 0 {
                Dir::S
            } else {
                Dir::N
            };
            let v = s.last.step(d);
            s.moved = true;
            if s.erase {
                if self.sim.erase(v) {
                    self.sfx |= sfx::CLICK;
                }
                s.last = v;
                continue;
            }
            let came = s.prev.and_then(|p| s.last.dir_to(p));
            if self.sim.draw_seg(came, s.last, v) {
                s.prev = Some(s.last);
                self.sfx |= sfx::DRAW;
            } else {
                s.prev = None;
            }
            s.last = v;
        }
        self.stroke = Some(s);
    }

    fn tutorial(&mut self) {
        let Some(tut) = self.tut.as_mut() else { return };
        let sim = &mut self.sim;
        let route = plan::route_ahead(sim);
        let reaches = |p: Pos| route.tiles.contains(&p) || route.end == p;
        let a = sim.stations[0].pos;
        let b = sim.stations[1].pos;
        if sim.tick >= sim.p.fade && tut.step != Step::LastStop {
            tut.step = Step::LastStop;
            sim.hold = false;
            sim.spawning = true;
        }
        tut.target = None;
        match tut.step {
            Step::Draw => {
                tut.target = Some(a);
                if route.tiles.contains(&a) {
                    sim.hold = false;
                    tut.step = Step::Match;
                }
            }
            Step::Match => {
                if !sim.cars.is_empty() {
                    tut.target = Some(b);
                }
                if sim.stats.delivered >= 1 {
                    tut.step = Step::Rear;
                }
            }
            Step::Rear => {
                tut.target = sim
                    .cars
                    .last()
                    .and_then(|&c| sim.stations.iter().find(|s| s.shape == c))
                    .map(|s| s.pos);
                if sim.cars.is_empty() {
                    sim.spawning = true;
                    tut.step = Step::Express;
                    tut.express_from = sim.stats.express_moves;
                    self.lamp_share = ((sim.p.ticks - sim.tick).max(1), sim.p.ticks);
                }
            }
            Step::Express => {
                if sim.stats.express_moves - tut.express_from >= 4 {
                    tut.step = Step::Free;
                }
            }
            Step::Free => {}
            Step::LastStop => {
                tut.target = Some(sim.terminus);
                if sim.parked {
                    tut.step = Step::Done;
                }
            }
            Step::Done => {}
        }
        // The ghost hand shows the shortest new rail from the end of the line to the target.
        tut.demo.clear();
        if let Some(t) = tut.target {
            if !reaches(t) && !route.looped {
                let avoid = vec![false; NT];
                if let Some(p) = plan::path_now(sim, &route, |q| q == t, &avoid) {
                    tut.demo = p;
                }
            }
        }
    }

    /// A switch whose closed arm is what stops the train.
    fn blocking_switch(&self) -> Option<Pos> {
        let e = self.sim.engine();
        let d = self.sim.exit_of(e, self.sim.entry)?;
        let n = e.step(d);
        (n.ok()
            && self.sim.mask[n.idx()].count_ones() == 3
            && self.sim.lit[n.idx()] & d.opp().bit() == 0)
            .then_some(n)
    }

    fn caption(&self) -> Option<&'static str> {
        if self.blocking_switch().is_some() && !self.sim.hold {
            return Some("CLICK THE SWITCH");
        }
        if self.release_hint > 0 {
            return Some("RELEASE AT STATIONS");
        }
        let tut = self.tut.as_ref()?;
        match tut.step {
            Step::Draw => Some("DRAW RAIL"),
            Step::Match if !self.sim.cars.is_empty() => Some("MATCH SHAPES"),
            Step::Rear if self.sim.cars.len() >= 2 => Some("REAR CAR FIRST"),
            Step::Express => {
                let r = plan::route_ahead(&self.sim);
                (r.tiles.len() >= 6 && r.next_stop(&self.sim).is_none_or(|i| i >= 3))
                    .then_some("HOLD SPACE")
            }
            Step::LastStop if !self.sim.parked => Some("LAST STOP"),
            _ => None,
        }
    }

    // ----------------------------------------------------------------------------------------
    // Drawing

    fn render(&mut self) {
        match self.screen {
            Screen::Title => self.render_title(),
            Screen::Play | Screen::End => {
                self.render_board();
                self.render_strip();
                if self.screen == Screen::End {
                    self.render_end();
                } else if self.paused {
                    self.fb.checker(0, 0, FB_W, STRIP_Y, INK);
                    self.fb.text_centered("PAUSED", FB_W / 2, 110, TEXT, 3);
                    self.fb
                        .text_centered("CLICK TO RESUME", FB_W / 2, 146, TEXT_DIM, 1);
                }
            }
        }
    }

    fn render_title(&mut self) {
        let fb = &mut self.fb;
        fb.clear(NIGHT);
        for i in 0..90u32 {
            let h = i.wrapping_mul(2_654_435_761);
            fb.pset(
                (h % 400) as i32,
                ((h >> 9) % 300) as i32,
                if i % 7 == 0 { TEXT } else { TEXT_DIM },
            );
        }
        fb.text_centered("MANY WORLDS LINE", FB_W / 2, 58, LAMP_ON, 3);
        fb.text_centered(TRACK, FB_W / 2, 112, TEXT, 1);
        fb.text_centered(ALBUM, FB_W / 2, 126, TEXT_DIM, 1);
        fb.text_centered("HENRY SOLBERG", FB_W / 2, 140, TEXT_DIM, 1);
        fb.bevel(FB_W / 2 - 22, 170, 44, 34, PANEL, PANEL_HI, PANEL_LO);
        for i in 0..12 {
            fb.rect(FB_W / 2 - 6 + i, 177 + i / 2, 1, 21 - i, LAMP_ON);
        }
        fb.text_centered("CLICK TO START", FB_W / 2, 222, TEXT_DIM, 1);
        fb.text_centered("F2 SCANLINES", FB_W / 2, 270, TEXT_DIM, 1);
    }

    fn render_board(&mut self) {
        let caption = self.caption();
        let blocking = self.blocking_switch();
        let (fb, sim, map) = (&mut self.fb, &self.sim, &self.map);
        // ground, water and buildings
        for ty in 0..GH {
            for tx in 0..GW {
                let p = Pos::new(tx, ty);
                let (x, y) = (tx * TILE, ty * TILE);
                if map.water[p.idx()] {
                    fb.rect(x, y, TILE, TILE, WATER0);
                    for k in 0..TILE * TILE {
                        let (px, py) = (x + k % TILE, y + k / TILE);
                        if (px * 3 + py * 7) % 13 == 0 && py % 4 == 1 {
                            fb.rect(px, py, 3, 1, WATER1);
                        }
                    }
                } else if sim.blocked[p.idx()] {
                    fb.rect(x, y, TILE, TILE, GROUND0);
                    fb.bevel(x + 1, y + 1, TILE - 2, TILE - 2, BLDG1, BLDG2, BLDG0);
                    for wy in 0..3 {
                        for wx in 0..3 {
                            let lit = ((tx * 7 + ty * 13 + wx * 3 + wy * 5) % 4) == 0;
                            fb.rect(
                                x + 3 + wx * 4,
                                y + 3 + wy * 4,
                                2,
                                2,
                                if lit { WINDOW } else { BLDG0 },
                            );
                        }
                    }
                } else {
                    fb.rect(
                        x,
                        y,
                        TILE,
                        TILE,
                        if (tx + ty) % 2 == 0 { GROUND0 } else { GROUND1 },
                    );
                    fb.pset(x, y, GROUND2);
                }
            }
        }
        // station plates under the rail
        for s in &sim.stations {
            let (cx, cy) = center(s.pos);
            if sim.tick >= s.appear {
                fb.rect(cx - 7, cy - 7, 14, 14, PLATFORM);
                fb.rect(cx - 7, cy - 7, 14, 1, PLATFORM_HI);
                fb.frame(cx - 8, cy - 8, 16, 16, INK);
                fb.shape(s.shape, cx, cy, 6, SHAPE[s.shape as usize]);
            } else {
                fb.frame_dotted(cx - 6, cy - 6, 12, 12, TEXT_DIM);
            }
        }
        {
            let (cx, cy) = center(sim.terminus);
            fb.rect(
                cx - 7,
                cy - 7,
                14,
                14,
                if sim.terminus_open() {
                    TERMINUS
                } else {
                    PLATFORM
                },
            );
            fb.frame(cx - 8, cy - 8, 16, 16, INK);
            fb.rect(
                cx - 5,
                cy - 2,
                10,
                4,
                if sim.terminus_open() {
                    LAMP_ON
                } else {
                    TEXT_DIM
                },
            );
            fb.frame(cx - 5, cy - 2, 10, 4, INK);
        }
        // the tutorial's ghost rail
        if let Some(t) = &self.tut {
            for p in t.demo.iter().skip(1) {
                fb.checker(p.x * TILE + 4, p.y * TILE + 4, 8, 8, GHOST);
            }
            if let Some(tp) = t.target {
                fb.frame_dotted(tp.x * TILE - 1, tp.y * TILE - 1, TILE + 2, TILE + 2, GHOST);
            }
        }
        // rail
        for i in 0..NT {
            let m = sim.mask[i];
            if m != 0 {
                draw_rail(fb, Pos::from_idx(i), m, sim.lit[i]);
            }
        }
        // what happens next: the tile the engine enters, or a warning
        let route = plan::route_ahead(sim);
        if let Some(n) = sim.next_tile() {
            fb.frame_dotted(n.x * TILE, n.y * TILE, TILE, TILE, HILITE);
        } else if !sim.hold && !sim.parked {
            let e = sim.engine();
            fb.frame(e.x * TILE, e.y * TILE, TILE, TILE, WARN);
        }
        let tail = sim.body.len() - 1;
        for t in route.tiles.iter().take(3) {
            if sim.body.iter().take(tail).any(|b| b == t) {
                fb.checker(t.x * TILE, t.y * TILE, TILE, TILE, WARN);
            }
        }
        if let Some(b) = blocking {
            fb.frame_dotted(b.x * TILE - 1, b.y * TILE - 1, TILE + 2, TILE + 2, GHOST);
        }
        draw_train(fb, sim, &route);
        // a station the train is standing on still shows its shape round the edge
        for s in &sim.stations {
            if sim.tick >= s.appear && sim.on_body(s.pos) {
                let (cx, cy) = center(s.pos);
                fb.frame(cx - 8, cy - 8, 16, 16, SHAPE[s.shape as usize]);
                fb.frame(cx - 7, cy - 7, 14, 14, SHAPE_DK[s.shape as usize]);
            }
        }
        // waiting passengers, first in line on the left
        for s in &sim.stations {
            if sim.tick < s.appear || s.queue.is_empty() {
                continue;
            }
            let n = s.queue.len() as i32;
            let (cx, cy) = center(s.pos);
            let w = n * 4 + 1;
            let x0 = (cx - w / 2).clamp(0, FB_W - w);
            let y0 = if s.pos.y == 0 { cy + 9 } else { cy - 13 };
            fb.rect(x0, y0, w, 5, INK);
            for (k, &c) in s.queue.iter().enumerate() {
                fb.rect(x0 + 1 + k as i32 * 4, y0 + 1, 3, 3, SHAPE[c as usize]);
            }
        }
        // the ghost hand, sliding along the demonstration
        if let Some(t) = &self.tut {
            if t.demo.len() >= 2 {
                let len = t.demo.len() as u32;
                let k = ((self.frame / 2) % (len + 6)).min(len - 1) as usize;
                let (cx, cy) = center(t.demo[k]);
                fb.blit(&self.art.hand, cx - 4, cy);
            }
        }
        if let Some(c) = caption {
            // Out of the way of the train: at the bottom while the engine is near the top.
            let y = if sim.engine().y <= 3 { STRIP_Y - 30 } else { 6 };
            let w = Fb::text_width(c, 2) + 16;
            fb.bevel(FB_W / 2 - w / 2, y, w, 24, PANEL, PANEL_HI, PANEL_LO);
            fb.text_centered(c, FB_W / 2, y + 5, TEXT, 2);
        }
        let caption_low = caption.is_some() && sim.engine().y <= 3;
        if self.play_frame < 75 && !caption_low {
            fb.text_shadow(TRACK, 6, STRIP_Y - 20, TEXT, 1);
            fb.text_shadow(ALBUM, 6, STRIP_Y - 10, TEXT_DIM, 1);
        }
    }

    fn render_strip(&mut self) {
        let lamps = self.lamps();
        let (fb, sim) = (&mut self.fb, &self.sim);
        fb.rect(0, STRIP_Y, FB_W, FB_H - STRIP_Y, PANEL);
        fb.rect(0, STRIP_Y, FB_W, 1, PANEL_HI);
        // lamps, and how close the next one is
        for (i, &t) in lamps.iter().enumerate() {
            let s = if sim.stats.score >= t {
                &self.art.lamp_on
            } else {
                &self.art.lamp_off
            };
            fb.blit(s, 4 + i as i32 * 10, 277);
        }
        let lit = lamps.iter().filter(|&&t| sim.stats.score >= t).count();
        let (lo, hi) = (
            if lit == 0 { 0 } else { lamps[lit - 1] },
            lamps.get(lit).copied().unwrap_or(lamps[2]),
        );
        let fill = if lit == 3 {
            27
        } else {
            27 * (sim.stats.score - lo).max(0) / (hi - lo).max(1)
        };
        fb.rect(4, 295, 27, 2, PANEL_LO);
        fb.rect(4, 295, fill, 2, LAMP_ON);
        // score and multiplier lanterns
        fb.text(&sim.stats.score.to_string(), 38, 278, TEXT, 1);
        for i in 0..4 {
            let on = i < sim.mult();
            fb.rect(38 + i * 6, 288, 4, 5, INK);
            fb.rect(39 + i * 6, 289, 2, 3, if on { LAMP_ON } else { LAMP_OFF });
        }
        // rail pieces left
        let free = sim.free();
        for k in 0..4 {
            fb.rect(70 + k * 3, 279, 1, 6, TIE);
        }
        fb.rect(69, 280, 12, 1, RAIL);
        fb.rect(69, 283, 12, 1, RAIL);
        fb.text(
            &free.to_string(),
            70,
            288,
            if free < 4 { WARN } else { TEXT_DIM },
            1,
        );
        // the song: time left, and where the terminus opens
        let left = ((sim.p.ticks - sim.tick).max(0) + 14) / 15;
        fb.text(
            &format!("{}:{:02}", left / 60, left % 60),
            98,
            278,
            TEXT_DIM,
            1,
        );
        let (bx, bw) = (98, 166);
        fb.rect(bx, 289, bw, 4, PANEL_LO);
        fb.rect(
            bx,
            289,
            bw * sim.tick.min(sim.p.ticks) / sim.p.ticks.max(1),
            4,
            TEXT_DIM,
        );
        fb.rect(
            bx + bw * sim.p.fade / sim.p.ticks.max(1),
            287,
            1,
            8,
            TERMINUS,
        );
        if self.erase_mode {
            fb.text("ERASE", 136, 278, WARN, 1);
        }
        // buttons
        for (i, b) in BUTTONS.iter().enumerate() {
            let on = match b {
                Btn::Erase => self.erase_mode,
                Btn::Express => sim.express,
                Btn::Lift => sim.p.auto_lift,
                Btn::Scan => self.scanlines,
                Btn::Sound => self.sound,
                Btn::Pause => self.paused,
                Btn::Restart => false,
            };
            let x = button_x(i);
            if on {
                fb.bevel(x, 278, 16, 16, PANEL_LO, PANEL_LO, PANEL_HI);
            } else {
                fb.bevel(x, 278, 16, 16, PANEL_HI, TEXT_DIM, PANEL_LO);
            }
            fb.blit(&self.art.icons[i], x + 2, 280);
            let tut_wants_express =
                *b == Btn::Express && self.tut.as_ref().is_some_and(|t| t.step == Step::Express);
            if tut_wants_express {
                fb.frame_dotted(x - 2, 276, 20, 20, GHOST);
            }
        }
    }

    fn render_end(&mut self) {
        let lamps = self.lamps();
        let (fb, sim) = (&mut self.fb, &self.sim);
        fb.checker(0, 0, FB_W, STRIP_Y, INK);
        let (x, y, w, h) = (FB_W / 2 - 100, 50, 200, 170);
        fb.bevel(x, y, w, h, PANEL, PANEL_HI, PANEL_LO);
        for (i, &t) in lamps.iter().enumerate() {
            let s = if sim.stats.score >= t {
                &self.art.lamp_on
            } else {
                &self.art.lamp_off
            };
            blit_scaled(fb, s, FB_W / 2 - 40 + i as i32 * 30 - 7, y + 14, 2);
        }
        fb.text_centered(&sim.stats.score.to_string(), FB_W / 2, y + 60, TEXT, 3);
        let line = format!("{} DELIVERED", sim.stats.delivered);
        fb.text_centered(&line, FB_W / 2, y + 92, TEXT_DIM, 1);
        if sim.parked {
            fb.text_centered("REACHED THE LAST STOP", FB_W / 2, y + 104, TERMINUS, 1);
        }
        let (bx, by) = end_button_rect();
        fb.bevel(bx, by, 60, 22, PANEL_HI, TEXT_DIM, PANEL_LO);
        fb.blit(&self.art.icons[6], bx + 6, by + 5);
        fb.text("AGAIN", bx + 22, by + 8, TEXT, 1);
    }

    /// The frame at an exact integer scale, RGBA.
    pub fn present(&mut self, n: usize) -> &[u8] {
        self.fb.present(&mut self.out, n, self.scanlines);
        &self.out
    }

    /// The pointer as an RGBA image at scale `n`: each logical pixel an exact n x n block,
    /// index 0 fully transparent, everything else fully opaque. Returns (pixels, w, h).
    pub fn cursor(&mut self, n: usize) -> (&[u8], usize, usize) {
        let s = &self.art.pointer;
        let (w, h) = (s.w as usize * n, s.h as usize * n);
        self.cursor.clear();
        self.cursor.resize(w * h * 4, 0);
        for y in 0..h {
            for x in 0..w {
                let c = s.px[(y / n) * s.w as usize + x / n];
                if c != 0 {
                    let v = rgb(c);
                    self.cursor[(y * w + x) * 4..(y * w + x) * 4 + 4].copy_from_slice(&[
                        (v >> 16) as u8,
                        (v >> 8) as u8,
                        v as u8,
                        255,
                    ]);
                }
            }
        }
        (&self.cursor, w, h)
    }
}

fn end_button_rect() -> (i32, i32) {
    (FB_W / 2 - 30, 186)
}

fn end_button(x: i32, y: i32) -> bool {
    let (bx, by) = end_button_rect();
    (bx..bx + 60).contains(&x) && (by..by + 22).contains(&y)
}

fn center(p: Pos) -> (i32, i32) {
    (p.x * TILE + 8, p.y * TILE + 8)
}

fn blit_scaled(fb: &mut Fb, s: &Sprite, x: i32, y: i32, k: i32) {
    for sy in 0..s.h {
        for sx in 0..s.w {
            let c = s.px[(sy * s.w + sx) as usize];
            if c != 0 {
                fb.rect(x + sx * k, y + sy * k, k, k, c);
            }
        }
    }
}

/// Track on one tile: a bed with ties and two rails along each arm, meeting in the middle.
/// On a switch the unlit arm is dimmed and barred.
fn draw_rail(fb: &mut Fb, p: Pos, m: u8, lit: u8) {
    let (x, y) = (p.x * TILE, p.y * TILE);
    let switch = m.count_ones() == 3;
    let beds = |d: Dir| match d {
        Dir::N => (x + 5, y, 6, 11),
        Dir::S => (x + 5, y + 5, 6, 11),
        Dir::E => (x + 5, y + 5, 11, 6),
        Dir::W => (x, y + 5, 11, 6),
    };
    for d in arms(m) {
        let (bx, by, bw, bh) = beds(d);
        fb.rect(bx, by, bw, bh, BED);
    }
    for d in arms(m) {
        let (bx, by, bw, bh) = beds(d);
        if bw == 6 {
            for ty in (by..by + bh).filter(|v| (v - y) % 3 == 1) {
                fb.rect(x + 4, ty, 8, 1, TIE);
            }
        } else {
            for tx in (bx..bx + bw).filter(|v| (v - x) % 3 == 1) {
                fb.rect(tx, y + 4, 1, 8, TIE);
            }
        }
    }
    for d in arms(m) {
        let (bx, by, bw, bh) = beds(d);
        let on = !switch || lit & d.bit() != 0;
        let col = if !switch {
            RAIL
        } else if on {
            RAIL_HI
        } else {
            RAIL_DIM
        };
        if bw == 6 {
            fb.rect(bx + 1, by, 1, bh, col);
            fb.rect(bx + 4, by, 1, bh, col);
        } else {
            fb.rect(bx, by + 1, bw, 1, col);
            fb.rect(bx, by + 4, bw, 1, col);
        }
        if !on {
            match d {
                Dir::N => fb.rect(x + 4, y + 2, 8, 2, STOP),
                Dir::S => fb.rect(x + 4, y + 12, 8, 2, STOP),
                Dir::E => fb.rect(x + 12, y + 4, 2, 8, STOP),
                Dir::W => fb.rect(x + 2, y + 4, 2, 8, STOP),
            }
        }
    }
}

/// The engine and its cars, each slid toward the tile it is moving into by whole pixels.
/// Passengers who have boarded but not yet pulled onto the line wait on the trail behind.
fn draw_train(fb: &mut Fb, sim: &Sim, route: &plan::Route) {
    let tt = if sim.express {
        sim.p.express_ticks
    } else {
        sim.p.tile_ticks
    };
    let moving = !sim.stalled
        && !sim.hold
        && !sim.parked
        && sim.crash_timer == 0
        && sim.next_tile().is_some();
    let frac = if moving { sim.progress.min(tt) } else { 0 };
    let slide = |from: Pos, to: Pos| {
        let (cx, cy) = center(from);
        (
            cx + (to.x - from.x) * TILE * frac / tt,
            cy + (to.y - from.y) * TILE * frac / tt,
        )
    };
    // every car with where it is and where it is going, front to back
    let n = sim.body.len();
    let mut cars: Vec<(Pos, Pos)> = (1..n).map(|k| (sim.body[k], sim.body[k - 1])).collect();
    let mut ahead = sim.body[n - 1];
    for &t in sim.trail.iter().take(sim.cars.len().saturating_sub(n - 1)) {
        cars.push((t, ahead));
        ahead = t;
    }
    for (i, &(from, to)) in cars.iter().enumerate().rev() {
        let (cx, cy) = slide(from, to);
        fb.rect(cx - 5, cy - 5, 10, 10, INK);
        fb.rect(cx - 4, cy - 4, 8, 8, CAR);
        fb.rect(cx - 4, cy - 4, 8, 1, CAR_HI);
        fb.rect(cx - 4, cy + 3, 8, 1, CAR_DK);
        if let Some(&c) = sim.cars.get(i) {
            fb.shape(c, cx, cy, 2, SHAPE[c as usize]);
        }
        if i + 1 == cars.len() {
            // the rear car's lamp: this is the one that gets off next
            let back = to.dir_to(from).unwrap_or(Dir::W);
            let (dx, dy) = back.delta();
            fb.rect(cx + dx * 4 - 1, cy + dy * 4 - 1, 2, 2, STOP);
        }
    }
    let e = sim.engine();
    let (cx, cy) = match route.tiles.first().filter(|_| moving) {
        Some(&t) => slide(e, t),
        None => center(e),
    };
    fb.rect(cx - 6, cy - 6, 12, 12, INK);
    fb.rect(cx - 5, cy - 5, 10, 10, ENGINE);
    fb.rect(cx - 5, cy - 5, 10, 1, ENGINE_HI);
    fb.rect(cx - 5, cy - 5, 1, 10, ENGINE_HI);
    fb.rect(cx - 5, cy + 4, 10, 1, ENGINE_DK);
    fb.rect(cx - 2, cy - 2, 4, 4, ENGINE_DK);
    let front = sim.entry.opp();
    let (dx, dy) = front.delta();
    fb.rect(cx + dx * 4 - 1, cy + dy * 4 - 1, 2, 2, WINDOW);
    if sim.express {
        let (bx, by) = (cx - dx * 9, cy - dy * 9);
        fb.checker(bx - 1, by - 1, 3, 3, TEXT_DIM);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn press(g: &mut Game, x: i32, y: i32) {
        g.push(In::Down(0, x, y));
        g.frame();
        g.push(In::Up(0, x, y));
        g.frame();
    }

    #[test]
    fn the_tutorial_holds_the_train_until_rail_reaches_the_first_station() {
        let mut g = Game::new(289);
        press(&mut g, 200, 150);
        assert_eq!(g.screen, Screen::Play);
        g.set_song_tick(60);
        g.frame();
        assert!(g.sim.hold);
        assert_eq!(g.sim.engine(), Pos::new(1, 8));
        // drag from the engine east one tile, then north to station a at (2, 2)
        g.push(In::Down(0, 1 * 16 + 8, 8 * 16 + 8));
        g.push(In::Move(2 * 16 + 8, 8 * 16 + 8));
        g.push(In::Move(2 * 16 + 8, 2 * 16 + 8));
        g.push(In::Up(0, 2 * 16 + 8, 2 * 16 + 8));
        g.frame();
        assert!(
            !g.sim.hold,
            "the signal clears once rail reaches the station"
        );
        g.set_song_tick(60 + 9 * 8);
        for _ in 0..20 {
            g.frame();
        }
        assert_eq!(
            g.sim.cars,
            vec![1],
            "boarded the passenger for the ◆ station"
        );
    }

    #[test]
    fn a_fast_drag_fills_every_tile_between_samples() {
        let mut g = Game::new(289);
        press(&mut g, 200, 150);
        g.push(In::Down(0, 5 * 16, 0));
        g.push(In::Move(12 * 16, 0));
        g.push(In::Up(0, 12 * 16, 0));
        g.frame();
        for x in 5..12 {
            assert_ne!(
                g.sim.mask[Pos::new(x, 0).idx()] & Dir::E.bit(),
                0,
                "gap at {x}"
            );
        }
    }

    #[test]
    fn the_cursor_is_opaque_or_clear_and_scales_exactly() {
        let mut g = Game::new(289);
        let (px, w, h) = g.cursor(3);
        assert_eq!((w, h), (30, 48));
        assert!(px.chunks(4).all(|c| c[3] == 0 || c[3] == 255));
        assert_eq!(px[3], 255, "the tip is drawn");
    }
}
