//! The human limits a bot plays under. A bot thinks every `poll` ticks (its reaction time) and
//! asks for gestures; the pilot carries them out one at a time at hand speed: moving the pointer
//! to a new spot, dragging rail one tile per `seg` ticks, erasing, clicking switches. Each new
//! plan replaces whatever the hand had not done yet. Express is a key, so it applies at once.

use crate::geom::*;
use crate::sim::Sim;
use std::collections::VecDeque;

#[derive(Clone, Debug)]
pub enum Gesture {
    /// A left-drag through these tiles, in order.
    Stroke(Vec<Pos>),
    /// A right-drag over these tiles.
    Erase(Vec<Pos>),
    Flip(Pos),
    Express(bool),
}

#[derive(Clone, Copy, Debug)]
pub struct Limits {
    pub poll: i32,
    pub seg: i32,
    pub erase: i32,
    pub flip: i32,
    /// Moving the pointer to a tile not next to where it is.
    pub reach: i32,
}

impl Default for Limits {
    /// 1/3 s reactions, 7.5 tiles a second when dragging, 1/5 s to move the pointer elsewhere.
    fn default() -> Self {
        Limits {
            poll: 5,
            seg: 2,
            erase: 1,
            flip: 3,
            reach: 3,
        }
    }
}

pub trait Bot {
    fn name(&self) -> &'static str;
    fn think(&mut self, sim: &Sim) -> Vec<Gesture>;
}

#[derive(Clone, Copy, Debug)]
enum Do {
    Reach(Pos),
    Seg(Option<Dir>, Pos, Pos),
    Erase(Pos),
    Flip(Pos),
}

/// One piece of handwork, tagged with the gesture it belongs to.
#[derive(Clone, Copy, Debug)]
struct Op {
    what: Do,
    gesture: u32,
}

pub struct Pilot {
    pub lim: Limits,
    ops: VecDeque<Op>,
    busy: i32,
    hand: Option<Pos>,
    /// The gesture the hand is in the middle of, if any.
    doing: Option<u32>,
    next_id: u32,
}

impl Pilot {
    pub fn new(lim: Limits) -> Pilot {
        Pilot {
            lim,
            ops: VecDeque::new(),
            busy: 0,
            hand: None,
            doing: None,
            next_id: 1,
        }
    }

    /// Let the bot think if it is time, then do the next bit of handwork. Call before `sim.step()`.
    pub fn tick(&mut self, sim: &mut Sim, bot: &mut dyn Bot) {
        if sim.tick % self.lim.poll == 0 {
            let g = bot.think(sim);
            self.plan(sim, g);
        }
        self.act(sim);
    }

    /// A new plan replaces everything not yet started; a gesture already under way is finished
    /// first, as a person finishes a drag before starting the next.
    fn plan(&mut self, sim: &mut Sim, gestures: Vec<Gesture>) {
        let doing = self.doing;
        self.ops.retain(|o| Some(o.gesture) == doing);
        let mut hand = self.ops.back().map_or(self.hand, |o| match o.what {
            Do::Reach(p) | Do::Erase(p) | Do::Flip(p) | Do::Seg(_, _, p) => Some(p),
        });
        for g in gestures {
            let id = self.next_id;
            self.next_id += 1;
            let push = |ops: &mut VecDeque<Op>, what: Do| ops.push_back(Op { what, gesture: id });
            let go = |ops: &mut VecDeque<Op>, hand: &mut Option<Pos>, p: Pos| {
                if !hand.is_some_and(|h| h.manhattan(p) <= 1) {
                    ops.push_back(Op {
                        what: Do::Reach(p),
                        gesture: id,
                    });
                }
                *hand = Some(p);
            };
            match g {
                Gesture::Express(on) => sim.set_express(on),
                Gesture::Stroke(path) => {
                    if path.len() < 2 {
                        continue;
                    }
                    go(&mut self.ops, &mut hand, path[0]);
                    for i in 1..path.len() {
                        let came = if i >= 2 {
                            path[i - 1].dir_to(path[i - 2])
                        } else {
                            None
                        };
                        push(&mut self.ops, Do::Seg(came, path[i - 1], path[i]));
                    }
                    hand = path.last().copied();
                }
                Gesture::Erase(tiles) => {
                    for t in tiles {
                        go(&mut self.ops, &mut hand, t);
                        push(&mut self.ops, Do::Erase(t));
                    }
                }
                Gesture::Flip(t) => {
                    go(&mut self.ops, &mut hand, t);
                    push(&mut self.ops, Do::Flip(t));
                }
            }
        }
    }

    /// Do the next piece of handwork. Something that changes nothing (dragging over rail that is
    /// already there) or that the rules refuse (no pieces left, a tile under the train) costs no
    /// time: the player can see it.
    fn act(&mut self, sim: &mut Sim) {
        while self.busy <= sim.tick {
            let Some(op) = self.ops.pop_front() else {
                self.doing = None;
                return;
            };
            self.doing = Some(op.gesture);
            let cost = match op.what {
                Do::Reach(p) => {
                    self.hand = Some(p);
                    self.lim.reach
                }
                Do::Seg(came, a, b) => {
                    let had = sim.mask[a.idx()] & a.dir_to(b).map_or(0, Dir::bit) != 0;
                    let lit = sim.lit[a.idx()];
                    if sim.draw_seg(came, a, b) {
                        self.hand = Some(b);
                        if had && lit == sim.lit[a.idx()] {
                            0
                        } else {
                            self.lim.seg
                        }
                    } else {
                        self.ops.retain(|o| o.gesture != op.gesture);
                        0
                    }
                }
                Do::Erase(t) => {
                    if sim.erase(t) {
                        self.hand = Some(t);
                        self.lim.erase
                    } else {
                        0
                    }
                }
                Do::Flip(t) => {
                    if sim.flip(t) {
                        self.hand = Some(t);
                        self.lim.flip
                    } else {
                        0
                    }
                }
            };
            self.busy = sim.tick + cost;
        }
    }
}

/// Play a whole leg.
pub fn play(sim: &mut Sim, bot: &mut dyn Bot, lim: Limits) {
    let mut pilot = Pilot::new(lim);
    while !sim.finished() {
        pilot.tick(sim, bot);
        sim.step();
    }
}
