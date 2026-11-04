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
    ItemData { good: id.into(), quality: 1.0, carried_tags: vec!["copper".into()], raw, depth: 1, value: 6.0, shares: Vec::new(), wear: 0, haul: Vec::new(), carted: None }
}

#[test]
fn the_outfitter_gives_a_piece_for_renown_and_it_raises_an_ability() {
    let (mut g, _) = at_rack("cloth_gloves", 120.0);
    let card = crate::inspect::card(&g, 0);
    assert_eq!(card.title, "CLOTH GLOVES");
    assert!(has_line(&card, "+1 DEX") && has_line(&card, "GIVEN AT 50 RENOWN. YOU HAVE 120.") && has_line(&card, "SPACE: MOGG PUTS IT ON, FOR GOOD."), "{:?}", card.lines().map(|l| &l.text).collect::<Vec<_>>());
    let thread = g.data.good("thread").clone();
    let (dex, before) = (g.monsters[0].stats.dex, crate::econ::production_seconds(&thread, &g.monsters[0].abilities(&g.data)));
    g.script("u");
    assert_eq!(g.monsters[0].gear, vec!["cloth_gloves".to_string()]);
    assert_eq!(g.communes[0].renown, 70.0, "given at 50 renown");
    assert!(said(&g, 0).contains("MOGG PUTS ON THE CLOTH GLOVES. +1 DEX!"), "{}", said(&g, 0));
    assert_eq!(g.monsters[0].abilities(&g.data).dex, dex + 1);
    assert_eq!(g.monsters[0].stats.dex, dex, "its own DEX is unchanged; the gloves add to it");
    assert!(crate::econ::production_seconds(&thread, &g.monsters[0].abilities(&g.data)) < before, "nimbler fingers spin faster");
    let card = crate::inspect::card(&g, 0);
    assert!(has_line(&card, "I HAVE THE CLOTH GLOVES ALREADY."));
    // Its own card says what it has on.
    g.monsters[0].facing = Dir::Up;
    assert!(has_line(&crate::inspect::card(&g, 0), "HAS ON CLOTH GLOVES"));
}

#[test]
fn space_beside_a_rack_turns_to_it_and_the_outfitter_just_talks() {
    let (mut g, _) = at_rack("rope_belt", 60.0);
    g.monsters[0].facing = Dir::Left;
    g.script("u");
    assert_eq!(g.monsters[0].gear, vec!["rope_belt".to_string()], "Space turned Mogg to the rack");
    // Old Thimble stands between the racks; Space only talks, adding nothing to the book.
    let t = g.world.townsfolk.iter().position(|t| t.kind == Folk::Outfitter).expect("the outfitter");
    let (tx, ty) = (g.world.townsfolk[t].x, g.world.townsfolk[t].y);
    (g.monsters[0].x, g.monsters[0].y, g.monsters[0].facing) = (tx, ty - 1, Dir::Down);
    let heard = g.communes[0].heard.clone();
    g.script("u");
    assert_eq!(g.communes[0].heard, heard);
    assert!(g.outfitter_says(0)[0].contains("YOURS HAS 10."), "{:?}", g.outfitter_says(0));
    assert_eq!(crate::inspect::card(&g, 0).title, "OLD THIMBLE THE OUTFITTER");
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
    // A towel, as the crafting pieces will be: held, not worn.
    let (mut g, r) = at_rack("cloth_gloves", 1000.0);
    let mut towel = g.data.piece("cloth_gloves").clone();
    (towel.id, towel.name, towel.slot, towel.kind) = ("towel".into(), "Towel".into(), crate::data::Slot::Hand, "towel".into());
    g.data.pieces.push(towel);
    g.world.racks[r].piece = "towel";
    g.monsters[0].hands = vec![Some(Held::Tool(tool("pick"))), Some(Held::Tool(tool("shovel")))];
    g.script("u");
    assert_eq!(said(&g, 0), "MY HANDS ARE FULL. PUT SOMETHING DOWN ON MY SHEET (C).");
    g.monsters[0].hands[1] = None;
    g.script("u");
    assert!(matches!(&g.monsters[0].hands[1], Some(Held::Piece(id)) if id == "towel"), "{:?}", g.monsters[0].hands);
    assert!(said(&g, 0).starts_with("MOGG TAKES THE TOWEL."), "{}", said(&g, 0));
    assert_eq!(g.monsters[0].abilities(&g.data).dex, g.monsters[0].stats.dex + 1, "a held piece counts as a worn one does");
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
