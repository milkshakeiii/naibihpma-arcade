//! Play on the Pinewater map: movement feel, harvesting, the whole Fabric chain by hand, and
//! two monsters looping it on their own.

use super::routes::*;
use super::*;
use crate::econ::ItemData;

fn boll(data: &Data) -> ItemData {
    let g = data.good("cotton_bolls");
    ItemData {
        good: g.id.clone(),
        quality: 0.5,
        carried_tags: Vec::new(),
        raw: vec![crate::econ::RawMaterial { good: g.id.clone(), base_value: g.raw_material_base_value.unwrap(), density: g.raw_material_density.unwrap() }],
        depth: 0,
        value: 0.0,
        shares: Vec::new(),
        wear: 0,
    }
}

#[test]
fn a_tap_moves_one_tile_and_a_long_hold_keeps_walking() {
    let hold = |ticks: u32| {
        let mut g = Game::new();
        g.script("t"); // Bruna has open grass to the north
        let start = g.monsters[1].y;
        let (mut req, mut sfx) = (Vec::new(), Vec::new());
        for t in 0..ticks {
            let mut inp = Input::default();
            inp.held[0] = true;
            inp.pressed[0] = t == 0;
            g.update(&inp, &mut req, &mut sfx);
        }
        g.run_ticks(10);
        start - g.monsters[1].y
    };
    assert_eq!(hold(1), 1);
    assert_eq!(hold(HOLD_REPEAT_TICKS as u32), 1, "a ~250 ms tap must not take a second step");
    assert!(hold(12) >= 4, "holding should keep walking");
}

#[test]
fn hedges_block_and_items_push_one_tile() {
    let mut g = Game::new();
    g.script("LL");
    assert_eq!((g.monsters[0].x, g.monsters[0].y), (1, 8), "the hedge stops the second step");
    g.script("RD");
    let data = boll(&g.data);
    g.world.items.push(Item::new(3, 9, data));
    g.script("R");
    let it = &g.world.items[0];
    assert_eq!((it.x, it.y), (4, 9));
}

#[test]
fn harvesting_pops_a_boll_and_teaches_the_harvester() {
    let mut g = Game::new();
    let before = g.monsters[0].skills.applied("harvesting");
    g.script("RRRRRRu");
    let it = g.world.item_at(9, 8).expect("a boll popped out in front");
    assert_eq!(g.world.items[it].good(), "cotton_bolls");
    assert!(g.world.items[it].data.quality > 0.0);
    assert!(g.monsters[0].skills.applied("harvesting") > before);
    assert!(g.monsters[0].learned.is_some(), "a skill popup is pending");
}

#[test]
fn the_whole_chain_makes_cotton_fabric_with_a_full_history() {
    let mut g = Game::new();
    g.script(MOGG_HARVEST);
    assert_eq!(g.tutorial.stage, 3, "harvested and binned");
    g.script("t");
    g.run_ticks(10);
    g.script(&bruna_chain());
    assert_eq!(g.cloth_stored(), 1, "one bolt of fabric in the cloth store");
    let fabric = &g.world.bins.iter().find(|b| b.accepts == "fabric").unwrap().stock[0];
    assert_eq!(fabric.good, "fabric");
    assert!(fabric.carried_tags.contains(&"cotton".to_string()), "{:?}", fabric.carried_tags);
    assert_eq!(fabric.depth, 3, "bolls, slivers, thread, fabric");
    assert!(fabric.quality > 0.0 && fabric.value > 0.0);
    let credited = |m: u32| fabric.shares.iter().any(|s| s.monster == m && s.count > 0.0);
    assert!(credited(1) && credited(2), "both the harvester and the weaver share in it: {:?}", fabric.shares);
    assert!(g.monsters[1].skills.applied("weaving") > 0.0);
    assert_eq!(g.tutorial.stage, 7);
}

#[test]
fn two_monsters_loop_the_chain_on_their_own() {
    let mut g = Game::new();
    g.script(MOGG_HARVEST);
    g.script(MOGG_HOME);
    g.script(&format!("r{MOGG_HARVEST}{MOGG_HOME}rp"));
    assert!(g.monsters[0].playing);
    g.script("t");
    g.run_ticks(10);
    g.script(&format!("r{}{}rp", bruna_chain(), bruna_home()));
    assert!(g.monsters[1].playing);
    assert_eq!(g.tutorial.stage, tutorial::DONE);

    let before = g.cloth_stored();
    g.run_ticks(6000);
    assert!(g.cloth_stored() >= before + 3, "only {} more fabric in 400 s", g.cloth_stored() - before);
    assert!(g.monsters.iter().all(|m| m.playing), "a loop stopped");
}
