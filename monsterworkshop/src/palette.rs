//! 64-entry indexed palette quantized to 6-bit VGA DAC precision.
//!
//! The yard has one fixed palette: every index means the same color everywhere. Ramps run
//! dark -> light so `BASE + n` reads as "n steps brighter".

pub const OUTLINE: u8 = 1;
pub const GRASS: u8 = 2; // 4
pub const PATH: u8 = 6; // 3
pub const HEDGE: u8 = 9; // 3
pub const WOOD: u8 = 12; // 4 walls, bins, wheel, spool flanges
pub const PLANK: u8 = 16; // 4 workshop floor
pub const MAT: u8 = 20; // 3 woven pads
pub const COTTON: u8 = 23; // 3 shadow, body, light
pub const THREAD: u8 = 26; // 3 madder-red thread
pub const LEAF: u8 = 29; // 3 cotton plants
pub const GOBLIN: u8 = 32; // 3
pub const CYCLOPS: u8 = 35; // 3
pub const EYE: u8 = 38;
pub const TOOTH: u8 = 39;
pub const SPARK: u8 = 40; // 3 sparkles, progress fill
pub const REC: u8 = 43; // 2 recording red
pub const LOOP: u8 = 45; // 2 loop cyan
pub const DUST: u8 = 47;
pub const SHADOW: u8 = 48; // dithered ground shadow
pub const HUD_SHADOW: u8 = 49;
pub const HUD_FILL: u8 = 50;
pub const HUD_MID: u8 = 51;
pub const HUD_HI: u8 = 52;
pub const TEXT: u8 = 53;
pub const TEXT_DIM: u8 = 54;
pub const TEXT_GOOD: u8 = 55;
pub const TEXT_WARN: u8 = 56;
pub const BUBBLE: u8 = 57;
pub const INK: u8 = 58; // dark text inside speech bubbles
pub const FLOWER: u8 = 59; // 2 decorative yard flowers

type Rgb = (u8, u8, u8);

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

pub fn build() -> [u32; 64] {
    let mut p = [argb((0, 0, 0)); 64];
    ramp(&mut p, GRASS, 4, ((28, 64, 36), (104, 156, 72)));
    ramp(&mut p, PATH, 3, ((96, 72, 48), (168, 136, 96)));
    ramp(&mut p, HEDGE, 3, ((14, 36, 26), (52, 96, 52)));
    ramp(&mut p, WOOD, 4, ((56, 34, 24), (180, 128, 78)));
    ramp(&mut p, PLANK, 4, ((84, 58, 40), (164, 124, 84)));
    ramp(&mut p, MAT, 3, ((104, 82, 52), (204, 176, 116)));
    ramp(&mut p, COTTON, 3, ((188, 184, 204), (252, 252, 248)));
    ramp(&mut p, THREAD, 3, ((120, 30, 44), (236, 112, 104)));
    ramp(&mut p, LEAF, 3, ((22, 84, 40), (116, 188, 72)));
    ramp(&mut p, GOBLIN, 3, ((40, 88, 36), (152, 208, 84)));
    ramp(&mut p, CYCLOPS, 3, ((68, 44, 116), (180, 140, 224)));
    ramp(&mut p, SPARK, 3, ((200, 136, 36), (255, 244, 148)));
    ramp(&mut p, REC, 2, ((152, 26, 34), (240, 76, 64)));
    ramp(&mut p, LOOP, 2, ((36, 132, 156), (128, 224, 232)));
    ramp(&mut p, FLOWER, 2, ((200, 100, 160), (252, 220, 120)));

    let fixed: [(u8, Rgb); 15] = [
        (OUTLINE, (16, 12, 22)),
        (EYE, (248, 244, 232)),
        (TOOTH, (236, 228, 196)),
        (DUST, (156, 132, 100)),
        (SHADOW, (18, 26, 20)),
        (HUD_SHADOW, (26, 20, 30)),
        (HUD_FILL, (58, 46, 54)),
        (HUD_MID, (86, 70, 78)),
        (HUD_HI, (140, 120, 122)),
        (TEXT, (240, 236, 224)),
        (TEXT_DIM, (152, 142, 140)),
        (TEXT_GOOD, (140, 222, 120)),
        (TEXT_WARN, (248, 176, 72)),
        (BUBBLE, (248, 244, 226)),
        (INK, (30, 24, 38)),
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
    fn every_entry_is_6bit_quantized() {
        for &c in build().iter() {
            assert_eq!(c & 0x0003_0303, 0, "channel not quantized to 6 bits: {c:08x}");
        }
    }

    #[test]
    fn ramps_do_not_overlap() {
        let ramps: [(u8, u8); 18] = [
            (GRASS, 4), (PATH, 3), (HEDGE, 3), (WOOD, 4), (PLANK, 4), (MAT, 3), (COTTON, 3), (THREAD, 3), (LEAF, 3),
            (GOBLIN, 3), (CYCLOPS, 3), (EYE, 1), (TOOTH, 1), (SPARK, 3), (REC, 2), (LOOP, 2), (DUST, 1), (SHADOW, 1),
        ];
        let mut used = [false; 64];
        for (start, n) in ramps {
            for i in start..start + n {
                assert!(!used[i as usize], "index {i} assigned twice");
                used[i as usize] = true;
            }
        }
        assert!(!used[0] && !used[OUTLINE as usize]);
    }
}
