//! A player's solution: what they built and which creatures haul what between where.

use crate::rules::*;

#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash, PartialOrd, Ord)]
pub enum NodeRef {
    Source(usize),
    Fortress,
    Building(u32),
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Placed {
    pub id: u32,
    pub kind: Kind,
    /// Top-left tile of the 2x2 footprint.
    pub x: i32,
    pub y: i32,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Route {
    pub species: Species,
    pub from: NodeRef,
    pub to: NodeRef,
    pub cargo: Good,
    pub count: i32,
}

#[derive(Clone, PartialEq, Eq, Debug, Default)]
pub struct Layout {
    pub buildings: Vec<Placed>,
    pub springs: Vec<(i32, i32)>,
    /// Top tile of each ladder; it hangs down to the first floor.
    pub ladders: Vec<(i32, i32)>,
    pub planks: Vec<(i32, i32)>,
    pub routes: Vec<Route>,
    pub next_id: u32,
}

impl Layout {
    pub fn building(&self, id: u32) -> Option<&Placed> {
        self.buildings.iter().find(|b| b.id == id)
    }

    pub fn add_building(&mut self, kind: Kind, x: i32, y: i32) -> u32 {
        let id = self.next_id;
        self.next_id += 1;
        self.buildings.push(Placed { id, kind, x, y });
        id
    }

    /// Removes a building and every route that touches it.
    pub fn remove_building(&mut self, id: u32) {
        self.buildings.retain(|b| b.id != id);
        self.routes.retain(|r| r.from != NodeRef::Building(id) && r.to != NodeRef::Building(id));
    }

    pub fn crew(&self) -> i32 {
        self.routes.iter().map(|r| r.count).sum()
    }

    /// Adds creatures to a matching route, or starts a new one.
    pub fn add_route(&mut self, species: Species, from: NodeRef, to: NodeRef, cargo: Good, n: i32) {
        if let Some(r) = self.routes.iter_mut().find(|r| r.species == species && r.from == from && r.to == to && r.cargo == cargo) {
            r.count = (r.count + n).min(9);
        } else {
            self.routes.push(Route { species, from, to, cargo, count: n.clamp(1, 9) });
        }
    }

    fn node_text(&self, n: NodeRef) -> String {
        match n {
            NodeRef::Source(i) => format!("s{i}"),
            NodeRef::Fortress => "F".into(),
            NodeRef::Building(id) => {
                let idx = self.buildings.iter().position(|b| b.id == id).unwrap_or(0);
                format!("b{idx}")
            }
        }
    }

    /// One item per `;`: `B KIND x y`, `S x y`, `L x y`, `P x y`, `R SPECIES from to CARGO n`.
    pub fn to_text(&self) -> String {
        let mut parts: Vec<String> = Vec::new();
        for b in &self.buildings {
            parts.push(format!("B {} {} {}", b.kind.name(), b.x, b.y));
        }
        for (x, y) in &self.springs {
            parts.push(format!("S {x} {y}"));
        }
        for (x, y) in &self.ladders {
            parts.push(format!("L {x} {y}"));
        }
        for (x, y) in &self.planks {
            parts.push(format!("P {x} {y}"));
        }
        for r in &self.routes {
            parts.push(format!("R {} {} {} {} {}", r.species.name(), self.node_text(r.from), self.node_text(r.to), r.cargo.name(), r.count));
        }
        parts.join(";")
    }

    pub fn from_text(text: &str) -> Layout {
        let mut l = Layout::default();
        let node = |l: &Layout, s: &str| -> Option<NodeRef> {
            if s == "F" {
                return Some(NodeRef::Fortress);
            }
            let (head, num) = s.split_at(1);
            let i: usize = num.parse().ok()?;
            match head {
                "s" => Some(NodeRef::Source(i)),
                "b" => l.buildings.get(i).map(|b| NodeRef::Building(b.id)),
                _ => None,
            }
        };
        for item in text.split(';') {
            let p: Vec<&str> = item.split_whitespace().collect();
            let num = |i: usize| p.get(i).and_then(|s| s.parse::<i32>().ok());
            match p.first().copied() {
                Some("B") => {
                    if let (Some(k), Some(x), Some(y)) = (p.get(1).and_then(|s| Kind::from_name(s)), num(2), num(3)) {
                        l.add_building(k, x, y);
                    }
                }
                Some("S") => {
                    if let (Some(x), Some(y)) = (num(1), num(2)) {
                        l.springs.push((x, y));
                    }
                }
                Some("L") => {
                    if let (Some(x), Some(y)) = (num(1), num(2)) {
                        l.ladders.push((x, y));
                    }
                }
                Some("P") => {
                    if let (Some(x), Some(y)) = (num(1), num(2)) {
                        l.planks.push((x, y));
                    }
                }
                Some("R") => {
                    let sp = p.get(1).and_then(|s| Species::from_name(s));
                    let from = p.get(2).and_then(|s| node(&l, s));
                    let to = p.get(3).and_then(|s| node(&l, s));
                    let cargo = p.get(4).and_then(|s| Good::from_name(s));
                    if let (Some(sp), Some(from), Some(to), Some(cargo)) = (sp, from, to, cargo) {
                        l.routes.push(Route { species: sp, from, to, cargo, count: num(5).unwrap_or(1).clamp(1, 9) });
                    }
                }
                _ => {}
            }
        }
        l
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn text_round_trips() {
        let mut l = Layout::default();
        let d = l.add_building(Kind::Depot, 5, 10);
        l.springs.push((3, 4));
        l.ladders.push((7, 2));
        l.planks.push((9, 9));
        l.add_route(Species::Dodo, NodeRef::Source(1), NodeRef::Building(d), Good::Twig, 2);
        l.add_route(Species::Tribble, NodeRef::Building(d), NodeRef::Fortress, Good::Twig, 1);
        let t = l.to_text();
        assert_eq!(Layout::from_text(&t).to_text(), t);
    }
}
