//! Monsters, input, pushing and record/replay, advanced in discrete 15 FPS ticks. Crafting,
//! harvesting and bins live in `work.rs`.
//!
//! A move updates the logical tile at once and then plays out as a short slide, so input is
//! answered on the same tick and collisions are settled before anything is drawn.

use crate::data::{Data, Stats};
use crate::econ::{ItemData, Learned, Rng, Share, Skills};
use crate::palette::{SPARK, TEXT_GOOD};
use serde::{Deserialize, Serialize};
use crate::tutorial::{self, Tutorial};
use crate::world::*;
#[cfg_attr(test, allow(unused_imports))]
pub use crate::world::Dir;
use std::collections::BTreeSet;

pub const WALK_TICKS: u8 = 2;
pub const PUSH_TICKS: u8 = 3;
/// Pulling is a way out of a stuck spot, not a way to haul: two seconds a tile.
pub const PULL_TICKS: u8 = 30;
const BUBBLE_TICKS: u16 = 45;
const WAIT_BEFORE_SAYING: u16 = 12;
/// A held direction starts walking on its own only after ~267 ms, so a tap is one tile.
const HOLD_REPEAT_TICKS: u8 = 4;
/// The ferry waits two seconds at each bank, then takes 6 ticks per tile.
const FERRY_DOCK_TICKS: u32 = 30;
const FERRY_STEP_TICKS: u8 = 6;
/// It waits up to ten more seconds for someone standing on the landing.
const FERRY_PATIENCE_TICKS: u32 = 150;
const BANNER_TICKS: u16 = 45;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Sfx {
    Step,
    Push,
    Bump,
    Pop,
    Plop,
    Pad,
    Spin,
    Chime,
    Deposit,
    RecOn,
    RecSave,
    LoopOn,
    LoopOff,
    Switch,
    Deny,
}

impl Sfx {
    #[cfg(test)]
    pub const ALL: [Sfx; 15] = [
        Sfx::Step,
        Sfx::Push,
        Sfx::Bump,
        Sfx::Pop,
        Sfx::Plop,
        Sfx::Pad,
        Sfx::Spin,
        Sfx::Chime,
        Sfx::Deposit,
        Sfx::RecOn,
        Sfx::RecSave,
        Sfx::LoopOn,
        Sfx::LoopOff,
        Sfx::Switch,
        Sfx::Deny,
    ];
}

pub enum Request {
    Quit,
}

/// One tick of latched input. Direction arrays are indexed like `DIRS`.
#[derive(Default, Clone, Copy)]
pub struct Input {
    pub held: [bool; 4],
    pub pressed: [bool; 4],
    pub use_pressed: bool,
    pub rec_pressed: bool,
    pub play_pressed: bool,
    pub tab_pressed: bool,
    /// Tab held down: show the roster strip.
    pub tab_held: bool,
    pub cancel_pressed: bool,
    pub help_pressed: bool,
    pub quit_pressed: bool,
    /// Del, on the title screen: delete the picked world.
    pub delete_pressed: bool,
    /// E held down: examine.
    pub inspect: bool,
    /// Shift held down: stepping away from an item drags it along.
    pub pull: bool,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub enum Species {
    Goblin,
    Cyclops,
}

/// One recorded action. A step remembers which good it pushed, so replay can tell "it isn't
/// here yet" (wait) from "something else is in the way" (also wait, but say so).
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub enum Act {
    Step(Dir, Option<String>),
    /// A stretch of plain walking: get to this tile however you can, then face this way.
    Go { to: (i32, i32), facing: Dir },
    /// A run of pushes (and pulls) of one good: get the good lying at `from` to `to` however
    /// you can. `to` may be a bin, board or wagon that takes it.
    Move { good: String, from: (i32, i32), to: (i32, i32) },
    /// Backing away in this direction, dragging this good from the tile in front.
    Pull(Dir, String),
    Use,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Routine {
    pub flag: (i32, i32),
    pub acts: Vec<Act>,
    /// The way the monster faced when recording began; each run of the loop starts facing it.
    #[serde(default = "face_down")]
    pub facing: Dir,
}

fn face_down() -> Dir {
    Dir::Down
}

pub struct Bubble {
    pub text: String,
    pub ttl: u16,
    pub warn: bool,
    pub waiting: bool, // a looping monster stuck on a step; drawn as a small icon when not controlled
}

#[derive(Serialize, Deserialize)]
pub struct Monster {
    pub id: u32,
    pub name: String,
    pub kind: String,
    pub species: Species,
    pub stats: Stats,
    pub skills: Skills,
    pub x: i32,
    pub y: i32,
    pub facing: Dir,
    pub flip: bool,
    pub slide: Option<Slide>,
    #[serde(skip)]
    pub bump: Option<(Dir, u8)>,
    pub busy: bool,
    pub walk_frame: u8,
    #[serde(skip)]
    pub bubble: Option<Bubble>,
    pub rec: Option<Routine>,
    pub routine: Option<Routine>,
    pub playing: bool,
    pub pc: usize,
    pub paused_at: Option<(i32, i32)>,
    pub wait: u16,
    pub wait_reason: String,
    /// The way to the current "go to" goal, and how long it has been blocked by another monster.
    #[serde(default)]
    pub path: Vec<(i32, i32)>,
    #[serde(default)]
    pub blocked: u16,
    /// Where the good it's moving is, as of its last push, and the pushes and pulls still
    /// planned for it.
    #[serde(default)]
    pub carry: Option<(i32, i32)>,
    #[serde(default)]
    pub carry_plan: Vec<crate::carry::CarryStep>,
    /// A stray it's shoving out of its way first (by id), and where to.
    #[serde(default)]
    pub shove: Option<(u32, (i32, i32))>,
    /// Walking itself back to its flag to finish a recording (R away from the flag).
    #[serde(default)]
    pub homing: bool,
    /// The step of its loop it's doing (by the index of its act), once it has picked one.
    #[serde(default)]
    pub task: Option<usize>,
    /// What that step has reserved: the good it's going for, the tiles it's about to use.
    #[serde(default)]
    pub holds: Vec<crate::sched::Hold>,
    /// Steps it gave up recently, and ticks before it tries each again.
    #[serde(default)]
    pub cooldown: Vec<(usize, u16)>,
    /// What it's waiting its turn for (a plant or work spot, or where a good is taken from),
    /// and since which tick.
    #[serde(default)]
    pub turns: Vec<((i32, i32), u32)>,
    /// The monster it's waiting on this tick, if any.
    #[serde(skip)]
    pub waiting_on: Option<usize>,
    pub loops: u32,
    /// Steps of its loop finished, for telling a loop that's working from one that's stuck.
    #[serde(default)]
    pub steps: u32,
    /// Renown credited to this monster from scored goods.
    pub earned: f64,
    /// What the last finished job taught, for skill popups.
    #[serde(skip)]
    pub learned: Option<(String, Learned, u16)>,
}

/// Someone waiting in the hiring hall: a monster type with three talents already rolled.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Applicant {
    pub kind: String,
    pub name: String,
    pub transferable: Vec<String>,
    pub cost: u32,
    pub stats: Stats,
}

/// Floating text that rises or flies between two points (pixels in world space).
pub struct Floater {
    pub text: String,
    pub from: (i32, i32),
    pub to: (i32, i32),
    pub t: u16,
    pub total: u16,
    /// Ticks before it appears (credits wait for the renown to rise first).
    pub delay: u16,
    pub color: u8,
}

const NAMES: [&str; 16] = ["GRUB", "NELL", "TOK", "ISKA", "BRAM", "ODA", "PIP", "WREN", "MOSK", "HILDA", "ZUG", "FERN", "KORR", "LUMA", "DREG", "SIV"];
/// Ticks before an empty stand gets a new applicant, and between roster refreshes.
const APPLICANT_ARRIVES: u32 = 300;
/// Renown a scored good pays per point of its value (values come from the tech tree's raw
/// materials and are small). A first tuning guess: early cotton fabric pays about 20.
pub const RENOWN_PER_VALUE: f64 = 10.0;
const ROSTER_REFRESH: u32 = 4500;

impl Monster {
    fn new(id: u32, name: &str, kind: &str, data: &Data, (x, y): (i32, i32), skills: Skills) -> Self {
        let species = if kind == "goblin" { Species::Goblin } else { Species::Cyclops };
        Self {
            id,
            name: name.to_string(),
            kind: kind.to_string(),
            species,
            stats: data.monster_types[kind].stats.clone(),
            skills,
            x,
            y,
            facing: Dir::Right,
            flip: false,
            slide: None,
            bump: None,
            busy: false,
            walk_frame: 0,
            bubble: None,
            rec: None,
            routine: None,
            playing: false,
            pc: 0,
            paused_at: None,
            wait: 0,
            wait_reason: String::new(),
            path: Vec::new(),
            blocked: 0,
            carry: None,
            carry_plan: Vec::new(),
            shove: None,
            homing: false,
            task: None,
            holds: Vec::new(),
            cooldown: Vec::new(),
            turns: Vec::new(),
            waiting_on: None,
            loops: 0,
            steps: 0,
            earned: 0.0,
            learned: None,
        }
    }

    pub fn idle(&self) -> bool {
        self.slide.is_none() && !self.busy
    }

    /// Round the end of its loop and back to the start, facing the way it started.
    pub(crate) fn lap(&mut self) {
        self.loops += 1;
        if let Some(f) = self.routine.as_ref().map(|r| r.facing) {
            self.facing = f;
        }
    }

}

/// Adds a recorded act, keeping recordings about goals: walking merges into one "go to", and
/// moving the same good on from where its last move left it extends that move. Walks up to a
/// move, or in between to get behind the good again, are the planner's business on replay:
/// recorded, they'd send a loop to stand on tiles that may well hold the next good by then.
fn record(r: &mut Routine, act: Act) {
    match act {
        Act::Go { to, facing } => match r.acts.last_mut() {
            Some(Act::Go { to: last, facing: last_facing }) => (*last, *last_facing) = (to, facing),
            _ => r.acts.push(Act::Go { to, facing }),
        },
        Act::Move { good, from, to } => {
            let k = r.acts.iter().rposition(|a| !matches!(a, Act::Go { .. }));
            if let Some(k) = k {
                if let Act::Move { good: g, from: first, to: last_to } = &mut r.acts[k] {
                    if *g == good && *last_to == from {
                        *last_to = to;
                        // Put back where it started (a slip, corrected): nothing to record.
                        let undone = *first == to;
                        r.acts.truncate(if undone { k } else { k + 1 });
                        return;
                    }
                }
            }
            if matches!(r.acts.last(), Some(Act::Go { .. })) {
                r.acts.pop();
            }
            r.acts.push(Act::Move { good, from, to });
        }
        act => r.acts.push(act),
    }
}

fn starting_skills(transferable: &[&str], applied: &[(&str, f64)]) -> Skills {
    Skills {
        transferable: transferable.iter().map(|s| s.to_string()).collect(),
        applied: applied.iter().map(|(k, v)| (k.to_string(), *v)).collect(),
        ..Skills::default()
    }
}

pub(crate) enum StepRes {
    Moved(Option<String>),
    Blocked(String),
}

pub(crate) enum UseRes {
    Done,
    Wait(String),
    /// This workshop (its label) has no room for what it would make.
    Full(String),
    /// What it would make from isn't on its pads (what to put there).
    Missing(String),
    Nothing,
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub(crate) enum Ev {
    Walked,
    Made(String),
    Delivered(String),
    Switched,
    /// A wagon set a good out.
    Unloaded,
}

pub struct Game {
    pub data: Data,
    pub world: World,
    pub monsters: Vec<Monster>,
    pub controlled: usize,
    pub frame: u32,
    pub help: bool,
    pub tutorial: Tutorial,
    pub rng: Rng,
    /// Goods made at least once (tutorial progress).
    pub made: BTreeSet<String>,
    queued: Option<Dir>,
    queued_use: bool,
    hold: Option<(Dir, u8)>, // direction held since its press, and for how many ticks
    pub(crate) events: Vec<Ev>,
    /// Holding E: show the inspect card for whatever the controlled monster faces.
    pub inspecting: bool,
    /// Holding Tab: the roster strip shows every monster and what it is doing.
    pub roster: bool,
    /// Which page of the inspect card shows; Tab turns it while E is held.
    pub card_page: usize,
    pub(crate) examine_hinted: bool,
    pub renown: f64,
    /// The named area the view is in, and a banner announcing it on arrival.
    area: &'static str,
    pub banner: Option<(&'static str, u16)>,
    /// One slot per stand in the gathering hall; empty while a new applicant is on the way.
    pub applicants: Vec<Option<Applicant>>,
    pub(crate) applicant_timers: Vec<u32>,
    pub(crate) roster_timer: u32,
    pub(crate) next_id: u32,
    pub floaters: Vec<Floater>,
}

impl Game {
    pub fn new() -> Self {
        let data = Data::load();
        let world = World::new(&data);
        let mogg = starting_skills(&["handcrafts", "science", "engineering"], &[("spinning", 0.3), ("textiles", 0.3), ("weaving", 0.2)]);
        let bruna = starting_skills(&["outdoorsmonstership", "athletics", "social"], &[("harvesting", 0.3), ("gathering", 0.3)]);
        let mut monsters = vec![
            Monster::new(1, "MOGG", "goblin", &data, world.starts[0], mogg),
            Monster::new(2, "BRUNA", "cyclops", &data, world.starts[1], bruna),
        ];
        let stands = world.stands.len();
        monsters[0].bubble = Some(Bubble { text: "HI, I'M MOGG! LET'S MAKE SOME CLOTH.".into(), ttl: 60, warn: false, waiting: false });
        let mut g = Self {
            data,
            world,
            monsters,
            controlled: 0,
            frame: 0,
            help: false,
            tutorial: Tutorial::default(),
            rng: Rng::new(0x5EED_2026),
            made: BTreeSet::new(),
            queued: None,
            queued_use: false,
            hold: None,
            events: Vec::new(),
            inspecting: false,
            roster: false,
            card_page: 0,
            examine_hinted: false,
            renown: 0.0,
            area: area_name(0),
            banner: None,
            applicants: Vec::new(),
            applicant_timers: vec![0; stands],
            roster_timer: 0,
            next_id: 3,
            floaters: Vec::new(),
        };
        g.applicants = (0..stands).map(|_| Some(g.roll_applicant())).collect();
        g
    }

    /// A new applicant: a type with three distinct talents, rolled from the world's dice.
    fn roll_applicant(&mut self) -> Applicant {
        const KINDS: [&str; 2] = ["goblin", "cyclops"]; // the types with sprites so far
        let kind = KINDS[(self.rng.next_u64() % KINDS.len() as u64) as usize];
        let mut pool = self.data.skills.transferable_skills.clone();
        let mut transferable = Vec::new();
        for _ in 0..3.min(pool.len()) {
            let k = (self.rng.next_u64() % pool.len() as u64) as usize;
            transferable.push(pool.remove(k));
        }
        let name = NAMES[(self.rng.next_u64() % NAMES.len() as u64) as usize].to_string();
        let t = &self.data.monster_types[kind];
        Applicant { kind: kind.to_string(), name, transferable, cost: t.cost, stats: t.stats.clone() }
    }

    /// Is an applicant standing on this tile? They block the way like anyone else.
    pub fn applicant_at(&self, x: i32, y: i32) -> bool {
        self.world.stands.iter().zip(&self.applicants).any(|(&s, a)| s == (x, y) && a.is_some())
    }

    /// The applicant stand the controlled monster faces, if someone waits there.
    pub fn faced_applicant(&self) -> Option<usize> {
        let m = &self.monsters[self.controlled];
        let (dx, dy) = m.facing.delta();
        let k = self.world.stands.iter().position(|&s| s == (m.x + dx, m.y + dy))?;
        self.applicants[k].as_ref().map(|_| k)
    }

    /// An applicant joins the commune; its renown makes room for them.
    fn welcome(&mut self, c: usize, k: usize, sfx: &mut Vec<Sfx>) {
        if self.monsters[c].rec.is_some() {
            self.say(c, "FINISH RECORDING FIRST (R).", true);
            sfx.push(Sfx::Deny);
            return;
        }
        let Some(a) = self.applicants[k].clone() else { return };
        if self.renown < a.cost as f64 {
            self.say(c, &format!("THEY JOIN AT {} RENOWN.", a.cost), true);
            sfx.push(Sfx::Deny);
            return;
        }
        self.renown -= a.cost as f64;
        self.applicants[k] = None;
        self.applicant_timers[k] = APPLICANT_ARRIVES;
        let skills = Skills { transferable: a.transferable.clone(), ..Skills::default() };
        let mut m = Monster::new(self.next_id, &a.name, &a.kind, &self.data, self.world.stands[k], skills);
        self.next_id += 1;
        m.facing = Dir::Up;
        m.bubble = Some(Bubble { text: format!("I'M {}! HAPPY TO JOIN.", a.name), ttl: BUBBLE_TICKS, warn: false, waiting: false });
        self.monsters.push(m);
        sfx.push(Sfx::RecSave);
    }

    /// A scored good pays renown, and each contributor's share flies to it as credit.
    pub(crate) fn score(&mut self, value: f64, shares: &[Share], bx: i32, by: i32) {
        let value = value * RENOWN_PER_VALUE;
        self.renown += value;
        let at = (bx * TS + 8, by * TS);
        self.floaters.push(Floater { text: format!("+{} RENOWN", value.round()), from: at, to: (at.0, at.1 - 26), t: 0, total: 40, delay: 0, color: SPARK + 2 });
        let total: f64 = shares.iter().map(|s| s.count).sum();
        if total <= 0.0 {
            return;
        }
        for s in shares {
            let credit = value * s.count / total;
            if let Some(m) = self.monsters.iter_mut().find(|m| m.id == s.monster) {
                m.earned += credit;
                if credit >= 0.5 {
                    let to = (m.x * TS + 8, m.y * TS - 6);
                    self.floaters.push(Floater { text: format!("{} +{}", m.name, credit.round()), from: (at.0, at.1 - 20), to, t: 0, total: 44, delay: 14, color: TEXT_GOOD });
                }
            }
        }
    }

    /// A good pushed into a project board: renown as for a delivery, and the project moves on.
    fn contribute(&mut self, b: usize, data: ItemData, sfx: &mut Vec<Sfx>) {
        let board = &mut self.world.boards[b];
        board.have += 1;
        board.bounce = 4;
        let (bx, by, done) = (board.x, board.y, board.done());
        self.world.burst(bx, by, PKind::Spark, 5);
        self.score(data.value, &data.shares, bx, by);
        if done {
            self.finish_project(b, sfx);
        }
    }

    fn finish_project(&mut self, b: usize, sfx: &mut Vec<Sfx>) {
        match self.world.boards[b].project {
            Project::Bridge => {
                self.world.build_bridge();
                for (x, y) in self.world.bridge.clone() {
                    self.world.burst(x, y, PKind::Spark, 2);
                }
                let &(x, y) = self.world.bridge.first().expect("a bridge site");
                let at = ((x + 3) * TS, y * TS);
                self.floaters.push(Floater { text: "THE BRIDGE IS BUILT!".into(), from: at, to: (at.0, at.1 - 30), t: 0, total: 60, delay: 20, color: SPARK + 2 });
            }
            Project::Ferry => {
                let Some(f) = self.world.ferry.as_mut() else { return };
                f.rigged = true;
                f.timer = 0;
                let (y, x0, x1) = (f.y, f.west, f.east + 1);
                for x in (x0..=x1).step_by(2) {
                    self.world.burst(x, y, PKind::Spark, 4);
                }
                let at = (((x0 + x1) / 2) * TS + 8, y * TS);
                self.floaters.push(Floater { text: "THE FERRY RUNS!".into(), from: at, to: (at.0, at.1 - 30), t: 0, total: 60, delay: 20, color: SPARK + 2 });
            }
        }
        sfx.push(Sfx::Chime);
    }

    /// The ferry waits at a bank, then pulls itself across with everything on its deck.
    fn tick_ferry(&mut self) {
        let Some(mut f) = self.world.ferry.clone() else { return };
        if !f.rigged {
            return;
        }
        if let Some(s) = f.slide.as_mut() {
            s.left -= 1;
            if s.left == 0 {
                f.slide = None;
            }
        }
        f.timer += 1;
        if f.moving {
            if f.timer >= FERRY_STEP_TICKS as u32 {
                f.timer = 0;
                let dir = if f.heading > 0 { Dir::Right } else { Dir::Left };
                let slide = Some(Slide { dir, left: FERRY_STEP_TICKS, total: FERRY_STEP_TICKS });
                for m in self.monsters.iter_mut().filter(|m| f.on_deck(m.x, m.y)) {
                    m.x += f.heading;
                    m.slide = slide;
                }
                for it in self.world.items.iter_mut().filter(|it| f.on_deck(it.x, it.y)) {
                    it.x += f.heading;
                    it.slide = slide;
                }
                f.x += f.heading;
                f.slide = slide;
                if f.x == f.west || f.x == f.east {
                    f.moving = false;
                    f.heading = -f.heading;
                }
            }
        } else {
            let landing = f.landing();
            let stepping = self.monsters.iter().any(|m| f.on_deck(m.x, m.y) && (m.slide.is_some() || m.busy)) || self.world.items.iter().any(|it| f.on_deck(it.x, it.y) && it.slide.is_some());
            let waiting = self.monsters.iter().any(|m| Some((m.x, m.y)) == landing) && f.timer < FERRY_DOCK_TICKS + FERRY_PATIENCE_TICKS;
            if f.timer >= FERRY_DOCK_TICKS && !stepping && !waiting {
                f.moving = true;
                f.timer = 0;
            }
        }
        self.world.ferry = Some(f);
    }

    /// Why a step into water fails: at the ferry, what the ferry is up to.
    fn water_note(&self, x: i32, y: i32) -> String {
        match &self.world.ferry {
            Some(f) if f.crossing(x, y) && !f.rigged => "THE FERRY NEEDS A ROPE.".into(),
            Some(f) if f.crossing(x, y) => "WAITING FOR THE FERRY.".into(),
            _ => String::new(),
        }
    }

    /// Applicants arrive at empty stands, and now and then one leaves for someone new.
    fn tick_hall(&mut self) {
        for k in 0..self.applicants.len() {
            let (sx, sy) = self.world.stands[k];
            let clear = self.monster_at(sx, sy).is_none() && self.world.item_at(sx, sy).is_none();
            if self.applicants[k].is_none() {
                self.applicant_timers[k] = self.applicant_timers[k].saturating_sub(1);
                if self.applicant_timers[k] == 0 && clear {
                    self.applicants[k] = Some(self.roll_applicant());
                }
            }
        }
        self.roster_timer += 1;
        if self.roster_timer >= ROSTER_REFRESH && !self.applicants.is_empty() {
            self.roster_timer = 0;
            let k = (self.frame / ROSTER_REFRESH) as usize % self.applicants.len();
            self.applicants[k] = Some(self.roll_applicant());
        }
    }

    pub fn update(&mut self, inp: &Input, req: &mut Vec<Request>, sfx: &mut Vec<Sfx>) {
        self.frame = self.frame.wrapping_add(1);
        if inp.quit_pressed {
            req.push(Request::Quit);
            return;
        }
        if inp.help_pressed {
            self.help = !self.help;
        }
        if inp.inspect && !self.inspecting {
            self.card_page = 0;
        }
        self.inspecting = inp.inspect;
        self.roster = inp.tab_held && !inp.inspect;
        if inp.tab_pressed && self.inspecting {
            // While examining, Tab turns the card's page instead of switching monsters.
            self.card_page += 1;
        } else if inp.tab_pressed {
            self.controlled = (self.controlled + 1) % self.monsters.len();
            self.queued = None;
            self.queued_use = false;
            self.events.push(Ev::Switched);
            sfx.push(Sfx::Switch);
        }
        self.player_input(inp, sfx);
        for i in 0..self.monsters.len() {
            if self.monsters[i].playing {
                self.run_loop(i, sfx);
            } else if self.monsters[i].homing && self.monsters[i].idle() {
                self.walk_home(i, sfx);
            }
        }
        self.give_way();
        self.advance_motion();
        self.tick_ferry();
        self.tick_wagons(sfx);
        self.tick_work(sfx);
        self.tick_hall();
        self.world.tick_particles(self.frame);
        for f in self.floaters.iter_mut() {
            f.t += 1;
        }
        self.floaters.retain(|f| f.t < f.total);
        for m in self.monsters.iter_mut() {
            if let Some(b) = m.bubble.as_mut() {
                b.ttl = b.ttl.saturating_sub(1);
            }
            if m.bubble.as_ref().is_some_and(|b| b.ttl == 0) {
                m.bubble = None;
            }
            if let Some((_, _, t)) = m.learned.as_mut() {
                *t = t.saturating_sub(1);
            }
            if m.learned.as_ref().is_some_and(|l| l.2 == 0) {
                m.learned = None;
            }
        }
        let c = &self.monsters[self.controlled];
        let area = area_name(crate::render::screen_of(c.x, c.y).0);
        if area != self.area {
            self.area = area;
            self.banner = Some((area, BANNER_TICKS));
        } else if let Some((_, t)) = self.banner.as_mut() {
            *t = t.saturating_sub(1);
            if *t == 0 {
                self.banner = None;
            }
        }
        self.advance_tutorial();
    }

    pub(crate) fn say(&mut self, i: usize, text: &str, warn: bool) {
        self.monsters[i].bubble = Some(Bubble { text: text.to_string(), ttl: BUBBLE_TICKS, warn, waiting: false });
    }

    pub fn monster_at(&self, x: i32, y: i32) -> Option<usize> {
        self.monsters.iter().position(|m| m.x == x && m.y == y)
    }

    pub fn good_name(&self, id: &str) -> String {
        self.data.try_good(id).map_or_else(|| id.to_uppercase(), |g| g.name.to_uppercase())
    }

    fn player_input(&mut self, inp: &Input, sfx: &mut Vec<Sfx>) {
        let c = self.controlled;
        let held = |d: Dir| inp.held[DIRS.iter().position(|&x| x == d).unwrap_or(0)];
        let mut fresh = None;
        for (k, d) in DIRS.iter().enumerate() {
            if inp.pressed[k] {
                fresh = Some(*d);
            }
        }
        self.hold = match (fresh, self.hold) {
            (Some(d), _) => Some((d, 0)),
            (None, Some((d, t))) if held(d) => Some((d, t.saturating_add(1))),
            // Released, but another direction is still down: keep walking that way.
            (None, Some((_, t))) => DIRS.iter().copied().find(|&d| held(d)).map(|d| (d, t)),
            (None, None) => None,
        };
        if inp.rec_pressed {
            self.toggle_record(c, sfx);
        }
        if inp.cancel_pressed && self.monsters[c].rec.take().is_some() {
            self.monsters[c].homing = false;
            self.say(c, "RECORDING CANCELLED.", false);
            sfx.push(Sfx::Deny);
        }
        if inp.play_pressed {
            self.toggle_play(c, sfx);
        }
        // Steering a monster that's walking itself home takes over: walk it back yourself.
        if self.monsters[c].homing && (fresh.is_some() || inp.use_pressed) {
            self.monsters[c].homing = false;
        }

        let m = &self.monsters[c];
        if m.playing {
            if fresh.is_some() || inp.use_pressed {
                self.say(c, "I'M ON A LOOP. P STOPS ME.", true);
            }
            return;
        }
        if m.busy {
            if fresh.is_some() || inp.use_pressed {
                self.say(c, "WORKING! TAB SWITCHES MONSTER.", false);
            }
            return;
        }
        if m.slide.is_some() {
            if fresh.is_some() {
                self.queued = fresh;
            }
            self.queued_use |= inp.use_pressed;
            return;
        }

        if inp.use_pressed {
            self.face_something_usable(c);
            if let Some(k) = self.faced_applicant() {
                self.welcome(c, k, sfx);
                return;
            }
        }
        if inp.use_pressed || std::mem::take(&mut self.queued_use) {
            self.queued = None;
            let leading = self.led_wagon(c).is_some();
            match self.try_use(c, sfx) {
                UseRes::Done => self.record_use(c, leading),
                UseRes::Wait(msg) | UseRes::Missing(msg) => {
                    // Space on a plant is always part of a routine, ripe or not: on replay the
                    // monster waits there until it can harvest.
                    let m = &self.monsters[c];
                    if self.world.plant_at(m.x, m.y).is_some() {
                        self.record_use(c, leading);
                    }
                    self.say(c, &msg, true);
                    sfx.push(Sfx::Deny);
                }
                // Still part of a routine: on replay there's usually room by then, and if not
                // the loop skips it.
                UseRes::Full(msg) => {
                    self.record_use(c, leading);
                    self.say(c, &msg, true);
                    sfx.push(Sfx::Deny);
                }
                UseRes::Nothing => self.say(c, "NOTHING TO USE HERE.", false),
            }
            return;
        }

        let pressed_now = self.queued.take().or(fresh);
        let repeat = self.hold.filter(|&(_, t)| t >= HOLD_REPEAT_TICKS).map(|(d, _)| d);
        let Some(d) = pressed_now.or(repeat) else { return };
        // Shift with an item in front and a step away from it: pull. Otherwise Shift is ignored.
        let pulling = inp.pull && self.pulled_item(c, d).is_some() && self.led_wagon(c).is_none();
        let res = if pulling { self.try_pull(c, d, sfx) } else { self.try_step(c, d, sfx) };
        match res {
            StepRes::Moved(moved) => {
                let (dx, dy) = d.delta();
                let to = (self.monsters[c].x, self.monsters[c].y);
                let ferry = self.world.ferry.as_ref().is_some_and(|f| f.crossing(to.0, to.1) || f.crossing(to.0 - dx, to.1 - dy));
                let leading = self.led_wagon(c).is_some();
                // Where the good went: pushed on ahead, or pulled along into the tile left.
                let (good_from, good_to) = if pulling { ((to.0 - 2 * dx, to.1 - 2 * dy), (to.0 - dx, to.1 - dy)) } else { (to, (to.0 + dx, to.1 + dy)) };
                let ferry_good = self.world.ferry.as_ref().is_some_and(|f| f.crossing(good_from.0, good_from.1) || f.crossing(good_to.0, good_to.1));
                let from = (to.0 - dx, to.1 - dy);
                let from_deck = self.world.on_deck(from.0, from.1);
                if let Some(r) = self.monsters[c].rec.as_mut() {
                    let act = match (pulling, moved) {
                        // Moving a good: record where it went, not how. Leading a wagon or
                        // using the ferry, the route itself matters, so those stay as recorded.
                        (_, Some(good)) if !leading && !ferry && !ferry_good => Act::Move { good, from: good_from, to: good_to },
                        (true, Some(good)) => Act::Pull(d, good),
                        (_, None) if !ferry && !leading => Act::Go { to, facing: d },
                        (_, moved) => Act::Step(d, moved),
                    };
                    // A step replayed as recorded starts where this one did; a planned move just
                    // before it may end somewhere else, so go there first.
                    if matches!(act, Act::Step(..) | Act::Pull(..)) && matches!(r.acts.last(), Some(Act::Move { .. })) && !from_deck {
                        r.acts.push(Act::Go { to: from, facing: if pulling { d.opposite() } else { d } });
                    }
                    record(r, act);
                }
                if pulling {
                    self.say(c, "HNNNG...", false);
                }
                self.events.push(Ev::Walked);
            }
            StepRes::Blocked(msg) => {
                // Holding into a wall bumps once; each fresh press bumps again.
                if pressed_now.is_some() {
                    self.monsters[c].bump = Some((d, 3));
                    sfx.push(Sfx::Bump);
                    let (dx, dy) = d.delta();
                    let (ix, iy) = (self.monsters[c].x + dx, self.monsters[c].y + dy);
                    if !msg.is_empty() {
                        self.say(c, &msg, true);
                    } else if !pulling && self.world.item_at(ix, iy).is_some() && self.stuck(ix, iy) {
                        self.say(c, "STUCK! HOLD SHIFT AND STEP BACK TO PULL IT. IT'S SLOW GOING.", true);
                    }
                }
            }
        }
    }

    /// Records a Space, after a "go to" right here facing this way: the act before it may leave
    /// the monster somewhere else on replay (a planned move ends wherever its last push does,
    /// not where a recorded pull did). Leading a wagon, the steps here were recorded as they were.
    fn record_use(&mut self, c: usize, leading: bool) {
        let m = &self.monsters[c];
        let (here, facing) = ((m.x, m.y), m.facing);
        let anchor = !leading && !self.world.on_deck(here.0, here.1);
        if let Some(r) = self.monsters[c].rec.as_mut() {
            if anchor {
                record(r, Act::Go { to: here, facing });
            }
            r.acts.push(Act::Use);
        }
    }

    /// The item a monster would drag by backing away in `d`: the one right in front of it.
    pub(crate) fn pulled_item(&self, i: usize, d: Dir) -> Option<usize> {
        let (dx, dy) = d.delta();
        let m = &self.monsters[i];
        self.world.item_at(m.x - dx, m.y - dy)
    }

    /// Can no push ever move this item? It needs room to stand behind it and room ahead of it,
    /// on some side.
    fn stuck(&self, x: i32, y: i32) -> bool {
        !DIRS.iter().any(|d| {
            let (dx, dy) = d.delta();
            self.world.walkable(x - dx, y - dy) && (self.world.item_floor(x + dx, y + dy) || self.world.bin_at(x + dx, y + dy).is_some())
        })
    }

    /// Back away in `d`, dragging the item in front into the tile you leave. Slow on purpose.
    pub(crate) fn try_pull(&mut self, i: usize, d: Dir, sfx: &mut Vec<Sfx>) -> StepRes {
        let (dx, dy) = d.delta();
        let (x, y) = (self.monsters[i].x, self.monsters[i].y);
        let (tx, ty) = (x + dx, y + dy);
        let Some(it) = self.pulled_item(i, d) else { return StepRes::Blocked(String::new()) };
        let m = &mut self.monsters[i];
        m.facing = d.opposite();
        if dx != 0 {
            m.flip = dx > 0;
        }
        if self.world.items[it].locked {
            return StepRes::Blocked("THAT'S BEING WORKED ON.".into());
        }
        if !self.world.walkable(tx, ty) || self.monster_at(tx, ty).is_some() || self.applicant_at(tx, ty) || self.world.item_at(tx, ty).is_some() || self.world.cart_at(tx, ty) {
            return StepRes::Blocked(self.water_note(tx, ty));
        }
        if self.led_wagon(i).is_some() {
            return StepRes::Blocked("LET GO OF THE WAGON FIRST.".into());
        }
        if !self.world.item_floor(x, y) {
            return StepRes::Blocked("I CAN'T PULL IT ONTO A WORK SPOT.".into());
        }
        let slide = Some(Slide { dir: d, left: PULL_TICKS, total: PULL_TICKS });
        self.practice(i, "hauling", PULL_TICKS);
        let id = self.monsters[i].id;
        let item = &mut self.world.items[it];
        item.data.hauled_by(id);
        let good = item.good().to_string();
        let (ix, iy) = (item.x, item.y);
        (item.x, item.y, item.hop, item.slide) = (x, y, 0, slide);
        if let Some((s, k)) = self.world.pad_owner(x, y) {
            self.world.shops[s].pad_flash[k] = 5;
            sfx.push(Sfx::Pad);
        }
        self.world.dust(ix, iy, d);
        self.world.burst(ix, iy, PKind::Dust, 3);
        sfx.push(Sfx::Push);
        let m = &mut self.monsters[i];
        (m.x, m.y, m.slide, m.bump) = (tx, ty, slide, None);
        StepRes::Moved(Some(good))
    }

    pub(crate) fn try_step(&mut self, i: usize, d: Dir, sfx: &mut Vec<Sfx>) -> StepRes {
        let (dx, dy) = d.delta();
        let (x, y) = (self.monsters[i].x, self.monsters[i].y);
        let (tx, ty) = (x + dx, y + dy);
        self.monsters[i].facing = d;
        if dx != 0 {
            self.monsters[i].flip = dx < 0;
        }
        if !self.world.walkable(tx, ty) || self.monster_at(tx, ty).is_some() || self.applicant_at(tx, ty) || self.world.cart_at(tx, ty) {
            return StepRes::Blocked(self.water_note(tx, ty));
        }
        let wagon = self.led_wagon(i);
        if wagon.is_some() && self.world.on_deck(tx, ty) {
            return StepRes::Blocked("THE WAGON WON'T FIT ON THE FERRY.".into());
        }
        // Leading a wagon, every step goes at the wagon's pace.
        let pull = wagon.map_or(0, |w| self.wagon_ticks(i, w));
        let id = self.monsters[i].id;
        let mut ticks = WALK_TICKS.max(pull);
        let mut pushed = None;
        if let Some(it) = self.world.item_at(tx, ty) {
            let good = self.world.items[it].good().to_string();
            if self.world.items[it].locked {
                return StepRes::Blocked("THAT'S BEING WORKED ON.".into());
            }
            let (bx, by) = (tx + dx, ty + dy);
            ticks = self.push_ticks(i, &self.world.items[it].data).max(pull);
            if let Some(b) = self.world.board_at(bx, by) {
                let board = &self.world.boards[b];
                if board.done() {
                    return StepRes::Blocked("THAT PROJECT IS DONE.".into());
                }
                if !self.board_takes(b, &good) {
                    return StepRes::Blocked(format!("THE BOARD WANTS {}.", self.good_name(&board.wants)));
                }
                let mut item = self.world.items.remove(it);
                item.data.hauled_by(id);
                crate::econ::pay_haul(&mut item.data);
                self.contribute(b, item.data, sfx);
                self.events.push(Ev::Delivered(good.clone()));
                sfx.push(Sfx::Deposit);
            } else if let Some(b) = self.world.bin_at(bx, by) {
                if !self.world.bins[b].takes(&good) {
                    let names: Vec<String> = self.world.bins[b].accepts.iter().map(|a| self.good_name(a)).collect();
                    return StepRes::Blocked(format!("ONLY {} GOES IN THERE.", names.join(" OR ")));
                }
                let mut item = self.world.items.remove(it);
                item.data.hauled_by(id);
                if self.world.bins[b].scores {
                    crate::econ::pay_haul(&mut item.data);
                }
                let bin = &mut self.world.bins[b];
                let scored = bin.scores.then(|| (item.data.value, item.data.shares.clone()));
                bin.stock.push(item.data);
                bin.bounce = 4;
                self.world.burst(bx, by, PKind::Spark, 5);
                if let Some((value, shares)) = scored {
                    self.score(value, &shares, bx, by);
                }
                self.events.push(Ev::Delivered(good.clone()));
                sfx.push(Sfx::Deposit);
            } else if let Some(w) = self.world.wagon_at(bx, by) {
                if let Err(msg) = self.load_wagon(w, it, id) {
                    return StepRes::Blocked(msg);
                }
                self.world.burst(bx, by, PKind::Spark, 3);
                sfx.push(Sfx::Deposit);
            } else {
                let free = self.world.item_floor(bx, by) && self.world.item_at(bx, by).is_none() && self.monster_at(bx, by).is_none() && !self.applicant_at(bx, by) && !self.world.cart_at(bx, by);
                if !free {
                    return StepRes::Blocked(self.water_note(bx, by));
                }
                let item = &mut self.world.items[it];
                item.x = bx;
                item.y = by;
                item.hop = 0;
                item.data.hauled_by(id);
                item.slide = Some(Slide { dir: d, left: ticks, total: ticks });
                if let Some((s, k)) = self.world.pad_owner(bx, by) {
                    self.world.shops[s].pad_flash[k] = 5;
                    sfx.push(Sfx::Pad);
                }
                self.world.dust(tx, ty, d);
                sfx.push(Sfx::Push);
            }
            self.practice(i, "hauling", ticks);
            pushed = Some(good);
        } else if i == self.controlled {
            sfx.push(Sfx::Step);
        }
        if let Some(w) = wagon {
            if !self.world.wagons[w].stock.is_empty() {
                self.practice(i, "wagon_driving", ticks);
            }
        }
        let m = &mut self.monsters[i];
        m.x = tx;
        m.y = ty;
        m.slide = Some(Slide { dir: d, left: ticks, total: ticks });
        m.bump = None;
        if let Some(w) = wagon {
            self.follow(w, (x, y), ticks);
        }
        StepRes::Moved(pushed)
    }

    /// Does this project board take this good: the good itself, or anything tagged with what
    /// it wants (any log for the bridge)?
    pub(crate) fn board_takes(&self, b: usize, good: &str) -> bool {
        let wants = &self.world.boards[b].wants;
        good == wants || self.data.try_good(good).is_some_and(|g| g.type_tags.iter().any(|t| t == wants))
    }

    pub(crate) fn try_use(&mut self, i: usize, sfx: &mut Vec<Sfx>) -> UseRes {
        self.face_something_usable(i);
        if let Some(res) = self.use_wagon_ahead(i, sfx) {
            return res;
        }
        let (x, y) = (self.monsters[i].x, self.monsters[i].y);
        if let Some(s) = self.world.shop_at_spot(x, y) {
            return self.use_shop(i, s, sfx);
        }
        if let Some(p) = self.world.plant_at(x, y) {
            return self.use_plant(i, p, sfx);
        }
        // Leading a wagon with nothing else to use: let go of it.
        if let Some(w) = self.led_wagon(i) {
            self.let_go(w, sfx);
            return UseRes::Done;
        }
        UseRes::Nothing
    }

    fn toggle_record(&mut self, i: usize, sfx: &mut Vec<Sfx>) {
        let leading = self.led_wagon(i).is_some();
        let m = &mut self.monsters[i];
        if m.playing {
            self.say(i, "STOP MY LOOP FIRST (P).", true);
            sfx.push(Sfx::Deny);
            return;
        }
        let pos = (m.x, m.y);
        match m.rec.take() {
            None => {
                m.rec = Some(Routine { flag: pos, acts: Vec::new(), facing: m.facing });
                self.say(i, "RECORDING! R AGAIN TO FINISH.", false);
                sfx.push(Sfx::RecOn);
            }
            Some(r) if r.acts.is_empty() => self.say(i, "NOTHING RECORDED YET.", false),
            // A wagon's route home matters (it follows every step), so that walk is the player's.
            Some(r) if r.flag != pos && leading => {
                m.rec = Some(r);
                self.say(i, "WALK US BACK TO THE FLAG, THEN R.", true);
                sfx.push(Sfx::Deny);
            }
            // Away from the flag: walk back to it, then save. The walk is the loop's last act.
            Some(r) if r.flag != pos => {
                m.rec = Some(r);
                m.homing = true;
                (m.path, m.blocked) = (Vec::new(), 0);
                self.say(i, "HEADING BACK TO MY FLAG.", false);
            }
            Some(r) => self.save_recording(i, r, sfx),
        }
    }

    fn save_recording(&mut self, i: usize, r: Routine, sfx: &mut Vec<Sfx>) {
        self.leave_loop(i);
        let m = &mut self.monsters[i];
        m.routine = Some(r);
        m.pc = 0;
        m.paused_at = None;
        self.say(i, "GOT IT! PRESS P TO LOOP.", false);
        sfx.push(Sfx::RecSave);
    }

    /// One step of a monster walking itself back to its flag to finish recording. There, the
    /// walk home goes on the end of the recording, and it's saved.
    fn walk_home(&mut self, i: usize, sfx: &mut Vec<Sfx>) {
        let Some((flag, facing)) = self.monsters[i].rec.as_ref().map(|r| (r.flag, r.facing)) else {
            self.monsters[i].homing = false;
            return;
        };
        match self.go_toward(i, flag, facing, sfx) {
            crate::walk::Going::Moving => {}
            crate::walk::Going::Arrived => {
                let m = &mut self.monsters[i];
                m.homing = false;
                let Some(mut r) = m.rec.take() else { return };
                record(&mut r, Act::Go { to: flag, facing });
                self.save_recording(i, r, sfx);
            }
            crate::walk::Going::Stuck(_) => {
                self.monsters[i].homing = false;
                self.say(i, "I CAN'T FIND THE WAY BACK. WALK ME TO THE FLAG, THEN R.", true);
                sfx.push(Sfx::Deny);
            }
            // Someone in the way, most likely: keep at it.
            crate::walk::Going::Waiting(msg) => self.say(i, &msg, false),
        }
    }

    fn toggle_play(&mut self, i: usize, sfx: &mut Vec<Sfx>) {
        let m = &mut self.monsters[i];
        if m.rec.is_some() {
            self.say(i, "FINISH RECORDING FIRST (R).", true);
            sfx.push(Sfx::Deny);
            return;
        }
        if m.playing {
            m.playing = false;
            m.paused_at = Some((m.x, m.y));
            self.leave_loop(i);
            self.say(i, "TAKING A BREAK.", false);
            sfx.push(Sfx::LoopOff);
            return;
        }
        let pos = (m.x, m.y);
        let resuming = m.paused_at == Some(pos) && m.pc > 0;
        let own_flag = m.routine.as_ref().is_some_and(|r| r.flag == pos);
        // On another monster's flag: take up their loop, flag and all, and start it.
        let theirs = (0..self.monsters.len()).find(|&o| o != i && self.monsters[o].routine.as_ref().is_some_and(|r| r.flag == pos));
        let mut cheer = "ON IT!".to_string();
        if let Some(o) = theirs.filter(|_| !resuming && !own_flag) {
            let (routine, name) = (self.monsters[o].routine.clone(), self.monsters[o].name.clone());
            let m = &mut self.monsters[i];
            m.routine = routine;
            m.paused_at = None;
            cheer = format!("I'LL DO {name}'S LOOP TOO!");
        }
        let m = &mut self.monsters[i];
        let Some(flag) = m.routine.as_ref().map(|r| r.flag) else {
            self.say(i, "NO ROUTINE YET. R RECORDS ONE.", true);
            sfx.push(Sfx::Deny);
            return;
        };
        if resuming {
            // Resume where the loop was stopped.
        } else if pos == flag {
            m.pc = 0;
            if let Some(f) = m.routine.as_ref().map(|r| r.facing) {
                m.facing = f;
            }
        } else {
            self.say(i, "TAKE ME TO MY FLAG FIRST.", true);
            sfx.push(Sfx::Deny);
            return;
        }
        m.playing = true;
        m.wait = 0;
        m.path.clear();
        m.carry_plan.clear();
        if !resuming {
            self.leave_loop(i);
        }
        self.say(i, &cheer, false);
        sfx.push(Sfx::LoopOn);
    }

    fn advance_motion(&mut self) {
        for m in self.monsters.iter_mut() {
            if let Some(s) = m.slide.as_mut() {
                s.left -= 1;
                m.walk_frame ^= 1;
                if s.left == 0 {
                    m.slide = None;
                }
            }
            if let Some((_, t)) = m.bump.as_mut() {
                *t -= 1;
                if *t == 0 {
                    m.bump = None;
                }
            }
        }
        for it in self.world.items.iter_mut() {
            if let Some(s) = it.slide.as_mut() {
                s.left -= 1;
                if s.left == 0 {
                    it.slide = None;
                }
            }
            it.hop = it.hop.saturating_sub(1);
        }
    }

    fn advance_tutorial(&mut self) {
        let events = std::mem::take(&mut self.events);
        for e in &events {
            if let Ev::Made(g) = e {
                self.made.insert(g.clone());
            }
        }
        let t = &mut self.tutorial;
        t.steps += events.iter().filter(|e| **e == Ev::Walked).count() as u32;
        let stored = |good: &str| self.world.bins.iter().any(|b| b.stock.iter().any(|d| d.good == good));
        let playing = self.monsters.iter().filter(|m| m.playing).count();
        let has_routine = self.monsters.iter().any(|m| m.routine.is_some());
        loop {
            let met = match t.stage {
                0 => t.steps >= 4,
                1 => self.made.contains("cotton_bolls"),
                2 => stored("cotton_bolls") || self.made.contains("cotton_slivers"),
                3 => self.made.contains("cotton_slivers"),
                4 => self.made.contains("thread"),
                5 => self.made.contains("fabric"),
                6 => stored("fabric"),
                7 => has_routine,
                8 => playing >= 2,
                9 => self.monsters.len() > 2,
                10 => self.made.contains("rope"),
                11 => self.world.ferry.as_ref().is_some_and(|f| f.rigged),
                12 => self.monsters.iter().any(|m| area_name(m.x) == "TANGLEMERE"),
                13 => self.made.contains("lye"),
                14 => self.made.contains("quicklime"),
                15 => self.made.contains("indigo"),
                16 => stored("dyed_fabric"),
                17 => self.world.wagons.iter().any(|w| w.last_leader.is_some()),
                18 => events.contains(&Ev::Unloaded),
                19 => self.world.bridge_built(),
                _ => false,
            };
            if !met || t.stage >= tutorial::DONE {
                break;
            }
            t.stage += 1;
        }
    }

    /// Fabric in the cloth store: the HUD's running tally for now.
    pub fn cloth_stored(&self) -> usize {
        self.world.bins.iter().flat_map(|b| &b.stock).filter(|d| d.good == "fabric" || d.good == "dyed_fabric").count()
    }

    /// The controlled monster's state for the HUD, and whether it is a warning.
    pub fn status(&self, i: usize) -> (String, bool) {
        let m = &self.monsters[i];
        let who = format!("{} THE {}", m.name, m.kind.to_uppercase());
        let riding = self.world.ferry.as_ref().is_some_and(|f| f.moving && f.on_deck(m.x, m.y));
        let state = if m.busy {
            ("WORKING".to_string(), false)
        } else if riding {
            ("RIDING THE FERRY".to_string(), false)
        } else if let Some(w) = self.led_wagon(i).filter(|_| !m.playing) {
            let slow = self.wagon_ticks(i, w) > PUSH_TICKS;
            (if slow { "LEADING THE WAGON. THE LOAD SLOWS IT".to_string() } else { "LEADING THE WAGON".to_string() }, false)
        } else if m.slide.is_some_and(|s| s.total == PULL_TICKS) {
            ("PULLING. HEAVY GOING".to_string(), false)
        } else if let Some(r) = &m.rec {
            (format!("RECORDING ({} ACTIONS){}", r.acts.len(), if m.homing { ". HEADING BACK TO THE FLAG" } else { "" }), false)
        } else if m.playing && m.wait >= WAIT_BEFORE_SAYING {
            (m.wait_reason.trim_end_matches('.').to_string(), true)
        } else if m.playing {
            (format!("LOOPING, {} DONE", m.loops), false)
        } else if let Some(preview) = crate::inspect::spot_preview(self, i) {
            (preview, false)
        } else if m.routine.is_some() {
            ("IDLE. P RUNS THE SAVED LOOP".to_string(), false)
        } else {
            ("IDLE".to_string(), false)
        };
        (format!("{who}: {}", state.0), state.1)
    }

    /// Scripted play for tests and screenshots: `U D L R` move, `u` use, `r` record,
    /// `p` play, `t` tab, `x` cancel, `h` help. Waits for each action to finish.
    pub fn script(&mut self, keys: &str) {
        let (mut req, mut sfx) = (Vec::new(), Vec::new());
        for ch in keys.chars() {
            let mut inp = Input::default();
            match ch {
                'U' | 'D' | 'L' | 'R' => {
                    let k = "UDLR".find(ch).unwrap_or(0);
                    inp.pressed[k] = true;
                    inp.held[k] = true;
                }
                // `^ v < >`: Shift held, stepping that way (a pull, if an item is in front).
                '^' | 'v' | '<' | '>' => {
                    let k = "^v<>".find(ch).unwrap_or(0);
                    inp.pressed[k] = true;
                    inp.held[k] = true;
                    inp.pull = true;
                }
                'u' => inp.use_pressed = true,
                'r' => inp.rec_pressed = true,
                'p' => inp.play_pressed = true,
                't' => inp.tab_pressed = true,
                'x' => inp.cancel_pressed = true,
                'h' => inp.help_pressed = true,
                _ => continue,
            }
            self.update(&inp, &mut req, &mut sfx);
            for _ in 0..2000 {
                if self.monsters[self.controlled].idle() {
                    break;
                }
                self.update(&Input::default(), &mut req, &mut sfx);
            }
        }
    }

    pub fn run_ticks(&mut self, n: u32) {
        let (mut req, mut sfx) = (Vec::new(), Vec::new());
        for _ in 0..n {
            self.update(&Input::default(), &mut req, &mut sfx);
        }
    }
}

/// Scripted routes on the Pinewater map, for tests.
#[cfg(test)]
pub mod routes {
    /// Mogg, from the start west of the fields: harvest the east end of the bottom row and
    /// push the boll into the cotton bin.
    pub const MOGG_HARVEST: &str = "RRRRRRu RRR";
    /// Back to the start along row 7, clear of the plant's drop tile.
    pub const MOGG_HOME: &str = "ULLLLLLLLLD";

    /// Bruna, from beside the chute: workbench, spinnery, loom house, cloth store.
    pub fn bruna_chain() -> String {
        format!("RDRRUu{}URuD{}UR{}LDuR{}", "R".repeat(15), "R".repeat(13), "D".repeat(12), "D".repeat(6))
    }

    /// From the cloth store back to the flag beside the chute, staying clear of the chute.
    pub fn bruna_home() -> String {
        format!("{}{}D{}U", "U".repeat(18), "L".repeat(28), "L".repeat(5))
    }
}

#[cfg(test)]
#[path = "game_tests.rs"]
mod tests;

#[cfg(test)]
#[path = "tangle_tests.rs"]
mod tangle_tests;
