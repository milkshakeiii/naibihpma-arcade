//! Tangles: several monsters on copied loops, stray goods and crowded rooms, run for a long
//! time. What's measured is the guarantee loops are built to: no looping monster waits forever
//! on another, so every one keeps finishing steps.

use super::*;
use crate::econ::{ItemData, RawMaterial};

/// A cotton good of some kind, as made from a boll.
fn cotton_good(g: &Game, id: &str) -> ItemData {
    let boll = g.data.good("cotton_bolls");
    let raw = vec![RawMaterial { good: boll.id.clone(), base_value: boll.raw_material_base_value.unwrap(), density: boll.raw_material_density.unwrap(), spent: false }];
    ItemData { good: id.into(), quality: 0.8, carried_tags: vec!["cotton".into()], raw, depth: 2, value: 2.0, shares: Vec::new(), wear: 0, haul: Vec::new(), carted: None }
}

/// The whole fabric chain as Henry recorded it in his world: harvest and bin two bolls, two
/// bolls from the chute through the workbench, the slivers to the spinnery, the thread to the
/// loom, the fabric to the cloth store, and home.
fn full_chain() -> Routine {
    let mv = |good: &str, from: (i32, i32), to: (i32, i32)| Act::Move { good: good.into(), from, to };
    let go = |to: (i32, i32), facing: Dir| Act::Go { to, facing };
    let acts = vec![
        go((8, 8), Dir::Right),
        Act::Use,
        mv("cotton_bolls", (9, 8), (12, 8)),
        mv("cotton_bolls", (9, 8), (12, 8)),
        mv("cotton_bolls", (12, 9), (13, 9)),
        go((14, 9), Dir::Left),
        Act::Use,
        mv("cotton_bolls", (12, 9), (13, 9)),
        go((14, 9), Dir::Left),
        Act::Use,
        mv("cotton_slivers", (15, 9), (30, 9)),
        go((30, 8), Dir::Right),
        Act::Use,
        mv("thread", (31, 9), (44, 21)),
        go((43, 21), Dir::Down),
        Act::Use,
        mv("fabric", (44, 22), (44, 28)),
        go((8, 10), Dir::Down),
    ];
    Routine { flag: (8, 10), acts, facing: Dir::Down }
}

/// `n` monsters, all looping `routine`, starting along the row below the flag.
fn on_one_loop(n: usize, routine: Routine) -> Game {
    let mut g = Game::new();
    let names = ["ISKA", "WREN", "LUMA", "BRAM"];
    while g.monsters.len() < n {
        let skills = g.monsters[0].skills.clone();
        let (id, name) = (10 + g.monsters.len() as u32, names[g.monsters.len() - 2]);
        g.monsters.push(Monster::new(id, name, "goblin", &g.data, (0, 0), skills));
    }
    for (k, m) in g.monsters.iter_mut().enumerate().take(n) {
        (m.x, m.y) = (6 + k as i32, 10);
        m.routine = Some(routine.clone());
        (m.pc, m.playing, m.bubble) = (0, true, None);
    }
    g
}

/// Runs `ticks` in windows of `window`: in every window, every looping monster finishes a step.
fn every_loop_keeps_going(g: &mut Game, ticks: u32, window: u32) {
    let mut last: Vec<u32> = g.monsters.iter().map(|m| m.steps).collect();
    for w in 1..=ticks / window {
        g.run_ticks(window);
        for (k, m) in g.monsters.iter().enumerate().filter(|(_, m)| m.playing) {
            assert!(m.steps > last[k], "{} stuck by tick {}: at ({},{}), step {}, {:?}", m.name, w * window, m.x, m.y, m.pc, m.wait_reason);
            last[k] = m.steps;
        }
    }
}

#[test]
fn three_copies_of_the_whole_chain_and_a_stray_thread_in_the_loom_house() {
    let mut g = on_one_loop(3, full_chain());
    // Left inside the loom house's north door by a loop that was stopped partway.
    let thread = cotton_good(&g, "thread");
    g.world.items.push(Item::new(44, 20, thread));
    let before = g.cloth_stored();
    every_loop_keeps_going(&mut g, 60_000, 3000);
    assert!(g.cloth_stored() >= before + 10, "{} fabric made", g.cloth_stored() - before);
}

#[test]
fn five_copies_of_the_whole_chain() {
    let mut g = on_one_loop(5, full_chain());
    let before = g.cloth_stored();
    every_loop_keeps_going(&mut g, 60_000, 3000);
    assert!(g.cloth_stored() >= before + 10, "{} fabric", g.cloth_stored() - before);
}

#[test]
fn copies_starting_at_different_steps_meet_head_on() {
    // Each starts somewhere else in the loop, so they cross paths going opposite ways.
    let mut g = on_one_loop(4, full_chain());
    for (k, (pc, at)) in [(0, (6, 10)), (10, (16, 9)), (13, (33, 9)), (16, (43, 22))].into_iter().enumerate() {
        let m = &mut g.monsters[k];
        (m.pc, m.x, m.y) = (pc, at.0, at.1);
    }
    every_loop_keeps_going(&mut g, 60_000, 3000);
}

#[test]
fn strays_in_every_doorway_on_the_way() {
    let mut g = on_one_loop(3, full_chain());
    // Left about by loops that were stopped: in the spinnery's doors and both of the loom house's.
    for (good, at) in [("cotton_slivers", (28, 9)), ("thread", (36, 9)), ("thread", (44, 19)), ("fabric", (44, 25))] {
        let data = cotton_good(&g, good);
        g.world.items.push(Item::new(at.0, at.1, data));
    }
    let before = g.cloth_stored();
    every_loop_keeps_going(&mut g, 60_000, 3000);
    assert!(g.cloth_stored() >= before + 10, "{} fabric", g.cloth_stored() - before);
}

#[test]
fn harvest_loops_and_whole_chains_together() {
    // Two only harvest and bin bolls; three run the whole chain off the same bin.
    let harvest = Routine { flag: (8, 10), acts: full_chain().acts[..4].to_vec(), facing: Dir::Down };
    let mut g = on_one_loop(5, full_chain());
    for k in 0..2 {
        g.monsters[k].routine = Some(harvest.clone());
    }
    let before = g.cloth_stored();
    every_loop_keeps_going(&mut g, 60_000, 3000);
    assert!(g.cloth_stored() >= before + 10, "{} fabric", g.cloth_stored() - before);
}

/// A long run, to catch rare tangles. Slow: run with `cargo test --release -- --ignored`.
#[test]
#[ignore]
fn five_copies_for_a_long_time() {
    let mut g = on_one_loop(5, full_chain());
    every_loop_keeps_going(&mut g, 300_000, 3000);
}

/// A tiny seeded generator, so random tangles are the same every run.
struct Lcg(u64);

impl Lcg {
    fn next(&mut self, n: usize) -> usize {
        self.0 = self.0.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
        ((self.0 >> 33) as usize) % n.max(1)
    }
}

/// A seeded random tangle: three to six monsters on the whole chain or part of it, starting
/// at random steps and spots, with strays dropped near the workshops and their doors.
fn random_tangle(seed: u64) -> Game {
    let mut r = Lcg(seed);
    let chain = full_chain();
    let n = 3 + r.next(4);
    let mut g = on_one_loop(n, chain.clone());
    let spots = [(10, 10), (16, 10), (20, 9), (26, 9), (33, 9), (40, 9), (44, 15), (45, 22), (42, 23), (35, 12), (12, 11), (8, 9)];
    for k in 0..n {
        // Some do only a stretch of the chain: the harvest, the middle, or the end.
        let acts = match r.next(4) {
            0 => chain.acts[..4].to_vec(),
            1 => chain.acts[4..13].to_vec(),
            2 => chain.acts[10..].to_vec(),
            _ => chain.acts.clone(),
        };
        let at = spots[r.next(spots.len())];
        let m = &mut g.monsters[k];
        m.routine = Some(Routine { flag: (8, 10), acts, facing: Dir::Down });
        (m.x, m.y) = at;
        m.pc = r.next(m.routine.as_ref().unwrap().acts.len());
    }
    // Monsters can't share a tile: nudge any that do.
    for k in 0..n {
        while (0..k).any(|o| (g.monsters[o].x, g.monsters[o].y) == (g.monsters[k].x, g.monsters[k].y)) {
            g.monsters[k].x += 1;
        }
    }
    let goods = ["cotton_bolls", "cotton_slivers", "thread", "fabric"];
    let near = [(28, 9), (36, 9), (44, 19), (44, 25), (29, 10), (41, 21), (45, 21), (14, 11), (24, 9), (44, 23), (31, 8), (40, 20)];
    for _ in 0..r.next(6) {
        let at = near[r.next(near.len())];
        if g.world.item_at(at.0, at.1).is_none() && g.monster_at(at.0, at.1).is_none() && g.world.item_floor(at.0, at.1) {
            let data = cotton_good(&g, goods[r.next(goods.len())]);
            g.world.items.push(Item::new(at.0, at.1, data));
        }
    }
    g
}

#[test]
fn forty_random_tangles() {
    // Random loops can lack a harvest, so some may have nothing ever come their way: that's
    // idle, not tangled. What must never happen is waiting on another monster for long.
    for seed in 1..=40 {
        let mut g = random_tangle(seed);
        let n = g.monsters.len();
        let (mut held_up, mut last) = (vec![0u32; n], g.monsters.iter().map(|m| m.steps).collect::<Vec<_>>());
        for t in 1..=30_000 {
            g.run_ticks(1);
            for (k, m) in g.monsters.iter().enumerate() {
                held_up[k] = if m.waiting_on.is_some() && m.steps == last[k] { held_up[k] + 1 } else { 0 };
                last[k] = m.steps;
                assert!(held_up[k] < 3000, "tangle {seed}: {} waited on {:?} from tick {} on: at ({},{}), step {}, {:?}", m.name, m.waiting_on.map(|o| g.monsters[o].name.clone()), t - 3000, m.x, m.y, m.pc, m.wait_reason);
            }
        }
    }
}

/// Not a check: prints how one random tangle goes (SEED picks it, 14 if unset). Run with
/// `cargo test --release trace_a_random_tangle -- --ignored --nocapture`.
#[test]
#[ignore]
fn trace_a_random_tangle() {
    let seed: u64 = std::env::var("SEED").ok().and_then(|s| s.parse().ok()).unwrap_or(14);
    let mut g = random_tangle(seed);
    for m in &g.monsters {
        let acts = &m.routine.as_ref().unwrap().acts;
        println!("{} at ({},{}) pc {} acts {}: {:?}", m.name, m.x, m.y, m.pc, acts.len(), acts.iter().map(|a| match a { Act::Move { good, from, to } => format!("M {good} {from:?}->{to:?}"), Act::Go { to, .. } => format!("G{to:?}"), Act::Use => "U".into(), _ => "?".into() }).collect::<Vec<_>>());
    }
    for it in &g.world.items {
        println!("item {} ({},{})", it.good(), it.x, it.y);
    }
    let until: u32 = std::env::var("UNTIL").ok().and_then(|s| s.parse().ok()).unwrap_or(30_000);
    for k in 1..=until / 3000 {
        g.run_ticks(3000);
        let row: Vec<String> = g.monsters.iter().map(|m| format!("{} s{} pc{} ({},{}) {:?}", m.name, m.steps, m.pc, m.x, m.y, m.wait_reason)).collect();
        println!("{}k: {} | cloth {}", k * 3, row.join(" | "), g.cloth_stored());
    }
    g.run_ticks(until % 3000);
    for m in &g.monsters {
        println!("{} ({},{}) task {:?} holds {:?} shove {:?} carry {:?} plan {} wait {} on {:?} cool {:?} busy {}", m.name, m.x, m.y, m.task, m.holds, m.shove, m.carry, m.carry_plan.len(), m.wait, m.waiting_on, m.cooldown, m.busy);
    }
    for it in &g.world.items {
        println!("item {} ({},{}) #{} locked {}", it.good(), it.x, it.y, it.id, it.locked);
    }
}
