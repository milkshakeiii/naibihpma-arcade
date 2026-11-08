//! The 400x300 indexed canvas, primitives, the bitmap font and exact integer presentation.

use std::path::Path;

pub const FB_W: usize = 400;
pub const FB_H: usize = 300;

/// A hand-authored indexed sprite. Index 0 = transparent, 1 = dark hull outline.
#[derive(Clone, Debug)]
pub struct Sprite {
    pub w: i32,
    pub h: i32,
    pub px: Vec<u8>,
}

impl Sprite {
    /// Builds a sprite from rows of characters through a key function.
    pub fn art(rows: &[&str], key: impl Fn(char) -> u8) -> Self {
        let h = rows.len() as i32;
        let w = rows.iter().map(|r| r.chars().count()).max().unwrap_or(0) as i32;
        let mut px = vec![0u8; (w * h) as usize];
        for (y, row) in rows.iter().enumerate() {
            for (x, c) in row.chars().enumerate() {
                px[y * w as usize + x] = key(c);
            }
        }
        Self { w, h, px }
    }

    /// Lowest row holding an opaque pixel (for checking feet are planted).
    pub fn bottom_row(&self) -> i32 {
        (0..self.h).rev().find(|&y| (0..self.w).any(|x| self.px[(y * self.w + x) as usize] != 0)).unwrap_or(-1)
    }
}

pub struct Framebuffer {
    pub px: Vec<u8>,
}

impl Framebuffer {
    pub fn new() -> Self {
        Self { px: vec![0; FB_W * FB_H] }
    }

    pub fn clear(&mut self, col: u8) {
        self.px.fill(col);
    }

    #[inline]
    pub fn pset(&mut self, x: i32, y: i32, col: u8) {
        if x >= 0 && x < FB_W as i32 && y >= 0 && y < FB_H as i32 {
            self.px[y as usize * FB_W + x as usize] = col & 63;
        }
    }

    #[inline]
    pub fn get(&self, x: i32, y: i32) -> u8 {
        if x >= 0 && x < FB_W as i32 && y >= 0 && y < FB_H as i32 {
            self.px[y as usize * FB_W + x as usize]
        } else {
            0
        }
    }

    pub fn rect_fill(&mut self, x: i32, y: i32, w: i32, h: i32, col: u8) {
        let (x0, y0) = (x.max(0), y.max(0));
        let (x1, y1) = ((x + w).min(FB_W as i32), (y + h).min(FB_H as i32));
        for py in y0..y1 {
            let row = py as usize * FB_W;
            for px in x0..x1 {
                self.px[row + px as usize] = col & 63;
            }
        }
    }

    pub fn rect_outline(&mut self, x: i32, y: i32, w: i32, h: i32, col: u8) {
        self.rect_fill(x, y, w, 1, col);
        self.rect_fill(x, y + h - 1, w, 1, col);
        self.rect_fill(x, y, 1, h, col);
        self.rect_fill(x + w - 1, y, 1, h, col);
    }

    /// Corner brackets only: a calm selection mark that never hides the tile.
    pub fn corners(&mut self, x: i32, y: i32, w: i32, h: i32, len: i32, col: u8) {
        for (cx, cy, sx, sy) in [(x, y, 1, 1), (x + w - 1, y, -1, 1), (x, y + h - 1, 1, -1), (x + w - 1, y + h - 1, -1, -1)] {
            for i in 0..len {
                self.pset(cx + i * sx, cy, col);
                self.pset(cx, cy + i * sy, col);
            }
        }
    }

    /// Classic Apogee-style beveled panel: outline, fill, top/left highlight, bottom/right shadow.
    pub fn rect_bevel(&mut self, x: i32, y: i32, w: i32, h: i32, fill: u8, hi: u8, lo: u8) {
        self.rect_fill(x, y, w, h, 1);
        self.rect_fill(x + 1, y + 1, w - 2, h - 2, fill);
        self.rect_fill(x + 1, y + 1, w - 3, 1, hi);
        self.rect_fill(x + 1, y + 1, 1, h - 3, hi);
        self.rect_fill(x + 2, y + h - 2, w - 3, 1, lo);
        self.rect_fill(x + w - 2, y + 2, 1, h - 3, lo);
    }

    /// Checkerboard dither of a single colour (dims and marks without alpha blending).
    pub fn rect_checker(&mut self, x: i32, y: i32, w: i32, h: i32, col: u8, phase: i32) {
        for py in y.max(0)..(y + h).min(FB_H as i32) {
            for px in x.max(0)..(x + w).min(FB_W as i32) {
                if (px + py + phase) & 1 == 0 {
                    self.px[py as usize * FB_W + px as usize] = col & 63;
                }
            }
        }
    }

    /// Sparse diagonal hatching: every fourth diagonal (danger marks that keep the tile readable).
    pub fn rect_hatch(&mut self, x: i32, y: i32, w: i32, h: i32, col: u8) {
        for py in y.max(0)..(y + h).min(FB_H as i32) {
            for px in x.max(0)..(x + w).min(FB_W as i32) {
                if (px - py).rem_euclid(4) == 0 {
                    self.px[py as usize * FB_W + px as usize] = col & 63;
                }
            }
        }
    }

    /// Integer Bresenham line.
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

    /// Integer midpoint circle outline.
    pub fn circle(&mut self, cx: i32, cy: i32, r: i32, col: u8) {
        let (mut x, mut y, mut err) = (r, 0, 1 - r);
        while x >= y {
            for (dx, dy) in [(x, y), (y, x), (-y, x), (-x, y), (-x, -y), (-y, -x), (y, -x), (x, -y)] {
                self.pset(cx + dx, cy + dy, col);
            }
            y += 1;
            if err < 0 {
                err += 2 * y + 1;
            } else {
                x -= 1;
                err += 2 * (y - x) + 1;
            }
        }
    }

    /// Blits an indexed sprite at integer coordinates (index 0 = transparent).
    pub fn blit(&mut self, spr: &Sprite, x: i32, y: i32, flip_x: bool) {
        self.blit_map(spr, x, y, flip_x, &IDENTITY);
    }

    /// Blits through a 64-entry palette remap table (team colours, spirit tint, silhouettes).
    pub fn blit_map(&mut self, spr: &Sprite, x: i32, y: i32, flip_x: bool, map: &[u8; 64]) {
        for sy in 0..spr.h {
            let py = y + sy;
            if py < 0 || py >= FB_H as i32 {
                continue;
            }
            for sx in 0..spr.w {
                let px = x + sx;
                if px < 0 || px >= FB_W as i32 {
                    continue;
                }
                let src_x = if flip_x { spr.w - 1 - sx } else { sx };
                let c = spr.px[(sy * spr.w + src_x) as usize];
                if c != 0 {
                    self.px[py as usize * FB_W + px as usize] = map[c as usize & 63];
                }
            }
        }
    }

    /// 5x7-in-8x8 bitmap font at integer scales with 6px kerning.
    pub fn text(&mut self, text: &str, x: i32, y: i32, col: u8, shadow: Option<u8>, scale: i32) {
        let s = scale.max(1);
        let mut cx = x;
        for ch in text.chars() {
            let glyph = glyph_8x8(ch);
            for pass in 0..2 {
                let (c, off) = match (pass, shadow) {
                    (0, Some(sc)) => (sc, s),
                    (0, None) => continue,
                    _ => (col, 0),
                };
                for (row, bits) in glyph.iter().enumerate() {
                    for bit in 0..8 {
                        if bits & (0x80 >> bit) != 0 {
                            self.rect_fill(cx + bit * s + off, y + row as i32 * s + off, s, s, c);
                        }
                    }
                }
            }
            cx += 6 * s;
        }
    }

    pub fn text_width(text: &str, scale: i32) -> i32 {
        text.chars().count() as i32 * 6 * scale.max(1)
    }

    pub fn text_centered(&mut self, text: &str, cx: i32, y: i32, col: u8, shadow: Option<u8>, scale: i32) {
        self.text(text, cx - Self::text_width(text, scale) / 2, y, col, shadow, scale);
    }

    /// Word-wrapped text; returns the number of lines drawn. `\n` forces a break.
    pub fn text_wrapped(&mut self, text: &str, x: i32, y: i32, max_w: i32, line_h: i32, col: u8, shadow: Option<u8>) -> i32 {
        let lines = wrap(text, (max_w / 6).max(1) as usize);
        for (i, l) in lines.iter().enumerate() {
            self.text(l, x, y + i as i32 * line_h, col, shadow, 1);
        }
        lines.len() as i32
    }

    /// Largest exact integer multiple `N >= 1` of the 400x300 canvas that fits the viewport.
    pub fn integer_scale_for_viewport(client_w: i32, client_h: i32) -> usize {
        let sx = client_w / FB_W as i32;
        let sy = client_h / FB_H as i32;
        sx.min(sy).max(1) as usize
    }

    /// Resolves indices into an exact (400*N)x(300*N) buffer. Optional scanlines darken only the
    /// bottom physical sub-row of each logical pixel.
    pub fn present(&self, out: &mut Vec<u32>, scale: usize, pal: &[u32; 64], scanlines: bool) -> (usize, usize) {
        let s = scale.max(1);
        let out_w = FB_W * s;
        let out_h = FB_H * s;
        out.resize(out_w * out_h, 0xFF00_0000);
        for y in 0..FB_H {
            let src_row = y * FB_W;
            let row0 = y * s * out_w;
            for x in 0..FB_W {
                let argb = pal[(self.px[src_row + x] & 63) as usize];
                let d = row0 + x * s;
                out[d..d + s].fill(argb);
            }
            let full_rows = if scanlines && s >= 2 { s - 1 } else { s };
            for sub in 1..full_rows {
                out.copy_within(row0..row0 + out_w, row0 + sub * out_w);
            }
            if scanlines && s >= 2 {
                let scan = row0 + (s - 1) * out_w;
                for x in 0..out_w {
                    let argb = out[row0 + x];
                    let r = (((argb >> 16) & 0xFF) * 198) >> 8;
                    let g = (((argb >> 8) & 0xFF) * 198) >> 8;
                    let b = ((argb & 0xFF) * 204) >> 8;
                    out[scan + x] = 0xFF00_0000 | (r << 16) | (g << 8) | b;
                }
            }
        }
        (out_w, out_h)
    }

    /// Exports an exact 2x (800x600) PNG for verification.
    pub fn save_png(&self, path: &Path, pal: &[u32; 64]) {
        let mut buf = Vec::new();
        let (w, h) = self.present(&mut buf, 2, pal, true);
        let mut rgb = Vec::with_capacity(w * h * 3);
        for &p in &buf {
            rgb.extend_from_slice(&[(p >> 16) as u8, (p >> 8) as u8, p as u8]);
        }
        let _ = image::save_buffer(path, &rgb, w as u32, h as u32, image::ColorType::Rgb8);
    }
}

pub const IDENTITY: [u8; 64] = {
    let mut m = [0u8; 64];
    let mut i = 0;
    while i < 64 {
        m[i] = i as u8;
        i += 1;
    }
    m
};

/// Greedy word wrap to `cols` characters; `\n` forces a break, long words are split.
pub fn wrap(text: &str, cols: usize) -> Vec<String> {
    let mut out = Vec::new();
    for para in text.split('\n') {
        let mut line = String::new();
        for word in para.split(' ') {
            let mut word = word.to_string();
            while word.chars().count() > cols {
                if !line.is_empty() {
                    out.push(std::mem::take(&mut line));
                }
                let head: String = word.chars().take(cols).collect();
                word = word.chars().skip(cols).collect();
                out.push(head);
            }
            let need = if line.is_empty() { word.chars().count() } else { line.chars().count() + 1 + word.chars().count() };
            if need > cols && !line.is_empty() {
                out.push(std::mem::take(&mut line));
            }
            if !line.is_empty() {
                line.push(' ');
            }
            line.push_str(&word);
        }
        out.push(line);
    }
    out
}

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
        ';' => [0x00, 0x30, 0x30, 0x00, 0x30, 0x30, 0x20, 0x40],
        '.' => [0x00, 0x00, 0x00, 0x00, 0x00, 0x30, 0x30, 0x00],
        ',' => [0x00, 0x00, 0x00, 0x00, 0x30, 0x30, 0x20, 0x40],
        '!' => [0x20, 0x20, 0x20, 0x20, 0x20, 0x00, 0x20, 0x00],
        '?' => [0x70, 0x88, 0x10, 0x20, 0x20, 0x00, 0x20, 0x00],
        '-' => [0x00, 0x00, 0x00, 0xF0, 0x00, 0x00, 0x00, 0x00],
        '_' => [0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xF8, 0x00],
        '+' => [0x00, 0x20, 0x20, 0xF8, 0x20, 0x20, 0x00, 0x00],
        '*' => [0x00, 0xA8, 0x70, 0xF8, 0x70, 0xA8, 0x00, 0x00],
        '/' => [0x08, 0x10, 0x20, 0x20, 0x40, 0x80, 0x80, 0x00],
        '\\' => [0x80, 0x40, 0x20, 0x20, 0x10, 0x08, 0x08, 0x00],
        '(' => [0x30, 0x40, 0x40, 0x40, 0x40, 0x40, 0x30, 0x00],
        ')' => [0x60, 0x10, 0x10, 0x10, 0x10, 0x10, 0x60, 0x00],
        '[' => [0x70, 0x40, 0x40, 0x40, 0x40, 0x40, 0x70, 0x00],
        ']' => [0x70, 0x10, 0x10, 0x10, 0x10, 0x10, 0x70, 0x00],
        '<' => [0x10, 0x20, 0x40, 0x80, 0x40, 0x20, 0x10, 0x00],
        '>' => [0x40, 0x20, 0x10, 0x08, 0x10, 0x20, 0x40, 0x00],
        '=' => [0x00, 0x00, 0xF8, 0x00, 0xF8, 0x00, 0x00, 0x00],
        '\'' => [0x20, 0x20, 0x40, 0x00, 0x00, 0x00, 0x00, 0x00],
        '"' => [0x50, 0x50, 0xA0, 0x00, 0x00, 0x00, 0x00, 0x00],
        '%' => [0xC8, 0xC8, 0x10, 0x20, 0x40, 0x98, 0x98, 0x00],
        '#' => [0x50, 0x50, 0xF8, 0x50, 0xF8, 0x50, 0x50, 0x00],
        '&' => [0x60, 0x90, 0xA0, 0x40, 0xA8, 0x90, 0x68, 0x00],
        '@' => [0x70, 0x88, 0xB8, 0xA8, 0xB8, 0x80, 0x78, 0x00],
        '$' => [0x20, 0x78, 0xA0, 0x70, 0x28, 0xF0, 0x20, 0x00],
        '^' => [0x20, 0x50, 0x88, 0x00, 0x00, 0x00, 0x00, 0x00],
        '|' => [0x20, 0x20, 0x20, 0x20, 0x20, 0x20, 0x20, 0x00],
        '~' => [0x00, 0x00, 0x40, 0xA8, 0x10, 0x00, 0x00, 0x00],
        // Small icons: heart, dew drop, right arrow, block (text cursor), dot.
        '\u{2665}' => [0x00, 0x6C, 0xFE, 0xFE, 0x7C, 0x38, 0x10, 0x00],
        '\u{2022}' => [0x00, 0x00, 0x30, 0x78, 0x78, 0x30, 0x00, 0x00],
        '\u{25B6}' => [0x40, 0x60, 0x70, 0x78, 0x70, 0x60, 0x40, 0x00],
        '\u{2588}' => [0xF8, 0xF8, 0xF8, 0xF8, 0xF8, 0xF8, 0xF8, 0x00],
        '\u{25C6}' => [0x20, 0x20, 0x70, 0xF8, 0xF8, 0x70, 0x00, 0x00],
        _ => [0x00; 8],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn integer_scale_matches_documented_table() {
        assert_eq!(Framebuffer::integer_scale_for_viewport(800, 600), 2);
        assert_eq!(Framebuffer::integer_scale_for_viewport(1920, 1080), 3);
        assert_eq!(Framebuffer::integer_scale_for_viewport(2560, 1440), 4);
        assert_eq!(Framebuffer::integer_scale_for_viewport(3840, 2160), 7);
    }

    #[test]
    fn present_uses_exact_blocks_and_bottom_scanline() {
        let mut pal = [0xFF00_0000u32; 64];
        pal[5] = 0xFF80_8080;
        let mut fb = Framebuffer::new();
        fb.pset(10, 20, 5);
        let mut out = Vec::new();
        let (w, _) = fb.present(&mut out, 3, &pal, false);
        for dy in 0..3 {
            for dx in 0..3 {
                assert_eq!(out[(60 + dy) * w + 30 + dx], 0xFF80_8080);
            }
        }
        fb.present(&mut out, 3, &pal, true);
        assert_eq!(out[61 * w + 30], 0xFF80_8080);
        assert_ne!(out[62 * w + 30], 0xFF80_8080);
    }

    #[test]
    fn wrap_breaks_on_words_and_newlines() {
        assert_eq!(wrap("ONE TWO THREE", 7), vec!["ONE TWO", "THREE"]);
        assert_eq!(wrap("A\nB", 10), vec!["A", "B"]);
        assert_eq!(wrap("ABCDEFGHIJ", 4), vec!["ABCD", "EFGH", "IJ"]);
    }
}
