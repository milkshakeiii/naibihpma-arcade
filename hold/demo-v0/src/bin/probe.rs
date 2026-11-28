//! Balance probe: a scripted Elder plays the lesson, then about nine minutes; prints the village
//! every 30 s. `cargo run --release --bin probe [seed]`
use hold::bot::elder;
use hold::sim::*;

fn main() {
    let seed: u64 = std::env::args()
        .nth(1)
        .and_then(|s| s.parse().ok())
        .unwrap_or(6);
    let quiet = std::env::args().any(|a| a == "-q");
    let mut w = World::new(seed);
    let mut counts = std::collections::BTreeMap::<String, i32>::new();
    let mut downs = vec![];
    let deaths = std::env::args().any(|a| a == "-d");
    for t in 0..15 * 60 * 10 {
        elder(&mut w, t);
        let before: Vec<(u32, String)> = w
            .units
            .iter()
            .filter(|u| u.kind.hero())
            .map(|u| {
                (
                    u.id,
                    format!(
                        "{} {:?} {:?} L{} at ({},{}) job {:?} herons near {}",
                        u.name,
                        u.kind,
                        u.trait_,
                        u.level,
                        u.x / 16,
                        u.y / 16,
                        u.job,
                        w.units
                            .iter()
                            .filter(|h| h.kind == Kind::Heron
                                && (h.x - u.x).abs() < 40
                                && (h.y - u.y).abs() < 40)
                            .count()
                    ),
                )
            })
            .collect();
        w.step();
        if deaths {
            for e in &w.events {
                if let Ev::Fell { id } = e {
                    let d = before
                        .iter()
                        .find(|b| b.0 == *id)
                        .map(|b| b.1.clone())
                        .unwrap_or_default();
                    println!("{}:{:02} FELL {}", t / 900, t / 15 % 60, d);
                }
            }
        }
        for e in w.events.drain(..) {
            if let Ev::Down { kind, .. } = &e {
                downs.push((t / 15, *kind));
            }
            let k = format!("{:?}", e)
                .split([' ', '{', '('])
                .next()
                .unwrap()
                .to_string();
            *counts.entry(k).or_default() += 1;
        }
        if !quiet && (t % 450 == 0 || w.over.is_some()) {
            println!(
                "--- {}:{:02}  dew {} reserved {}  nests {}  calm {}",
                t / 900,
                t / 15 % 60,
                w.dew,
                w.reserved(),
                w.nests_left(),
                w.calm
            );
            for f in &w.flags {
                println!(
                    "  flag {:?} reward {} takers {:?} hitters {:?} danger {}",
                    f.tgt,
                    f.reward,
                    f.takers,
                    f.hitters,
                    w.danger(f.tgt)
                );
            }
            for u in w.units.iter().filter(|u| u.kind.hero()) {
                let ask: Vec<i32> = w.flags.iter().map(|f| w.asking(u, f)).collect();
                println!(
                    "  {:8} {:?} {:?} L{} hp {}/{} purse {} job {:?} asks {:?}",
                    u.name, u.kind, u.trait_, u.level, u.hp, u.max_hp, u.purse, u.job, ask
                );
            }
            let herons = w.units.iter().filter(|u| u.kind == Kind::Heron).count();
            println!(
                "  herons {}  enemy hp {:?}",
                herons,
                w.bldgs
                    .iter()
                    .filter(|b| b.alive && b.kind.enemy())
                    .map(|b| (b.kind, b.hp))
                    .collect::<Vec<_>>()
            );
        }
        if w.over.is_some() {
            break;
        }
    }
    println!(
        "seed {seed}: over {:?} at {}:{:02}, nests left {}, fallen {}, paid {}",
        w.over,
        w.tick / 900,
        w.tick / 15 % 60,
        w.nests_left(),
        w.fallen,
        w.paid_total
    );
    println!("  downs {:?}", downs);
    println!("  {:?}", counts);
}
