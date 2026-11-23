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

/// The order ships are drawn in: the biggest first, so a smaller ship always shows over a
/// bigger one it's flying over or sitting on, and ships of a size in the order they were made.
/// A ship in a pad's hands counts as smaller than the pad's ship, whatever is left of either:
/// a miner fits a pad by the pixels it still has, so a big one shot small can sit on a pad
/// whose ship was drawn smaller than it. A ship's size is its drawing's, so two ships don't
/// change places as pixels are shot off them.
///
/// (They were drawn in the order they were made, so a PORT built after its miners was drawn
/// over them as they sat on its pad. Henry, 2026-11-21: "Gleaner appears to be rendered
/// beneath Port".)
pub fn layers(a: &Arena) -> Vec<usize> {
    let size = |i: usize| {
        let s = &a.ships[i];
        match s.berth.and_then(|b| a.ships.get(b.ship)) {
            Some(pad) => s.px_full().min(pad.px_full() - 1),
            None => s.px_full(),
        }
    };
    let mut order: Vec<usize> = (0..a.ships.len()).filter(|&i| a.ships[i].alive).collect();
    order.sort_by_key(|&i| (std::cmp::Reverse(size(i)), i));
    order
}

/// Every living ship on the map, in that order. A screen draws its ships with this and never
/// one by one, or it would have an order of its own.
pub fn ships(fb: &mut Framebuffer, f: Rect, a: &Arena, cam: (i32, i32)) {
    for i in layers(a) {
        ship(fb, f, &a.ships[i], cam);
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
    // A pad pixel that's resting after a miner unloaded on it shows: every other one is
    // dark, until it's ready again. (Until 2026-11-22 a resting pad looked like a ready one.
    // Henry, of miners by a BARGE: "people would wait for a while for them even though they
    // were empty".)
    for c in (0..s.hp.len()).filter(|&c| s.pad_resting(c)) {
        let (px, py) = s.cell_xy(c);
        if (px + py) & 1 == 0 {
            pc(fb, f, px - cx, py - cy, OUTLINE);
        }
    }
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
    // What a phase ring took shows in the space the ring shuts in (Henry, 2026-11-22: "since
    // you have that nice required empty space anyway, you could use it to graphically
    // indicate the pixels the phase ship is holding"): as much of it in the colour of loose
    // pixels as the share of that rock it still has aboard, filled from the stern. So it's
    // full the tick the rock goes, drains as a tractor empties it, and is empty the tick a
    // pad takes the lot.
    if s.carry > 0 && s.st.ring_hole > 0 {
        let mut shut = crate::pg::design::ring_cells(s.shape.w, &s.alive_cells());
        shut.sort();
        let of = s.took.map_or(s.carry, |t| t.1).max(s.carry);
        let lit = ((shut.len() as i32 * s.carry + of - 1) / of) as usize;
        for &c in shut.iter().rev().take(lit) {
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

/// How far apart the lines of a stasis field's web are on the map, and the dots along them.
pub const STASIS_WEB: i32 = 16;
pub const STASIS_WEB_DOT: i32 = 4;
/// How far apart the rings that close in on a working lattice are, and how far each moves in
/// a tick: half as fast as the slowest ship's shot, so they read as slow.
/// (32 px apart at first, so that a SNARE's field showed three or four at once. Henry,
/// 2026-11-22: "I think there should be fewer rings in the stasis field animation".)
pub const STASIS_RING: i32 = 64;
pub const STASIS_RING_STEP: i32 = 2;
/// A ring shows one pixel in this many along it.
pub const STASIS_RING_DOT: i32 = 2;

/// How far a point on the map is from a box, or None if that's further than `reach`.
fn box_dist((l, t, r, b): (i32, i32, i32, i32), reach: i32, (x, y): (i32, i32)) -> Option<i32> {
    let (dx, dy) = ((l - x).max(x - r).max(0), (t - y).max(y - b).max(0));
    match (dx, dy) {
        (0, d) | (d, 0) => (d <= reach).then_some(d),
        _ => (dx * dx + dy * dy <= reach * reach).then(|| isqrt(dx * dx + dy * dy)),
    }
}

/// Whether a line round a box runs more across the map than up and down it at a point: above
/// or below the box, and on the flatter half of each corner's quarter circle.
fn box_across((l, t, r, b): (i32, i32, i32, i32), (x, y): (i32, i32)) -> bool {
    (t - y).max(y - b).max(0) >= (l - x).max(x - r).max(0)
}

/// What a stasis field that's up shows at a point `d` px from its lattice's box, on the map
/// at (x, y), at tick `tick`: a dot of a ring, a dot of its web, or nothing. The web is fixed
/// to the map, so it doesn't crawl as the camera scrolls. A ring is the line of points one
/// distance from the box, dotted: every other pixel along it, counted across the map where
/// the ring runs `across` and down it where it doesn't, so the dots are evenly apart all the
/// way round. It comes in `STASIS_RING_STEP` px a tick, and its dots keep their columns and
/// rows as it does.
///
/// (For a day the rings were whole lines. Henry, 2026-11-22: "the animation for the field is
/// slightly too loud.")
fn stasis_mark(d: i32, (x, y): (i32, i32), across: bool, tick: u32) -> Option<u8> {
    if d < 1 {
        return None;
    }
    if (d + tick as i32 * STASIS_RING_STEP).rem_euclid(STASIS_RING) == 0 && if across { x } else { y }.rem_euclid(STASIS_RING_DOT) == 0 {
        return Some(STASIS);
    }
    let on = |a: i32, b: i32| a.rem_euclid(STASIS_WEB) == 0 && b.rem_euclid(STASIS_WEB_DOT) == 0;
    (on(x, y) || on(y, x)).then_some(SHIELD_DIM)
}

/// The fields of the stasis lattices that are up, on the ground: drawn before the ships, which
/// show over them. All of a field shows, from its lattice's box out to its reach: a dotted
/// web, with dotted rings that close in on the lattice, one after another, for as long as it
/// holds anything. A field that's only set (no enemy in it) shows none of this.
///
/// (Until 2026-11-21 a field that was up was a dotted line round its reach and nothing
/// inside it. Henry, who had never seen one work: "have a pretty loud animation when it's
/// doing its thing, if it doesn't yet".)
pub fn stasis_ground(fb: &mut Framebuffer, f: Rect, a: &Arena, (cx, cy): (i32, i32)) {
    for fld in a.stasis.iter().filter(|fld| fld.up && a.ships.get(fld.ship).is_some_and(|s| s.alive)) {
        let (l, t, r, b) = fld.bbox;
        let (x0, x1) = ((l - fld.reach).max(f.x + cx), (r + fld.reach).min(f.x + f.w - 1 + cx));
        let (y0, y1) = ((t - fld.reach).max(f.y + cy), (b + fld.reach).min(f.y + f.h - 1 + cy));
        for y in y0..=y1 {
            for x in x0..=x1 {
                if let Some(col) = box_dist(fld.bbox, fld.reach, (x, y)).and_then(|d| stasis_mark(d, (x, y), box_across(fld.bbox, (x, y)), a.tick)) {
                    fb.pset(x - cx, y - cy, col);
                }
            }
        }
    }
}

/// Stasis fields, over the ships. A lattice whose ship is moving shows nothing: it does
/// nothing. One that's set (its ship is still) shows where it would hold as a sparse dotted
/// line, the lattice's box grown by its reach. One that's up (an enemy is in it) shows that
/// line four times as close, every other pixel of the lattice turns white, and every ship it
/// holds, friend or enemy, has a violet corner at each corner of its box. (What it shows
/// under the ships is `stasis_ground`.)
fn stasis(fb: &mut Framebuffer, f: Rect, a: &Arena, (cx, cy): (i32, i32)) {
    for fld in a.stasis.iter().filter(|fld| fld.set && a.ships.get(fld.ship).is_some_and(|s| s.alive)) {
        reach_line(fb, f, fld.bbox, fld.reach, if fld.up { 2 } else { 8 }, STASIS, (cx, cy));
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
    for s in a.ships.iter().filter(|s| s.alive && s.slowed && s.warp == 0) {
        held_mark(fb, f, s, (cx, cy));
    }
}

/// The mark on a ship a stasis field holds: a violet corner, three pixels, just outside each
/// corner of its box.
fn held_mark(fb: &mut Framebuffer, f: Rect, s: &Ship, (cx, cy): (i32, i32)) {
    let (l, t, r, b) = s.bbox();
    for (x, y, sx, sy) in [(l - 1, t - 1, 1, 1), (r + 1, t - 1, -1, 1), (l - 1, b + 1, 1, -1), (r + 1, b + 1, -1, -1)] {
        for (dx, dy) in [(0, 0), (sx, 0), (0, sy)] {
            pc(fb, f, x + dx - cx, y + dy - cy, STASIS);
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

/// How wide a rock's charge bar is (px), and how many ticks the dot from a ring takes to
/// reach its rock.
pub const PHASE_BAR: i32 = 24;
pub const PHASE_DOT: u32 = 15;

/// Phase rings at work (match only). A rock that holds a charge wears a small bar at its
/// middle: as much of it lit as the charge is of what the rock has left, in the colour of the
/// side whose charge it is. And from each ring at work one pale dot runs to the middle of its
/// rock, once a second. (The rock going shows by itself: sparks where it was, and it's gone.)
fn phases(fb: &mut Framebuffer, f: Rect, a: &Arena, (cx, cy): (i32, i32)) {
    let Some(e) = a.eco.as_ref() else { return };
    for (i, s) in a.ships.iter().enumerate().filter(|(_, s)| s.alive) {
        let Some(rock) = s.charging.and_then(|r| e.rocks.get(r)) else { continue };
        let (from, to) = (s.ring_mid(), (rock.cx, rock.cy));
        let mut l = crate::geom::Line::new(from.0 - cx, from.1 - cy, to.0 - cx, to.1 - cy);
        let mut pts = vec![(l.x, l.y)];
        while (l.x, l.y) != (to.0 - cx, to.1 - cy) && pts.len() < 400 {
            l.step();
            pts.push((l.x, l.y));
        }
        let t = (a.tick + 7 * i as u32) % PHASE_DOT;
        let k = (pts.len() - 1) * t as usize / (PHASE_DOT as usize - 1);
        pc(fb, f, pts[k].0, pts[k].1, PHASE);
    }
    // (The bars go on last: a dot ends its run under its rock's.)
    for (r, ph) in e.phase.iter().enumerate().filter(|(_, ph)| ph.charge > 0) {
        let (rock, left) = (&e.rocks[r], a.rock_left(&e.rocks[r]));
        if left == 0 {
            continue;
        }
        let (x0, y0) = (rock.cx - cx - PHASE_BAR / 2, rock.cy - cy);
        let lit = (ph.charge * PHASE_BAR / left).clamp(1, PHASE_BAR);
        for k in -1..=PHASE_BAR {
            for j in -1..=2 {
                let inside = (0..PHASE_BAR).contains(&k) && (0..2).contains(&j);
                pc(fb, f, x0 + k, y0 + j, if !inside { OUTLINE } else if k < lit { team_col(ph.team, 2) } else { ROCK });
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
    phases(fb, f, a, (cx, cy));
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pg::design::{starter_deck, Design, Order, STASIS_RISE};

    /// A post with no engines whose lattice is 19 wide and 4 tall, so it reaches 92 px, at
    /// (450, 400) on an empty map: its field is set. With `enemy`, an enemy fighter 40 px
    /// above the lattice and a friend's beside that, both told to hold, and an enemy 200 px
    /// above it: the field is up and holds the first two. Returns the post, and those three.
    fn post(enemy: bool) -> (Arena, usize, Vec<usize>) {
        let (bar, hull) = ("x".repeat(19), "h".repeat(19));
        let mast = format!("{}x{}", ".".repeat(9), ".".repeat(9));
        let mid = format!("{}c{}", "h".repeat(9), "h".repeat(9));
        let web = Design::from_rows("WEB", &[&bar, &mast, &mast, &mast, &hull, &mid, &hull]);
        let mut a = Arena::blank(900, 700, 1);
        a.plain = [true; 2];
        let post = a.spawn(0, 0, &web, 450, 400, 0, false);
        let mut others = Vec::new();
        if enemy {
            // (Fighters with their flak turned to hull: nothing here shoots.)
            let mut plain = starter_deck()[0].clone();
            for c in plain.cells.iter_mut().filter(|c| **c == Some(Part::Flak)) {
                *c = Some(Part::Hull);
            }
            for (team, x, y) in [(1, 462, 356), (0, 438, 356), (1, 450, 196)] {
                let i = a.spawn(team, 0, &plain, x, y, 0, false);
                a.ships[i].order = Order::Hold;
                others.push(i);
            }
        }
        for _ in 0..STASIS_RISE + 2 {
            a.step();
        }
        assert_eq!(a.stasis.len(), 1);
        assert_eq!((a.stasis[0].reach, a.stasis[0].set, a.stasis[0].up), (92, true, enemy));
        (a, post, others)
    }

    /// The pixels a drawing call changed on an empty field, with the camera at `cam`: where
    /// each is on the map, and its colour.
    fn drawn(cam: (i32, i32), draw: impl Fn(&mut Framebuffer, Rect, (i32, i32))) -> Vec<((i32, i32), u8)> {
        let mut fb = Framebuffer::new();
        fb.clear(SPACE);
        let f = Rect::new(0, 0, 304, 300);
        draw(&mut fb, f, cam);
        let mut out = Vec::new();
        for y in 0..crate::framebuffer::FB_H as i32 {
            for x in 0..crate::framebuffer::FB_W as i32 {
                if fb.get(x, y) != SPACE {
                    assert!(f.contains(x, y), "a pixel was drawn at ({x}, {y}), outside the field");
                    out.push(((x + cam.0, y + cam.1), fb.get(x, y)));
                }
            }
        }
        out
    }

    #[test]
    fn a_working_lattices_field_shows_a_web_and_rings_that_close_in_on_it() {
        // Henry, 2026-11-21: "have a pretty loud animation when it's doing its thing".
        // A field that's only set shows nothing on the ground.
        let cam = (450 - 152, 400 - 150);
        let (a, _, _) = post(false);
        assert!(drawn(cam, |fb, f, cam| stasis_ground(fb, f, &a, cam)).is_empty());
        // Up, it shows all the way out to its reach and nowhere else, in two colours: the
        // web's, and the rings'.
        let (mut a, _, _) = post(true);
        let bbox = a.stasis[0].bbox;
        let look = |a: &Arena, cam: (i32, i32)| drawn(cam, |fb, f, cam| stasis_ground(fb, f, a, cam));
        let now = look(&a, cam);
        println!("{} pixels of it show", now.len());
        assert!(now.len() >= 700, "only {} pixels of it show", now.len());
        for &(p, col) in &now {
            let d = box_dist(bbox, 92, p);
            assert!(d.is_some_and(|d| d >= 1) && (col == STASIS || col == SHIELD_DIM), "a pixel of colour {col} at {p:?}, {d:?} px from the lattice");
        }
        // Its rings, read up the column over the lattice's middle: they're `STASIS_RING` px
        // apart, which is 64, so there are one or two of them inside 92 px (Henry, 2026-11-22,
        // of rings 32 px apart: "I think there should be fewer rings"), and a tick later
        // each is `STASIS_RING_STEP` px nearer the lattice.
        assert_eq!(STASIS_RING, 64);
        let rings = |shown: &[((i32, i32), u8)]| -> Vec<i32> {
            let mut d: Vec<i32> = shown.iter().filter(|&&((x, y), col)| x == 450 && y < bbox.1 && col == STASIS).map(|&((_, y), _)| bbox.1 - y).collect();
            d.sort();
            d
        };
        let r0 = rings(&now);
        assert!((1..=2).contains(&r0.len()) && r0.windows(2).all(|w| w[1] - w[0] == STASIS_RING), "{r0:?}");
        // A ring is dotted, all the way round the lattice. (Henry, 2026-11-22, of whole
        // lines: "slightly too loud".) Of the points that far from the lattice it shows the
        // ones in every other column where it runs across the map and in every other row
        // where it runs down it: half of them, near enough. Along the lattice's top that's a
        // dot and a gap, and down its side the same.
        let lit: Vec<(i32, i32)> = now.iter().filter(|&&(p, col)| col == STASIS && box_dist(bbox, 92, p) == Some(r0[0])).map(|x| x.0).collect();
        let all_round: Vec<(i32, i32)> = (bbox.0 - 100..=bbox.2 + 100).flat_map(|x| (bbox.1 - 100..=bbox.3 + 100).map(move |y| (x, y))).filter(|&p| box_dist(bbox, 92, p) == Some(r0[0])).collect();
        let dot = |p: (i32, i32)| if box_across(bbox, p) { p.0 } else { p.1 }.rem_euclid(STASIS_RING_DOT) == 0;
        assert!(all_round.iter().all(|&p| lit.contains(&p) == dot(p)));
        println!("the ring {} px out shows {} of its {} pixels", r0[0], lit.len(), all_round.len());
        assert!(lit.len() >= 50 && lit.len() * 5 >= all_round.len() * 2 && lit.len() * 5 <= all_round.len() * 3, "the ring {} px out shows {} of its {} pixels", r0[0], lit.len(), all_round.len());
        let (top, side) = (bbox.1 - r0[0], bbox.2 + r0[0]);
        assert!((bbox.0..=bbox.2).all(|x| lit.contains(&(x, top)) == (x % 2 == 0)) && (bbox.1..=bbox.3).all(|y| lit.contains(&(side, y)) == (y % 2 == 0)));
        let web0: Vec<(i32, i32)> = now.iter().filter(|x| x.1 == SHIELD_DIM).map(|x| x.0).collect();
        a.step();
        let next = look(&a, cam);
        let r1 = rings(&next);
        let want: Vec<i32> = r0.iter().map(|d| d - STASIS_RING_STEP).filter(|&d| d >= 1).collect();
        assert!(want.iter().all(|d| r1.contains(d)) && r1.len() <= want.len() + 1, "a tick on the rings are at {r1:?}; they were at {r0:?}");
        // The web is fixed to the map: a tick on, every dot of it that no ring covers is
        // where it was, and with the camera moved the same dots show.
        let web1: Vec<(i32, i32)> = next.iter().filter(|x| x.1 == SHIELD_DIM).map(|x| x.0).collect();
        let on_ring = |p: (i32, i32), tick: u32| box_dist(bbox, 92, p).is_some_and(|d| stasis_mark(d, p, box_across(bbox, p), tick) == Some(STASIS));
        assert!(web0.len() >= 600 && web0.iter().filter(|&&p| !on_ring(p, a.tick)).all(|p| web1.contains(p)));
        let moved = look(&a, (cam.0 + 5, cam.1 - 3));
        let both = |shown: &[((i32, i32), u8)]| -> Vec<((i32, i32), u8)> { shown.iter().filter(|&&((x, y), _)| x >= cam.0 + 5 && x < cam.0 + 304 && y >= cam.1 && y < cam.1 + 297).copied().collect() };
        assert_eq!(both(&next), both(&moved));
        // What a point shows: nothing on the lattice's own box; a ring's dot where its
        // distance and the tick say and its column is even (its row, where the ring runs down
        // the map); else the web, on its lines.
        assert_eq!((stasis_mark(0, (16, 16), true, 0), stasis_mark(64, (4, 5), true, 0), stasis_mark(64, (5, 4), true, 0), stasis_mark(64, (5, 4), false, 0)), (None, Some(STASIS), None, Some(STASIS)));
        assert_eq!((stasis_mark(62, (4, 5), true, 1), stasis_mark(62, (4, 5), true, 0), stasis_mark(32, (4, 5), true, 0)), (Some(STASIS), None, None));
        assert_eq!((stasis_mark(5, (16, 8), true, 0), stasis_mark(5, (8, 32), false, 7), stasis_mark(5, (16, 9), true, 0), stasis_mark(5, (9, 9), true, 0)), (Some(SHIELD_DIM), Some(SHIELD_DIM), None, None));
        // Nothing is drawn outside the field's rectangle on the screen (`drawn` checks), even
        // with the field half off it.
        assert!(!look(&a, (450 - 30, 400 - 20)).is_empty());
    }

    #[test]
    fn a_held_ship_wears_a_violet_corner_at_each_corner_of_its_box_and_a_free_one_none() {
        let cam = (450 - 152, 400 - 150);
        let (a, post, others) = post(true);
        assert_eq!(others.iter().map(|&i| a.ships[i].slowed).collect::<Vec<_>>(), [true, true, false]);
        let over = drawn(cam, |fb, f, cam| fire(fb, f, &a, cam));
        let violet = |p: (i32, i32)| over.iter().any(|&(q, col)| q == p && col == STASIS);
        let corners = |i: usize| {
            let (l, t, r, b) = a.ships[i].bbox();
            [(l - 1, t - 1), (r + 1, t - 1), (l - 1, b + 1), (r + 1, b + 1)]
        };
        // The enemy it holds and the friend it holds, alike; not the enemy out of reach, and
        // not the lattice's own ship.
        for (k, &i) in others.iter().enumerate() {
            let n = corners(i).iter().filter(|&&p| violet(p)).count();
            assert_eq!(n, if a.ships[i].slowed { 4 } else { 0 }, "ship {k} is {} and has {n} corners", if a.ships[i].slowed { "held" } else { "free" });
        }
        assert_eq!(corners(post).iter().filter(|&&p| violet(p)).count(), 0);
        // Each corner is three pixels: the corner, and one along each side from it.
        let (l, t, _, _) = a.ships[others[0]].bbox();
        assert!(violet((l, t - 1)) && violet((l - 1, t)) && !violet((l, t)) && !violet((l + 1, t - 1)));
        // The line round its reach is four times as close with the field up as with it only
        // set.
        let (idle, _, _) = self::post(false);
        let line = |shown: &[((i32, i32), u8)]| shown.iter().filter(|&&(p, col)| col == STASIS && box_dist(a.stasis[0].bbox, 92, p).is_some_and(|d| d >= 91)).count();
        let (up, set) = (line(&over), line(&drawn(cam, |fb, f, cam| fire(fb, f, &idle, cam))));
        assert!(set >= 40 && up >= set * 3, "{up} pixels of line with the field up, {set} with it set");
    }

    #[test]
    fn a_pad_pixel_thats_resting_shows_dark_until_its_ready_again() {
        // Henry, 2026-11-22, of miners by a BARGE: "people would wait for a while for them
        // even though they were empty". A pad pixel rests for three seconds after a miner has
        // unloaded on it, and a resting pad looked like a ready one.
        //
        // The SCAVENGERS' PORT and a full DIGGER beside it. Before the DIGGER lands every pad
        // pixel of the PORT is drawn one colour. Once it has landed, the pad pixels that were
        // under it are resting, one for each pixel of the DIGGER: every other one of those is
        // drawn dark, and the rest of the pad as it was. Three seconds on they're all ready,
        // and drawn as they were.
        use crate::pg::design::DOCK_TURN;
        let team = crate::pg::teams::scavengers();
        let mut a = Arena::for_match(1, [team.clone(), starter_deck()], [false, false]);
        let port = a.spawn(0, 7, &team[7], 700, 500, 0, false);
        let digger = a.spawn(0, 5, &team[5], 760, 520, 0, false);
        a.ships[digger].carry = a.ships[digger].st.hold();
        let cam = (700 - 152, 500 - 150);
        let pads: Vec<usize> = (0..a.ships[port].hp.len()).filter(|&c| a.ships[port].shape.cells[c] == Some(Part::DockPad)).collect();
        // The colour each pad pixel of the PORT is drawn, cell by cell, with the PORT drawn
        // alone (so that no miner on the pad covers any of it).
        let look = |a: &Arena| -> Vec<u8> {
            let shown = drawn(cam, |fb, f, cam| ship(fb, f, &a.ships[port], cam));
            pads.iter().map(|&c| shown.iter().find(|x| x.0 == a.ships[port].cell_xy(c)).map_or(SPACE, |x| x.1)).collect()
        };
        let ready = look(&a);
        assert!(pads.len() == 54 && ready.iter().all(|&col| col == ready[0] && col != OUTLINE && col != SPACE), "{ready:?}");
        let mut t = 0;
        while !pads.iter().any(|&c| a.ships[port].pad_resting(c)) {
            a.step();
            t += 1;
            assert!(t < 15 * 60, "a minute on the DIGGER hasn't landed");
        }
        let resting: Vec<bool> = pads.iter().map(|&c| a.ships[port].pad_resting(c)).collect();
        assert_eq!(resting.iter().filter(|&&r| r).count() as i32, a.ships[digger].px_now());
        let now = look(&a);
        for (k, &c) in pads.iter().enumerate() {
            let (px, py) = a.ships[port].cell_xy(c);
            let dark = resting[k] && (px + py) & 1 == 0;
            assert_eq!(now[k], if dark { OUTLINE } else { ready[0] }, "the pad pixel at {px},{py}, which is {}", if resting[k] { "resting" } else { "ready" });
        }
        assert!(now.iter().filter(|&&col| col == OUTLINE).count() >= 8);
        for _ in 0..DOCK_TURN {
            a.step();
        }
        assert!(!pads.iter().any(|&c| a.ships[port].pad_resting(c)));
        assert_eq!(look(&a), ready);
    }

    #[test]
    fn a_charged_rock_wears_a_bar_and_one_dot_runs_to_it_from_a_ring_at_work() {
        // A HALO beside blue's home rock, in a match nobody plays. Nothing else in it draws
        // anything over the map but the phase ring's two marks.
        use crate::pg::design::{part_examples, PHASE_RISE};
        let halo = part_examples().into_iter().find(|d| d.name == "HALO").unwrap();
        let mut deck = starter_deck();
        deck.push(halo.clone());
        let mut a = Arena::for_match(1, [deck.clone(), deck.clone()], [false, false]);
        let (r, (cx, cy)) = {
            let rocks = &a.eco.as_ref().unwrap().rocks;
            let r = (0..rocks.len()).min_by_key(|&r| crate::geom::dist2(rocks[r].cx, rocks[r].cy, KING_STARTS[0].0, KING_STARTS[0].1)).unwrap();
            (r, (rocks[r].cx, rocks[r].cy))
        };
        let ship = a.spawn(0, deck.len() - 1, &halo, cx - 30, cy + 4, 0, false);
        let cam = (cx - 152, cy - 150);
        let show = |a: &Arena| drawn(cam, |fb, f, cam| fire(fb, f, a, cam));
        // With no charge on any rock and no ring at work, nothing shows.
        assert!(show(&a).is_empty());
        for _ in 0..PHASE_RISE - 1 + 600 {
            a.step();
        }
        let (charge, left) = (a.eco.as_ref().unwrap().phase[r].charge, a.rock_left(&a.eco.as_ref().unwrap().rocks[r]));
        assert_eq!((charge, a.ships[ship].charging), (90, Some(r)));
        // The bar: 24 by 2 at the rock's middle, framed dark. As much of it is lit as the
        // charge is of what the rock has left, in the light shade of the side whose it is.
        let bar = |a: &Arena, lit: i32, team: usize| {
            let shown = show(a);
            let at = |p: (i32, i32)| shown.iter().find(|x| x.0 == p).map(|x| x.1);
            for k in -1..=PHASE_BAR {
                for j in -1..=2 {
                    let inside = (0..PHASE_BAR).contains(&k) && (0..2).contains(&j);
                    let want = if !inside { OUTLINE } else if k < lit { team_col(team, 2) } else { ROCK };
                    let got = at((cx - PHASE_BAR / 2 + k, cy + j));
                    assert_eq!(got, Some(want), "the bar at {k},{j}");
                }
            }
            shown
        };
        assert_eq!((charge * PHASE_BAR / left, PHASE_BAR), (5, 24));
        let shown = bar(&a, 5, 0);
        // And nothing else but one pale dot, at most: it's under the bar at the end of its
        // run.
        let frame = (PHASE_BAR + 2) * 4;
        assert!(shown.len() as i32 == frame + shown.iter().filter(|x| x.1 == PHASE).count() as i32 && shown.len() as i32 <= frame + 1, "{} pixels drawn", shown.len());
        // The dot runs from the middle of the ring towards the middle of the rock, once a
        // second, and goes under the bar there.
        let mut run = Vec::new();
        for _ in 0..PHASE_DOT * 2 {
            a.step();
            let dots: Vec<(i32, i32)> = show(&a).iter().filter(|x| x.1 == PHASE).map(|x| x.0).collect();
            assert!(dots.len() <= 1, "{dots:?}");
            run.push(dots.first().copied());
        }
        let (from, to) = (a.ships[ship].ring_mid(), (cx, cy));
        let start = run.iter().position(|&p| p == Some(from)).unwrap();
        let far = |p: (i32, i32)| crate::geom::dist2(p.0, p.1, to.0, to.1);
        let lap: Vec<(i32, i32)> = run[start..start + PHASE_DOT as usize].iter().map_while(|p| *p).collect();
        assert!(far(from) > 20 * 20 && lap.len() >= 8 && run[start + lap.len()..start + PHASE_DOT as usize].iter().all(|p| p.is_none()), "it runs from {from:?} to {to:?}: {run:?}");
        assert!(lap.windows(2).all(|w| far(w[1]) < far(w[0])) && far(*lap.last().unwrap()) <= 17 * 17, "{lap:?}");
        // A charge that's the other side's is lit in that side's colour, and one that's all
        // the rock has left lights the whole bar.
        let e = a.eco.as_mut().unwrap();
        (e.phase[r].team, e.phase[r].charge) = (1, left);
        bar(&a, PHASE_BAR, 1);
        // A ring that isn't at work sends no dot.
        a.ships[ship].charging = None;
        assert!(show(&a).iter().all(|x| x.1 != PHASE));
    }

    #[test]
    fn what_a_ring_took_shows_in_the_space_it_shuts_in() {
        // Henry, 2026-11-22: "since you have that nice required empty space anyway, you
        // could use it to graphically indicate the pixels the phase ship is holding."
        //
        // A HALO, which shuts in 225. With a rock of 400 just taken, all of that space is
        // the colour of loose pixels. With half of it left aboard, half the space is (113,
        // rounded up): the half astern. With one pixel left, one. With nothing, none.
        use crate::pg::design::{part_examples, ring_cells};
        let halo = part_examples().into_iter().find(|d| d.name == "HALO").unwrap();
        let mut a = Arena::blank(480, 360, 1);
        let i = a.spawn(0, 0, &halo, 240, 180, 1, false);
        let cam = (240 - 152, 180 - 150);
        let mut shut = ring_cells(a.ships[i].shape.w, &a.ships[i].alive_cells());
        shut.sort();
        assert_eq!(shut.len(), 225);
        let lit = |a: &Arena| -> Vec<(i32, i32)> { drawn(cam, |fb, f, cam| ship(fb, f, &a.ships[i], cam)).iter().filter(|x| x.1 == LOOSE).map(|x| x.0).collect() };
        assert!(lit(&a).is_empty());
        for (aboard, want) in [(400, 225), (200, 113), (1, 1), (399, 225), (0, 0)] {
            (a.ships[i].took, a.ships[i].carry) = (Some((0, 400)), aboard);
            let mut got = lit(&a);
            let mut should: Vec<(i32, i32)> = shut.iter().rev().take(want).map(|&c| a.ships[i].cell_xy(c)).collect();
            got.sort();
            should.sort();
            assert_eq!(got, should, "with {aboard} of 400 aboard");
        }
        // With its ring broken there's no such space, and nothing shows.
        (a.ships[i].carry, a.ships[i].st.ring_hole) = (400, 0);
        assert!(lit(&a).is_empty());
    }

    /// A field that's up is drawn on the ground by both screens that show the map, and before
    /// their ships, which show over it.
    #[test]
    fn both_screens_draw_a_working_field_under_their_ships() {
        for (name, src) in [("play.rs", include_str!("play.rs")), ("mod.rs", include_str!("mod.rs"))] {
            let (ground, ships) = (src.find("scene::stasis_ground(fb, f, a, self.cam)"), src.find("scene::ships(fb, f, a, self.cam)"));
            assert!(ground.is_some() && ground < ships, "{name}: {ground:?}, {ships:?}");
        }
    }

    /// Ships are drawn by `ships`, in `layers`' order, and by nothing else: a screen that drew
    /// them one by one would draw them in the order they were made, which is how a PORT came
    /// to be drawn over the miner on its pad.
    #[test]
    fn no_screen_draws_ships_in_an_order_of_its_own() {
        for (name, src) in [("mod.rs", include_str!("mod.rs")), ("play.rs", include_str!("play.rs")), ("yard.rs", include_str!("yard.rs")), ("library.rs", include_str!("library.rs")), ("export.rs", include_str!("../export.rs"))] {
            assert!(!src.contains(concat!("scene::", "ship(")), "{name} draws a ship itself: it's `scene::ships` that draws them, smaller over bigger");
        }
        let (field, arena) = (include_str!("play.rs"), include_str!("mod.rs"));
        assert!(field.contains("scene::ships(fb, f, a, self.cam)") && arena.contains("scene::ships(fb, f, a, self.cam)"));
    }
}
