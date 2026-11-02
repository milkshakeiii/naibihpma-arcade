//! Frontier workshops (M2 stage 4): built from kits out in the wilds, where the materials are.
//!
//! A kit is an ordinary good crafted at a bench (the builders' bench in Pinewater lends the
//! tools). Pushed or carted to where you want it, outside town and off the road, facing it shows
//! the workshop's footprint as a ghost on the ground, red where something's in the way; Space
//! unpacks it there, turned the way you face it. The kit decides the workshop: its quality (a
//! weight-8 tool in everything made there, for a walk-in workshop), its speed (from what its
//! materials are worth, gently), its name ("OAK WORKBENCH"), and who holds its shares (whoever
//! made and hauled the kit). Frontier workshops are open-air: their pads, work spot, mat and pegs
//! laid on the ground, with no walls or doors. They lend no tools.
//!
//! Wear counts time worked, not jobs done. A worn-out workshop stops; pushing a new kit of the
//! same kind onto its work spot renews it in place (same layout, so loops around it carry on).
//! Left worn out long enough, it falls apart and frees the ground.

use crate::econ::{ItemData, Share};
use crate::game::{Game, Sfx, UseRes};
use crate::world::{area_name, Dir, PKind, Shop, ShopKind, Tile};
use serde::{Deserialize, Serialize};

/// Ticks of work a kit's workshop lasts: about an hour at 15 ticks a second.
pub const LIFETIME_TICKS: u32 = 54_000;
/// Ticks a worn-out workshop stands unused before it falls apart: about ten minutes.
pub const FALL_APART_TICKS: u32 = 9_000;

/// What a frontier workshop got from its kit, and how worn it is.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Frontier {
    /// The kit it was unpacked from (renewing takes another of these).
    pub kit: String,
    /// Its main material, for its name: "OAK", "LIMESTONE".
    pub material: String,
    /// How much faster than a town workshop it works: 1.0 and up.
    pub speed: f64,
    /// Whoever made and hauled the kit: they share in everything made here.
    pub shares: Vec<Share>,
    /// Ticks worked, and ticks stood worn out and unused.
    pub worked: u32,
    pub idle: u32,
    /// Fallen apart: it's gone, and its ground is free again.
    pub fallen: bool,
}

/// Where a kit would set its workshop up, and what's in the way.
pub struct Footprint {
    pub kind: ShopKind,
    pub spot: (i32, i32),
    pub pads: Vec<(i32, i32)>,
    pub outs: Vec<(i32, i32)>,
    pub pegs: Vec<(i32, i32)>,
    /// The tiles something's in the way on, and what to say about it.
    pub blocked: Vec<(i32, i32)>,
    pub why: Option<String>,
}

impl Footprint {
    pub fn tiles(&self) -> impl Iterator<Item = (i32, i32)> + '_ {
        std::iter::once(self.spot).chain(self.pads.iter().copied()).chain(self.outs.iter().copied()).chain(self.pegs.iter().copied())
    }
}

/// The kind of workshop a kit sets up.
pub fn kit_kind(good: &str) -> Option<ShopKind> {
    match good {
        "workbench_kit" => Some(ShopKind::Workbench),
        "kiln_kit" => Some(ShopKind::Kiln),
        _ => None,
    }
}

/// A frontier workshop's layout around its work spot, for one facing down: (pads, output mats,
/// pegs). Nothing is ever just behind the spot, where the monster unpacking it stands.
fn layout(kind: ShopKind) -> (Vec<(i32, i32)>, Vec<(i32, i32)>, Vec<(i32, i32)>) {
    match kind {
        // Two pads to one side, the mat in front, two pegs to the other side.
        ShopKind::Workbench => (vec![(-2, 0), (-1, 0)], vec![(0, 1)], vec![(1, 0), (2, 0)]),
        // A pad on each side, the mat in front.
        _ => (vec![(-1, 0), (1, 0)], vec![(0, 1)], Vec::new()),
    }
}

/// An offset from the layout's facing (down) turned to face `d`.
fn turn((x, y): (i32, i32), d: Dir) -> (i32, i32) {
    match d {
        Dir::Down => (x, y),
        Dir::Up => (-x, -y),
        Dir::Right => (y, -x),
        Dir::Left => (-y, x),
    }
}

/// What a kit's materials are worth: the base values of everything in it, unaffected by quality.
fn worth(data: &ItemData) -> f64 {
    data.raw.iter().map(|r| r.base_value).sum()
}

/// How much faster a workshop from this kit works: by what its materials are worth against the
/// plainest kit of its kind (hemlock, cotton rope), gently: the square root, so about 20% faster
/// for an oak workbench, a little for an oak-framed kiln, whose stone is the same either way.
pub(crate) fn speed(data: &ItemData) -> f64 {
    let plainest = match data.good.as_str() {
        "kiln_kit" => 22.0,
        _ => 14.0,
    };
    (worth(data) / plainest).sqrt().clamp(1.0, 1.6)
}

/// Its main material, for the workshop's name: the wood or stone it carried over.
fn material(data: &ItemData) -> String {
    data.carried_tags.iter().find(|t| ["oak", "hemlock", "limestone"].contains(&t.as_str())).map_or_else(String::new, |t| t.to_uppercase())
}

impl Game {
    /// "OAK WORKBENCH" for a frontier workshop; a town one by its kind.
    pub(crate) fn shop_name(&self, s: usize) -> String {
        let shop = &self.world.shops[s];
        match shop.frontier.as_ref().filter(|f| !f.material.is_empty()) {
            Some(f) => format!("{} {}", f.material, shop.kind.label()),
            None => shop.kind.label().to_string(),
        }
    }

    /// Is this frontier workshop worn out (or fallen apart)?
    pub(crate) fn worn_out(&self, s: usize) -> bool {
        self.world.shops[s].frontier.as_ref().is_some_and(|f| f.fallen || f.worked >= LIFETIME_TICKS)
    }

    /// Why a worn-out workshop won't work, and what to do about it.
    pub(crate) fn worn_note(&self, s: usize) -> String {
        let kit = self.world.shops[s].frontier.as_ref().map_or("kit", |f| f.kit.as_str());
        format!("THE {} IS WORN OUT. PUSH A NEW {} ONTO ITS SPOT.", self.shop_name(s), self.good_name(kit))
    }

    /// The kit right in front of monster `i`, if any.
    pub(crate) fn kit_ahead(&self, i: usize) -> Option<usize> {
        let m = &self.monsters[i];
        let (dx, dy) = m.facing.delta();
        self.world.item_at(m.x + dx, m.y + dy).filter(|&k| kit_kind(self.world.items[k].good()).is_some())
    }

    /// Where kit `k` would set its workshop up for monster `i` facing it, and what's in the way.
    pub(crate) fn footprint(&self, i: usize, k: usize) -> Option<Footprint> {
        let it = &self.world.items[k];
        let kind = kit_kind(it.good())?;
        let (spot, facing) = ((it.x, it.y), self.monsters[i].facing);
        let at = |o: (i32, i32)| {
            let t = turn(o, facing);
            (spot.0 + t.0, spot.1 + t.1)
        };
        let (pads, outs, pegs) = layout(kind);
        let mut fp = Footprint { kind, spot, pads: pads.into_iter().map(at).collect(), outs: outs.into_iter().map(at).collect(), pegs: pegs.into_iter().map(at).collect(), blocked: Vec::new(), why: None };
        let tiles: Vec<(i32, i32)> = fp.tiles().collect();
        for t in tiles {
            let w = &self.world;
            let why = if t.0 < 0 || t.1 < 0 || t.0 >= w.w || t.1 >= w.h {
                Some("NO ROOM HERE.")
            } else if matches!(area_name(t.0), "PINEWATER" | "TANGLEMERE") {
                Some("NOBODY BUILDS IN TOWN. TAKE IT OUT INTO THE WILDS.")
            } else if w.tile(t.0, t.1) == Tile::Path {
                Some("NOT ON THE ROAD.")
            } else if w.tile(t.0, t.1) != Tile::Grass || w.plant_at(t.0, t.1).is_some() || w.bin_at(t.0, t.1).is_some() || w.board_at(t.0, t.1).is_some() {
                Some("NO ROOM HERE.")
            } else if (t != spot && w.item_at(t.0, t.1).is_some()) || self.monster_at(t.0, t.1).is_some() || w.cart_at(t.0, t.1) || self.applicant_at(t.0, t.1) {
                Some("SOMETHING'S IN THE WAY.")
            } else {
                None
            };
            if let Some(why) = why {
                fp.blocked.push(t);
                fp.why.get_or_insert_with(|| why.to_string());
            }
        }
        Some(fp)
    }

    /// Space facing a kit: unpack it into its workshop, if there's room.
    pub(crate) fn unpack_kit(&mut self, i: usize, k: usize, sfx: &mut Vec<Sfx>) -> UseRes {
        let Some(fp) = self.footprint(i, k) else { return UseRes::Nothing };
        if let Some(why) = fp.why {
            return UseRes::Wait(why);
        }
        let data = self.world.items.remove(k).data;
        let frontier = Frontier { kit: data.good.clone(), material: material(&data), speed: speed(&data), shares: data.shares.clone(), worked: 0, idle: 0, fallen: false };
        let shop = Shop { kind: fp.kind, spot: fp.spot, pad_flash: vec![0; fp.pads.len()], pads: fp.pads, pegs: fp.pegs, outs: fp.outs, quality: data.quality, job: None, pending: Vec::new(), out_timer: 0, frontier: Some(frontier) };
        self.world.shops.push(shop);
        let s = self.world.shops.len() - 1;
        self.world.lay_shop(s);
        let tiles: Vec<(i32, i32)> = std::iter::once(fp.spot).chain(self.world.shops[s].pads.clone()).collect();
        for (x, y) in tiles {
            self.world.burst(x, y, PKind::Spark, 3);
        }
        let name = self.shop_name(s);
        self.celebrate(fp.spot, &format!("THE {name} IS UP!"));
        sfx.push(Sfx::Chime);
        UseRes::Done
    }

    /// A kit pushed onto frontier workshop `s`'s work spot: if it's the right kind, it renews
    /// the workshop in place, with the new kit's quality, speed, name and makers.
    pub(crate) fn renew_shop(&mut self, s: usize, it: usize, hauler: u32) -> Result<(), String> {
        let shop = &self.world.shops[s];
        let Some(f) = shop.frontier.as_ref() else { return Err("THAT'S A WORK SPOT.".into()) };
        if self.world.items[it].good() != f.kit {
            return Err(format!("ONLY A {} RENEWS IT.", self.good_name(&f.kit.clone())));
        }
        let mut data = self.world.items.remove(it).data;
        data.hauled_by(hauler);
        let shop = &mut self.world.shops[s];
        shop.quality = data.quality;
        shop.frontier = Some(Frontier { kit: data.good.clone(), material: material(&data), speed: speed(&data), shares: data.shares.clone(), worked: 0, idle: 0, fallen: false });
        let spot = shop.spot;
        self.world.burst(spot.0, spot.1, PKind::Spark, 5);
        let name = self.shop_name(s);
        self.celebrate(spot, &format!("THE {name} IS LIKE NEW!"));
        Ok(())
    }

    /// Frontier workshops wear while they work; a worn-out one left unused falls apart.
    pub(crate) fn tick_frontier(&mut self) {
        for s in 0..self.world.shops.len() {
            let working = self.world.shops[s].job.is_some();
            let Some(f) = self.world.shops[s].frontier.as_mut().filter(|f| !f.fallen) else { continue };
            if working {
                f.worked = f.worked.saturating_add(1);
                f.idle = 0;
            } else if f.worked >= LIFETIME_TICKS {
                f.idle += 1;
                if f.idle >= FALL_APART_TICKS {
                    self.fall_apart(s);
                }
            }
        }
    }

    /// Worn-out workshop `s` falls apart: its ground is grass again, and what waited inside is
    /// set out where it stood, where there's room.
    fn fall_apart(&mut self, s: usize) {
        let tiles: Vec<(i32, i32)> = {
            let shop = &self.world.shops[s];
            std::iter::once(shop.spot).chain(shop.pads.iter().copied()).chain(shop.outs.iter().copied()).chain(shop.pegs.iter().copied()).collect()
        };
        let pending = std::mem::take(&mut self.world.shops[s].pending);
        let w = self.world.w;
        for &(x, y) in &tiles {
            self.world.tiles[(y * w + x) as usize] = Tile::Grass;
            self.world.burst(x, y, PKind::Dust, 2);
        }
        let free: Vec<(i32, i32)> = tiles.iter().copied().filter(|&(x, y)| self.world.item_at(x, y).is_none()).collect();
        for (data, (x, y)) in pending.into_iter().zip(free) {
            self.world.items.push(crate::world::Item::new(x, y, data));
        }
        let shop = &mut self.world.shops[s];
        (shop.pads, shop.outs, shop.pegs, shop.pad_flash) = (Vec::new(), Vec::new(), Vec::new(), Vec::new());
        if let Some(f) = shop.frontier.as_mut() {
            f.fallen = true;
        }
    }

    /// Text rising from a tile, like the bridge being built.
    fn celebrate(&mut self, (x, y): (i32, i32), text: &str) {
        use crate::world::TS;
        let at = (x * TS + 8, y * TS);
        self.floaters.push(crate::game::Floater { text: text.into(), from: at, to: (at.0, at.1 - 30), t: 0, total: 60, delay: 0, color: crate::palette::SPARK + 2 });
    }
}

impl crate::world::World {
    /// Lays frontier workshop `s`'s tiles on the ground (when it's unpacked, and again when a
    /// world is loaded).
    pub fn lay_shop(&mut self, s: usize) {
        let shop = &self.shops[s];
        if shop.frontier.as_ref().is_none_or(|f| f.fallen) {
            return;
        }
        let mut lay = vec![(shop.spot, Tile::Spot)];
        lay.extend(shop.pads.iter().map(|&p| (p, Tile::Pad)));
        lay.extend(shop.outs.iter().map(|&p| (p, Tile::OutMat)));
        lay.extend(shop.pegs.iter().map(|&p| (p, Tile::Peg)));
        for ((x, y), t) in lay {
            self.tiles[(y * self.w + x) as usize] = t;
        }
    }
}
