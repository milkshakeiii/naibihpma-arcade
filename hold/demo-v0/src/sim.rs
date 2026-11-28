//! The village and the marsh, one 15 FPS tick at a time.
//!
//! You are the Elder of Naibihpma Hold and can't give orders. Dew is the only lever: you spend it
//! on buildings, on hiring heroes, and on rewards posted against things in the marsh. Each hero
//! weighs a reward against the danger, their own nerve and how many others are already going,
//! and decides for themselves. Herons guard their nests and raid the hold; later Gorf's curse
//! takes root and sends wisps after the dew stills.
//!
//! Positions are integer pixels in the 400 x 240 play area; a unit's (x, y) is where its feet are.

use crate::map::{Map, Tile, MH, MW, TILE};
use crate::rng::Rng;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Kind {
    Newt,
    Toad,
    Heron,
    Wisp,
}

impl Kind {
    pub fn hero(self) -> bool {
        matches!(self, Kind::Newt | Kind::Toad)
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Trait {
    Brave,
    Greedy,
    Steady,
    Timid,
}

impl Trait {
    /// How much danger they'll face for the same reward, in percent.
    fn nerve(self) -> i32 {
        match self {
            Trait::Brave => 160,
            Trait::Greedy | Trait::Steady => 100,
            Trait::Timid => 60,
        }
    }
    /// How much a reward is worth to them, in percent.
    fn greed(self) -> i32 {
        if self == Trait::Greedy {
            150
        } else {
            100
        }
    }
    /// Below this share of health they go to the Star Pond, in percent.
    fn flee(self) -> i32 {
        match self {
            Trait::Brave => 30,
            Trait::Greedy | Trait::Steady => 45,
            Trait::Timid => 60,
        }
    }
    pub fn name(self) -> &'static str {
        match self {
            Trait::Brave => "BRAVE",
            Trait::Greedy => "GREEDY",
            Trait::Steady => "STEADY",
            Trait::Timid => "TIMID",
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum BKind {
    Hall,
    Lodge,
    ToadHall,
    Still,
    Post,
    Nest,
    Glade,
}

impl BKind {
    pub fn cost(self) -> i32 {
        match self {
            BKind::Lodge => 40,
            BKind::ToadHall => 60,
            BKind::Still => 30,
            BKind::Post => 35,
            _ => 0,
        }
    }
    pub fn max_hp(self) -> i32 {
        match self {
            BKind::Hall => 150,
            BKind::Lodge => 70,
            BKind::ToadHall => 80,
            BKind::Still => 45,
            BKind::Post => 60,
            BKind::Nest => 220,
            BKind::Glade => 90,
        }
    }
    pub fn name(self) -> &'static str {
        match self {
            BKind::Hall => "ELDER'S HALL",
            BKind::Lodge => "NEWT LODGE",
            BKind::ToadHall => "TOAD HALL",
            BKind::Still => "DEW STILL",
            BKind::Post => "WATCH POST",
            BKind::Nest => "HERON NEST",
            BKind::Glade => "GORF'S STONE",
        }
    }
    pub fn enemy(self) -> bool {
        matches!(self, BKind::Nest | BKind::Glade)
    }
}

/// What something is aimed at.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Tgt {
    Unit(u32),
    Bldg(u32),
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Job {
    Idle,
    Wander(i32, i32),
    Bounty(u32),
    Fight(u32),
    Defend(u32),
    Heal,
    Rest,
    // herons
    Guard,
    Raid(u32),
    Return,
    Leave,
    // wisps
    Drift,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Bubble {
    None,
    Yes,
    No,
    Dew,
    Heart,
    Home,
    Zz,
}

pub struct Unit {
    pub id: u32,
    pub kind: Kind,
    pub name: &'static str,
    pub trait_: Trait,
    pub x: i32,
    pub y: i32,
    pub hp: i32,
    pub max_hp: i32,
    pub level: i32,
    pub xp: i32,
    pub purse: i32,
    /// The building a hero lives at, or the nest or stone a creature came from.
    pub home: u32,
    pub job: Job,
    pub path: Vec<(i32, i32)>,
    pub path_to: (i32, i32),
    pub cool: i32,
    pub think: i32,
    pub left: bool,
    pub moving: bool,
    pub bubble: Bubble,
    pub bubble_t: i32,
    pub hit_by: Option<u32>,
    pub hit_t: i32,
    pub hurt_t: i32,
    /// Rewards already judged, at the amount they were judged at.
    pub seen: Vec<(u32, i32)>,
    /// What they last asked for, when they turned a reward down.
    pub wants: i32,
    pub dead: bool,
    pub timer: i32,
    /// Ticks spent waiting at the bridge for company.
    pub wait: i32,
}

pub struct Bldg {
    pub id: u32,
    pub kind: BKind,
    pub tx: i32,
    pub ty: i32,
    pub hp: i32,
    pub max_hp: i32,
    /// Construction progress, 0 to 100.
    pub built: i32,
    pub alive: bool,
    pub timer: i32,
    /// The tutorial's nest: weaker, and quiet until the lesson is over.
    pub weak: bool,
}

impl Bldg {
    /// Footprint in pixels: x, y, w, h.
    pub fn rect(&self) -> (i32, i32, i32, i32) {
        (self.tx * TILE, self.ty * TILE, 2 * TILE, 2 * TILE)
    }
    pub fn center(&self) -> (i32, i32) {
        (self.tx * TILE + TILE, self.ty * TILE + TILE)
    }
    pub fn ready(&self) -> bool {
        self.alive && self.built >= 100
    }
}

pub struct Flag {
    pub id: u32,
    pub tgt: Tgt,
    pub reward: i32,
    pub takers: Vec<u32>,
    pub hitters: Vec<u32>,
}

#[derive(Clone, Debug, PartialEq)]
pub enum Ev {
    Accept {
        hero: u32,
        flag: u32,
    },
    Refuse {
        hero: u32,
        flag: u32,
        wants: i32,
    },
    Paid {
        hero: u32,
        amount: i32,
    },
    Hit {
        x: i32,
        y: i32,
        by_hero: bool,
    },
    Bolt {
        x0: i32,
        y0: i32,
        x1: i32,
        y1: i32,
    },
    Died {
        id: u32,
        kind: Kind,
    },
    Fell {
        id: u32,
    },
    Down {
        id: u32,
        kind: BKind,
    },
    Raid {
        heron: u32,
    },
    Built {
        id: u32,
    },
    Hired {
        id: u32,
    },
    Healing {
        id: u32,
    },
    LevelUp {
        id: u32,
    },
    Curse,
    /// A reward lapsed because its target got away; the dew is free again.
    Lapsed {
        flag: u32,
    },
    Won,
    Lost,
}

const NEWT_NAMES: [&str; 6] = ["PIP", "WREN", "FEN", "SEDGE", "RIPPLE", "TANSY"];
const TOAD_NAMES: [&str; 6] = ["BUFO", "MOTT", "GRISTLE", "UMBER", "BRACK", "OLD LURK"];
pub const HIRE_NEWT: i32 = 15;
pub const HIRE_TOAD: i32 = 25;
pub const START_DEW: i32 = 60;
pub const REWARD_STEP: i32 = 10;
pub const REWARD_FIRST: i32 = 20;
pub const MAX_FLAGS: usize = 4;
/// Dew a hero asks per unit of danger.
const PRICE: i32 = 20;
/// Ticks after the lesson before Gorf's curse takes root (eighty seconds).
const CURSE_AFTER: i32 = 1200;

pub struct World {
    pub map: Map,
    pub units: Vec<Unit>,
    pub bldgs: Vec<Bldg>,
    pub flags: Vec<Flag>,
    pub graves: Vec<(i32, i32, i32)>,
    pub dew: i32,
    pub tick: u32,
    pub rng: Rng,
    pub events: Vec<Ev>,
    next_id: u32,
    /// While the lesson is on, nothing hatches, raids or spreads.
    pub calm: bool,
    pub calm_ended: u32,
    raid_t: i32,
    pub over: Option<bool>,
    /// Gorf's stone has risen (it must be broken to win).
    pub cursed: bool,
    pub paid_total: i32,
    pub fallen: i32,
    newts_named: usize,
    toads_named: usize,
}

impl World {
    pub fn new(seed: u64) -> World {
        let mut w = World {
            map: Map::new(),
            units: vec![],
            bldgs: vec![],
            flags: vec![],
            graves: vec![],
            dew: START_DEW,
            tick: 0,
            rng: Rng::new(seed),
            events: vec![],
            next_id: 1,
            calm: true,
            calm_ended: 0,
            raid_t: 0,
            over: None,
            cursed: false,
            paid_total: 0,
            fallen: 0,
            newts_named: 0,
            toads_named: 0,
        };
        w.add_bldg(BKind::Hall, 6, 6, true);
        let lodge = w.add_bldg(BKind::Lodge, 1, 4, true);
        w.add_bldg(BKind::ToadHall, 7, 9, true);
        w.add_bldg(BKind::Still, 1, 8, true);
        let n1 = w.add_bldg(BKind::Nest, 14, 1, true);
        w.bldg_mut(n1).unwrap().weak = true;
        w.bldg_mut(n1).unwrap().hp = 30;
        w.bldg_mut(n1).unwrap().max_hp = 30;
        w.add_bldg(BKind::Nest, 21, 5, true);
        w.add_bldg(BKind::Nest, 17, 12, true);
        let pip = w.spawn_hero(Kind::Newt, lodge);
        w.unit_mut(pip).unwrap().trait_ = Trait::Brave;
        w
    }

    fn id(&mut self) -> u32 {
        self.next_id += 1;
        self.next_id
    }

    pub fn add_bldg(&mut self, kind: BKind, tx: i32, ty: i32, built: bool) -> u32 {
        let id = self.id();
        self.map.occupy(tx, ty, 2, 2, true);
        let max = kind.max_hp();
        self.bldgs.push(Bldg {
            id,
            kind,
            tx,
            ty,
            hp: if built { max } else { max / 5 },
            max_hp: max,
            built: if built { 100 } else { 0 },
            alive: true,
            timer: 0,
            weak: false,
        });
        id
    }

    pub fn unit(&self, id: u32) -> Option<&Unit> {
        self.units.iter().find(|u| u.id == id && !u.dead)
    }
    pub fn unit_mut(&mut self, id: u32) -> Option<&mut Unit> {
        self.units.iter_mut().find(|u| u.id == id && !u.dead)
    }
    pub fn bldg(&self, id: u32) -> Option<&Bldg> {
        self.bldgs.iter().find(|b| b.id == id && b.alive)
    }
    pub fn bldg_mut(&mut self, id: u32) -> Option<&mut Bldg> {
        self.bldgs.iter_mut().find(|b| b.id == id && b.alive)
    }
    pub fn flag(&self, id: u32) -> Option<&Flag> {
        self.flags.iter().find(|f| f.id == id)
    }
    pub fn flag_on(&self, t: Tgt) -> Option<&Flag> {
        self.flags.iter().find(|f| f.tgt == t)
    }
    pub fn alive(&self, t: Tgt) -> bool {
        match t {
            Tgt::Unit(id) => self.unit(id).is_some(),
            Tgt::Bldg(id) => self.bldg(id).is_some(),
        }
    }
    /// Nests and Gorf's stone still standing: all must fall to win.
    pub fn foes_left(&self) -> usize {
        self.bldgs
            .iter()
            .filter(|b| b.alive && b.kind.enemy())
            .count()
    }
    pub fn glade_up(&self) -> bool {
        self.bldgs.iter().any(|b| b.alive && b.kind == BKind::Glade)
    }
    pub fn nests_left(&self) -> usize {
        self.bldgs
            .iter()
            .filter(|b| b.alive && b.kind == BKind::Nest)
            .count()
    }
    /// Dew committed to standing rewards.
    pub fn reserved(&self) -> i32 {
        self.flags.iter().map(|f| f.reward).sum()
    }

    /// Where a target stands: a point for a unit, the footprint centre for a building.
    pub fn tgt_pos(&self, t: Tgt) -> Option<(i32, i32)> {
        match t {
            Tgt::Unit(id) => self.unit(id).map(|u| (u.x, u.y)),
            Tgt::Bldg(id) => self.bldg(id).map(|b| b.center()),
        }
    }

    /// Distance from (x, y) to a target, in pixels (Chebyshev, to the edge of a footprint).
    fn reach(&self, x: i32, y: i32, t: Tgt) -> i32 {
        match t {
            Tgt::Unit(id) => self
                .unit(id)
                .map_or(9999, |u| (u.x - x).abs().max((u.y - 6 - (y - 6)).abs())),
            Tgt::Bldg(id) => self.bldg(id).map_or(9999, |b| {
                let (bx, by, bw, bh) = b.rect();
                let dx = (bx - x).max(x - (bx + bw - 1)).max(0);
                let dy = (by - y).max(y - (by + bh - 1)).max(0);
                dx.max(dy)
            }),
        }
    }

    // ---- heroes --------------------------------------------------------------------------------

    pub fn spawn_hero(&mut self, kind: Kind, home: u32) -> u32 {
        let id = self.id();
        let (name, trait_) = match kind {
            Kind::Newt => {
                let n = NEWT_NAMES[self.newts_named % NEWT_NAMES.len()];
                self.newts_named += 1;
                (n, Trait::Steady)
            }
            _ => {
                let n = TOAD_NAMES[self.toads_named % TOAD_NAMES.len()];
                self.toads_named += 1;
                (n, Trait::Timid)
            }
        };
        // After the first of each kind, traits are dealt at random.
        let trait_ = if (kind == Kind::Newt && self.newts_named > 1)
            || (kind == Kind::Toad && self.toads_named > 1)
        {
            [Trait::Brave, Trait::Greedy, Trait::Steady, Trait::Timid][self.rng.below(4) as usize]
        } else {
            trait_
        };
        let (x, y) = self
            .bldg(home)
            .map_or((80, 80), |b| (b.tx * TILE + TILE, (b.ty + 2) * TILE + 4));
        let max = base_hp(kind);
        self.units.push(Unit {
            id,
            kind,
            name,
            trait_,
            x,
            y,
            hp: max,
            max_hp: max,
            level: 1,
            xp: 0,
            purse: 0,
            home,
            job: Job::Idle,
            path: vec![],
            path_to: (-1, -1),
            cool: 0,
            think: (id % 10) as i32,
            left: false,
            moving: false,
            bubble: Bubble::None,
            bubble_t: 0,
            hit_by: None,
            hit_t: 0,
            hurt_t: 0,
            seen: vec![],
            wants: 0,
            dead: false,
            timer: 0,
            wait: 0,
        });
        id
    }

    pub fn power(&self, u: &Unit) -> i32 {
        let base = match u.kind {
            Kind::Newt => 16 + 5 * (u.level - 1),
            Kind::Toad => 20 + 6 * (u.level - 1),
            Kind::Heron => 14,
            Kind::Wisp => 5,
        };
        base * u.hp.max(1) / u.max_hp.max(1)
    }

    /// How dangerous a target looks.
    pub fn danger(&self, t: Tgt) -> i32 {
        match t {
            Tgt::Unit(id) => self.unit(id).map_or(0, |u| self.power(u).max(4)),
            Tgt::Bldg(id) => match self.bldg(id) {
                None => 0,
                Some(b) => {
                    let guards = self
                        .units
                        .iter()
                        .filter(|u| !u.dead && u.home == b.id && !u.kind.hero())
                        .count() as i32;
                    match b.kind {
                        BKind::Nest if b.weak => 14,
                        BKind::Nest => 20 + 10 * guards,
                        BKind::Glade => 36 + 6 * guards,
                        _ => 10,
                    }
                }
            },
        }
    }

    /// The least a hero would accept to go after this reward's target, given who's already going.
    pub fn asking(&self, hero: &Unit, f: &Flag) -> i32 {
        let allies: i32 = f
            .takers
            .iter()
            .filter(|&&id| id != hero.id)
            .filter_map(|&id| self.unit(id))
            .map(|u| self.power(u))
            .sum();
        let me = self.power(hero).max(1);
        let risk = self.danger(f.tgt) * 100 / (me + allies * 3 / 4).max(1);
        let tiles = self.tgt_pos(f.tgt).map_or(0, |(x, y)| {
            ((x - hero.x).abs().max((y - hero.y).abs())) / TILE
        });
        PRICE * risk / hero.trait_.nerve() + tiles / 4
    }

    fn accepts(&self, hero: &Unit, f: &Flag) -> bool {
        f.reward * hero.trait_.greed() / 100 >= self.asking(hero, f)
    }

    // ---- the player's levers -------------------------------------------------------------------

    pub fn can_hire(&self, at: u32) -> Option<i32> {
        let b = self.bldg(at)?;
        if !b.ready() {
            return None;
        }
        let (kind, cost) = match b.kind {
            BKind::Lodge => (Kind::Newt, HIRE_NEWT),
            BKind::ToadHall => (Kind::Toad, HIRE_TOAD),
            _ => return None,
        };
        let living = self
            .units
            .iter()
            .filter(|u| !u.dead && u.home == at && u.kind == kind)
            .count();
        if living >= 3 {
            return None;
        }
        Some(cost)
    }

    pub fn hire(&mut self, at: u32) -> Option<u32> {
        let cost = self.can_hire(at)?;
        if self.dew - self.reserved() < cost {
            return None;
        }
        let kind = if self.bldg(at)?.kind == BKind::Lodge {
            Kind::Newt
        } else {
            Kind::Toad
        };
        self.dew -= cost;
        let id = self.spawn_hero(kind, at);
        self.events.push(Ev::Hired { id });
        Some(id)
    }

    pub fn build(&mut self, kind: BKind, tx: i32, ty: i32) -> Option<u32> {
        if !self.map.buildable(tx, ty) || self.dew - self.reserved() < kind.cost() {
            return None;
        }
        self.dew -= kind.cost();
        Some(self.add_bldg(kind, tx, ty, false))
    }

    /// Can a reward go on this? Enemies only.
    pub fn rewardable(&self, t: Tgt) -> bool {
        match t {
            Tgt::Unit(id) => self.unit(id).is_some_and(|u| !u.kind.hero()),
            Tgt::Bldg(id) => self.bldg(id).is_some_and(|b| b.kind.enemy()),
        }
    }

    pub fn post(&mut self, t: Tgt) -> Option<u32> {
        if !self.rewardable(t) || self.flag_on(t).is_some() || self.flags.len() >= MAX_FLAGS {
            return None;
        }
        if self.dew - self.reserved() < REWARD_FIRST {
            return None;
        }
        let id = self.id();
        self.flags.push(Flag {
            id,
            tgt: t,
            reward: REWARD_FIRST,
            takers: vec![],
            hitters: vec![],
        });
        Some(id)
    }

    pub fn raise(&mut self, flag: u32) -> bool {
        let free = self.dew - self.reserved();
        match self.flags.iter_mut().find(|f| f.id == flag) {
            Some(f) if free >= REWARD_STEP && f.reward < 200 => {
                f.reward += REWARD_STEP;
                true
            }
            _ => false,
        }
    }

    pub fn lower(&mut self, flag: u32) -> bool {
        match self.flags.iter_mut().find(|f| f.id == flag) {
            Some(f) if f.reward > REWARD_STEP => {
                f.reward -= REWARD_STEP;
                true
            }
            _ => false,
        }
    }

    pub fn cancel(&mut self, flag: u32) {
        self.flags.retain(|f| f.id != flag);
        for u in self.units.iter_mut() {
            if u.job == Job::Bounty(flag) {
                u.job = Job::Idle;
                u.path.clear();
            }
        }
    }

    // ---- the tick ------------------------------------------------------------------------------

    pub fn end_calm(&mut self) {
        if self.calm {
            self.calm = false;
            self.calm_ended = self.tick;
            self.raid_t = 600;
            for b in self.bldgs.iter_mut() {
                if b.kind == BKind::Nest {
                    b.timer = 60 + (b.tx * 7) % 90;
                }
            }
        }
    }

    pub fn step(&mut self) {
        if self.over.is_some() {
            return;
        }
        self.tick += 1;
        self.economy();
        self.buildings();
        if !self.calm {
            self.raids();
            if self.tick == self.calm_ended + CURSE_AFTER as u32 && !self.cursed {
                self.curse();
            }
        }
        for i in 0..self.units.len() {
            if !self.units[i].dead {
                self.unit_tick(i);
            }
        }
        self.units.retain(|u| !u.dead);
        self.graves.iter_mut().for_each(|g| g.2 -= 1);
        self.graves.retain(|g| g.2 > 0);
        if self.nests_left() == 0 && !self.cursed && !self.calm {
            self.curse();
        }
        if self.foes_left() == 0 && self.cursed {
            self.over = Some(true);
            self.events.push(Ev::Won);
        } else if !self.bldgs.iter().any(|b| b.alive && b.kind == BKind::Hall) {
            self.over = Some(false);
            self.events.push(Ev::Lost);
        }
    }

    fn economy(&mut self) {
        let t = self.tick as i32;
        if t % 40 == 0 {
            self.dew += 1;
        }
        let wisps: Vec<(i32, i32)> = self
            .units
            .iter()
            .filter(|u| u.kind == Kind::Wisp && !u.dead)
            .map(|u| (u.x, u.y))
            .collect();
        let mut gain = 0;
        for b in self
            .bldgs
            .iter()
            .filter(|b| b.ready() && b.kind == BKind::Still)
        {
            let (cx, cy) = b.center();
            let drained = wisps
                .iter()
                .any(|&(x, y)| (x - cx).abs() < 28 && (y - cy).abs() < 28);
            if !drained && (t + b.tx) % 20 == 0 {
                gain += 1;
            }
        }
        self.dew += gain;
    }

    fn buildings(&mut self) {
        let enemies: Vec<(u32, i32, i32)> = self
            .units
            .iter()
            .filter(|u| !u.dead && !u.kind.hero())
            .map(|u| (u.id, u.x, u.y))
            .collect();
        let mut spawns = vec![];
        let mut shots = vec![];
        let calm = self.calm;
        let late = !calm && self.tick > self.calm_ended + 1350;
        for b in self.bldgs.iter_mut().filter(|b| b.alive) {
            if b.built < 100 {
                b.built += 1;
                b.hp = (b.max_hp * b.built / 100).max(b.hp);
                if b.built == 100 {
                    self.events.push(Ev::Built { id: b.id });
                }
                continue;
            }
            let (cx, cy) = b.center();
            let threatened = enemies
                .iter()
                .any(|&(_, x, y)| (x - cx).abs() < 64 && (y - cy).abs() < 64);
            if !b.kind.enemy() && !threatened && b.hp < b.max_hp && self.tick % 20 == 0 {
                b.hp += 1;
            }
            b.timer -= 1;
            match b.kind {
                BKind::Nest if !calm && b.timer <= 0 => {
                    b.timer = 400;
                    spawns.push((Kind::Heron, b.id, if late { 2 } else { 1 }));
                }
                BKind::Glade if b.timer <= 0 => {
                    b.timer = 360;
                    spawns.push((Kind::Wisp, b.id, 3));
                }
                BKind::Post if b.timer <= 0 => {
                    if let Some(&(id, x, y)) = enemies
                        .iter()
                        .filter(|&&(_, x, y)| (x - cx).abs() <= 56 && (y - cy).abs() <= 56)
                        .min_by_key(|&&(_, x, y)| (x - cx).abs() + (y - cy).abs())
                    {
                        b.timer = 20;
                        shots.push((b.id, id, cx, cy - 12, x, y));
                    }
                }
                _ => {}
            }
        }
        for (kind, home, max) in spawns {
            let living = self
                .units
                .iter()
                .filter(|u| !u.dead && u.home == home)
                .count();
            if living < max {
                self.spawn_creature(kind, home);
            }
        }
        for (_post, target, x0, y0, x1, y1) in shots {
            self.events.push(Ev::Bolt {
                x0,
                y0,
                x1,
                y1: y1 - 6,
            });
            let d = self.rng.range(2, 3);
            self.damage(None, Tgt::Unit(target), d);
        }
    }

    fn spawn_creature(&mut self, kind: Kind, home: u32) {
        let Some(b) = self.bldg(home) else { return };
        let (cx, cy) = b.center();
        let id = self.id();
        let max = base_hp(kind);
        let x = cx + self.rng.range(-10, 10);
        let y = cy + self.rng.range(4, 14);
        self.units.push(Unit {
            id,
            kind,
            name: if kind == Kind::Heron {
                "HERON"
            } else {
                "BOG WISP"
            },
            trait_: Trait::Steady,
            x,
            y,
            hp: max,
            max_hp: max,
            level: 1,
            xp: 0,
            purse: 0,
            home,
            job: if kind == Kind::Heron {
                Job::Guard
            } else {
                Job::Drift
            },
            path: vec![],
            path_to: (-1, -1),
            cool: 0,
            think: 0,
            left: true,
            moving: false,
            bubble: Bubble::None,
            bubble_t: 0,
            hit_by: None,
            hit_t: 0,
            hurt_t: 0,
            seen: vec![],
            wants: 0,
            dead: false,
            timer: 0,
            wait: 0,
        });
    }

    fn raids(&mut self) {
        self.raid_t -= 1;
        if self.raid_t > 0 {
            return;
        }
        self.raid_t = self.rng.range(600, 900);
        let guards: Vec<u32> = self
            .units
            .iter()
            .filter(|u| !u.dead && u.kind == Kind::Heron && u.job == Job::Guard)
            .map(|u| u.id)
            .collect();
        if guards.is_empty() {
            self.raid_t = 150;
            return;
        }
        let heron = guards[self.rng.below(guards.len() as u32) as usize];
        let mut targets: Vec<u32> = self
            .bldgs
            .iter()
            .filter(|b| b.alive && !b.kind.enemy() && b.kind == BKind::Still)
            .map(|b| b.id)
            .collect();
        if targets.is_empty() || self.rng.chance(1, 3) {
            targets = self
                .bldgs
                .iter()
                .filter(|b| b.alive && !b.kind.enemy())
                .map(|b| b.id)
                .collect();
        }
        let target = targets[self.rng.below(targets.len() as u32) as usize];
        if let Some(u) = self.unit_mut(heron) {
            u.job = Job::Raid(target);
            u.timer = 180;
        }
        self.events.push(Ev::Raid { heron });
    }

    fn curse(&mut self) {
        self.cursed = true;
        for y in 8..12 {
            for x in 19..23 {
                if self.map.at(x, y) != Tile::Tree {
                    self.map.set(x, y, Tile::Curse);
                }
            }
        }
        let id = self.add_bldg(BKind::Glade, 20, 9, true);
        if let Some(b) = self.bldg_mut(id) {
            b.timer = 60;
        }
        self.events.push(Ev::Curse);
    }

    // ---- units ---------------------------------------------------------------------------------

    fn unit_tick(&mut self, i: usize) {
        {
            let u = &mut self.units[i];
            u.cool = (u.cool - 1).max(0);
            u.bubble_t = (u.bubble_t - 1).max(0);
            u.hit_t = (u.hit_t - 1).max(0);
            u.hurt_t = (u.hurt_t - 1).max(0);
            u.timer -= 1;
            u.think -= 1;
            if u.bubble_t == 0 && matches!(u.bubble, Bubble::Yes | Bubble::No) {
                u.bubble = Bubble::None;
            }
        }
        if self.units[i].think <= 0 {
            self.units[i].think = 10;
            match self.units[i].kind {
                Kind::Newt | Kind::Toad => self.hero_think(i),
                Kind::Heron => self.heron_think(i),
                Kind::Wisp => self.wisp_think(i),
            }
        }
        self.act(i);
    }

    /// The nearest living unit of the other side within `r` pixels; fleeing heroes are let go.
    fn enemy_near(&self, x: i32, y: i32, r: i32, heroes: bool) -> Option<u32> {
        self.units
            .iter()
            .filter(|u| !u.dead && u.kind.hero() == heroes && u.job != Job::Heal)
            .filter(|u| (u.x - x).abs() <= r && (u.y - y).abs() <= r)
            .min_by_key(|u| (u.x - x).abs() + (u.y - y).abs())
            .map(|u| u.id)
    }

    fn hero_think(&mut self, i: usize) {
        let (id, x, y, hp, max, tr) = {
            let u = &self.units[i];
            (u.id, u.x, u.y, u.hp, u.max_hp, u.trait_)
        };
        let job = self.units[i].job;
        // Hurt heroes make for the Star Pond.
        if job == Job::Heal {
            if hp >= max {
                self.set_job(i, Job::Idle);
                self.units[i].bubble = Bubble::None;
            }
            return;
        }
        if hp * 100 < max * tr.flee() {
            self.drop_bounty(i);
            self.set_job(i, Job::Heal);
            self.units[i].bubble = Bubble::Heart;
            self.events.push(Ev::Healing { id });
            return;
        }
        // Struck by something? Hit back, unless it's far too strong for them.
        if let Some(att) = self.units[i].hit_by {
            if self.units[i].hit_t > 0 && self.unit(att).is_some() {
                if !matches!(self.units[i].job, Job::Fight(_)) {
                    self.units[i].job = Job::Fight(att);
                }
                return;
            }
        }
        if let Job::Fight(t) = job {
            if self.unit(t).is_some() && self.reach(x, y, Tgt::Unit(t)) < 6 * TILE {
                return;
            }
            self.set_job(i, Job::Idle);
        }
        // A reward they took: keep at it while it stands.
        if let Job::Bounty(f) = job {
            if self.flag(f).is_some_and(|fl| self.alive(fl.tgt)) {
                return;
            }
            self.set_job(i, Job::Idle);
        }
        if let Job::Defend(t) = job {
            if self.unit(t).is_some() {
                return;
            }
            self.set_job(i, Job::Idle);
        }
        // Anything raiding the hold nearby? The bold go out to meet it.
        if tr != Trait::Timid {
            let raider = self
                .units
                .iter()
                .filter(|u| !u.dead && !u.kind.hero())
                .find(|u| {
                    self.bldgs.iter().any(|b| {
                        b.alive
                            && !b.kind.enemy()
                            && (b.center().0 - u.x).abs() < 72
                            && (b.center().1 - u.y).abs() < 72
                    }) && (u.x - x).abs() < 9 * TILE
                        && (u.y - y).abs() < 9 * TILE
                })
                .map(|u| u.id);
            if let Some(r) = raider {
                self.set_job(i, Job::Defend(r));
                return;
            }
        }
        // Weigh the rewards on offer.
        let mut best: Option<(u32, i32)> = None;
        let mut refusals = vec![];
        for f in self.flags.iter() {
            if !self.alive(f.tgt) {
                continue;
            }
            let hero = &self.units[i];
            let ok = self.accepts(hero, f);
            if ok {
                let margin = f.reward * hero.trait_.greed() / 100 - self.asking(hero, f);
                if best.map_or(true, |(_, m)| margin > m) {
                    best = Some((f.id, margin));
                }
            } else if !hero.seen.contains(&(f.id, f.reward)) {
                refusals.push((f.id, f.reward, self.asking(hero, f)));
            }
        }
        for (fid, reward, asking) in refusals {
            let wants = round_up(asking * 100 / tr.greed(), REWARD_STEP);
            let u = &mut self.units[i];
            u.seen.push((fid, reward));
            u.wants = wants;
            u.bubble = Bubble::No;
            u.bubble_t = 25;
            self.events.push(Ev::Refuse {
                hero: id,
                flag: fid,
                wants,
            });
        }
        if let Some((fid, _)) = best {
            if let Some(f) = self.flags.iter_mut().find(|f| f.id == fid) {
                if !f.takers.contains(&id) {
                    f.takers.push(id);
                }
            }
            self.set_job(i, Job::Bounty(fid));
            let u = &mut self.units[i];
            u.bubble = Bubble::Yes;
            u.bubble_t = 25;
            u.seen.push((fid, 0));
            self.events.push(Ev::Accept {
                hero: id,
                flag: fid,
            });
            return;
        }
        if hp < max {
            if job != Job::Rest {
                self.set_job(i, Job::Rest);
            }
            return;
        }
        // Otherwise potter about near home; newts roam further.
        if matches!(job, Job::Idle | Job::Rest)
            || (matches!(job, Job::Wander(..)) && self.units[i].path.is_empty())
        {
            let home = self.units[i].home;
            if let Some((btx, bty)) = self.bldg(home).map(|b| (b.tx, b.ty)) {
                let r = if self.units[i].kind == Kind::Newt {
                    4
                } else {
                    2
                };
                let tx = (btx + self.rng.range(-r, r + 1)).clamp(1, 10);
                let ty = (bty + self.rng.range(-r, r + 1)).clamp(1, MH - 2);
                if self.map.walkable(tx, ty) && self.rng.chance(1, 3) {
                    self.set_job(i, Job::Wander(tx, ty));
                }
            }
        }
    }

    /// A reward's target got away: take the reward down and free its dew.
    fn lapse(&mut self, t: Tgt) {
        let Some(pos) = self.flags.iter().position(|f| f.tgt == t) else {
            return;
        };
        let f = self.flags.remove(pos);
        for u in self.units.iter_mut() {
            if u.job == Job::Bounty(f.id) {
                u.job = Job::Idle;
                u.path.clear();
            }
        }
        self.events.push(Ev::Lapsed { flag: f.id });
    }

    fn drop_bounty(&mut self, i: usize) {
        let id = self.units[i].id;
        if let Job::Bounty(f) = self.units[i].job {
            if let Some(fl) = self.flags.iter_mut().find(|fl| fl.id == f) {
                fl.takers.retain(|&t| t != id);
            }
        }
    }

    fn set_job(&mut self, i: usize, job: Job) {
        if self.units[i].job != job {
            if job != Job::Heal {
                self.drop_bounty(i);
            }
            self.units[i].job = job;
            self.units[i].path.clear();
            self.units[i].path_to = (-1, -1);
        }
    }

    fn heron_think(&mut self, i: usize) {
        let (id, x, y, home, job, hp, max) = {
            let u = &self.units[i];
            (u.id, u.x, u.y, u.home, u.job, u.hp, u.max_hp)
        };
        let nest = self.bldg(home).map(|b| b.center());
        let Some((nx, ny)) = nest else {
            self.units[i].job = Job::Leave;
            return;
        };
        let _ = id;
        match job {
            Job::Leave => {}
            Job::Raid(b) => {
                if self.bldg(b).is_none() || self.units[i].timer <= 0 || hp * 2 < max {
                    self.units[i].job = Job::Return;
                    let id = self.units[i].id;
                    self.lapse(Tgt::Unit(id));
                } else if let Some(att) = self.units[i].hit_by.filter(|_| self.units[i].hit_t > 0) {
                    if self.unit(att).is_some_and(|h| h.job != Job::Heal) {
                        self.units[i].job = Job::Fight(att);
                    }
                }
            }
            Job::Return => {
                if (x - nx).abs() < 20 && (y - ny).abs() < 24 {
                    self.units[i].job = Job::Guard;
                }
            }
            Job::Fight(t) => {
                let far = (x - nx).abs() > 5 * TILE || (y - ny).abs() > 5 * TILE;
                // A fleeing hero is chased for a moment, then let go.
                let gone = self.unit(t).is_none();
                let fleeing = self.unit(t).is_some_and(|h| h.job == Job::Heal);
                if gone || far || (fleeing && self.rng.chance(1, 2)) {
                    self.units[i].job = Job::Return;
                }
            }
            _ => {
                // Guard: strike at heroes that come near the nest, or at whoever struck.
                if let Some(att) = self.units[i].hit_by.filter(|_| self.units[i].hit_t > 0) {
                    if self.unit(att).is_some_and(|h| h.job != Job::Heal) {
                        self.units[i].job = Job::Fight(att);
                        return;
                    }
                }
                if let Some(h) = self.enemy_near(nx, ny, 4 * TILE, true) {
                    self.units[i].job = Job::Fight(h);
                }
            }
        }
    }

    fn wisp_think(&mut self, i: usize) {
        let (x, y) = (self.units[i].x, self.units[i].y);
        if self.bldg(self.units[i].home).is_none() {
            self.units[i].dead = true;
            return;
        }
        if let Some(h) = self.enemy_near(x, y, 3 * TILE, true) {
            self.units[i].job = Job::Fight(h);
        } else {
            self.units[i].job = Job::Drift;
        }
    }

    /// Where a unit is trying to get to and what it means to hit when it arrives.
    fn goal(&self, i: usize) -> Option<(Tgt, i32)> {
        let u = &self.units[i];
        let range = if u.kind == Kind::Toad { 40 } else { 10 };
        match u.job {
            Job::Bounty(f) => {
                let fl = self.flag(f)?;
                // Herons guarding a nest come first.
                if let Tgt::Bldg(nid) = fl.tgt {
                    if let Some(g) = self.units.iter().find(|h| {
                        !h.dead
                            && h.home == nid
                            && h.kind == Kind::Heron
                            && matches!(h.job, Job::Fight(t) if t == u.id)
                    }) {
                        return Some((Tgt::Unit(g.id), range));
                    }
                }
                Some((fl.tgt, range))
            }
            Job::Fight(t) | Job::Defend(t) => Some((Tgt::Unit(t), range)),
            Job::Raid(b) => Some((Tgt::Bldg(b), 10)),
            _ => None,
        }
    }

    /// Is this hero's reward target too strong for the party that has signed up, so that they
    /// should wait at the bridge for company first? Returns the rally tile if so.
    fn should_gather(&self, i: usize) -> Option<(i32, i32)> {
        let u = &self.units[i];
        let Job::Bounty(f) = u.job else { return None };
        let fl = self.flag(f)?;
        let (tx, ty) = self.tgt_pos(fl.tgt)?;
        if u.wait >= 300 || tx < 11 * TILE || self.reach(u.x, u.y, fl.tgt) < 5 * TILE {
            return None;
        }
        let party: i32 = fl
            .takers
            .iter()
            .filter_map(|&id| self.unit(id))
            .filter(|h| h.job != Job::Heal)
            .map(|h| self.power(h))
            .sum();
        if party * 4 >= self.danger(fl.tgt) * 3 {
            return None;
        }
        Some(if ty < 7 * TILE { (10, 3) } else { (10, 11) })
    }

    fn act(&mut self, i: usize) {
        let kind = self.units[i].kind;
        let (x, y) = (self.units[i].x, self.units[i].y);
        self.units[i].moving = false;
        if kind.hero() {
            if let Some((rx, ry)) = self.should_gather(i) {
                if (x / TILE - rx).abs() <= 1 && (y / TILE - ry).abs() <= 1 {
                    self.units[i].wait += 1;
                } else {
                    self.walk_to_tile(i, &|a, b| a == rx && b == ry);
                }
                return;
            }
            if !matches!(self.units[i].job, Job::Bounty(_)) {
                self.units[i].wait = 0;
            }
        }
        // Fight if something to fight is in reach.
        if let Some((t, range)) = self.goal(i) {
            let d = self.reach(x, y, t);
            if d <= range {
                if self.units[i].cool == 0 {
                    self.strike(i, t);
                }
                if let Some((tx, _)) = self.tgt_pos(t) {
                    self.units[i].left = tx < x;
                }
                return;
            }
            match kind {
                Kind::Heron | Kind::Wisp => {
                    if let Some((tx, ty)) = self.tgt_pos(t) {
                        let sp = if kind == Kind::Heron { 3 } else { 1 };
                        let ox = (self.units[i].id % 3) as i32 * 8 - 8;
                        self.fly_toward(i, tx + ox, ty + 4, sp);
                    }
                }
                _ => self.walk_toward(i, t),
            }
            return;
        }
        match self.units[i].job {
            Job::Heal => {
                // The Star Pond: heal while beside it.
                let at_pond = (1..=4).any(|tx| {
                    (11..=14).any(|ty| {
                        self.map.at(tx, ty) == Tile::Pond
                            && ((tx * TILE + 8) - x).abs() <= 24
                            && ((ty * TILE + 8) - y).abs() <= 24
                    })
                });
                if at_pond {
                    if self.tick % 4 == 0 {
                        let u = &mut self.units[i];
                        u.hp = (u.hp + 1).min(u.max_hp);
                    }
                } else {
                    self.walk_to_tile(i, &|tx, ty| {
                        (1..=4).contains(&tx) && (11..=14).contains(&ty)
                    });
                }
            }
            Job::Rest => {
                let home = self.units[i].home;
                if let Some(b) = self.bldg(home) {
                    let (bx, by) = (b.tx, b.ty);
                    if self.reach(x, y, Tgt::Bldg(home)) <= 16 {
                        if self.tick % 10 == 0 {
                            let u = &mut self.units[i];
                            u.hp = (u.hp + 1).min(u.max_hp);
                        }
                        self.units[i].bubble = Bubble::Zz;
                    } else {
                        self.walk_to_tile(i, &|tx, ty| {
                            tx >= bx - 1
                                && tx <= bx + 2
                                && ty >= by - 1
                                && ty <= by + 2
                                && (tx < bx || tx > bx + 1 || ty < by || ty > by + 1)
                        });
                    }
                }
            }
            Job::Wander(tx, ty) => {
                if x / TILE == tx && y / TILE == ty {
                    self.units[i].job = Job::Idle;
                } else {
                    self.walk_to_tile(i, &|a, b| a == tx && b == ty);
                }
                if self.units[i].bubble == Bubble::Zz {
                    self.units[i].bubble = Bubble::None;
                }
            }
            Job::Guard => {
                // Herons stand near the nest and shift about now and then.
                if let Some(b) = self.bldg(self.units[i].home) {
                    let (cx, cy) = b.center();
                    if (x - cx).abs() > 28 || (y - cy).abs() > 30 {
                        self.fly_toward(i, cx, cy + 12, 2);
                    } else {
                        // Pace slowly between spots beside the nest.
                        let (px, py) = self.units[i].path_to;
                        if px < 0 || (px == x && py == y) {
                            if self.rng.chance(1, 40) {
                                let nx = cx + self.rng.range(-20, 20);
                                let ny = cy + self.rng.range(10, 22);
                                self.units[i].path_to = (nx, ny);
                            }
                        } else {
                            self.step_to(i, px, py);
                        }
                    }
                }
            }
            Job::Return => {
                if let Some(b) = self.bldg(self.units[i].home) {
                    let (cx, cy) = b.center();
                    self.fly_toward(i, cx, cy + 12, 3);
                }
            }
            Job::Leave => {
                self.fly_toward(i, 460, y, 3);
                if self.units[i].x > 420 {
                    self.units[i].dead = true;
                }
            }
            Job::Drift => {
                // Wisps drift to the nearest dew still and hang about it, draining it.
                let target = self
                    .bldgs
                    .iter()
                    .filter(|b| b.ready() && b.kind == BKind::Still)
                    .min_by_key(|b| (b.center().0 - x).abs() + (b.center().1 - y).abs())
                    .map(|b| (b.center(), b.id));
                if let Some(((cx, cy), bid)) = target {
                    if (cx - x).abs() < 20 && (cy - y).abs() < 20 {
                        if self.units[i].cool == 0 {
                            self.units[i].cool = 30;
                            self.damage(Some(i), Tgt::Bldg(bid), 1);
                        }
                    } else {
                        self.fly_toward(i, cx, cy + 6, 1);
                    }
                }
            }
            _ => {}
        }
    }

    fn strike(&mut self, i: usize, t: Tgt) {
        let (kind, level, x, y) = {
            let u = &self.units[i];
            (u.kind, u.level, u.x, u.y)
        };
        let (lo, hi, cool) = match kind {
            Kind::Newt => (2, 4, 12),
            Kind::Toad => (3, 5, 18),
            Kind::Heron => (2, 4, 16),
            Kind::Wisp => (1, 2, 20),
        };
        let mut d = self.rng.range(lo, hi) + level - 1;
        if matches!(t, Tgt::Bldg(_)) && !kind.hero() {
            d = (d / 2).max(1);
        }
        self.units[i].cool = cool;
        if kind == Kind::Toad {
            if let Some((tx, ty)) = self.tgt_pos(t) {
                self.events.push(Ev::Bolt {
                    x0: x + 6,
                    y0: y - 12,
                    x1: tx,
                    y1: ty - 6,
                });
            }
        }
        self.damage(Some(i), t, d);
    }

    /// Deal damage; handle death, experience and rewards.
    fn damage(&mut self, by: Option<usize>, t: Tgt, d: i32) {
        let by_id = by.map(|i| self.units[i].id);
        let by_hero = by.is_some_and(|i| self.units[i].kind.hero());
        if let (Some(hid), true) = (by_id, by_hero) {
            if let Some(f) = self.flags.iter_mut().find(|f| f.tgt == t) {
                if !f.hitters.contains(&hid) {
                    f.hitters.push(hid);
                }
            }
        }
        let (killed, px, py) = match t {
            Tgt::Unit(id) => {
                let Some(u) = self.unit_mut(id) else { return };
                u.hp -= d;
                u.hurt_t = 2;
                if let Some(b) = by_id {
                    u.hit_by = Some(b);
                    u.hit_t = 45;
                }
                (u.hp <= 0, u.x, u.y - 8)
            }
            Tgt::Bldg(id) => {
                let calm = self.calm;
                let Some(b) = self.bldg_mut(id) else { return };
                b.hp -= d;
                let (cx, cy) = b.center();
                let summon = b.kind == BKind::Nest && !calm && by_hero && b.timer > 120;
                if summon {
                    b.timer = 120;
                }
                let dead = b.hp <= 0;
                if summon && !dead {
                    let guards = self
                        .units
                        .iter()
                        .filter(|u| !u.dead && u.home == id)
                        .count();
                    if guards < 3 {
                        self.spawn_creature(Kind::Heron, id);
                    }
                }
                (dead, cx, cy)
            }
        };
        self.events.push(Ev::Hit {
            x: px,
            y: py,
            by_hero,
        });
        if let Some(i) = by {
            if by_hero {
                // Fighting creatures teaches more than hacking at nests.
                let xp = if matches!(t, Tgt::Unit(_)) { d } else { d / 3 };
                self.gain_xp(i, xp);
            }
        }
        if killed {
            self.kill(t, by);
        }
    }

    fn gain_xp(&mut self, i: usize, xp: i32) {
        let u = &mut self.units[i];
        u.xp += xp;
        let need = [0, 30, 80, 150, 240];
        while u.level < 5 && u.xp >= need[u.level as usize] {
            u.level += 1;
            let extra = if u.kind == Kind::Newt { 5 } else { 6 };
            u.max_hp += extra;
            u.hp += extra;
            self.events.push(Ev::LevelUp { id: u.id });
        }
    }

    fn kill(&mut self, t: Tgt, by: Option<usize>) {
        match t {
            Tgt::Unit(id) => {
                let Some(u) = self.unit_mut(id) else { return };
                u.dead = true;
                let (kind, x, y) = (u.kind, u.x, u.y);
                if kind.hero() {
                    self.graves.push((x, y, 300));
                    self.fallen += 1;
                    self.events.push(Ev::Fell { id });
                    for f in self.flags.iter_mut() {
                        f.takers.retain(|&h| h != id);
                        f.hitters.retain(|&h| h != id);
                    }
                } else {
                    self.events.push(Ev::Died { id, kind });
                    if let Some(i) = by {
                        if self.units[i].kind.hero() {
                            let bonus = if kind == Kind::Heron { 8 } else { 4 };
                            self.gain_xp(i, bonus);
                        }
                    }
                }
            }
            Tgt::Bldg(id) => {
                let Some(b) = self.bldg_mut(id) else { return };
                b.alive = false;
                let (kind, tx, ty) = (b.kind, b.tx, b.ty);
                self.map.occupy(tx, ty, 2, 2, false);
                if kind == BKind::Glade {
                    for y in 0..MH {
                        for x in 0..MW {
                            if self.map.at(x, y) == Tile::Curse {
                                self.map.set(x, y, Tile::Moss);
                            }
                        }
                    }
                }
                self.events.push(Ev::Down { id, kind });
                if kind.enemy() {
                    // The nest's herons, or the stone's wisps, scatter.
                    let mut gone = vec![];
                    for u in self.units.iter_mut().filter(|u| u.home == id) {
                        if u.kind == Kind::Heron {
                            u.job = Job::Leave;
                        } else {
                            u.dead = true;
                        }
                        gone.push(u.id);
                    }
                    for g in gone {
                        self.lapse(Tgt::Unit(g));
                    }
                    if let Some(i) = by {
                        if self.units[i].kind.hero() {
                            self.gain_xp(i, 20);
                        }
                    }
                }
            }
        }
        self.pay(t);
    }

    /// The target of a reward is gone: pay everyone who fought for it, or refund it.
    fn pay(&mut self, t: Tgt) {
        let Some(pos) = self.flags.iter().position(|f| f.tgt == t) else {
            return;
        };
        let f = self.flags.remove(pos);
        let hitters: Vec<u32> = f
            .hitters
            .iter()
            .copied()
            .filter(|&h| self.unit(h).is_some())
            .collect();
        if hitters.is_empty() {
            return; // nobody earned it: the reserved dew stays in the treasury
        }
        let share = f.reward / hitters.len() as i32;
        let extra = f.reward - share * hitters.len() as i32;
        self.dew -= f.reward;
        self.paid_total += f.reward;
        for (k, h) in hitters.iter().enumerate() {
            let amount = share + if k == 0 { extra } else { 0 };
            if let Some(u) = self.unit_mut(*h) {
                u.purse += amount;
                u.bubble = Bubble::Dew;
                u.bubble_t = 30;
            }
            self.events.push(Ev::Paid { hero: *h, amount });
        }
        for u in self.units.iter_mut() {
            if u.job == Job::Bounty(f.id) {
                u.job = Job::Idle;
                u.path.clear();
            }
        }
    }

    // ---- movement ------------------------------------------------------------------------------

    fn fly_toward(&mut self, i: usize, tx: i32, ty: i32, sp: i32) {
        let u = &mut self.units[i];
        let (dx, dy) = (tx - u.x, ty - u.y);
        let dist = dx.abs().max(dy.abs());
        if dist == 0 {
            return;
        }
        let step = sp.min(dist);
        u.x += dx * step / dist;
        u.y += dy * step / dist;
        u.left = dx < 0;
        u.moving = true;
    }

    fn walk_toward(&mut self, i: usize, t: Tgt) {
        let Some((tx, ty)) = self.tgt_pos(t) else {
            return;
        };
        let range_tiles = if self.units[i].kind == Kind::Toad {
            2
        } else {
            1
        };
        let goal_tile = (tx / TILE, ty / TILE);
        let footprint = match t {
            Tgt::Bldg(id) => self.bldg(id).map(|b| (b.tx, b.ty, 2)),
            Tgt::Unit(_) => Some((goal_tile.0, goal_tile.1, 1)),
        };
        let Some((fx, fy, fw)) = footprint else {
            return;
        };
        let near = move |x: i32, y: i32| {
            let dx = (fx - x).max(x - (fx + fw - 1)).max(0);
            let dy = (fy - y).max(y - (fy + fw - 1)).max(0);
            dx.max(dy) <= range_tiles
        };
        let from = (self.units[i].x / TILE, self.units[i].y / TILE);
        if near(from.0, from.1) {
            // Beside it: close in on this hero's own side of the target, so a party fans out
            // around it rather than standing in one heap.
            self.units[i].path.clear();
            let k = (self.units[i].id % 6) as usize;
            let (ox, oy) = match t {
                Tgt::Bldg(_) => [(-18, 6), (18, 6), (-10, 18), (10, 18), (-18, -6), (18, -6)][k],
                Tgt::Unit(_) => [(-9, 0), (9, 0), (-6, 5), (6, 5), (-6, -5), (6, -5)][k],
            };
            self.step_to(i, tx + ox, ty + oy);
            return;
        }
        if self.units[i].path_to != goal_tile
            || self.units[i].path.is_empty()
            || self.tick % 30 == (self.units[i].id % 30)
        {
            self.units[i].path_to = goal_tile;
            self.units[i].path = self.map.path(from, &near).unwrap_or_default();
            if self.units[i].path.is_empty() {
                self.step_to(i, tx, ty);
                return;
            }
        }
        self.follow(i);
    }

    fn walk_to_tile(&mut self, i: usize, goal: &dyn Fn(i32, i32) -> bool) {
        if self.units[i].path.is_empty() {
            let from = (self.units[i].x / TILE, self.units[i].y / TILE);
            if goal(from.0, from.1) {
                return;
            }
            self.units[i].path = self.map.path(from, goal).unwrap_or_default();
            if self.units[i].path.is_empty() {
                return;
            }
        }
        self.follow(i);
    }

    fn follow(&mut self, i: usize) {
        let Some(&(tx, ty)) = self.units[i].path.first() else {
            return;
        };
        let (px, py) = (tx * TILE + 8, ty * TILE + 12);
        if self.units[i].x == px && self.units[i].y == py {
            self.units[i].path.remove(0);
            return;
        }
        self.step_to(i, px, py);
    }

    fn step_to(&mut self, i: usize, px: i32, py: i32) {
        let sp = if self.units[i].kind == Kind::Newt {
            2
        } else {
            1
        };
        let u = &mut self.units[i];
        let dx = (px - u.x).clamp(-sp, sp);
        let dy = (py - u.y).clamp(-sp, sp);
        u.x += dx;
        u.y += dy;
        if dx != 0 {
            u.left = dx < 0;
        }
        u.moving = dx != 0 || dy != 0;
    }

    /// One line on what a hero is doing, for the panel.
    pub fn status(&self, u: &Unit) -> String {
        match u.job {
            Job::Bounty(f) => match self.flag(f) {
                None => "BACK HOME".into(),
                Some(fl)
                    if self
                        .units
                        .iter()
                        .position(|v| v.id == u.id)
                        .is_some_and(|i| self.should_gather(i).is_some()) =>
                {
                    let i = self.units.iter().position(|v| v.id == u.id).unwrap();
                    let (rx, ry) = self.should_gather(i).unwrap();
                    if (u.x / TILE - rx).abs() <= 1 && (u.y / TILE - ry).abs() <= 1 {
                        format!("{} DEW: WAITING FOR COMPANY", fl.reward)
                    } else {
                        format!("{} DEW: OFF TO THE BRIDGE", fl.reward)
                    }
                }
                Some(fl) => format!("AFTER THE {} DEW REWARD", fl.reward),
            },
            Job::Fight(_) => "FIGHTING".into(),
            Job::Defend(_) => "DEFENDING THE HOLD".into(),
            Job::Heal => "HEALING AT THE STAR POND".into(),
            Job::Rest => "RESTING AT HOME".into(),
            Job::Wander(..) => "WANDERING".into(),
            Job::Guard => "GUARDING THE NEST".into(),
            Job::Raid(_) => "RAIDING THE HOLD".into(),
            Job::Return | Job::Leave => "FLYING OFF".into(),
            Job::Drift => "DRAINING THE STILLS".into(),
            Job::Idle if u.wants > 0 => format!("WANTS {} DEW TO GO", u.wants),
            Job::Idle => "IDLE".into(),
        }
    }
}

fn base_hp(kind: Kind) -> i32 {
    match kind {
        Kind::Newt => 20,
        Kind::Toad => 26,
        Kind::Heron => 12,
        Kind::Wisp => 8,
    }
}

fn round_up(v: i32, step: i32) -> i32 {
    (v + step - 1) / step * step
}

#[allow(dead_code)]
fn on_map(x: i32, y: i32) -> bool {
    x >= 0 && y >= 0 && x < MW * TILE && y < MH * TILE
}

#[cfg(test)]
mod tests {
    use super::*;

    fn nest(w: &World, n: usize) -> u32 {
        w.bldgs
            .iter()
            .filter(|b| b.kind == BKind::Nest)
            .nth(n)
            .unwrap()
            .id
    }

    fn run(w: &mut World, ticks: u32) {
        for _ in 0..ticks {
            w.step();
        }
    }

    #[test]
    fn pip_takes_the_first_reward_and_is_paid() {
        let mut w = World::new(1);
        let n1 = nest(&w, 0);
        let f = w.post(Tgt::Bldg(n1)).unwrap();
        assert_eq!(w.reserved(), 20);
        run(&mut w, 40);
        assert!(w.flag(f).unwrap().takers.len() == 1, "the brave newt goes");
        run(&mut w, 900);
        assert!(w.bldg(n1).is_none(), "the weak nest falls");
        let pip = &w.units[0];
        assert_eq!(pip.purse, 20);
        assert_eq!(w.reserved(), 0);
        assert_eq!(w.paid_total, 20, "the reward left the treasury");
    }

    #[test]
    fn a_timid_toad_wants_more_and_takes_courage_from_company() {
        let mut w = World::new(2);
        let hall = w
            .bldgs
            .iter()
            .find(|b| b.kind == BKind::ToadHall)
            .unwrap()
            .id;
        let bufo = w.hire(hall).unwrap();
        let n2 = nest(&w, 1);
        let f = w.post(Tgt::Bldg(n2)).unwrap();
        let toad = w.unit(bufo).unwrap();
        let alone = w.asking(toad, w.flag(f).unwrap());
        assert!(alone > 20, "Bufo won't go alone for 20 (asks {alone})");
        w.flags[0].takers.push(w.units[0].id);
        let with_pip = w.asking(w.unit(bufo).unwrap(), w.flag(f).unwrap());
        assert!(with_pip < alone, "company makes him braver");
        assert!(
            with_pip <= 30,
            "and 30 is enough with Pip along (asks {with_pip})"
        );
    }

    #[test]
    fn cancelling_returns_the_dew_and_calls_heroes_home() {
        let mut w = World::new(3);
        let n1 = nest(&w, 0);
        let f = w.post(Tgt::Bldg(n1)).unwrap();
        run(&mut w, 30);
        w.cancel(f);
        assert_eq!(w.reserved(), 0);
        assert!(w.units.iter().all(|u| !matches!(u.job, Job::Bounty(_))));
    }

    #[test]
    fn rewards_cost_what_is_free() {
        let mut w = World::new(4);
        w.dew = 25;
        let n1 = nest(&w, 0);
        let f = w.post(Tgt::Bldg(n1)).unwrap();
        assert!(!w.raise(f), "only 5 dew left free");
        assert!(w.post(Tgt::Bldg(nest(&w, 1))).is_none());
    }

    #[test]
    fn after_the_lesson_herons_hatch_and_raid() {
        let mut w = World::new(5);
        w.end_calm();
        run(&mut w, 900);
        assert!(w.units.iter().any(|u| u.kind == Kind::Heron));
        assert!(w.events.iter().any(|e| matches!(e, Ev::Raid { .. })));
    }

    #[test]
    fn a_focused_elder_clears_the_marsh_in_a_few_minutes() {
        for seed in [6, 7, 8, 9, 10] {
            let mut w = World::new(seed);
            for t in 0..15 * 60 * 8 {
                crate::bot::elder(&mut w, t);
                w.step();
                if w.over.is_some() {
                    break;
                }
            }
            assert_eq!(
                w.over,
                Some(true),
                "seed {seed} clears the marsh within eight minutes"
            );
            assert!(
                w.tick > 15 * 90,
                "seed {seed} takes more than a minute and a half"
            );
        }
    }
}
