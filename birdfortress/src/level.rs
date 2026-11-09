//! Site data: hand-drawn terrain maps plus what each site asks for and allows.

use crate::rules::*;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Tile {
    Air,
    /// Solid, buildable top (grass, soil).
    Earth,
    /// Solid, buildable top (grey stone).
    Rock,
    /// Solid, but nothing can be built on it (thorny bramble).
    Bramble,
    /// One-way platform you can build on (a stout tree branch).
    Branch,
    /// One-way platform you can't build on (a thin twiggy ledge).
    Ledge,
    /// Background tree trunk: creatures pass in front of it.
    Trunk,
    /// Background foliage.
    Leaves,
    /// Hangs in the air; apes climb it.
    Vine,
    /// Nobody enters water.
    Water,
    /// A ladder that came with the site.
    Ladder,
    /// A cloud bank: stand on it carrying nothing or something light; heavier loads fall through.
    Cloud,
}

impl Tile {
    pub fn solid(self) -> bool {
        matches!(self, Tile::Earth | Tile::Rock | Tile::Bramble)
    }
    pub fn platform(self) -> bool {
        matches!(self, Tile::Branch | Tile::Ledge)
    }
    /// Can a building stand on this tile?
    pub fn buildable_top(self) -> bool {
        matches!(self, Tile::Earth | Tile::Rock | Tile::Branch)
    }
    /// Free air you can put things in (foliage is just scenery).
    pub fn open(self) -> bool {
        matches!(self, Tile::Air | Tile::Leaves)
    }
    /// What hovering this tile says.
    pub fn label(self) -> &'static str {
        match self {
            Tile::Air => "",
            Tile::Earth => "Earth.",
            Tile::Rock => "Rock.",
            Tile::Bramble => "Bramble. Nothing can be built on it.",
            Tile::Branch => "Branch. Stand on it, or jump up through it.",
            Tile::Ledge => "Thin ledge. Like a branch, but nothing can be built on it.",
            Tile::Trunk => "Tree trunk. Creatures pass in front of it.",
            Tile::Leaves => "Leaves.",
            Tile::Vine => "Vine. Apes climb it, with anything.",
            Tile::Water => "Water. Nobody swims.",
            Tile::Ladder => "Ladder.",
            Tile::Cloud => "Cloud. It holds you up only if you carry nothing, or something light.",
        }
    }
}

#[derive(Clone, Debug)]
pub struct Source {
    pub x: i32,
    pub y: i32,
    pub good: Good,
    /// Goods on hand at the start and the most it holds (<= 0: endless).
    pub max: i32,
    /// Ticks to grow one more back (0 = never regrows).
    pub regrow: i32,
}

impl Source {
    pub fn infinite(&self) -> bool {
        self.max <= 0
    }
}

/// One stage of the fortress: what it takes.
#[derive(Clone, Debug)]
pub struct Stage {
    pub name: String,
    pub need: Vec<(Good, i32)>,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Goal {
    /// Every fortress stage built.
    Fortress,
    /// At least this many residents.
    Pop(i32),
    /// At least `count` homes at `tier` (0-based) or better.
    Tier(usize, i32),
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Challenge {
    None,
    /// Never more than this many stores standing.
    MaxStores(i32),
    /// Never house this species.
    NoSpecies(Species),
    /// No ladders, planks or springboards.
    NoInfra,
    /// No home ever slips a tier or loses a resident to hunger.
    NoHunger,
    /// Never more than this many homes standing.
    MaxHomes(i32),
}

impl Challenge {
    pub fn text(self) -> String {
        match self {
            Challenge::None => String::new(),
            Challenge::MaxStores(n) => format!("Never more than {n} store{}", if n == 1 { "" } else { "s" }),
            Challenge::NoSpecies(s) => format!("No {}", s.plural()),
            Challenge::NoInfra => "No ladders, planks or springboards".to_string(),
            Challenge::NoHunger => "No home ever slips".to_string(),
            Challenge::MaxHomes(n) => format!("Never more than {n} homes"),
        }
    }
}

#[derive(Clone, Debug)]
pub struct Level {
    pub id: String,
    pub name: String,
    pub chapter: usize,
    pub w: i32,
    pub h: i32,
    pub tiles: Vec<Tile>,
    pub springs: Vec<(i32, i32)>,
    pub sources: Vec<Source>,
    /// Where settlers step onto the map.
    pub entrance: (i32, i32),
    /// Fortress footprint: x, y (top-left), w, h (all zero when the site has none).
    pub fortress: (i32, i32, i32, i32),
    pub stages: Vec<Stage>,
    pub goals: Vec<Goal>,
    pub seeds: i32,
    /// Homes the player may build, by species.
    pub species: Vec<Species>,
    pub kinds: Vec<Kind>,
    pub infra: Vec<Infra>,
    /// Gold, silver and bronze finishing days.
    pub medals: [i32; 3],
    pub challenge: Challenge,
    /// The card shown on entering: teaches rules, never the solution.
    pub rule: String,
    /// What this site introduces, shown with its icon on the site card.
    pub new: Vec<String>,
    /// Bot build plans, each a `;`-separated action list (used by tests and the balance harness).
    pub plans: Vec<String>,
}

impl Level {
    pub fn tile(&self, x: i32, y: i32) -> Tile {
        if x < 0 || y < 0 || x >= self.w || y >= self.h {
            return Tile::Bramble;
        }
        self.tiles[(y * self.w + x) as usize]
    }

    pub fn has_fortress(&self) -> bool {
        self.fortress.2 > 0 && !self.stages.is_empty()
    }

    pub fn source_at(&self, x: i32, y: i32) -> Option<usize> {
        self.sources.iter().position(|s| s.x == x && s.y == y)
    }

    /// Every good that can turn up here.
    pub fn goods(&self) -> Vec<Good> {
        let mut v: Vec<Good> = Vec::new();
        for s in &self.sources {
            if !v.contains(&s.good) {
                v.push(s.good);
            }
        }
        for k in &self.kinds {
            if let Some(r) = k.recipe() {
                if !v.contains(&r.output) {
                    v.push(r.output);
                }
            }
        }
        v.sort();
        v
    }

    pub fn medal_for(&self, day: i32) -> usize {
        // 0 gold, 1 silver, 2 bronze, 3 finished without a medal.
        self.medals.iter().position(|&m| m > 0 && day <= m).unwrap_or(3)
    }
}

fn title_case(s: &str) -> String {
    s.split_whitespace()
        .map(|w| {
            let lower = w.to_lowercase();
            let mut c = lower.chars();
            match c.next() {
                Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
                None => String::new(),
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

/// Parses the site book. Each site starts with `@ id name`, then `key value` lines, then `map`,
/// the map rows, and `end`.
pub fn parse_book(text: &str) -> Vec<Level> {
    let mut out = Vec::new();
    let mut lines = text.lines().peekable();
    let mut chapter = 0usize;
    while let Some(line) = lines.next() {
        let line = line.trim_end();
        if let Some(rest) = line.strip_prefix("chapter ") {
            chapter = rest.trim().parse().unwrap_or(chapter);
            continue;
        }
        let Some(head) = line.strip_prefix("@ ") else { continue };
        let (id, name) = head.split_once(' ').unwrap_or((head, ""));
        let mut lv = Level {
            id: id.to_string(),
            name: title_case(name),
            chapter,
            w: 0,
            h: 0,
            tiles: Vec::new(),
            springs: Vec::new(),
            sources: Vec::new(),
            entrance: (0, 0),
            fortress: (0, 0, 0, 0),
            stages: Vec::new(),
            goals: Vec::new(),
            seeds: 30,
            species: Vec::new(),
            kinds: Vec::new(),
            infra: Vec::new(),
            medals: [0; 3],
            challenge: Challenge::None,
            rule: String::new(),
            new: Vec::new(),
            plans: Vec::new(),
        };
        let mut src_cfg: Vec<(Good, i32, i32)> = Vec::new();
        let mut letter_cfg: Vec<(char, i32, i32)> = Vec::new();
        let mut rows: Vec<String> = Vec::new();
        let mut plan = String::new();
        while let Some(l) = lines.next() {
            let l = l.trim_end();
            if l == "map" {
                for r in lines.by_ref() {
                    let r = r.trim_end();
                    if r == "end" {
                        break;
                    }
                    rows.push(r.to_string());
                }
                break;
            }
            let (key, val) = l.split_once(' ').unwrap_or((l, ""));
            let val = val.trim();
            match key {
                "seeds" => lv.seeds = val.parse().unwrap_or(30),
                "species" => lv.species = val.split_whitespace().filter_map(Species::from_name).collect(),
                "build" => {
                    for w in val.split_whitespace() {
                        if let Some(k) = Kind::from_name(w) {
                            lv.kinds.push(k);
                        } else if let Some(i) = Infra::from_name(w) {
                            lv.infra.push(i);
                        }
                    }
                }
                "stage" => {
                    // stage Name words... N GOOD N GOOD
                    let toks: Vec<&str> = val.split_whitespace().collect();
                    let first_num = toks.iter().position(|t| t.parse::<i32>().is_ok()).unwrap_or(toks.len());
                    let name = toks[..first_num].join(" ");
                    let mut need = Vec::new();
                    let mut i = first_num;
                    while i + 1 < toks.len() {
                        if let (Ok(n), Some(g)) = (toks[i].parse::<i32>(), Good::from_name(toks[i + 1])) {
                            need.push((g, n));
                        }
                        i += 2;
                    }
                    lv.stages.push(Stage { name, need });
                }
                "goal" => {
                    let toks: Vec<&str> = val.split_whitespace().collect();
                    match toks.first().copied() {
                        Some("fortress") => lv.goals.push(Goal::Fortress),
                        Some("pop") => lv.goals.push(Goal::Pop(toks.get(1).and_then(|t| t.parse().ok()).unwrap_or(10))),
                        Some("tier") => {
                            let t: usize = toks.get(1).and_then(|t| t.parse().ok()).unwrap_or(2);
                            let n: i32 = toks.get(2).and_then(|t| t.parse().ok()).unwrap_or(1);
                            lv.goals.push(Goal::Tier(t.saturating_sub(1).min(TIERS - 1), n));
                        }
                        _ => {}
                    }
                }
                "medals" => {
                    let n: Vec<i32> = val.split_whitespace().filter_map(|s| s.parse().ok()).collect();
                    for (i, v) in n.iter().take(3).enumerate() {
                        lv.medals[i] = *v;
                    }
                }
                "challenge" => {
                    let toks: Vec<&str> = val.split_whitespace().collect();
                    lv.challenge = match toks.first().copied() {
                        Some("maxstores") => Challenge::MaxStores(toks.get(1).and_then(|t| t.parse().ok()).unwrap_or(1)),
                        Some("maxhomes") => Challenge::MaxHomes(toks.get(1).and_then(|t| t.parse().ok()).unwrap_or(4)),
                        Some("no") => toks.get(1).and_then(|t| Species::from_name(t)).map(Challenge::NoSpecies).unwrap_or(Challenge::None),
                        Some("noinfra") => Challenge::NoInfra,
                        Some("nohunger") => Challenge::NoHunger,
                        _ => Challenge::None,
                    };
                }
                "src" => {
                    let p: Vec<&str> = val.split_whitespace().collect();
                    if p.len() == 3 {
                        if let (Ok(m), Ok(r)) = (p[1].parse(), p[2].parse()) {
                            if let Some(g) = Good::from_name(p[0]) {
                                src_cfg.push((g, m, r));
                            } else if let Some(c) = p[0].chars().next().filter(|_| p[0].len() == 1) {
                                letter_cfg.push((c, m, r));
                            }
                        }
                    }
                }
                "new" => lv.new.extend(val.split_whitespace().map(|w| w.to_ascii_uppercase())),
                "rule" | "intro" => {
                    if !lv.rule.is_empty() {
                        lv.rule.push('\n');
                    }
                    lv.rule.push_str(val);
                }
                "plan" => {
                    if !plan.is_empty() {
                        plan.push(';');
                    }
                    plan.push_str(val);
                }
                "nextplan" => {
                    if !plan.is_empty() {
                        lv.plans.push(std::mem::take(&mut plan));
                    }
                }
                _ => {}
            }
        }
        if !plan.is_empty() {
            lv.plans.push(plan);
        }
        if lv.goals.is_empty() && !lv.stages.is_empty() {
            lv.goals.push(Goal::Fortress);
        }
        if !lv.stages.is_empty() && !lv.goals.contains(&Goal::Fortress) {
            lv.goals.insert(0, Goal::Fortress);
        }
        let cfg = |g: Good| -> (i32, i32) {
            src_cfg.iter().find(|c| c.0 == g).map(|c| (c.1, c.2)).unwrap_or(match g {
                Good::Berry => (4, 50),
                Good::Nut => (8, 60),
                Good::Fish => (3, 70),
                Good::Twig => (5, 45),
                _ => (0, 0),
            })
        };
        let w = rows.iter().map(|r| r.chars().count()).max().unwrap_or(0) as i32;
        let h = rows.len() as i32;
        lv.w = w;
        lv.h = h;
        lv.tiles = vec![Tile::Air; (w * h) as usize];
        let (mut fx0, mut fy0, mut fx1, mut fy1) = (i32::MAX, i32::MAX, -1, -1);
        for (y, row) in rows.iter().enumerate() {
            for (x, ch) in row.chars().enumerate() {
                let (x, y) = (x as i32, y as i32);
                let t = match ch {
                    '#' => Tile::Earth,
                    '%' => Tile::Rock,
                    'X' => Tile::Bramble,
                    '=' => Tile::Branch,
                    '-' => Tile::Ledge,
                    '|' => Tile::Trunk,
                    ':' => Tile::Leaves,
                    'v' => Tile::Vine,
                    '~' => Tile::Water,
                    'H' => Tile::Ladder,
                    '@' => Tile::Cloud,
                    _ => Tile::Air,
                };
                lv.tiles[(y * w + x) as usize] = t;
                let good = match ch {
                    'b' | 'B' => Some(Good::Berry),
                    'n' | 'N' => Some(Good::Nut),
                    'f' => Some(Good::Fish),
                    't' | 'T' => Some(Good::Twig),
                    'q' | 'Q' => Some(Good::Stone),
                    'c' | 'C' => Some(Good::Clay),
                    _ => None,
                };
                if let Some(g) = good {
                    let (max, regrow) = letter_cfg.iter().find(|c| c.0 == ch).map(|c| (c.1, c.2)).unwrap_or_else(|| cfg(g));
                    lv.sources.push(Source { x, y, good: g, max, regrow });
                }
                if ch == '^' {
                    lv.springs.push((x, y));
                }
                if ch == 'E' {
                    lv.entrance = (x, y);
                }
                if ch == 'F' {
                    fx0 = fx0.min(x);
                    fy0 = fy0.min(y);
                    fx1 = fx1.max(x);
                    fy1 = fy1.max(y);
                }
            }
        }
        if fx1 >= 0 {
            lv.fortress = (fx0, fy0, fx1 - fx0 + 1, fy1 - fy0 + 1);
        }
        out.push(lv);
    }
    out
}
