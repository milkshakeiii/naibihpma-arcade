//! Play the first scenario headless with a scripted Elder and print how it goes.
//!     cargo run --release --bin probe            five seeds, the summary
//!     cargo run --release --bin probe -- 3 -v    one seed, every five minutes and the log

use hold::bot::{Bot, Style};
use hold::data::*;
use hold::world::{clock, World};

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let verbose = args.iter().any(|a| a == "-v");
    let style = if args.iter().any(|a| a == "--none") {
        Style::NoRewards
    } else if args.iter().any(|a| a == "--idle") {
        Style::Idle
    } else {
        Style::Steady
    };
    let seeds: Vec<u64> = match args.get(1).and_then(|s| s.parse().ok()) {
        Some(s) => vec![s],
        None => (1..=5).collect(),
    };
    let mut rows = Vec::new();
    for seed in seeds {
        let mut w = World::new(seed);
        w.keep_log = verbose;
        let mut bot = Bot::new(style);
        let limit = 75 * MINUTE;
        while w.over.is_none() && w.t < limit {
            bot.step(&mut w);
            w.tick();
            if verbose && w.t % (5 * MINUTE) == 0 {
                report(&w);
            }
            w.events.clear();
        }
        if verbose {
            for f in w.flags.iter().filter(|f| !f.done) {
                println!("open reward {:?} {:?} {} dew, takers {:?}: {}", f.kind, f.target, f.dew, f.takers.len(), w.candidate_summary(f.id));
            }
            let turtles: Vec<String> = w.cs.iter().filter(|c| c.alive() && c.kind == Kind::Turtle).map(|c| format!("{:?} hp{} {:?}", c.pos, c.hp(), c.task)).collect();
            println!("turtles: {turtles:?}");
            for c in w.cs.iter().filter(|c| c.alive() && c.hero()) {
                println!("  {} {:?} L{} {:?} at {:?}: {}", c.name, c.kind, c.level, c.gear, c.pos, c.said);
            }
            for (t, s) in w.log.iter() {
                println!("  {} {}", clock(*t), s);
            }
        }
        let result = match w.over {
            Some(true) => format!("WON at {}", clock(w.t)),
            Some(false) => format!("LOST at {}", clock(w.t)),
            None => "unfinished at 75:00".into(),
        };
        println!("seed {seed}: {result}; deaths {}, kills {}", w.stats.deaths, w.stats.kills);
        rows.push(w);
    }
    let keys = [
        "IRONCAP CAVE built",
        "first CAP INGOT picked",
        "STAR POND built",
        "first IRON SPEAR bought",
        "first SPOTTED CAP picked",
        "first AXOLOTL hatched",
        "SPAWNING POOL built",
        "first BULLFROG hatched",
        "THE OLM'S CAVE built",
        "first THE OLM hatched",
    ];
    println!();
    for k in keys {
        let mut line = format!("{k:28}");
        for w in &rows {
            line += &format!(" {:>7}", w.stats.at(k).map_or("--".into(), clock));
        }
        println!("{line}");
    }
}

fn report(w: &World) {
    let hall = w.hall().map(|h| w.b(h).has(Item::Dew)).unwrap_or(0);
    let mut kinds: Vec<(Kind, usize)> = Vec::new();
    for c in w.cs.iter().filter(|c| c.alive()) {
        if let Some(e) = kinds.iter_mut().find(|e| e.0 == c.kind) {
            e.1 += 1;
        } else {
            kinds.push((c.kind, 1));
        }
    }
    kinds.sort();
    let hold = w.hold_count();
    let nests = w.bs.iter().filter(|b| !b.dead && matches!(b.kind, BKind::Nest | BKind::LoneNest)).count();
    let heroes: Vec<String> = w.cs.iter().filter(|c| c.alive() && c.hero()).map(|c| format!("{}{}:{}", &c.name[..2.min(c.name.len())], c.level, c.purse)).collect();
    println!(
        "{} dew {hall:4} buildings {hold:2} nests {nests} {:?}\n      heroes {}",
        clock(w.t),
        kinds.iter().map(|(k, n)| format!("{}{}", &format!("{k:?}")[..3], n)).collect::<Vec<_>>(),
        heroes.join(" ")
    );
}
