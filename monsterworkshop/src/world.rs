//! Pinewater as data: a tile map (`data/pinewater.map`) with its workshops, bins and plants,
//! plus the items lying around. No monsters here; `game.rs` moves them and asks the world
//! what is where.

use crate::data::{matches, Data};
use crate::econ::ItemData;
use serde::{Deserialize, Serialize};

pub const TS: i32 = 16;
pub const SCREEN_W: i32 = 25;
pub const SCREEN_H: i32 = 16;
pub const DISPENSE_TICKS: u8 = 4;
/// Town workshops' built-in tools, used when nothing better hangs on the peg.
pub const BUILTIN_TOOL_QUALITY: f64 = 0.5;
pub const TOWN_WORKSHOP_QUALITY: f64 = 0.8;
/// Finished goods a workshop holds inside, beyond the one on its output mat.
pub const OUTPUT_BUFFER: usize = 9;

// `#` hedge  `.` grass  `,` street  `W` wall  `_` floor  `D` door  `f` cotton plant
// `C` cotton bin and `m` its chute  `T` cloth store (a drop-off)  `a` applicant stand in
// the hiring hall  `1` `2` monster starts
// Work spots: `B` workbench  `S` spinning wheel  `L` loom. Each `p` pad, `g` tool peg and
// `o` output mat belongs to the nearest work spot.
const MAP: &str = include_str!("../data/pinewater.map");

#[derive(Clone, Copy, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub enum Dir {
    Up,
    Down,
    Left,
    Right,
}

pub const DIRS: [Dir; 4] = [Dir::Up, Dir::Down, Dir::Left, Dir::Right];

impl Dir {
    pub fn delta(self) -> (i32, i32) {
        match self {
            Dir::Up => (0, -1),
            Dir::Down => (0, 1),
            Dir::Left => (-1, 0),
            Dir::Right => (1, 0),
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Tile {
    Grass,
    Path,
    Hedge,
    Wall,
    Floor,
    Door,
    Pad,
    Peg,
    OutMat,
    Spot,
    Bin,
}

impl Tile {
    pub fn walkable(self) -> bool {
        !matches!(self, Tile::Hedge | Tile::Wall | Tile::Bin)
    }

    /// Items slide over anything a monster can walk on except a work spot.
    pub fn item_floor(self) -> bool {
        self.walkable() && self != Tile::Spot
    }
}

/// A move in progress, drawn as an offset back toward the tile it came from.
#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub struct Slide {
    pub dir: Dir,
    pub left: u8,
    pub total: u8,
}

impl Slide {
    pub fn offset(s: Option<Slide>) -> (i32, i32) {
        match s {
            Some(s) => {
                let (dx, dy) = s.dir.delta();
                let k = TS * s.left as i32 / s.total as i32;
                (-dx * k, -dy * k)
            }
            None => (0, 0),
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Item {
    pub x: i32,
    pub y: i32,
    pub data: ItemData,
    pub slide: Option<Slide>,
    pub hop: u8,
    pub locked: bool, // on a pad that is being worked
}

impl Item {
    pub fn new(x: i32, y: i32, data: ItemData) -> Self {
        Self { x, y, data, slide: None, hop: 0, locked: false }
    }

    pub fn good(&self) -> &str {
        &self.data.good
    }
}

/// A timed job at a plant or a work spot.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Job {
    pub monster: usize,
    pub recipe: String,
    pub t: u32,
    pub total: u32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Plant {
    pub x: i32,
    pub y: i32,
    pub good: String,
    pub growth: u32,
    pub regrow: u32,
    /// Gathered goods waiting to come out, one at a time, like a chute.
    pub stock: Vec<ItemData>,
    pub harvest: Option<Job>,
    /// Where the goods come out: the tile the harvester faced, and that direction.
    pub drop: Option<((i32, i32), Dir)>,
    pub timer: u8,
}

impl Plant {
    pub fn ripe(&self) -> bool {
        self.growth >= self.regrow
    }

    pub fn stage(&self) -> u8 {
        if self.ripe() || !self.stock.is_empty() {
            2
        } else if self.growth >= self.regrow / 2 {
            1
        } else {
            0
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Bin {
    pub x: i32,
    pub y: i32,
    pub accepts: String,
    pub stock: Vec<ItemData>,
    pub mouth: Option<(i32, i32)>,
    pub timer: u8,
    pub bounce: u8,
    /// A drop-off: goods pushed in are scored for renown.
    pub scores: bool,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub enum ShopKind {
    Workbench,
    Spinnery,
    LoomHouse,
}

impl ShopKind {
    pub fn label(self) -> &'static str {
        match self {
            ShopKind::Workbench => "WORKBENCH",
            ShopKind::Spinnery => "SPINNERY",
            ShopKind::LoomHouse => "LOOM HOUSE",
        }
    }

    pub fn recipes(self) -> &'static [&'static str] {
        match self {
            ShopKind::Workbench => &["cotton_slivers"],
            ShopKind::Spinnery => &["thread"],
            ShopKind::LoomHouse => &["fabric"],
        }
    }

    /// Basic permanent tools a town workshop provides when nothing hangs on its pegs.
    pub fn builtin_tools(self) -> &'static [&'static str] {
        match self {
            ShopKind::Workbench => &["cotton_gin", "cotton_bow"],
            ShopKind::Spinnery => &["spool", "spinning_wheel"],
            ShopKind::LoomHouse => &["waist_loom"],
        }
    }

    /// Walk-in workshops count as a weight-8 tool; an outdoor bench does not.
    pub fn is_building(self) -> bool {
        self != ShopKind::Workbench
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Shop {
    pub kind: ShopKind,
    pub spot: (i32, i32),
    pub pads: Vec<(i32, i32)>,
    pub pegs: Vec<(i32, i32)>,
    pub outs: Vec<(i32, i32)>,
    pub quality: f64,
    pub job: Option<Job>,
    /// Finished goods waiting for a free output mat.
    pub pending: Vec<ItemData>,
    pub pad_flash: Vec<u8>,
    pub out_timer: u8,
}

#[derive(Clone, Copy, Debug)]
pub enum PKind {
    Dust,
    Fluff,
    Spark,
    Leaf,
}

#[derive(Clone, Copy, Debug)]
pub struct Particle {
    pub x: i32,
    pub y: i32,
    pub vx: i32,
    pub vy: i32,
    pub life: u8,
    pub kind: PKind,
}

pub struct World {
    pub w: i32,
    pub h: i32,
    pub tiles: Vec<Tile>,
    pub items: Vec<Item>,
    pub plants: Vec<Plant>,
    pub bins: Vec<Bin>,
    pub shops: Vec<Shop>,
    pub particles: Vec<Particle>,
    pub starts: Vec<(i32, i32)>,
    /// Where applicants wait in the hiring hall.
    pub stands: Vec<(i32, i32)>,
    seed: u32,
}

impl World {
    pub fn new(data: &Data) -> Self {
        let rows: Vec<&str> = MAP.lines().filter(|l| !l.is_empty()).collect();
        let (w, h) = (rows[0].len() as i32, rows.len() as i32);
        let mut tiles = Vec::with_capacity((w * h) as usize);
        let (mut plants, mut bins, mut shops, mut starts) = (Vec::new(), Vec::new(), Vec::new(), vec![(0, 0); 2]);
        let mut stands = Vec::new();
        let (mut mouths, mut parts) = (Vec::new(), Vec::new());
        let regrow = data.good("cotton_bolls").production_time;
        for (y, row) in rows.iter().enumerate() {
            assert_eq!(row.len() as i32, w, "map row {y} is {} wide", row.len());
            for (x, ch) in row.chars().enumerate() {
                let (x, y) = (x as i32, y as i32);
                let street_next = row.as_bytes().get(x as usize + 1) == Some(&b',') || (x > 0 && row.as_bytes()[x as usize - 1] == b',');
                let tile = match ch {
                    '#' => Tile::Hedge,
                    ',' | 'm' => Tile::Path,
                    'W' => Tile::Wall,
                    '_' | 'a' => Tile::Floor,
                    'D' => Tile::Door,
                    'p' => Tile::Pad,
                    'g' => Tile::Peg,
                    'o' => Tile::OutMat,
                    'B' | 'S' | 'L' => Tile::Spot,
                    'C' | 'T' => Tile::Bin,
                    '1' | '2' if street_next => Tile::Path,
                    _ => Tile::Grass,
                };
                match ch {
                    'f' => plants.push(Plant { x, y, good: "cotton_bolls".into(), growth: regrow, regrow, stock: Vec::new(), harvest: None, drop: None, timer: 0 }),
                    'C' => bins.push(Bin { x, y, accepts: "cotton_bolls".into(), stock: Vec::new(), mouth: None, timer: 0, bounce: 0, scores: false }),
                    'T' => bins.push(Bin { x, y, accepts: "fabric".into(), stock: Vec::new(), mouth: None, timer: 0, bounce: 0, scores: true }),
                    'a' => stands.push((x, y)),
                    'm' => mouths.push((x, y)),
                    'p' | 'g' | 'o' => parts.push((ch, x, y)),
                    'B' | 'S' | 'L' => {
                        let kind = match ch {
                            'B' => ShopKind::Workbench,
                            'S' => ShopKind::Spinnery,
                            _ => ShopKind::LoomHouse,
                        };
                        shops.push(Shop { kind, spot: (x, y), pads: Vec::new(), pegs: Vec::new(), outs: Vec::new(), quality: TOWN_WORKSHOP_QUALITY, job: None, pending: Vec::new(), pad_flash: Vec::new(), out_timer: 0 })
                    }
                    '1' | '2' => starts[(ch as u8 - b'1') as usize] = (x, y),
                    _ => {}
                }
                tiles.push(tile);
            }
        }
        for (ch, x, y) in parts {
            let shop = shops.iter_mut().min_by_key(|s| (s.spot.0 - x).abs() + (s.spot.1 - y).abs()).expect("a work spot");
            match ch {
                'p' => {
                    shop.pads.push((x, y));
                    shop.pad_flash.push(0);
                }
                'g' => shop.pegs.push((x, y)),
                _ => shop.outs.push((x, y)),
            }
        }
        for m in mouths {
            if let Some(b) = bins.iter_mut().find(|b| (b.x - m.0).abs() + (b.y - m.1).abs() == 1) {
                b.mouth = Some(m);
            }
        }
        Self { w, h, tiles, items: Vec::new(), plants, bins, shops, particles: Vec::new(), starts, stands, seed: 0x9E37_79B9 }
    }

    pub fn tile(&self, x: i32, y: i32) -> Tile {
        if x < 0 || y < 0 || x >= self.w || y >= self.h {
            Tile::Hedge
        } else {
            self.tiles[(y * self.w + x) as usize]
        }
    }

    pub fn item_at(&self, x: i32, y: i32) -> Option<usize> {
        self.items.iter().position(|it| it.x == x && it.y == y)
    }

    pub fn bin_at(&self, x: i32, y: i32) -> Option<usize> {
        self.bins.iter().position(|b| b.x == x && b.y == y)
    }

    pub fn plant_at(&self, x: i32, y: i32) -> Option<usize> {
        self.plants.iter().position(|p| p.x == x && p.y == y)
    }

    pub fn shop_at_spot(&self, x: i32, y: i32) -> Option<usize> {
        self.shops.iter().position(|s| s.spot == (x, y))
    }

    pub fn pad_owner(&self, x: i32, y: i32) -> Option<(usize, usize)> {
        self.shops.iter().enumerate().find_map(|(i, s)| s.pads.iter().position(|&p| p == (x, y)).map(|k| (i, k)))
    }

    /// The tool a peg offers: an item hanging on it, if any.
    pub fn peg_item(&self, x: i32, y: i32) -> Option<usize> {
        self.item_at(x, y)
    }

    /// Does this item's good (with its carried tags) satisfy a tag group?
    pub fn item_matches(&self, data: &Data, item: usize, group: &[String]) -> bool {
        matches(&self.items[item].data.tags(data), group)
    }

    pub fn rand(&mut self, n: u32) -> u32 {
        self.seed ^= self.seed << 13;
        self.seed ^= self.seed >> 17;
        self.seed ^= self.seed << 5;
        self.seed % n.max(1)
    }

    /// A small burst of particles from the center of a tile.
    pub fn burst(&mut self, tx: i32, ty: i32, kind: PKind, n: u32) {
        for _ in 0..n {
            let vx = self.rand(5) as i32 - 2;
            let vy = -(self.rand(3) as i32) - if matches!(kind, PKind::Spark) { 1 } else { 0 };
            let (ox, oy) = (self.rand(9) as i32 - 4, self.rand(7) as i32 - 3);
            let life = 6 + self.rand(8) as u8;
            self.particles.push(Particle { x: tx * TS + 8 + ox, y: ty * TS + 9 + oy, vx, vy, life, kind });
        }
    }

    /// Dust kicked up behind something sliding in `dir`.
    pub fn dust(&mut self, tx: i32, ty: i32, dir: Dir) {
        let (dx, dy) = dir.delta();
        for _ in 0..2 {
            let ox = self.rand(7) as i32 - 3;
            let vx = -dx + self.rand(3) as i32 - 1;
            let life = 4 + self.rand(4) as u8;
            let p = Particle { x: tx * TS + 8 - dx * 7 + ox, y: ty * TS + 14 - dy * 4, vx, vy: -(self.rand(2) as i32), life, kind: PKind::Dust };
            self.particles.push(p);
        }
    }

    pub fn tick_particles(&mut self, frame: u32) {
        for p in self.particles.iter_mut() {
            p.x += p.vx;
            p.y += p.vy;
            p.life = p.life.saturating_sub(1);
            match p.kind {
                PKind::Dust | PKind::Leaf if frame % 2 == 0 => p.vy += 1,
                PKind::Fluff if frame % 3 == 0 => p.vx = (p.vx + 1).min(1),
                PKind::Spark if frame % 2 == 0 => p.vx -= p.vx.signum(),
                _ => {}
            }
        }
        self.particles.retain(|p| p.life > 0);
    }
}
