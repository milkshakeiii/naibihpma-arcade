//! 64-entry indexed palette quantized to 6-bit VGA DAC precision.
//!
//! Indices 2..=47 are scene ramps whose colors change with the section's mood (the "key").
//! Index 0, index 1 (outline) and 48..=63 (newt + HUD) are identical in every mood, so the HUD
//! and the player never shift during a palette wipe.

pub const OUTLINE: u8 = 1;
pub const SKY: u8 = 2; // 5 shades, top -> horizon
pub const FAR: u8 = 7; // 3 distant forest silhouettes
pub const LEAF: u8 = 10; // 4
pub const BARK: u8 = 14; // 4
pub const SOIL: u8 = 18; // 4
pub const MOSS: u8 = 22; // 3
pub const WATER: u8 = 25; // 4: deep, mid, light, surface
pub const DEW: u8 = 29; // 4: shadow, body, light, glint
pub const GLOW: u8 = 33; // 3 spores / fireflies / remembered-drop core
pub const MIST: u8 = 36; // 2
pub const SUN: u8 = 38; // 2
pub const CAVE: u8 = 40; // 4 cavern back wall
pub const REED: u8 = 44; // 3
pub const LILY: u8 = 47;

pub const NEWT_DARK: u8 = 48;
pub const NEWT: u8 = 49;
pub const NEWT_BELLY: u8 = 50;
pub const HUD_SHADOW: u8 = 51;
pub const HUD_FILL: u8 = 52;
pub const HUD_MID: u8 = 53;
pub const HUD_HI: u8 = 54;
pub const TEXT: u8 = 55;
pub const TEXT_DIM: u8 = 56;
pub const SWATCH: u8 = 57; // 57..=61, one per mood
pub const HUD_DEW: u8 = 62;
pub const HUD_DEW_LIGHT: u8 = 63;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Mood {
    Mist,
    Cavern,
    Clearing,
    Swamp,
    Bog,
}

pub const MOODS: [Mood; 5] = [Mood::Mist, Mood::Cavern, Mood::Clearing, Mood::Swamp, Mood::Bog];

impl Mood {
    pub fn parse(s: &str) -> Option<Mood> {
        MOODS.iter().copied().find(|m| m.name() == s)
    }

    pub fn name(self) -> &'static str {
        match self {
            Mood::Mist => "mist",
            Mood::Cavern => "cavern",
            Mood::Clearing => "clearing",
            Mood::Swamp => "swamp",
            Mood::Bog => "bog",
        }
    }

    pub fn index(self) -> usize {
        self as usize
    }

    pub fn swatch(self) -> u8 {
        SWATCH + self as u8
    }
}

type Rgb = (u8, u8, u8);

/// Endpoints (dark, light) for each scene ramp of one mood.
struct MoodColors {
    sky: (Rgb, Rgb),
    far: (Rgb, Rgb),
    leaf: (Rgb, Rgb),
    bark: (Rgb, Rgb),
    soil: (Rgb, Rgb),
    moss: (Rgb, Rgb),
    water: (Rgb, Rgb),
    dew: (Rgb, Rgb),
    glow: (Rgb, Rgb),
    mist: (Rgb, Rgb),
    sun: (Rgb, Rgb),
    cave: (Rgb, Rgb),
    reed: (Rgb, Rgb),
    lily: Rgb,
}

fn colors(mood: Mood) -> MoodColors {
    match mood {
        // Beatless F/Bb canopy mist: violet night, teal foliage, pale bioluminescence.
        Mood::Mist => MoodColors {
            sky: ((12, 10, 36), (76, 64, 120)),
            far: ((22, 22, 52), (44, 44, 84)),
            leaf: ((12, 40, 52), (72, 140, 120)),
            bark: ((36, 28, 44), (112, 88, 104)),
            soil: ((28, 24, 40), (92, 76, 96)),
            moss: ((32, 72, 76), (92, 156, 132)),
            water: ((12, 24, 56), (84, 124, 176)),
            dew: ((64, 104, 168), (236, 248, 255)),
            glow: ((80, 196, 180), (204, 255, 224)),
            mist: ((80, 76, 128), (120, 116, 168)),
            sun: ((96, 96, 140), (124, 124, 168)),
            cave: ((16, 14, 28), (52, 46, 70)),
            reed: ((28, 56, 52), (100, 140, 96)),
            lily: (44, 100, 80),
        },
        // F drone cavern: dimmer indigo with echoing teal droplets.
        Mood::Cavern => MoodColors {
            sky: ((6, 8, 22), (32, 40, 72)),
            far: ((10, 14, 30), (26, 32, 56)),
            leaf: ((8, 30, 40), (44, 104, 104)),
            bark: ((28, 22, 34), (84, 68, 84)),
            soil: ((22, 20, 32), (76, 68, 92)),
            moss: ((24, 56, 64), (68, 128, 120)),
            water: ((8, 20, 48), (60, 104, 156)),
            dew: ((64, 124, 184), (224, 252, 255)),
            glow: ((60, 176, 204), (184, 248, 255)),
            mist: ((40, 48, 84), (60, 72, 108)),
            sun: ((40, 50, 84), (60, 72, 104)),
            cave: ((20, 18, 36), (72, 66, 104)),
            reed: ((20, 44, 44), (76, 112, 84)),
            lily: (32, 80, 68),
        },
        // Bb harp clearing: sunlit gold over bright green.
        Mood::Clearing => MoodColors {
            sky: ((116, 164, 204), (252, 232, 172)),
            far: ((100, 132, 124), (156, 176, 144)),
            leaf: ((36, 92, 40), (156, 204, 92)),
            bark: ((88, 56, 36), (188, 140, 92)),
            soil: ((80, 56, 40), (180, 140, 100)),
            moss: ((72, 136, 48), (180, 220, 100)),
            water: ((40, 96, 108), (148, 212, 204)),
            dew: ((96, 156, 204), (255, 255, 255)),
            glow: ((220, 200, 120), (255, 248, 204)),
            mist: ((220, 220, 204), (240, 236, 220)),
            sun: ((252, 232, 168), (255, 252, 224)),
            cave: ((44, 34, 40), (108, 88, 84)),
            reed: ((72, 120, 48), (172, 204, 100)),
            lily: (84, 156, 76),
        },
        // Nocturnal swamp foley: near-black blue-green with firefly yellow.
        Mood::Swamp => MoodColors {
            sky: ((4, 10, 20), (28, 48, 60)),
            far: ((8, 18, 26), (22, 38, 44)),
            leaf: ((6, 28, 26), (36, 84, 68)),
            bark: ((24, 22, 26), (72, 64, 64)),
            soil: ((18, 20, 22), (64, 64, 60)),
            moss: ((20, 48, 36), (60, 108, 76)),
            water: ((6, 20, 30), (44, 88, 100)),
            dew: ((60, 112, 152), (212, 244, 255)),
            glow: ((180, 200, 60), (252, 255, 164)),
            mist: ((30, 50, 58), (46, 68, 76)),
            sun: ((30, 50, 58), (46, 68, 76)),
            cave: ((10, 12, 18), (42, 42, 52)),
            reed: ((16, 40, 30), (64, 104, 64)),
            lily: (26, 66, 46),
        },
        // Am/D Dorian acid bog: murky green, sickly flood water.
        Mood::Bog => MoodColors {
            sky: ((16, 24, 16), (80, 100, 52)),
            far: ((20, 30, 18), (44, 60, 32)),
            leaf: ((20, 48, 20), (88, 128, 52)),
            bark: ((40, 36, 24), (108, 96, 64)),
            soil: ((36, 32, 20), (104, 92, 60)),
            moss: ((48, 80, 24), (132, 164, 60)),
            water: ((28, 52, 20), (144, 180, 64)),
            dew: ((88, 140, 120), (232, 255, 224)),
            glow: ((160, 200, 40), (232, 255, 124)),
            mist: ((80, 100, 56), (104, 124, 72)),
            sun: ((90, 110, 60), (112, 132, 80)),
            cave: ((16, 20, 12), (56, 64, 40)),
            reed: ((40, 72, 24), (124, 156, 60)),
            lily: (64, 112, 36),
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

pub fn build(mood: Mood) -> [u32; 64] {
    let mut p = [argb((0, 0, 0)); 64];
    let c = colors(mood);
    ramp(&mut p, SKY, 5, c.sky);
    ramp(&mut p, FAR, 3, c.far);
    ramp(&mut p, LEAF, 4, c.leaf);
    ramp(&mut p, BARK, 4, c.bark);
    ramp(&mut p, SOIL, 4, c.soil);
    ramp(&mut p, MOSS, 3, c.moss);
    ramp(&mut p, WATER, 4, c.water);
    ramp(&mut p, DEW, 4, c.dew);
    ramp(&mut p, GLOW, 3, c.glow);
    ramp(&mut p, MIST, 2, c.mist);
    ramp(&mut p, SUN, 2, c.sun);
    ramp(&mut p, CAVE, 4, c.cave);
    ramp(&mut p, REED, 3, c.reed);
    p[LILY as usize] = argb(c.lily);

    let fixed: [(u8, Rgb); 17] = [
        (OUTLINE, (12, 8, 20)),
        (NEWT_DARK, (168, 52, 24)),
        (NEWT, (236, 120, 40)),
        (NEWT_BELLY, (252, 204, 100)),
        (HUD_SHADOW, (28, 22, 30)),
        (HUD_FILL, (60, 48, 56)),
        (HUD_MID, (88, 72, 80)),
        (HUD_HI, (140, 120, 124)),
        (TEXT, (240, 236, 224)),
        (TEXT_DIM, (150, 140, 140)),
        (SWATCH, (100, 92, 172)),
        (SWATCH + 1, (48, 100, 124)),
        (SWATCH + 2, (224, 184, 88)),
        (SWATCH + 3, (40, 84, 76)),
        (SWATCH + 4, (128, 156, 48)),
        (HUD_DEW, (96, 168, 232)),
        (HUD_DEW_LIGHT, (228, 248, 255)),
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
    fn every_entry_is_6bit_quantized_and_fixed_entries_match_across_moods() {
        let all: Vec<[u32; 64]> = MOODS.iter().map(|&m| build(m)).collect();
        for p in &all {
            for &c in p.iter() {
                assert_eq!(c & 0x0003_0303, 0, "channel not quantized to 6 bits: {c:08x}");
            }
        }
        for i in [0usize, 1].into_iter().chain(48..64) {
            assert!(all.iter().all(|p| p[i] == all[0][i]), "fixed index {i} differs between moods");
        }
    }
}
