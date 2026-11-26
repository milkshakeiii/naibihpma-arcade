//! Four ways to play, from button-mashing to careful planning. The gate compares their scores.

pub mod bigloop;
pub mod greedy;
pub mod planner;
pub mod random;

use crate::pilot::Bot;

pub const NAMES: [&str; 4] = ["random", "greedy", "bigloop", "planner"];
pub const DIAGNOSTIC: [&str; 5] = [
    "greedy+express",
    "greedy+express+avoid",
    "planner-express",
    "planner-lookahead",
    "planner-deeper",
];

pub fn make(name: &str, seed: u64) -> Box<dyn Bot> {
    match name {
        "random" => Box::new(random::Random::new(seed)),
        "greedy" => Box::new(greedy::Greedy::default()),
        // Variants that switch one skill on or off, to see where the planner's lead comes from.
        "greedy+express" => Box::new(greedy::Greedy {
            express: true,
            avoid_stations: false,
        }),
        "greedy+express+avoid" => Box::new(greedy::Greedy {
            express: true,
            avoid_stations: true,
        }),
        "planner-express" => Box::new(planner::Planner::new(3, false)),
        "planner-lookahead" => Box::new(planner::Planner::new(1, true)),
        "planner-deeper" => Box::new(planner::Planner::new(4, true)),
        "bigloop" => Box::new(bigloop::BigLoop::new(seed)),
        "planner" => Box::new(planner::Planner::default()),
        _ => panic!("unknown bot {name}; try one of {NAMES:?}"),
    }
}
