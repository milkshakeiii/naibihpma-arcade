//! The shipyard. Designs are drawn on a 99x99 grid that zooms in whole steps (1x is the ship at
//! its real size), with a pencil, filled rectangles, flood fills and stamps (a copied box or a
//! whole design, placed as ordinary pixels). Parts are picked by category, and what the drawing
//! implies is read from icons and gauges, with the details on hover.

use super::design::*;
use super::icons::{self, Icon};
use super::{draw_shape, part_col, Pg};
use crate::app::Key;
use crate::framebuffer::{wrap, Framebuffer};
use crate::geom::Line;
use crate::palette::*;
use crate::ui::{self, Rect};

// Layout. The canvas is exactly the whole grid at 2x.
pub const CANVAS: Rect = Rect::new(6, 26, 198, 198);
pub const NAME_RECT: Rect = Rect::new(210, 26, 134, 18);
/// The library, beside the name of the drawing it would take or keep.
pub const BTN_LIBRARY: Rect = Rect::new(346, 27, 50, 16);
pub const STATUS_RECT: Rect = Rect::new(210, 133, 186, 11);
/// (Thirteen pixels shorter since 2026-11-17, when the build order got a second row: its nine
/// lines of text never filled it.)
const INFO_RECT: Rect = Rect::new(208, 202, 190, 96);
/// Built or seeded, and the build order: the deck slots it prints, and buttons to add them.
const KIND_BTNS: [Rect; 2] = [Rect::new(210, 148, 44, 13), Rect::new(256, 148, 50, 13)];
/// The build order: the START row (printed once), the REPEAT row under it (round and round),
/// and the ADD buttons, which put a design on the end of whichever row is lit.
const BUILD_Y: i32 = 163;
const ROUND_Y: i32 = 175;
const ADD_Y: i32 = 188;
const ROW_NAMES: [&str; 2] = ["START", "REPEAT"];
/// Where the word GROUP sits on the flight row, before its four buttons.
const COL_X: i32 = 212;
const COST_Y: i32 = 47;
const FLIGHT_Y: i32 = 58;
const GAUGE_Y: [i32; 4] = [69, 79, 89, 99];
const TALLY_Y: i32 = 111;
const BTN_ZOOM_OUT: Rect = Rect::new(6, 284, 16, 13);
const BTN_ZOOM_IN: Rect = Rect::new(70, 284, 16, 13);
const BTN_TO_ARENA: Rect = Rect::new(122, 284, 40, 13);
pub const BTN_MATCH: Rect = Rect::new(164, 284, 40, 13);
pub const ZOOMS: [i32; 4] = [1, 2, 4, 8];
/// Screen pixels the canvas scrolls per tick while a direction is held.
const SCROLL: i32 = 16;
const UNDO_DEPTH: usize = 64;

/// The team's tabs: `SLOTS` of them across the top.
pub fn tab_rect(i: usize) -> Rect {
    Rect::new(2 + i as i32 * 44, 3, 43, 20)
}
/// How much of a design's name a tab has room for.
const TAB_NAME: usize = 7;
/// Under the canvas: the categories, two rows for the open category's parts, the tools and the
/// zoom, each row 13 px high.
fn cat_rect(i: usize) -> Rect {
    // (Six of them 32 wide until 2026-11-17, when HEAVY WEAPONS made seven.)
    Rect::new(6 + i as i32 * 28, 228, 27, 13)
}
/// The i-th of n part buttons for the open category.
/// The button of part `i` of a category's `parts`: each as wide as its swatch and its name, so
/// four parts with names of different lengths all show theirs in the row.
/// A row that would run past the canvas carries on in the second row: five weapons don't fit one.
fn part_rect(i: usize, parts: &[Part]) -> Rect {
    let w = |p: &Part| 18 + Framebuffer::text_width(p.name(), 1);
    let (mut x, mut row) = (0, 0);
    for p in &parts[..i] {
        if x + w(p) > CANVAS.w + 2 {
            (x, row) = (0, row + 1);
        }
        x += w(p) + 2;
    }
    if x + w(&parts[i]) > CANVAS.w + 2 {
        (x, row) = (0, row + 1);
    }
    Rect::new(6 + x, 242 + row * 14, w(&parts[i]), 13)
}
fn tool_rect(i: usize) -> Rect {
    Rect::new(6 + i as i32 * 25, 270, 23, 13)
}
/// What the build order's rows and its ADD buttons say when pointed at.
fn build_row_tip(row: usize) -> &'static str {
    if row == 0 {
        "START: WHAT ITS FACTORY PIXELS PRINT FIRST, ONCE, IN ORDER. CLICK HERE AND THE ADD BUTTONS PUT DESIGNS ON THIS ROW. IT HOLDS 16."
    } else {
        "REPEAT: WHAT ITS FACTORY PIXELS PRINT AFTER THE START, ROUND AND ROUND. CLICK HERE AND THE ADD BUTTONS PUT DESIGNS ON THIS ROW. IT HOLDS 16. LEFT EMPTY, THE FACTORY STOPS AFTER ITS START."
    }
}
const ADD_TIP: &str = "ADD A DESIGN FROM YOUR DECK TO THE END OF THE LIT ROW: CLICK START OR REPEAT TO LIGHT IT. ITS FACTORY PIXELS PRINT THE START ONCE, THEN THE REPEAT ROW ROUND AND ROUND, SKIPPING WHAT IT'S TOO SMALL FOR.";

/// What the flight row says when pointed at. Ships choose how they fly, so there's nothing to
/// set: the row shows where the drawing starts them.
const FLIGHT_TIP: &str = "IN A FIGHT A SHIP HOLDS AT THE REACH OF ITS GUNS. NO ENGINES: IT STAYS PUT.";

/// Entry `k` of a build order's row: 0 the start, 1 the repeat. Sixteen fit across.
fn chip_rect(row: usize, k: usize) -> Rect {
    Rect::new(250 + k as i32 * 9, if row == 0 { BUILD_Y } else { ROUND_Y }, 8, 11)
}
/// A row's name: a click lights the row, and ADD adds to the lit one.
fn row_rect(row: usize) -> Rect {
    Rect::new(208, if row == 0 { BUILD_Y } else { ROUND_Y }, 40, 11)
}
fn add_rect(k: usize) -> Rect {
    Rect::new(250 + k as i32 * 14, ADD_Y, 12, 12)
}
fn tally_rect(k: usize) -> Rect {
    Rect::new(COL_X + (k as i32 % 5) * 37, TALLY_Y + (k as i32 / 5) * 11, 36, 10)
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Tool {
    Pencil,
    Rect,
    Fill,
    /// Drag a box to pick it up as a stamp; with a stamp held, clicks place it.
    Copy,
}

/// The buttons in the tool row.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Btn {
    Tool(Tool),
    Turn,
    Undo,
    Mirror,
    Clear,
}

const BTNS: [Btn; 8] = [Btn::Tool(Tool::Pencil), Btn::Tool(Tool::Rect), Btn::Tool(Tool::Fill), Btn::Tool(Tool::Copy), Btn::Turn, Btn::Undo, Btn::Mirror, Btn::Clear];

impl Btn {
    fn icon(self) -> Icon {
        match self {
            Btn::Tool(Tool::Pencil) => Icon::Pencil,
            Btn::Tool(Tool::Rect) => Icon::Rect,
            Btn::Tool(Tool::Fill) => Icon::Fill,
            Btn::Tool(Tool::Copy) => Icon::Copy,
            Btn::Turn => Icon::Turn,
            Btn::Undo => Icon::Undo,
            Btn::Mirror => Icon::Mirror,
            Btn::Clear => Icon::Clear,
        }
    }
}

/// The yard's editing state.
pub struct Ed {
    pub cat: Cat,
    pub part: Part,
    pub tool: Tool,
    pub mirror: bool,
    pub zoom: i32,
    /// Where the grid's top-left corner sits from the canvas's, in screen pixels.
    pub pan: (i32, i32),
    pub stamp: Option<Stamp>,
    /// The way the engine you paint pushes its ship: 0 ahead, 1 to its right, 2 astern, 3 to
    /// its left. [T] turns it.
    pub engine_way: usize,
    /// ADD puts designs on the build order's START row (else on its REPEAT row).
    pub add_to_start: bool,
    /// A mouse button held down on the canvas: whether it's the right one, and the cell it went
    /// down on.
    pub drag: Option<(bool, (i32, i32))>,
    last_cell: Option<(i32, i32)>,
    /// Earlier versions of designs, newest last: (slot, cells).
    undo: Vec<(usize, Vec<Option<Part>>)>,
    /// The design the view was last framed on, so coming back from the arena keeps your zoom.
    framed: Option<usize>,
}

impl Ed {
    /// Drops what undo remembers: the drawings it remembers have been swapped for others.
    pub(super) fn forget(&mut self) {
        self.undo.clear();
        self.stamp = None;
        self.drag = None;
    }

    pub fn new() -> Ed {
        Ed { cat: Cat::Armor, part: Part::Hull, tool: Tool::Pencil, mirror: true, zoom: 4, pan: (0, 0), stamp: None, engine_way: 0, add_to_start: false, drag: None, last_cell: None, undo: Vec::new(), framed: None }
    }
}

/// Keeps a pan inside the grid, or centres the grid when it's smaller than the canvas.
fn clamp_pan(zoom: i32, p: i32) -> i32 {
    let (span, room) = (GRID * zoom, CANVAS.w);
    if span <= room {
        (room - span) / 2
    } else {
        p.clamp(room - span, 0)
    }
}

fn mirror_x(x: i32) -> i32 {
    GRID - 1 - x
}

fn canvas_centre() -> (i32, i32) {
    (CANVAS.x + CANVAS.w / 2, CANVAS.y + CANVAS.h / 2)
}

/// What `yard_field` calls a phase ring.
pub const PHASE_RING: &str = "PHASE RING";

/// What a drawing is there for, if it has a stasis lattice, a slipstream or a vacuum, and how
/// far that reaches: for the status line of one with no guns. (Or a phase ring, and how many
/// empty pixels it encloses.)
pub fn yard_field(st: &Stats) -> Option<(&'static str, i32)> {
    // (A phase ring: what it's for is the space it encloses, which is none if it isn't
    // closed.)
    if st.ring > 0 {
        return Some((PHASE_RING, st.ring_hole));
    }
    [("STASIS", st.stasis_reach()), ("SLIPSTREAM", st.slip_reach()), ("VACUUM", st.vac_past())].into_iter().find(|f| f.1 > 0)
}

/// The yard's one-line status: what's wrong with the design, or how it will behave. `power` is
/// the percent of the power it needs that it has, and `field` what it's for if it's there for
/// a stasis lattice, a slipstream or a vacuum, and how far that reaches.
pub fn yard_status(check: &Check, armed: bool, power: i32, order: Order, collects: bool, field: Option<(&str, i32)>) -> (String, u8) {
    match check.problem() {
        Some(why) => (why, BAD),
        None if power < 100 => (format!("UNDERPOWERED: RUNS AT {power}%"), GOLD),
        // A ship that flies with a tractor and no scoop, armed or not.
        // (It read "A COLLECTOR: FOLLOWS YOUR MINERS" until 2026-11-17, a letter too long
        // for its box: the test of what fits never asked for this line.)
        None if collects => ("COLLECTOR: FOLLOWS YOUR MINERS".into(), GOOD),
        // A phase ring that isn't closed, or has something other than phase pixels on the
        // inside of it, encloses nothing and does nothing: it says so, guns or no. A closed
        // one says what it encloses, which is what it's worth.
        None if field == Some((PHASE_RING, 0)) => ("PHASE RING: ENCLOSES NOTHING".into(), GOLD),
        None if field.is_some_and(|f| f.0 == PHASE_RING) => (format!("PHASE RING: ENCLOSES {} PX", field.unwrap().1), GOOD),
        // A lattice and no guns: what it's for is its reach. (It read "NO GUNS: STAYS WITH
        // THE FLEET", which is what such a ship does least.)
        // A slipstream and no guns, or a vacuum: the same.
        None if !armed && field.is_some() => (format!("{}: REACHES {} PX", field.unwrap().0, field.unwrap().1), GOOD),
        None if !armed => {
            // What it does follows from the drawing: no engines, a scoop, or neither.
            let what = match order {
                Order::Hold => "NO GUNS: IT JUST SITS",
                Order::Harvest => "NO GUNS: RUNS HOME WHEN HIT",
                _ => "NO GUNS: STAYS WITH THE FLEET",
            };
            (what.into(), TEXT_DIM)
        }
        None => ("READY TO FLY".into(), GOOD),
    }
}

// ----- Clipped drawing -----

fn fill_clip(fb: &mut Framebuffer, clip: Rect, x: i32, y: i32, w: i32, h: i32, c: u8) {
    let (x0, y0) = (x.max(clip.x), y.max(clip.y));
    let (x1, y1) = ((x + w).min(clip.x + clip.w), (y + h).min(clip.y + clip.h));
    if x1 > x0 && y1 > y0 {
        fb.rect_fill(x0, y0, x1 - x0, y1 - y0, c);
    }
}

fn outline_clip(fb: &mut Framebuffer, clip: Rect, x: i32, y: i32, w: i32, h: i32, c: u8) {
    fill_clip(fb, clip, x, y, w, 1, c);
    fill_clip(fb, clip, x, y + h - 1, w, 1, c);
    fill_clip(fb, clip, x, y, 1, h, c);
    fill_clip(fb, clip, x + w - 1, y, 1, h, c);
}

/// A part's colour in a 9x9 outlined square with its shape inside.
pub fn swatch(fb: &mut Framebuffer, x: i32, y: i32, p: Part) {
    fb.rect_fill(x, y, 9, 9, OUTLINE);
    fb.rect_fill(x + 1, y + 1, 7, 7, part_col(p, 0));
    icons::draw(fb, Icon::Part(p), x + 1, y + 1, if p.turret() { breech_col(p) } else { OUTLINE });
}

/// The colour of a turret's breech, its centre pixel.
pub fn breech_col(p: Part) -> u8 {
    if p == Part::Tracker {
        TRACKER_HI
    } else {
        GOLD
    }
}

impl Pg {
    // ----- Canvas geometry -----

    /// The cell under a screen point, on or off the grid.
    fn cell_at(&self, x: i32, y: i32) -> (i32, i32) {
        let e = &self.ed;
        ((x - CANVAS.x - e.pan.0).div_euclid(e.zoom), (y - CANVAS.y - e.pan.1).div_euclid(e.zoom))
    }

    /// The cell under a screen point, if the point is on the canvas and the cell on the grid.
    pub(super) fn cell_under(&self, x: i32, y: i32) -> Option<(i32, i32)> {
        if !CANVAS.contains(x, y) {
            return None;
        }
        let c = self.cell_at(x, y);
        ((0..GRID).contains(&c.0) && (0..GRID).contains(&c.1)).then_some(c)
    }

    /// The screen point in the middle of a cell (for the screenshot tool).
    /// The middle of a part's button, with its category open (the screenshots point at it).
    pub fn part_button(&self, part: Part) -> (i32, i32) {
        let parts = Cat::of(part).parts();
        let r = part_rect(parts.iter().position(|&p| p == part).unwrap_or(0), &parts);
        (r.x + r.w / 2, r.y + r.h / 2)
    }

    pub fn cell_screen(&self, (cx, cy): (i32, i32)) -> (i32, i32) {
        let e = &self.ed;
        (CANVAS.x + e.pan.0 + cx * e.zoom + e.zoom / 2, CANVAS.y + e.pan.1 + cy * e.zoom + e.zoom / 2)
    }

    /// Picks the design to edit and frames it.
    pub fn select_slot(&mut self, i: usize) {
        self.slot = i;
        self.ed.drag = None;
        self.fit_view();
    }

    /// The biggest zoom that shows the whole drawing, centred on it.
    pub fn fit_view(&mut self) {
        let b = self.lib[self.slot].used_box();
        let (x0, y0, x1, y1) = b.unwrap_or((GRID / 2, GRID / 2, GRID / 2, GRID / 2));
        let (w, h) = (x1 - x0 + 3, y1 - y0 + 3);
        let z = if b.is_none() { 4 } else { ZOOMS.iter().rev().copied().find(|&z| w * z <= CANVAS.w && h * z <= CANVAS.h).unwrap_or(1) };
        let centre = |a: i32, b: i32| CANVAS.w / 2 - (a + b + 1) * z / 2;
        self.ed.zoom = z;
        self.ed.pan = (clamp_pan(z, centre(x0, x1)), clamp_pan(z, centre(y0, y1)));
        self.ed.framed = Some(self.slot);
    }

    /// Frames the design on the way into the yard, unless the view is already on it.
    pub(super) fn frame_if_new(&mut self) {
        if self.ed.framed != Some(self.slot) {
            self.fit_view();
        }
    }

    /// Zooms whole steps in (+) or out (-), keeping the point under (sx, sy) where it is.
    fn zoom_by(&mut self, steps: i32, (sx, sy): (i32, i32)) {
        let e = &mut self.ed;
        let k = ZOOMS.iter().position(|&z| z == e.zoom).unwrap_or(1) as i32;
        let (oz, nz) = (e.zoom, ZOOMS[(k + steps).clamp(0, ZOOMS.len() as i32 - 1) as usize]);
        if nz == oz {
            return;
        }
        let f = |p: i32, pan: i32| clamp_pan(nz, p - (p - pan) * nz / oz);
        e.pan = (f(sx - CANVAS.x, e.pan.0), f(sy - CANVAS.y, e.pan.1));
        e.zoom = nz;
    }

    /// Scrolls the canvas while WASD or the arrows are held.
    pub(super) fn scroll_yard(&mut self, dirs: &[bool]) {
        let e = &mut self.ed;
        let dx = (dirs[3] as i32 - dirs[2] as i32) * SCROLL;
        let dy = (dirs[1] as i32 - dirs[0] as i32) * SCROLL;
        e.pan = (clamp_pan(e.zoom, e.pan.0 - dx), clamp_pan(e.zoom, e.pan.1 - dy));
    }

    pub(super) fn wheel_yard(&mut self, notches: i32) {
        let at = if CANVAS.contains(self.mouse.0, self.mouse.1) { self.mouse } else { canvas_centre() };
        self.zoom_by(notches, at);
    }

    // ----- Undo -----

    /// Remembers the design as it is, before a change.
    fn remember(&mut self) {
        let (slot, cells) = (self.slot, self.lib[self.slot].cells.clone());
        let u = &mut self.ed.undo;
        if u.iter().rev().find(|(s, _)| *s == slot).is_some_and(|(_, c)| *c == cells) {
            return;
        }
        u.push((slot, cells));
        if u.len() > UNDO_DEPTH {
            u.remove(0);
        }
    }

    fn undo(&mut self) {
        let (slot, now) = (self.slot, self.lib[self.slot].cells.clone());
        while let Some(k) = self.ed.undo.iter().rposition(|(s, _)| *s == slot) {
            let (_, cells) = self.ed.undo.remove(k);
            if cells != now {
                self.lib[slot].cells = cells;
                self.lock_seed();
                self.edited = true;
                self.save();
                return;
            }
        }
        self.say("NOTHING TO UNDO");
    }

    // ----- Tools -----

    fn set_tool(&mut self, t: Tool) {
        self.ed.tool = t;
        self.ed.drag = None;
        if t == Tool::Copy {
            self.ed.stamp = None;
        }
    }

    /// Opens a category, or with it already open, moves to its next part.
    fn pick_cat(&mut self, c: Cat) {
        let parts = c.parts();
        if parts.is_empty() {
            self.say(&format!("NO {} PARTS YET", c.name()));
            return;
        }
        let e = &mut self.ed;
        e.part = if e.cat == c { parts[parts.iter().position(|&p| p == e.part).map_or(0, |k| k + 1) % parts.len()] } else { parts[0] };
        e.cat = c;
        if e.tool == Tool::Copy {
            e.tool = Tool::Pencil;
        }
    }

    /// Tab: the next part of the open category, round and round.
    fn next_part(&mut self) {
        let parts = self.ed.cat.parts();
        match parts.iter().position(|&p| p == self.ed.part) {
            Some(_) if parts.len() == 1 => self.say(&format!("{} IS THE ONLY PART IN {}", parts[0].name(), self.ed.cat.name())),
            Some(k) => self.pick_part(parts[(k + 1) % parts.len()]),
            None => {}
        }
    }

    fn pick_part(&mut self, p: Part) {
        self.ed.part = p;
        self.ed.cat = Cat::of(p);
        if self.ed.tool == Tool::Copy {
            self.ed.tool = Tool::Pencil;
        }
    }

    /// [T]: turns the stamp you're holding, or, painting engines, the way they push.
    fn turn_stamp(&mut self) {
        match &self.ed.stamp {
            Some(st) => self.ed.stamp = Some(st.turned()),
            None if self.ed.part == Part::Engine => {
                self.ed.engine_way = (self.ed.engine_way + 1) % 4;
                self.say(&format!("ENGINE: IT PUSHES THE SHIP {}", WAYS[self.ed.engine_way]));
            }
            None => self.say("NO STAMP TO TURN: [C] COPY ONE FIRST"),
        }
    }

    /// The part the tools put down: the one chosen, and for an engine the one turned the way
    /// [T] has it.
    fn brush(&self) -> Part {
        if self.ed.part == Part::Engine {
            Part::engine(self.ed.engine_way)
        } else {
            self.ed.part
        }
    }

    /// Picks up a whole design as a stamp.
    pub fn stamp_design(&mut self, i: usize) {
        let st = self.lib[i].as_stamp();
        if st.is_empty() {
            self.say("THAT DESIGN IS EMPTY");
            return;
        }
        self.ed.stamp = Some(st);
        self.ed.tool = Tool::Copy;
        self.ed.drag = None;
    }

    fn press_btn(&mut self, b: Btn) {
        match b {
            Btn::Tool(t) => self.set_tool(t),
            Btn::Turn => self.turn_stamp(),
            Btn::Undo => self.undo(),
            Btn::Mirror => self.ed.mirror = !self.ed.mirror,
            Btn::Clear => {
                self.remember();
                self.lib[self.slot].cells.iter_mut().for_each(|c| *c = None);
                self.lock_seed();
                self.edited = true;
                self.save();
            }
        }
    }

    /// A seeded design keeps its seed core at the grid's centre, whatever is painted over it.
    fn lock_seed(&mut self) {
        let d = &mut self.lib[self.slot];
        if d.seeded && d.get(SEED_CELL.0, SEED_CELL.1) != Some(Part::Core) {
            d.clear_at(SEED_CELL);
            d.set(SEED_CELL.0, SEED_CELL.1, Some(Part::Core));
        }
    }

    /// Built or seeded. Seeding puts the seed core at the grid's centre.
    fn set_kind(&mut self, seeded: bool) {
        if self.lib[self.slot].seeded == seeded {
            return;
        }
        self.remember();
        self.lib[self.slot].seeded = seeded;
        self.lock_seed();
        self.edited = true;
        self.save();
    }

    fn has_factory(&self) -> bool {
        self.lib[self.slot].cells.contains(&Some(Part::Factory))
    }

    /// The pencil on one cell, and its mirror.
    fn paint(&mut self, (cx, cy): (i32, i32), erase: bool) {
        let p = if erase { None } else { Some(self.brush()) };
        let mirror = self.ed.mirror;
        let d = &mut self.lib[self.slot];
        // A turret's twin is centred on the mirrored cell, and is skipped if it would overlap.
        let mut changed = d.put(cx, cy, p);
        let mx = mirror_x(cx);
        let n = p.map_or(1, |q| q.side());
        if mirror && mx != cx && !(n > 1 && (mx - cx).abs() < n) {
            // (An engine's twin pushes the mirrored way.)
            changed |= d.put(mx, cy, p.map(Part::mirrored));
        }
        self.edited |= changed;
        self.lock_seed();
    }

    /// Paints every cell between the last sample and this one, so a fast stroke leaves no gaps.
    /// A turret stamps once per click, not along the stroke.
    fn paint_to(&mut self, cell: (i32, i32), erase: bool) {
        if !erase && self.ed.part.side() > 1 {
            if self.ed.last_cell.is_none() {
                self.paint(cell, false);
            }
            self.ed.last_cell = Some(cell);
            return;
        }
        let from = self.ed.last_cell.unwrap_or(cell);
        let mut l = Line::new(from.0, from.1, cell.0, cell.1);
        loop {
            self.paint((l.x, l.y), erase);
            if (l.x, l.y) == cell {
                break;
            }
            l.step();
        }
        self.ed.last_cell = Some(cell);
    }

    fn flood_at(&mut self, cell: (i32, i32), erase: bool) {
        let p = if erase { None } else { Some(self.brush()) };
        let mirror = self.ed.mirror && mirror_x(cell.0) != cell.0;
        let d = &mut self.lib[self.slot];
        if erase && d.get(cell.0, cell.1).is_none() {
            return;
        }
        match d.flood(cell, p) {
            Ok(mut changed) => {
                if mirror {
                    changed |= d.flood((mirror_x(cell.0), cell.1), p.map(Part::mirrored)).unwrap_or(false);
                }
                self.edited |= changed;
                self.lock_seed();
            }
            Err(why) => self.say(why),
        }
    }

    /// The rectangle tool's box, and its mirror. A part that comes in blocks tiles the box in
    /// whole ones, and the mirrored box tiles exactly the mirror image of them.
    fn fill_box_at(&mut self, a: (i32, i32), b: (i32, i32), erase: bool) {
        let p = if erase { None } else { Some(self.brush()) };
        let (x0, y0) = (a.0.min(b.0), a.1.min(b.1));
        let (mut x1, mut y1) = (a.0.max(b.0), a.1.max(b.1));
        if let Some(n) = p.map(|q| q.side()).filter(|&n| n > 1) {
            x1 = x0 + (x1 - x0 + 1) / n * n - 1;
            y1 = y0 + (y1 - y0 + 1) / n * n - 1;
        }
        // Heavy plate fills an even number of cells: an odd box loses its right-hand column
        // here, so the mirrored box loses its left-hand one.
        if p == Some(Part::HeavyPlate) && (x1 - x0 + 1) * (y1 - y0 + 1) % 2 == 1 {
            if x1 > x0 {
                x1 -= 1;
            } else {
                y1 -= 1;
            }
        }
        let mirror = self.ed.mirror;
        let d = &mut self.lib[self.slot];
        let mut changed = d.fill_box((x0, y0), (x1, y1), p);
        if mirror && mirror_x(x1) != x0 {
            changed |= d.fill_box((mirror_x(x1), y0), (mirror_x(x0), y1), p.map(Part::mirrored));
        }
        self.edited |= changed;
        self.lock_seed();
    }

    /// Where a held stamp's mirror image goes, if it doesn't overlap the stamp itself.
    fn mirror_origin(&self, st: &Stamp, o: (i32, i32)) -> Option<(i32, i32)> {
        let mo = (GRID - o.0 - st.w, o.1);
        let overlap = o.0 < mo.0 + st.w && mo.0 < o.0 + st.w;
        (self.ed.mirror && !overlap).then_some(mo)
    }

    fn place(&mut self, st: &Stamp, cell: (i32, i32)) {
        let o = st.origin(cell);
        let mo = self.mirror_origin(st, o);
        let d = &mut self.lib[self.slot];
        let mut changed = d.paste(st, o);
        if let Some(mo) = mo {
            changed |= d.paste(&st.flipped(), mo);
        }
        self.edited |= changed;
        self.lock_seed();
    }

    fn finish_edit(&mut self) {
        if self.edited {
            self.save();
        }
    }

    // ----- Input -----

    pub(super) fn key_yard(&mut self, k: Key) {
        match k {
            Key::Num(n) if (1..=CATS.len() as u8).contains(&n) => self.pick_cat(CATS[n as usize - 1]),
            Key::Tab => self.next_part(),
            Key::Letter('P') => self.set_tool(Tool::Pencil),
            Key::Letter('R') => self.set_tool(Tool::Rect),
            Key::Letter('F') => self.set_tool(Tool::Fill),
            Key::Letter('C') => self.set_tool(Tool::Copy),
            Key::Letter('T') => self.turn_stamp(),
            Key::Letter('U') => self.undo(),
            Key::Letter('M') => self.ed.mirror = !self.ed.mirror,
            Key::Letter('L') => self.open_library(),
            Key::Letter('Q') => self.zoom_by(-1, canvas_centre()),
            Key::Letter('E') => self.zoom_by(1, canvas_centre()),
            // (A design's order, how it flies and how many leave together had keys too, O, Y
            // and G, until none of the three was a setting.)
            _ => {}
        }
    }


    pub(super) fn press_yard(&mut self, x: i32, y: i32, right: bool) {
        if let Some(cell) = self.cell_under(x, y) {
            self.press_canvas(cell, right);
            return;
        }
        if let Some(i) = (0..self.lib.len()).find(|&i| tab_rect(i).contains(x, y)) {
            if right {
                self.stamp_design(i);
            } else {
                self.select_slot(i);
            }
            return;
        }
        if self.has_factory() {
            for row in 0..2 {
                let d = &mut self.lib[self.slot];
                let list = if row == 0 { &mut d.first } else { &mut d.build };
                if let Some(k) = (0..list.len()).find(|&k| chip_rect(row, k).contains(x, y)) {
                    // Right-click takes an entry off the build order.
                    if right {
                        list.remove(k);
                        self.edited = true;
                        self.save();
                    }
                    return;
                }
                // A click on a row's name: ADD adds to that row from now on.
                if !right && row_rect(row).contains(x, y) {
                    self.ed.add_to_start = row == 0;
                    return;
                }
            }
        }
        if right {
            // Right-click off the canvas puts a held stamp down.
            self.ed.stamp = None;
            return;
        }
        let parts = self.ed.cat.parts();
        if let Some(i) = (0..CATS.len()).find(|&i| cat_rect(i).contains(x, y)) {
            self.pick_cat(CATS[i]);
        } else if let Some(i) = (0..parts.len()).find(|&i| part_rect(i, &parts).contains(x, y)) {
            self.pick_part(parts[i]);
        } else if let Some(i) = (0..BTNS.len()).find(|&i| tool_rect(i).contains(x, y)) {
            self.press_btn(BTNS[i]);
        } else if BTN_ZOOM_OUT.contains(x, y) {
            self.zoom_by(-1, canvas_centre());
        } else if BTN_ZOOM_IN.contains(x, y) {
            self.zoom_by(1, canvas_centre());
        } else if BTN_TO_ARENA.contains(x, y) {
            self.switch_screen();
        } else if BTN_MATCH.contains(x, y) {
            self.end_naming();
            self.menu = true;
        } else if let Some(k) = (0..2).find(|&k| KIND_BTNS[k].contains(x, y)) {
            self.set_kind(k == 1);
        } else if let Some(k) = (0..self.lib.len()).find(|&k| self.has_factory() && add_rect(k).contains(x, y)) {
            let start = self.ed.add_to_start;
            let d = &mut self.lib[self.slot];
            let list = if start { &mut d.first } else { &mut d.build };
            if list.len() < BUILD_MAX {
                list.push(k);
                self.edited = true;
                self.save();
            } else {
                self.say("A ROW OF THE BUILD ORDER HOLDS 16 AT MOST");
            }
        } else if BTN_LIBRARY.contains(x, y) {
            self.open_library();
        } else if NAME_RECT.contains(x, y) {
            self.naming = true;
        }
    }

    fn press_canvas(&mut self, cell: (i32, i32), right: bool) {
        match self.ed.tool {
            Tool::Pencil => {
                self.remember();
                self.ed.drag = Some((right, cell));
                self.ed.last_cell = None;
                self.paint_to(cell, right);
            }
            Tool::Rect => self.ed.drag = Some((right, cell)),
            Tool::Fill => {
                self.remember();
                self.flood_at(cell, right);
                self.finish_edit();
            }
            Tool::Copy => match (self.ed.stamp.clone(), right) {
                (Some(_), true) => self.ed.stamp = None,
                (Some(st), false) => {
                    self.remember();
                    self.place(&st, cell);
                    self.finish_edit();
                }
                (None, false) => self.ed.drag = Some((false, cell)),
                (None, true) => {}
            },
        }
    }

    /// The pointer moved: a pencil stroke follows it.
    pub(super) fn drag_yard(&mut self, x: i32, y: i32) {
        if let (Some((right, _)), Tool::Pencil) = (self.ed.drag, self.ed.tool) {
            if let Some(cell) = self.cell_under(x, y) {
                self.paint_to(cell, right);
            }
        }
    }

    pub(super) fn release_yard(&mut self, x: i32, y: i32) {
        let Some((right, start)) = self.ed.drag.take() else { return };
        let (ex, ey) = self.cell_at(x, y);
        let end = (ex.clamp(0, GRID - 1), ey.clamp(0, GRID - 1));
        match self.ed.tool {
            Tool::Rect => {
                self.remember();
                self.fill_box_at(start, end, right);
            }
            Tool::Copy => {
                let st = self.lib[self.slot].copy_box(start, end);
                if st.is_empty() {
                    self.say("NOTHING THERE TO COPY");
                } else {
                    self.ed.stamp = Some(st);
                }
            }
            Tool::Pencil | Tool::Fill => {}
        }
        self.ed.last_cell = None;
        self.finish_edit();
    }

    // ----- Drawing -----

    pub(super) fn draw_yard(&self, fb: &mut Framebuffer) {
        fb.rect_fill(0, 0, 400, 300, PANEL_LO);
        let m = self.mouse;
        let e = &self.ed;
        for (i, d) in self.lib.iter().enumerate() {
            let r = tab_rect(i);
            let (sel, hov) = (i == self.slot, r.contains(m.0, m.1));
            ui::button(fb, r, "", hov && !sel, sel);
            // An empty slot says so, quietly: it isn't a drawing with something wrong with it.
            let name: String = if d.is_empty() { "EMPTY".into() } else { d.name.chars().take(TAB_NAME).collect() };
            let col = if d.is_empty() { if sel { ACCENT } else { TEXT_FAINT } } else if !d.check().ok() { BAD } else if sel { ACCENT } else if hov { PANEL_DEEP } else { TEXT };
            fb.text_centered(&name, r.x + r.w / 2, r.y + 3, col, None, 1);
            fb.text_centered(&format!("{}", i + 1), r.x + r.w / 2, r.y + 11, if sel { ACCENT } else { TEXT_FAINT }, None, 1);
        }

        let d = &self.lib[self.slot];
        let check = d.check();
        self.draw_canvas(fb, d, &check);

        // Categories, the open category's parts, the tools and the zoom.
        for (i, &cat) in CATS.iter().enumerate() {
            let r = cat_rect(i);
            let (sel, hov) = (cat == e.cat, r.contains(m.0, m.1));
            ui::button(fb, r, "", hov && !sel, sel);
            let col = if cat.parts().is_empty() { TEXT_FAINT } else if sel { ACCENT } else if hov { PANEL_DEEP } else { TEXT };
            icons::draw(fb, Icon::Cat(cat), r.x + 5, r.y + 3, col);
            fb.text(&format!("{}", i + 1), r.x + 17, r.y + 3, if hov && !sel { PANEL_LO } else { TEXT_DIM }, None, 1);
        }
        let parts = e.cat.parts();
        for (i, &p) in parts.iter().enumerate() {
            let r = part_rect(i, &parts);
            let (sel, hov) = (p == e.part && e.tool != Tool::Copy, r.contains(m.0, m.1));
            ui::button(fb, r, "", hov && !sel, sel);
            swatch(fb, r.x + 3, r.y + 2, p);
            if r.w >= 17 + Framebuffer::text_width(p.name(), 1) {
                fb.text(p.name(), r.x + 15, r.y + 3, if sel { ACCENT } else if hov { PANEL_DEEP } else { TEXT }, None, 1);
            }
        }
        for (i, &b) in BTNS.iter().enumerate() {
            let r = tool_rect(i);
            let down = match b {
                Btn::Tool(t) => e.tool == t,
                Btn::Mirror => e.mirror,
                _ => false,
            };
            let hov = r.contains(m.0, m.1);
            ui::button(fb, r, "", hov && !down, down);
            let off = b == Btn::Turn && e.stamp.is_none() && e.part != Part::HeavyPlate && e.part != Part::Engine;
            let col = if off { TEXT_FAINT } else if down { ACCENT } else if hov { PANEL_DEEP } else { TEXT };
            icons::draw(fb, b.icon(), r.x + 8, r.y + 3, col);
        }
        ui::button(fb, BTN_ZOOM_OUT, "-", BTN_ZOOM_OUT.contains(m.0, m.1), false);
        ui::button(fb, BTN_ZOOM_IN, "+", BTN_ZOOM_IN.contains(m.0, m.1), false);
        fb.text_centered(&format!("ZOOM {}X", e.zoom), 46, 287, TEXT_DIM, None, 1);
        if let Some((cx, cy)) = self.cell_under(m.0, m.1) {
            fb.text(&format!("{cx},{cy}"), 90, 287, TEXT_FAINT, None, 1);
        }
        ui::button(fb, BTN_TO_ARENA, "ARENA", BTN_TO_ARENA.contains(m.0, m.1), false);
        ui::button(fb, BTN_MATCH, "MATCH", BTN_MATCH.contains(m.0, m.1), false);
        ui::button(fb, BTN_LIBRARY, "LIBRARY", BTN_LIBRARY.contains(m.0, m.1), false);

        // The name, and what the drawing implies.
        if self.naming {
            fb.rect_fill(NAME_RECT.x, NAME_RECT.y, NAME_RECT.w, NAME_RECT.h, PANEL_DEEP);
            fb.text(&format!("{}_", d.name), NAME_RECT.x + 2, NAME_RECT.y + 2, ACCENT, None, 2);
        } else {
            fb.text(&d.name, NAME_RECT.x + 2, NAME_RECT.y + 2, if NAME_RECT.contains(m.0, m.1) { WHITE } else { TEXT }, Some(OUTLINE), 2);
        }
        let st = d.stats();
        let cost = if st.cores > 1 { format!("COST {} PX (+{} CORES)", st.cost, st.cost - st.count) } else { format!("COST {} PX", st.cost) };
        fb.text(&cost, COL_X, COST_Y, GOLD, None, 1);
        // How the drawing flies until its side has thought about it. Nothing to choose: in a
        // fight, ships choose.
        icons::draw(fb, Icon::Flight(st.flight), COL_X, FLIGHT_Y, ACCENT);
        fb.text(st.flight.name(), COL_X + 11, FLIGHT_Y, ACCENT, None, 1);
        for (k, (icon, segs, mark, value, lit)) in gauges(&st).into_iter().enumerate() {
            let y = GAUGE_Y[k];
            icons::draw(fb, icon, COL_X, y, TEXT_DIM);
            for s in 0..10 {
                fb.rect_fill(COL_X + 11 + s * 6, y + 1, 5, 5, if s < segs { lit } else { PANEL_DEEP });
            }
            if let Some(mk) = mark {
                fb.rect_fill(COL_X + 11 + mk * 6 - 1, y - 1, 1, 9, GOLD);
            }
            fb.text(&value, COL_X + 76, y, TEXT, None, 1);
        }
        for (k, (p, n)) in tally(d, &st).into_iter().enumerate() {
            let r = tally_rect(k);
            swatch(fb, r.x, r.y, p);
            fb.text(&n.to_string(), r.x + 12, r.y + 1, TEXT, None, 1);
        }
        let (status, col) = yard_status(&check, st.armed(), st.power_pct(), st.order(), st.collector(), yard_field(&st));
        fb.text(&status, STATUS_RECT.x + 2, STATUS_RECT.y + 2, col, None, 1);

        // Built or seeded, and what its factory pixels print.
        for (k, label) in ["BUILT", "SEEDED"].iter().enumerate() {
            let r = KIND_BTNS[k];
            ui::button(fb, r, label, r.contains(m.0, m.1), d.seeded == (k == 1));
        }
        if d.seeded {
            fb.text(&format!("ROCK {}", st.count), 312, KIND_BTNS[0].y + 3, TEXT_DIM, None, 1);
        }
        if st.factory > 0 || d.cells.contains(&Some(Part::Factory)) {
            for (row, list) in [&d.first, &d.build].into_iter().enumerate() {
                // The row ADD adds to is lit.
                let (name, lit) = (row_rect(row), (row == 0) == self.ed.add_to_start);
                fb.text(ROW_NAMES[row], 210, name.y + 2, if lit { ACCENT } else if name.contains(m.0, m.1) { TEXT } else { TEXT_FAINT }, None, 1);
                for (k, &b) in list.iter().enumerate() {
                    let r = chip_rect(row, k);
                    let bad = self.build_problem(d, b).is_some();
                    fb.rect_fill(r.x, r.y, r.w, r.h, if bad { BAD_DIM } else { PANEL_DEEP });
                    fb.text_centered(&format!("{}", b + 1), r.x + r.w / 2, r.y + 2, if bad { BAD } else if r.contains(m.0, m.1) { WHITE } else { ACCENT }, None, 1);
                }
                if list.is_empty() {
                    let none = match (row, d.first.is_empty()) {
                        (0, _) => "NOTHING FIRST",
                        (_, true) => "NOTHING YET",
                        _ => "NOTHING: THEN IT STOPS",
                    };
                    fb.text(none, 250, name.y + 2, TEXT_FAINT, None, 1);
                }
            }
            fb.text("ADD", 210, ADD_Y + 3, TEXT_FAINT, None, 1);
            for k in 0..self.lib.len() {
                let r = add_rect(k);
                ui::button(fb, r, &format!("{}", k + 1), r.contains(m.0, m.1), false);
            }
        } else {
            fb.text("NO FACTORY PIXELS,", 210, BUILD_Y + 2, TEXT_FAINT, None, 1);
            fb.text("SO NO BUILD ORDER", 210, ADD_Y + 3, TEXT_FAINT, None, 1);
        }

        ui::inset(fb, INFO_RECT);
        let info = self.yard_info(d, &st);
        for (i, l) in wrap(&info, ((INFO_RECT.w - 8) / 6) as usize).iter().enumerate().take(9) {
            fb.text(l, INFO_RECT.x + 4, INFO_RECT.y + 4 + i as i32 * 9, TEXT, None, 1);
        }
    }

    fn draw_canvas(&self, fb: &mut Framebuffer, d: &Design, check: &Check) {
        let (c, e, m) = (CANVAS, &self.ed, self.mouse);
        let z = e.zoom;
        fb.rect_fill(c.x - 1, c.y - 1, c.w + 2, c.h + 2, PANEL_HI);
        fb.rect_fill(c.x, c.y, c.w, c.h, if z >= 4 { OUTLINE } else { SPACE });
        // Screen position of cell (0, 0). From 4x up each cell has a 1px gap round it.
        let (ox, oy) = (c.x + e.pan.0, c.y + e.pan.1);
        let inner = if z >= 4 { z - 1 } else { z };
        let ext = if z >= 4 { 1 } else { 2 };
        if z == 1 {
            // Real size: the ship as it looks in the arena.
            if let Some((x0, y0, _, _)) = d.used_box() {
                let sh = d.shape();
                draw_shape(fb, c, &sh, ox + x0 + sh.w / 2, oy + y0 + sh.h / 2, 0, 0, &|_| true, false);
            }
        } else {
            let (cx0, cy0) = ((-e.pan.0).div_euclid(z).max(0), (-e.pan.1).div_euclid(z).max(0));
            let (cx1, cy1) = (((c.w - e.pan.0) / z).min(GRID - 1), ((c.h - e.pan.1) / z).min(GRID - 1));
            // Guide blocks are 5 cells, centred, and the centre column (the mirror line) is lit.
            let band = |k: i32| ((k - GRID / 2).abs() + 2) / 5;
            let breeches = d.breeches();
            // The empty cells a phase ring shuts in: they're what the ring is worth, and each
            // is marked with a pale dot.
            let shut: std::collections::HashSet<usize> = if d.cells.contains(&Some(Part::PhaseRing)) { crate::pg::design::ring_cells(GRID, &d.cells).into_iter().collect() } else { Default::default() };
            for cy in cy0..=cy1 {
                for cx in cx0..=cx1 {
                    let (x, y) = (ox + cx * z, oy + cy * z);
                    let p = d.get(cx, cy);
                    let breech = p.is_some_and(|p| breeches.contains(&(p, (cx, cy))));
                    let col = match p {
                        Some(p) if breech => breech_col(p),
                        Some(p) => part_col(p, 0),
                        None if cx == GRID / 2 => PANEL,
                        None if (band(cx) + band(cy)) % 2 == 0 => PANEL_DEEP,
                        None => SPACE,
                    };
                    fill_clip(fb, c, x, y, inner, inner, col);
                    if p.is_none() && shut.contains(&((cy * GRID + cx) as usize)) && c.contains(x + inner / 2, y + inner / 2) {
                        fb.pset(x + inner / 2, y + inner / 2, crate::palette::PHASE);
                    }
                    // At 8x each cell shows its part's shape too.
                    if let Some(p) = p.filter(|&p| z >= 8 && (!p.turret() || breech) && c.contains(x, y) && c.contains(x + 6, y + 6)) {
                        icons::draw(fb, Icon::Part(p), x, y, OUTLINE);
                    }
                }
            }
            for &(cx, cy) in check.loose.iter().chain(&check.buried).chain(&check.hemmed).chain(&check.slight).chain(&check.scant) {
                outline_clip(fb, c, ox + cx * z - 1, oy + cy * z - 1, inner + 2, inner + 2, BAD);
            }
        }

        // What the pointer would do.
        let frame = |fb: &mut Framebuffer, a: (i32, i32), b: (i32, i32), col: u8| {
            let (x0, y0, x1, y1) = (a.0.min(b.0), a.1.min(b.1), a.0.max(b.0), a.1.max(b.1));
            outline_clip(fb, c, ox + x0 * z - 1, oy + y0 * z - 1, (x1 - x0 + 1) * z + ext, (y1 - y0 + 1) * z + ext, col);
        };
        let ghost = |fb: &mut Framebuffer, st: &Stamp, o: (i32, i32), col: u8| {
            for sy in 0..st.h {
                for sx in 0..st.w {
                    if let Some(p) = st.get(sx, sy) {
                        let (x, y) = (ox + (o.0 + sx) * z, oy + (o.1 + sy) * z);
                        // Dithered: every other pixel, no blending.
                        for k in 0..inner * inner {
                            let (px, py) = (x + k % inner, y + k / inner);
                            if (px + py) & 1 == 0 && c.contains(px, py) {
                                fb.pset(px, py, part_col(p, 0));
                            }
                        }
                    }
                }
            }
            frame(fb, o, (o.0 + st.w - 1, o.1 + st.h - 1), col);
        };
        if let Some((right, start)) = e.drag.filter(|_| e.tool != Tool::Pencil) {
            let now = self.cell_at(m.0, m.1);
            let end = (now.0.clamp(0, GRID - 1), now.1.clamp(0, GRID - 1));
            let col = if e.tool == Tool::Copy { ACCENT } else if right { BAD } else { WHITE };
            frame(fb, start, end, col);
            if e.mirror && e.tool == Tool::Rect {
                frame(fb, (mirror_x(start.0), start.1), (mirror_x(end.0), end.1), TEXT_FAINT);
            }
        } else if let Some(cell) = self.cell_under(m.0, m.1) {
            match (&e.stamp, e.tool) {
                (Some(st), Tool::Copy) => {
                    let o = st.origin(cell);
                    ghost(fb, st, o, WHITE);
                    if let Some(mo) = self.mirror_origin(st, o) {
                        ghost(fb, &st.flipped(), mo, TEXT_FAINT);
                    }
                }
                (None, Tool::Copy) => frame(fb, cell, cell, ACCENT),
                _ => {
                    // A turret shows the whole block the click would stamp, and its mirror twin.
                    let n = if e.tool == Tool::Pencil { e.part.side() } else { 1 };
                    let a = (n - 1) / 2;
                    frame(fb, (cell.0 - a, cell.1 - a), (cell.0 + a, cell.1 + a), WHITE);
                    let mx = mirror_x(cell.0);
                    if e.mirror && mx != cell.0 {
                        frame(fb, (mx - a, cell.1 - a), (mx + a, cell.1 + a), TEXT_FAINT);
                    }
                }
            }
        }
    }

    /// The info box: what the pointer is over, or else what the current tool does.
    fn yard_info(&self, d: &Design, st: &Stats) -> String {
        let m = self.mouse;
        let at = |r: Rect| r.contains(m.0, m.1);
        let on_row = |y: i32| (COL_X..396).contains(&m.0) && (y - 1..y + 9).contains(&m.1);
        let parts = self.ed.cat.parts();
        if (0..self.lib.len()).any(|i| at(tab_rect(i))) {
            return format!("YOUR {} DESIGNS. THE ARENA'S FIRST FIGHT HAS THESE IN ITS SLOTS; ITS READY-MADE FIGHTS BRING TEAMS OF THEIR OWN. A RED NAME CAN'T FLY YET. RIGHT-CLICK ONE TO PICK THE WHOLE DESIGN UP AS A STAMP.", self.lib.len());
        }
        if let Some(i) = (0..CATS.len()).find(|&i| at(cat_rect(i))) {
            let c = CATS[i];
            if c.parts().is_empty() {
                return format!("[{}] {}: NOTHING HERE YET. ITS PARTS COME NEXT.", i + 1, c.name());
            }
            let names: Vec<&str> = c.parts().iter().map(|p| p.name()).collect();
            return format!("[{}] {}: {}. [TAB] OR THE KEY AGAIN FOR THE NEXT PART.", i + 1, c.name(), names.join(", "));
        }
        if let Some(i) = (0..parts.len()).find(|&i| at(part_rect(i, &parts))) {
            return parts[i].blurb().into();
        }
        if let Some(i) = (0..BTNS.len()).find(|&i| at(tool_rect(i))) {
            return match BTNS[i] {
                Btn::Tool(t) => self.tool_info(t),
                Btn::Turn => "[T] TURN THE STAMP YOU'RE HOLDING A QUARTER TURN CLOCKWISE, OR THE WAY THE ENGINES YOU PAINT PUSH.".into(),
                Btn::Undo => "[U] UNDO THE LAST CHANGE TO THIS DESIGN.".into(),
                Btn::Mirror => "[M] MIRROR: EVERY TOOL ALSO WORKS ON THE OTHER SIDE OF THE LIT CENTRE COLUMN.".into(),
                Btn::Clear => "CLEAR THE WHOLE DESIGN. [U] UNDO BRINGS IT BACK.".into(),
            };
        }
        if at(BTN_ZOOM_OUT) || at(BTN_ZOOM_IN) || ((6..148).contains(&m.0) && (282..297).contains(&m.1)) {
            return format!("[Q] [E] OR THE MOUSE WHEEL ZOOM IN WHOLE STEPS. 1X IS THE SHIP AT ITS REAL SIZE, AS IN THE ARENA; 2X SHOWS THE WHOLE {GRID}X{GRID} GRID; 4X AND 8X ARE FOR FINE WORK. WASD OR THE ARROWS SCROLL. THE NUMBERS ARE THE CELL UNDER THE POINTER.");
        }
        for (row, list) in [&d.first, &d.build].into_iter().enumerate() {
            if let Some(k) = (0..list.len()).find(|&k| at(chip_rect(row, k))) {
                let b = list[k];
                let what = self.lib.get(b).map_or("AN EMPTY SLOT".to_string(), |o| format!("{} ({} PX)", o.name, o.stats().cost));
                return match self.build_problem(d, b) {
                    Some(why) => format!("{}: {what}. {why} RIGHT-CLICK TAKES IT OFF.", b + 1),
                    None => format!("{}: {what}. RIGHT-CLICK TAKES IT OFF THE BUILD ORDER.", b + 1),
                };
            }
            if self.has_factory() && at(row_rect(row)) {
                return build_row_tip(row).into();
            }
        }
        if (0..self.lib.len()).any(|k| at(add_rect(k))) {
            return ADD_TIP.into();
        }
        if KIND_BTNS.iter().any(|&r| at(r)) {
            return "BUILT: A FACTORY OR YOUR KING PRINTS IT. SEEDED: YOUR KING PRINTS A SEED THAT GROWS IT OUT OF A ROCK, PIXEL BY PIXEL FROM ITS CORE; THE ROCK MUST HOLD AS MANY PIXELS AS THE SHIP. ITS SEED CORE STAYS AT THE CENTRE.".into();
        }
        if at(BTN_MATCH) {
            return "PLAY A MATCH AGAINST THE AI WITH YOUR TEAM AS YOUR DECK, OR WITH THE STARTER DECK. YOU PICK THE TEAM THE AI PLAYS.".into();
        }
        if at(BTN_TO_ARENA) {
            return "[ENTER] THE ARENA, TO WATCH YOUR DESIGNS FIGHT.".into();
        }
        if at(BTN_LIBRARY) {
            return "[L] THE LIBRARY: EVERY DRAWING AND TEAM YOU'VE KEPT, AND THE READY-MADE ONES. TAKE ONE INTO THIS SLOT, KEEP THIS ONE, OR PASS A DRAWING TO SOMEONE AS TEXT.".into();
        }
        if at(NAME_RECT) {
            return "CLICK TO RENAME. ENTER WHEN DONE.".into();
        }
        if on_row(COST_Y) {
            return "COST: EVERY PIXEL COSTS 1. A SECOND CORE COSTS +8, A THIRD +16, A FOURTH +32, AND SO ON.".into();
        }
        if on_row(FLIGHT_Y) {
            return FLIGHT_TIP.into();
        }
        if on_row(GAUGE_Y[0]) {
            return format!("TOP SPEED: {} PX/S. IT'S THRUST OVER WEIGHT: TWICE THE ENGINES FOR THE WEIGHT, TWICE THE SPEED, UP TO {}. THE HEAVIEST SHIP WITH AN ENGINE CRAWLS AT {}.", st.top_speed_px_s(), SPEED_MOST * 15 / 4, SPEED_LEAST * 15 / 4);
        }
        if on_row(GAUGE_Y[1]) {
            return format!("TURNING: {}.{} S FOR A QUARTER TURN. A SHIP TURNS ON ITS CORES AND ROTATION ENGINES, SLOWER THE MORE WEIGHT IT HAS FAR FROM ITS MIDDLE, AND NEVER SLOWER THAN {} S.", st.turn_ds() / 10, st.turn_ds() % 10, secs(SLOWEST_TURN));
        }
        if on_row(GAUGE_Y[2]) {
            let (a, b) = (st.zero_to_top_ds() / 10, st.zero_to_top_ds() % 10);
            let short = if st.thrust < st.engines * 8 { format!(", DOWN FROM {} FOR WANT OF POWER", st.engines * 8) } else { String::new() };
            return format!("THRUST {}{short}, FOR WEIGHT {} (8 PER ENGINE). THE GOLD MARK IS 1 THRUST PER WEIGHT. 0 TO TOP SPEED IN {a}.{b} S.", st.thrust, st.mass);
        }
        if on_row(GAUGE_Y[3]) {
            return format!("POWER: IT USES {} OF THE {} IT MAKES. MADE PER PIXEL: {}. WHAT A PART USES IS ON ITS OWN HOVER. SHORT OF POWER, IT ALL SLOWS IN STEP: AT 75%, GUNS RELOAD A THIRD SLOWER.", st.used, st.made, Part::power_list(true));
        }
        if let Some(k) = (0..10).find(|&k| at(tally_rect(k))) {
            if let Some((p, _)) = tally(d, st).get(k) {
                return p.blurb().into();
            }
        }
        if at(STATUS_RECT) {
            return "A SHIP NEEDS A CORE, AND ITS PIXELS MUST ALL JOIN UP EDGE TO EDGE (CORNERS DON'T COUNT). EVERY ENGINE'S EXHAUST NEEDS A STRAIGHT WAY OUT OF THE SHIP, 3 WIDE, THE OTHER WAY FROM ITS PUSH. LOOSE PIXELS AND BURIED ENGINES SHOW RED. CUT APART, EACH PIECE WITH A CORE FLIES ON.".into();
        }
        self.tool_info(self.ed.tool)
    }

    /// Why a build order entry won't print, if it won't.
    fn build_problem(&self, d: &Design, b: usize) -> Option<&'static str> {
        let o = self.lib.get(b)?;
        if o.seeded {
            Some("A SEEDED DESIGN GROWS FROM A SEED: ONLY THE KING MAKES THOSE.")
        } else if o.stats().count > d.stats().max_build() {
            Some("TOO BIG EVEN WHEN THE FACTORY IS FULLY GROWN.")
        } else if !o.check().ok() {
            Some("IT CAN'T FLY YET.")
        } else {
            None
        }
    }

    fn tool_info(&self, t: Tool) -> String {
        let p = self.ed.part.name();
        let mirror = if self.ed.mirror { " MIRROR IS ON: IT ALSO WORKS ON THE OTHER SIDE." } else { "" };
        match t {
            Tool::Pencil if self.ed.part.side() == 3 => format!("[P] PENCIL, PAINTING {p}: A CLICK STAMPS A 3X3 {p} CENTRED ON THE CELL; RIGHT-CLICK ANY PART OF ONE TO REMOVE IT WHOLE. UP IS THE FRONT.{mirror}"),
            Tool::Pencil if self.ed.part == Part::PhaseRing => format!("[P] PENCIL, PAINTING {p}: DRAW A CLOSED RING. THE EMPTY PIXELS IT SHUTS IN SHOW A DOT: THEY'RE WHAT IT'S WORTH. NOTHING ELSE MAY STAND INSIDE IT OR IN ITS WALL. UP IS THE FRONT.{mirror}"),
            Tool::Pencil if self.ed.part == Part::HeavyPlate => format!("[P] PENCIL, PAINTING {p}: IT HAS TO BE 2 THICK, AND A PIXEL OF IT THAT ISN'T SHOWS RED. THE RECTANGLE [R] PUTS DOWN A SLAB. UP IS THE FRONT.{mirror}"),
            Tool::Pencil if self.ed.part == Part::Engine => format!("[P] PENCIL, PAINTING {p}: IT PUSHES THE SHIP {} ([T] TURNS IT). LEFT-CLICK OR DRAG PAINTS, RIGHT-CLICK ERASES. UP IS THE FRONT.{mirror}", WAYS[self.ed.engine_way]),
            Tool::Pencil => format!("[P] PENCIL, PAINTING {p}. LEFT-CLICK OR DRAG PAINTS, RIGHT-CLICK ERASES. UP IS THE FRONT.{mirror}"),
            Tool::Rect => format!("[R] RECTANGLE: DRAG A BOX TO FILL IT WITH {p}; RIGHT-DRAG EMPTIES ONE. TURRETS TILE THE BOX IN 3X3 BLOCKS; HEAVY PLATE FILLS AN EVEN NUMBER OF CELLS.{mirror}"),
            Tool::Fill => format!("[F] FILL: CLICK A HOLE INSIDE THE SHIP TO FILL IT WITH {p}, OR A PATCH OF ONE PART TO SWAP IT FOR {p}. RIGHT-CLICK EMPTIES A PATCH. OPEN SPACE WON'T FILL.{mirror}"),
            Tool::Copy if self.ed.stamp.is_some() => format!("STAMP: CLICK TO PLACE IT. IT BECOMES ORDINARY PIXELS; NOTHING STAYS LINKED. [T] TURNS IT, RIGHT-CLICK PUTS IT DOWN.{mirror}"),
            Tool::Copy => "[C] COPY: DRAG A BOX OVER PART OF A SHIP TO PICK IT UP AS A STAMP, OR RIGHT-CLICK A DESIGN'S TAB TO PICK UP THE WHOLE DESIGN.".into(),
        }
    }
}

/// The gauges: (icon, segments lit out of 10, a mark, the number beside it, the lit colour).
fn gauges(st: &Stats) -> [(Icon, i32, Option<i32>, String, u8); 4] {
    let moves = st.thrust > 0;
    let speed = if moves { st.vmax * 10 / 12 } else { 0 };
    let turn = if moves && st.quarter > 0 { (SLOWEST_TURN + 1 - st.quarter) * 10 / SLOWEST_TURN } else { 0 };
    // Half the bar is 1 thrust per weight, where attack runs start.
    let push = if st.mass > 0 { (st.thrust * 5 / st.mass).min(10) } else { 0 };
    // Power is a load: how much of what the ship makes it uses. Over 100% the bar turns red.
    let load = if st.used == 0 { 0 } else if st.used > st.made { 10 } else { ((st.used * 10 + st.made - 1) / st.made.max(1)).min(10) };
    [
        (Icon::Speed, speed, None, format!("{} PX/S", st.top_speed_px_s()), ACCENT),
        (Icon::TurnRate, turn, None, if moves { format!("{}.{} S", st.turn_ds() / 10, st.turn_ds() % 10) } else { "-".into() }, ACCENT),
        (Icon::Thrust, push, Some(5), format!("{}/{}", st.thrust, st.mass), ACCENT),
        (Icon::Power, load, None, format!("{}/{}", st.used, st.made), if st.used > st.made { BAD } else { ACCENT }),
    ]
}

/// How many of each part the design has (turrets by the block), leaving out the ones it lacks.
fn tally(d: &Design, st: &Stats) -> Vec<(Part, i32)> {
    PARTS
        .iter()
        .map(|&p| {
            // Turrets count by the block, as a turret is one gun; everything else by the pixel.
            let n = match p {
                Part::Cannon => st.cannon,
                Part::Tracker => st.tracker,
                _ => d.cells.iter().filter(|c| c.map(Part::kind) == Some(p)).count() as i32,
            };
            (p, n)
        })
        .filter(|&(_, n)| n > 0)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_parts_button_shows_its_name_and_the_row_fits_under_the_canvas() {
        // Four weapons in a row once left CANNON with a swatch and no name. A fifth doesn't fit
        // the row at all, so it goes in a second one: two rows, between the categories above
        // and the tools below.
        let apart = |a: Rect, b: Rect| a.x + a.w <= b.x || b.x + b.w <= a.x || a.y + a.h <= b.y || b.y + b.h <= a.y;
        for cat in CATS {
            let parts = cat.parts();
            for (i, p) in parts.iter().enumerate() {
                let r = part_rect(i, &parts);
                assert!(r.w >= 17 + Framebuffer::text_width(p.name(), 1), "{}'s button is too narrow for its name", p.name());
                assert!(r.x >= CANVAS.x && r.x + r.w <= CANVAS.x + CANVAS.w + 2, "{}'s button runs past the canvas", p.name());
                assert!(r.y >= cat_rect(0).y + cat_rect(0).h && r.y + r.h <= tool_rect(0).y, "{}'s button is off its two rows", p.name());
                for k in 0..i {
                    assert!(apart(r, part_rect(k, &parts)), "{}'s button overlaps {}'s", p.name(), parts[k].name());
                }
            }
        }
        // (Seven weapons filled two rows and an eighth had nowhere to go, so since 2026-11-17
        // the cannon, tracker, missile and energy ball are HEAVY WEAPONS. Each of the two
        // still runs on to a second row.)
        for cat in [Cat::Weapons, Cat::Heavy] {
            let parts = cat.parts();
            assert!(parts.len() == 4 && part_rect(3, &parts).y > part_rect(0, &parts).y, "{}'s fourth part should be on the second row", cat.name());
        }
        // The rows under the canvas keep clear of each other, down to the foot of the screen.
        assert!(CANVAS.y + CANVAS.h <= cat_rect(0).y && tool_rect(0).y + tool_rect(0).h <= BTN_ZOOM_OUT.y && BTN_MATCH.y + BTN_MATCH.h <= 300);
    }

    #[test]
    fn the_whole_grid_fits_the_canvas_at_2x() {
        assert_eq!(GRID * 2, CANVAS.w);
        assert_eq!(CANVAS.w, CANVAS.h);
        for &z in &ZOOMS {
            let p = clamp_pan(z, 0);
            assert!(p <= 0 || GRID * z <= CANVAS.w);
        }
    }

    #[test]
    fn the_canvas_dots_the_empty_cells_a_phase_ring_shuts_in() {
        use crate::app::App;
        use crate::pg::design::{part_examples, ring_cells};
        let mut pg = Pg::new(None, None);
        (pg.screen, pg.help) = (crate::pg::Screen::Yard, false);
        pg.lib[5] = part_examples().into_iter().find(|d| d.name == "HALO").unwrap();
        pg.select_slot(5);
        pg.mouse = (0, 0);
        let mut fb = Framebuffer::new();
        pg.draw(&mut fb);
        let z = pg.ed.zoom;
        let inner = if z >= 4 { z - 1 } else { z };
        // The pixel in the middle of a cell of the grid.
        let mid = |pg: &Pg, c: usize| {
            // (`cell_screen` is the middle of the cell's square on the canvas, gap and all.)
            let (sx, sy) = pg.cell_screen((c as i32 % GRID, c as i32 / GRID));
            (sx - z / 2 + inner / 2, sy - z / 2 + inner / 2)
        };
        // Every cell the HALO's ring shuts in has a pale dot in its middle, and no other
        // empty cell has: not the one just outside its band, at its top left.
        let shut = ring_cells(GRID, &pg.lib[5].cells);
        let (x0, y0, _, _) = pg.lib[5].used_box().unwrap();
        assert_eq!((shut.len(), z >= 2), (225, true));
        for &c in &shut {
            let (x, y) = mid(&pg, c);
            assert!(CANVAS.contains(x, y) && fb.get(x, y) == crate::palette::PHASE, "the cell at {},{}: at {z}x the pixel at {x},{y} is {}", c as i32 % GRID, c as i32 / GRID, fb.get(x, y));
        }
        let out = mid(&pg, (y0 * GRID + x0 - 1) as usize);
        assert!(fb.get(out.0, out.1) != crate::palette::PHASE);
        // With a pixel out of the ring's side it shuts nothing in: no dots, and the status
        // line says so.
        assert_eq!(pg.lib[5].get(x0 + 9, y0 + 1), Some(Part::PhaseRing));
        pg.lib[5].set(x0 + 9, y0 + 1, None);
        pg.draw(&mut fb);
        for &c in &shut {
            let (x, y) = mid(&pg, c);
            assert!(fb.get(x, y) != crate::palette::PHASE);
        }
        let st = pg.lib[5].stats();
        assert_eq!(yard_status(&pg.lib[5].check(), st.armed(), st.power_pct(), st.order(), st.collector(), yard_field(&st)).0, "PHASE RING: ENCLOSES NOTHING");
        let st = part_examples().into_iter().find(|d| d.name == "HALO").unwrap().stats();
        assert_eq!(yard_status(&Check { cores: 1, ..Check::default() }, false, 100, st.order(), false, yard_field(&st)).0, "PHASE RING: ENCLOSES 225 PX");
    }

    #[test]
    fn every_design_opens_framed_at_a_whole_zoom() {
        // The starting library (the starter deck), then the reference fleet with its titan.
        for mut pg in [Pg::new(None, None), Pg::reference()] {
            for i in 0..pg.lib.len() {
                pg.select_slot(i);
                // (An empty slot has nothing to frame.)
                let Some((x0, y0, x1, y1)) = pg.lib[i].used_box() else { continue };
                // Both corners of the drawing are on the canvas.
                for c in [(x0, y0), (x1, y1)] {
                    let (sx, sy) = pg.cell_screen(c);
                    assert!(CANVAS.contains(sx, sy), "{} at {}x: cell {:?} lands at {sx},{sy}", pg.lib[i].name, pg.ed.zoom, c);
                }
            }
        }
        let mut pg = Pg::reference();
        pg.select_slot(0);
        assert_eq!(pg.ed.zoom, 8, "a fighter opens at 8x");
        pg.select_slot(6);
        assert_eq!(pg.ed.zoom, 2, "the titan opens at 2x");
    }

    #[test]
    fn zooming_keeps_the_cell_under_the_pointer() {
        let mut pg = Pg::new(None, None);
        pg.select_slot(4);
        let at = pg.cell_screen((50, 40));
        let before = pg.cell_at(at.0, at.1);
        pg.zoom_by(1, at);
        assert_eq!(pg.ed.zoom, 8);
        assert_eq!(pg.cell_at(at.0, at.1), before);
        pg.zoom_by(-1, at);
        assert_eq!(pg.cell_at(at.0, at.1), before);
        // At 2x the whole grid fills the canvas exactly, so there's nothing left to scroll.
        pg.zoom_by(-1, at);
        assert_eq!(pg.ed.pan, (0, 0));
    }

    #[test]
    fn tab_steps_through_the_parts_of_the_open_category_and_enter_swaps_screens() {
        use crate::app::{App, Ev};
        use crate::pg::Screen;
        let mut pg = Pg::new(None, None);
        pg.help = false;
        pg.event(Ev::Key(Key::Enter));
        assert_eq!(pg.screen, Screen::Yard);
        // Every category: Tab visits each of its parts once and comes back to the first, and
        // never leaves the category or the yard.
        for (i, &cat) in CATS.iter().enumerate() {
            let parts = cat.parts();
            if parts.is_empty() {
                continue;
            }
            // Its key opens it on its first part, coming from another category.
            pg.pick_part(Part::Core);
            pg.pick_part(Part::Hull);
            if cat != Cat::Armor {
                pg.event(Ev::Key(Key::Num(i as u8 + 1)));
            }
            assert_eq!((pg.ed.cat, pg.ed.part), (cat, parts[0]));
            for k in 1..=parts.len() {
                pg.event(Ev::Key(Key::Tab));
                assert_eq!((pg.screen, pg.ed.cat, pg.ed.part), (Screen::Yard, cat, parts[k % parts.len()]));
            }
            // With one part there is nowhere to step: it says so.
            assert_eq!(pg.notice.is_some(), parts.len() == 1, "{}", cat.name());
            pg.notice = None;
        }
        // Tab puts a held stamp down, like picking a part with the mouse.
        pg.ed.tool = Tool::Copy;
        pg.event(Ev::Key(Key::Num(1)));
        pg.ed.tool = Tool::Copy;
        pg.event(Ev::Key(Key::Tab));
        assert_eq!(pg.ed.tool, Tool::Pencil);
        // Enter goes to the arena and back; Tab still leaves the arena.
        pg.event(Ev::Key(Key::Enter));
        assert_eq!(pg.screen, Screen::Arena);
        pg.event(Ev::Key(Key::Tab));
        assert_eq!(pg.screen, Screen::Yard);
    }

    #[test]
    fn undo_steps_back_through_changes() {
        let mut pg = Pg::new(None, None);
        pg.select_slot(0);
        let start = pg.lib[0].cells.clone();
        let n = pg.lib[0].stats().count;
        pg.ed.mirror = false;
        pg.ed.tool = Tool::Rect;
        pg.press_canvas((10, 10), false);
        let end = pg.cell_screen((12, 12));
        pg.release_yard(end.0, end.1);
        assert_eq!(pg.lib[0].stats().count, n + 9);
        pg.ed.tool = Tool::Fill;
        pg.press_canvas((11, 11), true);
        assert_eq!(pg.lib[0].stats().count, n);
        pg.undo();
        assert_eq!(pg.lib[0].stats().count, n + 9);
        pg.undo();
        assert_eq!(pg.lib[0].cells, start);
    }

    #[test]
    fn t_turns_the_engine_you_paint_and_its_mirror_twin_pushes_the_mirrored_way() {
        let mut pg = Pg::new(None, None);
        pg.select_slot(5);
        pg.press_btn(Btn::Clear);
        pg.ed.part = Part::Engine;
        pg.ed.mirror = true;
        let at = |pg: &Pg, x: i32, y: i32| pg.lib[5].get(x, y);
        // As chosen, an engine pushes its ship ahead, and so does its twin.
        pg.paint((20, 30), false);
        assert_eq!((at(&pg, 20, 30), at(&pg, mirror_x(20), 30)), (Some(Part::Engine), Some(Part::Engine)));
        // [T] turns what the pencil paints: to the right, and its twin across the ship pushes
        // to the left. The turn button does the same, and it's lit while engines are chosen.
        pg.key_yard(Key::Letter('T'));
        assert_eq!(pg.ed.engine_way, 1);
        pg.paint((20, 32), false);
        assert_eq!((at(&pg, 20, 32), at(&pg, mirror_x(20), 32)), (Some(Part::EngineRight), Some(Part::EngineLeft)));
        pg.press_btn(Btn::Turn);
        pg.paint((20, 34), false);
        assert_eq!((at(&pg, 20, 34), at(&pg, mirror_x(20), 34)), (Some(Part::EngineAstern), Some(Part::EngineAstern)));
        pg.key_yard(Key::Letter('T'));
        pg.paint((20, 36), false);
        assert_eq!((at(&pg, 20, 36), at(&pg, mirror_x(20), 36)), (Some(Part::EngineLeft), Some(Part::EngineRight)));
        // Round again to ahead.
        pg.key_yard(Key::Letter('T'));
        assert_eq!(pg.ed.engine_way, 0);
        // The box and the fill put down the turned engine too.
        pg.key_yard(Key::Letter('T'));
        pg.fill_box_at((10, 50), (11, 50), false);
        assert_eq!((at(&pg, 10, 50), at(&pg, mirror_x(10), 50)), (Some(Part::EngineRight), Some(Part::EngineLeft)));
        // However they're turned they're all engines to the tally, and they save and load as
        // they are.
        let st = pg.lib[5].stats();
        assert_eq!(tally(&pg.lib[5], &st).iter().find(|t| t.0 == Part::Engine).map(|t| t.1), Some(12));
        assert_eq!(Design::from_line(&pg.lib[5].to_line()).map(|d| d.cells), Some(pg.lib[5].cells.clone()));
        // With another part chosen and no stamp in hand, there's nothing to turn.
        pg.ed.part = Part::Hull;
        pg.key_yard(Key::Letter('T'));
        assert_eq!(pg.ed.engine_way, 1);
    }

    #[test]
    fn heavy_plate_is_painted_a_pixel_at_a_time_and_has_to_be_two_thick() {
        let mut pg = Pg::new(None, None);
        pg.select_slot(5);
        pg.press_btn(Btn::Clear);
        pg.ed.part = Part::HeavyPlate;
        pg.ed.mirror = true;
        let heavy = |pg: &Pg| (0..GRID * GRID).filter(|&i| pg.lib[5].get(i % GRID, i / GRID) == Some(Part::HeavyPlate)).count();
        // One click, one pixel and its mirror twin: no dominoes, and nothing for [T] to turn.
        pg.paint((20, 30), false);
        assert_eq!((heavy(&pg), pg.lib[5].get(mirror_x(20), 30)), (2, Some(Part::HeavyPlate)));
        pg.key_yard(Key::Letter('T'));
        assert!(pg.notice.as_ref().is_some_and(|(s, _)| s.starts_with("NO STAMP TO TURN")), "{:?}", pg.notice);
        // Alone they're too thin, and the status line says so.
        assert_eq!(pg.lib[5].thin_plate().len(), 2);
        // A stroke two thick is fine; so is a wall three wide across the middle column, which
        // a 2x2 block could never be laid on.
        for (x, y) in [(21, 30), (20, 31), (21, 31)] {
            pg.paint((x, y), false);
        }
        assert!(pg.lib[5].thin_plate().is_empty());
        pg.ed.mirror = false;
        let mid = GRID / 2;
        for y in 50..52 {
            for x in mid - 1..=mid + 1 {
                pg.paint((x, y), false);
            }
        }
        assert!(pg.lib[5].thin_plate().is_empty(), "{:?}", pg.lib[5].thin_plate());
        // An L two thick is fine too. A tail one pixel wide off it isn't: just the tail.
        for (x, y) in [(mid + 2, 50), (mid + 2, 51), (mid + 3, 50)] {
            pg.paint((x, y), false);
        }
        assert_eq!(pg.lib[5].thin_plate(), vec![(mid + 3, 50)]);
        // A right-click takes up one pixel, and what that leaves too thin shows.
        pg.paint((mid + 3, 50), true);
        pg.paint((mid - 1, 50), true);
        assert_eq!(pg.lib[5].thin_plate(), vec![(mid - 1, 51)]);
        // A tracker's twin is centred on the mirrored cell, as a cannon's is.
        pg.ed.mirror = true;
        pg.ed.part = Part::Tracker;
        pg.paint((30, 40), false);
        assert_eq!(pg.lib[5].blocks(Part::Tracker).0, vec![(29, 39), (mirror_x(31), 39)]);
    }

    #[test]
    fn a_mirrored_stamp_lands_flipped_on_the_other_side() {
        let mut pg = Pg::new(None, None);
        pg.select_slot(5);
        pg.press_btn(Btn::Clear);
        let pod = Design::from_rows("P", &["gh", "hc"]).as_stamp();
        pg.ed.mirror = true;
        pg.place(&pod, (20, 30));
        let o = pod.origin((20, 30));
        assert_eq!(pg.lib[5].get(o.0, o.1), Some(Part::Flak));
        assert_eq!(pg.lib[5].get(mirror_x(o.0), o.1), Some(Part::Flak));
        assert_eq!(pg.lib[5].get(mirror_x(o.0 + 1), o.1 + 1), Some(Part::Core));
    }

    #[test]
    fn the_readouts_fit_their_column() {
        for d in defaults() {
            let st = d.stats();
            for (_, segs, _, value, _) in gauges(&st) {
                assert!((0..=10).contains(&segs), "{}", d.name);
                assert!(COL_X + 76 + Framebuffer::text_width(&value, 1) <= 400, "{}: {value}", d.name);
            }
            assert!(tally(&d, &st).len() <= 10);
            let cost = format!("COST {} PX (+{} CORES)", st.cost, st.cost - st.count);
            assert!(COL_X + Framebuffer::text_width(&cost, 1) <= 400, "{cost}");
        }
    }

    #[test]
    fn what_a_designs_ships_do_and_how_they_fly_are_nothing_to_set_in_the_yard() {
        use crate::app::{App, Ev};
        let mut pg = Pg::new(None, None);
        pg.lib = starter_deck();
        pg.screen = crate::pg::Screen::Yard;
        pg.help = false;
        pg.select_slot(0);
        let press = |pg: &mut Pg, r: Rect| pg.event(Ev::Press { x: r.x + r.w / 2, y: r.y + r.h / 2, right: false });
        let key = |pg: &mut Pg, c: char| pg.event(Ev::Key(Key::Letter(c)));
        let d = |pg: &Pg| pg.lib[pg.slot].clone();
        // A design has no order, no way of flying and no group size to set. The keys that chose
        // them (O, Y and G), and clicks along the rows where their buttons were, change nothing.
        let before = d(&pg).to_line();
        key(&mut pg, 'O');
        key(&mut pg, 'Y');
        key(&mut pg, 'G');
        for y in [FLIGHT_Y + 3, 124, 136] {
            for x in (210..396).step_by(6) {
                pg.event(Ev::Press { x, y, right: false });
                pg.event(Ev::Release { x, y, right: false });
            }
        }
        assert_eq!(d(&pg).to_line(), before);
        // BUILT and SEEDED are the buttons left in that column, and they still work.
        press(&mut pg, KIND_BTNS[1]);
        assert!(d(&pg).seeded);
        press(&mut pg, KIND_BTNS[0]);
        assert!(!d(&pg).seeded);
        // They and the build row sit between the status line and the info box.
        assert!(KIND_BTNS[0].y >= STATUS_RECT.y + STATUS_RECT.h && BUILD_Y >= KIND_BTNS[0].y + KIND_BTNS[0].h && ROUND_Y >= BUILD_Y + 11 && ADD_Y >= ROUND_Y + 11 && ADD_Y + 12 <= INFO_RECT.y && INFO_RECT.y + INFO_RECT.h <= 298, "the column's rows overlap");
        // The row says whether the drawing flies, and its hover text what a ship does in a fight.
        pg.mouse = (COL_X + 20, FLIGHT_Y + 2);
        let st = d(&pg).stats();
        assert!(pg.yard_info(&d(&pg), &st).starts_with("IN A FIGHT A SHIP HOLDS AT THE REACH OF ITS GUNS"));
    }

    #[test]
    fn a_build_order_has_a_start_row_and_a_repeat_row() {
        use crate::app::{App, Ev};
        let mut pg = Pg::new(None, None);
        pg.screen = crate::pg::Screen::Yard;
        pg.help = false;
        pg.select_slot(4);
        let press = |pg: &mut Pg, r: Rect, right: bool| pg.event(Ev::Press { x: r.x + r.w / 2, y: r.y + r.h / 2, right });
        let rows = |pg: &Pg| (pg.lib[4].first.clone(), pg.lib[4].build.clone());
        assert_eq!(rows(&pg), (vec![], vec![3, 3, 0, 0, 1, 2]));
        // ADD adds to the repeat row until the start row's name is clicked.
        press(&mut pg, add_rect(7), false);
        assert_eq!(rows(&pg), (vec![], vec![3, 3, 0, 0, 1, 2, 7]));
        press(&mut pg, row_rect(0), false);
        assert!(pg.ed.add_to_start);
        press(&mut pg, add_rect(3), false);
        press(&mut pg, add_rect(7), false);
        assert_eq!(rows(&pg), (vec![3, 7], vec![3, 3, 0, 0, 1, 2, 7]));
        // And back.
        press(&mut pg, row_rect(1), false);
        press(&mut pg, add_rect(0), false);
        assert_eq!(rows(&pg), (vec![3, 7], vec![3, 3, 0, 0, 1, 2, 7, 0]));
        // A right-click takes an entry off its own row, and a right-click on a row's name
        // changes nothing.
        press(&mut pg, chip_rect(0, 0), true);
        press(&mut pg, chip_rect(1, 6), true);
        press(&mut pg, row_rect(0), true);
        assert_eq!(rows(&pg), (vec![7], vec![3, 3, 0, 0, 1, 2, 0]));
        assert!(!pg.ed.add_to_start);
        // It's saved with the design, and a row holds sixteen: the seventeenth is refused.
        assert!(pg.lib[4].to_line().ends_with("|SEEDED|7>3,3,0,0,1,2,0"), "{}", pg.lib[4].to_line());
        press(&mut pg, row_rect(0), false);
        for _ in 0..20 {
            press(&mut pg, add_rect(3), false);
        }
        assert_eq!((pg.lib[4].first.len(), BUILD_MAX), (16, 16));
        assert!(pg.notice.as_ref().is_some_and(|(s, _)| s == "A ROW OF THE BUILD ORDER HOLDS 16 AT MOST"), "{:?}", pg.notice);
        // Sixteen fit across, clear of each other, the rows' names and the screen's edge; the
        // rows are clear of each other, of the ADD buttons and of the text box.
        for row in 0..2 {
            for k in 0..BUILD_MAX {
                let c = chip_rect(row, k);
                assert!(c.x >= row_rect(row).x + row_rect(row).w && c.x + c.w <= 398 && (k == 0 || chip_rect(row, k - 1).x + chip_rect(row, k - 1).w <= c.x), "chip {k} of row {row}");
            }
            assert!(Framebuffer::text_width(ROW_NAMES[row], 1) + 2 <= row_rect(row).w);
        }
        assert!(chip_rect(0, 0).y + chip_rect(0, 0).h <= chip_rect(1, 0).y && chip_rect(1, 0).y + chip_rect(1, 0).h <= add_rect(0).y && add_rect(0).y + add_rect(0).h <= INFO_RECT.y);
        // A design with no factory pixels has no rows to click.
        pg.select_slot(0);
        press(&mut pg, row_rect(1), false);
        assert!(pg.ed.add_to_start && pg.lib[0].first.is_empty() && pg.lib[0].build.is_empty());
    }

    #[test]
    fn every_hover_text_in_the_yard_fits_its_box() {
        // Move the pointer over the whole screen for a few designs, then try every tool, stamp
        // and part over the canvas, and check the info box never has more to say than it has
        // lines for.
        let cols = ((INFO_RECT.w - 8) / 6) as usize;
        let rows = ((INFO_RECT.h - 4) / 9) as usize;
        let mut pg = Pg::new(None, None);
        pg.lib = starter_deck();
        let mut long: Vec<(usize, String)> = Vec::new();
        let mut look = |pg: &Pg, st: &Stats| {
            let info = pg.yard_info(&pg.lib[pg.slot], st);
            let n = wrap(&info, cols).len();
            if n > rows && !long.iter().any(|(_, t)| *t == info) {
                long.push((n, info));
            }
        };
        for slot in [0, 4, 5, 6] {
            pg.select_slot(slot);
            let st = pg.lib[slot].stats();
            for y in (0..300).step_by(2) {
                for x in (0..400).step_by(2) {
                    pg.mouse = (x, y);
                    look(&pg, &st);
                }
            }
            // The parts row and the default text change with the category, tool and stamp.
            pg.mouse = canvas_centre();
            for tool in [Tool::Pencil, Tool::Rect, Tool::Fill, Tool::Copy] {
                for stamp in [false, true] {
                    pg.ed.tool = tool;
                    pg.ed.stamp = (stamp && tool == Tool::Copy).then(|| pg.lib[0].as_stamp());
                    for p in PARTS {
                        pg.ed.part = p;
                        pg.ed.cat = Cat::of(p);
                        for mirror in [false, true] {
                            pg.ed.mirror = mirror;
                            look(&pg, &st);
                        }
                    }
                }
            }
            for &cat in &CATS {
                pg.ed.cat = cat;
                for i in 0..cat.parts().len() {
                    let r = part_rect(i, &cat.parts());
                    pg.mouse = (r.x + 2, r.y + 2);
                    look(&pg, &st);
                }
            }
        }
        long.sort();
        let list: Vec<String> = long.iter().map(|(n, t)| format!("{n} lines: {t}")).collect();
        assert!(long.is_empty(), "the box has {rows} lines; these need more:\n{}", list.join("\n"));
    }

    #[test]
    fn nine_tabs_and_nine_add_buttons_fit_the_screen() {
        for i in 0..SLOTS {
            let (t, a) = (tab_rect(i), add_rect(i));
            assert!(t.x >= 0 && t.x + t.w <= 400, "tab {} is off the screen", i + 1);
            assert!(i == 0 || tab_rect(i - 1).x + tab_rect(i - 1).w <= t.x, "tab {} overlaps the one before", i + 1);
            assert!(t.w >= Framebuffer::text_width(&"W".repeat(TAB_NAME), 1), "a tab is too narrow for the name it shows");
            assert!(a.x + a.w <= 398 && (i == 0 || add_rect(i - 1).x + add_rect(i - 1).w <= a.x), "add button {} doesn't fit", i + 1);
        }
        // The name being typed, with its cursor, stays clear of the LIBRARY button beside it.
        assert!(NAME_RECT.x + 2 + Framebuffer::text_width(&"W".repeat(11), 2) <= BTN_LIBRARY.x + 1);
        assert!(BTN_LIBRARY.w >= Framebuffer::text_width("LIBRARY", 1) + 4 && BTN_LIBRARY.x + BTN_LIBRARY.w <= 398);
    }
}
