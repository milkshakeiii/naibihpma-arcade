//! Ship designs. Each design is pixel art where every colour is a part, so what a ship can do
//! is exactly what is drawn: yellow pixels shoot, pale blue ones push, orange ones scoop, grey
//! ones soak an extra hit, and the white core is the ship's life.

use std::sync::OnceLock;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Part {
    Hull,
    Armor,
    Gun,
    Engine,
    Scoop,
    Core,
}

impl Part {
    pub fn hp(self) -> u8 {
        if self == Part::Armor {
            2
        } else {
            1
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Kind {
    King,
    Collector,
    Escort,
    Raider,
    OutpostSeed,
    ForwardSeed,
    Outpost,
    Forward,
}

#[cfg_attr(not(test), allow(dead_code))]
pub const ALL: [Kind; 8] =
    [Kind::King, Kind::Collector, Kind::Escort, Kind::Raider, Kind::OutpostSeed, Kind::ForwardSeed, Kind::Outpost, Kind::Forward];

/// The player's hand: what the king can print.
pub const CARDS: [Kind; 5] = [Kind::Collector, Kind::Escort, Kind::Raider, Kind::OutpostSeed, Kind::ForwardSeed];

impl Kind {
    pub fn idx(self) -> usize {
        self as usize
    }
    /// Kings and planted bases print ships and take deliveries.
    pub fn is_factory(self) -> bool {
        matches!(self, Kind::King | Kind::Outpost | Kind::Forward)
    }
    pub fn is_base(self) -> bool {
        matches!(self, Kind::Outpost | Kind::Forward)
    }
    pub fn is_seed(self) -> bool {
        matches!(self, Kind::OutpostSeed | Kind::ForwardSeed)
    }
    pub fn grows_into(self) -> Option<Kind> {
        match self {
            Kind::OutpostSeed => Some(Kind::Outpost),
            Kind::ForwardSeed => Some(Kind::Forward),
            _ => None,
        }
    }
    /// A planted base's pinned build order: it keeps one of each entry alive.
    pub fn pinned(self) -> &'static [Kind] {
        match self {
            Kind::Outpost => &[Kind::Collector, Kind::Collector, Kind::Escort, Kind::OutpostSeed],
            Kind::Forward => &[Kind::Escort, Kind::Escort, Kind::Raider, Kind::Raider, Kind::Raider],
            _ => &[],
        }
    }
}

pub struct Design {
    pub w: i32,
    pub h: i32,
    pub cells: Vec<Option<Part>>,
    /// Cells in build order: the core first, then outward.
    pub order: Vec<usize>,
    /// Position of each cell in `order`.
    pub rank: Vec<usize>,
    pub count: usize,
    pub full_hp: u32,
}

impl Design {
    fn parse(rows: &[&str]) -> Design {
        let h = rows.len() as i32;
        let w = rows[0].len() as i32;
        let mut cells = Vec::new();
        for r in rows {
            assert_eq!(r.len() as i32, w, "ragged design row {r:?}");
            for c in r.chars() {
                cells.push(match c {
                    'h' => Some(Part::Hull),
                    'a' => Some(Part::Armor),
                    'g' => Some(Part::Gun),
                    'e' => Some(Part::Engine),
                    's' => Some(Part::Scoop),
                    'c' => Some(Part::Core),
                    _ => None,
                });
            }
        }
        let mut order: Vec<usize> = (0..cells.len()).filter(|&i| cells[i].is_some()).collect();
        order.sort_by_key(|&i| {
            let x = (i as i32 % w) * 2 - (w - 1);
            let y = (i as i32 / w) * 2 - (h - 1);
            (cells[i] != Some(Part::Core), x * x + y * y, i)
        });
        let mut rank = vec![usize::MAX; cells.len()];
        for (k, &i) in order.iter().enumerate() {
            rank[i] = k;
        }
        let full_hp = cells.iter().flatten().map(|p| p.hp() as u32).sum();
        let count = order.len();
        Design { w, h, cells, order, rank, count, full_hp }
    }

    #[cfg_attr(not(test), allow(dead_code))]
    pub fn parts(&self, p: Part) -> usize {
        self.cells.iter().filter(|c| **c == Some(p)).count()
    }
}

pub struct Info {
    pub kind: Kind,
    pub name: &'static str,
    /// Short card label (fits the sidebar).
    pub label: &'static str,
    pub design: Design,
    pub speed: i32,
    pub range: i32,
    pub blurb: &'static str,
}

static INFOS: OnceLock<Vec<Info>> = OnceLock::new();

pub fn info(k: Kind) -> &'static Info {
    &INFOS.get_or_init(build)[k.idx()]
}

/// What printing a card costs: one bank pixel per pixel printed.
pub fn cost(k: Kind) -> usize {
    info(k).design.count
}

#[rustfmt::skip]
const KING: [&str; 15] = [
    "......aga......",
    "....aahhhaa....",
    "...ahhhghhha...",
    "..ahhhhhhhhha..",
    ".aahhhhhhhhhaa.",
    ".ahhhhhhhhhhha.",
    "aahhhhccchhhhaa",
    "aghhhhccchhhhga",
    "aahhhhccchhhhaa",
    ".ahhhhhhhhhhha.",
    ".aahhhhhhhhhaa.",
    "..ahhhhhhhhha..",
    "...aahhhhhaa...",
    "....aehhhea....",
    ".....e.e.e.....",
];

#[rustfmt::skip]
const COLLECTOR: [&str; 4] = [
    "s.s",
    "shs",
    "hch",
    ".e.",
];

#[rustfmt::skip]
const ESCORT: [&str; 4] = [
    ".g.",
    "aga",
    "hch",
    "e.e",
];

#[rustfmt::skip]
const RAIDER: [&str; 5] = [
    "g.g.g",
    "hhghh",
    "ahcha",
    ".aha.",
    ".e.e.",
];

#[rustfmt::skip]
const OUTPOST_SEED: [&str; 3] = [
    ".hsh.",
    "hhchh",
    ".heh.",
];

#[rustfmt::skip]
const FORWARD_SEED: [&str; 3] = [
    ".hgh.",
    "hhchh",
    ".heh.",
];

#[rustfmt::skip]
const OUTPOST: [&str; 11] = [
    "....aaa....",
    "..aahhhaa..",
    ".ahhhghhha.",
    ".ahshhhsha.",
    "ahhhhhhhhha",
    "ahhhhchhhha",
    "ahhhhhhhhha",
    ".ahshhhsha.",
    ".ahhhghhha.",
    "..aahhhaa..",
    "....aaa....",
];

#[rustfmt::skip]
const FORWARD: [&str; 13] = [
    ".....aaa.....",
    "...aahghaa...",
    "..ahhhhhhha..",
    ".ahghhhhhgha.",
    ".ahhhhhhhhha.",
    "aahhhhhhhhhaa",
    "aghhhhchhhhga",
    "aahhhhhhhhhaa",
    ".ahhhhhhhhha.",
    ".ahghhhhhgha.",
    "..ahhhhhhha..",
    "...aahghaa...",
    ".....aaa.....",
];

fn build() -> Vec<Info> {
    let mk = |kind, name, label, rows: &[&str], speed, range, blurb| Info { kind, name, label, design: Design::parse(rows), speed, range, blurb };
    let v = vec![
        mk(Kind::King, "KING", "KING", &KING, 3, 56, "YOUR KING. IT FLIES, SHOOTS AND PRINTS YOUR CARDS. IF IT DIES YOU LOSE."),
        mk(Kind::Collector, "COLLECTOR", "COLLECTOR", &COLLECTOR, 3, 0, "CHEWS PIXELS OFF ROCKS AND WRECKS AND HAULS THEM TO THE NEAREST FACTORY."),
        mk(Kind::Escort, "ESCORT", "ESCORT", &ESCORT, 4, 40, "GUARDS THE FACTORY THAT PRINTED IT."),
        mk(Kind::Raider, "RAIDER", "RAIDER", &RAIDER, 3, 44, "WAITS AT ITS FACTORY UNTIL THERE ARE 3, THEN THE PACK HUNTS THE NEAREST ENEMY FACTORY."),
        mk(
            Kind::OutpostSeed,
            "OUTPOST SEED",
            "OUTPOST",
            &OUTPOST_SEED,
            4,
            0,
            "FLIES TO A ROCK AND GROWS INTO AN OUTPOST. AN OUTPOST KEEPS 2 COLLECTORS, 1 ESCORT AND 1 OUTPOST SEED.",
        ),
        mk(
            Kind::ForwardSeed,
            "FORWARD BASE SEED",
            "FWD BASE",
            &FORWARD_SEED,
            4,
            0,
            "FLIES TO A ROCK AND GROWS INTO A FORWARD BASE. A FORWARD BASE KEEPS 2 ESCORTS AND 3 RAIDERS.",
        ),
        mk(Kind::Outpost, "OUTPOST", "OUTPOST", &OUTPOST, 0, 44, "KEEPS 2 COLLECTORS, 1 ESCORT AND 1 OUTPOST SEED, REBUILDING ANY THAT DIE."),
        mk(Kind::Forward, "FORWARD BASE", "FWD BASE", &FORWARD, 0, 52, "KEEPS 2 ESCORTS AND 3 RAIDERS, REBUILDING ANY THAT DIE."),
    ];
    for (i, inf) in v.iter().enumerate() {
        assert_eq!(inf.kind.idx(), i);
    }
    v
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_design_has_a_core_built_first() {
        for k in ALL {
            let d = &info(k).design;
            assert!(d.parts(Part::Core) >= 1, "{k:?}");
            assert_eq!(d.cells[d.order[0]], Some(Part::Core), "{k:?}");
            for (r, &i) in d.order.iter().enumerate() {
                assert_eq!(d.rank[i], r);
            }
        }
    }

    #[test]
    fn costs_are_pixel_counts() {
        assert_eq!(cost(Kind::Collector), 9);
        assert_eq!(cost(Kind::Escort), 9);
        assert_eq!(cost(Kind::Raider), 18);
        assert_eq!(cost(Kind::OutpostSeed), 11);
    }
}
