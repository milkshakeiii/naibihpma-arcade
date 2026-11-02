//! Timed work: harvesting plants, crafting at workshops, chutes and output mats. Every
//! finished job goes through the crafting rules in `econ.rs`, so goods carry real quality,
//! varieties, history and shares.

use crate::data::matches;
use crate::econ::{self, Craft, Crafter, ItemData, Workshop};
use crate::game::{Ev, Game, Sfx, UseRes};
use crate::world::*;

/// Game seconds that pass per 15 FPS tick: 4 makes a real second worth a game minute, the
/// fast single-player pace. A persistent world would run slower.
pub const GAME_SECONDS_PER_TICK: f64 = 4.0;
const LEARNED_POPUP_TICKS: u16 = 45;

enum ToolSource {
    Peg(usize),
    Builtin(&'static str),
}

/// A town workshop's built-in tool, used when nothing better hangs on its peg.
fn builtin_tool(id: &str) -> ItemData {
    ItemData { good: id.to_string(), quality: BUILTIN_TOOL_QUALITY, carried_tags: Vec::new(), raw: Vec::new(), depth: 1, value: 0.0, shares: Vec::new(), wear: 0, haul: Vec::new(), carted: None }
}

/// What a work spot would make right now, for the forecast card.
pub(crate) struct Preview {
    pub recipe: String,
    /// The inputs on the pads, or what is missing.
    pub inputs: Result<Vec<ItemData>, String>,
    /// The tools the craft would use, or the one that is missing.
    pub tools: Result<Vec<ItemData>, String>,
    pub workshop: Option<Workshop>,
}

impl Game {
    pub(crate) fn job_ticks(&self, recipe: &str, monster: usize) -> u32 {
        let secs = econ::production_seconds(self.data.good(recipe), &self.monsters[monster].stats);
        ((secs / GAME_SECONDS_PER_TICK).round() as u32).max(1)
    }

    /// "COTTON SLIVERS" for a recipe's input, for hints: a good this town makes or grows if
    /// one fits, otherwise any good that fits.
    pub(crate) fn describe_group(&self, group: &[String]) -> String {
        let local = |id: &str| self.world.shops.iter().any(|s| s.kind.recipes().contains(&id)) || self.world.plants.iter().any(|p| p.good == id);
        let fits: Vec<&crate::data::GoodType> = self.data.goods.iter().filter(|g| matches(&g.type_tags, group)).collect();
        fits.iter()
            .find(|g| local(&g.id))
            .or(fits.first())
            .map_or_else(|| group.join(" ").to_uppercase(), |g| g.name.to_uppercase())
    }

    /// Space on a plant: harvest it if it is ripe. A plant that still holds goods sets them
    /// out on its own, so Space there just succeeds.
    pub(crate) fn use_plant(&mut self, i: usize, p: usize, _sfx: &mut Vec<Sfx>) -> UseRes {
        let plant = &self.world.plants[p];
        if !plant.stock.is_empty() {
            return UseRes::Done;
        }
        if plant.harvest.is_some() {
            return UseRes::Wait("SOMEONE IS ALREADY HARVESTING.".into());
        }
        if !plant.ripe() {
            return UseRes::Wait("STILL GROWING.".into());
        }
        let m = &self.monsters[i];
        let facing = m.facing;
        let (dx, dy) = facing.delta();
        let drop = (m.x + dx, m.y + dy);
        if !self.world.item_floor(drop.0, drop.1) {
            return UseRes::Wait("NO ROOM TO DROP IT THERE.".into());
        }
        let (good, x, y) = (plant.good.clone(), plant.x, plant.y);
        let total = self.job_ticks(&good, i);
        let plant = &mut self.world.plants[p];
        plant.harvest = Some(Job { monster: i, recipe: good, t: 0, total });
        plant.drop = Some((drop, facing));
        self.monsters[i].busy = true;
        self.world.burst(x, y, PKind::Leaf, 3);
        UseRes::Done
    }

    /// A harvested plant sets its goods out on its drop tile, one at a time as it clears.
    fn dispense_plant(&mut self, p: usize, sfx: &mut Vec<Sfx>) {
        let plant = &self.world.plants[p];
        let Some(((x, y), dir)) = plant.drop else { return };
        if plant.stock.is_empty() {
            return;
        }
        let free = self.world.item_floor(x, y) && self.world.item_at(x, y).is_none() && self.monster_at(x, y).is_none();
        let plant = &mut self.world.plants[p];
        if !free {
            return;
        }
        plant.timer = plant.timer.saturating_add(1);
        if plant.timer < DISPENSE_TICKS {
            return;
        }
        plant.timer = 0;
        let data = plant.stock.remove(0);
        if plant.stock.is_empty() {
            plant.growth = 0;
        }
        let (px, py) = (plant.x, plant.y);
        let slide = Some(Slide { dir, left: crate::game::PUSH_TICKS, total: crate::game::PUSH_TICKS });
        self.world.items.push(Item { slide, hop: 6, ..Item::new(x, y, data) });
        self.world.burst(px, py, PKind::Leaf, 4);
        sfx.push(Sfx::Pop);
    }

    pub(crate) fn use_shop(&mut self, i: usize, s: usize, sfx: &mut Vec<Sfx>) -> UseRes {
        if self.worn_out(s) {
            return UseRes::Wait(self.worn_note(s));
        }
        let shop = &self.world.shops[s];
        if shop.job.is_some() {
            return UseRes::Wait(format!("THE {} IS BUSY.", shop.kind.label()));
        }
        let kind = shop.kind;
        // Extra output waits inside, so a loop that takes one per cycle never jams.
        if shop.pending.len() >= OUTPUT_BUFFER {
            return UseRes::Full(format!("THE {} IS FULL. CLEAR ITS OUTPUT MAT.", kind.label()));
        }
        // A loop leaves what another monster has reserved on the pads alone.
        let ok = |it: usize| self.may_take(i, it);
        let Some((recipe, inputs)) = self.best_recipe(s, kind.recipes(), &ok) else {
            return UseRes::Missing(self.pads_note(s, &ok));
        };
        if let Err(missing) = self.match_tools(s, &recipe) {
            return UseRes::Wait(format!("MISSING A TOOL: {missing}."));
        }
        for &it in &inputs {
            self.world.items[it].locked = true;
        }
        // A frontier workshop works faster the more its kit's materials were worth.
        let speed = self.world.shops[s].frontier.as_ref().map_or(1.0, |f| f.speed);
        let total = ((self.job_ticks(&recipe, i) as f64 / speed).round() as u32).max(1);
        self.world.shops[s].job = Some(Job { monster: i, recipe, t: 0, total });
        self.monsters[i].busy = true;
        sfx.push(Sfx::Spin);
        UseRes::Done
    }

    /// What workshop `s`'s pads lack, said so it's clear what to do: the first input not there
    /// yet of the recipe they come closest to ("PUT ANOTHER QUICKLIME ON A PAD." when indigo has
    /// one of its two), or, with every pad taken, what to swap for it ("SWAP THE LYE FOR ANOTHER
    /// QUICKLIME."). Only pad items `ok` accepts count.
    pub(crate) fn pads_note(&self, s: usize, ok: &dyn Fn(usize) -> bool) -> String {
        let shop = &self.world.shops[s];
        let on_pads: Vec<usize> = shop.pads.iter().filter_map(|&(x, y)| self.world.item_at(x, y)).filter(|&it| ok(it)).collect();
        let taken = shop.pads.iter().filter(|&&(x, y)| self.world.item_at(x, y).is_some()).count();
        let shown = shop.default_recipe(&self.data);
        // How close each recipe comes: using the most of what's there, then the one an empty
        // workshop shows (the one that fills its pads), then the fewest inputs missing.
        let closest = shop.kind.recipes().iter().enumerate().map(|(n, &r)| {
            let (mut used, mut missing) = (Vec::new(), Vec::new());
            for (_, group) in self.data.good(r).input_groups() {
                match on_pads.iter().copied().find(|&it| !used.contains(&it) && self.world.item_matches(&self.data, it, group)) {
                    Some(it) => used.push(it),
                    None => missing.push(group.clone()),
                }
            }
            ((std::cmp::Reverse(used.len()), r != shown, missing.len(), n), missing, used)
        });
        let Some((_, missing, used)) = closest.min_by_key(|(key, _, _)| *key) else { return String::new() };
        let Some(group) = missing.first() else { return String::new() };
        let another = used.iter().any(|&it| self.world.item_matches(&self.data, it, group));
        let want = format!("{}{}", if another { "ANOTHER " } else { "" }, self.describe_group(group));
        match on_pads.iter().find(|it| !used.contains(it)) {
            Some(&spare) if taken >= shop.pads.len() => format!("SWAP THE {} FOR {want}.", self.good_name(self.world.items[spare].good())),
            _ => format!("PUT {want} ON A PAD."),
        }
    }

    pub(crate) fn preview(&self, s: usize) -> Preview {
        let kind = self.world.shops[s].kind;
        let found = self.best_recipe(s, kind.recipes(), &|_| true);
        let (recipe, inputs) = match found {
            Some((r, idx)) => (r, Ok(idx.iter().map(|&i| self.world.items[i].data.clone()).collect())),
            None => (self.world.shops[s].default_recipe(&self.data).to_string(), Err(self.pads_note(s, &|_| true))),
        };
        let tools = self.match_tools(s, &recipe).map(|src| {
            src.iter()
                .map(|t| match t {
                    ToolSource::Peg(i) => self.world.items[*i].data.clone(),
                    ToolSource::Builtin(id) => builtin_tool(id),
                })
                .collect()
        });
        let shop = &self.world.shops[s];
        // A frontier workshop's makers share in everything made there.
        let shares = shop.frontier.as_ref().map_or_else(Vec::new, |f| f.shares.clone());
        let workshop = shop.kind.is_building().then(|| Workshop { quality: shop.quality, shares });
        Preview { recipe, inputs, tools, workshop }
    }

    /// The recipe the pads hold ingredients for. When several fit, the one that uses the most
    /// of what's there wins, so laying out a hammer's ingot, fuel, rope and sticks makes the
    /// hammer, not a knife from the ingot and fuel alone. Equal fits go to the earlier recipe
    /// (a signboard will choose between them once a workshop has a real tie).
    /// Only pad items `ok` accepts count.
    pub(crate) fn best_recipe(&self, s: usize, recipes: &[&str], ok: &dyn Fn(usize) -> bool) -> Option<(String, Vec<usize>)> {
        recipes.iter().rev().filter_map(|r| self.match_inputs(s, r, ok).map(|inputs| (r.to_string(), inputs))).max_by_key(|(_, inputs)| inputs.len())
    }

    /// Items on the pads that satisfy each of the recipe's inputs, in recipe order.
    fn match_inputs(&self, s: usize, recipe: &str, ok: &dyn Fn(usize) -> bool) -> Option<Vec<usize>> {
        let r = self.data.good(recipe);
        let on_pads: Vec<usize> = self.world.shops[s].pads.iter().filter_map(|&(x, y)| self.world.item_at(x, y)).filter(|&it| ok(it)).collect();
        let mut used = Vec::new();
        for (_, group) in r.input_groups() {
            let found = on_pads.iter().copied().find(|it| !used.contains(it) && self.world.item_matches(&self.data, *it, group))?;
            used.push(found);
        }
        Some(used)
    }

    /// The tool a recipe needs that this workshop doesn't have, if any.
    pub(crate) fn missing_tool(&self, s: usize, recipe: &str) -> Result<(), String> {
        self.match_tools(s, recipe).map(|_| ())
    }

    /// For each tool the recipe needs: a working tool on a peg, or the workshop's own.
    fn match_tools(&self, s: usize, recipe: &str) -> Result<Vec<ToolSource>, String> {
        let r = self.data.good(recipe);
        let shop = &self.world.shops[s];
        let mut used = Vec::new();
        let mut out = Vec::new();
        for group in r.tools_required_tags.iter().filter(|g| !g.is_empty()) {
            let on_peg = shop.pegs.iter().filter_map(|&(x, y)| self.world.peg_item(x, y)).find(|it| {
                !used.contains(it) && self.world.items[*it].data.durability_left() > 0 && self.world.item_matches(&self.data, *it, group)
            });
            if let Some(it) = on_peg {
                used.push(it);
                out.push(ToolSource::Peg(it));
            } else if let Some(id) = shop.kind.builtin_tools().iter().filter(|_| shop.frontier.is_none()).find(|id| matches(&self.data.good(id).type_tags, group)) {
                out.push(ToolSource::Builtin(id));
            } else {
                return Err(self.describe_group(group));
            }
        }
        Ok(out)
    }

    pub(crate) fn tick_work(&mut self, sfx: &mut Vec<Sfx>) {
        for p in 0..self.world.plants.len() {
            let plant = &mut self.world.plants[p];
            if plant.stock.is_empty() && plant.harvest.is_none() {
                plant.growth = (plant.growth + 1).min(plant.regrow);
            }
            let (x, y) = (plant.x, plant.y);
            let done = plant.harvest.as_mut().is_some_and(|j| {
                j.t += 1;
                j.t >= j.total
            });
            if done {
                self.finish_harvest(p);
            } else if self.world.plants[p].harvest.is_some() && self.frame % 5 == 0 {
                self.world.burst(x, y, PKind::Leaf, 1);
            }
            self.dispense_plant(p, sfx);
        }
        for s in 0..self.world.shops.len() {
            let heard = self.heard(self.world.shops[s].spot);
            let shop = &mut self.world.shops[s];
            let mut done = false;
            let mut fluff = None;
            if let Some(j) = shop.job.as_mut() {
                j.t += 1;
                done = j.t >= j.total;
                if j.t % 3 == 0 && !shop.pads.is_empty() {
                    fluff = Some(shop.pads[(j.t as usize / 3) % shop.pads.len()]);
                }
                if j.t % 6 == 0 && heard {
                    sfx.push(Sfx::Spin);
                }
            }
            if let Some((px, py)) = fluff {
                self.world.burst(px, py, PKind::Fluff, 1);
            }
            if done {
                self.finish_shop(s, sfx);
            }
            self.place_output(s, sfx);
            for f in self.world.shops[s].pad_flash.iter_mut() {
                *f = f.saturating_sub(1);
            }
        }
        self.tick_bins(sfx);
    }

    fn finish_harvest(&mut self, p: usize) {
        let Some(job) = self.world.plants[p].harvest.take() else { return };
        let m = job.monster;
        let recipe = self.data.good(&job.recipe).clone();
        let (stats, skills, id) = (self.monsters[m].stats.clone(), self.monsters[m].skills.clone(), self.monsters[m].id);
        let items = econ::gather(&recipe, Crafter { id, stats: &stats, skills: &skills }, &self.data, &mut self.rng);
        let secs = econ::production_seconds(&recipe, &stats);
        let learned = econ::learn(&mut self.monsters[m].skills, &stats, &recipe, &self.data, secs);
        self.monsters[m].learned = Some((recipe.id.clone(), learned, LEARNED_POPUP_TICKS));
        self.monsters[m].busy = false;
        let plant = &mut self.world.plants[p];
        plant.stock = items;
        plant.growth = 0;
        plant.timer = DISPENSE_TICKS; // the first one comes out straight away
        self.events.push(Ev::Made(recipe.id.clone()));
    }

    fn finish_shop(&mut self, s: usize, sfx: &mut Vec<Sfx>) {
        let Some(job) = self.world.shops[s].job.take() else { return };
        let m = job.monster;
        let recipe = self.data.good(&job.recipe).clone();
        // The inputs locked when the job started.
        let inputs_idx = self.match_inputs(s, &job.recipe, &|it| self.world.items[it].locked).unwrap_or_default();
        let tool_src = self.match_tools(s, &job.recipe).unwrap_or_default();
        // Using the inputs pays whoever hauled them here; that credit flows into the output.
        let inputs: Vec<ItemData> = inputs_idx
            .iter()
            .map(|&i| {
                let mut d = self.world.items[i].data.clone();
                econ::pay_haul(&mut d);
                d
            })
            .collect();
        let tools: Vec<ItemData> = tool_src
            .iter()
            .map(|t| match t {
                ToolSource::Peg(i) => self.world.items[*i].data.clone(),
                ToolSource::Builtin(id) => builtin_tool(id),
            })
            .collect();
        let shop = &self.world.shops[s];
        // A frontier workshop's makers share in everything made there.
        let shares = shop.frontier.as_ref().map_or_else(Vec::new, |f| f.shares.clone());
        let workshop = shop.kind.is_building().then(|| Workshop { quality: shop.quality, shares });
        let (stats, skills, id) = (self.monsters[m].stats.clone(), self.monsters[m].skills.clone(), self.monsters[m].id);
        let craft = Craft { recipe: &recipe, crafter: Crafter { id, stats: &stats, skills: &skills }, inputs: inputs.iter().collect(), tools: tools.iter().collect(), workshop: workshop.as_ref() };
        let produced = econ::produce(&craft, &self.data, &mut self.rng);
        let secs = econ::production_seconds(&recipe, &stats);
        let learned = econ::learn(&mut self.monsters[m].skills, &stats, &recipe, &self.data, secs);

        // Inputs are used up; tools on pegs wear by their weight and may break.
        let mut gone: Vec<(i32, i32)> = inputs_idx.iter().map(|&i| (self.world.items[i].x, self.world.items[i].y)).collect();
        let mut broke = false;
        for (k, t) in tool_src.iter().enumerate() {
            if let ToolSource::Peg(i) = t {
                let it = &mut self.world.items[*i];
                it.data.wear += recipe.tool_weight(k);
                if it.data.durability_left() == 0 {
                    gone.push((it.x, it.y));
                    broke = true;
                }
            }
        }
        for &(x, y) in &gone {
            self.world.burst(x, y, PKind::Fluff, 5);
        }
        self.world.items.retain(|it| !gone.contains(&(it.x, it.y)));

        let (sx, sy) = self.world.shops[s].spot;
        let shop = &mut self.world.shops[s];
        shop.pending.extend(produced);
        shop.out_timer = DISPENSE_TICKS;
        self.world.burst(sx, sy, PKind::Spark, 6);
        self.monsters[m].busy = false;
        self.monsters[m].learned = Some((recipe.id.clone(), learned, LEARNED_POPUP_TICKS));
        self.events.push(Ev::Made(recipe.id.clone()));
        if self.heard((sx, sy)) {
            sfx.push(Sfx::Chime);
        }
        if broke {
            self.say(m, "A TOOL WORE OUT!", true);
        } else if !self.examine_hinted && m == self.controlled {
            self.examine_hinted = true;
            self.say(m, "DONE! HOLD E TO EXAMINE THINGS.", false);
        }
    }

    /// Finished goods appear on a free output mat, one at a time.
    fn place_output(&mut self, s: usize, sfx: &mut Vec<Sfx>) {
        let shop = &self.world.shops[s];
        if shop.pending.is_empty() {
            return;
        }
        let Some((x, y)) = shop.outs.iter().copied().find(|&(x, y)| self.world.item_at(x, y).is_none() && self.monster_at(x, y).is_none()) else {
            return;
        };
        let shop = &mut self.world.shops[s];
        shop.out_timer = shop.out_timer.saturating_add(1);
        if shop.out_timer < DISPENSE_TICKS {
            return;
        }
        shop.out_timer = 0;
        let data = shop.pending.remove(0);
        self.world.items.push(Item { hop: 5, ..Item::new(x, y, data) });
        self.world.burst(x, y, PKind::Spark, 10);
        if self.heard((x, y)) {
            sfx.push(Sfx::Plop);
        }
    }

    /// Bins with a chute push their oldest item out whenever the chute is clear.
    fn tick_bins(&mut self, sfx: &mut Vec<Sfx>) {
        for b in 0..self.world.bins.len() {
            let bin = &self.world.bins[b];
            let Some((mx, my)) = bin.mouth else {
                let bin = &mut self.world.bins[b];
                bin.bounce = bin.bounce.saturating_sub(1);
                continue;
            };
            let blocked = self.world.item_at(mx, my).is_some() || self.monster_at(mx, my).is_some();
            let bin = &mut self.world.bins[b];
            bin.bounce = bin.bounce.saturating_sub(1);
            if blocked || bin.stock.is_empty() {
                bin.timer = 0;
                continue;
            }
            bin.timer += 1;
            if bin.timer < DISPENSE_TICKS {
                continue;
            }
            bin.timer = 0;
            bin.bounce = 3;
            let dir = if my > bin.y { Dir::Down } else if my < bin.y { Dir::Up } else if mx > bin.x { Dir::Right } else { Dir::Left };
            let data = bin.stock.remove(0);
            let slide = Some(Slide { dir, left: crate::game::PUSH_TICKS, total: crate::game::PUSH_TICKS });
            self.world.items.push(Item { slide, hop: 4, ..Item::new(mx, my, data) });
            if self.heard((mx, my)) {
                sfx.push(Sfx::Plop);
            }
        }
    }
}
