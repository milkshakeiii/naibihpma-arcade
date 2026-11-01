//! Tangles: several monsters on copied loops, stray goods and crowded rooms, tunnels and dead
//! ends, run for a long time. What's measured is the guarantee loops are built to: no looping
//! monster waits forever on another, so every one keeps finishing steps, and nobody ever passes
//! through anybody to get there. Where there's no way at all, they say so.

use super::*;
use crate::econ::{ItemData, RawMaterial};

/// One tick, checking that nobody passed through anybody: no two things on one tile, and no
/// monster stepped onto a tile another left this same tick unless that one went on elsewhere.
/// Following is fine; two trading tiles, or a ring of them turning at once, is passing through.
pub(super) fn tick_checked(g: &mut Game) {
    let before: Vec<(i32, i32)> = g.monsters.iter().map(|m| (m.x, m.y)).collect();
    g.run_ticks(1);
    let after: Vec<(i32, i32)> = g.monsters.iter().map(|m| (m.x, m.y)).collect();
    let n = after.len();
    let name = |k: usize| g.monsters[k].name.clone();
    for a in 0..n {
        for b in a + 1..n {
            assert_ne!(after[a], after[b], "{} and {} on one tile at tick {}", name(a), name(b), g.frame);
        }
        assert!(g.world.item_at(after[a].0, after[a].1).is_none(), "{} on a good at {:?}, tick {}", name(a), after[a], g.frame);
    }
    for (k, it) in g.world.items.iter().enumerate() {
        assert_eq!(g.world.item_at(it.x, it.y), Some(k), "two goods on one tile at ({},{}), tick {}", it.x, it.y, g.frame);
    }
    if let Some(ring) = passing_through(&before, &after) {
        panic!("passed through each other at tick {}: {:?}", g.frame, ring.iter().map(|&k| (name(k), before[k], after[k])).collect::<Vec<_>>());
    }
}

/// Monsters that passed through each other between two ticks, if any: each stepped onto the tile
/// the next had left, round in a ring (two of them: they traded tiles).
fn passing_through(before: &[(i32, i32)], after: &[(i32, i32)]) -> Option<Vec<usize>> {
    let n = after.len();
    let onto = |a: usize| (0..n).find(|&b| b != a && after[a] != before[a] && before[b] == after[a]);
    for a in 0..n {
        let mut ring = vec![a];
        while let Some(b) = onto(ring[ring.len() - 1]).filter(|_| ring.len() <= n) {
            if b == a {
                return Some(ring);
            }
            ring.push(b);
        }
    }
    None
}

#[test]
fn the_check_catches_trading_tiles_and_rings_but_not_following() {
    let swap = passing_through(&[(1, 1), (2, 1)], &[(2, 1), (1, 1)]);
    assert_eq!(swap, Some(vec![0, 1]));
    let ring = [(1, 1), (2, 1), (2, 2), (1, 2)];
    let turned = [(2, 1), (2, 2), (1, 2), (1, 1)];
    assert_eq!(passing_through(&ring, &turned).map(|r| r.len()), Some(4), "four round a block turning at once");
    let train = passing_through(&[(1, 1), (2, 1), (3, 1)], &[(2, 1), (3, 1), (4, 1)]);
    assert_eq!(train, None, "following one another is fine");
}

/// `n` ticks of `tick_checked`.
pub(super) fn ticks_checked(g: &mut Game, n: u32) {
    for _ in 0..n {
        tick_checked(g);
    }
}

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

/// Runs `ticks` in windows of `window`: in every window, every looping monster finishes a step,
/// and nobody ever passes through anybody.
fn every_loop_keeps_going(g: &mut Game, ticks: u32, window: u32) {
    let mut last: Vec<u32> = g.monsters.iter().map(|m| m.steps).collect();
    for w in 1..=ticks / window {
        ticks_checked(g, window);
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
            tick_checked(&mut g);
            for (k, m) in g.monsters.iter().enumerate() {
                held_up[k] = if m.waiting_on.is_some() && m.steps == last[k] { held_up[k] + 1 } else { 0 };
                last[k] = m.steps;
                assert!(held_up[k] < 3000, "tangle {seed}: {} waited on {:?} from tick {} on: at ({},{}), step {}, {:?}", m.name, m.waiting_on.map(|o| g.monsters[o].name.clone()), t - 3000, m.x, m.y, m.pc, m.wait_reason);
            }
        }
    }
}

/// Where test grounds are laid: the open meadow below the workshops.
const GROUND: (i32, i32) = (16, 12);

/// A ground's tile on the map.
fn on_ground((x, y): (i32, i32)) -> (i32, i32) {
    (GROUND.0 + x, GROUND.1 + y)
}

/// Monsters on walking loops on a test ground laid over the meadow ('#' is hedge, anything else
/// grass). Each starts at the first of its stops and walks to each in turn, round and round.
fn walkers(rows: &[&str], loops: &[&[(i32, i32)]]) -> Game {
    let mut g = on_one_loop(loops.len(), full_chain());
    let w = g.world.w;
    for (y, row) in rows.iter().enumerate() {
        for (x, c) in row.chars().enumerate() {
            let (tx, ty) = on_ground((x as i32, y as i32));
            g.world.tiles[(ty * w + tx) as usize] = if c == '#' { Tile::Hedge } else { Tile::Grass };
        }
    }
    for (k, stops) in loops.iter().enumerate() {
        let acts = stops[1..].iter().chain(&stops[..1]).map(|&s| Act::Go { to: on_ground(s), facing: Dir::Down }).collect();
        let m = &mut g.monsters[k];
        (m.x, m.y) = on_ground(stops[0]);
        m.routine = Some(Routine { flag: on_ground(stops[0]), acts, facing: Dir::Down });
    }
    g
}

/// Two rooms joined by an eight-tile tunnel one tile wide.
const TUNNEL: [&str; 7] = ["##################", "#....########....#", "#....########....#", "#................#", "#....########....#", "#....########....#", "##################"];

#[test]
fn two_loops_meet_head_on_in_a_long_one_tile_tunnel() {
    let mut g = walkers(&TUNNEL, &[&[(1, 1), (16, 5)], &[(16, 1), (1, 5)]]);
    every_loop_keeps_going(&mut g, 9000, 600);
}

#[test]
fn four_loops_cross_both_ways_through_a_one_tile_tunnel() {
    let mut g = walkers(&TUNNEL, &[&[(1, 1), (16, 5)], &[(16, 1), (1, 5)], &[(2, 5), (15, 1)], &[(15, 5), (2, 1)]]);
    every_loop_keeps_going(&mut g, 12_000, 1200);
}

#[test]
fn two_loops_share_a_dead_end_tunnel_and_back_out_for_each_other() {
    // Both walk to the end of a dead end four tiles deep. Whoever is coming in when the other
    // wants out goes first, so the one inside can't just step aside: the one coming in backs
    // out into the room, the other comes out past it, and it goes in again.
    let dead_end = ["##########", "#........#", "#........#", "#........#", "####.#####", "####.#####", "####.#####", "####.#####", "##########"];
    let mut g = walkers(&dead_end, &[&[(1, 1), (4, 7)], &[(8, 3), (4, 7)]]);
    let mut untangled = 0;
    let mut last: Vec<u32> = g.monsters.iter().map(|m| m.steps).collect();
    for w in 1..=15 {
        for _ in 0..600 {
            tick_checked(&mut g);
            untangled += g.monsters.iter().any(|m| m.untangle.is_some()) as u32;
        }
        for (k, m) in g.monsters.iter().enumerate().filter(|(_, m)| m.playing) {
            assert!(m.steps > last[k], "{} stuck by tick {}: at ({},{}), {:?}", m.name, w * 600, m.x, m.y, m.wait_reason);
            last[k] = m.steps;
        }
    }
    assert!(untangled > 0, "backing out of the dead end took moves searched for");
}

#[test]
fn loops_cross_between_two_rooms_through_one_doorway() {
    let rooms = ["###############", "#.....#.......#", "#.....#.......#", "#.............#", "#.....#.......#", "#.....#.......#", "###############"];
    let mut g = walkers(&rooms, &[&[(1, 1), (13, 5)], &[(13, 1), (1, 5)], &[(2, 5), (12, 1)], &[(12, 5), (2, 1)]]);
    every_loop_keeps_going(&mut g, 12_000, 1200);
}

#[test]
fn loops_in_and_out_of_a_dead_end_room_behind_a_one_tile_doorway() {
    let room = ["############", "#..........#", "#..........#", "#####.######", "#####.######", "###.....####", "###.....####", "###.....####", "############"];
    let mut g = walkers(&room, &[&[(1, 1), (3, 7)], &[(10, 1), (7, 7)], &[(10, 2), (5, 6)]]);
    every_loop_keeps_going(&mut g, 12_000, 1200);
}

#[test]
fn a_good_pushed_to_and_fro_through_a_tunnel_and_a_loop_walking_the_other_way() {
    let ground = ["################", "#.....####.....#", "#..............#", "#.....####.....#", "################"];
    let mut g = walkers(&ground, &[&[(14, 1), (1, 3)], &[(1, 1), (14, 3)]]);
    let boll = cotton_good(&g, "cotton_bolls");
    let (a, b) = (on_ground((2, 2)), on_ground((13, 2)));
    g.world.items.push(Item::new(a.0, a.1, boll));
    let mv = |from, to| Act::Move { good: "cotton_bolls".into(), from, to };
    g.monsters[1].routine = Some(Routine { flag: on_ground((1, 1)), acts: vec![mv(a, b), mv(b, a)], facing: Dir::Down });
    every_loop_keeps_going(&mut g, 12_000, 1500);
}

#[test]
fn two_loops_in_a_closed_one_tile_tunnel_say_they_need_more_room() {
    // Each wants the far end: with no way past without passing through, it can't be done.
    let mut g = walkers(&["######", "#....#", "######"], &[&[(1, 1), (4, 1)], &[(4, 1), (1, 1)]]);
    ticks_checked(&mut g, 600);
    for m in &g.monsters[..2] {
        assert_eq!(m.wait_reason, crate::tangle::NO_ROOM, "{} at ({},{})", m.name, m.x, m.y);
    }
}

#[test]
fn a_step_that_cant_be_done_sends_its_monster_to_the_back_of_the_line() {
    // Mogg's boll is meant to go into a hedge, so each time Mogg takes the step up it's given
    // up. Bruna, on a long walk, has finished nothing yet either. Mogg came first in line;
    // having given its step up, it comes after her: otherwise, taking a step up ahead of her
    // each time and giving it up again, it could push her back for ever with nothing done.
    let mut g = walkers(&TUNNEL, &[&[(1, 1), (5, 4)], &[(16, 1), (1, 5)]]);
    let (boll, at, hedge) = (cotton_good(&g, "cotton_bolls"), on_ground((2, 3)), on_ground((7, 1)));
    g.world.items.push(Item::new(at.0, at.1, boll));
    g.monsters[0].routine = Some(Routine { flag: on_ground((1, 1)), acts: vec![Act::Move { good: "cotton_bolls".into(), from: at, to: hedge }], facing: Dir::Down });
    assert!(g.rank(0).1 <= g.rank(1).1, "Mogg starts first in line");
    tick_checked(&mut g);
    tick_checked(&mut g);
    let (mogg, bruna) = (&g.monsters[0], &g.monsters[1]);
    assert!(!mogg.cooldown.is_empty() && mogg.steps == 0, "the step was given up: {:?}", mogg.wait_reason);
    assert!(bruna.task.is_some() && bruna.steps == 0, "Bruna is still on her way");
    assert!(mogg.since > bruna.since, "Mogg, taking a step up again, comes after Bruna");
}

/// A ring one tile wide round a block, opening into a room along the bottom.
const RING: [&str; 8] = ["###########", "#.........#", "#.#######.#", "#.#######.#", "#.........#", "#.........#", "#.........#", "###########"];

/// Monster `k` leads the town wagon on a loop of steps played as recorded, from `at`: the
/// pumpkin behind it at `pumpkin`, the wagon behind that at `bed`.
fn leading(g: &mut Game, k: usize, w: usize, (at, pumpkin, bed): ((i32, i32), (i32, i32), (i32, i32)), steps: &[(Dir, usize)]) {
    let acts = steps.iter().flat_map(|&(d, n)| std::iter::repeat_n(Act::Step(d, None), n)).collect();
    let id = g.monsters[k].id;
    while g.world.wagons.len() <= w {
        let copy = g.world.wagons[0].clone();
        g.world.wagons.push(copy);
    }
    let wagon = &mut g.world.wagons[w];
    (wagon.pumpkin, wagon.at, wagon.leader, wagon.last_leader) = (on_ground(pumpkin), on_ground(bed), Some(id), Some(id));
    let m = &mut g.monsters[k];
    (m.x, m.y) = on_ground(at);
    m.routine = Some(Routine { flag: on_ground(at), acts, facing: Dir::Up });
}

#[test]
fn a_loop_waiting_in_a_wagons_way_round_a_one_tile_ring_makes_way_for_it() {
    // Bruna leads the wagon round the ring as recorded: it can neither step aside nor go round.
    // Mogg waits in the top of the ring, at a Space that finds nothing to use there ("WAITING
    // FOR THE WAGON."), first in line by roster order. Whoever can't step aside goes first, so
    // Mogg backs round ahead of the wagon into the room, lets it by, and goes back to wait.
    let mut g = walkers(&RING, &[&[(5, 1), (5, 1)], &[(1, 4), (1, 4)]]);
    g.monsters[0].routine = Some(Routine { flag: on_ground((5, 1)), acts: vec![Act::Go { to: on_ground((5, 1)), facing: Dir::Down }, Act::Use], facing: Dir::Down });
    leading(&mut g, 1, 0, ((1, 4), (2, 4), (3, 4)), &[(Dir::Up, 3), (Dir::Right, 8), (Dir::Down, 3), (Dir::Left, 8)]);
    ticks_checked(&mut g, 3000);
    let (mogg, bruna) = (&g.monsters[0], &g.monsters[1]);
    assert!(bruna.loops >= 10, "the wagon went round {} times: Mogg at ({},{}), {:?}", bruna.loops, mogg.x, mogg.y, bruna.wait_reason);
    assert_eq!(mogg.wait_reason, "WAITING FOR THE WAGON.", "and Mogg went back to wait");
}

#[test]
fn a_wagon_gets_by_four_loops_waiting_in_a_row_in_its_way() {
    // Too many in a row to make way one for the next, so moves are searched for that clear
    // the wagon's next tile without it moving: the far one backs off first, then the rest.
    let waits = [(4, 1), (5, 1), (6, 1), (7, 1)];
    let mut g = walkers(&RING, &[&[(4, 1), (4, 1)], &[(5, 1), (5, 1)], &[(6, 1), (6, 1)], &[(7, 1), (7, 1)], &[(1, 4), (1, 4)]]);
    for (k, &at) in waits.iter().enumerate() {
        g.monsters[k].routine = Some(Routine { flag: on_ground(at), acts: vec![Act::Go { to: on_ground(at), facing: Dir::Down }, Act::Use], facing: Dir::Down });
    }
    leading(&mut g, 4, 0, ((1, 4), (2, 4), (3, 4)), &[(Dir::Up, 3), (Dir::Right, 8), (Dir::Down, 3), (Dir::Left, 8)]);
    let mut untangled = 0;
    for _ in 0..6000 {
        tick_checked(&mut g);
        untangled += g.monsters[4].untangle.is_some() as u32;
    }
    assert!(g.monsters[4].loops >= 3, "the wagon went round {} times: {:?}", g.monsters[4].loops, g.monsters[4].wait_reason);
    assert!(untangled > 0, "the way was cleared by moves searched for");
}

#[test]
fn two_wagons_meeting_head_on_in_a_one_tile_lane_say_they_need_more_room() {
    // Neither can step aside or go round: there's no way past, so they say so.
    let mut g = walkers(&["##############", "#............#", "##############"], &[&[(3, 1), (3, 1)], &[(10, 1), (10, 1)]]);
    leading(&mut g, 0, 0, ((3, 1), (2, 1), (1, 1)), &[(Dir::Right, 6)]);
    leading(&mut g, 1, 1, ((10, 1), (11, 1), (12, 1)), &[(Dir::Left, 6)]);
    ticks_checked(&mut g, 300);
    for m in &g.monsters[..2] {
        assert_eq!(m.wait_reason, crate::tangle::NO_ROOM, "{} at ({},{})", m.name, m.x, m.y);
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
