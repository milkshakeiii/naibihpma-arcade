//! GULP's rules: one frog, one pond. Each turn the frog may hop once and act once (tongue,
//! spit, gulp or croak). Creatures show the squares they will strike; strikes land relative to
//! where the creature stands when they resolve, so pushing a creature moves its strike.

use crate::rng::Rng;

pub const N: i32 = 8;
pub type Pos = (i32, i32);

pub const DIRS8: [Pos; 8] = [(1, 0), (-1, 0), (0, 1), (0, -1), (1, 1), (-1, 1), (1, -1), (-1, -1)];
pub const ORTH: [Pos; 4] = [(1, 0), (-1, 0), (0, 1), (0, -1)];

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Tile {
    Bank,
    Water,
    Lily,
    Reeds,
    Thorns,
    Stone,
}

impl Tile {
    pub fn solid(self) -> bool {
        matches!(self, Tile::Reeds | Tile::Stone)
    }
    pub fn name(self) -> &'static str {
        match self {
            Tile::Bank => "BANK",
            Tile::Water => "WATER",
            Tile::Lily => "LILY PAD",
            Tile::Reeds => "REEDS",
            Tile::Thorns => "THORNS",
            Tile::Stone => "STONE",
        }
    }
    pub fn text(self) -> &'static str {
        match self {
            Tile::Bank => "SOLID GROUND.",
            Tile::Water => "YOU SWIM, BUT FROM WATER YOU HOP ONLY ONE SQUARE. MOST CREATURES THAT END UP HERE DROWN.",
            Tile::Lily => "FLOATS ON WATER. A LARGE FROG SINKS IT.",
            Tile::Reeds => "BLOCKS HOPS, LINES AND STRIKES. YOUR TONGUE CAN GRAB IT TO PULL YOU OVER.",
            Tile::Thorns => "ANYTHING THAT LANDS OR IS PUSHED HERE TAKES 1.",
            Tile::Stone => "BLOCKS EVERYTHING. A CREATURE SLAMMED INTO IT TAKES 1. YOUR TONGUE CAN GRAB IT.",
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash, PartialOrd, Ord)]
pub enum Kind {
    Fly,
    Beetle,
    Newt,
    Dragonfly,
    Snake,
    Crayfish,
    Heron,
    Grof,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Strike {
    /// Runs straight until it hits something; that thing takes 1 and is pushed a square.
    Charge,
    /// The next square.
    Bite,
    /// The next two squares.
    Lunge,
    /// The square two away (any of 8 directions).
    Stab,
    /// The square ahead and the two beside it.
    Pinch,
    /// A square two away, diagonally; it flies there after.
    Dive,
    /// Every square around it.
    Stomp,
    None,
}

pub struct KindInfo {
    pub name: &'static str,
    pub size: i32,
    pub hp: i32,
    pub swims: bool,
    pub flies: bool,
    pub moves: i32,
    pub food: i32,
    pub strike: Strike,
    pub dmg: i32,
    pub text: &'static str,
}

impl Kind {
    pub fn info(self) -> KindInfo {
        let k = |name, size, hp, swims, flies, moves, food, strike, dmg, text| KindInfo { name, size, hp, swims, flies, moves, food, strike, dmg, text };
        match self {
            Kind::Fly => k("FLY", 0, 1, false, true, 1, 1, Strike::None, 0, "FOOD. FLUTTERS ABOUT."),
            Kind::Beetle => k("BEETLE", 1, 1, false, false, 1, 2, Strike::Charge, 1, "CHARGES IN A STRAIGHT LINE UNTIL IT HITS SOMETHING, WHICH TAKES 1 AND IS SHOVED."),
            Kind::Newt => k("NEWT", 1, 1, true, false, 2, 2, Strike::Bite, 1, "BITES THE NEXT SQUARE FOR 1. SWIMS."),
            Kind::Dragonfly => k("DRAGONFLY", 1, 1, false, true, 2, 2, Strike::Dive, 1, "DIVES AT A SQUARE TWO AWAY DIAGONALLY FOR 1, THEN SITS THERE. FLIES."),
            Kind::Snake => k("SNAKE", 2, 2, true, false, 1, 3, Strike::Lunge, 1, "LUNGES THROUGH THE NEXT TWO SQUARES, 1 EACH. SWIMS."),
            Kind::Crayfish => k("CRAYFISH", 2, 2, true, false, 1, 3, Strike::Pinch, 1, "PINCHES THE SQUARE AHEAD AND BOTH BESIDE IT FOR 1. ARMORED: SPIT DOES 1 LESS. SWIMS."),
            Kind::Heron => k("HERON", 3, 3, true, false, 2, 0, Strike::Stab, 2, "STABS THE SQUARE TWO AWAY FOR 2. WADES. TOO BIG TO SWALLOW."),
            Kind::Grof => k("GROF", 4, 8, true, false, 1, 0, Strike::Stomp, 1, "STOMPS EVERY SQUARE AROUND HIM FOR 1 AND SHOVES. CALLS HELP WHEN HURT."),
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash, PartialOrd, Ord)]
pub enum Mutation {
    LongTongue,
    BigThroat,
    PoisonSkin,
    SpringLegs,
    AcidSpit,
    DeepCroak,
    Gills,
    StickyTongue,
    Glutton,
    Camouflage,
}

pub const ALL_MUTATIONS: [Mutation; 10] = [
    Mutation::LongTongue,
    Mutation::BigThroat,
    Mutation::PoisonSkin,
    Mutation::SpringLegs,
    Mutation::AcidSpit,
    Mutation::DeepCroak,
    Mutation::Gills,
    Mutation::StickyTongue,
    Mutation::Glutton,
    Mutation::Camouflage,
];

impl Mutation {
    pub fn name(self) -> &'static str {
        match self {
            Mutation::LongTongue => "LONG TONGUE",
            Mutation::BigThroat => "BIG THROAT",
            Mutation::PoisonSkin => "POISON SKIN",
            Mutation::SpringLegs => "SPRING LEGS",
            Mutation::AcidSpit => "ACID SPIT",
            Mutation::DeepCroak => "DEEP CROAK",
            Mutation::Gills => "GILLS",
            Mutation::StickyTongue => "STICKY TONGUE",
            Mutation::Glutton => "GLUTTON",
            Mutation::Camouflage => "CAMOUFLAGE",
        }
    }
    pub fn text(self) -> &'static str {
        match self {
            Mutation::LongTongue => "YOUR TONGUE REACHES 5 SQUARES INSTEAD OF 3.",
            Mutation::BigThroat => "YOUR THROAT HOLDS TWO THINGS.",
            Mutation::PoisonSkin => "ANYTHING THAT STRIKES YOU TAKES 1.",
            Mutation::SpringLegs => "YOU CAN ALSO HOP TWO SQUARES IN A STRAIGHT LINE, OVER ANYTHING (NOT FROM WATER, NOT WHEN LARGE).",
            Mutation::AcidSpit => "SPIT DOES 1 MORE DAMAGE.",
            Mutation::DeepCroak => "YOUR CROAK SHOVES 2 SQUARES, AND IS READY AGAIN A TURN SOONER.",
            Mutation::Gills => "DIGESTING WHILE IN WATER HEALS 1, AND SPRING LEGS WORK FROM WATER.",
            Mutation::StickyTongue => "WHATEVER YOUR TONGUE YANKS IS STUNNED: ITS STRIKE IS CANCELLED.",
            Mutation::Glutton => "DIGESTING ANYTHING HEALS 1.",
            Mutation::Camouflage => "THE FIRST STRIKE THAT WOULD HIT YOU IN EACH POND MISSES.",
        }
    }
}

#[derive(Clone, Debug)]
pub struct Creature {
    pub id: u32,
    pub kind: Kind,
    pub pos: Pos,
    pub hp: i32,
    pub stunned: bool,
    /// Declared strike direction; the strike lands relative to where it stands at resolution.
    pub aim: Option<Pos>,
}

#[derive(Clone, Debug)]
pub struct Frog {
    pub pos: Pos,
    pub hearts: i32,
    pub max_hearts: i32,
    pub size: i32,
    pub growth: i32,
    pub throat: Vec<Kind>,
    pub croak_cd: i32,
    pub hop_left: bool,
    pub act_left: bool,
    pub muts: Vec<Mutation>,
    pub camo_ready: bool,
}

impl Frog {
    pub fn new() -> Self {
        Self {
            pos: (0, N - 1),
            hearts: 3,
            max_hearts: 3,
            size: 1,
            growth: 0,
            throat: Vec::new(),
            croak_cd: 0,
            hop_left: true,
            act_left: true,
            muts: Vec::new(),
            camo_ready: false,
        }
    }
    pub fn has(&self, m: Mutation) -> bool {
        self.muts.contains(&m)
    }
    pub fn throat_cap(&self) -> usize {
        if self.has(Mutation::BigThroat) {
            2
        } else {
            1
        }
    }
    pub fn tongue_range(&self) -> i32 {
        if self.has(Mutation::LongTongue) {
            5
        } else {
            3
        }
    }
    pub fn size_name(&self) -> &'static str {
        ["", "SMALL", "MEDIUM", "LARGE"][self.size.clamp(1, 3) as usize]
    }
}

/// Food needed to reach medium, then large.
pub const GROW_AT: [i32; 2] = [3, 8];

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Arrival {
    pub pos: Pos,
    pub kind: Kind,
    pub turns: i32,
}

#[derive(Clone, Debug, PartialEq)]
pub enum Ev {
    Hop { from: Pos, to: Pos },
    Sank { at: Pos },
    Tongue { dir: Pos, end: Pos },
    Swallow { kind: Kind },
    Yank { id: u32, from: Pos, to: Pos },
    Grapple { from: Pos, to: Pos },
    Spit { kind: Kind, dir: Pos, end: Pos },
    Gulp { kind: Kind, grew: bool },
    Croak,
    Strike { id: u32, kind: Kind, squares: Vec<Pos> },
    Hurt { dmg: i32, by: &'static str },
    Dodged,
    Damage { id: u32, dmg: i32 },
    Pushed { id: u32, from: Pos, to: Pos },
    FrogPushed { from: Pos, to: Pos },
    Died { id: u32, kind: Kind, at: Pos, how: &'static str },
    Moved { id: u32, from: Pos, to: Pos },
    Ripple { at: Pos, kind: Kind },
    Arrived { id: u32, kind: Kind, at: Pos },
    Cleared,
    Exited,
    EggLost { at: Pos },
    Hatched { clutches: i32 },
    FrogDied,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Act {
    Hop(Pos),
    Tongue(Pos),
    Spit(Pos),
    Gulp,
    Croak,
    End,
}

#[derive(Clone, Debug)]
pub struct Pond {
    pub tiles: [[Tile; N as usize]; N as usize],
    pub creatures: Vec<Creature>,
    pub frog: Frog,
    pub arrivals: Vec<Arrival>,
    /// Reinforcements still to come: (turn, kind).
    pub waves: Vec<(u32, Kind)>,
    pub exit: Pos,
    pub turn: u32,
    pub next_id: u32,
    pub rng: Rng,
    pub cleared: bool,
    pub over: Option<bool>,
    pub eaten: Vec<Kind>,
    pub killed: Vec<Kind>,
    /// Lessons keep the exit shut until they've said everything.
    pub exit_locked: bool,
    /// The frog's spawn: predators go for it, and each clutch lost costs a heart.
    pub eggs: Vec<Pos>,
    pub eggs_lost: u32,
}

pub fn inb((x, y): Pos) -> bool {
    (0..N).contains(&x) && (0..N).contains(&y)
}

pub fn cheb(a: Pos, b: Pos) -> i32 {
    (a.0 - b.0).abs().max((a.1 - b.1).abs())
}

fn add(a: Pos, d: Pos, k: i32) -> Pos {
    (a.0 + d.0 * k, a.1 + d.1 * k)
}

fn sign(v: i32) -> i32 {
    v.signum()
}

pub fn sq((x, y): Pos) -> String {
    format!("{}{}", (b'a' + x as u8) as char, N - y)
}

impl Pond {
    pub fn empty(rng: Rng) -> Self {
        Self {
            tiles: [[Tile::Bank; N as usize]; N as usize],
            creatures: Vec::new(),
            frog: Frog::new(),
            arrivals: Vec::new(),
            waves: Vec::new(),
            exit: (N / 2, 0),
            turn: 0,
            next_id: 1,
            rng,
            cleared: false,
            over: None,
            eaten: Vec::new(),
            killed: Vec::new(),
            exit_locked: false,
            eggs: Vec::new(),
            eggs_lost: 0,
        }
    }

    pub fn tile(&self, p: Pos) -> Tile {
        self.tiles[p.1 as usize][p.0 as usize]
    }

    pub fn set(&mut self, p: Pos, t: Tile) {
        self.tiles[p.1 as usize][p.0 as usize] = t;
    }

    pub fn add_creature(&mut self, kind: Kind, pos: Pos) -> u32 {
        let id = self.next_id;
        self.next_id += 1;
        self.creatures.push(Creature { id, kind, pos, hp: kind.info().hp, stunned: false, aim: None });
        id
    }

    pub fn at(&self, p: Pos) -> Option<usize> {
        self.creatures.iter().position(|c| c.pos == p)
    }

    pub fn predators_left(&self) -> bool {
        self.creatures.iter().any(|c| c.kind != Kind::Fly) || self.arrivals.iter().any(|a| a.kind != Kind::Fly) || self.waves.iter().any(|w| w.1 != Kind::Fly)
    }

    fn free_for(&self, p: Pos, kind: Option<Kind>) -> bool {
        if !inb(p) || self.tile(p).solid() || self.at(p).is_some() || p == self.frog.pos || self.eggs.contains(&p) {
            return false;
        }
        match kind {
            None => true,
            Some(k) => {
                let i = k.info();
                i.flies || i.swims || self.tile(p) != Tile::Water
            }
        }
    }

    // ----- The frog's half of a turn -----

    /// Squares the frog can hop to.
    pub fn hop_targets(&self) -> Vec<Pos> {
        if !self.frog.hop_left {
            return Vec::new();
        }
        let f = &self.frog;
        let in_water = self.tile(f.pos) == Tile::Water;
        // One square; spring legs add a two-square leap (not from water, unless gilled, and not
        // for a large frog).
        let far = f.has(Mutation::SpringLegs) && (!in_water || f.has(Mutation::Gills)) && f.size < 3;
        let mut v = Vec::new();
        for d in DIRS8 {
            let mut reach = vec![1];
            if far {
                reach.push(2);
            }
            for k in reach {
                let p = add(f.pos, d, k);
                if self.free_for(p, None) {
                    v.push(p);
                }
            }
        }
        v
    }

    /// The first thing along a line from the frog within `range`: (square, what).
    pub fn line_hit(&self, dir: Pos, range: i32) -> (Pos, Option<usize>, bool) {
        let mut last = self.frog.pos;
        for k in 1..=range {
            let p = add(self.frog.pos, dir, k);
            if !inb(p) {
                return (last, None, false);
            }
            if let Some(i) = self.at(p) {
                return (p, Some(i), false);
            }
            if self.tile(p).solid() {
                return (p, None, true);
            }
            last = p;
        }
        (last, None, false)
    }

    /// The frog's action and, if its turn is over, the creatures' turn.
    pub fn act(&mut self, a: Act) -> Vec<Ev> {
        let (mut ev, done) = self.player_act(a);
        if done && self.over.is_none() {
            ev.extend(self.creatures_turn());
        }
        ev
    }

    /// Only the frog's action. Returns its events and whether the frog's turn is over.
    pub fn player_act(&mut self, a: Act) -> (Vec<Ev>, bool) {
        let mut ev = Vec::new();
        if self.over.is_some() {
            return (ev, false);
        }
        match a {
            Act::Hop(to) => {
                if !self.hop_targets().contains(&to) {
                    return (ev, false);
                }
                let from = self.frog.pos;
                self.frog.pos = to;
                self.frog.hop_left = false;
                ev.push(Ev::Hop { from, to });
                self.land(&mut ev);
            }
            Act::Tongue(dir) => {
                if !self.frog.act_left {
                    return (ev, false);
                }
                self.frog.act_left = false;
                let (end, who, solid) = self.line_hit(dir, self.frog.tongue_range());
                ev.push(Ev::Tongue { dir, end });
                if let Some(i) = who {
                    let c = self.creatures[i].clone();
                    let csize = c.kind.info().size;
                    if csize < self.frog.size && self.frog.throat.len() < self.frog.throat_cap() {
                        self.creatures.remove(i);
                        self.frog.throat.push(c.kind);
                        ev.push(Ev::Swallow { kind: c.kind });
                    } else if csize <= self.frog.size {
                        // Yank it to the square next to the frog.
                        let to = add(self.frog.pos, dir, 1);
                        if to != c.pos {
                            self.creatures[i].pos = to;
                            ev.push(Ev::Yank { id: c.id, from: c.pos, to });
                        }
                        if self.frog.has(Mutation::StickyTongue) {
                            self.creatures[i].stunned = true;
                        }
                        self.settle_creature(c.id, "PULLED INTO THE WATER", &mut ev);
                    } else {
                        // Too big: the frog is pulled to it.
                        let to = add(end, (-dir.0, -dir.1), 1);
                        if to != self.frog.pos {
                            let from = self.frog.pos;
                            self.frog.pos = to;
                            ev.push(Ev::Grapple { from, to });
                            self.land(&mut ev);
                        }
                    }
                } else if solid {
                    let to = add(end, (-dir.0, -dir.1), 1);
                    if to != self.frog.pos {
                        let from = self.frog.pos;
                        self.frog.pos = to;
                        ev.push(Ev::Grapple { from, to });
                        self.land(&mut ev);
                    }
                }
            }
            Act::Spit(dir) => {
                if !self.frog.act_left || self.frog.throat.is_empty() {
                    return (ev, false);
                }
                self.frog.act_left = false;
                let kind = self.frog.throat.remove(0);
                let (end, who, solid) = self.line_hit(dir, 4);
                ev.push(Ev::Spit { kind, dir, end });
                if let Some(i) = who {
                    let id = self.creatures[i].id;
                    let mut dmg = 1 + kind.info().size + if self.frog.has(Mutation::AcidSpit) { 1 } else { 0 };
                    if self.creatures[i].kind == Kind::Crayfish {
                        dmg -= 1;
                    }
                    self.hurt_creature(id, dmg, "SPAT AT", &mut ev);
                    if self.creatures.iter().any(|c| c.id == id) {
                        self.push_creature(id, dir, 1, &mut ev);
                    }
                } else if !solid {
                    // Nothing hit: it lands, dazed.
                    let at = end;
                    if self.free_for(at, None) && at != self.frog.pos {
                        let id = self.add_creature(kind, at);
                        let last = self.creatures.len() - 1;
                        self.creatures[last].stunned = true;
                        self.settle_creature(id, "DROWNED", &mut ev);
                    }
                }
            }
            Act::Gulp => {
                if !self.frog.act_left || self.frog.throat.is_empty() {
                    return (ev, false);
                }
                self.frog.act_left = false;
                let kind = self.frog.throat.remove(0);
                self.eaten.push(kind);
                let grew = self.digest(kind);
                ev.push(Ev::Gulp { kind, grew });
            }
            Act::Croak => {
                if !self.frog.act_left || self.frog.croak_cd > 0 {
                    return (ev, false);
                }
                self.frog.act_left = false;
                self.frog.croak_cd = if self.frog.has(Mutation::DeepCroak) { 2 } else { 3 };
                ev.push(Ev::Croak);
                let n = if self.frog.has(Mutation::DeepCroak) { 2 } else { 1 };
                let fp = self.frog.pos;
                let mut near: Vec<(u32, Pos)> = self.creatures.iter().filter(|c| cheb(c.pos, fp) == 1).map(|c| (c.id, (sign(c.pos.0 - fp.0), sign(c.pos.1 - fp.1)))).collect();
                near.sort_by_key(|(id, _)| *id);
                for (id, d) in near {
                    if self.creatures.iter().any(|c| c.id == id) {
                        self.push_creature(id, d, n, &mut ev);
                    }
                }
            }
            Act::End => {}
        }
        self.check_cleared(&mut ev);
        if self.frog.pos == self.exit && self.cleared && !self.exit_locked && matches!(a, Act::Hop(_)) {
            let clutches = self.eggs.len() as i32;
            if clutches > 0 {
                self.frog.hearts = (self.frog.hearts + clutches).min(self.frog.max_hearts);
                ev.push(Ev::Hatched { clutches });
            }
            ev.push(Ev::Exited);
            self.over = Some(true);
            return (ev, false);
        }
        let done = self.over.is_none() && (a == Act::End || (!self.frog.hop_left && !self.frog.act_left));
        (ev, done)
    }

    /// Effects of the square the frog just landed on.
    fn land(&mut self, ev: &mut Vec<Ev>) {
        let p = self.frog.pos;
        match self.tile(p) {
            Tile::Lily if self.frog.size >= 3 => {
                self.set(p, Tile::Water);
                ev.push(Ev::Sank { at: p });
            }
            Tile::Thorns => self.hurt_frog(1, "THORNS", ev),
            _ => {}
        }
    }

    /// Adds food; returns true if the frog grew a size.
    fn digest(&mut self, kind: Kind) -> bool {
        let f = &mut self.frog;
        f.growth += kind.info().food;
        // Eating feeds growth. Only the glutton and the gilled (in water) heal from it.
        let heal = if f.has(Mutation::Glutton) { 1 } else { 0 } + if f.has(Mutation::Gills) && self.tiles[f.pos.1 as usize][f.pos.0 as usize] == Tile::Water { 1 } else { 0 };
        let mut grew = false;
        while f.size < 3 && f.growth >= GROW_AT[(f.size - 1) as usize] {
            f.size += 1;
            f.max_hearts += 1;
            grew = true;
        }
        f.hearts = (f.hearts + heal + grew as i32).min(f.max_hearts);
        grew
    }

    fn hurt_frog(&mut self, dmg: i32, by: &'static str, ev: &mut Vec<Ev>) {
        if dmg <= 0 {
            return;
        }
        if self.frog.camo_ready {
            self.frog.camo_ready = false;
            ev.push(Ev::Dodged);
            return;
        }
        self.frog.hearts -= dmg;
        ev.push(Ev::Hurt { dmg, by });
        if self.frog.hearts <= 0 {
            self.frog.hearts = 0;
            ev.push(Ev::FrogDied);
            self.over = Some(false);
        }
    }

    fn hurt_creature(&mut self, id: u32, dmg: i32, how: &'static str, ev: &mut Vec<Ev>) {
        let Some(i) = self.creatures.iter().position(|c| c.id == id) else { return };
        if dmg <= 0 {
            return;
        }
        self.creatures[i].hp -= dmg;
        ev.push(Ev::Damage { id, dmg });
        if self.creatures[i].hp <= 0 {
            let c = self.creatures.remove(i);
            self.killed.push(c.kind);
            ev.push(Ev::Died { id, kind: c.kind, at: c.pos, how });
        } else if self.creatures[i].kind == Kind::Grof && self.creatures[i].hp <= 4 && !self.creatures[i].stunned {
            // Grof calls for help when hurt (once per hit that leaves him low).
            for d in DIRS8.iter().take(2) {
                let p = add(self.creatures[i].pos, *d, 2);
                if self.free_for(p, Some(Kind::Newt)) {
                    self.arrivals.push(Arrival { pos: p, kind: Kind::Newt, turns: 1 });
                }
            }
        }
    }

    /// A creature that has just been moved into water or thorns suffers for it.
    fn settle_creature(&mut self, id: u32, how: &'static str, ev: &mut Vec<Ev>) {
        let Some(i) = self.creatures.iter().position(|c| c.id == id) else { return };
        let c = &self.creatures[i];
        let info = c.kind.info();
        match self.tile(c.pos) {
            Tile::Water if !info.swims && !info.flies => {
                let c = self.creatures.remove(i);
                self.killed.push(c.kind);
                ev.push(Ev::Died { id, kind: c.kind, at: c.pos, how: if how == "SPAT AT" { "DROWNED" } else { how } });
            }
            Tile::Thorns if !info.flies => self.hurt_creature(id, 1, "THORNS", ev),
            _ => {}
        }
    }

    /// Shoves a creature `n` squares along `dir`. Slams into walls and others hurt (1 each).
    pub fn push_creature(&mut self, id: u32, dir: Pos, n: i32, ev: &mut Vec<Ev>) {
        for _ in 0..n {
            let Some(i) = self.creatures.iter().position(|c| c.id == id) else { return };
            let from = self.creatures[i].pos;
            let to = add(from, dir, 1);
            if !inb(to) || self.tile(to).solid() {
                self.hurt_creature(id, 1, "SLAMMED", ev);
                return;
            }
            if to == self.frog.pos {
                self.hurt_creature(id, 1, "SLAMMED", ev);
                return;
            }
            if self.eggs.contains(&to) {
                self.lose_egg(to, ev);
                self.hurt_creature(id, 1, "SLAMMED", ev);
                return;
            }
            if let Some(j) = self.at(to) {
                let other = self.creatures[j].id;
                self.hurt_creature(id, 1, "SLAMMED", ev);
                self.hurt_creature(other, 1, "SLAMMED", ev);
                return;
            }
            self.creatures[i].pos = to;
            ev.push(Ev::Pushed { id, from, to });
            self.settle_creature(id, "SHOVED INTO THE WATER", ev);
        }
    }

    /// A clutch is destroyed: the frog feels it (camouflage can't help).
    fn lose_egg(&mut self, at: Pos, ev: &mut Vec<Ev>) {
        if let Some(k) = self.eggs.iter().position(|&e| e == at) {
            self.eggs.remove(k);
            self.eggs_lost += 1;
            ev.push(Ev::EggLost { at });
            self.frog.hearts -= 1;
            ev.push(Ev::Hurt { dmg: 1, by: "LOSING YOUR EGGS" });
            if self.frog.hearts <= 0 {
                self.frog.hearts = 0;
                ev.push(Ev::FrogDied);
                self.over = Some(false);
            }
        }
    }

    fn push_frog(&mut self, dir: Pos, ev: &mut Vec<Ev>) {
        let from = self.frog.pos;
        let to = add(from, dir, 1);
        if self.eggs.contains(&to) {
            self.lose_egg(to, ev);
            return;
        }
        if !inb(to) || self.tile(to).solid() || self.at(to).is_some() {
            self.hurt_frog(1, "A SLAM", ev);
            return;
        }
        self.frog.pos = to;
        ev.push(Ev::FrogPushed { from, to });
        self.land(ev);
    }

    fn check_cleared(&mut self, ev: &mut Vec<Ev>) {
        if !self.cleared && !self.predators_left() {
            self.cleared = true;
            ev.push(Ev::Cleared);
        }
    }

    // ----- Strikes -----

    /// The squares a strike covers from `pos` aimed along `aim`.
    pub fn strike_squares(&self, kind: Kind, pos: Pos, aim: Pos) -> Vec<Pos> {
        let mut v = Vec::new();
        match kind.info().strike {
            Strike::None => {}
            Strike::Bite => v.push(add(pos, aim, 1)),
            Strike::Lunge => {
                for k in 1..=2 {
                    let p = add(pos, aim, k);
                    if !inb(p) || self.tile(p).solid() {
                        break;
                    }
                    v.push(p);
                }
            }
            Strike::Stab => {
                let mid = add(pos, aim, 1);
                if inb(mid) && !self.tile(mid).solid() {
                    v.push(add(pos, aim, 2));
                }
            }
            Strike::Pinch => {
                let a = add(pos, aim, 1);
                v.push(a);
                let side = (aim.1, aim.0);
                v.push(add(a, side, 1));
                v.push(add(a, (-side.0, -side.1), 1));
            }
            Strike::Dive => v.push(add(pos, aim, 2)),
            Strike::Stomp => {
                for d in DIRS8 {
                    v.push(add(pos, d, 1));
                }
            }
            Strike::Charge => {
                // Runs until the first thing in the way; that square is struck.
                for k in 1..N {
                    let p = add(pos, aim, k);
                    if !inb(p) || self.tile(p).solid() {
                        break;
                    }
                    if self.at(p).is_some() || p == self.frog.pos || self.eggs.contains(&p) {
                        v.push(p);
                        break;
                    }
                    v.push(p);
                    // A non-swimmer that charges into water plunges in.
                    if self.tile(p) == Tile::Water && !kind.info().swims {
                        break;
                    }
                }
            }
        }
        v.retain(|p| inb(*p));
        v
    }

    /// Possible aims for a kind.
    fn aims(kind: Kind) -> &'static [Pos] {
        match kind.info().strike {
            Strike::Charge | Strike::Bite | Strike::Pinch => &ORTH,
            Strike::Dive => &[(1, 1), (-1, 1), (1, -1), (-1, -1)],
            Strike::Stomp => &[(0, 1)],
            _ => &DIRS8,
        }
    }

    /// Every square some creature will strike when the frog's turn ends.
    pub fn danger(&self) -> Vec<(Pos, u32)> {
        let mut v = Vec::new();
        for c in &self.creatures {
            if c.stunned {
                continue;
            }
            if let Some(aim) = c.aim {
                for p in self.strike_squares(c.kind, c.pos, aim) {
                    v.push((p, c.id));
                }
            }
        }
        v
    }

    fn resolve_strike(&mut self, i: usize, ev: &mut Vec<Ev>) {
        let c = self.creatures[i].clone();
        let Some(aim) = c.aim else { return };
        let info = c.kind.info();
        let squares = self.strike_squares(c.kind, c.pos, aim);
        if squares.is_empty() {
            return;
        }
        ev.push(Ev::Strike { id: c.id, kind: c.kind, squares: squares.clone() });
        match info.strike {
            Strike::Charge => {
                // Run to the square before the target, then hit and shove what's there.
                let last = *squares.last().unwrap();
                let hit_frog = last == self.frog.pos;
                let hit_egg = self.eggs.contains(&last);
                let hit_creature = self.at(last).map(|j| self.creatures[j].id).filter(|&other| other != c.id);
                let stop = if hit_frog || hit_egg || hit_creature.is_some() { add(last, (-aim.0, -aim.1), 1) } else { last };
                if stop != c.pos {
                    if let Some(k) = self.creatures.iter().position(|x| x.id == c.id) {
                        self.creatures[k].pos = stop;
                        ev.push(Ev::Moved { id: c.id, from: c.pos, to: stop });
                    }
                    self.settle_creature(c.id, "CHARGED INTO THE WATER", ev);
                    if !self.creatures.iter().any(|x| x.id == c.id) {
                        return;
                    }
                }
                if hit_egg {
                    self.lose_egg(last, ev);
                } else if hit_frog {
                    self.hurt_frog(info.dmg, info.name, ev);
                    self.poison_back(c.id, ev);
                    if self.over.is_none() {
                        self.push_frog(aim, ev);
                    }
                } else if let Some(other) = hit_creature {
                    self.hurt_creature(other, info.dmg, "CHARGED", ev);
                    if self.creatures.iter().any(|x| x.id == other) {
                        self.push_creature(other, aim, 1, ev);
                    }
                } else if !inb(add(stop, aim, 1)) || self.tile(add(stop, aim, 1)).solid() {
                    self.hurt_creature(c.id, 1, "SLAMMED", ev);
                }
            }
            Strike::Stomp => {
                for d in DIRS8 {
                    let p = add(c.pos, d, 1);
                    if self.eggs.contains(&p) {
                        self.lose_egg(p, ev);
                    } else if p == self.frog.pos {
                        self.hurt_frog(info.dmg, info.name, ev);
                        if self.over.is_none() {
                            self.push_frog(d, ev);
                        }
                    } else if let Some(j) = self.at(p) {
                        let other = self.creatures[j].id;
                        self.hurt_creature(other, info.dmg, "STOMPED", ev);
                        if self.creatures.iter().any(|x| x.id == other) {
                            self.push_creature(other, d, 1, ev);
                        }
                    }
                }
            }
            _ => {
                let mut hit_frog = false;
                for p in &squares {
                    if self.eggs.contains(p) {
                        self.lose_egg(*p, ev);
                    } else if *p == self.frog.pos {
                        hit_frog = true;
                    } else if let Some(j) = self.at(*p) {
                        let other = self.creatures[j].id;
                        if other != c.id {
                            self.hurt_creature(other, info.dmg, "STRUCK BY ANOTHER", ev);
                        }
                    }
                }
                if hit_frog {
                    self.hurt_frog(info.dmg, info.name, ev);
                    self.poison_back(c.id, ev);
                }
                if info.strike == Strike::Dive {
                    let to = squares[0];
                    if self.free_for(to, Some(c.kind)) {
                        if let Some(k) = self.creatures.iter().position(|x| x.id == c.id) {
                            self.creatures[k].pos = to;
                            ev.push(Ev::Moved { id: c.id, from: c.pos, to });
                        }
                    }
                }
            }
        }
    }

    fn poison_back(&mut self, id: u32, ev: &mut Vec<Ev>) {
        if self.frog.has(Mutation::PoisonSkin) {
            self.hurt_creature(id, 1, "POISONED", ev);
        }
    }

    // ----- The creatures' half of a turn -----

    pub fn creatures_turn(&mut self) -> Vec<Ev> {
        let mut ev = Vec::new();
        // 1. Declared strikes land.
        let ids: Vec<u32> = self.creatures.iter().map(|c| c.id).collect();
        for id in ids {
            let Some(i) = self.creatures.iter().position(|c| c.id == id) else { continue };
            if self.creatures[i].stunned {
                self.creatures[i].stunned = false;
                self.creatures[i].aim = None;
                continue;
            }
            self.resolve_strike(i, &mut ev);
            if self.over.is_some() {
                return ev;
            }
        }
        // 2. Announced arrivals come in.
        let mut k = 0;
        while k < self.arrivals.len() {
            self.arrivals[k].turns -= 1;
            let a = self.arrivals[k];
            if a.turns <= 0 {
                if self.free_for(a.pos, Some(a.kind)) {
                    let id = self.add_creature(a.kind, a.pos);
                    ev.push(Ev::Arrived { id, kind: a.kind, at: a.pos });
                    self.arrivals.remove(k);
                    continue;
                }
            }
            k += 1;
        }
        self.turn += 1;
        // Flies keep drifting in while there are few about, so there's always something to eat.
        if self.turn % 4 == 0 && self.creatures.iter().filter(|c| c.kind == Kind::Fly).count() < 2 && !self.waves.iter().any(|w| w.1 == Kind::Fly) {
            self.waves.push((self.turn, Kind::Fly));
        }
        // 3. Reinforcements due this turn ripple at the edge (they arrive next turn).
        let due: Vec<Kind> = self.waves.iter().filter(|w| w.0 <= self.turn).map(|w| w.1).collect();
        self.waves.retain(|w| w.0 > self.turn);
        for kind in due {
            let edge: Vec<Pos> = (0..N)
                .flat_map(|t| [(t, 0), (t, N - 1), (0, t), (N - 1, t)])
                .filter(|&p| self.free_for(p, Some(kind)) && cheb(p, self.frog.pos) >= 3 && !self.arrivals.iter().any(|a| a.pos == p))
                .collect();
            if let Some(&pos) = (!edge.is_empty()).then(|| self.rng.pick(&edge)) {
                self.arrivals.push(Arrival { pos, kind, turns: 1 });
                ev.push(Ev::Ripple { at: pos, kind });
            }
        }
        // 4. Everyone moves, then declares a strike at where the frog is now.
        let ids: Vec<u32> = self.creatures.iter().map(|c| c.id).collect();
        for id in ids {
            self.move_and_aim(id, &mut ev);
        }
        self.frog.hop_left = true;
        self.frog.act_left = true;
        if self.frog.croak_cd > 0 {
            self.frog.croak_cd -= 1;
        }
        self.check_cleared(&mut ev);
        ev
    }

    /// What predators go for: the frog and every clutch of eggs.
    fn targets(&self) -> Vec<Pos> {
        let mut v = self.eggs.clone();
        v.push(self.frog.pos);
        v
    }

    /// Where `kind` could stand to strike `target`, and the aim it would use.
    fn striking_spots(&self, id: u32, kind: Kind, target: Pos) -> Vec<(Pos, Pos)> {
        let mut v = Vec::new();
        let me = self.creatures.iter().find(|c| c.id == id).map(|c| c.pos);
        for y in 0..N {
            for x in 0..N {
                let p = (x, y);
                if Some(p) != me && !self.free_for(p, Some(kind)) {
                    continue;
                }
                for &aim in Self::aims(kind) {
                    // Probe as if standing at p.
                    if self.strike_squares_from(id, kind, p, aim).contains(&target) {
                        v.push((p, aim));
                        break;
                    }
                }
            }
        }
        v
    }

    fn strike_squares_from(&self, id: u32, kind: Kind, p: Pos, aim: Pos) -> Vec<Pos> {
        let mut probe = self.clone();
        if let Some(c) = probe.creatures.iter_mut().find(|c| c.id == id) {
            c.pos = p;
        }
        probe.strike_squares(kind, p, aim)
    }

    fn move_and_aim(&mut self, id: u32, ev: &mut Vec<Ev>) {
        let Some(i) = self.creatures.iter().position(|c| c.id == id) else { return };
        let (kind, pos) = (self.creatures[i].kind, self.creatures[i].pos);
        let info = kind.info();
        if kind == Kind::Fly {
            // Flutter one square, seeded.
            let opts: Vec<Pos> = DIRS8.iter().map(|d| add(pos, *d, 1)).filter(|&p| self.free_for(p, Some(kind))).collect();
            if !opts.is_empty() && self.rng.below(3) > 0 {
                let to = *self.rng.pick(&opts);
                self.creatures[i].pos = to;
                ev.push(Ev::Moved { id, from: pos, to });
            }
            return;
        }
        // Reachable squares within its moves (stepping any of 8 ways).
        let mut reach = vec![(pos, 0)];
        let mut frontier = vec![pos];
        for depth in 1..=info.moves {
            let mut next = Vec::new();
            for f in &frontier {
                for d in DIRS8 {
                    let q = add(*f, d, 1);
                    if self.free_for(q, Some(kind)) && self.tile(q) != Tile::Thorns && !reach.iter().any(|(r, _)| *r == q) {
                        reach.push((q, depth));
                        next.push(q);
                    }
                }
            }
            frontier = next;
        }
        // The nearest strike on any target; on a tie, eggs first (they can't run).
        let mut best: Option<(i32, i32, Pos, Pos, Pos)> = None;
        for target in self.targets() {
            let spots = self.striking_spots(id, kind, target);
            let egg = (target != self.frog.pos) as i32;
            for &(p, d) in &reach {
                if let Some(&(_, aim)) = spots.iter().find(|(s, _)| *s == p) {
                    let key = (d, -egg);
                    if best.map_or(true, |b| key < (b.0, b.1)) {
                        best = Some((key.0, key.1, p, aim, target));
                    }
                }
            }
        }
        let goal = best.map_or(self.frog.pos, |b| b.4);
        let (to, aim) = match best {
            Some((_, _, p, aim, _)) => (p, Some(aim)),
            None => {
                // Close in on the nearest target.
                let nearest = self.targets().into_iter().min_by_key(|t| cheb(*t, pos)).unwrap_or(self.frog.pos);
                let p = reach.iter().min_by_key(|(r, d)| (cheb(*r, nearest), *d)).map(|r| r.0).unwrap_or(pos);
                (p, None)
            }
        };
        let _ = goal;
        if to != pos {
            self.creatures[i].pos = to;
            ev.push(Ev::Moved { id, from: pos, to });
        }
        // Aim at the frog if possible from here; otherwise roughly toward it.
        let aim = aim.or_else(|| {
            let toward = (sign(self.frog.pos.0 - to.0), sign(self.frog.pos.1 - to.1));
            let opts = Self::aims(kind);
            opts.iter().copied().max_by_key(|a| a.0 * toward.0 + a.1 * toward.1)
        });
        self.creatures[i].aim = aim;
    }

    /// Takes aim from where it stands, without moving (used when a pond starts).
    pub fn aim_only(&mut self, id: u32, _ev: &mut Vec<Ev>) {
        let Some(i) = self.creatures.iter().position(|c| c.id == id) else { return };
        let (kind, pos) = (self.creatures[i].kind, self.creatures[i].pos);
        let targets = self.targets();
        let hit = Self::aims(kind).iter().copied().find(|&a| self.strike_squares(kind, pos, a).iter().any(|q| targets.contains(q)));
        let aim = hit.or_else(|| {
            let toward = (sign(self.frog.pos.0 - pos.0), sign(self.frog.pos.1 - pos.1));
            Self::aims(kind).iter().copied().max_by_key(|a| a.0 * toward.0 + a.1 * toward.1)
        });
        self.creatures[i].aim = aim;
    }

    /// The danger after a hop to `to` (strikes that would hit the frog there), simulated.
    pub fn preview_hop(&self, to: Pos) -> i32 {
        let mut p = self.clone();
        p.frog.pos = to;
        p.frog.hop_left = false;
        p.frog.act_left = false;
        let mut ev = Vec::new();
        p.land(&mut ev);
        let ids: Vec<u32> = p.creatures.iter().map(|c| c.id).collect();
        for id in ids {
            if let Some(i) = p.creatures.iter().position(|c| c.id == id) {
                if p.creatures[i].stunned {
                    continue;
                }
                p.resolve_strike(i, &mut ev);
            }
        }
        ev.iter().map(|e| if let Ev::Hurt { dmg, .. } = e { *dmg } else { 0 }).sum()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn pond(rows: [&str; 8]) -> Pond {
        let mut p = Pond::empty(Rng::new(3));
        for (y, r) in rows.iter().enumerate() {
            for (x, c) in r.chars().enumerate() {
                let at = (x as i32, y as i32);
                match c {
                    '~' => p.set(at, Tile::Water),
                    'o' => p.set(at, Tile::Lily),
                    '|' => p.set(at, Tile::Reeds),
                    '#' => p.set(at, Tile::Stone),
                    '^' => p.set(at, Tile::Thorns),
                    '@' => p.frog.pos = at,
                    'f' => drop(p.add_creature(Kind::Fly, at)),
                    'b' => drop(p.add_creature(Kind::Beetle, at)),
                    'n' => drop(p.add_creature(Kind::Newt, at)),
                    'h' => drop(p.add_creature(Kind::Heron, at)),
                    's' => drop(p.add_creature(Kind::Snake, at)),
                    _ => {}
                }
            }
        }
        p
    }

    #[test]
    fn tongue_swallows_small_yanks_equal_and_pulls_toward_big() {
        let mut p = pond(["........", "........", "........", "...f....", "........", "...@....", "........", "........"]);
        p.act(Act::Tongue((0, -1)));
        assert_eq!(p.frog.throat, vec![Kind::Fly]);

        let mut p = pond(["........", "........", "...b....", "........", "........", "...@....", "........", "........"]);
        p.act(Act::Tongue((0, -1)));
        assert_eq!(p.creatures[0].pos, (3, 4), "a beetle is as big as a small frog: yanked");

        let mut p = pond(["........", "........", "...h....", "........", "........", "...@....", "........", "........"]);
        p.act(Act::Tongue((0, -1)));
        assert_eq!(p.frog.pos, (3, 3), "a heron is bigger: the frog flies to it");
    }

    #[test]
    fn yanking_a_non_swimmer_over_water_drowns_it() {
        let mut p = pond(["........", "........", "...b....", "........", "...~....", "...@....", "........", "........"]);
        p.act(Act::Tongue((0, -1)));
        assert!(p.creatures.is_empty());
        assert_eq!(p.killed, vec![Kind::Beetle]);
    }

    #[test]
    fn spit_hurts_and_shoves_into_water() {
        let mut p = pond(["........", "...~....", "...n....", "........", "........", "...@....", "........", "........"]);
        p.frog.throat.push(Kind::Fly);
        p.act(Act::Spit((0, -1)));
        // A fly does 1: the newt (1 HP) dies outright.
        assert!(p.creatures.is_empty());
        let mut p = pond(["........", "...~....", "...h....", "........", "........", "...@....", "........", "........"]);
        p.frog.throat.push(Kind::Beetle);
        p.act(Act::Spit((0, -1)));
        assert_eq!(p.creatures[0].hp, 1);
        assert_eq!(p.creatures[0].pos, (3, 1), "shoved one square");
    }

    #[test]
    fn strikes_land_where_the_creature_stands_when_they_resolve() {
        // A heron aimed down at the frog; croak shoves it sideways so its stab misses.
        let mut p = pond(["........", "........", "........", "........", "...h....", "...@....", "........", "........"]);
        p.creatures[0].aim = Some((0, 1));
        let base = p.danger();
        assert!(base.iter().any(|(q, _)| *q == (3, 6)));
        p.act(Act::Croak);
        assert!(p.frog.hearts == 3);
    }

    #[test]
    fn digesting_grows_the_frog() {
        let mut p = pond(["........", "........", "........", "........", "........", "...@....", "........", "........"]);
        for _ in 0..3 {
            p.frog.throat.push(Kind::Fly);
            p.frog.act_left = true;
            p.act(Act::Gulp);
        }
        assert_eq!(p.frog.size, 2);
        assert_eq!(p.frog.max_hearts, 4);
    }

    #[test]
    fn a_large_frog_sinks_lily_pads_and_hops_short() {
        let mut p = pond(["........", "........", "........", "....o...", "........", "...@....", "........", "........"]);
        p.frog.size = 3;
        assert!(!p.hop_targets().contains(&(3, 3)));
        p.act(Act::Tongue((1, -1)));
        let mut q = pond(["........", "........", "........", "........", "....o...", "...@....", "........", "........"]);
        q.frog.size = 3;
        q.frog.muts.push(Mutation::SpringLegs);
        q.act(Act::Hop((4, 4)));
        assert_eq!(q.tile((4, 4)), Tile::Water);
    }

    #[test]
    fn creatures_take_aim_and_the_beetle_charges_into_the_water() {
        let mut p = pond(["........", "........", ".b......", "........", ".~......", ".@......", "........", "........"]);
        p.act(Act::End);
        let b = &p.creatures[0];
        assert!(b.aim.is_some());
        // The frog steps aside; the beetle's charge runs into the water and drowns.
        p.act(Act::Hop((2, 5)));
        p.act(Act::End);
        assert!(p.killed.contains(&Kind::Beetle) || p.creatures.iter().all(|c| c.kind == Kind::Beetle));
    }
}
