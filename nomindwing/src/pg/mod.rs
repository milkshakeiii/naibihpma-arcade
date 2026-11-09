//! The proving ground: draw ships in the shipyard, set them loose in the arena, and see whether
//! they fly and fight the way their drawings suggest. Round 2: connected ships, plate, the cannon
//! and a scrolling arena. Round 4: ships up to 99x99 and a yard with tools for drawing them.

pub mod design;
mod icons;
pub mod sim;
mod yard;

use self::design::*;
use self::sim::*;
use self::yard::{Ed, NAME_RECT};
use crate::app::{App, Ev, Key};
use crate::framebuffer::{wrap, Framebuffer};
use crate::geom::Line;
use crate::palette::{self, *};
use crate::ui::{self, Rect};
use std::fs::{File, OpenOptions};
use std::io::Write;
use std::path::PathBuf;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Screen {
    Yard,
    Arena,
}

/// A ready-made matchup: (team, slot, x, y) for each ship.
pub struct Test {
    pub name: &'static str,
    pub place: Vec<(usize, usize, i32, i32)>,
}

/// `n` ships of one design in a block centred on (cx, cy), `gap` px apart, up to 9 to a column.
pub fn block(team: usize, slot: usize, n: i32, cx: i32, cy: i32, gap: i32) -> Vec<(usize, usize, i32, i32)> {
    let rows = n.clamp(1, 9);
    let cols = (n + rows - 1) / rows;
    (0..n)
        .map(|k| {
            let (col, row) = (k / rows, k % rows);
            let in_col = if col == cols - 1 { n - col * rows } else { rows };
            let x = cx + col * gap - (cols - 1) * gap / 2;
            let y = cy + row * gap - (in_col - 1) * gap / 2;
            // Blue's first column faces the enemy; red mirrors it.
            let x = if team == 0 { 2 * cx - x } else { x };
            (team, slot, x, y)
        })
        .collect()
}

/// Slots: 0 fighter, 1 lancer, 2 frigate, 3 destroyer, 4 capital, 5 gunminer, 6 titan.
pub fn tests() -> Vec<Test> {
    let vs = |name: &'static str, blue: Vec<(usize, usize, i32, i32)>, red: Vec<(usize, usize, i32, i32)>| Test { name, place: blue.into_iter().chain(red).collect() };
    vec![
        vs("6 FIGHTERS VS 1 FRIGATE", block(0, 0, 6, 150, 180, 10), block(1, 2, 1, 330, 180, 0)),
        vs("6 LANCERS VS 1 FRIGATE", block(0, 1, 6, 150, 180, 10), block(1, 2, 1, 330, 180, 0)),
        vs("6 FIGHTERS VS 6 LANCERS", block(0, 0, 6, 150, 180, 10), block(1, 1, 6, 330, 180, 10)),
        vs("1 DESTROYER VS 4 FRIGATES", block(0, 3, 1, 150, 180, 0), block(1, 2, 4, 330, 180, 16)),
        vs("1 CAPITAL VS 30 FIGHTERS", block(0, 4, 1, 130, 180, 0), block(1, 0, 30, 350, 180, 9)),
        vs("1 CAPITAL VS 40 LANCERS", block(0, 4, 1, 130, 180, 0), block(1, 1, 40, 350, 180, 9)),
        vs("4 FIGHTERS VS 4 GUNMINERS", block(0, 0, 4, 120, 180, 12), block(1, 5, 4, 330, 180, 14)),
        vs("1 TITAN VS 5 CAPITALS", block(0, 6, 1, 110, 180, 0), block(1, 4, 5, 380, 180, 44)),
        vs("1 TITAN VS 40 LANCERS", block(0, 6, 1, 110, 180, 0), block(1, 1, 40, 350, 180, 9)),
        Test { name: "OPEN FIELD", place: Vec::new() },
    ]
}

#[derive(Clone, Copy, Debug)]
pub struct Placement {
    pub team: usize,
    pub slot: usize,
    pub x: i32,
    pub y: i32,
}

/// A notice over the arena, wrapped to fit inside the field with room for its panel.
fn notice_lines(s: &str) -> Vec<String> {
    wrap(s, ((FIELD.w - 8 - 12) / 6) as usize)
}

// Arena layout: the field is a 304x300 window onto the 480x360 arena.
const FIELD: Rect = Rect::new(0, 0, 304, 300);
const BTN_TEAM: [Rect; 2] = [Rect::new(306, 4, 44, 16), Rect::new(352, 4, 44, 16)];
const BTN_RUN: Rect = Rect::new(306, 149, 90, 15);
const BTN_RESET: Rect = Rect::new(306, 165, 44, 15);
const BTN_SPEED: Rect = Rect::new(352, 165, 44, 15);
const BTN_TEST: Rect = Rect::new(306, 181, 90, 15);
const BTN_EMPTY: Rect = Rect::new(306, 197, 44, 15);
const BTN_YARD: Rect = Rect::new(352, 197, 44, 15);
/// The minimap: the whole arena at exactly 1/6 scale.
const MINI_SCALE: i32 = 6;
const MINI: Rect = Rect::new(310, 213, AW / MINI_SCALE + 2, AH / MINI_SCALE + 2);
const CAM_STEP: i32 = 8;
const HELP_RECT: Rect = Rect::new(10, 8, 380, 284);
const QUIT_RECT: Rect = Rect::new(112, 108, 176, 70);
const QUIT_BTNS: [Rect; 2] = [Rect::new(124, 150, 72, 18), Rect::new(204, 150, 72, 18)];

fn roster_rect(i: usize) -> Rect {
    Rect::new(306, 22 + i as i32 * 18, 90, 17)
}

pub fn team_name(t: usize) -> &'static str {
    if t == 0 {
        "BLUE"
    } else {
        "RED"
    }
}

pub fn part_col(p: Part, team: usize) -> u8 {
    match p {
        Part::Hull => team_col(team, 1),
        Part::Plate => STEEL,
        Part::Flak => GUN,
        Part::Lance => LANCE,
        Part::Cannon => CANNON,
        Part::Engine => ENGINE,
        Part::Scoop => SCOOP,
        Part::Core => CORE,
    }
}

/// Which part shows when a downscaled icon has to pick one pixel for a block of cells.
fn icon_rank(p: Part) -> i32 {
    match p {
        Part::Core => 7,
        Part::Cannon => 6,
        Part::Lance => 5,
        Part::Flak => 4,
        Part::Engine => 3,
        Part::Scoop => 2,
        Part::Plate => 1,
        Part::Hull => 0,
    }
}

#[inline]
fn pc(fb: &mut Framebuffer, clip: Rect, x: i32, y: i32, c: u8) {
    if clip.contains(x, y) {
        fb.pset(x, y, c);
    }
}

/// A Bresenham line clipped to a rectangle.
fn clipped_line(fb: &mut Framebuffer, clip: Rect, x0: i32, y0: i32, x1: i32, y1: i32, c: u8, skip_first: bool) {
    let mut l = Line::new(x0, y0, x1, y1);
    let mut first = true;
    loop {
        if !(first && skip_first) {
            pc(fb, clip, l.x, l.y, c);
        }
        first = false;
        if (l.x, l.y) == (x1, y1) {
            break;
        }
        l.step();
    }
}

/// Draws a shape centred on (x, y): a 1px dark hull around the solid cells, the hull lit along
/// its top edge, each cannon's breech in gold, and checkerboard ghost dots where `ghost` says so.
pub fn draw_shape(fb: &mut Framebuffer, clip: Rect, sh: &Shape, x: i32, y: i32, facing: u8, team: usize, solid: &dyn Fn(usize) -> bool, ghost: bool) {
    let (w, h) = sh.dims(facing);
    let (l, t) = (x - w / 2, y - h / 2);
    let mut grid = vec![false; (w * h) as usize];
    for i in 0..sh.cells.len() {
        if sh.cells[i].is_some() && solid(i) {
            let (px, py) = sh.cell_xy(x, y, facing, i);
            grid[((py - t) * w + (px - l)) as usize] = true;
        }
    }
    if !ghost {
        for gy in 0..h {
            for gx in 0..w {
                if grid[(gy * w + gx) as usize] {
                    for (dx, dy) in [(1, 0), (-1, 0), (0, 1), (0, -1)] {
                        pc(fb, clip, l + gx + dx, t + gy + dy, OUTLINE);
                    }
                }
            }
        }
    }
    for i in 0..sh.cells.len() {
        let Some(p) = sh.cells[i] else { continue };
        if !solid(i) {
            continue;
        }
        let (px, py) = sh.cell_xy(x, y, facing, i);
        if ghost {
            if (px + py) & 1 == 0 {
                pc(fb, clip, px, py, team_col(team, 2));
            }
            continue;
        }
        let open_above = py - t == 0 || !grid[((py - t - 1) * w + (px - l)) as usize];
        let col = if p == Part::Hull && open_above {
            team_col(team, 2)
        } else if p == Part::Cannon && sh.cannons.contains(&i) {
            GOLD
        } else {
            part_col(p, team)
        };
        pc(fb, clip, px, py, col);
    }
}

/// A design's icon in a box: drawn 1:1 when it fits, otherwise at an exact 1/k scale where each
/// icon pixel shows the most telling part in its k x k block.
fn draw_icon(fb: &mut Framebuffer, d: &Design, r: Rect, team: usize) {
    fb.rect_fill(r.x, r.y, r.w, r.h, SPACE);
    let sh = d.shape();
    if sh.w <= r.w - 2 && sh.h <= r.h - 2 {
        draw_shape(fb, r, &sh, r.x + r.w / 2, r.y + r.h / 2, 0, team, &|_| true, false);
        return;
    }
    let k = ((sh.w + r.w - 3) / (r.w - 2)).max((sh.h + r.h - 3) / (r.h - 2)).max(1);
    let (iw, ih) = ((sh.w + k - 1) / k, (sh.h + k - 1) / k);
    let (ox, oy) = (r.x + (r.w - iw) / 2, r.y + (r.h - ih) / 2);
    for iy in 0..ih {
        for ix in 0..iw {
            let best = (0..k * k)
                .filter_map(|j| {
                    let (cx, cy) = (ix * k + j % k, iy * k + j / k);
                    if cx < sh.w && cy < sh.h {
                        sh.cells[(cy * sh.w + cx) as usize]
                    } else {
                        None
                    }
                })
                .max_by_key(|&p| icon_rank(p));
            if let Some(p) = best {
                pc(fb, r, ox + ix, oy + iy, if p == Part::Cannon { GOLD } else { part_col(p, team) });
            }
        }
    }
}

pub struct Pg {
    pub lib: Vec<Design>,
    pub screen: Screen,
    pub slot: usize,
    pub ed: Ed,
    edited: bool,
    naming: bool,
    pub arena: Arena,
    pub setup: Vec<Placement>,
    pub test: Option<usize>,
    tests: Vec<Test>,
    pub running: bool,
    pub speed: u32,
    pub team: usize,
    pub mouse: (i32, i32),
    /// Top-left of the field's window onto the arena.
    pub cam: (i32, i32),
    held: [bool; 4],
    latch: [bool; 4],
    map_drag: bool,
    pub help: bool,
    confirm_quit: bool,
    crt: bool,
    quit: bool,
    pal: Pal,
    save_path: Option<PathBuf>,
    log: Option<File>,
    logged_run: bool,
    logged_over: bool,
    notice: Option<(String, u32)>,
}

impl Pg {
    pub fn new(save_path: Option<PathBuf>, log_path: Option<PathBuf>) -> Pg {
        let mut lib = defaults();
        if let Some(p) = &save_path {
            if let Ok(text) = std::fs::read_to_string(p) {
                if matches!(text.lines().next(), Some(SAVE_HEADER | OLD_SAVE_HEADER)) {
                    for (i, d) in text.lines().skip(1).filter_map(Design::from_line).take(lib.len()).enumerate() {
                        lib[i] = d;
                    }
                    // A design still exactly like an old default was never edited: give it the new one.
                    let now = defaults();
                    for (slot, old) in retired_defaults() {
                        if lib[slot].to_line() == old.to_line() {
                            lib[slot] = now[slot].clone();
                        }
                    }
                } else if !text.trim().is_empty() {
                    // Round-1 designs use the old 15x15 grid and rules: keep them, start fresh.
                    let backup = p.with_file_name("proving_ground_designs_round1.txt");
                    if !backup.exists() {
                        let _ = std::fs::write(&backup, &text);
                    }
                }
            }
        }
        let log = log_path.and_then(|p| OpenOptions::new().create(true).append(true).open(p).ok());
        let mut pg = Pg {
            lib,
            screen: Screen::Arena,
            slot: 0,
            ed: Ed::new(),
            edited: false,
            naming: false,
            arena: Arena::new(),
            setup: Vec::new(),
            test: None,
            tests: tests(),
            running: false,
            speed: 1,
            team: 0,
            mouse: (0, 0),
            cam: ((AW - FIELD.w) / 2, (AH - FIELD.h) / 2),
            held: [false; 4],
            latch: [false; 4],
            map_drag: false,
            help: true,
            confirm_quit: false,
            crt: true,
            quit: false,
            pal: palette::game(),
            save_path,
            log,
            logged_run: false,
            logged_over: false,
            notice: None,
        };
        pg.log(&format!("##### proving ground round 4 session {} #####", crate::game::utc_stamp()));
        pg.load_test(0);
        pg.fit_view();
        pg
    }

    fn log(&mut self, s: &str) {
        if let Some(f) = self.log.as_mut() {
            let _ = writeln!(f, "{s}");
        }
    }

    fn save(&mut self) {
        if let Some(p) = &self.save_path {
            let text: String = std::iter::once(SAVE_HEADER.to_string() + "\n").chain(self.lib.iter().map(|d| d.to_line() + "\n")).collect();
            let _ = std::fs::write(p, text);
        }
    }

    fn say(&mut self, s: &str) {
        self.notice = Some((s.to_string(), 30));
    }

    pub fn load_test(&mut self, t: usize) {
        self.test = Some(t);
        self.setup = self.tests[t].place.iter().map(|&(team, slot, x, y)| Placement { team, slot, x, y }).collect();
        self.reset();
    }

    /// Rebuilds the arena from the setup with the current designs. The same setup always replays
    /// the same fight.
    pub fn reset(&mut self) {
        self.arena = Arena::new();
        let mut refused = None;
        for p in &self.setup {
            if !self.arena.add(p.team, p.slot, &self.lib[p.slot], p.x, p.y) {
                refused = Some(p.slot);
            }
        }
        if let Some(s) = refused {
            let why = self.lib[s].check().problem().unwrap_or_default();
            self.say(&format!("{} CAN'T FLY: {why}", self.lib[s].name));
        }
        self.running = false;
        self.logged_run = false;
        self.logged_over = false;
    }

    fn test_name(&self) -> &'static str {
        self.test.map_or("YOUR SETUP", |t| self.tests[t].name)
    }

    fn set_running(&mut self, on: bool) {
        if on && self.arena.over.is_some() {
            self.reset();
        }
        self.running = on;
        if on && !self.logged_run {
            self.logged_run = true;
            let side = |t: usize| {
                let mut counts: Vec<(usize, usize)> = Vec::new();
                for p in self.setup.iter().filter(|p| p.team == t) {
                    match counts.iter_mut().find(|(s, _)| *s == p.slot) {
                        Some(c) => c.1 += 1,
                        None => counts.push((p.slot, 1)),
                    }
                }
                counts.iter().map(|(s, n)| format!("{n} {}", self.lib[*s].name)).collect::<Vec<_>>().join(", ")
            };
            let line = format!("RUN {}: BLUE {} | RED {}", self.test_name(), side(0), side(1));
            self.log(&line);
            let mut slots: Vec<usize> = self.setup.iter().map(|p| p.slot).collect();
            slots.sort();
            slots.dedup();
            for s in slots {
                let line = format!("  DESIGN {}", self.lib[s].to_line());
                self.log(&line);
            }
        }
    }

    // ----- Input -----

    fn key(&mut self, k: Key) {
        if self.naming {
            match k {
                Key::Back => {
                    self.lib[self.slot].name.pop();
                }
                Key::Enter | Key::Esc | Key::Tab => self.end_naming(),
                _ => {}
            }
            return;
        }
        if self.help {
            self.help = false;
            return;
        }
        if self.confirm_quit {
            if k == Key::Esc || k == Key::Enter {
                self.quit = true;
            }
            self.confirm_quit = false;
            return;
        }
        match (self.screen, k) {
            (_, Key::Esc) => self.confirm_quit = true,
            (_, Key::Help) => self.help = true,
            (_, Key::Tab) => self.switch_screen(),
            (Screen::Yard, k) => self.key_yard(k),
            (Screen::Arena, Key::Num(n)) if (1..=self.lib.len()).contains(&(n as usize)) => self.slot = n as usize - 1,
            (Screen::Arena, Key::Space) => self.set_running(!self.running),
            (Screen::Arena, Key::Letter('R')) => self.reset(),
            (Screen::Arena, Key::Letter('F')) => self.cycle_speed(),
            (Screen::Arena, Key::Letter('N')) => self.next_test(),
            (Screen::Arena, Key::Letter('T')) => self.team = 1 - self.team,
            _ => {}
        }
    }

    fn cycle_speed(&mut self) {
        self.speed = match self.speed {
            1 => 2,
            2 => 4,
            _ => 1,
        };
    }

    fn next_test(&mut self) {
        let t = self.test.map_or(0, |t| (t + 1) % self.tests.len());
        self.load_test(t);
    }

    fn switch_screen(&mut self) {
        self.end_naming();
        self.screen = match self.screen {
            Screen::Yard => {
                if self.edited {
                    self.edited = false;
                    self.save();
                    let line = format!("EDITED {}", self.lib[self.slot].to_line());
                    self.log(&line);
                }
                // The arena replays with the new designs.
                self.reset();
                Screen::Arena
            }
            Screen::Arena => {
                self.running = false;
                self.frame_if_new();
                Screen::Yard
            }
        };
    }

    fn end_naming(&mut self) {
        if self.naming {
            self.naming = false;
            if self.lib[self.slot].name.trim().is_empty() {
                self.lib[self.slot].name = format!("DESIGN {}", self.slot + 1);
            }
            self.edited = true;
            self.save();
        }
    }

    fn to_world(&self, x: i32, y: i32) -> (i32, i32) {
        (x + self.cam.0, y + self.cam.1)
    }

    fn set_cam(&mut self, x: i32, y: i32) {
        self.cam = (x.clamp(0, AW - FIELD.w), y.clamp(0, AH - FIELD.h));
    }

    /// Centres the view on the arena point under a minimap click.
    fn cam_from_map(&mut self, x: i32, y: i32) {
        let (wx, wy) = ((x - MINI.x - 1) * MINI_SCALE, (y - MINI.y - 1) * MINI_SCALE);
        self.set_cam(wx - FIELD.w / 2, wy - FIELD.h / 2);
    }

    fn press(&mut self, x: i32, y: i32, right: bool) {
        if self.help {
            self.help = false;
            return;
        }
        if self.confirm_quit {
            if QUIT_BTNS[0].contains(x, y) {
                self.quit = true;
            }
            self.confirm_quit = false;
            return;
        }
        if self.naming && !NAME_RECT.contains(x, y) {
            self.end_naming();
        }
        match self.screen {
            Screen::Yard => self.press_yard(x, y, right),
            Screen::Arena => self.press_arena(x, y, right),
        }
    }

    fn press_arena(&mut self, x: i32, y: i32, right: bool) {
        if FIELD.contains(x, y) {
            if self.arena.tick > 0 {
                self.say("PRESS R TO RESET, THEN PLACE");
                return;
            }
            let (wx, wy) = self.to_world(x, y);
            if right {
                if let Some(i) = self.arena.ship_at(wx, wy) {
                    let (team, slot, (hx, hy)) = (self.arena.ships[i].team, self.arena.ships[i].slot, self.arena.ships[i].home);
                    let found = (0..self.setup.len())
                        .filter(|&k| self.setup[k].team == team && self.setup[k].slot == slot)
                        .min_by_key(|&k| crate::geom::dist2(self.setup[k].x, self.setup[k].y, hx, hy));
                    if let Some(k) = found {
                        self.setup.remove(k);
                        self.test = None;
                        self.reset();
                    }
                }
                return;
            }
            if let Some(why) = self.lib[self.slot].check().problem() {
                let s = format!("{} CAN'T FLY: {why}", self.lib[self.slot].name);
                self.say(&s);
                return;
            }
            self.setup.push(Placement { team: self.team, slot: self.slot, x: wx, y: wy });
            self.test = None;
            self.reset();
            return;
        }
        if right {
            return;
        }
        if MINI.contains(x, y) {
            self.map_drag = true;
            self.cam_from_map(x, y);
        } else if let Some(t) = (0..2).find(|&t| BTN_TEAM[t].contains(x, y)) {
            self.team = t;
        } else if let Some(i) = (0..self.lib.len()).find(|&i| roster_rect(i).contains(x, y)) {
            self.slot = i;
        } else if BTN_RUN.contains(x, y) {
            self.set_running(!self.running);
        } else if BTN_RESET.contains(x, y) {
            self.reset();
        } else if BTN_SPEED.contains(x, y) {
            self.cycle_speed();
        } else if BTN_TEST.contains(x, y) {
            self.next_test();
        } else if BTN_EMPTY.contains(x, y) {
            self.setup.clear();
            self.test = None;
            self.reset();
        } else if BTN_YARD.contains(x, y) {
            self.switch_screen();
        }
    }

    // ----- Tick -----

    fn step(&mut self) {
        if let Some((_, t)) = self.notice.as_mut() {
            *t = t.saturating_sub(1);
            if *t == 0 {
                self.notice = None;
            }
        }
        // Scroll the view with WASD or the arrows; a tap between ticks still counts.
        let dirs: Vec<bool> = (0..4).map(|k| self.held[k] || self.latch[k]).collect();
        self.latch = self.held;
        if self.screen == Screen::Arena && !self.help && !self.confirm_quit {
            let dx = (dirs[3] as i32 - dirs[2] as i32) * CAM_STEP;
            let dy = (dirs[1] as i32 - dirs[0] as i32) * CAM_STEP;
            if dx != 0 || dy != 0 {
                self.set_cam(self.cam.0 + dx, self.cam.1 + dy);
            }
        }
        if self.screen == Screen::Yard && !self.help && !self.confirm_quit && !self.naming {
            self.scroll_yard(&dirs);
        }
        if self.screen != Screen::Arena || !self.running || self.help || self.confirm_quit {
            return;
        }
        for _ in 0..self.speed {
            self.arena.step();
        }
        if let Some(result) = self.arena.over {
            self.running = false;
            if !self.logged_over {
                self.logged_over = true;
                let t = self.arena.tick / 15;
                let who = match result {
                    Some(team) => format!("{} WINS", team_name(team)),
                    None => "DRAW".into(),
                };
                let who = if self.arena.stalemate { "DRAW (STALEMATE)".to_string() } else { who };
                let s = &self.arena.score;
                let a = &self.arena;
                let side = |k: usize| {
                    format!(
                        "lost {}/{} ships ({} split off), {} px ({} broke away), {} bounced off plate, mined {}{}",
                        s[k].lost,
                        s[k].placed,
                        s[k].split,
                        s[k].px_lost,
                        s[k].broken,
                        s[1 - k].bounced,
                        s[k].mined,
                        if a.disarmed(k) { ", DISARMED" } else { "" }
                    )
                };
                let line = format!("RESULT {who} at {}:{:02} | blue {} | red {}", t / 60, t % 60, side(0), side(1));
                self.log(&line);
            }
        }
    }

    // ----- Drawing -----

    fn render(&self, fb: &mut Framebuffer) {
        match self.screen {
            Screen::Yard => self.draw_yard(fb),
            Screen::Arena => self.draw_arena(fb),
        }
        if let Some((s, _)) = &self.notice {
            let lines = notice_lines(s);
            let w = lines.iter().map(|l| Framebuffer::text_width(l, 1)).max().unwrap_or(0) + 12;
            let r = Rect::new((FIELD.w - w).max(4) / 2, 22, w, 6 + 9 * lines.len() as i32);
            ui::panel(fb, r);
            for (i, l) in lines.iter().enumerate() {
                fb.text(l, r.x + 6, r.y + 4 + 9 * i as i32, GOLD, None, 1);
            }
        }
        if self.help {
            self.draw_help(fb);
        } else if self.confirm_quit {
            ui::panel(fb, QUIT_RECT);
            fb.text_centered("QUIT THE PROVING GROUND?", QUIT_RECT.x + QUIT_RECT.w / 2, QUIT_RECT.y + 12, TEXT, None, 1);
            fb.text_centered("ESC AGAIN QUITS", QUIT_RECT.x + QUIT_RECT.w / 2, QUIT_RECT.y + 26, TEXT_FAINT, None, 1);
            for (r, l) in QUIT_BTNS.iter().zip(["QUIT", "BACK"]) {
                ui::button(fb, *r, l, r.contains(self.mouse.0, self.mouse.1), false);
            }
        }
    }

    fn draw_arena(&self, fb: &mut Framebuffer) {
        let a = &self.arena;
        let m = self.mouse;
        let (cx, cy) = self.cam;
        let f = FIELD;
        fb.rect_fill(f.x, f.y, f.w, f.h, SPACE);
        for &(x, y, c) in &a.stars {
            pc(fb, f, x - cx, y - cy, STAR + c);
        }
        for sy in 0..f.h {
            for sx in 0..f.w {
                let (x, y) = (sx + cx, sy + cy);
                if a.rock_at(x, y) {
                    let lit = !a.rock_at(x, y - 1) || !a.rock_at(x - 1, y);
                    let dark = !a.rock_at(x, y + 1) || !a.rock_at(x + 1, y);
                    fb.pset(sx, sy, if lit && !dark { ROCK + 3 } else if dark { ROCK } else { ROCK + 1 });
                }
            }
        }
        for (t, &(sx, sy)) in STATIONS.iter().enumerate() {
            let (sx, sy) = (sx - cx, sy - cy);
            for (x, y, w, h, c) in [(sx - 4, sy - 13, 9, 27, OUTLINE), (sx - 3, sy - 12, 7, 25, team_col(t, 0))] {
                for k in 0..w * h {
                    pc(fb, f, x + k % w, y + k / w, c);
                }
            }
            for k in 0..5 {
                for j in 0..10 {
                    pc(fb, f, sx - 2 + j % 5, sy - 10 + k * 5 + j / 5, team_col(t, 2));
                }
            }
        }
        for d in &a.debris {
            pc(fb, f, d.x - cx, d.y - cy, LOOSE);
        }
        for s in a.ships.iter().filter(|s| s.alive) {
            for &(x, y) in &s.trail {
                pc(fb, f, x - cx, y - cy, team_col(s.team, 0));
            }
        }
        let (wx, wy) = self.to_world(m.0, m.1);
        let hovered = if f.contains(m.0, m.1) { a.ship_at(wx, wy) } else { None };
        if let Some(i) = hovered {
            let s = &a.ships[i];
            if let Some(t) = s.target.filter(|&t| a.ships[t].alive) {
                crate::render::dotted(fb, f, s.x - cx, s.y - cy, a.ships[t].x - cx, a.ships[t].y - cy, 3, TEXT_FAINT);
            }
            if let Some((gx, gy)) = s.goal {
                for k in -2..=2 {
                    pc(fb, f, gx - cx + k, gy - cy, TEXT_DIM);
                    pc(fb, f, gx - cx, gy - cy + k, TEXT_DIM);
                }
            }
        }
        for s in a.ships.iter().filter(|s| s.alive) {
            draw_shape(fb, f, &s.shape, s.x - cx, s.y - cy, s.facing, s.team, &|c| s.hp[c] > 0, false);
            // Cannon barrels show where each turret is pointing.
            for t in s.turrets.iter().filter(|t| s.hp[t.cell] > 0) {
                let (gx, gy) = s.cell_xy(t.cell);
                let (ux, uy) = dir64(t.aim);
                let (ex, ey) = (gx + ux * BARREL / 1024, gy + uy * BARREL / 1024);
                clipped_line(fb, f, gx - cx, gy - cy, ex - cx, ey - cy, GUN, true);
            }
        }
        if let Some(i) = hovered {
            let (l, t, r, b) = a.ships[i].bbox();
            fb.corners(l - 2 - cx, t - 2 - cy, r - l + 5, b - t + 5, 2, WHITE);
        }
        for sh in &a.shots {
            let (p, q) = ((sh.prev.0 - cx, sh.prev.1 - cy), (sh.line.x - cx, sh.line.y - cy));
            let (c0, c1) = match sh.kind {
                Kind::Lance => (LANCE, LANCE_HI),
                Kind::Shell => (GOLD, WHITE),
                Kind::Flak => (team_col(sh.team, 1), team_col(sh.team, 3)),
            };
            pc(fb, f, p.0, p.1, c0);
            pc(fb, f, q.0, q.1, c1);
        }
        for sp in &a.sparks {
            let (x, y) = (sp.x - cx, sp.y - cy);
            if sp.blast {
                for k in 0..9 {
                    let (dx, dy) = (k % 3 - 1, k / 3 - 1);
                    let c = if sp.t >= 3 { if k == 4 { WHITE } else { GOLD } } else if dx == 0 || dy == 0 { SCOOP } else { continue };
                    pc(fb, f, x + dx, y + dy, c);
                }
            } else {
                pc(fb, f, x, y, if sp.t >= 2 { WHITE } else { GOLD });
            }
        }
        // Where a click would place the selected design.
        if a.tick == 0 && hovered.is_none() && f.contains(m.0, m.1) && !self.help {
            let sh = self.lib[self.slot].shape();
            draw_shape(fb, f, &sh, m.0, m.1, if self.team == 0 { 1 } else { 3 }, self.team, &|_| true, true);
        }
        fb.text(self.test_name(), 4, 3, TEXT_DIM, Some(OUTLINE), 1);
        let t = a.tick / 15;
        let clock = format!("{}:{:02}", t / 60, t % 60);
        fb.text(&clock, f.w - 4 - Framebuffer::text_width(&clock, 1), 3, TEXT_DIM, Some(OUTLINE), 1);

        // Inspector.
        let bar: Option<Vec<String>> = if let Some(i) = hovered {
            let s = &a.ships[i];
            let mut parts = vec![format!("{} PX/S", s.speed_px_s()), format!("{}/{} PX", s.px_now(), s.px_full())];
            for (n, name) in [(s.st.flak, "FLAK"), (s.st.lance, "LANCE"), (s.st.cannon, "CANNON")] {
                if n > 0 {
                    parts.push(format!("{name} {n}"));
                }
            }
            if s.carry > 0 {
                parts.push(format!("CARRYING {}", s.carry));
            }
            Some(vec![format!("{} {} - {} - {}", team_name(s.team), s.name, s.order.name(), s.doing.name()), parts.join("  ")])
        } else if a.tick == 0 && f.contains(m.0, m.1) {
            Some(vec![format!("CLICK PLACES A {} {}. RIGHT-CLICK REMOVES.", team_name(self.team), self.lib[self.slot].name), "SPACE RUNS THE FIGHT. WASD SCROLLS.".into()])
        } else {
            None
        };
        if let Some(lines) = bar {
            let h = lines.len() as i32 * 9 + 6;
            let r = Rect::new(0, f.h - h, f.w, h);
            ui::panel(fb, r);
            for (k, l) in lines.iter().enumerate() {
                fb.text(l, 4, r.y + 4 + k as i32 * 9, TEXT, None, 1);
            }
        }
        if let Some(result) = a.over {
            // A stalemate: 20 s with no shots, no hits and nobody closing in. Say who can't fight any more.
            let disarmed: Vec<&str> = (0..2).filter(|&t| a.disarmed(t)).map(team_name).collect();
            let extra = if a.stalemate && !disarmed.is_empty() { 10 } else { 0 };
            let r = Rect::new(62, 112, 180, 62 + extra);
            ui::panel(fb, r);
            let (head, col) = match result {
                Some(t) => (format!("{} WINS", team_name(t)), team_col(t, 2)),
                None if a.stalemate => ("STALEMATE".to_string(), TEXT),
                None => ("DRAW".to_string(), TEXT),
            };
            fb.text_centered(&head, r.x + r.w / 2, r.y + 8, col, Some(OUTLINE), 2);
            let when = if a.stalemate { format!("A DRAW AT {clock}") } else { format!("IN {clock}") };
            fb.text_centered(&when, r.x + r.w / 2, r.y + 28, TEXT, None, 1);
            if extra > 0 {
                fb.text_centered(&format!("{} DISARMED", disarmed.join(" AND ")), r.x + r.w / 2, r.y + 38, GOLD, None, 1);
            }
            fb.text_centered("SPACE OR R REPLAYS", r.x + r.w / 2, r.y + 44 + extra, TEXT_FAINT, None, 1);
        }

        // Sidebar.
        ui::panel(fb, Rect::new(304, 0, 96, 300));
        for t in 0..2 {
            let r = BTN_TEAM[t];
            if self.team == t {
                fb.rect_bevel(r.x, r.y, r.w, r.h, team_col(t, 0), team_col(t, 1), OUTLINE);
                fb.text_centered(team_name(t), r.x + r.w / 2, r.y + 5, WHITE, None, 1);
            } else {
                ui::button(fb, r, team_name(t), r.contains(m.0, m.1), false);
            }
        }
        for (i, d) in self.lib.iter().enumerate() {
            let r = roster_rect(i);
            let sel = i == self.slot;
            let hover = r.contains(m.0, m.1);
            if sel {
                fb.rect_bevel(r.x, r.y, r.w, r.h, PANEL_DEEP, PANEL_LO, PANEL_HI);
                fb.rect_outline(r.x + 1, r.y + 1, r.w - 2, r.h - 2, ACCENT);
            } else if hover {
                fb.rect_bevel(r.x, r.y, r.w, r.h, PANEL_HI, WHITE, PANEL);
            } else {
                ui::panel(fb, r);
            }
            draw_icon(fb, d, Rect::new(r.x + 2, r.y + 1, 14, 14), self.team);
            let ok = d.check().ok();
            let tc = if !ok { BAD } else if hover && !sel { PANEL_DEEP } else { TEXT };
            let name: String = d.name.chars().take(10).collect();
            fb.text(&name, r.x + 19, r.y + 1, tc, None, 1);
            let st = d.stats();
            fb.text(&format!("{} PX {}", st.cost, &d.order.name()[..4]), r.x + 19, r.y + 9, if hover && !sel { PANEL_LO } else { TEXT_DIM }, None, 1);
            fb.text(&format!("{}", i + 1), r.x + r.w - 8, r.y + 1, TEXT_FAINT, None, 1);
        }
        let run_label = if self.running { "PAUSE" } else if a.over.is_some() { "REPLAY" } else { "RUN" };
        ui::button(fb, BTN_RUN, run_label, BTN_RUN.contains(m.0, m.1), self.running);
        ui::button(fb, BTN_RESET, "RESET", BTN_RESET.contains(m.0, m.1), false);
        ui::button(fb, BTN_SPEED, &format!("{}X", self.speed), BTN_SPEED.contains(m.0, m.1), self.speed > 1);
        let tl = match self.test {
            Some(t) => format!("TEST {}/{}", t + 1, self.tests.len()),
            None => "NEXT TEST".into(),
        };
        ui::button(fb, BTN_TEST, &tl, BTN_TEST.contains(m.0, m.1), false);
        ui::button(fb, BTN_EMPTY, "EMPTY", BTN_EMPTY.contains(m.0, m.1), false);
        ui::button(fb, BTN_YARD, "YARD", BTN_YARD.contains(m.0, m.1), false);
        self.draw_minimap(fb);
        let sc = &a.score;
        for t in 0..2 {
            let y = 278 + t as i32 * 11;
            fb.text(team_name(t), 308, y, team_col(t, 2), None, 1);
            fb.text(&format!("{}/{}", a.alive(t), sc[t].placed), 338, y, TEXT, None, 1);
            fb.text(&format!("-{}", sc[t].px_lost), 368, y, TEXT_DIM, None, 1);
        }

        // Button tips go in the inspector's place when nothing in the field is hovered.
        if !f.contains(m.0, m.1) {
            let tip = if BTN_RUN.contains(m.0, m.1) {
                Some("[SPACE] RUN OR PAUSE THE FIGHT.")
            } else if BTN_RESET.contains(m.0, m.1) {
                Some("[R] PUT EVERY SHIP BACK WHERE IT STARTED. THE SAME SETUP ALWAYS PLAYS OUT THE SAME WAY.")
            } else if BTN_SPEED.contains(m.0, m.1) {
                Some("[F] 1X, 2X OR 4X. THE FRAMES STAY AT 15 FPS; FAST SKIPS AHEAD.")
            } else if BTN_TEST.contains(m.0, m.1) {
                Some("[N] LOAD THE NEXT READY-MADE MATCHUP.")
            } else if BTN_EMPTY.contains(m.0, m.1) {
                Some("REMOVE EVERY SHIP, FOR A SETUP OF YOUR OWN.")
            } else if BTN_YARD.contains(m.0, m.1) {
                Some("[TAB] THE SHIPYARD, TO DRAW AND EDIT DESIGNS.")
            } else if MINI.contains(m.0, m.1) {
                Some("THE WHOLE ARENA AT 1/6 SCALE. CLICK OR DRAG TO MOVE THE VIEW, OR SCROLL WITH WASD OR THE ARROWS.")
            } else if (0..2).any(|t| BTN_TEAM[t].contains(m.0, m.1)) {
                Some("[T] WHICH TEAM YOUR CLICKS PLACE.")
            } else if (306..400).contains(&m.0) && m.1 >= 276 {
                Some("SHIPS LEFT / PLACED, AND PIXELS LOST, FOR EACH TEAM.")
            } else {
                (0..self.lib.len()).find(|&i| roster_rect(i).contains(m.0, m.1)).map(|_| "[1-7] THE DESIGN YOUR CLICKS PLACE. EDIT IT IN THE YARD. A RED NAME CAN'T FLY YET.")
            };
            if let Some(tip) = tip {
                let lines = wrap(tip, 49);
                let h = lines.len() as i32 * 9 + 6;
                let r = Rect::new(0, f.h - h, f.w, h);
                ui::panel(fb, r);
                for (k, l) in lines.iter().enumerate() {
                    fb.text(l, 4, r.y + 4 + k as i32 * 9, TEXT, None, 1);
                }
            }
        }
    }

    fn draw_minimap(&self, fb: &mut Framebuffer) {
        let a = &self.arena;
        let r = MINI;
        ui::inset(fb, r);
        let (ox, oy) = (r.x + 1, r.y + 1);
        let inner = Rect::new(ox, oy, AW / MINI_SCALE, AH / MINI_SCALE);
        fb.rect_fill(ox, oy, inner.w, inner.h, SPACE);
        for my in 0..inner.h {
            for mx in 0..inner.w {
                if a.rock_at(mx * MINI_SCALE + MINI_SCALE / 2, my * MINI_SCALE + MINI_SCALE / 2) {
                    fb.pset(ox + mx, oy + my, ROCK + 1);
                }
            }
        }
        for (t, &(sx, sy)) in STATIONS.iter().enumerate() {
            for k in -2..=2 {
                pc(fb, inner, ox + sx / MINI_SCALE, oy + sy / MINI_SCALE + k, team_col(t, 0));
            }
        }
        for s in a.ships.iter().filter(|s| s.alive) {
            let n = (s.half() * 2 / MINI_SCALE).max(1);
            let (x, y) = (ox + s.x / MINI_SCALE - n / 2, oy + s.y / MINI_SCALE - n / 2);
            for k in 0..n * n {
                pc(fb, inner, x + k % n, y + k / n, team_col(s.team, 2));
            }
        }
        let (cx, cy) = self.cam;
        fb.rect_outline(ox + cx / MINI_SCALE, oy + cy / MINI_SCALE, FIELD.w / MINI_SCALE, FIELD.h / MINI_SCALE, WHITE);
    }

    fn draw_help(&self, fb: &mut Framebuffer) {
        let r = HELP_RECT;
        ui::panel(fb, r);
        fb.text("PROVING GROUND", r.x + 10, r.y + 8, ACCENT, Some(OUTLINE), 2);
        fb.text("ROUND 4", r.x + r.w - 52, r.y + 12, GOLD, None, 1);
        let body = [
            ("SHIPS", "DRAW A SHIP IN THE YARD. EVERY COLOUR IS A PART AND UP IS THE FRONT. AT LEAST ONE CORE, EVERY PIXEL JOINED UP EDGE TO EDGE, AND EVERY ENGINE WITH A STRAIGHT 3-WIDE WAY OUT."),
            ("NEW", "SHIPS UP TO 99X99. KEYS 1-6 PICK A PART CATEGORY. P PENCIL, R RECTANGLE, F FILL, C COPY A STAMP (OR RIGHT-CLICK A DESIGN'S TAB), T TURN IT, U UNDO. Q/E OR THE WHEEL ZOOM; 1X IS REAL SIZE."),
            ("FLYING", "AT LEAST 1 THRUST PER WEIGHT: ATTACK RUNS. LESS: IT HOLDS AT WEAPON RANGE. HEAVIER SHIPS TURN SLOWER."),
            ("ARENA", "PICK A TEAM AND A DESIGN, CLICK TO PLACE, RIGHT-CLICK TO REMOVE. SPACE RUNS, R REPLAYS. WASD, ARROWS OR THE MINIMAP SCROLL THE VIEW."),
            ("KEYS", "TAB YARD/ARENA  N NEXT TEST  F SPEED  T TEAM  F2 SCANLINES  F1 HELP  ESC QUIT"),
        ];
        let mut y = r.y + 34;
        for (head, text) in body {
            fb.text(head, r.x + 10, y, GOLD, None, 1);
            let lines = wrap(text, 50);
            for (i, l) in lines.iter().enumerate() {
                fb.text(l, r.x + 58, y + i as i32 * 9, TEXT, None, 1);
            }
            y += lines.len() as i32 * 9 + 6;
        }
        let ly = r.y + r.h - 40;
        for (k, p) in [Part::Flak, Part::Lance, Part::Cannon, Part::Plate, Part::Engine, Part::Core].iter().enumerate() {
            let x = r.x + 14 + k as i32 * 60;
            fb.rect_fill(x - 1, ly - 1, 6, 6, OUTLINE);
            fb.rect_fill(x, ly, 4, 4, part_col(*p, 0));
            fb.text(p.name(), x + 8, ly - 1, TEXT_DIM, None, 1);
        }
        fb.text_centered("CLICK OR PRESS ANY KEY", r.x + r.w / 2, r.y + r.h - 18, ACCENT, None, 1);
    }
}

impl App for Pg {
    fn event(&mut self, ev: Ev) {
        match ev {
            Ev::Key(k) => self.key(k),
            Ev::Char(c) => {
                if self.naming {
                    let c = c.to_ascii_uppercase();
                    let name = &mut self.lib[self.slot].name;
                    if (c.is_ascii_alphanumeric() || c == ' ' || c == '-') && name.chars().count() < 10 {
                        name.push(c);
                    }
                }
            }
            Ev::Move(x, y) => {
                self.mouse = (x, y);
                if self.screen == Screen::Yard {
                    self.drag_yard(x, y);
                }
                if self.map_drag {
                    self.cam_from_map(x.clamp(MINI.x + 1, MINI.x + MINI.w - 2), y.clamp(MINI.y + 1, MINI.y + MINI.h - 2));
                }
            }
            Ev::Press { x, y, right } => self.press(x, y, right),
            Ev::Release { x, y, .. } => {
                self.map_drag = false;
                if self.screen == Screen::Yard {
                    self.release_yard(x, y);
                }
            }
            Ev::Wheel(n) => {
                if self.screen == Screen::Yard && !self.help && !self.confirm_quit {
                    self.wheel_yard(n);
                }
            }
        }
    }
    fn hold(&mut self, dirs: [bool; 4]) {
        self.held = dirs;
        for k in 0..4 {
            self.latch[k] |= dirs[k];
        }
    }
    fn tick(&mut self) {
        self.step();
    }
    fn draw(&self, fb: &mut Framebuffer) {
        self.render(fb);
    }
    fn palette(&self) -> &Pal {
        &self.pal
    }
    fn crt(&self) -> bool {
        self.crt
    }
    fn toggle_crt(&mut self) {
        self.crt = !self.crt;
    }
    fn quit(&self) -> bool {
        self.quit
    }
}

#[cfg(test)]
mod tests {
    use super::yard::{yard_status, STATUS_RECT};
    use super::*;

    #[test]
    fn every_test_places_every_ship() {
        let lib = defaults();
        for t in tests() {
            let mut a = Arena::new();
            for &(team, slot, x, y) in &t.place {
                assert!(a.add(team, slot, &lib[slot], x, y), "{}", t.name);
                assert!((0..AW).contains(&x) && (0..AH).contains(&y), "{} places a ship off the arena at {x},{y}", t.name);
            }
        }
    }

    #[test]
    fn every_problem_message_fits_where_it_is_shown() {
        // Worst cases: every cell of the grid loose or buried, and the longest name a design can have.
        let n = (GRID * GRID) as usize;
        let checks = [
            Check { cores: 0, ..Check::default() },
            Check { cores: 1, loose: vec![(0, 0)], ..Check::default() },
            Check { cores: 1, loose: vec![(0, 0); n], ..Check::default() },
            Check { cores: 1, buried: vec![(0, 0)], ..Check::default() },
            Check { cores: 1, buried: vec![(0, 0); n], ..Check::default() },
        ];
        for c in checks.iter().chain([&Check { cores: 1, ..Check::default() }]) {
            for armed in [false, true] {
                for &order in &ORDERS {
                    // The yard's status line, one line inside its box.
                    let (status, _) = yard_status(c, armed, order);
                    assert!(Framebuffer::text_width(&status, 1) <= STATUS_RECT.w - 2, "the yard's status line overflows: {status}");
                }
            }
            // The arena's notice, wrapped inside the field.
            if let Some(why) = c.problem() {
                let notice = format!("WWWWWWWWWW CAN'T FLY: {why}");
                for l in notice_lines(&notice) {
                    assert!(Framebuffer::text_width(&l, 1) + 12 <= FIELD.w - 8, "the arena notice overflows: {l}");
                }
            }
        }
    }
}
