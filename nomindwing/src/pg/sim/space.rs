//! Where pixels may be. Every rule about two sides' pixels sharing a place lives in this file, and
//! every way a ship changes position or a pixel comes into being goes through it:
//!
//! - `shift` moves a ship that's on the map (flying, or a king steered), and `try_turn` turns one;
//! - `carry` places something that isn't on the map in its own right: a ship still in hyperspace,
//!   or a half-printed ship riding beside its carrier;
//! - `enemy_pixel`, `in_enemy` and `room_to_grow` say whether a pixel may come into being
//!   (growing, printing, arriving).
//!
//! The rules are four. A bigger enemy is solid to a smaller ship that moves; ships of the same
//! size or smaller pass through each other. Anything planted (grown out of a rock and still
//! standing on it: a factory, mostly) is solid to every enemy ship whatever its size, so nothing
//! flies over an enemy factory or sits on one. Nothing comes into being on an enemy pixel of any
//! size. And a factory growing out of a rock doesn't grow into anything of its own side that's
//! standing there (another factory, say), though its own side's ships fly over it freely. A ship
//! still in hyperspace isn't on the map, so it's never in anything's way.
//!
//! A move is walked a pixel at a time, so a ship stops flush against whatever is solid to it at
//! any speed, and can't skip across something thinner than its step. A ship can find itself
//! inside a bigger enemy (the bigger ship flew onto it) and is let out; it's never inside
//! anything planted, because planted things don't move and nothing grows onto an enemy pixel.
//!
//! Each of these was once a separate check written after a bug (fighters inside the capital,
//! factories growing through each other, ships arriving inside ships). New ways for pixels to
//! appear or move belong here, not beside them.

use super::*;

/// Which enemies count as in the way.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(super) enum Solid {
    /// Moving: enemies bigger than the mover, and planted ones of any size.
    Moving,
    /// Only what's planted: a mover never ends a step inside one of these, whatever else it's
    /// inside.
    Planted,
    /// Coming into being: any enemy.
    Any,
}

impl Arena {
    /// How many of ship `i`'s living pixels would sit on living enemy pixels that count under
    /// `rule`, were it centred on (x, y) facing `facing`; and the first such enemy.
    pub(super) fn clash(&self, i: usize, x: i32, y: i32, facing: u8, rule: Solid) -> (i32, Option<usize>) {
        let s = &self.ships[i];
        let (w, h) = s.shape.dims(facing);
        let (l, t) = (x - w / 2, y - h / 2);
        let size = s.px_full();
        let (mut n, mut first) = (0, None);
        for (k, e) in self.ships.iter().enumerate() {
            let counts = match rule {
                Solid::Moving => e.rooted || e.px_full() > size,
                Solid::Planted => e.rooted,
                Solid::Any => true,
            };
            if !e.alive || e.team == s.team || e.warp > 0 || !counts {
                continue;
            }
            let (el, et, er, eb) = e.bbox();
            if el > l + w - 1 || er < l || et > t + h - 1 || eb < t {
                continue;
            }
            let m = (0..s.hp.len())
                .filter(|&c| {
                    s.hp[c] > 0 && {
                        let (px, py) = s.shape.cell_xy(x, y, facing, c);
                        e.cell_at(px, py).map_or(false, |j| e.hp[j] > 0)
                    }
                })
                .count() as i32;
            if m > 0 {
                n += m;
                first = first.or(Some(k));
            }
        }
        (n, first)
    }

    /// `clash` under the moving rule: pixels that would be inside enemies solid to ship `i`
    /// (bigger ones, and anything planted).
    pub(super) fn overlap(&self, i: usize, x: i32, y: i32, facing: u8) -> (i32, Option<usize>) {
        self.clash(i, x, y, facing, Solid::Moving)
    }

    /// Whether ship `i` centred on (x, y) would have a living pixel on one of an enemy that counts
    /// under `rule`. A carrier takes the ship it's printing along, so that ship must fit where
    /// it would be carried to as well.
    fn hits(&self, i: usize, x: i32, y: i32, rule: Solid) -> bool {
        if self.clash(i, x, y, self.ships[i].facing, rule).0 > 0 {
            return true;
        }
        match self.ships[i].making {
            Making::Child(c) if self.ships[c].alive => {
                let off = self.ships[i].make_off;
                let (px, py) = self.carried_at(c, x + off.0, y + off.1);
                self.clash(c, px, py, self.ships[c].facing, rule).0 > 0
            }
            _ => false,
        }
    }

    /// Whether ship `i` centred on (x, y) would be inside something solid to it.
    pub(super) fn blocked(&self, i: usize, x: i32, y: i32) -> bool {
        self.hits(i, x, y, Solid::Moving)
    }

    /// Whether ship `i` centred on (x, y) would be inside something planted.
    pub(super) fn in_planted(&self, i: usize, x: i32, y: i32) -> bool {
        self.hits(i, x, y, Solid::Planted)
    }

    /// Whether a living pixel of the other side sits at a point: nothing grows or is printed
    /// into one.
    pub fn enemy_pixel(&self, team: usize, x: i32, y: i32) -> bool {
        self.ships.iter().any(|e| e.alive && e.team != team && e.warp == 0 && e.cell_at(x, y).is_some_and(|c| e.hp[c] > 0))
    }

    /// Whether something of `team`'s own is standing at a point: a living pixel of a ship that's
    /// growing out of a rock, or that has no thrust (ship `me` itself aside). Ships that fly, and
    /// ones still being printed, don't count: they pass over their own side's things.
    pub(super) fn own_standing_pixel(&self, team: usize, me: Option<usize>, x: i32, y: i32) -> bool {
        self.ships.iter().enumerate().any(|(k, e)| {
            Some(k) != me && e.alive && e.team == team && e.warp == 0 && e.maker.is_none() && (e.grow.is_some() || e.st.thrust == 0) && e.cell_at(x, y).is_some_and(|c| e.hp[c] > 0)
        })
    }

    /// Whether a pixel growing out of a rock for `team` has room at a point: no enemy pixel is
    /// there, and nothing of its own side is standing there (`me` is the grower, once it's landed).
    /// Two factories never share a pixel, whoever they belong to.
    pub fn room_to_grow(&self, team: usize, me: Option<usize>, x: i32, y: i32) -> bool {
        !self.enemy_pixel(team, x, y) && !self.own_standing_pixel(team, me, x, y)
    }

    /// Whether any of ship `i`'s pixels sits on a living enemy pixel of any size: nothing comes
    /// out of hyperspace into one.
    pub(super) fn in_enemy(&self, i: usize) -> bool {
        let s = &self.ships[i];
        self.clash(i, s.x, s.y, s.facing, Solid::Any).0 > 0
    }

    /// Moves ship `i` to (cx, cy), or as near as the rules let it, and returns where it ended up.
    ///
    /// From clear space the move is walked a pixel at a time. A step into something solid slides
    /// along whichever axis is free, or is lost: so a fighter stops flush against a capital's plate
    /// (or a factory's) and skims along it instead of flying over it and shooting it from inside,
    /// and nothing skips across a wall thinner than its step.
    ///
    /// A ship already inside a bigger enemy (the bigger ship flew onto it) takes the move if it
    /// clears the hull, or else heads for the nearest clear spot in any of 8 directions, up to
    /// 2 px a tick, never by way of anything planted. Only something that can move itself does
    /// that: a factory, or a hulk with no engines left, stays exactly where it is while the bigger
    /// ship passes over.
    pub(super) fn shift(&mut self, i: usize, cx: i32, cy: i32) -> (i32, i32) {
        let (ox, oy, h) = (self.ships[i].x, self.ships[i].y, self.ships[i].half());
        let (mut fx, mut fy) = (ox, oy);
        let inside = self.blocked(i, ox, oy);
        // Never true by the rules above. If it ever were, the ship is let out as from a bigger
        // enemy instead of being stuck for good.
        let stuck = inside && self.in_planted(i, ox, oy);
        if !inside {
            let (dx, dy) = (cx - ox, cy - oy);
            let steps = dx.abs().max(dy.abs());
            let (mut px, mut py) = (ox, oy);
            for k in 1..=steps {
                // This step of the straight line from where it is to where it's going: at most
                // one pixel each way.
                let (qx, qy) = (ox + dx * k / steps, oy + dy * k / steps);
                let (sx, sy) = (qx - px, qy - py);
                (px, py) = (qx, qy);
                if !self.blocked(i, fx + sx, fy + sy) {
                    fx += sx;
                    fy += sy;
                } else if sx != 0 && sy != 0 && !self.blocked(i, fx + sx, fy) {
                    fx += sx;
                } else if sx != 0 && sy != 0 && !self.blocked(i, fx, fy + sy) {
                    fy += sy;
                }
            }
        } else {
            (fx, fy) = (cx, cy);
            if self.blocked(i, cx, cy) {
                let mut exit: Option<(i32, (i32, i32))> = None;
                let dirs: &[(i32, i32)] = if self.self_moving(i) { &[(1, 0), (-1, 0), (0, 1), (0, -1), (1, 1), (1, -1), (-1, 1), (-1, -1)] } else { &[] };
                for &(dx, dy) in dirs {
                    for k in 1..=24 {
                        let (px, py) = (ox + dx * k, oy + dy * k);
                        if px < h || py < h || px > self.w - 1 - h || py > self.h - 1 - h {
                            break;
                        }
                        // There's no way out through something planted.
                        if !stuck && self.in_planted(i, px, py) {
                            break;
                        }
                        if !self.blocked(i, px, py) {
                            if exit.map_or(true, |(best, _)| k < best) {
                                exit = Some((k, (dx, dy)));
                            }
                            break;
                        }
                    }
                }
                (fx, fy) = exit.map_or((ox, oy), |(k, (dx, dy))| (ox + dx * k.min(2), oy + dy * k.min(2)));
            }
            // On its way out it may cross the bigger ship's pixels, but it stops short of anything
            // planted.
            let (dx, dy) = (fx - ox, fy - oy);
            let reach = dx.abs().max(dy.abs());
            (fx, fy) = (ox, oy);
            for k in 1..=reach {
                let (qx, qy) = (ox + dx * k / reach, oy + dy * k / reach);
                if !stuck && self.in_planted(i, qx, qy) {
                    break;
                }
                (fx, fy) = (qx, qy);
            }
        }
        if cfg!(test) {
            if !inside {
                assert!(!self.blocked(i, fx, fy), "{} was moved from clear space into a bigger enemy at ({fx},{fy})", self.ships[i].name);
            }
            if !stuck {
                assert!(!self.in_planted(i, fx, fy), "{} was moved into something planted at ({fx},{fy})", self.ships[i].name);
            }
        }
        let s = &mut self.ships[i];
        s.x = fx;
        s.y = fy;
        (fx, fy)
    }

    /// Whether a ship can move itself: it has thrust, or it's a king (steered whatever state its
    /// engines are in).
    pub(super) fn self_moving(&self, i: usize) -> bool {
        self.ships[i].st.thrust > 0 || self.ships[i].role == Role::King
    }

    /// Turns ship `i` to face `nf`, unless the turn would swing more of its pixels into a bigger
    /// enemy's hull than are there already, or any into something planted. True if it turned.
    pub(super) fn try_turn(&mut self, i: usize, nf: u8) -> bool {
        let (x, y, f0) = (self.ships[i].x, self.ships[i].y, self.ships[i].facing);
        let more = |rule: Solid| self.clash(i, x, y, nf, rule).0 > self.clash(i, x, y, f0, rule).0;
        let ok = !more(Solid::Moving) && !more(Solid::Planted);
        if ok {
            self.ships[i].facing = nf;
        }
        ok
    }

    /// Where a carried ship put at (x, y) ends up: there, kept on the map.
    fn carried_at(&self, i: usize, x: i32, y: i32) -> (i32, i32) {
        let half = self.ships[i].half();
        (x.clamp(half, self.w - 1 - half), y.clamp(half, self.h - 1 - half))
    }

    /// Places a ship that isn't on the map in its own right: one still in hyperspace, or a
    /// half-printed ship riding beside its carrier (the carrier's own move already made sure it
    /// fits). Kept on the map.
    pub(super) fn carry(&mut self, i: usize, x: i32, y: i32, facing: Option<u8>) {
        if let Some(f) = facing {
            self.ships[i].facing = f;
        }
        let (x, y) = self.carried_at(i, x, y);
        let s = &mut self.ships[i];
        s.x = x;
        s.y = y;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A fight with nothing in it but what the test puts there.
    fn empty() -> Arena {
        Arena::new()
    }

    fn shares(a: &Arena, i: usize, j: usize) -> bool {
        let (s, e) = (&a.ships[i], &a.ships[j]);
        (0..s.hp.len()).any(|c| {
            s.hp[c] > 0 && {
                let (x, y) = s.cell_xy(c);
                e.cell_at(x, y).is_some_and(|d| e.hp[d] > 0)
            }
        })
    }

    #[test]
    fn a_fast_thin_ship_cant_skip_through_a_thin_planted_wall() {
        // A dart one pixel wide moving 3 px a tick, and a planted wall one pixel thick. Checking
        // only where a move ends would let the dart land beyond the wall without ever touching it,
        // depending on where its steps happened to fall. Every phase of the approach is tried; the
        // dart never gets past, and stops right up against the wall.
        let wall = Design::from_rows("WALL", Order::Hold, &["p", "p", "p", "c", "p", "p", "p"]);
        let dart = Design::from_rows("DART", Order::Hunt, &["c", "e"]);
        for phase in 0..3 {
            let mut a = empty();
            let w = a.spawn(1, 0, &wall, 200, 150, 0, false);
            a.ships[w].rooted = true;
            let d = a.spawn(0, 0, &dart, 180 + phase, 150, 0, false);
            for _ in 0..20 {
                let (x, y) = (a.ships[d].x, a.ships[d].y);
                let (nx, _) = a.shift(d, x + 3, y);
                assert!(nx < 200, "phase {phase}: the dart got past the wall, to x = {nx}");
                assert!(!shares(&a, d, w));
            }
            assert_eq!(a.ships[d].x, 199, "phase {phase}: the dart should end up against the wall");
        }
    }

    #[test]
    fn a_fast_thin_ship_cant_skip_through_a_thin_bigger_ship() {
        // The same for a ship that's merely bigger, not planted.
        let wall = Design::from_rows("SPAR", Order::Hold, &["p", "p", "p", "c", "p", "p", "p"]);
        let dart = Design::from_rows("DART", Order::Hunt, &["c", "e"]);
        for phase in 0..3 {
            let mut a = empty();
            let w = a.spawn(1, 0, &wall, 200, 150, 0, false);
            let d = a.spawn(0, 0, &dart, 180 + phase, 150, 0, false);
            for _ in 0..20 {
                let (x, y) = (a.ships[d].x, a.ships[d].y);
                let (nx, _) = a.shift(d, x + 3, y);
                assert!(nx < 200, "phase {phase}: the dart got past the spar, to x = {nx}");
            }
            let _ = w;
        }
    }

    #[test]
    fn the_way_out_from_under_a_bigger_ship_never_goes_through_a_factory() {
        // A fighter finds itself under a big enemy slab (the slab flew onto it), and the nearest
        // clear space lies east, past an enemy wall planted there. It must not be squeezed out
        // through the wall: it goes another way, never shares a pixel with the wall, and gets out.
        let slab = Design::from_rows("SLAB", Order::Hold, &["hhhhhhhhhhhhhhhhhhhhh"; 21]);
        let wall = Design::from_rows("WALL", Order::Hold, &["p", "p", "p", "p", "c", "p", "p", "p", "p"]);
        let fighter = Design::from_rows("FIGHTER", Order::Hunt, &["hch", ".e."]);
        let mut a = empty();
        let mut rows: Vec<String> = vec!["h".repeat(21); 21];
        rows[10] = format!("{}c{}", "h".repeat(10), "h".repeat(10));
        let rows: Vec<&str> = rows.iter().map(|r| r.as_str()).collect();
        let slab = Design::from_rows(&slab.name, Order::Hold, &rows);
        let b = a.spawn(1, 0, &slab, 200, 150, 0, false);
        let w = a.spawn(1, 0, &wall, 209, 150, 0, false);
        a.ships[w].rooted = true;
        let f = a.spawn(0, 0, &fighter, 205, 150, 0, false);
        assert!(a.blocked(f, 205, 150) && !a.in_planted(f, 205, 150));
        let mut out = None;
        for t in 0..40 {
            let (x, y) = (a.ships[f].x, a.ships[f].y);
            a.shift(f, x, y);
            assert!(!shares(&a, f, w), "tick {t}: the fighter is inside the wall at {:?}", (a.ships[f].x, a.ships[f].y));
            if out.is_none() && !shares(&a, f, b) {
                out = Some(t);
            }
        }
        assert!(out.is_some(), "the fighter never got out from under the slab: it's at {:?}", (a.ships[f].x, a.ships[f].y));
    }

    #[test]
    fn a_ship_cant_turn_into_a_factory() {
        // A long ship lying north-south beside a planted block can't swing round into it, even
        // though it's the bigger of the two.
        let block = Design::from_rows("BLOCK", Order::Hold, &["ppp", "pcp", "ppp"]);
        let long = Design::from_rows("LONG", Order::Hunt, &["h", "h", "h", "h", "c", "h", "h", "h", "e"]);
        let mut a = empty();
        let w = a.spawn(1, 0, &block, 204, 150, 0, false);
        a.ships[w].rooted = true;
        let l = a.spawn(0, 0, &long, 200, 150, 0, false);
        assert!(!shares(&a, l, w));
        assert!(!a.try_turn(l, 1), "it turned into the block");
        assert!(!a.try_turn(l, 3), "it turned into the block");
        assert_eq!(a.ships[l].facing, 0);
        a.ships[w].rooted = false;
        assert!(a.try_turn(l, 1), "with nothing planted there, a bigger ship turns freely");
    }

    /// Ship positions and facings are only ever written in this file (and where a ship is first
    /// put on the map). A line anywhere else in the sim that moves a ship directly skips the rules
    /// above; this test fails on it and says where.
    #[test]
    fn only_this_file_moves_ships() {
        for (name, src) in [("sim.rs", include_str!("../sim.rs")), ("sim/eco.rs", include_str!("eco.rs"))] {
            let code = src.split("#[cfg(test)]").next().unwrap();
            for (n, line) in code.lines().enumerate() {
                let l = line.trim();
                // Loose pixels (`d.`) drift on their own; `// spawn` marks a ship first being placed.
                if l.starts_with("//") || l.starts_with("d.") || l.contains("// spawn") {
                    continue;
                }
                let moves = [".x = ", ".y = ", ".x += ", ".y += ", ".x -= ", ".y -= ", ".facing = "].iter().any(|p| l.contains(p));
                assert!(!moves, "{name}:{}: `{l}` moves a ship directly. Use shift, try_turn or carry in sim/space.rs.", n + 1);
            }
        }
    }
}
