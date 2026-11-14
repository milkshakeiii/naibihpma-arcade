//! The proving ground: draw ships in the shipyard, set them loose in the arena, and see whether
//! they fly and fight the way their drawings suggest. Round 2: connected ships, plate, the cannon
//! and a scrolling arena. Round 4: ships up to 99x99 and a yard with tools for drawing them.

pub mod design;
mod icons;
mod keys;
pub mod library;
pub mod link;
pub mod play;
mod scene;
pub mod sim;
mod yard;

use self::design::*;
use self::library::{Browse, Library};
use self::sim::*;
use self::play::Play;
use self::yard::{Ed, NAME_RECT};
use crate::app::{App, Ev, Key};
use crate::framebuffer::{wrap, Framebuffer};
use crate::geom::Line;
use crate::palette::{self, *};
use crate::ui::{self, Rect};
use std::fs::{File, OpenOptions};
use std::io::Write;
use std::path::PathBuf;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Screen {
    Yard,
    Arena,
    Match,
}

/// A ready-made matchup: (team, slot, x, y) for each ship.
pub struct Test {
    pub name: &'static str,
    pub place: Vec<(usize, usize, i32, i32)>,
}

/// `n` ships of one design in a block centred on (cx, cy), `gap` px apart, up to 9 to a column.
pub fn block(team: usize, slot: usize, n: i32, cx: i32, cy: i32, gap: i32) -> Vec<(usize, usize, i32, i32)> {
    let rows = n.clamp(1, 9);
    let cols = (n + rows - 1) / rows;
    (0..n)
        .map(|k| {
            let (col, row) = (k / rows, k % rows);
            let in_col = if col == cols - 1 { n - col * rows } else { rows };
            let x = cx + col * gap - (cols - 1) * gap / 2;
            let y = cy + row * gap - (in_col - 1) * gap / 2;
            // Blue's first column faces the enemy; red mirrors it.
            let x = if team == 0 { 2 * cx - x } else { x };
            (team, slot, x, y)
        })
        .collect()
}

/// The ready-made fight that's made from the team as it stands: every design that flies and
/// fights, against the same again. With the same fleet on both sides, what's left to tell them
/// apart is how they're flown.
pub const TWIN: &str = "A MIXED FLEET VS ITS TWIN";

/// The designs of a team that go in a fleet: everything that flies and fights, and the unarmed
/// ships that fly with them (a projector ship).
pub fn fliers(lib: &[Design]) -> Vec<usize> {
    (0..lib.len())
        .filter(|&s| {
            let c = Card::of(&lib[s]);
            c.warship() || (c.ok && !c.seeded && c.st.thrust > 0 && c.st.projector > 0)
        })
        .collect()
}

/// One side's fleet: each design in a block of its own, two blocks abreast, with as many of
/// each as `budget` pixels buys (one at least, six at most).
pub fn fleet_side(lib: &[Design], team: usize, designs: &[usize], budget: i32) -> Vec<(usize, usize, i32, i32)> {
    let rows = (designs.len() as i32 + 1) / 2;
    let mut out = Vec::new();
    for (k, &s) in designs.iter().enumerate() {
        let n = (budget / lib[s].stats().cost.max(1)).clamp(1, 6);
        let sh = lib[s].shape();
        let (col, row) = (k as i32 % 2, k as i32 / 2);
        let cx = if team == 0 { 150 - col * 64 } else { 330 + col * 64 };
        let cy = 180 + (2 * row - (rows - 1)) * 32;
        out.extend(block(team, s, n, cx, cy, sh.w.max(sh.h) + 4));
    }
    out
}

/// The twin fight for a team: its fleet on each side, with as many of each design as the
/// dearest one's cost buys.
pub fn twin_fleet(lib: &[Design]) -> Vec<(usize, usize, i32, i32)> {
    let fleet = fliers(lib);
    let budget = fleet.iter().map(|&s| lib[s].stats().cost).max().unwrap_or(1);
    let mut place = fleet_side(lib, 0, &fleet, budget);
    place.extend(fleet_side(lib, 1, &fleet, budget));
    place
}

impl Test {
    /// What the test is, from the names of the designs in its slots: "6 FIGHTERS VS 1 FRIGATE".
    /// It stays true when a design is redrawn, renamed or swapped for another.
    pub fn title(&self, lib: &[Design]) -> String {
        if self.place.is_empty() {
            return self.name.into();
        }
        let side = |t: usize| {
            let mut counts: Vec<(usize, usize)> = Vec::new();
            for &(_, slot, _, _) in self.place.iter().filter(|p| p.0 == t) {
                match counts.iter_mut().find(|c| c.0 == slot) {
                    Some(c) => c.1 += 1,
                    None => counts.push((slot, 1)),
                }
            }
            counts.iter().map(|&(s, n)| format!("{n} {}{}", lib[s].name, if n > 1 { "S" } else { "" })).collect::<Vec<_>>().join(" + ")
        };
        format!("{} VS {}", side(0), side(1))
    }
}

fn vs(name: &'static str, blue: Vec<(usize, usize, i32, i32)>, red: Vec<(usize, usize, i32, i32)>) -> Test {
    Test { name, place: blue.into_iter().chain(red).collect() }
}

/// The arena's ready-made fights, for the starting library (the starter deck). Slots: 0 fighter,
/// 1 lancer, 2 frigate, 3 miner, 4 outpost, 5 shipyard, 6 capital. The first three are the
/// counter triangle; the last three are raids on a factory, which stands planted as it would on
/// its rock. The window names each from what's in its slots (`Test::title`).
pub fn tests() -> Vec<Test> {
    vec![
        vs("6 FIGHTERS VS 1 FRIGATE", block(0, 0, 6, 150, 180, 10), block(1, 2, 1, 330, 180, 0)),
        vs("6 LANCERS VS 1 FRIGATE", block(0, 1, 6, 150, 180, 10), block(1, 2, 1, 330, 180, 0)),
        vs("6 FIGHTERS VS 6 LANCERS", block(0, 0, 6, 150, 180, 10), block(1, 1, 6, 330, 180, 10)),
        vs("4 FIGHTERS VS 4 MINERS", block(0, 0, 4, 120, 180, 12), block(1, 3, 4, 330, 180, 14)),
        vs("1 CAPITAL VS 30 FIGHTERS", block(0, 6, 1, 130, 180, 0), block(1, 0, 30, 350, 180, 9)),
        vs("1 CAPITAL VS 40 LANCERS", block(0, 6, 1, 130, 180, 0), block(1, 1, 40, 350, 180, 9)),
        vs("12 LANCERS VS 1 OUTPOST", block(0, 1, 12, 150, 180, 10), block(1, 4, 1, 330, 180, 0)),
        vs("4 FRIGATES VS 1 OUTPOST", block(0, 2, 4, 150, 180, 16), block(1, 4, 1, 330, 180, 0)),
        vs("1 CAPITAL VS 1 SHIPYARD", block(0, 6, 1, 130, 180, 0), block(1, 5, 1, 350, 180, 0)),
        Test { name: TWIN, place: Vec::new() },
        Test { name: "OPEN FIELD", place: Vec::new() },
    ]
}

/// The fights the proving ground's rounds were judged on, for the reference fleet (`defaults()`).
/// Slots: 0 fighter, 1 lancer, 2 frigate, 3 destroyer, 4 capital, 5 gunminer, 6 titan. The
/// headless runs and the checks still fight these, so a change to the sim shows up against the
/// results the rounds recorded.
pub fn reference_tests() -> Vec<Test> {
    vec![
        vs("6 FIGHTERS VS 1 FRIGATE", block(0, 0, 6, 150, 180, 10), block(1, 2, 1, 330, 180, 0)),
        vs("6 LANCERS VS 1 FRIGATE", block(0, 1, 6, 150, 180, 10), block(1, 2, 1, 330, 180, 0)),
        vs("6 FIGHTERS VS 6 LANCERS", block(0, 0, 6, 150, 180, 10), block(1, 1, 6, 330, 180, 10)),
        vs("1 DESTROYER VS 4 FRIGATES", block(0, 3, 1, 150, 180, 0), block(1, 2, 4, 330, 180, 16)),
        vs("1 CAPITAL VS 30 FIGHTERS", block(0, 4, 1, 130, 180, 0), block(1, 0, 30, 350, 180, 9)),
        vs("1 CAPITAL VS 40 LANCERS", block(0, 4, 1, 130, 180, 0), block(1, 1, 40, 350, 180, 9)),
        vs("4 FIGHTERS VS 4 GUNMINERS", block(0, 0, 4, 120, 180, 12), block(1, 5, 4, 330, 180, 14)),
        vs("1 TITAN VS 5 CAPITALS", block(0, 6, 1, 110, 180, 0), block(1, 4, 5, 380, 180, 44)),
        vs("1 TITAN VS 40 LANCERS", block(0, 6, 1, 110, 180, 0), block(1, 1, 40, 350, 180, 9)),
        Test { name: TWIN, place: Vec::new() },
        Test { name: "OPEN FIELD", place: Vec::new() },
    ]
}

/// A library that shows the beam and the shield at work (`--parts`): the three example ships,
/// and four of the starter deck's to set them against. Slots: 0 beamer, 1 warden, 2 needle,
/// 2 raker (the repeater), 3 fighter, 4 lancer, 5 frigate, 6 outpost, 7 archer (the missile),
/// 8 aegis (the projector).
pub fn parts_library() -> Vec<Design> {
    let (deck, ex) = (starter_deck(), crate::pg::design::part_examples());
    let named = |name: &str| ex.iter().find(|d| d.name == name).unwrap_or_else(|| panic!("no example called {name}")).clone();
    // Nine slots: the examples (the small NEEDLE gave its slot to the RAKER, and is still in the
    // library), and four of the starter deck's to set them against.
    let mut lib = vec![named("BEAMER"), named("WARDEN"), named("RAKER")];
    lib.extend([0, 1, 2, 4].map(|s| deck[s].clone()));
    lib.extend([named("ARCHER"), named("AEGIS")]);
    lib
}

/// The ready-made fights for `parts_library`. The beam against what can't get out of its way,
/// what can, and what holds off at range; a shield under lances, under flak, and under a beam.
pub fn part_tests() -> Vec<Test> {
    vec![
        vs("1 BEAMER VS 1 OUTPOST", block(0, 0, 1, 110, 180, 0), block(1, 6, 1, 330, 180, 0)),
        vs("2 BEAMERS VS 6 FIGHTERS", block(0, 0, 2, 110, 180, 30), block(1, 3, 6, 330, 180, 10)),
        vs("2 BEAMERS VS 6 LANCERS", block(0, 0, 2, 110, 180, 30), block(1, 4, 6, 330, 180, 10)),
        vs("1 BEAMER VS 1 FRIGATE", block(0, 0, 1, 110, 180, 0), block(1, 5, 1, 330, 180, 0)),
        vs("6 LANCERS VS 1 WARDEN", block(0, 4, 6, 150, 180, 10), block(1, 1, 1, 330, 180, 0)),
        vs("6 FIGHTERS VS 1 WARDEN", block(0, 3, 6, 150, 180, 10), block(1, 1, 1, 330, 180, 0)),
        vs("1 FRIGATE VS 1 WARDEN", block(0, 5, 1, 150, 180, 0), block(1, 1, 1, 330, 180, 0)),
        vs("1 BEAMER VS 1 WARDEN", block(0, 0, 1, 110, 180, 0), block(1, 1, 1, 330, 180, 0)),
        // The repeater: against a shield, against plate, and beside lancers that can't crack
        // the shield alone.
        vs("2 RAKERS VS 1 WARDEN", block(0, 2, 2, 110, 180, 30), block(1, 1, 1, 330, 180, 0)),
        vs("2 RAKERS VS 1 FRIGATE", block(0, 2, 2, 110, 180, 30), block(1, 5, 1, 330, 180, 0)),
        vs("1 RAKER + 6 LANCERS VS 1 WARDEN", [block(0, 2, 1, 110, 180, 0), block(0, 4, 6, 150, 180, 10)].concat(), block(1, 1, 1, 330, 180, 0)),
        // The missile: against what can't move and has little flak, a flak screen, what outruns
        // it, and a shield.
        vs("2 ARCHERS VS 1 OUTPOST", block(0, 7, 2, 90, 180, 30), block(1, 6, 1, 330, 180, 0)),
        vs("2 ARCHERS VS 1 FRIGATE", block(0, 7, 2, 90, 180, 30), block(1, 5, 1, 330, 180, 0)),
        vs("2 ARCHERS VS 6 FIGHTERS", block(0, 7, 2, 90, 180, 30), block(1, 3, 6, 330, 180, 10)),
        vs("2 ARCHERS VS 1 WARDEN", block(0, 7, 2, 90, 180, 30), block(1, 1, 1, 330, 180, 0)),
        // The projector, beside the big slow ships it can keep up with: three fights from above
        // again with an AEGIS added (lances, which it stops one for one; flak, which empties
        // it; missiles, which it rests between), and the stream that keeps it empty.
        vs("2 BEAMERS + 1 AEGIS VS 6 LANCERS", [block(0, 0, 2, 110, 180, 30), block(0, 8, 1, 70, 180, 0)].concat(), block(1, 4, 6, 330, 180, 10)),
        vs("2 BEAMERS + 1 AEGIS VS 6 FIGHTERS", [block(0, 0, 2, 110, 180, 30), block(0, 8, 1, 70, 180, 0)].concat(), block(1, 3, 6, 330, 180, 10)),
        vs("2 ARCHERS VS 1 OUTPOST + 1 AEGIS", block(0, 7, 2, 90, 180, 30), [block(1, 6, 1, 330, 180, 0), block(1, 8, 1, 370, 180, 0)].concat()),
        vs("2 RAKERS VS 2 BEAMERS + 1 AEGIS", block(0, 2, 2, 110, 180, 30), [block(1, 0, 2, 330, 180, 30), block(1, 8, 1, 370, 180, 0)].concat()),
        Test { name: TWIN, place: Vec::new() },
        Test { name: "OPEN FIELD", place: Vec::new() },
    ]
}

#[derive(Clone, Copy, Debug)]
pub struct Placement {
    pub team: usize,
    pub slot: usize,
    pub x: i32,
    pub y: i32,
}

/// The proving ground's help, by heading. The power numbers come from the part table.
fn help_body() -> Vec<(&'static str, String)> {
    vec![
        ("SHIPS", "DRAW A SHIP IN THE YARD. EVERY COLOUR IS A PART AND UP IS THE FRONT. IT NEEDS A CORE, EVERY PIXEL JOINED EDGE TO EDGE, AND A STRAIGHT 3-WIDE WAY OUT FOR EVERY ENGINE.".into()),
        ("POWER", format!("MADE PER PIXEL: {}. EVERYTHING ELSE THAT WORKS USES IT: A PART'S HOVER IN THE YARD SAYS HOW MUCH. SHORT OF POWER, GUNS RELOAD SLOWER AND ENGINES PUSH LESS, IN STEP.", Part::power_list(true))),
        ("YARD", format!("SHIPS UP TO {GRID}X{GRID}. 1-6 PICK A PART CATEGORY, TAB ITS NEXT PART. P PENCIL, R RECTANGLE, F FILL, C STAMP, T TURN IT, U UNDO. Q/E OR THE WHEEL ZOOM.")),
        ("FLYING", "RUNS, KITE OR ORBIT: PICK IN THE YARD. LEFT ALONE, 1 THRUST PER WEIGHT RUNS AND LESS KITES. HEAVIER SHIPS TURN SLOWER.".into()),
        ("ARENA", "PICK A TEAM AND A DESIGN, CLICK TO PLACE, RIGHT-CLICK TO REMOVE. SPACE RUNS, R REPLAYS. WASD, ARROWS OR THE MINIMAP SCROLL THE VIEW.".into()),
        ("KEYS", "ENTER YARD/ARENA  L LIBRARY  N NEXT TEST  F SPEED  T TEAM  F2 SCANLINES  F1 HELP  ESC QUIT".into()),
    ]
}

/// What a side's brain button says when pointed at.
fn brain_tip(team: usize, b: Brain) -> String {
    let key = ["B", "V"][team];
    let now = match (b.targets, b.ways) {
        (false, Ways::Card) => "CARD: EACH SHIP GOES FOR THE NEAREST ENEMY AND FLIES AT IT AS ITS CARD SAYS.".to_string(),
        (targets, ways) => {
            let t = if targets { " A SHIP GOES FOR WHAT ITS DRILLS SAY IT BEATS, NOT JUST THE NEAREST." } else { "" };
            let w = match ways {
                Ways::Card => "",
                Ways::Safe => " IT FLIES AT EACH ENEMY THE WAY WHOSE WORST DRILL WAS BEST.",
                Ways::Bold => " IT FLIES AT EACH ENEMY THE WAY THAT DID BEST IN ITS DRILLS.",
                Ways::Plan => "",
            };
            if ways == Ways::Plan {
                format!("ARCH: ONE PLANNER FOR THE SIDE. IT PLAYS THE FIGHT FORWARD {} S FOR EACH WAY EACH DESIGN COULD FLY AND KEEPS WHAT CAME OUT BEST, AND THINKS AGAIN EVERY {} S.", ARCH_AHEAD / 15, ARCH_EVERY / 15)
            } else {
                format!("DRILLED: BEFORE THE FIGHT EVERY DESIGN FIGHTS EVERY ENEMY DESIGN, EACH WAY IT CAN FLY.{t}{w}")
            }
        }
    };
    format!("[{key}] HOW {}'S SHIPS CHOOSE. {now} CLICK FOR THE NEXT; THE FIGHT STARTS AGAIN.", team_name(team))
}

/// The inspector's line for what a ship learned in its drills about the enemy it's after.
fn learned_line(foe: &str, k: Drill, b: Brain) -> String {
    let way = match k.style(b.ways) {
        Some(s) => format!("{}, ", s.name()),
        None => String::new(),
    };
    let m = k.margin(b.ways) / 10;
    format!("DRILLS V {foe}: {way}{}", if m > 0 { format!("WON BY {m}%") } else if m < 0 { format!("LOST BY {}%", -m) } else { "EVEN".to_string() })
}

/// A notice over the arena, wrapped to fit inside the field with room for its panel.
fn notice_lines(s: &str) -> Vec<String> {
    wrap(s, ((FIELD.w - 8 - 12) / 6) as usize)
}

// Arena layout: the field is a 304x300 window onto the 480x360 arena.
const FIELD: Rect = Rect::new(0, 0, 304, 300);
const BTN_TEAM: [Rect; 2] = [Rect::new(306, 4, 44, 16), Rect::new(352, 4, 44, 16)];
const BTN_RUN: Rect = Rect::new(306, 149, 90, 15);
const BTN_RESET: Rect = Rect::new(306, 165, 44, 15);
const BTN_SPEED: Rect = Rect::new(352, 165, 44, 15);
const BTN_TEST: Rect = Rect::new(306, 181, 90, 15);
const BTN_EMPTY: Rect = Rect::new(306, 197, 44, 15);
const BTN_YARD: Rect = Rect::new(352, 197, 44, 15);
/// Each side's brain, on the field's top edge under the fight's name.
const BTN_BRAIN: [Rect; 2] = [Rect::new(2, 13, 88, 11), Rect::new(92, 13, 88, 11)];
/// The minimap: the whole arena at exactly 1/6 scale.
const MINI_SCALE: i32 = 6;
const MINI: Rect = Rect::new(310, 213, AW / MINI_SCALE + 2, AH / MINI_SCALE + 2);
const CAM_STEP: i32 = 8;
const HELP_RECT: Rect = Rect::new(10, 8, 380, 284);
const QUIT_RECT: Rect = Rect::new(112, 108, 176, 70);
const QUIT_BTNS: [Rect; 2] = [Rect::new(124, 150, 72, 18), Rect::new(204, 150, 72, 18)];

/// A design's row in the arena's roster and on the match's cards is one line, so a team of nine
/// fits where seven rows of two lines did: this high, with this much of its name.
pub const ROW_H: i32 = 14;
pub const ROW_NAME: usize = 7;

fn roster_rect(i: usize) -> Rect {
    Rect::new(306, 22 + i as i32 * ROW_H, 90, ROW_H - 1)
}

pub fn team_name(t: usize) -> &'static str {
    if t == 0 {
        "BLUE"
    } else {
        "RED"
    }
}

pub fn part_col(p: Part, team: usize) -> u8 {
    p.info().colour.unwrap_or(team_col(team, 1))
}

/// Which part shows when a downscaled icon has to pick one pixel for a block of cells.
fn icon_rank(p: Part) -> i32 {
    p.info().rank
}

#[inline]
fn pc(fb: &mut Framebuffer, clip: Rect, x: i32, y: i32, c: u8) {
    if clip.contains(x, y) {
        fb.pset(x, y, c);
    }
}

/// A Bresenham line clipped to a rectangle.
fn clipped_line(fb: &mut Framebuffer, clip: Rect, x0: i32, y0: i32, x1: i32, y1: i32, c: u8, skip_first: bool) {
    let mut l = Line::new(x0, y0, x1, y1);
    let mut first = true;
    loop {
        if !(first && skip_first) {
            pc(fb, clip, l.x, l.y, c);
        }
        first = false;
        if (l.x, l.y) == (x1, y1) {
            break;
        }
        l.step();
    }
}

/// Draws a shape centred on (x, y): a 1px dark hull around the solid cells, the hull lit along
/// its top edge, each cannon's breech in gold, and checkerboard ghost dots where `ghost` says so.
pub fn draw_shape(fb: &mut Framebuffer, clip: Rect, sh: &Shape, x: i32, y: i32, facing: u8, team: usize, solid: &dyn Fn(usize) -> bool, ghost: bool) {
    let (w, h) = sh.dims(facing);
    let (l, t) = (x - w / 2, y - h / 2);
    let mut grid = vec![false; (w * h) as usize];
    // Where its projector pixels are: a lump of them is drawn with a pale middle.
    let mut knob = vec![false; (w * h) as usize];
    for i in 0..sh.cells.len() {
        if sh.cells[i].is_some() && solid(i) {
            let (px, py) = sh.cell_xy(x, y, facing, i);
            grid[((py - t) * w + (px - l)) as usize] = true;
            knob[((py - t) * w + (px - l)) as usize] = sh.cells[i] == Some(Part::Projector);
        }
    }
    let in_knob = |gx: i32, gy: i32| gx >= 0 && gy >= 0 && gx < w && gy < h && knob[(gy * w + gx) as usize];
    if !ghost {
        for gy in 0..h {
            for gx in 0..w {
                if grid[(gy * w + gx) as usize] {
                    for (dx, dy) in [(1, 0), (-1, 0), (0, 1), (0, -1)] {
                        pc(fb, clip, l + gx + dx, t + gy + dy, OUTLINE);
                    }
                }
            }
        }
    }
    for i in 0..sh.cells.len() {
        let Some(p) = sh.cells[i] else { continue };
        if !solid(i) {
            continue;
        }
        let (px, py) = sh.cell_xy(x, y, facing, i);
        if ghost {
            if (px + py) & 1 == 0 {
                pc(fb, clip, px, py, team_col(team, 2));
            }
            continue;
        }
        let open_above = py - t == 0 || !grid[((py - t - 1) * w + (px - l)) as usize];
        let col = if p == Part::Hull && open_above {
            team_col(team, 2)
        } else if p == Part::Cannon && sh.cannons.contains(&i) {
            GOLD
        } else if p == Part::Projector && [(1, 0), (-1, 0), (0, 1), (0, -1)].iter().all(|&(dx, dy)| in_knob(px - l + dx, py - t + dy)) {
            // Projector on all four sides of it: the lump's pale middle.
            PROJECTOR_HI
        } else {
            part_col(p, team)
        };
        pc(fb, clip, px, py, col);
    }
}

/// A design's icon in a box: drawn 1:1 when it fits, otherwise at an exact 1/k scale where each
/// icon pixel shows the most telling part in its k x k block.
fn draw_icon(fb: &mut Framebuffer, d: &Design, r: Rect, team: usize) {
    fb.rect_fill(r.x, r.y, r.w, r.h, SPACE);
    let sh = d.shape();
    if sh.w <= r.w - 2 && sh.h <= r.h - 2 {
        draw_shape(fb, r, &sh, r.x + r.w / 2, r.y + r.h / 2, 0, team, &|_| true, false);
        return;
    }
    let k = ((sh.w + r.w - 3) / (r.w - 2)).max((sh.h + r.h - 3) / (r.h - 2)).max(1);
    let (iw, ih) = ((sh.w + k - 1) / k, (sh.h + k - 1) / k);
    let (ox, oy) = (r.x + (r.w - iw) / 2, r.y + (r.h - ih) / 2);
    for iy in 0..ih {
        for ix in 0..iw {
            let best = (0..k * k)
                .filter_map(|j| {
                    let (cx, cy) = (ix * k + j % k, iy * k + j / k);
                    if cx < sh.w && cy < sh.h {
                        sh.cells[(cy * sh.w + cx) as usize]
                    } else {
                        None
                    }
                })
                .max_by_key(|&p| icon_rank(p));
            if let Some(p) = best {
                pc(fb, r, ox + ix, oy + iy, if p == Part::Cannon { GOLD } else { part_col(p, team) });
            }
        }
    }
}

pub struct Pg {
    pub lib: Vec<Design>,
    pub screen: Screen,
    pub slot: usize,
    pub ed: Ed,
    edited: bool,
    naming: bool,
    pub arena: Arena,
    pub setup: Vec<Placement>,
    /// How each side's ships choose what to go for and how to fly at it, in the arena.
    pub brains: [Brain; 2],
    /// Frames still to draw before the window stops to run the drills a fight needs (0: none
    /// are owed).
    drill_wait: u8,
    /// What pairs of drawings learned of each other, by their saved lines, so a pair is only
    /// drilled once however often its fight is replayed.
    drill_cache: std::collections::HashMap<(String, String), (Drill, Drill)>,
    pub test: Option<usize>,
    tests: Vec<Test>,
    pub running: bool,
    pub speed: u32,
    pub team: usize,
    pub mouse: (i32, i32),
    /// Top-left of the field's window onto the arena.
    pub cam: (i32, i32),
    held: [bool; 4],
    latch: [bool; 4],
    /// Ctrl is held: in a match, a seed goes to open space.
    pub ctrl: bool,
    /// Shift is held: in a match, a click or a box adds to what's selected.
    pub shift: bool,
    /// Frames drawn so far: what a double click and a key pressed twice are timed by.
    pub frames: u32,
    map_drag: bool,
    pub help: bool,
    /// The keys page is up (F3): every key this screen takes.
    pub keys: bool,
    confirm_quit: bool,
    crt: bool,
    quit: bool,
    pal: Pal,
    save_path: Option<PathBuf>,
    log: Option<File>,
    logged_run: bool,
    logged_over: bool,
    notice: Option<(String, u32)>,
    /// The "play a match" panel over the yard.
    pub menu: bool,
    /// Finding another machine to play against, before the match.
    pub lobby: Option<link::Lobby>,
    pub play: Option<Play>,
    /// Every drawing and team you've kept, and the ready-made ones.
    pub library: Library,
    /// The library, when it's open over the yard.
    pub browse: Option<Browse>,
}

impl Pg {
    /// The game as it starts: the starter deck in your first seven slots (so there's a working match
    /// deck to redraw from the first minute), unless a save says otherwise.
    pub fn new(save_path: Option<PathBuf>, log_path: Option<PathBuf>) -> Pg {
        Pg::with(starter_deck(), tests(), save_path, log_path)
    }

    /// The proving ground as its rounds left it: the reference fleet and the fights it was judged
    /// on. Nothing is saved or logged.
    pub fn reference() -> Pg {
        Pg::with(defaults(), reference_tests(), None, None)
    }

    /// The beam and the shield at work: three example ships and the fights that show what the
    /// parts do. Nothing is saved or logged, so the examples can be redrawn freely.
    pub fn parts() -> Pg {
        Pg::with(parts_library(), part_tests(), None, None)
    }

    fn with(lib: Vec<Design>, tests: Vec<Test>, save_path: Option<PathBuf>, log_path: Option<PathBuf>) -> Pg {
        // A team is nine slots: what it starts with, and empty ones after. A save from when a
        // team was seven fills the first seven.
        let mut lib = team_of(lib);
        if let Some(p) = &save_path {
            if let Ok(text) = std::fs::read_to_string(p) {
                if matches!(text.lines().next(), Some(SAVE_HEADER | OLD_SAVE_HEADER)) {
                    for (i, d) in text.lines().skip(1).filter_map(Design::from_line).take(lib.len()).enumerate() {
                        lib[i] = d;
                    }
                    // A design still exactly like an old default was never edited: give it the new one.
                    let now = defaults();
                    for (slot, old) in retired_defaults() {
                        if lib[slot].to_line() == old.to_line() {
                            lib[slot] = now[slot].clone();
                        }
                    }
                    // The same for a starter-deck card as an earlier round drew it.
                    let now = starter_deck();
                    for (slot, old) in retired_starters() {
                        if lib[slot].to_line() == old.to_line() {
                            lib[slot] = now[slot].clone();
                        }
                    }
                } else if !text.trim().is_empty() {
                    // Round-1 designs use the old 15x15 grid and rules: keep them, start fresh.
                    let backup = p.with_file_name("proving_ground_designs_round1.txt");
                    if !backup.exists() {
                        let _ = std::fs::write(&backup, &text);
                    }
                }
            }
        }
        let log = log_path.and_then(|p| OpenOptions::new().create(true).append(true).open(p).ok());
        // Your library lives beside your team's save.
        let library = Library::load(save_path.as_ref().map(|p| p.with_file_name("design_library.txt")));
        let mut pg = Pg {
            lib,
            screen: Screen::Arena,
            slot: 0,
            ed: Ed::new(),
            edited: false,
            naming: false,
            arena: Arena::new(),
            setup: Vec::new(),
            brains: [Brain::CARD; 2],
            drill_wait: 0,
            drill_cache: Default::default(),
            test: None,
            tests,
            running: false,
            speed: 1,
            team: 0,
            mouse: (0, 0),
            cam: ((AW - FIELD.w) / 2, (AH - FIELD.h) / 2),
            held: [false; 4],
            latch: [false; 4],
            ctrl: false,
            shift: false,
            frames: 0,
            map_drag: false,
            help: true,
            keys: false,
            confirm_quit: false,
            crt: true,
            quit: false,
            pal: palette::game(),
            save_path,
            log,
            logged_run: false,
            logged_over: false,
            notice: None,
            menu: false,
            lobby: None,
            play: None,
            library,
            browse: None,
        };
        pg.log(&format!("##### proving ground round 5 session {} #####", crate::game::utc_stamp()));
        pg.load_test(0);
        pg.fit_view();
        pg
    }

    fn log(&mut self, s: &str) {
        if let Some(f) = self.log.as_mut() {
            let _ = writeln!(f, "{s}");
        }
    }

    fn save(&mut self) {
        if let Some(p) = &self.save_path {
            let text: String = std::iter::once(SAVE_HEADER.to_string() + "\n").chain(self.lib.iter().map(|d| d.to_line() + "\n")).collect();
            let _ = std::fs::write(p, text);
        }
    }

    fn say(&mut self, s: &str) {
        self.notice = Some((s.to_string(), 30));
    }

    pub fn load_test(&mut self, t: usize) {
        self.test = Some(t);
        // The twin fight is made from the team as it stands now.
        let place = if self.tests[t].name == TWIN { twin_fleet(&self.lib) } else { self.tests[t].place.clone() };
        self.setup = place.iter().map(|&(team, slot, x, y)| Placement { team, slot, x, y }).collect();
        self.reset();
    }

    /// Loads the ready-made fight of that name, if there is one.
    pub fn load_named(&mut self, name: &str) -> bool {
        match self.tests.iter().position(|t| t.name == name) {
            Some(t) => {
                self.load_test(t);
                true
            }
            None => false,
        }
    }

    /// Rebuilds the arena from the setup with the current designs. The same setup always replays
    /// the same fight.
    pub fn reset(&mut self) {
        self.arena = Arena::new();
        let mut refused = None;
        for p in &self.setup {
            if !self.arena.add(p.team, p.slot, &self.lib[p.slot], p.x, p.y) {
                refused = Some(p.slot);
            }
        }
        if let Some(s) = refused {
            let why = self.lib[s].check().problem().unwrap_or_default();
            self.say(&format!("{} CAN'T FLY: {why}", self.lib[s].name));
        }
        self.running = false;
        self.logged_run = false;
        self.logged_over = false;
        // A side that goes by its drills needs them run on the designs it faces. Ones already
        // run on these drawings are to hand; otherwise the window says so for a frame, then
        // stops to run them.
        self.arena.brain = self.brains;
        self.drill_wait = 0;
        if self.brains.iter().any(|b| b.targets || matches!(b.ways, Ways::Safe | Ways::Bold)) {
            if self.drill_pairs().iter().all(|&k| self.drill_cache.contains_key(&self.drill_key(k))) {
                self.learn();
            } else {
                self.drill_wait = 2;
                self.say("RUNNING DRILLS...");
            }
        }
    }

    /// The pairs of designs that face each other in the setup, lower slot first.
    fn drill_pairs(&self) -> Vec<(usize, usize)> {
        let mut out = Vec::new();
        for a in self.setup.iter().filter(|p| p.team == 0) {
            for b in self.setup.iter().filter(|p| p.team == 1) {
                let k = (a.slot.min(b.slot), a.slot.max(b.slot));
                if !out.contains(&k) {
                    out.push(k);
                }
            }
        }
        out
    }

    fn drill_key(&self, (a, b): (usize, usize)) -> (String, String) {
        (self.lib[a].to_line(), self.lib[b].to_line())
    }

    /// Gives the arena what the designs facing each other learned in their drills, running any
    /// that haven't been run on these drawings yet.
    fn learn(&mut self) {
        let (mut table, mut ran, t0) = (Drills::default(), 0, std::time::Instant::now());
        for (a, b) in self.drill_pairs() {
            let (da, db) = (&self.lib[a], &self.lib[b]);
            if !da.check().ok() || !db.check().ok() || !(da.stats().armed() || db.stats().armed()) {
                continue;
            }
            let key = self.drill_key((a, b));
            let (x, y) = match self.drill_cache.get(&key) {
                Some(&v) => v,
                None => {
                    ran += 1;
                    let v = Drills::pair(da, db);
                    self.drill_cache.insert(key, v);
                    v
                }
            };
            table.set(a, b, x);
            if a != b {
                table.set(b, a, y);
            }
        }
        self.arena.drills = std::sync::Arc::new(table);
        if ran > 0 {
            let ms = t0.elapsed().as_millis();
            let line = format!("{ran} {} DRILLED IN {}.{} S", if ran == 1 { "PAIR" } else { "PAIRS" }, ms / 1000, ms % 1000 / 100);
            self.log(&format!("DRILLS {line}"));
            self.say(&line);
        }
    }

    /// The next brain for a side. The fight starts again, so what's watched is one brain's fight
    /// from the first tick.
    fn next_brain(&mut self, team: usize) {
        let k = BRAINS.iter().position(|b| *b == self.brains[team]).map_or(0, |k| (k + 1) % BRAINS.len());
        self.brains[team] = BRAINS[k];
        self.reset();
    }

    fn test_name(&self) -> String {
        self.test.map_or("YOUR SETUP".into(), |t| self.tests[t].title(&self.lib))
    }

    fn set_running(&mut self, on: bool) {
        if on && self.arena.over.is_some() {
            self.reset();
        }
        self.running = on;
        if on && !self.logged_run {
            self.logged_run = true;
            let side = |t: usize| {
                let mut counts: Vec<(usize, usize)> = Vec::new();
                for p in self.setup.iter().filter(|p| p.team == t) {
                    match counts.iter_mut().find(|(s, _)| *s == p.slot) {
                        Some(c) => c.1 += 1,
                        None => counts.push((p.slot, 1)),
                    }
                }
                counts.iter().map(|(s, n)| format!("{n} {}", self.lib[*s].name)).collect::<Vec<_>>().join(", ")
            };
            let brains = if self.brains == [Brain::CARD; 2] { String::new() } else { format!(" | BRAINS {} V {}", self.brains[0].name(), self.brains[1].name()) };
            let line = format!("RUN {}: BLUE {} | RED {}{brains}", self.test_name(), side(0), side(1));
            self.log(&line);
            let mut slots: Vec<usize> = self.setup.iter().map(|p| p.slot).collect();
            slots.sort();
            slots.dedup();
            for s in slots {
                let line = format!("  DESIGN {}", self.lib[s].to_line());
                self.log(&line);
            }
        }
    }

    // ----- Input -----

    fn key(&mut self, k: Key) {
        if self.naming {
            match k {
                Key::Back => {
                    self.lib[self.slot].name.pop();
                }
                Key::Enter | Key::Esc | Key::Tab => self.end_naming(),
                _ => {}
            }
            return;
        }
        // Any key puts the keys page away, and the rules. From the rules, F3 goes on to the keys.
        if self.keys {
            self.keys = false;
            return;
        }
        if self.help {
            self.help = false;
            self.keys = k == Key::Keys;
            return;
        }
        if self.confirm_quit {
            if k == Key::Esc || k == Key::Enter {
                self.quit = true;
            }
            self.confirm_quit = false;
            return;
        }
        if library::open(self) {
            match k {
                Key::Help => self.help = true,
                Key::Keys => self.keys = true,
                k => self.key_library(k),
            }
            return;
        }
        if self.key_lobby(k) {
            return;
        }
        match (self.screen, k) {
            (Screen::Match, Key::Esc) if self.cancel_planting() => {}
            (_, Key::Esc) => self.confirm_quit = true,
            (_, Key::Help) => self.help = true,
            (_, Key::Keys) => self.keys = true,
            (Screen::Match, k) => self.key_match(k),
            (Screen::Arena, Key::Tab) | (_, Key::Enter) => self.switch_screen(),
            (Screen::Yard, k) => self.key_yard(k),
            (Screen::Arena, Key::Num(n)) if (1..=self.lib.len()).contains(&(n as usize)) => self.slot = n as usize - 1,
            (Screen::Arena, Key::Space) => self.set_running(!self.running),
            (Screen::Arena, Key::Letter('R')) => self.reset(),
            (Screen::Arena, Key::Letter('F')) => self.cycle_speed(),
            (Screen::Arena, Key::Letter('N')) => self.next_test(),
            (Screen::Arena, Key::Letter('T')) => self.team = 1 - self.team,
            (Screen::Arena, Key::Letter('B')) => self.next_brain(0),
            (Screen::Arena, Key::Letter('V')) => self.next_brain(1),
            _ => {}
        }
    }

    fn cycle_speed(&mut self) {
        self.speed = match self.speed {
            1 => 2,
            2 => 4,
            _ => 1,
        };
    }

    fn next_test(&mut self) {
        let t = self.test.map_or(0, |t| (t + 1) % self.tests.len());
        self.load_test(t);
    }

    fn switch_screen(&mut self) {
        self.end_naming();
        self.screen = match self.screen {
            Screen::Yard => {
                if self.edited {
                    self.edited = false;
                    self.save();
                    let line = format!("EDITED {}", self.lib[self.slot].to_line());
                    self.log(&line);
                }
                // The arena replays with the new designs.
                self.reset();
                Screen::Arena
            }
            Screen::Arena => {
                self.running = false;
                self.frame_if_new();
                Screen::Yard
            }
            Screen::Match => Screen::Match,
        };
    }

    fn end_naming(&mut self) {
        if self.naming {
            self.naming = false;
            if self.lib[self.slot].name.trim().is_empty() {
                self.lib[self.slot].name = format!("DESIGN {}", self.slot + 1);
            }
            self.edited = true;
            self.save();
        }
    }

    fn to_world(&self, x: i32, y: i32) -> (i32, i32) {
        (x + self.cam.0, y + self.cam.1)
    }

    fn set_cam(&mut self, x: i32, y: i32) {
        self.cam = (x.clamp(0, AW - FIELD.w), y.clamp(0, AH - FIELD.h));
    }

    /// Centres the view on the arena point under a minimap click.
    fn cam_from_map(&mut self, x: i32, y: i32) {
        let (wx, wy) = ((x - MINI.x - 1) * MINI_SCALE, (y - MINI.y - 1) * MINI_SCALE);
        self.set_cam(wx - FIELD.w / 2, wy - FIELD.h / 2);
    }

    fn press(&mut self, x: i32, y: i32, right: bool) {
        if self.keys {
            self.keys = false;
            return;
        }
        if self.help {
            // A click puts the rules away. On their KEYS button it goes on to the keys page.
            self.help = false;
            self.keys = !right && keys::BTN_KEYS.contains(x, y);
            return;
        }
        if self.confirm_quit {
            if QUIT_BTNS[0].contains(x, y) {
                self.quit = true;
            }
            self.confirm_quit = false;
            return;
        }
        if library::open(self) {
            self.press_library(x, y, right);
            return;
        }
        if self.naming && !NAME_RECT.contains(x, y) {
            self.end_naming();
        }
        if self.lobby.is_some() {
            self.press_lobby(x, y);
            return;
        }
        if self.menu {
            self.press_menu(x, y);
            return;
        }
        match self.screen {
            Screen::Yard => self.press_yard(x, y, right),
            Screen::Arena => self.press_arena(x, y, right),
            Screen::Match => self.press_match(x, y, right),
        }
    }

    fn press_arena(&mut self, x: i32, y: i32, right: bool) {
        if let Some(t) = (0..2).find(|&t| BTN_BRAIN[t].contains(x, y)) {
            if !right {
                self.next_brain(t);
            }
            return;
        }
        if FIELD.contains(x, y) {
            if self.arena.tick > 0 {
                self.say("PRESS R TO RESET, THEN PLACE");
                return;
            }
            let (wx, wy) = self.to_world(x, y);
            if right {
                if let Some(i) = self.arena.ship_at(wx, wy) {
                    let (team, slot, (hx, hy)) = (self.arena.ships[i].team, self.arena.ships[i].slot, self.arena.ships[i].home);
                    let found = (0..self.setup.len())
                        .filter(|&k| self.setup[k].team == team && self.setup[k].slot == slot)
                        .min_by_key(|&k| crate::geom::dist2(self.setup[k].x, self.setup[k].y, hx, hy));
                    if let Some(k) = found {
                        self.setup.remove(k);
                        self.test = None;
                        self.reset();
                    }
                }
                return;
            }
            if let Some(why) = self.lib[self.slot].check().problem() {
                let s = format!("{} CAN'T FLY: {why}", self.lib[self.slot].name);
                self.say(&s);
                return;
            }
            self.setup.push(Placement { team: self.team, slot: self.slot, x: wx, y: wy });
            self.test = None;
            self.reset();
            return;
        }
        if right {
            return;
        }
        if MINI.contains(x, y) {
            self.map_drag = true;
            self.cam_from_map(x, y);
        } else if let Some(t) = (0..2).find(|&t| BTN_TEAM[t].contains(x, y)) {
            self.team = t;
        } else if let Some(i) = (0..self.lib.len()).find(|&i| roster_rect(i).contains(x, y)) {
            self.slot = i;
        } else if BTN_RUN.contains(x, y) {
            self.set_running(!self.running);
        } else if BTN_RESET.contains(x, y) {
            self.reset();
        } else if BTN_SPEED.contains(x, y) {
            self.cycle_speed();
        } else if BTN_TEST.contains(x, y) {
            self.next_test();
        } else if BTN_EMPTY.contains(x, y) {
            self.setup.clear();
            self.test = None;
            self.reset();
        } else if BTN_YARD.contains(x, y) {
            self.switch_screen();
        }
    }

    // ----- Tick -----

    fn step(&mut self) {
        self.frames += 1;
        if let Some((_, t)) = self.notice.as_mut() {
            *t = t.saturating_sub(1);
            if *t == 0 {
                self.notice = None;
            }
        }
        // Scroll the view with WASD or the arrows; a tap between ticks still counts.
        let dirs: Vec<bool> = (0..4).map(|k| self.held[k] || self.latch[k]).collect();
        self.latch = self.held;
        let panel = self.help || self.keys;
        if self.screen == Screen::Arena && !panel && !self.confirm_quit {
            let dx = (dirs[3] as i32 - dirs[2] as i32) * CAM_STEP;
            let dy = (dirs[1] as i32 - dirs[0] as i32) * CAM_STEP;
            if dx != 0 || dy != 0 {
                self.set_cam(self.cam.0 + dx, self.cam.1 + dy);
            }
        }
        if library::open(self) {
            if !panel && !self.confirm_quit {
                self.nav_library(&dirs);
            }
        } else if self.screen == Screen::Yard && !panel && !self.confirm_quit && !self.naming && !self.menu {
            self.scroll_yard(&dirs);
        }
        self.step_lobby();
        // A match against the AI stops while a panel is open over it. One against another
        // machine can't: their clock is running, so it plays on underneath with the king's keys
        // let go.
        let free = !panel && !self.confirm_quit;
        let net = self.play.as_ref().is_some_and(|p| p.link.net.is_some());
        if self.screen == Screen::Match && (free || net) {
            self.step_match(if free { &dirs } else { &[false; 4] });
        }
        // Drills owed: the notice has had its frame, so run them now. The fight waits.
        if self.screen == Screen::Arena && self.drill_wait > 0 {
            self.drill_wait -= 1;
            if self.drill_wait == 0 {
                self.learn();
            }
            return;
        }
        if self.screen != Screen::Arena || !self.running || panel || self.confirm_quit {
            return;
        }
        for _ in 0..self.speed {
            self.arena.step();
        }
        if let Some(result) = self.arena.over {
            self.running = false;
            if !self.logged_over {
                self.logged_over = true;
                let t = self.arena.tick / 15;
                let who = match result {
                    Some(team) => format!("{} WINS", team_name(team)),
                    None => "DRAW".into(),
                };
                let who = if self.arena.stalemate { "DRAW (STALEMATE)".to_string() } else { who };
                let s = &self.arena.score;
                let a = &self.arena;
                let side = |k: usize| {
                    format!(
                        "lost {}/{} ships ({} split off), {} px ({} broke away), {} bounced off plate{}, mined {}{}",
                        s[k].lost,
                        s[k].placed,
                        s[k].split,
                        s[k].px_lost,
                        s[k].broken,
                        s[1 - k].bounced,
                        format!("{}{}", if s[1 - k].shielded > 0 { format!(", {} stopped at its shields", s[1 - k].shielded) } else { String::new() }, if s[k].downed > 0 { format!(", shot down {} missiles", s[k].downed) } else { String::new() }),
                        s[k].mined,
                        if a.disarmed(k) { ", DISARMED" } else { "" }
                    )
                };
                let line = format!("RESULT {who} at {}:{:02} | blue {} | red {}", t / 60, t % 60, side(0), side(1));
                self.log(&line);
            }
        }
    }

    // ----- Drawing -----

    fn render(&self, fb: &mut Framebuffer) {
        match self.screen {
            Screen::Yard => self.draw_yard(fb),
            Screen::Arena => self.draw_arena(fb),
            Screen::Match => self.draw_match(fb),
        }
        if library::open(self) {
            self.draw_library(fb);
        }
        if self.menu && self.screen == Screen::Yard {
            self.draw_menu(fb);
        }
        self.draw_lobby(fb);
        if let Some((s, _)) = &self.notice {
            let lines = notice_lines(s);
            let w = lines.iter().map(|l| Framebuffer::text_width(l, 1)).max().unwrap_or(0) + 12;
            let r = Rect::new((FIELD.w - w).max(4) / 2, 22, w, 6 + 9 * lines.len() as i32);
            ui::panel(fb, r);
            for (i, l) in lines.iter().enumerate() {
                fb.text(l, r.x + 6, r.y + 4 + 9 * i as i32, GOLD, None, 1);
            }
        }
        if self.keys {
            self.draw_keys(fb);
        } else if self.help && self.screen == Screen::Match {
            self.draw_match_help(fb);
        } else if self.help {
            self.draw_help(fb);
        } else if self.confirm_quit {
            ui::panel(fb, QUIT_RECT);
            fb.text_centered("QUIT THE PROVING GROUND?", QUIT_RECT.x + QUIT_RECT.w / 2, QUIT_RECT.y + 12, TEXT, None, 1);
            fb.text_centered("ESC AGAIN QUITS", QUIT_RECT.x + QUIT_RECT.w / 2, QUIT_RECT.y + 26, TEXT_FAINT, None, 1);
            for (r, l) in QUIT_BTNS.iter().zip(["QUIT", "BACK"]) {
                ui::button(fb, *r, l, r.contains(self.mouse.0, self.mouse.1), false);
            }
        }
    }

    fn draw_arena(&self, fb: &mut Framebuffer) {
        let a = &self.arena;
        let m = self.mouse;
        let (cx, cy) = self.cam;
        let f = FIELD;
        scene::backdrop(fb, f, a, self.cam);
        for (t, &(sx, sy)) in STATIONS.iter().enumerate() {
            let (sx, sy) = (sx - cx, sy - cy);
            for (x, y, w, h, c) in [(sx - 4, sy - 13, 9, 27, OUTLINE), (sx - 3, sy - 12, 7, 25, team_col(t, 0))] {
                for k in 0..w * h {
                    pc(fb, f, x + k % w, y + k / w, c);
                }
            }
            for k in 0..5 {
                for j in 0..10 {
                    pc(fb, f, sx - 2 + j % 5, sy - 10 + k * 5 + j / 5, team_col(t, 2));
                }
            }
        }
        scene::drift(fb, f, a, self.cam);
        let (wx, wy) = self.to_world(m.0, m.1);
        let on_brain = (0..2).find(|&t| BTN_BRAIN[t].contains(m.0, m.1));
        let hovered = if f.contains(m.0, m.1) && on_brain.is_none() { a.ship_at(wx, wy) } else { None };
        if let Some(i) = hovered {
            let s = &a.ships[i];
            if let Some(t) = s.target.filter(|&t| a.ships[t].alive) {
                crate::render::dotted(fb, f, s.x - cx, s.y - cy, a.ships[t].x - cx, a.ships[t].y - cy, 3, TEXT_FAINT);
            }
            if let Some((gx, gy)) = s.goal {
                for k in -2..=2 {
                    pc(fb, f, gx - cx + k, gy - cy, TEXT_DIM);
                    pc(fb, f, gx - cx, gy - cy + k, TEXT_DIM);
                }
            }
        }
        for s in a.ships.iter().filter(|s| s.alive) {
            scene::ship(fb, f, s, self.cam);
        }
        if let Some(i) = hovered {
            scene::corners(fb, &a.ships[i], self.cam);
        }
        scene::fire(fb, f, a, self.cam);
        // Where a click would place the selected design.
        if a.tick == 0 && hovered.is_none() && on_brain.is_none() && f.contains(m.0, m.1) && !self.help {
            let sh = self.lib[self.slot].shape();
            draw_shape(fb, f, &sh, m.0, m.1, if self.team == 0 { 1 } else { 3 }, self.team, &|_| true, true);
        }
        fb.text(&self.test_name(), 4, 3, TEXT_DIM, Some(OUTLINE), 1);
        let t = a.tick / 15;
        let clock = format!("{}:{:02}", t / 60, t % 60);
        fb.text(&clock, f.w - 4 - Framebuffer::text_width(&clock, 1), 3, TEXT_DIM, Some(OUTLINE), 1);
        for t in 0..2 {
            ui::button(fb, BTN_BRAIN[t], &format!("{} {}", team_name(t), self.brains[t].short()), on_brain == Some(t), self.brains[t] != Brain::CARD);
        }

        // Inspector.
        let bar: Option<Vec<String>> = if let Some(t) = on_brain {
            Some(wrap(&brain_tip(t, self.brains[t]), 49))
        } else if let Some(i) = hovered {
            let s = &a.ships[i];
            let mut parts = vec![format!("{} PX/S", s.speed_px_s()), format!("{}/{} PX", s.px_now(), s.px_full())];
            if s.st.power_pct() < 100 {
                parts.push(format!("POWER {}%", s.st.power_pct()));
            }
            for (n, name) in [(s.st.flak, "FLAK"), (s.st.lance, "LANCE"), (s.st.cannon, "CANNON"), (s.st.beam, "BEAM"), (s.st.missile, "MISSILE"), (s.st.repeater, "REPEATER"), (s.st.shields, "SHIELD")] {
                if n > 0 {
                    parts.push(format!("{name} {n}"));
                }
            }
            // A projector: how many shots it can still stop, of how many when it's rested.
            if s.st.projector > 0 {
                parts.push(format!("PROJECTOR {}/{}", s.skin.0, s.st.projector));
            }
            if s.carry > 0 {
                parts.push(format!("CARRYING {}", s.carry));
            }
            let mut lines = vec![format!("{} {} - {} - {}", team_name(s.team), s.name, s.order.name(), s.doing.name()), parts.join("  ")];
            // What a ship that goes by its drills learned about the one it's after.
            let foe = s.target.filter(|&t| a.ships[t].alive && a.ships[t].team != s.team);
            if a.brain[s.team].ways == Ways::Plan {
                // What its side's arch-AI has decided for its design.
                if let Some(style) = a.plan[s.team].get(s.slot).copied().flatten() {
                    lines.push(format!("ARCH: EVERY {} FLIES {}", s.name, style.name()));
                }
            } else if let (Some(t), true) = (foe, a.brain[s.team] != Brain::CARD) {
                if let Some(k) = a.drills.get(s.slot, a.ships[t].slot) {
                    lines.push(learned_line(&a.ships[t].name, k, a.brain[s.team]));
                }
            }
            Some(lines)
        } else if a.tick == 0 && f.contains(m.0, m.1) {
            Some(vec![format!("CLICK PLACES A {} {}. RIGHT-CLICK REMOVES.", team_name(self.team), self.lib[self.slot].name), "SPACE RUNS THE FIGHT. WASD SCROLLS.".into()])
        } else {
            None
        };
        if let Some(lines) = bar {
            let h = lines.len() as i32 * 9 + 6;
            let r = Rect::new(0, f.h - h, f.w, h);
            ui::panel(fb, r);
            for (k, l) in lines.iter().enumerate() {
                fb.text(l, 4, r.y + 4 + k as i32 * 9, TEXT, None, 1);
            }
        }
        if let Some(result) = a.over {
            // A stalemate: 20 s with no shots, no hits and nobody closing in. Say who can't fight any more.
            let disarmed: Vec<&str> = (0..2).filter(|&t| a.disarmed(t)).map(team_name).collect();
            let extra = if a.stalemate && !disarmed.is_empty() { 10 } else { 0 };
            let r = Rect::new(62, 112, 180, 62 + extra);
            ui::panel(fb, r);
            let (head, col) = match result {
                Some(t) => (format!("{} WINS", team_name(t)), team_col(t, 2)),
                None if a.stalemate => ("STALEMATE".to_string(), TEXT),
                None => ("DRAW".to_string(), TEXT),
            };
            fb.text_centered(&head, r.x + r.w / 2, r.y + 8, col, Some(OUTLINE), 2);
            let when = if a.stalemate { format!("A DRAW AT {clock}") } else { format!("IN {clock}") };
            fb.text_centered(&when, r.x + r.w / 2, r.y + 28, TEXT, None, 1);
            if extra > 0 {
                fb.text_centered(&format!("{} DISARMED", disarmed.join(" AND ")), r.x + r.w / 2, r.y + 38, GOLD, None, 1);
            }
            fb.text_centered("SPACE OR R REPLAYS", r.x + r.w / 2, r.y + 44 + extra, TEXT_FAINT, None, 1);
        }

        // Sidebar.
        ui::panel(fb, Rect::new(304, 0, 96, 300));
        for t in 0..2 {
            let r = BTN_TEAM[t];
            if self.team == t {
                fb.rect_bevel(r.x, r.y, r.w, r.h, team_col(t, 0), team_col(t, 1), OUTLINE);
                fb.text_centered(team_name(t), r.x + r.w / 2, r.y + 5, WHITE, None, 1);
            } else {
                ui::button(fb, r, team_name(t), r.contains(m.0, m.1), false);
            }
        }
        for (i, d) in self.lib.iter().enumerate() {
            let r = roster_rect(i);
            let sel = i == self.slot;
            let hover = r.contains(m.0, m.1);
            if sel {
                fb.rect_bevel(r.x, r.y, r.w, r.h, PANEL_DEEP, PANEL_LO, PANEL_HI);
                fb.rect_outline(r.x + 1, r.y + 1, r.w - 2, r.h - 2, ACCENT);
            } else if hover {
                fb.rect_bevel(r.x, r.y, r.w, r.h, PANEL_HI, WHITE, PANEL);
            } else {
                ui::panel(fb, r);
            }
            if d.is_empty() {
                fb.text("EMPTY", r.x + 14, r.y + 3, TEXT_FAINT, None, 1);
                continue;
            }
            draw_icon(fb, d, Rect::new(r.x + 1, r.y + 1, 11, 11), self.team);
            let ok = d.check().ok();
            let tc = if !ok { BAD } else if hover && !sel { PANEL_DEEP } else { TEXT };
            let cost = format!("{}", d.stats().cost);
            let cx = r.x + r.w - 3 - Framebuffer::text_width(&cost, 1);
            let name: String = d.name.chars().take(ROW_NAME.min(((cx - 2 - (r.x + 14)) / 6) as usize)).collect();
            fb.text(&name, r.x + 14, r.y + 3, tc, None, 1);
            fb.text(&cost, cx, r.y + 3, if hover && !sel { PANEL_LO } else { TEXT_DIM }, None, 1);
        }
        let run_label = if self.running { "PAUSE" } else if a.over.is_some() { "REPLAY" } else { "RUN" };
        ui::button(fb, BTN_RUN, run_label, BTN_RUN.contains(m.0, m.1), self.running);
        ui::button(fb, BTN_RESET, "RESET", BTN_RESET.contains(m.0, m.1), false);
        ui::button(fb, BTN_SPEED, &format!("{}X", self.speed), BTN_SPEED.contains(m.0, m.1), self.speed > 1);
        let tl = match self.test {
            Some(t) => format!("TEST {}/{}", t + 1, self.tests.len()),
            None => "NEXT TEST".into(),
        };
        ui::button(fb, BTN_TEST, &tl, BTN_TEST.contains(m.0, m.1), false);
        ui::button(fb, BTN_EMPTY, "EMPTY", BTN_EMPTY.contains(m.0, m.1), false);
        ui::button(fb, BTN_YARD, "YARD", BTN_YARD.contains(m.0, m.1), false);
        self.draw_minimap(fb);
        let sc = &a.score;
        for t in 0..2 {
            let y = 278 + t as i32 * 11;
            fb.text(team_name(t), 308, y, team_col(t, 2), None, 1);
            fb.text(&format!("{}/{}", a.alive(t), sc[t].placed), 338, y, TEXT, None, 1);
            fb.text(&format!("-{}", sc[t].px_lost), 368, y, TEXT_DIM, None, 1);
        }

        // Button tips go in the inspector's place when nothing in the field is hovered.
        if !f.contains(m.0, m.1) {
            let row = (0..self.lib.len()).find(|&i| roster_rect(i).contains(m.0, m.1)).map(|i| {
                let d = &self.lib[i];
                if d.is_empty() {
                    format!("[{}] AN EMPTY SLOT. DRAW A SHIP IN IT IN THE YARD, OR TAKE ONE FROM THE LIBRARY THERE.", i + 1)
                } else {
                    format!("[{}] {}: {} PX, ON {}. YOUR CLICKS PLACE IT. EDIT IT IN THE YARD. A RED NAME CAN'T FLY YET.", i + 1, d.name, d.stats().cost, d.order.name())
                }
            });
            let tip = if BTN_RUN.contains(m.0, m.1) {
                Some("[SPACE] RUN OR PAUSE THE FIGHT.")
            } else if BTN_RESET.contains(m.0, m.1) {
                Some("[R] PUT EVERY SHIP BACK WHERE IT STARTED. THE SAME SETUP ALWAYS PLAYS OUT THE SAME WAY.")
            } else if BTN_SPEED.contains(m.0, m.1) {
                Some("[F] 1X, 2X OR 4X. THE FRAMES STAY AT 15 FPS; FAST SKIPS AHEAD.")
            } else if BTN_TEST.contains(m.0, m.1) {
                Some("[N] LOAD THE NEXT READY-MADE MATCHUP.")
            } else if BTN_EMPTY.contains(m.0, m.1) {
                Some("REMOVE EVERY SHIP, FOR A SETUP OF YOUR OWN.")
            } else if BTN_YARD.contains(m.0, m.1) {
                Some("[ENTER] THE SHIPYARD, TO DRAW AND EDIT DESIGNS.")
            } else if MINI.contains(m.0, m.1) {
                Some("THE WHOLE ARENA AT 1/6 SCALE. CLICK OR DRAG TO MOVE THE VIEW, OR SCROLL WITH WASD OR THE ARROWS.")
            } else if (0..2).any(|t| BTN_TEAM[t].contains(m.0, m.1)) {
                Some("[T] WHICH TEAM YOUR CLICKS PLACE.")
            } else if (306..400).contains(&m.0) && m.1 >= 276 {
                Some("SHIPS LEFT / PLACED, AND PIXELS LOST, FOR EACH TEAM.")
            } else {
                row.as_deref()
            };
            if let Some(tip) = tip {
                let lines = wrap(tip, 49);
                let h = lines.len() as i32 * 9 + 6;
                let r = Rect::new(0, f.h - h, f.w, h);
                ui::panel(fb, r);
                for (k, l) in lines.iter().enumerate() {
                    fb.text(l, 4, r.y + 4 + k as i32 * 9, TEXT, None, 1);
                }
            }
        }
    }

    fn draw_minimap(&self, fb: &mut Framebuffer) {
        let a = &self.arena;
        let r = MINI;
        let inner = scene::minimap_ground(fb, r, a, MINI_SCALE);
        let (ox, oy) = (inner.x, inner.y);
        for (t, &(sx, sy)) in STATIONS.iter().enumerate() {
            for k in -2..=2 {
                pc(fb, inner, ox + sx / MINI_SCALE, oy + sy / MINI_SCALE + k, team_col(t, 0));
            }
        }
        for s in a.ships.iter().filter(|s| s.alive) {
            let n = (s.half() * 2 / MINI_SCALE).max(1);
            let (x, y) = (ox + s.x / MINI_SCALE - n / 2, oy + s.y / MINI_SCALE - n / 2);
            for k in 0..n * n {
                pc(fb, inner, x + k % n, y + k / n, team_col(s.team, 2));
            }
        }
        scene::minimap_view(fb, inner, FIELD, self.cam, MINI_SCALE);
    }

    fn draw_help(&self, fb: &mut Framebuffer) {
        let r = HELP_RECT;
        ui::panel(fb, r);
        fb.text("PROVING GROUND", r.x + 10, r.y + 8, ACCENT, Some(OUTLINE), 2);
        fb.text("ROUND 5", r.x + r.w - 52, r.y + 12, GOLD, None, 1);
        ui::button(fb, keys::BTN_KEYS, "F3: KEYS", keys::BTN_KEYS.contains(self.mouse.0, self.mouse.1), false);
        let mut y = r.y + 34;
        for (head, text) in help_body() {
            fb.text(head, r.x + 10, y, GOLD, None, 1);
            let lines = wrap(&text, 50);
            for (i, l) in lines.iter().enumerate() {
                fb.text(l, r.x + 58, y + i as i32 * 9, TEXT, None, 1);
            }
            y += lines.len() as i32 * 9 + 6;
        }
        // Every part's colour, in the yard's order, seven to a row, each as wide as its name.
        let ly = r.y + r.h - 44;
        let mut x = r.x + 14;
        for (k, p) in PARTS.iter().enumerate() {
            if k % 7 == 0 {
                x = r.x + 14;
            }
            let y = ly + (k / 7) as i32 * 10;
            fb.rect_fill(x - 1, y - 1, 6, 6, OUTLINE);
            fb.rect_fill(x, y, 4, 4, part_col(*p, 0));
            fb.text(p.name(), x + 8, y - 1, TEXT_DIM, None, 1);
            x += 14 + Framebuffer::text_width(p.name(), 1);
        }
        fb.text_centered("CLICK OR PRESS ANY KEY", r.x + r.w / 2, r.y + r.h - 18, ACCENT, None, 1);
    }
}

impl App for Pg {
    fn event(&mut self, ev: Ev) {
        match ev {
            Ev::Key(k) => self.key(k),
            Ev::Char(c) => {
                self.char_lobby(c);
                self.char_library(c);
                if self.naming {
                    let c = c.to_ascii_uppercase();
                    let name = &mut self.lib[self.slot].name;
                    if (c.is_ascii_alphanumeric() || c == ' ' || c == '-') && name.chars().count() < 10 {
                        name.push(c);
                    }
                }
            }
            Ev::Move(x, y) => {
                self.mouse = (x, y);
                if self.screen == Screen::Yard {
                    self.drag_yard(x, y);
                }
                if self.screen == Screen::Match {
                    self.drag_match(x, y);
                }
                if self.map_drag {
                    self.cam_from_map(x.clamp(MINI.x + 1, MINI.x + MINI.w - 2), y.clamp(MINI.y + 1, MINI.y + MINI.h - 2));
                }
            }
            Ev::Press { x, y, right } => self.press(x, y, right),
            Ev::Release { x, y, .. } => {
                self.map_drag = false;
                if self.screen == Screen::Yard {
                    self.release_yard(x, y);
                }
                if self.screen == Screen::Match {
                    self.release_match(x, y);
                }
            }
            Ev::Wheel(n) => {
                if library::open(self) {
                    self.wheel_library(n);
                } else if self.screen == Screen::Yard && !self.help && !self.keys && !self.confirm_quit {
                    self.wheel_yard(n);
                }
            }
        }
    }
    fn ctrl(&mut self, held: bool) {
        self.ctrl = held;
    }
    fn shift(&mut self, held: bool) {
        self.shift = held;
    }
    fn hold(&mut self, dirs: [bool; 4]) {
        self.held = dirs;
        for k in 0..4 {
            self.latch[k] |= dirs[k];
        }
    }
    fn tick(&mut self) {
        self.step();
    }
    fn draw(&self, fb: &mut Framebuffer) {
        self.render(fb);
    }
    fn cursor_kind(&self) -> u8 {
        (self.screen == Screen::Match && self.play.as_ref().is_some_and(|p| p.planting.is_some())) as u8
    }
    fn palette(&self) -> &Pal {
        &self.pal
    }
    fn crt(&self) -> bool {
        self.crt
    }
    fn toggle_crt(&mut self) {
        self.crt = !self.crt;
    }
    fn quit(&self) -> bool {
        self.quit
    }
}

#[cfg(test)]
mod tests {
    use super::yard::{yard_status, STATUS_RECT};
    use super::*;

    #[test]
    fn every_test_places_every_ship() {
        for (lib, tests) in [(defaults(), reference_tests()), (starter_deck(), tests())] {
            for t in tests {
                let mut a = Arena::new();
                for &(team, slot, x, y) in &t.place {
                    assert!(a.add(team, slot, &lib[slot], x, y), "{}", t.name);
                    assert!((0..AW).contains(&x) && (0..AH).contains(&y), "{} places a ship off the arena at {x},{y}", t.name);
                }
                // Each is named for what it places, and the name fits across the top of the field
                // beside the clock.
                assert_eq!(t.title(&lib), t.name);
                assert!(4 + Framebuffer::text_width(&t.title(&lib), 1) <= FIELD.w - 40, "{}", t.name);
            }
        }
    }

    #[test]
    fn the_game_starts_with_a_deck_that_plays_a_match() {
        // A fresh team is the starter deck and two empty slots: every design legal, a seeded
        // factory to plant, a miner on a build order, and nothing on any build order that could
        // never be built.
        let pg = Pg::new(None, None);
        assert_eq!(pg.lib.len(), SLOTS);
        assert_eq!(pg.lib[..7].iter().map(|d| d.to_line()).collect::<Vec<_>>(), starter_deck().iter().map(|d| d.to_line()).collect::<Vec<_>>());
        assert!(pg.lib[..7].iter().all(|d| d.check().ok()));
        assert!(pg.lib[7..].iter().all(|d| d.is_empty()));
        assert!(pg.lib.iter().any(|d| d.seeded && d.stats().factory > 0));
        assert!(pg.lib.iter().filter(|d| d.seeded).any(|d| d.build.iter().any(|&b| pg.lib[b].stats().scoops > 0)));
        for d in &pg.lib {
            for &b in &d.build {
                assert!(!pg.lib[b].seeded && pg.lib[b].stats().count <= d.stats().max_build(), "{} can't build {}", d.name, pg.lib[b].name);
            }
        }
        // Every ready-made fight runs to an end with it, factories and all, and stays sound.
        for t in tests().iter().filter(|t| !t.place.is_empty()) {
            let mut a = Arena::new();
            for &(team, slot, x, y) in &t.place {
                a.add(team, slot, &pg.lib[slot], x, y);
            }
            while a.over.is_none() {
                a.step();
                if let Err(e) = a.audit() {
                    panic!("{} at tick {}: {e}", t.name, a.tick);
                }
            }
        }
    }

    #[test]
    fn a_save_still_holding_a_retired_card_gets_todays_and_an_edited_card_is_kept() {
        // A save written before the miner was redrawn: the deck as it was then, with the fighter
        // edited by hand. Loading swaps the old miner for today's and leaves the fighter alone.
        let now = starter_deck();
        let old_miner = retired_starters().into_iter().filter(|(slot, _)| *slot == 3).last().unwrap().1;
        assert_ne!(old_miner.to_line(), now[3].to_line());
        let mut mine = now[0].clone();
        mine.set(GRID / 2, GRID / 2 - 6, Some(Part::Hull));
        let mut lines = vec![SAVE_HEADER.to_string(), mine.to_line()];
        lines.extend(now[1..3].iter().map(|d| d.to_line()));
        lines.push(old_miner.to_line());
        lines.extend(now[4..].iter().map(|d| d.to_line()));
        let path = std::env::temp_dir().join(format!("nomindwing_save_test_{}.txt", std::process::id()));
        std::fs::write(&path, lines.join("\n") + "\n").unwrap();
        let pg = Pg::new(Some(path.clone()), None);
        let _ = std::fs::remove_file(&path);
        assert_eq!(pg.lib[3].to_line(), now[3].to_line(), "the retired miner wasn't swapped for today's");
        assert_eq!(pg.lib[0].to_line(), mine.to_line(), "an edited card was swapped for the starter's");
        for slot in [1, 2, 4, 5, 6] {
            assert_eq!(pg.lib[slot].to_line(), now[slot].to_line());
        }
    }

    #[test]
    fn the_twin_fight_is_the_teams_fleet_on_both_sides_as_the_team_stands() {
        // The parts team: everything flies and fights but the outpost (slot 6), and the AEGIS
        // (slot 8), with no guns, flies with them.
        let lib = parts_library();
        assert_eq!(fliers(&lib), vec![0, 1, 2, 3, 4, 5, 7, 8]);
        let mut pg = Pg::parts();
        assert!(pg.load_named(TWIN) && !pg.load_named("NO SUCH FIGHT"));
        let count = |pg: &Pg, team: usize, slot: usize| pg.setup.iter().filter(|p| p.team == team && p.slot == slot).count();
        for s in 0..lib.len() {
            // As many of each as the dearest flier's cost buys: one at least, six at most.
            let dearest = fliers(&lib).iter().map(|&f| lib[f].stats().cost).max().unwrap();
            let want = if s == 6 { 0 } else { (dearest / lib[s].stats().cost).clamp(1, 6) as usize };
            assert_eq!((count(&pg, 0, s), count(&pg, 1, s)), (want, want), "{}", lib[s].name);
        }
        // Red stands where blue does, the other way round: the arena is 480 across.
        let mut blue: Vec<(usize, i32, i32)> = pg.setup.iter().filter(|p| p.team == 0).map(|p| (p.slot, AW - p.x, p.y)).collect();
        let mut red: Vec<(usize, i32, i32)> = pg.setup.iter().filter(|p| p.team == 1).map(|p| (p.slot, p.x, p.y)).collect();
        blue.sort();
        red.sort();
        assert_eq!(blue, red);
        // Every ship is placed, on the map.
        assert_eq!(pg.arena.ships.len(), pg.setup.len());
        // It's made from the team as it stands: with the frigate gone from the team, it's gone
        // from the fight.
        pg.lib[5] = Design::blank("");
        pg.load_named(TWIN);
        assert_eq!((count(&pg, 0, 5), count(&pg, 1, 5)), (0, 0));
        assert!(count(&pg, 0, 0) > 0);
    }

    #[test]
    fn a_sides_brain_button_steps_through_the_brains_and_the_drills_they_need_get_run() {
        let mut pg = Pg::new(None, None);
        pg.help = false;
        // 6 FIGHTERS VS 1 FRIGATE: slots 0 and 2.
        pg.load_test(0);
        assert_eq!((pg.brains, pg.arena.brain), ([Brain::CARD; 2], [Brain::CARD; 2]));
        // Each side's key takes its brain to the next, and a click on its button does the same.
        pg.key(Key::Letter('B'));
        pg.key(Key::Letter('V'));
        assert_eq!((pg.brains, pg.arena.brain), ([BRAINS[1]; 2], [BRAINS[1]; 2]));
        let r = BTN_BRAIN[0];
        pg.event(Ev::Press { x: r.x + 3, y: r.y + 3, right: false });
        assert_eq!(pg.brains, [BRAINS[2], BRAINS[1]]);
        assert!(pg.setup.len() == 7, "a click on the button places nothing");
        // A brain that goes by its drills: the fight is put back, the notice gets its frame,
        // and then the drills are run. The fight doesn't start before they are.
        pg.brains = [Brain { targets: true, ways: Ways::Bold }, Brain::CARD];
        pg.reset();
        pg.key(Key::Space);
        assert!(pg.arena.drills.get(0, 2).is_none() && pg.notice.as_ref().is_some_and(|n| n.0.contains("DRILLS")));
        pg.tick();
        assert!(pg.arena.drills.get(0, 2).is_none() && pg.arena.tick == 0);
        pg.tick();
        assert!(pg.arena.drills.get(0, 2).is_some() && pg.arena.drills.get(2, 0).is_some() && pg.arena.tick == 0);
        pg.tick();
        assert_eq!(pg.arena.tick, 1);
        // Replayed, the same drawings' drills are to hand at once.
        pg.key(Key::Letter('R'));
        assert!(pg.arena.drills.get(0, 2).is_some() && pg.arena.tick == 0);
        // The arch-AI needs no drills.
        pg.brains = [Brain::ARCH, Brain::CARD];
        pg.reset();
        assert!(pg.arena.drills.get(0, 2).is_none() && pg.arena.brain[0] == Brain::ARCH);
        // Every label fits its button, every tip fits the bar, and so does what a ship learned
        // about an enemy with the longest name there is.
        for t in 0..2 {
            for b in BRAINS {
                let label = format!("{} {}", team_name(t), b.short());
                assert!(Framebuffer::text_width(&label, 1) + 4 <= BTN_BRAIN[t].w, "{label}");
                assert!(wrap(&brain_tip(t, b), 49).len() <= 7, "{}", brain_tip(t, b));
            }
        }
        let k = Drill { safe: Some(Style::Orbit), safe_margin: -1000, bold: Some(Style::Orbit), bold_margin: -1000 };
        assert!(learned_line("WWWWWWWWWW", k, Brain { targets: true, ways: Ways::Bold }).len() <= 49);
        assert!(BTN_BRAIN[1].x + BTN_BRAIN[1].w <= FIELD.w && BTN_BRAIN[0].x + BTN_BRAIN[0].w <= BTN_BRAIN[1].x);
    }

    #[test]
    fn every_problem_message_fits_where_it_is_shown() {
        // Worst cases: every cell of the grid loose or buried, and the longest name a design can have.
        let n = (GRID * GRID) as usize;
        let checks = [
            Check { cores: 0, ..Check::default() },
            Check { cores: 1, loose: vec![(0, 0)], ..Check::default() },
            Check { cores: 1, loose: vec![(0, 0); n], ..Check::default() },
            Check { cores: 1, buried: vec![(0, 0)], ..Check::default() },
            Check { cores: 1, buried: vec![(0, 0); n], ..Check::default() },
            Check { cores: 1, hemmed: vec![(0, 0)], ..Check::default() },
            Check { cores: 1, hemmed: vec![(0, 0); n], ..Check::default() },
        ];
        for c in checks.iter().chain([&Check { cores: 1, ..Check::default() }]) {
            for (armed, power) in [(false, 100), (true, 100), (true, 99), (false, 1)] {
                for &order in &ORDERS {
                    // The yard's status line, one line inside its box.
                    let (status, _) = yard_status(c, armed, power, order);
                    assert!(Framebuffer::text_width(&status, 1) <= STATUS_RECT.w - 2, "the yard's status line overflows: {status}");
                }
            }
            // The arena's notice, wrapped inside the field.
            if let Some(why) = c.problem() {
                let notice = format!("WWWWWWWWWW CAN'T FLY: {why}");
                for l in notice_lines(&notice) {
                    assert!(Framebuffer::text_width(&l, 1) + 12 <= FIELD.w - 8, "the arena notice overflows: {l}");
                }
            }
        }
    }

    #[test]
    fn f1_opens_the_rules_in_a_match() {
        let mut pg = Pg::new(None, None);
        pg.start_match(true, 5);
        pg.help = false;
        pg.key(Key::Help);
        assert!(pg.help, "F1 did nothing in a match");
        // And Esc cancels planting before it offers to quit.
        pg.help = false;
        pg.key(Key::Letter('B'));
        assert!(pg.play.as_ref().unwrap().planting.is_some());
        pg.key(Key::Esc);
        assert!(pg.play.as_ref().unwrap().planting.is_none() && !pg.confirm_quit);
    }

    #[test]
    fn the_help_fits_its_panel() {
        // Headings start 34 px down, and each section ends with 6 px of gap. The part legend
        // starts 41 px from the bottom, so the last line of text must end at least 3 px above it.
        let end: i32 = 34 + help_body().iter().map(|(_, t)| wrap(t, 50).len() as i32 * 9 + 6).sum::<i32>() - 8;
        assert!(end <= HELP_RECT.h - 44, "the help runs {} px into the legend", end - (HELP_RECT.h - 44));
    }

    #[test]
    fn nine_roster_rows_fit_above_the_run_button() {
        for i in 0..SLOTS {
            let r = roster_rect(i);
            assert!(r.y + r.h <= BTN_RUN.y, "roster row {} runs into RUN", i + 1);
            assert!(r.y >= BTN_TEAM[0].y + BTN_TEAM[0].h, "roster row {} runs into the team buttons", i + 1);
            assert!(i == 0 || roster_rect(i - 1).y + roster_rect(i - 1).h <= r.y);
        }
        // Every slot answers to its number key, the ninth too.
        let mut pg = Pg::new(None, None);
        pg.help = false;
        pg.key(Key::Num(9));
        assert_eq!(pg.slot, 8);
    }

    #[test]
    fn a_team_saved_when_it_was_seven_opens_as_nine() {
        let path = std::env::temp_dir().join(format!("nmw_seven_{}.txt", std::process::id()));
        let seven: String = std::iter::once(SAVE_HEADER.to_string()).chain(starter_deck().iter().map(|d| d.to_line())).collect::<Vec<_>>().join("\n");
        std::fs::write(&path, seven).unwrap();
        let mut pg = Pg::new(Some(path.clone()), None);
        assert_eq!(pg.lib.len(), SLOTS);
        assert!(pg.lib[..7].iter().all(|d| !d.is_empty()) && pg.lib[7..].iter().all(|d| d.is_empty()));
        // Filling the ninth and saving writes nine lines, which read back the same.
        pg.lib[8] = starter_deck()[1].clone();
        pg.save();
        let again = Pg::new(Some(path.clone()), None);
        assert_eq!(again.lib.iter().map(|d| d.to_line()).collect::<Vec<_>>(), pg.lib.iter().map(|d| d.to_line()).collect::<Vec<_>>());
        let _ = std::fs::remove_file(&path);
        let _ = std::fs::remove_file(path.with_file_name("design_library.txt"));
    }
}
