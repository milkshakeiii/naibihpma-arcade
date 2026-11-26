//! Playfields: the hand-made "town" test map, and generated maps so the gate isn't tuned to one.

use crate::geom::*;
use crate::rng::Rng;
use std::collections::VecDeque;

/// Station shapes. A passenger wants any station of the shape they carry.
pub const SHAPES: [char; 8] = ['●', '◆', '■', '▲', '★', '✚', '♥', '♣'];

#[derive(Clone, Debug)]
pub struct StationDef {
    pub pos: Pos,
    pub shape: u8,
    /// When the station opens, in thousandths of the leg.
    pub appear: i32,
}

#[derive(Clone, Debug)]
pub struct Map {
    pub name: String,
    pub blocked: Vec<bool>,
    /// Which blocked tiles are water (the rest are buildings); only the picture differs.
    pub water: Vec<bool>,
    pub stations: Vec<StationDef>,
    /// The engine starts on `depot`, having arrived from `buffer`; both carry fixed rail.
    pub buffer: Pos,
    pub depot: Pos,
    pub terminus: Pos,
}

const TOWN: [&str; 17] = [
    ".........................",
    ".........................",
    "..a.......~~~.......b....",
    "..........~~~............",
    ".........................",
    "....##...........##......",
    "....##...........##..d...",
    ".........................",
    "<@.......................",
    ".........................",
    "......##.......~~~.......",
    "......##.......~~~...e...",
    ".........................",
    "...c.....................",
    "..................##.....",
    "...........f......##...T.",
    ".........................",
];

/// Three stations open at the start, and one more at 20%, 40% and 60% of the leg.
pub fn town() -> Map {
    parse(
        "town",
        &TOWN,
        &[
            ('a', 0, 0),
            ('b', 1, 0),
            ('c', 2, 0),
            ('d', 0, 200),
            ('e', 1, 400),
            ('f', 2, 600),
        ],
    )
}

/// Build a map from rows of `.` open ground, `~` or `#` blocked, `<` buffer, `@` depot,
/// `T` terminus, and station letters listed in `defs` as (letter, shape, appear per mille).
pub fn parse(name: &str, rows: &[&str], defs: &[(char, u8, i32)]) -> Map {
    assert_eq!(rows.len(), GH as usize, "{name}: wrong number of rows");
    let mut blocked = vec![false; NT];
    let mut water = vec![false; NT];
    let (mut buffer, mut depot, mut terminus) = (None, None, None);
    let mut found = vec![None; defs.len()];
    for (y, row) in rows.iter().enumerate() {
        let chars: Vec<char> = row.chars().collect();
        assert_eq!(
            chars.len(),
            GW as usize,
            "{name}: row {y} has the wrong width"
        );
        for (x, &c) in chars.iter().enumerate() {
            let p = Pos::new(x as i32, y as i32);
            match c {
                '.' => {}
                '~' => {
                    blocked[p.idx()] = true;
                    water[p.idx()] = true;
                }
                '#' => blocked[p.idx()] = true,
                '<' => buffer = Some(p),
                '@' => depot = Some(p),
                'T' => terminus = Some(p),
                _ => {
                    let k = defs.iter().position(|d| d.0 == c);
                    found[k.unwrap_or_else(|| panic!("{name}: unknown map letter {c}"))] = Some(p);
                }
            }
        }
    }
    let stations = defs
        .iter()
        .zip(found)
        .map(|(d, p)| StationDef {
            pos: p.expect("station letter missing from map"),
            shape: d.1,
            appear: d.2,
        })
        .collect();
    Map {
        name: name.to_string(),
        blocked,
        water,
        stations,
        buffer: buffer.expect("map needs a buffer <"),
        depot: depot.expect("map needs a depot @"),
        terminus: terminus.expect("map needs a terminus T"),
    }
}

/// A random town in the same family as `town()`: scattered blocks and ponds, six stations
/// (two of each shape, three opening late), the depot on the left and the terminus on the right.
pub fn generated(seed: u64) -> Map {
    let mut rng = Rng::new(seed.wrapping_mul(0x2545_F491_4F6C_DD1D) ^ 0x6d61_7073);
    let buffer = Pos::new(0, GH / 2);
    let depot = Pos::new(1, GH / 2);
    'attempt: loop {
        let mut blocked = vec![false; NT];
        let mut water = vec![false; NT];
        for _ in 0..rng.range(5, 9) {
            let (w, h) = if rng.chance(1, 2) { (2, 2) } else { (3, 2) };
            let wet = w == 3;
            let x = rng.range(3, GW - w - 1);
            let y = rng.range(1, GH - h - 1);
            for dy in 0..h {
                for dx in 0..w {
                    blocked[Pos::new(x + dx, y + dy).idx()] = true;
                    water[Pos::new(x + dx, y + dy).idx()] = wet;
                }
            }
        }
        for x in 0..4 {
            blocked[Pos::new(x, GH / 2).idx()] = false;
            water[Pos::new(x, GH / 2).idx()] = false;
        }
        let open_around = |p: Pos, blocked: &[bool]| {
            Dir::ALL
                .iter()
                .filter(|&&d| p.step(d).ok() && !blocked[p.step(d).idx()])
                .count()
        };
        let shapes = [0u8, 1, 2, 0, 1, 2];
        let appear = [0, 0, 0, 200, 400, 600];
        let mut stations: Vec<StationDef> = vec![];
        for _ in 0..4000 {
            if stations.len() == 6 {
                break;
            }
            let p = Pos::new(rng.range(2, GW - 3), rng.range(1, GH - 2));
            if blocked[p.idx()]
                || p.manhattan(depot) < 5
                || open_around(p, &blocked) < 3
                || stations.iter().any(|s| s.pos.manhattan(p) < 6)
            {
                continue;
            }
            let k = stations.len();
            stations.push(StationDef {
                pos: p,
                shape: shapes[k],
                appear: appear[k],
            });
        }
        if stations.len() < 6 {
            continue;
        }
        let mut terminus = None;
        for _ in 0..400 {
            let p = Pos::new(rng.range(GW - 5, GW - 1), rng.range(0, GH - 1));
            if !blocked[p.idx()]
                && open_around(p, &blocked) >= 2
                && stations.iter().all(|s| s.pos.manhattan(p) >= 3)
            {
                terminus = Some(p);
                break;
            }
        }
        let Some(terminus) = terminus else { continue };
        let reach = reachable(&blocked, depot);
        for p in stations.iter().map(|s| s.pos).chain([terminus]) {
            if !reach[p.idx()] {
                continue 'attempt;
            }
        }
        return Map {
            name: format!("gen{seed}"),
            blocked,
            water,
            stations,
            buffer,
            depot,
            terminus,
        };
    }
}

fn reachable(blocked: &[bool], from: Pos) -> Vec<bool> {
    let mut seen = vec![false; NT];
    seen[from.idx()] = true;
    let mut q = VecDeque::from([from]);
    while let Some(p) = q.pop_front() {
        for d in Dir::ALL {
            let n = p.step(d);
            if n.ok() && !blocked[n.idx()] && !seen[n.idx()] {
                seen[n.idx()] = true;
                q.push_back(n);
            }
        }
    }
    seen
}

/// Every map the gate plays: the hand-made town and `n` generated ones.
pub fn gate_maps(n: u64) -> Vec<Map> {
    std::iter::once(town())
        .chain((1..=n).map(generated))
        .collect()
}

pub fn by_name(name: &str) -> Option<Map> {
    if name == "town" {
        return Some(town());
    }
    name.strip_prefix("gen")
        .and_then(|s| s.parse().ok())
        .map(generated)
}
