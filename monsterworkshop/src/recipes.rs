//! Knowing recipes (M3d): naming what a recipe takes as types of things, saying what a
//! workshop makes and what a good goes into, and the ideas a craftsman like Old Sawgrass has
//! about what could be made from what's around him.

use crate::data::{matches, GoodType};
use crate::game::Game;
use crate::world::{ShopKind, SCREEN_H, SCREEN_W};

/// Words that take no "a" in front ("PUT FIBER ON A PAD"): stuff rather than things.
const MASS: [&str; 30] = [
    "FIBER", "THREAD", "FABRIC", "LIME", "QUICKLIME", "LYE", "DYE", "INDIGO", "LIMESTONE", "CLAY", "SAND", "SALT", "SUGAR", "OIL", "CHARCOAL", "COAL", "COKE", "MORTAR", "BEESWAX", "PITCH", "ROSIN", "ALUM", "OCHRE", "FUEL",
    "FLOUR", "MALT", "GRAIN", "BAMBOO", "ROPE", "WOOD",
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
        out.extend(self.monsters[c].hands.iter().flatten().map(tags));
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
    /// What a working townsperson says, facing them: what their bench makes from what (so
    /// watching them teaches it), then their own words, one at a time; Space asks for the next.
    pub(crate) fn worker_says(&self, j: usize, k: usize) -> Vec<String> {
        let Some(w) = self.monsters[j].worker.and_then(|n| self.world.workers.get(n)) else { return Vec::new() };
        let bench = self.world.shops.iter().position(|s| s.owner == Some(self.monsters[j].id));
        let mut lines: Vec<String> = bench.map(|s| self.offers(s)).unwrap_or_default().iter().map(|r| format!("{} TAKES {}.", one(&self.good_name(r)), self.inputs_text(self.data.good(r)))).collect();
        lines.extend(w.says.iter().map(|s| s.to_string()));
        let n = self.communes[k].idea_turn % lines.len().max(1);
        lines.into_iter().nth(n).into_iter().collect()
    }

    /// What the trader says, facing him: what his stall gives, while it gives anything.
    pub(crate) fn trader_says(&self, t: usize) -> Vec<String> {
        let tf = &self.world.townsfolk[t];
        let stall = (0..self.world.shops.len()).filter(|&s| self.world.shops[s].kind.trades()).min_by_key(|&s| (self.world.shops[s].spot.0 - tf.x).abs() + (self.world.shops[s].spot.1 - tf.y).abs());
        let names: Vec<String> = stall.map(|s| self.offers(s)).unwrap_or_default().iter().map(|r| self.good_name(self.data.good(r).output())).collect();
        let Some((last, rest)) = names.split_last() else {
            return vec!["YOU MAKE EVERYTHING I'VE GOT NOW. I'LL BE HEADING HOME.".into()];
        };
        let list = if rest.is_empty() { last.clone() } else { format!("{} AND {last}", rest.join(", ")) };
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
