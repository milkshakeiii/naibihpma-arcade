//! A Conduit contest: deterministic and tick-based, after the player's curses engine. Every
//! action sets a cooldown in ticks; units act when it reaches zero. Break the rival conduit.

use super::data::*;

pub const W: i32 = 15;
pub const H: i32 = 7;
pub const CONDUIT_HP: i32 = 30;
pub const TICK_CAP: u32 = 900;
pub type Pos = (i32, i32);

#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash)]
pub enum Side {
    Blue,
    Red,
}

impl Side {
    pub fn idx(self) -> usize {
        self as usize
    }
    pub fn other(self) -> Side {
        match self {
            Side::Blue => Side::Red,
            Side::Red => Side::Blue,
        }
    }
}

/// The three squares of each side's conduit.
pub fn conduit_tiles(side: Side) -> [Pos; 3] {
    let x = if side == Side::Blue { 0 } else { W - 1 };
    [(x, 2), (x, 3), (x, 4)]
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Target {
    Unit(usize),
    Conduit(Side),
}

#[derive(Clone, Debug)]
pub struct Unit {
    pub side: Side,
    pub build: Build,
    pub stats: Stats,
    pub pos: Pos,
    pub hp: i32,
    pub cd: i32,
    pub burn: i32,
    pub alive: bool,
    pub moved_from: Pos,
    pub moved_at: u32,
    pub dealt: i32,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Outcome {
    Won,
    Lost,
    Draw,
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Fx {
    Hit { from: Pos, to: Pos, dmg_type: Dmg, dmg: i32, primary: bool },
    Heal { from: Pos, to: Pos, amount: i32 },
    Burn { at: Pos },
    Spiked { at: Pos },
    Died { at: Pos, side: Side },
    ConduitHit { side: Side, dmg: i32 },
}

#[derive(Clone, Debug)]
pub struct Battle {
    pub units: Vec<Unit>,
    pub conduit_hp: [i32; 2],
    pub tick: u32,
    pub over: Option<Outcome>,
    /// Effects of recent ticks, for drawing: (tick, effect).
    pub fx: Vec<(u32, Fx)>,
}

pub fn inb((x, y): Pos) -> bool {
    (0..W).contains(&x) && (0..H).contains(&y)
}

fn manh(a: Pos, b: Pos) -> i32 {
    (a.0 - b.0).abs() + (a.1 - b.1).abs()
}

const DIRS: [Pos; 4] = [(1, 0), (-1, 0), (0, 1), (0, -1)];

/// Starting squares: front column for short reach, back column for long reach or healing.
pub fn formation(side: Side, builds: &[Build]) -> Vec<Pos> {
    let rows = [3, 1, 5, 2, 4, 0, 6];
    let (front, back) = if side == Side::Blue { (3, 2) } else { (W - 4, W - 3) };
    let mut taken: Vec<Pos> = Vec::new();
    builds
        .iter()
        .map(|b| {
            let s = b.stats();
            let long = s.heal || s.attacks.iter().map(|a| a.range).max().unwrap_or(1) >= 3;
            let col = if long { back } else { front };
            let row = rows.iter().copied().find(|&r| !taken.contains(&(col, r))).unwrap_or(0);
            taken.push((col, row));
            (col, row)
        })
        .collect()
}

impl Battle {
    pub fn new(blue: &[Build], red: &[Build]) -> Self {
        let bp = formation(Side::Blue, blue);
        let rp = formation(Side::Red, red);
        let mut units = Vec::new();
        // Interleave the sides so neither always acts first.
        for i in 0..blue.len().max(red.len()) {
            for (side, builds, pos) in [(Side::Blue, blue, &bp), (Side::Red, red, &rp)] {
                if let Some(b) = builds.get(i) {
                    let stats = b.stats();
                    units.push(Unit {
                        side,
                        build: b.clone(),
                        hp: stats.max_hp,
                        stats,
                        pos: pos[i],
                        cd: 2 + i as i32,
                        burn: 0,
                        alive: true,
                        moved_from: pos[i],
                        moved_at: 0,
                        dealt: 0,
                    });
                }
            }
        }
        Self { units, conduit_hp: [CONDUIT_HP; 2], tick: 0, over: None, fx: Vec::new() }
    }

    pub fn unit_at(&self, p: Pos) -> Option<usize> {
        self.units.iter().position(|u| u.alive && u.pos == p)
    }

    fn blocked(&self, p: Pos) -> bool {
        !inb(p) || self.unit_at(p).is_some() || conduit_tiles(Side::Blue).contains(&p) || conduit_tiles(Side::Red).contains(&p)
    }

    pub fn target_pos(&self, t: Target, from: Pos) -> Pos {
        match t {
            Target::Unit(j) => self.units[j].pos,
            Target::Conduit(s) => *conduit_tiles(s).iter().min_by_key(|&&c| manh(c, from)).unwrap(),
        }
    }

    fn dist(&self, from: Pos, t: Target) -> i32 {
        manh(from, self.target_pos(t, from))
    }

    fn enemies(&self, i: usize) -> impl Iterator<Item = usize> + '_ {
        let side = self.units[i].side;
        (0..self.units.len()).filter(move |&j| self.units[j].alive && self.units[j].side != side)
    }

    /// What unit `i`'s mind wants to fight right now.
    pub fn choose_target(&self, i: usize) -> Option<Target> {
        let u = &self.units[i];
        let foe_conduit = Target::Conduit(u.side.other());
        let nearest = self.enemies(i).min_by_key(|&j| (manh(u.pos, self.units[j].pos), self.units[j].hp, j));
        match u.stats.brain {
            Brain::Brute => Some(nearest.map_or(foe_conduit, Target::Unit)),
            Brain::Hunter => Some(self.enemies(i).min_by_key(|&j| (self.units[j].hp, manh(u.pos, self.units[j].pos), j)).map_or(foe_conduit, Target::Unit)),
            Brain::Raider => Some(foe_conduit),
            Brain::Guard => {
                let home = Target::Conduit(u.side);
                let close = self.enemies(i).filter(|&j| self.dist(self.units[j].pos, home) <= 4).min_by_key(|&j| (manh(u.pos, self.units[j].pos), j));
                match close {
                    Some(j) => Some(Target::Unit(j)),
                    None if self.enemies(i).next().is_none() => Some(foe_conduit),
                    None => None,
                }
            }
        }
    }

    fn defense(&self, t: Target, d: Dmg) -> i32 {
        match t {
            Target::Conduit(_) => 0,
            Target::Unit(j) => {
                let s = &self.units[j].stats;
                match d {
                    Dmg::Melee => s.armor,
                    Dmg::Ranged => s.dodge,
                    Dmg::Magic => s.ward,
                }
            }
        }
    }

    /// Damage `a` would do to `t` on a hit.
    pub fn damage(&self, a: &Attack, t: Target) -> i32 {
        let mut d = (a.dmg - self.defense(t, a.dmg_type)).max(1);
        if let Target::Unit(j) = t {
            if self.units[j].stats.stone_gut {
                d = (d + 1) / 2;
            }
        }
        d
    }

    pub fn step(&mut self) {
        if self.over.is_some() {
            return;
        }
        self.tick += 1;
        let t = self.tick;
        self.fx.retain(|(ft, _)| t.saturating_sub(*ft) < 8);
        for i in 0..self.units.len() {
            if !self.units[i].alive {
                continue;
            }
            if self.units[i].burn > 0 && t % 4 == 0 {
                self.units[i].burn -= 1;
                self.units[i].hp -= 1;
                let at = self.units[i].pos;
                self.fx.push((t, Fx::Burn { at }));
                if self.check_death(i) {
                    continue;
                }
            }
            if self.units[i].stats.regen && t % 10 == 0 && self.units[i].hp < self.units[i].stats.max_hp {
                self.units[i].hp += 1;
            }
            if self.units[i].cd > 0 {
                self.units[i].cd -= 1;
                continue;
            }
            self.act(i);
            if self.over.is_some() {
                return;
            }
        }
        if t >= TICK_CAP {
            let (b, r) = (self.conduit_hp[0], self.conduit_hp[1]);
            let hp = |s: Side| self.units.iter().filter(|u| u.alive && u.side == s).map(|u| u.hp).sum::<i32>();
            self.over = Some(if b > r || (b == r && hp(Side::Blue) > hp(Side::Red)) {
                Outcome::Won
            } else if r > b || hp(Side::Red) > hp(Side::Blue) {
                Outcome::Lost
            } else {
                Outcome::Draw
            });
        }
    }

    fn check_death(&mut self, i: usize) -> bool {
        if self.units[i].hp <= 0 && self.units[i].alive {
            self.units[i].alive = false;
            let (at, side) = (self.units[i].pos, self.units[i].side);
            self.fx.push((self.tick, Fx::Died { at, side }));
            return true;
        }
        false
    }

    fn act(&mut self, i: usize) {
        let pos = self.units[i].pos;
        // Healers mend first if someone nearby needs it.
        if self.units[i].stats.heal {
            let side = self.units[i].side;
            let hurt = (0..self.units.len())
                .filter(|&j| self.units[j].alive && self.units[j].side == side && manh(self.units[j].pos, pos) <= 3)
                .filter(|&j| self.units[j].hp <= self.units[j].stats.max_hp - 3)
                .max_by_key(|&j| (self.units[j].stats.max_hp - self.units[j].hp, std::cmp::Reverse(j)));
            if let Some(j) = hurt {
                let amount = 3.min(self.units[j].stats.max_hp - self.units[j].hp);
                self.units[j].hp += amount;
                self.fx.push((self.tick, Fx::Heal { from: pos, to: self.units[j].pos, amount }));
                self.units[i].cd = if self.units[i].stats.quick { 10 } else { 12 };
                return;
            }
        }
        let target = self.choose_target(i);
        if let Some(t) = target {
            if let Some(a) = self.best_attack(i, t) {
                self.attack(i, t, a);
                return;
            }
        }
        // A raider whose way is blocked, or a guard at home, fights whatever it can reach.
        let fallback = self.enemies(i).find_map(|j| self.best_attack(i, Target::Unit(j)).map(|a| (j, a)));
        let goal = match target {
            Some(t) => Some(t),
            None => {
                // A guard with nothing near: drift back toward home.
                let home = Target::Conduit(self.units[i].side);
                if self.dist(pos, home) > 2 {
                    Some(home)
                } else {
                    if let Some((j, a)) = fallback {
                        self.attack(i, Target::Unit(j), a);
                    }
                    return;
                }
            }
        };
        let goal = goal.unwrap();
        let range = if matches!(target, None) { 1 } else { self.units[i].stats.attacks.iter().map(|a| a.range).max().unwrap_or(1) };
        match self.path_step(i, goal, range) {
            Some(to) => {
                let u = &mut self.units[i];
                u.moved_from = u.pos;
                u.moved_at = self.tick;
                u.pos = to;
                u.cd = u.stats.step;
            }
            None => {
                if let Some((j, a)) = fallback {
                    self.attack(i, Target::Unit(j), a);
                }
            }
        }
    }

    /// The attack that does the most to `t` from here, if any reaches.
    fn best_attack(&self, i: usize, t: Target) -> Option<Attack> {
        let d = self.dist(self.units[i].pos, t);
        self.units[i]
            .stats
            .attacks
            .iter()
            .filter(|a| a.range >= d)
            .max_by_key(|a| (self.damage(a, t) + a.burn, -a.cooldown))
            .copied()
    }

    fn attack(&mut self, i: usize, t: Target, a: Attack) {
        let from = self.units[i].pos;
        let side = self.units[i].side;
        self.units[i].cd = a.cooldown;
        let mut hits = vec![(t, true)];
        if a.burst {
            if let Target::Unit(j) = t {
                let at = self.units[j].pos;
                for k in self.enemies(i).collect::<Vec<_>>() {
                    if k != j && manh(self.units[k].pos, at) == 1 {
                        hits.push((Target::Unit(k), false));
                    }
                }
            }
        }
        if a.sweep {
            for k in self.enemies(i).collect::<Vec<_>>() {
                if Target::Unit(k) != t && manh(self.units[k].pos, from) == 1 {
                    hits.push((Target::Unit(k), false));
                }
            }
        }
        for (target, primary) in hits {
            let dmg = self.damage(&a, target);
            let to = self.target_pos(target, from);
            self.fx.push((self.tick, Fx::Hit { from, to, dmg_type: a.dmg_type, dmg, primary }));
            self.units[i].dealt += dmg;
            match target {
                Target::Conduit(s) => {
                    self.conduit_hp[s.idx()] -= dmg;
                    self.fx.push((self.tick, Fx::ConduitHit { side: s, dmg }));
                    if self.conduit_hp[s.idx()] <= 0 {
                        self.over = Some(if s == Side::Red { Outcome::Won } else { Outcome::Lost });
                        return;
                    }
                }
                Target::Unit(j) => {
                    self.units[j].hp -= dmg;
                    if a.burn > 0 {
                        self.units[j].burn += a.burn;
                    }
                    if a.dmg_type == Dmg::Melee && self.units[i].stats.lifelink {
                        let u = &mut self.units[i];
                        u.hp = (u.hp + (dmg + 1) / 2).min(u.stats.max_hp);
                    }
                    if a.dmg_type == Dmg::Melee && self.units[j].stats.spikes {
                        self.units[i].hp -= 1;
                        self.fx.push((self.tick, Fx::Spiked { at: from }));
                    }
                    self.check_death(j);
                }
            }
        }
        let _ = side;
        self.check_death(i);
    }

    /// One orthogonal step along a shortest path to a square within `range` of the goal.
    fn path_step(&self, i: usize, goal: Target, range: i32) -> Option<Pos> {
        let start = self.units[i].pos;
        let in_range = |p: Pos| self.dist(p, goal) <= range;
        let mut prev: Vec<(Pos, Pos)> = vec![(start, start)];
        let mut frontier = vec![start];
        let mut found = None;
        'bfs: while !frontier.is_empty() {
            let mut next = Vec::new();
            for &f in &frontier {
                for (dx, dy) in DIRS {
                    let q = (f.0 + dx, f.1 + dy);
                    if self.blocked(q) || prev.iter().any(|(p, _)| *p == q) {
                        continue;
                    }
                    prev.push((q, f));
                    if in_range(q) {
                        found = Some(q);
                        break 'bfs;
                    }
                    next.push(q);
                }
            }
            frontier = next;
        }
        let mut q = found?;
        loop {
            let back = prev.iter().find(|(p, _)| *p == q).unwrap().1;
            if back == start {
                return Some(q);
            }
            q = back;
        }
    }

    pub fn run(&mut self) {
        while self.over.is_none() {
            self.step();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn build(body: Body, parts: &[Part]) -> Build {
        let mut b = Build::new(body);
        for (k, p) in parts.iter().enumerate() {
            b.parts[k] = Some(*p);
        }
        b
    }

    #[test]
    fn contests_end_and_armed_crews_beat_bare_ones() {
        let blue = [build(Body::Orc, &[Part::Club]), build(Body::Kobold, &[Part::Sling])];
        let red = [Build::new(Body::Kobold), Build::new(Body::Kobold)];
        let mut b = Battle::new(&blue, &red);
        b.run();
        assert_eq!(b.over, Some(Outcome::Won), "after {} ticks", b.tick);
        assert!(b.tick < TICK_CAP);
    }

    #[test]
    fn defense_matches_damage_type() {
        let blue = [build(Body::Orc, &[Part::Club])];
        let red = [build(Body::Orc, &[Part::Shield, Part::Shield, Part::Plate])];
        let b = Battle::new(&blue, &red);
        let club = Part::Club.attack().unwrap();
        assert_eq!(b.damage(&club, Target::Unit(1)), 1, "armor 7 vs melee 3: at least 1");
        let wand = Part::SparkWand.attack().unwrap();
        assert_eq!(b.damage(&wand, Target::Unit(1)), 2, "magic ignores armor");
    }

    #[test]
    fn many_random_contests_finish() {
        let mut rng = crate::rng::Rng::new(5);
        for _ in 0..60 {
            let mut sides: Vec<Vec<Build>> = Vec::new();
            for _ in 0..2 {
                let mut crew = Vec::new();
                for _ in 0..rng.range(1, 4) {
                    let body = *rng.pick(&ALL_BODIES);
                    let mut b = Build::new(body);
                    for _ in 0..6 {
                        let p = *rng.pick(&ALL_PARTS);
                        if let Some(i) = (0..b.parts.len()).find(|&i| b.parts[i].is_none() && b.refusal(i, p).is_none()) {
                            b.parts[i] = Some(p);
                        }
                    }
                    crew.push(b);
                }
                sides.push(crew);
            }
            let mut b = Battle::new(&sides[0], &sides[1]);
            b.run();
            assert!(b.over.is_some() && b.tick <= TICK_CAP);
        }
    }
}
