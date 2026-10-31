//! Game data loaded from `data/*.json`, embedded in the executable. Goods, skills and monster
//! types are pure data so the tech tree can be edited without touching code.

use serde::Deserialize;
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

#[derive(Deserialize, Clone, Debug)]
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
}

#[derive(Deserialize)]
struct MonstersFile {
    monster_types: BTreeMap<String, MonsterType>,
}

pub struct Data {
    pub goods: Vec<GoodType>,
    by_id: HashMap<String, usize>,
    pub monster_types: BTreeMap<String, MonsterType>,
    pub skills: SkillDefs,
}

impl Data {
    pub fn load() -> Self {
        let goods = serde_json::from_str::<GoodsFile>(GOOD_TYPES).expect("data/good_types.json").good_types;
        let by_id = goods.iter().enumerate().map(|(i, g)| (g.id.clone(), i)).collect();
        let monster_types = serde_json::from_str::<MonstersFile>(MONSTER_TYPES).expect("data/monster_types.json").monster_types;
        let skills = serde_json::from_str::<SkillDefs>(SKILLS).expect("data/skills.json");
        Self { goods, by_id, monster_types, skills }
    }

    pub fn good(&self, id: &str) -> &GoodType {
        let i = *self.by_id.get(id).unwrap_or_else(|| panic!("unknown good id {id}"));
        &self.goods[i]
    }

    pub fn try_good(&self, id: &str) -> Option<&GoodType> {
        self.by_id.get(id).map(|&i| &self.goods[i])
    }
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
}
