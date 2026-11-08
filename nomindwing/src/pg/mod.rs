//! The proving ground: draw ships in the shipyard, set them loose in the arena, and see whether
//! they fly and fight the way their drawings suggest.

pub mod design;
pub mod sim;

use self::design::*;
use self::sim::*;
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
    pub place: &'static [(usize, usize, i32, i32)],
}

pub const TESTS: [Test; 6] = [
    Test { name: "6 FIGHTERS VS 1 FRIGATE", place: &[(0, 0, 60, 110), (0, 0, 60, 126), (0, 0, 60, 142), (0, 0, 60, 158), (0, 0, 60, 174), (0, 0, 60, 190), (1, 2, 244, 150)] },
    Test { name: "3 FRIGATES VS 1 CAPITAL", place: &[(0, 2, 60, 100), (0, 2, 60, 150), (0, 2, 60, 200), (1, 3, 240, 150)] },
    Test { name: "1 CAPITAL VS 6 FIGHTERS", place: &[(0, 3, 64, 150), (1, 0, 244, 110), (1, 0, 244, 126), (1, 0, 244, 142), (1, 0, 244, 158), (1, 0, 244, 174), (1, 0, 244, 190)] },
    Test { name: "3 LANCERS VS 1 CAPITAL", place: &[(0, 1, 60, 120), (0, 1, 60, 150), (0, 1, 60, 180), (1, 3, 240, 150)] },
    Test { name: "4 FIGHTERS VS 4 MINERS", place: &[(0, 0, 40, 120), (0, 0, 40, 140), (0, 0, 40, 160), (0, 0, 40, 180), (1, 4, 236, 110), (1, 4, 236, 190), (1, 5, 256, 140), (1, 5, 256, 160)] },
    Test { name: "OPEN FIELD", place: &[] },
];

#[derive(Clone, Copy, Debug)]
pub struct Placement {
    pub team: usize,
    pub slot: usize,
    pub x: i32,
    pub y: i32,
}

// Shipyard layout.
const CELL: i32 = 9;
const GRID_RECT: Rect = Rect::new(6, 28, GRID * CELL + 1, GRID * CELL + 1);
const BTN_MIRROR: Rect = Rect::new(76, 221, 68, 15);
const BTN_CLEAR: Rect = Rect::new(6, 240, 68, 15);
const BTN_TO_ARENA: Rect = Rect::new(76, 240, 68, 15);
const NAME_RECT: Rect = Rect::new(150, 26, 184, 18);
const PREVIEW: Rect = Rect::new(338, 27, 58, 58);
const FLIGHT_RECT: Rect = Rect::new(150, 90, 186, 10);
const INFO_RECT: Rect = Rect::new(148, 216, 250, 82);

fn tab_rect(i: usize) -> Rect {
    Rect::new(4 + i as i32 * 66, 3, 63, 20)
}
fn part_rect(i: usize) -> Rect {
    Rect::new(6 + (i as i32 % 2) * 70, 170 + (i as i32 / 2) * 17, 68, 15)
}
fn order_rect(i: usize) -> Rect {
    Rect::new(150 + i as i32 * 62, 196, 60, 16)
}

// Arena layout.
const FIELD: Rect = Rect::new(0, 0, AW, AH);
const BTN_TEAM: [Rect; 2] = [Rect::new(306, 4, 44, 16), Rect::new(352, 4, 44, 16)];
const BTN_RUN: Rect = Rect::new(306, 172, 90, 16);
const BTN_RESET: Rect = Rect::new(306, 190, 44, 16);
const BTN_SPEED: Rect = Rect::new(352, 190, 44, 16);
const BTN_TEST: Rect = Rect::new(306, 208, 90, 16);
const BTN_EMPTY: Rect = Rect::new(306, 226, 44, 16);
const BTN_YARD: Rect = Rect::new(352, 226, 44, 16);
const HELP_RECT: Rect = Rect::new(10, 8, 380, 284);
const QUIT_RECT: Rect = Rect::new(112, 108, 176, 70);
const QUIT_BTNS: [Rect; 2] = [Rect::new(124, 150, 72, 18), Rect::new(204, 150, 72, 18)];

fn roster_rect(i: usize) -> Rect {
    Rect::new(306, 24 + i as i32 * 24, 90, 22)
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
        Part::Armor => STEEL,
        Part::Flak => GUN,
        Part::Lance => LANCE,
        Part::Engine => ENGINE,
        Part::Scoop => SCOOP,
        Part::Core => CORE,
    }
}

#[inline]
fn pc(fb: &mut Framebuffer, clip: Rect, x: i32, y: i32, c: u8) {
    if clip.contains(x, y) {
        fb.pset(x, y, c);
    }
}

/// Draws a shape centred on (x, y): a 1px dark hull around the solid cells, the hull lit along
/// its top edge, and checkerboard ghost dots where `ghost` says so.
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
        let col = if p == Part::Hull && open_above { team_col(team, 2) } else { part_col(p, team) };
        pc(fb, clip, px, py, col);
    }
}

pub struct Pg {
    pub lib: Vec<Design>,
    pub screen: Screen,
    pub slot: usize,
    pub part: Part,
    pub mirror: bool,
    painting: Option<bool>,
    last_cell: Option<(i32, i32)>,
    edited: bool,
    naming: bool,
    pub arena: Arena,
    pub setup: Vec<Placement>,
    pub test: Option<usize>,
    pub running: bool,
    pub speed: u32,
    pub team: usize,
    pub mouse: (i32, i32),
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
        if let Some(text) = save_path.as_ref().and_then(|p| std::fs::read_to_string(p).ok()) {
            for (i, d) in text.lines().filter_map(Design::from_line).take(lib.len()).enumerate() {
                lib[i] = d;
            }
        }
        let log = log_path.and_then(|p| OpenOptions::new().create(true).append(true).open(p).ok());
        let mut pg = Pg {
            lib,
            screen: Screen::Arena,
            slot: 0,
            part: Part::Hull,
            mirror: true,
            painting: None,
            last_cell: None,
            edited: false,
            naming: false,
            arena: Arena::new(),
            setup: Vec::new(),
            test: None,
            running: false,
            speed: 1,
            team: 0,
            mouse: (0, 0),
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
        pg.log(&format!("##### proving ground session {} #####", crate::game::utc_stamp()));
        pg.load_test(0);
        pg
    }

    fn log(&mut self, s: &str) {
        if let Some(f) = self.log.as_mut() {
            let _ = writeln!(f, "{s}");
        }
    }

    fn save(&mut self) {
        if let Some(p) = &self.save_path {
            let text: String = self.lib.iter().map(|d| d.to_line() + "\n").collect();
            let _ = std::fs::write(p, text);
        }
    }

    fn say(&mut self, s: &str) {
        self.notice = Some((s.to_string(), 30));
    }

    pub fn load_test(&mut self, t: usize) {
        self.test = Some(t);
        self.setup = TESTS[t].place.iter().map(|&(team, slot, x, y)| Placement { team, slot, x, y }).collect();
        self.reset();
    }

    /// Rebuilds the arena from the setup with the current designs. The same setup always replays
    /// the same fight.
    pub fn reset(&mut self) {
        self.arena = Arena::new();
        let mut missing = false;
        for p in &self.setup {
            if !self.arena.add(p.team, p.slot, &self.lib[p.slot], p.x, p.y) {
                missing = true;
            }
        }
        if missing {
            self.say("A DESIGN WITH NO CORE CAN'T BE PLACED");
        }
        self.running = false;
        self.logged_run = false;
        self.logged_over = false;
    }

    fn test_name(&self) -> &'static str {
        self.test.map_or("YOUR SETUP", |t| TESTS[t].name)
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
            (Screen::Yard, Key::Num(n)) if (1..=7).contains(&n) => self.part = PARTS[n as usize - 1],
            (Screen::Yard, Key::Letter('M')) => self.mirror = !self.mirror,
            (Screen::Arena, Key::Num(n)) if (1..=6).contains(&n) => self.slot = n as usize - 1,
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
        let t = self.test.map_or(0, |t| (t + 1) % TESTS.len());
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

    fn cell_under(x: i32, y: i32) -> Option<(i32, i32)> {
        if !GRID_RECT.contains(x, y) {
            return None;
        }
        let (cx, cy) = ((x - GRID_RECT.x) / CELL, (y - GRID_RECT.y) / CELL);
        if cx < GRID && cy < GRID {
            Some((cx, cy))
        } else {
            None
        }
    }

    fn paint(&mut self, (cx, cy): (i32, i32), erase: bool) {
        let p = if erase { None } else { Some(self.part) };
        let mirror = self.mirror;
        let d = &mut self.lib[self.slot];
        d.set(cx, cy, p);
        if mirror {
            d.set(GRID - 1 - cx, cy, p);
        }
        self.edited = true;
    }

    /// Paints every cell between the last sample and this one, so a fast stroke leaves no gaps.
    fn paint_to(&mut self, cell: (i32, i32), erase: bool) {
        let from = self.last_cell.unwrap_or(cell);
        let mut l = Line::new(from.0, from.1, cell.0, cell.1);
        loop {
            self.paint((l.x, l.y), erase);
            if (l.x, l.y) == cell {
                break;
            }
            l.step();
        }
        self.last_cell = Some(cell);
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

    fn press_yard(&mut self, x: i32, y: i32, right: bool) {
        if let Some(cell) = Self::cell_under(x, y) {
            self.painting = Some(right);
            self.last_cell = None;
            self.paint_to(cell, right);
            return;
        }
        if right {
            return;
        }
        if let Some(i) = (0..self.lib.len()).find(|&i| tab_rect(i).contains(x, y)) {
            self.slot = i;
        } else if let Some(i) = (0..PARTS.len()).find(|&i| part_rect(i).contains(x, y)) {
            self.part = PARTS[i];
        } else if let Some(i) = (0..ORDERS.len()).find(|&i| order_rect(i).contains(x, y)) {
            self.lib[self.slot].order = ORDERS[i];
            self.edited = true;
            self.save();
        } else if BTN_MIRROR.contains(x, y) {
            self.mirror = !self.mirror;
        } else if BTN_CLEAR.contains(x, y) {
            let d = &mut self.lib[self.slot];
            d.cells.iter_mut().for_each(|c| *c = None);
            self.edited = true;
            self.save();
        } else if BTN_TO_ARENA.contains(x, y) {
            self.switch_screen();
        } else if NAME_RECT.contains(x, y) {
            self.naming = true;
        }
    }

    fn press_arena(&mut self, x: i32, y: i32, right: bool) {
        if FIELD.contains(x, y) {
            if self.arena.tick > 0 {
                self.say("PRESS R TO RESET, THEN PLACE");
                return;
            }
            if right {
                if let Some(i) = self.arena.ship_at(x, y) {
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
            if self.lib[self.slot].stats().cores == 0 {
                self.say("THAT DESIGN HAS NO CORE");
                return;
            }
            self.setup.push(Placement { team: self.team, slot: self.slot, x, y });
            self.test = None;
            self.reset();
            return;
        }
        if right {
            return;
        }
        if let Some(t) = (0..2).find(|&t| BTN_TEAM[t].contains(x, y)) {
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
                let s = &self.arena.score;
                let line = format!(
                    "RESULT {who} at {}:{:02} | blue lost {}/{} ships, {} px, mined {} | red lost {}/{} ships, {} px, mined {}",
                    t / 60,
                    t % 60,
                    s[0].lost,
                    s[0].placed,
                    s[0].px_lost,
                    s[0].mined,
                    s[1].lost,
                    s[1].placed,
                    s[1].px_lost,
                    s[1].mined
                );
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
            let w = Framebuffer::text_width(s, 1) + 12;
            let r = Rect::new((AW - w) / 2, 22, w, 15);
            ui::panel(fb, r);
            fb.text(s, r.x + 6, r.y + 4, GOLD, None, 1);
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

    fn icon(&self, fb: &mut Framebuffer, d: &Design, r: Rect, team: usize) {
        fb.rect_fill(r.x, r.y, r.w, r.h, SPACE);
        let sh = d.shape();
        draw_shape(fb, r, &sh, r.x + r.w / 2, r.y + r.h / 2, 0, team, &|_| true, false);
    }

    fn draw_yard(&self, fb: &mut Framebuffer) {
        fb.rect_fill(0, 0, 400, 300, PANEL_LO);
        let m = self.mouse;
        for (i, d) in self.lib.iter().enumerate() {
            let r = tab_rect(i);
            let sel = i == self.slot;
            if sel {
                fb.rect_bevel(r.x, r.y, r.w, r.h, PANEL_DEEP, PANEL_LO, PANEL_HI);
                fb.rect_outline(r.x + 1, r.y + 1, r.w - 2, r.h - 2, ACCENT);
            } else {
                ui::panel(fb, r);
            }
            self.icon(fb, d, Rect::new(r.x + 3, r.y + 3, 14, 14), 0);
            let name: String = d.name.chars().take(7).collect();
            fb.text(&name, r.x + 20, r.y + 7, if sel { ACCENT } else { TEXT }, None, 1);
        }
        // The grid: every cell a part, up is the front.
        let d = &self.lib[self.slot];
        let g = GRID_RECT;
        fb.rect_fill(g.x - 1, g.y - 1, g.w + 2, g.h + 2, PANEL_HI);
        fb.rect_fill(g.x, g.y, g.w, g.h, OUTLINE);
        let hover = Self::cell_under(m.0, m.1);
        for cy in 0..GRID {
            for cx in 0..GRID {
                let (x, y) = (g.x + 1 + cx * CELL, g.y + 1 + cy * CELL);
                let col = match d.get(cx, cy) {
                    Some(p) => part_col(p, 0),
                    None if cx == GRID / 2 => PANEL,
                    None if (cx + cy) % 2 == 0 => PANEL_DEEP,
                    None => SPACE,
                };
                fb.rect_fill(x, y, CELL - 1, CELL - 1, col);
                if d.get(cx, cy) == Some(Part::Hull) {
                    fb.rect_fill(x, y, CELL - 1, 1, team_col(0, 2));
                }
            }
        }
        if let Some((cx, cy)) = hover {
            fb.rect_outline(g.x + cx * CELL, g.y + cy * CELL, CELL + 1, CELL + 1, WHITE);
            if self.mirror && cx != GRID - 1 - cx {
                fb.rect_outline(g.x + (GRID - 1 - cx) * CELL, g.y + cy * CELL, CELL + 1, CELL + 1, TEXT_FAINT);
            }
        }
        for (i, &p) in PARTS.iter().enumerate() {
            let r = part_rect(i);
            let sel = p == self.part;
            if sel {
                fb.rect_bevel(r.x, r.y, r.w, r.h, PANEL_DEEP, PANEL_LO, PANEL_HI);
                fb.rect_outline(r.x + 1, r.y + 1, r.w - 2, r.h - 2, ACCENT);
            } else if r.contains(m.0, m.1) {
                fb.rect_bevel(r.x, r.y, r.w, r.h, PANEL_HI, WHITE, PANEL);
            } else {
                ui::panel(fb, r);
            }
            fb.rect_fill(r.x + 4, r.y + 3, 9, 9, OUTLINE);
            fb.rect_fill(r.x + 5, r.y + 4, 7, 7, part_col(p, 0));
            let tc = if sel { ACCENT } else if r.contains(m.0, m.1) { PANEL_DEEP } else { TEXT };
            fb.text(p.name(), r.x + 17, r.y + 4, tc, None, 1);
            fb.text(&format!("{}", i + 1), r.x + r.w - 9, r.y + 4, TEXT_FAINT, None, 1);
        }
        ui::button(fb, BTN_MIRROR, if self.mirror { "MIRROR ON" } else { "MIRROR" }, BTN_MIRROR.contains(m.0, m.1), self.mirror);
        ui::button(fb, BTN_CLEAR, "CLEAR", BTN_CLEAR.contains(m.0, m.1), false);
        ui::button(fb, BTN_TO_ARENA, "ARENA", BTN_TO_ARENA.contains(m.0, m.1), false);
        fb.text("LEFT PAINTS", 6, 262, TEXT_FAINT, None, 1);
        fb.text("RIGHT ERASES", 6, 272, TEXT_FAINT, None, 1);
        fb.text("TAB: ARENA", 6, 282, TEXT_FAINT, None, 1);

        // Name, preview and the stats the drawing implies.
        if self.naming {
            fb.rect_fill(NAME_RECT.x, NAME_RECT.y, NAME_RECT.w, NAME_RECT.h, PANEL_DEEP);
            fb.text(&format!("{}_", d.name), NAME_RECT.x + 2, NAME_RECT.y + 2, ACCENT, None, 2);
        } else {
            fb.text(&d.name, NAME_RECT.x + 2, NAME_RECT.y + 2, if NAME_RECT.contains(m.0, m.1) { WHITE } else { TEXT }, Some(OUTLINE), 2);
        }
        ui::inset(fb, PREVIEW);
        fb.rect_fill(PREVIEW.x + 2, PREVIEW.y + 2, PREVIEW.w - 4, PREVIEW.h - 4, SPACE);
        let sh = d.shape();
        draw_shape(fb, PREVIEW, &sh, PREVIEW.x + PREVIEW.w / 2, PREVIEW.y + PREVIEW.h / 2, 0, 0, &|_| true, false);

        let st = d.stats();
        let x = 152;
        let line = |fb: &mut Framebuffer, y: i32, s: &str, c: u8| fb.text(s, x, y, c, None, 1);
        line(fb, 50, &format!("COST    {} PX", st.count), GOLD);
        line(fb, 60, &format!("HP      {}", st.hp), TEXT);
        line(fb, 70, &format!("WEIGHT  {}   ENGINES {}", st.mass, st.engines), TEXT);
        line(fb, 80, &format!("THRUST  {}", st.thrust), TEXT);
        line(fb, 92, &format!("FLIGHT  {}", st.flight.name()), ACCENT);
        let ds = |d: i32| format!("{}.{} S", d / 10, d % 10);
        if st.engines > 0 {
            line(fb, 104, &format!("TOP SPEED     {} PX/S", st.top_speed_px_s()), TEXT);
            line(fb, 114, &format!("0 TO TOP      {}", ds(st.zero_to_top_ds())), TEXT);
            line(fb, 124, &format!("QUARTER TURN  {}", ds(st.turn_ds())), TEXT);
        } else {
            line(fb, 104, "NO ENGINES: IT STAYS PUT", TEXT_DIM);
            line(fb, 114, "AND CAN'T TURN", TEXT_DIM);
        }
        line(fb, 136, &format!("FLAK   {}  TURRETS, 48 PX", st.flak), if st.flak > 0 { GUN } else { TEXT_FAINT });
        line(fb, 146, &format!("LANCE  {}  STRAIGHT AHEAD, 100 PX", st.lance), if st.lance > 0 { LANCE } else { TEXT_FAINT });
        line(fb, 156, &format!("SCOOPS {}  CARRIES {}", st.scoops, st.scoops * SCOOP_CARRY), if st.scoops > 0 { SCOOP } else { TEXT_FAINT });
        if st.cores == 0 {
            line(fb, 170, "NO CORE: IT CAN'T BE BUILT", BAD);
        } else if st.flak + st.lance == 0 {
            line(fb, 170, "NO GUNS: IT RUNS HOME WHEN HIT", TEXT_DIM);
        }
        fb.text("ORDER", 152, 186, TEXT_FAINT, None, 1);
        for (i, &o) in ORDERS.iter().enumerate() {
            let r = order_rect(i);
            ui::button(fb, r, o.name(), r.contains(m.0, m.1), o == d.order);
        }

        ui::inset(fb, INFO_RECT);
        let info: String = if let Some(i) = (0..PARTS.len()).find(|&i| part_rect(i).contains(m.0, m.1)) {
            format!("[{}] {}", i + 1, PARTS[i].blurb())
        } else if let Some(i) = (0..ORDERS.len()).find(|&i| order_rect(i).contains(m.0, m.1)) {
            ORDERS[i].blurb().into()
        } else if FLIGHT_RECT.contains(m.0, m.1) || (150..336).contains(&m.0) && (100..132).contains(&m.1) {
            "FLIGHT FOLLOWS FROM THE DRAWING. AT LEAST 1 THRUST PER WEIGHT (8 PER ENGINE): ATTACK RUNS, DIVING THROUGH THE TARGET AND LOOPING BACK. LESS: STAND-OFF, HOLDING AT WEAPON RANGE AND LINING UP SO LANCES BEAR. NO ENGINES: STAYS PUT.".into()
        } else if BTN_MIRROR.contains(m.0, m.1) {
            "[M] MIRROR PAINTS BOTH SIDES OF THE CENTRE COLUMN AT ONCE.".into()
        } else if NAME_RECT.contains(m.0, m.1) {
            "CLICK TO RENAME. ENTER WHEN DONE.".into()
        } else if (0..self.lib.len()).any(|i| tab_rect(i).contains(m.0, m.1)) {
            "YOUR SIX DESIGNS. THE TESTS IN THE ARENA USE THESE SLOTS, SO EDITING ONE CHANGES THE TESTS TOO.".into()
        } else {
            format!("PAINTING {}. LEFT-CLICK PAINTS, RIGHT-CLICK ERASES. UP IS THE FRONT: LANCES FIRE THAT WAY. EVERY PIXEL COSTS 1 AND WEIGHS 1 (ARMOR AND LANCE 2).", self.part.name())
        };
        for (i, l) in wrap(&info, 40).iter().enumerate().take(8) {
            fb.text(l, INFO_RECT.x + 4, INFO_RECT.y + 4 + i as i32 * 9, TEXT, None, 1);
        }
    }

    fn draw_arena(&self, fb: &mut Framebuffer) {
        let a = &self.arena;
        let m = self.mouse;
        fb.rect_fill(FIELD.x, FIELD.y, FIELD.w, FIELD.h, SPACE);
        for &(x, y, c) in &a.stars {
            fb.pset(x, y, STAR + c);
        }
        for y in 0..AH {
            for x in 0..AW {
                if a.rock[(y * AW + x) as usize] {
                    let lit = !a.rock_at(x, y - 1) || !a.rock_at(x - 1, y);
                    let dark = !a.rock_at(x, y + 1) || !a.rock_at(x + 1, y);
                    fb.pset(x, y, if lit && !dark { ROCK + 3 } else if dark { ROCK } else { ROCK + 1 });
                }
            }
        }
        for (t, &(sx, sy)) in STATIONS.iter().enumerate() {
            fb.rect_fill(sx - 4, sy - 13, 9, 27, OUTLINE);
            fb.rect_fill(sx - 3, sy - 12, 7, 25, team_col(t, 0));
            for k in 0..5 {
                fb.rect_fill(sx - 2, sy - 10 + k * 5, 5, 2, team_col(t, 2));
            }
        }
        for d in &a.debris {
            fb.pset(d.x, d.y, LOOSE);
        }
        for s in a.ships.iter().filter(|s| s.alive) {
            for &(x, y) in &s.trail {
                pc(fb, FIELD, x, y, team_col(s.team, 0));
            }
        }
        let hovered = if FIELD.contains(m.0, m.1) { a.ship_at(m.0, m.1) } else { None };
        if let Some(i) = hovered {
            let s = &a.ships[i];
            if let Some(t) = s.target.filter(|&t| a.ships[t].alive) {
                crate::render::dotted(fb, FIELD, s.x, s.y, a.ships[t].x, a.ships[t].y, 3, TEXT_FAINT);
            }
            if let Some((gx, gy)) = s.goal {
                for k in -2..=2 {
                    pc(fb, FIELD, gx + k, gy, TEXT_DIM);
                    pc(fb, FIELD, gx, gy + k, TEXT_DIM);
                }
            }
        }
        for s in a.ships.iter().filter(|s| s.alive) {
            draw_shape(fb, FIELD, &s.shape, s.x, s.y, s.facing, s.team, &|c| s.hp[c] > 0, false);
        }
        if let Some(i) = hovered {
            let (l, t, r, b) = a.ships[i].bbox();
            fb.corners(l - 2, t - 2, r - l + 5, b - t + 5, 2, WHITE);
        }
        for sh in &a.shots {
            if sh.lance {
                pc(fb, FIELD, sh.prev.0, sh.prev.1, LANCE);
                pc(fb, FIELD, sh.line.x, sh.line.y, LANCE_HI);
            } else {
                pc(fb, FIELD, sh.prev.0, sh.prev.1, team_col(sh.team, 1));
                pc(fb, FIELD, sh.line.x, sh.line.y, team_col(sh.team, 3));
            }
        }
        // Where a click would place the selected design.
        if a.tick == 0 && hovered.is_none() && FIELD.contains(m.0, m.1) && !self.help {
            let sh = self.lib[self.slot].shape();
            draw_shape(fb, FIELD, &sh, m.0, m.1, if self.team == 0 { 1 } else { 3 }, self.team, &|_| true, true);
        }
        fb.text(self.test_name(), 4, 3, TEXT_DIM, Some(OUTLINE), 1);

        // Inspector.
        let bar: Option<Vec<String>> = if let Some(i) = hovered {
            let s = &a.ships[i];
            let mut parts = vec![format!("{} PX/S", s.speed_px_s()), format!("{}/{} HP", s.hp_now(), s.hp_full())];
            if s.st.flak > 0 {
                parts.push(format!("FLAK {}", s.st.flak));
            }
            if s.st.lance > 0 {
                parts.push(format!("LANCE {}", s.st.lance));
            }
            if s.carry > 0 {
                parts.push(format!("CARRYING {}", s.carry));
            }
            parts.push(s.st.flight.name().into());
            Some(vec![format!("{} {} - {} - {}", team_name(s.team), s.name, s.order.name(), s.doing.name()), parts.join("  ")])
        } else if a.tick == 0 && FIELD.contains(m.0, m.1) {
            Some(vec![
                format!("CLICK PLACES A {} {}. RIGHT-CLICK REMOVES.", team_name(self.team), self.lib[self.slot].name),
                "SPACE RUNS THE FIGHT.".into(),
            ])
        } else {
            None
        };
        if let Some(lines) = bar {
            let h = lines.len() as i32 * 9 + 6;
            let r = Rect::new(0, AH - h, AW, h);
            ui::panel(fb, r);
            for (k, l) in lines.iter().enumerate() {
                fb.text(l, 4, r.y + 4 + k as i32 * 9, TEXT, None, 1);
            }
        }
        if let Some(result) = a.over {
            let r = Rect::new(62, 112, 180, 62);
            ui::panel(fb, r);
            let (head, col) = match result {
                Some(t) => (format!("{} WINS", team_name(t)), team_col(t, 2)),
                None => ("DRAW".to_string(), TEXT),
            };
            fb.text_centered(&head, r.x + r.w / 2, r.y + 8, col, Some(OUTLINE), 2);
            let t = a.tick / 15;
            fb.text_centered(&format!("IN {}:{:02}", t / 60, t % 60), r.x + r.w / 2, r.y + 28, TEXT, None, 1);
            fb.text_centered("SPACE OR R REPLAYS", r.x + r.w / 2, r.y + 44, TEXT_FAINT, None, 1);
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
            self.icon(fb, d, Rect::new(r.x + 3, r.y + 3, 16, 16), self.team);
            let tc = if hover && !sel { PANEL_DEEP } else { TEXT };
            let name: String = d.name.chars().take(10).collect();
            fb.text(&name, r.x + 22, r.y + 3, tc, None, 1);
            let st = d.stats();
            fb.text(&format!("{} PX {}", st.count, &d.order.name()[..4]), r.x + 22, r.y + 12, if hover && !sel { PANEL_LO } else { TEXT_DIM }, None, 1);
            fb.text(&format!("{}", i + 1), r.x + r.w - 8, r.y + 3, TEXT_FAINT, None, 1);
        }
        let run_label = if self.running { "PAUSE" } else if a.over.is_some() { "REPLAY" } else { "RUN" };
        ui::button(fb, BTN_RUN, run_label, BTN_RUN.contains(m.0, m.1), self.running);
        ui::button(fb, BTN_RESET, "RESET", BTN_RESET.contains(m.0, m.1), false);
        ui::button(fb, BTN_SPEED, &format!("{}X", self.speed), BTN_SPEED.contains(m.0, m.1), self.speed > 1);
        let tl = match self.test {
            Some(t) => format!("TEST {}/{}", t + 1, TESTS.len()),
            None => "NEXT TEST".into(),
        };
        ui::button(fb, BTN_TEST, &tl, BTN_TEST.contains(m.0, m.1), false);
        ui::button(fb, BTN_EMPTY, "EMPTY", BTN_EMPTY.contains(m.0, m.1), false);
        ui::button(fb, BTN_YARD, "YARD", BTN_YARD.contains(m.0, m.1), false);
        let sc = &a.score;
        for t in 0..2 {
            let y = 248 + t as i32 * 10;
            fb.text(team_name(t), 308, y, team_col(t, 2), None, 1);
            fb.text(&format!("{}/{}", a.alive(t), sc[t].placed), 340, y, TEXT, None, 1);
            fb.text(&format!("-{}", sc[t].px_lost), 370, y, TEXT_DIM, None, 1);
        }
        let t = a.tick / 15;
        fb.text(&format!("TIME {}:{:02}", t / 60, t % 60), 308, 272, TEXT_DIM, None, 1);
        fb.text("SHIPS / PX LOST", 308, 286, TEXT_FAINT, None, 1);

        // Button tips go in the inspector's place when nothing in the field is hovered.
        if !FIELD.contains(m.0, m.1) {
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
            } else if (0..2).any(|t| BTN_TEAM[t].contains(m.0, m.1)) {
                Some("[T] WHICH TEAM YOUR CLICKS PLACE.")
            } else {
                (0..self.lib.len()).find(|&i| roster_rect(i).contains(m.0, m.1)).map(|_| "[1-6] THE DESIGN YOUR CLICKS PLACE. EDIT IT IN THE YARD.")
            };
            if let Some(tip) = tip {
                let lines = wrap(tip, 49);
                let h = lines.len() as i32 * 9 + 6;
                let r = Rect::new(0, AH - h, AW, h);
                ui::panel(fb, r);
                for (k, l) in lines.iter().enumerate() {
                    fb.text(l, 4, r.y + 4 + k as i32 * 9, TEXT, None, 1);
                }
            }
        }
    }

    fn draw_help(&self, fb: &mut Framebuffer) {
        let r = HELP_RECT;
        ui::panel(fb, r);
        fb.text("PROVING GROUND", r.x + 10, r.y + 8, ACCENT, Some(OUTLINE), 2);
        fb.text("NO MIND WING", r.x + r.w - 82, r.y + 12, TEXT_DIM, None, 1);
        let body = [
            ("SHIPS", "DRAW A SHIP IN THE YARD. EVERY COLOUR IS A PART AND UP IS THE FRONT. NOTHING ELSE DECIDES WHAT IT IS."),
            ("FLYING", "AT LEAST 1 THRUST PER WEIGHT: ATTACK RUNS. LESS: IT HOLDS AT WEAPON RANGE, LINED UP SO ITS LANCES BEAR. HEAVIER SHIPS TURN AND CHANGE COURSE SLOWER."),
            ("GUNS", "FLAK TURRETS FIRE AT THE NEAREST ENEMY IN RANGE WHATEVER THE ORDER. LANCES FIRE ONLY STRAIGHT AHEAD AND PUNCH THROUGH."),
            ("ORDERS", "HUNT, GUARD, HARVEST OR HOLD, ONE PER DESIGN. A SHIP WITH NO GUNS LEFT RUNS HOME WHEN HIT."),
            ("ARENA", "PICK A TEAM AND A DESIGN, CLICK TO PLACE, RIGHT-CLICK TO REMOVE. SPACE RUNS. R REPLAYS THE SAME SETUP. HOVER A SHIP TO SEE WHAT IT'S DOING."),
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
        for (k, (p, name)) in [(Part::Flak, "FLAK"), (Part::Lance, "LANCE"), (Part::Engine, "ENGINE"), (Part::Scoop, "SCOOP"), (Part::Armor, "ARMOR"), (Part::Core, "CORE")].iter().enumerate() {
            let x = r.x + 14 + k as i32 * 60;
            fb.rect_fill(x - 1, ly - 1, 6, 6, OUTLINE);
            fb.rect_fill(x, ly, 4, 4, part_col(*p, 0));
            fb.text(name, x + 8, ly - 1, TEXT_DIM, None, 1);
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
                if let Some(erase) = self.painting {
                    if let Some(cell) = Self::cell_under(x, y) {
                        self.paint_to(cell, erase);
                    }
                }
            }
            Ev::Press { x, y, right } => self.press(x, y, right),
            Ev::Release { .. } => {
                if self.painting.take().is_some() {
                    self.last_cell = None;
                    self.save();
                }
            }
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
