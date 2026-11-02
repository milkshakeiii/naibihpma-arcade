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
        raw: vec![crate::econ::RawMaterial { good: g.id.clone(), base_value: g.raw_material_base_value.unwrap(), density: g.raw_material_density.unwrap(), spent: false }],
        depth: 0,
        value: 0.0,
        shares: Vec::new(),
        wear: 0,
        haul: Vec::new(),
        carted: None,
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
    let fabric = &g.world.bins.iter().find(|b| b.takes("fabric")).unwrap().stock[0];
    assert_eq!(fabric.good, "fabric");
    assert!(fabric.carried_tags.contains(&"cotton".to_string()), "{:?}", fabric.carried_tags);
    assert_eq!(fabric.depth, 3, "bolls, slivers, thread, fabric");
    assert!(fabric.quality > 0.0 && fabric.value > 0.0);
    let credited = |m: u32| fabric.shares.iter().any(|s| s.monster == m && s.count > 0.0);
    assert!(credited(1) && credited(2), "both the harvester and the weaver share in it: {:?}", fabric.shares);
    assert!(g.monsters[1].skills.applied("weaving") > 0.0);
    let paid = fabric.value * RENOWN_PER_VALUE;
    assert!((g.renown - paid).abs() < 1e-9, "the cloth store paid renown for its value");
    let earned: f64 = g.monsters.iter().map(|m| m.earned).sum();
    assert!((earned - paid).abs() < 1e-9 && g.monsters.iter().all(|m| m.earned > 0.0), "credit split between both");
    assert!(g.floaters.iter().any(|f| f.text.contains("RENOWN")), "a renown celebration rises");
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
    assert!(tutorial::STAGES[g.tutorial.stage].contains("GATHERING HALL"), "next the tutorial asks you to welcome a helper");

    let before = g.cloth_stored();
    g.run_ticks(6000);
    assert!(g.cloth_stored() >= before + 3, "only {} more fabric in 400 s", g.cloth_stored() - before);
    assert!(g.monsters.iter().all(|m| m.playing), "a loop stopped");
}

/// Does any line on any page of the card say this (in its text or its trailing text)?
fn has_line(card: &crate::inspect::Card, needle: &str) -> bool {
    card.lines().any(|l| l.text.contains(needle) || l.then.as_ref().is_some_and(|(_, t)| t.contains(needle)))
}

fn on_first_page(card: &crate::inspect::Card, needle: &str) -> bool {
    card.pages[0].iter().any(|l| l.text.contains(needle) || l.then.as_ref().is_some_and(|(_, t)| t.contains(needle)))
}

#[test]
fn examining_a_boll_shows_its_quality_and_history() {
    let mut g = Game::new();
    g.script("RRRRRRu");
    let card = crate::inspect::card(&g);
    assert_eq!(card.title, "COTTON BOLLS");
    assert!(card.lines().all(|l| l.meter.is_none() && l.icon != Some(crate::inspect::Icon::Quality)), "bolls have no quality, so the card doesn't show one");
    assert!(!has_line(&card, "RENOWN"), "nothing takes bolls for renown, so the card doesn't promise any");
    assert!(has_line(&card, "RAW MATERIAL") && has_line(&card, "CREDIT: MOGG"));
}

#[test]
fn examining_with_nothing_ahead_describes_your_monster() {
    let mut g = Game::new();
    g.monsters[0].facing = Dir::Left; // open grass, not the cotton plant to the east
    let card = crate::inspect::card(&g);
    assert_eq!(card.title, "MOGG THE GOBLIN");
    assert!(on_first_page(&card, "SPINNING"), "skills first");
    assert!(has_line(&card, "TALENTS: HANDCRAFTS") && !on_first_page(&card, "TALENTS"), "talents on page 2");
}

#[test]
fn the_spinnery_forecast_shows_quality_odds_learning_and_what_holds_it_back() {
    let mut g = Game::new();
    let mut slivers = boll(&g.data);
    slivers.good = "cotton_slivers".into();
    slivers.carried_tags = vec!["cotton".into()];
    slivers.depth = 1;
    g.world.items.push(Item::new(30, 9, slivers));
    g.controlled = 1;
    (g.monsters[1].x, g.monsters[1].y) = (30, 8);
    let card = crate::inspect::card(&g);
    assert_eq!(card.title, "SPINNERY: THREAD");
    let first = &card.pages[0];
    assert!(first.len() <= 6, "the first page stays short: {:?}", first.iter().map(|l| &l.text).collect::<Vec<_>>());
    assert!(first.iter().any(|l| l.meter.is_some()), "with a quality meter");
    for needle in ["1 THREAD", "SPINNING +", "HELD BACK"] {
        assert!(on_first_page(&card, needle), "page 1 misses {needle}");
    }
    for needle in ["USUALLY", "EXCEPTIONAL", "FORGETS", "LEVELS OFF"] {
        assert!(has_line(&card, needle) && !on_first_page(&card, needle), "{needle} belongs on page 2");
    }
    assert!(g.status(1).0.contains("SPINNERY EXPECTS QUALITY"), "{}", g.status(1).0);
    g.world.items.clear();
    assert!(has_line(&crate::inspect::card(&g), "NEEDS COTTON SLIVERS ON THE PAD."));
}

#[test]
fn extra_output_waits_inside_so_loops_never_jam() {
    let mut g = Game::new();
    let bench = g.world.shop_at_spot(14, 9).unwrap();
    let extra = boll(&g.data);
    g.world.shops[bench].pending = vec![extra.clone(), extra.clone()];
    g.world.items.push(Item::new(15, 9, extra.clone())); // one already out on the mat
    g.world.items.push(Item::new(13, 9, boll(&g.data))); // a fresh boll on the pad
    g.controlled = 1;
    (g.monsters[1].x, g.monsters[1].y) = (14, 9);
    g.script("u");
    assert!(g.made.contains("cotton_slivers"), "the bench works even with output waiting");
    assert!(g.world.shops[bench].pending.len() >= 3, "new slivers queue behind the waiting ones");

    g.world.shops[bench].pending = vec![extra; crate::world::OUTPUT_BUFFER];
    g.world.items.push(Item::new(13, 9, boll(&g.data)));
    g.script("u");
    let said = g.monsters[1].bubble.as_ref().map(|b| b.text.clone()).unwrap_or_default();
    assert!(said.contains("IS FULL"), "a full workshop says so: {said}");
}

#[test]
fn a_loop_makes_nothing_more_for_a_full_workshop_backed_up_with_its_product() {
    let full_bench = |waiting: &str| {
        let mut g = Game::new();
        let bench = g.world.shop_at_spot(14, 9).unwrap();
        g.world.shops[bench].pending = vec![good_like(&g.data, waiting, 0.8); crate::world::OUTPUT_BUFFER];
        g.world.items.push(Item::new(15, 9, good_like(&g.data, waiting, 0.8)));
        g.world.items.push(Item::new(13, 9, boll(&g.data))); // would make slivers
        start_loop(&mut g, 0, Routine { flag: (14, 9), goals: vec![Goal::Activate(Thing::Shop(bench))] }, (14, 9));
        g.run_ticks(60);
        (g, bench)
    };
    // Its output all taken up with slivers already: nothing to make. Not stuck waiting on it.
    let (g, bench) = full_bench("cotton_slivers");
    assert!(g.world.shops[bench].job.is_none() && g.world.item_at(13, 9).is_some(), "nothing made, the boll left for later");
    assert!(!g.monsters[0].wait_reason.contains("FULL"), "{}", g.monsters[0].wait_reason);
    // Taken up with something else: wait for someone to clear it.
    let (g, _) = full_bench("thread");
    assert!(g.monsters[0].wait_reason.contains("CLEAR ITS OUTPUT MAT"), "{}", g.monsters[0].wait_reason);
}

#[test]
fn a_loop_keeps_a_workshop_working_while_its_product_waits_on_the_mat() {
    // Slivers on the bench's mat, two more waiting inside: room for plenty more, so the loop
    // works the bolls it has rather than waiting for the slivers to be taken away.
    let mut g = Game::new();
    let bench = g.world.shop_at_spot(14, 9).unwrap();
    g.world.shops[bench].pending = vec![good_like(&g.data, "cotton_slivers", 0.8); 2];
    g.world.items.push(Item::new(15, 9, good_like(&g.data, "cotton_slivers", 0.8)));
    g.world.items.push(Item::new(13, 9, boll(&g.data)));
    g.world.items.push(Item::new(12, 9, boll(&g.data)));
    let chute_to_pad = Goal::Move { good: "cotton_bolls".into(), from: (12, 9), to: (13, 9) };
    start_loop(&mut g, 0, Routine { flag: (14, 10), goals: vec![chute_to_pad, Goal::Activate(Thing::Shop(bench))] }, (14, 10));
    (g.monsters[1].x, g.monsters[1].y) = (20, 13); // Bruna, off out of the way
    g.run_ticks(400);
    assert_eq!(g.world.shops[bench].pending.len(), 4, "both bolls worked: {}", g.monsters[0].wait_reason);
}

#[test]
fn examining_a_plant_forecasts_the_harvest() {
    let g = Game::new(); // Mogg starts facing the first cotton plant
    let card = crate::inspect::card(&g);
    assert_eq!(card.title, "COTTON PLANT");
    for needle in ["RIPE", "COTTON BOLLS", "HARVESTING +"] {
        assert!(on_first_page(&card, needle), "page 1 misses {needle}: {:?}", card.pages[0].iter().map(|l| &l.text).collect::<Vec<_>>());
    }
    assert!(has_line(&card, "LEVELS OFF"));
    assert!(card.lines().all(|l| l.meter.is_none()), "cotton bolls are ungraded");
}

#[test]
fn renown_welcomes_an_applicant_and_a_new_one_arrives() {
    let mut g = Game::new();
    let stand = g.world.stands[1];
    (g.monsters[0].x, g.monsters[0].y) = (stand.0, stand.1 - 1);
    g.monsters[0].facing = Dir::Down;
    let cost = g.applicants[1].as_ref().unwrap().cost as f64;
    g.script("u");
    assert_eq!(g.monsters.len(), 2, "not enough renown, nobody joins");
    assert!(g.monsters[0].bubble.as_ref().is_some_and(|b| b.text.contains("THEY JOIN AT")));

    g.renown = cost + 7.0;
    let name = g.applicants[1].as_ref().unwrap().name.clone();
    g.script("u");
    assert_eq!(g.monsters.len(), 3);
    assert_eq!(g.monsters[2].name, name);
    assert_eq!((g.monsters[2].x, g.monsters[2].y), stand);
    assert!((g.renown - 7.0).abs() < 1e-9);
    assert!(g.applicants[1].is_none());
    g.run_ticks(400);
    assert!(g.applicants[1].is_none(), "nobody arrives while the newcomer still stands there");
    g.controlled = 2;
    g.script("UL");
    g.run_ticks(400);
    assert!(g.applicants[1].is_some(), "a new applicant arrives once the stand is clear");
}

fn good_like(data: &Data, id: &str, quality: f64) -> ItemData {
    ItemData { good: id.into(), quality, carried_tags: vec!["cotton".into()], depth: 2, value: 2.0, ..boll(data) }
}

/// Puts the controlled monster at a tile, facing a way.
fn stand(g: &mut Game, (x, y): (i32, i32), facing: Dir) {
    let c = g.controlled;
    (g.monsters[c].x, g.monsters[c].y, g.monsters[c].facing) = (x, y, facing);
}

#[test]
fn two_thread_on_the_gate_bench_twist_into_rope_by_hand() {
    let mut g = Game::new();
    let bench = g.world.shop_at_spot(42, 8).expect("the gate bench");
    assert_eq!(g.world.shops[bench].default_recipe(&g.data), "rope");
    g.controlled = 1;
    stand(&mut g, (42, 8), Dir::Down);
    assert!(has_line(&crate::inspect::card(&g), "NEEDS THREAD ON THE PAD."), "an empty gate bench asks for thread");
    g.world.items.push(Item::new(40, 8, good_like(&g.data, "thread", 0.8)));
    g.world.items.push(Item::new(41, 8, good_like(&g.data, "thread", 0.8)));
    assert_eq!(crate::inspect::card(&g).title, "WORKBENCH: ROPE");
    g.script("u");
    g.run_ticks(20);
    assert!(g.made.contains("rope"));
    let rope = g.world.items.iter().find(|it| it.good() == "rope").expect("rope on the output mat");
    assert_eq!(rope.data.carried_tags, vec!["cotton".to_string()], "cotton rope keeps its fiber");
    assert!(g.world.items.iter().all(|it| it.good() != "thread"), "both thread were used");
}

#[test]
fn rope_for_the_board_builds_a_footbridge_that_goods_cross() {
    let mut g = Game::new();
    let (bx, by) = (g.world.boards[0].x, g.world.boards[0].y);
    let (&(west, y), &(east, _)) = (g.world.footbridge.first().unwrap(), g.world.footbridge.last().unwrap());

    // Not built yet: the river stops you, and says why.
    stand(&mut g, (west - 1, y), Dir::Right);
    g.script("R");
    assert_eq!(g.monsters[0].x, west - 1, "the river stops you");
    assert!(g.monsters[0].bubble.as_ref().is_some_and(|b| b.text.contains("NEEDS ROPE")));

    g.world.boards[0].have = g.world.boards[0].need - 1;
    g.world.items.push(Item::new(bx, by + 1, good_like(&g.data, "rope", 0.9)));
    stand(&mut g, (bx, by + 2), Dir::Up);
    let renown = g.renown;
    g.script("U");
    assert!(g.world.boards[0].done() && g.world.footbridge_built(), "the last rope builds the footbridge");
    assert!(g.renown > renown, "rope for the board earns renown");

    // Push a bolt of cloth across it.
    g.world.items.push(Item::new(west - 1, y, good_like(&g.data, "fabric", 1.0)));
    stand(&mut g, (west - 2, y), Dir::Right);
    g.script(&"R".repeat((east - west + 3) as usize));
    assert_eq!(g.monsters[0].x, east + 1, "walked across");
    assert!(g.world.item_at(east + 2, y).is_some(), "pushing the cloth over ahead of it");
}

#[test]
fn a_loop_carries_goods_over_the_footbridge_and_back() {
    let mut g = Game::new();
    g.world.build_footbridge();
    let (&(west, y), &(east, _)) = (g.world.footbridge.first().unwrap(), g.world.footbridge.last().unwrap());
    g.world.items.push(Item::new(west - 1, y, good_like(&g.data, "fabric", 1.0)));
    let (here, there) = ((west - 1, y), (east + 2, y));
    let fabric = |from, to| Goal::Move { good: "fabric".into(), from, to };
    start_loop(&mut g, 0, Routine { flag: (west - 2, y), goals: vec![fabric(here, there), fabric(there, here)] }, (west - 2, y));
    g.run_ticks(1500);
    let m = &g.monsters[0];
    assert!(m.playing && m.loops >= 2, "looped {} times: {}", m.loops, m.wait_reason);
}

#[test]
fn a_cornered_item_says_how_to_pull_it_and_pulling_is_slow() {
    let mut g = Game::new();
    g.world.items.push(Item::new(1, 1, boll(&g.data))); // Pinewater's top-left corner
    stand(&mut g, (2, 1), Dir::Left);
    g.script("L");
    let said = g.monsters[0].bubble.as_ref().map(|b| b.text.clone()).unwrap_or_default();
    assert!(said.contains("HOLD SHIFT"), "pushing a stuck item explains pulling: {said}");

    let before = g.frame;
    g.script(">");
    assert_eq!((g.monsters[0].x, g.world.items[0].x), (3, 2), "backed away one tile, dragging the boll");
    assert_eq!(g.monsters[0].facing, Dir::Left, "still facing what it pulls");
    assert!(g.frame - before >= PULL_TICKS as u32, "a pull takes {} ticks", g.frame - before);

    g.script("<");
    assert_eq!(g.monsters[0].x, 2, "Shift with nothing to drag is just a step");
}

#[test]
fn a_loop_pulls_out_of_a_corner_only_when_it_must_and_pays_for_it() {
    let mut g = Game::new();
    g.world.items.push(Item::new(1, 1, boll(&g.data))); // Pinewater's top-left corner
    stand(&mut g, (6, 3), Dir::Left);
    let m = &mut g.monsters[0];
    let boll = || "cotton_bolls".to_string();
    m.routine = Some(Routine { flag: (6, 3), goals: vec![Goal::Move { good: boll(), from: (1, 1), to: (4, 1) }, Goal::Move { good: boll(), from: (4, 1), to: (1, 1) }] });
    (m.pc, m.playing) = (0, true);
    let (plan, _) = g.plan_carry((1, 1), (4, 1), (6, 3), "cotton_bolls").expect("a plan out of the corner");
    assert_eq!(plan.iter().filter(|s| s.1).count(), 1, "one pull gets it out of the corner, pushes do the rest: {plan:?}");
    (g.world.items[0].x, g.world.items[0].y) = (4, 1);
    assert!(g.plan_carry((4, 1), (1, 1), (6, 3), "cotton_bolls").unwrap().0.iter().all(|s| !s.1), "back into the corner needs no pull");
    (g.world.items[0].x, g.world.items[0].y) = (1, 1);
    g.run_ticks(600);
    let loops = g.monsters[0].loops;
    assert!((3..=12).contains(&loops), "each round trip pays one slow pull: {loops} loops in 600 ticks");
}

#[test]
fn tab_turns_the_card_page_while_examining_and_switches_monsters_otherwise() {
    let mut g = Game::new();
    let (mut req, mut sfx) = (Vec::new(), Vec::new());
    let examine = Input { inspect: true, ..Input::default() };
    g.update(&examine, &mut req, &mut sfx);
    g.update(&Input { tab_pressed: true, ..examine }, &mut req, &mut sfx);
    assert_eq!((g.card_page, g.controlled), (1, 0), "Tab with E held turns the page");
    g.update(&Input::default(), &mut req, &mut sfx);
    g.update(&examine, &mut req, &mut sfx);
    assert_eq!(g.card_page, 0, "a new look starts on page 1");
    g.update(&Input { tab_pressed: true, ..Input::default() }, &mut req, &mut sfx);
    assert_eq!(g.controlled, 1, "without E, Tab switches monsters");
}

#[test]
fn two_logs_on_the_gate_bench_make_a_wooden_tub_with_the_towns_lent_tools() {
    let mut g = Game::new();
    let log = |d: &Data| ItemData { good: "oak_log".into(), quality: 0.8, carried_tags: Vec::new(), ..boll(d) };
    g.world.items.push(Item::new(40, 8, log(&g.data)));
    g.world.items.push(Item::new(41, 8, log(&g.data)));
    g.controlled = 1;
    stand(&mut g, (42, 8), Dir::Down);
    assert_eq!(crate::inspect::card(&g).title, "WORKBENCH: WOODEN TUB");
    g.script("u");
    g.run_ticks(20);
    let tub = g.world.items.iter().find(|it| it.good() == "wooden_tub").expect("a tub on the output mat");
    assert!(tub.data.carried_tags.contains(&"oak".to_string()), "an oak tub");
}

#[test]
fn a_workshop_makes_the_recipe_that_uses_the_most_of_its_pads() {
    let mut g = Game::new();
    let bench = g.world.shop_at_spot(42, 8).unwrap();
    g.world.shops[bench].pads.extend([(43, 8), (44, 8)]); // four pads for the test
    let unit = |d: &Data, id: &str| ItemData { good: id.into(), carried_tags: Vec::new(), ..boll(d) };
    for (k, id) in ["safflower_cakes", "lye", "plums", "elder_sticks"].iter().enumerate() {
        let (x, y) = g.world.shops[bench].pads[k];
        g.world.items.push(Item::new(x, y, unit(&g.data, id)));
    }
    let pink_fits = g.best_recipe(bench, &["pink_dye"], &|_| true).is_some();
    assert!(pink_fits, "pink dye's cakes and fuel are on the pads too");
    let best = g.best_recipe(bench, &["pink_dye", "crimson_dye"], &|_| true).map(|(r, _)| r);
    assert_eq!(best.as_deref(), Some("crimson_dye"), "all four ingredients make crimson, not pink from two of them");
}

/// Puts goods on a workshop's pads, has Bruna work it, and lists what came out.
fn work(g: &mut Game, spot: (i32, i32), goods: &[&str]) -> Vec<String> {
    let s = g.world.shop_at_spot(spot.0, spot.1).expect("a workshop there");
    let pads = g.world.shops[s].pads.clone();
    for (k, id) in goods.iter().enumerate() {
        g.world.items.push(Item::new(pads[k].0, pads[k].1, good_like(&g.data, id, 0.8)));
    }
    g.controlled = 1;
    stand(g, spot, Dir::Up);
    g.script("u");
    g.run_ticks(10);
    let shop = &g.world.shops[s];
    let mut out: Vec<String> = shop.outs.iter().filter_map(|&(x, y)| g.world.item_at(x, y)).map(|i| g.world.items[i].good().to_string()).collect();
    out.extend(shop.pending.iter().map(|d| d.good.clone()));
    out
}

#[test]
fn tanglemere_dyes_cotton_indigo_from_kiln_to_cloth_hall() {
    let mut g = Game::new();
    let t = |x: i32, y: i32| (125 + x, y); // Tanglemere's map starts at column 125
    let (kiln, vats, dye_house, hall) = (t(7, 21), t(29, 2), t(16, 4), t(39, 4));
    assert!(work(&mut g, kiln, &["ramie_stalks", "elder_sticks"]).contains(&"lye".to_string()), "ramie ash and fuel make lye");
    assert!(work(&mut g, kiln, &["limestone", "elder_sticks"]).contains(&"quicklime".to_string()), "the same kiln burns limestone");
    let indigo = work(&mut g, vats, &["indigofera_leaves", "lye", "quicklime", "quicklime", "elder_sticks"]);
    assert!(indigo.iter().filter(|o| *o == "indigo").count() >= 3, "one batch of indigo is about three: {indigo:?}");
    assert!(work(&mut g, dye_house, &["fabric", "indigo"]).contains(&"dyed_fabric".to_string()), "with the town's tub");

    let k = g.world.items.iter().position(|it| it.good() == "dyed_fabric").unwrap();
    let tags = g.world.items[k].data.carried_tags.clone();
    assert!(tags.contains(&"indigo".to_string()) && tags.contains(&"cotton".to_string()), "indigo cotton: {tags:?}");
    (g.world.items[k].x, g.world.items[k].y) = (hall.0, hall.1 + 1);
    stand(&mut g, (hall.0, hall.1 + 2), Dir::Up);
    let renown = g.renown;
    g.script("U");
    assert!(g.renown > renown, "the cloth hall takes dyed fabric for renown");
    assert!(g.world.bins.iter().any(|b| b.stock.iter().any(|d| d.good == "dyed_fabric")));
}

#[test]
fn space_records_the_thing_it_used_and_pushing_records_a_goal() {
    let mut g = Game::new();
    g.script(&format!("r{MOGG_HARVEST}"));
    let goals = &g.monsters[0].rec.as_ref().unwrap().goals;
    assert_eq!(goals[0], Goal::Activate(Thing::Plant { at: (8, 8), facing: Dir::Right }), "the walk there isn't recorded, only the plant");
    assert_eq!(goals[1], Goal::Move { good: "cotton_bolls".into(), from: (9, 8), to: (12, 8) }, "pushing the boll into the bin is one goal");
    assert_eq!(goals.len(), 2);
}

#[test]
fn hauling_pays_one_share_when_a_good_is_used_however_far_it_was_pushed() {
    let mut g = Game::new();
    g.world.items.push(Item::new(41, 27, good_like(&g.data, "fabric", 0.8)));
    stand(&mut g, (40, 27), Dir::Right);
    g.script("RR<R<R<RR"); // pushed back and forth: six pushes, three pulls
    let k = g.world.items.iter().position(|it| it.good() == "fabric").unwrap();
    assert_eq!(g.world.items[k].data.haul, vec![(1, 9)], "nine tiles, all Mogg's");
    assert_eq!((g.world.items[k].x, g.world.items[k].y), (44, 27));
    g.script("URD"); // around, then down into the cloth store
    let stored = g.world.bins.iter().flat_map(|b| &b.stock).find(|d| d.good == "fabric").expect("delivered");
    let mogg = stored.shares.iter().find(|s| s.monster == 1).map_or(0.0, |s| s.count);
    assert!((mogg - 1.0).abs() < 1e-9, "one hauling share, not ten: {mogg}");
    assert!(stored.haul.is_empty() && g.monsters[0].earned > 0.0);
}

#[test]
fn twelve_logs_of_any_wood_build_the_bridge() {
    let mut g = Game::new();
    let b = g.world.boards.iter().position(|b| b.project == crate::world::Project::Bridge).expect("a bridge board");
    let (bx, by) = (g.world.boards[b].x, g.world.boards[b].y);
    let deliver = |g: &mut Game, id: &str| {
        g.world.items.push(Item::new(bx + 1, by, ItemData { good: id.into(), carried_tags: Vec::new(), ..boll(&g.data) }));
        stand(g, (bx + 2, by), Dir::Left);
        g.script("L");
    };
    deliver(&mut g, "fabric");
    assert_eq!(g.world.boards[b].have, 0, "fabric isn't a log");
    g.world.items.clear();
    deliver(&mut g, "hemlock_log");
    g.world.boards[b].have = 11;
    assert!(!g.world.bridge_built());
    deliver(&mut g, "oak_log");
    assert!(g.world.bridge_built(), "the twelfth log builds it");
    let &(x, y) = g.world.bridge.first().unwrap();
    stand(&mut g, (x - 1, y + 1), Dir::Right);
    g.script("RRRRRRR");
    assert_eq!(g.monsters[g.controlled].x, x + 6, "walked across the river");
}

/// Mogg beside the town wagon's pumpkin, facing it.
fn at_the_wagon(g: &mut Game) -> ((i32, i32), (i32, i32)) {
    let w = &g.world.wagons[0];
    let (bed, pumpkin) = (w.at, w.pumpkin);
    stand(g, (pumpkin.0 + 1, pumpkin.1), Dir::Left);
    (bed, pumpkin)
}

#[test]
fn a_led_wagon_follows_takes_one_good_and_unloads_out_its_tailgate() {
    let mut g = Game::new();
    let (bed, pumpkin) = at_the_wagon(&mut g);
    g.script("u");
    assert_eq!(g.world.wagons[0].leader, Some(1), "Mogg leads the pumpkin");
    g.script("U");
    assert_eq!((g.world.wagons[0].pumpkin, g.world.wagons[0].at), ((pumpkin.0 + 1, pumpkin.1), pumpkin), "pumpkin and wagon follow, snake-like");
    g.script("D"); // into the pumpkin: blocked
    assert_eq!(g.monsters[0].y, pumpkin.1 - 1);
    g.script("u");
    assert!(g.world.wagons[0].leader.is_none(), "Space with nothing else to do lets go");

    // Load it from behind, open the tailgate, step away: it unloads.
    let w = &g.world.wagons[0];
    let back = w.behind();
    let (dx, dy) = (back.0 - w.at.0, back.1 - w.at.1);
    let dir = if dx > 0 { Dir::Left } else if dx < 0 { Dir::Right } else if dy > 0 { Dir::Up } else { Dir::Down };
    g.world.items.push(Item::new(back.0, back.1, good_like(&g.data, "fabric", 0.8)));
    g.controlled = 1;
    stand(&mut g, (back.0 + dx, back.1 + dy), dir);
    let key = match dir { Dir::Left => "L", Dir::Right => "R", Dir::Up => "U", Dir::Down => "D" };
    g.script(key);
    assert_eq!(g.world.wagons[0].stock.len(), 1, "pushed into the bed");
    g.world.items.push(Item::new(back.0, back.1, good_like(&g.data, "thread", 0.8)));
    stand(&mut g, (back.0 + dx, back.1 + dy), dir);
    g.script(key);
    assert!(g.monsters[1].bubble.as_ref().is_some_and(|b| b.text.contains("CARRYING FABRIC")), "one kind of good at a time");
    g.world.items.retain(|it| it.good() != "thread");
    stand(&mut g, back, dir);
    g.script("u");
    assert!(g.world.wagons[0].open);
    let away = match dir { Dir::Left => "R", Dir::Right => "L", Dir::Up => "D", Dir::Down => "U" };
    g.script(away);
    g.run_ticks(10);
    assert!(g.world.wagons[0].stock.is_empty() && g.world.item_at(back.0, back.1).is_some(), "set out behind it");
    let _ = bed;
}

#[test]
fn a_wagon_wont_fit_the_footbridge_and_a_heavy_load_slows_it() {
    let mut g = Game::new();
    g.world.build_footbridge();
    let &(west, y) = g.world.footbridge.first().unwrap();
    {
        let w = &mut g.world.wagons[0];
        (w.at, w.pumpkin, w.leader) = ((west - 3, y), (west - 2, y), Some(1));
    }
    stand(&mut g, (west - 1, y), Dir::Right);
    g.script("R");
    assert_eq!(g.monsters[0].x, west - 1, "stays on the bank");
    assert!(g.monsters[0].bubble.as_ref().is_some_and(|b| b.text.contains("DON'T FIT")));

    assert_eq!(g.wagon_ticks(0, 0), PUSH_TICKS, "empty, it rolls at push speed");
    let log = ItemData { good: "oak_log".into(), carried_tags: Vec::new(), ..boll(&g.data) };
    g.world.wagons[0].stock = vec![(log, "PINEWATER".into()); 6];
    assert!(g.wagon_ticks(0, 0) > PUSH_TICKS, "six oak logs are more than Mogg's CHA pulls at full speed");
}

#[test]
fn a_wagon_journey_pays_its_leader_two_shares() {
    let mut g = Game::new();
    let t = |x: i32, y: i32| (125 + x, y);
    {
        let w = &mut g.world.wagons[0];
        (w.at, w.pumpkin, w.open, w.last_leader) = (t(20, 13), t(21, 13), true, Some(1));
        w.stock = vec![(good_like(&g.data, "fabric", 0.8), "PINEWATER".into())];
    }
    g.run_ticks(10);
    let k = g.world.items.iter().position(|it| it.good() == "fabric").expect("unloaded in Tanglemere");
    assert_eq!(g.world.items[k].data.carted, Some(1), "brought from Pinewater by Mogg");
    let mut data = g.world.items[k].data.clone();
    crate::econ::pay_haul(&mut data);
    assert!(data.shares.iter().any(|s| s.monster == 1 && (s.count - 2.0).abs() < 1e-9), "two shares when it's used");
}

#[test]
fn pushing_teaches_hauling() {
    let mut g = Game::new();
    g.world.items.push(Item::new(30, 13, good_like(&g.data, "fabric", 0.8)));
    stand(&mut g, (29, 13), Dir::Right);
    let before = g.monsters[0].skills.applied("hauling");
    g.script("RRRRRR");
    g.world.items.clear();
    assert!(g.monsters[0].skills.applied("hauling") > before);
}

#[test]
fn a_recorded_wagon_loop_leads_it_there_and_back_and_lets_go() {
    let mut g = Game::new();
    let (bed, pumpkin) = at_the_wagon(&mut g);
    // Lead it up and along, let go; take the lead again, bring it back round, let go.
    g.script("ruULLuuLDRRRur");
    assert_eq!((g.world.wagons[0].at, g.world.wagons[0].pumpkin), (bed, pumpkin), "back where it started");
    let there = Goal::Lead { wagon: 0, bed: (pumpkin.0 + 1, pumpkin.1 - 1), pumpkin: (pumpkin.0, pumpkin.1 - 1) };
    let back = Goal::Lead { wagon: 0, bed, pumpkin };
    assert_eq!(g.monsters[0].routine.as_ref().unwrap().goals, vec![there, back], "two journeys: where it was led each time");
    assert!(g.world.wagons[0].leader.is_none());
    g.script("p");
    g.run_ticks(1500);
    assert!(g.monsters[0].playing && g.monsters[0].loops >= 2, "looped {} times: {}", g.monsters[0].loops, g.monsters[0].wait_reason);
}

#[test]
fn space_turns_to_a_pumpkin_or_tailgate_beside_you() {
    let mut g = Game::new();
    let (bed, pumpkin) = (g.world.wagons[0].at, g.world.wagons[0].pumpkin);
    stand(&mut g, (pumpkin.0 + 1, pumpkin.1), Dir::Up); // beside the pumpkin, facing away
    g.script("u");
    assert_eq!(g.world.wagons[0].leader, Some(1), "Space found the pumpkin");
    assert_eq!(g.monsters[0].facing, Dir::Left);
    g.script("u");
    assert!(g.world.wagons[0].leader.is_none());

    let back = g.world.wagons[0].behind();
    stand(&mut g, back, Dir::Down); // at the tailgate, facing along the road
    g.script("u");
    assert!(g.world.wagons[0].open, "Space found the tailgate");
    let _ = bed;
}

#[test]
fn a_run_of_pushes_records_as_one_goal_even_around_a_corner() {
    let mut g = Game::new();
    g.world.items.push(Item::new(31, 13, good_like(&g.data, "fabric", 0.8)));
    stand(&mut g, (30, 13), Dir::Right);
    g.script("rRRR"); // push it east to (34,13)
    g.script("URD"); // walk around to above it, then push it south
    g.script("D");
    let goals = &g.monsters[0].rec.as_ref().unwrap().goals;
    assert_eq!(goals, &vec![Goal::Move { good: "fabric".into(), from: (31, 13), to: (34, 15) }], "one goal, with the walk around dropped");
}

#[test]
fn a_goal_already_met_needs_nothing_and_a_blocked_one_goes_around() {
    let mut g = Game::new();
    // Already there: slivers sit on the spinnery's pad, so "move them onto the pad" isn't wanted.
    g.world.items.push(Item::new(30, 9, good_like(&g.data, "cotton_slivers", 0.8)));
    g.world.items.push(Item::new(15, 9, good_like(&g.data, "cotton_slivers", 0.8)));
    start_loop(&mut g, 0, Routine { flag: (14, 10), goals: vec![Goal::Move { good: "cotton_slivers".into(), from: (15, 9), to: (30, 9) }] }, (14, 10));
    g.run_ticks(60);
    assert!(g.world.item_at(15, 9).is_some(), "the other slivers stay put");

    // In the way: a bolt sits on the straight line, so the plan pushes around it.
    let mut g = Game::new();
    g.world.items.push(Item::new(31, 13, good_like(&g.data, "fabric", 0.8)));
    g.world.items.push(Item::new(34, 13, good_like(&g.data, "thread", 0.8)));
    stand(&mut g, (30, 13), Dir::Right);
    let m = &mut g.monsters[0];
    m.routine = Some(Routine { flag: (30, 13), goals: vec![Goal::Move { good: "fabric".into(), from: (31, 13), to: (37, 13) }] });
    (m.pc, m.playing) = (0, true);
    g.run_ticks(300);
    assert!(g.world.item_at(37, 13).is_some_and(|k| g.world.items[k].good() == "fabric"), "the fabric got past the thread");
    assert_eq!(g.world.item_at(34, 13).map(|k| g.world.items[k].good()), Some("thread"), "and the thread wasn't shoved aside");
}

#[test]
fn a_goal_can_end_in_a_bin() {
    let mut g = Game::new();
    g.world.items.push(Item::new(40, 27, good_like(&g.data, "fabric", 0.8)));
    stand(&mut g, (38, 27), Dir::Right);
    let m = &mut g.monsters[0];
    m.routine = Some(Routine { flag: (38, 27), goals: vec![Goal::Move { good: "fabric".into(), from: (40, 27), to: (44, 28) }] });
    (m.pc, m.playing) = (0, true);
    g.run_ticks(300);
    assert!(g.world.bins.iter().any(|b| b.stock.iter().any(|d| d.good == "fabric")), "pushed into the cloth store");
}

#[test]
fn a_slip_put_right_while_recording_records_nothing() {
    let mut g = Game::new();
    g.world.items.push(Item::new(31, 13, good_like(&g.data, "fabric", 0.8)));
    stand(&mut g, (30, 13), Dir::Right);
    g.script("rR"); // oops, pushed it one too far
    g.script("URRD"); // walk around to its other side...
    g.script("L"); // ...and push it back
    let goals = &g.monsters[0].rec.as_ref().unwrap().goals;
    assert!(goals.is_empty(), "no goal left for a good put back where it was: {goals:?}");
}

#[test]
fn the_next_good_waits_ready_beside_a_pad_still_being_worked() {
    let mut g = Game::new();
    // Mogg weaves the first thread at the loom...
    g.world.items.push(Item::new(44, 21, good_like(&g.data, "thread", 0.8)));
    stand(&mut g, (43, 21), Dir::Right);
    g.update(&Input { use_pressed: true, ..Input::default() }, &mut Vec::new(), &mut Vec::new());
    let pad = |g: &Game| g.world.item_at(44, 21).map(|k| (g.world.items[k].good().to_string(), g.world.items[k].locked));
    assert_eq!(pad(&g), Some(("thread".into(), true)), "the loom is busy with it");
    // ...while Bruna's loop brings the next one.
    g.world.items.push(Item::new(44, 17, good_like(&g.data, "thread", 0.8)));
    let m = &mut g.monsters[1];
    (m.x, m.y, m.facing) = (44, 16, Dir::Down);
    m.routine = Some(Routine { flag: (44, 16), goals: vec![Goal::Move { good: "thread".into(), from: (44, 17), to: (44, 21) }] });
    (m.pc, m.playing) = (0, true);
    let mut ready = None;
    for _ in 0..2000 {
        if !pad(&g).is_some_and(|(_, locked)| locked) {
            break;
        }
        if g.monsters[1].wait_reason.starts_with("READY") {
            let next = g.world.items.iter().find(|it| it.good() == "thread" && !it.locked).unwrap();
            ready = Some(((next.x, next.y), (g.monsters[1].x, g.monsters[1].y)));
        }
        g.run_ticks(1);
    }
    assert_eq!(ready, Some(((44, 20), (44, 19))), "waited right up beside the pad, Bruna behind it, while the weaving ran");
    g.run_ticks(8);
    assert_eq!(pad(&g), Some(("thread".into(), false)), "in the moment the weaving's done");
}

#[test]
fn a_stray_on_the_goal_is_shoved_aside_and_the_good_goes_in() {
    let mut g = Game::new();
    g.world.items.push(Item::new(44, 21, good_like(&g.data, "fabric", 0.8))); // left on the loom pad
    g.world.items.push(Item::new(44, 17, good_like(&g.data, "thread", 0.8)));
    start_loop(&mut g, 0, Routine { flag: (44, 16), goals: vec![Goal::Move { good: "thread".into(), from: (44, 17), to: (44, 21) }] }, (44, 16));
    g.run_ticks(600);
    assert_eq!(g.world.item_at(44, 21).map(|k| g.world.items[k].good()), Some("thread"), "{}", g.monsters[0].wait_reason);
    assert!(g.world.items.iter().any(|it| it.good() == "fabric" && (it.x, it.y) != (44, 21)), "the fabric moved aside, not gone");
}

#[test]
fn a_push_goes_around_a_monster_standing_in_the_way() {
    let mut g = Game::new();
    g.world.items.push(Item::new(31, 13, good_like(&g.data, "fabric", 0.8)));
    stand(&mut g, (30, 13), Dir::Right);
    (g.monsters[1].x, g.monsters[1].y) = (34, 13); // standing still on the straight line
    let m = &mut g.monsters[0];
    m.routine = Some(Routine { flag: (30, 13), goals: vec![Goal::Move { good: "fabric".into(), from: (31, 13), to: (37, 13) }] });
    (m.pc, m.playing) = (0, true);
    g.run_ticks(300);
    assert!(g.world.item_at(37, 13).is_some_and(|k| g.world.items[k].good() == "fabric"), "the fabric got past the monster");
    assert_eq!((g.monsters[1].x, g.monsters[1].y), (34, 13), "who never had to move");
}

#[test]
fn r_away_from_the_flag_walks_home_and_saves_the_loop() {
    let mut g = Game::new();
    g.world.items.push(Item::new(22, 13, good_like(&g.data, "fabric", 0.8)));
    stand(&mut g, (20, 13), Dir::Right);
    g.script("rRRRr");
    assert!(g.monsters[0].homing, "heading back on its own");
    g.run_ticks(60);
    let m = &g.monsters[0];
    assert_eq!((m.x, m.y), (20, 13), "home at the flag");
    assert!(m.rec.is_none() && !m.homing);
    assert_eq!(m.routine.as_ref().unwrap().goals, vec![Goal::Move { good: "fabric".into(), from: (22, 13), to: (24, 13) }], "the walk home isn't a goal");

    // Steering it on the way takes over.
    let mut g = Game::new();
    stand(&mut g, (20, 13), Dir::Right);
    g.world.items.push(Item::new(21, 13, good_like(&g.data, "fabric", 0.8)));
    g.script("rRRUr");
    g.script("D");
    assert!(!g.monsters[0].homing && g.monsters[0].rec.is_some(), "still recording, walked by hand now");
}

#[test]
fn a_loop_starts_from_its_flag() {
    let mut g = Game::new();
    g.world.items.push(Item::new(22, 13, good_like(&g.data, "fabric", 0.8)));
    let fabric = |from, to| Goal::Move { good: "fabric".into(), from, to };
    g.monsters[0].routine = Some(Routine { flag: (20, 13), goals: vec![fabric((22, 13), (25, 13)), fabric((25, 13), (22, 13))] });
    stand(&mut g, (20, 12), Dir::Down);
    g.script("p");
    assert!(!g.monsters[0].playing, "not from beside the flag");
    g.script("D");
    g.script("p");
    assert!(g.monsters[0].playing, "from the flag");
    g.run_ticks(100);
    g.script("p");
    let m = &g.monsters[0];
    assert!(!m.playing && (m.x, m.y) != (20, 13), "stopped away from the flag");
    g.script("p");
    assert!(g.monsters[0].playing, "resumed right where it was stopped");
}

#[test]
fn p_on_another_monsters_flag_takes_up_their_loop() {
    let mut g = Game::new();
    g.world.items.push(Item::new(22, 13, good_like(&g.data, "fabric", 0.8)));
    let fabric = |from, to| Goal::Move { good: "fabric".into(), from, to };
    let loop_ = Routine { flag: (20, 13), goals: vec![fabric((22, 13), (25, 13)), fabric((25, 13), (22, 13))] };
    g.monsters[1].routine = Some(loop_.clone());
    stand(&mut g, (20, 13), Dir::Up);
    g.script("p");
    let m = &g.monsters[0];
    assert!(m.playing);
    assert_eq!((m.routine.as_ref().unwrap().flag, &m.routine.as_ref().unwrap().goals), (loop_.flag, &loop_.goals), "the same flag and loop");
    g.run_ticks(200);
    assert!(g.monsters[0].loops >= 1, "and it runs");
}

#[test]
fn a_recorded_pull_and_space_replay_as_a_move_and_working_the_bench() {
    // Pull the boll off the chute onto the workbench's pad, backing onto the bench's spot, and
    // use the bench. Replayed, the planner pushes the boll from the chute's far side instead,
    // ending somewhere else, and the loop walks to the bench's spot to work it.
    let mut g = Game::new();
    g.world.items.push(Item::new(12, 9, boll(&g.data)));
    stand(&mut g, (13, 9), Dir::Left);
    g.script("r>u");
    g.run_ticks(60);
    g.script("r");
    g.run_ticks(30); // back to the flag, a step away, and saved
    let bench = g.world.shop_at_spot(14, 9).unwrap();
    let m = &g.monsters[0];
    assert_eq!(m.routine.as_ref().unwrap().goals, vec![Goal::Move { good: "cotton_bolls".into(), from: (12, 9), to: (13, 9) }, Goal::Activate(Thing::Shop(bench))]);
    g.world.items.retain(|it| it.good() != "cotton_slivers");
    g.world.shops[bench].pending.clear();
    g.world.items.push(Item::new(12, 9, boll(&g.data)));
    let (plan, _) = g.plan_carry((12, 9), (13, 9), (13, 9), "cotton_bolls").unwrap();
    assert!(plan.iter().all(|s| !s.1), "replayed as a push, not the recorded pull: {plan:?}");
    (g.monsters[1].x, g.monsters[1].y) = (20, 13); // Bruna, off out of the way
    g.script("p");
    g.run_ticks(200);
    assert!(g.monsters[0].playing, "never lost: {:?}", g.monsters[0].bubble.as_ref().map(|b| &b.text));
    let slivers = g.world.items.iter().filter(|it| it.good() == "cotton_slivers").count() + g.world.shops[bench].pending.len();
    assert!(slivers >= 1, "the bench was used from its spot");
}

#[test]
fn walking_up_to_a_push_isnt_a_goal_so_the_next_thread_on_the_pad_doesnt_jam_the_loop() {
    // Recorded: off the loom's spot onto the empty pad, then push the fabric down.
    let mut g = Game::new();
    g.world.items.push(Item::new(44, 22, good_like(&g.data, "fabric", 0.8)));
    stand(&mut g, (43, 21), Dir::Down);
    g.script("rRD");
    let goals = &g.monsters[0].rec.as_ref().unwrap().goals;
    assert_eq!(goals, &vec![Goal::Move { good: "fabric".into(), from: (44, 22), to: (44, 23) }], "no 'go to the pad'");

    // Replayed from across town, with the next thread already on the pad.
    let mut g = Game::new();
    g.world.items.push(Item::new(44, 21, good_like(&g.data, "thread", 0.8)));
    g.world.items.push(Item::new(44, 22, good_like(&g.data, "fabric", 0.8)));
    stand(&mut g, (8, 10), Dir::Down);
    let m = &mut g.monsters[0];
    m.routine = Some(Routine { flag: (8, 10), goals: vec![Goal::Move { good: "fabric".into(), from: (44, 22), to: (44, 28) }] });
    (m.pc, m.playing) = (0, true);
    g.run_ticks(900);
    assert!(g.world.bins.iter().any(|b| b.stock.iter().any(|d| d.good == "fabric")), "the fabric got out around the thread: {}", g.monsters[0].wait_reason);
    assert_eq!(g.world.item_at(44, 21).map(|k| g.world.items[k].good()), Some("thread"), "which stayed on the pad");
}

#[test]
fn a_loop_keeps_track_of_its_good_when_someone_else_pushes_it() {
    let mut g = Game::new();
    g.world.items.push(Item::new(31, 13, good_like(&g.data, "fabric", 0.8)));
    let m = &mut g.monsters[1];
    (m.x, m.y, m.facing) = (30, 13, Dir::Right);
    m.routine = Some(Routine { flag: (30, 13), goals: vec![Goal::Move { good: "fabric".into(), from: (31, 13), to: (38, 13) }] });
    (m.pc, m.playing) = (0, true);
    for _ in 0..200 {
        if g.monsters[1].carry == Some((33, 13)) {
            break;
        }
        g.run_ticks(1);
    }
    assert_eq!(g.monsters[1].carry, Some((33, 13)), "partway there");
    // Mogg shoves it off the line while Bruna's moving it.
    stand(&mut g, (33, 12), Dir::Down);
    g.script("D");
    assert!(g.world.item_at(33, 14).is_some());
    g.script("UU"); // and gets out of the way
    g.run_ticks(300);
    assert!(g.world.item_at(38, 13).is_some_and(|k| g.world.items[k].good() == "fabric"), "Bruna followed it: {}", g.monsters[1].wait_reason);
    assert!(g.monsters[1].loops >= 1);
}

/// The spinnery's east doorway, as in Henry's world: the thread in the doorway, Bruna leading
/// the wagon right outside it, Mogg inside.
fn doorway_jam() -> Game {
    let mut g = Game::new();
    g.world.items.push(Item::new(36, 9, good_like(&g.data, "thread", 0.8)));
    (g.monsters[0].x, g.monsters[0].y) = (35, 9);
    (g.monsters[1].x, g.monsters[1].y) = (37, 9);
    let id = g.monsters[1].id;
    let w = &mut g.world.wagons[0];
    (w.pumpkin, w.at, w.leader) = ((38, 9), (39, 9), Some(id));
    g
}

#[test]
fn blocked_in_a_doorway_by_a_monster_standing_still_a_loop_goes_the_long_way_round() {
    // Bruna, leading the wagon, has stopped right outside the door: she's standing still, so
    // Mogg takes the thread out the west door and round the spinnery instead of waiting.
    let mut g = doorway_jam();
    let m = &mut g.monsters[0];
    m.routine = Some(Routine { flag: (30, 9), goals: vec![Goal::Move { good: "thread".into(), from: (36, 9), to: (44, 21) }] });
    (m.pc, m.playing) = (0, true);
    g.run_ticks(2000);
    assert!(g.world.item_at(44, 21).is_some_and(|k| g.world.items[k].good() == "thread"), "then goes round: {}", g.monsters[0].wait_reason);
    assert_eq!((g.monsters[1].x, g.monsters[1].y), (37, 9), "Bruna never moved");
}

#[test]
fn a_hopeless_plan_out_in_the_open_gives_up_quickly() {
    // The whole map is searched, so a good with nowhere to go tries everywhere first.
    let mut g = Game::new();
    g.world.items.push(Item::new(20, 13, good_like(&g.data, "thread", 0.8)));
    for door in [(44, 19), (44, 25)] {
        g.world.items.push(Item::new(door.0, door.1, good_like(&g.data, "fabric", 0.8)));
    }
    let t = std::time::Instant::now();
    assert!(g.plan_carry((20, 13), (44, 21), (19, 13), "thread").is_none());
    assert!(t.elapsed().as_millis() < 50, "well within a frame: {:?}", t.elapsed());
}

#[test]
fn a_wagon_loop_fetches_the_wagon_from_wherever_it_was_left() {
    let mut g = Game::new();
    let (pumpkin, bed) = (g.world.wagons[0].pumpkin, g.world.wagons[0].at);
    (g.world.wagons[0].pumpkin, g.world.wagons[0].at) = ((pumpkin.0 - 10, pumpkin.1), (bed.0 - 10, bed.1));
    start_loop(&mut g, 0, Routine { flag: (pumpkin.0 + 1, pumpkin.1), goals: vec![Goal::Lead { wagon: 0, bed, pumpkin }] }, (pumpkin.0 + 1, pumpkin.1));
    g.run_ticks(600);
    assert_eq!((g.world.wagons[0].at, g.world.wagons[0].pumpkin), (bed, pumpkin), "led back where it's parked: {}", g.monsters[0].wait_reason);
    assert!(g.world.wagons[0].leader.is_none(), "and let go");
}

#[test]
fn several_monsters_off_screen_in_one_corner_still_draw() {
    // Their edge markers all start in the same corner; finding each a spot used to loop forever.
    let mut g = Game::new();
    for k in 0..4 {
        let skills = g.monsters[0].skills.clone();
        g.monsters.push(Monster::new(10 + k, "TEST", "goblin", &g.data, (60 + k as i32, 30), skills));
    }
    stand(&mut g, (5, 5), Dir::Down);
    let (sp, mut fb) = (crate::sprites::Sprites::new(), crate::framebuffer::Framebuffer::new());
    crate::render::draw(&g, &sp, &mut fb);
}

/// Harvest the plant at (8,8) facing the cotton bin, and push `pushes` bolls into the bin.
fn harvest_loop(pushes: usize) -> Routine {
    let mut goals = vec![Goal::Activate(Thing::Plant { at: (8, 8), facing: Dir::Right })];
    goals.extend((0..pushes).map(|_| Goal::Move { good: "cotton_bolls".into(), from: (9, 8), to: (12, 8) }));
    Routine { flag: (8, 10), goals }
}

fn start_loop(g: &mut Game, i: usize, r: Routine, at: (i32, i32)) {
    let m = &mut g.monsters[i];
    (m.x, m.y) = at;
    m.routine = Some(r);
    (m.pc, m.playing) = (0, true);
}

fn binned(g: &Game, good: &str) -> usize {
    g.world.bins.iter().flat_map(|b| &b.stock).filter(|d| d.good == good).count()
}

#[test]
fn a_good_a_loop_goes_for_is_reserved_and_other_loops_leave_it() {
    let mut g = Game::new();
    g.world.items.push(Item::new(9, 8, boll(&g.data)));
    let one_push = || Routine { flag: (10, 10), goals: vec![Goal::Move { good: "cotton_bolls".into(), from: (9, 8), to: (12, 8) }] };
    start_loop(&mut g, 0, one_push(), (8, 10));
    g.run_ticks(1);
    let id = g.world.items[g.world.item_at(9, 8).unwrap()].id;
    assert!(g.monsters[0].holds.contains(&crate::sched::Hold::Item(id)), "Mogg is going for it");
    start_loop(&mut g, 1, one_push(), (10, 9));
    g.run_ticks(1);
    assert!(g.monsters[1].holds.is_empty() && g.monsters[1].waiting_on == Some(0), "Bruna waits on Mogg rather than go for the same boll");
    g.run_ticks(200);
    // In the bin, or already set out of its chute.
    assert_eq!(binned(&g, "cotton_bolls") + g.world.item_at(12, 9).iter().count(), 1);
}

#[test]
fn two_monsters_on_one_harvest_loop_never_wait_on_each_other() {
    // The deadlock Henry hit: both took one boll of a harvest, then waited at the second push
    // for a boll only their own harvest step makes.
    let mut g = Game::new();
    let p = g.world.plant_at(8, 8).unwrap();
    g.world.plants[p].regrow = 40;
    start_loop(&mut g, 0, harvest_loop(2), (8, 10));
    start_loop(&mut g, 1, harvest_loop(2), (7, 10));
    g.run_ticks(5000);
    let (a, b) = (&g.monsters[0], &g.monsters[1]);
    assert!(a.loops >= 3 && b.loops >= 3, "both keep going: {} ({}), {} ({})", a.loops, a.wait_reason, b.loops, b.wait_reason);
}

#[test]
fn a_harvest_short_of_bolls_is_made_up_by_harvesting_again() {
    let mut g = Game::new();
    let p = g.world.plant_at(8, 8).unwrap();
    g.world.plants[p].regrow = 20;
    start_loop(&mut g, 0, harvest_loop(3), (8, 10));
    for _ in 0..4000 {
        // Every harvest yields just the one boll that pops straight out.
        if g.world.item_at(9, 8).is_some() {
            g.world.plants[p].stock.clear();
        }
        g.run_ticks(1);
    }
    assert!(g.monsters[0].loops >= 2, "three pushes a lap, harvesting as often as it takes: {} ({})", g.monsters[0].loops, g.monsters[0].wait_reason);
    assert!(binned(&g, "cotton_bolls") >= 6);
}

#[test]
fn a_good_taken_by_hand_is_made_again() {
    let mut g = Game::new();
    let p = g.world.plant_at(8, 8).unwrap();
    g.world.plants[p].regrow = 20;
    start_loop(&mut g, 0, harvest_loop(1), (8, 10));
    for _ in 0..400 {
        if g.world.item_at(9, 8).is_some() {
            break;
        }
        g.run_ticks(1);
    }
    g.world.plants[p].stock.clear();
    // Bruna, steered by hand, shoves the boll off before Mogg pushes it.
    g.controlled = 1;
    stand(&mut g, (9, 7), Dir::Down);
    g.script("DU");
    (g.monsters[1].x, g.monsters[1].y) = (20, 13); // and off out of the way
    g.run_ticks(600);
    assert!(binned(&g, "cotton_bolls") >= 1, "Mogg got one into the bin: {}", g.monsters[0].wait_reason);
}

#[test]
fn stopping_a_loop_gives_up_what_it_reserved() {
    let mut g = Game::new();
    g.world.items.push(Item::new(9, 8, boll(&g.data)));
    start_loop(&mut g, 0, harvest_loop(2), (8, 10));
    g.run_ticks(2);
    assert!(!g.monsters[0].holds.is_empty());
    g.script("p"); // Mogg is the one you steer
    assert!(g.monsters[0].holds.is_empty() && g.monsters[0].task.is_none());
}

/// Not a check: replays a saved world to see how its loops get on. MW_SAVE is the save's
/// path, MW_K how many thousand ticks (6 if unset). Run with
/// `cargo test --release replay_a_saved_world -- --ignored --nocapture`.
#[test]
#[ignore]
fn replay_a_saved_world() {
    let path = std::env::var("MW_SAVE").unwrap();
    let mut g = crate::save::from_json(&std::fs::read_to_string(&path).unwrap()).unwrap();
    let before: Vec<u32> = g.monsters.iter().map(|m| m.loops).collect();
    let t = std::time::Instant::now();
    let laps: usize = std::env::var("MW_K").ok().and_then(|v| v.parse().ok()).unwrap_or(6);
    for k in 0..laps {
        g.run_ticks(1000);
        let row: Vec<String> = g.monsters.iter().zip(&before).map(|(m, b)| format!("{} +{} pc{} {:?}", m.name, m.loops - b, m.pc, m.wait_reason)).collect();
        println!("{}k: {}", k + 1, row.join(" | "));
    }
    println!("{:?}", t.elapsed());
    for m in &g.monsters {
        println!("{} at ({},{}) pc {} task {:?} holds {:?} playing {} busy {}", m.name, m.x, m.y, m.pc, m.task, m.holds, m.playing, m.busy);
    }
    for it in &g.world.items {
        println!("item {} #{} ({},{}) locked {}", it.good(), it.id, it.x, it.y, it.locked);
    }
    let p = g.world.plant_at(8, 8).unwrap();
    let pl = &g.world.plants[p];
    println!("plant growth {}/{} stock {} harvest {:?} drop {:?}", pl.growth, pl.regrow, pl.stock.len(), pl.harvest.as_ref().map(|j| j.monster), pl.drop);
    for (k, m) in g.monsters.iter().enumerate() {
        println!("{} id {} k {}", m.name, m.id, k);
    }
}

#[test]
fn examining_a_good_a_monster_is_going_for_says_who() {
    let mut g = Game::new();
    g.world.items.push(Item::new(5, 10, boll(&g.data)));
    let id = g.world.item_id(g.world.item_at(5, 10).unwrap());
    g.monsters[1].holds.push(crate::sched::Hold::Item(id));
    stand(&mut g, (4, 10), Dir::Right);
    g.inspecting = true;
    let card = crate::inspect::card(&g);
    let text: Vec<String> = card.pages[0].iter().map(|l| l.text.clone()).collect();
    assert!(text.iter().any(|t| t == "BRUNA IS SEEING TO IT"), "{text:?}");
}

#[test]
fn whoever_has_waited_longest_for_the_plant_harvests_first() {
    let mut g = Game::new();
    let p = g.world.plant_at(8, 8).unwrap();
    (g.world.plants[p].growth, g.world.plants[p].regrow) = (0, 60);
    start_loop(&mut g, 0, harvest_loop(2), (3, 10)); // Mogg, the long way off, starts waiting first
    g.run_ticks(20);
    start_loop(&mut g, 1, harvest_loop(2), (8, 9)); // Bruna, right beside it, comes later
    for _ in 0..200 {
        if g.world.plants[p].harvest.is_some() {
            break;
        }
        g.run_ticks(1);
    }
    assert_eq!(g.world.plants[p].harvest.as_ref().map(|j| j.monster), Some(0), "Mogg's turn: {:?}", g.monsters[1].wait_reason);
}

#[test]
fn a_place_in_line_held_by_one_busy_elsewhere_doesnt_make_anyone_wait() {
    // Bruna joined the line for the plant earlier, then went off to work across the village:
    // Mogg harvests rather than say "AFTER YOU, BRUNA." and wait for someone who can't come.
    let mut g = Game::new();
    let p = g.world.plant_at(8, 8).unwrap();
    g.frame = 100;
    start_loop(&mut g, 0, harvest_loop(1), (8, 9));
    start_loop(&mut g, 1, harvest_loop(1), (40, 20));
    (g.monsters[1].turns, g.monsters[1].busy) = (vec![((8, 8), 0)], true);
    for _ in 0..60 {
        if g.world.plants[p].harvest.is_some() {
            break;
        }
        g.run_ticks(1);
    }
    assert_eq!(g.world.plants[p].harvest.as_ref().map(|j| j.monster), Some(0), "Mogg harvests: {}", g.monsters[0].wait_reason);
}

#[test]
fn waits_say_who_and_where_so_they_make_sense_from_anywhere() {
    // Mogg waits across town for bolls at the chute, and for the bench Wren is using.
    let mut g = Game::new();
    let bench = g.world.shop_at_spot(14, 9).unwrap();
    let chute_to_pad = Goal::Move { good: "cotton_bolls".into(), from: (12, 9), to: (13, 9) };
    start_loop(&mut g, 0, Routine { flag: (30, 13), goals: vec![chute_to_pad] }, (30, 13));
    g.run_ticks(20);
    assert_eq!(g.monsters[0].wait_reason, "WAITING FOR COTTON BOLLS AT THE COTTON BOLLS BIN'S CHUTE.");
    g.world.items.push(Item::new(13, 9, boll(&g.data)));
    g.monsters[1].holds.push(crate::sched::Hold::Tile((14, 9)));
    g.monsters[0].routine = Some(Routine { flag: (30, 13), goals: vec![Goal::Activate(Thing::Shop(bench))] });
    g.run_ticks(20);
    assert_eq!(g.monsters[0].wait_reason, format!("WAITING FOR {} AT THE WORKBENCH.", g.monsters[1].name));
}

#[test]
fn a_loop_boxed_in_by_monsters_left_standing_gives_up_for_now_and_says_who() {
    // Henry's jam: Bruna, steered by hand, stands in the loom house's south door, and someone
    // else in its north door. Mogg inside can't get the fabric out: rather than hold on to it
    // for ever, it lets go and says who's in the way.
    let mut g = Game::new();
    let skills = g.monsters[0].skills.clone();
    g.monsters.push(Monster::new(10, "ISKA", "goblin", &g.data, (44, 19), skills));
    (g.monsters[1].x, g.monsters[1].y) = (44, 25);
    g.world.items.push(Item::new(44, 22, good_like(&g.data, "fabric", 0.8)));
    start_loop(&mut g, 0, Routine { flag: (43, 22), goals: vec![Goal::Move { good: "fabric".into(), from: (44, 22), to: (44, 28) }] }, (43, 22));
    let mut said = Vec::new();
    for _ in 0..300 {
        g.run_ticks(1);
        let m = &g.monsters[0];
        if !said.contains(&m.wait_reason) {
            said.push(m.wait_reason.clone());
        }
        if m.holds.is_empty() && m.wait_reason.ends_with("IS IN THE WAY.") {
            return;
        }
    }
    panic!("never let go: {said:?}");
}

#[test]
fn monsters_sharing_one_plant_all_get_to_harvest_it() {
    let mut g = Game::new();
    let p = g.world.plant_at(8, 8).unwrap();
    g.world.plants[p].regrow = 40;
    for (k, name) in ["ISKA", "WREN"].into_iter().enumerate() {
        let skills = g.monsters[0].skills.clone();
        g.monsters.push(Monster::new(10 + k as u32, name, "goblin", &g.data, (0, 0), skills));
    }
    // Two stand near the plant, two far off.
    for (k, at) in [(9, 9), (7, 9), (3, 12), (20, 12)].into_iter().enumerate() {
        start_loop(&mut g, k, harvest_loop(2), at);
    }
    let before: Vec<f64> = g.monsters.iter().map(|m| m.skills.applied("harvesting")).collect();
    g.run_ticks(15_000);
    for (k, m) in g.monsters.iter().enumerate() {
        assert!(m.skills.applied("harvesting") > before[k], "{} never got a turn: {}", m.name, m.wait_reason);
    }
}
