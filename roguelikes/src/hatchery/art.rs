//! HATCHERY's night-pond palette and sprites. Familiars face right; genes are drawn over the
//! body sprite (see `render::draw_familiar`), so each one looks like what it was built from.

use super::data::{Body, Foe};
use crate::framebuffer::Sprite;
use crate::gulp::art as g;
use crate::palette::*;

pub const BG: u8 = 2; // 4
pub const GROUND_L: u8 = 6;
pub const GROUND_D: u8 = 7;
pub const GRASS: u8 = 8;
pub const MUD: u8 = 9;
pub const WATER: u8 = 10; // 3
pub const TWIGS: u8 = 13; // 2
pub const EGG: u8 = 15; // 2: jelly, yolk
pub const FROG_R: u8 = 17;
pub const TOAD_R: u8 = 20;
pub const NEWT_R: u8 = 23;
pub const SALA_R: u8 = 26;
pub const AXO_R: u8 = 29;
pub const RAT_R: u8 = 32;
pub const CROW_R: u8 = 35;
pub const HOG_R: u8 = 38;
pub const SNAKE_R: u8 = 41;
pub const PURPLE_R: u8 = 44;
pub const LIT: u8 = 47;

pub fn palette() -> Pal {
    let mut p = [argb((0, 0, 0)); 64];
    ramp(&mut p, BG, 4, (6, 8, 20), (24, 30, 52));
    set(&mut p, GROUND_L, (40, 56, 44));
    set(&mut p, GROUND_D, (32, 46, 38));
    set(&mut p, GRASS, (60, 84, 56));
    set(&mut p, MUD, (66, 50, 36));
    ramp3(&mut p, WATER, (16, 36, 68), (28, 60, 100), (110, 160, 210));
    ramp(&mut p, TWIGS, 2, (90, 64, 36), (150, 112, 64));
    set(&mut p, EGG, (190, 220, 236));
    set(&mut p, EGG + 1, (40, 40, 56));
    ramp3(&mut p, FROG_R, (44, 116, 48), (96, 180, 72), (184, 232, 128));
    ramp3(&mut p, TOAD_R, (96, 76, 40), (148, 120, 64), (204, 176, 112));
    ramp3(&mut p, NEWT_R, (150, 60, 18), (228, 118, 34), (252, 188, 96));
    ramp3(&mut p, SALA_R, (28, 24, 32), (60, 52, 64), (240, 200, 48));
    ramp3(&mut p, AXO_R, (196, 104, 132), (240, 160, 184), (252, 216, 228));
    ramp3(&mut p, RAT_R, (84, 76, 76), (136, 124, 120), (200, 192, 184));
    ramp3(&mut p, CROW_R, (16, 18, 32), (40, 44, 72), (100, 108, 152));
    ramp3(&mut p, HOG_R, (88, 56, 32), (144, 100, 60), (212, 176, 128));
    ramp3(&mut p, SNAKE_R, (20, 92, 96), (40, 156, 150), (132, 220, 196));
    ramp3(&mut p, PURPLE_R, (60, 32, 96), (112, 64, 168), (184, 148, 232));
    set(&mut p, LIT, (72, 96, 72));
    ui_block(
        &mut p,
        &UiTheme {
            panel: [(30, 34, 52), (48, 54, 80), (92, 100, 140), (16, 18, 30)],
            text: (236, 236, 222),
            text_dim: (164, 172, 190),
            text_faint: (104, 110, 136),
            accent: (176, 232, 140),
            accent_hi: (224, 252, 200),
        },
    );
    p
}

pub fn body_ramp(b: Body) -> u8 {
    match b {
        Body::Frog => FROG_R,
        Body::Toad => TOAD_R,
        Body::Newt => NEWT_R,
        Body::Salamander => SALA_R,
        Body::Axolotl => AXO_R,
    }
}

fn build(rows: &[&str], ramp: u8) -> Sprite {
    Sprite::art(rows, |c| match c {
        '1' => ramp,
        '2' => ramp + 1,
        '3' | 't' => ramp + 2,
        'a' => NEWT_R,
        'b' => NEWT_R + 1,
        'e' => HILITE,
        'p' | 'k' | 'm' => OUTLINE,
        'r' => HEART,
        'y' => GOLD,
        'w' => WHITE,
        'g' => RAT_R + 2,
        _ => 0,
    })
    .outlined()
}

pub struct Art {
    pub bodies: Vec<(Body, Sprite)>,
    pub tadpole: Sprite,
    pub foes: Vec<(Foe, Sprite)>,
}

impl Art {
    pub fn new() -> Self {
        Self {
            bodies: vec![
                (Body::Frog, build(&g::FROG_M, FROG_R)),
                (Body::Toad, build(&TOAD, TOAD_R)),
                (Body::Newt, build(&g::NEWT, NEWT_R)),
                (Body::Salamander, build(&SALAMANDER, SALA_R)),
                (Body::Axolotl, build(&AXOLOTL, AXO_R)),
            ],
            tadpole: build(&TADPOLE, FROG_R),
            foes: vec![
                (Foe::Rat, build(&RAT, RAT_R)),
                (Foe::Beetle, build(&g::BEETLE, PURPLE_R)),
                (Foe::Crow, build(&CROW, CROW_R)),
                (Foe::Hedgehog, build(&HEDGEHOG, HOG_R)),
                (Foe::Snake, build(&g::SNAKE, SNAKE_R)),
                (Foe::Leech, build(&LEECH, SALA_R)),
                (Foe::Heron, build(&g::HERON, RAT_R + 1)),
                (Foe::Toadling, build(&g::FROG_S, PURPLE_R)),
                (Foe::Grof, build(&g::GROF, PURPLE_R)),
            ],
        }
    }

    pub fn body(&self, b: Body) -> &Sprite {
        &self.bodies.iter().find(|(k, _)| *k == b).unwrap().1
    }

    pub fn foe(&self, f: Foe) -> &Sprite {
        &self.foes.iter().find(|(k, _)| *k == f).unwrap().1
    }
}

/// Recolours a body ramp (bright colours, bark skin).
pub fn ramp_map(from: u8, to: [u8; 3]) -> [u8; 64] {
    let mut m = crate::framebuffer::IDENTITY;
    for k in 0..3 {
        m[(from + k) as usize] = to[k as usize];
    }
    m
}

pub fn flat_map(col: u8) -> [u8; 64] {
    let mut m = [col; 64];
    m[0] = 0;
    m[OUTLINE as usize] = OUTLINE;
    m
}

const TOAD: [&str; 11] = [
    "....33.....33...",
    "...3ep3...3pe3..",
    "..3222333332223.",
    ".22212222221222.",
    "222222222222222.",
    "22mmmmmmmmmmm222",
    "2222tttttttt2222",
    "1222222122222221",
    "11222221222221.1",
    "111.11....11.111",
    "11...11..11...11",
];

const SALAMANDER: [&str; 9] = [
    "..............33..",
    ".............3ep3.",
    "....222322232333..",
    "...23222232222222.",
    "..2223222232222...",
    ".222111111111.....",
    "21.1.1....1..1....",
    "1..1.1....1..1....",
    "...1.11...11.11...",
];

const AXOLOTL: [&str; 10] = [
    "..............1.1.",
    ".............1.1.1",
    "..............333.",
    ".............3ep3.",
    "....222222222333..",
    "...22222222222222.",
    "..2222222222222...",
    ".222111111111.....",
    "21.1.1....1..1....",
    "...1.11...11.11...",
];

const TADPOLE: [&str; 5] = ["....222.", "..22ep22", "2222222.", ".1.2222.", "1...11.."];

const RAT: [&str; 8] = ["..........11..", ".........1221.", "....2222223ep2", "..222222222222", ".1222222222k..", "1.1111111.....", "1..1.1..1.1...", "..11.11.11.11."];

const CROW: [&str; 10] = [
    "..........22..",
    ".........2ep2w",
    "........22222w",
    "...111122222..",
    "..1111222222..",
    ".1111122222...",
    "111.1222222...",
    ".....22222....",
    "......w..w....",
    ".....ww.ww....",
];

const HEDGEHOG: [&str; 9] = [
    "...1.1.1.1....",
    "..1.1.1.1.1...",
    ".11111111111..",
    "1111111111122.",
    "111111111223ep",
    "11111111122222",
    ".1122222222k..",
    "..2.2...2.2...",
    ".22.22.22.22..",
];

const LEECH: [&str; 5] = ["...2222222....", ".22222222222..", "2222222222222r", ".11111111111..", "..1.1.1.1.1..."];

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
    fn sprites_fit_and_stand() {
        let a = Art::new();
        for (b, s) in &a.bodies {
            assert!(s.w <= 26 && s.h <= 26, "{b:?}");
            assert_eq!(s.bottom_row(), s.h - 1, "{b:?} floats");
        }
        for (f, s) in &a.foes {
            assert!(s.w <= 26 && s.h <= 26, "{f:?}");
            assert_eq!(s.bottom_row(), s.h - 1, "{f:?} floats");
        }
    }
}
