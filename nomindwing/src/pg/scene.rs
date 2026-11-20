//! Drawing the world. The arena and the match show the same things (space, rocks, loose pixels,
//! ships, shots), so they draw them with the same code: something new on the map is drawn once,
//! here, and looks the same on both screens. Each screen adds only what's its own (the arena's
//! stations, the match's network links and seed courses).
//!
//! Everything takes the field it draws into and the camera: the world point at the field's top
//! left. A world point (x, y) lands on screen at (x - cam.0, y - cam.1).

use super::design::{Part, SCOOP_CARRY, SHIELD_R, TRACTOR_TICKS};
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

/// Loose pixels, and the exhaust engines have left.
pub fn drift(fb: &mut Framebuffer, f: Rect, a: &Arena, (cx, cy): (i32, i32)) {
    for d in &a.debris {
        pc(fb, f, d.x - cx, d.y - cy, LOOSE);
    }
    // (A ship in a slipstream leaves green exhaust, and a green pixel in its wake every few
    // ticks while it's under way: that's how it shows.)
    for p in &a.exhaust {
        pc(fb, f, p.x - cx, p.y - cy, if p.fast { SLIP } else { team_col(p.team, 0) });
    }
}

/// A tractor's reach is shown by one pink dot that goes once round it, then rests: the frames
/// (15 a second) a lap takes, and the frames before the next.
pub const RING_LAP: u32 = 60;
pub const RING_REST: u32 = 45;

/// A tractor's reach: one pink dot going round its ship's middle at that distance, a lap
/// every so often. Drawn for the tractor ship under the pointer and the ones selected.
/// (A dotted circle for an evening, 2026-11-16. Henry: "can we have the circle showing the
/// tractor's range just be one pink dot that goes around in a circle occasionally. I think
/// it's too loud right now.")
pub fn tractor_ring(fb: &mut Framebuffer, f: Rect, (x, y): (i32, i32), reach: i32, (cx, cy): (i32, i32), frame: u32) {
    let t = frame % (RING_LAP + RING_REST);
    if t >= RING_LAP || reach <= 0 {
        return;
    }
    let round = circle_round(reach);
    let (dx, dy) = round[t as usize * round.len() / RING_LAP as usize];
    pc(fb, f, x - cx + dx, y - cy + dy, TRACTOR);
}

/// The tractors' pull: a dotted line from each ship that's being emptied to the tractor ship
/// emptying it, and a bright pixel running along it to the tractor, once for each pixel that
/// comes across. The line stays while pixels keep coming.
pub fn tractors(fb: &mut Framebuffer, f: Rect, a: &Arena, (cx, cy): (i32, i32)) {
    for m in a.ships.iter().filter(|m| m.alive) {
        let Some((j, at)) = m.pulled else { continue };
        let px = a.tractor_px(j);
        if px == 0 {
            continue;
        }
        // How long a pixel takes to come across: the time between two pulls, and not so short
        // that the line can't be seen.
        let (since, every) = (a.tick.saturating_sub(at) as i32, (TRACTOR_TICKS / px).max(6));
        if since > every {
            continue;
        }
        let t = &a.ships[j];
        // It ends on one of the tractor ship's tractor pixels: the first still standing.
        let to = (0..t.hp.len()).find(|&c| t.hp[c] > 0 && t.shape.cells[c] == Some(Part::Tractor)).map_or((t.x, t.y), |c| t.cell_xy(c));
        let mut l = crate::geom::Line::new(m.x - cx, m.y - cy, to.0 - cx, to.1 - cy);
        let mut pts = vec![(l.x, l.y)];
        while (l.x, l.y) != (to.0 - cx, to.1 - cy) && pts.len() < 400 {
            l.step();
            pts.push((l.x, l.y));
        }
        for (k, &(x, y)) in pts.iter().enumerate() {
            if k % 3 == 0 {
                pc(fb, f, x, y, TRACTOR);
            }
        }
        let k = (pts.len() - 1) * since.min(every) as usize / every as usize;
        pc(fb, f, pts[k].0, pts[k].1, TRACTOR_HI);
    }
}

/// A miner's load going at a dock pad: a pink line that sweeps the miner from nose to tail
/// while it sits on its place. (Henry, 2026-11-17: "There should be a pink animation when a
/// miner offloads at a pad.")
pub fn docking(fb: &mut Framebuffer, f: Rect, a: &Arena, (cx, cy): (i32, i32)) {
    for m in a.ships.iter().filter(|m| m.alive) {
        let Some(b) = m.berth.filter(|b| b.phase == 1) else { continue };
        let (l, t, r, bt) = m.bbox();
        // How far the sweep has got: from the end it faces to the other.
        use crate::pg::design::DOCK_SHOW;
        let done = (DOCK_SHOW - b.t).clamp(0, DOCK_SHOW);
        let (along, across) = if m.facing & 1 == 0 { (bt - t + 1, (l, r)) } else { (r - l + 1, (t, bt)) };
        let k = (along - 1) * done / DOCK_SHOW.max(1);
        for q in across.0 - 1..=across.1 + 1 {
            let (x, y) = match m.facing {
                0 => (q, t + k),
                2 => (q, bt - k),
                1 => (r - k, q),
                _ => (l + k, q),
            };
            pc(fb, f, x - cx, y - cy, if q < across.0 || q > across.1 { TRACTOR_HI } else { TRACTOR });
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
        // (And in its vacuum pixels, which hold as much each.)
        for c in (0..s.hp.len()).filter(|&c| s.hp[c] > 0 && matches!(s.shape.cells[c], Some(Part::Scoop | Part::Vacuum))).take(full) {
            let (px, py) = s.cell_xy(c);
            pc(fb, f, px - cx, py - cy, LOOSE);
        }
    }
    for t in s.turrets.iter().filter(|t| s.hp[t.cell] > 0) {
        let (gx, gy) = s.cell_xy(t.cell);
        let (ux, uy) = dir64(t.aim);
        let (ex, ey) = (gx + ux * BARREL / 1024, gy + uy * BARREL / 1024);
        let col = if s.shape.cells[t.cell] == Some(Part::Tracker) { TRACKER_HI } else { GUN };
        clipped_line(fb, f, gx - cx, gy - cy, ex - cx, ey - cy, col, true);
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

/// The edge of a field: a box on the map grown by `reach`, with round corners, as a dotted
/// line with a dot every `step` pixels.
fn reach_line(fb: &mut Framebuffer, f: Rect, (l, t, r, b): (i32, i32, i32, i32), reach: i32, step: i32, col: u8, (cx, cy): (i32, i32)) {
    let (l, t, r, b) = (l - cx, t - cy, r - cx, b - cy);
    for x in (l..=r).filter(|x| x.rem_euclid(step) == 0) {
        pc(fb, f, x, t - reach, col);
        pc(fb, f, x, b + reach, col);
    }
    for y in (t..=b).filter(|y| y.rem_euclid(step) == 0) {
        pc(fb, f, l - reach, y, col);
        pc(fb, f, r + reach, y, col);
    }
    // Each corner's quarter circle, walked along whichever way it runs flatter so the dots
    // stay evenly apart.
    for k in (step..reach * 181 / 256).step_by(step as usize) {
        let far = isqrt(reach * reach - k * k);
        for (dx, dy) in [(k, far), (far, k)] {
            pc(fb, f, r + dx, b + dy, col);
            pc(fb, f, r + dx, t - dy, col);
            pc(fb, f, l - dx, b + dy, col);
            pc(fb, f, l - dx, t - dy, col);
        }
    }
}

/// Stasis fields. A lattice whose ship is moving shows nothing: it does nothing. One that's
/// set (its ship is still) shows where it would hold as a sparse dotted line, the lattice's
/// box grown by its reach. One that's up (an enemy is in it) shows that line twice as close,
/// and every other pixel of the lattice turns white. Any ship with part of itself inside the
/// line is held, friend or enemy.
fn stasis(fb: &mut Framebuffer, f: Rect, a: &Arena, (cx, cy): (i32, i32)) {
    for fld in a.stasis.iter().filter(|fld| fld.set && a.ships.get(fld.ship).is_some_and(|s| s.alive)) {
        reach_line(fb, f, fld.bbox, fld.reach, if fld.up { 4 } else { 8 }, STASIS, (cx, cy));
        if fld.up {
            let s = &a.ships[fld.ship];
            let (l, t, r, b) = fld.bbox;
            for c in (0..s.hp.len()).filter(|&c| s.hp[c] > 0 && s.shape.cells[c] == Some(Part::Stasis)) {
                let (px, py) = s.cell_xy(c);
                if (l..=r).contains(&px) && (t..=b).contains(&py) && (px + py) & 1 == 0 {
                    pc(fb, f, px - cx, py - cy, WHITE);
                }
            }
        }
    }
}

/// Once round a box grown by `reach`, with round corners: clockwise from above its top left.
fn box_round((l, t, r, b): (i32, i32, i32, i32), reach: i32) -> Vec<(i32, i32)> {
    let c = circle_round(reach);
    let at = |p: (i32, i32)| c.iter().position(|&q| q == p).unwrap_or(0);
    let (e, s, w) = (at((reach, 0)), at((0, reach)), at((-reach, 0)));
    let mut round: Vec<(i32, i32)> = Vec::new();
    round.extend((l..=r).map(|x| (x, t - reach)));
    round.extend(c[..=e].iter().map(|&(dx, dy)| (r + dx, t + dy)));
    round.extend((t..=b).map(|y| (r + reach, y)));
    round.extend(c[e..=s].iter().map(|&(dx, dy)| (r + dx, b + dy)));
    round.extend((l..=r).rev().map(|x| (x, b + reach)));
    round.extend(c[s..=w].iter().map(|&(dx, dy)| (l + dx, b + dy)));
    round.extend((t..=b).rev().map(|y| (l - reach, y)));
    round.extend(c[w..].iter().map(|&(dx, dy)| (l + dx, t + dy)));
    round.dedup();
    round
}

/// Slipstreams. One's reach is shown as a tractor's is: one dot, green, that goes once round
/// the lump's box at that distance and then rests. It's always on, so there's nothing else to
/// show of the field; what it speeds leaves a green wake (`drift`).
fn slips(fb: &mut Framebuffer, f: Rect, a: &Arena, (cx, cy): (i32, i32)) {
    for (k, fld) in a.slip_fields().iter().enumerate() {
        let t = (a.tick + 23 * k as u32) % (RING_LAP + RING_REST);
        if t >= RING_LAP || fld.reach <= 0 {
            continue;
        }
        let round = box_round(fld.bbox, fld.reach);
        let (x, y) = round[t as usize * round.len() / RING_LAP as usize];
        pc(fb, f, x - cx, y - cy, SLIP);
    }
}

/// Vacuums. One's reach is shown as a tractor's is: one dot that goes once round its ship's
/// middle at that distance and then rests, in the colour of the loose pixels it takes. What
/// it pulls shows by itself: the pixels stream in.
fn vacs(fb: &mut Framebuffer, f: Rect, a: &Arena, (cx, cy): (i32, i32)) {
    for (k, v) in a.vac_fields().iter().enumerate() {
        let t = (a.tick + 37 * k as u32 + 11) % (RING_LAP + RING_REST);
        if t >= RING_LAP || v.reach <= 0 {
            continue;
        }
        let round = circle_round(v.reach);
        let (dx, dy) = round[t as usize * round.len() / RING_LAP as usize];
        pc(fb, f, v.mid.0 - cx + dx, v.mid.1 - cy + dy, LOOSE);
    }
}

/// A projector's skin. Its reach is a dotted line: any friend with part of itself inside that
/// line is covered. Every ship one covers wears a blue outline two pixels outside its box, with
/// its corners cut. A full pool is a solid outline, and it thins by a quarter for each quarter of
/// the pool that's spent, as a shield's circle does. With the pool empty there's no outline:
/// nothing's covered.
fn skins(fb: &mut Framebuffer, f: Rect, a: &Arena, (cx, cy): (i32, i32)) {
    // The reach: the lump's box grown by its reach, with round corners. A dot every fourth pixel.
    for (_, _, bbox, reach) in a.proj_fields() {
        reach_line(fb, f, bbox, reach, 4, PROJECTOR_DIM, (cx, cy));
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
    stasis(fb, f, a, (cx, cy));
    slips(fb, f, a, (cx, cy));
    vacs(fb, f, a, (cx, cy));
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
        // An energy ball is 2x2, with one bright pixel.
        if sh.kind == Kind::Ball {
            for (dx, dy, col) in [(0, 0, BALL_HI), (1, 0, BALL), (0, 1, BALL), (1, 1, BALL)] {
                pc(fb, f, q.0 + dx, q.1 + dy, col);
            }
            continue;
        }
        let (c0, c1) = match sh.kind {
            Kind::Lance => (LANCE, LANCE_HI),
            Kind::Shell => (GOLD, WHITE),
            Kind::Missile => (MISSILE, MISSILE_HI),
            Kind::Repeat => (REPEATER, WHITE),
            Kind::Track => (TRACKER, TRACKER_HI),
            Kind::Flak => (team_col(sh.team, 1), team_col(sh.team, 3)),
            Kind::Ball => (BALL, BALL_HI),
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
