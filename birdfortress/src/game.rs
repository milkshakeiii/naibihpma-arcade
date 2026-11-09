//! The game: title, site selection, the colony screen and the rules pages.

use crate::app::{App, Ev, Key};
use crate::art::Art;
use crate::audio::{Audio, Sfx, Track};
use crate::city::{self, Action, City, Event, Id, Note, Order, Place};
use crate::framebuffer::Framebuffer;
use crate::level::{Goal, Level};
use crate::palette::{self, Pal};
use crate::path::{self, Field};
use crate::render;
use crate::rules::*;
use crate::save::{Log, Save};
use crate::ui::Rect;
use crate::world::Occ;
use std::cell::RefCell;
use std::rc::Rc;

pub const TILE: i32 = 10;
pub const VIEW_W: i32 = 400;
pub const VIEW_H: i32 = 240;
pub const SPEEDS: [i32; 4] = [1, 2, 4, 8];
pub const HELP_PAGES: usize = 6;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Tool {
    Select,
    Build(Kind),
    Infra(Infra),
    Demolish,
}

impl Tool {
    pub fn workplace(self) -> Option<Kind> {
        match self {
            Tool::Build(k) if k.slots() > 0 => Some(k),
            _ => None,
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Screen {
    Title,
    Select,
    Play,
    Help,
}

#[derive(Clone, Debug)]
pub enum Overlay {
    None,
    Intro,
    Results { day: i32, medal: usize, challenge: bool, best: bool },
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Sel {
    B(Id),
    Fortress,
    Source(usize),
}

/// Clickable spots on the inspector card.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Hot {
    Species(Species),
    Order(Good),
    Pause,
    Demolish,
}

pub const R_PLAY: Rect = Rect::new(333, 243, 20, 20);
pub const R_SPEED: Rect = Rect::new(355, 243, 20, 20);
pub const R_HOME: Rect = Rect::new(377, 243, 20, 20);
pub const R_HELP: Rect = Rect::new(355, 1, 14, 11);
pub const R_MENU: Rect = Rect::new(371, 1, 28, 11);
pub const R_MINI: Rect = Rect::new(206, 1, 146, 10);
pub const R_STATUS: Rect = Rect::new(3, 265, 250, 33);
pub const R_GOALS: Rect = Rect::new(255, 265, 142, 33);
pub const R_RESULTS: Rect = Rect::new(40, 30, 320, 180);
pub const R_INTRO: Rect = Rect::new(30, 18, 340, 210);
pub const CARD_W: i32 = 176;

pub fn tool_rect(i: usize) -> Rect {
    Rect::new(3 + i as i32 * 21, 243, 20, 20)
}

/// Hire chips shown in the status box while a workplace tool is out.
pub fn hire_rect(i: usize) -> Rect {
    Rect::new(R_STATUS.x + R_STATUS.w - 64 + i as i32 * 21, R_STATUS.y + 2, 20, 12)
}

/// Buttons inside the results box: next, again, menu.
pub fn result_buttons() -> [Rect; 3] {
    let y = R_RESULTS.y + R_RESULTS.h - 24;
    [Rect::new(R_RESULTS.x + 18, y, 90, 16), Rect::new(R_RESULTS.x + 115, y, 90, 16), Rect::new(R_RESULTS.x + 212, y, 90, 16)]
}

/// Buttons on the site card: start (or continue), start over.
pub fn intro_buttons() -> [Rect; 2] {
    let y = R_INTRO.y + R_INTRO.h - 24;
    [Rect::new(R_INTRO.x + R_INTRO.w / 2 - 100, y, 96, 16), Rect::new(R_INTRO.x + R_INTRO.w / 2 + 4, y, 96, 16)]
}

pub fn select_rect(chapter: usize, i: usize) -> Rect {
    Rect::new(4 + chapter as i32 * 79, 46 + i as i32 * 30, 76, 27)
}

pub const R_SEL_HELP: Rect = Rect::new(120, 276, 76, 16);
pub const R_SEL_QUIT: Rect = Rect::new(204, 276, 76, 16);
pub const R_HELP_PREV: Rect = Rect::new(8, 278, 60, 16);
pub const R_HELP_NEXT: Rect = Rect::new(332, 278, 60, 16);
pub const R_HELP_CLOSE: Rect = Rect::new(170, 278, 60, 16);

/// Something that rises off a building for a moment: a word or a sign.
#[derive(Clone, Debug)]
pub struct Floater {
    pub x: i32,
    pub y: i32,
    pub text: String,
    pub col: u8,
    pub life: i32,
}

/// A speech bubble pointing at a spot in the world (pixels).
#[derive(Clone, Debug)]
pub struct Bubble {
    pub text: String,
    pub x: i32,
    pub y: i32,
    pub until: u32,
}

pub struct Game {
    pub levels: Vec<Level>,
    pub art: Art,
    pub pal: Pal,
    pub title_pal: Pal,
    pub gallery: crate::picture::Gallery,
    pub save: Save,
    pub log: Log,
    pub audio: Option<Audio>,
    pub frame: u32,
    pub quit: bool,
    pub crt: bool,
    pub screen: Screen,
    pub help_page: usize,
    pub help_return: Screen,
    pub unlock_all: bool,
    // Play state.
    pub cur: usize,
    pub city: City,
    pub actions: Vec<(i32, Action)>,
    pub paused: bool,
    pub speed: usize,
    pub tool: Tool,
    pub hire: Species,
    pub sel: Option<Sel>,
    pub mouse: (i32, i32),
    pub cam: (i32, i32),
    pub held: [bool; 4],
    pub msg: Option<(String, u32, bool)>,
    pub overlay: Overlay,
    pub floaters: Vec<Floater>,
    pub tip: Option<Bubble>,
    pub level_start: std::time::Instant,
    /// The reach overlay for a building not yet placed: (x, y, species, stamp) -> field.
    pub ghost_field: RefCell<Option<((i32, i32, Species, u32, Kind), Rc<Field>)>>,
    pub resume_day: Option<i32>,
    last_season: usize,
}

impl Game {
    pub fn new(levels: Vec<Level>, save: Save, log: Log, audio: Option<Audio>) -> Game {
        let city = City::new(&levels[0]);
        let crt = save.crt;
        let mut g = Game {
            levels,
            art: Art::new(),
            pal: palette::game(),
            title_pal: palette::game(),
            gallery: crate::picture::Gallery::new(),
            save,
            log,
            audio,
            frame: 0,
            quit: false,
            crt,
            screen: Screen::Title,
            help_page: 0,
            help_return: Screen::Title,
            unlock_all: false,
            cur: 0,
            city,
            actions: Vec::new(),
            paused: true,
            speed: 0,
            tool: Tool::Select,
            hire: Species::Tribble,
            sel: None,
            mouse: (0, 0),
            cam: (0, 0),
            held: [false; 4],
            msg: None,
            overlay: Overlay::None,
            floaters: Vec::new(),
            tip: None,
            level_start: std::time::Instant::now(),
            ghost_field: RefCell::new(None),
            resume_day: None,
            last_season: 0,
        };
        let slots: Vec<u8> = (0..palette::TITLE_SLOTS as u8).map(|k| palette::TITLE_FIRST + k).collect();
        palette::load_slots(&mut g.title_pal, &slots, &g.gallery.title.colours);
        g.music(Track::Menu);
        g
    }

    pub fn level(&self) -> &Level {
        &self.levels[self.cur]
    }

    pub fn sfx(&self, s: Sfx) {
        if let Some(a) = &self.audio {
            a.play_sfx(s);
        }
    }

    fn music(&mut self, t: Track) {
        if let Some(a) = self.audio.as_mut() {
            a.music(t);
        }
    }

    pub fn tools(&self) -> Vec<Tool> {
        let lv = self.level();
        let mut v = vec![Tool::Select];
        for s in &lv.species {
            v.push(Tool::Build(s.home()));
        }
        v.extend(lv.kinds.iter().filter(|k| !k.is_home()).map(|&k| Tool::Build(k)));
        v.extend(lv.infra.iter().map(|&i| Tool::Infra(i)));
        v.push(Tool::Demolish);
        v
    }

    pub fn unlocked(&self, i: usize) -> bool {
        self.unlock_all || i < 2 || (1..=2).any(|k| i >= k && self.save.rec(&self.levels[i - k].id).done)
    }

    /// Chapters as lists of level indices.
    pub fn chapters(&self) -> Vec<Vec<usize>> {
        let mut out: Vec<Vec<usize>> = Vec::new();
        for (i, l) in self.levels.iter().enumerate() {
            let c = l.chapter.max(1) - 1;
            while out.len() <= c {
                out.push(Vec::new());
            }
            out[c].push(i);
        }
        out
    }

    pub fn say(&mut self, s: &str, bad: bool) {
        self.msg = Some((s.to_string(), self.frame + 60, bad));
        if bad {
            self.sfx(Sfx::Deny);
        }
    }

    // ----- Entering and leaving ---------------------------------------------------------------

    pub fn enter_level(&mut self, i: usize) {
        self.cur = i;
        let rec = self.save.rec(&self.levels[i].id);
        self.city = City::new(&self.levels[i]);
        self.actions.clear();
        self.resume_day = None;
        if !rec.run.is_empty() {
            if let Some((c, _)) = city::replay(&self.levels[i], &rec.run) {
                self.resume_day = Some(c.day());
            }
        }
        self.paused = true;
        self.speed = 0;
        self.tool = Tool::Select;
        self.sel = None;
        self.floaters.clear();
        self.tip = None;
        self.overlay = Overlay::Intro;
        self.screen = Screen::Play;
        self.last_season = 0;
        self.center_on_entrance();
        self.level_start = std::time::Instant::now();
        let (id, name, ch) = (self.level().id.clone(), self.level().name.clone(), self.level().chapter);
        self.log.line(&format!("enter {id} {name}{}", if self.resume_day.is_some() { " (saved run)" } else { "" }));
        palette::load_slots(&mut self.pal, &palette::BACKDROP, &self.gallery.backdrop(ch).colours);
        self.music(Track::for_chapter(ch));
    }

    /// Starts the site fresh or from the saved run.
    pub fn begin(&mut self, resume: bool) {
        let id = self.level().id.clone();
        if resume {
            let run = self.save.rec(&id).run;
            if let Some((c, acts)) = city::replay(self.level(), &run) {
                self.city = c;
                self.actions = acts;
                self.last_season = self.city.season();
                self.log.line(&format!("resume {id} at day {}", self.city.day()));
            }
        } else {
            self.city = City::new(self.level());
            self.actions.clear();
            self.save.rec_mut(&id).run.clear();
            self.log.line(&format!("start {id}"));
        }
        self.overlay = Overlay::None;
        self.paused = false;
        self.sfx(Sfx::Page);
    }

    fn store_run(&mut self) {
        let id = self.level().id.clone();
        if self.city.won.is_none() && !self.actions.is_empty() {
            self.save.rec_mut(&id).run = city::run_to_text(&self.actions, self.city.tick);
        }
        self.save.write();
    }

    fn leave_level(&mut self) {
        self.store_run();
        let id = self.level().id.clone();
        self.log.line(&format!("leave {id} at day {} after {}s (pop {}, stage {})", self.city.day(), self.level_start.elapsed().as_secs(), self.city.pop(), self.city.stage));
        self.screen = Screen::Select;
        self.paused = true;
        self.music(Track::Menu);
    }

    fn center_on_entrance(&mut self) {
        let (ex, ey) = self.level().entrance;
        self.cam = (ex * TILE - VIEW_W / 4, ey * TILE - VIEW_H / 2);
        self.clamp_cam();
    }

    pub fn clamp_cam(&mut self) {
        let lv = &self.levels[self.cur];
        let maxx = (lv.w * TILE - VIEW_W).max(0);
        let maxy = (lv.h * TILE - VIEW_H).max(0);
        self.cam.0 = self.cam.0.clamp(0, maxx);
        self.cam.1 = self.cam.1.clamp(0, maxy);
    }

    pub fn scrolls(&self) -> bool {
        let lv = self.level();
        lv.w * TILE > VIEW_W || lv.h * TILE > VIEW_H
    }

    // ----- Geometry ---------------------------------------------------------------------------

    pub fn hovered_tile(&self) -> Option<(i32, i32)> {
        let (mx, my) = self.mouse;
        if self.screen != Screen::Play || my < 12 || my >= VIEW_H || mx < 0 || mx >= VIEW_W {
            return None;
        }
        let (tx, ty) = ((mx + self.cam.0) / TILE, (my + self.cam.1) / TILE);
        if !self.city.world.inside(tx, ty) {
            return None;
        }
        Some((tx, ty))
    }

    /// Where a building of this kind would go with the pointer on (tx, ty): its top-left.
    pub fn place_at(&self, k: Kind, tx: i32, ty: i32) -> (i32, i32) {
        if k == Kind::Camp {
            return (tx, ty);
        }
        let (w, h) = k.size();
        // The pointer marks the bottom-left tile, so a building sits where you point at the floor.
        (tx - (w - 1) / 2, ty - h + 1)
    }

    pub fn hovered_sel(&self) -> Option<Sel> {
        let (tx, ty) = self.hovered_tile()?;
        match self.city.world.occ(tx, ty) {
            Occ::Building(id) => Some(Sel::B(id)),
            Occ::Fortress => Some(Sel::Fortress),
            Occ::Source(i) => Some(Sel::Source(i)),
            _ => None,
        }
    }

    /// Footprint of a selection, in tiles: x, y, w, h.
    pub fn sel_box(&self, s: Sel) -> Option<(i32, i32, i32, i32)> {
        match s {
            Sel::B(id) => self.city.building(id).map(|b| {
                let (w, h) = b.kind.size();
                (b.x, b.y, w, h)
            }),
            Sel::Fortress => Some(self.level().fortress),
            Sel::Source(i) => self.level().sources.get(i).map(|s| (s.x, s.y, 1, 1)),
        }
    }

    /// Converts world pixels to screen pixels.
    pub fn to_screen(&self, x: i32, y: i32) -> (i32, i32) {
        (x - self.cam.0, y - self.cam.1)
    }

    /// The inspector card for the selection, with its clickable spots (screen pixels).
    pub fn card(&self) -> Option<(Rect, Vec<(Rect, Hot)>)> {
        let s = self.sel?;
        let (x, y, w, _h) = self.sel_box(s)?;
        let height = match s {
            Sel::B(id) => {
                let b = self.city.building(id)?;
                match b.kind {
                    Kind::Store => 78,
                    k if k.is_home() => 70,
                    Kind::Garden => 34,
                    _ => 60,
                }
            }
            Sel::Fortress => 30 + 10 * self.level().stages.get(self.city.stage).map(|s| s.need.len() as i32).unwrap_or(1),
            Sel::Source(_) => 40,
        };
        let (sx, sy) = self.to_screen(x * TILE + w * TILE / 2, y * TILE);
        let cx = (sx - CARD_W / 2).clamp(2, VIEW_W - CARD_W - 2);
        let mut cy = sy - height - 6;
        if cy < 14 {
            let (_, by) = self.to_screen(0, (y + self.sel_box(s)?.3) * TILE);
            cy = (by + 6).min(VIEW_H - height - 2);
        }
        let r = Rect::new(cx, cy, CARD_W, height);
        let mut hots = Vec::new();
        if let Sel::B(id) = s {
            let b = self.city.building(id)?;
            if b.kind.slots() > 0 {
                for (i, sp) in SPECIES.iter().enumerate() {
                    hots.push((Rect::new(r.x + r.w - 66 + i as i32 * 21, r.y + 3, 20, 12), Hot::Species(*sp)));
                }
                hots.push((Rect::new(r.x + r.w - 76, r.y + r.h - 15, 34, 12), Hot::Pause));
            }
            hots.push((Rect::new(r.x + r.w - 40, r.y + r.h - 15, 37, 12), Hot::Demolish));
            if b.kind == Kind::Store {
                for (i, g) in self.store_goods().into_iter().enumerate() {
                    let col = i as i32 % 6;
                    let row = i as i32 / 6;
                    hots.push((Rect::new(r.x + 4 + col * 28, r.y + 30 + row * 20, 26, 18), Hot::Order(g)));
                }
            }
        }
        Some((r, hots))
    }

    /// The goods a store's order grid shows: everything that can exist here.
    pub fn store_goods(&self) -> Vec<Good> {
        self.level().goods()
    }

    // ----- Tips --------------------------------------------------------------------------------

    /// Shows a one-time tip bubble pointing at a world spot, unless seen before or another is up.
    pub fn tip(&mut self, key: &str, text: &str, at: (i32, i32)) {
        if self.save.seen(key) || self.tip.is_some() {
            return;
        }
        self.save.tips.push(key.to_string());
        self.tip = Some(Bubble { text: text.to_string(), x: at.0, y: at.1, until: self.frame + 200 });
        self.log.line(&format!("tip {key}"));
    }

    fn building_px(&self, id: Id) -> (i32, i32) {
        self.city.building(id).map(|b| (b.x * TILE + b.kind.size().0 * TILE / 2, b.y * TILE)).unwrap_or((0, 0))
    }

    /// Looks over the colony for things worth a first-time tip.
    fn watch(&mut self) {
        if self.tip.is_some() || !matches!(self.overlay, Overlay::None) {
            return;
        }
        let notes: Vec<(Id, Note, Kind)> = self.city.buildings.iter().map(|b| (b.id, b.note, b.kind)).collect();
        for (id, n, k) in notes {
            let at = self.building_px(id);
            match n {
                Note::Hungry if self.city.building(id).is_some_and(|b| b.residents > 0) => {
                    self.tip("hungry", "Hungry! Put a camp on a berry bush in reach.", at);
                }
                Note::NoWorkers(s) => self.tip("noworkers", &format!("No {} free. Every resident works: build homes.", s.plural()), at),
                Note::NoReach(s) => self.tip("noreach", &format!("No {} home can walk here.", s.title().to_lowercase()), at),
                Note::NoSettlers => self.tip("nosettlers", "Settlers can't walk here from the trail.", at),
                Note::Stuck(g) => self.tip("stuck", &format!("Nobody in reach can take {}. A store takes anything.", g.words()), at),
                Note::Bare if k == Kind::Camp => self.tip("bare", "Picked bare. It grows back.", at),
                Note::Lacks(g) => self.tip(&format!("lacks_{}", g.word()), &format!("Wants {} to stay at this tier.", g.words()), at),
                _ => {}
            }
            if self.tip.is_some() {
                return;
            }
        }
        let season = self.city.season();
        if season != self.last_season {
            self.last_season = season;
            let goods = self.level().goods();
            let (ex, ey) = self.level().entrance;
            let at = (ex * TILE + 20, ey * TILE);
            match season {
                3 => self.tip("winter", "Winter. Berries, nuts and fish stop growing.", at),
                2 if goods.contains(&Good::Nut) => self.tip("autumn", "Autumn. Nut trees fruit now, and only now.", at),
                _ => {}
            }
        }
    }

    // ----- Running ----------------------------------------------------------------------------

    pub fn advance(&mut self, n: i32) {
        for _ in 0..n {
            if self.city.won.is_some() {
                break;
            }
            self.city.step();
            let evs: Vec<Event> = self.city.events.drain(..).collect();
            for e in evs {
                self.on_event(e);
            }
        }
    }

    fn float(&mut self, at: (i32, i32), text: &str, col: u8) {
        if self.floaters.len() < 24 {
            self.floaters.push(Floater { x: at.0, y: at.1, text: text.to_string(), col, life: 24 });
        }
    }

    fn on_event(&mut self, e: Event) {
        use crate::palette::*;
        match e {
            Event::Settled(id) => {
                let at = self.building_px(id);
                self.float(at, "+1", GOOD);
                if self.save.tips.iter().all(|t| t != "settled") {
                    self.tip("settled", "A settler moved in. Every resident is a worker.", at);
                }
            }
            Event::TierUp(id, t) => {
                let at = self.building_px(id);
                self.float(at, TIER_NAMES[t], GOLD);
                self.sfx(Sfx::Bell);
                self.tip("tierup", "Moved up! Bigger homes hold more and pay more.", at);
            }
            Event::TierDown(id, t) => {
                let at = self.building_px(id);
                self.float(at, TIER_NAMES[t], BAD);
                self.sfx(Sfx::Stall);
                self.tip("slip", "It slipped back: it ran out of something it needs.", at);
            }
            Event::Left(id) => {
                let at = self.building_px(id);
                self.float(at, "-1", BAD);
                self.sfx(Sfx::Stall);
            }
            Event::Stage(i) => {
                let name = self.level().stages.get(i).map(|s| s.name.clone()).unwrap_or_default();
                self.say(&format!("The fortress: {name} done!"), false);
                self.sfx(Sfx::Win);
                let (fx, fy, fw, _) = self.level().fortress;
                self.float((fx * TILE + fw * TILE / 2, fy * TILE), &name, GOLD);
                self.log.line(&format!("stage {} ({name}) at day {}", i + 1, self.city.day()));
            }
            Event::Won => self.finish(),
            Event::Crafted(_) | Event::Delivered(..) | Event::Broke(_) => {}
        }
    }

    fn finish(&mut self) {
        let day = self.city.won.unwrap_or(self.city.day());
        let lv = self.level().clone();
        let medal = lv.medal_for(day);
        let challenge = lv.challenge != crate::level::Challenge::None && !self.city.broke_challenge;
        let rec = self.save.rec_mut(&lv.id);
        let best = rec.best_day.map_or(true, |b| day < b);
        rec.done = true;
        if best {
            rec.best_day = Some(day);
        }
        if challenge {
            rec.challenge = true;
        }
        rec.run.clear();
        self.save.write();
        self.paused = true;
        self.overlay = Overlay::Results { day, medal, challenge, best };
        self.sfx(Sfx::Win);
        self.log.line(&format!(
            "WON {} day {day} medal {} challenge {} after {}s | peak pop {} spent {} slips {} | {}",
            lv.id,
            ["gold", "silver", "bronze", "none"][medal],
            challenge,
            self.level_start.elapsed().as_secs(),
            self.city.peak_pop,
            self.city.spent,
            self.city.slipped,
            city::run_to_text(&self.actions, self.city.tick)
        ));
    }

    pub fn act(&mut self, a: Action) -> bool {
        match self.city.apply(&a) {
            Ok(()) => {
                self.actions.push((self.city.tick, a.clone()));
                self.log.line(&format!("d{} {}", self.city.day(), a.text().to_lowercase()));
                true
            }
            Err(e) => {
                self.say(&e, true);
                false
            }
        }
    }

    // ----- Input ------------------------------------------------------------------------------

    fn set_tool(&mut self, t: Tool) {
        self.tool = t;
        self.sfx(Sfx::Click);
        if let Some(k) = t.workplace() {
            let at = self.mouse_world();
            self.tip("hire", &format!("Pick who works at the {}: dots show where they can walk.", k.title().to_lowercase()), at);
        } else if let Tool::Build(k) = t {
            if k.is_home() {
                let (ex, ey) = self.level().entrance;
                self.tip("home", "Settlers walk in from this trail. Homes need food.", (ex * TILE + 5, ey * TILE));
            }
        }
    }

    fn mouse_world(&self) -> (i32, i32) {
        (self.mouse.0 + self.cam.0, self.mouse.1 + self.cam.1)
    }

    fn click_map(&mut self, tx: i32, ty: i32) {
        match self.tool {
            Tool::Select => {
                let s = self.hovered_sel();
                self.sel = if s == self.sel { None } else { s };
                if self.sel.is_some() {
                    self.sfx(Sfx::Click);
                }
            }
            Tool::Build(k) => {
                let (x, y) = self.place_at(k, tx, ty);
                let species = if k.slots() > 0 { self.hire } else { k.resident().unwrap_or(Species::Tribble) };
                if self.act(Action::Build { kind: k, x, y, species }) {
                    self.sfx(Sfx::Place);
                    if k == Kind::Store {
                        let id = self.city.buildings.last().map(|b| b.id).unwrap_or(0);
                        let at = self.building_px(id);
                        self.tip("store", "Store carriers take goods to anyone in reach. Click it to set orders.", at);
                    }
                }
            }
            Tool::Infra(i) => {
                if self.act(Action::Infra { infra: i, x: tx, y: ty }) {
                    self.sfx(Sfx::Place);
                }
            }
            Tool::Demolish => {
                if self.act(Action::Demolish { x: tx, y: ty }) {
                    self.sfx(Sfx::Erase);
                    if let Some(Sel::B(id)) = self.sel {
                        if self.city.building(id).is_none() {
                            self.sel = None;
                        }
                    }
                }
            }
        }
    }

    fn card_click(&mut self, x: i32, y: i32) -> bool {
        let Some((r, hots)) = self.card() else { return false };
        if !r.contains(x, y) {
            return false;
        }
        let Some(Sel::B(id)) = self.sel else { return true };
        for (hr, h) in hots {
            if !hr.contains(x, y) {
                continue;
            }
            match h {
                Hot::Species(s) => {
                    if self.city.building(id).is_some_and(|b| b.species != s) && self.act(Action::SetSpecies { id, species: s }) {
                        self.hire = s;
                        self.sfx(Sfx::Click);
                    }
                }
                Hot::Order(g) => {
                    let cur = self.city.building(id).map(|b| b.orders[g.index()]).unwrap_or(Order::Accept);
                    let next = match cur {
                        Order::Accept => Order::Get,
                        Order::Get => Order::Refuse,
                        Order::Refuse => Order::Accept,
                    };
                    if self.act(Action::SetOrder { id, good: g, order: next }) {
                        self.sfx(Sfx::Click);
                    }
                }
                Hot::Pause => {
                    let p = self.city.building(id).map(|b| b.paused).unwrap_or(false);
                    if self.act(Action::Pause { id, paused: !p }) {
                        self.sfx(Sfx::Click);
                    }
                }
                Hot::Demolish => {
                    if let Some(b) = self.city.building(id) {
                        let (bx, by) = (b.x, b.y);
                        if self.act(Action::Demolish { x: bx, y: by }) {
                            self.sfx(Sfx::Erase);
                            self.sel = None;
                        }
                    }
                }
            }
            return true;
        }
        true
    }

    fn play_press(&mut self, x: i32, y: i32, right: bool) {
        match self.overlay.clone() {
            Overlay::Intro => {
                if right {
                    return;
                }
                let [a, b] = intro_buttons();
                if a.contains(x, y) {
                    self.begin(self.resume_day.is_some());
                } else if b.contains(x, y) && self.resume_day.is_some() {
                    self.begin(false);
                }
                return;
            }
            Overlay::Results { .. } => {
                let [next, again, menu] = result_buttons();
                if next.contains(x, y) {
                    let n = self.cur + 1;
                    self.leave_level();
                    if n < self.levels.len() {
                        self.enter_level(n);
                    }
                } else if again.contains(x, y) {
                    let c = self.cur;
                    self.enter_level(c);
                    self.begin(false);
                } else if menu.contains(x, y) {
                    self.leave_level();
                }
                return;
            }
            Overlay::None => {}
        }
        if right {
            if self.tool != Tool::Select {
                self.tool = Tool::Select;
            } else {
                self.sel = None;
            }
            return;
        }
        if self.card_click(x, y) {
            return;
        }
        if R_MENU.contains(x, y) {
            self.leave_level();
            return;
        }
        if R_HELP.contains(x, y) {
            self.open_help(Screen::Play);
            return;
        }
        if R_MINI.contains(x, y) && self.scrolls() {
            let lv = self.level();
            let (w, h) = (lv.w, lv.h);
            let tx = (x - R_MINI.x) * w / R_MINI.w;
            let ty = (y - R_MINI.y) * h / R_MINI.h;
            self.cam = (tx * TILE - VIEW_W / 2, ty * TILE - VIEW_H / 2);
            self.clamp_cam();
            return;
        }
        if R_PLAY.contains(x, y) {
            self.paused = !self.paused;
            self.sfx(Sfx::Click);
            return;
        }
        if R_SPEED.contains(x, y) {
            self.speed = (self.speed + 1) % SPEEDS.len();
            self.paused = false;
            self.sfx(Sfx::Click);
            return;
        }
        if R_HOME.contains(x, y) {
            self.center_on_entrance();
            self.sfx(Sfx::Click);
            return;
        }
        if self.tool.workplace().is_some() {
            for (i, s) in SPECIES.iter().enumerate() {
                if hire_rect(i).contains(x, y) {
                    self.hire = *s;
                    self.sfx(Sfx::Click);
                    return;
                }
            }
        }
        let tools = self.tools();
        for (i, t) in tools.iter().enumerate() {
            if tool_rect(i).contains(x, y) {
                self.set_tool(*t);
                return;
            }
        }
        if let Some((tx, ty)) = self.hovered_tile() {
            self.click_map(tx, ty);
        }
    }

    fn open_help(&mut self, ret: Screen) {
        self.help_return = ret;
        self.screen = Screen::Help;
        self.paused = true;
        self.sfx(Sfx::Page);
    }

    fn play_key(&mut self, k: Key) {
        match &self.overlay {
            Overlay::Intro => {
                match k {
                    Key::Enter | Key::Space => self.begin(self.resume_day.is_some()),
                    Key::Esc => self.leave_level(),
                    Key::Help => self.open_help(Screen::Play),
                    _ => {}
                }
                return;
            }
            Overlay::Results { .. } => {
                match k {
                    Key::Enter => {
                        let n = self.cur + 1;
                        self.leave_level();
                        if n < self.levels.len() {
                            self.enter_level(n);
                        }
                    }
                    Key::Esc => self.leave_level(),
                    _ => {}
                }
                return;
            }
            Overlay::None => {}
        }
        match k {
            Key::Space => {
                self.paused = !self.paused;
                self.sfx(Sfx::Click);
            }
            Key::Tab | Key::Letter('F') => {
                self.speed = (self.speed + 1) % SPEEDS.len();
                self.sfx(Sfx::Click);
            }
            Key::Letter('H') => self.center_on_entrance(),
            Key::Letter('X') | Key::Num(0) => self.set_tool(Tool::Demolish),
            Key::Letter('T') => self.hire = Species::Tribble,
            Key::Letter('D') => self.hire = Species::Dodo,
            Key::Letter('A') => self.hire = Species::Ape,
            Key::Help => self.open_help(Screen::Play),
            Key::Num(n) => {
                let tools = self.tools();
                if let Some(&t) = tools.get((n as usize).wrapping_sub(1)) {
                    self.set_tool(t);
                }
            }
            Key::Esc => {
                if self.tool != Tool::Select {
                    self.tool = Tool::Select;
                } else if self.sel.is_some() {
                    self.sel = None;
                } else {
                    self.leave_level();
                }
            }
            _ => {}
        }
    }

    // ----- Other screens ----------------------------------------------------------------------

    fn select_press(&mut self, x: i32, y: i32) {
        for (c, list) in self.chapters().into_iter().enumerate() {
            for (i, li) in list.into_iter().enumerate() {
                if select_rect(c, i).contains(x, y) {
                    if self.unlocked(li) {
                        self.sfx(Sfx::Click);
                        self.enter_level(li);
                    } else {
                        self.sfx(Sfx::Deny);
                    }
                    return;
                }
            }
        }
        if R_SEL_HELP.contains(x, y) {
            self.open_help(Screen::Select);
        } else if R_SEL_QUIT.contains(x, y) {
            self.quit = true;
        }
    }

    fn help_press(&mut self, x: i32, y: i32) {
        if R_HELP_PREV.contains(x, y) && self.help_page > 0 {
            self.help_page -= 1;
            self.sfx(Sfx::Page);
        } else if R_HELP_NEXT.contains(x, y) && self.help_page + 1 < HELP_PAGES {
            self.help_page += 1;
            self.sfx(Sfx::Page);
        } else if R_HELP_CLOSE.contains(x, y) {
            self.screen = self.help_return;
            self.sfx(Sfx::Page);
        }
    }

    /// The first site not yet finished (where "continue" goes).
    pub fn next_unsolved(&self) -> usize {
        (0..self.levels.len()).find(|&i| !self.save.rec(&self.levels[i].id).done).unwrap_or(0)
    }

    // ----- Status text ------------------------------------------------------------------------

    /// What a building is up to, in a few words.
    pub fn note_text(&self, b: &crate::city::Building) -> (String, bool) {
        let good = |s: String| (s, false);
        let bad = |s: String| (s, true);
        match b.note {
            Note::Fine => {
                if b.kind.is_home() {
                    if b.residents == 0 && b.coming > 0 {
                        return good("Settlers on the way.".into());
                    }
                    if b.tier + 1 < TIERS && self.city.tier_possible(b.tier + 1) {
                        if let Some(n) = self.city.missing(b, b.tier + 1) {
                            return good(match n {
                                Note::Lacks(g) => format!("To grow: {}.", g.words()),
                                _ => "Growing.".into(),
                            });
                        }
                        if self.city.pleasant(b) < TIER_NEEDS[b.tier + 1].pleasant {
                            return good("To grow: a pleasant spot (a garden, no noise).".into());
                        }
                        if b.residents < residents(b.species, b.tier) {
                            return good("Filling up.".into());
                        }
                        return good("Moving up soon.".into());
                    }
                    return good("Content.".into());
                }
                match b.kind {
                    Kind::Camp => good("Gathering.".into()),
                    Kind::Store => good(if b.total() > 0 { "Supplying." } else { "Waiting for goods." }.into()),
                    Kind::Garden => good("Pleasant.".into()),
                    _ => good("Working.".into()),
                }
            }
            Note::NoWorkers(s) => bad(format!("No free {}.", s.plural())),
            Note::NoReach(s) => bad(format!("No {} home can walk here.", s.title().to_lowercase())),
            Note::Bare => good("Picked bare. Growing back.".into()),
            Note::NeedInput(g) => bad(format!("Wants {}.", g.words())),
            Note::Stuck(g) => bad(format!("Nobody in reach takes {}.", g.words())),
            Note::NoSettlers => bad("Settlers can't walk here.".into()),
            Note::Lacks(g) => bad(format!("Needs {} or it slips.", g.words())),
            Note::Hungry => bad("Hungry!".into()),
            Note::Full => good("Full.".into()),
            Note::Stocked(g) => good(format!("Everyone in reach has {}.", g.words())),
            Note::Paused => good("Paused.".into()),
        }
    }

    /// The status line: messages first, then whatever the pointer is over, then the tool.
    pub fn status_line(&self) -> (String, bool) {
        if let Some((m, until, bad)) = &self.msg {
            if self.frame < *until {
                return (m.clone(), *bad);
            }
        }
        if self.overlay_open() {
            return (String::new(), false);
        }
        let hover = self.hovered_tile();
        match self.tool {
            Tool::Build(k) => {
                let base = format!("{}: {} seeds. {}", k.title(), k.cost(), k.blurb());
                if let Some((tx, ty)) = hover {
                    let (x, y) = self.place_at(k, tx, ty);
                    if let Err(e) = self.city.check(&Action::Build { kind: k, x, y, species: self.hire }) {
                        return (e, true);
                    }
                    if let Some(s) = k.resident() {
                        if !self.settlers_can_reach(k, x, y, s) {
                            return (format!("Settlers can't walk here: no {} could move in.", s.title().to_lowercase()), true);
                        }
                    }
                    if k == Kind::Camp {
                        if let Some(i) = self.level().source_at(x, y) {
                            let g = self.level().sources[i].good;
                            return (format!("Camp on the {}: {} weigh {}.", g.source_name().to_lowercase(), cap(g.words()), g.weight().word()), false);
                        }
                    }
                }
                return (base, false);
            }
            Tool::Infra(i) => return (format!("{}: {}", i.title(), i.blurb()), false),
            Tool::Demolish => return ("Demolish: half the seeds come back.".into(), false),
            Tool::Select => {}
        }
        if let Some((tx, ty)) = hover {
            match self.hovered_sel() {
                Some(Sel::B(id)) => {
                    if let Some(b) = self.city.building(id) {
                        let (t, bad) = self.note_text(b);
                        return (format!("{}. {t}", self.building_title(b)), bad);
                    }
                }
                Some(Sel::Fortress) => return (self.fortress_line(), false),
                Some(Sel::Source(i)) => {
                    let s = &self.level().sources[i];
                    let stock = if s.infinite() { "Endless".to_string() } else { format!("{} of {}", self.city.src_stock[i], s.max) };
                    return (format!("{}. {stock}. Build a camp on it.", s.good.source_name()), false);
                }
                None => {}
            }
            if (tx, ty) == self.level().entrance {
                return ("The trail. Settlers walk in here.".into(), false);
            }
            let t = self.city.world.tile(tx, ty).label();
            if !t.is_empty() {
                return (t.to_string(), false);
            }
        }
        ("Pick a tool below. Space pauses, Tab changes speed.".into(), false)
    }

    pub fn overlay_open(&self) -> bool {
        !matches!(self.overlay, Overlay::None)
    }

    pub fn building_title(&self, b: &crate::city::Building) -> String {
        if b.kind.is_home() {
            format!("{} ({})", b.kind.title(), TIER_NAMES[b.tier])
        } else if b.kind == Kind::Camp {
            let g = b.source.map(|s| self.level().sources[s].good).unwrap_or(Good::Berry);
            format!("{} camp", g.source_name())
        } else {
            b.kind.title().to_string()
        }
    }

    pub fn fortress_line(&self) -> String {
        match self.level().stages.get(self.city.stage) {
            Some(st) => {
                let parts: Vec<String> = st.need.iter().map(|(g, n)| format!("{}/{} {}", self.city.fort_stock[g.index()].min(*n), n, g.words())).collect();
                format!("The fortress, {}: {}.", st.name.to_lowercase(), join_and(&parts))
            }
            None => "The fortress stands.".into(),
        }
    }

    pub fn settlers_can_reach(&self, k: Kind, x: i32, y: i32, s: Species) -> bool {
        let f = self.city.field(Place::Entrance, s, Load::EMPTY);
        let (w, h) = k.size();
        (0..w).any(|dx| f.at(&self.city.world, x + dx, y + h - 1) < path::FAR)
    }

    /// The reach overlay's field for the ghost of a workplace about to be placed.
    pub fn ghost_reach(&self, k: Kind, x: i32, y: i32, s: Species) -> Rc<Field> {
        let key = (x, y, s, self.city.world.stamp, k);
        if let Some((kk, f)) = self.ghost_field.borrow().as_ref() {
            if *kk == key {
                return f.clone();
            }
        }
        let (w, h) = k.size();
        let starts: Vec<(i32, i32)> = (0..w).map(|dx| (x + dx, y + h - 1)).collect();
        let f = Rc::new(path::field(&self.city.world, s, Load::EMPTY, &starts));
        *self.ghost_field.borrow_mut() = Some((key, f.clone()));
        f
    }

    pub fn goal_text(&self, g: Goal) -> (String, bool) {
        let c = &self.city;
        match g {
            Goal::Fortress => {
                let (got, total) = c.fortress_progress();
                (format!("Fortress {got}/{total}"), c.fortress_done())
            }
            Goal::Pop(n) => (format!("Residents {}/{n}", c.pop()), c.pop() >= n),
            Goal::Tier(t, n) => (format!("{}s {}/{n}", TIER_NAMES[t], c.homes_at(t)), c.homes_at(t) >= n),
        }
    }
}

impl App for Game {
    fn event(&mut self, ev: Ev) {
        match ev {
            Ev::Move(x, y) => self.mouse = (x, y),
            Ev::Wheel(d) => {
                if self.screen == Screen::Play {
                    self.cam.0 -= d.signum() * 4 * TILE;
                    self.clamp_cam();
                }
            }
            Ev::Press { x, y, right } => {
                self.mouse = (x, y);
                match self.screen {
                    Screen::Title => {
                        self.sfx(Sfx::Page);
                        self.save.seen_intro = true;
                        self.screen = Screen::Select;
                    }
                    Screen::Select => {
                        if !right {
                            self.select_press(x, y);
                        }
                    }
                    Screen::Help => {
                        if right {
                            self.screen = self.help_return;
                        } else {
                            self.help_press(x, y);
                        }
                    }
                    Screen::Play => self.play_press(x, y, right),
                }
            }
            Ev::Release { .. } => {}
            Ev::Char(_) => {}
            Ev::Key(k) => match self.screen {
                Screen::Title => match k {
                    Key::Esc => self.quit = true,
                    Key::Help => self.open_help(Screen::Title),
                    _ => {
                        self.save.seen_intro = true;
                        self.screen = Screen::Select;
                        self.sfx(Sfx::Page);
                    }
                },
                Screen::Select => match k {
                    Key::Esc => self.screen = Screen::Title,
                    Key::Help => self.open_help(Screen::Select),
                    Key::Enter | Key::Space => {
                        let i = self.next_unsolved();
                        self.enter_level(i);
                    }
                    _ => {}
                },
                Screen::Help => match k {
                    Key::Esc | Key::Help | Key::Enter => self.screen = self.help_return,
                    Key::Letter('A') | Key::Back => self.help_page = self.help_page.saturating_sub(1),
                    Key::Letter('D') | Key::Space | Key::Tab => self.help_page = (self.help_page + 1).min(HELP_PAGES - 1),
                    Key::Num(n) if n >= 1 && (n as usize) <= HELP_PAGES => self.help_page = n as usize - 1,
                    _ => {}
                },
                Screen::Play => self.play_key(k),
            },
        }
    }

    fn hold(&mut self, dirs: [bool; 4]) {
        self.held = dirs;
    }

    fn tick(&mut self) {
        self.frame = self.frame.wrapping_add(1);
        palette::cycle_water(&mut self.pal, self.frame);
        if self.screen == Screen::Play {
            // Scroll: arrow keys, or the pointer at the screen's left or right edge.
            let step = TILE;
            let [up, down, left, right] = self.held;
            let (mx, my) = self.mouse;
            let edge_l = mx <= 1 && my > 12 && my < VIEW_H;
            let edge_r = mx >= VIEW_W - 2 && my > 12 && my < VIEW_H;
            if left || edge_l {
                self.cam.0 -= step;
            }
            if right || edge_r {
                self.cam.0 += step;
            }
            if up {
                self.cam.1 -= step;
            }
            if down {
                self.cam.1 += step;
            }
            self.clamp_cam();
            if !self.paused && !self.overlay_open() {
                self.advance(SPEEDS[self.speed]);
                self.watch();
            }
        }
        if self.tip.as_ref().is_some_and(|t| self.frame > t.until) {
            self.tip = None;
        }
        for f in self.floaters.iter_mut() {
            f.life -= 1;
            if f.life % 2 == 0 {
                f.y -= 1;
            }
        }
        self.floaters.retain(|f| f.life > 0);
        if let Some(Sel::B(id)) = self.sel {
            if self.city.building(id).is_none() {
                self.sel = None;
            }
        }
    }

    fn draw(&self, fb: &mut Framebuffer) {
        render::draw(self, fb);
    }

    fn palette(&self) -> &Pal {
        if matches!(self.screen, Screen::Title | Screen::Select) {
            return &self.title_pal;
        }
        &self.pal
    }

    fn cursor_kind(&self) -> u8 {
        if self.screen == Screen::Play && !self.overlay_open() && self.tool == Tool::Select && self.hovered_sel().is_some() {
            1
        } else {
            0
        }
    }

    fn crt(&self) -> bool {
        self.crt
    }

    fn toggle_crt(&mut self) {
        self.crt = !self.crt;
        self.save.crt = self.crt;
    }

    fn music_step(&mut self) {
        self.save.music = (self.save.music + 1) % 4;
        let lvl = self.save.music;
        if let Some(a) = self.audio.as_mut() {
            a.set_level(lvl);
        }
        self.say(&format!("Music: {}", crate::audio::MUSIC_LEVEL_NAMES[lvl].to_lowercase()), false);
    }

    fn quit(&self) -> bool {
        self.quit
    }
}

impl Drop for Game {
    fn drop(&mut self) {
        if self.screen == Screen::Play {
            self.store_run();
            let id = self.level().id.clone();
            self.log.line(&format!("quit in {id} at day {} (pop {}, stage {})", self.city.day(), self.city.pop(), self.city.stage));
        }
        self.save.write();
        self.log.line("=== session end");
    }
}

pub fn cap(s: &str) -> String {
    let mut c = s.chars();
    match c.next() {
        Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
        None => String::new(),
    }
}

pub fn join_and(parts: &[String]) -> String {
    match parts.len() {
        0 => String::new(),
        1 => parts[0].clone(),
        n => format!("{} and {}", parts[..n - 1].join(", "), parts[n - 1]),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn game() -> Game {
        let levels = crate::level::parse_book(crate::BOOK);
        Game::new(levels, Save::load(None), Log::open(None), None)
    }

    fn click(g: &mut Game, x: i32, y: i32) {
        g.event(Ev::Move(x, y));
        g.event(Ev::Press { x, y, right: false });
        g.event(Ev::Release { x, y, right: false });
    }

    #[test]
    fn first_site_by_mouse() {
        let mut g = game();
        click(&mut g, 200, 150);
        assert_eq!(g.screen, Screen::Select);
        let r = select_rect(0, 0);
        click(&mut g, r.x + 5, r.y + 5);
        assert_eq!(g.screen, Screen::Play);
        let [start, _] = intro_buttons();
        click(&mut g, start.x + 4, start.y + 4);
        assert!(matches!(g.overlay, Overlay::None));
        assert!(!g.paused);
        // Pick the first home tool and drop it somewhere it fits.
        let tools = g.tools();
        let hi = tools.iter().position(|t| matches!(t, Tool::Build(k) if k.is_home())).unwrap();
        let tr = tool_rect(hi);
        click(&mut g, tr.x + 4, tr.y + 4);
        let mut placed = false;
        'outer: for ty in 2..23 {
            for tx in 1..39 {
                let (sx, sy) = (tx * TILE + 5 - g.cam.0, ty * TILE + 5 - g.cam.1);
                if sx < 0 || sx >= VIEW_W || sy < 14 || sy >= VIEW_H {
                    continue;
                }
                let before = g.city.buildings.len();
                g.event(Ev::Move(sx, sy));
                if let Some((x, y)) = g.hovered_tile() {
                    let (bx, by) = g.place_at(Kind::Burrow, x, y);
                    if g.city.check(&Action::Build { kind: Kind::Burrow, x: bx, y: by, species: Species::Tribble }).is_ok() {
                        click(&mut g, sx, sy);
                        assert_eq!(g.city.buildings.len(), before + 1);
                        placed = true;
                        break 'outer;
                    }
                }
            }
        }
        assert!(placed, "found nowhere to build a burrow");
        for _ in 0..200 {
            g.tick();
        }
        assert!(g.city.tick > 100);
    }

    #[test]
    fn saved_runs_resume() {
        let mut g = game();
        g.enter_level(0);
        g.begin(false);
        let lv = g.level().clone();
        let steps = crate::bot::parse_plan(&lv.plans[0]).unwrap();
        // Do the plan's first few builds by hand at different ticks.
        let mut done = 0;
        for st in steps.iter() {
            if let crate::bot::Step::Do(a) = st {
                for _ in 0..30 {
                    g.tick();
                }
                if g.city.check(a).is_ok() {
                    g.act(a.clone());
                    done += 1;
                }
            }
            if done >= 3 {
                break;
            }
        }
        for _ in 0..100 {
            g.tick();
        }
        let (tick, pop, seeds) = (g.city.tick, g.city.pop(), g.city.seeds_tenths);
        let text = city::run_to_text(&g.actions, g.city.tick);
        let (c, _) = city::replay(&lv, &text).unwrap();
        assert_eq!((c.tick, c.pop(), c.seeds_tenths), (tick, pop, seeds));
    }
}
