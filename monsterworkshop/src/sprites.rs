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
    fn blank(w: i32, h: i32) -> Self {
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

    fn set(&mut self, x: i32, y: i32, c: u8) {
        if x >= 0 && y >= 0 && x < self.w && y < self.h {
            self.px[(y * self.w + x) as usize] = c;
        }
    }

    /// Adds a 1px dark hull (index 1) around every colored pixel.
    fn outline(mut self) -> Self {
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

fn circle_fill(s: &mut Sprite, cx: i32, cy: i32, r: i32, ramp: [u8; 3]) {
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
        for x in 1..=6 {
            s.set(x, 1, WOOD + 2);
            s.set(x, 6, WOOD + 2);
        }
        for y in 2..=5 {
            for x in 2..=5 {
                s.set(x, y, if y % 2 == 0 { THREAD + 1 } else { THREAD + 2 });
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
        "ramie_stalks" => {
            for x in [1, 3, 5] {
                for y in 1..=7 {
                    s.set(x, y, LEAF + 1);
                    s.set(x + 1, y, LEAF + 2);
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
        "spool" => {
            for x in 2..=5 {
                s.set(x, 1, WOOD + 3);
                s.set(x, 6, WOOD + 2);
            }
            for y in 2..=5 {
                s.set(3, y, COTTON + 2);
                s.set(4, y, COTTON + 1);
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

/// Wild plants by the good they yield, at growth stage 0 (regrowing), 1 (half grown) or 2
/// (ready to gather).
fn make_wild(good: &str, stage: u8) -> Sprite {
    let mut s = Sprite::blank(16, 16);
    let light = [LEAF, LEAF + 1, LEAF + 2];
    let dark = [HEDGE, HEDGE + 1, LEAF + 1];
    match (good, stage) {
        ("hemlock_log" | "oak_log", 0) => stump(&mut s),
        ("hemlock_log", 1) => conifer(&mut s, 10, [HEDGE + 1, LEAF, LEAF + 1]),
        ("hemlock_log", _) => conifer(&mut s, 16, [HEDGE + 1, LEAF, LEAF + 1]),
        ("oak_log", 1) => broadleaf(&mut s, 4, light),
        ("oak_log", _) => broadleaf(&mut s, 6, light),
        ("arrow_bamboo", _) => stalks(&mut s, &[4, 8, 12], [5, 10, 15][stage as usize], true),
        ("limestone", 0) => {
            for (cx, cy) in [(5, 14), (10, 14)] {
                circle_fill(&mut s, cx, cy, 1, STONE);
            }
        }
        ("limestone", 1) => {
            for (cx, cy, r) in [(5, 12, 3), (11, 13, 2)] {
                circle_fill(&mut s, cx, cy, r, STONE);
            }
        }
        ("limestone", _) => {
            for (cx, cy, r) in [(5, 10, 4), (11, 11, 4), (8, 6, 3)] {
                circle_fill(&mut s, cx, cy, r, STONE);
            }
            for x in 2..=14 {
                s.set(x, 15, STONE[0]);
            }
        }
        ("ramie_stalks", _) => stalks(&mut s, &[5, 10], [5, 10, 14][stage as usize], false),
        (_, 0) => {
            for y in 12..=15 {
                s.set(8, y, LEAF);
            }
            for (x, y) in [(6, 11), (7, 12), (9, 11), (10, 12)] {
                s.set(x, y, LEAF + 2);
            }
        }
        ("indigofera_leaves", 1) => shrub(&mut s, 4, 3, light),
        ("indigofera_leaves", _) => shrub(&mut s, 6, 5, light),
        (_, 1) => shrub(&mut s, 4, 3, dark),
        _ => shrub(&mut s, 6, 5, dark),
    }
    if stage == 2 {
        // Blossoms show what is ready: pink rhododendron and indigofera, white elder.
        let bloom = match good {
            "rhododendron_sticks" => Some((FLOWER, FLOWER + 1)),
            "indigofera_leaves" => Some((FLOWER, WATER + 2)),
            "elder_sticks" => Some((COTTON + 1, COTTON + 2)),
            _ => None,
        };
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

/// The ferry: a flat two-tile boat with a plank deck.
fn make_boat() -> Sprite {
    let mut s = Sprite::blank(32, 9);
    for y in 1..=7 {
        let inset = (y - 4).max(0);
        for x in 1 + inset..=30 - inset {
            let c = match y {
                1 => WOOD + 3,
                2 | 3 => if (x + 2) % 8 == 0 { PLANK } else { PLANK + 2 },
                4 => WOOD + 2,
                _ => WOOD + (y == 5) as u8,
            };
            s.set(x, y, c);
        }
    }
    for x in [2, 29] {
        s.set(x, 0, WOOD + 3);
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
const STONE: [u8; 3] = [HUD_MID, HUD_HI, TEXT_DIM];

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

/// Goods gathered in the wild, each with its plant.
const WILD: [&str; 8] = ["hemlock_log", "oak_log", "elder_sticks", "rhododendron_sticks", "arrow_bamboo", "ramie_stalks", "indigofera_leaves", "limestone"];

pub struct Sprites {
    pub goblin: [Sprite; 4],
    pub cyclops: [Sprite; 4],
    pub cotton: Sprite,
    pub slivers: Sprite,
    pub thread: Sprite,
    pub fabric: Sprite,
    pub bench: Sprite,
    pub loom: Sprite,
    pub crate_: Sprite,
    tool_icons: Vec<(&'static str, Sprite)>,
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
    pub trees: [Sprite; 2],
    pub hemlock_log: Sprite,
    pub oak_log: Sprite,
    pub sticks: Sprite,
    pub bamboo: Sprite,
    pub stalk_bundle: Sprite,
    pub leaves: Sprite,
    pub rope: Sprite,
    pub boat: Sprite,
    pub signpost: Sprite,
    pub board: Sprite,
    pub kiln: [Sprite; 2],
    pub indigo_vat: Sprite,
    pub dye_vat: Sprite,
    pub jar: Sprite,
    pub limestone: Sprite,
    pub quicklime: Sprite,
    pub indigo: Sprite,
    pub tub: Sprite,
}

impl Sprites {
    pub fn new() -> Self {
        Self {
            goblin: monster_frames(&GOBLIN_ART, &GOBLIN_LEGS_B),
            cyclops: monster_frames(&CYCLOPS_ART, &CYCLOPS_LEGS_B),
            cotton: make_cotton(),
            slivers: make_slivers(),
            thread: make_thread(),
            fabric: make_fabric(),
            bench: make_bench(),
            loom: make_loom(),
            crate_: make_crate(),
            tool_icons: ["cotton_gin", "cotton_bow", "spinning_wheel", "spool", "waist_loom", "cotton_slivers", "fabric", "rope", "lye", "limestone", "quicklime", "indigo", "wooden_tub", "dyed_fabric", "elder_sticks", "rhododendron_sticks", "hemlock_log", "oak_log", "ramie_stalks", "indigofera_leaves"].into_iter().map(|id| (id, make_tool_icon(id))).collect(),
            spool: make_spool(),
            plant: [make_plant(0), make_plant(1), make_plant(2)],
            wheel: [make_wheel(0), make_wheel(1)],
            flag_goblin: make_flag(GOBLIN + 2),
            flag_cyclops: make_flag(CYCLOPS + 2),
            loop_icon: make_loop_icon(),
            marker: make_marker(),
            icon_cotton: make_icon(true),
            icon_thread: make_icon(false),
            wild: WILD.iter().map(|&g| (g, [make_wild(g, 0), make_wild(g, 1), make_wild(g, 2)])).collect(),
            trees: [make_tree(false), make_tree(true)],
            hemlock_log: make_log(WOOD + 1),
            oak_log: make_log(WOOD),
            sticks: make_sticks(),
            bamboo: make_bamboo(),
            stalk_bundle: make_stalk_bundle(),
            leaves: make_leaves(),
            rope: make_rope(),
            boat: make_boat(),
            signpost: make_signpost(),
            board: make_board(),
            kiln: [make_kiln(0), make_kiln(1)],
            indigo_vat: make_vat(false),
            dye_vat: make_vat(true),
            jar: make_jar(),
            limestone: make_lumps(STONE),
            quicklime: make_lumps([TEXT_DIM, COTTON + 1, COTTON + 2]),
            indigo: make_indigo(),
            tub: make_tub(),
        }
    }
}

impl Sprites {
    /// The sprite for a good lying in the world; unknown goods show as a crate.
    pub fn item(&self, good: &str) -> &Sprite {
        match good {
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
            "quicklime" => &self.quicklime,
            "indigo" => &self.indigo,
            "wooden_tub" => &self.tub,
            other => self.icon(other).unwrap_or(&self.crate_),
        }
    }

    /// A plant's sprite for its good and growth stage; cotton has its own.
    pub fn plant(&self, good: &str, stage: u8) -> &Sprite {
        let stage = stage.min(2) as usize;
        self.wild.iter().find(|(g, _)| *g == good).map_or(&self.plant[stage], |(_, frames)| &frames[stage])
    }

    pub fn icon(&self, good: &str) -> Option<&Sprite> {
        match good {
            "cotton_bolls" => Some(&self.icon_cotton),
            "thread" => Some(&self.icon_thread),
            _ => self.tool_icons.iter().find(|(id, _)| *id == good).map(|(_, s)| s),
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
        let wild = sp.wild.iter().flat_map(|(_, f)| f.iter());
        let goods = [&sp.hemlock_log, &sp.oak_log, &sp.sticks, &sp.bamboo, &sp.stalk_bundle, &sp.leaves, &sp.rope, &sp.signpost, &sp.board, &sp.kiln[0], &sp.kiln[1], &sp.indigo_vat, &sp.dye_vat, &sp.jar, &sp.limestone, &sp.quicklime, &sp.indigo, &sp.tub];
        let grounded = sp.goblin.iter().chain(&sp.cyclops).chain(&sp.plant).chain(&sp.wheel).chain(wild).chain(&sp.trees).chain(goods).chain([&sp.cotton, &sp.spool, &sp.slivers, &sp.thread, &sp.fabric, &sp.bench, &sp.loom, &sp.crate_]);
        for s in grounded {
            assert!(bottom_row_used(s), "floating sprite {}x{}", s.w, s.h);
            assert!(s.w <= 16 && s.h <= 16);
        }
    }
}
