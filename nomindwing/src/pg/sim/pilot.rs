//! Nose-first flight: one ship's own motion, and the pilots that fly it.
//!
//! `Flyer` is everything about a ship that flying moves: where it is, its speed, the way it
//! faces, and how far through a turn and a burn it is. The plain pilot's rules work on one
//! (`steer`, `arrive`), so the game flies a ship with them (`Arena::pilot`) and a ship can fly
//! a copy of itself forward with the very same rules.
//!
//! That's the rollout pilot (`Arena::on_course`). A ship in a fight plays its own next four
//! seconds forward some thirty ways: as its plan has it, holding each of a spread of speeds,
//! stepping aside each way for a moment before carrying on, and going round its target
//! either way. Nothing else is played forward. The enemy ships are carried on at the speed
//! they have, and the shots in the air fly on along their lines. Each way is weighed, in
//! pixels, by what it would cost the ship and what it would let the ship do:
//!
//! - a shot already in the air that would meet it;
//! - enemy guns that could bring a shot onto it: a flak gun or a turret fires at where the
//!   ship will be if it keeps the speed it has, so a ship that's changing its speed isn't
//!   there, and a turret has to swing its barrel round first (an enemy's guns are shared
//!   among the ships of this side inside their reach);
//! - an enemy lance, down the line it's on;
//! - its own guns in reach, and its lances on a row the target still has pixels in;
//! - how far it is from where its plan wants it, over the whole way. For a ship whose guns
//!   need no lining up that's a range from its target, on whatever side, and each pixel off
//!   it costs a pixel. For any other it's a spot, with a short leash round it that costs
//!   next to nothing. So a ship dodges where its orders put it, and doesn't leave. A ship
//!   on its way to where it was sent has that place for its spot, so it goes, and no faster
//!   than its group's pace;
//! - each quarter turn, a little, and changing its mind, a little.
//!
//! It takes the cheapest, keeps it for a few ticks, and thinks again. Everything is whole
//! numbers, and ships think in the order they're listed in, so two machines get the same
//! answers.
//!
//! Each of those was switched off in turn and measured (MATCH.md, 2026-11-16). Weaving back
//! and forth was a way a ship could go, and earned nothing; sharing an enemy's fire among
//! every ship in the fight, not only the ones in its reach, sent ships in to be shot.

use super::*;

/// How far ahead a ship plays its own motion, in ticks: four seconds.
const AHEAD: usize = 60;

/// Whether a ship's own next four seconds are worth its weighing: whether its strongest
/// engines can burn off its top speed inside them. One that can't has no way to fly that
/// shows any gain in the time it looks ahead (a stop it began now would still be going on
/// when the four seconds were up), so whatever speed it had once taken up it kept: it flies
/// as its plan says instead, like a ship with no guns.
///
/// (Every card the game ships with can: the slowest to is the SWARM's WASP, at 48 ticks,
/// and a ship whose top speed is above the least there is takes about 24. Henry's own MAUL
/// of 2026-11-22 is far under that least speed's weight for four engines and takes 115: on
/// guard under fire it was on a course of its own for thousands of ticks, as far as the
/// edge of the map. The wider blind spot, a ship that needs longer than four seconds to
/// turn about and stop, is NC3's and is as it was.)
pub(super) fn weighs(st: &Stats) -> bool {
    match *st.push.iter().max().unwrap() {
        0 => true,
        top => (st.vmax * PUSH_PER_WEIGHT * st.mass.max(1) / top) as usize <= AHEAD,
    }
}
/// A ship picks a course every this many ticks. Ships take turns, by their place in the list.
const EVERY: u32 = 4;
/// Guns, its own and the enemy's, are reckoned at every this many ticks of the way.
const STEP: usize = 5;
/// With no enemy this near (px) and no enemy shot near, a ship flies as its plan says and
/// thinks about nothing.
const SIGHT: i32 = 240;
/// What a course costs or earns is counted in sixteenths of a pixel.
const PX: i32 = 16;
/// What each quarter-pixel a tick of speed a ship still has at the end costs, and each pixel
/// it is from the spot its plan wants it on, averaged over the way, out to `LEASH`.
const OFF: i32 = 1;
/// How far (px) a ship may be from the spot its plan wants it on for next to nothing: room to
/// step out of a shot's way, or to fly a lance across a target. Each pixel beyond costs a
/// whole pixel, and so does each pixel off the range a ship with no gun to line up is wanted
/// at. Its orders aren't the pilot's to change: it dodges where it's sent, and doesn't leave.
const LEASH: i32 = 24;
/// What the course a ship is already on is let off: it changes its mind only for something
/// clearly better.
const KEEP: i32 = PX;
/// What each quarter turn on the way costs. Turning is free to do, and a ship that turns for
/// every small thing looks as if it can't make up its mind.
const TURN: i32 = PX;
/// What a tick spent overlapping a bigger enemy's box costs: it's solid, and the ship would
/// be bumping along it.
const SOLID: i32 = 4 * PX;

/// Pixels a shot of each kind is taken to cost the ship it meets.
fn hurt(kind: Kind) -> i32 {
    match kind {
        Kind::Lance => 4,
        Kind::Shell | Kind::Missile => 8,
        _ => 1,
    }
}

/// One ship's own motion: see the top of this file.
#[derive(Clone, Copy, Debug)]
pub(super) struct Flyer {
    pub x: i32,
    pub y: i32,
    sx: i32,
    sy: i32,
    pub vx: i32,
    pub vy: i32,
    pub facing: u8,
    turn_t: i32,
    acc2: (i32, i32),
    pub burn: [bool; 4],
    pub want: (i32, i32),
    /// How near a point it has to get, when that's nearer than `near_enough` (0: it isn't).
    pub fine: i32,
}

impl Flyer {
    pub fn of(s: &Ship) -> Flyer {
        Flyer { x: s.x, y: s.y, sx: s.sx, sy: s.sy, vx: s.vx, vy: s.vy, facing: s.facing, turn_t: s.turn_t, acc2: s.acc2, burn: s.burn, want: s.want, fine: s.fine }
    }

    /// Gives a ship what steering changed: its speed, how far through a turn and a burn it is,
    /// its exhaust, and what it's making for. (Its place and its facing are `Arena::fly`'s to
    /// change: whatever is solid to the ship has a say in those.)
    pub fn put(&self, s: &mut Ship) {
        (s.vx, s.vy, s.turn_t, s.acc2, s.burn, s.want) = (self.vx, self.vy, self.turn_t, self.acc2, self.burn, self.want);
    }

    /// The plain pilot, for one tick. A ship's engines push it only the ways they're turned to
    /// (`Stats::push`, turned with the ship), it keeps the speed it has until something pushes
    /// against it, and it turns a quarter at a time, as fast as its rotation engines manage
    /// (`Stats::quarter`).
    ///
    /// It burns whichever engines bring its speed nearer to `dv`, along and across the map. It
    /// flies in legs: while its engines are doing a fair share of that as it faces (a third or
    /// more of the most any facing would let them do, `WEAK`), it keeps facing that way; when
    /// they aren't and there's still a change to make, it turns to the way that lets them do
    /// the most of it; and with no change left to make it faces `face` (to point a gun). The
    /// quarter turn to make, if one is due.
    pub fn steer(&mut self, st: &Stats, dv: (i32, i32), face: Option<u8>) -> Option<u8> {
        /// A change of speed this small or smaller (quarter-pixels a tick) is near enough.
        const NEAR: i32 = 1;
        let s = self;
        s.burn = [false; 4];
        // What it's making for: the speed asked for, cut to what it can do.
        let w = isqrt(dv.0 * dv.0 + dv.1 * dv.1);
        s.want = if w > st.vmax && w > 0 { (dv.0 * st.vmax / w, dv.1 * st.vmax / w) } else { dv };
        if st.push == [0; 4] {
            // Nothing to push with: it drifts on as it was going, and can't turn.
            (s.acc2, s.turn_t) = ((0, 0), 0);
            return None;
        }
        let need = (s.want.0 - s.vx, s.want.1 - s.vy);
        // The side of the map each change of speed has to be pushed towards (0 north, 1 east,
        // 2 south, 3 west), and how much of it there is: across the map, then up and down it.
        let across = (if need.0 > 0 { Some(1usize) } else if need.0 < 0 { Some(3) } else { None }, need.0.abs());
        let along = (if need.1 > 0 { Some(2usize) } else if need.1 < 0 { Some(0) } else { None }, need.1.abs());
        // What a ship facing `f` can push with towards a side of the map.
        let towards = |f: u8, side: usize| st.push[(side + 4 - f as usize) & 3];
        let rate = PUSH_PER_WEIGHT * st.mass.max(1);
        let facing = s.facing;
        let burn = |side: Option<usize>, much: i32, acc: &mut i32, burning: &mut [bool; 4]| -> i32 {
            match side {
                Some(side) if towards(facing, side) > 0 => {
                    *acc += towards(facing, side);
                    let dv = (*acc / rate).min(much);
                    *acc %= rate;
                    burning[(side + 4 - facing as usize) & 3] = true;
                    dv
                }
                _ => {
                    *acc = 0;
                    0
                }
            }
        };
        let (mut acc2, mut burning) = (s.acc2, [false; 4]);
        s.vx += need.0.signum() * burn(across.0, across.1, &mut acc2.0, &mut burning);
        s.vy += need.1.signum() * burn(along.0, along.1, &mut acc2.1, &mut burning);
        (s.acc2, s.burn) = (acc2, burning);
        let v = isqrt(s.vx * s.vx + s.vy * s.vy);
        if v > st.vmax && v > 0 {
            s.vx = s.vx * st.vmax / v;
            s.vy = s.vy * st.vmax / v;
        }
        // Which way to face. A change of speed counts if it's more than next to nothing. Any
        // change counts if it's a slowing down (a ship that begins to brake late overshoots),
        // for a ship that's meant to be stopping (the speed it has left over), and for one
        // that's standing still and meant to move.
        let whole = s.want == (0, 0) || (s.vx, s.vy) == (0, 0);
        let (vx, vy) = (s.vx, s.vy);
        let counts = |much: i32, slowing: bool| much > if whole || slowing { 0 } else { NEAR };
        let changes = [(across.0, across.1, need.0 * vx < 0), (along.0, along.1, need.1 * vy < 0)];
        // How much of the change a facing would let its engines work on.
        let worth = |f: u8| changes.iter().map(|&(side, much, slowing)| if counts(much, slowing) { side.map_or(0, |side| much * towards(f, side)) } else { 0 }).sum::<i32>();
        let best = (0..4u8).max_by_key(|&f| (worth(f), f == facing)).unwrap();
        // (Going slowly, a small change isn't worth a turn: whatever engine pushes that way
        // does it. `SLOW` is the speed, and the change.)
        let small = across.1.max(along.1) <= SLOW && vx.abs().max(vy.abs()) <= SLOW;
        let wish = if worth(facing) > 0 && (worth(facing) * WEAK >= worth(best) || small) {
            // Its engines are doing a fair share of it as the ship faces: it finishes this leg.
            None
        } else if worth(best) > 0 {
            Some(best)
        } else {
            face
        };
        match wish {
            Some(f) if f != s.facing && st.quarter > 0 => {
                s.turn_t += 1;
                if s.turn_t >= st.quarter {
                    s.turn_t = 0;
                    return Some(if (f + 4 - s.facing) % 4 == 3 { (s.facing + 3) % 4 } else { (s.facing + 1) % 4 });
                }
            }
            _ => s.turn_t = 0,
        }
        None
    }

    /// Arriving: the speed to make for now, across the map and along it at once, so as to fly
    /// straight at a point and stop on it.
    ///
    /// Each way (across, along) gets its share of the ship's top speed, so the two together
    /// point at the place. Each is held to the fastest the ship may be going that way and
    /// still stop in the room it has left that way. And each is stopped in one burn, when
    /// it's going faster than that: asked for no speed at all that way, not a little less.
    ///
    /// A ship with one engine can stop only one way at a time, so the two stops are taken
    /// in turn: the shorter way first, then the longer. For that the shorter way has to be
    /// done early, so it's flown a little faster than its share, and the ship's line points
    /// at where the longer way's own stop will begin. The longer way's room has to cover
    /// the shorter's whole stop, turn and burn, and then its own turn; if it runs short of
    /// that, the shorter way is stopped at once. Setting off, it gets up speed the shorter
    /// way first, so that it flies with its nose along the longer. So a ship with one engine
    /// astern burns the shorter way, turns a quarter, burns the longer, coasts at the place,
    /// turns a quarter and stops the shorter, turns a quarter and stops the longer: three
    /// quarter turns, and near enough a straight line.
    ///
    /// The pilot (`steer`) does the burning and turning, to cut whatever gap there is between
    /// the speed the ship has and this one. So that it isn't for ever trimming, a speed that's
    /// near enough what's wanted, or more than it but still safe to stop from, is left be.
    ///
    /// (Until 2026-11-16 a ship flew one leg at a time, across and then along, so that it
    /// only ever had one way's speed to stop: half as far again as the straight way, by way
    /// of a corner. Henry: "we need ships to fly approximately straight towards their
    /// destinations. It will be surprising behavior for the player otherwise.")
    pub fn arrive(&self, st: &Stats, (gx, gy): (i32, i32)) -> (i32, i32) {
        let s = self;
        // On the point it stops. Stopped as near it as a ship of its sort can be sure of
        // placing itself (`near_enough`: 3 px for one with rotation engines, 6 for the starter
        // fighter, 14 for a frigate with none), it stays.
        // (A ship asked to place itself finer than that, as a miner is at the rock, stays only
        // when it's that near.)
        // (Nor finer than its burns let it: `burn_px`.)
        let fine = s.fine > 0 && s.fine < near_enough(st);
        let (near, stopped, off) = (if fine { s.fine.max(2 + burn_px(st)) } else { near_enough(st) }, (s.vx, s.vy) == (0, 0), cheb(s.x, s.y, gx, gy));
        if off <= 2 || (off <= near && stopped) {
            return (0, 0);
        }
        let top = *st.push.iter().max().unwrap();
        if top == 0 {
            return (0, 0);
        }
        // (A ship that keeps its speed can't shuffle: the shortest hop it can make and stop
        // from is as far as it drifts while it turns its engine about, `shortest_hop`. Sent
        // to a point nearer than that, it sets off all the same, is carried past, and stops
        // as near on the far side as it started on this one, or nearer.)
        let hop = shortest_hop(st);
        let (d, v) = ([gx - s.x, gy - s.y], [s.vx, s.vy]);
        // Standing a hop or two from the point, it makes for it one way at a time, the
        // longer first, at the slowest speed there is, whatever the room: it's carried a
        // little past, and stops nearer than it was. (Reckoning both ways' stops from how
        // it happened to face, which way looked better changed with every turn it made
        // to set off, and it turned on the spot for ever.)
        if stopped && off <= 2 * hop {
            // Asked to place itself finely, and nearer the point than it can stop on: it
            // sets off the other way. Turning about to come back carries it a hop further
            // off, and from there it can come in and stop on the point. (Setting off towards
            // it, it would be carried as far past as it began short, and so on for good.
            // Henry, 2026-11-17: "Ships should be able to fly short distances, the flying
            // algorithm might need to be improved if they can't.")
            let way = if fine && off < hop { -1 } else { 1 };
            return if d[0].abs() >= d[1].abs() { (way * d[0].signum(), 0) } else { (0, way * d[1].signum()) };
        }
        // Its speed towards the point each way (less than nothing: it's going the wrong way).
        let tv = [v[0] * d[0].signum(), v[1] * d[1].signum()];
        let (rate, quarter) = ((PUSH_PER_WEIGHT * st.mass.max(1)) as i64, st.quarter as i64);
        // The side of the map a stop along axis `a` has to push towards: back along its way.
        let back = |a: usize| -> usize {
            if a == 0 {
                if d[0] > 0 { 3 } else { 1 }
            } else if d[1] > 0 {
                0
            } else {
                2
            }
        };
        // The engine it would push towards a side of the map with if it faced `f`, and the
        // quarter turns that bring that engine to bear: of its engines with a fair share of
        // its strongest's push (`WEAK`), the one needing fewest turns, the stronger of two
        // as near. (One that can't turn has only what already points there.)
        let gear = |side: usize, f: u8| -> Option<(usize, i64)> {
            (0..4usize)
                .filter(|&way| st.push[way] > 0 && st.push[way] * WEAK >= top)
                .map(|way| {
                    (way, match (side + 8 - way - f as usize) & 3 {
                        0 => 0i64,
                        2 => 2,
                        _ => 1,
                    })
                })
                .filter(|&(_, turns)| turns == 0 || quarter > 0)
                .min_by_key(|&(way, turns)| (turns, -st.push[way]))
        };
        // The one it would stop its way along axis `a` with.
        let brake = |a: usize, f: u8| gear(back(a), f);
        // Going away from the point one way, and the other way too fast to stop in the room
        // it has once that's been turned round: it stops dead first, and sets off again from
        // rest. It has been sent somewhere else while it was at speed, or been carried past.
        // (Steering for the point from here, a ship with one engine turned the one way's
        // speed round while the other way's carried it past, and then the same the other way
        // about, round and round the point for good: of 416 starts at speed, Henry's fighter
        // never came to rest from 11, and from 17 at the speed it had the day before. Found
        // 2026-11-17 by a fighter in a match, sent to a spot 18 px behind it. From rest every
        // ship gets there. A ship only going away, with room the other way, turns round as
        // it always did: a ship holding its range from something that moves does that all
        // the time.)
        for a in 0..2 {
            let b = 1 - a;
            if !(tv[a] < 0 && d[a].abs() > 2 && tv[b] > 0) {
                continue;
            }
            // Turning the one way's speed round: the turns to the engine, and the burn.
            let Some((way, turns)) = gear((back(a) + 2) & 3, s.facing) else { continue };
            let round = turns * quarter + (-tv[a]) as i64 * rate / st.push[way] as i64 + 1;
            let facing_after = if turns > 0 { (((back(a) + 2) & 3) + 4 - way) as u8 & 3 } else { s.facing };
            // Then the other way's stop: what it covers meanwhile, and stopping.
            let Some((stopper, more)) = brake(b, facing_after) else { continue };
            let (vb, room) = (tv[b] as i64, ((d[b].abs() - 2).max(0) * 4) as i64);
            if vb * (round + more * quarter) + vb * vb * rate / (2 * st.push[stopper] as i64) > room {
                return (0, 0);
            }
        }
        // The shorter way and the longer.
        let (short, long) = if d[0].abs() <= d[1].abs() { (0, 1) } else { (1, 0) };
        let hyp = isqrt(d[0] * d[0] + d[1] * d[1]).max(1);
        // A stretch too short to stop on is left until the other way is flown: begun first,
        // it would be overshot and come back over, again and again.
        let later = |a: usize| d[a].abs() > 2 && d[a].abs() <= hop && d[1 - a].abs() > hop && v[a] == 0;
        // Each way's share of its top speed, flying straight at the point. With the shorter
        // way left till later, the longer has all of it. (Until 2026-11-17 it had only its
        // share, rounded down: Henry's miner of that day, with three steps of speed, flew
        // 200 px across and 23 along at two of them.)
        let mut share = [(st.vmax * d[0].abs() / hyp).max(1), (st.vmax * d[1].abs() / hyp).max(1)];
        if later(short) {
            share[long] = st.vmax.max(1);
        }
        // The shorter way's stop: its engine, the turns to it as the ship faces, and how the
        // ship will face when it's done. Then the longer way's, from there. (With the shorter
        // way left till later there's no such stop before the longer's, and the longer's
        // turns are counted from the way the ship faces now. Until 2026-11-17 they were
        // counted as if that stop had been made: a quarter turn too few for a ship with one
        // engine, which ran past the end of the longer way by what it flies in a quarter
        // turn. Henry's miner, 22 px. The same goes when both ways are too short to stop on
        // and it's flying only the longer: a collector coming in to a spot it had backed
        // off from ran 10 px past it, backed off the other way, and so on for good.)
        let first = brake(short, s.facing);
        let unflown = v[short] == 0 && d[short].abs() <= hop;
        let facing_then = match first {
            Some((way, turns)) if turns > 0 && d[short].abs() > 2 && !unflown => ((back(short) + 4 - way) & 3) as u8,
            _ => s.facing,
        };
        let second = brake(long, facing_then);
        // How long a stop from speed `v` with engine `way` keeps that engine, after `turns`.
        let stop_time = |gear: Option<(usize, i64)>, v: i32| gear.map_or(0, |(way, turns)| turns * quarter + v.max(0) as i64 * rate / st.push[way] as i64 + 1);
        // The longer way's stop can't begin until the shorter's is done: its room has to
        // cover that as well, as things stand now.
        let busy = if tv[short] > 0 { stop_time(first, tv[short]) } else { 0 };
        let wait = if short == 0 { [first.map_or(0, |g| g.1 * quarter), busy + second.map_or(0, |g| g.1 * quarter)] } else { [busy + second.map_or(0, |g| g.1 * quarter), first.map_or(0, |g| g.1 * quarter)] };
        let gear = if short == 0 { [first, second] } else { [second, first] };
        // The fastest it may go along axis `a` and still stop in the room it has there: going
        // v it covers v x wait while it waits for its engine and turns it, and
        // v x v x rate / (2 x push) while it brakes: the v for which that's the room. (Two
        // pixels short of the point, for the steps its speed changes in.)
        let limit = |a: usize| -> i32 {
            let room = ((d[a].abs() - 2).max(0) * 4) as i64;
            gear[a].map_or(0, |(way, _)| {
                let push = st.push[way] as i64;
                ((isqrt_i64(push * push * wait[a] * wait[a] + 2 * push * rate * room) - push * wait[a]) / rate) as i32
            })
        };
        let limits = [limit(0), limit(1)];
        let due = [tv[0] > limits[0], tv[1] > limits[1]];
        // The shorter way is stopped when it's due, or when the longer is (whose room counts
        // on the shorter being out of the way). The longer is stopped once the shorter is.
        let mut stop = [false; 2];
        stop[short] = due[short] || (due[long] && tv[short] > 0);
        stop[long] = due[long] && tv[short] <= 0;
        // So that the shorter way is done in time, it's flown faster than its share: by as
        // much as gets it there while the longer way still has its tail to fly (what it
        // covers while the shorter is stopped and its own engine turned, and its own stop).
        let cruise = [share[0].min(limits[0]), share[1].min(limits[1])];
        let tail = {
            let vl = if tv[long] > 0 { tv[long] } else { cruise[long] } as i64;
            let wait = stop_time(first, cruise[short]) + second.map_or(0, |g| g.1 * quarter);
            let slow = second.map_or(0, |(way, _)| vl * vl * rate / (2 * st.push[way] as i64));
            ((vl * wait + slow) / 4) as i32
        };
        let far_long = d[long].abs();
        let mut want = cruise;
        if d[short].abs() > hop {
            let lead = (far_long - tail).max(far_long / 2).max(1);
            want[short] = (share[short] * far_long / lead).min(limits[short]).max(1);
        }
        let mut dv = [0, 0];
        for a in 0..2 {
            let far = d[a].abs();
            // On its line that way: no speed wanted (and any it has, to be lost). Nor for a
            // stretch that's left until the other way is flown.
            if far <= 2 || later(a) {
                continue;
            }
            if stop[a] {
                // The last way left to stop is slowed as its room runs out, so that it
                // stops on the point: nothing else is waiting for its engine. (The first
                // is stopped all at once, to have it out of the way.)
                if a == long && tv[short] <= 0 {
                    dv[a] = d[a].signum() * limits[a].min(far * 4);
                }
                continue;
            }
            let want = want[a].min(far * 4);
            // Whether an engine pushes that way as the ship faces.
            let onward: usize = if a == 0 {
                if d[0] > 0 { 1 } else { 3 }
            } else if d[1] > 0 {
                2
            } else {
                0
            };
            let pushes = st.push[(onward + 4 - s.facing as usize) & 3] > 0;
            dv[a] = if tv[a] > 0 && tv[a] <= limits[a] && (tv[a] >= want || (!pushes && tv[a] * 2 >= want)) {
                // That or more, and safe to stop from: left be. So is half of it or more,
                // if topping it up would take a turn: the stop that's coming needs the
                // engine where it is.
                v[a]
            } else {
                d[a].signum() * want
            };
        }
        // Setting off, the shorter way first: the longer isn't asked for until the shorter
        // is up to speed, so the ship ends up with its nose along the way it's mostly going.
        if tv[long] <= 0 && dv[short] != 0 && dv[short] != v[short] && !stop[long] {
            dv[long] = v[long].max(0) * 0 + if tv[long] < 0 { 0 } else { v[long] };
        }
        if dv == [0, 0] && stopped {
            // A ship that's standing sets off at the slowest speed there is, whatever the
            // room, the longer way first.
            dv[long] = d[long].signum();
        }
        (dv[0], dv[1])
    }

    /// The way to face while it makes for speed `dv` with nothing to burn: its nose along
    /// the way it's mostly going, while it's going both ways at once. With only one way
    /// under way (the other done, or being stopped, or not begun) it holds the facing it
    /// has: the stop it has left is a quarter turn from there, and two from nose-first.
    pub fn nose(dv: (i32, i32)) -> Option<u8> {
        if dv.0 != 0 && dv.1 != 0 && dv.0.abs() + dv.1.abs() > 2 {
            axis_face(dv.0, dv.1)
        } else {
            None
        }
    }

    /// Where it is after `ticks` of making for the speed `dv`, with nothing in its way.
    pub fn flown(mut self, st: &Stats, dv: (i32, i32), ticks: i32, half: i32, w: i32, h: i32) -> Flyer {
        for _ in 0..ticks.max(0) {
            if let Some(turned) = self.steer(st, dv, None) {
                self.facing = turned;
            }
            self.drift(half, w, h);
        }
        self
    }

    /// One tick of going on at its speed: its place moves, and the edge of the map stops it.
    /// (In the game `Arena::fly` does this, and whatever is solid to the ship has its say
    /// after. Played forward, a ship takes nothing for solid: bumping costs it instead.)
    pub fn drift(&mut self, half: i32, w: i32, h: i32) {
        self.sx += self.vx;
        self.sy += self.vy;
        let (dx, dy) = (self.sx.div_euclid(4), self.sy.div_euclid(4));
        self.sx = self.sx.rem_euclid(4);
        self.sy = self.sy.rem_euclid(4);
        let (nx, ny) = (self.x + dx, self.y + dy);
        let (cx, cy) = (nx.clamp(half, w - 1 - half), ny.clamp(half, h - 1 - half));
        if cx != nx {
            self.vx = 0;
            self.sx = 0;
        }
        if cy != ny {
            self.vy = 0;
            self.sy = 0;
        }
        (self.x, self.y) = (cx, cy);
    }
}

/// An enemy ship as a ship thinking about its course takes it: where it is and how it's going
/// (it's carried on at that speed), how big, the way it faces, and its guns.
struct Threat {
    x: i32,
    y: i32,
    vx: i32,
    vy: i32,
    half: i32,
    full: i32,
    facing: u8,
    flak: i32,
    /// The lines its working lances fire down, as it faces: how far each is across from its
    /// middle, and how many lances are on it.
    lanes: Vec<(i32, i32)>,
    /// Each working turret: where its barrel points (64ths of a turn), and whether it's a
    /// tracker (else a cannon).
    turrets: Vec<(i32, bool)>,
    /// The sides a gun can break something of it from (`Arena::open_sides`).
    open: [u16; 2],
    /// How many ships it has to share its flak between, and its turrets: the thinking ship,
    /// and each of that ship's side already inside the gun's reach. An enemy's guns can't all
    /// be for one ship when others are as near.
    flak_for: i32,
    turret_for: i32,
}

impl Threat {
    /// Where it'll be `t` ticks on, at the speed it has.
    fn at(&self, t: usize) -> (i32, i32) {
        (self.x + self.vx * t as i32 / 4, self.y + self.vy * t as i32 / 4)
    }
}

/// A leg of a course. When it's over the ship flies as its plan says.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash)]
pub(super) enum Leg {
    /// Holding a speed for `left` ticks more.
    Hold { dv: (i32, i32), left: u16 },
    /// Going round its target, `r` px from its middle, at speed `v`: clockwise as the screen
    /// shows it (`way` 1) or the other way (-1), for `left` ticks more.
    Round { r: i32, v: i32, way: i32, left: u16 },
}

impl Leg {
    /// The leg a tick later. None: it's over.
    fn on(self) -> Option<Leg> {
        match self {
            Leg::Hold { dv, left } if left > 1 => Some(Leg::Hold { dv, left: left - 1 }),
            Leg::Round { r, v, way, left } if left > 1 => Some(Leg::Round { r, v, way, left: left - 1 }),
            _ => None,
        }
    }

    /// The speed it asks of a ship at `at` whose target is at `target`. None: it can't be flown
    /// (there's no going round a target that's gone).
    fn dv(self, at: (i32, i32), target: Option<(i32, i32)>) -> Option<(i32, i32)> {
        match self {
            Leg::Hold { dv, .. } => Some(dv),
            Leg::Round { r, v, way, .. } => {
                let (cx, cy) = target?;
                let (dx, dy) = (at.0 - cx, at.1 - cy);
                let d = isqrt(dx * dx + dy * dy).max(1);
                // Along the circle, and in or out to it.
                let out = (r - d).clamp(-v, v);
                Some(((-dy * way * v + dx * out) / d, (dx * way * v + dy * out) / d))
            }
        }
    }
}

/// A course: the leg a ship is on. None: it flies as its plan says.
pub(super) type Course = Option<Leg>;

impl Arena {
    /// The plan a ship flies this tick under the rollout pilot: its own plan, or the speed of
    /// the course it has picked. Only a ship in a fight picks one (an enemy within `SIGHT`, or
    /// an enemy shot near): any other flies as its plan says.
    pub(super) fn on_course(&mut self, i: usize, mut plan: Plan, snap: &[Snap]) -> Plan {
        let (team, x, y, flies) = {
            let s = &self.ships[i];
            (s.team, s.x, s.y, s.st.push != [0; 4])
        };
        let fight = flies
            && (snap.iter().any(|e| e.team != team && cheb(e.x, e.y, x, y) <= SIGHT) || self.shots.iter().any(|sh| sh.team != team && cheb(sh.line.x, sh.line.y, x, y) <= SIGHT / 2));
        if !fight {
            self.ships[i].course = None;
            return plan;
        }
        if (self.tick + i as u32) % EVERY == 0 {
            self.ships[i].course = self.pick_course(i, &plan, snap);
        }
        if let Some(leg) = self.ships[i].course {
            let target = plan.target.and_then(|t| snap.iter().find(|e| e.idx == t)).map(|e| (e.x, e.y));
            match leg.dv((x, y), target) {
                Some(dv) => {
                    plan.dv = dv;
                    self.ships[i].course = leg.on();
                }
                None => self.ships[i].course = None,
            }
        }
        plan
    }

    /// The course ship `i` picks: see the top of this file.
    fn pick_course(&self, i: usize, plan: &Plan, snap: &[Snap]) -> Course {
        let s = &self.ships[i];
        let (st, team, half, full) = (s.st, s.team, s.half(), s.px_full());
        let me = Flyer::of(s);
        // The enemy ships near enough to matter, nearest first, a dozen at most.
        let mut near: Vec<&Snap> = snap.iter().filter(|e| e.team != team && cheb(e.x, e.y, s.x, s.y) <= SIGHT).collect();
        near.sort_by_key(|e| (dist2(e.x, e.y, s.x, s.y), e.idx));
        near.truncate(12);
        let threats: Vec<Threat> = near
            .iter()
            .map(|e| {
                let o = &self.ships[e.idx];
                let turrets = o.turrets.iter().filter(|t| o.hp[t.cell] > 0).map(|t| (t.aim, o.shape.cells[t.cell] == Some(Part::Tracker))).collect();
                let others = |reach: i32| 1 + snap.iter().filter(|f| f.team == team && f.idx != i && cheb(f.x, f.y, e.x, e.y) <= reach + f.half).count() as i32;
                let mut lanes: Vec<(i32, i32)> = Vec::new();
                for c in (0..o.hp.len()).filter(|&c| o.hp[c] > 0 && o.shape.cells[c] == Some(Part::Lance)) {
                    let (ox, oy) = o.shape.cell_xy(0, 0, o.facing, c);
                    let lane = if o.facing & 1 == 0 { ox } else { oy };
                    match lanes.iter_mut().find(|l| l.0 == lane) {
                        Some(l) => l.1 += 1,
                        None => lanes.push((lane, 1)),
                    }
                }
                Threat { x: e.x, y: e.y, vx: e.vx, vy: e.vy, half: e.half, full: e.full, facing: o.facing, flak: o.st.flak, lanes, turrets, open: self.sides.get(e.idx).map_or([u16::MAX; 2], |s| s.2), flak_for: others(FLAK_RANGE), turret_for: others(CANNON_RANGE) }
            })
            .collect();
        // The enemy shots in the air that could reach it in the time, each as the pixels it
        // flies through tick by tick. (A missile steers: it isn't a line to step off.)
        let shots: Vec<(i32, Vec<Vec<(i32, i32)>>)> = self
            .shots
            .iter()
            .filter(|sh| sh.team != team && sh.kind != Kind::Missile && cheb(sh.line.x, sh.line.y, s.x, s.y) <= SIGHT)
            .map(|sh| {
                let (mut line, mut left) = (sh.line, sh.left);
                let ticks = (0..AHEAD)
                    .map(|_| {
                        let mut px = Vec::new();
                        for _ in 0..sh.speed {
                            if left <= 0 {
                                break;
                            }
                            line.step();
                            left -= 1;
                            px.push((line.x, line.y));
                        }
                        px
                    })
                    .collect();
                (hurt(sh.kind), ticks)
            })
            .collect();
        // Its working lances: the ticks until each is loaded, where the gun sits from the
        // ship's middle for each way it could face, and its round (how hard and how fast).
        let lances: Vec<(i32, [(i32, i32); 4], i32, i32)> = (0..s.hp.len())
            .filter(|&c| s.hp[c] > 0 && s.shape.cells[c] == Some(Part::Lance))
            .map(|c| {
                let (pierce, speed) = rounds(LANCE_PIERCE, LANCE_SPEED, s.heavy_rounds(&[c]));
                (s.cool[c], [0u8, 1, 2, 3].map(|f| s.shape.cell_xy(0, 0, f, c)), pierce, speed)
            })
            .collect();
        let reload = st.reload(LANCE_PERIOD);
        // What its own flak pierces (a gun beside a piercer fires a heavier round).
        let flak_pierce = (0..s.hp.len()).filter(|&c| s.hp[c] > 0 && s.shape.cells[c] == Some(Part::Flak)).map(|c| rounds(FLAK_PIERCE, FLAK_SPEED, s.heavy_rounds(&[c])).0).max().unwrap_or(FLAK_PIERCE);
        // The enemy beams that are warming, and will cut while it's looking ahead.
        let beams: Vec<Warm> = self.warm.iter().filter(|w| w.team != team && w.left <= AHEAD as i32).copied().collect();
        // The target's standing pixels, as rows and as columns from its middle: a lance does
        // something only on a line the target still has a pixel in.
        let target = plan.target.and_then(|t| near.iter().position(|e| e.idx == t).map(|k| (k, &self.ships[t])));
        let span = target.map_or(0, |(_, t)| t.half() + 2);
        // For each way the ship could face, the lines (columns facing north or south, rows
        // facing east or west) down which its beam or its repeater would break something:
        // the first standing pixel of the target on the line, from the side the shot comes
        // in by, is one the gun out-pierces.
        let mut lines: [Vec<bool>; 4] = std::array::from_fn(|_| vec![false; 2 * span as usize + 1]);
        if let (Some((_, t)), Some(j)) = (target, plan.target) {
            let pierce = self.fwd_pierce(i);
            for f in 0..4u8 {
                let (ux, uy) = facing_vec(f);
                for k in -span..=span {
                    // The line's far end from the target's middle, on the side the shot comes from.
                    let from = if f & 1 == 0 { (t.x + k - ux * span, t.y - uy * span) } else { (t.x - ux * span, t.y + k - uy * span) };
                    let first = (0..=2 * span).find_map(|n| t.cell_at(from.0 + ux * n, from.1 + uy * n).filter(|&c| t.hp[c] > 0));
                    lines[f as usize][(k + span) as usize] = first.is_some_and(|c| t.shape.cells[c].map_or(0, |p| p.armor()) < pierce);
                }
            }
            let _ = j;
        }
        let reach = st.fwd_reach();
        // The ways it could go. As its plan says; the course it's on; standing still; each of
        // eight ways at its top speed and slowly, held; and a step each of four ways, short or
        // long, before carrying on as planned.
        // (A ship on its way to where it was sent keeps to its group's pace, as its plan does.)
        let pace = if plan.doing == Doing::Sent && s.pace > 0 { s.pace } else { i32::MAX };
        let paced = |dv: (i32, i32)| {
            let v = isqrt(dv.0 * dv.0 + dv.1 * dv.1);
            if v > pace { (dv.0 * pace / v, dv.1 * pace / v) } else { dv }
        };
        let (top, slow) = (st.vmax.min(pace), SLOW.min(pace));
        let hold = |dv: (i32, i32), left: u16| Some(Leg::Hold { dv, left });
        let mut ways: Vec<Course> = vec![None];
        ways.extend(s.course.map(Some));
        ways.push(hold((0, 0), AHEAD as u16));
        for (ux, uy) in [(1, 0), (1, 1), (0, 1), (-1, 1), (-1, 0), (-1, -1), (0, -1), (1, -1)] {
            for speed in [top, slow] {
                let k = if ux != 0 && uy != 0 { speed * 7 / 10 } else { speed };
                ways.push(hold((ux * k, uy * k), AHEAD as u16));
            }
        }
        for (ux, uy) in [(1, 0), (0, 1), (-1, 0), (0, -1)] {
            for ticks in [8u16, 20] {
                ways.push(hold((ux * top, uy * top), ticks));
            }
            ways.push(hold((ux * slow, uy * slow), 6));
        }
        // And going round its target at the range its plan wants, either way, fast or at half
        // its speed.
        let range = target.map(|(n, _)| {
            let th = &threats[n];
            plan.goal.map_or_else(|| isqrt(dist2(s.x, s.y, th.x, th.y)), |g| isqrt(dist2(g.0, g.1, th.x, th.y)))
        });
        if let Some(r) = range {
            for way in [1, -1] {
                for v in [st.vmax, st.vmax / 2] {
                    if v > 0 {
                        ways.push(Some(Leg::Round { r, v, way, left: AHEAD as u16 }));
                    }
                }
            }
        }
        let mut best: Option<(i32, usize)> = None;
        for (k, &way) in ways.iter().enumerate() {
            // Its own motion, flown that way by the plain pilot.
            let (mut f, mut leg) = (me, way);
            let mut path = [(0i32, 0i32, 0i32, 0i32, 0u8); AHEAD + 1];
            path[0] = (f.x, f.y, f.vx, f.vy, f.facing);
            for t in 0..AHEAD {
                let dv = leg.and_then(|l| l.dv((f.x, f.y), target.map(|(n, _)| threats[n].at(t)))).unwrap_or_else(|| plan.goal.map_or(plan.dv, |g| paced(f.arrive(&st, g))));
                leg = leg.and_then(Leg::on);
                // (A plan that has it face its target has it face where the target will be.)
                let face = match target {
                    Some((n, _)) if plan.face.is_some() => {
                        let (ex, ey) = threats[n].at(t);
                        axis_face(ex - f.x, ey - f.y)
                    }
                    _ => plan.face,
                };
                if let Some(turned) = f.steer(&st, dv, face) {
                    f.facing = turned;
                }
                f.drift(half, self.w, self.h);
                path[t + 1] = (f.x, f.y, f.vx, f.vy, f.facing);
            }
            let mut cost = if k > 0 && way == s.course { -KEEP } else { 0 };
            cost += TURN * (0..AHEAD).filter(|&t| path[t].4 != path[t + 1].4).count() as i32;
            // How far it is from where its plan wants it, over the whole way, and the speed it
            // still has at the end. A ship whose guns need no lining up is wanted at a range
            // from its target, on whatever side (the spot its plan names follows it round),
            // and every pixel off that range costs a pixel. Any other is wanted on a spot,
            // and has a leash round it.
            if let Some(g) = plan.goal {
                // What being out of place costs at tick `t`, in sixteenths of a pixel.
                let away = |t: usize| match (target, range) {
                    (Some((n, _)), Some(r)) if !st.aims() => {
                        let (ex, ey) = threats[n].at(t);
                        PX * (isqrt(dist2(path[t].0, path[t].1, ex, ey)) - r).abs()
                    }
                    _ => {
                        let d = isqrt(dist2(path[t].0, path[t].1, g.0, g.1));
                        OFF * d.min(LEASH) + PX * (d - LEASH).max(0)
                    }
                };
                let end = path[AHEAD];
                cost += (STEP..=AHEAD).step_by(STEP).map(away).sum::<i32>() * STEP as i32 / AHEAD as i32 + OFF * (end.2.abs() + end.3.abs());
            }
            // Shots in the air that would meet it.
            for (hurts, ticks) in &shots {
                if (0..AHEAD).any(|t| ticks[t].iter().any(|&(px, py)| cheb(px, py, path[t + 1].0, path[t + 1].1) <= half)) {
                    cost += hurts * PX;
                }
            }
            // An enemy beam that's warming cuts down its line when it's ready: whatever of the
            // ship is on the line then goes.
            for w in &beams {
                let at = path[w.left.max(0) as usize];
                let (rx, ry) = (at.0 - w.from.0, at.1 - w.from.1);
                let (along, across) = (rx * w.dir.0 + ry * w.dir.1, rx * w.dir.1 + ry * w.dir.0);
                if along > -half && along <= w.reach + half && across.abs() <= half + 1 {
                    cost += PX * (2 * half + 1).min(full);
                }
            }
            // Its own lances, by the gun's own rule (`Arena::met_at`): each fires the moment
            // it's loaded and its shot would meet a pixel of its target, if the target flies
            // on at the speed it has. A shot that meets a pixel it can break is worth what a
            // lance shot breaks; one that meets armour it can't is fired all the same, and
            // worth nothing. (So a ship that can't sit still on a line flies its lance across
            // it, and flies it to where the target is going.)
            if let (Some((n, tgt)), Some(j), false) = (target, plan.target, lances.is_empty()) {
                let th = &threats[n];
                let mut loaded: Vec<i32> = lances.iter().map(|l| l.0).collect();
                for t in 1..=AHEAD {
                    if loaded.iter().all(|&c| c > t as i32) {
                        continue;
                    }
                    let (x, y, _, _, facing) = path[t];
                    let (ex, ey) = th.at(t);
                    let (ux, uy) = facing_vec(facing);
                    let along = (ex - x) * ux + (ey - y) * uy;
                    if along <= 0 || along > LANCE_LIFE + th.half {
                        continue;
                    }
                    for (k, l) in lances.iter().enumerate() {
                        if loaded[k] > t as i32 {
                            continue;
                        }
                        // (Only when the target, led, is near the gun's line: this saves the looking.)
                        let g = (x + l.1[facing as usize].0, y + l.1[facing as usize].1);
                        let ticks = along / l.3;
                        let off_line = if facing & 1 == 0 { ex - g.0 + th.vx * ticks / 4 } else { ey - g.1 + th.vy * ticks / 4 };
                        if off_line.abs() > th.half + 4 {
                            continue;
                        }
                        if let Some(c) = self.met_at(j, (ex - tgt.x, ey - tgt.y), g, (ux, uy), l.3, LANCE_LIFE - 1, (th.vx, th.vy)) {
                            if tgt.shape.cells[c].map_or(0, |p| p.armor()) < l.2 {
                                cost -= hurt(Kind::Lance) * PX;
                            }
                            loaded[k] = t as i32 + reload;
                        }
                    }
                }
            }
            // Guns, tick by tick. Each enemy turret's barrel swings on as the way goes.
            let mut barrels: Vec<Vec<i32>> = threats.iter().map(|th| th.turrets.iter().map(|t| t.0).collect()).collect();
            for t in (0..AHEAD - 1).step_by(STEP) {
                let (x, y, vx, vy, facing) = path[t];
                // Where a shot fired now, at where it would be if it kept this speed, would
                // find it: so many ticks on, did it turn out to be within `miss` of there?
                let led = |ticks: i32, miss: i32| {
                    let aim = (x + vx * ticks / 4, y + vy * ticks / 4);
                    let then = path[(t + ticks.max(0) as usize).min(AHEAD)];
                    cheb(then.0, then.1, aim.0, aim.1) <= miss
                };
                let mut in_flak = false;
                for (n, th) in threats.iter().enumerate() {
                    let (ex, ey) = th.at(t);
                    let d = cheb(ex, ey, x, y);
                    if th.full > full && d < th.half + half {
                        cost += SOLID;
                    }
                    if th.flak > 0 && d <= FLAK_RANGE + half && led(flight_ticks(d, FLAK_SPEED) + 1, half) {
                        cost += th.flak * STEP as i32 * PX / FLAK_PERIOD / th.flak_for;
                    }
                    // A lance fires straight ahead, down the line it's on, when its shot would
                    // meet the ship if the ship kept this speed. The shot lands if the ship
                    // is still on that line when it gets there.
                    if !th.lanes.is_empty() {
                        let (ux, uy) = facing_vec(th.facing);
                        let along = (x - ex) * ux + (y - ey) * uy;
                        if along > 0 && along <= LANCE_RANGE + half {
                            let ticks = along / LANCE_SPEED;
                            let then = path[(t + ticks as usize).min(AHEAD)];
                            let (across, led, later) = if th.facing & 1 == 0 { (x - ex, vx * ticks / 4, then.0 - ex) } else { (y - ey, vy * ticks / 4, then.1 - ey) };
                            let on: i32 = th.lanes.iter().filter(|l| (across + led - l.0).abs() <= half && (later - l.0).abs() <= half).map(|l| l.1).sum();
                            cost += on * STEP as i32 * PX * hurt(Kind::Lance) / LANCE_PERIOD;
                        }
                    }
                    // A turret swings its barrel towards where the ship will be, and fires
                    // when it's on.
                    for (b, turret) in th.turrets.iter().enumerate() {
                        let (range, speed, swing, period, hurts) = if turret.1 {
                            (TRACKER_RANGE, TRACKER_SPEED, TRACKER_TURN * STEP as i32, TRACKER_PERIOD, hurt(Kind::Track))
                        } else {
                            (CANNON_RANGE, SHELL_SPEED, STEP as i32 / TURRET_TURN as i32, CANNON_PERIOD, hurt(Kind::Shell))
                        };
                        if d > range + half {
                            continue;
                        }
                        let ticks = flight_ticks(cheb(ex, ey, x, y), speed) + 1;
                        let want = angle_of(x + vx * ticks / 4 - ex, y + vy * ticks / 4 - ey);
                        let round = angle_diff(barrels[n][b], want);
                        barrels[n][b] = (barrels[n][b] + round.clamp(-swing, swing)).rem_euclid(64);
                        if angle_diff(barrels[n][b], want).abs() <= 1 && led(ticks, half + 1) {
                            cost += STEP as i32 * PX * hurts / period / th.turret_for;
                        }
                    }
                    // Its own guns count only from where they can break something: against
                    // armour they can't, that's from a side the armour leaves a way in by.
                    let side = (angle_of(x - ex, y - ey) / 4) as u32 & 15;
                    let can = |pierce: i32| pierce > 2 || th.open[(pierce.max(1) - 1) as usize] >> side & 1 == 1;
                    in_flak |= d <= FLAK_RANGE + th.half && can(flak_pierce);
                    // Its own turrets and missiles reach further.
                    if d <= CANNON_RANGE + th.half {
                        let tracker = if can(TRACKER_PIERCE) { st.tracker * PX / TRACKER_PERIOD } else { 0 };
                        cost -= (st.cannon * hurt(Kind::Shell) * PX / CANNON_PERIOD + tracker) * STEP as i32 / threats.len().max(1) as i32;
                    }
                }
                // Its own flak, with anything in reach.
                if in_flak {
                    cost -= st.flak * STEP as i32 * PX / FLAK_PERIOD;
                }
                // Its beam or its repeater, which fire straight ahead too: on its target, ahead
                // of it and in reach, counted as one line through the ship's middle.
                if let (Some((n, _)), true) = (target, reach > 0 && lances.is_empty()) {
                    let th = &threats[n];
                    let (ex, ey) = th.at(t);
                    let (ux, uy) = facing_vec(facing);
                    let along = (ex - x) * ux + (ey - y) * uy;
                    if along > 0 && along <= reach + th.half {
                        // (Led as the gun leads: a beam by the time it warms, a repeater by
                        // the time its shot takes.)
                        let lead = if st.beam > 0 { BEAM_WARM } else { along / REPEAT_SPEED };
                        let (mine, theirs) = if facing & 1 == 0 { (x, ex + th.vx * lead / 4) } else { (y, ey + th.vy * lead / 4) };
                        if lines[facing as usize].get((mine - theirs + span) as usize).copied().unwrap_or(false) {
                            cost -= STEP as i32 * PX * hurt(Kind::Lance) / LANCE_PERIOD;
                        }
                    }
                }
            }
            if best.map_or(true, |b| cost < b.0) {
                best = Some((cost, k));
            }
        }
        ways[best.map_or(0, |b| b.1)]
    }
}
