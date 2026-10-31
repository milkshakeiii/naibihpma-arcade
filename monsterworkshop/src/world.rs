//! The yard as data: a tile map plus the things that sit on it. No monsters here; `game.rs`
//! moves them and asks the world what is where.

pub const TS: i32 = 16;
pub const MAP_W: i32 = 25;
pub const MAP_H: i32 = 16;
pub const REGROW_TICKS: u16 = 90; // 6 s at 15 FPS
pub const CRAFT_TICKS: u16 = 45; // 3 s
pub const DISPENSE_TICKS: u8 = 4;

// `#` hedge  `.` grass  `,` path  `W` workshop wall  `_` floor  `D` door  `p` input pad
// `o` output mat  `S` spinning wheel  `f` cotton plant  `C` cotton bin  `m` its chute
// `T` thread bin  `1` `2` monster starts
const MAP: [&str; MAP_H as usize] = [
    "#########################",
    "#.......................#",
    "#.......................#",
    "#.......................#",
    "#.......................#",
    "#........WWWWWWWWW......#",
    "#.fff....W_______W......#",
    "#.fff..C.WS_o____D,T....#",
    "#.fff.1m,D_pp____W......#",
    "#.....,,,D_______W......#",
    "#2.......W_______W......#",
    "#........WWWWWWWWW......#",
    "#.......................#",
    "#.......................#",
    "#.......................#",
    "#########################",
];

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
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
    OutMat,
    Wheel,
    Bin,
}

impl Tile {
    pub fn walkable(self) -> bool {
        !matches!(self, Tile::Hedge | Tile::Wall | Tile::Bin)
    }

    /// Items slide over anything a monster can walk on except the wheel itself.
    pub fn item_floor(self) -> bool {
        self.walkable() && self != Tile::Wheel
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Kind {
    Cotton,
    Thread,
}

/// A move in progress, drawn as an offset back toward the tile it came from.
#[derive(Clone, Copy, Debug)]
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

#[derive(Clone, Debug)]
pub struct Item {
    pub x: i32,
    pub y: i32,
    pub kind: Kind,
    pub slide: Option<Slide>,
    pub hop: u8,     // pop arc after harvest / dispense
    pub locked: bool, // on a pad that is being spun
}

#[derive(Clone, Debug)]
pub struct Plant {
    pub x: i32,
    pub y: i32,
    pub growth: u16,
}

impl Plant {
    pub fn ripe(&self) -> bool {
        self.growth >= REGROW_TICKS
    }

    pub fn stage(&self) -> u8 {
        if self.ripe() {
            2
        } else if self.growth >= REGROW_TICKS / 2 {
            1
        } else {
            0
        }
    }
}

#[derive(Clone, Debug)]
pub struct Bin {
    pub x: i32,
    pub y: i32,
    pub kind: Kind,
    pub count: u32,
    pub mouth: Option<(i32, i32)>,
    pub timer: u8,
    pub bounce: u8,
}

#[derive(Clone, Debug)]
pub struct Shop {
    pub wheel: (i32, i32),
    pub pads: Vec<(i32, i32)>,
    pub out: (i32, i32),
    pub crafting: Option<(usize, u16)>, // (monster, ticks spun)
    pub pad_flash: Vec<u8>,
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
    pub tiles: Vec<Tile>,
    pub items: Vec<Item>,
    pub plants: Vec<Plant>,
    pub bins: Vec<Bin>,
    pub shop: Shop,
    pub particles: Vec<Particle>,
    pub starts: Vec<(i32, i32)>,
    seed: u32,
}

impl World {
    pub fn new() -> Self {
        let mut tiles = Vec::with_capacity((MAP_W * MAP_H) as usize);
        let (mut plants, mut bins, mut starts) = (Vec::new(), Vec::new(), vec![(0, 0); 2]);
        let mut shop = Shop { wheel: (0, 0), pads: Vec::new(), out: (0, 0), crafting: None, pad_flash: Vec::new() };
        let mut mouths = Vec::new();
        for (y, row) in MAP.iter().enumerate() {
            assert_eq!(row.len() as i32, MAP_W, "map row {y} is {} wide", row.len());
            for (x, ch) in row.chars().enumerate() {
                let (x, y) = (x as i32, y as i32);
                let tile = match ch {
                    '#' => Tile::Hedge,
                    ',' | 'm' => Tile::Path,
                    'W' => Tile::Wall,
                    '_' => Tile::Floor,
                    'D' => Tile::Door,
                    'p' => Tile::Pad,
                    'o' => Tile::OutMat,
                    'S' => Tile::Wheel,
                    'C' | 'T' => Tile::Bin,
                    _ => Tile::Grass,
                };
                match ch {
                    'f' => plants.push(Plant { x, y, growth: REGROW_TICKS }),
                    'C' => bins.push(Bin { x, y, kind: Kind::Cotton, count: 4, mouth: None, timer: 0, bounce: 0 }),
                    'T' => bins.push(Bin { x, y, kind: Kind::Thread, count: 0, mouth: None, timer: 0, bounce: 0 }),
                    'm' => mouths.push((x, y)),
                    'p' => {
                        shop.pads.push((x, y));
                        shop.pad_flash.push(0);
                    }
                    'o' => shop.out = (x, y),
                    'S' => shop.wheel = (x, y),
                    '1' | '2' => starts[(ch as u8 - b'1') as usize] = (x, y),
                    _ => {}
                }
                tiles.push(if ch == '1' { Tile::Path } else { tile });
            }
        }
        // Each chute starts with its first item already out, so the yard opens ready to push.
        let mut items = Vec::new();
        for m in mouths {
            if let Some(b) = bins.iter_mut().find(|b| (b.x - m.0).abs() + (b.y - m.1).abs() == 1) {
                b.mouth = Some(m);
                if b.count > 0 {
                    b.count -= 1;
                    items.push(Item { x: m.0, y: m.1, kind: b.kind, slide: None, hop: 0, locked: false });
                }
            }
        }
        Self { tiles, items, plants, bins, shop, particles: Vec::new(), starts, seed: 0x9E37_79B9 }
    }

    pub fn tile(&self, x: i32, y: i32) -> Tile {
        if x < 0 || y < 0 || x >= MAP_W || y >= MAP_H {
            Tile::Hedge
        } else {
            self.tiles[(y * MAP_W + x) as usize]
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

    pub fn pads_full(&self) -> bool {
        self.shop.pads.iter().all(|&(x, y)| self.item_at(x, y).is_some_and(|i| self.items[i].kind == Kind::Cotton))
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
