#[derive(Clone, Copy, Debug)]
pub struct Rgb {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

impl Rgb {
    /// Quantize to 6-bit-per-channel IBM VGA hardware DAC precision (& 0xFC)
    pub const fn vga(r: u8, g: u8, b: u8) -> Self {
        Self {
            r: r & 0xFC,
            g: g & 0xFC,
            b: b & 0xFC,
        }
    }
}

/// Authentic 64-entry 6-bit VGA hardware palette.
/// Index 0 = sprite transparency; Index 1 = 1px dark hull outline.
pub const BASE_PALETTE: [Rgb; 64] = [
    // 0..1: Core
    Rgb::vga(0, 0, 0),       // 0: Transparent / Black
    Rgb::vga(12, 12, 20),    // 1: Dark Hull Outline (#0C0C14)
    // 2..9: Recessed Ground & Terrain Ramps
    Rgb::vga(20, 24, 32),    // 2: Deep Chasm Abyss
    Rgb::vga(28, 32, 44),    // 3: Chasm Rim
    Rgb::vga(36, 40, 38),    // 4: Dark Soil Base
    Rgb::vga(44, 48, 44),    // 5: Mid Soil / Grid
    Rgb::vga(52, 56, 52),    // 6: Light Soil Fleck
    Rgb::vga(64, 68, 64),    // 7: High Ground Stone Dark
    Rgb::vga(84, 88, 84),    // 8: High Ground Stone Mid
    Rgb::vga(108, 112, 108), // 9: High Ground Edge
    // 10..15: Raised Stone & Ore Ramps
    Rgb::vga(72, 60, 48),    // 10: Ore Rock Dark
    Rgb::vga(104, 88, 68),   // 11: Ore Rock Light
    Rgb::vga(124, 116, 100), // 12: Raised Stone Light
    Rgb::vga(92, 84, 72),    // 13: Raised Stone Mid
    Rgb::vga(60, 52, 44),    // 14: Raised Stone Shadow
    Rgb::vga(24, 36, 48),    // 15: Ambient Water / Deep Tile
    // 16..23: Apogee/DOS Beveled HUD Chrome & Text
    Rgb::vga(24, 26, 36),    // 16: HUD Deep Well
    Rgb::vga(40, 44, 58),    // 17: HUD Panel Face
    Rgb::vga(58, 64, 82),    // 18: HUD Mid Slate
    Rgb::vga(92, 100, 120),  // 19: HUD Bevel Highlight
    Rgb::vga(136, 144, 164), // 20: HUD Bright Steel
    Rgb::vga(176, 184, 196), // 21: Muted Text / Trim
    Rgb::vga(248, 248, 240), // 22: Crisp Cream White Text
    Rgb::vga(252, 252, 252), // 23: Pure Highlight White
    // 24..31: Mana, Compute & Ley Blues/Cyans
    Rgb::vga(28, 48, 96),    // 24: Deep Cobalt
    Rgb::vga(44, 84, 156),   // 25: Steel Blue
    Rgb::vga(68, 128, 224),  // 26: Bright Blue (Compute)
    Rgb::vga(116, 184, 252), // 27: Sky Cyan
    Rgb::vga(24, 108, 128),  // 28: Dark Teal (Ley)
    Rgb::vga(56, 200, 212),  // 29: Bright Arcane Cyan
    Rgb::vga(136, 244, 244), // 30: Ley Pulse Flash
    Rgb::vga(96, 232, 252),  // 31: Tesla Electric Cyan
    // 32..39: Emerald, Nature & Dwarven Bronze
    Rgb::vga(28, 72, 40),    // 32: Deep Forest
    Rgb::vga(48, 124, 60),   // 33: Mid Emerald
    Rgb::vga(84, 196, 88),   // 34: Bright Powered Green
    Rgb::vga(148, 244, 128), // 35: Neon Lime
    Rgb::vga(100, 68, 36),   // 36: Bronze Dark
    Rgb::vga(152, 104, 52),  // 37: Bronze Mid
    Rgb::vga(204, 148, 76),  // 38: Brass / Wood
    Rgb::vga(236, 196, 112), // 39: Pale Gold
    // 40..47: Fire, Crimson, Solar Gold & Steel
    Rgb::vga(144, 32, 24),   // 40: Orc Blood Red
    Rgb::vga(220, 76, 32),   // 41: Flame Orange
    Rgb::vga(248, 192, 44),  // 42: Sun Gold (+Force / Leone)
    Rgb::vga(252, 236, 104), // 43: Solar Flare Yellow
    Rgb::vga(180, 188, 200), // 44: Bright Steel (Human)
    Rgb::vga(116, 124, 140), // 45: Mid Steel
    Rgb::vga(160, 208, 236), // 46: Dwarf Mithril
    Rgb::vga(240, 124, 56),  // 47: Amber Warning / Mech
    // 48..55: Void, -Force & Arcane Violet
    Rgb::vga(56, 20, 76),    // 48: Deep Void
    Rgb::vga(96, 36, 136),   // 49: Void Purple (-Force)
    Rgb::vga(152, 68, 204),  // 50: Bright Violet
    Rgb::vga(208, 116, 240), // 51: Magenta Spark
    Rgb::vga(88, 40, 124),   // 52: Arcane Dusk
    Rgb::vga(168, 44, 112),  // 53: Void Crimson
    Rgb::vga(232, 88, 176),  // 54: Dracone Tax Pink
    Rgb::vga(72, 148, 168),  // 55: Ambient Ley Shimmer
    // 56..63: Team Colors (P1 Blue, P2 Red) & Extra Accents
    Rgb::vga(32, 68, 152),   // 56: P1 Banner Dark
    Rgb::vga(56, 120, 236),  // 57: P1 Banner Bright
    Rgb::vga(120, 176, 252), // 58: P1 Banner Trim
    Rgb::vga(228, 224, 212), // 59: Neutral Spark
    Rgb::vga(140, 28, 32),   // 60: P2 Banner Dark
    Rgb::vga(228, 56, 56),   // 61: P2 Banner Bright
    Rgb::vga(252, 116, 96),  // 62: P2 Banner Trim
    Rgb::vga(16, 20, 28),    // 63: Deep Shadow
];

pub fn build_palette_for_tick(tick: u32) -> [u32; 64] {
    let mut lut = [0u32; 64];
    for (i, c) in BASE_PALETTE.iter().enumerate() {
        let r = c.r;
        let mut g = c.g;
        let mut b = c.b;
        // Subtle slow ambient palette cycling on index 55 (Ley spring water shimmer)
        if i == 55 {
            let phase = (tick / 6) % 8;
            let delta = match phase {
                0 | 7 => 0i16,
                1 | 6 => 8,
                2 | 5 => 16,
                _ => 24,
            };
            g = ((g as i16 + delta).min(248) as u8) & 0xFC;
            b = ((b as i16 + delta).min(252) as u8) & 0xFC;
        }
        lut[i] = ((r as u32) << 16) | ((g as u32) << 8) | (b as u32);
    }
    lut
}
