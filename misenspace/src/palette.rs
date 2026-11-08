//! 64-entry indexed palette quantized to 6-bit VGA DAC precision.

pub const OUTLINE: u8 = 1;
pub const SPACE: u8 = 2; // 4: deep space, darkest first
pub const FRAME: u8 = 6; // 4: steel frame and walls
pub const KFLOOR: u8 = 10; // 4: kitchen tiles
pub const DFLOOR: u8 = 14; // 4: dining-room carpet
pub const BELT: u8 = 18; // 3
pub const METAL: u8 = 21; // 4
pub const WOOD: u8 = 25; // 3
pub const GLASS: u8 = 28; // 3
pub const STAR: u8 = 31;
pub const GOLD: u8 = 32; // 3
pub const BROWN: u8 = 35; // 3
pub const ICE: u8 = 38; // 3
pub const GREEN: u8 = 41; // 3
pub const PURP: u8 = 44; // 3
pub const PLATE: u8 = 47;
pub const PLATE_SH: u8 = 48;
pub const ORANGE: u8 = 49; // 3
pub const RED: u8 = 52;
pub const PINK: u8 = 53;
pub const VISOR: u8 = 54;
pub const TEXT: u8 = 55;
pub const TEXT_DIM: u8 = 56;
pub const ACCENT: u8 = 57;
pub const ACCENT_HI: u8 = 58;
pub const CURSOR: u8 = 59;
pub const GOOD: u8 = 60;
pub const GOOD_DIM: u8 = 61;
pub const FIRE: u8 = 62;
pub const WHITE: u8 = 63;

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
    ramp(&mut p, SPACE, 4, ((6, 8, 20), (40, 44, 76)));
    ramp(&mut p, FRAME, 4, ((26, 30, 42), (116, 124, 146)));
    ramp(&mut p, KFLOOR, 4, ((26, 34, 42), (88, 104, 116)));
    ramp(&mut p, DFLOOR, 4, ((48, 28, 34), (140, 92, 78)));
    ramp(&mut p, BELT, 3, ((14, 14, 18), (78, 78, 90)));
    ramp(&mut p, METAL, 4, ((54, 58, 70), (200, 204, 214)));
    ramp(&mut p, WOOD, 3, ((84, 50, 30), (200, 146, 92)));
    ramp(&mut p, GLASS, 3, ((16, 30, 52), (150, 200, 232)));
    p[STAR as usize] = argb((236, 240, 255));
    let ramps: [(u8, Rgb, Rgb, Rgb); 6] = [
        (GOLD, (140, 96, 16), (224, 172, 40), (252, 232, 124)),
        (BROWN, (92, 52, 24), (156, 96, 46), (212, 156, 92)),
        (ICE, (36, 84, 156), (104, 168, 232), (212, 240, 255)),
        (GREEN, (28, 88, 48), (72, 168, 80), (164, 232, 132)),
        (PURP, (72, 30, 104), (144, 72, 184), (212, 152, 240)),
        (ORANGE, (148, 60, 20), (232, 120, 40), (252, 192, 112)),
    ];
    for (start, a, b, c) in ramps {
        p[start as usize] = argb(a);
        p[start as usize + 1] = argb(b);
        p[start as usize + 2] = argb(c);
    }
    let fixed: [(u8, Rgb); 14] = [
        (OUTLINE, (10, 8, 16)),
        (PLATE, (236, 232, 224)),
        (PLATE_SH, (168, 168, 180)),
        (RED, (224, 64, 52)),
        (PINK, (252, 152, 176)),
        (VISOR, (120, 228, 252)),
        (TEXT, (240, 232, 212)),
        (TEXT_DIM, (148, 140, 132)),
        (ACCENT, (200, 148, 52)),
        (ACCENT_HI, (248, 216, 120)),
        (CURSOR, (252, 248, 216)),
        (GOOD, (104, 228, 120)),
        (GOOD_DIM, (36, 80, 52)),
        (FIRE, (252, 140, 40)),
    ];
    for (i, rgb) in fixed {
        p[i as usize] = argb(rgb);
    }
    p[WHITE as usize] = argb((255, 255, 255));
    p
}

#[cfg(test)]
mod tests {
    #[test]
    fn every_entry_is_6bit() {
        for &c in super::build().iter() {
            assert_eq!(c & 0x0003_0303, 0, "channel not quantized to 6 bits: {c:08x}");
        }
    }
}
