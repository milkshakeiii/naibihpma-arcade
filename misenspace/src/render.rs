//! Draws a frame into the 400x300 indexed framebuffer: integer pixels only, dithers instead of
//! alpha, nothing that flashes.

use crate::framebuffer::{Framebuffer, BAYER4};
use crate::game::*;
use crate::geom::*;
use crate::model::*;
use crate::palette::*;
use crate::sim::GState;
use crate::sprites::Sprites;

pub fn draw(g: &Game, s: &Sprites, fb: &mut Framebuffer) {
    draw_space(g, fb);
    draw_header(g, s, fb);
    draw_kitchen(g, s, fb);
    draw_rail(g, s, fb);
    draw_dining(g, s, fb);
    draw_cursor(g, fb);
    draw_bottom(g, s, fb);
    if g.phase == Phase::Closed {
        draw_tally(g, fb);
    }
    if g.show_help {
        draw_help(fb);
    }
}

/// Fixed pseudo-random stars; a few twinkle slowly (ambient palette play only).
fn draw_space(g: &Game, fb: &mut Framebuffer) {
    fb.rect_fill(0, 0, 400, 300, SPACE);
    for y in 0..300 {
        let band = BAYER4[(y & 3) as usize];
        for x in 0..400 {
            if (band[(x & 3) as usize] as i32) < (y / 40).min(3) {
                fb.pset(x, y, SPACE + 1);
            }
        }
    }
    let mut seed = 0x1234_5678u32;
    for k in 0..90 {
        seed ^= seed << 13;
        seed ^= seed >> 17;
        seed ^= seed << 5;
        let (x, y) = ((seed % 400) as i32, ((seed / 400) % 300) as i32);
        let twinkle = k % 7 == 0 && (g.ticks / 20 + k as u64) % 5 == 0;
        fb.pset(x, y, if twinkle { SPACE + 3 } else if k % 3 == 0 { STAR } else { SPACE + 2 });
    }
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

fn along(cx: i32, cy: i32, d: Dir, a: i32, b: i32) -> (i32, i32) {
    let side = d.cw();
    (cx + d.dx() * a + side.dx() * b, cy + d.dy() * a + side.dy() * b)
}

fn filled_circle(fb: &mut Framebuffer, cx: i32, cy: i32, r: i32, fill: u8, hi: Option<u8>) {
    for dy in -r..=r {
        for dx in -r..=r {
            let d2 = dx * dx + dy * dy;
            if d2 <= r * r {
                let edge = d2 > (r - 1) * (r - 1);
                let c = if edge {
                    OUTLINE
                } else if hi.is_some() && dx + dy < -r / 2 {
                    hi.unwrap()
                } else {
                    fill
                };
                fb.pset(cx + dx, cy + dy, c);
            }
        }
    }
}

fn arrow_tip(fb: &mut Framebuffer, cx: i32, cy: i32, d: Dir, col: u8) {
    for (a, b) in [(7, 0), (6, -1), (6, 0), (6, 1), (5, -2), (5, 2)] {
        let (x, y) = along(cx, cy, d, a, b);
        fb.pset(x, y, col);
    }
}

// ---------------------------------------------------------------------------------------------
// Kitchen pieces, each in the 20x20 tile whose top-left is (px, py).

fn draw_belt(fb: &mut Framebuffer, px: i32, py: i32, out: Dir, ins: [bool; 4], phase: i32) {
    let (cx, cy) = (px + 10, py + 10);
    let seg = |d: Dir| -> (i32, i32, i32, i32) {
        match d {
            Dir::E => (cx - 6, cy - 6, 16, 12),
            Dir::W => (px, cy - 6, 16, 12),
            Dir::N => (cx - 6, py, 12, 16),
            Dir::S => (cx - 6, cy - 6, 12, 16),
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
    for base in [-6, 0, 6] {
        let a = base + phase;
        if !(-5..=8).contains(&a) {
            continue;
        }
        for (da, db) in [(-1, -3), (0, -2), (1, -1), (2, 0), (1, 1), (0, 2), (-1, 3)] {
            let (x, y) = along(cx, cy, out, a + da, db);
            fb.pset(x, y, BELT + 2);
        }
    }
}

impl Dir {
    fn vertical(self) -> bool {
        matches!(self, Dir::N | Dir::S)
    }
    fn index(self) -> usize {
        self as usize
    }
}

fn draw_box(fb: &mut Framebuffer, px: i32, py: i32, fill: u8, hi: u8, lo: u8) {
    fb.rect_bevel(px + 1, py + 1, 18, 18, fill, hi, lo);
}

/// Claw offsets (forward, side) for an arm swinging 180 degrees from back to front.
const SWING: [(i32, i32); 9] = [(-7, 0), (-6, 3), (-5, 5), (-3, 6), (0, 7), (3, 6), (5, 5), (6, 3), (7, 0)];

fn arm_claw(g: &Game, x: i32, y: i32, d: Dir) -> (i32, i32) {
    let (cx, cy) = cell_center(Board::Kitchen, x, y);
    let (num, den) = g.anim;
    let step = match g.sim.arm[idx(x, y)] {
        1 => num * 8 / den.max(1),
        2 => 8,
        3 => 8 - num * 8 / den.max(1),
        _ => 0,
    };
    let (a, b) = SWING[step.clamp(0, 8) as usize];
    along(cx, cy, d, a, b)
}

fn feeds_belt(g: &Game, x: i32, y: i32, d: Dir) -> bool {
    if !inside(x, y) {
        return false;
    }
    match g.sim.cells[idx(x, y)] {
        KCell::Belt(o) | KCell::Machine(_, o) | KCell::Counter(_, o) => o == d,
        KCell::Hatch { dir, .. } => dir == d,
        _ => false,
    }
}

fn draw_kcell(g: &Game, s: &Sprites, fb: &mut Framebuffer, x: i32, y: i32) {
    let (px, py) = cell_px(Board::Kitchen, x, y);
    let (cx, cy) = (px + 10, py + 10);
    let i = idx(x, y);
    let busy = |t: u8| t > 0;
    match g.sim.cells[i] {
        KCell::Floor => {}
        KCell::Wall => {
            // A frosted coolant pipe.
            fb.rect_fill(px + 4, py, 12, 20, OUTLINE);
            fb.rect_fill(px + 5, py, 10, 20, METAL + 1);
            fb.rect_fill(px + 6, py, 2, 20, METAL + 3);
            for yy in [py + 3, py + 13] {
                fb.rect_fill(px + 3, yy, 14, 3, METAL);
                fb.rect_fill(px + 3, yy, 14, 1, METAL + 2);
            }
            for k in 0..4 {
                fb.pset(px + 12 - k % 2, py + 6 + k * 2, ICE + 2);
            }
        }
        KCell::Hatch { item, dir, period } => {
            fb.rect_bevel(px, py + 1, 19, 18, FRAME + 1, FRAME + 3, FRAME);
            fb.rect_fill(px + 3, py + 4, 13, 12, OUTLINE);
            fb.blit(s.item(item), px + 4, py + 5, false);
            let ready = g.sim.timer[i] >= period;
            fb.rect_fill(px + 15, py + 2, 3, 2, if ready { GOOD } else { GOOD_DIM });
            arrow_tip(fb, cx, cy, dir, ACCENT_HI);
        }
        KCell::Belt(o) => {
            let mut ins = [false; 4];
            for side in Dir::ALL {
                ins[side.index()] = side != o && feeds_belt(g, x + side.dx(), y + side.dy(), side.opposite());
            }
            let phase = if g.phase == Phase::Running { (g.ticks as i32 * (g.speed as i32 + 1)) % 6 } else { 0 };
            draw_belt(fb, px, py, o, ins, phase);
        }
        KCell::Arm(d) => {
            // Base plate with a line to the tile behind and in front.
            filled_circle(fb, cx, cy, 5, METAL + 1, Some(METAL + 2));
            let (bx, by) = along(cx, cy, d, -9, 0);
            let (fx, fy) = along(cx, cy, d, 9, 0);
            fb.pset(bx, by, ACCENT);
            arrow_tip(fb, cx, cy, d, ACCENT);
            let _ = (fx, fy);
        }
        KCell::Machine(Machine::Slicer, d) => {
            draw_box(fb, px, py, METAL + 1, METAL + 3, METAL);
            fb.circle(cx, cy, 6, OUTLINE);
            filled_circle(fb, cx, cy, 5, METAL + 2, None);
            let spin = if busy(g.sim.timer[i]) || g.sim.slot[i].is_some() { (g.ticks % 4) as usize } else { 0 };
            let spokes: [(i32, i32); 4] = [(5, 0), (4, 4), (0, 5), (-4, 4)];
            let (ax, ay) = spokes[spin];
            fb.line(cx - ax, cy - ay, cx + ax, cy + ay, METAL + 3);
            fb.line(cx - ay, cy + ax, cx + ay, cy - ax, METAL + 3);
            fb.rect_fill(cx - 1, cy - 1, 3, 3, OUTLINE);
            arrow_tip(fb, cx, cy, d, ACCENT_HI);
        }
        KCell::Machine(Machine::Oven, d) => {
            draw_box(fb, px, py, BROWN, BROWN + 1, OUTLINE);
            fb.rect_fill(px + 4, py + 4, 12, 9, OUTLINE);
            let baking = g.sim.slot[i].is_some();
            fb.rect_fill(px + 5, py + 5, 10, 7, if baking { FIRE } else { FRAME });
            if baking {
                for k in 0..3 {
                    let flick = ((g.ticks / 3 + k) % 3) as i32;
                    fb.pset(px + 6 + k as i32 * 3, py + 10 - flick, GOLD + 2);
                }
            }
            // Progress pips: one per beat of the bake.
            let beats = Machine::Oven.beats() as i32;
            for k in 0..beats {
                let lit = baking && k < g.sim.timer[i] as i32;
                fb.rect_fill(px + 4 + k * 3, py + 15, 2, 2, if lit { FIRE } else { OUTLINE });
            }
            arrow_tip(fb, cx, cy, d, ACCENT_HI);
        }
        KCell::Counter(dish, d) => {
            draw_box(fb, px, py, PLATE, WHITE, PLATE_SH);
            // The dish this counter plates, dithered as a label.
            fb.blit(s.item(dish), px + 5, py + 3, false);
            fb.rect_checker(px + 5, py + 3, 10, 10, PLATE);
            // Part pips: filled as parts arrive.
            let mut k = 0;
            for &(part, n) in dish.recipe() {
                for j in 0..n {
                    let have = g.sim.parts[i][part.index()] > j;
                    let col = item_color(part);
                    let x0 = px + 3 + k * 4;
                    fb.rect_fill(x0, py + 14, 3, 3, if have { col } else { PLATE_SH });
                    k += 1;
                }
            }
            arrow_tip(fb, cx, cy, d, ACCENT);
        }
        KCell::Chute => {
            fb.rect_fill(px + 2, py + 2, 16, 16, ACCENT);
            fb.rect_fill(px + 3, py + 3, 14, 14, OUTLINE);
            fb.rect_fill(px + 4, py + 4, 12, 12, SPACE);
            for k in 0..3 {
                let yy = py + 13 - k * 4 - ((g.ticks / 4) % 4) as i32 % 2;
                for a in -2..=2i32 {
                    fb.pset(cx + a, yy + a.abs(), ACCENT_HI);
                }
            }
            fb.rect_fill(px + 2, py + 2, 16, 1, ACCENT_HI);
        }
    }
}

pub fn item_color(i: Item) -> u8 {
    match i {
        Item::Rye => GOLD + 1,
        Item::Berry | Item::Sliced => ICE + 1,
        Item::Moss => GREEN + 1,
        Item::Bean => PURP + 1,
        Item::Bread | Item::Roasted => BROWN + 1,
        _ => PLATE,
    }
}

fn draw_kitchen(g: &Game, s: &Sprites, fb: &mut Framebuffer) {
    fb.rect_bevel(LB_X - 4, BOARD_Y - 4, BOARD_PX + 8, BOARD_PX + 8, FRAME + 1, FRAME + 3, FRAME);
    for y in 0..N {
        for x in 0..N {
            let (px, py) = cell_px(Board::Kitchen, x, y);
            fb.rect_fill(px, py, TILE, TILE, KFLOOR + 1);
            fb.rect_fill(px, py, TILE, 1, KFLOOR);
            fb.rect_fill(px, py, 1, TILE, KFLOOR);
            fb.pset(px + 2, py + 2, KFLOOR + 2);
            fb.pset(px + 17, py + 17, KFLOOR);
            // A table stands above this square: mark the ceiling hatch spot.
            if g.r.table_at(x, y) {
                for k in (0..16).step_by(3) {
                    for (ax, ay) in [(px + 2 + k, py + 2), (px + 2 + k, py + 17), (px + 2, py + 2 + k), (px + 17, py + 2 + k)] {
                        fb.pset(ax, ay, WOOD + 1);
                    }
                }
            }
        }
    }
    for y in 0..N {
        for x in 0..N {
            draw_kcell(g, s, fb, x, y);
        }
    }

    // Items.
    let (num, den) = g.anim;
    for y in 0..N {
        for x in 0..N {
            let i = idx(x, y);
            let c = cell_center(Board::Kitchen, x, y);
            let from = g.sim.slot_from[i].unwrap_or(c);
            match g.sim.cells[i] {
                KCell::Belt(_) => {
                    if let Some(it) = g.sim.slot[i] {
                        let (ix, iy) = lerp(from, c, num, den);
                        fb.blit(s.item(it), ix - 5, iy - 5, false);
                    }
                }
                KCell::Machine(_, d) | KCell::Counter(_, d) => {
                    if let Some(it) = g.sim.out[i] {
                        fb.blit(s.item(it), c.0 - 5 + d.dx() * 5, c.1 - 5 + d.dy() * 5, false);
                    }
                }
                KCell::Chute => {
                    if let Some(it) = g.sim.slot[i] {
                        fb.blit(s.item(it), c.0 - 5, c.1 - 5, false);
                    }
                }
                KCell::Arm(d) => {
                    let (clx, cly) = arm_claw(g, x, y, d);
                    for t in -1..=0 {
                        fb.line(c.0 + t, c.1, clx + t, cly, METAL + 3);
                    }
                    fb.line(c.0, c.1 + 1, clx, cly + 1, OUTLINE);
                    if let Some(it) = g.sim.slot[i] {
                        fb.blit(s.item(it), clx - 5, cly - 5, false);
                    }
                    fb.rect_fill(clx - 1, cly - 1, 3, 3, ACCENT);
                    filled_circle(fb, c.0, c.1, 2, METAL + 3, None);
                }
                _ => {}
            }
            if let Some((it, _)) = g.sim.pass[i] {
                let (ix, iy) = lerp(from, c, num, den);
                fb.blit(s.item(it), ix - 5, iy - 5, false);
            }
        }
    }
}

// ---------------------------------------------------------------------------------------------

/// The ticket rail in the pass window between the rooms: one ticket per open order.
fn draw_rail(g: &Game, s: &Sprites, fb: &mut Framebuffer) {
    let (x0, x1) = (LB_X + BOARD_PX + 4, RB_X - 4);
    fb.rect_fill(x0, BOARD_Y - 4, x1 - x0, BOARD_PX + 8, FRAME);
    fb.rect_checker(x0, BOARD_Y - 4, x1 - x0, BOARD_PX + 8, FRAME + 1);
    fb.rect_fill(x0 + 2, BOARD_Y - 1, x1 - x0 - 4, 2, METAL + 2);
    fb.text_centered("ORDERS", (x0 + x1) / 2, BOARD_Y + 3, TEXT_DIM, None, 1);
    let open: Vec<_> = g.sim.guests.iter().filter(|gu| matches!(gu.state, GState::Queue | GState::Walking | GState::Waiting)).collect();
    for (k, gu) in open.iter().take(8).enumerate() {
        let (tx, ty) = (x0 + 3, BOARD_Y + 13 + k as i32 * 18);
        fb.rect_fill(tx, ty, x1 - x0 - 6, 16, OUTLINE);
        fb.rect_fill(tx + 1, ty + 1, x1 - x0 - 8, 14, PLATE);
        fb.pset(tx + (x1 - x0 - 6) / 2, ty - 1, METAL + 3);
        fb.blit(s.item(gu.order), tx + 2, ty + 3, false);
        let label = gu.table.map_or("..".to_string(), |(x, y)| square_name(x, y));
        fb.text(&label, tx + 15, ty + 5, if gu.claimed { GOOD_DIM } else { OUTLINE }, None, 1);
        let frac = gu.patience as i32 * (x1 - x0 - 10) / gu.max_patience.max(1) as i32;
        let col = patience_color(gu.patience, gu.max_patience);
        fb.rect_fill(tx + 2, ty + 13, frac, 1, col);
    }
    if open.len() > 8 {
        fb.text_centered(&format!("+{}", open.len() - 8), (x0 + x1) / 2, BOARD_Y + BOARD_PX - 8, TEXT, Some(OUTLINE), 1);
    }
}

fn patience_color(p: u16, max: u16) -> u8 {
    let f = p as u32 * 100 / max.max(1) as u32;
    if f > 50 {
        GOOD
    } else if f > 25 {
        ORANGE + 1
    } else {
        RED
    }
}

fn draw_dining(g: &Game, s: &Sprites, fb: &mut Framebuffer) {
    fb.rect_bevel(RB_X - 4, BOARD_Y - 4, BOARD_PX + 8, BOARD_PX + 8, FRAME + 1, FRAME + 3, FRAME);
    for y in 0..N {
        for x in 0..N {
            let (px, py) = cell_px(Board::Dining, x, y);
            // Warm carpet with a small diamond motif on alternate squares.
            fb.rect_fill(px, py, TILE, TILE, DFLOOR + 1);
            fb.rect_fill(px, py, TILE, 1, DFLOOR);
            fb.rect_fill(px, py, 1, TILE, DFLOOR);
            if (x + y) % 2 == 0 {
                for (a, b) in [(0, -2), (-2, 0), (2, 0), (0, 2), (0, 0)] {
                    fb.pset(px + 10 + a, py + 10 + b, DFLOOR + 2);
                }
            }
        }
    }
    // Furniture.
    for y in 0..N {
        for x in 0..N {
            let (px, py) = cell_px(Board::Dining, x, y);
            let (cx, cy) = (px + 10, py + 10);
            match g.sim.dining[idx(x, y)] {
                DCell::Table => {
                    // A round wooden table with the brass ring of the chute hatch in the middle.
                    filled_circle(fb, cx, cy + 2, 9, OUTLINE, None);
                    filled_circle(fb, cx, cy, 9, WOOD + 1, Some(WOOD + 2));
                    fb.circle(cx, cy, 6, WOOD);
                    filled_circle(fb, cx, cy, 3, ACCENT, Some(ACCENT_HI));
                    fb.rect_fill(cx - 1, cy - 1, 3, 3, OUTLINE);
                }
                DCell::Seat(d) => {
                    // A cushioned chair, its back away from the table.
                    let away = d.opposite();
                    let (sx, sy) = (cx + away.dx() * 2, cy + away.dy() * 2);
                    fb.rect_fill(sx - 5, sy - 4, 10, 9, OUTLINE);
                    fb.rect_fill(sx - 4, sy - 3, 8, 7, RED);
                    fb.rect_fill(sx - 4, sy - 3, 8, 1, PINK);
                    let (bx, by) = (sx + away.dx() * 4, sy + away.dy() * 4);
                    if d.dx() != 0 {
                        fb.rect_fill(bx - 1, by - 5, 3, 11, OUTLINE);
                        fb.rect_fill(bx, by - 4, 1, 9, WOOD + 2);
                    } else {
                        fb.rect_fill(bx - 5, by - 1, 11, 3, OUTLINE);
                        fb.rect_fill(bx - 4, by, 9, 1, WOOD + 2);
                    }
                }
                DCell::Door => {
                    fb.rect_bevel(px + 1, py + 1, 18, 18, FRAME + 2, FRAME + 3, FRAME);
                    fb.rect_fill(px + 4, py + 3, 12, 16, METAL);
                    fb.rect_fill(px + 9, py + 3, 1, 16, OUTLINE);
                    let open = g.sim.guests.iter().any(|gu| (gu.x, gu.y) == (x, y) && gu.state != GState::Gone);
                    fb.rect_fill(px + 8, py + 1, 4, 2, if open { GOOD } else { GOOD_DIM });
                }
                DCell::Plant => {
                    fb.rect_fill(px + 6, py + 12, 8, 7, BROWN);
                    fb.rect_fill(px + 6, py + 12, 8, 1, BROWN + 2);
                    for (a, b) in [(-5, -2), (-3, -6), (0, -8), (3, -6), (5, -2), (0, -3)] {
                        filled_circle(fb, cx + a, cy + b, 2, GREEN + 1, Some(GREEN + 2));
                    }
                }
                DCell::Wall => fb.rect_bevel(px, py, 20, 20, FRAME + 1, FRAME + 2, FRAME),
                DCell::Floor => {}
            }
        }
    }
    // Dishes on tables, rising out of the chute hatch.
    for i in 0..64 {
        if let Some(it) = g.sim.table_dish[i] {
            let (cx, cy) = cell_center(Board::Dining, i as i32 % N, i as i32 / N);
            let rise = g.sim.table_pop[i] as i32;
            fb.blit(s.item(it), cx - 5, cy - 5 + rise, false);
        }
    }
    // Guests, back to front.
    let (num, den) = g.anim;
    let mut order: Vec<usize> = (0..g.sim.guests.len()).filter(|&k| g.sim.guests[k].state != GState::Gone).collect();
    order.sort_by_key(|&k| lerp(g.sim.guests[k].from, g.sim.guests[k].to, num, den).1);
    for k in order {
        let gu = &g.sim.guests[k];
        let (x, y) = lerp(gu.from, gu.to, num, den);
        let walking = matches!(gu.state, GState::Walking | GState::Leaving { .. }) && gu.from != gu.to;
        let bob = if walking && (g.ticks / 2) % 2 == 0 { -1 } else { 0 };
        let seated = matches!(gu.state, GState::Waiting | GState::Eating);
        let (sx, sy) = (x - 6, y - 13 + bob + if seated { 2 } else { 0 });
        fb.blit(s.guest(gu.species), sx, sy, false);
        match gu.state {
            GState::Waiting | GState::Queue => {
                // Order bubble; its border tells how patient they still are.
                let col = patience_color(gu.patience, gu.max_patience);
                let (bx, by) = (sx - 1, sy - 15);
                fb.rect_fill(bx, by, 14, 13, col);
                fb.rect_fill(bx + 1, by + 1, 12, 11, PLATE);
                fb.blit(s.item(gu.order), bx + 2, by + 1, false);
                fb.pset(bx + 6, by + 13, col);
                fb.pset(bx + 7, by + 13, col);
                fb.pset(bx + 6, by + 14, col);
            }
            GState::Eating => {
                if let Some((tx, ty)) = gu.table {
                    let (cx, cy) = cell_center(Board::Dining, tx, ty);
                    let (ex, ey) = ((x + cx) / 2, (y + cy) / 2);
                    if (g.ticks / 3) % 2 == 0 || g.phase != Phase::Running {
                        fb.blit(s.item(gu.order), ex - 5, ey - 5, false);
                    }
                }
                if gu.timer + 10 > g.r.service.eat_beats {
                    fb.blit(&s.heart, x - 3, sy - 8, false);
                }
            }
            GState::Leaving { happy: false } => fb.blit(&s.cloud, x - 5, sy - 8, false),
            _ => {}
        }
    }
    // Coin pop-ups.
    for p in &g.pops {
        let rise = (24 - p.ttl as i32) / 2;
        fb.blit(&s.coin, p.x - 10, p.y - rise, false);
        fb.text(&format!("+{}", p.coins), p.x - 2, p.y - rise, GOLD + 2, Some(OUTLINE), 1);
    }
}

fn draw_cursor(g: &Game, fb: &mut Framebuffer) {
    let (b, x, y) = g.cursor_cell();
    let (px, py) = cell_px(b, x, y);
    let other = if b == Board::Kitchen { Board::Dining } else { Board::Kitchen };
    let (ox, oy) = cell_px(other, x, y);
    // The same square in the other room: a chute here would join the two.
    for k in (0..22).step_by(2) {
        for (ax, ay) in [(ox - 1 + k, oy - 1), (ox - 1 + k, oy + 20), (ox - 1, oy - 1 + k), (ox + 20, oy - 1 + k)] {
            fb.pset(ax, ay, ACCENT);
        }
    }
    if b == Board::Kitchen && g.phase != Phase::Closed {
        let tool = g.r.shelf[g.tool].0;
        let free = g.layout[idx(x, y)].is_none() && !g.is_fixed(x, y);
        let allowed = tool != Tool::Chute || g.r.table_at(x, y);
        if free && allowed && g.shelf_left(g.tool) != Some(0) {
            ghost(fb, px, py, tool, g.ticks);
        }
    }
    brackets(fb, px - 1, py - 1, TILE + 2, TILE + 2, CURSOR, 1);
}

/// A dithered preview of the part that would be placed.
fn ghost(fb: &mut Framebuffer, px: i32, py: i32, tool: Tool, _ticks: u64) {
    let (cx, cy) = (px + 10, py + 10);
    match tool {
        Tool::Belt => fb.rect_checker(cx - 6, cy - 6, 12, 12, BELT + 2),
        Tool::Chute => fb.rect_checker(px + 3, py + 3, 14, 14, ACCENT),
        _ => fb.rect_checker(px + 2, py + 2, 16, 16, METAL + 2),
    }
}

// ---------------------------------------------------------------------------------------------

fn draw_header(g: &Game, s: &Sprites, fb: &mut Framebuffer) {
    fb.text(g.r.name, 8, 5, ACCENT_HI, Some(OUTLINE), 1);
    let (label, col) = match g.phase {
        Phase::Prep => ("PREP", TEXT),
        Phase::Running => ("SERVICE", GOOD),
        Phase::Paused => ("PAUSED", ACCENT_HI),
        Phase::Closed => ("CLOSED", TEXT_DIM),
    };
    fb.text(label, 8, 16, col, Some(OUTLINE), 1);
    // The evening's progress, with a tick where the last guests arrive.
    let (bx, by, bw) = (60, 18, 64);
    fb.rect_fill(bx - 1, by - 1, bw + 2, 6, OUTLINE);
    let frac = (g.sim.beat.min(g.sim.service_beats()) as i32 * bw) / g.sim.service_beats().max(1) as i32;
    fb.rect_fill(bx, by, bw, 4, FRAME);
    fb.rect_fill(bx, by, frac, 4, ACCENT);
    fb.rect_fill(bx + bw * 9 / 10, by - 1, 1, 6, TEXT_DIM);

    let mut x = 136;
    fb.blit(&s.coin, x, 7, false);
    fb.text(&g.sim.coins.to_string(), x + 9, 7, GOLD + 2, Some(OUTLINE), 1);
    x += 40;
    fb.blit(&s.heart, x, 7, false);
    fb.text(&format!("{}/{}", g.sim.fed, g.sim.total_guests()), x + 10, 7, TEXT, Some(OUTLINE), 1);
    x = 136;
    fb.blit(&s.cloud, x - 1, 17, false);
    fb.text(&g.sim.hungry.to_string(), x + 11, 18, if g.sim.hungry > 0 { RED } else { TEXT_DIM }, Some(OUTLINE), 1);
    if g.best > 0 {
        fb.text("BEST", x + 40, 18, TEXT_DIM, None, 1);
        for k in 0..g.best as i32 {
            mini_star(fb, x + 66 + k * 7, 18);
        }
    }

    for (b, (bx, by, bw, bh)) in BUTTONS {
        let (label, lit) = match b {
            Button::Run => match g.phase {
                Phase::Prep => ("OPEN", true),
                Phase::Running => ("PAUSE", false),
                Phase::Paused => ("GO ON", true),
                Phase::Closed => ("AGAIN", true),
            },
            Button::Restart => ("RESTART", false),
            Button::Speed => (["1X", "2X", "", "4X"][(g.speed.clamp(1, 4) - 1) as usize], g.speed > 1),
        };
        fb.rect_bevel(bx, by, bw, bh, if lit { FRAME + 3 } else { FRAME + 2 }, METAL + 3, FRAME);
        let tw = Framebuffer::text_width(label, 1);
        fb.text(label, bx + (bw - tw) / 2, by + 6, TEXT, Some(OUTLINE), 1);
    }
}

fn tool_hint(t: Tool) -> &'static str {
    match t {
        Tool::Belt => "BELT: HOLD SPACE OR DRAG TO DRAW. TAP ONE TO TURN IT",
        Tool::Arm => "ARM: LIFTS FROM BEHIND, SETS DOWN IN FRONT - ONLY WHAT FITS",
        Tool::Machine(Machine::Slicer) => "SLICER: FROSTBERRY -> SLICED BERRIES (2 BEATS)",
        Tool::Machine(Machine::Oven) => "OVEN: ICE-RYE -> BREAD, PULSAR BEANS -> ROAST (4 BEATS)",
        Tool::Counter(_) => "COUNTER: PLATES ITS DISH ONCE THE PARTS ARE IN AND IT'S ORDERED",
        Tool::Chute => "CHUTE: GOES UNDER A TABLE. SENDS UP DISHES ORDERED THERE",
    }
}

/// What the piece or guest under the cursor is.
fn look(g: &Game) -> Option<String> {
    let (b, x, y) = g.cursor_cell();
    let i = idx(x, y);
    if b == Board::Dining {
        if let Some(gu) = g.sim.guests.iter().find(|gu| (gu.x, gu.y) == (x, y) && gu.state != GState::Gone) {
            let mood = match gu.state {
                GState::Waiting | GState::Queue | GState::Walking => format!("WANTS {}", gu.order.name()),
                GState::Eating => "EATING HAPPILY".into(),
                GState::Leaving { happy: true } => "LEAVING HAPPY".into(),
                _ => "LEAVING HUNGRY".into(),
            };
            return Some(format!("{}: {mood}", gu.species.name()));
        }
        return Some(match g.sim.dining[i] {
            DCell::Table => format!("TABLE {}: A CHUTE UNDER KITCHEN {} SERVES IT", square_name(x, y), square_name(x, y)),
            DCell::Seat(_) => "A SEAT".into(),
            DCell::Door => "THE AIRLOCK DOOR: GUESTS COME IN HERE".into(),
            DCell::Plant => "A HYDROPONIC FERN".into(),
            _ => return None,
        });
    }
    Some(match g.sim.cells[i] {
        KCell::Hatch { item, period, .. } => format!("PANTRY HATCH: ONE {} EVERY {period} BEATS", item.name()),
        KCell::Wall => "COOLANT PIPE".into(),
        KCell::Belt(_) => match g.sim.slot[i] {
            Some(it) => format!("BELT CARRYING {}", it.name()),
            None => "BELT".into(),
        },
        KCell::Arm(_) => "GRABBER ARM".into(),
        KCell::Machine(m, _) => m.name().into(),
        KCell::Counter(dish, _) => {
            let parts: Vec<String> = dish.recipe().iter().map(|(p, n)| format!("{n} {}", p.name())).collect();
            format!("{} COUNTER: {}", dish.name(), parts.join(" + "))
        }
        KCell::Chute => format!("CHUTE UP TO TABLE {}", square_name(x, y)),
        KCell::Floor if g.r.table_at(x, y) => format!("TABLE {} IS RIGHT ABOVE THIS SQUARE", square_name(x, y)),
        KCell::Floor => return None,
    })
}

fn tool_icon(fb: &mut Framebuffer, s: &Sprites, x: i32, y: i32, t: Tool) {
    let (cx, cy) = (x + 10, y + 10);
    match t {
        Tool::Belt => {
            let mut ins = [false; 4];
            ins[Dir::W.index()] = true;
            draw_belt(fb, x, y, Dir::E, ins, 0);
        }
        Tool::Arm => {
            filled_circle(fb, cx, cy, 5, METAL + 1, Some(METAL + 2));
            fb.line(cx, cy, cx + 6, cy - 4, METAL + 3);
            fb.rect_fill(cx + 5, cy - 5, 3, 3, ACCENT);
        }
        Tool::Machine(Machine::Slicer) => {
            draw_box(fb, x, y, METAL + 1, METAL + 3, METAL);
            fb.circle(cx, cy, 5, OUTLINE);
            fb.line(cx - 4, cy, cx + 4, cy, METAL + 3);
            fb.line(cx, cy - 4, cx, cy + 4, METAL + 3);
        }
        Tool::Machine(Machine::Oven) => {
            draw_box(fb, x, y, BROWN, BROWN + 1, OUTLINE);
            fb.rect_fill(x + 5, y + 5, 10, 7, FIRE);
        }
        Tool::Counter(d) => {
            draw_box(fb, x, y, PLATE, WHITE, PLATE_SH);
            fb.blit(s.item(d), x + 5, y + 5, false);
        }
        Tool::Chute => {
            fb.rect_fill(x + 2, y + 2, 16, 16, ACCENT);
            fb.rect_fill(x + 4, y + 4, 12, 12, SPACE);
            for a in -2..=2i32 {
                fb.pset(cx + a, cy - 1 + a.abs(), ACCENT_HI);
            }
        }
    }
}

fn draw_bottom(g: &Game, s: &Sprites, fb: &mut Framebuffer) {
    fb.rect_bevel(4, 206, 392, 42, FRAME + 1, FRAME + 3, FRAME);
    for (k, &(tool, _)) in g.r.shelf.iter().enumerate() {
        let (x, y, w, h) = shelf_rect(k);
        let sel = k == g.tool;
        fb.rect_bevel(x, y, w, h, if sel { FRAME + 3 } else { FRAME }, if sel { METAL + 3 } else { FRAME }, FRAME + 2);
        tool_icon(fb, s, x + 3, y + 3, tool);
        let left = g.shelf_left(k);
        if left == Some(0) {
            fb.rect_checker(x + 1, y + 1, w - 2, h - 2, FRAME);
        }
        if let Some(n) = left {
            fb.text(&n.to_string(), x + w - 7, y + h - 8, TEXT, Some(OUTLINE), 1);
        }
        fb.text(&(k + 1).to_string(), x + 1, y + 1, TEXT_DIM, None, 1);
        if sel {
            brackets(fb, x - 2, y - 2, w + 4, h + 4, CURSOR, 1);
        }
    }
    // Tonight's menu: dish = parts, price.
    let mx = 246;
    fb.rect_fill(mx - 2, 209, 148, 36, OUTLINE);
    fb.rect_fill(mx - 1, 210, 146, 34, PLATE_SH);
    let menu: Vec<Item> = g.r.service.menu.iter().map(|(_, d)| *d).collect();
    let open_now = g.r.menu_at(g.sim.beat);
    for (k, dish) in menu.iter().enumerate() {
        let y = 211 + k as i32 * 11;
        fb.blit(s.item(*dish), mx, y, false);
        let mut x = mx + 13;
        fb.text("=", x, y + 2, OUTLINE, None, 1);
        x += 8;
        for &(p, n) in dish.recipe() {
            for _ in 0..n {
                fb.blit(s.item(p), x, y, false);
                x += 11;
            }
        }
        if open_now.contains(dish) {
            fb.blit(&s.coin, mx + 118, y + 2, false);
            fb.text(&dish.price().to_string(), mx + 126, y + 2, OUTLINE, None, 1);
        } else {
            fb.rect_checker(mx - 1, y, 146, 11, PLATE_SH);
            fb.text("LATER", mx + 112, y + 2, OUTLINE, None, 1);
        }
    }

    // Message or what's under the cursor, then the tool hint and keys.
    if let Some((text, kind)) = &g.message {
        let col = match *kind {
            MSG_BAD => RED,
            MSG_GOOD => GOOD,
            _ => ACCENT_HI,
        };
        fb.text_centered(text, 200, 252, col, Some(OUTLINE), 1);
    } else if let Some(text) = look(g) {
        fb.text_centered(&text, 200, 252, TEXT_DIM, Some(OUTLINE), 1);
    }
    fb.text_centered(tool_hint(g.r.shelf[g.tool].0), 200, 264, TEXT, Some(OUTLINE), 1);
    let note = g.r.note;
    let mut lines = vec![String::new()];
    for w in note.split_whitespace() {
        if lines.last().unwrap().len() + w.len() + 1 > 64 {
            lines.push(String::new());
        }
        let l = lines.last_mut().unwrap();
        if !l.is_empty() {
            l.push(' ');
        }
        l.push_str(w);
    }
    for (k, l) in lines.iter().take(2).enumerate() {
        fb.text_centered(l, 200, 276 + k as i32 * 9, TEXT_DIM, None, 1);
    }
}

fn mini_star(fb: &mut Framebuffer, x: i32, y: i32) {
    const S: [&str; 6] = ["..k..", ".kyk.", "kyyyk", ".kyk.", "kk.kk", "....."];
    for (r, row) in S.iter().enumerate() {
        for (c, ch) in row.chars().enumerate() {
            match ch {
                'k' => fb.pset(x + c as i32, y + r as i32, OUTLINE),
                'y' => fb.pset(x + c as i32, y + r as i32, GOLD + 2),
                _ => {}
            }
        }
    }
}

fn star(fb: &mut Framebuffer, cx: i32, cy: i32, lit: bool) {
    const STAR: [&str; 9] = ["....k....", "...kyk...", "...kyk...", "kkkkyykkk", "kyyyyyyyk", ".kyyyyyk.", "..kyyyk..", ".kyykyyk.", ".kk...kk."];
    for (y, row) in STAR.iter().enumerate() {
        for (x, ch) in row.chars().enumerate() {
            let c = match ch {
                'k' => OUTLINE,
                'y' => {
                    if lit {
                        GOLD + 2
                    } else {
                        FRAME
                    }
                }
                _ => continue,
            };
            fb.pset(cx - 4 + x as i32, cy - 4 + y as i32, c);
        }
    }
}

fn draw_tally(g: &Game, fb: &mut Framebuffer) {
    fb.rect_checker(LB_X, BOARD_Y, RB_X + BOARD_PX - LB_X, BOARD_PX, OUTLINE);
    let (x, y, w, h) = (110, 66, 180, 112);
    fb.rect_bevel(x, y, w, h, FRAME + 2, METAL + 3, FRAME);
    fb.text_centered("CLOSING TIME", 200, y + 8, ACCENT_HI, Some(OUTLINE), 2);
    let stars = g.sim.stars();
    for k in 0..3 {
        star(fb, 200 - 14 + k * 14, y + 32, k < stars as i32);
    }
    let rows = [
        format!("FED {} OF {}", g.sim.fed, g.sim.total_guests()),
        format!("LEFT HUNGRY {}", g.sim.hungry),
        format!("COINS {}", g.sim.coins),
    ];
    for (k, r) in rows.iter().enumerate() {
        fb.text_centered(r, 200, y + 46 + k as i32 * 11, TEXT, Some(OUTLINE), 1);
    }
    fb.text_centered("ENTER: SERVE ANOTHER EVENING", 200, y + 86, GOOD, Some(OUTLINE), 1);
    fb.text_centered("YOUR KITCHEN STAYS AS BUILT", 200, y + 97, TEXT_DIM, None, 1);
}

fn draw_help(fb: &mut Framebuffer) {
    fb.rect_checker(0, 0, 400, 300, OUTLINE);
    fb.rect_bevel(30, 26, 340, 248, FRAME + 2, METAL + 3, FRAME);
    fb.text_centered("HOW THE KITCHEN WORKS", 200, 34, ACCENT_HI, Some(OUTLINE), 1);
    let rules = [
        "HATCHES PUT OUT INGREDIENTS. BELTS CARRY THEM.",
        "MACHINES AND COUNTERS LET THROUGH WHAT THEY DON'T USE.",
        "COUNTERS PLATE A DISH ONLY WHEN SOMEONE HAS ORDERED IT.",
        "ARMS LIFT ONLY WHAT THE TILE IN FRONT CAN USE.",
        "A CHUTE SENDS A DISH UP TO THE TABLE ON ITS SQUARE",
        "IF SOMEONE THERE ORDERED IT. OTHER DISHES ROLL BY.",
        "A FULL LOOP OF BELTS KEEPS TURNING.",
    ];
    for (k, r) in rules.iter().enumerate() {
        fb.text_centered(r, 200, 50 + k as i32 * 10, TEXT, None, 1);
    }
    let keys: [(&str, &str); 11] = [
        ("ARROWS / WASD", "MOVE (ACROSS BOTH ROOMS)"),
        ("SPACE / CLICK", "PLACE - HOLD + MOVE DRAWS BELTS"),
        ("R / WHEEL", "TURN"),
        ("X / RIGHT CLICK", "REMOVE"),
        ("1-8  Q  E", "PICK A PART"),
        ("BACKSPACE   C", "UNDO   CLEAR THE KITCHEN"),
        ("ENTER", "OPEN / PAUSE / GO ON"),
        ("F5", "RESTART THE EVENING"),
        ("F", "SPEED 1X 2X 4X"),
        ("TAB  F2", "OTHER ROOM   SCANLINES"),
        ("ESC", "PAUSE, THEN QUIT"),
    ];
    for (k, (a, b)) in keys.iter().enumerate() {
        let y = 130 + k as i32 * 10;
        fb.text(a, 46, y, ACCENT_HI, None, 1);
        fb.text(b, 152, y, TEXT, None, 1);
    }
    fb.text_centered("BUILD ANY TIME, EVEN MID-SERVICE. F1 CLOSES.", 200, 254, TEXT_DIM, None, 1);
}
