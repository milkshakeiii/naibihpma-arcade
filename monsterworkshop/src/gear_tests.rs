//! Equipment given for renown at Tanglemere's outfitter, rolled newcomers and seasoning (DESIGN.md:
//! Equipment given for renown; Abilities that differ and grow).

use super::*;
use crate::data::{DEX, INT};
use crate::econ::ItemData;

/// Mogg in the outfitter's, just above the rack showing `piece`, facing it, with `renown`.
fn at_rack(piece: &str, renown: f64) -> (Game, usize) {
    let mut g = Game::new();
    let r = g.world.racks.iter().position(|r| r.piece == piece).unwrap_or_else(|| panic!("a rack shows {piece}"));
    let (x, y) = (g.world.racks[r].x, g.world.racks[r].y);
    let m = &mut g.monsters[0];
    (m.x, m.y, m.facing) = (x, y - 1, Dir::Down);
    g.communes[0].renown = renown;
    (g, r)
}

fn said(g: &Game, i: usize) -> String {
    g.monsters[i].bubble.as_ref().map(|b| b.text.clone()).unwrap_or_default()
}

fn has_line(card: &crate::inspect::Card, needle: &str) -> bool {
    card.lines().any(|l| l.text.contains(needle))
}

/// A tool in hand, plainly made.
fn tool(id: &str) -> ItemData {
    let raw = vec![crate::econ::RawMaterial { good: "copper_ingot".into(), base_value: 6.0, density: 100.0, spent: false }];
    ItemData { good: id.into(), quality: 1.0, carried_tags: vec!["copper".into()], raw, depth: 1, value: 6.0, shares: Vec::new(), wear: 0.0, haul: Vec::new(), carted: None }
}

#[test]
fn the_outfitter_gives_a_piece_for_renown_and_it_raises_an_ability() {
    let (mut g, _) = at_rack("cloth_gloves", 120.0);
    let card = crate::inspect::card(&g, 0);
    assert_eq!(card.title, "CLOTH GLOVES");
    assert!(has_line(&card, "+1 DEX") && has_line(&card, "GIVEN AT 50 RENOWN. YOU HAVE 120.") && has_line(&card, "SPACE: MOGG PUTS IT ON, FOR GOOD."), "{:?}", card.lines().map(|l| &l.text).collect::<Vec<_>>());
    let thread = g.data.good("thread").clone();
    let (dex, before) = (g.monsters[0].stats.dex, crate::econ::production_seconds(&thread, &g.monsters[0].abilities(&g.data), &crate::econ::PLAIN));
    g.script("u");
    assert_eq!(g.monsters[0].gear, vec!["cloth_gloves".to_string()]);
    assert_eq!(g.communes[0].renown, 70.0, "given at 50 renown");
    assert!(said(&g, 0).contains("MOGG PUTS ON THE CLOTH GLOVES. +1 DEX!"), "{}", said(&g, 0));
    assert_eq!(g.monsters[0].abilities(&g.data).dex, dex + 1);
    assert_eq!(g.monsters[0].stats.dex, dex, "its own DEX is unchanged; the gloves add to it");
    assert!(crate::econ::production_seconds(&thread, &g.monsters[0].abilities(&g.data), &crate::econ::PLAIN) < before, "nimbler fingers spin faster");
    let card = crate::inspect::card(&g, 0);
    assert!(has_line(&card, "I HAVE THE CLOTH GLOVES ALREADY."));
    // Its own card says what it has on.
    g.monsters[0].facing = Dir::Up;
    assert!(has_line(&crate::inspect::card(&g, 0), "HAS ON CLOTH GLOVES"));
}

#[test]
fn space_beside_a_rack_turns_to_it_and_the_outfitter_just_talks() {
    let (mut g, _) = at_rack("cloth_gloves", 60.0);
    g.monsters[0].facing = Dir::Left;
    g.script("u");
    assert_eq!(g.monsters[0].gear, vec!["cloth_gloves".to_string()], "Space turned Mogg to the rack");
    // Old Thimble stands between the racks; Space only talks, adding nothing to the book.
    let t = g.world.townsfolk.iter().position(|t| t.kind == Folk::Outfitter).expect("the outfitter");
    let (tx, ty) = (g.world.townsfolk[t].x, g.world.townsfolk[t].y);
    (g.monsters[0].x, g.monsters[0].y, g.monsters[0].facing) = (tx, ty - 1, Dir::Down);
    let heard = g.communes[0].heard.clone();
    g.script("u");
    assert_eq!(g.communes[0].heard, heard);
    assert!(g.outfitter_says(0)[0].contains("YOURS HAS 10."), "{:?}", g.outfitter_says(0));
    assert_eq!(crate::inspect::card(&g, 0).title, "OLD THIMBLE THE OUTFITTER");
    // Beside two racks, it doesn't guess: what's given is for good.
    let (mut g, _) = at_rack("rope_belt", 60.0);
    g.monsters[0].facing = Dir::Left;
    assert!(g.racks_beside(0) == 2, "the rope belt's rack and the whetstone's above it");
    g.script("u");
    assert!(g.monsters[0].gear.is_empty() && g.monsters[0].hands.iter().all(Option::is_none) && g.communes[0].renown == 60.0);
    assert_eq!((said(&g, 0), g.monsters[0].facing), ("WHICH ONE? FACE IT FIRST.".to_string(), Dir::Left));
}

#[test]
fn too_little_renown_a_full_slot_or_one_already_on_is_said_plainly() {
    let (mut g, _) = at_rack("cloth_gloves", 30.0);
    g.script("u");
    assert!(g.monsters[0].gear.is_empty() && g.communes[0].renown == 30.0);
    assert_eq!(said(&g, 0), "GIVEN AT 50 RENOWN.");
    // A goblin has three worn slots.
    g.communes[0].renown = 1000.0;
    g.monsters[0].gear = vec!["rope_belt".into(), "straw_sandals".into(), "temple_charm".into()];
    g.script("u");
    assert_eq!(said(&g, 0), "ALL 3 OF MY WORN SLOTS ARE FULL. TAKE SOMETHING OFF ON MY SHEET (C).");
    assert_eq!(g.communes[0].renown, 1000.0, "nothing given, nothing spent");
    assert!(has_line(&crate::inspect::card(&g, 0), "WORN SLOTS ARE FULL"), "the card says so first");
}

#[test]
fn a_better_piece_of_a_kind_replaces_the_old_and_only_the_best_bonus_to_an_ability_counts() {
    let (mut g, r) = at_rack("cloth_gloves", 1000.0);
    let dex = g.monsters[0].stats.dex;
    g.monsters[0].gear = vec!["cloth_gloves".into(), "rope_belt".into(), "straw_sandals".into()];
    // A finer pair, as a rack elsewhere will show: it takes the cloth gloves' place, though
    // every worn slot is full.
    g.world.racks[r].piece = "gloves_of_dexterity";
    assert!(has_line(&crate::inspect::card(&g, 0), "IN PLACE OF THE CLOTH GLOVES"));
    g.script("u");
    assert_eq!(g.monsters[0].gear, vec!["rope_belt".to_string(), "straw_sandals".into(), "gloves_of_dexterity".into()]);
    assert_eq!(g.communes[0].renown, 200.0);
    assert!(said(&g, 0).contains("IN PLACE OF THE CLOTH GLOVES"), "{}", said(&g, 0));
    assert_eq!(g.monsters[0].abilities(&g.data).get(DEX), dex + 3);
    // Spectacles and a headband both add INT: only the better counts.
    let int = g.monsters[0].stats.int;
    g.monsters[0].gear = vec!["headband".into(), "spectacles".into()];
    assert_eq!(g.monsters[0].abilities(&g.data).get(INT), int + 2);
    g.monsters[0].gear = vec!["spectacles".into()];
    g.world.racks[r].piece = "headband";
    assert!(has_line(&crate::inspect::card(&g, 0), "MOGG'S SPECTACLES GIVE AS MUCH INT: ONLY THE BEST COUNTS."));
}

#[test]
fn body_and_mind_capacity_limit_what_a_monster_can_have_on() {
    let mut g = Game::new();
    // Bruna, a cyclops (body 100), holding a pick and a shovel (84 of it).
    let b = 1;
    g.monsters[b].hands = vec![Some(Held::Tool(tool("pick"))), Some(Held::Tool(tool("shovel")))];
    assert!((g.body_used(b) - 84.0).abs() < 1.0, "{}", g.body_used(b));
    assert_eq!(g.piece_refusal(b, g.data.piece("belt_of_strength")).as_deref(), Some("THAT'S MORE THAN MY BODY CAN TAKE."));
    assert_eq!(g.piece_refusal(b, g.data.piece("cloth_gloves")), None, "gloves (10) still fit");
    // What a monster has on counts against its tools too.
    g.monsters[b].hands = vec![None, None];
    g.monsters[b].gear = vec!["belt_of_strength".into(), "boots_of_endurance".into()];
    assert!((g.body_used(b) - 80.0).abs() < 1e-9);
    let mut sfx = Vec::new();
    g.monsters[b].facing = Dir::Right;
    let (x, y) = (g.monsters[b].x + 1, g.monsters[b].y);
    g.world.items.retain(|it| (it.x, it.y) != (x, y));
    g.world.items.push(crate::world::Item::new(x, y, tool("pick")));
    let it = g.world.item_at(x, y).expect("the pick ahead");
    assert!(matches!(g.take_tool(b, it, &mut sfx), UseRes::Wait(why) if why == "THAT'S MORE THAN I CAN HOLD."));
    // Mogg, a goblin, has a mind of 50: two pieces of 40 don't both fit.
    g.monsters[0].gear = vec!["headband_of_intellect".into()];
    assert_eq!(g.piece_refusal(0, g.data.piece("periapt_of_wisdom")).as_deref(), Some("THAT'S MORE THAN MY MIND CAN TAKE."));
}

#[test]
fn taking_a_piece_off_on_the_sheet_asks_first_then_frees_its_slot() {
    let mut g = Game::new();
    g.monsters[0].gear = vec!["cloth_gloves".into()];
    let lines = g.sheet_lines(0);
    assert_eq!(lines.len(), 2 + 3 + 1, "two hands, then a goblin's 3 worn and 1 inside slots");
    assert_eq!(lines[2], crate::gear::SheetLine::Gear(crate::data::Slot::Worn, Some("cloth_gloves".into())));
    g.script("cDD");
    assert_eq!(g.communes[0].sheet_hand, 2);
    g.script("u");
    assert_eq!(g.communes[0].sheet_confirm, Some(2), "Space asks first");
    assert_eq!(g.monsters[0].gear.len(), 1);
    g.script("u");
    assert!(g.monsters[0].gear.is_empty(), "Space again takes them off, for good");
    assert!(said(&g, 0).contains("MOGG TAKES OFF THE CLOTH GLOVES."));
    // Moving off a line forgets the question.
    g.monsters[0].gear = vec!["cloth_gloves".into()];
    g.script("uDU");
    assert_eq!(g.communes[0].sheet_confirm, None);
    g.script("u");
    assert_eq!(g.monsters[0].gear.len(), 1, "the first Space only asks");
}

#[test]
fn a_carried_piece_is_held_in_a_hand_as_a_tool_is() {
    // A towel: held, not worn.
    let (mut g, r) = at_rack("cloth_gloves", 1000.0);
    g.world.racks[r].piece = "towel";
    g.monsters[0].hands = vec![Some(Held::Tool(tool("pick"))), Some(Held::Tool(tool("shovel")))];
    g.script("u");
    assert_eq!(said(&g, 0), "MY HANDS ARE FULL. PUT SOMETHING DOWN ON MY SHEET (C).");
    g.monsters[0].hands[1] = None;
    g.script("u");
    assert!(matches!(&g.monsters[0].hands[1], Some(Held::Piece(id)) if id == "towel"), "{:?}", g.monsters[0].hands);
    assert!(said(&g, 0).starts_with("MOGG TAKES THE TOWEL."), "{}", said(&g, 0));
    assert_eq!(g.monsters[0].knack(&g.data, "textiles").more, 1, "a held piece counts as a worn one does");
    // With the towel in one hand, a second tool doesn't fit.
    let (x, y) = (g.monsters[0].x - 1, g.monsters[0].y);
    g.world.items.retain(|it| (it.x, it.y) != (x, y));
    g.world.items.push(crate::world::Item::new(x, y, tool("shovel")));
    let it = g.world.item_at(x, y).expect("a shovel beside Mogg");
    let mut sfx = Vec::new();
    g.monsters[0].facing = Dir::Left;
    assert!(matches!(g.take_tool(0, it, &mut sfx), UseRes::Wait(why) if why == "MY HANDS ARE FULL."));
    // On the sheet it's in HANDS, and Space twice puts it away for good.
    assert_eq!(g.sheet_lines(0)[1], crate::gear::SheetLine::Hand(1));
    g.script("cD");
    g.script("uu");
    assert!(g.monsters[0].hands[1].is_none() && g.monsters[0].tool(0).is_some(), "{:?}", g.monsters[0].hands);
    assert!(said(&g, 0).contains("MOGG GIVES UP THE TOWEL."), "{}", said(&g, 0));
    // Orcs have three hands; everyone else two.
    assert_eq!(g.data.monster_types["orc"].slots.hands, 3);
    assert!(["goblin", "cyclops", "elf"].iter().all(|k| g.data.monster_types[*k].slots.hands == 2));
}

#[test]
fn newcomers_at_the_hall_are_rolled_a_point_or_two_either_side_of_their_type() {
    let mut g = Game::new();
    let mut seen = std::collections::BTreeMap::<i32, u32>::new();
    for n in 0..300 {
        g.frame = n * 37;
        let a = g.roll_applicant(n as usize % 3);
        let base = &g.data.monster_types[a.kind.as_str()].stats;
        for k in 0..6 {
            let d = a.stats.get(k) - base.get(k);
            assert!((-2..=2).contains(&d), "{} off its type", d);
            *seen.entry(d).or_default() += 1;
        }
    }
    assert_eq!(seen.len(), 5, "every roll from -2 to +2 comes up: {seen:?}");
    assert!(seen[&0] > seen[&1] && seen[&1] > seen[&2] && seen[&0] > seen[&-1] && seen[&-1] > seen[&-2], "0 most often: {seen:?}");
    // The founders keep their types' own: the tutorial is made for them.
    let fresh = Game::new();
    assert_eq!(fresh.monsters[0].stats.dex, fresh.data.monster_types["goblin"].stats.dex);
    // A welcomed newcomer keeps what it rolled.
    let mut g = Game::new();
    g.communes[0].renown = 1000.0;
    let (sx, sy) = g.world.stands[0];
    let rolled = g.applicants[0].clone().expect("someone at the first stand").stats;
    (g.monsters[0].x, g.monsters[0].y, g.monsters[0].facing) = (sx, sy + 1, Dir::Up);
    g.script("u");
    let newcomer = g.monsters.last().expect("they joined");
    assert_eq!((newcomer.stats.str, newcomer.stats.dex, newcomer.stats.cha), (rolled.str, rolled.dex, rolled.cha));
}

#[test]
fn bringing_in_renown_seasons_a_monster_and_its_player_hears_of_it() {
    let mut g = Game::new();
    let (id, base) = (g.monsters[0].id, g.monsters[0].stats.clone());
    g.monsters[0].earned = 990.0;
    g.score(2.0, &[crate::econ::Share { monster: id, count: 1.0 }], 10, 10);
    assert!(g.monsters[0].earned >= 1000.0);
    let st = g.monsters[0].abilities(&g.data);
    assert!((0..6).all(|a| st.get(a) == base.get(a) + 1), "+1 to every ability");
    assert_eq!(said(&g, 0), "MOGG IS SEASONED: +1 TO EVERY ABILITY.");
    assert!(g.sounds_for(0).contains(&Sfx::Chime));
    g.monsters[0].facing = Dir::Left; // open grass: the card describes Mogg
    assert!(has_line(&crate::inspect::card(&g, 0), "SEASONED: +1 TO EVERY ABILITY"));
    g.monsters[0].earned = 4990.0;
    g.score(2.0, &[crate::econ::Share { monster: id, count: 1.0 }], 10, 10);
    assert_eq!(said(&g, 0), "MOGG IS SEASONED AGAIN: +2 TO EVERY ABILITY.");
    // A working townsperson stays as the town made them.
    let spoke = g.monsters.iter().position(|m| m.worker.is_some()).expect("Old Spoke");
    g.monsters[spoke].earned = 6000.0;
    assert_eq!(g.monsters[spoke].abilities(&g.data).str, g.monsters[spoke].stats.str);
    // Seasoning and a piece add up: they're the monster's own, not another piece's bonus.
    g.monsters[0].gear = vec!["cloth_gloves".into()];
    assert_eq!(g.monsters[0].abilities(&g.data).dex, base.dex + 3);
}

#[test]
fn what_a_monster_has_on_and_its_rolled_abilities_are_saved() {
    let mut g = Game::new();
    g.monsters[0].gear = vec!["cloth_gloves".into(), "headband".into()];
    g.monsters[1].stats.str += 2;
    let copy = crate::save::from_json(&crate::save::to_json(&g)).expect("loads");
    assert_eq!(copy.monsters[0].gear, g.monsters[0].gear);
    assert_eq!(copy.monsters[1].stats.str, g.monsters[1].stats.str);
}

#[test]
fn what_a_monster_has_on_shows_on_its_sprite() {
    let c = crate::palette::color_named("cotton", 2).expect("a color");
    let plain = crate::sprites::dressed_frames(true, &[]);
    let gloved = crate::sprites::dressed_frames(true, &[("gloves", c)]);
    for f in 0..4 {
        assert_ne!(plain[f].px, gloved[f].px, "frame {f} shows the gloves");
        assert!(gloved[f].px.contains(&c));
    }
    let shod = crate::sprites::dressed_frames(false, &[("shoes", c)]);
    assert!((0..16).any(|x| shod[3].px[(15 * 16 + x) as usize] == c), "a cyclops's sandals show mid-stride too");
}

// --- The crafting pieces (stage 2): what each does to a craft, and that the cards say so. ---

/// Mogg on a coal seam anyone may dig, a pick and a shovel in hand, `third` in a third hand (as an
/// orc has) and `gear` on: one dig, and how worn the pick is after.
fn dig_once(third: Option<&str>, gear: &[&str]) -> f64 {
    let mut g = Game::new();
    let seam = g.world.plants.iter().position(|p| p.good == "coal" && p.owner.is_none()).expect("a seam for anyone");
    let p = &mut g.world.plants[seam];
    p.growth = p.regrow;
    let (x, y) = (p.x, p.y);
    let m = &mut g.monsters[0];
    (m.x, m.y, m.facing) = (x, y, Dir::Down);
    m.hands = vec![Some(Held::Tool(tool("pick"))), Some(Held::Tool(tool("shovel")))];
    m.hands.extend(third.map(|p| Some(Held::Piece(p.into()))));
    m.gear = gear.iter().map(|s| s.to_string()).collect();
    g.script("u");
    for _ in 0..5000 {
        if g.communes[0].made.contains("coal") {
            break;
        }
        g.run_ticks(1);
    }
    assert!(g.communes[0].made.contains("coal"), "dug: {}", said(&g, 0));
    g.monsters[0].tool(0).expect("the pick").wear
}

/// Bruna at the spinnery with fiber on its pad, steered, as the forecast card is seen.
fn at_spinnery() -> (Game, usize) {
    let mut g = Game::new();
    let mut slivers = tool("cotton_slivers");
    (slivers.carried_tags, slivers.quality) = (vec!["cotton".into()], 0.6);
    g.world.items.push(crate::world::Item::new(30, 9, slivers));
    g.communes[0].controlled = 1;
    (g.monsters[1].x, g.monsters[1].y) = (30, 8);
    // Some way into spinning, so there's a forecast to lift.
    let sk = &mut g.monsters[1].skills;
    for k in ["spinning", "textiles", "weaving"] {
        sk.applied.insert(k.into(), 0.4);
    }
    sk.specific.insert("thread".into(), 0.4);
    let s = g.world.shop_at_spot(30, 8).expect("the spinnery");
    (g, s)
}

#[test]
fn each_crafting_piece_does_what_its_card_says() {
    let mut g = Game::new();
    let mut knack = |id: &str, skill: &str| {
        g.monsters[0].gear = vec![id.to_string()];
        g.monsters[0].knack(&g.data, skill)
    };
    let plain = crate::econ::PLAIN;
    assert_eq!(knack("towel", "spinning").more, 1);
    assert!((knack("magnifying_glass", "spinning").quality - 1.1).abs() < 1e-9);
    assert!((knack("wristwatch", "spinning").time - 0.9).abs() < 1e-9);
    assert!((knack("writing_brush", "hauling").learning - 1.1).abs() < 1e-9, "everything learned, practice too");
    assert!((knack("whetstone", "spinning").wear - 0.5).abs() < 1e-9);
    let apron = knack("hasty_apron", "spinning");
    assert!((apron.time - 0.8).abs() < 1e-9 && (apron.wear - 2.0).abs() < 1e-9);
    assert!((knack("notebook", "spinning").forgetting - 0.8).abs() < 1e-9);
    assert!((knack("carved_seal", "spinning").value - 1.1).abs() < 1e-9);
    assert!((knack("lucky_charm", "spinning").above - 1.5).abs() < 1e-9);
    assert!((knack("patience", "spinning").below - 0.5).abs() < 1e-9);
    // A craft's secret counts only at its own craft.
    assert!((knack("secret_spinning", "spinning").quality - 1.1).abs() < 1e-9);
    assert_eq!(knack("secret_spinning", "weaving"), plain);
    // Together they multiply: a whetstone makes up for a hasty apron's wear.
    let m = &mut g.monsters[0];
    m.gear = vec!["hasty_apron".into(), "secret_spinning".into()];
    m.hands = vec![Some(Held::Piece("whetstone".into())), Some(Held::Piece("magnifying_glass".into()))];
    let k = g.monsters[0].knack(&g.data, "spinning");
    assert!((k.wear - 1.0).abs() < 1e-9 && (k.quality - 1.21).abs() < 1e-9, "{k:?}");
    // What each says, as the sheet and the racks say it, fits the sheet's line.
    for p in &g.data.pieces {
        for text in [p.name.to_uppercase(), p.effect_text()] {
            assert!(!text.is_empty() && text.chars().count() <= 28, "{}: {text:?} doesn't fit the sheet", p.id);
        }
        assert!(p.color.as_ref().is_some_and(|(n, s)| crate::palette::color_named(n, *s).is_some()), "{} has a color to be drawn in", p.id);
        assert!(p.does.at.as_ref().is_none_or(|a| g.data.skills.applied_skills.contains(a)), "{}'s skill", p.id);
        assert!(p.does.talent.as_ref().is_none_or(|t| g.data.skills.transferable_skills.contains(t)), "{}'s talent", p.id);
    }
    assert_eq!(g.data.pieces.iter().filter(|p| p.kind == "teaching").count(), g.data.skills.transferable_skills.len(), "a teaching for every talent");
}

#[test]
fn the_forecast_card_says_what_helps_and_a_magnifying_glass_lifts_it() {
    let (mut g, s) = at_spinnery();
    let before = crate::inspect::forecast(&g, s, 1).expect("a forecast").mean;
    g.monsters[1].hands = vec![Some(Held::Piece("magnifying_glass".into())), Some(Held::Piece("towel".into()))];
    g.monsters[1].gear = vec!["secret_weaving".into()];
    let after = crate::inspect::forecast(&g, s, 1).expect("a forecast").mean;
    assert!((after / before - 1.1).abs() < 0.01, "+10% quality: {before} to {after}");
    let card = crate::inspect::card(&g, 0);
    assert!(has_line(&card, "MAGNIFYING GLASS: +10% QUALITY"), "{:?}", card.lines().map(|l| &l.text).collect::<Vec<_>>());
    assert!(!has_line(&card, "TOWEL"), "thread's yield doesn't vary: a towel adds nothing there");
    assert!(!has_line(&card, "SECRET OF WEAVING"), "weaving's secret is for weaving");
    // A wristwatch and a hasty apron shorten the job.
    let plain = g.job_ticks("thread", 1);
    g.monsters[1].hands = vec![Some(Held::Piece("wristwatch".into())), None];
    g.monsters[1].gear = vec!["hasty_apron".into()];
    let quick = g.job_ticks("thread", 1);
    assert!((quick as f64 / plain as f64 - 0.72).abs() < 0.02, "{plain} ticks to {quick}");
}

#[test]
fn a_towel_makes_one_more_where_how_many_varies() {
    let g = Game::new();
    let (st, sk) = (g.monsters[0].abilities(&g.data), g.monsters[0].skills.clone());
    let towel = crate::econ::PLAIN.with(&g.data.piece("towel").does);
    for (good, more) in [("cotton_bolls", 1), ("thread", 0)] {
        let recipe = g.data.good(good);
        let craft = |knack| crate::econ::Craft { recipe, crafter: crate::econ::Crafter { id: 1, stats: &st, skills: &sk, knack }, inputs: Vec::new(), tools: Vec::new(), workshop: None };
        for seed in 1..50 {
            let (a, b) = (crate::econ::roll_quantity(&craft(&crate::econ::PLAIN), &mut crate::econ::Rng::new(seed)), crate::econ::roll_quantity(&craft(&towel), &mut crate::econ::Rng::new(seed)));
            assert_eq!(b, a + more, "{good}");
        }
    }
}

#[test]
fn patience_and_a_lucky_charm_each_raise_the_average() {
    let g = Game::new();
    let st = g.monsters[0].abilities(&g.data);
    let lucky = crate::econ::PLAIN.with(&g.data.piece("lucky_charm").does);
    let patient = crate::econ::PLAIN.with(&g.data.piece("patience").does);
    // Thread with a destabilizer mastered spreads its results 0.4 either way; without, 0.1.
    for (spread, least) in [(1.0, 0.04), (0.0, 0.01)] {
        let mut recipe = g.data.good("thread").clone();
        recipe.destabilizer_skills = vec!["dyeing".into()];
        let mut sk = crate::econ::Skills::default();
        for (k, v) in [("spinning", 0.6), ("textiles", 0.6), ("weaving", 0.6), ("dyeing", spread)] {
            sk.applied.insert(k.into(), v);
        }
        sk.specific.insert("thread".into(), 0.6);
        let fiber = tool("cotton_slivers");
        let rolls = |knack: &crate::econ::Knack| -> Vec<f64> {
            let c = crate::econ::Craft { recipe: &recipe, crafter: crate::econ::Crafter { id: 1, stats: &st, skills: &sk, knack }, inputs: vec![&fiber], tools: Vec::new(), workshop: None };
            let mut rng = crate::econ::Rng::new(11);
            (0..4000).map(|_| crate::econ::roll_quality(&c, &mut rng)).collect()
        };
        let (base, l, p) = (rolls(&crate::econ::PLAIN), rolls(&lucky), rolls(&patient));
        let avg = |v: &[f64]| v.iter().sum::<f64>() / v.len() as f64;
        assert!(avg(&p) - avg(&base) > least && avg(&l) - avg(&base) > least, "spread {spread}: {} {} {}", avg(&base), avg(&l), avg(&p));
        // Roll by roll: patience only ever lifts a flop and keeps every masterwork; luck only ever
        // lifts a good result and leaves the flops.
        for k in 0..base.len() {
            assert!(p[k] >= base[k] - 1e-12 && l[k] >= base[k] - 1e-12);
            assert!((p[k] - base[k]).abs() < 1e-12 || (l[k] - base[k]).abs() < 1e-12, "never both at once");
        }
        assert!(p.iter().cloned().fold(0.0, f64::max) == base.iter().cloned().fold(0.0, f64::max), "patience keeps the best");
    }
}

#[test]
fn a_whetstone_halves_the_wear_on_tools_and_a_hasty_apron_doubles_it() {
    assert_eq!(dig_once(None, &[]), 3.0, "a pick wears 3 a dig");
    assert_eq!(dig_once(Some("whetstone"), &[]), 1.5);
    assert_eq!(dig_once(None, &["hasty_apron"]), 6.0);
    assert_eq!(dig_once(Some("whetstone"), &["hasty_apron"]), 3.0);
    let mut pick = tool("pick");
    pick.wear = 99.5;
    assert_eq!(pick.durability_left(), 1, "half a use left is still a use");
}

#[test]
fn a_writing_brush_a_notebook_and_a_carved_seal() {
    let g = Game::new();
    let (st, thread) = (g.monsters[0].abilities(&g.data), g.data.good("thread"));
    let mut sk = crate::econ::Skills::default();
    sk.applied.insert("spinning".into(), 0.3);
    let learn = |id: &str| {
        let knack = crate::econ::PLAIN.with(&g.data.piece(id).does);
        crate::econ::learn(&mut sk.clone(), &st, &knack, thread, &g.data, 300.0)
    };
    let (plain, brush, notebook) = (learn("towel"), learn("writing_brush"), learn("notebook"));
    assert!((brush.primary / plain.primary - 1.1).abs() < 0.01, "+10% learned: {} {}", plain.primary, brush.primary);
    assert!((notebook.forgotten / plain.forgotten - 0.8).abs() < 1e-9, "-20% forgotten");
    // A seal: what's made is worth 10% more.
    let fiber = tool("cotton_slivers");
    let seal = crate::econ::PLAIN.with(&g.data.piece("carved_seal").does);
    let made = |knack| {
        let c = crate::econ::Craft { recipe: thread, crafter: crate::econ::Crafter { id: 1, stats: &st, skills: &sk, knack }, inputs: vec![&fiber], tools: Vec::new(), workshop: None };
        crate::econ::produce(&c, &g.data, &mut crate::econ::Rng::new(5))[0].value
    };
    assert!((made(&seal) / made(&crate::econ::PLAIN) - 1.1).abs() < 1e-9);
}

#[test]
fn a_teaching_counts_as_a_talent_the_monster_lacks() {
    let (mut g, s) = at_spinnery();
    let thread = g.data.good("thread").clone();
    let lacks = thread.transferable_skills.iter().find(|t| !g.monsters[1].skills.transferable.contains(t)).expect("a talent thread wants that Bruna lacks").clone();
    let before = crate::inspect::forecast(&g, s, 1).expect("a forecast");
    let (sk, st) = (g.monsters[1].skills.clone(), g.monsters[1].abilities(&g.data));
    let learned = |g: &Game| crate::econ::learn(&mut sk.clone(), &st, &g.monsters[1].knack(&g.data, "spinning"), &thread, &g.data, 300.0).primary;
    let slow = learned(&g);
    g.monsters[1].gear = vec![format!("teaching_{lacks}")];
    assert_eq!(g.monsters[1].knack(&g.data, "spinning").talent.as_deref(), Some(lacks.as_str()));
    assert!(learned(&g) > slow * 1.1, "a talent the recipe wants speeds learning");
    let after = crate::inspect::forecast(&g, s, 1).expect("a forecast");
    assert!(after.secondary >= before.secondary && after.tool_quality >= before.tool_quality, "and skips the weakest helper");
    assert!(has_line(&crate::inspect::card(&g, 0), &format!("TALENT: {}", lacks.replace('_', " ").to_uppercase())));
    // One it was born with can't be taught.
    let has = g.monsters[1].skills.transferable[0].clone();
    assert_eq!(g.piece_refusal(1, g.data.piece(&format!("teaching_{has}"))), Some(format!("I HAVE A TALENT FOR {} ALREADY.", has.replace('_', " ").to_uppercase())));
    // A second teaching takes the first one's place.
    let other = g.data.skills.transferable_skills.iter().find(|t| **t != lacks && !g.monsters[1].skills.transferable.contains(t)).expect("another").clone();
    assert_eq!(g.replaces(1, g.data.piece(&format!("teaching_{other}"))).map(|p| p.id.clone()), Some(format!("teaching_{lacks}")));
}

#[test]
fn the_outfitter_has_things_to_work_with_by_the_door() {
    let g = Game::new();
    for piece in ["towel", "whetstone", "hasty_apron"] {
        let r = g.world.racks.iter().find(|r| r.piece == piece).unwrap_or_else(|| panic!("a rack shows the {piece}"));
        assert_eq!(crate::world::area_name(r.x, r.y), "TANGLEMERE");
    }
    // Every piece on a rack is one that's given in an open place, so for now none from overseas or
    // made of paper.
    for r in &g.world.racks {
        assert!(!["magnifying_glass", "wristwatch", "writing_brush", "notebook", "carved_seal"].contains(&r.piece), "{} isn't to be had yet", r.piece);
    }
}

#[test]
fn the_temple_keeps_a_teaching_for_every_talent_and_nothing_hangs_over_a_stand() {
    let mut g = Game::new();
    let in_temple_town = |g: &Game, piece: &str| g.world.racks.iter().any(|r| r.piece == piece && crate::world::area_name(r.x, r.y) == "TEMPLE TOWN");
    for t in &g.data.skills.transferable_skills {
        assert!(in_temple_town(&g, &format!("teaching_{t}")), "a scroll for {t}");
    }
    assert!(in_temple_town(&g, "patience") && in_temple_town(&g, "lucky_charm"));
    let k = g.world.townsfolk.iter().position(|t| t.kind == Folk::Keeper).expect("the temple keeper");
    let (kx, ky) = (g.world.townsfolk[k].x, g.world.townsfolk[k].y);
    assert_eq!(crate::world::area_name(kx, ky), "TEMPLE TOWN");
    // Facing the keeper, a word about the teachings; Space only talks.
    (g.monsters[0].x, g.monsters[0].y, g.monsters[0].facing) = (kx, ky - 1, Dir::Down);
    g.communes[0].renown = 300.0;
    g.script("u");
    assert!(g.monsters[0].gear.is_empty() && g.communes[0].renown == 300.0);
    assert!(g.keeper_says(0)[0].contains("YOURS HAS 300."));
    assert_eq!(crate::inspect::card(&g, 0).title, "OLD QUIET THE TEMPLE KEEPER");
    // A teaching, learned at a scroll: given at 200 renown.
    let lacks = g.data.skills.transferable_skills.iter().find(|t| !g.monsters[0].skills.transferable.contains(t)).expect("a talent Mogg lacks").clone();
    let r = g.world.racks.iter().find(|r| r.piece == format!("teaching_{lacks}")).expect("its scroll");
    (g.monsters[0].x, g.monsters[0].y, g.monsters[0].facing) = (r.x, r.y + 1, Dir::Up);
    let card = crate::inspect::card(&g, 0);
    assert!(has_line(&card, "SPACE: MOGG LEARNS IT, FOR GOOD."), "{:?}", card.lines().map(|l| &l.text).collect::<Vec<_>>());
    g.script("u");
    assert_eq!(g.monsters[0].gear, vec![format!("teaching_{lacks}")]);
    assert_eq!(g.communes[0].renown, 100.0);
    assert!(said(&g, 0).starts_with(&format!("MOGG LEARNS THE TEACHING OF")), "{}", said(&g, 0));
    // Nothing hung on any wall hangs down over a stand or someone standing below it.
    for &((x, y), prop) in &g.world.props {
        if prop.kind().drop > 0 {
            assert!(g.rack_at(x, y + 1).is_none(), "{} at ({x},{y}) hangs over a stand", prop.0);
            assert!(!g.world.townsfolk.iter().any(|t| (t.x, t.y) == (x, y + 1)), "{} at ({x},{y}) hangs over someone", prop.0);
        }
    }
}
