//! Level data: hand-drawn terrain maps plus what each level asks for and allows.

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
    /// A ladder that came with the level.
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

    pub fn name(self) -> &'static str {
        match self {
            Tile::Air => "AIR",
            Tile::Earth => "EARTH",
            Tile::Rock => "ROCK",
            Tile::Bramble => "BRAMBLE (NO BUILDING)",
            Tile::Branch => "BRANCH (ONE-WAY)",
            Tile::Ledge => "THIN LEDGE (ONE-WAY, NO BUILDING)",
            Tile::Trunk => "TREE TRUNK",
            Tile::Leaves => "LEAVES",
            Tile::Vine => "VINE (APES CLIMB)",
            Tile::Water => "WATER",
            Tile::Ladder => "LADDER",
            Tile::Cloud => "CLOUD (HOLDS LIGHT LOADS ONLY)",
        }
    }
}

#[derive(Clone, Debug)]
pub struct Source {
    pub x: i32,
    pub y: i32,
    pub good: Good,
    /// Goods on hand at the start and the most it holds.
    pub max: i32,
    /// Ticks to grow one more back (0 = never regrows).
    pub regrow: i32,
}

impl Source {
    pub fn infinite(&self) -> bool {
        self.max <= 0
    }
    pub fn label(&self) -> &'static str {
        match self.good {
            Good::Berry => "berry bush",
            Good::Twig => "twig pile",
            Good::Stone => "quarry",
            Good::Clay => "clay bank",
            Good::Egg => "nest",
            _ => "source",
        }
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub struct Targets {
    pub ticks: i32,
    pub cost: i32,
    pub crew: i32,
    /// The combo goal: both at once in a single run (0 = none).
    pub combo_ticks: i32,
    pub combo_cost: i32,
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
    /// Fortress footprint: x, y (top-left), w, h.
    pub fortress: (i32, i32, i32, i32),
    pub need: Vec<(Good, i32)>,
    pub species: Vec<Species>,
    pub kinds: Vec<Kind>,
    pub infra: Vec<Infra>,
    /// The rule card shown on entering: teaches rules, never the solution.
    pub rule: String,
    /// What this site introduces, shown with its icon on the site card.
    pub new: Vec<String>,
    pub targets: Targets,
    /// A known solution in layout text form (used by tests and the balance harness).
    pub reference: String,
}

pub const MAP_W: i32 = 40;
pub const MAP_H: i32 = 24;

impl Level {
    pub fn tile(&self, x: i32, y: i32) -> Tile {
        if x < 0 || y < 0 || x >= self.w || y >= self.h {
            return Tile::Bramble;
        }
        self.tiles[(y * self.w + x) as usize]
    }

    pub fn need_of(&self, g: Good) -> i32 {
        self.need.iter().find(|(n, _)| *n == g).map(|(_, c)| *c).unwrap_or(0)
    }

    pub fn total_need(&self) -> i32 {
        self.need.iter().map(|(_, c)| c).sum()
    }

    /// Goods that can exist in this level (from sources and recipes).
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
}

/// Parses the level book. Each level starts with `@ id name`, then `key value` lines, then
/// `map` and exactly MAP_H rows of MAP_W characters.
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
            name: name.split_whitespace().map(|w| {
                let lower = w.to_lowercase();
                let mut c = lower.chars();
                match c.next() {
                    Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
                    None => String::new(),
                }
            }).collect::<Vec<_>>().join(" "),
            chapter,
            w: MAP_W,
            h: MAP_H,
            tiles: vec![Tile::Air; (MAP_W * MAP_H) as usize],
            springs: Vec::new(),
            sources: Vec::new(),
            fortress: (0, 0, 0, 0),
            need: Vec::new(),
            species: Vec::new(),
            kinds: Vec::new(),
            infra: Vec::new(),
            rule: String::new(),
            new: Vec::new(),
            targets: Targets::default(),
            reference: String::new(),
        };
        // Per-good source settings: (max, regrow).
        let mut src_cfg: Vec<(Good, i32, i32)> = Vec::new();
        // Per-letter settings override per-good ones (uppercase letters are a level's odd sources).
        let mut letter_cfg: Vec<(char, i32, i32)> = Vec::new();
        let mut rows: Vec<String> = Vec::new();
        while let Some(l) = lines.next() {
            let l = l.trim_end();
            if l == "map" {
                for _ in 0..MAP_H {
                    rows.push(lines.next().unwrap_or("").to_string());
                }
                break;
            }
            let (key, val) = l.split_once(' ').unwrap_or((l, ""));
            let val = val.trim();
            match key {
                "need" => {
                    let mut it = val.split_whitespace();
                    while let (Some(n), Some(g)) = (it.next(), it.next()) {
                        if let (Ok(n), Some(g)) = (n.parse(), Good::from_name(g)) {
                            lv.need.push((g, n));
                        }
                    }
                }
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
                "combo" => {
                    let n: Vec<i32> = val.split_whitespace().filter_map(|s| s.parse().ok()).collect();
                    if n.len() == 2 {
                        lv.targets.combo_ticks = n[0];
                        lv.targets.combo_cost = n[1];
                    }
                }
                "rule" | "intro" => {
                    if !lv.rule.is_empty() {
                        lv.rule.push('\n');
                    }
                    lv.rule.push_str(val);
                }
                "targets" => {
                    let n: Vec<i32> = val.split_whitespace().filter_map(|s| s.parse().ok()).collect();
                    if n.len() == 3 {
                        lv.targets.ticks = n[0];
                        lv.targets.cost = n[1];
                        lv.targets.crew = n[2];
                    }
                }
                "ref" => {
                    if !lv.reference.is_empty() {
                        lv.reference.push(';');
                    }
                    lv.reference.push_str(val);
                }
                _ => {}
            }
        }
        let cfg = |g: Good| -> (i32, i32) {
            src_cfg.iter().find(|c| c.0 == g).map(|c| (c.1, c.2)).unwrap_or(match g {
                Good::Berry => (3, 24),
                Good::Twig => (0, 0),
                Good::Stone => (0, 0),
                Good::Clay => (0, 0),
                Good::Egg => (2, 40),
                _ => (0, 0),
            })
        };
        let (mut fx0, mut fy0, mut fx1, mut fy1) = (i32::MAX, i32::MAX, -1, -1);
        for (y, row) in rows.iter().enumerate() {
            for (x, ch) in row.chars().enumerate().take(MAP_W as usize) {
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
                lv.tiles[(y * MAP_W + x) as usize] = t;
                let good = match ch {
                    'b' | 'B' => Some(Good::Berry),
                    't' | 'T' => Some(Good::Twig),
                    'q' | 'Q' => Some(Good::Stone),
                    'c' | 'C' => Some(Good::Clay),
                    'e' | 'E' => Some(Good::Egg),
                    _ => None,
                };
                if let Some(g) = good {
                    let (max, regrow) = letter_cfg.iter().find(|c| c.0 == ch).map(|c| (c.1, c.2)).unwrap_or_else(|| cfg(g));
                    lv.sources.push(Source { x, y, good: g, max, regrow });
                }
                if ch == '^' {
                    lv.springs.push((x, y));
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
