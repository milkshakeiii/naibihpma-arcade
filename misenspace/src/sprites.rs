//! Hand-authored indexed sprites. Index 0 = transparent, 1 = dark hull outline.

use crate::model::{Item, Species, ITEM_COUNT};
use crate::palette::*;

#[derive(Clone)]
pub struct Sprite {
    pub w: i32,
    pub h: i32,
    pub px: Vec<u8>,
}

/// One key for every sprite: ramps are digits/letters, darkest first.
fn key(c: char) -> u8 {
    match c {
        'k' => OUTLINE,
        '1' => GOLD,
        '2' => GOLD + 1,
        '3' => GOLD + 2,
        '4' => BROWN,
        '5' => BROWN + 1,
        '6' => BROWN + 2,
        '7' => ICE,
        '8' => ICE + 1,
        '9' => ICE + 2,
        'a' => GREEN,
        'b' => GREEN + 1,
        'c' => GREEN + 2,
        'e' => PURP,
        'f' => PURP + 1,
        'g' => PURP + 2,
        'h' => ORANGE,
        'i' => ORANGE + 1,
        'j' => ORANGE + 2,
        'w' => PLATE,
        's' => PLATE_SH,
        'v' => VISOR,
        'r' => RED,
        'n' => PINK,
        'W' => WHITE,
        'x' => METAL + 1,
        _ => 0,
    }
}

impl Sprite {
    fn art(rows: &[&str]) -> Self {
        let h = rows.len() as i32;
        let w = rows[0].len() as i32;
        let mut px = Vec::with_capacity((w * h) as usize);
        for row in rows {
            assert_eq!(row.len() as i32, w, "ragged sprite row: {row}");
            px.extend(row.chars().map(key));
        }
        Self { w, h, px }
    }
}

const RYE: [&str; 10] = [
    "..k.k.k...", ".k3k3k3k..", ".k2k2k2k..", ".k2k2k2k..", "..k222k...", "...k1k....", "..k2k2k...", ".k2k.k2k..", ".k1k.k1k..",
    "..k...k...",
];
const BREAD: [&str; 10] = [
    "..........", "...kkkk...", ".kk6666kk.", "k66565656k", "k55555555k", "k55555555k", "k45555554k", ".k444444k.", "..kkkkkk..",
    "..........",
];
const BERRY: [&str; 10] = [
    ".....kk...", "....kbck..", "...kkkk...", "..k9988k..", ".k9W8888k.", ".k988887k.", ".k888877k.", "..k8777k..", "...kkkk...",
    "..........",
];
const SLICED: [&str; 10] = [
    "..........", "..........", ".kkkk.kkkk", "k9WW8kWW8k", "k9W88kW88k", "k8888k888k", ".k77k.k77k", "..kk...kk.", "..........",
    "..........",
];
const MOSS: [&str; 10] = [
    "..........", "...kk.kk..", "..kcckcck.", ".kcbbcbbk.", "kcbbbbbbbk", "kbbabbabbk", ".kbaabaak.", "..kaaaak..", "...kkkk...",
    "..........",
];
const BEAN: [&str; 10] = [
    "..........", "..kkk.....", ".kgffk....", ".kfffk....", ".kfeek.kk.", "..kkk.kgfk", ".....kfffk", ".....kfeek", "......kkk.",
    "..........",
];
const ROASTED: [&str; 10] = [
    "..........", "..kkk.....", ".k655k....", ".k555k....", ".k544k.kk.", "..kkk.k65k", ".....k555k", ".....k544k", "......kkk.",
    "..........",
];
const TOAST: [&str; 10] = [
    "..........", "..kkkkkk..", ".k466664k.", "k46896664k", "k46666984k", "k48966664k", "k46669864k", "k44444444k", ".kkkkkkkk.",
    "..........",
];
const BOWL: [&str; 10] = [
    "..........", "...kkkk...", "..kcbbck..", ".kbbabbbk.", "kkkkkkkkkk", "kwwwwwwwsk", ".kwwwwwsk.", "..kwwwsk..", "...kkkk...",
    "..........",
];
const BEANPLATE: [&str; 10] = [
    "..........", "..........", "..kkkkkk..", ".kwfewbwk.", "kwwfe5bwsk", "kwww54wwsk", ".kssssssk.", "..kkkkkk..", "..........",
    "..........",
];

const CREW: [&str; 16] = [
    "....kkkk....", "...kjjjjk...", "..kjiiiiik..", "..kivvvvik..", "..kivWvvik..", "..kiiiiiik..", "...kkiikk...", "..kjiiiiik..",
    ".kjiiiiiiik.", ".kiihiihiik.", ".kiiiiiiiik.", "..khhhhhhk..", "...k5k.k5k..", "...k5k.k5k..", "..k44k.k44k.", "............",
];
const BLOB: [&str; 16] = [
    "............", "............", "............", "....kkkk....", "..kkccccckk.", ".kcccccccbbk", ".kcWkccWkbbk", "kcckkcckkbbk",
    "kbbbbbbbbbbk", "kbbbbnnbbbak", "kbbbbbbbbaak", ".kbbbbbbaak.", ".kabbbbbaak.", "..kkaaaakk..", "...kak.kak..", "............",
];
const STALK: [&str; 16] = [
    ".....kk.....", "....knnk....", ".....kk.....", ".....kk.....", "....kffk....", "...kgffek...", "...kfWWfk...", "...kfkkfk...",
    "...kfffek...", "....kffk....", "...kgfffek..", "..kgffffeek.", "..kfffffeek.", "...kfffek...", "...kek.kek..", "...kk...kk..",
];

const CLOUD: [&str; 6] = ["..kkk.....", ".kssskkk..", "ksssssssk.", "ksssssssk.", ".kkkkkkk..", "..8..8...."];
const HEART: [&str; 6] = [".kk.kk.", "knnknnk", "knnnnnk", ".knnnk.", "..knk..", "...k..."];
const COIN: [&str; 6] = [".kkkk.", "k3332k", "k3222k", "k3222k", "k2221k", ".kkkk."];
const POINTER: [&str; 11] = [
    "k.......", "kk......", "kWk.....", "kWWk....", "kWWWk...", "kWWWWk..", "kWWWWWk.", "kWWWkkkk", "kWkWk...", "kk.kWk..",
    "....kk..",
];

pub struct Sprites {
    pub items: Vec<Sprite>,
    pub guests: Vec<Sprite>,
    pub cloud: Sprite,
    pub heart: Sprite,
    pub coin: Sprite,
    pub pointer: Sprite,
}

impl Sprites {
    pub fn new() -> Self {
        let items: Vec<Sprite> = [&RYE, &BERRY, &MOSS, &BEAN, &BREAD, &SLICED, &ROASTED, &TOAST, &BOWL, &BEANPLATE]
            .iter()
            .map(|a| Sprite::art(&a[..]))
            .collect();
        assert_eq!(items.len(), ITEM_COUNT);
        let mut pointer = Sprite::art(&POINTER);
        for p in pointer.px.iter_mut() {
            if *p == WHITE {
                *p = CURSOR;
            }
        }
        Self {
            items,
            guests: vec![Sprite::art(&CREW), Sprite::art(&BLOB), Sprite::art(&STALK)],
            cloud: Sprite::art(&CLOUD),
            heart: Sprite::art(&HEART),
            coin: Sprite::art(&COIN),
            pointer,
        }
    }

    pub fn item(&self, i: Item) -> &Sprite {
        &self.items[i.index()]
    }

    pub fn guest(&self, s: Species) -> &Sprite {
        &self.guests[s as usize]
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn all_sprites_are_rectangular() {
        let s = super::Sprites::new();
        assert!(s.items.iter().all(|i| i.w == 10 && i.h == 10));
        assert!(s.guests.iter().all(|g| g.w == 12 && g.h == 16));
    }
}
