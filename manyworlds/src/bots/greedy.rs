//! Greedy: a newcomer who has understood the rules. Always heads for the nearest station where
//! stopping would deliver or board someone, by the shortest path over empty ground, keeps the
//! line drawn ahead, and picks up rail the train has left when it needs the pieces. Never
//! touches express. Heads for the terminus as soon as it opens.

use crate::geom::NT;
use crate::pilot::{Bot, Gesture};
use crate::plan::*;
use crate::sim::Sim;

#[derive(Default)]
pub struct Greedy {
    /// Use express the way the planner does, and steer around stations it isn't heading for
    /// (both only for diagnostics).
    pub express: bool,
    pub avoid_stations: bool,
}

impl Bot for Greedy {
    fn name(&self) -> &'static str {
        "greedy"
    }

    fn think(&mut self, sim: &Sim) -> Vec<Gesture> {
        let route = route_ahead(sim);
        let junk = leftovers(sim, &keep_mask(sim, &route));
        let mut stroke = None;
        if !route.looped {
            let pred = predict(sim, &route);
            let mut avoid = if self.avoid_stations {
                stations_and_terminus(sim)
            } else {
                vec![false; NT]
            };
            avoid[sim.terminus.idx()] = true;
            let path = if sim.terminus_open() {
                path_now(sim, &route, |p| p == sim.terminus, &avoid)
            } else {
                path_now(
                    sim,
                    &route,
                    |p| p != route.end && sim.station(p).is_some_and(|si| useful(sim, &pred, si)),
                    &avoid,
                )
            };
            stroke = match path {
                Some(p) => Some(route.stroke(sim, &p[1..])),
                None if route.tiles.len() < 2 => survive(sim, &route, &avoid),
                None => None,
            };
        }
        let mut out = vec![Gesture::Express(
            self.express && express_policy(sim, &route),
        )];
        out.extend(handwork(sim, stroke, junk, vec![]));
        out
    }
}
