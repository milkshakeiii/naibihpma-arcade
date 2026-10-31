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
            tool_icons: ["cotton_gin", "cotton_bow", "spinning_wheel", "spool", "waist_loom", "cotton_slivers", "fabric"].into_iter().map(|id| (id, make_tool_icon(id))).collect(),
            spool: make_spool(),
            plant: [make_plant(0), make_plant(1), make_plant(2)],
            wheel: [make_wheel(0), make_wheel(1)],
            flag_goblin: make_flag(GOBLIN + 2),
            flag_cyclops: make_flag(CYCLOPS + 2),
            loop_icon: make_loop_icon(),
            marker: make_marker(),
            icon_cotton: make_icon(true),
            icon_thread: make_icon(false),
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
            other => self.icon(other).unwrap_or(&self.crate_),
        }
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
        let grounded = sp.goblin.iter().chain(&sp.cyclops).chain(&sp.plant).chain(&sp.wheel).chain([&sp.cotton, &sp.spool, &sp.slivers, &sp.thread, &sp.fabric, &sp.bench, &sp.loom, &sp.crate_]);
        for s in grounded {
            assert!(bottom_row_used(s), "floating sprite {}x{}", s.w, s.h);
            assert!(s.w <= 16 && s.h <= 16);
        }
    }
}
