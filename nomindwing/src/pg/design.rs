//! Proving-ground designs: a 41x41 pixel grid where every colour is a part, and the stats that
//! drawing implies. Nothing here knows what "kind" of ship a design is: a fighter is just a
//! small drawing with engines to spare.
//!
//! Round 2 rules: every pixel must connect to the one core through other pixels (edges, not
//! corners); plate shrugs off hits no stronger than it; the cannon is a 3x3 block.

pub const GRID: i32 = 41;
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
/// Pixels each scoop carries.
pub const SCOOP_CARRY: i32 = 3;
/// The first line of a round-2 save file.
pub const SAVE_HEADER: &str = "#ROUND2";

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Part {
    Hull,
    Plate,
    Flak,
    Lance,
    Cannon,
    Engine,
    Scoop,
    Core,
}

pub const PARTS: [Part; 8] = [Part::Hull, Part::Plate, Part::Flak, Part::Lance, Part::Cannon, Part::Engine, Part::Scoop, Part::Core];

impl Part {
    /// Hits this strong or weaker bounce off.
    pub fn armor(self) -> i32 {
        if self == Part::Plate {
            1
        } else {
            0
        }
    }
    pub fn mass(self) -> i32 {
        match self {
            Part::Plate | Part::Lance | Part::Cannon => 2,
            _ => 1,
        }
    }
    pub fn ch(self) -> char {
        match self {
            Part::Hull => 'h',
            Part::Plate => 'p',
            Part::Flak => 'g',
            Part::Lance => 'l',
            Part::Cannon => 'n',
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
            Part::Plate => "PLATE",
            Part::Flak => "FLAK",
            Part::Lance => "LANCE",
            Part::Cannon => "CANNON",
            Part::Engine => "ENGINE",
            Part::Scoop => "SCOOP",
            Part::Core => "CORE",
        }
    }
    pub fn blurb(self) -> &'static str {
        match self {
            Part::Hull => "HULL - HOLDS THE SHIP TOGETHER. ANY HIT BREAKS IT. WEIGHS 1.",
            Part::Plate => "PLATE - FLAK BOUNCES OFF IT. LANCES AND CANNON SHELLS BREAK IT, AND IT SOAKS TWICE AS MUCH OF A LANCE. WEIGHS 2.",
            Part::Flak => "FLAK - A TURRET. SHOOTS THE NEAREST ENEMY WITHIN 48 PX, LEADING ITS AIM, SPREADING FIRE OVER SEVERAL TARGETS. TOO WEAK FOR PLATE. ONE SHOT EVERY 1.3 S.",
            Part::Lance => "LANCE - SHOOTS STRAIGHT AHEAD ONLY, UP TO 100 PX. PUNCHES THROUGH 6 PIXELS, OR 3 OF PLATE. ONE SHOT EVERY 3 S. WEIGHS 2.",
            Part::Cannon => "CANNON - A BIG 3X3 TURRET, 130 PX. IT TURNS SLOWLY, SO FAST CLOSE SHIPS ESCAPE IT. ITS SHELL BLASTS A 3X3 HOLE, PLATE AND ALL. PREFERS THE BIGGEST TARGET. EVERY 3 S. WEIGHS 18.",
            Part::Engine => "ENGINE - THRUST. MORE THRUST FOR THE WEIGHT MEANS A HIGHER TOP SPEED AND QUICKER TURNS AND COURSE CHANGES.",
            Part::Scoop => "SCOOP - MINES ROCK AND PICKS UP LOOSE PIXELS. EACH SCOOP CARRIES 3.",
            Part::Core => "CORE - THE SHIP'S LIFE: EXACTLY ONE PER SHIP. EVERY OTHER PIXEL MUST CONNECT TO IT THROUGH OTHERS, EDGE TO EDGE. CUT OFF, A PIECE BREAKS AWAY.",
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
    pub engines: i32,
    pub plate: i32,
    pub flak: i32,
    pub lance: i32,
    pub cannon: i32,
    pub scoops: i32,
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
    Stats { count, mass, engines, plate: n(Part::Plate), flak: n(Part::Flak), lance: n(Part::Lance), cannon: cannons, scoops: n(Part::Scoop), thrust, vmax, turn, flight }
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
#[derive(Clone, Debug)]
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
    pub fn core(&self) -> Option<usize> {
        self.cells.iter().position(|c| *c == Some(Part::Core))
    }
}

/// What's wrong with a design, if anything.
#[derive(Clone, Debug, Default)]
pub struct Check {
    pub cores: i32,
    /// Cells not connected to the core (only worked out when there is exactly one core).
    pub loose: Vec<(i32, i32)>,
}

impl Check {
    pub fn ok(&self) -> bool {
        self.cores == 1 && self.loose.is_empty()
    }
    pub fn problem(&self) -> Option<String> {
        if self.cores == 0 {
            Some("NO CORE: IT CAN'T FLY".into())
        } else if self.cores > 1 {
            Some(format!("{} CORES: ONLY ONE ALLOWED", self.cores))
        } else if !self.loose.is_empty() {
            let n = self.loose.len();
            Some(format!("{n} LOOSE PIXEL{}: CONNECT TO CORE", if n == 1 { "" } else { "S" }))
        } else {
            None
        }
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

    pub fn check(&self) -> Check {
        let cores: Vec<usize> = (0..self.cells.len()).filter(|&i| self.cells[i] == Some(Part::Core)).collect();
        let mut c = Check { cores: cores.len() as i32, loose: Vec::new() };
        if cores.len() == 1 {
            let seen = reach(GRID, GRID, cores[0], |i| self.cells[i].is_some());
            c.loose = (0..self.cells.len()).filter(|&i| self.cells[i].is_some() && !seen[i]).map(|i| (i as i32 % GRID, i as i32 / GRID)).collect();
        }
        c.loose.extend(self.cannon_blocks().1);
        c
    }

    pub fn stats(&self) -> Stats {
        stats_of(&self.cells, self.cannon_blocks().0.len() as i32)
    }

    pub fn shape(&self) -> Shape {
        let used: Vec<(i32, i32)> = (0..GRID * GRID).filter(|&i| self.cells[i as usize].is_some()).map(|i| (i % GRID, i / GRID)).collect();
        if used.is_empty() {
            return Shape { w: 1, h: 1, cells: vec![None], cannons: Vec::new() };
        }
        let (x0, x1) = (used.iter().map(|p| p.0).min().unwrap(), used.iter().map(|p| p.0).max().unwrap());
        let (y0, y1) = (used.iter().map(|p| p.1).min().unwrap(), used.iter().map(|p| p.1).max().unwrap());
        let (w, h) = (x1 - x0 + 1, y1 - y0 + 1);
        let cells = (0..w * h).map(|i| self.get(x0 + i % w, y0 + i / w)).collect();
        let cannons = self.cannon_blocks().0.iter().map(|&(bx, by)| ((by + 1 - y0) * w + bx + 1 - x0) as usize).collect();
        Shape { w, h, cells, cannons }
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
                "...e.e.e.e...",
                "...e.e.e.e...",
            ],
        ),
        // Sealed in plate all round: fighters can only strip its sponsons and engines.
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
                ".phhhhhhhhhhhhhhhhhp.",
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
                ".phhhhhhhhhhhhhhhhhp.",
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
    ]
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
    }

    #[test]
    fn loose_pixels_and_extra_cores_are_flagged() {
        let mut d = Design::from_rows("T", Order::Hunt, &["g.g", "hch", "e.e"]);
        assert!(d.check().ok());
        d.set(0, 0, Some(Part::Hull));
        assert_eq!(d.check().loose, vec![(0, 0)]);
        d.set(0, 0, None);
        d.set(GRID / 2, GRID / 2 - 1, Some(Part::Core));
        assert_eq!(d.check().cores, 2);
        assert!(!d.check().ok());
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
}
