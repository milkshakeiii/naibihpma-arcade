//! The hold-E inspect card: a few lines about whatever the controlled monster faces, or the
//! workshop forecast when it stands at a work spot. It never pauses the world.

use crate::data::matches;
use crate::econ::{self, chance_at_least, tier_name, Craft, Crafter, Forecast, ItemData, EXCEPTIONAL, MASTERWORK};
use crate::game::Game;
use crate::palette::*;
use crate::world::{Tile, TS};

pub struct Line {
    pub text: String,
    pub color: u8,
    /// A 0..1 bar drawn at the right end of the line (skill levels).
    pub bar: Option<f64>,
}

pub struct Card {
    pub title: String,
    pub lines: Vec<Line>,
    /// The tile the card points at.
    pub at: (i32, i32),
}

fn line(text: impl Into<String>, color: u8) -> Line {
    Line { text: text.into(), color, bar: None }
}

const ABILITIES: [&str; 6] = ["STR", "DEX", "CON", "INT", "WIS", "CHA"];

fn skill_name(id: &str) -> String {
    id.replace('_', " ").to_uppercase()
}

/// Skills read in points out of 1000, like a batting average: 0.312 shows as 312. Gains
/// under one point keep a decimal so they don't read as nothing.
fn points(v: f64) -> String {
    let p = v * 1000.0;
    if (0.05..0.95).contains(&p) {
        format!("{p:.1}")
    } else {
        format!("{p:.0}")
    }
}

fn quality_color(q: f64) -> u8 {
    if q >= EXCEPTIONAL {
        SPARK + 2
    } else if q < 0.25 {
        TEXT_DIM
    } else {
        TEXT
    }
}

fn odds(p: f64) -> String {
    if p >= 0.5 {
        "USUALLY".into()
    } else if p < 1e-4 {
        "ALMOST NEVER".into()
    } else {
        format!("1 IN {}", (1.0 / p).round() as u64)
    }
}

/// "COTTON THREAD": the good's name led by the varieties it carries.
pub fn display_name(g: &Game, item: &ItemData) -> String {
    let name = g.good_name(&item.good);
    let mut words: Vec<String> = item.carried_tags.iter().map(|t| t.to_uppercase()).filter(|t| !name.contains(t.as_str())).collect();
    words.push(name);
    words.join(" ")
}

pub fn card(g: &Game) -> Card {
    let c = g.controlled;
    let m = &g.monsters[c];
    let (dx, dy) = m.facing.delta();
    let (fx, fy) = (m.x + dx, m.y + dy);
    if let Some(s) = g.world.shop_at_spot(m.x, m.y).or_else(|| g.world.shop_at_spot(fx, fy)) {
        return forecast_card(g, s, c);
    }
    if let Some(k) = g.faced_applicant() {
        return applicant_card(g, k);
    }
    if let Some(i) = g.world.item_at(fx, fy) {
        return item_card(g, &g.world.items[i].data, (fx, fy));
    }
    if g.world.tile(fx, fy) == Tile::Peg {
        if let Some(card) = town_tool_card(g, (fx, fy)) {
            return card;
        }
    }
    if let Some(o) = g.monster_at(fx, fy) {
        return monster_card(g, o);
    }
    if let Some(p) = g.world.plant_at(fx, fy).or_else(|| g.world.plant_at(m.x, m.y)) {
        return plant_card(g, p);
    }
    if let Some(b) = g.world.bin_at(fx, fy) {
        return bin_card(g, b);
    }
    monster_card(g, c)
}

fn is_tool(g: &Game, item: &ItemData) -> bool {
    let tags = item.tags(&g.data);
    g.data.goods.iter().any(|r| r.tools_required_tags.iter().any(|grp| !grp.is_empty() && matches(&tags, grp)))
}

fn item_card(g: &Game, item: &ItemData, at: (i32, i32)) -> Card {
    let q = item.quality;
    let gt = g.data.good(&item.good);
    let mut lines = Vec::new();
    if gt.has_quality {
        lines.push(line(format!("QUALITY {q:.2} ({})", tier_name(q)), quality_color(q)));
    }
    lines.push(line(format!("VALUE {:.1}  WEIGHT {:.1}", item.value, econ::weight(gt, item)), TEXT));
    lines.push(if item.depth == 0 {
        line("RAW MATERIAL", TEXT_DIM)
    } else {
        let n = item.raw.len();
        line(format!("{} STEPS FROM {n} RAW MATERIAL{}", item.depth, if n == 1 { "" } else { "S" }), TEXT_DIM)
    });
    if is_tool(g, item) {
        lines.push(line(format!("DURABILITY {}/{}", item.durability_left(), econ::TOOL_DURABILITY), TEXT));
    }
    if !item.shares.is_empty() {
        let credit: Vec<String> = item
            .shares
            .iter()
            .map(|s| format!("{} {:.1}", g.monsters.iter().find(|m| m.id == s.monster).map_or("?", |m| m.name.as_str()), s.count))
            .collect();
        lines.push(line(format!("CREDIT: {}", credit.join(", ")), TEXT_DIM));
    }
    Card { title: display_name(g, item), lines, at }
}

fn applicant_card(g: &Game, k: usize) -> Card {
    let a = g.applicants[k].as_ref().expect("a faced applicant");
    let s = &a.stats;
    let affordable = g.renown >= a.cost as f64;
    let lines = vec![
        line(format!("{} RENOWN TO HIRE. YOU HAVE {}.", a.cost, g.renown.floor()), if affordable { TEXT_GOOD } else { TEXT_WARN }),
        line(format!("STR {} DEX {} CON {} INT {} WIS {} CHA {}", s.str, s.dex, s.con, s.int, s.wis, s.cha), TEXT),
        line(format!("TALENTS: {}", a.transferable.iter().map(|t| skill_name(t)).collect::<Vec<_>>().join(", ")), TEXT_DIM),
        line(if affordable { "SPACE HIRES THEM." } else { "EARN MORE RENOWN TO HIRE THEM." }, TEXT_DIM),
    ];
    Card { title: format!("{} THE {}", a.name, a.kind.to_uppercase()), lines, at: g.world.stands[k] }
}

fn town_tool_card(g: &Game, at: (i32, i32)) -> Option<Card> {
    let shop = g.world.shops.iter().find(|s| s.pegs.contains(&at))?;
    let k = shop.pegs.iter().position(|&p| p == at)?;
    let id = shop.kind.builtin_tools().get(k)?;
    let q = crate::world::BUILTIN_TOOL_QUALITY;
    Some(Card {
        title: format!("TOWN {}", g.good_name(id)),
        lines: vec![
            line(format!("QUALITY {q:.2} ({})", tier_name(q)), quality_color(q)),
            line("THE TOWN'S OWN TOOL. IT NEVER WEARS OUT.", TEXT_DIM),
            line("PUSH A BETTER ONE ONTO THIS PEG.", TEXT_DIM),
        ],
        at,
    })
}

fn monster_card(g: &Game, i: usize) -> Card {
    let m = &g.monsters[i];
    let s = &m.stats;
    let mut lines = vec![
        line(format!("STR {} DEX {} CON {} INT {} WIS {} CHA {}", s.str, s.dex, s.con, s.int, s.wis, s.cha), TEXT),
        line(format!("TALENTS: {}", m.skills.transferable.iter().map(|t| skill_name(t)).collect::<Vec<_>>().join(", ")), TEXT_DIM),
    ];
    let mut skills: Vec<(String, f64)> = m.skills.applied.keys().map(|k| (k.clone(), m.skills.applied(k))).filter(|(_, v)| *v > 0.0).collect();
    skills.sort_by(|a, b| b.1.total_cmp(&a.1));
    if skills.is_empty() {
        lines.push(line("NO SKILLS YET", TEXT_DIM));
    }
    for (k, v) in skills.into_iter().take(4) {
        lines.push(Line { text: format!("{:<13}{}", skill_name(&k), points(v)), color: TEXT, bar: Some(v) });
    }
    if m.earned > 0.0 {
        lines.push(line(format!("EARNED {} RENOWN SO FAR", m.earned.floor()), TEXT));
    }
    let (status, warn) = g.status(i);
    let state = status.split_once(": ").map_or(status.clone(), |(_, s)| s.to_string());
    lines.push(line(state, if warn { TEXT_WARN } else { TEXT_GOOD }));
    Card { title: format!("{} THE {}", m.name, m.kind.to_uppercase()), lines, at: (m.x, m.y) }
}

fn plant_card(g: &Game, p: usize) -> Card {
    let plant = &g.world.plants[p];
    let name = g.good_name(&plant.good);
    let state = if plant.harvest.is_some() {
        line("BEING HARVESTED", TEXT_GOOD)
    } else if !plant.stock.is_empty() {
        line(format!("{} MORE {name} COMING OUT", plant.stock.len()), TEXT_GOOD)
    } else if plant.ripe() {
        line("RIPE. STAND ON IT AND PRESS SPACE.", TEXT_GOOD)
    } else {
        line(format!("GROWING: {}%", plant.growth * 100 / plant.regrow.max(1)), TEXT_DIM)
    };
    let mut lines = vec![state];
    // What a harvest would give the monster you control: gathering is a craft with no
    // inputs or tools.
    let c = g.controlled;
    let m = &g.monsters[c];
    let recipe = g.data.good(&plant.good);
    let craft = Craft { recipe, crafter: Crafter { id: m.id, stats: &m.stats, skills: &m.skills }, inputs: Vec::new(), tools: Vec::new(), workshop: None };
    if recipe.has_quality {
        let f = econ::forecast(&craft);
        let exp = f.expected(&m.stats);
        lines.push(line(format!("EXPECTED QUALITY {exp:.2} ({})", tier_name(exp)), quality_color(exp)));
    }
    let (lo, hi, avg) = econ::quantity_range(&craft);
    lines.push(if lo == hi {
        line(format!("A HARVEST GIVES {lo} {name}"), TEXT)
    } else {
        line(format!("A HARVEST GIVES {lo} TO {hi} {name}, ABOUT {avg:.1}"), TEXT)
    });
    lines.extend(time_and_learning(g, recipe, c, "HARVESTS"));
    Card { title: "COTTON PLANT".into(), lines, at: (plant.x, plant.y) }
}

fn bin_card(g: &Game, b: usize) -> Card {
    let bin = &g.world.bins[b];
    let name = g.good_name(&bin.accepts);
    let mut lines = vec![line(format!("{} {name}", bin.stock.len()), TEXT)];
    if !bin.stock.is_empty() && g.data.good(&bin.accepts).has_quality {
        let avg = bin.stock.iter().map(|i| i.quality).sum::<f64>() / bin.stock.len() as f64;
        lines.push(line(format!("AVERAGE QUALITY {avg:.2} ({})", tier_name(avg)), quality_color(avg)));
    }
    if bin.mouth.is_some() {
        lines.push(line("ITS CHUTE PUSHES THEM OUT ONE AT A TIME.", TEXT_DIM));
    }
    Card { title: format!("{name} BIN"), lines, at: (bin.x, bin.y) }
}

/// The forecast for the monster `i` working at shop `s` with what is there now.
pub fn forecast(g: &Game, s: usize, i: usize) -> Option<Forecast> {
    let pv = g.preview(s);
    let (Ok(inputs), Ok(tools)) = (&pv.inputs, &pv.tools) else { return None };
    let m = &g.monsters[i];
    let craft = Craft {
        recipe: g.data.good(&pv.recipe),
        crafter: Crafter { id: m.id, stats: &m.stats, skills: &m.skills },
        inputs: inputs.iter().collect(),
        tools: tools.iter().collect(),
        workshop: pv.workshop.as_ref(),
    };
    Some(econ::forecast(&craft))
}

/// The HUD's one-line preview while a monster stands at a work spot.
pub fn spot_preview(g: &Game, i: usize) -> Option<String> {
    let m = &g.monsters[i];
    let s = g.world.shop_at_spot(m.x, m.y)?;
    if !g.data.good(&g.preview(s).recipe).has_quality {
        return None;
    }
    let f = forecast(g, s, i)?;
    Some(format!("{} EXPECTS QUALITY {:.2}", g.world.shops[s].kind.label(), f.expected(&m.stats)))
}

fn forecast_card(g: &Game, s: usize, i: usize) -> Card {
    let pv = g.preview(s);
    let shop = &g.world.shops[s];
    let recipe = g.data.good(&pv.recipe);
    let m = &g.monsters[i];
    let mut lines = Vec::new();
    if let Some(j) = &shop.job {
        lines.push(line(format!("WORKING: {}%", j.t * 100 / j.total.max(1)), TEXT_GOOD));
    }
    if !shop.pending.is_empty() {
        lines.push(line(format!("{} MORE WAITING INSIDE", shop.pending.len()), TEXT));
    }
    match (&pv.inputs, &pv.tools, forecast(g, s, i)) {
        (Err(need), _, _) => lines.push(line(format!("NEEDS {need} ON THE PAD."), TEXT_WARN)),
        (_, Err(tool), _) => lines.push(line(format!("MISSING A TOOL: {tool}."), TEXT_WARN)),
        (_, _, Some(f)) => {
            let exp = f.expected(&m.stats);
            if recipe.has_quality {
                lines.push(line(format!("EXPECTED QUALITY {exp:.2} ({})", tier_name(exp)), quality_color(exp)));
                let lo = econ::nudge((f.mean - 2.0 * f.sigma).max(0.0), &m.stats);
                lines.push(line(format!("USUALLY {lo:.2} TO {:.2}", f.mean + 2.0 * f.sigma), TEXT_DIM));
                let (exc, mw) = (chance_at_least(&f, EXCEPTIONAL), chance_at_least(&f, MASTERWORK));
                lines.push(if exc < 1e-4 {
                    line("NO EXCEPTIONAL WORK YET", TEXT_DIM)
                } else {
                    line(format!("EXCEPTIONAL {}, MASTERWORK {}", odds(exc), odds(mw)), TEXT_DIM)
                });
            }
            if let (Ok(inputs), Ok(tools)) = (&pv.inputs, &pv.tools) {
                let craft = Craft { recipe, crafter: Crafter { id: m.id, stats: &m.stats, skills: &m.skills }, inputs: inputs.iter().collect(), tools: tools.iter().collect(), workshop: pv.workshop.as_ref() };
                let (lo, hi, avg) = econ::quantity_range(&craft);
                let name = recipe.name.to_uppercase();
                lines.push(if lo == hi { line(format!("MAKES {lo} {name}"), TEXT) } else { line(format!("MAKES {lo} TO {hi} {name}, ABOUT {avg:.1}"), TEXT) });
            }
            if recipe.has_quality {
                lines.extend(held_back(g, &f, recipe, i));
            }
        }
        _ => {}
    }
    lines.extend(time_and_learning(g, recipe, i, "CRAFTS"));
    Card { title: format!("{}: {}", shop.kind.label(), recipe.name.to_uppercase()), lines, at: shop.spot }
}

/// How long a job takes this monster and what it teaches: a dry run on a copy of its skills.
fn time_and_learning(g: &Game, recipe: &crate::data::GoodType, i: usize, jobs: &str) -> Vec<Line> {
    let m = &g.monsters[i];
    let secs = g.job_ticks(&recipe.id, i) as f64 / 15.0;
    let mut lines = vec![line(format!("TAKES {secs:.1} SECONDS FOR {}", m.name), TEXT_DIM)];
    let mut sk = m.skills.clone();
    let job_secs = econ::production_seconds(recipe, &m.stats);
    let l = econ::learn(&mut sk, &m.stats, recipe, &g.data, job_secs);
    let primary = &recipe.primary_applied_skill;
    lines.push(line(format!("LEARNS {} +{}, PRACTICE +{}", skill_name(primary), points(l.primary), points(l.specific)), TEXT_GOOD));
    if !l.secondary.is_empty() {
        let also: Vec<String> = l.secondary.iter().map(|(k, v)| format!("{} +{}", skill_name(k), points(*v))).collect();
        lines.push(line(format!("ALSO {}", also.join(", ")), TEXT_GOOD));
    }
    lines.push(line(format!("FORGETS {} OF EVERY OTHER SKILL", points(l.forgotten)), TEXT_DIM));
    let (level, n) = level_off(g, i, recipe, job_secs);
    lines.push(line(format!("{} LEVELS OFF NEAR {} IN ABOUT {n} {jobs}", skill_name(primary), points(level)), TEXT_DIM));
    lines
}

/// Where the primary skill settles if the monster keeps doing this, and how soon.
fn level_off(g: &Game, i: usize, recipe: &crate::data::GoodType, secs: f64) -> (f64, u32) {
    let m = &g.monsters[i];
    let mut sk = m.skills.clone();
    let skill = &recipe.primary_applied_skill;
    for n in 0..3000 {
        let before = sk.applied(skill);
        econ::learn(&mut sk, &m.stats, recipe, &g.data, secs);
        if sk.applied(skill) - before < 0.0005 {
            return (sk.applied(skill), n);
        }
    }
    (sk.applied(skill), 3000)
}

/// The weakest part of the roll, and what would help.
fn held_back(g: &Game, f: &Forecast, recipe: &crate::data::GoodType, i: usize) -> Vec<Line> {
    let m = &g.monsters[i];
    let primary = skill_name(&recipe.primary_applied_skill);
    let ability = ABILITIES.get(recipe.relevant_ability_score).copied().unwrap_or("STR");
    // Each factor is compared on its 0..1 strength; the shown amount is in its own unit:
    // quality as a decimal, skills in points, the ability as its score.
    let factors = [
        ("INPUT QUALITY".to_string(), f.input_quality, format!("{:.2}", f.input_quality), "BETTER INPUTS MAKE BETTER GOODS.".to_string()),
        (format!("{primary} SKILL"), f.primary, points(f.primary), "IT GROWS WITH EVERY CRAFT.".to_string()),
        ("SECONDARY SKILLS".to_string(), f.secondary, points(f.secondary), {
            let missing: Vec<String> = recipe.transferable_skills.iter().filter(|t| !m.skills.transferable.contains(t)).map(|t| skill_name(t)).collect();
            if missing.is_empty() { "THEY GROW SLOWLY WITH PRACTICE.".into() } else { format!("A TALENT FOR {} SKIPS THE WEAKEST.", missing.join(" OR ")) }
        }),
        ("TOOL QUALITY".to_string(), f.tool_quality, format!("{:.2}", f.tool_quality), "A BETTER TOOL ON THE PEG WOULD HELP.".to_string()),
        ("PRACTICE".to_string(), f.specific, points(f.specific), if f.primary < 0.1 { format!("IT GROWS ONCE {primary} DOES.") } else { "IT GROWS WITH EVERY CRAFT.".to_string() }),
        (ability.to_string(), f.ability / econ::ABILITY_FACTOR_CAP, m.stats.get(recipe.relevant_ability_score).to_string(), format!("A MONSTER WITH MORE {ability} WOULD DO BETTER.")),
    ];
    let Some((name, v, shown, hint)) = factors.into_iter().min_by(|a, b| a.1.total_cmp(&b.1)) else { return Vec::new() };
    if v >= 0.95 {
        return vec![line("NOTHING HOLDS IT BACK MUCH.", TEXT_GOOD)];
    }
    vec![line(format!("HELD BACK BY {name} ({shown})"), TEXT_WARN), line(hint, TEXT_DIM)]
}

/// Which side of its target a card goes on, and how wide it may be there: the right if
/// the card fits, else the left, else the roomier side (the card then wraps narrower).
/// A card always sits beside its target, so it never covers it.
pub fn side(at_px: (i32, i32), wanted_w: i32) -> (bool, i32) {
    let right_room = 398 - (at_px.0 + TS + 6);
    let left_room = at_px.0 - 6 - 2;
    if wanted_w <= right_room {
        (true, right_room)
    } else if wanted_w <= left_room {
        (false, left_room)
    } else if right_room >= left_room {
        (true, right_room)
    } else {
        (false, left_room)
    }
}

pub fn place(at_px: (i32, i32), w: i32, h: i32, view_h: i32, right: bool) -> (i32, i32) {
    let (ax, ay) = at_px;
    let x = if right { ax + TS + 6 } else { ax - 6 - w };
    let y = (ay + TS / 2 - h / 2).clamp(2, (view_h - h - 2).max(2));
    (x, y)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn skills_read_in_points_out_of_a_thousand() {
        assert_eq!(points(0.312), "312");
        assert_eq!(points(0.019), "19");
        assert_eq!(points(0.0004), "0.4", "a gain under one point still shows");
        assert_eq!(points(0.00099), "1");
        assert_eq!(points(0.0), "0");
        assert_eq!(points(0.00001), "0");
    }

    #[test]
    fn the_card_never_covers_its_target() {
        for (wanted, h) in [(120, 60), (230, 120), (300, 164)] {
            for ty in 0..16 {
                for tx in 0..25 {
                    let at = (tx * TS, ty * TS);
                    let (right, room) = side(at, wanted);
                    let w = wanted.min(room);
                    let (x, y) = place(at, w, h, 256, right);
                    let overlap = x < at.0 + TS && at.0 < x + w && y < at.1 + TS && at.1 < y + h;
                    assert!(!overlap, "a {w}x{h} card covers tile ({tx},{ty}) at ({x},{y})");
                    assert!(x >= 2 && x + w <= 398, "a {w}-wide card at x={x} leaves the screen for tile ({tx},{ty})");
                }
            }
        }
    }
}
