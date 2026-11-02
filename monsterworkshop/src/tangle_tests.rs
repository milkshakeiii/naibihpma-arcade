//! Tangles: several monsters on copied loops, stray goods and crowded rooms, tunnels and dead
//! ends, run for a long time. What's measured is what loops are built to: no looping monster
//! waits forever on another, so every one keeps finishing goals, and nobody ever passes through
//! anybody to get there.
//!
//! These were the yardstick the movement principles in `motion.rs` were built against, one at a
//! time; they all pass now. The ignored soaks run hundreds more random tangles and random maps:
//! `cargo test --release -- --ignored` (SEEDS=from..to picks the seeds).

use super::*;
use crate::econ::{ItemData, RawMaterial};
use crate::motion::Going;

/// One step of the world, checking that nobody passed through anybody: no two things on one
/// tile, and no monster stepped onto a tile another left this same tick unless that one went
/// on elsewhere. Following is fine; two trading tiles, or a ring of them turning at once, is
/// passing through.
fn checked(g: &mut Game, step: impl FnOnce(&mut Game)) {
    let before: Vec<(i32, i32)> = g.monsters.iter().map(|m| (m.x, m.y)).collect();
    step(g);
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

/// One tick, checked.
pub(super) fn tick_checked(g: &mut Game) {
    checked(g, |g| g.run_ticks(1));
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

/// Monsters walking between stops by themselves, round and round: movement alone, with no loop
/// (a loop's goals each bring their own walk). Monster `k` walks `stops[k]` in turn; one with
/// no stops is left to its loop, or to stand still. Walkers count as looping, so they're
/// standing still only while waiting, like a loop.
pub(super) struct Walkers {
    stops: Vec<Vec<(i32, i32)>>,
    next: Vec<usize>,
}

impl Walkers {
    pub(super) fn new(stops: Vec<Vec<(i32, i32)>>) -> Self {
        let next = vec![0; stops.len()];
        Walkers { stops, next }
    }

    fn none() -> Self {
        Walkers::new(Vec::new())
    }

    fn walking(&self, k: usize) -> bool {
        self.stops.get(k).is_some_and(|s| !s.is_empty())
    }

    /// One tick: each walker that isn't mid-step steps toward its next stop (reaching it counts
    /// as a goal done), then the world goes on.
    fn tick(&mut self, g: &mut Game) {
        let mut sfx = Vec::new();
        for (k, stops) in self.stops.iter().enumerate() {
            if stops.is_empty() || !g.monsters[k].idle() || g.in_knot(k) {
                continue;
            }
            g.monsters[k].waiting_on = None;
            let went = if g.monsters[k].giving_way.is_some() { g.give_way_tick(k, &mut sfx) } else { g.go_toward(k, stops[self.next[k]], None, &mut sfx) };
            let frame = g.frame;
            let m = &mut g.monsters[k];
            match went {
                Going::Arrived => {
                    self.next[k] = (self.next[k] + 1) % stops.len();
                    (m.steps, m.since) = (m.steps + 1, frame);
                    (m.wait, m.wait_reason) = (0, String::new());
                }
                Going::Waiting(msg) | Going::Stuck(msg) => {
                    m.wait += 1;
                    m.wait_reason = msg;
                }
                Going::Moving => (m.wait, m.wait_reason) = (0, String::new()),
            }
        }
        g.run_ticks(1);
    }

    /// `n` ticks, checked.
    pub(super) fn run(&mut self, g: &mut Game, n: u32) {
        for _ in 0..n {
            checked(g, |g| self.tick(g));
        }
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
/// loom, the fabric to the cloth store.
pub(super) fn full_chain(g: &Game) -> Routine {
    let mv = |good: &str, from: (i32, i32), to: (i32, i32)| Goal::Move { good: good.into(), from, to };
    let work = |spot: (i32, i32)| Goal::Activate(Thing::Shop(g.world.shop_at_spot(spot.0, spot.1).expect("a work spot")));
    let goals = vec![
        Goal::Activate(Thing::Plant { at: (8, 8), facing: Dir::Right }),
        mv("cotton_bolls", (9, 8), (12, 8)),
        mv("cotton_bolls", (9, 8), (12, 8)),
        mv("cotton_bolls", (12, 9), (13, 9)),
        work((14, 9)),
        mv("cotton_bolls", (12, 9), (13, 9)),
        work((14, 9)),
        mv("cotton_slivers", (15, 9), (30, 9)),
        work((30, 8)),
        mv("thread", (31, 9), (44, 21)),
        work((43, 21)),
        mv("fabric", (44, 22), (44, 28)),
    ];
    Routine { flag: (8, 10), goals }
}

/// `n` monsters, all looping the routine `make` gives, starting along the row below the flag.
fn on_one_loop(n: usize, make: fn(&Game) -> Routine) -> Game {
    let mut g = Game::new();
    let names = ["ISKA", "WREN", "LUMA", "BRAM"];
    while g.monsters.len() < n {
        let skills = g.monsters[0].skills.clone();
        let (id, name) = (10 + g.monsters.len() as u32, names[g.monsters.len() - 2]);
        g.monsters.push(Monster::new(id, name, "goblin", &g.data, (0, 0), skills));
    }
    let routine = make(&g);
    for (k, m) in g.monsters.iter_mut().enumerate().take(n) {
        (m.x, m.y) = (6 + k as i32, 10);
        m.routine = Some(routine.clone());
        (m.pc, m.playing, m.bubble) = (0, true, None);
    }
    g
}

/// Runs `ticks` in windows of `window`: in every window, every looping monster and every walker
/// finishes a goal, and nobody ever passes through anybody.
fn keeps_going(g: &mut Game, walk: &mut Walkers, ticks: u32, window: u32) {
    let mut last: Vec<u32> = g.monsters.iter().map(|m| m.steps).collect();
    for w in 1..=ticks / window {
        walk.run(g, window);
        for (k, m) in g.monsters.iter().enumerate().filter(|&(k, m)| m.playing || walk.walking(k)) {
            assert!(m.steps > last[k], "{} stuck by tick {}: at ({},{}), goal {}, {:?}", m.name, w * window, m.x, m.y, m.pc, m.wait_reason);
            last[k] = m.steps;
        }
    }
}

fn every_loop_keeps_going(g: &mut Game, ticks: u32, window: u32) {
    keeps_going(g, &mut Walkers::none(), ticks, window);
}

#[test]
fn three_copies_of_the_whole_chain_and_a_stray_thread_in_the_loom_house() {
    let mut g = on_one_loop(3, full_chain);
    // Left inside the loom house's north door by a loop that was stopped partway.
    let thread = cotton_good(&g, "thread");
    g.world.items.push(Item::new(44, 20, thread));
    let before = g.cloth_stored();
    every_loop_keeps_going(&mut g, 60_000, 3000);
    assert!(g.cloth_stored() >= before + 10, "{} fabric made", g.cloth_stored() - before);
}

#[test]
fn five_copies_of_the_whole_chain() {
    let mut g = on_one_loop(5, full_chain);
    let before = g.cloth_stored();
    every_loop_keeps_going(&mut g, 60_000, 3000);
    assert!(g.cloth_stored() >= before + 10, "{} fabric", g.cloth_stored() - before);
}

#[test]
fn copies_starting_at_different_goals_meet_head_on() {
    // Each starts somewhere else in the loop, so they cross paths going opposite ways.
    let mut g = on_one_loop(4, full_chain);
    for (k, (pc, at)) in [(0, (6, 10)), (7, (16, 9)), (9, (33, 9)), (11, (43, 22))].into_iter().enumerate() {
        let m = &mut g.monsters[k];
        (m.pc, m.x, m.y) = (pc, at.0, at.1);
    }
    every_loop_keeps_going(&mut g, 60_000, 3000);
}

#[test]
fn strays_in_every_doorway_on_the_way() {
    let mut g = on_one_loop(3, full_chain);
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
    let mut g = on_one_loop(5, full_chain);
    let harvest = Routine { flag: (8, 10), goals: full_chain(&g).goals[..3].to_vec() };
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
    let mut g = on_one_loop(5, full_chain);
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
/// at random goals and spots, with strays dropped near the workshops and their doors.
fn random_tangle(seed: u64) -> Game {
    let mut r = Lcg(seed);
    let n = 3 + r.next(4);
    let mut g = on_one_loop(n, full_chain);
    let chain = full_chain(&g);
    let spots = [(10, 10), (16, 10), (20, 9), (26, 9), (33, 9), (40, 9), (44, 15), (45, 22), (42, 23), (35, 12), (12, 11), (8, 9)];
    for k in 0..n {
        // Some do only a stretch of the chain: the harvest, the middle, or the end.
        let goals = match r.next(4) {
            0 => chain.goals[..3].to_vec(),
            1 => chain.goals[3..9].to_vec(),
            2 => chain.goals[7..].to_vec(),
            _ => chain.goals.clone(),
        };
        let at = spots[r.next(spots.len())];
        let m = &mut g.monsters[k];
        m.routine = Some(Routine { flag: (8, 10), goals });
        (m.x, m.y) = at;
        m.pc = r.next(m.routine.as_ref().unwrap().goals.len());
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

/// Random tangle `seed` for 30,000 ticks. Random loops can lack a harvest, so some may have
/// nothing ever come their way: that's idle, not tangled. What must never happen is waiting on
/// another monster for long.
fn no_one_held_up_for_long(seed: u64) {
    let mut g = random_tangle(seed);
    let n = g.monsters.len();
    let (mut held_up, mut last) = (vec![0u32; n], g.monsters.iter().map(|m| m.steps).collect::<Vec<_>>());
    for t in 1..=30_000 {
        tick_checked(&mut g);
        for (k, m) in g.monsters.iter().enumerate() {
            held_up[k] = if m.waiting_on.is_some() && m.steps == last[k] { held_up[k] + 1 } else { 0 };
            last[k] = m.steps;
            assert!(held_up[k] < 3000, "tangle {seed}: {} waited on {:?} from tick {} on: at ({},{}), goal {}, {:?}", m.name, m.waiting_on.map(|o| g.monsters[o].name.clone()), t - 3000, m.x, m.y, m.pc, m.wait_reason);
        }
    }
}

#[test]
fn forty_random_tangles() {
    for seed in 1..=40 {
        no_one_held_up_for_long(seed);
    }
}

/// The seeds in `SEEDS` ("from..to", to not included), or `default`.
fn seeds(default: std::ops::Range<u64>) -> std::ops::Range<u64> {
    let var = std::env::var("SEEDS").ok();
    let parsed = var.as_deref().and_then(|s| s.split_once("..")).and_then(|(a, b)| Some(a.parse().ok()?..b.parse().ok()?));
    parsed.unwrap_or(default)
}

/// Seeds that fail `check`, each failure's message printed as it happens.
fn failing(range: std::ops::Range<u64>, check: fn(u64)) -> Vec<u64> {
    range
        .filter(|&seed| {
            eprintln!("seed {seed}");
            std::panic::catch_unwind(|| check(seed)).is_err()
        })
        .collect()
}

#[test]
#[ignore]
fn many_random_tangles() {
    let failed = failing(seeds(41..441), no_one_held_up_for_long);
    assert!(failed.is_empty(), "tangles that failed: {failed:?}");
}

/// Where test grounds are laid: the open meadow below the workshops.
const GROUND: (i32, i32) = (16, 12);

/// A ground's tile on the map.
fn on_ground((x, y): (i32, i32)) -> (i32, i32) {
    (GROUND.0 + x, GROUND.1 + y)
}

/// Walkers on a test ground laid over the meadow ('#' is hedge, anything else grass). Each
/// starts at the first of its stops and walks to each in turn, round and round.
fn walkers(rows: &[&str], stops: &[&[(i32, i32)]]) -> (Game, Walkers) {
    let mut g = Game::new();
    while g.monsters.len() < stops.len() {
        let skills = g.monsters[0].skills.clone();
        let id = 10 + g.monsters.len() as u32;
        g.monsters.push(Monster::new(id, "WALKER", "goblin", &g.data, (0, 0), skills));
    }
    let w = g.world.w;
    for (y, row) in rows.iter().enumerate() {
        for (x, c) in row.chars().enumerate() {
            let (tx, ty) = on_ground((x as i32, y as i32));
            g.world.tiles[(ty * w + tx) as usize] = if c == '#' { Tile::Hedge } else { Tile::Grass };
        }
    }
    let mut all = vec![Vec::new(); g.monsters.len()];
    for (k, s) in stops.iter().enumerate() {
        (g.monsters[k].x, g.monsters[k].y) = on_ground(s[0]);
        g.monsters[k].playing = true;
        all[k] = s[1..].iter().chain(&s[..1]).map(|&t| on_ground(t)).collect();
    }
    // Anyone not walking stands well out of the way.
    for k in stops.len()..g.monsters.len() {
        (g.monsters[k].x, g.monsters[k].y) = (2 + k as i32, 30);
    }
    (g, Walkers::new(all))
}

/// Two rooms joined by an eight-tile tunnel one tile wide.
const TUNNEL: [&str; 7] = ["##################", "#....########....#", "#....########....#", "#................#", "#....########....#", "#....########....#", "##################"];

#[test]
fn a_walker_goes_around_a_monster_standing_in_its_way() {
    let (mut g, mut walk) = walkers(&["############", "#..........#", "#..........#", "#..........#", "############"], &[&[(1, 2), (10, 2)]]);
    let bruna = g.monsters.len() - 1;
    (g.monsters[bruna].x, g.monsters[bruna].y) = on_ground((5, 2)); // stands still on the way
    keeps_going(&mut g, &mut walk, 3000, 300);
    assert_eq!((g.monsters[bruna].x, g.monsters[bruna].y), on_ground((5, 2)), "nobody shoved Bruna aside");
}

#[test]
fn two_walkers_meet_head_on_in_a_long_one_tile_tunnel() {
    let (mut g, mut walk) = walkers(&TUNNEL, &[&[(1, 1), (16, 5)], &[(16, 1), (1, 5)]]);
    keeps_going(&mut g, &mut walk, 9000, 600);
}

#[test]
fn four_walkers_cross_both_ways_through_a_one_tile_tunnel() {
    let (mut g, mut walk) = walkers(&TUNNEL, &[&[(1, 1), (16, 5)], &[(16, 1), (1, 5)], &[(2, 5), (15, 1)], &[(15, 5), (2, 1)]]);
    keeps_going(&mut g, &mut walk, 12_000, 1200);
}

#[test]
fn two_walkers_share_a_dead_end_tunnel_and_back_out_for_each_other() {
    // Both walk to the end of a dead end four tiles deep: whoever is coming in when the other
    // wants out has to back out into the room and let it by.
    let dead_end = ["##########", "#........#", "#........#", "#........#", "####.#####", "####.#####", "####.#####", "####.#####", "##########"];
    let (mut g, mut walk) = walkers(&dead_end, &[&[(1, 1), (4, 7)], &[(8, 3), (4, 7)]]);
    keeps_going(&mut g, &mut walk, 9000, 600);
}

#[test]
fn walkers_cross_between_two_rooms_through_one_doorway() {
    let rooms = ["###############", "#.....#.......#", "#.....#.......#", "#.............#", "#.....#.......#", "#.....#.......#", "###############"];
    let (mut g, mut walk) = walkers(&rooms, &[&[(1, 1), (13, 5)], &[(13, 1), (1, 5)], &[(2, 5), (12, 1)], &[(12, 5), (2, 1)]]);
    keeps_going(&mut g, &mut walk, 12_000, 1200);
}

#[test]
fn walkers_in_and_out_of_a_dead_end_room_behind_a_one_tile_doorway() {
    let room = ["############", "#..........#", "#..........#", "#####.######", "#####.######", "###.....####", "###.....####", "###.....####", "############"];
    let (mut g, mut walk) = walkers(&room, &[&[(1, 1), (3, 7)], &[(10, 1), (7, 7)], &[(10, 2), (5, 6)]]);
    keeps_going(&mut g, &mut walk, 12_000, 1200);
}

#[test]
fn a_good_pushed_to_and_fro_through_a_tunnel_and_a_walker_going_the_other_way() {
    let ground = ["################", "#.....####.....#", "#..............#", "#.....####.....#", "################"];
    let (mut g, mut walk) = walkers(&ground, &[&[(14, 1), (1, 3)], &[(1, 1)]]);
    walk.stops[1].clear(); // Bruna loops instead
    let boll = cotton_good(&g, "cotton_bolls");
    let (a, b) = (on_ground((2, 2)), on_ground((13, 2)));
    g.world.items.push(Item::new(a.0, a.1, boll));
    let mv = |from, to| Goal::Move { good: "cotton_bolls".into(), from, to };
    let m = &mut g.monsters[1];
    m.routine = Some(Routine { flag: on_ground((1, 1)), goals: vec![mv(a, b), mv(b, a)] });
    m.playing = true;
    keeps_going(&mut g, &mut walk, 12_000, 1500);
}

#[test]
fn two_walkers_in_a_closed_one_tile_tunnel_say_they_need_more_room() {
    // Each wants the far end: with no way past without passing through, it can't be done, and
    // they should say so rather than wait for ever.
    let (mut g, mut walk) = walkers(&["######", "#....#", "######"], &[&[(1, 1), (4, 1)], &[(4, 1), (1, 1)]]);
    walk.run(&mut g, 600);
    for m in &g.monsters[..2] {
        assert_eq!(m.wait_reason, "WE NEED MORE ROOM HERE.", "{} at ({},{})", m.name, m.x, m.y);
    }
}

/// A seeded random ground: rooms joined by one-tile corridors, dead-end spurs, two to five
/// monsters walking between random stops, and a few strays lying about. Returns the ground's
/// rows too, for printing.
fn random_ground(seed: u64) -> (Game, Walkers, Vec<String>) {
    let mut r = Lcg(seed);
    let (w, h) = (18, 9);
    let mut open = vec![vec![false; w]; h];
    let mut centres = Vec::new();
    for _ in 0..2 + r.next(3) {
        let (rw, rh) = (2 + r.next(4), 1 + r.next(3));
        let (x0, y0) = (1 + r.next(w - 1 - rw), 1 + r.next(h - 1 - rh));
        for row in open.iter_mut().skip(y0).take(rh) {
            row[x0..x0 + rw].fill(true);
        }
        centres.push((x0 + r.next(rw), y0 + r.next(rh)));
    }
    // Each room joined to the one before by a corridor, across then down or down then across.
    for k in 1..centres.len() {
        let ((ax, ay), (bx, by)) = (centres[k - 1], centres[k]);
        let corner = if r.next(2) == 0 { (bx, ay) } else { (ax, by) };
        for (p, q) in [((ax, ay), corner), (corner, (bx, by))] {
            for row in open.iter_mut().take(p.1.max(q.1) + 1).skip(p.1.min(q.1)) {
                row[p.0.min(q.0)..=p.0.max(q.0)].fill(true);
            }
        }
    }
    for _ in 0..r.next(4) {
        let (mut x, mut y) = (1 + r.next(w - 2), 1 + r.next(h - 2));
        let d = DIRS[r.next(4)].delta();
        if !open[y][x] {
            continue;
        }
        for _ in 0..1 + r.next(4) {
            let (nx, ny) = (x as i32 + d.0, y as i32 + d.1);
            if nx < 1 || ny < 1 || nx >= w as i32 - 1 || ny >= h as i32 - 1 {
                break;
            }
            (x, y) = (nx as usize, ny as usize);
            open[y][x] = true;
        }
    }
    let rows: Vec<String> = open.iter().map(|row| row.iter().map(|&o| if o { '.' } else { '#' }).collect()).collect();
    let tiles: Vec<(i32, i32)> = (0..h).flat_map(|y| (0..w).map(move |x| (x, y))).filter(|&(x, y)| open[y][x]).map(|(x, y)| (x as i32, y as i32)).collect();
    let n = (2 + r.next(4)).min(tiles.len().saturating_sub(2)).max(2);
    let mut starts: Vec<(i32, i32)> = Vec::new();
    while starts.len() < n {
        let t = tiles[r.next(tiles.len())];
        if !starts.contains(&t) {
            starts.push(t);
        }
    }
    // Two or three different stops each: a loop that only stands on one tile isn't walking.
    let mut loops: Vec<Vec<(i32, i32)>> = Vec::new();
    for &s in &starts {
        let (mut stops, n) = (vec![s], 2 + r.next(2));
        while stops.len() < n {
            let t = tiles[r.next(tiles.len())];
            if !stops.contains(&t) {
                stops.push(t);
            }
        }
        loops.push(stops);
    }
    let (mut g, walk) = walkers(&rows.iter().map(|s| s.as_str()).collect::<Vec<_>>(), &loops.iter().map(|l| l.as_slice()).collect::<Vec<_>>());
    for _ in 0..r.next(3) {
        let t = tiles[r.next(tiles.len())];
        let at = on_ground(t);
        if !loops.iter().flatten().any(|&s| s == t) && g.world.item_at(at.0, at.1).is_none() && g.monster_at(at.0, at.1).is_none() {
            let boll = cotton_good(&g, "cotton_bolls");
            g.world.items.push(Item::new(at.0, at.1, boll));
        }
    }
    (g, walk, rows)
}

/// Random ground `seed` for 12,000 ticks with nobody passing through anybody. Any walker that
/// reaches no stop in the second half is printed with the ground (`o` its stops, `*` strays)
/// and what it says, for judging by eye whether there really was no way.
fn random_ground_runs(seed: u64) {
    let (mut g, mut walk, rows) = random_ground(seed);
    walk.run(&mut g, 6000);
    let half: Vec<u32> = g.monsters.iter().map(|m| m.steps).collect();
    walk.run(&mut g, 6000);
    // One whose next stop goods have cut off altogether isn't held up by anyone: walkers can't
    // shove goods aside (loops can).
    let cut_off = |g: &Game, k: usize| g.monsters[k].wait_reason == crate::motion::NO_WAY;
    let stuck: Vec<usize> = (0..g.monsters.len()).filter(|&k| walk.walking(k) && g.monsters[k].steps == half[k] && !cut_off(&g, k)).collect();
    if stuck.is_empty() {
        return;
    }
    let mut map: Vec<Vec<char>> = rows.iter().map(|r| r.chars().collect()).collect();
    let mut mark = |(x, y): (i32, i32), c: char| {
        let (gx, gy) = (x - GROUND.0, y - GROUND.1);
        if gx >= 0 && gy >= 0 && (gy as usize) < map.len() && (gx as usize) < map[0].len() {
            map[gy as usize][gx as usize] = c;
        }
    };
    for it in &g.world.items {
        mark((it.x, it.y), '*');
    }
    for &t in walk.stops.iter().flatten() {
        mark(t, 'o');
    }
    for (k, m) in g.monsters.iter().enumerate().filter(|&(k, _)| walk.walking(k)) {
        mark((m.x, m.y), (b'A' + k as u8) as char);
    }
    println!("ground {seed}:");
    for row in &map {
        println!("  {}", row.iter().collect::<String>());
    }
    let letter = |k: usize| (b'A' + k as u8) as char;
    let ground = |(x, y): (i32, i32)| (x - GROUND.0, y - GROUND.1);
    for (k, m) in g.monsters.iter().enumerate().filter(|&(k, _)| walk.walking(k)) {
        let stops: Vec<(i32, i32)> = walk.stops[k].iter().map(|&t| ground(t)).collect();
        let state = if stuck.contains(&k) { "STUCK" } else { "ok" };
        println!("  {} {state} at {:?} stops {stops:?} on {:?} says {:?}", letter(k), ground((m.x, m.y)), m.waiting_on.map(letter), m.wait_reason);
    }
    panic!("ground {seed}: someone stuck");
}

#[test]
#[ignore]
fn random_grounds() {
    let failed = failing(seeds(1..201), random_ground_runs);
    assert!(failed.is_empty(), "grounds with someone stuck: {failed:?}");
}

/// Not a check: prints how one random tangle goes (SEED picks it, 14 if unset). Run with
/// `cargo test --release trace_a_random_tangle -- --ignored --nocapture`.
#[test]
#[ignore]
fn trace_a_random_tangle() {
    let seed: u64 = std::env::var("SEED").ok().and_then(|s| s.parse().ok()).unwrap_or(14);
    let mut g = random_tangle(seed);
    for m in &g.monsters {
        let goals = &m.routine.as_ref().unwrap().goals;
        println!("{} at ({},{}) goal {} of {}: {:?}", m.name, m.x, m.y, m.pc, goals.len(), goals);
    }
    for it in &g.world.items {
        println!("item {} ({},{})", it.good(), it.x, it.y);
    }
    let until: u32 = std::env::var("UNTIL").ok().and_then(|s| s.parse().ok()).unwrap_or(30_000);
    for k in 1..=until / 3000 {
        g.run_ticks(3000);
        let row: Vec<String> = g.monsters.iter().map(|m| format!("{} s{} goal{} ({},{}) {:?}", m.name, m.steps, m.pc, m.x, m.y, m.wait_reason)).collect();
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

/// Not a check: prints how one random ground goes (SEED picks it, 1 if unset; UNTIL ticks, and
/// EVERY how often to print). Run with
/// `cargo test --release trace_a_random_ground -- --ignored --nocapture`.
#[test]
#[ignore]
fn trace_a_random_ground() {
    let var = |k: &str, d: u32| std::env::var(k).ok().and_then(|s| s.parse().ok()).unwrap_or(d);
    let (mut g, mut walk, rows) = random_ground(var("SEED", 1) as u64);
    for row in &rows {
        println!("  {row}");
    }
    let ground = |(x, y): (i32, i32)| (x - GROUND.0, y - GROUND.1);
    for t in 0..var("UNTIL", 3000) {
        walk.run(&mut g, 1);
        if t % var("EVERY", 300) == 0 {
            let row: Vec<String> = (0..g.monsters.len()).filter(|&k| walk.walking(k)).map(|k| {
                let m = &g.monsters[k];
                let knot = m.knot.as_ref().map(|kn| format!(" knot {}/{}", kn.next, kn.moves.len())).unwrap_or_default();
                let gw = m.giving_way.map(|gw| format!(" giving way to #{} at {:?}", gw.to, ground(gw.spot))).unwrap_or_default();
                format!("{}#{} {:?} s{} w{} on {:?}{knot}{gw} {:?}", (b'A' + k as u8) as char, m.id, ground((m.x, m.y)), m.steps, m.wait, m.waiting_on, m.wait_reason)
            }).collect();
            println!("t{t}: {}", row.join(" | "));
        }
    }
}
