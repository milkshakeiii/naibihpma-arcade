//! Several players in one world: each commune's own monsters, renown, tutorial and sounds.

use super::*;
use crate::econ::{ItemData, Share};

fn fabric(g: &Game) -> ItemData {
    let boll = g.data.good("cotton_bolls");
    let raw = crate::econ::RawMaterial { good: boll.id.clone(), base_value: boll.raw_material_base_value.unwrap(), density: boll.raw_material_density.unwrap(), spent: false };
    ItemData { good: "fabric".into(), quality: 0.5, carried_tags: Vec::new(), raw: vec![raw], depth: 2, value: 2.0, shares: Vec::new(), wear: 0, haul: Vec::new(), carted: None }
}

fn key(d: Dir) -> &'static str {
    match d {
        Dir::Up => "U",
        Dir::Down => "D",
        Dir::Left => "L",
        Dir::Right => "R",
    }
}

/// Puts monster `i` where it can take a step in `d`, two tiles clear of everyone else.
fn room_to_step(g: &mut Game, i: usize, d: Dir) {
    let (dx, dy) = d.delta();
    let clear = |g: &Game, x: i32, y: i32| g.world.walkable(x, y) && g.world.item_at(x, y).is_none() && !g.applicant_at(x, y) && !g.world.cart_at(x, y) && !g.monsters.iter().enumerate().any(|(o, m)| o != i && (m.x - x).abs() + (m.y - y).abs() <= 2);
    let (x, y) = (0..g.world.h).flat_map(|y| (0..g.world.w).map(move |x| (x, y))).find(|&(x, y)| clear(g, x, y) && clear(g, x + dx, y + dy)).expect("open ground");
    (g.monsters[i].x, g.monsters[i].y) = (x, y);
}

#[test]
fn a_newcomer_starts_at_the_gate_beside_whoever_is_there() {
    let mut g = Game::new().without_town();
    let k = g.join();
    assert_eq!((k, g.communes.len()), (1, 2));
    let theirs: Vec<&Monster> = g.monsters.iter().filter(|m| m.commune == 1).collect();
    assert_eq!(theirs.len(), 2, "a crafter and a gatherer");
    assert_eq!((theirs[0].kind.as_str(), theirs[1].kind.as_str()), ("goblin", "cyclops"));
    let mut tiles: Vec<(i32, i32)> = g.monsters.iter().map(|m| (m.x, m.y)).collect();
    tiles.sort();
    tiles.dedup();
    assert_eq!(tiles.len(), 4, "nobody starts on top of anyone");
    // Each beside where Mogg and Bruna started.
    for (m, &(sx, sy)) in theirs.iter().zip(&g.world.starts) {
        assert!((m.x - sx).abs() + (m.y - sy).abs() <= 4, "{} starts at {:?}, far from {:?}", m.name, (m.x, m.y), (sx, sy));
    }
    assert_eq!(g.monsters[g.communes[1].controlled].commune, 1);
    assert_eq!(g.communes[1].tutorial.stage, 0);
}

#[test]
fn tab_switches_only_between_your_own_monsters() {
    let mut g = Game::new().without_town();
    g.join();
    let first = g.communes[1].controlled;
    g.script_as(1, "t");
    let second = g.communes[1].controlled;
    assert_ne!(second, first);
    assert_eq!(g.monsters[second].commune, 1, "never someone else's monster");
    g.script_as(1, "t");
    assert_eq!(g.communes[1].controlled, first, "and back round to the first");
    assert_eq!(g.communes[0].controlled, 0, "the other player still controls Mogg");
}

#[test]
fn each_player_moves_only_the_monster_they_control() {
    let mut g = Game::new().without_town();
    g.join();
    let (mogg, theirs) = (g.communes[0].controlled, g.communes[1].controlled);
    room_to_step(&mut g, mogg, Dir::Up);
    room_to_step(&mut g, theirs, Dir::Down);
    room_to_step(&mut g, theirs, Dir::Up);
    let before = [(g.monsters[mogg].x, g.monsters[mogg].y), (g.monsters[theirs].x, g.monsters[theirs].y)];
    g.script_as(1, "U");
    assert_eq!((g.monsters[mogg].x, g.monsters[mogg].y), before[0]);
    assert_ne!((g.monsters[theirs].x, g.monsters[theirs].y), before[1]);
    // Both at once, in one tick.
    let up = Input { pressed: [true, false, false, false], held: [true, false, false, false], ..Input::default() };
    let down = Input { pressed: [false, true, false, false], held: [false, true, false, false], ..Input::default() };
    room_to_step(&mut g, theirs, Dir::Down);
    g.run_ticks(10);
    let (y0, y1) = (g.monsters[mogg].y, g.monsters[theirs].y);
    g.update(&[up, down]);
    assert_eq!((g.monsters[mogg].y, g.monsters[theirs].y), (y0 - 1, y1 + 1));
}

#[test]
fn renown_goes_to_each_contributors_commune_by_its_shares() {
    let mut g = Game::new().without_town();
    g.join();
    let (mogg, theirs) = (g.monsters[0].id, g.monsters.iter().find(|m| m.commune == 1).unwrap().id);
    let shares = [Share { monster: mogg, count: 3.0 }, Share { monster: theirs, count: 1.0 }];
    g.score(2.0, &shares, 10, 10);
    let paid = 2.0 * RENOWN_PER_VALUE;
    assert!((g.communes[0].renown - paid * 0.75).abs() < 1e-9, "{}", g.communes[0].renown);
    assert!((g.communes[1].renown - paid * 0.25).abs() < 1e-9, "{}", g.communes[1].renown);
}

#[test]
fn a_delivery_pays_the_maker_and_the_hauler_from_another_commune() {
    let mut g = Game::new().without_town();
    g.join();
    let store = g.world.bins.iter().position(|b| b.scores).expect("the cloth store");
    let (bx, by) = (g.world.bins[store].x, g.world.bins[store].y);
    let mut cloth = fabric(&g);
    cloth.shares = vec![Share { monster: g.monsters[0].id, count: 4.0 }];
    // The newcomer pushes Mogg's cloth into the store, from whichever side has room.
    let free = |x: i32, y: i32| g.monster_at(x, y).is_none() && g.world.item_at(x, y).is_none();
    let d = DIRS
        .into_iter()
        .find(|d| {
            let (dx, dy) = d.delta();
            g.world.item_floor(bx - dx, by - dy) && free(bx - dx, by - dy) && g.world.walkable(bx - 2 * dx, by - 2 * dy) && free(bx - 2 * dx, by - 2 * dy)
        })
        .expect("a side to push from");
    let (dx, dy) = d.delta();
    g.world.items.push(Item::new(bx - dx, by - dy, cloth));
    let c = g.communes[1].controlled;
    (g.monsters[c].x, g.monsters[c].y) = (bx - 2 * dx, by - 2 * dy);
    g.script_as(1, key(d));
    assert_eq!(g.world.bins[store].stock.len(), 1);
    assert!(g.communes[0].renown > 0.0, "the maker's commune is paid");
    assert!(g.communes[1].renown > 0.0, "and so is the hauler's");
}

#[test]
fn hiring_spends_your_renown_and_the_helper_joins_your_commune() {
    let mut g = Game::new().without_town();
    g.join();
    let stand = g.world.stands[1];
    let c = g.communes[1].controlled;
    (g.monsters[c].x, g.monsters[c].y, g.monsters[c].facing) = (stand.0, stand.1 - 1, Dir::Down);
    let cost = g.applicants[1].as_ref().unwrap().cost as f64;
    g.communes[0].renown = cost * 10.0;
    g.script_as(1, "u");
    assert_eq!(g.monsters.len(), 4, "the other commune's renown isn't yours to spend");
    g.communes[1].renown = cost + 3.0;
    g.script_as(1, "u");
    assert_eq!(g.monsters.len(), 5);
    assert_eq!(g.monsters[4].commune, 1);
    assert!((g.communes[1].renown - 3.0).abs() < 1e-9);
    assert!((g.communes[0].renown - cost * 10.0).abs() < 1e-9);
}

#[test]
fn a_newcomer_skips_steps_already_done_in_the_world_but_not_their_own() {
    let mut g = Game::new().without_town();
    g.join();
    // Somebody else stored fabric and recorded a loop.
    let store = g.world.bins.iter().position(|b| b.scores).expect("the cloth store");
    let cloth = fabric(&g);
    g.world.bins[store].stock.push(cloth);
    g.monsters[0].routine = Some(Routine { flag: (0, 0), goals: Vec::new() });
    g.communes[1].tutorial.stage = 6;
    g.run_ticks(1);
    assert_eq!(g.communes[1].tutorial.stage, 7, "fabric is already stored, but recording a loop is theirs to do");
    // Somebody else built the footbridge.
    g.world.build_footbridge();
    g.communes[1].tutorial.stage = 11;
    g.run_ticks(1);
    assert_eq!(g.communes[1].tutorial.stage, 12, "the footbridge is up, but they still have to walk to Tanglemere");
    // What one commune makes doesn't count for another.
    g.event(0, Ev::Made("rope".into()));
    g.run_ticks(1);
    assert!(g.communes[0].made.contains("rope") && !g.communes[1].made.contains("rope"));
}

#[test]
fn each_player_hears_their_own_actions_and_their_own_screen() {
    let mut g = Game::new().without_town();
    g.join();
    g.press(1, Input { rec_pressed: true, ..Input::default() });
    assert!(g.sounds_for(1).contains(&Sfx::RecOn));
    assert!(!g.sounds_for(0).contains(&Sfx::RecOn), "another player's button press is theirs alone");
    // A sound on Mogg's screen, heard there and not from another screen.
    let (x, y) = (g.monsters[0].x, g.monsters[0].y);
    let c = g.communes[1].controlled;
    g.monsters[c].x += 2 * crate::world::SCREEN_W;
    g.run_ticks(1);
    g.sound((x, y), Sfx::Chime);
    assert!(g.sounds_for(0).contains(&Sfx::Chime));
    assert!(!g.sounds_for(1).contains(&Sfx::Chime));
}

#[test]
fn a_world_with_two_players_saves_and_loads_exactly() {
    // With the town and its working folk, as every real world has.
    let mut g = Game::new();
    let k = g.join();
    g.script_as(k, "UUt");
    g.communes[k].renown = 12.5;
    let copy = crate::save::from_json(&crate::save::to_json(&g)).expect("loads");
    assert_eq!(crate::save::to_json(&copy), crate::save::to_json(&g));
    assert_eq!(copy.communes.len(), 3, "the first player's, the town's, and the newcomer's");
    assert_eq!(copy.communes[k].controlled, g.communes[k].controlled);
}
