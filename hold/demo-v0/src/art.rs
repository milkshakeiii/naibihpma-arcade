//! The palette and all the art. Every colour is a palette index resolved through a fixed table at
//! 6-bit-per-channel VGA DAC precision (`channel & 0xFC`); index 0 is transparent. A few indices
//! rotate their colours slowly (water glints, glowing spores) for ambience: palette cycling, never
//! a whole-screen flash.
//!
//! Small creatures are drawn by hand as character grids. Buildings, the nest and the map tiles are
//! painted by code with integer shapes and dithering. Every foreground sprite gets a 1 px ink hull.

pub struct Sprite {
    pub w: i32,
    pub h: i32,
    pub px: Vec<u8>,
}

impl Sprite {
    pub fn new(w: i32, h: i32) -> Sprite {
        Sprite {
            w,
            h,
            px: vec![0; (w * h) as usize],
        }
    }

    pub fn get(&self, x: i32, y: i32) -> u8 {
        if x < 0 || y < 0 || x >= self.w || y >= self.h {
            0
        } else {
            self.px[(y * self.w + x) as usize]
        }
    }

    pub fn set(&mut self, x: i32, y: i32, c: u8) {
        if x >= 0 && y >= 0 && x < self.w && y < self.h {
            self.px[(y * self.w + x) as usize] = c;
        }
    }

    pub fn rect(&mut self, x: i32, y: i32, w: i32, h: i32, c: u8) {
        for yy in y..y + h {
            for xx in x..x + w {
                self.set(xx, yy, c);
            }
        }
    }

    /// Filled ellipse with integer arithmetic.
    pub fn ellipse(&mut self, cx: i32, cy: i32, rx: i32, ry: i32, c: u8) {
        for yy in cy - ry..=cy + ry {
            for xx in cx - rx..=cx + rx {
                let (dx, dy) = (xx - cx, yy - cy);
                if dx * dx * ry * ry + dy * dy * rx * rx <= rx * rx * ry * ry {
                    self.set(xx, yy, c);
                }
            }
        }
    }

    /// Recolour every pixel of colour `from` inside the ellipse to `to` on a checkerboard.
    pub fn dither_in(&mut self, cx: i32, cy: i32, rx: i32, ry: i32, from: u8, to: u8) {
        for yy in cy - ry..=cy + ry {
            for xx in cx - rx..=cx + rx {
                let (dx, dy) = (xx - cx, yy - cy);
                if dx * dx * ry * ry + dy * dy * rx * rx <= rx * rx * ry * ry
                    && (xx + yy) & 1 == 0
                    && self.get(xx, yy) == from
                {
                    self.set(xx, yy, to);
                }
            }
        }
    }

    pub fn line(&mut self, x0: i32, y0: i32, x1: i32, y1: i32, c: u8) {
        let (dx, dy) = ((x1 - x0).abs(), -(y1 - y0).abs());
        let (sx, sy) = (if x0 < x1 { 1 } else { -1 }, if y0 < y1 { 1 } else { -1 });
        let (mut x, mut y, mut err) = (x0, y0, dx + dy);
        loop {
            self.set(x, y, c);
            if x == x1 && y == y1 {
                break;
            }
            let e2 = 2 * err;
            if e2 >= dy {
                err += dy;
                x += sx;
            }
            if e2 <= dx {
                err += dx;
                y += sy;
            }
        }
    }

    /// Grow the sprite by one pixel on every side and put an ink hull around it (AGENTS.md 4).
    pub fn outlined(&self) -> Sprite {
        let mut o = Sprite::new(self.w + 2, self.h + 2);
        for y in 0..self.h {
            for x in 0..self.w {
                o.set(x + 1, y + 1, self.get(x, y));
            }
        }
        let src = o.px.clone();
        let (ow, oh) = (o.w, o.h);
        let at = |x: i32, y: i32| {
            if x < 0 || y < 0 || x >= ow || y >= oh {
                0
            } else {
                src[(y * ow + x) as usize]
            }
        };
        for y in 0..o.h {
            for x in 0..o.w {
                if at(x, y) == 0
                    && (at(x - 1, y) != 0
                        || at(x + 1, y) != 0
                        || at(x, y - 1) != 0
                        || at(x, y + 1) != 0)
                    && [at(x - 1, y), at(x + 1, y), at(x, y - 1), at(x, y + 1)]
                        .iter()
                        .any(|&c| c != INK)
                {
                    o.set(x, y, INK);
                }
            }
        }
        o
    }

    /// The sprite mirrored left to right.
    pub fn flipped(&self) -> Sprite {
        let mut f = Sprite::new(self.w, self.h);
        for y in 0..self.h {
            for x in 0..self.w {
                f.set(self.w - 1 - x, y, self.get(x, y));
            }
        }
        f
    }

    /// Every opaque pixel replaced by one colour (a silhouette, used for ghosts and hit marks).
    pub fn tinted(&self, c: u8) -> Sprite {
        let mut t = Sprite::new(self.w, self.h);
        for (i, &p) in self.px.iter().enumerate() {
            if p != 0 {
                t.px[i] = c;
            }
        }
        t
    }
}

/// Build a sprite from rows of characters mapped to palette indices by `key` (`.` is transparent).
pub fn grid(rows: &[&str], key: &[(char, u8)]) -> Sprite {
    let w = rows[0].chars().count() as i32;
    let mut s = Sprite::new(w, rows.len() as i32);
    for (y, r) in rows.iter().enumerate() {
        assert_eq!(r.chars().count() as i32, w, "ragged sprite row {r}");
        for (x, ch) in r.chars().enumerate() {
            if ch != '.' {
                let c = key.iter().find(|k| k.0 == ch).map_or(0, |k| k.1);
                s.set(x as i32, y as i32, c);
            }
        }
    }
    s
}

// ---- the palette -------------------------------------------------------------------------------

pub const CLEAR: u8 = 0;
pub const INK: u8 = 1;
pub const MOSS0: u8 = 2;
pub const MOSS1: u8 = 3;
pub const MOSS2: u8 = 4;
pub const MOSS3: u8 = 5;
pub const PATH0: u8 = 6;
pub const PATH1: u8 = 7;
pub const FERN0: u8 = 8;
pub const FERN1: u8 = 9;
pub const TREE0: u8 = 10;
pub const TREE1: u8 = 11;
pub const TREE2: u8 = 12;
pub const TREE3: u8 = 13;
pub const TRUNK: u8 = 14;
pub const WATER0: u8 = 15;
pub const WATER1: u8 = 16;
pub const WATER2: u8 = 17;
/// Three water-glint indices whose colours rotate.
pub const WGLINT: [u8; 3] = [18, 19, 20];
pub const STAR: u8 = 21;
pub const LOG0: u8 = 22;
pub const LOG1: u8 = 23;
/// Three glowing-spore indices whose colours rotate.
pub const SPORE: [u8; 3] = [24, 25, 26];
pub const WOOD0: u8 = 27;
pub const WOOD1: u8 = 28;
pub const WOOD2: u8 = 29;
pub const CAP0: u8 = 30;
pub const CAP1: u8 = 31;
pub const CAP2: u8 = 32;
pub const CREAM: u8 = 33;
pub const LEAF0: u8 = 34;
pub const LEAF1: u8 = 35;
pub const COPPER0: u8 = 36;
pub const COPPER1: u8 = 37;
pub const COPPER2: u8 = 38;
pub const STONE0: u8 = 39;
pub const STONE1: u8 = 40;
pub const STONE2: u8 = 41;
pub const GLOW: u8 = 42;
pub const DEW: u8 = 43;
pub const DEW_DK: u8 = 44;
pub const NEWT0: u8 = 45;
pub const NEWT1: u8 = 46;
pub const NEWT2: u8 = 47;
pub const TOAD0: u8 = 48;
pub const TOAD1: u8 = 49;
pub const TOAD2: u8 = 50;
pub const ROBE0: u8 = 51;
pub const ROBE1: u8 = 52;
pub const HERON0: u8 = 53;
pub const HERON1: u8 = 54;
pub const HERON2: u8 = 55;
pub const BEAK: u8 = 56;
pub const WISP0: u8 = 57;
pub const WISP1: u8 = 58;
pub const WISP2: u8 = 59;
pub const CURSE0: u8 = 60;
pub const CURSE1: u8 = 61;
pub const TWIG0: u8 = 62;
pub const TWIG1: u8 = 63;
pub const EGG: u8 = 64;
pub const PANEL: u8 = 65;
pub const PANEL_HI: u8 = 66;
pub const PANEL_LO: u8 = 67;
pub const TEXT: u8 = 68;
pub const TEXT_DIM: u8 = 69;
/// The yellow of shortcut letters.
pub const KEY: u8 = 70;
pub const GOOD: u8 = 71;
pub const WARN: u8 = 72;
pub const BAD: u8 = 73;
pub const HILITE: u8 = 74;
pub const BTN: u8 = 75;
pub const BTN_HI: u8 = 76;
pub const BTN_LO: u8 = 77;
pub const BTN_ON: u8 = 78;
pub const FLAG: u8 = 79;
pub const FLAG_DK: u8 = 80;
pub const POLE: u8 = 81;
pub const HP_OK: u8 = 82;
pub const HP_LOW: u8 = 83;
pub const HEART: u8 = 84;
pub const NIGHT: u8 = 85;
pub const SHADE: u8 = 86;
pub const GHOST_OK: u8 = 87;
pub const GHOST_NO: u8 = 88;

const RGB: [u32; 89] = [
    0x000000, // clear (never shown)
    0x080c10, // ink hulls
    0x0e2018, 0x15301f, 0x1c3a26, 0x2a4c32, // moss ramp
    0x33301f, 0x433e29, // path
    0x1c4628, 0x2c6638, // fern
    0x06120c, 0x0c2416, 0x143820, 0x1e4e2a, 0x2a1c14, // trees, trunk
    0x081a2c, 0x0c2844, 0x18466c, // water
    0x18466c, 0x2a6690, 0x4a90b8, // water glints (rotating)
    0xf0ecc4, // star reflections
    0x3a2614, 0x5c3e20, // bridge logs
    0x183020, 0x2a6a50, 0x60e0b0, // spores (rotating)
    0x3a2618, 0x5a3c22, 0x7c5632, // wood
    0x6a1c1c, 0x9a2e26, 0xc85a3c, 0xe8d8b0, // mushroom cap, cream spots and stalk
    0x2c5a24, 0x4a8a34, // leaves
    0x6a3a1c, 0xa8642c, 0xe0a050, // copper
    0x34384a, 0x50566a, 0x7a8094, // stone
    0xf8d070, // warm window glow
    0x7ae8f0, 0x2a8aa0, // dew
    0x8a2a10, 0xd85a1c, 0xf8a040, // newt
    0x3a3a14, 0x6e6e26, 0xa0a044, // toad
    0x4a2a6a, 0x7a4aa0, // robe
    0x3a4a5c, 0x6a7e94, 0xb4c8d8, 0xe0b040, // heron, beak
    0x4a1a6a, 0x9a4ae0, 0xe4b8ff, // wisp
    0x22122a, 0x341a40, // cursed ground
    0x3a2a18, 0x6a5030, 0xd8e0e8, // nest twigs, eggs
    0x0e1620, 0x2a3846, 0x04070a, // panel
    0xe8ecf0, 0x8a96a4, // text
    0xfce050, // shortcut-letter yellow
    0x74d474, 0xf29440, 0xe04a3a, // good, warn, bad
    0x94f2c4, // highlight
    0x1a2834, 0x3a4c5c, 0x080e14, 0x245446, // buttons, pressed/active
    0xf0c840, 0x9a7a20, 0xc8b890, // reward flag, pole
    0x58c058, 0xd84030, 0xe04a5a, // hp bars, heart
    0x060a10, // title night
    0x050a08, // dithered shade
    0x3c8a4c, 0x8a3030, // placement ghost ok / blocked
];

/// The palette at tick `t`, at VGA DAC precision: the glint and spore indices rotate slowly.
pub fn palette(t: u32) -> [u32; 256] {
    let mut p = [0u32; 256];
    for (i, c) in RGB.iter().enumerate() {
        p[i] = c & 0xFC_FC_FC;
    }
    let w = (t / 10) as usize;
    for k in 0..3 {
        p[WGLINT[k] as usize] = RGB[WGLINT[(k + w) % 3] as usize] & 0xFC_FC_FC;
    }
    let s = (t / 15) as usize;
    for k in 0..3 {
        p[SPORE[k] as usize] = RGB[SPORE[(k + s) % 3] as usize] & 0xFC_FC_FC;
    }
    p
}

/// The fixed palette (no cycling).
pub fn rgb(i: u8) -> u32 {
    RGB.get(i as usize).copied().unwrap_or(0) & 0xFC_FC_FC
}

// ---- the art -----------------------------------------------------------------------------------

/// Two walking frames, facing right; flip for left.
pub struct Walk {
    pub right: [Sprite; 2],
    pub left: [Sprite; 2],
}

fn walk(a: Sprite, b: Sprite) -> Walk {
    let (a, b) = (a.outlined(), b.outlined());
    Walk {
        left: [a.flipped(), b.flipped()],
        right: [a, b],
    }
}

pub struct Art {
    pub pointer: Sprite,
    pub newt: Walk,
    pub toad: Walk,
    pub heron: Walk,
    pub heron_fly: Walk,
    pub wisp: [Sprite; 2],
    pub hall: Sprite,
    pub lodge: Sprite,
    pub toad_hall: Sprite,
    pub still: Sprite,
    pub post: Sprite,
    pub nest: Sprite,
    pub totem: Sprite,
    pub rubble: Sprite,
    pub grave: Sprite,
    pub flag: Sprite,
    pub icon_dew: Sprite,
    pub icon_heart: Sprite,
    pub icon_home: Sprite,
    pub icon_no: Sprite,
    pub icon_yes: Sprite,
    pub icon_zz: Sprite,
    pub icon_nest: Sprite,
    pub icon_stone: Sprite,
    pub fern: Sprite,
    pub lily: Sprite,
}

/// The mouse pointer's hot spot, in sprite pixels.
pub const POINTER_HOT: (i32, i32) = (0, 0);

impl Art {
    pub fn new() -> Art {
        Art {
            pointer: pointer(),
            newt: walk(newt(false), newt(true)),
            toad: walk(toad(false), toad(true)),
            heron: walk(heron(false), heron(true)),
            heron_fly: walk(heron_fly(false), heron_fly(true)),
            wisp: [wisp(false).outlined(), wisp(true).outlined()],
            hall: hall().outlined(),
            lodge: lodge().outlined(),
            toad_hall: toad_hall().outlined(),
            still: still().outlined(),
            post: post().outlined(),
            nest: nest().outlined(),
            totem: totem().outlined(),
            rubble: rubble().outlined(),
            grave: grave().outlined(),
            flag: flag().outlined(),
            icon_dew: icon(&["..d..", ".ddd.", "ddDdd", "ddddd", ".ddd."]),
            icon_heart: icon(&[".h.h.", "hhhhh", "hhhhh", ".hhh.", "..h.."]),
            icon_home: icon(&["..w..", ".www.", "wwwww", ".w.w.", ".www."]),
            icon_no: icon(&["n...n", ".n.n.", "..n..", ".n.n.", "n...n"]),
            icon_yes: icon(&["..k..", "..k..", "..k..", ".....", "..k.."]),
            icon_zz: icon(&["zzz..", "..z..", ".z...", "zzzzz", "...zz"]),
            icon_nest: icon(&["e.e.e", "ttttt", "TtTtT", ".TTT.", "....."]),
            icon_stone: icon(&["s...s", "sSSSs", "SSSSS", "SsSsS", "SSSSS"]),
            fern: fern(),
            lily: grid(
                &[".LLLL.", "LLlLLL", "LLLLlL", ".LLL.."],
                &[('L', LEAF1), ('l', LEAF0)],
            ),
        }
    }
}

fn pointer() -> Sprite {
    grid(
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
        &[('1', INK), ('2', TEXT)],
    )
}

fn icon(rows: &[&str]) -> Sprite {
    grid(
        rows,
        &[
            ('d', DEW),
            ('D', TEXT),
            ('h', HEART),
            ('w', CREAM),
            ('n', BAD),
            ('k', KEY),
            ('z', TEXT_DIM),
            ('e', EGG),
            ('t', TWIG1),
            ('T', TWIG0),
            ('s', WISP2),
            ('S', STONE1),
        ],
    )
    .outlined()
}

/// An upright newt scout with a spear, facing right.
fn newt(step: bool) -> Sprite {
    let legs: [&str; 2] = if step {
        ["...r...r..", "...o....o."]
    } else {
        ["....r..r..", "....o..o.."]
    };
    grid(
        &[
            ".........t",
            "....rrr..t",
            "...ryyyrks",
            "...ryyyr.s",
            "....rrr..s",
            "...ryyyrrs",
            "..rryyyr.s",
            ".rr.ryyr.s",
            "rr..rkyr.s",
            "r...rrrr.s",
            legs[0],
            legs[1],
        ],
        &[
            ('r', NEWT1),
            ('y', NEWT2),
            ('o', NEWT0),
            ('k', INK),
            ('s', POLE),
            ('t', STONE2),
        ],
    )
}

/// A squat toad enchanter in a robe and hood, with a glowing staff, facing right.
fn toad(step: bool) -> Sprite {
    let feet = if step { "..t......t.." } else { "...t....t..." };
    grid(
        &[
            "...........d",
            "....hhhh...s",
            "...hHHHHh..s",
            "..tTTTTTTt.s",
            ".tTkTTTkTTts",
            ".tTTTTTTTt.s",
            "..pPPPPPptts",
            ".pPPPPPPPp.s",
            ".pPPPPPPPp.s",
            "..pppppppp..",
            feet,
        ],
        &[
            ('d', DEW),
            ('s', WOOD2),
            ('h', ROBE0),
            ('H', ROBE1),
            ('t', TOAD0),
            ('T', TOAD2),
            ('k', INK),
            ('p', ROBE0),
            ('P', ROBE1),
        ],
    )
}

/// A grey heron standing, facing right.
fn heron(step: bool) -> Sprite {
    let legs: [&str; 3] = if step {
        ["......l..l....", ".....l....l...", "....ll....ll.."]
    } else {
        ["......l..l....", "......l..l....", ".....ll..ll..."]
    };
    grid(
        &[
            ".......hhh....",
            "......hHHHhbbb",
            "......hHkHh...",
            "......hHh.....",
            "......hHh.....",
            "....hhHHh.....",
            "..hhHHHHHh....",
            ".hHHWWWHHHh...",
            "hhHWWWWHHHh...",
            ".hhHHHHHHh....",
            "...hhhhhh.....",
            "......l..l....",
            "......l..l....",
            legs[0],
            legs[1],
            legs[2],
        ],
        &[
            ('h', HERON0),
            ('H', HERON1),
            ('W', HERON2),
            ('b', BEAK),
            ('k', INK),
            ('l', HERON0),
        ],
    )
}

/// A heron in flight, wings up or down: neck tucked, legs trailing, broad wings.
fn heron_fly(up: bool) -> Sprite {
    let rows: [&str; 11] = if up {
        [
            "......WWW...........",
            ".....WWHWW..........",
            "....WWHHW...........",
            "....WHHW.....hhh....",
            "...WHHW.....hHkHbbbb",
            "llhhHHHHHHHHHHHh....",
            ".lhHHHHHHHHHHhh.....",
            "...hhhhhhhhhh.......",
            "....................",
            "....................",
            "....................",
        ]
    } else {
        [
            "....................",
            "....................",
            ".............hhh....",
            "............hHkHbbbb",
            "llhhHHHHHHHHHHHh....",
            ".lhHHHHHHHHHHhh.....",
            "...hhWHHHWhhhh......",
            "....WHHHW...........",
            "....WHHW............",
            ".....WHW............",
            "......W.............",
        ]
    };
    grid(
        &rows,
        &[
            ('h', HERON0),
            ('H', HERON1),
            ('W', HERON2),
            ('b', BEAK),
            ('k', INK),
            ('l', HERON0),
        ],
    )
}

/// A bog wisp: a small cold flame with two eyes.
fn wisp(alt: bool) -> Sprite {
    grid(
        if alt {
            &[
                "...a....", "..aAa...", ".aAWAa..", ".AWWWA..", "aAkWkAa.", ".AWWWA..", "..AAA...",
                "...a.a..",
            ]
        } else {
            &[
                "....a...", "...aAa..", "..aAWAa.", "..AWWWA.", ".aAkWkAa", "..AWWWA.", "...AAA..",
                "..a.a...",
            ]
        },
        &[('a', WISP0), ('A', WISP1), ('W', WISP2), ('k', INK)],
    )
}

fn shadow(s: &mut Sprite, cx: i32, cy: i32, rx: i32) {
    for xx in cx - rx..=cx + rx {
        if (xx + cy) & 1 == 0 {
            s.set(xx, cy, SHADE);
        }
    }
}

/// The Elder's hall: a great mushroom with a lit door and window. 30 x 30.
fn hall() -> Sprite {
    let mut s = Sprite::new(30, 30);
    shadow(&mut s, 15, 29, 11);
    s.rect(10, 15, 10, 14, CREAM);
    s.rect(10, 15, 2, 14, STONE2);
    s.rect(13, 21, 4, 8, WOOD0);
    s.rect(14, 22, 2, 6, GLOW);
    s.set(18, 18, GLOW);
    s.set(19, 18, GLOW);
    s.ellipse(15, 10, 14, 9, CAP1);
    s.ellipse(15, 13, 14, 5, CAP0);
    s.ellipse(15, 9, 14, 8, CAP1);
    s.ellipse(11, 6, 6, 3, CAP2);
    for (x, y, r) in [
        (7, 9, 2),
        (15, 5, 2),
        (22, 8, 2),
        (18, 12, 1),
        (10, 13, 1),
        (25, 12, 1),
    ] {
        s.ellipse(x, y, r, r, CREAM);
    }
    s
}

/// The newt lodge: a bark hut under a leaf roof. 28 x 26.
fn lodge() -> Sprite {
    let mut s = Sprite::new(28, 26);
    shadow(&mut s, 14, 25, 11);
    s.rect(4, 11, 20, 14, WOOD1);
    for x in (5..24).step_by(4) {
        s.rect(x, 11, 1, 14, WOOD0);
    }
    s.rect(11, 16, 6, 9, WOOD0);
    s.rect(12, 17, 4, 8, GLOW);
    s.rect(6, 15, 3, 3, GLOW);
    s.ellipse(14, 7, 13, 7, LEAF0);
    s.ellipse(14, 5, 12, 5, LEAF1);
    s.dither_in(14, 9, 13, 3, LEAF0, TREE1);
    s.set(14, 0, NEWT2);
    s.rect(13, 1, 3, 2, NEWT1);
    s
}

/// The toad hall: a mossy stone dome with an arched door and a purple banner. 30 x 26.
fn toad_hall() -> Sprite {
    let mut s = Sprite::new(30, 26);
    shadow(&mut s, 15, 25, 13);
    s.ellipse(15, 16, 13, 9, STONE1);
    s.rect(2, 16, 27, 9, STONE1);
    s.dither_in(15, 18, 13, 7, STONE1, STONE0);
    s.ellipse(15, 9, 12, 5, FERN1);
    s.ellipse(15, 8, 9, 3, LEAF1);
    s.ellipse(15, 21, 4, 5, WOOD0);
    s.rect(11, 21, 9, 4, WOOD0);
    s.ellipse(15, 21, 2, 3, GLOW);
    s.rect(13, 21, 5, 4, GLOW);
    s.rect(4, 13, 3, 8, ROBE1);
    s.rect(4, 20, 1, 1, ROBE0);
    s.rect(6, 20, 1, 1, ROBE0);
    s.rect(24, 6, 2, 6, CREAM);
    s.ellipse(25, 5, 3, 2, CAP1);
    s
}

/// The dew still: a copper pot on a stand with a coil and a drop. 24 x 26.
fn still() -> Sprite {
    let mut s = Sprite::new(24, 26);
    shadow(&mut s, 12, 25, 9);
    s.rect(4, 20, 2, 5, WOOD1);
    s.rect(17, 20, 2, 5, WOOD1);
    s.rect(3, 19, 18, 2, WOOD2);
    s.ellipse(11, 13, 8, 7, COPPER1);
    s.dither_in(11, 15, 8, 5, COPPER1, COPPER0);
    s.ellipse(8, 10, 3, 2, COPPER2);
    s.rect(10, 3, 3, 4, COPPER1);
    s.line(12, 3, 20, 8, COPPER2);
    s.line(20, 8, 20, 15, COPPER2);
    s.rect(18, 16, 5, 3, WOOD2);
    s.set(20, 1, DEW);
    s.rect(19, 2, 3, 2, DEW);
    s.set(20, 4, DEW_DK);
    s
}

/// A reed watch post with a lantern on top. 16 x 30.
fn post() -> Sprite {
    let mut s = Sprite::new(16, 30);
    shadow(&mut s, 8, 29, 6);
    for x in [3, 7, 12] {
        s.rect(x, 8, 2, 21, WOOD1);
    }
    s.line(3, 26, 13, 12, WOOD0);
    s.line(13, 26, 3, 12, WOOD0);
    s.rect(1, 7, 14, 3, WOOD2);
    s.rect(1, 2, 2, 5, WOOD1);
    s.rect(13, 2, 2, 5, WOOD1);
    s.rect(0, 0, 16, 2, LEAF0);
    s.rect(6, 3, 4, 4, GLOW);
    s
}

/// A heron's nest of twigs on a dead stump, with eggs. 30 x 22.
fn nest() -> Sprite {
    let mut s = Sprite::new(30, 22);
    shadow(&mut s, 15, 21, 11);
    s.rect(11, 12, 8, 9, TRUNK);
    s.rect(12, 12, 2, 9, WOOD1);
    s.ellipse(15, 9, 14, 6, TWIG0);
    let mut h = 7u32;
    for y in 3..16 {
        for x in 1..29 {
            h = h.wrapping_mul(1103515245).wrapping_add(12345);
            if s.get(x, y) == TWIG0 && (h >> 16) % 3 == 0 {
                s.set(x, y, TWIG1);
            }
        }
    }
    s.ellipse(15, 7, 9, 3, WOOD0);
    for (x, y) in [(11, 6), (15, 5), (19, 6)] {
        s.ellipse(x, y, 2, 2, EGG);
    }
    s
}

/// Gorf's croaking stone: a standing stone carved as a great frog's head, runes aglow. 22 x 30.
fn totem() -> Sprite {
    let mut s = Sprite::new(22, 30);
    shadow(&mut s, 11, 29, 9);
    s.ellipse(11, 27, 9, 2, LEAF0);
    s.rect(4, 9, 14, 18, STONE0);
    s.rect(4, 9, 3, 18, STONE1);
    s.ellipse(11, 9, 8, 6, STONE1);
    s.dither_in(11, 9, 8, 6, STONE1, STONE0);
    // Bulging eyes on top, glowing.
    s.ellipse(6, 4, 3, 3, STONE1);
    s.ellipse(16, 4, 3, 3, STONE1);
    s.ellipse(6, 4, 2, 2, WISP2);
    s.ellipse(16, 4, 2, 2, WISP2);
    s.set(6, 4, INK);
    s.set(16, 4, INK);
    // A wide frog mouth and glowing runes down the front.
    s.line(5, 12, 17, 12, INK);
    s.set(4, 11, INK);
    s.set(18, 11, INK);
    for (y, c) in [(16, WISP1), (19, WISP0), (22, WISP1)] {
        s.rect(9, y, 4, 1, c);
        s.set(8, y + 1, c);
        s.set(13, y + 1, c);
    }
    s
}

fn rubble() -> Sprite {
    let mut s = Sprite::new(24, 12);
    shadow(&mut s, 12, 11, 10);
    for (x, y, r, c) in [
        (5, 7, 3, STONE1),
        (12, 6, 4, WOOD0),
        (18, 8, 3, STONE0),
        (9, 9, 2, WOOD1),
        (15, 4, 2, STONE2),
    ] {
        s.ellipse(x, y, r, r - 1, c);
    }
    s
}

fn grave() -> Sprite {
    grid(
        &[
            "..ggg..", ".gGGGg.", ".gGkGg.", ".gGGGg.", ".gGGGg.", "lllllll",
        ],
        &[('g', STONE0), ('G', STONE1), ('k', INK), ('l', LEAF0)],
    )
}

fn flag() -> Sprite {
    grid(
        &[
            "pFFFFF", "pFFFFf", "pFFFff", "pFFFFf", "p.....", "p.....", "p.....", "p.....",
        ],
        &[('p', POLE), ('F', FLAG), ('f', FLAG_DK)],
    )
}

fn fern() -> Sprite {
    grid(
        &[
            "...f.....",
            "f..f..f..",
            ".f.F.f...",
            "..fFf..f.",
            "f..F..f..",
            ".ffFff...",
            "...F.....",
        ],
        &[('f', FERN1), ('F', FERN0)],
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn palette_is_dac_exact() {
        let p = palette(37);
        for c in p.iter() {
            assert_eq!(c & 0x03_03_03, 0);
        }
    }

    #[test]
    fn walkers_stand_on_their_bottom_row() {
        let a = Art::new();
        for s in [&a.newt.right[0], &a.toad.right[0], &a.heron.right[0]] {
            let last = s.h - 1;
            assert!(
                (0..s.w).any(|x| s.get(x, last) != 0),
                "feet touch the bottom row"
            );
        }
    }
}
