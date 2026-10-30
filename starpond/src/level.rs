use crate::palette::Mood;

pub const TS: i32 = 20;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Tile {
    Empty,
    Soil,
    Bark,
    Branch,
    Lily,
    Frond,
    Reed,
}

impl Tile {
    pub fn solid(self) -> bool {
        matches!(self, Tile::Soil | Tile::Bark)
    }

    pub fn oneway(self) -> bool {
        matches!(self, Tile::Branch | Tile::Lily)
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum DewMode {
    Condense(char),
    Evaporate,
    Hold,
}

#[derive(Clone, Debug)]
pub struct Section {
    pub start_ms: u32,
    pub mood: Mood,
    pub dew: DewMode,
    pub echo_of: Option<usize>,
    pub name: String,
}

#[derive(Clone, Copy, Debug)]
pub struct Site {
    pub tx: i32,
    pub ty: i32,
    pub tag: char,
    /// Nothing at all above it in its column, so sunny sections evaporate it.
    pub exposed: bool,
    /// Hangs from a solid ceiling instead of resting on a surface.
    pub hanging: bool,
}

pub struct Level {
    pub title: String,
    pub len_ms: u32,
    pub music: Vec<String>,
    pub sections: Vec<Section>,
    pub water: Vec<(u32, i32)>, // (ms, surface y in px)
    pub cave: (i32, i32, i32, i32),
    pub w: i32,
    pub h: i32,
    tiles: Vec<Tile>,
    pub sites: Vec<Site>,
    pub start: (i32, i32),
}

pub fn parse_time(s: &str) -> Result<u32, String> {
    let (m, sec) = s.split_once(':').ok_or_else(|| format!("bad time '{s}'"))?;
    let m: u32 = m.parse().map_err(|_| format!("bad minutes in '{s}'"))?;
    let sec: u32 = sec.parse().map_err(|_| format!("bad seconds in '{s}'"))?;
    Ok((m * 60 + sec) * 1000)
}

impl Level {
    pub fn parse(src: &str) -> Result<Level, String> {
        let mut title = String::new();
        let mut len_ms = 0;
        let mut music = Vec::new();
        let mut sections = Vec::new();
        let mut echo_times: Vec<Option<u32>> = Vec::new();
        let mut water = Vec::new();
        let mut cave = (0, 0, -1, -1);
        let mut rows: Vec<&str> = Vec::new();
        let mut in_map = false;

        for line in src.lines() {
            let line = line.trim_end();
            if in_map {
                if !line.is_empty() {
                    rows.push(line);
                }
                continue;
            }
            let line = line.split('#').next().unwrap_or("").trim();
            let f: Vec<&str> = line.split_whitespace().collect();
            match f.first().copied() {
                None => {}
                Some("track") if f.len() == 3 => {
                    title = f[1].replace('_', " ");
                    len_ms = parse_time(f[2])?;
                }
                Some("music") => music = f[1..].iter().map(|s| s.to_string()).collect(),
                Some("section") if f.len() == 6 => {
                    let dew = match f[3] {
                        "evaporate" => DewMode::Evaporate,
                        "hold" => DewMode::Hold,
                        d => match d.strip_prefix("condense:").and_then(|t| t.chars().next()) {
                            Some(tag) => DewMode::Condense(tag),
                            None => return Err(format!("bad dew mode '{d}'")),
                        },
                    };
                    sections.push(Section {
                        start_ms: parse_time(f[1])?,
                        mood: Mood::parse(f[2]).ok_or_else(|| format!("unknown mood '{}'", f[2]))?,
                        dew,
                        echo_of: None,
                        name: f[5].replace('_', " "),
                    });
                    echo_times.push(if f[4] == "-" { None } else { Some(parse_time(f[4])?) });
                }
                Some("water") => {
                    for pair in f[1..].chunks(2) {
                        let row: i32 = pair.get(1).and_then(|r| r.parse().ok()).ok_or("bad water row")?;
                        water.push((parse_time(pair[0])?, row * TS));
                    }
                }
                Some("cave") if f.len() == 5 => {
                    let n: Vec<i32> = f[1..].iter().filter_map(|v| v.parse().ok()).collect();
                    if n.len() != 4 {
                        return Err("bad cave rect".into());
                    }
                    cave = (n[0], n[1], n[2], n[3]);
                }
                Some("map") => in_map = true,
                Some(other) => return Err(format!("unrecognized line starting '{other}'")),
            }
        }

        for (i, echo) in echo_times.iter().enumerate() {
            if let Some(t) = echo {
                let src = sections
                    .iter()
                    .position(|s: &Section| s.start_ms == *t)
                    .ok_or_else(|| format!("echo of {t}ms matches no section"))?;
                if src >= i {
                    return Err("an echo must follow its source section".into());
                }
                sections[i].echo_of = Some(src);
            }
        }
        if sections.is_empty() || sections[0].start_ms != 0 {
            return Err("first section must start at 0:00".into());
        }

        let h = rows.len() as i32;
        let w = rows.first().map_or(0, |r| r.len()) as i32;
        let mut tiles = Vec::with_capacity((w * h) as usize);
        let mut raw_sites = Vec::new();
        let mut start = None;
        for (ty, row) in rows.iter().enumerate() {
            if row.len() as i32 != w {
                return Err(format!("map row {ty} has {} columns, expected {w}", row.len()));
            }
            for (tx, ch) in row.chars().enumerate() {
                let (tx, ty) = (tx as i32, ty as i32);
                tiles.push(match ch {
                    '#' => Tile::Soil,
                    'B' => Tile::Bark,
                    '=' => Tile::Branch,
                    'l' => Tile::Lily,
                    'f' => Tile::Frond,
                    'w' => Tile::Reed,
                    '.' => Tile::Empty,
                    'S' => {
                        start = Some((tx, ty));
                        Tile::Empty
                    }
                    'a'..='z' => {
                        raw_sites.push((tx, ty, ch));
                        Tile::Empty
                    }
                    _ => return Err(format!("unknown map char '{ch}'")),
                });
            }
        }

        let mut level = Level {
            title,
            len_ms,
            music,
            sections,
            water,
            cave,
            w,
            h,
            tiles,
            sites: Vec::new(),
            start: start.ok_or("map has no S start tile")?,
        };
        level.sites = raw_sites
            .into_iter()
            .map(|(tx, ty, tag)| Site {
                tx,
                ty,
                tag,
                exposed: (0..ty).all(|y| level.tile(tx, y) == Tile::Empty),
                hanging: level.tile(tx, ty - 1).solid() && !level.tile(tx, ty + 1).solid(),
            })
            .collect();
        Ok(level)
    }

    /// Out-of-bounds tiles are solid so the map edges act as walls, floor and ceiling.
    pub fn tile(&self, tx: i32, ty: i32) -> Tile {
        if tx < 0 || ty < 0 || tx >= self.w || ty >= self.h {
            Tile::Soil
        } else {
            self.tiles[(ty * self.w + tx) as usize]
        }
    }

    pub fn in_cave(&self, tx: i32, ty: i32) -> bool {
        let (x0, y0, x1, y1) = self.cave;
        tx >= x0 && tx <= x1 && ty >= y0 && ty <= y1
    }

    pub fn section_end_ms(&self, i: usize) -> u32 {
        self.sections.get(i + 1).map_or(self.len_ms, |s| s.start_ms)
    }

    pub fn water_top_px(&self, ms: u32) -> i32 {
        let Some(&(_, first)) = self.water.first() else {
            return self.h * TS + TS;
        };
        let mut y = first;
        for pair in self.water.windows(2) {
            let ((t0, y0), (t1, y1)) = (pair[0], pair[1]);
            if ms >= t1 {
                y = y1;
            } else if ms >= t0 {
                let span = (t1 - t0).max(1) as i64;
                y = y0 + ((y1 - y0) as i64 * (ms - t0) as i64 / span) as i32;
                break;
            }
        }
        y
    }
}
