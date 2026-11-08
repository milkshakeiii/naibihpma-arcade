//! Draws a frame into the 400x300 indexed framebuffer. Everything snaps to integer pixels;
//! dim overlays are checkerboard dithers, never alpha.

use crate::framebuffer::{Framebuffer, BAYER4};
use crate::game::*;
use crate::geom::*;
use crate::levels::HALL_NAMES;
use crate::model::*;
use crate::palette::*;
use crate::sprites::Sprites;

pub fn draw(g: &Game, s: &Sprites, fb: &mut Framebuffer) {
    match g.screen {
        Screen::Select => draw_select(g, s, fb),
        Screen::Play => draw_play(g, s, fb),
    }
    if g.show_help {
        draw_help(fb);
    }
}

fn draw_room(fb: &mut Framebuffer) {
    fb.rect_fill(0, 0, 400, 300, WALL + 1);
    for row in 0..30 {
        let y = row * 10;
        fb.rect_fill(0, y, 400, 1, WALL);
        let off = if row % 2 == 0 { 0 } else { 12 };
        for x in (off..400).step_by(24) {
            fb.rect_fill(x, y, 1, 10, WALL);
        }
    }
    // A pool of lamplight from above, dithered rather than blended.
    for y in 0..140 {
        for x in 60..340 {
            let (dx, dy) = ((x - 200) / 2, y);
            let d2 = dx * dx + dy * dy;
            let t = 14 - d2 * 14 / (70 * 70);
            if (BAYER4[(y & 3) as usize][(x & 3) as usize] as i32) < t && fb.px[(y * 400 + x) as usize] != WALL {
                fb.pset(x, y, WALL + 2);
            }
        }
    }
}

fn wrap(text: &str, width: usize) -> Vec<String> {
    let mut lines = Vec::new();
    let mut cur = String::new();
    for w in text.split_whitespace() {
        if !cur.is_empty() && cur.len() + 1 + w.len() > width {
            lines.push(std::mem::take(&mut cur));
        }
        if !cur.is_empty() {
            cur.push(' ');
        }
        cur.push_str(w);
    }
    if !cur.is_empty() {
        lines.push(cur);
    }
    lines
}

fn brackets(fb: &mut Framebuffer, x: i32, y: i32, w: i32, h: i32, col: u8, t: i32) {
    let l = 5;
    for (cx, cy, sx, sy) in [(x, y, 1, 1), (x + w - 1, y, -1, 1), (x, y + h - 1, 1, -1), (x + w - 1, y + h - 1, -1, -1)] {
        for i in 0..t {
            for k in 0..l {
                fb.pset(cx + k * sx, cy + i * sy, col);
                fb.pset(cx + i * sx, cy + k * sy, col);
            }
        }
    }
}

fn dotted(fb: &mut Framebuffer, (x0, y0): (i32, i32), (x1, y1): (i32, i32), col: u8) {
    let n = (x1 - x0).abs().max((y1 - y0).abs()).max(1);
    for i in (0..=n).step_by(3) {
        fb.pset(x0 + (x1 - x0) * i / n, y0 + (y1 - y0) * i / n, col);
    }
}

/// Pixel at `a` steps forward and `b` steps to the side of `d`, from (cx, cy).
fn along(cx: i32, cy: i32, d: Dir, a: i32, b: i32) -> (i32, i32) {
    (cx + d.dx() * a - d.dy() * b, cy + d.dy() * a + d.dx() * b)
}

// ---------------------------------------------------------------------------------------------
// Pieces. Each draws into the 20x20 tile whose top-left is (px, py).

fn draw_wall(fb: &mut Framebuffer, px: i32, py: i32) {
    fb.rect_bevel(px, py, 20, 20, WALL + 2, WALL + 3, WALL);
    fb.rect_fill(px + 1, py + 10, 18, 1, WALL);
    fb.rect_fill(px + 10, py + 1, 1, 9, WALL);
    fb.rect_fill(px + 5, py + 11, 1, 8, WALL);
    fb.rect_fill(px + 15, py + 11, 1, 8, WALL);
}

pub fn draw_belt(fb: &mut Framebuffer, px: i32, py: i32, out: Dir, ins: [bool; 4], phase: i32) {
    let (cx, cy) = (px + 10, py + 10);
    let seg = |d: Dir| -> (i32, i32, i32, i32) {
        match d {
            Dir::E => (cx - 5, cy - 5, 15, 10),
            Dir::W => (px, cy - 5, 15, 10),
            Dir::N => (cx - 5, py, 10, 15),
            Dir::S => (cx - 5, cy - 5, 10, 15),
        }
    };
    let mut segs = vec![out];
    segs.extend(Dir::ALL.into_iter().filter(|d| ins[d.index()] && *d != out));
    for &d in &segs {
        let (x, y, w, h) = seg(d);
        fb.rect_fill(x, y, w, h, BELT);
    }
    for &d in &segs {
        let (x, y, w, h) = seg(d);
        let (x, y, w, h) = if d.vertical() { (x + 1, y, w - 2, h) } else { (x, y + 1, w, h - 2) };
        fb.rect_fill(x, y, w, h, BELT + 1);
    }
    // Chevrons ride along the out direction.
    for base in [-6, 0, 6] {
        let a = base + phase;
        if !(-5..=7).contains(&a) {
            continue;
        }
        for (da, db) in [(-1, -2), (0, -1), (1, 0), (0, 1), (-1, 2)] {
            let (x, y) = along(cx, cy, out, a + da, db);
            fb.pset(x, y, BELT + 2);
        }
    }
}

fn draw_crate(fb: &mut Framebuffer, px: i32, py: i32, dir: Dir, items: &[Color]) {
    fb.rect_bevel(px + 1, py + 1, 18, 18, WOOD + 1, WOOD + 2, WOOD);
    fb.rect_fill(px + 2, py + 7, 16, 1, WOOD);
    fb.rect_fill(px + 2, py + 13, 16, 1, WOOD);
    let (cx, cy) = (px + 10, py + 10);
    // The mouth faces where items leave.
    for b in -3..=3 {
        for a in 7..=9 {
            let (x, y) = along(cx, cy, dir, a, b);
            fb.pset(x, y, OUTLINE);
        }
    }
    for (i, &c) in items.iter().take(4).enumerate() {
        let (x, y) = along(cx, cy, dir, 4 - i as i32 * 4, 0);
        let (_, m, l) = item_ramp(c);
        fb.rect_fill(x - 2, y - 2, 5, 5, OUTLINE);
        fb.rect_fill(x - 1, y - 1, 3, 3, m);
        fb.pset(x - 1, y - 1, l);
    }
}

fn draw_booth(fb: &mut Framebuffer, px: i32, py: i32, c: Color) {
    fb.rect_bevel(px + 1, py + 1, 18, 18, METAL + 1, METAL + 3, METAL);
    fb.rect_fill(px + 5, py + 5, 10, 10, METAL);
    let (d, m, l) = item_ramp(c);
    fb.rect_fill(px + 5, py + 5, 10, 3, m);
    fb.rect_fill(px + 5, py + 5, 10, 1, l);
    fb.rect_fill(px + 5, py + 8, 10, 1, d);
    fb.pset(px + 8, py + 10, m);
    fb.pset(px + 12, py + 11, m);
    // Four doors.
    fb.rect_fill(px + 7, py + 1, 6, 2, OUTLINE);
    fb.rect_fill(px + 7, py + 17, 6, 2, OUTLINE);
    fb.rect_fill(px + 1, py + 7, 2, 6, OUTLINE);
    fb.rect_fill(px + 17, py + 7, 2, 6, OUTLINE);
}

fn draw_switch(fb: &mut Framebuffer, px: i32, py: i32, lever: Dir) {
    let (cx, cy) = (px + 10, py + 10);
    for dy in -8..=8 {
        for dx in -8..=8 {
            let r2 = dx * dx + dy * dy;
            if r2 <= 64 {
                fb.pset(cx + dx, cy + dy, if r2 > 49 { OUTLINE } else if dx + dy < -6 { METAL + 2 } else { METAL + 1 });
            }
        }
    }
    // The other side: where the next item goes after this one.
    let (ox, oy) = along(cx, cy, lever.opposite(), 5, 0);
    fb.rect_fill(ox - 1, oy - 1, 3, 3, METAL);
    for a in 0..=7 {
        for b in -1..=1 {
            let (x, y) = along(cx, cy, lever, a, b);
            fb.pset(x, y, if b == -1 { ACCENT_HI } else { ACCENT });
        }
    }
    let (kx, ky) = along(cx, cy, lever, 7, 0);
    fb.rect_fill(kx - 2, ky - 2, 5, 5, OUTLINE);
    fb.rect_fill(kx - 1, ky - 1, 3, 3, ACCENT_HI);
    fb.rect_fill(cx - 1, cy - 1, 3, 3, METAL + 3);
}

fn draw_zipper(fb: &mut Framebuffer, px: i32, py: i32, out: Dir, first: Dir, next: Dir) {
    fb.rect_bevel(px + 2, py + 2, 16, 16, METAL + 1, METAL + 3, METAL);
    let (cx, cy) = (px + 10, py + 10);
    for a in 0..6 {
        for b in -(5 - a)..=(5 - a) {
            let (x, y) = along(cx, cy, out, a + 1, b);
            fb.pset(x, y, if a == 0 || b.abs() == 5 - a { OUTLINE } else { ACCENT });
        }
    }
    for side in [first, first.opposite()] {
        for b in [-3, 0, 3] {
            let (x, y) = along(cx, cy, side, 6, b);
            fb.pset(x, y, OUTLINE);
            let (x, y) = along(cx, cy, side, 5, b);
            fb.pset(x, y, OUTLINE);
        }
        let (lx, ly) = along(cx, cy, side, 3, 0);
        fb.rect_fill(lx - 1, ly - 1, 3, 3, if side == next { GOOD } else { GOOD_DIM });
    }
}

fn arrow_points(cx: i32, cy: i32, d: Dir) -> Vec<(i32, i32)> {
    let mut pts = Vec::new();
    for a in -6..=0 {
        for b in -2..=1 {
            pts.push(along(cx, cy, d, a, b));
        }
    }
    for a in 1..=6 {
        let half = 6 - a;
        for b in -half - 1..=half {
            pts.push(along(cx, cy, d, a, b));
        }
    }
    pts
}

fn draw_arrow(fb: &mut Framebuffer, px: i32, py: i32, d: Dir, fill: u8) {
    let pts = arrow_points(px + 10, py + 10, d);
    for &(x, y) in &pts {
        for (dx, dy) in [(1, 0), (-1, 0), (0, 1), (0, -1)] {
            fb.pset(x + dx, y + dy, OUTLINE);
        }
    }
    for &(x, y) in &pts {
        fb.pset(x, y, fill);
    }
}

fn draw_bin(fb: &mut Framebuffer, s: &Sprites, px: i32, py: i32, bin: &Bin, got: &[Color]) {
    fb.rect_bevel(px + 1, py + 2, 18, 17, WOOD + 1, WOOD + 2, WOOD);
    let (d, m, l) = item_ramp(bin.want);
    fb.rect_fill(px + 1, py + 2, 18, 4, OUTLINE);
    fb.rect_fill(px + 2, py + 3, 16, 2, m);
    fb.rect_fill(px + 2, py + 3, 16, 1, l);
    fb.rect_fill(px + 3, py + 6, 14, 7, OUTLINE);
    fb.blit(s.mini(bin.want), px + 6, py + 5, false);
    let n = bin.need as i32;
    let x0 = px + 10 - (n * 5 - 1) / 2;
    for j in 0..n {
        let x = x0 + j * 5;
        fb.rect_fill(x, py + 14, 4, 4, OUTLINE);
        if (j as usize) < got.len() {
            fb.rect_fill(x + 1, py + 15, 2, 2, m);
            fb.pset(x + 1, py + 15, l);
        } else {
            fb.rect_fill(x + 1, py + 15, 2, 2, d);
        }
    }
    if got.len() >= bin.need as usize {
        fb.rect_fill(px + 15, py, 4, 3, OUTLINE);
        fb.rect_fill(px + 16, py + 1, 2, 1, GOOD);
    }
}

fn bolts(fb: &mut Framebuffer, px: i32, py: i32) {
    for (x, y) in [(px + 3, py + 3), (px + 16, py + 16)] {
        fb.pset(x, y, OUTLINE);
        fb.pset(x - 1, y - 1, METAL + 3);
    }
}

/// Does the left-board cell at (x, y) send items out in direction `d`? Used to draw belt
/// joints; booths pass straight through, so they look one tile further back.
fn feeds(g: &Game, x: i32, y: i32, d: Dir, depth: u8) -> bool {
    if !inside(x, y) {
        return false;
    }
    match g.cell(Board::Left, x, y) {
        Cell::Belt(o) | Cell::Zipper(o, _) => o == d,
        Cell::Crate(k) => g.level().crates[k as usize].dir == d,
        Cell::Switch(l) => l.vertical() == d.vertical(),
        Cell::Booth(_) if depth == 0 => feeds(g, x - d.dx(), y - d.dy(), d, 1),
        _ => false,
    }
}

fn belt_phase(g: &Game) -> i32 {
    if g.phase == Phase::Make {
        (g.ticks as i32 * (g.speed as i32 + 1)) % 6
    } else {
        0
    }
}

fn draw_cell(g: &Game, s: &Sprites, fb: &mut Framebuffer, b: Board, x: i32, y: i32) {
    let (px, py) = cell_px(b, x, y);
    let lv = g.level();
    let cell = g.cell(b, x, y);
    match cell {
        Cell::Floor => {}
        Cell::Wall => draw_wall(fb, px, py),
        Cell::Crate(k) => {
            let k = k as usize;
            // Once a Make has finished, the crates stand empty until you go back to building.
            let emptied = matches!(g.phase, Phase::Lock(_) | Phase::Deliver | Phase::Failed(..) | Phase::Solved);
            let items = match &g.make {
                Some(m) => m.crate_left(k),
                None if emptied => &[][..],
                None => &lv.crates[k].items[..],
            };
            draw_crate(fb, px, py, lv.crates[k].dir, items);
        }
        Cell::Belt(o) => {
            let mut ins = [false; 4];
            for side in Dir::ALL {
                ins[side.index()] = side != o && feeds(g, x + side.dx(), y + side.dy(), side.opposite(), 0);
            }
            draw_belt(fb, px, py, o, ins, belt_phase(g));
        }
        Cell::Booth(c) => draw_booth(fb, px, py, c),
        Cell::Switch(l) => draw_switch(fb, px, py, l),
        Cell::Zipper(o, f) => {
            let next = g.make.as_ref().map_or(f, |m| m.zip_next[idx(x, y)]);
            draw_zipper(fb, px, py, o, f, next);
        }
        Cell::Arrow(d) => draw_arrow(fb, px, py, d, if g.is_fixed(b, x, y) { METAL + 2 } else { ACCENT_HI }),
        Cell::Bin(i) => {
            let got = g.deliver.as_ref().map_or(&[][..], |d| &d.got[i as usize][..]);
            draw_bin(fb, s, px, py, &lv.bins[i as usize], got);
        }
    }
    if g.is_fixed(b, x, y) && matches!(cell, Cell::Booth(_) | Cell::Switch(_) | Cell::Zipper(..)) {
        bolts(fb, px, py);
    }
}

/// The piece a tool would place, for tray icons and the cursor ghost.
fn tool_cell(t: Tool) -> Cell {
    match t {
        Tool::Belt => Cell::Belt(Dir::E),
        Tool::Booth(c) => Cell::Booth(c),
        Tool::Switch => Cell::Switch(Dir::N),
        Tool::Zipper => Cell::Zipper(Dir::E, Dir::N),
        Tool::Arrow => Cell::Arrow(Dir::E),
    }
}

fn draw_piece(fb: &mut Framebuffer, px: i32, py: i32, c: Cell) {
    match c {
        Cell::Belt(d) => {
            let mut ins = [false; 4];
            ins[d.opposite().index()] = true;
            draw_belt(fb, px, py, d, ins, 0);
        }
        Cell::Booth(col) => draw_booth(fb, px, py, col),
        Cell::Switch(l) => draw_switch(fb, px, py, l),
        Cell::Zipper(o, f) => draw_zipper(fb, px, py, o, f, f),
        Cell::Arrow(d) => draw_arrow(fb, px, py, d, ACCENT_HI),
        _ => {}
    }
}

// ---------------------------------------------------------------------------------------------

fn draw_play(g: &Game, s: &Sprites, fb: &mut Framebuffer) {
    let lv = g.level();
    draw_room(fb);
    fb.rect_bevel(6, 32, 388, 176, BENCH + 1, BENCH + 2, BENCH);
    fb.rect_fill(GUTTER_X0, BOARD_Y, GUTTER_X1 - GUTTER_X0, BOARD_PX, BENCH);
    fb.rect_checker(GUTTER_X0, BOARD_Y, GUTTER_X1 - GUTTER_X0, BOARD_PX, BENCH + 1);

    for b in [Board::Left, Board::Right] {
        let bx = board_x(b);
        let base = if b == Board::Left { FLOOR_L } else { FLOOR_R };
        fb.rect_fill(bx - 1, BOARD_Y - 1, BOARD_PX + 2, BOARD_PX + 2, OUTLINE);
        for y in 0..N {
            for x in 0..N {
                let (px, py) = cell_px(b, x, y);
                fb.rect_fill(px, py, TILE, TILE, base + 1);
                fb.rect_fill(px, py, TILE, 1, base);
                fb.rect_fill(px, py, 1, TILE, base);
                if b == Board::Right {
                    for i in 0..3 {
                        fb.pset(px + 3 + i, py + 5 - i, base + 2);
                    }
                } else if (x + y) % 2 == 0 {
                    fb.pset(px + 10, py + 10, base + 2);
                }
            }
        }
    }

    // Chutes, hatches and inlets.
    for (k, &row) in lv.chutes.iter().enumerate() {
        let cy = row_center_y(row);
        fb.rect_fill(GUTTER_X0, cy - 6, GUTTER_X1 - GUTTER_X0, 13, OUTLINE);
        fb.rect_fill(GUTTER_X0, cy - 5, GUTTER_X1 - GUTTER_X0, 11, GLASS);
        fb.rect_fill(GUTTER_X0 + 1, cy - 4, GUTTER_X1 - GUTTER_X0 - 2, 1, GLASS + 2);
        fb.rect_fill(GUTTER_X0 + 1, cy + 4, GUTTER_X1 - GUTTER_X0 - 2, 1, GLASS + 1);
        fb.rect_bevel(GUTTER_X0 - 4, cy - 7, 6, 15, METAL + 2, METAL + 3, METAL);
        fb.rect_bevel(GUTTER_X1 - 2, cy - 7, 5, 15, METAL + 1, METAL + 3, METAL);
        let open = g.phase == Phase::Deliver || matches!(g.phase, Phase::Failed(..) | Phase::Solved);
        if !open {
            fb.rect_fill(GUTTER_X1 - 5, cy - 5, 2, 11, METAL + 3);
        }
        let firing = match (&g.deliver, g.phase) {
            (Some(d), Phase::Deliver) => d.next == k && !d.queues[k].is_empty(),
            (_, Phase::Lock(_)) => k == 0,
            _ => false,
        };
        fb.rect_fill(GUTTER_X1 - 9, cy - 12, 6, 4, OUTLINE);
        fb.rect_fill(GUTTER_X1 - 8, cy - 11, 4, 2, if firing { GOOD } else { GOOD_DIM });
        fb.text(&(k + 1).to_string(), GUTTER_X0 + 6, cy - 15, TEXT_DIM, None, 1);
        for (i, &c) in g.chute_view(k).iter().enumerate() {
            let (x, y) = chute_slot(row, i);
            fb.blit(s.mini(c), x - 4, y - 4, false);
        }
    }

    for b in [Board::Left, Board::Right] {
        for y in 0..N {
            for x in 0..N {
                draw_cell(g, s, fb, b, x, y);
            }
        }
    }

    for t in &g.trails {
        for w in t.windows(2) {
            dotted(fb, w[0], w[1], CHALK);
        }
    }

    // Hints: the stored solution, ghosted in where the board doesn't match it yet.
    if g.hint > 0 && !g.phase.running() {
        for p in lv.solution.iter().filter(|p| hint_shows(g, p)) {
            if g.layout.get(p.board, p.x, p.y) == Some(p.cell) {
                continue;
            }
            let (px, py) = cell_px(p.board, p.x, p.y);
            draw_piece(fb, px, py, p.cell);
            fb.rect_checker(px + 1, py + 1, TILE - 1, TILE - 1, GOOD_DIM);
            brackets(fb, px + 1, py + 1, TILE - 2, TILE - 2, GOOD, 1);
        }
    }

    // Items in motion.
    let (num, den) = g.anim;
    if let Some(m) = &g.make {
        for it in m.items.iter().filter(|it| it.alive) {
            let (x, y) = lerp(it.from, it.to, num, den);
            // Items waiting at a zipper shrink to their slot so the zipper stays readable.
            let waiting = inside(it.x, it.y) && matches!(m.grid[idx(it.x, it.y)], Cell::Zipper(..));
            if it.chute.is_some() || waiting {
                fb.blit(s.mini(it.color), x - 4, y - 4, false);
            } else {
                fb.blit(s.item(it.color), x - 6, y - 6, false);
            }
        }
    }
    if let Some(d) = &g.deliver {
        if let Some(it) = &d.item {
            let (x, y) = lerp(it.from, it.to, num, den);
            fb.blit(s.item(it.color), x - 6, y - 6, false);
        }
    }

    // Cursor, and a dithered ghost of the part it would place.
    let (b, x, y) = g.cursor_cell();
    let (px, py) = cell_px(b, x, y);
    let tool = lv.tray[g.tool].tool;
    let free = g.layout.get(b, x, y).is_none() && lv.fixed(b, x, y) == Cell::Floor;
    if !g.phase.running() && g.phase != Phase::Solved && free && tool.allowed(b) && g.tray_left(g.tool) != Some(0) {
        draw_piece(fb, px, py, tool_cell(tool));
        fb.rect_checker(px + 1, py + 1, TILE - 1, TILE - 1, if b == Board::Left { FLOOR_L + 1 } else { FLOOR_R + 1 });
    }
    brackets(fb, px - 1, py - 1, TILE + 2, TILE + 2, CURSOR, 1);

    let marker = match g.phase {
        Phase::Jammed(_, x, y) => Some(cell_px(Board::Left, x, y)),
        Phase::Failed(_, x, y) if x >= 0 => Some(cell_px(Board::Right, x, y)),
        _ => None,
    };
    if let Some((mx, my)) = marker {
        brackets(fb, mx - 2, my - 2, TILE + 4, TILE + 4, BAD, 2);
    }

    // Header: hall and state, room name, and the playback buttons.
    let (state, lamp) = match g.phase {
        Phase::Edit => ("BUILD", GOOD_DIM),
        Phase::Make => ("MAKING", GOOD),
        Phase::Lock(_) => ("CHUTES LOCKED", ACCENT_HI),
        Phase::Deliver => ("DELIVERING", GOOD),
        Phase::Jammed(..) => ("JAMMED", BAD),
        Phase::Failed(..) => ("STOPPED", BAD),
        Phase::Solved => ("SOLVED", GOOD),
    };
    fb.rect_fill(14, 8, 7, 7, OUTLINE);
    fb.rect_fill(15, 9, 5, 5, lamp);
    fb.text(&format!("{} - {state}", HALL_NAMES[(lv.hall as usize - 1).min(1)]), 25, 8, TEXT_DIM, None, 1);
    let solved_mark = if g.solved[g.cur] { "  (SOLVED)" } else { "" };
    fb.text(&format!("{}  {}{solved_mark}", lv.id, lv.name), 14, 19, TEXT, Some(OUTLINE), 1);
    for (b, (x, y, w, h)) in BUTTONS {
        let (label, lit) = match b {
            Button::Run => (if g.phase.running() { "STOP" } else { "RUN" }, g.phase.running()),
            Button::Restart => ("RESTART", g.phase != Phase::Edit),
            Button::Speed => (["1X", "2X", "", "4X"][(g.speed.clamp(1, 4) - 1) as usize], g.speed > 1),
        };
        fb.rect_bevel(x, y, w, h, if lit { BENCH + 2 } else { BENCH + 1 }, BENCH + 3, BENCH);
        let col = if b == Button::Restart && g.phase == Phase::Edit { TEXT_DIM } else { TEXT };
        let tw = Framebuffer::text_width(label, 1);
        let icon_w = if b == Button::Speed { 10 } else { 0 };
        let tx = x + (w - tw - icon_w) / 2 + icon_w;
        if b == Button::Speed {
            // Fast-forward chevrons.
            for k in 0..2 {
                for i in 0..4 {
                    fb.pset(tx - 10 + k * 4 + i / 2, y + 7 + i, col);
                    fb.pset(tx - 10 + k * 4 + i / 2, y + 13 - i, col);
                }
            }
        }
        fb.text(label, tx, y + 6, col, Some(OUTLINE), 1);
    }

    draw_tray(g, fb);

    // The latest message, then the foreman's note for this room (always shown).
    if let Some((text, kind)) = &g.message {
        let col = match *kind {
            MSG_BAD => BAD,
            MSG_GOOD => GOOD,
            _ => ACCENT_HI,
        };
        fb.text_centered(text, 200, 261, col, Some(OUTLINE), 1);
    } else if let Some(text) = look(g) {
        fb.text_centered(&text, 200, 261, TEXT_DIM, Some(OUTLINE), 1);
    }
    for (i, line) in wrap(&lv.note.to_uppercase(), 64).iter().take(3).enumerate() {
        fb.text_centered(line, 200, 272 + i as i32 * 9, TEXT, Some(OUTLINE), 1);
    }

    if g.phase == Phase::Solved {
        fb.rect_checker(GUTTER_X0 - 40, 94, 128, 60, OUTLINE);
        fb.rect_bevel(132, 100, 136, 48, BENCH + 1, BENCH + 3, BENCH);
        fb.text_centered("SOLVED", 200, 108, GOOD, Some(OUTLINE), 2);
        let next = if g.cur + 1 < g.levels.len() { "ENTER: NEXT ROOM" } else { "ENTER: BACK TO HALLS" };
        fb.text_centered(next, 200, 132, TEXT, None, 1);
    }
}

fn draw_tray(g: &Game, fb: &mut Framebuffer) {
    let lv = g.level();
    fb.rect_bevel(6, 210, 388, 46, WOOD, WOOD + 1, OUTLINE);
    fb.rect_checker(7, 211, 386, 44, WOOD + 1);
    fb.rect_fill(7, 253, 386, 2, OUTLINE);
    for (i, slot) in lv.tray.iter().enumerate() {
        let (x, y, w, h) = tray_slot_rect(i);
        let sel = i == g.tool;
        if sel {
            fb.rect_bevel(x, y - 1, w, h, BENCH + 2, BENCH + 3, BENCH);
        } else {
            fb.rect_bevel(x, y, w, h, BENCH, BENCH, BENCH + 2);
        }
        let left = g.tray_left(i);
        draw_piece(fb, x + 4, y + 4 - sel as i32, tool_cell(slot.tool));
        if left == Some(0) {
            fb.rect_checker(x + 1, y + 1, w - 2, h - 2, BENCH);
        }
        fb.text(&(i + 1).to_string(), x + 2, y + 2, TEXT_DIM, None, 1);
        if let (Some(n), Some(l)) = (slot.count, left) {
            let x0 = x + w / 2 - (n as i32 * 4 - 1) / 2;
            for j in 0..n as i32 {
                fb.rect_fill(x0 + j * 4, y + h + 3, 3, 3, if j < l as i32 { ACCENT_HI } else { WOOD });
            }
        }
        if sel {
            brackets(fb, x - 2, y - 3, w + 4, h + 4, CURSOR, 1);
        }
    }
    let tool = lv.tray[g.tool].tool;
    let tx = tray_slot_rect(lv.tray.len()).0 + 8;
    let hint: [&str; 2] = match tool {
        Tool::Belt => ["HOLD SPACE (OR DRAG) AND MOVE", "TAP A BELT TO TURN IT"],
        Tool::Zipper => ["TAKES FROM ITS TOOTHED SIDES", "LIT SIDE FIRST. R TURNS, T FLIPS"],
        Tool::Switch => ["LEVER POINTS WHERE NEXT ONE GOES", "ON BELTS, FEED IT FROM THE SIDE"],
        Tool::Arrow => ["TURNS WHATEVER SLIDES ONTO IT", "R TURNS. FLOOR ONLY"],
        Tool::Booth(_) => ["PAINTS WHAT GOES STRAIGHT THROUGH", "WORKS ON EITHER BOARD"],
    };
    fb.text(tool.name(), tx, 215, TEXT, Some(OUTLINE), 1);
    fb.text(hint[0], tx, 225, TEXT_DIM, Some(OUTLINE), 1);
    fb.text(hint[1], tx, 234, TEXT_DIM, Some(OUTLINE), 1);
    let keys = if g.phase.running() { "ENTER STOP  R RESTART  F SPEED" } else { "ENTER RUN  BKSP UNDO  H HINT  F1 KEYS" };
    fb.text(keys, tx, 245, ACCENT, Some(OUTLINE), 1);
}

/// What the piece under the cursor is and does.
fn look(g: &Game) -> Option<String> {
    let (b, x, y) = g.cursor_cell();
    let lv = g.level();
    let bolted = if g.is_fixed(b, x, y) && !matches!(lv.fixed(b, x, y), Cell::Wall | Cell::Crate(_) | Cell::Bin(_)) { " (BOLTED)" } else { "" };
    let chute = lv.chutes.iter().position(|&r| r == y).map(|k| k + 1);
    let text = match g.cell(b, x, y) {
        Cell::Crate(_) => "CRATE: SENDS OUT ITS ITEMS, THE ONE BY THE MOUTH FIRST".to_string(),
        Cell::Belt(_) => format!("BELT: CARRIES ITEMS ONE TILE A BEAT{bolted}"),
        Cell::Booth(c) => format!("{} BOOTH: PAINTS ITEMS GOING STRAIGHT THROUGH{bolted}", color_name(c)),
        Cell::Switch(_) if b == Board::Left => format!("SWITCH: DEALS ITEMS TO ITS LEVER SIDES IN TURN{bolted}"),
        Cell::Switch(_) => format!("SWITCH: TURNS EACH ITEM TOWARD ITS LEVER, THEN FLIPS{bolted}"),
        Cell::Zipper(..) => format!("ZIPPER: TAKES FROM ITS TOOTHED SIDES IN TURN{bolted}"),
        Cell::Arrow(_) => format!("ARROW: TURNS WHATEVER SLIDES ONTO IT{bolted}"),
        Cell::Bin(i) => {
            let bin = lv.bins[i as usize];
            format!("BIN: WANTS {} {}", bin.need, color_name(bin.want))
        }
        Cell::Wall => "WALL".to_string(),
        Cell::Floor => match (b, x, chute) {
            (Board::Left, 7, Some(k)) => format!("A BELT LEAVING THIS EDGE FILLS CHUTE {k}"),
            (Board::Right, 0, Some(k)) => format!("CHUTE {k} DROPS ITEMS HERE, SLIDING RIGHT"),
            _ => return None,
        },
    };
    Some(text)
}

/// Hint 1 ghosts the key parts: every non-belt piece, or for belt-only rooms the belt tiles
/// that touch a machine. Hint 2 ghosts the whole stored solution.
fn hint_shows(g: &Game, p: &Placement) -> bool {
    if g.hint >= 2 || !matches!(p.cell, Cell::Belt(_)) {
        return true;
    }
    let lv = g.level();
    if lv.solution.iter().any(|q| !matches!(q.cell, Cell::Belt(_))) {
        return false;
    }
    Dir::ALL.into_iter().any(|d| matches!(lv.fixed(p.board, p.x + d.dx(), p.y + d.dy()), Cell::Booth(_) | Cell::Switch(_) | Cell::Zipper(..)))
}

fn draw_select(g: &Game, s: &Sprites, fb: &mut Framebuffer) {
    draw_room(fb);
    fb.text_centered("CHUTEWORKS", 200, 18, ACCENT_HI, Some(OUTLINE), 3);
    fb.text_centered("A SMALL FACTORY ON THE NIGHT SHIFT", 200, 48, TEXT_DIM, None, 1);
    for (h, name) in HALL_NAMES.iter().enumerate() {
        let rooms: Vec<usize> = (0..g.levels.len()).filter(|&i| g.levels[i].hall as usize == h + 1).collect();
        let (Some(&a), Some(&b)) = (rooms.first(), rooms.last()) else { continue };
        let (x0, y0, _, _) = g.room_rect(a);
        let (x1, _, w, _) = g.room_rect(b);
        fb.text(name, x0 - 6, y0 - 14, TEXT, Some(OUTLINE), 1);
        fb.rect_bevel(x0 - 10, y0 - 4, x1 + w - x0 + 20, 50, BENCH + 1, BENCH + 2, BENCH);
    }
    for i in 0..g.levels.len() {
        let (x, y, w, h) = g.room_rect(i);
        let lit = g.solved[i];
        fb.rect_fill(x - 1, y - 1, w + 2, h + 2, OUTLINE);
        fb.rect_fill(x, y, w, h, if lit { ACCENT } else { GLASS });
        if lit {
            fb.rect_checker(x, y, w, h / 2, ACCENT_HI);
        } else {
            for k in 0..6 {
                fb.pset(x + 4 + k, y + 10 - k, GLASS + 1);
            }
        }
        fb.rect_fill(x + w / 2, y, 1, h, OUTLINE);
        fb.rect_fill(x, y + h / 2, w, 1, OUTLINE);
        fb.rect_fill(x - 2, y + h, w + 4, 2, METAL + 2);
        fb.text_centered(&g.levels[i].id, x + w / 2, y + h + 4, if i == g.sel { TEXT } else { TEXT_DIM }, None, 1);
        if i == g.sel {
            brackets(fb, x - 4, y - 4, w + 8, h + 8, CURSOR, 1);
        }
    }
    let lv = &g.levels[g.sel];
    fb.text_centered(&format!("{}  {}", lv.id, lv.name), 200, 266, TEXT, Some(OUTLINE), 1);
    let done = g.solved.iter().filter(|s| **s).count();
    fb.text_centered(&format!("{done} OF {} ROOMS SOLVED", g.levels.len()), 200, 254, TEXT_DIM, None, 1);
    fb.text_centered("ARROWS PICK  ENTER PLAY  F1 KEYS  ESC QUIT", 200, 287, TEXT_DIM, None, 1);
    // Each solved room shows what it ships, on its sill.
    for i in 0..g.levels.len() {
        if g.solved[i] {
            let (x, y, w, h) = g.room_rect(i);
            for (k, b) in g.levels[i].bins.iter().enumerate() {
                let n = g.levels[i].bins.len() as i32;
                fb.blit(s.mini(b.want), x + w / 2 - 4 - (n - 1) * 5 + k as i32 * 10, y + h - 9, false);
            }
        }
    }
}

fn draw_help(fb: &mut Framebuffer) {
    fb.rect_checker(0, 0, 400, 300, OUTLINE);
    fb.rect_bevel(40, 30, 320, 236, BENCH + 1, BENCH + 3, BENCH);
    fb.text_centered("KEYS", 200, 40, ACCENT_HI, Some(OUTLINE), 2);
    let rows: [(&str, &str); 13] = [
        ("ARROWS / WASD", "MOVE, ACROSS BOTH BOARDS"),
        ("SPACE / CLICK", "PLACE THE PICKED PART"),
        ("HOLD + MOVE", "DRAW BELTS (BACK UP TO ERASE)"),
        ("R / WHEEL", "TURN"),
        ("T", "FLIP A SWITCH OR ZIPPER"),
        ("X / RIGHT CLICK", "REMOVE"),
        ("1-9  Q  E", "PICK A PART FROM THE SHELF"),
        ("BACKSPACE   C", "UNDO    CLEAR THE ROOM"),
        ("ENTER", "RUN / STOP / NEXT ROOM"),
        ("F  /  R IN A RUN", "SPEED 1X 2X 4X / RESTART"),
        ("H", "HINT (AGAIN: MORE, THEN OFF)"),
        ("TAB / F2", "OTHER BOARD / SCANLINES"),
        ("ESC", "BACK TO THE HALLS"),
    ];
    for (i, (k, v)) in rows.iter().enumerate() {
        let y = 60 + i as i32 * 10;
        fb.text(k, 56, y, ACCENT_HI, None, 1);
        fb.text(v, 158, y, TEXT, None, 1);
    }
    fb.text_centered("BELTS CROSS ONLY INSIDE BOOTHS.", 200, 194, TEXT_DIM, None, 1);
    fb.text_centered("CHUTES TAKE TURNS FROM THE TOP.", 200, 204, TEXT_DIM, None, 1);
    fb.text_centered("ONE ITEM ON THE FLOOR AT A TIME.", 200, 214, TEXT_DIM, None, 1);
    fb.text_centered("DOTS ON AN ITEM SPELL ITS PAINT:", 200, 228, TEXT_DIM, None, 1);
    fb.text_centered("RED LEFT, YELLOW MIDDLE, BLUE RIGHT.", 200, 238, TEXT_DIM, None, 1);
    fb.text_centered("F1 OR ESC TO CLOSE", 200, 254, ACCENT, None, 1);
}
