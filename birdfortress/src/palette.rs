//! One 64-entry palette quantized to 6-bit VGA DAC precision (`channel & 0xFC`).

pub type Pal = [u32; 64];
pub type Rgb = (u8, u8, u8);

pub const OUTLINE: u8 = 1;
/// Sky, top (deepest) to horizon: five bands.
pub const SKY: u8 = 2;
/// Distant hills: dark, mid.
pub const HILL: u8 = 7;
pub const GRASS_DK: u8 = 9;
pub const GRASS: u8 = 10;
pub const GRASS_HI: u8 = 11;
pub const SOIL_DK: u8 = 12;
pub const SOIL: u8 = 13;
pub const SOIL_HI: u8 = 14;
pub const ROCK_DK: u8 = 15;
pub const ROCK: u8 = 16;
pub const ROCK_HI: u8 = 17;
pub const THORN_DK: u8 = 18;
pub const THORN: u8 = 19;
pub const WOOD_DK: u8 = 20;
pub const WOOD: u8 = 21;
pub const WOOD_HI: u8 = 22;
pub const LEAF_DK: u8 = 23;
pub const LEAF: u8 = 24;
pub const LEAF_HI: u8 = 25;
pub const VINE: u8 = 26;
/// Water: four cycling shades.
pub const WATER: u8 = 27;
pub const FOAM: u8 = 31;
pub const TRIB_DK: u8 = 32;
pub const TRIB: u8 = 33;
pub const TRIB_HI: u8 = 34;
pub const DODO_DK: u8 = 35;
pub const DODO: u8 = 36;
pub const DODO_HI: u8 = 37;
pub const BEAK: u8 = 38;
pub const APE_DK: u8 = 39;
pub const APE: u8 = 40;
pub const APE_FACE: u8 = 41;
pub const BERRY: u8 = 42;
pub const CLAY: u8 = 43;
pub const EGG: u8 = 44;
pub const BRICK: u8 = 45;
pub const POT: u8 = 46;
pub const SPRING: u8 = 47;

// Shared UI block.
pub const PANEL_LO: u8 = 48;
pub const PANEL: u8 = 49;
pub const PANEL_HI: u8 = 50;
pub const PANEL_DEEP: u8 = 51;
pub const TEXT: u8 = 52;
pub const TEXT_DIM: u8 = 53;
pub const TEXT_FAINT: u8 = 54;
pub const ACCENT: u8 = 55;
pub const ACCENT_HI: u8 = 56;
pub const GOOD: u8 = 57;
pub const BAD: u8 = 58;
pub const BAD_DIM: u8 = 59;
pub const BANNER: u8 = 60;
pub const GOLD: u8 = 61;
pub const SHADOW: u8 = 62;
pub const WHITE: u8 = 63;

#[inline]
pub fn argb((r, g, b): Rgb) -> u32 {
    let (r, g, b) = ((r & 0xFC) as u32, (g & 0xFC) as u32, (b & 0xFC) as u32);
    0xFF00_0000 | (r << 16) | (g << 8) | b
}

fn set(p: &mut Pal, i: u8, rgb: Rgb) {
    p[i as usize] = argb(rgb);
}

const WATER_SHADES: [Rgb; 4] = [(28, 64, 120), (40, 88, 152), (56, 112, 176), (40, 88, 152)];

pub fn game() -> Pal {
    let mut p = [argb((0, 0, 0)); 64];
    set(&mut p, OUTLINE, (16, 12, 20));
    for (i, c) in [(44, 60, 112), (64, 92, 148), (92, 128, 180), (132, 168, 204), (176, 200, 216)].into_iter().enumerate() {
        set(&mut p, SKY + i as u8, c);
    }
    set(&mut p, HILL, (76, 108, 132));
    set(&mut p, HILL + 1, (100, 136, 148));
    set(&mut p, GRASS_DK, (40, 92, 44));
    set(&mut p, GRASS, (72, 140, 56));
    set(&mut p, GRASS_HI, (128, 188, 72));
    set(&mut p, SOIL_DK, (64, 40, 28));
    set(&mut p, SOIL, (100, 64, 40));
    set(&mut p, SOIL_HI, (136, 96, 60));
    set(&mut p, ROCK_DK, (72, 72, 84));
    set(&mut p, ROCK, (116, 116, 128));
    set(&mut p, ROCK_HI, (164, 164, 172));
    set(&mut p, THORN_DK, (56, 28, 48));
    set(&mut p, THORN, (120, 56, 84));
    set(&mut p, WOOD_DK, (80, 48, 24));
    set(&mut p, WOOD, (140, 92, 48));
    set(&mut p, WOOD_HI, (196, 148, 88));
    set(&mut p, LEAF_DK, (28, 72, 48));
    set(&mut p, LEAF, (44, 108, 60));
    set(&mut p, LEAF_HI, (84, 152, 76));
    set(&mut p, VINE, (96, 176, 64));
    for (i, c) in WATER_SHADES.into_iter().enumerate() {
        set(&mut p, WATER + i as u8, c);
    }
    set(&mut p, FOAM, (188, 220, 236));
    set(&mut p, TRIB_DK, (164, 84, 92));
    set(&mut p, TRIB, (228, 140, 140));
    set(&mut p, TRIB_HI, (252, 200, 188));
    set(&mut p, DODO_DK, (64, 80, 112));
    set(&mut p, DODO, (112, 132, 168));
    set(&mut p, DODO_HI, (176, 192, 216));
    set(&mut p, BEAK, (240, 184, 64));
    set(&mut p, APE_DK, (68, 40, 32));
    set(&mut p, APE, (120, 76, 52));
    set(&mut p, APE_FACE, (212, 168, 124));
    set(&mut p, BERRY, (216, 40, 72));
    set(&mut p, CLAY, (196, 112, 60));
    set(&mut p, EGG, (244, 236, 212));
    set(&mut p, BRICK, (176, 64, 44));
    set(&mut p, POT, (220, 132, 76));
    set(&mut p, SPRING, (232, 200, 72));

    set(&mut p, PANEL_LO, (52, 40, 32));
    set(&mut p, PANEL, (88, 68, 48));
    set(&mut p, PANEL_HI, (140, 112, 80));
    set(&mut p, PANEL_DEEP, (24, 18, 16));
    set(&mut p, TEXT, (240, 232, 212));
    set(&mut p, TEXT_DIM, (184, 168, 140));
    set(&mut p, TEXT_FAINT, (120, 104, 84));
    set(&mut p, ACCENT, (120, 212, 236));
    set(&mut p, ACCENT_HI, (200, 244, 252));
    set(&mut p, GOOD, (120, 224, 108));
    set(&mut p, BAD, (236, 84, 64));
    set(&mut p, BAD_DIM, (120, 40, 36));
    set(&mut p, BANNER, (200, 52, 56));
    set(&mut p, GOLD, (252, 208, 72));
    set(&mut p, SHADOW, (36, 44, 72));
    set(&mut p, WHITE, (255, 255, 255));
    p
}

/// Slow water shimmer: the four water shades rotate every few frames (ambience only).
pub fn cycle_water(p: &mut Pal, frame: u32) {
    let k = (frame / 6) as usize;
    for i in 0..4 {
        p[(WATER + i as u8) as usize] = argb(WATER_SHADES[(i + k) % 4]);
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn every_entry_is_6bit() {
        for &c in super::game().iter() {
            assert_eq!(c & 0x0003_0303, 0, "channel not quantized to 6 bits: {c:08x}");
        }
    }
}
