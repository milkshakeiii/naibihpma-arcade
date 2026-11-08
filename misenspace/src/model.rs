//! Directions, food, kitchen and dining-room cells, and the restaurant layout.

pub const N: i32 = 8;

#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash)]
pub enum Dir {
    N,
    E,
    S,
    W,
}

impl Dir {
    pub const ALL: [Dir; 4] = [Dir::N, Dir::E, Dir::S, Dir::W];

    pub fn dx(self) -> i32 {
        match self {
            Dir::E => 1,
            Dir::W => -1,
            _ => 0,
        }
    }

    pub fn dy(self) -> i32 {
        match self {
            Dir::S => 1,
            Dir::N => -1,
            _ => 0,
        }
    }

    pub fn opposite(self) -> Dir {
        match self {
            Dir::N => Dir::S,
            Dir::E => Dir::W,
            Dir::S => Dir::N,
            Dir::W => Dir::E,
        }
    }

    pub fn cw(self) -> Dir {
        match self {
            Dir::N => Dir::E,
            Dir::E => Dir::S,
            Dir::S => Dir::W,
            Dir::W => Dir::N,
        }
    }

    pub fn between(a: (i32, i32), b: (i32, i32)) -> Option<Dir> {
        Dir::ALL.into_iter().find(|d| a.0 + d.dx() == b.0 && a.1 + d.dy() == b.1)
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash)]
pub enum Item {
    Rye,
    Berry,
    Moss,
    Bean,
    Bread,
    Sliced,
    Roasted,
    Toast,
    Bowl,
    BeanPlate,
}

pub const ITEM_COUNT: usize = 10;

impl Item {
    pub fn index(self) -> usize {
        self as usize
    }

    pub fn name(self) -> &'static str {
        match self {
            Item::Rye => "ICE-RYE",
            Item::Berry => "FROSTBERRY",
            Item::Moss => "RADIO-MOSS",
            Item::Bean => "PULSAR BEANS",
            Item::Bread => "RYE BREAD",
            Item::Sliced => "SLICED BERRIES",
            Item::Roasted => "ROAST BEANS",
            Item::Toast => "BERRY TOAST",
            Item::Bowl => "MOSS BOWL",
            Item::BeanPlate => "BEAN PLATE",
        }
    }

    pub fn is_dish(self) -> bool {
        matches!(self, Item::Toast | Item::Bowl | Item::BeanPlate)
    }

    /// Coins a guest pays for the dish (tips come on top).
    pub fn price(self) -> u32 {
        match self {
            Item::Toast => 3,
            Item::Bowl => 2,
            Item::BeanPlate => 4,
            _ => 0,
        }
    }

    /// The parts a plating counter needs for this dish, with counts.
    pub fn recipe(self) -> &'static [(Item, u8)] {
        match self {
            Item::Toast => &[(Item::Bread, 1), (Item::Sliced, 2)],
            Item::Bowl => &[(Item::Bread, 1), (Item::Moss, 1)],
            Item::BeanPlate => &[(Item::Roasted, 2), (Item::Moss, 1)],
            _ => &[],
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Machine {
    /// Frostberry -> sliced berries.
    Slicer,
    /// Ice-rye -> rye bread, pulsar beans -> roast beans.
    Oven,
}

impl Machine {
    pub fn product(self, input: Item) -> Option<Item> {
        match (self, input) {
            (Machine::Slicer, Item::Berry) => Some(Item::Sliced),
            (Machine::Oven, Item::Rye) => Some(Item::Bread),
            (Machine::Oven, Item::Bean) => Some(Item::Roasted),
            _ => None,
        }
    }

    /// Beats one batch takes.
    pub fn beats(self) -> u8 {
        match self {
            Machine::Slicer => 2,
            Machine::Oven => 4,
        }
    }

    pub fn name(self) -> &'static str {
        match self {
            Machine::Slicer => "SLICER",
            Machine::Oven => "OVEN",
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum KCell {
    Floor,
    Wall,
    /// Fixed pantry hatch: puts out `item` toward `dir` every `period` beats.
    Hatch { item: Item, dir: Dir, period: u8 },
    Belt(Dir),
    /// Lifts from the tile behind it and sets down on the tile in front (toward `Dir`).
    Arm(Dir),
    Machine(Machine, Dir),
    /// Plating counter for one dish; finished dishes leave toward `Dir`.
    Counter(Item, Dir),
    /// Sends what goes in up to the same square in the dining room.
    Chute,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Tool {
    Belt,
    Arm,
    Machine(Machine),
    Counter(Item),
    Chute,
}

impl Tool {
    pub fn of(c: KCell) -> Option<Tool> {
        Some(match c {
            KCell::Belt(_) => Tool::Belt,
            KCell::Arm(_) => Tool::Arm,
            KCell::Machine(m, _) => Tool::Machine(m),
            KCell::Counter(d, _) => Tool::Counter(d),
            KCell::Chute => Tool::Chute,
            _ => return None,
        })
    }

    pub fn cell(self, d: Dir) -> KCell {
        match self {
            Tool::Belt => KCell::Belt(d),
            Tool::Arm => KCell::Arm(d),
            Tool::Machine(m) => KCell::Machine(m, d),
            Tool::Counter(i) => KCell::Counter(i, d),
            Tool::Chute => KCell::Chute,
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum DCell {
    Floor,
    Wall,
    Table,
    /// A seat at the table on its `Dir` side.
    Seat(Dir),
    Door,
    Plant,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Species {
    /// Survey crew in orange parkas.
    Crew,
    /// Round, green, cheerful.
    Blob,
    /// Tall and violet, with an antenna.
    Stalk,
}

impl Species {
    pub fn name(self) -> &'static str {
        match self {
            Species::Crew => "SURVEY CREW",
            Species::Blob => "GLOOB",
            Species::Stalk => "ANTENNID",
        }
    }
}

#[inline]
pub fn idx(x: i32, y: i32) -> usize {
    (y * N + x) as usize
}

#[inline]
pub fn inside(x: i32, y: i32) -> bool {
    (0..N).contains(&x) && (0..N).contains(&y)
}

pub fn square_name(x: i32, y: i32) -> String {
    format!("{}{}", (b'A' + x as u8) as char, y + 1)
}

/// One evening: when guests arrive and which dishes are on the menu.
#[derive(Clone, Debug)]
pub struct Service {
    pub beats: u32,
    /// (beat, species)
    pub arrivals: Vec<(u32, Species)>,
    /// (from beat, dish) — a dish joins the menu at that beat.
    pub menu: Vec<(u32, Item)>,
    pub patience: u16,
    /// Beats a guest spends eating.
    pub eat_beats: u16,
    /// Fed share needed for 1, 2, 3 stars (percent).
    pub stars: [u32; 3],
}

#[derive(Clone, Debug)]
pub struct Restaurant {
    pub name: &'static str,
    pub note: &'static str,
    pub kitchen: [KCell; 64],
    pub dining: [DCell; 64],
    pub shelf: Vec<(Tool, Option<u8>)>,
    pub service: Service,
}

impl Restaurant {
    pub fn menu_at(&self, beat: u32) -> Vec<Item> {
        self.service.menu.iter().filter(|(b, _)| *b <= beat).map(|(_, d)| *d).collect()
    }

    pub fn table_at(&self, x: i32, y: i32) -> bool {
        inside(x, y) && self.dining[idx(x, y)] == DCell::Table
    }
}

/// Object 25 Canteen: a survey outpost on an ice planetoid.
pub fn object25() -> Restaurant {
    // Kitchen: pantry hatches through the west wall, two coolant pillars.
    const K: [&str; 8] = ["F.......", "........", "R.......", "......#.", "M.......", ".....#..", "B.......", "........"];
    // Dining room: three tables with seats (arrows point at their table), door at the south-west.
    const D: [&str; 8] = ["......v.", "..p..>T<", "p.....^.", ".......v", "......>T", ".......^", ".....v..", "D...>T<."];
    let mut kitchen = [KCell::Floor; 64];
    let mut dining = [DCell::Floor; 64];
    for y in 0..8 {
        for (x, ch) in K[y].chars().enumerate() {
            let hatch = |item, period| KCell::Hatch { item, dir: Dir::E, period };
            kitchen[y * 8 + x] = match ch {
                '#' => KCell::Wall,
                'R' => hatch(Item::Rye, 2),
                'F' => hatch(Item::Berry, 2),
                'M' => hatch(Item::Moss, 3),
                'B' => hatch(Item::Bean, 3),
                _ => KCell::Floor,
            };
        }
        for (x, ch) in D[y].chars().enumerate() {
            dining[y * 8 + x] = match ch {
                'T' => DCell::Table,
                '>' => DCell::Seat(Dir::E),
                '<' => DCell::Seat(Dir::W),
                'v' => DCell::Seat(Dir::S),
                '^' => DCell::Seat(Dir::N),
                'D' => DCell::Door,
                'p' => DCell::Plant,
                '#' => DCell::Wall,
                _ => DCell::Floor,
            };
        }
    }
    use Species::*;
    // A calm start, a steady middle, a rush, then last orders.
    let mut arrivals = Vec::new();
    let mut b = 6u32;
    let mut k = 0usize;
    let kinds = [Crew, Blob, Crew, Stalk, Blob, Crew, Stalk, Crew, Blob, Stalk];
    while b < 900 {
        arrivals.push((b, kinds[k % kinds.len()]));
        k += 1;
        b += match b {
            0..=200 => 28,
            201..=560 => 15,
            561..=780 => 9,
            _ => 18,
        };
    }
    Restaurant {
        name: "OBJECT 25 CANTEEN",
        note: "A CHUTE SENDS A DISH UP THROUGH THE FLOOR TO THE SAME SQUARE, BUT ONLY ONE THAT SOMEONE AT THAT TABLE IS WAITING FOR.",
        kitchen,
        dining,
        shelf: vec![
            (Tool::Belt, None),
            (Tool::Arm, Some(10)),
            (Tool::Machine(Machine::Slicer), Some(2)),
            (Tool::Machine(Machine::Oven), Some(3)),
            (Tool::Counter(Item::Toast), Some(2)),
            (Tool::Counter(Item::Bowl), Some(2)),
            (Tool::Counter(Item::BeanPlate), Some(1)),
            (Tool::Chute, Some(3)),
        ],
        service: Service {
            beats: 1000,
            arrivals,
            menu: vec![(0, Item::Toast), (0, Item::Bowl), (320, Item::BeanPlate)],
            patience: 120,
            eat_beats: 30,
            stars: [50, 75, 92],
        },
    }
}
