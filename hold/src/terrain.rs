//! The first scenario's marsh (document 05): 100 x 45 squares of 16 px. West is home and east is
//! Gorf. What each square is, what's damp and what's dry, what can be walked or flown, grid
//! pathfinding, and what the Hold has seen.

use std::cmp::Reverse;
use std::collections::BinaryHeap;

pub const MW: i32 = 100;
pub const MH: i32 = 45;
pub const SQ: i32 = 16;

#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash)]
pub enum Tile {
    Water,
    Deep,
    Moss,
    Fern,
    Mud,
    Path,
    Sand,
    Rock,
    Reeds,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash)]
pub enum Deco {
    BigTree,
    Tree,
    Stump,
    Stone,
    LeyStone,
    Lily,
    ReedClump,
    Mushroom,
    Log,
}

/// Fixed places in the first scenario (top-left squares of footprints).
pub mod places {
    pub const HALL: (i32, i32) = (8, 21);
    pub const LONE_NEST: (i32, i32) = (27, 22);
    pub const NEAR_NESTS: [(i32, i32); 2] = [(35, 5), (35, 34)];
    pub const MIDDLE_NESTS: [(i32, i32); 2] = [(49, 13), (54, 30)];
    pub const FAR_NESTS: [(i32, i32); 2] = [(71, 9), (77, 33)];
    pub const ROOST: (i32, i32) = (59, 42);
    pub const HOLLOW: (i32, i32) = (93, 19);
    pub const STAR_POND: (i32, i32) = (9, 7);
    pub const RIDGE: (i32, i32) = (63, 68);
    pub const RIDGE_GAP: (i32, i32) = (19, 23);
}

pub struct Map {
    pub t: Vec<Tile>,
    pub deco: Vec<Option<Deco>>,
    /// Cursed ground: ticks of curse left (u32::MAX is permanent).
    pub curse: Vec<u32>,
    /// A dug puddle (or a goliath's pool, which is 2).
    pub puddle: Vec<u8>,
    /// A leech pool infests this water (its building id + 1, or 0).
    pub leech: Vec<u32>,
    /// A building stands here (its id + 1, or 0).
    pub occ: Vec<u32>,
    pub explored: Vec<bool>,
    pub visible: Vec<bool>,
}

pub fn idx(x: i32, y: i32) -> usize {
    (y * MW + x) as usize
}

pub fn on_map(x: i32, y: i32) -> bool {
    x >= 0 && y >= 0 && x < MW && y < MH
}

pub fn dist(a: (i32, i32), b: (i32, i32)) -> i32 {
    (a.0 - b.0).abs().max((a.1 - b.1).abs())
}

/// A deterministic hash in 0..1000 for a square.
fn h(x: i32, y: i32, salt: i32) -> i32 {
    let mut v = (x as u32).wrapping_mul(374761393) ^ (y as u32).wrapping_mul(668265263) ^ (salt as u32).wrapping_mul(2246822519);
    v = (v ^ (v >> 13)).wrapping_mul(1274126177);
    ((v ^ (v >> 16)) % 1000) as i32
}

/// Smooth-ish value noise in 0..1000 at a cell size of `cell` squares.
fn noise(x: i32, y: i32, cell: i32, salt: i32) -> i32 {
    let (cx, cy) = (x.div_euclid(cell), y.div_euclid(cell));
    let (fx, fy) = (x.rem_euclid(cell), y.rem_euclid(cell));
    let a = h(cx, cy, salt);
    let b = h(cx + 1, cy, salt);
    let c = h(cx, cy + 1, salt);
    let d = h(cx + 1, cy + 1, salt);
    let top = a * (cell - fx) + b * fx;
    let bot = c * (cell - fx) + d * fx;
    (top * (cell - fy) + bot * fy) / (cell * cell)
}

impl Default for Map {
    fn default() -> Self {
        Self::new()
    }
}

impl Map {
    pub fn new() -> Map {
        let n = (MW * MH) as usize;
        let mut m = Map {
            t: vec![Tile::Moss; n],
            deco: vec![None; n],
            curse: vec![0; n],
            puddle: vec![0; n],
            leech: vec![0; n],
            occ: vec![0; n],
            explored: vec![false; n],
            visible: vec![false; n],
        };
        m.generate();
        m
    }

    pub fn at(&self, x: i32, y: i32) -> Tile {
        if on_map(x, y) {
            self.t[idx(x, y)]
        } else {
            Tile::Rock
        }
    }

    fn set(&mut self, x: i32, y: i32, t: Tile) {
        if on_map(x, y) {
            self.t[idx(x, y)] = t;
        }
    }

    fn blob(&mut self, cx: i32, cy: i32, rx: i32, ry: i32, t: Tile, ragged: i32, salt: i32) {
        for y in cy - ry - 2..=cy + ry + 2 {
            for x in cx - rx - 2..=cx + rx + 2 {
                let dx = (x - cx) * 1000 / rx.max(1);
                let dy = (y - cy) * 1000 / ry.max(1);
                let r = dx * dx / 1000 + dy * dy / 1000;
                if r <= 1000 + (h(x, y, salt) - 500) * ragged / 500 {
                    self.set(x, y, t);
                }
            }
        }
    }

    /// A stream two squares wide along a polyline, with mud banks.
    fn stream(&mut self, pts: &[(i32, i32)]) {
        for w in pts.windows(2) {
            let ((x0, y0), (x1, y1)) = (w[0], w[1]);
            let steps = (x1 - x0).abs().max((y1 - y0).abs()).max(1);
            for s in 0..=steps {
                let x = x0 + (x1 - x0) * s / steps;
                let y = y0 + (y1 - y0) * s / steps;
                for (dx, dy) in [(-2, 0), (2, 0), (0, -2), (0, 2), (-1, -1), (1, 1), (-1, 1), (1, -1)] {
                    if matches!(self.at(x + dx, y + dy), Tile::Moss | Tile::Fern | Tile::Path) {
                        self.set(x + dx, y + dy, Tile::Mud);
                    }
                }
                for (dx, dy) in [(0, 0), (1, 0), (0, 1), (1, 1), (-1, 0), (0, -1)] {
                    if self.at(x + dx, y + dy) != Tile::Rock {
                        self.set(x + dx, y + dy, Tile::Water);
                    }
                }
            }
        }
    }

    fn path(&mut self, pts: &[(i32, i32)]) {
        for w in pts.windows(2) {
            let ((x0, y0), (x1, y1)) = (w[0], w[1]);
            let steps = (x1 - x0).abs().max((y1 - y0).abs()).max(1);
            for s in 0..=steps {
                let x = x0 + (x1 - x0) * s / steps;
                let y = y0 + (y1 - y0) * s / steps;
                if matches!(self.at(x, y), Tile::Moss | Tile::Fern | Tile::Mud) {
                    self.set(x, y, Tile::Path);
                }
            }
        }
    }

    fn generate(&mut self) {
        use places::*;
        // Fern ground in the damp west, moss elsewhere, open ground scattered in the east.
        for y in 0..MH {
            for x in 0..MW {
                let n = noise(x, y, 5, 1);
                let t = if x < 34 && n > 560 {
                    Tile::Fern
                } else if x > 70 && n < 300 {
                    Tile::Path
                } else {
                    Tile::Moss
                };
                self.set(x, y, t);
            }
        }
        // The cliffs along the north-west, with a ragged foot.
        for x in 0..38 {
            let foot = 3 + (noise(x, 0, 4, 2) > 520) as i32 + (x > 30) as i32 * ((x - 30) / 3).min(2);
            for y in 0..=foot {
                self.set(x, y, Tile::Rock);
            }
        }
        // The bog: open water, reeds round it, mud between.
        for y in 0..MH {
            for x in 34..63 {
                let n = noise(x, y, 6, 3);
                if n > 600 {
                    self.set(x, y, Tile::Water);
                } else if n > 520 {
                    self.set(x, y, Tile::Reeds);
                } else if n > 470 {
                    self.set(x, y, Tile::Mud);
                }
            }
        }
        // Gorf's side: some water and reeds among moss and dry ground.
        for y in 0..MH {
            for x in 69..MW {
                let n = noise(x, y, 5, 4);
                if n > 700 {
                    self.set(x, y, Tile::Water);
                } else if n > 640 {
                    self.set(x, y, Tile::Reeds);
                }
            }
        }
        // Streams: from the cliffs down past the Hall, south into the old growth and east to the bog.
        self.stream(&[(19, 4), (18, 10), (15, 16), (13, 20), (12, 24), (13, 30), (18, 36), (27, 39), (36, 38)]);
        self.stream(&[(3, 6), (4, 14), (6, 20), (8, 24)]);
        // The Hall's pool, under and in front of it.
        self.blob(10, 25, 3, 1, Tile::Water, 120, 5);
        // The Star Pond: deep water ringed with water, north of the Hall.
        self.blob(10, 9, 4, 3, Tile::Water, 150, 6);
        self.blob(10, 9, 3, 2, Tile::Deep, 0, 7);
        // Ponds round the nests, so herons stand in water.
        for p in NEAR_NESTS.iter().chain(MIDDLE_NESTS.iter()).chain(FAR_NESTS.iter()).chain([LONE_NEST].iter()) {
            self.blob(p.0 + 1, p.1 + 1, 3, 2, Tile::Water, 200, p.0 + p.1);
            for (dx, dy) in [(-3, 0), (3, 0), (0, -3), (0, 3)] {
                if self.at(p.0 + 1 + dx, p.1 + 1 + dy) == Tile::Moss {
                    self.set(p.0 + 1 + dx, p.1 + 1 + dy, Tile::Reeds);
                }
            }
        }
        // The sand ridge, north to south, with one gap of reeds.
        for y in 0..MH {
            let w = 2 + (noise(0, y, 4, 8) * 4 / 1000);
            for x in RIDGE.0..RIDGE.0 + w {
                if y >= RIDGE_GAP.0 && y <= RIDGE_GAP.1 {
                    self.set(x, y, Tile::Reeds);
                } else {
                    self.set(x, y, Tile::Sand);
                }
            }
        }
        // Paths: dry ground from the Hall east and north (open, so drying).
        self.path(&[(12, 23), (20, 23), (26, 21), (30, 20)]);
        self.path(&[(10, 20), (12, 15), (16, 12), (20, 9)]);
        self.path(&[(10, 24), (8, 30), (10, 36)]);
        // Rock round Gorf's Hollow, and its curse.
        for y in HOLLOW.1 - 6..HOLLOW.1 + 9 {
            for x in HOLLOW.0 - 6..MW {
                if on_map(x, y) && dist((x, y), (HOLLOW.0 + 2, HOLLOW.1 + 1)) <= 6 {
                    self.curse[idx(x, y)] = u32::MAX;
                    if self.t[idx(x, y)] == Tile::Water || self.t[idx(x, y)] == Tile::Reeds {
                        self.t[idx(x, y)] = Tile::Mud;
                    }
                }
            }
        }
        // The old growth to the south: big trees, trees and the ley stones.
        for y in 31..MH {
            for x in 0..33 {
                if self.at(x, y) == Tile::Water || self.at(x, y) == Tile::Mud {
                    continue;
                }
                let r = h(x, y, 9);
                if r < 70 && x % 3 == 0 && y % 3 == 1 {
                    self.deco[idx(x, y)] = Some(Deco::BigTree);
                } else if r < 120 {
                    self.deco[idx(x, y)] = Some(Deco::Tree);
                } else if r < 140 {
                    self.deco[idx(x, y)] = Some(Deco::Mushroom);
                }
            }
        }
        for p in [(18, 38), (23, 40), (20, 42)] {
            self.deco[idx(p.0, p.1)] = Some(Deco::LeyStone);
        }
        // Scattered trees and things elsewhere, kept off the water and the Hold's own ground.
        for y in 0..MH {
            for x in 0..MW {
                let i = idx(x, y);
                if self.deco[i].is_some() || self.t[i] == Tile::Rock {
                    continue;
                }
                let r = h(x, y, 10);
                let near_hall = dist((x, y), (HALL.0 + 2, HALL.1 + 1)) < 9;
                let d = match self.t[i] {
                    Tile::Water if r < 60 => Some(Deco::Lily),
                    Tile::Reeds if r < 250 => Some(Deco::ReedClump),
                    Tile::Moss | Tile::Fern if !near_hall && r < 25 => Some(Deco::Tree),
                    Tile::Moss | Tile::Fern if !near_hall && r < 40 => Some(Deco::Stone),
                    Tile::Moss | Tile::Fern if r < 55 => Some(Deco::Mushroom),
                    Tile::Moss | Tile::Mud if !near_hall && r < 62 => Some(Deco::Log),
                    Tile::Moss if !near_hall && r < 70 => Some(Deco::Stump),
                    _ => None,
                };
                self.deco[i] = d;
            }
        }
        // Keep the fixed places clear.
        let mut clear = vec![(HALL, (4, 3)), (ROOST, (2, 2)), (HOLLOW, (4, 3)), (LONE_NEST, (2, 2))];
        for p in NEAR_NESTS.iter().chain(MIDDLE_NESTS.iter()).chain(FAR_NESTS.iter()) {
            clear.push((*p, (2, 2)));
        }
        for ((x0, y0), (w, hh)) in clear {
            for y in y0 - 1..y0 + hh + 1 {
                for x in x0 - 1..x0 + w + 1 {
                    if on_map(x, y) {
                        self.deco[idx(x, y)] = None;
                    }
                }
            }
        }
    }

    pub fn cursed(&self, x: i32, y: i32) -> bool {
        on_map(x, y) && self.curse[idx(x, y)] > 0
    }

    /// Water that clears Dry: the stream, ponds, pools and puddles.
    pub fn wet(&self, x: i32, y: i32) -> bool {
        on_map(x, y)
            && !self.cursed(x, y)
            && (matches!(self.t[idx(x, y)], Tile::Water | Tile::Deep) || self.puddle[idx(x, y)] > 0)
    }

    /// Damp ground stops Dry and lets spots ripen.
    pub fn damp(&self, x: i32, y: i32) -> bool {
        if !on_map(x, y) || self.cursed(x, y) {
            return false;
        }
        match self.t[idx(x, y)] {
            Tile::Moss | Tile::Fern | Tile::Mud | Tile::Reeds | Tile::Water | Tile::Deep => true,
            Tile::Rock => true,
            Tile::Path | Tile::Sand => self.near_puddle(x, y),
        }
    }

    fn near_puddle(&self, x: i32, y: i32) -> bool {
        for dy in -1..=1 {
            for dx in -1..=1 {
                if on_map(x + dx, y + dy) && self.puddle[idx(x + dx, y + dy)] > 0 {
                    return true;
                }
            }
        }
        false
    }

    pub fn blocks_walk(&self, x: i32, y: i32) -> bool {
        if !on_map(x, y) {
            return true;
        }
        let i = idx(x, y);
        self.t[i] == Tile::Rock || self.occ[i] != 0 || matches!(self.deco[i], Some(Deco::BigTree) | Some(Deco::Tree))
    }

    /// Ticks to step onto a square, for a walker (flyers always take their base).
    pub fn step_cost(&self, x: i32, y: i32) -> i32 {
        match self.at(x, y) {
            Tile::Reeds => 15,
            Tile::Mud | Tile::Deep => 12,
            _ => 10,
        }
    }

    /// Squares on a straight line between two squares that are dry ground (for asking prices).
    pub fn dry_between(&self, a: (i32, i32), b: (i32, i32)) -> i32 {
        let steps = dist(a, b).max(1);
        let mut n = 0;
        for s in 0..=steps {
            let x = a.0 + (b.0 - a.0) * s / steps;
            let y = a.1 + (b.1 - a.1) * s / steps;
            if !self.damp(x, y) {
                n += 1;
            }
        }
        n
    }

    /// A* over the grid, 8 directions. Returns the squares to step through (not including the
    /// start), ending next to `to` when `adjacent` (for buildings and creatures), or None.
    pub fn find_path(&self, from: (i32, i32), to: (i32, i32), adjacent: i32, flies: bool, limit: usize) -> Option<Vec<(i32, i32)>> {
        if dist(from, to) <= adjacent {
            return Some(Vec::new());
        }
        let n = (MW * MH) as usize;
        let mut g = vec![i32::MAX; n];
        let mut came = vec![u32::MAX; n];
        let mut open = BinaryHeap::new();
        let s = idx(from.0, from.1);
        g[s] = 0;
        open.push(Reverse((dist(from, to) * 10, s as u32)));
        let mut expanded = 0usize;
        let mut best = (i32::MAX, s);
        while let Some(Reverse((_, cur))) = open.pop() {
            let cur = cur as usize;
            let (cx, cy) = ((cur as i32) % MW, (cur as i32) / MW);
            let dd = dist((cx, cy), to);
            if dd <= adjacent {
                return Some(self.unwind(&came, s, cur));
            }
            if dd < best.0 {
                best = (dd, cur);
            }
            expanded += 1;
            if expanded > limit {
                break;
            }
            for (dx, dy) in [(1, 0), (-1, 0), (0, 1), (0, -1), (1, 1), (1, -1), (-1, 1), (-1, -1)] {
                let (nx, ny) = (cx + dx, cy + dy);
                if !on_map(nx, ny) {
                    continue;
                }
                let goal = dist((nx, ny), to) <= adjacent;
                if !flies && self.blocks_walk(nx, ny) && !goal {
                    continue;
                }
                if !flies && dx != 0 && dy != 0 && (self.blocks_walk(cx + dx, cy) || self.blocks_walk(cx, cy + dy)) {
                    continue;
                }
                let ni = idx(nx, ny);
                let step = if flies { 10 } else { self.step_cost(nx, ny) };
                let ng = g[cur] + step;
                if ng < g[ni] {
                    g[ni] = ng;
                    came[ni] = cur as u32;
                    open.push(Reverse((ng + dist((nx, ny), to) * 10, ni as u32)));
                }
            }
        }
        // Couldn't reach it: go as near as we got.
        if best.1 != s {
            Some(self.unwind(&came, s, best.1))
        } else {
            None
        }
    }

    fn unwind(&self, came: &[u32], s: usize, mut cur: usize) -> Vec<(i32, i32)> {
        let mut out = Vec::new();
        while cur != s {
            out.push(((cur as i32) % MW, (cur as i32) / MW));
            cur = came[cur] as usize;
        }
        out.reverse();
        out
    }

    /// Whether a footprint can take a building: free, on the map, and on the right ground.
    pub fn can_place(&self, x: i32, y: i32, w: i32, hh: i32, place: crate::data::Place) -> bool {
        use crate::data::Place;
        for yy in y..y + hh {
            for xx in x..x + w {
                if !on_map(xx, yy) {
                    return false;
                }
                let i = idx(xx, yy);
                if self.occ[i] != 0 || self.t[i] == Tile::Rock || matches!(self.deco[i], Some(Deco::BigTree) | Some(Deco::Tree)) {
                    return false;
                }
                if place != Place::Pond && matches!(self.t[i], Tile::Water | Tile::Deep) {
                    return false;
                }
            }
        }
        match place {
            Place::Anywhere => true,
            // A cave needs rock all along the row above it.
            Place::Rock => (x..x + w).all(|xx| self.at(xx, y - 1) == Tile::Rock),
            // The Star Pond needs the deep pond under part of it.
            Place::Pond => {
                let mut deep = 0;
                for yy in y..y + hh {
                    for xx in x..x + w {
                        if self.at(xx, yy) == Tile::Deep {
                            deep += 1;
                        }
                    }
                }
                deep >= 2
            }
        }
    }

    pub fn reveal(&mut self, c: (i32, i32), r: i32) {
        for y in c.1 - r..=c.1 + r {
            for x in c.0 - r..=c.0 + r {
                if on_map(x, y) && (x - c.0) * (x - c.0) + (y - c.1) * (y - c.1) <= r * r + r {
                    let i = idx(x, y);
                    self.explored[i] = true;
                    self.visible[i] = true;
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::Place;

    #[test]
    fn the_hall_can_reach_every_nest_and_the_hollow() {
        let m = Map::new();
        let hall = (places::HALL.0 + 2, places::HALL.1 + 3);
        for p in places::NEAR_NESTS.iter().chain(places::FAR_NESTS.iter()).chain([places::HOLLOW].iter()) {
            let path = m.find_path(hall, *p, 2, false, 20000).expect("a path");
            let end = *path.last().unwrap();
            assert!(dist(end, *p) <= 2, "reached {p:?}, got to {end:?}");
        }
    }

    #[test]
    fn there_are_places_for_caves_and_the_star_pond() {
        let m = Map::new();
        let caves = (0..MW).flat_map(|x| (0..MH).map(move |y| (x, y))).filter(|&(x, y)| m.can_place(x, y, 3, 2, Place::Rock)).count();
        assert!(caves >= 6, "cave sites: {caves}");
        let pond = (0..MW).flat_map(|x| (0..MH).map(move |y| (x, y))).filter(|&(x, y)| m.can_place(x, y, 3, 3, Place::Pond)).count();
        assert!(pond >= 1, "star pond sites: {pond}");
    }

    #[test]
    fn the_ridge_is_dry_and_the_underbrush_damp() {
        let m = Map::new();
        assert!(!m.damp(places::RIDGE.0, 5));
        assert!(m.damp(places::HALL.0 - 2, places::HALL.1));
    }
}
