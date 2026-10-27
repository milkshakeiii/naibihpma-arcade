use crate::palette::{IndexedSprite, VgaPalette};
use std::path::Path;

pub const FB_W: usize = 400;
pub const FB_H: usize = 300;
pub const WIN_W: usize = 800;
pub const WIN_H: usize = 600;

pub struct Framebuffer {
    pub indices: Vec<u8>, // 400x300 8-bit palette indices (0..63)
    pub shake_x: i32,
    pub shake_y: i32,
}

impl Framebuffer {
    pub fn new() -> Self {
        Self {
            indices: vec![16; FB_W * FB_H],
            shake_x: 0,
            shake_y: 0,
        }
    }

    pub fn clear(&mut self, col: u8) {
        self.indices.fill(col & 63);
    }

    #[inline]
    pub fn pset(&mut self, x: i32, y: i32, col: u8) {
        if x >= 0 && x < FB_W as i32 && y >= 0 && y < FB_H as i32 {
            self.indices[y as usize * FB_W + x as usize] = col & 63;
        }
    }

    #[inline]
    pub fn pget(&self, x: i32, y: i32) -> u8 {
        if x >= 0 && x < FB_W as i32 && y >= 0 && y < FB_H as i32 {
            self.indices[y as usize * FB_W + x as usize]
        } else {
            0
        }
    }

    pub fn rect_fill(&mut self, x: i32, y: i32, w: i32, h: i32, col: u8) {
        let x0 = x.max(0);
        let y0 = y.max(0);
        let x1 = (x + w).min(FB_W as i32);
        let y1 = (y + h).min(FB_H as i32);
        let c = col & 63;
        for py in y0..y1 {
            let row = py as usize * FB_W;
            for px in x0..x1 {
                self.indices[row + px as usize] = c;
            }
        }
    }

    pub fn rect_outline(&mut self, x: i32, y: i32, w: i32, h: i32, col: u8) {
        if w <= 0 || h <= 0 {
            return;
        }
        self.rect_fill(x, y, w, 1, col);
        self.rect_fill(x, y + h - 1, w, 1, col);
        self.rect_fill(x, y, 1, h, col);
        self.rect_fill(x + w - 1, y, 1, h, col);
    }

    /// Classic 1992 Apogee / Coin-Op 3D beveled stone/metal UI box
    pub fn rect_bevel(
        &mut self,
        x: i32,
        y: i32,
        w: i32,
        h: i32,
        fill_col: u8,
        hi_col: u8,
        lo_col: u8,
        outline_col: u8,
    ) {
        if w < 4 || h < 4 {
            self.rect_fill(x, y, w, h, fill_col);
            return;
        }
        self.rect_fill(x, y, w, h, outline_col);
        self.rect_fill(x + 1, y + 1, w - 2, h - 2, fill_col);
        // Top & left highlight
        self.rect_fill(x + 1, y + 1, w - 3, 1, hi_col);
        self.rect_fill(x + 1, y + 1, 1, h - 3, hi_col);
        // Bottom & right shadow
        self.rect_fill(x + 2, y + h - 2, w - 3, 1, lo_col);
        self.rect_fill(x + w - 2, y + 2, 1, h - 3, lo_col);
    }

    /// Classic DOS 2x2 Bayer checkerboard dithering (zero alpha blending!)
    pub fn rect_dither(&mut self, x: i32, y: i32, w: i32, h: i32, col: u8, density: u8) {
        let x0 = x.max(0);
        let y0 = y.max(0);
        let x1 = (x + w).min(FB_W as i32);
        let y1 = (y + h).min(FB_H as i32);
        for py in y0..y1 {
            for px in x0..x1 {
                let bayer = (((px & 1) << 1) | (py & 1)) as u8;
                let draw = match density {
                    1 => bayer == 0,
                    2 => ((px + py) & 1) == 0,
                    _ => bayer != 3,
                };
                if draw {
                    self.pset(px, py, col);
                }
            }
        }
    }

    /// Integer Bresenham line for suction-cup tethers and electric arcs
    pub fn line(&mut self, mut x0: i32, mut y0: i32, x1: i32, y1: i32, col: u8) {
        let dx = (x1 - x0).abs();
        let sx = if x0 < x1 { 1 } else { -1 };
        let dy = -(y1 - y0).abs();
        let sy = if y0 < y1 { 1 } else { -1 };
        let mut err = dx + dy;
        loop {
            self.pset(x0, y0, col);
            if x0 == x1 && y0 == y1 {
                break;
            }
            let e2 = 2 * err;
            if e2 >= dy {
                err += dy;
                x0 += sx;
            }
            if e2 <= dx {
                err += dx;
                y0 += sy;
            }
        }
    }

    pub fn circle_fill(&mut self, cx: i32, cy: i32, r: i32, col: u8) {
        let r2 = r * r;
        for dy in -r..=r {
            for dx in -r..=r {
                if dx * dx + dy * dy <= r2 {
                    self.pset(cx + dx, cy + dy, col);
                }
            }
        }
    }

    /// Blits an indexed sprite at integer coordinates (color 0 = transparent)
    pub fn blit_sprite(
        &mut self,
        spr: &IndexedSprite,
        x: i32,
        y: i32,
        flip_x: bool,
        flip_y: bool,
        flash_override: Option<u8>,
    ) {
        let sw = spr.w as i32;
        let sh = spr.h as i32;
        for sy in 0..sh {
            let py = y + sy;
            if py < 0 || py >= FB_H as i32 {
                continue;
            }
            let src_y = if flip_y { sh - 1 - sy } else { sy } as usize;
            for sx in 0..sw {
                let px = x + sx;
                if px < 0 || px >= FB_W as i32 {
                    continue;
                }
                let src_x = if flip_x { sw - 1 - sx } else { sx } as usize;
                let c = spr.pixels[src_y * spr.w + src_x];
                if c != 0 {
                    let out_c = match flash_override {
                        Some(fc) if c != 1 => fc,
                        _ => c,
                    };
                    self.indices[py as usize * FB_W + px as usize] = out_c & 63;
                }
            }
        }
    }

    /// Blits a scrolling parallax background into the playfield viewport (`0..view_h`)
    pub fn blit_parallax_bg(
        &mut self,
        bg: &IndexedSprite,
        cam_x: i32,
        cam_y: i32,
        view_h: usize,
    ) {
        let bw = bg.w as i32;
        let bh = bg.h as i32;
        let ox = (cam_x / 3).rem_euclid(bw);
        let oy = (cam_y / 4).clamp(0, (bh - view_h as i32).max(0));

        for y in 0..view_h.min(FB_H) {
            let sy = ((y as i32 + oy) % bh) as usize;
            let dst_row = y * FB_W;
            let src_row = sy * bg.w;
            for x in 0..FB_W {
                let sx = ((x as i32 + ox) % bw) as usize;
                self.indices[dst_row + x] = bg.pixels[src_row + sx];
            }
        }
    }

    /// Renders crisp 8x8 bitmap arcade typography (100% integer pixel grid)
    pub fn draw_text(
        &mut self,
        text: &str,
        x: i32,
        y: i32,
        col: u8,
        shadow_col: Option<u8>,
        scale: i32,
    ) {
        let s = scale.max(1);
        let mut cx = x;
        for ch in text.chars() {
            let glyph = glyph_8x8(ch);
            if let Some(sc) = shadow_col {
                for row in 0..8 {
                    let bits = glyph[row];
                    if bits == 0 {
                        continue;
                    }
                    for bit in 0..8 {
                        if (bits & (0x80 >> bit)) != 0 {
                            self.rect_fill(
                                cx + (bit as i32) * s + s,
                                y + (row as i32) * s + s,
                                s,
                                s,
                                sc,
                            );
                        }
                    }
                }
            }
            for row in 0..8 {
                let bits = glyph[row];
                if bits == 0 {
                    continue;
                }
                for bit in 0..8 {
                    if (bits & (0x80 >> bit)) != 0 {
                        self.rect_fill(cx + (bit as i32) * s, y + (row as i32) * s, s, s, col);
                    }
                }
            }
            cx += 6 * s; // Compact 6px arcade kerning for 5x7-in-8x8 glyphs
        }
    }

    pub fn draw_text_centered(
        &mut self,
        text: &str,
        center_x: i32,
        y: i32,
        col: u8,
        shadow_col: Option<u8>,
        scale: i32,
    ) {
        let s = scale.max(1);
        let total_w = (text.chars().count() as i32) * 6 * s;
        self.draw_text(text, center_x - total_w / 2, y, col, shadow_col, s);
    }

    /// Resolves the 400x300 8-bit indexed framebuffer through the 64-color VGA LUT
    /// into the 800x600 32-bit RGB window buffer (exact 2x2 integer blocks + optional CRT scanlines).
    pub fn present_to_800x600(
        &self,
        out_800x600: &mut [u32],
        pal: &VgaPalette,
        crt_scanlines: bool,
    ) {
        let sx_off = self.shake_x;
        let sy_off = self.shake_y;

        for y in 0..FB_H {
            let src_y = (y as i32 + sy_off).clamp(0, FB_H as i32 - 1) as usize;
            let src_row = src_y * FB_W;
            let dst_row_even = (y * 2) * WIN_W;
            let dst_row_odd = (y * 2 + 1) * WIN_W;

            for x in 0..FB_W {
                let src_x = (x as i32 + sx_off).clamp(0, FB_W as i32 - 1) as usize;
                let idx = (self.indices[src_row + src_x] & 63) as usize;
                let argb = pal.active_argb[idx];

                let dx = x * 2;
                out_800x600[dst_row_even + dx] = argb;
                out_800x600[dst_row_even + dx + 1] = argb;

                if crt_scanlines {
                    // Authentic arcade CRT 240p/300p scanline: darken odd physical line by ~22%
                    let r = (((argb >> 16) & 0xFF) * 198) >> 8;
                    let g = (((argb >> 8) & 0xFF) * 198) >> 8;
                    let b = ((argb & 0xFF) * 204) >> 8;
                    let scan_argb = 0xFF00_0000 | (r << 16) | (g << 8) | b;
                    out_800x600[dst_row_odd + dx] = scan_argb;
                    out_800x600[dst_row_odd + dx + 1] = scan_argb;
                } else {
                    out_800x600[dst_row_odd + dx] = argb;
                    out_800x600[dst_row_odd + dx + 1] = argb;
                }
            }
        }
    }

    /// Exports an exact 800x600 PNG screenshot for verification
    pub fn save_png_800x600(&self, path: &Path, pal: &VgaPalette, crt_scanlines: bool) {
        let mut buf32 = vec![0u32; WIN_W * WIN_H];
        self.present_to_800x600(&mut buf32, pal, crt_scanlines);
        let mut rgba = vec![0u8; WIN_W * WIN_H * 4];
        for (i, &px) in buf32.iter().enumerate() {
            rgba[i * 4] = ((px >> 16) & 0xFF) as u8;
            rgba[i * 4 + 1] = ((px >> 8) & 0xFF) as u8;
            rgba[i * 4 + 2] = (px & 0xFF) as u8;
            rgba[i * 4 + 3] = 255;
        }
        let _ = image::save_buffer(
            path,
            &rgba,
            WIN_W as u32,
            WIN_H as u32,
            image::ColorType::Rgba8,
        );
    }
}

/// Crisp 5x7-in-8x8 Arcade Bitmap Font covering uppercase/lowercase, digits, and symbols
fn glyph_8x8(ch: char) -> [u8; 8] {
    match ch.to_ascii_uppercase() {
        'A' => [0x70, 0x88, 0x88, 0xF8, 0x88, 0x88, 0x88, 0x00],
        'B' => [0xF0, 0x88, 0x88, 0xF0, 0x88, 0x88, 0xF0, 0x00],
        'C' => [0x70, 0x88, 0x80, 0x80, 0x80, 0x88, 0x70, 0x00],
        'D' => [0xE0, 0x90, 0x88, 0x88, 0x88, 0x90, 0xE0, 0x00],
        'E' => [0xF8, 0x80, 0x80, 0xF0, 0x80, 0x80, 0xF8, 0x00],
        'F' => [0xF8, 0x80, 0x80, 0xF0, 0x80, 0x80, 0x80, 0x00],
        'G' => [0x70, 0x88, 0x80, 0xB8, 0x88, 0x88, 0x70, 0x00],
        'H' => [0x88, 0x88, 0x88, 0xF8, 0x88, 0x88, 0x88, 0x00],
        'I' => [0x70, 0x20, 0x20, 0x20, 0x20, 0x20, 0x70, 0x00],
        'J' => [0x38, 0x10, 0x10, 0x10, 0x90, 0x90, 0x60, 0x00],
        'K' => [0x88, 0x90, 0xA0, 0xC0, 0xA0, 0x90, 0x88, 0x00],
        'L' => [0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0xF8, 0x00],
        'M' => [0x88, 0xD8, 0xA8, 0xA8, 0x88, 0x88, 0x88, 0x00],
        'N' => [0x88, 0xC8, 0xA8, 0x98, 0x88, 0x88, 0x88, 0x00],
        'O' => [0x70, 0x88, 0x88, 0x88, 0x88, 0x88, 0x70, 0x00],
        'P' => [0xF0, 0x88, 0x88, 0xF0, 0x80, 0x80, 0x80, 0x00],
        'Q' => [0x70, 0x88, 0x88, 0x88, 0xA8, 0x90, 0x68, 0x00],
        'R' => [0xF0, 0x88, 0x88, 0xF0, 0xA0, 0x90, 0x88, 0x00],
        'S' => [0x78, 0x80, 0x80, 0x70, 0x08, 0x08, 0xF0, 0x00],
        'T' => [0xF8, 0x20, 0x20, 0x20, 0x20, 0x20, 0x20, 0x00],
        'U' => [0x88, 0x88, 0x88, 0x88, 0x88, 0x88, 0x70, 0x00],
        'V' => [0x88, 0x88, 0x88, 0x88, 0x50, 0x50, 0x20, 0x00],
        'W' => [0x88, 0x88, 0x88, 0xA8, 0xA8, 0xD8, 0x88, 0x00],
        'X' => [0x88, 0x88, 0x50, 0x20, 0x50, 0x88, 0x88, 0x00],
        'Y' => [0x88, 0x88, 0x50, 0x20, 0x20, 0x20, 0x20, 0x00],
        'Z' => [0xF8, 0x08, 0x10, 0x20, 0x40, 0x80, 0xF8, 0x00],
        '0' => [0x70, 0x88, 0x98, 0xA8, 0xC8, 0x88, 0x70, 0x00],
        '1' => [0x20, 0x60, 0x20, 0x20, 0x20, 0x20, 0x70, 0x00],
        '2' => [0x70, 0x88, 0x08, 0x30, 0x40, 0x80, 0xF8, 0x00],
        '3' => [0x70, 0x88, 0x08, 0x30, 0x08, 0x88, 0x70, 0x00],
        '4' => [0x10, 0x30, 0x50, 0x90, 0xF8, 0x10, 0x10, 0x00],
        '5' => [0xF8, 0x80, 0xF0, 0x08, 0x08, 0x88, 0x70, 0x00],
        '6' => [0x38, 0x40, 0x80, 0xF0, 0x88, 0x88, 0x70, 0x00],
        '7' => [0xF8, 0x08, 0x10, 0x20, 0x40, 0x40, 0x40, 0x00],
        '8' => [0x70, 0x88, 0x88, 0x70, 0x88, 0x88, 0x70, 0x00],
        '9' => [0x70, 0x88, 0x88, 0x78, 0x08, 0x10, 0xE0, 0x00],
        ':' => [0x00, 0x30, 0x30, 0x00, 0x30, 0x30, 0x00, 0x00],
        '.' => [0x00, 0x00, 0x00, 0x00, 0x00, 0x30, 0x30, 0x00],
        ',' => [0x00, 0x00, 0x00, 0x00, 0x30, 0x30, 0x20, 0x40],
        '!' => [0x20, 0x20, 0x20, 0x20, 0x20, 0x00, 0x20, 0x00],
        '?' => [0x70, 0x88, 0x10, 0x20, 0x20, 0x00, 0x20, 0x00],
        '-' => [0x00, 0x00, 0x00, 0xF0, 0x00, 0x00, 0x00, 0x00],
        '+' => [0x00, 0x20, 0x20, 0xF8, 0x20, 0x20, 0x00, 0x00],
        '/' => [0x08, 0x10, 0x20, 0x20, 0x40, 0x80, 0x80, 0x00],
        '[' => [0x70, 0x40, 0x40, 0x40, 0x40, 0x40, 0x70, 0x00],
        ']' => [0x70, 0x10, 0x10, 0x10, 0x10, 0x10, 0x70, 0x00],
        '(' => [0x30, 0x40, 0x40, 0x40, 0x40, 0x40, 0x30, 0x00],
        ')' => [0x60, 0x10, 0x10, 0x10, 0x10, 0x10, 0x60, 0x00],
        '$' => [0x20, 0x78, 0xA0, 0x70, 0x28, 0xF0, 0x20, 0x00],
        '%' => [0xC8, 0xC8, 0x10, 0x20, 0x40, 0x98, 0x98, 0x00],
        '&' => [0x60, 0x90, 0x60, 0xA8, 0x90, 0x90, 0x68, 0x00],
        '*' => [0x00, 0x88, 0x50, 0xF8, 0x50, 0x88, 0x00, 0x00],
        '>' => [0x80, 0x40, 0x20, 0x10, 0x20, 0x40, 0x80, 0x00],
        '<' => [0x10, 0x20, 0x40, 0x80, 0x40, 0x20, 0x10, 0x00],
        '\'' => [0x20, 0x20, 0x40, 0x00, 0x00, 0x00, 0x00, 0x00],
        '~' => [0x6C, 0xFE, 0xFE, 0xFE, 0x7C, 0x38, 0x10, 0x00], // Crisp 7x7 Arcade Heart Icon!
        _ => [0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00],
    }
}
