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

/// Runs until the ferry is docked on the given side (1 east, -1 west), or gives up.
fn until_docked(g: &mut Game, side: i32) {
    for _ in 0..600 {
        let f = g.world.ferry.as_ref().unwrap();
        if !f.moving && f.slide.is_none() && f.x == if side > 0 { f.east } else { f.west } {
            return;
        }
        g.run_ticks(1);
    }
    panic!("the ferry never docked");
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
fn rope_for_the_board_strings_the_ferry_and_it_carries_goods_across() {
    let mut g = Game::new();
    let (bx, by) = (g.world.boards[0].x, g.world.boards[0].y);
    let f = g.world.ferry.clone().unwrap();
    assert!(!f.rigged);

    // Not rigged yet: the river stops you, and says why.
    stand(&mut g, (f.west - 1, f.y), Dir::Right);
    g.script("R");
    assert_eq!((g.monsters[0].x, g.monsters[0].y), (f.west, f.y), "you can step aboard the moored boat");
    g.run_ticks(200);
    assert_eq!(g.world.ferry.as_ref().unwrap().x, f.west, "it doesn't move without a rope");

    g.world.boards[0].have = g.world.boards[0].need - 1;
    g.world.items.push(Item::new(bx, by + 1, good_like(&g.data, "rope", 0.9)));
    stand(&mut g, (bx, by + 2), Dir::Up);
    let renown = g.renown;
    g.script("U");
    assert!(g.world.boards[0].done() && g.world.ferry.as_ref().unwrap().rigged, "the last rope strings the ferry");
    assert!(g.renown > renown, "rope for the board earns renown");

    // Push a bolt of cloth aboard and ride across with it.
    g.world.items.push(Item::new(f.west - 1, f.y, good_like(&g.data, "fabric", 1.0)));
    stand(&mut g, (f.west - 2, f.y), Dir::Right);
    g.script("RR");
    assert_eq!((g.monsters[0].x, g.world.items.last().unwrap().x), (f.west, f.west + 1), "both aboard");
    until_docked(&mut g, 1);
    assert_eq!(g.monsters[0].x, f.east, "the monster rode across");
    assert!(g.world.item_at(f.east + 1, f.y).is_some(), "and so did the cloth");
    g.script("RR");
    assert_eq!(g.monsters[0].x, f.east + 2, "off onto the far landing");
    assert!(g.world.item_at(f.east + 3, f.y).is_some());
}

#[test]
fn a_loop_across_the_river_waits_for_the_ferry() {
    let mut g = Game::new();
    let f = g.world.ferry.clone().unwrap();
    g.world.ferry.as_mut().unwrap().rigged = true;
    stand(&mut g, (f.west - 1, f.y), Dir::Right);
    g.script("rR");
    until_docked(&mut g, 1);
    g.script("RRL");
    until_docked(&mut g, -1);
    g.script("LLr");
    let m = &g.monsters[0];
    assert!(m.routine.is_some() && (m.x, m.y) == (f.west - 1, f.y), "back on the flag with a saved loop");
    g.script("p");
    g.run_ticks(1500);
    let m = &g.monsters[0];
    assert!(m.playing && m.loops >= 3, "looped {} times", m.loops);
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
fn a_loop_can_pull_but_pays_for_it_every_time() {
    let mut g = Game::new();
    g.world.items.push(Item::new(1, 1, boll(&g.data)));
    stand(&mut g, (2, 1), Dir::Left);
    g.script("r>Lr");
    assert_eq!(g.monsters[0].routine.as_ref().unwrap().acts, vec![Act::Pull(Dir::Right, "cotton_bolls".into()), Act::Step(Dir::Left, Some("cotton_bolls".into()))]);
    g.script("p");
    g.run_ticks(400);
    let loops = g.monsters[0].loops;
    assert!((8..=13).contains(&loops), "a pull-and-push loop takes over two seconds: {loops} loops in 400 ticks");
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
