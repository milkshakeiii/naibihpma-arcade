//! The palette and the hand-placed sprites. Every color is a palette index resolved through a
//! fixed table at 6-bit-per-channel VGA DAC precision (`channel & 0xFC`); index 0 is transparent.

/// A sprite of palette indices; 0 is transparent.
pub struct Sprite {
    pub w: i32,
    pub h: i32,
    pub px: Vec<u8>,
}

/// Build a sprite from rows of characters, each mapped to a palette index by `key`
/// (`.` is always transparent).
pub fn sprite(rows: &[&str], key: &[(char, u8)]) -> Sprite {
    let w = rows[0].chars().count() as i32;
    let mut px = Vec::with_capacity(rows.len() * w as usize);
    for r in rows {
        assert_eq!(r.chars().count() as i32, w, "ragged sprite row {r}");
        for c in r.chars() {
            px.push(if c == '.' {
                0
            } else {
                key.iter().find(|k| k.0 == c).map_or(0, |k| k.1)
            });
        }
    }
    Sprite {
        w,
        h: rows.len() as i32,
        px,
    }
}

pub const CLEAR: u8 = 0;
pub const INK: u8 = 1;
pub const GROUND0: u8 = 2;
pub const GROUND1: u8 = 3;
pub const GROUND2: u8 = 4;
pub const WATER0: u8 = 5;
pub const WATER1: u8 = 6;
pub const BLDG0: u8 = 7;
pub const BLDG1: u8 = 8;
pub const BLDG2: u8 = 9;
pub const BED: u8 = 10;
pub const TIE: u8 = 11;
pub const RAIL: u8 = 12;
pub const RAIL_HI: u8 = 13;
pub const RAIL_DIM: u8 = 14;
pub const STOP: u8 = 15;
pub const PLATFORM: u8 = 16;
pub const PLATFORM_HI: u8 = 17;
/// Shape colors: ● lamp yellow, ◆ cyan, ■ rose, each with a darker partner.
pub const SHAPE: [u8; 3] = [18, 20, 22];
pub const SHAPE_DK: [u8; 3] = [19, 21, 23];
pub const ENGINE: u8 = 24;
pub const ENGINE_HI: u8 = 25;
pub const ENGINE_DK: u8 = 26;
pub const CAR: u8 = 27;
pub const CAR_HI: u8 = 28;
pub const CAR_DK: u8 = 29;
pub const WINDOW: u8 = 30;
pub const PANEL: u8 = 31;
pub const PANEL_HI: u8 = 32;
pub const PANEL_LO: u8 = 33;
pub const TEXT: u8 = 34;
pub const TEXT_DIM: u8 = 35;
pub const WARN: u8 = 36;
pub const GOOD: u8 = 37;
pub const LAMP_ON: u8 = 38;
pub const LAMP_OFF: u8 = 39;
pub const HILITE: u8 = 40;
pub const GHOST: u8 = 41;
pub const TERMINUS: u8 = 42;
pub const NIGHT: u8 = 43;

const RGB: [u32; 44] = [
    0x000000, // clear (never shown)
    0x0a0a14, // ink: 1 px hull outlines
    0x162230, 0x1b2836, 0x24323f, // ground ramp, recessed
    0x10213c, 0x1c3a64, // water
    0x262636, 0x34344a, 0x47475e, // buildings
    0x3a2a1e, 0x584230, // rail bed and ties
    0x8a94a2, 0xd0d8e2, 0x48505c, // rail, lit rail, unlit rail
    0xb83434, // stop bar, rear lamp
    0x384658, 0x56667e, // platform
    0xf2c850, 0x9a7426, // ● yellow
    0x5ad2e2, 0x2a7a8e, // ◆ cyan
    0xe274a4, 0x8e3e66, // ■ rose
    0xc4422e, 0xec7050, 0x6e2418, // engine
    0x6e7e96, 0xa0b0c8, 0x3a4256, // cars
    0xf8eaa8, // lit windows, headlamp
    0x101826, 0x2c3852, 0x070b12, // panel
    0xe8ecf4, 0x8a92a6, // text
    0xf29440, 0x74d474, // warn, good
    0xfce274, 0x3a3628, // lamp on, off
    0x94f2c4, // next-tile highlight
    0xc8d8ff, // ghost demonstration
    0xa676e4, // terminus
    0x0c1220, // title night
];

/// The palette as 0xRRGGBB at VGA DAC precision.
pub fn rgb(i: u8) -> u32 {
    RGB.get(i as usize).copied().unwrap_or(0) & 0xFC_FC_FC
}

pub struct Art {
    pub pointer: Sprite,
    pub hand: Sprite,
    pub lamp_on: Sprite,
    pub lamp_off: Sprite,
    pub icons: [Sprite; 7],
}

/// The mouse pointer's hot spot, in sprite pixels.
pub const POINTER_HOT: (i32, i32) = (0, 0);

impl Art {
    pub fn new() -> Art {
        let k = [
            ('1', INK),
            ('2', TEXT),
            ('3', TEXT_DIM),
            ('y', LAMP_ON),
            ('o', LAMP_OFF),
            ('g', PANEL_HI),
        ];
        let pointer = sprite(
            &[
                "1.........",
                "11........",
                "121.......",
                "1221......",
                "12221.....",
                "122221....",
                "1222221...",
                "12222221..",
                "122222221.",
                "1222211111",
                "1221221...",
                "121.1221..",
                "11..1221..",
                "1....1221.",
                ".....1221.",
                "......11..",
            ],
            &k,
        );
        let hand = sprite(
            &[
                "....11......",
                "...1221.....",
                "...1221.....",
                "...1221.....",
                "...1222111..",
                "...12222221.",
                ".11122222221",
                ".12122222221",
                ".12222222221",
                "..1222222221",
                "..1222222221",
                "...122222221",
                "...111111111",
            ],
            &k,
        );
        let lamp = |head: char| {
            sprite(
                &[
                    "..111..", ".1HHH1.", "1HHHHH1", ".1HHH1.", "..111..", "...1...", "..121..",
                    "..121..", "..121..", "..121..", "..121..", "..121..", "..121..", ".11211.",
                    ".12221.", ".11111.",
                ],
                &[
                    ('1', INK),
                    ('2', TEXT_DIM),
                    ('H', if head == 'y' { LAMP_ON } else { LAMP_OFF }),
                ],
            )
        };
        let icon = |rows: &[&str]| sprite(rows, &k);
        let icons = [
            // eraser
            icon(&[
                "............",
                "......1111..",
                ".....122221.",
                "....1222221.",
                "...12222221.",
                "..13322221..",
                ".133332221..",
                ".13333221...",
                "..133321....",
                "...1111.....",
                "............",
                "............",
            ]),
            // express lever
            icon(&[
                "........11..",
                ".......1221.",
                "......1221..",
                ".....1221...",
                "....1221....",
                "...1221.....",
                "..1221......",
                ".11111111...",
                ".12222221...",
                ".11111111...",
                "............",
                "............",
            ]),
            // lift rail behind
            icon(&[
                ".....11.....",
                "....1221....",
                "...122221...",
                "..12222221..",
                ".....22.....",
                ".....22.....",
                "............",
                "111111111111",
                "1.1..1..1..1",
                "111111111111",
                "............",
                "............",
            ]),
            // scanlines
            icon(&[
                "............",
                "222222222222",
                "............",
                "222222222222",
                "............",
                "222222222222",
                "............",
                "222222222222",
                "............",
                "222222222222",
                "............",
                "............",
            ]),
            // sound
            icon(&[
                "............",
                ".....1......",
                "....12...2..",
                "...122....2.",
                "1111222.2.2.",
                "1222222..2.2",
                "1222222..2.2",
                "1111222.2.2.",
                "...122....2.",
                "....12...2..",
                ".....1......",
                "............",
            ]),
            // pause
            icon(&[
                "............",
                "..111..111..",
                "..121..121..",
                "..121..121..",
                "..121..121..",
                "..121..121..",
                "..121..121..",
                "..121..121..",
                "..121..121..",
                "..111..111..",
                "............",
                "............",
            ]),
            // restart
            icon(&[
                "............",
                "....11111...",
                "...1222221..",
                "..121...121.",
                "..121....1..",
                "..121.......",
                "..121...1...",
                "..121..121..",
                "...1222221..",
                "....111121..",
                "........11..",
                "............",
            ]),
        ];
        Art {
            pointer,
            hand,
            lamp_on: lamp('y'),
            lamp_off: lamp('o'),
            icons,
        }
    }
}
