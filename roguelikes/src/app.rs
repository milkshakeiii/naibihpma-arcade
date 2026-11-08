//! The shell around the two games: launcher, pause menu, feedback form, save file and the
//! playtest log.

use crate::audio::{Sfx, Track, MUSIC_LEVEL_NAMES};
use crate::gulp;
use crate::hatchery;
use crate::form::{Form, Question};
use crate::framebuffer::{wrap, Framebuffer};
use crate::palette::{self, *};
use crate::ui::{self, Rect};
use std::collections::BTreeMap;
use std::io::Write;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Key {
    Up,
    Down,
    Left,
    Right,
    Enter,
    Space,
    Esc,
    Back,
    Tab,
    Help,
    Rest,
    Fast,
    Undo,
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Ev {
    Key(Key),
    Char(char),
    Move(i32, i32),
    Press { x: i32, y: i32, right: bool },
    Release { x: i32, y: i32, right: bool },
}

/// What a game asks of the shell each frame.
#[derive(Default)]
pub struct Out {
    pub sfx: Vec<Sfx>,
    pub music: Option<Track>,
    pub log: Vec<String>,
}

impl Out {
    pub fn sfx(&mut self, s: Sfx) {
        self.sfx.push(s);
    }
    pub fn log(&mut self, s: impl Into<String>) {
        self.log.push(s.into());
    }
}

/// How a run ended, for the form heading and the log.
pub struct Summary {
    pub headline: String,
    pub detail: String,
    pub best_key: u32,
}

/// The interface both games implement.
pub trait Run {
    fn event(&mut self, ev: Ev, out: &mut Out);
    fn tick(&mut self, out: &mut Out);
    fn draw(&self, fb: &mut Framebuffer);
    fn palette(&self) -> &Pal;
    fn ended(&self) -> bool;
    fn summary(&self) -> Summary;
    /// The game has an overlay open that Esc should close instead of pausing.
    fn takes_esc(&self) -> bool {
        false
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum GameId {
    Gulp,
    Hatchery,
}

impl GameId {
    /// Save-file prefix.
    pub fn key(self) -> &'static str {
        match self {
            GameId::Gulp => "gulp",
            GameId::Hatchery => "hatchery",
        }
    }
    pub fn letter(self) -> &'static str {
        match self {
            GameId::Gulp => "A",
            GameId::Hatchery => "B",
        }
    }
    pub fn name(self) -> &'static str {
        match self {
            GameId::Gulp => "GULP",
            GameId::Hatchery => "HATCHERY",
        }
    }
    fn version(self) -> u32 {
        match self {
            GameId::Gulp => gulp::VERSION,
            GameId::Hatchery => hatchery::VERSION,
        }
    }
    fn lesson_version(self) -> u32 {
        match self {
            GameId::Gulp => gulp::LESSON_VERSION,
            GameId::Hatchery => hatchery::LESSON_VERSION,
        }
    }
    fn special(self) -> &'static Question {
        match self {
            GameId::Gulp => &gulp::FORM_QUESTION,
            GameId::Hatchery => &hatchery::FORM_QUESTION,
        }
    }
}

/// `key=value` lines.
pub struct Save {
    path: Option<PathBuf>,
    kv: BTreeMap<String, String>,
}

impl Save {
    pub fn load(path: Option<PathBuf>) -> Self {
        let mut kv = BTreeMap::new();
        if let Some(text) = path.as_ref().and_then(|p| std::fs::read_to_string(p).ok()) {
            for line in text.lines() {
                if let Some((k, v)) = line.split_once('=') {
                    kv.insert(k.trim().to_string(), v.trim().to_string());
                }
            }
        }
        Self { path, kv }
    }
    pub fn get(&self, k: &str) -> u32 {
        self.kv.get(k).and_then(|v| v.parse().ok()).unwrap_or(0)
    }
    pub fn get_str(&self, k: &str) -> Option<&str> {
        self.kv.get(k).map(|s| s.as_str())
    }
    pub fn set(&mut self, k: &str, v: impl ToString) {
        self.kv.insert(k.to_string(), v.to_string());
        if let Some(p) = &self.path {
            let text: String = self.kv.iter().map(|(k, v)| format!("{k}={v}\n")).collect();
            let _ = std::fs::write(p, text);
        }
    }
}

pub struct Logger {
    path: Option<PathBuf>,
}

impl Logger {
    pub fn line(&self, s: &str) {
        if let Some(p) = &self.path {
            if let Ok(mut f) = std::fs::OpenOptions::new().create(true).append(true).open(p) {
                let _ = writeln!(f, "{s}");
            }
        }
    }
}

pub fn unix_now() -> u64 {
    SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_secs()).unwrap_or(0)
}

/// `YYYY-MM-DD HH:MM UTC` without a date crate (civil-from-days).
pub fn utc_stamp(secs: u64) -> String {
    let days = (secs / 86_400) as i64;
    let rem = secs % 86_400;
    let z = days + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z - era * 146_097;
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let y = yoe + era * 400 + if m <= 2 { 1 } else { 0 };
    format!("{y:04}-{m:02}-{d:02} {:02}:{:02} UTC", rem / 3600, rem % 3600 / 60)
}

enum Screen {
    Title,
    Play(GameId, Box<dyn Run>),
    Form(GameId, Box<Form>),
}

const CARD_A: Rect = Rect::new(10, 92, 184, 150);
const CARD_B: Rect = Rect::new(206, 92, 184, 150);
const LESSON_A: Rect = Rect::new(22, 220, 160, 14);
const LESSON_B: Rect = Rect::new(218, 220, 160, 14);
const PAUSE_ITEMS: [&str; 3] = ["KEEP PLAYING", "END THIS RUN (AND RATE IT)", "QUIT TO WINDOWS"];

pub struct App {
    screen: Screen,
    pub save: Save,
    log: Logger,
    pub out: Out,
    pub crt: bool,
    pub quit: bool,
    pub music_level: usize,
    pub music_changed: bool,
    mouse: (i32, i32),
    paused: Option<usize>,
    shell_pal: Pal,
    frame: u64,
    run_frames: u64,
}

impl App {
    pub fn new(save_path: Option<PathBuf>, log_path: Option<PathBuf>) -> Self {
        let save = Save::load(save_path);
        let music_level = save.get_str("music_level").and_then(|v| v.parse().ok()).unwrap_or(2usize).min(3);
        let crt = save.get_str("crt") != Some("0");
        let mut app = Self {
            screen: Screen::Title,
            save,
            log: Logger { path: log_path },
            out: Out::default(),
            crt,
            quit: false,
            music_level,
            music_changed: false,
            mouse: (-1, -1),
            paused: None,
            shell_pal: palette::shell(),
            frame: 0,
            run_frames: 0,
        };
        app.out.music = Some(Track::Menu);
        app
    }

    pub fn log_line(&self, s: &str) {
        self.log.line(s);
    }

    pub fn palette(&self) -> &Pal {
        match &self.screen {
            Screen::Play(_, g) => g.palette(),
            _ => &self.shell_pal,
        }
    }

    pub fn toggle_crt(&mut self) {
        self.crt = !self.crt;
        self.save.set("crt", if self.crt { 1 } else { 0 });
    }

    pub fn cycle_music(&mut self) {
        self.music_level = (self.music_level + 3) % 4; // HIGH -> MEDIUM -> LOW -> OFF -> HIGH
        self.save.set("music_level", self.music_level);
        self.music_changed = true;
    }

    pub fn start(&mut self, id: GameId, force_lesson: bool) {
        let runs = self.save.get(&format!("{}_runs", id.key()));
        let seen = self.save.get(&format!("{}_lesson", id.key()));
        let lesson = force_lesson || seen < id.lesson_version();
        let seed = unix_now() ^ (self.frame << 20) ^ 0x5DEE_CE66;
        self.log.line("");
        self.log.line(&format!(
            "=== {} v{} | run {} | {} | lesson: {} | seed {seed} ===",
            id.name(),
            id.version(),
            runs + 1,
            utc_stamp(unix_now()),
            if lesson { "yes" } else { "no" }
        ));
        let game: Box<dyn Run> = match id {
            GameId::Gulp => Box::new(gulp::Game::new(seed, lesson)),
            GameId::Hatchery => Box::new(hatchery::Game::new(seed, lesson)),
        };
        self.run_frames = 0;
        self.screen = Screen::Play(id, game);
        self.paused = None;
    }

    fn end_run(&mut self, abandoned: bool) {
        let Screen::Play(id, game) = &self.screen else { return };
        let id = *id;
        let s = game.summary();
        let mins = self.run_frames / 15 / 60;
        let secs = self.run_frames / 15 % 60;
        self.log.line(&format!(
            "--- {} after {mins}:{secs:02}: {} | {}",
            if abandoned { "ABANDONED" } else { "RUN OVER" },
            s.headline,
            s.detail
        ));
        let runs = self.save.get(&format!("{}_runs", id.key())) + 1;
        self.save.set(&format!("{}_runs", id.key()), runs);
        let best_k = format!("{}_best", id.key());
        if s.best_key > self.save.get(&best_k) {
            self.save.set(&best_k, s.best_key);
        }
        self.save.set(&format!("{}_lesson", id.key()), id.lesson_version().max(self.save.get(&format!("{}_lesson", id.key()))));
        let heading = format!("{} - RUN {runs} OVER. HOW WAS IT?", id.name());
        let summary = format!("{} {}", s.headline, s.detail);
        self.screen = Screen::Form(id, Box::new(Form::new(heading, summary, id.special())));
        self.paused = None;
    }

    /// Called when the window closes; records an unfinished run.
    pub fn shutdown(&mut self) {
        if let Screen::Play(_, g) = &self.screen {
            let s = g.summary();
            self.log.line(&format!("--- WINDOW CLOSED MID-RUN: {} | {}", s.headline, s.detail));
        }
    }

    pub fn event(&mut self, ev: Ev) {
        if let Ev::Move(x, y) | Ev::Press { x, y, .. } | Ev::Release { x, y, .. } = ev {
            self.mouse = (x, y);
        }
        match &mut self.screen {
            Screen::Title => self.title_event(ev),
            Screen::Form(id, form) => {
                form.event(ev, &mut self.out.sfx);
                if form.done {
                    let id = *id;
                    self.log.line(&format!("--- FEEDBACK ({} v{}) ---", id.name(), id.version()));
                    for l in form.report() {
                        self.log.line(&l);
                    }
                    self.screen = Screen::Title;
                    self.out.music = Some(Track::Menu);
                }
            }
            Screen::Play(_, game) => {
                if let Some(sel) = self.paused {
                    let pick = match ev {
                        Ev::Key(Key::Esc) => Some(0),
                        Ev::Key(Key::Up) => {
                            self.paused = Some((sel + 2) % 3);
                            None
                        }
                        Ev::Key(Key::Down) | Ev::Key(Key::Tab) => {
                            self.paused = Some((sel + 1) % 3);
                            None
                        }
                        Ev::Key(Key::Enter) | Ev::Key(Key::Space) => Some(sel),
                        Ev::Move(x, y) => {
                            if let Some(i) = (0..3).find(|&i| pause_rect(i).contains(x, y)) {
                                self.paused = Some(i);
                            }
                            None
                        }
                        Ev::Press { x, y, right: false } => (0..3).find(|&i| pause_rect(i).contains(x, y)),
                        _ => None,
                    };
                    match pick {
                        Some(0) => {
                            self.paused = None;
                            self.out.sfx(Sfx::Click);
                        }
                        Some(1) => {
                            self.out.sfx(Sfx::Click);
                            self.end_run(true);
                        }
                        Some(2) => {
                            self.shutdown();
                            self.quit = true;
                        }
                        _ => {}
                    }
                    return;
                }
                if ev == Ev::Key(Key::Esc) && !game.takes_esc() {
                    self.paused = Some(0);
                    self.out.sfx(Sfx::Click);
                    return;
                }
                game.event(ev, &mut self.out);
            }
        }
    }

    fn title_event(&mut self, ev: Ev) {
        match ev {
            Ev::Press { x, y, right: false } => {
                let seen = |id: GameId| self.save.get(&format!("{}_lesson", id.key())) >= id.lesson_version();
                if LESSON_A.contains(x, y) && seen(GameId::Gulp) {
                    self.out.sfx(Sfx::Click);
                    self.start(GameId::Gulp, true);
                } else if LESSON_B.contains(x, y) && seen(GameId::Hatchery) {
                    self.out.sfx(Sfx::Click);
                    self.start(GameId::Hatchery, true);
                } else if CARD_A.contains(x, y) {
                    self.out.sfx(Sfx::Click);
                    self.start(GameId::Gulp, false);
                } else if CARD_B.contains(x, y) {
                    self.out.sfx(Sfx::Click);
                    self.start(GameId::Hatchery, false);
                }
            }
            Ev::Char('a') | Ev::Char('A') => self.start(GameId::Gulp, false),
            Ev::Char('b') | Ev::Char('B') => self.start(GameId::Hatchery, false),
            Ev::Key(Key::Esc) => self.quit = true,
            _ => {}
        }
    }

    pub fn tick(&mut self) {
        self.frame += 1;
        if let Screen::Play(_, game) = &mut self.screen {
            if self.paused.is_none() {
                self.run_frames += 1;
                game.tick(&mut self.out);
            }
            let ended = game.ended();
            if !self.out.log.is_empty() {
                let t = self.run_frames / 15;
                for l in self.out.log.drain(..) {
                    self.log.line(&format!("[{:>2}:{:02}] {l}", t / 60, t % 60));
                }
            }
            if ended {
                self.end_run(false);
            }
        }
    }

    pub fn draw(&self, fb: &mut Framebuffer) {
        match &self.screen {
            Screen::Title => self.draw_title(fb),
            Screen::Form(_, f) => f.draw(fb),
            Screen::Play(_, g) => {
                g.draw(fb);
                if let Some(sel) = self.paused {
                    fb.rect_checker(0, 0, 400, 300, OUTLINE, 0);
                    ui::panel(fb, Rect::new(90, 96, 220, 104));
                    fb.text_centered("PAUSED", 200, 104, ACCENT_HI, Some(OUTLINE), 1);
                    for i in 0..3 {
                        ui::button(fb, pause_rect(i), PAUSE_ITEMS[i], sel == i, true);
                    }
                    fb.text_centered("ESC KEEPS PLAYING", 200, 188, TEXT_DIM, None, 1);
                }
            }
        }
    }

    fn draw_title(&self, fb: &mut Framebuffer) {
        fb.clear(2);
        for y in 0..300 {
            if y % 3 == 0 {
                fb.rect_fill(0, y, 400, 1, 3);
            }
        }
        fb.text_centered("TWO ROGUELIKES", 200, 10, ACCENT_HI, Some(OUTLINE), 2);
        let note = "HENRY: ROUND 3. TWO NEW GAMES IN THE FOREST OF NAIBIHPMA, BUILT FRESH FROM YOUR NOTES: A FROG WITH A TONGUE, AND A HATCHERY OF FAMILIARS YOU DESIGN AND RAISE. BOTH TEACH THEMSELVES; EVERY RUN ENDS WITH THE FORM.";
        for (i, l) in wrap(note, 62).iter().enumerate() {
            fb.text(l, 14, 36 + i as i32 * 10, TEXT_DIM, None, 1);
        }
        let hover = |r: Rect| r.contains(self.mouse.0, self.mouse.1);
        for (id, r, lesson_r) in [(GameId::Gulp, CARD_A, LESSON_A), (GameId::Hatchery, CARD_B, LESSON_B)] {
            let k = id.key();
            let h = hover(r) && !hover(lesson_r);
            if h {
                fb.rect_bevel(r.x, r.y, r.w, r.h, PANEL_HI, WHITE, PANEL);
            } else {
                ui::panel(fb, r);
            }
            let (title_col, sub_col) = if h { (PANEL_DEEP, PANEL_DEEP) } else { (ACCENT_HI, TEXT) };
            fb.text(&format!("{}  {}", id.letter(), id.name()), r.x + 8, r.y + 8, title_col, if h { None } else { Some(OUTLINE) }, 2);
            let (tag, how) = match id {
                GameId::Gulp => ("A FROG WITH A TONGUE. EAT, SPIT, GROW.", "ONE FROG, ONE POND AT A TIME. EVERY MOVE IS YOURS."),
                GameId::Hatchery => ("HATCH FAMILIARS GENE BY GENE, AND RAISE THEM.", "DESIGN THEM, THEN WATCH THEM DEFEND THE EGGS."),
            };
            let mut y = r.y + 30;
            for l in wrap(tag, 28) {
                fb.text(&l, r.x + 8, y, sub_col, None, 1);
                y += 10;
            }
            y += 4;
            for l in wrap(how, 28) {
                fb.text(&l, r.x + 8, y, if h { PANEL_DEEP } else { TEXT_DIM }, None, 1);
                y += 10;
            }
            draw_card_icon(fb, id, r.x + r.w - 52, r.y + 84);
            let runs = self.save.get(&format!("{k}_runs"));
            let seen = self.save.get(&format!("{k}_lesson")) >= id.lesson_version();
            let status = if runs == 0 { "NOT PLAYED YET".to_string() } else { format!("RUNS: {runs}") };
            fb.text(&status, r.x + 8, r.y + 96, if h { PANEL_DEEP } else { GOLD }, None, 1);
            if !seen {
                fb.text("STARTS WITH A LESSON", r.x + 8, r.y + 108, if h { PANEL_DEEP } else { TEXT_DIM }, None, 1);
            }
            if seen {
                ui::button(fb, lesson_r, "REPLAY THE LESSON", hover(lesson_r), true);
            } else {
                fb.text_centered("CLICK TO PLAY", r.x + r.w / 2, lesson_r.y + 3, if h { PANEL_DEEP } else { ACCENT }, None, 1);
            }
        }
        fb.text_centered("CLICK A GAME, OR PRESS A OR B", 200, 250, TEXT, None, 1);
        let footer = format!(
            "F2 SCANLINES {}   F3 MUSIC {}   ESC QUIT",
            if self.crt { "ON" } else { "OFF" },
            MUSIC_LEVEL_NAMES[self.music_level]
        );
        fb.text_centered(&footer, 200, 266, TEXT_DIM, None, 1);
        fb.text_centered("MUSIC: DELTA, FROM ABSTRACT OBJECTS", 200, 282, TEXT_FAINT, None, 1);
    }
}

fn pause_rect(i: usize) -> Rect {
    Rect::new(104, 118 + i as i32 * 22, 192, 18)
}

/// Small shell-palette emblems: a frog shooting its tongue at a fly, and a familiar's gene
/// slots beside a clutch of eggs.
fn draw_card_icon(fb: &mut Framebuffer, id: GameId, x: i32, y: i32) {
    match id {
        GameId::Gulp => {
            fb.rect_fill(x, y, 42, 42, OUTLINE);
            fb.rect_fill(x + 1, y + 1, 40, 40, 18);
            fb.rect_fill(x + 1, y + 28, 40, 13, 12);
            // The frog.
            fb.rect_fill(x + 5, y + 26, 14, 8, OUTLINE);
            fb.rect_fill(x + 6, y + 27, 12, 6, 13);
            fb.rect_fill(x + 6, y + 23, 4, 4, OUTLINE);
            fb.rect_fill(x + 14, y + 23, 4, 4, OUTLINE);
            fb.rect_fill(x + 7, y + 24, 2, 2, WHITE);
            fb.rect_fill(x + 15, y + 24, 2, 2, WHITE);
            // Tongue to a fly.
            fb.line(x + 18, y + 29, x + 33, y + 12, 16);
            fb.line(x + 19, y + 29, x + 34, y + 12, 16);
            fb.rect_fill(x + 33, y + 9, 4, 3, OUTLINE);
            fb.pset(x + 32, y + 8, WHITE);
            fb.pset(x + 37, y + 8, WHITE);
        }
        GameId::Hatchery => {
            for (k, l) in ["H", "S", "L", "T"].iter().enumerate() {
                let yy = y + k as i32 * 11;
                fb.rect_fill(x, yy, 24, 10, OUTLINE);
                fb.rect_fill(x + 1, yy + 1, 22, 8, PANEL_DEEP);
                fb.text(l, x + 2, yy + 1, ACCENT, None, 1);
                fb.rect_fill(x + 10, yy + 3, 11, 4, [13, 16, 19, 14][k]);
            }
            for (ex, ey) in [(30, 26), (36, 26), (33, 32), (30, 38), (36, 38)] {
                fb.rect_fill(x + ex, y + ey - 6, 6, 7, OUTLINE);
                fb.rect_fill(x + ex + 1, y + ey - 5, 4, 5, WHITE);
                fb.pset(x + ex + 2, y + ey - 3, OUTLINE);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn utc_stamp_known_dates() {
        assert_eq!(utc_stamp(0), "1970-01-01 00:00 UTC");
        assert_eq!(utc_stamp(1_794_096_000), "2026-11-08 00:00 UTC");
    }
}
