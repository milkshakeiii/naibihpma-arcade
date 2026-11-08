//! One 64-entry palette quantized to 6-bit VGA DAC precision (`channel & 0xFC`).

pub type Pal = [u32; 64];
pub type Rgb = (u8, u8, u8);

pub const OUTLINE: u8 = 1;
pub const SPACE: u8 = 2;
/// Three star shades, dimmest first.
pub const STAR: u8 = 3;
/// Five rock shades: shadow edge, base, speck, lit edge, lit speck.
pub const ROCK: u8 = 6;
/// Team ramps: dark, mid, light, highlight. Player at 12, enemy at 16.
pub const TEAM: u8 = 12;
pub const STEEL: u8 = 20;
pub const STEEL_DK: u8 = 21;
pub const GUN: u8 = 22;
pub const ENGINE: u8 = 23;
pub const SCOOP: u8 = 24;
pub const CORE: u8 = 25;
pub const LOOSE: u8 = 26;

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
pub const GOLD: u8 = 61;
pub const WHITE: u8 = 63;

/// A team colour: shade 0 dark, 1 mid, 2 light, 3 highlight.
pub fn team_col(team: usize, shade: u8) -> u8 {
    TEAM + team as u8 * 4 + shade.min(3)
}

#[inline]
pub fn argb((r, g, b): Rgb) -> u32 {
    let (r, g, b) = ((r & 0xFC) as u32, (g & 0xFC) as u32, (b & 0xFC) as u32);
    0xFF00_0000 | (r << 16) | (g << 8) | b
}

fn set(p: &mut Pal, i: u8, rgb: Rgb) {
    p[i as usize] = argb(rgb);
}

pub fn game() -> Pal {
    let mut p = [argb((0, 0, 0)); 64];
    set(&mut p, OUTLINE, (8, 8, 14));
    set(&mut p, SPACE, (6, 8, 22));
    for (i, c) in [(30, 34, 60), (70, 76, 112), (150, 156, 196)].into_iter().enumerate() {
        set(&mut p, STAR + i as u8, c);
    }
    for (i, c) in [(40, 32, 34), (66, 54, 50), (88, 74, 62), (124, 106, 86), (150, 132, 106)].into_iter().enumerate() {
        set(&mut p, ROCK + i as u8, c);
    }
    for (i, c) in [(24, 56, 120), (52, 116, 210), (112, 180, 248), (196, 232, 255)].into_iter().enumerate() {
        set(&mut p, TEAM + i as u8, c);
    }
    for (i, c) in [(112, 24, 32), (204, 56, 48), (244, 124, 84), (255, 204, 156)].into_iter().enumerate() {
        set(&mut p, TEAM + 4 + i as u8, c);
    }
    set(&mut p, STEEL, (136, 144, 160));
    set(&mut p, STEEL_DK, (84, 90, 104));
    set(&mut p, GUN, (248, 224, 88));
    set(&mut p, ENGINE, (172, 236, 252));
    set(&mut p, SCOOP, (232, 140, 40));
    set(&mut p, CORE, (255, 255, 255));
    set(&mut p, LOOSE, (216, 204, 140));

    set(&mut p, PANEL_LO, (36, 40, 56));
    set(&mut p, PANEL, (60, 66, 88));
    set(&mut p, PANEL_HI, (108, 116, 144));
    set(&mut p, PANEL_DEEP, (14, 16, 26));
    set(&mut p, TEXT, (232, 234, 242));
    set(&mut p, TEXT_DIM, (152, 158, 182));
    set(&mut p, TEXT_FAINT, (92, 98, 122));
    set(&mut p, ACCENT, (120, 220, 240));
    set(&mut p, ACCENT_HI, (200, 248, 252));
    set(&mut p, GOOD, (104, 224, 120));
    set(&mut p, BAD, (236, 76, 64));
    set(&mut p, BAD_DIM, (120, 36, 40));
    set(&mut p, GOLD, (248, 204, 72));
    set(&mut p, WHITE, (255, 255, 255));
    p
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
