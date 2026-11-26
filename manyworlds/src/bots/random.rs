//! Random: the floor. Scribbles rail off the end of the line, erases and clicks at random, and
//! taps express now and then. Anything it scores comes from the rules, not from play.

use crate::geom::*;
use crate::pilot::{Bot, Gesture};
use crate::plan::*;
use crate::rng::Rng;
use crate::sim::Sim;

pub struct Random {
    rng: Rng,
}

impl Random {
    pub fn new(seed: u64) -> Random {
        Random {
            rng: Rng::new(seed ^ 0x7261_6e64),
        }
    }
}

impl Bot for Random {
    fn name(&self) -> &'static str {
        "random"
    }

    fn think(&mut self, sim: &Sim) -> Vec<Gesture> {
        let route = route_ahead(sim);
        match self.rng.below(10) {
            0..=5 => {
                let mut ext = vec![];
                let mut at = route.end;
                let mut back = Some(route.end_entry);
                for _ in 0..self.rng.range(1, 3) {
                    let opts: Vec<Pos> = Dir::ALL
                        .into_iter()
                        .filter(|&d| Some(d) != back)
                        .map(|d| at.step(d))
                        .filter(|&n| {
                            n.ok()
                                && !sim.blocked[n.idx()]
                                && sim.mask[n.idx()] == 0
                                && !ext.contains(&n)
                        })
                        .collect();
                    if opts.is_empty() {
                        break;
                    }
                    let n = opts[self.rng.below(opts.len() as u32) as usize];
                    back = n.dir_to(at);
                    ext.push(n);
                    at = n;
                }
                vec![Gesture::Stroke(route.stroke(sim, &ext))]
            }
            6 => {
                let rail: Vec<Pos> = (0..NT)
                    .filter(|&i| sim.mask[i] != 0)
                    .map(Pos::from_idx)
                    .collect();
                let t = rail[self.rng.below(rail.len() as u32) as usize];
                vec![Gesture::Erase(vec![t])]
            }
            7 => {
                let switches: Vec<Pos> = (0..NT)
                    .filter(|&i| sim.mask[i].count_ones() == 3)
                    .map(Pos::from_idx)
                    .collect();
                if switches.is_empty() {
                    return vec![];
                }
                vec![Gesture::Flip(
                    switches[self.rng.below(switches.len() as u32) as usize],
                )]
            }
            8 => vec![Gesture::Express(!sim.express)],
            _ => vec![],
        }
    }
}
