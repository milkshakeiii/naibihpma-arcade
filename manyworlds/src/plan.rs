//! What the bots can see and work out from the screen: the route ahead, who will be aboard when
//! the train gets there, and paths for new rail.

use crate::geom::*;
use crate::pilot::Gesture;
use crate::sim::Sim;
use std::collections::VecDeque;

pub const FAR: i32 = i32::MAX / 4;

/// The rail the train will follow from where it is now, until it would stop.
pub struct Route {
    /// Tiles ahead of the engine, in order; the engine's own tile is not included.
    pub tiles: Vec<Pos>,
    /// The last tile the train can reach (the engine's tile if it can't move).
    pub end: Pos,
    pub end_entry: Dir,
    /// The route comes back on itself: the train will circle forever.
    pub looped: bool,
}

impl Route {
    /// Where a drag that extends this route starts: one tile back, so the drag passes through
    /// the end tile and lights any switch it makes the way the train is going.
    pub fn stroke_start(&self, sim: &Sim) -> Vec<Pos> {
        let n = self.tiles.len();
        let mut v = vec![];
        if n >= 2 {
            v.push(self.tiles[n - 2]);
        } else if n == 1 {
            v.push(sim.engine());
        } else if sim.mask[self.end.idx()] & self.end_entry.bit() != 0 {
            v.push(self.end.step(self.end_entry));
        }
        v.push(self.end);
        v
    }

    /// A drag that extends the route through `ext`.
    pub fn stroke(&self, sim: &Sim, ext: &[Pos]) -> Vec<Pos> {
        let mut v = self.stroke_start(sim);
        v.extend_from_slice(ext);
        v
    }

    /// Index in `tiles` of the next open station.
    pub fn next_stop(&self, sim: &Sim) -> Option<usize> {
        self.tiles.iter().position(|&t| sim.station(t).is_some())
    }
}

pub fn route_ahead(sim: &Sim) -> Route {
    let mut seen = vec![0u8; NT];
    let mut cur = sim.engine();
    let mut e = sim.entry;
    seen[cur.idx()] |= e.bit();
    let mut tiles = vec![];
    let mut looped = false;
    while let Some(d) = sim.exit_of(cur, e) {
        let n = cur.step(d);
        if !sim.can_enter(n, d.opp()) {
            break;
        }
        let ne = d.opp();
        if seen[n.idx()] & ne.bit() != 0 {
            looped = true;
            break;
        }
        seen[n.idx()] |= ne.bit();
        tiles.push(n);
        cur = n;
        e = ne;
    }
    Route {
        tiles,
        end: cur,
        end_entry: e,
        looped,
    }
}

/// Tiles under the train or on its route: rail the bot must not erase.
pub fn keep_mask(sim: &Sim, route: &Route) -> Vec<bool> {
    let mut keep = vec![false; NT];
    for &b in &sim.body {
        keep[b.idx()] = true;
    }
    for &t in &route.tiles {
        keep[t.idx()] = true;
    }
    keep
}

/// Rail nobody needs any more, ordered as one sweep outward from the train's tail.
pub fn leftovers(sim: &Sim, keep: &[bool]) -> Vec<Pos> {
    let want: Vec<bool> = (0..NT)
        .map(|i| sim.mask[i] != 0 && !sim.fixed[i] && !keep[i])
        .collect();
    let mut out = vec![];
    let mut seen = vec![false; NT];
    let tail = *sim.body.back().unwrap();
    seen[tail.idx()] = true;
    let mut q = VecDeque::from([tail]);
    while let Some(p) = q.pop_front() {
        for d in arms(sim.mask[p.idx()]) {
            let m = p.step(d);
            if m.ok() && want[m.idx()] && !seen[m.idx()] {
                seen[m.idx()] = true;
                out.push(m);
                q.push_back(m);
            }
        }
    }
    out.extend((0..NT).filter(|&i| want[i] && !seen[i]).map(Pos::from_idx));
    out
}

/// Who will be aboard, and how many passengers each station will have lost, at some point ahead.
#[derive(Clone)]
pub struct Pred {
    pub cars: Vec<u8>,
    pub taken: Vec<usize>,
}

/// The train's load after it has run the route ahead.
pub fn predict(sim: &Sim, route: &Route) -> Pred {
    let mut pred = Pred {
        cars: sim.cars.clone(),
        taken: vec![0; sim.stations.len()],
    };
    for &t in &route.tiles {
        if let Some(si) = sim.station(t) {
            visit(sim, &mut pred, si);
        }
    }
    pred
}

/// Stopping at station `si`: (delivered, boarded).
pub fn visit(sim: &Sim, pred: &mut Pred, si: usize) -> (usize, usize) {
    let s = &sim.stations[si];
    let delivered = crate::sim::unload(&mut pred.cars, s.shape, sim.p.unload);
    let mut boarded = 0;
    while pred.cars.len() < sim.p.capacity && pred.taken[si] < s.queue.len() {
        pred.cars.push(s.queue[pred.taken[si]]);
        pred.taken[si] += 1;
        boarded += 1;
    }
    (delivered, boarded)
}

/// Whether stopping at open station `si` would deliver or board anyone.
pub fn useful(sim: &Sim, pred: &Pred, si: usize) -> bool {
    let s = &sim.stations[si];
    sim.tick >= s.appear
        && (crate::sim::unload(&mut pred.cars.clone(), s.shape, sim.p.unload) > 0
            || (pred.cars.len() < sim.p.capacity && pred.taken[si] < s.queue.len()))
}

/// Steps from `from` to every tile over open ground, ignoring rail and the train.
pub fn ground_dist(sim: &Sim, from: Pos) -> Vec<i32> {
    let mut dist = vec![FAR; NT];
    dist[from.idx()] = 0;
    let mut q = VecDeque::from([from]);
    while let Some(p) = q.pop_front() {
        for d in Dir::ALL {
            let n = p.step(d);
            if n.ok() && !sim.blocked[n.idx()] && dist[n.idx()] == FAR {
                dist[n.idx()] = dist[p.idx()] + 1;
                q.push_back(n);
            }
        }
    }
    dist
}

/// Shortest new rail from the route's end over tiles that are empty now, to the first tile that
/// `goal` accepts. `avoid` tiles can be a goal but are never passed through. Returns the path
/// including the route's end.
pub fn path_now(
    sim: &Sim,
    route: &Route,
    goal: impl Fn(Pos) -> bool,
    avoid: &[bool],
) -> Option<Vec<Pos>> {
    let mut parent = vec![usize::MAX; NT];
    let start = route.end;
    parent[start.idx()] = start.idx();
    let mut q = VecDeque::from([start]);
    while let Some(p) = q.pop_front() {
        for d in Dir::ALL {
            if p == start && d == route.end_entry {
                continue;
            }
            let n = p.step(d);
            if !n.ok() || parent[n.idx()] != usize::MAX || sim.blocked[n.idx()] {
                continue;
            }
            if sim.mask[n.idx()] != 0 || sim.on_body(n) {
                continue;
            }
            parent[n.idx()] = p.idx();
            if goal(n) {
                return Some(trace(&parent, start, n));
            }
            if !avoid[n.idx()] {
                q.push_back(n);
            }
        }
    }
    None
}

fn trace(parent: &[usize], start: Pos, end: Pos) -> Vec<Pos> {
    let mut v = vec![end];
    let mut i = end.idx();
    while i != start.idx() {
        i = parent[i];
        v.push(Pos::from_idx(i));
    }
    v.reverse();
    v
}

/// Distances for new rail from the route's end, counting tiles the train will have left by the
/// time the engine gets there: rail under the train or on its route frees up once the tail has
/// passed. `train_len` is the longest the train will be meanwhile; `margin` is spare moves.
/// `avoid` tiles can be reached but not passed through.
pub struct Reach {
    pub dist: Vec<i32>,
    parent: Vec<usize>,
    start: Pos,
}

impl Reach {
    pub fn new(sim: &Sim, route: &Route, train_len: i32, margin: i32, avoid: &[bool]) -> Reach {
        let r = route.tiles.len() as i32;
        // The engine move count after which each tile is free. Leftover rail counts as free: the
        // bot erases it this turn, and only draws over a tile once it is empty.
        let mut free_at = vec![0; NT];
        for i in 0..NT {
            if sim.blocked[i] || sim.fixed[i] {
                free_at[i] = FAR;
            }
        }
        for (k, b) in sim.body.iter().enumerate() {
            free_at[b.idx()] = train_len - k as i32 + margin;
        }
        for (k, t) in route.tiles.iter().enumerate() {
            free_at[t.idx()] = free_at[t.idx()].max(k as i32 + 1 + train_len + margin);
        }
        let mut dist = vec![FAR; NT];
        let mut parent = vec![usize::MAX; NT];
        let start = route.end;
        dist[start.idx()] = 0;
        parent[start.idx()] = start.idx();
        let mut layer = vec![start];
        let mut g = 0;
        while !layer.is_empty() {
            let mut next = vec![];
            for &p in &layer {
                if p != start && avoid[p.idx()] {
                    continue;
                }
                for d in Dir::ALL {
                    if p == start && d == route.end_entry {
                        continue;
                    }
                    let n = p.step(d);
                    if !n.ok() || dist[n.idx()] != FAR || r + g + 1 < free_at[n.idx()] {
                        continue;
                    }
                    dist[n.idx()] = g + 1;
                    parent[n.idx()] = p.idx();
                    next.push(n);
                }
            }
            layer = next;
            g += 1;
        }
        Reach {
            dist,
            parent,
            start,
        }
    }

    pub fn path_to(&self, t: Pos) -> Option<Vec<Pos>> {
        (self.dist[t.idx()] < FAR).then(|| trace(&self.parent, self.start, t))
    }
}

/// The free tiles at the start of `path[1..]`: rail can be drawn there right now.
pub fn drawable_prefix(sim: &Sim, path: &[Pos], max: usize) -> Vec<Pos> {
    path.iter()
        .skip(1)
        .take_while(|&&t| sim.mask[t.idx()] == 0 && !sim.on_body(t))
        .take(max)
        .copied()
        .collect()
}

/// Put a turn's handwork in order. The hand can't be at both ends of the train at once, so rail
/// behind is only picked up when pieces run low or there is nothing to draw. `clear` is old rail
/// in the way of the planned path: that goes first. Then drawing, unless there are no pieces
/// to draw with at all.
pub fn handwork(
    sim: &Sim,
    stroke: Option<Vec<Pos>>,
    mut junk: Vec<Pos>,
    clear: Vec<Pos>,
) -> Vec<Gesture> {
    // One sweep of at most six tiles, so the hand is soon back at the head of the line.
    junk.truncate(6);
    let new_tiles = stroke
        .as_ref()
        .map_or(0, |s| s.iter().filter(|t| sim.mask[t.idx()] == 0).count())
        as i32;
    let mut out = vec![];
    if !clear.is_empty() {
        out.push(Gesture::Erase(clear));
    }
    let broke = sim.free() < 2;
    if broke && !junk.is_empty() {
        out.push(Gesture::Erase(junk.clone()));
    }
    let want_more = sim.free() < new_tiles + 4 || stroke.is_none();
    out.extend(stroke.map(Gesture::Stroke));
    if !broke && want_more && !junk.is_empty() {
        out.push(Gesture::Erase(junk));
    }
    out
}

/// Leftover rail on `path` beyond its first tile, which must go before new rail can go there.
/// Rail the train still needs (`keep`) stays: the path only crosses it after the train has gone.
pub fn in_the_way(sim: &Sim, path: &[Pos], max: usize, keep: &[bool]) -> Vec<Pos> {
    path.iter()
        .skip(1)
        .take(max)
        .filter(|&&t| sim.mask[t.idx()] != 0 && !sim.fixed[t.idx()] && !keep[t.idx()])
        .copied()
        .collect()
}

/// Hold express while there is plenty of rail ahead and no station close; let go before
/// stations so the train stops there.
pub fn express_policy(sim: &Sim, route: &Route) -> bool {
    if !sim.p.express_skips {
        return route.tiles.len() >= 6 || route.looped;
    }
    if route.tiles.len() < 6 && !route.looped {
        return false;
    }
    route.next_stop(sim).is_none_or(|i| i >= 3)
}

/// A step or two into the most open space, when there is nowhere useful to go. If the engine is
/// boxed in, run into the train's own cars on purpose (losing them) rather than stall forever.
pub fn survive(sim: &Sim, route: &Route, avoid: &[bool]) -> Option<Vec<Pos>> {
    let end = route.end;
    let mut best: Option<(usize, Pos)> = None;
    for d in Dir::ALL {
        if d == route.end_entry {
            continue;
        }
        let n = end.step(d);
        if !n.ok()
            || sim.blocked[n.idx()]
            || sim.mask[n.idx()] != 0
            || sim.on_body(n)
            || avoid[n.idx()]
        {
            continue;
        }
        let room = open_area(sim, n, avoid);
        if best.is_none_or(|(r, _)| room > r) {
            best = Some((room, n));
        }
    }
    if let Some((_, n)) = best {
        return Some(route.stroke(sim, &[n]));
    }
    if !route.tiles.is_empty() {
        return None;
    }
    for d in Dir::ALL {
        let n = end.step(d);
        if d != route.end_entry && n.ok() && sim.body.iter().skip(1).any(|&b| b == n) {
            return Some(route.stroke(sim, &[n]));
        }
    }
    None
}

fn open_area(sim: &Sim, from: Pos, avoid: &[bool]) -> usize {
    let mut seen = vec![false; NT];
    seen[from.idx()] = true;
    let mut q = VecDeque::from([from]);
    let mut n = 0;
    while let Some(p) = q.pop_front() {
        n += 1;
        if n > 80 {
            break;
        }
        for d in Dir::ALL {
            let m = p.step(d);
            if m.ok()
                && !seen[m.idx()]
                && !sim.blocked[m.idx()]
                && sim.mask[m.idx()] == 0
                && !avoid[m.idx()]
                && !sim.on_body(m)
            {
                seen[m.idx()] = true;
                q.push_back(m);
            }
        }
    }
    n
}

/// Tiles new rail should not pass through: open stations other than the one aimed for, and the
/// terminus (an open terminus would end the leg).
pub fn stations_and_terminus(sim: &Sim) -> Vec<bool> {
    let mut avoid = vec![false; NT];
    for s in &sim.stations {
        if sim.tick >= s.appear {
            avoid[s.pos.idx()] = true;
        }
    }
    avoid[sim.terminus.idx()] = true;
    avoid
}
