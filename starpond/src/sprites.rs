use crate::palette::*;

#[derive(Clone)]
pub struct Sprite {
    pub w: i32,
    pub h: i32,
    pub px: Vec<u8>, // 0 = transparent
}

impl Sprite {
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

    fn get(&self, x: i32, y: i32) -> u8 {
        if x < 0 || y < 0 || x >= self.w || y >= self.h {
            0
        } else {
            self.px[(y * self.w + x) as usize]
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
}

fn newt_key(c: char) -> u8 {
    match c {
        'k' | 'e' => OUTLINE,
        'o' => NEWT,
        'r' => NEWT_DARK,
        'y' => NEWT_BELLY,
        _ => 0,
    }
}

// Twen faces right. Feet rest on the bottom row (row h-1) so standing sprites never float.
const NEWT_WALK_A: [&str; 10] = [
    "....................",
    ".............oooo...",
    "............ooeooo..",
    "......ooooooooooooo.",
    ".ooooooroooorooooooo",
    "...ooooooooooooooo..",
    ".....yyyyyyyyyyyy...",
    ".....oo......oo.....",
    "....ooo.....ooo.....",
    "...kkkk....kkkk.....",
];
const NEWT_WALK_B: [&str; 10] = [
    "....................",
    ".............oooo...",
    "............ooeooo..",
    "......ooooooooooooo.",
    ".ooooooroooorooooooo",
    "...ooooooooooooooo..",
    ".....yyyyyyyyyyyy...",
    ".......oo......oo...",
    "......ooo.....ooo...",
    ".....kkkk....kkkk...",
];
const NEWT_AIR: [&str; 10] = [
    "....................",
    ".............oooo...",
    "............ooeooo..",
    "......ooooooooooooo.",
    ".ooooooroooorooooooo",
    "...ooooooooooooooo..",
    ".....yyyyyyyyyyyy...",
    "....oo..........oo..",
    "...oo............oo.",
    "...k..............k.",
];
// Clinging, head up, seen from above; legs reach both sides (wall side is the right edge).
const NEWT_CLING_A: [&str; 20] = [
    "............",
    ".....oo.....",
    "....oooo....",
    "....eooe....",
    "....oooo....",
    "..o.oooo.o..",
    "..oooroooo..",
    "....oooo....",
    "....oroo....",
    "....oooo....",
    "....ooro....",
    "..o.oooo.o..",
    "..oooooooo..",
    ".....oo.....",
    ".....oo.....",
    "......o.....",
    "......o.....",
    ".....o......",
    ".....o......",
    "............",
];
const NEWT_CLING_B: [&str; 20] = [
    "............",
    ".....oo.....",
    "....oooo....",
    "....eooe....",
    "....oooo....",
    "....oooo....",
    "..oooroooo..",
    "..o.oooo.o..",
    "....oroo....",
    "....oooo....",
    "....ooro....",
    "....oooo....",
    "..oooooooo..",
    "..o..oo..o..",
    ".....oo.....",
    "......o.....",
    "......o.....",
    "......o.....",
    ".....o......",
    "............",
];

pub struct Sprites {
    pub walk: [Sprite; 2],
    pub air: Sprite,
    pub cling: [Sprite; 2],
    drops: Vec<Sprite>, // index (size - 1) * 2 + remembered
    pub hud_drop: Sprite,
}

impl Sprites {
    pub fn new() -> Self {
        let newt = |art: &[&str]| Sprite::from_art(art, newt_key).outline();
        let mut drops = Vec::new();
        for size in 1..=5u8 {
            for remembered in [false, true] {
                drops.push(make_drop(drop_radius(size), [DEW, DEW + 1, DEW + 2, DEW + 3], remembered));
            }
        }
        Self {
            walk: [newt(&NEWT_WALK_A), newt(&NEWT_WALK_B)],
            air: newt(&NEWT_AIR),
            cling: [newt(&NEWT_CLING_A), newt(&NEWT_CLING_B)],
            drops,
            hud_drop: make_drop(5, [SWATCH + 1, HUD_DEW, HUD_DEW_LIGHT, TEXT], false),
        }
    }

    pub fn drop(&self, size: u8, remembered: bool) -> &Sprite {
        let s = size.clamp(1, 5) as usize;
        &self.drops[(s - 1) * 2 + remembered as usize]
    }
}

fn drop_radius(size: u8) -> i32 {
    size.clamp(1, 5) as i32 + 1
}

/// Pixel size of a dew drop sprite (shared by rendering and pickup collision).
pub fn drop_dims(size: u8) -> (i32, i32) {
    let r = drop_radius(size);
    (2 * r + 3, 2 * r + 1 + r / 2 + 1 + 2)
}

/// Teardrop: a round body with a short tip, lit from the upper left. Drops remembered by a
/// pending echo carry a faint glowing core.
fn make_drop(r: i32, ramp: [u8; 4], remembered: bool) -> Sprite {
    let tip = r / 2 + 1;
    let w = 2 * r + 3;
    let h = 2 * r + 1 + tip + 2;
    let cx = r + 1;
    let cy = 1 + tip + r;
    let top = 1;
    let mut s = Sprite { w, h, px: vec![0; (w * h) as usize] };
    for y in 0..h {
        for x in 0..w {
            let (dx, dy) = (x - cx, y - cy);
            let in_body = dx * dx + dy * dy <= r * r + r / 2;
            let in_tip = y >= top && y < cy && dx.abs() <= (y - top) * r / (cy - top);
            if !(in_body || in_tip) {
                continue;
            }
            let shade = dx + dy;
            let c = if shade <= -r {
                ramp[2]
            } else if shade >= r - r / 3 {
                ramp[0]
            } else {
                ramp[1]
            };
            s.px[(y * w + x) as usize] = c;
        }
    }
    let g = (cx - r / 2, cy - r / 2);
    s.px[(g.1 * w + g.0) as usize] = ramp[3];
    if r >= 4 {
        s.px[(g.1 * w + g.0 + 1) as usize] = ramp[3];
        s.px[((g.1 + 1) * w + g.0) as usize] = ramp[3];
    }
    if remembered {
        let core = GLOW + 1;
        s.px[(cy * w + cx) as usize] = core;
        if r >= 4 {
            for (ox, oy) in [(1, 0), (0, 1), (1, 1)] {
                s.px[((cy + oy) * w + cx + ox) as usize] = core;
            }
        }
    }
    debug_assert!(s.get(cx, cy) != 0);
    s.outline()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn standing_newt_frames_have_feet_on_bottom_row() {
        let sp = Sprites::new();
        for spr in &sp.walk {
            let bottom = (spr.h - 1) * spr.w;
            assert!(spr.px[bottom as usize..(bottom + spr.w) as usize].iter().any(|&c| c != 0));
        }
    }

    #[test]
    fn drop_dims_match_generated_sprites_and_fit_a_tile() {
        let sp = Sprites::new();
        for size in 1..=5u8 {
            let s = sp.drop(size, false);
            assert_eq!((s.w, s.h), drop_dims(size));
            assert!(s.w <= 20 && s.h <= 20, "size {size} drop is {}x{}", s.w, s.h);
        }
    }
}
