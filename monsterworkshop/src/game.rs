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
    /// Every sound, in the order they're numbered when sent to a player.
    #[cfg_attr(not(windows), allow(dead_code))]
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

/// One tick of latched input. Direction arrays are indexed like `DIRS`.
#[derive(Default, Clone, Copy, PartialEq, Debug)]
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
    /// C: open or close the character sheet.
    pub sheet_pressed: bool,
    /// Esc with the character sheet open: close it. (A player's game turns Esc into this,
    /// rather than leaving the world, while it knows the sheet is open.)
    pub close_pressed: bool,
}

impl Input {
    /// Both together: anything held or gone down in either. Keys polled between ticks are
    /// gathered like this.
    pub fn or(&self, o: &Input) -> Input {
        let mut i = *self;
        for k in 0..4 {
            i.held[k] |= o.held[k];
            i.pressed[k] |= o.pressed[k];
        }
        for (a, b) in [
            (&mut i.use_pressed, o.use_pressed),
            (&mut i.rec_pressed, o.rec_pressed),
            (&mut i.play_pressed, o.play_pressed),
            (&mut i.tab_pressed, o.tab_pressed),
            (&mut i.tab_held, o.tab_held),
            (&mut i.cancel_pressed, o.cancel_pressed),
            (&mut i.help_pressed, o.help_pressed),
            (&mut i.quit_pressed, o.quit_pressed),
            (&mut i.delete_pressed, o.delete_pressed),
            (&mut i.inspect, o.inspect),
            (&mut i.pull, o.pull),
            (&mut i.sheet_pressed, o.sheet_pressed),
            (&mut i.close_pressed, o.close_pressed),
        ] {
            *a |= b;
        }
        i
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub enum Species {
    Goblin,
    Cyclops,
}

/// One recorded action. A step remembers which good it pushed, so replay can tell "it isn't
/// here yet" (wait) from "something else is in the way" (also wait, but say so).
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub enum Goal {
    /// Press Space at this thing.
    Activate(Thing),
    /// Get the good lying at `from` to `to`, pushing (and pulling) it however works. `to` may be
    /// a bin, board or wagon that takes it.
    Move { good: String, from: (i32, i32), to: (i32, i32) },
    /// Take this wagon's lead and park it with its bed at `bed` and its pumpkin at `pumpkin`
    /// (so its tailgate faces away from the pumpkin), then let go.
    Lead { wagon: usize, bed: (i32, i32), pumpkin: (i32, i32) },
}

/// Something a monster activates with Space. Each says where it's activated from.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Thing {
    /// Harvest the plant at `at`, standing on it and facing `facing`: the harvest is set out on
    /// that side.
    Plant { at: (i32, i32), facing: Dir },
    /// Work this workshop or bench, standing on its work spot.
    Shop(usize),
    /// Open or shut this wagon's tailgate, standing behind it.
    Tailgate { wagon: usize, open: bool },
}

/// A loop: its flag, and its goals in the order they were shown.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Routine {
    pub flag: (i32, i32),
    pub goals: Vec<Goal>,
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
    /// The commune it belongs to (an index into `Game::communes`).
    pub commune: usize,
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
    /// The way to where its current goal takes it.
    #[serde(default)]
    pub path: Vec<(i32, i32)>,
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
    /// The goal of its loop it's working on (by its index), once it has picked one.
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
    /// Giving way to another monster: off its way, until it has gone by.
    #[serde(default)]
    pub giving_way: Option<crate::motion::GiveWay>,
    /// A knot it's untangling, as the one whose way is being cleared (see `knot.rs`).
    #[serde(default)]
    pub knot: Option<crate::knot::Knot>,
    /// Ticks it goes on saying there's no way through without passing through someone.
    #[serde(default)]
    pub no_room: u16,
    /// Its two hands: the gathering tools it holds (see Equipment in DESIGN.md).
    #[serde(default)]
    pub hands: [Option<crate::econ::ItemData>; 2],
    /// Why it last gave a goal up (someone in the way, say), told until it next finishes one.
    #[serde(default)]
    pub gave_up: String,
    /// The tick it last finished a step of its loop (or started the loop): who has gone longest
    /// without finishing one goes first.
    #[serde(default)]
    pub since: u32,
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

pub(crate) const NAMES: [&str; 16] = ["GRUB", "NELL", "TOK", "ISKA", "BRAM", "ODA", "PIP", "WREN", "MOSK", "HILDA", "ZUG", "FERN", "KORR", "LUMA", "DREG", "SIV"];
/// Ticks before an empty stand gets a new applicant, and between roster refreshes.
const APPLICANT_ARRIVES: u32 = 300;
/// Renown a scored good pays per point of its value (values come from the tech tree's raw
/// materials and are small). A first tuning guess: early cotton fabric pays about 20.
pub const RENOWN_PER_VALUE: f64 = 10.0;
const ROSTER_REFRESH: u32 = 4500;

impl Monster {
    pub(crate) fn new(id: u32, commune: usize, name: &str, kind: &str, data: &Data, (x, y): (i32, i32), skills: Skills) -> Self {
        let species = if kind == "goblin" { Species::Goblin } else { Species::Cyclops };
        Self {
            id,
            commune,
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
            carry: None,
            carry_plan: Vec::new(),
            shove: None,
            homing: false,
            task: None,
            holds: Vec::new(),
            cooldown: Vec::new(),
            turns: Vec::new(),
            waiting_on: None,
            giving_way: None,
            knot: None,
            no_room: 0,
            hands: [None, None],
            gave_up: String::new(),
            since: 0,
            loops: 0,
            steps: 0,
            earned: 0.0,
            learned: None,
        }
    }

    pub fn idle(&self) -> bool {
        self.slide.is_none() && !self.busy
    }

    /// Round the end of its loop and back to the start.
    pub(crate) fn lap(&mut self) {
        self.loops += 1;
    }

}

/// Adds a goal to a recording. Moving the same good on from where its last move left it extends
/// that move, and a good put back where it started (a slip, corrected) records nothing.
fn record(r: &mut Routine, goal: Goal) {
    if let Goal::Move { good, from, to } = &goal {
        if let Some(Goal::Move { good: g, from: first, to: last_to }) = r.goals.last_mut() {
            if g == good && last_to == from {
                *last_to = *to;
                if first == to {
                    r.goals.pop();
                }
                return;
            }
        }
    }
    r.goals.push(goal);
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

/// A player's commune: its renown, its tutorial, the monster its player controls, and what that
/// player sees. A commune is one player. Its monsters each say which commune they belong to.
/// Communes are only ever added, so a commune's index never changes.
#[derive(Default, Serialize, Deserialize)]
pub struct Commune {
    /// The key its player's game identifies itself with; empty until a player claims it.
    pub key: String,
    pub renown: f64,
    /// The monster this player controls, always one of the commune's own.
    pub controlled: usize,
    pub tutorial: Tutorial,
    /// Goods its monsters have made at least once (tutorial progress).
    pub made: BTreeSet<String>,
    pub(crate) examine_hinted: bool,
    #[serde(skip)]
    queued: Option<Dir>,
    #[serde(skip)]
    queued_use: bool,
    /// The direction held since its press, and for how many ticks.
    #[serde(skip)]
    hold: Option<(Dir, u8)>,
    // What this player's screen shows. The server draws it.
    #[serde(skip)]
    pub help: bool,
    /// Holding E: show the inspect card for whatever the controlled monster faces.
    #[serde(skip)]
    pub inspecting: bool,
    /// Holding Tab: the roster strip shows the commune's monsters and what they're doing.
    #[serde(skip)]
    pub roster: bool,
    /// Which page of the inspect card shows; Tab turns it while E is held.
    #[serde(skip)]
    pub card_page: usize,
    /// Recipes its monsters have heard of without making them yet (from a craftsman): the
    /// recipe book lists them with what they've made.
    #[serde(default)]
    pub heard: BTreeSet<String>,
    /// Which of a craftsman's ideas this player hears next.
    #[serde(skip)]
    pub idea_turn: usize,
    /// How far down the recipe book (the sheet's third page) is scrolled.
    #[serde(skip)]
    pub sheet_scroll: usize,
    /// The character sheet (C): open, which page, and which hand is chosen.
    #[serde(skip)]
    pub sheet: bool,
    #[serde(skip)]
    pub sheet_page: usize,
    #[serde(skip)]
    pub sheet_hand: usize,
    /// The named area the view is in, and a banner announcing it on arrival.
    #[serde(skip)]
    pub(crate) area: &'static str,
    #[serde(skip)]
    pub banner: Option<(&'static str, u16)>,
}

/// Who hears a sound: everyone looking at the screen it's on, or only one commune's player
/// (what their own actions sound like).
#[derive(Clone, Copy)]
pub(crate) enum Heard {
    At((i32, i32)),
    By(usize),
}

pub struct Game {
    pub data: Data,
    pub world: World,
    pub monsters: Vec<Monster>,
    pub communes: Vec<Commune>,
    pub frame: u32,
    pub rng: Rng,
    /// What happened this tick, and to which commune (tutorial progress).
    pub(crate) events: Vec<(usize, Ev)>,
    /// This tick's sounds. Each player hears theirs through `sounds_for`.
    sounds: Vec<(Heard, Sfx)>,
    /// One slot per stand in the gathering hall; empty while a new applicant is on the way.
    pub applicants: Vec<Option<Applicant>>,
    pub(crate) applicant_timers: Vec<u32>,
    pub(crate) roster_timer: u32,
    pub(crate) next_id: u32,
    pub floaters: Vec<Floater>,
}

impl Game {
    /// A new world, with the first player's commune: Mogg and Bruna.
    pub fn new() -> Self {
        let data = Data::load();
        let world = World::new(&data);
        Self::on(data, world)
    }

    /// A new game in `world`, with the first player's commune: Mogg and Bruna.
    pub(crate) fn on(data: Data, world: World) -> Self {
        let stands = world.stands.len();
        let mut g = Self {
            data,
            world,
            monsters: Vec::new(),
            communes: Vec::new(),
            frame: 0,
            rng: Rng::new(0x5EED_2026),
            events: Vec::new(),
            sounds: Vec::new(),
            applicants: Vec::new(),
            applicant_timers: vec![0; stands],
            roster_timer: 0,
            next_id: 1,
            floaters: Vec::new(),
        };
        g.found_commune(["MOGG", "BRUNA"]);
        g.applicants = (0..stands).map(|_| Some(g.roll_applicant())).collect();
        g
    }

    /// The commune of the player with this key: theirs if they've played here before, else one
    /// nobody has claimed yet (a new world's first), else a new one.
    pub fn player(&mut self, key: &str) -> usize {
        if let Some(k) = self.communes.iter().position(|c| c.key == key) {
            return k;
        }
        let k = self.communes.iter().position(|c| c.key.is_empty()).unwrap_or_else(|| self.join());
        self.communes[k].key = key.to_string();
        k
    }

    /// A new player joins: their commune starts at the first town's gate with two monsters, a
    /// crafter and a gatherer, on the nearest free tiles. Returns the commune's index.
    pub fn join(&mut self) -> usize {
        let roll = |rng: &mut Rng| NAMES[(rng.next_u64() % NAMES.len() as u64) as usize];
        let crafter = roll(&mut self.rng);
        let gatherer = roll(&mut self.rng);
        self.found_commune([crafter, gatherer])
    }

    fn found_commune(&mut self, [crafter, gatherer]: [&str; 2]) -> usize {
        let k = self.communes.len();
        let skills = [
            starting_skills(&["handcrafts", "science", "engineering"], &[("spinning", 0.3), ("textiles", 0.3), ("weaving", 0.2)]),
            starting_skills(&["outdoorsmonstership", "athletics", "social"], &[("harvesting", 0.3), ("gathering", 0.3)]),
        ];
        let first = self.monsters.len();
        for (n, ((name, kind), skills)) in [(crafter, "goblin"), (gatherer, "cyclops")].into_iter().zip(skills).enumerate() {
            let at = self.free_tile_near(self.world.starts[n]);
            self.monsters.push(Monster::new(self.next_id, k, name, kind, &self.data, at, skills));
            self.next_id += 1;
        }
        self.monsters[first].bubble = Some(Bubble { text: format!("HI, I'M {crafter}! LET'S MAKE SOME CLOTH."), ttl: 60, warn: false, waiting: false });
        let m = &self.monsters[first];
        let area = area_name(crate::render::screen_of(m.x, m.y).0);
        self.communes.push(Commune { controlled: first, area, ..Commune::default() });
        k
    }

    /// The nearest tile to `from` that nothing stands on (itself, if free), searching outward
    /// over walkable ground.
    pub(crate) fn free_tile_near(&self, from: (i32, i32)) -> (i32, i32) {
        let free = |(x, y): (i32, i32)| self.monster_at(x, y).is_none() && self.world.item_at(x, y).is_none() && !self.applicant_at(x, y) && !self.world.cart_at(x, y);
        let mut seen = std::collections::HashSet::from([from]);
        let mut queue = std::collections::VecDeque::from([from]);
        while let Some(at) = queue.pop_front() {
            if free(at) {
                return at;
            }
            for d in DIRS {
                let (dx, dy) = d.delta();
                let next = (at.0 + dx, at.1 + dy);
                if self.world.walkable(next.0, next.1) && seen.insert(next) {
                    queue.push_back(next);
                }
            }
        }
        from
    }

    /// The commune whose player controls monster `i`, if any.
    pub fn controller(&self, i: usize) -> Option<usize> {
        let k = self.monsters[i].commune;
        (self.communes[k].controlled == i).then_some(k)
    }

    /// What commune `k`'s player hears this tick: their own actions, and what happens on the
    /// screen they're looking at.
    pub fn sounds_for(&self, k: usize) -> Vec<Sfx> {
        let m = &self.monsters[self.communes[k].controlled];
        let screen = crate::render::screen_of(m.x, m.y);
        self.sounds
            .iter()
            .filter(|(h, _)| match *h {
                Heard::By(c) => c == k,
                Heard::At((x, y)) => crate::render::screen_of(x, y) == screen,
            })
            .map(|&(_, s)| s)
            .collect()
    }

    /// A sound at a tile, heard by whoever is looking at that screen.
    pub(crate) fn sound(&mut self, at: (i32, i32), s: Sfx) {
        self.sounds.push((Heard::At(at), s));
    }

    /// Something that happened to monster `i`'s commune, for its tutorial.
    pub(crate) fn event(&mut self, i: usize, e: Ev) {
        self.events.push((self.monsters[i].commune, e));
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
    /// Is someone who isn't a monster of any commune standing here: an applicant waiting at the
    /// hall, or a townsperson like Old Sawgrass? Either blocks the way like anyone.
    pub fn applicant_at(&self, x: i32, y: i32) -> bool {
        self.world.stands.iter().zip(&self.applicants).any(|(&s, a)| s == (x, y) && a.is_some()) || self.world.townsfolk.iter().any(|t| (t.x, t.y) == (x, y))
    }

    /// The townsperson monster `c` faces, if any.
    pub fn faced_townsfolk(&self, c: usize) -> Option<usize> {
        let m = &self.monsters[c];
        let (dx, dy) = m.facing.delta();
        self.world.townsfolk.iter().position(|t| (t.x, t.y) == (m.x + dx, m.y + dy))
    }

    /// The applicant stand monster `c` faces, if someone waits there.
    pub fn faced_applicant(&self, c: usize) -> Option<usize> {
        let m = &self.monsters[c];
        let (dx, dy) = m.facing.delta();
        let k = self.world.stands.iter().position(|&s| s == (m.x + dx, m.y + dy))?;
        self.applicants[k].as_ref().map(|_| k)
    }

    /// An applicant joins monster `c`'s commune; its renown makes room for them.
    fn welcome(&mut self, c: usize, k: usize, sfx: &mut Vec<Sfx>) {
        if self.monsters[c].rec.is_some() {
            self.say(c, "FINISH RECORDING FIRST (R).", true);
            sfx.push(Sfx::Deny);
            return;
        }
        let Some(a) = self.applicants[k].clone() else { return };
        let commune = self.monsters[c].commune;
        if self.communes[commune].renown < a.cost as f64 {
            self.say(c, &format!("THEY JOIN AT {} RENOWN.", a.cost), true);
            sfx.push(Sfx::Deny);
            return;
        }
        self.communes[commune].renown -= a.cost as f64;
        self.applicants[k] = None;
        self.applicant_timers[k] = APPLICANT_ARRIVES;
        let skills = Skills { transferable: a.transferable.clone(), ..Skills::default() };
        let mut m = Monster::new(self.next_id, commune, &a.name, &a.kind, &self.data, self.world.stands[k], skills);
        self.next_id += 1;
        m.facing = Dir::Up;
        m.bubble = Some(Bubble { text: format!("I'M {}! HAPPY TO JOIN.", a.name), ttl: BUBBLE_TICKS, warn: false, waiting: false });
        self.monsters.push(m);
        sfx.push(Sfx::RecSave);
    }

    /// A scored good pays renown, split by shares: each contributor's share goes to its commune,
    /// and flies to it as credit.
    pub(crate) fn score(&mut self, value: f64, shares: &[Share], bx: i32, by: i32) {
        let value = value * RENOWN_PER_VALUE;
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
                self.communes[m.commune].renown += credit;
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
            Project::Footbridge => {
                self.world.build_footbridge();
                let span = self.world.footbridge.clone();
                for &(x, y) in span.iter().step_by(2) {
                    self.world.burst(x, y, PKind::Spark, 4);
                }
                let (Some(&(x0, y)), Some(&(x1, _))) = (span.first(), span.last()) else { return };
                let at = (((x0 + x1) / 2) * TS + 8, y * TS);
                self.floaters.push(Floater { text: "THE FOOTBRIDGE IS UP!".into(), from: at, to: (at.0, at.1 - 30), t: 0, total: 60, delay: 20, color: SPARK + 2 });
            }
        }
        sfx.push(Sfx::Chime);
    }

    /// Why a step into water fails: where the footbridge goes, that it needs rope.
    fn water_note(&self, x: i32, y: i32) -> String {
        if self.world.footbridge.contains(&(x, y)) && !self.world.footbridge_built() {
            "THE FOOTBRIDGE NEEDS ROPE.".into()
        } else {
            String::new()
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

    /// One tick. `inputs[k]` is what commune `k`'s player pressed; a commune with none (its
    /// player away) presses nothing, and its monsters carry on.
    pub fn update(&mut self, inputs: &[Input]) {
        self.frame = self.frame.wrapping_add(1);
        self.sounds.clear();
        for k in 0..self.communes.len() {
            let mut sfx = Vec::new();
            self.player_input(k, &inputs.get(k).copied().unwrap_or_default(), &mut sfx);
            self.sounds.extend(sfx.into_iter().map(|s| (Heard::By(k), s)));
        }
        for i in 0..self.monsters.len() {
            // Heard on the screen where the monster ends its turn.
            let mut heard = Vec::new();
            if self.monsters[i].playing {
                self.run_loop(i, &mut heard);
            } else if self.monsters[i].homing && self.monsters[i].idle() {
                self.walk_home(i, &mut heard);
            }
            let at = (self.monsters[i].x, self.monsters[i].y);
            self.sounds.extend(heard.into_iter().map(|s| (Heard::At(at), s)));
        }
        // Knots' moves are only ever steps, which make no sound but yours.
        self.untangle(&mut Vec::new());
        self.advance_motion();
        self.tick_wagons();
        self.tick_work();
        self.tick_frontier();
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
        for c in self.communes.iter_mut() {
            let m = &self.monsters[c.controlled];
            let area = area_name(crate::render::screen_of(m.x, m.y).0);
            if area != c.area {
                c.area = area;
                c.banner = Some((area, BANNER_TICKS));
            } else if let Some((_, t)) = c.banner.as_mut() {
                *t = t.saturating_sub(1);
                if *t == 0 {
                    c.banner = None;
                }
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

    /// Commune `k`'s player: the screen they look at, and the monster they control.
    fn player_input(&mut self, k: usize, inp: &Input, sfx: &mut Vec<Sfx>) {
        let me = &mut self.communes[k];
        if inp.help_pressed {
            me.help = !me.help;
        }
        if inp.sheet_pressed {
            me.sheet = !me.sheet;
            (me.sheet_page, me.sheet_hand) = (0, 0);
        }
        if inp.close_pressed {
            me.sheet = false;
        }
        if me.sheet {
            // The sheet takes the keys: Tab turns its page, up and down choose a hand, Space
            // puts that hand's tool down on the tile ahead. The world goes on behind it.
            me.inspecting = false;
            me.roster = false;
            (me.hold, me.queued, me.queued_use) = (None, None, false);
            if inp.tab_pressed {
                me.sheet_page = (me.sheet_page + 1) % 3;
                me.sheet_scroll = 0;
            }
            // Up and down choose a hand on the first page, and scroll the book on the third.
            match me.sheet_page {
                0 if inp.pressed[0] || inp.pressed[1] => me.sheet_hand = 1 - me.sheet_hand,
                2 if inp.pressed[0] => me.sheet_scroll = me.sheet_scroll.saturating_sub(1),
                2 if inp.pressed[1] => me.sheet_scroll += 1,
                _ => {}
            }
            let (c, hand) = (me.controlled, me.sheet_hand);
            if inp.use_pressed && !self.monsters[c].playing && self.monsters[c].idle() {
                self.put_down(c, hand, sfx);
            }
            return;
        }
        if inp.inspect && !me.inspecting {
            me.card_page = 0;
        }
        me.inspecting = inp.inspect;
        me.roster = inp.tab_held && !inp.inspect;
        if inp.tab_pressed && me.inspecting {
            // While examining, Tab turns the card's page instead of switching monsters.
            me.card_page += 1;
        } else if inp.tab_pressed {
            // The next of the commune's own monsters.
            let (n, cur) = (self.monsters.len(), me.controlled);
            me.controlled = (1..=n).map(|d| (cur + d) % n).find(|&j| self.monsters[j].commune == k).unwrap_or(cur);
            (me.queued, me.queued_use) = (None, false);
            self.events.push((k, Ev::Switched));
            sfx.push(Sfx::Switch);
        }
        let c = self.communes[k].controlled;
        let held = |d: Dir| inp.held[DIRS.iter().position(|&x| x == d).unwrap_or(0)];
        let mut fresh = None;
        for (n, d) in DIRS.iter().enumerate() {
            if inp.pressed[n] {
                fresh = Some(*d);
            }
        }
        let me = &mut self.communes[k];
        me.hold = match (fresh, me.hold) {
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
            let me = &mut self.communes[k];
            if fresh.is_some() {
                me.queued = fresh;
            }
            me.queued_use |= inp.use_pressed;
            return;
        }

        if inp.use_pressed {
            self.face_something_usable(c);
            if let Some(a) = self.faced_applicant(c) {
                self.welcome(c, a, sfx);
                return;
            }
        }
        if inp.use_pressed || std::mem::take(&mut self.communes[k].queued_use) {
            self.communes[k].queued = None;
            self.face_something_usable(c);
            let (thing, led) = (self.thing_here(c), self.led_wagon(c));
            match self.try_use(c, sfx) {
                UseRes::Done => match (led, thing) {
                    (Some(w), _) => {
                        let wagon = &self.world.wagons[w];
                        let (bed, pumpkin) = (wagon.at, wagon.pumpkin);
                        self.record_goal(c, Goal::Lead { wagon: w, bed, pumpkin });
                    }
                    (None, Some(t)) => self.record_goal(c, Goal::Activate(t)),
                    (None, None) => {}
                },
                UseRes::Wait(msg) | UseRes::Missing(msg) => {
                    // Space on a plant is always part of a routine, ripe or not: on replay the
                    // monster waits there until it can harvest.
                    if let Some(t @ Thing::Plant { .. }) = thing {
                        self.record_goal(c, Goal::Activate(t));
                    }
                    self.say(c, &msg, true);
                    sfx.push(Sfx::Deny);
                }
                // Still part of a routine: on replay there's usually room by then, and if not
                // the loop skips it.
                UseRes::Full(msg) => {
                    if let Some(t) = thing {
                        self.record_goal(c, Goal::Activate(t));
                    }
                    self.say(c, &msg, true);
                    sfx.push(Sfx::Deny);
                }
                UseRes::Nothing => self.say(c, "NOTHING TO USE HERE.", false),
            }
            return;
        }

        let pressed_now = self.communes[k].queued.take().or(fresh);
        let repeat = self.communes[k].hold.filter(|&(_, t)| t >= HOLD_REPEAT_TICKS).map(|(d, _)| d);
        let Some(d) = pressed_now.or(repeat) else { return };
        // Shift with an item in front and a step away from it: pull. Otherwise Shift is ignored.
        let pulling = inp.pull && self.pulled_item(c, d).is_some() && self.led_wagon(c).is_none();
        let res = if pulling { self.try_pull(c, d, sfx) } else { self.try_step(c, d, sfx) };
        match res {
            StepRes::Moved(moved) => {
                let (dx, dy) = d.delta();
                let to = (self.monsters[c].x, self.monsters[c].y);
                // Moving a good records where it went, not how. Walking records nothing: every
                // goal brings its own walk.
                if let (Some(good), Some(r)) = (moved, self.monsters[c].rec.as_mut()) {
                    // Pushed on ahead, or pulled along into the tile left.
                    let (from, to) = if pulling { ((to.0 - 2 * dx, to.1 - 2 * dy), (to.0 - dx, to.1 - dy)) } else { (to, (to.0 + dx, to.1 + dy)) };
                    record(r, Goal::Move { good, from, to });
                }
                if pulling {
                    self.say(c, "HNNNG...", false);
                }
                self.events.push((k, Ev::Walked));
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

    fn record_goal(&mut self, c: usize, goal: Goal) {
        if let Some(r) = self.monsters[c].rec.as_mut() {
            record(r, goal);
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
        // Leading a wagon, stepping back into the pumpkin backs the train up.
        if let Some(w) = self.led_wagon(i).filter(|&w| self.world.wagons[w].pumpkin == (tx, ty)) {
            return self.back_up(i, w, d);
        }
        if !self.world.walkable(tx, ty) || self.monster_at(tx, ty).is_some() || self.applicant_at(tx, ty) || self.world.cart_at(tx, ty) {
            return StepRes::Blocked(self.water_note(tx, ty));
        }
        let wagon = self.led_wagon(i);
        if wagon.is_some() && !self.world.wagon_way(tx, ty) {
            return StepRes::Blocked("WAGONS DON'T FIT ON THE FOOTBRIDGE.".into());
        }
        if wagon.is_some() && self.world.item_at(tx, ty).is_some() {
            return StepRes::Blocked("LET GO OF THE WAGON TO PUSH THAT.".into());
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
                self.event(i, Ev::Delivered(good.clone()));
                sfx.push(Sfx::Deposit);
            } else if let Some(b) = self.world.bin_at(bx, by) {
                if let Err(msg) = self.bin_takes(b, &good) {
                    return StepRes::Blocked(msg);
                }
                let mut item = self.world.items.remove(it);
                item.data.hauled_by(id);
                if self.world.bins[b].scores {
                    crate::econ::pay_haul(&mut item.data);
                }
                let scored = self.world.bins[b].scores.then(|| (item.data.value, item.data.shares.clone()));
                self.put_in_bin(b, item.data);
                self.world.burst(bx, by, PKind::Spark, 5);
                if let Some((value, shares)) = scored {
                    self.score(value, &shares, bx, by);
                }
                self.event(i, Ev::Delivered(good.clone()));
                sfx.push(Sfx::Deposit);
            } else if let Some(w) = self.world.wagon_at(bx, by) {
                if let Err(msg) = self.load_wagon(w, it, id) {
                    return StepRes::Blocked(msg);
                }
                self.world.burst(bx, by, PKind::Spark, 3);
                sfx.push(Sfx::Deposit);
            } else if let Some(s) = self.world.shop_at_spot(bx, by).filter(|&s| self.world.shops[s].frontier.is_some()) {
                // A kit pushed onto a frontier workshop's work spot renews it.
                if let Err(msg) = self.renew_shop(s, it, id) {
                    return StepRes::Blocked(msg);
                }
                sfx.push(Sfx::Chime);
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
        } else if self.controller(i).is_some() {
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
        // Leading a wagon, Space lets go of it.
        if let Some(w) = self.led_wagon(i) {
            self.let_go(w, sfx);
            return UseRes::Done;
        }
        self.face_something_usable(i);
        if let Some(res) = self.use_wagon_ahead(i, sfx) {
            return res;
        }
        // Asking a craftsman: another idea, and it goes in the commune's book.
        if let Some(t) = self.faced_townsfolk(i).filter(|_| !self.monsters[i].playing) {
            let k = self.monsters[i].commune;
            self.communes[k].idea_turn += 1;
            let ideas = self.ideas(t, k);
            if let Some(idea) = ideas.get(self.communes[k].idea_turn % ideas.len().max(1)) {
                let r = idea.recipe.to_string();
                if !self.communes[k].made.contains(&r) {
                    self.communes[k].heard.insert(r);
                }
            }
            sfx.push(Sfx::Pop);
            return UseRes::Done;
        }
        let (x, y) = (self.monsters[i].x, self.monsters[i].y);
        if let Some(s) = self.world.shop_at_spot(x, y) {
            return self.use_shop(i, s, sfx);
        }
        if let Some(p) = self.world.plant_at(x, y) {
            return self.use_plant(i, p, sfx);
        }
        if let Some(k) = self.kit_ahead(i) {
            return self.unpack_kit(i, k, sfx);
        }
        // Taking a tool in hand is the player's doing, never a loop's.
        if let Some(it) = self.tool_ahead(i).filter(|_| !self.monsters[i].playing) {
            return self.take_tool(i, it, sfx);
        }
        UseRes::Nothing
    }

    /// A tool on the ground just ahead of monster `i` that it could hold in a hand.
    fn tool_ahead(&self, i: usize) -> Option<usize> {
        let m = &self.monsters[i];
        let (dx, dy) = m.facing.delta();
        self.world.item_at(m.x + dx, m.y + dy).filter(|&it| self.data.try_good(self.world.items[it].good()).is_some_and(|g| self.data.holdable(g)))
    }

    /// What monster `i` holds weighs against its body's capacity.
    fn held_weight(&self, i: usize) -> f64 {
        self.monsters[i].hands.iter().flatten().map(|d| crate::econ::weight(self.data.good(&d.good), d)).sum()
    }

    /// Monster `i` takes tool `it` from the ground into a free hand, if it has one and can
    /// carry it with what it holds already.
    fn take_tool(&mut self, i: usize, it: usize, sfx: &mut Vec<Sfx>) -> UseRes {
        let Some(hand) = self.monsters[i].hands.iter().position(|h| h.is_none()) else {
            sfx.push(Sfx::Deny);
            return UseRes::Wait("MY HANDS ARE FULL.".into());
        };
        let data = &self.world.items[it].data;
        let cap = self.data.monster_types[self.monsters[i].kind.as_str()].body_cap as f64;
        if self.held_weight(i) + crate::econ::weight(self.data.good(&data.good), data) > cap {
            sfx.push(Sfx::Deny);
            return UseRes::Wait("THAT'S MORE THAN I CAN HOLD.".into());
        }
        let data = self.world.items.remove(it).data;
        let text = format!("{} TAKES THE {}.", self.monsters[i].name, self.good_name(&data.good));
        self.monsters[i].hands[hand] = Some(data);
        self.say(i, &text, false);
        sfx.push(Sfx::Pop);
        UseRes::Done
    }

    /// Monster `i` puts the tool in `hand` down on the tile ahead, from the character sheet.
    pub(crate) fn put_down(&mut self, i: usize, hand: usize, sfx: &mut Vec<Sfx>) {
        let m = &self.monsters[i];
        let (dx, dy) = m.facing.delta();
        let (x, y) = (m.x + dx, m.y + dy);
        let Some(good) = m.hands[hand].as_ref().map(|d| d.good.clone()) else {
            self.say(i, "THAT HAND IS EMPTY.", true);
            sfx.push(Sfx::Deny);
            return;
        };
        let free = self.world.item_floor(x, y) && self.world.item_at(x, y).is_none() && self.monster_at(x, y).is_none() && !self.world.cart_at(x, y);
        if !free {
            self.say(i, "NO ROOM TO PUT IT DOWN THERE.", true);
            sfx.push(Sfx::Deny);
            return;
        }
        let text = format!("{} PUTS DOWN THE {}.", self.monsters[i].name, self.good_name(&good));
        if let Some(data) = self.monsters[i].hands[hand].take() {
            self.world.items.push(crate::world::Item::new(x, y, data));
        }
        self.say(i, &text, false);
        sfx.push(Sfx::Plop);
    }

    /// The hands holding the tools that gathering `good` needs, one for each tool it asks
    /// for, or what's missing ("MISSING A TOOL: SHOVEL.").
    pub(crate) fn held_tools(&self, i: usize, good: &crate::data::GoodType) -> Result<Vec<usize>, String> {
        let hands = &self.monsters[i].hands;
        let mut used = Vec::new();
        for group in good.tools_required_tags.iter().filter(|g| !g.is_empty()) {
            let fits = |h: usize| hands[h].as_ref().is_some_and(|d| d.durability_left() > 0 && self.data.try_good(&d.good).is_some_and(|t| crate::data::matches(&t.type_tags, group)));
            match (0..2).find(|&h| !used.contains(&h) && fits(h)) {
                Some(h) => used.push(h),
                None => return Err(format!("MISSING A TOOL: {}.", self.describe_group(group))),
            }
        }
        Ok(used)
    }

    /// What Space would activate for monster `i` where it stands and faces, if anything (taking
    /// a wagon's lead isn't a thing: letting go records where it was led).
    pub(crate) fn thing_here(&self, i: usize) -> Option<Thing> {
        let m = &self.monsters[i];
        let (dx, dy) = m.facing.delta();
        let (here, ahead) = ((m.x, m.y), (m.x + dx, m.y + dy));
        if self.world.pumpkin_at(ahead.0, ahead.1).is_some() {
            return None;
        }
        if let Some(w) = self.world.wagon_at(ahead.0, ahead.1).filter(|&w| self.world.wagons[w].behind() == here) {
            return Some(Thing::Tailgate { wagon: w, open: !self.world.wagons[w].open });
        }
        if let Some(s) = self.world.shop_at_spot(here.0, here.1) {
            return Some(Thing::Shop(s));
        }
        self.world.plant_at(here.0, here.1).map(|_| Thing::Plant { at: here, facing: m.facing })
    }

    /// Where a monster stands to activate `t`, and which way it faces there (any way, if `None`).
    pub(crate) fn activation_spot(&self, t: Thing) -> Option<((i32, i32), Option<Dir>)> {
        match t {
            Thing::Plant { at, facing } => Some((at, Some(facing))),
            Thing::Shop(s) => self.world.shops.get(s).filter(|s| s.frontier.as_ref().is_none_or(|f| !f.fallen)).map(|s| (s.spot, None)),
            Thing::Tailgate { wagon, .. } => self.world.wagons.get(wagon).map(|w| {
                let back = w.behind();
                let d = DIRS.iter().copied().find(|d| (back.0 + d.delta().0, back.1 + d.delta().1) == w.at).unwrap_or(Dir::Up);
                (back, Some(d))
            }),
        }
    }

    fn toggle_record(&mut self, i: usize, sfx: &mut Vec<Sfx>) {
        let leading = self.led_wagon(i).is_some();
        let m = &mut self.monsters[i];
        if leading && m.rec.is_some() {
            self.say(i, "LET GO OF THE WAGON FIRST (SPACE).", true);
            sfx.push(Sfx::Deny);
            return;
        }
        let m = &mut self.monsters[i];
        if m.playing {
            self.say(i, "STOP MY LOOP FIRST (P).", true);
            sfx.push(Sfx::Deny);
            return;
        }
        let pos = (m.x, m.y);
        match m.rec.take() {
            None => {
                m.rec = Some(Routine { flag: pos, goals: Vec::new() });
                self.say(i, "RECORDING! R AGAIN TO FINISH.", false);
                sfx.push(Sfx::RecOn);
            }
            Some(r) if r.goals.is_empty() => self.say(i, "NOTHING RECORDED YET.", false),
            // Away from the flag: walk back to it, then save. The walk isn't part of the loop
            // (every goal brings its own walk); a loop just begins and ends at its flag.
            Some(r) if r.flag != pos => {
                m.rec = Some(r);
                m.homing = true;
                m.path.clear();
                self.say(i, "HEADING BACK TO MY FLAG.", false);
            }
            Some(r) => self.save_recording(i, r, sfx),
        }
    }

    /// One step of a monster walking itself back to its flag to finish recording. There, the
    /// recording is saved.
    fn walk_home(&mut self, i: usize, sfx: &mut Vec<Sfx>) {
        let Some(flag) = self.monsters[i].rec.as_ref().map(|r| r.flag) else {
            self.monsters[i].homing = false;
            return;
        };
        match self.go_toward(i, flag, None, sfx) {
            crate::motion::Going::Moving => {}
            crate::motion::Going::Arrived => {
                let m = &mut self.monsters[i];
                m.homing = false;
                let Some(r) = m.rec.take() else { return };
                self.save_recording(i, r, sfx);
            }
            crate::motion::Going::Stuck(_) => {
                self.monsters[i].homing = false;
                self.say(i, "I CAN'T FIND THE WAY BACK. WALK ME TO THE FLAG, THEN R.", true);
                sfx.push(Sfx::Deny);
            }
            // Someone in the way, most likely: keep at it.
            crate::motion::Going::Waiting(msg) => self.say(i, &msg, false),
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
        let resuming = m.paused_at == Some(pos);
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
        let frame = self.frame;
        let m = &mut self.monsters[i];
        let Some(flag) = m.routine.as_ref().map(|r| r.flag) else {
            self.say(i, "NO ROUTINE YET. R RECORDS ONE.", true);
            sfx.push(Sfx::Deny);
            return;
        };
        // A loop starts from its flag (or carries on from the goal it was at, resumed where it
        // was stopped).
        if !resuming {
            if pos != flag {
                self.say(i, "TAKE ME TO MY FLAG FIRST.", true);
                sfx.push(Sfx::Deny);
                return;
            }
            m.pc = 0;
        }
        m.playing = true;
        m.wait = 0;
        m.since = frame;
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

    /// Each commune's tutorial moves on. What a player does with their own monsters (making
    /// things, loops, helpers, leading a wagon) counts only for their commune; a result in the
    /// world (goods stored, a bridge built, a workshop set up) counts for everyone, so a newcomer
    /// skips steps someone else has already done.
    fn advance_tutorial(&mut self) {
        let events = std::mem::take(&mut self.events);
        let stored = |good: &str| self.world.bins.iter().any(|b| b.stock.iter().any(|d| d.good == good));
        for k in 0..self.communes.len() {
            let mine: Vec<&Ev> = events.iter().filter(|(c, _)| *c == k).map(|(_, e)| e).collect();
            let own = || self.monsters.iter().filter(move |m| m.commune == k);
            let c = &mut self.communes[k];
            for e in &mine {
                if let Ev::Made(g) = e {
                    c.made.insert(g.clone());
                }
            }
            c.tutorial.steps += mine.iter().filter(|e| ***e == Ev::Walked).count() as u32;
            let playing = own().filter(|m| m.playing).count();
            let has_routine = own().any(|m| m.routine.is_some());
            let led_wagon = self.world.wagons.iter().any(|w| w.last_leader.is_some_and(|id| own().any(|m| m.id == id)));
            loop {
                let made = |g: &str| c.made.contains(g);
                let met = match c.tutorial.stage {
                    0 => c.tutorial.steps >= 4,
                    1 => made("cotton_bolls"),
                    2 => stored("cotton_bolls") || made("cotton_slivers"),
                    3 => made("cotton_slivers"),
                    4 => made("thread"),
                    5 => made("fabric"),
                    6 => stored("fabric"),
                    7 => has_routine,
                    8 => playing >= 2,
                    9 => own().count() > 2,
                    10 => made("rope"),
                    11 => self.world.footbridge_built(),
                    12 => own().any(|m| area_name(m.x) == "TANGLEMERE"),
                    13 => made("lye"),
                    14 => made("quicklime"),
                    15 => made("indigo"),
                    16 => stored("dyed_fabric"),
                    17 => led_wagon,
                    18 => mine.contains(&&Ev::Unloaded),
                    19 => self.world.bridge_built(),
                    20 => made("kiln_kit") || made("workbench_kit"),
                    21 => self.world.shops.iter().any(|s| s.frontier.as_ref().is_some_and(|f| !f.fallen)),
                    _ => false,
                };
                if !met || c.tutorial.stage >= tutorial::DONE {
                    break;
                }
                c.tutorial.stage += 1;
            }
        }
    }

    /// Fabric of any kind in the cloth store: the HUD's running tally for now.
    pub fn cloth_stored(&self) -> usize {
        self.world.bins.iter().flat_map(|b| &b.stock).filter(|d| self.data.try_good(&d.good).is_some_and(|gt| gt.type_tags.iter().any(|t| t == "fabric"))).count()
    }

    /// The controlled monster's state for the HUD, and whether it is a warning.
    pub fn status(&self, i: usize) -> (String, bool) {
        let m = &self.monsters[i];
        let who = format!("{} THE {}", m.name, m.kind.to_uppercase());
        let state = if m.busy {
            ("WORKING".to_string(), false)
        } else if let Some(w) = self.led_wagon(i).filter(|_| !m.playing) {
            let slow = self.wagon_ticks(i, w) > PUSH_TICKS;
            (if slow { "LEADING THE WAGON. THE LOAD SLOWS IT".to_string() } else { "LEADING THE WAGON".to_string() }, false)
        } else if m.slide.is_some_and(|s| s.total == PULL_TICKS) {
            ("PULLING. HEAVY GOING".to_string(), false)
        } else if let Some(r) = &m.rec {
            (format!("RECORDING ({} GOALS){}", r.goals.len(), if m.homing { ". HEADING BACK TO THE FLAG" } else { "" }), false)
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

    /// Scripted play for tests and screenshots, as the first commune's player: `U D L R` move,
    /// `u` use, `r` record, `p` play, `t` tab, `x` cancel, `h` help. Waits for each action to
    /// finish.
    pub fn script(&mut self, keys: &str) {
        self.script_as(0, keys);
    }

    /// The same, as commune `k`'s player; the other players press nothing.
    pub fn script_as(&mut self, k: usize, keys: &str) {
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
                'c' => inp.sheet_pressed = true,
                _ => continue,
            }
            self.press(k, inp);
            for _ in 0..2000 {
                if self.monsters[self.communes[k].controlled].idle() {
                    break;
                }
                self.update(&[]);
            }
        }
    }

    /// One tick in which only commune `k`'s player presses anything.
    pub fn press(&mut self, k: usize, inp: Input) {
        let mut inputs = vec![Input::default(); k + 1];
        inputs[k] = inp;
        self.update(&inputs);
    }

    pub fn run_ticks(&mut self, n: u32) {
        for _ in 0..n {
            self.update(&[]);
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

#[cfg(test)]
#[path = "frontier_tests.rs"]
mod frontier_tests;

#[cfg(test)]
#[path = "commune_tests.rs"]
mod commune_tests;
