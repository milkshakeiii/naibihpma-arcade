//! Frontier workshops: kits made at the builders' bench, unpacked out in the wilds, worn out,
//! renewed and fallen apart.

use super::*;
use crate::econ::{ItemData, RawMaterial, Share};
use crate::frontier::{FALL_APART_TICKS, LIFETIME_TICKS};

/// A raw material straight from the wild.
fn raw(g: &Game, id: &str) -> ItemData {
    let gt = g.data.good(id);
    let base = gt.raw_material_base_value.unwrap_or(1.0);
    let density = gt.raw_material_density.unwrap_or(1.0);
    ItemData { good: id.into(), quality: 0.5, carried_tags: Vec::new(), raw: vec![RawMaterial { good: id.into(), base_value: base, density, spent: false }], depth: 0, value: base, shares: Vec::new(), wear: 0, haul: Vec::new(), carted: None }
}

/// A made good from raw materials, carrying `tags`.
fn made(g: &Game, id: &str, from: &[&str], tags: &[&str]) -> ItemData {
    let raws = from.iter().flat_map(|r| raw(g, r).raw).collect();
    ItemData { good: id.into(), quality: 0.6, carried_tags: tags.iter().map(|t| t.to_string()).collect(), raw: raws, depth: 2, value: 3.0, ..raw(g, from[0]) }
}

/// Cotton rope, as twisted from thread.
fn rope(g: &Game) -> ItemData {
    made(g, "rope", &["cotton_bolls", "cotton_bolls"], &["cotton"])
}

/// A workbench kit of this wood, made by Mogg.
fn bench_kit(g: &Game, wood: &str, quality: f64) -> ItemData {
    let mut kit = made(g, "workbench_kit", &[wood, wood, "cotton_bolls", "cotton_bolls"], &[&wood.replace("_log", "")]);
    (kit.quality, kit.shares) = (quality, vec![Share { monster: 1, count: 1.0 }]);
    kit
}

/// Puts the controlled monster at a tile, facing a way.
fn stand(g: &mut Game, (x, y): (i32, i32), facing: Dir) {
    let c = g.controlled;
    (g.monsters[c].x, g.monsters[c].y, g.monsters[c].facing) = (x, y, facing);
}

/// Open grass in Tinless, the wilderness just east of Pinewater, with a road to its west.
const MEADOW: (i32, i32) = (66, 11);

/// A hemlock workbench kit lying at the meadow, Mogg just north of it facing it, unpacked.
fn bench_in_the_meadow(g: &mut Game) -> usize {
    g.world.items.push(Item::new(MEADOW.0, MEADOW.1, bench_kit(g, "hemlock_log", 0.7)));
    stand(g, (MEADOW.0, MEADOW.1 - 1), Dir::Down);
    g.script("u");
    let s = g.world.shops.len() - 1;
    assert!(g.world.shops[s].frontier.is_some(), "unpacked: {:?}", g.monsters[0].bubble.as_ref().map(|b| &b.text));
    s
}

#[test]
fn the_builders_bench_makes_a_kiln_kit_from_six_goods_with_the_tools_it_lends() {
    let mut g = Game::new();
    let bench = g.world.shop_at_spot(36, 28).expect("the builders' bench, by the southeast road");
    let pads = g.world.shops[bench].pads.clone();
    assert_eq!(pads.len(), 6);
    assert_eq!(g.world.shops[bench].default_recipe(&g.data), "kiln_kit", "an empty one asks for a kiln kit's goods");
    assert_eq!(g.world.shops[bench].kind.builtin_tools(), ["saw", "metal_chisel", "stone_hammer"], "the tools for building, and no cotton tools");
    let mut stone = raw(&g, "limestone");
    stone.carried_tags.clear();
    let quicklime = made(&g, "quicklime", &["limestone", "hemlock_log"], &[]);
    let goods = [stone.clone(), stone.clone(), stone, quicklime, rope(&g), raw(&g, "oak_log")];
    for (k, data) in goods.into_iter().enumerate() {
        g.world.items.push(Item::new(pads[k].0, pads[k].1, data));
    }
    g.controlled = 1;
    stand(&mut g, (36, 28), Dir::Up);
    g.script("u");
    for _ in 0..4000 {
        if g.made.contains("kiln_kit") {
            break;
        }
        g.run_ticks(1);
    }
    g.run_ticks(30);
    // Made with the stone hammer and chisel the bench lends (no kit otherwise).
    let kit = g.world.items.iter().map(|it| &it.data).chain(&g.world.shops[bench].pending).find(|d| d.good == "kiln_kit").expect("a kiln kit");
    assert!(kit.carried_tags.contains(&"limestone".to_string()) && kit.carried_tags.contains(&"oak".to_string()), "{:?}", kit.carried_tags);
}

#[test]
fn a_kit_unpacks_into_its_workshop_out_in_the_wilds_turned_the_way_you_face_it() {
    let mut g = Game::new();
    let before = g.world.shops.len();
    let s = bench_in_the_meadow(&mut g);
    assert_eq!(s, before, "a new workshop");
    let shop = &g.world.shops[s];
    assert_eq!((shop.kind, shop.spot), (ShopKind::Workbench, MEADOW));
    assert_eq!((shop.pads.clone(), shop.outs.clone(), shop.pegs.clone()), (vec![(64, 11), (65, 11)], vec![(66, 12)], vec![(67, 11), (68, 11)]), "facing south: pads west, mat south, pegs east");
    assert!((shop.quality - 0.7).abs() < 1e-9, "the kit's quality");
    assert!(g.world.items.iter().all(|it| it.good() != "workbench_kit"), "the kit is used up");
    assert_eq!((g.world.tile(64, 11), g.world.tile(66, 11), g.world.tile(66, 12), g.world.tile(67, 11)), (Tile::Pad, Tile::Spot, Tile::OutMat, Tile::Peg));
    assert_eq!(g.shop_name(s), "HEMLOCK WORKBENCH");

    // Faced from below, it turns round: pads east, mat north, pegs west.
    let mut g = Game::new();
    g.world.items.push(Item::new(MEADOW.0, MEADOW.1, bench_kit(&g, "oak_log", 0.7)));
    stand(&mut g, (MEADOW.0, MEADOW.1 + 1), Dir::Up);
    g.script("u");
    let shop = g.world.shops.last().unwrap();
    assert_eq!((shop.pads.clone(), shop.outs.clone(), shop.pegs.clone()), (vec![(68, 11), (67, 11)], vec![(66, 10)], vec![(65, 11), (64, 11)]));
}

#[test]
fn nobody_builds_in_town_on_a_road_or_on_top_of_something() {
    let refused = |at: (i32, i32), facing: Dir, from: (i32, i32), clutter: Option<(i32, i32)>| {
        let mut g = Game::new();
        let before = g.world.shops.len();
        g.world.items.push(Item::new(at.0, at.1, bench_kit(&g, "hemlock_log", 0.5)));
        if let Some(c) = clutter {
            g.world.items.push(Item::new(c.0, c.1, raw(&g, "oak_log")));
        }
        stand(&mut g, from, facing);
        g.script("u");
        assert_eq!(g.world.shops.len(), before, "not set up at {at:?}");
        g.monsters[0].bubble.as_ref().map(|b| b.text.clone()).unwrap_or_default()
    };
    assert!(refused((20, 13), Dir::Down, (20, 12), None).contains("TOWN"), "Pinewater");
    assert!(refused((62, 13), Dir::Down, (62, 12), None).contains("ROAD"), "its pads would be on the road");
    assert!(refused(MEADOW, Dir::Down, (MEADOW.0, MEADOW.1 - 1), Some((66, 12))).contains("IN THE WAY"), "a log where its mat goes");
}

#[test]
fn a_frontier_bench_lends_no_tools_but_twists_rope() {
    let mut g = Game::new();
    let s = bench_in_the_meadow(&mut g);
    let pads = g.world.shops[s].pads.clone();
    for &p in &pads {
        g.world.items.push(Item::new(p.0, p.1, raw(&g, "oak_log")));
    }
    stand(&mut g, MEADOW, Dir::Down);
    g.script("u");
    let said = g.monsters[0].bubble.as_ref().map(|b| b.text.clone()).unwrap_or_default();
    assert!(said.contains("MISSING A TOOL"), "no saw or chisel out here: {said}");
    g.world.items.retain(|it| it.good() != "oak_log");
    for &p in &pads {
        g.world.items.push(Item::new(p.0, p.1, made(&g, "thread", &["cotton_bolls"], &["cotton"])));
    }
    g.script("u");
    g.run_ticks(600);
    assert!(g.made.contains("rope"), "{:?}", g.monsters[0].bubble.as_ref().map(|b| &b.text));
}

#[test]
fn a_kit_worth_more_makes_a_faster_workshop_gently() {
    let g = Game::new();
    let (hemlock, oak) = (crate::frontier::speed(&bench_kit(&g, "hemlock_log", 0.5)), crate::frontier::speed(&bench_kit(&g, "oak_log", 0.5)));
    assert!((hemlock - 1.0).abs() < 1e-9, "a hemlock bench works at town speed: {hemlock}");
    assert!((1.15..1.25).contains(&oak), "an oak one about a fifth faster: {oak}");
}

#[test]
fn a_worn_out_workshop_stops_until_a_new_kit_renews_it_and_left_alone_falls_apart() {
    let mut g = Game::new();
    let s = bench_in_the_meadow(&mut g);
    g.world.shops[s].frontier.as_mut().unwrap().worked = LIFETIME_TICKS;
    stand(&mut g, MEADOW, Dir::Down);
    g.script("u");
    let said = g.monsters[0].bubble.as_ref().map(|b| b.text.clone()).unwrap_or_default();
    assert!(said.contains("WORN OUT"), "{said}");

    // Pushed onto its work spot, a new kit renews it.
    g.world.items.push(Item::new(MEADOW.0, MEADOW.1 - 1, bench_kit(&g, "oak_log", 0.9)));
    stand(&mut g, (MEADOW.0, MEADOW.1 - 2), Dir::Down);
    g.script("D");
    let shop = &g.world.shops[s];
    assert_eq!(shop.frontier.as_ref().map(|f| f.worked), Some(0), "like new");
    assert!((shop.quality - 0.9).abs() < 1e-9 && g.shop_name(s) == "OAK WORKBENCH", "with the new kit's quality and wood");
    assert!(g.world.items.iter().all(|it| it.good() != "workbench_kit"));

    // Worn out and left unused long enough, it falls apart.
    let f = g.world.shops[s].frontier.as_mut().unwrap();
    (f.worked, f.idle) = (LIFETIME_TICKS, FALL_APART_TICKS - 1);
    g.run_ticks(2);
    assert!(g.world.shops[s].frontier.as_ref().unwrap().fallen);
    assert_eq!((g.world.tile(64, 11), g.world.tile(66, 11)), (Tile::Grass, Tile::Grass), "its ground is free again");
    assert_eq!(g.world.shop_at_spot(MEADOW.0, MEADOW.1), None);
}

#[test]
fn frontier_workshops_are_saved_with_their_ground() {
    let mut g = Game::new();
    let s = bench_in_the_meadow(&mut g);
    let back = crate::save::from_json(&crate::save::to_json(&g)).expect("it loads");
    assert_eq!(back.world.shops.len(), g.world.shops.len());
    assert_eq!(back.world.shops[s].spot, MEADOW);
    assert_eq!((back.world.tile(64, 11), back.world.tile(66, 11)), (Tile::Pad, Tile::Spot));
}

#[test]
fn a_loop_works_a_frontier_workshop_like_any_other() {
    let mut g = Game::new();
    let s = bench_in_the_meadow(&mut g);
    for &p in &g.world.shops[s].pads.clone() {
        g.world.items.push(Item::new(p.0, p.1, made(&g, "thread", &["cotton_bolls"], &["cotton"])));
    }
    let m = &mut g.monsters[0];
    m.routine = Some(Routine { flag: (MEADOW.0, MEADOW.1 - 1), goals: vec![Goal::Activate(Thing::Shop(s))] });
    (m.pc, m.playing) = (0, true);
    g.run_ticks(600);
    assert!(g.made.contains("rope"), "{}", g.monsters[0].wait_reason);
}
