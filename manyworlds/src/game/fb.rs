//! The 400 x 300 indexed framebuffer: integer-only drawing primitives, a 5 x 7 bitmap font, and
//! presentation at an exact integer scale `N` with optional scanlines (AGENTS.md rules 1-4).

use super::art::{rgb, Sprite};

pub const FB_W: i32 = 400;
pub const FB_H: i32 = 300;

pub struct Fb {
    pub px: Vec<u8>,
}

impl Default for Fb {
    fn default() -> Self {
        Fb {
            px: vec![0; (FB_W * FB_H) as usize],
        }
    }
}

impl Fb {
    pub fn clear(&mut self, col: u8) {
        self.px.fill(col);
    }

    pub fn pset(&mut self, x: i32, y: i32, col: u8) {
        if x >= 0 && y >= 0 && x < FB_W && y < FB_H {
            self.px[(y * FB_W + x) as usize] = col;
        }
    }

    pub fn get(&self, x: i32, y: i32) -> u8 {
        if x >= 0 && y >= 0 && x < FB_W && y < FB_H {
            self.px[(y * FB_W + x) as usize]
        } else {
            0
        }
    }

    pub fn rect(&mut self, x: i32, y: i32, w: i32, h: i32, col: u8) {
        for py in y.max(0)..(y + h).min(FB_H) {
            for px in x.max(0)..(x + w).min(FB_W) {
                self.px[(py * FB_W + px) as usize] = col;
            }
        }
    }

    /// A 1 px outline.
    pub fn frame(&mut self, x: i32, y: i32, w: i32, h: i32, col: u8) {
        self.rect(x, y, w, 1, col);
        self.rect(x, y + h - 1, w, 1, col);
        self.rect(x, y, 1, h, col);
        self.rect(x + w - 1, y, 1, h, col);
    }

    /// A dotted 1 px outline: every other pixel, fixed (never animated).
    pub fn frame_dotted(&mut self, x: i32, y: i32, w: i32, h: i32, col: u8) {
        for i in 0..w {
            if (x + i + y) & 1 == 0 {
                self.pset(x + i, y, col);
                self.pset(x + i, y + h - 1, col);
            }
        }
        for j in 0..h {
            if (x + y + j) & 1 == 0 {
                self.pset(x, y + j, col);
                self.pset(x + w - 1, y + j, col);
            }
        }
    }

    /// Checkerboard dither of one color: shadows and dim overlays without alpha blending.
    pub fn checker(&mut self, x: i32, y: i32, w: i32, h: i32, col: u8) {
        for py in y.max(0)..(y + h).min(FB_H) {
            for px in x.max(0)..(x + w).min(FB_W) {
                if (px + py) & 1 == 0 {
                    self.px[(py * FB_W + px) as usize] = col;
                }
            }
        }
    }

    /// A raised panel: ink outline, fill, light top-left edge, dark bottom-right edge.
    pub fn bevel(&mut self, x: i32, y: i32, w: i32, h: i32, fill: u8, hi: u8, lo: u8) {
        self.rect(x, y, w, h, super::art::INK);
        self.rect(x + 1, y + 1, w - 2, h - 2, fill);
        self.rect(x + 1, y + 1, w - 2, 1, hi);
        self.rect(x + 1, y + 1, 1, h - 2, hi);
        self.rect(x + 1, y + h - 2, w - 2, 1, lo);
        self.rect(x + w - 2, y + 1, 1, h - 2, lo);
    }

    /// Integer Bresenham line.
    pub fn line(&mut self, x0: i32, y0: i32, x1: i32, y1: i32, col: u8) {
        let (dx, dy) = ((x1 - x0).abs(), -(y1 - y0).abs());
        let (sx, sy) = (if x0 < x1 { 1 } else { -1 }, if y0 < y1 { 1 } else { -1 });
        let (mut x, mut y, mut err) = (x0, y0, dx + dy);
        loop {
            self.pset(x, y, col);
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

    pub fn blit(&mut self, s: &Sprite, x: i32, y: i32) {
        for sy in 0..s.h {
            for sx in 0..s.w {
                let c = s.px[(sy * s.w + sx) as usize];
                if c != 0 {
                    self.pset(x + sx, y + sy, c);
                }
            }
        }
    }

    /// A filled shape centred on (cx, cy) with an ink hull: 0 circle, 1 diamond, 2 square.
    /// `r` is the radius in pixels.
    pub fn shape(&mut self, kind: u8, cx: i32, cy: i32, r: i32, fill: u8) {
        let inside = |dx: i32, dy: i32, r: i32| match kind {
            0 => dx * dx + dy * dy <= r * r + r,
            1 => dx.abs() + dy.abs() <= r,
            _ => dx.abs() < r && dy.abs() < r,
        };
        for dy in -r - 1..=r + 1 {
            for dx in -r - 1..=r + 1 {
                if inside(dx, dy, r) {
                    self.pset(cx + dx, cy + dy, fill);
                } else if inside(dx + 1, dy, r)
                    || inside(dx - 1, dy, r)
                    || inside(dx, dy + 1, r)
                    || inside(dx, dy - 1, r)
                {
                    self.pset(cx + dx, cy + dy, super::art::INK);
                }
            }
        }
    }

    /// Text in the 5 x 7 font at integer `scale`, 6 px advance per character at scale 1.
    pub fn text(&mut self, s: &str, x: i32, y: i32, col: u8, scale: i32) {
        let mut cx = x;
        for ch in s.chars() {
            let g = glyph(ch);
            for (row, bits) in g.iter().enumerate() {
                for bit in 0..5 {
                    if bits & (0x10 >> bit) != 0 {
                        self.rect(cx + bit * scale, y + row as i32 * scale, scale, scale, col);
                    }
                }
            }
            cx += 6 * scale;
        }
    }

    /// Text with a 1 px ink shadow down-right, for reading over the playfield.
    pub fn text_shadow(&mut self, s: &str, x: i32, y: i32, col: u8, scale: i32) {
        self.text(
            s,
            x + scale.max(1),
            y + scale.max(1),
            super::art::INK,
            scale,
        );
        self.text(s, x, y, col, scale);
    }

    pub fn text_width(s: &str, scale: i32) -> i32 {
        (s.chars().count() as i32 * 6 - 1) * scale
    }

    pub fn text_centered(&mut self, s: &str, cx: i32, y: i32, col: u8, scale: i32) {
        self.text_shadow(s, cx - Self::text_width(s, scale) / 2, y, col, scale);
    }

    /// Expand into an exact (400 N) x (300 N) RGBA image: every logical pixel an N x N block.
    /// With scanlines (and N >= 2) the bottom physical row of each block is darkened by ~22%.
    pub fn present(&self, out: &mut Vec<u8>, n: usize, scanlines: bool) {
        let n = n.max(1);
        let w = FB_W as usize * n;
        out.resize(w * FB_H as usize * n * 4, 255);
        let mut lut = [[0u8; 4]; 256];
        let mut dim = [[0u8; 4]; 256];
        for i in 0..256 {
            let c = rgb(i as u8);
            let (r, g, b) = ((c >> 16) as u8, (c >> 8) as u8, c as u8);
            lut[i] = [r, g, b, 255];
            let d = |v: u8| ((v as u32 * 200) >> 8) as u8 & 0xFC;
            dim[i] = [d(r), d(g), d(b), 255];
        }
        let lines = scanlines && n >= 2;
        for y in 0..FB_H as usize {
            let row0 = y * n * w * 4;
            for x in 0..FB_W as usize {
                let c = lut[self.px[y * FB_W as usize + x] as usize];
                let d = row0 + x * n * 4;
                for k in 0..n {
                    out[d + k * 4..d + k * 4 + 4].copy_from_slice(&c);
                }
            }
            let full = if lines { n - 1 } else { n };
            for sub in 1..full {
                out.copy_within(row0..row0 + w * 4, row0 + sub * w * 4);
            }
            if lines {
                let last = row0 + (n - 1) * w * 4;
                for x in 0..FB_W as usize {
                    let c = dim[self.px[y * FB_W as usize + x] as usize];
                    let d = last + x * n * 4;
                    for k in 0..n {
                        out[d + k * 4..d + k * 4 + 4].copy_from_slice(&c);
                    }
                }
            }
        }
    }
}

/// Largest exact integer scale of the 400 x 300 framebuffer that fits `w` x `h` physical pixels.
pub fn integer_scale(w: i32, h: i32) -> i32 {
    (w / FB_W).min(h / FB_H).max(1)
}

/// 5 x 7 glyphs, one byte per row, bit 4 is the leftmost column.
fn glyph(ch: char) -> [u8; 7] {
    match ch.to_ascii_uppercase() {
        'A' => [0x0E, 0x11, 0x11, 0x1F, 0x11, 0x11, 0x11],
        'B' => [0x1E, 0x11, 0x11, 0x1E, 0x11, 0x11, 0x1E],
        'C' => [0x0E, 0x11, 0x10, 0x10, 0x10, 0x11, 0x0E],
        'D' => [0x1C, 0x12, 0x11, 0x11, 0x11, 0x12, 0x1C],
        'E' => [0x1F, 0x10, 0x10, 0x1E, 0x10, 0x10, 0x1F],
        'F' => [0x1F, 0x10, 0x10, 0x1E, 0x10, 0x10, 0x10],
        'G' => [0x0E, 0x11, 0x10, 0x17, 0x11, 0x11, 0x0E],
        'H' => [0x11, 0x11, 0x11, 0x1F, 0x11, 0x11, 0x11],
        'I' => [0x0E, 0x04, 0x04, 0x04, 0x04, 0x04, 0x0E],
        'J' => [0x07, 0x02, 0x02, 0x02, 0x12, 0x12, 0x0C],
        'K' => [0x11, 0x12, 0x14, 0x18, 0x14, 0x12, 0x11],
        'L' => [0x10, 0x10, 0x10, 0x10, 0x10, 0x10, 0x1F],
        'M' => [0x11, 0x1B, 0x15, 0x15, 0x11, 0x11, 0x11],
        'N' => [0x11, 0x19, 0x15, 0x13, 0x11, 0x11, 0x11],
        'O' => [0x0E, 0x11, 0x11, 0x11, 0x11, 0x11, 0x0E],
        'P' => [0x1E, 0x11, 0x11, 0x1E, 0x10, 0x10, 0x10],
        'Q' => [0x0E, 0x11, 0x11, 0x11, 0x15, 0x12, 0x0D],
        'R' => [0x1E, 0x11, 0x11, 0x1E, 0x14, 0x12, 0x11],
        'S' => [0x0F, 0x10, 0x10, 0x0E, 0x01, 0x01, 0x1E],
        'T' => [0x1F, 0x04, 0x04, 0x04, 0x04, 0x04, 0x04],
        'U' => [0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x0E],
        'V' => [0x11, 0x11, 0x11, 0x11, 0x0A, 0x0A, 0x04],
        'W' => [0x11, 0x11, 0x11, 0x15, 0x15, 0x1B, 0x11],
        'X' => [0x11, 0x11, 0x0A, 0x04, 0x0A, 0x11, 0x11],
        'Y' => [0x11, 0x11, 0x0A, 0x04, 0x04, 0x04, 0x04],
        'Z' => [0x1F, 0x01, 0x02, 0x04, 0x08, 0x10, 0x1F],
        '0' => [0x0E, 0x11, 0x13, 0x15, 0x19, 0x11, 0x0E],
        '1' => [0x04, 0x0C, 0x04, 0x04, 0x04, 0x04, 0x0E],
        '2' => [0x0E, 0x11, 0x01, 0x06, 0x08, 0x10, 0x1F],
        '3' => [0x0E, 0x11, 0x01, 0x06, 0x01, 0x11, 0x0E],
        '4' => [0x02, 0x06, 0x0A, 0x12, 0x1F, 0x02, 0x02],
        '5' => [0x1F, 0x10, 0x1E, 0x01, 0x01, 0x11, 0x0E],
        '6' => [0x07, 0x08, 0x10, 0x1E, 0x11, 0x11, 0x0E],
        '7' => [0x1F, 0x01, 0x02, 0x04, 0x08, 0x08, 0x08],
        '8' => [0x0E, 0x11, 0x11, 0x0E, 0x11, 0x11, 0x0E],
        '9' => [0x0E, 0x11, 0x11, 0x0F, 0x01, 0x02, 0x1C],
        ':' => [0x00, 0x0C, 0x0C, 0x00, 0x0C, 0x0C, 0x00],
        '.' => [0x00, 0x00, 0x00, 0x00, 0x00, 0x0C, 0x0C],
        ',' => [0x00, 0x00, 0x00, 0x00, 0x0C, 0x04, 0x08],
        '!' => [0x04, 0x04, 0x04, 0x04, 0x04, 0x00, 0x04],
        '-' => [0x00, 0x00, 0x00, 0x1F, 0x00, 0x00, 0x00],
        '+' => [0x00, 0x04, 0x04, 0x1F, 0x04, 0x04, 0x00],
        '/' => [0x01, 0x02, 0x02, 0x04, 0x08, 0x08, 0x10],
        '\'' => [0x04, 0x04, 0x08, 0x00, 0x00, 0x00, 0x00],
        'x' | '×' => [0x00, 0x11, 0x0A, 0x04, 0x0A, 0x11, 0x00],
        _ => [0x00; 7],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn integer_scale_fits_the_screen() {
        assert_eq!(integer_scale(800, 600), 2);
        assert_eq!(integer_scale(1920, 1080), 3);
        assert_eq!(integer_scale(2560, 1440), 4);
        assert_eq!(integer_scale(3840, 2160), 7);
        assert_eq!(integer_scale(300, 200), 1);
    }

    #[test]
    fn present_makes_exact_blocks_and_darkens_only_the_bottom_row() {
        let mut fb = Fb::default();
        fb.pset(10, 20, super::super::art::TEXT);
        let mut out = vec![];
        fb.present(&mut out, 3, false);
        let w = 400 * 3;
        let px = |out: &Vec<u8>, x: usize, y: usize| out[(y * w + x) * 4];
        for dy in 0..3 {
            for dx in 0..3 {
                assert_eq!(
                    px(&out, 30 + dx, 60 + dy),
                    (rgb(super::super::art::TEXT) >> 16) as u8
                );
            }
        }
        fb.present(&mut out, 3, true);
        assert_eq!(px(&out, 30, 61), (rgb(super::super::art::TEXT) >> 16) as u8);
        assert!(px(&out, 30, 62) < px(&out, 30, 61));
        assert_eq!(px(&out, 30, 62) & 3, 0, "still at DAC precision");
    }
}
