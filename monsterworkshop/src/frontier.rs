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
//!
//! Bin and wagon kits (M3d) unpack the same way. A bin goes where a workshop can, its chute on
//! the far side; it holds a limited room (more for a kit worth more), takes the first kind of
//! good pushed in and only that for good, and falls apart when nothing has gone in or out for
//! about half an hour, setting its goods out. A wagon unpacks anywhere a train fits, town and
//! road included: its pumpkin wakes where the kit stood, hitched to the cart beyond it, and it
//! pulls harder the better made it is.

use crate::econ::{ItemData, Share};
use crate::game::{Game, Sfx, UseRes};
use crate::workshops::ThingKit;
use crate::world::{Bin, Dir, FrontierBin, PKind, Shop, ShopKind, Tile, Wagon, TOWN_WAGON_POWER};
use serde::{Deserialize, Serialize};

/// Ticks of work a kit's workshop lasts: about an hour at 15 ticks a second.
pub const LIFETIME_TICKS: u32 = 54_000;
/// Ticks a worn-out workshop stands unused before it falls apart: about ten minutes.
pub const FALL_APART_TICKS: u32 = 9_000;
/// Ticks a bin from a kit stands with nothing going in or out before it falls apart: about half
/// an hour.
pub const BIN_IDLE_TICKS: u32 = 27_000;

/// What a kit unpacks into.
#[derive(Clone, Copy)]
pub enum Unpacks {
    Shop(ShopKind),
    Bin(&'static ThingKit),
    Wagon(&'static ThingKit),
}

/// What this good unpacks into, if it's a kit.
pub fn unpacks(good: &str) -> Option<Unpacks> {
    ShopKind::of_kit(good)
        .map(Unpacks::Shop)
        .or_else(|| crate::workshops::bin_kit().filter(|k| k.good == good).map(Unpacks::Bin))
        .or_else(|| crate::workshops::wagon_kit().filter(|k| k.good == good).map(Unpacks::Wagon))
}

/// The category of variety that names and colors what this kit unpacks into (wood, stone).
pub fn named_by(good: &str) -> Option<&'static str> {
    match unpacks(good)? {
        Unpacks::Shop(kind) => kind.kit().map(|kit| kit.named_by),
        Unpacks::Bin(kit) | Unpacks::Wagon(kit) => Some(kit.named_by),
    }
}

/// How much more than the plainest kit like it a kit's materials are worth, gently (the square
/// root), from 1 up to 1.6: a faster workshop, a roomier bin, a stronger wagon.
fn betterness(data: &ItemData, plain_worth: f64) -> f64 {
    (worth(data) / plain_worth).sqrt().clamp(1.0, 1.6)
}

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

/// Where a kit would set up what it unpacks into, and what's in the way.
pub struct Footprint {
    pub what: Unpacks,
    pub spot: (i32, i32),
    pub pads: Vec<(i32, i32)>,
    pub outs: Vec<(i32, i32)>,
    pub pegs: Vec<(i32, i32)>,
    /// A wagon's cart, beyond its pumpkin on the kit's spot.
    pub cart: Vec<(i32, i32)>,
    /// The tiles something's in the way on, and what to say about it.
    pub blocked: Vec<(i32, i32)>,
    pub why: Option<String>,
}

impl Footprint {
    pub fn tiles(&self) -> impl Iterator<Item = (i32, i32)> + '_ {
        std::iter::once(self.spot).chain(self.pads.iter().copied()).chain(self.outs.iter().copied()).chain(self.pegs.iter().copied()).chain(self.cart.iter().copied())
    }
}

/// The kind of workshop a kit sets up.
pub fn kit_kind(good: &str) -> Option<ShopKind> {
    ShopKind::of_kit(good)
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
    crate::econ::total(data.raw.iter().map(|r| r.base_value))
}

/// How much faster a workshop from this kit works: by what its materials are worth against the
/// plainest kit of its kind (its `plain_worth`: hemlock and cotton rope), gently: the square root,
/// so about 20% faster for an oak workbench, a little for an oak-framed kiln, whose stone is the
/// same either way.
pub(crate) fn speed(data: &ItemData) -> f64 {
    kit_kind(&data.good).and_then(|k| k.kit()).map_or(1.0, |kit| betterness(data, kit.plain_worth))
}

/// Its main material, for the name of what it unpacks into: the variety it carried over of the
/// category that names it (the wood of a workbench, the stone of a kiln), whatever variety.
fn material(goods: &crate::data::Data, data: &ItemData) -> String {
    let named_by = named_by(&data.good).unwrap_or("");
    goods.variety_in(&data.carried_tags, named_by).map_or_else(String::new, |t| t.to_uppercase())
}

/// "THE OAK BIN", or "THE BIN" with no wood to name it by.
fn the(material: &str, what: &str) -> String {
    if material.is_empty() {
        format!("THE {what}")
    } else {
        format!("THE {material} {what}")
    }
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
        self.world.item_at(m.x + dx, m.y + dy).filter(|&k| unpacks(self.world.items[k].good()).is_some())
    }

    /// Where kit `k` would set up what it unpacks into for monster `i` facing it, and what's in
    /// the way. A workshop or bin is laid out from its spot; a wagon's pumpkin wakes on the
    /// kit's tile with its cart beyond it, so whoever unpacked it stands ready to lead.
    pub(crate) fn footprint(&self, i: usize, k: usize) -> Option<Footprint> {
        let it = &self.world.items[k];
        let what = unpacks(it.good())?;
        let (spot, facing) = ((it.x, it.y), self.monsters[i].facing);
        let at = |o: (i32, i32)| {
            let t = turn(o, facing);
            (spot.0 + t.0, spot.1 + t.1)
        };
        let mut fp = Footprint { what, spot, pads: Vec::new(), outs: Vec::new(), pegs: Vec::new(), cart: Vec::new(), blocked: Vec::new(), why: None };
        match what {
            Unpacks::Shop(kind) => {
                let kit = kind.kit()?;
                fp.pads = kit.pads.iter().copied().map(at).collect();
                fp.outs = kit.outs.iter().copied().map(at).collect();
                fp.pegs = kit.pegs.iter().copied().map(at).collect();
            }
            // Its chute on the far side.
            Unpacks::Bin(_) => fp.outs = vec![at((0, 1))],
            Unpacks::Wagon(_) => fp.cart = vec![at((0, 1))],
        }
        let wagon = matches!(what, Unpacks::Wagon(_));
        let tiles: Vec<(i32, i32)> = fp.tiles().collect();
        for t in tiles {
            let w = &self.world;
            let why = if t.0 < 0 || t.1 < 0 || t.0 >= w.w || t.1 >= w.h {
                Some("NO ROOM HERE.")
            } else if wagon {
                // A vehicle, not a building: anywhere a train can stand.
                if !matches!(w.tile(t.0, t.1), Tile::Grass | Tile::Gravel | Tile::Path) || w.plant_at(t.0, t.1).is_some() {
                    Some("NO ROOM HERE.")
                } else if (t != spot && w.item_at(t.0, t.1).is_some()) || self.monster_at(t.0, t.1).is_some() || w.cart_at(t.0, t.1) || self.applicant_at(t.0, t.1) {
                    Some("SOMETHING'S IN THE WAY.")
                } else {
                    None
                }
            } else if w.in_town(t.0, t.1) {
                Some("NOBODY BUILDS IN TOWN. TAKE IT OUT INTO THE WILDS.")
            } else if w.tile(t.0, t.1) == Tile::Path {
                Some("NOT ON THE ROAD.")
            } else if !matches!(w.tile(t.0, t.1), Tile::Grass | Tile::Gravel) || w.plant_at(t.0, t.1).is_some() || w.bin_at(t.0, t.1).is_some() || w.board_at(t.0, t.1).is_some() {
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

    /// Space facing a kit: unpack it into its workshop, bin or wagon, if there's room.
    pub(crate) fn unpack_kit(&mut self, i: usize, k: usize, sfx: &mut Vec<Sfx>) -> UseRes {
        let Some(fp) = self.footprint(i, k) else { return UseRes::Nothing };
        if let Some(why) = fp.why {
            return UseRes::Wait(why);
        }
        let data = self.world.items.remove(k).data;
        let material = material(&self.data, &data);
        let news = match fp.what {
            Unpacks::Shop(kind) => {
                let frontier = Frontier { kit: data.good.clone(), material, speed: speed(&data), shares: data.shares.clone(), worked: 0, idle: 0, fallen: false };
                let shop = Shop { kind, spot: fp.spot, pad_flash: vec![0; fp.pads.len()], pads: fp.pads, pegs: fp.pegs, outs: fp.outs, quality: data.quality, job: None, pending: Vec::new(), out_timer: 0, frontier: Some(frontier), aimed: None, chosen: None, owner: None };
                self.world.shops.push(shop);
                let s = self.world.shops.len() - 1;
                self.world.lay_shop(s);
                for (x, y) in self.world.shops[s].pads.clone() {
                    self.world.burst(x, y, PKind::Spark, 3);
                }
                format!("THE {} IS UP!", self.shop_name(s))
            }
            Unpacks::Bin(kit) => {
                let room = kit.room * betterness(&data, kit.plain_worth);
                let news = format!("{} IS UP!", the(&material, "BIN"));
                let frontier = FrontierBin { kit: data.good, material, room, idle: 0, held: 0 };
                let (x, y) = fp.spot;
                self.world.bins.push(Bin { x, y, accepts: Vec::new(), stock: Vec::new(), mouth: fp.outs.first().copied(), timer: 0, bounce: 4, scores: false, frontier: Some(frontier), name: None });
                let w = self.world.w;
                self.world.tiles[(y * w + x) as usize] = Tile::Bin;
                news
            }
            Unpacks::Wagon(kit) => {
                // A plain one of middling make pulls like the town's; a better one, harder.
                let power = TOWN_WAGON_POWER * betterness(&data, kit.plain_worth) * (0.5 + data.quality);
                let news = format!("{} IS READY TO ROLL!", the(&material, "WAGON"));
                let wagon = Wagon { at: fp.cart[0], pumpkin: fp.spot, slide: None, pumpkin_slide: None, stock: Vec::new(), room: kit.room, power, open: false, leader: None, last_leader: None, timer: 0, bounce: 4, material };
                self.world.wagons.push(wagon);
                self.world.burst(fp.cart[0].0, fp.cart[0].1, PKind::Spark, 3);
                news
            }
        };
        self.world.burst(fp.spot.0, fp.spot.1, PKind::Spark, 3);
        self.celebrate(fp.spot, &news);
        sfx.push(Sfx::Chime);
        UseRes::Done
    }

    /// Why bin `b` won't take this good, if it won't: it's the wrong kind, or it's full.
    pub(crate) fn bin_takes(&self, b: usize, good: &str) -> Result<(), String> {
        let bin = &self.world.bins[b];
        let gt = self.data.try_good(good).ok_or_else(|| "NOTHING GOES IN THERE.".to_string())?;
        if !bin.takes(gt) {
            let names: Vec<String> = bin.accepts.iter().map(|a| self.good_name(a)).collect();
            return Err(format!("ONLY {} GOES IN THERE.", names.join(" OR ")));
        }
        if let Some(f) = &bin.frontier {
            let used = crate::econ::total(bin.stock.iter().map(|d| self.data.good(&d.good).storage_volume));
            if used + gt.storage_volume > f.room + 1e-9 {
                return Err(format!("{} IS FULL.", self.bin_name(b)));
            }
        }
        Ok(())
    }

    /// A good goes into bin `b`: a new bin from a kit takes its kind from the first.
    pub(crate) fn put_in_bin(&mut self, b: usize, data: ItemData) {
        let bin = &mut self.world.bins[b];
        if bin.accepts.is_empty() {
            bin.accepts.push(data.good.clone());
        }
        bin.stock.push(data);
        bin.bounce = 4;
    }

    /// "THE COTTON BOLLS BIN" by what it takes; a new one from a kit by its wood until then.
    pub(crate) fn bin_name(&self, b: usize) -> String {
        let bin = &self.world.bins[b];
        if let Some(name) = &bin.name {
            return name.clone();
        }
        match bin.accepts.first() {
            Some(a) => format!("THE {} BIN", self.good_name(a)),
            None => the(bin.frontier.as_ref().map_or("", |f| f.material.as_str()), "BIN"),
        }
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
        shop.frontier = Some(Frontier { kit: data.good.clone(), material: material(&self.data, &data), speed: speed(&data), shares: data.shares.clone(), worked: 0, idle: 0, fallen: false });
        let spot = shop.spot;
        self.world.burst(spot.0, spot.1, PKind::Spark, 5);
        let name = self.shop_name(s);
        self.celebrate(spot, &format!("THE {name} IS LIKE NEW!"));
        Ok(())
    }

    /// Frontier workshops wear while they work; a worn-out one left unused falls apart. A bin
    /// from a kit falls apart when nothing has gone in or out of it for long.
    pub(crate) fn tick_frontier(&mut self) {
        for b in (0..self.world.bins.len()).rev() {
            let held = self.world.bins[b].stock.len();
            let Some(f) = self.world.bins[b].frontier.as_mut() else { continue };
            if held != f.held {
                (f.held, f.idle) = (held, 0);
            } else {
                f.idle += 1;
            }
            if f.idle >= BIN_IDLE_TICKS {
                self.bin_falls_apart(b);
            }
        }
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

    /// Worn-out workshop `s` falls apart: its ground is as the map had it again (grass, or the
    /// hills' stony ground), and what waited inside is set out where it stood, where there's room.
    fn fall_apart(&mut self, s: usize) {
        let tiles: Vec<(i32, i32)> = {
            let shop = &self.world.shops[s];
            std::iter::once(shop.spot).chain(shop.pads.iter().copied()).chain(shop.outs.iter().copied()).chain(shop.pegs.iter().copied()).collect()
        };
        let pending = std::mem::take(&mut self.world.shops[s].pending);
        let w = self.world.w;
        for &(x, y) in &tiles {
            self.world.tiles[(y * w + x) as usize] = self.world.ground[(y * w + x) as usize];
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

    /// Bin `b` falls apart: its ground is as the map had it again, and its goods are set out around it, ring
    /// by ring outwards from where it stood, on open ground.
    fn bin_falls_apart(&mut self, b: usize) {
        let bin = self.world.bins.remove(b);
        let w = self.world.w;
        self.world.tiles[(bin.y * w + bin.x) as usize] = self.world.ground[(bin.y * w + bin.x) as usize];
        self.world.burst(bin.x, bin.y, PKind::Dust, 3);
        let mut goods = bin.stock.into_iter().peekable();
        for r in 0..8 {
            for y in bin.y - r..=bin.y + r {
                for x in bin.x - r..=bin.x + r {
                    let ring = (x - bin.x).abs().max((y - bin.y).abs()) == r;
                    let inside = x >= 0 && y >= 0 && x < w && y < self.world.h;
                    if !ring || !inside || goods.peek().is_none() {
                        continue;
                    }
                    let open = matches!(self.world.tile(x, y), Tile::Grass | Tile::Gravel | Tile::Path) && self.world.item_at(x, y).is_none() && self.world.plant_at(x, y).is_none() && self.monster_at(x, y).is_none() && !self.world.cart_at(x, y);
                    if open {
                        let data = goods.next().expect("peeked");
                        self.world.items.push(crate::world::Item::new(x, y, data));
                    }
                }
            }
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
