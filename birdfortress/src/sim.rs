//! The deterministic colony simulation. Routes are planned once (their paths depend only on
//! terrain, species and load), then creatures shuttle: wait at the pickup, take one good, carry
//! it, wait for the dock, hand it over, walk back empty.

use crate::layout::{Layout, NodeRef};
use crate::level::Level;
use crate::path::{self, Path};
use crate::rules::*;
use crate::world::{layout_cost, World};

pub const MAX_TICKS: i32 = 20_000;

#[derive(Clone, Debug)]
pub enum RouteProblem {
    /// The pickup never offers this cargo.
    NoOffer,
    /// The drop-off never takes this cargo.
    NoTake,
    /// No way there with the load.
    NoWayThere,
    /// No way back empty.
    NoWayBack,
}

impl RouteProblem {
    pub fn text(&self) -> &'static str {
        match self {
            RouteProblem::NoOffer => "NOTHING TO PICK UP THERE",
            RouteProblem::NoTake => "THE DROP-OFF DOESN'T WANT IT",
            RouteProblem::NoWayThere => "CAN'T GET THERE CARRYING IT",
            RouteProblem::NoWayBack => "CAN'T GET BACK EMPTY",
        }
    }
}

#[derive(Clone, Debug)]
pub struct RoutePlan {
    pub loaded: Path,
    pub empty: Path,
    pub problem: Option<RouteProblem>,
}

impl RoutePlan {
    /// Ticks for one full trip, docks included.
    pub fn cycle(&self) -> i32 {
        self.loaded.ticks + self.empty.ticks + PICKUP_TICKS + DROP_TICKS
    }
}

pub fn offers(level: &Level, layout: &Layout, n: NodeRef, g: Good) -> bool {
    match n {
        NodeRef::Source(i) => level.sources.get(i).is_some_and(|s| s.good == g),
        NodeRef::Fortress => false,
        NodeRef::Building(id) => match layout.building(id).map(|b| b.kind) {
            Some(Kind::Depot) => true,
            Some(k) => k.recipe().is_some_and(|r| r.output == g),
            None => false,
        },
    }
}

pub fn takes(level: &Level, layout: &Layout, n: NodeRef, g: Good) -> bool {
    match n {
        NodeRef::Source(_) => false,
        NodeRef::Fortress => level.need_of(g) > 0,
        NodeRef::Building(id) => match layout.building(id).map(|b| b.kind) {
            Some(Kind::Depot) => true,
            Some(k) => k.recipe().is_some_and(|r| r.inputs.iter().any(|(i, _)| *i == g)),
            None => false,
        },
    }
}

/// Cargo choices for a route between two nodes, most useful first.
pub fn cargo_options(level: &Level, layout: &Layout, from: NodeRef, to: NodeRef) -> Vec<Good> {
    let goods = level.goods();
    let mut v: Vec<Good> = goods.into_iter().filter(|&g| offers(level, layout, from, g) && takes(level, layout, to, g)).collect();
    // Prefer what the fortress needs, then raw goods the pickup actually produces.
    v.sort_by_key(|&g| (level.need_of(g) == 0, g));
    v
}

pub fn plan_route(level: &Level, layout: &Layout, world: &World, r: &crate::layout::Route) -> RoutePlan {
    let fail = |p| RoutePlan { loaded: Path::default(), empty: Path::default(), problem: Some(p) };
    if !offers(level, layout, r.from, r.cargo) {
        return fail(RouteProblem::NoOffer);
    }
    if !takes(level, layout, r.to, r.cargo) {
        return fail(RouteProblem::NoTake);
    }
    let a = world.access(level, layout, r.from);
    let b = world.access(level, layout, r.to);
    let load = Load::of(r.cargo);
    // Pick the start tile whose full loop (there loaded, back empty to the same tile) is fastest.
    let mut best: Option<RoutePlan> = None;
    let mut any_there = false;
    for &start in &a {
        let Some(there) = path::find(world, r.species, load, &[start], &b) else { continue };
        any_there = true;
        let Some(back) = path::find(world, r.species, Load::EMPTY, &[there.end()], &[start]) else { continue };
        let plan = RoutePlan { loaded: there, empty: back, problem: None };
        if best.as_ref().map_or(true, |bp| plan.cycle() < bp.cycle()) {
            best = Some(plan);
        }
    }
    best.unwrap_or_else(|| fail(if any_there { RouteProblem::NoWayBack } else { RouteProblem::NoWayThere }))
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum CState {
    WaitPickup,
    Picking(i32),
    Carry,
    WaitDrop,
    Dropping(i32),
    Return,
}

#[derive(Clone, Debug)]
pub struct Creature {
    pub route: usize,
    pub species: Species,
    pub cargo: Option<Good>,
    pub state: CState,
    /// Current tile and the step being walked toward (index into the active path).
    pub x: i32,
    pub y: i32,
    pub step: usize,
    pub prog: i32,
    pub facing: i32,
    /// Ticks spent waiting since the last thing it did.
    pub idle: i32,
}

#[derive(Clone, Debug)]
pub struct NodeState {
    pub node: NodeRef,
    pub stock: [i32; 8],
    pub incoming: [i32; 8],
    /// Sources: goods on hand cap (<= 0: endless) and regrowth.
    pub max: i32,
    pub regrow: i32,
    pub regrow_t: i32,
    pub kind: Option<Kind>,
    /// What a source grows.
    pub good: Option<Good>,
    pub craft_t: i32,
    pub docks: i32,
    pub busy: i32,
    pub delivered: i32,
}

impl NodeState {
    pub fn total(&self) -> i32 {
        self.stock.iter().sum()
    }
    fn available(&self, g: Good) -> bool {
        if self.max <= 0 && self.kind.is_none() && matches!(self.node, NodeRef::Source(_)) {
            return true;
        }
        match self.kind {
            Some(k) if k != Kind::Depot => k.recipe().is_some_and(|r| r.output == g) && self.stock[g.index()] > 0,
            _ => self.stock[g.index()] > 0,
        }
    }
    fn space(&self, need: &[(Good, i32)], g: Good) -> bool {
        let gi = g.index();
        let need_of = need.iter().find(|(n, _)| *n == g).map(|(_, c)| *c).unwrap_or(0);
        match (self.node, self.kind) {
            (NodeRef::Fortress, _) => self.stock[gi] + self.incoming[gi] < need_of,
            (_, Some(Kind::Depot)) => self.stock.iter().sum::<i32>() + self.incoming.iter().sum::<i32>() < DEPOT_CAP,
            (_, Some(k)) => k.recipe().is_some_and(|r| r.inputs.iter().any(|(i, _)| *i == g)) && self.stock[gi] + self.incoming[gi] < SHOP_IN_CAP,
            _ => false,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Outcome {
    Running,
    Done(i32),
    Stalled(i32),
    TooSlow,
}

pub struct Sim {
    pub tick: i32,
    pub plans: Vec<RoutePlan>,
    pub routes: Vec<crate::layout::Route>,
    pub nodes: Vec<NodeState>,
    pub creatures: Vec<Creature>,
    pub outcome: Outcome,
    pub cost: i32,
    pub crew: i32,
    need: Vec<(Good, i32)>,
    /// Events this tick for sound and sparkle: (x, y, kind) with kind 0 pick, 1 drop, 2 craft, 3 fortress.
    pub events: Vec<(i32, i32, u8)>,
}

pub struct Score {
    pub ticks: i32,
    pub cost: i32,
    pub crew: i32,
}

impl Sim {
    /// Builds the colony at tick 0. Fails if any route can't run.
    pub fn new(level: &Level, layout: &Layout) -> Result<Sim, (usize, RouteProblem)> {
        let world = World::new(level, layout);
        let mut plans = Vec::new();
        for (i, r) in layout.routes.iter().enumerate() {
            let p = plan_route(level, layout, &world, r);
            if let Some(pr) = p.problem.clone() {
                return Err((i, pr));
            }
            plans.push(p);
        }
        let mut nodes = Vec::new();
        for (i, s) in level.sources.iter().enumerate() {
            let mut st = [0; 8];
            st[s.good.index()] = s.max.max(0);
            nodes.push(NodeState { node: NodeRef::Source(i), stock: st, incoming: [0; 8], max: s.max, regrow: s.regrow, regrow_t: 0, kind: None, good: Some(s.good), craft_t: 0, docks: 1, busy: 0, delivered: 0 });
        }
        nodes.push(NodeState { node: NodeRef::Fortress, stock: [0; 8], incoming: [0; 8], max: 0, regrow: 0, regrow_t: 0, kind: None, good: None, craft_t: 0, docks: 2, busy: 0, delivered: 0 });
        for b in &layout.buildings {
            nodes.push(NodeState { node: NodeRef::Building(b.id), stock: [0; 8], incoming: [0; 8], max: 0, regrow: 0, regrow_t: 0, kind: Some(b.kind), good: None, craft_t: 0, docks: 1, busy: 0, delivered: 0 });
        }
        let mut creatures = Vec::new();
        for (ri, r) in layout.routes.iter().enumerate() {
            let (x, y) = plans[ri].loaded.start;
            for _ in 0..r.count {
                creatures.push(Creature { route: ri, species: r.species, cargo: None, state: CState::WaitPickup, x, y, step: 0, prog: 0, facing: 1, idle: 0 });
            }
        }
        Ok(Sim {
            tick: 0,
            plans,
            routes: layout.routes.clone(),
            nodes,
            creatures,
            outcome: Outcome::Running,
            cost: layout_cost(level, layout),
            crew: layout.crew(),
            need: level.need.clone(),
            events: Vec::new(),
        })
    }

    pub fn node_index(&self, n: NodeRef) -> Option<usize> {
        self.nodes.iter().position(|s| s.node == n)
    }

    pub fn node(&self, n: NodeRef) -> Option<&NodeState> {
        self.nodes.iter().find(|s| s.node == n)
    }

    pub fn delivered(&self, g: Good) -> i32 {
        self.node(NodeRef::Fortress).map(|f| f.stock[g.index()]).unwrap_or(0)
    }

    pub fn delivered_total(&self) -> i32 {
        self.node(NodeRef::Fortress).map(|f| f.total()).unwrap_or(0)
    }

    fn complete(&self) -> bool {
        self.need.iter().all(|&(g, n)| self.delivered(g) >= n)
    }

    pub fn step(&mut self) {
        if self.outcome != Outcome::Running {
            return;
        }
        self.events.clear();
        self.tick += 1;
        let mut active = false;
        // Sources regrow, workshops craft.
        for n in self.nodes.iter_mut() {
            if let Some(g) = n.good {
                if n.max > 0 && n.regrow > 0 && n.stock[g.index()] < n.max {
                    active = true;
                    n.regrow_t += 1;
                    if n.regrow_t >= n.regrow {
                        n.regrow_t = 0;
                        n.stock[g.index()] += 1;
                    }
                }
            }
        }
        for idx in 0..self.nodes.len() {
            let n = &mut self.nodes[idx];
            if let Some(k) = n.kind {
                if let Some(r) = k.recipe() {
                    if n.craft_t > 0 {
                        active = true;
                        n.craft_t -= 1;
                        if n.craft_t == 0 {
                            n.stock[r.output.index()] += 1;
                            self.events.push((idx as i32, 0, 2));
                        }
                    }
                    if n.craft_t == 0 && n.stock[r.output.index()] < SHOP_OUT_CAP && r.inputs.iter().all(|(g, c)| n.stock[g.index()] >= *c) {
                        for (g, c) in r.inputs {
                            n.stock[g.index()] -= c;
                        }
                        n.craft_t = r.ticks;
                        active = true;
                    }
                }
            }
        }
        // Creatures, in a fixed order.
        for ci in 0..self.creatures.len() {
            if self.creature_step(ci) {
                active = true;
            }
        }
        if self.complete() {
            self.outcome = Outcome::Done(self.tick);
        } else if !active {
            self.outcome = Outcome::Stalled(self.tick);
        } else if self.tick >= MAX_TICKS {
            self.outcome = Outcome::TooSlow;
        }
    }

    /// Advances one creature; returns whether anything is still in motion for it.
    fn creature_step(&mut self, ci: usize) -> bool {
        let c = self.creatures[ci].clone();
        let route = self.routes[c.route];
        let (fi, ti) = (self.node_index(route.from).unwrap(), self.node_index(route.to).unwrap());
        let g = route.cargo;
        let mut cr = c;
        let mut moving = true;
        match cr.state {
            CState::WaitPickup => {
                let ok = self.nodes[fi].busy < self.nodes[fi].docks && self.nodes[fi].available(g) && self.nodes[ti].space(&self.need, g);
                if ok {
                    let f = &mut self.nodes[fi];
                    f.busy += 1;
                    if !(f.max <= 0 && matches!(f.node, NodeRef::Source(_))) {
                        f.stock[g.index()] -= 1;
                    }
                    self.nodes[ti].incoming[g.index()] += 1;
                    cr.state = CState::Picking(PICKUP_TICKS);
                    cr.idle = 0;
                } else {
                    // Waiting can be undone by others, so it doesn't count as stalled on its own.
                    moving = false;
                    cr.idle += 1;
                }
            }
            CState::Picking(t) => {
                if t <= 1 {
                    self.nodes[fi].busy -= 1;
                    cr.cargo = Some(g);
                    cr.state = CState::Carry;
                    cr.step = 0;
                    cr.prog = 0;
                    self.events.push((cr.x, cr.y, 0));
                    if self.plans[cr.route].loaded.steps.is_empty() {
                        cr.state = CState::WaitDrop;
                    }
                } else {
                    cr.state = CState::Picking(t - 1);
                }
            }
            CState::Carry | CState::Return => {
                let carrying = cr.state == CState::Carry;
                let path = if carrying { &self.plans[cr.route].loaded } else { &self.plans[cr.route].empty };
                if let Some(st) = path.steps.get(cr.step) {
                    cr.prog += 1;
                    if st.x != cr.x {
                        cr.facing = (st.x - cr.x).signum();
                    }
                    if cr.prog >= st.t {
                        cr.x = st.x;
                        cr.y = st.y;
                        cr.step += 1;
                        cr.prog = 0;
                    }
                }
                if cr.step >= path.steps.len() {
                    cr.step = 0;
                    cr.prog = 0;
                    cr.state = if carrying { CState::WaitDrop } else { CState::WaitPickup };
                }
            }
            CState::WaitDrop => {
                if self.nodes[ti].busy < self.nodes[ti].docks {
                    self.nodes[ti].busy += 1;
                    cr.state = CState::Dropping(DROP_TICKS);
                    cr.idle = 0;
                } else {
                    moving = false;
                    cr.idle += 1;
                }
            }
            CState::Dropping(t) => {
                if t <= 1 {
                    let to = &mut self.nodes[ti];
                    to.busy -= 1;
                    to.incoming[g.index()] -= 1;
                    to.stock[g.index()] += 1;
                    to.delivered += 1;
                    cr.cargo = None;
                    cr.state = CState::Return;
                    cr.step = 0;
                    cr.prog = 0;
                    self.events.push((cr.x, cr.y, if route.to == NodeRef::Fortress { 3 } else { 1 }));
                    if self.plans[cr.route].empty.steps.is_empty() {
                        cr.state = CState::WaitPickup;
                    }
                } else {
                    cr.state = CState::Dropping(t - 1);
                }
            }
        }
        self.creatures[ci] = cr;
        moving
    }

    /// Runs to the end (or the tick cap) and scores it.
    pub fn run(&mut self) -> Outcome {
        while self.outcome == Outcome::Running {
            self.step();
        }
        self.outcome.clone()
    }

    pub fn score(&self) -> Option<Score> {
        match self.outcome {
            Outcome::Done(t) => Some(Score { ticks: t, cost: self.cost, crew: self.crew }),
            _ => None,
        }
    }
}

/// Simulates a layout to the end: `Ok(score)` or why it failed.
pub fn evaluate(level: &Level, layout: &Layout) -> Result<Score, String> {
    let mut sim = Sim::new(level, layout).map_err(|(i, p)| format!("ROUTE {} {}", i + 1, p.text()))?;
    if layout.routes.is_empty() {
        return Err("NO ROUTES".into());
    }
    match sim.run() {
        Outcome::Done(_) => Ok(sim.score().unwrap()),
        Outcome::Stalled(t) => Err(format!("STALLED AT TICK {t}")),
        Outcome::TooSlow => Err("TOO SLOW".into()),
        Outcome::Running => unreachable!(),
    }
}
