//! Conduit's palettes (one per district of the Electrical World of Tomorrow) and monsters.
//! Each monster wears a sash in a placeholder team ramp, remapped to blue or red when drawn.

use super::data::Body;
use crate::changeling::art::outlined;
use crate::framebuffer::Sprite;
use crate::palette::*;

pub const BG: u8 = 2; // 4
pub const GROUND_L: u8 = 6;
pub const GROUND_D: u8 = 7;
pub const GROUND_X: u8 = 8; // 2
pub const TEAM: u8 = 10; // 3 placeholder
pub const BLUE: u8 = 13; // 3
pub const RED: u8 = 16; // 3
pub const KOBOLD_R: u8 = 19;
pub const ORC_R: u8 = 22;
pub const ELF_R: u8 = 25;
pub const CYCLOPS_R: u8 = 28;
pub const TROLL_R: u8 = 31;
pub const METAL: u8 = 34; // 3
pub const WOOD: u8 = 37; // 3
pub const SPARK: u8 = 40; // 3
pub const FIRE: u8 = 43; // 2
pub const HEAL: u8 = 45;
pub const SLOT_BG: u8 = 46;
pub const SLOT_HI: u8 = 47;

/// Ground per district: Maple Street brick, museum marble, junction steel, the starbridge at
/// night, railway brass.
const GROUNDS: [((u8, u8, u8), (u8, u8, u8), (u8, u8, u8), (u8, u8, u8)); 5] = [
    ((112, 72, 60), (96, 60, 52), (136, 92, 76), (72, 48, 40)),
    ((176, 168, 156), (152, 144, 136), (196, 188, 176), (120, 112, 104)),
    ((60, 68, 80), (48, 56, 68), (84, 96, 112), (40, 180, 200)),
    ((40, 32, 72), (32, 24, 60), (72, 60, 120), (200, 180, 252)),
    ((104, 84, 52), (88, 70, 44), (140, 116, 68), (60, 48, 32)),
];

pub fn palette(district: usize) -> Pal {
    let mut p = [argb((0, 0, 0)); 64];
    let (l, d, x1, x2) = GROUNDS[district.min(4)];
    ramp(&mut p, BG, 4, (6, 8, 18), (30, 34, 56));
    set(&mut p, GROUND_L, l);
    set(&mut p, GROUND_D, d);
    set(&mut p, GROUND_X, x1);
    set(&mut p, GROUND_X + 1, x2);
    ramp3(&mut p, TEAM, (90, 90, 90), (150, 150, 150), (210, 210, 210));
    ramp3(&mut p, BLUE, (24, 60, 152), (56, 120, 232), (150, 196, 252));
    ramp3(&mut p, RED, (136, 28, 28), (220, 64, 48), (252, 156, 120));
    ramp3(&mut p, KOBOLD_R, (128, 64, 32), (184, 104, 52), (228, 156, 96));
    ramp3(&mut p, ORC_R, (44, 96, 40), (84, 152, 64), (148, 204, 108));
    ramp3(&mut p, ELF_R, (152, 132, 168), (204, 188, 216), (244, 236, 248));
    ramp3(&mut p, CYCLOPS_R, (140, 112, 76), (196, 164, 116), (236, 212, 168));
    ramp3(&mut p, TROLL_R, (68, 96, 96), (108, 144, 140), (164, 196, 188));
    ramp3(&mut p, METAL, (84, 88, 100), (152, 156, 168), (220, 224, 232));
    ramp3(&mut p, WOOD, (76, 48, 28), (124, 84, 48), (172, 128, 80));
    ramp3(&mut p, SPARK, (40, 140, 200), (120, 220, 252), (232, 252, 255));
    set(&mut p, FIRE, (232, 96, 24));
    set(&mut p, FIRE + 1, (252, 200, 72));
    set(&mut p, HEAL, (112, 236, 120));
    set(&mut p, SLOT_BG, (24, 22, 30));
    set(&mut p, SLOT_HI, (60, 56, 76));
    ui_block(
        &mut p,
        &UiTheme {
            panel: [(40, 44, 60), (64, 70, 92), (112, 120, 152), (22, 24, 34)],
            text: (236, 236, 228),
            text_dim: (168, 172, 188),
            text_faint: (108, 112, 132),
            accent: (120, 220, 252),
            accent_hi: (220, 248, 255),
        },
    );
    p
}

pub fn ramp_of(b: Body) -> u8 {
    match b {
        Body::Kobold => KOBOLD_R,
        Body::Orc => ORC_R,
        Body::Elf => ELF_R,
        Body::Cyclops => CYCLOPS_R,
        Body::Troll => TROLL_R,
    }
}

pub fn team_map(blue: bool) -> [u8; 64] {
    let mut m = crate::framebuffer::IDENTITY;
    let to = if blue { BLUE } else { RED };
    for s in 0..3 {
        m[(TEAM + s) as usize] = to + s;
    }
    m
}

fn monster(rows: &[&str], b: Body) -> Sprite {
    let r = ramp_of(b);
    outlined(Sprite::art(rows, |c| match c {
        '1' => r,
        '2' => r + 1,
        '3' => r + 2,
        't' => TEAM,
        'T' => TEAM + 1,
        'U' => TEAM + 2,
        'e' => WHITE,
        'p' | 'k' => OUTLINE,
        'm' => METAL + 1,
        'w' => WOOD + 1,
        'y' => GOLD,
        _ => 0,
    }))
}

pub struct Art {
    pub bodies: Vec<(Body, Sprite)>,
}

impl Art {
    pub fn new() -> Self {
        Self {
            bodies: vec![
                (Body::Kobold, monster(&KOBOLD, Body::Kobold)),
                (Body::Orc, monster(&ORC, Body::Orc)),
                (Body::Elf, monster(&ELF, Body::Elf)),
                (Body::Cyclops, monster(&CYCLOPS, Body::Cyclops)),
                (Body::Troll, monster(&TROLL, Body::Troll)),
            ],
        }
    }

    pub fn of(&self, b: Body) -> &Sprite {
        &self.bodies.iter().find(|(k, _)| *k == b).unwrap().1
    }
}

const KOBOLD: [&str; 14] = [
    "2..........2",
    "22.2222...22",
    ".2222222222.",
    "..2epp2epp2.",
    "..22222222..",
    "...2k22k2...",
    "....2222....",
    "..tTTTTTTt..",
    ".2tTTTTTTt2.",
    ".2.tTTTTt.2.",
    "...222222...",
    "...22..22...",
    "...22..22...",
    "..222..222..",
];

const ORC: [&str; 17] = [
    "....1111......",
    "...222222.....",
    "..22222222....",
    "..2epp2epp2...",
    "..22222222....",
    "..2e2222e2....",
    "...222222.....",
    ".122222222221.",
    "12tTTTTTTTt221",
    "12TTTTTTTTTt21",
    "22.tTTTTTTt.22",
    "22.2222222..22",
    "...222..222...",
    "...222..222...",
    "...22....22...",
    "..222....222..",
    "..222....222..",
];

const ELF: [&str; 18] = [
    "...3333.....",
    "..333333....",
    "1.2222222..1",
    "12223223221.",
    ".2epp22epp2.",
    "..22222222..",
    "...222222...",
    "....2222....",
    "..tTTTTTTt..",
    ".2tTTTTTTt2.",
    ".2tTTTTTTt2.",
    ".2.tTTTTt.2.",
    "...tTTTTt...",
    "...tTTTTt...",
    "...22..22...",
    "...22..22...",
    "...22..22...",
    "..222..222..",
];

const CYCLOPS: [&str; 20] = [
    ".....222222.......",
    "....22222222......",
    "...2222222222.....",
    "...222eeee222.....",
    "...22eeppee22.....",
    "...222eeee222.....",
    "...2222222222.....",
    "....22kkkk22......",
    ".1122222222221....",
    "122tTTTTTTTTt221..",
    "122TTTTTTTTTTT221.",
    "122.tTTTTTTTt.221.",
    "22..2222222222.22.",
    "22..2222222222.22.",
    "....222....222....",
    "....222....222....",
    "....222....222....",
    "....222....222....",
    "...2222....2222...",
    "...2222....2222...",
];

const TROLL: [&str; 18] = [
    "....11111.......",
    "...2222222......",
    "..222222222.....",
    "..22epp2epp2....",
    "..222222222.....",
    "..22e22222e2....",
    "...2222222......",
    ".1122222222221..",
    "122tTTTTTTTt221.",
    "122TTTTTTTTTT221",
    "22.tTTTTTTTt.122",
    "2..222222222..22",
    "...2222222222...",
    "...222....222...",
    "...222....222...",
    "...222....222...",
    "..2222....2222..",
    "..2222....2222..",
];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn palettes_are_6bit() {
        for d in 0..5 {
            for &c in palette(d).iter() {
                assert_eq!(c & 0x0003_0303, 0);
            }
        }
    }

    #[test]
    fn monsters_fit_tiles_and_stand_on_their_bottom_row() {
        let a = Art::new();
        for (b, s) in &a.bodies {
            assert!(s.w <= 22 && s.h <= 23, "{b:?} is {}x{}", s.w, s.h);
            assert_eq!(s.bottom_row(), s.h - 1, "{b:?} floats");
        }
    }
}
