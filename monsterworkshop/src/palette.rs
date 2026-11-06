//! Indexed palette quantized to 6-bit VGA DAC precision.
//!
//! Its first 64 colors mean the same everywhere: goods, monsters, props and the interface. The
//! rest is a land bank each province fills with its own ground, foliage, paths, water, flowers,
//! walls and floors, reloaded as the view enters one of its screens, as VGA games reloaded the
//! DAC for each level (Henry, 2026-11-04). Ramps run dark -> light so `BASE + n` reads as "n
//! steps brighter".

/// How many colors the palette holds: 64 fixed, and a land bank of 64.
pub const PALETTE_SIZE: usize = 128;
pub type Palette = [u32; PALETTE_SIZE];

pub const OUTLINE: u8 = 1;
// 2..=5 and 9..=11 are free: the grass and hedges moved to the land bank.
pub const PATH: u8 = 6; // 3 stone and gravel goods; the land's paths are LAND_PATH
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
pub const WATER: u8 = 61; // 3 indigo and water-blue goods; the land's water is LAND_WATER

// The land bank.
pub const GRASS: u8 = 64; // 4 ground
pub const HEDGE: u8 = 68; // 3 hedges and trees not for gathering
pub const LAND_PATH: u8 = 71; // 3 paths and the cobbles' mortar
pub const LAND_WATER: u8 = 74; // 3 rivers, lakes and ponds
pub const LAND_FLOWER: u8 = 77; // 2 flowers in the grass and in beds
pub const WALL: u8 = 79; // 4 building walls
pub const FLOOR: u8 = 83; // 4 building floors

/// Every multi-shade ramp, dark to light, so a sprite can be brightened or dimmed by
/// shifting its palette indices within their ramps (no alpha, per AGENTS.md).
const RAMPS: [(u8, u8); 21] = [
    (LAND_PATH, 3),
    (LAND_WATER, 3),
    (LAND_FLOWER, 2),
    (WALL, 4),
    (FLOOR, 4),
    (GRASS, 4),
    (PATH, 3),
    (HEDGE, 3),
    (WOOD, 4),
    (PLANK, 4),
    (MAT, 3),
    (COTTON, 3),
    (THREAD, 3),
    (LEAF, 3),
    (GOBLIN, 3),
    (CYCLOPS, 3),
    (SPARK, 3),
    (REC, 2),
    (LOOP, 2),
    (FLOWER, 2),
    (WATER, 3),
];

/// Whether `c` is in the lighter half of its ramp. A color of its own counts as light.
pub fn is_light(c: u8) -> bool {
    RAMPS.iter().find(|&&(start, n)| c >= start && c < start + n).is_none_or(|&(start, n)| c - start >= n.div_ceil(2))
}

/// `c` moved `k` steps lighter (or darker, if negative) within its ramp.
pub fn shift(c: u8, k: i32) -> u8 {
    for (start, n) in RAMPS {
        if c >= start && c < start + n {
            return (c as i32 + k).clamp(start as i32, (start + n - 1) as i32) as u8;
        }
    }
    c
}

/// A shade of a ramp by its name, as the data gives a piece of equipment's color: ("thread", 1)
/// is the madder red's middle. Past a ramp's last shade, its lightest.
pub fn color_named(name: &str, shade: u8) -> Option<u8> {
    let start = match name {
        "spark" => SPARK,
        "goblin" => GOBLIN,
        "cyclops" => CYCLOPS,
        "path" => PATH,
        _ => ramp_named(name)?.0,
    };
    RAMPS.iter().find(|(s, _)| *s == start).map(|&(s, n)| s + shade.min(n - 1))
}

/// A ramp by the name the data gives a variety's color ("plank", "water"): its first index and
/// how many shades it has.
pub fn ramp_named(name: &str) -> Option<(u8, u8)> {
    let start = match name {
        "grass" => GRASS,
        "hedge" => HEDGE,
        "wood" => WOOD,
        "plank" => PLANK,
        "mat" => MAT,
        "cotton" => COTTON,
        "thread" => THREAD,
        "leaf" => LEAF,
        "flower" => FLOWER,
        "water" => WATER,
        _ => return None,
    };
    RAMPS.iter().find(|(s, _)| *s == start).copied()
}

/// Recolors one ramp's pixels into another: a dyed good's cotton-white into its dye's color,
/// a kit's wood into its own wood's. A shorter ramp keeps its lightest shade for the rest.
#[derive(Clone, Copy)]
pub struct Tint {
    pub from: (u8, u8),
    pub to: (u8, u8),
}

impl Tint {
    pub fn apply(self, c: u8) -> u8 {
        let ((from, n), (to, m)) = (self.from, self.to);
        if c >= from && c < from + n { to + (c - from).min(m - 1) } else { c }
    }
}

type Rgb = (u8, u8, u8);

#[inline]
fn argb((r, g, b): Rgb) -> u32 {
    let (r, g, b) = ((r & 0xFC) as u32, (g & 0xFC) as u32, (b & 0xFC) as u32);
    0xFF00_0000 | (r << 16) | (g << 8) | b
}

fn ramp(p: &mut Palette, start: u8, n: u8, (a, b): (Rgb, Rgb)) {
    let n = n as i32;
    for i in 0..n {
        let lerp = |x: u8, y: u8| (x as i32 + (y as i32 - x as i32) * i / (n - 1).max(1)) as u8;
        p[(start as i32 + i) as usize] = argb((lerp(a.0, b.0), lerp(a.1, b.1), lerp(a.2, b.2)));
    }
}

/// A province's look: the colors of its land bank, and what its buildings are made of. One
/// style throughout, each province turning a few dials: the light, outlines and patterns are
/// the same everywhere.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Style {
    /// The first province, the green river lowland of cotton and dyers: wooden halls.
    Cloth,
    /// Cool, weathered hills of mines: grey stone, slate-blue water, dark pines.
    Hills,
    /// A warm valley of potters: red clay, brick and terracotta, willows.
    Kiln,
    /// Fresh, wet plains of paddies: bright green, sky water, daub and thatch.
    Plains,
    /// A bleached, hot coast: sand, turquoise water, whitewash.
    Coast,
}

/// What a style's walls are: wooden planks, dry stone, brick, timber and daub, or whitewashed
/// plaster.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Walls {
    Planks,
    Stone,
    Brick,
    Daub,
    Plaster,
}

/// What a style's floors are: planks, flagstones, terracotta, beaten earth or pale tiles.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Floors {
    Planks,
    Flags,
    Terracotta,
    Earth,
    Tiles,
}

impl Style {
    pub const ALL: [Style; 5] = [Style::Cloth, Style::Hills, Style::Kiln, Style::Plains, Style::Coast];

    /// Its number on the wire: its place in `ALL`.
    pub fn index(self) -> u8 {
        Style::ALL.iter().position(|&s| s == self).unwrap_or(0) as u8
    }

    /// The style numbered `i`, or the first province's for a number it doesn't know.
    pub fn from_index(i: u8) -> Style {
        Style::ALL.get(i as usize).copied().unwrap_or(Style::Cloth)
    }

    pub fn name(self) -> &'static str {
        match self {
            Style::Cloth => "cloth",
            Style::Hills => "hills",
            Style::Kiln => "kiln",
            Style::Plains => "plains",
            Style::Coast => "coast",
        }
    }

    pub fn walls(self) -> Walls {
        match self {
            Style::Cloth => Walls::Planks,
            Style::Hills => Walls::Stone,
            Style::Kiln => Walls::Brick,
            Style::Plains => Walls::Daub,
            Style::Coast => Walls::Plaster,
        }
    }

    pub fn floors(self) -> Floors {
        match self {
            Style::Cloth => Floors::Planks,
            Style::Hills => Floors::Flags,
            Style::Kiln => Floors::Terracotta,
            Style::Plains => Floors::Earth,
            Style::Coast => Floors::Tiles,
        }
    }

    /// The color a building's name is painted on its wall in, and its shadow: light on dark
    /// walls, dark on light ones.
    pub fn label(self) -> (u8, u8) {
        match self.walls() {
            Walls::Daub | Walls::Plaster => (INK, WALL + 3),
            _ => (WALL + 3, OUTLINE),
        }
    }

    /// Its land bank: ground, foliage, paths, water, flowers, walls and floors, each a ramp
    /// from its darkest to its lightest. Each stays near the first province's saturation and
    /// brightness, so all five belong to one game.
    fn bank(self) -> [(u8, u8, (Rgb, Rgb)); 7] {
        let ramps = |grass, hedge, path, water, flower, wall, floor| {
            [(GRASS, 4, grass), (HEDGE, 3, hedge), (LAND_PATH, 3, path), (LAND_WATER, 3, water), (LAND_FLOWER, 2, flower), (WALL, 4, wall), (FLOOR, 4, floor)]
        };
        match self {
            Style::Cloth => ramps(
                ((28, 64, 36), (104, 156, 72)),
                ((14, 36, 26), (52, 96, 52)),
                ((96, 72, 48), (168, 136, 96)),
                ((30, 58, 104), (104, 164, 204)),
                ((200, 100, 160), (252, 220, 120)),
                ((56, 34, 24), (180, 128, 78)),
                ((84, 58, 40), (164, 124, 84)),
            ),
            Style::Hills => ramps(
                ((34, 54, 42), (108, 136, 96)),
                ((12, 30, 34), (46, 82, 68)),
                ((78, 74, 68), (150, 142, 128)),
                ((30, 52, 80), (100, 140, 168)),
                ((152, 112, 176), (232, 216, 144)),
                ((52, 52, 60), (150, 148, 152)),
                ((66, 64, 70), (140, 136, 132)),
            ),
            Style::Kiln => ramps(
                ((48, 66, 30), (150, 162, 82)),
                ((24, 46, 22), (96, 128, 60)),
                ((112, 62, 40), (192, 128, 84)),
                ((36, 66, 74), (116, 158, 150)),
                ((224, 124, 76), (252, 216, 120)),
                ((86, 44, 36), (188, 112, 86)),
                ((96, 54, 38), (196, 134, 96)),
            ),
            Style::Plains => ramps(
                ((26, 74, 32), (120, 184, 82)),
                ((14, 46, 26), (62, 120, 56)),
                ((112, 92, 62), (196, 170, 122)),
                ((40, 86, 142), (140, 198, 232)),
                ((232, 132, 172), (252, 238, 222)),
                ((104, 80, 54), (216, 192, 142)),
                ((92, 72, 50), (172, 142, 102)),
            ),
            Style::Coast => ramps(
                ((104, 98, 60), (232, 218, 168)),
                ((28, 58, 40), (98, 136, 74)),
                ((168, 150, 112), (244, 236, 210)),
                ((22, 94, 118), (100, 210, 206)),
                ((240, 112, 92), (252, 232, 144)),
                ((132, 128, 124), (246, 242, 230)),
                ((148, 128, 100), (230, 214, 182)),
            ),
        }
    }
}

/// The palette for a screen of a province in `style`: the fixed 64, and its land bank.
pub fn build(style: Style) -> Palette {
    let mut p = [argb((0, 0, 0)); PALETTE_SIZE];
    for (start, n, ends) in style.bank() {
        ramp(&mut p, start, n, ends);
    }
    // The coast's ground is sand, not grass: light all through, a little dune grass in it.
    if style == Style::Coast {
        for (k, rgb) in [(118, 122, 70), (212, 196, 148), (228, 214, 170), (246, 238, 214)].into_iter().enumerate() {
            p[GRASS as usize + k] = argb(rgb);
        }
    }
    ramp(&mut p, PATH, 3, ((96, 72, 48), (168, 136, 96)));
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
    ramp(&mut p, WATER, 3, ((30, 58, 104), (104, 164, 204)));

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
        for &c in build(Style::Cloth).iter() {
            assert_eq!(c & 0x0003_0303, 0, "channel not quantized to 6 bits: {c:08x}");
        }
    }

    #[test]
    fn shifting_stays_inside_the_ramp() {
        assert_eq!(shift(COTTON + 1, 1), COTTON + 2);
        assert_eq!(shift(COTTON + 2, 1), COTTON + 2);
        assert_eq!(shift(COTTON, -1), COTTON);
        assert_eq!(shift(OUTLINE, 1), OUTLINE);
    }

    #[test]
    fn ramps_do_not_overlap() {
        let ramps: [(u8, u8); 25] = [
            (PATH, 3), (WOOD, 4), (PLANK, 4), (MAT, 3), (COTTON, 3), (THREAD, 3), (LEAF, 3),
            (GOBLIN, 3), (CYCLOPS, 3), (EYE, 1), (TOOTH, 1), (SPARK, 3), (REC, 2), (LOOP, 2), (DUST, 1), (SHADOW, 1),
            (FLOWER, 2), (WATER, 3),
            // The land bank.
            (GRASS, 4), (HEDGE, 3), (LAND_PATH, 3), (LAND_WATER, 3), (LAND_FLOWER, 2), (WALL, 4), (FLOOR, 4),
        ];
        let mut used = [false; PALETTE_SIZE];
        for (start, n) in ramps {
            for i in start..start + n {
                assert!(!used[i as usize], "index {i} assigned twice");
                used[i as usize] = true;
            }
        }
        assert!(!used[0] && !used[OUTLINE as usize]);
        // The land is all in its bank, which every style fills.
        for (start, n) in &ramps[18..] {
            assert!(*start as usize >= 64 && (*start + *n) as usize <= PALETTE_SIZE);
        }
        for style in Style::ALL {
            let p = build(style);
            assert!(ramps[18..].iter().all(|&(start, n)| (start..start + n).all(|i| p[i as usize] != argb((0, 0, 0)))), "{style:?} fills its land bank");
        }
    }
}
