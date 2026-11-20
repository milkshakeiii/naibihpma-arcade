pub mod font;
pub mod palette;
pub mod render_game;
pub mod sprites;

use font::glyph_5x7;
use palette::build_palette_for_tick;

pub const FB_W: usize = 400;
pub const FB_H: usize = 300;
pub const FB_LEN: usize = FB_W * FB_H;

pub struct Framebuffer {
    pub pixels: [u8; FB_LEN],
}

impl Default for Framebuffer {
    fn default() -> Self {
        Self::new()
    }
}

impl Framebuffer {
    pub fn new() -> Self {
        Self {
            pixels: [0; FB_LEN],
        }
    }

    pub fn clear(&mut self, color: u8) {
        self.pixels.fill(color);
    }

    #[inline]
    pub fn set_px(&mut self, x: i32, y: i32, color: u8) {
        if color == 0 || x < 0 || y < 0 || x >= FB_W as i32 || y >= FB_H as i32 {
            return;
        }
        self.pixels[(y as usize) * FB_W + (x as usize)] = color;
    }

    pub fn rect(&mut self, x: i32, y: i32, w: i32, h: i32, color: u8) {
        if w <= 0 || h <= 0 || color == 0 {
            return;
        }
        let x0 = x.max(0);
        let y0 = y.max(0);
        let x1 = (x + w).min(FB_W as i32);
        let y1 = (y + h).min(FB_H as i32);
        if x0 >= x1 || y0 >= y1 {
            return;
        }
        for py in y0..y1 {
            let row = (py as usize) * FB_W;
            self.pixels[(row + x0 as usize)..(row + x1 as usize)].fill(color);
        }
    }

    pub fn rect_outline(&mut self, x: i32, y: i32, w: i32, h: i32, color: u8) {
        if w <= 0 || h <= 0 {
            return;
        }
        self.rect(x, y, w, 1, color);
        self.rect(x, y + h - 1, w, 1, color);
        self.rect(x, y, 1, h, color);
        self.rect(x + w - 1, y, 1, h, color);
    }

    /// Apogee / Dune 2000 style 3D beveled frame
    pub fn rect_bevel(&mut self, x: i32, y: i32, w: i32, h: i32, fill: u8, hi: u8, lo: u8) {
        self.rect(x, y, w, h, fill);
        if w >= 2 && h >= 2 {
            self.rect(x, y, w, 1, hi);
            self.rect(x, y, 1, h, hi);
            self.rect(x, y + h - 1, w, 1, lo);
            self.rect(x + w - 1, y, 1, h, lo);
        }
    }

    /// Classic DOS 2x2 Bayer checkerboard dithered rectangle (zero alpha blending)
    pub fn rect_dither(&mut self, x: i32, y: i32, w: i32, h: i32, color: u8, phase: i32) {
        if w <= 0 || h <= 0 || color == 0 {
            return;
        }
        let x0 = x.max(0);
        let y0 = y.max(0);
        let x1 = (x + w).min(FB_W as i32);
        let y1 = (y + h).min(FB_H as i32);
        for py in y0..y1 {
            for px in x0..x1 {
                if ((px + py + phase) & 1) == 0 {
                    self.pixels[(py as usize) * FB_W + (px as usize)] = color;
                }
            }
        }
    }

    /// 25% sparse Bayer dithered rectangle
    pub fn rect_dither_sparse(&mut self, x: i32, y: i32, w: i32, h: i32, color: u8) {
        if w <= 0 || h <= 0 || color == 0 {
            return;
        }
        let x0 = x.max(0);
        let y0 = y.max(0);
        let x1 = (x + w).min(FB_W as i32);
        let y1 = (y + h).min(FB_H as i32);
        for py in y0..y1 {
            for px in x0..x1 {
                if (px & 1) == 0 && (py & 1) == 0 {
                    self.pixels[(py as usize) * FB_W + (px as usize)] = color;
                }
            }
        }
    }

    /// Integer Bresenham line
    pub fn line_bresenham(&mut self, mut x0: i32, mut y0: i32, x1: i32, y1: i32, color: u8) {
        let dx = (x1 - x0).abs();
        let dy = -(y1 - y0).abs();
        let sx = if x0 < x1 { 1 } else { -1 };
        let sy = if y0 < y1 { 1 } else { -1 };
        let mut err = dx + dy;
        loop {
            self.set_px(x0, y0, color);
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

    /// Dashed integer Bresenham line (used for Ley links and range rings)
    pub fn line_dashed(
        &mut self,
        mut x0: i32,
        mut y0: i32,
        x1: i32,
        y1: i32,
        color: u8,
        phase: i32,
    ) {
        let dx = (x1 - x0).abs();
        let dy = -(y1 - y0).abs();
        let sx = if x0 < x1 { 1 } else { -1 };
        let sy = if y0 < y1 { 1 } else { -1 };
        let mut err = dx + dy;
        let mut step = 0i32;
        loop {
            if ((step + phase) % 4) < 2 {
                self.set_px(x0, y0, color);
            }
            if x0 == x1 && y0 == y1 {
                break;
            }
            step += 1;
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

    /// Midpoint integer circle outline
    pub fn circle_int(&mut self, cx: i32, cy: i32, r: i32, color: u8) {
        if r <= 0 {
            return;
        }
        let mut x = r;
        let mut y = 0;
        let mut err = 1 - x;
        while x >= y {
            self.set_px(cx + x, cy + y, color);
            self.set_px(cx + y, cy + x, color);
            self.set_px(cx - y, cy + x, color);
            self.set_px(cx - x, cy + y, color);
            self.set_px(cx - x, cy - y, color);
            self.set_px(cx - y, cy - x, color);
            self.set_px(cx + y, cy - x, color);
            self.set_px(cx + x, cy - y, color);
            y += 1;
            if err < 0 {
                err += 2 * y + 1;
            } else {
                x -= 1;
                err += 2 * (y - x) + 1;
            }
        }
    }

    pub fn text(&mut self, x: i32, y: i32, s: &str, color: u8, scale: i32) {
        let sc = scale.max(1);
        let mut cx = x;
        for &b in s.as_bytes() {
            if b != b' ' {
                let rows = glyph_5x7(b);
                for (ry, &bits) in rows.iter().enumerate() {
                    for rx in 0..5 {
                        if (bits & (1 << (4 - rx))) != 0 {
                            if sc == 1 {
                                self.set_px(cx + rx, y + ry as i32, color);
                            } else {
                                self.rect(cx + rx * sc, y + (ry as i32) * sc, sc, sc, color);
                            }
                        }
                    }
                }
            }
            cx += 6 * sc;
        }
    }

    pub fn text_shadow(&mut self, x: i32, y: i32, s: &str, color: u8, shadow: u8, scale: i32) {
        let sc = scale.max(1);
        self.text(x + sc, y + sc, s, shadow, sc);
        self.text(x, y, s, color, sc);
    }

    pub fn text_centered(&mut self, cx: i32, y: i32, s: &str, color: u8, shadow: u8, scale: i32) {
        let sc = scale.max(1);
        let w = (s.len() as i32) * 6 * sc - sc;
        self.text_shadow(cx - w / 2, y, s, color, shadow, sc);
    }

    pub fn blit_sprite(&mut self, dx: i32, dy: i32, w: usize, h: usize, data: &[u8]) {
        for sy in 0..h {
            let py = dy + sy as i32;
            if py < 0 || py >= FB_H as i32 {
                continue;
            }
            for sx in 0..w {
                let px = dx + sx as i32;
                if px < 0 || px >= FB_W as i32 {
                    continue;
                }
                let c = data[sy * w + sx];
                if c != 0 {
                    self.pixels[(py as usize) * FB_W + (px as usize)] = c;
                }
            }
        }
    }

    /// Compute largest exact integer scale N >= 1 fitting inside viewport (cw x ch)
    pub fn integer_scale_for_viewport(cw: i32, ch: i32) -> i32 {
        let sx = cw / (FB_W as i32);
        let sy = ch / (FB_H as i32);
        sx.min(sy).max(1)
    }

    /// Expand 400x300 indexed canvas into exact (400*N) x (300*N) 32-bit XRGB buffer
    /// with physical-scale toggleable CRT scanlines on bottom sub-row (sub_y == N - 1).
    pub fn present_integer_scaled(
        &self,
        scale: i32,
        scanlines: bool,
        tick: u32,
        out_bgra: &mut Vec<u32>,
    ) {
        let n = scale.max(1) as usize;
        let out_w = FB_W * n;
        let out_h = FB_H * n;
        out_bgra.resize(out_w * out_h, 0);

        let lut = build_palette_for_tick(tick);
        let mut scan_lut = [0u32; 64];
        for i in 0..64 {
            let c = lut[i];
            let r = (((c >> 16) & 0xFF) * 198) >> 8;
            let g = (((c >> 8) & 0xFF) * 198) >> 8;
            let b = ((c & 0xFF) * 204) >> 8;
            scan_lut[i] = (r << 16) | (g << 8) | b;
        }

        let apply_scan = scanlines && n >= 2;

        for ly in 0..FB_H {
            let src_row = &self.pixels[(ly * FB_W)..((ly + 1) * FB_W)];
            for sub_y in 0..n {
                let dst_y = ly * n + sub_y;
                let dst_row = &mut out_bgra[(dst_y * out_w)..((dst_y + 1) * out_w)];
                let active_lut = if apply_scan && sub_y == n - 1 {
                    &scan_lut
                } else {
                    &lut
                };
                if n == 1 {
                    for lx in 0..FB_W {
                        dst_row[lx] = active_lut[(src_row[lx] & 63) as usize];
                    }
                } else {
                    for lx in 0..FB_W {
                        let col = active_lut[(src_row[lx] & 63) as usize];
                        let base = lx * n;
                        for sub_x in 0..n {
                            dst_row[base + sub_x] = col;
                        }
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_integer_scale_math() {
        assert_eq!(Framebuffer::integer_scale_for_viewport(800, 600), 2);
        assert_eq!(Framebuffer::integer_scale_for_viewport(1280, 720), 2);
        assert_eq!(Framebuffer::integer_scale_for_viewport(1920, 1080), 3);
        assert_eq!(Framebuffer::integer_scale_for_viewport(2560, 1440), 4);
        assert_eq!(Framebuffer::integer_scale_for_viewport(3840, 2160), 7);
        assert_eq!(Framebuffer::integer_scale_for_viewport(320, 200), 1);
    }

    #[test]
    fn test_scanline_subrow_darkening() {
        let mut fb = Framebuffer::new();
        fb.clear(22); // Crisp cream text white
        let mut out = Vec::new();
        fb.present_integer_scaled(3, true, 0, &mut out);
        assert_eq!(out.len(), (FB_W * 3) * (FB_H * 3));
        // Top 2 physical sub-rows full brightness; bottom sub-row ~22% darkened
        let top = out[0];
        let mid = out[FB_W * 3];
        let bot = out[FB_W * 6];
        assert_eq!(top, mid);
        assert!(bot < top);

        // With scanlines disabled, all 3 sub-rows are solid undivided blocks
        fb.present_integer_scaled(3, false, 0, &mut out);
        assert_eq!(out[0], out[FB_W * 6]);
    }
}
