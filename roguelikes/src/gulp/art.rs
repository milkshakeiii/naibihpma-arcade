//! GULP's pond palette and sprites. Sprites are fills only; `outlined` adds the dark hull.

use super::rules::Kind;
use crate::framebuffer::Sprite;
use crate::palette::*;

pub const BG: u8 = 2; // 4: night
pub const BANK_L: u8 = 6;
pub const BANK_D: u8 = 7;
pub const GRASS: u8 = 8;
pub const GRASS_HI: u8 = 9;
pub const WATER: u8 = 10; // 3: deep, mid, glint
pub const LILY: u8 = 13; // 2
pub const REEDS: u8 = 15; // 2
pub const STONE: u8 = 17; // 3
pub const THORN: u8 = 20; // 2
pub const FROG: u8 = 22; // 3
pub const POISON: u8 = 25; // 2: blue, yellow
pub const PINK: u8 = 27;
pub const NEWT_R: u8 = 28;
pub const BEETLE_R: u8 = 31;
pub const SNAKE_R: u8 = 34;
pub const HERON_R: u8 = 37;
pub const CRAY_R: u8 = 40;
pub const FLY_R: u8 = 43; // 2: body, wing
pub const DRAGON_R: u8 = 45; // 2
pub const MUD: u8 = 47;

pub fn palette() -> Pal {
    let mut p = [argb((0, 0, 0)); 64];
    ramp(&mut p, BG, 4, (4, 10, 16), (18, 34, 40));
    set(&mut p, BANK_L, (52, 72, 40));
    set(&mut p, BANK_D, (42, 60, 34));
    set(&mut p, GRASS, (70, 96, 50));
    set(&mut p, GRASS_HI, (104, 132, 64));
    ramp3(&mut p, WATER, (18, 44, 72), (30, 72, 108), (120, 180, 212));
    ramp(&mut p, LILY, 2, (48, 120, 56), (112, 184, 88));
    ramp(&mut p, REEDS, 2, (100, 112, 44), (172, 168, 80));
    ramp3(&mut p, STONE, (64, 68, 76), (112, 116, 124), (168, 172, 176));
    ramp(&mut p, THORN, 2, (96, 56, 40), (168, 108, 72));
    ramp3(&mut p, FROG, (44, 116, 48), (96, 180, 72), (184, 232, 128));
    set(&mut p, POISON, (40, 96, 220));
    set(&mut p, POISON + 1, (248, 216, 64));
    set(&mut p, PINK, (236, 120, 140));
    ramp3(&mut p, NEWT_R, (150, 60, 18), (228, 118, 34), (252, 188, 96));
    ramp3(&mut p, BEETLE_R, (52, 34, 100), (100, 72, 176), (176, 160, 240));
    ramp3(&mut p, SNAKE_R, (20, 92, 96), (40, 156, 150), (132, 220, 196));
    ramp3(&mut p, HERON_R, (100, 108, 124), (176, 184, 196), (236, 240, 244));
    ramp3(&mut p, CRAY_R, (128, 32, 28), (200, 64, 44), (244, 140, 104));
    set(&mut p, FLY_R, (40, 36, 44));
    set(&mut p, FLY_R + 1, (200, 220, 240));
    set(&mut p, DRAGON_R, (40, 160, 200));
    set(&mut p, DRAGON_R + 1, (180, 240, 252));
    set(&mut p, MUD, (72, 52, 36));
    ui_block(
        &mut p,
        &UiTheme {
            panel: [(26, 34, 28), (42, 56, 44), (80, 104, 76), (14, 20, 16)],
            text: (236, 232, 204),
            text_dim: (160, 168, 136),
            text_faint: (100, 112, 88),
            accent: (120, 220, 160),
            accent_hi: (208, 252, 208),
        },
    );
    p
}

pub fn ramp_of(k: Kind) -> u8 {
    match k {
        Kind::Fly => FLY_R,
        Kind::Beetle | Kind::Grof => BEETLE_R,
        Kind::Newt => NEWT_R,
        Kind::Dragonfly => DRAGON_R,
        Kind::Snake => SNAKE_R,
        Kind::Crayfish => CRAY_R,
        Kind::Heron => HERON_R,
    }
}

fn creature(rows: &[&str], k: Kind) -> Sprite {
    let r = ramp_of(k);
    Sprite::art(rows, |c| match c {
        '1' => r,
        '2' => r + 1,
        '3' => r + 2,
        'a' => NEWT_R,
        'b' => NEWT_R + 1,
        'e' => HILITE,
        'p' | 'k' => OUTLINE,
        'r' => HEART,
        'y' => GOLD,
        'w' => WHITE,
        'f' => FLY_R,
        'F' => FLY_R + 1,
        _ => 0,
    })
    .outlined()
}

/// The frog in three sizes: '1'-'3' skin, 't' throat, 'e' eye, 'p' pupil.
fn frog(rows: &[&str]) -> Sprite {
    Sprite::art(rows, |c| match c {
        '1' => FROG,
        '2' => FROG + 1,
        '3' => FROG + 2,
        't' => FROG + 2,
        'e' => HILITE,
        'p' => OUTLINE,
        'm' => OUTLINE,
        _ => 0,
    })
    .outlined()
}

pub struct Art {
    pub kinds: Vec<(Kind, Sprite)>,
    pub frogs: [Sprite; 3],
}

impl Art {
    pub fn new() -> Self {
        Self {
            kinds: vec![
                (Kind::Fly, creature(&FLY, Kind::Fly)),
                (Kind::Beetle, creature(&BEETLE, Kind::Beetle)),
                (Kind::Newt, creature(&NEWT, Kind::Newt)),
                (Kind::Dragonfly, creature(&DRAGONFLY, Kind::Dragonfly)),
                (Kind::Snake, creature(&SNAKE, Kind::Snake)),
                (Kind::Crayfish, creature(&CRAYFISH, Kind::Crayfish)),
                (Kind::Heron, creature(&HERON, Kind::Heron)),
                (Kind::Grof, creature(&GROF, Kind::Grof)),
            ],
            frogs: [frog(&FROG_S), frog(&FROG_M), frog(&FROG_L)],
        }
    }

    pub fn of(&self, k: Kind) -> &Sprite {
        &self.kinds.iter().find(|(kk, _)| *kk == k).unwrap().1
    }
}

/// Poison skin: the frog in dart-frog blue with yellow lights.
pub fn poison_map() -> [u8; 64] {
    let mut m = crate::framebuffer::IDENTITY;
    m[FROG as usize] = POISON;
    m[(FROG + 1) as usize] = POISON;
    m[(FROG + 2) as usize] = POISON + 1;
    m
}

pub fn flat_map(col: u8) -> [u8; 64] {
    let mut m = [col; 64];
    m[0] = 0;
    m[OUTLINE as usize] = OUTLINE;
    m
}

const FLY: [&str; 5] = ["F.F.F", ".FfF.", ".fff.", "..f..", ".f.f."];

const DRAGONFLY: [&str; 7] = ["FF....FF....", "FFF..FFF....", ".FF22FF.....", "..2222222222", ".FF22FF.....", "FFF..FFF....", "FF....FF...."];

const CRAYFISH: [&str; 10] = [
    "22.........22.",
    "222.......222.",
    ".22.......22..",
    "..2.33333.2...",
    "...2322223....",
    "...2222222....",
    "..2.22222.2...",
    ".2..12221..2..",
    "....1.1.1.....",
    "...1..1..1....",
];

pub(crate) const FROG_S: [&str; 9] = [".ep...pe.", "e22...22e", "2222222222", "22mmmmm22.", ".2ttttt2..", "222222222.", "21.2222.12", "1...11...1", "11.....11."];

pub(crate) const FROG_M: [&str; 12] = [
    "..33.....33...",
    ".3ep3...3pe3..",
    ".32223332223..",
    "2222222222222.",
    "222mmmmmmm222.",
    ".22ttttttt22..",
    "22222222222222",
    "2221222221222.",
    "21.1222221.12.",
    "1...12221...1.",
    "11...1.1...11.",
    "111.......111.",
];

const FROG_L: [&str; 15] = [
    "...333.......333...",
    "..3epp3.....3ppe3..",
    "..3epp3.....3ppe3..",
    ".322223333333222233",
    "2222222222222222222",
    "222mmmmmmmmmmmmm222",
    ".222ttttttttttt222.",
    "2222222222222222222",
    "2222222333332222222",
    "22212223333322212222",
    "221.122222222221.12",
    "21...1222222221...1",
    "1....1.1...1.1....1",
    "11...............11",
    "111.............111",
];

pub(crate) const NEWT: [&str; 9] = [
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

pub(crate) const BEETLE: [&str; 12] = [
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

pub(crate) const HERON: [&str; 19] = [
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

pub(crate) const SNAKE: [&str; 13] = [
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

pub(crate) const GROF: [&str; 22] = [
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
        for s in &a.frogs {
            assert!(s.w <= 24 && s.h <= 20);
            assert_eq!(s.bottom_row(), s.h - 1);
        }
    }
}
