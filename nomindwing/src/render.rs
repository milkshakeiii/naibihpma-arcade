//! Drawing the world view, the minimap and the tactical view: integer pixels, indexed colour,
//! a 1px dark hull around every ship, and checkerboard "blueprint" dots for pixels not yet built.

use crate::design::{info, Design, Kind, Part};
use crate::framebuffer::Framebuffer;
use crate::geom::Line;
use crate::palette::*;
use crate::ui::Rect;
use crate::world::*;

pub const VIEW: Rect = Rect::new(0, 0, 304, 300);
pub const TAC_RECT: Rect = Rect::new((304 - TAC_W) / 2, (300 - TAC_H) / 2 + 6, TAC_W, TAC_H);

#[inline]
fn pc(fb: &mut Framebuffer, clip: Rect, x: i32, y: i32, c: u8) {
    if clip.contains(x, y) {
        fb.pset(x, y, c);
    }
}

pub fn part_col(p: Part, team: usize) -> u8 {
    match p {
        Part::Hull => team_col(team, 1),
        Part::Armor => STEEL,
        Part::Gun => GUN,
        Part::Engine => ENGINE,
        Part::Scoop => SCOOP,
        Part::Core => CORE,
    }
}

/// Draws a design centred on screen point (x, y). `solid` cells get the outline and their part
/// colour; `ghost` cells get blueprint dots in the team's dark shade.
pub fn draw_design(fb: &mut Framebuffer, clip: Rect, d: &Design, x: i32, y: i32, facing: u8, team: usize, solid: &dyn Fn(usize) -> bool, ghost: &dyn Fn(usize) -> bool) {
    let (w, h) = dims(d, facing);
    let (l, t) = (x - w / 2, y - h / 2);
    let mut grid = vec![false; (w * h) as usize];
    for i in 0..d.cells.len() {
        if d.cells[i].is_some() && solid(i) {
            let (px, py) = cell_pos(d, x, y, facing, i);
            grid[((py - t) * w + (px - l)) as usize] = true;
        }
    }
    for gy in 0..h {
        for gx in 0..w {
            if grid[(gy * w + gx) as usize] {
                for (dx, dy) in [(1, 0), (-1, 0), (0, 1), (0, -1)] {
                    pc(fb, clip, l + gx + dx, t + gy + dy, OUTLINE);
                }
            }
        }
    }
    for i in 0..d.cells.len() {
        let Some(p) = d.cells[i] else { continue };
        let (px, py) = cell_pos(d, x, y, facing, i);
        if solid(i) {
            // Hull is lit along its top edge so shapes read at a glance.
            let open_above = py - t == 0 || !grid[((py - t - 1) * w + (px - l)) as usize];
            let col = if p == Part::Hull && open_above { team_col(team, 2) } else { part_col(p, team) };
            pc(fb, clip, px, py, col);
        } else if ghost(i) && (px + py) & 1 == 0 {
            pc(fb, clip, px, py, team_col(team, 0));
        }
    }
}

pub fn draw_ship(fb: &mut Framebuffer, clip: Rect, s: &Ship, sx: i32, sy: i32) {
    let d = s.design();
    let growing = !s.complete();
    let grown = s.grown;
    draw_design(fb, clip, d, sx, sy, s.facing, s.team, &|i| s.hp[i] > 0, &|i| growing && d.rank[i] >= grown);
}

fn layer(k: Kind) -> u8 {
    match k {
        Kind::Outpost | Kind::Forward => 0,
        Kind::Collector | Kind::OutpostSeed | Kind::ForwardSeed => 1,
        Kind::Escort | Kind::Raider => 2,
        Kind::King => 3,
    }
}

fn rock_col(w: &World, x: i32, y: i32) -> u8 {
    let (up, left) = (w.rock_at(x, y - 1) == 0, w.rock_at(x - 1, y) == 0);
    let (down, right) = (w.rock_at(x, y + 1) == 0, w.rock_at(x + 1, y) == 0);
    let n = ((x.wrapping_mul(73_856_093)) ^ (y.wrapping_mul(19_349_663))) as u32 >> 4 & 7;
    if up && left {
        ROCK + 4
    } else if (up || left) && !(down || right) {
        ROCK + 3
    } else if down || right {
        ROCK
    } else if n == 0 {
        ROCK + 2
    } else {
        ROCK + 1
    }
}

/// Corner brackets around a rock's bounding box.
pub fn bracket_rock(fb: &mut Framebuffer, clip: Rect, w: &World, a: usize, ox: i32, oy: i32, scale: i32, col: u8) {
    let r = &w.rocks[a];
    let (x0, y0) = (ox + (r.x0 - 3) / scale, oy + (r.y0 - 3) / scale);
    let (x1, y1) = (ox + (r.x1 + 3) / scale, oy + (r.y1 + 3) / scale);
    let len = (4 / scale).max(2);
    for (cx, cy, sx, sy) in [(x0, y0, 1, 1), (x1, y0, -1, 1), (x0, y1, 1, -1), (x1, y1, -1, -1)] {
        for i in 0..len {
            pc(fb, clip, cx + i * sx, cy, col);
            pc(fb, clip, cx, cy + i * sy, col);
        }
    }
}

/// A dotted Bresenham line (every `gap`th pixel), for seed flight paths.
pub fn dotted(fb: &mut Framebuffer, clip: Rect, x0: i32, y0: i32, x1: i32, y1: i32, gap: i32, col: u8) {
    let mut l = Line::new(x0, y0, x1, y1);
    let mut k = 0;
    loop {
        if k % gap == 0 {
            pc(fb, clip, l.x, l.y, col);
        }
        if l.x == x1 && l.y == y1 || k > 2000 {
            break;
        }
        l.step();
        k += 1;
    }
}

/// What the planting cursor is pointing at.
pub struct Aim {
    pub point: (i32, i32),
    pub rock: Option<usize>,
    pub ok: bool,
}

pub fn world(fb: &mut Framebuffer, w: &World, cam: (i32, i32), me: usize, aim: Option<&Aim>) {
    let clip = VIEW;
    fb.rect_fill(clip.x, clip.y, clip.w, clip.h, SPACE);
    for &(sx, sy, c) in &w.stars {
        pc(fb, clip, sx - cam.0 / 2, sy - cam.1 / 2, STAR + c);
    }
    for vy in 0..clip.h {
        let wy = cam.1 + vy;
        if !(0..WORLD_H).contains(&wy) {
            continue;
        }
        for vx in 0..clip.w {
            let wx = cam.0 + vx;
            if (0..WORLD_W).contains(&wx) && w.rock_at(wx, wy) != 0 {
                fb.pset(vx, vy, rock_col(w, wx, wy));
            }
        }
    }
    for d in &w.debris {
        pc(fb, clip, d.x - cam.0, d.y - cam.1, LOOSE);
    }
    for m in &w.motes {
        pc(fb, clip, m.x - cam.0, m.y - cam.1, team_col(m.team, 2));
    }
    // Where my seeds are headed, and which rocks my king's queued seeds have claimed.
    for s in w.ships.iter().filter(|s| s.alive && s.team == me && s.kind.is_seed()) {
        if let Some(a) = s.dest {
            let r = &w.rocks[a];
            dotted(fb, clip, s.x - cam.0, s.y - cam.1, r.cx - cam.0, r.cy - cam.1, 3, team_col(me, 1));
            bracket_rock(fb, clip, w, a, -cam.0, -cam.1, 1, team_col(me, 2));
        }
    }
    let king = w.king(me);
    for j in king.queue.iter().chain(king.job.iter()) {
        if let Some(a) = j.dest {
            bracket_rock(fb, clip, w, a, -cam.0, -cam.1, 1, team_col(me, 0));
        }
    }
    let on_screen = |s: &Ship| {
        let (l, t, r, b) = s.bbox();
        r >= cam.0 - 2 && b >= cam.1 - 2 && l <= cam.0 + clip.w + 2 && t <= cam.1 + clip.h + 2
    };
    for lay in 0..4 {
        for s in w.ships.iter().filter(|s| s.alive && layer(s.kind) == lay && on_screen(s)) {
            draw_ship(fb, clip, s, s.x - cam.0, s.y - cam.1);
        }
    }
    // Ships being printed appear pixel by pixel beside their factory.
    for (i, s) in w.ships.iter().enumerate().filter(|(_, s)| s.alive && s.kind.is_factory()) {
        if let Some(j) = s.job {
            let (px, py, f) = w.print_spot(i, j.kind);
            let d = &info(j.kind).design;
            draw_design(fb, clip, d, px - cam.0, py - cam.1, f, s.team, &|c| d.rank[c] < j.printed, &|c| d.rank[c] >= j.printed);
        }
    }
    for sh in &w.shots {
        pc(fb, clip, sh.prev.0 - cam.0, sh.prev.1 - cam.1, team_col(sh.team, 1));
        pc(fb, clip, sh.line.x - cam.0, sh.line.y - cam.1, team_col(sh.team, 3));
    }
    if let Some(aim) = aim {
        let col = if aim.ok { GOOD } else { BAD };
        match aim.rock {
            Some(a) => {
                bracket_rock(fb, clip, w, a, -cam.0, -cam.1, 1, col);
                if king.alive {
                    dotted(fb, clip, king.x - cam.0, king.y - cam.1, w.rocks[a].cx - cam.0, w.rocks[a].cy - cam.1, 4, col);
                }
            }
            None => {
                let (x, y) = (aim.point.0 - cam.0, aim.point.1 - cam.1);
                for k in -2..=2 {
                    pc(fb, clip, x + k, y + k, BAD);
                    pc(fb, clip, x + k, y - k, BAD);
                }
            }
        }
    }
}

pub fn minimap(fb: &mut Framebuffer, w: &World, r: Rect, cam: (i32, i32), view: Rect) {
    for my in 0..MINI_H {
        for mx in 0..MINI_W {
            let n = w.mini[(my * MINI_W + mx) as usize];
            let c = if n >= 40 {
                ROCK + 2
            } else if n > 0 {
                ROCK + 1
            } else {
                SPACE
            };
            fb.pset(r.x + mx, r.y + my, c);
        }
    }
    for s in w.ships.iter().filter(|s| s.alive) {
        let (mx, my) = (r.x + s.x / MINI, r.y + s.y / MINI);
        match s.kind {
            Kind::King => fb.rect_fill(mx - 1, my - 1, 3, 3, team_col(s.team, 3)),
            Kind::Outpost | Kind::Forward => fb.rect_fill(mx - 1, my - 1, 2, 2, team_col(s.team, if s.complete() { 2 } else { 0 })),
            _ => fb.pset(mx, my, team_col(s.team, 2)),
        }
    }
    fb.rect_outline(r.x + cam.0 / MINI, r.y + cam.1 / MINI, view.w / MINI + 1, view.h / MINI + 1, TEXT_DIM);
}

pub fn tactical(fb: &mut Framebuffer, w: &World, cam: (i32, i32), me: usize, aim: Option<&Aim>) {
    let r = TAC_RECT;
    fb.rect_fill(VIEW.x, VIEW.y, VIEW.w, VIEW.h, PANEL_DEEP);
    fb.rect_checker(VIEW.x, VIEW.y, VIEW.w, VIEW.h, PANEL_LO, 0);
    fb.rect_bevel(r.x - 2, r.y - 2, r.w + 4, r.h + 4, SPACE, PANEL_LO, PANEL_HI);
    for ty in 0..TAC_H {
        for tx in 0..TAC_W {
            let n = w.tac[(ty * TAC_W + tx) as usize];
            let c = if n >= 9 {
                ROCK + 2
            } else if n >= 3 {
                ROCK + 1
            } else if n > 0 {
                ROCK
            } else {
                SPACE
            };
            fb.pset(r.x + tx, r.y + ty, c);
        }
    }
    for d in &w.debris {
        fb.pset(r.x + d.x / TAC, r.y + d.y / TAC, LOOSE);
    }
    for s in w.ships.iter().filter(|s| s.alive && s.team == me && s.kind.is_seed()) {
        if let Some(a) = s.dest {
            let k = &w.rocks[a];
            dotted(fb, r, r.x + s.x / TAC, r.y + s.y / TAC, r.x + k.cx / TAC, r.y + k.cy / TAC, 2, team_col(me, 1));
        }
    }
    for s in w.ships.iter().filter(|s| s.alive) {
        let (x, y) = (r.x + s.x / TAC, r.y + s.y / TAC);
        let c = team_col(s.team, 2);
        match s.kind {
            Kind::King => {
                fb.rect_fill(x - 3, y - 3, 7, 7, OUTLINE);
                fb.rect_fill(x - 2, y - 2, 5, 5, team_col(s.team, 3));
                fb.pset(x, y, CORE);
            }
            Kind::Outpost | Kind::Forward => {
                let n = if s.kind == Kind::Forward { 2 } else { 1 };
                fb.rect_fill(x - n - 1, y - n - 1, 2 * n + 3, 2 * n + 3, OUTLINE);
                if s.complete() {
                    fb.rect_fill(x - n, y - n, 2 * n + 1, 2 * n + 1, c);
                } else {
                    fb.rect_checker(x - n, y - n, 2 * n + 1, 2 * n + 1, c, 0);
                }
            }
            Kind::Raider => fb.rect_fill(x, y, 2, 2, c),
            Kind::OutpostSeed | Kind::ForwardSeed => fb.pset(x, y, team_col(s.team, 3)),
            _ => fb.pset(x, y, c),
        }
    }
    for sh in &w.shots {
        fb.pset(r.x + sh.line.x / TAC, r.y + sh.line.y / TAC, team_col(sh.team, 3));
    }
    if let Some(aim) = aim {
        let col = if aim.ok { GOOD } else { BAD };
        if let Some(a) = aim.rock {
            bracket_rock(fb, r, w, a, r.x, r.y, TAC, col);
        }
    }
    fb.rect_outline(r.x + cam.0 / TAC, r.y + cam.1 / TAC, VIEW.w / TAC, VIEW.h / TAC, TEXT_DIM);
    fb.text("TACTICAL VIEW", r.x, r.y - 14, ACCENT, Some(OUTLINE), 1);
    fb.text("TAB RETURNS", r.x + r.w - 66, r.y - 14, TEXT_DIM, Some(OUTLINE), 1);
}
