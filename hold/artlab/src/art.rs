//! The palette and all the art. Every colour is a palette index resolved through a fixed table at
//! 6-bit-per-channel VGA DAC precision (`channel & 0xFC`); index 0 is transparent. A few indices
//! rotate their colours slowly (water glints, glowing spores, fireflies) for ambience: palette
//! cycling, never a whole-screen flash.
//!
//! The palette has two halves. *Background* ramps (terrain tiles and decorations) are remapped by
//! `palette(t, sky)` for dusk, night, rain and dry spells. *Foreground* colours (creatures,
//! buildings, spots, items, UI) never change, so at night the Hold's folk and windows stand out
//! against a dark marsh. A third group *glows*: spores, stars, glints, ley runes, the curse's
//! sparks and fireflies keep their light whatever the sky, so the marsh shines at night.
//!
//! Small creatures are drawn by hand as character grids. Buildings, spots and the map tiles are
//! painted by code with integer shapes and dithering. Every foreground sprite gets a 1 px ink hull;
//! decorations get a hull of the background ink so they darken with the land.

use std::collections::HashMap;

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
        self.outlined_with(INK)
    }

    /// As `outlined`, with a hull of colour `ink` (decorations use the background ink `BGINK`).
    pub fn outlined_with(&self, ink: u8) -> Sprite {
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
                        .any(|&c| c != ink)
                {
                    o.set(x, y, ink);
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

    /// The sprite mirrored top to bottom (a creature on its back).
    pub fn flipped_v(&self) -> Sprite {
        let mut f = Sprite::new(self.w, self.h);
        for y in 0..self.h {
            for x in 0..self.w {
                f.set(x, self.h - 1 - y, self.get(x, y));
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

    /// The sprite with some colours swapped for others (an older or poisoned look).
    pub fn recolored(&self, map: &[(u8, u8)]) -> Sprite {
        let mut t = Sprite::new(self.w, self.h);
        for (i, &p) in self.px.iter().enumerate() {
            t.px[i] = map.iter().find(|m| m.0 == p).map_or(p, |m| m.1);
        }
        t
    }

    /// Draw `s` onto this sprite with its top-left at (x, y); transparent pixels are skipped.
    pub fn blit(&mut self, s: &Sprite, x: i32, y: i32) {
        for sy in 0..s.h {
            for sx in 0..s.w {
                let c = s.get(sx, sy);
                if c != 0 {
                    self.set(x + sx, y + sy, c);
                }
            }
        }
    }

    /// Recolour pixels of `from` to `to` inside a rectangle on a checkerboard (2 x 2 dither).
    pub fn dither_rect(&mut self, x: i32, y: i32, w: i32, h: i32, from: u8, to: u8) {
        for yy in y..y + h {
            for xx in x..x + w {
                if (xx + yy) & 1 == 0 && self.get(xx, yy) == from {
                    self.set(xx, yy, to);
                }
            }
        }
    }

    /// Recolour every pixel of `from` to `to` inside a rectangle.
    pub fn swap_rect(&mut self, x: i32, y: i32, w: i32, h: i32, from: u8, to: u8) {
        for yy in y..y + h {
            for xx in x..x + w {
                if self.get(xx, yy) == from {
                    self.set(xx, yy, to);
                }
            }
        }
    }

    /// Set a pixel only where something is already drawn (texture inside a shape).
    pub fn over(&mut self, x: i32, y: i32, c: u8) {
        if self.get(x, y) != 0 {
            self.set(x, y, c);
        }
    }

    /// True when no pixel is drawn.
    pub fn is_empty(&self) -> bool {
        self.px.iter().all(|&p| p == 0)
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

// Background ramps added for v1 (remapped by the sky).
/// The dark hull drawn around decorations; it darkens with the land.
pub const BGINK: u8 = 89;
pub const MOSS4: u8 = 90;
pub const DEEP0: u8 = 91;
pub const DEEP1: u8 = 92;
pub const DEEP2: u8 = 93;
pub const MUD0: u8 = 94;
pub const MUD1: u8 = 95;
pub const MUD2: u8 = 96;
/// The wet sheen on mud.
pub const MUD3: u8 = 97;
pub const SAND0: u8 = 98;
pub const SAND1: u8 = 99;
pub const SAND2: u8 = 100;
pub const PATH2: u8 = 101;
pub const PATH3: u8 = 102;
pub const ROCK0: u8 = 103;
pub const ROCK1: u8 = 104;
pub const ROCK2: u8 = 105;
pub const ROCK3: u8 = 106;
pub const REED0: u8 = 107;
pub const REED1: u8 = 108;
pub const REED2: u8 = 109;
pub const CATTAIL0: u8 = 110;
pub const CATTAIL1: u8 = 111;
pub const CURSE2: u8 = 112;
pub const CURSE3: u8 = 113;
pub const BARK0: u8 = 114;
pub const BARK1: u8 = 115;
pub const BARK2: u8 = 116;
pub const BLEAF0: u8 = 117;
pub const BLEAF1: u8 = 118;
pub const BLEAF2: u8 = 119;
pub const BPINK: u8 = 120;
pub const BCAP0: u8 = 121;
pub const BCAP1: u8 = 122;
pub const BCREAM: u8 = 123;

// Glowing background indices: never darkened by the sky.
pub const LEY0: u8 = 124;
pub const LEY1: u8 = 125;
pub const CURSE_GLOW: u8 = 126;
/// Three firefly indices: by day they match the canopy (TREE1); at dusk and night they blink.
pub const FIREFLY: [u8; 3] = [127, 128, 129];

// Foreground colours added for v1 (never remapped).
pub const EYE: u8 = 130;
pub const CAEC0: u8 = 131;
pub const CAEC1: u8 = 132;
pub const CAEC2: u8 = 133;
pub const TFROG0: u8 = 134;
pub const TFROG1: u8 = 135;
pub const TFROG2: u8 = 136;
pub const TPAD: u8 = 137;
pub const SAL0: u8 = 138;
pub const SAL1: u8 = 139;
pub const SALY0: u8 = 140;
pub const SALY1: u8 = 141;
pub const CTOAD0: u8 = 142;
pub const CTOAD1: u8 = 143;
pub const CTOAD2: u8 = 144;
pub const SAC0: u8 = 145;
pub const SAC1: u8 = 146;
pub const BTOAD0: u8 = 147;
pub const BTOAD1: u8 = 148;
pub const BTOAD2: u8 = 149;
pub const WART: u8 = 150;
pub const AXO0: u8 = 151;
pub const AXO1: u8 = 152;
pub const AXO2: u8 = 153;
pub const GILL0: u8 = 154;
pub const GILL1: u8 = 155;
pub const BULL0: u8 = 156;
pub const BULL1: u8 = 157;
pub const BULL2: u8 = 158;
pub const BELLY: u8 = 159;
pub const GOL0: u8 = 160;
pub const GOL1: u8 = 161;
pub const GOL2: u8 = 162;
pub const OLM0: u8 = 163;
pub const OLM1: u8 = 164;
pub const OLM2: u8 = 165;
pub const OLD_NEWT1: u8 = 166;
pub const OLD_NEWT2: u8 = 167;
pub const OLD_CAEC1: u8 = 168;
pub const OLD_CAEC2: u8 = 169;
pub const OLD_TFROG1: u8 = 170;
pub const OLD_TFROG2: u8 = 171;
pub const OLD_SALY0: u8 = 172;
pub const OLD_SALY1: u8 = 173;
pub const OLD_CTOAD1: u8 = 174;
pub const OLD_CTOAD2: u8 = 175;
pub const OLD_BTOAD1: u8 = 176;
pub const OLD_BTOAD2: u8 = 177;
pub const OLD_AXO1: u8 = 178;
pub const OLD_AXO2: u8 = 179;
pub const OLD_BULL1: u8 = 180;
pub const OLD_BULL2: u8 = 181;
pub const OLD_GOL1: u8 = 182;
pub const OLD_GOL2: u8 = 183;
pub const OLD_OLM1: u8 = 184;
pub const OLD_OLM2: u8 = 185;
pub const OLD_TOAD1: u8 = 186;
pub const OLD_TOAD2: u8 = 187;
pub const OLD_ROBE0: u8 = 188;
pub const OLD_ROBE1: u8 = 189;
/// White brows and whiskers of the old.
pub const OLD_WHITE: u8 = 190;
pub const SNAKE0: u8 = 191;
pub const SNAKE1: u8 = 192;
pub const SNAKE2: u8 = 193;
pub const SNAKE_Y: u8 = 194;
pub const CROW0: u8 = 195;
pub const CROW1: u8 = 196;
pub const CROW2: u8 = 197;
pub const TURT0: u8 = 198;
pub const TURT1: u8 = 199;
pub const TURT2: u8 = 200;
pub const TSKIN0: u8 = 201;
pub const TSKIN1: u8 = 202;
pub const GORF0: u8 = 203;
pub const GORF1: u8 = 204;
pub const GORF2: u8 = 205;
pub const GROBE0: u8 = 206;
pub const GROBE1: u8 = 207;
/// Foreground water (the pools of buildings, which keep their colours at night).
pub const POOL0: u8 = 208;
pub const POOL1: u8 = 209;
pub const POOL2: u8 = 210;
pub const EARTH0: u8 = 211;
pub const EARTH1: u8 = 212;
pub const EARTH2: u8 = 213;
pub const EMBER0: u8 = 214;
pub const EMBER1: u8 = 215;
pub const EMBER2: u8 = 216;
pub const IRON0: u8 = 217;
pub const IRON1: u8 = 218;
pub const IRON2: u8 = 219;
pub const MOREL0: u8 = 220;
pub const MOREL1: u8 = 221;
pub const BREW0: u8 = 222;
pub const BREW1: u8 = 223;
pub const METEOR0: u8 = 224;
pub const METEOR1: u8 = 225;
pub const VENOM0: u8 = 226;
pub const VENOM1: u8 = 227;
pub const PEARL: u8 = 228;
pub const HUSK0: u8 = 229;
pub const HUSK1: u8 = 230;
pub const LEAF2: u8 = 231;

const RGB: [u32; 232] = [
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
    // ---- v1 background ramps (remapped by the sky) ----
    0x06100c, // background ink
    0x3a6a3c, // moss tips
    0x040c18, 0x061628, 0x0c2640, // deep water
    0x120a06, 0x22150c, 0x342314, 0x3e4a50, // mud and its wet sheen
    0x7a6c48, 0xa49468, 0xc8b888, // sand
    0x5c5436, 0x786e4a, // dusty path
    0x1a1e2a, 0x2c3242, 0x444c5e, 0x646e82, // rock
    0x1e3818, 0x3e6428, 0x6e9040, // reeds
    0x3a2210, 0x6a4220, // cattails
    0x0a0410, 0x4a2262, // curse cracks and veins
    0x1e140c, 0x3a2818, 0x5a4028, // bark
    0x1a4422, 0x2a6430, 0x4a8a40, // lily and leaf
    0xd890b0, // lily flower
    0x5a1818, 0x922c24, 0xc8b890, // wild mushroom
    // ---- v1 glows (never darkened) ----
    0x2a6a9a, 0x90e0fc, // ley runes
    0xa050e8, // curse sparks
    0x0c2416, 0x0c2416, 0x0c2416, // fireflies (dark by day)
    // ---- v1 foreground ----
    0xf0c040, // eyes
    0x3a3048, 0x625474, 0x928aa4, // caecilian
    0x1c6a24, 0x3cb83a, 0x96ec5a, 0xf8b060, // tree frog, toe pads
    0x1a1a24, 0x363846, 0xc48c10, 0xfcd828, // fire salamander
    0x4a4418, 0x8a8030, 0xc0b050, 0xd8a050, 0xf8e0a0, // Chorus Toad, throat sac
    0x40280e, 0x744c26, 0xa07440, 0xc89c5c, // toad, warts
    0xa04c6c, 0xe08ca4, 0xfcccd8, 0xb82448, 0xf86488, // axolotl, gills
    0x1c481c, 0x3a7c2c, 0x6cb040, 0xe0e0a0, // bullfrog, belly
    0x343618, 0x5c602c, 0x8c9048, // goliath frog
    0xb08c98, 0xe8c8cc, 0xfcf0f0, // olm
    0xa86448, 0xd09c78, // old newt
    0x6e6878, 0xa29eaa, // old caecilian
    0x5a9056, 0xa8c898, // old tree frog
    0xa08c58, 0xd8cc98, // old salamander spots
    0x8a8464, 0xb8b490, // old Chorus Toad
    0x7a6650, 0xa8947c, // old toad
    0xc8a4b0, 0xece0e4, // old axolotl
    0x5a7a4c, 0x90a878, // old bullfrog
    0x6a6a50, 0x9a9a7c, // old goliath
    0xd0c4c4, 0xf0ecec, // old olm
    0x7a7a58, 0xa8a888, // old Daot
    0x4a3e5a, 0x6e6084, // old robe
    0xdce0e4, // old white
    0x18220e, 0x34442a, 0x5a6c3c, 0xe0d060, // snake, stripe
    0x16162a, 0x2a2a48, 0x52528a, // crow
    0x2a2810, 0x4e4a20, 0x7e7634, 0x384230, 0x687258, // snapping turtle, skin
    0x1a2610, 0x34461e, 0x5e7230, 0x1c0c28, 0x3a1c50, // Gorf, his robe
    0x061424, 0x0c2a44, 0x1c4c70, // pool water
    0x2a1a0e, 0x4a3220, 0x6e5034, // earth
    0x8a1a08, 0xe05810, 0xfcb430, // embers
    0x3c4454, 0x707c90, 0xbcc8d8, // iron
    0x5c3c1c, 0x9c7444, // morel
    0x2a7a2a, 0x8ae860, // brew
    0x6a1a5c, 0xd84aa0, // meteor
    0x4a7a1a, 0xa8e040, // venom
    0xf4e8f4, // pearl
    0x9a9478, 0xd8d4b8, // husk
    0x7ab84a, // bright leaf
];

/// What the sky is doing; the palette remaps only the *background* ramps for it.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub struct Sky {
    /// 0 day, 1 dusk or dawn, 2 night, 3 deep night.
    pub dark: u8,
    pub rain: bool,
    pub dry: bool,
}

/// How the sky treats a palette index.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Layer {
    /// Terrain and decorations: remapped for dusk, night, rain and dry spells.
    Background,
    /// Light in the marsh (spores, stars, glints, runes, curse sparks): never darkened.
    Glow,
    /// Blinks at dusk and night, matches the canopy by day.
    Firefly,
    /// Creatures, buildings, spots, items and UI: never remapped.
    Foreground,
}

/// Which layer a palette index belongs to.
pub fn layer(i: u8) -> Layer {
    match i {
        2..=17 | 22 | 23 | 60 | 61 | 89..=123 => Layer::Background,
        18..=21 | 24..=26 | 124..=126 => Layer::Glow,
        127..=129 => Layer::Firefly,
        _ => Layer::Foreground,
    }
}

fn split(c: u32) -> (i32, i32, i32) {
    (
        (c >> 16 & 255) as i32,
        (c >> 8 & 255) as i32,
        (c & 255) as i32,
    )
}

fn join(r: i32, g: i32, b: i32) -> u32 {
    let k = |v: i32| v.clamp(0, 255) as u32;
    ((k(r) << 16) | (k(g) << 8) | k(b)) & 0xFC_FC_FC
}

/// A background colour under the sky: parched for dry spells, grey for rain, then darkened and
/// cooled for dusk and night. Integer arithmetic only, like a DOS palette fade.
fn sky_tint(c: u32, sky: Sky) -> u32 {
    let (mut r, mut g, mut b) = split(c);
    if sky.dry {
        // Greens fade toward straw and water turns murky.
        let gr = g - r;
        r += gr / 3 + 10;
        g = g * 9 / 10 + 4;
        b = b * 7 / 10;
    }
    if sky.rain {
        let l = (r * 3 + g * 6 + b) / 10;
        r = (r * 5 + l * 3) / 8 * 82 / 100;
        g = (g * 5 + l * 3) / 8 * 86 / 100;
        b = (b * 5 + l * 3) / 8 * 92 / 100 + 6;
    }
    // In the dark, bright ground is held down more than dark ground, so pale sand reads as
    // moonlit rather than glowing.
    if sky.dark >= 2 {
        let l = (r * 3 + g * 6 + b) / 10;
        let k = 256 - l * 3 / 5;
        r = r * k / 256;
        g = g * k / 256;
        b = b * k / 256;
    }
    match sky.dark {
        0 => {}
        1 => {
            r = r * 80 / 100 + 6;
            g = g * 62 / 100 + 2;
            b = b * 76 / 100 + 12;
        }
        2 => {
            r = r * 36 / 100 + 2;
            g = g * 48 / 100 + 4;
            b = b * 62 / 100 + 16;
        }
        _ => {
            r = r * 26 / 100 + 1;
            g = g * 36 / 100 + 3;
            b = b * 50 / 100 + 12;
        }
    }
    join(r, g, b)
}

/// The 256-entry palette at tick `t` under `sky`, every entry at 6-bit DAC precision.
/// The glint, spore and firefly indices cycle slowly; only background ramps follow the sky.
pub fn palette(t: u32, sky: Sky) -> [u32; 256] {
    let mut p = [0u32; 256];
    for (i, c) in RGB.iter().enumerate() {
        p[i] = match layer(i as u8) {
            Layer::Background => sky_tint(*c, sky),
            _ => c & 0xFC_FC_FC,
        };
    }
    let w = (t / 10) as usize;
    for k in 0..3 {
        p[WGLINT[k] as usize] = RGB[WGLINT[(k + w) % 3] as usize] & 0xFC_FC_FC;
    }
    let s = (t / 15) as usize;
    for k in 0..3 {
        p[SPORE[k] as usize] = RGB[SPORE[(k + s) % 3] as usize] & 0xFC_FC_FC;
    }
    // Fireflies: dark as the canopy by day; at dusk and night each blinks in its own slow turn.
    let host = sky_tint(RGB[TREE1 as usize], sky);
    let ph = (t / 6) % 16;
    for k in 0..3u32 {
        let on = (ph + k * 5) % 16;
        p[FIREFLY[k as usize] as usize] = match (sky.dark, on) {
            (0, _) => host,
            (1, 0..=2) => 0x90_b0_40,
            (_, 0) | (_, 2) => 0x90_c0_48,
            (_, 1) => 0xd8_f8_60,
            _ => host,
        };
    }
    p
}

/// The fixed daylight palette (no cycling).
pub fn rgb(i: u8) -> u32 {
    RGB.get(i as usize).copied().unwrap_or(0) & 0xFC_FC_FC
}

// ---- what the game asks for --------------------------------------------------------------------

#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash)]
pub enum Look {
    Newt,
    Caecilian,
    TreeFrog,
    Salamander,
    ChorusToad,
    Toad,
    Axolotl,
    Bullfrog,
    Goliath,
    Olm,
    Daot,
    Heron,
    Snake,
    Crow,
    Wisp,
    Turtle,
    Gorf,
}

impl Look {
    pub const ALL: [Look; 17] = [
        Look::Newt,
        Look::Caecilian,
        Look::TreeFrog,
        Look::Salamander,
        Look::ChorusToad,
        Look::Toad,
        Look::Axolotl,
        Look::Bullfrog,
        Look::Goliath,
        Look::Olm,
        Look::Daot,
        Look::Heron,
        Look::Snake,
        Look::Crow,
        Look::Wisp,
        Look::Turtle,
        Look::Gorf,
    ];

    /// The marsh's enemies (drawn as adults only).
    pub fn is_enemy(self) -> bool {
        matches!(
            self,
            Look::Heron | Look::Snake | Look::Crow | Look::Wisp | Look::Turtle | Look::Gorf
        )
    }

    /// Creatures that fly or float (their Fly poses are real flight frames).
    pub fn flies(self) -> bool {
        matches!(self, Look::Heron | Look::Crow | Look::Wisp)
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash)]
pub enum Age {
    Young,
    Adult,
    Old,
}

impl Age {
    pub const ALL: [Age; 3] = [Age::Young, Age::Adult, Age::Old];
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash)]
pub enum Pose {
    Walk0,
    Walk1,
    Work0,
    Work1,
    Strike,
    Fly0,
    Fly1,
    DugIn,
    Dream,
}

impl Pose {
    pub const ALL: [Pose; 9] = [
        Pose::Walk0,
        Pose::Walk1,
        Pose::Work0,
        Pose::Work1,
        Pose::Strike,
        Pose::Fly0,
        Pose::Fly1,
        Pose::DugIn,
        Pose::Dream,
    ];
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash)]
pub enum BLook {
    Hall,
    Burrow,
    TwigLoft,
    MorelCave,
    FairyRing,
    ChorusLog,
    EmberLog,
    IroncapCave,
    ToadHall,
    ShellForge,
    Cauldron,
    StarPond,
    NewtHouse,
    SpawningPool,
    DaotsGrotto,
    OlmsCave,
    Nest,
    LoneNest,
    GorfStone,
    LeechPool,
    CrowRoost,
    GorfHollow,
}

impl BLook {
    pub const ALL: [BLook; 22] = [
        BLook::Hall,
        BLook::Burrow,
        BLook::TwigLoft,
        BLook::MorelCave,
        BLook::FairyRing,
        BLook::ChorusLog,
        BLook::EmberLog,
        BLook::IroncapCave,
        BLook::ToadHall,
        BLook::ShellForge,
        BLook::Cauldron,
        BLook::StarPond,
        BLook::NewtHouse,
        BLook::SpawningPool,
        BLook::DaotsGrotto,
        BLook::OlmsCave,
        BLook::Nest,
        BLook::LoneNest,
        BLook::GorfStone,
        BLook::LeechPool,
        BLook::CrowRoost,
        BLook::GorfHollow,
    ];
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash)]
pub enum SpotLook {
    Fern,
    RedFungus,
    WhiteFungus,
    HybridFungus,
    Puffball,
}

impl SpotLook {
    pub const ALL: [SpotLook; 5] = [
        SpotLook::Fern,
        SpotLook::RedFungus,
        SpotLook::WhiteFungus,
        SpotLook::HybridFungus,
        SpotLook::Puffball,
    ];
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash)]
pub enum TileLook {
    Water,
    Deep,
    Moss,
    Fern,
    Mud,
    Path,
    Sand,
    Rock,
    Reeds,
    Cursed,
    Puddle,
}

impl TileLook {
    pub const ALL: [TileLook; 11] = [
        TileLook::Water,
        TileLook::Deep,
        TileLook::Moss,
        TileLook::Fern,
        TileLook::Mud,
        TileLook::Path,
        TileLook::Sand,
        TileLook::Rock,
        TileLook::Reeds,
        TileLook::Cursed,
        TileLook::Puddle,
    ];
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash)]
pub enum Deco {
    BigTree,
    Tree,
    Stump,
    Stone,
    LeyStone,
    Lily,
    ReedClump,
    Mushroom,
    Log,
}

impl Deco {
    pub const ALL: [Deco; 9] = [
        Deco::BigTree,
        Deco::Tree,
        Deco::Stump,
        Deco::Stone,
        Deco::LeyStone,
        Deco::Lily,
        Deco::ReedClump,
        Deco::Mushroom,
        Deco::Log,
    ];
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash)]
pub enum ItemLook {
    Dew,
    Fungiwood,
    CapIngot,
    SpottedCap,
    GlowSpores,
    SunOrb,
    ClargentOrb,
    MeteorOrb,
    WoodenShovel,
    IronShovel,
    VenomwoodShovel,
    Slingshot,
    Catapult,
    IronSpear,
    VenomwoodSpear,
    VenomironSpear,
    Chanfron,
    Peytral,
    Croupiere,
    VenomironChanfron,
    DewBead,
    GlowCharm,
    StarPearl,
}

impl ItemLook {
    pub const ALL: [ItemLook; 23] = [
        ItemLook::Dew,
        ItemLook::Fungiwood,
        ItemLook::CapIngot,
        ItemLook::SpottedCap,
        ItemLook::GlowSpores,
        ItemLook::SunOrb,
        ItemLook::ClargentOrb,
        ItemLook::MeteorOrb,
        ItemLook::WoodenShovel,
        ItemLook::IronShovel,
        ItemLook::VenomwoodShovel,
        ItemLook::Slingshot,
        ItemLook::Catapult,
        ItemLook::IronSpear,
        ItemLook::VenomwoodSpear,
        ItemLook::VenomironSpear,
        ItemLook::Chanfron,
        ItemLook::Peytral,
        ItemLook::Croupiere,
        ItemLook::VenomironChanfron,
        ItemLook::DewBead,
        ItemLook::GlowCharm,
        ItemLook::StarPearl,
    ];
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash)]
pub enum CondLook {
    Dry,
    Poisoned,
    Blinded,
    Bolstered,
    Regenerating,
    Altered,
    Dreaming,
    Leeched,
    Frightened,
}

impl CondLook {
    pub const ALL: [CondLook; 9] = [
        CondLook::Dry,
        CondLook::Poisoned,
        CondLook::Blinded,
        CondLook::Bolstered,
        CondLook::Regenerating,
        CondLook::Altered,
        CondLook::Dreaming,
        CondLook::Leeched,
        CondLook::Frightened,
    ];
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash)]
pub enum UiLook {
    Pause,
    Play,
    Fast,
    Magic,
    Screens,
    Attack,
    Explore,
    Guard,
    Fetch,
    RainCall,
    OldSong,
    Starlight,
    LeyLight,
    DeepCalling,
}

impl UiLook {
    pub const ALL: [UiLook; 14] = [
        UiLook::Pause,
        UiLook::Play,
        UiLook::Fast,
        UiLook::Magic,
        UiLook::Screens,
        UiLook::Attack,
        UiLook::Explore,
        UiLook::Guard,
        UiLook::Fetch,
        UiLook::RainCall,
        UiLook::OldSong,
        UiLook::Starlight,
        UiLook::LeyLight,
        UiLook::DeepCalling,
    ];
}

/// Footprint in map squares (16 px each), width x height.
pub fn footprint(b: BLook) -> (i32, i32) {
    use BLook::*;
    match b {
        Hall | GorfHollow => (4, 3),
        ChorusLog | ToadHall | ShellForge | DaotsGrotto | OlmsCave => (3, 2),
        StarPond | SpawningPool => (3, 3),
        GorfStone => (1, 2),
        Burrow | TwigLoft | MorelCave | FairyRing | EmberLog | IroncapCave | Cauldron
        | NewtHouse | Nest | LoneNest | LeechPool | CrowRoost => (2, 2),
    }
}

/// The map square, in pixels.
const TILE: i32 = 16;
/// How many variants each terrain tile has (any variant number is taken modulo this). Pick the
/// variant by a hash of the square's position, so the rarer details (spores, a toadstool, a
/// firefly) are scattered rather than lined up.
pub const TILE_VARIANTS: u8 = 8;

// ---- the art store -----------------------------------------------------------------------------

pub struct Art {
    pointer: Sprite,
    creatures: Vec<Sprite>,
    buildings: Vec<Sprite>,
    sites: HashMap<(i32, i32), Sprite>,
    ruins: HashMap<(i32, i32), Sprite>,
    spots: Vec<Sprite>,
    tiles: Vec<Sprite>,
    decos: Vec<Sprite>,
    items: Vec<Sprite>,
    conds: Vec<Sprite>,
    uis: Vec<Sprite>,
    husk: Sprite,
    eggs: Vec<Sprite>,
    flag: Sprite,
}

/// The mouse pointer's hot spot, in sprite pixels.
pub const POINTER_HOT: (i32, i32) = (0, 0);

fn creature_index(look: Look, age: Age, pose: Pose, left: bool) -> usize {
    ((look as usize * 3 + age as usize) * 9 + pose as usize) * 2 + left as usize
}

impl Default for Art {
    fn default() -> Self {
        Art::new()
    }
}

impl Art {
    pub fn new() -> Art {
        let mut creatures = Vec::with_capacity(17 * 3 * 9 * 2);
        for look in Look::ALL {
            for age in Age::ALL {
                for pose in Pose::ALL {
                    let s = creature(look, age, pose);
                    let l = s.flipped();
                    creatures.push(s);
                    creatures.push(l);
                }
            }
        }
        let mut sites = HashMap::new();
        let mut ruins = HashMap::new();
        for w in 1..=4 {
            for h in 1..=4 {
                sites.insert((w, h), site(w, h));
                ruins.insert((w, h), ruin(w, h));
            }
        }
        let mut spots = vec![];
        for s in SpotLook::ALL {
            for stage in 0..4 {
                spots.push(spot(s, stage));
            }
        }
        let mut tiles = vec![];
        for t in TileLook::ALL {
            for v in 0..TILE_VARIANTS {
                tiles.push(tile(t, v));
            }
        }
        Art {
            pointer: pointer(),
            creatures,
            buildings: BLook::ALL.iter().map(|&b| building(b)).collect(),
            sites,
            ruins,
            spots,
            tiles,
            decos: Deco::ALL.iter().map(|&d| deco(d)).collect(),
            items: ItemLook::ALL.iter().map(|&i| item(i)).collect(),
            conds: CondLook::ALL.iter().map(|&c| cond(c)).collect(),
            uis: UiLook::ALL.iter().map(|&u| ui(u)).collect(),
            husk: husk(),
            eggs: (0..3).map(eggs).collect(),
            flag: flag().outlined(),
        }
    }

    /// The mouse pointer (hot spot POINTER_HOT).
    pub fn pointer(&self) -> &Sprite {
        &self.pointer
    }

    /// A creature facing right, or left when `left`. Grounded poses stand on the bottom row.
    pub fn creature(&self, look: Look, age: Age, pose: Pose, left: bool) -> &Sprite {
        &self.creatures[creature_index(look, age, pose, left)]
    }

    /// A finished building: exactly its footprint wide, bottom-aligned, rising above it.
    pub fn building(&self, b: BLook) -> &Sprite {
        &self.buildings[b as usize]
    }

    /// A building site of that footprint (squares): dug earth and stakes.
    pub fn site(&self, w: i32, h: i32) -> &Sprite {
        &self.sites[&(w.clamp(1, 4), h.clamp(1, 4))]
    }

    /// Rubble of that footprint (squares).
    pub fn ruin(&self, w: i32, h: i32) -> &Sprite {
        &self.ruins[&(w.clamp(1, 4), h.clamp(1, 4))]
    }

    /// A growing spot, 16 x 16: stage 0 bare, 1 sprouting, 2 ripening, 3 ripe.
    pub fn spot(&self, s: SpotLook, stage: u8) -> &Sprite {
        &self.spots[s as usize * 4 + stage.min(3) as usize]
    }

    /// A terrain tile, exactly 16 x 16; any variant number is fine.
    pub fn tile(&self, t: TileLook, variant: u8) -> &Sprite {
        &self.tiles[t as usize * TILE_VARIANTS as usize + (variant % TILE_VARIANTS) as usize]
    }

    /// A decoration (background colours, so it follows the sky).
    pub fn deco(&self, d: Deco) -> &Sprite {
        &self.decos[d as usize]
    }

    /// An item icon: 7 x 7 inside its hull (9 x 9).
    pub fn item(&self, i: ItemLook) -> &Sprite {
        &self.items[i as usize]
    }

    /// A condition icon: 5 x 5 inside its hull (7 x 7).
    pub fn cond(&self, c: CondLook) -> &Sprite {
        &self.conds[c as usize]
    }

    /// A UI icon, exactly 11 x 11 (hull included), for 13 x 13 buttons.
    pub fn ui(&self, u: UiLook) -> &Sprite {
        &self.uis[u as usize]
    }

    /// A fallen amphibian's empty skin.
    pub fn husk(&self) -> &Sprite {
        &self.husk
    }

    /// Spawn in a pool: 0 eggs, 1 tadpole, 2 froglet.
    pub fn eggs(&self, stage: u8) -> &Sprite {
        &self.eggs[stage.min(2) as usize]
    }

    /// A posted reward's flag.
    pub fn flag(&self) -> &Sprite {
        &self.flag
    }
}

/// A small integer hash for texture noise.
fn hash(x: i32, y: i32, seed: u32) -> u32 {
    let mut h = (x as u32).wrapping_mul(374_761_393)
        ^ (y as u32).wrapping_mul(668_265_263)
        ^ seed.wrapping_mul(2_246_822_519);
    h = (h ^ (h >> 13)).wrapping_mul(1_274_126_177);
    h ^ (h >> 16)
}

// ==== SECTIONS ====

// ---- creatures ---------------------------------------------------------------------------------

/// One creature frame, outlined, facing right.
fn creature(look: Look, age: Age, pose: Pose) -> Sprite {
    let pose = match pose {
        Pose::Fly0 if !look.flies() => Pose::Walk0,
        Pose::Fly1 if !look.flies() => Pose::Walk1,
        p => p,
    };
    if look.is_enemy() {
        return enemy(look, pose).outlined();
    }
    match pose {
        Pose::DugIn => dug_in(look, age).outlined(),
        Pose::Dream => dream(look, age).outlined(),
        _ => amphibian(look, age, pose).outlined(),
    }
}

/// The rows of a hand-drawn frame and the key that colours them for an age.
fn amphibian_rows(look: Look, young: bool, pose: Pose) -> &'static [&'static str] {
    match look {
        Look::Newt => newt_rows(young, pose),
        Look::Caecilian => caecilian_rows(young, pose),
        Look::TreeFrog => tree_frog_rows(young, pose),
        Look::Salamander => salamander_rows(young, pose),
        Look::ChorusToad => chorus_toad_rows(young, pose),
        Look::Toad => toad_rows(young, pose),
        Look::Axolotl => axolotl_rows(young, pose),
        Look::Bullfrog => bullfrog_rows(young, pose),
        Look::Goliath => goliath_rows(young, pose),
        Look::Olm => olm_rows(young, pose),
        _ => daot_rows(young, pose),
    }
}

/// Colour keys. Every amphibian uses `d m l` for its dark, mid and light skin, `e` for the eye's
/// iris, `k` for ink, `B` for a brow that turns white with age, plus its own extras.
fn amphibian_key(look: Look, age: Age) -> Vec<(char, u8)> {
    let old = age == Age::Old;
    let o = |a: u8, b: u8| if old { b } else { a };
    let mut k = vec![('k', INK), ('e', EYE), ('w', TEXT), ('t', GILL1)];
    let skin: [(char, u8); 4] = match look {
        Look::Newt => [
            ('d', NEWT0),
            ('m', o(NEWT1, OLD_NEWT1)),
            ('l', o(NEWT2, OLD_NEWT2)),
            ('B', o(NEWT1, OLD_WHITE)),
        ],
        Look::Caecilian => [
            ('d', CAEC0),
            ('m', o(CAEC1, OLD_CAEC1)),
            ('l', o(CAEC2, OLD_CAEC2)),
            ('B', o(CAEC1, OLD_WHITE)),
        ],
        Look::TreeFrog => [
            ('d', TFROG0),
            ('m', o(TFROG1, OLD_TFROG1)),
            ('l', o(TFROG2, OLD_TFROG2)),
            ('B', o(TFROG1, OLD_WHITE)),
        ],
        Look::Salamander => [
            ('d', SAL0),
            ('m', SAL1),
            ('l', o(SALY1, OLD_SALY1)),
            ('B', o(SAL1, OLD_WHITE)),
        ],
        Look::ChorusToad => [
            ('d', CTOAD0),
            ('m', o(CTOAD1, OLD_CTOAD1)),
            ('l', o(CTOAD2, OLD_CTOAD2)),
            ('B', o(CTOAD1, OLD_WHITE)),
        ],
        Look::Toad => [
            ('d', BTOAD0),
            ('m', o(BTOAD1, OLD_BTOAD1)),
            ('l', o(BTOAD2, OLD_BTOAD2)),
            ('B', o(BTOAD1, OLD_WHITE)),
        ],
        Look::Axolotl => [
            ('d', AXO0),
            ('m', o(AXO1, OLD_AXO1)),
            ('l', o(AXO2, OLD_AXO2)),
            ('B', o(AXO1, OLD_WHITE)),
        ],
        Look::Bullfrog => [
            ('d', BULL0),
            ('m', o(BULL1, OLD_BULL1)),
            ('l', o(BULL2, OLD_BULL2)),
            ('B', o(BULL1, OLD_WHITE)),
        ],
        Look::Goliath => [
            ('d', GOL0),
            ('m', o(GOL1, OLD_GOL1)),
            ('l', o(GOL2, OLD_GOL2)),
            ('B', o(GOL1, OLD_WHITE)),
        ],
        Look::Olm => [
            ('d', OLM0),
            ('m', o(OLM1, OLD_OLM1)),
            ('l', o(OLM2, OLD_OLM2)),
            ('B', o(OLM1, OLD_WHITE)),
        ],
        _ => [
            ('d', TOAD0),
            ('m', o(TOAD1, OLD_TOAD1)),
            ('l', o(TOAD2, OLD_TOAD2)),
            ('B', o(TOAD2, OLD_WHITE)),
        ],
    };
    k.extend_from_slice(&skin);
    match look {
        Look::Salamander => k.push(('y', o(SALY0, OLD_SALY0))),
        Look::TreeFrog => {
            k.push(('p', TPAD));
            k.push(('c', CREAM));
            k.push(('h', STONE1));
            k.push(('s', WOOD2));
        }
        Look::ChorusToad => {
            k.push(('S', SAC0));
            k.push(('s', SAC1));
            k.push(('h', IRON1));
            k.push(('p', WOOD1));
        }
        Look::Toad => {
            k.push(('W', o(WART, OLD_BTOAD2)));
            k.push(('b', o(WART, OLD_BTOAD2)));
            k.push(('p', POLE));
            k.push(('x', IRON2));
        }
        Look::Axolotl => {
            k.push(('g', GILL0));
            k.push(('G', o(GILL1, AXO1)));
            k.push(('z', KEY));
        }
        Look::Bullfrog => {
            k.push(('b', o(BELLY, HUSK1)));
            k.push(('T', BULL0));
        }
        Look::Goliath => {
            k.push(('b', o(GOL2, OLD_GOL2)));
            k.push(('s', STONE1));
            k.push(('S', STONE2));
        }
        Look::Olm => k.push(('g', GILL0)),
        Look::Daot => {
            k.push(('r', o(ROBE0, OLD_ROBE0)));
            k.push(('R', o(ROBE1, OLD_ROBE1)));
            k.push(('s', WOOD2));
            k.push(('o', STAR));
            k.push(('D', DEW));
            k.push(('b', o(ROBE1, OLD_WHITE)));
            k.push(('y', KEY));
        }
        _ => {}
    }
    k
}

fn amphibian(look: Look, age: Age, pose: Pose) -> Sprite {
    grid(
        amphibian_rows(look, age == Age::Young, pose),
        &amphibian_key(look, age),
    )
}

/// Lying on its back, eyes shut, feet in the air: the walking frame turned belly-up.
fn dream(look: Look, age: Age) -> Sprite {
    let young = age == Age::Young;
    if look == Look::Toad {
        return grid(toad_dream_rows(young), &amphibian_key(look, age));
    }
    let rows = amphibian_rows(look, young, Pose::Walk0);
    let shut: Vec<String> = rows
        .iter()
        .rev()
        .map(|r| {
            r.chars()
                .map(|c| match c {
                    'e' | 'k' => 'd',
                    'p' | 'x' | 's' | 'o' | 'D' | 'y' | 'z' if look == Look::Daot => '.',
                    c => c,
                })
                .collect()
        })
        .collect();
    let refs: Vec<&str> = shut.iter().map(|s| s.as_str()).collect();
    let mut s = grid(&refs, &amphibian_key(look, age));
    // Trim empty rows top and bottom so it lies on the ground.
    trim_rows(&mut s);
    s
}

fn trim_rows(s: &mut Sprite) {
    let row_empty = |s: &Sprite, y: i32| (0..s.w).all(|x| s.get(x, y) == 0);
    let mut top = 0;
    while top < s.h - 1 && row_empty(s, top) {
        top += 1;
    }
    let mut bot = s.h - 1;
    while bot > top && row_empty(s, bot) {
        bot -= 1;
    }
    let mut t = Sprite::new(s.w, bot - top + 1);
    for y in top..=bot {
        for x in 0..s.w {
            t.set(x, y - top, s.get(x, y));
        }
    }
    *s = t;
}

/// Dug in: a low mound of earth with the creature's eyes peeping over the top.
fn dug_in(look: Look, age: Age) -> Sprite {
    let young = age == Age::Young;
    let w = match look {
        Look::Bullfrog => 22,
        Look::Goliath | Look::Olm => 16,
        Look::Caecilian | Look::Newt | Look::Salamander | Look::Axolotl => 11,
        _ => 12,
    } - if young { 4 } else { 0 };
    let key = amphibian_key(look, age);
    let skin = key.iter().find(|k| k.0 == 'm').map_or(TOAD1, |k| k.1);
    let dark = key.iter().find(|k| k.0 == 'd').map_or(TOAD0, |k| k.1);
    let h = if young { 5 } else { 6 };
    let mut s = Sprite::new(w, h + 2);
    let cx = w / 2;
    s.ellipse(cx, h + 2, cx, h, EARTH1);
    s.dither_in(cx, h + 2, cx, h - 1, EARTH1, EARTH0);
    s.ellipse(cx - 1, h - 1, cx - 3, 2, EARTH1);
    for x in 0..w {
        for y in 0..h + 2 {
            if s.get(x, y) == EARTH1 && hash(x, y, 9) % 7 == 0 {
                s.set(x, y, EARTH2);
            }
        }
    }
    // The eyes: two bumps of skin, each with an iris and a pupil.
    let ex = cx + 1;
    let blind = look == Look::Olm || look == Look::Caecilian;
    for (dx, near) in [(-2, false), (1, true)] {
        let x = ex + dx;
        s.rect(x, 1, 2, 2, if near { skin } else { dark });
        if !blind {
            s.set(x + 1, 1, if near { EYE } else { dark });
            s.set(x + 1, 2, INK);
        }
    }
    s.set(ex - 1, 3, EARTH2);
    s.set(ex + 3, 3, EARTH2);
    if look == Look::Toad {
        // The spear stands in the ground beside the mound.
        s = {
            let mut t = Sprite::new(w + 2, h + 2);
            t.blit(&s, 0, 0);
            for y in 1..h + 2 {
                t.set(w + 1, y, POLE);
            }
            t.set(w + 1, 0, IRON2);
            t
        };
    }
    s
}

fn newt_rows(young: bool, pose: Pose) -> &'static [&'static str] {
    match (young, pose) {
        (false, Pose::Walk1) => &[
            "........mmB.",
            "...ddmmdmmkm",
            "ddmmmmlmmmmm",
            "..ddmllllll.",
            "....mm...mm.",
            "...mm...mm..",
        ],
        (false, Pose::Work0) => &[
            "............",
            "...ddmmdmmB.",
            "ddmmmmlmmmkm",
            "..ddmllllllm",
            "....mm...m.m",
            "...mm...mm..",
        ],
        (false, Pose::Work1) => &[
            ".........mmB",
            "...ddmmdmmkm",
            "ddmmmmlmmmm.",
            "..ddmllllll.",
            "....mm..m..m",
            "...mm...m...",
        ],
        (false, Pose::Strike) => &[
            ".........mmB",
            "....ddmdmmkm",
            "dddmmmmlmm..",
            "...ddmllllkm",
            "..m..m..m...",
            ".m....mm....",
        ],
        (false, _) => &[
            "........mmB.",
            "...ddmmdmmkm",
            "ddmmmmlmmmmm",
            "..ddmllllll.",
            "...m.m..m.m.",
            "..m...mm...m",
        ],
        (true, Pose::Walk1) => &["......mB.", "..ddmmmkm", "ddmllllm.", "...m...m."],
        (true, Pose::Work0) => &[".........", "..ddmmmB.", "ddmllllkm", "...m.m..m"],
        (true, Pose::Work1) => &[".......mB", "..ddmmmkm", "ddmllll..", "...m.m..m"],
        (true, Pose::Strike) => &[".......mB", "..ddmmmkm", "ddmllll..", "..m.m..mm"],
        (true, _) => &["......mB.", "..ddmmmkm", "ddmllllm.", "..m..m.m."],
    }
}

fn caecilian_rows(young: bool, pose: Pose) -> &'static [&'static str] {
    match (young, pose) {
        (false, Pose::Walk1) => &[
            "..............",
            "mdm......dmmB.",
            "lllmdmdmdllmkm",
            "...llllll..ll.",
        ],
        (false, Pose::Work0) => &[
            "..............",
            "..........dmB.",
            "mdmdmdmdmdmmkm",
            "llllllllll.lll",
        ],
        (false, Pose::Work1) => &[
            "...........mB.",
            "..........mmkm",
            "mdmdmdmdmdml..",
            "llllllllll....",
        ],
        (false, Pose::Strike) => &[
            "..........mmB.",
            ".........dmmkm",
            "mdmdmdmdmll...",
            "lllllllll..lmm",
        ],
        (false, _) => &[
            "......dmd.....",
            "...mdmllldmmB.",
            "dmdlll...llmkm",
            "lll........ll.",
        ],
        (true, Pose::Walk1) => &["......mB..", "mdmdmd.mkm", ".......ll."],
        (true, Pose::Work0) => &["..........", "dmdmdmdmB.", ".......lkm"],
        (true, Pose::Work1) => &["........B.", "......dmkm", "dmdmdml..."],
        (true, Pose::Strike) => &[".......mB.", "dmdmdmmk..", "........mm"],
        (true, _) => &["...mdm.mB.", "dmd...dmkm", ".......ll."],
    }
}

fn tree_frog_rows(young: bool, pose: Pose) -> &'static [&'static str] {
    match (young, pose) {
        (false, Pose::Walk1) => &[
            "......dBd.",
            ".....dmekm",
            "...ddmmmmm",
            "..dmmlmmkk",
            ".dmmmmmcc.",
            "dmmlmmmcm.",
            ".dmd.mlp..",
            "..pp..pp..",
        ],
        (false, Pose::Work0) => &[
            "......dBhh",
            ".....dmehh",
            "...ddmmmms",
            "..dmmlmmks",
            ".dmmmmmcms",
            "dmmlmmmcm.",
            "dmd..mlp..",
            "ppp..pp...",
        ],
        (false, Pose::Work1) => &[
            "......dBd.",
            ".....dmekm",
            "...ddmmmmm",
            "..dmmlmmkk",
            ".dmmmmmcc.",
            "dmmlmmmcmm",
            "dmd..mlpss",
            "ppp..pp.hh",
        ],
        (false, Pose::Strike) => &[
            "......dBd.",
            ".....dmekm",
            "...ddmmmmm",
            "..dmmlmmtt",
            ".dmmmmmcc.",
            "dmmlmmmcm.",
            "dmd..mlp..",
            "ppp..pp...",
        ],
        (false, _) => &[
            "......dBd.",
            ".....dmekm",
            "...ddmmmmm",
            "..dmmlmmkk",
            ".dmmmmmcc.",
            "dmmlmmmcm.",
            "dmd..mlp..",
            "ppp..pp...",
        ],
        (true, Pose::Walk1) => &[
            "....dB.", "...dmek", ".dmmmkk", "dmmmcc.", ".dmlpm.", "..pp.p.",
        ],
        (true, Pose::Work0) => &[
            "....dBh", "...dmek", ".dmmmks", "dmmmccs", "dmlpm..", "pp.pp..",
        ],
        (true, Pose::Work1) => &[
            "....dB.", "...dmek", ".dmmmkk", "dmmmccm", "dmlpm.s", "pp.pp.h",
        ],
        (true, Pose::Strike) => &[
            "....dB.", "...dmek", ".dmmmtt", "dmmmcc.", "dmlpm..", "pp.pp..",
        ],
        (true, _) => &[
            "....dB.", "...dmek", ".dmmmkk", "dmmmcc.", "dmlpm..", "pp.pp..",
        ],
    }
}

fn salamander_rows(young: bool, pose: Pose) -> &'static [&'static str] {
    match (young, pose) {
        (false, Pose::Walk1) => &[
            "........dmll.",
            "...dmllmmmlkm",
            "ddmmmmmmllmmm",
            "..dmllmmmmmm.",
            "....mm...mm..",
            "...mm...mm...",
        ],
        (false, Pose::Work0) => &[
            ".............",
            "...dmllmmdmB.",
            "ddmmmmmmllmkm",
            "..dmllmmmmmmm",
            "....mm...m.m.",
            "...mm...mm...",
        ],
        (false, Pose::Work1) => &[
            ".........dmll",
            "...dmllmmmmkm",
            "ddmmmmmmllmm.",
            "..dmllmmmmm..",
            "....mm..m..m.",
            "...mm...m....",
        ],
        (false, Pose::Strike) => &[
            ".........dmll",
            "....dmllmmmkm",
            "dddmmmmmmll..",
            "...dmllmmmmkm",
            "..m..m..m....",
            ".m....mm.....",
        ],
        (false, _) => &[
            "........dmll.",
            "...dmllmmmlkm",
            "ddmmmmmmllmmm",
            "..dmllmmmmmm.",
            "...m.m..m.m..",
            "..m...mm...m.",
        ],
        (true, Pose::Walk1) => &["......ml.", "..dmlmmkm", "ddmmmllm.", "...m...m."],
        (true, Pose::Work0) => &[".........", "..dmlmmB.", "ddmmmllkm", "...m.m..m"],
        (true, Pose::Work1) => &[".......ml", "..dmlmmkm", "ddmmmll..", "...m.m..m"],
        (true, Pose::Strike) => &[".......ml", "..dmlmmkm", "ddmmmll..", "..m.m..mm"],
        (true, _) => &["......ml.", "..dmlmmkm", "ddmmmllm.", "..m..m.m."],
    }
}

fn chorus_toad_rows(young: bool, pose: Pose) -> &'static [&'static str] {
    match (young, pose) {
        (false, Pose::Walk1) => &[
            "........dBd..",
            "......ddmekd.",
            "....ddmmmmmmd",
            "...dmmlmmmmmd",
            "..dmmmmmmmkkk",
            ".dmmlmmmmSsss",
            "dmmmmmmmSssss",
            "dmmmlmmmSssss",
            "dmmmmmmmmSSS.",
            ".dmmmd.lm.lm.",
            "..lll..ll.ll.",
        ],
        (false, Pose::Work0) => &[
            ".............",
            "........dBd..",
            "......ddmekdh",
            "....ddmmmmmdh",
            "...dmmlmmmmkp",
            "..dmmmmmmmSsp",
            ".dmmlmmmmSssp",
            "dmmmmmmmSsssp",
            "dmmmlmmmmSSmp",
            ".lmmmd.lm.lmp",
            "llll...ll.llp",
        ],
        (false, Pose::Work1) => &[
            ".........hhh.",
            "........dBpd.",
            "......ddmepd.",
            "....ddmmmmpmd",
            "...dmmlmmmmmd",
            "..dmmmmmmmkkk",
            ".dmmlmmmmSsss",
            "dmmmmmmmSssss",
            "dmmmlmmmmSSS.",
            ".lmmmd.lm.lm.",
            "llll...ll.ll.",
        ],
        (false, Pose::Strike) => &[
            "........dBd..",
            "......ddmekd.",
            "....ddmmmmmmd",
            "...dmmlmmmmm.",
            "..dmmmmmmmSSS",
            ".dmmlmmmmSsss",
            "dmmmmmmmSssss",
            "dmmmlmmmSssss",
            "dmmmmmmmSssss",
            ".lmmmd.lmSSS.",
            "llll...ll.ll.",
        ],
        (false, _) => &[
            "........dBd..",
            "......ddmekd.",
            "....ddmmmmmmd",
            "...dmmlmmmmmd",
            "..dmmmmmmmkkk",
            ".dmmlmmmmSsss",
            "dmmmmmmmSssss",
            "dmmmlmmmSssss",
            "dmmmmmmmmSSS.",
            ".lmmmd.lm.lm.",
            "llll...ll.ll.",
        ],
        (true, Pose::Walk1) => &[
            ".....dBd.",
            "....dmekd",
            "..ddmmmmm",
            ".dmmlmkkk",
            "dmmmmSss.",
            "dmmlmSss.",
            ".dmmd.ll.",
            "..ll.ll..",
        ],
        (true, Pose::Work0) => &[
            ".........",
            ".....dBd.",
            "....dmekh",
            "..ddmmmkp",
            ".dmmlmSsp",
            "dmmmmSssp",
            "dmmlmmSmp",
            "lll.ll.lp",
        ],
        (true, Pose::Work1) => &[
            "......hhh",
            ".....dBp.",
            "....dmepd",
            "..ddmmmmm",
            ".dmmlmkkk",
            "dmmmmSss.",
            "dmmlmmSS.",
            "lll.ll.l.",
        ],
        (true, Pose::Strike) => &[
            ".....dBd.",
            "....dmekd",
            "..ddmmmm.",
            ".dmmlmSSS",
            "dmmmmSsss",
            "dmmlmSsss",
            ".dmmd.SS.",
            "lll.ll.l.",
        ],
        (true, _) => &[
            ".....dBd.",
            "....dmekd",
            "..ddmmmmm",
            ".dmmlmkkk",
            "dmmmmSss.",
            "dmmlmSss.",
            ".dmmd.ll.",
            "lll.ll.l.",
        ],
    }
}

fn toad_rows(young: bool, pose: Pose) -> &'static [&'static str] {
    match (young, pose) {
        (false, Pose::Walk1) => &[
            "............x",
            ".......dBd..x",
            "....ddmmekd.p",
            "..dmmWmmmmmdp",
            ".dmWmmmmWmkkp",
            "dmmmmmmmllllp",
            "dmmmmWmmbbbbp",
            "dmmWmmmbbbbmp",
            ".dmmd..lm.lmp",
            "..lll.ll..llp",
        ],
        (false, Pose::Work0) => &[
            ".............",
            "........dBd..",
            ".....ddmmekd.",
            "...dmmWmmmmmd",
            "..dmWmmmmWmkk",
            ".dmmmmmmmllll",
            "dmmmmWmmbbbbm",
            "dmmWmmmbbbbmp",
            ".dmmd..lm.lmp",
            "lll...ll..llx",
        ],
        (false, Pose::Work1) => &[
            "...........x.",
            ".......dBd.p.",
            "....ddmmekdp.",
            "..dmmWmmmmmpd",
            ".dmWmmmmWmkpk",
            "dmmmmmmmlllpl",
            "dmmmmWmmbbbpm",
            "dmmWmmmbbbbmp",
            ".dmmd..lm.lmp",
            "lll...ll..llp",
        ],
        (false, Pose::Strike) => &[
            ".............",
            ".............",
            ".....dBd.....",
            "...ddmmekd...",
            ".dmmWmmmmmd..",
            "dmWmmmmWmkkk.",
            "dmmmmmmllllpx",
            "dmmmWmmbbbbpp",
            ".dmmmmbbbbm..",
            "llll..lm..lm.",
        ],
        (false, _) => &[
            "............x",
            ".......dBd..x",
            "....ddmmekd.p",
            "..dmmWmmmmmdp",
            ".dmWmmmmWmkkp",
            "dmmmmmmmllllp",
            "dmmmmWmmbbbbp",
            "dmmWmmmbbbbmp",
            ".dmmd..lm.lmp",
            "lll...ll..llp",
        ],
        (true, Pose::Walk1) => &[
            ".........x",
            ".....dBd.p",
            "...dmmekdp",
            ".dmWmmmkkp",
            "dmmmmmlllp",
            "dmmWmbbbbp",
            ".dmd.l..lp",
            "..ll.ll.lp",
        ],
        (true, Pose::Work0) => &[
            "..........",
            ".....dBd..",
            "...dmmekd.",
            ".dmWmmmkk.",
            "dmmmmmlllp",
            "dmmWmbbbbp",
            ".dmd.l..lp",
            "lll.ll.llx",
        ],
        (true, Pose::Work1) => &[
            "........x.",
            ".....dBdp.",
            "...dmmekp.",
            ".dmWmmmkpk",
            "dmmmmmllpl",
            "dmmWmbbbbp",
            ".dmd.l..lp",
            "lll.ll.llp",
        ],
        (true, Pose::Strike) => &[
            "..........",
            "....dBd...",
            "..dmmekd..",
            "dmWmmmkkk.",
            "dmmmmlllpx",
            "dmmWbbbbpp",
            ".dmd.l.l..",
            "lll.ll.ll.",
        ],
        (true, _) => &[
            ".........x",
            ".....dBd.p",
            "...dmmekdp",
            ".dmWmmmkkp",
            "dmmmmmlllp",
            "dmmWmbbbbp",
            ".dmd.l..lp",
            "lll.ll.llp",
        ],
    }
}

fn toad_dream_rows(young: bool) -> &'static [&'static str] {
    if young {
        &[
            ".lm...ml..",
            "..dm.md...",
            ".dbbbbbbd.",
            "dmbbbbbbmd",
            "dmWmmmmkBd",
            ".ddmmmmdd.",
            "ppppppppx.",
        ]
    } else {
        &[
            ".lm.....ml...",
            "..dm...md....",
            "..dmbbbbmd...",
            ".dbbbbbbbbd..",
            "dmbbbbbbbbbmd",
            "dmmWmmmmmkkBd",
            ".dmmmmWmmmmd.",
            "..ddmmmmmdd..",
            "ppppppppppppx",
        ]
    }
}

fn axolotl_rows(young: bool, pose: Pose) -> &'static [&'static str] {
    match (young, pose) {
        (false, Pose::Walk1) => &[
            "......G.G.G..",
            ".....GgGgGg..",
            "lll...gmmmmm.",
            "lmmlmmmmmmBkm",
            ".lmmmmmmmmmmm",
            "..dmmllllmmkk",
            "....mm...mm..",
            "...mm...mm...",
        ],
        (false, Pose::Work0) => &[
            ".............",
            "......G.G.G..",
            "lll..GgGgGg..",
            "lmmlmmgmmmmB.",
            ".lmmmmmmmmmkm",
            "..dmmllllmmmm",
            "....mm...m.kk",
            "...mm...mm...",
        ],
        (false, Pose::Work1) => &[
            ".......G.G.G.",
            "......GgGgGg.",
            "lll....gmmmmm",
            "lmmlmmmmmmmBk",
            ".lmmmmmmmmmmm",
            "..dmmllllmmkk",
            "....mm..m..m.",
            "...mm...m....",
        ],
        (false, Pose::Strike) => &[
            "......G.G.Gz.",
            ".....GgGgGgwz",
            "lll...gmmmm.z",
            "lmmlmmmmmmBkm",
            ".lmmmmmmmmmmm",
            "..dmmllllmmkk",
            "..m..m..m....",
            ".m....mm.....",
        ],
        (false, _) => &[
            "......G.G.G..",
            ".....GgGgGg..",
            "lll...gmmmmm.",
            "lmmlmmmmmmBkm",
            ".lmmmmmmmmmmm",
            "..dmmllllmmkk",
            "...m.m..m.m..",
            "..m...mm...m.",
        ],
        (true, Pose::Walk1) => &[
            "....G.G..",
            "...GgGg..",
            "ll..gmmm.",
            "lmmmmmBkm",
            ".dmllmmkk",
            "...m...m.",
        ],
        (true, Pose::Work0) => &[
            ".........",
            "....G.G..",
            "ll.GgGgm.",
            "lmmmmmmB.",
            ".dmllmmkm",
            "...m.m.kk",
        ],
        (true, Pose::Work1) => &[
            ".....G.G.",
            "....GgGg.",
            "ll...gmmm",
            "lmmmmmmBk",
            ".dmllmmmk",
            "...m.m..m",
        ],
        (true, Pose::Strike) => &[
            "....G.Gz.",
            "...GgGgwz",
            "ll..gmm.z",
            "lmmmmmBkm",
            ".dmllmmkk",
            "..m.m..m.",
        ],
        (true, _) => &[
            "....G.G..",
            "...GgGg..",
            "ll..gmmm.",
            "lmmmmmBkm",
            ".dmllmmkk",
            "..m..m.m.",
        ],
    }
}

fn bullfrog_rows(young: bool, pose: Pose) -> &'static [&'static str] {
    match (young, pose) {
        (false, Pose::Walk1) => &[
            "..............dBdd......",
            "............ddmmmmd.....",
            "..........ddmmmeekd.....",
            "........ddmmmmmeeemd....",
            "......ddmmmmlmmmmmmmdd..",
            "....ddmmmmmmmmmmmmmmmmd.",
            "...dmmmlmmmmmmmdTTdmmmmd",
            "..dmmmmmmmmmmmmdTmdmkkkk",
            ".dmmmmlmmmmmmmmmddmbbbb.",
            ".dmmmmmmmmmmmmmmmbbbbbb.",
            "dmmmmmmmmmlmmmmbbbbbbbb.",
            "dmmlmmmmmmmmmmmbbbbbbm..",
            "dmmmmmmmmmmmmmdbmmbbmm..",
            ".dmmmmmmmmmdd..mm...mm..",
            "..ddmmmmmdd...mm...mm...",
            "..lllllllll..lll..lll...",
        ],
        (false, Pose::Work0) => &[
            "........................",
            "........................",
            "..............dBdd......",
            "............ddmmmmd.....",
            "..........ddmmmeekd.....",
            "........ddmmmmmeeemd....",
            "......ddmmmmlmmmmmmmdd..",
            "....ddmmmmmmmmmmmmmmmmd.",
            "...dmmmlmmmmmmmdTTdmmmmd",
            "..dmmmmmmmmmmmmdTmdmkkkk",
            ".dmmmmlmmmmmmmmmddmbbbbm",
            "dmmmmmmmmmlmmmmmbbbbbbbm",
            "dmmlmmmmmmmmmmmbbbbbbmm.",
            ".dmmmmmmmmmmmdbmmbbmmm..",
            "..ddmmmmmmdd..mm..mmm...",
            "..lllllllll.lll..llll...",
        ],
        (false, Pose::Work1) => &[
            "..............dBdd....m.",
            "............ddmmmmd..mm.",
            "..........ddmmmeekd.mm..",
            "........ddmmmmmeeemmm...",
            "......ddmmmmlmmmmmmmdd..",
            "....ddmmmmmmmmmmmmmmmmd.",
            "...dmmmlmmmmmmmdTTdmmmmd",
            "..dmmmmmmmmmmmmdTmdmkkkk",
            ".dmmmmlmmmmmmmmmddmbbbb.",
            ".dmmmmmmmmmmmmmmmbbbbbb.",
            "dmmmmmmmmmlmmmmbbbbbbbb.",
            "dmmlmmmmmmmmmmmbbbbbbm..",
            "dmmmmmmmmmmmmmdbmmbbb...",
            ".dmmmmmmmmmdd..mm..b....",
            "..ddmmmmmdd....mm.......",
            "..lllllllll...lll.......",
        ],
        (false, Pose::Strike) => &[
            "................dBdd....",
            "..............ddmmmmd...",
            "............ddmmmeekd...",
            "..........ddmmmmmeeemd..",
            "........ddmmmmlmmmmmmmd.",
            "......ddmmmmmmmmmmmmmmmd",
            ".....dmmmlmmmmmmmdTTdmmm",
            "...ddmmmmmmmmmmmmdTmdmkk",
            "..dmmmmlmmmmmmmmmmddmbbb",
            ".dmmmmmmmmmmmmmmmmmbbbbb",
            "dmmmmmmmmmmlmmmmmbbbbbbm",
            "dmmlmmmmmmmmmmmmbbbbbmmm",
            "dmmmmmmmmmmmmmdbbmm..mm.",
            ".dmmmmmmmmmdd..........m",
            "..ddmmmmmmmd............",
            "...lllllllllll..........",
        ],
        (false, _) => &[
            "..............dBdd......",
            "............ddmmmmd.....",
            "..........ddmmmeekd.....",
            "........ddmmmmmeeemd....",
            "......ddmmmmlmmmmmmmdd..",
            "....ddmmmmmmmmmmmmmmmmd.",
            "...dmmmlmmmmmmmdTTdmmmmd",
            "..dmmmmmmmmmmmmdTmdmkkkk",
            ".dmmmmlmmmmmmmmmddmbbbb.",
            ".dmmmmmmmmmmmmmmmbbbbbb.",
            "dmmmmmmmmmlmmmmbbbbbbbb.",
            "dmmlmmmmmmmmmmmbbbbbbm..",
            "dmmmmmmmmmmmmmdbmmbbmm..",
            ".dmmmmmmmmmdd..mm..mm...",
            "..ddmmmmmdd....mm..mm...",
            "..lllllllll...lll.lll...",
        ],
        (true, Pose::Walk1) => &[
            "..........dBd...",
            "........ddmeekd.",
            "......ddmmmmmmd.",
            "....ddmmlmmdTmmd",
            "..ddmmmmmmmdTkkk",
            ".dmmmlmmmmmmbbb.",
            "dmmmmmmmmmbbbbb.",
            "dmmlmmmmmbbbbm..",
            ".dmmmmmdd.mm.m..",
            "..ddmdd..mm.mm..",
            "..lllllll.ll.ll.",
        ],
        (true, Pose::Work0) => &[
            "................",
            "..........dBd...",
            "........ddmeekd.",
            "......ddmmmmmmd.",
            "....ddmmlmmdTmmd",
            "..ddmmmmmmmdTkkk",
            ".dmmmlmmmmmmbbbm",
            "dmmmmmmmmmbbbbbm",
            "dmmlmmmmmbbbbmm.",
            ".dmmmmmdd.mm.m..",
            "..lllllll.ll.ll.",
        ],
        (true, Pose::Work1) => &[
            "..........dBd.m.",
            "........ddmeekmm",
            "......ddmmmmmmd.",
            "....ddmmlmmdTmmd",
            "..ddmmmmmmmdTkkk",
            ".dmmmlmmmmmmbbb.",
            "dmmmmmmmmmbbbbb.",
            "dmmlmmmmmbbbbm..",
            ".dmmmmmdd.mm....",
            "..ddmdd...mm....",
            "..lllllll.ll....",
        ],
        (true, Pose::Strike) => &[
            "...........dBd..",
            ".........ddmeekd",
            ".......ddmmmmmmd",
            ".....ddmmlmmdTmm",
            "...ddmmmmmmmdTkk",
            "..dmmmlmmmmmmbbb",
            ".dmmmmmmmmmbbbbm",
            "dmmlmmmmmmbbbmmm",
            ".dmmmmmmdd....m.",
            "..ddmmdd........",
            "..lllllllll.....",
        ],
        (true, _) => &[
            "..........dBd...",
            "........ddmeekd.",
            "......ddmmmmmmd.",
            "....ddmmlmmdTmmd",
            "..ddmmmmmmmdTkkk",
            ".dmmmlmmmmmmbbb.",
            "dmmmmmmmmmbbbbb.",
            "dmmlmmmmmbbbbm..",
            ".dmmmmmdd.mm.m..",
            "..ddmdd...mm.mm.",
            "..lllllll.ll.ll.",
        ],
    }
}

fn goliath_rows(young: bool, pose: Pose) -> &'static [&'static str] {
    match (young, pose) {
        (false, Pose::Walk1) => &[
            "..........dBd....",
            "........ddmmekd..",
            "......ddmmmmmmmd.",
            "....ddmmlmmmlmmmd",
            "..ddmmmmmmmmmmkkk",
            ".dmmmlmmmlmmmbbb.",
            "dmmmmmmmmmmmbbbb.",
            "dmmlmmmlmmmbbbm..",
            "dmmmmmmmmmdbmbm..",
            ".dmmmmmmd..mm.m..",
            "..ddmmmd..mm..mm.",
            "..lllllll.lll.lll",
        ],
        (false, Pose::Work0) => &[
            "............sss..",
            "..........dBsSsS.",
            "........ddmmsssd.",
            "......ddmmmmmmmmm",
            "....ddmmlmmmlmmmm",
            "..ddmmmmmmmmmmkkk",
            ".dmmmlmmmlmmmbbb.",
            "dmmmmmmmmmmmbbbb.",
            "dmmlmmmlmmmbbbm..",
            ".dmmmmmmmmdbmbm..",
            "..ddmmmmd..mm.mm.",
            "..lllllll.lll.lll",
        ],
        (false, Pose::Work1) => &[
            ".................",
            "..........dBd....",
            "........ddmmekd..",
            "......ddmmmmmmmd.",
            "....ddmmlmmmlmmmd",
            "..ddmmmmmmmmmmkkk",
            ".dmmmlmmmlmmmbbbm",
            "dmmmmmmmmmmmbbbbm",
            "dmmlmmmlmmmbbbmsS",
            ".dmmmmmmmmdbmbsss",
            "..ddmmmmd..mm.sss",
            "..lllllll.lll.sss",
        ],
        (false, Pose::Strike) => &[
            "...........dBd..m",
            ".........ddmmekdm",
            ".......ddmmmmmmmm",
            ".....ddmmlmmmlmmd",
            "...ddmmmmmmmmmmkk",
            "..dmmmlmmmlmmmbbb",
            ".dmmmmmmmmmmmbbbb",
            "dmmlmmmlmmmmbbbm.",
            "dmmmmmmmmmmdbm...",
            ".dmmmmmmmd.......",
            "..ddmmmmmd.......",
            "..llllllllll.....",
        ],
        (false, _) => &[
            "..........dBd....",
            "........ddmmekd..",
            "......ddmmmmmmmd.",
            "....ddmmlmmmlmmmd",
            "..ddmmmmmmmmmmkkk",
            ".dmmmlmmmlmmmbbb.",
            "dmmmmmmmmmmmbbbb.",
            "dmmlmmmlmmmbbbm..",
            "dmmmmmmmmmdbmbm..",
            ".dmmmmmmd..mm.m..",
            "..ddmmmd...mm.mm.",
            "..lllllll.lll.lll",
        ],
        (true, Pose::Walk1) => &[
            ".......dBd..",
            ".....ddmekd.",
            "...ddmmmmmmd",
            ".ddmmlmmmmkk",
            "dmmmmmmmbbb.",
            "dmlmmmmbbbm.",
            ".dmmmd.mm.m.",
            "..llll.ll.ll",
        ],
        (true, Pose::Work0) => &[
            "........sss.",
            ".......dsSs.",
            ".....ddmsssd",
            "...ddmmmmmmd",
            ".ddmmlmmmmkk",
            "dmmmmmmmbbb.",
            "dmlmmmmbbbm.",
            "lllll..ll.ll",
        ],
        (true, Pose::Work1) => &[
            "............",
            ".......dBd..",
            ".....ddmekd.",
            "...ddmmmmmmd",
            ".ddmmlmmmmkk",
            "dmmmmmmmbbbm",
            "dmlmmmmbbbsS",
            "lllll..ll.ss",
        ],
        (true, Pose::Strike) => &[
            "........dBdm",
            "......ddmekd",
            "....ddmmmmmm",
            "..ddmmlmmmmk",
            ".dmmmmmmmbbb",
            "dmlmmmmmbbbm",
            ".dmmmmd.....",
            "..lllllll...",
        ],
        (true, _) => &[
            ".......dBd..",
            ".....ddmekd.",
            "...ddmmmmmmd",
            ".ddmmlmmmmkk",
            "dmmmmmmmbbb.",
            "dmlmmmmbbbm.",
            ".dmmmd.mm.m.",
            "..llll.ll.ll",
        ],
    }
}

fn olm_rows(young: bool, pose: Pose) -> &'static [&'static str] {
    match (young, pose) {
        (false, Pose::Walk1) => &[
            "...............g.g....",
            "......ddmmmmmmmmgmgmB.",
            "dddmmmmmllmmmmmmmmmmmm",
            "..ddmllllllllllllllll.",
            ".....mm.......mm......",
            "....mm.......mm.......",
        ],
        (false, Pose::Work0) => &[
            "......................",
            "...............g.g....",
            "......ddmmmmmmmmgmgmB.",
            "dddmmmmmllmmmmmmmmmmmm",
            "..ddmllllllllllllllllm",
            "....m.m......m.m......",
        ],
        (false, Pose::Work1) => &[
            "................g.g.B.",
            "......ddmmmmmmmmgmgmmm",
            "dddmmmmmllmmmmmmmmmm..",
            "..ddmllllllllllllll...",
            "....m.m......m..m.....",
            "...m...m.....m........",
        ],
        (false, Pose::Strike) => &[
            "................g.g.mB",
            ".......ddmmmmmmmgmgmmm",
            "ddddmmmmmllmmmmmmmm...",
            "...ddmllllllllllllllkm",
            "...m..m.......m.m.....",
            "..m....m.....m...m....",
        ],
        (false, _) => &[
            "...............g.g....",
            "......ddmmmmmmmmgmgmB.",
            "dddmmmmmllmmmmmmmmmmmm",
            "..ddmllllllllllllllll.",
            "....m.m......m.m......",
            "...m...m....m...m.....",
        ],
        (true, Pose::Walk1) => &[
            "..........g.g..",
            "dddmmmmmmmmgmmB",
            "..ddllllllllll.",
            "....mm....mm...",
        ],
        (true, Pose::Work0) => &[
            "...............",
            "..........g.g..",
            "dddmmmmmmmmgmmB",
            "..ddllllllllllm",
        ],
        (true, Pose::Work1) => &[
            "...........g.gB",
            "dddmmmmmmmmmgmm",
            "..ddlllllllll..",
            "...m.m...m.m...",
        ],
        (true, Pose::Strike) => &[
            "...........g.gB",
            "ddddmmmmmmmmgmm",
            "...ddllllllll.m",
            "...m.m...m.m...",
        ],
        (true, _) => &[
            "..........g.g..",
            "dddmmmmmmmmgmmB",
            "..ddllllllllll.",
            "...m.m...m.m...",
        ],
    }
}

fn daot_rows(young: bool, pose: Pose) -> &'static [&'static str] {
    match (young, pose) {
        (false, Pose::Walk1) => &[
            ".....r.....D.",
            "....rR....DoD",
            "....rRR....D.",
            "...rRRRr...s.",
            "..rrrRRrrr.s.",
            ".dmmmmmmmmds.",
            ".dmkmmmmkmds.",
            "..dBkkkkBd.s.",
            "..rRbbbbrlls.",
            ".rRRRbbRRr.s.",
            ".rRRRRRRRr.s.",
            ".rRRRRRRRRrs.",
            "..rrrrrrrr.s.",
            "...dd..dd..s.",
        ],
        (false, Pose::Work0) => &[
            "..........DoD",
            ".....r....yDy",
            "....rR.....s.",
            "....rRR...ls.",
            "...rRRRr..ls.",
            "..rrrRRrrrl..",
            ".dmmmmmmmmd..",
            ".dmkmmmmkmd..",
            "..dBkkkkBd...",
            "..rRbbbbRr...",
            ".rRRRbbRRr...",
            ".rRRRRRRRRr..",
            "..rrrrrrrr...",
            "..dd....dd...",
        ],
        (false, Pose::Work1) => &[
            ".....r.....y.",
            "....rR....yDy",
            "....rRR...DoD",
            "...rRRRr..yDy",
            "..rrrRRrrr.s.",
            ".dmmmmmmmmds.",
            ".dmkmmmmkmds.",
            "..dBkkkkBd.s.",
            "..rRbbbbrlls.",
            ".rRRRbbRRr.s.",
            ".rRRRRRRRr.s.",
            ".rRRRRRRRRrs.",
            "..rrrrrrrr.s.",
            "..dd....dd.s.",
        ],
        (false, Pose::Strike) => &[
            ".............",
            ".....r.......",
            "....rR.......",
            "....rRR....yo",
            "...rRRRr..yDy",
            "..rrrRRrrrs.y",
            ".dmmmmmmmsd..",
            ".dmkmmmmsmd..",
            "..dBkkkskd...",
            "..rRRRsllr...",
            ".rRRRsRRRr...",
            ".rRRsRRRRRr..",
            "..rsrrrrrr...",
            "..dd....dd...",
        ],
        (false, _) => &[
            ".....r.....D.",
            "....rR....DoD",
            "....rRR....D.",
            "...rRRRr...s.",
            "..rrrRRrrr.s.",
            ".dmmmmmmmmds.",
            ".dmkmmmmkmds.",
            "..dBkkkkBd.s.",
            "..rRbbbbrlls.",
            ".rRRRbbRRr.s.",
            ".rRRRRRRRr.s.",
            ".rRRRRRRRRrs.",
            "..rrrrrrrr.s.",
            "..dd....dd.s.",
        ],
        (true, Pose::Walk1) => &[
            "....r...D.",
            "...rRr.DoD",
            "..rrRRr.s.",
            ".dmmmmmds.",
            ".dkmmmkds.",
            "..dkkkdls.",
            ".rRbbbRrs.",
            ".rRRRRRRs.",
            "..rrrrrrs.",
            "...d..d.s.",
        ],
        (true, Pose::Work0) => &[
            "......DoD.",
            "....r..s..",
            "...rRr.s..",
            "..rrRRrl..",
            ".dmmmmmd..",
            ".dkmmmkd..",
            "..dkkkd...",
            ".rRRRRRr..",
            ".rRRRRRRr.",
            "..d...d...",
        ],
        (true, Pose::Work1) => &[
            "....r..yDy",
            "...rRr.DoD",
            "..rrRRryDy",
            ".dmmmmmds.",
            ".dkmmmkds.",
            "..dkkkdls.",
            ".rRbbbRrs.",
            ".rRRRRRRs.",
            "..rrrrrrs.",
            "..d...d.s.",
        ],
        (true, Pose::Strike) => &[
            "..........",
            "....r...yo",
            "...rRr.yDy",
            "..rrRRrs.y",
            ".dmmmmsd..",
            ".dkmmskd..",
            "..dkskd...",
            ".rRsRllr..",
            ".rsRRRRRr.",
            "..d...d...",
        ],
        (true, _) => &[
            "....r...D.",
            "...rRr.DoD",
            "..rrRRr.s.",
            ".dmmmmmds.",
            ".dkmmmkds.",
            "..dkkkdls.",
            ".rRbbbRrs.",
            ".rRRRRRRs.",
            "..rrrrrrs.",
            "..d...d.s.",
        ],
    }
}

// ---- enemies -----------------------------------------------------------------------------------

fn enemy(look: Look, pose: Pose) -> Sprite {
    match look {
        Look::Heron => heron(pose),
        Look::Snake => snake(pose),
        Look::Crow => crow(pose),
        Look::Wisp => wisp(pose),
        Look::Turtle => turtle(pose),
        _ => gorf(pose),
    }
}

const HERON_KEY: [(char, u8); 6] = [
    ('h', HERON0),
    ('H', HERON1),
    ('W', HERON2),
    ('b', BEAK),
    ('k', INK),
    ('l', HERON0),
];

/// A grey heron, facing right: standing, wading, stabbing or in flight.
fn heron(pose: Pose) -> Sprite {
    let rows: &[&str] = match pose {
        Pose::Walk1 => &[
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
            "......l..l....",
            ".....l....l...",
            "....ll....ll..",
        ],
        Pose::Work0 | Pose::Work1 => &[
            "..............",
            "..............",
            "..............",
            "..............",
            "..............",
            "....hhhh......",
            "..hhHHHHhh....",
            ".hHHWWWHHHhh..",
            "hhHWWWWHHHHHh.",
            ".hhHHHHHHhhHh.",
            "...hhhhhh..hHh",
            "......l..l..hk",
            "......l..l..bb",
            "......l..l...b",
            "......l..l...b",
            ".....ll..ll...",
        ],
        Pose::Strike => &[
            "..............",
            "..............",
            "..............",
            "........hhh...",
            ".......hHkHbbb",
            "......hHHh....",
            "....hhHHh.....",
            "..hhHHHHh.....",
            ".hHHWWWHHh....",
            "hhHWWWWHHHh...",
            ".hhHHHHHHh....",
            "...hhhhhh.....",
            "......l..l....",
            "......l..l....",
            ".....l....l...",
            "....ll....ll..",
        ],
        Pose::Fly0 => &[
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
        ],
        Pose::Fly1 => &[
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
        ],
        _ => &[
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
            "......l..l....",
            "......l..l....",
            ".....ll..ll...",
        ],
    };
    grid(rows, &HERON_KEY)
}

/// A garter snake: dark, with a yellow stripe down its side and a red tongue. Painted along a
/// wave so each frame is a smooth curve.
fn snake(pose: Pose) -> Sprite {
    let mut s = Sprite::new(16, 6);
    let wave: [i32; 12] = match pose {
        Pose::Walk1 | Pose::Work1 => [1, 1, 2, 2, 3, 3, 3, 2, 2, 1, 1, 1],
        Pose::Strike => [3, 3, 3, 3, 3, 3, 3, 3, 3, 2, 1, 0],
        _ => [3, 3, 2, 2, 1, 1, 1, 2, 2, 3, 3, 2],
    };
    for (x, &y) in wave.iter().enumerate() {
        let x = x as i32;
        if x < 2 {
            s.set(x, y + 1, SNAKE1);
            continue;
        }
        s.set(x, y, SNAKE0);
        s.set(x, y + 1, if x % 3 == 0 { SNAKE1 } else { SNAKE_Y });
        if x > 3 {
            s.set(x, y + 2, SNAKE2);
        }
    }
    // The head, a little wider than the neck.
    let hy = wave[11] - 1;
    s.rect(12, hy + 1, 3, 2, SNAKE1);
    s.rect(12, hy, 2, 1, SNAKE0);
    s.set(14, hy + 2, SNAKE2);
    s.set(15, hy + 1, SNAKE1);
    s.set(13, hy + 1, EYE);
    s.set(12, hy + 2, SNAKE_Y);
    match pose {
        Pose::Strike => {
            // Jaws open, tongue out.
            s.set(15, hy + 2, 0);
            s.set(14, hy + 3, SNAKE2);
            s.set(15, hy + 3, SNAKE2);
            s.set(15, hy + 2, GILL1);
        }
        Pose::Walk1 | Pose::Work1 => {}
        _ => {
            s.set(15, hy + 3, GILL1);
        }
    }
    trim_rows(&mut s);
    // Keep every frame the same height, belly on the bottom row.
    let mut t = Sprite::new(16, 6);
    t.blit(&s, 0, 6 - s.h);
    t
}

/// A crow: glossy blue-black, grey bill; it hops on the ground and flaps in the air.
fn crow(pose: Pose) -> Sprite {
    let rows: &[&str] = match pose {
        Pose::Walk1 => &[
            ".............",
            ".............",
            "......dmm....",
            ".....dmmebb..",
            "....ddmmm....",
            "..ddmmlmmd...",
            "dddmmllllm...",
            "..dmmmmmd....",
            ".....b.b.....",
            "....bb.bb....",
        ],
        Pose::Work0 | Pose::Work1 | Pose::Strike => &[
            ".............",
            ".............",
            ".............",
            ".............",
            "..ddmmm......",
            "dddmmlmmmd...",
            "..dmmmllmmm..",
            "...dmmmmmmeb.",
            "....b..b..bbb",
            "...bb.bb.....",
        ],
        Pose::Fly0 => &[
            ".ll..........",
            ".lml.........",
            "..lmml.......",
            "...dmml......",
            "....ddmm..mm.",
            "dddmmmmmmmmeb",
            "..ddmmmmmmm.b",
            "....d.d......",
            ".............",
            ".............",
        ],
        Pose::Fly1 => &[
            ".............",
            ".............",
            ".............",
            "..........mm.",
            "dddmmmmmmmmeb",
            "..ddmmmmmmm.b",
            "....ddmml....",
            "...dmml......",
            "..lml........",
            ".ll..........",
        ],
        _ => &[
            ".............",
            ".............",
            "......dmm....",
            ".....dmmebb..",
            "....ddmmm....",
            "..ddmmlmmd...",
            "dddmmllllm...",
            "..dmmmmmd....",
            ".....b..b....",
            "....bb.bb....",
        ],
    };
    grid(
        rows,
        &[
            ('d', CROW0),
            ('m', CROW1),
            ('l', CROW2),
            ('b', STONE2),
            ('e', TEXT_DIM),
        ],
    )
}

/// A bog wisp: a small cold flame with two eyes. It floats, so every pose is a float.
fn wisp(pose: Pose) -> Sprite {
    let alt = matches!(pose, Pose::Walk1 | Pose::Fly1 | Pose::Work1);
    let mut s = grid(
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
    );
    match pose {
        Pose::Strike => {
            // Flaring: the cold fire burns white.
            s = s.recolored(&[(WISP1, WISP2), (WISP0, WISP1)]);
        }
        Pose::Work0 | Pose::Work1 => {
            // Drinking dew: a drop rises into it.
            let mut t = Sprite::new(8, 10);
            t.blit(&s, 0, 0);
            t.set(3 + alt as i32, 8, DEW);
            t.set(3 + alt as i32, 9, DEW_DK);
            s = t;
        }
        _ => {}
    }
    s
}

/// A snapping turtle: a heavy ridged shell, a big hooked-beaked head and a spiked tail.
fn turtle(pose: Pose) -> Sprite {
    let rows: &[&str] = match pose {
        Pose::Walk1 | Pose::Work1 => &[
            ".......l...l...l......",
            ".....ddlmmmlmmmld.....",
            "....dmmmmmmmmmmmmd....",
            "...dmlmmmlmmmlmmmmd...",
            "..dmmmmmmmmmmmmmmmmdSS",
            ".dmmmlmmmlmmmlmmmmdSeS",
            "sddmmmmmmmmmmmmmmddSSb",
            "SsdddddddddddddddsSkbb",
            ".s.sSSs....sSSs.SSS...",
            "....sSs....sSs..SS....",
            "...sSSs...sSSs........",
        ],
        Pose::Strike => &[
            "......l...l...l.......",
            "....ddlmmmlmmmld......",
            "...dmmmmmmmmmmmmd.....",
            "..dmlmmmlmmmlmmmmdSS..",
            ".dmmmmmmmmmmmmmmmdSSeS",
            "dmmmlmmmlmmmlmmmdSSSSb",
            "ddmmmmmmmmmmmmmddsSk.b",
            "SsddddddddddddddsSSkbb",
            "s.sSSs....sSSs..SS....",
            "...sSs....sSs.........",
            "..sSSs...sSSs.........",
        ],
        Pose::DugIn => &[
            "......................",
            "......................",
            ".......l...l...l......",
            ".....ddlmmmlmmmld.....",
            "....dmmmmmmmmmmmmd....",
            "...dmlmmmlmmmlmmmmd...",
            "..dmmmmmmmmmmmmmmmmd..",
            ".dmmmlmmmlmmmlmmmmdk..",
            ".ddmmmmmmmmmmmmmmdd...",
            "..dddddddddddddddd....",
            "...s.............s....",
        ],
        _ => &[
            ".......l...l...l......",
            ".....ddlmmmlmmmld.....",
            "....dmmmmmmmmmmmmd....",
            "...dmlmmmlmmmlmmmmd...",
            "..dmmmmmmmmmmmmmmmmdSS",
            ".dmmmlmmmlmmmlmmmmdSeS",
            "sddmmmmmmmmmmmmmmddSSb",
            "SsdddddddddddddddsSkbb",
            ".s.sSSs....sSSs.SSS...",
            "...sSSs.....sSSs.SS...",
            "..sSSs.......sSSs.....",
        ],
    };
    grid(
        rows,
        &[
            ('d', TURT0),
            ('m', TURT1),
            ('l', TURT2),
            ('s', TSKIN0),
            ('S', TSKIN1),
            ('k', INK),
            ('e', EYE),
            ('b', STONE2),
        ],
    )
}

/// Gorf: a bloated frog sorcerer, warty and hunched in a ragged cowl, his bulging eyes and the
/// orb of his twisted staff burning violet, a necklace of teeth on his chest.
fn gorf(pose: Pose) -> Sprite {
    let casting = matches!(pose, Pose::Work0 | Pose::Work1 | Pose::Strike);
    let step = matches!(pose, Pose::Walk1 | Pose::Work1);
    let mut rows: Vec<String> = [
        "......................aAa.",
        "...dmmd...c.c...dmmd..AWA.",
        "..dmAWmd.ccc...dmWAmd.aAa.",
        "..dmAAmdddddddddmAAmd..s..",
        ".rdmmmmmlmmmmmmlmmmmdr.s..",
        ".rdmmlmmmmmmlmmmmmlmdr.s..",
        "rRdkkkkkkkkkkkkkkkkkdRrs..",
        "rRdmkAAAAAAAAAAAAAkmdRrs..",
        "rRRddmmmmmmmmmmmmmddRRrs..",
        "rRRRdddcmcmcmcmcdddRRRmms.",
        "rRRRRRRRRRRRRRRRRRRRRmmms.",
        "rRRRRRRrRRRRRRrRRRRRRRms..",
        "rRRRRRRrRRRRRRrRRRRRRrs...",
        "rRRRRRrRRRRRRRRrRRRRRrs...",
        "rRRRRRrRRRRRRRRrRRRRRRs...",
        "rRRRRRRRRRRRRRRRRRRRRrs...",
        "rRRRRrRRRRRRRRRRRrRRRRs...",
        "rRRRRrRRRRRRRRRRRrRRRRs...",
        "rRRRRRRRRRRRRRRRRRRRRRs...",
        ".rRRRRRRRRRRRRRRRRRRRrs...",
        ".rRrRRrRRRrRRRRrRRrRRrs...",
        "..r.rr.rr.rrr.rr.rr.r.s...",
        "..mmmmm.........mmmmm.s...",
        ".mmmmmmm.......mmmmmmms...",
    ]
    .iter()
    .map(|r| r.to_string())
    .collect();
    if step {
        rows[22] = "...mmmmm.......mmmmm..s...".into();
        rows[23] = "..mmmmmmm.....mmmmmmm.s...".into();
    }
    if casting {
        // The other claw raised beside his face, a violet flame dancing on it.
        rows[7] = "rRdmkWWWWWWWWWWWWWkmdRrs..".into();
        let mut g: Vec<Vec<char>> = rows.iter().map(|r| r.chars().collect()).collect();
        let pad = ["aAa", "AWA", "aAa", "mm.", "mm.", "mm."];
        let mut out: Vec<String> = vec![];
        for (y, row) in g.iter_mut().enumerate() {
            let pre: String = if y < 6 { pad[y].into() } else { "...".into() };
            out.push(pre + &row.iter().collect::<String>());
        }
        rows = out;
    } else {
        rows = rows.iter().map(|r| format!("...{r}")).collect();
    }
    let refs: Vec<&str> = rows.iter().map(|s| s.as_str()).collect();
    grid(
        &refs,
        &[
            ('d', GORF0),
            ('m', GORF1),
            ('l', GORF2),
            ('r', GROBE0),
            ('R', GROBE1),
            ('a', WISP0),
            ('A', WISP1),
            ('W', WISP2),
            ('k', INK),
            ('s', WOOD0),
            ('c', CREAM),
        ],
    )
}

// ---- buildings ---------------------------------------------------------------------------------

/// A blank building canvas: the footprint's width less the hull, `h` tall (hull not included).
fn canvas(b: BLook, h: i32) -> Sprite {
    Sprite::new(footprint(b).0 * TILE - 2, h)
}

/// Outline the body, then lay unhulled light (glows, motes, spilled light) over it. `over` is in
/// the outlined sprite's coordinates (canvas coordinates plus one).
fn finish(body: Sprite, over: Option<Sprite>) -> Sprite {
    let mut o = body.outlined();
    if let Some(v) = over {
        o.blit(&v, 0, 0);
    }
    o
}

fn building(b: BLook) -> Sprite {
    let s = match b {
        BLook::Hall => hall(),
        BLook::Burrow => burrow(),
        BLook::TwigLoft => twig_loft(),
        BLook::MorelCave => fungus_cave(false),
        BLook::IroncapCave => fungus_cave(true),
        BLook::FairyRing => fairy_ring(),
        BLook::ChorusLog => chorus_log(),
        BLook::EmberLog => ember_log(),
        BLook::ToadHall => toad_hall(),
        BLook::ShellForge => shell_forge(),
        BLook::Cauldron => cauldron(),
        BLook::StarPond => star_pond(),
        BLook::NewtHouse => newt_house(),
        BLook::SpawningPool => spawning_pool(),
        BLook::DaotsGrotto => daots_grotto(),
        BLook::OlmsCave => olms_cave(),
        BLook::Nest => nest(false),
        BLook::LoneNest => nest(true),
        BLook::GorfStone => gorf_stone(),
        BLook::LeechPool => leech_pool(),
        BLook::CrowRoost => crow_roost(),
        BLook::GorfHollow => gorf_hollow(),
    };
    let (fw, fh) = footprint(b);
    debug_assert_eq!(s.w, fw * TILE, "{b:?} is its footprint wide");
    // Never shorter than the footprint, so it can be picked anywhere on its squares.
    if s.h < fh * TILE {
        let mut t = Sprite::new(s.w, fh * TILE);
        t.blit(&s, 0, fh * TILE - s.h);
        return t;
    }
    s
}

/// Texture a region already filled with `base`: a lit top-left, a shaded bottom-right.
fn shade_mass(s: &mut Sprite, cx: i32, cy: i32, rx: i32, ry: i32, base: u8, lo: u8, hi: u8) {
    for y in cy - ry..=cy + ry {
        for x in cx - rx..=cx + rx {
            if s.get(x, y) != base {
                continue;
            }
            let (dx, dy) = (x - cx, y - cy);
            // A diagonal light from the upper left.
            let d = dx * ry + dy * rx;
            let edge = rx * ry;
            if d > edge * 2 / 5 || (d > edge / 6 && (x + y) & 1 == 0) {
                s.set(x, y, lo);
            } else if d < -edge * 3 / 5 && (x + y) & 1 == 0 || d < -edge * 4 / 5 {
                s.set(x, y, hi);
            }
        }
    }
}

/// A foreground pool: dark water, a lighter rim of ripples, a few glints.
fn pool(s: &mut Sprite, cx: i32, cy: i32, rx: i32, ry: i32, seed: u32) {
    s.ellipse(cx, cy, rx, ry, POOL1);
    s.ellipse(cx, cy + 1, rx - 2, ry - 1, POOL0);
    for k in 0..(rx / 3) {
        let h = hash(k, 3, seed);
        let x = cx - rx + 3 + (h % (2 * rx as u32 - 5)) as i32;
        let y = cy - ry + 2 + (h / 97 % (2 * ry as u32 - 3).max(1)) as i32;
        if s.get(x, y) == POOL0 && s.get(x + 2, y) == POOL0 {
            s.rect(x, y, 3, 1, POOL1);
        }
    }
    for k in 0..(rx / 4).max(1) {
        let h = hash(k, 7, seed);
        let x = cx - rx / 2 + (h % rx as u32) as i32;
        let y = cy - ry / 2 + (h / 31 % ry.max(1) as u32) as i32;
        if s.get(x, y) == POOL0 {
            s.set(x, y, WGLINT[(h % 3) as usize]);
        }
    }
}

/// A rock outcrop.
fn rock_mass(s: &mut Sprite, cx: i32, cy: i32, rx: i32, ry: i32, seed: u32) {
    s.ellipse(cx, cy, rx, ry, STONE1);
    shade_mass(s, cx, cy, rx, ry, STONE1, STONE0, STONE2);
    // Cracks and ledges.
    for k in 0..(rx / 3) {
        let h = hash(k, 11, seed);
        let x = cx - rx + 2 + (h % (2 * rx as u32 - 3)) as i32;
        let y = cy - ry + 3 + (h / 53 % (2 * ry as u32 - 4).max(1)) as i32;
        let len = 2 + (h / 7 % 3) as i32;
        for i in 0..len {
            s.over(x + i, y + i / 2, STONE0);
        }
        s.over(x, y - 1, STONE2);
    }
}

/// A little toadstool: cream stem and a spotted red cap; `r` is the cap's half-width.
fn toadstool(s: &mut Sprite, x: i32, y: i32, r: i32) {
    if r <= 1 {
        s.rect(x, y - 1, 1, 2, CREAM);
        s.rect(x - 1, y - 2, 3, 1, CAP1);
        s.set(x, y - 3, CAP1);
        s.set(x - 1, y - 2, CAP2);
        s.set(x + 1, y - 2, CAP0);
        return;
    }
    s.rect(x, y - r - 1, 2, r + 1, CREAM);
    s.set(x + 1, y - 1, HUSK0);
    s.ellipse(x, y - r - 1, r, (r / 2).max(1), CAP1);
    s.set(x - r + 1, y - r - 1, CAP2);
    s.set(x, y - r - 2, CREAM);
    s.rect(x - r, y - r, 2 * r + 1, 1, CAP0);
    s.set(x - r, y - r - 1, CAP0);
    if r >= 3 {
        s.set(x + 2, y - r - 1, CREAM);
        s.set(x - 2, y - r - 1, CREAM);
    }
}

/// A morel: a tall pitted brown cone on a pale stem.
fn morel(s: &mut Sprite, x: i32, y: i32, tall: i32) {
    s.rect(x, y - 1, 2, 2, CREAM);
    s.set(x + 1, y, HUSK0);
    for i in 0..tall {
        let w = if i == tall - 1 {
            1
        } else if i >= tall - 3 {
            2
        } else {
            3
        };
        let x0 = x + if w == 3 { -1 } else { 0 } + (w == 1) as i32;
        for j in 0..w {
            let c = if (i + j) % 2 == 0 { MOREL1 } else { MOREL0 };
            s.set(x0 + j, y - 2 - i, c);
        }
    }
}

/// An ironcap: a broad flat grey-blue cap with a bright rim, on a short pale stem.
fn ironcap(s: &mut Sprite, x: i32, y: i32, r: i32) {
    s.rect(x, y - 2, 1, 2, CREAM);
    s.rect(x - r, y - 3, 2 * r + 1, 1, IRON0);
    s.rect(x - r + 1, y - 4, 2 * r - 1, 1, IRON1);
    s.set(x - r, y - 4, IRON2);
    s.set(x - r + 1, y - 4, IRON2);
    if r >= 2 {
        s.rect(x - r + 2, y - 5, 2 * r - 3, 1, IRON1);
        s.set(x - r + 2, y - 5, IRON2);
    }
}

/// The Elder's hall: a great spotted mushroom standing over a pool, its stalk lit from within.
fn hall() -> Sprite {
    let mut s = canvas(BLook::Hall, 62);
    let mut glow = Sprite::new(s.w + 2, s.h + 2);
    // The pool at its foot.
    pool(&mut s, 31, 54, 29, 7, 1);
    // The stalk, flaring into roots that grip the pool's edge.
    s.rect(23, 28, 16, 26, CREAM);
    s.ellipse(31, 52, 11, 3, CREAM);
    for (x, y) in [(20, 53), (42, 53), (17, 55), (45, 55)] {
        s.rect(x, y, 3, 2, CREAM);
    }
    s.rect(23, 28, 3, 27, HUSK0);
    s.dither_rect(26, 28, 3, 27, CREAM, HUSK0);
    s.rect(35, 28, 4, 25, HUSK1);
    // A lit door and windows.
    s.ellipse(31, 45, 4, 5, WOOD0);
    s.rect(27, 45, 9, 8, WOOD0);
    s.ellipse(31, 46, 2, 3, GLOW);
    s.rect(29, 46, 5, 7, GLOW);
    s.rect(31, 43, 1, 10, COPPER2);
    s.rect(34, 34, 3, 3, WOOD0);
    s.set(35, 35, GLOW);
    s.rect(25, 37, 3, 3, WOOD0);
    s.set(26, 38, GLOW);
    // The cap: a great dome with gills underneath and cream spots.
    s.ellipse(31, 22, 30, 10, CAP0);
    s.ellipse(31, 18, 30, 15, CAP1);
    for x in 4..59 {
        if x % 3 == 0 {
            s.over(x, 26, CAP0);
            s.over(x, 27, WOOD0);
        }
    }
    s.ellipse(31, 26, 26, 2, CAP0);
    for x in (6..58).step_by(2) {
        s.over(x, 27, WOOD0);
    }
    s.ellipse(22, 9, 12, 5, CAP2);
    s.dither_in(31, 18, 30, 15, CAP1, CAP2);
    s.ellipse(31, 19, 29, 12, CAP1);
    s.ellipse(24, 10, 11, 4, CAP2);
    s.dither_in(24, 12, 13, 4, CAP1, CAP2);
    s.dither_in(40, 23, 18, 4, CAP1, CAP0);
    for (x, y, r) in [
        (12, 16, 3),
        (24, 7, 3),
        (40, 9, 3),
        (51, 17, 2),
        (31, 17, 2),
        (18, 22, 2),
        (44, 21, 2),
        (6, 21, 1),
        (57, 22, 1),
        (33, 3, 1),
    ] {
        s.ellipse(x, y, r, (r - 1).max(1), CREAM);
        s.set(x + r - 1, y + (r - 1).max(1), HUSK0);
    }
    // Small caps at the stalk's foot and lily pads on the pool.
    toadstool(&mut s, 19, 52, 2);
    toadstool(&mut s, 44, 51, 1);
    for (x, y, f) in [(8, 55, false), (52, 57, true), (14, 58, false)] {
        lily_pad(&mut s, x, y, f);
    }
    // Warm light spilling from the door onto the water.
    for (i, y) in (56..60).enumerate() {
        for x in 29 - i as i32..=33 + i as i32 {
            if (x + y) & 1 == 0 && s.get(x, y) == POOL0 {
                glow.set(x + 1, y + 1, COPPER0);
            }
        }
    }
    finish(s, Some(glow))
}

/// The Burrow: a big mossy earth mound full of holes, its main door framed in wood.
fn burrow() -> Sprite {
    let mut s = canvas(BLook::Burrow, 30);
    s.ellipse(15, 30, 15, 21, EARTH1);
    shade_mass(&mut s, 15, 30, 15, 21, EARTH1, EARTH0, EARTH2);
    for k in 0..16 {
        let h = hash(k, 1, 77);
        s.over(2 + (h % 26) as i32, 12 + (h / 26 % 17) as i32, EARTH2);
        s.over(3 + (h / 7 % 24) as i32, 14 + (h / 99 % 15) as i32, EARTH0);
    }
    // A cap of moss with ferns.
    s.ellipse(14, 11, 10, 3, LEAF0);
    s.ellipse(13, 10, 7, 2, LEAF1);
    s.dither_in(14, 13, 10, 1, LEAF0, EARTH1);
    for (fx, fy) in [(20, 8), (8, 9)] {
        for (dx, dy) in [
            (0, 0),
            (-1, -1),
            (1, -1),
            (-2, -2),
            (2, -2),
            (0, -3),
            (-3, -1),
            (3, -1),
        ] {
            s.set(fx + dx, fy + dy, LEAF2);
        }
    }
    // Small holes, each with a lip.
    for (x, y) in [(6, 19), (24, 17), (19, 13)] {
        s.ellipse(x, y + 1, 3, 2, EARTH2);
        s.ellipse(x, y + 1, 2, 2, INK);
        s.set(x, y + 2, EARTH0);
    }
    // Caecilians peeping out.
    s.set(6, 19, CAEC1);
    s.set(7, 19, CAEC2);
    s.set(24, 17, CAEC1);
    // The main door, framed in wood, warm inside.
    s.ellipse(14, 24, 5, 5, WOOD1);
    s.rect(9, 24, 11, 6, WOOD1);
    s.ellipse(14, 24, 4, 4, INK);
    s.rect(10, 24, 9, 6, INK);
    s.rect(11, 26, 7, 4, COPPER0);
    s.dither_rect(11, 25, 7, 5, COPPER0, INK);
    s.set(14, 27, COPPER1);
    s.set(14, 19, WOOD2);
    // Roots, pebbles and a toadstool.
    s.set(2, 24, WOOD0);
    s.set(3, 25, WOOD0);
    s.set(27, 26, STONE2);
    s.set(26, 28, STONE1);
    toadstool(&mut s, 25, 29, 1);
    toadstool(&mut s, 4, 29, 1);
    s.outlined()
}

/// The Twig Loft: a woven hut sheltered under the arching roots of an old tree.
fn twig_loft() -> Sprite {
    let mut s = canvas(BLook::TwigLoft, 36);
    // The stump and its crown of leaves.
    s.rect(11, 6, 9, 9, WOOD1);
    s.rect(11, 6, 2, 9, WOOD2);
    s.rect(18, 6, 2, 9, WOOD0);
    s.ellipse(15, 4, 10, 4, LEAF0);
    s.ellipse(13, 3, 7, 2, LEAF1);
    s.dither_in(15, 6, 10, 2, LEAF0, BULL0);
    s.set(9, 2, LEAF2);
    s.set(14, 1, LEAF2);
    // Back roots arching down to the ground.
    curve(&mut s, (13, 12), (3, 14), (2, 35), 3, WOOD0);
    curve(&mut s, (18, 12), (28, 14), (28, 35), 3, WOOD0);
    // The hut: a woven dome nestled under the roots.
    s.ellipse(15, 27, 9, 9, TWIG0);
    s.rect(6, 27, 19, 9, TWIG0);
    for y in 17..36 {
        for x in 5..26 {
            if s.get(x, y) == TWIG0 && ((x + y * 2) % 4 == 0 || (2 * x - y) % 5 == 0) {
                s.set(x, y, TWIG1);
            }
        }
    }
    s.dither_rect(5, 31, 21, 5, TWIG0, WOOD0);
    // A round door and window, lit.
    s.ellipse(16, 30, 3, 4, WOOD0);
    s.rect(13, 30, 7, 6, WOOD0);
    s.ellipse(16, 31, 2, 3, GLOW);
    s.rect(14, 31, 5, 5, GLOW);
    s.rect(16, 29, 1, 7, COPPER1);
    s.ellipse(10, 24, 2, 2, WOOD0);
    s.set(10, 24, GLOW);
    s.set(9, 24, GLOW);
    s.set(10, 23, GLOW);
    // Front roots gripping the hut's sides.
    curve(&mut s, (12, 13), (5, 20), (5, 35), 2, WOOD1);
    curve(&mut s, (19, 13), (26, 20), (25, 35), 2, WOOD1);
    for (x, y) in [(5, 25), (25, 26), (4, 31)] {
        s.set(x, y, WOOD2);
    }
    // A lantern hanging from the right root, and a little ladder up to the crown.
    s.set(25, 18, POLE);
    s.rect(24, 19, 3, 3, COPPER0);
    s.set(25, 20, GLOW);
    for y in (9..16).step_by(2) {
        s.rect(20, y, 3, 1, WOOD2);
    }
    s.outlined()
}

/// A fungus cave: a morel cave in warm earthen rock, or an ironcap cave in cold blue stone.
fn fungus_cave(iron: bool) -> Sprite {
    let mut s = canvas(BLook::MorelCave, 30);
    let (base, lo, hi) = if iron {
        (STONE1, STONE0, STONE2)
    } else {
        (EARTH1, EARTH0, EARTH2)
    };
    s.ellipse(15, 24, 14, 16, base);
    s.rect(0, 22, 30, 8, base);
    shade_mass(&mut s, 15, 24, 15, 16, base, lo, hi);
    for k in 0..8 {
        let h = hash(k, 11, if iron { 5 } else { 6 });
        let (x, y) = (3 + (h % 24) as i32, 10 + (h / 24 % 16) as i32);
        for i in 0..3 {
            s.over(x + i, y + i / 2, lo);
        }
        s.over(x, y - 1, hi);
    }
    if iron {
        // Veins of iron ore.
        for (x, y) in [
            (5, 16),
            (6, 17),
            (7, 17),
            (8, 18),
            (22, 13),
            (23, 14),
            (24, 14),
            (26, 21),
        ] {
            s.over(x, y, IRON1);
        }
        s.over(5, 15, IRON2);
        s.over(22, 12, IRON2);
    } else {
        // Roots hanging over the earthen lip.
        for (x, l) in [(9, 3), (12, 2), (20, 4)] {
            for i in 0..l {
                s.set(x + i % 2, 17 + i, WOOD0);
            }
        }
    }
    // The mouth, dark, with the fungus bed's light inside.
    s.ellipse(15, 24, 8, 8, lo);
    s.ellipse(15, 25, 7, 7, INK);
    s.rect(8, 25, 15, 5, INK);
    s.rect(7, 29, 17, 1, lo);
    if iron {
        s.dither_rect(10, 27, 11, 2, INK, IRON0);
        s.set(13, 27, IRON1);
        s.set(17, 28, IRON1);
    } else {
        s.dither_rect(10, 27, 11, 2, INK, MOREL0);
        s.set(12, 28, COPPER0);
        s.set(17, 27, COPPER0);
    }
    // A cap of moss on the crown.
    s.ellipse(14, 9, 9, 2, LEAF0);
    s.ellipse(12, 8, 6, 1, LEAF1);
    if iron {
        // Shelf ironcaps stacked up the rock and clustered at the mouth.
        for (x, y, r) in [
            (3, 29, 2),
            (26, 29, 3),
            (28, 25, 1),
            (5, 19, 2),
            (24, 17, 2),
            (11, 8, 2),
            (19, 8, 3),
            (1, 25, 1),
        ] {
            ironcap(&mut s, x, y, r);
        }
    } else {
        for (x, y, t) in [
            (2, 29, 7),
            (5, 29, 5),
            (24, 29, 8),
            (27, 29, 6),
            (4, 21, 5),
            (11, 8, 5),
            (16, 8, 7),
            (20, 8, 4),
        ] {
            morel(&mut s, x, y, t);
        }
    }
    s.outlined()
}

/// The Fairy Ring: a ring of spotted toadstools on a ring of darker grass, spores drifting up.
fn fairy_ring() -> Sprite {
    let mut s = canvas(BLook::FairyRing, 30);
    let mut glow = Sprite::new(s.w + 2, s.h + 2);
    let (cx, cy, rx, ry) = (15, 19, 13, 9);
    s.ellipse(cx, cy, rx + 1, ry + 1, LEAF0);
    s.ellipse(cx, cy, rx - 2, ry - 2, 0);
    s.dither_in(cx, cy, rx + 1, ry + 1, LEAF0, BULL0);
    // Toadstools around the ellipse (a fixed table of directions), far ones first.
    let dirs = [
        (13, 0),
        (11, 5),
        (6, 8),
        (0, 9),
        (-6, 8),
        (-11, 5),
        (-13, 0),
        (-11, -5),
        (-6, -8),
        (0, -9),
        (6, -8),
        (11, -5),
    ];
    let mut caps: Vec<(i32, i32, i32)> = dirs
        .iter()
        .map(|&(dx, dy)| {
            (
                cx + dx,
                cy + dy + 1,
                if dy > 4 {
                    3
                } else if dy > -4 {
                    2
                } else {
                    1
                },
            )
        })
        .collect();
    caps.sort_by_key(|c| c.1);
    for (x, y, r) in caps {
        toadstool(&mut s, x, y, r);
    }
    // Spores rising from the middle, unhulled.
    for (x, y, k) in [
        (15, 17, 0),
        (12, 14, 1),
        (18, 12, 2),
        (14, 9, 0),
        (17, 18, 1),
        (11, 18, 2),
        (16, 5, 1),
        (13, 20, 0),
        (19, 20, 2),
    ] {
        glow.set(x + 1, y + 1, SPORE[k]);
    }
    finish(s, Some(glow))
}

/// The Chorus Log: a great hollow log turned into a stage, its end open like a sounding horn.
fn chorus_log() -> Sprite {
    let mut s = canvas(BLook::ChorusLog, 30);
    // The log's body.
    s.rect(4, 12, 34, 16, WOOD1);
    s.ellipse(4, 20, 3, 8, WOOD1);
    s.rect(4, 12, 34, 3, WOOD2);
    s.rect(4, 24, 34, 4, WOOD0);
    s.dither_rect(4, 21, 34, 3, WOOD1, WOOD0);
    s.dither_rect(4, 15, 34, 2, WOOD1, WOOD2);
    for (x, y, l) in [(8, 17, 9), (20, 19, 12), (12, 22, 6), (27, 16, 8)] {
        s.rect(x, y, l, 1, WOOD0);
    }
    // The open end: bark rim, dark hollow, a warm glow deep inside.
    s.ellipse(39, 20, 5, 9, WOOD2);
    s.ellipse(39, 20, 4, 8, TWIG1);
    s.ellipse(39, 20, 3, 6, WOOD0);
    s.ellipse(40, 21, 2, 4, INK);
    s.set(40, 22, COPPER0);
    // The stage on top: a trimmed moss carpet, two lantern caps and a little mushroom crowd.
    s.rect(6, 10, 30, 2, LEAF0);
    s.rect(8, 9, 26, 1, LEAF1);
    s.dither_rect(6, 11, 30, 1, LEAF0, WOOD2);
    for x in [7, 33] {
        s.rect(x, 4, 1, 5, WOOD0);
        s.rect(x - 1, 0, 3, 1, WOOD0);
        s.rect(x - 1, 1, 3, 3, COPPER0);
        s.set(x, 2, GLOW);
        s.set(x, 1, GLOW);
    }
    toadstool(&mut s, 12, 9, 1);
    toadstool(&mut s, 29, 9, 1);
    // A notch of shelf fungus and moss on the side.
    s.rect(16, 24, 4, 1, CREAM);
    s.rect(17, 23, 2, 1, CREAM);
    s.ellipse(10, 27, 5, 1, LEAF0);
    s.ellipse(30, 27, 4, 1, LEAF0);
    s.outlined()
}

/// The Ember Log: a charred log smouldering from within, cracks glowing.
fn ember_log() -> Sprite {
    let mut s = canvas(BLook::EmberLog, 24);
    let mut glow = Sprite::new(s.w + 2, s.h + 2);
    s.rect(3, 12, 22, 11, STONE0);
    s.ellipse(3, 17, 2, 5, STONE0);
    s.rect(3, 12, 22, 2, STONE1);
    s.dither_rect(3, 14, 22, 2, STONE0, STONE1);
    s.dither_rect(3, 20, 22, 3, STONE0, INK);
    // Glowing cracks.
    let cracks = [
        (5, 15, 9, 16),
        (9, 16, 12, 14),
        (12, 14, 17, 17),
        (17, 17, 21, 15),
        (8, 19, 14, 20),
        (14, 20, 19, 19),
        (20, 13, 22, 18),
    ];
    for (x0, y0, x1, y1) in cracks {
        s.line(x0, y0, x1, y1, EMBER1);
    }
    for (x, y) in [(9, 16), (12, 14), (17, 17), (14, 20), (21, 15)] {
        s.set(x, y, EMBER2);
    }
    // The open end: a glowing core.
    s.ellipse(25, 17, 3, 5, STONE1);
    s.ellipse(25, 17, 2, 4, EMBER0);
    s.ellipse(25, 17, 1, 2, EMBER1);
    s.set(25, 17, EMBER2);
    // Small flames licking the top.
    for (x, h) in [(8, 3), (13, 5), (18, 4), (22, 2)] {
        s.rect(x, 12 - h, 1, h, EMBER1);
        s.set(x, 12 - h, EMBER2);
        s.set(x + 1, 11, EMBER0);
        s.set(x, 11, EMBER2);
    }
    // Ash and a ring of stones.
    for (x, y) in [(1, 22), (6, 23), (12, 23), (19, 23), (26, 22), (28, 21)] {
        s.rect(x, y, 2, 1, STONE2);
    }
    // Sparks and smoke above, unhulled.
    for (x, y, c) in [
        (10, 3, EMBER2),
        (16, 1, EMBER1),
        (20, 4, EMBER2),
        (14, 6, EMBER1),
    ] {
        glow.set(x + 1, y + 1, c);
    }
    for (x, y) in [(12, 0), (13, 1), (11, 2), (18, 2), (19, 1)] {
        glow.set(x + 1, y + 1, STONE1);
    }
    finish(s, Some(glow))
}

/// Toad Hall: a sturdy stone-and-timber longhouse under a thick turf roof.
fn toad_hall() -> Sprite {
    let mut s = canvas(BLook::ToadHall, 38);
    // Stone footing.
    s.rect(2, 30, 42, 8, STONE1);
    for y in [31, 34] {
        for x in (2..44).step_by(5) {
            let o = if y == 34 { 2 } else { 0 };
            s.set(x + o, y, STONE0);
            s.set(x + o, y + 1, STONE0);
            s.set(x + o + 1, y - 1, STONE2);
        }
    }
    s.rect(2, 37, 42, 1, STONE0);
    // Timber walls with posts.
    s.rect(4, 18, 38, 12, WOOD1);
    for x in [4, 13, 32, 41] {
        s.rect(x, 18, 2, 12, WOOD0);
    }
    s.rect(4, 18, 38, 1, WOOD2);
    s.dither_rect(4, 26, 38, 4, WOOD1, WOOD0);
    // The turf roof, deep and overhanging.
    for y in 2..20 {
        let inset = (20 - y) * 6 / 10;
        let x0 = -1 + inset / 2 + (y < 6) as i32 * 2;
        s.rect(x0.max(0), y, 46 - 2 * x0.max(0), 1, LEAF0);
    }
    s.ellipse(23, 6, 18, 5, LEAF0);
    s.ellipse(21, 5, 15, 3, LEAF1);
    s.dither_in(23, 10, 20, 6, LEAF0, LEAF1);
    s.rect(0, 18, 46, 2, LEAF0);
    s.dither_rect(0, 19, 46, 1, LEAF0, WOOD0);
    for x in (2..44).step_by(4) {
        s.set(x, 20, LEAF0);
    }
    // A ridge beam with carved toad-heads.
    s.rect(9, 1, 28, 2, WOOD0);
    s.rect(9, 1, 28, 1, WOOD2);
    for x in [8, 37] {
        s.rect(x - 1, 0, 3, 2, BTOAD1);
        s.set(x - 1, 0, EYE);
    }
    // A big arched door with warm light.
    s.ellipse(23, 26, 5, 5, WOOD0);
    s.rect(18, 26, 11, 12, WOOD0);
    s.ellipse(23, 27, 3, 4, GLOW);
    s.rect(20, 27, 7, 11, GLOW);
    s.rect(23, 24, 1, 14, COPPER1);
    // Round windows.
    for x in [9, 37] {
        s.ellipse(x, 23, 2, 2, WOOD0);
        s.ellipse(x, 23, 1, 1, GLOW);
    }
    // A shield and crossed spears above the door, and a banner.
    s.line(17, 13, 29, 19, POLE);
    s.line(29, 13, 17, 19, POLE);
    s.set(17, 13, IRON2);
    s.set(29, 13, IRON2);
    s.ellipse(23, 16, 2, 2, IRON1);
    s.set(22, 15, IRON2);
    s.set(23, 16, BAD);
    s.rect(42, 8, 1, 22, POLE);
    s.rect(43, 9, 3, 7, BAD);
    s.set(45, 16, BAD);
    s.set(43, 16, BAD);
    s.outlined()
}

/// The Shell Forge: a stone forge with a glowing mouth, an anvil, and plates on a rack.
fn shell_forge() -> Sprite {
    let mut s = canvas(BLook::ShellForge, 34);
    let mut glow = Sprite::new(s.w + 2, s.h + 2);
    // The chimney.
    s.rect(6, 0, 7, 14, STONE1);
    s.rect(6, 0, 2, 14, STONE2);
    s.rect(11, 0, 2, 14, STONE0);
    s.rect(5, 0, 9, 2, STONE0);
    for y in [4, 8] {
        s.rect(6, y, 7, 1, STONE0);
    }
    // The furnace: a squat dome of stone.
    s.ellipse(12, 24, 11, 10, STONE1);
    s.rect(1, 24, 23, 10, STONE1);
    shade_mass(&mut s, 12, 26, 11, 9, STONE1, STONE0, STONE2);
    for (x, y) in [(4, 20), (10, 17), (17, 21), (5, 28), (19, 27)] {
        s.rect(x, y, 3, 1, STONE0);
    }
    // Its fiery mouth.
    s.ellipse(12, 28, 5, 4, STONE0);
    s.ellipse(12, 29, 4, 3, EMBER0);
    s.rect(8, 29, 9, 4, EMBER0);
    s.ellipse(12, 30, 3, 2, EMBER1);
    s.rect(10, 31, 5, 2, EMBER2);
    s.rect(7, 33, 11, 1, STONE0);
    // The anvil on a block.
    s.rect(24, 22, 9, 2, IRON1);
    s.rect(24, 22, 9, 1, IRON2);
    s.rect(22, 23, 2, 1, IRON1);
    s.rect(26, 24, 5, 3, IRON0);
    s.rect(25, 27, 7, 6, WOOD0);
    s.rect(25, 27, 7, 1, WOOD1);
    s.rect(25, 27, 1, 6, WOOD1);
    plate(&mut s, 26, 18);
    // The rack of shell plates.
    s.rect(35, 6, 1, 27, WOOD1);
    s.rect(44, 6, 1, 27, WOOD1);
    s.rect(34, 6, 12, 2, WOOD2);
    s.rect(35, 19, 10, 1, WOOD0);
    for (x, y) in [(35, 9), (40, 9), (35, 21), (40, 21), (37, 27)] {
        plate(&mut s, x, y);
    }
    s.rect(34, 32, 12, 1, WOOD0);
    // Warm light on the ground and rising sparks, unhulled.
    for x in 5..20 {
        if x & 1 == 0 {
            glow.set(x + 1, 35, EMBER0);
        }
    }
    for (x, y, c) in [
        (9, -1, EMBER1),
        (11, 1, EMBER2),
        (28, 16, EMBER2),
        (8, 3, STONE1),
    ] {
        glow.set(x + 1, y + 1, c);
    }
    finish(s, Some(glow))
}

/// The Cauldron: an iron pot on three stones over a fire, its brew bubbling green.
fn cauldron() -> Sprite {
    let mut s = canvas(BLook::Cauldron, 28);
    let mut glow = Sprite::new(s.w + 2, s.h + 2);
    // Fire and stones beneath.
    for (x, y) in [(5, 25), (24, 25), (14, 26)] {
        s.ellipse(x, y, 3, 2, STONE1);
        s.set(x - 1, y - 1, STONE2);
        s.rect(x - 2, y + 1, 5, 1, STONE0);
    }
    s.rect(9, 22, 12, 4, EMBER0);
    for (x, h) in [(10, 3), (13, 4), (16, 3), (19, 2)] {
        s.rect(x, 25 - h, 1, h, EMBER1);
        s.set(x, 26 - h, EMBER2);
    }
    // The pot.
    s.ellipse(15, 15, 12, 9, IRON0);
    s.rect(3, 9, 25, 3, IRON0);
    shade_mass(&mut s, 15, 15, 12, 9, IRON0, INK, IRON1);
    s.ellipse(9, 13, 3, 3, IRON1);
    s.set(8, 12, IRON2);
    s.set(9, 12, IRON2);
    // The rim and the brew.
    s.ellipse(15, 8, 13, 3, IRON1);
    s.ellipse(15, 8, 11, 2, BREW0);
    s.rect(5, 7, 21, 1, BREW1);
    s.dither_rect(5, 8, 21, 2, BREW0, BREW1);
    for x in [2, 28] {
        s.rect(x, 8, 1, 5, IRON1);
    }
    // Little legs.
    s.rect(6, 22, 2, 3, IRON0);
    s.rect(23, 22, 2, 3, IRON0);
    // Bubbles that shimmer (spore colours cycle), and steam, unhulled.
    for (x, y, k) in [(9, 6, 0), (14, 5, 1), (20, 6, 2), (17, 3, 0), (11, 2, 1)] {
        glow.set(x + 1, y + 1, SPORE[k]);
    }
    for (x, y) in [(13, 0), (14, 1), (19, 0), (18, 1)] {
        glow.set(x + 1, y + 1, STONE2);
    }
    finish(s, Some(glow))
}

/// The Star Pond: still black water full of reflected stars, a stone lantern-shrine on its bank.
fn star_pond() -> Sprite {
    let mut s = canvas(BLook::StarPond, 50);
    // Bank stones around the pond.
    s.ellipse(23, 32, 23, 17, STONE1);
    shade_mass(&mut s, 23, 32, 23, 17, STONE1, STONE0, STONE2);
    s.ellipse(23, 33, 21, 15, POOL1);
    s.ellipse(23, 34, 19, 13, POOL0);
    for k in 0..34 {
        let h = hash(k, 21, 5);
        let x = 5 + (h % 38) as i32;
        let y = 22 + (h / 38 % 25) as i32;
        if s.get(x, y) == POOL0 {
            s.set(
                x,
                y,
                if h % 5 == 0 {
                    WGLINT[(h % 3) as usize]
                } else {
                    STAR
                },
            );
        }
    }
    // A few big stars with a twinkle cross.
    for (x, y) in [(14, 30), (31, 37), (22, 43), (36, 28)] {
        s.set(x, y, STAR);
        for (dx, dy) in [(-1, 0), (1, 0), (0, -1), (0, 1)] {
            s.set(x + dx, y + dy, POOL2);
        }
    }
    // Reed clumps on the near bank.
    reed_fan(&mut s, 3, 46, 9, 3);
    reed_fan(&mut s, 43, 45, 8, 4);
    // The shrine on the far bank: a stone lantern with a star burning inside.
    let x = 29;
    s.rect(x - 4, 18, 9, 2, STONE0);
    s.rect(x - 3, 17, 7, 1, STONE2);
    s.rect(x - 1, 12, 3, 5, STONE1);
    s.set(x - 1, 12, STONE2);
    s.rect(x - 3, 7, 7, 5, STONE1);
    s.rect(x - 3, 7, 1, 5, STONE2);
    s.rect(x + 3, 7, 1, 5, STONE0);
    s.rect(x - 1, 8, 3, 3, INK);
    s.set(x, 9, STAR);
    s.set(x - 1, 9, GLOW);
    s.set(x + 1, 9, GLOW);
    s.set(x, 8, GLOW);
    s.set(x, 10, GLOW);
    s.rect(x - 5, 5, 11, 2, STONE0);
    s.rect(x - 4, 4, 9, 1, STONE2);
    s.rect(x - 2, 3, 5, 1, STONE1);
    s.set(x, 2, STONE2);
    s.set(x, 1, STONE1);
    // Stepping stones down to the water, and an offering of dew.
    for (sx, sy) in [(21, 19), (17, 20)] {
        s.ellipse(sx, sy, 2, 1, STONE2);
        s.set(sx + 1, sy + 1, STONE0);
    }
    s.set(33, 17, DEW);
    s.set(34, 17, DEW_DK);
    s.outlined()
}

/// The newt house: a small bark lodge under a leaf roof.
fn newt_house() -> Sprite {
    let mut s = canvas(BLook::NewtHouse, 28);
    s.rect(4, 12, 22, 16, WOOD1);
    for x in (5..26).step_by(4) {
        s.rect(x, 12, 1, 16, WOOD0);
    }
    s.dither_rect(4, 23, 22, 5, WOOD1, WOOD0);
    s.rect(11, 17, 7, 11, WOOD0);
    s.rect(12, 18, 5, 10, GLOW);
    s.rect(14, 18, 1, 10, COPPER1);
    s.rect(6, 16, 3, 3, GLOW);
    s.rect(21, 16, 3, 3, GLOW);
    s.set(7, 17, COPPER1);
    s.set(22, 17, COPPER1);
    s.ellipse(15, 8, 14, 7, LEAF0);
    s.ellipse(15, 6, 13, 5, LEAF1);
    s.dither_in(15, 10, 14, 3, LEAF0, BULL0);
    s.dither_in(13, 4, 9, 2, LEAF1, LEAF2);
    s.set(15, 0, NEWT2);
    s.rect(14, 1, 3, 2, NEWT1);
    s.outlined()
}

/// The Spawning Pool: a big pool ringed with reeds, frogspawn in the shallows.
fn spawning_pool() -> Sprite {
    let mut s = canvas(BLook::SpawningPool, 50);
    s.ellipse(23, 34, 22, 15, EARTH1);
    shade_mass(&mut s, 23, 34, 22, 15, EARTH1, EARTH0, EARTH2);
    s.ellipse(23, 35, 20, 13, POOL1);
    s.ellipse(23, 36, 18, 11, POOL0);
    // Lighter green shallows near the bank.
    for y in 24..48 {
        for x in 3..44 {
            let (dx, dy) = (x - 23, y - 36);
            if s.get(x, y) == POOL0 && dx * dx * 121 + dy * dy * 324 > 230 * 121 && (x + y) & 1 == 0
            {
                s.set(x, y, BREW0);
            }
        }
    }
    for (x, y) in [(18, 33), (26, 40)] {
        s.rect(x, y, 4, 1, POOL1);
    }
    s.set(31, 34, WGLINT[0]);
    frogspawn(&mut s, 13, 36);
    frogspawn(&mut s, 30, 42);
    frogspawn(&mut s, 19, 44);
    // Lily pads, one in flower.
    for (x, y, f) in [(33, 30, true), (9, 41, false), (37, 45, false)] {
        lily_pad(&mut s, x, y, f);
    }
    // Reeds around the back and sides.
    for (x, b, h) in [
        (4, 31, 10),
        (11, 25, 12),
        (20, 23, 11),
        (29, 23, 13),
        (37, 26, 12),
        (43, 32, 9),
    ] {
        reed_fan(&mut s, x, b, h, x as u32);
    }
    reed_fan(&mut s, 2, 47, 8, 7);
    reed_fan(&mut s, 44, 48, 7, 8);
    s.outlined()
}

/// Daot's Grotto: a mage's cave in the rock, curtained, with a star banner and a crystal.
fn daots_grotto() -> Sprite {
    let mut s = canvas(BLook::DaotsGrotto, 38);
    let mut glow = Sprite::new(s.w + 2, s.h + 2);
    rock_mass(&mut s, 23, 26, 22, 18, 13);
    s.rect(1, 26, 45, 12, STONE1);
    shade_mass(&mut s, 23, 30, 22, 8, STONE1, STONE0, STONE2);
    // The arch: carved stones, a purple curtain drawn back, warm light within.
    s.ellipse(23, 27, 9, 10, STONE2);
    s.ellipse(23, 28, 7, 9, ROBE0);
    s.rect(16, 28, 15, 10, ROBE0);
    s.ellipse(23, 29, 5, 7, GLOW);
    s.rect(18, 29, 11, 9, GLOW);
    s.dither_rect(18, 33, 11, 5, GLOW, COPPER2);
    for (x, y) in [(16, 22), (18, 19), (23, 18), (28, 19), (30, 22)] {
        s.set(x, y, STONE0);
    }
    s.rect(16, 28, 3, 10, ROBE1);
    s.rect(28, 28, 3, 10, ROBE1);
    s.set(18, 30, ROBE0);
    s.set(28, 32, ROBE0);
    // Runes on the lintel.
    for (x, y) in [(19, 20), (22, 19), (25, 19), (28, 20)] {
        s.set(x, y, DEW);
    }
    // A crystal on the crown of the rock.
    s.rect(22, 4, 3, 6, DEW);
    s.rect(22, 4, 1, 6, TEXT);
    s.rect(24, 5, 1, 5, DEW_DK);
    s.set(23, 3, DEW);
    s.rect(19, 7, 2, 3, DEW_DK);
    s.set(19, 7, DEW);
    s.rect(26, 8, 2, 2, DEW);
    // A banner with a star on a pole.
    s.rect(38, 10, 1, 27, WOOD1);
    s.rect(39, 11, 5, 9, ROBE0);
    s.rect(40, 11, 3, 8, ROBE1);
    s.set(41, 14, STAR);
    s.set(40, 15, KEY);
    s.set(42, 15, KEY);
    s.set(41, 16, KEY);
    s.set(39, 20, ROBE0);
    s.set(43, 20, ROBE0);
    // Bottles by the door.
    for (x, c) in [(9, BREW1), (12, DEW), (34, HEART)] {
        s.rect(x, 34, 2, 3, c);
        s.set(x, 33, STONE2);
    }
    // Glints of the crystal, unhulled.
    for (x, y) in [(20, 1), (27, 2), (17, 5)] {
        glow.set(x + 1, y + 1, STAR);
    }
    finish(s, Some(glow))
}

/// The Olm's Cave: a black mouth going down into the deep, pale crystal teeth, a cold stream.
fn olms_cave() -> Sprite {
    let mut s = canvas(BLook::OlmsCave, 34);
    rock_mass(&mut s, 23, 24, 22, 16, 17);
    s.rect(1, 24, 45, 10, STONE1);
    shade_mass(&mut s, 23, 28, 22, 6, STONE1, STONE0, STONE2);
    s.dither_rect(0, 0, 46, 34, STONE2, STONE1);
    // The mouth, fading from dark stone into black.
    s.ellipse(23, 26, 13, 10, STONE0);
    s.rect(10, 26, 27, 8, STONE0);
    s.ellipse(23, 27, 11, 9, INK);
    s.rect(12, 27, 23, 7, INK);
    s.dither_in(23, 27, 12, 10, STONE0, INK);
    // Crystal teeth hanging from the lip and rising from the floor.
    for (x, l) in [(14, 3), (18, 4), (22, 2), (27, 4), (31, 3)] {
        s.rect(x, 18 + (x - 23).abs() / 4, 1, l, OLM1);
        s.set(x, 18 + (x - 23).abs() / 4, OLM2);
    }
    for (x, l) in [(16, 2), (29, 3), (25, 1)] {
        s.rect(x, 33 - l, 1, l, OLM0);
        s.set(x, 33 - l, OLM1);
    }
    // A cold stream running out over the stones.
    s.rect(20, 31, 6, 3, POOL1);
    s.rect(21, 32, 3, 2, POOL2);
    s.set(22, 33, DEW_DK);
    s.set(24, 31, DEW_DK);
    // Faint pale lights deep inside.
    s.set(21, 25, OLM0);
    s.set(26, 23, OLM0);
    s.set(24, 28, CAEC1);
    s.outlined()
}

/// A heron's nest: a great bowl of twigs on splayed stilts over the water. The lone nest is
/// smaller and leaner.
fn nest(lone: bool) -> Sprite {
    let h = if lone { 32 } else { 40 };
    let mut s = canvas(BLook::Nest, h);
    let (cx, top, rx, ry) = if lone { (15, 9, 9, 4) } else { (15, 10, 13, 6) };
    // Water around the stilts, with a reed or two.
    s.ellipse(15, h - 3, 14, 2, POOL1);
    s.ellipse(15, h - 3, 12, 1, POOL0);
    s.set(9, h - 3, WGLINT[0]);
    s.set(21, h - 2, WGLINT[1]);
    reed_fan(&mut s, 2, h - 3, 7, 5);
    // Stilts: splayed poles, lashed where they cross.
    let legs: &[(i32, i32)] = if lone {
        &[(13, 8), (17, 22)]
    } else {
        &[(10, 5), (14, 12), (17, 19), (20, 26)]
    };
    for &(x0, x1) in legs {
        s.line(x0, top + 2, x1, h - 4, TWIG0);
        s.set(x1, h - 3, POOL2);
        s.set(x1 + 1, h - 3, POOL2);
    }
    let knot = top + (h - top) / 3;
    s.rect(
        legs[0].0 - 2,
        knot,
        legs[legs.len() - 1].0 - legs[0].0 + 5,
        1,
        TWIG1,
    );
    // The bowl of twigs, sticks poking out all round.
    s.ellipse(cx, top + ry / 2, rx, ry, TWIG0);
    for y in 0..h {
        for x in 0..s.w {
            if s.get(x, y) == TWIG0 && hash(x, y, 31) % 3 == 0 {
                s.set(x, y, if hash(x, y, 7) % 2 == 0 { WOOD0 } else { TWIG1 });
            }
        }
    }
    for k in 0..10 {
        let hh = hash(k, 2, if lone { 4 } else { 3 });
        let side = if k % 2 == 0 { -1 } else { 1 };
        let y = top - 1 + (hh % (ry as u32 + 2)) as i32;
        let x = cx + side * (rx - 2 - (hh / 5 % 3) as i32);
        s.line(
            x,
            y,
            x + side * (3 + (hh / 17 % 2) as i32),
            y - 1 + (hh / 3 % 3) as i32,
            TWIG1,
        );
    }
    s.ellipse(cx, top - 1, rx - 3, 2, WOOD0);
    let eggs: &[(i32, i32)] = if lone {
        &[(13, 8), (17, 8)]
    } else {
        &[(10, 9), (15, 8), (20, 9)]
    };
    for &(x, y) in eggs {
        s.ellipse(x, y, 2, 1, EGG);
        s.set(x - 1, y - 1, TEXT);
    }
    // A shed feather.
    s.rect(cx + rx - 3, top + ry, 3, 1, HERON2);
    s.outlined()
}

/// Gorf's Stone: a tall standing stone carved with a frog's face, its runes glowing violet.
fn gorf_stone() -> Sprite {
    let mut s = canvas(BLook::GorfStone, 40);
    let mut glow = Sprite::new(s.w + 2, s.h + 2);
    s.rect(2, 8, 10, 30, STONE0);
    s.ellipse(7, 8, 5, 5, STONE0);
    s.rect(2, 8, 3, 30, STONE1);
    s.ellipse(6, 7, 3, 4, STONE1);
    s.rect(1, 36, 12, 2, STONE0);
    s.dither_rect(5, 10, 4, 26, STONE0, STONE1);
    // Bulging eyes at the crown, glowing.
    s.ellipse(4, 4, 2, 2, STONE1);
    s.ellipse(10, 4, 2, 2, STONE1);
    s.set(4, 4, WISP2);
    s.set(10, 4, WISP2);
    s.set(5, 4, WISP1);
    s.set(9, 4, WISP1);
    // A wide mouth and runes down the face.
    s.rect(3, 10, 9, 1, INK);
    s.set(2, 9, INK);
    s.set(12, 9, INK);
    for (y, c) in [
        (15, WISP1),
        (19, WISP2),
        (23, WISP1),
        (27, WISP0),
        (31, WISP1),
    ] {
        s.rect(5, y, 4, 1, c);
        s.set(4, y + 1, c);
        s.set(9, y + 1, c);
        s.set(7, y - 1, c);
    }
    // Moss at the foot.
    s.ellipse(7, 37, 6, 1, LEAF0);
    // Violet motes about it, unhulled.
    for (x, y, c) in [
        (0, 14, WISP0),
        (13, 20, WISP1),
        (1, 26, WISP1),
        (12, 31, WISP0),
        (11, 1, WISP0),
    ] {
        glow.set(x + 1, y + 1, c);
    }
    finish(s, Some(glow))
}

/// A leech pool: black still water, flecked with leeches, in a muddy bank with rotting reeds.
fn leech_pool() -> Sprite {
    let mut s = canvas(BLook::LeechPool, 28);
    s.ellipse(15, 17, 14, 10, EARTH0);
    s.ellipse(15, 17, 13, 9, EARTH1);
    s.dither_in(15, 17, 13, 9, EARTH1, EARTH0);
    s.ellipse(15, 18, 12, 8, POOL0);
    s.ellipse(14, 18, 10, 6, INK);
    s.dither_in(15, 18, 12, 8, POOL0, INK);
    // Leeches: dark red curls, one rising.
    for (x, y, flip) in [
        (8, 16, false),
        (15, 20, true),
        (20, 15, false),
        (11, 22, true),
        (22, 21, true),
        (16, 14, false),
    ] {
        let d = if flip { -1 } else { 1 };
        s.set(x, y, EMBER0);
        s.set(x + d, y, CAP0);
        s.set(x + 2 * d, y - 1, CAP0);
        s.set(x - d, y + 1, CAP0);
    }
    // Bubbles and an oily sheen.
    for (x, y) in [(10, 13), (19, 12), (24, 18), (6, 19)] {
        s.set(x, y, STONE2);
    }
    s.rect(12, 13, 4, 1, POOL1);
    s.rect(6, 21, 2, 1, POOL1);
    s.set(13, 12, METEOR0);
    // Rotting reeds, bent over.
    for (x, h, bend) in [(2, 9, 1), (4, 12, 2), (27, 10, -2), (25, 7, -1)] {
        for i in 0..h {
            let xx = x + if i > h * 2 / 3 { bend } else { 0 };
            s.set(xx, 18 - i, if i % 3 == 0 { WOOD0 } else { TWIG0 });
        }
    }
    s.outlined()
}

/// The crow roost: a dead tree, bare and grey, with a hoard of stolen glints in its fork.
fn crow_roost() -> Sprite {
    let mut s = canvas(BLook::CrowRoost, 48);
    // Trunk and roots.
    s.rect(12, 14, 6, 32, TWIG0);
    s.rect(12, 14, 2, 32, STONE1);
    s.rect(16, 14, 2, 32, WOOD0);
    for (x0, x1) in [(12, 5), (17, 25), (13, 9), (16, 21)] {
        s.line(x0, 40, x1, 47, TWIG0);
        s.line(x0, 41, x1, 47, WOOD0);
    }
    // Bare branches.
    for (x0, y0, x1, y1) in [
        (13, 18, 2, 8),
        (6, 12, 3, 3),
        (16, 16, 27, 6),
        (22, 10, 26, 1),
        (14, 14, 12, 2),
        (12, 6, 9, 0),
        (17, 20, 25, 16),
        (4, 10, 0, 10),
    ] {
        s.line(x0, y0, x1, y1, TWIG0);
        s.line(x0 + 1, y0, x1 + 1, y1, STONE1);
    }
    // A hollow knot.
    s.ellipse(15, 30, 1, 2, INK);
    // The hoard in the fork: twigs and stolen things that glint.
    s.ellipse(15, 13, 5, 2, TWIG1);
    s.rect(11, 13, 9, 1, TWIG0);
    for (x, c) in [(12, KEY), (14, DEW), (16, COPPER2), (18, EGG)] {
        s.set(x, 11, c);
    }
    s.set(15, 10, METEOR1);
    s.outlined()
}

/// Gorf's Hollow: a vast rotten stump with a gaping toothed maw full of violet light.
fn gorf_hollow() -> Sprite {
    let mut s = canvas(BLook::GorfHollow, 58);
    let mut glow = Sprite::new(s.w + 2, s.h + 2);
    // The stump: bulging sides, flaring at the foot.
    for y in 8..58 {
        let bulge = [0, 1, 2, 2, 1, 0, -1, -1, 0, 1][(y / 5 % 10) as usize];
        let flare = if y > 46 { (y - 46) * 3 / 2 } else { 0 };
        let hw = 22 + bulge + flare;
        for x in 31 - hw..=31 + hw {
            let col = hash(x / 2, 0, 88);
            let c = if (x - 31 + hw) < 3 {
                STONE0
            } else if 31 + hw - x < 3 {
                INK
            } else if col % 3 == 0 {
                WOOD0
            } else {
                GROBE0
            };
            s.set(x, y, c);
        }
    }
    for k in 0..30 {
        let h = hash(k, 3, 88);
        s.over(10 + (h % 42) as i32, 10 + (h / 42 % 40) as i32, STONE0);
    }
    // Its jagged broken top.
    for (x, h) in [
        (10, 6),
        (15, 10),
        (21, 4),
        (27, 12),
        (33, 7),
        (39, 11),
        (45, 5),
        (51, 8),
    ] {
        for i in 0..h {
            let w = 3 - i * 3 / h;
            s.rect(x - w / 2, 8 - i, w.max(1), 1, WOOD0);
        }
        s.set(x, 8 - h, STONE0);
    }
    s.ellipse(31, 9, 20, 3, INK);
    s.dither_in(31, 9, 20, 3, INK, GROBE1);
    // Twisted roots.
    for (x0, x1) in [(12, 1), (17, 6), (46, 58), (50, 61), (26, 21), (37, 42)] {
        s.line(x0, 50, x1, 57, WOOD0);
        s.line(x0, 51, x1, 57, GROBE0);
    }
    // The maw: a toothed hollow filling with violet light.
    s.ellipse(31, 42, 15, 14, INK);
    s.rect(16, 42, 31, 16, INK);
    s.ellipse(31, 45, 12, 11, WISP0);
    s.rect(19, 45, 25, 13, WISP0);
    s.ellipse(31, 49, 8, 8, WISP1);
    s.rect(23, 49, 17, 9, WISP1);
    s.ellipse(31, 54, 4, 4, WISP2);
    s.dither_in(31, 45, 12, 11, WISP0, INK);
    s.dither_in(31, 50, 9, 8, WISP1, WISP0);
    for x in (18..46).step_by(4) {
        let l = 3 + (hash(x, 1, 2) % 3) as i32;
        let y0 = 29 + ((x - 31) * (x - 31)) / 18;
        for i in 0..l {
            s.set(x + 1, y0 + i, CREAM);
            if i < l - 1 {
                s.set(x, y0 + i, HUSK0);
            }
        }
    }
    // Lower teeth rising from the maw's floor.
    for x in [21, 27, 36, 42] {
        s.set(x, 56, CREAM);
        s.set(x, 55, CREAM);
        s.set(x + 1, 56, HUSK0);
    }
    // Carved eyes above, burning.
    for x in [21, 41] {
        s.ellipse(x, 22, 3, 2, INK);
        s.ellipse(x, 22, 2, 1, WISP1);
        s.set(x, 22, WISP2);
    }
    s.line(17, 18, 24, 20, INK);
    s.line(45, 18, 38, 20, INK);
    // Hanging moss and skull stones.
    for x in [12, 17, 45, 50] {
        for i in 0..(4 + x % 5) {
            s.set(x, 10 + i, GORF1);
        }
    }
    for (x, y) in [(9, 55), (54, 54)] {
        s.rect(x - 1, y - 1, 3, 1, HUSK1);
        s.rect(x - 2, y, 5, 1, HUSK1);
        s.set(x - 1, y, INK);
        s.set(x + 1, y, INK);
        s.rect(x - 1, y + 1, 3, 1, HUSK0);
        s.set(x, y + 1, INK);
    }
    // Violet light spilling out onto the ground, and motes, unhulled.
    for x in 14..50 {
        if x & 1 == 0 {
            glow.set(x + 1, 59, WISP0);
        }
    }
    for (x, y, c) in [
        (31, 4, WISP2),
        (24, 1, WISP1),
        (38, 0, WISP0),
        (28, -1, WISP1),
        (5, 30, WISP0),
        (57, 24, WISP0),
    ] {
        glow.set(x + 1, y + 1, c);
    }
    finish(s, Some(glow))
}

// ---- building sites and ruins ------------------------------------------------------------------

/// A building site: a dug pit with spoil heaps, corner stakes and a line, timber and a spade.
fn site(w: i32, h: i32) -> Sprite {
    let (pw, ph) = (w * TILE - 2, h * TILE - 2);
    let mut s = Sprite::new(pw, ph);
    let top = (ph / 3).min(10);
    // The dug ground, its corners rounded, the pit darker inside with furrows.
    s.rect(1, top + 1, pw - 2, ph - top - 1, EARTH1);
    s.rect(2, top, pw - 4, ph - top, EARTH1);
    s.rect(4, top + 3, pw - 8, ph - top - 6, EARTH0);
    for y in top + 4..ph - 3 {
        for x in 4..pw - 4 {
            if (y - top) % 3 == 0 && (x + y) & 1 == 0 {
                s.set(x, y, EARTH1);
            }
        }
    }
    for y in top..ph {
        for x in 1..pw - 1 {
            if s.get(x, y) == EARTH1 && hash(x, y, (w * 7 + h) as u32) % 7 == 0 {
                s.set(x, y, EARTH2);
            }
        }
    }
    s.rect(4, top + 3, pw - 8, 1, INK);
    // Spoil heaps at two corners.
    for (x, y) in [(4, top + 1), (pw - 6, ph - 4)] {
        s.ellipse(x, y, 3, 2, EARTH2);
        s.rect(x - 2, y + 1, 5, 1, EARTH1);
    }
    // Stakes at the corners and a line between them.
    let corners = [
        (1, top - 4),
        (pw - 2, top - 4),
        (1, ph - 6),
        (pw - 2, ph - 6),
    ];
    let (a, b, c, d) = (corners[0], corners[1], corners[2], corners[3]);
    for (p, q) in [(a, b), (a, c), (b, d)] {
        s.line(p.0, p.1 + 1, q.0, q.1 + 1, TEXT_DIM);
    }
    for &(x, y) in &corners {
        s.rect(x, y, 1, 6, POLE);
        s.set(x, y, FLAG);
    }
    // A pile of timber and a spade stuck in the earth.
    let (tx, ty) = (pw / 2 - 4, ph - 3);
    for i in 0..3 {
        s.rect(tx + i, ty - i, 8, 1, if i % 2 == 0 { WOOD2 } else { WOOD1 });
        s.set(tx + i + 7, ty - i, WOOD0);
    }
    if w >= 2 {
        s.line(pw - 10, top + 2, pw - 8, top + 8, WOOD1);
        s.rect(pw - 9, top + 8, 3, 2, IRON1);
        s.set(pw - 9, top + 8, IRON2);
    }
    s.outlined()
}

/// Rubble: a heap of broken stone and charred timber across the footprint, one wall stub left.
fn ruin(w: i32, h: i32) -> Sprite {
    let (pw, ph) = (w * TILE - 2, h * TILE - 2);
    let mut s = Sprite::new(pw, ph);
    let top = ph / 3;
    // Ash and a low heap.
    s.ellipse(pw / 2, ph - 3, pw / 2 - 1, (ph - top) / 2, SHADE);
    s.ellipse(pw / 2, ph - 2, pw / 2 - 3, (ph - top) / 3, STONE0);
    // The wall stub, its top broken.
    let sx = pw / 4;
    let sw = (pw / 4).clamp(4, 10);
    let sh = (ph - top).min(20);
    s.rect(sx, ph - sh - 2, sw, sh, STONE1);
    for y in (ph - sh - 2..ph - 2).step_by(3) {
        s.rect(sx, y, sw, 1, STONE0);
        s.set(sx + (y % 4), y + 1, STONE0);
    }
    s.rect(sx, ph - sh - 2, 1, sh, STONE2);
    for i in 0..sw {
        let cut = (hash(i, w, 9) % 4) as i32;
        s.rect(sx + i, ph - sh - 2, 1, cut, 0);
    }
    let n = 4 + w * h * 3;
    for k in 0..n {
        let hh = hash(k, w * 5 + h, 41);
        let x = 2 + (hh % (pw as u32 - 4)) as i32;
        let y = top + 4 + (hh / 101 % (ph - top - 5).max(1) as u32) as i32;
        let r = 1 + (hh / 7 % 2) as i32;
        match hh % 5 {
            0 | 1 | 2 => {
                s.ellipse(x, y, r + 1, r, STONE1);
                s.set(x - r, y - r + 1, STONE2);
                s.set(x - r + 1, y - r, STONE2);
                s.rect(x - r, y + r, 2 * r + 1, 1, STONE0);
            }
            3 => {
                let l = 3 + r * 2;
                s.line(x - l, y + 1, x + l, y - 2, WOOD0);
                s.line(x - l, y + 2, x + l, y - 1, INK);
                s.set(x + l, y - 2, EMBER0);
            }
            _ => {
                s.rect(x - 1, y, 3, 2, STONE0);
                s.set(x, y, STONE2);
            }
        }
    }
    // A charred beam leaning on the stub.
    s.line(sx + sw, ph - sh, sx + sw + 8.min(pw / 3), ph - 3, WOOD0);
    s.line(
        sx + sw + 1,
        ph - sh,
        sx + sw + 9.min(pw / 3 + 1),
        ph - 3,
        INK,
    );
    s.outlined()
}

// ---- spots -------------------------------------------------------------------------------------

/// A growing spot on its 16 x 16 square, bottom-aligned: stage 0 bare, 1 sprouting, 2 ripening,
/// 3 ripe.
fn spot(kind: SpotLook, stage: u8) -> Sprite {
    let mut s = Sprite::new(14, 14);
    let mut glow = Sprite::new(16, 16);
    // The bed of turned earth.
    s.ellipse(7, 12, 6, 1, EARTH1);
    s.rect(2, 13, 11, 1, EARTH0);
    s.set(4, 12, EARTH2);
    s.set(9, 11, EARTH2);
    s.set(11, 12, EARTH0);
    match (kind, stage) {
        (_, 0) => {
            s.set(6, 11, EARTH0);
            s.set(8, 12, EARTH0);
        }
        (SpotLook::Fern, st) => fern_spot(&mut s, st),
        (SpotLook::RedFungus, 1) => {
            s.rect(5, 10, 2, 2, MOREL0);
            s.rect(9, 11, 1, 1, MOREL0);
        }
        (SpotLook::RedFungus, 2) => {
            morel(&mut s, 4, 12, 3);
            morel(&mut s, 8, 12, 4);
        }
        (SpotLook::RedFungus, _) => {
            morel(&mut s, 2, 12, 4);
            morel(&mut s, 6, 12, 7);
            morel(&mut s, 10, 12, 5);
        }
        (SpotLook::WhiteFungus, 1) => {
            s.rect(5, 11, 2, 1, IRON1);
            s.set(9, 11, IRON1);
        }
        (SpotLook::WhiteFungus, 2) => {
            ironcap(&mut s, 5, 12, 1);
            ironcap(&mut s, 9, 12, 2);
        }
        (SpotLook::WhiteFungus, _) => {
            ironcap(&mut s, 3, 12, 2);
            ironcap(&mut s, 10, 12, 2);
            ironcap(&mut s, 7, 10, 3);
        }
        (SpotLook::HybridFungus, 1) => {
            s.rect(6, 10, 2, 2, CREAM);
            s.rect(6, 9, 2, 1, CAP1);
        }
        (SpotLook::HybridFungus, 2) => {
            s.rect(6, 8, 2, 4, CREAM);
            s.rect(4, 7, 6, 1, CAP1);
            s.rect(5, 6, 4, 1, CAP1);
            s.set(5, 6, CREAM);
            s.set(8, 7, CREAM);
            s.rect(4, 8, 6, 1, CAP0);
            s.rect(10, 10, 1, 2, CREAM);
            s.rect(9, 9, 3, 1, CAP1);
        }
        (SpotLook::HybridFungus, _) => {
            // A tall spotted toadstool and a small one.
            s.rect(6, 6, 3, 7, CREAM);
            s.rect(8, 6, 1, 7, HUSK1);
            s.rect(5, 8, 5, 1, HUSK1);
            s.ellipse(7, 4, 6, 3, CAP1);
            s.rect(1, 6, 13, 1, CAP0);
            s.dither_in(5, 3, 4, 2, CAP1, CAP2);
            for (x, y) in [(4, 3), (8, 2), (11, 4), (6, 5), (2, 5)] {
                s.set(x, y, CREAM);
            }
            s.set(9, 3, CREAM);
            toadstool(&mut s, 12, 12, 1);
        }
        (SpotLook::Puffball, 1) => {
            s.ellipse(7, 11, 1, 1, HUSK1);
        }
        (SpotLook::Puffball, 2) => {
            s.ellipse(7, 10, 3, 2, HUSK1);
            s.rect(5, 12, 5, 1, HUSK0);
            s.set(6, 9, CREAM);
            s.ellipse(11, 11, 1, 1, HUSK1);
        }
        (SpotLook::Puffball, _) => {
            // Ripe: swollen and glowing, breathing out spores.
            s.ellipse(7, 8, 5, 4, HUSK1);
            s.rect(3, 11, 9, 2, HUSK0);
            s.ellipse(6, 7, 2, 1, CREAM);
            s.dither_in(8, 10, 4, 2, HUSK1, HUSK0);
            for (x, y, k) in [(7, 4, 2), (5, 9, 1), (9, 7, 1), (10, 10, 0), (4, 6, 0)] {
                s.set(x, y, SPORE[k]);
            }
            s.ellipse(12, 11, 1, 1, HUSK1);
            s.set(12, 10, SPORE[2]);
            for (x, y, k) in [(5, 1, 0), (9, 0, 1), (11, 3, 2), (3, 4, 1), (13, 6, 0)] {
                glow.set(x + 1, y + 1, SPORE[k]);
            }
        }
    }
    let mut o = s.outlined();
    o.blit(&glow, 0, 0);
    o
}

/// A dew fern: a fiddlehead, a young fern, then a full one beaded with dew.
fn fern_spot(s: &mut Sprite, stage: u8) {
    match stage {
        1 => {
            s.rect(7, 9, 1, 3, LEAF1);
            s.set(8, 8, LEAF1);
            s.set(8, 9, LEAF2);
            s.set(5, 11, LEAF1);
            s.set(4, 10, LEAF2);
        }
        2 => {
            for (x0, y0, x1, y1) in [(7, 12, 7, 5), (7, 11, 3, 6), (7, 11, 11, 6)] {
                s.line(x0, y0, x1, y1, LEAF0);
            }
            for (x, y) in [
                (6, 7),
                (8, 7),
                (5, 8),
                (9, 8),
                (4, 7),
                (10, 7),
                (7, 5),
                (6, 9),
                (8, 9),
            ] {
                s.set(x, y, LEAF1);
            }
            s.set(7, 4, LEAF2);
        }
        _ => {
            for (x0, y0, x1, y1) in [
                (7, 12, 7, 1),
                (7, 12, 1, 4),
                (7, 12, 13, 4),
                (7, 12, 0, 9),
                (7, 12, 13, 9),
            ] {
                s.line(x0, y0, x1, y1, LEAF0);
            }
            // Leaflets along each frond.
            let src: Vec<u8> = s.px.clone();
            for y in 0..13 {
                for x in 0..14 {
                    if src[(y * 14 + x) as usize] == LEAF0 && (x + y) % 2 == 0 {
                        s.set(x - 1, y, LEAF1);
                        s.set(x + 1, y - 1, LEAF2);
                    }
                }
            }
            // Beads of dew on the tips.
            for (x, y) in [(7, 1), (1, 4), (13, 4), (0, 9), (13, 9), (4, 6), (10, 6)] {
                s.set(x, y, DEW);
            }
            s.set(7, 0, TEXT);
            s.set(1, 3, TEXT);
            s.set(13, 3, TEXT);
        }
    }
}

// ---- terrain tiles -----------------------------------------------------------------------------

/// Smooth value noise, 0..=255, that wraps every 16 px (cells of `cell` px; 16 % cell == 0).
/// The lattice on the tile's edges depends only on the low byte of the seed's family
/// (`seed >> 10`, the terrain type), so every variant of a terrain joins every other; a seed
/// below 1024 has free edges (small blobs that never line up into a grid).
fn vnoise(x: i32, y: i32, cell: i32, seed: u32) -> i32 {
    let n = 16 / cell;
    let (cx, cy) = (x.div_euclid(cell), y.div_euclid(cell));
    let (fx, fy) = (x.rem_euclid(cell), y.rem_euclid(cell));
    let family = seed >> 10;
    let v = |i: i32, j: i32| {
        let (i, j) = (i.rem_euclid(n), j.rem_euclid(n));
        let sd = if (i == 0 || j == 0) && family > 0 {
            family
        } else {
            seed
        };
        (hash(i, j, sd) % 256) as i32
    };
    let top = v(cx, cy) * (cell - fx) + v(cx + 1, cy) * fx;
    let bot = v(cx, cy + 1) * (cell - fx) + v(cx + 1, cy + 1) * fx;
    (top * (cell - fy) + bot * fy) / (cell * cell)
}

/// A 2 x 2 Bayer offset, -24..=24, for dithered thresholds.
fn bayer(x: i32, y: i32) -> i32 {
    [0, 2, 3, 1][((x & 1) + 2 * (y & 1)) as usize] * 16 - 24
}

/// Pick from a ramp by a dithered level: `cuts` are the thresholds between the colours.
fn ramp(level: i32, x: i32, y: i32, cols: &[u8], cuts: &[i32]) -> u8 {
    let l = level + bayer(x, y);
    let mut i = 0;
    while i < cuts.len() && l >= cuts[i] {
        i += 1;
    }
    cols[i]
}

/// A terrain tile, exactly 16 x 16, in background colours only.
fn tile(t: TileLook, v: u8) -> Sprite {
    let mut s = Sprite::new(TILE, TILE);
    let seed = ((t as u32 + 1) << 10) + v as u32 * 17 + 5;
    let n = |x: i32, y: i32| hash(x, y, seed);
    let v = v as i32;
    match t {
        TileLook::Water | TileLook::Reeds => {
            water(&mut s, seed);
            if t == TileLook::Reeds {
                reeds(&mut s, seed);
                // A firefly among the stalks: dark by day, blinking at night.
                let h = n(6, 6);
                s.set(
                    2 + (h % 12) as i32,
                    4 + (h / 12 % 7) as i32,
                    FIREFLY[(h % 3) as usize],
                );
            }
        }
        TileLook::Deep => {
            for y in 0..16 {
                for x in 0..16 {
                    let l = vnoise(x, y, 8, seed);
                    s.set(x, y, ramp(l, x, y, &[DEEP0, DEEP1, DEEP1], &[96, 200]));
                }
            }
            for k in 0..3 {
                let h = n(k, 2);
                let (x, y) = (1 + (h % 11) as i32, 1 + k * 5 + (h / 11 % 3) as i32);
                s.rect(x, y, 2 + (h / 23 % 3) as i32, 1, DEEP2);
            }
            if v % 3 == 0 {
                let h = n(5, 5);
                s.set(
                    2 + (h % 12) as i32,
                    2 + (h / 12 % 12) as i32,
                    WGLINT[(h % 3) as usize],
                );
            }
        }
        TileLook::Moss | TileLook::Fern | TileLook::Puddle => {
            moss(&mut s, seed, t == TileLook::Fern);
            match t {
                TileLook::Fern => {
                    let fronds: &[(i32, i32, bool, bool)] = match v % 8 {
                        0 => &[(4, 4, false, true)],
                        1 => &[(0, 1, true, true), (7, 8, false, false)],
                        2 => &[(6, 2, true, true)],
                        3 => &[(1, 8, false, false), (8, 0, true, true)],
                        4 => &[(3, 6, false, true), (10, 1, true, false)],
                        5 => &[(5, 8, true, true)],
                        6 => &[(0, 4, false, true), (9, 6, true, true)],
                        _ => &[(2, 0, false, false), (7, 7, true, true)],
                    };
                    for &(x, y, f, big) in fronds {
                        fern_frond(&mut s, x, y, f, big);
                    }
                    if v % 3 == 1 {
                        let h = n(6, 6);
                        s.set(
                            1 + (h % 14) as i32,
                            1 + (h / 14 % 14) as i32,
                            FIREFLY[(h % 3) as usize],
                        );
                    }
                }
                TileLook::Puddle => puddle(&mut s, seed),
                _ => match v % 8 {
                    5 => {
                        let h = n(1, 1);
                        let (x, y) = (2 + (h % 10) as i32, 2 + (h / 10 % 10) as i32);
                        s.set(x, y, SPORE[0]);
                        s.set(x + 2, y + 1, SPORE[1]);
                        s.set(x + 1, y + 3, SPORE[2]);
                    }
                    6 => {
                        let h = n(2, 2);
                        let (x, y) = (3 + (h % 9) as i32, 4 + (h / 9 % 8) as i32);
                        s.set(x, y + 1, BCREAM);
                        s.set(x - 1, y, BCAP1);
                        s.set(x, y, BCAP1);
                        s.set(x + 1, y, BCAP0);
                        s.set(x, y - 1, BCAP1);
                    }
                    7 => {
                        let h = n(3, 3);
                        let (x, y) = (2 + (h % 9) as i32, 4 + (h / 9 % 9) as i32);
                        for (dx, dy) in [(0, 0), (1, -1), (2, 0), (3, -2), (4, 0), (2, -1)] {
                            s.set(x + dx, y + dy, if dy < -1 { MOSS4 } else { MOSS3 });
                        }
                    }
                    _ => {}
                },
            }
        }
        TileLook::Mud => {
            for y in 0..16 {
                for x in 0..16 {
                    let l = vnoise(x, y, 4, seed) * 2 / 3 + vnoise(x, y, 8, seed + 1) / 3;
                    s.set(x, y, ramp(l, x, y, &[MUD0, MUD1, MUD2], &[90, 175]));
                    if n(x, y) % 17 == 0 {
                        s.set(x, y, MUD0);
                    }
                }
            }
            // A wet sheen on the tops of lumps.
            for y in 1..16 {
                for x in 0..16 {
                    if s.get(x, y) == MUD2 && s.get(x, y - 1) != MUD2 && n(x, y) % 3 == 0 {
                        s.set(x, y - 1, MUD3);
                    }
                }
            }
            // Pockets of standing water.
            for k in 0..(1 + (v % 2)) {
                let h = n(9, k);
                let (x, y) = (2 + (h % 9) as i32, 2 + (h / 9 % 11) as i32);
                let w = 2 + (h / 99 % 3) as i32;
                s.rect(x, y, w, 1, WATER0);
                s.rect(x + 1, y + 1, w, 1, WATER0);
                s.set(x + 1, y, WATER1);
                s.set(x + w, y + 1, WGLINT[(h % 3) as usize]);
                s.rect(x, y + 2, w + 1, 1, MUD0);
            }
            if v % 4 == 3 {
                let h = n(4, 4);
                let (x, y) = (3 + (h % 10) as i32, 5 + (h / 10 % 9) as i32);
                s.set(x, y, REED1);
                s.set(x, y - 1, REED1);
                s.set(x + 1, y - 2, REED2);
                s.set(x - 1, y - 1, REED2);
            }
        }
        TileLook::Path => {
            for y in 0..16 {
                for x in 0..16 {
                    let l = vnoise(x, y, 4, seed & 1023);
                    let mut c = ramp(l, x, y, &[PATH1, PATH1, PATH2], &[60, 150]);
                    let h = n(x, y);
                    if h % 7 == 0 {
                        c = PATH0;
                    } else if h % 19 == 0 {
                        c = PATH3;
                    }
                    s.set(x, y, c);
                }
            }
            // Pebbles with a shadow beneath.
            for k in 0..(1 + v % 3) {
                let h = n(k, 6);
                let (x, y) = (1 + (h % 13) as i32, 1 + (h / 13 % 13) as i32);
                s.set(x, y, ROCK3);
                if h % 2 == 0 {
                    s.set(x + 1, y, ROCK2);
                    s.set(x + 1, y + 1, PATH0);
                }
                s.set(x, y + 1, PATH0);
            }
            match v % 4 {
                1 => {
                    // A dry crack.
                    let h = n(3, 3);
                    let (x, y) = (3 + (h % 8) as i32, 3 + (h / 8 % 8) as i32);
                    s.line(x, y, x + 3, y + 1, PATH0);
                    s.line(x + 3, y + 1, x + 4, y + 3, PATH0);
                    s.set(x + 4, y + 1, PATH0);
                }
                3 => {
                    // A webbed footprint.
                    let h = n(8, 8);
                    let (x, y) = (3 + (h % 8) as i32, 4 + (h / 8 % 7) as i32);
                    for (dx, dy) in [(0, 0), (2, -1), (4, 0), (2, 1), (2, 2)] {
                        s.set(x + dx, y + dy, PATH0);
                    }
                }
                _ => {}
            }
        }
        TileLook::Sand => {
            for y in 0..16 {
                for x in 0..16 {
                    let l = vnoise(x, y, 8, seed);
                    let mut c = ramp(l, x, y, &[SAND0, SAND1, SAND1], &[40, 210]);
                    let h = n(x, y);
                    if h % 9 == 0 {
                        c = SAND0;
                    } else if h % 13 == 0 {
                        c = SAND2;
                    }
                    s.set(x, y, c);
                }
            }
            // Wind ripples: short arcs, shadow under a lit crest.
            for k in 0..3 {
                let h = n(k, 8);
                let y = 2 + k * 5 + (h % 3) as i32;
                let x0 = (h / 3 % 8) as i32;
                let len = 5 + (h / 24 % 4) as i32;
                for i in 0..len {
                    let up = (i > 0 && i < len - 1) as i32;
                    s.set(x0 + i, y - up, SAND0);
                    s.set(x0 + i, y - up - 1, SAND2);
                }
            }
            match v % 8 {
                6 => {
                    s.set(11, 11, BCREAM);
                    s.set(12, 11, BCREAM);
                    s.set(11, 12, SAND0);
                    s.set(12, 12, SAND0);
                }
                7 => {
                    s.set(4, 10, ROCK3);
                    s.set(5, 10, ROCK2);
                    s.set(4, 11, SAND0);
                    s.set(5, 11, SAND0);
                }
                _ => {}
            }
        }
        TileLook::Rock => rock_tile(&mut s, seed, v),
        TileLook::Cursed => {
            for y in 0..16 {
                for x in 0..16 {
                    let l = vnoise(x, y, 8, seed);
                    s.set(x, y, ramp(l, x, y, &[CURSE0, CURSE0, CURSE1], &[40, 150]));
                }
            }
            // A network of cracks, edged with violet.
            let (mut x, mut y) = (2 + (n(0, 0) % 12) as i32, 0);
            while y < 16 {
                let h = n(x, y);
                s.set(x, y, CURSE2);
                s.set(x + 1, y, CURSE3);
                y += 1;
                x += (h % 3) as i32 - 1;
                if h % 6 == 0 {
                    let (mut bx, mut by) = (x, y);
                    let dir = if h % 12 == 0 { 1 } else { -1 };
                    for _ in 0..4 + (h % 3) as i32 {
                        bx += dir;
                        by += (hash(bx, by, seed) % 2) as i32;
                        s.set(bx, by, CURSE2);
                        s.set(bx, by - 1, CURSE3);
                    }
                }
            }
            let (mut x, mut y) = (0, 3 + (n(1, 1) % 10) as i32);
            while x < 16 && v % 2 == 0 {
                let h = n(x, y + 40);
                s.set(x, y, CURSE2);
                s.set(x, y - 1, CURSE3);
                x += 1;
                y += (h % 3) as i32 - 1;
            }
            let h = n(7, 7);
            let (gx, gy) = (2 + (h % 12) as i32, 2 + (h / 12 % 12) as i32);
            s.set(gx, gy, CURSE_GLOW);
            if v % 3 == 0 {
                s.set((gx + 7) % 16, (gy + 5) % 16, CURSE_GLOW);
            }
        }
    }
    s
}

/// Open water: dark with soft deeper patches, short ripples and a glint or two.
fn water(s: &mut Sprite, seed: u32) {
    for y in 0..16 {
        for x in 0..16 {
            let l = vnoise(x, y, 4, seed & 1023);
            s.set(
                x,
                y,
                if l + bayer(x, y) / 2 > 196 {
                    WATER1
                } else {
                    WATER0
                },
            );
        }
    }
    for k in 0..3 {
        let h = hash(k, 1, seed);
        let (x, y) = (1 + (h % 11) as i32, (k * 5 + 1 + (h / 11 % 3) as i32) % 16);
        let l = 2 + (h / 41 % 3) as i32;
        s.rect(x, y, l, 1, WATER1);
        if h % 3 == 0 {
            s.set(x + l, y, WATER2);
        }
    }
    for k in 0..2 {
        let h = hash(k, 9, seed);
        s.set(
            1 + (h % 14) as i32,
            1 + (h / 14 % 14) as i32,
            WGLINT[(h % 3) as usize],
        );
    }
}

/// The damp moss ground: soft mottling, dark specks and lit leaflets.
fn moss(s: &mut Sprite, seed: u32, ferny: bool) {
    for y in 0..16 {
        for x in 0..16 {
            let l = vnoise(x, y, 8, seed) * 3 / 5 + vnoise(x, y, 4, seed + 7) * 2 / 5;
            let l = if ferny { l - 30 } else { l };
            let mut c = ramp(l, x, y, &[MOSS0, MOSS1, MOSS2], &[70, 160]);
            let h = hash(x, y, seed);
            if h % 11 == 0 {
                c = MOSS0;
            } else if c == MOSS2 && h % 7 == 0 {
                c = MOSS3;
            }
            s.set(x, y, c);
        }
    }
    // Little leaflets catching the light: a lit pixel over a dark one.
    for k in 0..4 {
        let h = hash(k, 77, seed);
        let (x, y) = ((h % 15) as i32, 1 + (h / 15 % 14) as i32);
        s.set(x, y, MOSS3);
        s.set(x, y + 1, MOSS0);
    }
}

/// A fern frond painted into a tile (the demo's fern, or a smaller one), facing either way.
fn fern_frond(s: &mut Sprite, x: i32, y: i32, flip: bool, big: bool) {
    let rows: &[&str] = if big {
        &[
            "...f.....",
            "f..f..f..",
            ".f.F.f...",
            "..fFf..f.",
            "f..F..f..",
            ".ffFff...",
            "...F.....",
        ]
    } else {
        &["..f..", "f.F.f", ".fFf.", "..F.."]
    };
    let w = rows[0].len() as i32 - 1;
    for (dy, r) in rows.iter().enumerate() {
        for (dx, ch) in r.chars().enumerate() {
            let dx = if flip { w - dx as i32 } else { dx as i32 };
            match ch {
                'f' => s.set(x + dx, y + dy as i32, FERN1),
                'F' => s.set(x + dx, y + dy as i32, FERN0),
                _ => {}
            }
        }
    }
}

/// A puddle on damp ground: two or three overlapping pools in a muddy rim.
fn puddle(s: &mut Sprite, seed: u32) {
    let k = 2 + (hash(0, 0, seed) % 2) as i32;
    let blobs: Vec<(i32, i32, i32, i32)> = (0..k)
        .map(|i| {
            let h = hash(i, 2, seed);
            (
                4 + (h % 8) as i32,
                4 + (h / 8 % 8) as i32,
                2 + (h / 64 % 3) as i32,
                1 + (h / 192 % 2) as i32,
            )
        })
        .collect();
    for &(cx, cy, rx, ry) in &blobs {
        s.ellipse(cx, cy, rx + 1, ry + 1, MUD2);
    }
    for &(cx, cy, rx, ry) in &blobs {
        s.dither_in(cx, cy, rx + 1, ry + 1, MUD2, MUD1);
    }
    for &(cx, cy, rx, ry) in &blobs {
        s.ellipse(cx, cy, rx, ry, WATER0);
    }
    for (i, &(cx, cy, rx, _)) in blobs.iter().enumerate() {
        s.rect(cx - rx + 1, cy - 1, rx, 1, WATER1);
        if i == 0 {
            s.set(cx + rx - 1, cy, WGLINT[(seed % 3) as usize]);
        }
    }
}

/// Reed stalks and cattails standing in water: a dark clump at the base, uneven heights.
fn reeds(s: &mut Sprite, seed: u32) {
    let count = 9;
    let mut stalks: Vec<(i32, i32, i32)> = (0..count)
        .map(|k| {
            let h = hash(k, 3, seed);
            (
                (k * 16 / count + (h % 2) as i32) % 16,
                6 + (h / 2 % 9) as i32,
                (h / 18 % 3) as i32 - 1,
            )
        })
        .collect();
    stalks.sort_by_key(|s| -s.1);
    for (i, &(x, h, lean)) in stalks.iter().enumerate() {
        for k in 0..h {
            let xx = x + if k > h / 2 { lean } else { 0 };
            let y = 15 - k;
            let c = if k >= h - 2 {
                REED2
            } else if i % 2 == 0 {
                REED0
            } else {
                REED1
            };
            s.set(xx, y, c);
        }
        if i % 4 == 1 {
            let xx = x + lean;
            let y = 15 - h;
            s.rect(xx, y + 1, 1, 3, CATTAIL1);
            s.set(xx, y + 3, CATTAIL0);
            s.set(xx, y, REED2);
        }
    }
    for x in 0..16 {
        if hash(x, 15, seed) % 2 == 0 {
            s.set(x, 15, REED0);
        }
    }
}

/// Craggy rock: cobbles lit from the upper left, cracks between them (a wrapped Voronoi).
fn rock_tile(s: &mut Sprite, seed: u32, v: i32) {
    // Seeds on the edges are shared by every variant, so neighbours' cracks meet; the interior
    // seeds vary.
    let mut pts: Vec<(i32, i32, u32)> = vec![(0, 0, 0), (8, 0, 1), (0, 8, 1), (8, 8, 0)];
    pts.truncate(3);
    for k in 0..3 {
        let h = hash(k, 5, seed);
        pts.push((4 + (h % 9) as i32, 4 + (h / 9 % 9) as i32, h / 256));
    }
    let wrap = |d: i32| {
        let d = d.rem_euclid(16);
        if d > 8 {
            d - 16
        } else {
            d
        }
    };
    for y in 0..16 {
        for x in 0..16 {
            let mut best = (i32::MAX, i32::MAX, 0usize, 0, 0);
            for (i, &(px, py, _)) in pts.iter().enumerate() {
                let (dx, dy) = (wrap(x - px), wrap(y - py));
                let d = dx * dx + dy * dy;
                if d < best.0 {
                    best = (d, best.0, i, dx, dy);
                } else if d < best.1 {
                    best.1 = d;
                }
            }
            let (d1, d2, i, dx, dy) = best;
            let tone = pts[i].2 % 2;
            let lit = -(dx + dy);
            let c = if d2 - d1 <= 3 {
                ROCK0
            } else if lit > 3 {
                if tone == 0 {
                    ROCK3
                } else {
                    ROCK2
                }
            } else if lit > -1 {
                if tone == 0 {
                    ROCK2
                } else {
                    ROCK1
                }
            } else if lit > -4 {
                ROCK1
            } else {
                ramp(64, x, y, &[ROCK0, ROCK1], &[64])
            };
            s.set(x, y, c);
        }
    }
    if v % 4 == 2 {
        let h = hash(9, 9, seed);
        let (x, y) = ((h % 12) as i32, 2 + (h / 12 % 12) as i32);
        s.set(x, y, MOSS2);
        s.set(x + 1, y, MOSS3);
        s.set(x + 2, y, MOSS2);
        s.set(x + 1, y + 1, MOSS2);
    }
}

// ---- decorations -------------------------------------------------------------------------------

/// A decoration in background colours, hulled in the background ink.
fn deco(d: Deco) -> Sprite {
    let mut glow: Option<Sprite> = None;
    let s = match d {
        Deco::BigTree => {
            let (s, g) = big_tree();
            glow = Some(g);
            s
        }
        Deco::Tree => tree(),
        Deco::Stump => stump(),
        Deco::Stone => stone(),
        Deco::LeyStone => ley_stone(),
        Deco::Lily => grid(
            &[
                "..LLLLL...",
                ".LLlLLLLL.",
                "LLLLlLLHLL",
                "LLlLLLlLLL",
                ".LLLLLLLL.",
                "...LLLL...",
            ],
            &[('L', BLEAF1), ('l', BLEAF0), ('H', BLEAF2)],
        ),
        Deco::ReedClump => reed_clump(),
        Deco::Mushroom => wild_mushrooms(),
        Deco::Log => fallen_log(),
    };
    // Reeds are too fine for a hull: the ground shows between the stalks. Everything else gets a
    // hull of the background ink.
    let mut o = if d == Deco::ReedClump {
        let mut t = Sprite::new(s.w + 2, s.h + 2);
        t.blit(&s, 1, 1);
        t
    } else {
        s.outlined_with(BGINK)
    };
    if d == Deco::Lily {
        // A flower on the pad.
        o.set(6, 2, BPINK);
        o.set(7, 1, BPINK);
        o.set(8, 2, BPINK);
        o.set(7, 2, BCREAM);
    }
    if let Some(g) = glow {
        o.blit(&g, 1, 1);
    }
    o
}

/// A canopy of dark leaves (the demo's tree painting, in sprite space).
fn canopy(s: &mut Sprite, cx: i32, cy: i32, r: i32) {
    s.ellipse(cx, cy + 2, r, r - 1, TREE0);
    s.ellipse(cx - 1, cy, r - 1, r - 2, TREE1);
    s.dither_in(cx, cy + 2, r - 1, r - 2, TREE1, TREE0);
    s.ellipse(cx - 2, cy - 2, r - 3, r - 4, TREE1);
    s.ellipse(cx - 3, cy - 3, (r - 5).max(1), (r - 6).max(1), TREE2);
    s.dither_in(cx - 3, cy - 3, r - 4, r - 5, TREE1, TREE2);
    s.ellipse(cx - 4, cy - 5, 2, 1, TREE3);
}

/// An old-growth tree, about 32 x 48: a broad lumpy crown, hanging moss, glowing shelf fungus,
/// fireflies, and a trunk flaring into roots.
fn big_tree() -> (Sprite, Sprite) {
    let mut s = Sprite::new(32, 48);
    let mut glow = Sprite::new(34, 50);
    // Roots and trunk.
    for (x0, x1) in [(12, 2), (13, 7), (19, 29), (18, 24), (15, 14)] {
        s.line(x0, 37, x1, 47, BARK1);
        s.line(x0 + 1, 37, x1 + 1, 47, BARK0);
    }
    s.rect(11, 20, 10, 24, BARK1);
    s.ellipse(16, 44, 8, 3, BARK1);
    s.rect(11, 20, 3, 24, BARK2);
    s.rect(18, 20, 3, 24, BARK0);
    for (x, y, l) in [
        (14, 25, 5),
        (16, 31, 4),
        (13, 36, 3),
        (17, 39, 4),
        (15, 22, 2),
    ] {
        s.rect(x, y, 1, l, BARK0);
    }
    s.ellipse(15, 32, 1, 2, BGINK);
    s.set(15, 30, BARK0);
    // Shelf fungus on the trunk.
    for (x, y) in [(20, 29), (21, 33)] {
        s.rect(x, y, 3, 1, BCREAM);
        s.set(x + 1, y + 1, BCREAM);
    }
    // The crown: a broad dark mass of lumps, lit from the upper left.
    s.ellipse(16, 13, 15, 10, TREE0);
    for (cx, cy, r) in [
        (7, 12, 7),
        (24, 11, 7),
        (15, 6, 8),
        (11, 17, 6),
        (21, 18, 6),
        (16, 13, 7),
    ] {
        canopy(&mut s, cx, cy, r);
    }
    // Dark underside where the crown meets the trunk.
    s.dither_in(16, 21, 12, 2, TREE1, TREE0);
    // Hanging moss.
    for (x, y, l) in [
        (4, 18, 5),
        (9, 22, 4),
        (25, 19, 6),
        (28, 15, 4),
        (21, 23, 3),
        (13, 22, 3),
    ] {
        for i in 0..l {
            s.set(x, y + i, if i % 2 == 0 { MOSS3 } else { MOSS2 });
        }
    }
    // Fireflies in the dark under the crown.
    for (k, (x, y)) in [(6, 16), (25, 14), (17, 21)].iter().enumerate() {
        s.set(*x, *y, FIREFLY[k]);
    }
    for (x, y, k) in [(23, 28, 0), (24, 32, 1), (21, 27, 2)] {
        glow.set(x + 1, y + 1, SPORE[k]);
    }
    (s, glow)
}

/// A smaller marsh tree.
fn tree() -> Sprite {
    let mut s = Sprite::new(20, 28);
    s.rect(8, 14, 4, 14, BARK1);
    s.rect(8, 14, 1, 14, BARK2);
    s.rect(11, 14, 1, 14, BARK0);
    s.line(8, 25, 5, 27, BARK1);
    s.line(11, 25, 14, 27, BARK0);
    canopy(&mut s, 10, 8, 9);
    canopy(&mut s, 7, 11, 5);
    for (x, y, l) in [(4, 15, 3), (15, 14, 4)] {
        for i in 0..l {
            s.set(x, y + i, if i % 2 == 0 { MOSS3 } else { MOSS2 });
        }
    }
    s.set(13, 13, FIREFLY[1]);
    s
}

fn stump() -> Sprite {
    let mut s = Sprite::new(14, 10);
    s.rect(2, 3, 10, 6, BARK1);
    s.rect(2, 3, 2, 6, BARK2);
    s.rect(10, 3, 2, 6, BARK0);
    s.line(1, 9, 3, 7, BARK1);
    s.line(12, 9, 10, 7, BARK0);
    s.rect(0, 9, 14, 1, BARK0);
    s.ellipse(7, 3, 5, 2, BARK2);
    s.ellipse(7, 3, 3, 1, BARK1);
    s.set(7, 3, BARK2);
    s.rect(3, 2, 3, 1, MOSS2);
    s.set(4, 1, MOSS3);
    s.rect(4, 5, 1, 3, BARK0);
    s.set(12, 6, BCAP1);
    s.set(13, 6, BCAP1);
    s.set(12, 7, BCREAM);
    s
}

fn stone() -> Sprite {
    let mut s = Sprite::new(11, 7);
    s.ellipse(5, 4, 5, 3, ROCK1);
    s.rect(0, 4, 11, 3, ROCK1);
    s.ellipse(4, 3, 3, 2, ROCK2);
    s.set(2, 2, ROCK3);
    s.set(3, 2, ROCK3);
    s.rect(1, 6, 10, 1, ROCK0);
    s.rect(7, 4, 3, 2, ROCK0);
    s.ellipse(5, 1, 3, 1, MOSS2);
    s.set(4, 0, MOSS3);
    s.set(6, 1, MOSS3);
    s
}

fn ley_stone() -> Sprite {
    let mut s = Sprite::new(8, 17);
    s.rect(1, 3, 6, 13, ROCK1);
    s.ellipse(4, 3, 3, 3, ROCK1);
    s.rect(1, 3, 2, 13, ROCK2);
    s.rect(6, 3, 1, 13, ROCK0);
    s.set(2, 1, ROCK3);
    s.set(3, 0, ROCK2);
    // Runes that glow whatever the hour.
    for (x, y, c) in [
        (4, 4, LEY1),
        (3, 5, LEY0),
        (5, 5, LEY0),
        (4, 7, LEY1),
        (4, 8, LEY0),
        (3, 10, LEY1),
        (5, 10, LEY1),
        (4, 11, LEY0),
        (4, 13, LEY1),
    ] {
        s.set(x, y, c);
    }
    s.rect(0, 15, 8, 2, MOSS2);
    s.set(1, 14, MOSS3);
    s.set(6, 14, MOSS2);
    s
}

fn reed_clump() -> Sprite {
    let mut s = Sprite::new(13, 18);
    // Stalks rising from a spread base, back ones dark, front ones lit, leaning apart.
    let stalks: [(i32, i32, i32, u8); 7] = [
        (4, -1, 15, REED0),
        (8, 2, 16, REED0),
        (2, -2, 9, REED1),
        (4, -3, 12, REED1),
        (6, 0, 17, REED1),
        (7, 3, 11, REED1),
        (9, 3, 13, REED1),
    ];
    for (i, &(bx, lean, h, c)) in stalks.iter().enumerate() {
        for k in 0..h {
            let x = bx + lean * k / h;
            let col = if k >= h - 2 && i >= 2 { REED2 } else { c };
            s.set(x, 17 - k, col);
        }
        if h >= 15 {
            let x = bx + lean;
            s.rect(x, 17 - h, 1, 3, CATTAIL1);
            s.set(x, 19 - h, CATTAIL0);
            s.set(x, 16 - h, REED2);
        }
    }
    // The clump's dark root, and fireflies resting on the stalks.
    s.rect(3, 16, 7, 2, REED0);
    s.set(6, 15, REED0);
    s.set(4, 9, FIREFLY[0]);
    s.set(9, 8, FIREFLY[2]);
    s
}

fn wild_mushrooms() -> Sprite {
    let mut s = Sprite::new(9, 8);
    for (x, y, r) in [(2, 7, 2), (6, 7, 1), (5, 5, 2)] {
        s.rect(x, y - r, 1, r + 1, BCREAM);
        s.ellipse(x, y - r - 1, r, (r / 2).max(1), BCAP1);
        s.set(x - r + 1, y - r - 1, BCREAM);
        if r > 1 {
            s.rect(x - r, y - r, 2 * r + 1, 1, BCAP0);
        }
    }
    s.set(7, 7, SPORE[1]);
    s
}

fn fallen_log() -> Sprite {
    let mut s = Sprite::new(24, 9);
    s.rect(2, 2, 19, 7, BARK1);
    s.rect(2, 2, 19, 2, BARK2);
    s.rect(2, 7, 19, 2, BARK0);
    s.ellipse(2, 5, 2, 3, BARK1);
    for (x, l) in [(5, 6), (13, 5)] {
        s.rect(x, 5, l, 1, BARK0);
    }
    s.ellipse(21, 5, 2, 4, BARK2);
    s.ellipse(21, 5, 1, 2, BARK1);
    s.set(21, 5, BARK0);
    s.rect(6, 1, 8, 1, MOSS2);
    s.rect(7, 0, 4, 1, MOSS3);
    for (dx, dy) in [(0, 0), (-1, -1), (1, -1), (0, -2)] {
        s.set(16 + dx, 1 + dy, FERN1);
    }
    s
}

// ---- icons -------------------------------------------------------------------------------------

fn icon(rows: &[&str], key: &[(char, u8)]) -> Sprite {
    grid(rows, key).outlined()
}

/// A 7 x 7 item icon.
fn item(i: ItemLook) -> Sprite {
    use ItemLook::*;
    let orb = [
        "..mmm..", ".mhmmm.", "mhhmmms", "mhmmmms", "mmmmmss", ".mmmss.", "..sss..",
    ];
    let shovel = [
        "......h", ".....h.", "....h..", "...h...", ".bBh...", "bBBb...", "bbb....",
    ];
    let spear = [
        "....tTT", ".....tT", "....p.t", "...p...", "..p....", ".p.....", "p......",
    ];
    let chanfron = [
        ".mmmmm.", "mhhhhhm", "mhkhkhm", ".mhhhm.", ".mhvhm.", "..mhm..", "...m...",
    ];
    match i {
        Dew => icon(
            &[
                "...d...", "..ddd..", ".dwddd.", "dwddddd", "ddddddD", ".dddDD.", "..DDD..",
            ],
            &[('d', DEW), ('D', DEW_DK), ('w', TEXT)],
        ),
        Fungiwood => icon(
            &[
                ".....ll", "....lLL", "...lLLd", "..lLLd.", ".lLLd..", "lLLd...", "Ldd....",
            ],
            &[('l', CREAM), ('L', WOOD2), ('d', WOOD1)],
        ),
        CapIngot => icon(
            &[
                ".......", "..hhhh.", ".hhhhhm", "mmmmmmd", "mmmmmmd", ".ddddd.", ".......",
            ],
            &[('h', IRON2), ('m', IRON1), ('d', IRON0)],
        ),
        SpottedCap => icon(
            &[
                "..rrr..", ".rwrrr.", "rrrrwrr", "RRRRRRR", "..ccc..", "..ccC..", ".cccC..",
            ],
            &[
                ('r', CAP1),
                ('R', CAP0),
                ('w', CREAM),
                ('c', CREAM),
                ('C', HUSK0),
            ],
        ),
        GlowSpores => icon(
            &[
                "...a...", ".b...c.", "...b...", ".a.c.a.", "..bab..", ".cabca.", "..bcb..",
            ],
            &[('a', SPORE[0]), ('b', SPORE[1]), ('c', SPORE[2])],
        ),
        SunOrb => icon(&orb, &[('m', EMBER2), ('h', GLOW), ('s', COPPER1)]),
        ClargentOrb => icon(&orb, &[('m', IRON2), ('h', TEXT), ('s', IRON1)]),
        MeteorOrb => icon(&orb, &[('m', METEOR1), ('h', PEARL), ('s', METEOR0)]),
        WoodenShovel => icon(&shovel, &[('h', WOOD1), ('b', WOOD2), ('B', CREAM)]),
        IronShovel => icon(&shovel, &[('h', WOOD1), ('b', IRON1), ('B', IRON2)]),
        VenomwoodShovel => icon(&shovel, &[('h', WOOD1), ('b', VENOM0), ('B', VENOM1)]),
        Slingshot => icon(
            &[
                ".h...h.", ".hrrrh.", ".h...h.", "..h.h..", "...h...", "...h...", "...H...",
            ],
            &[('h', WOOD2), ('H', WOOD1), ('r', BAD)],
        ),
        Catapult => icon(
            &[
                ".vV....", ".vvi...", "...i...", "....i..", "..h.hi.", "hhhhhhh", ".w...w.",
            ],
            &[
                ('v', VENOM0),
                ('V', VENOM1),
                ('i', IRON1),
                ('h', WOOD2),
                ('w', IRON0),
            ],
        ),
        IronSpear => icon(&spear, &[('p', POLE), ('t', IRON1), ('T', IRON2)]),
        VenomwoodSpear => icon(&spear, &[('p', POLE), ('t', VENOM0), ('T', VENOM1)]),
        VenomironSpear => icon(&spear, &[('p', VENOM0), ('t', IRON1), ('T', VENOM1)]),
        Chanfron => icon(
            &chanfron,
            &[('m', IRON1), ('h', IRON2), ('k', INK), ('v', IRON1)],
        ),
        VenomironChanfron => icon(
            &chanfron,
            &[('m', VENOM0), ('h', IRON2), ('k', INK), ('v', VENOM1)],
        ),
        Peytral => icon(
            &[
                ".......", "mm...mm", "mhhhhhm", "mhhmhhm", ".mhhhm.", "..mmm..", ".......",
            ],
            &[('m', IRON1), ('h', IRON2)],
        ),
        Croupiere => icon(
            &[
                ".......", "..mmm..", ".mhhhm.", "mhhkhhm", "mhhhhhm", "m.....m", ".......",
            ],
            &[('m', IRON1), ('h', IRON2), ('k', IRON0)],
        ),
        DewBead => icon(
            &[
                "c.....c", ".c...c.", "..ddd..", ".dwddd.", ".ddddD.", ".dddDD.", "..DDD..",
            ],
            &[('c', TWIG1), ('d', DEW), ('D', DEW_DK), ('w', TEXT)],
        ),
        GlowCharm => icon(
            &[
                "t.....t", ".t...t.", "..tit..", ".iaGai.", ".iGgGi.", ".iaGai.", "..iii..",
            ],
            &[
                ('t', TWIG1),
                ('i', COPPER1),
                ('a', COPPER2),
                ('G', SPORE[1]),
                ('g', SPORE[2]),
            ],
        ),
        StarPearl => icon(
            &[
                "...s...", "..ppp..", ".ppwpp.", "sppppps", ".pppPp.", "..pPP..", "...s...",
            ],
            &[('s', STAR), ('p', PEARL), ('w', TEXT), ('P', OLM1)],
        ),
    }
}

/// A 5 x 5 condition icon.
fn cond(c: CondLook) -> Sprite {
    use CondLook::*;
    let drop = ["..d..", ".ddd.", "ddwdd", "ddddD", ".dDD."];
    match c {
        Dry => icon(&drop, &[('d', DEW), ('D', DEW_DK), ('w', TEXT)]),
        Leeched => icon(&drop, &[('d', HEART), ('D', CAP0), ('w', PEARL)]),
        Poisoned => icon(
            &[".vvv.", "vkvkv", "vvvvv", ".vkv.", ".v.v."],
            &[('v', VENOM1), ('k', INK)],
        ),
        Blinded => icon(
            &["r...r", ".www.", "wkkkw", ".www.", "r...r"],
            &[('w', TEXT), ('k', INK), ('r', BAD)],
        ),
        Bolstered => icon(
            &["..y..", ".yyy.", "yyyyy", "..y..", "..y.."],
            &[('y', KEY)],
        ),
        Regenerating => icon(
            &["..g..", "..g..", "ggGgg", "..g..", "..g.."],
            &[('g', GOOD), ('G', HILITE)],
        ),
        Altered => icon(
            &[".aaa.", "a...a", "a.A.a", "a.AA.", ".a..."],
            &[('a', WISP1), ('A', WISP2)],
        ),
        Dreaming => icon(
            &["..mm.", ".mm..", ".m...", ".mm..", "..mm."],
            &[('m', KEY)],
        ),
        Frightened => icon(
            &["..w..", "..w..", "..w..", ".....", "..w.."],
            &[('w', WARN)],
        ),
    }
}

/// An 11 x 11 UI icon: a 9 x 9 symbol inside its ink hull.
fn ui(u: UiLook) -> Sprite {
    use UiLook::*;
    let (rows, key): ([&str; 9], &[(char, u8)]) = match u {
        Pause => (
            [
                ".........",
                "..ww.ww..",
                "..ww.ww..",
                "..ww.ww..",
                "..ww.ww..",
                "..ww.ww..",
                "..ww.ww..",
                "..dd.dd..",
                ".........",
            ],
            &[('w', TEXT), ('d', TEXT_DIM)],
        ),
        Play => (
            [
                ".........",
                "..w......",
                "..ww.....",
                "..www....",
                "..wwww...",
                "..wwwd...",
                "..wwd....",
                "..wd.....",
                ".........",
            ],
            &[('w', TEXT), ('d', TEXT_DIM)],
        ),
        Fast => (
            [
                ".........",
                ".w...w...",
                ".ww..ww..",
                ".www.www.",
                ".wwwwwwww",
                ".wwwdwwwd",
                ".wwd.wwd.",
                ".wd..wd..",
                ".........",
            ],
            &[('w', TEXT), ('d', TEXT_DIM)],
        ),
        Magic => (
            [
                "....y....",
                "....y....",
                "...yYy...",
                "yyyYwYyyy",
                "...yYy...",
                "....y..a.",
                "....y.aAa",
                "a......a.",
                ".........",
            ],
            &[
                ('y', KEY),
                ('Y', GLOW),
                ('w', TEXT),
                ('a', WISP1),
                ('A', WISP2),
            ],
        ),
        Screens => (
            [
                ".........",
                ".ddddd...",
                ".dbbbd...",
                ".dbwwwww.",
                ".ddwbbbw.",
                "...wbbbw.",
                "...wbbbw.",
                "...wwwww.",
                ".........",
            ],
            &[('w', TEXT), ('d', TEXT_DIM), ('b', BTN_ON)],
        ),
        Attack => (
            [
                "t.......t",
                ".t.....t.",
                "..p...p..",
                "...p.p...",
                "....p....",
                "...p.p...",
                "..p...p..",
                ".p.....p.",
                "p.......p",
            ],
            &[('t', IRON2), ('p', POLE)],
        ),
        Explore => (
            [
                "....r....",
                "...rrr...",
                "...rRr...",
                ".d.rRr.d.",
                "ddddkdddd",
                ".d.wWw.d.",
                "...wWw...",
                "...www...",
                "....w....",
            ],
            &[
                ('r', BAD),
                ('R', HEART),
                ('w', TEXT),
                ('W', TEXT_DIM),
                ('d', TEXT_DIM),
                ('k', INK),
            ],
        ),
        Guard => (
            [
                ".mmmmmmm.",
                "mhhhhhhhm",
                "mhhhbhhhm",
                "mhhbbbhhm",
                "mhhhbhhhm",
                ".mhhhhhm.",
                "..mhhhm..",
                "...mhm...",
                "....m....",
            ],
            &[('m', IRON1), ('h', IRON2), ('b', BAD)],
        ),
        Fetch => (
            [
                "...a.....",
                "..aaa....",
                ".a.a.....",
                "...a.tt..",
                "....tTTt.",
                "...tTTTTt",
                "...TTTTTT",
                "...tTTTTt",
                "....tttt.",
            ],
            &[('a', HILITE), ('t', WOOD1), ('T', WOOD2)],
        ),
        RainCall => (
            [
                "...ww....",
                ".wwWWww..",
                "wWWWWWWw.",
                "dddddddd.",
                ".........",
                ".b..b..b.",
                "b..b..b..",
                ".........",
                "..b..b...",
            ],
            &[('w', TEXT), ('W', TEXT), ('d', TEXT_DIM), ('b', DEW)],
        ),
        OldSong => (
            [
                "...yyyyy.",
                "...yyyyy.",
                "...y...y.",
                "...y...y.",
                "...y...y.",
                ".yyy.yyy.",
                "yyyyyyyy.",
                "yyy.yyy..",
                ".........",
            ],
            &[('y', KEY)],
        ),
        Starlight => (
            [
                "....s....",
                "....s....",
                "...sSs...",
                "ssSSWSSss",
                ".sSSSSSs.",
                "..sSSSs..",
                "..sS.Ss..",
                ".sS...Ss.",
                ".s.....s.",
            ],
            &[('s', KEY), ('S', STAR), ('W', TEXT)],
        ),
        LeyLight => (
            [
                "...sss...",
                "..sSSSs..",
                "..sSLSs..",
                "..sLSLs..",
                "..sSLSs..",
                "..sSSSs..",
                "..sSLSs..",
                "..sSSSs..",
                ".ggggggg.",
            ],
            &[('s', STONE1), ('S', STONE2), ('L', LEY1), ('g', LEAF1)],
        ),
        DeepCalling => (
            [
                "..ppppp..",
                ".p.....p.",
                "p..ppp..p",
                "p.p...p.p",
                "p.p.k.p.p",
                "p.p..pp.p",
                "p..pp...p",
                ".p......p",
                "..pppppp.",
            ],
            &[('p', OLM1), ('k', OLM2)],
        ),
    };
    icon(&rows, key)
}

// ---- odds and ends -----------------------------------------------------------------------------

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

fn flag() -> Sprite {
    grid(
        &[
            "pFFFFF", "pFFFFf", "pFFFff", "pFFFFf", "p.....", "p.....", "p.....", "p.....",
        ],
        &[('p', POLE), ('F', FLAG), ('f', FLAG_DK)],
    )
}

/// A fallen amphibian's empty skin: pale, crumpled, see-through (a dither of two pales).
fn husk() -> Sprite {
    let mut s = grid(
        &[
            "......hhh...",
            "..hhhhHkHh..",
            ".hHHHHHHHHh.",
            "hHhHHHHhHHHh",
            ".hh.hhhh.hh.",
            "hh..h..h..hh",
        ],
        &[('h', HUSK0), ('H', HUSK1), ('k', INK)],
    );
    s.dither_rect(0, 0, 12, 6, HUSK1, HUSK0);
    s.outlined()
}

/// Spawn in a little pool: 0 eggs, 1 tadpole, 2 froglet.
fn eggs(stage: u8) -> Sprite {
    let mut s = Sprite::new(14, 8);
    s.ellipse(7, 4, 7, 3, POOL1);
    s.ellipse(7, 5, 6, 2, POOL0);
    s.set(11, 3, WGLINT[0]);
    match stage {
        0 => {
            for (x, y) in [(4, 4), (6, 4), (5, 5), (7, 5), (6, 3), (8, 4), (3, 5)] {
                s.set(x, y, EGG);
            }
            for (x, y) in [(4, 4), (7, 5), (6, 3)] {
                s.set(x, y, INK);
            }
        }
        1 => {
            s.rect(5, 4, 2, 2, INK);
            s.set(4, 4, SAL1);
            s.rect(7, 5, 3, 1, SAL1);
            s.set(10, 4, SAL1);
            s.set(9, 3, EGG);
            s.set(3, 6, EGG);
        }
        _ => {
            s.ellipse(9, 4, 2, 1, LEAF1);
            s = {
                let mut t = Sprite::new(14, 9);
                t.blit(&s, 0, 1);
                t
            };
            // A froglet on a lily pad.
            s.rect(3, 2, 4, 2, TFROG1);
            s.set(5, 1, TFROG1);
            s.set(6, 1, EYE);
            s.set(6, 2, INK);
            s.set(2, 4, TFROG0);
            s.set(6, 4, TFROG0);
            s.rect(3, 4, 3, 1, TFROG2);
        }
    }
    s.outlined()
}

/// A thick curve (integer quadratic Bezier) of `w` px.
fn curve(s: &mut Sprite, p0: (i32, i32), p1: (i32, i32), p2: (i32, i32), w: i32, c: u8) {
    let n = 32;
    for t in 0..=n {
        let (a, b, d) = ((n - t) * (n - t), 2 * (n - t) * t, t * t);
        let x = (a * p0.0 + b * p1.0 + d * p2.0) / (n * n);
        let y = (a * p0.1 + b * p1.1 + d * p2.1) / (n * n);
        s.rect(x - w / 2, y, w, 1, c);
    }
}

/// A clump of reeds: a few stalks fanning from one root, cattails on the tall ones.
fn reed_fan(s: &mut Sprite, x: i32, base: i32, h: i32, seed: u32) {
    let stalks = [(-3, 6), (-1, 9), (0, 10), (2, 8), (3, 5)];
    for (i, &(lean, frac)) in stalks.iter().enumerate() {
        let hh = h * frac / 10 - (hash(i as i32, x, seed) % 2) as i32;
        for k in 0..hh {
            let xx = x + lean * k / h.max(1);
            let c = if k >= hh - 1 {
                LEAF2
            } else if (i as i32 + k) % 4 == 0 {
                LEAF0
            } else {
                LEAF1
            };
            s.set(xx, base - k, c);
        }
        if frac >= 9 || (frac == 8 && hash(x, 1, seed) % 2 == 0) {
            let xx = x + lean * hh / h.max(1);
            s.rect(xx, base - hh - 2, 1, 3, WOOD1);
            s.set(xx, base - hh, WOOD0);
            s.set(xx, base - hh - 3, LEAF1);
        }
    }
}

/// A clump of frogspawn: a jelly of pale eggs, each with a dark speck.
fn frogspawn(s: &mut Sprite, cx: i32, cy: i32) {
    s.ellipse(cx, cy, 3, 1, POOL2);
    for (dx, dy) in [(-2, 0), (0, 0), (2, 0), (-1, -1), (1, -1), (-1, 1), (1, 1)] {
        s.set(cx + dx, cy + dy, EGG);
    }
    for (dx, dy) in [(-1, 0), (1, 0), (0, -1), (0, 1)] {
        s.set(cx + dx, cy + dy, INK);
    }
}

/// A shell plate: a rounded plate with ridged scutes.
fn plate(s: &mut Sprite, x: i32, y: i32) {
    s.ellipse(x + 2, y + 2, 3, 2, IRON1);
    s.rect(x, y + 1, 5, 2, IRON1);
    s.set(x, y + 1, IRON2);
    s.set(x + 1, y, IRON2);
    s.set(x + 2, y + 2, IRON0);
    s.set(x + 1, y + 3, IRON0);
    s.set(x + 3, y + 3, IRON0);
    s.set(x + 2, y + 1, IRON2);
}

/// A lily pad on foreground water, its notch toward the viewer.
fn lily_pad(s: &mut Sprite, x: i32, y: i32, flower: bool) {
    s.rect(x - 2, y - 1, 5, 1, LEAF1);
    s.rect(x - 3, y, 7, 1, LEAF1);
    s.rect(x - 2, y + 1, 5, 1, LEAF0);
    s.set(x, y + 1, 0);
    s.set(x, y, LEAF0);
    s.set(x - 2, y - 1, LEAF2);
    if flower {
        s.set(x + 1, y - 2, HEART);
        s.set(x + 2, y - 1, HEART);
        s.set(x + 1, y - 1, TEXT);
    }
}

// ---- tests -------------------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn palette_is_dac_exact() {
        for t in [0, 37, 200, 1234] {
            for dark in 0..4 {
                for rain in [false, true] {
                    for dry in [false, true] {
                        let p = palette(t, Sky { dark, rain, dry });
                        for c in p.iter() {
                            assert_eq!(c & 0x03_03_03, 0);
                        }
                    }
                }
            }
        }
        for i in 0..=255u8 {
            assert_eq!(rgb(i) & 0x03_03_03, 0);
        }
    }

    #[test]
    fn existing_palette_indices_are_unchanged() {
        assert_eq!(rgb(INK), 0x080c10);
        assert_eq!(rgb(MOSS1), 0x15301f & 0xFCFCFC);
        assert_eq!(rgb(GHOST_NO), 0x8a3030 & 0xFCFCFC);
        assert_eq!(GHOST_NO, 88);
        assert_eq!(BGINK, 89);
    }

    #[test]
    fn the_sky_moves_only_the_background() {
        let day = palette(0, Sky::default());
        let night = palette(
            0,
            Sky {
                dark: 2,
                rain: true,
                dry: false,
            },
        );
        for i in 0..=255u8 {
            match layer(i) {
                Layer::Foreground | Layer::Glow => assert_eq!(day[i as usize], night[i as usize]),
                Layer::Background if (i as usize) < RGB.len() && rgb(i) > 0x101010 => {
                    assert_ne!(day[i as usize], night[i as usize], "index {i} darkens")
                }
                _ => {}
            }
        }
    }

    fn colours<'a>(sprites: impl Iterator<Item = &'a Sprite>) -> Vec<u8> {
        let mut v: Vec<u8> = sprites.flat_map(|s| s.px.iter().copied()).collect();
        v.sort();
        v.dedup();
        v
    }

    #[test]
    fn tiles_and_decorations_use_background_colours_and_the_rest_foreground() {
        let a = Art::new();
        let bg = colours(
            TileLook::ALL
                .iter()
                .flat_map(|&t| (0..TILE_VARIANTS).map(move |v| (t, v)))
                .map(|(t, v)| a.tile(t, v))
                .chain(Deco::ALL.iter().map(|&d| a.deco(d))),
        );
        for c in bg {
            assert!(
                c == 0 || layer(c) != Layer::Foreground,
                "background uses index {c}"
            );
        }
        let mut fg: Vec<&Sprite> = vec![];
        for look in Look::ALL {
            for age in Age::ALL {
                for pose in Pose::ALL {
                    fg.push(a.creature(look, age, pose, false));
                }
            }
        }
        fg.extend(BLook::ALL.iter().map(|&b| a.building(b)));
        fg.extend(ItemLook::ALL.iter().map(|&i| a.item(i)));
        fg.extend(UiLook::ALL.iter().map(|&u| a.ui(u)));
        fg.extend(CondLook::ALL.iter().map(|&c| a.cond(c)));
        for sp in SpotLook::ALL {
            for st in 0..4 {
                fg.push(a.spot(sp, st));
            }
        }
        for w in 1..=4 {
            for h in 1..=4 {
                fg.push(a.site(w, h));
                fg.push(a.ruin(w, h));
            }
        }
        fg.extend([
            a.husk(),
            a.flag(),
            a.pointer(),
            a.eggs(0),
            a.eggs(1),
            a.eggs(2),
        ]);
        for c in colours(fg.into_iter()) {
            assert!(
                c == 0 || layer(c) != Layer::Background,
                "foreground uses index {c}"
            );
        }
    }

    #[test]
    fn walkers_stand_on_their_bottom_row() {
        let a = Art::new();
        for look in Look::ALL {
            if look == Look::Wisp {
                continue; // it floats
            }
            for age in Age::ALL {
                for pose in [
                    Pose::Walk0,
                    Pose::Walk1,
                    Pose::Work0,
                    Pose::Work1,
                    Pose::Strike,
                ] {
                    for left in [false, true] {
                        let s = a.creature(look, age, pose, left);
                        // The hull is the bottom row; the feet sit right on top of it.
                        let (hull, feet) = (s.h - 1, s.h - 2);
                        assert!(
                            (0..s.w).any(|x| s.get(x, hull) != 0),
                            "{look:?} {age:?} {pose:?}: hull under the feet"
                        );
                        assert!(
                            (0..s.w).any(|x| s.get(x, feet) != 0 && s.get(x, feet) != INK),
                            "{look:?} {age:?} {pose:?}: feet on the bottom row"
                        );
                    }
                }
            }
        }
    }

    #[test]
    fn sizes_are_exact() {
        let a = Art::new();
        for t in TileLook::ALL {
            for v in 0..=255u8 {
                let s = a.tile(t, v);
                assert_eq!((s.w, s.h), (16, 16), "{t:?} {v}");
            }
        }
        for u in UiLook::ALL {
            let s = a.ui(u);
            assert_eq!((s.w, s.h), (11, 11), "{u:?}");
        }
        for i in ItemLook::ALL {
            let s = a.item(i);
            assert_eq!((s.w, s.h), (9, 9), "{i:?}");
        }
        for c in CondLook::ALL {
            let s = a.cond(c);
            assert_eq!((s.w, s.h), (7, 7), "{c:?}");
        }
        for sp in SpotLook::ALL {
            for st in 0..4 {
                let s = a.spot(sp, st);
                assert_eq!((s.w, s.h), (16, 16), "{sp:?} {st}");
            }
        }
        for b in BLook::ALL {
            let (fw, fh) = footprint(b);
            let s = a.building(b);
            assert_eq!(s.w, fw * TILE, "{b:?} width");
            assert!(s.h >= fh * TILE, "{b:?} height");
        }
        for w in 1..=4 {
            for h in 1..=4 {
                assert_eq!(a.site(w, h).w, w * TILE);
                assert_eq!(a.site(w, h).h, h * TILE);
                assert_eq!(a.ruin(w, h).w, w * TILE);
                assert_eq!(a.ruin(w, h).h, h * TILE);
            }
        }
    }

    #[test]
    fn every_variant_has_a_sprite() {
        let a = Art::new();
        let mut all: Vec<&Sprite> = vec![a.pointer(), a.husk(), a.flag()];
        for look in Look::ALL {
            for age in Age::ALL {
                for pose in Pose::ALL {
                    for left in [false, true] {
                        all.push(a.creature(look, age, pose, left));
                    }
                }
            }
        }
        all.extend(BLook::ALL.iter().map(|&b| a.building(b)));
        for s in SpotLook::ALL {
            for st in 0..4 {
                all.push(a.spot(s, st));
            }
        }
        all.extend(Deco::ALL.iter().map(|&d| a.deco(d)));
        all.extend(ItemLook::ALL.iter().map(|&i| a.item(i)));
        all.extend(CondLook::ALL.iter().map(|&c| a.cond(c)));
        all.extend(UiLook::ALL.iter().map(|&u| a.ui(u)));
        for st in 0..3 {
            all.push(a.eggs(st));
        }
        for s in all {
            assert!(s.w > 0 && s.h > 0 && !s.is_empty());
        }
    }

    #[test]
    fn amphibian_grids_use_only_keyed_colours() {
        for look in Look::ALL.iter().filter(|l| !l.is_enemy()) {
            for age in Age::ALL {
                let key = amphibian_key(*look, age);
                for young in [false, true] {
                    for pose in [
                        Pose::Walk0,
                        Pose::Walk1,
                        Pose::Work0,
                        Pose::Work1,
                        Pose::Strike,
                    ] {
                        for row in amphibian_rows(*look, young, pose) {
                            for ch in row.chars() {
                                assert!(
                                    ch == '.' || key.iter().any(|k| k.0 == ch),
                                    "{look:?} {pose:?}: {ch:?} has no colour"
                                );
                            }
                        }
                    }
                }
            }
        }
    }

    #[test]
    fn left_is_the_mirror_of_right() {
        let a = Art::new();
        for look in Look::ALL {
            let r = a.creature(look, Age::Adult, Pose::Walk0, false);
            let l = a.creature(look, Age::Adult, Pose::Walk0, true);
            assert_eq!(r.flipped().px, l.px);
        }
    }

    #[test]
    fn ages_differ() {
        let a = Art::new();
        for look in Look::ALL {
            if look.is_enemy() {
                continue;
            }
            let y = a.creature(look, Age::Young, Pose::Walk0, false);
            let ad = a.creature(look, Age::Adult, Pose::Walk0, false);
            let o = a.creature(look, Age::Old, Pose::Walk0, false);
            assert!(y.w * y.h < ad.w * ad.h, "{look:?}: the young are smaller");
            assert_ne!(ad.px, o.px, "{look:?}: the old look older");
        }
    }
}
