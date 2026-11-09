//! Movement and route finding. A creature walks, drops off edges, jumps (rise, then sideways,
//! then fall) and climbs, and every one of those depends on its species and its load.

use crate::rules::*;
use crate::world::World;
use std::cmp::Reverse;
use std::collections::BinaryHeap;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Move {
    Walk,
    Rise,
    Leap,
    Fall,
    Climb,
}

/// One tile of motion: arrive at (x, y) after `t` ticks.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Step {
    pub x: i32,
    pub y: i32,
    pub t: i32,
    pub m: Move,
}

#[derive(Clone, Debug, Default)]
pub struct Path {
    pub start: (i32, i32),
    pub steps: Vec<Step>,
    pub ticks: i32,
}

impl Path {
    pub fn end(&self) -> (i32, i32) {
        self.steps.last().map(|s| (s.x, s.y)).unwrap_or(self.start)
    }
}

/// Where a creature lands if it falls from (x, y): the landing tile and the drop height.
pub fn fall(world: &World, s: Species, w: Weight, x: i32, y0: i32) -> Option<(i32, i32)> {
    let mut y = y0;
    loop {
        if !world.passable(x, y) {
            return None;
        }
        if world.standable_w(x, y, w) || world.climbable(x, y, s, w) {
            return Some((y, y - y0));
        }
        y += 1;
        if y >= world.h {
            return None;
        }
    }
}

fn push_fall(steps: &mut Vec<Step>, x: i32, y0: i32, dist: i32) {
    for k in 1..=dist {
        steps.push(Step { x, y: y0 + k, t: 1, m: Move::Fall });
    }
}

/// All single moves from (x, y), each as its tile-by-tile steps.
pub fn moves(world: &World, s: Species, load: Load, x: i32, y: i32, out: &mut Vec<Vec<Step>>) {
    out.clear();
    let w = load.weight;
    let p = s.pace(w);
    let max_fall = if load.fragile { FRAGILE_FALL } else { i32::MAX };
    let standing = world.standable_w(x, y, w);
    let hanging = world.climbable(x, y, s, w);
    if !standing && !hanging {
        return;
    }
    // Walk sideways, or step off an edge and fall.
    for d in [-1, 1] {
        let nx = x + d;
        if !world.passable(nx, y) {
            continue;
        }
        if world.standable_w(nx, y, w) || world.climbable(nx, y, s, w) {
            out.push(vec![Step { x: nx, y, t: p, m: Move::Walk }]);
        } else if let Some((_, dist)) = fall(world, s, w, nx, y) {
            if dist <= max_fall {
                let mut v = vec![Step { x: nx, y, t: p, m: Move::Walk }];
                push_fall(&mut v, nx, y, dist);
                out.push(v);
            }
        }
    }
    // Climb.
    if hanging {
        for d in [-1, 1] {
            let ny = y + d;
            if world.climbable(x, ny, s, w) {
                out.push(vec![Step { x, y: ny, t: 2 * p, m: Move::Climb }]);
            } else if d == 1 && world.passable(x, ny) && !world.standable_w(x, y, w) {
                // Let go of the bottom of a hanging vine.
                if let Some((_, dist)) = fall(world, s, w, x, ny) {
                    if dist + 1 <= max_fall {
                        let mut v = vec![Step { x, y: ny, t: 1, m: Move::Fall }];
                        push_fall(&mut v, x, ny, dist);
                        out.push(v);
                    }
                }
            }
        }
    }
    // Jump: rise up to J tiles, carry up to J+1 sideways, then fall.
    if !standing {
        return;
    }
    let j = s.jump(w) + if world.is_spring(x, y) { SPRING_BONUS } else { 0 };
    if j <= 0 {
        return;
    }
    for d in [-1, 1] {
        for h in 1..=j {
            if !world.passable(x, y - h) {
                break;
            }
            for r in 1..=j + 1 {
                let (cx, cy) = (x + d * r, y - h);
                if !world.passable(cx, cy) {
                    break;
                }
                let mut v: Vec<Step> = (1..=h).map(|k| Step { x, y: y - k, t: p, m: Move::Rise }).collect();
                v.extend((1..=r).map(|k| Step { x: x + d * k, y: cy, t: p, m: Move::Leap }));
                if world.standable_w(cx, cy, w) || world.climbable(cx, cy, s, w) {
                    out.push(v);
                    break;
                }
                if let Some((_, dist)) = fall(world, s, w, cx, cy) {
                    if dist <= max_fall {
                        push_fall(&mut v, cx, cy, dist);
                        out.push(v);
                    }
                }
            }
        }
    }
}

/// Cheapest path (in ticks) from any start tile to any goal tile.
pub fn find(world: &World, s: Species, load: Load, starts: &[(i32, i32)], goals: &[(i32, i32)]) -> Option<Path> {
    let n = (world.w * world.h) as usize;
    let mut dist = vec![i32::MAX; n];
    let mut came: Vec<Option<(usize, Vec<Step>)>> = vec![None; n];
    let mut origin = vec![usize::MAX; n];
    let mut heap = BinaryHeap::new();
    for &(x, y) in starts {
        if !world.inside(x, y) {
            continue;
        }
        let i = world.idx(x, y);
        dist[i] = 0;
        origin[i] = i;
        heap.push(Reverse((0i32, i)));
    }
    let is_goal = |i: usize| goals.iter().any(|&(gx, gy)| world.inside(gx, gy) && world.idx(gx, gy) == i);
    let mut buf = Vec::new();
    while let Some(Reverse((d, i))) = heap.pop() {
        if d > dist[i] {
            continue;
        }
        if is_goal(i) {
            let mut steps: Vec<Step> = Vec::new();
            let mut cur = i;
            while let Some((prev, seg)) = &came[cur] {
                for st in seg.iter().rev() {
                    steps.push(*st);
                }
                cur = *prev;
            }
            steps.reverse();
            let start = ((cur as i32) % world.w, (cur as i32) / world.w);
            return Some(Path { start, steps, ticks: d });
        }
        let (x, y) = ((i as i32) % world.w, (i as i32) / world.w);
        moves(world, s, load, x, y, &mut buf);
        for seg in buf.drain(..) {
            let last = *seg.last().unwrap();
            let j = world.idx(last.x, last.y);
            let nd = d + seg.iter().map(|st| st.t).sum::<i32>();
            if nd < dist[j] {
                dist[j] = nd;
                origin[j] = origin[i];
                came[j] = Some((i, seg));
                heap.push(Reverse((nd, j)));
            }
        }
    }
    None
}

/// Every tile from which this creature, with this load, can reach any of `goals`.
pub fn reach_back(world: &World, s: Species, load: Load, goals: &[(i32, i32)]) -> Vec<bool> {
    let n = (world.w * world.h) as usize;
    let mut rev: Vec<Vec<usize>> = vec![Vec::new(); n];
    let mut buf = Vec::new();
    for y in 0..world.h {
        for x in 0..world.w {
            moves(world, s, load, x, y, &mut buf);
            let i = world.idx(x, y);
            for seg in buf.drain(..) {
                let last = *seg.last().unwrap();
                rev[world.idx(last.x, last.y)].push(i);
            }
        }
    }
    let mut seen = vec![false; n];
    let mut stack: Vec<usize> = goals.iter().filter(|&&(x, y)| world.inside(x, y)).map(|&(x, y)| world.idx(x, y)).collect();
    for &i in &stack {
        seen[i] = true;
    }
    while let Some(i) = stack.pop() {
        for &j in &rev[i] {
            if !seen[j] {
                seen[j] = true;
                stack.push(j);
            }
        }
    }
    seen
}

/// Every tile reachable from (x, y) for this creature and load (for the reach overlay).
pub fn reach(world: &World, s: Species, load: Load, x: i32, y: i32) -> Vec<bool> {
    let n = (world.w * world.h) as usize;
    let mut seen = vec![false; n];
    if !world.inside(x, y) {
        return seen;
    }
    let mut stack = vec![(x, y)];
    seen[world.idx(x, y)] = true;
    let mut buf = Vec::new();
    while let Some((cx, cy)) = stack.pop() {
        moves(world, s, load, cx, cy, &mut buf);
        for seg in buf.drain(..) {
            let last = *seg.last().unwrap();
            let j = world.idx(last.x, last.y);
            if !seen[j] {
                seen[j] = true;
                stack.push((last.x, last.y));
            }
        }
    }
    seen
}

/// Every tile's cheapest arrival time from a set of start tiles, for one species and load, with
/// enough breadcrumbs to walk any of those paths. Deliveries compare destinations by looking up
/// a field instead of searching once per candidate.
#[derive(Clone, Debug)]
pub struct Field {
    pub dist: Vec<i32>,
    pub prev: Vec<u32>,
}

pub const FAR: i32 = i32::MAX;

pub fn field(world: &World, s: Species, load: Load, starts: &[(i32, i32)]) -> Field {
    let n = (world.w * world.h) as usize;
    let mut dist = vec![FAR; n];
    let mut prev = vec![u32::MAX; n];
    let mut heap = BinaryHeap::new();
    for &(x, y) in starts {
        if !world.inside(x, y) {
            continue;
        }
        let i = world.idx(x, y);
        dist[i] = 0;
        heap.push(Reverse((0i32, i)));
    }
    let mut buf = Vec::new();
    while let Some(Reverse((d, i))) = heap.pop() {
        if d > dist[i] {
            continue;
        }
        let (x, y) = ((i as i32) % world.w, (i as i32) / world.w);
        moves(world, s, load, x, y, &mut buf);
        for seg in buf.drain(..) {
            let last = *seg.last().unwrap();
            let j = world.idx(last.x, last.y);
            let nd = d + seg.iter().map(|st| st.t).sum::<i32>();
            if nd < dist[j] {
                dist[j] = nd;
                prev[j] = i as u32;
                heap.push(Reverse((nd, j)));
            }
        }
    }
    Field { dist, prev }
}

impl Field {
    pub fn at(&self, world: &World, x: i32, y: i32) -> i32 {
        if !world.inside(x, y) {
            return FAR;
        }
        self.dist[world.idx(x, y)]
    }

    /// The closest of `goals`: its time and tile.
    pub fn best(&self, world: &World, goals: &[(i32, i32)]) -> Option<(i32, (i32, i32))> {
        goals.iter().map(|&(x, y)| (self.at(world, x, y), (x, y))).filter(|(d, _)| *d < FAR).min_by_key(|(d, _)| *d)
    }

    /// The path the field found to (gx, gy), step by step.
    pub fn path_to(&self, world: &World, s: Species, load: Load, gx: i32, gy: i32) -> Option<Path> {
        if !world.inside(gx, gy) || self.dist[world.idx(gx, gy)] == FAR {
            return None;
        }
        let mut segs: Vec<Vec<Step>> = Vec::new();
        let mut cur = world.idx(gx, gy);
        let mut buf = Vec::new();
        let mut guard = 0;
        while self.prev[cur] != u32::MAX {
            let p = self.prev[cur] as usize;
            let (px, py) = ((p as i32) % world.w, (p as i32) / world.w);
            let want = self.dist[cur] - self.dist[p];
            moves(world, s, load, px, py, &mut buf);
            let seg = buf.drain(..).find(|seg| {
                let last = seg.last().unwrap();
                world.idx(last.x, last.y) == cur && seg.iter().map(|st| st.t).sum::<i32>() == want
            })?;
            segs.push(seg);
            cur = p;
            guard += 1;
            if guard > 100_000 {
                return None;
            }
        }
        let start = ((cur as i32) % world.w, (cur as i32) / world.w);
        let steps: Vec<Step> = segs.into_iter().rev().flatten().collect();
        let ticks = steps.iter().map(|s| s.t).sum();
        Some(Path { start, steps, ticks })
    }
}
