//! The level's terrain with the player's additions laid over it, answering the movement and
//! placement questions everything else asks.

use crate::layout::{Layout, NodeRef};
use crate::level::{Level, Tile};
use crate::rules::*;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Occ {
    None,
    Building(u32),
    Source(usize),
    Fortress,
    Spring,
}

pub struct World {
    pub w: i32,
    pub h: i32,
    pub tiles: Vec<Tile>,
    /// True where a plank was added (a one-way platform).
    pub plank: Vec<bool>,
    pub spring: Vec<bool>,
    pub occ: Vec<Occ>,
    /// Length of each of the layout's ladders, in layout order (0 = couldn't hang).
    pub ladder_lens: Vec<i32>,
}

impl World {
    pub fn new(level: &Level, layout: &Layout) -> World {
        let n = (level.w * level.h) as usize;
        let mut w = World { w: level.w, h: level.h, tiles: level.tiles.clone(), plank: vec![false; n], spring: vec![false; n], occ: vec![Occ::None; n], ladder_lens: Vec::new() };
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
        for &(x, y) in &layout.planks {
            if w.inside(x, y) {
                let i = w.idx(x, y);
                w.plank[i] = true;
            }
        }
        for &(x, y) in &layout.ladders {
            let len = w.ladder_len(x, y).unwrap_or(0);
            w.ladder_lens.push(len);
            for k in 0..len {
                let i = w.idx(x, y + k);
                w.tiles[i] = Tile::Ladder;
            }
        }
        for &(x, y) in &layout.springs {
            if w.inside(x, y) {
                let i = w.idx(x, y);
                w.spring[i] = true;
                w.occ[i] = Occ::Spring;
            }
        }
        for b in &layout.buildings {
            for dy in 0..2 {
                for dx in 0..2 {
                    w.set_occ(b.x + dx, b.y + dy, Occ::Building(b.id));
                }
            }
        }
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
            if !self.inside(x, yy) || !self.tile(x, yy).open() || self.platform(x, yy) {
                return None;
            }
            if self.floor_below(x, yy) {
                return if k >= 1 { Some(k + 1) } else { None };
            }
        }
        None
    }

    /// Tiles where creatures stand to use a node: the bottom row of its footprint.
    pub fn access(&self, level: &Level, layout: &Layout, n: NodeRef) -> Vec<(i32, i32)> {
        match n {
            NodeRef::Source(i) => level.sources.get(i).map(|s| vec![(s.x, s.y)]).unwrap_or_default(),
            NodeRef::Fortress => {
                let (fx, fy, fw, fh) = level.fortress;
                (fx..fx + fw).map(|x| (x, fy + fh - 1)).collect()
            }
            NodeRef::Building(id) => layout.building(id).map(|b| vec![(b.x, b.y + 1), (b.x + 1, b.y + 1)]).unwrap_or_default(),
        }
    }

    /// The node whose footprint covers this tile.
    pub fn node_at(&self, x: i32, y: i32) -> Option<NodeRef> {
        match self.occ(x, y) {
            Occ::Building(id) => Some(NodeRef::Building(id)),
            Occ::Source(i) => Some(NodeRef::Source(i)),
            Occ::Fortress => Some(NodeRef::Fortress),
            _ => None,
        }
    }

    fn free(&self, x: i32, y: i32) -> bool {
        self.inside(x, y) && self.tile(x, y).open() && !self.platform(x, y) && self.occ(x, y) == Occ::None
    }

    /// A 2x2 building with its top-left at (x, y).
    pub fn can_build(&self, x: i32, y: i32) -> Result<(), &'static str> {
        for dy in 0..2 {
            for dx in 0..2 {
                if !self.free(x + dx, y + dy) {
                    return Err("No room here.");
                }
            }
        }
        for dx in 0..2 {
            if !self.inside(x + dx, y + 2) || !self.tile(x + dx, y + 2).buildable_top() {
                return Err("It needs earth, rock or a branch under both halves.");
            }
        }
        Ok(())
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
        if !self.free(x, y) {
            return Err("No room here.");
        }
        let len = self.ladder_len(x, y).ok_or("Click the top rung. A ladder hangs 2 to 12 tiles down to a floor.")?;
        for k in 0..len {
            if !self.free(x, y + k) {
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
}

/// Total price of a layout in seeds.
pub fn layout_cost(level: &Level, layout: &Layout) -> i32 {
    let world = World::new(level, layout);
    let mut c = 0;
    for b in &layout.buildings {
        c += b.kind.cost();
    }
    c += layout.springs.len() as i32 * Infra::Spring.cost();
    c += layout.planks.len() as i32 * Infra::Plank.cost();
    for len in &world.ladder_lens {
        c += len * Infra::Ladder.cost();
    }
    for r in &layout.routes {
        c += r.species.cost() * r.count;
    }
    c
}
