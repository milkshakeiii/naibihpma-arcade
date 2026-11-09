//! Proving-ground designs: a 41x41 pixel grid where every colour is a part, and the stats that
//! drawing implies. Nothing here knows what "kind" of ship a design is: a fighter is just a
//! small drawing with engines to spare.
//!
//! Round 2 rules: every pixel must connect to the core through other pixels (edges, not
//! corners); plate shrugs off hits no stronger than it; the cannon is a 3x3 block.
//! Round 3: extra cores are allowed at a steeply rising cost, and every engine needs a 3-wide
//! way out of the ship for its exhaust.

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
/// The first line of a save file (the format hasn't changed since round 2).
pub const SAVE_HEADER: &str = "#ROUND2";
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
            Part::Engine => "ENGINE - THRUST. ITS EXHAUST NEEDS A 3-WIDE WAY OUT OF THE SHIP, IN ANY DIRECTION AND ALLOWED TO BEND. A BURIED ENGINE SHOWS RED. MORE THRUST FOR THE WEIGHT MEANS A HIGHER TOP SPEED AND QUICKER TURNS.",
            Part::Scoop => "SCOOP - MINES ROCK AND PICKS UP LOOSE PIXELS. EACH SCOOP CARRIES 3.",
            Part::Core => "CORE - THE SHIP'S LIFE: IT DIES WHEN ITS LAST CORE IS GONE. EXTRA CORES COST MORE EACH: +8, +16, +32. CUT IN TWO, EVERY PIECE WITH A CORE FLIES ON AS A SHIP; PIECES WITHOUT ONE BREAK AWAY.",
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
    pub cores: i32,
    /// What the ship costs to build: its pixels plus its extra cores.
    pub cost: i32,
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
    let cores = n(Part::Core);
    let cost = count + core_cost(cores);
    Stats { count, cores, cost, mass, engines, plate: n(Part::Plate), flak: n(Part::Flak), lance: n(Part::Lance), cannon: cannons, scoops: n(Part::Scoop), thrust, vmax, turn, flight }
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
    /// Engines with no 3-wide way out of the ship for their exhaust.
    pub buried: Vec<(i32, i32)>,
}

impl Check {
    pub fn ok(&self) -> bool {
        self.cores >= 1 && self.loose.is_empty() && self.buried.is_empty()
    }
    pub fn problem(&self) -> Option<String> {
        let s = |n: usize| if n == 1 { "" } else { "S" };
        if self.cores == 0 {
            Some("NO CORE: IT CAN'T FLY".into())
        } else if !self.loose.is_empty() {
            let n = self.loose.len();
            Some(format!("{n} LOOSE PIXEL{}: CONNECT TO CORE", s(n)))
        } else if !self.buried.is_empty() {
            let n = self.buried.len();
            Some(format!("{n} BURIED ENGINE{}: NO 3-WIDE EXIT", s(n)))
        } else {
            None
        }
    }
}

/// Engines whose exhaust has no way out: a 3x3 window of empty space has to fit against one
/// face of the engine and slide, a cell at a time through empty space, out past the edge of the
/// ship's bounding box. It can leave in any direction and can turn corners. `filled` says whether
/// a cell holds a pixel (anything outside the drawing is empty).
pub fn buried_engines(engines: &[(i32, i32)], bbox: (i32, i32, i32, i32), filled: impl Fn(i32, i32) -> bool) -> Vec<(i32, i32)> {
    let (x0, y0, x1, y1) = bbox;
    // Windows go by their top-left corner. A window poking out past the box can slide on out
    // through empty space, so those are where the exhaust gets away. Flood inward from them to
    // find every window with a way out, then check what sits against each engine.
    let free = |wx: i32, wy: i32| (0..3).all(|dy| (0..3).all(|dx| !filled(wx + dx, wy + dy)));
    let out = |wx: i32, wy: i32| wx < x0 || wy < y0 || wx + 2 > x1 || wy + 2 > y1;
    let (w, h) = (x1 - x0 + 3, y1 - y0 + 3);
    let inside = |wx: i32, wy: i32| (x0 - 2..=x1).contains(&wx) && (y0 - 2..=y1).contains(&wy);
    let idx = |wx: i32, wy: i32| ((wy - (y0 - 2)) * w + (wx - (x0 - 2))) as usize;
    let mut escapes = vec![false; (w * h) as usize];
    let mut todo = Vec::new();
    for wy in y0 - 2..=y1 {
        for wx in x0 - 2..=x1 {
            if out(wx, wy) && free(wx, wy) {
                escapes[idx(wx, wy)] = true;
                todo.push((wx, wy));
            }
        }
    }
    while let Some((wx, wy)) = todo.pop() {
        for (dx, dy) in [(1, 0), (-1, 0), (0, 1), (0, -1)] {
            let (nx, ny) = (wx + dx, wy + dy);
            if inside(nx, ny) && !escapes[idx(nx, ny)] && free(nx, ny) {
                escapes[idx(nx, ny)] = true;
                todo.push((nx, ny));
            }
        }
    }
    let leads_out = |wx: i32, wy: i32| if inside(wx, wy) { escapes[idx(wx, wy)] } else { free(wx, wy) };
    engines
        .iter()
        .copied()
        .filter(|&(ex, ey)| {
            // The windows sharing an edge with the engine: three behind it, three ahead, three each side.
            let exposed = (-2..=0).any(|k| leads_out(ex + k, ey + 1) || leads_out(ex + k, ey - 3) || leads_out(ex - 3, ey + k) || leads_out(ex + 1, ey + k));
            !exposed
        })
        .collect()
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

    /// A design is legal with at least one core, every pixel joined into one piece edge to edge
    /// (so all its cores are joined too), whole cannon blocks, and no buried engines.
    pub fn check(&self) -> Check {
        let cores: Vec<usize> = (0..self.cells.len()).filter(|&i| self.cells[i] == Some(Part::Core)).collect();
        let mut c = Check { cores: cores.len() as i32, ..Default::default() };
        // The ship is the biggest piece with a core in it; anything else is loose.
        let ship = cores.iter().map(|&k| reach(GRID, GRID, k, |i| self.cells[i].is_some())).max_by_key(|seen| seen.iter().filter(|&&b| b).count());
        if let Some(seen) = ship {
            c.loose = (0..self.cells.len()).filter(|&i| self.cells[i].is_some() && !seen[i]).map(|i| (i as i32 % GRID, i as i32 / GRID)).collect();
        }
        c.loose.extend(self.cannon_blocks().1);
        let used: Vec<(i32, i32)> = (0..GRID * GRID).filter(|&i| self.cells[i as usize].is_some()).map(|i| (i % GRID, i / GRID)).collect();
        if !used.is_empty() {
            let bbox = (used.iter().map(|p| p.0).min().unwrap(), used.iter().map(|p| p.1).min().unwrap(), used.iter().map(|p| p.0).max().unwrap(), used.iter().map(|p| p.1).max().unwrap());
            let engines: Vec<(i32, i32)> = used.iter().copied().filter(|&(x, y)| self.get(x, y) == Some(Part::Engine)).collect();
            c.buried = buried_engines(&engines, bbox, |x, y| self.get(x, y).is_some());
        }
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
    ]
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
    fn engines_need_a_three_wide_way_out() {
        let buried = |rows: &[&str]| Design::from_rows("T", Order::Hunt, rows).check().buried.len();
        // Straight out the back, or out the side.
        assert_eq!(buried(&["hch", "e.e"]), 0);
        assert_eq!(buried(&["hhh", "ech", "hhh"]), 0);
        // A pixel diagonally behind still leaves a 3-wide gap a cell over.
        assert_eq!(buried(&["hchh", "hhe.", "hh.."]), 0);
        // An engine stacked behind another is buried; the one at the back isn't.
        assert_eq!(buried(&["hhchh", "hheh.", "hheh."]), 1);
        // A 2-wide slot out the back is too narrow; a 3-wide one is enough.
        assert_eq!(buried(&["hhchhh", "hhehhh", "hh..hh", "hh..hh"]), 1);
        assert_eq!(buried(&["hhchhh", "hhehhh", "hh...h", "hh...h"]), 0);
        // A 3-wide channel that turns a corner on its way out is fine...
        let bent = ["ppppppppp", "pphchpppp", "pp.e.pppp", "pp...pppp", "pp...pppp", "pp...pppp", "pp.......", "pp.......", "pp.......", "ppppppppp"];
        assert_eq!(buried(&bent), 0);
        // ...and the same channel closed off at the end is not.
        let mut closed = bent.map(String::from);
        for r in 6..9 {
            closed[r].replace_range(8..9, "p");
        }
        assert_eq!(buried(&closed.iter().map(|s| s.as_str()).collect::<Vec<_>>()), 1);
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
}
