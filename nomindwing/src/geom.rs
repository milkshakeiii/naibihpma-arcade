//! Integer geometry: a Bresenham walker, distances and a 16-point circle table.

/// Walks a Bresenham line one pixel per step. It keeps going past the end point on the same
/// slope, so shots fly on in a straight line.
#[derive(Clone, Copy, Debug, Hash)]
pub struct Line {
    pub x: i32,
    pub y: i32,
    dx: i32,
    dy: i32,
    sx: i32,
    sy: i32,
    err: i32,
}

impl Line {
    pub fn new(x0: i32, y0: i32, x1: i32, y1: i32) -> Self {
        let dx = (x1 - x0).abs();
        let dy = -(y1 - y0).abs();
        Self { x: x0, y: y0, dx, dy, sx: if x0 < x1 { 1 } else { -1 }, sy: if y0 < y1 { 1 } else { -1 }, err: dx + dy }
    }

    pub fn step(&mut self) {
        if self.dx == 0 && self.dy == 0 {
            return;
        }
        let e2 = 2 * self.err;
        if e2 >= self.dy {
            self.err += self.dy;
            self.x += self.sx;
        }
        if e2 <= self.dx {
            self.err += self.dx;
            self.y += self.sy;
        }
    }

    /// The sign of travel on each axis.
    pub fn dir(&self) -> (i32, i32) {
        (if self.dx == 0 { 0 } else { self.sx }, if self.dy == 0 { 0 } else { self.sy })
    }
}

/// Takes up to `n` Bresenham steps from (x, y) toward (tx, ty), stopping on the target.
pub fn step_toward(x: i32, y: i32, tx: i32, ty: i32, n: i32) -> (i32, i32) {
    let mut l = Line::new(x, y, tx, ty);
    for _ in 0..n.max(0) {
        if l.x == tx && l.y == ty {
            break;
        }
        l.step();
    }
    (l.x, l.y)
}

/// Chessboard distance (a diagonal step counts as one).
pub fn cheb(ax: i32, ay: i32, bx: i32, by: i32) -> i32 {
    (ax - bx).abs().max((ay - by).abs())
}

pub fn dist2(ax: i32, ay: i32, bx: i32, by: i32) -> i32 {
    let (dx, dy) = (ax - bx, ay - by);
    dx * dx + dy * dy
}

/// cos(k * 22.5 degrees) * 1024.
pub const COS16: [i32; 16] = [1024, 946, 724, 392, 0, -392, -724, -946, -1024, -946, -724, -392, 0, 392, 724, 946];

/// Point `a` of 16 on a circle of radius `r` (y grows downward).
pub fn on_circle(cx: i32, cy: i32, r: i32, a: usize) -> (i32, i32) {
    (cx + r * COS16[a % 16] / 1024, cy + r * COS16[(a + 12) % 16] / 1024)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn step_toward_reaches_and_stops() {
        assert_eq!(step_toward(0, 0, 3, 7, 100), (3, 7));
        assert_eq!(step_toward(5, 5, 5, 5, 3), (5, 5));
        assert_eq!(step_toward(0, 0, 10, 0, 4), (4, 0));
    }

    #[test]
    fn lines_continue_past_the_end() {
        let mut l = Line::new(0, 0, 2, 1);
        for _ in 0..4 {
            l.step();
        }
        assert_eq!((l.x, l.y), (4, 2));
    }

    #[test]
    fn circle_table_is_round() {
        assert_eq!(on_circle(0, 0, 10, 0), (10, 0));
        assert_eq!(on_circle(0, 0, 10, 4), (0, 10));
        assert_eq!(on_circle(0, 0, 10, 8), (-10, 0));
    }
}
