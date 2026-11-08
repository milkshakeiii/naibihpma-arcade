//! The game: title, site selection, the build/run editor and the rules pages.

use crate::app::{App, Ev, Key};
use crate::art::Art;
use crate::audio::{Audio, Sfx, Track};
use crate::framebuffer::Framebuffer;
use crate::layout::{Layout, NodeRef};
use crate::level::Level;
use crate::palette::{self, Pal};
use crate::render;
use crate::rules::*;
use crate::save::{Log, Save};
use crate::sim::{self, CState, Outcome, RoutePlan, Sim};
use crate::ui::Rect;
use crate::world::World;

pub const TILE: i32 = 10;
pub const VIEW_H: i32 = 240;
pub const SPEEDS: [i32; 5] = [1, 2, 4, 8, 32];
pub const HELP_PAGES: usize = 5;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Tool {
    Select,
    Species(Species),
    Build(Kind),
    Infra(Infra),
    Erase,
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
    Rule,
    Results { ticks: i32, cost: i32, crew: i32, best: [bool; 3], combo: bool },
    Failed(String),
}

pub const R_RUN: Rect = Rect::new(307, 243, 20, 20);
pub const R_STEP: Rect = Rect::new(329, 243, 20, 20);
pub const R_SPEED: Rect = Rect::new(351, 243, 24, 20);
pub const R_RESET: Rect = Rect::new(377, 243, 20, 20);
pub const R_HELP: Rect = Rect::new(318, 1, 14, 11);
pub const R_ROUTES: Rect = Rect::new(334, 1, 34, 11);
pub const R_MENU: Rect = Rect::new(370, 1, 28, 11);
pub const R_STATUS: Rect = Rect::new(3, 265, 268, 33);
pub const R_METRICS: Rect = Rect::new(273, 265, 124, 33);
pub const R_RESULTS: Rect = Rect::new(26, 36, 348, 168);
pub const CARD_W: i32 = 166;

pub fn tool_rect(i: usize) -> Rect {
    Rect::new(3 + i as i32 * 22, 243, 20, 20)
}

/// Buttons inside a result/failure box: next, keep tuning, menu.
pub fn result_buttons() -> [Rect; 3] {
    let y = R_RESULTS.y + R_RESULTS.h - 24;
    [Rect::new(R_RESULTS.x + 24, y, 90, 16), Rect::new(R_RESULTS.x + 129, y, 90, 16), Rect::new(R_RESULTS.x + 234, y, 90, 16)]
}

pub fn select_rect(chapter: usize, i: usize) -> Rect {
    Rect::new(4 + chapter as i32 * 79, 46 + i as i32 * 30, 76, 27)
}

pub const R_SEL_HELP: Rect = Rect::new(120, 276, 76, 16);
pub const R_SEL_QUIT: Rect = Rect::new(204, 276, 76, 16);
pub const R_HELP_PREV: Rect = Rect::new(8, 278, 60, 16);
pub const R_HELP_NEXT: Rect = Rect::new(332, 278, 60, 16);
pub const R_HELP_CLOSE: Rect = Rect::new(170, 278, 60, 16);

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
    pub title_hover: bool,
    // Play state.
    pub cur: usize,
    pub layout: Layout,
    pub undo: Vec<Layout>,
    pub world: World,
    pub plans: Vec<RoutePlan>,
    pub sim: Option<Sim>,
    pub running: bool,
    pub speed: usize,
    pub tool: Tool,
    pub from: Option<NodeRef>,
    pub selected: Option<NodeRef>,
    pub preview: Option<(NodeRef, Good, RoutePlan)>,
    pub mouse: (i32, i32),
    pub msg: Option<(String, u32, bool)>,
    pub overlay: Overlay,
    pub show_routes: bool,
    pub sparks: Vec<(i32, i32, i32, u8)>,
    pub level_start: std::time::Instant,
    pub histos: Vec<(String, crate::harness::Histo)>,
    pub tip: Option<Bubble>,
}

impl Game {
    pub fn new(levels: Vec<Level>, save: Save, log: Log, audio: Option<Audio>) -> Game {
        let world = World::new(&levels[0], &Layout::default());
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
            title_hover: false,
            cur: 0,
            layout: Layout::default(),
            undo: Vec::new(),
            world,
            plans: Vec::new(),
            sim: None,
            running: false,
            speed: 1,
            tool: Tool::Select,
            from: None,
            selected: None,
            preview: None,
            mouse: (0, 0),
            msg: None,
            overlay: Overlay::None,
            show_routes: true,
            sparks: Vec::new(),
            level_start: std::time::Instant::now(),
            histos: crate::harness::parse_histos(include_str!("levels/histo.txt")),
            tip: None,
        };
        let slots: Vec<u8> = (0..palette::TITLE_SLOTS as u8).map(|k| palette::TITLE_FIRST + k).collect();
        palette::load_slots(&mut g.title_pal, &slots, &g.gallery.title.colours);
        g.music(Track::Menu);
        g
    }

    pub fn level(&self) -> &Level {
        &self.levels[self.cur]
    }

    fn sfx(&self, s: Sfx) {
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
        v.extend(lv.species.iter().map(|&s| Tool::Species(s)));
        v.extend(lv.kinds.iter().map(|&k| Tool::Build(k)));
        v.extend(lv.infra.iter().map(|&i| Tool::Infra(i)));
        v.push(Tool::Erase);
        v
    }

    pub fn unlocked(&self, i: usize) -> bool {
        self.unlock_all || i < 2 || (1..=2).any(|k| i >= k && self.save.rec(&self.levels[i - k].id).solved)
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
        self.msg = Some((s.to_string(), self.frame + 45, bad));
        if bad {
            self.sfx(Sfx::Deny);
        }
    }

    // ----- Play state -------------------------------------------------------------------------

    pub fn enter_level(&mut self, i: usize) {
        self.cur = i;
        let rec = self.save.rec(&self.levels[i].id);
        self.layout = Layout::from_text(&rec.layout);
        self.undo.clear();
        self.sim = None;
        self.running = false;
        self.tool = Tool::Select;
        self.from = None;
        self.selected = None;
        self.preview = None;
        self.sparks.clear();
        self.overlay = Overlay::Rule;
        self.screen = Screen::Play;
        self.rebuild();
        self.level_start = std::time::Instant::now();
        let (id, name, ch) = (self.level().id.clone(), self.level().name.clone(), self.level().chapter);
        self.log.line(&format!("enter {id} {name} (layout: {})", if rec.layout.is_empty() { "empty" } else { "saved" }));
        palette::load_slots(&mut self.pal, &palette::BACKDROP, &self.gallery.backdrop(ch).colours);
        self.music(Track::for_chapter(ch));
    }

    fn leave_level(&mut self) {
        let id = self.level().id.clone();
        self.save.rec_mut(&id).layout = self.layout.to_text();
        self.save.write();
        self.log.line(&format!("leave {id} after {}s", self.level_start.elapsed().as_secs()));
        self.screen = Screen::Select;
        self.sim = None;
        self.running = false;
        self.music(Track::Menu);
    }

    pub fn rebuild(&mut self) {
        let lv = &self.levels[self.cur];
        self.world = World::new(lv, &self.layout);
        self.plans = self.layout.routes.iter().map(|r| sim::plan_route(lv, &self.layout, &self.world, r)).collect();
        let id = lv.id.clone();
        self.save.rec_mut(&id).layout = self.layout.to_text();
    }

    /// Call before any change to the layout: stops the run and remembers the old layout.
    fn begin_edit(&mut self) {
        self.sim = None;
        self.running = false;
        self.sparks.clear();
        self.undo.push(self.layout.clone());
        if self.undo.len() > 200 {
            self.undo.remove(0);
        }
    }

    fn undo(&mut self) {
        if let Some(l) = self.undo.pop() {
            self.sim = None;
            self.running = false;
            self.layout = l;
            self.from = None;
            if let Some(NodeRef::Building(id)) = self.selected {
                if self.layout.building(id).is_none() {
                    self.selected = None;
                }
            }
            self.rebuild();
            self.say("Undone.", false);
            self.sfx(Sfx::Erase);
        } else {
            self.say("Nothing to undo.", true);
        }
    }

    pub fn hovered_tile(&self) -> Option<(i32, i32)> {
        let (mx, my) = self.mouse;
        if self.screen != Screen::Play || my < 0 || my >= VIEW_H || mx < 0 || mx >= 400 {
            return None;
        }
        Some((mx / TILE, my / TILE))
    }

    pub fn hovered_node(&self) -> Option<NodeRef> {
        let (tx, ty) = self.hovered_tile()?;
        self.world.node_at(tx, ty)
    }

    /// A place's name in running text: "twig pile", "fortress", "depot".
    pub fn node_name(&self, n: NodeRef) -> String {
        match n {
            NodeRef::Source(i) => self.level().sources[i].label().to_string(),
            NodeRef::Fortress => "fortress".into(),
            NodeRef::Building(id) => self.layout.building(id).map(|b| b.kind.title().to_lowercase()).unwrap_or_default(),
        }
    }

    /// Centre of a place on screen, in pixels.
    pub fn node_px(&self, n: NodeRef) -> (i32, i32) {
        let (x, y, w, h) = self.node_box(n);
        (x * TILE + w * TILE / 2, y * TILE + h * TILE / 2)
    }

    /// Shows a one-time tip bubble pointing at `at`, unless it was seen before or another is up.
    pub fn tip(&mut self, key: &str, text: &str, at: (i32, i32)) {
        if self.save.seen(key) || self.tip.is_some() {
            return;
        }
        self.save.tips.push(key.to_string());
        self.tip = Some(Bubble { text: text.to_string(), x: at.0, y: at.1, until: self.frame + 180 });
        self.log.line(&format!("tip {key}"));
    }

    /// The guided first site: what to do next, and where to look.
    pub fn coach(&self) -> Option<(String, i32, i32)> {
        if self.screen != Screen::Play || !matches!(self.overlay, Overlay::None) || self.level().id != "1-1" {
            return None;
        }
        if self.save.rec("1-1").solved {
            return None;
        }
        if self.layout.routes.is_empty() {
            let tools = self.tools();
            if self.tool != Tool::Species(Species::Tribble) {
                let i = tools.iter().position(|t| *t == Tool::Species(Species::Tribble)).unwrap_or(1);
                let r = tool_rect(i);
                return Some(("Hire a tribble.".into(), r.x + 10, r.y));
            }
            if self.from.is_none() {
                let (x, y) = self.node_px(NodeRef::Source(0));
                return Some(("Click the twig pile. That's where it picks up.".into(), x, y - 4));
            }
            let (x, y) = self.node_px(NodeRef::Fortress);
            return Some(("Now click the fortress to drop off there.".into(), x, y - 12));
        }
        if self.sim.is_none() {
            return Some(("Press run, or tap Space.".into(), R_RUN.x + 10, R_RUN.y));
        }
        None
    }

    pub fn node_offers_any(&self, n: NodeRef) -> bool {
        match n {
            NodeRef::Source(_) => true,
            NodeRef::Fortress => false,
            NodeRef::Building(_) => true,
        }
    }

    /// Top-left tile of a node's footprint, and its size in tiles.
    pub fn node_box(&self, n: NodeRef) -> (i32, i32, i32, i32) {
        match n {
            NodeRef::Source(i) => {
                let s = &self.level().sources[i];
                (s.x, s.y, 1, 1)
            }
            NodeRef::Fortress => self.level().fortress,
            NodeRef::Building(id) => self.layout.building(id).map(|b| (b.x, b.y, 2, 2)).unwrap_or((0, 0, 0, 0)),
        }
    }

    pub fn card_routes(&self) -> Vec<usize> {
        let Some(n) = self.selected else { return Vec::new() };
        self.layout.routes.iter().enumerate().filter(|(_, r)| r.from == n || r.to == n).map(|(i, _)| i).take(8).collect()
    }

    pub fn card_rect(&self) -> Option<Rect> {
        let n = self.selected?;
        if !matches!(self.overlay, Overlay::None) {
            return None;
        }
        let (x, _, w, _) = self.node_box(n);
        let rows = self.card_routes().len() as i32;
        let h = 26 + rows.max(1) * 12 + 3;
        let cx = if (x + w / 2) * TILE < 200 { 400 - CARD_W - 3 } else { 3 };
        Some(Rect::new(cx, 14, CARD_W, h))
    }

    /// Row i of the card: (row, cargo, minus, plus, delete).
    pub fn card_row(&self, card: Rect, i: usize) -> (Rect, Rect, Rect, Rect, Rect) {
        let y = card.y + 26 + i as i32 * 12;
        (
            Rect::new(card.x + 2, y, card.w - 4, 12),
            Rect::new(card.x + 26, y + 1, 10, 10),
            Rect::new(card.x + 118, y + 1, 10, 10),
            Rect::new(card.x + 138, y + 1, 10, 10),
            Rect::new(card.x + 152, y + 1, 10, 10),
        )
    }

    fn stop_sim(&mut self) {
        self.sim = None;
        self.running = false;
        self.sparks.clear();
    }

    fn start_run(&mut self) -> bool {
        if self.sim.is_some() {
            return true;
        }
        if self.layout.routes.is_empty() {
            self.say("No routes yet. Pick a creature below, then where it picks up and drops off.", true);
            return false;
        }
        match Sim::new(self.level(), &self.layout) {
            Ok(s) => {
                self.sim = Some(s);
                self.overlay = Overlay::None;
                let txt = self.layout.to_text();
                self.log.line(&format!("run {} | {}", self.level().id, txt));
                true
            }
            Err((i, p)) => {
                self.selected = Some(self.layout.routes[i].from);
                self.say(&format!("A {} route can't run: {}.", self.layout.routes[i].species.title().to_lowercase(), p.text()), true);
                false
            }
        }
    }

    fn toggle_run(&mut self) {
        if self.running {
            self.running = false;
            self.sfx(Sfx::Click);
            return;
        }
        if let Some(s) = &self.sim {
            if s.outcome != Outcome::Running {
                self.stop_sim();
            }
        }
        if self.start_run() {
            self.running = true;
            self.tool = Tool::Select;
            self.tip("speed", "Too slow? This speeds it up. Tab does too.", (R_SPEED.x + 12, R_SPEED.y));
            self.from = None;
            self.preview = None;
            self.sfx(Sfx::Click);
        }
    }

    fn step_once(&mut self) {
        if let Some(s) = &self.sim {
            if s.outcome != Outcome::Running {
                self.stop_sim();
            }
        }
        if self.start_run() {
            self.running = false;
            self.advance(1);
        }
    }

    fn reset(&mut self) {
        if self.sim.is_some() {
            self.stop_sim();
            self.sfx(Sfx::Erase);
        }
    }

    fn advance(&mut self, n: i32) {
        let mut sounds: Vec<Sfx> = Vec::new();
        let mut finished = None;
        if let Some(sim) = self.sim.as_mut() {
            for _ in 0..n {
                sim.step();
                for &(x, y, k) in &sim.events {
                    match k {
                        0 => sounds.push(Sfx::Pick),
                        1 => sounds.push(Sfx::Drop),
                        2 => sounds.push(Sfx::Craft),
                        _ => sounds.push(Sfx::Bell),
                    }
                    if k != 2 {
                        let col = if k == 3 { palette::GOLD } else { palette::WHITE };
                        self.sparks.push((x * TILE + 5, y * TILE + 2, 8, col));
                    }
                }
                if sim.outcome != Outcome::Running {
                    finished = Some(sim.outcome.clone());
                    break;
                }
            }
        }
        sounds.sort_by_key(|s| *s as u8);
        sounds.dedup();
        for s in sounds {
            self.sfx(s);
        }
        self.watch_run();
        if let Some(o) = finished {
            self.running = false;
            self.finish(o);
        }
    }

    /// Notices things worth explaining while a run plays: a queue at a dock, an empty bush.
    fn watch_run(&mut self) {
        let Some(sim) = &self.sim else { return };
        if sim.tick < 30 {
            return;
        }
        let mut at_queue = None;
        let mut at_empty = None;
        for c in &sim.creatures {
            if c.state != CState::WaitPickup || c.idle < 12 {
                continue;
            }
            let r = &sim.routes[c.route];
            let n = sim.node(r.from);
            let empty = n.is_some_and(|n| n.good.is_some() && n.max > 0 && n.total() == 0);
            if empty {
                at_empty = Some((c.x * TILE + 5, c.y * TILE - 2));
            } else if sim.creatures.iter().filter(|o| o.x == c.x && o.y == c.y && o.state == CState::WaitPickup).count() >= 2 {
                at_queue = Some((c.x * TILE + 5, c.y * TILE - 2));
            }
        }
        if let Some(at) = at_empty {
            self.tip("regrow", "Picked clean. It grows back slowly, so this crew waits. Other sources could help.", at);
        } else if let Some(at) = at_queue {
            self.tip("queue", "Only one creature fits at a dock at a time. The rest wait their turn.", at);
        }
    }

    fn finish(&mut self, o: Outcome) {
        let id = self.level().id.clone();
        let tg = self.level().targets;
        match o {
            Outcome::Done(_) => {
                let sc = self.sim.as_ref().and_then(|s| s.score()).unwrap();
                let rec = self.save.rec_mut(&id);
                let better = |old: Option<i32>, new: i32| old.map_or(true, |o| new < o);
                let best = [better(rec.ticks, sc.ticks), better(rec.cost, sc.cost), better(rec.crew, sc.crew)];
                if best[0] {
                    rec.ticks = Some(sc.ticks);
                }
                if best[1] {
                    rec.cost = Some(sc.cost);
                }
                if best[2] {
                    rec.crew = Some(sc.crew);
                }
                let combo = tg.combo_ticks > 0 && sc.ticks <= tg.combo_ticks && sc.cost <= tg.combo_cost;
                let new_combo = combo && !rec.combo;
                if combo {
                    rec.combo = true;
                }
                let first = !rec.solved;
                rec.solved = true;
                rec.layout = self.layout.to_text();
                self.save.write();
                self.log.line(&format!(
                    "SOLVED {id} ticks {} cost {} crew {}{}{} after {}s | {}",
                    sc.ticks,
                    sc.cost,
                    sc.crew,
                    if first { " (first time)" } else { "" },
                    if new_combo { " COMBO" } else { "" },
                    self.level_start.elapsed().as_secs(),
                    self.layout.to_text()
                ));
                self.overlay = Overlay::Results { ticks: sc.ticks, cost: sc.cost, crew: sc.crew, best, combo };
                self.sfx(Sfx::Win);
            }
            Outcome::Stalled(t) => {
                let why = self.diagnose();
                self.log.line(&format!("stalled {id} at tick {t}: {why}"));
                self.overlay = Overlay::Failed(format!("Everyone stopped at tick {t}. {why}"));
                self.sfx(Sfx::Stall);
            }
            Outcome::TooSlow => {
                self.log.line(&format!("too slow {id}"));
                self.overlay = Overlay::Failed(format!("Still going after {} ticks. Goods may be going round in circles.", sim::MAX_TICKS));
                self.sfx(Sfx::Stall);
            }
            Outcome::Running => {}
        }
    }

    /// Why is everyone waiting? Names the first stuck creature's problem.
    fn diagnose(&self) -> String {
        let Some(sim) = &self.sim else { return String::new() };
        for c in &sim.creatures {
            let r = &sim.routes[c.route];
            if c.state == CState::WaitPickup {
                let from = sim.node(r.from);
                let has = from.is_some_and(|n| n.stock[r.cargo.index()] > 0 || (n.max <= 0 && matches!(n.node, NodeRef::Source(_))));
                if !has {
                    return cap(&format!("{} at the {} are waiting for {} that never come.", r.species.plural(), self.node_name(r.from), r.cargo.words()));
                }
                return format!("The {} has no room for more {}.", self.node_name(r.to), r.cargo.words());
            }
        }
        "Nobody can move.".into()
    }

    pub fn status_line(&self) -> (String, bool) {
        if let Some((m, until, bad)) = &self.msg {
            if self.frame < *until {
                return (m.clone(), *bad);
            }
        }
        if let Tool::Species(s) = self.tool {
            if self.from.is_some() {
                if let Some((_, g, p)) = &self.preview {
                    return match &p.problem {
                        Some(pr) => (format!("{} with {}: {}.", s.title(), a_good(*g), pr.text()), true),
                        None => (format!("{} with {}: {} ticks a round trip. Click to hire.", s.title(), a_good(*g), p.cycle()), false),
                    };
                }
                return ("Now click where it drops off. Gold dots: it can carry it there. Red: no way home.".into(), false);
            }
            return (format!("Click where the {} picks up: a bush, a pile, a depot or a workshop.", s.title().to_lowercase()), false);
        }
        if let Some((tx, ty)) = self.hovered_tile() {
            if let Some(n) = self.world.node_at(tx, ty) {
                return (self.describe_node(n), false);
            }
            match self.tool {
                Tool::Build(k) => return (format!("{}. {}", k.title(), k.blurb()), false),
                Tool::Infra(i) => return (format!("{}. {}", i.title(), i.blurb()), false),
                Tool::Erase => return ("Click something you built to take it away.".into(), false),
                _ => {}
            }
            if let Some(sim) = &self.sim {
                if let Some(c) = sim.creatures.iter().find(|c| c.x == tx && c.y == ty) {
                    let r = &sim.routes[c.route];
                    let what = match c.state {
                        CState::WaitPickup => format!("waiting at the {}", self.node_name(r.from)),
                        CState::Picking(_) => "picking up".into(),
                        CState::Carry => format!("carrying {} to the {}", a_good(r.cargo), self.node_name(r.to)),
                        CState::WaitDrop => format!("waiting its turn at the {}", self.node_name(r.to)),
                        CState::Dropping(_) => "handing it over".into(),
                        CState::Return => format!("heading back to the {}", self.node_name(r.from)),
                    };
                    return (format!("{}, {}.", c.species.title(), what), false);
                }
            }
            if self.world.is_spring(tx, ty) {
                return ("Springboard. Jumps from here go 2 higher.".into(), false);
            }
            if self.world.is_plank(tx, ty) {
                return ("Plank. Stand on it, or jump up through it.".into(), false);
            }
            let t = self.world.tile(tx, ty);
            if t != crate::level::Tile::Air {
                return (t.label().to_string(), false);
            }
        }
        match self.tool {
            Tool::Build(k) => (format!("{}. {}", k.title(), k.blurb()), false),
            Tool::Infra(i) => (format!("{}. {}", i.title(), i.blurb()), false),
            Tool::Erase => ("Click something you built to take it away.".into(), false),
            _ => {
                if self.sim.is_some() {
                    ("Space pauses. Changing anything starts the run over.".into(), false)
                } else {
                    ("Pick a creature below to start a route. F1 has the rules.".into(), false)
                }
            }
        }
    }

    pub fn describe_node(&self, n: NodeRef) -> String {
        let lv = self.level();
        match n {
            NodeRef::Source(i) => {
                let s = &lv.sources[i];
                let what = if s.infinite() {
                    format!("Never runs out of {}.", s.good.words())
                } else if s.regrow > 0 {
                    format!("Holds {}, grows one back every {} ticks.", s.max, s.regrow)
                } else {
                    format!("Only {}, and no more.", s.good.count(s.max))
                };
                let fragile = if s.good.fragile() { " Fragile!" } else { "" };
                format!("{}. {what} {} {} {}.{fragile}", cap(s.label()), cap(s.good.words()), if s.good == Good::Clay || s.good == Good::Wicker { "is" } else { "are" }, s.good.weight().word())
            }
            NodeRef::Fortress => {
                let parts: Vec<String> = lv.need.iter().map(|(g, c)| g.count(*c)).collect();
                format!("The fortress wants {}.", join_and(&parts))
            }
            NodeRef::Building(id) => match self.layout.building(id) {
                Some(b) => format!("{}. {}", b.kind.title(), b.kind.blurb()),
                None => String::new(),
            },
        }
    }

    // ----- Editing ----------------------------------------------------------------------------

    fn click_map(&mut self, tx: i32, ty: i32) {
        let node = self.world.node_at(tx, ty);
        match self.tool {
            Tool::Select => {
                self.selected = node;
                if node.is_some() {
                    self.sfx(Sfx::Click);
                }
            }
            Tool::Species(s) => self.click_species(s, node),
            Tool::Build(k) => {
                let (x, y) = (tx, ty - 1);
                match self.world.can_build(x, y) {
                    Ok(()) => {
                        self.begin_edit();
                        let id = self.layout.add_building(k, x, y);
                        self.rebuild();
                        self.selected = Some(NodeRef::Building(id));
                        self.sfx(Sfx::Place);
                        let at = (x * TILE + 10, y * TILE - 2);
                        if let Some(r) = k.recipe() {
                            let ins: Vec<String> = r.inputs.iter().map(|(g, n)| g.count(*n)).collect();
                            let text = format!("Bring it {}, and send crews to take the {} away.", join_and(&ins), r.output.words());
                            self.tip("workshop", &text, at);
                        } else {
                            self.tip("depot", "Now one crew can drop off here, and another pick up from here.", at);
                        }
                    }
                    Err(e) => {
                        if let Some(n) = node {
                            self.selected = Some(n);
                        } else {
                            self.say(e, true);
                        }
                    }
                }
            }
            Tool::Infra(i) => {
                let r = match i {
                    Infra::Spring => self.world.can_spring(tx, ty).map(|_| ()),
                    Infra::Ladder => self.world.can_ladder(tx, ty).map(|_| ()),
                    Infra::Plank => self.world.can_plank(tx, ty),
                };
                match r {
                    Ok(()) => {
                        self.begin_edit();
                        match i {
                            Infra::Spring => self.layout.springs.push((tx, ty)),
                            Infra::Ladder => self.layout.ladders.push((tx, ty)),
                            Infra::Plank => self.layout.planks.push((tx, ty)),
                        }
                        self.rebuild();
                        self.sfx(Sfx::Place);
                    }
                    Err(e) => self.say(e, true),
                }
            }
            Tool::Erase => self.erase_at(tx, ty),
        }
    }

    fn click_species(&mut self, s: Species, node: Option<NodeRef>) {
        let Some(n) = node else {
            self.from = None;
            self.preview = None;
            return;
        };
        match self.from {
            None => {
                if self.node_offers_any(n) {
                    self.from = Some(n);
                    self.selected = Some(n);
                    self.sfx(Sfx::Click);
                } else {
                    self.say("The fortress only takes. Pick up from a source, a depot or a workshop.", true);
                }
            }
            Some(f) if f == n => {
                self.from = None;
                self.preview = None;
            }
            Some(f) => {
                let opts = sim::cargo_options(self.level(), &self.layout, f, n);
                let Some(g) = self.default_cargo(f, &opts) else {
                    let msg = format!("Nothing from the {} is wanted at the {}.", self.node_name(f), self.node_name(n));
                    self.say(&msg, true);
                    return;
                };
                self.begin_edit();
                self.layout.add_route(s, f, n, g, 1);
                self.rebuild();
                let ri = self.layout.routes.iter().position(|r| r.species == s && r.from == f && r.to == n && r.cargo == g).unwrap_or(0);
                if let Some(p) = self.plans[ri].problem.clone() {
                    let msg = format!("Hired, but {}.", p.text());
                    self.say(&msg, true);
                    self.route_trouble(f, g);
                } else {
                    self.sfx(Sfx::Place);
                }
                if opts.len() > 1 {
                    let at = self.node_px(f);
                    self.tip("cargo", "This route could carry other things too. Click the little cargo picture on the card to change it.", (at.0, at.1 - 8));
                }
                self.selected = Some(f);
                self.from = None;
                self.preview = None;
            }
        }
    }

    /// Which cargo a new route takes: from a depot, whatever is being brought to it.
    fn default_cargo(&self, from: NodeRef, opts: &[Good]) -> Option<Good> {
        if let NodeRef::Building(id) = from {
            if self.layout.building(id).is_some_and(|b| b.kind == Kind::Depot) {
                if let Some(g) = self.layout.routes.iter().filter(|r| r.to == from).map(|r| r.cargo).find(|g| opts.contains(g)) {
                    return Some(g);
                }
            }
        }
        opts.first().copied()
    }

    /// Just-in-time hints when a route can't run: once each, most specific first.
    fn route_trouble(&mut self, from: NodeRef, cargo: Good) {
        let (x, y, w, _) = self.node_box(from);
        let at = (x * TILE + w * TILE / 2, y * TILE - 6);
        let tools = self.tools();
        let tool_at = |t: Tool| tools.iter().position(|x| *x == t).map(|i| (tool_rect(i).x + 10, tool_rect(i).y));
        if cargo.fragile() && !self.save.seen("fragile") {
            self.tip("fragile", "Fragile goods break if they fall more than a tile, so the carrier won't jump down. Ladders help.", at);
        } else if !self.save.seen("route_fail") {
            self.tip("route_fail", "This crew can't do the whole trip. Pick the creature and click the pickup again: gold dots show where it can carry it, red where it can't get home.", at);
        } else if let Some(p) = tool_at(Tool::Build(Kind::Depot)).filter(|_| !self.save.seen("handoff")) {
            self.tip("handoff", "Stuck? A depot lets two different crews split one trip.", p);
        } else if let Some(p) = tool_at(Tool::Infra(Infra::Spring)).filter(|_| !self.save.seen("spring_hint")) {
            self.tip("spring_hint", "A springboard adds 2 to any jump made from it.", p);
        } else if let Some(p) = tool_at(Tool::Infra(Infra::Ladder)).filter(|_| !self.save.seen("ladder_hint")) {
            self.tip("ladder_hint", "Ladders go straight up and down. Click where the top rung should be.", p);
        }
    }

    /// Is there something the player built at this tile?
    fn erasable_at(&self, tx: i32, ty: i32) -> bool {
        matches!(self.world.node_at(tx, ty), Some(NodeRef::Building(_)))
            || self.layout.springs.contains(&(tx, ty))
            || self.layout.planks.contains(&(tx, ty))
            || self.layout.ladders.iter().enumerate().any(|(k, &(x, y))| x == tx && ty >= y && ty < y + self.world.ladder_lens.get(k).copied().unwrap_or(1))
    }

    fn erase_at(&mut self, tx: i32, ty: i32) {
        if let Some(NodeRef::Building(id)) = self.world.node_at(tx, ty) {
            self.begin_edit();
            self.layout.remove_building(id);
            if self.selected == Some(NodeRef::Building(id)) {
                self.selected = None;
            }
            self.rebuild();
            self.sfx(Sfx::Erase);
            return;
        }
        if let Some(i) = self.layout.springs.iter().position(|&p| p == (tx, ty)) {
            self.begin_edit();
            self.layout.springs.remove(i);
            self.rebuild();
            self.sfx(Sfx::Erase);
            return;
        }
        if let Some(i) = self.layout.planks.iter().position(|&p| p == (tx, ty)) {
            self.begin_edit();
            self.layout.planks.remove(i);
            self.rebuild();
            self.sfx(Sfx::Erase);
            return;
        }
        let lens = self.world.ladder_lens.clone();
        if let Some(i) = self.layout.ladders.iter().enumerate().position(|(k, &(x, y))| x == tx && ty >= y && ty < y + lens.get(k).copied().unwrap_or(1)) {
            self.begin_edit();
            self.layout.ladders.remove(i);
            self.rebuild();
            self.sfx(Sfx::Erase);
            return;
        }
        if self.tool == Tool::Erase {
            self.say("Nothing of yours there to take away.", true);
        }
    }

    fn card_click(&mut self, card: Rect, x: i32, y: i32) -> bool {
        if !card.contains(x, y) {
            return false;
        }
        for (i, ri) in self.card_routes().into_iter().enumerate() {
            let (_, cargo, minus, plus, del) = self.card_row(card, i);
            if minus.contains(x, y) {
                self.begin_edit();
                self.layout.routes[ri].count -= 1;
                if self.layout.routes[ri].count <= 0 {
                    self.layout.routes.remove(ri);
                }
                self.rebuild();
                self.sfx(Sfx::Erase);
            } else if plus.contains(x, y) {
                if self.layout.routes[ri].count >= 9 {
                    self.say("Nine is the most one route can hold.", true);
                } else {
                    self.begin_edit();
                    self.layout.routes[ri].count += 1;
                    self.rebuild();
                    self.sfx(Sfx::Place);
                }
            } else if del.contains(x, y) {
                self.begin_edit();
                self.layout.routes.remove(ri);
                self.rebuild();
                self.sfx(Sfx::Erase);
            } else if cargo.contains(x, y) {
                let r = self.layout.routes[ri];
                let opts = sim::cargo_options(self.level(), &self.layout, r.from, r.to);
                if opts.len() > 1 {
                    let k = opts.iter().position(|&g| g == r.cargo).map(|k| (k + 1) % opts.len()).unwrap_or(0);
                    self.begin_edit();
                    self.layout.routes[ri].cargo = opts[k];
                    self.rebuild();
                    self.sfx(Sfx::Click);
                } else {
                    self.say("This route can only carry the one thing.", true);
                }
            } else {
                continue;
            }
            return true;
        }
        true
    }

    fn update_preview(&mut self) {
        self.preview = None;
        let (Tool::Species(s), Some(f)) = (self.tool, self.from) else { return };
        let Some(n) = self.hovered_node() else { return };
        if n == f {
            return;
        }
        let opts = sim::cargo_options(self.level(), &self.layout, f, n);
        let Some(g) = self.default_cargo(f, &opts) else { return };
        let r = crate::layout::Route { species: s, from: f, to: n, cargo: g, count: 1 };
        let plan = sim::plan_route(self.level(), &self.layout, &self.world, &r);
        self.preview = Some((n, g, plan));
    }

    fn set_tool(&mut self, t: Tool) {
        self.tool = t;
        self.from = None;
        self.preview = None;
        self.sfx(Sfx::Click);
    }

    fn play_press(&mut self, x: i32, y: i32, right: bool) {
        self.tip = None;
        // Overlays first.
        match self.overlay.clone() {
            Overlay::Rule => {
                self.overlay = Overlay::None;
                self.sfx(Sfx::Page);
                return;
            }
            Overlay::Results { .. } | Overlay::Failed(_) => {
                let b = result_buttons();
                let solved = matches!(self.overlay, Overlay::Results { .. });
                if b[0].contains(x, y) {
                    if solved && self.cur + 1 < self.levels.len() {
                        self.leave_level();
                        self.enter_level(self.cur + 1);
                    } else {
                        self.overlay = Overlay::None;
                        self.stop_sim();
                    }
                    self.sfx(Sfx::Click);
                } else if b[1].contains(x, y) {
                    self.overlay = Overlay::None;
                    self.sfx(Sfx::Click);
                    if solved && self.level().id == "1-1" {
                        let at = self.node_px(NodeRef::Source(0));
                        self.tip("more_crew", "Faster? Click the twig pile, then + to hire more tribbles.", (at.0, at.1 - 6));
                    }
                } else if b[2].contains(x, y) {
                    self.overlay = Overlay::None;
                    self.leave_level();
                    self.sfx(Sfx::Click);
                }
                return;
            }
            Overlay::None => {}
        }
        if R_HELP.contains(x, y) {
            self.open_help(Screen::Play);
            return;
        }
        if R_MENU.contains(x, y) {
            self.leave_level();
            self.sfx(Sfx::Click);
            return;
        }
        if R_ROUTES.contains(x, y) {
            self.show_routes = !self.show_routes;
            self.sfx(Sfx::Click);
            return;
        }
        if y >= VIEW_H {
            for (i, t) in self.tools().into_iter().enumerate() {
                if tool_rect(i).contains(x, y) {
                    self.set_tool(t);
                    return;
                }
            }
            if R_RUN.contains(x, y) {
                self.toggle_run();
            } else if R_STEP.contains(x, y) {
                self.step_once();
            } else if R_SPEED.contains(x, y) {
                self.speed = (self.speed + 1) % SPEEDS.len();
                self.sfx(Sfx::Click);
            } else if R_RESET.contains(x, y) {
                self.reset();
            }
            return;
        }
        if let Some(card) = self.card_rect() {
            if !right && self.card_click(card, x, y) {
                return;
            }
        }
        let (tx, ty) = (x / TILE, y / TILE);
        if right {
            if self.from.is_some() {
                self.from = None;
                self.preview = None;
            } else if self.erasable_at(tx, ty) {
                self.erase_at(tx, ty);
            } else if self.tool != Tool::Select {
                self.tool = Tool::Select;
            }
            return;
        }
        self.click_map(tx, ty);
    }

    fn open_help(&mut self, ret: Screen) {
        self.help_return = ret;
        self.screen = Screen::Help;
        self.running = false;
        self.sfx(Sfx::Page);
    }

    fn play_key(&mut self, k: Key) {
        if !matches!(self.overlay, Overlay::None) {
            match (k, &self.overlay) {
                (Key::Esc | Key::Enter | Key::Space, Overlay::Rule) => self.overlay = Overlay::None,
                (Key::Enter, Overlay::Results { .. }) => {
                    if self.cur + 1 < self.levels.len() {
                        self.leave_level();
                        self.enter_level(self.cur + 1);
                    }
                }
                (Key::Esc | Key::Space, _) => self.overlay = Overlay::None,
                (Key::Help, _) => self.open_help(Screen::Play),
                _ => {}
            }
            return;
        }
        match k {
            Key::Space => self.toggle_run(),
            Key::Tab | Key::Letter('F') => {
                self.speed = (self.speed + 1) % SPEEDS.len();
                self.sfx(Sfx::Click);
            }
            Key::Letter('S') => self.step_once(),
            Key::Letter('R') => self.reset(),
            Key::Letter('Z') | Key::Back => self.undo(),
            Key::Letter('V') => self.show_routes = !self.show_routes,
            Key::Letter('I') => self.overlay = Overlay::Rule,
            Key::Letter('X') | Key::Num(0) => self.set_tool(Tool::Erase),
            Key::Help => self.open_help(Screen::Play),
            Key::Num(n) => {
                let tools = self.tools();
                if let Some(&t) = tools.get(n as usize - 1) {
                    self.set_tool(t);
                }
            }
            Key::Esc => {
                if self.from.is_some() {
                    self.from = None;
                    self.preview = None;
                } else if self.tool != Tool::Select {
                    self.tool = Tool::Select;
                } else if self.selected.is_some() {
                    self.selected = None;
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

    /// The first level not yet solved (where "continue" goes).
    pub fn next_unsolved(&self) -> usize {
        (0..self.levels.len()).find(|&i| !self.save.rec(&self.levels[i].id).solved).unwrap_or(0)
    }
}

impl App for Game {
    fn event(&mut self, ev: Ev) {
        match ev {
            Ev::Move(x, y) => {
                self.mouse = (x, y);
                if self.screen == Screen::Play {
                    self.update_preview();
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
                    Key::Num(n) if (n as usize) <= HELP_PAGES => self.help_page = n as usize - 1,
                    _ => {}
                },
                Screen::Play => self.play_key(k),
            },
        }
    }

    fn tick(&mut self) {
        self.frame = self.frame.wrapping_add(1);
        palette::cycle_water(&mut self.pal, self.frame);
        if self.screen == Screen::Play && self.running {
            self.advance(SPEEDS[self.speed]);
        }
        if self.tip.as_ref().is_some_and(|t| self.frame > t.until) {
            self.tip = None;
        }
        for s in self.sparks.iter_mut() {
            s.1 -= 1;
            s.2 -= 1;
        }
        self.sparks.retain(|s| s.2 > 0);
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
        if self.screen == Screen::Play && self.mouse.1 < VIEW_H && matches!(self.overlay, Overlay::None) && self.tool == Tool::Select && self.hovered_node().is_some() {
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
            let id = self.level().id.clone();
            self.save.rec_mut(&id).layout = self.layout.to_text();
        }
        self.save.write();
        self.log.line("=== session end");
    }
}

/// "a twig", "an egg", "some clay".
pub fn a_good(g: Good) -> String {
    match g {
        Good::Clay | Good::Wicker => format!("some {}", g.word()),
        Good::Egg => "an egg".into(),
        _ => format!("a {}", g.word()),
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

/// A speech bubble pointing at a spot on screen.
#[derive(Clone, Debug)]
pub struct Bubble {
    pub text: String,
    pub x: i32,
    pub y: i32,
    pub until: u32,
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

    fn tile(g: &mut Game, tx: i32, ty: i32) {
        click(g, tx * TILE + 5, ty * TILE + 5);
    }

    /// Plays the first site with the mouse alone: route, run, results, next site.
    #[test]
    fn first_site_by_mouse() {
        let mut g = game();
        click(&mut g, 200, 150);
        assert_eq!(g.screen, Screen::Select);
        let r = select_rect(0, 0);
        click(&mut g, r.x + 5, r.y + 5);
        assert_eq!(g.screen, Screen::Play);
        assert!(matches!(g.overlay, Overlay::Rule));
        click(&mut g, 200, 100);
        assert!(matches!(g.overlay, Overlay::None));
        let t = tool_rect(1);
        click(&mut g, t.x + 5, t.y + 5);
        assert_eq!(g.tool, Tool::Species(Species::Tribble));
        tile(&mut g, 4, 16);
        assert_eq!(g.from, Some(NodeRef::Source(0)));
        tile(&mut g, 34, 16);
        assert_eq!(g.layout.routes.len(), 1);
        // Add two more tribbles from the card.
        let card = g.card_rect().expect("card");
        let (_, _, _, plus, _) = g.card_row(card, 0);
        click(&mut g, plus.x + 2, plus.y + 2);
        click(&mut g, plus.x + 2, plus.y + 2);
        assert_eq!(g.layout.routes[0].count, 3);
        click(&mut g, R_RUN.x + 5, R_RUN.y + 5);
        assert!(g.running);
        g.speed = SPEEDS.len() - 1;
        for _ in 0..200 {
            g.tick();
        }
        match g.overlay {
            Overlay::Results { ticks, cost, crew, .. } => assert_eq!((ticks, cost, crew), (114, 6, 3)),
            ref o => panic!("expected results, got {o:?}"),
        }
        let b = result_buttons();
        click(&mut g, b[0].x + 5, b[0].y + 5);
        assert_eq!(g.cur, 1);
        assert!(matches!(g.overlay, Overlay::Rule));
    }

    /// Building, erasing with the right button, and undo.
    #[test]
    fn build_erase_undo() {
        let mut g = game();
        let i = g.levels.iter().position(|l| l.id == "1-4").unwrap();
        g.enter_level(i);
        g.overlay = Overlay::None;
        let tools = g.tools();
        let k = tools.iter().position(|t| *t == Tool::Build(Kind::Depot)).unwrap();
        let t = tool_rect(k);
        click(&mut g, t.x + 5, t.y + 5);
        tile(&mut g, 8, 15);
        assert_eq!(g.layout.buildings.len(), 1);
        g.event(Ev::Press { x: 85, y: 155, right: true });
        g.event(Ev::Press { x: 85, y: 155, right: true });
        g.event(Ev::Press { x: 85, y: 155, right: true });
        assert_eq!(g.layout.buildings.len(), 0, "right-click erases once the tool is put down");
        g.event(Ev::Key(Key::Letter('Z')));
        assert_eq!(g.layout.buildings.len(), 1);
    }
}
