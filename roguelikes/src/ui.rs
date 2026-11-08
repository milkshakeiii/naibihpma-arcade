//! Shared widgets: buttons, panels, shape diagrams and the mouse pointer.

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

/// A raised button; pressed-looking (inset) when `down`, highlighted text when hovered.
pub fn button(fb: &mut Framebuffer, r: Rect, label: &str, hover: bool, enabled: bool) {
    if enabled && hover {
        fb.rect_bevel(r.x, r.y, r.w, r.h, PANEL_HI, WHITE, PANEL);
    } else {
        fb.rect_bevel(r.x, r.y, r.w, r.h, PANEL, PANEL_HI, PANEL_LO);
    }
    let col = if !enabled {
        TEXT_FAINT
    } else if hover {
        PANEL_DEEP
    } else {
        TEXT
    };
    fb.text_centered(label, r.x + r.w / 2, r.y + (r.h - 7) / 2, col, None, 1);
}

/// A shape diagram: a (2r+1)^2 grid of `cell`-pixel squares, the piece in the middle, and its
/// reach filled. `marks` are offsets from the piece; `ahead_up` draws a small arrow above.
pub fn pattern(fb: &mut Framebuffer, x: i32, y: i32, r: i32, cell: i32, marks: &[(i32, i32)], fill: u8, piece: u8) {
    let n = 2 * r + 1;
    fb.rect_fill(x, y, n * cell + 1, n * cell + 1, OUTLINE);
    for gy in 0..n {
        for gx in 0..n {
            let (dx, dy) = (gx - r, gy - r);
            let base = if (gx + gy) % 2 == 0 { PANEL_LO } else { PANEL_DEEP };
            let c = if dx == 0 && dy == 0 {
                piece
            } else if marks.contains(&(dx, dy)) {
                fill
            } else {
                base
            };
            fb.rect_fill(x + 1 + gx * cell, y + 1 + gy * cell, cell - 1, cell - 1, c);
        }
    }
}

/// The DOS mouse pointer (turned into a hardware cursor at the integer scale).
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

/// Draws hearts left to right: filled for `have`, outlined for the rest of `max`.
pub fn hearts(fb: &mut Framebuffer, x: i32, y: i32, have: i32, max: i32) {
    for i in 0..max {
        let col = if i < have { HEART } else { BAD_DIM };
        fb.text("\u{2665}", x + i * 9, y, col, Some(OUTLINE), 1);
    }
}
