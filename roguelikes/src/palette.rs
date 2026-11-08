//! 64-entry palettes quantized to 6-bit VGA DAC precision. Each game loads its own palette, like
//! a DOS game reprogramming the DAC per level, but indices 1 and 48..63 mean the same thing
//! everywhere so the shared UI (panels, text, the form) draws with one set of constants.

pub type Pal = [u32; 64];
pub type Rgb = (u8, u8, u8);

pub const OUTLINE: u8 = 1;
// Shared UI block.
pub const PANEL_LO: u8 = 48;
pub const PANEL: u8 = 49;
pub const PANEL_HI: u8 = 50;
pub const PANEL_DEEP: u8 = 51;
pub const TEXT: u8 = 52;
pub const TEXT_DIM: u8 = 53;
pub const TEXT_FAINT: u8 = 54;
pub const ACCENT: u8 = 55;
pub const ACCENT_HI: u8 = 56;
pub const GOOD: u8 = 57;
pub const BAD: u8 = 58;
pub const BAD_DIM: u8 = 59;
pub const HEART: u8 = 60;
pub const GOLD: u8 = 61;
pub const HILITE: u8 = 62;
pub const WHITE: u8 = 63;

#[inline]
pub fn argb((r, g, b): Rgb) -> u32 {
    let (r, g, b) = ((r & 0xFC) as u32, (g & 0xFC) as u32, (b & 0xFC) as u32);
    0xFF00_0000 | (r << 16) | (g << 8) | b
}

pub fn set(p: &mut Pal, i: u8, rgb: Rgb) {
    p[i as usize] = argb(rgb);
}

/// Linear ramp of `n` entries from `a` to `b`, darkest first.
pub fn ramp(p: &mut Pal, start: u8, n: u8, a: Rgb, b: Rgb) {
    let n = n as i32;
    for i in 0..n {
        let lerp = |x: u8, y: u8| (x as i32 + (y as i32 - x as i32) * i / (n - 1).max(1)) as u8;
        p[(start as i32 + i) as usize] = argb((lerp(a.0, b.0), lerp(a.1, b.1), lerp(a.2, b.2)));
    }
}

/// Three-step ramp (shadow, base, light).
pub fn ramp3(p: &mut Pal, start: u8, a: Rgb, b: Rgb, c: Rgb) {
    set(p, start, a);
    set(p, start + 1, b);
    set(p, start + 2, c);
}

pub struct UiTheme {
    pub panel: [Rgb; 4], // lo, base, hi, deep
    pub text: Rgb,
    pub text_dim: Rgb,
    pub text_faint: Rgb,
    pub accent: Rgb,
    pub accent_hi: Rgb,
}

pub fn ui_block(p: &mut Pal, t: &UiTheme) {
    set(p, OUTLINE, (10, 8, 14));
    set(p, PANEL_LO, t.panel[0]);
    set(p, PANEL, t.panel[1]);
    set(p, PANEL_HI, t.panel[2]);
    set(p, PANEL_DEEP, t.panel[3]);
    set(p, TEXT, t.text);
    set(p, TEXT_DIM, t.text_dim);
    set(p, TEXT_FAINT, t.text_faint);
    set(p, ACCENT, t.accent);
    set(p, ACCENT_HI, t.accent_hi);
    set(p, GOOD, (104, 224, 120));
    set(p, BAD, (232, 72, 60));
    set(p, BAD_DIM, (120, 36, 40));
    set(p, HEART, (236, 60, 76));
    set(p, GOLD, (248, 204, 72));
    set(p, HILITE, (252, 244, 200));
    set(p, WHITE, (255, 255, 255));
}

/// The launcher and feedback form: a calm blue DOS setup-program look.
pub fn shell() -> Pal {
    let mut p = [argb((0, 0, 0)); 64];
    ramp(&mut p, 2, 6, (4, 6, 24), (28, 40, 104));
    ramp(&mut p, 8, 4, (60, 70, 120), (150, 160, 210));
    ramp3(&mut p, 12, (40, 100, 60), (80, 170, 96), (170, 230, 150)); // forest green for game A
    ramp3(&mut p, 15, (120, 40, 36), (200, 84, 60), (244, 160, 112)); // banner red for game B
    ramp3(&mut p, 18, (32, 64, 140), (72, 120, 220), (160, 196, 248)); // banner blue for game B
    ui_block(
        &mut p,
        &UiTheme {
            panel: [(24, 32, 84), (44, 56, 132), (96, 112, 196), (12, 16, 48)],
            text: (236, 236, 244),
            text_dim: (156, 164, 204),
            text_faint: (92, 100, 156),
            accent: (84, 220, 236),
            accent_hi: (196, 248, 252),
        },
    );
    p
}

#[cfg(test)]
mod tests {
    #[test]
    fn every_entry_is_6bit() {
        for &c in super::shell().iter() {
            assert_eq!(c & 0x0003_0303, 0, "channel not quantized to 6 bits: {c:08x}");
        }
    }
}
