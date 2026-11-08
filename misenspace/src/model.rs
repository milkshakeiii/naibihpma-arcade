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
    /// (from beat, dish): a dish joins the menu at that beat.
    pub menu: Vec<(u32, Item)>,
    pub patience: u16,
    /// Beats a guest spends eating.
    pub eat_beats: u16,
    /// Fed share needed for 1, 2, 3 stars (percent).
    pub stars: [u32; 3],
}

/// What finishes a lesson step.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Cond {
    /// The doors are open.
    Running,
    /// Some guest has sat down and ordered.
    Seated,
    /// A belt pointing this way on this kitchen square.
    Belt(i32, i32, Dir),
    /// A grabber arm pointing this way on this kitchen square.
    Arm(i32, i32, Dir),
    /// The counter on this square has received this part (or already plated).
    Part(i32, i32, Item),
    /// This many guests fed.
    Fed(u32),
}

/// One instruction in a lesson, with the kitchen squares it points at.
#[derive(Clone, Debug)]
pub struct Step {
    pub text: &'static str,
    pub show: &'static [(i32, i32)],
    pub done: Cond,
}

#[derive(Clone, Debug)]
pub struct Restaurant {
    pub name: &'static str,
    pub note: &'static str,
    pub kitchen: [KCell; 64],
    pub dining: [DCell; 64],
    pub shelf: Vec<(Tool, Option<u8>)>,
    pub service: Service,
    pub steps: Vec<Step>,
    /// Lessons end once this many guests are fed; restaurants end at closing time.
    pub goal_fed: Option<u32>,
}

impl Restaurant {
    pub fn menu_at(&self, beat: u32) -> Vec<Item> {
        self.service.menu.iter().filter(|(b, _)| *b <= beat).map(|(_, d)| *d).collect()
    }

    pub fn table_at(&self, x: i32, y: i32) -> bool {
        inside(x, y) && self.dining[idx(x, y)] == DCell::Table
    }

    pub fn is_lesson(&self) -> bool {
        self.goal_fed.is_some()
    }
}

/// One kitchen token: `..` floor, `##` wall, `r> f> m> b>` pantry hatches (rye, frostberry,
/// moss, beans; the arrow is where they face), `B>` belt, `A>` arm, `S>` slicer, `O>` oven,
/// `T> W> P>` toast/bowl/bean-plate counters, `CH` chute. `None` for `..`.
pub fn kitchen_token(tok: &str) -> Option<KCell> {
    let b = tok.as_bytes();
    let d = match b[1] {
        b'^' => Dir::N,
        b'>' => Dir::E,
        b'v' => Dir::S,
        _ => Dir::W,
    };
    let hatch = |item, period| Some(KCell::Hatch { item, dir: d, period });
    match b[0] {
        b'.' => None,
        b'#' => Some(KCell::Wall),
        b'r' => hatch(Item::Rye, 2),
        b'f' => hatch(Item::Berry, 2),
        b'm' => hatch(Item::Moss, 3),
        b'b' => hatch(Item::Bean, 3),
        b'B' => Some(KCell::Belt(d)),
        b'A' => Some(KCell::Arm(d)),
        b'S' => Some(KCell::Machine(Machine::Slicer, d)),
        b'O' => Some(KCell::Machine(Machine::Oven, d)),
        b'T' => Some(KCell::Counter(Item::Toast, d)),
        b'W' => Some(KCell::Counter(Item::Bowl, d)),
        b'P' => Some(KCell::Counter(Item::BeanPlate, d)),
        b'C' => Some(KCell::Chute),
        _ => panic!("bad kitchen token {tok}"),
    }
}

/// A kitchen from 8 rows of 8 tokens.
pub fn kitchen_rows(rows: [&str; 8]) -> [KCell; 64] {
    let mut k = [KCell::Floor; 64];
    for (y, row) in rows.iter().enumerate() {
        for (x, tok) in row.split_whitespace().enumerate() {
            if let Some(c) = kitchen_token(tok) {
                k[y * 8 + x] = c;
            }
        }
    }
    k
}

/// A dining room from 8 rows of 8 characters: `T` table, `> < v ^` seats pointing at their
/// table, `D` door, `p` plant, `#` wall.
pub fn dining_rows(rows: [&str; 8]) -> [DCell; 64] {
    let mut d = [DCell::Floor; 64];
    for (y, row) in rows.iter().enumerate() {
        for (x, ch) in row.chars().enumerate() {
            d[y * 8 + x] = match ch {
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
    d
}

/// Guests at a steady interval, cycling through the species.
fn steady(first: u32, every: u32, count: usize) -> Vec<(u32, Species)> {
    use Species::*;
    let kinds = [Crew, Blob, Stalk];
    (0..count).map(|k| (first + every * k as u32, kinds[k % 3])).collect()
}

fn lesson_service(menu: Vec<(u32, Item)>, every: u32, count: usize) -> Service {
    Service { beats: 1600, arrivals: steady(4, every, count), menu, patience: 1400, eat_beats: 30, stars: [0, 0, 0] }
}

/// Lesson 1: one moss bowl line, built a step at a time.
pub fn lesson_first_bowl() -> Restaurant {
    Restaurant {
        name: "LESSON 1: FIRST BOWL",
        note: "SERVE THREE GUESTS.",
        kitchen: kitchen_rows([
            ".. .. .. .. .. .. .. ..",
            "r> .. O> .. .. .. .. ..",
            ".. .. .. .. .. .. .. ..",
            "m> .. .. .. Wv .. .. ..",
            ".. .. .. .. .. .. .. ..",
            ".. .. .. .. CH .. .. ..",
            ".. .. .. .. .. .. .. ..",
            ".. .. .. .. .. .. .. ..",
        ]),
        dining: dining_rows(["........", ".p....p.", "........", "........", "........", "...>T<..", "....^...", ".......D"]),
        shelf: vec![(Tool::Belt, None)],
        service: lesson_service(vec![(0, Item::Bowl)], 40, 8),
        steps: vec![
            Step {
                text: "THE KITCHEN IS ON THE LEFT, THE DINING ROOM ON THE RIGHT. GUESTS COME IN THROUGH THE AIRLOCK (BOTTOM RIGHT) AND SIT AT TABLES. PRESS ENTER TO OPEN THE DOORS.",
                show: &[],
                done: Cond::Running,
            },
            Step {
                text: "A GUEST SITS AT TABLE E6 AND ORDERS A MOSS BOWL. SEE THE BUBBLE OVER THEIR HEAD, AND THE ORDER'S TICKET ON THE RAIL BETWEEN THE ROOMS.",
                show: &[],
                done: Cond::Seated,
            },
            Step {
                text: "THE COUNTER AT E4 PLATES MOSS BOWLS FROM 1 RYE BREAD + 1 RADIO-MOSS. BREAD FIRST: PUT A BELT AT B2 (SPACE OR CLICK) SO ICE-RYE ROLLS INTO THE OVEN.",
                show: &[(1, 1)],
                done: Cond::Belt(1, 1, Dir::E),
            },
            Step {
                text: "THE OVEN BAKES BREAD AND PUSHES IT OUT OF ITS ARROW SIDE. HOLD SPACE (OR DRAG) TO DRAW BELTS FROM D2 RIGHT TO E2, THEN DOWN INTO THE COUNTER.",
                show: &[(3, 1), (4, 1), (4, 2)],
                done: Cond::Part(4, 3, Item::Bread),
            },
            Step {
                text: "NOW THE MOSS: DRAW BELTS FROM B4 TO D4 SO RADIO-MOSS FROM THE HATCH ROLLS INTO THE COUNTER TOO.",
                show: &[(1, 3), (2, 3), (3, 3)],
                done: Cond::Part(4, 3, Item::Moss),
            },
            Step {
                text: "A COUNTER PLATES A DISH ONLY WHEN SOMEONE HAS ORDERED IT, THEN PUSHES IT OUT OF ITS ARROW (SOUTH). A BELT AT E5 TAKES IT INTO THE CHUTE AT E6, RIGHT UNDER TABLE E6.",
                show: &[(4, 4)],
                done: Cond::Fed(1),
            },
            Step {
                text: "SERVED! EACH GUEST PAYS, PLUS A TIP FOR SPEED. LEAVE THE KITCHEN RUNNING AND SERVE TWO MORE GUESTS.",
                show: &[],
                done: Cond::Fed(3),
            },
        ],
        goal_fed: Some(3),
    }
}

/// Lesson 2: two tables, so the dishes need to ride a loop past both chutes.
pub fn lesson_two_tables() -> Restaurant {
    Restaurant {
        name: "LESSON 2: TWO TABLES",
        note: "SERVE SIX GUESTS.",
        kitchen: kitchen_rows([
            "f> S> B> Bv .. .. .. ..",
            ".. .. .. T> .. .. CH ..",
            "r> O> B> B^ .. .. .. ..",
            ".. .. .. Av .. .. .. ..",
            "m> B> B> W> .. .. .. ..",
            ".. .. .. .. .. .. CH ..",
            ".. .. .. .. .. .. .. ..",
            ".. .. .. .. .. .. .. ..",
        ]),
        dining: dining_rows(["........", ".....>T<", "..p...^.", "........", "........", ".....>T<", "......^.", "D......p"]),
        shelf: vec![(Tool::Belt, None)],
        service: lesson_service(vec![(0, Item::Toast), (0, Item::Bowl)], 30, 12),
        steps: vec![
            Step {
                text: "THIS KITCHEN ALREADY MAKES BERRY TOAST (COUNTER D2) AND MOSS BOWLS (D5). PIECES WITH BOLTS ARE PART OF THE STATION. PRESS ENTER TO OPEN.",
                show: &[],
                done: Cond::Running,
            },
            Step {
                text: "TWO TABLES THIS TIME, G2 AND G6, WITH A CHUTE UNDER EACH. A CHUTE ONLY SENDS UP A DISH SOMEONE AT ITS TABLE ORDERED. ANY OTHER DISH ROLLS STRAIGHT THROUGH.",
                show: &[(6, 1), (6, 5)],
                done: Cond::Seated,
            },
            Step {
                text: "SO BUILD A LOOP: BELTS FROM BOTH COUNTERS (STARTING AT E2 AND E5) THROUGH BOTH CHUTES AND BACK ROUND, SO EVERY DISH PASSES EVERY TABLE UNTIL SOMEONE TAKES IT.",
                show: &[(4, 1), (4, 4)],
                done: Cond::Fed(2),
            },
            Step {
                text: "A FULL LOOP KEEPS TURNING, AND COUNTERS ONLY PLATE WHAT'S ORDERED, SO NOTHING PILES UP. SERVE SIX GUESTS.",
                show: &[],
                done: Cond::Fed(6),
            },
        ],
        goal_fed: Some(6),
    }
}

/// Lesson 3: grabber arms split a shared bread line between two counters.
pub fn lesson_arms() -> Restaurant {
    Restaurant {
        name: "LESSON 3: ARMS",
        note: "SERVE FIVE GUESTS.",
        kitchen: kitchen_rows([
            ".. .. .. .. .. .. .. ..",
            ".. .. .. CH .. .. CH ..",
            ".. .. .. B^ .. .. B^ ..",
            "m> B> B> W^ .. .. W^ m<",
            ".. .. .. .. .. .. .. ..",
            "r> O> B> B> B> B> B> B>",
            ".. .. .. .. .. .. .. ..",
            ".. .. .. .. .. .. .. ..",
        ]),
        dining: dining_rows(["........", "..>T<>T<", "...^..^.", "........", ".p....p.", "........", "........", "D......."]),
        shelf: vec![(Tool::Belt, None), (Tool::Arm, Some(2))],
        service: lesson_service(vec![(0, Item::Bowl)], 28, 10),
        steps: vec![
            Step {
                text: "TWO MOSS BOWL COUNTERS (D4, G4) GET THEIR MOSS, BUT THE BREAD ROLLS PAST ON THE BELT BELOW THEM. PRESS ENTER TO OPEN.",
                show: &[],
                done: Cond::Running,
            },
            Step {
                text: "A GRABBER ARM LIFTS FROM THE TILE BEHIND IT AND SETS DOWN ON THE TILE IN FRONT. PICK THE ARM (KEY 2) AND PLACE ONE AT D5. R TURNS IT: POINT ITS ARROW UP.",
                show: &[(3, 4)],
                done: Cond::Arm(3, 4, Dir::N),
            },
            Step {
                text: "ARMS ONLY LIFT WHAT THE TILE IN FRONT CAN USE, SO SPARE BREAD ROLLS ON. PUT A SECOND ARM AT G5, POINTING UP AT THE OTHER COUNTER.",
                show: &[(6, 4)],
                done: Cond::Arm(6, 4, Dir::N),
            },
            Step {
                text: "THAT'S EVERY PIECE EXCEPT THE SLICER. SERVE FIVE GUESTS, THEN IT'S THE REAL CANTEEN.",
                show: &[],
                done: Cond::Fed(5),
            },
        ],
        goal_fed: Some(5),
    }
}

/// Object 25 Canteen: a survey outpost on an ice planetoid.
pub fn object25() -> Restaurant {
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
        // Pantry hatches through the west wall, two coolant pillars.
        kitchen: kitchen_rows([
            "f> .. .. .. .. .. .. ..",
            ".. .. .. .. .. .. .. ..",
            "r> .. .. .. .. .. .. ..",
            ".. .. .. .. .. .. ## ..",
            "m> .. .. .. .. .. .. ..",
            ".. .. .. .. .. ## .. ..",
            "b> .. .. .. .. .. .. ..",
            ".. .. .. .. .. .. .. ..",
        ]),
        // Three tables with seats, the door at the south-west.
        dining: dining_rows(["........", "..p..>T<", "p.....^.", ".......v", "......>T", ".......^", ".....v..", "D...>T<."]),
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
        steps: vec![
            Step {
                text: "THE REAL CANTEEN: THREE TABLES, THREE DISHES, A BUSY EVENING. BERRY TOAST NEEDS 2 SLICED BERRIES - A SLICER MAKES THEM FROM FROSTBERRIES. BUILD, THEN ENTER.",
                show: &[],
                done: Cond::Running,
            },
            Step {
                text: "BEAN PLATES JOIN THE MENU LATER (OVENS ROAST PULSAR BEANS TOO). YOU CAN BUILD MID-SERVICE, PAUSE ANY TIME, AND F5 RESTARTS THE EVENING.",
                show: &[],
                done: Cond::Fed(4),
            },
        ],
        goal_fed: None,
    }
}

/// The lessons, then the canteen.
pub fn levels() -> Vec<Restaurant> {
    vec![lesson_first_bowl(), lesson_two_tables(), lesson_arms(), object25()]
}
