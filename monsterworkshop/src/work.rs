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
/// How long a trade takes: two seconds, however skilled the monster, since the trader does it.
const TRADE_TICKS: u32 = 30;

enum ToolSource {
    Peg(usize),
    Builtin(&'static str),
}

/// A town workshop's built-in tool, used when nothing better hangs on its peg.
fn builtin_tool(id: &str) -> ItemData {
    ItemData { good: id.to_string(), quality: BUILTIN_TOOL_QUALITY, carried_tags: Vec::new(), raw: Vec::new(), depth: 1, value: 0.0, shares: Vec::new(), wear: 0, haul: Vec::new(), carted: None }
}

/// What a workshop's pads are working toward (see `Game::aim`).
pub(crate) struct Aim {
    pub recipe: String,
    /// The pad items it uses, in recipe order.
    pub used: Vec<usize>,
    /// What it still needs, in recipe order: nothing when the pads make it now.
    pub missing: Vec<Vec<String>>,
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
        if self.data.good(recipe).is_trade() {
            return TRADE_TICKS;
        }
        let secs = econ::production_seconds(self.data.good(recipe), &self.monsters[monster].stats);
        ((secs / GAME_SECONDS_PER_TICK).round() as u32).max(1)
    }

    /// "COTTON SLIVERS" for a recipe's input, for hints: a good this town makes or grows if
    /// one fits, otherwise any good that fits.
    pub(crate) fn describe_group(&self, group: &[String]) -> String {
        let open = |(x, y): (i32, i32)| !self.world.closed_at(x, y);
        let local = |id: &str| self.world.shops.iter().any(|s| open(s.spot) && s.kind.recipes().contains(&id)) || self.world.plants.iter().any(|p| open((p.x, p.y)) && p.good == id);
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
        if let Some(name) = plant.owner.filter(|&o| o != self.monsters[i].id).and_then(|o| self.monsters.iter().find(|m| m.id == o)).map(|m| m.name.clone()) {
            let what = crate::inspect::plant_name(&self.data, &plant.good);
            return UseRes::Wait(format!("THAT'S {name}'S OWN {what}. ANOTHER NEARBY IS FOR ANYONE."));
        }
        if !plant.stock.is_empty() {
            return UseRes::Done;
        }
        if plant.harvest.is_some() {
            return UseRes::Wait("SOMEONE IS ALREADY HARVESTING.".into());
        }
        if !plant.ripe() {
            return UseRes::Wait("STILL GROWING.".into());
        }
        // Gathering takes its tools in hand.
        if let Err(missing) = self.held_tools(i, self.data.good(&plant.good)) {
            return UseRes::Wait(missing);
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
    fn dispense_plant(&mut self, p: usize) {
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
        let (px, py, owner) = (plant.x, plant.y, plant.owner);
        let slide = Some(Slide { dir, left: crate::game::PUSH_TICKS, total: crate::game::PUSH_TICKS });
        self.world.items.push(Item { slide, hop: 6, owner, ..Item::new(x, y, data) });
        self.world.burst(px, py, PKind::Leaf, 4);
        self.sound((x, y), Sfx::Pop);
    }

    pub(crate) fn use_shop(&mut self, i: usize, s: usize, sfx: &mut Vec<Sfx>) -> UseRes {
        if self.worn_out(s) {
            return UseRes::Wait(self.worn_note(s));
        }
        let shop = &self.world.shops[s];
        if let Some(name) = shop.owner.filter(|&o| o != self.monsters[i].id).and_then(|o| self.monsters.iter().find(|m| m.id == o)).map(|m| m.name.clone()) {
            return UseRes::Wait(format!("THAT'S {name}'S OWN BENCH. ANOTHER NEARBY IS FOR ANYONE."));
        }
        if shop.job.is_some() {
            return UseRes::Wait(format!("THE {} IS BUSY.", shop.kind.label()));
        }
        let kind = shop.kind;
        // Extra output waits inside, so a loop that takes one per cycle never jams.
        if shop.pending.len() >= OUTPUT_BUFFER {
            return UseRes::Full(format!("THE {} IS FULL. CLEAR ITS OUTPUT MAT.", kind.label()));
        }
        let offers = self.offers(s);
        if offers.is_empty() {
            return UseRes::Wait("THE TRADER HAS NOTHING YOU CAN'T MAKE HERE.".into());
        }
        // A loop leaves what another monster has reserved on the pads alone.
        let ok = |it: usize| self.may_take(i, it);
        let Some((recipe, inputs)) = self.best_recipe(s, &offers, &ok) else {
            let note = self.pads_note(s, &ok);
            // Nothing on the pads yet: say there's more it makes, and where to see it.
            let empty = self.world.shops[s].pads.iter().all(|&(x, y)| self.world.item_at(x, y).is_none());
            let more = offers.len().saturating_sub(1);
            return UseRes::Missing(if empty && more > 0 { format!("{note} IT MAKES {more} MORE: HOLD E, THEN TAB TWICE.") } else { note });
        };
        if let Err(missing) = self.match_tools(s, &recipe) {
            return UseRes::Wait(format!("FOR {}: {missing} ON A PEG.", self.good_name(&recipe)));
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

    /// What workshop `s`'s pads lack, said so it's clear what to do and what for: the recipe
    /// they come closest to and the first input it still needs ("FOR INDIGO: PUT ANOTHER LIME ON
    /// A PAD." when indigo has one of its two), or, with every pad taken, what to swap for it
    /// ("FOR INDIGO: SWAP THE LYE FOR ANOTHER LIME."). Goods that fit nothing it makes are said
    /// to first ("NOTHING HERE TAKES COTTON BOLLS WITH A ROPE."). Inputs are named as types of
    /// things. Only pad items `ok` accepts count.
    pub(crate) fn pads_note(&self, s: usize, ok: &dyn Fn(usize) -> bool) -> String {
        let shop = &self.world.shops[s];
        let on_pads: Vec<usize> = shop.pads.iter().filter_map(|&(x, y)| self.world.item_at(x, y)).filter(|&it| ok(it)).collect();
        let taken = shop.pads.iter().filter(|&&(x, y)| self.world.item_at(x, y).is_some()).count();
        let Aim { recipe, used, missing } = self.aim(s, ok);
        let Some(group) = missing.first() else { return String::new() };
        let another = used.iter().any(|&it| self.world.item_matches(&self.data, it, group));
        let name = self.describe_type(group);
        let want = if another { format!("ANOTHER {name}") } else { crate::recipes::one(&name) };
        let aim = format!("FOR {}:", self.good_name(&recipe));
        let lead = if used.is_empty() && !on_pads.is_empty() {
            let there: Vec<String> = on_pads.iter().map(|&it| crate::recipes::one(&self.good_name(self.world.items[it].good()))).collect();
            format!("NOTHING HERE TAKES {}. ", there.join(" WITH "))
        } else {
            String::new()
        };
        match on_pads.iter().find(|it| !used.contains(it)) {
            Some(&spare) if taken >= shop.pads.len() => format!("{lead}{aim} SWAP THE {} FOR {want}.", self.good_name(self.world.items[spare].good())),
            _ => format!("{lead}{aim} PUT {want} ON A PAD."),
        }
    }

    pub(crate) fn preview(&self, s: usize) -> Preview {
        let Aim { recipe, used, missing } = self.aim(s, &|_| true);
        let inputs = if missing.is_empty() { Ok(used.iter().map(|&i| self.world.items[i].data.clone()).collect()) } else { Err(self.pads_note(s, &|_| true)) };
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

    /// A good just landed on one of workshop `s`'s pads: if they're working toward something
    /// now, the workshop stays set up for it once they're cleared. (Every good reaches a pad by
    /// a push or a pull, loops' included.)
    pub(crate) fn remember_aim(&mut self, s: usize) {
        let aim = self.aim(s, &|_| true);
        if !aim.used.is_empty() {
            self.world.shops[s].aimed = Some(aim.recipe);
        }
    }

    /// What workshop `s`'s pads are working toward, the one answer its card, its mats and its
    /// monsters' messages all give: the recipe they make right now, if any (`best_recipe`);
    /// else the one that uses the most of what's on them, with the fewest inputs still missing;
    /// with nothing of use on them, the one it rests on (`Shop::resting`: what they last worked
    /// toward). Ties go to the resting recipe, then the earlier one. Only pad items `ok`
    /// accepts count.
    pub(crate) fn aim(&self, s: usize, ok: &dyn Fn(usize) -> bool) -> Aim {
        let shop = &self.world.shops[s];
        let offers = self.offers(s);
        if let Some((recipe, used)) = self.best_recipe(s, &offers, ok) {
            return Aim { recipe, used, missing: Vec::new() };
        }
        let on_pads: Vec<usize> = shop.pads.iter().filter_map(|&(x, y)| self.world.item_at(x, y)).filter(|&it| ok(it)).collect();
        let resting = shop.resting(&self.data);
        if offers.is_empty() {
            // A trader with nothing left to offer.
            return Aim { recipe: resting.to_string(), used: Vec::new(), missing: self.data.good(resting).input_groups().map(|(_, g)| g.clone()).collect() };
        }
        let closest = offers.iter().enumerate().map(|(n, &r)| {
            let (mut used, mut missing) = (Vec::new(), Vec::new());
            for (_, group) in self.data.good(r).input_groups() {
                match on_pads.iter().copied().find(|&it| !used.contains(&it) && self.world.item_matches(&self.data, it, group)) {
                    Some(it) => used.push(it),
                    None => missing.push(group.clone()),
                }
            }
            // Using nothing, every recipe is as far off: the resting one.
            let short = if used.is_empty() { 0 } else { missing.len() };
            ((std::cmp::Reverse(used.len()), short, r != resting, n), r, used, missing)
        });
        let (_, recipe, used, missing) = closest.min_by_key(|(key, ..)| *key).expect("a workshop makes something");
        Aim { recipe: recipe.to_string(), used, missing }
    }

    /// What workshop `s` makes now: its kind's recipes. But a trader's stall offers only what
    /// nothing else in the world makes or grows, so each trade leaves by itself once something
    /// in play makes its good.
    pub(crate) fn offers(&self, s: usize) -> Vec<&'static str> {
        let kind = self.world.shops[s].kind;
        if !kind.trades() {
            return kind.recipes().to_vec();
        }
        // What grows or is made in a province still closed off doesn't count yet.
        let open = |(x, y): (i32, i32)| !self.world.closed_at(x, y);
        let made_here = |good: &str| self.world.plants.iter().any(|p| open((p.x, p.y)) && p.good == good) || self.world.shops.iter().any(|o| open(o.spot) && !o.kind.trades() && o.kind.recipes().contains(&good));
        kind.recipes().iter().copied().filter(|r| !made_here(self.data.good(r).output())).collect()
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
                return Err(crate::recipes::one(&self.describe_type(group)));
            }
        }
        Ok(out)
    }

    pub(crate) fn tick_work(&mut self) {
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
            self.dispense_plant(p);
        }
        for s in 0..self.world.shops.len() {
            let shop = &mut self.world.shops[s];
            let (mut done, mut fluff, mut spin) = (false, None, false);
            if let Some(j) = shop.job.as_mut() {
                j.t += 1;
                done = j.t >= j.total;
                if j.t % 3 == 0 && !shop.pads.is_empty() {
                    fluff = Some(shop.pads[(j.t as usize / 3) % shop.pads.len()]);
                }
                spin = j.t % 6 == 0;
            }
            if let Some((px, py)) = fluff {
                self.world.burst(px, py, PKind::Fluff, 1);
            }
            if spin {
                self.sound(self.world.shops[s].spot, Sfx::Spin);
            }
            if done {
                self.finish_shop(s);
            }
            self.place_output(s);
            for f in self.world.shops[s].pad_flash.iter_mut() {
                *f = f.saturating_sub(1);
            }
        }
        self.tick_bins();
    }

    fn finish_harvest(&mut self, p: usize) {
        let Some(job) = self.world.plants[p].harvest.take() else { return };
        let m = job.monster;
        let recipe = self.data.good(&job.recipe).clone();
        let (stats, skills, id) = (self.monsters[m].stats.clone(), self.monsters[m].skills.clone(), self.monsters[m].id);
        // The tools in hand count as tools on pegs do: in the quality roll, and their makers'
        // shares.
        let hands = self.held_tools(m, &recipe).unwrap_or_default();
        let tools: Vec<ItemData> = hands.iter().filter_map(|&h| self.monsters[m].hands[h].clone()).collect();
        let items = econ::gather(&recipe, Crafter { id, stats: &stats, skills: &skills }, tools.iter().collect(), &self.data, &mut self.rng);
        // And they wear by their weight, and may break: but a working townsperson's own tools
        // never wear, as a town's workshops never do.
        let worn = if self.monsters[m].worker.is_some() { &[][..] } else { &hands[..] };
        for (k, &h) in worn.iter().enumerate() {
            let Some(tool) = self.monsters[m].hands[h].as_mut() else { continue };
            tool.wear += recipe.tool_weight(k);
            if tool.durability_left() == 0 {
                let good = tool.good.clone();
                let name = self.good_name(&good);
                self.monsters[m].hands[h] = None;
                self.say(m, &format!("MY {name} BROKE."), true);
            }
        }
        let secs = econ::production_seconds(&recipe, &stats);
        let learned = econ::learn(&mut self.monsters[m].skills, &stats, &recipe, &self.data, secs);
        self.monsters[m].learned = Some((recipe.id.clone(), learned, LEARNED_POPUP_TICKS));
        self.monsters[m].busy = false;
        let plant = &mut self.world.plants[p];
        plant.stock = items;
        plant.growth = 0;
        plant.timer = DISPENSE_TICKS; // the first one comes out straight away
        self.event(m, Ev::Made(recipe.id.clone()));
    }

    fn finish_shop(&mut self, s: usize) {
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
        // A trade teaches nothing: the trader does the work.
        let (produced, learned) = if recipe.is_trade() {
            (vec![econ::trade(&recipe, &inputs.iter().collect::<Vec<_>>(), &self.data)], None)
        } else {
            let craft = Craft { recipe: &recipe, crafter: Crafter { id, stats: &stats, skills: &skills }, inputs: inputs.iter().collect(), tools: tools.iter().collect(), workshop: workshop.as_ref() };
            let produced = econ::produce(&craft, &self.data, &mut self.rng);
            let secs = econ::production_seconds(&recipe, &stats);
            (produced, Some(econ::learn(&mut self.monsters[m].skills, &stats, &recipe, &self.data, secs)))
        };

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
        if let Some(learned) = learned {
            self.monsters[m].learned = Some((recipe.id.clone(), learned, LEARNED_POPUP_TICKS));
        }
        self.event(m, Ev::Made(recipe.id.clone()));
        self.sound((sx, sy), Sfx::Chime);
        let k = self.monsters[m].commune;
        if broke {
            self.say(m, "A TOOL WORE OUT!", true);
        } else if !self.communes[k].examine_hinted && self.controller(m) == Some(k) {
            self.communes[k].examine_hinted = true;
            self.say(m, "DONE! HOLD E TO EXAMINE THINGS.", false);
        }
    }

    /// Finished goods appear on a free output mat, one at a time.
    fn place_output(&mut self, s: usize) {
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
        // What a working townsperson's bench makes is theirs.
        let owner = shop.owner;
        self.world.items.push(Item { hop: 5, owner, ..Item::new(x, y, data) });
        self.world.burst(x, y, PKind::Spark, 10);
        self.sound((x, y), Sfx::Plop);
    }

    /// Bins with a chute push their oldest item out whenever the chute is clear.
    fn tick_bins(&mut self) {
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
            self.sound((mx, my), Sfx::Plop);
        }
    }
}
