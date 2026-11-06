//! Placeholder sprites authored in code. Every grounded sprite has its feet (or base) on the
//! bottom row, and everything gets a 1px dark hull from `outline()`.

use crate::palette::*;

#[derive(Clone)]
pub struct Sprite {
    pub w: i32,
    pub h: i32,
    pub px: Vec<u8>, // 0 = transparent
}

impl Sprite {
    pub(crate) fn blank(w: i32, h: i32) -> Self {
        Self { w, h, px: vec![0; (w * h) as usize] }
    }

    fn from_art(rows: &[&str], key: fn(char) -> u8) -> Self {
        let h = rows.len() as i32;
        let w = rows[0].len() as i32;
        let mut px = Vec::with_capacity((w * h) as usize);
        for row in rows {
            assert_eq!(row.len() as i32, w, "ragged sprite row: {row}");
            px.extend(row.chars().map(key));
        }
        Self { w, h, px }
    }

    pub(crate) fn set(&mut self, x: i32, y: i32, c: u8) {
        if x >= 0 && y >= 0 && x < self.w && y < self.h {
            self.px[(y * self.w + x) as usize] = c;
        }
    }

    /// Adds a 1px dark hull (index 1) around every colored pixel.
    pub(crate) fn outline(mut self) -> Self {
        let src = self.px.clone();
        let (w, h) = (self.w, self.h);
        let body = |x: i32, y: i32| x >= 0 && y >= 0 && x < w && y < h && src[(y * w + x) as usize] > 1;
        for y in 0..h {
            for x in 0..w {
                let i = (y * w + x) as usize;
                if src[i] == 0 && (body(x - 1, y) || body(x + 1, y) || body(x, y - 1) || body(x, y + 1)) {
                    self.px[i] = OUTLINE;
                }
            }
        }
        self
    }

    /// Breathing frame: drop one row and pad the top, so the feet stay planted.
    fn squash(&self, row: i32) -> Self {
        let mut s = Sprite::blank(self.w, self.h);
        for y in 0..self.h {
            let src_y = if y <= row { y - 1 } else { y };
            if src_y >= 0 {
                for x in 0..self.w {
                    s.px[(y * self.w + x) as usize] = self.px[(src_y * self.w + x) as usize];
                }
            }
        }
        s
    }
}

fn monster_key(c: char) -> u8 {
    match c {
        '1' => GOBLIN,
        '2' => GOBLIN + 1,
        '3' => GOBLIN + 2,
        '4' => CYCLOPS,
        '5' => CYCLOPS + 1,
        '6' => CYCLOPS + 2,
        'w' => EYE,
        'k' => OUTLINE,
        't' | 'h' => TOOTH,
        'b' => WOOD + 1,
        _ => 0,
    }
}

// Both face right; feet rest on row 15.
const GOBLIN_ART: [&str; 16] = [
    "................",
    "................",
    "................",
    "................",
    ".1............1.",
    ".12...2222...21.",
    "..122222222221..",
    "...23ww23ww322..",
    "...23wk23wk3222.",
    "...2233333332...",
    "....2t2tt2t22...",
    ".....222222.....",
    "....12bbbb21....",
    "....12222221....",
    ".....2....2.....",
    "....11....11....",
];
const GOBLIN_LEGS_B: [&str; 2] = ["......2..2......", ".....11..11....."];

const CYCLOPS_ART: [&str; 16] = [
    "................",
    ".......h........",
    "......hh........",
    ".....444444.....",
    "....45555554....",
    "...4555555554...",
    "...455wwww554...",
    "...45wwwkkw554..",
    "...455wwww5554..",
    "...45555555554..",
    "....45tttt554...",
    ".....4555554....",
    "...44555555544..",
    "...4.5555555.4..",
    ".....55...55....",
    "....444...444...",
];
const CYCLOPS_LEGS_B: [&str; 2] = ["......55.55.....", ".....444.444...."];

/// A goblin's or a cyclops's frames with what it has on drawn in: each piece by its kind, in its
/// color, where it's worn (gloves on the hands, a belt at the waist, sandals on the feet).
pub(crate) fn dressed_frames(goblin: bool, look: &[(&str, u8)]) -> [Sprite; 4] {
    let (art, legs_b) = if goblin { (&GOBLIN_ART, &GOBLIN_LEGS_B) } else { (&CYCLOPS_ART, &CYCLOPS_LEGS_B) };
    let mut b = art.to_vec();
    b[14] = legs_b[0];
    b[15] = legs_b[1];
    let (mut idle, mut walk_b) = (Sprite::from_art(art, monster_key), Sprite::from_art(&b, monster_key));
    for &(kind, c) in look {
        wear(&mut idle, goblin, kind, c);
        wear(&mut walk_b, goblin, kind, c);
    }
    let breath = idle.squash(11);
    [idle.clone().outline(), breath.outline(), idle.outline(), walk_b.outline()]
}

/// Draws a piece that looks like `kind` onto a monster's raw art (before its outline), in color
/// `c`.
fn wear(s: &mut Sprite, goblin: bool, kind: &str, c: u8) {
    let body = |s: &Sprite, x: i32, y: i32| x >= 0 && y >= 0 && x < s.w && y < s.h && s.px[(y * s.w + x) as usize] > 1;
    // Recolors the body's own pixels in a row from x0 to x1.
    let band = |s: &mut Sprite, y: i32, x0: i32, x1: i32| {
        for x in x0..=x1 {
            if body(s, x, y) {
                s.set(x, y, c);
            }
        }
    };
    let dots: &[(i32, i32)] = match (kind, goblin) {
        ("gloves", true) => &[(4, 12), (4, 13), (11, 12), (11, 13)],
        ("gloves", false) => &[(3, 12), (3, 13), (13, 12), (13, 13)],
        ("charm", true) => &[(7, 11), (8, 11)],
        ("charm", false) => &[(8, 11)],
        ("earrings", true) => &[(1, 6), (14, 6)],
        ("earrings", false) => &[(2, 8), (14, 8)],
        ("spectacles", true) => &[(4, 7), (7, 7), (8, 7), (11, 7)],
        ("spectacles", false) => &[(4, 7), (11, 7)],
        _ => &[],
    };
    for &(x, y) in dots {
        s.set(x, y, c);
    }
    match (kind, goblin) {
        ("shoes", _) => band(s, 15, 0, 15),
        ("belt", true) => band(s, 12, 5, 10),
        ("belt", false) => band(s, 13, 5, 11),
        ("headband", true) => band(s, 6, 3, 12),
        ("headband", false) => band(s, 4, 4, 11),
        ("cloak", true) => band(s, 13, 4, 11),
        ("cloak", false) => {
            band(s, 12, 3, 13);
            band(s, 13, 3, 13);
        }
        // An apron down the front, below the chin.
        ("apron", true) => {
            for y in 11..=13 {
                band(s, y, 6, 9);
            }
        }
        ("apron", false) => {
            for y in 12..=14 {
                band(s, y, 6, 10);
            }
        }
        // A belt at the waist with a pouch hanging from it.
        ("tool_belt", goblin) => {
            let (row, x0, x1, pouch) = if goblin { (12, 4, 11, [(9, 13), (10, 13)]) } else { (13, 4, 12, [(8, 14), (9, 14)]) };
            band(s, row, x0, x1);
            for (x, y) in pouch {
                s.set(x, y, shift(c, -1));
            }
        }
        // A pack on the back, behind the body (it faces right), its bottom in shadow, and a
        // strap over the chest.
        ("backpack", goblin) => {
            let ((x0, x1, y0, y1), strap) = if goblin { ((2, 4, 10, 13), [(5, 11), (5, 12)]) } else { ((1, 3, 9, 12), [(6, 12), (6, 13)]) };
            for y in y0..=y1 {
                for x in x0..=x1 {
                    if !body(s, x, y) {
                        s.set(x, y, if y == y1 { shift(c, -1) } else { c });
                    }
                }
            }
            for (x, y) in strap {
                s.set(x, y, shift(c, -1));
            }
        }
        // A cap over the brow, its band in shadow, a red feather up and back from its crown.
        // A cyclops's horn shows through.
        ("cap", goblin) => {
            let (crown, band_row, feather): (&[(i32, i32, i32)], (i32, i32, i32), [(i32, i32); 3]) =
                if goblin { (&[(4, 6, 9), (5, 5, 10)], (6, 4, 12), [(5, 3), (4, 2), (3, 1)]) } else { (&[(2, 5, 10), (3, 5, 10)], (4, 4, 11), [(4, 1), (3, 0), (4, 0)]) };
            for &(y, x0, x1) in crown {
                for x in x0..=x1 {
                    if s.px[(y * s.w + x) as usize] != TOOTH {
                        s.set(x, y, c);
                    }
                }
            }
            let (y, x0, x1) = band_row;
            for x in x0..=x1 {
                if body(s, x, y) {
                    s.set(x, y, shift(c, -1));
                }
            }
            for (x, y) in feather {
                s.set(x, y, THREAD + 1);
            }
        }
        _ => {}
    }
}

/// Frames: idle, idle breathing (squashed), walk A, walk B.
fn monster_frames(art: &[&str; 16], legs_b: &[&str; 2]) -> [Sprite; 4] {
    let mut b = art.to_vec();
    b[14] = legs_b[0];
    b[15] = legs_b[1];
    let idle = Sprite::from_art(art, monster_key);
    let breath = idle.squash(11);
    let walk_b = Sprite::from_art(&b, monster_key);
    [idle.clone().outline(), breath.outline(), idle.outline(), walk_b.outline()]
}

pub(crate) fn circle_fill(s: &mut Sprite, cx: i32, cy: i32, r: i32, ramp: [u8; 3]) {
    for y in cy - r..=cy + r {
        for x in cx - r..=cx + r {
            let (dx, dy) = (x - cx, y - cy);
            if dx * dx + dy * dy <= r * r + r / 2 {
                let shade = dx + dy;
                let c = if shade <= -r + 1 { ramp[2] } else if shade >= r - 1 { ramp[0] } else { ramp[1] };
                s.set(x, y, c);
            }
        }
    }
}

/// A fluffy boll: separate lobes (so the silhouette is bumpy, not a pebble) sitting in a
/// green calyx. Back lobes first, so front lobes cast a crease where they overlap.
fn make_cotton() -> Sprite {
    let mut s = Sprite::blank(16, 15);
    for (cx, cy) in [(5, 5), (10, 5), (7, 7), (4, 9), (11, 9)] {
        let r = 3;
        for y in cy - r..=cy + r {
            for x in cx - r..=cx + r {
                let (dx, dy) = (x - cx, y - cy);
                if dx * dx + dy * dy <= r * r + 1 {
                    let shade = dx + dy;
                    let c = if shade <= 0 { COTTON + 2 } else if shade <= 2 { COTTON + 1 } else { COTTON };
                    s.set(x, y, c);
                }
            }
        }
    }
    for x in 4..=11 {
        s.set(x, 13, if x == 4 || x == 11 { LEAF } else { LEAF + 1 });
    }
    for x in [5, 6, 9, 10] {
        s.set(x, 14, LEAF);
    }
    s.outline()
}

/// A spool: wooden flanges around wound red thread.
fn make_spool() -> Sprite {
    let mut s = Sprite::blank(12, 14);
    for x in 1..=10 {
        for (y, c) in [(1, WOOD + 3), (2, WOOD + 1), (12, WOOD + 2), (13, WOOD)] {
            s.set(x, y, c);
        }
    }
    for y in 3..=11 {
        for x in 2..=9 {
            let c = if x == 2 || x == 9 { THREAD } else if y % 2 == 0 { THREAD + 1 } else { THREAD + 2 };
            s.set(x, y, c);
        }
    }
    s.outline()
}

/// Cotton plant growth stages: sprout, leafy bush, bush with open bolls.
fn make_plant(stage: u8) -> Sprite {
    let mut s = Sprite::blank(16, 16);
    let leaf = [LEAF, LEAF + 1, LEAF + 2];
    match stage {
        0 => {
            for y in 12..=15 {
                s.set(8, y, LEAF);
            }
            for (x, y) in [(6, 11), (7, 12), (9, 11), (10, 12), (5, 11), (11, 11)] {
                s.set(x, y, LEAF + 2);
            }
        }
        _ => {
            let (rx, ry, cy) = if stage == 1 { (4, 4, 11) } else { (6, 5, 10) };
            for y in cy - ry..=15 {
                for x in 8 - rx..=8 + rx {
                    let (dx, dy) = (x - 8, (y - cy).min(ry));
                    if dx * dx * ry * ry + dy * dy * rx * rx <= rx * rx * ry * ry {
                        let shade = dx + (y - cy);
                        let c = if shade <= -2 { leaf[2] } else if shade >= 3 { leaf[0] } else { leaf[1] };
                        s.set(x, y, c);
                    }
                }
            }
            if stage == 2 {
                for (x, y) in [(5, 7), (10, 6), (8, 9), (4, 11), (11, 10), (7, 12)] {
                    s.set(x, y, COTTON + 2);
                    s.set(x + 1, y, COTTON + 2);
                    s.set(x, y + 1, COTTON + 1);
                    s.set(x + 1, y + 1, COTTON + 1);
                }
            }
        }
    }
    s.outline()
}

/// Spinning wheel on a stand; `frame` turns the spokes by 45 degrees.
fn make_wheel(frame: u8) -> Sprite {
    let mut s = Sprite::blank(16, 16);
    let (cx, cy, r) = (8, 7, 5);
    let spokes: [(i32, i32); 2] = if frame == 0 { [(1, 0), (0, 1)] } else { [(1, 1), (1, -1)] };
    for (dx, dy) in spokes {
        for k in -(r - 1)..=(r - 1) {
            let k = if dx != 0 && dy != 0 { k * 3 / 4 } else { k };
            s.set(cx + dx * k, cy + dy * k, WOOD + 2);
        }
    }
    // Two-pixel rim, lit on top.
    for (rad, top, bottom) in [(r, SPARK + 1, WOOD + 3), (r - 1, WOOD + 3, WOOD + 2)] {
        for a in 0..64 {
            let t = a as f32 / 64.0 * std::f32::consts::TAU;
            let x = cx + (t.cos() * rad as f32).round() as i32;
            let y = cy + (t.sin() * rad as f32).round() as i32;
            s.set(x, y, if y < cy { top } else { bottom });
        }
    }
    s.set(cx, cy, SPARK);
    for (x0, x1) in [(5, 3), (11, 13)] {
        for y in 12..=15 {
            let x = x0 + (x1 - x0) * (y - 12) / 3;
            s.set(x, y, WOOD + 1);
        }
    }
    for x in 3..=13 {
        s.set(x, 15, WOOD);
    }
    for y in 9..=11 {
        s.set(14, y, THREAD + 1);
    }
    s.outline()
}

fn make_flag(color: u8) -> Sprite {
    let mut s = Sprite::blank(9, 14);
    for y in 1..14 {
        s.set(1, y, WOOD + 3);
    }
    for (y, len) in [(1, 6), (2, 5), (3, 4), (4, 3), (5, 2)] {
        for x in 2..2 + len {
            s.set(x, y, color);
        }
    }
    s.outline()
}

fn make_loop_icon() -> Sprite {
    let mut s = Sprite::blank(9, 9);
    for (x, y) in [(3, 1), (4, 1), (5, 1), (2, 2), (1, 3), (1, 4), (1, 5), (2, 6), (3, 7), (4, 7), (5, 7), (6, 6), (7, 5), (7, 4)] {
        s.set(x, y, LOOP + 1);
    }
    for (x, y) in [(6, 2), (7, 2), (7, 3), (6, 3)] {
        s.set(x, y, LOOP);
    }
    s.outline()
}

fn make_marker() -> Sprite {
    let mut s = Sprite::blank(7, 5);
    for (y, x0, x1) in [(1, 1, 5), (2, 2, 4), (3, 3, 3)] {
        for x in x0..=x1 {
            s.set(x, y, SPARK + 2);
        }
    }
    s.outline()
}

fn make_icon(kind_cotton: bool) -> Sprite {
    let mut s = Sprite::blank(8, 8);
    if kind_cotton {
        circle_fill(&mut s, 3, 3, 2, [COTTON, COTTON + 1, COTTON + 2]);
        circle_fill(&mut s, 5, 4, 1, [COTTON, COTTON + 1, COTTON + 2]);
        s.set(3, 6, WOOD + 1);
        s.set(4, 6, WOOD + 1);
    } else {
        // The hank of thread as it lies in the world, small: a white ball lit from the top
        // left, wound with two diagonal strands.
        for y in 1..=6 {
            for x in 1..=6 {
                if (x == 1 || x == 6) && (y == 1 || y == 6) {
                    continue;
                }
                let strand = x - y == 1 || x - y == -2;
                let c = if strand || x + y >= 11 { COTTON } else if x + y <= 4 { COTTON + 2 } else { COTTON + 1 };
                s.set(x, y, c);
            }
        }
    }
    s.outline()
}

/// Carded cotton rolled into a long sliver.
fn make_slivers() -> Sprite {
    let mut s = Sprite::blank(16, 9);
    for y in 2..=7 {
        for x in 1..=14 {
            let edge = y == 2 || y == 7;
            let c = if y <= 3 { COTTON + 2 } else if y >= 6 { COTTON } else if (x + y) % 5 == 0 { COTTON } else { COTTON + 1 };
            if !(edge && (x == 1 || x == 14)) {
                s.set(x, y, c);
            }
        }
    }
    s.set(15, 5, COTTON + 1);
    s.set(15, 6, COTTON);
    for x in 3..=12 {
        s.set(x, 8, COTTON);
    }
    s.outline()
}

/// A hank of spun thread.
fn make_thread() -> Sprite {
    let mut s = Sprite::blank(13, 12);
    circle_fill(&mut s, 6, 6, 5, [COTTON, COTTON + 1, COTTON + 2]);
    for (x0, y0, x1, y1) in [(2, 4, 9, 9), (4, 2, 10, 8), (3, 7, 7, 10)] {
        let (mut x, mut y) = (x0, y0);
        while x <= x1 && y <= y1 {
            s.set(x, y, COTTON);
            x += 1;
            y += 1;
        }
    }
    for x in 4..=8 {
        s.set(x, 11, COTTON);
    }
    s.outline()
}

/// A folded bolt of cloth.
fn make_fabric() -> Sprite {
    let mut s = Sprite::blank(16, 11);
    for y in 1..=10 {
        for x in 1..=14 {
            let c = if y == 1 || x == 1 { COTTON + 2 } else if y % 3 == 0 { COTTON } else { COTTON + 1 };
            s.set(x, y, c);
        }
    }
    for y in 2..=9 {
        s.set(11, y, COTTON);
    }
    s.outline()
}

/// An outdoor workbench: a plank top on two legs.
fn make_bench() -> Sprite {
    let mut s = Sprite::blank(16, 12);
    for x in 1..=14 {
        s.set(x, 2, WOOD + 3);
        s.set(x, 3, WOOD + 2);
        s.set(x, 4, WOOD + 1);
    }
    for y in 5..=11 {
        for x in [2, 3, 12, 13] {
            s.set(x, y, if x % 2 == 0 { WOOD + 1 } else { WOOD });
        }
    }
    s.outline()
}

/// A waist loom: a frame with the warp stretched across it.
fn make_loom() -> Sprite {
    let mut s = Sprite::blank(16, 16);
    for y in 2..=15 {
        s.set(1, y, WOOD + 2);
        s.set(14, y, WOOD + 1);
    }
    for x in 1..=14 {
        s.set(x, 2, WOOD + 3);
        s.set(x, 12, WOOD + 2);
    }
    for x in (3..=12).step_by(2) {
        for y in 3..=11 {
            s.set(x, y, COTTON + 1);
        }
    }
    for x in 2..=13 {
        s.set(x, 7, WOOD + 3);
    }
    s.outline()
}

/// A hand roller gin set down on the floor: two rollers between posts on a base plank, a crank
/// on the right, a tuft of cotton waiting at the top.
fn make_gin() -> Sprite {
    let mut s = Sprite::blank(16, 12);
    for x in 1..=14 {
        s.set(x, 10, WOOD + 2);
        s.set(x, 11, WOOD);
    }
    for y in 2..=9 {
        s.set(2, y, if y == 2 { WOOD + 3 } else { WOOD + 1 });
        s.set(11, y, if y == 2 { WOOD + 3 } else { WOOD + 1 });
    }
    for x in 3..=10 {
        for (y, c) in [(4, WOOD + 3), (5, WOOD + 2), (7, WOOD + 3), (8, WOOD + 2)] {
            s.set(x, y, c);
        }
    }
    for (x, y, c) in [(5, 3, COTTON + 1), (6, 3, COTTON + 2), (7, 3, COTTON + 2), (8, 3, COTTON + 1), (6, 2, COTTON + 2)] {
        s.set(x, y, c);
    }
    for x in 12..=13 {
        s.set(x, 5, WOOD + 2);
    }
    for y in 3..=5 {
        s.set(14, y, WOOD + 3);
    }
    s.outline()
}

/// A cotton bow lying on its back: an arched wooden bow, its string along the floor.
fn make_bow() -> Sprite {
    let mut s = Sprite::blank(16, 7);
    for (x, y) in [(1, 6), (2, 5), (3, 4), (4, 3), (11, 3), (12, 4), (13, 5), (14, 6)] {
        s.set(x, y, WOOD + 2);
    }
    for x in 5..=10 {
        s.set(x, 2, WOOD + 3);
        s.set(x, 3, if (7..=8).contains(&x) { WOOD + 1 } else { WOOD + 2 });
    }
    for x in 2..=13 {
        s.set(x, 6, COTTON + 2);
    }
    s.outline()
}

/// A small spinning wheel set down on the floor: smaller than the spinnery's, and bare.
fn make_wheel_tool() -> Sprite {
    let mut s = Sprite::blank(14, 13);
    let (cx, cy, r) = (7, 5, 4);
    for k in -(r - 1)..=(r - 1) {
        s.set(cx + k, cy, WOOD + 2);
        s.set(cx, cy + k, WOOD + 2);
    }
    for a in 0..48 {
        let t = a as f32 / 48.0 * std::f32::consts::TAU;
        let x = cx + (t.cos() * r as f32).round() as i32;
        let y = cy + (t.sin() * r as f32).round() as i32;
        s.set(x, y, if y < cy { WOOD + 3 } else { WOOD + 2 });
    }
    s.set(cx, cy, SPARK);
    for (x0, x1) in [(5, 3), (9, 11)] {
        for y in 9..=11 {
            s.set(x0 + (x1 - x0) * (y - 9) / 2, y, WOOD + 1);
        }
    }
    for x in 2..=12 {
        s.set(x, 12, WOOD);
    }
    s.outline()
}

/// A waist loom laid flat: the far beam, the warp running down to a strip already woven, and
/// the beam that sits at the weaver's waist.
fn make_waist_loom() -> Sprite {
    let mut s = Sprite::blank(16, 10);
    for x in 1..=14 {
        s.set(x, 2, WOOD + 3);
        s.set(x, 9, WOOD + 2);
    }
    for x in (3..=13).step_by(2) {
        for y in 3..=5 {
            s.set(x, y, COTTON + 1);
        }
    }
    for x in 2..=13 {
        s.set(x, 4, if x % 2 == 1 { COTTON + 1 } else { WOOD + 3 });
        for y in 6..=8 {
            s.set(x, y, if y == 6 { COTTON + 2 } else { COTTON + 1 });
        }
    }
    s.outline()
}

/// A workbench kit: boards bundled up with two turns of rope.
fn make_bench_kit() -> Sprite {
    let mut s = Sprite::blank(14, 9);
    for (k, row) in [2, 4, 6].into_iter().enumerate() {
        let (x0, x1) = if k == 1 { (2, 12) } else { (1, 11) };
        for x in x0..=x1 {
            s.set(x, row, WOOD + 3);
            s.set(x, row + 1, WOOD + 1);
        }
        // The end grain.
        s.set(x0, row, WOOD + 2);
        s.set(x0, row + 1, WOOD);
    }
    for y in 1..=8 {
        s.set(4, y, COTTON + 1);
        s.set(9, y, COTTON + 1);
    }
    s.outline()
}

/// A kiln kit: cut stone stacked on a wooden pallet, lashed down with rope.
fn make_kiln_kit() -> Sprite {
    let mut s = Sprite::blank(14, 12);
    for x in 1..=12 {
        s.set(x, 10, WOOD + 2);
    }
    for x in [1, 2, 6, 7, 11, 12] {
        s.set(x, 11, WOOD);
    }
    for (bx, by) in [(1, 7), (5, 7), (9, 7), (3, 4), (7, 4)] {
        for y in by..by + 3 {
            for x in bx..bx + 4 {
                let c = if x == bx + 3 || y == by + 2 { STONE[0] } else if y == by { STONE[2] } else { STONE[1] };
                s.set(x, y, c);
            }
        }
    }
    for y in 3..=10 {
        s.set(7, y, COTTON + 1);
    }
    s.outline()
}

fn make_crate() -> Sprite {
    let mut s = Sprite::blank(12, 10);
    for y in 1..=9 {
        for x in 1..=10 {
            s.set(x, y, if y == 1 || x == 1 { WOOD + 3 } else if y % 4 == 0 { WOOD } else { WOOD + 1 });
        }
    }
    s.outline()
}

/// 8x8 icons for tools, drawn as ghosts on empty pegs.

/// The Temple Bell as it stands on the ground: bronze, wide at the lip, a loop on its crown.
fn make_bell() -> Sprite {
    let mut s = Sprite::blank(16, 16);
    // Half widths from the crown down to the flared lip.
    let half = [2, 3, 3, 4, 4, 4, 5, 5, 5, 6, 6, 7, 8];
    for (k, &hw) in half.iter().enumerate() {
        let y = 3 + k as i32;
        for x in 8 - hw..8 + hw {
            let dx = x - (8 - hw);
            let c = if y >= 14 {
                if dx == 0 { SPARK } else { WOOD + 2 }
            } else if dx == 1 {
                SPARK + 1
            } else if dx < hw {
                SPARK
            } else if dx < 2 * hw - 1 {
                WOOD + 3
            } else {
                WOOD + 2
            };
            s.set(x, y, c);
        }
    }
    for (x, y) in [(6, 2), (6, 1), (7, 0), (8, 0), (9, 1), (9, 2)] {
        s.set(x, y, WOOD + 2);
    }
    s.outline()
}

/// The bell as it hangs in the temple's frame: smaller, its clapper showing under the lip.
fn make_bell_hung() -> Sprite {
    let mut s = Sprite::blank(10, 11);
    for (k, hw) in [1, 2, 2, 3, 3, 3, 4, 4, 5].into_iter().enumerate() {
        let y = 1 + k as i32;
        for x in 5 - hw..5 + hw {
            let dx = x - (5 - hw);
            s.set(x, y, if dx == 1 && hw > 1 { SPARK + 1 } else if dx < hw { SPARK } else { WOOD + 3 });
        }
    }
    s.set(4, 0, WOOD + 2);
    s.set(5, 0, WOOD + 2);
    s.set(4, 10, WOOD + 1);
    s.set(5, 10, WOOD + 1);
    s
}

/// The temple's bell frame: two posts and a beam.
fn make_bell_frame() -> Sprite {
    let mut s = Sprite::blank(16, 16);
    for y in 2..16 {
        for x in [1, 2, 13, 14] {
            s.set(x, y, if x == 1 || x == 13 { WOOD + 2 } else { WOOD + 1 });
        }
    }
    for x in 0..16 {
        s.set(x, 1, WOOD + 3);
        s.set(x, 2, WOOD + 1);
    }
    s.outline()
}

/// A stone plaque on the temple wall, lines cut into it.
fn make_plaque() -> Sprite {
    let mut s = Sprite::blank(12, 9);
    for y in 0..9 {
        for x in 0..12 {
            let edge = x == 0 || y == 0 || x == 11 || y == 8;
            let cut = !edge && y % 2 == 0 && (2..10).contains(&x) && (x + y) % 5 != 0;
            s.set(x, y, if edge { HUD_MID } else if cut { HUD_FILL } else { HUD_HI });
        }
    }
    s
}

/// The trader: a human from the hills in a straw hat and a red coat. On the second frame he
/// lifts a hand.
fn make_trader(frame: usize) -> Sprite {
    let mut s = Sprite::blank(12, 16);
    for x in 4..8 {
        s.set(x, 0, SPARK + 1);
    }
    for x in 3..9 {
        s.set(x, 1, SPARK);
    }
    for x in 1..11 {
        s.set(x, 2, if x == 1 || x == 10 { SPARK } else { SPARK + 1 });
    }
    for y in 3..7 {
        for x in 3..9 {
            s.set(x, y, if x == 8 { PATH + 1 } else { PATH + 2 });
        }
    }
    s.set(4, 4, OUTLINE);
    s.set(7, 4, OUTLINE);
    for x in 4..8 {
        s.set(x, 6, WOOD + 1);
    }
    for y in 7..13 {
        for x in 3..9 {
            s.set(x, y, if y == 10 { WOOD + 2 } else if x == 3 { THREAD + 1 } else { THREAD });
        }
    }
    for y in 8..12 {
        s.set(2, y, THREAD);
    }
    s.set(2, 12, PATH + 2);
    if frame == 0 {
        for y in 8..12 {
            s.set(9, y, THREAD);
        }
        s.set(9, 12, PATH + 2);
    } else {
        for (x, y) in [(9, 8), (10, 7)] {
            s.set(x, y, THREAD);
        }
        s.set(10, 6, PATH + 2);
    }
    for y in 13..15 {
        for x in [4, 5, 6, 7] {
            s.set(x, y, WOOD + 1);
        }
    }
    for x in [3, 4, 5, 6, 7, 8] {
        s.set(x, 15, WOOD);
    }
    s.outline()
}

/// An ingot lying on the ground: a bar narrower on top, in its metal's colors.
fn make_ingot(top: u8, face: u8, side: u8) -> Sprite {
    let mut s = Sprite::blank(14, 8);
    for x in 3..11 {
        s.set(x, 3, top);
    }
    for x in 2..12 {
        s.set(x, 4, top);
    }
    for y in 5..8 {
        for x in 1..13 {
            s.set(x, y, if x >= 11 { side } else { face });
        }
    }
    s.outline()
}

fn make_tool_icon(id: &str) -> Sprite {
    let mut s = Sprite::blank(8, 8);
    match id {
        "cotton_gin" => {
            for x in 1..=6 {
                s.set(x, 2, WOOD + 3);
                s.set(x, 3, WOOD + 1);
                s.set(x, 5, WOOD + 3);
                s.set(x, 6, WOOD + 1);
            }
            s.set(7, 2, WOOD);
            s.set(7, 1, WOOD);
        }
        "cotton_bow" => {
            for (x, y) in [(1, 6), (2, 4), (3, 2), (4, 1), (5, 2), (6, 4), (7, 6)] {
                s.set(x, y, WOOD + 2);
            }
            for x in 1..=7 {
                s.set(x, 6, COTTON + 2);
            }
        }
        "spinning_wheel" => {
            for (x, y) in [(3, 1), (4, 1), (2, 2), (5, 2), (1, 3), (6, 3), (1, 4), (6, 4), (2, 5), (5, 5), (3, 6), (4, 6)] {
                s.set(x, y, WOOD + 3);
            }
            s.set(3, 3, WOOD + 1);
            s.set(4, 4, WOOD + 1);
        }
        "cotton_slivers" => {
            for y in 2..=5 {
                for x in 0..=7 {
                    s.set(x, y, if y == 2 { COTTON + 2 } else if y == 5 { COTTON } else { COTTON + 1 });
                }
            }
        }
        "fabric" => {
            for y in 1..=6 {
                for x in 1..=6 {
                    s.set(x, y, if y == 1 || x == 1 { COTTON + 2 } else if y % 3 == 0 { COTTON } else { COTTON + 1 });
                }
            }
        }
        "elder_sticks" | "rhododendron_sticks" => {
            for (k, y0) in [(0, 2), (1, 4), (2, 6)] {
                for x in 0..=7 {
                    s.set(x, (y0 + (x + k) / 4 - 1).clamp(0, 7), WOOD + 2 + (k == 1) as u8);
                }
            }
        }
        "hemlock_log" | "oak_log" => {
            for y in 2..=5 {
                for x in 0..=5 {
                    s.set(x, y, if y == 2 { WOOD + 2 } else { WOOD + 1 });
                }
            }
            circle_fill(&mut s, 6, 3, 1, [WOOD + 2, WOOD + 3, WOOD + 3]);
        }
        // Small versions of how they lie in the world: ramie a tied bundle, bamboo loose canes.
        "ramie_stalks" => {
            for y in 2..=5 {
                for x in 0..=7 {
                    s.set(x, y, if y == 2 { LEAF + 2 } else if (x + y) % 3 == 0 { LEAF } else { LEAF + 1 });
                }
                s.set(5, y, WOOD + 2);
            }
        }
        "arrow_bamboo" => {
            for (y, c) in [(1, LEAF + 2), (3, LEAF + 1), (5, LEAF + 1)] {
                for x in 0..=7 {
                    s.set(x, y, if x % 4 == 3 { LEAF } else { c });
                    s.set(x, y + 1, LEAF);
                }
            }
        }
        "indigofera_leaves" => {
            circle_fill(&mut s, 3, 4, 2, [LEAF, LEAF + 1, LEAF + 2]);
            circle_fill(&mut s, 5, 3, 2, [LEAF, LEAF + 1, LEAF + 2]);
            s.set(4, 3, WATER + 1);
        }
        "lye" => {
            for (x, y) in [(3, 1), (4, 1), (3, 2), (4, 2)] {
                s.set(x, y, PATH + 2);
            }
            for y in 3..=6 {
                for x in 1..=6 {
                    s.set(x, y, if x <= 2 { PATH + 2 } else { PATH + 1 });
                }
            }
        }
        "limestone" | "quicklime" => {
            let ramp = if id == "limestone" { STONE } else { [TEXT_DIM, COTTON + 1, COTTON + 2] };
            circle_fill(&mut s, 2, 4, 2, ramp);
            circle_fill(&mut s, 5, 4, 2, ramp);
        }
        "indigo" => {
            for y in 2..=3 {
                for x in 1..=6 {
                    s.set(x, y, WATER + (y == 2) as u8);
                }
            }
            for (y, x0, x1) in [(4, 0, 7), (5, 1, 6), (6, 2, 5)] {
                for x in x0..=x1 {
                    s.set(x, y, PATH + 1);
                }
            }
        }
        "wooden_tub" => {
            for y in 2..=6 {
                for x in 1..=6 {
                    s.set(x, y, if y == 3 || y == 5 { HUD_MID } else { PLANK + 1 });
                }
            }
            for x in 1..=6 {
                s.set(x, 1, WOOD + 3);
            }
        }
        "dyed_fabric" => {
            for y in 1..=6 {
                for x in 1..=6 {
                    s.set(x, y, if y == 1 || x == 1 { WATER + 2 } else if y % 3 == 0 { WATER } else { WATER + 1 });
                }
            }
        }
        "rope" => {
            for (x, y) in [(2, 1), (3, 1), (4, 1), (5, 1), (1, 2), (6, 2), (1, 3), (6, 3), (1, 4), (6, 4), (1, 5), (6, 5), (2, 6), (3, 6), (4, 6), (5, 6)] {
                s.set(x, y, COTTON + 1);
            }
            for (x, y) in [(3, 3), (4, 3), (3, 4), (4, 4)] {
                s.set(x, y, COTTON + 2);
            }
        }
        // Bundled boards and stacked stone, each lashed with a rope band, as the kits lie in the
        // world.
        "workbench_kit" => {
            for row in [1, 3, 5] {
                for x in 1..=6 {
                    s.set(x, row, WOOD + 3);
                    s.set(x, row + 1, WOOD + 1);
                }
            }
            for y in 1..=6 {
                s.set(4, y, COTTON + 1);
            }
        }
        "kiln_kit" => {
            for x in 0..=7 {
                s.set(x, 7, WOOD + 2);
            }
            for (bx, by) in [(0, 4), (4, 4), (2, 1)] {
                for y in by..by + 3 {
                    for x in bx..bx + 4 {
                        let c = if x == bx + 3 || y == by + 2 { STONE[0] } else if y == by { STONE[2] } else { STONE[1] };
                        s.set(x, y, c);
                    }
                }
            }
            for y in 1..=7 {
                s.set(4, y, COTTON + 1);
            }
        }
        // Two boards, end grain to the right.
        "planks" => {
            for row in [2, 5] {
                for x in 0..=7 {
                    s.set(x, row, WOOD + 3);
                    s.set(x, row + 1, if x == 7 { WOOD + 1 } else { WOOD + 2 });
                }
            }
        }
        // A rim, four spokes and a hub.
        "wagon_wheel" => {
            for (x, y) in [(2, 1), (3, 1), (4, 1), (1, 2), (5, 2), (0, 3), (6, 3), (0, 4), (6, 4), (0, 5), (6, 5), (1, 6), (5, 6), (2, 7), (3, 7), (4, 7)] {
                s.set(x, y, if y > 4 { WOOD } else { WOOD + 2 });
            }
            for (x, y) in [(3, 2), (3, 3), (3, 5), (3, 6), (1, 4), (2, 4), (4, 4), (5, 4)] {
                s.set(x, y, WOOD + 1);
            }
            s.set(3, 4, WOOD + 3);
        }
        // Ribbed and round, with its stem.
        "pumpkin" => {
            s.set(4, 1, LEAF);
            for y in 2..=6 {
                for x in 1..=6 {
                    let corner = (y == 2 || y == 6) && (x == 1 || x == 6);
                    if !corner {
                        s.set(x, y, if x == 3 || x == 5 { WOOD + 2 } else if y == 2 { SPARK + 1 } else { SPARK });
                    }
                }
            }
        }
        // A small bronze bell.
        "temple_bell" => {
            for (k, hw) in [1, 1, 2, 2, 2, 3, 4].into_iter().enumerate() {
                let y = 1 + k as i32;
                for x in 4 - hw..4 + hw {
                    s.set(x, y, if x == 4 - hw && hw > 1 { SPARK + 1 } else if x < 4 { SPARK } else { WOOD + 3 });
                }
            }
            s.set(3, 0, WOOD + 2);
            s.set(4, 0, WOOD + 2);
        }
        // Grey paste in a little trough.
        "mortar" => {
            for y in 3..=4 {
                for x in 1..=6 {
                    s.set(x, y, if y == 3 { TEXT_DIM } else { HUD_HI });
                }
            }
            for x in 0..=7 {
                s.set(x, 5, WOOD + 2);
                s.set(x, 6, WOOD + 1);
            }
        }
        // A cake of wax with its comb still showing.
        "beeswax" => {
            for y in 2..=6 {
                for x in 1..=6 {
                    s.set(x, y, if y == 2 { SPARK + 2 } else if (x + y) % 3 == 0 { SPARK } else { SPARK + 1 });
                }
            }
        }
        // A golden jar of oil.
        "oil" => {
            for (x, y) in [(3, 1), (4, 1), (3, 2), (4, 2)] {
                s.set(x, y, WOOD + 3);
            }
            for y in 3..=6 {
                for x in 1..=6 {
                    s.set(x, y, if x <= 2 { SPARK + 1 } else { SPARK });
                }
            }
        }
        "red_clay" => circle_fill(&mut s, 4, 4, 3, [WOOD + 1, THREAD + 1, THREAD + 2]),
        "charcoal" => {
            circle_fill(&mut s, 2, 5, 2, [HUD_SHADOW, HUD_FILL, HUD_MID]);
            circle_fill(&mut s, 5, 4, 2, [HUD_SHADOW, HUD_FILL, HUD_MID]);
        }
        // Black lumps with a glint.
        "coal" => {
            circle_fill(&mut s, 2, 5, 2, COAL);
            circle_fill(&mut s, 5, 4, 2, COAL);
            s.set(1, 4, TEXT_DIM);
            s.set(4, 3, TEXT_DIM);
        }
        // Porous grey lumps.
        "coke" => {
            circle_fill(&mut s, 2, 4, 2, [HUD_MID, HUD_HI, TEXT_DIM]);
            circle_fill(&mut s, 5, 4, 2, [HUD_MID, HUD_HI, TEXT_DIM]);
            s.set(2, 4, HUD_SHADOW);
            s.set(5, 3, HUD_SHADOW);
        }
        // A bar, narrower on top, in its metal's colors.
        "copper_ingot" | "tin_ingot" => {
            let (top, face, side) = if id == "copper_ingot" { (SPARK + 1, SPARK, WOOD + 2) } else { (COTTON + 2, COTTON + 1, COTTON) };
            for x in 2..=5 {
                s.set(x, 3, top);
            }
            for y in 4..=6 {
                for x in 1..=6 {
                    s.set(x, y, if x == 6 { side } else { face });
                }
            }
        }
        // A blade with its teeth along the bottom, and a handle.
        "saw" => {
            for x in 1..=5 {
                s.set(x, 3, TEXT_DIM);
                s.set(x, 4, HUD_HI);
            }
            for x in (1..=5).step_by(2) {
                s.set(x, 5, HUD_HI);
            }
            for y in 2..=5 {
                s.set(6, y, WOOD + 2);
            }
            s.set(7, 3, WOOD + 2);
        }
        // An iron blade on a wooden handle.
        "metal_chisel" => {
            for (x, y) in [(1, 6), (2, 5), (3, 4)] {
                s.set(x, y, HUD_HI);
            }
            for (x, y) in [(4, 3), (5, 2), (6, 1)] {
                s.set(x, y, WOOD + 2);
            }
            s.set(4, 4, WOOD + 1);
        }
        // An axe: an iron head on a wooden haft.
        "metal_axe" => {
            for (x, y) in [(2, 7), (3, 6), (4, 5), (5, 4), (6, 3)] {
                s.set(x, y, WOOD + 2);
            }
            for (x, y) in [(5, 1), (6, 1), (4, 2), (5, 2), (6, 2), (7, 2), (7, 3)] {
                s.set(x, y, if y == 1 { TEXT_DIM } else { HUD_HI });
            }
            s.set(4, 3, HUD_MID);
        }
        // A smith's hammer: a squared iron head on a wooden handle.
        "metal_hammer" => {
            for y in 1..=3 {
                for x in 1..=6 {
                    s.set(x, y, if y == 1 { TEXT_DIM } else if x == 6 { HUD_MID } else { HUD_HI });
                }
            }
            for y in 4..=7 {
                s.set(3, y, WOOD + 2);
                s.set(4, y, WOOD + 1);
            }
        }
        // A sickle: a curved blade on a short handle.
        "sickle" => {
            for (x, y) in [(2, 1), (3, 0), (4, 0), (5, 0), (6, 1), (6, 2), (5, 3), (4, 4)] {
                s.set(x, y, if y == 0 { TEXT_DIM } else { HUD_HI });
            }
            for (x, y) in [(3, 5), (2, 6), (1, 7)] {
                s.set(x, y, WOOD + 2);
            }
        }
        // A scythe: a long snath and its long blade.
        "scythe" => {
            for (x, y) in [(6, 7), (6, 6), (5, 5), (5, 4), (4, 3), (4, 2), (3, 1)] {
                s.set(x, y, WOOD + 2);
            }
            for x in 0..=3 {
                s.set(x, 1, if x == 0 { HUD_HI } else { TEXT_DIM });
            }
            s.set(0, 2, HUD_HI);
            s.set(1, 2, HUD_MID);
            s.set(5, 5, WOOD + 3);
        }
        // Tongs: two long arms crossed at a pin, jaws at the top; iron forceps, or wooden tongs.
        "forceps" | "primitive_tongs" => {
            let c = if id == "forceps" { HUD_HI } else { WOOD + 2 };
            for k in 0..=6 {
                s.set(1 + k, 7 - k, c);
                s.set(6 - k / 2, 7 - k, if id == "forceps" { HUD_MID } else { WOOD + 1 });
            }
            s.set(4, 4, OUTLINE);
        }
        // A knife: an iron blade, lit along its back, on a wooden handle.
        "knife" => {
            for (x, y) in [(1, 6), (2, 5), (3, 4)] {
                s.set(x, y, WOOD + 2);
                s.set(x, y + 1, WOOD);
            }
            for (x, y) in [(4, 3), (5, 2), (6, 1)] {
                s.set(x, y, TEXT_DIM);
                s.set(x, y + 1, HUD_MID);
            }
            s.set(7, 0, TEXT_DIM);
        }
        // A needle, its eye threaded with red.
        "needle" => {
            for (x, y) in [(1, 7), (2, 6), (3, 5), (4, 4), (5, 3), (6, 2)] {
                s.set(x, y, HUD_HI);
            }
            s.set(6, 1, TEXT_DIM);
            s.set(7, 1, THREAD + 1);
            s.set(7, 2, THREAD + 1);
            s.set(7, 3, THREAD);
        }
        // A sack, tied at the neck.
        "sack" => {
            for y in 2..=7 {
                for x in 1..=6 {
                    s.set(x, y, if x == 1 { MAT + 2 } else if y == 7 { MAT } else { MAT + 1 });
                }
            }
            s.set(3, 1, MAT);
            s.set(4, 1, MAT);
            s.set(2, 0, MAT + 2);
            s.set(5, 0, MAT + 2);
        }
        // A wooden roller, handles at its ends.
        "roller" => {
            for x in 1..=6 {
                s.set(x, 3, WOOD + 3);
                s.set(x, 4, WOOD + 2);
                s.set(x, 5, WOOD + 1);
            }
            for y in [4] {
                s.set(0, y, WOOD);
                s.set(7, y, WOOD);
            }
        }
        // Pitch: a black lump with a sheen, in a clay bowl.
        "pitch" => {
            for x in 2..=5 {
                s.set(x, 2, HUD_SHADOW);
                s.set(x, 3, HUD_SHADOW);
            }
            s.set(3, 2, HUD_MID);
            for x in 1..=6 {
                s.set(x, 4, WOOD + 1);
                s.set(x, 5, WOOD + 2);
            }
            for x in 2..=5 {
                s.set(x, 6, WOOD + 1);
            }
        }
        // A stone anvil: a dressed block, narrower at its waist.
        "stone_anvil" => {
            for x in 0..=7 {
                s.set(x, 1, STONE[2]);
                s.set(x, 2, STONE[1]);
            }
            for y in 3..=4 {
                for x in 2..=5 {
                    s.set(x, y, STONE[1]);
                }
            }
            for x in 1..=6 {
                s.set(x, 5, STONE[1]);
                s.set(x, 6, STONE[0]);
            }
        }
        // A stone head lashed to a stick.
        "stone_hammer" => {
            for y in 1..=3 {
                for x in 1..=5 {
                    s.set(x, y, if y == 1 { STONE[2] } else { STONE[1] });
                }
            }
            for y in 4..=7 {
                s.set(3, y, WOOD + 2);
            }
        }
        // An iron head, curved to its points, on a wooden handle.
        "pick" => {
            for x in 2..=5 {
                s.set(x, 1, TEXT_DIM);
            }
            s.set(1, 2, HUD_HI);
            s.set(6, 2, HUD_HI);
            for y in 2..=7 {
                s.set(4, y, WOOD + 2);
            }
            s.set(4, 1, HUD_HI);
        }
        // A wooden handle and an iron blade.
        "shovel" => {
            for (x, y) in [(6, 0), (5, 1), (5, 2), (4, 3)] {
                s.set(x, y, WOOD + 2);
            }
            for y in 4..=7 {
                for x in 1..=4 {
                    s.set(x, y, if x == 1 || y == 4 { HUD_HI } else { TEXT_DIM });
                }
            }
        }
        // Wooden flanges around wound red thread, as the spool lies in the world.
        "spool" => {
            for x in 1..=6 {
                s.set(x, 1, WOOD + 3);
                s.set(x, 6, WOOD + 2);
            }
            for y in 2..=5 {
                for x in 2..=5 {
                    s.set(x, y, if x == 2 || x == 5 { THREAD } else if y % 2 == 0 { THREAD + 1 } else { THREAD + 2 });
                }
            }
        }
        _ => {
            for y in 1..=6 {
                s.set(1, y, WOOD + 2);
                s.set(6, y, WOOD + 1);
            }
            for x in 1..=6 {
                s.set(x, 1, WOOD + 3);
            }
            for x in [2, 4] {
                for y in 2..=5 {
                    s.set(x, y, COTTON + 1);
                }
            }
        }
    }
    s.outline()
}

/// A cut stump with a sprout: a tree regrowing after its log was taken.
fn stump(s: &mut Sprite) {
    for y in 12..=15 {
        for x in 5..=10 {
            s.set(x, y, if x == 5 { WOOD + 2 } else if x == 10 { WOOD } else { WOOD + 1 });
        }
    }
    for x in 5..=10 {
        s.set(x, 12, WOOD + 3);
    }
    s.set(11, 11, LEAF + 2);
    s.set(11, 10, LEAF + 1);
}

/// A conifer `h` rows tall: stacked tiers in hedge greens, lit on the left.
fn conifer(s: &mut Sprite, h: i32, ramp: [u8; 3]) {
    let top = 16 - h;
    for y in top..=12 {
        let k = y - top;
        let half = (1 + k * 6 / (13 - top).max(1)).min(6) - if k % 4 == 3 { 1 } else { 0 };
        for x in 8 - half..=7 + half {
            let c = if x <= 8 - half + 1 { ramp[2] } else if x >= 7 + half - 1 { ramp[0] } else { ramp[1] };
            s.set(x, y, c);
        }
    }
    for y in 13..=15 {
        s.set(7, y, WOOD + 1);
        s.set(8, y, WOOD);
    }
}

/// A broadleaf tree: a round crown on a short trunk.
fn broadleaf(s: &mut Sprite, r: i32, ramp: [u8; 3]) {
    circle_fill(s, 8, 12 - r, r, ramp);
    for y in 12..=15 {
        s.set(7, y, WOOD + 1);
        s.set(8, y, WOOD);
    }
}

fn shrub(s: &mut Sprite, rx: i32, ry: i32, ramp: [u8; 3]) {
    let cy = 15 - ry;
    for y in cy - ry..=15 {
        for x in 8 - rx..=8 + rx {
            let (dx, dy) = (x - 8, (y - cy).min(ry));
            if dx * dx * ry * ry + dy * dy * rx * rx <= rx * rx * ry * ry {
                let shade = dx + (y - cy);
                s.set(x, y, if shade <= -2 { ramp[2] } else if shade >= 3 { ramp[0] } else { ramp[1] });
            }
        }
    }
}

/// Tall stalks with leaves, `h` rows tall: bamboo poles (`nodes`) or ramie.
fn stalks(s: &mut Sprite, xs: &[i32], h: i32, nodes: bool) {
    for (k, &x) in xs.iter().enumerate() {
        let top = 16 - h + (k as i32 % 2) * 2;
        for y in top..=15 {
            let node = nodes && (y - top) % 4 == 3;
            s.set(x, y, if node { LEAF } else { LEAF + 1 });
            s.set(x + 1, y, if node { LEAF } else { LEAF + 2 });
        }
        // Leaves sprouting from the upper part of each stalk: slim on bamboo, broad on ramie.
        for (dy, dx) in [(1, -1), (3, 2), (5, -2), (7, 2)] {
            let y = top + dy;
            if y < 14 {
                let (lx, len) = if dx < 0 { (x + dx - 1, 2) } else { (x + dx, 2) };
                for k in 0..len {
                    s.set(lx + k, y, LEAF + 2);
                    if !nodes {
                        s.set(lx + k, y + 1, LEAF + 1);
                    }
                }
            }
        }
    }
}

/// How a wild plant grows, by the type of what it yields, so a new variety gets its type's look.
#[derive(Clone, Copy, PartialEq)]
enum Look {
    Conifer,
    Broadleaf,
    Bamboo,
    Rocks,
    Stalks,
    Leafy,
    Bush,
    Cotton,
    /// A vine that ripens into a draft creature: a pumpkin patch.
    Gourd,
    /// A rock ledge with coal in it: a seam.
    Seam,
}

impl Look {
    fn of(tags: &[String]) -> Look {
        let has = |t: &str| tags.iter().any(|x| x == t);
        match () {
            _ if has("log") && has("softwood") => Look::Conifer,
            _ if has("log") => Look::Broadleaf,
            _ if has("bamboo") => Look::Bamboo,
            _ if has("coal") || has("ore") => Look::Seam,
            _ if has("stone") => Look::Rocks,
            _ if has("stalks") => Look::Stalks,
            _ if has("leaves") => Look::Leafy,
            _ if has("bolls") => Look::Cotton,
            _ if has("draft") => Look::Gourd,
            _ => Look::Bush,
        }
    }
}

/// Every look, for plants with no art of their own.
const LOOKS: [Look; 9] = [Look::Conifer, Look::Broadleaf, Look::Bamboo, Look::Rocks, Look::Stalks, Look::Leafy, Look::Bush, Look::Gourd, Look::Seam];

/// A wild plant of `look`, at growth stage 0 (regrowing), 1 (half grown) or 2 (ready to
/// gather), with its blossoms if it has any.
fn make_wild(look: Look, bloom: Option<(u8, u8)>, stage: u8) -> Sprite {
    let mut s = Sprite::blank(16, 16);
    let light = [LEAF, LEAF + 1, LEAF + 2];
    let dark = [HEDGE, HEDGE + 1, LEAF + 1];
    match (look, stage) {
        // Ripe, it's the pumpkin itself, asleep on its vine.
        (Look::Gourd, 2) => return make_sleeping_pumpkin(true),
        (Look::Conifer | Look::Broadleaf, 0) => stump(&mut s),
        (Look::Conifer, 1) => conifer(&mut s, 10, [HEDGE + 1, LEAF, LEAF + 1]),
        (Look::Conifer, _) => conifer(&mut s, 16, [HEDGE + 1, LEAF, LEAF + 1]),
        (Look::Broadleaf, 1) => broadleaf(&mut s, 4, light),
        (Look::Broadleaf, _) => broadleaf(&mut s, 6, light),
        (Look::Bamboo, _) => stalks(&mut s, &[4, 8, 12], [5, 10, 15][stage as usize], true),
        (Look::Rocks, 0) => {
            for (cx, cy) in [(5, 14), (10, 14)] {
                circle_fill(&mut s, cx, cy, 1, STONE);
            }
        }
        (Look::Rocks, 1) => {
            for (cx, cy, r) in [(5, 12, 3), (11, 13, 2)] {
                circle_fill(&mut s, cx, cy, r, STONE);
            }
        }
        (Look::Rocks, _) => {
            for (cx, cy, r) in [(5, 10, 4), (11, 11, 4), (8, 6, 3)] {
                circle_fill(&mut s, cx, cy, r, STONE);
            }
            for x in 2..=14 {
                s.set(x, 15, STONE[0]);
            }
        }
        (Look::Stalks, _) => stalks(&mut s, &[5, 10], [5, 10, 14][stage as usize], false),
        (Look::Seam, _) => seam(&mut s, stage, bloom),
        (_, 0) => {
            for y in 12..=15 {
                s.set(8, y, LEAF);
            }
            for (x, y) in [(6, 11), (7, 12), (9, 11), (10, 12)] {
                s.set(x, y, LEAF + 2);
            }
        }
        (Look::Leafy | Look::Gourd, 1) => shrub(&mut s, 4, 3, light),
        (Look::Leafy, _) => shrub(&mut s, 6, 5, light),
        (_, 1) => shrub(&mut s, 4, 3, dark),
        _ => shrub(&mut s, 6, 5, dark),
    }
    if stage == 2 && look != Look::Seam {
        if let Some((a, b)) = bloom {
            for (x, y) in [(5, 7), (10, 6), (8, 9), (4, 11), (12, 10), (7, 5)] {
                s.set(x, y, b);
                s.set(x + 1, y, a);
                s.set(x, y + 1, a);
                s.set(x + 1, y + 1, a);
            }
        }
    }
    s.outline()
}

/// A grey rock face with a band of coal through it: black and glinting when it's ready, half
/// dug out halfway, and a dark hollow just after it's dug. Another ore's band is its own color,
/// flecked with a second (`ore`): copper's, green with verdigris.
fn seam(s: &mut Sprite, stage: u8, ore: Option<(u8, u8)>) {
    let tops = [8, 7, 6, 6, 5, 5, 5, 6, 5, 5, 6, 6, 7, 8];
    for (k, &top) in tops.iter().enumerate() {
        let x = k as i32 + 1;
        for y in top..=15 {
            s.set(x, y, if y == top { STONE[2] } else if y < 12 { STONE[1] } else { STONE[0] });
        }
    }
    // The band, from x 2 to 13: coal up to `coal`, the hollow it was dug from after.
    let coal = [1, 7, 13][stage as usize];
    for x in 2..=13 {
        if x <= coal {
            let (top, low) = ore.map_or((COAL[1], COAL[0]), |(c, _)| (shift(c, 1), c));
            s.set(x, 9, top);
            s.set(x, 10, low);
            s.set(x, 11, low);
        } else {
            s.set(x, 9, HUD_FILL);
            s.set(x, 10, STONE[0]);
            s.set(x, 11, STONE[0]);
        }
    }
    for (x, y) in [(4, 10), (8, 9), (11, 10), (6, 11)] {
        if x <= coal {
            s.set(x, y, ore.map_or(TEXT_DIM, |(_, fleck)| fleck));
        }
    }
    // A few lumps fallen at its foot while it's ready.
    if stage == 2 {
        for (x, y) in [(3, 14), (4, 14), (10, 14), (11, 14), (10, 13)] {
            s.set(x, y, ore.map_or(COAL[0], |(c, _)| c));
        }
        s.set(10, 13, COAL[1]);
    }
}

/// A decorative tree that is not for gathering: darker than the wild trees that are.
fn make_tree(round: bool) -> Sprite {
    let mut s = Sprite::blank(16, 16);
    if round {
        broadleaf(&mut s, 6, [HEDGE, HEDGE + 1, HEDGE + 2]);
    } else {
        conifer(&mut s, 16, [HEDGE, HEDGE + 1, HEDGE + 2]);
    }
    s.outline()
}

/// A log lying on its side, its cut end facing you.
fn make_log(bark: u8) -> Sprite {
    let mut s = Sprite::blank(16, 9);
    for y in 2..=7 {
        for x in 1..=11 {
            s.set(x, y, if y == 2 { bark + 2 } else if y == 7 { bark } else if (x * 3 + y) % 7 == 0 { bark } else { bark + 1 });
        }
    }
    circle_fill(&mut s, 12, 5, 3, [WOOD + 2, WOOD + 3, WOOD + 3]);
    s.set(12, 5, WOOD + 2);
    for x in 3..=10 {
        s.set(x, 8, WOOD);
    }
    s.outline()
}

/// Sawn planks, three boards stacked, their ends showing the saw's cut.
fn make_planks() -> Sprite {
    let mut s = Sprite::blank(16, 9);
    for (k, y0) in [(0, 6), (1, 4), (2, 2)] {
        let x0 = 1 + k;
        for x in x0..=x0 + 11 {
            s.set(x, y0, WOOD + 3);
            s.set(x, y0 + 1, if x == x0 + 11 { WOOD + 1 } else { WOOD + 2 });
        }
        s.set(x0 + 11, y0, WOOD + 2);
    }
    for x in 2..=12 {
        s.set(x, 8, WOOD);
    }
    s.outline()
}

/// A wagon wheel standing on its rim: hub, eight spokes and the rim.
fn make_wagon_wheel() -> Sprite {
    let mut s = Sprite::blank(15, 14);
    let (cx, cy, r) = (7, 7, 6);
    for y in 0..14 {
        for x in 0..15 {
            let d = (x - cx) * (x - cx) + (y - cy) * (y - cy);
            if d <= r * r && d > (r - 1) * (r - 1) {
                s.set(x, y, if y > cy { WOOD } else { WOOD + 2 });
            }
        }
    }
    for (dx, dy, n) in [(1, 0, 4), (-1, 0, 4), (0, 1, 4), (0, -1, 4), (1, 1, 3), (-1, 1, 3), (1, -1, 3), (-1, -1, 3)] {
        for t in 2..=n {
            s.set(cx + dx * t, cy + dy * t, WOOD + 1);
        }
    }
    circle_fill(&mut s, cx, cy, 1, [WOOD + 1, WOOD + 2, WOOD + 3]);
    s.outline()
}

/// The wagonwright's bench: a bench with a wheel leaning on it and a spoke across its top.
fn make_wagon_bench() -> Sprite {
    let mut s = Sprite::blank(16, 14);
    for x in 1..=14 {
        s.set(x, 4, WOOD + 3);
        s.set(x, 5, WOOD + 2);
        s.set(x, 6, WOOD + 1);
    }
    for y in 7..=13 {
        for x in [2, 3, 12, 13] {
            s.set(x, y, if x % 2 == 0 { WOOD + 1 } else { WOOD });
        }
    }
    // A spoke being shaped, on top.
    for x in 4..=10 {
        s.set(x, 3, PLANK + 2);
    }
    // The wheel against its front leg.
    let (cx, cy, r) = (10, 9, 4);
    for y in cy - r..=cy + r {
        for x in cx - r..=cx + r {
            let d = (x - cx) * (x - cx) + (y - cy) * (y - cy);
            if d <= r * r && d > (r - 1) * (r - 1) {
                s.set(x, y, WOOD + 2);
            }
        }
    }
    for t in -2..=2 {
        s.set(cx + t, cy, WOOD + 1);
        s.set(cx, cy + t, WOOD + 1);
    }
    s.outline()
}

/// A cart half built: its box of planks propped on a block, its shaft out front, and one
/// wheel leaning on it, waiting for the other.
fn make_cart_frame() -> Sprite {
    let mut s = Sprite::blank(16, 14);
    // The box: a plank floor and side boards with posts.
    for x in 3..=14 {
        s.set(x, 3, WOOD + 3);
        s.set(x, 4, WOOD + 2);
        s.set(x, 6, WOOD + 2);
        s.set(x, 7, WOOD + 1);
    }
    for x in [3, 8, 14] {
        s.set(x, 5, WOOD + 1);
    }
    // The shaft, out to the left.
    for x in 0..=2 {
        s.set(x, 6, WOOD + 2);
    }
    // A stone block under its back, and its first wheel leaning on its side.
    for y in 8..=13 {
        for x in 11..=14 {
            s.set(x, y, if y == 8 { STONE[2] } else { STONE[1] });
        }
    }
    let (cx, cy, r) = (6, 9, 4);
    for y in cy - r..=cy + r {
        for x in cx - r..=cx + r {
            let d = (x - cx) * (x - cx) + (y - cy) * (y - cy);
            if d <= r * r && d > (r - 1) * (r - 1) {
                s.set(x, y, if y > cy { WOOD } else { WOOD + 2 });
            }
        }
    }
    for t in -2..=2 {
        s.set(cx + t, cy, WOOD + 1);
        s.set(cx, cy + t, WOOD + 1);
    }
    s.outline()
}

/// A barrel: bulging staves, two iron hoops, a lid.
fn make_barrel() -> Sprite {
    let mut s = Sprite::blank(12, 14);
    for y in 2..=13 {
        let inset = if (4..=10).contains(&y) { 0 } else { 1 };
        for x in 1 + inset..=10 - inset {
            // Lit from the left; a darker seam between staves.
            let c = if (x + 1) % 3 == 0 { WOOD } else if x <= 4 { WOOD + 2 } else { WOOD + 1 };
            s.set(x, y, c);
        }
    }
    for x in 2..=9 {
        s.set(x, 1, WOOD + 3);
        s.set(x, 2, if x == 2 || x == 9 { WOOD + 2 } else { WOOD + 3 });
    }
    for y in [4, 11] {
        for x in 1..=10 {
            s.set(x, y, if x <= 3 { HUD_HI } else { HUD_MID });
        }
    }
    s.outline()
}

/// A chopping block: a stump of oak, its cut top ringed, an axe left in it.
fn make_chopping_block() -> Sprite {
    let mut s = Sprite::blank(14, 13);
    for y in 7..=12 {
        for x in 2..=11 {
            s.set(x, y, if x % 3 == 0 { WOOD } else if x <= 5 { WOOD + 2 } else { WOOD + 1 });
        }
    }
    for y in 5..=7 {
        for x in 2..=11 {
            let ring = (x - 6i32).abs() + (y - 6i32).abs() * 2;
            s.set(x, y, if ring == 2 || ring == 4 { PLANK + 1 } else { PLANK + 2 });
        }
    }
    for x in 3..=10 {
        s.set(x, 4, PLANK + 2);
    }
    // The axe: its head sunk in the top, its handle up to the right.
    for (x, y) in [(5, 4), (6, 4), (5, 5), (6, 5), (7, 5), (6, 3)] {
        s.set(x, y, if y == 3 { HUD_HI } else { HUD_MID });
    }
    for (x, y) in [(7, 3), (8, 2), (9, 1), (10, 0)] {
        s.set(x, y, WOOD + 2);
    }
    s.outline()
}

/// A sawhorse: a beam on two splayed pairs of legs.
fn make_sawhorse() -> Sprite {
    let mut s = Sprite::blank(16, 10);
    for x in 1..=14 {
        s.set(x, 2, WOOD + 3);
        s.set(x, 3, WOOD + 2);
    }
    for (x0, x1) in [(3, 1), (4, 6), (12, 14), (11, 9)] {
        for y in 4..=9 {
            let x = x0 + (x1 - x0) * (y - 4) / 5;
            s.set(x, y, if x0 < 8 { WOOD + 1 } else { WOOD });
        }
    }
    s.outline()
}

/// Planks stacked to season: four layers with thin sticks between, their cut ends showing.
fn make_lumber() -> Sprite {
    let mut s = Sprite::blank(16, 12);
    for (k, y0) in [(0, 9), (1, 7), (2, 5), (3, 3)] {
        let x0 = 1 + k % 2;
        for x in x0..=x0 + 12 {
            s.set(x, y0, WOOD + 3);
            s.set(x, y0 + 1, if x == x0 + 12 { PLANK + 2 } else { WOOD + 2 });
        }
        // The sticks between layers, showing at each end.
        if k > 0 {
            s.set(x0 + 1, y0 + 1, WOOD);
            s.set(x0 + 10, y0 + 1, WOOD);
        }
    }
    for x in [2, 12] {
        s.set(x, 11, WOOD);
        s.set(x + 1, 11, WOOD);
    }
    s.outline()
}

/// A bundle of sticks tied in the middle.
fn make_sticks() -> Sprite {
    let mut s = Sprite::blank(15, 9);
    for (k, y0) in [(0, 2), (1, 4), (2, 6)] {
        for x in 1..=13 {
            let y = y0 + (x + k) / 7 - 1;
            s.set(x, y.clamp(1, 8), if k == 1 { WOOD + 3 } else { WOOD + 2 });
        }
    }
    for y in 2..=7 {
        s.set(7, y, LEAF + 1);
    }
    for x in 3..=11 {
        s.set(x, 8, WOOD + 1);
    }
    s.outline()
}

/// Bamboo poles in a bundle.
fn make_bamboo() -> Sprite {
    let mut s = Sprite::blank(16, 8);
    for (y, c) in [(2, LEAF + 2), (4, LEAF + 1), (6, LEAF + 1)] {
        for x in 1..=14 {
            s.set(x, y, if x % 5 == 0 { LEAF } else { c });
            s.set(x, y + 1, if x % 5 == 0 { LEAF } else { LEAF });
        }
    }
    s.outline()
}

/// Cut green stalks, tied.
fn make_stalk_bundle() -> Sprite {
    let mut s = Sprite::blank(15, 8);
    for y in 2..=6 {
        for x in 1..=13 {
            s.set(x, y, if y == 2 { LEAF + 2 } else if (x + y) % 3 == 0 { LEAF } else { LEAF + 1 });
        }
    }
    for y in 2..=6 {
        s.set(9, y, WOOD + 2);
    }
    for x in 3..=11 {
        s.set(x, 7, LEAF);
    }
    s.outline()
}

/// A heap of indigofera leaves.
fn make_leaves() -> Sprite {
    let mut s = Sprite::blank(13, 9);
    for (cx, cy) in [(4, 5), (8, 5), (6, 3)] {
        circle_fill(&mut s, cx, cy, 3, [LEAF, LEAF + 1, LEAF + 2]);
    }
    for (x, y) in [(5, 4), (8, 6), (6, 2)] {
        s.set(x, y, WATER + 1);
    }
    for x in 2..=10 {
        s.set(x, 8, LEAF);
    }
    s.outline()
}

/// A coil of rope.
fn make_rope() -> Sprite {
    let mut s = Sprite::blank(14, 12);
    circle_fill(&mut s, 7, 6, 5, [COTTON, COTTON + 1, COTTON + 2]);
    for (x, y) in [(5, 4), (6, 4), (7, 4), (8, 4), (4, 5), (9, 5), (4, 6), (9, 6), (5, 7), (8, 7)] {
        s.set(x, y, COTTON);
    }
    for (x, y) in [(6, 5), (7, 5), (6, 6), (7, 6)] {
        s.set(x, y, 0);
    }
    for (x, y) in [(2, 3), (3, 8), (10, 2), (11, 8)] {
        s.set(x, y, COTTON);
    }
    for x in 4..=10 {
        s.set(x, 11, COTTON);
    }
    s.outline()
}

fn make_signpost() -> Sprite {
    let mut s = Sprite::blank(14, 16);
    for y in 5..=15 {
        s.set(6, y, WOOD + 2);
        s.set(7, y, WOOD + 1);
    }
    for y in 1..=5 {
        for x in 1..=12 {
            s.set(x, y, if y == 1 { WOOD + 3 } else if y == 5 { WOOD + 1 } else { WOOD + 2 });
        }
    }
    s.outline()
}

/// A project board on two posts with a notice pinned to it.
fn make_board() -> Sprite {
    let mut s = Sprite::blank(16, 16);
    for y in 9..=15 {
        s.set(2, y, WOOD + 1);
        s.set(13, y, WOOD + 1);
    }
    for y in 1..=10 {
        for x in 1..=14 {
            s.set(x, y, if y == 1 { WOOD + 3 } else if y == 10 { WOOD + 1 } else { WOOD + 2 });
        }
    }
    for y in 3..=8 {
        for x in 4..=11 {
            s.set(x, y, if y == 3 { COTTON + 2 } else { COTTON + 1 });
        }
    }
    for y in [5, 7] {
        for x in 5..=10 {
            s.set(x, y, HUD_HI);
        }
    }
    s.set(7, 3, REC + 1);
    s.outline()
}

/// Grey limestone, dark to light.
pub(crate) const STONE: [u8; 3] = [HUD_MID, HUD_HI, TEXT_DIM];

/// Coal, dark to light: nearly black, with a purple sheen.
const COAL: [u8; 3] = [HUD_SHADOW, HUD_FILL, HUD_MID];

/// A clay kiln: a dome with a fire mouth; `frame` flickers the fire while it burns.
fn make_kiln(frame: u8) -> Sprite {
    let mut s = Sprite::blank(16, 16);
    for y in 2..=15 {
        for x in 1..=14 {
            let (dx, dy) = (x - 8, (y - 9).min(0));
            if dx * dx * 49 + dy * dy * 49 > 49 * 49 + 20 {
                continue;
            }
            let c = if (y + if (y / 3) % 2 == 0 { 0 } else { 2 } + x) % 4 == 0 && y % 3 == 0 {
                PATH
            } else if x <= 4 {
                PATH + 2
            } else if x >= 12 {
                PATH
            } else {
                PATH + 1
            };
            s.set(x, y, c);
        }
    }
    for y in 10..=15 {
        for x in 6..=9 {
            if !(y == 10 && (x == 6 || x == 9)) {
                s.set(x, y, OUTLINE);
            }
        }
    }
    let flames: &[(i32, i32)] = if frame == 0 { &[(7, 13), (8, 12), (7, 14), (8, 14), (6, 15), (9, 15)] } else { &[(8, 13), (7, 12), (7, 14), (8, 15), (6, 14), (9, 15)] };
    for (k, &(x, y)) in flames.iter().enumerate() {
        s.set(x, y, SPARK + (k % 3) as u8);
    }
    s.outline()
}

/// A forge: a stone hearth with its fire, a bellows at its side and an anvil before it. Working
/// (`frame` 1 and 2), the coals glow and spark.
fn make_forge(frame: u8) -> Sprite {
    let mut s = Sprite::blank(16, 24);
    // The chimney, then the hood flaring out over the hearth.
    for y in 0..=5 {
        for x in 6..=9 {
            s.set(x, y, if x == 6 { STONE[2] } else if x == 9 { STONE[0] } else { STONE[1] });
        }
    }
    for y in 6..=9 {
        let half = 2 + (y - 5) * 3 / 2;
        for x in 8 - half..8 + half {
            s.set(x, y, if x == 8 - half { STONE[2] } else if x == 7 + half { STONE[0] } else { STONE[1] });
        }
    }
    // The hearth: a low stone block, its fire bed open under the hood.
    for y in 10..=16 {
        for x in 1..=14 {
            s.set(x, y, if y == 13 || x == 1 { STONE[2] } else if x == 14 || y == 16 { STONE[0] } else { STONE[1] });
        }
    }
    for y in 10..=12 {
        for x in 2..=13 {
            let c = if frame == 0 { if (x + y) % 3 == 0 { THREAD } else { OUTLINE } } else if (x + y + frame as i32) % 3 == 0 { SPARK + 2 } else { SPARK + (y as u8 % 2) };
            s.set(x, y, c);
        }
    }
    if frame > 0 {
        for (x, y) in [(5 + frame as i32 * 2, 9), (10 - frame as i32, 8)] {
            s.set(x, y, SPARK + 2);
        }
    }
    // The anvil before it: dark iron, its face lit, its horn to the left.
    for x in 3..=12 {
        s.set(x, 18, TEXT_DIM);
        s.set(x, 19, HUD_FILL);
    }
    s.set(1, 19, HUD_FILL);
    s.set(2, 19, HUD_FILL);
    s.set(2, 18, TEXT_DIM);
    for y in 20..=21 {
        for x in 6..=10 {
            s.set(x, y, HUD_SHADOW);
        }
    }
    for x in 4..=12 {
        s.set(x, 22, WOOD + 1);
        s.set(x, 23, WOOD);
    }
    s.outline()
}

/// A charcoal mound: a low dome of earth patched with turf, a vent at its top, and a glowing
/// draught hole at its foot. While it burns (`frame` 1 and 2), smoke curls from the vent.
fn make_mound(frame: u8) -> Sprite {
    let mut s = Sprite::blank(16, 22);
    // An earth dome, its foot the sprite's whole width, its crown six rows down.
    for y in 8..=21 {
        for x in 0..=15 {
            let (dx, dy) = (2 * x - 15, 2 * (y - 21));
            if dx * dx * 169 + dy * dy * 64 > 4 * 64 * 169 {
                continue;
            }
            let c = if dx < -7 { WOOD + 2 } else if dx > 7 { WOOD } else { WOOD + 1 };
            s.set(x, y, c);
        }
    }
    // Turf laid over the earth.
    for (x, y) in [(5, 11), (6, 11), (9, 12), (10, 12), (3, 15), (4, 15), (11, 15), (12, 16), (6, 18), (7, 18), (13, 19)] {
        s.set(x, y, LEAF);
        s.set(x, y - 1, LEAF + 1);
    }
    // The vent, and the draught hole.
    s.set(7, 9, OUTLINE);
    s.set(8, 9, OUTLINE);
    for (x, y) in [(7, 20), (8, 20), (7, 21), (8, 21)] {
        s.set(x, y, if frame == 0 { OUTLINE } else { SPARK + (x as u8 % 2) });
    }
    if frame > 0 {
        let puffs: &[(i32, i32)] = if frame == 1 { &[(7, 7), (8, 5), (7, 3), (6, 1)] } else { &[(8, 7), (7, 5), (8, 3), (9, 1)] };
        for &(x, y) in puffs {
            s.set(x, y, HUD_HI);
            s.set(x + 1, y, TEXT_DIM);
        }
    }
    s.outline()
}

/// A wooden vat with dark liquid; the dye vat has a length of blue cloth over its rim.
fn make_vat(cloth: bool) -> Sprite {
    let mut s = Sprite::blank(16, 14);
    for y in 3..=13 {
        for x in 2..=13 {
            let band = y == 5 || y == 10;
            s.set(x, y, if band { WOOD } else if x <= 3 { PLANK + 2 } else if x >= 12 { PLANK } else { PLANK + 1 });
        }
    }
    for x in 1..=14 {
        s.set(x, 2, WOOD + 3);
    }
    for x in 3..=12 {
        s.set(x, 3, if x % 3 == 0 { WATER + 1 } else { WATER });
    }
    if cloth {
        for y in 2..=9 {
            s.set(12, y, WATER + 2);
            s.set(13, y, WATER + 1);
        }
        s.set(14, 9, WATER + 1);
    }
    s.outline()
}

/// A clay jar (lye).
fn make_jar() -> Sprite {
    let mut s = Sprite::blank(11, 13);
    circle_fill(&mut s, 5, 8, 4, [PATH, PATH + 1, PATH + 2]);
    for y in 1..=4 {
        for x in 4..=6 {
            s.set(x, y, if x == 4 { PATH + 2 } else { PATH + 1 });
        }
    }
    for x in 3..=7 {
        s.set(x, 1, PATH + 2);
    }
    for x in 3..=7 {
        s.set(x, 12, PATH);
    }
    s.outline()
}

/// Coal: a heap of black lumps, each with a glint, as coal catches the light where charcoal
/// doesn't.
fn make_coal() -> Sprite {
    let mut s = Sprite::blank(14, 10);
    for (cx, cy, r) in [(4, 6, 3), (9, 6, 3), (7, 4, 2)] {
        circle_fill(&mut s, cx, cy, r, COAL);
    }
    for x in 2..=11 {
        s.set(x, 9, COAL[0]);
    }
    for (x, y) in [(3, 5), (8, 5), (6, 3)] {
        s.set(x, y, TEXT_DIM);
    }
    s.outline()
}

/// A heap of lumps flecked with another color or two: copper ore (rusty, with verdigris and a
/// coppery glint) or granite (grey, speckled).
fn make_flecked(ramp: [u8; 3], flecks: &[(i32, i32, u8)]) -> Sprite {
    let mut s = Sprite::blank(14, 10);
    for (cx, cy, r) in [(4, 6, 3), (9, 6, 3), (7, 4, 2)] {
        circle_fill(&mut s, cx, cy, r, ramp);
    }
    for x in 2..=11 {
        s.set(x, 9, ramp[0]);
    }
    for &(x, y, c) in flecks {
        s.set(x, y, c);
    }
    s.outline()
}

/// A heap of lumps: limestone (grey) or quicklime (white).
fn make_lumps(ramp: [u8; 3]) -> Sprite {
    let mut s = Sprite::blank(14, 10);
    for (cx, cy, r) in [(4, 6, 3), (9, 6, 3), (7, 4, 2)] {
        circle_fill(&mut s, cx, cy, r, ramp);
    }
    for x in 2..=11 {
        s.set(x, 9, ramp[0]);
    }
    s.outline()
}

/// Indigo paste in a shallow bowl.
fn make_indigo() -> Sprite {
    let mut s = Sprite::blank(14, 9);
    for y in 2..=4 {
        for x in 3..=10 {
            s.set(x, y, if y == 2 { WATER + 1 } else { WATER });
        }
    }
    for (y, x0, x1) in [(5, 1, 12), (6, 2, 11), (7, 3, 10), (8, 4, 9)] {
        for x in x0..=x1 {
            s.set(x, y, if x <= x0 + 1 { PATH + 2 } else { PATH + 1 });
        }
    }
    s.outline()
}

/// A wooden tub with iron-dark hoops.
fn make_tub() -> Sprite {
    let mut s = Sprite::blank(16, 11);
    for y in 2..=10 {
        let inset = (y - 2) / 4;
        for x in 1 + inset..=14 - inset {
            let hoop = y == 3 || y == 8;
            s.set(x, y, if hoop { HUD_MID } else if x <= 2 + inset { PLANK + 2 } else if x >= 13 - inset { PLANK } else { PLANK + 1 });
        }
    }
    for x in 2..=13 {
        s.set(x, 1, WOOD + 3);
        s.set(x, 2, WOOD);
    }
    s.outline()
}

/// A magical walking pumpkin: ribbed, with a curly stem, two bright eyes and little vine feet
/// that step on alternate frames.
fn make_pumpkin(frame: u8) -> Sprite {
    let mut s = Sprite::blank(16, 15);
    let (cx, cy, rx, ry) = (8, 7, 6, 5);
    for y in cy - ry..=cy + ry {
        for x in cx - rx..=cx + rx {
            let (dx, dy) = (x - cx, y - cy);
            if dx * dx * ry * ry + dy * dy * rx * rx > rx * rx * ry * ry {
                continue;
            }
            // Orange: the warm end of the spark ramp, ribbed and shaded in brown.
            let rib = dx == -3 || dx == 0 || dx == 3;
            let c = if rib { WOOD + 2 } else if dx + dy <= -4 { SPARK + 1 } else if dx + dy >= 4 { WOOD + 3 } else { SPARK };
            s.set(x, y, c);
        }
    }
    for (x, y) in [(8, 1), (8, 0), (9, 0), (10, 1)] {
        s.set(x, y, LEAF + (x == 10) as u8);
    }
    for (x, y) in [(6, 7), (10, 7)] {
        s.set(x, y, OUTLINE);
    }
    let feet: [(i32, i32); 4] = if frame == 0 { [(5, 13), (4, 14), (11, 13), (11, 14)] } else { [(5, 13), (5, 14), (11, 13), (12, 14)] };
    for (x, y) in feet {
        s.set(x, y, LEAF + 1);
    }
    s.outline()
}

/// A pumpkin asleep, not yet pulling anything: sitting on the ground, eyes shut, no feet. On
/// its vine in a patch, `vine` adds the vine and its leaves trailing either side.
fn make_sleeping_pumpkin(vine: bool) -> Sprite {
    let mut s = Sprite::blank(16, 15);
    let (cx, cy, rx, ry) = (8, 9, 6, 5);
    for y in cy - ry..=cy + ry {
        for x in cx - rx..=cx + rx {
            let (dx, dy) = (x - cx, y - cy);
            if dx * dx * ry * ry + dy * dy * rx * rx > rx * rx * ry * ry {
                continue;
            }
            let rib = dx == -3 || dx == 0 || dx == 3;
            let c = if rib { WOOD + 2 } else if dx + dy <= -4 { SPARK + 1 } else if dx + dy >= 4 { WOOD + 3 } else { SPARK };
            s.set(x, y, c);
        }
    }
    for (x, y) in [(8, 3), (8, 2), (9, 2)] {
        s.set(x, y, LEAF + 1);
    }
    // Shut eyes: a short dark line each.
    for (x, y) in [(5, 9), (6, 9), (10, 9), (11, 9)] {
        s.set(x, y, OUTLINE);
    }
    if vine {
        for (x, y, c) in [(10, 2, LEAF), (11, 1, LEAF), (12, 1, LEAF + 1), (0, 14, LEAF), (1, 13, LEAF + 1), (1, 14, LEAF), (14, 13, LEAF + 1), (15, 14, LEAF), (14, 14, LEAF)] {
            s.set(x, y, c);
        }
    }
    s.outline()
}

/// The wagon from the side: a plank bed on two spoked wheels, its tailgate at the left
/// (flipped to face the other way). Open, the tailgate hangs down.
fn make_wagon(open: bool) -> Sprite {
    let mut s = Sprite::blank(16, 14);
    for y in 3..=8 {
        for x in 2..=14 {
            let c = if y == 3 { WOOD + 3 } else if y == 8 { WOOD } else if x == 2 || x == 14 { WOOD + 1 } else if y % 2 == 0 { PLANK + 1 } else { PLANK + 2 };
            s.set(x, y, c);
        }
    }
    if open {
        for y in 7..=11 {
            s.set(1, y, WOOD + 2);
        }
        for y in 3..=6 {
            s.set(2, y, 0);
        }
    }
    for cx in [5, 11] {
        circle_fill(&mut s, cx, 11, 2, [WOOD, WOOD + 1, WOOD + 2]);
        s.set(cx, 11, WOOD + 3);
    }
    s.outline()
}

/// Wild plants with art of their own: how each grows, and the blossoms that show it's ready
/// (pink rhododendron and indigofera, white elder). Others grow by their type's look.
const WILD: [(&str, Look, Option<(u8, u8)>); 11] = [
    ("hemlock_log", Look::Conifer, None),
    ("oak_log", Look::Broadleaf, None),
    ("elder_sticks", Look::Bush, Some((COTTON + 1, COTTON + 2))),
    ("rhododendron_sticks", Look::Bush, Some((FLOWER, FLOWER + 1))),
    ("arrow_bamboo", Look::Bamboo, None),
    ("ramie_stalks", Look::Stalks, None),
    ("indigofera_leaves", Look::Leafy, Some((FLOWER, WATER + 2))),
    ("limestone", Look::Rocks, None),
    // A seam's "blossoms" are its ore's colors: copper, flecked green with verdigris.
    ("copper_ore", Look::Seam, Some((WOOD + 2, LEAF + 2))),
    ("granite", Look::Rocks, None),
    // A tapped pine is a pine; the resin is drawn on it later.
    ("pine_resin", Look::Conifer, None),
];

pub struct Sprites {
    /// Scenery's art by kind, each kind's variants (`scenery.rs`).
    props: Vec<(&'static str, Vec<Sprite>)>,
    pub goblin: [Sprite; 4],
    pub cyclops: [Sprite; 4],
    pub cotton: Sprite,
    pub slivers: Sprite,
    pub thread: Sprite,
    pub fabric: Sprite,
    pub bench: Sprite,
    pub loom: Sprite,
    pub crate_: Sprite,
    pub bench_kit: Sprite,
    pub kiln_kit: Sprite,
    pub planks: Sprite,
    pub wagon_wheel: Sprite,
    pub wagon_bench: Sprite,
    pub cart_frame: Sprite,
    pub barrel: Sprite,
    pub chopping_block: Sprite,
    pub sawhorse: Sprite,
    pub lumber: Sprite,
    pub pumpkin_asleep: Sprite,
    pub bell: Sprite,
    pub bell_hung: Sprite,
    pub bell_frame: Sprite,
    pub plaque: Sprite,
    pub trader: [Sprite; 2],
    pub copper_ingot: Sprite,
    pub tin_ingot: Sprite,
    tool_icons: Vec<(&'static str, Sprite)>,
    /// The type of thing each good with no icon of its own is shown as, from the data.
    typed_icons: std::collections::HashMap<String, Type>,
    pub spool: Sprite,
    pub plant: [Sprite; 3],
    pub wheel: [Sprite; 2],
    pub flag_goblin: Sprite,
    pub flag_cyclops: Sprite,
    pub loop_icon: Sprite,
    pub marker: Sprite,
    pub icon_cotton: Sprite,
    pub icon_thread: Sprite,
    wild: Vec<(&'static str, [Sprite; 3])>,
    looks: Vec<(Look, [Sprite; 3])>,
    pub trees: [Sprite; 2],
    pub hemlock_log: Sprite,
    pub oak_log: Sprite,
    pub sticks: Sprite,
    pub bamboo: Sprite,
    pub stalk_bundle: Sprite,
    pub leaves: Sprite,
    pub rope: Sprite,
    pub signpost: Sprite,
    pub board: Sprite,
    pub kiln: [Sprite; 2],
    /// A charcoal mound, cold, and burning in two frames of smoke.
    pub mound: [Sprite; 3],
    /// A forge, cold, and working in two frames of sparks.
    pub forge: [Sprite; 3],
    pub indigo_vat: Sprite,
    pub dye_vat: Sprite,
    pub jar: Sprite,
    pub limestone: Sprite,
    pub copper_ore: Sprite,
    pub granite: Sprite,
    pub coal: Sprite,
    pub quicklime: Sprite,
    pub indigo: Sprite,
    pub tub: Sprite,
    pub pumpkin: [Sprite; 2],
    /// Tailgate shut, open.
    pub wagon: [Sprite; 2],
    /// Tools lying on the floor (on a peg they hang as their icon).
    pub gin: Sprite,
    pub bow: Sprite,
    pub wheel_tool: Sprite,
    pub waist_loom: Sprite,
}

impl Sprites {
    pub fn new() -> Self {
        Self {
            props: crate::world::PROPS.iter().map(|k| (k.id, crate::scenery::variants(k.id))).filter(|(_, v)| !v.is_empty()).collect(),
            goblin: monster_frames(&GOBLIN_ART, &GOBLIN_LEGS_B),
            cyclops: monster_frames(&CYCLOPS_ART, &CYCLOPS_LEGS_B),
            cotton: make_cotton(),
            slivers: make_slivers(),
            thread: make_thread(),
            fabric: make_fabric(),
            bench: make_bench(),
            loom: make_loom(),
            crate_: make_crate(),
            bench_kit: make_bench_kit(),
            kiln_kit: make_kiln_kit(),
            planks: make_planks(),
            wagon_wheel: make_wagon_wheel(),
            wagon_bench: make_wagon_bench(),
            cart_frame: make_cart_frame(),
            barrel: make_barrel(),
            chopping_block: make_chopping_block(),
            sawhorse: make_sawhorse(),
            lumber: make_lumber(),
            pumpkin_asleep: make_sleeping_pumpkin(false),
            bell: make_bell(),
            bell_hung: make_bell_hung(),
            bell_frame: make_bell_frame(),
            plaque: make_plaque(),
            trader: [make_trader(0), make_trader(1)],
            copper_ingot: make_ingot(SPARK + 1, SPARK, WOOD + 2),
            tin_ingot: make_ingot(COTTON + 2, COTTON + 1, COTTON),
            tool_icons: ["cotton_gin", "cotton_bow", "spinning_wheel", "spool", "waist_loom", "cotton_slivers", "fabric", "rope", "lye", "limestone", "quicklime", "indigo", "wooden_tub", "dyed_fabric", "elder_sticks", "rhododendron_sticks", "hemlock_log", "oak_log", "ramie_stalks", "arrow_bamboo", "indigofera_leaves", "workbench_kit", "kiln_kit", "planks", "wagon_wheel", "pumpkin", "temple_bell", "mortar", "beeswax", "oil", "red_clay", "charcoal", "coke", "copper_ingot", "tin_ingot", "shovel", "saw", "metal_chisel", "stone_hammer", "coal", "pick", "sack", "roller", "pitch", "stone_anvil", "knife", "needle", "metal_axe", "metal_hammer", "sickle", "scythe", "forceps", "primitive_tongs"].into_iter().map(|id| (id, make_tool_icon(id))).collect(),
            typed_icons: crate::data::Data::load().goods.iter().filter_map(|gt| Some((gt.id.clone(), type_of(gt)?))).collect(),
            spool: make_spool(),
            plant: [make_plant(0), make_plant(1), make_plant(2)],
            wheel: [make_wheel(0), make_wheel(1)],
            flag_goblin: make_flag(GOBLIN + 2),
            flag_cyclops: make_flag(CYCLOPS + 2),
            loop_icon: make_loop_icon(),
            marker: make_marker(),
            icon_cotton: make_icon(true),
            icon_thread: make_icon(false),
            wild: WILD.iter().map(|&(g, look, bloom)| (g, [0, 1, 2].map(|stage| make_wild(look, bloom, stage)))).collect(),
            looks: LOOKS.iter().map(|&look| (look, [0, 1, 2].map(|stage| make_wild(look, None, stage)))).collect(),
            trees: [make_tree(false), make_tree(true)],
            hemlock_log: make_log(WOOD + 1),
            oak_log: make_log(WOOD),
            sticks: make_sticks(),
            bamboo: make_bamboo(),
            stalk_bundle: make_stalk_bundle(),
            leaves: make_leaves(),
            rope: make_rope(),
            signpost: make_signpost(),
            board: make_board(),
            kiln: [make_kiln(0), make_kiln(1)],
            mound: [make_mound(0), make_mound(1), make_mound(2)],
            forge: [make_forge(0), make_forge(1), make_forge(2)],
            indigo_vat: make_vat(false),
            dye_vat: make_vat(true),
            jar: make_jar(),
            limestone: make_lumps(STONE),
            coal: make_coal(),
            copper_ore: make_flecked([WOOD, WOOD + 1, WOOD + 2], &[(3, 5, LEAF + 2), (8, 4, LEAF + 1), (10, 6, LEAF + 2), (6, 7, SPARK), (4, 7, LEAF + 1), (7, 3, SPARK + 1)]),
            granite: make_flecked(STONE, &[(3, 5, HUD_FILL), (5, 7, THREAD + 2), (8, 5, HUD_FILL), (10, 7, THREAD + 2), (7, 3, HUD_FILL), (9, 4, TEXT)]),
            quicklime: make_lumps([TEXT_DIM, COTTON + 1, COTTON + 2]),
            indigo: make_indigo(),
            tub: make_tub(),
            pumpkin: [make_pumpkin(0), make_pumpkin(1)],
            wagon: [make_wagon(false), make_wagon(true)],
            gin: make_gin(),
            bow: make_bow(),
            wheel_tool: make_wheel_tool(),
            waist_loom: make_waist_loom(),
        }
    }
}

/// The types of thing a good with no art of its own is drawn as, by its tags.
#[derive(Clone, Copy)]
enum Type {
    Log,
    Sticks,
    Bamboo,
    Stalks,
    Leaves,
    Stone,
    Fiber,
    Thread,
    Fabric,
    Rope,
    Lime,
    Tub,
    Kit,
}

fn type_of(gt: &crate::data::GoodType) -> Option<Type> {
    let raw = gt.is_raw();
    let has = |t: &str| gt.type_tags.iter().any(|x| x == t);
    Some(match () {
        _ if raw && has("log") => Type::Log,
        _ if raw && has("sticks") => Type::Sticks,
        _ if raw && has("bamboo") => Type::Bamboo,
        _ if raw && has("stalks") => Type::Stalks,
        _ if raw && has("leaves") => Type::Leaves,
        _ if raw && has("stone") => Type::Stone,
        _ if has("fiber") => Type::Fiber,
        _ if has("thread") => Type::Thread,
        _ if has("fabric") => Type::Fabric,
        _ if has("rope") => Type::Rope,
        _ if has("lime") => Type::Lime,
        _ if has("tub") => Type::Tub,
        _ if has("kit") => Type::Kit,
        _ => return None,
    })
}

impl Sprites {
    /// How a good lies in the world: its own art if it has some (similar goods can look a little
    /// different, not just be colored differently), otherwise its type's art, with the ramp its
    /// variety's color goes on (a pine log is drawn as a log in pine's color), otherwise its
    /// icon, otherwise a crate.
    pub fn item(&self, good: &str, gt: Option<&crate::data::GoodType>) -> (&Sprite, Option<(u8, u8)>) {
        if let Some(own) = self.own_item(good) {
            return (own, None);
        }
        let (wood, cotton) = (ramp_named("wood"), ramp_named("cotton"));
        let typed = gt.and_then(type_of).map(|t| match t {
            Type::Log => (&self.oak_log, wood),
            Type::Sticks => (&self.sticks, wood),
            Type::Bamboo => (&self.bamboo, None),
            Type::Stalks => (&self.stalk_bundle, None),
            Type::Leaves => (&self.leaves, None),
            Type::Stone => (&self.limestone, None),
            Type::Fiber => (&self.slivers, cotton),
            Type::Thread => (&self.thread, None),
            Type::Fabric => (&self.fabric, cotton),
            Type::Rope => (&self.rope, None),
            Type::Lime => (&self.quicklime, None),
            Type::Tub => (&self.tub, None),
            Type::Kit => (&self.bench_kit, wood),
        });
        typed.unwrap_or_else(|| (self.icon(good).unwrap_or(&self.crate_), None))
    }

    /// A kind of scenery's art: variant `n` (modulo how many it has), or the yard props that
    /// had art before scenery was a table. None for kinds drawn by joining their neighbors.
    pub fn prop(&self, id: &str, n: u32) -> Option<&Sprite> {
        Some(match id {
            "wheels" => &self.wagon_wheel,
            "cart" => &self.cart_frame,
            "barrel" => &self.barrel,
            "chopping_block" => &self.chopping_block,
            "sawhorse" => &self.sawhorse,
            "lumber" => &self.lumber,
            _ => {
                let v = &self.props.iter().find(|(k, _)| *k == id)?.1;
                &v[n as usize % v.len()]
            }
        })
    }

    /// The art a good has of its own, if any.
    fn own_item(&self, good: &str) -> Option<&Sprite> {
        Some(match good {
            "cotton_bolls" => &self.cotton,
            "cotton_slivers" => &self.slivers,
            "thread" => &self.thread,
            "fabric" | "dyed_fabric" => &self.fabric,
            "spool" => &self.spool,
            "hemlock_log" => &self.hemlock_log,
            "oak_log" => &self.oak_log,
            "elder_sticks" | "rhododendron_sticks" => &self.sticks,
            "arrow_bamboo" => &self.bamboo,
            "ramie_stalks" => &self.stalk_bundle,
            "indigofera_leaves" => &self.leaves,
            "rope" => &self.rope,
            "lye" => &self.jar,
            "limestone" => &self.limestone,
            "coal" => &self.coal,
            "copper_ore" => &self.copper_ore,
            "granite" => &self.granite,
            "quicklime" => &self.quicklime,
            "indigo" => &self.indigo,
            "wooden_tub" => &self.tub,
            "cotton_gin" => &self.gin,
            "cotton_bow" => &self.bow,
            "spinning_wheel" => &self.wheel_tool,
            "waist_loom" => &self.waist_loom,
            "workbench_kit" => &self.bench_kit,
            "kiln_kit" => &self.kiln_kit,
            "planks" => &self.planks,
            "wagon_wheel" => &self.wagon_wheel,
            "pumpkin" => &self.pumpkin_asleep,
            "temple_bell" => &self.bell,
            "copper_ingot" => &self.copper_ingot,
            "tin_ingot" => &self.tin_ingot,
            _ => return None,
        })
    }

    /// A plant's sprite for its good and growth stage: its own art, or its type's look; cotton
    /// has its own.
    pub fn plant(&self, good: &str, gt: Option<&crate::data::GoodType>, stage: u8) -> &Sprite {
        let stage = stage.min(2) as usize;
        if let Some((_, frames)) = self.wild.iter().find(|(g, _)| *g == good) {
            return &frames[stage];
        }
        let look = Look::of(gt.map_or(&[][..], |gt| &gt.type_tags[..]));
        self.looks.iter().find(|(l, _)| *l == look).map_or(&self.plant[stage], |(_, frames)| &frames[stage])
    }

    pub fn icon(&self, good: &str) -> Option<&Sprite> {
        match good {
            "cotton_bolls" => Some(&self.icon_cotton),
            "thread" => Some(&self.icon_thread),
            _ => self.tool_icons.iter().find(|(id, _)| *id == good).map(|(_, s)| s).or_else(|| {
                // No icon of its own: its type's, as a pine log is drawn as a log.
                let of = match self.typed_icons.get(good)? {
                    Type::Log => "oak_log",
                    Type::Sticks => "elder_sticks",
                    Type::Bamboo => "arrow_bamboo",
                    Type::Stalks => "ramie_stalks",
                    Type::Leaves => "indigofera_leaves",
                    Type::Stone => "limestone",
                    Type::Fiber => "cotton_slivers",
                    Type::Thread => "thread",
                    Type::Fabric => "fabric",
                    Type::Rope => "rope",
                    Type::Lime => "quicklime",
                    Type::Tub => "wooden_tub",
                    Type::Kit => "workbench_kit",
                };
                (of != good).then(|| self.icon(of)).flatten()
            }),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn bottom_row_used(s: &Sprite) -> bool {
        let b = ((s.h - 1) * s.w) as usize;
        s.px[b..b + s.w as usize].iter().any(|&c| c > 1)
    }

    #[test]
    fn grounded_sprites_have_feet_on_bottom_row_and_fit_a_tile() {
        let sp = Sprites::new();
        let wild = sp.wild.iter().flat_map(|(_, f)| f.iter()).chain(sp.looks.iter().flat_map(|(_, f)| f.iter()));
        let goods = [&sp.coal, &sp.hemlock_log, &sp.oak_log, &sp.sticks, &sp.bamboo, &sp.stalk_bundle, &sp.leaves, &sp.rope, &sp.signpost, &sp.board, &sp.kiln[0], &sp.kiln[1], &sp.indigo_vat, &sp.dye_vat, &sp.jar, &sp.limestone, &sp.quicklime, &sp.indigo, &sp.tub, &sp.pumpkin[0], &sp.pumpkin[1], &sp.wagon[0], &sp.wagon[1], &sp.gin, &sp.bow, &sp.wheel_tool, &sp.waist_loom, &sp.bench_kit, &sp.kiln_kit, &sp.planks, &sp.wagon_wheel, &sp.pumpkin_asleep, &sp.bell, &sp.bell_frame, &sp.trader[0], &sp.trader[1], &sp.copper_ingot, &sp.tin_ingot, &sp.wagon_bench, &sp.cart_frame, &sp.barrel, &sp.chopping_block, &sp.sawhorse, &sp.lumber];
        let grounded = sp.goblin.iter().chain(&sp.cyclops).chain(&sp.plant).chain(&sp.wheel).chain(wild).chain(&sp.trees).chain(goods).chain([&sp.cotton, &sp.spool, &sp.slivers, &sp.thread, &sp.fabric, &sp.bench, &sp.loom, &sp.crate_]);
        for s in grounded {
            assert!(bottom_row_used(s), "floating sprite {}x{}", s.w, s.h);
            assert!(s.w <= 16 && s.h <= 16);
        }
    }

    /// A good with no art of its own, such as a new provincial wood, is drawn as its type.
    #[test]
    fn a_new_variety_is_drawn_and_grows_as_its_type() {
        let sp = Sprites::new();
        let json = r#"{"id": "pine_log", "name": "Pine Log", "type_tags": ["log", "wood", "softwood", "pine", "fuel"], "primary_applied_skill": "gathering", "raw_material_base_value": 1.0}"#;
        let pine: crate::data::GoodType = serde_json::from_str(json).unwrap();
        let (s, recolor) = sp.item("pine_log", Some(&pine));
        assert!(std::ptr::eq(s, &sp.oak_log) && recolor == ramp_named("wood"), "a log, its wood in its own color");
        assert!(sp.plant("pine_log", Some(&pine), 2).px == sp.plant("hemlock_log", None, 2).px, "growing on a conifer, as hemlock does");
        assert!(std::ptr::eq(sp.item("hemlock_log", None).0, &sp.hemlock_log), "a good with art of its own keeps it");
    }
}
