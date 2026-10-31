//! Behavior of the crafting rules: the Python port's tested behaviors (skills grow and never
//! go negative, INT speeds learning, WIS slows forgetting, transferable skills count) plus the
//! DESIGN.md changes (averaged and capped mean, wider destabilizer spread, workshop as a tool).

use super::*;
use crate::data::{Data, GoodType, Stats};

fn close(a: f64, b: f64) -> bool {
    (a - b).abs() < 1e-9
}

fn stats(d: &Data, kind: &str) -> Stats {
    d.monster_types[kind].stats.clone()
}

fn item(d: &Data, id: &str, quality: f64, carried: &[&str]) -> ItemData {
    let g = d.good(id);
    let (raw, depth) = match g.raw_material_base_value {
        Some(v) => (vec![RawMaterial { good: id.into(), base_value: v, density: g.raw_material_density.unwrap_or(0.0) }], 0),
        None => (Vec::new(), 1),
    };
    ItemData { good: id.into(), quality, carried_tags: carried.iter().map(|s| s.to_string()).collect(), raw, depth, value: 0.0, shares: Vec::new(), wear: 0 }
}

/// Every skill the recipe uses at `level`, with the given transferable skills.
fn skilled(recipe: &GoodType, level: f64, transferable: &[&str]) -> Skills {
    let mut s = Skills { transferable: transferable.iter().map(|t| t.to_string()).collect(), ..Skills::default() };
    s.applied.insert(recipe.primary_applied_skill.clone(), level);
    for k in &recipe.secondary_applied_skills {
        s.applied.insert(k.clone(), level);
    }
    s.specific.insert(recipe.id.clone(), level);
    s
}

fn craft<'a>(recipe: &'a GoodType, st: &'a Stats, sk: &'a Skills, inputs: Vec<&'a ItemData>, tools: Vec<&'a ItemData>, workshop: Option<&'a Workshop>) -> Craft<'a> {
    Craft { recipe, crafter: Crafter { id: 1, stats: st, skills: sk }, inputs, tools, workshop }
}

#[test]
fn skill_values_default_to_zero_and_never_go_negative() {
    let mut s = Skills::default();
    assert_eq!(s.applied("spinning"), 0.0);
    s.applied.insert("spinning".into(), 0.2);
    s.forgotten = 0.5;
    assert_eq!(s.applied("spinning"), 0.0);
    assert_eq!(s.specific("thread"), 0.0);
}

#[test]
fn crafting_teaches_every_skill_and_secondaries_learn_slower() {
    let d = Data::load();
    let thread = d.good("thread");
    let st = stats(&d, "goblin");
    let mut sk = skilled(thread, 0.3, &[]);
    let before = (sk.applied("spinning"), sk.specific("thread"), sk.applied("textiles"));
    let l = learn(&mut sk, &st, thread, &d, production_seconds(thread, &st));
    assert!(sk.applied("spinning") > before.0 && sk.specific("thread") > before.1 && sk.applied("textiles") > before.2);
    assert!(l.secondary.iter().all(|(_, g)| *g < l.primary), "secondary {:?} vs primary {}", l.secondary, l.primary);
    assert!(l.forgotten > 0.0);
}

#[test]
fn a_novice_learns_its_specific_skill_only_once_it_knows_the_craft() {
    let d = Data::load();
    let thread = d.good("thread");
    let st = stats(&d, "goblin");
    let mut sk = Skills::default();
    let l = learn(&mut sk, &st, thread, &d, 300.0);
    assert_eq!(l.specific, 0.0, "specific learning scales with the primary skill");
    assert!(l.primary > 0.0);
}

#[test]
fn intelligence_speeds_learning_and_wisdom_slows_forgetting() {
    let d = Data::load();
    let thread = d.good("thread");
    let dull = Stats { str: 10, dex: 10, con: 10, int: 8, wis: 8, cha: 10 };
    let bright = Stats { int: 18, wis: 18, ..dull.clone() };
    let (mut a, mut b) = (Skills::default(), Skills::default());
    let la = learn(&mut a, &dull, thread, &d, 600.0);
    let lb = learn(&mut b, &bright, thread, &d, 600.0);
    assert!(lb.primary > la.primary);
    assert!(lb.forgotten < la.forgotten);
}

#[test]
fn forgetting_drains_skills_that_are_not_practiced() {
    let d = Data::load();
    let thread = d.good("thread");
    let st = stats(&d, "goblin");
    let mut sk = skilled(thread, 0.5, &[]);
    sk.applied.insert("blacksmithing".into(), 0.8);
    for _ in 0..200 {
        learn(&mut sk, &st, thread, &d, production_seconds(thread, &st));
    }
    assert!(sk.applied("blacksmithing") < 0.6, "unused skill kept {}", sk.applied("blacksmithing"));
    assert!(sk.applied("spinning") > 0.5);
}

#[test]
fn production_time_follows_dex_and_int() {
    let d = Data::load();
    let goblin = stats(&d, "goblin");
    let expected = 300.0 * 10.0 / goblin.dex as f64 * 30.0 / (20.0 + goblin.int as f64);
    assert!(close(production_seconds(d.good("thread"), &goblin), expected));
}

#[test]
fn a_novice_makes_low_quality_goods_from_good_materials() {
    let d = Data::load();
    let thread = d.good("thread");
    let st = stats(&d, "goblin");
    let sk = Skills::default();
    let fiber = item(&d, "cotton_slivers", 0.9, &["cotton"]);
    let (spool, spinner) = (item(&d, "spool", 0.9, &[]), item(&d, "spinning_wheel", 0.9, &[]));
    let f = forecast(&craft(thread, &st, &sk, vec![&fiber], vec![&spool, &spinner], None));
    assert_eq!(f.mean, 0.0);
    let wis_str = st.wis as f64 + st.str as f64 * 0.25;
    assert!(close(f.expected(&st), wis_str / 25.0 * 0.25));
}

#[test]
fn the_mean_averages_the_material_and_tool_terms() {
    let d = Data::load();
    let thread = d.good("thread");
    let st = stats(&d, "goblin");
    let sk = skilled(thread, 0.8, &[]);
    let fiber = item(&d, "cotton_slivers", 0.9, &[]);
    let (spool, spinner) = (item(&d, "spool", 0.6, &[]), item(&d, "spinning_wheel", 0.6, &[]));
    let f = forecast(&craft(thread, &st, &sk, vec![&fiber], vec![&spool, &spinner], None));
    let ability = (st.get(thread.relevant_ability_score) as f64 / thread.difficulty_rating as f64).min(ABILITY_FACTOR_CAP);
    let material = 0.9 * 0.8 * 0.8;
    let tool = 0.6 * 0.8 * ability;
    assert!(close(f.mean, (material + tool) / 2.0), "{} vs {}", f.mean, (material + tool) / 2.0);
}

#[test]
fn the_soft_cap_bounds_the_mean_and_tool_loops_settle() {
    assert_eq!(soft_cap(0.7), 0.7);
    assert!(soft_cap(10.0) < QUALITY_MEAN_CAP && soft_cap(1.5) > 1.0);

    // A Metal Hammer made with the previous Metal Hammer, by a master with every match.
    let d = Data::load();
    let hammer = d.good("metal_hammer");
    let st = Stats { str: 18, dex: 18, con: 18, int: 18, wis: 18, cha: 18 };
    let sk = skilled(hammer, 0.95, &hammer.transferable_skills.iter().map(String::as_str).collect::<Vec<_>>());
    let inputs: Vec<ItemData> = hammer.input_groups().map(|_| item(&d, "oak_log", 1.0, &[])).collect();
    let mut q = 1.0;
    let mut last = 0.0;
    for _ in 0..40 {
        let tools: Vec<ItemData> = hammer.tools_required_tags.iter().map(|_| item(&d, "metal_hammer", q, &[])).collect();
        last = q;
        q = forecast(&craft(hammer, &st, &sk, inputs.iter().collect(), tools.iter().collect(), None)).mean;
        assert!(q < QUALITY_MEAN_CAP, "hammer quality {q}");
    }
    assert!((q - last).abs() < 0.01, "still climbing: {last} -> {q}");
}

#[test]
fn destabilizers_widen_the_spread() {
    let d = Data::load();
    let thread = d.good("thread");
    assert_eq!(thread.destabilizer_skills, vec!["sericulture".to_string()]);
    let st = stats(&d, "goblin");
    let mut sk = skilled(thread, 0.8, &[]);
    let fiber = item(&d, "cotton_slivers", 0.9, &[]);
    assert!(close(forecast(&craft(thread, &st, &sk, vec![&fiber], vec![], None)).sigma, SIGMA_BASE));
    sk.applied.insert("sericulture".into(), 1.0);
    assert!(close(forecast(&craft(thread, &st, &sk, vec![&fiber], vec![], None)).sigma, SIGMA_BASE + SIGMA_PER_DESTABILIZER));
}

#[test]
fn matching_transferable_skills_drop_the_weakest_secondary_skills() {
    let d = Data::load();
    let thread = d.good("thread"); // secondaries: textiles, weaving; transferable: handcrafts, music, athletics
    let st = stats(&d, "goblin");
    let fiber = item(&d, "cotton_slivers", 1.0, &[]);
    let mut sk = skilled(thread, 0.5, &["handcrafts"]);
    sk.applied.insert("textiles".into(), 0.6);
    sk.applied.insert("weaving".into(), 0.2);
    assert!(close(forecast(&craft(thread, &st, &sk, vec![&fiber], vec![], None)).secondary, 0.6));
    sk.transferable.push("music".into());
    assert!(close(forecast(&craft(thread, &st, &sk, vec![&fiber], vec![], None)).secondary, 1.0), "dropping every secondary counts as perfect");
}

#[test]
fn a_workshop_counts_as_a_weight_8_tool() {
    let d = Data::load();
    let thread = d.good("thread"); // spool weight 3, spinner weight 5
    let st = stats(&d, "goblin");
    let sk = skilled(thread, 0.5, &[]);
    let fiber = item(&d, "cotton_slivers", 1.0, &[]);
    let (spool, spinner) = (item(&d, "spool", 0.5, &[]), item(&d, "spinning_wheel", 0.5, &[]));
    let town = Workshop { quality: 0.8, shares: Vec::new() };
    let f = forecast(&craft(thread, &st, &sk, vec![&fiber], vec![&spool, &spinner], Some(&town)));
    assert!(close(f.tool_quality, (8.0 * 0.5 + 8.0 * 0.8) / 16.0));
}

#[test]
fn varieties_carry_through_the_chain() {
    let d = Data::load();
    let bolls = item(&d, "cotton_bolls", 1.0, &[]);
    let slivers = item(&d, "cotton_slivers", 1.0, &carried_tags(d.good("cotton_slivers"), &[&bolls], &d).iter().map(String::as_str).collect::<Vec<_>>());
    let thread_tags = carried_tags(d.good("thread"), &[&slivers], &d);
    assert_eq!(thread_tags, vec!["cotton".to_string()]);
    let thread = item(&d, "thread", 1.0, &thread_tags.iter().map(String::as_str).collect::<Vec<_>>());
    let fabric_tags = carried_tags(d.good("fabric"), &[&thread], &d);
    let fabric = item(&d, "fabric", 1.0, &fabric_tags.iter().map(String::as_str).collect::<Vec<_>>());
    let dye = item(&d, "crimson_dye", 1.0, &[]);
    let dyed = carried_tags(d.good("dyed_fabric"), &[&fabric, &dye], &d);
    assert!(dyed.contains(&"cotton".to_string()) && dyed.contains(&"crimson".to_string()), "{dyed:?}");

    let oak = item(&d, "oak_log", 1.0, &[]);
    let loom: Vec<&ItemData> = vec![&oak, &oak];
    assert_eq!(carried_tags(d.good("waist_loom"), &loom, &d), vec!["oak".to_string()]);
}

#[test]
fn value_and_weight_follow_the_raw_materials() {
    let d = Data::load();
    let loom = d.good("waist_loom");
    let bamboo = item(&d, "arrow_bamboo", 1.0, &[]);
    let ramie = item(&d, "ramie_stalks", 1.0, &[]);
    let make = |log: &str| {
        let l = item(&d, log, 1.0, &[]);
        let (raw, depth) = lineage(&[&l, &l, &bamboo, &bamboo, &bamboo, &ramie]);
        let it = ItemData { good: loom.id.clone(), quality: 1.0, carried_tags: Vec::new(), raw: raw.clone(), depth, value: 0.0, shares: Vec::new(), wear: 0 };
        (value(loom, &raw, depth, 1.0, None), weight(loom, &it))
    };
    let (oak_value, oak_weight) = make("oak_log");
    let (hemlock_value, hemlock_weight) = make("hemlock_log");
    assert!(oak_value > hemlock_value && oak_weight > hemlock_weight);

    let bolls = d.good("cotton_bolls");
    assert!(close(value(bolls, &[], 0, 0.5, None), bolls.raw_material_base_value.unwrap()));
}

#[test]
fn depth_counts_refinement_steps() {
    let d = Data::load();
    let bolls = item(&d, "cotton_bolls", 1.0, &[]);
    let (_, slivers_depth) = lineage(&[&bolls]);
    assert_eq!(slivers_depth, 1);
    let slivers = ItemData { depth: slivers_depth, ..item(&d, "cotton_slivers", 1.0, &[]) };
    assert_eq!(lineage(&[&slivers]).1, 2);
}

#[test]
fn shares_flow_from_inputs_tools_workshop_and_crafter() {
    let d = Data::load();
    let thread = d.good("thread");
    let st = stats(&d, "goblin");
    let sk = Skills::default();
    let mut fiber = item(&d, "cotton_slivers", 1.0, &[]);
    fiber.shares = vec![Share { monster: 5, count: 2.0 }];
    let mut spool = item(&d, "spool", 1.0, &[]);
    spool.shares = vec![Share { monster: 7, count: 1.0 }, Share { monster: 8, count: 1.0 }];
    let spinner = item(&d, "spinning_wheel", 1.0, &[]);
    let shop = Workshop { quality: 0.8, shares: vec![Share { monster: 9, count: 3.0 }] };
    let shares = output_shares(&craft(thread, &st, &sk, vec![&fiber], vec![&spool, &spinner], Some(&shop)));
    let of = |m: u32| shares.iter().find(|s| s.monster == m).map_or(0.0, |s| s.count);
    assert!(close(of(5), 2.0), "inputs pass on their shares");
    assert!(close(of(7), 1.5) && close(of(8), 1.5), "the spool's weight 3 splits between its makers");
    assert!(close(of(9), 8.0), "the workshop passes on 8");
    assert!(close(of(1), thread.value_added_shares as f64), "the crafter earns the recipe's shares");
}

#[test]
fn production_is_deterministic_per_seed_and_fixed_quantities_hold() {
    let d = Data::load();
    let spool = d.good("spool");
    let st = stats(&d, "goblin");
    let sk = skilled(spool, 0.6, &[]);
    let bamboo = item(&d, "arrow_bamboo", 1.0, &[]);
    let c = craft(spool, &st, &sk, vec![&bamboo], vec![], None);
    let a = produce(&c, &d, &mut Rng::new(42));
    let b = produce(&c, &d, &mut Rng::new(42));
    assert_eq!(a.len(), spool.quantity as usize);
    assert!(a.iter().zip(&b).all(|(x, y)| x.quality == y.quality));
    assert_eq!(a[0].carried_tags, vec!["arrow bamboo".to_string()]);
}

#[test]
fn gathering_starts_an_item_history() {
    let d = Data::load();
    let st = stats(&d, "cyclops");
    let sk = Skills::default();
    let got = gather(d.good("oak_log"), Crafter { id: 3, stats: &st, skills: &sk }, &d, &mut Rng::new(7));
    assert!(!got.is_empty());
    assert_eq!(got[0].depth, 0);
    assert_eq!(got[0].raw[0].good, "oak_log");
    assert!(got[0].shares.iter().any(|s| s.monster == 3));
}

#[test]
fn odds_of_exceptional_results_follow_the_spread() {
    let f = |mean: f64, sigma: f64| Forecast { input_quality: 1.0, primary: 1.0, secondary: 1.0, tool_quality: 1.0, specific: 1.0, ability: 1.0, mean, sigma };
    let p = chance_at_least(&f(0.97, 0.4), EXCEPTIONAL);
    assert!((p - 0.242).abs() < 0.01, "1 in 4ish, got {p}");
    assert!(chance_at_least(&f(0.97, 0.1), MASTERWORK) < 1e-6);
    assert!((chance_at_least(&f(1.25, 0.2), EXCEPTIONAL) - 0.5).abs() < 1e-6);
    assert_eq!(quality_level(0.1), -2);
    assert_eq!(quality_level(1.6), 2);
    assert_eq!(tier_name(1.3), "EXCEPTIONAL");
}

#[test]
fn ungraded_goods_pass_their_inputs_quality_through_and_ignore_tools() {
    let d = Data::load();
    let slivers = d.good("cotton_slivers");
    assert!(!slivers.has_quality);
    let st = stats(&d, "goblin");
    let sk = skilled(slivers, 0.5, &[]);
    let bolls = item(&d, "cotton_bolls", 1.0, &[]);
    let (gin, bow) = (item(&d, "cotton_gin", 0.2, &[]), item(&d, "cotton_bow", 0.2, &[]));
    let c = craft(slivers, &st, &sk, vec![&bolls], vec![&gin, &bow], None);
    assert_eq!(roll_quality(&c, &mut Rng::new(1)), 1.0, "poor tools don't drag an ungraded good");
}
