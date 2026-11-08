//! A night at the hatchery: Grof's creatures come in from the right for the eggs on the left;
//! your familiars fight on their own. Deterministic and tick-based, like the player's curses.

use super::data::*;
use crate::rng::Rng;

pub const W: i32 = 10;
pub const H: i32 = 6;
pub const NEST: [Pos; 2] = [(0, 2), (0, 3)];
pub const TICK_CAP: u32 = 1400;
pub type Pos = (i32, i32);

const ORTH: [Pos; 4] = [(1, 0), (-1, 0), (0, 1), (0, -1)];

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Side {
    Ours,
    Theirs,
}

#[derive(Clone, Debug)]
pub struct Unit {
    pub id: u32,
    pub side: Side,
    /// Index into the roster, for familiars.
    pub fam: Option<usize>,
    pub foe: Option<Foe>,
    pub body: Option<Body>,
    pub name: &'static str,
    pub genes: Vec<Gene>,
    pub hp: i32,
    pub max: i32,
    pub step: i32,
    pub bite: i32,
    pub reach: i32,
    pub cooldown: i32,
    pub flies: bool,
    pub cd: i32,
    pub pos: Pos,
    pub alive: bool,
    pub poison: i32,
    pub slime_ready: bool,
    pub split_ready: bool,
    pub buried: bool,
    pub first_bite: bool,
    pub moved_from: Pos,
    pub moved_at: u32,
    pub kills: i32,
}

impl Unit {
    pub fn has(&self, g: Gene) -> bool {
        self.genes.contains(&g)
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum Fx {
    Bite { from: Pos, to: Pos, dmg: i32 },
    Word { at: Pos, text: &'static str },
    Fire { squares: Vec<Pos> },
    Died { at: Pos, side: Side },
    Egg { at: Pos },
    Heal { at: Pos },
    Chorus { at: Pos },
}

#[derive(Clone, Debug)]
pub struct Battle {
    pub units: Vec<Unit>,
    pub tick: u32,
    pub eggs: i32,
    pub eggs_eaten: i32,
    pub schedule: Vec<(u32, Foe)>,
    pub water: Vec<Pos>,
    pub fx: Vec<(u32, Fx)>,
    pub over: bool,
    pub next_id: u32,
    pub rng: Rng,
    pub defeated: Vec<Foe>,
}

pub fn inb((x, y): Pos) -> bool {
    (0..W).contains(&x) && (0..H).contains(&y)
}

pub fn manh(a: Pos, b: Pos) -> i32 {
    (a.0 - b.0).abs() + (a.1 - b.1).abs()
}

fn nest_dist(p: Pos) -> i32 {
    NEST.iter().map(|&n| manh(n, p)).min().unwrap()
}

/// Where familiars may stand when the night begins.
pub fn deploy_squares() -> Vec<Pos> {
    (1..=4).flat_map(|x| (0..H).map(move |y| (x, y))).collect()
}

impl Battle {
    pub fn new(roster: &[Familiar], posts: &[(usize, Pos)], night: u32, eggs: i32, seed: u64) -> Self {
        let mut rng = Rng::new(seed);
        let mut b = Self {
            units: Vec::new(),
            tick: 0,
            eggs,
            eggs_eaten: 0,
            schedule: Vec::new(),
            water: Vec::new(),
            fx: Vec::new(),
            over: false,
            next_id: 1,
            rng: Rng::new(seed ^ 0xABCD),
            defeated: Vec::new(),
        };
        // A stream across the middle of the pond.
        let x0 = rng.range(4, 6);
        for y in 0..H {
            let x = x0 + if y % 3 == 1 { 1 } else { 0 };
            if rng.below(100) < 80 {
                b.water.push((x, y));
            }
        }
        for &(i, p) in posts {
            let f = &roster[i];
            b.add_familiar(i, f, p);
        }
        let kinds = night_foes(night);
        let mut t = 10;
        if night == NIGHTS {
            b.schedule.push((30, Foe::Grof));
        }
        for _ in 0..night_size(night) {
            b.schedule.push((t, *rng.pick(&kinds)));
            t += rng.range(10, 18) as u32;
        }
        b
    }

    fn add_familiar(&mut self, i: usize, f: &Familiar, p: Pos) {
        let info = f.body.info();
        let id = self.next_id;
        self.next_id += 1;
        let reach = if f.has(Gene::LongTongue) { 3 } else { 1 };
        self.units.push(Unit {
            id,
            side: Side::Ours,
            fam: Some(i),
            foe: None,
            body: Some(f.body),
            name: f.name,
            genes: f.genes.clone(),
            hp: f.max_hp(),
            max: f.max_hp(),
            step: info.step,
            bite: info.bite,
            reach,
            cooldown: 7,
            flies: false,
            cd: 2 + i as i32,
            pos: p,
            alive: true,
            poison: 0,
            slime_ready: f.has(Gene::Slime),
            split_ready: f.has(Gene::SplitTail),
            buried: f.has(Gene::DiggingClaws),
            first_bite: true,
            moved_from: p,
            moved_at: 0,
            kills: 0,
        });
    }

    fn add_tadpole(&mut self, p: Pos) {
        let id = self.next_id;
        self.next_id += 1;
        self.units.push(Unit {
            id,
            side: Side::Ours,
            fam: None,
            foe: None,
            body: None,
            name: "TADPOLE",
            genes: Vec::new(),
            hp: 2,
            max: 2,
            step: 2,
            bite: 1,
            reach: 1,
            cooldown: 6,
            flies: false,
            cd: 3,
            pos: p,
            alive: true,
            poison: 0,
            slime_ready: false,
            split_ready: false,
            buried: false,
            first_bite: false,
            moved_from: p,
            moved_at: self.tick,
            kills: 0,
        });
    }

    fn add_foe(&mut self, foe: Foe, p: Pos) {
        let i = foe.info();
        let id = self.next_id;
        self.next_id += 1;
        self.units.push(Unit {
            id,
            side: Side::Theirs,
            fam: None,
            foe: Some(foe),
            body: None,
            name: i.name,
            genes: Vec::new(),
            hp: i.hp,
            max: i.hp,
            step: i.step,
            bite: i.bite,
            reach: i.reach,
            cooldown: i.cooldown,
            flies: i.flies,
            cd: 4,
            pos: p,
            alive: true,
            poison: 0,
            slime_ready: false,
            split_ready: false,
            buried: false,
            first_bite: false,
            moved_from: (p.0 + 1, p.1),
            moved_at: self.tick,
            kills: 0,
        });
    }

    pub fn at(&self, p: Pos) -> Option<usize> {
        self.units.iter().position(|u| u.alive && u.pos == p)
    }

    pub fn foes_left(&self) -> usize {
        self.units.iter().filter(|u| u.alive && u.side == Side::Theirs).count() + self.schedule.len()
    }

    fn word(&mut self, at: Pos, text: &'static str) {
        self.fx.push((self.tick, Fx::Word { at, text }));
    }

    pub fn step(&mut self) {
        if self.over {
            return;
        }
        self.tick += 1;
        let t = self.tick;
        self.fx.retain(|(ft, _)| t.saturating_sub(*ft) < 10);
        // Arrivals at the right edge.
        let mut k = 0;
        while k < self.schedule.len() {
            if self.schedule[k].0 <= t {
                let foe = self.schedule[k].1;
                let rows: Vec<i32> = (0..H).filter(|&y| self.at((W - 1, y)).is_none()).collect();
                if !rows.is_empty() {
                    let y = *self.rng.pick(&rows);
                    self.add_foe(foe, (W - 1, y));
                    self.schedule.remove(k);
                    continue;
                }
            }
            k += 1;
        }
        let n = self.units.len();
        for i in 0..n {
            if !self.units[i].alive {
                continue;
            }
            self.passives(i);
            if !self.units[i].alive {
                continue;
            }
            if self.units[i].cd > 0 {
                self.units[i].cd -= 1;
                continue;
            }
            match self.units[i].side {
                Side::Ours => self.familiar_act(i),
                Side::Theirs => self.foe_act(i),
            }
            if self.eggs <= 0 {
                self.over = true;
                return;
            }
        }
        if (self.foes_left() == 0) || t >= TICK_CAP {
            self.over = true;
        }
    }

    /// Timed effects: poison, regrowth, chorus, fire, summons.
    fn passives(&mut self, i: usize) {
        let t = self.tick;
        let at = self.units[i].pos;
        if self.units[i].poison > 0 && t % 4 == 0 {
            self.units[i].poison -= 1;
            self.damage(i, 1, None);
            if !self.units[i].alive {
                return;
            }
        }
        let regen = (self.units[i].has(Gene::Regrowth) && t % 4 == 0)
            || (self.units[i].body == Some(Body::Axolotl) && t % 6 == 0 && self.water.contains(&at))
            || (self.units[i].foe == Some(Foe::Leech) && t % 4 == 0);
        if regen && self.units[i].hp < self.units[i].max {
            self.units[i].hp += 1;
            self.fx.push((t, Fx::Heal { at }));
        }
        if self.units[i].has(Gene::ChorusSac) && t % 10 == 0 {
            self.fx.push((t, Fx::Chorus { at }));
            for j in 0..self.units.len() {
                if j != i && self.units[j].alive && self.units[j].side == Side::Ours && manh(self.units[j].pos, at) <= 2 {
                    self.units[j].cd = 0;
                }
            }
        }
        if self.units[i].has(Gene::FireGland) && t % 12 == 0 && !self.units[i].buried {
            for d in ORTH {
                let line: Vec<Pos> = (1..=3).map(|k| (at.0 + d.0 * k, at.1 + d.1 * k)).filter(|p| inb(*p)).collect();
                let hit: Vec<usize> = line.iter().filter_map(|p| self.at(*p)).filter(|&j| self.units[j].side == Side::Theirs).collect();
                if !hit.is_empty() {
                    self.fx.push((t, Fx::Fire { squares: line.clone() }));
                    for j in hit {
                        self.damage(j, 2, Some(i));
                    }
                    break;
                }
            }
        }
        if self.units[i].foe == Some(Foe::Grof) && t % 20 == 0 {
            let spots: Vec<Pos> = ORTH.iter().map(|d| (at.0 + d.0, at.1 + d.1)).filter(|p| inb(*p) && self.at(*p).is_none()).collect();
            for p in spots.into_iter().take(2) {
                self.add_foe(Foe::Toadling, p);
            }
            self.word(at, "TOADLINGS!");
        }
    }

    /// Deals damage to unit `i`, from `by` (a unit index) if it was a bite.
    fn damage(&mut self, i: usize, mut dmg: i32, by: Option<usize>) {
        let u = &mut self.units[i];
        if !u.alive {
            return;
        }
        if u.slime_ready && by.is_some() {
            u.slime_ready = false;
            let at = u.pos;
            self.word(at, "SLIME!");
            return;
        }
        if u.has(Gene::BarkSkin) || u.foe == Some(Foe::Beetle) {
            dmg = (dmg - 1).max(1);
        }
        let u = &mut self.units[i];
        u.hp -= dmg;
        if u.hp <= 0 && u.split_ready {
            u.split_ready = false;
            u.hp = 1;
            let at = u.pos;
            self.word(at, "SPLIT TAIL!");
            return;
        }
        if self.units[i].hp <= 0 {
            self.kill(i, by);
        }
    }

    fn kill(&mut self, i: usize, by: Option<usize>) {
        let u = &mut self.units[i];
        u.alive = false;
        let (at, side) = (u.pos, u.side);
        self.fx.push((self.tick, Fx::Died { at, side }));
        if let Some(f) = self.units[i].foe {
            self.defeated.push(f);
            if let Some(k) = by {
                self.units[k].kills += 1;
            }
        }
        if self.units[i].has(Gene::EggSac) {
            self.word(at, "EGG SAC!");
            let spots: Vec<Pos> = std::iter::once(at).chain(ORTH.iter().map(|d| (at.0 + d.0, at.1 + d.1))).filter(|p| inb(*p) && self.at(*p).is_none()).collect();
            for p in spots.into_iter().take(2) {
                self.add_tadpole(p);
            }
        }
    }

    fn glow_bonus(&self, i: usize) -> i32 {
        let at = self.units[i].pos;
        self.units.iter().any(|u| u.alive && u.side == Side::Ours && u.has(Gene::GlowSpots) && manh(u.pos, at) <= 2) as i32
    }

    fn bite(&mut self, i: usize, j: usize) {
        let from = self.units[i].pos;
        let to = self.units[j].pos;
        let mut dmg = self.units[i].bite;
        if self.units[i].side == Side::Ours {
            dmg += self.glow_bonus(i);
            if self.units[i].has(Gene::WideMouth) {
                dmg += 1;
            }
            if self.units[i].has(Gene::DiggingClaws) && self.units[i].first_bite {
                dmg += 2;
                self.word(from, "DIG!");
            }
            self.units[i].first_bite = false;
            self.units[i].buried = false;
        }
        self.units[i].cd = self.units[i].cooldown;
        self.fx.push((self.tick, Fx::Bite { from, to, dmg }));
        let spiky = self.units[j].has(Gene::Warts) || self.units[j].foe == Some(Foe::Hedgehog);
        self.damage(j, dmg, Some(i));
        // Venom.
        if self.units[j].alive && (self.units[i].has(Gene::VenomFangs) || self.units[i].foe.is_some_and(|f| f.venom())) {
            self.units[j].poison += 3;
        }
        // Swallow whole.
        if self.units[j].alive && self.units[i].has(Gene::WideMouth) && self.units[j].hp <= 3 && !matches!(self.units[j].foe, Some(Foe::Grof) | Some(Foe::Heron)) {
            self.word(to, "SWALLOWED!");
            self.kill(j, Some(i));
        }
        // The long tongue reels its prey in.
        if self.units[j].alive && self.units[i].has(Gene::LongTongue) && manh(from, to) > 1 && self.units[j].foe != Some(Foe::Grof) {
            let d = ((from.0 - to.0).signum(), if from.0 == to.0 { (from.1 - to.1).signum() } else { 0 });
            let p = (to.0 + d.0, to.1 + d.1);
            if inb(p) && self.at(p).is_none() && !NEST.contains(&p) {
                self.units[j].pos = p;
                self.units[j].moved_from = to;
                self.units[j].moved_at = self.tick;
            }
        }
        // Club tail.
        if self.units[i].has(Gene::ClubTail) {
            if let Some(k) = (0..self.units.len()).find(|&k| k != j && self.units[k].alive && self.units[k].side == Side::Theirs && manh(self.units[k].pos, from) == 1) {
                self.word(self.units[k].pos, "CLUB!");
                self.damage(k, 1, Some(i));
            }
        }
        if spiky && self.units[i].alive {
            self.damage(i, 1, None);
        }
    }

    fn free(&self, p: Pos, me: usize) -> bool {
        inb(p) && !NEST.contains(&p) && self.at(p).map_or(true, |k| k == me)
    }

    /// One orthogonal step along a shortest path to any square within `reach` of `goal`.
    fn path_step(&self, i: usize, goal: Pos, reach: i32) -> Option<Pos> {
        let start = self.units[i].pos;
        let sticky = self.units[i].has(Gene::StickyPads);
        let side = self.units[i].side;
        let passable = |p: Pos| {
            inb(p) && !NEST.contains(&p) && self.at(p).map_or(true, |k| k == i || (sticky && self.units[k].side == side))
        };
        let mut prev: Vec<(Pos, Pos)> = vec![(start, start)];
        let mut frontier = vec![start];
        let mut found = None;
        'bfs: while !frontier.is_empty() {
            let mut next = Vec::new();
            for &f in &frontier {
                let mut dirs = ORTH;
                dirs.sort_by_key(|d| manh((f.0 + d.0, f.1 + d.1), goal));
                for d in dirs {
                    let q = (f.0 + d.0, f.1 + d.1);
                    if !passable(q) || prev.iter().any(|(p, _)| *p == q) {
                        continue;
                    }
                    prev.push((q, f));
                    if manh(q, goal) <= reach && self.at(q).is_none() {
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
                // A sticky climber can't stop on top of a friend: only take the step if it's free.
                return self.free(q, i).then_some(q);
            }
            q = back;
        }
    }

    fn move_to(&mut self, i: usize, to: Pos) {
        let u = &mut self.units[i];
        u.moved_from = u.pos;
        u.moved_at = self.tick;
        u.pos = to;
        let in_water = self.water.contains(&to);
        let u = &mut self.units[i];
        u.cd = if in_water && u.has(Gene::WebbedFeet) { (u.step + 1) / 2 } else { u.step };
    }

    fn familiar_act(&mut self, i: usize) {
        let me = self.units[i].pos;
        let target = (0..self.units.len())
            .filter(|&j| self.units[j].alive && self.units[j].side == Side::Theirs)
            .min_by_key(|&j| (manh(self.units[j].pos, me), self.units[j].hp, j));
        let Some(j) = target else {
            // Nothing to fight: drift back toward the nest.
            if nest_dist(me) > 2 {
                if let Some(to) = self.path_step(i, NEST[0], 2) {
                    self.move_to(i, to);
                }
            }
            return;
        };
        let tp = self.units[j].pos;
        if manh(me, tp) <= self.units[i].reach {
            self.bite(i, j);
            return;
        }
        if self.units[i].has(Gene::LeapingLegs) {
            // Leap straight along the longer axis, up to 3 squares, stopping short of anything.
            let (dx, dy) = (tp.0 - me.0, tp.1 - me.1);
            let d = if dx.abs() >= dy.abs() { (dx.signum(), 0) } else { (0, dy.signum()) };
            let mut to = me;
            for k in 1..=3 {
                let p = (me.0 + d.0 * k, me.1 + d.1 * k);
                if !self.free(p, i) || manh(to, tp) <= self.units[i].reach {
                    break;
                }
                to = p;
            }
            if to != me && manh(to, tp) < manh(me, tp) {
                self.move_to(i, to);
                if manh(me, to) > 1 {
                    self.word(to, "LEAP!");
                }
                return;
            }
        }
        if let Some(to) = self.path_step(i, tp, self.units[i].reach) {
            self.move_to(i, to);
        }
    }

    fn foe_act(&mut self, i: usize) {
        let me = self.units[i].pos;
        // At the nest: carry off an egg.
        if nest_dist(me) <= 1 {
            self.eggs -= 1;
            self.eggs_eaten += 1;
            self.fx.push((self.tick, Fx::Egg { at: me }));
            self.units[i].alive = false;
            return;
        }
        if !self.units[i].flies {
            // Bite a familiar in reach: glowing ones first, bright ones last.
            let mut prey: Vec<usize> = (0..self.units.len())
                .filter(|&j| self.units[j].alive && self.units[j].side == Side::Ours && !self.units[j].buried && manh(self.units[j].pos, me) <= self.units[i].reach)
                .collect();
            if prey.len() > 1 {
                prey.retain(|&j| !self.units[j].has(Gene::BrightColors));
            }
            if prey.is_empty() {
                // A glowing familiar within 2 draws foes in.
                let glow = (0..self.units.len()).find(|&j| self.units[j].alive && self.units[j].side == Side::Ours && self.units[j].has(Gene::GlowSpots) && manh(self.units[j].pos, me) <= 2);
                if let Some(j) = glow {
                    if let Some(to) = self.path_step(i, self.units[j].pos, self.units[i].reach) {
                        self.move_to(i, to);
                        return;
                    }
                }
            }
            prey.sort_by_key(|&j| (!self.units[j].has(Gene::GlowSpots), self.units[j].hp, j));
            if let Some(&j) = prey.first() {
                if !(self.units[j].has(Gene::BrightColors) && prey.len() > 1) {
                    self.bite(i, j);
                    return;
                }
            }
        }
        // Head for the nest. Fliers go straight over everything.
        let goal = *NEST.iter().min_by_key(|&&n| manh(n, me)).unwrap();
        if self.units[i].flies {
            let (dx, dy) = (goal.0 - me.0, goal.1 - me.1);
            let d = if dx.abs() >= dy.abs() { (dx.signum(), 0) } else { (0, dy.signum()) };
            let to = (me.0 + d.0, me.1 + d.1);
            if inb(to) && !NEST.contains(&to) {
                let u = &mut self.units[i];
                u.moved_from = u.pos;
                u.moved_at = self.tick;
                u.pos = to;
                u.cd = u.step;
            }
            return;
        }
        match self.path_step(i, goal, 1) {
            Some(to) => self.move_to(i, to),
            None => {
                // Blocked in: bite whatever familiar is closest.
                if let Some(j) = (0..self.units.len()).filter(|&j| self.units[j].alive && self.units[j].side == Side::Ours && !self.units[j].buried).min_by_key(|&j| manh(self.units[j].pos, me)) {
                    if manh(self.units[j].pos, me) <= self.units[i].reach {
                        self.bite(i, j);
                        return;
                    }
                    if let Some(to) = self.path_step(i, self.units[j].pos, self.units[i].reach) {
                        self.move_to(i, to);
                    }
                }
            }
        }
    }

    pub fn run(&mut self) {
        while !self.over {
            self.step();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn nights_end_and_defenders_matter() {
        let roster = vec![Familiar::new("A", Body::Toad, Gene::Warts), Familiar::new("B", Body::Frog, Gene::LongTongue)];
        let posts = vec![(0, (2, 2)), (1, (2, 3))];
        let mut b = Battle::new(&roster, &posts, 1, 6, 9);
        b.run();
        let mut empty = Battle::new(&[], &[], 1, 6, 9);
        empty.run();
        assert!(b.tick < TICK_CAP && empty.tick < TICK_CAP);
        assert!(empty.eggs_eaten > b.eggs_eaten, "defended {} vs undefended {}", b.eggs_eaten, empty.eggs_eaten);
    }

    #[test]
    fn every_night_finishes() {
        for night in 1..=NIGHTS {
            for seed in 0..6 {
                let roster: Vec<Familiar> = ALL_BODIES
                    .iter()
                    .take(3)
                    .enumerate()
                    .map(|(k, &b)| {
                        let g = *ALL_GENES.iter().find(|g| g.loc() == b.info().locs[0]).unwrap();
                        let mut f = Familiar::new(NAMES[k], b, g);
                        f.level = 2;
                        f
                    })
                    .collect();
                let posts: Vec<(usize, Pos)> = (0..roster.len()).map(|k| (k, (3, 1 + k as i32))).collect();
                let mut b = Battle::new(&roster, &posts, night, 20, seed);
                b.run();
                assert!(b.over);
            }
        }
    }

    #[test]
    fn egg_sac_hatches_tadpoles() {
        let roster = vec![Familiar { name: "E", body: Body::Newt, genes: vec![Gene::EggSac], xp: 0, level: 1 }];
        let mut b = Battle::new(&roster, &[(0, (3, 2))], 1, 6, 1);
        b.units[0].hp = 1;
        let i = 0;
        b.kill(i, None);
        assert_eq!(b.units.iter().filter(|u| u.name == "TADPOLE").count(), 2);
    }
}
