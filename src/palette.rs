use image::GenericImageView;
use std::path::Path;

#[derive(Clone)]
pub struct IndexedSprite {
    pub w: usize,
    pub h: usize,
    pub pixels: Vec<u8>, // 0 = transparent, 1..63 = palette index
}

impl IndexedSprite {
    pub fn new(w: usize, h: usize) -> Self {
        Self {
            w,
            h,
            pixels: vec![0; w * h],
        }
    }

    #[inline]
    pub fn get(&self, x: usize, y: usize) -> u8 {
        if x < self.w && y < self.h {
            self.pixels[y * self.w + x]
        } else {
            0
        }
    }

    #[inline]
    pub fn set(&mut self, x: usize, y: usize, v: u8) {
        if x < self.w && y < self.h {
            self.pixels[y * self.w + x] = v;
        }
    }

    /// Scales a pixel-art sprite using EPX / Scale2x pixel-art corner rules + 1px hull outline
    /// so larger Cosmo-scale sprites (36x44) keep razor-sharp 1px outlines and diagonal steps.
    pub fn scale_epx_2x(&self) -> Self {
        let mut out = Self::new(self.w * 2, self.h * 2);
        for y in 0..self.h {
            for x in 0..self.w {
                let p = self.get(x, y);
                // Strip old 1px outline before EPX scaling so we can apply a fresh 1px outline after
                let p_clean = if p == 1 { 0 } else { p };
                let a = {
                    let v = if y > 0 { self.get(x, y - 1) } else { 0 };
                    if v == 1 { 0 } else { v }
                };
                let b = {
                    let v = if x + 1 < self.w { self.get(x + 1, y) } else { 0 };
                    if v == 1 { 0 } else { v }
                };
                let c = {
                    let v = if x > 0 { self.get(x - 1, y) } else { 0 };
                    if v == 1 { 0 } else { v }
                };
                let d = {
                    let v = if y + 1 < self.h { self.get(x, y + 1) } else { 0 };
                    if v == 1 { 0 } else { v }
                };

                let mut p1 = p_clean;
                let mut p2 = p_clean;
                let mut p3 = p_clean;
                let mut p4 = p_clean;

                if c == a && c != d && a != b {
                    p1 = a;
                }
                if a == b && a != c && b != d {
                    p2 = b;
                }
                if d == c && d != b && c != a {
                    p3 = c;
                }
                if b == d && b != a && d != c {
                    p4 = d;
                }

                out.set(x * 2, y * 2, p1);
                out.set(x * 2 + 1, y * 2, p2);
                out.set(x * 2, y * 2 + 1, p3);
                out.set(x * 2 + 1, y * 2 + 1, p4);
            }
        }
        out.enforce_1px_outline();
        out
    }

    /// Creates a "diced / chopped" culinary variant by slicing the sprite into 3 offset chunks
    /// and re-outlining with 1px dark hull.
    pub fn make_diced_variant(&self) -> Self {
        let mut out = Self::new(self.w, self.h);
        for y in 0..self.h {
            for x in 0..self.w {
                let band = (x + y / 2) / 6;
                let (sx, sy) = match band % 3 {
                    0 => (x.wrapping_add(1), y),
                    1 => (x, y.wrapping_add(1)),
                    _ => (x.wrapping_sub(1), y.wrapping_sub(1)),
                };
                if (x + y / 2) % 6 == 0 {
                    continue;
                }
                if sx < self.w && sy < self.h {
                    let c = self.get(sx, sy);
                    if c != 0 && c != 1 {
                        out.set(x, y, c);
                    }
                }
            }
        }
        out.enforce_1px_outline();
        out
    }

    /// Enforces a clean 1px dark outline (index 1) around all non-transparent interior pixels
    pub fn enforce_1px_outline(&mut self) {
        let orig = self.pixels.clone();
        for y in 0..self.h {
            for x in 0..self.w {
                if orig[y * self.w + x] == 0 {
                    let mut touches_body = false;
                    if x > 0 && orig[y * self.w + (x - 1)] > 1 {
                        touches_body = true;
                    }
                    if x + 1 < self.w && orig[y * self.w + (x + 1)] > 1 {
                        touches_body = true;
                    }
                    if y > 0 && orig[(y - 1) * self.w + x] > 1 {
                        touches_body = true;
                    }
                    if y + 1 < self.h && orig[(y + 1) * self.w + x] > 1 {
                        touches_body = true;
                    }
                    if touches_body {
                        self.pixels[y * self.w + x] = 1;
                    }
                }
            }
        }
    }

    /// Removes isolated 1-pixel noise speckles produced by raw AI generation
    pub fn despeckle(&mut self) {
        let orig = self.pixels.clone();
        for y in 1..self.h.saturating_sub(1) {
            for x in 1..self.w.saturating_sub(1) {
                let c = orig[y * self.w + x];
                if c <= 1 {
                    continue;
                }
                let n = [
                    orig[(y - 1) * self.w + x],
                    orig[(y + 1) * self.w + x],
                    orig[y * self.w + (x - 1)],
                    orig[y * self.w + (x + 1)],
                ];
                if n[0] != c && n[1] != c && n[2] != c && n[3] != c {
                    if n[0] == n[1] && n[0] > 0 {
                        self.pixels[y * self.w + x] = n[0];
                    } else if n[2] == n[3] && n[2] > 0 {
                        self.pixels[y * self.w + x] = n[2];
                    } else if n[0] > 0 {
                        self.pixels[y * self.w + x] = n[0];
                    }
                }
            }
        }
    }
}

#[derive(Clone, Copy)]
pub struct VgaPalette {
    pub base_rgb: [(u8, u8, u8); 64],
    pub active_argb: [u32; 64],
    pub oklab_table: [(f32, f32, f32); 64],
    pub fade_level: i32,
}

impl VgaPalette {
    pub fn new_naibihpma() -> Self {
        let base_rgb: [(u8, u8, u8); 64] = [
            // 0..15: Core Arcade UI, Outlines, & Chef Cosmo Ramp
            (0x05, 0x06, 0x0D), // 0: Transparent / Void Black
            (0x08, 0x0A, 0x14), // 1: 1px Sprite Hull Outline (Deepest Bog Ink)
            (0x14, 0x1B, 0x2D), // 2: Dark Slate Shadow / HUD Background
            (0x22, 0x2F, 0x49), // 3: Mid Slate Bevel Face
            (0x3B, 0x4F, 0x73), // 4: Light Slate Bevel Highlight
            (0x68, 0x82, 0xAA), // 5: Steel Trim / Muted Text
            (0xA6, 0xBD, 0xDC), // 6: Silver Plate / Secondary Text
            (0xF2, 0xF7, 0xFF), // 7: Crisp Arcade White / Chef Toque
            (0x16, 0x5A, 0x4C), // 8: Cosmo Dark Teal Shadow
            (0x24, 0x96, 0x58), // 9: Cosmo Emerald Skin Mid
            (0x4E, 0xD4, 0x68), // 10: Cosmo Lime Skin Bright
            (0x98, 0xF6, 0x98), // 11: Cosmo Mint Highlight
            (0x8C, 0x22, 0x18), // 12: Suction Cup Deep Crimson
            (0xD8, 0x46, 0x20), // 13: Suction Cup Bright Orange-Red
            (0xFF, 0x82, 0x2E), // 14: Suction Cup Tangerine Highlight
            (0xFF, 0xCC, 0x40), // 15: Arcade Gold / Score Counter Yellow

            // 16..31: The Enchanted Forest of Naibihpma Biome Ramp
            (0x09, 0x10, 0x1E), // 16: Deep Canopy Night Sky
            (0x10, 0x1E, 0x2E), // 17: Distant Marsh Silhouette Dark
            (0x17, 0x30, 0x40), // 18: Distant Canopy Silhouette Mid
            (0x2B, 0x1A, 0x17), // 19: Ancient Hollow Bark Dark
            (0x4A, 0x2D, 0x22), // 20: Ancient Bark Mid Brown
            (0x70, 0x48, 0x30), // 21: Bark Ridge Highlight
            (0x11, 0x33, 0x24), // 22: Deep Bog Moss Shadow
            (0x1E, 0x59, 0x38), // 23: Forest Canopy Green
            (0x33, 0x88, 0x4D), // 24: Lush Lily-Pad Jade
            (0x63, 0xB8, 0x48), // 25: Bright Bog-Fern Green
            (0x9A, 0xE2, 0x42), // 26: Wet Mucus Chartreuse
            (0x1C, 0x12, 0x30), // 27: Spore-Cavern Deep Violet
            (0x32, 0x1E, 0x4E), // 28: Twilight Bog Purple
            (0x7A, 0x3E, 0x96), // 29: Bruised Lotus Magenta
            (0xC8, 0x54, 0xC2), // 30: Orchid Spore Glow
            (0x5C, 0x16, 0x2A), // 31: Thorny Bramble Dark Maroon

            // 32..47: Amphibian Creatures, Ingredients, & Galley Kitchen Ramp
            (0x0E, 0x4C, 0x68), // 32: Dew-Newt Deep Cerulean
            (0x18, 0x88, 0xB0), // 33: Dew-Newt Cyan Body
            (0x3C, 0xD0, 0xEC), // 34: Dew-Newt Bright Aqua
            (0x8E, 0xF6, 0xFF), // 35: Bioluminescent Ice Highlight
            (0x6E, 0x18, 0x24), // 36: Bellow-Toad Dark Crimson
            (0xB4, 0x28, 0x38), // 37: Bellow-Toad Vocal-Sac Ruby
            (0xF0, 0x4E, 0x58), // 38: Inflated Sac Coral Pink
            (0x94, 0x58, 0x16), // 39: Golden Roe / Truffle Umber
            (0xCC, 0x84, 0x22), // 40: Copper Wok / Truffle Ochre
            (0xF6, 0xB4, 0x36), // 41: Puffed Vocal-Roe Gold
            (0xFF, 0xE2, 0x7A), // 42: Butter-Glaze Cream
            (0x36, 0x22, 0x18), // 43: Seared Char-Bark Dark
            (0x54, 0x38, 0x28), // 44: Roasted Marsh-Meat Brown
            (0x4E, 0x62, 0x32), // 45: Toad Olive Skin Mid
            (0x76, 0x90, 0x42), // 46: Toad Olive Skin Highlight
            (0x8C, 0xA0, 0xB4), // 47: Galley Cleaver Blade Steel

            // 48..63: Hardware Palette-Cycling Animated Banks
            (0x18, 0x6E, 0x8C), // 48
            (0x28, 0xAE, 0xCC), // 49
            (0x58, 0xE8, 0xF8), // 50
            (0xCA, 0xFC, 0xFF), // 51

            (0x26, 0x68, 0x34), // 52
            (0x46, 0x98, 0x42), // 53
            (0x7C, 0xC8, 0x48), // 54
            (0xB8, 0xF2, 0x64), // 55

            (0x28, 0x44, 0x98), // 56
            (0x48, 0x88, 0xE8), // 57
            (0x88, 0xE0, 0xFF), // 58
            (0xFF, 0xFA, 0x98), // 59

            (0x98, 0x22, 0x48), // 60
            (0xD8, 0x48, 0x28), // 61
            (0xF8, 0x90, 0x24), // 62
            (0xFF, 0xE8, 0x68), // 63
        ];

        let mut oklab_table = [(0.0, 0.0, 0.0); 64];
        for i in 0..64 {
            let (r, g, b) = base_rgb[i];
            oklab_table[i] = rgb_to_oklab(r, g, b);
        }

        let mut pal = Self {
            base_rgb,
            active_argb: [0; 64],
            oklab_table,
            fade_level: 16,
        };
        pal.update_cycler(0);
        pal
    }

    pub fn update_cycler(&mut self, tick: u64) {
        let mut rgb = self.base_rgb;

        // Only gently breathe Bank A (48..51) at a very slow, calm pace for distant background lanterns.
        // Keep all other banks (52..63) 100% steady so NO text or UI border ever flashes!
        let shift_a = ((tick / 32) % 4) as usize;
        for i in 0..4 {
            rgb[48 + i] = self.base_rgb[48 + ((i + shift_a) % 4)];
        }

        for i in 0..64 {
            let (mut r, mut g, mut b) = rgb[i];
            if self.fade_level < 16 {
                let lvl = self.fade_level.max(0) as u32;
                r = ((r as u32 * lvl) / 16) as u8;
                g = ((g as u32 * lvl) / 16) as u8;
                b = ((b as u32 * lvl) / 16) as u8;
            }
            let vr = r & 0xFC;
            let vg = g & 0xFC;
            let vb = b & 0xFC;
            self.active_argb[i] =
                0xFF00_0000 | ((vr as u32) << 16) | ((vg as u32) << 8) | (vb as u32);
        }
    }

    pub fn nearest_index_oklab(&self, r: u8, g: u8, b: u8, allow_cyclers: bool) -> u8 {
        if allow_cyclers && g > 215 && b > 220 && r < 130 {
            return if g > 245 { 51 } else { 50 };
        }
        if allow_cyclers && r > 240 && g > 180 && b < 110 {
            return 62;
        }

        let (l, a, b_lab) = rgb_to_oklab(r, g, b);
        let mut best_idx = 1u8;
        let mut best_dist = f32::MAX;
        for idx in 1..48 {
            let (tl, ta, tb) = self.oklab_table[idx];
            let dl = l - tl;
            let da = a - ta;
            let db = b_lab - tb;
            let dist = dl * dl * 1.35 + da * da + db * db;
            if dist < best_dist {
                best_dist = dist;
                best_idx = idx as u8;
            }
        }
        best_idx
    }

    pub fn quantize_background_png(
        &self,
        path: &Path,
        target_w: usize,
        target_h: usize,
    ) -> Option<IndexedSprite> {
        let img = image::open(path).ok()?;
        let (src_w, src_h) = img.dimensions();
        let rgba = img.to_rgba8();
        let mut out = IndexedSprite::new(target_w, target_h);

        for ty in 0..target_h {
            let y0 = (ty * src_h as usize) / target_h;
            let y1 = (((ty + 1) * src_h as usize) / target_h).max(y0 + 1);
            for tx in 0..target_w {
                let x0 = (tx * src_w as usize) / target_w;
                let x1 = (((tx + 1) * src_w as usize) / target_w).max(x0 + 1);

                let mut counts = [0u8; 64];
                let mut best_idx = 16u8;
                let mut best_count = 0u8;

                for sy_step in 0..3 {
                    let sy = (y0 + ((y1 - y0) * (sy_step * 2 + 1)) / 6).min(src_h as usize - 1);
                    for sx_step in 0..3 {
                        let sx = (x0 + ((x1 - x0) * (sx_step * 2 + 1)) / 6).min(src_w as usize - 1);
                        let p = rgba.get_pixel(sx as u32, sy as u32);
                        let idx = self.nearest_index_oklab(p[0], p[1], p[2], true);
                        counts[idx as usize] += 1;
                        if counts[idx as usize] > best_count {
                            best_count = counts[idx as usize];
                            best_idx = idx;
                        }
                    }
                }
                out.set(tx, ty, best_idx);
            }
        }
        Some(out)
    }

    /// Creates a deep-recessed twilight version of the background for gameplay so foreground
    /// tiles and characters pop with 100% clarity (classic Cosmo foreground/background separation).
    pub fn make_recessed_playfield_bg(&self, src: &IndexedSprite) -> IndexedSprite {
        let mut out = IndexedSprite::new(src.w, src.h);
        for y in 0..src.h {
            for x in 0..src.w {
                let c = src.get(x, y);
                let mapped = match c {
                    48..=51 => 48,
                    60..=63 => 29,
                    30 | 29 => 28,
                    24 | 25 | 26 | 10 | 11 => 18,
                    20 | 21 | 22 | 23 | 8 | 9 | 18 | 32 | 33 => 17,
                    27 | 28 => 27,
                    _ => 16,
                };
                out.set(x, y, mapped);
            }
        }
        out
    }

    pub fn quantize_sprite_sheet_png(
        &self,
        path: &Path,
        cols: usize,
        rows: usize,
        target_size: usize,
    ) -> Option<Vec<IndexedSprite>> {
        let img = image::open(path).ok()?;
        let (src_w, src_h) = img.dimensions();
        let rgba = img.to_rgba8();

        let cell_w = src_w as usize / cols;
        let cell_h = src_h as usize / rows;
        let mut sprites = Vec::with_capacity(cols * rows);

        let is_magenta_bg = |r: u8, g: u8, b: u8, a: u8| -> bool {
            if a < 128 {
                return true;
            }
            let r_i = r as i32;
            let g_i = g as i32;
            let b_i = b as i32;
            (r_i > 165 && b_i > 165 && g_i < 105 && (r_i + b_i - 2 * g_i) > 210)
                || (r_i > 210 && b_i > 200 && g_i < 135)
        };

        for row in 0..rows {
            for col in 0..cols {
                let cx0 = col * cell_w + cell_w / 18;
                let cy0 = row * cell_h + cell_h / 18;
                let cx1 = (col + 1) * cell_w - cell_w / 18;
                let cy1 = (row + 1) * cell_h - cell_h / 18;

                let mut min_x = cx1;
                let mut min_y = cy1;
                let mut max_x = cx0;
                let mut max_y = cy0;

                for y in cy0..cy1 {
                    for x in cx0..cx1 {
                        let p = rgba.get_pixel(x as u32, y as u32);
                        if !is_magenta_bg(p[0], p[1], p[2], p[3]) {
                            if x < min_x {
                                min_x = x;
                            }
                            if x > max_x {
                                max_x = x;
                            }
                            if y < min_y {
                                min_y = y;
                            }
                            if y > max_y {
                                max_y = y;
                            }
                        }
                    }
                }

                let mut spr = IndexedSprite::new(target_size, target_size);
                if max_x <= min_x || max_y <= min_y {
                    sprites.push(spr);
                    continue;
                }

                let bw = (max_x - min_x + 1).max(1);
                let bh = (max_y - min_y + 1).max(1);
                let box_span = bw.max(bh);
                let pad_x = (box_span - bw) / 2;
                let pad_y = (box_span - bh) / 2;

                let inner = target_size.saturating_sub(2).max(1);

                for ty in 0..inner {
                    for tx in 0..inner {
                        let sx_center = min_x as i32 - pad_x as i32
                            + ((tx * 2 + 1) * box_span) as i32 / (inner as i32 * 2);
                        let sy_center = min_y as i32 - pad_y as i32
                            + ((ty * 2 + 1) * box_span) as i32 / (inner as i32 * 2);

                        let mut counts = [0u8; 64];
                        let mut fg_votes = 0;
                        let mut best_idx = 1u8;
                        let mut best_count = 0u8;

                        for dy in -1..=1 {
                            for dx in -1..=1 {
                                let sx = (sx_center + dx * 2).clamp(cx0 as i32, cx1 as i32 - 1);
                                let sy = (sy_center + dy * 2).clamp(cy0 as i32, cy1 as i32 - 1);
                                let p = rgba.get_pixel(sx as u32, sy as u32);
                                if !is_magenta_bg(p[0], p[1], p[2], p[3]) {
                                    fg_votes += 1;
                                    let idx = self.nearest_index_oklab(p[0], p[1], p[2], false);
                                    counts[idx as usize] += 1;
                                    if counts[idx as usize] > best_count {
                                        best_count = counts[idx as usize];
                                        best_idx = idx;
                                    }
                                }
                            }
                        }

                        if fg_votes >= 4 {
                            spr.set(tx + 1, ty + 1, best_idx);
                        }
                    }
                }

                spr.despeckle();
                spr.enforce_1px_outline();
                sprites.push(spr);
            }
        }

        Some(sprites)
    }
}

fn srgb_to_linear(c: u8) -> f32 {
    let x = c as f32 / 255.0;
    if x <= 0.04045 {
        x / 12.92
    } else {
        ((x + 0.055) / 1.055).powf(2.4)
    }
}

pub fn rgb_to_oklab(r: u8, g: u8, b: u8) -> (f32, f32, f32) {
    let lr = srgb_to_linear(r);
    let lg = srgb_to_linear(g);
    let lb = srgb_to_linear(b);

    let l = (0.4122214708 * lr + 0.5363325363 * lg + 0.0514459929 * lb).cbrt();
    let m = (0.2119034982 * lr + 0.6806995451 * lg + 0.1073969566 * lb).cbrt();
    let s = (0.0883024619 * lr + 0.2817188376 * lg + 0.6299787005 * lb).cbrt();

    (
        0.2104542553 * l + 0.7936177850 * m - 0.0040720468 * s,
        1.9779984951 * l - 2.4285922050 * m + 0.4505937099 * s,
        0.0259040371 * l + 0.7827717662 * m - 0.8086757660 * s,
    )
}
