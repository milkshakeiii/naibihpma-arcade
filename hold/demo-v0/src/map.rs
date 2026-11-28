//! The one-screen marsh: 25 x 15 tiles of 16 px (the play area is the top 400 x 240 of the
//! framebuffer). Tiles, what can be walked, grid pathfinding, and the painted background.

use crate::art::*;

pub const TILE: i32 = 16;
pub const MW: i32 = 25;
pub const MH: i32 = 15;
pub const PLAY_H: i32 = MH * TILE;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Tile {
    Moss,
    Fern,
    Path,
    Tree,
    Water,
    Bridge,
    Pond,
    Curse,
}

/// The marsh. West of the stream is the hold; the herons nest in the east.
const LAYOUT: [&str; 15] = [
    "TTTTTTTTTTTWWTTTTTTTTTTTT",
    "T..,......TWW..,.......TT",
    "T.........TWW.......T...T",
    "T...:::::::==::::.......T",
    "T...:......WW....TT.....T",
    "T...:....,.WW....TT...,.T",
    "T...:......WW...........T",
    "T..::......WW......,....T",
    "T...:......WW..T........T",
    "T...:......WW..TT.......T",
    "T...:...,..WW...........T",
    "T...:::::::==:::::......T",
    "T.PP.......WW......T....T",
    "T.PP...,..TWW...........T",
    "TTTTTTTTTTTWWTTTTTTTTTTTT",
];

pub struct Map {
    pub t: Vec<Tile>,
    /// Tiles taken by buildings, nests and the like.
    pub occupied: Vec<bool>,
}

impl Default for Map {
    fn default() -> Self {
        Self::new()
    }
}

impl Map {
    pub fn new() -> Map {
        let mut t = Vec::with_capacity((MW * MH) as usize);
        for row in LAYOUT.iter() {
            assert_eq!(row.len(), MW as usize, "map row {row}");
            for ch in row.chars() {
                t.push(match ch {
                    'T' => Tile::Tree,
                    'W' => Tile::Water,
                    '=' => Tile::Bridge,
                    'P' => Tile::Pond,
                    ':' => Tile::Path,
                    ',' => Tile::Fern,
                    _ => Tile::Moss,
                });
            }
        }
        Map {
            t,
            occupied: vec![false; (MW * MH) as usize],
        }
    }

    pub fn ok(x: i32, y: i32) -> bool {
        x >= 0 && y >= 0 && x < MW && y < MH
    }

    pub fn at(&self, x: i32, y: i32) -> Tile {
        if Self::ok(x, y) {
            self.t[(y * MW + x) as usize]
        } else {
            Tile::Tree
        }
    }

    pub fn set(&mut self, x: i32, y: i32, tile: Tile) {
        if Self::ok(x, y) {
            self.t[(y * MW + x) as usize] = tile;
        }
    }

    pub fn ground(&self, x: i32, y: i32) -> bool {
        matches!(
            self.at(x, y),
            Tile::Moss | Tile::Fern | Tile::Path | Tile::Bridge | Tile::Curse
        )
    }

    pub fn walkable(&self, x: i32, y: i32) -> bool {
        self.ground(x, y) && !self.occupied[(y * MW + x) as usize]
    }

    pub fn occupy(&mut self, tx: i32, ty: i32, w: i32, h: i32, on: bool) {
        for y in ty..ty + h {
            for x in tx..tx + w {
                if Self::ok(x, y) {
                    self.occupied[(y * MW + x) as usize] = on;
                }
            }
        }
    }

    /// Can a 2 x 2 building go with its top-left tile here? Only on the hold's side of the stream.
    pub fn buildable(&self, tx: i32, ty: i32) -> bool {
        tx >= 1 && tx + 1 <= 10 && (0..2).all(|dy| (0..2).all(|dx| self.walkable(tx + dx, ty + dy)))
    }

    /// Shortest 8-way path (no corner cutting) from `from` to any tile where `goal` holds.
    /// Returns the tiles to walk through, excluding `from`.
    pub fn path(
        &self,
        from: (i32, i32),
        goal: &dyn Fn(i32, i32) -> bool,
    ) -> Option<Vec<(i32, i32)>> {
        let n = (MW * MH) as usize;
        let idx = |x: i32, y: i32| (y * MW + x) as usize;
        let mut dist = vec![i32::MAX; n];
        let mut prev = vec![usize::MAX; n];
        let mut done = vec![false; n];
        if !Self::ok(from.0, from.1) {
            return None;
        }
        dist[idx(from.0, from.1)] = 0;
        loop {
            let mut best = usize::MAX;
            let mut bd = i32::MAX;
            for i in 0..n {
                if !done[i] && dist[i] < bd {
                    bd = dist[i];
                    best = i;
                }
            }
            if best == usize::MAX {
                return None;
            }
            done[best] = true;
            let (x, y) = (best as i32 % MW, best as i32 / MW);
            if goal(x, y) {
                let mut out = vec![];
                let mut c = best;
                while c != idx(from.0, from.1) {
                    out.push((c as i32 % MW, c as i32 / MW));
                    c = prev[c];
                }
                out.reverse();
                return Some(out);
            }
            for (dx, dy) in [
                (1, 0),
                (-1, 0),
                (0, 1),
                (0, -1),
                (1, 1),
                (1, -1),
                (-1, 1),
                (-1, -1),
            ] {
                let (nx, ny) = (x + dx, y + dy);
                if !Self::ok(nx, ny) || !self.walkable(nx, ny) {
                    continue;
                }
                if dx != 0 && dy != 0 && (!self.walkable(x + dx, y) || !self.walkable(x, y + dy)) {
                    continue;
                }
                let nd = bd + if dx != 0 && dy != 0 { 14 } else { 10 };
                let j = idx(nx, ny);
                if nd < dist[j] {
                    dist[j] = nd;
                    prev[j] = best;
                }
            }
        }
    }

    /// The painted background of the play area: 400 x 240 palette indices.
    pub fn paint(&self, art: &Art) -> Vec<u8> {
        let mut bg = Sprite::new(MW * TILE, PLAY_H);
        for ty in 0..MH {
            for tx in 0..MW {
                self.paint_ground(&mut bg, tx, ty, art);
            }
        }
        // Tree canopies overlap their neighbours, so the forest reads as one dark mass.
        for ty in 0..MH {
            for tx in 0..MW {
                if self.at(tx, ty) == Tile::Tree {
                    let open_below = self.at(tx, ty + 1) != Tile::Tree && ty + 1 < MH;
                    paint_tree(&mut bg, tx, ty, open_below);
                }
            }
        }
        bg.px
    }

    fn paint_ground(&self, bg: &mut Sprite, tx: i32, ty: i32, art: &Art) {
        let (x0, y0) = (tx * TILE, ty * TILE);
        let tile = self.at(tx, ty);
        for y in 0..TILE {
            for x in 0..TILE {
                let h = hash(x0 + x, y0 + y);
                // Gorf's curse creeps out from his stone with a ragged edge.
                let (cdx, cdy) = (x0 + x - 336, y0 + y - 158);
                let reach = 22 + (hash((x0 + x) / 3, (y0 + y) / 3) % 12) as i32;
                let cursed = tile == Tile::Curse && cdx * cdx + cdy * cdy * 3 / 2 < reach * reach;
                let tile = if tile == Tile::Curse && !cursed {
                    Tile::Moss
                } else {
                    tile
                };
                let c = match tile {
                    Tile::Moss | Tile::Fern | Tile::Tree => {
                        let clump = hash((x0 + x) / 5 + 3, (y0 + y) / 4 + 11);
                        let big = hash((x0 + x) / 23, (y0 + y) / 19);
                        if h % 13 == 0 {
                            MOSS0
                        } else if clump % 5 == 0 || (big % 4 == 0 && clump % 3 == 0) {
                            MOSS2
                        } else if clump % 7 == 0 {
                            MOSS0
                        } else if h % 61 == 0 {
                            MOSS3
                        } else {
                            MOSS1
                        }
                    }
                    Tile::Path => {
                        if h % 5 == 0 {
                            PATH1
                        } else if h % 13 == 0 {
                            MOSS1
                        } else {
                            PATH0
                        }
                    }
                    Tile::Water | Tile::Pond => {
                        let band = (y0 + y + (x0 + x) / 5) % 6;
                        if h % 41 == 0 {
                            WGLINT[(h / 41 % 3) as usize]
                        } else if band == 0 {
                            WATER1
                        } else {
                            WATER0
                        }
                    }
                    Tile::Bridge => {
                        if y == 0 || y == TILE - 1 {
                            LOG0
                        } else if x % 4 == 3 {
                            INK
                        } else if h % 6 == 0 {
                            LOG0
                        } else {
                            LOG1
                        }
                    }
                    Tile::Curse => {
                        let edge = cdx * cdx + cdy * cdy * 3 / 2 > (reach - 6) * (reach - 6);
                        if edge && (x + y) & 1 == 0 {
                            MOSS0
                        } else if (x + y) & 1 == 0 && h % 3 != 0 {
                            CURSE1
                        } else if h % 23 == 0 {
                            WISP0
                        } else {
                            CURSE0
                        }
                    }
                };
                bg.set(x0 + x, y0 + y, c);
            }
        }
        if matches!(tile, Tile::Moss | Tile::Fern) {
            let d = hash(tx * 7 + 1, ty * 13 + 5);
            if d % 3 == 0 {
                let (gx, gy) = (x0 + 2 + (d / 3 % 11) as i32, y0 + 4 + (d / 33 % 9) as i32);
                for (dx, dy, c) in [
                    (0, 0, FERN0),
                    (1, -1, FERN1),
                    (2, 0, FERN0),
                    (3, -2, FERN1),
                    (4, 0, FERN0),
                ] {
                    bg.set(gx + dx, gy + dy, c);
                }
            }
            if d % 11 == 4 {
                let (mx, my) = (x0 + 3 + (d / 7 % 9) as i32, y0 + 3 + (d / 77 % 9) as i32);
                bg.set(mx, my + 2, CREAM);
                bg.set(mx - 1, my + 1, CAP1);
                bg.set(mx, my + 1, CAP2);
                bg.set(mx + 1, my + 1, CAP1);
                bg.set(mx, my, CAP1);
            }
            if d % 5 == 2 {
                let (sx, sy) = (x0 + 1 + (d / 5 % 13) as i32, y0 + 1 + (d / 65 % 13) as i32);
                bg.set(sx, sy, SPORE[(d % 3) as usize]);
                bg.set(sx + 2, sy + 1, SPORE[((d + 1) % 3) as usize]);
                bg.set(sx + 1, sy + 3, SPORE[((d + 2) % 3) as usize]);
            }
        }
        match tile {
            Tile::Fern => blit_into(bg, &art.fern, x0 + 3, y0 + 5),
            Tile::Pond => {
                for k in 0..3 {
                    let h = hash(tx * 31 + k, ty * 17 + k);
                    bg.set(
                        x0 + (h % 14) as i32 + 1,
                        y0 + (h / 14 % 14) as i32 + 1,
                        STAR,
                    );
                }
                if (tx + ty) % 2 == 0 {
                    blit_into(bg, &art.lily, x0 + 5, y0 + 9);
                }
            }
            Tile::Water if hash(tx, ty) % 5 == 0 => blit_into(bg, &art.lily, x0 + 4, y0 + 4),
            _ => {}
        }
        // A dark lip where land meets water, so the banks read.
        if matches!(tile, Tile::Water | Tile::Pond) {
            for x in 0..TILE {
                if !matches!(
                    self.at(tx, ty - 1),
                    Tile::Water | Tile::Pond | Tile::Bridge | Tile::Tree
                ) {
                    bg.set(x0 + x, y0, INK);
                }
            }
        }
    }
}

fn paint_tree(bg: &mut Sprite, tx: i32, ty: i32, open_below: bool) {
    let h = hash(tx, ty);
    let (cx, cy) = (
        tx * TILE + 8 + (h % 5) as i32 - 2,
        ty * TILE + 7 + (h / 5 % 5) as i32 - 2,
    );
    let r = 8 + (h / 25 % 4) as i32;
    if open_below {
        bg.rect(cx - 1, cy + r - 2, 3, 5, TRUNK);
        bg.set(cx - 1, cy + r + 2, INK);
        bg.set(cx + 1, cy + r + 2, INK);
    }
    bg.ellipse(cx, cy + 2, r, r - 1, TREE0);
    bg.ellipse(cx - 1, cy, r - 1, r - 2, TREE1);
    bg.dither_in(cx, cy + 2, r - 1, r - 2, TREE1, TREE0);
    bg.ellipse(cx - 2, cy - 2, r - 3, r - 4, TREE1);
    bg.ellipse(cx - 3, cy - 3, r - 5, r - 6, TREE2);
    bg.dither_in(cx - 3, cy - 3, r - 4, r - 5, TREE1, TREE2);
    bg.ellipse(cx - 4, cy - 5, 2, 1, TREE3);
    if h % 3 == 0 {
        bg.set(cx + 3, cy - 4, TREE3);
    }
}

fn blit_into(bg: &mut Sprite, s: &Sprite, x: i32, y: i32) {
    for sy in 0..s.h {
        for sx in 0..s.w {
            let c = s.get(sx, sy);
            if c != 0 {
                bg.set(x + sx, y + sy, c);
            }
        }
    }
}

/// A small integer hash for texture noise.
pub fn hash(x: i32, y: i32) -> u32 {
    let mut h = (x as u32).wrapping_mul(374_761_393) ^ (y as u32).wrapping_mul(668_265_263);
    h = (h ^ (h >> 13)).wrapping_mul(1_274_126_177);
    h ^ (h >> 16)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_bridges_join_both_banks() {
        let m = Map::new();
        let p = m.path((5, 6), &|x, y| x == 20 && y == 6);
        assert!(p.is_some());
        assert!(p.unwrap().iter().any(|&(x, y)| m.at(x, y) == Tile::Bridge));
    }

    #[test]
    fn buildings_only_go_on_the_hold_side() {
        let m = Map::new();
        assert!(m.buildable(5, 5));
        assert!(!m.buildable(14, 6));
        assert!(!m.buildable(2, 12), "not on the pond");
    }
}
