//! Hand-drawn sprites: creatures, goods, pointers and tool icons.

use crate::framebuffer::Sprite;
use crate::palette::*;
use crate::rules::*;

pub fn key(c: char) -> u8 {
    match c {
        'k' => OUTLINE,
        'W' => WHITE,
        'p' => TRIB_DK,
        'P' => TRIB,
        'Q' => TRIB_HI,
        'd' => DODO_DK,
        'D' => DODO,
        'E' => DODO_HI,
        'y' => BEAK,
        'a' => APE_DK,
        'A' => APE,
        'f' => APE_FACE,
        'r' => BERRY,
        'g' => GRASS,
        'G' => GRASS_HI,
        'l' => LEAF,
        'L' => LEAF_HI,
        'w' => WOOD_DK,
        'o' => WOOD,
        'O' => WOOD_HI,
        's' => ROCK_DK,
        'S' => ROCK,
        'T' => ROCK_HI,
        'c' => CLAY,
        'e' => EGG,
        'b' => BRICK,
        'u' => POT,
        'm' => SOIL,
        'M' => SOIL_HI,
        'x' => SPRING,
        'B' => BANNER,
        'Y' => GOLD,
        'z' => BAD,
        'v' => VINE,
        't' => TEXT,
        'h' => TEXT_DIM,
        'n' => PANEL_DEEP,
        'i' => ACCENT,
        _ => 0,
    }
}

fn art(rows: &[&str]) -> Sprite {
    Sprite::art(rows, key)
}

pub struct Art {
    /// [species][frame]: 0,1 walk, 2 jump/fall, 3 climb.
    pub creatures: Vec<Vec<Sprite>>,
    /// Indexed by `Good::index()`.
    pub goods: Vec<Sprite>,
    pub pointer: Sprite,
    pub hand: Sprite,
    /// Score icons: ticks, seeds, crew.
    pub clock: Sprite,
    pub seed: Sprite,
    pub paw: Sprite,
    /// "Carrying nothing", for the load columns of a creature card.
    pub empty: Sprite,
}

impl Art {
    pub fn new() -> Art {
        let tribble = vec![
            art(&["..kkkk..", ".kQQQPk.", "kQPPPPPk", "kPkPPkPk", "kPPPPPPk", "kpPPPPpk", ".kkkkkk."]),
            art(&["........", "..kkkk..", ".kQQQPk.", "kPkPPkPk", "kPPPPPPk", "kpPPPPpk", "kkkkkkkk"]),
            art(&[".k.kk.k.", "kQkQQkPk", "kQPPPPPk", "kPkPPkPk", "kPPPPPPk", "kpPPPPpk", ".kk..kk."]),
            art(&["..kkkk..", ".kQQQPk.", "kQPPPPPk", "kPPPPPPk", "kPPPPPPk", "kpPPPPpk", ".kkkkkk."]),
        ];
        let dodo = vec![
            art(&["...kkk...", "..kEEEk..", "..kEkEkkk", "..kDEEyyk", ".kkDDkkkk", "kdDDDDk..", "kdDDDDDk.", "kddDDDdk.", ".kkdddk..", "..ky.ky.."]),
            art(&["...kkk...", "..kEEEk..", "..kEkEkkk", "..kDEEyyk", ".kkDDkkkk", "kdDDDDk..", "kdDDDDDk.", "kddDDDdk.", ".kkdddk..", "...kyky.."]),
            art(&["...kkk...", "..kEEEk..", "..kEkEkkk", "k.kDEEyyk", "dkkDDkkkk", "kdDDDDk..", "kdDDDDDk.", "kddDDDdk.", ".kkdddk..", ".ky...ky."]),
            art(&["...kkk...", "..kEEEk..", "..kEEEk..", "..kDEEk..", ".kkDDDkk.", "kdDDDDDdk", "kdDDDDDk.", "kddDDDdk.", ".kkdddk..", "..ky.ky.."]),
        ];
        let ape = vec![
            art(&["..kkkk..", ".kAAAAk.", ".kAffffk", ".kAfkfkk", ".kAffffk", "kAAkkkAk", "kAkAAAkA", ".kAAAAk.", ".kak.kak"]),
            art(&["..kkkk..", ".kAAAAk.", ".kAffffk", ".kAfkfkk", ".kAffffk", "kAAkkkAk", "kAkAAAkA", ".kAAAAk.", "..kakak."]),
            art(&["kk.kkkk.", "kAkAAAAk", ".kAffffk", ".kAfkfkk", ".kAffffk", ".kAkkkAk", ".kAAAAAk", ".kAAAAk.", "kak..kak"]),
            art(&["kk....kk", "kAkkkkAk", "kAAAAAAk", ".kAAAAk.", ".kAAAAk.", ".kAAAAk.", "kAAAAAAk", "kAkAAkAk", "kak..kak"]),
        ];
        let goods = vec![
            art(&["..g..", ".rWr.", "rrrrr", "rrrrr", ".rrr."]).outlined(),
            art(&["....O", "...o.", "..oO.", ".o...", "o...."]).outlined(),
            art(&[".STT.", "SSSST", "sSSSS", "ssSSS", ".sss."]).outlined(),
            art(&[".cc..", "ccMc.", "ccccc", "ccccc", ".ccc."]).outlined(),
            art(&[".ee.", "eeWe", "eeee", "eeee", ".ee."]).outlined(),
            art(&["bbbbW", "bbbbb", "bbbbb"]).outlined(),
            art(&["OoOoO", "oOoOo", "OoOoO", "oOoOo"]).outlined(),
            art(&[".uuu.", "..u..", ".uuu.", "uuWuu", ".uuu."]).outlined(),
        ];
        let pointer = art(&[
            "k.........",
            "kk........",
            "kWk.......",
            "kWWk......",
            "kWWWk.....",
            "kWWWWk....",
            "kWWWWWk...",
            "kWWWWWWk..",
            "kWWWWWWWk.",
            "kWWWWWkkkk",
            "kWWkWWk...",
            "kWk.kWWk..",
            "kk..kWWk..",
            "k....kWWk.",
            ".....kWWk.",
            "......kk..",
        ]);
        let hand = art(&[
            "....kk......",
            "...kWWk.....",
            "...kWWk.....",
            "...kWWkkk...",
            "...kWWkWWkk.",
            "kk.kWWkWWkWk",
            "kWkkWWWWWkWk",
            "kWWkWWWWWWWk",
            ".kWWWWWWWWWk",
            "..kWWWWWWWk.",
            "...kWWWWWWk.",
            "....kWWWWk..",
            "....kkkkkk..",
        ]);
        let clock = art(&[".kkkkk.", "kWWkWWk", "kWWkWWk", "kWWkkWk", "kWWWWWk", "kWWWWWk", ".kkkkk."]);
        let seed = art(&["...k...", "..kwk..", ".kOOOk.", "kOOOOOk", "kooooOk", ".kooOk.", "..kkk.."]);
        let paw = art(&[".k.k.k.", "kPkPkPk", ".k.k.k.", "..kkk..", ".kPPPk.", ".kPPPk.", "..kkk.."]);
        let empty = art(&[".h.h.", "h...h", ".....", "h...h", ".h.h."]);
        Art { creatures: vec![tribble, dodo, ape], goods, pointer, hand, clock, seed, paw, empty }
    }

    pub fn creature(&self, s: Species, frame: usize) -> &Sprite {
        &self.creatures[s as usize][frame.min(3)]
    }

    pub fn good(&self, g: Good) -> &Sprite {
        &self.goods[g.index()]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn creature_feet_are_planted() {
        let a = Art::new();
        for s in SPECIES {
            for f in 0..4 {
                let spr = a.creature(s, f);
                assert_eq!(spr.bottom_row(), spr.h - 1, "{s:?} frame {f} floats");
                assert!(spr.h <= 10 && spr.w <= 10, "{s:?} frame {f} is bigger than a tile");
            }
        }
    }
}
