//! The site's terrain with the player's additions laid over it, answering the movement and
//! placement questions everything else asks. Buildings never block movement (creatures pass in
//! front of them); only ladders, planks and springboards change anyone's map.

use crate::level::{Level, Tile};
use crate::rules::*;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Occ {
    None,
    Building(u32),
    Source(usize),
    Fortress,
    Spring,
    Entrance,
}

#[derive(Clone)]
pub struct World {
    pub w: i32,
    pub h: i32,
    pub tiles: Vec<Tile>,
    /// True where a plank was added (a one-way platform).
    pub plank: Vec<bool>,
    pub spring: Vec<bool>,
    pub occ: Vec<Occ>,
    /// Player ladders: top tile and length.
    pub ladders: Vec<(i32, i32, i32)>,
    /// Bumped whenever movement changes (paths cached against an older stamp are stale).
    pub stamp: u32,
}

impl World {
    pub fn new(level: &Level) -> World {
        let n = (level.w * level.h) as usize;
        let mut w = World {
            w: level.w,
            h: level.h,
            tiles: level.tiles.clone(),
            plank: vec![false; n],
            spring: vec![false; n],
            occ: vec![Occ::None; n],
            ladders: Vec::new(),
            stamp: 1,
        };
        for (i, s) in level.sources.iter().enumerate() {
            w.set_occ(s.x, s.y, Occ::Source(i));
        }
        let (fx, fy, fw, fh) = level.fortress;
        for y in fy..fy + fh {
            for x in fx..fx + fw {
                w.set_occ(x, y, Occ::Fortress);
            }
        }
        for &(x, y) in &level.springs {
            if w.inside(x, y) {
                let i = w.idx(x, y);
                w.spring[i] = true;
            }
        }
        let (ex, ey) = level.entrance;
        w.set_occ(ex, ey, Occ::Entrance);
        w
    }

    fn set_occ(&mut self, x: i32, y: i32, o: Occ) {
        if self.inside(x, y) {
            let i = self.idx(x, y);
            self.occ[i] = o;
        }
    }

    #[inline]
    pub fn inside(&self, x: i32, y: i32) -> bool {
        x >= 0 && y >= 0 && x < self.w && y < self.h
    }

    #[inline]
    pub fn idx(&self, x: i32, y: i32) -> usize {
        (y * self.w + x) as usize
    }

    pub fn tile(&self, x: i32, y: i32) -> Tile {
        if !self.inside(x, y) {
            return Tile::Bramble;
        }
        self.tiles[self.idx(x, y)]
    }

    pub fn occ(&self, x: i32, y: i32) -> Occ {
        if !self.inside(x, y) {
            return Occ::None;
        }
        self.occ[self.idx(x, y)]
    }

    pub fn is_plank(&self, x: i32, y: i32) -> bool {
        self.inside(x, y) && self.plank[self.idx(x, y)]
    }

    pub fn is_spring(&self, x: i32, y: i32) -> bool {
        self.inside(x, y) && self.spring[self.idx(x, y)]
    }

    /// The map edges and the sky above row 0 count as solid.
    pub fn solid(&self, x: i32, y: i32) -> bool {
        !self.inside(x, y) || self.tile(x, y).solid()
    }

    pub fn platform(&self, x: i32, y: i32) -> bool {
        self.inside(x, y) && (self.tile(x, y).platform() || self.plank[self.idx(x, y)])
    }

    pub fn water(&self, x: i32, y: i32) -> bool {
        self.inside(x, y) && self.tile(x, y) == Tile::Water
    }

    pub fn passable(&self, x: i32, y: i32) -> bool {
        !self.solid(x, y) && !self.water(x, y)
    }

    pub fn floor_below(&self, x: i32, y: i32) -> bool {
        self.solid(x, y + 1) || self.platform(x, y + 1)
    }

    pub fn cloud(&self, x: i32, y: i32) -> bool {
        self.inside(x, y) && self.tile(x, y) == Tile::Cloud
    }

    /// Ground anyone can stand on (clouds excluded: they depend on the load).
    pub fn standable(&self, x: i32, y: i32) -> bool {
        self.passable(x, y) && !self.platform(x, y) && !self.cloud(x, y) && self.floor_below(x, y)
    }

    /// Standable for a creature carrying this weight: clouds hold empty and light loads.
    pub fn standable_w(&self, x: i32, y: i32, w: Weight) -> bool {
        self.standable(x, y) || (self.passable(x, y) && !self.platform(x, y) && !self.cloud(x, y) && w <= Weight::Light && self.cloud(x, y + 1))
    }

    pub fn climbable(&self, x: i32, y: i32, s: Species, w: Weight) -> bool {
        match self.tile(x, y) {
            Tile::Vine => s.climbs_vine(),
            Tile::Ladder => s.climbs_ladder(w),
            _ => false,
        }
    }

    /// How far a ladder hung from (x, y) would reach down to a floor, if it can be hung there.
    pub fn ladder_len(&self, x: i32, y: i32) -> Option<i32> {
        for k in 0..MAX_LADDER {
            let yy = y + k;
            if !self.inside(x, yy) || !(self.tile(x, yy).open() || self.tile(x, yy) == Tile::Vine) || self.platform(x, yy) {
                return None;
            }
            if self.floor_below(x, yy) {
                return if k >= 1 { Some(k + 1) } else { None };
            }
        }
        None
    }

    fn free(&self, x: i32, y: i32) -> bool {
        self.inside(x, y) && self.tile(x, y).open() && !self.platform(x, y) && self.occ(x, y) == Occ::None
    }

    /// Can a building of this kind have its top-left at (x, y)? Camps go on a source's tile.
    pub fn can_build(&self, kind: Kind, x: i32, y: i32) -> Result<(), &'static str> {
        if kind == Kind::Camp {
            return match self.occ(x, y) {
                Occ::Source(_) => Ok(()),
                Occ::Building(_) => Err("That source already has a camp."),
                _ => Err("A camp goes on a source: a bush, tree, pile, quarry, bank or fishing spot."),
            };
        }
        let (w, h) = kind.size();
        for dy in 0..h {
            for dx in 0..w {
                if !self.free(x + dx, y + dy) {
                    return Err("No room here.");
                }
            }
        }
        for dx in 0..w {
            if !self.inside(x + dx, y + h) || !self.tile(x + dx, y + h).buildable_top() {
                return Err(if w > 1 { "It needs earth, rock or a branch under both halves." } else { "It needs earth, rock or a branch under it." });
            }
        }
        Ok(())
    }

    pub fn place_building(&mut self, id: u32, kind: Kind, x: i32, y: i32) {
        let (w, h) = kind.size();
        for dy in 0..h {
            for dx in 0..w {
                self.set_occ(x + dx, y + dy, Occ::Building(id));
            }
        }
    }

    /// Clears a building's footprint (a camp gives its tile back to the source).
    pub fn remove_building(&mut self, kind: Kind, x: i32, y: i32, source: Option<usize>) {
        let (w, h) = kind.size();
        for dy in 0..h {
            for dx in 0..w {
                self.set_occ(x + dx, y + dy, Occ::None);
            }
        }
        if let Some(i) = source {
            self.set_occ(x, y, Occ::Source(i));
        }
    }

    pub fn can_spring(&self, x: i32, y: i32) -> Result<(), &'static str> {
        if !self.free(x, y) {
            return Err("No room here.");
        }
        if !self.floor_below(x, y) {
            return Err("A springboard has to sit on the ground.");
        }
        Ok(())
    }

    pub fn can_ladder(&self, x: i32, y: i32) -> Result<i32, &'static str> {
        // A ladder can be lashed along a vine.
        let room = |xx: i32, yy: i32| self.free(xx, yy) || (self.inside(xx, yy) && self.tile(xx, yy) == Tile::Vine && self.occ(xx, yy) == Occ::None);
        if !room(x, y) {
            return Err("No room here.");
        }
        let len = self.ladder_len(x, y).ok_or("Click the top rung. A ladder hangs 2 to 12 tiles down to a floor.")?;
        for k in 0..len {
            if !room(x, y + k) {
                return Err("Something is in the ladder's way.");
            }
        }
        Ok(len)
    }

    pub fn can_plank(&self, x: i32, y: i32) -> Result<(), &'static str> {
        if !self.free(x, y) {
            return Err("No room here.");
        }
        let touches = |xx: i32| self.inside(xx, y) && (self.solid(xx, y) || self.platform(xx, y));
        if !touches(x - 1) && !touches(x + 1) {
            return Err("A plank has to touch ground or another plank, side by side.");
        }
        Ok(())
    }

    pub fn add_spring(&mut self, x: i32, y: i32) {
        let i = self.idx(x, y);
        self.spring[i] = true;
        self.occ[i] = Occ::Spring;
        self.stamp += 1;
    }

    pub fn add_plank(&mut self, x: i32, y: i32) {
        let i = self.idx(x, y);
        self.plank[i] = true;
        self.stamp += 1;
    }

    pub fn add_ladder(&mut self, x: i32, y: i32) -> i32 {
        let len = self.ladder_len(x, y).unwrap_or(0);
        for k in 0..len {
            let i = self.idx(x, y + k);
            self.tiles[i] = Tile::Ladder;
        }
        self.ladders.push((x, y, len));
        self.stamp += 1;
        len
    }

    /// Takes away the player infrastructure covering (x, y), if any. Returns its refund basis
    /// (seeds it cost).
    pub fn remove_infra(&mut self, x: i32, y: i32, level: &Level) -> Option<i32> {
        if !self.inside(x, y) {
            return None;
        }
        let i = self.idx(x, y);
        if self.occ[i] == Occ::Spring {
            self.spring[i] = false;
            self.occ[i] = Occ::None;
            self.stamp += 1;
            return Some(Infra::Spring.cost());
        }
        if self.plank[i] {
            self.plank[i] = false;
            self.stamp += 1;
            return Some(Infra::Plank.cost());
        }
        if let Some(k) = self.ladders.iter().position(|&(lx, ly, len)| lx == x && y >= ly && y < ly + len) {
            let (lx, ly, len) = self.ladders.remove(k);
            for j in 0..len {
                let ti = self.idx(lx, ly + j);
                self.tiles[ti] = level.tile(lx, ly + j);
            }
            self.stamp += 1;
            return Some(len * Infra::Ladder.cost());
        }
        None
    }

    /// Player infrastructure covering (x, y).
    pub fn infra_at(&self, x: i32, y: i32) -> Option<Infra> {
        if !self.inside(x, y) {
            return None;
        }
        let i = self.idx(x, y);
        if self.occ[i] == Occ::Spring {
            Some(Infra::Spring)
        } else if self.plank[i] {
            Some(Infra::Plank)
        } else if self.ladders.iter().any(|&(lx, ly, len)| lx == x && y >= ly && y < ly + len) {
            Some(Infra::Ladder)
        } else {
            None
        }
    }
}
