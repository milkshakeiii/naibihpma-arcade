//! A Turncoat battle: deterministic, one person's action at a time, so the screen can show each.

use super::data::*;

pub const W: i32 = 7;
pub const H: i32 = 6;
pub const ROUND_CAP: u32 = 30;
pub type Pos = (i32, i32);

#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash)]
pub enum Side {
    Blue,
    Red,
}

impl Side {
    pub fn other(self) -> Side {
        match self {
            Side::Blue => Side::Red,
            Side::Red => Side::Blue,
        }
    }
    pub fn ahead(self) -> i32 {
        match self {
            Side::Blue => -1,
            Side::Red => 1,
        }
    }
}

#[derive(Clone, Debug)]
pub struct Person {
    pub uid: u32,
    pub kind: Kind,
    pub side: Side,
    pub origin: Side,
    pub pos: Pos,
    pub resolve: i32,
    pub max: i32,
    pub veteran: bool,
    pub career: i32,
    pub turned_here: i32,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Outcome {
    Won,
    Lost,
    Draw,
}

#[derive(Clone, Debug, PartialEq)]
pub enum Act {
    Round(u32),
    Speak { actor: usize, hits: Vec<(usize, i32)> },
    Turned { who: usize, by: usize, to: Side },
    Held { who: usize },
    Walk { who: usize, from: Pos, to: Pos },
    Veteran { who: usize },
    Mend { who: usize },
}

#[derive(Clone, Debug)]
pub struct Battle {
    pub people: Vec<Person>,
    pub ideas: Vec<Idea>,
    pub round: u32,
    order: Vec<usize>,
    cursor: usize,
    pub over: Option<Outcome>,
}

pub fn inb((x, y): Pos) -> bool {
    (0..W).contains(&x) && (0..H).contains(&y)
}

fn cheb(a: Pos, b: Pos) -> i32 {
    (a.0 - b.0).abs().max((a.1 - b.1).abs())
}

fn manh(a: Pos, b: Pos) -> i32 {
    (a.0 - b.0).abs() + (a.1 - b.1).abs()
}

/// Mirrors base offsets every way.
pub fn mirrored(bases: &[(i32, i32)]) -> Vec<Pos> {
    let mut v = Vec::new();
    for &(a, b) in bases {
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

pub fn max_resolve(kind: Kind, veteran: bool, side: Side, ideas: &[Idea]) -> i32 {
    kind.info().resolve + if veteran { 2 } else { 0 } + if side == Side::Blue && ideas.contains(&Idea::Bronze) { 2 } else { 0 }
}

impl Battle {
    pub fn new(people: Vec<Person>, ideas: Vec<Idea>) -> Self {
        let mut b = Self { people, ideas, round: 0, order: Vec::new(), cursor: 0, over: None };
        b.check_over();
        b
    }

    pub fn at(&self, p: Pos) -> Option<usize> {
        self.people.iter().position(|q| q.pos == p)
    }

    fn blue_has(&self, side: Side, idea: Idea) -> bool {
        side == Side::Blue && self.ideas.contains(&idea)
    }

    pub fn pace(&self, i: usize) -> i32 {
        let p = &self.people[i];
        p.kind.info().pace + if self.blue_has(p.side, Idea::Roads) { 1 } else { 0 }
    }

    fn swift(&self, i: usize) -> bool {
        let p = &self.people[i];
        p.kind.has(Trait::Swift) || self.blue_has(p.side, Idea::Roads)
    }

    fn neighbours(&self, i: usize) -> impl Iterator<Item = &Person> + '_ {
        let (pos, side) = (self.people[i].pos, self.people[i].side);
        self.people.iter().filter(move |q| q.side == side && q.pos != pos && cheb(q.pos, pos) == 1)
    }

    pub fn voice(&self, i: usize) -> i32 {
        let p = &self.people[i];
        let mut v = p.kind.info().voice + if p.veteran { 1 } else { 0 };
        if self.neighbours(i).any(|q| q.kind.has(Trait::Rally)) {
            v += 1;
        }
        if self.blue_has(p.side, Idea::Rhetoric) && p.resolve >= p.max {
            v += 1;
        }
        if self.blue_has(p.side, Idea::OralTradition) && matches!(p.kind, Kind::Herald | Kind::Broadcaster) {
            v += 1;
        }
        v
    }

    fn damage_to(&self, actor: usize, target: usize) -> i32 {
        let t = &self.people[target];
        let mut d = self.voice(actor);
        if t.kind.has(Trait::Sturdy) {
            d -= 1;
        }
        if self.neighbours(target).any(|q| q.kind.has(Trait::Builder)) {
            d -= 1;
        }
        if self.blue_has(t.side, Idea::Walls) && t.pos.1 == H - 1 {
            d -= 1;
        }
        d.max(1)
    }

    /// Squares person `i` would reach standing at `from` (others where they are).
    pub fn reach_from(&self, i: usize, from: Pos) -> Vec<Pos> {
        let p = &self.people[i];
        let occupied = |q: Pos| q != p.pos && self.at(q).is_some();
        let mut v = Vec::new();
        match p.kind.info().reach {
            Reach::Steps(bases) => {
                for (dx, dy) in mirrored(bases) {
                    let q = (from.0 + dx, from.1 + dy);
                    if inb(q) {
                        v.push(q);
                    }
                }
            }
            Reach::Ahead(n) => {
                for k in 1..=n {
                    let q = (from.0, from.1 + p.side.ahead() * k);
                    if !inb(q) {
                        break;
                    }
                    v.push(q);
                    if occupied(q) {
                        break;
                    }
                }
            }
            Reach::Lines(dirs, n) => {
                for &(dx, dy) in dirs {
                    for k in 1..=n {
                        let q = (from.0 + dx * k, from.1 + dy * k);
                        if !inb(q) {
                            break;
                        }
                        v.push(q);
                        if occupied(q) {
                            break;
                        }
                    }
                }
            }
            Reach::Radius(r) => {
                let r = if self.blue_has(p.side, Idea::OralTradition) && p.kind == Kind::Elder { r + 1 } else { r };
                for dy in -r..=r {
                    for dx in -r..=r {
                        let q = (from.0 + dx, from.1 + dy);
                        if (dx, dy) != (0, 0) && inb(q) {
                            v.push(q);
                        }
                    }
                }
            }
            Reach::Column => {
                for k in 1..H {
                    let q = (from.0, from.1 + p.side.ahead() * k);
                    if inb(q) {
                        v.push(q);
                    }
                }
            }
        }
        v
    }

    fn rivals_in_reach(&self, i: usize, from: Pos) -> Vec<usize> {
        let side = self.people[i].side;
        self.reach_from(i, from).into_iter().filter_map(|q| self.at(q)).filter(|&j| j != i && self.people[j].side != side).collect()
    }

    /// Who person `i` would persuade right now.
    pub fn targets(&self, i: usize) -> Vec<usize> {
        let p = &self.people[i];
        let rivals = self.rivals_in_reach(i, p.pos);
        if rivals.is_empty() {
            return rivals;
        }
        if p.kind.info().reach == Reach::Column {
            return rivals;
        }
        let best = *rivals
            .iter()
            .min_by_key(|&&j| {
                let q = &self.people[j];
                let row = if p.side == Side::Blue { -q.pos.1 } else { q.pos.1 };
                (q.resolve, manh(q.pos, p.pos), -row, q.pos.0)
            })
            .unwrap();
        vec![best]
    }

    fn check_over(&mut self) {
        let blue = self.people.iter().filter(|p| p.side == Side::Blue).count();
        let red = self.people.len() - blue;
        if red == 0 {
            self.over = Some(Outcome::Won);
        } else if blue == 0 {
            self.over = Some(Outcome::Lost);
        }
    }

    fn start_round(&mut self, acts: &mut Vec<Act>) {
        self.round += 1;
        if self.round > ROUND_CAP {
            let blue = self.people.iter().filter(|p| p.side == Side::Blue).count();
            let red = self.people.len() - blue;
            self.over = Some(if blue > red {
                Outcome::Won
            } else if red > blue {
                Outcome::Lost
            } else {
                Outcome::Draw
            });
            return;
        }
        acts.push(Act::Round(self.round));
        if self.ideas.contains(&Idea::Medicine) {
            if let Some(i) = (0..self.people.len())
                .filter(|&i| self.people[i].side == Side::Blue && self.people[i].resolve < self.people[i].max)
                .min_by_key(|&i| (self.people[i].resolve, self.people[i].uid))
            {
                self.people[i].resolve += 1;
                acts.push(Act::Mend { who: i });
            }
        }
        let mut order: Vec<usize> = (0..self.people.len()).collect();
        order.sort_by_key(|&i| {
            let p = &self.people[i];
            (-self.pace(i), (2 * p.pos.1 - (H - 1)).abs(), p.pos.0, p.uid)
        });
        self.order = order;
        self.cursor = 0;
    }

    /// Advances to the next person who does something, and returns what happened.
    pub fn step(&mut self) -> Vec<Act> {
        let mut acts = Vec::new();
        let mut guard = 0;
        while self.over.is_none() && guard < 400 {
            guard += 1;
            if self.cursor >= self.order.len() {
                self.start_round(&mut acts);
                continue;
            }
            let i = self.order[self.cursor];
            self.cursor += 1;
            self.act(i, &mut acts);
            if acts.iter().any(|a| !matches!(a, Act::Round(_) | Act::Mend { .. })) {
                break;
            }
        }
        acts
    }

    fn act(&mut self, i: usize, acts: &mut Vec<Act>) {
        let targets = self.targets(i);
        if !targets.is_empty() {
            let mut hits = Vec::new();
            for &t in &targets {
                hits.push((t, self.damage_to(i, t)));
            }
            acts.push(Act::Speak { actor: i, hits: hits.clone() });
            let side = self.people[i].side;
            for (t, d) in hits {
                if self.people[t].side == side {
                    continue; // turned earlier in this same speech
                }
                self.people[t].resolve -= d;
                if self.people[t].resolve > 0 {
                    continue;
                }
                let held = self.neighbours(t).any(|q| q.kind.has(Trait::Faithful));
                if held {
                    self.people[t].resolve = 1;
                    acts.push(Act::Held { who: t });
                    continue;
                }
                let full = self.people[i].kind.has(Trait::Kindred) || self.people[i].kind.has(Trait::Awe) || self.blue_has(side, Idea::Writing);
                let max = max_resolve(self.people[t].kind, self.people[t].veteran, side, &self.ideas);
                let tp = &mut self.people[t];
                tp.side = side;
                tp.max = max;
                tp.resolve = if full { max } else { (max + 1) / 2 };
                acts.push(Act::Turned { who: t, by: i, to: side });
                let a = &mut self.people[i];
                a.turned_here += 1;
                a.career += 1;
                if a.kind.has(Trait::Trader) {
                    a.resolve = a.max;
                }
                if a.career >= 3 && !a.veteran {
                    a.veteran = true;
                    a.max += 2;
                    a.resolve += 2;
                    acts.push(Act::Veteran { who: i });
                }
            }
            self.check_over();
            return;
        }
        if self.people[i].kind.has(Trait::Rooted) {
            return;
        }
        let steps = if self.swift(i) { 2 } else { 1 };
        for _ in 0..steps {
            if !self.rivals_in_reach(i, self.people[i].pos).is_empty() {
                break;
            }
            let Some(to) = self.next_step(i) else { break };
            let from = self.people[i].pos;
            self.people[i].pos = to;
            acts.push(Act::Walk { who: i, from, to });
        }
    }

    /// One king step along a shortest path to a square that has a rival in reach; otherwise a
    /// step that closes on the nearest rival.
    fn next_step(&self, i: usize) -> Option<Pos> {
        let start = self.people[i].pos;
        let free = |q: Pos| inb(q) && self.at(q).is_none();
        let mut prev: Vec<(Pos, Pos)> = vec![(start, start)];
        let mut frontier = vec![start];
        let mut found = None;
        'bfs: for _ in 0..(W * H) {
            let mut next = Vec::new();
            for &f in &frontier {
                let mut dirs = ALL8.to_vec();
                // Prefer heading toward the other side, then straight steps.
                let ahead = self.people[i].side.ahead();
                dirs.sort_by_key(|&(dx, dy)| (-(dy * ahead), (dx != 0 && dy != 0) as i32, dx));
                for (dx, dy) in dirs {
                    let q = (f.0 + dx, f.1 + dy);
                    if !free(q) || prev.iter().any(|(p, _)| *p == q) {
                        continue;
                    }
                    prev.push((q, f));
                    if !self.rivals_in_reach(i, q).is_empty() {
                        found = Some(q);
                        break 'bfs;
                    }
                    next.push(q);
                }
            }
            if next.is_empty() {
                break;
            }
            frontier = next;
        }
        if let Some(mut q) = found {
            loop {
                let back = prev.iter().find(|(p, _)| *p == q).unwrap().1;
                if back == start {
                    return Some(q);
                }
                q = back;
            }
        }
        let side = self.people[i].side;
        let nearest = self.people.iter().filter(|p| p.side != side).map(|p| p.pos).min_by_key(|&p| cheb(p, start))?;
        let d0 = cheb(start, nearest);
        ALL8.iter().map(|d| (start.0 + d.0, start.1 + d.1)).filter(|&q| free(q) && cheb(q, nearest) < d0).min_by_key(|&q| (cheb(q, nearest), manh(q, nearest)))
    }

    /// Plays the battle to its end (for tests and the SKIP button).
    pub fn run(&mut self) -> Vec<Act> {
        let mut all = Vec::new();
        while self.over.is_none() {
            all.extend(self.step());
        }
        all
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;

    pub fn person(uid: u32, kind: Kind, side: Side, pos: Pos) -> Person {
        let max = max_resolve(kind, false, side, &[]);
        Person { uid, kind, side, origin: side, pos, resolve: max, max, veteran: false, career: 0, turned_here: 0 }
    }

    #[test]
    fn the_last_word_turns_someone() {
        let mut b = Battle::new(vec![person(1, Kind::Hunter, Side::Blue, (3, 4)), person(2, Kind::Gatherer, Side::Red, (3, 2))], vec![]);
        b.run();
        assert_eq!(b.over, Some(Outcome::Won));
        assert!(b.people.iter().all(|p| p.side == Side::Blue));
        assert_eq!(b.people[1].resolve, 2, "joins with half resolve");
    }

    #[test]
    fn battles_always_end() {
        let kinds = [Kind::Gatherer, Kind::Hunter, Kind::Elder, Kind::Shaman, Kind::Smith, Kind::Rider, Kind::Herald, Kind::Priest, Kind::Broadcaster];
        for seed in 0..40u32 {
            let mut people = Vec::new();
            for k in 0..6u32 {
                let kind = kinds[((seed * 7 + k * 3) % kinds.len() as u32) as usize];
                people.push(person(k, kind, Side::Blue, ((k as i32 * 2 + seed as i32) % W, 3 + (k as i32 % 3))));
                let kind = kinds[((seed * 5 + k * 2 + 1) % kinds.len() as u32) as usize];
                people.push(person(100 + k, kind, Side::Red, ((k as i32 * 3 + seed as i32) % W, k as i32 % 3)));
            }
            people.dedup_by_key(|p| p.pos);
            let mut seen = std::collections::HashSet::new();
            people.retain(|p| seen.insert(p.pos));
            let mut b = Battle::new(people, vec![]);
            b.run();
            assert!(b.over.is_some());
            assert!(b.round <= ROUND_CAP + 1);
        }
    }

    #[test]
    fn priests_hold_their_neighbours() {
        let mut b = Battle::new(
            vec![
                person(1, Kind::Scholar, Side::Blue, (3, 4)),
                person(2, Kind::Gatherer, Side::Red, (3, 3)),
                person(3, Kind::Priest, Side::Red, (4, 2)),
            ],
            vec![],
        );
        b.people[1].resolve = 2;
        let mut acts = Vec::new();
        for _ in 0..4 {
            acts.extend(b.step());
            if acts.iter().any(|a| matches!(a, Act::Held { .. })) {
                break;
            }
        }
        assert!(acts.iter().any(|a| matches!(a, Act::Held { .. })), "{acts:?}");
        assert_eq!(b.people[1].side, Side::Red);
        assert_eq!(b.people[1].resolve, 1);
    }

    #[test]
    fn heralds_speak_to_the_whole_column() {
        let mut b = Battle::new(
            vec![
                person(1, Kind::Herald, Side::Blue, (2, 5)),
                person(2, Kind::Gatherer, Side::Red, (2, 3)),
                person(3, Kind::Gatherer, Side::Red, (2, 1)),
            ],
            vec![],
        );
        let acts = b.step();
        match &acts[..] {
            [Act::Round(1), Act::Speak { hits, .. }] => assert_eq!(hits.len(), 2),
            other => panic!("{other:?}"),
        }
    }

    #[test]
    fn three_turns_make_a_veteran() {
        let mut b = Battle::new(vec![person(1, Kind::Scholar, Side::Blue, (3, 5)), person(2, Kind::Gatherer, Side::Red, (3, 4))], vec![]);
        b.people[0].career = 2;
        b.people[1].resolve = 1;
        let acts = b.run();
        assert!(acts.iter().any(|a| matches!(a, Act::Veteran { who: 0 })));
        assert!(b.people[0].veteran);
        assert_eq!(b.people[0].max, 5);
    }
}
