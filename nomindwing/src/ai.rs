//! A computer king. It plays by the player's rules and cards, deciding once a second what to
//! print and steering its king every tick.

use crate::design::Kind;
use crate::geom::{cheb, dist2, step_toward};
use crate::world::{Cmd, World};

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Mode {
    Build,
    Attack,
    Retreat,
}

pub struct Ai {
    pub team: usize,
    pub mode: Mode,
    home: (i32, i32),
    next: u32,
}

impl Ai {
    pub fn new(team: usize, w: &World) -> Self {
        let k = w.king(team);
        Self { team, mode: Mode::Build, home: (k.x, k.y), next: 0 }
    }

    /// Queues this tick's cards and returns the king's steering.
    pub fn step(&mut self, w: &World, cmds: &mut Vec<Cmd>) -> (i32, i32) {
        let king = w.king(self.team);
        if !king.alive || w.winner.is_some() {
            return (0, 0);
        }
        let hp = king.hp_pct();
        if w.tick >= self.next {
            self.next = w.tick + 15;
            self.think(w, cmds, hp);
        }
        let (gx, gy) = self.goal(w, hp);
        let (dx, dy) = (gx - king.x, gy - king.y);
        (if dx.abs() > 2 { dx.signum() } else { 0 }, if dy.abs() > 2 { dy.signum() } else { 0 })
    }

    fn think(&mut self, w: &World, cmds: &mut Vec<Cmd>, hp: u32) {
        let me = self.team;
        let kid = w.teams[me].king;
        let king = &w.ships[kid];
        let pending = |k: Kind| king.queue.iter().filter(|j| j.kind == k).count() + king.job.iter().filter(|j| j.kind == k).count();
        let count = |k: Kind| w.ships.iter().filter(|s| s.alive && s.team == me && s.kind == k).count() + pending(k);
        let collectors = count(Kind::Collector);
        let escorts = w.ships.iter().filter(|s| s.alive && s.team == me && s.kind == Kind::Escort && s.parent == Some(kid)).count() + pending(Kind::Escort);
        let raiders = count(Kind::Raider);
        let outposts = count(Kind::Outpost) + count(Kind::OutpostSeed);
        let forwards = count(Kind::Forward) + count(Kind::ForwardSeed);
        let army = raiders + escorts;
        self.mode = match self.mode {
            Mode::Build if hp >= 75 && army >= 7 && w.tick > 15 * 200 => Mode::Attack,
            Mode::Attack if hp < 45 => Mode::Retreat,
            Mode::Attack if army < 3 => Mode::Build,
            Mode::Retreat if hp >= 85 => Mode::Build,
            m => m,
        };
        if king.queue.len() + king.job.is_some() as usize >= 2 {
            return;
        }
        let t = w.tick;
        let bank = w.teams[me].bank;
        // A slow opening: economy first, its forward base a little before two minutes, and no
        // raiders of its own before 1:40, so a new player has time to plant and look around.
        let pick = (|| {
            if collectors < 4 {
                return Some((Kind::Collector, None));
            }
            if t > 15 * 25 && outposts == 0 {
                if let Some(r) = self.outpost_rock(w) {
                    return Some((Kind::OutpostSeed, Some(r)));
                }
            }
            if escorts < 2 {
                return Some((Kind::Escort, None));
            }
            if collectors < 6 {
                return Some((Kind::Collector, None));
            }
            // Spare pixels become more factories: one more forward base per 400 PX banked.
            if t > 15 * 110 && forwards < (1 + bank / 400).min(4) as usize {
                if let Some(r) = self.forward_rock(w) {
                    return Some((Kind::ForwardSeed, Some(r)));
                }
            }
            if t < 15 * 100 {
                return if collectors < 8 {
                    Some((Kind::Collector, None))
                } else if escorts < 3 {
                    Some((Kind::Escort, None))
                } else {
                    None
                };
            }
            if escorts < 4 && bank >= 20 {
                return Some((Kind::Escort, None));
            }
            if bank >= 24 {
                return Some((Kind::Raider, None));
            }
            None
        })();
        if let Some((kind, dest)) = pick {
            cmds.push(Cmd::Queue { team: me, kind, dest });
        }
    }

    /// The nearest free rock on our own side of the map.
    fn outpost_rock(&self, w: &World) -> Option<usize> {
        let foe = w.king(1 - self.team);
        let (hx, hy) = self.home;
        (0..w.rocks.len())
            .filter(|&a| w.rock_free(a, self.team, None))
            .filter(|&a| {
                let r = &w.rocks[a];
                dist2(r.cx, r.cy, hx, hy) < dist2(r.cx, r.cy, foe.x, foe.y)
            })
            .min_by_key(|&a| dist2(w.rocks[a].cx, w.rocks[a].cy, hx, hy))
    }

    /// A free rock a little past halfway toward the enemy king.
    fn forward_rock(&self, w: &World) -> Option<usize> {
        let foe = w.king(1 - self.team);
        let (hx, hy) = self.home;
        let (px, py) = (hx + (foe.x - hx) * 55 / 100, hy + (foe.y - hy) * 55 / 100);
        (0..w.rocks.len()).filter(|&a| w.rock_free(a, self.team, None)).min_by_key(|&a| dist2(w.rocks[a].cx, w.rocks[a].cy, px, py))
    }

    fn goal(&self, w: &World, hp: u32) -> (i32, i32) {
        let me = self.team;
        let king = w.king(me);
        match self.mode {
            Mode::Retreat => self.home,
            Mode::Attack => {
                let t = w.ships.iter().filter(|s| s.alive && s.team != me && s.kind.is_factory()).min_by_key(|s| dist2(s.x, s.y, king.x, king.y));
                match t {
                    Some(s) => step_toward(s.x, s.y, king.x, king.y, 44),
                    None => self.home,
                }
            }
            Mode::Build => {
                if hp >= 60 {
                    for b in w.ships.iter().filter(|s| s.alive && s.team == me && s.kind.is_base()) {
                        if w.ships.iter().any(|e| e.alive && e.team != me && !e.kind.is_seed() && cheb(e.x, e.y, b.x, b.y) <= 100) {
                            return (b.x, b.y);
                        }
                    }
                }
                self.home
            }
        }
    }
}
