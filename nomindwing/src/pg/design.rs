//! Proving-ground designs: a 41x41 pixel grid where every colour is a part, and the stats that
//! drawing implies. Nothing here knows what "kind" of ship a design is: a fighter is just a
//! small drawing with engines to spare.
//!
//! Round 2 rules: every pixel must connect to the core through other pixels (edges, not
//! corners); plate shrugs off hits no stronger than it; the cannon is a 3x3 block.
//! Round 3: extra cores are allowed at a steeply rising cost, and every engine needs a 3-wide
//! way out of the ship for its exhaust (straight out since round 4, so it's always in the open).
//! Round 4: ships up to 99x99 (odd, so the mirror has a centre column), parts sorted into
//! categories, and the editing tools big ships need: rectangles, fills and stamps.

use crate::palette as pal;

pub const GRID: i32 = 99;
pub const FLAK_RANGE: i32 = 48;
pub const LANCE_RANGE: i32 = 100;
pub const CANNON_RANGE: i32 = 130;
/// Ticks between shots, per gun.
pub const FLAK_PERIOD: i32 = 20;
pub const LANCE_PERIOD: i32 = 45;
pub const CANNON_PERIOD: i32 = 45;
/// Damage per hit. A hit no stronger than a pixel's armour bounces off.
pub const FLAK_DMG: i32 = 1;
pub const LANCE_DMG: i32 = 2;
pub const CANNON_DMG: i32 = 3;
/// How far a lance shot punches: a bare pixel uses up 1, plate 2.
pub const LANCE_PEN: i32 = 6;
/// Ticks a ship with two or more scoops takes to bite a pixel out of a rock (one scoop takes
/// twice as long). 40 until 2026-11-11.
pub const CHEW_TICKS: i32 = 50;
/// Pixels each scoop carries.
pub const SCOOP_CARRY: i32 = 3;
/// The first line of a save file. Round 4 saves each design trimmed to its drawing, with where it
/// sits on the grid; round 2 and 3 files saved the whole 41x41 grid and still load, centred.
pub const SAVE_HEADER: &str = "#ROUND4";
pub const OLD_SAVE_HEADER: &str = "#ROUND2";
/// Ticks for one factory pixel to print one ship pixel (4 s).
pub const PRINT_TICKS: i32 = 400;
/// Ship pixels a factory can build per factory pixel: the biggest ship it can make.
pub const FACTORY_REACH: i32 = 10;
/// What the second core costs on top of the ship's pixels. Each core after that costs twice the
/// one before: +8, +16, +32 and so on.
pub const CORE_EXTRA: i32 = 8;

/// The extra cost of a ship's cores beyond the first.
pub fn core_cost(cores: i32) -> i32 {
    if cores <= 1 {
        0
    } else {
        CORE_EXTRA.saturating_mul((1 << (cores - 1).min(24)) - 1)
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash)]
pub enum Part {
    Hull,
    Plate,
    Flak,
    Lance,
    Cannon,
    Engine,
    Scoop,
    Core,
    /// Prints ships (match only): speed and the biggest ship it can build come from the count.
    Factory,
}

/// The parts in the order the yard shows them.
pub const PARTS: [Part; 9] = [Part::Hull, Part::Plate, Part::Flak, Part::Lance, Part::Cannon, Part::Engine, Part::Scoop, Part::Factory, Part::Core];

/// The yard's part categories, keys 1-6. Power is empty until its parts arrive.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Cat {
    Armor,
    Weapons,
    Drive,
    Power,
    Industry,
    Command,
}

pub const CATS: [Cat; 6] = [Cat::Armor, Cat::Weapons, Cat::Drive, Cat::Power, Cat::Industry, Cat::Command];

impl Cat {
    pub fn name(self) -> &'static str {
        match self {
            Cat::Armor => "HULL AND ARMOR",
            Cat::Weapons => "WEAPONS",
            Cat::Drive => "DRIVE",
            Cat::Power => "POWER",
            Cat::Industry => "INDUSTRY",
            Cat::Command => "COMMAND",
        }
    }
    /// The category's parts, in the yard's order.
    pub fn parts(self) -> Vec<Part> {
        PARTS.iter().copied().filter(|p| p.info().cat == self).collect()
    }
    pub fn of(p: Part) -> Cat {
        p.info().cat
    }
}

/// Everything about a part that isn't behaviour, in one place. A new part is a variant of `Part`,
/// a row here and a place in `PARTS`; the yard, the stats, the save format and the drawing all
/// read from this.
pub struct PartInfo {
    /// Its letter in designs and save files.
    pub ch: char,
    pub name: &'static str,
    pub cat: Cat,
    /// Weight of one pixel.
    pub mass: i32,
    /// Hits this strong or weaker bounce off.
    pub armor: i32,
    /// Power one pixel makes, and what one uses (round 5). A cannon's is for the whole block.
    pub makes: i32,
    pub uses: i32,
    /// Pixels in one of it: 9 for the cannon's 3x3 block, 1 for everything else.
    pub block: i32,
    /// Its palette colour. None is the team's own colour (hull).
    pub colour: Option<u8>,
    /// Which part shows when a shrunken icon has one pixel for a block of cells: highest wins.
    pub rank: i32,
    /// The 7x7 shape drawn inside its colour, so parts stay easy to tell apart.
    pub glyph: [&'static str; 7],
    /// What it does. Its numbers come from the constants; power and weight are added from above.
    pub about: fn() -> String,
}

/// A number of ticks as seconds, to a tenth: "1.3", or "3" when it's whole.
pub fn secs(ticks: i32) -> String {
    let tenths = ticks * 10 / 15;
    if tenths % 10 == 0 {
        format!("{}", tenths / 10)
    } else {
        format!("{}.{}", tenths / 10, tenths % 10)
    }
}

/// One row per part, in the order of the `Part` enum.
pub const PART_INFO: [PartInfo; 9] = [
    PartInfo {
        ch: 'h',
        name: "HULL",
        cat: Cat::Armor,
        mass: 1,
        armor: 0,
        makes: 1,
        uses: 0,
        block: 1,
        colour: None,
        rank: 0,
        glyph: [".......", ".......", ".......", ".......", ".......", ".......", "......."],
        about: || "HOLDS THE SHIP TOGETHER, AND IS WHERE ITS POWER COMES FROM. ANY HIT BREAKS IT.".into(),
    },
    PartInfo {
        ch: 'p',
        name: "PLATE",
        cat: Cat::Armor,
        mass: 2,
        armor: 1,
        makes: 0,
        uses: 0,
        block: 1,
        colour: Some(pal::STEEL),
        rank: 1,
        glyph: [".......", ".#...#.", ".......", ".......", ".......", ".#...#.", "......."],
        about: || "FLAK BOUNCES OFF IT. LANCES AND CANNON SHELLS BREAK IT, AND IT SOAKS TWICE AS MUCH OF A LANCE.".into(),
    },
    PartInfo {
        ch: 'g',
        name: "FLAK",
        cat: Cat::Weapons,
        mass: 1,
        armor: 0,
        makes: 0,
        uses: 1,
        block: 1,
        colour: Some(pal::GUN),
        rank: 4,
        glyph: [".......", "...#...", "...#...", "..###..", "..###..", ".......", "......."],
        about: || format!("A TURRET. SHOOTS THE NEAREST ENEMY WITHIN {FLAK_RANGE} PX, LEADING ITS AIM AND SPREADING FIRE OVER SEVERAL TARGETS. TOO WEAK FOR PLATE. ONE SHOT EVERY {} S.", secs(FLAK_PERIOD)),
    },
    PartInfo {
        ch: 'l',
        name: "LANCE",
        cat: Cat::Weapons,
        mass: 2,
        armor: 0,
        makes: 0,
        uses: 3,
        block: 1,
        colour: Some(pal::LANCE),
        rank: 5,
        glyph: ["...#...", "..###..", ".#.#.#.", "...#...", "...#...", "...#...", "......."],
        about: || format!("SHOOTS STRAIGHT AHEAD ONLY, UP TO {LANCE_RANGE} PX. PUNCHES THROUGH {LANCE_PEN} PIXELS, OR {} OF PLATE. ONE SHOT EVERY {} S.", LANCE_PEN / 2, secs(LANCE_PERIOD)),
    },
    PartInfo {
        ch: 'n',
        name: "CANNON",
        cat: Cat::Weapons,
        mass: 2,
        armor: 0,
        makes: 0,
        uses: 8,
        block: 9,
        colour: Some(pal::CANNON),
        rank: 6,
        glyph: [".......", "..###..", ".#...#.", ".#...#.", ".#...#.", "..###..", "......."],
        about: || format!("A BIG 3X3 TURRET, {CANNON_RANGE} PX. IT TURNS SLOWLY, SO FAST CLOSE SHIPS ESCAPE IT. ITS SHELL BLASTS A 3X3 HOLE, PLATE AND ALL. IT GOES FOR THE BIG SHIPS IN RANGE. ONE SHOT EVERY {} S.", secs(CANNON_PERIOD)),
    },
    PartInfo {
        ch: 'e',
        name: "ENGINE",
        cat: Cat::Drive,
        mass: 1,
        armor: 0,
        makes: 0,
        uses: 1,
        block: 1,
        colour: Some(pal::ENGINE),
        rank: 3,
        glyph: ["#.....#", ".#...#.", "..#.#..", "#..#..#", ".#...#.", "..#.#..", "...#..."],
        about: || "THRUST: MORE OF IT FOR THE WEIGHT MEANS FASTER AND QUICKER TO TURN. ITS EXHAUST NEEDS A STRAIGHT WAY OUT, 3 WIDE, IN ONE OF THE FOUR DIRECTIONS. A BURIED ENGINE SHOWS RED.".into(),
    },
    PartInfo {
        ch: 's',
        name: "SCOOP",
        cat: Cat::Industry,
        mass: 2,
        armor: 0,
        makes: 0,
        uses: 1,
        block: 1,
        colour: Some(pal::SCOOP),
        rank: 2,
        glyph: [".......", ".#...#.", ".#...#.", ".#...#.", "..###..", ".......", "......."],
        about: || format!("MINES ROCK AND PICKS UP LOOSE PIXELS. EACH SCOOP CARRIES {SCOOP_CARRY}."),
    },
    PartInfo {
        ch: 'c',
        name: "CORE",
        cat: Cat::Command,
        mass: 1,
        armor: 0,
        makes: 4,
        uses: 0,
        block: 1,
        colour: Some(pal::CORE),
        rank: 8,
        glyph: ["...#...", "..#.#..", ".#...#.", "#.....#", ".#...#.", "..#.#..", "...#..."],
        about: || format!("THE SHIP'S LIFE: IT DIES WHEN ITS LAST CORE GOES. EXTRA CORES COST +{}, +{}, +{}. CUT IN TWO, EVERY PIECE WITH A CORE FLIES ON AS A SHIP.", CORE_EXTRA, CORE_EXTRA * 2, CORE_EXTRA * 4),
    },
    PartInfo {
        ch: 'f',
        name: "FACTORY",
        cat: Cat::Industry,
        mass: 1,
        armor: 0,
        makes: 0,
        uses: 2,
        block: 1,
        colour: Some(pal::FACTORY),
        rank: 7,
        glyph: [".......", ".#.#.#.", ".......", ".#.#.#.", ".......", ".#.#.#.", "......."],
        about: || format!("IN A MATCH, PRINTS ITS BUILD ORDER FROM YOUR BANK: 1 PIXEL EVERY {} S EACH, AND SHIPS UP TO {FACTORY_REACH} PX BIGGER EACH. A SHIP WITH ANY STORES PIXELS AND LINKS YOUR NETWORK.", secs(PRINT_TICKS)),
    },
];

impl Part {
    pub fn info(self) -> &'static PartInfo {
        &PART_INFO[self as usize]
    }
    /// Hits this strong or weaker bounce off.
    pub fn armor(self) -> i32 {
        self.info().armor
    }
    pub fn mass(self) -> i32 {
        self.info().mass
    }
    pub fn ch(self) -> char {
        self.info().ch
    }
    pub fn from_ch(c: char) -> Option<Part> {
        PARTS.iter().copied().find(|p| p.ch() == c)
    }
    pub fn name(self) -> &'static str {
        self.info().name
    }
    /// The yard's description: what it does, then its power and weight from the table.
    pub fn blurb(self) -> String {
        let i = self.info();
        let power = match (i.makes, i.uses) {
            (0, 0) => "NO POWER".to_string(),
            (m, 0) => format!("MAKES {m} POWER"),
            (_, u) => format!("USES {u} POWER"),
        };
        format!("{} - {} {power}. WEIGHS {}.", i.name, (i.about)(), i.mass * i.block)
    }
    /// Who makes power and who uses it, for the help and the power gauge: "HULL 1, CORE 4". A part
    /// that comes as a block (the cannon) says so, since its number is for the whole block.
    pub fn power_list(makers: bool) -> String {
        let of = |p: &Part| if makers { p.info().makes } else { p.info().uses };
        let block = |p: &Part| if p.info().block > 1 { " (3X3)" } else { "" };
        PARTS.iter().filter(|p| of(p) > 0).map(|p| format!("{}{} {}", p.name(), block(p), of(p))).collect::<Vec<_>>().join(", ")
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash)]
pub enum Order {
    Hunt,
    Guard,
    Escort,
    Harvest,
    Hold,
    /// Sabotage: goes for the other side's miners and factories.
    Sab,
}

/// The orders in the order the yard shows them, three to a row.
pub const ORDERS: [Order; 6] = [Order::Hunt, Order::Sab, Order::Guard, Order::Escort, Order::Harvest, Order::Hold];

impl Order {
    pub fn name(self) -> &'static str {
        match self {
            Order::Hunt => "HUNT",
            Order::Guard => "GUARD",
            Order::Escort => "ESCORT",
            Order::Harvest => "HARVEST",
            Order::Hold => "HOLD",
            Order::Sab => "SAB",
        }
    }
    /// Three letters, for a card's order tag in a match.
    pub fn tag(self) -> &'static str {
        match self {
            Order::Hunt => "HNT",
            Order::Guard => "GRD",
            Order::Escort => "ESC",
            Order::Harvest => "HRV",
            Order::Hold => "HLD",
            Order::Sab => "SAB",
        }
    }
    pub fn blurb(self) -> &'static str {
        match self {
            Order::Hunt => "HUNT - GOES AFTER THE NEAREST ENEMY, WHEREVER IT IS.",
            Order::Guard => "GUARD - IN A MATCH, WAITS IN THE RINGS ROUND WHAT BUILT IT AND FIGHTS WHAT COMES WITHIN 160 PX OF THERE. IN THE ARENA IT HUNTS.",
            Order::Escort => "ESCORT - IN A MATCH, FORMS UP ON YOUR KING, GOES WHERE IT FLIES AND FIGHTS WHAT COMES NEAR IT. IN THE ARENA IT HUNTS.",
            Order::Harvest => "HARVEST - MINES THE NEAREST ROCK AND HAULS THE PIXELS HOME. ITS GUNS STILL FIRE ON THE WAY.",
            Order::Hold => "HOLD - STAYS EXACTLY WHERE IT WAS PLACED, TURNING ITS LANCES TO FACE ANY ENEMY IN RANGE.",
            Order::Sab => "SAB - SABOTAGE: GOES AFTER THE NEAREST ENEMY MINER OR FACTORY, PAST WHATEVER GUARDS IT. WITH NONE LEFT IT HUNTS.",
        }
    }
    /// The orders a design can be switched between in a match: the four a warship takes, and
    /// first the one it was drawn with if that's another (a miner can be sent back to mining).
    pub fn in_match(drawn: Order) -> Vec<Order> {
        let mut v = vec![Order::Hunt, Order::Sab, Order::Guard, Order::Escort];
        if !v.contains(&drawn) {
            v.insert(0, drawn);
        }
        v
    }
    fn from_name(s: &str) -> Option<Order> {
        ORDERS.iter().copied().find(|o| o.name() == s)
    }
}

/// How a ship flies, which follows from its drawing alone.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash)]
pub enum Flight {
    /// At least 1 thrust per weight: dives through the target and loops back.
    Runs,
    /// Heavier: holds at weapon range, lined up so its lances bear.
    StandOff,
    /// No engines: stays put and can't turn.
    Still,
}

impl Flight {
    pub fn name(self) -> &'static str {
        match self {
            Flight::Runs => "ATTACK RUNS",
            Flight::StandOff => "STAND-OFF",
            Flight::Still => "STAYS PUT",
        }
    }
}

/// How a ship flies at what it's fighting. It's set on the design in the yard and fixed for a
/// match. Until it's set the drawing decides: enough thrust for the weight runs, less kites.
/// Either way the drawing decides how well it works.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash)]
pub enum Style {
    /// Dives through the target and loops back.
    Runs,
    /// Holds at weapon range, and backs away from whatever comes closer.
    Kite,
    /// Circles the target at weapon range, flat out.
    Orbit,
}

pub const STYLES: [Style; 3] = [Style::Runs, Style::Kite, Style::Orbit];

/// The sizes of group a design can leave in on HUNT or SAB: so many gather before any sets off.
pub const GROUPS: [u8; 4] = [1, 3, 6, 12];

impl Style {
    pub fn name(self) -> &'static str {
        match self {
            Style::Runs => "RUNS",
            Style::Kite => "KITE",
            Style::Orbit => "ORBIT",
        }
    }
    pub fn blurb(self) -> &'static str {
        match self {
            Style::Runs => "RUNS - DIVES STRAIGHT THROUGH ITS TARGET AND COMES ABOUT FOR ANOTHER PASS. A LANCE SHIP SLIDES ONTO THE TARGET'S ROW OR COLUMN FIRST.",
            Style::Kite => "KITE - HOLDS AT THE RANGE OF ITS GUNS AND BACKS AWAY FROM ANYTHING THAT COMES CLOSER. IT WORKS ON WHAT IT CAN OUTRUN OR OUTREACH.",
            Style::Orbit => "ORBIT - CIRCLES ITS TARGET AT THE RANGE OF ITS GUNS, FLAT OUT. FAST AND CLOSE, IT GETS ROUND QUICKER THAN A CANNON TURNS. A LANCE RARELY BEARS.",
        }
    }
    fn from_name(s: &str) -> Option<Style> {
        STYLES.iter().copied().find(|o| o.name() == s)
    }
    /// What a drawing flies as when nothing was chosen for it. None: it has no engines.
    pub fn of(f: Flight) -> Option<Style> {
        match f {
            Flight::Runs => Some(Style::Runs),
            Flight::StandOff => Some(Style::Kite),
            Flight::Still => None,
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash)]
pub struct Stats {
    pub count: i32,
    pub cores: i32,
    /// What the ship costs to build: its pixels plus its extra cores.
    pub cost: i32,
    pub mass: i32,
    pub engines: i32,
    pub flak: i32,
    pub lance: i32,
    pub cannon: i32,
    pub scoops: i32,
    pub factory: i32,
    /// Power made (hull and cores) and used (engines and guns).
    pub made: i32,
    pub used: i32,
    /// Thrust after any power shortage.
    pub thrust: i32,
    /// Top speed in quarter-pixels per tick.
    pub vmax: i32,
    /// Ticks per quarter turn (0 = can't turn).
    pub turn: i32,
    pub flight: Flight,
}

impl Stats {
    pub fn armed(&self) -> bool {
        self.flak + self.lance + self.cannon > 0
    }
    /// The biggest ship (in pixels) its factory pixels can build.
    pub fn max_build(&self) -> i32 {
        self.factory * FACTORY_REACH
    }
    /// How much of the power it needs the ship has, in percent (at most 100).
    pub fn power_pct(&self) -> i32 {
        if self.used <= self.made {
            100
        } else {
            self.made * 100 / self.used
        }
    }
    /// A gun's ticks between shots, stretched by any power shortage.
    pub fn reload(&self, period: i32) -> i32 {
        if self.used <= self.made {
            period
        } else {
            (period * self.used + self.made.max(1) - 1) / self.made.max(1)
        }
    }
    pub fn top_speed_px_s(&self) -> i32 {
        self.vmax * 15 / 4
    }
    /// Tenths of a second from standing still to top speed.
    pub fn zero_to_top_ds(&self) -> i32 {
        if self.thrust == 0 {
            0
        } else {
            self.vmax * self.mass * 10 / (self.thrust * 15)
        }
    }
    /// Tenths of a second per quarter turn.
    pub fn turn_ds(&self) -> i32 {
        self.turn * 10 / 15
    }
}

/// Stats from a set of cells. `cannons` is how many working cannons there are (a cannon is a
/// block, so the cells alone can't say).
pub fn stats_of(cells: &[Option<Part>], cannons: i32) -> Stats {
    let n = |p: Part| cells.iter().filter(|c| **c == Some(p)).count() as i32;
    let count = cells.iter().flatten().count() as i32;
    let mass: i32 = cells.iter().flatten().map(|p| p.mass()).sum();
    let engines = n(Part::Engine);
    let (flak, lance, cores, factory) = (n(Part::Flak), n(Part::Lance), n(Part::Core), n(Part::Factory));
    // Power (round 5): what each pixel makes and uses comes from the part table. A cannon's draw
    // is per working cannon, not per pixel of its block.
    let made: i32 = PARTS.iter().map(|&p| n(p) * p.info().makes).sum();
    let used: i32 = PARTS.iter().map(|&p| if p == Part::Cannon { cannons } else { n(p) } * p.info().uses).sum();
    // Engines push in proportion to the power they get.
    let full = engines * 8;
    let thrust = if used > made { full * made / used } else { full };
    let vmax = if thrust == 0 { 0 } else { (thrust * 5 / mass.max(1) + 2).clamp(3, 12) };
    let turn = if thrust == 0 { 0 } else { (mass * 2 / thrust).clamp(1, 24) };
    let flight = if thrust == 0 {
        Flight::Still
    } else if thrust >= mass {
        Flight::Runs
    } else {
        Flight::StandOff
    };
    let cost = count + core_cost(cores);
    Stats { count, cores, cost, mass, engines, flak, lance, cannon: cannons, scoops: n(Part::Scoop), factory, made, used, thrust, vmax, turn, flight }
}

/// Which of a w x h grid's cells connect to `start` edge to edge, through cells where `on` holds.
pub fn reach(w: i32, h: i32, start: usize, on: impl Fn(usize) -> bool) -> Vec<bool> {
    let mut seen = vec![false; (w * h) as usize];
    if !on(start) {
        return seen;
    }
    seen[start] = true;
    let mut todo = vec![start];
    while let Some(i) = todo.pop() {
        let (x, y) = (i as i32 % w, i as i32 / w);
        for (dx, dy) in [(1, 0), (-1, 0), (0, 1), (0, -1)] {
            let (nx, ny) = (x + dx, y + dy);
            if nx < 0 || ny < 0 || nx >= w || ny >= h {
                continue;
            }
            let j = (ny * w + nx) as usize;
            if !seen[j] && on(j) {
                seen[j] = true;
                todo.push(j);
            }
        }
    }
    seen
}

/// A design trimmed to its bounding box, facing north (up is forward).
#[derive(Clone, Debug, Hash)]
pub struct Shape {
    pub w: i32,
    pub h: i32,
    pub cells: Vec<Option<Part>>,
    /// The centre cell of each cannon block.
    pub cannons: Vec<usize>,
}

impl Shape {
    pub fn dims(&self, facing: u8) -> (i32, i32) {
        if facing & 1 == 1 {
            (self.h, self.w)
        } else {
            (self.w, self.h)
        }
    }
    /// Where cell `i` lands when the shape is centred on (x, y) with a facing (exact quarter turns).
    pub fn cell_xy(&self, x: i32, y: i32, facing: u8, i: usize) -> (i32, i32) {
        let (dx, dy) = (i as i32 % self.w, i as i32 / self.w);
        let (w, h, rx, ry) = match facing & 3 {
            0 => (self.w, self.h, dx, dy),
            1 => (self.h, self.w, self.h - 1 - dy, dx),
            2 => (self.w, self.h, self.w - 1 - dx, self.h - 1 - dy),
            _ => (self.h, self.w, dy, self.w - 1 - dx),
        };
        (x - w / 2 + rx, y - h / 2 + ry)
    }
    pub fn cell_at(&self, x: i32, y: i32, facing: u8, px: i32, py: i32) -> Option<usize> {
        let (dx, dy) = self.to_local(x, y, facing, px, py);
        self.local_cell(dx, dy)
    }
    /// A point in the arena in the shape's own grid (north up), for the shape centred on (x, y)
    /// with a facing. Points outside the shape map too, just outside the grid.
    pub fn to_local(&self, x: i32, y: i32, facing: u8, px: i32, py: i32) -> (i32, i32) {
        let (w, h) = self.dims(facing);
        let (rx, ry) = (px - (x - w / 2), py - (y - h / 2));
        match facing & 3 {
            0 => (rx, ry),
            1 => (ry, self.h - 1 - rx),
            2 => (self.w - 1 - rx, self.h - 1 - ry),
            _ => (self.w - 1 - ry, rx),
        }
    }
    /// The cell at a point of the shape's own grid, if there's a part there.
    pub fn local_cell(&self, dx: i32, dy: i32) -> Option<usize> {
        if dx < 0 || dy < 0 || dx >= self.w || dy >= self.h {
            return None;
        }
        let i = (dy * self.w + dx) as usize;
        self.cells[i].map(|_| i)
    }
    pub fn is_core(&self, i: usize) -> bool {
        self.cells[i] == Some(Part::Core)
    }
}

/// What's wrong with a design, if anything.
#[derive(Clone, Debug, Default)]
pub struct Check {
    pub cores: i32,
    /// Cells not joined to the cores: the whole drawing must be one piece.
    pub loose: Vec<(i32, i32)>,
    /// Engines with no straight 3-wide way out of the ship for their exhaust.
    pub buried: Vec<(i32, i32)>,
    /// A seeded design whose seed cell (the grid's centre) isn't a core.
    pub no_seed: bool,
}

impl Check {
    pub fn ok(&self) -> bool {
        self.cores >= 1 && self.loose.is_empty() && self.buried.is_empty() && !self.no_seed
    }
    pub fn problem(&self) -> Option<String> {
        let s = |n: usize| if n == 1 { "" } else { "S" };
        if self.cores == 0 {
            Some("NO CORE: IT CAN'T FLY".into())
        } else if self.no_seed {
            Some("NO CORE ON THE SEED".into())
        } else if !self.loose.is_empty() {
            let n = self.loose.len();
            Some(format!("{n} LOOSE PIXEL{}", s(n)))
        } else if !self.buried.is_empty() {
            let n = self.buried.len();
            Some(format!("{n} ENGINE{}: NO 3-WIDE EXIT", s(n)))
        } else {
            None
        }
    }
}

/// Engines whose exhaust has no straight way out. Like a rook's move: in one of the four
/// directions, the engine's own column (or row) and the ones either side of it must be empty all
/// the way out past the edge of the ship's bounding box. So there's always a straight line of
/// fire to every engine; plate can't fold round it. `filled` says whether a cell holds a pixel
/// (anything outside the drawing is empty).
pub fn buried_engines(engines: &[(i32, i32)], bbox: (i32, i32, i32, i32), filled: impl Fn(i32, i32) -> bool) -> Vec<(i32, i32)> {
    let (x0, y0, x1, y1) = bbox;
    let clear = |(ex, ey): (i32, i32), (dx, dy): (i32, i32)| {
        // Sideways from the line of the exhaust.
        let (sx, sy) = (dy.abs(), dx.abs());
        (1..)
            .map(|k| (ex + dx * k, ey + dy * k))
            .take_while(|&(x, y)| (x0..=x1).contains(&x) && (y0..=y1).contains(&y))
            .all(|(x, y)| (-1..=1).all(|s| !filled(x + sx * s, y + sy * s)))
    };
    engines.iter().copied().filter(|&e| ![(0, 1), (0, -1), (1, 0), (-1, 0)].iter().any(|&d| clear(e, d))).collect()
}

#[derive(Clone, Debug, Hash)]
pub struct Design {
    pub name: String,
    pub order: Order,
    /// GRID x GRID, row-major, north up.
    pub cells: Vec<Option<Part>>,
    /// Grown from a seed on a rock (match only), rather than printed by a factory or the king.
    /// Its seed is a core at the grid's centre, which the yard keeps there.
    pub seeded: bool,
    /// What its factory pixels print, by deck slot, round and round (match only).
    pub build: Vec<usize>,
    /// How it flies at what it's fighting, once chosen in the yard. None: as its drawing suggests.
    pub style: Option<Style>,
    /// How many of it gather before they leave on HUNT or SAB (match only): one of `GROUPS`.
    pub group: u8,
}

/// Where a seeded design's seed core sits on the grid.
pub const SEED_CELL: (i32, i32) = (GRID / 2, GRID / 2);
/// The longest build order the yard allows.
pub const BUILD_MAX: usize = 12;

impl Design {
    pub fn blank(name: &str) -> Design {
        Design { name: name.into(), order: Order::Hunt, cells: vec![None; (GRID * GRID) as usize], seeded: false, build: Vec::new(), style: None, group: 1 }
    }

    /// A design from art rows, centred in the grid.
    pub fn from_rows(name: &str, order: Order, rows: &[&str]) -> Design {
        let mut d = Design::blank(name);
        d.order = order;
        let h = rows.len() as i32;
        let w = rows.iter().map(|r| r.len()).max().unwrap_or(0) as i32;
        let (ox, oy) = ((GRID - w) / 2, (GRID - h) / 2);
        for (y, r) in rows.iter().enumerate() {
            for (x, c) in r.chars().enumerate() {
                d.set(ox + x as i32, oy + y as i32, Part::from_ch(c));
            }
        }
        d
    }

    pub fn get(&self, x: i32, y: i32) -> Option<Part> {
        if (0..GRID).contains(&x) && (0..GRID).contains(&y) {
            self.cells[(y * GRID + x) as usize]
        } else {
            None
        }
    }

    /// Sets one cell with no regard for cannon blocks; the editor uses `put`.
    pub fn set(&mut self, x: i32, y: i32, p: Option<Part>) {
        if (0..GRID).contains(&x) && (0..GRID).contains(&y) {
            self.cells[(y * GRID + x) as usize] = p;
        }
    }

    /// The top-left corner of every cannon block, and any cannon cells that don't make a whole
    /// block. Blocks never overlap (the editor sees to that), so the first unclaimed cannon cell
    /// in reading order is always some block's top-left corner.
    pub fn cannon_blocks(&self) -> (Vec<(i32, i32)>, Vec<(i32, i32)>) {
        let mut claimed = vec![false; self.cells.len()];
        let (mut blocks, mut stray) = (Vec::new(), Vec::new());
        for y in 0..GRID {
            for x in 0..GRID {
                let i = (y * GRID + x) as usize;
                if claimed[i] || self.cells[i] != Some(Part::Cannon) {
                    continue;
                }
                let whole = (0..3).all(|dy| (0..3).all(|dx| self.get(x + dx, y + dy) == Some(Part::Cannon) && !claimed[((y + dy) * GRID + x + dx) as usize]));
                if whole {
                    for dy in 0..3 {
                        for dx in 0..3 {
                            claimed[((y + dy) * GRID + x + dx) as usize] = true;
                        }
                    }
                    blocks.push((x, y));
                } else {
                    claimed[i] = true;
                    stray.push((x, y));
                }
            }
        }
        (blocks, stray)
    }

    /// The cannon block covering a cell, by its top-left corner.
    pub fn cannon_at(&self, x: i32, y: i32) -> Option<(i32, i32)> {
        self.cannon_blocks().0.into_iter().find(|&(bx, by)| (bx..bx + 3).contains(&x) && (by..by + 3).contains(&y))
    }

    fn clear_cannon(&mut self, (bx, by): (i32, i32)) {
        for dy in 0..3 {
            for dx in 0..3 {
                self.set(bx + dx, by + dy, None);
            }
        }
    }

    /// The editor's paint: a cannon cell is only ever removed with its whole block, and a cannon
    /// stamps a 3x3 block centred on the cell (refused if it doesn't fit). Returns whether
    /// anything changed.
    pub fn put(&mut self, x: i32, y: i32, p: Option<Part>) -> bool {
        if !(0..GRID).contains(&x) || !(0..GRID).contains(&y) {
            return false;
        }
        if p == Some(Part::Cannon) {
            if x < 1 || y < 1 || x > GRID - 2 || y > GRID - 2 {
                return false;
            }
            for (bx, by) in self.cannon_blocks().0 {
                if (bx - (x - 1)).abs() < 3 && (by - (y - 1)).abs() < 3 {
                    self.clear_cannon((bx, by));
                }
            }
            for dy in -1..=1 {
                for dx in -1..=1 {
                    self.set(x + dx, y + dy, Some(Part::Cannon));
                }
            }
            return true;
        }
        if self.get(x, y) == p {
            return false;
        }
        if self.get(x, y) == Some(Part::Cannon) {
            match self.cannon_at(x, y) {
                Some(b) => self.clear_cannon(b),
                None => self.set(x, y, None),
            }
        }
        self.set(x, y, p);
        true
    }

    /// A design is legal with at least one core, every pixel joined into one piece edge to edge
    /// (so all its cores are joined too), whole cannon blocks, and no buried engines.
    pub fn check(&self) -> Check {
        let cores: Vec<usize> = (0..self.cells.len()).filter(|&i| self.cells[i] == Some(Part::Core)).collect();
        let mut c = Check { cores: cores.len() as i32, no_seed: self.seeded && self.get(SEED_CELL.0, SEED_CELL.1) != Some(Part::Core), ..Default::default() };
        // The ship is the biggest piece with a core in it; anything else is loose.
        let ship = cores.iter().map(|&k| reach(GRID, GRID, k, |i| self.cells[i].is_some())).max_by_key(|seen| seen.iter().filter(|&&b| b).count());
        if let Some(seen) = ship {
            c.loose = (0..self.cells.len()).filter(|&i| self.cells[i].is_some() && !seen[i]).map(|i| (i as i32 % GRID, i as i32 / GRID)).collect();
        }
        c.loose.extend(self.cannon_blocks().1);
        if let Some(bbox) = self.used_box() {
            let (x0, y0, x1, y1) = bbox;
            let engines: Vec<(i32, i32)> = (y0..=y1).flat_map(|y| (x0..=x1).map(move |x| (x, y))).filter(|&(x, y)| self.get(x, y) == Some(Part::Engine)).collect();
            c.buried = buried_engines(&engines, bbox, |x, y| self.get(x, y).is_some());
        }
        c
    }

    pub fn stats(&self) -> Stats {
        stats_of(&self.cells, self.cannon_blocks().0.len() as i32)
    }

    /// How it flies at what it's fighting: what was chosen for it, or else what its drawing
    /// suggests. None: it has no engines, and stays put.
    pub fn flies(&self) -> Option<Style> {
        let st = self.stats();
        if st.thrust == 0 {
            None
        } else {
            self.style.or(Style::of(st.flight))
        }
    }

    /// The box round everything drawn, as (left, top, right, bottom), inclusive.
    pub fn used_box(&self) -> Option<(i32, i32, i32, i32)> {
        let mut b: Option<(i32, i32, i32, i32)> = None;
        for i in 0..self.cells.len() {
            if self.cells[i].is_some() {
                let (x, y) = (i as i32 % GRID, i as i32 / GRID);
                b = Some(match b {
                    None => (x, y, x, y),
                    Some((l, t, r, bt)) => (l.min(x), t.min(y), r.max(x), bt.max(y)),
                });
            }
        }
        b
    }

    pub fn shape(&self) -> Shape {
        let Some((x0, y0, x1, y1)) = self.used_box() else {
            return Shape { w: 1, h: 1, cells: vec![None], cannons: Vec::new() };
        };
        let (w, h) = (x1 - x0 + 1, y1 - y0 + 1);
        let cells = (0..w * h).map(|i| self.get(x0 + i % w, y0 + i / w)).collect();
        let cannons = self.cannon_blocks().0.iter().map(|&(bx, by)| ((by + 1 - y0) * w + bx + 1 - x0) as usize).collect();
        Shape { w, h, cells, cannons }
    }

    /// The whole drawing as a stamp.
    pub fn as_stamp(&self) -> Stamp {
        let sh = self.shape();
        Stamp { w: sh.w, h: sh.h, cells: sh.cells }
    }

    /// The cells in a box (corners inclusive, either order) as a stamp, trimmed to what's drawn.
    /// A cannon block the box cuts through is left out, so a stamp never holds part of one.
    pub fn copy_box(&self, a: (i32, i32), b: (i32, i32)) -> Stamp {
        let (x0, x1, y0, y1) = (a.0.min(b.0), a.0.max(b.0), a.1.min(b.1), a.1.max(b.1));
        let inside = |x: i32, y: i32| (x0..=x1).contains(&x) && (y0..=y1).contains(&y);
        let blocks = self.cannon_blocks().0;
        let keep = |x: i32, y: i32| match self.get(x, y) {
            Some(Part::Cannon) => blocks.iter().any(|&(bx, by)| (bx..bx + 3).contains(&x) && (by..by + 3).contains(&y) && inside(bx, by) && inside(bx + 2, by + 2)),
            p => p.is_some(),
        };
        let used: Vec<(i32, i32)> = (y0..=y1).flat_map(|y| (x0..=x1).map(move |x| (x, y))).filter(|&(x, y)| keep(x, y)).collect();
        if used.is_empty() {
            return Stamp { w: 1, h: 1, cells: vec![None] };
        }
        let (l, r) = (used.iter().map(|p| p.0).min().unwrap(), used.iter().map(|p| p.0).max().unwrap());
        let (t, bt) = (used.iter().map(|p| p.1).min().unwrap(), used.iter().map(|p| p.1).max().unwrap());
        let (w, h) = (r - l + 1, bt - t + 1);
        let cells = (0..w * h)
            .map(|i| {
                let (x, y) = (l + i % w, t + i / w);
                if keep(x, y) {
                    self.get(x, y)
                } else {
                    None
                }
            })
            .collect();
        Stamp { w, h, cells }
    }

    /// Writes a stamp's pixels with its top-left at (x, y); its empty cells leave what's there,
    /// and anything off the grid is dropped. A cannon block the stamp lands on is removed whole
    /// first, so no scraps are left. Returns whether anything changed.
    pub fn paste(&mut self, st: &Stamp, (x, y): (i32, i32)) -> bool {
        let mut changed = false;
        for (bx, by) in self.cannon_blocks().0 {
            if (0..9).any(|k| st.get(bx + k % 3 - x, by + k / 3 - y).is_some()) {
                self.clear_cannon((bx, by));
                changed = true;
            }
        }
        for sy in 0..st.h {
            for sx in 0..st.w {
                let (gx, gy) = (x + sx, y + sy);
                if let Some(p) = st.get(sx, sy) {
                    if (0..GRID).contains(&gx) && (0..GRID).contains(&gy) && self.get(gx, gy) != Some(p) {
                        self.set(gx, gy, Some(p));
                        changed = true;
                    }
                }
            }
        }
        changed
    }

    /// Fills a box (corners inclusive, either order) with a part, or empties it. Cannons tile the
    /// box in whole 3x3 blocks from its top-left corner, leaving any strip too thin for one; a
    /// cannon block the box touches is removed whole. Returns whether anything changed.
    pub fn fill_box(&mut self, a: (i32, i32), b: (i32, i32), p: Option<Part>) -> bool {
        let (x0, x1) = (a.0.min(b.0).max(0), a.0.max(b.0).min(GRID - 1));
        let (y0, y1) = (a.1.min(b.1).max(0), a.1.max(b.1).min(GRID - 1));
        if x0 > x1 || y0 > y1 {
            return false;
        }
        let (mut w, mut h) = (x1 - x0 + 1, y1 - y0 + 1);
        if p == Some(Part::Cannon) {
            (w, h) = (w / 3 * 3, h / 3 * 3);
            if w == 0 || h == 0 {
                return false;
            }
        }
        match p {
            Some(_) => self.paste(&Stamp { w, h, cells: vec![p; (w * h) as usize] }, (x0, y0)),
            None => {
                let mut changed = false;
                for (bx, by) in self.cannon_blocks().0 {
                    if bx + 2 >= x0 && bx <= x1 && by + 2 >= y0 && by <= y1 {
                        self.clear_cannon((bx, by));
                        changed = true;
                    }
                }
                for y in y0..=y1 {
                    for x in x0..=x1 {
                        changed |= self.get(x, y).is_some();
                        self.set(x, y, None);
                    }
                }
                changed
            }
        }
    }

    /// Flood-fills the patch joined edge to edge that matches the cell at (x, y): an empty hole,
    /// or a patch of one part, which this swaps for another part or erases. An empty area that
    /// reaches the edge of the grid isn't a hole, so it's refused, and so are cannons (they come
    /// in 3x3 blocks; the rectangle tiles them). Returns whether anything changed.
    pub fn flood(&mut self, (x, y): (i32, i32), p: Option<Part>) -> Result<bool, &'static str> {
        if !(0..GRID).contains(&x) || !(0..GRID).contains(&y) {
            return Ok(false);
        }
        if p == Some(Part::Cannon) {
            return Err("CANNONS DON'T FILL: USE THE RECTANGLE");
        }
        let was = self.get(x, y);
        if was == p {
            return Ok(false);
        }
        let seen = reach(GRID, GRID, (y * GRID + x) as usize, |i| self.cells[i] == was);
        let patch: Vec<usize> = (0..seen.len()).filter(|&i| seen[i]).collect();
        if was.is_none() && patch.iter().any(|&i| (i as i32 % GRID) % (GRID - 1) == 0 || (i as i32 / GRID) % (GRID - 1) == 0) {
            return Err("THAT SPACE ISN'T CLOSED IN: FILL A HOLE INSIDE THE SHIP");
        }
        for i in patch {
            self.cells[i] = p;
        }
        Ok(true)
    }

    /// One line for the save file: `NAME|ORDER|X,Y|row/row/...|KIND|BUILD|STYLE|GROUP`. The
    /// drawing is trimmed to its box with the box's top-left on the grid and `.` for empty cells;
    /// KIND is BUILT or SEEDED, and BUILD the build order as deck slots (round 5 lines stop after
    /// the rows). STYLE and GROUP are left off a design that has neither chosen, so such a line is
    /// what it always was.
    pub fn to_line(&self) -> String {
        let kind = if self.seeded { "SEEDED" } else { "BUILT" };
        let build: Vec<String> = self.build.iter().map(|b| b.to_string()).collect();
        let flying = if self.style.is_none() && self.group <= 1 { String::new() } else { format!("|{}|{}", self.style.map_or("-", |s| s.name()), self.group) };
        let Some((x0, y0, x1, y1)) = self.used_box() else {
            return format!("{}|{}|0,0|.|{kind}|{}{flying}", self.name, self.order.name(), build.join(","));
        };
        let rows: Vec<String> = (y0..=y1).map(|y| (x0..=x1).map(|x| self.get(x, y).map_or('.', |p| p.ch())).collect()).collect();
        format!("{}|{}|{x0},{y0}|{}|{kind}|{}{flying}", self.name, self.order.name(), rows.join("/"), build.join(","))
    }

    /// Reads a save line. A round 2 or 3 line (the whole 41x41 grid, no position) is centred.
    pub fn from_line(s: &str) -> Option<Design> {
        let f: Vec<&str> = s.trim().split('|').collect();
        let (name, order) = (*f.first()?, Order::from_name(f.get(1)?)?);
        let (at, rows) = match f.len() {
            4 | 6 | 8 => {
                let (x, y) = f[2].split_once(',')?;
                ((x.parse::<i32>().ok()?, y.parse::<i32>().ok()?), f[3])
            }
            3 => {
                let n = f[2].split('/').count() as i32;
                if n > GRID {
                    return None;
                }
                (((GRID - n) / 2, (GRID - n) / 2), f[2])
            }
            _ => return None,
        };
        let mut d = Design::blank(name);
        d.order = order;
        for (y, r) in rows.split('/').enumerate() {
            for (x, c) in r.chars().enumerate() {
                d.set(at.0 + x as i32, at.1 + y as i32, Part::from_ch(c));
            }
        }
        if f.len() >= 6 {
            d.seeded = f[4] == "SEEDED";
            d.build = f[5].split(',').filter_map(|b| b.trim().parse().ok()).take(BUILD_MAX).collect();
        }
        if f.len() == 8 {
            d.style = Style::from_name(f[6]);
            d.group = f[7].parse().ok().filter(|g| GROUPS.contains(g)).unwrap_or(1);
        }
        Some(d)
    }

    /// The shape's cells in the order they grow: outward from `start` edge to edge, so what's
    /// grown is one connected piece at every step. Ties go in reading order.
    pub fn growth_order(sh: &Shape, start: usize) -> Vec<usize> {
        let (w, h) = (sh.w, sh.h);
        let mut seen = vec![false; sh.cells.len()];
        let mut order = Vec::new();
        let mut queue = std::collections::VecDeque::new();
        if sh.cells.get(start).copied().flatten().is_some() {
            seen[start] = true;
            queue.push_back(start);
        }
        while let Some(c) = queue.pop_front() {
            order.push(c);
            let (x, y) = (c as i32 % w, c as i32 / w);
            for (dx, dy) in [(0, -1), (-1, 0), (1, 0), (0, 1)] {
                let (nx, ny) = (x + dx, y + dy);
                if nx >= 0 && ny >= 0 && nx < w && ny < h {
                    let k = (ny * w + nx) as usize;
                    if !seen[k] && sh.cells[k].is_some() {
                        seen[k] = true;
                        queue.push_back(k);
                    }
                }
            }
        }
        order
    }

    /// The shape cell where a seeded design's seed core sits, or the first core of any other.
    pub fn start_cell(&self) -> Option<usize> {
        let (x0, y0, _, _) = self.used_box()?;
        let sh = self.shape();
        if self.seeded {
            let c = ((SEED_CELL.1 - y0) * sh.w + SEED_CELL.0 - x0) as usize;
            return sh.is_core(c).then_some(c);
        }
        (0..sh.cells.len()).find(|&c| sh.is_core(c))
    }
}

/// A piece of a drawing picked up to place elsewhere: a whole design, or a box copied off the
/// grid. Placing it writes ordinary pixels; nothing stays linked to where it came from.
#[derive(Clone, Debug, PartialEq)]
pub struct Stamp {
    pub w: i32,
    pub h: i32,
    pub cells: Vec<Option<Part>>,
}

impl Stamp {
    pub fn get(&self, x: i32, y: i32) -> Option<Part> {
        if (0..self.w).contains(&x) && (0..self.h).contains(&y) {
            self.cells[(y * self.w + x) as usize]
        } else {
            None
        }
    }
    /// A quarter turn clockwise.
    pub fn turned(&self) -> Stamp {
        let (w, h) = (self.h, self.w);
        let cells = (0..w * h).map(|i| self.get(i / w, self.h - 1 - i % w)).collect();
        Stamp { w, h, cells }
    }
    /// Mirrored left to right.
    pub fn flipped(&self) -> Stamp {
        let cells = (0..self.w * self.h).map(|i| self.get(self.w - 1 - i % self.w, i / self.w)).collect();
        Stamp { w: self.w, h: self.h, cells }
    }
    pub fn is_empty(&self) -> bool {
        self.cells.iter().all(|c| c.is_none())
    }
    /// Where the top-left goes for the stamp to sit centred on a cell.
    pub fn origin(&self, (cx, cy): (i32, i32)) -> (i32, i32) {
        (cx - self.w / 2, cy - self.h / 2)
    }
}

/// The starting library. Slots are referred to by index in the test setups.
pub fn defaults() -> Vec<Design> {
    vec![
        Design::from_rows("FIGHTER", Order::Hunt, &["g.g", "hch", "e.e"]),
        // A hull pixel ahead of the lance soaks the first hit (own shots pass through own pixels).
        Design::from_rows("LANCER", Order::Hunt, &[".h.", ".l.", "hch", "e.e"]),
        // Flak and lance tucked inside plate (a ship's own shots pass through its own pixels);
        // only the engines are bare.
        Design::from_rows("FRIGATE", Order::Hunt, &["...ppp...", "..pplpp..", ".pphhhpp.", ".pghchgp.", ".pphhhpp.", ".pghhhgp.", "..ppppp..", "..e.e.e.."]),
        // One cannon behind a plated prow; its flak sponsons are bolted on outside the plate.
        Design::from_rows(
            "DESTROYER",
            Order::Hunt,
            &[
                ".....ppp.....",
                "....pphpp....",
                "...pphhhpp...",
                "..pphhhhhpp..",
                ".pphhnnnhhpp.",
                ".phhhnnnhhhp.",
                ".phhhnnnhhhp.",
                ".phhhhhhhhhp.",
                ".phhhhhhhhhp.",
                "gphhhhhhhhhpg",
                ".phhhhhhhhhp.",
                ".phhhhchhhhp.",
                ".phhhhhhhhhp.",
                "gphhhhhhhhhpg",
                ".phhhhhhhhhp.",
                "..phhhhhhhp..",
                "...ppppppp...",
                "..eeee.eeee..",
            ],
        ),
        // Sealed in plate all round: fighters can only strip its sponsons and engines. Three cores
        // (fore, middle, aft) for +24: cheap insurance on a ship this size.
        Design::from_rows(
            "CAPITAL",
            Order::Hunt,
            &[
                ".........ppp.........",
                "........pphpp........",
                ".......pphhhpp.......",
                "......pphhhhhpp......",
                ".....pphhhhhhhpp.....",
                "....pphhhhhhhhhpp....",
                "...pphhhhnnnhhhhpp...",
                "..pphhhhhnnnhhhhhpp..",
                ".pphhhhhhnnnhhhhhhpp.",
                ".phhhhhhhhhhhhhhhhhp.",
                ".phhhhhhhhchhhhhhhhp.",
                "gphhhhhhghhhghhhhhhpg",
                ".phhhhhhhhhhhhhhhhhp.",
                ".phhhhhhhhhhhhhhhhhp.",
                ".phhhnnnhhhhhnnnhhhp.",
                ".phhhnnnhhhhhnnnhhhp.",
                ".phhhnnnhhhhhnnnhhhp.",
                ".phhhhhhhhhhhhhhhhhp.",
                ".phhhhhhhhchhhhhhhhp.",
                "gphhhhhhhhhhhhhhhhhpg",
                ".phhhhhhhhhhhhhhhhhp.",
                ".phhhhhhhnnnhhhhhhhp.",
                ".phhhhhhhnnnhhhhhhhp.",
                ".phhhhhhhnnnhhhhhhhp.",
                ".phhhhhhhhhhhhhhhhhp.",
                ".phhhhhhhhhhhhhhhhhp.",
                ".phhhhhhhhchhhhhhhhp.",
                "gphhhhhhghhhghhhhhhpg",
                ".phhhhhhhhhhhhhhhhhp.",
                ".phhhhhhhhhhhhhhhhhp.",
                ".phhhhhhhhhhhhhhhhhp.",
                ".phhhhhhghhhghhhhhhp.",
                ".phhhhhhhhhhhhhhhhhp.",
                ".phhhhhhhhhhhhhhhhhp.",
                "..phhhhhhhhhhhhhhhp..",
                "...ppppppppppppppp...",
                "...e.e.e.e.e.e.e.e...",
            ],
        ),
        Design::from_rows("GUNMINER", Order::Harvest, &["s.s", "sgs", "hch", ".e."]),
        titan(),
    ]
}

/// A 97-pixel battleship, the round 4 scale test, drawn in code: plated two pixels deep, nine
/// cannons, flak on sponsons down both sides and more behind the plate, three lances in the prow,
/// three cores down the spine (+24) and a row of engines across the stern.
fn titan() -> Design {
    let mut d = Design::blank("TITAN");
    let c = GRID / 2;
    let half = |y: i32| if y < 40 { y / 2 } else if y <= 88 { 20 } else { 20 - (y - 88) };
    for y in 0..=95 {
        for x in c - half(y)..=c + half(y) {
            d.set(x, y, Some(Part::Hull));
        }
    }
    // Two pixels of plate all round: hull touching anything that isn't hull becomes plate, twice.
    for _ in 0..2 {
        let skin: Vec<(i32, i32)> = (0..GRID * GRID)
            .map(|i| (i % GRID, i / GRID))
            .filter(|&(x, y)| d.get(x, y) == Some(Part::Hull) && [(1, 0), (-1, 0), (0, 1), (0, -1)].iter().any(|&(dx, dy)| d.get(x + dx, y + dy) != Some(Part::Hull)))
            .collect();
        for (x, y) in skin {
            d.set(x, y, Some(Part::Plate));
        }
    }
    for y in [30, 55, 80] {
        d.set(c, y, Some(Part::Core));
    }
    for (x, y) in [(c, 18), (c, 42), (c, 67), (c - 12, 48), (c + 12, 48), (c - 12, 62), (c + 12, 62), (c - 12, 76), (c + 12, 76)] {
        d.put(x, y, Some(Part::Cannon));
    }
    for (x, y) in [(c, 8), (c - 4, 24), (c + 4, 24)] {
        d.set(x, y, Some(Part::Lance));
    }
    for s in [-1, 1] {
        // Sponsons just outside the plate, and guns behind it (a ship's own shots pass through it).
        for (x, y) in [(half(20) + 1, 20), (half(32) + 1, 32)].into_iter().chain((44..=84).step_by(8).map(|y| (21, y))).chain((44..=88).step_by(6).map(|y| (16, y))) {
            d.set(c + s * x, y, Some(Part::Flak));
        }
    }
    for x in (c - 12..=c + 12).step_by(2) {
        d.set(x, 96, Some(Part::Engine));
    }
    d
}

/// The king (match only): a fixed design for now. It flies like Toy 1's (straight from the keys,
/// no inertia), prints anything in the deck, and is the root node of its network.
pub fn king() -> Design {
    Design::from_rows(
        "KING",
        Order::Hold,
        &[
            "......ppppp......",
            "....pphhhhhpp....",
            "...phhhhghhhhp...",
            "..phhhhhhhhhhhp..",
            ".phhhhhhhhhhhhhp.",
            ".phhhchhhhhchhhp.",
            "gphhhhhhhhhhhhhpg",
            "phhhhhhhhhhhhhhhp",
            "phhhhhhhchhhhhhhp",
            "phhhhhhhhhhhhhhhp",
            "gphhhhhhhhhhhhhpg",
            ".phhhhhhhhhhhhhp.",
            ".phhhhhhhhhhhhhp.",
            "..phhhhhhhhhhhp..",
            "...ppphhhhhppp...",
            ".....hhhhhhh.....",
            ".....e.e.e.e.....",
        ],
    )
}

/// A seed (match only): a core and an engine, unarmed. The king prints one for a seeded design;
/// it flies to a rock and grows the design out of it.
pub fn seed() -> Design {
    Design::from_rows("SEED", Order::Hunt, &["c", "e"])
}

/// A design drawn at an exact place in the grid, as the yard saves it. `build` makes it a seeded
/// factory with that build order.
fn drawn(name: &str, order: Order, at: (i32, i32), rows: &[&str], build: Option<&[usize]>) -> Design {
    let mut d = Design::blank(name);
    d.order = order;
    for (y, r) in rows.iter().enumerate() {
        for (x, c) in r.chars().enumerate() {
            d.set(at.0 + x as i32, at.1 + y as i32, Part::from_ch(c));
        }
    }
    if let Some(b) = build {
        d.seeded = true;
        d.build = b.to_vec();
    }
    d
}

/// The deck the AI plays, and the one you can borrow before you've drawn factories of your own.
/// Slots: 0 fighter, 1 lancer, 2 frigate, 3 miner, 4 outpost, 5 shipyard, 6 capital.
///
/// Henry's drawings (2026-11-10): the round 2 cards with every pixel of power put to use. None
/// carries hull that feeds nothing, so what a match shows is the rules and not a wasteful
/// drawing. The warships are drawn on GUARD: in a match they gather where they're built until
/// you send them, instead of trickling out one at a time.
pub fn starter_deck() -> Vec<Design> {
    vec![
        // Four flak round the nose: every pixel of power its hull makes is spent.
        drawn("FIGHTER", Order::Guard, (48, 47), &[".g.", "ggg", "hch", "e.e"], None),
        // Two lances in a column: the rear one fires through the front one. Hull either side
        // of it since 2026-11-11, when a lance went from 2 power to 3.
        drawn("LANCER", Order::Guard, (48, 47), &[".l.", "hlh", "hch", "e.e"], None),
        // Six flak and four lances down the spine, behind plate, with two more engines. Four
        // of its plates are hull now, to feed the lances.
        drawn(
            "FRIGATE",
            Order::Guard,
            (45, 44),
            &[
                "....p....",
                "...php...",
                "..pplpp..",
                "phghlhghp",
                "phghchghp",
                "phhhlhhhp",
                "phghlhghp",
                "e.ppppp.e",
                "..e.e.e..",
            ],
            None,
        ),
        // Three scoops, a flak turret on the nose and three engines round the core: as fast as
        // a ship can fly, with all four of its core's power spent and no hull at all. A miner
        // bites a pixel every 2.7 s however many scoops it has past two, so the cheapest fast
        // one earns the most. Of eighteen drawings played against each other (MATCH.md, M2.14)
        // this came first; Henry's own first miner (below, with the retired cards) cost 14 px
        // and flew at a third of the speed.
        //
        // 2026-11-11: scoops use power and weigh 2, so it carries two hull pixels where it had
        // an engine, costs 9 px and flies at 30 px/s. Nobody has played drawings against each
        // other for it under these costs.
        drawn("MINER", Order::Harvest, (48, 47), &[".g.", "sss", "hch", "e.e"], None),
        // Twenty-eight factory pixels inside the plate, so it builds anything up to 280 px.
        // (Forty-two until a factory pixel went from 1 power to 2: the same size and price,
        // with a third of its factory turned to the hull the rest needs.)
        drawn(
            "OUTPOST",
            Order::Hold,
            (43, 43),
            &[
                ".....ppp.....",
                "...pphhhpp...",
                "..phhhghhhp..",
                ".phhhhhhhhhp.",
                ".phhfffffhhp.",
                "phfffhhhfffhp",
                "pgfffhchfffgp",
                "phfffhhhfffhp",
                ".phhfffffhhp.",
                ".phhhhhhhhhp.",
                "..phhhghhhp..",
                "...pphhhpp...",
                ".....ppp.....",
            ],
            Some(&[3, 3, 0, 0, 1, 2]),
        ),
        // A plated block, 118 factory pixels in a ring and a heart, flak and lances on all four
        // sides. (184 in two rings until 2026-11-11: the outer ring is one pixel thick now, and
        // the rest of it is the hull that what's left needs.)
        drawn(
            "SHIPYARD",
            Order::Hold,
            (36, 36),
            &[
                "...ppppppppppppppppppppp...",
                "...ppppppppppppppppppppp...",
                "...ppppphhhhlllhhhhppppp...",
                "ppppppphhhggggggghhhppppppp",
                "pppppphhhhhhhghhhhhhhpppppp",
                "ppppphhhhhhhhhhhhhhhhhppppp",
                "pppphhfffffffffffffffhhpppp",
                "ppphhhfghhhhhhhhhhhgfhhhppp",
                "pphhhhfhhhhhhhhhhhhhfhhhhpp",
                "pphhhhfhhhhhhhhhhhhhfhhhhpp",
                "pphhhhfhhfffffffffhhfhhhhpp",
                "pphhhhfhhfffffffffhhfhhhhpp",
                "pplhhhfhhfffffffffhhfhhhlpp",
                "pplghhfhhffffcffffhhfhhglpp",
                "pplhhhfhhfffffffffhhfhhhlpp",
                "pphhhhfhhfffffffffhhfhhhhpp",
                "pphhhhfhhfffffffffhhfhhhhpp",
                "pphhhhfhhhhhhhhhhhhhfhhhhpp",
                "pphhhhfhhhhhhhhhhhhhfhhhhpp",
                "ppphhhfghhhhhhhhhhhgfhhhppp",
                "pppphhfffffffffffffffhhpppp",
                "ppppphhhhhhhhhhhhhhhhhppppp",
                "pppppphhhhhhhhhhhhhhhpppppp",
                "ppppppphhggggggggghhppppppp",
                "...ppppphhhhlllhhhhppppp...",
                "...ppppppppppppppppppppp...",
                "...ppppppppppppppppppppp...",
            ],
            Some(&[3, 2, 2, 6]),
        ),
        // Twenty-seven cannons behind plate, and a full row of engines across the stern.
        drawn(
            "CAPITAL",
            Order::Guard,
            (33, 31),
            &[
                "...............ppp...............",
                "..............pphpp..............",
                ".............pphhhpp.............",
                "............pphnnnhpp............",
                "...........pphhnnnhhpp...........",
                "..........pphhhnnnhhhpp..........",
                ".........ppnnnhnnnhnnnpp.........",
                "........pphnnnhnnnhnnnhpp........",
                ".......pphhnnnhnnnhnnnhhpp.......",
                ".......phhhhhhhhhhhhhhhhhp.......",
                ".......phhhhhhhhchhhhhhhhp.......",
                "......gpnnnnnnghhhgnnnnnnpg......",
                ".......pnnnnnnhhhhhnnnnnnp.......",
                ".......pnnnnnnhhhhhnnnnnnp.......",
                ".......pnnnnnnhhhhhnnnnnnp.......",
                ".......pnnnnnnhhhhhnnnnnnp.......",
                ".......pnnnnnnhhhhhnnnnnnp.......",
                ".......pnnnnnnhhhhhnnnnnnp.......",
                ".......pnnnnnnhhchhnnnnnnp.......",
                "......gpnnnnnnhhhhhnnnnnnpg......",
                ".......phhhhhhhhhhhhhhhhhp.......",
                ".......pnnnnnnhnnnhnnnnnnp.......",
                ".......pnnnnnnhnnnhnnnnnnp.......",
                ".......pnnnnnnhnnnhnnnnnnp.......",
                ".......pnnnnnnhhhhhnnnnnnp.......",
                ".......pnnnnnnhhhhhnnnnnnp.......",
                ".......pnnnnnnhhchhnnnnnnp.......",
                "......gphhhhhhghhhghhhhhhpg......",
                ".......phhhhhhhnnnhhhhhhhp.......",
                ".......phhhhhhhnnnhhhhhhhp.......",
                ".......phhhhhhhnnnhhhhhhhp.......",
                ".......phhhhhhghhhghhhhhhp.......",
                ".......phhhhhhhnnnhhhhhhhp.......",
                ".......phhhhhhhnnnhhhhhhhp.......",
                "........phhhhhhnnnhhhhhhp........",
                "ppppppppppppppppppppppppppppppppp",
                "peeeeeeeeeeeeeeeeeeeeeeeeeeeeeeep",
            ],
            None,
        ),
    ]
}

/// The starter deck's cards as they were before a round changed them, by slot. A saved design
/// still exactly like one of these was never edited, so loading swaps in the current card (match
/// round 2 drew the four warships on GUARD; after it Henry redrew all seven to use their power,
/// and the miner was redrawn once more).
pub fn retired_starters() -> Vec<(usize, Design)> {
    let d = defaults();
    let mut v = vec![(0, d[0].clone()), (1, d[1].clone()), (2, d[2].clone()), (6, d[4].clone())];
    v.extend(round2_deck().into_iter().enumerate());
    // Henry's first miner (2026-11-10): five scoops and five flak on one engine. Replaced
    // the same day at Henry's request.
    v.push((3, drawn("MINER", Order::Harvest, (48, 45), &[".g.", "ggg", "sss", "sgs", "hch", ".e."], None)));
    v.extend(cheap_parts_deck());
    v
}

/// The five starter cards that the costs of 2026-11-11 left short of power (a lance at 3, a
/// factory pixel at 2, a scoop at 1), as Henry drew them and as the miner tournament left the
/// miner. Kept for saves that still hold them.
fn cheap_parts_deck() -> Vec<(usize, Design)> {
    vec![
        (1, drawn("LANCER", Order::Guard, (48, 47), &[".l.", ".l.", "hch", "e.e"], None)),
        (2, drawn("FRIGATE", Order::Guard, (45, 44), &["....p....", "...php...", "..pplpp..", "ppghlhgpp", "phghchghp", "phhhlhhhp", "ppghlhgpp", "e.ppppp.e", "..e.e.e.."], None)),
        (3, drawn("MINER", Order::Harvest, (48, 47), &[".g.", "sss", "ece", ".e."], None)),
        (4, drawn("OUTPOST", Order::Hold, (43, 43), &[".....ppp.....", "...pphhhpp...", "..phhhghhhp..", ".phhfffffhhp.", ".phfffffffhp.", "phfffhhhfffhp", "pgfffhchfffgp", "phfffhhhfffhp", ".phfffffffhp.", ".phhfffffhhp.", "..phhhghhhp..", "...pphhhpp...", ".....ppp....."], Some(&[3, 3, 0, 0, 1, 2]))),
        (
            5,
            drawn(
                "SHIPYARD",
                Order::Hold,
                (36, 36),
                &[
                    "...ppppppppppppppppppppp...",
                    "...ppppppppppppppppppppp...",
                    "...ppppphhhhlllhhhhppppp...",
                    "ppppppphhhggggggghhhppppppp",
                    "pppppphhhhhhhhhhhhhhhpppppp",
                    "pppppfffffffffffffffffppppp",
                    "pppphfffffffffffffffffhpppp",
                    "ppphhffghhhhhfhhhhhgffhhppp",
                    "pphhhffhhhhhhhhhhhhhffhhhpp",
                    "pphhhffhhhhhhfhhhhhhffhhhpp",
                    "pphhhffhhfffffffffhhffhhhpp",
                    "pphhhffhhfffffffffhhffhhhpp",
                    "pplhhffhhfffffffffhhffhhlpp",
                    "pplghffhhffffcffffhhffhglpp",
                    "pplhhffhhfffffffffhhffhhlpp",
                    "pphhhffhhfffffffffhhffhhhpp",
                    "pphhhffhhfffffffffhhffhhhpp",
                    "pphhhffhhhhhhhhhhhhhffhhhpp",
                    "pphhhffhhhhhhhhhhhhhffhhhpp",
                    "ppphhffghhhhhhhhhhhgffhhppp",
                    "pppphfffffffffffffffffhpppp",
                    "pppppfffffffffffffffffppppp",
                    "pppppphhhhhhhhhhhhhhhpppppp",
                    "ppppppphhggggggggghhppppppp",
                    "...ppppphhhhlllhhhhppppp...",
                    "...ppppppppppppppppppppp...",
                    "...ppppppppppppppppppppp...",
                ],
                Some(&[3, 2, 2, 6]),
            ),
        ),
    ]
}

/// Match round 2's starter deck, as it was before Henry redrew it: cards with hull to spare,
/// and an outpost slow enough to leave its bank room to save. Kept for saves that still hold
/// its cards.
fn round2_deck() -> Vec<Design> {
    let d = defaults();
    let guard = |d: &Design| Design { order: Order::Guard, ..d.clone() };
    let mut outpost = Design::from_rows(
        "OUTPOST",
        Order::Hold,
        &[
            ".....ppp.....",
            "...pphhhpp...",
            "..phhhghhhp..",
            ".phhfffffhhp.",
            ".phfhhhhhfhp.",
            "phhfhhhhhfhhp",
            "pghfhhchhfhgp",
            "phhfhhhhhfhhp",
            ".phfhhhhhfhp.",
            ".phhfffffhhp.",
            "..phhhghhhp..",
            "...pphhhpp...",
            ".....ppp.....",
        ],
    );
    outpost.seeded = true;
    outpost.build = vec![3, 3, 0, 0, 1, 2];
    let miner = Design::from_rows("MINER", Order::Harvest, &["s.s", "sgs", "hch", ".e."]);
    vec![guard(&d[0]), guard(&d[1]), guard(&d[2]), miner, outpost, round2_shipyard(), guard(&d[4])]
}

/// Round 2's shipyard, kept so a save still holding it gets today's: an octagon plated all
/// round, with two rings of factory pixels (88 of them) and flak behind the plate.
fn round2_shipyard() -> Design {
    let mut d = Design::blank("SHIPYARD");
    let c = GRID / 2;
    for y in -12i32..=12 {
        for x in -12i32..=12 {
            if x.abs() + y.abs() <= 17 {
                d.set(c + x, c + y, Some(Part::Hull));
            }
        }
    }
    let skin: Vec<(i32, i32)> = (0..GRID * GRID)
        .map(|i| (i % GRID, i / GRID))
        .filter(|&(x, y)| d.get(x, y) == Some(Part::Hull) && [(1, 0), (-1, 0), (0, 1), (0, -1)].iter().any(|&(dx, dy)| d.get(x + dx, y + dy).is_none()))
        .collect();
    for (x, y) in skin {
        d.set(x, y, Some(Part::Plate));
    }
    for y in -8i32..=8 {
        for x in -8i32..=8 {
            let m = x.abs().max(y.abs());
            if m == 3 || m == 8 {
                d.set(c + x, c + y, Some(Part::Factory));
            }
        }
    }
    for (x, y) in [(10, 0), (-10, 0), (0, 10), (0, -10), (6, 6), (-6, 6), (6, -6), (-6, -6)] {
        d.set(c + x, c + y, Some(Part::Flak));
    }
    d.set(c, c, Some(Part::Core));
    d.order = Order::Hold;
    d.seeded = true;
    d.build = vec![3, 2, 2, 6];
    d
}

/// Earlier versions of the defaults, by slot. A saved design still exactly like one of these was
/// never edited, so loading swaps in the current default (round 3 buried the old destroyer's
/// front engines and gave the capital extra cores).
pub fn retired_defaults() -> Vec<(usize, Design)> {
    let destroyer = Design::from_rows(
        "DESTROYER",
        Order::Hunt,
        &[
            ".....ppp.....",
            "....pphpp....",
            "...pphhhpp...",
            "..pphhhhhpp..",
            ".pphhnnnhhpp.",
            ".phhhnnnhhhp.",
            ".phhhnnnhhhp.",
            ".phhhhhhhhhp.",
            ".phhhhhhhhhp.",
            "gphhhhhhhhhpg",
            ".phhhhhhhhhp.",
            ".phhhhchhhhp.",
            ".phhhhhhhhhp.",
            "gphhhhhhhhhpg",
            ".phhhhhhhhhp.",
            "..phhhhhhhp..",
            "...ppppppp...",
            "...e.e.e.e...",
            "...e.e.e.e...",
        ],
    );
    let mut capital = defaults()[4].clone();
    let (cx, top) = (GRID / 2, (GRID - 37) / 2);
    for row in [10, 26] {
        capital.set(cx, top + row, Some(Part::Hull));
    }
    vec![(3, destroyer), (4, capital)]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_default_is_legal() {
        for d in defaults() {
            assert!(d.check().ok(), "{}: {:?}", d.name, d.check());
        }
    }

    #[test]
    fn a_designs_style_and_group_are_kept_in_its_line_and_old_lines_still_load() {
        let mut d = defaults()[0].clone();
        // With neither chosen, the line is what it was before there was anything to choose.
        let plain = d.to_line();
        assert_eq!(plain.split('|').count(), 6, "{plain}");
        let back = Design::from_line(&plain).unwrap();
        assert_eq!((back.style, back.group), (None, 1));
        // Either one chosen, both are written, and come back.
        for (style, group) in [(Some(Style::Orbit), 1), (None, 6), (Some(Style::Kite), 12), (Some(Style::Runs), 3)] {
            (d.style, d.group) = (style, group);
            let line = d.to_line();
            assert_eq!(line.split('|').count(), 8, "{line}");
            let back = Design::from_line(&line).unwrap();
            assert_eq!((back.style, back.group, &back.cells, back.order), (style, group, &d.cells, d.order), "{line}");
        }
        // A group size that isn't one of the four is no group at all.
        let odd = d.to_line().rsplit_once('|').unwrap().0.to_string() + "|5";
        assert_eq!(Design::from_line(&odd).unwrap().group, 1);
        assert_eq!(Design::from_line(&(plain.clone() + "|WARP|3")).unwrap().style, None);
        // SAB is an order like the others.
        d.order = Order::Sab;
        assert_eq!(Design::from_line(&d.to_line()).unwrap().order, Order::Sab);
        assert!(Order::in_match(Order::Guard).contains(&Order::Sab));
    }

    #[test]
    fn a_design_flies_as_chosen_or_else_as_its_drawing_suggests() {
        let d = defaults();
        let (mut fighter, mut capital) = (d[0].clone(), d[4].clone());
        assert_eq!((fighter.flies(), capital.flies()), (Some(Style::Runs), Some(Style::Kite)));
        fighter.style = Some(Style::Orbit);
        capital.style = Some(Style::Runs);
        assert_eq!((fighter.flies(), capital.flies()), (Some(Style::Orbit), Some(Style::Runs)));
        // No engines, nothing to choose: whatever its line says, it stays put.
        let mut post = Design::from_rows("POST", Order::Hold, &["ggg", "hch"]);
        post.style = Some(Style::Orbit);
        assert_eq!(post.flies(), None);
    }

    #[test]
    fn the_defaults_fly_the_way_their_drawings_suggest() {
        let d = defaults();
        assert_eq!(d[0].stats().flight, Flight::Runs, "fighter");
        assert_eq!(d[1].stats().flight, Flight::Runs, "lancer");
        assert_eq!(d[2].stats().flight, Flight::StandOff, "frigate");
        assert_eq!(d[4].stats().flight, Flight::StandOff, "capital");
        assert!(d[0].stats().vmax > d[2].stats().vmax);
        assert!(d[2].stats().turn < d[3].stats().turn && d[3].stats().turn < d[4].stats().turn, "bigger ships turn slower");
        assert_eq!(d[3].stats().cannon, 1);
        assert_eq!(d[4].stats().cannon, 4);
        assert_eq!(d[6].stats().cannon, 9);
        assert_eq!(d[6].stats().flight, Flight::StandOff, "titan");
        let sh = d[6].shape();
        assert!(sh.w <= GRID && sh.h <= GRID && sh.h >= 95, "the titan is {}x{}", sh.w, sh.h);
    }

    #[test]
    fn saves_from_the_41_grid_still_load_centred() {
        // A round 3 line: the whole 41x41 grid, no position.
        let mut rows = vec![".".repeat(41); 41];
        rows[19] = format!("{}g.g{}", ".".repeat(19), ".".repeat(19));
        rows[20] = format!("{}hch{}", ".".repeat(19), ".".repeat(19));
        rows[21] = format!("{}e.e{}", ".".repeat(19), ".".repeat(19));
        let d = Design::from_line(&format!("OLD|HUNT|{}", rows.join("/"))).expect("parses");
        assert_eq!(d.to_line(), Design::from_rows("OLD", Order::Hunt, &["g.g", "hch", "e.e"]).to_line());
        assert_eq!(d.get(GRID / 2, GRID / 2), Some(Part::Core));
    }

    #[test]
    fn boxes_fill_and_erase_and_tile_cannons() {
        let mut d = Design::blank("T");
        assert!(d.fill_box((10, 10), (14, 12), Some(Part::Hull)));
        assert_eq!(d.stats().count, 15);
        // Cannons tile in whole blocks: a 7x4 box takes two, side by side.
        let mut e = Design::blank("T");
        e.fill_box((20, 20), (26, 23), Some(Part::Cannon));
        assert_eq!(e.cannon_blocks(), (vec![(20, 20), (23, 20)], vec![]));
        // Erasing a box that clips a cannon takes the whole block.
        e.fill_box((22, 22), (22, 22), None);
        assert_eq!(e.cannon_blocks(), (vec![(23, 20)], vec![]));
        assert_eq!(e.stats().count, 9);
    }

    #[test]
    fn fills_take_holes_and_patches_but_not_open_space() {
        let mut d = Design::from_rows("T", Order::Hunt, &["ppppp", "p...p", "p.c.p", "p...p", "ppppp"]);
        let c = GRID / 2;
        // Outside the ring is open space.
        assert!(d.flood((0, 0), Some(Part::Hull)).is_err());
        // The hole inside fills, around the core.
        assert_eq!(d.flood((c - 1, c - 1), Some(Part::Hull)), Ok(true));
        assert_eq!(d.stats().count, 25);
        assert!(d.check().ok());
        // Clicking the ring swaps the whole ring.
        assert_eq!(d.flood((c - 2, c - 2), Some(Part::Hull)), Ok(true));
        assert!(!d.cells.contains(&Some(Part::Plate)));
        assert!(d.flood((c, c - 1), Some(Part::Cannon)).is_err());
    }

    #[test]
    fn stamps_copy_turn_and_paste_as_plain_pixels() {
        let src = Design::from_rows("T", Order::Hunt, &["g.g", "hch", "e.e"]);
        let st = src.as_stamp();
        assert_eq!((st.w, st.h), (3, 3));
        // Four quarter turns come back to the start; one turn puts the engines on the left.
        assert_eq!(st.turned().turned().turned().turned(), st);
        assert_eq!(st.turned().get(0, 0), Some(Part::Engine));
        assert_eq!(st.flipped().flipped(), st);
        let mut d = Design::blank("T");
        assert!(d.paste(&st, (5, 5)));
        assert_eq!(d.get(6, 6), Some(Part::Core));
        // Empty stamp cells leave what's under them.
        d.set(6, 5, Some(Part::Plate));
        d.paste(&st, (5, 5));
        assert_eq!(d.get(6, 5), Some(Part::Plate));
        // A copied box leaves out a cannon block it cuts through.
        let mut e = Design::blank("T");
        e.put(11, 11, Some(Part::Cannon));
        e.set(13, 11, Some(Part::Hull));
        let cut = e.copy_box((11, 10), (13, 12));
        assert_eq!((cut.w, cut.h), (1, 1));
        assert_eq!(cut.get(0, 0), Some(Part::Hull));
        let whole = e.copy_box((10, 10), (13, 12));
        assert_eq!(whole.cells.iter().filter(|c| **c == Some(Part::Cannon)).count(), 9);
        // Pasting over part of a cannon removes the whole block first.
        e.paste(&Stamp { w: 1, h: 1, cells: vec![Some(Part::Hull)] }, (10, 10));
        assert_eq!(e.stats().cannon, 0);
        assert_eq!(e.cannon_blocks().1, vec![]);
    }

    #[test]
    fn every_part_has_one_category() {
        for p in PARTS {
            assert_eq!(CATS.iter().filter(|c| c.parts().contains(&p)).count(), 1, "{:?}", p);
        }
    }

    #[test]
    fn loose_pixels_are_flagged_and_extra_cores_cost_more_each() {
        let mut d = Design::from_rows("T", Order::Hunt, &["g.g", "hch", "e.e"]);
        assert!(d.check().ok());
        d.set(0, 0, Some(Part::Hull));
        assert_eq!(d.check().loose, vec![(0, 0)]);
        d.set(0, 0, None);
        // A second core joined to the ship is legal, at a price.
        d.set(GRID / 2, GRID / 2 - 1, Some(Part::Core));
        assert!(d.check().ok(), "{:?}", d.check());
        assert_eq!(d.check().cores, 2);
        assert_eq!(d.stats().cost, d.stats().count + 8);
        // A core off on its own is a loose pixel, not a second ship.
        d.set(0, 0, Some(Part::Core));
        assert_eq!(d.check().loose, vec![(0, 0)]);
        assert_eq!([1, 2, 3, 4, 5].map(core_cost), [0, 8, 24, 56, 120]);
    }

    #[test]
    fn engines_need_a_straight_three_wide_way_out() {
        let buried = |rows: &[&str]| Design::from_rows("T", Order::Hunt, rows).check().buried.len();
        // Straight out the back, or out the side.
        assert_eq!(buried(&["hch", "e.e"]), 0);
        assert_eq!(buried(&["hhh", "ech", "hhh"]), 0);
        // A pixel diagonally behind is in the way: the line either side of the engine must be clear.
        assert_eq!(buried(&["hchh", "hhe.", "hh.."]), 1);
        // An engine stacked behind another is buried; the one at the back isn't.
        assert_eq!(buried(&["hhchh", "hheh.", "hheh."]), 1);
        // A 2-wide slot out the back is too narrow, and so is a 3-wide one that isn't centred on
        // the engine; a 3-wide one centred on it is enough.
        assert_eq!(buried(&["hhchhh", "hhehhh", "hh..hh", "hh..hh"]), 1);
        assert_eq!(buried(&["hhchhh", "hhehhh", "hh...h", "hh...h"]), 1);
        assert_eq!(buried(&["hhchhh", "hhehhh", "h...hh", "h...hh"]), 0);
        // A straight well, however deep in plate, leaves a line of fire to the engine.
        assert_eq!(buried(&["ppppppp", "pphchpp", "pp.e.pp", "pp...pp", "pp...pp", "pp...pp"]), 0);
        // A channel that turns a corner doesn't: plate wrapped round the engine (Henry's capital).
        let bent = ["ppppppppp", "pphchpppp", "pp.e.pppp", "pp...pppp", "pp...pppp", "pp...pppp", "pp.......", "pp.......", "pp.......", "ppppppppp"];
        assert_eq!(buried(&bent), 1);
    }

    #[test]
    fn untouched_old_defaults_are_recognised() {
        let now = defaults();
        for (slot, old) in retired_defaults() {
            assert_eq!(old.name, now[slot].name);
            assert_ne!(old.to_line(), now[slot].to_line());
        }
        assert!(!retired_defaults()[0].1.check().ok(), "the old destroyer's front engines are buried");
    }

    #[test]
    fn cannons_stamp_and_erase_as_whole_blocks() {
        let mut d = Design::blank("T");
        assert!(d.put(10, 10, Some(Part::Cannon)));
        assert_eq!(d.stats().cannon, 1);
        assert_eq!(d.stats().count, 9);
        // Stamping over part of it replaces it rather than leaving scraps.
        d.put(11, 10, Some(Part::Cannon));
        assert_eq!(d.cannon_blocks(), (vec![(10, 9)], vec![]));
        // Painting over any cell of a cannon removes the whole block.
        d.put(10, 9, Some(Part::Hull));
        assert_eq!(d.stats().cannon, 0);
        assert_eq!(d.stats().count, 1);
        // Too close to the edge to fit.
        assert!(!d.put(0, 5, Some(Part::Cannon)));
    }

    #[test]
    fn shapes_know_where_their_cannons_are() {
        let d = &defaults()[3];
        let s = d.shape();
        assert_eq!(s.cannons.len(), 1);
        let c = s.cannons[0];
        assert_eq!((c as i32 % s.w, c as i32 / s.w), (6, 5));
    }

    #[test]
    fn save_lines_round_trip() {
        for d in defaults() {
            let back = Design::from_line(&d.to_line()).expect("parses");
            assert_eq!(back.to_line(), d.to_line());
        }
    }

    #[test]
    fn rotated_cells_map_back() {
        for d in defaults() {
            let s = d.shape();
            for f in 0..4 {
                for i in 0..s.cells.len() {
                    if s.cells[i].is_some() {
                        let (x, y) = s.cell_xy(50, 50, f, i);
                        assert_eq!(s.cell_at(50, 50, f, x, y), Some(i));
                    }
                }
            }
        }
    }

    #[test]
    fn power_comes_from_hull_and_cores() {
        // A fighter: 2 hull and a core make 6; 2 flak and 2 engines use 4.
        let st = defaults()[0].stats();
        assert_eq!((st.made, st.used, st.power_pct()), (6, 4, 100));
        // Every default has the power it needs, so the old matchups don't move.
        for d in defaults() {
            let st = d.stats();
            assert!(st.used <= st.made, "{} uses {} of {}", d.name, st.used, st.made);
        }
        // A cannon on a bare core: 4 made, 9 used. Its gun reloads 9/4 as slow and its engine
        // pushes 4/9 as hard.
        let boat = Design::from_rows("BOAT", Order::Hunt, &["nnn", "nnn", "nnn", ".c.", ".e."]);
        let st = boat.stats();
        assert_eq!((st.made, st.used, st.power_pct()), (4, 9, 44));
        assert_eq!(st.reload(45), 102);
        assert_eq!(st.thrust, 8 * 4 / 9);
        // Five pixels of hull bring it up to full power.
        let fed = Design::from_rows("BOAT", Order::Hunt, &["nnn", "nnn", "nnn", "hch", "hhh", ".e."]);
        assert_eq!(fed.stats().power_pct(), 100);
        assert_eq!(fed.stats().reload(45), 45);
    }

    #[test]
    fn the_match_designs_are_legal() {
        assert!(king().check().ok(), "{:?}", king().check());
        assert!(seed().check().ok());
        let deck = starter_deck();
        assert_eq!(deck.len(), 7);
        for d in &deck {
            assert!(d.check().ok(), "{}: {:?}", d.name, d.check());
            assert!(d.stats().power_pct() == 100, "{} is short of power", d.name);
            if d.seeded {
                let c = d.start_cell().expect("a seed core");
                assert!(d.shape().is_core(c));
            }
            // Everything on a build order is a built design the full-grown factory can make.
            for &b in &d.build {
                assert!(!deck[b].seeded && deck[b].stats().count <= d.stats().max_build(), "{} can't build {}", d.name, deck[b].name);
            }
        }
        assert_eq!(deck[5].stats().factory, 118);
    }

    #[test]
    fn seeded_designs_and_build_orders_save() {
        let deck = starter_deck();
        for d in &deck {
            let back = Design::from_line(&d.to_line()).expect("parses");
            assert_eq!((back.seeded, &back.build), (d.seeded, &d.build));
            assert_eq!(back.to_line(), d.to_line());
        }
        // A seeded design whose seed cell isn't a core can't fly.
        let mut d = deck[4].clone();
        d.set(SEED_CELL.0, SEED_CELL.1, Some(Part::Hull));
        assert!(d.check().no_seed && !d.check().ok());
    }

    #[test]
    fn growth_goes_outward_and_stays_in_one_piece() {
        let d = &starter_deck()[5];
        let sh = d.shape();
        let order = Design::growth_order(&sh, d.start_cell().unwrap());
        assert_eq!(order.len() as i32, d.stats().count);
        assert!(sh.is_core(order[0]));
        // Every cell after the first touches one grown before it.
        let mut grown = vec![false; sh.cells.len()];
        for (k, &c) in order.iter().enumerate() {
            let (x, y) = (c as i32 % sh.w, c as i32 / sh.w);
            let touches = [(1, 0), (-1, 0), (0, 1), (0, -1)].iter().any(|&(dx, dy)| {
                let (nx, ny) = (x + dx, y + dy);
                nx >= 0 && ny >= 0 && nx < sh.w && ny < sh.h && grown[(ny * sh.w + nx) as usize]
            });
            assert!(k == 0 || touches, "cell {c} grew out of nowhere");
            grown[c] = true;
        }
    }

    #[test]
    fn the_part_table_lines_up_with_the_parts() {
        // Every part finds its own row: the table is in the order of the enum.
        assert_eq!(PARTS.len(), PART_INFO.len());
        for p in PARTS {
            let info = p.info();
            assert_eq!(format!("{p:?}").to_uppercase(), info.name, "{:?} got {}'s row", p, info.name);
            assert_eq!(Part::from_ch(info.ch), Some(p));
            assert!(info.glyph.iter().all(|r| r.chars().count() == 7), "{}'s shape isn't 7x7", info.name);
        }
        // No two parts share a letter or a name.
        for a in &PART_INFO {
            assert_eq!(PART_INFO.iter().filter(|b| b.ch == a.ch || b.name == a.name).count(), 1, "{}", a.name);
        }
        assert_eq!(Part::Cannon.blurb().matches("WEIGHS 18.").count(), 1);
        assert_eq!(Part::power_list(true), "HULL 1, CORE 4");
        assert_eq!(secs(20), "1.3");
        assert_eq!(secs(45), "3");
    }
}
