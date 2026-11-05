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
    assert!(has_line(&card, "+1 DEX") && has_line(&card, "(50 RENOWN) YOU HAVE 120.") && has_line(&card, "SPACE: MOGG PUTS IT ON, FOR GOOD."), "{:?}", card.lines().map(|l| &l.text).collect::<Vec<_>>());
    let thread = g.data.good("thread").clone();
    let (dex, before) = (g.monsters[0].stats.dex, crate::econ::production_seconds(&thread, &g.monsters[0].abilities(&g.data), &crate::econ::PLAIN_KNACK));
    g.script("u");
    assert_eq!(g.monsters[0].gear, vec!["cloth_gloves".to_string()]);
    assert_eq!(g.communes[0].renown, 70.0, "given at 50 renown");
    assert!(said(&g, 0).contains("MOGG PUTS ON THE CLOTH GLOVES. +1 DEX!"), "{}", said(&g, 0));
    assert_eq!(g.monsters[0].abilities(&g.data).dex, dex + 1);
    assert_eq!(g.monsters[0].stats.dex, dex, "its own DEX is unchanged; the gloves add to it");
    assert!(crate::econ::production_seconds(&thread, &g.monsters[0].abilities(&g.data), &crate::econ::PLAIN_KNACK) < before, "nimbler fingers spin faster");
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
    assert_eq!(said(&g, 0), "NOT ENOUGH RENOWN YET (50).");
    // A goblin has three worn slots.
    g.communes[0].renown = 1000.0;
    g.monsters[0].gear = vec!["rope_belt".into(), "straw_sandals".into(), "temple_charm".into()];
    g.script("u");
    assert_eq!(said(&g, 0), "ALL 3 OF MY WORN SLOTS ARE FULL. TAKE SOMETHING OFF ON MY SHEET (C).");
    assert_eq!(g.communes[0].renown, 1000.0, "nothing given, nothing spent");
    assert!(has_line(&crate::inspect::card(&g, 0), "WORN SLOTS ARE FULL"), "the card says so first");
}

#[test]
fn a_piece_needs_a_free_slot_and_only_the_best_bonus_to_an_ability_counts() {
    let (mut g, r) = at_rack("cloth_gloves", 1000.0);
    let dex = g.monsters[0].stats.dex;
    g.monsters[0].gear = vec!["cloth_gloves".into(), "rope_belt".into(), "straw_sandals".into()];
    // A finer pair, as a rack elsewhere will show: nothing takes another's place (Henry), so with
    // every worn slot full it needs one freed.
    g.world.racks[r].piece = "gloves_of_dexterity";
    assert!(has_line(&crate::inspect::card(&g, 0), "ALL 3 OF MY WORN SLOTS ARE FULL"));
    g.script("u");
    assert_eq!((g.monsters[0].gear.len(), g.communes[0].renown), (3, 1000.0), "{}", said(&g, 0));
    // With one free, both pairs are worn, and the finer one's DEX is what counts.
    g.monsters[0].gear.retain(|p| p != "straw_sandals");
    g.script("u");
    assert_eq!(g.monsters[0].gear, vec!["cloth_gloves".to_string(), "rope_belt".into(), "gloves_of_dexterity".into()]);
    assert_eq!(g.communes[0].renown, 200.0);
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
    // The seam's card says beforehand what a dig will take off each tool.
    let card = crate::inspect::card(&g, 0);
    let shown = card.lines().map(|l| l.text.clone()).find(|t| t.starts_with("WEARS ")).unwrap_or_default();
    g.script("u");
    for _ in 0..5000 {
        if g.communes[0].made.contains("coal") {
            break;
        }
        g.run_ticks(1);
    }
    assert!(g.communes[0].made.contains("coal"), "dug: {}", said(&g, 0));
    let wear = g.monsters[0].tool(0).expect("the pick").wear;
    let n = format!("{wear:.2}");
    let n = n.trim_end_matches('0').trim_end_matches('.');
    assert_eq!(shown, format!("WEARS PICK {n}, SHOVEL {n}."), "the card agrees with the dig");
    wear
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
    // Mogg with nothing on: what his type's ability does (a goblin scrounges), and no more.
    let plain = knack("nothing", "weaving");
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
    assert_eq!(g.data.pieces.iter().filter(|p| p.look == "teaching").count(), g.data.skills.transferable_skills.len(), "a teaching for every talent");
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
            let (a, b) = (crate::econ::roll_quantity(&craft(&crate::econ::PLAIN_KNACK), &mut crate::econ::Rng::new(seed)), crate::econ::roll_quantity(&craft(&towel), &mut crate::econ::Rng::new(seed)));
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
        let (base, l, p) = (rolls(&crate::econ::PLAIN_KNACK), rolls(&lucky), rolls(&patient));
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
    let (mut pick, data) = (tool("pick"), crate::data::Data::load());
    pick.wear = pick.durability(&data) as f64 - 0.5;
    assert_eq!(pick.durability_left(&data), 1, "half a point left still works once more");
}

/// Tool life by material (Henry, 2026-11-05): a steel tool has twice the usual durability, an
/// elder one half, copper with tin a bronze tool's, and the head decides, not the handle. And a
/// player can find it out where it matters: the material's card, the tool's, the workshop's
/// while it makes one, and the bubble when one wears out, all with the same numbers. Durability,
/// not uses: each go wears a tool by its recipe's weight, which the plant's and the workshop's
/// cards say.
#[test]
fn a_tool_lasts_by_what_its_made_of_and_every_card_says_so() {
    let mut g = Game::new();
    let made = |g: &Game, id: &str, from: &str| crate::econ::plain(&g.data, id, from);
    assert_eq!(made(&g, "knife", "steel_ingot").durability(&g.data), 200);
    assert_eq!(made(&g, "pick", "pig_iron_ingot").durability(&g.data), 75);
    assert_eq!(made(&g, "spool", "elder_sticks").durability(&g.data), 50);
    assert_eq!(made(&g, "spool", "rhododendron_sticks").durability(&g.data), 100);
    assert_eq!(made(&g, "wooden_tub", "hemlock_log").durability(&g.data), 75);
    let mut bronze = made(&g, "knife", "copper_ingot");
    bronze.carried_tags.push("tin".into());
    assert_eq!(bronze.durability(&g.data), 150, "copper with tin is a bronze tool");
    let mut pick = made(&g, "pick", "pig_iron_ingot");
    pick.carried_tags.push("oak".into());
    assert_eq!(pick.durability(&g.data), 75, "an oak handle on a cast iron head is a cast iron pick");
    assert_eq!(made(&g, "pick", "pick").durability(&g.data), 100, "made of nothing known: the usual");

    // What a workshop making a tool could make it of, among what the world has, longest-lasting
    // first: the Cloth Reaches' sticks, and no hard metal yet (the hills aren't built).
    assert_eq!(g.durability_by_material(g.data.good("spool")).as_deref(), Some("DURABILITY BY WOOD: RHODODENDRON 100, ELDER 50."));
    assert_eq!(g.durability_by_material(g.data.good("pick")), None, "no hard metal to be had yet");
    assert_eq!(g.durability_by_material(g.data.good("wooden_tub")).as_deref(), Some("DURABILITY BY WOOD: OAK 100, HEMLOCK 75."), "not the woods still to be planted");
    assert_eq!(g.durability_by_material(g.data.good("thread")), None, "thread isn't a tool");
    // Wearing out: something lasting longer that grows here, if anything does.
    assert_eq!(g.wore_out("THE", &made(&g, "spool", "elder_sticks")), "THE ELDER SPOOL WORE OUT. RHODODENDRON LASTS LONGER: DURABILITY 100, ELDER 50.");
    assert_eq!(g.wore_out("MY", &made(&g, "pick", "pig_iron_ingot")), "MY CAST IRON PICK WORE OUT.", "no better hard metal to be had here yet");

    // The cards: a steel knife ahead, then elder sticks, then limestone, which makes no tool.
    let (x, y) = (g.monsters[0].x, g.monsters[0].y);
    g.monsters[0].facing = Dir::Right;
    let look = |g: &mut Game, item: ItemData| {
        g.world.items.retain(|it| (it.x, it.y) != (x + 1, y));
        g.world.items.push(crate::world::Item::new(x + 1, y, item));
        crate::inspect::card(g, 0)
    };
    let knife = made(&g, "knife", "steel_ingot");
    let card = look(&mut g, knife);
    assert!(has_line(&card, "DURABILITY 200/200") && has_line(&card, "STEEL: 200 (MOST 100)."), "{:?}", card.lines().map(|l| &l.text).collect::<Vec<_>>());
    let sticks = made(&g, "elder_sticks", "elder_sticks");
    let card = look(&mut g, sticks);
    assert!(has_line(&card, "TOOL DURABILITY 50 (MOST 100)."), "{:?}", card.lines().map(|l| &l.text).collect::<Vec<_>>());
    let sticks = made(&g, "rhododendron_sticks", "rhododendron_sticks");
    let card = look(&mut g, sticks);
    assert!(has_line(&card, "TOOL DURABILITY 100, AS MOST."), "{:?}", card.lines().map(|l| &l.text).collect::<Vec<_>>());
    let stone = made(&g, "limestone", "limestone");
    assert!(!has_line(&look(&mut g, stone), "TOOL DURABILITY"), "limestone makes no tool");
    // At a bench with elder sticks on its pad, making a spool: the card ranks the woods.
    let s = (0..g.world.shops.len()).find(|&s| g.world.shops[s].kind.recipes().contains(&"spool") && g.world.shops[s].frontier.is_none()).expect("a bench that makes spools");
    let (pad, spot) = (g.world.shops[s].pads[0], g.world.shops[s].spot);
    let sticks = made(&g, "elder_sticks", "elder_sticks");
    g.world.items.push(crate::world::Item::new(pad.0, pad.1, sticks));
    (g.monsters[0].x, g.monsters[0].y) = spot;
    assert_eq!(g.preview(s).recipe, "spool");
    let card = crate::inspect::card(&g, 0);
    assert!(has_line(&card, "DURABILITY BY WOOD: RHODODENDRON 100, ELDER 50."), "{:?}", card.lines().map(|l| &l.text).collect::<Vec<_>>());
    assert!(!card.lines().any(|l| l.text.starts_with("WEARS ")), "a spool takes no tools");

    // Spinning thread: the spinnery's own spool and wheel never wear, so the card says nothing
    // of wear; an elder spool hung on its peg does, and the card says by how much.
    let (mut g, s) = at_spinnery();
    let lines = |g: &Game| crate::inspect::card(g, 0).lines().map(|l| l.text.clone()).collect::<Vec<_>>();
    assert_eq!(g.preview(s).recipe, "thread");
    assert!(!lines(&g).iter().any(|t| t.starts_with("WEARS ")), "{:?}", lines(&g));
    let peg = g.world.shops[s].pegs[0];
    g.world.items.push(crate::world::Item::new(peg.0, peg.1, made(&g, "spool", "elder_sticks")));
    assert!(lines(&g).iter().any(|t| t == "WEARS SPOOL 3."), "{:?}", lines(&g));
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
    assert!((made(&seal) / made(&crate::econ::PLAIN_KNACK) - 1.1).abs() < 1e-9);
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
    assert_eq!(g.monsters[1].knack(&g.data, "spinning").talents, vec![lacks.clone()]);
    assert!(learned(&g) > slow * 1.1, "a talent the recipe wants speeds learning");
    let after = crate::inspect::forecast(&g, s, 1).expect("a forecast");
    assert!(after.secondary >= before.secondary && after.tool_quality >= before.tool_quality, "and skips the weakest helper");
    assert!(has_line(&crate::inspect::card(&g, 0), &format!("TALENT: {}", lacks.replace('_', " ").to_uppercase())));
    // One it was born with can't be taught.
    let has = g.monsters[1].skills.transferable[0].clone();
    assert_eq!(g.piece_refusal(1, g.data.piece(&format!("teaching_{has}"))), Some(format!("I HAVE A TALENT FOR {} ALREADY.", has.replace('_', " ").to_uppercase())));
    // A second needs an inside slot of its own: Bruna, a cyclops, has one. Where there's room
    // (an elf has three), each counts as a talent.
    let other = g.data.skills.transferable_skills.iter().find(|t| **t != lacks && !g.monsters[1].skills.transferable.contains(t)).expect("another").clone();
    assert!(g.piece_refusal(1, g.data.piece(&format!("teaching_{other}"))).is_some_and(|w| w.contains("SLOT IS FULL")));
    g.monsters[1].gear.push(format!("teaching_{other}"));
    assert_eq!(g.monsters[1].knack(&g.data, "spinning").talents, vec![lacks.clone(), other.clone()]);
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

/// Mogg on open ground beside `at`, facing it, beside `racks` racks there.
fn beside(g: &mut Game, at: (i32, i32), racks: usize) {
    let d = crate::world::DIRS.iter().copied().find(|d| {
        let (x, y) = (at.0 - d.delta().0, at.1 - d.delta().1);
        g.world.walkable(x, y) && g.monster_at(x, y).is_none() && g.world.shop_at_spot(x, y).is_none() && g.racks_beside_tile((x, y)) == racks
    });
    let d = d.unwrap_or_else(|| panic!("open ground beside {at:?}"));
    (g.monsters[0].x, g.monsters[0].y, g.monsters[0].facing) = (at.0 - d.delta().0, at.1 - d.delta().1, d);
}

#[test]
fn each_working_townsperson_writes_down_their_trades_secret() {
    let workers = Game::new().world.workers.clone();
    assert!(!workers.is_empty());
    for (n, w) in workers.iter().enumerate() {
        let mut g = Game::new();
        // Their trade is what they're best at, and its secret stands by their work.
        let trade = w.skills.iter().max_by(|a, b| a.1.total_cmp(&b.1)).expect("a trade").0;
        assert_eq!(w.secret.1, format!("secret_{trade}"), "{}'s secret is their trade's", w.name);
        let r = g.rack_at(w.secret.0 .0, w.secret.0 .1).unwrap_or_else(|| panic!("a stand for {}'s secret", w.name));
        assert_eq!(g.world.racks[r].piece, w.secret.1);
        let near = |(x, y): (i32, i32)| (x - w.secret.0 .0).abs() + (y - w.secret.0 .1).abs() <= 4;
        assert!(near(w.at) || g.world.boards.iter().any(|b| near((b.x, b.y)) && b.title.starts_with(w.name)), "{}'s stand is by their work", w.name);
        // What they say, and their card, tell where it is.
        let j = g.monsters.iter().position(|m| m.worker == Some(n)).expect("their monster");
        let name = g.data.piece(w.secret.1).name.to_uppercase();
        let says = (0..12).flat_map(|t| { g.communes[0].idea_turn = t; g.worker_says(j, 0) }).collect::<Vec<_>>().join(" ");
        assert!(says.contains(&format!("I'VE WRITTEN THE {name} ON THE SCROLL NEARBY")), "{says}");
        let at = (g.monsters[j].x, g.monsters[j].y);
        beside(&mut g, at, 0);
        let card = crate::inspect::card(&g, 0);
        assert!(has_line(&card, &format!("{}'S {name} IS ON THE SCROLL NEARBY (100 RENOWN).", w.name)), "{:?}", card.lines().map(|l| &l.text).collect::<Vec<_>>());
        // The scroll's card says whose it is, and Space learns it for renown, as at any rack.
        beside(&mut g, w.secret.0, 1);
        g.communes[0].renown = 150.0;
        let card = crate::inspect::card(&g, 0);
        assert_eq!(card.title, name);
        assert!(has_line(&card, &format!("WRITTEN DOWN BY {} THE {}.", w.name, w.title)), "{:?}", card.lines().map(|l| &l.text).collect::<Vec<_>>());
        g.script("u");
        assert_eq!(g.monsters[0].gear, vec![w.secret.1.to_string()], "{}", said(&g, 0));
        assert_eq!(g.communes[0].renown, 50.0);
        assert!(said(&g, 0).contains(&format!("LEARNS THE {name}")), "{}", said(&g, 0));
    }
}

/// The average of `n` quantity rolls of `good` by Mogg with these skills and pieces.
fn average_count(g: &Game, good: &str, skill: f64, pieces: &[&str], n: u64) -> f64 {
    let st = g.monsters[0].abilities(&g.data);
    let recipe = g.data.good(good);
    let mut sk = g.monsters[0].skills.clone();
    sk.applied.insert(recipe.primary_applied_skill.clone(), skill);
    for k in &recipe.secondary_applied_skills {
        sk.applied.insert(k.clone(), skill);
    }
    sk.specific.insert(good.into(), skill);
    let knack = pieces.iter().fold(crate::econ::PLAIN, |k, p| k.with(&g.data.piece(p).does));
    let craft = crate::econ::Craft { recipe, crafter: crate::econ::Crafter { id: 1, stats: &st, skills: &sk, knack: &knack }, inputs: Vec::new(), tools: Vec::new(), workshop: None };
    (1..=n).map(|seed| crate::econ::roll_quantity(&craft, &mut crate::econ::Rng::new(seed)) as f64).sum::<f64>() / n as f64
}

#[test]
fn a_secret_lifts_quality_where_its_goods_have_it_and_how_many_where_not() {
    let g = Game::new();
    // Each craft's secret lifts what its own goods roll: quality where they have it, how many
    // where they don't, and a craft with both kinds gets both.
    for p in g.data.pieces.iter().filter(|p| p.id.starts_with("secret_")) {
        let skill = p.does.at.as_deref().expect("a secret is of one craft");
        let mine: Vec<_> = g.data.goods.iter().filter(|r| r.primary_applied_skill == skill).collect();
        assert_eq!(p.does.quality != 0, mine.iter().any(|r| r.has_quality), "{}: quality", p.id);
        assert_eq!(p.does.quantity != 0, mine.iter().any(|r| !r.has_quality), "{}: yield", p.id);
    }
    // Coal has no quality roll: the Secret of Prospecting sends its count roll 10% further before
    // it's rounded. The more skill spreads the roll, the more often that tips it into one more:
    // skilled, about 10% more coal; partway there, less; new to it, the roll sits on 2 and 2.2
    // still rounds to 2.
    let secret = ["secret_prospecting"];
    let gain = |skill| average_count(&g, "coal", skill, &secret, 4000) / average_count(&g, "coal", skill, &[], 4000);
    let (skilled, partway) = (gain(0.9), gain(0.6));
    assert!((1.07..1.16).contains(&skilled), "skilled: {skilled:.3}");
    assert!(partway > 1.01 && partway < skilled, "partway: {partway:.3}");
    assert_eq!(average_count(&g, "coal", 0.0, &secret, 200), average_count(&g, "coal", 0.0, &[], 200), "new to it");
    // Plums have a quality roll, so gathering's secret lifts their quality and not how many.
    let gathering = ["secret_gathering"];
    assert!(g.data.good("plums").has_quality && !g.data.good("plums").is_fixed_quantity);
    assert_eq!(average_count(&g, "plums", 0.6, &gathering, 400), average_count(&g, "plums", 0.6, &[], 400));
    let knack = crate::econ::PLAIN.with(&g.data.piece("secret_gathering").does);
    assert!((knack.quality - 1.1).abs() < 1e-9 && (knack.quantity - 1.1).abs() < 1e-9);
    // The cards say it in a line, the secret's name saying which craft: "QUALITY OR YIELD" where
    // its craft's goods are of both kinds.
    assert_eq!(g.data.piece("secret_prospecting").effect_text(), "+10% YIELD");
    assert_eq!(g.data.piece("secret_gathering").effect_text(), "+10% QUALITY OR YIELD");
    assert_eq!(g.data.piece("secret_spinning").effect_text(), "+10% QUALITY");
    // On the rope belt's rack, for the test.
    let (mut g, r) = at_rack("rope_belt", 0.0);
    g.world.racks[r].piece = "secret_gathering";
    let card = crate::inspect::card(&g, 0);
    assert!(has_line(&card, "+10% QUALITY OR YIELD"), "{:?}", card.lines().map(|l| &l.text).collect::<Vec<_>>());
}

#[test]
fn the_coal_seam_lists_only_what_helps_there() {
    let mut g = Game::new();
    let p = g.world.plants.iter().position(|p| p.good == "coal" && p.owner.is_none()).expect("a seam for anyone");
    let (x, y) = (g.world.plants[p].x, g.world.plants[p].y);
    let m = &mut g.monsters[0];
    (m.x, m.y, m.facing) = (x, y, Dir::Down);
    m.gear = vec!["secret_prospecting".into()];
    m.hands = vec![Some(Held::Piece("magnifying_glass".into())), Some(Held::Piece("lucky_charm".into()))];
    let card = crate::inspect::card(&g, 0);
    let lines: Vec<&String> = card.lines().map(|l| &l.text).collect();
    // Coal's count is rolled, its quality isn't: the secret's yield helps, the glass and the
    // charm don't.
    assert!(has_line(&card, "SECRET OF PROSPECTING: +10% YIELD"), "{lines:?}");
    assert!(!has_line(&card, "MAGNIFYING GLASS") && !has_line(&card, "LUCKY CHARM"), "{lines:?}");
}

/// Coal pushed into wagon `w` by Mogg until it's full: how many went in.
fn fill_with_coal(g: &mut Game, w: usize) -> usize {
    let id = g.monsters[0].id;
    g.world.wagons[w].stock.clear();
    for n in 0.. {
        g.world.items.push(crate::world::Item::new(0, 0, tool("coal")));
        if g.load_wagon(w, g.world.items.len() - 1, id).is_err() {
            g.world.items.pop();
            return n;
        }
    }
    unreachable!()
}

#[test]
fn a_backpack_carries_more_a_feathered_cap_pulls_harder_and_a_shoulder_pole_pushes_heavier() {
    let mut g = Game::new();
    let (w, start) = (0, (g.monsters[0].x, g.monsters[0].y));
    // A backpack: the wagon its wearer leads, or led last and parked, holds half again as much.
    g.world.wagons[w].last_leader = Some(g.monsters[0].id);
    let plain = fill_with_coal(&mut g, w);
    g.monsters[0].gear = vec!["backpack".into()];
    assert_eq!(fill_with_coal(&mut g, w), plain * 3 / 2, "the town's wagon, {plain} coal without");
    let (bx, by) = g.world.wagons[w].at;
    (g.monsters[0].x, g.monsters[0].y, g.monsters[0].facing) = (bx, by - 1, Dir::Down);
    let card = crate::inspect::card(&g, 0);
    let lines: Vec<&String> = card.lines().map(|l| &l.text).collect();
    assert!(has_line(&card, &format!("OF {:.1}", g.world.wagons[w].room * 1.5)) && has_line(&card, "MOGG'S BACKPACK: +50% WAGON ROOM"), "{lines:?}");
    g.monsters[0].gear.clear();
    // A feathered cap: a load that slowed its leader goes at full speed.
    let cap = g.pull_capacity(0, w);
    let logs = (cap * 1.3 / 100.0).ceil() as usize;
    g.world.wagons[w].stock = vec![(tool("oak_log"), String::new()); logs];
    assert!(g.wagon_ticks(0, w) > crate::game::PUSH_TICKS, "{logs} logs slow a plain leader");
    g.monsters[0].gear = vec!["feathered_cap".into()];
    assert!((g.pull_capacity(0, w) / cap - 1.5).abs() < 1e-9);
    assert_eq!(g.wagon_ticks(0, w), crate::game::PUSH_TICKS, "and not one in a feathered cap");
    g.monsters[0].gear.clear();
    // A shoulder pole: a good that slowed its pusher goes at full speed; it takes a hand.
    let push = g.push_capacity(0);
    let heavy = g.data.goods.iter().filter(|r| r.raw_material_density.is_some()).map(|r| tool(&r.id)).find(|d| {
        let wt = crate::econ::weight(g.data.good(&d.good), d);
        wt > push && wt <= push * 1.5
    });
    let heavy = heavy.expect("a good a little too heavy for Mogg");
    assert!(g.push_ticks(0, &heavy) > crate::game::PUSH_TICKS, "{} slows Mogg", heavy.good);
    g.monsters[0].hands[0] = Some(Held::Piece("shoulder_pole".into()));
    assert_eq!(g.push_ticks(0, &heavy), crate::game::PUSH_TICKS, "but not with a shoulder pole");
    // A good's card says what the one looking pushes at full speed, and what helps.
    (g.monsters[0].x, g.monsters[0].y, g.monsters[0].facing) = (start.0, start.1, Dir::Right);
    g.world.items.push(crate::world::Item::new(start.0 + 1, start.1, heavy));
    let card = crate::inspect::card(&g, 0);
    let lines: Vec<&String> = card.lines().map(|l| &l.text).collect();
    assert!(has_line(&card, &format!("MOGG PUSHES {:.0} AT FULL SPEED.", push * 1.5)) && has_line(&card, "SHOULDER POLE: +50% PUSHING POWER"), "{lines:?}");
    // A backpack is worn beside a cloak, in a worn slot of its own; then all three are full.
    g.monsters[0].gear = vec!["cloak_of_charisma".into(), "headband".into()];
    assert_eq!(g.piece_refusal(0, g.data.piece("backpack")), None);
    g.monsters[0].gear.push("backpack".into());
    assert!(g.piece_refusal(0, g.data.piece("feathered_cap")).is_some_and(|w| w.contains("SLOTS ARE FULL")));
    // Hauling isn't crafting: a forecast card leaves them out.
    let (mut g, _) = at_spinnery();
    g.monsters[1].gear = vec!["backpack".into(), "feathered_cap".into()];
    g.monsters[1].hands[0] = Some(Held::Piece("shoulder_pole".into()));
    let card = crate::inspect::card(&g, 0);
    assert!(!has_line(&card, "BACKPACK") && !has_line(&card, "CAP") && !has_line(&card, "POLE"), "{:?}", card.lines().map(|l| &l.text).collect::<Vec<_>>());
}

#[test]
fn things_for_hauling_stand_outside_the_outfitter_along_the_path() {
    let mut g = Game::new();
    for piece in ["backpack", "feathered_cap", "shoulder_pole", "tool_belt"] {
        let r = g.world.racks.iter().position(|r| r.piece == piece).unwrap_or_else(|| panic!("a stand for the {piece}"));
        let (x, y) = (g.world.racks[r].x, g.world.racks[r].y);
        assert_eq!(crate::world::area_name(x, y), "TANGLEMERE");
        // Faced from the path, with no other stand beside: Space there takes it, no guessing.
        let front = (x - 1, y);
        assert!(g.world.tile(front.0, front.1).walkable() && g.racks_beside_tile(front) == 1, "{piece} faced from {front:?}");
        (g.monsters[0].x, g.monsters[0].y, g.monsters[0].facing) = (front.0, front.1, Dir::Down);
        g.communes[0].renown = 100.0;
        g.script("u");
        assert!(g.monsters[0].pieces().any(|p| p == piece), "{piece}: {}", said(&g, 0));
    }
}

#[test]
fn heavy_goods_slow_a_push_three_ticks_for_each_half_over_and_a_shoulder_pole_lifts_them() {
    use crate::wagon::heavy_ticks;
    // A goblin of STR 8 pushes 80 at full speed; with a shoulder pole, 120. Clay, an oak log,
    // limestone, a pumpkin and a copper ingot:
    for (weight, plain, pole) in [(75.0, 3, 3), (100.0, 5, 3), (125.0, 7, 4), (160.0, 9, 5), (178.0, 11, 6)] {
        assert_eq!((heavy_ticks(weight, 80.0), heavy_ticks(weight, 120.0)), (plain, pole), "weight {weight}");
    }
    assert_eq!(heavy_ticks(10_000.0, 80.0), crate::game::PULL_TICKS, "no slower than a pull");
    // The temple bell (491) is past that for a goblin, but not with a pole: 22 ticks a tile.
    assert_eq!((heavy_ticks(491.0, 80.0), heavy_ticks(491.0, 120.0)), (crate::game::PULL_TICKS, 22));
    // Pushing something clearly heavy says so; something a little slower doesn't.
    let push = |good: &str| {
        let mut g = Game::new();
        let open = |g: &Game, x: i32, y: i32| g.world.tile(x, y) == crate::world::Tile::Grass && g.monster_at(x, y).is_none() && g.world.item_at(x, y).is_none();
        let (x, y) = (1..30).flat_map(|y| (1..45).map(move |x| (x, y))).find(|&(x, y)| (0..3).all(|k| open(&g, x + k, y))).expect("open ground");
        (g.monsters[0].x, g.monsters[0].y) = (x, y);
        g.world.items.push(crate::world::Item::new(x + 1, y, tool(good)));
        let mut right = Input::default();
        (right.pressed[3], right.held[3]) = (true, true);
        g.press(0, right);
        let want = heavy_ticks(crate::econ::weight(g.data.good(good), &tool(good)), g.push_capacity(0));
        assert_eq!(g.monsters[0].slide.map(|s| s.total), Some(want), "{good}");
        (want, g.status(0).0)
    };
    let (ticks, said) = push("pumpkin");
    assert!(ticks >= crate::game::HEAVY_GOING_TICKS && said.ends_with("PUSHING. HEAVY GOING"), "a pumpkin, {ticks} ticks: {said}");
    let (ticks, said) = push("oak_log");
    assert!(ticks > crate::game::PUSH_TICKS && !said.contains("HEAVY"), "an oak log, {ticks} ticks: {said}");
}

#[test]
fn a_tool_belt_is_a_third_hand_and_comes_off_only_with_somewhere_for_what_it_held() {
    let mut g = Game::new();
    let r = g.world.racks.iter().position(|r| r.piece == "tool_belt").expect("the tool belt's stand");
    let (x, y) = (g.world.racks[r].x, g.world.racks[r].y);
    (g.monsters[0].x, g.monsters[0].y, g.monsters[0].facing) = (x - 1, y, Dir::Right);
    g.communes[0].renown = 100.0;
    assert_eq!(g.monsters[0].hands.len(), 2);
    g.script("u");
    assert_eq!(g.monsters[0].gear, vec!["tool_belt".to_string()], "{}", said(&g, 0));
    assert_eq!(g.monsters[0].hands.len(), 3, "a third hand");
    assert_eq!(g.sheet_lines(0).iter().filter(|l| matches!(l, crate::gear::SheetLine::Hand(_))).count(), 3);
    // A pick and a shovel to dig coal with, and a shoulder pole on the belt: all three at once.
    g.monsters[0].hands = vec![Some(Held::Tool(tool("pick"))), Some(Held::Tool(tool("shovel"))), Some(Held::Piece("shoulder_pole".into()))];
    assert_eq!(g.held_tools(0, g.data.good("coal")), Ok(vec![0, 1]));
    assert!(g.monsters[0].knack(&g.data, "hauling").push > 1.4, "the pole helps from the belt");
    // Three things held and two hands without it: the belt stays on, and says why. A belt of
    // strength would go beside it, in a worn slot of its own.
    g.take_off(0, "tool_belt", &mut Vec::new());
    assert!(g.monsters[0].gear.contains(&"tool_belt".to_string()) && said(&g, 0) == crate::gear::TOO_FULL, "{}", said(&g, 0));
    assert!(g.piece_refusal(0, g.data.piece("belt_of_strength")).is_none_or(|w| !w.contains("SLOT") && w != crate::gear::TOO_FULL));
    // With a hand free, it comes off, and what it held goes to that hand.
    g.monsters[0].hands[1] = None;
    g.take_off(0, "tool_belt", &mut Vec::new());
    assert!(g.monsters[0].gear.is_empty(), "{}", said(&g, 0));
    assert_eq!(g.monsters[0].hands.len(), 2);
    assert!(g.monsters[0].hands.iter().any(|h| matches!(h, Some(Held::Piece(p)) if p == "shoulder_pole")) && g.monsters[0].holding() == 2);
}

#[test]
fn a_work_song_speeds_everyone_working_near_its_singer() {
    use crate::work::SONG_REACH;
    let mut g = Game::new();
    // The temple keeps it, and its card says what it does.
    let r = g.world.racks.iter().position(|r| r.piece == "work_song").expect("its stand");
    assert_eq!(crate::world::area_name(g.world.racks[r].x, g.world.racks[r].y), "TEMPLE TOWN");
    assert_eq!(g.data.piece("work_song").effect_text(), "-5% TIME FOR ALL NEARBY");
    // Old Spoke, of the town's own commune, at his bench, and Mogg beside him with the song,
    // not working himself: he sings while anyone near him works.
    let spoke = g.monsters.iter().position(|m| m.name == "OLD SPOKE").expect("Old Spoke");
    let plain = g.job_ticks("wagon_wheel", spoke);
    let (sx, sy) = (g.monsters[spoke].x, g.monsters[spoke].y);
    (g.monsters[0].x, g.monsters[0].y) = (sx + 2, sy);
    g.monsters[0].gear = vec!["work_song".into()];
    assert!(!g.singing(0), "with nobody near him at work, he's quiet");
    g.monsters[spoke].busy = true;
    assert!(g.singing(0), "Old Spoke at work, Mogg sings");
    let sung = g.job_ticks("wagon_wheel", spoke);
    assert!((sung as f64 / plain as f64 - 0.95).abs() < 0.02, "{plain} ticks to {sung}");
    (g.monsters[0].x, g.monsters[0].y) = (sx + SONG_REACH + 1, sy);
    assert_eq!(g.job_ticks("wagon_wheel", spoke), plain, "out of its reach");
    // Two singers are no faster than one.
    (g.monsters[0].x, g.monsters[0].y) = (sx + 2, sy);
    (g.monsters[1].x, g.monsters[1].y) = (sx - 2, sy);
    g.monsters[1].gear = vec!["work_song".into()];
    assert_eq!(g.job_ticks("wagon_wheel", spoke), sung);
    // The singer works faster too, as it starts.
    g.monsters[spoke].busy = false;
    g.monsters[1] = Game::new().monsters.swap_remove(1);
    let own = g.job_ticks("thread", 0);
    g.monsters[0].gear.clear();
    assert!((own as f64 / g.job_ticks("thread", 0) as f64 - 0.95).abs() < 0.02);
    // At the spinnery, Bruna's card says whose song carries there; as she spins, notes rise
    // beside Mogg, who isn't working.
    let (mut g, _) = at_spinnery();
    (g.monsters[0].x, g.monsters[0].y) = (g.monsters[1].x + 1, g.monsters[1].y + 1);
    g.monsters[0].gear = vec!["work_song".into()];
    let card = crate::inspect::card(&g, 0);
    assert!(has_line(&card, "MOGG'S WORK SONG: -5% TIME"), "{:?}", card.lines().map(|l| &l.text).collect::<Vec<_>>());
    let notes = |g: &Game| g.world.particles.iter().filter(|p| matches!(p.kind, crate::world::PKind::Note)).count();
    for _ in 0..30 {
        g.update(&[]);
    }
    assert_eq!(notes(&g), 0, "nobody working, no song");
    let mut space = Input::default();
    space.use_pressed = true;
    g.press(0, space);
    assert!(g.monsters[1].busy && !g.monsters[0].busy, "Bruna spinning, Mogg not");
    for _ in 0..30 {
        g.update(&[]);
    }
    assert!(notes(&g) > 0, "notes beside Mogg while Bruna works");
}

#[test]
fn each_monster_type_has_its_small_ability() {
    // A goblin scrounges: about one gather in five turns up one good more. The same monster as a
    // troll, which has no ability, digs the same coal for comparison.
    let dig = |kind: &str| {
        let mut g = Game::new();
        g.monsters[0].kind = kind.into();
        let p = g.world.plants.iter().position(|p| p.good == "coal" && p.owner.is_none()).expect("a seam");
        let skills = g.monsters[0].skills.clone();
        let mut total = 0;
        for _ in 0..2000 {
            g.monsters[0].skills = skills.clone();
            g.world.plants[p].stock.clear();
            g.world.plants[p].harvest = Some(crate::world::Job { monster: 0, recipe: "coal".into(), t: 0, total: 1 });
            g.finish_harvest(p);
            total += g.world.plants[p].stock.len();
        }
        (total as f64 / 2000.0, said(&g, 0))
    };
    let ((scrounger, _), (plain, _)) = (dig("goblin"), dig("troll"));
    assert!((scrounger - plain - 0.2).abs() < 0.05, "{plain:.3} coal a dig, {scrounger:.3} for a goblin");
    // An elf forgets half as much; a cyclops pushes half again as heavy; an orc leads a wagon by
    // its STR. The same monster as each type, its own abilities the same.
    let mut g = Game::new();
    let as_kind = |g: &mut Game, kind: &str| g.monsters[0].kind = kind.to_string();
    as_kind(&mut g, "goblin");
    let (forgets, pushes, pulls) = (g.monsters[0].knack(&g.data, "spinning").forgetting, g.push_capacity(0), g.pull_capacity(0, 0));
    as_kind(&mut g, "elf");
    assert!((g.monsters[0].knack(&g.data, "spinning").forgetting / forgets - 0.5).abs() < 1e-9);
    as_kind(&mut g, "cyclops");
    assert!((g.push_capacity(0) / pushes - 1.5).abs() < 1e-9);
    as_kind(&mut g, "orc");
    let st = g.monsters[0].abilities(&g.data);
    assert!((g.pull_capacity(0, 0) / pulls - st.str as f64 / st.cha as f64).abs() < 1e-9);
    assert_eq!(crate::inspect::ability_uses(&g, crate::data::STR, g.monsters[0].leads_by(&g.data)).split(", ").take(2).collect::<Vec<_>>(), vec!["PUSHING", "LEADING WAGONS"]);
    assert!(!crate::inspect::ability_uses(&g, crate::data::CHA, g.monsters[0].leads_by(&g.data)).contains("WAGONS"));
    // A monster's card and a newcomer's at the hall say what its type does; so does a seam's
    // card, for a goblin.
    as_kind(&mut g, "goblin");
    let (mx, my) = (g.monsters[0].x, g.monsters[0].y);
    (g.monsters[1].x, g.monsters[1].y, g.monsters[0].facing) = (mx + 1, my, Dir::Right);
    let card = crate::inspect::card(&g, 0);
    let ability = |kind: &str| g.data.monster_types[kind].ability.as_ref().map(|a| a.line());
    let bruna = g.monsters[1].kind.clone();
    assert!(has_line(&card, &ability(&bruna).expect("an ability")), "{:?}", card.lines().map(|l| &l.text).collect::<Vec<_>>());
    let k = g.applicants.iter().position(|a| a.as_ref().is_some_and(|a| ability(&a.kind).is_some())).expect("a newcomer with an ability");
    let (sx, sy) = g.world.stands[k];
    (g.monsters[0].x, g.monsters[0].y, g.monsters[0].facing) = (sx, sy + 1, Dir::Up);
    let kind = g.applicants[k].as_ref().map(|a| a.kind.clone()).unwrap_or_default();
    assert!(has_line(&crate::inspect::card(&g, 0), &ability(&kind).unwrap_or_default()));
    let p = g.world.plants.iter().position(|p| p.good == "coal" && p.owner.is_none()).expect("a seam");
    (g.monsters[0].x, g.monsters[0].y) = (g.world.plants[p].x, g.world.plants[p].y);
    assert!(has_line(&crate::inspect::card(&g, 0), "SCROUNGER: 1 GATHER IN 5 GIVES 1 MORE"));
}

/// The varieties batch (Henry, 2026-11-05): what's made of a new material says so, and oil says
/// what it was pressed from.
#[test]
fn new_varieties_carry_their_names_into_what_is_made_of_them() {
    let g = Game::new();
    let made = |id: &str, from: &str| crate::econ::plain(&g.data, id, from);
    let name = |item: &ItemData| crate::inspect::display_name(&g, item);
    assert_eq!(name(&made("oil", "tung_nuts")), "TUNG OIL");
    assert_eq!(name(&made("oil", "sesame_seeds")), "SESAME OIL");
    assert_eq!(name(&made("planks", "jujube_log")), "JUJUBE PLANKS");
    let willow = made("spool", "willow_sticks");
    assert_eq!((name(&willow), willow.durability(&g.data)), ("WILLOW SPOOL".to_string(), 75));
    assert_eq!(made("stone_anvil", "basalt").durability(&g.data), 125, "basalt is hard stone");
}
