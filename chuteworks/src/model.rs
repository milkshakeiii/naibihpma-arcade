//! Level data: directions, colors, cells, tray pieces, and the plain-text `.lvl` format.

pub const N: i32 = 8; // both boards are 8x8

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

    pub fn ccw(self) -> Dir {
        self.cw().opposite()
    }

    pub fn vertical(self) -> bool {
        matches!(self, Dir::N | Dir::S)
    }

    pub fn index(self) -> usize {
        self as usize
    }

    pub fn parse(s: &str) -> Option<Dir> {
        match s {
            "N" | "^" => Some(Dir::N),
            "E" | ">" => Some(Dir::E),
            "S" | "v" => Some(Dir::S),
            "W" | "<" => Some(Dir::W),
            _ => None,
        }
    }

    pub fn between(from: (i32, i32), to: (i32, i32)) -> Option<Dir> {
        Dir::ALL.into_iter().find(|d| from.0 + d.dx() == to.0 && from.1 + d.dy() == to.1)
    }
}

/// Paint is a bitmask of primaries. Two primaries make a secondary; all three make mud.
pub type Color = u8;
pub const GREY: Color = 0;
pub const RED: Color = 1;
pub const YELLOW: Color = 2;
pub const BLUE: Color = 4;
pub const MUD: Color = 7;

pub fn parse_color(s: &str) -> Option<Color> {
    Some(match s {
        "grey" | "gray" => GREY,
        "red" => RED,
        "yellow" => YELLOW,
        "blue" => BLUE,
        "orange" => RED | YELLOW,
        "purple" => RED | BLUE,
        "green" => YELLOW | BLUE,
        _ => return None,
    })
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Board {
    Left,
    Right,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Cell {
    Floor,
    Wall,
    Crate(u8),
    /// Left only: a belt carries items one tile per beat toward `Dir`.
    Belt(Dir),
    /// Paints one primary. On belts two lines may cross inside it.
    Booth(Color),
    /// The lever's current side. Belts: deals items to the two lever sides in turn.
    /// Floor: turns items toward the lever, then flips.
    Switch(Dir),
    /// Left only: (out, first input side). Takes from its two inputs in turn.
    Zipper(Dir, Dir),
    /// Right only: turns everything.
    Arrow(Dir),
    Bin(u8),
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Tool {
    Belt,
    Booth(Color),
    Switch,
    Zipper,
    Arrow,
}

impl Tool {
    pub fn allowed(self, b: Board) -> bool {
        match self {
            Tool::Belt | Tool::Zipper => b == Board::Left,
            Tool::Arrow => b == Board::Right,
            Tool::Booth(_) | Tool::Switch => true,
        }
    }

    pub fn of(cell: Cell) -> Option<Tool> {
        Some(match cell {
            Cell::Belt(_) => Tool::Belt,
            Cell::Booth(c) => Tool::Booth(c),
            Cell::Switch(_) => Tool::Switch,
            Cell::Zipper(..) => Tool::Zipper,
            Cell::Arrow(_) => Tool::Arrow,
            _ => return None,
        })
    }

    pub fn name(self) -> &'static str {
        match self {
            Tool::Belt => "BELT",
            Tool::Booth(RED) => "RED BOOTH",
            Tool::Booth(YELLOW) => "YELLOW BOOTH",
            Tool::Booth(_) => "BLUE BOOTH",
            Tool::Switch => "SWITCH",
            Tool::Zipper => "ZIPPER",
            Tool::Arrow => "ARROW",
        }
    }

    fn parse(s: &str) -> Option<Tool> {
        Some(match s {
            "belt" => Tool::Belt,
            "switch" => Tool::Switch,
            "zipper" => Tool::Zipper,
            "arrow" => Tool::Arrow,
            "booth-red" => Tool::Booth(RED),
            "booth-yellow" => Tool::Booth(YELLOW),
            "booth-blue" => Tool::Booth(BLUE),
            _ => return None,
        })
    }
}

#[derive(Clone, Debug)]
pub struct Crate {
    pub x: i32,
    pub y: i32,
    pub dir: Dir,
    pub items: Vec<Color>,
}

#[derive(Clone, Copy, Debug)]
pub struct Bin {
    pub x: i32,
    pub y: i32,
    pub want: Color,
    pub need: u8,
}

#[derive(Clone, Copy, Debug)]
pub struct TraySlot {
    pub tool: Tool,
    /// `None` = unlimited (belts).
    pub count: Option<u8>,
}

/// A fixed 8x8 grid, row-major.
pub type Grid = [Cell; 64];

#[inline]
pub fn idx(x: i32, y: i32) -> usize {
    (y * N + x) as usize
}

#[inline]
pub fn inside(x: i32, y: i32) -> bool {
    (0..N).contains(&x) && (0..N).contains(&y)
}

#[derive(Clone, Copy, Debug)]
pub struct Placement {
    pub board: Board,
    pub x: i32,
    pub y: i32,
    pub cell: Cell,
}

#[derive(Clone, Debug)]
pub struct Level {
    pub id: String,
    pub name: String,
    pub hall: u8,
    pub note: String,
    pub left: Grid,
    pub right: Grid,
    pub crates: Vec<Crate>,
    pub bins: Vec<Bin>,
    /// Chute rows, top first. Each is a hatch on the left board's east edge and an inlet on the
    /// right board's west edge at the same row.
    pub chutes: Vec<i32>,
    pub chute_cap: usize,
    pub tray: Vec<TraySlot>,
    pub solution: Vec<Placement>,
}

/// Player-placed cells on top of a level's fixed cells.
#[derive(Clone, Copy, PartialEq, Debug)]
pub struct Layout {
    pub left: [Option<Cell>; 64],
    pub right: [Option<Cell>; 64],
}

impl Default for Layout {
    fn default() -> Self {
        Self { left: [None; 64], right: [None; 64] }
    }
}

impl Layout {
    pub fn get(&self, b: Board, x: i32, y: i32) -> Option<Cell> {
        if !inside(x, y) {
            return None;
        }
        match b {
            Board::Left => self.left[idx(x, y)],
            Board::Right => self.right[idx(x, y)],
        }
    }

    pub fn set(&mut self, b: Board, x: i32, y: i32, c: Option<Cell>) {
        if inside(x, y) {
            match b {
                Board::Left => self.left[idx(x, y)] = c,
                Board::Right => self.right[idx(x, y)] = c,
            }
        }
    }

    pub fn placed(&self, tool: Tool) -> u8 {
        self.left.iter().chain(self.right.iter()).filter(|c| c.and_then(Tool::of) == Some(tool)).count() as u8
    }

    pub fn from_solution(level: &Level) -> Layout {
        let mut l = Layout::default();
        for p in &level.solution {
            l.set(p.board, p.x, p.y, Some(p.cell));
        }
        l
    }
}

impl Level {
    pub fn grid(&self, b: Board) -> &Grid {
        match b {
            Board::Left => &self.left,
            Board::Right => &self.right,
        }
    }

    /// The fixed cell, or `Wall` outside the board.
    pub fn fixed(&self, b: Board, x: i32, y: i32) -> Cell {
        if inside(x, y) {
            self.grid(b)[idx(x, y)]
        } else {
            Cell::Wall
        }
    }

    /// What the simulation sees: placed cells over fixed ones.
    pub fn combined(&self, layout: &Layout, b: Board) -> Grid {
        let mut g = *self.grid(b);
        for i in 0..64 {
            let placed = match b {
                Board::Left => layout.left[i],
                Board::Right => layout.right[i],
            };
            if let Some(c) = placed {
                g[i] = c;
            }
        }
        g
    }

    #[cfg(test)]
    pub fn tray_count(&self, tool: Tool) -> Option<u8> {
        self.tray.iter().find(|s| s.tool == tool).and_then(|s| s.count)
    }

    pub fn parse(id: &str, src: &str) -> Result<Level, String> {
        let mut lv = Level {
            id: id.to_string(),
            name: String::new(),
            hall: 1,
            note: String::new(),
            left: [Cell::Floor; 64],
            right: [Cell::Floor; 64],
            crates: Vec::new(),
            bins: Vec::new(),
            chutes: Vec::new(),
            chute_cap: 5,
            tray: vec![TraySlot { tool: Tool::Belt, count: None }],
            solution: Vec::new(),
        };
        let lines: Vec<&str> = src.lines().map(|l| l.split('#').next().unwrap_or("").trim_end()).collect();
        // A '#' starts a comment except inside grids, so grids are read from the raw lines.
        let raw: Vec<&str> = src.lines().collect();
        let mut grids: [Vec<String>; 2] = [Vec::new(), Vec::new()];
        let mut i = 0;
        let err = |n: usize, m: &str| format!("{id}:{}: {m}", n + 1);
        while i < lines.len() {
            let line = lines[i].trim();
            if line == "left:" || line == "right:" {
                let g = if line == "left:" { 0 } else { 1 };
                for k in 1..=8 {
                    let row = raw.get(i + k).map(|r| r.trim()).unwrap_or("");
                    if row.chars().count() != 8 {
                        return Err(err(i + k, "grid rows must be 8 characters"));
                    }
                    grids[g].push(row.to_string());
                }
                i += 9;
                continue;
            }
            i += 1;
            if line.is_empty() {
                continue;
            }
            if let Some((key, val)) = line.split_once(':') {
                let val = val.trim();
                match key.trim() {
                    "name" => lv.name = val.to_string(),
                    "hall" => lv.hall = val.parse().map_err(|_| err(i - 1, "bad hall"))?,
                    "note" => lv.note = val.to_string(),
                    "cap" => lv.chute_cap = val.parse().map_err(|_| err(i - 1, "bad cap"))?,
                    "tray" => {
                        for part in val.split(',').map(str::trim).filter(|p| !p.is_empty()) {
                            let mut w = part.split_whitespace();
                            let tool = w.next().and_then(Tool::parse).ok_or_else(|| err(i - 1, "bad tray tool"))?;
                            let n: u8 = w.next().and_then(|n| n.parse().ok()).ok_or_else(|| err(i - 1, "bad tray count"))?;
                            lv.tray.push(TraySlot { tool, count: Some(n) });
                        }
                    }
                    "solve" => lv.solution.extend(parse_solve(val).ok_or_else(|| err(i - 1, "bad solve line"))?),
                    _ => return Err(err(i - 1, "unknown key")),
                }
                continue;
            }
            let w: Vec<&str> = line.split_whitespace().collect();
            let num = |k: usize| w.get(k).and_then(|s| s.parse::<i32>().ok()).ok_or_else(|| err(i - 1, "bad number"));
            match w[0] {
                "crate" => {
                    let dir = w.get(3).and_then(|d| Dir::parse(d)).ok_or_else(|| err(i - 1, "bad crate dir"))?;
                    let items = w[4..].iter().map(|c| parse_color(c)).collect::<Option<Vec<_>>>().ok_or_else(|| err(i - 1, "bad color"))?;
                    lv.crates.push(Crate { x: num(1)?, y: num(2)?, dir, items });
                }
                "bin" => {
                    let want = w.get(3).and_then(|c| parse_color(c)).ok_or_else(|| err(i - 1, "bad bin color"))?;
                    lv.bins.push(Bin { x: num(1)?, y: num(2)?, want, need: num(4)? as u8 });
                }
                "chute" => lv.chutes.push(num(1)?),
                "switch" | "zipper" => {
                    let b = if w.get(1) == Some(&"L") { Board::Left } else { Board::Right };
                    let (x, y) = (num(2)?, num(3)?);
                    let d = w.get(4).and_then(|d| Dir::parse(d)).ok_or_else(|| err(i - 1, "bad dir"))?;
                    let cell = if w[0] == "switch" {
                        Cell::Switch(d)
                    } else {
                        Cell::Zipper(d, w.get(5).and_then(|d| Dir::parse(d)).ok_or_else(|| err(i - 1, "bad dir"))?)
                    };
                    let g = if b == Board::Left { &mut lv.left } else { &mut lv.right };
                    g[idx(x, y)] = cell;
                }
                _ => return Err(err(i - 1, "unknown line")),
            }
        }
        for (g, rows) in grids.iter().enumerate() {
            if rows.len() != 8 {
                return Err(format!("{id}: missing {} grid", if g == 0 { "left" } else { "right" }));
            }
            for (y, row) in rows.iter().enumerate() {
                for (x, ch) in row.chars().enumerate() {
                    let (x, y) = (x as i32, y as i32);
                    let cell = match ch {
                        '.' => Cell::Floor,
                        '#' => Cell::Wall,
                        'r' => Cell::Booth(RED),
                        'y' => Cell::Booth(YELLOW),
                        'b' => Cell::Booth(BLUE),
                        '^' | '>' | 'v' | '<' => {
                            let d = Dir::parse(&ch.to_string()).unwrap();
                            if g == 0 {
                                Cell::Belt(d)
                            } else {
                                Cell::Arrow(d)
                            }
                        }
                        // Crates, bins, switches and zippers come from their own lines.
                        '@' | 'S' | 'Z' | 'A'..='R' | 'T'..='Y' => continue,
                        _ => return Err(format!("{id}: bad grid char '{ch}'")),
                    };
                    let grid = if g == 0 { &mut lv.left } else { &mut lv.right };
                    if !matches!(grid[idx(x, y)], Cell::Switch(_) | Cell::Zipper(..)) {
                        grid[idx(x, y)] = cell;
                    }
                }
            }
        }
        for (k, c) in lv.crates.iter().enumerate() {
            lv.left[idx(c.x, c.y)] = Cell::Crate(k as u8);
        }
        for (k, b) in lv.bins.iter().enumerate() {
            lv.right[idx(b.x, b.y)] = Cell::Bin(k as u8);
        }
        lv.chutes.sort();
        if lv.name.is_empty() || lv.chutes.is_empty() || lv.crates.is_empty() || lv.bins.is_empty() {
            return Err(format!("{id}: needs a name, crates, chutes and bins"));
        }
        Ok(lv)
    }
}

/// `L belt 1 1 EESSS` lays belts starting at (1,1); each letter is that tile's direction, then
/// the next tile along it. `R arrow 5 2 S`, `L switch 2 3 N`, `L zipper 4 3 E S`, `R booth-red 5 3`.
fn parse_solve(s: &str) -> Option<Vec<Placement>> {
    let w: Vec<&str> = s.split_whitespace().collect();
    let board = match *w.first()? {
        "L" => Board::Left,
        "R" => Board::Right,
        _ => return None,
    };
    let (mut x, mut y): (i32, i32) = (w.get(2)?.parse().ok()?, w.get(3)?.parse().ok()?);
    let dir = |k: usize| w.get(k).and_then(|d| Dir::parse(d));
    let one = |cell| Some(vec![Placement { board, x, y, cell }]);
    match *w.get(1)? {
        "belt" => {
            let mut out = Vec::new();
            for ch in w.get(4)?.chars() {
                let d = Dir::parse(&ch.to_string())?;
                out.push(Placement { board, x, y, cell: Cell::Belt(d) });
                x += d.dx();
                y += d.dy();
            }
            Some(out)
        }
        "arrow" => one(Cell::Arrow(dir(4)?)),
        "switch" => one(Cell::Switch(dir(4)?)),
        "zipper" => one(Cell::Zipper(dir(4)?, dir(5)?)),
        t => match Tool::parse(t)? {
            Tool::Booth(c) => one(Cell::Booth(c)),
            _ => None,
        },
    }
}
