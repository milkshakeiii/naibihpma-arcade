//! Game data loaded from `data/*.json`, embedded in the executable. Goods, skills and monster
//! types are pure data so the tech tree can be edited without touching code.

use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, HashMap};

const GOOD_TYPES: &str = include_str!("../data/good_types.json");
const SKILLS: &str = include_str!("../data/skills.json");
const MONSTER_TYPES: &str = include_str!("../data/monster_types.json");
const EQUIPMENT: &str = include_str!("../data/equipment.json");

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
    /// For a trade, what the good it gives is made of (the trader's pick is iron), so it weighs
    /// as that.
    #[serde(default)]
    pub made_of: Option<String>,
}

/// A good's own variety: its name (one of its type tags), the categories it's a variety of,
/// and the palette ramp it shows in, if any ("plank" for hemlock's paler wood).
#[derive(Deserialize, Clone, Debug)]
pub struct VarietyOf {
    pub name: String,
    pub of: Vec<String>,
    #[serde(default)]
    pub color: Option<String>,
    /// For a tool material, its durability against the usual: 2 for steel, 0.5 for elder.
    #[serde(default)]
    pub durability: Option<f64>,
    /// For a tool material, how much finer work its tools do: +0.1 quality on what's made with
    /// silver or silk (Henry, 2026-11-05: dear materials shouldn't simply be worst).
    #[serde(default)]
    pub fine: Option<f64>,
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

/// The abilities' short names, in index order.
pub const ABILITIES: [&str; 6] = ["STR", "DEX", "CON", "INT", "WIS", "CHA"];

/// An ability's index from its name in the data ("str", "dex", ...).
pub fn ability_index(name: &str) -> Option<usize> {
    ABILITIES.iter().position(|a| a.eq_ignore_ascii_case(name))
}

impl Stats {
    pub fn set(&mut self, index: usize, v: i32) {
        match index {
            STR => self.str = v,
            DEX => self.dex = v,
            CON => self.con = v,
            INT => self.int = v,
            WIS => self.wis = v,
            _ => self.cha = v,
        }
    }

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

/// A monster type's unique ability, the small version (DESIGN.md): its name, a line about it,
/// and what it does, as a piece's effects are said.
#[derive(Deserialize, Clone, Debug)]
pub struct TypeAbility {
    pub name: String,
    pub about: String,
    #[serde(default)]
    pub does: Does,
}

impl TypeAbility {
    /// Its name and what it does, for cards: "SCROUNGER: 1 GATHER IN 5 GIVES 1 MORE".
    pub fn line(&self) -> String {
        format!("{}: {}", self.name.to_uppercase(), self.does.text().join(", "))
    }
}

#[derive(Deserialize, Clone, Debug)]
pub struct MonsterType {
    pub name: String,
    pub cost: u32,
    pub stats: Stats,
    /// How many of each kind of slot it has for equipment.
    pub slots: Slots,
    /// Its unique ability, if it has one (a troll hasn't yet).
    #[serde(default)]
    pub ability: Option<TypeAbility>,
}

/// A monster type's slots: its hands (each holds a tool or a carried piece, as the original's
/// personal items were held), and what it can wear and have inside.
#[derive(Deserialize, Clone, Copy, Debug)]
pub struct Slots {
    pub hands: u32,
    pub worn: u32,
    pub inside: u32,
}

impl Slots {
    pub fn of(&self, slot: Slot) -> u32 {
        match slot {
            Slot::Hand => self.hands,
            Slot::Worn => self.worn,
            Slot::Inside => self.inside,
        }
    }
}

/// Where a piece of equipment goes on a monster: held in a hand, worn, or inside.
#[derive(Deserialize, Serialize, Clone, Copy, Debug, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum Slot {
    Hand,
    Worn,
    Inside,
}

impl Slot {
    pub fn label(self) -> &'static str {
        match self {
            Slot::Hand => "HANDS",
            Slot::Worn => "WORN",
            Slot::Inside => "INSIDE",
        }
    }
}

/// A piece of equipment, given for renown (`data/equipment.json`).
#[derive(Deserialize, Clone, Debug)]
pub struct Piece {
    pub id: String,
    pub name: String,
    pub slot: Slot,
    /// How it's drawn, on a monster (gloves on the hands, a belt at the waist) and on its rack.
    /// Pieces don't compete by what they are (Henry): any fits a free slot of its own.
    pub look: String,
    /// The renown it's given at.
    pub renown: u32,
    /// What it adds to abilities ("dex": 1).
    #[serde(default)]
    pub bonus: BTreeMap<String, i32>,
    /// The color it's drawn in: a ramp's name and a shade of it.
    #[serde(default)]
    pub color: Option<(String, u8)>,
    /// A line about it for its card.
    #[serde(default)]
    pub about: String,
    /// What it does besides adding to abilities: to crafts, learning, tools.
    #[serde(default)]
    pub does: Does,
    /// What a monster does to have it, if not what its slot says ("KEEPS" a notebook).
    #[serde(default)]
    pub verb: Option<String>,
}

/// What a piece does to the crafts a monster does (DESIGN.md: Equipment given for renown, the
/// other pieces), each a change in percent unless said: a magnifying glass's quality +10, a
/// wristwatch's time -10. Changes from several pieces multiply.
#[derive(Deserialize, Clone, Debug, Default)]
#[serde(default)]
pub struct Does {
    /// More of what a craft makes, in units, where how many varies (a towel's 1).
    #[serde(rename = "yield")]
    pub more: u32,
    pub quality: i32,
    /// How many a craft rolls, where nothing it makes has a quality roll: the roll goes this
    /// many percent further before it's rounded (a craft's secret's 10 where its goods have no
    /// quality).
    pub quantity: i32,
    pub time: i32,
    pub learning: i32,
    pub forgetting: i32,
    pub value: i32,
    /// The wear on the tools the monster crafts with, in hand or on a peg.
    pub wear: i32,
    /// Hauling, in percent: the room in a wagon the monster leads (a backpack), the weight it
    /// pulls one at full speed (a feathered cap), and the weight it pushes a good at full
    /// speed (a shoulder pole).
    pub room: i32,
    pub pull: i32,
    pub push: i32,
    /// Hands more than its type has, for a tool or anything else held (a tool belt's one).
    pub hands: u32,
    /// Everyone working near its singer, of any commune, the singer too, works this many percent
    /// faster (the work song's 5); it sings whenever anyone near it works.
    pub song: i32,
    /// The chance in a hundred that a gather turns up one good more (a goblin's 20).
    pub scrounge: i32,
    /// The ability it leads a wagon by, if not CHA ("str", an orc's).
    pub lead_by: Option<String>,
    /// How much further a result above the forecast goes (a lucky charm's 50: half as far again).
    pub luck: i32,
    /// How much of the way back to the forecast a result below it comes (patience: halfway).
    pub patience: i32,
    /// A talent it counts as, if the monster doesn't have it already (a teaching).
    pub talent: Option<String>,
    /// Only at crafts whose main skill is this one (a craft's secret).
    pub at: Option<String>,
}

impl Does {
    /// What it does at a craft making `recipe`: lifting quality only where the result has a
    /// quality roll, how many only where that varies, and a craft's quantity only where nothing
    /// has a quality roll (so a secret lifts one roll or the other, never both).
    pub fn at_craft(&self, recipe: &GoodType) -> Does {
        let mut d = self.clone();
        if recipe.has_quality {
            d.quantity = 0;
        } else {
            (d.quality, d.luck, d.patience) = (0, 0, 0);
        }
        if recipe.is_fixed_quantity {
            (d.more, d.quantity) = (0, 0);
        }
        // Hauling isn't crafting, and a third hand doesn't help a craft (its tool does).
        (d.room, d.pull, d.push, d.hands, d.lead_by) = (0, 0, 0, 0, None);
        // Scrounging is for gathering, which takes nothing in.
        if !recipe.input_goods_tags_required.is_empty() {
            d.scrounge = 0;
        }
        d
    }

    /// What it does, each as short as the character sheet's line: "+10% QUALITY".
    pub fn text(&self) -> Vec<String> {
        let mut out = Vec::new();
        if self.more > 0 {
            out.push(format!("+{} YIELD", self.more));
        }
        match self.hands {
            0 => {}
            1 => out.push("+1 HAND".into()),
            n => out.push(format!("+{n} HANDS")),
        }
        // A secret's own name says which craft (Henry: "+10% QUALITY OR YIELD").
        match (self.quality, self.quantity) {
            (0, 0) => {}
            (q, 0) => out.push(format!("{q:+}% QUALITY")),
            (0, n) => out.push(format!("{n:+}% YIELD")),
            (q, n) if q == n => out.push(format!("{q:+}% QUALITY OR YIELD")),
            (q, n) => out.extend([format!("{q:+}% QUALITY"), format!("{n:+}% YIELD")]),
        }
        for (v, what) in [(self.time, "TIME"), (self.learning, "LEARNING"), (self.forgetting, "FORGETTING"), (self.value, "VALUE"), (self.room, "WAGON ROOM"), (self.pull, "PULLING POWER"), (self.push, "HAULING POWER")] {
            if v != 0 {
                out.push(format!("{v:+}% {what}"));
            }
        }
        match self.wear {
            0 => {}
            -50 => out.push("HALF THE TOOL WEAR".into()),
            100 => out.push("DOUBLE TOOL WEAR".into()),
            w => out.push(format!("{w:+}% TOOL WEAR")),
        }
        if self.luck != 0 {
            out.push(format!("GOOD ROLLS {}% BETTER", self.luck));
        }
        match self.patience {
            0 => {}
            50 => out.push("BAD ROLLS HALF AS BAD".into()),
            p => out.push(format!("BAD ROLLS {p}% LESS BAD")),
        }
        if self.song != 0 {
            out.push(format!("{:+}% TIME FOR ALL NEARBY", -self.song));
        }
        if self.scrounge > 0 {
            out.push(format!("1 GATHER IN {} GIVES 1 MORE", 100 / self.scrounge));
        }
        if let Some(a) = &self.lead_by {
            out.push(format!("LEADS WAGONS BY {}", a.to_uppercase()));
        }
        if let Some(t) = &self.talent {
            out.push(format!("TALENT: {}", t.replace('_', " ").to_uppercase()));
        }
        out
    }
}

impl Piece {
    /// Everything it adds, as a card or the character sheet says it: "+1 DEX", "+10% QUALITY".
    pub fn effect_text(&self) -> String {
        let mut all: Vec<String> = (0..6).filter(|&a| self.bonus_to(a) != 0).map(|a| format!("{:+} {}", self.bonus_to(a), ABILITIES[a])).collect();
        all.extend(self.does.text());
        all.join(", ")
    }

    /// How it's drawn: its look, or else its kind.
    pub fn look(&self) -> &str {
        &self.look
    }

    /// What a monster does to have it: takes it in hand, puts it on, learns it, or its own verb.
    pub fn verb(&self) -> &str {
        self.verb.as_deref().unwrap_or(match self.slot {
            Slot::Hand => "TAKES",
            Slot::Worn => "PUTS ON",
            Slot::Inside => "LEARNS",
        })
    }

    /// The same, said of it: "PUTS IT ON", "TAKES IT".
    pub fn verb_it(&self) -> String {
        match self.verb().split_once(' ') {
            Some((verb, on)) => format!("{verb} IT {on}"),
            None => format!("{} IT", self.verb()),
        }
    }

    /// What a monster does to be rid of it, from its sheet.
    pub fn off_verb(&self) -> &str {
        match (self.slot, self.verb.is_some()) {
            (Slot::Hand, _) | (Slot::Inside, true) => "GIVES UP",
            (Slot::Worn, _) => "TAKES OFF",
            (Slot::Inside, false) => "FORGETS",
        }
    }

    /// What it adds to ability `index`.
    pub fn bonus_to(&self, index: usize) -> i32 {
        self.bonus.iter().filter(|(k, _)| ability_index(k) == Some(index)).map(|(_, v)| *v).sum()
    }
}

#[derive(Deserialize)]
struct EquipmentFile {
    pieces: Vec<Piece>,
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
    /// A tool of it: its durability against the usual (none: the usual).
    pub durability: Option<f64>,
    /// A tool of it: how much finer what's made with it comes out (none: no finer).
    pub fine: Option<f64>,
}

impl Variety {
    pub fn is(&self, category: &str) -> bool {
        self.of.iter().any(|c| c == category)
    }

    /// The material category it's a tool material of, if any ("metal" for steel).
    pub fn material(&self) -> Option<&'static str> {
        MATERIALS.iter().copied().find(|c| self.is(c))
    }
}

/// What a tool can be made of, in the order that decides what it's made of: its metal if it
/// carries one (an iron head on an oak handle is an iron tool), then its stone, its wood or
/// bamboo, its clay, and last its cloth (a sack; a stone hammer's rope binding doesn't count).
pub const MATERIALS: [&str; 6] = ["metal", "stone", "wood", "bamboo", "clay", "textiles"];

pub struct Data {
    pub goods: Vec<GoodType>,
    by_id: HashMap<String, usize>,
    pub monster_types: BTreeMap<String, MonsterType>,
    pub skills: SkillDefs,
    pub varieties: Vec<Variety>,
    pub alloys: Vec<Alloy>,
    /// Equipment, given for renown.
    pub pieces: Vec<Piece>,
}

impl Data {
    pub fn load() -> Self {
        let file = serde_json::from_str::<GoodsFile>(GOOD_TYPES).expect("data/good_types.json");
        let (mut goods, alloys) = (file.good_types, file.alloys);
        // A trader's trades are recipes too, each named for the good it gives.
        for t in crate::workshops::trades() {
            let gives = goods.iter().find(|g| g.id == t.makes).unwrap_or_else(|| panic!("a trade makes {}, which isn't a good", t.makes));
            let trade = serde_json::json!({ "id": t.id, "name": gives.name, "type_tags": [], "has_quality": false, "primary_applied_skill": "", "input_goods_tags_required": t.wants, "makes": t.makes, "made_of": t.of });
            goods.push(serde_json::from_value(trade).expect("a trade as a recipe"));
        }
        let mut varieties = gather_varieties(&goods);
        for a in &alloys {
            let of: Vec<String> = varieties.iter().find(|v| a.of.first() == Some(&v.tag)).map_or_else(Vec::new, |first| first.of.iter().filter(|c| a.of.iter().all(|m| varieties.iter().any(|v| &v.tag == m && v.is(c)))).cloned().collect());
            match varieties.iter_mut().find(|v| v.tag == a.name) {
                Some(v) => v.of.extend(of.into_iter().filter(|c| !v.of.contains(c)).collect::<Vec<_>>()),
                None => varieties.push(Variety { tag: a.name.clone(), of, color: None, durability: None, fine: None }),
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
        let pieces = serde_json::from_str::<EquipmentFile>(EQUIPMENT).expect("data/equipment.json").pieces;
        Self { goods, by_id, monster_types, skills, varieties, alloys, pieces }
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

    /// What a tool with these tags is made of, for how long it lasts: the first of its
    /// varieties in the first material category it has any of (`MATERIALS`), metals once
    /// they've met (copper with tin is a bronze tool).
    pub fn material(&self, tags: &[String]) -> Option<&Variety> {
        let tags = self.alloyed(tags);
        MATERIALS.iter().find_map(|c| tags.iter().filter_map(|t| self.variety(t)).find(|v| v.is(c)))
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

    pub fn piece(&self, id: &str) -> &Piece {
        self.try_piece(id).unwrap_or_else(|| panic!("unknown piece of equipment {id}"))
    }

    pub fn try_piece(&self, id: &str) -> Option<&Piece> {
        self.pieces.iter().find(|p| p.id == id)
    }
}

/// Every variety the goods declare, as the original tree's `add_variety` registered them: the
/// same variety declared by several goods (rice on three kinds of rice stalks; cotton of textiles
/// on bolls, of oil on seeds) is one variety of all their categories.
fn gather_varieties(goods: &[GoodType]) -> Vec<Variety> {
    let mut out: Vec<Variety> = Vec::new();
    for d in goods.iter().filter_map(|g| g.variety.as_ref()) {
        let k = out.iter().position(|v| v.tag == d.name).unwrap_or_else(|| {
            out.push(Variety { tag: d.name.clone(), of: Vec::new(), color: None, durability: None, fine: None });
            out.len() - 1
        });
        let v = &mut out[k];
        let new: Vec<String> = d.of.iter().filter(|c| !v.of.contains(c)).cloned().collect();
        v.of.extend(new);
        v.color = v.color.take().or_else(|| d.color.clone());
        v.durability = v.durability.or(d.durability);
        v.fine = v.fine.or(d.fine);
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

    /// The metal batch (PROVINCES.md, "Metals, from ore to edge", agreed 2026-11-05): every
    /// input and tool its goods ask for is something, and its chains bottom out in ore and
    /// what else can be gathered.
    #[test]
    fn the_metal_batch_bottoms_out_in_ore_and_every_tool_it_asks_for_exists() {
        let d = Data::load();
        fn makeable(d: &Data, id: &str, depth: u32) -> bool {
            let g = d.good(id);
            depth < 10
                && g.input_goods_tags_required
                    .iter()
                    .filter(|grp| !grp.is_empty())
                    .all(|grp| d.goods.iter().any(|p| matches(&p.type_tags, grp) && p.id != id && makeable(d, &p.id, depth + 1)))
        }
        let batch = [
            "copper_ore", "tin_sand", "iron_ore", "silver_ore", "granite", "pyrite", "copper_ingot", "tin_ingot", "silver_ingot", "pig_iron_ingot", "wrought_iron_ingot", "steel_ingot",
            "bronze_ingot", "solder", "primitive_tongs", "double_tongs", "forceps", "basket", "shovel", "sickle", "scythe", "billhook", "strong_pan",
        ];
        for id in batch.into_iter().chain(["knife", "metal_chisel", "file", "pick", "needle", "wire", "nail", "metal_anvil", "green_vitriol", "stone_anvil"]) {
            let g = d.good(id);
            for group in g.input_goods_tags_required.iter().chain(&g.tools_required_tags).filter(|g| !g.is_empty()) {
                assert!(d.goods.iter().any(|p| matches(&p.type_tags, group)), "{id} needs {group:?}, which nothing provides");
            }
            assert!(makeable(&d, id, 0), "{id} can't be made from gatherable goods");
        }
        // Edges of hard metal: pig iron, steel or bronze; a knife takes any metal.
        let fits = |group: &[&str]| -> Vec<&str> { d.goods.iter().filter(|p| group.iter().all(|t| p.type_tags.iter().any(|x| x == t))).map(|p| p.id.as_str()).collect() };
        assert_eq!(fits(&["hard", "metal", "ingot"]), ["pig_iron_ingot", "steel_ingot", "bronze_ingot"]);
        assert_eq!(d.good("pick").input_goods_tags_required[0], ["hard", "metal", "ingot"]);
        assert_eq!(d.good("knife").input_goods_tags_required[0], ["metal", "ingot"]);
        // Wire is drawn through a metal die, not the wooden pattern; the metal pot is a cauldron.
        assert_eq!(fits(&["die", "metal"]), ["die"]);
        assert_eq!(fits(&["cauldron"]), ["metal_pot"]);
        // The die is cast in a sand mold made on its pattern, which takes the pattern's name:
        // a "die" mold.
        assert!(d.good("die_pattern").variety.as_ref().is_some_and(|v| v.name == "die" && v.of == ["pattern"]));
        let mold = d.good("sand_mold");
        let k = mold.input_goods_tags_required.iter().position(|grp| grp == &["pattern"]).expect("a sand mold takes a pattern");
        assert!(mold.input_goods_tags_carryover[k].iter().any(|t| t == "die"), "and carries its name");
        assert_eq!(d.good("die").input_goods_tags_required[0], ["die", "mold"]);
    }

    /// Fixes found while placing everything (PROVINCES.md, 2026-11-05): the misspellings, the
    /// palace goods' yellow clay, and the pot mold, which isn't a pot.
    #[test]
    fn the_data_fixes_found_placing_everything_hold() {
        let d = Data::load();
        let fits = |group: &[&str]| -> Vec<&str> { d.goods.iter().filter(|p| group.iter().all(|t| p.type_tags.iter().any(|x| x == t))).map(|p| p.id.as_str()).collect() };
        let asks = |t: &str| d.goods.iter().filter(|g| g.input_goods_tags_required.iter().flatten().any(|x| x == t)).count();
        assert!(fits(&d.good("seeth_water").input_goods_tags_required[0].iter().map(String::as_str).collect::<Vec<_>>()).contains(&"indigofera_leaves"));
        assert_eq!((asks("soy beans"), asks("soybeans")), (0, 2), "one spelling");
        for id in ["clay_palace_bricks", "clay_palace_tiles"] {
            assert_eq!(fits(&d.good(id).input_goods_tags_required[0].iter().map(String::as_str).collect::<Vec<_>>()), ["yellow_clay"]);
        }
        assert_eq!(fits(&["pot"]), ["metal_pot"], "malt, the glazes and porcelain paste want a real pot");
        assert_eq!(fits(&["pot mold"]), ["ceramic_pot_mold"]);
        assert_eq!(d.good("metal_pot").tools_required_tags, [["pot mold"]]);
    }

    /// The varieties batch (Henry, 2026-11-05: add them all; but 14 dye colors is right): every
    /// kind comes in several, each with its tool durability, and the pine's goods feed the
    /// palace goods and royal blue cloth.
    #[test]
    fn every_kind_comes_in_several() {
        let d = Data::load();
        let of = |c: &str| d.varieties.iter().filter(|v| v.is(c)).count();
        for (c, at_least) in [("wood", 15), ("stone", 6), ("clay", 7), ("bamboo", 2), ("indigo plant", 4), ("oil", 5), ("metal", 7)] {
            assert!(of(c) >= at_least, "{c}: {}", of(c));
        }
        assert_eq!(of("dye"), 14, "fourteen colors is right");
        let durability = |t: &str| d.variety(t).and_then(|v| v.durability);
        for (t, x) in [("paulownia", 0.5), ("jujube", 1.25), ("sandalwood", 1.25), ("willow", 0.75), ("palm", 0.5), ("slate", 0.75), ("sandstone", 0.75), ("basalt", 1.25), ("purple clay", 0.5)] {
            assert_eq!(durability(t), Some(x), "{t}");
        }
        for t in ["elm", "scholartree", "camphor", "juniper", "mulberry", "marble", "moso bamboo"] {
            assert_eq!(durability(t), None, "{t} is the usual");
        }
        let fits = |t: &str| -> Vec<&str> { d.goods.iter().filter(|p| p.type_tags.iter().any(|x| x == t)).map(|p| p.id.as_str()).collect() };
        assert_eq!((fits("resin"), fits("rosin"), fits("pitch")), (vec!["pine_resin"], vec!["rosin"], vec!["pitch"]));
        assert!(d.good("oil").input_goods_tags_carryover[0].iter().any(|t| t == "tung"), "oil carries its seed");
    }

    /// A tag list written as a string splits into letters on loading ("trellis" became t, r,
    /// e, l, i, s, which no tool is): no tag is a single letter.
    #[test]
    fn no_tag_is_a_single_letter() {
        let d = Data::load();
        for g in &d.goods {
            let lists = [&g.input_goods_tags_required, &g.input_goods_tags_carryover, &g.tools_required_tags];
            for t in g.type_tags.iter().chain(lists.into_iter().flatten().flatten()) {
                assert!(t.chars().count() > 1, "{} has the one-letter tag {t:?}", g.id);
            }
        }
    }

    /// Tool life by material (agreed 2026-11-05), each variety's number declared once.
    #[test]
    fn tool_materials_declare_their_durability_once() {
        let d = Data::load();
        for v in &d.varieties {
            let declared: Vec<f64> = d.goods.iter().filter_map(|g| g.variety.as_ref()).filter(|w| w.name == v.tag).filter_map(|w| w.durability).collect();
            assert!(declared.windows(2).all(|w| w[0] == w[1]), "{} is declared with different durabilities: {declared:?}", v.tag);
            assert!(v.durability.map_or(true, |u| u > 0.0 && v.material().is_some()), "{}'s durability needs a tool material", v.tag);
        }
        let agreed = [("steel", 2.0), ("bronze", 1.5), ("wrought iron", 1.25), ("copper", 0.75), ("cast iron", 0.75), ("tin", 0.5), ("silver", 0.5), ("hemlock", 0.75), ("pine", 0.75), ("elder", 0.5), ("arrow bamboo", 0.75), ("red clay", 0.5)];
        for (tag, factor) in agreed {
            assert_eq!(d.variety(tag).and_then(|v| v.durability), Some(factor), "{tag}");
        }
        for tag in ["oak", "rhododendron", "granite"] {
            assert_eq!(d.variety(tag).and_then(|v| v.durability), None, "{tag} is the usual");
        }
        // Cloth, for sacks (agreed 2026-11-05): hemp the toughest.
        for (tag, x) in [("hemp", 1.5), ("ramie", 1.25), ("climbing hempvine", 1.25), ("velvetleaf", 0.75), ("silk", 0.5)] {
            assert_eq!(d.variety(tag).and_then(|v| v.durability), Some(x), "{tag}");
        }
        // Finer work: silver and silk only, and only on tool materials.
        let fine: Vec<(&str, f64)> = d.varieties.iter().filter_map(|v| Some((v.tag.as_str(), v.fine?))).collect();
        assert_eq!(fine, [("silk", 0.1), ("silver", 0.1)].iter().copied().filter(|(t, _)| fine.iter().any(|f| f.0 == *t)).collect::<Vec<_>>());
        assert_eq!(fine.len(), 2, "{fine:?}");
        assert!(d.varieties.iter().filter(|v| v.fine.is_some()).all(|v| v.material().is_some()));
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
