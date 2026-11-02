//! The hold-E inspect card: whatever the controlled monster faces, or the workshop forecast
//! when it stands at a work spot. It never pauses the world. The first page holds the few
//! things that decide what to do next, led by icons; Tab while examining shows the details.

use crate::data::{matches, GoodType};
use crate::econ::{self, chance_at_least, tier_name, Craft, Crafter, Forecast, ItemData, EXCEPTIONAL, MASTERWORK};
use crate::game::{Game, RENOWN_PER_VALUE};
use crate::palette::*;
use crate::world::{Dir, Project, Tile, TS};

/// Small pictures that lead a line. Each means the same thing on every card.
#[derive(Clone, Debug, PartialEq)]
pub enum Icon {
    Quality,
    Time,
    Warn,
    Learn,
    Forget,
    Renown,
    Weight,
    Wear,
    /// A good's own picture: what comes out.
    Good(String),
}

/// Where a quality sits on the 0..2 scale: the usual range and the expected (or actual) value.
#[derive(Clone, Copy, Debug)]
pub struct Meter {
    pub lo: f64,
    pub mid: f64,
    pub hi: f64,
}

pub struct Line {
    pub text: String,
    pub color: u8,
    pub icon: Option<Icon>,
    /// A second icon and text on the same line: "[time] 5.5S  [rope] 1 ROPE".
    pub then: Option<(Icon, String)>,
    /// A 0..1 bar at the right end of the line (skill levels, durability).
    pub bar: Option<f64>,
    /// A quality meter drawn in place of text.
    pub meter: Option<Meter>,
}

impl Line {
    fn icon(mut self, icon: Icon) -> Self {
        self.icon = Some(icon);
        self
    }

    fn then(mut self, icon: Icon, text: impl Into<String>) -> Self {
        self.then = Some((icon, text.into()));
        self
    }

    fn bar(mut self, v: f64) -> Self {
        self.bar = Some(v);
        self
    }
}

pub struct Card {
    pub title: String,
    /// The first page is what matters most; the rest is details.
    pub pages: Vec<Vec<Line>>,
    /// The tile the card points at.
    pub at: (i32, i32),
}

impl Card {
    fn one(title: impl Into<String>, lines: Vec<Line>, at: (i32, i32)) -> Self {
        Card { title: title.into(), pages: vec![lines], at }
    }

    /// A first page and a details page; if either is empty, one page.
    fn two(title: impl Into<String>, first: Vec<Line>, details: Vec<Line>, at: (i32, i32)) -> Self {
        let pages = match (first.is_empty(), details.is_empty()) {
            (true, _) => vec![details],
            (_, true) => vec![first],
            _ => vec![first, details],
        };
        Card { title: title.into(), pages, at }
    }

    /// Every line on every page.
    #[cfg(test)]
    pub fn lines(&self) -> impl Iterator<Item = &Line> {
        self.pages.iter().flatten()
    }
}

fn line(text: impl Into<String>, color: u8) -> Line {
    Line { text: text.into(), color, icon: None, then: None, bar: None, meter: None }
}

/// "[star] 0.84 GOOD" over a meter showing where it falls.
fn quality(lo: f64, mid: f64, hi: f64) -> [Line; 2] {
    [
        line(format!("{mid:.2} {}", tier_name(mid)), quality_color(mid)).icon(Icon::Quality),
        Line { meter: Some(Meter { lo, mid, hi }), ..line("", TEXT) },
    ]
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
        TEXT_WARN
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

/// "INDIGO COTTON DYED FABRIC": the good's name led by the varieties it carries, the last
/// one added first.
pub fn display_name(g: &Game, item: &ItemData) -> String {
    let name = g.good_name(&item.good);
    let mut words: Vec<String> = item.carried_tags.iter().rev().map(|t| t.to_uppercase()).filter(|t| !name.contains(t.as_str())).collect();
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
    if let Some(w) = g.world.pumpkin_at(fx, fy) {
        let leader = g.world.wagons[w].leader.and_then(|id| g.monsters.iter().find(|m| m.id == id)).map(|m| m.name.clone());
        let lines = vec![
            line("A MAGICAL PUMPKIN. IT FOLLOWS ANYONE CHARMING ENOUGH.", TEXT),
            match leader {
                Some(name) => line(format!("{name} IS LEADING IT."), TEXT_GOOD),
                None => line("SPACE: LEAD IT, AND ITS WAGON COMES ALONG.", TEXT_DIM),
            },
        ];
        return Card::one("PUMPKIN", lines, (fx, fy));
    }
    if let Some(w) = g.world.wagon_at(fx, fy) {
        return wagon_card(g, w, (fx, fy));
    }
    if let Some(p) = g.world.plant_at(fx, fy).or_else(|| g.world.plant_at(m.x, m.y)) {
        return plant_card(g, p);
    }
    if let Some(b) = g.world.bin_at(fx, fy) {
        return bin_card(g, b);
    }
    if let Some(b) = g.world.board_at(fx, fy) {
        return board_card(g, b);
    }
    if let Some(s) = g.world.sign_at(fx, fy) {
        let way = match s.dir {
            Dir::Left => "WEST",
            Dir::Right => "EAST",
            Dir::Up => "NORTH",
            Dir::Down => "SOUTH",
        };
        return Card::one("SIGNPOST", vec![line(format!("{} LIES {way}.", s.text), TEXT)], (fx, fy));
    }
    if g.world.footbridge.contains(&(fx, fy)) {
        return footbridge_card(g, (fx, fy));
    }
    if g.world.tile(fx, fy) == Tile::Tree {
        return Card::one("TREE", vec![line("AN OLD TREE. IT ISN'T FOR GATHERING.", TEXT), line("LIGHTER, YOUNGER TREES GIVE WOOD.", TEXT_DIM)], (fx, fy));
    }
    monster_card(g, c)
}

fn wagon_card(g: &Game, w: usize, at: (i32, i32)) -> Card {
    let wagon = &g.world.wagons[w];
    let mut first = vec![match wagon.good() {
        Some(good) => line(format!("{} {}", wagon.stock.len(), g.good_name(good)), TEXT).icon(Icon::Good(good.to_string())),
        None => line("EMPTY. PUSH GOODS INTO IT, ONE KIND AT A TIME.", TEXT_DIM),
    }];
    let used: f64 = wagon.stock.iter().map(|(d, _)| g.data.good(&d.good).storage_volume).sum();
    first.push(line(format!("ROOM {used:.1} OF {:.1}", wagon.room), TEXT).bar(used / wagon.room));
    first.push(if wagon.open { line("TAILGATE OPEN: PARKED, IT UNLOADS BEHIND.", TEXT_GOOD) } else { line("TAILGATE SHUT. SPACE AT ITS BACK OPENS IT.", TEXT_DIM) });
    let c = g.controlled;
    let details = vec![
        line(format!("LOAD WEIGHS {:.0}. {} PULLS {:.0} AT FULL SPEED.", g.load_weight(w), g.monsters[c].name, g.pull_capacity(c, w)), TEXT).icon(Icon::Weight),
        line("PULLING POWER: THE WAGON, ITS LEADER'S CHA AND WAGON DRIVING.", TEXT_DIM),
        line("THE TOWN'S WAGON, ON LOAN.", TEXT_DIM),
    ];
    Card::two("WAGON", first, details, at)
}

fn board_card(g: &Game, b: usize) -> Card {
    let board = &g.world.boards[b];
    let name = g.good_name(&board.wants);
    let lines = if board.done() {
        vec![line("DONE! THANK YOU, EVERYONE.", TEXT_GOOD)]
    } else {
        vec![
            line(format!("{} OF {} {name}", board.have, board.need), TEXT)
                .icon(Icon::Good(g.data.goods.iter().find(|p| p.id == board.wants || p.type_tags.contains(&board.wants)).map_or(board.wants.clone(), |p| p.id.clone())))
                .bar(board.have as f64 / board.need.max(1) as f64),
            line("EACH ONE EARNS RENOWN.", TEXT_DIM).icon(Icon::Renown),
        ]
    };
    Card::one(board.title.clone(), lines, (board.x, board.y))
}

fn footbridge_card(g: &Game, at: (i32, i32)) -> Card {
    let lines = if !g.world.footbridge_built() {
        let board = g.world.boards.iter().find(|b| b.project == Project::Footbridge);
        let left = board.map_or(0, |b| b.need - b.have.min(b.need));
        vec![line("NO ROPE ACROSS YET.", TEXT_WARN).icon(Icon::Warn), line(format!("{left} MORE ROPE FOR THE BOARD ON THE BANK."), TEXT_DIM).icon(Icon::Good("rope".into()))]
    } else {
        vec![line("MONSTERS AND GOODS CROSS IT.", TEXT_GOOD), line("WAGONS DON'T FIT: THEY CROSS AT THE BRIDGE.", TEXT_DIM)]
    };
    Card::one("THE FOOTBRIDGE", lines, at)
}

fn is_tool(g: &Game, item: &ItemData) -> bool {
    let tags = item.tags(&g.data);
    g.data.goods.iter().any(|r| r.tools_required_tags.iter().any(|grp| !grp.is_empty() && matches(&tags, grp)))
}

fn item_card(g: &Game, item: &ItemData, at: (i32, i32)) -> Card {
    let q = item.quality;
    let gt = g.data.good(&item.good);
    let mut first = Vec::new();
    if gt.has_quality {
        first.extend(quality(q, q, q));
    }
    // Renown only means something for goods a drop-off or a project takes.
    let wanted = g.world.bins.iter().any(|b| b.scores && b.takes(&item.good)) || (0..g.world.boards.len()).any(|b| !g.world.boards[b].done() && g.board_takes(b, &item.good));
    let mut details = Vec::new();
    if wanted {
        first.push(line(format!("WORTH {} RENOWN", (item.value * RENOWN_PER_VALUE).round()), TEXT).icon(Icon::Renown));
    } else {
        details.push(line(format!("VALUE {:.1}", item.value), TEXT_DIM));
    }
    // Why other monsters leave it alone.
    let id = g.world.item_at(at.0, at.1).map_or(0, |k| g.world.items[k].id);
    if let Some(m) = g.monsters.iter().find(|m| id != 0 && m.holds.contains(&crate::sched::Hold::Item(id))) {
        first.push(line(format!("{} IS SEEING TO IT", m.name), TEXT_DIM));
    }
    if is_tool(g, item) {
        let left = item.durability_left();
        first.push(line(format!("{left}/{} USES LEFT", econ::TOOL_DURABILITY), TEXT).icon(Icon::Wear).bar(left as f64 / econ::TOOL_DURABILITY as f64));
    }
    if let Some(kind) = crate::frontier::kit_kind(&item.good) {
        first.push(line(format!("FACE IT AND PRESS SPACE TO SET UP A {}. NOT IN TOWN OR ON A ROAD.", kind.label()), TEXT));
        let c = g.controlled;
        if let Some(why) = g.kit_ahead(c).filter(|&k| (g.world.items[k].x, g.world.items[k].y) == at).and_then(|k| g.footprint(c, k)).and_then(|fp| fp.why) {
            first.push(line(why, TEXT_WARN).icon(Icon::Warn));
        }
    }
    details.push(line(format!("WEIGHS {:.1}", econ::weight(gt, item)), TEXT).icon(Icon::Weight));
    details.push(if item.depth == 0 {
        line("RAW MATERIAL", TEXT_DIM)
    } else {
        let n = item.raw.len();
        line(format!("{} STEPS FROM {n} RAW MATERIAL{}", item.depth, if n == 1 { "" } else { "S" }), TEXT_DIM)
    });
    if !item.shares.is_empty() {
        let credit: Vec<String> = item
            .shares
            .iter()
            .map(|s| format!("{} {:.1}", g.monsters.iter().find(|m| m.id == s.monster).map_or("?", |m| m.name.as_str()), s.count))
            .collect();
        details.push(line(format!("CREDIT: {}", credit.join(", ")), TEXT_DIM));
    }
    if !item.haul.is_empty() {
        let name = |id: u32| g.monsters.iter().find(|m| m.id == id).map_or("?", |m| m.name.as_str());
        let moved: Vec<String> = item.haul.iter().map(|&(id, tiles)| format!("{} {tiles}", name(id))).collect();
        details.push(line(format!("HAULED (TILES): {}. PAID WHEN IT'S USED.", moved.join(", ")), TEXT_DIM));
    }
    Card::two(display_name(g, item), first, details, at)
}

fn applicant_card(g: &Game, k: usize) -> Card {
    let a = g.applicants[k].as_ref().expect("a faced applicant");
    let s = &a.stats;
    let affordable = g.renown >= a.cost as f64;
    let first = vec![
        line(format!("JOINS AT {} RENOWN. YOU HAVE {}.", a.cost, g.renown.floor()), if affordable { TEXT_GOOD } else { TEXT_WARN }).icon(Icon::Renown),
        line(format!("TALENTS: {}", a.transferable.iter().map(|t| skill_name(t)).collect::<Vec<_>>().join(", ")), TEXT),
        line(if affordable { "SPACE WELCOMES THEM." } else { "EARN MORE RENOWN TO WELCOME THEM." }, TEXT_DIM),
    ];
    let details = vec![line(format!("STR {} DEX {} CON {} INT {} WIS {} CHA {}", s.str, s.dex, s.con, s.int, s.wis, s.cha), TEXT)];
    Card::two(format!("{} THE {}", a.name, a.kind.to_uppercase()), first, details, g.world.stands[k])
}

fn town_tool_card(g: &Game, at: (i32, i32)) -> Option<Card> {
    let shop = g.world.shops.iter().find(|s| s.pegs.contains(&at))?;
    let k = shop.pegs.iter().position(|&p| p == at)?;
    let id = shop.kind.builtin_tools().get(k)?;
    let q = crate::world::BUILTIN_TOOL_QUALITY;
    let mut lines = Vec::from(quality(q, q, q));
    lines.push(line("THE TOWN'S OWN TOOL. IT NEVER WEARS OUT.", TEXT_DIM));
    lines.push(line("PUSH A BETTER ONE ONTO THIS PEG.", TEXT_DIM));
    Some(Card::one(format!("TOWN {}", g.good_name(id)), lines, at))
}

fn monster_card(g: &Game, i: usize) -> Card {
    let m = &g.monsters[i];
    let s = &m.stats;
    let (status, warn) = g.status(i);
    let state = status.split_once(": ").map_or(status.clone(), |(_, s)| s.to_string());
    let mut first = vec![if warn { line(state, TEXT_WARN).icon(Icon::Warn) } else { line(state, TEXT_GOOD) }];
    let mut skills: Vec<(String, f64)> = m.skills.applied.keys().map(|k| (k.clone(), m.skills.applied(k))).filter(|(_, v)| *v > 0.0).collect();
    skills.sort_by(|a, b| b.1.total_cmp(&a.1));
    if skills.is_empty() {
        first.push(line("NO SKILLS YET", TEXT_DIM));
    }
    for (k, v) in skills.into_iter().take(4) {
        first.push(line(format!("{:<13}{}", skill_name(&k), points(v)), TEXT).bar(v));
    }
    let mut details = vec![
        line(format!("STR {} DEX {} CON {} INT {} WIS {} CHA {}", s.str, s.dex, s.con, s.int, s.wis, s.cha), TEXT),
        line(format!("TALENTS: {}", m.skills.transferable.iter().map(|t| skill_name(t)).collect::<Vec<_>>().join(", ")), TEXT_DIM),
    ];
    if m.earned > 0.0 {
        details.push(line(format!("EARNED {} RENOWN SO FAR", m.earned.floor()), TEXT).icon(Icon::Renown));
    }
    Card::two(format!("{} THE {}", m.name, m.kind.to_uppercase()), first, details, (m.x, m.y))
}

fn plant_card(g: &Game, p: usize) -> Card {
    let plant = &g.world.plants[p];
    let name = g.good_name(&plant.good);
    let state = if plant.harvest.is_some() {
        line("BEING HARVESTED", TEXT_GOOD)
    } else if !plant.stock.is_empty() {
        line(format!("{} MORE {name} COMING OUT", plant.stock.len()), TEXT_GOOD)
    } else if plant.ripe() {
        line(if plant.good == "cotton_bolls" { "RIPE. STAND ON IT AND PRESS SPACE." } else { "READY. STAND ON IT AND PRESS SPACE." }, TEXT_GOOD)
    } else {
        line(format!("GROWING: {}%", plant.growth * 100 / plant.regrow.max(1)), TEXT_DIM)
    };
    let mut first = vec![state];
    // What a harvest would give the monster you control: gathering is a craft with no
    // inputs or tools.
    let c = g.controlled;
    let m = &g.monsters[c];
    let recipe = g.data.good(&plant.good);
    let craft = Craft { recipe, crafter: Crafter { id: m.id, stats: &m.stats, skills: &m.skills }, inputs: Vec::new(), tools: Vec::new(), workshop: None };
    let mut details = Vec::new();
    if recipe.has_quality {
        let f = econ::forecast(&craft);
        let (lo, hi) = usual_range(&f, m);
        first.extend(quality(lo, f.expected(&m.stats), hi));
        details.push(line(format!("USUALLY {lo:.2} TO {hi:.2}"), TEXT_DIM));
    }
    let (lo, hi, avg) = econ::quantity_range(&craft);
    first.push(time_line(g, recipe, c, lo, hi));
    if lo != hi {
        details.push(line(format!("ABOUT {avg:.1} {name} A HARVEST"), TEXT_DIM));
    }
    let (learns, more) = learning(g, recipe, c, "HARVESTS");
    first.push(learns);
    details.extend(more);
    Card::two(plant_name(&plant.good), first, details, (plant.x, plant.y))
}

pub(crate) fn plant_name(good: &str) -> &'static str {
    match good {
        "hemlock_log" => "HEMLOCK TREE",
        "oak_log" => "OAK TREE",
        "elder_sticks" => "ELDER BUSH",
        "rhododendron_sticks" => "RHODODENDRON",
        "arrow_bamboo" => "ARROW BAMBOO",
        "ramie_stalks" => "RAMIE",
        "indigofera_leaves" => "INDIGOFERA",
        "limestone" => "LIMESTONE OUTCROP",
        _ => "COTTON PLANT",
    }
}

fn bin_card(g: &Game, b: usize) -> Card {
    let bin = &g.world.bins[b];
    let name = g.good_name(&bin.accepts[0]);
    let mut lines: Vec<Line> = bin
        .accepts
        .iter()
        .map(|a| line(format!("{} {}", bin.stock.iter().filter(|d| &d.good == a).count(), g.good_name(a)), TEXT).icon(Icon::Good(a.clone())))
        .collect();
    if !bin.stock.is_empty() && bin.accepts.iter().all(|a| g.data.good(a).has_quality) {
        let qs: Vec<f64> = bin.stock.iter().map(|i| i.quality).collect();
        let avg = qs.iter().sum::<f64>() / qs.len() as f64;
        let (lo, hi) = qs.iter().fold((f64::MAX, f64::MIN), |(a, b), &q| (a.min(q), b.max(q)));
        lines.extend(quality(lo, avg, hi));
    }
    if bin.scores {
        lines.push(line("PUSH IT IN TO EARN RENOWN.", TEXT_DIM).icon(Icon::Renown));
    }
    if bin.mouth.is_some() {
        lines.push(line("ITS CHUTE PUSHES THEM OUT ONE AT A TIME.", TEXT_DIM));
    }
    Card::one(format!("{name} BIN"), lines, (bin.x, bin.y))
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

/// Two standard deviations either side: the range most rolls land in.
fn usual_range(f: &Forecast, m: &crate::game::Monster) -> (f64, f64) {
    (econ::nudge((f.mean - 2.0 * f.sigma).max(0.0), &m.stats), f.mean + 2.0 * f.sigma)
}

fn forecast_card(g: &Game, s: usize, i: usize) -> Card {
    let pv = g.preview(s);
    let shop = &g.world.shops[s];
    let recipe = g.data.good(&pv.recipe);
    let m = &g.monsters[i];
    let (mut first, mut details) = (Vec::new(), Vec::new());
    if let Some(j) = &shop.job {
        first.push(line(format!("WORKING: {}%", j.t * 100 / j.total.max(1)), TEXT_GOOD));
    }
    if !shop.pending.is_empty() {
        details.push(line(format!("{} MORE WAITING INSIDE", shop.pending.len()), TEXT));
    }
    let (mut lo, mut hi, mut held) = (1, 1, None);
    match (&pv.inputs, &pv.tools, forecast(g, s, i)) {
        (Err(need), _, _) => first.push(line(need.clone(), TEXT_WARN).icon(Icon::Warn)),
        (_, Err(tool), _) => first.push(line(format!("MISSING A TOOL: {tool}."), TEXT_WARN).icon(Icon::Warn)),
        (Ok(inputs), Ok(tools), Some(f)) => {
            let craft = Craft { recipe, crafter: Crafter { id: m.id, stats: &m.stats, skills: &m.skills }, inputs: inputs.iter().collect(), tools: tools.iter().collect(), workshop: pv.workshop.as_ref() };
            let (qlo, qhi, avg) = econ::quantity_range(&craft);
            (lo, hi) = (qlo, qhi);
            if qlo != qhi {
                details.push(line(format!("ABOUT {avg:.1} EACH TIME"), TEXT_DIM));
            }
            if recipe.has_quality {
                let (rlo, rhi) = usual_range(&f, m);
                first.extend(quality(rlo, f.expected(&m.stats), rhi));
                details.push(line(format!("USUALLY {rlo:.2} TO {rhi:.2}"), TEXT_DIM));
                let (exc, mw) = (chance_at_least(&f, EXCEPTIONAL), chance_at_least(&f, MASTERWORK));
                details.push(if exc < 1e-4 {
                    line("NO EXCEPTIONAL WORK YET", TEXT_DIM)
                } else {
                    line(format!("EXCEPTIONAL {}, MASTERWORK {}", odds(exc), odds(mw)), TEXT_DIM)
                });
                match held_back(g, &f, recipe, i) {
                    Some((name, shown, hint)) => {
                        held = Some(line(format!("HELD BACK: {name}"), TEXT_WARN).icon(Icon::Warn));
                        details.push(line(format!("{name} IS AT {shown}. {hint}"), TEXT_DIM));
                    }
                    None => details.push(line("NOTHING HOLDS IT BACK MUCH.", TEXT_GOOD)),
                }
            }
        }
        _ => {}
    }
    // What the spot makes and what it teaches, whether or not it can start yet.
    first.push(time_line(g, recipe, i, lo, hi));
    first.extend(held);
    let (learns, more) = learning(g, recipe, i, "CRAFTS");
    first.push(learns);
    details.extend(more);
    if let Some(fr) = &shop.frontier {
        if g.worn_out(s) {
            first.insert(0, line(g.worn_note(s), TEXT_WARN).icon(Icon::Warn));
        }
        let left = crate::frontier::LIFETIME_TICKS.saturating_sub(fr.worked);
        details.push(line(format!("WEAR: {}% LEFT", left * 100 / crate::frontier::LIFETIME_TICKS), TEXT).icon(Icon::Wear).bar(left as f64 / crate::frontier::LIFETIME_TICKS as f64));
        if fr.speed > 1.005 {
            details.push(line(format!("WORKS {:.0}% FASTER THAN A TOWN WORKSHOP", (fr.speed - 1.0) * 100.0), TEXT_DIM));
        }
        let makers: Vec<&str> = fr.shares.iter().filter_map(|sh| g.monsters.iter().find(|m| m.id == sh.monster)).map(|m| m.name.as_str()).collect();
        if !makers.is_empty() {
            details.push(line(format!("BUILT BY {}: THEY SHARE IN ITS WORK.", makers.join(", ")), TEXT_DIM));
        }
    }
    Card::two(format!("{}: {}", g.shop_name(s), recipe.name.to_uppercase()), first, details, shop.spot)
}

/// "[time] 5.5S  [rope] 1 ROPE": how long a job takes this monster and what comes out.
fn time_line(g: &Game, recipe: &GoodType, i: usize, lo: u32, hi: u32) -> Line {
    let secs = g.job_ticks(&recipe.id, i) as f64 / 15.0;
    let name = recipe.name.to_uppercase();
    let makes = if lo == hi { format!("{lo} {name}") } else { format!("{lo} TO {hi} {name}") };
    line(format!("{secs:.1}S"), TEXT).icon(Icon::Time).then(Icon::Good(recipe.id.clone()), makes)
}

/// What a job teaches, as a dry run on a copy of the monster's skills: the main line for the
/// first page, then the rest.
fn learning(g: &Game, recipe: &GoodType, i: usize, jobs: &str) -> (Line, Vec<Line>) {
    let m = &g.monsters[i];
    let mut sk = m.skills.clone();
    let job_secs = econ::production_seconds(recipe, &m.stats);
    let l = econ::learn(&mut sk, &m.stats, recipe, &g.data, job_secs);
    let primary = skill_name(&recipe.primary_applied_skill);
    let main = line(format!("{primary} +{}  PRACTICE +{}", points(l.primary), points(l.specific)), TEXT_GOOD).icon(Icon::Learn);
    let mut more = Vec::new();
    if !l.secondary.is_empty() {
        let also: Vec<String> = l.secondary.iter().map(|(k, v)| format!("{} +{}", skill_name(k), points(*v))).collect();
        more.push(line(format!("ALSO {}", also.join(", ")), TEXT_GOOD).icon(Icon::Learn));
    }
    more.push(line(format!("FORGETS {} OF EVERY OTHER SKILL", points(l.forgotten)), TEXT_DIM).icon(Icon::Forget));
    let (level, n) = level_off(g, i, recipe, job_secs);
    more.push(line(format!("{primary} LEVELS OFF NEAR {} IN ABOUT {n} {jobs}", points(level)), TEXT_DIM));
    (main, more)
}

/// Where the primary skill settles if the monster keeps doing this, and how soon.
fn level_off(g: &Game, i: usize, recipe: &GoodType, secs: f64) -> (f64, u32) {
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

/// The weakest part of the roll, how it stands, and what would help; None if nothing much
/// holds it back.
fn held_back(g: &Game, f: &Forecast, recipe: &GoodType, i: usize) -> Option<(String, String, String)> {
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
    let (name, v, shown, hint) = factors.into_iter().min_by(|a, b| a.1.total_cmp(&b.1))?;
    (v < 0.95).then_some((name, shown, hint))
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
