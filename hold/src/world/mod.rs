//! The rules of Naibihpma Hold v1: documents 01 to 05 played out tick by tick. Nothing here draws
//! or reads input; `game` shows it and the Elder's commands come in through `elder`.
//!
//! RockCollector's rules (attributes, skills, d20 rolls, conditions, behaviour lists, stock levels,
//! claiming, crafting, building from a tenth of health, hatching) with Majesty's rewards and purses
//! on top, the amphibian rules (Dry, day and night, husks, dreams) and the marsh's dangers.

pub mod econ;
pub mod elder;
pub mod fight;
pub mod marsh;
pub mod think;

use crate::data::*;
use crate::rng::Rng;
use crate::terrain::{dist, idx, on_map, places, Map, Tile, MH, MW};

pub type Id = u32;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Trait {
    Bold,
    Greedy,
    Steady,
    Timid,
}

impl Trait {
    pub fn name(self) -> &'static str {
        match self {
            Trait::Bold => "BOLD",
            Trait::Greedy => "GREEDY",
            Trait::Steady => "STEADY",
            Trait::Timid => "TIMID",
        }
    }
    /// Danger it will face for the same reward, in percent.
    pub fn nerve(self) -> i32 {
        match self {
            Trait::Bold => 160,
            Trait::Greedy | Trait::Steady => 100,
            Trait::Timid => 60,
        }
    }
    pub fn greed(self) -> i32 {
        if self == Trait::Greedy {
            150
        } else {
            100
        }
    }
}

/// A dream's vision (document 01).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Vision {
    Orb(Id),
    Raid(Id, u32),
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Src {
    Ground(usize),
    Stock(Id),
}

#[derive(Clone, PartialEq, Debug)]
pub enum Task {
    Idle { until: u32 },
    Wander { to: (i32, i32), until: u32 },
    Repair { b: Id },
    Deliver { item: Item, n: i32, src: Src, dst: Id, picked: bool },
    Harvest { s: usize },
    Plant { s: usize },
    Tend { s: usize, until: u32 },
    Craft { b: Id, item: Item, until: u32 },
    Equip { b: Id, item: Item, slot: Slot, pay: bool },
    Rest { b: Id },
    Fight { t: Id },
    Raid { b: Id },
    Flee { to: (i32, i32), until: u32 },
    Water { to: (i32, i32) },
    Reward { f: Id },
    Follow { t: Id, until: u32 },
    Learn { b: Id },
    DigIn { until: u32 },
    Dream { until: u32 },
    Moths { until: u32 },
    Patrol { to: (i32, i32), until: u32 },
    DigPuddle { p: (i32, i32) },
    Widen { p: (i32, i32) },
    CarryHusk { h: Id, dst: Id },
    PickUp { g: usize, dst: Id },
    Sleep,
    // enemies
    Roost,
    Hunt { t: Id },
    Steal { g: Option<usize>, h: Option<Id> },
    Drain { b: Option<Id>, s: Option<usize>, until: u32 },
    Guard,
    Gone,
}

#[derive(Clone, Debug)]
pub struct Creature {
    pub id: Id,
    pub kind: Kind,
    pub name: String,
    pub team: Team,
    pub pos: (i32, i32),
    /// The square it's stepping from, and how far through the step it is (of `step_len`).
    pub from: (i32, i32),
    pub step_t: i32,
    pub step_len: i32,
    pub path: Vec<(i32, i32)>,
    pub goal: Option<((i32, i32), i32)>,
    pub attrs: [i32; 6],
    pub level: i32,
    pub xp: i32,
    pub skills: [f32; 18],
    pub dmg: i32,
    pub conds: Vec<(Cond, f32, i32)>,
    pub gear: Vec<(Slot, Item)>,
    pub purse: i32,
    pub trait_: Trait,
    pub home: Option<Id>,
    pub task: Task,
    pub busy: i32,
    pub used: Vec<(Ability, u32)>,
    pub carry: Option<(Item, i32)>,
    pub dead: bool,
    pub born: u32,
    pub friend: Option<Id>,
    pub grudge: Option<Kind>,
    pub dreamt: Option<Vision>,
    pub orb: Option<Item>,
    pub drained: i32,
    pub swallowed: Option<(Id, u32)>,
    pub inside: Option<Id>,
    pub dug_in: bool,
    pub said: String,
    pub wave: bool,
    /// When a raider gives up and flies home.
    pub wave_until: u32,
    pub regrown: i32,
    pub last_hit: u32,
    pub hits: Vec<(Id, i32)>,
    pub thought: u32,
    pub learned: i32,
    pub swallowed_husk: Option<Husk>,
}

#[derive(Clone, Debug)]
pub struct Spot {
    pub kind: SpotKind,
    pub pos: (i32, i32),
    pub owner: Id,
    pub bonus: bool,
    /// 0 empty, 1 growing, 2 ripe, 3 gone (a building's own spot regrowing).
    pub state: u8,
    pub progress: i32,
    pub claim: Option<Id>,
    pub tender: Option<Id>,
    pub gone_until: u32,
    pub ripe_since: u32,
    /// Dew a wisp has drained from this fern.
    pub drained: i32,
}

#[derive(Clone, Debug)]
pub struct Building {
    pub id: Id,
    pub kind: BKind,
    pub team: Team,
    pub pos: (i32, i32),
    pub fp: (i32, i32),
    pub hp: i32,
    pub max_hp: i32,
    /// Waiting for its materials at the building it's built from.
    pub site: bool,
    pub src: Option<Id>,
    pub stock: Vec<(Item, i32)>,
    pub keeps: Vec<(Item, i32)>,
    pub res: Vec<(Item, i32)>,
    pub incoming: Vec<(Item, i32)>,
    pub repairers: Vec<Id>,
    pub crafter: Option<Id>,
    pub hatching: Option<(Kind, u32)>,
    pub hatch_res: Option<Kind>,
    pub auto_hatch: bool,
    pub routes: Vec<Id>,
    pub dead: bool,
    pub waves: Vec<(u32, i32)>,
    pub filled: bool,
    pub back_at: u32,
    pub grown: u32,
    pub curse_r: i32,
    pub hits: Vec<(Id, i32)>,
}

#[derive(Clone, Debug)]
pub struct Ground {
    pub pos: (i32, i32),
    pub item: Item,
    pub n: i32,
    pub since: u32,
    pub claim: Option<Id>,
    pub gone: bool,
}

#[derive(Clone, Debug)]
pub struct Husk {
    pub id: Id,
    pub pos: (i32, i32),
    pub was: Box<Creature>,
    pub since: u32,
    pub claim: Option<Id>,
    pub gone: bool,
    /// At the Star Pond, waiting to be regrown.
    pub home: bool,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum RKind {
    Attack,
    Explore,
    Guard,
    Fetch,
}

impl RKind {
    pub fn name(self) -> &'static str {
        match self {
            RKind::Attack => "ATTACK",
            RKind::Explore => "EXPLORE",
            RKind::Guard => "GUARD",
            RKind::Fetch => "FETCH",
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum RTarget {
    Creature(Id),
    Bldg(Id),
    Square((i32, i32)),
    Husk(Id),
    Ground(usize),
}

#[derive(Clone, Debug)]
pub struct Flag {
    pub id: Id,
    pub kind: RKind,
    pub target: RTarget,
    pub dew: i32,
    pub takers: Vec<Id>,
    pub posted: u32,
    pub until: u32,
    pub done: bool,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub struct Weather {
    pub rain: bool,
    pub dry: bool,
    pub mist: bool,
}

/// What happened this tick, for the event line, the log and the sound effects.
#[derive(Clone, Debug, PartialEq)]
pub enum Ev {
    Say(String),
    Hit,
    Bolt,
    Died(Kind, bool),
    Paid(i32),
    Built(BKind),
    Hatched(Kind),
    Raid(Id),
    Took(Id, Id),
    Refused,
    Won,
    Lost,
    Hint(u8),
    /// A blow, for drawing: from, to (squares), and 0 melee, 1 sling, 2 lightning, 3 pond light.
    Shot((i32, i32), (i32, i32), u8),
}

pub struct World {
    pub t: u32,
    pub rng: Rng,
    pub map: Map,
    pub cs: Vec<Creature>,
    pub bs: Vec<Building>,
    pub spots: Vec<Spot>,
    pub ground: Vec<Ground>,
    pub husks: Vec<Husk>,
    pub flags: Vec<Flag>,
    pub weather: Weather,
    pub weather_until: u32,
    pub spells_ready: [u32; 5],
    pub rain_at: Vec<((i32, i32), u32)>,
    pub starlight: Vec<((i32, i32), u32)>,
    pub ley: Vec<((i32, i32), u32)>,
    pub events: Vec<Ev>,
    pub log: Vec<(u32, String)>,
    pub next_id: Id,
    pub names_used: Vec<(Kind, usize)>,
    pub over: Option<bool>,
    pub stones_risen: usize,
    pub next_snake: u32,
    pub puddles_marked: Vec<(i32, i32)>,
    pub hints_given: u32,
    /// While the guided start is on, nothing hatches, raids or spreads.
    pub lesson: bool,
    pub newest_bld: (i32, i32),
    pub stats: Stats,
    /// Keep the whole event log (for the probe), not just the last 60 lines.
    pub keep_log: bool,
}

#[derive(Clone, Debug, Default)]
pub struct Stats {
    pub first: Vec<(String, u32)>,
    pub hall_income: Vec<i32>,
    pub deaths: i32,
    pub kills: i32,
}

impl Stats {
    pub fn note(&mut self, what: &str, t: u32) {
        if !self.first.iter().any(|f| f.0 == what) {
            self.first.push((what.to_string(), t));
        }
    }
    pub fn at(&self, what: &str) -> Option<u32> {
        self.first.iter().find(|f| f.0 == what).map(|f| f.1)
    }
}

pub fn mod_of(score: i32) -> i32 {
    score.div_euclid(2) - 5
}

pub fn day_tick(t: u32) -> u32 {
    t % DAY
}

pub fn is_night(t: u32) -> bool {
    day_tick(t) >= DUSK
}

pub fn clock(t: u32) -> String {
    let s = t / TPS;
    format!("{}:{:02}", s / 60, s % 60)
}

fn add(v: &mut Vec<(Item, i32)>, i: Item, n: i32) {
    if let Some(e) = v.iter_mut().find(|e| e.0 == i) {
        e.1 += n;
    } else if n != 0 {
        v.push((i, n));
    }
    v.retain(|e| e.1 != 0);
}

fn get(v: &[(Item, i32)], i: Item) -> i32 {
    v.iter().find(|e| e.0 == i).map_or(0, |e| e.1)
}

impl Building {
    pub fn done(&self) -> bool {
        !self.site && self.hp >= self.max_hp && !self.dead
    }
    pub fn standing(&self) -> bool {
        !self.dead
    }
    pub fn has(&self, i: Item) -> i32 {
        get(&self.stock, i)
    }
    pub fn add(&mut self, i: Item, n: i32) {
        add(&mut self.stock, i, n)
    }
    pub fn keep(&self, i: Item) -> i32 {
        get(&self.keeps, i)
    }
    pub fn set_keep(&mut self, i: Item, n: i32) {
        if let Some(e) = self.keeps.iter_mut().find(|e| e.0 == i) {
            e.1 = n;
        } else {
            self.keeps.push((i, n));
        }
    }
    pub fn reserved(&self, i: Item) -> i32 {
        get(&self.res, i)
    }
    pub fn reserve(&mut self, i: Item, n: i32) {
        add(&mut self.res, i, n)
    }
    pub fn coming(&self, i: Item) -> i32 {
        get(&self.incoming, i)
    }
    pub fn expect(&mut self, i: Item, n: i32) {
        add(&mut self.incoming, i, n)
    }
    pub fn centre(&self) -> (i32, i32) {
        (self.pos.0 + self.fp.0 / 2, self.pos.1 + self.fp.1 / 2)
    }
    /// The square in front of the building, where creatures go to use it.
    pub fn door(&self) -> (i32, i32) {
        (self.pos.0 + self.fp.0 / 2, self.pos.1 + self.fp.1)
    }
    pub fn info(&self) -> &'static BInfo {
        bld(self.kind)
    }
    /// Distance from a square to the nearest square of the footprint.
    pub fn dist_to(&self, p: (i32, i32)) -> i32 {
        let cx = p.0.clamp(self.pos.0, self.pos.0 + self.fp.0 - 1);
        let cy = p.1.clamp(self.pos.1, self.pos.1 + self.fp.1 - 1);
        dist(p, (cx, cy))
    }
}

impl Creature {
    pub fn info(&self) -> &'static KindInfo {
        kind(self.kind)
    }
    pub fn role(&self) -> Role {
        self.info().role
    }
    pub fn hero(&self) -> bool {
        self.role() == Role::Hero
    }
    pub fn folk(&self) -> bool {
        self.role() == Role::Folk
    }
    pub fn alive(&self) -> bool {
        !self.dead && self.inside.is_none()
    }
    pub fn stacks(&self, c: Cond) -> f32 {
        self.conds.iter().find(|e| e.0 == c).map_or(0.0, |e| e.1)
    }
    pub fn attr(&self, a: usize) -> i32 {
        let mut v = self.attrs[a];
        for (c, s, _) in &self.conds {
            v += cond(*c).attrs[a] * s.floor() as i32;
        }
        v
    }
    pub fn max_hp(&self) -> i32 {
        let (s, p) = self.info().hp;
        s + self.level * p
    }
    pub fn hp(&self) -> i32 {
        self.max_hp() - self.dmg
    }
    pub fn gear(&self, s: Slot) -> Option<Item> {
        self.gear.iter().find(|g| g.0 == s).map(|g| g.1)
    }
    pub fn has_shovel(&self) -> Option<Item> {
        self.gear(Slot::Shovel)
    }
    pub fn age(&self) -> u8 {
        if self.level < 3 {
            0
        } else if self.level < 7 {
            1
        } else {
            2
        }
    }
    pub fn enc(&self) -> i32 {
        10 + self.level
    }
    pub fn has_ability(&self, a: Ability) -> bool {
        self.info().abilities.iter().any(|&(ab, l)| ab == a && self.level >= l)
            && !(a == Ability::PondLight && self.kind == Kind::Axolotl && self.learned < 1)
            && !(a == Ability::Regrowth && self.kind == Kind::Axolotl && self.learned < 2)
    }
    pub fn ready(&self, a: Ability, t: u32) -> bool {
        let r = ability(a).recharge as u32;
        self.used.iter().find(|u| u.0 == a).is_none_or(|u| t >= u.1 + r)
    }
    pub fn dry_half(&self) -> bool {
        self.gear.iter().any(|g| item(g.1).dry_half)
    }
    /// The skill modifier: practice, attribute, items, conditions and the shell.
    pub fn skill(&self, s: Skill) -> i32 {
        let mut m = self.skills[s as usize].floor() as i32 + mod_of(self.attr(key_attr(s)));
        for (_, g) in &self.gear {
            let gi = item(*g);
            m += match s {
                Skill::Defense => gi.def,
                Skill::Dodge => gi.dodge,
                Skill::Resistance => gi.res,
                Skill::Energy => gi.energy,
                Skill::Healing => gi.healing,
                _ => 0,
            };
        }
        for (c, st, _) in &self.conds {
            let ci = cond(*c);
            let k = st.floor() as i32;
            m += match s {
                Skill::Defense => ci.def * k,
                Skill::Dodge => ci.dodge * k,
                _ => 0,
            };
        }
        if matches!(s, Skill::Defense | Skill::Dodge | Skill::Resistance) {
            m += self.info().shell;
        }
        m
    }
    pub fn move_ticks(&self) -> i32 {
        let mut t = self.info().walk;
        for (c, s, _) in &self.conds {
            let ci = cond(*c);
            if *c == Cond::Dry {
                t += (s.floor() as i32) / 2;
            } else {
                t += ci.move_ticks * s.floor().max(0.0) as i32;
            }
        }
        t.max(3)
    }
    /// Pixel position, between squares while stepping (integer only).
    pub fn px(&self) -> (i32, i32) {
        let sq = crate::terrain::SQ;
        if self.step_len <= 0 {
            return (self.pos.0 * sq, self.pos.1 * sq);
        }
        let f = self.step_t.clamp(0, self.step_len);
        (
            self.from.0 * sq + (self.pos.0 - self.from.0) * sq * f / self.step_len,
            self.from.1 * sq + (self.pos.1 - self.from.1) * sq * f / self.step_len,
        )
    }
}

impl World {
    pub fn new(seed: u64) -> World {
        let mut w = World {
            t: 0,
            rng: Rng::new(seed),
            map: Map::new(),
            cs: Vec::new(),
            bs: Vec::new(),
            spots: Vec::new(),
            ground: Vec::new(),
            husks: Vec::new(),
            flags: Vec::new(),
            weather: Weather::default(),
            weather_until: 0,
            spells_ready: [0; 5],
            rain_at: Vec::new(),
            starlight: Vec::new(),
            ley: Vec::new(),
            events: Vec::new(),
            log: Vec::new(),
            next_id: 1,
            names_used: Vec::new(),
            over: None,
            stones_risen: 0,
            next_snake: SNAKE_FROM_MIN * MINUTE,
            puddles_marked: Vec::new(),
            hints_given: 0,
            lesson: false,
            newest_bld: places::HALL,
            stats: Stats::default(),
            keep_log: false,
        };
        w.setup();
        w
    }

    fn new_id(&mut self) -> Id {
        self.next_id += 1;
        self.next_id
    }

    /// The first scenario's start (document 05).
    fn setup(&mut self) {
        use places::*;
        let hall = self.place_bld(BKind::Hall, HALL, true);
        let burrow = self.place_bld(BKind::Burrow, (HALL.0 + 5, HALL.1 - 5), true);
        let loft = self.place_bld(BKind::TwigLoft, (HALL.0 - 6, HALL.1 - 3), true);
        let chorus = self.place_bld(BKind::ChorusLog, (HALL.0 - 1, HALL.1 - 5), true);
        let toads = self.place_bld(BKind::ToadHall, (HALL.0 - 6, HALL.1 + 5), true);
        let _ = (loft, chorus);
        {
            let h = self.b_mut(hall);
            h.add(Item::Dew, 100);
            h.add(Item::Fungiwood, 6);
            h.add(Item::CapIngot, 3);
            h.add(Item::WoodenShovel, 4);
        }
        for i in 0..8 {
            let c = self.spawn(Kind::Newt, hall, Team::Hold);
            if i == 0 {
                self.c_mut(c).name = "TWEN".into();
            }
        }
        for _ in 0..2 {
            self.spawn(Kind::Caecilian, burrow, Team::Hold);
            self.spawn(Kind::TreeFrog, loft, Team::Hold);
        }
        self.spawn(Kind::ChorusToad, chorus, Team::Hold);
        for (name, tr) in [("PIP", Trait::Bold), ("BUFO", Trait::Timid)] {
            let c = self.spawn(Kind::Toad, toads, Team::Hold);
            let c = self.c_mut(c);
            c.name = name.into();
            c.trait_ = tr;
            c.gear.push((Slot::Sling, Item::Slingshot));
            c.purse = 10;
        }
        // The Hall's ferns are ripe at the start.
        for s in self.spots.iter_mut() {
            if s.owner == hall {
                s.state = 2;
            }
        }
        // The marsh's lairs.
        let lone = self.place_bld(BKind::LoneNest, LONE_NEST, true);
        // The cliffs north of the Hold, where its caves go, start explored.
        for y in 0..11 {
            for x in 0..32 {
                self.map.explored[idx(x, y)] = true;
            }
        }
        // The guided start shows the lone nest, so its ground starts explored.
        let lc = self.b(lone).centre();
        self.map.reveal(lc, 4);
        self.fill_lair(lone, 1);
        self.b_mut(lone).waves = vec![(13 * MINUTE, 1)];
        for p in NEAR_NESTS {
            let n = self.place_bld(BKind::Nest, p, true);
            self.fill_lair(n, 2);
            self.b_mut(n).waves = vec![(6 * MINUTE, 1), (16 * MINUTE, 1), (26 * MINUTE, 2), (36 * MINUTE, 2)];
        }
        for p in MIDDLE_NESTS {
            let n = self.place_bld(BKind::Nest, p, true);
            self.fill_lair(n, 3);
            self.b_mut(n).waves = vec![(11 * MINUTE, 1), (21 * MINUTE, 1), (31 * MINUTE, 2), (41 * MINUTE, 3)];
        }
        for p in FAR_NESTS {
            let n = self.place_bld(BKind::Nest, p, true);
            self.fill_lair(n, 4);
            self.b_mut(n).waves = vec![(18 * MINUTE, 1), (28 * MINUTE, 2), (38 * MINUTE, 3)];
        }
        let roost = self.place_bld(BKind::CrowRoost, ROOST, true);
        self.fill_lair(roost, 3);
        let hollow = self.place_bld(BKind::GorfHollow, HOLLOW, true);
        self.fill_lair(hollow, 5);
        let g = self.spawn(Kind::Gorf, hollow, Team::Gorf);
        {
            let g = self.c_mut(g);
            g.level = 10;
            g.name = "GORF".into();
            g.task = Task::Guard;
            g.inside = Some(hollow);
        }
        self.newest_bld = HALL;
        self.reveal_all_near();
    }

    pub fn fill_lair(&mut self, b: Id, n: i32) {
        let k = bld(self.b(b).kind).houses[0].0;
        let team = self.b(b).team;
        for _ in 0..n {
            self.spawn(k, b, team);
        }
    }

    pub fn place_bld(&mut self, k: BKind, pos: (i32, i32), built: bool) -> Id {
        let id = self.new_id();
        let info = bld(k);
        let mut b = Building {
            id,
            kind: k,
            team: info.team,
            pos,
            fp: info.fp,
            hp: if built { info.hp } else { 0 },
            max_hp: info.hp,
            site: !built,
            src: None,
            stock: Vec::new(),
            keeps: info.keeps.to_vec(),
            res: Vec::new(),
            incoming: Vec::new(),
            repairers: Vec::new(),
            crafter: None,
            hatching: None,
            hatch_res: None,
            auto_hatch: info.team == Team::Hold,
            routes: Vec::new(),
            dead: false,
            waves: Vec::new(),
            filled: false,
            back_at: 0,
            grown: 0,
            curse_r: 1,
            hits: Vec::new(),
        };
        // Workshops keep their own inputs (document 01's fix).
        for m in info.makes {
            for (inp, _) in item(*m).inputs {
                if b.keep(*inp) == 0 {
                    b.set_keep(*inp, 1);
                }
            }
        }
        for y in pos.1..pos.1 + info.fp.1 {
            for x in pos.0..pos.0 + info.fp.0 {
                if on_map(x, y) {
                    self.map.occ[idx(x, y)] = id + 1;
                }
            }
        }
        self.bs.push(b);
        // Its spots: in a row along the front, then down the sides.
        let mut slots = Vec::new();
        let (x0, y0, w, h) = (pos.0, pos.1, info.fp.0, info.fp.1);
        for x in x0 - 1..x0 + w + 1 {
            slots.push((x, y0 + h));
        }
        for y in (y0..y0 + h).rev() {
            slots.push((x0 - 1, y));
            slots.push((x0 + w, y));
        }
        for x in x0 - 1..x0 + w + 1 {
            slots.push((x, y0 - 1));
        }
        let door = (x0 + w / 2, y0 + h);
        slots.retain(|&p| p != door && on_map(p.0, p.1) && !self.map.blocks_walk(p.0, p.1));
        let mut kinds = Vec::new();
        for (k, n) in info.spots {
            for _ in 0..*n {
                kinds.push((*k, true));
            }
        }
        if let Some((k, n)) = info.plot {
            for _ in 0..n {
                kinds.push((k, false));
            }
        }
        for ((k, bonus), p) in kinds.into_iter().zip(slots) {
            self.spots.push(Spot {
                kind: k,
                pos: p,
                owner: id,
                bonus,
                state: if bonus { 1 } else { 0 },
                progress: 0,
                claim: None,
                tender: None,
                gone_until: 0,
                ripe_since: 0,
                drained: 0,
            });
        }
        if built {
            // Standing from the start: not a new building when it's mended later.
            if let Some(b) = self.bs.last_mut() {
                b.grown = 1;
            }
            if info.team == Team::Hold {
                self.newest_bld = pos;
            }
        }
        id
    }

    pub fn spawn(&mut self, k: Kind, home: Id, team: Team) -> Id {
        let id = self.new_id();
        let info = kind(k);
        let mut attrs = [0; 6];
        for (a, v) in attrs.iter_mut().enumerate() {
            *v = 8 + self.rng.r(1, 7) + info.attrs[a];
        }
        let name = self.pick_name(k);
        let b = self.b(home);
        let pos = self.free_near(b.door());
        let trait_ = match self.rng.r(0, 4) {
            0 => Trait::Bold,
            1 => Trait::Greedy,
            2 => Trait::Steady,
            _ => Trait::Timid,
        };
        let mut c = Creature {
            id,
            kind: k,
            name,
            team,
            pos,
            from: pos,
            step_t: 0,
            step_len: 0,
            path: Vec::new(),
            goal: None,
            attrs,
            level: 1,
            xp: 0,
            skills: [0.0; 18],
            dmg: 0,
            conds: Vec::new(),
            gear: Vec::new(),
            purse: 0,
            trait_,
            home: Some(home),
            task: Task::Idle { until: 0 },
            busy: self.rng.r(0, 30),
            used: Vec::new(),
            carry: None,
            dead: false,
            born: self.t,
            friend: None,
            grudge: None,
            dreamt: None,
            orb: None,
            drained: 0,
            swallowed: None,
            inside: None,
            dug_in: false,
            said: String::new(),
            wave: false,
            wave_until: 0,
            regrown: 0,
            last_hit: 0,
            hits: Vec::new(),
            thought: 0,
            learned: 0,
            swallowed_husk: None,
        };
        if k == Kind::Daot {
            c.skills[Skill::Enchanting as usize] = 8.0;
        }
        // What an enemy carries is rolled when it hatches, so a dream can see it (document 03).
        match k {
            Kind::Heron => {
                c.orb = match self.rng.r(0, 6) {
                    0 => Some(Item::SunOrb),
                    1 => Some(Item::ClargentOrb),
                    2 => Some(Item::MeteorOrb),
                    _ => None,
                }
            }
            Kind::Turtle => {
                c.orb = Some([Item::SunOrb, Item::ClargentOrb, Item::MeteorOrb][self.rng.r(0, 3) as usize]);
            }
            _ => {}
        }
        self.cs.push(c);
        id
    }

    fn pick_name(&mut self, k: Kind) -> String {
        let names = kind(k).names;
        if names.is_empty() {
            return kind(k).name.to_string();
        }
        let used = self.names_used.iter().filter(|u| u.0 == k).count();
        self.names_used.push((k, used));
        if used < names.len() {
            names[used].to_string()
        } else {
            format!("{} {}", names[used % names.len()], used / names.len() + 1)
        }
    }

    /// The nearest walkable square to `p` with nobody's building on it.
    pub fn free_near(&self, p: (i32, i32)) -> (i32, i32) {
        for r in 0..8 {
            for y in p.1 - r..=p.1 + r {
                for x in p.0 - r..=p.0 + r {
                    if on_map(x, y) && !self.map.blocks_walk(x, y) {
                        return (x, y);
                    }
                }
            }
        }
        p
    }

    // ---- lookups

    pub fn c(&self, id: Id) -> &Creature {
        self.cs.iter().find(|c| c.id == id).expect("creature")
    }
    pub fn c_mut(&mut self, id: Id) -> &mut Creature {
        self.cs.iter_mut().find(|c| c.id == id).expect("creature")
    }
    pub fn ci(&self, id: Id) -> Option<usize> {
        self.cs.iter().position(|c| c.id == id)
    }
    pub fn b(&self, id: Id) -> &Building {
        self.bs.iter().find(|b| b.id == id).expect("building")
    }
    pub fn b_mut(&mut self, id: Id) -> &mut Building {
        self.bs.iter_mut().find(|b| b.id == id).expect("building")
    }
    pub fn bi(&self, id: Id) -> Option<usize> {
        self.bs.iter().position(|b| b.id == id)
    }
    pub fn find_bld(&self, k: BKind) -> Option<Id> {
        self.bs.iter().find(|b| b.kind == k && !b.dead).map(|b| b.id)
    }
    pub fn has_done(&self, k: BKind) -> bool {
        self.bs.iter().any(|b| b.kind == k && b.done())
    }
    pub fn bld_at(&self, p: (i32, i32)) -> Option<Id> {
        if !on_map(p.0, p.1) {
            return None;
        }
        let o = self.map.occ[idx(p.0, p.1)];
        if o == 0 {
            None
        } else {
            Some(o - 1)
        }
    }
    pub fn hall(&self) -> Option<Id> {
        self.find_bld(BKind::Hall)
    }
    pub fn hold_count(&self) -> usize {
        self.bs.iter().filter(|b| b.team == Team::Hold && !b.dead).count()
    }

    pub fn say(&mut self, s: impl Into<String>) {
        let s = s.into();
        self.log.push((self.t, s.clone()));
        if self.log.len() > 60 && !self.keep_log {
            self.log.remove(0);
        }
        self.events.push(Ev::Say(s));
    }

    // ---- the tick

    pub fn tick(&mut self) {
        if self.over.is_some() {
            return;
        }
        self.t += 1;
        let t = self.t;
        if t % 15 == 0 {
            self.grow_spots();
            self.update_fog();
        }
        if !self.lesson {
            self.weather_tick();
            self.marsh_tick();
        }
        self.sites_and_hatching();
        self.flags_tick();
        // Creatures act: those due think, then everyone moves or works.
        let n = self.cs.len();
        for i in 0..n {
            if i >= self.cs.len() {
                break;
            }
            if self.cs[i].dead {
                continue;
            }
            self.conditions_tick(i);
            if self.cs[i].dead || self.cs[i].inside.is_some() {
                continue;
            }
            self.act(i);
        }
        self.cleanup();
        if t % MINUTE == 0 {
            self.stats.hall_income.push(0);
        }
        self.check_end();
    }

    fn check_end(&mut self) {
        if self.over.is_some() {
            return;
        }
        if !self.bs.iter().any(|b| b.kind == BKind::Hall && !b.dead) {
            self.over = Some(false);
            self.events.push(Ev::Lost);
            self.say("THE HALL HAS FALLEN.");
        } else if !self.bs.iter().any(|b| b.kind == BKind::GorfHollow && !b.dead) {
            self.over = Some(true);
            self.events.push(Ev::Won);
            self.say("GORF'S HOLLOW IS BROKEN. THE MARSH IS FREE.");
        }
    }

    fn cleanup(&mut self) {
        if self.t % 150 != 0 {
            return;
        }
        let dead: Vec<Id> = self.cs.iter().filter(|c| c.dead).map(|c| c.id).collect();
        if dead.is_empty() {
            return;
        }
        self.cs.retain(|c| !c.dead);
        for f in self.flags.iter_mut() {
            f.takers.retain(|t| !dead.contains(t));
        }
    }

    fn update_fog(&mut self) {
        for v in self.map.visible.iter_mut() {
            *v = false;
        }
        let mist = self.weather.mist;
        let pts: Vec<((i32, i32), i32)> = self
            .cs
            .iter()
            .filter(|c| c.team == Team::Hold && c.alive())
            .map(|c| (c.pos, if mist { c.info().sight / 2 } else { c.info().sight }))
            .chain(self.bs.iter().filter(|b| b.team == Team::Hold && !b.dead).map(|b| (b.centre(), 5 + b.fp.0 / 2)))
            .collect();
        for (p, r) in pts {
            self.map.reveal(p, r);
        }
        let stars: Vec<(i32, i32)> = self.starlight.iter().filter(|s| s.1 > self.t).map(|s| s.0).collect();
        for p in stars {
            self.map.reveal(p, spell(Spell::Starlight).radius);
        }
    }

    fn reveal_all_near(&mut self) {
        self.update_fog();
    }

    pub fn visible(&self, p: (i32, i32)) -> bool {
        on_map(p.0, p.1) && self.map.visible[idx(p.0, p.1)]
    }

    // ---- movement

    /// Take one step toward `to` (within `adj`); true when there.
    pub fn approach(&mut self, i: usize, to: (i32, i32), adj: i32) -> bool {
        let (pos, flies) = (self.cs[i].pos, self.cs[i].info().flies);
        if dist(pos, to) <= adj {
            self.cs[i].path.clear();
            // Arrived, but someone who got here first is standing on this square: take a free
            // square next to it that still does the job (Henry: overlap on the move, not at work).
            let settled = self.cs[i].step_t >= self.cs[i].step_len;
            if adj >= 1 && settled && self.crowded_by_earlier(i, pos) {
                if let Some(n) = self.free_square_near(i, pos, to, adj) {
                    let ticks = self.cs[i].move_ticks();
                    let c = &mut self.cs[i];
                    c.from = c.pos;
                    c.pos = n;
                    c.step_t = 0;
                    c.step_len = ticks;
                    c.busy = ticks * 10;
                    return false;
                }
            }
            return true;
        }
        let need = match self.cs[i].goal {
            Some((g, a)) => g != to || a != adj || self.cs[i].path.is_empty(),
            None => true,
        };
        if need {
            let path = self.map.find_path(pos, to, adj, flies, 6000).unwrap_or_default();
            self.cs[i].path = path;
            self.cs[i].goal = Some((to, adj));
            if self.cs[i].path.is_empty() {
                // Nowhere to go: give up on this.
                self.cs[i].busy = 30;
                return dist(pos, to) <= adj + 1;
            }
        }
        let next = self.cs[i].path.remove(0);
        if !flies && self.map.blocks_walk(next.0, next.1) && dist(next, to) > adj {
            self.cs[i].path.clear();
            self.cs[i].busy = 10;
            return false;
        }
        let ticks = if flies { self.cs[i].move_ticks() } else { self.cs[i].move_ticks() * self.map.step_cost(next.0, next.1) / 10 };
        let c = &mut self.cs[i];
        c.from = c.pos;
        c.pos = next;
        c.step_t = 0;
        c.step_len = ticks;
        c.busy = ticks * 10;
        false
    }

    /// Another creature, there before this one (a lower id), standing on this square.
    fn crowded_by_earlier(&self, i: usize, p: (i32, i32)) -> bool {
        let me = self.cs[i].id;
        self.cs.iter().any(|o| o.id < me && o.alive() && o.pos == p && o.step_t >= o.step_len)
    }

    fn occupied(&self, i: usize, p: (i32, i32)) -> bool {
        let me = self.cs[i].id;
        self.cs.iter().any(|o| o.id != me && o.alive() && o.pos == p)
    }

    /// A free square next to `p`, still within `adj` of `to`, nearest `to` first.
    fn free_square_near(&self, i: usize, p: (i32, i32), to: (i32, i32), adj: i32) -> Option<(i32, i32)> {
        let flies = self.cs[i].info().flies;
        let mut best: Option<(i32, (i32, i32))> = None;
        for r in 1..=2 {
            for dy in -r..=r {
                for dx in -r..=r {
                    let n = (p.0 + dx, p.1 + dy);
                    if dist(n, p) != r || !on_map(n.0, n.1) || dist(n, to) > adj {
                        continue;
                    }
                    if !flies && self.map.blocks_walk(n.0, n.1) || self.occupied(i, n) {
                        continue;
                    }
                    let score = r * 10 + dist(n, to);
                    if best.is_none_or(|b| score < b.0) {
                        best = Some((score, n));
                    }
                }
            }
            if best.is_some() {
                break;
            }
        }
        best.map(|b| b.1)
    }

    // ---- the per-creature loop

    fn act(&mut self, i: usize) {
        {
            let c = &mut self.cs[i];
            if c.step_t < c.step_len {
                c.step_t += 1;
            }
            if c.busy > 0 {
                // Altered acts a little slower; nothing else changes action speed.
                let slow = (c.stacks(Cond::Altered).floor() as i32).clamp(0, 5);
                c.busy -= 10 - slow;
                return;
            }
        }
        if self.cs[i].stacks(Cond::Dreaming) >= 1.0 {
            self.cs[i].busy = 30;
            return;
        }
        self.think::<false>(i);
        self.perform(i);
    }

    // ---- items on the ground

    pub fn drop_item(&mut self, pos: (i32, i32), it: Item, n: i32) {
        if n <= 0 {
            return;
        }
        if let Some(g) = self.ground.iter_mut().find(|g| !g.gone && g.pos == pos && g.item == it && g.claim.is_none()) {
            g.n += n;
            return;
        }
        if let Some(slot) = self.ground.iter().position(|g| g.gone) {
            self.ground[slot] = Ground { pos, item: it, n, since: self.t, claim: None, gone: false };
        } else {
            self.ground.push(Ground { pos, item: it, n, since: self.t, claim: None, gone: false });
        }
    }

    pub fn hall_income(&mut self, n: i32) {
        if let Some(last) = self.stats.hall_income.last_mut() {
            *last += n;
        } else {
            self.stats.hall_income.push(n);
        }
    }

    // ---- what the panel says

    pub fn doing(&self, c: &Creature) -> String {
        if !c.said.is_empty() {
            return c.said.clone();
        }
        "IDLE".into()
    }

    /// Enemies of a team.
    pub fn hostile(a: Team, b: Team) -> bool {
        a != b
    }

    pub fn squares_seen(&self) -> usize {
        self.map.explored.iter().filter(|e| **e).count()
    }

    /// For the minimap and tests: every Hold building that stands.
    pub fn hold_blds(&self) -> impl Iterator<Item = &Building> {
        self.bs.iter().filter(|b| b.team == Team::Hold && !b.dead)
    }

    pub fn map_size() -> (i32, i32) {
        (MW, MH)
    }
}
