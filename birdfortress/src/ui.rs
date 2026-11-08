//! Shared widgets: rectangles, beveled panels and buttons.

use crate::framebuffer::Framebuffer;
use crate::palette::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub struct Rect {
    pub x: i32,
    pub y: i32,
    pub w: i32,
    pub h: i32,
}

impl Rect {
    pub const fn new(x: i32, y: i32, w: i32, h: i32) -> Self {
        Self { x, y, w, h }
    }
    pub fn contains(&self, px: i32, py: i32) -> bool {
        px >= self.x && py >= self.y && px < self.x + self.w && py < self.y + self.h
    }
}

pub fn panel(fb: &mut Framebuffer, r: Rect) {
    fb.rect_bevel(r.x, r.y, r.w, r.h, PANEL, PANEL_HI, PANEL_LO);
}

pub fn inset(fb: &mut Framebuffer, r: Rect) {
    fb.rect_bevel(r.x, r.y, r.w, r.h, PANEL_DEEP, PANEL_LO, PANEL_HI);
}

/// A raised button face; highlighted when hovered, sunk when `down`. Returns the label colour.
pub fn button_face(fb: &mut Framebuffer, r: Rect, hover: bool, down: bool) -> u8 {
    if down {
        fb.rect_bevel(r.x, r.y, r.w, r.h, PANEL_DEEP, PANEL_LO, PANEL_HI);
        ACCENT_HI
    } else if hover {
        fb.rect_bevel(r.x, r.y, r.w, r.h, PANEL_HI, WHITE, PANEL);
        PANEL_DEEP
    } else {
        fb.rect_bevel(r.x, r.y, r.w, r.h, PANEL, PANEL_HI, PANEL_LO);
        TEXT
    }
}

pub fn button(fb: &mut Framebuffer, r: Rect, label: &str, hover: bool, down: bool) {
    let col = button_face(fb, r, hover, down);
    fb.text_centered(label, r.x + r.w / 2, r.y + (r.h - 7) / 2, col, None, 1);
}

/// A dimmed, unclickable button.
pub fn button_off(fb: &mut Framebuffer, r: Rect, label: &str) {
    fb.rect_bevel(r.x, r.y, r.w, r.h, PANEL_LO, PANEL, PANEL_DEEP);
    fb.text_centered(label, r.x + r.w / 2, r.y + (r.h - 7) / 2, TEXT_FAINT, None, 1);
}

/// Dims everything with a checkerboard (no alpha blending).
pub fn dim(fb: &mut Framebuffer, r: Rect) {
    fb.rect_checker(r.x, r.y, r.w, r.h, SHADOW, 0);
}

/// A five-point star, 7x7, filled or hollow.
pub fn star(fb: &mut Framebuffer, x: i32, y: i32, filled: bool) {
    const ROWS: [u8; 7] = [0b0001000, 0b0001000, 0b1111111, 0b0111110, 0b0011100, 0b0110110, 0b1100011];
    for (r, bits) in ROWS.iter().enumerate() {
        for c in 0..7 {
            if bits & (0x40 >> c) != 0 {
                fb.pset(x + c, y + r as i32, if filled { GOLD } else { PANEL_LO });
            }
        }
    }
}
