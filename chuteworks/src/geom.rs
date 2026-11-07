//! Screen geometry on the 400x300 logical canvas, shared by the simulation (which animates in
//! screen pixels) and the renderer.

use crate::model::Board;

pub const TILE: i32 = 20;
pub const LB_X: i32 = 16;
pub const RB_X: i32 = 224;
pub const BOARD_Y: i32 = 40;
pub const BOARD_PX: i32 = TILE * 8;
pub const GUTTER_X0: i32 = LB_X + BOARD_PX; // 176
pub const GUTTER_X1: i32 = RB_X; // 224
pub const SLOT_W: i32 = 9;

pub fn board_x(b: Board) -> i32 {
    match b {
        Board::Left => LB_X,
        Board::Right => RB_X,
    }
}

/// Top-left pixel of a cell.
pub fn cell_px(b: Board, x: i32, y: i32) -> (i32, i32) {
    (board_x(b) + x * TILE, BOARD_Y + y * TILE)
}

pub fn cell_center(b: Board, x: i32, y: i32) -> (i32, i32) {
    let (px, py) = cell_px(b, x, y);
    (px + TILE / 2, py + TILE / 2)
}

pub fn row_center_y(row: i32) -> i32 {
    BOARD_Y + row * TILE + TILE / 2
}

/// Center of a mini item inside a chute tube; slot 0 is next to the gate (first out).
pub fn chute_slot(row: i32, slot: usize) -> (i32, i32) {
    (GUTTER_X1 - 7 - SLOT_W * slot as i32, row_center_y(row))
}

/// Where a released item starts: the chute's gate.
pub fn chute_gate(row: i32) -> (i32, i32) {
    (GUTTER_X1 - 4, row_center_y(row))
}

/// Integer interpolation between two pixel points; `num/den` in 0..=1.
pub fn lerp(a: (i32, i32), b: (i32, i32), num: i32, den: i32) -> (i32, i32) {
    let den = den.max(1);
    (a.0 + (b.0 - a.0) * num / den, a.1 + (b.1 - a.1) * num / den)
}
