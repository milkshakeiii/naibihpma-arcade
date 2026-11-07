//! Workshop kinds, from `data/workshops.json`: what each one makes, the tools a town one lends,
//! the character that marks its work spot on a map, how it looks, and the kit that sets one up out
//! in the wilds. Onboarding an industry is new entries here and in `good_types.json`, a sprite or
//! two and a place on the map, not new code: the tests below check that every recipe in play can
//! be made from what grows on the map, and that the data fits together.
//!
//! An entry's fields:
//! - `id`: how saves name it; `label`: its name in the game ("LOOM HOUSE").
//! - `map_char`: marks its work spot on a region map; its pads, pegs and mat belong to the
//!   nearest work spot.
//! - `look`: which sprite draws its work spot (see `render::LOOKS`), so a new kind can borrow an
//!   existing one until it has its own.
//! - `building`: a walk-in workshop counts as a weight-8 tool in everything made there; an open
//!   bench doesn't.
//! - `recipes`: what it can make; what's on its pads picks one.
//! - `lends`: basic tools a town one provides when nothing better hangs on its pegs (frontier
//!   ones lend none).
//! - `kit`, if it can be built out in the wilds: the kit good, what the plainest such kit's
//!   materials are worth (a kit worth more makes a faster workshop), the category of variety
//!   that names and colors it (a workbench by its wood: "OAK WORKBENCH"), and its open-air
//!   layout as offsets from the work spot for one
//!   facing down (turned the way it's unpacked): `pads`, `outs` (output mats) and `pegs`.
//! - `trades`, for a trader's stall instead of `recipes`: each trade's `id` (a recipe, made
//!   when the data loads), the good it `makes`, and what it `wants` on the pads. A trade has no
//!   skill or quality roll; see `econ::trade`.
//! - `about`: a note for people; the game doesn't read it.
//!
//! Beside the kinds, `bin_kit` and `wagon_kit` are the kits that unpack into a bin or a wagon
//! rather than a workshop: the kit good, the plainest such kit's worth, the category of variety
//! that names and colors what it unpacks into, and the `room` it holds (a better kit, more).

use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::sync::OnceLock;

const WORKSHOPS: &str = include_str!("../data/workshops.json");

#[derive(Deserialize)]
// Borrows its strings from the file, which is compiled in.
#[serde(bound(deserialize = "'de: 'static"))]
struct WorkshopsFile {
    workshop_types: Vec<WorkshopType>,
    #[serde(default)]
    bin_kit: Option<ThingKit>,
    #[serde(default)]
    wagon_kit: Option<ThingKit>,
}

/// A kit that unpacks into a bin or a wagon.
#[derive(Deserialize)]
pub struct ThingKit {
    pub good: &'static str,
    pub plain_worth: f64,
    pub named_by: &'static str,
    pub room: f64,
}

/// The kit that unpacks into a bin, if there is one.
pub fn bin_kit() -> Option<&'static ThingKit> {
    file().bin_kit.as_ref()
}

/// The kit that unpacks into a wagon, if there is one.
pub fn wagon_kit() -> Option<&'static ThingKit> {
    file().wagon_kit.as_ref()
}

#[derive(Deserialize)]
struct WorkshopType {
    id: &'static str,
    label: &'static str,
    map_char: char,
    look: &'static str,
    building: bool,
    #[serde(default)]
    recipes: Vec<&'static str>,
    #[serde(default)]
    lends: Vec<&'static str>,
    kit: Option<Kit>,
    #[serde(default)]
    trades: Vec<Trade>,
}

/// One of a trader's trades: a recipe whose product is another good.
#[derive(Deserialize)]
pub struct Trade {
    pub id: &'static str,
    pub makes: &'static str,
    pub wants: Vec<Vec<&'static str>>,
    /// What the good it gives is made of, where it has no weight of its own.
    #[serde(default)]
    pub of: Option<String>,
}

/// Every trader's trades, for the data to make recipes of.
pub fn trades() -> impl Iterator<Item = &'static Trade> {
    types().iter().flat_map(|t| &t.trades)
}

/// The kit that sets a workshop up out in the wilds, and how it lays it out.
#[derive(Deserialize)]
pub struct Kit {
    pub good: &'static str,
    pub plain_worth: f64,
    pub named_by: &'static str,
    pub pads: Vec<(i32, i32)>,
    pub outs: Vec<(i32, i32)>,
    #[serde(default)]
    pub pegs: Vec<(i32, i32)>,
}

fn file() -> &'static WorkshopsFile {
    static FILE: OnceLock<WorkshopsFile> = OnceLock::new();
    FILE.get_or_init(|| {
        let mut file = serde_json::from_str::<WorkshopsFile>(WORKSHOPS).expect("data/workshops.json");
        for t in &mut file.workshop_types {
            // A trader's recipes are its trades.
            if !t.trades.is_empty() {
                t.recipes = t.trades.iter().map(|tr| tr.id).collect();
            }
            assert!(!crate::world::MAP_LEGEND.contains(t.map_char), "{}'s map character {:?} already means something else on the map", t.id, t.map_char);
            assert!(crate::render::LOOKS.contains(&t.look), "{} looks like {:?}, which nothing draws", t.id, t.look);
        }
        file
    })
}

fn types() -> &'static [WorkshopType] {
    &file().workshop_types
}

/// A kind of workshop: an entry in `data/workshops.json`.
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct ShopKind(u8);

impl ShopKind {
    /// The kind with this id; there must be one.
    #[cfg(test)]
    pub fn named(id: &str) -> ShopKind {
        Self::all().find(|k| k.id() == id).unwrap_or_else(|| panic!("unknown workshop kind {id}"))
    }

    pub fn all() -> impl Iterator<Item = ShopKind> {
        (0..types().len()).map(|i| ShopKind(i as u8))
    }

    /// The kind whose work spot this map character marks.
    pub fn by_map_char(ch: char) -> Option<ShopKind> {
        Self::all().find(|k| k.ty().map_char == ch)
    }

    /// The kind of workshop this kit sets up.
    pub fn of_kit(good: &str) -> Option<ShopKind> {
        Self::all().find(|k| k.kit().is_some_and(|kit| kit.good == good))
    }

    fn ty(self) -> &'static WorkshopType {
        &types()[self.0 as usize]
    }

    pub fn id(self) -> &'static str {
        self.ty().id
    }

    pub fn label(self) -> &'static str {
        self.ty().label
    }

    /// A trader's stall: its recipes are trades.
    pub fn trades(self) -> bool {
        !self.ty().trades.is_empty()
    }

    pub fn look(self) -> &'static str {
        self.ty().look
    }

    pub fn recipes(self) -> &'static [&'static str] {
        &self.ty().recipes
    }

    /// Basic permanent tools a town workshop provides when nothing hangs on its pegs.
    pub fn builtin_tools(self) -> &'static [&'static str] {
        &self.ty().lends
    }

    /// Walk-in workshops count as a weight-8 tool; an outdoor bench does not.
    pub fn is_building(self) -> bool {
        self.ty().building
    }

    pub fn kit(self) -> Option<&'static Kit> {
        self.ty().kit.as_ref()
    }
}

impl std::fmt::Debug for ShopKind {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        f.write_str(self.id())
    }
}

/// Saved by id, so reordering the data file never changes what a save means.
impl Serialize for ShopKind {
    fn serialize<S: Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        s.serialize_str(self.id())
    }
}

impl<'de> Deserialize<'de> for ShopKind {
    fn deserialize<D: Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        let id = String::deserialize(d)?;
        Self::all().find(|k| k.id() == id).ok_or_else(|| serde::de::Error::custom(format!("unknown workshop kind {id}")))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::{matches, Data};
    use crate::world::World;
    use std::collections::{BTreeSet, HashSet};

    #[test]
    fn the_workshop_data_fits_together() {
        let data = Data::load();
        let mut ids = HashSet::new();
        let mut chars = HashSet::new();
        for k in ShopKind::all() {
            assert!(ids.insert(k.id()), "two workshop kinds called {}", k.id());
            let ch = k.ty().map_char;
            assert!(chars.insert(ch), "two workshop kinds marked {ch:?} on the map");
            for id in k.recipes().iter().chain(k.builtin_tools()) {
                assert!(data.try_good(id).is_some(), "{} names {id}, which isn't a good", k.id());
            }
            for r in k.recipes() {
                assert!(k.is_building() || !data.good(r).requires_workshop, "{r} needs a walk-in workshop, but the {} is a bench", k.id());
            }
            if let Some(kit) = k.kit() {
                assert!(data.try_good(kit.good).is_some(), "{}'s kit {} isn't a good", k.id(), kit.good);
                assert!(ShopKind::all().any(|b| b.recipes().contains(&kit.good)), "nothing makes the {}", kit.good);
                assert!(kit.plain_worth > 0.0);
                let tiles: Vec<(i32, i32)> = kit.pads.iter().chain(&kit.outs).chain(&kit.pegs).copied().collect();
                let unique: HashSet<_> = tiles.iter().collect();
                assert_eq!(unique.len(), tiles.len(), "{}'s layout uses a tile twice", k.id());
                // The one unpacking stands just behind the work spot.
                assert!(!tiles.contains(&(0, 0)) && !tiles.contains(&(0, -1)), "{}'s layout covers its work spot or the tile behind it", k.id());
                assert!(!kit.pads.is_empty() && !kit.outs.is_empty(), "{}'s layout needs a pad and a mat", k.id());
            }
        }
        for kit in [bin_kit(), wagon_kit()].into_iter().flatten() {
            assert!(data.try_good(kit.good).is_some(), "the kit {} isn't a good", kit.good);
            assert!(ShopKind::all().any(|b| b.recipes().contains(&kit.good)), "nothing makes the {}", kit.good);
            assert!(kit.plain_worth > 0.0 && kit.room > 0.0);
        }
    }

    /// Every recipe of every kind can be made starting from what grows on the map, or lies on it:
    /// at a town workshop on the map, with the tools it lends, or at a frontier one whose kit can
    /// be made, with tools that can be made. A trader's goods count as made once its trade can be.
    #[test]
    fn everything_in_play_can_be_made_from_what_grows_on_the_map() {
        let data = Data::load();
        let world = World::new(&data);
        let in_town: HashSet<ShopKind> = world.shops.iter().map(|s| s.kind).collect();
        // Goods that can be had, and recipes that can be made (a trade's good is another good):
        // what grows, and what lies about when a world begins (the hills' hammers, chisels, knife
        // and needle until their smithy).
        let mut made: BTreeSet<&str> = world.plants.iter().map(|p| data.good(&p.good).id.as_str()).chain(world.items.iter().map(|it| data.good(it.good()).id.as_str())).collect();
        let mut can: BTreeSet<&str> = BTreeSet::new();
        let provided = |made: &BTreeSet<&str>, group: &Vec<String>| made.iter().any(|id| matches(&data.good(id).type_tags, group));
        loop {
            let before = made.len();
            for k in ShopKind::all() {
                let town = in_town.contains(&k);
                let frontier = k.kit().is_some_and(|kit| made.contains(kit.good));
                for &r in k.recipes() {
                    let g = data.good(r);
                    let inputs = g.input_goods_tags_required.iter().all(|grp| provided(&made, grp));
                    let tools = |lent: bool| g.tools_required_tags.iter().filter(|grp| !grp.is_empty()).all(|grp| provided(&made, grp) || (lent && k.builtin_tools().iter().any(|t| matches(&data.good(t).type_tags, grp))));
                    if inputs && ((town && tools(true)) || (frontier && tools(false))) {
                        can.insert(r);
                        made.insert(g.output());
                    }
                }
            }
            if made.len() == before {
                break;
            }
        }
        let mut missing = Vec::new();
        for k in ShopKind::all() {
            for &r in k.recipes().iter().filter(|r| !can.contains(*r)) {
                let g = data.good(r);
                let lent = |grp: &Vec<String>| in_town.contains(&k) && k.builtin_tools().iter().any(|t| matches(&data.good(t).type_tags, grp));
                let lacks: Vec<String> = g.input_goods_tags_required.iter().chain(&g.tools_required_tags).filter(|grp| !grp.is_empty() && !provided(&made, grp) && !lent(grp)).map(|grp| grp.join(" ")).collect();
                let place = if in_town.contains(&k) || k.kit().is_some() { "" } else { ", and none on the map or from a kit" };
                missing.push(format!("{r} at the {} (nothing makes: {}{place})", k.id(), lacks.join(", ")));
            }
        }
        assert!(missing.is_empty(), "can't be made from what grows on the map:\n{}", missing.join("\n"));
    }
}
