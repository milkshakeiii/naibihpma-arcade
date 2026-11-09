//! The playable toy: input, camera, the card sidebar, overlays and the playtest log.

use crate::ai::Ai;
use crate::app::{App, Ev, Key};
use crate::design::{cost, info, Kind, CARDS};
use crate::framebuffer::{wrap, Framebuffer};
use crate::geom::dist2;
use crate::palette::{self, *};
use crate::render::{self, Aim, TAC_RECT, VIEW};
use crate::ui::{self, Rect};
use crate::world::*;
use std::fs::{File, OpenOptions};
use std::io::Write;
use std::path::PathBuf;

pub const SIDEBAR: Rect = Rect::new(304, 0, 96, 300);
pub const MINI_RECT: Rect = Rect::new(310, 6, MINI_W, MINI_H);
pub const BTN_TAC: Rect = Rect::new(306, 262, 44, 16);
pub const BTN_PAUSE: Rect = Rect::new(352, 262, 44, 16);
pub const BTN_HELP: Rect = Rect::new(306, 280, 44, 16);
pub const BTN_UNDO: Rect = Rect::new(352, 280, 44, 16);
const HELP_RECT: Rect = Rect::new(14, 12, 372, 276);
const PAUSE_RECT: Rect = Rect::new(112, 92, 176, 112);
const PAUSE_BTNS: [Rect; 3] = [Rect::new(132, 120, 136, 18), Rect::new(132, 144, 136, 18), Rect::new(132, 168, 136, 18)];
const END_RECT: Rect = Rect::new(40, 60, 320, 180);
const END_BTNS: [Rect; 2] = [Rect::new(78, 210, 116, 18), Rect::new(206, 210, 116, 18)];

pub fn card_rect(i: usize) -> Rect {
    Rect::new(306, 98 + i as i32 * 26, 90, 24)
}

/// The print queue's icons: the current job first, then the queue.
pub fn queue_rect(n: usize) -> Rect {
    Rect::new(306 + n as i32 * 15, 240, 14, 16)
}

const PAUSE: Key = Key::Letter('P');
const RESTART: Key = Key::Letter('R');

pub struct Game {
    pub w: World,
    pub ai: Ai,
    /// Drives the player's side too (headless sims and screenshots).
    pub autopilot: Option<Ai>,
    pub seed: u64,
    pub match_no: u32,
    pub cam: (i32, i32),
    pub follow: bool,
    pub mouse: (i32, i32),
    held: [bool; 4],
    latch: [bool; 4],
    pub targeting: Option<Kind>,
    pub tactical: bool,
    pub paused: bool,
    pub help: bool,
    pub crt: bool,
    pub quit: bool,
    pal: Pal,
    log: Option<File>,
    notice: Option<(String, u32)>,
    ended_logged: bool,
    cmds: Vec<Cmd>,
}

impl Game {
    pub fn new(seed: u64, log_path: Option<PathBuf>) -> Self {
        let w = World::new(seed);
        let ai = Ai::new(1, &w);
        let log = log_path.and_then(|p| OpenOptions::new().create(true).append(true).open(p).ok());
        let mut g = Game {
            w,
            ai,
            autopilot: None,
            seed,
            match_no: 1,
            cam: (0, 0),
            follow: true,
            mouse: (0, 0),
            held: [false; 4],
            latch: [false; 4],
            targeting: None,
            tactical: false,
            paused: false,
            help: true,
            crt: true,
            quit: false,
            pal: palette::game(),
            log,
            notice: None,
            ended_logged: false,
            cmds: Vec::new(),
        };
        g.center_camera();
        g.log_raw(&format!("##### match {} seed {} start {} #####", g.match_no, seed, utc_stamp()));
        g
    }

    fn new_match(&mut self) {
        let seed = self.seed.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407) >> 16;
        self.seed = seed;
        self.match_no += 1;
        self.w = World::new(seed);
        self.ai = Ai::new(1, &self.w);
        if self.autopilot.is_some() {
            self.autopilot = Some(Ai::new(0, &self.w));
        }
        self.targeting = None;
        self.tactical = false;
        self.paused = false;
        self.ended_logged = false;
        self.notice = None;
        self.center_camera();
        self.log_raw(&format!("##### match {} seed {} start {} #####", self.match_no, seed, utc_stamp()));
    }

    pub fn palette(&self) -> &Pal {
        &self.pal
    }

    /// 0 = arrow, 1 = planting crosshair.
    pub fn cursor_kind(&self) -> u8 {
        if self.targeting.is_some() && !self.help && !self.paused && self.w.winner.is_none() && (self.mouse.0 < VIEW.w || MINI_RECT.contains(self.mouse.0, self.mouse.1)) {
            1
        } else {
            0
        }
    }

    pub fn toggle_crt(&mut self) {
        self.crt = !self.crt;
    }

    /// Movement keys held right now: up, down, left, right. Presses are latched until the tick.
    pub fn hold(&mut self, dirs: [bool; 4]) {
        self.held = dirs;
        for k in 0..4 {
            self.latch[k] |= dirs[k];
        }
    }

    // ----- Input -----

    pub fn event(&mut self, ev: Ev) {
        match ev {
            Ev::Move(x, y) => self.mouse = (x, y),
            Ev::Key(k) => self.key(k),
            Ev::Press { x, y, right } => self.press(x, y, right),
            Ev::Release { .. } | Ev::Char(_) | Ev::Wheel(_) => {}
        }
    }

    fn key(&mut self, k: Key) {
        if self.help {
            // Movement keys don't dismiss the rules, so you can't fly off by accident.
            if !matches!(k, Key::Letter('W' | 'A' | 'S' | 'D')) {
                self.help = false;
            }
            return;
        }
        if self.w.winner.is_some() {
            match k {
                Key::Enter | RESTART => self.new_match(),
                Key::Esc => self.quit = true,
                _ => {}
            }
            return;
        }
        if self.paused {
            match k {
                PAUSE | Key::Esc | Key::Enter => self.set_paused(false),
                RESTART => self.new_match(),
                _ => {}
            }
            return;
        }
        match k {
            Key::Num(n) if (1..=5).contains(&n) => self.card(n as usize - 1),
            Key::Tab => self.toggle_tactical(),
            PAUSE => self.set_paused(true),
            Key::Esc => {
                if self.targeting.is_some() {
                    self.targeting = None;
                } else {
                    self.set_paused(true);
                }
            }
            Key::Help => self.help = true,
            Key::Back => self.unqueue(),
            _ => {}
        }
    }

    fn press(&mut self, x: i32, y: i32, right: bool) {
        if self.help {
            self.help = false;
            return;
        }
        if self.w.winner.is_some() {
            if END_BTNS[0].contains(x, y) {
                self.new_match();
            } else if END_BTNS[1].contains(x, y) {
                self.quit = true;
            }
            return;
        }
        if self.paused {
            if PAUSE_BTNS[0].contains(x, y) {
                self.set_paused(false);
            } else if PAUSE_BTNS[1].contains(x, y) {
                self.new_match();
            } else if PAUSE_BTNS[2].contains(x, y) {
                self.quit = true;
            }
            return;
        }
        if right {
            if let Some(n) = (0..QUEUE_MAX).find(|&n| queue_rect(n).contains(x, y)) {
                self.toggle_repeat(n);
            } else {
                self.targeting = None;
            }
            return;
        }
        if let Some(i) = (0..CARDS.len()).find(|&i| card_rect(i).contains(x, y)) {
            self.card(i);
        } else if BTN_TAC.contains(x, y) {
            self.toggle_tactical();
        } else if BTN_PAUSE.contains(x, y) {
            self.set_paused(true);
        } else if BTN_HELP.contains(x, y) {
            self.help = true;
        } else if BTN_UNDO.contains(x, y) {
            self.unqueue();
        } else if let Some(kind) = self.targeting {
            if let Some((wx, wy)) = self.world_point(x, y) {
                self.plant_at(kind, wx, wy);
            }
        }
    }

    /// The world point under a screen point (playfield, tactical view or minimap).
    fn world_point(&self, x: i32, y: i32) -> Option<(i32, i32)> {
        if MINI_RECT.contains(x, y) {
            return Some(((x - MINI_RECT.x) * MINI + MINI / 2, (y - MINI_RECT.y) * MINI + MINI / 2));
        }
        if !VIEW.contains(x, y) {
            return None;
        }
        if self.tactical {
            if TAC_RECT.contains(x, y) {
                Some(((x - TAC_RECT.x) * TAC + TAC / 2, (y - TAC_RECT.y) * TAC + TAC / 2))
            } else {
                None
            }
        } else {
            Some((self.cam.0 + x, self.cam.1 + y))
        }
    }

    fn card(&mut self, i: usize) {
        let kind = CARDS[i];
        if kind.is_seed() {
            self.targeting = if self.targeting == Some(kind) { None } else { Some(kind) };
            return;
        }
        self.targeting = None;
        if self.w.king(0).queue.len() >= QUEUE_MAX {
            self.say("THE PRINT QUEUE IS FULL");
            return;
        }
        self.cmds.push(Cmd::Queue { team: 0, kind, dest: None });
        self.log(&format!("CARD {}", info(kind).name));
    }

    fn slack(&self) -> i32 {
        if self.tactical {
            12
        } else {
            8
        }
    }

    fn aim(&self) -> Option<Aim> {
        let _ = self.targeting?;
        let (wx, wy) = self.world_point(self.mouse.0, self.mouse.1)?;
        let slack = if MINI_RECT.contains(self.mouse.0, self.mouse.1) { 16 } else { self.slack() };
        let rock = self.w.rock_near_point(wx, wy, slack);
        let ok = rock.map_or(false, |a| self.w.rock_free(a, 0, None) && !self.queued_for(a));
        Some(Aim { point: (wx, wy), rock, ok })
    }

    fn queued_for(&self, a: usize) -> bool {
        let k = self.w.king(0);
        k.queue.iter().chain(k.job.iter()).any(|j| j.dest == Some(a))
    }

    fn plant_at(&mut self, kind: Kind, wx: i32, wy: i32) {
        let slack = if MINI_RECT.contains(self.mouse.0, self.mouse.1) { 16 } else { self.slack() };
        match self.w.rock_near_point(wx, wy, slack) {
            Some(a) if self.w.rock_free(a, 0, None) && !self.queued_for(a) => {
                if self.w.king(0).queue.len() >= QUEUE_MAX {
                    self.say("THE PRINT QUEUE IS FULL");
                    return;
                }
                self.cmds.push(Cmd::Queue { team: 0, kind, dest: Some(a) });
                let r = &self.w.rocks[a];
                let msg = format!("CARD {} -> ROCK {} AT ({},{}) {} PX", info(kind).name, a, r.cx, r.cy, r.count);
                self.log(&msg);
                self.targeting = None;
            }
            Some(a) if self.w.rocks[a].count < PLANT_MIN => self.say("THAT ROCK IS TOO SMALL TO GROW ON"),
            Some(_) => self.say("THAT ROCK IS ALREADY TAKEN"),
            None => self.say("SEEDS GROW ON ROCKS"),
        }
    }

    /// Right-clicking a print-queue icon makes that card print again each time it finishes.
    fn toggle_repeat(&mut self, n: usize) {
        let k = self.w.king(0);
        let Some(j) = k.job.iter().chain(k.queue.iter()).nth(n).copied() else { return };
        if j.kind.is_seed() {
            self.say("SEEDS NEED A NEW ROCK EACH TIME");
            return;
        }
        self.cmds.push(Cmd::ToggleRepeat { team: 0, n });
        self.log(&format!("REPEAT {} {}", info(j.kind).name, if j.repeat { "OFF" } else { "ON" }));
    }

    fn unqueue(&mut self) {
        let k = self.w.king(0);
        if k.queue.is_empty() && k.job.is_none() {
            return;
        }
        self.cmds.push(Cmd::Unqueue { team: 0 });
        self.log("UNQUEUE");
    }

    fn toggle_tactical(&mut self) {
        self.tactical = !self.tactical;
        self.log(if self.tactical { "TACTICAL ON" } else { "TACTICAL OFF" });
    }

    fn set_paused(&mut self, p: bool) {
        self.paused = p;
        self.log(if p { "PAUSE" } else { "RESUME" });
    }

    fn say(&mut self, s: &str) {
        self.notice = Some((s.to_string(), 30));
    }

    // ----- Tick -----

    pub fn tick(&mut self) {
        if let Some((_, t)) = self.notice.as_mut() {
            *t = t.saturating_sub(1);
            if *t == 0 {
                self.notice = None;
            }
        }
        if self.help || self.paused || self.w.winner.is_some() {
            self.latch = self.held;
            self.log_end();
            return;
        }
        let d = [self.held[0] || self.latch[0], self.held[1] || self.latch[1], self.held[2] || self.latch[2], self.held[3] || self.latch[3]];
        self.latch = [false; 4];
        self.w.steer[0] = (d[3] as i32 - d[2] as i32, d[1] as i32 - d[0] as i32);
        let mut cmds = std::mem::take(&mut self.cmds);
        if let Some(ap) = self.autopilot.as_mut() {
            self.w.steer[0] = ap.step(&self.w, &mut cmds);
        }
        self.w.steer[1] = self.ai.step(&self.w, &mut cmds);
        self.w.tick(&cmds);
        let events: Vec<Event> = self.w.events.drain(..).collect();
        for e in events {
            match e {
                Event::Planted { team, kind, x, y, by_king } => {
                    let how = if by_king { "by king" } else { "spread" };
                    self.log(&format!("PLANTED {} {} at ({x},{y}) {how}", side(team), info(kind).name));
                }
                Event::Lost { team, kind, x, y } if kind != Kind::Collector && kind != Kind::Escort && kind != Kind::Raider => {
                    self.log(&format!("LOST {} {} at ({x},{y})", side(team), info(kind).name));
                }
                _ => {}
            }
        }
        if self.w.tick % (15 * 30) == 0 {
            let line = format!("SNAP {} | {}", self.team_line(0), self.team_line(1));
            self.log(&line);
        }
        if self.follow {
            self.center_camera();
        }
        self.log_end();
    }

    fn team_line(&self, t: usize) -> String {
        let n = |k: Kind| self.w.ships.iter().filter(|s| s.alive && s.team == t && s.kind == k).count();
        let tm = &self.w.teams[t];
        format!(
            "{} px={} mined={} king={}% col={} esc={} raid={} outposts={} fwd={} kills={}",
            side(t),
            tm.bank,
            tm.mined,
            self.w.king(t).hp_pct(),
            n(Kind::Collector),
            n(Kind::Escort),
            n(Kind::Raider),
            n(Kind::Outpost),
            n(Kind::Forward),
            tm.kills
        )
    }

    fn log_end(&mut self) {
        if self.ended_logged {
            return;
        }
        if let Some(win) = self.w.winner {
            self.ended_logged = true;
            let line = format!("END winner={} | {} | {}", side(win), self.team_line(0), self.team_line(1));
            self.log(&line);
        }
    }

    pub fn center_camera(&mut self) {
        let k = self.w.king(0);
        self.cam = ((k.x - VIEW.w / 2).clamp(0, WORLD_W - VIEW.w), (k.y - VIEW.h / 2).clamp(0, WORLD_H - VIEW.h));
    }

    fn log(&mut self, s: &str) {
        let t = self.w.tick / 15;
        let line = format!("t={:02}:{:02} {s}", t / 60, t % 60);
        self.log_raw(&line);
    }

    fn log_raw(&mut self, s: &str) {
        if let Some(f) = self.log.as_mut() {
            let _ = writeln!(f, "{s}");
        }
    }

    // ----- Drawing -----

    pub fn draw(&self, fb: &mut Framebuffer) {
        let aim = self.aim();
        if self.tactical {
            render::tactical(fb, &self.w, self.cam, 0, aim.as_ref());
        } else {
            render::world(fb, &self.w, self.cam, 0, aim.as_ref());
        }
        self.sidebar(fb);
        if let Some((s, _)) = &self.notice {
            let w = Framebuffer::text_width(s, 1) + 12;
            let r = Rect::new((VIEW.w - w) / 2, 6, w, 15);
            ui::panel(fb, r);
            fb.text(s, r.x + 6, r.y + 4, GOLD, None, 1);
        }
        if !self.help && !self.paused && self.w.winner.is_none() {
            self.hover_bar(fb);
        }
        if self.help {
            self.help_overlay(fb);
        } else if self.w.winner.is_some() {
            self.end_overlay(fb);
        } else if self.paused {
            self.pause_overlay(fb);
        }
    }

    fn sidebar(&self, fb: &mut Framebuffer) {
        ui::panel(fb, SIDEBAR);
        ui::inset(fb, Rect::new(MINI_RECT.x - 2, MINI_RECT.y - 2, MINI_RECT.w + 4, MINI_RECT.h + 4));
        render::minimap(fb, &self.w, MINI_RECT, self.cam, VIEW);
        let king = self.w.king(0);
        fb.text("PX", 308, 72, TEXT_DIM, Some(OUTLINE), 1);
        fb.text(&format!("{}", self.w.teams[0].bank), 324, 72, GOLD, Some(OUTLINE), 1);
        fb.pset(388, 74, LOOSE);
        fb.pset(390, 76, LOOSE);
        fb.pset(387, 77, LOOSE);
        let hp = king.hp_pct() as i32;
        fb.text("KING", 308, 84, TEXT_DIM, Some(OUTLINE), 1);
        fb.rect_fill(334, 85, 58, 6, OUTLINE);
        fb.rect_fill(335, 86, 56, 4, PANEL_DEEP);
        fb.rect_fill(335, 86, 56 * hp.clamp(0, 100) / 100, 4, if hp > 40 { GOOD } else { BAD });
        for (i, &k) in CARDS.iter().enumerate() {
            let r = card_rect(i);
            let hover = r.contains(self.mouse.0, self.mouse.1);
            let sel = self.targeting == Some(k);
            if sel {
                fb.rect_bevel(r.x, r.y, r.w, r.h, PANEL_DEEP, PANEL_LO, PANEL_HI);
                fb.rect_outline(r.x + 1, r.y + 1, r.w - 2, r.h - 2, ACCENT);
            } else if hover {
                fb.rect_bevel(r.x, r.y, r.w, r.h, PANEL_HI, WHITE, PANEL);
            } else {
                fb.rect_bevel(r.x, r.y, r.w, r.h, PANEL, PANEL_HI, PANEL_LO);
            }
            fb.rect_fill(r.x + 3, r.y + 3, 18, 18, SPACE);
            let shown = k.grows_into().unwrap_or(k);
            let d = &info(shown).design;
            let clip = Rect::new(r.x + 3, r.y + 3, 18, 18);
            render::draw_design(fb, clip, d, r.x + 12, r.y + 12, 0, 0, &|_| true, &|_| false);
            let tc = if hover && !sel { PANEL_DEEP } else { TEXT };
            fb.text(info(k).label, r.x + 24, r.y + 4, tc, None, 1);
            let dim = if hover && !sel { PANEL_LO } else { TEXT_DIM };
            fb.text(&format!("{} PX", cost(k)), r.x + 24, r.y + 14, dim, None, 1);
            fb.text(&format!("{}", i + 1), r.x + r.w - 9, r.y + 14, if hover && !sel { PANEL_LO } else { TEXT_FAINT }, None, 1);
        }
        fb.text("PRINTING", 308, 230, TEXT_FAINT, None, 1);
        let jobs: Vec<Job> = king.job.iter().chain(king.queue.iter()).copied().collect();
        for (n, j) in jobs.iter().enumerate().take(QUEUE_MAX) {
            let r = queue_rect(n);
            let x = r.x;
            fb.rect_fill(r.x, r.y, r.w, r.h, PANEL_DEEP);
            if j.repeat {
                fb.rect_outline(r.x, r.y, r.w, r.h, ACCENT);
            }
            let d = &info(j.kind).design;
            let printed = j.printed;
            render::draw_design(fb, r, d, x + 7, 247, 0, 0, &|c| n > 0 || d.rank[c] < printed, &|c| n == 0 && d.rank[c] >= printed);
            if n == 0 {
                let need = d.count.max(1);
                fb.rect_fill(r.x, r.y + r.h - 2, r.w * printed as i32 / need as i32, 2, GOOD);
            }
        }
        let m = self.mouse;
        ui::button(fb, BTN_TAC, "TAC", BTN_TAC.contains(m.0, m.1), self.tactical);
        ui::button(fb, BTN_PAUSE, "PAUSE", BTN_PAUSE.contains(m.0, m.1), false);
        ui::button(fb, BTN_HELP, "HELP", BTN_HELP.contains(m.0, m.1), false);
        ui::button(fb, BTN_UNDO, "UNDO", BTN_UNDO.contains(m.0, m.1), false);
    }

    fn hover_text(&self) -> Option<String> {
        let (x, y) = self.mouse;
        if let Some(i) = (0..CARDS.len()).find(|&i| card_rect(i).contains(x, y)) {
            let k = CARDS[i];
            let how = if k.is_seed() { "CLICK, THEN CLICK A ROCK." } else { "CLICK TO PRINT." };
            return Some(format!("[{}] {} - {} PX. {} {}", i + 1, info(k).name, cost(k), info(k).blurb, how));
        }
        if BTN_TAC.contains(x, y) {
            return Some("[TAB] TACTICAL VIEW: THE WHOLE MAP AT A QUARTER SCALE. YOUR KING KEEPS FLYING.".into());
        }
        if BTN_PAUSE.contains(x, y) {
            return Some("[P] PAUSE".into());
        }
        if BTN_HELP.contains(x, y) {
            return Some("[F1] THE RULES AND KEYS".into());
        }
        if BTN_UNDO.contains(x, y) {
            return Some("[BACKSPACE] TAKE THE LAST CARD OFF THE PRINT QUEUE. PRINTED PIXELS COME BACK.".into());
        }
        if (306..396).contains(&x) && (230..258).contains(&y) {
            return Some("YOUR KING PRINTS THESE IN ORDER, A PIXEL A FRAME WHILE PX LAST. RIGHT-CLICK ONE TO REPEAT IT (BLUE FRAME).".into());
        }
        if let Some(k) = self.targeting {
            if let Some(aim) = self.aim() {
                return Some(match aim.rock {
                    Some(a) if aim.ok => format!("PLANT {} ON THIS ROCK ({} PX). RIGHT-CLICK CANCELS.", info(k).name, self.w.rocks[a].count),
                    Some(a) if self.w.rocks[a].count < PLANT_MIN => "TOO SMALL TO GROW ON.".into(),
                    Some(_) => "THIS ROCK IS TAKEN.".into(),
                    None => format!("CLICK A ROCK FOR YOUR {}. RIGHT-CLICK CANCELS.", info(k).name),
                });
            }
        }
        let (wx, wy) = self.world_point(x, y)?;
        let slack = if self.tactical || MINI_RECT.contains(x, y) { 6 } else { 1 };
        let ship = self
            .w
            .ships
            .iter()
            .filter(|s| {
                let (l, t, r, b) = s.bbox();
                s.alive && wx >= l - slack && wy >= t - slack && wx <= r + slack && wy <= b + slack
            })
            .min_by_key(|s| dist2(s.x, s.y, wx, wy));
        if let Some(s) = ship {
            let who = if s.team == 0 { "YOUR" } else { "ENEMY" };
            let doing = match s.kind {
                Kind::King | Kind::Outpost | Kind::Forward if !s.complete() => format!("GROWING {}/{}", s.grown, s.design().count),
                Kind::King | Kind::Outpost | Kind::Forward => match s.job {
                    Some(j) => format!("PRINTING {} {}/{}", info(j.kind).name, j.printed, cost(j.kind)),
                    None => "IDLE".into(),
                },
                Kind::Collector if s.returning => format!("HAULING {} PX", s.carry),
                Kind::Collector => "MINING".into(),
                Kind::Escort => "GUARDING".into(),
                Kind::Raider => "HUNTING".into(),
                _ => "FLYING TO A ROCK".into(),
            };
            return Some(format!("{who} {} - {doing}. {}/{} HP.", info(s.kind).name, s.hp_now(), s.design().full_hp));
        }
        let a = self.w.rock_near_point(wx, wy, 0)?;
        let r = &self.w.rocks[a];
        let owner = match self.w.base_on(a) {
            Some(b) => format!(" {} {} STANDS ON IT.", if b.team == 0 { "YOUR" } else { "AN ENEMY" }, info(b.kind).name),
            None if r.count < PLANT_MIN => " TOO SMALL TO GROW ON.".into(),
            None => String::new(),
        };
        Some(format!("ROCK - {} PX.{owner}", r.count))
    }

    fn hover_bar(&self, fb: &mut Framebuffer) {
        let Some(s) = self.hover_text() else { return };
        let lines = wrap(&s, 49);
        let h = lines.len() as i32 * 9 + 6;
        let r = Rect::new(0, VIEW.h - h, VIEW.w, h);
        ui::panel(fb, r);
        for (i, l) in lines.iter().enumerate() {
            fb.text(l, 4, r.y + 4 + i as i32 * 9, TEXT, None, 1);
        }
    }

    fn help_overlay(&self, fb: &mut Framebuffer) {
        let r = HELP_RECT;
        ui::panel(fb, r);
        fb.text("NO MIND WING", r.x + 10, r.y + 8, ACCENT, Some(OUTLINE), 2);
        fb.text("TOY 1", r.x + r.w - 42, r.y + 12, TEXT_DIM, None, 1);
        let body = [
            ("FLY", "WASD OR ARROWS FLY YOUR KING. IF IT DIES, YOU LOSE. SO DOES THE ENEMY."),
            ("PRINT", "CLICK A CARD OR PRESS 1-5. YOUR KING PRINTS IT ONE PIXEL AT A TIME, PAID FROM PX."),
            ("PX", "COLLECTORS CHEW PIXELS OFF ROCKS AND WRECKS AND BRING THEM HOME."),
            ("SEEDS", "PICK A SEED CARD, THEN CLICK A ROCK (MAP, MINIMAP OR TAC VIEW). THE SEED FLIES THERE AND EATS THE ROCK INTO A BASE."),
            ("BASES", "A PLANTED BASE PRINTS ITS OWN PINNED LIST AND REBUILDS WHATEVER DIES."),
            ("PARTS", "YELLOW SHOOTS, BLUE-WHITE PUSHES, ORANGE SCOOPS, GREY TAKES TWO HITS. A SHIP DIES WHEN ITS WHITE CORE IS GONE."),
            ("KEYS", "TAB TACTICAL VIEW  P PAUSE  BACKSPACE UNQUEUE  RIGHT-CLICK A QUEUED CARD TO REPEAT IT  F2 SCANLINES  F1 HELP"),
        ];
        let mut y = r.y + 34;
        for (head, text) in body {
            fb.text(head, r.x + 10, y, GOLD, None, 1);
            let lines = wrap(text, 50);
            for (i, l) in lines.iter().enumerate() {
                fb.text(l, r.x + 52, y + i as i32 * 9, TEXT, None, 1);
            }
            y += lines.len() as i32 * 9 + 6;
        }
        // The parts legend, drawn with the real colours.
        let lx = r.x + 52;
        let ly = r.y + r.h - 40;
        for (k, (c, name)) in [(GUN, "GUN"), (ENGINE, "ENGINE"), (SCOOP, "SCOOP"), (STEEL, "ARMOR"), (CORE, "CORE")].iter().enumerate() {
            let x = lx + k as i32 * 56;
            fb.rect_fill(x - 1, ly - 1, 6, 6, OUTLINE);
            fb.rect_fill(x, ly, 4, 4, *c);
            fb.text(name, x + 8, ly - 1, TEXT_DIM, None, 1);
        }
        fb.text_centered("CLICK OR PRESS ANY KEY TO PLAY", r.x + r.w / 2, r.y + r.h - 18, ACCENT, None, 1);
    }

    fn pause_overlay(&self, fb: &mut Framebuffer) {
        ui::panel(fb, PAUSE_RECT);
        fb.text_centered("PAUSED", PAUSE_RECT.x + PAUSE_RECT.w / 2, PAUSE_RECT.y + 8, ACCENT, None, 1);
        let m = self.mouse;
        for (r, l) in PAUSE_BTNS.iter().zip(["RESUME", "NEW MATCH", "QUIT"]) {
            ui::button(fb, *r, l, r.contains(m.0, m.1), false);
        }
        fb.text_centered("R NEW MATCH", PAUSE_RECT.x + PAUSE_RECT.w / 2, PAUSE_RECT.y + PAUSE_RECT.h - 14, TEXT_FAINT, None, 1);
    }

    fn end_overlay(&self, fb: &mut Framebuffer) {
        let r = END_RECT;
        ui::panel(fb, r);
        let won = self.w.winner == Some(0);
        let (head, col, line) =
            if won { ("VICTORY", GOOD, "THE ENEMY KING IS LOOSE PIXELS.") } else { ("DEFEAT", BAD, "YOUR KING IS LOOSE PIXELS.") };
        fb.text_centered(head, r.x + r.w / 2, r.y + 10, col, Some(OUTLINE), 2);
        fb.text_centered(line, r.x + r.w / 2, r.y + 32, TEXT, None, 1);
        let t = self.w.tick / 15;
        fb.text_centered(&format!("MATCH TIME {:02}:{:02}", t / 60, t % 60), r.x + r.w / 2, r.y + 46, TEXT_DIM, None, 1);
        let (a, b) = (&self.w.teams[0], &self.w.teams[1]);
        let rows = [
            ("PIXELS MINED", a.mined, b.mined),
            ("SHIPS PRINTED", a.printed, b.printed),
            ("BASES PLANTED", a.planted, b.planted),
            ("SHIPS DESTROYED", a.kills, b.kills),
        ];
        fb.text("YOU", r.x + 196, r.y + 64, team_col(0, 2), None, 1);
        fb.text("ENEMY", r.x + 244, r.y + 64, team_col(1, 2), None, 1);
        for (i, (name, x, y)) in rows.iter().enumerate() {
            let yy = r.y + 78 + i as i32 * 12;
            fb.text(name, r.x + 30, yy, TEXT, None, 1);
            fb.text(&format!("{x}"), r.x + 196, yy, TEXT, None, 1);
            fb.text(&format!("{y}"), r.x + 244, yy, TEXT, None, 1);
        }
        let m = self.mouse;
        ui::button(fb, END_BTNS[0], "NEW MATCH", END_BTNS[0].contains(m.0, m.1), false);
        ui::button(fb, END_BTNS[1], "QUIT", END_BTNS[1].contains(m.0, m.1), false);
    }
}

impl App for Game {
    fn event(&mut self, ev: Ev) {
        Game::event(self, ev)
    }
    fn hold(&mut self, dirs: [bool; 4]) {
        Game::hold(self, dirs)
    }
    fn tick(&mut self) {
        Game::tick(self)
    }
    fn draw(&self, fb: &mut Framebuffer) {
        Game::draw(self, fb)
    }
    fn palette(&self) -> &Pal {
        Game::palette(self)
    }
    fn cursor_kind(&self) -> u8 {
        Game::cursor_kind(self)
    }
    fn crt(&self) -> bool {
        self.crt
    }
    fn toggle_crt(&mut self) {
        Game::toggle_crt(self)
    }
    fn quit(&self) -> bool {
        self.quit
    }
}

fn side(t: usize) -> &'static str {
    if t == 0 {
        "you"
    } else {
        "enemy"
    }
}

/// "YYYY-MM-DD HH:MM:SS UTC" without a date library.
pub fn utc_stamp() -> String {
    let secs = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map(|d| d.as_secs() as i64).unwrap_or(0);
    let (days, rem) = (secs.div_euclid(86_400), secs.rem_euclid(86_400));
    // Civil-from-days (Howard Hinnant).
    let z = days + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z - era * 146_097;
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let y = yoe + era * 400 + if m <= 2 { 1 } else { 0 };
    format!("{y:04}-{m:02}-{d:02} {:02}:{:02}:{:02} UTC", rem / 3600, rem / 60 % 60, rem % 60)
}
