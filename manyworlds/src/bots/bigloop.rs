//! Big loop: the low-attention strategy the design worries about. Builds one closed loop through
//! every open station the stock allows, then lets the train circle. When a station opens off the
//! loop it splices in a detour, done while the train is elsewhere on the loop. It plays the loop
//! as well as it can: express between stations, let go before each.

use crate::geom::*;
use crate::pilot::{Bot, Gesture};
use crate::plan::*;
use crate::rng::Rng;
use crate::sim::Sim;
use std::collections::VecDeque;

pub struct BigLoop {
    rng: Rng,
    cyc: Vec<Pos>,
    on: Vec<bool>,
    planned: bool,
    pending: Option<Detour>,
    given_up: Vec<bool>,
}

/// Replace the loop between `cyc[i]` and `cyc[j]` (`span` steps on) with `path`, which runs from
/// `cyc[i]` through the new station to `cyc[j]`.
struct Detour {
    i: usize,
    span: usize,
    path: Vec<Pos>,
}

impl BigLoop {
    pub fn new(seed: u64) -> BigLoop {
        BigLoop {
            rng: Rng::new(seed ^ 0x6c6f_6f70),
            cyc: vec![],
            on: vec![false; NT],
            planned: false,
            pending: None,
            given_up: vec![],
        }
    }

    fn set_cycle(&mut self, cyc: Vec<Pos>) {
        self.on = vec![false; NT];
        for p in &cyc {
            self.on[p.idx()] = true;
        }
        self.cyc = cyc;
    }

    /// The shortest loop through as many open stations as the stock allows.
    fn plan(&mut self, sim: &Sim) -> Vec<Pos> {
        let depot = sim.engine();
        let buffer = depot.step(sim.entry);
        let stations: Vec<Pos> = sim
            .stations
            .iter()
            .filter(|s| sim.tick >= s.appear)
            .map(|s| s.pos)
            .collect();
        for k in (1..=stations.len()).rev() {
            let mut best: Option<Vec<Pos>> = None;
            for attempt in 0..120 {
                let mut pick = stations.clone();
                for a in (1..pick.len()).rev() {
                    let b = self.rng.below(a as u32 + 1) as usize;
                    pick.swap(a, b);
                }
                pick.truncate(k);
                if attempt % 2 == 0 {
                    pick = nearest_first(depot, pick);
                }
                if let Some(c) = build(sim, depot, buffer, &pick) {
                    let pieces = c.len() as i32 - 2;
                    if pieces <= sim.p.stock && best.as_ref().is_none_or(|b| c.len() < b.len()) {
                        best = Some(c);
                    }
                }
            }
            if let Some(b) = best {
                return b;
            }
        }
        vec![depot]
    }

    /// The cheapest way to splice station `si` into the loop, if the stock allows.
    fn detour(&self, sim: &Sim, si: usize) -> Option<Detour> {
        let s = sim.stations[si].pos;
        let n = self.cyc.len();
        if n < 4 {
            return None;
        }
        let mut allowed = vec![false; NT];
        for i in 0..NT {
            allowed[i] = !sim.blocked[i]
                && !self.on[i]
                && sim.mask[i] == 0
                && Pos::from_idx(i) != sim.terminus;
        }
        let mut best: Option<(i32, Detour)> = None;
        for i in 0..n {
            for span in 2..=12.min(n - 1) {
                let interior = (1..span).map(|k| self.cyc[(i + k) % n]);
                if interior.clone().any(|t| sim.fixed[t.idx()]) {
                    continue;
                }
                let j = (i + span) % n;
                let Some(p1) = bfs(self.cyc[i], s, &allowed, true) else {
                    continue;
                };
                let mut allowed2 = allowed.clone();
                for t in &p1 {
                    allowed2[t.idx()] = false;
                }
                let Some(p2) = bfs(s, self.cyc[j], &allowed2, true) else {
                    continue;
                };
                let added = (p1.len() - 1 + p2.len() - 2) as i32 - (span as i32 - 1);
                if sim.used + added > sim.p.stock {
                    continue;
                }
                if best.as_ref().is_none_or(|(a, _)| added < *a) {
                    let mut path = p1;
                    path.extend_from_slice(&p2[1..]);
                    best = Some((added, Detour { i, span, path }));
                }
            }
        }
        best.map(|(_, d)| d)
    }

    /// Whether the train is far enough round the loop to rebuild that stretch before it comes.
    fn safe(&self, sim: &Sim, d: &Detour) -> bool {
        let n = self.cyc.len();
        if (0..=d.span).any(|k| sim.on_body(self.cyc[(d.i + k) % n])) {
            return false;
        }
        let Some(e) = self.cyc.iter().position(|&p| p == sim.engine()) else {
            return false;
        };
        let ahead = ((d.i + n - e) % n) as i32;
        let work = 10 + (d.span as i32 - 1) + 2 * d.path.len() as i32;
        ahead * sim.p.express_ticks > work + 15
    }

    fn apply(&mut self, d: Detour) {
        let n = self.cyc.len();
        let mut new = d.path.clone();
        new.pop(); // cyc[j] comes again below
        for k in d.span..n {
            new.push(self.cyc[(d.i + k) % n]);
        }
        self.set_cycle(new);
    }

    /// Drags that draw whatever part of the loop is missing, nearest the engine first.
    fn missing(&self, sim: &Sim) -> Vec<Vec<Pos>> {
        let n = self.cyc.len();
        let mut out = vec![];
        if n < 3 {
            return out;
        }
        let e = self
            .cyc
            .iter()
            .position(|&p| p == sim.engine())
            .unwrap_or(0);
        let mut cur: Vec<Pos> = vec![];
        for s in 0..n {
            let i = (e + s) % n;
            let (a, b) = (self.cyc[i], self.cyc[(i + 1) % n]);
            let d = a.dir_to(b).expect("loop tiles are neighbours");
            if sim.mask[a.idx()] & d.bit() == 0 {
                if cur.is_empty() {
                    cur.push(self.cyc[(i + n - 1) % n]);
                    cur.push(a);
                }
                cur.push(b);
            } else if !cur.is_empty() {
                out.push(std::mem::take(&mut cur));
            }
        }
        if !cur.is_empty() {
            out.push(cur);
        }
        out
    }
}

impl Bot for BigLoop {
    fn name(&self) -> &'static str {
        "bigloop"
    }

    fn think(&mut self, sim: &Sim) -> Vec<Gesture> {
        if !self.planned {
            let c = self.plan(sim);
            self.set_cycle(c);
            self.given_up = vec![false; sim.stations.len()];
            self.planned = true;
        }
        if self.pending.is_none() {
            for si in 0..sim.stations.len() {
                let s = &sim.stations[si];
                if sim.tick < s.appear || self.on[s.pos.idx()] || self.given_up[si] {
                    continue;
                }
                match self.detour(sim, si) {
                    Some(d) => {
                        self.pending = Some(d);
                        break;
                    }
                    None => self.given_up[si] = true,
                }
            }
        }
        if self.pending.as_ref().is_some_and(|d| self.safe(sim, d)) {
            let d = self.pending.take().unwrap();
            self.apply(d);
        }
        let mut out = vec![Gesture::Express(express_policy(sim, &route_ahead(sim)))];
        let junk: Vec<Pos> = (0..NT)
            .filter(|&i| sim.mask[i] != 0 && !sim.fixed[i] && !self.on[i])
            .map(Pos::from_idx)
            .filter(|&p| !sim.on_body(p))
            .collect();
        if !junk.is_empty() {
            out.push(Gesture::Erase(junk));
        }
        out.extend(self.missing(sim).into_iter().map(Gesture::Stroke));
        out
    }
}

fn nearest_first(from: Pos, mut rest: Vec<Pos>) -> Vec<Pos> {
    let mut out = vec![];
    let mut at = from;
    while !rest.is_empty() {
        let k = (0..rest.len())
            .min_by_key(|&k| rest[k].manhattan(at))
            .unwrap();
        at = rest.swap_remove(k);
        out.push(at);
    }
    out
}

/// A loop from the depot through `order` and back in to the depot through its buffer.
fn build(sim: &Sim, depot: Pos, buffer: Pos, order: &[Pos]) -> Option<Vec<Pos>> {
    let mut allowed = vec![false; NT];
    for i in 0..NT {
        allowed[i] = !sim.blocked[i] && !sim.fixed[i] && Pos::from_idx(i) != sim.terminus;
    }
    let mut cyc = vec![depot];
    let mut cur = depot;
    for &t in order.iter().chain([buffer].iter()) {
        if t != buffer && !allowed[t.idx()] {
            continue; // already on the loop
        }
        let path = bfs(cur, t, &allowed, !(cur == depot && t == buffer))?;
        for &p in &path[1..] {
            cyc.push(p);
            allowed[p.idx()] = false;
        }
        cur = t;
    }
    Some(cyc)
}

/// Shortest path from `a` to `b` over `allowed` tiles (`b` itself may be anything). With
/// `direct` false the path may not be the single step from `a` to `b`.
fn bfs(a: Pos, b: Pos, allowed: &[bool], direct: bool) -> Option<Vec<Pos>> {
    let mut parent = vec![usize::MAX; NT];
    parent[a.idx()] = a.idx();
    let mut q = VecDeque::from([a]);
    while let Some(p) = q.pop_front() {
        for d in Dir::ALL {
            let n = p.step(d);
            if !n.ok() || parent[n.idx()] != usize::MAX {
                continue;
            }
            if n == b && (direct || p != a) {
                parent[n.idx()] = p.idx();
                let mut v = vec![n];
                let mut i = n.idx();
                while i != a.idx() {
                    i = parent[i];
                    v.push(Pos::from_idx(i));
                }
                v.reverse();
                return Some(v);
            }
            if allowed[n.idx()] {
                parent[n.idx()] = p.idx();
                q.push_back(n);
            }
        }
    }
    None
}
