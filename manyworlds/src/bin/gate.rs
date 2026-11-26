//! Phase 0 gate: four bots play the rules, with no graphics, under human speed limits.
//!
//!   cargo run --release --bin gate                    the gate at the current rules
//!   cargo run --release --bin gate -- sweep           the same across rule variants
//!   cargo run --release --bin gate -- show planner town 1 300 900
//!                                                     the board at those ticks of one run
//!
//! Lamps are set per map from the planner's mean score there: one lamp at 30%, two at 60%,
//! three at 85%. The gate asks that random earns no lamps, greedy about one, the planner three,
//! and that the big loop scores below the planner.

use manyworlds::bots;
use manyworlds::map::{self, Map};
use manyworlds::pilot::{play, Limits, Pilot};
use manyworlds::show::render;
use manyworlds::sim::{Params, Sim, Stats, Unload};
use std::thread;

/// "The Corner of Maple St. and the Future" is 4:49.
const LEG_SECONDS: i32 = 289;
const MAPS: u64 = 12;
const SEEDS: u64 = 8;
const LAMPS: [f64; 3] = [0.30, 0.60, 0.85];

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    match args.first().map(String::as_str) {
        Some("show") => show(&args[1..]),
        Some("sweep") => sweep(),
        Some("lamps") => {
            // Per-map lamp thresholds at the current rules: 30/60/85% of the planner's mean.
            let p = Params::for_track(LEG_SECONDS);
            let res = evaluate(&p, &["planner"]);
            for (m, map) in map::gate_maps(MAPS).iter().enumerate() {
                let mean = mean(res[0][m].iter().map(|s| s.score as f64));
                let t: Vec<i32> = LAMPS.iter().map(|f| (f * mean).round() as i32).collect();
                println!(
                    "{:<6} planner mean {:>5.0}  lamps at {:?}",
                    map.name, mean, t
                );
            }
        }
        Some("diag") => {
            let want = args.get(1).map(String::as_str).unwrap_or("current");
            let (title, p) = variants()
                .into_iter()
                .find(|(t, _)| t.contains(want))
                .expect("no such variant");
            let names: Vec<&str> = bots::NAMES
                .iter()
                .chain(bots::DIAGNOSTIC.iter())
                .copied()
                .collect();
            report(
                &format!("{title}, with one-skill variants"),
                &p,
                &evaluate(&p, &names),
                &names,
            );
        }
        _ => {
            let p = Params::for_track(LEG_SECONDS);
            report(
                "current rules",
                &p,
                &evaluate(&p, &bots::NAMES),
                &bots::NAMES,
            );
        }
    }
}

/// results[bot][map][seed]
fn evaluate(p: &Params, names: &[&str]) -> Vec<Vec<Vec<Stats>>> {
    let maps = map::gate_maps(MAPS);
    let per_map: Vec<Vec<Vec<Stats>>> = thread::scope(|s| {
        let handles: Vec<_> = maps
            .iter()
            .map(|m| {
                let p = p.clone();
                s.spawn(move || {
                    names
                        .iter()
                        .map(|name| (1..=SEEDS).map(|seed| one(m, &p, name, seed)).collect())
                        .collect::<Vec<Vec<Stats>>>()
                })
            })
            .collect();
        handles.into_iter().map(|h| h.join().unwrap()).collect()
    });
    (0..names.len())
        .map(|b| per_map.iter().map(|m| m[b].clone()).collect())
        .collect()
}

fn one(m: &Map, p: &Params, bot: &str, seed: u64) -> Stats {
    let mut sim = Sim::new(m, p.clone(), seed);
    let mut b = bots::make(bot, seed);
    play(&mut sim, b.as_mut(), Limits::default());
    sim.stats
}

fn report(title: &str, p: &Params, res: &[Vec<Vec<Stats>>], names: &[&str]) {
    let planner = names.iter().position(|&n| n == "planner").unwrap();
    let maps = res[0].len();
    let plan_mean: Vec<f64> = (0..maps)
        .map(|m| mean(res[planner][m].iter().map(|s| s.score as f64)))
        .collect();
    println!(
        "\n== {title}: stock {}, erase delay {} ticks, capacity {}, express {}, batch scoring {}, {} maps x {} seeds",
        p.stock,
        p.erase_delay,
        p.capacity,
        if p.express_skips { "skips stations" } else { "stops at stations" },
        if p.batch_scoring { "on" } else { "off" },
        maps,
        res[0][0].len()
    );
    println!("bot                score  vs planner  delivered  lost  stalls  crashes  express  lamps 0/1/2/3");
    for (b, name) in names.iter().enumerate() {
        let all: Vec<&Stats> = res[b].iter().flatten().collect();
        let mut ratio = vec![];
        let mut lamps = [0; 4];
        for m in 0..maps {
            for s in &res[b][m] {
                let r = s.score as f64 / plan_mean[m].max(1.0);
                ratio.push(r);
                lamps[LAMPS.iter().filter(|&&t| r >= t).count()] += 1;
            }
        }
        let f = |g: &dyn Fn(&Stats) -> i32| mean(all.iter().map(|s| g(s) as f64));
        println!(
            "{:<17} {:>6.0} {:>10.0}% {:>10.1} {:>5.1} {:>7.1} {:>8.1} {:>7.0}%  {}/{}/{}/{}",
            name,
            f(&|s| s.score),
            100.0 * mean(ratio.iter().copied()),
            f(&|s| s.delivered),
            f(&|s| s.lost),
            f(&|s| s.stalls),
            f(&|s| s.crashes),
            100.0
                * mean(
                    all.iter()
                        .map(|s| s.express_moves as f64 / s.moves.max(1) as f64)
                ),
            lamps[0],
            lamps[1],
            lamps[2],
            lamps[3],
        );
    }
}

fn mean(it: impl Iterator<Item = f64>) -> f64 {
    let (mut n, mut t) = (0.0, 0.0);
    for v in it {
        n += 1.0;
        t += v;
    }
    if n == 0.0 {
        0.0
    } else {
        t / n
    }
}

fn sweep() {
    let only = std::env::args().nth(2);
    for (title, p) in variants() {
        if only.as_deref().is_some_and(|o| !title.contains(o)) {
            continue;
        }
        report(&title, &p, &evaluate(&p, &bots::NAMES), &bots::NAMES);
    }
}

fn variants() -> Vec<(String, Params)> {
    let base = Params::for_track(LEG_SECONDS);
    let mut v: Vec<(String, Params)> = vec![("current rules".into(), base.clone())];
    let mut add = |title: &str, f: &dyn Fn(&mut Params)| {
        let mut p = base.clone();
        f(&mut p);
        v.push((title.to_string(), p));
    };
    add("any car can get off (the rules as first written)", &|p| {
        p.unload = Unload::Any;
        p.auto_lift = false;
    });
    add("front car only", &|p| p.unload = Unload::Front);
    add("rail behind not lifted", &|p| p.auto_lift = false);
    add("streak needs a delivery every 12 s", &|p| {
        p.combo_window = 12 * 15
    });
    add("stock 25", &|p| p.stock = 25);
    add("stock 60", &|p| p.stock = 60);
    add("stock 120", &|p| p.stock = 120);
    add("express stops at stations", &|p| p.express_skips = false);
    add("capacity 4", &|p| p.capacity = 4);
    add("capacity 10", &|p| p.capacity = 10);
    add("every station its own shape", &|p| {
        p.distinct_stations = true
    });
    add("half as many passengers", &|p| {
        p.spawn_start *= 2;
        p.spawn_end *= 2;
    });
    v
}

fn show(args: &[String]) {
    let bot = args.first().map(String::as_str).unwrap_or("planner");
    let m =
        map::by_name(args.get(1).map(String::as_str).unwrap_or("town")).expect("map: town or genN");
    let seed: u64 = args.get(2).and_then(|s| s.parse().ok()).unwrap_or(1);
    let mut at: Vec<i32> = args.iter().skip(3).filter_map(|s| s.parse().ok()).collect();
    at.sort();
    // "stalls" shows the board as each of the first few stalls begins
    let mut stalls_left = if args.iter().any(|a| a == "stalls") {
        4
    } else {
        0
    };
    let mut sim = Sim::new(&m, Params::for_track(LEG_SECONDS), seed);
    let mut b = bots::make(bot, seed);
    let mut pilot = Pilot::new(Limits::default());
    while !sim.finished() {
        if at.first() == Some(&sim.tick) {
            at.remove(0);
            println!("{}", render(&sim));
        }
        let before = sim.stats.stalls;
        pilot.tick(&mut sim, b.as_mut());
        sim.step();
        if sim.stats.stalls > before && stalls_left > 0 {
            stalls_left -= 1;
            println!("-- stall --\n{}", render(&sim));
        }
    }
    println!("{}", render(&sim));
    println!("{:?}", sim.stats);
}
