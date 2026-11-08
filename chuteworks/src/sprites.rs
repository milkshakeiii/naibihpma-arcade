//! Hand-authored indexed sprites. Index 0 = transparent, 1 = dark hull outline.

use crate::model::{Color, BLUE, MUD, RED, YELLOW};
use crate::palette::*;

#[derive(Clone)]
pub struct Sprite {
    pub w: i32,
    pub h: i32,
    pub px: Vec<u8>,
}

impl Sprite {
    fn from_art(rows: &[&str], key: impl Fn(char) -> u8) -> Self {
        let h = rows.len() as i32;
        let w = rows[0].len() as i32;
        let mut px = Vec::with_capacity((w * h) as usize);
        for row in rows {
            assert_eq!(row.len() as i32, w, "ragged sprite row: {row}");
            px.extend(row.chars().map(&key));
        }
        Self { w, h, px }
    }

    fn set(&mut self, x: i32, y: i32, c: u8) {
        if x >= 0 && y >= 0 && x < self.w && y < self.h {
            self.px[(y * self.w + x) as usize] = c;
        }
    }
}

// A rounded block lit from the top left: 'l' light, 'm' body, 'd' shade, 'k' outline.
const ITEM: [&str; 12] = [
    "..kkkkkkkk..",
    ".kllllllmmk.",
    "kllmmmmmmmdk",
    "klmmmmmmmmdk",
    "klmmmmmmmmdk",
    "kmmmmmmmmmdk",
    "kmmmmmmmmmdk",
    "kmmmmmmmmmdk",
    "kmmmmmmmmmdk",
    "kmmmmmmmmddk",
    ".kmddddddddk",
    "..kkkkkkkk..",
];

const MINI: [&str; 8] = [
    ".kkkkkk.",
    "kllllmdk",
    "klmmmmdk",
    "kmmmmmdk",
    "kmmmmmdk",
    "kmmmmmdk",
    "kmdddddk",
    ".kkkkkk.",
];

const POINTER: [&str; 11] = [
    "k.......",
    "kk......",
    "kwk.....",
    "kwwk....",
    "kwwwk...",
    "kwwwwk..",
    "kwwwwwk.",
    "kwwwkkkk",
    "kwkwk...",
    "kk.kwk..",
    "....kk..",
];

pub struct Sprites {
    /// 12x12 items indexed by color bitmask (0 grey .. 7 mud).
    pub items: Vec<Sprite>,
    /// 8x8 chute miniatures.
    pub minis: Vec<Sprite>,
    pub pointer: Sprite,
}

impl Sprites {
    pub fn new() -> Self {
        let mut items = Vec::new();
        let mut minis = Vec::new();
        for c in 0..8u8 {
            let (d, m, l) = item_ramp(c);
            let key = |ch: char| match ch {
                'k' => OUTLINE,
                'd' => d,
                'm' => m,
                'l' => l,
                _ => 0,
            };
            let mut s = Sprite::from_art(&ITEM, key);
            // Pips spell the primaries so colors read without color vision: red left,
            // yellow middle, blue right. Mud gets a cross.
            if c == MUD {
                for i in 0..6 {
                    s.set(3 + i, 3 + i, OUTLINE);
                    s.set(8 - i, 3 + i, OUTLINE);
                }
            } else {
                for (bit, px) in [(RED, 2), (YELLOW, 5), (BLUE, 8)] {
                    if c & bit != 0 {
                        for (dx, dy) in [(0, 0), (1, 0), (0, 1), (1, 1)] {
                            s.set(px + dx, 6 + dy, OUTLINE);
                        }
                    }
                }
            }
            items.push(s);

            let mut mi = Sprite::from_art(&MINI, key);
            if c == MUD {
                mi.set(2, 2, OUTLINE);
                mi.set(5, 5, OUTLINE);
                mi.set(5, 2, OUTLINE);
                mi.set(2, 5, OUTLINE);
            } else {
                for (bit, px) in [(RED, 2), (YELLOW, 4), (BLUE, 6)] {
                    if c & bit != 0 {
                        mi.set(px - 1, 4, OUTLINE);
                    }
                }
            }
            minis.push(mi);
        }
        let pointer = Sprite::from_art(&POINTER, |ch| match ch {
            'k' => OUTLINE,
            'w' => CURSOR,
            _ => 0,
        });
        Self { items, minis, pointer }
    }

    pub fn item(&self, c: Color) -> &Sprite {
        &self.items[(c & 7) as usize]
    }

    pub fn mini(&self, c: Color) -> &Sprite {
        &self.minis[(c & 7) as usize]
    }
}
