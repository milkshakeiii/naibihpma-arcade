//! Screen geometry on the 400x300 logical canvas.

pub const TILE: i32 = 20;
pub const LB_X: i32 = 16;
pub const RB_X: i32 = 224;
pub const BOARD_Y: i32 = 40;
pub const BOARD_PX: i32 = TILE * 8;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Board {
    Kitchen,
    Dining,
}

pub fn board_x(b: Board) -> i32 {
    match b {
        Board::Kitchen => LB_X,
        Board::Dining => RB_X,
    }
}

pub fn cell_px(b: Board, x: i32, y: i32) -> (i32, i32) {
    (board_x(b) + x * TILE, BOARD_Y + y * TILE)
}

pub fn cell_center(b: Board, x: i32, y: i32) -> (i32, i32) {
    let (px, py) = cell_px(b, x, y);
    (px + TILE / 2, py + TILE / 2)
}

/// Integer interpolation between two points; `num/den` in 0..=1.
pub fn lerp(a: (i32, i32), b: (i32, i32), num: i32, den: i32) -> (i32, i32) {
    let den = den.max(1);
    (a.0 + (b.0 - a.0) * num / den, a.1 + (b.1 - a.1) * num / den)
}
