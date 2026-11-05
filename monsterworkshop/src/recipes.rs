//! Knowing recipes (M3d): naming what a recipe takes as types of things, saying what a
//! workshop makes and what a good goes into, and the ideas a craftsman like Old Sawgrass has
//! about what could be made from what's around him.

use crate::data::{matches, GoodType};
use crate::game::Game;
use crate::world::{ShopKind, SCREEN_H, SCREEN_W};

/// Words that take no "a" in front ("PUT FIBER ON A PAD"): stuff rather than things.
const MASS: [&str; 35] = [
    "FIBER", "THREAD", "FABRIC", "LIME", "QUICKLIME", "LYE", "DYE", "INDIGO", "LIMESTONE", "CLAY", "SAND", "SALT", "SUGAR", "OIL", "CHARCOAL", "COAL", "COKE", "MORTAR", "BEESWAX", "PITCH", "ROSIN", "ALUM", "OCHRE", "FUEL",
    "FLOUR", "MALT", "GRAIN", "BAMBOO", "ROPE", "WOOD", "HEAT", "ORE", "GRANITE", "PYRITE", "SOLDER",
];

/// "A LOG", "AN OAK LOG", "FIBER", "PLANKS": a name as one of it, or as stuff.
pub(crate) fn one(name: &str) -> String {
    let last = name.rsplit(' ').next().unwrap_or(name);
    if last.ends_with('S') || MASS.contains(&last) {
        name.to_string()
    } else if name.starts_with(['A', 'E', 'I', 'O', 'U']) {
        format!("AN {name}")
    } else {
        format!("A {name}")
    }
}

/// "A PICK AND A SHOVEL", "OIL, MORTAR AND BEESWAX": a list as it's said.
pub(crate) fn and_list(parts: &[String]) -> String {
    match parts.split_last() {
        None => String::new(),
        Some((last, [])) => last.clone(),
        Some((last, rest)) => format!("{} AND {last}", rest.join(", ")),
    }
}

/// "2 LOGS", "2 PLANKS", "3 ARROW BAMBOO": several of a name.
fn several(n: usize, name: &str) -> String {
    let last = name.rsplit(' ').next().unwrap_or(name);
    if last.ends_with('S') || MASS.contains(&last) {
        format!("{n} {name}")
    } else {
        format!("{n} {name}S")
    }
}

impl Game {
    /// What a recipe input (or tool) asks for, as a type of thing: the one good that fits if
    /// only one does ("ARROW BAMBOO"), else the last word the fitting goods' names share with
    /// whatever the group says beyond it ("LOG", "HARDWOOD LOG", "BAST STALKS"), else the
    /// group's own words ("FIBER", "DYE").
    pub(crate) fn describe_type(&self, group: &[String]) -> String {
        let fits: Vec<&GoodType> = self.data.goods.iter().filter(|g| matches(&g.type_tags, group)).collect();
        match fits.as_slice() {
            [] => return group.join(" ").to_uppercase(),
            [only] => return only.name.to_uppercase(),
            _ => {}
        }
        // The last word of its name, leaving out a note in brackets ("RICE STALKS (IRRIGATED)").
        let last = |g: &GoodType| g.name.split(" (").next().unwrap_or("").rsplit(' ').next().unwrap_or("").to_uppercase();
        let head = last(fits[0]);
        if !fits.iter().all(|g| last(g) == head) {
            return group.join(" ").to_uppercase();
        }
        // A tag every good called that has anyway ("wood" on every log) says nothing.
        let called: Vec<&GoodType> = self.data.goods.iter().filter(|g| last(g) == head).collect();
        let says = group.iter().filter(|t| t.to_uppercase() != head && !called.iter().all(|g| g.type_tags.contains(t)));
        let mut words: Vec<String> = says.map(|t| t.to_uppercase()).collect();
        words.push(head);
        words.join(" ")
    }

    /// "A LOG", "2 PLANKS, 2 ARROW BAMBOO, A ROPE": everything a recipe takes, repeats counted.
    pub(crate) fn inputs_text(&self, recipe: &GoodType) -> String {
        let mut parts: Vec<(String, usize)> = Vec::new();
        for (_, group) in recipe.input_groups() {
            let name = self.describe_type(group);
            match parts.iter_mut().find(|(n, _)| *n == name) {
                Some(p) => p.1 += 1,
                None => parts.push((name, 1)),
            }
        }
        parts.into_iter().map(|(n, k)| if k == 1 { one(&n) } else { several(k, &n) }).collect::<Vec<_>>().join(", ")
    }

    /// Every recipe some workshop in the game can make (a trader's trades aren't making).
    pub(crate) fn recipes_in_play(&self) -> Vec<&'static str> {
        let mut all: Vec<&'static str> = Vec::new();
        for r in ShopKind::all().filter(|k| !k.trades()).flat_map(|k| k.recipes().iter().copied()) {
            if !all.contains(&r) {
                all.push(r);
            }
        }
        all
    }

    /// What a good goes into, among recipes in play, not counting being burned as fuel (said
    /// once, not forty times).
    pub(crate) fn goes_into(&self, good: &GoodType) -> Vec<&'static str> {
        let mut out = Vec::new();
        for r in self.recipes_in_play() {
            let takes = self.data.good(r).input_groups().any(|(_, grp)| grp.iter().all(|t| t != "fuel") && matches(&good.type_tags, grp));
            if takes && !out.contains(&r) {
                out.push(r);
            }
        }
        out
    }

    /// The workshops that make a good.
    pub(crate) fn made_at(&self, good: &str) -> Vec<ShopKind> {
        ShopKind::all().filter(|k| k.recipes().contains(&good)).collect()
    }
}

/// Tool life by material, where a player meets it (Henry, 2026-11-05: which materials make good
/// tools must be discoverable): on the material's card, the tool's, the workshop's while it
/// makes a tool, and when a tool wears out. Every one gives the same numbers, in durability,
/// and the plant's and workshop's cards say how much each go takes off.
impl Game {
    /// Is this good a tool: something some recipe asks for as one.
    pub(crate) fn is_tool_good(&self, good: &GoodType) -> bool {
        self.data.goods.iter().any(|r| r.tools_required_tags.iter().any(|grp| !grp.is_empty() && matches(&good.type_tags, grp)))
    }

    /// What decides a tool recipe's material: the first material category one of its inputs
    /// carries (the pick's ingot, not its handle), with that input's slot.
    fn material_slot<'a>(&self, recipe: &'a GoodType) -> Option<(&'static str, &'a Vec<String>)> {
        if !self.is_tool_good(recipe) {
            return None;
        }
        crate::data::MATERIALS.iter().find_map(|&c| {
            let k = recipe.input_goods_tags_carryover.iter().position(|carry| carry.iter().any(|t| self.data.variety(t).is_some_and(|v| v.is(c))))?;
            Some((c, recipe.input_goods_tags_required.get(k)?))
        })
    }

    /// The materials that fit a tool recipe's material slot, longest-lasting first, each once:
    /// a pick's hard metals, a spool's sticks. Only those the world has (something grows them, or
    /// a workshop or trader gives them): the data names many more woods than are planted yet.
    fn materials_for(&self, recipe: &GoodType) -> Vec<&crate::data::Variety> {
        let Some((c, group)) = self.material_slot(recipe) else { return Vec::new() };
        let here = |id: &str| self.world.plants.iter().any(|p| p.good == id) || (0..self.world.shops.len()).any(|s| self.offers(s).iter().any(|r| self.data.good(r).output() == id));
        let mut out: Vec<&crate::data::Variety> = Vec::new();
        for g in self.data.goods.iter().filter(|g| matches(&g.type_tags, group) && here(&g.id)) {
            if let Some(v) = g.variety.as_ref().and_then(|v| self.data.variety(&v.name)).filter(|v| v.is(c)) {
                if !out.iter().any(|o| o.tag == v.tag) {
                    out.push(v);
                }
            }
        }
        out.sort_by(|a, b| crate::econ::tool_durability(Some(b)).cmp(&crate::econ::tool_durability(Some(a))).then(a.tag.cmp(&b.tag)));
        out
    }

    /// "DURABILITY BY WOOD: RHODODENDRON 100, ELDER 50.": what a workshop making a tool could
    /// make it of, among what the world has, for its card.
    pub(crate) fn durability_by_material(&self, recipe: &GoodType) -> Option<String> {
        let (c, _) = self.material_slot(recipe)?;
        let all = self.materials_for(recipe);
        (!all.is_empty()).then(|| {
            let each: Vec<String> = all.iter().map(|v| format!("{} {}", v.tag.to_uppercase(), crate::econ::tool_durability(Some(v)))).collect();
            format!("DURABILITY BY {}: {}.", c.to_uppercase(), each.join(", "))
        })
    }

    /// "TOOL DURABILITY 200 (MOST 100).": on the card of a good some tool can be made of (the card
    /// names the material).
    pub(crate) fn tool_life_of(&self, good: &GoodType) -> Option<String> {
        let v = good.variety.as_ref().and_then(|v| self.data.variety(&v.name)).filter(|v| v.material().is_some())?;
        let makes_tools = self.data.goods.iter().filter(|r| self.material_slot(r).is_some_and(|(c, group)| v.is(c) && matches(&good.type_tags, group))).count() > 0;
        makes_tools.then(|| format!("TOOL DURABILITY {}.", against_most(crate::econ::tool_durability(Some(v)))))
    }

    /// What's said when a tool wears out: which, and something lasting longer that can be had
    /// here, if anything can ("THE ELDER SPOOL WORE OUT. RHODODENDRON LASTS LONGER: DURABILITY
    /// 100, ELDER 50.").
    pub(crate) fn wore_out(&self, whose: &str, tool: &crate::econ::ItemData) -> String {
        let said = format!("{whose} {} WORE OUT.", crate::inspect::display_name(self, tool));
        let Some(made_of) = self.data.material(&tool.tags(&self.data)) else { return said };
        let had = crate::econ::tool_durability(Some(made_of));
        match self.materials_for(self.data.good(&tool.good)).into_iter().find(|v| crate::econ::tool_durability(Some(v)) > had) {
            Some(better) => format!("{said} {} LASTS LONGER: DURABILITY {}, {} {had}.", better.tag.to_uppercase(), crate::econ::tool_durability(Some(better)), made_of.tag.to_uppercase()),
            None => said,
        }
    }

    /// "WEARS PICK 3, SHOVEL 3.": what one go at `recipe` takes off the durability of
    /// each tool monster `m` would use, its own gear counted (a whetstone halves it). `wears(k)`
    /// says whether the k-th tool would wear at all (a town workshop's own never does). Durability
    /// alone doesn't say how many goes a tool has: digging wears a pick 3, a loom's weaving 8.
    pub(crate) fn wear_each_time(&self, recipe: &GoodType, m: usize, wears: impl Fn(usize) -> bool) -> Option<String> {
        // A working townsperson's own tools never wear.
        if self.monsters[m].worker.is_some() {
            return None;
        }
        let knack = self.monsters[m].knack(&self.data, &recipe.primary_applied_skill);
        let each: Vec<String> = recipe
            .tools_required_tags
            .iter()
            .filter(|g| !g.is_empty())
            .enumerate()
            .filter(|&(k, _)| wears(k))
            .map(|(k, group)| {
                let w = format!("{:.2}", knack.wear_of(recipe.tool_weight(k)));
                format!("{} {}", self.describe_type(group), w.trim_end_matches('0').trim_end_matches('.'))
            })
            .collect();
        (!each.is_empty()).then(|| format!("WEARS {}.", each.join(", ")))
    }
}

/// "200 (MOST 100)", or "100, AS MOST": a tool durability against the usual.
pub(crate) fn against_most(n: u32) -> String {
    match crate::econ::TOOL_DURABILITY {
        most if n == most => format!("{n}, AS MOST"),
        most => format!("{n} (MOST {most})"),
    }
}

/// An idea a craftsman has: a recipe a workshop near him could make, and what it lacks, if
/// anything (the nearest miss: one input short).
pub struct Idea {
    pub recipe: &'static str,
    pub shop: usize,
    pub missing: Option<Vec<String>>,
}

impl Game {
    /// What craftsman `t` counts as "around here", for commune `k`'s player: every good lying
    /// on his screen, in its bins, growing there, and in the hands of the monster talking to
    /// him, as the tags each would be matched by.
    fn around(&self, t: usize, k: usize) -> Vec<Vec<String>> {
        let tf = &self.world.townsfolk[t];
        let on = |x: i32, y: i32| x.div_euclid(SCREEN_W) == tf.x.div_euclid(SCREEN_W) && y.div_euclid(SCREEN_H) == tf.y.div_euclid(SCREEN_H);
        let tags = |d: &crate::econ::ItemData| d.tags(&self.data);
        let mut out: Vec<Vec<String>> = self.world.items.iter().filter(|it| on(it.x, it.y) && !it.locked).map(|it| tags(&it.data)).collect();
        for b in self.world.bins.iter().filter(|b| on(b.x, b.y)) {
            out.extend(b.stock.iter().map(tags));
        }
        for p in self.world.plants.iter().filter(|p| on(p.x, p.y)) {
            out.extend(self.data.try_good(&p.good).map(|g| g.type_tags.clone()));
        }
        let c = self.communes[k].controlled;
        out.extend(self.monsters[c].tools().map(tags));
        out
    }

    /// The ideas craftsman `t` has for commune `k`: recipes a workshop on his screen could make
    /// right now from what's around (with its tools to hand), else those one input short;
    /// what the commune has never made first.
    pub(crate) fn ideas(&self, t: usize, k: usize) -> Vec<Idea> {
        let tf = &self.world.townsfolk[t];
        let units = self.around(t, k);
        let on = |x: i32, y: i32| x.div_euclid(SCREEN_W) == tf.x.div_euclid(SCREEN_W) && y.div_euclid(SCREEN_H) == tf.y.div_euclid(SCREEN_H);
        let (mut ready, mut short) = (Vec::new(), Vec::new());
        // A craftsman's ideas are for making things, not trading.
        for s in (0..self.world.shops.len()).filter(|&s| on(self.world.shops[s].spot.0, self.world.shops[s].spot.1) && !self.worn_out(s) && !self.world.shops[s].kind.trades()) {
            for &r in self.world.shops[s].kind.recipes() {
                let recipe = self.data.good(r);
                let mut used = Vec::new();
                let mut missing = Vec::new();
                for (_, grp) in recipe.input_groups() {
                    match (0..units.len()).find(|&u| !used.contains(&u) && matches(&units[u], grp)) {
                        Some(u) => used.push(u),
                        None => missing.push(grp.clone()),
                    }
                }
                // The tools: on its pegs or lent, or lying around to bring.
                let tools = self.missing_tool(s, r).is_ok() || recipe.tools_required_tags.iter().filter(|g| !g.is_empty()).all(|grp| units.iter().any(|u| matches(u, grp)));
                if !tools {
                    continue;
                }
                // A near miss has something of it here already: one input short, not one of one.
                match missing.len() {
                    0 => ready.push(Idea { recipe: r, shop: s, missing: None }),
                    1 if !used.is_empty() => short.push(Idea { recipe: r, shop: s, missing: missing.pop() }),
                    _ => {}
                }
            }
        }
        let made = &self.communes[k].made;
        let list = if ready.is_empty() { short } else { ready };
        let (mut new, old): (Vec<Idea>, Vec<Idea>) = list.into_iter().partition(|i| !made.contains(i.recipe));
        new.extend(old);
        new
    }

    /// What craftsman `t` says to commune `k`'s player now: a thoughtful "HMM..." when someone
    /// works near his door, and the idea it's their turn for.
    /// What a working townsperson says, facing them: what their bench makes from what, and what
    /// gathering from their plants takes in hand (so watching them teaches it), then their own
    /// words and where their trade's secret is, one at a time; Space asks for the next.
    pub(crate) fn worker_says(&self, j: usize, k: usize) -> Vec<String> {
        let Some(w) = self.monsters[j].worker.and_then(|n| self.world.workers.get(n)) else { return Vec::new() };
        let id = Some(self.monsters[j].id);
        let bench = self.world.shops.iter().position(|s| s.owner == id);
        let mut lines: Vec<String> = bench.map(|s| self.offers(s)).unwrap_or_default().iter().map(|r| format!("{} TAKES {}.", one(&self.good_name(r)), self.inputs_text(self.data.good(r)))).collect();
        let mut gathers: Vec<&str> = self.world.plants.iter().filter(|p| p.owner == id).map(|p| p.good.as_str()).collect();
        gathers.dedup();
        for good in gathers {
            let tools: Vec<String> = self.data.good(good).tools_required_tags.iter().filter(|t| !t.is_empty()).map(|t| one(&self.describe_type(t))).collect();
            if !tools.is_empty() {
                lines.push(format!("{} TAKES {} IN HAND.", crate::inspect::plant_name(&self.data, good), and_list(&tools)));
            }
        }
        lines.extend(w.says.iter().map(|s| s.to_string()));
        let secret = self.data.piece(w.secret.1);
        lines.push(format!("I'VE WRITTEN THE {} ON THE SCROLL NEARBY, FOR MONSTERS OF SOME RENOWN. FACE IT TO SEE WHAT IT DOES.", secret.name.to_uppercase()));
        let n = self.communes[k].idea_turn % lines.len().max(1);
        lines.into_iter().nth(n).into_iter().collect()
    }

    /// What the trader says, facing him: what his stall gives, while it gives anything.
    pub(crate) fn trader_says(&self, t: usize) -> Vec<String> {
        let tf = &self.world.townsfolk[t];
        let stall = (0..self.world.shops.len()).filter(|&s| self.world.shops[s].kind.trades()).min_by_key(|&s| (self.world.shops[s].spot.0 - tf.x).abs() + (self.world.shops[s].spot.1 - tf.y).abs());
        let names: Vec<String> = stall.map(|s| self.offers(s)).unwrap_or_default().iter().map(|r| self.good_name(self.data.good(r).output())).collect();
        if names.is_empty() {
            return vec!["YOU MAKE EVERYTHING I'VE GOT NOW. I'LL BE HEADING HOME.".into()];
        }
        let list = and_list(&names);
        vec![format!("I'VE GOT {list} FROM THE HILLS, FOR YOUR CLOTH, THREAD AND ROPE. PUT THEM ON MY COUNTER.")]
    }

    pub(crate) fn craftsman_says(&self, t: usize, k: usize) -> Vec<String> {
        let tf = &self.world.townsfolk[t];
        let mut said = Vec::new();
        let near = |s: &crate::world::Shop| (s.spot.0 - tf.x).abs() + (s.spot.1 - tf.y).abs() <= 10;
        if self.world.shops.iter().any(|s| near(s) && s.job.is_some()) {
            // Watching someone work by his door (班门弄斧, an axe swung at Lu Ban's).
            said.push("HMM...".to_string());
        }
        let ideas = self.ideas(t, k);
        said.push(match ideas.get(self.communes[k].idea_turn % ideas.len().max(1)) {
            Some(Idea { recipe, shop, missing: None }) => {
                format!("I BET YOU COULD MAKE {} WITH WHAT'S AROUND HERE. THE {} COULD DO IT.", one(&self.good_name(recipe)), self.shop_name(*shop))
            }
            Some(Idea { recipe, shop, missing: Some(grp) }) => {
                format!("WITH ONE MORE {} YOU COULD MAKE {} AT THE {}.", self.describe_type(grp), one(&self.good_name(recipe)), self.shop_name(*shop))
            }
            None => "BRING SOME GOODS BY AND I'LL TELL YOU WHAT THEY COULD BECOME.".to_string(),
        });
        said
    }
}
