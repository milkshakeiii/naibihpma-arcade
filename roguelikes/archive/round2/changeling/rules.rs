//! Changeling's rules: one burrow (an 8x8 board), one action at a time. Pure and deterministic,
//! so the hover preview can simulate any move exactly.

use crate::rng::Rng;

pub const N: i32 = 8;
pub type Pos = (i32, i32);

#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash, PartialOrd, Ord)]
pub enum Kind {
    Wanderer,
    Newt,
    Beetle,
    Frog,
    Cricket,
    Heron,
    Snake,
    Salamander,
    Glowtoad,
    Grof,
}

pub const ORTH: [Pos; 4] = [(1, 0), (-1, 0), (0, 1), (0, -1)];
pub const DIAG: [Pos; 4] = [(1, 1), (-1, 1), (1, -1), (-1, -1)];
pub const ALL8: [Pos; 8] = [(1, 0), (-1, 0), (0, 1), (0, -1), (1, 1), (-1, 1), (1, -1), (-1, -1)];

impl Kind {
    pub fn name(self) -> &'static str {
        match self {
            Kind::Wanderer => "YOURSELF",
            Kind::Newt => "NEWT",
            Kind::Beetle => "BEETLE",
            Kind::Frog => "FROG",
            Kind::Cricket => "CRICKET",
            Kind::Heron => "HERON",
            Kind::Snake => "SNAKE",
            Kind::Salamander => "SALAMANDER",
            Kind::Glowtoad => "GLOWTOAD",
            Kind::Grof => "GROF",
        }
    }

    /// How it moves and strikes, in words.
    pub fn shape_text(self) -> &'static str {
        match self {
            Kind::Wanderer => "ONE STEP, ANY DIRECTION",
            Kind::Newt => "ONE STEP, STRAIGHT",
            Kind::Beetle => "ONE STEP, DIAGONAL",
            Kind::Frog => "LEAPS TWO-AND-ONE",
            Kind::Cricket => "HOPS EXACTLY TWO, ANY DIRECTION",
            Kind::Heron => "STRAIGHT LINES, UP TO 3",
            Kind::Snake => "DIAGONAL LINES, UP TO 3",
            Kind::Salamander => "LEAPS THREE-AND-ONE",
            Kind::Glowtoad => "LINES UP TO 2, ANY DIRECTION",
            Kind::Grof => "LINES UP TO 3, ANY DIRECTION",
        }
    }

    /// One extra rule, if it has one.
    pub fn trait_text(self) -> Option<&'static str> {
        match self {
            Kind::Heron => Some("WARY: WON'T STOP WHERE YOU COULD CATCH IT."),
            Kind::Glowtoad => Some("TOXIC: CATCHING IT COSTS YOU A HEART."),
            Kind::Grof => Some("SHRUGS OFF TWO CATCHES, THEN FLEES AND CALLS FOR HELP. CATCH HIM THREE TIMES."),
            _ => None,
        }
    }

    pub fn dew(self) -> i32 {
        match self {
            Kind::Wanderer | Kind::Grof => 0,
            Kind::Newt | Kind::Beetle => 1,
            Kind::Frog | Kind::Cricket => 2,
            Kind::Heron | Kind::Snake | Kind::Salamander => 3,
            Kind::Glowtoad => 4,
        }
    }

    fn leap_bases(self) -> &'static [Pos] {
        match self {
            Kind::Wanderer => &[(1, 0), (1, 1)],
            Kind::Newt => &[(1, 0)],
            Kind::Beetle => &[(1, 1)],
            Kind::Frog => &[(1, 2)],
            Kind::Cricket => &[(2, 0), (2, 2)],
            Kind::Salamander => &[(1, 3)],
            _ => &[],
        }
    }

    pub fn leaps(self) -> Vec<Pos> {
        let mut v = Vec::new();
        for &(a, b) in self.leap_bases() {
            for (x, y) in [(a, b), (b, a)] {
                for (sx, sy) in [(1, 1), (-1, 1), (1, -1), (-1, -1)] {
                    let p = (x * sx, y * sy);
                    if !v.contains(&p) {
                        v.push(p);
                    }
                }
            }
        }
        v
    }

    /// Line directions and range.
    pub fn lines(self) -> Option<(&'static [Pos], i32)> {
        match self {
            Kind::Heron => Some((&ORTH, 3)),
            Kind::Snake => Some((&DIAG, 3)),
            Kind::Glowtoad => Some((&ALL8, 2)),
            Kind::Grof => Some((&ALL8, 3)),
            _ => None,
        }
    }

    pub fn wary(self) -> bool {
        self == Kind::Heron
    }

    /// The shape on an open board, for diagrams.
    pub fn diagram(self, extra: i32) -> Vec<Pos> {
        let mut v = self.leaps();
        if let Some((dirs, r)) = self.lines() {
            for &(dx, dy) in dirs {
                for k in 1..=r + extra {
                    v.push((dx * k, dy * k));
                }
            }
        }
        v
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Cell {
    Ground,
    Rock,
    Burrow,
}

#[derive(Clone, Debug)]
pub struct Creature {
    pub id: u32,
    pub kind: Kind,
    pub pos: Pos,
    pub dazed: bool,
    pub fresh: bool,
    pub wards: i32,
    /// Where it will move after the player's turn (planned in the open, before the player moves).
    pub intent: Option<Pos>,
}

#[derive(Clone, Copy, Debug)]
pub struct Spawn {
    pub pos: Pos,
    pub kind: Kind,
    pub turns: i32,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash, PartialOrd, Ord)]
pub enum Charm {
    ThickSkin,
    TwoSkins,
    DewSense,
    Shell,
    LongShadow,
    Shove,
    SecondWind,
}

pub const ALL_CHARMS: [Charm; 7] = [Charm::ThickSkin, Charm::TwoSkins, Charm::DewSense, Charm::Shell, Charm::LongShadow, Charm::Shove, Charm::SecondWind];

impl Charm {
    pub fn name(self) -> &'static str {
        match self {
            Charm::ThickSkin => "THICK SKIN",
            Charm::TwoSkins => "TWO SKINS",
            Charm::DewSense => "DEW SENSE",
            Charm::Shell => "SHELL",
            Charm::LongShadow => "LONG SHADOW",
            Charm::Shove => "SHOVE",
            Charm::SecondWind => "SECOND WIND",
        }
    }
    pub fn text(self) -> &'static str {
        match self {
            Charm::ThickSkin => "ONE MORE HEART, AND MEND ONE NOW.",
            Charm::TwoSkins => "YOU KEEP YOUR LAST TWO SHAPES. EACH MOVE, YOU MAY USE EITHER ONE.",
            Charm::DewSense => "ONE MORE DEW FROM EVERY CATCH.",
            Charm::Shell => "THE FIRST STRIKE IN EACH BURROW IS BLOCKED.",
            Charm::LongShadow => "YOUR LINE SHAPES REACH 2 FARTHER.",
            Charm::Shove => "AFTER EVERY CATCH, EACH CREATURE NEXT TO YOU IS SHOVED ONE SQUARE AWAY FROM YOU. ONE THAT HAS NOWHERE TO GO IS STUNNED: IT SKIPS ITS NEXT TURN.",
            Charm::SecondWind => "LEAVE A BURROW WITHOUT BEING STRUCK AND YOU MEND A HEART.",
        }
    }
    pub fn cost(self) -> i32 {
        match self {
            Charm::ThickSkin => 8,
            Charm::TwoSkins => 10,
            Charm::DewSense => 6,
            Charm::Shell => 7,
            Charm::LongShadow => 6,
            Charm::Shove => 6,
            Charm::SecondWind => 6,
        }
    }
}

pub const MEND_COST: i32 = 4;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Action {
    Move(Pos),
    /// Click yourself: become yourself again (or, mid-pounce, stop).
    Rest,
}

#[derive(Clone, Debug, PartialEq)]
pub enum Event {
    Moved { from: Pos, to: Pos },
    Caught { kind: Kind, at: Pos, dew: i32 },
    Dew { at: Pos },
    Rested,
    Descended,
    Toxic,
    Pounce,
    Strike { id: u32, kind: Kind, from: Pos },
    Blocked { id: u32, from: Pos },
    Crept { id: u32, from: Pos, to: Pos },
    Pushed { id: u32, from: Pos, to: Pos },
    Dazed { id: u32 },
    Rustle { at: Pos, kind: Kind },
    Arrived { id: u32, kind: Kind, at: Pos },
    Ward { left: i32, from: Pos, to: Pos },
    Summoned { id: u32, kind: Kind, at: Pos },
    Opened,
    Died,
    Won,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Occ {
    Out,
    Rock,
    Empty,
    Burrow,
    Player,
    Creature(usize),
}

#[derive(Clone, Debug)]
pub struct Floor {
    pub cells: [[Cell; N as usize]; N as usize],
    pub dew: Vec<Pos>,
    pub creatures: Vec<Creature>,
    pub spawns: Vec<Spawn>,
    pub player: Pos,
    pub form: Kind,
    pub prev_form: Option<Kind>,
    pub hearts: i32,
    pub max_hearts: i32,
    pub dew_total: i32,
    pub charms: Vec<Charm>,
    pub shell_ready: bool,
    pub struck: bool,
    pub turn: u32,
    pub spawn_every: u32,
    pub spawn_pool: Vec<Kind>,
    pub pouncing: bool,
    pub next_id: u32,
    pub rng: Rng,
    pub over: Option<bool>, // Some(true) = descended / won, Some(false) = died
    pub won: bool,
    pub catches: Vec<Kind>,
    /// Catches still needed before the burrow opens (0 = open).
    pub seal: i32,
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Preview {
    pub hits: i32,
    pub blocked: bool,
    pub catch: Option<Kind>,
    pub toxic: bool,
    pub dew: i32,
    pub descend: bool,
    pub pounce: bool,
    pub dies: bool,
    pub wins: bool,
    pub opens: bool,
}

pub fn inb((x, y): Pos) -> bool {
    (0..N).contains(&x) && (0..N).contains(&y)
}

pub fn cheb(a: Pos, b: Pos) -> i32 {
    (a.0 - b.0).abs().max((a.1 - b.1).abs())
}

/// Chess-style square name (a-h left to right, 8 at the top), for the log.
pub fn sq((x, y): Pos) -> String {
    format!("{}{}", (b'a' + x as u8) as char, N - y)
}

impl Floor {
    pub fn empty(rng: Rng) -> Self {
        Self {
            cells: [[Cell::Ground; N as usize]; N as usize],
            dew: Vec::new(),
            creatures: Vec::new(),
            spawns: Vec::new(),
            player: (0, N - 1),
            form: Kind::Wanderer,
            prev_form: None,
            hearts: 3,
            max_hearts: 3,
            dew_total: 0,
            charms: Vec::new(),
            shell_ready: false,
            struck: false,
            turn: 0,
            spawn_every: 0,
            spawn_pool: Vec::new(),
            pouncing: false,
            next_id: 1,
            rng,
            over: None,
            won: false,
            catches: Vec::new(),
            seal: 0,
        }
    }

    pub fn has(&self, c: Charm) -> bool {
        self.charms.contains(&c)
    }

    pub fn cell(&self, (x, y): Pos) -> Cell {
        self.cells[y as usize][x as usize]
    }

    pub fn add_creature(&mut self, kind: Kind, pos: Pos) -> u32 {
        let id = self.next_id;
        self.next_id += 1;
        let wards = if kind == Kind::Grof { 3 } else { 1 };
        self.creatures.push(Creature { id, kind, pos, dazed: false, fresh: false, wards, intent: None });
        id
    }

    pub fn creature_at(&self, p: Pos) -> Option<usize> {
        self.creatures.iter().position(|c| c.pos == p)
    }

    pub fn occ(&self, p: Pos) -> Occ {
        if !inb(p) {
            return Occ::Out;
        }
        if p == self.player {
            return Occ::Player;
        }
        if let Some(i) = self.creature_at(p) {
            return Occ::Creature(i);
        }
        match self.cell(p) {
            Cell::Rock => Occ::Rock,
            Cell::Burrow => Occ::Burrow,
            Cell::Ground => Occ::Empty,
        }
    }

    /// Squares a shape can land on from `from`: leaps land anywhere but rocks; lines stop at
    /// rocks (excluded) and at the first creature or player (included). `empty_at` is treated
    /// as vacant (the mover's own square when planning).
    pub fn reach(&self, from: Pos, kind: Kind, extra: i32, empty_at: Option<Pos>) -> Vec<Pos> {
        let mut v = Vec::new();
        let occ = |p: Pos| if Some(p) == empty_at { if inb(p) { Occ::Empty } else { Occ::Out } } else { self.occ(p) };
        for (dx, dy) in kind.leaps() {
            let p = (from.0 + dx, from.1 + dy);
            if !matches!(occ(p), Occ::Out | Occ::Rock) {
                v.push(p);
            }
        }
        if let Some((dirs, r)) = kind.lines() {
            for &(dx, dy) in dirs {
                for k in 1..=r + extra {
                    let p = (from.0 + dx * k, from.1 + dy * k);
                    match occ(p) {
                        Occ::Out | Occ::Rock => break,
                        Occ::Player | Occ::Creature(_) => {
                            v.push(p);
                            break;
                        }
                        Occ::Empty | Occ::Burrow => v.push(p),
                    }
                }
            }
        }
        v
    }

    fn player_extra(&self) -> i32 {
        if self.has(Charm::LongShadow) {
            2
        } else {
            0
        }
    }

    /// Where the player may move now (excluding their own square).
    pub fn targets(&self) -> Vec<Pos> {
        let mut forms = vec![self.form];
        if let Some(p) = self.prev_form.filter(|_| self.has(Charm::TwoSkins)) {
            if p != self.form {
                forms.push(p);
            }
        }
        let mut v = Vec::new();
        for f in forms {
            for p in self.reach(self.player, f, self.player_extra(), None) {
                if !v.contains(&p) && p != self.player {
                    v.push(p);
                }
            }
        }
        if self.pouncing {
            v.retain(|&p| self.creature_at(p).is_some());
        }
        v
    }

    fn can_strike(&self, i: usize) -> bool {
        let c = &self.creatures[i];
        !c.dazed && !c.fresh && self.reach(c.pos, c.kind, 0, None).contains(&self.player)
    }

    /// Squares any creature could strike if the player stood there (the player lifted off the
    /// board, so lines run through their square). Used for the red marks off the lit squares.
    pub fn threatened(&self) -> Vec<Pos> {
        let mut ghost = self.clone();
        ghost.player = (-9, -9);
        let mut v = Vec::new();
        for i in 0..ghost.creatures.len() {
            let c = ghost.creatures[i].clone();
            if c.dazed || c.fresh {
                continue;
            }
            let mut spots = ghost.reach(c.pos, c.kind, 0, None);
            if let Some(to) = c.intent {
                ghost.creatures[i].pos = to;
                spots.extend(ghost.reach(to, c.kind, 0, None));
                ghost.creatures[i].pos = c.pos;
            }
            for p in spots {
                if !v.contains(&p) {
                    v.push(p);
                }
            }
        }
        v
    }

    pub fn act(&mut self, a: Action) -> Vec<Event> {
        let mut ev = self.apply_player(a);
        if self.over.is_none() && !self.pouncing {
            ev.extend(self.enemy_phase());
        }
        ev
    }

    /// The player's half of a turn.
    pub fn apply_player(&mut self, a: Action) -> Vec<Event> {
        let mut ev = Vec::new();
        match a {
            Action::Rest => {
                if self.pouncing {
                    self.pouncing = false;
                } else {
                    self.form = Kind::Wanderer;
                    self.prev_form = None;
                    ev.push(Event::Rested);
                }
            }
            Action::Move(to) => {
                if !self.targets().contains(&to) {
                    return ev;
                }
                let from = self.player;
                self.pouncing = false;
                ev.push(Event::Moved { from, to });
                if let Some(i) = self.creature_at(to) {
                    let kind = self.creatures[i].kind;
                    let mut dew = kind.dew() + if self.has(Charm::DewSense) && kind != Kind::Grof { 1 } else { 0 };
                    self.catches.push(kind);
                    if self.seal > 0 {
                        self.seal -= 1;
                        if self.seal == 0 {
                            ev.push(Event::Opened);
                        }
                    }
                    if kind == Kind::Grof {
                        self.creatures[i].wards -= 1;
                        let left = self.creatures[i].wards;
                        self.player = to;
                        self.take_shape(kind);
                        if left <= 0 {
                            self.creatures.remove(i);
                            ev.push(Event::Caught { kind, at: to, dew: 0 });
                            ev.push(Event::Won);
                            self.won = true;
                            self.over = Some(true);
                            return ev;
                        }
                        dew = 3;
                        self.dew_total += dew;
                        ev.push(Event::Caught { kind, at: to, dew });
                        let dest = self.flee_square();
                        self.creatures[i].pos = dest;
                        self.plan_intent(i);
                        ev.push(Event::Ward { left, from: to, to: dest });
                        for _ in 0..2 {
                            let spots: Vec<Pos> =
                                ALL8.iter().map(|d| (dest.0 + d.0, dest.1 + d.1)).filter(|&p| self.occ(p) == Occ::Empty).collect();
                            if spots.is_empty() || self.spawn_pool.is_empty() {
                                break;
                            }
                            let at = *self.rng.pick(&spots);
                            let k = *self.rng.pick(&self.spawn_pool.clone());
                            let id = self.add_creature(k, at);
                            self.creatures.last_mut().unwrap().fresh = true;
                            let last = self.creatures.len() - 1;
                            self.plan_intent(last);
                            ev.push(Event::Summoned { id, kind: k, at });
                        }
                    } else {
                        self.creatures.remove(i);
                        self.player = to;
                        self.dew_total += dew;
                        self.take_shape(kind);
                        ev.push(Event::Caught { kind, at: to, dew });
                        if kind == Kind::Glowtoad {
                            self.hearts -= 1;
                            self.struck = true;
                            ev.push(Event::Toxic);
                            if self.hearts <= 0 {
                                ev.push(Event::Died);
                                self.over = Some(false);
                                return ev;
                            }
                        }
                    }
                    if self.has(Charm::Shove) {
                        self.shove(&mut ev);
                    }
                    if !self.targets().iter().all(|&p| self.creature_at(p).is_none()) {
                        self.pouncing = true;
                        ev.push(Event::Pounce);
                    }
                } else {
                    self.player = to;
                }
                if let Some(k) = self.dew.iter().position(|&d| d == to) {
                    self.dew.remove(k);
                    self.dew_total += 1;
                    ev.push(Event::Dew { at: to });
                }
                if self.cell(to) == Cell::Burrow && self.seal == 0 {
                    self.pouncing = false;
                    ev.push(Event::Descended);
                    self.over = Some(true);
                }
            }
        }
        ev
    }

    fn take_shape(&mut self, kind: Kind) {
        if self.form != kind {
            self.prev_form = Some(self.form);
        }
        self.form = kind;
    }

    fn shove(&mut self, ev: &mut Vec<Event>) {
        let me = self.player;
        for i in 0..self.creatures.len() {
            let c = self.creatures[i].pos;
            if cheb(c, me) != 1 || self.creatures[i].kind == Kind::Grof {
                continue;
            }
            let to = (c.0 + (c.0 - me.0), c.1 + (c.1 - me.1));
            if self.occ(to) == Occ::Empty {
                self.creatures[i].pos = to;
                self.plan_intent(i);
                ev.push(Event::Pushed { id: self.creatures[i].id, from: c, to });
            } else {
                self.creatures[i].dazed = true;
                ev.push(Event::Dazed { id: self.creatures[i].id });
            }
        }
    }

    fn flee_square(&self) -> Pos {
        let mut best = (self.player, -1);
        for y in 0..N {
            for x in 0..N {
                let p = (x, y);
                if self.occ(p) == Occ::Empty {
                    let free_nb = ALL8.iter().filter(|d| self.occ((x + d.0, y + d.1)) == Occ::Empty).count() as i32;
                    let score = cheb(p, self.player) * 10 + free_nb.min(2);
                    if score > best.1 {
                        best = (p, score);
                    }
                }
            }
        }
        best.0
    }

    /// Squares from which creature `i` could strike the player, with its own square vacated.
    fn strike_squares(&self, i: usize) -> Vec<Pos> {
        let c = &self.creatures[i];
        let me = c.pos;
        let pl = self.player;
        let stand = |p: Pos| -> bool {
            inb(p) && self.cell(p) == Cell::Ground && (p == me || (self.creature_at(p).is_none() && p != pl))
        };
        let mut v = Vec::new();
        for (dx, dy) in c.kind.leaps() {
            let p = (pl.0 - dx, pl.1 - dy);
            if stand(p) {
                v.push(p);
            }
        }
        if let Some((dirs, r)) = c.kind.lines() {
            for &(dx, dy) in dirs {
                for k in 1..=r {
                    let p = (pl.0 + dx * k, pl.1 + dy * k);
                    if !inb(p) || self.cell(p) == Cell::Rock {
                        break;
                    }
                    if p != me && self.creature_at(p).is_some() {
                        break;
                    }
                    if stand(p) {
                        v.push(p);
                    }
                }
            }
        }
        v
    }

    /// Squares creature `i` could move to from `from` (free ground only).
    fn moves_from(&self, i: usize, from: Pos) -> Vec<Pos> {
        let me = self.creatures[i].pos;
        self.reach(from, self.creatures[i].kind, 0, Some(me))
            .into_iter()
            .filter(|&p| p != me && inb(p) && self.cell(p) == Cell::Ground && self.creature_at(p).is_none() && p != self.player)
            .collect()
    }

    /// Moves needed for creature `i` to stand where it could strike the player.
    fn approach(&self, i: usize, from: Pos, goals: &[Pos]) -> i32 {
        if goals.contains(&from) {
            return 0;
        }
        let mut seen = vec![from];
        let mut frontier = vec![from];
        for depth in 1..=10 {
            let mut next = Vec::new();
            for &f in &frontier {
                for p in self.moves_from(i, f) {
                    if goals.contains(&p) {
                        return depth;
                    }
                    if !seen.contains(&p) {
                        seen.push(p);
                        next.push(p);
                    }
                }
            }
            if next.is_empty() {
                break;
            }
            frontier = next;
        }
        99
    }

    /// The creatures' half of a turn, then their plans for the next one.
    pub fn enemy_phase(&mut self) -> Vec<Event> {
        let ev = self.creatures_act();
        if self.over.is_none() {
            self.plan_intents();
        }
        ev
    }

    /// Strikes, planned moves, arrivals (no new plans: the preview doesn't need them).
    fn creatures_act(&mut self) -> Vec<Event> {
        let mut ev = Vec::new();
        for i in 0..self.creatures.len() {
            if self.creatures[i].fresh {
                self.creatures[i].fresh = false;
                continue;
            }
            if self.creatures[i].dazed {
                self.creatures[i].dazed = false;
                continue;
            }
            let (id, pos) = (self.creatures[i].id, self.creatures[i].pos);
            if !self.can_strike(i) {
                // Follow the plan made in the open before the player's turn...
                if let Some(to) = self.creatures[i].intent {
                    if self.occ(to) == Occ::Empty && self.moves_from(i, pos).contains(&to) {
                        self.creatures[i].pos = to;
                        ev.push(Event::Crept { id, from: pos, to });
                    }
                }
            }
            // ...and strike if the player is in reach, from where it stood or where it landed.
            if self.can_strike(i) && self.strike(i, &mut ev) {
                return ev;
            }
        }
        self.turn += 1;
        // Arrivals announced two turns ago.
        let mut k = 0;
        while k < self.spawns.len() {
            self.spawns[k].turns -= 1;
            let s = self.spawns[k];
            if s.turns <= 0 && self.occ(s.pos) == Occ::Empty {
                let id = self.add_creature(s.kind, s.pos);
                ev.push(Event::Arrived { id, kind: s.kind, at: s.pos });
                self.spawns.remove(k);
            } else {
                k += 1;
            }
        }
        if self.spawn_every > 0 && self.turn % self.spawn_every == 0 && !self.spawn_pool.is_empty() {
            let edge: Vec<Pos> = (0..N)
                .flat_map(|t| [(t, 0), (t, N - 1), (0, t), (N - 1, t)])
                .filter(|&p| self.occ(p) == Occ::Empty && cheb(p, self.player) >= 3 && !self.spawns.iter().any(|s| s.pos == p))
                .collect();
            if !edge.is_empty() {
                let pos = *self.rng.pick(&edge);
                let kind = *self.rng.pick(&self.spawn_pool.clone());
                self.spawns.push(Spawn { pos, kind, turns: 2 });
                ev.push(Event::Rustle { at: pos, kind });
            }
        }
        ev
    }

    /// Creature `i` strikes the player. Returns true if that was the last heart.
    fn strike(&mut self, i: usize, ev: &mut Vec<Event>) -> bool {
        let (id, kind, pos) = (self.creatures[i].id, self.creatures[i].kind, self.creatures[i].pos);
        if self.shell_ready {
            self.shell_ready = false;
            ev.push(Event::Blocked { id, from: pos });
            return false;
        }
        self.hearts -= 1;
        self.struck = true;
        ev.push(Event::Strike { id, kind, from: pos });
        if self.hearts <= 0 {
            ev.push(Event::Died);
            self.over = Some(false);
            return true;
        }
        false
    }

    /// Every creature decides, in the open, where it will move after the player's next turn:
    /// toward a square from which it could strike the player where they stand now. A creature
    /// that can already strike plans to stay and strike.
    pub fn plan_intents(&mut self) {
        for c in self.creatures.iter_mut() {
            c.intent = None;
        }
        let reach = self.targets_ignoring_pounce();
        for i in 0..self.creatures.len() {
            self.plan_with(i, &reach);
        }
    }

    pub fn plan_intent(&mut self, i: usize) {
        let reach = self.targets_ignoring_pounce();
        self.plan_with(i, &reach);
    }

    fn plan_with(&mut self, i: usize, player_reach: &[Pos]) {
        self.creatures[i].intent = None;
        let (kind, pos) = (self.creatures[i].kind, self.creatures[i].pos);
        if self.reach(pos, kind, 0, None).contains(&self.player) {
            return;
        }
        let claimed: Vec<Pos> = self.creatures.iter().enumerate().filter(|(j, _)| *j != i).filter_map(|(_, c)| c.intent).collect();
        let goals = self.strike_squares(i);
        let key = |f: &Floor, p: Pos, stay: bool| {
            let unsafe_sq = kind.wary() && player_reach.contains(&p);
            (unsafe_sq, f.approach(i, p, &goals), cheb(p, f.player), !stay)
        };
        let mut best = (key(self, pos, true), pos);
        for p in self.moves_from(i, pos) {
            if claimed.contains(&p) {
                continue;
            }
            let k = key(self, p, false);
            if k < best.0 {
                best = (k, p);
            }
        }
        if best.1 != pos {
            self.creatures[i].intent = Some(best.1);
        }
    }

    fn targets_ignoring_pounce(&self) -> Vec<Pos> {
        let mut f = self.clone();
        f.pouncing = false;
        f.targets()
    }

    /// Exactly what would happen if the player moved to `to` (and stopped there).
    pub fn preview(&self, a: Action) -> Preview {
        let mut f = self.clone();
        let mut ev = f.apply_player(a);
        let pounce = f.pouncing;
        f.pouncing = false;
        if f.over.is_none() {
            ev.extend(f.creatures_act());
        }
        let mut p = Preview { pounce, ..Default::default() };
        for e in ev {
            match e {
                Event::Strike { .. } => p.hits += 1,
                Event::Blocked { .. } => p.blocked = true,
                Event::Caught { kind, dew, .. } => {
                    p.catch = Some(kind);
                    p.dew += dew;
                }
                Event::Dew { .. } => p.dew += 1,
                Event::Toxic => p.toxic = true,
                Event::Descended => p.descend = true,
                Event::Died => p.dies = true,
                Event::Won => p.wins = true,
                Event::Opened => p.opens = true,
                _ => {}
            }
        }
        p
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn board(rows: [&str; 8]) -> Floor {
        let mut f = Floor::empty(Rng::new(1));
        for (y, r) in rows.iter().enumerate() {
            for (x, c) in r.chars().enumerate() {
                let p = (x as i32, y as i32);
                match c {
                    'o' => f.cells[y][x] = Cell::Rock,
                    'B' => f.cells[y][x] = Cell::Burrow,
                    '@' => f.player = p,
                    'd' => f.dew.push(p),
                    'n' => drop(f.add_creature(Kind::Newt, p)),
                    'b' => drop(f.add_creature(Kind::Beetle, p)),
                    'f' => drop(f.add_creature(Kind::Frog, p)),
                    'h' => drop(f.add_creature(Kind::Heron, p)),
                    'g' => drop(f.add_creature(Kind::Glowtoad, p)),
                    'G' => drop(f.add_creature(Kind::Grof, p)),
                    _ => {}
                }
            }
        }
        f.plan_intents();
        f
    }

    #[test]
    fn shapes_expand_by_symmetry() {
        assert_eq!(Kind::Wanderer.leaps().len(), 8);
        assert_eq!(Kind::Frog.leaps().len(), 8);
        assert_eq!(Kind::Cricket.leaps().len(), 8);
        assert_eq!(Kind::Newt.leaps().len(), 4);
        assert_eq!(Kind::Salamander.leaps().len(), 8);
    }

    #[test]
    fn catching_takes_the_shape_and_rest_returns_it() {
        let mut f = board(["B.......", "........", "........", "........", "........", "........", "...n....", "...@...."]);
        f.act(Action::Move((3, 6)));
        assert_eq!(f.form, Kind::Newt);
        assert_eq!(f.dew_total, 1);
        let t = f.targets();
        assert!(t.contains(&(3, 5)) && !t.contains(&(4, 5)), "newt shape is one straight step");
        f.act(Action::Rest);
        assert_eq!(f.form, Kind::Wanderer);
    }

    #[test]
    fn lines_stop_at_rocks_and_creatures() {
        let f = board(["B.......", "........", "........", "h..o....", "........", "........", "........", "@......."]);
        let r = f.reach((0, 3), Kind::Heron, 0, None);
        assert!(r.contains(&(2, 3)) && !r.contains(&(3, 3)) && !r.contains(&(4, 3)));
        assert!(r.contains(&(0, 6)) && !r.contains(&(0, 7)), "range 3");
    }

    #[test]
    fn creatures_strike_when_they_can_reach_you() {
        let mut f = board(["B.......", "........", "........", "........", "........", "........", "..f.....", "........"]);
        f.player = (6, 0);
        // The frog's leaps from (2,6) include (3,4); moving the player there gets struck.
        f.player = (3, 3);
        let p = f.preview(Action::Move((3, 4)));
        assert_eq!(p.hits, 1);
        let mut g = f.clone();
        g.act(Action::Move((3, 4)));
        assert_eq!(g.hearts, 2);
    }

    #[test]
    fn moving_away_along_a_line_is_previewed_as_struck() {
        // The heron's line is blocked by the player; stepping back along it is still a hit.
        let f = board(["B.......", "........", "........", "h.@.....", "........", "........", "........", "........"]);
        let pv = f.preview(Action::Move((3, 3)));
        assert_eq!(pv.hits, 1);
        assert!(f.threatened().contains(&(3, 3)));
    }

    #[test]
    fn creatures_approach_a_striking_square() {
        let mut f = board(["B.......", "........", "........", "........", "....n...", "........", "........", "...@...."]);
        f.act(Action::Rest);
        let c = &f.creatures[0];
        assert!(cheb(c.pos, (3, 7)) < cheb((4, 4), (3, 7)));
    }

    #[test]
    fn wary_heron_keeps_out_of_reach() {
        let mut f = board(["B.......", "........", "....h...", "........", "........", "........", "........", "...@...."]);
        for _ in 0..4 {
            f.act(Action::Rest);
            let c = f.creatures[0].pos;
            assert!(!f.targets().contains(&c), "heron stopped in reach at {c:?}");
        }
    }

    #[test]
    fn glowtoad_costs_a_heart_and_grof_needs_three_catches() {
        let mut f = board(["B.......", "........", "........", "........", "........", "........", "...g....", "...@...."]);
        f.act(Action::Move((3, 6)));
        assert_eq!(f.hearts, 2);
        assert_eq!(f.form, Kind::Glowtoad);

        let mut g = board(["........", "........", "........", "........", "........", "........", "...G....", "...@...."]);
        g.spawn_pool = vec![Kind::Newt];
        g.hearts = 9;
        let mut catches = 0;
        for _ in 0..200 {
            if g.over.is_some() {
                break;
            }
            let t = g.targets();
            let grof = g.creatures.iter().find(|c| c.kind == Kind::Grof).map(|c| c.pos);
            if let Some(gp) = grof.filter(|p| t.contains(p)) {
                g.act(Action::Move(gp));
                catches += 1;
            } else if let Some(&c) = t.iter().find(|&&p| g.creature_at(p).is_some()) {
                g.act(Action::Move(c));
            } else {
                let gp = grof.unwrap();
                let step = *t.iter().min_by_key(|&&p| cheb(p, gp)).unwrap();
                g.act(Action::Move(step));
            }
            g.hearts = 9;
        }
        assert!(g.won);
        assert_eq!(catches, 3);
    }

    #[test]
    fn spawns_are_announced_then_arrive() {
        let mut f = board(["B.......", "........", "........", "........", "........", "........", "........", "...@...."]);
        f.spawn_every = 2;
        f.spawn_pool = vec![Kind::Newt];
        let mut rustled = false;
        let mut arrived = false;
        for _ in 0..6 {
            for e in f.act(Action::Rest) {
                rustled |= matches!(e, Event::Rustle { .. });
                arrived |= matches!(e, Event::Arrived { .. });
            }
        }
        assert!(rustled && arrived);
    }

    #[test]
    fn catches_chain_before_creatures_move() {
        let mut f = board(["B.......", "........", "........", "........", ".....b..", "........", "....n...", "...@...."]);
        // Catch the newt diagonally: the newt shape (one straight step) can't reach the beetle.
        let ev = f.act(Action::Move((4, 6)));
        assert!(!ev.contains(&Event::Pounce));
        let mut g = board(["B.......", "........", "........", "........", "........", ".....b..", "....b...", "...@...."]);
        let ev = g.act(Action::Move((4, 6)));
        assert!(ev.contains(&Event::Pounce), "beetle shape reaches the second beetle");
        assert!(g.pouncing);
        g.act(Action::Move((5, 5)));
        assert_eq!(g.creatures.len(), 0);
    }

    #[test]
    fn creatures_follow_the_plan_they_showed() {
        // The frog plans its move toward where the player stands now; when the player steps
        // away, it still goes where its arrow pointed.
        let mut f = board(["B.......", "........", "........", "........", "........", "..f.....", "........", "......@."]);
        let planned = f.creatures[0].intent.expect("frog has a plan");
        f.act(Action::Move((7, 7)));
        assert_eq!(f.creatures[0].pos, planned);
    }

    #[test]
    fn shell_blocks_the_first_strike() {
        let mut f = board(["B.......", "........", "........", "........", "........", "........", "...n....", "....@..."]);
        f.shell_ready = true;
        f.charms.push(Charm::Shell);
        // Newt at (3,6) strikes (4,6) and (3,7); step to (3,7)... that's adjacent straight: struck.
        let ev = f.act(Action::Move((4, 6)));
        assert!(ev.iter().any(|e| matches!(e, Event::Blocked { .. })));
        assert_eq!(f.hearts, 3);
    }
}
