//! Tile coordinates and directions on the 25 x 17 playfield (16 px tiles on the 400 x 300
//! framebuffer, above a 28 px control strip).

pub const GW: i32 = 25;
pub const GH: i32 = 17;
pub const NT: usize = (GW * GH) as usize;

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, PartialOrd, Ord)]
pub struct Pos {
    pub x: i32,
    pub y: i32,
}

impl Pos {
    pub const fn new(x: i32, y: i32) -> Pos {
        Pos { x, y }
    }

    pub fn ok(self) -> bool {
        self.x >= 0 && self.y >= 0 && self.x < GW && self.y < GH
    }

    pub fn idx(self) -> usize {
        (self.y * GW + self.x) as usize
    }

    pub fn from_idx(i: usize) -> Pos {
        Pos::new(i as i32 % GW, i as i32 / GW)
    }

    pub fn step(self, d: Dir) -> Pos {
        let (dx, dy) = d.delta();
        Pos::new(self.x + dx, self.y + dy)
    }

    pub fn dir_to(self, o: Pos) -> Option<Dir> {
        Dir::ALL.into_iter().find(|&d| self.step(d) == o)
    }

    pub fn manhattan(self, o: Pos) -> i32 {
        (self.x - o.x).abs() + (self.y - o.y).abs()
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Dir {
    N,
    E,
    S,
    W,
}

impl Dir {
    pub const ALL: [Dir; 4] = [Dir::N, Dir::E, Dir::S, Dir::W];

    pub fn bit(self) -> u8 {
        1 << self as u8
    }

    pub fn opp(self) -> Dir {
        Dir::ALL[(self as usize + 2) % 4]
    }

    pub fn delta(self) -> (i32, i32) {
        match self {
            Dir::N => (0, -1),
            Dir::E => (1, 0),
            Dir::S => (0, 1),
            Dir::W => (-1, 0),
        }
    }
}

/// The directions present in an arm mask, in N, E, S, W order.
pub fn arms(mask: u8) -> impl Iterator<Item = Dir> {
    Dir::ALL.into_iter().filter(move |d| mask & d.bit() != 0)
}
