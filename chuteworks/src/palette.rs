//! 64-entry indexed palette quantized to 6-bit VGA DAC precision.
//!
//! Indices 2..=31 are the hall's room ramps and change from hall to hall. Items (32..=54) and
//! the UI (55..=63) never change, so a red item is the same red in every hall.

use crate::model::{Color, MUD};

pub const OUTLINE: u8 = 1;
pub const WALL: u8 = 2; // 4 shades
pub const BENCH: u8 = 6; // 4
pub const FLOOR_L: u8 = 10; // 4
pub const FLOOR_R: u8 = 14; // 4
pub const BELT: u8 = 18; // 3
pub const METAL: u8 = 21; // 4
pub const WOOD: u8 = 25; // 3
pub const GLASS: u8 = 28; // 3
pub const CHALK: u8 = 31;
pub const ITEMS: u8 = 32; // 7 colors x 3 shades, by color bitmask
pub const MUD_D: u8 = 53;
pub const MUD_M: u8 = 54;
pub const TEXT: u8 = 55;
pub const TEXT_DIM: u8 = 56;
pub const ACCENT: u8 = 57;
pub const ACCENT_HI: u8 = 58;
pub const CURSOR: u8 = 59;
pub const GOOD: u8 = 60;
pub const GOOD_DIM: u8 = 61;
pub const BAD: u8 = 62;
pub const WHITE: u8 = 63;

/// (shade, body, light) indices for an item color.
pub fn item_ramp(c: Color) -> (u8, u8, u8) {
    if c >= MUD {
        (MUD_D, MUD_M, MUD_M)
    } else {
        let b = ITEMS + c * 3;
        (b, b + 1, b + 2)
    }
}

type Rgb = (u8, u8, u8);

struct Hall {
    wall: (Rgb, Rgb),
    bench: (Rgb, Rgb),
    floor_l: (Rgb, Rgb),
    floor_r: (Rgb, Rgb),
    belt: (Rgb, Rgb),
    metal: (Rgb, Rgb),
    wood: (Rgb, Rgb),
    glass: (Rgb, Rgb),
    chalk: Rgb,
}

fn hall(n: u8) -> Hall {
    match n {
        // The Switchyard: blue-teal steel at night.
        2 => Hall {
            wall: ((8, 14, 24), (40, 64, 84)),
            bench: ((16, 26, 38), (88, 116, 140)),
            floor_l: ((20, 28, 36), (64, 84, 96)),
            floor_r: ((26, 36, 46), (92, 116, 128)),
            belt: ((10, 12, 16), (52, 60, 68)),
            metal: ((44, 52, 64), (168, 184, 200)),
            wood: ((72, 52, 36), (176, 132, 88)),
            glass: ((12, 24, 32), (152, 212, 228)),
            chalk: (196, 224, 228),
        },
        // The Paint Shop: warm brick and ochre lamplight.
        _ => Hall {
            wall: ((22, 14, 14), (84, 54, 42)),
            bench: ((36, 26, 22), (144, 106, 76)),
            floor_l: ((34, 28, 28), (92, 80, 72)),
            floor_r: ((42, 38, 34), (124, 110, 94)),
            belt: ((14, 12, 14), (60, 56, 56)),
            metal: ((52, 52, 60), (180, 178, 184)),
            wood: ((88, 52, 28), (200, 142, 80)),
            glass: ((18, 26, 34), (164, 204, 214)),
            chalk: (224, 214, 188),
        },
    }
}

#[inline]
fn argb((r, g, b): Rgb) -> u32 {
    let (r, g, b) = ((r & 0xFC) as u32, (g & 0xFC) as u32, (b & 0xFC) as u32);
    0xFF00_0000 | (r << 16) | (g << 8) | b
}

fn ramp(p: &mut [u32; 64], start: u8, n: u8, (a, b): (Rgb, Rgb)) {
    let n = n as i32;
    for i in 0..n {
        let lerp = |x: u8, y: u8| (x as i32 + (y as i32 - x as i32) * i / (n - 1).max(1)) as u8;
        p[(start as i32 + i) as usize] = argb((lerp(a.0, b.0), lerp(a.1, b.1), lerp(a.2, b.2)));
    }
}

pub fn build(hall_no: u8) -> [u32; 64] {
    let mut p = [argb((0, 0, 0)); 64];
    let h = hall(hall_no);
    ramp(&mut p, WALL, 4, h.wall);
    ramp(&mut p, BENCH, 4, h.bench);
    ramp(&mut p, FLOOR_L, 4, h.floor_l);
    ramp(&mut p, FLOOR_R, 4, h.floor_r);
    ramp(&mut p, BELT, 3, h.belt);
    ramp(&mut p, METAL, 4, h.metal);
    ramp(&mut p, WOOD, 3, h.wood);
    ramp(&mut p, GLASS, 3, h.glass);
    p[CHALK as usize] = argb(h.chalk);

    // By color bitmask: grey, red, yellow, orange, blue, purple, green.
    let items: [[Rgb; 3]; 7] = [
        [(68, 68, 76), (128, 128, 136), (192, 192, 200)],
        [(116, 20, 24), (196, 48, 44), (248, 116, 96)],
        [(148, 108, 16), (228, 188, 40), (252, 236, 132)],
        [(148, 60, 16), (232, 120, 32), (252, 184, 100)],
        [(24, 40, 124), (52, 96, 212), (128, 172, 252)],
        [(76, 28, 112), (144, 64, 192), (204, 136, 240)],
        [(20, 84, 40), (56, 160, 64), (144, 224, 120)],
    ];
    for (c, shades) in items.iter().enumerate() {
        for (s, rgb) in shades.iter().enumerate() {
            p[ITEMS as usize + c * 3 + s] = argb(*rgb);
        }
    }
    let fixed: [(u8, Rgb); 12] = [
        (OUTLINE, (12, 8, 16)),
        (MUD_D, (64, 46, 28)),
        (MUD_M, (112, 84, 52)),
        (TEXT, (240, 232, 212)),
        (TEXT_DIM, (148, 138, 126)),
        (ACCENT, (200, 148, 52)),
        (ACCENT_HI, (248, 216, 120)),
        (CURSOR, (252, 248, 216)),
        (GOOD, (100, 228, 120)),
        (GOOD_DIM, (36, 80, 52)),
        (BAD, (240, 88, 64)),
        (WHITE, (255, 255, 255)),
    ];
    for (i, rgb) in fixed {
        p[i as usize] = argb(rgb);
    }
    p
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_entry_is_6bit_and_items_and_ui_match_across_halls() {
        let a = build(1);
        let b = build(2);
        for p in [&a, &b] {
            for &c in p.iter() {
                assert_eq!(c & 0x0003_0303, 0, "channel not quantized to 6 bits: {c:08x}");
            }
        }
        for i in [0usize, 1].into_iter().chain(32..64) {
            assert_eq!(a[i], b[i], "fixed index {i} differs between halls");
        }
    }
}
