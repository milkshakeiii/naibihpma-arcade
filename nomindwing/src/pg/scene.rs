//! Drawing the world. The arena and the match show the same things (space, rocks, loose pixels,
//! ships, shots), so they draw them with the same code: something new on the map is drawn once,
//! here, and looks the same on both screens. Each screen adds only what's its own (the arena's
//! stations, the match's network links and seed courses).
//!
//! Everything takes the field it draws into and the camera: the world point at the field's top
//! left. A world point (x, y) lands on screen at (x - cam.0, y - cam.1).

use super::design::{Part, SCOOP_CARRY, SHIELD_R};
use super::sim::*;
use super::{clipped_line, draw_shape, pc};
use crate::framebuffer::Framebuffer;
use crate::palette::*;
use crate::ui::{self, Rect};

/// Space, stars and rocks.
pub fn backdrop(fb: &mut Framebuffer, f: Rect, a: &Arena, (cx, cy): (i32, i32)) {
    fb.rect_fill(f.x, f.y, f.w, f.h, SPACE);
    for &(x, y, c) in &a.stars {
        pc(fb, f, x - cx, y - cy, STAR + c);
    }
    for sy in f.y..f.y + f.h {
        for sx in f.x..f.x + f.w {
            let (x, y) = (sx + cx, sy + cy);
            if a.rock_at(x, y) {
                let lit = !a.rock_at(x, y - 1) || !a.rock_at(x - 1, y);
                let dark = !a.rock_at(x, y + 1) || !a.rock_at(x + 1, y);
                fb.pset(sx, sy, if lit && !dark { ROCK + 3 } else if dark { ROCK } else { ROCK + 1 });
            }
        }
    }
}

/// Loose pixels, and the trails behind ships.
pub fn drift(fb: &mut Framebuffer, f: Rect, a: &Arena, (cx, cy): (i32, i32)) {
    for d in &a.debris {
        pc(fb, f, d.x - cx, d.y - cy, LOOSE);
    }
    for s in a.ships.iter().filter(|s| s.alive && s.warp == 0) {
        for &(x, y) in &s.trail {
            pc(fb, f, x - cx, y - cy, team_col(s.team, 0));
        }
    }
}

/// A ship on the map, with its cannon barrels showing where each turret points. Pixels still to
/// grow show as a faint dither of where they'll be; a ship arriving by hyperspace is revealed
/// behind a sweeping blue line.
pub fn ship(fb: &mut Framebuffer, f: Rect, s: &Ship, (cx, cy): (i32, i32)) {
    let (x, y) = (s.x - cx, s.y - cy);
    if s.warp > 0 {
        let (l, t, r, b) = s.bbox();
        let w = r - l + 1;
        let line = l - cx + w * (WARP_TICKS - s.warp) as i32 / WARP_TICKS as i32;
        let clip = Rect::new(f.x, f.y, (line - f.x).clamp(0, f.w), f.h);
        draw_shape(fb, clip, &s.shape, x, y, s.facing, s.team, &|c| s.hp[c] > 0, false);
        for k in t - 3..=b + 3 {
            pc(fb, f, line, k - cy, ACCENT_HI);
            pc(fb, f, line - 1, k - cy, ACCENT);
            pc(fb, f, line + 1, k - cy, ACCENT);
        }
        return;
    }
    if s.grow.is_some() {
        for c in 0..s.hp.len() {
            if s.shape.cells[c].is_some() && s.hp[c] == 0 {
                let (px, py) = s.cell_xy(c);
                if (px + py) & 1 == 0 {
                    pc(fb, f, px - cx, py - cy, team_col(s.team, 0));
                }
            }
        }
    }
    draw_shape(fb, f, &s.shape, x, y, s.facing, s.team, &|c| s.hp[c] > 0, false);
    // What it's carrying shows in its scoops: one turns the colour of loose pixels for each
    // scoopful aboard, so a miner fills up before your eyes and flies home full.
    if s.carry > 0 {
        let full = ((s.carry + SCOOP_CARRY - 1) / SCOOP_CARRY) as usize;
        for c in (0..s.hp.len()).filter(|&c| s.hp[c] > 0 && s.shape.cells[c] == Some(Part::Scoop)).take(full) {
            let (px, py) = s.cell_xy(c);
            pc(fb, f, px - cx, py - cy, LOOSE);
        }
    }
    for t in s.turrets.iter().filter(|t| s.hp[t.cell] > 0) {
        let (gx, gy) = s.cell_xy(t.cell);
        let (ux, uy) = dir64(t.aim);
        let (ex, ey) = (gx + ux * BARREL / 1024, gy + uy * BARREL / 1024);
        clipped_line(fb, f, gx - cx, gy - cy, ex - cx, ey - cy, GUN, true);
    }
    shields(fb, f, s, (cx, cy));
}

/// The circles a ship's shield pixels throw, as one outline: a circle's edge shows only where no
/// other of the ship's circles covers it. A full circle is a solid line, and it thins by a
/// quarter of its pixels for each shot it has stopped, so what's left of it can be read at a
/// glance. A circle that's out isn't drawn.
fn shields(fb: &mut Framebuffer, f: Rect, s: &Ship, (cx, cy): (i32, i32)) {
    if s.st.shields == 0 {
        return;
    }
    let up: Vec<((i32, i32), u8)> = s.live_shields().into_iter().map(|c| (s.cell_xy(c), s.charge[c])).collect();
    let (r2, band) = (SHIELD_R * SHIELD_R, SHIELD_R);
    for (k, &((sx, sy), charge)) in up.iter().enumerate() {
        for dy in -SHIELD_R..=SHIELD_R {
            for dx in -SHIELD_R..=SHIELD_R {
                // The ring of pixels at the circle's edge.
                let d = dx * dx + dy * dy;
                if d > r2 || d <= r2 - 2 * band {
                    continue;
                }
                if ((dx + dy) & 3) >= charge as i32 {
                    continue;
                }
                let (x, y) = (sx + dx, sy + dy);
                let covered = up.iter().enumerate().any(|(j, &((ox, oy), _))| j != k && (x - ox) * (x - ox) + (y - oy) * (y - oy) <= r2 - 2 * band);
                if !covered {
                    pc(fb, f, x - cx, y - cy, SHIELD_DIM);
                }
            }
        }
    }
}

/// A projector's skin. Its reach is a dotted line: any friend with part of itself inside that
/// line is covered. Every ship one covers wears a blue outline two pixels outside its box, with
/// its corners cut. A full pool is a solid outline, and it thins by a quarter for each quarter of
/// the pool that's spent, as a shield's circle does. With the pool empty there's no outline:
/// nothing's covered.
fn skins(fb: &mut Framebuffer, f: Rect, a: &Arena, (cx, cy): (i32, i32)) {
    // The reach: the lump's box grown by its reach, with round corners. A dot every fourth pixel.
    for (_, _, (l, t, r, b), reach) in a.proj_fields() {
        let (l, t, r, b) = (l - cx, t - cy, r - cx, b - cy);
        for x in (l..=r).filter(|x| x & 3 == 0) {
            pc(fb, f, x, t - reach, PROJECTOR_DIM);
            pc(fb, f, x, b + reach, PROJECTOR_DIM);
        }
        for y in (t..=b).filter(|y| y & 3 == 0) {
            pc(fb, f, l - reach, y, PROJECTOR_DIM);
            pc(fb, f, r + reach, y, PROJECTOR_DIM);
        }
        // Each corner's quarter circle, walked along whichever way it runs flatter so the dots
        // stay evenly apart.
        for k in (4..reach * 181 / 256).step_by(4) {
            let far = isqrt(reach * reach - k * k);
            for (dx, dy) in [(k, far), (far, k)] {
                pc(fb, f, r + dx, b + dy, PROJECTOR_DIM);
                pc(fb, f, r + dx, t - dy, PROJECTOR_DIM);
                pc(fb, f, l - dx, b + dy, PROJECTOR_DIM);
                pc(fb, f, l - dx, t - dy, PROJECTOR_DIM);
            }
        }
    }
    for (i, s) in a.ships.iter().enumerate() {
        if !s.alive || s.warp > 0 {
            continue;
        }
        let Some(p) = a.skins.get(i).and_then(|by| by.iter().copied().find(|&p| a.ships[p].alive && a.ships[p].skin.0 > 0)) else { continue };
        let (left, max) = (a.ships[p].skin.0, a.ships[p].st.projector.max(1));
        let quarters = (left * 4 + max - 1) / max;
        let (l, t, r, b) = s.bbox();
        let (l, t, r, b) = (l - 2 - cx, t - 2 - cy, r + 2 - cx, b + 2 - cy);
        // Once round, clockwise from the top, a corner cut two pixels deep at each turn.
        const CUT: i32 = 2;
        let mut round: Vec<(i32, i32)> = Vec::new();
        round.extend((l + CUT..=r - CUT).map(|x| (x, t)));
        round.extend((1..CUT).map(|k| (r - CUT + k, t + k)));
        round.extend((t + CUT..=b - CUT).map(|y| (r, y)));
        round.extend((1..CUT).map(|k| (r - k, b - CUT + k)));
        round.extend((l + CUT..=r - CUT).rev().map(|x| (x, b)));
        round.extend((1..CUT).map(|k| (l + CUT - k, b - k)));
        round.extend((t + CUT..=b - CUT).rev().map(|y| (l, y)));
        round.extend((1..CUT).map(|k| (l + k, t + CUT - k)));
        for (k, &(x, y)) in round.iter().enumerate() {
            if (k as i32 & 3) < quarters {
                pc(fb, f, x, y, PROJECTOR);
            }
        }
    }
}

/// The white corners round the ship under the pointer.
pub fn corners(fb: &mut Framebuffer, s: &Ship, (cx, cy): (i32, i32)) {
    let (l, t, r, b) = s.bbox();
    fb.corners(l - 2 - cx, t - 2 - cy, r - l + 5, b - t + 5, 2, WHITE);
}

/// Shots in flight, and the sparks and blasts where they landed. A beam that's warming shows
/// where it will cut as a dotted line, and one that has just cut as a solid one.
pub fn fire(fb: &mut Framebuffer, f: Rect, a: &Arena, (cx, cy): (i32, i32)) {
    skins(fb, f, a, (cx, cy));
    for w in a.warming() {
        for k in (2..=w.reach).step_by(2) {
            pc(fb, f, w.from.0 + w.dir.0 * k - cx, w.from.1 + w.dir.1 * k - cy, BEAM);
        }
    }
    for c in &a.cuts {
        clipped_line(fb, f, c.from.0 - cx, c.from.1 - cy, c.to.0 - cx, c.to.1 - cy, if c.t >= 2 { BEAM_HI } else { BEAM }, true);
    }
    for sh in &a.shots {
        let (p, q) = ((sh.prev.0 - cx, sh.prev.1 - cy), (sh.line.x - cx, sh.line.y - cy));
        let (c0, c1) = match sh.kind {
            Kind::Lance => (LANCE, LANCE_HI),
            Kind::Shell => (GOLD, WHITE),
            Kind::Missile => (MISSILE, MISSILE_HI),
            Kind::Repeat => (REPEATER, WHITE),
            Kind::Flak => (team_col(sh.team, 1), team_col(sh.team, 3)),
        };
        pc(fb, f, p.0, p.1, c0);
        pc(fb, f, q.0, q.1, c1);
    }
    for sp in &a.sparks {
        let (x, y) = (sp.x - cx, sp.y - cy);
        if sp.blast {
            for k in 0..9 {
                let (dx, dy) = (k % 3 - 1, k / 3 - 1);
                let c = if sp.t >= 3 {
                    if k == 4 {
                        WHITE
                    } else {
                        GOLD
                    }
                } else if dx == 0 || dy == 0 {
                    SCOOP
                } else {
                    continue;
                };
                pc(fb, f, x + dx, y + dy, c);
            }
        } else {
            pc(fb, f, x, y, if sp.t >= 2 { WHITE } else { GOLD });
        }
    }
}

/// The minimap's frame, with space and the rocks at one map pixel per `scale` world pixels.
/// Returns the map area inside the frame, for the screen to put its own dots on.
pub fn minimap_ground(fb: &mut Framebuffer, r: Rect, a: &Arena, scale: i32) -> Rect {
    ui::inset(fb, r);
    let inner = Rect::new(r.x + 1, r.y + 1, a.w / scale, a.h / scale);
    fb.rect_fill(inner.x, inner.y, inner.w, inner.h, SPACE);
    for my in 0..inner.h {
        for mx in 0..inner.w {
            if a.rock_at(mx * scale + scale / 2, my * scale + scale / 2) {
                fb.pset(inner.x + mx, inner.y + my, ROCK + 1);
            }
        }
    }
    inner
}

/// The frame on the minimap showing what the field `f` is looking at.
pub fn minimap_view(fb: &mut Framebuffer, inner: Rect, f: Rect, (cx, cy): (i32, i32), scale: i32) {
    fb.rect_outline(inner.x + cx / scale, inner.y + cy / scale, f.w / scale, f.h / scale, WHITE);
}
