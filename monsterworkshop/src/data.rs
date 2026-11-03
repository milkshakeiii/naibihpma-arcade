//! Game data loaded from `data/*.json`, embedded in the executable. Goods, skills and monster
//! types are pure data so the tech tree can be edited without touching code.

use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, HashMap};

const GOOD_TYPES: &str = include_str!("../data/good_types.json");
const SKILLS: &str = include_str!("../data/skills.json");
const MONSTER_TYPES: &str = include_str!("../data/monster_types.json");

/// Ability score indices, as used by `relevant_ability_score` in the data.
pub const STR: usize = 0;
pub const DEX: usize = 1;
pub const CON: usize = 2;
pub const INT: usize = 3;
pub const WIS: usize = 4;
pub const CHA: usize = 5;

#[derive(Deserialize, Clone, Debug)]
pub struct GoodType {
    pub id: String,
    pub name: String,
    pub type_tags: Vec<String>,
    #[serde(default = "yes")]
    pub has_quality: bool,
    #[serde(default)]
    pub is_fixed_quantity: bool,
    #[serde(default)]
    pub requires_workshop: bool,
    pub primary_applied_skill: String,
    #[serde(default)]
    pub secondary_applied_skills: Vec<String>,
    #[serde(default)]
    pub destabilizer_skills: Vec<String>,
    #[serde(default)]
    pub transferable_skills: Vec<String>,
    #[serde(default)]
    pub relevant_ability_score: usize,
    #[serde(default = "one")]
    pub difficulty_rating: u32,
    #[serde(default = "sixty")]
    pub production_time: u32,
    #[serde(default = "one")]
    pub quantity: u32,
    #[serde(default)]
    pub value_added_shares: u32,
    #[serde(default)]
    pub input_goods_tags_required: Vec<Vec<String>>,
    #[serde(default)]
    pub input_goods_tags_carryover: Vec<Vec<String>>,
    #[serde(default)]
    pub tools_required_tags: Vec<Vec<String>>,
    #[serde(default)]
    pub tools_weights: Vec<u32>,
    #[serde(default)]
    pub raw_material_base_value: Option<f64>,
    #[serde(default)]
    pub raw_material_density: Option<f64>,
    #[serde(default)]
    pub raw_material_rarity: Option<u32>,
    #[serde(default = "one_f")]
    pub storage_volume: f64,
    /// The variety this good is, if it is one (the original tree's `add_variety`): oak logs are
    /// the oak variety of wood, so whatever carries wood from them carries "oak".
    #[serde(default)]
    pub variety: Option<VarietyOf>,
    /// For a trade (see `workshops.rs`), the good it gives: a trade is a recipe whose product is
    /// another good.
    #[serde(default)]
    pub makes: Option<String>,
}

/// A good's own variety: its name (one of its type tags), the categories it's a variety of,
/// and the palette ramp it shows in, if any ("plank" for hemlock's paler wood).
#[derive(Deserialize, Clone, Debug)]
pub struct VarietyOf {
    pub name: String,
    pub of: Vec<String>,
    #[serde(default)]
    pub color: Option<String>,
}

fn yes() -> bool {
    true
}
fn one() -> u32 {
    1
}
fn one_f() -> f64 {
    1.0
}
fn sixty() -> u32 {
    60
}

impl GoodType {
    pub fn is_raw(&self) -> bool {
        self.raw_material_base_value.is_some()
    }

    /// The good this recipe makes: itself, or for a trade the good it gives.
    pub fn output(&self) -> &str {
        self.makes.as_deref().unwrap_or(&self.id)
    }

    pub fn is_trade(&self) -> bool {
        self.makes.is_some()
    }

    /// Input slots that actually require something (some data lists empty groups).
    pub fn input_groups(&self) -> impl Iterator<Item = (usize, &Vec<String>)> {
        self.input_goods_tags_required.iter().enumerate().filter(|(_, g)| !g.is_empty())
    }

    /// The weight of tool slot `i`; missing or zero weights count as 1.
    pub fn tool_weight(&self, i: usize) -> u32 {
        self.tools_weights.get(i).copied().unwrap_or(1).max(1)
    }
}

/// True when every tag in `group` is among `tags` (the original matching rule).
pub fn matches(tags: &[String], group: &[String]) -> bool {
    group.iter().all(|t| tags.contains(t))
}

#[derive(Deserialize, Clone, Debug, Serialize)]
pub struct Stats {
    pub str: i32,
    pub dex: i32,
    pub con: i32,
    pub int: i32,
    pub wis: i32,
    pub cha: i32,
}

impl Stats {
    pub fn get(&self, index: usize) -> i32 {
        match index {
            STR => self.str,
            DEX => self.dex,
            CON => self.con,
            INT => self.int,
            WIS => self.wis,
            _ => self.cha,
        }
    }
}

#[derive(Deserialize, Clone, Debug)]
pub struct MonsterType {
    pub name: String,
    pub cost: u32,
    pub stats: Stats,
    pub body_cap: u32,
    pub mind_cap: u32,
}

#[derive(Deserialize, Clone, Debug, Default)]
pub struct SkillDefs {
    pub transferable_skills: Vec<String>,
    pub applied_skills: Vec<String>,
    #[serde(default)]
    pub relevant_transferable_skills: BTreeMap<String, Vec<String>>,
}

#[derive(Deserialize)]
struct GoodsFile {
    good_types: Vec<GoodType>,
    #[serde(default)]
    alloys: Vec<Alloy>,
}

/// Metals that become an alloy when they meet in a good: copper with tin is bronze. An alloy is
/// a variety of whatever its metals both are (bronze is a metal), so it's one more road to its
/// tag: bronze from copper and tin is the same bronze as from a bronze ingot.
#[derive(Deserialize, Clone, Debug)]
pub struct Alloy {
    pub name: String,
    pub of: Vec<String>,
}

#[derive(Deserialize)]
struct MonstersFile {
    monster_types: BTreeMap<String, MonsterType>,
}

/// A variety, gathered from every good that declares it: its tag, the categories it's a
/// variety of ("ramie" of textiles and ash, "oak" of wood), and the palette ramp it shows in.
pub struct Variety {
    pub tag: String,
    pub of: Vec<String>,
    pub color: Option<String>,
}

impl Variety {
    pub fn is(&self, category: &str) -> bool {
        self.of.iter().any(|c| c == category)
    }
}

pub struct Data {
    pub goods: Vec<GoodType>,
    by_id: HashMap<String, usize>,
    pub monster_types: BTreeMap<String, MonsterType>,
    pub skills: SkillDefs,
    pub varieties: Vec<Variety>,
    pub alloys: Vec<Alloy>,
}

impl Data {
    pub fn load() -> Self {
        let file = serde_json::from_str::<GoodsFile>(GOOD_TYPES).expect("data/good_types.json");
        let (mut goods, alloys) = (file.good_types, file.alloys);
        // A trader's trades are recipes too, each named for the good it gives.
        for t in crate::workshops::trades() {
            let gives = goods.iter().find(|g| g.id == t.makes).unwrap_or_else(|| panic!("a trade makes {}, which isn't a good", t.makes));
            let trade = serde_json::json!({ "id": t.id, "name": gives.name, "type_tags": [], "has_quality": false, "primary_applied_skill": "", "input_goods_tags_required": t.wants, "makes": t.makes });
            goods.push(serde_json::from_value(trade).expect("a trade as a recipe"));
        }
        let mut varieties = gather_varieties(&goods);
        for a in &alloys {
            let of: Vec<String> = varieties.iter().find(|v| a.of.first() == Some(&v.tag)).map_or_else(Vec::new, |first| first.of.iter().filter(|c| a.of.iter().all(|m| varieties.iter().any(|v| &v.tag == m && v.is(c)))).cloned().collect());
            match varieties.iter_mut().find(|v| v.tag == a.name) {
                Some(v) => v.of.extend(of.into_iter().filter(|c| !v.of.contains(c)).collect::<Vec<_>>()),
                None => varieties.push(Variety { tag: a.name.clone(), of, color: None }),
            }
        }
        for g in &mut goods {
            for carry in &mut g.input_goods_tags_carryover {
                *carry = expand(carry, &varieties);
            }
        }
        let by_id = goods.iter().enumerate().map(|(i, g)| (g.id.clone(), i)).collect();
        let monster_types = serde_json::from_str::<MonstersFile>(MONSTER_TYPES).expect("data/monster_types.json").monster_types;
        let skills = serde_json::from_str::<SkillDefs>(SKILLS).expect("data/skills.json");
        Self { goods, by_id, monster_types, skills, varieties, alloys }
    }

    /// Carried varieties once metals have met: an alloy's metals, all there, become the alloy,
    /// where the first of them was; and a metal meeting its alloy goes into it (bronze with more
    /// copper is still bronze).
    pub fn alloyed(&self, carried: &[String]) -> Vec<String> {
        let mut out = carried.to_vec();
        for a in &self.alloys {
            if let Some(at) = out.iter().position(|t| a.of.contains(t)).filter(|_| a.of.iter().all(|m| out.contains(m))) {
                out.retain(|t| !a.of.contains(t));
                out.insert(at.min(out.len()), a.name.clone());
            }
            if out.contains(&a.name) {
                out.retain(|t| !a.of.contains(t));
            }
        }
        out
    }

    /// The metal a good is made of ("bronze"), if it carries any.
    pub fn metal_of(&self, carried: &[String]) -> Option<String> {
        let metals: Vec<String> = self.alloyed(carried).into_iter().filter(|t| self.variety(t).is_some_and(|v| v.is("metal"))).collect();
        (!metals.is_empty()).then(|| metals.join(" "))
    }

    pub fn variety(&self, tag: &str) -> Option<&Variety> {
        self.varieties.iter().find(|v| v.tag == tag)
    }

    /// The first of `tags` that's a variety of `category`: the wood of an oak kit.
    pub fn variety_in<'a>(&self, tags: &'a [String], category: &str) -> Option<&'a str> {
        tags.iter().find(|t| self.variety(t).is_some_and(|v| v.is(category))).map(|t| t.as_str())
    }

    /// Can this good be held in a hand: an implement some gathered good needs (a pick, a
    /// shovel, a sickle), worked out from the data. Big workshop tools stay on pegs.
    pub fn holdable(&self, good: &GoodType) -> bool {
        self.goods.iter().filter(|g| g.is_raw()).flat_map(|g| &g.tools_required_tags).any(|group| !group.is_empty() && matches(&good.type_tags, group))
    }

    /// The color a good's own variety shows in: the hemlock of a hemlock log.
    pub fn own_color(&self, good: &GoodType) -> Option<&str> {
        good.variety.as_ref().and_then(|v| self.variety(&v.name)).and_then(|v| v.color.as_deref())
    }

    pub fn good(&self, id: &str) -> &GoodType {
        let i = *self.by_id.get(id).unwrap_or_else(|| panic!("unknown good id {id}"));
        &self.goods[i]
    }

    pub fn try_good(&self, id: &str) -> Option<&GoodType> {
        self.by_id.get(id).map(|&i| &self.goods[i])
    }
}

/// Every variety the goods declare, as the original tree's `add_variety` registered them: the
/// same variety declared by several goods (rice on three kinds of rice stalks; cotton of textiles
/// on bolls, of oil on seeds) is one variety of all their categories.
fn gather_varieties(goods: &[GoodType]) -> Vec<Variety> {
    let mut out: Vec<Variety> = Vec::new();
    for d in goods.iter().filter_map(|g| g.variety.as_ref()) {
        let k = out.iter().position(|v| v.tag == d.name).unwrap_or_else(|| {
            out.push(Variety { tag: d.name.clone(), of: Vec::new(), color: None });
            out.len() - 1
        });
        let v = &mut out[k];
        let new: Vec<String> = d.of.iter().filter(|c| !v.of.contains(c)).cloned().collect();
        v.of.extend(new);
        v.color = v.color.take().or_else(|| d.color.clone());
    }
    out
}

/// A recipe's carryover for one input, as the data writes it (categories, as the original
/// tree's `Carryover("textiles")`, plus single tags as its `.plus("oily")`, written "+oily"),
/// spelled out as the tags that flow: every variety of each category.
fn expand(carry: &[String], varieties: &[Variety]) -> Vec<String> {
    let mut out = Vec::new();
    for e in carry {
        match e.strip_prefix('+') {
            Some(tag) => out.push(tag.to_string()),
            None => out.extend(varieties.iter().filter(|v| v.is(e)).map(|v| v.tag.clone())),
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn data_loads_with_unique_ids_and_consistent_tool_weights() {
        let d = Data::load();
        assert_eq!(d.goods.len(), d.by_id.len(), "duplicate good ids");
        for g in &d.goods {
            assert!(g.tools_weights.len() <= g.tools_required_tags.len().max(g.tools_weights.len()));
            assert!(g.storage_volume > 0.0, "{} has no volume", g.id);
        }
        assert!(d.monster_types.contains_key("goblin") && d.monster_types.contains_key("cyclops"));
        assert!(d.skills.applied_skills.contains(&"spinning".to_string()));
    }

    #[test]
    fn every_fabric_chain_requirement_has_a_provider() {
        let d = Data::load();
        for id in ["cotton_bolls", "cotton_slivers", "thread", "fabric", "cotton_gin", "cotton_bow", "spinning_wheel", "spool", "waist_loom"] {
            let g = d.good(id);
            for group in g.input_goods_tags_required.iter().chain(&g.tools_required_tags).filter(|g| !g.is_empty()) {
                assert!(d.goods.iter().any(|p| matches(&p.type_tags, group)), "{id} needs {group:?}, which nothing provides");
            }
        }
    }

    /// Dyed fabric's inputs resolve all the way down to goods you can gather. Tools that
    /// can't be made yet (the saw and chisel) are lent by town benches, so only inputs count.
    #[test]
    fn the_indigo_chain_bottoms_out_in_gatherable_goods() {
        let d = Data::load();
        fn makeable(d: &Data, id: &str, depth: u32) -> bool {
            let g = d.good(id);
            depth < 8
                && g.input_goods_tags_required
                    .iter()
                    .filter(|grp| !grp.is_empty())
                    .all(|grp| d.goods.iter().any(|p| matches(&p.type_tags, grp) && p.id != id && makeable(d, &p.id, depth + 1)))
        }
        for id in ["dyed_fabric", "indigo", "lye", "quicklime", "wooden_tub"] {
            assert!(makeable(&d, id, 0), "{id} can't be made from gatherable goods");
        }
        assert!(d.goods.iter().any(|p| matches(&p.type_tags, &["tub".to_string()])), "something is a tub");
    }

    /// Varieties are declared on the goods that are them, and recipes carry whole categories:
    /// a new wood is one good declaring itself a variety of wood, and every recipe that carries
    /// wood carries it.
    #[test]
    fn metals_that_meet_become_their_alloy() {
        let d = Data::load();
        let tags = |t: &[&str]| t.iter().map(|s| s.to_string()).collect::<Vec<_>>();
        assert!(d.variety("bronze").is_some_and(|v| v.is("metal")), "bronze is a metal of its own");
        assert_eq!(d.alloyed(&tags(&["copper", "tin"])), ["bronze"]);
        assert_eq!(d.alloyed(&tags(&["cotton", "tin", "copper"])), ["cotton", "bronze"], "where the first of its metals was");
        assert_eq!(d.alloyed(&tags(&["bronze", "copper"])), ["bronze"], "more copper in bronze is still bronze");
        assert_eq!(d.alloyed(&tags(&["copper"])), ["copper"], "copper alone is copper");
        assert!(d.good("temple_bell").input_goods_tags_carryover.iter().any(|c| c.iter().any(|t| t == "bronze")), "bronze carries over into a bell by either road");
        assert_eq!(d.metal_of(&tags(&["copper", "tin"])).as_deref(), Some("bronze"));
    }

    #[test]
    fn recipes_carry_categories_of_declared_varieties() {
        let d = Data::load();
        for g in &d.goods {
            if let Some(v) = &g.variety {
                assert!(g.type_tags.contains(&v.name), "{} is the {:?} variety but isn't tagged it", g.id, v.name);
                assert!(!v.of.is_empty(), "{}'s variety is of nothing", g.id);
            }
        }
        for v in &d.varieties {
            if let Some(c) = &v.color {
                assert!(crate::palette::ramp_named(c).is_some(), "{}'s color {c:?} isn't a palette ramp", v.tag);
            }
            let colors: Vec<&str> = d.goods.iter().filter_map(|g| g.variety.as_ref()).filter(|w| w.name == v.tag).filter_map(|w| w.color.as_deref()).collect();
            assert!(colors.windows(2).all(|w| w[0] == w[1]), "{} is declared in different colors: {colors:?}", v.tag);
        }
        // As written: each name a category some good's variety is of, or one the tree already
        // uses as a type with no varieties yet (metal, sand), or a single "+tag".
        let raw: serde_json::Value = serde_json::from_str(GOOD_TYPES).unwrap();
        let used = |t: &str| d.goods.iter().any(|g| g.type_tags.iter().any(|x| x == t) || g.input_goods_tags_required.iter().flatten().any(|x| x == t));
        for g in raw["good_types"].as_array().unwrap() {
            for name in g["input_goods_tags_carryover"].as_array().unwrap().iter().flat_map(|c| c.as_array().unwrap()) {
                let name = name.as_str().unwrap();
                let category = d.varieties.iter().any(|v| v.is(name)) || used(name);
                assert!(name.starts_with('+') || category, "{} carries {name:?}, which is no category", g["id"]);
            }
        }
    }
}

#[cfg(test)]
mod quantity_rule {
    use super::*;

    /// Goods without quality grades let skill show in how many come out instead.
    #[test]
    fn every_ungraded_good_rolls_its_quantity() {
        let d = Data::load();
        let fixed: Vec<&str> = d.goods.iter().filter(|g| !g.has_quality && g.is_fixed_quantity).map(|g| g.name.as_str()).collect();
        assert!(fixed.is_empty(), "ungraded goods with a fixed quantity: {fixed:?}");
    }
}
