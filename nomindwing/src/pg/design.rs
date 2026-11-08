//! Proving-ground designs: a 15x15 pixel grid where every colour is a part, and the stats that
//! drawing implies. Nothing here knows what "kind" of ship a design is: a fighter is just a
//! small drawing with engines to spare.

pub const GRID: i32 = 15;
pub const FLAK_RANGE: i32 = 48;
pub const LANCE_RANGE: i32 = 100;
/// Ticks between shots, per gun pixel.
pub const FLAK_PERIOD: i32 = 20;
pub const LANCE_PERIOD: i32 = 45;
/// Hit points a lance shot punches through before it stops.
pub const LANCE_PEN: i32 = 5;
/// Pixels each scoop carries.
pub const SCOOP_CARRY: i32 = 3;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Part {
    Hull,
    Armor,
    Flak,
    Lance,
    Engine,
    Scoop,
    Core,
}

pub const PARTS: [Part; 7] = [Part::Hull, Part::Armor, Part::Flak, Part::Lance, Part::Engine, Part::Scoop, Part::Core];

impl Part {
    pub fn hp(self) -> u8 {
        if self == Part::Armor {
            2
        } else {
            1
        }
    }
    pub fn mass(self) -> i32 {
        match self {
            Part::Armor | Part::Lance => 2,
            _ => 1,
        }
    }
    pub fn ch(self) -> char {
        match self {
            Part::Hull => 'h',
            Part::Armor => 'a',
            Part::Flak => 'g',
            Part::Lance => 'l',
            Part::Engine => 'e',
            Part::Scoop => 's',
            Part::Core => 'c',
        }
    }
    pub fn from_ch(c: char) -> Option<Part> {
        PARTS.iter().copied().find(|p| p.ch() == c)
    }
    pub fn name(self) -> &'static str {
        match self {
            Part::Hull => "HULL",
            Part::Armor => "ARMOR",
            Part::Flak => "FLAK",
            Part::Lance => "LANCE",
            Part::Engine => "ENGINE",
            Part::Scoop => "SCOOP",
            Part::Core => "CORE",
        }
    }
    pub fn blurb(self) -> &'static str {
        match self {
            Part::Hull => "HULL - TAKES 1 HIT, WEIGHS 1. HOLDS THE SHIP TOGETHER.",
            Part::Armor => "ARMOR - TAKES 2 HITS, WEIGHS 2.",
            Part::Flak => "FLAK - A TURRET. SHOOTS THE NEAREST ENEMY WITHIN 48 PX, LEADING ITS AIM. 1 HIT PER SHOT, ONE SHOT PER GUN EVERY 1.3 S.",
            Part::Lance => "LANCE - SHOOTS STRAIGHT AHEAD ONLY, UP TO 100 PX. A SLOW SHOT THAT PUNCHES THROUGH 5 HITS. ONE SHOT EVERY 3 S. WEIGHS 2.",
            Part::Engine => "ENGINE - THRUST. MORE THRUST FOR THE WEIGHT MEANS A HIGHER TOP SPEED AND QUICKER TURNS AND COURSE CHANGES.",
            Part::Scoop => "SCOOP - MINES ROCK AND PICKS UP LOOSE PIXELS. EACH SCOOP CARRIES 3.",
            Part::Core => "CORE - THE SHIP'S LIFE. IT DIES WHEN EVERY CORE PIXEL IS GONE. NO CORE, NO SHIP.",
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Order {
    Hunt,
    Guard,
    Harvest,
    Hold,
}

pub const ORDERS: [Order; 4] = [Order::Hunt, Order::Guard, Order::Harvest, Order::Hold];

impl Order {
    pub fn name(self) -> &'static str {
        match self {
            Order::Hunt => "HUNT",
            Order::Guard => "GUARD",
            Order::Harvest => "HARVEST",
            Order::Hold => "HOLD",
        }
    }
    pub fn blurb(self) -> &'static str {
        match self {
            Order::Hunt => "HUNT - GOES AFTER THE NEAREST ENEMY, WHEREVER IT IS.",
            Order::Guard => "GUARD - STAYS WHERE IT WAS PLACED AND FIGHTS ANYTHING THAT COMES WITHIN 100 PX OF THAT SPOT.",
            Order::Harvest => "HARVEST - MINES THE NEAREST ROCK AND HAULS THE PIXELS HOME. ITS GUNS STILL FIRE ON THE WAY.",
            Order::Hold => "HOLD - STAYS EXACTLY WHERE IT WAS PLACED, TURNING ITS LANCES TO FACE ANY ENEMY IN RANGE.",
        }
    }
    fn from_name(s: &str) -> Option<Order> {
        ORDERS.iter().copied().find(|o| o.name() == s)
    }
}

/// How a ship flies, which follows from its drawing alone.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
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

#[derive(Clone, Copy, Debug)]
pub struct Stats {
    pub count: i32,
    pub mass: i32,
    pub hp: i32,
    pub engines: i32,
    pub flak: i32,
    pub lance: i32,
    pub scoops: i32,
    pub cores: i32,
    pub thrust: i32,
    /// Top speed in quarter-pixels per tick.
    pub vmax: i32,
    /// Ticks per quarter turn (0 = can't turn).
    pub turn: i32,
    pub flight: Flight,
}

impl Stats {
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

pub fn stats_of(cells: &[Option<Part>]) -> Stats {
    let n = |p: Part| cells.iter().filter(|c| **c == Some(p)).count() as i32;
    let count = cells.iter().flatten().count() as i32;
    let mass: i32 = cells.iter().flatten().map(|p| p.mass()).sum();
    let hp: i32 = cells.iter().flatten().map(|p| p.hp() as i32).sum();
    let engines = n(Part::Engine);
    let thrust = engines * 8;
    let vmax = if engines == 0 { 0 } else { (engines * 40 / mass.max(1) + 2).clamp(3, 12) };
    let turn = if engines == 0 { 0 } else { (mass * 2 / thrust).clamp(1, 24) };
    let flight = if engines == 0 {
        Flight::Still
    } else if thrust >= mass {
        Flight::Runs
    } else {
        Flight::StandOff
    };
    Stats { count, mass, hp, engines, flak: n(Part::Flak), lance: n(Part::Lance), scoops: n(Part::Scoop), cores: n(Part::Core), thrust, vmax, turn, flight }
}

/// A design trimmed to its bounding box, facing north (up is forward).
#[derive(Clone, Debug)]
pub struct Shape {
    pub w: i32,
    pub h: i32,
    pub cells: Vec<Option<Part>>,
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
        let (w, h) = self.dims(facing);
        let (rx, ry) = (px - (x - w / 2), py - (y - h / 2));
        if rx < 0 || ry < 0 || rx >= w || ry >= h {
            return None;
        }
        let (dx, dy) = match facing & 3 {
            0 => (rx, ry),
            1 => (ry, self.h - 1 - rx),
            2 => (self.w - 1 - rx, self.h - 1 - ry),
            _ => (self.w - 1 - ry, rx),
        };
        let i = (dy * self.w + dx) as usize;
        self.cells[i].map(|_| i)
    }
}

#[derive(Clone, Debug)]
pub struct Design {
    pub name: String,
    pub order: Order,
    /// GRID x GRID, row-major, north up.
    pub cells: Vec<Option<Part>>,
}

impl Design {
    pub fn blank(name: &str) -> Design {
        Design { name: name.into(), order: Order::Hunt, cells: vec![None; (GRID * GRID) as usize] }
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

    pub fn set(&mut self, x: i32, y: i32, p: Option<Part>) {
        if (0..GRID).contains(&x) && (0..GRID).contains(&y) {
            self.cells[(y * GRID + x) as usize] = p;
        }
    }

    pub fn stats(&self) -> Stats {
        stats_of(&self.cells)
    }

    pub fn shape(&self) -> Shape {
        let used: Vec<(i32, i32)> = (0..GRID * GRID).filter(|&i| self.cells[i as usize].is_some()).map(|i| (i % GRID, i / GRID)).collect();
        if used.is_empty() {
            return Shape { w: 1, h: 1, cells: vec![None] };
        }
        let (x0, x1) = (used.iter().map(|p| p.0).min().unwrap(), used.iter().map(|p| p.0).max().unwrap());
        let (y0, y1) = (used.iter().map(|p| p.1).min().unwrap(), used.iter().map(|p| p.1).max().unwrap());
        let (w, h) = (x1 - x0 + 1, y1 - y0 + 1);
        let cells = (0..w * h).map(|i| self.get(x0 + i % w, y0 + i / w)).collect();
        Shape { w, h, cells }
    }

    /// One line for the save file: `NAME|ORDER|row/row/...` with `.` for empty cells.
    pub fn to_line(&self) -> String {
        let rows: Vec<String> = (0..GRID).map(|y| (0..GRID).map(|x| self.get(x, y).map_or('.', |p| p.ch())).collect()).collect();
        format!("{}|{}|{}", self.name, self.order.name(), rows.join("/"))
    }

    pub fn from_line(s: &str) -> Option<Design> {
        let mut it = s.trim().split('|');
        let name = it.next()?.to_string();
        let order = Order::from_name(it.next()?)?;
        let rows: Vec<&str> = it.next()?.split('/').collect();
        if rows.len() != GRID as usize {
            return None;
        }
        let mut d = Design::blank(&name);
        d.order = order;
        for (y, r) in rows.iter().enumerate() {
            for (x, c) in r.chars().enumerate().take(GRID as usize) {
                d.set(x as i32, y as i32, Part::from_ch(c));
            }
        }
        Some(d)
    }
}

/// The starting library. Slots are referred to by index in the test setups.
pub fn defaults() -> Vec<Design> {
    vec![
        Design::from_rows("FIGHTER", Order::Hunt, &["g.g", "hch", "e.e"]),
        Design::from_rows("LANCER", Order::Hunt, &[".l.", "hch", "e.e"]),
        // Lances behind an armour row: a ship's own shots pass through its own pixels.
        Design::from_rows("FRIGATE", Order::Hunt, &[".a.a.", ".lhl.", "ahhha", "ahcha", "ahhha", ".e.e."]),
        Design::from_rows(
            "CAPITAL",
            Order::Hunt,
            &[
                "..aagagaa..",
                ".ahhhhhhha.",
                "aghhhhhhhga",
                "ahhhhhhhhha",
                "aghhccchhga",
                "ahhhccchhha",
                "aghhccchhga",
                "ahhhhhhhhha",
                "aghhhhhhhga",
                ".ahhhhhhha.",
                "..aeeeeea..",
            ],
        ),
        Design::from_rows("MINER", Order::Harvest, &["s.s", "shs", "hch", ".e."]),
        Design::from_rows("GUNMINER", Order::Harvest, &["s.s", "sgs", "hch", ".e."]),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_defaults_fly_the_way_their_drawings_suggest() {
        let d = defaults();
        assert_eq!(d[0].stats().flight, Flight::Runs, "fighter");
        assert_eq!(d[1].stats().flight, Flight::Runs, "lancer");
        assert_eq!(d[2].stats().flight, Flight::StandOff, "frigate");
        assert_eq!(d[3].stats().flight, Flight::StandOff, "capital");
        assert!(d[0].stats().vmax > d[2].stats().vmax && d[2].stats().vmax > d[3].stats().vmax);
        assert_eq!(d[3].stats().flak, 10);
        assert_eq!(d[3].stats().cores, 9);
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
}
