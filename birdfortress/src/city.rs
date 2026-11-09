//! The deterministic colony. Homes fill with settlers who walk in from the trail; every resident
//! is a worker of its species; workplaces hire the species the player picked; workers carry goods
//! to whoever wants them within reach, going by their own species' map with that load.

use crate::level::{Challenge, Goal, Level};
use crate::path::{self, Field, Path, FAR};
use crate::rules::*;
use crate::world::{Occ, World};
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

pub type Id = u32;

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Place {
    B(Id),
    Fortress,
    Entrance,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Order {
    Accept,
    Refuse,
    Get,
}

/// What a building is stuck on, for its status line.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Note {
    Fine,
    /// No free workers of this species.
    NoWorkers(Species),
    /// No home of this species can walk here.
    NoReach(Species),
    /// The source is bare (maybe out of season).
    Bare,
    /// Waiting for an input.
    NeedInput(Good),
    /// Has a good that someone wants but nobody here can deliver: the nearest place that wants it
    /// and the ticks it takes carrying one (FAR: no way there at all; within RANGE: no way back).
    Stuck(Good, Place, i32),
    /// Settlers can't walk here from the trail.
    NoSettlers,
    /// A home lacks this for its tier.
    Lacks(Good),
    Hungry,
    /// Full and nothing to do.
    Full,
    /// Holding goods nobody needs right now (everyone in reach is stocked).
    Stocked(Good),
    Paused,
}

#[derive(Clone, Debug)]
pub struct Building {
    pub id: Id,
    pub kind: Kind,
    pub x: i32,
    pub y: i32,
    pub source: Option<usize>,
    /// Who works here (workplaces) or lives here (homes).
    pub species: Species,
    pub stock: [i32; NGOODS],
    pub incoming: [i32; NGOODS],
    pub orders: [Order; NGOODS],
    pub paused: bool,
    // Homes.
    pub tier: usize,
    pub residents: i32,
    pub coming: i32,
    pub settle_cd: i32,
    pub lack: i32,
    pub ready: i32,
    pub eat_acc: i32,
    pub wear_acc: i32,
    // Workplaces.
    pub progress: i32,
    pub note: Note,
}

impl Building {
    pub fn total(&self) -> i32 {
        self.stock.iter().sum()
    }
    pub fn total_in(&self) -> i32 {
        self.incoming.iter().sum()
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Job {
    /// Walking from a home to start work.
    Hire,
    /// A settler walking from the trail to a home.
    Settle,
    Idle,
    Harvest(i32),
    /// Carrying a good to a place.
    Carry(Place),
    Drop(i32, Place),
    /// Walking home empty.
    Return,
    /// A store carrier walking empty to another store to fetch a good.
    Fetch(Id),
    /// Carrying a fetched good home.
    Bring,
    /// Holding a good with nowhere to take it; try again when this runs out.
    Wait(i32),
}

#[derive(Clone, Debug)]
pub struct Creature {
    pub uid: u32,
    pub species: Species,
    /// The workplace it works at, or the home a settler is heading for.
    pub base: Id,
    pub cargo: Option<Good>,
    pub job: Job,
    pub path: Path,
    pub step: usize,
    pub prog: i32,
    pub x: i32,
    pub y: i32,
    pub facing: i32,
}

impl Creature {
    pub fn walking(&self) -> bool {
        matches!(self.job, Job::Hire | Job::Settle | Job::Carry(_) | Job::Return | Job::Fetch(_) | Job::Bring)
    }
    pub fn settler(&self) -> bool {
        self.job == Job::Settle
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Event {
    Settled(Id),
    TierUp(Id, usize),
    TierDown(Id, usize),
    Left(Id),
    Stage(usize),
    Crafted(Id),
    Broke(Id),
    Won,
    Delivered(Place, Good),
}

#[derive(Clone, Debug)]
pub enum Action {
    Build { kind: Kind, x: i32, y: i32, species: Species },
    Infra { infra: Infra, x: i32, y: i32 },
    Demolish { x: i32, y: i32 },
    SetSpecies { id: Id, species: Species },
    SetOrder { id: Id, good: Good, order: Order },
    Pause { id: Id, paused: bool },
}

impl Action {
    pub fn text(&self) -> String {
        match self {
            Action::Build { kind, x, y, species } => {
                if kind.is_home() || *kind == Kind::Garden {
                    format!("{} {x} {y}", kind.name())
                } else {
                    format!("{} {x} {y} {}", kind.name(), species.name())
                }
            }
            Action::Infra { infra, x, y } => format!("{} {x} {y}", infra.short()),
            Action::Demolish { x, y } => format!("DEMOLISH {x} {y}"),
            Action::SetSpecies { id, species } => format!("HIRE #{id} {}", species.name()),
            Action::SetOrder { id, good, order } => format!("ORDER #{id} {} {order:?}", good.name()),
            Action::Pause { id, paused } => format!("PAUSE #{id} {paused}"),
        }
    }
}

pub struct City {
    pub level: Level,
    pub world: World,
    pub tick: i32,
    pub seeds_tenths: i32,
    pub spent: i32,
    pub buildings: Vec<Building>,
    pub creatures: Vec<Creature>,
    pub src_stock: Vec<i32>,
    src_grow: Vec<i32>,
    pub stage: usize,
    pub fort_stock: [i32; NGOODS],
    pub fort_in: [i32; NGOODS],
    pub events: Vec<Event>,
    pub won: Option<i32>,
    pub peak_pop: i32,
    /// Set once the site's challenge has been broken.
    pub broke_challenge: bool,
    pub slipped: i32,
    next_id: Id,
    next_uid: u32,
    fields: RefCell<HashMap<(Place, usize, usize), Rc<Field>>>,
    field_stamp: RefCell<u32>,
}

/// Tiles where creatures stand to use a building: the bottom row of its footprint.
fn access_of(kind: Kind, x: i32, y: i32) -> Vec<(i32, i32)> {
    let (w, h) = kind.size();
    (0..w).map(|dx| (x + dx, y + h - 1)).collect()
}

impl City {
    pub fn new(level: &Level) -> City {
        let world = World::new(level);
        let src_stock = level.sources.iter().map(|s| if s.infinite() { 0 } else { s.max }).collect();
        let n = level.sources.len();
        City {
            level: level.clone(),
            world,
            tick: 0,
            seeds_tenths: level.seeds * 10,
            spent: 0,
            buildings: Vec::new(),
            creatures: Vec::new(),
            src_stock,
            src_grow: vec![0; n],
            stage: 0,
            fort_stock: [0; NGOODS],
            fort_in: [0; NGOODS],
            events: Vec::new(),
            won: None,
            peak_pop: 0,
            broke_challenge: false,
            slipped: 0,
            next_id: 1,
            next_uid: 1,
            fields: RefCell::new(HashMap::new()),
            field_stamp: RefCell::new(0),
        }
    }

    pub fn seeds(&self) -> i32 {
        self.seeds_tenths / 10
    }

    pub fn day(&self) -> i32 {
        day_of(self.tick)
    }

    pub fn season(&self) -> usize {
        season_of(self.tick)
    }

    pub fn building(&self, id: Id) -> Option<&Building> {
        self.buildings.iter().find(|b| b.id == id)
    }

    fn bidx(&self, id: Id) -> Option<usize> {
        self.buildings.iter().position(|b| b.id == id)
    }

    pub fn building_at(&self, x: i32, y: i32) -> Option<&Building> {
        match self.world.occ(x, y) {
            Occ::Building(id) => self.building(id),
            _ => None,
        }
    }

    pub fn camp_on(&self, src: usize) -> Option<&Building> {
        self.buildings.iter().find(|b| b.source == Some(src))
    }

    pub fn access(&self, p: Place) -> Vec<(i32, i32)> {
        match p {
            Place::B(id) => self.building(id).map(|b| access_of(b.kind, b.x, b.y)).unwrap_or_default(),
            Place::Fortress => {
                let (fx, fy, fw, fh) = self.level.fortress;
                (fx..fx + fw).map(|x| (x, fy + fh - 1)).collect()
            }
            Place::Entrance => vec![self.level.entrance],
        }
    }

    /// Arrival times from a place for a species and load (cached until movement changes).
    pub fn field(&self, p: Place, s: Species, load: Load) -> Rc<Field> {
        if *self.field_stamp.borrow() != self.world.stamp {
            self.fields.borrow_mut().clear();
            *self.field_stamp.borrow_mut() = self.world.stamp;
        }
        let key = (p, s.index(), load.key());
        if let Some(f) = self.fields.borrow().get(&key) {
            return f.clone();
        }
        let f = Rc::new(path::field(&self.world, s, load, &self.access(p)));
        self.fields.borrow_mut().insert(key, f.clone());
        f
    }

    /// Ticks for a species carrying `load` from one place to another (FAR if it can't).
    pub fn trip(&self, from: Place, to: Place, s: Species, load: Load) -> i32 {
        let f = self.field(from, s, load);
        f.best(&self.world, &self.access(to)).map(|(d, _)| d).unwrap_or(FAR)
    }

    // ----- Population -----------------------------------------------------------------------

    pub fn pop(&self) -> i32 {
        self.buildings.iter().map(|b| b.residents).sum()
    }

    pub fn pop_of(&self, s: Species) -> i32 {
        self.buildings.iter().filter(|b| b.kind.is_home() && b.species == s).map(|b| b.residents).sum()
    }

    /// Workers of a species hired (at work or walking there).
    pub fn employed(&self, s: Species) -> i32 {
        self.creatures.iter().filter(|c| c.species == s && !c.settler()).count() as i32
    }

    /// Worker slots at a building: camps on slow, renewable sources take one.
    pub fn slots(&self, b: &Building) -> i32 {
        match (b.kind, b.source) {
            (Kind::Camp, Some(s)) => camp_slots(self.level.sources[s].good),
            (k, _) => k.slots(),
        }
    }

    pub fn workers_at(&self, id: Id) -> i32 {
        self.creatures.iter().filter(|c| c.base == id && !c.settler()).count() as i32
    }

    /// How pleasant the spot around a building is (gardens up, noisy industry down).
    pub fn pleasant(&self, b: &Building) -> i32 {
        let (bw, bh) = b.kind.size();
        let mut p = 0;
        for o in &self.buildings {
            if o.id == b.id {
                continue;
            }
            let a = match o.kind {
                Kind::Camp => o.source.map(|s| camp_aura(self.level.sources[s].good)).unwrap_or(0),
                k => k.aura(),
            };
            if a == 0 {
                continue;
            }
            let (ow, oh) = o.kind.size();
            let gap_x = (o.x - (b.x + bw - 1)).max(b.x - (o.x + ow - 1)).max(0);
            let gap_y = (o.y - (b.y + bh - 1)).max(b.y - (o.y + oh - 1)).max(0);
            if gap_x <= AURA && gap_y <= AURA {
                p += a;
            }
        }
        p
    }

    /// Does this home have everything a tier asks for?
    pub fn meets(&self, b: &Building, tier: usize) -> bool {
        let n = &TIER_NEEDS[tier];
        let foods = FOODS.iter().filter(|f| b.stock[f.index()] > 0).count() as i32;
        foods >= n.foods && n.goods.iter().all(|g| b.stock[g.index()] > 0) && self.pleasant(b) >= n.pleasant
    }

    /// The first thing a home is missing for a tier.
    pub fn missing(&self, b: &Building, tier: usize) -> Option<Note> {
        let n = &TIER_NEEDS[tier];
        let foods = FOODS.iter().filter(|f| b.stock[f.index()] > 0).count() as i32;
        if foods == 0 {
            return Some(Note::Hungry);
        }
        if foods < n.foods {
            let have: Vec<Good> = FOODS.iter().copied().filter(|f| b.stock[f.index()] > 0).collect();
            let other = FOODS.iter().copied().find(|f| !have.contains(f) && self.level.goods().contains(f)).unwrap_or(Good::Nut);
            return Some(Note::Lacks(other));
        }
        for g in n.goods {
            if b.stock[g.index()] == 0 {
                return Some(Note::Lacks(*g));
            }
        }
        None
    }

    /// Can this site's homes ever reach a tier (are its goods on the map)?
    pub fn tier_possible(&self, tier: usize) -> bool {
        let goods = self.level.goods();
        let n = &TIER_NEEDS[tier];
        let foods = FOODS.iter().filter(|f| goods.contains(f)).count() as i32;
        foods >= n.foods && n.goods.iter().all(|g| goods.contains(g)) && (n.pleasant <= 0 || self.level.kinds.contains(&Kind::Garden))
    }

    /// How many more of a good this place wants (after what's on the way).
    pub fn want(&self, p: Place, g: Good, from_store: bool) -> i32 {
        let gi = g.index();
        match p {
            Place::Fortress => {
                let Some(st) = self.level.stages.get(self.stage) else { return 0 };
                let need = st.need.iter().find(|(n, _)| *n == g).map(|(_, c)| *c).unwrap_or(0);
                need - self.fort_stock[gi] - self.fort_in[gi]
            }
            Place::Entrance => 0,
            Place::B(id) => {
                let Some(b) = self.building(id) else { return 0 };
                let have = b.stock[gi] + b.incoming[gi];
                match b.kind {
                    Kind::Burrow | Kind::Nest | Kind::Lodge => {
                        if g.food() {
                            return HOME_FOOD_CAP - have;
                        }
                        let now = TIER_NEEDS[b.tier].goods.contains(&g);
                        let next = b.tier + 1 < TIERS && TIER_NEEDS[b.tier + 1].goods.contains(&g) && self.pleasant(b) >= TIER_NEEDS[b.tier + 1].pleasant;
                        if now || next {
                            HOME_GOOD_CAP - have
                        } else {
                            0
                        }
                    }
                    Kind::Store => {
                        let relay = b.orders.iter().any(|o| *o == Order::Get);
                        if from_store || b.orders[gi] == Order::Refuse || (relay && b.orders[gi] != Order::Get) {
                            0
                        } else {
                            (STORE_CAP - b.total() - b.total_in()).min(STORE_PER_GOOD - have)
                        }
                    }
                    k => match k.recipe() {
                        Some(r) if r.inputs.iter().any(|(i, _)| *i == g) => SHOP_IN_CAP - have,
                        _ => 0,
                    },
                }
            }
        }
    }

    // ----- Actions --------------------------------------------------------------------------

    pub fn can_afford(&self, cost: i32) -> bool {
        self.seeds() >= cost
    }

    fn pay(&mut self, cost: i32) {
        self.seeds_tenths -= cost * 10;
        self.spent += cost;
    }

    /// Checks an action without doing it: Ok(cost) or why not.
    pub fn check(&self, a: &Action) -> Result<i32, String> {
        match a {
            Action::Build { kind, x, y, .. } => {
                if !self.level.kinds.contains(kind) {
                    return Err(format!("No {} here.", kind.title().to_lowercase()));
                }
                if let Some(s) = kind.resident() {
                    if !self.level.species.contains(&s) {
                        return Err(format!("No {} at this site.", s.plural()));
                    }
                }
                self.world.can_build(*kind, *x, *y).map_err(|e| e.to_string())?;
                let cost = kind.cost();
                if !self.can_afford(cost) {
                    return Err(format!("It costs {cost} seeds."));
                }
                Ok(cost)
            }
            Action::Infra { infra, x, y } => {
                if !self.level.infra.contains(infra) {
                    return Err(format!("No {}s here.", infra.title().to_lowercase()));
                }
                let cost = match infra {
                    Infra::Spring => self.world.can_spring(*x, *y).map(|_| infra.cost()),
                    Infra::Plank => self.world.can_plank(*x, *y).map(|_| infra.cost()),
                    Infra::Ladder => self.world.can_ladder(*x, *y).map(|len| len * infra.cost()),
                }
                .map_err(|e| e.to_string())?;
                if !self.can_afford(cost) {
                    return Err(format!("It costs {cost} seeds."));
                }
                Ok(cost)
            }
            Action::Demolish { x, y } => {
                if self.building_at(*x, *y).is_some() || self.world.infra_at(*x, *y).is_some() {
                    Ok(0)
                } else {
                    Err("Nothing of yours there.".to_string())
                }
            }
            Action::SetSpecies { id, .. } | Action::SetOrder { id, .. } | Action::Pause { id, .. } => {
                if self.building(*id).is_some() {
                    Ok(0)
                } else {
                    Err("It's gone.".to_string())
                }
            }
        }
    }

    pub fn apply(&mut self, a: &Action) -> Result<(), String> {
        let cost = self.check(a)?;
        match a {
            Action::Build { kind, x, y, species } => {
                self.pay(cost);
                let id = self.next_id;
                self.next_id += 1;
                let source = if *kind == Kind::Camp { self.level.source_at(*x, *y) } else { None };
                self.world.place_building(id, *kind, *x, *y);
                let mut orders = [Order::Accept; NGOODS];
                if *kind == Kind::Store {
                    for g in GOODS {
                        if !self.level.goods().contains(&g) {
                            orders[g.index()] = Order::Refuse;
                        }
                    }
                }
                self.buildings.push(Building {
                    id,
                    kind: *kind,
                    x: *x,
                    y: *y,
                    source,
                    species: kind.resident().unwrap_or(*species),
                    stock: [0; NGOODS],
                    incoming: [0; NGOODS],
                    orders,
                    paused: false,
                    tier: 0,
                    residents: 0,
                    coming: 0,
                    settle_cd: 0,
                    lack: 0,
                    ready: 0,
                    eat_acc: 0,
                    wear_acc: 0,
                    progress: 0,
                    note: Note::Fine,
                });
                match self.level.challenge {
                    Challenge::MaxStores(n) if *kind == Kind::Store && self.count(Kind::Store) > n => self.broke_challenge = true,
                    Challenge::MaxHomes(n) if kind.is_home() && self.buildings.iter().filter(|b| b.kind.is_home()).count() as i32 > n => self.broke_challenge = true,
                    Challenge::NoSpecies(s) if kind.resident() == Some(s) => self.broke_challenge = true,
                    _ => {}
                }
            }
            Action::Infra { infra, x, y } => {
                self.pay(cost);
                match infra {
                    Infra::Spring => self.world.add_spring(*x, *y),
                    Infra::Plank => self.world.add_plank(*x, *y),
                    Infra::Ladder => {
                        self.world.add_ladder(*x, *y);
                    }
                }
                if self.level.challenge == Challenge::NoInfra {
                    self.broke_challenge = true;
                }
                self.reroute();
            }
            Action::Demolish { x, y } => {
                if let Some(id) = self.building_at(*x, *y).map(|b| b.id) {
                    self.demolish(id);
                } else {
                    let level = self.level.clone();
                    if let Some(c) = self.world.remove_infra(*x, *y, &level) {
                        self.seeds_tenths += c * 5;
                        self.reroute();
                    }
                }
            }
            Action::SetSpecies { id, species } => {
                if let Some(i) = self.bidx(*id) {
                    if self.buildings[i].kind.slots() > 0 && self.buildings[i].species != *species {
                        self.buildings[i].species = *species;
                        self.fire_all(*id);
                    }
                }
            }
            Action::SetOrder { id, good, order } => {
                if let Some(i) = self.bidx(*id) {
                    self.buildings[i].orders[good.index()] = *order;
                }
            }
            Action::Pause { id, paused } => {
                if let Some(i) = self.bidx(*id) {
                    self.buildings[i].paused = *paused;
                    if *paused {
                        self.fire_all(*id);
                    }
                }
            }
        }
        Ok(())
    }

    pub fn count(&self, k: Kind) -> i32 {
        self.buildings.iter().filter(|b| b.kind == k).count() as i32
    }

    fn demolish(&mut self, id: Id) {
        let Some(i) = self.bidx(id) else { return };
        let b = self.buildings[i].clone();
        self.seeds_tenths += b.kind.cost() * 5;
        // Its workers leave, and anyone bringing it something drops the errand.
        self.fire_all(id);
        let settlers: Vec<u32> = self.creatures.iter().filter(|c| c.base == id).map(|c| c.uid).collect();
        for u in settlers {
            self.remove_creature(u);
        }
        let headed: Vec<u32> = self.creatures.iter().filter(|c| matches!(c.job, Job::Carry(Place::B(t)) | Job::Drop(_, Place::B(t)) if t == id) || c.job == Job::Fetch(id)).map(|c| c.uid).collect();
        for u in headed {
            if let Some(c) = self.creatures.iter_mut().find(|c| c.uid == u) {
                match c.job {
                    Job::Fetch(_) => {
                        c.job = Job::Return;
                        let (g, base) = (c.cargo.take(), c.base);
                        if let Some(g) = g {
                            self.unreserve(Place::B(base), g);
                        }
                    }
                    _ => {
                        c.job = Job::Wait(1);
                    }
                }
            }
            self.walk_home(u);
        }
        let i = self.bidx(id).unwrap();
        self.buildings.remove(i);
        self.world.remove_building(b.kind, b.x, b.y, b.source);
        self.fields.borrow_mut().retain(|k, _| k.0 != Place::B(id));
        self.balance_labour();
    }

    /// Paths depend on the terrain: after it changes, walkers re-plan from where they stand.
    fn reroute(&mut self) {
        let uids: Vec<u32> = self.creatures.iter().filter(|c| c.walking()).map(|c| c.uid).collect();
        for u in uids {
            let Some(c) = self.creatures.iter().find(|c| c.uid == u) else { continue };
            let (x, y, s, load) = (c.x, c.y, c.species, c.cargo.map(Load::of).unwrap_or(Load::EMPTY));
            let goal = match c.job {
                Job::Hire | Job::Return | Job::Bring => Place::B(c.base),
                Job::Settle => Place::B(c.base),
                Job::Carry(p) => p,
                Job::Fetch(src) => Place::B(src),
                _ => continue,
            };
            let goals = self.access(goal);
            let p = path::find(&self.world, s, load, &[(x, y)], &goals);
            let c = self.creatures.iter_mut().find(|c| c.uid == u).unwrap();
            match p {
                Some(p) => {
                    c.path = p;
                    c.step = 0;
                    c.prog = 0;
                }
                None => {
                    // Stranded: it gives up and the colony loses the errand.
                    let uid = c.uid;
                    self.remove_creature(uid);
                }
            }
        }
    }

    // ----- Creatures ------------------------------------------------------------------------

    fn spawn(&mut self, s: Species, base: Id, job: Job, path: Path) {
        let (x, y) = path.start;
        let uid = self.next_uid;
        self.next_uid += 1;
        self.creatures.push(Creature { uid, species: s, base, cargo: None, job, path, step: 0, prog: 0, x, y, facing: 1 });
    }

    /// Takes a creature off the map, undoing whatever it had reserved.
    fn remove_creature(&mut self, uid: u32) {
        let Some(i) = self.creatures.iter().position(|c| c.uid == uid) else { return };
        let c = self.creatures.remove(i);
        match c.job {
            Job::Carry(p) | Job::Drop(_, p) => {
                if let Some(g) = c.cargo {
                    self.unreserve(p, g);
                }
            }
            Job::Fetch(src) => {
                // The good it was going for goes back on the shelf; home stops expecting it.
                if let Some(g) = self.fetch_good(&c) {
                    if let Some(si) = self.bidx(src) {
                        self.buildings[si].stock[g.index()] += 1;
                    }
                    self.unreserve(Place::B(c.base), g);
                }
            }
            Job::Bring => {
                if let Some(g) = c.cargo {
                    self.unreserve(Place::B(c.base), g);
                }
            }
            Job::Settle => {
                if let Some(bi) = self.bidx(c.base) {
                    self.buildings[bi].coming = (self.buildings[bi].coming - 1).max(0);
                }
            }
            _ => {}
        }
    }

    /// A store carrier's fetch errand remembers its good in `cargo` until pickup (then holds it).
    fn fetch_good(&self, c: &Creature) -> Option<Good> {
        c.cargo
    }

    fn unreserve(&mut self, p: Place, g: Good) {
        match p {
            Place::Fortress => self.fort_in[g.index()] = (self.fort_in[g.index()] - 1).max(0),
            Place::B(id) => {
                if let Some(i) = self.bidx(id) {
                    self.buildings[i].incoming[g.index()] = (self.buildings[i].incoming[g.index()] - 1).max(0);
                }
            }
            Place::Entrance => {}
        }
    }

    fn fire_all(&mut self, id: Id) {
        let uids: Vec<u32> = self.creatures.iter().filter(|c| c.base == id && !c.settler()).map(|c| c.uid).collect();
        for u in uids {
            self.remove_creature(u);
        }
    }

    /// Sends a creature back to its base from where it stands (dropping any errand).
    fn walk_home(&mut self, uid: u32) {
        let Some(c) = self.creatures.iter().find(|c| c.uid == uid) else { return };
        let load = c.cargo.map(Load::of).unwrap_or(Load::EMPTY);
        let goals = self.access(Place::B(c.base));
        let p = path::find(&self.world, c.species, load, &[(c.x, c.y)], &goals);
        match p {
            Some(p) => {
                let c = self.creatures.iter_mut().find(|c| c.uid == uid).unwrap();
                c.path = p;
                c.step = 0;
                c.prog = 0;
                if c.cargo.is_some() {
                    c.job = Job::Bring;
                    // It is bringing it home: home expects it.
                    let (g, base) = (c.cargo.unwrap(), c.base);
                    if let Some(bi) = self.bidx(base) {
                        if self.buildings[bi].kind == Kind::Store {
                            self.buildings[bi].incoming[g.index()] += 1;
                        } else {
                            let c = self.creatures.iter_mut().find(|c| c.uid == uid).unwrap();
                            c.job = Job::Return;
                        }
                    }
                } else {
                    c.job = Job::Return;
                }
            }
            None => self.remove_creature(uid),
        }
    }

    // ----- The tick -------------------------------------------------------------------------

    pub fn step(&mut self) {
        if self.won.is_some() {
            return;
        }
        self.tick += 1;
        self.grow_sources();
        self.homes();
        if self.tick % DAY == 0 {
            self.taxes();
        }
        if self.tick % 10 == 1 {
            self.balance_labour();
        }
        self.workshops();
        self.move_creatures();
        self.check_goals();
    }

    fn grow_sources(&mut self) {
        let season = self.season();
        for (i, s) in self.level.sources.iter().enumerate() {
            if s.infinite() || s.regrow <= 0 || self.src_stock[i] >= s.max {
                continue;
            }
            self.src_grow[i] += s.good.season_growth(season);
            if self.src_grow[i] >= s.regrow {
                self.src_grow[i] = 0;
                self.src_stock[i] += 1;
            }
        }
    }

    fn taxes(&mut self) {
        let t: i32 = self.buildings.iter().filter(|b| b.kind.is_home()).map(|b| b.residents * TAX_TENTHS[b.tier]).sum();
        self.seeds_tenths += t;
    }

    fn homes(&mut self) {
        for i in 0..self.buildings.len() {
            if !self.buildings[i].kind.is_home() {
                continue;
            }
            let id = self.buildings[i].id;
            // Eating and wear.
            {
                let b = &mut self.buildings[i];
                b.eat_acc += b.residents;
                while b.eat_acc >= EAT_TICKS {
                    b.eat_acc -= EAT_TICKS;
                    // Eat whichever food it has most of (keeps variety on the shelf).
                    if let Some(f) = FOODS.iter().copied().filter(|f| b.stock[f.index()] > 0).max_by_key(|f| (b.stock[f.index()], std::cmp::Reverse(f.index()))) {
                        b.stock[f.index()] -= 1;
                    }
                }
                b.wear_acc += 1;
                if b.wear_acc >= WEAR_TICKS {
                    b.wear_acc = 0;
                    for g in TIER_NEEDS[b.tier].goods {
                        if b.stock[g.index()] > 0 {
                            b.stock[g.index()] -= 1;
                        }
                    }
                }
            }
            let b = self.buildings[i].clone();
            let ok = b.residents == 0 || self.meets(&b, b.tier);
            let up = b.residents > 0 && b.tier + 1 < TIERS && b.residents >= residents(b.species, b.tier) && self.meets(&b, b.tier + 1);
            let mut ev = None;
            {
                let b = &mut self.buildings[i];
                if ok {
                    b.lack = 0;
                } else {
                    b.lack += 1;
                }
                if up {
                    b.ready += 1;
                    if b.ready >= SETTLE {
                        b.ready = 0;
                        b.tier += 1;
                        ev = Some(Event::TierUp(id, b.tier));
                    }
                } else {
                    b.ready = 0;
                }
                if b.lack >= GRACE {
                    b.lack = GRACE / 2;
                    if b.tier > 0 {
                        b.tier -= 1;
                        let cap = residents(b.species, b.tier);
                        b.residents = b.residents.min(cap);
                        ev = Some(Event::TierDown(id, b.tier));
                    } else if b.residents > 0 {
                        b.residents -= 1;
                        ev = Some(Event::Left(id));
                    }
                }
            }
            if let Some(e) = ev {
                if matches!(e, Event::TierDown(..) | Event::Left(_)) {
                    self.slipped += 1;
                    if self.level.challenge == Challenge::NoHunger {
                        self.broke_challenge = true;
                    }
                }
                self.events.push(e);
            }
            // Settlers.
            let b = &self.buildings[i];
            let cap = residents(b.species, b.tier);
            let has_food = FOODS.iter().any(|f| b.stock[f.index()] > 0);
            let wants = b.residents + b.coming < cap && (has_food || (b.residents == 0 && b.coming == 0));
            if b.settle_cd > 0 {
                self.buildings[i].settle_cd -= 1;
            } else if wants {
                let s = b.species;
                let f = self.field(Place::Entrance, s, Load::EMPTY);
                let goals = access_of(b.kind, b.x, b.y);
                let p = f.best(&self.world, &goals).and_then(|(_, (gx, gy))| f.path_to(&self.world, s, Load::EMPTY, gx, gy));
                let b = &mut self.buildings[i];
                b.settle_cd = SETTLER_GAP;
                match p {
                    Some(p) => {
                        b.coming += 1;
                        if b.note == Note::NoSettlers {
                            b.note = Note::Fine;
                        }
                        self.spawn(s, id, Job::Settle, p);
                    }
                    None => b.note = Note::NoSettlers,
                }
            }
            // Status.
            let b = self.buildings[i].clone();
            if b.note != Note::NoSettlers || b.residents > 0 {
                let note = if b.residents == 0 && b.coming == 0 { Note::Fine } else { self.missing(&b, b.tier).unwrap_or(Note::Fine) };
                self.buildings[i].note = note;
            }
        }
        let p = self.pop();
        self.peak_pop = self.peak_pop.max(p);
    }

    /// Hires and fires so each species works no more than it has residents.
    fn balance_labour(&mut self) {
        for s in SPECIES {
            let pool = self.pop_of(s);
            let mut over = self.employed(s) - pool;
            while over > 0 {
                // Fire from the newest workplace first, idle workers before busy ones.
                let pick = self
                    .creatures
                    .iter()
                    .filter(|c| c.species == s && !c.settler())
                    .max_by_key(|c| (c.base, matches!(c.job, Job::Idle | Job::Hire), c.uid))
                    .map(|c| c.uid);
                match pick {
                    Some(u) => self.remove_creature(u),
                    None => break,
                }
                over -= 1;
            }
        }
        let mut free: [i32; 3] = [0; 3];
        for s in SPECIES {
            free[s.index()] = self.pop_of(s) - self.employed(s);
        }
        let ids: Vec<Id> = self.buildings.iter().filter(|b| b.kind.slots() > 0).map(|b| b.id).collect();
        for id in ids {
            let b = self.building(id).unwrap().clone();
            if b.paused {
                self.set_note(id, Note::Paused);
                continue;
            }
            let have = self.workers_at(id);
            if have >= self.slots(&b) {
                if matches!(b.note, Note::NoWorkers(_) | Note::NoReach(_) | Note::Paused) {
                    self.set_note(id, Note::Fine);
                }
                continue;
            }
            let s = b.species;
            if free[s.index()] <= 0 {
                if have == 0 {
                    self.set_note(id, Note::NoWorkers(s));
                }
                continue;
            }
            // The nearest home of that species with someone in it that can walk here.
            let goals = access_of(b.kind, b.x, b.y);
            let mut best: Option<(i32, Id)> = None;
            for h in self.buildings.iter().filter(|h| h.kind.is_home() && h.species == s && h.residents > 0) {
                let f = self.field(Place::B(h.id), s, Load::EMPTY);
                if let Some((d, _)) = f.best(&self.world, &goals) {
                    if best.map_or(true, |(bd, _)| d < bd) {
                        best = Some((d, h.id));
                    }
                }
            }
            let Some((_, home)) = best else {
                self.set_note(id, Note::NoReach(s));
                continue;
            };
            let f = self.field(Place::B(home), s, Load::EMPTY);
            let Some((_, (gx, gy))) = f.best(&self.world, &goals) else { continue };
            let Some(p) = f.path_to(&self.world, s, Load::EMPTY, gx, gy) else { continue };
            free[s.index()] -= 1;
            self.spawn(s, id, Job::Hire, p);
            if matches!(b.note, Note::NoWorkers(_) | Note::NoReach(_) | Note::Paused) {
                self.set_note(id, Note::Fine);
            }
        }
    }

    fn set_note(&mut self, id: Id, n: Note) {
        if let Some(i) = self.bidx(id) {
            self.buildings[i].note = n;
        }
    }

    fn workshops(&mut self) {
        for i in 0..self.buildings.len() {
            let Some(r) = self.buildings[i].kind.recipe() else { continue };
            let id = self.buildings[i].id;
            let present = self.creatures.iter().any(|c| c.base == id && !c.settler() && c.job != Job::Hire);
            let b = &mut self.buildings[i];
            let has_inputs = r.inputs.iter().all(|(g, n)| b.stock[g.index()] >= *n);
            if b.stock[r.output.index()] >= SHOP_OUT_CAP {
                continue;
            }
            if !has_inputs {
                if b.progress == 0 {
                    if let Some((g, _)) = r.inputs.iter().find(|(g, n)| b.stock[g.index()] < *n) {
                        if !matches!(b.note, Note::NoWorkers(_) | Note::NoReach(_) | Note::Paused) {
                            b.note = Note::NeedInput(*g);
                        }
                    }
                }
                continue;
            }
            if matches!(b.note, Note::NeedInput(_)) {
                b.note = Note::Fine;
            }
            if !present {
                continue;
            }
            b.progress += 1;
            if b.progress >= r.ticks {
                b.progress = 0;
                for (g, n) in r.inputs {
                    b.stock[g.index()] -= n;
                }
                b.stock[r.output.index()] += 1;
                if matches!(b.note, Note::NeedInput(_)) {
                    b.note = Note::Fine;
                }
                self.events.push(Event::Crafted(id));
            }
        }
    }

    fn move_creatures(&mut self) {
        let n = self.creatures.len();
        let mut i = 0;
        let mut done: Vec<u32> = Vec::new();
        while i < n.min(self.creatures.len()) {
            let uid = self.creatures[i].uid;
            if self.creatures[i].walking() {
                let c = &mut self.creatures[i];
                if c.step < c.path.steps.len() {
                    let st = c.path.steps[c.step];
                    c.prog += 1;
                    if c.prog >= st.t {
                        if st.x != c.x {
                            c.facing = if st.x > c.x { 1 } else { -1 };
                        }
                        c.x = st.x;
                        c.y = st.y;
                        c.step += 1;
                        c.prog = 0;
                    }
                }
                if c.step >= c.path.steps.len() {
                    done.push(uid);
                }
            } else {
                done.push(uid);
            }
            i += 1;
        }
        for uid in done {
            self.act(uid);
        }
    }

    /// A creature that has arrived somewhere or is standing at work decides what's next.
    fn act(&mut self, uid: u32) {
        let Some(ci) = self.creatures.iter().position(|c| c.uid == uid) else { return };
        let c = self.creatures[ci].clone();
        match c.job {
            Job::Settle => {
                self.creatures.remove(ci);
                if let Some(bi) = self.bidx(c.base) {
                    let b = &mut self.buildings[bi];
                    b.coming = (b.coming - 1).max(0);
                    if b.residents < residents(b.species, b.tier) {
                        b.residents += 1;
                        self.events.push(Event::Settled(c.base));
                    }
                }
            }
            Job::Hire | Job::Return => {
                self.creatures[ci].job = Job::Idle;
            }
            Job::Carry(p) => {
                self.creatures[ci].job = Job::Drop(DROP_TICKS, p);
            }
            Job::Drop(t, p) => {
                if t > 1 {
                    self.creatures[ci].job = Job::Drop(t - 1, p);
                    return;
                }
                let g = c.cargo.unwrap();
                self.unreserve(p, g);
                self.receive(p, g, c.base);
                self.creatures[ci].cargo = None;
                self.go_back(uid, p);
            }
            Job::Fetch(src) => {
                // Take the good (it was set aside for us) and bring it home.
                let g = c.cargo.unwrap();
                let base = c.base;
                let f = self.field(Place::B(src), c.species, Load::of(g));
                let p = f.best(&self.world, &self.access(Place::B(base))).and_then(|(_, (gx, gy))| f.path_to(&self.world, c.species, Load::of(g), gx, gy));
                match p {
                    Some(p) => {
                        let c = &mut self.creatures[ci];
                        c.path = p;
                        c.step = 0;
                        c.prog = 0;
                        c.x = c.path.start.0;
                        c.y = c.path.start.1;
                        c.job = Job::Bring;
                    }
                    None => {
                        self.remove_creature(uid);
                    }
                }
            }
            Job::Bring => {
                let g = c.cargo.unwrap();
                self.unreserve(Place::B(c.base), g);
                if let Some(bi) = self.bidx(c.base) {
                    self.buildings[bi].stock[g.index()] += 1;
                }
                let c = &mut self.creatures[ci];
                c.cargo = None;
                c.job = Job::Idle;
            }
            Job::Harvest(t) => {
                if t > 1 {
                    self.creatures[ci].job = Job::Harvest(t - 1);
                    return;
                }
                let g = self.building(c.base).and_then(|b| b.source).map(|s| self.level.sources[s].good);
                self.creatures[ci].cargo = g;
                self.creatures[ci].job = Job::Idle;
                self.dispatch(uid);
            }
            Job::Wait(t) => {
                if t > 1 {
                    self.creatures[ci].job = Job::Wait(t - 1);
                    return;
                }
                self.creatures[ci].job = Job::Idle;
                self.dispatch(uid);
            }
            Job::Idle => self.dispatch(uid),
        }
    }

    fn receive(&mut self, p: Place, g: Good, _from: Id) {
        match p {
            Place::Fortress => {
                self.fort_stock[g.index()] += 1;
                self.events.push(Event::Delivered(p, g));
                self.check_stage();
            }
            Place::B(id) => {
                if let Some(i) = self.bidx(id) {
                    self.buildings[i].stock[g.index()] += 1;
                }
                self.events.push(Event::Delivered(p, g));
            }
            Place::Entrance => {}
        }
    }

    fn check_stage(&mut self) {
        while let Some(st) = self.level.stages.get(self.stage) {
            if st.need.iter().all(|(g, n)| self.fort_stock[g.index()] >= *n) {
                for (g, n) in st.need.clone() {
                    self.fort_stock[g.index()] -= n;
                }
                self.events.push(Event::Stage(self.stage));
                self.stage += 1;
            } else {
                break;
            }
        }
    }

    pub fn fortress_done(&self) -> bool {
        self.stage >= self.level.stages.len()
    }

    fn go_back(&mut self, uid: u32, from: Place) {
        let Some(ci) = self.creatures.iter().position(|c| c.uid == uid) else { return };
        let c = &self.creatures[ci];
        let (s, base) = (c.species, c.base);
        let f = self.field(from, s, Load::EMPTY);
        let goals = self.access(Place::B(base));
        let p = f.best(&self.world, &goals).and_then(|(_, (gx, gy))| f.path_to(&self.world, s, Load::EMPTY, gx, gy));
        match p {
            Some(p) => {
                let c = &mut self.creatures[ci];
                c.x = p.start.0;
                c.y = p.start.1;
                c.path = p;
                c.step = 0;
                c.prog = 0;
                c.job = Job::Return;
            }
            None => self.remove_creature(uid),
        }
    }

    /// Does anywhere at all (reachable or not) want this good?
    pub fn anyone_wants(&self, from: Place, g: Good, from_store: bool) -> bool {
        if self.level.has_fortress() && from != Place::Fortress && self.want(Place::Fortress, g, from_store) > 0 {
            return true;
        }
        self.buildings.iter().any(|b| Place::B(b.id) != from && self.want(Place::B(b.id), g, from_store) > 0)
    }

    /// Why a good can't go anywhere: nobody needs it (Stocked), or the nearest place that wants it,
    /// with the ticks it takes this species to carry one there.
    pub fn why_stuck(&self, from: Place, s: Species, g: Good, from_store: bool) -> Note {
        let f = self.field(from, s, Load::of(g));
        let mut places: Vec<Place> = self.buildings.iter().map(|b| Place::B(b.id)).collect();
        if self.level.has_fortress() {
            places.push(Place::Fortress);
        }
        let mut nearest: Option<(i32, Place)> = None;
        let mut any: Option<Place> = None;
        for p in places {
            if p == from || self.want(p, g, from_store) <= 0 {
                continue;
            }
            any.get_or_insert(p);
            if let Some((d, _)) = f.best(&self.world, &self.access(p)) {
                if nearest.map_or(true, |(nd, _)| d < nd) {
                    nearest = Some((d, p));
                }
            }
        }
        match (nearest, any) {
            (Some((d, p)), _) => Note::Stuck(g, p, d),
            (None, Some(p)) => Note::Stuck(g, p, FAR),
            (None, None) => Note::Stocked(g),
        }
    }

    /// The best place within reach to take a good to: consumers first, then stores.
    fn best_drop(&self, from: Place, s: Species, g: Good, from_store: bool) -> Option<(i32, Place, (i32, i32))> {
        let load = Load::of(g);
        let f = self.field(from, s, load);
        let mut best: Option<(i32, Place, (i32, i32))> = None;
        let mut consider = |score: i32, p: Place, at: (i32, i32), best: &mut Option<(i32, Place, (i32, i32))>| {
            if best.map_or(true, |(bs, _, _)| score < bs) {
                *best = Some((score, p, at));
            }
        };
        let from_access = self.access(from);
        let back_ok = |p: Place| -> bool {
            let bf = self.field(p, s, Load::EMPTY);
            bf.best(&self.world, &from_access).is_some()
        };
        let mut places: Vec<Place> = self.buildings.iter().filter(|b| Place::B(b.id) != from).map(|b| Place::B(b.id)).collect();
        if self.level.has_fortress() {
            places.push(Place::Fortress);
        }
        for p in places {
            let w = self.want(p, g, from_store);
            if w <= 0 {
                continue;
            }
            let (class, penalty) = match p {
                Place::B(id) => {
                    let b = self.building(id).unwrap();
                    match b.kind {
                        Kind::Store => (1, b.total() * 2),
                        k if k.is_home() => (0, (b.stock[g.index()] + b.incoming[g.index()]) * 12),
                        _ => (0, (b.stock[g.index()] + b.incoming[g.index()]) * 6),
                    }
                }
                _ => (0, 0),
            };
            let Some((d, at)) = f.best(&self.world, &self.access(p)) else { continue };
            if d > RANGE {
                continue;
            }
            let score = class * 100_000 + d + penalty;
            if best.map_or(false, |(bs, _, _)| score >= bs) {
                continue;
            }
            if !back_ok(p) {
                continue;
            }
            consider(score, p, at, &mut best);
        }
        best
    }

    fn start_carry(&mut self, uid: u32, from: Place, g: Good, to: Place, at: (i32, i32)) -> bool {
        let Some(ci) = self.creatures.iter().position(|c| c.uid == uid) else { return false };
        let s = self.creatures[ci].species;
        let f = self.field(from, s, Load::of(g));
        let Some(p) = f.path_to(&self.world, s, Load::of(g), at.0, at.1) else { return false };
        match to {
            Place::Fortress => self.fort_in[g.index()] += 1,
            Place::B(id) => {
                if let Some(i) = self.bidx(id) {
                    self.buildings[i].incoming[g.index()] += 1;
                }
            }
            Place::Entrance => {}
        }
        let c = &mut self.creatures[ci];
        c.cargo = Some(g);
        c.x = p.start.0;
        c.y = p.start.1;
        c.path = p;
        c.step = 0;
        c.prog = 0;
        c.job = Job::Carry(to);
        true
    }

    /// An idle worker at its workplace looks for something to do.
    fn dispatch(&mut self, uid: u32) {
        let Some(ci) = self.creatures.iter().position(|c| c.uid == uid) else { return };
        let c = self.creatures[ci].clone();
        let Some(b) = self.building(c.base).cloned() else {
            self.creatures.remove(ci);
            return;
        };
        let here = Place::B(b.id);
        // Holding something (a fresh harvest)? Take it somewhere.
        if let Some(g) = c.cargo {
            if let Some((_, to, at)) = self.best_drop(here, c.species, g, false) {
                if self.start_carry(uid, here, g, to, at) {
                    if matches!(self.building(b.id).map(|b| b.note), Some(Note::Stuck(..) | Note::Stocked(_))) {
                        self.set_note(b.id, Note::Fine);
                    }
                    return;
                }
            }
            let n = self.why_stuck(here, c.species, g, false);
            self.set_note(b.id, n);
            self.creatures[ci].job = Job::Wait(15);
            return;
        }
        match b.kind {
            Kind::Camp => {
                let Some(si) = b.source else { return };
                let src = &self.level.sources[si];
                if src.infinite() || self.src_stock[si] > 0 {
                    if !src.infinite() {
                        self.src_stock[si] -= 1;
                    }
                    self.creatures[ci].job = Job::Harvest(src.good.harvest_ticks());
                    if self.building(b.id).map(|b| b.note) == Some(Note::Bare) {
                        self.set_note(b.id, Note::Fine);
                    }
                } else {
                    self.set_note(b.id, Note::Bare);
                    self.creatures[ci].job = Job::Wait(10);
                }
            }
            Kind::Store => {
                // Supply consumers first, the scarcest-first among what's on the shelf.
                let mut best: Option<(i32, Good, Place, (i32, i32))> = None;
                for g in GOODS {
                    if b.stock[g.index()] <= 0 {
                        continue;
                    }
                    if let Some((sc, to, at)) = self.best_drop(here, c.species, g, true) {
                        if best.map_or(true, |(bs, ..)| sc < bs) {
                            best = Some((sc, g, to, at));
                        }
                    }
                }
                if let Some((_, g, to, at)) = best {
                    if let Some(bi) = self.bidx(b.id) {
                        self.buildings[bi].stock[g.index()] -= 1;
                    }
                    if !self.start_carry(uid, here, g, to, at) {
                        if let Some(bi) = self.bidx(b.id) {
                            self.buildings[bi].stock[g.index()] += 1;
                        }
                        self.creatures[ci].job = Job::Wait(10);
                    }
                    return;
                }
                // Then fetch what it's been told to get.
                if b.total() + b.total_in() < STORE_CAP {
                    for g in GOODS {
                        if b.orders[g.index()] != Order::Get || b.stock[g.index()] + b.incoming[g.index()] >= STORE_PER_GOOD {
                            continue;
                        }
                        if let Some((src, at)) = self.best_fetch(b.id, c.species, g) {
                            let f = self.field(here, c.species, Load::EMPTY);
                            let Some(p) = f.path_to(&self.world, c.species, Load::EMPTY, at.0, at.1) else { continue };
                            let si = self.bidx(src).unwrap();
                            self.buildings[si].stock[g.index()] -= 1;
                            let bi = self.bidx(b.id).unwrap();
                            self.buildings[bi].incoming[g.index()] += 1;
                            let c = &mut self.creatures[ci];
                            c.cargo = Some(g);
                            c.x = p.start.0;
                            c.y = p.start.1;
                            c.path = p;
                            c.step = 0;
                            c.prog = 0;
                            c.job = Job::Fetch(src);
                            return;
                        }
                    }
                }
                self.creatures[ci].job = Job::Wait(10);
            }
            k if k.recipe().is_some() => {
                let r = k.recipe().unwrap();
                let out = r.output;
                if b.stock[out.index()] > 0 {
                    if let Some((_, to, at)) = self.best_drop(here, c.species, out, false) {
                        if let Some(bi) = self.bidx(b.id) {
                            self.buildings[bi].stock[out.index()] -= 1;
                        }
                        if !self.start_carry(uid, here, out, to, at) {
                            if let Some(bi) = self.bidx(b.id) {
                                self.buildings[bi].stock[out.index()] += 1;
                            }
                        }
                        return;
                    } else if b.stock[out.index()] >= SHOP_OUT_CAP {
                        let n = self.why_stuck(here, c.species, out, false);
                        self.set_note(b.id, n);
                    }
                }
                self.creatures[ci].job = Job::Wait(10);
            }
            _ => {
                self.creatures[ci].job = Job::Wait(10);
            }
        }
    }

    /// The nearest other store holding a good, that a carrier can reach empty and bring back
    /// loaded within range.
    fn best_fetch(&self, home: Id, s: Species, g: Good) -> Option<(Id, (i32, i32))> {
        let fe = self.field(Place::B(home), s, Load::EMPTY);
        let mut best: Option<(i32, Id, (i32, i32))> = None;
        for o in &self.buildings {
            let mine = self.building(home).map(|h| h.stock[g.index()] + h.incoming[g.index()]).unwrap_or(0);
            // Another relay only lends what it has clearly more of, so goods flow down a chain and never back.
            if o.id == home || o.kind != Kind::Store || o.stock[g.index()] <= 0 || (o.orders[g.index()] == Order::Get && o.stock[g.index()] <= mine + 1) {
                continue;
            }
            let Some((d, at)) = fe.best(&self.world, &self.access(Place::B(o.id))) else { continue };
            let back = self.trip(Place::B(o.id), Place::B(home), s, Load::of(g));
            if back > RANGE {
                continue;
            }
            let score = d + back;
            if best.map_or(true, |(bs, ..)| score < bs) {
                best = Some((score, o.id, at));
            }
        }
        best.map(|(_, id, at)| (id, at))
    }

    // ----- Goals ----------------------------------------------------------------------------

    pub fn goal_met(&self, g: Goal) -> bool {
        match g {
            Goal::Fortress => self.fortress_done(),
            Goal::Pop(n) => self.pop() >= n,
            Goal::Tier(t, n) => self.homes_at(t) >= n,
        }
    }

    pub fn homes_at(&self, t: usize) -> i32 {
        self.buildings.iter().filter(|b| b.kind.is_home() && b.tier >= t && b.residents > 0).count() as i32
    }

    fn check_goals(&mut self) {
        if self.won.is_none() && !self.level.goals.is_empty() && self.level.goals.iter().all(|g| self.goal_met(*g)) {
            self.won = Some(self.day());
            self.events.push(Event::Won);
        }
    }

    /// Fraction of the fortress built so far, in parts out of `total`.
    pub fn fortress_progress(&self) -> (i32, i32) {
        let total: i32 = self.level.stages.iter().flat_map(|s| s.need.iter().map(|(_, n)| *n)).sum();
        let mut got: i32 = self.level.stages.iter().take(self.stage).flat_map(|s| s.need.iter().map(|(_, n)| *n)).sum();
        if let Some(st) = self.level.stages.get(self.stage) {
            got += st.need.iter().map(|(g, n)| self.fort_stock[g.index()].min(*n)).sum::<i32>();
        }
        (got, total.max(1))
    }
}

impl Action {
    /// One line of a saved run (the tick is written separately).
    pub fn to_line(&self) -> String {
        match self {
            Action::Build { kind, x, y, species } => format!("B {} {x} {y} {}", kind.name(), species.name()),
            Action::Infra { infra, x, y } => format!("I {} {x} {y}", infra.short()),
            Action::Demolish { x, y } => format!("D {x} {y}"),
            Action::SetSpecies { id, species } => format!("S {id} {}", species.name()),
            Action::SetOrder { id, good, order } => format!(
                "O {id} {} {}",
                good.name(),
                match order {
                    Order::Accept => "A",
                    Order::Refuse => "R",
                    Order::Get => "G",
                }
            ),
            Action::Pause { id, paused } => format!("P {id} {}", *paused as u8),
        }
    }

    pub fn from_line(s: &str) -> Option<Action> {
        let t: Vec<&str> = s.split_whitespace().collect();
        let n = |i: usize| t.get(i).and_then(|v| v.parse::<i32>().ok());
        Some(match *t.first()? {
            "B" => Action::Build { kind: Kind::from_name(t.get(1)?)?, x: n(2)?, y: n(3)?, species: Species::from_name(t.get(4)?)? },
            "I" => Action::Infra { infra: Infra::from_name(t.get(1)?)?, x: n(2)?, y: n(3)? },
            "D" => Action::Demolish { x: n(1)?, y: n(2)? },
            "S" => Action::SetSpecies { id: n(1)? as u32, species: Species::from_name(t.get(2)?)? },
            "O" => Action::SetOrder {
                id: n(1)? as u32,
                good: Good::from_name(t.get(2)?)?,
                order: match *t.get(3)? {
                    "R" => Order::Refuse,
                    "G" => Order::Get,
                    _ => Order::Accept,
                },
            },
            "P" => Action::Pause { id: n(1)? as u32, paused: n(2)? != 0 },
            _ => return None,
        })
    }
}

/// A run as text: `tick:action` entries separated by `,`, ending with the tick it stopped at.
pub fn run_to_text(actions: &[(i32, Action)], end: i32) -> String {
    let mut parts: Vec<String> = actions.iter().map(|(t, a)| format!("{t}:{}", a.to_line())).collect();
    parts.push(format!("{end}:END"));
    parts.join(",")
}

/// Rebuilds a city from a saved run: replays every action at its tick, then runs to the end.
pub fn replay(level: &Level, text: &str) -> Option<(City, Vec<(i32, Action)>)> {
    let mut acts: Vec<(i32, Action)> = Vec::new();
    let mut end = 0;
    for part in text.split(',') {
        let (t, a) = part.split_once(':')?;
        let t: i32 = t.trim().parse().ok()?;
        if a == "END" {
            end = t;
        } else {
            acts.push((t, Action::from_line(a)?));
        }
    }
    let mut city = City::new(level);
    let mut i = 0;
    loop {
        while i < acts.len() && acts[i].0 <= city.tick {
            let _ = city.apply(&acts[i].1);
            i += 1;
        }
        if city.tick >= end && i >= acts.len() {
            break;
        }
        if city.won.is_some() || city.tick > end + 10 * DAY {
            break;
        }
        city.step();
        city.events.clear();
    }
    Some((city, acts))
}
