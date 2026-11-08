//! Changeling's night-forest palette and creature sprites. Sprites are authored as fills only;
//! `outlined` adds the 1px dark hull so every creature reads against the ground.

use super::rules::Kind;
use crate::framebuffer::Sprite;
use crate::palette::*;

pub const BG: u8 = 2; // 4: deep night, darkest first
pub const GROUND_D: u8 = 6;
pub const GROUND_L: u8 = 7;
pub const GRASS: u8 = 8;
pub const GRASS_HI: u8 = 9;
pub const ROCK: u8 = 10; // 4
pub const MOSS: u8 = 14; // 2
pub const HOLE: u8 = 16;
pub const ROOT: u8 = 17; // 2
pub const NEWT_R: u8 = 19;
pub const BEETLE_R: u8 = 22;
pub const FROG_R: u8 = 25;
pub const CRICKET_R: u8 = 28;
pub const HERON_R: u8 = 31;
pub const SNAKE_R: u8 = 34;
pub const SALA_R: u8 = 37;
pub const GLOW_R: u8 = 40;
pub const DEW: u8 = 43; // 2
pub const SPIRIT: u8 = 45; // 3

pub fn palette() -> Pal {
    let mut p = [argb((0, 0, 0)); 64];
    ramp(&mut p, BG, 4, (4, 8, 14), (16, 30, 38));
    set(&mut p, GROUND_D, (22, 40, 30));
    set(&mut p, GROUND_L, (32, 54, 38));
    set(&mut p, GRASS, (44, 72, 44));
    set(&mut p, GRASS_HI, (70, 104, 58));
    ramp(&mut p, ROCK, 4, (40, 44, 50), (132, 136, 140));
    ramp(&mut p, MOSS, 2, (52, 92, 48), (96, 140, 70));
    set(&mut p, HOLE, (8, 6, 6));
    ramp(&mut p, ROOT, 2, (74, 48, 30), (130, 92, 56));
    ramp3(&mut p, NEWT_R, (150, 60, 18), (228, 118, 34), (252, 188, 96));
    ramp3(&mut p, BEETLE_R, (52, 34, 100), (100, 72, 176), (176, 160, 240));
    ramp3(&mut p, FROG_R, (30, 100, 42), (76, 172, 64), (168, 228, 120));
    ramp3(&mut p, CRICKET_R, (110, 100, 30), (176, 164, 56), (232, 220, 120));
    ramp3(&mut p, HERON_R, (100, 108, 124), (176, 184, 196), (236, 240, 244));
    ramp3(&mut p, SNAKE_R, (20, 92, 96), (40, 156, 150), (132, 220, 196));
    ramp3(&mut p, SALA_R, (128, 24, 36), (204, 48, 56), (248, 128, 112));
    ramp3(&mut p, GLOW_R, (88, 28, 104), (172, 56, 196), (248, 140, 252));
    ramp(&mut p, DEW, 2, (76, 156, 236), (200, 240, 255));
    ramp3(&mut p, SPIRIT, (40, 120, 156), (96, 208, 228), (220, 252, 252));
    ui_block(
        &mut p,
        &UiTheme {
            panel: [(26, 30, 24), (44, 52, 38), (84, 96, 66), (14, 18, 14)],
            text: (236, 228, 200),
            text_dim: (156, 160, 128),
            text_faint: (96, 104, 80),
            accent: (96, 208, 228),
            accent_hi: (200, 248, 252),
        },
    );
    p
}

/// The creature's own ramp start.
pub fn ramp_of(k: Kind) -> u8 {
    match k {
        Kind::Wanderer => SPIRIT,
        Kind::Newt => NEWT_R,
        Kind::Beetle => BEETLE_R,
        Kind::Frog => FROG_R,
        Kind::Cricket => CRICKET_R,
        Kind::Heron => HERON_R,
        Kind::Snake => SNAKE_R,
        Kind::Salamander => SALA_R,
        Kind::Glowtoad | Kind::Grof => GLOW_R,
    }
}

/// Every creature ramp mapped onto the spirit ramp: what the changeling looks like in a shape.
pub fn spirit_map() -> [u8; 64] {
    let mut m = crate::framebuffer::IDENTITY;
    for start in [NEWT_R, BEETLE_R, FROG_R, CRICKET_R, HERON_R, SNAKE_R, SALA_R, GLOW_R] {
        for s in 0..3 {
            m[(start + s) as usize] = SPIRIT + s;
        }
    }
    m[HEART as usize] = SPIRIT + 2;
    m[GOLD as usize] = SPIRIT + 2;
    m
}

/// A one-colour silhouette (for arrivals still in the grass, and dazed creatures).
pub fn flat_map(col: u8) -> [u8; 64] {
    let mut m = [col; 64];
    m[0] = 0;
    m[OUTLINE as usize] = OUTLINE;
    m
}

/// Adds a 1px outline around the opaque fill (4-neighbour), growing the sprite by 1 on each side.
pub fn outlined(s: Sprite) -> Sprite {
    let (w, h) = (s.w + 2, s.h + 2);
    let mut px = vec![0u8; (w * h) as usize];
    for y in 0..s.h {
        for x in 0..s.w {
            px[((y + 1) * w + x + 1) as usize] = s.px[(y * s.w + x) as usize];
        }
    }
    let src = px.clone();
    for y in 0..h {
        for x in 0..w {
            if src[(y * w + x) as usize] != 0 {
                continue;
            }
            let near = [(1, 0), (-1, 0), (0, 1), (0, -1)].iter().any(|(dx, dy)| {
                let (nx, ny) = (x + dx, y + dy);
                nx >= 0 && ny >= 0 && nx < w && ny < h && src[(ny * w + nx) as usize] != 0 && src[(ny * w + nx) as usize] != OUTLINE
            });
            if near {
                px[(y * w + x) as usize] = OUTLINE;
            }
        }
    }
    Sprite { w, h, px }
}

fn creature(rows: &[&str], k: Kind) -> Sprite {
    let r = ramp_of(k);
    outlined(Sprite::art(rows, |c| match c {
        '1' => r,
        '2' => r + 1,
        '3' => r + 2,
        'a' => NEWT_R,
        'b' => NEWT_R + 1,
        'e' => HILITE,
        'p' => OUTLINE,
        'k' => OUTLINE,
        'r' => HEART,
        'y' => GOLD,
        'w' => WHITE,
        's' => SPIRIT + 2,
        _ => 0,
    }))
}

pub struct Art {
    pub kinds: Vec<(Kind, Sprite)>,
    pub rock: Sprite,
    pub dew: Sprite,
    pub tuft: Sprite,
}

impl Art {
    pub fn new() -> Self {
        let kinds = vec![
            (Kind::Wanderer, wanderer()),
            (Kind::Newt, creature(&NEWT, Kind::Newt)),
            (Kind::Beetle, creature(&BEETLE, Kind::Beetle)),
            (Kind::Frog, creature(&FROG, Kind::Frog)),
            (Kind::Cricket, creature(&CRICKET, Kind::Cricket)),
            (Kind::Heron, creature(&HERON, Kind::Heron)),
            (Kind::Snake, creature(&SNAKE, Kind::Snake)),
            (Kind::Salamander, creature(&SALAMANDER, Kind::Salamander)),
            (Kind::Glowtoad, creature(&GLOWTOAD, Kind::Glowtoad)),
            (Kind::Grof, creature(&GROF, Kind::Grof)),
        ];
        let rock = outlined(Sprite::art(&ROCK_ART, |c| match c {
            '1' => ROCK,
            '2' => ROCK + 1,
            '3' => ROCK + 2,
            '4' => ROCK + 3,
            'm' => MOSS,
            'M' => MOSS + 1,
            _ => 0,
        }));
        let dew = outlined(Sprite::art(&["..1..", ".112.", "11122", "11112", ".111."], |c| match c {
            '1' => DEW,
            '2' => DEW + 1,
            _ => 0,
        }));
        let tuft = Sprite::art(&["1.1.1", ".1.1.", ".111."], |c| if c == '1' { GRASS } else { 0 });
        Self { kinds, rock, dew, tuft }
    }

    pub fn of(&self, k: Kind) -> &Sprite {
        &self.kinds.iter().find(|(kk, _)| *kk == k).unwrap().1
    }
}

fn wanderer() -> Sprite {
    outlined(Sprite::art(
        &[
            ".....222.....",
            "....22222....",
            "...2233322...",
            "..223pp3p22..",
            "..22ps3ps22..",
            "..223333322..",
            "...2222222...",
            "..122222221..",
            ".12223332221.",
            ".12223332221.",
            "1122233322211",
            "1122222222211",
            "1112222222111",
            "1111222221111",
            ".1.11.1.11.1.",
        ],
        |c| match c {
            '1' => SPIRIT,
            '2' => SPIRIT + 1,
            '3' => SPIRIT + 2,
            'p' => OUTLINE,
            's' => WHITE,
            _ => 0,
        },
    ))
}

const NEWT: [&str; 9] = [
    "..............33..",
    ".............3ep3.",
    "....222222222333..",
    "...23232322222222.",
    "..2222222222222...",
    ".222111111111.....",
    "21.1.1....1..1....",
    "1..1.1....1..1....",
    "...1.11...11.11...",
];

const BEETLE: [&str; 12] = [
    "...k.....k...",
    "....k...k....",
    ".....111.....",
    "...1122211...",
    "..123k32221..",
    ".1233k332221.",
    ".1233k322221.",
    ".1223k222221.",
    "..122k22221..",
    "...1111111...",
    "..1.1...1.1..",
    ".1..1...1..1.",
];

const FROG: [&str; 13] = [
    "..333.....333...",
    ".3ep33...33ep3..",
    ".33pp3...3pp33..",
    ".2333322223332..",
    "222222222222222.",
    "2222kkkkkkk22222",
    ".22222222222222.",
    "1223333333333221",
    "1222333333332221",
    "11222222222222.1",
    ".11.1222222221..",
    "111..11....11.11",
    "11..111....111.1",
];

const CRICKET: [&str; 12] = [
    "k..............k",
    ".k.....3......k.",
    "..k...323....k..",
    "...k.32223..k...",
    "....3ep222.3....",
    "...32pp2222323..",
    "..32222222222223",
    "..122222222221..",
    "...1111111111...",
    "...1.1...1.3.3..",
    "...1.1..1..3..3.",
    "..11.11.11.3...3",
];

const HERON: [&str; 19] = [
    ".....333.......",
    "....3ep3bbbb...",
    "....3333.......",
    ".....33........",
    ".....33........",
    "......33.......",
    "......33.......",
    ".....3333......",
    "...333332222...",
    "..33333222221..",
    ".3333322222211.",
    ".2333222222111.",
    "..22222222111..",
    "...2222221.....",
    "......1.1......",
    "......1.1......",
    "......1.1......",
    "......1.1......",
    ".....11.11.....",
];

const SNAKE: [&str; 13] = [
    "........333...",
    ".......3ep33..",
    ".......3pp3rr.",
    "........333...",
    ".........22...",
    "........222...",
    ".....222221...",
    "...2222221....",
    "..22333221....",
    "..2233332222..",
    "...1133332222.",
    "..111111222221",
    ".1111111111111",
];

const SALAMANDER: [&str; 10] = [
    "...............333..",
    "..............3ep33.",
    ".....22222222223333.",
    "...222y22222y2222222",
    "..22222222y2222222..",
    ".2222y222222222y2...",
    "22.1111111111111....",
    "1..1.1......1..1....",
    "...1.1......1..1....",
    "..11.11....11.11....",
];

const GLOWTOAD: [&str; 13] = [
    "...222.....222....",
    "..2ep22...22ep2...",
    "..2pp2222222pp2...",
    ".2223222222232222.",
    "22232223222322222.",
    "2222kkkkkkkkk22222",
    "22322222222222322.",
    "222233333333322222",
    "122333333333332221",
    "11223332333332211.",
    ".1112222222222111.",
    "111.11......11.111",
    "11..111....111..11",
];

const GROF: [&str; 22] = [
    "..........1...........",
    ".........121..........",
    "........1221..........",
    ".......12221..........",
    "......122y21..........",
    ".....1222221..........",
    "....111111111.........",
    "..1111111111111.......",
    "....222.....222.......",
    "...2re22...22re2......",
    "...2rr2222222rr2......",
    "..222222222222222.....",
    ".2222kkkkkkkkk2222....",
    ".22222kkkkkkk22222....",
    "2222222222222222222...",
    "22233333333333332222..",
    "1223333333333333322221",
    "1122333333333333222111",
    ".1112222222222222111..",
    "..11122222222222111...",
    ".111.111.....111.111..",
    "1111..111...111..1111.",
];

const ROCK_ART: [&str; 15] = [
    "......mMMm..........",
    "....mMMMMmm22.......",
    "...mMmm3332222......",
    "..mm33344332222.....",
    ".m233344443322221...",
    ".2333344433322221...",
    "2233333333322222211.",
    "22333333332222222111",
    "22233333322222221111",
    "12223332222222221111",
    "1122222222222221111.",
    "11122222222221111111",
    ".1111122222111111111",
    "..11111111111111111.",
    "....111111111111....",
];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn palette_is_6bit() {
        for &c in palette().iter() {
            assert_eq!(c & 0x0003_0303, 0);
        }
    }

    #[test]
    fn sprites_fit_tiles_and_stand_on_their_bottom_row() {
        let a = Art::new();
        for (k, s) in &a.kinds {
            assert!(s.w <= 26 && s.h <= 26, "{k:?} is {}x{}", s.w, s.h);
            assert_eq!(s.bottom_row(), s.h - 1, "{k:?} floats");
        }
    }
}
