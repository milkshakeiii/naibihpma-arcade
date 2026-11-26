//! Planner: the best play written so far, standing in for a skilled player.
//!
//! - Picks the next station by looking three stops ahead: who would get off, who would board
//!   (first come, first served, up to capacity), and how far each hop is, discounted by distance.
//! - Routes new rail around other stations, and through tiles the train's own tail will have
//!   left by the time the engine arrives.
//! - Keeps only a short stretch drawn ahead, erases rail behind, and erases first when short of
//!   pieces.
//! - Holds express while the line ahead is long and clear of stations, letting go before each.
//! - Reaches the terminus just before the song ends.

use crate::geom::*;
use crate::pilot::{Bot, Gesture};
use crate::plan::*;
use crate::sim::Sim;

/// How much of the route to keep drawn ahead of the engine.
const AHEAD: usize = 14;
/// Discount per tile of travel, and what a boarding passenger is worth against a delivery.
const GAMMA: f64 = 0.955;
const BOARD: f64 = 0.35;
const DEPTH: usize = 3;

pub struct Planner {
    /// Ground distance from each station to every tile.
    from_station: Vec<Vec<i32>>,
    /// Stops to look ahead, and whether to use express (both lowered only for diagnostics).
    pub depth: usize,
    pub express: bool,
}

impl Default for Planner {
    fn default() -> Self {
        Planner::new(DEPTH, true)
    }
}

impl Planner {
    pub fn new(depth: usize, express: bool) -> Planner {
        Planner {
            from_station: vec![],
            depth,
            express,
        }
    }
}

impl Bot for Planner {
    fn name(&self) -> &'static str {
        "planner"
    }

    fn think(&mut self, sim: &Sim) -> Vec<Gesture> {
        if self.from_station.is_empty() {
            self.from_station = sim
                .stations
                .iter()
                .map(|s| ground_dist(sim, s.pos))
                .collect();
        }
        let route = route_ahead(sim);
        let keep = keep_mask(sim, &route);
        let junk = leftovers(sim, &keep);
        let mut stroke = None;
        let mut clear = vec![];
        if !route.looped {
            let pred = predict(sim, &route);
            let train_len = 1 + sim.cars.len().max(pred.cars.len()) as i32;
            let avoid = stations_and_terminus(sim);
            let reach = Reach::new(sim, &route, train_len, 2, &avoid);
            let target = self
                .park(sim, &route, &reach)
                .or_else(|| self.choose(sim, &route, &pred, &reach));
            let path = target.and_then(|t| reach.path_to(t)).or_else(|| {
                // Nowhere worth going is reachable around the stations; try straight through.
                let open = vec![false; NT];
                let r2 = Reach::new(sim, &route, train_len, 2, &open);
                target.and_then(|t| r2.path_to(t))
            });
            let room = AHEAD.saturating_sub(route.tiles.len());
            match path {
                Some(p) => {
                    clear = in_the_way(sim, &p, room.max(4), &keep);
                    let ext = drawable_prefix(sim, &p, room);
                    if !ext.is_empty() {
                        stroke = Some(route.stroke(sim, &ext));
                    }
                }
                None if route.tiles.len() < 2 => stroke = survive(sim, &route, &avoid),
                None => {}
            }
        }
        let mut out = vec![Gesture::Express(
            self.express && express_policy(sim, &route),
        )];
        out.extend(handwork(sim, stroke, junk, clear));
        out
    }
}

impl Planner {
    /// Head for the open terminus once there is only just time to get there.
    fn park(&self, sim: &Sim, route: &Route, reach: &Reach) -> Option<Pos> {
        if !sim.terminus_open() {
            return None;
        }
        let d = reach.dist[sim.terminus.idx()];
        if d >= FAR {
            return None;
        }
        let need = (route.tiles.len() as i32 + d) * sim.p.tile_ticks + 3 * 15;
        (sim.p.ticks - sim.tick <= need).then_some(sim.terminus)
    }

    /// The first stop of the best three-stop plan.
    fn choose(&self, sim: &Sim, route: &Route, pred: &Pred, reach: &Reach) -> Option<Pos> {
        let mut best: Option<(f64, usize)> = None;
        for si in 0..sim.stations.len() {
            let pos = sim.stations[si].pos;
            let d = reach.dist[pos.idx()];
            if pos == route.end || d >= FAR || !useful(sim, pred, si) {
                continue;
            }
            let mut p = pred.clone();
            let (k, b) = visit(sim, &mut p, si);
            let v = worth(sim, k, b, d) + self.rollout(sim, &p, si, d, self.depth - 1);
            if best.is_none_or(|(bv, _)| v > bv) {
                best = Some((v, si));
            }
        }
        best.map(|(_, si)| sim.stations[si].pos)
    }

    fn rollout(&self, sim: &Sim, pred: &Pred, at: usize, travelled: i32, depth: usize) -> f64 {
        if depth == 0 {
            return 0.0;
        }
        let mut best = 0.0;
        for si in 0..sim.stations.len() {
            if si == at || !useful(sim, pred, si) {
                continue;
            }
            let hop = self.from_station[at][sim.stations[si].pos.idx()];
            if hop >= FAR {
                continue;
            }
            let mut p = pred.clone();
            let (k, b) = visit(sim, &mut p, si);
            let d = travelled + hop;
            let v = worth(sim, k, b, d) + self.rollout(sim, &p, si, d, depth - 1);
            if v > best {
                best = v;
            }
        }
        best
    }
}

fn worth(sim: &Sim, delivered: usize, boarded: usize, dist: i32) -> f64 {
    let k = delivered as f64;
    let points = if sim.p.batch_scoring {
        k * (k + 1.0) / 2.0
    } else {
        k
    };
    (points + BOARD * boarded as f64) * GAMMA.powi(dist)
}
