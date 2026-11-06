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
        wear: 0.0,
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
        for t in 0..ticks {
            let mut inp = Input::default();
            inp.held[0] = true;
            inp.pressed[0] = t == 0;
            g.update(&[inp]);
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
    let mut g = Game::new().without_town();
    g.script("LL");
    assert_eq!((g.monsters[0].x, g.monsters[0].y), (1, 8), "the hedge stops the second step");
    g.script("RD");
    let data = boll(&g.data);
    g.world.items.push(Item::new(3, 9, data));
    g.script("R");
    let it = g.world.items.last().expect("the boll");
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
    let mut g = Game::new().without_town();
    g.script(MOGG_HARVEST);
    assert_eq!(g.communes[0].tutorial.stage, 3, "harvested and binned");
    g.script("t");
    g.run_ticks(10);
    g.script(&bruna_chain());
    assert_eq!(g.cloth_stored(), 1, "one bolt of fabric in the cloth store");
    let store = g.world.bins.iter().find(|b| b.takes(g.data.good("fabric"))).unwrap();
    assert!(store.takes(g.data.good("royal_blue_dyed_fabric")), "the cloth store takes any kind of fabric");
    assert_eq!(crate::inspect::plant_name(&g.data, "hemp_stalks"), "HEMP", "a plant with no name of its own goes by its variety");
    let fabric = &store.stock[0];
    assert_eq!(fabric.good, "fabric");
    assert!(fabric.carried_tags.contains(&"cotton".to_string()), "{:?}", fabric.carried_tags);
    assert_eq!(fabric.depth, 3, "bolls, slivers, thread, fabric");
    assert!(fabric.quality > 0.0 && fabric.value > 0.0);
    let credited = |m: u32| fabric.shares.iter().any(|s| s.monster == m && s.count > 0.0);
    assert!(credited(1) && credited(2), "both the harvester and the weaver share in it: {:?}", fabric.shares);
    assert!(g.monsters[1].skills.applied("weaving") > 0.0);
    let paid = fabric.value * RENOWN_PER_VALUE;
    assert!((g.communes[0].renown - paid).abs() < 1e-9, "the cloth store paid renown for its value");
    let earned: f64 = g.monsters.iter().map(|m| m.earned).sum();
    assert!((earned - paid).abs() < 1e-9 && g.monsters.iter().all(|m| m.earned > 0.0), "credit split between both");
    assert!(g.floaters.iter().any(|f| f.text.contains("RENOWN")), "a renown celebration rises");
    assert_eq!(g.communes[0].tutorial.stage, 7);
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
    assert!(tutorial::STAGES[g.communes[0].tutorial.stage].contains("GATHERING HALL"), "next the tutorial asks you to welcome a helper");

    let before = g.cloth_stored();
    g.run_ticks(6000);
    assert!(g.cloth_stored() >= before + 3, "only {} more fabric in 400 s", g.cloth_stored() - before);
    assert!(g.monsters.iter().filter(|m| m.commune == 0).all(|m| m.playing), "a loop stopped");
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
    let card = crate::inspect::card(&g, 0);
    assert_eq!(card.title, "COTTON BOLLS");
    assert!(card.lines().all(|l| l.meter.is_none() && l.icon != Some(crate::inspect::Icon::Quality)), "bolls have no quality, so the card doesn't show one");
    assert!(!has_line(&card, "RENOWN"), "nothing takes bolls for renown, so the card doesn't promise any");
    assert!(has_line(&card, "RAW MATERIAL") && has_line(&card, "CREDIT: MOGG"));
}

#[test]
fn examining_with_nothing_ahead_describes_your_monster() {
    let mut g = Game::new();
    g.monsters[0].facing = Dir::Left; // open grass, not the cotton plant to the east
    let card = crate::inspect::card(&g, 0);
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
    g.communes[0].controlled = 1;
    (g.monsters[1].x, g.monsters[1].y) = (30, 8);
    let card = crate::inspect::card(&g, 0);
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
    assert!(has_line(&crate::inspect::card(&g, 0), "FOR THREAD: PUT FIBER ON A PAD."));
}

#[test]
fn extra_output_waits_inside_so_loops_never_jam() {
    let mut g = Game::new();
    let bench = g.world.shop_at_spot(14, 9).unwrap();
    let extra = boll(&g.data);
    g.world.shops[bench].pending = vec![extra.clone(), extra.clone()];
    g.world.items.push(Item::new(15, 9, extra.clone())); // one already out on the mat
    g.world.items.push(Item::new(13, 9, boll(&g.data))); // a fresh boll on the pad
    g.communes[0].controlled = 1;
    (g.monsters[1].x, g.monsters[1].y) = (14, 9);
    g.script("u");
    assert!(g.communes[0].made.contains("cotton_slivers"), "the bench works even with output waiting");
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
    let card = crate::inspect::card(&g, 0);
    assert_eq!(card.title, "COTTON PLANT");
    for needle in ["RIPE", "COTTON BOLLS", "HARVESTING +"] {
        assert!(on_first_page(&card, needle), "page 1 misses {needle}: {:?}", card.pages[0].iter().map(|l| &l.text).collect::<Vec<_>>());
    }
    assert!(has_line(&card, "LEVELS OFF"));
    assert!(card.lines().all(|l| l.meter.is_none()), "cotton bolls are ungraded");
}

#[test]
fn renown_welcomes_an_applicant_and_a_new_one_arrives() {
    let mut g = Game::new().without_town();
    let stand = g.world.stands[1];
    (g.monsters[0].x, g.monsters[0].y) = (stand.0, stand.1 - 1);
    g.monsters[0].facing = Dir::Down;
    let cost = g.applicants[1].as_ref().unwrap().cost as f64;
    g.script("u");
    assert_eq!(g.monsters.len(), 2, "not enough renown, nobody joins");
    assert!(g.monsters[0].bubble.as_ref().is_some_and(|b| b.text.contains("THEY JOIN AT")));

    g.communes[0].renown = cost + 7.0;
    let name = g.applicants[1].as_ref().unwrap().name.clone();
    g.script("u");
    assert_eq!(g.monsters.len(), 3);
    assert_eq!(g.monsters[2].name, name);
    assert_eq!((g.monsters[2].x, g.monsters[2].y), stand);
    assert!((g.communes[0].renown - 7.0).abs() < 1e-9);
    assert!(g.applicants[1].is_none());
    g.run_ticks(400);
    assert!(g.applicants[1].is_none(), "nobody arrives while the newcomer still stands there");
    g.communes[0].controlled = 2;
    g.script("UL");
    g.run_ticks(400);
    assert!(g.applicants[1].is_some(), "a new applicant arrives once the stand is clear");
}

fn good_like(data: &Data, id: &str, quality: f64) -> ItemData {
    ItemData { good: id.into(), quality, carried_tags: vec!["cotton".into()], depth: 2, value: 2.0, ..boll(data) }
}

/// Puts the controlled monster at a tile, facing a way.
fn stand(g: &mut Game, (x, y): (i32, i32), facing: Dir) {
    let c = g.communes[0].controlled;
    (g.monsters[c].x, g.monsters[c].y, g.monsters[c].facing) = (x, y, facing);
}

#[test]
fn two_thread_on_the_gate_bench_twist_into_rope_by_hand() {
    let mut g = Game::new();
    let bench = g.world.shop_at_spot(42, 8).expect("the gate bench");
    assert_eq!(g.world.shops[bench].default_recipe(&g.data), "rope");
    g.communes[0].controlled = 1;
    stand(&mut g, (42, 8), Dir::Down);
    assert!(has_line(&crate::inspect::card(&g, 0), "FOR ROPE: PUT THREAD ON A PAD."), "an empty gate bench asks for thread");
    g.world.items.push(Item::new(40, 8, good_like(&g.data, "thread", 0.8)));
    g.world.items.push(Item::new(41, 8, good_like(&g.data, "thread", 0.8)));
    assert_eq!(crate::inspect::card(&g, 0).title, "WORKBENCH: ROPE");
    g.script("u");
    g.run_ticks(20);
    assert!(g.communes[0].made.contains("rope"));
    let rope = g.world.items.iter().find(|it| it.good() == "rope").expect("rope on the output mat");
    assert_eq!(rope.data.carried_tags, vec!["cotton".to_string()], "cotton rope keeps its fiber");
    assert!(g.world.items.iter().all(|it| it.good() != "thread"), "both thread were used");
}

#[test]
fn rope_for_the_board_builds_a_footbridge_that_goods_cross() {
    let mut g = Game::new();
    let fb = g.world.boards.iter().position(|b| b.project == Project::Footbridge).expect("the footbridge's board");
    let (bx, by) = (g.world.boards[fb].x, g.world.boards[fb].y);
    let (&(west, y), &(east, _)) = (g.world.footbridge.first().unwrap(), g.world.footbridge.last().unwrap());

    // Not built yet: the river stops you, and says why.
    stand(&mut g, (west - 1, y), Dir::Right);
    g.script("R");
    assert_eq!(g.monsters[0].x, west - 1, "the river stops you");
    assert!(g.monsters[0].bubble.as_ref().is_some_and(|b| b.text.contains("NEEDS ROPE")));

    g.world.boards[fb].have = g.world.boards[fb].need - 1;
    g.world.items.push(Item::new(bx, by + 1, good_like(&g.data, "rope", 0.9)));
    stand(&mut g, (bx, by + 2), Dir::Up);
    let renown = g.communes[0].renown;
    g.script("U");
    assert!(g.world.boards[fb].done() && g.world.footbridge_built(), "the last rope builds the footbridge");
    assert!(g.communes[0].renown > renown, "rope for the board earns renown");

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
    let b = g.world.items.len();
    g.world.items.push(Item::new(1, 1, boll(&g.data))); // Pinewater's top-left corner
    stand(&mut g, (2, 1), Dir::Left);
    g.script("L");
    let said = g.monsters[0].bubble.as_ref().map(|b| b.text.clone()).unwrap_or_default();
    assert!(said.contains("HOLD SHIFT"), "pushing a stuck item explains pulling: {said}");

    let before = g.frame;
    g.script(">");
    assert_eq!((g.monsters[0].x, g.world.items[b].x), (3, 2), "backed away one tile, dragging the boll");
    assert_eq!(g.monsters[0].facing, Dir::Left, "still facing what it pulls");
    assert!(g.frame - before >= PULL_TICKS as u32, "a pull takes {} ticks", g.frame - before);

    g.script("<");
    assert_eq!(g.monsters[0].x, 2, "Shift with nothing to drag is just a step");
}

#[test]
fn a_loop_pulls_out_of_a_corner_only_when_it_must_and_pays_for_it() {
    let mut g = Game::new();
    let b = g.world.items.len();
    g.world.items.push(Item::new(1, 1, boll(&g.data))); // Pinewater's top-left corner
    stand(&mut g, (6, 3), Dir::Left);
    let m = &mut g.monsters[0];
    let boll = || "cotton_bolls".to_string();
    m.routine = Some(Routine { flag: (6, 3), goals: vec![Goal::Move { good: boll(), from: (1, 1), to: (4, 1) }, Goal::Move { good: boll(), from: (4, 1), to: (1, 1) }] });
    (m.pc, m.playing) = (0, true);
    let (plan, _) = g.plan_carry((1, 1), (4, 1), (6, 3), "cotton_bolls").expect("a plan out of the corner");
    assert_eq!(plan.iter().filter(|s| s.1).count(), 1, "one pull gets it out of the corner, pushes do the rest: {plan:?}");
    (g.world.items[b].x, g.world.items[b].y) = (4, 1);
    assert!(g.plan_carry((4, 1), (1, 1), (6, 3), "cotton_bolls").unwrap().0.iter().all(|s| !s.1), "back into the corner needs no pull");
    (g.world.items[b].x, g.world.items[b].y) = (1, 1);
    g.run_ticks(600);
    let loops = g.monsters[0].loops;
    assert!((3..=12).contains(&loops), "each round trip pays one slow pull: {loops} loops in 600 ticks");
}

#[test]
fn tab_turns_the_card_page_while_examining_and_switches_monsters_otherwise() {
    let mut g = Game::new();
    let examine = Input { inspect: true, ..Input::default() };
    g.update(&[examine]);
    g.update(&[Input { tab_pressed: true, ..examine }]);
    assert_eq!((g.communes[0].card_page, g.communes[0].controlled), (1, 0), "Tab with E held turns the page");
    g.update(&[Input::default()]);
    g.update(&[examine]);
    assert_eq!(g.communes[0].card_page, 0, "a new look starts on page 1");
    g.update(&[Input { tab_pressed: true, ..Input::default() }]);
    assert_eq!(g.communes[0].controlled, 1, "without E, Tab switches monsters");
}

#[test]
fn two_logs_on_the_gate_bench_make_a_wooden_tub_with_the_towns_lent_tools() {
    let mut g = Game::new();
    let log = |d: &Data| ItemData { good: "oak_log".into(), quality: 0.8, carried_tags: Vec::new(), ..boll(d) };
    g.world.items.push(Item::new(40, 8, log(&g.data)));
    g.world.items.push(Item::new(41, 8, log(&g.data)));
    g.communes[0].controlled = 1;
    stand(&mut g, (42, 8), Dir::Down);
    assert_eq!(crate::inspect::card(&g, 0).title, "WORKBENCH: WOODEN TUB");
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
    g.communes[0].controlled = 1;
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
    // Tanglemere's workshops and its cloth hall's store, wherever its layout puts them.
    let spot = |g: &Game, label: &str| g.world.shops.iter().find(|s| s.kind.label() == label && s.spot.0 >= 125).map(|s| s.spot).expect(label);
    let (kiln, vats, dye_house) = (spot(&g, "KILN"), spot(&g, "INDIGO HOUSE"), spot(&g, "DYE HOUSE"));
    let hall = g.world.bins.iter().find(|b| b.scores && b.x >= 125).map(|b| (b.x, b.y)).expect("the cloth hall's store");
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
    let renown = g.communes[0].renown;
    g.script("U");
    assert!(g.communes[0].renown > renown, "the cloth hall takes dyed fabric for renown");
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
    assert_eq!(g.monsters[g.communes[0].controlled].x, x + 6, "walked across the river");
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
    g.script("D"); // back into the pumpkin: the train backs up, here right where it started
    assert_eq!(((g.monsters[0].x, g.monsters[0].y), g.world.wagons[0].pumpkin, g.world.wagons[0].at), ((pumpkin.0 + 1, pumpkin.1), pumpkin, bed));
    g.script("u");
    assert!(g.world.wagons[0].leader.is_none(), "Space with nothing else to do lets go");

    // Load it from behind, open the tailgate, step away: it unloads.
    let w = &g.world.wagons[0];
    let back = w.behind();
    let (dx, dy) = (back.0 - w.at.0, back.1 - w.at.1);
    let dir = if dx > 0 { Dir::Left } else if dx < 0 { Dir::Right } else if dy > 0 { Dir::Up } else { Dir::Down };
    g.world.items.push(Item::new(back.0, back.1, good_like(&g.data, "fabric", 0.8)));
    g.communes[0].controlled = 1;
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
    g.press(0, Input { use_pressed: true, ..Input::default() });
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
        g.monsters.push(Monster::new(10 + k, 0, "TEST", "goblin", &g.data, (60 + k as i32, 30), skills));
    }
    stand(&mut g, (5, 5), Dir::Down);
    let (sp, mut fb) = (crate::sprites::Sprites::new(), crate::framebuffer::Framebuffer::new());
    crate::render::draw(&g, 0, &sp, &mut fb);
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
    g.communes[0].controlled = 1;
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
    g.communes[0].inspecting = true;
    let card = crate::inspect::card(&g, 0);
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
    g.monsters.push(Monster::new(10, 0, "ISKA", "goblin", &g.data, (44, 19), skills));
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
            // And goes on saying so while nothing gets done, rather than something from the
            // far end of its loop.
            g.run_ticks(60);
            assert!(g.monsters[0].wait_reason.ends_with("IS IN THE WAY."), "{}", g.monsters[0].wait_reason);
            return;
        }
    }
    panic!("never let go: {said:?}");
}

#[test]
fn a_loop_with_nothing_to_do_keeps_off_where_goods_come_and_go() {
    // Waiting for fabric to be brought to the tile it takes it from, Mogg doesn't stand on that
    // tile: there it would be in the way of the very fabric it's waiting for.
    let mut g = Game::new();
    let fabric = Goal::Move { good: "fabric".into(), from: (20, 13), to: (24, 13) };
    start_loop(&mut g, 0, Routine { flag: (20, 13), goals: vec![fabric] }, (20, 13));
    g.run_ticks(10);
    assert_ne!((g.monsters[0].x, g.monsters[0].y), (20, 13), "{}", g.monsters[0].wait_reason);
    assert!(g.monsters[0].wait_reason.starts_with("WAITING FOR FABRIC"));
}

#[test]
fn a_workshop_says_what_its_pads_lack_and_what_to_swap() {
    // Henry's indigo house: quicklime, leaves, lye, lye and sticks. Indigo takes two quicklime and
    // one lye, and every pad is taken.
    let mut g = Game::new();
    let vats = g.world.shops.iter().position(|s| s.kind.label() == "INDIGO HOUSE").expect("the indigo house");
    let pads = g.world.shops[vats].pads.clone();
    for (k, id) in ["quicklime", "indigofera_leaves", "lye", "lye", "elder_sticks"].iter().enumerate() {
        g.world.items.push(Item::new(pads[k].0, pads[k].1, good_like(&g.data, id, 0.8)));
    }
    assert_eq!(g.pads_note(vats, &|_| true), "FOR INDIGO: SWAP THE LYE FOR ANOTHER LIME.");
    // With the second lye gone, there's a pad for it.
    g.world.items.retain(|it| (it.x, it.y) != pads[3]);
    assert_eq!(g.pads_note(vats, &|_| true), "FOR INDIGO: PUT ANOTHER LIME ON A PAD.");
}

#[test]
fn a_good_that_lands_on_a_loops_way_is_walked_around_not_pushed() {
    // Mogg heads for the bench across the meadow; a bolt is set down right in front of him.
    let mut g = Game::new();
    let bench = g.world.shop_at_spot(14, 9).unwrap();
    start_loop(&mut g, 0, Routine { flag: (24, 13), goals: vec![Goal::Activate(Thing::Shop(bench))] }, (24, 13));
    g.world.items.push(Item::new(13, 9, boll(&g.data)));
    g.run_ticks(1);
    let next = g.monsters[0].path[0];
    g.world.items.push(Item::new(next.0, next.1, good_like(&g.data, "fabric", 0.8)));
    g.run_ticks(60);
    assert!(g.world.item_at(next.0, next.1).is_some_and(|k| g.world.items[k].good() == "fabric"), "the bolt stayed where it was set down");
}

#[test]
fn a_loop_on_another_screen_isnt_heard() {
    // Bruna pushes a bolt to and fro out in Tinless, a screen east of Pinewater.
    let pushes = |watch_from: (i32, i32)| {
        let mut g = Game::new();
        g.world.items.push(Item::new(66, 11, good_like(&g.data, "fabric", 0.8)));
        let fabric = |from, to| Goal::Move { good: "fabric".into(), from, to };
        start_loop(&mut g, 1, Routine { flag: (65, 11), goals: vec![fabric((66, 11), (69, 11)), fabric((69, 11), (66, 11))] }, (65, 11));
        (g.monsters[0].x, g.monsters[0].y) = watch_from;
        let mut heard = 0;
        for _ in 0..300 {
            g.update(&[]);
            heard += g.sounds_for(0).into_iter().filter(|s| *s == Sfx::Push).count();
        }
        heard
    };
    assert_eq!(pushes((5, 13)), 0, "Mogg is back in Pinewater");
    assert!(pushes((70, 13)) > 0, "standing by, Mogg hears them");
}

#[test]
fn a_led_wagon_backs_out_of_a_pocket_it_cant_turn_round_in() {
    // Henry's spot in the Thatch Hills: Bruna has led the wagon into a pocket with trees on
    // three sides of her, the pumpkin on the fourth, and let go. Taking the lead again, she
    // steps back into the pumpkin over and over: the train backs out, its tail turning where
    // the way bends, until she can let go and walk off.
    let mut g = Game::new();
    g.communes[0].controlled = 1;
    {
        let w = &mut g.world.wagons[0];
        (w.at, w.pumpkin, w.leader) = ((76, 15), (76, 16), None);
    }
    stand(&mut g, (77, 16), Dir::Right);
    g.script("R");
    assert_eq!((g.monsters[1].x, g.monsters[1].y), (77, 16), "boxed in");
    g.script("u");
    assert_eq!(g.world.wagons[0].leader, Some(g.monsters[1].id), "Space turned her to the pumpkin and she took its lead");
    g.script("L");
    let w = &g.world.wagons[0];
    assert_eq!(((g.monsters[1].x, g.monsters[1].y), w.pumpkin, w.at), ((76, 16), (76, 15), (75, 15)), "a tree straight behind: its tail turned west");
    g.script("ULL");
    let w = &g.world.wagons[0];
    assert_eq!(((g.monsters[1].x, g.monsters[1].y), w.pumpkin, w.at), ((74, 15), (73, 15), (72, 15)));
    g.script("uU");
    assert_eq!((g.monsters[1].x, g.monsters[1].y), (74, 14), "let go and walked off");
}

#[test]
fn monsters_sharing_one_plant_all_get_to_harvest_it() {
    let mut g = Game::new().without_town();
    let p = g.world.plant_at(8, 8).unwrap();
    g.world.plants[p].regrow = 40;
    for (k, name) in ["ISKA", "WREN"].into_iter().enumerate() {
        let skills = g.monsters[0].skills.clone();
        g.monsters.push(Monster::new(10 + k as u32, 0, name, "goblin", &g.data, (0, 0), skills));
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

/// A tool as its makers left it, of oak.
fn tool(g: &Game, id: &str) -> ItemData {
    let raw = vec![crate::econ::RawMaterial { good: "oak_log".into(), base_value: 6.0, density: 100.0, spent: false }];
    ItemData { good: id.into(), quality: 0.6, carried_tags: vec!["oak".into()], raw, depth: 1, value: 6.0, shares: vec![crate::econ::Share { monster: 9, count: 1.0 }], wear: 0.0, haul: Vec::new(), carted: None }.tap(g)
}

trait Tap {
    fn tap(self, g: &Game) -> Self;
}

impl Tap for ItemData {
    /// Checks it's a good the data knows.
    fn tap(self, g: &Game) -> Self {
        assert!(g.data.try_good(&self.good).is_some(), "{} isn't a good", self.good);
        self
    }
}

fn said(g: &Game, i: usize) -> String {
    g.monsters[i].bubble.as_ref().map(|b| b.text.clone()).unwrap_or_default()
}

#[test]
fn a_monster_takes_gathering_tools_in_hand_and_needs_them_to_gather() {
    let mut g = Game::new();
    let c = g.communes[0].controlled;
    // Oyster lime wants a hammer and a chisel to knock it loose.
    let regrow = 10;
    g.world.plants.push(Plant { x: 20, y: 13, good: "oyster_lime".into(), growth: regrow, regrow, stock: Vec::new(), harvest: None, drop: None, timer: 0, owner: None });
    (g.monsters[c].x, g.monsters[c].y, g.monsters[c].facing) = (20, 13, Dir::Down);
    g.script("u");
    assert!(said(&g, c).contains("MISSING A TOOL"), "{}", said(&g, c));
    // Space facing a tool on the ground takes it into a free hand.
    (g.monsters[c].x, g.monsters[c].y, g.monsters[c].facing) = (25, 13, Dir::Right);
    for id in ["stone_hammer", "metal_chisel"] {
        g.world.items.push(Item::new(26, 13, tool(&g, id)));
        g.script("u");
        assert!(g.world.item_at(26, 13).is_none(), "{id} taken: {}", said(&g, c));
    }
    assert!(said(&g, c).contains("TAKES THE METAL CHISEL"), "{}", said(&g, c));
    g.world.items.push(Item::new(26, 13, tool(&g, "stone_hammer")));
    g.script("u");
    assert!(said(&g, c).contains("I'VE GOT A STONE HAMMER ALREADY") && g.world.item_at(26, 13).is_some(), "{}", said(&g, c));
    g.world.items.retain(|it| (it.x, it.y) != (26, 13));
    g.world.items.push(Item::new(26, 13, tool(&g, "shovel")));
    g.script("u");
    assert!(said(&g, c).contains("MY HANDS ARE FULL") && g.world.item_at(26, 13).is_some(), "{}", said(&g, c));
    g.world.items.retain(|it| (it.x, it.y) != (26, 13));
    // A log is no tool: Space leaves it be.
    g.world.items.push(Item::new(26, 13, ItemData { good: "oak_log".into(), ..tool(&g, "oak_log") }));
    g.script("u");
    assert!(g.world.item_at(26, 13).is_some(), "a log stays on the ground");
    // With both in hand, it gathers, and the tools wear.
    (g.monsters[c].x, g.monsters[c].y, g.monsters[c].facing) = (20, 13, Dir::Down);
    g.script("u");
    for _ in 0..5000 {
        if g.communes[0].made.contains("oyster_lime") {
            break;
        }
        g.run_ticks(1);
    }
    assert!(g.communes[0].made.contains("oyster_lime"), "gathered: {}", said(&g, c));
    assert!(g.monsters[c].tools().all(|t| t.wear == 2.0), "each wears by its weight");
}

#[test]
fn the_character_sheet_takes_the_keys_and_puts_a_tool_down() {
    let mut g = Game::new();
    let c = g.communes[0].controlled;
    (g.monsters[c].x, g.monsters[c].y, g.monsters[c].facing) = (25, 13, Dir::Right);
    g.monsters[c].hands[1] = Some(Held::Tool(tool(&g, "stone_hammer")));
    g.script("c");
    assert!(g.communes[0].sheet, "C opens it");
    g.script("D");
    assert_eq!((g.monsters[c].x, g.monsters[c].y), (25, 13), "the sheet takes the arrow keys");
    assert_eq!(g.communes[0].sheet_hand, 1, "down chooses the other hand");
    g.script("u");
    assert!(g.monsters[c].hands[1].is_none(), "put down: {}", said(&g, c));
    assert!(g.world.item_at(26, 13).is_some_and(|it| g.world.items[it].good() == "stone_hammer"), "on the tile ahead");
    g.script("t");
    assert_eq!(g.communes[0].sheet_page, 1, "Tab turns to its loop");
    g.script("c");
    assert!(!g.communes[0].sheet, "C closes it");
    g.script("R");
    assert_eq!(g.monsters[c].x, 26, "closed, the arrows walk again (pushing the hammer)");
}

#[test]
fn esc_backs_out_of_whats_open_then_asks_before_leaving() {
    let mut g = Game::new();
    g.script("ch");
    assert!(g.communes[0].help && g.communes[0].sheet);
    g.script("b");
    assert!(!g.communes[0].help && g.communes[0].sheet, "the help first, on top");
    g.script("b");
    assert!(!g.communes[0].sheet, "then the sheet");
    assert!(!g.communes[0].leaving);
    g.script("b");
    assert!(g.communes[0].leaving && !g.communes[0].left, "with nothing open, it asks");
    let c = g.communes[0].controlled;
    let x = g.monsters[c].x;
    g.script("R");
    assert!(!g.communes[0].leaving, "another key takes the question back");
    assert_eq!(g.monsters[c].x, x, "and does nothing else");
    g.script("bb");
    assert!(g.communes[0].left, "Esc twice leaves");
}

#[test]
fn inputs_are_named_as_types_of_things() {
    let g = Game::new();
    let group = |tags: &[&str]| tags.iter().map(|t| t.to_string()).collect::<Vec<String>>();
    assert_eq!(g.describe_type(&group(&["log"])), "LOG", "any log");
    assert_eq!(g.describe_type(&group(&["stalks", "bast"])), "BAST STALKS", "any bast plant's stalks, not rice's");
    assert_eq!(g.describe_type(&group(&["fiber"])), "FIBER");
    assert_eq!(g.describe_type(&group(&["ashy"])), "ASHY STALKS", "a note in brackets doesn't count");
    assert_eq!(g.inputs_text(g.data.good("wagon_kit")), "2 WAGON WHEELS, A HARDWOOD LOG, 2 PLANKS, A PUMPKIN");
    assert_eq!(g.inputs_text(g.data.good("workbench_kit")), "2 LOGS, ROPE");
}

#[test]
fn a_benchs_card_and_message_aim_at_the_closest_recipe() {
    let mut g = Game::new();
    let c = g.communes[0].controlled;
    let bench = g.world.shops.iter().position(|s| s.kind.label() == "BUILDERS' BENCH").expect("the builders' bench");
    let pad = g.world.shops[bench].pads[0];
    (g.monsters[c].x, g.monsters[c].y) = g.world.shops[bench].spot;
    assert_eq!(g.aim(bench, &|_| true).recipe, "kiln_kit", "empty, it rests on what fills its pads");
    // One log: the nearest of what a log goes into (two short), not the resting kiln kit (five).
    g.world.items.push(Item::new(pad.0, pad.1, ItemData { good: "hemlock_log".into(), carried_tags: Vec::new(), ..tool(&g, "hemlock_log") }));
    assert_eq!(g.aim(bench, &|_| true).recipe, "workbench_kit");
    let card = crate::inspect::card(&g, 0);
    assert_eq!(card.title, "BUILDERS' BENCH: WORKBENCH KIT", "the card names what it aims at");
    assert!(has_line(&card, "FOR WORKBENCH KIT: PUT ANOTHER LOG ON A PAD."), "and says the same");
    g.script("u");
    assert!(said(&g, c).contains("FOR WORKBENCH KIT: PUT ANOTHER LOG ON A PAD."), "{}", said(&g, c));
    // Cloth instead: only a loom house kit takes it.
    let log = g.world.item_at(pad.0, pad.1).unwrap();
    g.world.items[log].data = ItemData { good: "fabric".into(), carried_tags: Vec::new(), ..tool(&g, "fabric") };
    let aim = g.aim(bench, &|_| true);
    assert_eq!((aim.recipe.as_str(), aim.missing.len()), ("loom_house_kit", 5));
    assert_eq!(crate::inspect::card(&g, 0).title, "BUILDERS' BENCH: LOOM HOUSE KIT");
}

#[test]
fn a_cleared_bench_stays_set_up_for_what_its_pads_last_worked_toward() {
    let mut g = Game::new();
    let c = g.communes[0].controlled;
    let bench = g.world.shops.iter().position(|s| s.kind.label() == "BUILDERS' BENCH").expect("the builders' bench");
    let pad = g.world.shops[bench].pads[3];
    // Cloth pushed onto a pad from the west.
    g.world.items.push(Item::new(pad.0 - 1, pad.1, ItemData { good: "fabric".into(), carried_tags: Vec::new(), ..tool(&g, "fabric") }));
    (g.monsters[c].x, g.monsters[c].y, g.monsters[c].facing) = (pad.0 - 2, pad.1, Dir::Right);
    assert_eq!(g.aim(bench, &|_| true).recipe, "kiln_kit");
    g.script("R");
    assert_eq!(g.world.item_at(pad.0, pad.1).map(|it| g.world.items[it].good().to_string()).as_deref(), Some("fabric"), "{}", said(&g, c));
    assert_eq!(g.aim(bench, &|_| true).recipe, "loom_house_kit");
    let it = g.world.item_at(pad.0, pad.1).unwrap();
    g.world.items.remove(it);
    assert_eq!(g.aim(bench, &|_| true).recipe, "loom_house_kit", "cleared, it stays set up for a loom house kit");
    (g.monsters[c].x, g.monsters[c].y) = g.world.shops[bench].spot;
    assert_eq!(crate::inspect::card(&g, 0).title, "BUILDERS' BENCH: LOOM HOUSE KIT");
}

#[test]
fn a_bench_says_what_its_pads_are_for_and_what_else_it_makes() {
    let mut g = Game::new();
    let c = g.communes[0].controlled;
    let gate = g.world.shops.iter().position(|s| s.kind == ShopKind::named("workbench") && s.pads.len() == 2).expect("the gate bench");
    let spot = g.world.shops[gate].spot;
    (g.monsters[c].x, g.monsters[c].y) = spot;
    g.script("u");
    assert!(said(&g, c).contains("FOR ROPE: PUT THREAD ON A PAD.") && said(&g, c).contains("IT MAKES 5 MORE"), "{}", said(&g, c));
    // Goods that go into nothing a bench makes: said so, then what to swap for the nearest.
    let pad = g.world.shops[gate].pads[0];
    g.world.items.push(Item::new(pad.0, pad.1, ItemData { good: "fabric".into(), carried_tags: Vec::new(), ..tool(&g, "fabric") }));
    let pad = g.world.shops[gate].pads[1];
    g.world.items.push(Item::new(pad.0, pad.1, ItemData { good: "pumpkin".into(), carried_tags: Vec::new(), ..tool(&g, "pumpkin") }));
    g.script("u");
    assert!(said(&g, c).contains("NOTHING HERE TAKES FABRIC WITH A PUMPKIN. FOR ROPE: SWAP THE FABRIC FOR THREAD."), "{}", said(&g, c));
    let card = crate::inspect::card(&g, 0);
    assert!(has_line(&card, "IT ALSO MAKES") && has_line(&card, "AND 2 MORE."), "its card gives a few examples of what else it makes");
    assert_eq!(card.pages.len(), 3, "on a third page of its own, after the details");
    assert!(card.pages[2].iter().any(|l| l.text.starts_with("IT ALSO MAKES")), "the third page");
}

#[test]
fn a_goods_card_says_where_it_comes_from_and_goes() {
    let mut g = Game::new();
    let c = g.communes[0].controlled;
    (g.monsters[c].x, g.monsters[c].y, g.monsters[c].facing) = (25, 13, Dir::Right);
    g.world.items.push(Item::new(26, 13, ItemData { good: "planks".into(), ..tool(&g, "planks") }));
    let card = crate::inspect::card(&g, 0);
    assert!(has_line(&card, "MADE AT A WORKBENCH FROM A LOG."), "planks");
    assert!(has_line(&card, "GOES INTO") && has_line(&card, "AND 4 MORE"), "seven things in play take planks, three named");
    g.world.items.clear();
    g.world.items.push(Item::new(26, 13, ItemData { good: "oak_log".into(), ..tool(&g, "oak_log") }));
    let card = crate::inspect::card(&g, 0);
    assert!(has_line(&card, "BURNS AS FUEL."), "said once, not forty times");
}

#[test]
fn old_sawgrass_says_what_could_be_made_from_whats_around() {
    let mut g = Game::new();
    let c = g.communes[0].controlled;
    let t = g.world.townsfolk.iter().position(|t| t.name == "OLD SAWGRASS").expect("by the builders' bench");
    let (tx, ty) = (g.world.townsfolk[t].x, g.world.townsfolk[t].y);
    assert!(g.applicant_at(tx, ty), "he stands in the way like anyone");
    (g.monsters[c].x, g.monsters[c].y, g.monsters[c].facing) = (tx, ty + 1, Dir::Up);
    assert!(g.craftsman_says(t, 0).last().unwrap().contains("BRING SOME GOODS BY"), "nothing to work with yet");
    // Two logs and a rope lying near him: a workbench kit, at the builders' bench.
    for (x, good) in [(30, "hemlock_log"), (31, "hemlock_log"), (32, "rope")] {
        g.world.items.push(Item::new(x, 20, ItemData { good: good.into(), carried_tags: Vec::new(), ..tool(&g, good) }));
    }
    let says = g.craftsman_says(t, 0).join(" ");
    assert!(says.contains("I BET YOU COULD MAKE A WORKBENCH KIT") && says.contains("THE BUILDERS' BENCH COULD DO IT"), "{says}");
    g.script("u");
    assert!(g.communes[0].heard.contains("workbench_kit"), "it goes in the commune's book");
    // One rope short of a bin kit's two, with planks enough: the nearest miss.
    g.world.items.clear();
    for x in 30..34 {
        g.world.items.push(Item::new(x, 20, ItemData { good: "planks".into(), ..tool(&g, "planks") }));
    }
    g.world.items.push(Item::new(34, 20, ItemData { good: "rope".into(), carried_tags: Vec::new(), ..tool(&g, "rope") }));
    let says = g.craftsman_says(t, 0).join(" ");
    assert!(says.contains("WITH ONE MORE ROPE YOU COULD MAKE A BIN KIT"), "{says}");
}

#[test]
fn a_trader_gives_plain_goods_worth_what_was_given_carrying_the_givers_shares() {
    let mut g = Game::new();
    let c = g.communes[0].controlled;
    let stall = g.world.shops.iter().position(|s| s.kind.trades()).expect("the trader's stall");
    let (spot, pads) = (g.world.shops[stall].spot, g.world.shops[stall].pads.clone());
    let cloth = ItemData { good: "fabric".into(), carried_tags: Vec::new(), ..tool(&g, "fabric") };
    let given: f64 = cloth.raw.iter().map(|r| r.base_value).sum();
    g.world.items.push(Item::new(pads[0].0, pads[0].1, cloth));
    // At the stall's spot, facing the trader behind it: Space trades, rather than talking.
    (g.monsters[c].x, g.monsters[c].y, g.monsters[c].facing) = (spot.0, spot.1, Dir::Up);
    assert!(g.faced_townsfolk(c).is_some(), "the trader stands behind his stall");
    assert_eq!(g.aim(stall, &|_| true).recipe, "trade_beeswax", "one cloth is for beeswax");
    g.script("u");
    g.run_ticks(60);
    let out = g.world.items.iter().find(|it| it.good() == "beeswax").unwrap_or_else(|| panic!("no beeswax: {}", said(&g, c)));
    assert_eq!(out.data.quality, crate::econ::TRADE_QUALITY, "plain work");
    assert!(out.data.shares.iter().any(|s| s.monster == 9), "the cloth's makers share in it");
    assert!((out.data.raw[0].base_value - given).abs() < 1e-9, "worth what was given");
    // It offers only what nothing in the world makes or grows.
    assert!(g.offers(stall).contains(&"trade_red_clay"));
    let mut clay = g.world.plants[0].clone();
    clay.good = "red_clay".into();
    g.world.plants.push(clay);
    assert!(!g.offers(stall).contains(&"trade_red_clay"), "clay grows here now");
}

#[test]
fn a_bell_cast_from_traded_goods_hangs_rings_for_everyone_and_is_remembered() {
    let mut g = Game::new();
    let (a, b) = (g.player("a"), g.player("b"));
    assert_ne!(a, b);
    let c = g.communes[a].controlled;
    let foundry = g.world.shops.iter().position(|s| s.kind.label() == "FOUNDRY").expect("the foundry");
    let (spot, pads) = (g.world.shops[foundry].spot, g.world.shops[foundry].pads.clone());
    assert_eq!(crate::world::area_name(spot.0, spot.1), "TEMPLE TOWN");
    let goods = ["quicklime", "mortar", "beeswax", "oil", "red_clay", "charcoal", "hemlock_log", "elder_sticks", "coke", "copper_ingot", "tin_ingot"];
    for (&id, &(x, y)) in goods.iter().zip(&pads) {
        g.world.items.push(Item::new(x, y, ItemData { good: id.into(), carried_tags: Vec::new(), ..tool(&g, id) }));
    }
    (g.monsters[c].x, g.monsters[c].y) = spot;
    assert_eq!(g.aim(foundry, &|_| true).recipe, "temple_bell", "{}", g.pads_note(foundry, &|_| true));
    g.script_as(a, "u");
    g.run_ticks(60);
    let bell = g.world.items.iter().position(|it| it.good() == "temple_bell").unwrap_or_else(|| panic!("no bell: {}", said(&g, c)));
    assert_eq!(g.world.items[bell].data.carried_tags, ["bronze"], "copper and tin met in it: a bronze bell");
    assert_eq!(crate::inspect::display_name(&g, &g.world.items[bell].data), "BRONZE TEMPLE BELL", "copper with tin goes by its alloy's name");
    // Pushed into the temple's frame, it hangs and rings.
    let frame = g.world.boards.iter().position(|b| b.project == Project::Bell).expect("the bell frame");
    let (fx, fy) = (g.world.boards[frame].x, g.world.boards[frame].y);
    (g.world.items[bell].x, g.world.items[bell].y) = (fx, fy - 1);
    (g.monsters[c].x, g.monsters[c].y, g.monsters[c].facing) = (fx, fy - 2, Dir::Down);
    let renown = g.communes[a].renown;
    let mut down = Input::default();
    (down.pressed[1], down.held[1]) = (true, true);
    g.press(a, down);
    assert_eq!(g.world.bells.len(), 1, "{}", said(&g, c));
    assert!(g.sounds_for(b).contains(&Sfx::Bell), "the other player hears it, wherever they are");
    assert!(g.announcement.as_ref().is_some_and(|n| n.line.contains("BRONZE BELL, CAST BY")), "and everyone sees how well it was made, in what: {:?}", g.announcement.as_ref().map(|n| &n.line));
    assert!(g.communes[a].renown > renown, "its caster's commune shares in it");
    assert!(g.world.boards[frame].done() && g.world.boards[frame].rest > 0, "then the frame rests");
    // Saved and loaded, the temple remembers it.
    let copy = crate::save::from_json(&crate::save::to_json(&g)).expect("loads");
    assert_eq!(copy.world.bells.len(), 1);
    // After its rest, a new bell can replace it.
    g.world.boards[frame].rest = 1;
    g.run_ticks(1);
    assert!(!g.world.boards[frame].done(), "it takes a new bell");
}

/// A good as plainly made as a test needs, carrying no variety.
fn plain_good(g: &Game, id: &str) -> ItemData {
    ItemData { good: id.into(), carried_tags: Vec::new(), ..tool(g, id) }
}

/// Monster `c` trades what's on the stall's pads, then takes what it gives off the mat into a
/// free hand, standing beside the mat.
fn trade_and_take(g: &mut Game, stall: usize, gives: &[&str], recipe: &str) {
    let c = g.communes[0].controlled;
    let (spot, pads, out) = (g.world.shops[stall].spot, g.world.shops[stall].pads.clone(), g.world.shops[stall].outs[0]);
    for (&id, &(x, y)) in gives.iter().zip(&pads) {
        g.world.items.push(Item::new(x, y, plain_good(g, id)));
    }
    assert_eq!(g.aim(stall, &|_| true).recipe, recipe, "{gives:?}");
    stand(g, spot, Dir::Up);
    g.script("u");
    g.run_ticks(60);
    let made = g.data.good(recipe).output().to_string();
    assert!(g.world.item_at(out.0, out.1).is_some_and(|it| g.world.items[it].good() == made), "the {made} on the mat: {}", said(g, c));
    let beside = DIRS.iter().copied().find(|d| {
        let (dx, dy) = d.delta();
        let at = (out.0 - dx, out.1 - dy);
        g.world.walkable(at.0, at.1) && g.world.shop_at_spot(at.0, at.1).is_none() && g.world.item_at(at.0, at.1).is_none()
    });
    let d = beside.expect("room beside the mat");
    stand(g, (out.0 - d.delta().0, out.1 - d.delta().1), d);
    g.script("u");
    assert!(g.monsters[c].tools().any(|h| h.good == made), "{made} in hand: {}", said(g, c));
}

/// Harvests the plant monster `c` stands on, facing down, and waits for it.
fn dig(g: &mut Game, p: usize) -> String {
    let c = g.communes[0].controlled;
    let (x, y) = (g.world.plants[p].x, g.world.plants[p].y);
    g.world.items.retain(|it| (it.x, it.y) != (x, y + 1));
    (g.world.plants[p].growth, g.world.plants[p].stock) = (g.world.plants[p].regrow, Vec::new());
    stand(g, (x, y), Dir::Down);
    g.script("u");
    let said_then = said(g, c);
    for _ in 0..2000 {
        if g.world.plants[p].harvest.is_none() {
            break;
        }
        g.run_ticks(1);
    }
    format!("{said_then} / {}", said(g, c))
}

#[test]
fn coal_is_dug_with_a_traded_pick_and_shovel_in_hand_which_wear_and_break() {
    let mut g = Game::new();
    let c = g.communes[0].controlled;
    let stall = g.world.shops.iter().position(|s| s.kind.trades()).expect("the trader's stall");
    let offers = g.offers(stall);
    assert!(offers.contains(&"trade_pick") && offers.contains(&"trade_shovel"), "nobody makes them yet: {offers:?}");
    assert!(!offers.iter().any(|r| g.data.good(r).output() == "coke"), "the kiln makes coke, from coal");
    let seam = g.world.plants.iter().position(|p| p.good == "coal" && p.owner.is_none()).expect("a seam for anyone");
    assert_eq!(crate::world::area_name(g.world.plants[seam].x, g.world.plants[seam].y), "TEMPLE TOWN");
    // Without tools, nothing, and its card says what it takes.
    assert!(dig(&mut g, seam).contains("MISSING A TOOL: PICK."), "{}", said(&g, c));
    let card = crate::inspect::card(&g, 0);
    assert!(has_line(&card, "MISSING A TOOL: PICK.") && has_line(&card, "IT TAKES A PICK AND A SHOVEL IN HAND."), "{:?}", card.lines().map(|l| l.text.clone()).collect::<Vec<_>>());
    // Two rope for a pick, a rope and a thread for a shovel, each taken in hand off the mat.
    trade_and_take(&mut g, stall, &["rope", "rope"], "trade_pick");
    trade_and_take(&mut g, stall, &["rope", "thread"], "trade_shovel");
    let at = (g.world.plants[seam].x, g.world.plants[seam].y);
    stand(&mut g, at, Dir::Down);
    assert!(!has_line(&crate::inspect::card(&g, 0), "MISSING A TOOL"), "both in hand");
    // With both, it digs coal, and each wears by its weight.
    dig(&mut g, seam);
    assert!(g.communes[0].made.contains("coal"), "dug: {}", said(&g, c));
    assert!(g.monsters[c].tools().all(|t| t.wear == 3.0), "a pick and a shovel each wear 3 a dig");
    // Worn through, the pick breaks, and the seam waits for another.
    for t in g.monsters[c].hands.iter_mut().flatten().filter_map(Held::tool_mut).filter(|t| t.good == "pick") {
        t.wear = crate::econ::TOOL_DURABILITY as f64 - 1.0;
    }
    let got = dig(&mut g, seam);
    assert!(got.contains("MY PICK WORE OUT."), "{got}");
    assert!(g.monsters[c].tools().all(|t| t.good == "shovel"), "only the shovel left");
    assert!(dig(&mut g, seam).contains("MISSING A TOOL: PICK."), "{}", said(&g, c));
}

#[test]
fn the_kiln_bakes_coal_into_coke_even_with_a_log_for_fuel() {
    let mut g = Game::new();
    let kiln = g.world.shops.iter().position(|s| s.kind.label() == "KILN").expect("Tanglemere's kiln");
    let pads = g.world.shops[kiln].pads.clone();
    for (id, &(x, y)) in ["coal", "oak_log"].iter().zip(&pads) {
        g.world.items.push(Item::new(x, y, plain_good(&g, id)));
    }
    assert_eq!(g.aim(kiln, &|_| true).recipe, "coke", "brought coal means coke, not charcoal from the log");
}

/// Old Soot, the collier, and his order board.
fn old_soot(g: &Game) -> (usize, usize) {
    let soot = g.monsters.iter().position(|m| m.name == "OLD SOOT").expect("Old Soot");
    (soot, g.world.boards.iter().position(|b| b.wants == "coal").expect("his order board"))
}

#[test]
fn old_soot_digs_his_own_end_of_the_seam_and_his_tools_never_wear() {
    let mut g = Game::new();
    let (soot, order) = old_soot(&g);
    let id = Some(g.monsters[soot].id);
    let his: Vec<usize> = (0..g.world.plants.len()).filter(|&p| g.world.plants[p].owner == id).collect();
    assert_eq!(his.len(), 2, "two seams of his own");
    assert!(g.world.plants.iter().filter(|p| p.good == "coal" && p.owner.is_none()).count() >= 3, "and more for anyone");
    let says = (0..4).flat_map(|k| { g.communes[0].idea_turn = k; g.worker_says(soot, 0) }).collect::<Vec<_>>().join(" ");
    assert!(says.contains("COAL SEAM TAKES A PICK AND A SHOVEL IN HAND."), "{says}");
    let mut filled = false;
    for _ in 0..6000 {
        g.run_ticks(1);
        filled |= g.world.boards[order].done();
    }
    assert!(filled, "six coal filled his order: at goal {}, {:?}", g.monsters[soot].pc, g.monsters[soot].wait_reason);
    assert!(g.monsters[soot].hands.iter().all(|h| h.as_ref().and_then(Held::tool).is_some_and(|t| t.wear == 0.0)), "a pick and a shovel, unworn");
    // His seam is his.
    g.monsters[soot].playing = false;
    let c = g.communes[0].controlled;
    let p = his[0];
    let at = (g.world.plants[p].x, g.world.plants[p].y);
    stand(&mut g, at, Dir::Down);
    g.monsters[c].hands = vec![Some(Held::Tool(tool(&g, "pick"))), Some(Held::Tool(tool(&g, "shovel")))];
    g.script("u");
    assert!(said(&g, c).contains("THAT'S OLD SOOT'S OWN COAL SEAM"), "{}", said(&g, c));
}

/// Old Spoke, and his order board.
fn old_spoke(g: &Game) -> (usize, usize) {
    (g.monsters.iter().position(|m| m.worker.is_some()).expect("Old Spoke"), g.world.boards.iter().position(|b| b.project == Project::Order).expect("his order board"))
}

#[test]
fn old_spoke_makes_wheels_from_his_own_wood_and_fills_his_orders() {
    let mut g = Game::new();
    let (spoke, order) = old_spoke(&g);
    let mut filled = false;
    for _ in 0..6000 {
        g.run_ticks(1);
        filled |= g.world.boards[order].done();
    }
    assert!(filled, "four wheels filled an order: {} at goal {}, {:?}", g.monsters[spoke].name, g.monsters[spoke].pc, g.monsters[spoke].wait_reason);
    assert!(g.communes.iter().filter(|c| c.town).all(|c| c.renown > 0.0), "his wheels earn the town renown, not anyone's commune");
}

#[test]
fn old_spokes_wood_and_bench_are_his() {
    let mut g = Game::new();
    let (spoke, _) = old_spoke(&g);
    g.monsters[spoke].playing = false;
    g.run_ticks(2);
    let c = g.communes[0].controlled;
    let plank = g.world.items.iter().position(|it| it.owner.is_some() && it.good() == "planks").expect("his planks set out");
    let (px, py) = (g.world.items[plank].x, g.world.items[plank].y);
    stand(&mut g, (px + 1, py), Dir::Left);
    g.script("L");
    assert_eq!((g.world.items[plank].x, g.world.items[plank].y), (px, py), "Mogg can't push it");
    assert!(said(&g, c).contains("THAT'S OLD SPOKE'S."), "{}", said(&g, c));
    assert!(!g.may_take(c, plank), "nor can a loop take it");
    let bench = g.world.shops.iter().position(|s| s.owner == Some(g.monsters[spoke].id)).expect("his bench");
    let spot = g.world.shops[bench].spot;
    stand(&mut g, spot, Dir::Down);
    g.script("u");
    assert!(said(&g, c).contains("THAT'S OLD SPOKE'S OWN BENCH"), "{}", said(&g, c));
}

#[test]
fn anyone_can_aid_old_spokes_order_with_their_own_wheels() {
    let mut g = Game::new();
    let (spoke, order) = old_spoke(&g);
    g.monsters[spoke].playing = false;
    let (bx, by) = (g.world.boards[order].x, g.world.boards[order].y);
    g.world.items.push(Item::new(bx - 1, by, ItemData { good: "wagon_wheel".into(), ..tool(&g, "wagon_wheel") }));
    stand(&mut g, (bx - 2, by), Dir::Right);
    let renown = g.communes[0].renown;
    g.script("R");
    assert_eq!(g.world.boards[order].have, 1, "{}", said(&g, g.communes[0].controlled));
    assert!(g.communes[0].renown >= renown, "it earns renown for whoever made it");
}

#[test]
fn the_wagonwrights_bench_hints_only_at_wagon_things_and_nobody_joins_the_town() {
    let mut g = Game::new();
    let gate = g.world.shops.iter().position(|s| s.kind.label() == "WAGONWRIGHT'S BENCH" && s.owner.is_none()).expect("the bench by his gate");
    assert_eq!(g.offers(gate), ["wagon_wheel", "wagon_kit"]);
    let builders = g.world.shops.iter().position(|s| s.kind.label() == "BUILDERS' BENCH").expect("the builders' bench");
    assert!(!g.offers(builders).iter().any(|r| r.starts_with("wagon")), "wheels and wagon kits moved to the wagonwright's");
    let c = g.communes[0].controlled;
    let spot = g.world.shops[gate].spot;
    stand(&mut g, spot, Dir::Down);
    let card = crate::inspect::card(&g, 0);
    assert_eq!(card.title, "WAGONWRIGHT'S BENCH: WAGON KIT");
    assert!(has_line(&card, "WAGON WHEEL:") && !has_line(&card, "MORE."), "and it also makes only wheels");
    // Facing him, he says what his bench makes, from what.
    let (spoke, _) = old_spoke(&g);
    g.monsters[spoke].playing = false;
    let (sx, sy) = (g.monsters[spoke].x, g.monsters[spoke].y);
    stand(&mut g, (sx, sy + 1), Dir::Up);
    assert_eq!(g.faced_worker(c), Some(spoke));
    assert_eq!(g.worker_says(spoke, 0), ["A WAGON WHEEL TAKES A HARDWOOD LOG, 2 PLANKS, 2 STICKS."]);
    // Players never get the town's commune.
    let (a, b) = (g.player("a"), g.player("b"));
    assert!(!g.communes[a].town && !g.communes[b].town && a != b);
}

#[test]
fn old_spoke_takes_a_break_after_each_order_then_picks_up_where_he_left_off() {
    let mut g = Game::new();
    let (spoke, order) = old_spoke(&g);
    let need = g.world.boards[order].need;
    g.world.boards[order].have = need - 1;
    for _ in 0..3000 {
        g.run_ticks(1);
        // The break begins once the push that filled the order is done.
        if g.monsters[spoke].rest > 0 && !g.monsters[spoke].playing {
            break;
        }
    }
    assert!(g.monsters[spoke].rest > 0 && !g.monsters[spoke].playing, "the last wheel of an order, then a break: {:?}", g.monsters[spoke].wait_reason);
    assert_eq!(g.worker_says(spoke, 0), ["A WAGON WHEEL TAKES A HARDWOOD LOG, 2 PLANKS, 2 STICKS."], "nothing said about it: he just talks as ever");
    let spot = g.monsters[spoke].routine.as_ref().map(|r| r.flag).expect("his loop");
    let mut rested_there = false;
    for _ in 0..(15 * 30 + 300) {
        g.run_ticks(1);
        rested_there |= (g.monsters[spoke].x, g.monsters[spoke].y) == spot;
        if g.monsters[spoke].playing {
            break;
        }
    }
    assert!(rested_there, "he rests at his own spot");
    assert!(g.monsters[spoke].playing && g.monsters[spoke].rest == 0, "then he's back at it");
}

#[test]
fn a_supply_beside_a_shed_sets_its_goods_out_of_the_shed() {
    let mut g = Game::new();
    let (spoke, _) = old_spoke(&g);
    g.monsters[spoke].playing = false;
    let at = g.world.supplies[0].at;
    // A shed wall to its west; its good taken.
    let w = g.world.w;
    g.world.tiles[(at.1 * w + at.0 - 1) as usize] = Tile::Wall;
    g.world.items.retain(|it| (it.x, it.y) != at);
    g.world.supplies[0].timer = 0;
    g.run_ticks(crate::world::SUPPLY_TICKS);
    let k = g.world.item_at(at.0, at.1).expect("a fresh good set out");
    assert_eq!(g.world.items[k].slide.map(|s| s.dir), Some(Dir::Right), "sliding out of the shed, not appearing from nowhere");
}

/// The layout workflow's checks (DESIGN.md, Layout by image model), for every place a layout
/// rebuilds: no pad, peg or output mat lies as near another work spot as its own, so none goes
/// to the wrong bench.
#[test]
fn every_pad_peg_and_mat_is_nearer_its_own_bench_than_any_other() {
    let g = Game::new();
    let d = |a: (i32, i32), b: (i32, i32)| (a.0 - b.0).abs() + (a.1 - b.1).abs();
    for (k, s) in g.world.shops.iter().enumerate() {
        for &at in s.pads.iter().chain(&s.pegs).chain(&s.outs) {
            let theirs = d(at, s.spot);
            for (j, o) in g.world.shops.iter().enumerate().filter(|&(j, _)| j != k) {
                assert!(d(at, o.spot) > theirs, "{at:?} of the {} at {:?} is as near the {} at {:?} (shop {j})", s.kind.label(), s.spot, o.kind.label(), o.spot);
            }
        }
    }
}

/// And every working townsperson's loop is short and stays in their yard: their first good
/// delivered to their order board within forty seconds of a new world, never more than six
/// tiles from their bench (or, gathering, their first plant).
#[test]
fn a_working_townsperson_delivers_soon_and_stays_by_their_bench() {
    let mut g = Game::new();
    let workers: Vec<usize> = (0..g.monsters.len()).filter(|&i| g.monsters[i].worker.is_some()).collect();
    assert!(workers.len() >= 2, "Old Spoke and Old Soot");
    let home = |g: &Game, i: usize| {
        let id = Some(g.monsters[i].id);
        let bench = g.world.shops.iter().find(|s| s.owner == id).map(|s| s.spot);
        bench.or_else(|| g.world.plants.iter().find(|p| p.owner == id).map(|p| (p.x, p.y))).expect("their bench or plant")
    };
    let homes: Vec<(i32, i32)> = workers.iter().map(|&i| home(&g, i)).collect();
    let boards: Vec<usize> = workers
        .iter()
        .map(|&i| {
            let goals = &g.monsters[i].routine.as_ref().expect("their loop").goals;
            let to = goals.iter().rev().find_map(|gl| if let Goal::Move { to, .. } = gl { Some(*to) } else { None }).expect("a delivery");
            g.world.board_at(to.0, to.1).expect("their order board")
        })
        .collect();
    let mut first = vec![None; workers.len()];
    for t in 1..=600 {
        g.run_ticks(1);
        for (n, &i) in workers.iter().enumerate() {
            let (m, (hx, hy)) = (&g.monsters[i], homes[n]);
            assert!((m.x - hx).abs().max((m.y - hy).abs()) <= 6, "{} is at {:?}, far from home at {:?}", m.name, (m.x, m.y), (hx, hy));
            if first[n].is_none() && g.world.boards[boards[n]].have > 0 {
                first[n] = Some(t);
            }
        }
    }
    for (n, &i) in workers.iter().enumerate() {
        assert!(first[n].is_some(), "{}'s first good within forty seconds", g.monsters[i].name);
    }
}

/// Temple Town's layout (rebuilt 2026-11-03 from the image model's variants): a bell cast at the
/// foundry can be pushed out of its door, along the road, through the temple's gate and its door,
/// and into its frame, by a loop as by a player.
#[test]
fn a_bell_from_the_foundry_can_be_pushed_through_the_gate_into_the_temples_frame() {
    let mut g = Game::new();
    let foundry = g.world.shops.iter().position(|s| s.kind.label() == "FOUNDRY").expect("the foundry");
    let out = g.world.shops[foundry].outs[0];
    let frame = g.world.boards.iter().find(|b| b.project == Project::Bell).map(|b| (b.x, b.y)).expect("the bell frame");
    let bell = crate::econ::plain(&g.data, "temple_bell", "copper_ingot");
    g.world.items.push(Item::new(out.0, out.1, bell));
    let below = (out.0, out.1 + 1);
    start_loop(&mut g, 0, Routine { flag: below, goals: vec![Goal::Move { good: "temple_bell".into(), from: out, to: frame }] }, below);
    let mut ticks = 0;
    while g.world.bells.is_empty() && ticks < 15 * 240 {
        g.run_ticks(1);
        ticks += 1;
    }
    assert_eq!(g.world.bells.len(), 1, "hung: {} at {:?}, {:?}", g.monsters[0].name, (g.monsters[0].x, g.monsters[0].y), g.monsters[0].wait_reason);
    assert!(ticks > 15 * 20, "it was pushed the whole way, which takes a while: {ticks} ticks");
}

#[test]
fn taking_a_tool_while_recording_is_part_of_the_routine() {
    let mut g = Game::new();
    stand(&mut g, (25, 13), Dir::Right);
    g.world.items.push(Item::new(26, 13, tool(&g, "pick")));
    g.script("ru");
    let goals = &g.monsters[0].rec.as_ref().expect("recording").goals;
    assert_eq!(goals, &vec![Goal::Take { good: "pick".into(), from: (26, 13) }]);
    assert!(crate::inspect::goal_text(&g, &goals[0]).starts_with("TAKE A PICK FROM"), "{}", crate::inspect::goal_text(&g, &goals[0]));
}

/// A digger's loop re-arms itself: its pick breaks, it takes the spare it was shown, and digs
/// on. Its shovel still has wear, so the spare shovel stays where it lies.
#[test]
fn a_loop_takes_a_spare_tool_when_its_own_breaks_and_leaves_spares_be_otherwise() {
    let mut g = Game::new();
    let seam = g.world.plants.iter().position(|p| p.good == "coal" && p.owner.is_none()).expect("a seam for anyone");
    let (sx, sy) = (g.world.plants[seam].x, g.world.plants[seam].y);
    let (pick_at, shovel_at) = ((sx, sy + 3), (sx + 1, sy + 3));
    g.world.items.push(Item::new(pick_at.0, pick_at.1, tool(&g, "pick")));
    g.world.items.push(Item::new(shovel_at.0, shovel_at.1, tool(&g, "shovel")));
    let mut worn = tool(&g, "pick");
    worn.wear = crate::econ::TOOL_DURABILITY as f64 - 3.0;
    g.monsters[0].hands = vec![Some(Held::Tool(worn)), Some(Held::Tool(tool(&g, "shovel")))];
    let goals = vec![
        Goal::Take { good: "pick".into(), from: pick_at },
        Goal::Take { good: "shovel".into(), from: shovel_at },
        Goal::Activate(Thing::Plant { at: (sx, sy), facing: Dir::Down }),
    ];
    start_loop(&mut g, 0, Routine { flag: (sx, sy + 2), goals }, (sx, sy + 2));
    let (mut digs, mut digging) = (0, false);
    for _ in 0..15 * 120 {
        g.run_ticks(1);
        // Clear the coal away and let the rock fill back in at once.
        g.world.items.retain(|it| it.good() != "coal");
        let plant = &mut g.world.plants[seam];
        if plant.stock.is_empty() && plant.harvest.is_none() {
            plant.growth = plant.regrow;
        }
        if plant.harvest.is_some() && !digging {
            digs += 1;
        }
        digging = plant.harvest.is_some();
        if digs >= 2 && !digging {
            break;
        }
    }
    assert!(digs >= 2, "dug {digs} times: {:?}", g.monsters[0].wait_reason);
    assert!(g.world.item_at(pick_at.0, pick_at.1).is_none(), "the spare pick taken");
    assert!(g.monsters[0].tools().any(|t| t.good == "pick" && t.wear == 3.0), "the new pick, one dig worn: {:?}", g.monsters[0].hands);
    assert!(g.world.item_at(shovel_at.0, shovel_at.1).is_some(), "the spare shovel left be");
}

/// Space beside tools turns to one: one not yet in hand before one that is, and a second of
/// the same is refused. Between the test world's two tool bins, Space twice arms a digger.
#[test]
fn space_beside_tools_takes_one_not_yet_in_hand() {
    let mut g = crate::temple_test_game();
    let c = g.communes[0].controlled;
    let chute = |g: &Game, tool: &str| g.world.bins.iter().find(|b| b.accepts == [tool]).and_then(|b| b.mouth).expect("a tool bin");
    g.run_ticks(10);
    let (p, s) = (chute(&g, "pick"), chute(&g, "shovel"));
    assert_eq!((p.1, s.0 - p.0), (s.1, 2), "the chutes, a tile apart");
    stand(&mut g, (p.0 + 1, p.1), Dir::Up);
    let held = |g: &Game| g.monsters[c].tools().map(|t| t.good.clone()).collect::<Vec<_>>();
    g.script("u");
    g.run_ticks(10);
    g.script("u");
    let mut got = held(&g);
    got.sort();
    assert_eq!(got, ["pick", "shovel"], "{}", said(&g, c));
    g.run_ticks(10);
    g.script("u");
    assert!(said(&g, c).contains("ALREADY"), "a second is no use: {}", said(&g, c));
}

/// Whatever a plant in play asks for in hand, a goblin and a cyclops can hold together, made
/// of anything (a pick from copper weighed more than a goblin can carry, 2026-11-03).
#[test]
fn the_tools_a_plant_asks_for_fit_together_in_a_goblins_and_a_cyclopss_hands() {
    let g = Game::new();
    for p in &g.world.plants {
        let groups = &g.data.good(&p.good).tools_required_tags;
        let weight: f64 = groups
            .iter()
            .filter(|t| !t.is_empty())
            .map(|t| {
                let tool = g.data.goods.iter().find(|x| crate::data::matches(&x.type_tags, t)).expect("a good for the tool");
                ["copper_ingot", "oak_log", "limestone"].iter().map(|from| crate::econ::weight(tool, &crate::econ::plain(&g.data, &tool.id, from))).fold(0.0, f64::max)
            })
            .sum();
        for kind in ["goblin", "cyclops"] {
            let cap = g.data.monster_types[kind].body_cap as f64;
            assert!(weight <= cap, "a {kind} can't hold what the {} asks for: {weight} of {cap}", p.good);
        }
    }
}

/// A loop delivering to an order that's filled and resting waits, saying so plainly, rather than
/// trying to push it in and giving up ("I CAN'T GET THE COAL THERE", Henry, 2026-11-03).
#[test]
fn a_loop_delivering_to_a_filled_order_waits_saying_so() {
    let mut g = Game::new();
    let (_, order) = old_soot(&g);
    let (bx, by) = (g.world.boards[order].x, g.world.boards[order].y);
    let board = &mut g.world.boards[order];
    (board.have, board.rest) = (board.need, 15 * 600);
    let from = (bx + 3, by);
    g.world.items.push(Item::new(from.0, from.1, plain_good(&g, "coal")));
    start_loop(&mut g, 0, Routine { flag: (bx + 4, by), goals: vec![Goal::Move { good: "coal".into(), from, to: (bx, by) }] }, (bx + 4, by));
    g.run_ticks(60);
    assert!(g.monsters[0].wait_reason.contains("OLD SOOT'S ORDER IS FILLED. A NEW ONE SOON."), "{}", g.monsters[0].wait_reason);
    assert_eq!(g.world.item_at(from.0, from.1).map(|k| g.world.items[k].good().to_string()), Some("coal".into()), "the coal left where it lies");
}

/// The gate into the hills and its board.
fn hill_gate(g: &Game) -> (usize, usize) {
    let k = g.world.gates.iter().position(|gt| gt.province == "THE HILLS").expect("the gate into the hills");
    (k, g.world.gates[k].board)
}

#[test]
fn a_shut_gate_lets_nobody_past_and_says_what_opens_it() {
    let mut g = Game::new();
    let c = g.communes[0].controlled;
    let (k, b) = hill_gate(&g);
    let (gx, gy) = g.world.gates[k].tiles[0];
    assert_eq!(crate::world::area_name(gx, gy), "THE HIGH PASS");
    assert_eq!(g.world.boards[b].project, Project::Gate);
    stand(&mut g, (gx - 1, gy), Dir::Right);
    g.script("R");
    assert_eq!(g.monsters[c].x, gx - 1, "it doesn't open for walking into it");
    assert!(said(&g, c).contains("THE GATE TO THE HILLS IS SHUT"), "{}", said(&g, c));
    assert!(has_line(&crate::inspect::card(&g, 0), "SHUT"), "its card");
    assert!(!g.world.item_floor(gx, gy), "and no good gets pushed through it");
}

#[test]
fn twenty_cloth_within_the_hour_open_the_gate_for_good_and_everyone_hears() {
    let mut g = Game::new();
    let other = g.player("b");
    let (k, b) = hill_gate(&g);
    let need = g.world.boards[b].need;
    let mut sfx = Vec::new();
    for _ in 0..need - 1 {
        g.contribute(b, plain_good(&g, "fabric"), &mut sfx);
        g.run_ticks(15);
    }
    assert!(!g.world.gate_open(k) && g.world.tile(g.world.gates[k].tiles[0].0, g.world.gates[k].tiles[0].1) == Tile::Gate, "one short");
    g.contribute(b, plain_good(&g, "dyed_fabric"), &mut sfx);
    assert!(g.world.gate_open(k), "dyed cloth is cloth too");
    assert!(g.world.gates[k].tiles.iter().all(|&(x, y)| g.world.tile(x, y) == Tile::Path), "the gate's tiles are road now");
    assert_eq!(g.announcement.as_ref().map(|a| a.title.as_str()), Some("THE PASS TO THE HILLS IS OPEN!"));
    assert!(g.sounds_for(other).contains(&Sfx::Chime), "everyone hears it, wherever they are");
    // An hour on, with nothing more coming in, it stays open.
    g.frame += crate::world::GATE_WINDOW_TICKS + 1;
    g.run_ticks(1);
    assert!(g.world.gate_open(k), "open for good");
    let copy = crate::save::from_json(&crate::save::to_json(&g)).expect("loads");
    let (x, y) = copy.world.gates[k].tiles[0];
    assert_eq!(copy.world.tile(x, y), Tile::Path, "and still open after a save");
}

#[test]
fn a_gate_counts_a_rate_so_old_deliveries_drop_out() {
    let mut g = Game::new();
    let (k, b) = hill_gate(&g);
    let mut sfx = Vec::new();
    for _ in 0..10 {
        g.contribute(b, plain_good(&g, "fabric"), &mut sfx);
    }
    assert_eq!(g.world.boards[b].have, 10);
    g.frame += crate::world::GATE_WINDOW_TICKS;
    g.run_ticks(1);
    assert_eq!(g.world.boards[b].have, 0, "an hour later they no longer count");
    assert!(!g.world.gate_open(k));
}

#[test]
fn what_grows_in_a_closed_province_doesnt_count_until_its_gate_opens() {
    let mut g = Game::new();
    let (k, b) = hill_gate(&g);
    let stall = g.world.shops.iter().position(|s| s.kind.trades()).expect("the trader's stall");
    // Clay growing past the shut gate: the trader still has some.
    let (gx, gy) = g.world.gates[k].tiles[0];
    let mut clay = g.world.plants[0].clone();
    (clay.good, clay.x, clay.y) = ("red_clay".into(), gx + 2, gy);
    g.world.plants.push(clay);
    assert!(g.world.closed_at(gx + 2, gy));
    assert!(g.offers(stall).contains(&"trade_red_clay"), "nobody can get at that clay yet");
    let mut sfx = Vec::new();
    for _ in 0..g.world.boards[b].need {
        g.contribute(b, plain_good(&g, "fabric"), &mut sfx);
    }
    assert!(!g.world.closed_at(gx + 2, gy));
    assert!(!g.offers(stall).contains(&"trade_red_clay"), "now it can");
}

/// The gate's board is on the open side: cloth pushed out of Temple Town along the road east
/// reaches it.
#[test]
fn cloth_from_temple_town_can_be_pushed_out_to_the_gates_board() {
    let mut g = Game::new();
    let (_, b) = hill_gate(&g);
    let board = (g.world.boards[b].x, g.world.boards[b].y);
    let from = (75 + 40, 32 + 14);
    assert_eq!(crate::world::area_name(from.0, from.1), "TEMPLE TOWN");
    g.world.items.push(Item::new(from.0, from.1, plain_good(&g, "fabric")));
    start_loop(&mut g, 0, Routine { flag: (from.0 - 1, from.1 - 1), goals: vec![Goal::Move { good: "fabric".into(), from, to: board }] }, (from.0 - 1, from.1 - 1));
    for _ in 0..15 * 120 {
        if g.world.boards[b].have > 0 {
            break;
        }
        g.run_ticks(1);
    }
    assert_eq!(g.world.boards[b].have, 1, "delivered: {:?}", g.monsters[0].wait_reason);
}

/// The Cliff Lift's vine (2026-11-05): a good set at its foot is carried up the cliff and set
/// down past its head, and one arriving at a head that isn't clear waits in its cradle.
#[test]
fn the_cliff_lift_vine_carries_goods_up_and_waits_for_room_at_the_top() {
    let mut g = Game::new();
    assert_eq!(g.world.lifts.iter().map(|l| l.dir).collect::<Vec<_>>(), [Dir::Up, Dir::Down, Dir::Right], "the lift vine up from the way in, the ore vine down from the Copper Cut, the charcoal vine east from Charcoal Wood");
    let lift = g.world.lifts[0].clone();
    assert_eq!(crate::world::area_name(lift.foot.0, lift.foot.1), "THE CLIFF LIFT");
    assert!(g.world.walkable(lift.foot.0, lift.foot.1) && g.world.walkable(lift.top.0, lift.top.1), "open ground at both ends");
    assert!(lift.vine.len() >= 2 && lift.vine.iter().all(|&(x, y)| !g.world.walkable(x, y)), "it climbs the cliff");
    let fabric = |x: i32, y: i32| Item::new(x, y, crate::econ::plain(&g.data, "fabric", "cotton_bolls"));
    let (a, b) = (fabric(lift.foot.0, lift.foot.1), fabric(lift.top.0, lift.top.1));
    g.world.items.push(a);
    g.world.items.push(b);
    let ride = (lift.vine.len() as u32 + 2) * (crate::world::LIFT_TICKS as u32 + 1);
    g.run_ticks(ride);
    let top_vine = *lift.vine.last().unwrap();
    assert!(g.world.item_at(top_vine.0, top_vine.1).is_some(), "it waits in the cradle while the top is taken");
    let blocker = g.world.item_at(lift.top.0, lift.top.1).unwrap();
    g.world.items.remove(blocker);
    g.run_ticks(crate::world::LIFT_TICKS as u32 + 2);
    assert!(g.world.item_at(lift.top.0, lift.top.1).is_some_and(|it| g.world.items[it].good() == "fabric"), "set down at the top");
    assert!(lift.vine.iter().all(|&(x, y)| g.world.item_at(x, y).is_none()), "nothing left on the vine");
}

/// The hills walk as one (2026-11-05): from the way in, the long way round by Hemp Hollow and
/// Charcoal Wood, up the Stone Stair into Verdigris, to the lift vine's head and on to the Copper
/// Cut; and no other way, since the cliff keeps the way in's low ground from the heights.
#[test]
fn the_hills_walk_the_long_way_round_from_the_way_in_to_the_copper_cut() {
    let g = Game::new();
    let lift = g.world.lifts.iter().find(|l| l.dir == Dir::Up).expect("the lift vine").clone();
    let area = |p: &(i32, i32)| crate::world::area_name(p.0, p.1);
    // Every tile walkable from the lift vine's foot, and how far, keeping out of `skip`.
    let walk = |skip: &str| {
        let mut far = std::collections::HashMap::from([(lift.foot, 0u32)]);
        let mut queue = std::collections::VecDeque::from([lift.foot]);
        while let Some((x, y)) = queue.pop_front() {
            let d = far[&(x, y)];
            for (dx, dy) in [(1, 0), (-1, 0), (0, 1), (0, -1)] {
                let n = (x + dx, y + dy);
                if g.world.walkable(n.0, n.1) && area(&n) != skip && !far.contains_key(&n) {
                    far.insert(n, d + 1);
                    queue.push_back(n);
                }
            }
        }
        far
    };
    let far = walk("");
    for place in ["HEMP HOLLOW", "CHARCOAL WOOD", "VERDIGRIS", "COPPER CUT"] {
        assert!(far.keys().any(|p| area(p) == place), "{place} is reachable from the way in");
    }
    let by_vine = lift.vine.len() as u32 + 1;
    let on_foot = far.get(&lift.top).copied().expect("the vine's head, on foot");
    assert!(on_foot >= 8 * by_vine, "on foot the vine's head is the long way round: {on_foot} steps, {by_vine} by vine");
    assert!(!walk("CHARCOAL WOOD").contains_key(&lift.top), "the only way up on foot is the Stone Stair");
    let smithy = g.world.shops.iter().find(|s| s.kind.label() == "SMITHY" && s.owner.is_none()).expect("the smithy");
    assert!((smithy.spot.0 - lift.top.0).abs() + (smithy.spot.1 - lift.top.1).abs() <= 15, "the lift vine sets goods down a few steps from the smithy");
}

/// The world's height in the code (for made-up worlds and area names) is the regions' (a region
/// further south once made it taller without it, 2026-11-06).
#[test]
fn the_worlds_height_is_its_regions() {
    let g = Game::new();
    assert_eq!(g.world.h, crate::world::WORLD_ROWS);
    assert_eq!(crate::world::area_name(160, 70), "IRON BROOK");
}

/// Iron Brook's ore goes all the way to steel at Bellows' smelter (2026-11-06): two iron ore and
/// heat make pig iron; pig iron stirred with clay and sticks makes wrought iron; wrought iron and
/// pig iron with stalks, clay and heat make steel.
#[test]
fn iron_ore_becomes_pig_iron_wrought_iron_and_steel_at_the_smelter() {
    let mut g = Game::new();
    assert!(g.world.plants.iter().any(|p| p.good == "iron_ore" && crate::world::area_name(p.x, p.y) == "IRON BROOK"), "iron in Iron Brook's banks");
    let smelter = g.world.shops.iter().find(|s| s.kind.label() == "SMELTER").map(|s| s.spot).expect("a smelter in Bellows");
    assert!(work(&mut g, smelter, &["iron_ore", "iron_ore", "charcoal"]).contains(&"pig_iron_ingot".to_string()), "pig iron");
    g.world.items.retain(|it| !it.good().ends_with("_ingot"));
    assert!(work(&mut g, smelter, &["pig_iron_ingot", "red_clay", "elder_sticks"]).contains(&"wrought_iron_ingot".to_string()), "wrought iron");
    g.world.items.retain(|it| !it.good().ends_with("_ingot"));
    let steel = work(&mut g, smelter, &["wrought_iron_ingot", "pig_iron_ingot", "hemp_stalks", "red_clay", "charcoal"]);
    assert!(steel.contains(&"steel_ingot".to_string()), "steel: {steel:?}");
}

/// A workshop whose recipes can take the same goods has a choice board beside it (Henry,
/// 2026-11-06): the same ingot and heat make forceps when the board says so, not the knife that
/// comes first; Space on the board turns it; a loop records what it was set to, and sets it so
/// on replay.
#[test]
fn a_choice_board_says_what_a_workshop_makes_when_its_goods_could_make_several() {
    let mut g = Game::new();
    for s in 0..g.world.shops.len() {
        if g.world.shops[s].owner.is_none() && !g.world.shops[s].kind.trades() && !g.shadowed(s).is_empty() {
            assert!(g.world.choices.iter().any(|&(_, c)| c == s), "the {} at {:?} needs a choice board: {:?} can't be made without one", g.world.shops[s].kind.label(), g.world.shops[s].spot, g.shadowed(s));
        }
    }
    let smithy = g.world.shops.iter().position(|s| s.kind.label() == "SMITHY" && s.owner.is_none()).expect("the smithy");
    let spot = g.world.shops[smithy].spot;
    assert!(work(&mut g, spot, &["copper_ingot", "charcoal"]).contains(&"knife".to_string()), "nothing chosen: the first that fits");
    g.world.items.retain(|it| it.good() != "knife");
    // Space on the board, recorded.
    assert!(g.world.choices.iter().any(|&(_, s)| s == smithy), "its board");
    assert_eq!(g.shadowed(smithy), ["metal_hammer", "sickle", "forceps", "file", "billhook"], "what the smithy's pads alone could never make");
    let forceps = g.world.shops[smithy].kind.recipes().iter().position(|r| *r == "forceps").unwrap() as u8;
    let (before, facing) = g.activation_spot(Thing::Choice { shop: smithy, recipe: forceps }).expect("somewhere to stand");
    stand(&mut g, before, facing.unwrap());
    g.script("r");
    for _ in 0..12 {
        if g.world.shops[smithy].chosen.as_deref() == Some("forceps") {
            break;
        }
        g.script("u");
    }
    g.script("r");
    let c = g.communes[0].controlled;
    let goals = g.monsters[c].routine.as_ref().map(|r| r.goals.clone()).unwrap_or_default();
    assert!(goals.iter().any(|gl| matches!(gl, Goal::Activate(Thing::Choice { recipe, .. }) if *recipe == forceps)), "the loop recorded forceps: {goals:?}");
    assert!(work(&mut g, spot, &["copper_ingot", "charcoal"]).contains(&"forceps".to_string()), "the board says forceps");
}
