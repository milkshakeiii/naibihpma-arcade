//! The match, on top of the arena: kings, seeds that grow factories out of rocks pixel by pixel,
//! factories printing their build orders a pixel at a time, and the networks that pool what's
//! mined. None of it has its own state about rocks: a rock is just pixels, and a planted factory
//! is the claim.

use super::*;

/// The map: 1200 by 900 until 2026-11-11.
pub const MW: i32 = 1400;
pub const MH: i32 = 1050;
pub const KING_STARTS: [(i32, i32); 2] = [(150, MH - 150), (MW - 1 - 150, 149)];
pub const START_BANK: i32 = 190;
/// The king prints ships the way a factory does, as if it had this many factory pixels: a little
/// slower than the starter deck's outpost (20), and with no limit on size.
pub const KING_FACTORY: i32 = 16;
/// The king makes a pixel by itself every this many ticks: a floor under the economy, so a side
/// that has lost every miner can still print one.
pub const KING_MAKES_EVERY: u32 = 60;
/// The king flies like Toy 1's: straight from the keys, at this many quarter-pixels a tick,
/// which is 37.5 px/s. (45 until 2026-11-11: as fast as the fastest ship a drawing can make.)
pub const KING_SPEED: i32 = 10;
/// A king flies at that speed only while its side has this many pixels of ships and factories
/// standing: everything of its team but the king itself. With less, the side is beaten, and its
/// king drops to `KING_SLOW` so that whatever is chasing it can end the match. At full speed a
/// king keeps pace with a fighter (and outruns one on a diagonal), and one with nothing left to
/// lose could run until the clock called a draw.
pub const KING_NEEDS: i32 = 40;
/// A beaten king's speed, in quarter-pixels a tick: half a pixel, a fifth of its full speed. The
/// slowest ship that flies at all makes 3 (`Design::stats`), so anything with an engine runs a
/// beaten king down, even one fleeing on a diagonal.
pub const KING_SLOW: i32 = 2;
/// The rule waits this long from the start, when nobody has had time to build anything.
pub const KING_GRACE: u32 = 15 * 60;
/// Pixels a second a seed eats into its factory, before the factory pixels it has grown join in.
/// 6 until 2026-11-11.
pub const GROW_RATE: i32 = 4;
pub const WARP_TICKS: u32 = 30;
/// How far a node reaches: two of a team's nodes link when their reaches overlap.
pub const NODE_RANGE: i32 = 80;
pub const KING_RANGE: i32 = 120;
/// Most loose pixels a destroyed node spills.
pub const SPILL_CAP: i32 = 300;
/// A match still going after this long is a draw.
pub const MATCH_LIMIT: u32 = 15 * 60 * 40;
pub const KING_SLOT: usize = 100;
pub const SEED_SLOT: usize = 101;
/// Ships of a design count as one group once they're waiting this close to one of them.
pub const MUSTER: i32 = 96;
/// A wave needs this many pixels of warships, and is called home after this long.
const WAVE_MIN: i32 = 100;
const WAVE_TICKS: u32 = 15 * 240;
/// An enemy warship this near a bot's king or anything it has planted is a raid. A fleet turned
/// out for one stays out until every raider is `ALARM_CLEAR` further off than that: a raider
/// hovering at the edge would otherwise turn it out and home again every few seconds.
const ALARM_RANGE: i32 = 200;
const ALARM_CLEAR: i32 = 60;
/// How near the rushing opening's king goes to what it attacks, middle to middle.
const RUSH_CLOSE: i32 = 40;
/// The clear pixels the AI's king leaves between itself and the factory it posts itself beside.
const STRONGPOINT_GAP: i32 = 4;
/// Accumulators count in 60ths of a pixel per tick: a factory pixel adds 1 a tick, so it prints a
/// pixel every 4 s.
const UNIT: i32 = PRINT_TICKS;

#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash)]
pub enum Role {
    Ship,
    King,
    Seed,
}

/// What a factory is printing: nothing, the next thing on its list (no pixel paid yet), or the
/// ship it's partway through.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash)]
pub enum Making {
    None,
    Pending(usize),
    Child(usize),
}

/// Something a side tells its king to do. Commands are the only way input reaches a match, the
/// AI's included: they're queued and applied at the start of the next tick, and every one applied
/// is logged with its tick. The same commands on the same ticks replay the same match, which is
/// what lockstep multiplayer and replays need.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash)]
pub enum Cmd {
    /// The king's keys: -1, 0 or 1 on each axis, held until the next one.
    Steer { team: usize, dx: i32, dy: i32 },
    /// The king prints a deck slot. A seeded design is paid for in full on the spot, and its seed
    /// flies to the rock pixel `target`.
    Print { team: usize, slot: usize, target: Option<(i32, i32)> },
    /// A design's standing order: every ship of it, flying or still to be printed, takes it.
    Order { team: usize, slot: usize, order: Order },
    /// The king keeps printing a deck slot, or stops. A repeating job goes back on the end of the
    /// queue each time it's finished. With no place given (the card was picked): if a job of the
    /// design is repeating already, every job of it stops repeating; otherwise a new repeating
    /// job of it joins the end of the queue. With a place `at` (a job in the queue was picked,
    /// found as for `Cancel`): that job repeats, or stops.
    Repeat { team: usize, slot: usize, at: Option<usize> },
    /// The king drops a job from its queue and gets back what it had paid: the job `at` places
    /// down the queue (0 is the one it's printing), which was of the design in `slot`. The queue
    /// may have moved up by the time the command is applied, so the job dropped is the last one
    /// of that design at or before `at`; if there's none, nothing is.
    Cancel { team: usize, at: usize, slot: usize },
}

/// Orders by their number on the wire. New ones go on the end.
const WIRE_ORDERS: [Order; 6] = [Order::Hunt, Order::Guard, Order::Escort, Order::Harvest, Order::Hold, Order::Sab];

/// A place past the end of any queue. As the `at` of a `Repeat` or a `Cancel` it means the last
/// job of that design, wherever it is when the command is applied: the way to name a job that
/// was only just asked for and, with commands waiting out a lag, isn't in the queue yet.
pub const LAST_JOB: usize = 99_999;

impl Cmd {
    /// Whose command it is.
    pub fn team(&self) -> usize {
        match *self {
            Cmd::Steer { team, .. } | Cmd::Print { team, .. } | Cmd::Order { team, .. } | Cmd::Repeat { team, .. } | Cmd::Cancel { team, .. } => team,
        }
    }

    /// A command as a few words of text, to send to another machine. `from_wire` reads it back.
    pub fn to_wire(&self) -> String {
        match *self {
            Cmd::Steer { team, dx, dy } => format!("S {team} {dx} {dy}"),
            Cmd::Print { team, slot, target: None } => format!("P {team} {slot}"),
            Cmd::Print { team, slot, target: Some((x, y)) } => format!("P {team} {slot} {x} {y}"),
            Cmd::Order { team, slot, order } => format!("O {team} {slot} {}", WIRE_ORDERS.iter().position(|&o| o == order).unwrap_or(0)),
            Cmd::Repeat { team, slot, at: None } => format!("R {team} {slot}"),
            Cmd::Repeat { team, slot, at: Some(at) } => format!("R {team} {slot} {at}"),
            Cmd::Cancel { team, at, slot } => format!("C {team} {at} {slot}"),
        }
    }

    /// Reads a command another machine sent. Anything that isn't exactly a command is refused:
    /// what arrives over a wire is never trusted to index into the match.
    pub fn from_wire(s: &str) -> Option<Cmd> {
        let mut words = s.split(' ');
        let kind = words.next()?;
        let nums: Vec<i64> = words.map(|w| w.parse::<i64>()).collect::<Result<_, _>>().ok()?;
        let n = |k: usize, max: i64| nums.get(k).copied().filter(|v| (0..=max).contains(v)).map(|v| v as usize);
        let far = |k: usize| nums.get(k).copied().filter(|v| (-100_000..=100_000).contains(v)).map(|v| v as i32);
        let team = n(0, 1)?;
        Some(match (kind, nums.len()) {
            ("S", 3) => Cmd::Steer { team, dx: far(1)?.signum(), dy: far(2)?.signum() },
            ("P", 2) => Cmd::Print { team, slot: n(1, 63)?, target: None },
            ("P", 4) => Cmd::Print { team, slot: n(1, 63)?, target: Some((far(2)?, far(3)?)) },
            ("O", 3) => Cmd::Order { team, slot: n(1, 63)?, order: WIRE_ORDERS[n(2, 5)?] },
            ("R", 2) => Cmd::Repeat { team, slot: n(1, 63)?, at: None },
            ("R", 3) => Cmd::Repeat { team, slot: n(1, 63)?, at: Some(n(2, 100_000)?) },
            ("C", 3) => Cmd::Cancel { team, at: n(1, 100_000)?, slot: n(2, 63)? },
            _ => return None,
        })
    }
}

/// Who plays a side: a person (through the window's commands), the AI, or one of the scripted
/// openings the AI is measured against. They all play by commands, like a person.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash)]
pub enum Bot {
    None,
    /// The opponent: plants when it can pay, keeps miners working, masses its warships on GUARD
    /// and sends them out in waves, and has its king print what counters the enemy's fleet. It
    /// takes an enemy king on its half of the map for a raid, arms first against a raid or a
    /// fleet it can't match, and stands its king beside its factory while a raid is on.
    Ai,
    /// Round 1's opponent: one more factory every 75 s by the clock. Everything on HUNT.
    Clock,
    /// Plants the cheapest factory every time the bank allows. Everything on HUNT.
    Planter,
    /// The AI's economy with everything on HUNT: ships go out as they're made.
    Expander,
    /// One factory, and a king that prints warships without a pause. Everything on HUNT.
    Summoner,
    /// The boom that won Henry's third playtest: the cheapest factory every time the bank
    /// allows, and a king that prints miners without a pause. Everything on HUNT. It's what the
    /// pace of a match is measured with (`--pace`).
    Boomer,
    /// The rush that won Henry's match of 2026-11-11 in 4:42: one factory at once, and a king
    /// that prints its cheapest lance or cannon ship without a pause. Everything on HUNT. Its
    /// king goes too: up to the nearest thing the enemy has planted, and prints there. Once it
    /// has lost a quarter of itself it goes home for good, and prints from there.
    Rusher,
    /// The sabotage that won the match after it in 9:10: the AI's economy with everything on
    /// SAB, so ships go for miners and factories as they're made.
    Saboteur,
}

impl Bot {
    pub fn from_name(s: &str) -> Option<Bot> {
        Some(match s.to_ascii_lowercase().as_str() {
            "ai" => Bot::Ai,
            "clock" => Bot::Clock,
            "planter" => Bot::Planter,
            "expander" => Bot::Expander,
            "summoner" => Bot::Summoner,
            "boomer" => Bot::Boomer,
            "rusher" => Bot::Rusher,
            "saboteur" => Bot::Saboteur,
            _ => return None,
        })
    }
}

/// What a bot remembers between thoughts.
#[derive(Clone, Debug, Default, Hash)]
struct Mind {
    next: u32,
    /// A wave is out: when it was called, what the fleet was worth then, and the tick each design
    /// is let go (the slow ones first, so the wave arrives together).
    wave: bool,
    wave_from: u32,
    wave_worth: i32,
    release: Vec<u32>,
    /// When the last wave came home.
    rested: u32,
    last_plant: u32,
    big_at: u32,
    turn: usize,
    /// Its fleet is turned out against a raid.
    alarm: bool,
    /// The rushing opening's king has been hurt enough, and gone home.
    gone_home: bool,
}

/// What a warship is to the counter triangle: small with flak, small with a lance or cannon, or
/// big.
fn class_of(st: &Stats) -> usize {
    if st.count > 20 {
        2
    } else if st.lance + st.cannon + st.beam > 0 {
        1
    } else {
        0
    }
}

/// Whoever is printing this tick: a king working through its queue, or a factory (by ship index).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Printer {
    King(usize),
    Factory(usize),
}

/// What growing the next pixel of a ship would do: grow this pending cell, wait for an enemy to
/// move out of the way, or nothing (it's as grown as it can get).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(super) enum Next {
    Cell(usize),
    Blocked,
    Done,
}

#[derive(Clone, Debug, Hash)]
pub struct Grow {
    /// Cells still to grow, in growth order (outward from the core).
    pub pending: Vec<usize>,
    /// Rock pixels left to eat, nearest last. Empty for a ship being printed.
    food: Vec<(i32, i32)>,
    /// Growing out of a rock (a planted design) rather than being printed.
    pub eats: bool,
    acc: i32,
}

/// Something the king is printing: a deck slot, and for a seeded design the rock pixel its seed
/// flies to.
#[derive(Clone, Copy, Debug, Hash)]
pub struct Job {
    pub slot: usize,
    pub target: Option<(i32, i32)>,
    pub paid: i32,
    pub cost: i32,
    /// Printed again each time it's finished.
    pub repeat: bool,
}

#[derive(Clone, Copy, Debug, Default, Hash)]
pub struct Tally {
    pub planted: i32,
    pub printed: i32,
    pub summoned: i32,
    pub bank_peak: i32,
    /// Times a design's standing order was changed.
    pub orders: i32,
}

/// A rock as generated: its middle and its pixels (some may since have been eaten or mined).
#[derive(Clone, Debug, Hash)]
pub struct Rock {
    pub cx: i32,
    pub cy: i32,
    pub px: Vec<(i32, i32)>,
}

/// What planting a seeded design on a rock pixel would link, as things stand (see
/// `Arena::plant_links`).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Links {
    /// Where the middle of what grows would be.
    pub at: (i32, i32),
    /// The middle of every node of the side's that it would link to directly: the ones
    /// standing, and the ones its seeds already on their way will grow.
    pub to: Vec<(i32, i32)>,
    /// Whether those links lead to its king.
    pub king: bool,
    /// With nothing in reach: the nearest node, and how many pixels too far it is.
    pub gap: Option<((i32, i32), i32)>,
}

/// What the match needs to know about each deck slot, worked out once.
#[derive(Clone, Copy, Debug, Hash)]
pub struct Card {
    pub ok: bool,
    pub count: i32,
    pub cost: i32,
    pub seeded: bool,
    pub st: Stats,
    /// The order it was drawn with: the one it starts the match on.
    pub order: Order,
    /// How many of it gather before they leave on HUNT or SAB.
    pub group: u8,
}

#[derive(Hash)]
pub struct Eco {
    pub deck: [Vec<Design>; 2],
    pub cards: [Vec<Card>; 2],
    pub king: [usize; 2],
    pub queue: [Vec<Job>; 2],
    /// Each design's standing order: what ships of it do, and what new ones are printed with.
    pub orders: [Vec<Order>; 2],
    king_acc: [i32; 2],
    /// The king's keys this tick: -1, 0 or 1 on each axis.
    steer: [(i32, i32); 2],
    /// Commands waiting for the next tick, and every command applied so far with its tick.
    inbox: Vec<Cmd>,
    pub log: Vec<(u32, Cmd)>,
    pub bot: [Bot; 2],
    mind: [Mind; 2],
    /// Network id per ship (usize::MAX: not a node).
    pub net: Vec<usize>,
    /// The tick each team's network last broke in two (0: never): two of its nodes parted, or
    /// its king was cut off by the loss of a node. A king that flies out of reach by itself
    /// hasn't broken anything.
    pub split_at: [u32; 2],
    pub tally: [Tally; 2],
    pub rocks: Vec<Rock>,
    /// Pixels each side has taken out of its banks to print with, and pixels lost with destroyed
    /// nodes (spilled or not). With what's mined and what's stored, these must always add up.
    pub spent: [i32; 2],
    pub spilt: [i32; 2],
    /// Pixels each king has made by itself.
    pub made: [i32; 2],
    /// Pixels handed out to printers so far: who's first in line moves round with it.
    turn: usize,
}

impl Card {
    pub fn of(d: &Design) -> Card {
        let st = d.stats();
        Card { ok: d.check().ok(), count: st.count, cost: st.cost, seeded: d.seeded, st, order: d.order, group: d.group.max(1) }
    }
    /// What the king pays to print it. A ship is paid for a pixel at a time as it's printed; a
    /// seeded design is paid for in full, everything it will grow into, before its seed sets off.
    pub fn price(&self) -> i32 {
        self.cost
    }
    /// A ship that mines.
    pub fn miner(&self) -> bool {
        self.ok && !self.seeded && self.st.scoops > 0 && self.order == Order::Harvest
    }
    /// A ship that flies and fights.
    pub fn warship(&self) -> bool {
        self.ok && !self.seeded && self.st.armed() && self.st.thrust > 0 && !self.miner() && self.order != Order::Hold
    }
    /// A seeded design that prints.
    pub fn factory(&self) -> bool {
        self.ok && self.seeded && self.st.factory > 0
    }
    /// Whether its standing order can be switched in a match: anything that flies.
    pub fn takes_orders(&self) -> bool {
        self.ok && self.st.thrust > 0
    }
}

impl Arena {
    /// A match on a mirrored map: each side starts with its king and a bank, nothing else.
    pub fn for_match(seed: u64, deck: [Vec<Design>; 2], ai: [bool; 2]) -> Arena {
        Arena::for_bots(seed, deck, [if ai[0] { Bot::Ai } else { Bot::None }, if ai[1] { Bot::Ai } else { Bot::None }])
    }

    /// The same, saying who plays each side.
    pub fn for_bots(seed: u64, deck: [Vec<Design>; 2], bot: [Bot; 2]) -> Arena {
        let mut a = Arena::blank(MW, MH, seed);
        let rocks = a.match_rocks();
        for _ in 0..900 {
            let (x, y) = (a.rng.range(0, MW - 1), a.rng.range(0, MH - 1));
            let c = if a.rng.chance(15) { 2 } else if a.rng.chance(40) { 1 } else { 0 };
            a.stars.push((x, y, c));
        }
        let k = king();
        let mut kings = [0; 2];
        for t in 0..2 {
            let (x, y) = KING_STARTS[t];
            let i = a.spawn(t, KING_SLOT, &k, x, y, if t == 0 { 0 } else { 2 }, false);
            let s = &mut a.ships[i];
            s.role = Role::King;
            s.store = START_BANK;
            s.node = true;
            a.score[t].placed += 1;
            kings[t] = i;
        }
        let cards = [deck[0].iter().map(Card::of).collect(), deck[1].iter().map(Card::of).collect()];
        let orders = [deck[0].iter().map(|d| d.order).collect(), deck[1].iter().map(|d| d.order).collect()];
        a.eco = Some(Eco {
            deck,
            cards,
            king: kings,
            queue: [Vec::new(), Vec::new()],
            orders,
            king_acc: [0; 2],
            steer: [(0, 0); 2],
            inbox: Vec::new(),
            log: Vec::new(),
            bot,
            mind: [Mind::default(), Mind::default()],
            net: Vec::new(),
            split_at: [0; 2],
            tally: [Tally::default(); 2],
            rocks,
            spent: [0; 2],
            spilt: [0; 2],
            made: [0; 2],
            turn: 0,
        });
        a
    }

    fn eco(&self) -> &Eco {
        self.eco.as_ref().expect("a match")
    }
    fn eco_mut(&mut self) -> &mut Eco {
        self.eco.as_mut().expect("a match")
    }

    // ----- The map -----

    /// Rocks in mirrored pairs (every rock has a twin through the centre, so neither king is ever
    /// a pixel closer to anything), one giant in the middle, and two home rocks by each king.
    fn match_rocks(&mut self) -> Vec<Rock> {
        let (w, h) = (self.w, self.h);
        let mut rocks = Vec::new();
        let mut placed: Vec<(i32, i32, i32)> = Vec::new();
        // The giant: point-symmetric itself, so it's its own twin.
        let (cx, cy, r) = (w / 2, h / 2, 30);
        let half: Vec<(i32, i32)> = self.blob(cx, cy, r).into_iter().filter(|&(x, y)| y < cy || (y == cy && x <= cx)).collect();
        let mut giant = half.clone();
        giant.extend(half.iter().map(|&(x, y)| (w - 1 - x, h - 1 - y)));
        giant.sort();
        giant.dedup();
        self.put_rock(giant, &mut rocks);
        placed.push((cx, cy, r));
        let (kx, ky) = KING_STARTS[0];
        let mut pairs: Vec<(i32, i32, i32)> = vec![(kx + 125, ky + 35, 12), (kx - 15, ky - 150, 9)];
        // Two big contested pairs, then the rest.
        let mut tries = 0;
        let mut want: Vec<(i32, i32)> = vec![(20, 24), (20, 24)];
        // Twenty-two small pairs: as thick on the ground as sixteen were on the 1200 by 900 map.
        want.extend(std::iter::repeat((6, 16)).take(22));
        for (lo, hi) in want {
            loop {
                tries += 1;
                if tries > 20000 {
                    break;
                }
                let r = self.rng.range(lo, hi);
                let (x, y) = (self.rng.range(r + 14, w - r - 15), self.rng.range(r + 14, h - r - 15));
                let (mx, my) = (w - 1 - x, h - 1 - y);
                let clear = |px: i32, py: i32, all: &Vec<(i32, i32, i32)>| {
                    KING_STARTS.iter().all(|&(a, b)| dist2(px, py, a, b) >= (r + 90) * (r + 90)) && all.iter().all(|&(a, b, r2)| dist2(px, py, a, b) >= (r + r2 + 26) * (r + r2 + 26))
                };
                let mut all = placed.clone();
                all.extend(pairs.iter().copied());
                if clear(x, y, &all) && clear(mx, my, &all) && dist2(x, y, mx, my) >= (2 * r + 26) * (2 * r + 26) {
                    pairs.push((x, y, r));
                    break;
                }
            }
        }
        for (x, y, r) in pairs {
            let px = self.blob(x, y, r);
            let twin: Vec<(i32, i32)> = px.iter().map(|&(a, b)| (w - 1 - a, h - 1 - b)).collect();
            self.put_rock(px, &mut rocks);
            self.put_rock(twin, &mut rocks);
            placed.push((x, y, r));
        }
        rocks
    }

    /// A rough disc of rock: a ragged edge, solid inside.
    fn blob(&mut self, cx: i32, cy: i32, r: i32) -> Vec<(i32, i32)> {
        let mut px = Vec::new();
        for y in -r..=r {
            for x in -r..=r {
                let d = dist2(x, y, 0, 0);
                let edge = d > (r - 2) * (r - 2);
                if d <= r * r && !(edge && self.rng.chance(35)) {
                    px.push((cx + x, cy + y));
                }
            }
        }
        px
    }

    fn put_rock(&mut self, px: Vec<(i32, i32)>, rocks: &mut Vec<Rock>) {
        let px: Vec<(i32, i32)> = px.into_iter().filter(|&(x, y)| (0..self.w).contains(&x) && (0..self.h).contains(&y)).collect();
        if px.is_empty() {
            return;
        }
        for &(x, y) in &px {
            self.rock[(y * self.w + x) as usize] = true;
            self.rock_px.push((x, y));
        }
        let n = px.len() as i32;
        let (cx, cy) = (px.iter().map(|p| p.0).sum::<i32>() / n, px.iter().map(|p| p.1).sum::<i32>() / n);
        rocks.push(Rock { cx, cy, px });
    }

    /// The pixels of a rock still standing.
    pub fn rock_left(&self, r: &Rock) -> i32 {
        r.px.iter().filter(|&&(x, y)| self.rock_at(x, y)).count() as i32
    }

    /// The rock pixel nearest a point, within `reach` (a square search).
    pub fn rock_near(&self, (x, y): (i32, i32), reach: i32) -> Option<(i32, i32)> {
        let mut best: Option<(i32, (i32, i32))> = None;
        for py in y - reach..=y + reach {
            for px in x - reach..=x + reach {
                if self.rock_at(px, py) {
                    let d = dist2(px, py, x, y);
                    if best.map_or(true, |(b, _)| d < b) {
                        best = Some((d, (px, py)));
                    }
                }
            }
        }
        best.map(|(_, p)| p)
    }

    /// The rock pixel nearest a point that no enemy pixel is sitting on, within `reach` if given.
    pub fn free_rock(&self, (x, y): (i32, i32), team: usize, reach: Option<i32>) -> Option<(i32, i32)> {
        let mut best: Option<((i32, i32, i32), (i32, i32))> = None;
        for &(px, py) in &self.rock_px {
            if !self.rock_at(px, py) || reach.is_some_and(|r| cheb(px, py, x, y) > r) {
                continue;
            }
            let key = (dist2(px, py, x, y), py, px);
            if best.map_or(true, |(b, _)| key < b) && self.room_to_grow(team, None, px, py) {
                best = Some((key, (px, py)));
            }
        }
        best.map(|(_, p)| p)
    }

    /// Every pixel of the rock touching `start`, edge to edge.
    pub fn rock_blob(&self, start: (i32, i32)) -> Vec<(i32, i32)> {
        let mut seen = std::collections::HashSet::new();
        let mut todo = vec![start];
        let mut out = Vec::new();
        seen.insert(start);
        while let Some((x, y)) = todo.pop() {
            out.push((x, y));
            for (dx, dy) in [(1, 0), (-1, 0), (0, 1), (0, -1)] {
                let n = (x + dx, y + dy);
                if self.rock_at(n.0, n.1) && seen.insert(n) {
                    todo.push(n);
                }
            }
        }
        out
    }

    // ----- The tick -----

    /// Groups set off. Ships of a design that leaves in groups wait at their posts, guarding,
    /// until as many as its group size are waiting within `MUSTER` px of one of them. Those go,
    /// all of them, and anything that joins later waits for a group of its own.
    fn muster(&mut self) {
        for team in 0..2 {
            for slot in 0..self.eco().cards[team].len() {
                let size = self.eco().cards[team][slot].group as usize;
                let waiting: Vec<usize> = (0..self.ships.len())
                    .filter(|&i| {
                        let s = &self.ships[i];
                        s.alive && s.waiting && s.team == team && s.slot == slot && s.role == Role::Ship && matches!(s.order, Order::Hunt | Order::Sab) && s.warp == 0 && s.grow.is_none() && s.maker.is_none()
                    })
                    .collect();
                for &a in &waiting {
                    if !self.ships[a].waiting {
                        continue;
                    }
                    let (ax, ay) = (self.ships[a].x, self.ships[a].y);
                    let near: Vec<usize> = waiting.iter().copied().filter(|&b| self.ships[b].waiting && cheb(self.ships[b].x, self.ships[b].y, ax, ay) <= MUSTER).collect();
                    if near.len() >= size {
                        for b in near {
                            self.ships[b].waiting = false;
                        }
                    }
                }
            }
        }
    }

    pub(super) fn eco_pre(&mut self) {
        for t in 0..2 {
            if self.eco().bot[t] != Bot::None && self.tick >= self.eco().mind[t].next {
                self.bot_think(t);
                self.eco_mut().mind[t].next = self.tick + 15;
            }
            self.apply_commands();
            self.steer_king(t);
        }
        // After this tick's orders, so a group already gathered leaves on the tick it's told to.
        self.muster();
    }

    /// Queues a command for the next tick. A steer that changes nothing is dropped.
    pub fn command(&mut self, c: Cmd) {
        let e = self.eco_mut();
        if let Cmd::Steer { team, dx, dy } = c {
            let last = e.inbox.iter().rev().find_map(|q| match *q {
                Cmd::Steer { team: t, dx, dy } if t == team => Some((dx, dy)),
                _ => None,
            });
            if last.unwrap_or(e.steer[team]) == (dx.signum(), dy.signum()) {
                return;
            }
        }
        e.inbox.push(c);
    }

    /// Applies everything queued, in order, and logs it against this tick.
    fn apply_commands(&mut self) {
        let cmds = std::mem::take(&mut self.eco_mut().inbox);
        let tick = self.tick;
        for c in cmds {
            match c {
                Cmd::Steer { team, dx, dy } => self.eco_mut().steer[team] = (dx.signum(), dy.signum()),
                Cmd::Print { team, slot, target } => {
                    // One that's no longer allowed (it was checked when it was given) is dropped.
                    let Ok(cost) = self.can_print(team, slot, target) else { continue };
                    if self.eco().cards[team][slot].seeded {
                        // A seed is paid for in full, now, and sets off at once.
                        let k = self.eco().king[team];
                        for _ in 0..cost {
                            let paid = self.take_px(k);
                            debug_assert!(paid, "the bank was counted before the seed was sold");
                        }
                        self.deliver(team, Job { slot, target, paid: cost, cost, repeat: false });
                    } else {
                        self.eco_mut().queue[team].push(Job { slot, target, paid: 0, cost, repeat: false });
                    }
                }
                Cmd::Order { team, slot, order } => {
                    if self.eco().orders[team].get(slot).map_or(true, |&o| o == order) {
                        continue;
                    }
                    let e = self.eco_mut();
                    let out = |o: Order| matches!(o, Order::Hunt | Order::Sab);
                    // Sent out from guarding or escorting, they gather first and leave as a group.
                    // Switched between HUNT and SAB, the ones already out stay out.
                    let gather = e.cards[team][slot].group > 1 && !(out(e.orders[team][slot]) && out(order));
                    e.orders[team][slot] = order;
                    e.tally[team].orders += 1;
                    for s in self.ships.iter_mut().filter(|s| s.alive && s.team == team && s.slot == slot && s.role == Role::Ship) {
                        s.order = order;
                        s.waiting |= gather;
                    }
                }
                Cmd::Repeat { team, slot, at: Some(at) } => {
                    // That job in the queue, wherever the queue has moved it to since.
                    let q = &mut self.eco_mut().queue[team];
                    let Some(at) = (0..q.len().min(at.saturating_add(1))).rev().find(|&i| q[i].slot == slot) else { continue };
                    q[at].repeat = !q[at].repeat;
                }
                Cmd::Repeat { team, slot, at: None } => {
                    if self.eco().queue[team].iter().any(|j| j.slot == slot && j.repeat) {
                        for j in self.eco_mut().queue[team].iter_mut().filter(|j| j.slot == slot) {
                            j.repeat = false;
                        }
                    } else {
                        // A new job of it on the end of the queue, on repeat: whatever was asked
                        // for before it comes first, one-off jobs of the same design included.
                        match self.can_print(team, slot, None) {
                            Ok(cost) if !self.eco().cards[team][slot].seeded => self.eco_mut().queue[team].push(Job { slot, target: None, paid: 0, cost, repeat: true }),
                            _ => continue,
                        }
                    }
                }
                Cmd::Cancel { team, at, slot } => {
                    let k = self.eco().king[team];
                    let q = &self.eco().queue[team];
                    let Some(at) = (0..q.len().min(at.saturating_add(1))).rev().find(|&i| q[i].slot == slot) else { continue };
                    if !self.ships[k].alive {
                        continue;
                    }
                    let job = self.eco_mut().queue[team].remove(at);
                    self.ships[k].store += job.paid;
                    let e = self.eco_mut();
                    e.spent[team] -= job.paid;
                    if at == 0 {
                        e.king_acc[team] = 0;
                    }
                }
            }
            self.eco_mut().log.push((tick, c));
        }
    }

    pub(super) fn eco_post(&mut self) {
        self.networks();
        self.spills();
        // Anything delivered or printed this tick is dealt with from next tick on.
        let n = self.ships.len();
        // Everyone with something to print: kings with a queue, factories with a build order.
        let mut printers: Vec<Printer> = Vec::new();
        for t in 0..2 {
            let k = self.eco().king[t];
            if self.ships[k].alive && self.tick % KING_MAKES_EVERY == 0 {
                self.ships[k].store += 1;
                self.eco_mut().made[t] += 1;
            }
            if self.ships[k].alive && !self.eco().queue[t].is_empty() {
                self.eco_mut().king_acc[t] += KING_FACTORY;
                printers.push(Printer::King(t));
            }
        }
        for i in 0..n {
            if !self.ships[i].alive {
                continue;
            }
            if self.ships[i].warp > 0 {
                // It can't come out of hyperspace into another ship's pixels: it waits, still
                // beside its king, until its spot is clear.
                if self.ships[i].warp > 1 || !self.in_enemy(i) {
                    self.ships[i].warp -= 1;
                }
                continue;
            }
            // A half-printed ship whose factory died flies on as it is.
            if let Some(m) = self.ships[i].maker {
                if !self.ships[m].alive {
                    let s = &mut self.ships[i];
                    s.maker = None;
                    s.grow = None;
                    s.home = (s.x, s.y);
                }
                continue;
            }
            if self.ships[i].role == Role::Seed {
                self.seed_step(i);
            } else if self.ships[i].grow.as_ref().is_some_and(|g| g.eats) {
                // Its factory pixels grow it first. Wedged against an enemy with nowhere to grow,
                // they print instead.
                if self.growth_step(i) && self.factory_ready(i) {
                    printers.push(Printer::Factory(i));
                }
            } else if self.factory_ready(i) {
                printers.push(Printer::Factory(i));
            }
        }
        self.take_turns(printers);
        for t in 0..2 {
            let k = self.eco().king[t];
            let bank = self.bank(k);
            let tl = &mut self.eco_mut().tally[t];
            tl.bank_peak = tl.bank_peak.max(bank);
        }
        let k = self.eco().king;
        let dead = [!self.ships[k[0]].alive, !self.ships[k[1]].alive];
        if dead[0] || dead[1] {
            self.over = Some(match dead {
                [true, true] => None,
                [true, false] => Some(1),
                _ => Some(0),
            });
        } else if self.tick >= MATCH_LIMIT {
            self.over = Some(None);
        }
    }

    /// The king flies straight from its keys, like Toy 1's. It moves and turns by the same rules as
    /// every other ship: it can't fly through a bigger enemy, or through anything planted.
    fn steer_king(&mut self, t: usize) {
        let (k, (dx, dy)) = (self.eco().king[t], self.eco().steer[t]);
        if !self.ships[k].alive {
            return;
        }
        let (dx, dy) = (dx.signum(), dy.signum());
        let slowed = self.king_slowed(t);
        let pace = if slowed { KING_SLOW } else { KING_SPEED };
        let (mx, my) = {
            // So many quarter-pixels a tick, through the remainders every other ship moves by,
            // and kept the way they keep them (0 to 3): a piece shot off the king takes its
            // remainders with it, and must not drift on them.
            let s = &mut self.ships[k];
            s.sx = if dx == 0 { 0 } else { s.sx + dx * pace };
            s.sy = if dy == 0 { 0 } else { s.sy + dy * pace };
            let m = (s.sx.div_euclid(4), s.sy.div_euclid(4));
            s.sx = s.sx.rem_euclid(4);
            s.sy = s.sy.rem_euclid(4);
            m
        };
        let (x0, y0, h) = (self.ships[k].x, self.ships[k].y, self.ships[k].half());
        let nx = (x0 + mx).clamp(h, self.w - 1 - h);
        let ny = (y0 + my).clamp(h, self.h - 1 - h);
        let (x, y) = self.shift(k, nx, ny);
        // Its speed, in the quarter-pixels the rest of the sim uses, so guns lead it properly:
        // its pace, over the ticks it moves a pixel less as well, unless something stopped it.
        let s = &mut self.ships[k];
        s.vx = if mx != 0 && x == x0 { 0 } else { dx * pace };
        s.vy = if my != 0 && y == y0 { 0 } else { dy * pace };
        // A king answers its keys at once: what it's flying at is what it's trying to.
        s.want = (s.vx, s.vy);
        if let Some(f) = axis_face(dx, dy) {
            if f != self.ships[k].facing {
                self.try_turn(k, f);
            }
        }
    }

    /// Pixels of a side's ships and factories standing: everything of its team but its king.
    pub fn standing(&self, t: usize) -> i32 {
        self.ships.iter().filter(|s| s.alive && s.team == t && s.role != Role::King).map(|s| s.px_now()).sum()
    }

    /// A beaten side's king: after the first minute, with next to nothing of its side standing.
    pub fn king_slowed(&self, t: usize) -> bool {
        self.eco.is_some() && self.tick >= KING_GRACE && self.standing(t) < KING_NEEDS
    }

    // ----- Networks and banks -----

    fn reach(&self, i: usize) -> i32 {
        if self.ships[i].role == Role::King {
            KING_RANGE
        } else {
            NODE_RANGE
        }
    }

    pub fn is_node(&self, i: usize) -> bool {
        let s = &self.ships[i];
        s.alive && s.node && s.warp == 0 && s.maker.is_none()
    }

    /// Links every pair of a team's nodes whose reaches overlap, and notes when a network that was
    /// one piece last tick is broken in two now. A king that has only flown out of reach of its
    /// own nodes hasn't broken anything, and can fly back: that isn't noted. A king that leaves
    /// two nodes apart which were linked through it is, and so is one cut off by a node lost.
    fn networks(&mut self) {
        let n = self.ships.len();
        let ids: Vec<usize> = (0..n).filter(|&i| self.is_node(i)).collect();
        let mut parent: Vec<usize> = (0..n).collect();
        fn find(p: &mut [usize], mut x: usize) -> usize {
            while p[x] != x {
                p[x] = p[p[x]];
                x = p[x];
            }
            x
        }
        for a in 0..ids.len() {
            for b in a + 1..ids.len() {
                let (i, j) = (ids[a], ids[b]);
                let (si, sj) = (&self.ships[i], &self.ships[j]);
                let r = self.reach(i) + self.reach(j);
                if si.team == sj.team && dist2(si.x, si.y, sj.x, sj.y) <= r * r {
                    let (ri, rj) = (find(&mut parent, i), find(&mut parent, j));
                    parent[ri] = rj;
                }
            }
        }
        let mut net = vec![usize::MAX; n];
        for &i in &ids {
            net[i] = find(&mut parent, i);
        }
        let old = &self.eco().net;
        // Whether a side has a node fewer than it had last tick.
        let mut lost = [false; 2];
        for i in 0..n.min(old.len()) {
            if old[i] != usize::MAX && net[i] == usize::MAX {
                lost[self.ships[i].team] = true;
            }
        }
        let mut split = [false; 2];
        for a in 0..ids.len() {
            for b in a + 1..ids.len() {
                let (i, j) = (ids[a], ids[b]);
                let was = old.get(i).copied().unwrap_or(usize::MAX);
                let team = self.ships[i].team;
                let parted = team == self.ships[j].team && was != usize::MAX && old.get(j) == Some(&was) && net[i] != net[j];
                let flew_off = (self.ships[i].role == Role::King || self.ships[j].role == Role::King) && !lost[team];
                if parted && !flew_off {
                    split[team] = true;
                }
            }
        }
        let tick = self.tick;
        let e = self.eco_mut();
        for t in 0..2 {
            if split[t] {
                e.split_at[t] = tick;
            }
        }
        e.net = net;
    }

    /// Everything stored in the network a ship belongs to (just its own store if it isn't a node).
    pub fn bank(&self, i: usize) -> i32 {
        match self.eco().net.get(i).copied() {
            Some(id) if id != usize::MAX => (0..self.ships.len()).filter(|&j| self.eco().net.get(j) == Some(&id)).map(|j| self.ships[j].store).sum(),
            _ => self.ships[i].store,
        }
    }

    /// Takes a pixel for printing: from the printer's own store first, then from the nearest node
    /// in its network that has one.
    fn take_px(&mut self, i: usize) -> bool {
        let team = self.ships[i].team;
        if self.ships[i].store > 0 {
            self.ships[i].store -= 1;
            self.eco_mut().spent[team] += 1;
            return true;
        }
        let id = match self.eco().net.get(i).copied() {
            Some(id) if id != usize::MAX => id,
            _ => return false,
        };
        let (x, y) = (self.ships[i].x, self.ships[i].y);
        let from = (0..self.ships.len()).filter(|&j| self.eco().net.get(j) == Some(&id) && self.ships[j].store > 0).min_by_key(|&j| dist2(self.ships[j].x, self.ships[j].y, x, y));
        match from {
            Some(j) => {
                self.ships[j].store -= 1;
                self.eco_mut().spent[team] += 1;
                true
            }
            None => false,
        }
    }

    /// A destroyed node spills what it held as loose pixels for anyone to scoop.
    fn spills(&mut self) {
        for i in 0..self.ships.len() {
            if self.ships[i].alive || self.ships[i].store == 0 {
                continue;
            }
            let (x, y, h) = (self.ships[i].x, self.ships[i].y, self.ships[i].half().max(2));
            let n = self.ships[i].store.min(SPILL_CAP);
            let team = self.ships[i].team;
            self.eco_mut().spilt[team] += self.ships[i].store;
            self.ships[i].store = 0;
            for _ in 0..n {
                let (dx, dy) = (self.rng.range(-h, h), self.rng.range(-h, h));
                let drift = 2 + self.rng.below(6) as i32;
                self.debris.push(Debris { x: (x + dx).clamp(0, self.w - 1), y: (y + dy).clamp(0, self.h - 1), vx: dx.signum(), vy: dy.signum(), drift, taken: false });
            }
        }
    }

    // ----- Growing and printing -----

    /// The next pixel a growing or half-printed ship would grow: the first pending cell that touches
    /// one already there (the core, to start) with room where it would go (see `room_to_grow`).
    /// Looked up before anything is eaten or paid for, so nothing is spent on a pixel that can't be
    /// placed.
    pub(super) fn next_cell(&self, i: usize) -> Next {
        let s = &self.ships[i];
        let Some(g) = s.grow.as_ref() else { return Next::Done };
        let (w, h) = (s.shape.w, s.shape.h);
        let started = s.hp.iter().any(|&p| p > 0);
        let mut blocked = false;
        for (k, &c) in g.pending.iter().enumerate() {
            let touches = if started {
                let (x, y) = (c as i32 % w, c as i32 / w);
                [(1, 0), (-1, 0), (0, 1), (0, -1)].iter().any(|&(dx, dy)| {
                    let (nx, ny) = (x + dx, y + dy);
                    nx >= 0 && ny >= 0 && nx < w && ny < h && s.hp[(ny * w + nx) as usize] > 0
                })
            } else {
                s.shape.is_core(c)
            };
            if !touches {
                continue;
            }
            // A factory growing out of a rock needs room; a ship being printed only needs no enemy
            // there (its own side's things are under it, and it flies off when it's done).
            let (x, y) = s.cell_xy(c);
            let room = if g.eats { self.room_to_grow(s.team, Some(i), x, y) } else { !self.enemy_pixel(s.team, x, y) };
            if !room {
                blocked = true;
                continue;
            }
            return Next::Cell(k);
        }
        if blocked {
            Next::Blocked
        } else {
            Next::Done
        }
    }

    fn grow_cell(&mut self, i: usize, k: usize) {
        if cfg!(test) {
            let s = &self.ships[i];
            let g = s.grow.as_ref().unwrap();
            let (x, y) = s.cell_xy(g.pending[k]);
            assert!(!self.enemy_pixel(s.team, x, y), "{} grew a pixel onto an enemy pixel at ({x},{y})", s.name);
            assert!(!g.eats || !self.own_standing_pixel(s.team, Some(i), x, y), "{} grew a pixel into a factory of its own side at ({x},{y})", s.name);
        }
        let s = &mut self.ships[i];
        let c = s.grow.as_mut().unwrap().pending.remove(k);
        s.hp[c] = 1;
        s.dirty = true;
    }

    fn pending(&self, i: usize) -> usize {
        self.ships[i].grow.as_ref().map_or(0, |g| g.pending.len())
    }

    /// A factory growing out of its rock: the seed eats at a steady rate and every factory pixel
    /// grown so far helps, one rock pixel for each pixel grown. It stops when it's whole or the
    /// rock runs out, and waits while an enemy sits where its next pixels would go (returning true
    /// for a tick it spent waiting).
    fn growth_step(&mut self, i: usize) -> bool {
        let f = self.ships[i].st.factory;
        let mut food = {
            let g = self.ships[i].grow.as_mut().unwrap();
            g.acc += GROW_RATE * UNIT / 15 + f;
            std::mem::take(&mut g.food)
        };
        let (mut done, mut waiting) = (false, false);
        while self.ships[i].grow.as_ref().is_some_and(|g| g.acc >= UNIT) {
            // The next rock pixel to eat (any mined away since it landed are skipped).
            while food.last().is_some_and(|&(x, y)| !self.rock_at(x, y)) {
                food.pop();
            }
            let Some(&(x, y)) = food.last() else {
                done = true;
                break;
            };
            match self.next_cell(i) {
                Next::Cell(k) => {
                    food.pop();
                    self.rock[(y * self.w + x) as usize] = false;
                    self.grow_cell(i, k);
                    self.ships[i].grow.as_mut().unwrap().acc -= UNIT;
                    if self.pending(i) == 0 {
                        done = true;
                        break;
                    }
                }
                Next::Blocked => {
                    self.ships[i].grow.as_mut().unwrap().acc = UNIT;
                    waiting = true;
                    break;
                }
                Next::Done => {
                    done = true;
                    break;
                }
            }
        }
        if done {
            self.ships[i].grow = None;
        } else if let Some(g) = self.ships[i].grow.as_mut() {
            g.food = food;
        }
        waiting
    }

    /// Where something printed beside `m` goes: just outside it, on the side facing the enemy king,
    /// facing that way.
    fn beside(&self, m: usize, sh: &Shape) -> (i32, i32, u8) {
        let s = &self.ships[m];
        let ek = &self.ships[self.eco().king[1 - s.team]];
        let f = axis_face(ek.x - s.x, ek.y - s.y).unwrap_or(1);
        let (w, h) = sh.dims(f);
        let (l, t, r, b) = s.bbox();
        let (x, y) = match f {
            1 => (r + 3 + w / 2, s.y),
            3 => (l - 3 - (w - 1 - w / 2), s.y),
            2 => (s.x, b + 3 + h / 2),
            _ => (s.x, t - 3 - (h - 1 - h / 2)),
        };
        (x, y, f)
    }

    /// A factory (or carrier) works through its build order: the next thing it's big enough for.
    /// This settles what it's printing and gives it this tick's printing; false if it has nothing
    /// it can build.
    fn factory_ready(&mut self, i: usize) -> bool {
        if self.ships[i].st.factory == 0 || self.ships[i].build.is_empty() {
            return false;
        }
        let team = self.ships[i].team;
        if let Making::Child(c) = self.ships[i].making {
            if !self.ships[c].alive {
                // Shot while printing: start it again.
                self.ships[i].making = Making::None;
            }
        }
        if self.ships[i].making == Making::None {
            let s = &self.ships[i];
            let cards = &self.eco().cards[team];
            let n = s.build.len();
            let max = s.st.max_build();
            let fits = |slot: usize| cards.get(slot).is_some_and(|c| c.ok && !c.seeded && c.count <= max);
            let Some(k) = (0..n).map(|k| (s.build_i + k) % n).find(|&k| fits(s.build[k])) else { return false };
            let slot = s.build[k];
            self.ships[i].build_i = k;
            self.ships[i].making = Making::Pending(slot);
        }
        let f = self.ships[i].st.factory;
        self.ships[i].print_acc += f;
        true
    }

    /// Everyone printing takes turns, a pixel each, until each has printed what its rate allows or
    /// can't get a pixel. Who goes first moves round with every pixel handed out, so when the
    /// bank is short it's shared out evenly, not handed to whoever was built first. (It used to
    /// move round with the clock, and pixels that arrive on a steady beat, like the king's own,
    /// then always found the same printer first in line.)
    fn take_turns(&mut self, mut printers: Vec<Printer>) {
        if printers.is_empty() {
            return;
        }
        let first = self.eco().turn % printers.len();
        printers.rotate_left(first);
        let spent = |a: &Arena| (a.eco().spent[0] + a.eco().spent[1]) as usize;
        let before = spent(self);
        while !printers.is_empty() {
            let mut again = Vec::new();
            for p in printers {
                if self.print_turn(p) {
                    again.push(p);
                }
            }
            printers = again;
        }
        let handed = spent(self) - before;
        self.eco_mut().turn += handed;
    }

    /// One printer's turn: one pixel, paid for from its network as it goes down. True if it printed
    /// and is due another this tick. Short of pixels, or with an enemy where the pixel would go, it
    /// waits.
    fn print_turn(&mut self, p: Printer) -> bool {
        match p {
            Printer::King(t) => {
                let k = self.eco().king[t];
                if self.eco().king_acc[t] < UNIT || self.eco().queue[t].is_empty() {
                    return false;
                }
                if !self.take_px(k) {
                    self.eco_mut().king_acc[t] = UNIT;
                    return false;
                }
                let e = self.eco_mut();
                e.king_acc[t] -= UNIT;
                e.queue[t][0].paid += 1;
                if e.queue[t][0].paid >= e.queue[t][0].cost {
                    let job = e.queue[t].remove(0);
                    if job.repeat {
                        e.queue[t].push(Job { paid: 0, ..job });
                    }
                    self.deliver(t, job);
                    return false;
                }
                self.eco().king_acc[t] >= UNIT
            }
            Printer::Factory(i) => {
                if self.ships[i].print_acc < UNIT {
                    return false;
                }
                // Is there a pixel to print? Look first, so nothing is paid for one that can't go down.
                let making = self.ships[i].making;
                let next = match making {
                    Making::Child(c) => self.next_cell(c),
                    Making::Pending(slot) if self.print_spot_clear(i, slot) => Next::Cell(0),
                    Making::Pending(_) => Next::Blocked,
                    Making::None => return false,
                };
                match (next, making) {
                    (Next::Cell(k), _) => {
                        if !self.take_px(i) {
                            self.ships[i].print_acc = UNIT;
                            return false;
                        }
                        self.ships[i].print_acc -= UNIT;
                        match making {
                            Making::Pending(slot) => self.start_print(i, slot),
                            Making::Child(c) => {
                                self.grow_cell(c, k);
                                if self.pending(c) == 0 {
                                    self.launch(i, c);
                                }
                            }
                            Making::None => {}
                        }
                        self.ships[i].making != Making::None && self.ships[i].print_acc >= UNIT
                    }
                    // An enemy sits where the next pixel goes: wait.
                    (Next::Blocked, _) => {
                        self.ships[i].print_acc = UNIT;
                        false
                    }
                    // Nothing more of it can grow (pieces were shot off): it launches as it is.
                    (Next::Done, Making::Child(c)) => {
                        self.launch(i, c);
                        false
                    }
                    (Next::Done, _) => false,
                }
            }
        }
    }

    /// Whether the first pixel of the next ship (its core) has a free spot beside the factory.
    fn print_spot_clear(&self, i: usize, slot: usize) -> bool {
        let team = self.ships[i].team;
        let d = &self.eco().deck[team][slot];
        let sh = d.shape();
        let (x, y, f) = self.beside(i, &sh);
        let (cx, cy) = sh.cell_xy(x, y, f, d.start_cell().unwrap_or(0));
        !self.enemy_pixel(team, cx, cy)
    }

    /// Puts down the first pixel of the next ship on the list: its core, beside the factory.
    fn start_print(&mut self, i: usize, slot: usize) {
        let team = self.ships[i].team;
        let d = self.eco().deck[team][slot].clone();
        let sh = d.shape();
        let (x, y, f) = self.beside(i, &sh);
        let c = self.spawn(team, slot, &d, x, y, f, true);
        self.ships[c].order = self.eco().orders[team][slot];
        let start = d.start_cell().unwrap_or(0);
        self.ships[c].grow = Some(Grow { pending: Design::growth_order(&sh, start), food: Vec::new(), eats: false, acc: 0 });
        self.ships[c].maker = Some(i);
        if let Some(k) = self.ships[c].grow.as_ref().unwrap().pending.iter().position(|&p| p == start) {
            self.grow_cell(c, k);
        }
        let off = (self.ships[c].x - self.ships[i].x, self.ships[c].y - self.ships[i].y);
        self.ships[i].make_off = off;
        self.ships[i].making = Making::Child(c);
        if self.pending(c) == 0 {
            self.launch(i, c);
        }
    }

    /// A ship being printed stays beside its factory. Carriers move, so this runs with the rest of
    /// the movement, before shots are resolved: shots then see it move like any other ship.
    pub(super) fn eco_follow(&mut self) {
        // A ship still arriving comes out beside its king, wherever the king has got to.
        for i in 0..self.ships.len() {
            if self.ships[i].alive && self.ships[i].warp > 0 {
                let k = self.eco().king[self.ships[i].team];
                if self.ships[k].alive {
                    let (x, y, f) = self.beside(k, &self.ships[i].shape);
                    self.carry(i, x, y, Some(f));
                    // It was built here, beside its king. If ships built this near already keep
                    // posts, it joins their rings instead of starting its own on top of them.
                    let (at, team) = ((self.ships[k].x, self.ships[k].y), self.ships[i].team);
                    let kept = self
                        .ships
                        .iter()
                        .enumerate()
                        .filter(|&(j, o)| j != i && o.alive && o.team == team && o.role == Role::Ship && o.ring0 > 0 && o.warp == 0 && o.grow.is_none() && o.st.thrust > 0)
                        .filter(|(_, o)| cheb(o.home.0, o.home.1, at.0, at.1) <= POST_JOIN)
                        .min_by_key(|(_, o)| dist2(o.home.0, o.home.1, at.0, at.1))
                        .map(|(_, o)| (o.home, o.ring0));
                    let (home, ring0) = kept.unwrap_or((at, self.ships[k].half() + POST_GAP));
                    let s = &mut self.ships[i];
                    s.home = home;
                    s.ring0 = ring0;
                    s.pose0 = (s.x, s.y, f);
                }
            }
        }
        for i in 0..self.ships.len() {
            if let (true, Making::Child(c)) = (self.ships[i].alive, self.ships[i].making) {
                if self.ships[c].alive {
                    let (mx, my, off) = (self.ships[i].x, self.ships[i].y, self.ships[i].make_off);
                    self.carry(c, mx + off.0, my + off.1, None);
                }
            }
        }
    }

    /// A printed ship is finished: it's on its own now.
    fn launch(&mut self, i: usize, c: usize) {
        // It was built here: its home is its factory, and its post is in the rings round it.
        let (home, ring0) = ((self.ships[i].x, self.ships[i].y), self.ships[i].half() + POST_GAP);
        let s = &mut self.ships[c];
        s.grow = None;
        s.maker = None;
        s.home = home;
        s.ring0 = ring0;
        let team = s.team;
        let f = &mut self.ships[i];
        f.making = Making::None;
        f.build_i = (f.build_i + 1) % f.build.len().max(1);
        self.score[team].placed += 1;
        self.eco_mut().tally[team].printed += 1;
    }

    /// What a king finishes printing arrives by hyperspace beside it.
    fn deliver(&mut self, t: usize, job: Job) {
        let k = self.eco().king[t];
        let seeded = self.eco().cards[t][job.slot].seeded;
        let d = if seeded { seed() } else { self.eco().deck[t][job.slot].clone() };
        let (x, y, f) = self.beside(k, &d.shape());
        let i = self.spawn(t, if seeded { SEED_SLOT } else { job.slot }, &d, x, y, f, false);
        let order = self.eco().orders[t][job.slot];
        let s = &mut self.ships[i];
        s.warp = WARP_TICKS;
        if seeded {
            s.role = Role::Seed;
            s.seed_for = Some((job.slot, job.target.unwrap_or((x, y))));
        } else {
            s.order = order;
        }
        self.score[t].placed += 1;
        if !seeded {
            self.eco_mut().tally[t].summoned += 1;
        }
    }

    /// Whether a king may print a deck slot, and what it costs. A seeded design needs the rock pixel
    /// its seed should fly to.
    pub fn can_print(&self, t: usize, slot: usize, target: Option<(i32, i32)>) -> Result<i32, &'static str> {
        let card = *self.eco().cards.get(t).and_then(|c| c.get(slot)).ok_or("NO SUCH DESIGN")?;
        if !card.ok {
            return Err("THAT DESIGN CAN'T FLY YET");
        }
        if card.seeded && target.is_none() {
            return Err("A SEED NEEDS A ROCK");
        }
        if card.seeded && self.bank(self.eco().king[t]) < card.price() {
            return Err("NOT ENOUGH IN THE BANK: A SEED IS PAID FOR IN FULL");
        }
        Ok(card.price())
    }

    // ----- Seeds -----

    /// Where a seeded design comes down when its seed lands by the rock pixel `near`: the free
    /// rock pixel its core takes (the nearest within 10 px), and the middle of the design.
    fn landing(&self, team: usize, slot: usize, near: (i32, i32)) -> Option<((i32, i32), (i32, i32))> {
        let start = self.free_rock(near, team, Some(10))?;
        let d = self.eco().deck[team].get(slot)?;
        let (ox, oy) = d.shape().cell_xy(0, 0, 0, d.start_cell()?);
        Some((start, (start.0 - ox, start.1 - oy)))
    }

    /// What a seeded design would link to if its seed were sent to the rock pixel `target` now:
    /// what the match screen shows while you pick a rock. None for a design that links nothing
    /// (no factory pixels), or with no free rock there to land on. Seeds already on their way
    /// count as the nodes they'll grow, so a chain can be planted one after another.
    pub fn plant_links(&self, team: usize, slot: usize, target: (i32, i32)) -> Option<Links> {
        let e = self.eco();
        let links = |slot: usize| e.cards[team].get(slot).is_some_and(|c| c.st.factory > 0);
        if !links(slot) {
            return None;
        }
        let (_, at) = self.landing(team, slot, target)?;
        // Everything of the side's that links: x, y, reach, and whether it's the king.
        let mut sites: Vec<(i32, i32, i32, bool)> =
            (0..self.ships.len()).filter(|&i| self.is_node(i) && self.ships[i].team == team).map(|i| (self.ships[i].x, self.ships[i].y, self.reach(i), self.ships[i].role == Role::King)).collect();
        for s in self.ships.iter().filter(|s| s.alive && s.team == team && s.role == Role::Seed) {
            if let Some((_, (x, y))) = s.seed_for.filter(|f| links(f.0)).and_then(|(sl, t)| self.landing(team, sl, t)) {
                sites.push((x, y, NODE_RANGE, false));
            }
        }
        let touch = |a: (i32, i32, i32), b: &(i32, i32, i32, bool)| dist2(a.0, a.1, b.0, b.1) <= (a.2 + b.2) * (a.2 + b.2);
        let me = (at.0, at.1, NODE_RANGE);
        let to: Vec<(i32, i32)> = sites.iter().filter(|s| touch(me, s)).map(|s| (s.0, s.1)).collect();
        // Out from the new factory through everything that links, to see whether the king is there.
        let (mut seen, mut todo, mut king) = (vec![false; sites.len()], vec![me], false);
        while let Some(from) = todo.pop() {
            for (k, s) in sites.iter().enumerate() {
                if !seen[k] && touch(from, s) {
                    seen[k] = true;
                    king |= s.3;
                    todo.push((s.0, s.1, s.2));
                }
            }
        }
        // Nothing in reach: the nearest node by how far short the two reaches fall, rounded up.
        let short = |s: &(i32, i32, i32, bool)| isqrt(dist2(at.0, at.1, s.0, s.1) - 1) + 1 - (NODE_RANGE + s.2);
        let gap = if to.is_empty() { sites.iter().min_by_key(|s| short(s)).map(|s| ((s.0, s.1), short(s))) } else { None };
        Some(Links { at, to, king, gap })
    }

    pub(super) fn seed_plan(&mut self, i: usize) -> Plan {
        let s = &self.ships[i];
        match s.seed_for {
            Some((_, target)) => self.go(i, target, Doing::ToRock),
            None => self.go(i, (s.x, s.y), Doing::Idle),
        }
    }

    /// A seed that has reached its rock comes down on it and starts growing its design out of it,
    /// from the seed's own core outward.
    fn seed_step(&mut self, i: usize) {
        let Some((slot, target)) = self.ships[i].seed_for else { return };
        let (x, y, team) = (self.ships[i].x, self.ships[i].y, self.ships[i].team);
        // It comes down when it reaches its rock pixel, or when it's run up against a bigger enemy
        // and the pixel it was after is gone or under an enemy (so it can never get there).
        let there = cheb(x, y, target.0, target.1) <= 2;
        let shut_out = self.ships[i].bumped && (!self.rock_at(target.0, target.1) || self.enemy_pixel(team, target.0, target.1));
        if !there && !shut_out {
            return;
        }
        // It lands on the rock pixel it was sent to, or the nearest to that with room (`landing`,
        // which is also what the match screen showed when the rock was picked). Shut out, it
        // lands by where it is.
        let Some((start, mid)) = self.landing(team, slot, if there { target } else { (x, y) }) else {
            // None in reach (its rock was mined or eaten away, or is covered): it heads for the
            // nearest free rock left anywhere, or waits where it is if there's none.
            if there {
                self.ships[i].seed_for = self.free_rock((x, y), team, None).map(|p| (slot, p));
            }
            return;
        };
        let mut food = self.rock_blob(start);
        food.sort_by_key(|&(px, py)| std::cmp::Reverse((dist2(px, py, start.0, start.1), py, px)));
        let d = self.eco().deck[team][slot].clone();
        let sh = d.shape();
        let Some(c0) = d.start_cell() else { return };
        let f = self.spawn(team, slot, &d, mid.0, mid.1, 0, true);
        self.ships[f].order = self.eco().orders[team][slot];
        self.ships[f].rooted = true;
        self.ships[f].grow = Some(Grow { pending: Design::growth_order(&sh, c0), food, eats: true, acc: 0 });
        // The seed's own core is the factory's first pixel.
        if let Some(k) = self.ships[f].grow.as_ref().unwrap().pending.iter().position(|&p| p == c0) {
            self.grow_cell(f, k);
        }
        self.ships[i].alive = false;
        self.score[team].placed += 1;
        self.eco_mut().tally[team].planted += 1;
    }

    // ----- The bots -----

    /// A bot's thought, once a second. Everything it does is a command, the same ones a person
    /// gives: steer, print, plant, change a design's standing order.
    fn bot_think(&mut self, t: usize) {
        let bot = self.eco().bot[t];
        let k = self.eco().king[t];
        if !self.ships[k].alive {
            return;
        }
        self.bot_steer(t);
        let tick = self.tick;
        let cards = self.eco().cards[t].clone();
        let mut left = self.bank(k);
        let mut mind = self.eco().mind[t].clone();
        let is = |s: &Ship, f: fn(&Card) -> bool| cards.get(s.slot).is_some_and(f);
        // What it has.
        let own = |s: &&Ship| s.alive && s.team == t && s.role == Role::Ship;
        let factories = self.ships.iter().filter(|s| own(s) && is(s, Card::factory)).count();
        let seeds = self.ships.iter().filter(|s| s.alive && s.team == t && s.role == Role::Seed).count();
        let theirs = self.ships.iter().filter(|s| s.alive && s.team != t && s.role == Role::Ship && s.rooted && s.st.factory > 0).count();
        let queued_miners = self.eco().queue[t].iter().filter(|j| cards[j.slot].miner()).count();
        let miners = self.ships.iter().filter(|s| own(s) && is(s, Card::miner)).count() + queued_miners;
        let fleet: Vec<(usize, i32)> = self.ships.iter().filter(|s| own(s) && s.grow.is_none() && s.warp == 0 && is(s, Card::warship)).map(|s| (s.slot, s.px_now())).collect();
        let army: i32 = fleet.iter().map(|f| f.1).sum();
        // What it faces: the enemy's warships by class, and whether any is near its king or
        // anything it has planted.
        let foe = |s: &&Ship| s.alive && s.team != t && s.role == Role::Ship && s.grow.is_none() && s.warp == 0 && s.st.armed() && s.st.thrust > 0 && s.order != Order::Harvest;
        let mut faced = [0i32; 3];
        for s in self.ships.iter().filter(foe) {
            faced[class_of(&s.st)] += s.px_now();
        }
        let enemy_army: i32 = faced.iter().sum();
        // To the AI, an enemy king that has come over to its half of the map is a raid already,
        // however far it still has to fly: a king on its way is the one attack that's seen
        // coming. (The scripted openings see only ships, as they always have.)
        let (home, far) = (KING_STARTS[t], KING_STARTS[1 - t]);
        let invader = |s: &&Ship| bot == Bot::Ai && s.alive && s.team != t && s.role == Role::King && dist2(s.x, s.y, home.0, home.1) < dist2(s.x, s.y, far.0, far.1);
        // What's the AI's to defend: its king, what it has planted, and its miners. (For the
        // scripted openings, as always, the king and what's planted.)
        let by_mine = |x: i32, y: i32, range: i32| self.ships.iter().any(|s| s.alive && s.team == t && (s.role == Role::King || s.rooted || (bot == Bot::Ai && s.role == Role::Ship && is(s, Card::miner))) && cheb(s.x, s.y, x, y) <= range);
        let raiders = |range: i32| -> Vec<(i32, i32, i32, bool)> { self.ships.iter().filter(|s| invader(s) || (foe(s) && by_mine(s.x, s.y, range))).map(|f| (f.x, f.y, f.px_now(), f.role == Role::King)).collect() };
        let alarm = !raiders(ALARM_RANGE).is_empty();
        // Its fleet, once turned out, stays out until the raiders are well clear.
        let raid = if alarm || mind.alarm { raiders(ALARM_RANGE + ALARM_CLEAR) } else { Vec::new() };
        let turned_out = !raid.is_empty();
        mind.alarm = turned_out;
        // A king raiding with ships round it is answered through its ships: they're what does
        // the damage, and what it would have to be reached through. A king raiding alone is the
        // biggest ship there is, and what answers it is what answers big ships.
        if raid.iter().all(|r| r.3) {
            faced[2] += raid.iter().map(|r| r.2).sum::<i32>();
        }
        // The raid against what the AI has to answer it with: every warship it has, or, while
        // a wave is out, those near its king or anything it has planted. A raid that outweighs
        // them changes what it prints and where it plants.
        let raid_px: i32 = raid.iter().map(|r| r.2).sum();
        let home_px: i32 = if mind.wave {
            self.ships.iter().filter(|s| own(s) && s.grow.is_none() && s.warp == 0 && is(s, Card::warship) && by_mine(s.x, s.y, ALARM_RANGE)).map(|s| s.px_now()).sum()
        } else {
            army
        };
        let outgunned = bot == Bot::Ai && raid_px > home_px;
        // The enemy's fleet has the edge over its own that the AI would itself attack with.
        let outarmed = enemy_army > 0 && enemy_army * 4 >= army * 5;

        // Orders. The AI masses its warships on GUARD and sends them out as a wave: when it's the
        // stronger side, or after a rest if it's not much the weaker, or after a long rest
        // whatever the odds. The scripted bots leave everything on HUNT.
        let warships: Vec<usize> = (0..cards.len()).filter(|&s| cards[s].warship()).collect();
        if bot == Bot::Ai {
            let rest = tick - mind.rested;
            if mind.wave {
                // The wave is spent, or has been out long enough: home, and mass again.
                if army * 5 < mind.wave_worth * 2 || tick - mind.wave_from > WAVE_TICKS {
                    mind.wave = false;
                    mind.rested = tick;
                }
            } else if army >= WAVE_MIN && (army * 4 >= enemy_army * 5 || (rest > 15 * 120 && army * 5 >= enemy_army * 4) || rest > 15 * 240) {
                // The slow designs leave first and the fast ones wait, so the wave reaches the
                // enemy king's side of the map together.
                let (ek, mk) = (&self.ships[self.eco().king[1 - t]], &self.ships[k]);
                let far = isqrt(dist2(ek.x, ek.y, mk.x, mk.y));
                let trip = |s: usize| far * 4 / cards[s].st.vmax.max(1);
                let longest = warships.iter().filter(|&&s| fleet.iter().any(|f| f.0 == s)).map(|&s| trip(s)).max().unwrap_or(0);
                mind.release = (0..cards.len()).map(|s| tick + (longest - trip(s)).max(0) as u32).collect();
                mind.wave = true;
                mind.wave_from = tick;
                mind.wave_worth = army;
            }
        }
        for &s in &warships {
            let out = bot != Bot::Ai || turned_out || (mind.wave && mind.release.get(s).is_some_and(|&r| tick >= r));
            let want = if bot == Bot::Saboteur { Order::Sab } else if out { Order::Hunt } else { Order::Guard };
            if self.eco().orders[t][s] != want {
                self.command(Cmd::Order { team: t, slot: s, order: want });
            }
        }

        // Planting: the next factory, as soon as the bank holds its price.
        let mut facts: Vec<usize> = (0..cards.len()).filter(|&s| cards[s].factory()).collect();
        facts.sort_by_key(|&s| cards[s].cost);
        let (small, big) = (facts.first().copied(), facts.last().copied());
        let n = factories + seeds;
        let pick = match bot {
            Bot::Planter | Bot::Boomer => small,
            Bot::Summoner | Bot::Rusher => small.filter(|_| n == 0),
            Bot::Clock => (if n >= 2 { big } else { small }).filter(|_| seeds == 0 && n < (1 + (tick / (15 * 75)) as usize).min(9)),
            _ => {
                // Two small ones, then big ones: six in all. A big one it has waited four minutes
                // for gives way to a small one. And it doesn't let itself be out-built: while the
                // enemy has more factories than it has, it plants small ones as fast as it can
                // pay for them, past six if need be.
                let waited = tick - mind.last_plant > 15 * 240;
                let behind = theirs > n;
                (if n < 2 || waited || behind { small } else { big }).filter(|_| n < 6 || (behind && n < 14))
            }
        };
        // It keeps something back for the new factory to print with. The scripted openings
        // with the AI's economy keep more under a raid, as the AI did until 2026-11-12: a side
        // that saves more while it's being raided has less to answer the raid with.
        let spare = match bot {
            Bot::Expander | Bot::Saboteur if alarm => 150,
            Bot::Ai | Bot::Expander | Bot::Saboteur => 40,
            _ => 0,
        };
        let mut saving = 0;
        if let Some(slot) = pick {
            // Against a raid that outweighs it the AI plants nowhere the raiders are, and only
            // where the new factory joins its network: one cut off would have nothing to print
            // with. With no factory left it plants on any rock the raiders aren't at.
            let (avoid, joined) = if outgunned { (&raid[..], n > 0) } else { (&raid[..0], false) };
            let rock = |s: usize| self.ai_rock(t, cards[s].count, avoid, joined).map(|r| (s, r));
            if let Some((slot, target)) = rock(slot).or_else(|| small.filter(|&s| s != slot).and_then(rock)) {
                saving = cards[slot].price() + spare;
                if left >= saving {
                    self.command(Cmd::Print { team: t, slot, target: Some(target) });
                    mind.last_plant = tick;
                    left -= cards[slot].price();
                    saving = 0;
                }
            }
        }

        // The king's own printing, one thing at a time.
        if self.eco().queue[t].is_empty() {
            let cheapest = |f: fn(&Card) -> bool| (0..cards.len()).filter(|&s| f(&cards[s])).min_by_key(|&s| cards[s].cost);
            let short_of_miners = miners < factories.max(1) + 1;
            let print = match bot {
                // Against a raid that outweighs what it has, guns before anything: something
                // small, right now.
                Bot::Ai if outgunned => Self::counter(&cards, faced, left.min(100)),
                // Miners: one more than it has factories, whatever the factories add.
                Bot::Ai | Bot::Expander | Bot::Saboteur if short_of_miners && cheapest(Card::miner).is_some() => cheapest(Card::miner),
                // Then guns: against any other raid, something small right now. While the
                // enemy's fleet has the edge, guns come before what it's saving for. Otherwise
                // only with more in the bank than the next factory needs.
                Bot::Ai if alarm => Self::counter(&cards, faced, left.min(100)),
                Bot::Ai if outarmed => Self::counter(&cards, faced, (left - saving).max(0)),
                Bot::Ai if left >= saving + 120 => Self::counter(&cards, faced, left - saving),
                Bot::Summoner => {
                    // Every warship but its dearest, in turn, without a pause.
                    let mut w = warships.clone();
                    w.sort_by_key(|&s| cards[s].cost);
                    if w.len() > 1 {
                        w.pop();
                    }
                    mind.turn += 1;
                    w.get(mind.turn % w.len().max(1)).copied()
                }
                Bot::Boomer => cheapest(Card::miner),
                // Its cheapest ship with a lance or a cannon; with none in the deck, its
                // cheapest warship.
                Bot::Rusher => (0..cards.len()).filter(|&s| cards[s].warship() && class_of(&cards[s].st) == 1).min_by_key(|&s| cards[s].cost).or_else(|| cheapest(Card::warship)),
                Bot::Clock if tick > 15 * 240 && tick >= mind.big_at => {
                    mind.big_at = tick + 15 * 150;
                    warships.iter().copied().max_by_key(|&s| cards[s].cost)
                }
                _ => None,
            };
            if let Some(slot) = print {
                self.command(Cmd::Print { team: t, slot, target: None });
            }
        }
        self.eco_mut().mind[t] = mind;
    }

    /// A bot's king stays home, and backs away from any armed enemy that comes near. The AI's
    /// king is also its link: when nodes cut off from it hold a good many pixels, it flies over
    /// until they're in its network, and goes home once it has spent them. And it defends:
    /// while a raid is on it stands beside its nearest factory, it backs away only from what
    /// outweighs it, and then towards a factory nobody is attacking. The rushing opening's king
    /// does none of that: it flies at the enemy.
    fn bot_steer(&mut self, t: usize) {
        let k = self.eco().king[t];
        let (kx, ky) = (self.ships[k].x, self.ships[k].y);
        if self.eco().bot[t] == Bot::Rusher {
            // Up to the nearest thing the enemy has planted (its king, when it has nothing),
            // close enough for the king's own flak, and no backing away: until it has lost a
            // quarter of the pixels it started with, when it goes home and stays there.
            if self.ships[k].px_now() * 4 < crate::pg::design::king().stats().count * 3 {
                self.eco_mut().mind[t].gone_home = true;
            }
            let at = if self.eco().mind[t].gone_home {
                Some(KING_STARTS[t]).filter(|&p| cheb(kx, ky, p.0, p.1) > 6)
            } else {
                self.ships.iter().filter(|s| s.alive && s.team != t && (s.rooted || s.role == Role::King)).min_by_key(|s| (s.role == Role::King, dist2(s.x, s.y, kx, ky))).map(|s| (s.x, s.y)).filter(|&p| cheb(kx, ky, p.0, p.1) > RUSH_CLOSE)
            };
            let steer = at.map_or((0, 0), |p| ((p.0 - kx).signum(), (p.1 - ky).signum()));
            self.command(Cmd::Steer { team: t, dx: steer.0, dy: steer.1 });
            return;
        }
        let mut post = KING_STARTS[t];
        if self.eco().bot[t] == Bot::Ai {
            // While a raid is on, its post is beside the factory nearest its start, on the side
            // it came from, if that's within its network's reach of the start: one strongpoint,
            // where the factory's guns and its own cover each other and what it prints is posted
            // too. With no raid it keeps its start, which its network was laid out round. And
            // it keeps its start in a raid too if moving would cut a node off: every node in its
            // reach from the start must still be in its reach from the post. (A node the factory
            // would link is always one of those: the post is nearer the factory than a king's
            // reach is longer than a node's.)
            let start = KING_STARTS[t];
            let far = KING_RANGE + NODE_RANGE;
            let base = self.ships.iter().filter(|s| s.alive && s.team == t && s.rooted).min_by_key(|s| dist2(s.x, s.y, start.0, start.1)).filter(|s| dist2(s.x, s.y, start.0, start.1) <= far * far);
            if let Some(f) = base.filter(|_| self.eco().mind[t].alarm) {
                let gap = f.half() + self.ships[k].half() + STRONGPOINT_GAP;
                let (dx, dy) = (start.0 - f.x, start.1 - f.y);
                let p = if dx.abs() >= dy.abs() { (f.x + dx.signum() * gap, f.y) } else { (f.x, f.y + dy.signum() * gap) };
                let kept = (0..self.ships.len())
                    .filter(|&j| j != k && self.ships[j].alive && self.ships[j].team == t && self.is_node(j))
                    .map(|j| (self.ships[j].x, self.ships[j].y))
                    .filter(|&(x, y)| dist2(x, y, start.0, start.1) <= far * far)
                    .all(|(x, y)| dist2(x, y, p.0, p.1) <= far * far);
                if kept {
                    post = p;
                }
            }
            let net = &self.eco().net;
            let mine = net.get(k).copied().unwrap_or(usize::MAX);
            // The cut-off network holding the most, if it's worth the trip (or the king is there
            // already and it still holds something).
            let cut: Vec<usize> = (0..self.ships.len()).filter(|&j| self.ships[j].alive && self.ships[j].team == t && self.is_node(j) && net.get(j).is_some_and(|&n| n != mine)).collect();
            let held = |id: usize| cut.iter().filter(|&&j| net[j] == id).map(|&j| self.ships[j].store).sum::<i32>();
            let away = cheb(kx, ky, post.0, post.1) > 20;
            let rich = cut.iter().map(|&j| net[j]).max_by_key(|&id| held(id)).filter(|&id| held(id) >= 200);
            let here = (0..self.ships.len()).filter(|&j| j != k && self.ships[j].alive && self.ships[j].rooted && net.get(j) == Some(&mine)).min_by_key(|&j| dist2(self.ships[j].x, self.ships[j].y, kx, ky));
            if let Some(id) = rich {
                // Fly to the nearest of its nodes, stopping where the link is made.
                let j = cut.iter().copied().filter(|&j| net[j] == id).min_by_key(|&j| dist2(self.ships[j].x, self.ships[j].y, kx, ky)).unwrap();
                post = (self.ships[j].x, self.ships[j].y);
                let near = KING_RANGE + NODE_RANGE - 30;
                if dist2(kx, ky, post.0, post.1) <= near * near {
                    post = (kx, ky);
                }
            } else if let (true, Some(j)) = (away && self.bank(k) >= 60, here) {
                // Linked up away from home, with pixels still to spend: stay where it is.
                let _ = j;
                post = (kx, ky);
            }
        }
        let mut threat = self.ships.iter().filter(|s| s.alive && s.team != t && s.st.armed() && cheb(s.x, s.y, kx, ky) < 90).min_by_key(|s| dist2(s.x, s.y, kx, ky));
        if self.eco().bot[t] == Bot::Ai {
            // The AI's king stands its ground where it and the armed friends round it outweigh
            // the armed enemies round it, each within the alarm's range. Backing away from less
            // only takes its guns out of the fight and leaves its factory to it.
            let (mine, theirs) = self.ships.iter().filter(|s| s.alive && s.st.armed() && cheb(s.x, s.y, kx, ky) <= ALARM_RANGE).fold((0, 0), |(m, e), s| if s.team == t { (m + s.px_now(), e) } else { (m, e + s.px_now()) });
            if mine >= theirs {
                threat = None;
            }
        }
        // Outweighed, the AI's king falls back on the nearest of its factories that no enemy gun
        // is near, if it has one. With none it backs straight away, as every bot's king does.
        let refuge = if threat.is_some() && self.eco().bot[t] == Bot::Ai {
            let quiet = |f: &&Ship| !self.ships.iter().any(|e| e.alive && e.team != t && e.st.armed() && cheb(e.x, e.y, f.x, f.y) <= ALARM_RANGE);
            self.ships.iter().filter(|s| s.alive && s.team == t && s.rooted).filter(quiet).min_by_key(|f| dist2(f.x, f.y, kx, ky)).map(|f| (f.x, f.y, f.half() + self.ships[k].half() + STRONGPOINT_GAP))
        } else {
            None
        };
        let steer = match threat {
            Some(_) if refuge.is_some() => {
                let (x, y, gap) = refuge.unwrap();
                if cheb(kx, ky, x, y) > gap + 6 { ((x - kx).signum(), (y - ky).signum()) } else { (0, 0) }
            }
            Some(s) => ((kx - s.x).signum(), (ky - s.y).signum()),
            None if cheb(kx, ky, post.0, post.1) > 6 => ((post.0 - kx).signum(), (post.1 - ky).signum()),
            None => (0, 0),
        };
        self.command(Cmd::Steer { team: t, dx: steer.0, dy: steer.1 });
    }

    /// The warship to print against what the enemy is fielding: against small flak ships
    /// something big, against small lance ships a flak swarm, against big ships lances. Of that
    /// class, the dearest that `bank` covers, or failing that the cheapest.
    fn counter(cards: &[Card], faced: [i32; 3], bank: i32) -> Option<usize> {
        let most = (0..3).max_by_key(|&c| faced[c]).filter(|&c| faced[c] > 0);
        let want = match most {
            Some(0) | None => 2,
            Some(1) => 0,
            Some(_) => 1,
        };
        let of = |class: usize| (0..cards.len()).filter(move |&s| cards[s].warship() && class_of(&cards[s].st) == class);
        let pool: Vec<usize> = if of(want).next().is_some() { of(want).collect() } else { (0..cards.len()).filter(|&s| cards[s].warship()).collect() };
        let dearest = pool.iter().copied().filter(|&s| cards[s].cost <= bank).max_by_key(|&s| cards[s].cost);
        dearest.or_else(|| pool.iter().copied().min_by_key(|&s| cards[s].cost))
    }

    /// The free rock nearest a bot's king that's big enough: nobody's growing on it and none of
    /// its seeds is headed there. First choice is a rock on its own side where a factory would
    /// link to its king's network (so what's mined there reaches the bank it plants from), then
    /// any rock on its own side, then any rock at least 300 px from the enemy king. Returns the
    /// rock pixel nearest the rock's middle.
    fn ai_rock(&self, t: usize, need: i32, raid: &[(i32, i32, i32, bool)], joined: bool) -> Option<(i32, i32)> {
        let (k, ek) = (self.eco().king[t], self.eco().king[1 - t]);
        let (kx, ky) = (self.ships[k].x, self.ships[k].y);
        let (ex, ey) = (self.ships[ek].x, self.ships[ek].y);
        let rocks = &self.eco().rocks;
        // Where its king's network reaches: the king's post at home, what's planted in the
        // network, and the rocks its seeds are headed for that will link in their turn.
        let net = self.eco().net.get(k).copied().unwrap_or(usize::MAX);
        let home = KING_STARTS[t];
        let mut reach: Vec<(i32, i32, i32)> = vec![(home.0, home.1, KING_RANGE)];
        reach.extend((0..self.ships.len()).filter(|&j| self.ships[j].alive && self.ships[j].rooted && net != usize::MAX && self.eco().net.get(j) == Some(&net)).map(|j| (self.ships[j].x, self.ships[j].y, NODE_RANGE)));
        let linked = |reach: &[(i32, i32, i32)], x: i32, y: i32| reach.iter().any(|&(nx, ny, far)| dist2(x, y, nx, ny) <= (far + NODE_RANGE - 20) * (far + NODE_RANGE - 20));
        let mut seeds: Vec<(i32, i32)> = self.ships.iter().filter(|s| s.alive && s.team == t).filter_map(|s| s.seed_for).map(|(_, p)| p).collect();
        while let Some(i) = seeds.iter().position(|&(x, y)| linked(&reach, x, y)) {
            let (x, y) = seeds.remove(i);
            reach.push((x, y, NODE_RANGE));
        }
        for pass in 0..if joined { 1 } else { 3 } {
            let mut best: Option<(i32, (i32, i32))> = None;
            for r in rocks {
                let left = self.rock_left(r);
                if left < need {
                    continue;
                }
                let rad = isqrt(left / 3) + 4;
                let taken = self.ships.iter().any(|s| s.alive && s.role == Role::Ship && (s.node || s.grow.as_ref().is_some_and(|g| g.eats)) && cheb(s.x, s.y, r.cx, r.cy) <= rad + s.half())
                    || self.ships.iter().any(|s| s.alive && s.seed_for.is_some_and(|(_, p)| cheb(p.0, p.1, r.cx, r.cy) <= rad));
                let (mine, theirs) = (dist2(r.cx, r.cy, kx, ky), dist2(r.cx, r.cy, ex, ey));
                let links = linked(&reach, r.cx, r.cy);
                let allowed = match pass {
                    0 => mine < theirs && links,
                    1 => mine < theirs,
                    _ => theirs >= 300 * 300,
                };
                // A seed sent to a rock the raiders are at is a factory thrown away.
                let raided = raid.iter().any(|&(x, y, _, _)| cheb(x, y, r.cx, r.cy) <= ALARM_RANGE);
                if taken || !allowed || raided {
                    continue;
                }
                if best.map_or(true, |(b, _)| mine < b) {
                    if let Some(target) = r.px.iter().copied().filter(|&(x, y)| self.rock_at(x, y)).min_by_key(|&(x, y)| dist2(x, y, r.cx, r.cy)) {
                        best = Some((mine, target));
                    }
                }
            }
            if best.is_some() {
                return best.map(|(_, p)| p);
            }
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The most pixels a king moves in a tick.
    const STEP: i32 = (KING_SPEED + 3) / 4;

    fn quiet() -> Arena {
        Arena::for_match(1, [starter_deck(), starter_deck()], [false, false])
    }

    /// Blue's home rock (the one nearest its king) and the rock pixel nearest its middle.
    fn home_rock(a: &Arena) -> (usize, (i32, i32)) {
        let (kx, ky) = KING_STARTS[0];
        let e = a.eco.as_ref().unwrap();
        let r = (0..e.rocks.len()).min_by_key(|&r| dist2(e.rocks[r].cx, e.rocks[r].cy, kx, ky)).unwrap();
        let rock = &e.rocks[r];
        let p = rock.px.iter().copied().min_by_key(|&(x, y)| dist2(x, y, rock.cx, rock.cy)).unwrap();
        (r, p)
    }

    fn factory(a: &Arena) -> Option<usize> {
        (0..a.ships.len()).find(|&i| a.ships[i].alive && a.ships[i].team == 0 && a.ships[i].role == Role::Ship && a.ships[i].node)
    }

    #[test]
    fn the_map_is_a_mirror_image() {
        let a = quiet();
        for y in 0..a.h {
            for x in 0..a.w {
                assert_eq!(a.rock_at(x, y), a.rock_at(a.w - 1 - x, a.h - 1 - y), "({x},{y})");
            }
        }
        let k = a.eco.as_ref().unwrap().king;
        assert_eq!((a.ships[k[0]].x, a.ships[k[0]].y), (a.w - 1 - a.ships[k[1]].x, a.h - 1 - a.ships[k[1]].y));
    }

    #[test]
    fn a_seed_grows_its_factory_out_of_the_rock_pixel_by_pixel() {
        let mut a = quiet();
        let (r, target) = home_rock(&a);
        let rock = a.eco.as_ref().unwrap().rocks[r].clone();
        let before = a.rock_left(&rock);
        a.command(Cmd::Print { team: 0, slot: 4, target: Some(target) });
        // The king prints the seed (20 px at 6 px/s), it arrives by hyperspace, flies and lands.
        let mut last = 0;
        let mut grew = false;
        for _ in 0..15 * 120 {
            a.step();
            if let Some(f) = factory(&a) {
                let now = a.ships[f].px_now();
                assert!(now >= last && now <= last + 3, "it jumped from {last} to {now} px in a tick");
                last = now;
                if a.ships[f].grow.is_none() {
                    grew = true;
                    break;
                }
            }
        }
        assert!(grew, "the outpost never finished growing");
        let f = factory(&a).unwrap();
        let count = starter_deck()[4].stats().count;
        assert_eq!(a.ships[f].px_now(), count, "it should finish whole on a rock this big");
        // One rock pixel eaten for every pixel grown, bar the core the seed brought.
        assert_eq!(before - a.rock_left(&rock), count - 1);
        assert_eq!(a.eco.as_ref().unwrap().tally[0].planted, 1);
    }

    #[test]
    fn factories_print_a_pixel_at_a_time_and_wait_when_broke() {
        let mut a = quiet();
        let (_, target) = home_rock(&a);
        a.command(Cmd::Print { team: 0, slot: 4, target: Some(target) });
        while factory(&a).map_or(true, |f| a.ships[f].grow.is_some()) {
            a.step();
        }
        let f = factory(&a).unwrap();
        let k = a.eco.as_ref().unwrap().king[0];
        while !matches!(a.ships[f].making, Making::Child(_)) {
            a.step();
        }
        let Making::Child(c) = a.ships[f].making else { unreachable!() };
        assert_eq!(a.ships[c].name, "MINER");
        assert!(a.ships[c].maker == Some(f) && a.ships[c].px_now() >= 1);
        // Empty the bank: from here it prints only the pixels the king makes, as they come.
        for i in 0..a.ships.len() {
            a.ships[i].store = 0;
        }
        let (stuck, made) = (a.ships[c].px_now(), a.eco.as_ref().unwrap().made[0]);
        for _ in 0..15 * 10 {
            a.step();
        }
        let trickle = a.eco.as_ref().unwrap().made[0] - made;
        assert_eq!(a.ships[c].px_now() - stuck, trickle, "with an empty bank it should print what the king makes and no more");
        assert!(trickle <= 3, "the king made {trickle} px in ten seconds");
        assert_eq!(a.ships[f].making, Making::Child(c));
        // Pixels arrive: it carries on, then launches the finished miner.
        a.ships[k].store = 100;
        for _ in 0..15 * 30 {
            a.step();
        }
        assert!(a.ships[c].maker.is_none() && a.ships[c].px_now() == a.ships[c].px_full(), "the miner never launched");
        assert!(a.eco.as_ref().unwrap().tally[0].printed >= 1);
    }

    #[test]
    fn a_factory_builds_only_what_its_factory_pixels_allow() {
        // A tiny factory (2 factory pixels: ships up to 20 px) with a frigate (44 px) first on its
        // list skips straight to the fighter.
        let mut deck = starter_deck();
        let mut tiny = Design::from_rows("TINY", Order::Hold, &["hfh", "hch", "hfh"]);
        tiny.seeded = true;
        tiny.build = vec![2, 0];
        deck[4] = tiny;
        let mut a = Arena::for_match(1, [deck, starter_deck()], [false, false]);
        let (_, target) = home_rock(&a);
        a.command(Cmd::Print { team: 0, slot: 4, target: Some(target) });
        while factory(&a).map_or(true, |f| !matches!(a.ships[f].making, Making::Child(_))) {
            a.step();
        }
        let f = factory(&a).unwrap();
        let Making::Child(c) = a.ships[f].making else { unreachable!() };
        assert_eq!(a.ships[c].name, "FIGHTER");
    }

    #[test]
    fn linked_nodes_share_a_bank_and_split_where_the_pixels_sit() {
        let mut a = quiet();
        let (_, target) = home_rock(&a);
        a.command(Cmd::Print { team: 0, slot: 4, target: Some(target) });
        while factory(&a).is_none() {
            a.step();
        }
        let f = factory(&a).unwrap();
        let k = a.eco.as_ref().unwrap().king[0];
        a.step();
        a.ships[f].store = 50;
        let both = a.ships[k].store + 50;
        assert_eq!(a.bank(k), both, "the king and its outpost should be one network");
        // Fly the king far off: the network splits, each half keeping what it holds.
        for _ in 0..15 * 8 {
            a.command(Cmd::Steer { team: 0, dx: 1, dy: -1 });
            a.step();
        }
        assert_eq!(a.bank(f), a.ships[f].store);
        assert_eq!(a.bank(k), a.ships[k].store);
        assert_eq!(a.eco.as_ref().unwrap().split_at[0], 0, "a king flying out of reach was noted as a split");
    }

    /// A quiet match with two of blue's outposts standing at these offsets from its king.
    fn two_outposts(da: (i32, i32), db: (i32, i32)) -> (Arena, usize, usize, usize) {
        let mut a = quiet();
        let k = a.eco.as_ref().unwrap().king[0];
        let (kx, ky) = KING_STARTS[0];
        let outpost = starter_deck()[4].clone();
        let put = |a: &mut Arena, (dx, dy): (i32, i32)| {
            let i = a.spawn(0, 4, &outpost, kx + dx, ky + dy, 0, false);
            a.ships[i].rooted = true;
            i
        };
        let (fa, fb) = (put(&mut a, da), put(&mut a, db));
        a.step();
        (a, k, fa, fb)
    }

    #[test]
    fn a_network_split_is_two_nodes_parting_or_a_king_cut_off_by_a_loss() {
        let split = |a: &Arena| a.eco.as_ref().unwrap().split_at[0];
        let net = |a: &Arena, i: usize| a.eco.as_ref().unwrap().net[i];
        // King, outpost, outpost in a row: the far one links through the near one. The king flies
        // up and out of reach: the two outposts are still linked, and nothing is noted.
        let (mut a, k, near, far) = two_outposts((150, 0), (290, 0));
        assert!(net(&a, k) == net(&a, near) && net(&a, near) == net(&a, far), "the chain isn't one network");
        for _ in 0..15 * 6 {
            a.command(Cmd::Steer { team: 0, dx: 0, dy: -1 });
            a.step();
        }
        assert!(net(&a, k) != net(&a, near) && net(&a, near) == net(&a, far), "the king didn't leave the two linked");
        assert_eq!(split(&a), 0, "the king flying off by itself was noted as a split");
        // The same row with the king at home: the near outpost is destroyed, and the king is cut
        // off from the far one. That's a split, though one of the two parted is the king.
        let (mut a, k, near, far) = two_outposts((150, 0), (290, 0));
        a.ships[near].hp.iter_mut().for_each(|h| *h = 0);
        a.ships[near].alive = false;
        a.step();
        assert!(net(&a, k) != net(&a, far));
        assert!(split(&a) > 0, "a king cut off by the loss of a node wasn't noted");
        // Two outposts linked only through the king between them: when it flies off they part,
        // and that's a split too.
        let (mut a, k, left, up) = two_outposts((190, 0), (0, -190));
        assert!(net(&a, left) == net(&a, up) && net(&a, k) == net(&a, up), "the king isn't the link");
        for _ in 0..15 * 6 {
            a.command(Cmd::Steer { team: 0, dx: 0, dy: 1 });
            a.step();
        }
        assert!(net(&a, left) != net(&a, up));
        assert!(split(&a) > 0, "two nodes parting went unnoticed");
    }

    #[test]
    fn a_seed_comes_down_where_the_preview_said_and_links_what_it_showed() {
        let mut a = quiet();
        let k = a.eco.as_ref().unwrap().king[0];
        let king = (a.ships[k].x, a.ships[k].y);
        // Rock to plant on, placed to the pixel: one at the very edge of the king's reach and
        // one a pixel past it, one 37 px past it, and one that only a factory on the home rock
        // would reach.
        let (_, home) = home_rock(&a);
        let reach = KING_RANGE + NODE_RANGE;
        let mut scratch = Vec::new();
        let mut rock = |a: &mut Arena, at: (i32, i32)| {
            a.put_rock((0..25).map(|i| (at.0 - 2 + i % 5, at.1 - 2 + i / 5)).collect(), &mut scratch);
            at
        };
        let edge = rock(&mut a, (king.0 + reach, king.1));
        let past = rock(&mut a, (king.0, king.1 - reach - 1));
        let far = rock(&mut a, (king.0 + reach + 37, king.1 - 60));
        let onward = rock(&mut a, (home.0 + 2 * NODE_RANGE - 10, home.1));
        let beyond = rock(&mut a, (past.0, past.1 - 100));
        // The outpost's core is its middle, so it stands exactly on the pixel its seed is sent to.
        let at_home = a.plant_links(0, 4, home).unwrap();
        assert_eq!(at_home, Links { at: home, to: vec![king], king: true, gap: None });
        assert_eq!(a.plant_links(0, 4, edge).unwrap(), Links { at: edge, to: vec![king], king: true, gap: None });
        assert_eq!(a.plant_links(0, 4, past).unwrap(), Links { at: past, to: vec![], king: false, gap: Some((king, 1)) });
        let alone = a.plant_links(0, 4, far).unwrap();
        assert!(!alone.king && alone.to.is_empty() && alone.gap.is_some_and(|g| g.0 == king && g.1 >= 37), "{alone:?}");
        assert_eq!(a.plant_links(0, 4, onward).unwrap().to, vec![], "nothing stands on the home rock yet");
        // A fighter links nothing: there's nothing to show.
        assert_eq!(a.plant_links(0, 0, home), None);
        // A seed on its way to the home rock counts as the outpost it will grow: the rock beyond
        // now links to it, and through it to the king.
        a.ships[k].store += 400;
        a.score[0].mined += 400;
        a.command(Cmd::Print { team: 0, slot: 4, target: Some(home) });
        a.step();
        a.step();
        assert!(factory(&a).is_none(), "the seed has landed already");
        let chained = a.plant_links(0, 4, onward).unwrap();
        assert_eq!(chained, Links { at: onward, to: vec![home], king: true, gap: None });
        // Both are planted, and stand where the preview put them, in the network it showed.
        a.command(Cmd::Print { team: 0, slot: 4, target: Some(onward) });
        a.command(Cmd::Print { team: 0, slot: 4, target: Some(past) });
        let stands = |a: &Arena, at: (i32, i32)| (0..a.ships.len()).find(|&i| a.ships[i].alive && a.ships[i].rooted && (a.ships[i].x, a.ships[i].y) == at);
        for _ in 0..15 * 60 {
            a.step();
        }
        let rooted: Vec<(i32, i32)> = a.ships.iter().filter(|s| s.alive && s.rooted).map(|s| (s.x, s.y)).collect();
        let (Some(h), Some(o), Some(p)) = (stands(&a, home), stands(&a, onward), stands(&a, past)) else { panic!("the outposts stand at {rooted:?}, not at {home:?}, {onward:?} and {past:?}") };
        let net = &a.eco.as_ref().unwrap().net;
        assert!(net[h] == net[k] && net[o] == net[k], "the chain the preview showed isn't one network");
        assert!(net[p] != net[k], "the outpost a pixel out of reach is in the king's network");
        // A rock in reach of that outpost alone would link, but not to the king's bank.
        assert_eq!(a.plant_links(0, 4, beyond).unwrap(), Links { at: beyond, to: vec![past], king: false, gap: None });
    }

    #[test]
    fn a_destroyed_node_spills_what_it_held() {
        let mut a = quiet();
        let k = a.eco.as_ref().unwrap().king[1];
        let store = a.ships[k].store;
        let n = a.debris.len();
        a.ships[k].hp.iter_mut().for_each(|h| *h = 0);
        a.step();
        assert!(!a.ships[k].alive);
        assert!(a.debris.len() - n >= store.min(SPILL_CAP) as usize);
        assert_eq!(a.over, Some(Some(0)), "killing the red king wins for blue");
    }

    #[test]
    fn the_king_flies_straight_from_its_keys() {
        let mut a = quiet();
        let k = a.eco.as_ref().unwrap().king[0];
        let (x, y) = (a.ships[k].x, a.ships[k].y);
        a.command(Cmd::Steer { team: 0, dx: 1, dy: 0 });
        for _ in 0..10 {
            a.step();
        }
        assert_eq!((a.ships[k].x, a.ships[k].y), (x + 10 * KING_SPEED / 4, y));
    }

    /// A block of hull round a core, `w` by `h`: pixels standing that do nothing else.
    fn slab(w: usize, h: usize) -> Design {
        let rows: Vec<String> = (0..h).map(|y| (0..w).map(|x| if (x, y) == (w / 2, h / 2) { 'c' } else { 'h' }).collect()).collect();
        Design::from_rows("SLAB", Order::Hold, &rows.iter().map(|r| r.as_str()).collect::<Vec<_>>())
    }

    #[test]
    fn a_king_with_next_to_nothing_standing_is_slowed() {
        let mut a = quiet();
        let k = a.eco.as_ref().unwrap().king;
        // How far a king gets in eight ticks, flown towards the middle of the map.
        let pace = |a: &mut Arena, t: usize| {
            let x = a.ships[k[t]].x;
            a.command(Cmd::Steer { team: t, dx: if t == 0 { 1 } else { -1 }, dy: 0 });
            for _ in 0..8 {
                a.step();
            }
            a.command(Cmd::Steer { team: t, dx: 0, dy: 0 });
            a.step();
            (a.ships[k[t]].x - x).abs()
        };
        let (full, slow) = (8 * KING_SPEED / 4, 8 * KING_SLOW / 4);
        assert!(slow > 0 && slow * 4 < full);
        // The first minute: nobody has built anything, and both kings fly as they always did.
        assert_eq!((a.standing(0), a.standing(1)), (0, 0));
        assert_eq!((pace(&mut a, 0), pace(&mut a, 1)), (full, full));
        while a.tick < KING_GRACE {
            a.step();
        }
        // After it, a side with nothing standing has a slow king. The king's own pixels don't count.
        assert!(a.ships[k[0]].px_now() > KING_NEEDS);
        assert_eq!((pace(&mut a, 0), pace(&mut a, 1)), (slow, slow));
        // One pixel short of enough is still short, and what the other side has doesn't count.
        let (kx, ky) = (a.ships[k[0]].x, a.ships[k[0]].y);
        let (ex, ey) = (a.ships[k[1]].x, a.ships[k[1]].y);
        assert_eq!(slab(13, 3).stats().count, KING_NEEDS - 1);
        a.spawn(0, 0, &slab(13, 3), kx - 60, ky - 60, 0, false);
        a.spawn(1, 0, &slab(8, 5), ex + 60, ey + 60, 0, false);
        assert_eq!((a.standing(0), a.standing(1)), (KING_NEEDS - 1, KING_NEEDS));
        assert_eq!((pace(&mut a, 0), pace(&mut a, 1)), (slow, full));
        // The last pixel: full speed again. Lose it: slow again.
        let (kx, ky) = (a.ships[k[0]].x, a.ships[k[0]].y);
        let dot = a.spawn(0, 0, &slab(1, 1), kx - 60, ky + 40, 0, false);
        assert_eq!(a.standing(0), KING_NEEDS);
        assert_eq!(pace(&mut a, 0), full);
        a.ships[dot].hp[0] = 0;
        a.step();
        assert!(!a.ships[dot].alive);
        assert_eq!(pace(&mut a, 0), slow);
    }

    #[test]
    fn a_slowed_king_flying_up_or_left_keeps_its_remainders_in_range() {
        // Half a pixel a tick, leftward and upward: the quarter-pixels left over stay 0 to 3, as
        // every other ship's do. Kept negative, a piece shot off the king took them along and
        // drifted a pixel with no engine.
        let mut a = quiet();
        while a.tick < KING_GRACE {
            a.step();
        }
        let k = a.eco.as_ref().unwrap().king[1];
        assert!(a.king_slowed(1));
        let (x, y) = (a.ships[k].x, a.ships[k].y);
        a.command(Cmd::Steer { team: 1, dx: -1, dy: -1 });
        for _ in 0..8 {
            a.step();
            a.audit().unwrap();
        }
        assert_eq!((x - a.ships[k].x, y - a.ships[k].y), (8 * KING_SLOW / 4, 8 * KING_SLOW / 4));
    }

    #[test]
    fn a_beaten_king_cant_outrun_anything_that_flies_and_a_king_with_a_fleet_can() {
        // A red ship hunts the blue king from behind while the king runs along the map for
        // `secs`. Returns the nearest the hunter came, how far the king got, and the pixels
        // the king lost.
        let chase = |fleet: bool, ship: &Design, behind: i32, secs: u32| {
            let mut a = quiet();
            while a.tick < KING_GRACE {
                a.step();
            }
            let k = a.eco.as_ref().unwrap().king[0];
            let (kx, ky) = (a.ships[k].x, a.ships[k].y);
            if fleet {
                // Enough standing, and too far off for the hunter to prefer it to the king.
                a.spawn(0, 0, &slab(8, 5), kx + 600, ky - 320, 0, false);
            }
            let hunter = Design { order: Order::Hunt, ..ship.clone() };
            let f = a.spawn(1, 0, &hunter, kx - behind, ky, 1, false);
            a.ships[f].order = Order::Hunt;
            a.command(Cmd::Steer { team: 0, dx: 1, dy: 0 });
            let (mut nearest, whole) = (i32::MAX, a.ships[k].px_now());
            for _ in 0..15 * secs {
                a.step();
                if a.ships[f].alive {
                    nearest = nearest.min(cheb(a.ships[f].x, a.ships[f].y, a.ships[k].x, a.ships[k].y));
                }
            }
            (nearest, a.ships[k].x - kx, whole - a.ships[k].px_now())
        };
        // A fighter, the fastest ship in the starter deck.
        let fighter = starter_deck()[0].clone();
        let (near, ran, _) = chase(false, &fighter, 120, 12);
        assert_eq!(ran, 12 * 15 * KING_SLOW / 4);
        assert!(near <= 60, "a fighter never caught a beaten king: it got within {near} px");
        let (near, ran, hit) = chase(true, &fighter, 120, 12);
        assert_eq!(ran, 12 * 15 * KING_SPEED / 4);
        assert!(near >= 100 && hit == 0, "a fighter caught a king flying at full speed: it got within {near} px and shot off {hit}");
        // The arena's frigate, as slow as a ship with an engine can be, starting out of range of
        // the king: it brings its lance to bear too.
        let frigate = crate::pg::design::defaults()[2].clone();
        let slowest = frigate.stats().vmax;
        assert_eq!(slowest, 3, "the arena's frigate is no longer the slowest a ship can be");
        let (near, _, hit) = chase(false, &frigate, LANCE_RANGE + 60, 40);
        assert!(near <= LANCE_RANGE && hit > 0, "a frigate never caught a beaten king: it got within {near} px and shot off {hit}");
        // Even on a diagonal, where a king covers the most ground, it is the slower of the two.
        assert!(2 * KING_SLOW * KING_SLOW < slowest * slowest);
    }

    /// A blue outpost on its home rock with a bank that never runs dry, five minutes on: the
    /// arena, the factory, and the guards it has printed, in the order they were built.
    fn guarded_outpost() -> (Arena, usize, Vec<usize>) {
        let mut a = quiet();
        let k = a.eco.as_ref().unwrap().king[0];
        let (_, target) = home_rock(&a);
        a.command(Cmd::Print { team: 0, slot: 4, target: Some(target) });
        for _ in 0..15 * 300 {
            // The gift goes in the books as mined, so the audit still balances.
            let gift = (3000 - a.bank(k)).max(0);
            a.ships[k].store += gift;
            a.score[0].mined += gift;
            a.step();
        }
        let f = factory(&a).unwrap();
        let guards: Vec<usize> = (0..a.ships.len()).filter(|&i| {
            let s = &a.ships[i];
            s.alive && s.team == 0 && s.order == Order::Guard && s.role == Role::Ship && s.grow.is_none() && s.warp == 0 && s.st.thrust > 0
        }).collect();
        (a, f, guards)
    }

    fn spots(a: &Arena, ships: &[usize]) -> Vec<(i32, i32)> {
        ships.iter().map(|&i| (a.ships[i].x, a.ships[i].y)).collect()
    }

    /// Steps `secs` seconds and returns how many of `ships` moved at all.
    fn restless(a: &mut Arena, ships: &[usize], secs: u32) -> usize {
        let mut moved = vec![false; ships.len()];
        for _ in 0..15 * secs {
            let was = spots(a, ships);
            a.step();
            a.audit().unwrap();
            for (n, now) in spots(a, ships).into_iter().enumerate() {
                moved[n] |= now != was[n];
            }
        }
        moved.iter().filter(|&&m| m).count()
    }

    #[test]
    fn guards_wait_in_rings_round_their_factory_and_sit_still() {
        let (mut a, f, guards) = guarded_outpost();
        assert!(guards.len() >= 10, "only {} guards after five minutes", guards.len());
        let (fx, fy, fh) = (a.ships[f].x, a.ships[f].y, a.ships[f].half());
        // Miners fly in and out past them the whole time, and the factory prints on: not one
        // guard stirs. (Before they had posts, every one of them moved two ticks in three.)
        assert_eq!(restless(&mut a, &guards, 10), 0, "guards with nothing to do moved");
        let at = spots(&a, &guards);
        for (n, &i) in guards.iter().enumerate() {
            let s = &a.ships[i];
            // Its home is the factory, and its post is in a ring round it: outside the gap the
            // factory prints into, and no further out than the rings this many guards fill.
            assert_eq!((s.home, s.ring0), ((fx, fy), fh + POST_GAP));
            let far = isqrt(dist2(s.x, s.y, fx, fy));
            assert!(far >= fh + POST_GAP && far <= fh + POST_GAP + 2 * POST_SPACE + s.half() + 2, "{} waits {far} px from its factory", s.name);
            // Nothing is parked on top of anything else: clear water between every two hulls.
            for (m, &j) in guards.iter().enumerate().skip(n + 1) {
                let gap = cheb(at[n].0, at[n].1, at[m].0, at[m].1) - s.half() - a.ships[j].half();
                assert!(gap >= 4, "{} and {} wait {gap} px apart", s.name, a.ships[j].name);
            }
            // At its post it looks outward.
            let out = if (s.x - fx).abs() >= (s.y - fy).abs() { if s.x > fx { 1 } else { 3 } } else if s.y > fy { 2 } else { 0 };
            assert_eq!(s.facing, out, "{} at ({}, {}) from its factory faces {}", s.name, s.x - fx, s.y - fy, s.facing);
        }
        // The first ring goes all the way round the factory, not down one side of it.
        let ring: Vec<(i32, i32)> = at.iter().take(8).map(|&(x, y)| (x - fx, y - fy)).collect();
        for (sx, sy) in [(1, 0), (-1, 0), (0, 1), (0, -1)] {
            assert!(ring.iter().any(|&(x, y)| x * sx + y * sy >= fh + POST_GAP - 2), "nobody waits on side ({sx}, {sy}): {ring:?}");
        }
        // Nor does a guard stir when a ship of its own side flies right over it. (The miners
        // above put nothing to the test: a miner at work pushes nobody. This ship would.) It
        // crosses the first guard's post along the ring, from one side to the other.
        let (gx, gy) = at[0];
        let along = if (gx - fx).abs() >= (gy - fy).abs() { (0, 1) } else { (1, 0) };
        let runner = a.spawn(0, 0, &starter_deck()[0], gx - 24 * along.0, gy - 24 * along.1, 0, false);
        a.ships[runner].order = Order::Hold;
        a.ships[runner].home = (gx + 24 * along.0, gy + 24 * along.1);
        let mut nearest = i32::MAX;
        for _ in 0..15 * 4 {
            a.step();
            nearest = nearest.min(cheb(a.ships[runner].x, a.ships[runner].y, gx, gy));
            assert_eq!(spots(&a, &guards), at, "a guard was pushed off its post by a ship flying over it");
        }
        assert!(nearest <= 3, "the ship never flew over the guard: {nearest} px off at its nearest");
    }

    #[test]
    fn a_waiting_guard_gives_way_to_what_cant_move_aside_and_then_sits_still() {
        let (mut a, f, guards) = guarded_outpost();
        // The outpost prints no more: a ship being printed can't move aside either, and a guard
        // pushed over to where they come out would step clear of each one as it began.
        a.ships[f].build.clear();
        let before = spots(&a, &guards);
        // A block of hull dropped on the first guard's post: it can't move, so the guard does.
        let g = guards[0];
        let block = a.spawn(0, 4, &slab(5, 5), before[0].0, before[0].1, 0, false);
        restless(&mut a, &guards, 6);
        let reach = a.ships[block].half() + a.ships[g].half() + 3;
        assert_eq!(cheb(a.ships[g].x, a.ships[g].y, before[0].0, before[0].1), reach, "the guard under the block didn't step just clear of it");
        // Dead centre under the block, every side is as near: it takes the one away from home.
        let home = (a.ships[f].x, a.ships[f].y);
        assert!(dist2(a.ships[g].x, a.ships[g].y, home.0, home.1) > dist2(before[0].0, before[0].1, home.0, home.1), "the guard under the block stepped in towards its factory");
        assert_eq!(restless(&mut a, &guards, 6), 0, "guards moved with the block in place");
        assert_eq!(spots(&a, &guards)[1..], before[1..], "the block moved a guard it never touched");
        // The king flies in and parks on the second guard's post. Guards under it make way, the
        // rest stay put, and once it has stopped they all sit still again.
        let k = a.eco.as_ref().unwrap().king[0];
        assert!(!a.king_slowed(0));
        for _ in 0..15 * 60 {
            let (dx, dy) = (before[1].0 - a.ships[k].x, before[1].1 - a.ships[k].y);
            if dx.abs() < STEP && dy.abs() < STEP {
                break;
            }
            a.command(Cmd::Steer { team: 0, dx: if dx.abs() < STEP { 0 } else { dx.signum() }, dy: if dy.abs() < STEP { 0 } else { dy.signum() } });
            a.step();
        }
        a.command(Cmd::Steer { team: 0, dx: 0, dy: 0 });
        assert!(cheb(a.ships[k].x, a.ships[k].y, before[1].0, before[1].1) < STEP, "the king never reached the post");
        restless(&mut a, &guards, 8);
        assert_eq!(restless(&mut a, &guards, 6), 0, "guards moved with the king parked among them");
        let (kx, ky, kh) = (a.ships[k].x, a.ships[k].y, a.ships[k].half());
        let mut stayed = 0;
        for (n, &i) in guards.iter().enumerate() {
            let s = &a.ships[i];
            assert!(cheb(s.x, s.y, kx, ky) >= kh + s.half() + 3, "{} waits under the king", s.name);
            stayed += ((s.x, s.y) == before[n]) as usize;
        }
        assert!(stayed >= guards.len() - 5, "the king's arrival moved {} of {} guards", guards.len() - stayed, guards.len());
        // It flies off again: everyone it moved goes back to the post it had.
        a.command(Cmd::Steer { team: 0, dx: -1, dy: -1 });
        for _ in 0..15 * 4 {
            a.step();
        }
        a.command(Cmd::Steer { team: 0, dx: 0, dy: 0 });
        restless(&mut a, &guards, 8);
        assert_eq!(spots(&a, &guards)[1..], before[1..], "a guard didn't go back to its post when the king left");
    }

    #[test]
    fn a_guard_that_leaves_to_fight_comes_back_to_the_post_it_had() {
        let (mut a, f, guards) = guarded_outpost();
        let before = spots(&a, &guards);
        // The last fighter built has lost its guns: it has nothing to fight with, so it stays.
        let (n, g) = guards.iter().copied().enumerate().filter(|&(_, i)| a.ships[i].name == "FIGHTER").last().unwrap();
        assert!(n >= 8, "the gunless guard should be one from the second ring");
        for c in 0..a.ships[g].hp.len() {
            if a.ships[g].shape.cells[c] == Some(Part::Flak) {
                a.ships[g].hp[c] = 0;
            }
        }
        a.ships[g].dirty = true;
        a.step();
        assert!(!a.ships[g].st.armed());
        // Something to shoot, inside the guard radius and out of everyone's reach from a post.
        let (fx, fy) = (a.ships[f].x, a.ships[f].y);
        let bait = a.spawn(1, 0, &slab(3, 3), fx + 110, fy - 60, 0, false);
        let mut left = 0;
        for _ in 0..15 * 60 {
            a.step();
            left = left.max(spots(&a, &guards).iter().zip(&before).filter(|(now, was)| now != was).count());
            // With every post inside it empty, the one that stayed doesn't move up into them.
            assert_eq!((a.ships[g].x, a.ships[g].y), before[n], "a guard took the post of one that was away fighting");
            if !a.ships[bait].alive {
                break;
            }
        }
        assert!(!a.ships[bait].alive, "the guards never dealt with an enemy {} px from their factory", 110);
        assert!(left >= guards.len() / 2, "only {left} of {} guards went for it", guards.len());
        restless(&mut a, &guards, 20);
        assert_eq!(spots(&a, &guards), before, "a guard came home to a different post");
        assert_eq!(restless(&mut a, &guards, 5), 0);
    }

    #[test]
    fn guards_of_two_homes_whose_rings_cross_each_get_a_place() {
        let (mut a, f, guards) = guarded_outpost();
        // The outpost prints no more, so the only newcomers are the king's.
        a.ships[f].build.clear();
        let before = spots(&a, &guards);
        let k = a.eco.as_ref().unwrap().king[0];
        // The king parks where the first post of its own rings falls right on the outpost's
        // third guard, too far from the factory for what it prints to join the outpost's rings,
        // and prints four fighters.
        let fighter = a.ships[guards[0]].half();
        let park = (before[2].0, before[2].1 + a.ships[k].half() + POST_GAP + fighter);
        assert!(cheb(park.0, park.1, a.ships[f].x, a.ships[f].y) > POST_JOIN);
        for _ in 0..15 * 60 {
            let (dx, dy) = (park.0 - a.ships[k].x, park.1 - a.ships[k].y);
            if dx == 0 && dy == 0 {
                break;
            }
            // The last pixel or two, a tick at a time.
            a.command(Cmd::Steer { team: 0, dx: if dx.abs() < STEP { 0 } else { dx.signum() }, dy: if dy.abs() < STEP { 0 } else { dy.signum() } });
            a.step();
            if dx.abs() < STEP && dy.abs() < STEP {
                break;
            }
        }
        a.command(Cmd::Steer { team: 0, dx: 0, dy: 0 });
        a.step();
        let king = (a.ships[k].x, a.ships[k].y);
        assert!(cheb(king.0, king.1, park.0, park.1) < STEP);
        let n = a.ships.len();
        for _ in 0..4 {
            a.command(Cmd::Print { team: 0, slot: 0, target: None });
        }
        let summoned = |a: &Arena| (n..a.ships.len()).filter(|&i| a.ships[i].alive && a.ships[i].name == "FIGHTER" && a.ships[i].home == king && a.ships[i].warp == 0).collect::<Vec<_>>();
        while summoned(&a).len() < 4 {
            a.step();
            assert!(a.tick < 15 * 600, "the king never printed four fighters");
        }
        let new = summoned(&a);
        let all: Vec<usize> = guards.iter().chain(&new).copied().collect();
        restless(&mut a, &all, 12);
        assert_eq!(restless(&mut a, &all, 6), 0, "guards moved with two sets of rings crossing");
        // The new one whose post was taken sits just clear of the guard that had it, who hasn't
        // moved; and no two of them all share water.
        assert_eq!(spots(&a, &guards)[2], before[2], "a guard gave up its post to one built after it");
        let at = spots(&a, &all);
        for x in 0..all.len() {
            for y in x + 1..all.len() {
                let gap = cheb(at[x].0, at[x].1, at[y].0, at[y].1) - a.ships[all[x]].half() - a.ships[all[y]].half();
                assert!(gap >= 3, "{} and {} wait {gap} px apart", a.ships[all[x]].name, a.ships[all[y]].name);
            }
            assert!(cheb(at[x].0, at[x].1, king.0, king.1) >= a.ships[k].half() + a.ships[all[x]].half() + 3, "a guard waits under the king");
        }
        // The others go off to fight and come back. The summoned fighter that wanted the older
        // guard's post has lost its guns, so it stays: it takes the post while the other is away,
        // and gives it up again when its owner comes home.
        let late = new[0];
        for c in 0..a.ships[late].hp.len() {
            if a.ships[late].shape.cells[c] == Some(Part::Flak) {
                a.ships[late].hp[c] = 0;
            }
        }
        a.ships[late].dirty = true;
        let (fx, fy) = (a.ships[f].x, a.ships[f].y);
        let bait = a.spawn(1, 0, &slab(3, 3), fx + 100, fy - 70, 0, false);
        let mut took = false;
        for _ in 0..15 * 60 {
            a.step();
            took |= cheb(a.ships[late].x, a.ships[late].y, before[2].0, before[2].1) <= 2;
            if !a.ships[bait].alive {
                break;
            }
        }
        assert!(!a.ships[bait].alive, "the guards never dealt with the enemy");
        assert!(took, "the gunless fighter never moved onto the post left empty beside it");
        restless(&mut a, &all, 20);
        assert_eq!(spots(&a, &all), at, "a guard came back from the fight to a different place");
    }

    #[test]
    fn ships_a_king_summons_in_one_place_share_one_set_of_rings() {
        let mut a = quiet();
        let k = a.eco.as_ref().unwrap().king[0];
        a.ships[k].store = 500;
        a.score[0].mined += 500 - START_BANK;
        let fighters = |a: &Arena| (0..a.ships.len()).filter(|&i| a.ships[i].alive && a.ships[i].team == 0 && a.ships[i].name == "FIGHTER" && a.ships[i].warp == 0).collect::<Vec<_>>();
        let summon = |a: &mut Arena, n: usize| {
            a.command(Cmd::Print { team: 0, slot: 0, target: None });
            while fighters(a).len() < n {
                a.step();
                assert!(a.tick < 15 * 600, "the king never printed fighter {n}");
            }
        };
        let fly = |a: &mut Arena, dx: i32, ticks: u32| {
            a.command(Cmd::Steer { team: 0, dx, dy: 0 });
            for _ in 0..ticks {
                a.step();
            }
            a.command(Cmd::Steer { team: 0, dx: 0, dy: 0 });
            a.step();
        };
        // Three where the king starts: their home is the king's place, not the three spots they
        // came out at.
        let start = (a.ships[k].x, a.ships[k].y);
        for n in 1..=3 {
            summon(&mut a, n);
        }
        let f = fighters(&a);
        assert!(f.iter().all(|&i| (a.ships[i].home, a.ships[i].ring0) == (start, a.ships[k].half() + POST_GAP)));
        // The king shifts a little and prints a fourth: it joins the three.
        fly(&mut a, 1, (POST_JOIN * 4 / KING_SPEED) as u32 - 1);
        assert!(!a.king_slowed(0) || a.tick < KING_GRACE);
        summon(&mut a, 4);
        let f = fighters(&a);
        assert_eq!(a.ships[f[3]].home, start, "a fighter summoned {} px away started rings of its own", a.ships[k].x - start.0);
        // It flies well off and prints a fifth: that one's home is where it was summoned.
        a.spawn(0, 4, &slab(8, 5), start.0, start.1 + 150, 0, false);
        fly(&mut a, 1, 60);
        let there = (a.ships[k].x, a.ships[k].y);
        assert!(there.0 - start.0 > POST_JOIN * 4, "the king only got {} px", there.0 - start.0);
        summon(&mut a, 5);
        let f = fighters(&a);
        assert_eq!(a.ships[f[4]].home, there);
        // All five end up at posts of their own, clear of each other and of the king, and still.
        restless(&mut a, &f, 12);
        assert_eq!(restless(&mut a, &f, 5), 0, "summoned guards moved with nothing to do");
        let at = spots(&a, &f);
        for n in 0..5 {
            assert!(cheb(at[n].0, at[n].1, a.ships[k].x, a.ships[k].y) >= a.ships[k].half() + 4, "a guard waits under its king");
            for m in n + 1..5 {
                assert!(cheb(at[n].0, at[n].1, at[m].0, at[m].1) >= 8, "two summoned guards wait {} px apart", cheb(at[n].0, at[n].1, at[m].0, at[m].1));
            }
        }
    }

    #[test]
    fn the_kings_builds_arrive_by_hyperspace() {
        let mut a = quiet();
        a.command(Cmd::Print { team: 0, slot: 0, target: None });
        let n = a.ships.len();
        while a.ships.len() == n {
            a.step();
        }
        let i = n;
        assert_eq!(a.ships[i].name, "FIGHTER");
        assert_eq!(a.ships[i].warp, WARP_TICKS);
        assert_eq!(a.eco.as_ref().unwrap().tally[0].summoned, 1);
        for _ in 0..WARP_TICKS {
            a.step();
        }
        assert_eq!(a.ships[i].warp, 0);
    }

    #[test]
    fn ai_matches_end_and_replay_exactly() {
        let run = || {
            let mut a = Arena::for_match(2, [starter_deck(), starter_deck()], [true, true]);
            while a.over.is_none() {
                a.step();
            }
            (a.over, a.tick, a.score[0].mined, a.score[1].mined)
        };
        let first = run();
        assert!(first.0.is_some() && first.1 < MATCH_LIMIT, "{first:?}");
        assert_eq!(run(), first, "the same seed played differently");
    }

    /// Living pixels two ships have in the same place.
    fn shared(a: &Arena, i: usize, j: usize) -> usize {
        let (si, sj) = (&a.ships[i], &a.ships[j]);
        (0..si.hp.len()).filter(|&c| si.hp[c] > 0 && { let (x, y) = si.cell_xy(c); sj.cell_at(x, y).is_some_and(|d| sj.hp[d] > 0) }).count()
    }

    #[test]
    fn one_side_cant_plant_a_factory_on_top_of_another() {
        // Blue grows a yard, then sends a second seed to the very same rock pixel. The second
        // comes down on the nearest rock the first isn't on and grows up against it, not through
        // it: stacking factories on one spot would otherwise be free armour.
        let mut yard = Design::from_rows("YARD", Order::Hold, &["hhhhhhh", "hfffffh", "hfhhhfh", "hfhchfh", "hfhhhfh", "hfffffh", "hhhhhhh"]);
        yard.seeded = true;
        let mut blue = starter_deck();
        blue[4] = yard;
        let mut a = Arena::for_match(1, [blue, starter_deck()], [false, false]);
        let (_, target) = home_rock(&a);
        let yards = |a: &Arena| -> Vec<usize> { (0..a.ships.len()).filter(|&i| a.ships[i].alive && a.ships[i].team == 0 && a.ships[i].role == Role::Ship && a.ships[i].name == "YARD").collect() };
        a.command(Cmd::Print { team: 0, slot: 4, target: Some(target) });
        while yards(&a).first().map_or(true, |&y| a.ships[y].px_now() < 49) {
            a.step();
            assert!(a.tick < 15 * 120, "the first yard never finished growing");
        }
        a.command(Cmd::Print { team: 0, slot: 4, target: Some(target) });
        let mut seen = 0;
        for _ in 0..15 * 90 {
            a.step();
            let y = yards(&a);
            if y.len() == 2 {
                seen += 1;
                assert_eq!(shared(&a, y[0], y[1]), 0, "the two yards share pixels at tick {}", a.tick);
            }
        }
        assert!(seen > 15 * 20, "the second seed never came down");
        let y = yards(&a);
        let (first, second) = (&a.ships[y[0]], &a.ships[y[1]]);
        assert_eq!(first.px_now(), 49, "the first yard should be whole");
        assert!(cheb(first.x, first.y, second.x, second.y) <= 12, "the second yard should have come down beside the first");
        assert!(second.px_now() > 5, "the second should have grown what it had room for: {} px", second.px_now());
        println!("second yard: {} of 49 px, {} px from the first", second.px_now(), cheb(first.x, first.y, second.x, second.y));
    }

    #[test]
    fn a_bigger_ship_flying_over_a_hulk_doesnt_push_it() {
        // Red's king, far bigger than a little plated hulk with no engines, is steered straight
        // over it. The hulk can't move itself, so it stays exactly where it is while the king
        // passes over. It used to be pushed out from under the king a pixel or two a tick.
        let hulk = Design::from_rows("HULK", Order::Hold, &["ppppp", "pfffp", "pfcfp", "pfffp", "ppppp"]);
        let mut a = Arena::for_match(1, [starter_deck(), starter_deck()], [false, false]);
        let k = a.eco.as_ref().unwrap().king[1];
        let (kx, ky) = (a.ships[k].x, a.ships[k].y);
        let y = a.spawn(0, 0, &hulk, kx - 40, ky, 0, false);
        let at = (a.ships[y].x, a.ships[y].y);
        a.command(Cmd::Steer { team: 1, dx: -1, dy: 0 });
        let mut over = 0;
        for _ in 0..40 {
            a.step();
            if !a.ships[y].alive {
                break;
            }
            assert_eq!((a.ships[y].x, a.ships[y].y), at, "the hulk was moved at tick {}", a.tick);
            over += (shared(&a, y, k) > 0) as i32;
        }
        assert!(over >= 3, "the king never flew over the hulk");
    }

    #[test]
    fn nothing_flies_through_a_planted_factory() {
        // The same little yard, planted this time. Red's king is steered straight at it and stops
        // against its plate: it never shares a pixel with it, however much bigger it is, and the
        // yard doesn't move. Steered diagonally instead, the king slides along the plate and past.
        let yard = Design::from_rows("YARD", Order::Hold, &["ppppp", "pfffp", "pfcfp", "pfffp", "ppppp"]);
        for slide in [false, true] {
            let mut a = Arena::for_match(1, [starter_deck(), starter_deck()], [false, false]);
            let k = a.eco.as_ref().unwrap().king[1];
            let (kx, ky) = (a.ships[k].x, a.ships[k].y);
            let y = a.spawn(0, 0, &yard, kx - 40, ky, 0, false);
            a.ships[y].rooted = true;
            let at = (a.ships[y].x, a.ships[y].y);
            a.command(Cmd::Steer { team: 1, dx: -1, dy: if slide { -1 } else { 0 } });
            let (mut held, mut last) = (0, (kx, ky));
            for _ in 0..40 {
                a.step();
                if !a.ships[y].alive {
                    break;
                }
                assert_eq!((a.ships[y].x, a.ships[y].y), at, "the yard was moved at tick {}", a.tick);
                assert_eq!(shared(&a, y, k), 0, "the king is inside the yard at tick {}", a.tick);
                let now = (a.ships[k].x, a.ships[k].y);
                held += (now == last) as i32;
                last = now;
            }
            if slide {
                assert!(a.ships[k].x < at.0, "the king should have slid past the yard: it's at {:?}, the yard at {at:?}", last);
            } else {
                assert!(a.ships[y].alive && held >= 10, "the king should have been held at the yard: held {held} ticks");
                assert!(a.ships[k].x > at.0 && a.ships[k].x - at.0 <= 14, "the king should be right up against the yard: {} px off", a.ships[k].x - at.0);
            }
        }
    }

    #[test]
    fn a_seeded_warship_grows_out_of_a_rock_and_flies_off() {
        // A seeded design may have engines and guns. It grows out of its rock like a factory,
        // planted and solid while it grows; once it's whole it lifts off and is a ship like any
        // other.
        let mut blue = starter_deck();
        let mut d = blue[0].clone();
        d.name = "SPROUT".into();
        d.seeded = true;
        d.order = Order::Hunt;
        assert!(d.stats().thrust > 0, "the test needs a design that can fly");
        blue[4] = d;
        let mut a = Arena::for_match(1, [blue, starter_deck()], [false, false]);
        let (_, target) = home_rock(&a);
        a.command(Cmd::Print { team: 0, slot: 4, target: Some(target) });
        let sprout = |a: &Arena| (0..a.ships.len()).find(|&i| a.ships[i].alive && a.ships[i].role == Role::Ship && a.ships[i].name == "SPROUT");
        let (mut landed, mut flew) = (None, false);
        for _ in 0..15 * 180 {
            a.step();
            let Some(i) = sprout(&a) else { continue };
            let s = &a.ships[i];
            if s.grow.is_some() {
                assert!(s.rooted, "growing but not planted");
                let at = *landed.get_or_insert((s.x, s.y));
                assert_eq!((s.x, s.y), at, "it moved while it was growing");
            } else if let Some(at) = landed {
                if cheb(s.x, s.y, at.0, at.1) > 10 {
                    assert!(!s.rooted, "flying but still planted");
                    assert_eq!(s.px_now(), s.px_full(), "it should have flown off whole");
                    flew = true;
                    break;
                }
            }
        }
        assert!(landed.is_some(), "the seed never came down");
        assert!(flew, "the grown ship never flew off");
    }

    #[test]
    fn two_sides_planting_on_one_rock_never_share_a_pixel() {
        // Both sides send a seed to the same rock pixel, with unarmed factories so nothing shoots
        // the seeds down. Blue gets there first. Red can't reach its pixel (blue's factory is on
        // it), comes down on the nearest free rock, grows round blue's factory, not through it,
        // and prints with what it managed to grow.
        let yard = |build: Vec<usize>| {
            let mut d = Design::from_rows("YARD", Order::Hold, &["hhhhh", "hfffh", "hfcfh", "hfffh", "hhhhh"]);
            d.seeded = true;
            d.build = build;
            d
        };
        let (mut blue, mut red) = (starter_deck(), starter_deck());
        blue[4] = yard(Vec::new());
        red[4] = yard(vec![0]);
        let mut a = Arena::for_match(1, [blue, red], [false, false]);
        let (_, target) = home_rock(&a);
        a.command(Cmd::Print { team: 0, slot: 4, target: Some(target) });
        a.command(Cmd::Print { team: 1, slot: 4, target: Some(target) });
        let both = |a: &Arena| -> Option<(usize, usize)> {
            let f = |t: usize| (0..a.ships.len()).find(|&i| a.ships[i].alive && a.ships[i].team == t && a.ships[i].role == Role::Ship && a.ships[i].node);
            Some((f(0)?, f(1)?))
        };
        let mut seen = 0;
        for _ in 0..15 * 120 {
            a.step();
            if let Some((b, r)) = both(&a) {
                seen += 1;
                assert_eq!(shared(&a, b, r), 0, "the two factories share pixels at tick {}", a.tick);
            }
        }
        assert!(seen > 15 * 30, "red's seed never came down beside blue's factory");
        let (b, r) = both(&a).unwrap();
        assert_eq!(a.ships[b].px_now(), a.ships[b].px_full(), "blue got there first and should be whole");
        let red = &a.ships[r];
        assert!(red.px_now() > 1 && red.px_now() < red.px_full(), "red should be wedged in part-grown: {} of {}", red.px_now(), red.px_full());
        assert!(cheb(red.x, red.y, a.ships[b].x, a.ships[b].y) <= 8, "red should be right beside blue");
        // Wedged with nowhere to grow, its factory pixels print instead of waiting for ever.
        assert!(red.st.factory > 0 && red.making != Making::None, "red's wedged factory never started on its build order: {:?}", red.making);
    }

    #[test]
    fn a_seed_whose_rock_is_gone_finds_another() {
        let mut a = quiet();
        let (r, target) = home_rock(&a);
        a.command(Cmd::Print { team: 0, slot: 4, target: Some(target) });
        // The seed is out and flying: its rock vanishes (mined or eaten by someone else).
        while !a.ships.iter().any(|s| s.alive && s.role == Role::Seed && s.warp == 0) {
            a.step();
        }
        let rock = a.eco.as_ref().unwrap().rocks[r].clone();
        for &(x, y) in &rock.px {
            let k = (y * a.w + x) as usize;
            a.rock[k] = false;
        }
        for _ in 0..15 * 60 {
            a.step();
        }
        assert_eq!(a.eco.as_ref().unwrap().tally[0].planted, 1, "the seed never came down anywhere");
        let f = factory(&a).unwrap();
        assert!(cheb(a.ships[f].x, a.ships[f].y, rock.cx, rock.cy) > 30, "it planted on the rock that had gone");
        assert!(!a.ships.iter().any(|s| s.alive && s.role == Role::Seed), "a seed is still hanging about");
    }

    #[test]
    fn the_books_balance_and_structures_keep_apart() {
        // Six minutes of AI against AI: every tick, what each side has mined plus its starting
        // bank equals what it holds, has spent on printing and lost with destroyed nodes. No
        // ship is off the map, and no two sides' structures share a pixel.
        let mut a = Arena::for_match(3, [starter_deck(), starter_deck()], [true, true]);
        while a.over.is_none() && a.tick < 15 * 360 {
            a.step();
            let e = a.eco.as_ref().unwrap();
            for t in 0..2 {
                let stored: i32 = a.ships.iter().filter(|s| s.team == t).map(|s| s.store).sum();
                assert_eq!(START_BANK + a.score[t].mined + e.made[t], stored + e.spent[t] + e.spilt[t], "team {t}'s books are off at tick {}", a.tick);
            }
            for s in a.ships.iter().filter(|s| s.alive) {
                let (l, t, r, b) = s.bbox();
                assert!(l >= 0 && t >= 0 && r < a.w && b < a.h && s.store >= 0, "{} is off the map or overdrawn", s.name);
            }
            if a.tick % 15 == 0 {
                let st: Vec<usize> = (0..a.ships.len()).filter(|&i| a.ships[i].alive && a.ships[i].role == Role::Ship && (a.ships[i].node || a.ships[i].grow.is_some())).collect();
                for (k, &i) in st.iter().enumerate() {
                    for &j in &st[k + 1..] {
                        if a.ships[i].team != a.ships[j].team {
                            assert_eq!(shared(&a, i, j), 0, "{} and {} share pixels", a.ships[i].name, a.ships[j].name);
                        }
                    }
                }
            }
        }
    }

    #[test]
    fn factories_short_of_pixels_take_turns() {
        // Two small yards in one network, one printing fighters and one lancers (7 px each), with
        // nothing coming in but the pixels the king makes: less than the two could print. Each
        // should get half. Before turns, whichever was built first took the lot and the other
        // printed nothing.
        let yard = |name: &str, build: Vec<usize>| {
            let mut d = Design::from_rows(name, Order::Hold, &["hhhhh", "hfffh", "hfcfh", "hfffh", "hhhhh"]);
            d.seeded = true;
            d.build = build;
            d
        };
        let mut deck = starter_deck();
        deck[4] = yard("YARD A", vec![0]);
        deck[5] = yard("YARD B", vec![1]);
        // Hold-order copies, so what's printed stays put and nothing wanders off to fight.
        deck[0].order = Order::Hold;
        deck[1].order = Order::Hold;
        let mut a = Arena::for_match(1, [deck, starter_deck()], [false, false]);
        let e = a.eco.as_ref().unwrap();
        let (kx, ky) = KING_STARTS[0];
        let mut near: Vec<usize> = (0..e.rocks.len()).collect();
        near.sort_by_key(|&r| dist2(e.rocks[r].cx, e.rocks[r].cy, kx, ky));
        let spot = |r: usize| e.rocks[r].px.iter().copied().min_by_key(|&(x, y)| dist2(x, y, e.rocks[r].cx, e.rocks[r].cy)).unwrap();
        let (t0, t1) = (spot(near[0]), spot(near[1]));
        a.command(Cmd::Print { team: 0, slot: 4, target: Some(t0) });
        a.command(Cmd::Print { team: 0, slot: 5, target: Some(t1) });
        let yards = |a: &Arena| a.ships.iter().filter(|s| s.alive && s.team == 0 && s.node && s.role == Role::Ship && s.grow.is_none()).count();
        while yards(&a) < 2 {
            a.step();
            assert!(a.tick < 15 * 120, "the two yards never finished growing");
        }
        // 16 factory pixels between them: together they can print more than the king makes.
        assert!(16 * KING_MAKES_EVERY as i32 > PRINT_TICKS, "the yards aren't short of pixels at this pace");
        let made = |a: &Arena, name: &str| a.ships.iter().filter(|s| s.team == 0 && s.name == name && s.maker.is_none() && s.grow.is_none()).count() as i32;
        let (f0, l0) = (made(&a, "FIGHTER"), made(&a, "LANCER"));
        for i in 0..a.ships.len() {
            a.ships[i].store = 0;
        }
        let fed0 = a.eco.as_ref().unwrap().made[0];
        for _ in 0..15 * 600 {
            a.step();
        }
        let fed = a.eco.as_ref().unwrap().made[0] - fed0;
        let (f, l) = (made(&a, "FIGHTER") - f0, made(&a, "LANCER") - l0);
        // They share by the pixel: each yard has spent about half of what came in, whatever its
        // ship costs.
        let cost = |slot: usize| a.eco.as_ref().unwrap().cards[0][slot].cost;
        let (pf, pl, dear) = (f * cost(0), l * cost(1), cost(0).max(cost(1)));
        println!("two yards fed {fed} px: {f} fighters ({pf} px), {l} lancers ({pl} px)");
        assert!(pf + pl >= fed - 2 * dear && (pf - pl).abs() <= 2 * dear, "{fed} px should be about {} px of each: {pf} px of fighters, {pl} px of lancers", fed / 2);
    }

    // ----- Match round 2 -----

    #[test]
    fn a_seed_is_paid_for_in_full_or_not_sold() {
        let mut a = quiet();
        let (_, target) = home_rock(&a);
        let k = a.eco.as_ref().unwrap().king[0];
        let price = a.eco.as_ref().unwrap().cards[0][4].price();
        assert_eq!(price, starter_deck()[4].stats().cost, "a seed costs what it grows into");
        assert_eq!(a.can_print(0, 4, Some(target)), Ok(price));
        a.command(Cmd::Print { team: 0, slot: 4, target: Some(target) });
        a.step();
        // Paid in one go, and the seed is on its way by hyperspace.
        assert_eq!(a.bank(k), START_BANK - price);
        assert_eq!(a.ships.iter().filter(|s| s.alive && s.team == 0 && s.role == Role::Seed && s.warp > 0).count(), 1);
        assert!(a.eco.as_ref().unwrap().queue[0].is_empty(), "a seed doesn't wait in the king's queue");
        // The shipyard costs more than is left: it's refused, and an order given anyway is dropped
        // without touching the bank or the log.
        let dear = a.eco.as_ref().unwrap().cards[0][5].price();
        assert!(dear > a.bank(k));
        assert!(a.can_print(0, 5, Some(target)).is_err());
        let (bank, logged) = (a.bank(k), a.eco.as_ref().unwrap().log.len());
        a.command(Cmd::Print { team: 0, slot: 5, target: Some(target) });
        a.step();
        assert_eq!((a.bank(k), a.eco.as_ref().unwrap().log.len()), (bank, logged));
        assert_eq!(a.ships.iter().filter(|s| s.alive && s.role == Role::Seed).count(), 1);
        a.audit().unwrap();
    }

    #[test]
    fn the_king_makes_a_pixel_now_and_then() {
        // Nothing mined, nothing spent: after a minute each bank is up by what its king made.
        let mut a = quiet();
        for _ in 0..15 * 60 {
            a.step();
            a.audit().unwrap();
        }
        let e = a.eco.as_ref().unwrap();
        assert_eq!(e.made, [(15 * 60 / KING_MAKES_EVERY) as i32; 2]);
        assert_eq!(a.bank(e.king[0]), START_BANK + e.made[0]);
    }

    #[test]
    fn every_ship_of_a_design_takes_its_new_order() {
        // Three fighters arrive on GUARD (the starter deck's order) and stay where they arrived
        // while the king flies off. On ESCORT they catch it up and stay with it. On HUNT they go
        // for the enemy, and a fighter printed after the switch is a hunter from the start.
        let mut a = quiet();
        let k = a.eco.as_ref().unwrap().king[0];
        a.ships[k].store = 500;
        a.score[0].mined += 500 - START_BANK;
        for _ in 0..3 {
            a.command(Cmd::Print { team: 0, slot: 0, target: None });
        }
        let fighters = |a: &Arena| (0..a.ships.len()).filter(|&i| a.ships[i].alive && a.ships[i].team == 0 && a.ships[i].name == "FIGHTER" && a.ships[i].warp == 0).collect::<Vec<_>>();
        while fighters(&a).len() < 3 {
            a.step();
            assert!(a.tick < 15 * 120, "the king never printed three fighters");
        }
        assert!(fighters(&a).iter().all(|&i| a.ships[i].order == Order::Guard));
        // Three fighters are not a fleet: something more standing, so the king flies at full speed.
        let (kx, ky) = (a.ships[k].x, a.ships[k].y);
        a.spawn(0, 4, &slab(8, 5), kx - 100, ky + 100, 0, false);
        assert!(!a.king_slowed(0));
        let run = |a: &mut Arena, secs: u32| {
            for _ in 0..15 * secs {
                a.step();
                a.audit().unwrap();
            }
        };
        let far = |a: &Arena, i: usize| isqrt(dist2(a.ships[i].x, a.ships[i].y, a.ships[k].x, a.ships[k].y));
        // The king flies 300 px up the map; the guards stay behind.
        a.command(Cmd::Steer { team: 0, dx: 0, dy: -1 });
        run(&mut a, 300 * 4 / (KING_SPEED as u32 * 15) + 1);
        a.command(Cmd::Steer { team: 0, dx: 0, dy: 0 });
        run(&mut a, 5);
        assert!(fighters(&a).iter().all(|&i| far(&a, i) > 200), "a guard left its post to follow the king");
        // ESCORT: all three form up on the king, and follow when it moves on.
        a.command(Cmd::Order { team: 0, slot: 0, order: Order::Escort });
        run(&mut a, 20);
        assert!(fighters(&a).iter().all(|&i| a.ships[i].order == Order::Escort && far(&a, i) < 40), "an escort isn't with the king");
        a.command(Cmd::Steer { team: 0, dx: 1, dy: 0 });
        run(&mut a, 6);
        a.command(Cmd::Steer { team: 0, dx: 0, dy: 0 });
        run(&mut a, 8);
        assert!(fighters(&a).iter().all(|&i| far(&a, i) < 40), "an escort lost the king");
        // HUNT: off they go, and the next one printed is a hunter from the start.
        a.command(Cmd::Order { team: 0, slot: 0, order: Order::Hunt });
        run(&mut a, 8);
        let f = fighters(&a);
        assert!(f.len() == 3 && f.iter().all(|&i| a.ships[i].order == Order::Hunt && far(&a, i) > 150), "a hunter stayed at home");
        a.command(Cmd::Print { team: 0, slot: 0, target: None });
        let n = a.ships.len();
        while a.ships.len() == n {
            a.step();
            assert!(a.tick < 15 * 300, "the fourth fighter never arrived");
        }
        assert_eq!((a.ships[n].name.as_str(), a.ships[n].order), ("FIGHTER", Order::Hunt));
        assert_eq!(a.eco.as_ref().unwrap().tally[0].orders, 2);
        // The other side's fighters, and this side's other designs, were never touched.
        let e = a.eco.as_ref().unwrap();
        assert_eq!(e.orders[1][0], Order::Guard);
        assert_eq!(e.orders[0][1], Order::Guard);
    }

    #[test]
    fn a_repeated_card_keeps_printing_and_a_cancelled_one_is_refunded() {
        let mut a = quiet();
        let k = a.eco.as_ref().unwrap().king[0];
        // Right-click on the fighter with nothing queued: one is queued, on repeat.
        a.command(Cmd::Repeat { team: 0, slot: 0, at: None });
        let summoned = |a: &Arena| a.eco.as_ref().unwrap().tally[0].summoned;
        while summoned(&a) < 3 {
            a.step();
            a.audit().unwrap();
            assert!(a.tick < 15 * 240, "the king stopped repeating");
        }
        let q = &a.eco.as_ref().unwrap().queue[0];
        assert!(q.len() == 1 && q[0].slot == 0 && q[0].repeat, "{q:?}");
        // Again: it finishes the one it's on and stops.
        a.command(Cmd::Repeat { team: 0, slot: 0, at: None });
        a.step();
        assert!(!a.eco.as_ref().unwrap().queue[0][0].repeat);
        while !a.eco.as_ref().unwrap().queue[0].is_empty() {
            a.step();
            assert!(a.tick < 15 * 300);
        }
        assert_eq!(summoned(&a), 4);
        // A frigate, dropped part-way: what was paid comes back, and nothing arrives.
        a.command(Cmd::Print { team: 0, slot: 2, target: None });
        while a.eco.as_ref().unwrap().queue[0].first().map_or(0, |j| j.paid) < 10 {
            a.step();
        }
        let (bank, paid) = (a.bank(k), a.eco.as_ref().unwrap().queue[0][0].paid);
        a.command(Cmd::Cancel { team: 0, at: 0, slot: 2 });
        a.step();
        a.audit().unwrap();
        assert!(a.eco.as_ref().unwrap().queue[0].is_empty());
        assert!(a.bank(k) >= bank + paid, "the king got back {} of the {paid} px it had paid", a.bank(k) - bank);
        assert_eq!(summoned(&a), 4);
        // Three in the queue. The one in the middle is dropped: the other two stay, and the one
        // being printed keeps what it has paid.
        let slots = |a: &Arena| a.eco.as_ref().unwrap().queue[0].iter().map(|j| j.slot).collect::<Vec<_>>();
        for slot in [2, 1, 0] {
            a.command(Cmd::Print { team: 0, slot, target: None });
        }
        while a.eco.as_ref().unwrap().queue[0].first().map_or(0, |j| j.paid) < 5 {
            a.step();
        }
        a.command(Cmd::Cancel { team: 0, at: 1, slot: 1 });
        a.step();
        a.audit().unwrap();
        assert_eq!(slots(&a), vec![2, 0]);
        assert!(a.eco.as_ref().unwrap().queue[0][0].paid >= 5, "dropping a job that waited cost the one being printed its pixels");
        // A command that names a place the queue has moved up from drops the job it meant: the
        // fighter, now second and not third. One that names a design not queued drops nothing.
        a.command(Cmd::Cancel { team: 0, at: 2, slot: 0 });
        a.command(Cmd::Cancel { team: 0, at: 1, slot: 6 });
        a.step();
        a.audit().unwrap();
        assert_eq!(slots(&a), vec![2]);
        a.command(Cmd::Cancel { team: 0, at: 0, slot: 2 });
        a.step();
        assert!(slots(&a).is_empty());
        // A seeded card can't repeat: there's no rock to send every seed to.
        a.command(Cmd::Repeat { team: 0, slot: 4, at: None });
        a.step();
        assert!(a.eco.as_ref().unwrap().queue[0].is_empty());
    }

    /// A match with blue's fighters on HUNT in groups of three, and blue's king's place.
    fn match_with_groups_of_three() -> (Arena, Design, (i32, i32)) {
        let mut deck = starter_deck();
        deck[0].order = Order::Hunt;
        deck[0].group = 3;
        let a = Arena::for_match(5, [deck.clone(), starter_deck()], [false, false]);
        let k = a.eco().king[0];
        let at = (a.ships[k].x, a.ships[k].y);
        (a, deck[0].clone(), at)
    }

    #[test]
    fn ships_of_a_design_that_leaves_in_groups_wait_until_the_group_is_there() {
        let (mut a, fighter, (kx, ky)) = match_with_groups_of_three();
        let far = |a: &Arena, f: usize| cheb(a.ships[f].x, a.ships[f].y, a.ships[f].home.0, a.ships[f].home.1);
        // Two of them wait where they were built, however long, and guard there.
        let two = [a.spawn(0, 0, &fighter, kx + 40, ky, 1, false), a.spawn(0, 0, &fighter, kx + 40, ky + 14, 1, false)];
        for _ in 0..15 * 20 {
            a.step();
        }
        for f in two {
            assert!(a.ships[f].waiting && a.ships[f].doing == Doing::Muster, "{:?}", a.ships[f].doing);
            assert!(far(&a, f) <= 6, "it wandered {} px from where it was built", far(&a, f));
        }
        // A third makes the group, and all three go.
        let third = a.spawn(0, 0, &fighter, kx + 40, ky - 14, 1, false);
        a.step();
        let three = [two[0], two[1], third];
        assert!(three.iter().all(|&f| !a.ships[f].waiting));
        for _ in 0..15 * 10 {
            a.step();
        }
        for f in three {
            assert!(far(&a, f) > GUARD_RADIUS, "ten seconds on it has gone {} px", far(&a, f));
        }
        // One built after they've gone waits for a group of its own.
        let late = a.spawn(0, 0, &fighter, kx + 40, ky, 1, false);
        for _ in 0..15 * 5 {
            a.step();
        }
        assert!(a.ships[late].waiting && far(&a, late) <= 6);
        // Told SAB, the three out stay out, even one that's strayed from the others, and the one
        // waiting still waits.
        (a.ships[three[0]].x, a.ships[three[0]].y) = (kx + 400, ky);
        assert!(three[1..].iter().all(|&f| cheb(a.ships[f].x, a.ships[f].y, kx + 400, ky) > MUSTER));
        a.command(Cmd::Order { team: 0, slot: 0, order: Order::Sab });
        a.step();
        assert!(three.iter().all(|&f| !a.ships[f].waiting) && a.ships[late].waiting);
        // Called home and sent out again, they gather first. Four of them, but no three together:
        // nobody leaves.
        a.command(Cmd::Order { team: 0, slot: 0, order: Order::Guard });
        a.step();
        a.command(Cmd::Order { team: 0, slot: 0, order: Order::Hunt });
        a.step();
        assert!(three.iter().all(|&f| a.ships[f].waiting) && a.ships[late].waiting, "ships more than {MUSTER} px apart left as a group");
    }

    #[test]
    fn a_group_already_gathered_leaves_on_the_tick_its_told_to_and_ones_leave_at_once() {
        let (mut a, fighter, (kx, ky)) = match_with_groups_of_three();
        a.command(Cmd::Order { team: 0, slot: 0, order: Order::Guard });
        let three = [a.spawn(0, 0, &fighter, kx + 40, ky, 1, false), a.spawn(0, 0, &fighter, kx + 40, ky + 14, 1, false), a.spawn(0, 0, &fighter, kx + 40, ky - 14, 1, false)];
        for _ in 0..15 {
            a.step();
        }
        assert!(three.iter().all(|&f| a.ships[f].order == Order::Guard));
        a.command(Cmd::Order { team: 0, slot: 0, order: Order::Hunt });
        a.step();
        assert!(three.iter().all(|&f| !a.ships[f].waiting && a.ships[f].doing != Doing::Muster));
        // Lancers, in the same deck, leave in ones: each is off the moment it exists.
        let lancer = a.eco().deck[0][1].clone();
        assert_eq!(lancer.group, 1);
        let one = a.spawn(0, 1, &lancer, kx + 60, ky, 1, false);
        assert!(!a.ships[one].waiting);
    }

    #[test]
    fn sab_passes_a_king_by_for_a_miner_and_goes_on_the_wire_like_any_order() {
        let mut deck = starter_deck();
        deck[0].order = Order::Sab;
        let mut a = Arena::for_match(5, [deck.clone(), starter_deck()], [false, false]);
        let k = a.eco().king[1];
        let (kx, ky) = (a.ships[k].x, a.ships[k].y);
        // A raider 60 px from the red king, and a red miner 200 px beyond: the miner.
        let raider = a.spawn(0, 0, &deck[0], kx - 60, ky, 1, false);
        let miner = a.spawn(1, 3, &deck[3], kx - 260, ky, 1, false);
        a.step();
        assert_eq!(a.ships[raider].prey, Some(miner));
        // With no miner or factory to go for, the king is the nearest thing to hunt.
        a.ships[miner].alive = false;
        a.step();
        assert_eq!(a.ships[raider].prey, Some(k));
        let c = Cmd::Order { team: 1, slot: 2, order: Order::Sab };
        assert_eq!(Cmd::from_wire(&c.to_wire()), Some(c));
        assert_eq!(Cmd::from_wire("O 1 2 6"), None);
    }

    #[test]
    fn a_repeating_job_goes_on_the_end_of_the_queue_and_any_job_can_be_set_repeating() {
        let mut a = quiet();
        let jobs = |a: &Arena| a.eco.as_ref().unwrap().queue[0].iter().map(|j| (j.slot, j.repeat)).collect::<Vec<_>>();
        let give = |a: &mut Arena, c: Cmd| {
            a.command(c);
            a.step();
        };
        // Two fighters asked for, then "fighters, on repeat" (the card, right-clicked): a third
        // job, on the end. The two asked for first are still just two.
        for _ in 0..2 {
            give(&mut a, Cmd::Print { team: 0, slot: 0, target: None });
        }
        give(&mut a, Cmd::Repeat { team: 0, slot: 0, at: None });
        assert_eq!(jobs(&a), vec![(0, false), (0, false), (0, true)]);
        // A lancer, then "the last lancer repeats", given before the lancer's job could be seen:
        // LAST_JOB finds it when the command lands.
        a.command(Cmd::Print { team: 0, slot: 1, target: None });
        give(&mut a, Cmd::Repeat { team: 0, slot: 1, at: Some(LAST_JOB) });
        assert_eq!(jobs(&a), vec![(0, false), (0, false), (0, true), (1, true)]);
        // One job in the queue picked out to repeat, and to stop: the second fighter, and no other.
        give(&mut a, Cmd::Repeat { team: 0, slot: 0, at: Some(1) });
        assert_eq!(jobs(&a), vec![(0, false), (0, true), (0, true), (1, true)]);
        give(&mut a, Cmd::Repeat { team: 0, slot: 0, at: Some(1) });
        assert_eq!(jobs(&a), vec![(0, false), (0, false), (0, true), (1, true)]);
        // The card again: every fighter job stops repeating, and none is added or dropped.
        give(&mut a, Cmd::Repeat { team: 0, slot: 0, at: None });
        assert_eq!(jobs(&a), vec![(0, false), (0, false), (0, false), (1, true)]);
        // A place with no job of that design at or before it, and a seeded card: nothing happens.
        give(&mut a, Cmd::Repeat { team: 0, slot: 1, at: Some(2) });
        give(&mut a, Cmd::Repeat { team: 0, slot: 4, at: None });
        assert_eq!(jobs(&a), vec![(0, false), (0, false), (0, false), (1, true)]);
        // A repeating job, finished, goes to the back: what was asked for once comes before
        // its next turn.
        give(&mut a, Cmd::Repeat { team: 0, slot: 0, at: Some(0) });
        let k = a.eco.as_ref().unwrap().king[0];
        a.ships[k].store += 100;
        a.score[0].mined += 100;
        while jobs(&a).first() == Some(&(0, true)) {
            a.step();
            assert!(a.tick < 15 * 60, "the king never finished its first fighter");
        }
        assert_eq!(jobs(&a), vec![(0, false), (0, false), (1, true), (0, true)]);
    }

    #[test]
    fn miners_keep_working_with_a_fleet_parked_on_their_rock() {
        // The AI's warships gather on GUARD round the factory that printed them, which stands on
        // the rock its miners work. Ships in flight keep apart, but a miner still has to reach
        // rock under a parked friend or under its own factory: before that was so, mining
        // stopped at 72 px a side by 4:00. (The rule itself is tested just below; this is the
        // floor under a whole economy, which runs at about 250 px by 4:00 on the slower rules of
        // 2026-11-11, and ran at 440 before them.)
        let mut a = Arena::for_match(1, [starter_deck(), starter_deck()], [true, true]);
        while a.tick < 15 * 240 {
            a.step();
        }
        assert!(a.score[0].mined >= 150 && a.score[1].mined >= 150, "mined {} and {} px by 4:00", a.score[0].mined, a.score[1].mined);
    }

    #[test]
    fn a_miner_bites_rock_under_a_friend_parked_on_it() {
        // A block of the miner's own side sits on the middle of a rock, as a factory grown there
        // or a fleet parked there would, and the miner starts on top of it. Friends keep apart,
        // but not a miner at its rock pixel: it stays put and takes a bite every 2.7 s, each one
        // from under the block. Pushed off like any other ship, it never got one.
        let mut a = quiet();
        let (_, target) = home_rock(&a);
        let block = a.spawn(0, 4, &slab(9, 9), target.0, target.1, 0, false);
        let m = a.spawn(0, 3, &starter_deck()[3], target.0, target.1, 0, false);
        let (l, t, r, b) = a.ships[block].bbox();
        let under = |a: &Arena| (t..=b).flat_map(|y| (l..=r).map(move |x| (x, y))).filter(|&(x, y)| a.rock_at(x, y)).count() as i32;
        let before = under(&a);
        assert!(before >= 40, "the block should sit on rock: {before} px under it");
        for _ in 0..15 * 12 {
            a.step();
        }
        let got = a.ships[m].carry;
        assert!(got >= 3, "a miner on a rock under a parked friend took {got} bites in 12 s");
        assert_eq!(before - under(&a), got, "its bites didn't come from under the block");
    }

    #[test]
    fn full_miners_get_home_in_the_time_the_flight_takes() {
        // A whole economy at work: a side that plants factories and prints miners without a
        // pause, against the AI, for six minutes. Miners heap up on the rock faces, and each one
        // that fills has to get home past the rest. Every trip home to a drop-off that stayed
        // where it was, with nothing solid in the way, takes the straight flight at the miner's
        // speed and no more than 3 s on top. (When friends pushed a full miner like any other
        // ship, the heaps held some of them back: 22 s at the worst here, and over a minute in
        // longer matches.)
        let mut a = Arena::for_bots(1, [starter_deck(), starter_deck()], [Bot::Boomer, Bot::Ai]);
        // For each miner on its way home: when it set off, how far it had to go, where to, and
        // whether anything solid stopped it on the way.
        let mut out: Vec<Option<(u32, i32, usize, (i32, i32), bool)>> = Vec::new();
        let (mut trips, mut worst) = (0, 0);
        while a.over.is_none() && a.tick < 15 * 60 * 6 {
            a.step();
            out.resize(a.ships.len(), None);
            for i in 0..a.ships.len() {
                let s = &a.ships[i];
                if s.alive && s.doing == Doing::Haul {
                    if out[i].is_none() {
                        if let (at, reach, Some(node)) = a.station(i) {
                            out[i] = Some((a.tick, (isqrt(dist2(s.x, s.y, at.0, at.1)) - reach).max(0), node, at, false));
                        }
                    }
                    if let Some(trip) = out[i].as_mut() {
                        trip.4 |= s.bumped;
                    }
                } else if let Some((t0, far, node, at, bumped)) = out[i].take() {
                    let o = &a.ships[node];
                    if s.alive && o.alive && (o.x, o.y) == at && !bumped {
                        trips += 1;
                        worst = worst.max((a.tick - t0) as i32 - far * 4 / s.st.vmax);
                    }
                }
            }
        }
        assert!(trips >= 100, "only {trips} trips home in six minutes");
        assert!(worst <= 45, "a full miner took {} s longer to get home than the flight takes", worst / 15);
    }

    #[test]
    fn the_ai_keeps_what_it_plants_in_its_kings_network() {
        // Half a minute after each AI has planted its second factory, every node it has shares
        // its king's bank: nothing it has mined is stranded where it can't plant with it. (Seed 2:
        // a match in which both get that far. On seed 1 one of them is beaten at 7:25 with a
        // single factory planted.)
        let mut a = Arena::for_match(2, [starter_deck(), starter_deck()], [true, true]);
        let nodes = |a: &Arena, t: usize| (0..a.ships.len()).filter(|&i| a.ships[i].team == t && a.is_node(i)).collect::<Vec<usize>>();
        let mut since = [None; 2];
        let mut looked = [false; 2];
        while !(looked[0] && looked[1]) {
            a.step();
            assert!(a.over.is_none() && a.tick < 15 * 720, "an AI hadn't planted a second factory by {}", a.tick / 15);
            for t in 0..2 {
                if since[t].is_none() && nodes(&a, t).len() >= 3 {
                    since[t] = Some(a.tick);
                }
                if !looked[t] && since[t].is_some_and(|s| a.tick == s + 15 * 30) {
                    let e = a.eco.as_ref().unwrap();
                    let n = nodes(&a, t);
                    assert!(n.len() >= 3, "team {t} lost a node within half a minute of planting it");
                    assert!(n.iter().all(|&i| e.net[i] == e.net[e.king[t]]), "team {t} has a node cut off from its king");
                    looked[t] = true;
                }
            }
        }
    }

    #[test]
    fn the_ai_plants_cheap_while_the_enemy_has_more_factories() {
        // With two factories down, the AI's plan is to save for a shipyard. Shown an enemy with
        // three, it plants an outpost with what it has instead, and doesn't let itself be
        // out-built while it saves. (This was a ten-minute match against a planter. On today's
        // starter deck the outpost spends the bank as it comes in, nobody out-plants anybody in
        // ten minutes, and that match passed with the rule switched off.)
        let planted = |enemy: i32| {
            let mut a = Arena::for_bots(1, [starter_deck(), starter_deck()], [Bot::Ai, Bot::None]);
            let outpost = starter_deck()[4].clone();
            for n in 0..2 {
                a.spawn(0, 4, &outpost, 60 + 40 * n, 60, 0, false);
            }
            for n in 0..enemy {
                let f = a.spawn(1, 4, &outpost, a.w - 60 - 40 * n, a.h - 60, 0, false);
                a.ships[f].rooted = true;
            }
            let k = a.eco.as_ref().unwrap().king[0];
            let cards = &a.eco.as_ref().unwrap().cards[0];
            assert!(a.bank(k) >= cards[4].cost + 40 && a.bank(k) < cards[5].cost, "the bank should hold an outpost and not a shipyard");
            for _ in 0..15 * 5 {
                a.step();
            }
            a.eco.as_ref().unwrap().tally[0].planted
        };
        assert_eq!(planted(3), 1, "behind by a factory, the AI didn't plant the outpost it could pay for");
        assert_eq!(planted(2), 0, "level on factories, the AI spent its shipyard savings on an outpost");
    }

    #[test]
    fn a_fleet_turned_out_for_a_raid_stays_out_until_the_raider_is_well_clear() {
        // (The match below used to show this, by counting how often the AI changed its orders.
        // A match is one roll of the dice: after an unrelated change it stayed under the count
        // with the rule switched off.)
        let mut a = Arena::for_bots(1, [starter_deck(), starter_deck()], [Bot::Ai, Bot::None]);
        let k = a.eco.as_ref().unwrap().king[0];
        let (kx, ky) = (a.ships[k].x, a.ships[k].y);
        let clear = ALARM_RANGE + ALARM_CLEAR;
        // An enemy fighter on HOLD, parked straight up the map from the AI's king: away from the
        // home rock, where the AI's first outpost goes (a raid on that counts too).
        let raider = a.spawn(1, 0, &starter_deck()[0], kx, ky - clear - 40, 0, false);
        a.ships[raider].order = Order::Hold;
        // Parks it so many pixels from the king, gives the AI a second to think, and says what
        // the AI's own fighters are told.
        let told_with_raider_at = |a: &mut Arena, far: i32| {
            let s = &mut a.ships[raider];
            (s.x, s.y, s.vx, s.vy) = (kx, ky - far, 0, 0);
            s.home = (s.x, s.y);
            for _ in 0..16 {
                a.step();
            }
            let r = &a.ships[raider];
            assert!(r.alive && (r.x, r.y) == (kx, ky - far), "the raider parked {far} px out is at ({}, {}), {}", r.x, r.y, r.doing.name());
            a.eco.as_ref().unwrap().orders[0][0]
        };
        assert_eq!(told_with_raider_at(&mut a, clear + 40), Order::Guard, "the fleet is out with nobody near");
        assert_eq!(told_with_raider_at(&mut a, ALARM_RANGE - 10), Order::Hunt, "a raider in range didn't turn the fleet out");
        assert_eq!(told_with_raider_at(&mut a, clear - 20), Order::Hunt, "the fleet went home with the raider a step outside the range");
        assert_eq!(told_with_raider_at(&mut a, clear + 40), Order::Guard, "the fleet stayed out with the raider well clear");
        assert_eq!(told_with_raider_at(&mut a, clear - 20), Order::Guard, "a raider outside the range turned the fleet out");
    }

    #[test]
    fn the_ai_masses_then_attacks_and_the_scripts_just_hunt() {
        // By 8:00 the AI has sent its warships out and called them back at least once each way,
        // without switching every few seconds. A scripted side sets HUNT once per warship design
        // and never again.
        let mut a = Arena::for_bots(2, [starter_deck(), starter_deck()], [Bot::Ai, Bot::Planter]);
        while a.over.is_none() && a.tick < 15 * 480 {
            a.step();
        }
        let e = a.eco.as_ref().unwrap();
        let switches = |t: usize| e.log.iter().filter(|(_, c)| matches!(c, Cmd::Order { team, .. } if *team == t)).count();
        assert_eq!(switches(1), 4, "the planter sets its four warships to HUNT, once");
        let hunts = e.log.iter().filter(|(_, c)| matches!(c, Cmd::Order { team: 0, order: Order::Hunt, .. })).count();
        let guards = e.log.iter().filter(|(_, c)| matches!(c, Cmd::Order { team: 0, order: Order::Guard, .. })).count();
        assert!(hunts >= 4 && guards >= 1, "the AI sent {hunts} designs out and called {guards} back in eight minutes");
        assert!(switches(0) <= 80, "the AI changed orders {} times in eight minutes", switches(0));
    }

    // ----- The AI's defence, and the two openings it's measured against -----

    /// The AI as blue, with nobody playing red.
    fn ai_alone() -> Arena {
        Arena::for_bots(1, [starter_deck(), starter_deck()], [Bot::Ai, Bot::None])
    }

    /// An enemy ship of the starter deck's design `slot`, on HOLD so that it stays where it's put.
    fn parked(a: &mut Arena, slot: usize, x: i32, y: i32) -> usize {
        let i = a.spawn(1, slot, &starter_deck()[slot], x, y, 0, false);
        a.ships[i].order = Order::Hold;
        a.ships[i].home = (x, y);
        i
    }

    /// One of the AI's own ships of the starter deck's design `slot`.
    fn friend(a: &mut Arena, slot: usize, x: i32, y: i32) -> usize {
        a.spawn(0, slot, &starter_deck()[slot], x, y, 0, false)
    }

    /// A second and a tick: long enough for a bot to have thought again and been obeyed.
    fn think(a: &mut Arena) {
        for _ in 0..16 {
            a.step();
        }
    }

    /// The first ship (not a seed) blue's king was told to print.
    fn first_ship_printed(a: &Arena) -> Option<usize> {
        a.eco.as_ref().unwrap().log.iter().find_map(|(_, c)| match c {
            Cmd::Print { team: 0, slot, target: None } => Some(*slot),
            _ => None,
        })
    }

    /// Where blue's first seed was sent.
    fn first_seed_sent_to(a: &Arena) -> Option<(i32, i32)> {
        a.eco.as_ref().unwrap().log.iter().find_map(|(_, c)| match c {
            Cmd::Print { team: 0, target: Some(p), .. } => Some(*p),
            _ => None,
        })
    }

    const FIGHTER: usize = 0;
    const LANCER: usize = 1;
    const FRIGATE: usize = 2;
    const MINER: usize = 3;
    const OUTPOST: usize = 4;
    const CAPITAL: usize = 6;

    #[test]
    fn an_enemy_king_on_the_ais_half_is_a_raid_and_a_raid_it_cant_match_comes_before_miners() {
        // What the AI's king prints first, after its seed, with the enemy king put somewhere.
        let first = |king_at: Option<(i32, i32)>| {
            let mut a = ai_alone();
            let ek = a.eco.as_ref().unwrap().king[1];
            if let Some((x, y)) = king_at {
                (a.ships[ek].x, a.ships[ek].y) = (x, y);
            }
            while first_ship_printed(&a).is_none() {
                a.step();
                assert!(a.tick < 15 * 30, "the AI's king printed nothing in half a minute");
            }
            first_ship_printed(&a).unwrap()
        };
        let (home, far) = (KING_STARTS[0], KING_STARTS[1]);
        assert_eq!(first(None), MINER, "with the enemy king at home, a miner comes first");
        // On the AI's half, and much further from anything of the AI's than a raiding ship
        // would have to be.
        let over = (home.0 + 420, home.1 - 260);
        assert!(dist2(over.0, over.1, home.0, home.1) < dist2(over.0, over.1, far.0, far.1) && cheb(over.0, over.1, home.0, home.1) > 2 * ALARM_RANGE);
        assert_eq!(first(Some(over)), LANCER, "a king on its way over wasn't taken for a raid, or a miner came before a gun");
        // The same distance out from its own start, it's on its own half: no raid.
        let back = (far.0 - 420, far.1 + 260);
        assert_eq!(first(Some(back)), MINER, "a king on its own half was taken for a raid");
    }

    #[test]
    fn the_ai_answers_a_raiding_kings_ships_before_the_king() {
        // The enemy king comes up to the AI's home, alone or with two lancers. Alone, it's the
        // biggest ship there is and lances answer it. With lancers, flak answers the lancers.
        let first = |lancers: i32| {
            let mut a = ai_alone();
            let ek = a.eco.as_ref().unwrap().king[1];
            let (hx, hy) = KING_STARTS[0];
            (a.ships[ek].x, a.ships[ek].y) = (hx + 150, hy - 100);
            for n in 0..lancers {
                parked(&mut a, LANCER, hx + 130 + 40 * n, hy - 150);
            }
            while first_ship_printed(&a).is_none() {
                a.step();
                assert!(a.tick < 15 * 30, "the AI's king printed nothing in half a minute");
            }
            first_ship_printed(&a).unwrap()
        };
        assert_eq!(first(0), LANCER, "a king raiding alone is answered with lances");
        assert_eq!(first(2), FIGHTER, "a king raiding with lancers is answered through the lancers");
    }

    #[test]
    fn while_a_raid_is_on_the_ais_king_stands_beside_its_first_factory() {
        // A minute in, with nobody near, the king is where it started.
        let mut a = ai_alone();
        for _ in 0..15 * 60 {
            a.step();
        }
        let k = a.eco.as_ref().unwrap().king[0];
        let f = factory(&a).expect("the AI planted nothing in a minute");
        let (sx, sy) = KING_STARTS[0];
        assert_eq!((a.ships[k].x, a.ships[k].y), (sx, sy), "with no raid, the king left its start");
        // A frigate parks in reach of it (more than the AI's first two fighters fly out to
        // meet). Six seconds later the king is right beside its outpost, on the side it came
        // from: near enough that its flak reaches anything shooting the outpost from close by.
        let raider = parked(&mut a, FRIGATE, sx, sy - ALARM_RANGE + 10);
        for _ in 0..15 * 6 {
            a.step();
        }
        let (king, post) = (&a.ships[k], &a.ships[f]);
        let gap = post.half() + king.half() + STRONGPOINT_GAP;
        assert!(a.ships[raider].alive);
        assert!(cheb(king.x, king.y, post.x, post.y) <= gap + 4, "under a raid the king is at ({}, {}), not beside its outpost at ({}, {})", king.x, king.y, post.x, post.y);
        assert!(dist2(king.x, king.y, sx, sy) < dist2(post.x, post.y, sx, sy), "the king is on the far side of its outpost");
        // The raider gone, it goes back to its start.
        a.ships[raider].alive = false;
        for _ in 0..15 * 8 {
            a.step();
        }
        assert!(cheb(a.ships[k].x, a.ships[k].y, sx, sy) <= 6, "the raid over, the king is at ({}, {})", a.ships[k].x, a.ships[k].y);
    }

    #[test]
    fn the_ais_king_keeps_its_start_when_its_post_would_cut_a_node_off() {
        // The AI has an outpost 120 px along the map from its king's start, and perhaps a second
        // 139 px the other way: in reach of the start, and out of reach of the post beside the
        // first. A frigate parks in reach of the king. Where the king is four seconds later.
        let king_after = |second: bool| {
            let mut a = ai_alone();
            let k = a.eco.as_ref().unwrap().king[0];
            let (kx, ky) = (a.ships[k].x, a.ships[k].y);
            // Nothing in the bank, so it plants nothing of its own to confuse the picture.
            a.ships[k].store = 0;
            let mut nodes = vec![(kx + 120, ky)];
            if second {
                nodes.push((kx - 125, ky + 60));
            }
            for &(x, y) in &nodes {
                let f = friend(&mut a, OUTPOST, x, y);
                (a.ships[f].rooted, a.ships[f].node) = (true, true);
            }
            let first = a.ships.len() - nodes.len();
            parked(&mut a, FRIGATE, kx, ky - ALARM_RANGE + 10);
            for _ in 0..15 * 4 {
                a.step();
            }
            let (king, post) = (&a.ships[k], &a.ships[first]);
            let e = a.eco.as_ref().unwrap();
            let whole = (0..a.ships.len()).filter(|&i| a.ships[i].team == 0 && a.is_node(i)).all(|i| e.net[i] == e.net[k]);
            ((king.x - kx, king.y - ky), cheb(king.x, king.y, post.x, post.y) <= post.half() + king.half() + STRONGPOINT_GAP + 4, whole)
        };
        let (_, beside, whole) = king_after(false);
        assert!(beside && whole, "with one outpost the king should be beside it, in one network: beside {beside}, whole {whole}");
        let (moved, beside, whole) = king_after(true);
        assert!(moved == (0, 0) && !beside && whole, "with a second outpost only its start reaches, the king moved {moved:?}: beside {beside}, whole {whole}");
    }

    #[test]
    fn a_raid_changes_where_the_ai_plants_only_when_it_outweighs_the_ai() {
        // The AI's bank covers an outpost and the 40 px it always keeps, not the 150 px it
        // used to keep back under a raid. An enemy fighter is parked in reach of its home
        // rocks. Where its seed is sent within a second, if anywhere.
        let seed = |raid: bool, fighters: i32, factory: bool| {
            let mut a = ai_alone();
            let k = a.eco.as_ref().unwrap().king[0];
            let (kx, ky) = (a.ships[k].x, a.ships[k].y);
            a.ships[k].store = a.eco.as_ref().unwrap().cards[0][OUTPOST].cost + 45;
            let at = (kx, ky - 120);
            if raid {
                parked(&mut a, FIGHTER, at.0, at.1);
            }
            for n in 0..fighters {
                friend(&mut a, FIGHTER, kx - 10 + 20 * n, ky + 60);
            }
            if factory {
                let f = friend(&mut a, OUTPOST, kx - 80, ky + 80);
                a.ships[f].rooted = true;
            }
            think(&mut a);
            (first_seed_sent_to(&a), at)
        };
        let (quiet, at) = seed(false, 0, false);
        let quiet = quiet.expect("with nobody near, the AI plants");
        assert!(cheb(quiet.0, quiet.1, at.0, at.1) <= ALARM_RANGE, "the home rock isn't in reach of where the raider parks");
        // Two fighters of its own outweigh the raider: it plants where it would have, and
        // holds nothing extra back.
        let (met, _) = seed(true, 2, false);
        assert_eq!(met, Some(quiet), "a raid the AI outweighs changed its planting");
        // With nothing to meet the raider, and no factory yet, it plants out of the raider's
        // reach.
        let (away, at) = seed(true, 0, false);
        let away = away.expect("outweighed, with no factory, the AI didn't plant anywhere");
        assert!(cheb(away.0, away.1, at.0, at.1) > ALARM_RANGE - 4, "outweighed, it sent a seed to ({}, {}), in the raider's reach", away.0, away.1);
        // With a factory already, it doesn't plant one that would be cut off: every rock that
        // would join its network is in the raider's reach.
        let (cut, _) = seed(true, 0, true);
        assert_eq!(cut, None, "outweighed, with a factory, the AI planted one out on its own");
    }

    #[test]
    fn the_ais_king_stands_by_what_outweighs_a_raider_and_falls_back_from_what_outweighs_it() {
        // A minute in, the AI's king is at its start with its outpost planted. An enemy parks
        // 85 px up the map from it, inside the 90 px at which a bot's king backs away. Four
        // seconds later: whether the king is beside its outpost, how much further it is from
        // the enemy, and how much nearer the place a second factory stands (or would).
        let after = |slot: usize, refuge: bool| {
            let mut a = ai_alone();
            for _ in 0..15 * 60 {
                a.step();
            }
            let k = a.eco.as_ref().unwrap().king[0];
            let f = factory(&a).unwrap();
            let (kx, ky) = (a.ships[k].x, a.ships[k].y);
            let far = (kx + 400, ky - 400);
            if refuge {
                // A second factory of the AI's, far up the map and along it, with nothing near
                // it: past the enemy, the opposite way from backing off.
                let f2 = friend(&mut a, OUTPOST, far.0, far.1);
                a.ships[f2].rooted = true;
            }
            let e = parked(&mut a, slot, kx, ky - 85);
            let (ex, ey) = (a.ships[e].x, a.ships[e].y);
            for _ in 0..15 * 4 {
                a.step();
            }
            let (king, post) = (&a.ships[k], &a.ships[f]);
            let beside = cheb(king.x, king.y, post.x, post.y) <= post.half() + king.half() + STRONGPOINT_GAP + 4;
            (beside, cheb(king.x, king.y, ex, ey) - cheb(kx, ky, ex, ey), cheb(kx, ky, far.0, far.1) - cheb(king.x, king.y, far.0, far.1))
        };
        // A fighter is 9 px. The king and its outpost outweigh it thirty times over: the king
        // takes its post beside the outpost, and that's all.
        let (beside, _, _) = after(FIGHTER, false);
        assert!(beside, "raided by a single fighter, the AI's king isn't at its post");
        // A capital outweighs them both. With nowhere else to go the king backs straight away
        // from it.
        let (beside, further, _) = after(CAPITAL, false);
        assert!(!beside && further >= 20, "outweighed, with one factory: beside it {beside}, {further} px further from the capital");
        // With a second factory that nothing is near, it makes for that instead. (Backing away
        // from the capital took it the other way: see the line above.)
        let (_, _, wrong_way) = after(CAPITAL, false);
        let (beside, _, nearer) = after(CAPITAL, true);
        assert!(wrong_way < 20, "with no second factory the king still went {wrong_way} px towards where one would be");
        assert!(!beside && nearer >= 60, "outweighed, with a quiet factory to fall back on, the king is {nearer} px nearer it");
    }

    #[test]
    fn the_ai_arms_while_the_enemys_fleet_has_the_edge() {
        // The AI has an outpost and its two miners, and a bank too small for the factory it's
        // saving for. The enemy has so many fighters parked at its own home, raiding nobody.
        // What the AI's king is printing a second later.
        let printing = |fighters: i32| {
            let mut a = ai_alone();
            let k = a.eco.as_ref().unwrap().king[0];
            let (kx, ky) = (a.ships[k].x, a.ships[k].y);
            let f = friend(&mut a, OUTPOST, kx + 80, ky - 60);
            a.ships[f].rooted = true;
            for n in 0..2 {
                friend(&mut a, MINER, kx - 40 + 20 * n, ky + 50);
            }
            let (ex, ey) = KING_STARTS[1];
            for n in 0..fighters {
                parked(&mut a, FIGHTER, ex - 40 + 14 * n, ey + 60);
            }
            a.ships[k].store = 60;
            think(&mut a);
            a.eco.as_ref().unwrap().queue[0].first().map(|j| j.slot)
        };
        assert_eq!(printing(0), None, "with no enemy fleet, the AI spent what it was saving");
        // Against small flak ships, something big: the frigate is the cheapest it has.
        assert_eq!(printing(3), Some(FRIGATE), "with the enemy three fighters up, the AI printed no gun");
    }

    #[test]
    fn a_raid_on_the_ais_miners_turns_its_fleet_out() {
        // An enemy fighter parks out in the map, far from anything the AI has planted, with one
        // of the AI's miners 100 px from it or not. What the AI's fighters are told.
        let told = |miner: bool| {
            let mut a = ai_alone();
            let (mx, my) = (a.w / 2 - 150, a.h / 2 + 150);
            if miner {
                friend(&mut a, MINER, mx, my);
            }
            parked(&mut a, FIGHTER, mx + 100, my);
            think(&mut a);
            a.eco.as_ref().unwrap().orders[0][FIGHTER]
        };
        assert_eq!(told(false), Order::Guard, "a fighter near nothing of the AI's turned its fleet out");
        assert_eq!(told(true), Order::Hunt, "a fighter 100 px from one of the AI's miners didn't turn its fleet out");
    }

    #[test]
    fn the_scripted_openings_see_a_raid_as_they_always_did() {
        // The expander keeps 150 px back under a raid, and its bank here covers an outpost and
        // 45 px: it plants only if it sees no raid. To it a raid is an enemy ship near its king
        // or something it has planted. An enemy king on its half is none (to the AI it is), and
        // neither is a fighter beside one of its miners.
        let plants = |what: usize| {
            let mut a = Arena::for_bots(1, [starter_deck(), starter_deck()], [Bot::Expander, Bot::None]);
            let (k, ek) = (a.eco.as_ref().unwrap().king[0], a.eco.as_ref().unwrap().king[1]);
            let (kx, ky) = (a.ships[k].x, a.ships[k].y);
            a.ships[k].store = a.eco.as_ref().unwrap().cards[0][OUTPOST].cost + 45;
            match what {
                1 => {
                    parked(&mut a, FIGHTER, kx, ky - 150);
                }
                2 => (a.ships[ek].x, a.ships[ek].y) = (kx + 420, ky - 260),
                3 => {
                    let (mx, my) = (a.w / 2 - 150, a.h / 2 + 150);
                    friend(&mut a, MINER, mx, my);
                    parked(&mut a, FIGHTER, mx + 100, my);
                }
                _ => {}
            }
            think(&mut a);
            first_seed_sent_to(&a).is_some()
        };
        assert!(plants(0), "with nobody near, the expander didn't plant");
        assert!(!plants(1), "with a fighter by its king, the expander didn't keep its 150 px back");
        assert!(plants(2), "the expander took an enemy king on its half for a raid");
        assert!(plants(3), "the expander took a fighter beside its miner for a raid");
    }

    #[test]
    fn the_rush_opening_brings_its_king_and_takes_it_home_once_its_hurt() {
        let mut a = Arena::for_bots(1, [starter_deck(), starter_deck()], [Bot::Rusher, Bot::None]);
        let (k, ek) = (a.eco.as_ref().unwrap().king[0], a.eco.as_ref().unwrap().king[1]);
        think(&mut a);
        // It has planted its one factory, asked for a lancer, and put every warship on HUNT.
        // (Its bank is emptied here, so that no lancer is there to end the match.)
        {
            let e = a.eco.as_ref().unwrap();
            assert!(first_seed_sent_to(&a).is_some());
            assert_eq!(first_ship_printed(&a), Some(LANCER));
            assert!((0..e.cards[0].len()).filter(|&s| e.cards[0][s].warship()).all(|s| e.orders[0][s] == Order::Hunt));
        }
        a.ships[k].store = 0;
        // The enemy has nothing planted, so its king is what the rush goes for: 34 s later the
        // rusher's king is as close to it as it comes.
        for _ in 0..15 * 34 {
            a.step();
        }
        let d = cheb(a.ships[k].x, a.ships[k].y, a.ships[ek].x, a.ships[ek].y);
        assert!(d <= RUSH_CLOSE + STEP && d >= RUSH_CLOSE - 10, "the rusher's king is {d} px from the enemy's");
        let seeds = a.eco.as_ref().unwrap().log.iter().filter(|(_, c)| matches!(c, Cmd::Print { team: 0, target: Some(_), .. })).count();
        assert_eq!(seeds, 1, "the rusher plants once");
        // A quarter of it knocked off, it goes home, and stays there.
        let full = crate::pg::design::king().stats().count;
        let at = (a.ships[k].x, a.ships[k].y);
        for c in 0..a.ships[k].hp.len() {
            if a.ships[k].px_now() * 4 < full * 3 {
                break;
            }
            a.kill_cell(k, c, at, (0, 0));
        }
        assert!(a.ships[k].alive && a.ships[k].px_now() * 4 < full * 3);
        for _ in 0..15 * 45 {
            a.step();
        }
        let (hx, hy) = KING_STARTS[0];
        assert!(a.over.is_none() && cheb(a.ships[k].x, a.ships[k].y, hx, hy) <= 6, "the hurt king is at ({}, {}), not home", a.ships[k].x, a.ships[k].y);
    }

    #[test]
    fn the_sabotage_opening_is_the_ais_economy_with_everything_on_sab() {
        let mut a = Arena::for_bots(1, [starter_deck(), starter_deck()], [Bot::Saboteur, Bot::None]);
        think(&mut a);
        let e = a.eco.as_ref().unwrap();
        let warships: Vec<usize> = (0..e.cards[0].len()).filter(|&s| e.cards[0][s].warship()).collect();
        assert!(warships.len() >= 3 && warships.iter().all(|&s| e.orders[0][s] == Order::Sab), "its warships are told {:?}", warships.iter().map(|&s| e.orders[0][s]).collect::<Vec<_>>());
        assert!(first_seed_sent_to(&a).is_some(), "it plants its first factory at once, like the AI");
    }


    #[test]
    fn nothing_comes_out_of_hyperspace_into_an_enemy() {
        // A slab of enemy plate sits right where the king's fighter would arrive. The fighter
        // stays in hyperspace until the king moves and its spot is clear.
        let mut rows = vec!["ppppppppp"; 9];
        rows[4] = "ppppcpppp";
        let slab = Design::from_rows("SLAB", Order::Hold, &rows);
        let mut a = quiet();
        let k = a.eco.as_ref().unwrap().king[0];
        let (_, _, r, _) = a.ships[k].bbox();
        let slab_i = a.spawn(1, 0, &slab, r + 6, a.ships[k].y, 0, false);
        a.command(Cmd::Print { team: 0, slot: 0, target: None });
        let n = a.ships.len();
        while a.ships.len() == n {
            a.step();
            assert!(a.tick < 15 * 60, "the fighter was never delivered");
        }
        for _ in 0..15 * 2 {
            a.step();
        }
        let f = n;
        assert_eq!(a.ships[f].name, "FIGHTER");
        assert!(a.ships[f].warp > 0, "it came out of hyperspace inside the slab");
        // The king flies off; the fighter comes out beside it, clear of the slab.
        for _ in 0..15 * 4 {
            a.command(Cmd::Steer { team: 0, dx: 0, dy: 1 });
            a.step();
        }
        assert_eq!(a.ships[f].warp, 0, "with its spot clear it should have arrived");
        assert_eq!(shared(&a, f, slab_i), 0);
    }

    /// Enough of a match's state to tell two runs apart.
    fn fingerprint(a: &Arena) -> (u32, Option<Option<usize>>, usize, i64, [i32; 2]) {
        let sum: i64 = a.ships.iter().map(|s| (s.x as i64 * 31 + s.y as i64) * 7 + s.px_now() as i64 + s.store as i64 * 3 + s.alive as i64).sum();
        (a.tick, a.over, a.ships.len(), sum, [a.score[0].mined, a.score[1].mined])
    }

    #[test]
    fn a_match_replays_from_its_commands() {
        // Five minutes of AI against AI, keeping the log of every command. The same commands fed
        // on the same ticks into a match with no AI at all give the same match: commands are the
        // only thing that drives one.
        let mut a = Arena::for_match(4, [starter_deck(), starter_deck()], [true, true]);
        while a.over.is_none() && a.tick < 15 * 300 {
            a.step();
        }
        let log = a.eco.as_ref().unwrap().log.clone();
        assert!(log.len() >= 6, "only {} commands in five minutes", log.len());
        let mut b = Arena::for_match(4, [starter_deck(), starter_deck()], [false, false]);
        let mut k = 0;
        while b.tick < a.tick {
            while k < log.len() && log[k].0 == b.tick + 1 {
                b.command(log[k].1);
                k += 1;
            }
            b.step();
        }
        assert_eq!(k, log.len());
        assert_eq!(fingerprint(&a), fingerprint(&b), "the replay went differently");
        assert_eq!(b.eco.as_ref().unwrap().log, log, "the replay's own log should match");
    }
}
