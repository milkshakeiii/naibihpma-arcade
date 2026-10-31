//! Crafting economy rules from the original MonsterMakers design, ported from the Python version
//! (sokobanmonstermakers) with the changes recorded in DESIGN.md, "Crafting math":
//! - the quality mean averages the material and tool terms, and a soft cap bounds it;
//! - destabilizers widen the spread more;
//! - a workshop counts as one more tool, with weight 8.
//! Refinement depth follows the Django original (a good made from raw materials is depth 1).

use crate::data::{Data, GoodType, Stats, CON, INT, STR, WIS};
use std::collections::BTreeMap;

pub const QUALITY_MEAN_CAP: f64 = 2.0;
pub const SIGMA_BASE: f64 = 0.1;
pub const SIGMA_PER_DESTABILIZER: f64 = 0.3;
pub const ABILITY_FACTOR_CAP: f64 = 1.2;
pub const WORKSHOP_WEIGHT: u32 = 8;
pub const EXCEPTIONAL: f64 = 1.25;
pub const MASTERWORK: f64 = 1.5;
pub const TOOL_DURABILITY: u32 = 100;
pub const WORKSHOP_DURABILITY: u32 = 1000;
const LEARN_SPECIFIC: f64 = 0.002;
const LEARN_PRIMARY: f64 = 0.001;
const LEARN_SECONDARY: f64 = 0.0005;
const FORGET: f64 = 0.0001;
const LEARN_STEP_SECONDS: f64 = 10.0;

/// Deterministic random numbers (xorshift64*), so a saved world replays identically.
#[derive(Clone, Debug)]
pub struct Rng(u64);

impl Rng {
    pub fn new(seed: u64) -> Self {
        Self(seed.max(1))
    }

    pub fn next_u64(&mut self) -> u64 {
        let mut x = self.0;
        x ^= x >> 12;
        x ^= x << 25;
        x ^= x >> 27;
        self.0 = x;
        x.wrapping_mul(0x2545_F491_4F6C_DD1D)
    }

    /// Uniform in [0, 1).
    pub fn unit(&mut self) -> f64 {
        (self.next_u64() >> 11) as f64 / (1u64 << 53) as f64
    }

    /// Standard normal (Box-Muller).
    pub fn gauss(&mut self) -> f64 {
        let u1 = self.unit().max(1e-12);
        let u2 = self.unit();
        (-2.0 * u1.ln()).sqrt() * (std::f64::consts::TAU * u2).cos()
    }
}

/// A monster's skills: the total learned for each skill, and one forgetting counter for the
/// whole monster. A skill's value is its total minus the forgetting, never below 0.
#[derive(Clone, Debug, Default)]
pub struct Skills {
    pub transferable: Vec<String>,
    pub applied: BTreeMap<String, f64>,
    pub specific: BTreeMap<String, f64>,
    pub forgotten: f64,
}

impl Skills {
    pub fn applied(&self, skill: &str) -> f64 {
        (self.applied.get(skill).copied().unwrap_or(0.0) - self.forgotten).max(0.0)
    }

    pub fn specific(&self, good_id: &str) -> f64 {
        (self.specific.get(good_id).copied().unwrap_or(0.0) - self.forgotten).max(0.0)
    }

    pub fn matching(&self, wanted: &[String]) -> usize {
        wanted.iter().filter(|s| self.transferable.contains(s)).count()
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct RawMaterial {
    pub good: String,
    pub base_value: f64,
    pub density: f64,
}

/// Credit toward a good: renown from scoring it is split by share count.
#[derive(Clone, Debug, PartialEq)]
pub struct Share {
    pub monster: u32,
    pub count: f64,
}

/// The economic side of one physical item.
#[derive(Clone, Debug)]
pub struct ItemData {
    pub good: String,
    pub quality: f64,
    pub carried_tags: Vec<String>,
    /// Every raw material in this item's history (a raw item lists itself).
    pub raw: Vec<RawMaterial>,
    /// Refinement steps: 0 for raw materials, 1 for goods made only from raw materials.
    pub depth: u32,
    pub value: f64,
    pub shares: Vec<Share>,
    /// Durability spent so far (tools and frontier workshops).
    pub wear: u32,
}

impl ItemData {
    pub fn tags(&self, data: &Data) -> Vec<String> {
        let mut t = data.good(&self.good).type_tags.clone();
        t.extend(self.carried_tags.iter().cloned());
        t
    }

    pub fn durability_left(&self) -> u32 {
        TOOL_DURABILITY.saturating_sub(self.wear)
    }
}

/// A workshop as the crafting rules see it. Town workshops have no shares and never wear.
#[derive(Clone, Debug)]
pub struct Workshop {
    pub quality: f64,
    pub shares: Vec<Share>,
}

pub struct Crafter<'a> {
    pub id: u32,
    pub stats: &'a Stats,
    pub skills: &'a Skills,
}

/// Everything one craft depends on. `inputs` and `tools` follow the recipe's slot order.
pub struct Craft<'a> {
    pub recipe: &'a GoodType,
    pub crafter: Crafter<'a>,
    pub inputs: Vec<&'a ItemData>,
    pub tools: Vec<&'a ItemData>,
    pub workshop: Option<&'a Workshop>,
}

/// The parts of the quality roll, for the forecast card and for "what holds quality back".
#[derive(Clone, Debug)]
pub struct Forecast {
    pub input_quality: f64,
    pub primary: f64,
    pub secondary: f64,
    pub tool_quality: f64,
    pub specific: f64,
    pub ability: f64,
    pub mean: f64,
    pub sigma: f64,
}

impl Forecast {
    /// Expected final quality, including the WIS/STR nudge.
    pub fn expected(&self, stats: &Stats) -> f64 {
        nudge(self.mean, stats)
    }
}

fn ability(stats: &Stats, index: usize) -> f64 {
    stats.get(index) as f64
}

/// Game seconds one craft takes for this monster: DEX speeds it up, INT a little.
pub fn production_seconds(recipe: &GoodType, stats: &Stats) -> f64 {
    let dex = ability(stats, crate::data::DEX).max(1.0);
    let int = ability(stats, INT);
    (recipe.production_time as f64 * 10.0 / dex * 30.0 / (20.0 + int)).max(1.0)
}

/// Weakest secondary skills are dropped, one per matching transferable skill. An empty list
/// counts as perfect (the original rule; see DESIGN.md for the fallback if this misbehaves).
fn secondary_average(c: &Craft) -> f64 {
    let k = c.crafter.skills.matching(&c.recipe.transferable_skills);
    let mut v: Vec<f64> = c.recipe.secondary_applied_skills.iter().map(|s| c.crafter.skills.applied(s)).collect();
    v.sort_by(f64::total_cmp);
    let kept = &v[k.min(v.len())..];
    if kept.is_empty() {
        1.0
    } else {
        kept.iter().sum::<f64>() / kept.len() as f64
    }
}

/// Tool qualities repeated by weight (the workshop counts with weight 8), lowest two units
/// dropped per matching transferable skill. No tools at all counts as perfect.
fn tool_average(c: &Craft) -> f64 {
    let mut units: Vec<f64> = Vec::new();
    for (i, t) in c.tools.iter().enumerate() {
        units.extend(std::iter::repeat(t.quality).take(c.recipe.tool_weight(i) as usize));
    }
    if let Some(w) = c.workshop {
        units.extend(std::iter::repeat(w.quality).take(WORKSHOP_WEIGHT as usize));
    }
    units.sort_by(f64::total_cmp);
    let k = c.crafter.skills.matching(&c.recipe.transferable_skills) * 2;
    let kept = &units[k.min(units.len())..];
    if kept.is_empty() {
        1.0
    } else {
        kept.iter().sum::<f64>() / kept.len() as f64
    }
}

/// Above 1 the mean is compressed toward QUALITY_MEAN_CAP, so loops can't compound forever.
pub fn soft_cap(m: f64) -> f64 {
    if m <= 1.0 {
        m
    } else {
        let room = QUALITY_MEAN_CAP - 1.0;
        1.0 + room * (1.0 - (-(m - 1.0) / room).exp())
    }
}

/// Wisdom (and a little strength) lifts results that fall short of perfect.
pub fn nudge(q: f64, stats: &Stats) -> f64 {
    let wis_str = ability(stats, WIS) + ability(stats, STR) * 0.25;
    q + wis_str / 25.0 * (1.0 - q).max(0.0) * 0.25
}

pub fn forecast(c: &Craft) -> Forecast {
    let s = c.crafter.skills;
    let input_quality = if c.inputs.is_empty() {
        1.0
    } else {
        c.inputs.iter().map(|i| i.quality).sum::<f64>() / c.inputs.len() as f64
    };
    let tool_quality = tool_average(c);
    let secondary = secondary_average(c);
    let primary = s.applied(&c.recipe.primary_applied_skill);
    let specific = s.specific(&c.recipe.id);
    let difficulty = c.recipe.difficulty_rating.max(1) as f64;
    let ability_factor = (ability(c.crafter.stats, c.recipe.relevant_ability_score) / difficulty).min(ABILITY_FACTOR_CAP);
    let destab = &c.recipe.destabilizer_skills;
    let destab_avg = if destab.is_empty() {
        0.0
    } else {
        destab.iter().map(|d| s.applied(d)).sum::<f64>() / destab.len() as f64
    };
    let (mean, sigma) = if c.recipe.has_quality {
        let material = input_quality * primary * secondary;
        let tool = tool_quality * specific * ability_factor;
        (soft_cap((material + tool) / 2.0), SIGMA_BASE + SIGMA_PER_DESTABILIZER * destab_avg)
    } else {
        // Ungraded goods pass their inputs' quality through unchanged (raw ones count as 1).
        (input_quality, 0.0)
    };
    Forecast { input_quality, primary, secondary, tool_quality, specific, ability: ability_factor, mean, sigma }
}

pub fn roll_quality(c: &Craft, rng: &mut Rng) -> f64 {
    let f = forecast(c);
    if !c.recipe.has_quality {
        return f.mean;
    }
    nudge((f.mean + f.sigma * rng.gauss()).max(0.0), c.crafter.stats)
}

pub fn roll_quantity(c: &Craft, rng: &mut Rng) -> u32 {
    let mu = c.recipe.quantity.max(1) as f64;
    if c.recipe.is_fixed_quantity {
        return mu as u32;
    }
    let f = forecast(c);
    let sigma = mu * 0.05 * ability(c.crafter.stats, c.recipe.relevant_ability_score) * f.primary * f.specific * f.tool_quality * f.secondary;
    let base = (rng.gauss() * sigma).abs() + mu;
    let st = c.crafter.stats;
    let q = base + (base * (ability(st, STR) - 10.0) / 10.0).round() + (base * (ability(st, crate::data::DEX) - 10.0) / 10.0 * 0.25).round();
    q.round().max(1.0) as u32
}

/// Tags that flow from each input slot into the output, per the recipe's carryover lists.
pub fn carried_tags(recipe: &GoodType, inputs: &[&ItemData], data: &Data) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    for (i, carry) in recipe.input_goods_tags_carryover.iter().enumerate() {
        let Some(input) = inputs.get(i) else { continue };
        let tags = input.tags(data);
        for t in carry {
            if tags.contains(t) && !out.contains(t) {
                out.push(t.clone());
            }
        }
    }
    out
}

/// All raw materials behind the inputs, and the refinement depth of the output.
pub fn lineage(inputs: &[&ItemData]) -> (Vec<RawMaterial>, u32) {
    let raw = inputs.iter().flat_map(|i| i.raw.iter().cloned()).collect();
    let depth = 1 + inputs.iter().map(|i| i.depth).max().unwrap_or(0);
    (raw, depth)
}

pub fn weight(recipe: &GoodType, item: &ItemData) -> f64 {
    if let Some(d) = recipe.raw_material_density {
        return d * recipe.storage_volume;
    }
    if item.raw.is_empty() {
        return recipe.storage_volume;
    }
    item.raw.iter().map(|r| r.density).sum::<f64>() / item.raw.len() as f64 * recipe.storage_volume
}

/// Value fixed when the item is made: raw materials scale gently with quality; refined goods
/// sum every raw material behind them and scale more steeply the deeper they are. A
/// charismatic crafter adds value to refined goods.
pub fn value(recipe: &GoodType, raw: &[RawMaterial], depth: u32, quality: f64, crafter: Option<&Stats>) -> f64 {
    let q = quality.max(0.0) + 0.5;
    if let Some(base) = recipe.raw_material_base_value {
        return base * q.sqrt();
    }
    let raw_value: f64 = raw.iter().map(|r| r.base_value).sum();
    let v = raw_value * q.powf(0.5 + 0.5 * depth as f64);
    match crafter {
        Some(st) => v * (10.0 + ability(st, crate::data::CHA) / 2.0) / 10.0,
        None => v,
    }
}

fn add_share(shares: &mut Vec<Share>, monster: u32, count: f64) {
    if count <= 0.0 {
        return;
    }
    match shares.iter_mut().find(|s| s.monster == monster) {
        Some(s) => s.count += count,
        None => shares.push(Share { monster, count }),
    }
}

/// Inputs pass on all their shares; each tool passes on `weight` shares split among its own
/// contributors; the workshop passes on 8; the crafter adds the recipe's value-added shares.
pub fn output_shares(c: &Craft) -> Vec<Share> {
    let mut out = Vec::new();
    for i in &c.inputs {
        for s in &i.shares {
            add_share(&mut out, s.monster, s.count);
        }
    }
    let spread = |shares: &[Share], weight: f64, out: &mut Vec<Share>| {
        let total: f64 = shares.iter().map(|s| s.count).sum();
        if total > 0.0 {
            for s in shares {
                add_share(out, s.monster, weight * s.count / total);
            }
        }
    };
    for (i, t) in c.tools.iter().enumerate() {
        spread(&t.shares, c.recipe.tool_weight(i) as f64, &mut out);
    }
    if let Some(w) = c.workshop {
        spread(&w.shares, WORKSHOP_WEIGHT as f64, &mut out);
    }
    add_share(&mut out, c.crafter.id, c.recipe.value_added_shares as f64);
    out
}

/// The goods one finished craft produces (quantity, quality, tags, history, value, shares).
/// Quality is rolled with the skills the monster had when the craft started.
pub fn produce(c: &Craft, data: &Data, rng: &mut Rng) -> Vec<ItemData> {
    let n = roll_quantity(c, rng);
    let carried = carried_tags(c.recipe, &c.inputs, data);
    let (raw, depth) = if c.recipe.is_raw() {
        let r = c.recipe;
        (vec![RawMaterial { good: r.id.clone(), base_value: r.raw_material_base_value.unwrap_or(0.0), density: r.raw_material_density.unwrap_or(0.0) }], 0)
    } else {
        lineage(&c.inputs)
    };
    let shares = output_shares(c);
    (0..n)
        .map(|_| {
            let quality = roll_quality(c, rng);
            let value = value(c.recipe, &raw, depth, quality, Some(c.crafter.stats));
            ItemData { good: c.recipe.id.clone(), quality, carried_tags: carried.clone(), raw: raw.clone(), depth, value, shares: shares.clone(), wear: 0 }
        })
        .collect()
}

/// What one craft taught, for skill popups and the forecast card.
#[derive(Clone, Debug, Default)]
pub struct Learned {
    pub specific: f64,
    pub primary: f64,
    pub secondary: Vec<(String, f64)>,
    pub forgotten: f64,
}

fn grow(start: f64, steps: usize, rate: f64) -> f64 {
    let mut r = 0.0;
    for _ in 0..steps {
        r += rate * (1.0 - start - r);
    }
    r
}

/// The transferable skills that speed up learning a primary skill. The data only defines
/// them for hauling and wagon driving, so crafts use the recipe's own transferable skills.
fn learning_transferables<'a>(recipe: &'a GoodType, data: &'a Data) -> &'a [String] {
    data.skills
        .relevant_transferable_skills
        .get(&recipe.primary_applied_skill)
        .map(Vec::as_slice)
        .unwrap_or(&recipe.transferable_skills)
}

/// Learning and forgetting for one finished craft lasting `seconds` of game time: every 10
/// seconds each skill closes part of its remaining distance to 1, and the monster forgets a
/// little of everything. Skill totals never drop below the forgetting counter.
pub fn learn(skills: &mut Skills, stats: &Stats, recipe: &GoodType, data: &Data, seconds: f64) -> Learned {
    let steps = (seconds / LEARN_STEP_SECONDS) as usize;
    let ability_factor = (ability(stats, INT) * 0.8 + ability(stats, CON) * 0.2) / 20.0;
    let transferable_factor = 1.0 + skills.matching(learning_transferables(recipe, data)) as f64 / 4.0;
    let primary = recipe.primary_applied_skill.clone();
    let primary_value = skills.applied(&primary);
    let floor = skills.forgotten;

    let specific_gain = grow(skills.specific(&recipe.id), steps, LEARN_SPECIFIC * ability_factor * primary_value);
    let primary_gain = grow(primary_value, steps, LEARN_PRIMARY * ability_factor * transferable_factor);
    let secondary: Vec<(String, f64)> = recipe
        .secondary_applied_skills
        .iter()
        .map(|s| (s.clone(), grow(skills.applied(s), steps, LEARN_SECONDARY * ability_factor)))
        .collect();
    let forgotten = steps as f64 * FORGET * (1.0 - ability(stats, WIS) / 20.0 * 0.25);

    let bump = |m: &mut BTreeMap<String, f64>, key: &str, gain: f64| {
        let total = m.get(key).copied().unwrap_or(floor).max(floor);
        m.insert(key.to_string(), total + gain);
    };
    bump(&mut skills.specific, &recipe.id, specific_gain);
    bump(&mut skills.applied, &primary, primary_gain);
    for (s, g) in &secondary {
        bump(&mut skills.applied, s, *g);
    }
    skills.forgotten += forgotten;
    let floor = skills.forgotten;
    skills.applied.values_mut().chain(skills.specific.values_mut()).for_each(|v| *v = v.max(floor));

    Learned { specific: specific_gain, primary: primary_gain, secondary, forgotten }
}

/// A gathered raw material, the start of every item history.
pub fn gather(recipe: &GoodType, gatherer: Crafter, data: &Data, rng: &mut Rng) -> Vec<ItemData> {
    let c = Craft { recipe, crafter: gatherer, inputs: Vec::new(), tools: Vec::new(), workshop: None };
    produce(&c, data, rng)
}

/// Quantity a craft makes: (fewest, most, average) over many rolls, for the forecast card.
pub fn quantity_range(c: &Craft) -> (u32, u32, f64) {
    if c.recipe.is_fixed_quantity {
        let n = c.recipe.quantity.max(1);
        return (n, n, n as f64);
    }
    let mut rng = Rng::new(0xF0_2E_CA_57);
    let rolls: Vec<u32> = (0..400).map(|_| roll_quantity(c, &mut rng)).collect();
    let avg = rolls.iter().sum::<u32>() as f64 / rolls.len() as f64;
    (*rolls.iter().min().unwrap_or(&1), *rolls.iter().max().unwrap_or(&1), avg)
}

/// How bright a good looks, in half steps of its palette ramps: -2 dull to +2 brilliant.
pub fn quality_level(q: f64) -> i32 {
    match q {
        q if q < 0.25 => -2,
        q if q < 0.5 => -1,
        q if q < 0.9 => 0,
        q if q < EXCEPTIONAL => 1,
        _ => 2,
    }
}

pub fn tier_name(q: f64) -> &'static str {
    match q {
        q if q < 0.25 => "POOR",
        q if q < 0.5 => "PLAIN",
        q if q < 0.9 => "GOOD",
        q if q < EXCEPTIONAL => "FINE",
        q if q < MASTERWORK => "EXCEPTIONAL",
        _ => "MASTERWORK",
    }
}

/// Upper tail of the standard normal, P(Z > z) (Abramowitz and Stegun 7.1.26).
fn upper_tail(z: f64) -> f64 {
    let x = z.abs() / std::f64::consts::SQRT_2;
    let t = 1.0 / (1.0 + 0.327_591_1 * x);
    let poly = t * (0.254_829_592 + t * (-0.284_496_736 + t * (1.421_413_741 + t * (-1.453_152_027 + t * 1.061_405_429))));
    let erfc = poly * (-x * x).exp();
    if z >= 0.0 {
        erfc / 2.0
    } else {
        1.0 - erfc / 2.0
    }
}

/// Chance a craft lands at quality `q` or better (exact for q >= 1, where no nudge applies).
pub fn chance_at_least(f: &Forecast, q: f64) -> f64 {
    if f.sigma <= 0.0 {
        return if f.mean >= q { 1.0 } else { 0.0 };
    }
    upper_tail((q - f.mean) / f.sigma)
}

#[cfg(test)]
#[path = "econ_tests.rs"]
mod tests;
