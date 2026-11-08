//! Shared widgets: rectangles, beveled panels, buttons and the two mouse pointers.

use crate::framebuffer::{Framebuffer, Sprite};
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

/// A raised button; highlighted when hovered, sunk when `down`.
pub fn button(fb: &mut Framebuffer, r: Rect, label: &str, hover: bool, down: bool) {
    if down {
        fb.rect_bevel(r.x, r.y, r.w, r.h, PANEL_DEEP, PANEL_LO, PANEL_HI);
    } else if hover {
        fb.rect_bevel(r.x, r.y, r.w, r.h, PANEL_HI, WHITE, PANEL);
    } else {
        fb.rect_bevel(r.x, r.y, r.w, r.h, PANEL, PANEL_HI, PANEL_LO);
    }
    let col = if down {
        ACCENT
    } else if hover {
        PANEL_DEEP
    } else {
        TEXT
    };
    fb.text_centered(label, r.x + r.w / 2, r.y + (r.h - 7) / 2, col, None, 1);
}

/// The DOS arrow pointer (hotspot at its tip).
pub fn pointer() -> Sprite {
    Sprite::art(
        &[
            "k.........",
            "kk........",
            "kWk.......",
            "kWWk......",
            "kWWWk.....",
            "kWWWWk....",
            "kWWWWWk...",
            "kWWWWWWk..",
            "kWWWWWWWk.",
            "kWWWWWkkkk",
            "kWWkWWk...",
            "kWk.kWWk..",
            "kk..kWWk..",
            "k....kWWk.",
            ".....kWWk.",
            "......kk..",
        ],
        |c| match c {
            'k' => OUTLINE,
            'W' => WHITE,
            _ => 0,
        },
    )
}

/// The planting crosshair (hotspot at its centre, pixel 5,5).
pub fn crosshair() -> Sprite {
    Sprite::art(
        &[
            "....kWk....",
            "....kWk....",
            "....kWk....",
            "...........",
            "kkk.....kkk",
            "WWW..W..WWW",
            "kkk.....kkk",
            "...........",
            "....kWk....",
            "....kWk....",
            "....kWk....",
        ],
        |c| match c {
            'k' => OUTLINE,
            'W' => WHITE,
            _ => 0,
        },
    )
}
