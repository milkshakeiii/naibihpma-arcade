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
/// A beaten king's speed, in quarter-pixels a tick: a quarter of a pixel, a tenth of its full
/// speed. The slowest ship that flies at all makes 2 (`SPEED_LEAST`), so anything with an
/// engine runs a beaten king down, even one fleeing on a diagonal. (2, a fifth, until
/// 2026-11-17, when the slowest ship went from 3 to 2.)
pub const KING_SLOW: i32 = 1;
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
/// A bot's king keeps this far outside the reach of an enemy's guns, edge to edge.
const KING_BERTH: i32 = 30;
/// Running from guns, it looks this far each way it could go for the way that leaves it safest.
const KING_LOOK: i32 = 120;
/// The AI's rules of 2026-11-15, after Henry's match against SWARM, each with a bit of `Eco.off`
/// that switches it off for measuring (`--teams --off`, `--old`). In play neither is ever off.
/// (Three more were tried and measured with these, and made no difference: see MATCH.md.)
pub const RULE_MINERS: u32 = 1;
/// The AI keeps printing miners while the rock its stations can reach holds this many pixels for
/// each miner it has, up to `MINERS_MOST` miners. A rock counts if its edge is within
/// `MINE_REACH` of one of its nodes.
const MINER_ROCK: i32 = 60;
const MINERS_MOST: usize = 30;
const MINE_REACH: i32 = 200;
/// Money nothing is spending plants another factory: a bank that has held the cheapest
/// factory's price and what the AI keeps besides for `PILE_TICKS` plants one past the most its
/// strategy (or its rules) would have, up to `PILE_MOST` in all.
pub const RULE_SPEND: u32 = 2;
/// The AI plants as soon as its bank holds the factory's price, keeping nothing back for the new
/// factory to print with (Henry planted at 121, the outpost's price, all through his match).
pub const RULE_SPARE: u32 = 4;
/// The AI keeps its tractors up with its miners (2026-11-16, with the tractor): when its tractors
/// can't pull as fast as two thirds of its miners bite (a miner flies as well as bites), its
/// king prints a collector before another miner. With the rule off it never prints one, and
/// lives on what its king's and its factories' tractors take in.
pub const RULE_TRACTORS: u32 = 8;
const PILE_TICKS: u32 = 15 * 30;
const PILE_MOST: usize = 14;
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
    /// The king sends a seeded design's seed to the spot `at` in open space. The seed is paid for
    /// in full on the spot, like any seed. With no rock under it, what grows there takes every
    /// pixel from its network's bank.
    Sow { team: usize, slot: usize, at: (i32, i32) },
    /// A design's standing order: every ship of it, flying or still to be printed, takes it.
    /// An AI's command only: a person's side has the stance (`Stance`), and this is dropped
    /// when it comes from one.
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
    /// One of the side's ships is sent to a place, whatever its card's order. It flies there no
    /// faster than `pace` (the slowest of the ships sent with it; 0: as fast as it can), fights
    /// what comes close on the way, and fights what's round the place once it's there. A group
    /// is sent with one of these for each ship. A king can be sent too: it flies there while
    /// its keys are let go, and stops when it arrives.
    Send { team: usize, ship: usize, at: (i32, i32), pace: i32 },
    /// A ship that was sent is let go: it does as its order says again, which for a person's
    /// side is the stance.
    Home { team: usize, ship: usize },
    /// One of the side's ships (or its king) is sent after a thing, its own side's or the
    /// enemy's: as `Send`, to wherever that thing is, tick by tick. When the thing is gone the
    /// order stands at the place it last was.
    Follow { team: usize, ship: usize, target: usize, pace: i32 },
    /// The side's one stance, for every ship it hasn't sent anywhere: attack the nearest enemy,
    /// or guard where it was built. Every card that takes the stance (`Card::stanced`: all that
    /// fly, bar what has mining equipment) goes on HUNT or on GUARD, and its ships go at once.
    Stance { team: usize, attack: bool },
    /// A factory's rally point: the ships it builds from now on make that place their home, and
    /// wait and guard there and not round the factory. A place on the factory itself is no
    /// rally point: its ships stay round it again.
    Rally { team: usize, ship: usize, at: (i32, i32) },
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
            Cmd::Steer { team, .. } | Cmd::Print { team, .. } | Cmd::Sow { team, .. } | Cmd::Order { team, .. } | Cmd::Repeat { team, .. } | Cmd::Cancel { team, .. } | Cmd::Send { team, .. } | Cmd::Home { team, .. } | Cmd::Follow { team, .. } | Cmd::Stance { team, .. } | Cmd::Rally { team, .. } => team,
        }
    }

    /// A command as a few words of text, to send to another machine. `from_wire` reads it back.
    pub fn to_wire(&self) -> String {
        match *self {
            Cmd::Steer { team, dx, dy } => format!("S {team} {dx} {dy}"),
            Cmd::Print { team, slot, target: None } => format!("P {team} {slot}"),
            Cmd::Print { team, slot, target: Some((x, y)) } => format!("P {team} {slot} {x} {y}"),
            Cmd::Sow { team, slot, at: (x, y) } => format!("W {team} {slot} {x} {y}"),
            Cmd::Order { team, slot, order } => format!("O {team} {slot} {}", WIRE_ORDERS.iter().position(|&o| o == order).unwrap_or(0)),
            Cmd::Repeat { team, slot, at: None } => format!("R {team} {slot}"),
            Cmd::Repeat { team, slot, at: Some(at) } => format!("R {team} {slot} {at}"),
            Cmd::Cancel { team, at, slot } => format!("C {team} {at} {slot}"),
            Cmd::Send { team, ship, at: (x, y), pace } => format!("G {team} {ship} {x} {y} {pace}"),
            Cmd::Home { team, ship } => format!("H {team} {ship}"),
            Cmd::Follow { team, ship, target, pace } => format!("F {team} {ship} {target} {pace}"),
            Cmd::Stance { team, attack } => format!("T {team} {}", attack as u8),
            Cmd::Rally { team, ship, at: (x, y) } => format!("Y {team} {ship} {x} {y}"),
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
            ("W", 4) => Cmd::Sow { team, slot: n(1, 63)?, at: (far(2)?, far(3)?) },
            ("O", 3) => Cmd::Order { team, slot: n(1, 63)?, order: WIRE_ORDERS[n(2, 5)?] },
            ("R", 2) => Cmd::Repeat { team, slot: n(1, 63)?, at: None },
            ("R", 3) => Cmd::Repeat { team, slot: n(1, 63)?, at: Some(n(2, 100_000)?) },
            ("C", 3) => Cmd::Cancel { team, at: n(1, 100_000)?, slot: n(2, 63)? },
            ("G", 5) => Cmd::Send { team, ship: n(1, 1_000_000)?, at: (far(2)?, far(3)?), pace: n(4, 255)? as i32 },
            ("H", 2) => Cmd::Home { team, ship: n(1, 1_000_000)? },
            ("F", 4) => Cmd::Follow { team, ship: n(1, 1_000_000)?, target: n(2, 1_000_000)?, pace: n(3, 255)? as i32 },
            ("T", 2) => Cmd::Stance { team, attack: n(1, 1)? == 1 },
            ("Y", 4) => Cmd::Rally { team, ship: n(1, 1_000_000)?, at: (far(2)?, far(3)?) },
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

/// A strategy for the AI to play a team by: what it plants and when, how many miners it keeps,
/// what its king prints, when it first attacks and with how much, and which cards raid. Slots are
/// the team's own. An AI with no strategy plays as it always has, the way the starter deck suits it.
/// Whatever the strategy, the AI still answers raids as it always does (`bot_think`).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Strategy {
    /// The factories it plants in turn, each with the earliest second it may be planted at. Past
    /// the end of the list it plants the last one again.
    pub plant: &'static [(usize, u32)],
    /// The most factories it has at once, seeds on their way counted.
    pub most: usize,
    /// Till this second it keeps its miners lean, one for each factory it has and one more, so
    /// its opening gets everything else. After that it keeps printing them while there's rock
    /// for them (`RULE_MINERS`).
    pub lean: u32,
    /// What its king prints in turn once it has its miners. Empty: what counters the enemy's
    /// fleet, as the AI always has.
    pub prints: &'static [usize],
    /// What it keeps in the bank past what the next factory needs before its king prints guns.
    pub keep: i32,
    /// Its first wave goes no earlier than this second, and a wave needs this many pixels of
    /// warships.
    pub first: u32,
    pub wave: i32,
    /// A bold side sends a wave as soon as it has the pixels, whatever the enemy has. A careful
    /// one also waits for the edge, as the AI always has.
    pub bold: bool,
    /// Cards that raid: they go for miners and factories (SAB) as soon as they're built, whatever
    /// the waves are doing.
    pub raid: &'static [usize],
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
    /// Since when its bank has held more than it's spending (0: it hasn't), for `RULE_SPEND`.
    rich_since: u32,
}

/// What a warship is to the counter triangle: small with flak, small with a lance or cannon, or
/// big.
fn class_of(st: &Stats) -> usize {
    if st.count > 20 {
        2
    } else if st.lance + st.cannon + st.tracker + st.beam + st.missile + st.repeater > 0 {
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
    /// A factory still growing, with no rock left under it to grow from: its network pays for
    /// each pixel, in turn with everything else that's printing.
    Grower(usize),
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
    /// The tick the AI first sent a wave out (0: never). A raid turning its fleet out isn't one.
    pub first_wave: u32,
}

/// A rock as generated: its middle and its pixels (some may since have been eaten or mined).
#[derive(Clone, Debug, Hash)]
pub struct Rock {
    pub cx: i32,
    pub cy: i32,
    pub px: Vec<(i32, i32)>,
}

/// The biggest body among some pixels, in the order they came: the most of them that hang
/// together, edge to edge or corner to corner. (Of two as big, the one that comes first.)
fn one_body(px: Vec<(i32, i32)>) -> Vec<(i32, i32)> {
    let all: std::collections::HashSet<(i32, i32)> = px.iter().copied().collect();
    let mut body_of: std::collections::HashMap<(i32, i32), usize> = std::collections::HashMap::new();
    let mut sizes: Vec<usize> = Vec::new();
    for &start in &px {
        if body_of.contains_key(&start) {
            continue;
        }
        let id = sizes.len();
        let (mut todo, mut n) = (vec![start], 0);
        body_of.insert(start, id);
        while let Some((x, y)) = todo.pop() {
            n += 1;
            for (dx, dy) in [(1, 0), (-1, 0), (0, 1), (0, -1), (1, 1), (1, -1), (-1, 1), (-1, -1)] {
                let p = (x + dx, y + dy);
                if all.contains(&p) && !body_of.contains_key(&p) {
                    body_of.insert(p, id);
                    todo.push(p);
                }
            }
        }
        sizes.push(n);
    }
    let Some(best) = (0..sizes.len()).max_by_key(|&id| (sizes[id], std::cmp::Reverse(id))) else { return px };
    px.into_iter().filter(|p| body_of[p] == best).collect()
}

/// How a map's rocks are laid out (2026-11-17). Henry: "the asteroids make a big S on the map,
/// and there's empty space otherwise"; "like an S, a donut, or two triangles, chosen randomly";
/// and "let's keep the existing map style as a fourth option."
#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash)]
pub enum Layout {
    /// Rocks scattered all over the map, a giant in the middle: every map until 2026-11-17.
    Scatter,
    /// One big S across the map, its tails by the two kings, a big rock where its curves meet.
    Ess,
    /// A ring round the map's middle, which is empty.
    Ring,
    /// Two triangles, one each side, with open space between them.
    Triangles,
}

pub const LAYOUTS: [Layout; 4] = [Layout::Scatter, Layout::Ess, Layout::Ring, Layout::Triangles];

impl Layout {
    pub fn name(self) -> &'static str {
        match self {
            Layout::Scatter => "SCATTERED",
            Layout::Ess => "AN S",
            Layout::Ring => "A RING",
            Layout::Triangles => "TWO TRIANGLES",
        }
    }
    /// A layout by the word the tools take for it: scatter, s, ring, triangles.
    pub fn from_name(name: &str) -> Option<Layout> {
        match name.to_ascii_lowercase().as_str() {
            "scatter" | "scattered" => Some(Layout::Scatter),
            "s" | "ess" => Some(Layout::Ess),
            "ring" | "donut" => Some(Layout::Ring),
            "triangles" | "triangle" => Some(Layout::Triangles),
            _ => None,
        }
    }
    /// The layout of the map a match on `seed` is played on: one of the four, as the seed
    /// falls, and the same on every machine.
    pub fn pick(seed: u64) -> Layout {
        LAYOUTS[(Rng::keyed([seed, 0x4c41_594f_5554, 0, 0]).next_u64() % LAYOUTS.len() as u64) as usize]
    }
}

/// A figure's rocks: most are small (their radius, least and most), one in seven is big, and
/// between one and the next along the figure there's this much open space (least and most).
/// With these a figure holds about as much rock as a scattered map does.
const FIGURE_SMALL: (i32, i32) = (7, 13);
const FIGURE_BIG: (i32, i32) = (18, 24);
const FIGURE_GAP: (i32, i32) = (16, 44);
/// The big rock where an S's two curves meet.
const ESS_MIDDLE: i32 = 26;
/// No rock's edge is nearer a king's start than this, on any map.
const KING_CLEAR: i32 = 90;

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
}

#[derive(Clone, Hash)]
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
    /// The strategy each side's AI plays its team by (none: the AI as it always was).
    pub strategy: [Option<Strategy>; 2],
    mind: [Mind; 2],
    /// Network id per ship (usize::MAX: not a node).
    pub net: Vec<usize>,
    /// The tick each team's network last broke in two (0: never): two of its nodes parted, or
    /// its king was cut off by the loss of a node. A king that flies out of reach by itself
    /// hasn't broken anything.
    pub split_at: [u32; 2],
    pub tally: [Tally; 2],
    pub rocks: Vec<Rock>,
    /// How those rocks are laid out.
    pub layout: Layout,
    /// Pixels each side has taken out of its banks to print with, and pixels lost with destroyed
    /// nodes (spilled or not). With what's mined and what's stored, these must always add up.
    pub spent: [i32; 2],
    pub spilt: [i32; 2],
    /// Pixels each king has made by itself.
    pub made: [i32; 2],
    /// Pixels handed out to printers so far: who's first in line moves round with it.
    turn: usize,
    /// For a side whose AI sends its warships by playing the match forward (see `dispatch`):
    /// what its dispatcher remembers. None: the AI sends them by its rules of thumb, in waves.
    pub dispatch: [Option<Dispatch>; 2],
    /// Each side's AI rules switched off, for measuring: a bit for each (`RULE_MINERS` and the
    /// rest). Always 0 in play.
    pub off: [u32; 2],
}

impl Card {
    pub fn of(d: &Design) -> Card {
        let st = d.stats();
        Card { ok: d.check().ok(), count: st.count, cost: st.cost, seeded: d.seeded, st }
    }
    /// What the king pays to print it. A ship is paid for a pixel at a time as it's printed; a
    /// seeded design is paid for in full, everything it will grow into, before its seed sets off.
    pub fn price(&self) -> i32 {
        self.cost
    }
    /// A ship that mines: it flies and has mining equipment.
    pub fn miner(&self) -> bool {
        self.ok && !self.seeded && self.st.thrust > 0 && self.st.gathers()
    }
    /// A ship that flies with a tractor and no scoop: it follows the miners.
    pub fn collector(&self) -> bool {
        self.ok && !self.seeded && self.st.collector()
    }
    /// A ship that flies and fights: armed, with no mining equipment and no tractor (a
    /// collector with guns stays with the miners).
    pub fn warship(&self) -> bool {
        self.ok && !self.seeded && self.st.armed() && self.st.thrust > 0 && !self.st.gathers() && self.st.tractor + self.st.dock == 0
    }
    /// A seeded design that prints.
    pub fn factory(&self) -> bool {
        self.ok && self.seeded && self.st.factory > 0
    }
    /// Whether its ships do as their side's stance says while they haven't been sent anywhere:
    /// anything that flies, unless it has mining equipment. (Henry, 2026-11-14: "ATTACK should
    /// send everything unless it has mining equipment. miners should still mine.")
    pub fn stanced(&self) -> bool {
        self.ok && !self.seeded && self.st.thrust > 0 && !self.st.gathers() && self.st.tractor + self.st.dock == 0
    }

}

impl Arena {
    /// A match on a mirrored map: each side starts with its king and a bank, nothing else.
    /// (The tests' way in: the game goes through `for_map`.)
    #[cfg(test)]
    pub fn for_match(seed: u64, deck: [Vec<Design>; 2], ai: [bool; 2]) -> Arena {
        Arena::for_bots(seed, deck, [if ai[0] { Bot::Ai } else { Bot::None }, if ai[1] { Bot::Ai } else { Bot::None }])
    }

    /// The same, saying who plays each side: on a map of scattered rocks, as every map was
    /// until 2026-11-17 and as the tests' maps still are. (The game picks a layout by the
    /// seed: `Layout::pick`, `Arena::for_map`.)
    #[cfg(test)]
    pub fn for_bots(seed: u64, deck: [Vec<Design>; 2], bot: [Bot; 2]) -> Arena {
        Arena::for_map(seed, Layout::Scatter, deck, bot)
    }

    /// A match on a map whose rocks are laid out as `layout`.
    pub fn for_map(seed: u64, layout: Layout, deck: [Vec<Design>; 2], bot: [Bot; 2]) -> Arena {
        let mut a = Arena::blank(MW, MH, seed);
        // The map's own dice: they make the rocks and stars, and nothing in the match rolls them.
        let mut rng = Rng::new(seed);
        let rocks = match layout {
            Layout::Scatter => a.match_rocks(&mut rng),
            figure => a.figure_rocks(figure, &mut rng),
        };
        for _ in 0..900 {
            let (x, y) = (rng.range(0, MW - 1), rng.range(0, MH - 1));
            let c = if rng.chance(15) { 2 } else if rng.chance(40) { 1 } else { 0 };
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
        let cards: [Vec<Card>; 2] = [deck[0].iter().map(Card::of).collect(), deck[1].iter().map(Card::of).collect()];
        // What each card's ships start on follows from its drawing, on every side: what flies
        // with no mining equipment guards where it was built, what flies with a scoop mines, and
        // what can't fly holds where it is. After that a person's stance moves the first kind
        // all together, and an AI orders its cards one by one.
        let orders: [Vec<Order>; 2] = [0, 1].map(|t| cards[t].iter().map(|c| if c.stanced() { Order::Guard } else { c.st.order() }).collect());
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
            strategy: [None, None],
            mind: [Mind::default(), Mind::default()],
            net: Vec::new(),
            split_at: [0; 2],
            tally: [Tally::default(); 2],
            layout,
            rocks,
            spent: [0; 2],
            spilt: [0; 2],
            made: [0; 2],
            turn: 0,
            dispatch: [None, None],
            off: [0; 2],
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
    fn match_rocks(&mut self, rng: &mut Rng) -> Vec<Rock> {
        let (w, h) = (self.w, self.h);
        let mut rocks = Vec::new();
        let mut placed: Vec<(i32, i32, i32)> = Vec::new();
        // The giant: point-symmetric itself, so it's its own twin.
        let (cx, cy, r) = (w / 2, h / 2, 30);
        let half: Vec<(i32, i32)> = Arena::blob(rng, cx, cy, r).into_iter().filter(|&(x, y)| y < cy || (y == cy && x <= cx)).collect();
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
                let r = rng.range(lo, hi);
                let (x, y) = (rng.range(r + 14, w - r - 15), rng.range(r + 14, h - r - 15));
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
            let px = Arena::blob(rng, x, y, r);
            let twin: Vec<(i32, i32)> = px.iter().map(|&(a, b)| (w - 1 - a, h - 1 - b)).collect();
            self.put_rock(px, &mut rocks);
            self.put_rock(twin, &mut rocks);
            placed.push((x, y, r));
        }
        rocks
    }

    /// A map whose rocks are strung along one big figure, with open space everywhere else: an
    /// S, a ring or two triangles. Half the figure is walked, a rock put down every so often
    /// along it, and the other half is that half turned half round about the map's middle,
    /// rock for rock, so neither side has the better of it. No rock is nearer a king's start
    /// than `KING_CLEAR`, and each figure comes within reach of a king's first factory.
    fn figure_rocks(&mut self, layout: Layout, rng: &mut Rng) -> Vec<Rock> {
        let (w, h) = (self.w, self.h);
        let (cx, cy) = (w / 2, h / 2);
        let mirror = |(x, y): (i32, i32)| (w - 1 - x, h - 1 - y);
        // A circle's points in order, clockwise from the top, for the two round figures: point
        // `k` of `n` is `k / n` of the way round. No sines, so every machine makes the same map.
        let round = circle_round(1000);
        let n = round.len();
        let on_oval = |k: usize, (ox, oy): (i32, i32), (rx, ry): (i32, i32)| (ox + round[k % n].0 * rx / 1000, oy + round[k % n].1 * ry / 1000);
        // Half the figure, as points a pixel or so apart.
        let path: Vec<(i32, i32)> = match layout {
            // The S's upper curve: an oval above the map's middle, from its tail up and to the
            // right (by red's king), back over the top, down its left side and in to the
            // middle of the map. Its mirror is the lower curve, with its tail by blue's king.
            Layout::Ess => {
                let (rx, ry) = (w * 5 / 14, h * 4 / 21);
                let from = n * 65 / 360;
                (0..=n * 245 / 360).map(|k| on_oval(from + n - k, (cx, cy - ry), (rx, ry))).collect()
            }
            // The ring's right half, from the top round to the bottom.
            Layout::Ring => (0..n / 2).map(|k| on_oval(k, (cx, cy), (w * 3 / 7, h * 2 / 5))).collect(),
            // Blue's triangle: a corner by its king, one along the bottom of the map and one up
            // its left side, so its long side faces the middle. Red's is its mirror.
            Layout::Triangles => {
                let corners = [(w * 11 / 70, h * 27 / 35), (w * 18 / 35, h * 31 / 35), (w * 5 / 28, h * 11 / 35)];
                let mut pts = Vec::new();
                for k in 0..3 {
                    let (from, to) = (corners[k], corners[(k + 1) % 3]);
                    let mut l = crate::geom::Line::new(from.0, from.1, to.0, to.1);
                    while (l.x, l.y) != to {
                        pts.push((l.x, l.y));
                        l.step();
                    }
                }
                pts
            }
            Layout::Scatter => Vec::new(),
        };
        let middle = if layout == Layout::Ess { ESS_MIDDLE } else { 0 };
        // Whether a rock of radius `r` may stand at `p`: on the map, clear of both kings, of
        // the S's middle rock, of every rock put down so far and of their mirrors, and of its
        // own mirror.
        let clear = |p: (i32, i32), r: i32, placed: &[(i32, i32, i32)]| {
            let apart = |q: (i32, i32), far: i32| dist2(p.0, p.1, q.0, q.1) >= far * far;
            (r + 14..w - r - 14).contains(&p.0)
                && (r + 14..h - r - 14).contains(&p.1)
                && KING_STARTS.iter().all(|&k| apart(k, r + KING_CLEAR))
                && (middle == 0 || apart((cx, cy), r + middle + FIGURE_GAP.0))
                && apart(mirror(p), 2 * r + FIGURE_GAP.0)
                && placed.iter().all(|&(x, y, r2)| apart((x, y), r + r2 + FIGURE_GAP.0 / 2) && apart(mirror((x, y)), r + r2 + FIGURE_GAP.0 / 2))
        };
        let radius = |rng: &mut Rng| if rng.chance(14) { rng.range(FIGURE_BIG.0, FIGURE_BIG.1) } else { rng.range(FIGURE_SMALL.0, FIGURE_SMALL.1) };
        let mut placed: Vec<(i32, i32, i32)> = Vec::new();
        let (mut r, mut last, mut step) = (radius(rng), None::<(i32, i32)>, 0);
        for &p in &path {
            if last.is_some_and(|l| dist2(p.0, p.1, l.0, l.1) < step * step) {
                continue;
            }
            // A little off the line, so the figure is made of rocks and not ruled.
            let at = (p.0 + rng.range(-6, 6), p.1 + rng.range(-6, 6));
            if clear(at, r, &placed) {
                placed.push((at.0, at.1, r));
            }
            // On to the next, a rock's width and a gap further along, whether this one stood
            // or not.
            last = Some(p);
            let next = radius(rng);
            step = r + next + rng.range(FIGURE_GAP.0, FIGURE_GAP.1);
            r = next;
        }
        let mut rocks = Vec::new();
        if middle > 0 {
            // The rock in the middle is its own mirror image, as a scattered map's giant is.
            let half: Vec<(i32, i32)> = Arena::blob(rng, cx, cy, middle).into_iter().filter(|&(x, y)| y < cy || (y == cy && x <= cx)).collect();
            let mut whole = half.clone();
            whole.extend(half.iter().map(|&p| mirror(p)));
            whole.sort();
            whole.dedup();
            self.put_rock(whole, &mut rocks);
        }
        for (x, y, r) in placed {
            let px = Arena::blob(rng, x, y, r);
            let twin: Vec<(i32, i32)> = px.iter().map(|&p| mirror(p)).collect();
            self.put_rock(px, &mut rocks);
            self.put_rock(twin, &mut rocks);
        }
        rocks
    }

    /// A rough disc of rock: a ragged edge, solid inside.
    fn blob(rng: &mut Rng, cx: i32, cy: i32, r: i32) -> Vec<(i32, i32)> {
        let mut px = Vec::new();
        for y in -r..=r {
            for x in -r..=r {
                let d = dist2(x, y, 0, 0);
                let edge = d > (r - 2) * (r - 2);
                if d <= r * r && !(edge && rng.chance(35)) {
                    px.push((cx + x, cy + y));
                }
            }
        }
        px
    }

    fn put_rock(&mut self, px: Vec<(i32, i32)>, rocks: &mut Vec<Rock>) {
        let px: Vec<(i32, i32)> = px.into_iter().filter(|&(x, y)| (0..self.w).contains(&x) && (0..self.h).contains(&y)).collect();
        // A rock is one body. Its ragged edge leaves a speck or two standing clear of the rest
        // (twelve on the map of Henry's match of 2026-11-17), and to a seed that lands on one,
        // a speck is the whole rock: those are left out.
        let px = one_body(px);
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

    /// Every pixel of the rock `start` is part of: the rock pixels touching it, edge to edge or
    /// corner to corner, and the ones touching those. What looks like one rock is one rock.
    /// (Edge to edge only, until 2026-11-17. Every rock's outline has a few pixels that hang on
    /// by a corner, 278 of the 22,496 on the map of Henry's match that day, and each counted
    /// as a rock of its own: a seed whose core came down on one found a single pixel to eat
    /// and grew the rest of its factory out of the bank, at whatever came in.)
    pub fn rock_blob(&self, start: (i32, i32)) -> Vec<(i32, i32)> {
        let mut seen = std::collections::HashSet::new();
        let mut todo = vec![start];
        let mut out = Vec::new();
        seen.insert(start);
        while let Some((x, y)) = todo.pop() {
            out.push((x, y));
            for (dx, dy) in [(1, 0), (-1, 0), (0, 1), (0, -1), (1, 1), (1, -1), (-1, 1), (-1, -1)] {
                let n = (x + dx, y + dy);
                if self.rock_at(n.0, n.1) && seen.insert(n) {
                    todo.push(n);
                }
            }
        }
        out
    }

    // ----- The tick -----

    /// A design's standing order changes: every ship of it takes the new one, and so will the
    /// ones still to be printed. (Ships sent out used to gather into groups first. Group size
    /// went with the manual flight patterns, 2026-11-14.)
    fn set_order(&mut self, team: usize, slot: usize, order: Order) {
        if self.eco().orders[team].get(slot).map_or(true, |&o| o == order) {
            return;
        }
        let e = self.eco_mut();
        e.orders[team][slot] = order;
        e.tally[team].orders += 1;
        for s in self.ships.iter_mut().filter(|s| s.alive && s.team == team && s.slot == slot && s.role == Role::Ship) {
            s.order = order;
        }
    }

    /// Whether a side's unsent ships are out attacking: it has cards that take the stance, and
    /// they're all on HUNT (or SAB).
    pub fn attacking(&self, team: usize) -> bool {
        let e = self.eco();
        let mut stanced = (0..e.cards[team].len()).filter(|&s| e.cards[team][s].stanced()).peekable();
        stanced.peek().is_some() && stanced.all(|s| matches!(e.orders[team][s], Order::Hunt | Order::Sab))
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
                Cmd::Sow { team, slot, at } => {
                    let Ok(cost) = self.can_sow(team, slot, at) else { continue };
                    let k = self.eco().king[team];
                    for _ in 0..cost {
                        let paid = self.take_px(k);
                        debug_assert!(paid, "the bank was counted before the seed was sold");
                    }
                    let i = self.deliver(team, Job { slot, target: Some(at), paid: cost, cost, repeat: false });
                    self.ships[i].sow = true;
                }
                Cmd::Send { team, ship, at, pace } => {
                    // Only a ship of that side's, finished and able to fly, and only to a place
                    // on the map. Anything else is dropped: what was true when the order was
                    // given may not be by the time it's applied.
                    let (w, h) = (self.w, self.h);
                    let Some(s) = self.ships.get_mut(ship).filter(|s| s.commandable(team)) else { continue };
                    (s.sent, s.pace, s.chase) = (Some((at.0.clamp(0, w - 1), at.1.clamp(0, h - 1))), pace, None);
                }
                Cmd::Home { team, ship } => {
                    let Some(s) = self.ships.get_mut(ship).filter(|s| s.commandable(team)) else { continue };
                    (s.sent, s.pace, s.chase) = (None, 0, None);
                }
                Cmd::Follow { team, ship, target, pace } => {
                    // After anything that's still there, of either side, but itself.
                    let Some(at) = self.ships.get(target).filter(|t| t.alive && target != ship).map(|t| (t.x, t.y)) else { continue };
                    let Some(s) = self.ships.get_mut(ship).filter(|s| s.commandable(team)) else { continue };
                    (s.sent, s.pace, s.chase) = (Some(at), pace, Some(target));
                }
                Cmd::Order { team, slot, order } => {
                    // A card's own order is an AI's to give. A person's side has the stance and
                    // nothing else: from one, this is dropped.
                    if self.eco().bot[team] != Bot::None {
                        self.set_order(team, slot, order);
                    }
                }
                Cmd::Stance { team, attack } => {
                    let order = if attack { Order::Hunt } else { Order::Guard };
                    let stanced: Vec<usize> = (0..self.eco().cards[team].len()).filter(|&s| self.eco().cards[team][s].stanced()).collect();
                    for slot in stanced {
                        self.set_order(team, slot, order);
                    }
                }
                Cmd::Rally { team, ship, at } => {
                    let (w, h) = (self.w, self.h);
                    let Some(s) = self.ships.get_mut(ship).filter(|s| s.rallies(team)) else { continue };
                    let at = (at.0.clamp(0, w - 1), at.1.clamp(0, h - 1));
                    let (l, t, r, b) = s.bbox();
                    s.rally = (!((l..=r).contains(&at.0) && (t..=b).contains(&at.1))).then_some(at);
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

    /// Tractors pull (2026-11-16). Every ship with tractor pixels working takes a pixel off a
    /// ship of its side in its reach that's carrying any, one every `TRACTOR_TICKS` for each
    /// tractor pixel it has, into its own store: that's when the pixel is banked. Of the ships
    /// waiting to be emptied it takes from the one with the least left, so that one is away
    /// soonest; with none waiting, from the one carrying most, which is a miner still at work
    /// in its reach. A tractor with nothing to pull is ready to pull at once, and no more than
    /// that: its idle time isn't saved up.
    fn tractors(&mut self) {
        let tick = self.tick;
        for j in 0..self.ships.len() {
            let px = self.tractor_px(j);
            if px == 0 {
                self.ships[j].pull = 0;
                continue;
            }
            let (team, x, y, reach) = (self.ships[j].team, self.ships[j].x, self.ships[j].y, self.tractor_reach(j));
            let mut pull = self.ships[j].pull + px;
            while pull >= TRACTOR_TICKS {
                let from = (0..self.ships.len())
                    .filter(|&m| {
                        let o = &self.ships[m];
                        o.alive && o.team == team && o.carry > 0 && o.warp == 0 && o.maker.is_none() && dist2(o.x, o.y, x, y) <= reach * reach
                    })
                    .min_by_key(|&m| {
                        let o = &self.ships[m];
                        if o.hauling {
                            (0, o.carry, m)
                        } else {
                            (1, -o.carry, m)
                        }
                    });
                let Some(m) = from else {
                    pull = TRACTOR_TICKS;
                    break;
                };
                pull -= TRACTOR_TICKS;
                self.ships[m].carry -= 1;
                self.ships[m].pulled = Some((j, tick));
                self.ships[j].store += 1;
                self.score[team].mined += 1;
            }
            self.ships[j].pull = pull;
        }
    }

    /// Dock pads at work. A pad takes hold of a miner of its side that's waiting by its ship
    /// with a load, when there's a place for it: pad under every pixel of the miner, turned
    /// whichever way fits, clear of the miners already on the pad and of pad pixels that are
    /// resting. From then the pad moves the miner, `DOCK_SPEED` px a tick: on to its place;
    /// there, with the pad's ship at rest, everything it carries goes into that ship's store
    /// at once (that's when it's banked), and the pad pixels under it start their rest; and
    /// when the sweep has shown, off again to where a miner waits, on the side of the rock
    /// it's working. A pad with room takes several miners side by side.
    fn docks(&mut self) {
        let n = self.ships.len();
        // Pad pixels rest.
        for j in 0..n {
            if self.ships[j].st.dock == 0 {
                continue;
            }
            let s = &mut self.ships[j];
            for c in 0..s.cool.len() {
                if s.shape.cells[c] == Some(Part::DockPad) && s.cool[c] > 0 {
                    s.cool[c] -= 1;
                }
            }
        }
        // The miners a pad has hold of.
        for i in 0..n {
            let Some(mut b) = self.ships[i].berth.filter(|_| self.ships[i].alive) else {
                self.ships[i].berth = None;
                continue;
            };
            let j = b.ship;
            let o = &self.ships[j];
            // Its pad's ship gone, or turned under it: it's let go where it is.
            if !o.alive || o.facing != b.facing {
                self.ships[i].berth = None;
                continue;
            }
            let (spot, still) = ((o.x + b.at.0, o.y + b.at.1), (o.vx, o.vy) == (0, 0));
            let step = |a: &mut Arena, to: (i32, i32)| {
                let (x, y) = (a.ships[i].x, a.ships[i].y);
                let (dx, dy) = ((to.0 - x).clamp(-DOCK_SPEED, DOCK_SPEED), (to.1 - y).clamp(-DOCK_SPEED, DOCK_SPEED));
                a.shift(i, x + dx, y + dy) != (x, y)
            };
            match b.phase {
                0 => {
                    if self.ships[i].facing != b.turn {
                        self.try_turn(i, b.turn);
                    }
                    step(self, spot);
                    // Down: every pixel of it has to have a pad pixel under it still.
                    if (self.ships[i].x, self.ships[i].y) == spot && self.ships[i].facing == b.turn && still {
                        let under: Vec<Option<usize>> = (0..self.ships[i].hp.len()).filter(|&c| self.ships[i].hp[c] > 0).map(|c| self.ships[i].cell_xy(c)).map(|(x, y)| self.ships[j].cell_at(x, y).filter(|&k| self.ships[j].hp[k] > 0 && self.ships[j].shape.cells[k] == Some(Part::DockPad))).collect();
                        if under.iter().any(|k| k.is_none()) {
                            // Its place was shot out from under it: it's let go, to try again.
                            self.ships[i].berth = None;
                            continue;
                        }
                        for k in under.into_iter().flatten() {
                            self.ships[j].cool[k] = DOCK_TURN;
                        }
                        let (team, load) = (self.ships[i].team, self.ships[i].carry);
                        let m = &mut self.ships[i];
                        (m.carry, m.hauling, m.dock) = (0, false, None);
                        self.ships[j].store += load;
                        self.score[team].mined += load;
                        self.score[team].landed += load;
                        // Where it's put off: clear of the pad's ship, towards its work.
                        let (jx, jy) = (self.ships[j].x, self.ships[j].y);
                        let to = self.ships[i].worked.unwrap_or(self.ships[i].home);
                        let (far, wait) = (isqrt(dist2(to.0, to.1, jx, jy)).max(1), self.pad_wait(i, j));
                        let off = if (to.0, to.1) == (jx, jy) { (jx + wait, jy) } else { (jx + (to.0 - jx) * wait / far, jy + (to.1 - jy) * wait / far) };
                        (b.phase, b.t, b.off) = (1, DOCK_SHOW, off);
                    }
                }
                1 => {
                    step(self, spot);
                    b.t -= 1;
                    if b.t <= 0 {
                        b.phase = 2;
                    }
                }
                _ => {
                    // Off, and let go there; or where it is, if something solid is in the way.
                    if !step(self, b.off) || (self.ships[i].x, self.ships[i].y) == b.off {
                        self.ships[i].berth = None;
                        continue;
                    }
                }
            }
            self.ships[i].berth = Some(b);
        }
        // Miners waiting by a pad's ship that's at rest: the nearest first, while there's a
        // place.
        for j in 0..n {
            if self.ships[j].st.dock == 0 || !self.ships[j].alive || (self.ships[j].vx, self.ships[j].vy) != (0, 0) {
                continue;
            }
            let (jx, jy, facing) = (self.ships[j].x, self.ships[j].y, self.ships[j].facing);
            let mut waiting: Vec<(i32, usize)> = (0..n)
                .filter(|&i| {
                    let m = &self.ships[i];
                    m.alive && m.hauling && m.carry > 0 && m.dock == Some(j) && m.berth.is_none() && m.vx.abs() <= 4 * DOCK_SPEED && m.vy.abs() <= 4 * DOCK_SPEED
                })
                .map(|i| (isqrt(dist2(self.ships[i].x, self.ships[i].y, jx, jy)), i))
                .filter(|&(d, i)| d <= self.pad_wait(i, j) + DOCK_GRAB)
                .collect();
            waiting.sort();
            for (_, i) in waiting {
                let Some((spot, turn)) = self.pad_slot(i, j) else { continue };
                let m = &mut self.ships[i];
                (m.vx, m.vy, m.sx, m.sy, m.want) = (0, 0, 0, 0, (0, 0));
                m.berth = Some(Berth { ship: j, facing, at: (spot.0 - jx, spot.1 - jy), turn, phase: 0, t: 0, off: (0, 0) });
            }
        }
    }

    pub(super) fn eco_post(&mut self) {
        self.networks();
        self.spills();
        self.tractors();
        self.docks();
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
                // they print instead. With no rock left under it, its network feeds it.
                let wedged = self.growth_step(i);
                if self.bank_fed(i) {
                    printers.push(Printer::Grower(i));
                } else if wedged && self.factory_ready(i) {
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
        let mut pace = if slowed { KING_SLOW } else { KING_SPEED };
        // A king that's been sent somewhere flies there while its keys are let go: along both
        // axes until it's level with the place, then straight, and it stops when it arrives.
        // With ships sent beside it, it keeps their pace. A key takes it back at once.
        let (dx, dy) = if (dx, dy) != (0, 0) {
            (self.ships[k].sent, self.ships[k].pace, self.ships[k].chase) = (None, 0, None);
            (dx, dy)
        } else if let Some((px, py)) = self.ships[k].sent {
            let (ex, ey) = (px - self.ships[k].x, py - self.ships[k].y);
            if ex.abs() <= 1 && ey.abs() <= 1 {
                // Sent to a place, it has arrived. Sent after a thing, it stays with it.
                if self.ships[k].chase.is_none() {
                    (self.ships[k].sent, self.ships[k].pace) = (None, 0);
                }
                (0, 0)
            } else {
                if self.ships[k].pace > 0 {
                    pace = pace.min(self.ships[k].pace);
                }
                (if ex.abs() > 1 { ex.signum() } else { 0 }, if ey.abs() > 1 { ey.signum() } else { 0 })
            }
        } else {
            (0, 0)
        };
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

    pub(super) fn reach(&self, i: usize) -> i32 {
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
                // (Nor has a node that flies by itself, a collector or a carrier, by flying
                // out of reach: 2026-11-16, when collectors began to come and go all match.)
                let flies = |k: usize| self.ships[k].role == Role::King || self.ships[k].st.thrust > 0;
                let flew_off = (flies(i) || flies(j)) && !lost[team];
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

    /// Takes a pixel for printing: from the nodes of its network that fly first, the nearest
    /// of them (Henry, 2026-11-16: "Resources get used from mobile collectors first before
    /// getting used from stationary outposts": a collector is the node easiest to lose, and
    /// what it holds spills). Then from the printer's own store, then from the nearest node in
    /// its network that has one.
    fn take_px(&mut self, i: usize) -> bool {
        let team = self.ships[i].team;
        let (x, y) = (self.ships[i].x, self.ships[i].y);
        let net = self.eco().net.get(i).copied().filter(|&id| id != usize::MAX);
        if let Some(id) = net {
            let flying = (0..self.ships.len()).filter(|&j| self.eco().net.get(j) == Some(&id) && self.ships[j].store > 0 && self.ships[j].role == Role::Ship && self.ships[j].st.thrust > 0).min_by_key(|&j| (dist2(self.ships[j].x, self.ships[j].y, x, y), j));
            if let Some(j) = flying {
                self.ships[j].store -= 1;
                self.eco_mut().spent[team] += 1;
                return true;
            }
        }
        if self.ships[i].store > 0 {
            self.ships[i].store -= 1;
            self.eco_mut().spent[team] += 1;
            return true;
        }
        let Some(id) = net else { return false };
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
            for k in 0..n {
                let mut dice = self.roll(i, k as usize, Roll::Spill);
                let (dx, dy) = (dice.range(-h, h), dice.range(-h, h));
                let drift = 2 + dice.below(6) as i32;
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
    /// grown so far helps, one rock pixel for each pixel grown. It waits while an enemy sits where
    /// its next pixels would go (returning true for a tick it spent waiting). When there's no rock
    /// left to eat, or never was (a seed sent into open space), a factory's network pays for the
    /// rest a pixel at a time (`bank_fed`, `Printer::Grower`): the bank supplies what a rock would
    /// have. Something with no factory pixels is in no network, and stops as it is.
    fn growth_step(&mut self, i: usize) -> bool {
        let f = self.ships[i].st.factory;
        let mut food = {
            let g = self.ships[i].grow.as_mut().unwrap();
            g.acc += GROW_RATE * UNIT / 15 + f;
            std::mem::take(&mut g.food)
        };
        // Rock pixels mined away since it landed are no longer there to eat.
        food.retain(|&(x, y)| self.rock_at(x, y));
        let (mut done, mut waiting) = (false, false);
        while self.ships[i].grow.as_ref().is_some_and(|g| g.acc >= UNIT) {
            let Some(&(x, y)) = food.last() else {
                done = !self.ships[i].node;
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

    /// Whether a growing factory has nothing left to grow from but its network's bank.
    fn bank_fed(&self, i: usize) -> bool {
        let s = &self.ships[i];
        s.node && s.grow.as_ref().is_some_and(|g| g.eats && !g.pending.is_empty() && g.food.is_empty())
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
            // What's ahead of it, in order: the rest of the start (once), then the round from
            // its beginning; or, once it's in the round, the rest of the round and round
            // again to where it is. With the start done and no round, nothing.
            let (at, round) = (s.build_i.min(n), s.loop_from.min(n));
            let ahead = (at..n).chain(round..at.max(round));
            let Some(k) = ahead.into_iter().find(|&k| fits(s.build[k])) else { return false };
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
            Printer::Grower(i) => {
                if self.ships[i].grow.as_ref().map_or(true, |g| g.acc < UNIT) {
                    return false;
                }
                match self.next_cell(i) {
                    Next::Cell(k) => {
                        // Short of pixels, it waits, as a factory printing does.
                        if !self.take_px(i) {
                            self.ships[i].grow.as_mut().unwrap().acc = UNIT;
                            return false;
                        }
                        self.grow_cell(i, k);
                        let g = self.ships[i].grow.as_mut().unwrap();
                        g.acc -= UNIT;
                        if g.pending.is_empty() {
                            self.ships[i].grow = None;
                            return false;
                        }
                        g.acc >= UNIT
                    }
                    Next::Blocked => {
                        self.ships[i].grow.as_mut().unwrap().acc = UNIT;
                        false
                    }
                    Next::Done => {
                        self.ships[i].grow = None;
                        false
                    }
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
        // It was built here: its home is its factory, and its post is in the rings round it. A
        // factory with a rally point sends it there instead: that place is its home, and its
        // post is in the rings round that.
        let (home, ring0) = match self.ships[i].rally {
            Some(at) => (at, POST_GAP),
            None => ((self.ships[i].x, self.ships[i].y), self.ships[i].half() + POST_GAP),
        };
        let s = &mut self.ships[c];
        s.grow = None;
        s.maker = None;
        s.home = home;
        s.ring0 = ring0;
        let team = s.team;
        let f = &mut self.ships[i];
        f.making = Making::None;
        // On to the next: and from the end of the round, back to where the round begins
        // (which is the end too, if it has a start and no round).
        f.build_i = if f.build_i + 1 >= f.build.len() { f.loop_from.min(f.build.len()) } else { f.build_i + 1 };
        self.score[team].placed += 1;
        self.eco_mut().tally[team].printed += 1;
    }

    /// What a king finishes printing arrives by hyperspace beside it.
    fn deliver(&mut self, t: usize, job: Job) -> usize {
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
        i
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
        Some((start, self.mid_from(team, slot, start)?))
    }

    /// The middle of a seeded design whose core comes down on the pixel `start`.
    pub fn mid_from(&self, team: usize, slot: usize, start: (i32, i32)) -> Option<(i32, i32)> {
        let d = self.eco().deck[team].get(slot)?;
        let (ox, oy) = d.shape().cell_xy(0, 0, 0, d.start_cell()?);
        Some((start.0 - ox, start.1 - oy))
    }

    /// Whether a king may send a seeded design's seed to the spot `at` in open space, and what
    /// the seed costs: the design's whole price, as on a rock. (What it then grows costs as much
    /// again, a pixel at a time from its network, where a rock would have supplied it.)
    pub fn can_sow(&self, t: usize, slot: usize, at: (i32, i32)) -> Result<i32, &'static str> {
        let card = *self.eco().cards.get(t).and_then(|c| c.get(slot)).ok_or("NO SUCH DESIGN")?;
        if !card.ok {
            return Err("THAT DESIGN CAN'T FLY YET");
        }
        if !card.seeded {
            return Err("ONLY A SEEDED DESIGN IS PLANTED");
        }
        self.sow_links(t, slot, at)?;
        if self.bank(self.eco().king[t]) < card.price() {
            return Err("NOT ENOUGH IN THE BANK: A SEED IS PAID FOR IN FULL");
        }
        Ok(card.price())
    }

    /// What a seed sent to the spot `at` in open space would link to, or why it can't go there.
    /// It has to be a factory (only a network can feed it), the whole drawing has to have room
    /// on the map clear of rock and of everything planted, and it has to link to a node of its
    /// side's, standing or on its way: out of reach of its network there'd be nothing to grow from.
    pub fn sow_links(&self, t: usize, slot: usize, at: (i32, i32)) -> Result<Links, &'static str> {
        let card = self.eco().cards.get(t).and_then(|c| c.get(slot)).ok_or("NO SUCH DESIGN")?;
        if card.st.factory == 0 {
            return Err("WITH NO FACTORY PIXELS IT HAS NO NETWORK TO FEED IT: IT NEEDS A ROCK");
        }
        let mid = self.mid_from(t, slot, at).ok_or("THAT DESIGN CAN'T FLY YET")?;
        let sh = self.eco().deck[t][slot].shape();
        for c in (0..sh.cells.len()).filter(|&c| sh.cells[c].is_some()) {
            let (x, y) = sh.cell_xy(mid.0, mid.1, 0, c);
            if x < 0 || y < 0 || x >= self.w || y >= self.h {
                return Err("IT WOULD HANG OFF THE EDGE OF THE MAP");
            }
            if self.rock_at(x, y) {
                return Err("THERE'S ROCK UNDER IT: PLANT ON THE ROCK, OR MOVE CLEAR OF IT");
            }
            // The whole of anything planted counts, what it has still to grow included.
            if self.ships.iter().any(|s| s.alive && s.rooted && s.cell_at(x, y).is_some()) {
                return Err("SOMETHING PLANTED IS IN THE WAY");
            }
        }
        let links = self.links_at(t, mid);
        if links.to.is_empty() {
            return Err("TOO FAR FROM YOUR NETWORK: NOTHING THERE COULD FEED IT");
        }
        Ok(links)
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
        Some(self.links_at(team, at))
    }

    /// What a factory whose middle is `at` would link to, as things stand.
    fn links_at(&self, team: usize, at: (i32, i32)) -> Links {
        let e = self.eco();
        let links = |slot: usize| e.cards[team].get(slot).is_some_and(|c| c.st.factory > 0);
        // Everything of the side's that links: x, y, reach, and whether it's the king.
        let mut sites: Vec<(i32, i32, i32, bool)> =
            (0..self.ships.len()).filter(|&i| self.is_node(i) && self.ships[i].team == team).map(|i| (self.ships[i].x, self.ships[i].y, self.reach(i), self.ships[i].role == Role::King)).collect();
        for s in self.ships.iter().filter(|s| s.alive && s.team == team && s.role == Role::Seed) {
            let Some((sl, t)) = s.seed_for.filter(|f| links(f.0)) else { continue };
            // A seed bound for open space comes down just where it was sent.
            let mid = if s.sow { self.mid_from(team, sl, t) } else { self.landing(team, sl, t).map(|l| l.1) };
            if let Some((x, y)) = mid {
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
        Links { at, to, king, gap }
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
        //
        // Reaching it is being within 2 px of it, or having come to rest as near it as a ship
        // of its sort can place itself (`near_enough`: 3 px for a seed). That's where its
        // flight leaves it, and stopped that near it flies no further. (With only the 2 px, a
        // seed that stopped 3 px off sat by its rock for good: Henry's match of 2026-11-17,
        // and 4 seeds in 135 sent to the pixels of one rock. It still comes down on the pixel
        // it was sent to, not where it stopped.)
        let sow = self.ships[i].sow;
        let (off, at_rest) = (cheb(x, y, target.0, target.1), (self.ships[i].vx, self.ships[i].vy) == (0, 0));
        let there = off <= 2 || (at_rest && off <= near_enough(&self.ships[i].st));
        let shut_out = !sow && self.ships[i].bumped && (!self.rock_at(target.0, target.1) || self.enemy_pixel(team, target.0, target.1));
        if !there && !shut_out {
            return;
        }
        // It lands on the rock pixel it was sent to, or the nearest to that with room (`landing`,
        // which is also what the match screen showed when the rock was picked). Shut out, it
        // lands by where it is.
        // A seed sent into open space comes down on the very pixel it was sent to, once nothing
        // is standing on it.
        let landed = if sow {
            Some(target).filter(|&(tx, ty)| self.room_to_grow(team, None, tx, ty)).and_then(|start| self.mid_from(team, slot, start).map(|mid| (start, mid)))
        } else {
            self.landing(team, slot, if there { target } else { (x, y) })
        };
        let Some((start, mid)) = landed else {
            // None in reach (its rock was mined or eaten away, or is covered): it heads for the
            // nearest free rock left anywhere, or waits where it is if there's none.
            if there && !sow {
                self.ships[i].seed_for = self.free_rock((x, y), team, None).map(|p| (slot, p));
            }
            return;
        };
        // In open space there's no rock to eat: its network feeds it from the first pixel.
        let mut food = if sow { Vec::new() } else { self.rock_blob(start) };
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

    /// Rock pixels left in the rocks a side's miners can work from its stations: every rock whose
    /// edge is within `MINE_REACH` of one of its nodes, its king among them.
    pub fn rock_in_reach(&self, t: usize) -> i32 {
        let nodes: Vec<(i32, i32)> = (0..self.ships.len()).filter(|&j| self.ships[j].team == t && self.is_node(j)).map(|j| (self.ships[j].x, self.ships[j].y)).collect();
        let mut sum = 0;
        for r in &self.eco().rocks {
            let left = self.rock_left(r);
            let far = MINE_REACH + isqrt(left / 3) + 4;
            if left > 0 && nodes.iter().any(|&(x, y)| dist2(x, y, r.cx, r.cy) <= far * far) {
                sum += left;
            }
        }
        sum
    }

    /// A bot's thought, once a second. Everything it does is a command, the same ones a person
    /// gives: steer, print, plant, change a design's standing order.
    fn bot_think(&mut self, t: usize) {
        let bot = self.eco().bot[t];
        // Only the AI plays by a strategy; the scripted openings are the yardstick, and stay put.
        let strategy = self.eco().strategy[t].filter(|_| bot == Bot::Ai);
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
        // What's the AI's to defend: its king, what it has planted, its miners and its
        // collectors, which hold what the miners bit. (For the
        // scripted openings, as always, the king and what's planted.)
        let by_mine = |x: i32, y: i32, range: i32| self.ships.iter().any(|s| s.alive && s.team == t && (s.role == Role::King || s.rooted || (bot == Bot::Ai && s.role == Role::Ship && (is(s, Card::miner) || is(s, Card::collector)))) && cheb(s.x, s.y, x, y) <= range);
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
        // A side with a dispatcher has no waves: every card stays on GUARD, and the dispatcher
        // sends ships where playing the match forward says to.
        let sends = self.eco().dispatch[t].is_some();
        if bot == Bot::Ai && !sends {
            let rest = tick - mind.rested;
            // The edge it waits for, unless its strategy is a bold one.
            let edge = army * 4 >= enemy_army * 5 || (rest > 15 * 120 && army * 5 >= enemy_army * 4) || rest > 15 * 240;
            let go = match strategy {
                Some(p) => tick >= p.first * 15 && army >= p.wave && (p.bold || edge),
                None => army >= WAVE_MIN && edge,
            };
            if mind.wave {
                // The wave is spent, or has been out long enough: home, and mass again.
                if army * 5 < mind.wave_worth * 2 || tick - mind.wave_from > WAVE_TICKS {
                    mind.wave = false;
                    mind.rested = tick;
                }
            } else if go {
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
                if self.eco().tally[t].first_wave == 0 {
                    self.eco_mut().tally[t].first_wave = tick;
                }
            }
        }
        // (Its ships that fly without guns, a shield ship say, take the same orders as its
        // warships: they go with the fleet.)
        for s in (0..cards.len()).filter(|&s| cards[s].stanced()) {
            let out = bot != Bot::Ai || turned_out || (mind.wave && mind.release.get(s).is_some_and(|&r| tick >= r));
            let raids = bot == Bot::Saboteur || strategy.is_some_and(|p| p.raid.contains(&s));
            let want = if sends { Order::Guard } else if raids { Order::Sab } else if out { Order::Hunt } else { Order::Guard };
            if self.eco().orders[t][s] != want {
                self.command(Cmd::Order { team: t, slot: s, order: want });
            }
        }
        if sends {
            self.dispatch(t);
        }

        // Planting: the next factory, as soon as the bank holds its price.
        let mut facts: Vec<usize> = (0..cards.len()).filter(|&s| cards[s].factory()).collect();
        facts.sort_by_key(|&s| cards[s].cost);
        let (small, big) = (facts.first().copied(), facts.last().copied());
        let n = factories + seeds;
        // Money nothing is spending: the bank has held the cheapest factory's price, and what it
        // keeps besides, for a while. That plants another, whatever the most it would have.
        let keep = strategy.map_or(120, |p| p.keep);
        let rich = bot == Bot::Ai && self.eco().off[t] & RULE_SPEND == 0 && small.is_some_and(|s| left >= cards[s].price() + keep + 40);
        mind.rich_since = if !rich { 0 } else if mind.rich_since == 0 { tick.max(1) } else { mind.rich_since };
        let piled = rich && tick - mind.rich_since >= PILE_TICKS && n < PILE_MOST;
        let pick = match bot {
            Bot::Planter | Bot::Boomer => small,
            Bot::Summoner | Bot::Rusher => small.filter(|_| n == 0),
            Bot::Clock => (if n >= 2 { big } else { small }).filter(|_| seeds == 0 && n < (1 + (tick / (15 * 75)) as usize).min(9)),
            _ if strategy.is_some() => {
                // Its strategy's next factory, once that one's time has come.
                let p = strategy.unwrap();
                let (slot, at) = p.plant[n.min(p.plant.len() - 1)];
                Some(slot).filter(|&s| cards.get(s).is_some_and(|c| c.factory()) && (n < p.most || piled) && tick >= at * 15)
            }
            _ => {
                // Two small ones, then big ones: six in all. A big one it has waited four minutes
                // for gives way to a small one. And it doesn't let itself be out-built: while the
                // enemy has more factories than it has, it plants small ones as fast as it can
                // pay for them, past six if need be.
                let waited = tick - mind.last_plant > 15 * 240;
                let behind = theirs > n;
                (if n < 2 || waited || behind { small } else { big }).filter(|_| n < 6 || (behind && n < 14) || piled)
            }
        };
        // It keeps something back for the new factory to print with. The scripted openings
        // with the AI's economy keep more under a raid, as the AI did until 2026-11-12: a side
        // that saves more while it's being raided has less to answer the raid with.
        let spare = match bot {
            Bot::Expander | Bot::Saboteur if alarm => 150,
            Bot::Ai if self.eco().off[t] & RULE_SPARE == 0 => 0,
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
            } else if let Some((slot, at)) = small.or(Some(slot)).filter(|_| bot == Bot::Ai && !outgunned).and_then(|s| self.ai_space(t, s).map(|at| (s, at))) {
                // No rock will take a factory: the AI sends its cheapest one's seed into open
                // space, in reach of its network. (The scripted openings never do.)
                saving = cards[slot].price() + spare;
                if left >= saving {
                    self.command(Cmd::Sow { team: t, slot, at });
                    mind.last_plant = tick;
                    left -= cards[slot].price();
                    saving = 0;
                }
            }
        }

        // The king's own printing, one thing at a time.
        if self.eco().queue[t].is_empty() {
            let cheapest = |f: fn(&Card) -> bool| (0..cards.len()).filter(|&s| f(&cards[s])).min_by_key(|&s| cards[s].cost);
            // A miner earns its price back in about a minute of mining, so the AI keeps printing
            // them while there's rock for them within reach of its stations: unless its strategy
            // keeps them lean for its opening, or the rule's off.
            let lean = strategy.is_some_and(|p| tick < p.lean * 15) || self.eco().off[t] & RULE_MINERS != 0;
            let short_of_miners = if bot == Bot::Ai && !lean { miners < MINERS_MOST && miners as i32 * MINER_ROCK < self.rock_in_reach(t) } else { miners < factories.max(1) + 1 };
            // What its tractors can pull against what its miners bite, counting what's on its
            // king's queue as there already: `RULE_TRACTORS`.
            let pull: i32 = (0..self.ships.len()).filter(|&j| self.ships[j].team == t).map(|j| self.tractor_px(j)).sum::<i32>() + self.eco().queue[t].iter().map(|j| cards[j.slot].st.tractor).sum::<i32>();
            // (What its miners bite goes by their scoops, the ones on its king's queue too.)
            let scoops: i32 = self.ships.iter().filter(|s| own(s) && is(s, Card::miner)).map(|s| s.st.scoops).sum::<i32>() + self.eco().queue[t].iter().filter(|j| cards[j.slot].miner()).map(|j| cards[j.slot].st.scoops).sum::<i32>();
            let short_of_tractors = self.eco().off[t] & RULE_TRACTORS == 0 && scoops * TRACTOR_TICKS * 2 > pull * SCOOP_TICKS * 3;
            let print = match bot {
                // Against a raid that outweighs what it has, guns before anything: something
                // small, right now.
                Bot::Ai if outgunned => Self::counter(&cards, faced, left.min(100)),
                // A collector, when its miners bite faster than its tractors pull: a miner
                // more would only wait its turn.
                Bot::Ai if short_of_tractors && cheapest(Card::collector).is_some() => cheapest(Card::collector),
                // Miners: while there's rock for them (or, for the scripted openings, one more
                // than it has factories), whatever the factories add.
                Bot::Ai | Bot::Expander | Bot::Saboteur if short_of_miners && cheapest(Card::miner).is_some() => cheapest(Card::miner),
                // Then guns: against any other raid, something small right now. While the
                // enemy's fleet has the edge, guns come before what it's saving for. Otherwise
                // only with more in the bank than the next factory needs.
                Bot::Ai if alarm => Self::counter(&cards, faced, left.min(100)),
                // A strategy that names what its king prints has it print those, in turn.
                Bot::Ai if strategy.is_some_and(|p| !p.prints.is_empty()) => {
                    let p = strategy.unwrap();
                    let next = p.prints[mind.turn % p.prints.len()];
                    if left >= saving + keep && cards.get(next).is_some_and(|c| c.ok && !c.seeded) {
                        mind.turn += 1;
                        Some(next)
                    } else {
                        None
                    }
                }
                Bot::Ai if outarmed => Self::counter(&cards, faced, (left - saving).max(0)),
                Bot::Ai if left >= saving + keep => Self::counter(&cards, faced, left - saving),
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

    /// A bot's king stays home, and keeps out of reach of any armed enemy's guns. The AI's
    /// king is also its link: when nodes cut off from it hold a good many pixels, it flies over
    /// until they're in its network, and goes home once it has spent them. And it defends:
    /// while a raid is on it stands beside its nearest factory, it keeps clear only of what
    /// outweighs it, and then by way of a factory nobody is attacking. The rushing opening's
    /// king does none of that: it flies at the enemy.
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
        // What a king keeps clear of: an armed enemy with the king in reach of the gun of its
        // that reaches furthest, and a berth besides. (Until 2026-11-17 it was any armed enemy
        // within 90 px, whatever its guns. A cannon and a tracker reach 130 and a missile 200,
        // so a king stood still while it was shelled; and the moment the enemy was 90 px off
        // it flew back to its post, which was where the enemy was. Henry, of a match he won
        // early: "the enemy king didn't seem to run away correctly".)
        let ai = self.eco().bot[t] == Bot::Ai;
        let (half, king_px) = (self.ships[k].half(), self.ships[k].px_now());
        let fear = |e: &Ship| e.st.gun_far() + e.half() + half + KING_BERTH;
        let foes: Vec<&Ship> = self.ships.iter().filter(|s| s.alive && s.team != t && s.st.armed()).collect();
        // How far a place is inside the reach of the enemy gun it's furthest inside (under 0:
        // it's clear of them all), and whose gun that is.
        let inside = |p: (i32, i32)| foes.iter().map(|e| (fear(e) - cheb(e.x, e.y, p.0, p.1), *e)).max_by_key(|&(m, e)| (m, -dist2(e.x, e.y, p.0, p.1)));
        // The AI's king stands its ground where it and the armed friends round a place outweigh
        // the armed enemies round it, each within the alarm's range (or with the place in its
        // guns' reach). Keeping clear of less only takes its guns out of the fight and leaves
        // its factory to it. Any other bot's king keeps clear of whatever has it in reach. So
        // does the AI's once it has lost a quarter of its pixels: being shot to pieces is the
        // proof that the weighing was wrong, as it is when what it outweighs by the pixel is
        // armour its side's guns can't break. (The rushing opening's king turns for home at
        // the same loss.)
        let wounded = king_px * 4 < self.ships[k].px_full() * 3;
        let outweighed = |p: (i32, i32)| {
            let mine: i32 = king_px + self.ships.iter().enumerate().filter(|&(j, s)| j != k && s.alive && s.team == t && s.st.armed() && cheb(s.x, s.y, p.0, p.1) <= ALARM_RANGE).map(|(_, s)| s.px_now()).sum::<i32>();
            let theirs: i32 = foes.iter().filter(|e| cheb(e.x, e.y, p.0, p.1) <= ALARM_RANGE.max(fear(e))).map(|e| e.px_now()).sum();
            !ai || wounded || mine < theirs
        };
        // The enemy a place is no place for a king because of, if it's one.
        let danger = |p: (i32, i32)| inside(p).filter(|&(m, _)| m > 0 && outweighed(p)).map(|(_, e)| e);
        // Whether the next stretch of its way to a place is clear of them: as a king flies, both
        // ways at once until it's level with the place and then straight, looked at every
        // 30 px for as far as it looks ahead. (Until later on 2026-11-17 it asked only whether
        // the place itself was safe, and flew to a safe post straight through the guns that
        // stood between: found by its own test, the day's change to mining having moved the
        // fight a little.)
        let clear_to = |to: (i32, i32)| {
            let (dx, dy) = (to.0 - kx, to.1 - ky);
            (1..=KING_LOOK / 30).all(|n| danger((kx + dx.signum() * (30 * n).min(dx.abs()), ky + dy.signum() * (30 * n).min(dy.abs()))).is_none())
        };
        let steer = if danger(post).is_none() && clear_to(post) {
            // Its post is safe, and so is the way there: it goes, and stays.
            if cheb(kx, ky, post.0, post.1) > 6 { ((post.0 - kx).signum(), (post.1 - ky).signum()) } else { (0, 0) }
        } else if let Some(s) = danger((kx, ky)) {
            // In reach of guns that outweigh it. The AI's king falls back on the nearest of its
            // factories that no enemy gun is near, if there's one it's nearer than that enemy
            // is: not one it would have to pass the enemy to reach.
            let refuge = if ai {
                let quiet = |f: &&Ship| !foes.iter().any(|e| cheb(e.x, e.y, f.x, f.y) <= ALARM_RANGE.max(fear(e)));
                self.ships.iter().filter(|f| f.alive && f.team == t && f.rooted).filter(quiet).filter(|f| dist2(f.x, f.y, kx, ky) < dist2(f.x, f.y, s.x, s.y)).min_by_key(|f| dist2(f.x, f.y, kx, ky)).map(|f| (f.x, f.y, f.half() + half + STRONGPOINT_GAP))
            } else {
                None
            };
            match refuge {
                Some((x, y, gap)) if cheb(kx, ky, x, y) > gap + 6 => ((x - kx).signum(), (y - ky).signum()),
                Some(_) => (0, 0),
                // With none, whichever of the eight ways leaves it safest a little way on:
                // straight away from the enemy unless another way is better, as along the
                // map's edge is for a king with its back to it.
                None => {
                    let away = ((kx - s.x).signum(), (ky - s.y).signum());
                    let (w, h) = (self.w, self.h);
                    let after = |v: (i32, i32)| ((kx + v.0 * KING_LOOK).clamp(half, w - 1 - half), (ky + v.1 * KING_LOOK).clamp(half, h - 1 - half));
                    let ways = [(0, -1), (1, -1), (1, 0), (1, 1), (0, 1), (-1, 1), (-1, 0), (-1, -1)];
                    ways.into_iter().min_by_key(|&v| (inside(after(v)).map_or(i32::MIN, |(m, _)| m), v != away)).unwrap()
                }
            }
        } else {
            // Its post is in their reach and it isn't: it waits where it is.
            (0, 0)
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
    /// Where the AI sends a seed when no rock will take it: a spot in open space a link's length
    /// from something planted in its king's network, the nearest such to where its king started
    /// (the safest end of its network), and not where a seed of its own is already headed.
    fn ai_space(&self, t: usize, slot: usize) -> Option<(i32, i32)> {
        let k = self.eco().king[t];
        let net = self.eco().net.get(k).copied().filter(|&id| id != usize::MAX)?;
        let home = KING_STARTS[t];
        let step = 2 * NODE_RANGE - 30;
        let mut best: Option<(i32, (i32, i32))> = None;
        for j in (0..self.ships.len()).filter(|&j| self.ships[j].alive && self.ships[j].rooted && self.ships[j].node && self.eco().net.get(j) == Some(&net)) {
            for (dx, dy) in [(10, 0), (-10, 0), (0, 10), (0, -10), (7, 7), (7, -7), (-7, 7), (-7, -7)] {
                let at = (self.ships[j].x + dx * step / 10, self.ships[j].y + dy * step / 10);
                let headed = self.ships.iter().any(|s| s.alive && s.team == t && s.sow && s.seed_for.is_some_and(|(_, p)| cheb(p.0, p.1, at.0, at.1) < NODE_RANGE));
                if headed || self.sow_links(t, slot, at).is_err() {
                    continue;
                }
                let d = dist2(at.0, at.1, home.0, home.1);
                if best.map_or(true, |(b, _)| d < b) {
                    best = Some((d, at));
                }
            }
        }
        best.map(|b| b.1)
    }

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
                let taken = self.ships.iter().any(|s| s.alive && s.role == Role::Ship && (s.plant() || s.grow.as_ref().is_some_and(|g| g.eats)) && cheb(s.x, s.y, r.cx, r.cy) <= rad + s.half())
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
        let mut tiny = Design::from_rows("TINY", &["hfh", "hch", "hfh"]);
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

    /// The names of the first `n` ships a factory of blue's prints, in the order it starts on
    /// them, with a full bank: an outpost of `deck`'s (slot 4) standing whole beside the king.
    /// It stops waiting after four minutes with nothing new begun.
    fn printed_in_order(deck: Vec<Design>, n: usize) -> Vec<String> {
        let mut a = Arena::for_match(1, [deck.clone(), starter_deck()], [false, false]);
        let k = a.eco.as_ref().unwrap().king[0];
        let (kx, ky) = (a.ships[k].x, a.ships[k].y);
        let f = a.spawn(0, 4, &deck[4], kx + 80, ky - 60, 0, false);
        a.ships[f].rooted = true;
        let (mut names, mut last, mut idle) = (Vec::new(), None, 0);
        while names.len() < n && idle < 15 * 240 {
            let gift = (2000 - a.bank(k)).max(0);
            a.ships[k].store += gift;
            a.eco.as_mut().unwrap().made[0] += gift;
            a.step();
            idle += 1;
            if let Making::Child(c) = a.ships[f].making {
                if last != Some(c) {
                    last = Some(c);
                    names.push(a.ships[c].name.clone());
                    idle = 0;
                }
            }
        }
        names
    }

    #[test]
    fn a_factory_prints_its_start_once_and_then_its_round_for_good() {
        // Henry, 2026-11-17: "split the builds cycle into a fixed start and then a repeat
        // sequence". Two miners to start with, then a fighter and a lancer round and round.
        let mut deck = starter_deck();
        (deck[4].first, deck[4].build) = (vec![3, 3], vec![0, 1]);
        assert_eq!(printed_in_order(deck.clone(), 8), ["MINER", "MINER", "FIGHTER", "LANCER", "FIGHTER", "LANCER", "FIGHTER", "LANCER"]);
        // With no start it's the round from the beginning, as it always was.
        deck[4].first.clear();
        assert_eq!(printed_in_order(deck.clone(), 5), ["FIGHTER", "LANCER", "FIGHTER", "LANCER", "FIGHTER"]);
        // With a start and no round, it prints the start and stops.
        (deck[4].first, deck[4].build) = (vec![3, 0, 3], Vec::new());
        assert_eq!(printed_in_order(deck.clone(), 6), ["MINER", "FIGHTER", "MINER"]);
        // What it's too small for is passed over in the start as it is in the round: the
        // capital (687 px, and the outpost builds up to 240) never comes, and nothing waits
        // for it.
        (deck[4].first, deck[4].build) = (vec![6, 3], vec![6, 0]);
        assert_eq!(printed_in_order(deck.clone(), 4), ["MINER", "FIGHTER", "FIGHTER", "FIGHTER"]);
        // A start of sixteen and a round of sixteen: every one in its place.
        (deck[4].first, deck[4].build) = (vec![3; 16], [0, 1].repeat(8));
        let names = printed_in_order(deck, 36);
        assert!(names[..16].iter().all(|n| n == "MINER") && names[16..].chunks(2).all(|c| c == ["FIGHTER", "LANCER"]), "{names:?}");
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
        Design::from_rows("SLAB", &rows.iter().map(|r| r.as_str()).collect::<Vec<_>>())
    }

    #[test]
    fn a_king_in_a_stasis_field_flies_as_fast_as_ever() {
        // The enemy's stasis post right beside blue's king, with a blue fighter beside the
        // king to be held (a field with nobody it can hold in it isn't up). The king isn't
        // held: it goes where its keys say at the pace it always does.
        let mut a = quiet();
        let k = a.eco.as_ref().unwrap().king[0];
        let (kx, ky) = (a.ships[k].x, a.ships[k].y);
        let (bar, hull) = ("x".repeat(41), "h".repeat(41));
        let mast = format!("{}x{}", ".".repeat(20), ".".repeat(20));
        let mid = format!("{}c{}", "h".repeat(20), "h".repeat(20));
        let post = a.spawn(1, 0, &Design::from_rows("WEB", &[&bar, &mast, &mast, &mast, &hull, &mid, &hull]), kx, ky + 70, 0, false);
        let fighter = a.spawn(0, 0, &Design::from_rows("HULL", &["hch", "heh"]), kx + 40, ky, 1, false);
        a.ships[fighter].order = Order::Hold;
        for _ in 0..STASIS_RISE + 3 {
            a.step();
        }
        let field = a.stasis.iter().find(|f| f.ship == post).copied().expect("the post has a field");
        assert!(field.up && a.ships[fighter].slowed && !a.ships[k].slowed, "the field: {field:?}");
        assert!(crate::pg::sim::box_gap2(a.ships[k].bbox(), field.bbox) <= field.reach * field.reach, "the king should be inside it");
        assert_eq!((a.ships[k].x, a.ships[k].y), (kx, ky));
        a.command(Cmd::Steer { team: 0, dx: 1, dy: 0 });
        for _ in 0..10 {
            a.step();
        }
        assert_eq!((a.ships[k].x, a.ships[k].y), (kx + 10 * KING_SPEED / 4, ky));
        assert!(a.stasis.iter().any(|f| f.ship == post && f.up) && a.ships[fighter].slowed && !a.ships[k].slowed);
    }

    #[test]
    fn a_king_in_a_slipstream_flies_as_fast_as_ever() {
        // A ship that does 45 px/s right over blue's king, with a slipstream that reaches 11
        // px, and a blue fighter beside it. The fighter is sped. The king isn't, slower
        // though its drawing's engines are: it goes where its keys say at the pace it
        // always does.
        let mut a = quiet();
        let k = a.eco.as_ref().unwrap().king[0];
        let (kx, ky) = (a.ships[k].x, a.ships[k].y);
        let top = a.ships[k].bbox().1;
        let quick = Design::from_rows("QUICK", &["vvvvvvvvvvv", "hhhhhhhhhhh", "hhhhhchhhhh", "eeeeeeeeeee"]);
        assert_eq!((quick.stats().vmax, quick.stats().slip_reach(), quick.stats().power_pct()), (SPEED_MOST, 11, 100));
        let q = a.spawn(0, 0, &quick, kx, top - 6, 0, false);
        let fighter = a.spawn(0, 0, &starter_deck()[0], kx + 12, top - 6, 0, false);
        (a.ships[q].order, a.ships[fighter].order) = (Order::Hold, Order::Hold);
        let own = a.ships[k].st.vmax;
        assert!(own < SPEED_MOST && a.ships[fighter].st.vmax < SPEED_MOST, "the king's drawing does {own}");
        a.step();
        let f = a.slip_fields();
        assert_eq!((f.len(), f[0].ship, f[0].pace), (1, q, SPEED_MOST));
        assert!(crate::pg::sim::box_gap2(a.ships[k].bbox(), f[0].bbox) <= f[0].reach * f[0].reach, "the king should be inside it");
        assert!(a.ships[fighter].sped && !a.ships[k].sped && a.ships[k].st.vmax == own);
        assert_eq!((a.ships[k].x, a.ships[k].y), (kx, ky));
        a.command(Cmd::Steer { team: 0, dx: 1, dy: 0 });
        for _ in 0..10 {
            a.step();
        }
        assert_eq!((a.ships[k].x, a.ships[k].y), (kx + 10 * KING_SPEED / 4, ky));
        assert!(!a.ships[k].sped && a.ships[k].st.vmax == own);
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
        while a.tick < KING_GRACE && a.over.is_none() {
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
        while a.tick < KING_GRACE && a.over.is_none() {
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
            while a.tick < KING_GRACE && a.over.is_none() {
                a.step();
            }
            let k = a.eco.as_ref().unwrap().king[0];
            let (kx, ky) = (a.ships[k].x, a.ships[k].y);
            if fleet {
                // Enough standing, and too far off for the hunter to prefer it to the king.
                a.spawn(0, 0, &slab(8, 5), kx + 600, ky - 320, 0, false);
            }
            let hunter = ship.clone();
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
        assert_eq!(slowest, SPEED_LEAST, "the arena's frigate is no longer the slowest a ship can be");
        let (near, _, hit) = chase(false, &frigate, LANCE_RANGE + 60, 40);
        assert!(near <= LANCE_RANGE && hit > 0, "a frigate never caught a beaten king: it got within {near} px and shot off {hit}");
        // Even on a diagonal, where a king covers the most ground, it is the slower of the two.
        assert!(2 * KING_SLOW * KING_SLOW < slowest * slowest);
    }

    /// A blue outpost on its home rock with a bank that never runs dry, six minutes on: the
    /// arena, the factory, and the guards it has printed, in the order they were built. (Five
    /// minutes until 2026-11-17: the miners on its build order are 12 px now, not 9, and in
    /// five it had only eight guards out.)
    fn guarded_outpost() -> (Arena, usize, Vec<usize>) {
        let mut a = quiet();
        let k = a.eco.as_ref().unwrap().king[0];
        let (_, target) = home_rock(&a);
        a.command(Cmd::Print { team: 0, slot: 4, target: Some(target) });
        for _ in 0..15 * 360 {
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

    #[test]
    fn a_factorys_rally_point_is_home_to_the_ships_it_builds_from_then_on() {
        let (mut a, f, guards) = guarded_outpost();
        let k = a.eco().king[0];
        let (fx, fy, fh) = (a.ships[f].x, a.ships[f].y, a.ships[f].half());
        assert!(guards.len() >= 5 && guards.iter().all(|&g| a.ships[g].home == (fx, fy)));
        // A minute's building, with the bank kept full.
        let build = |a: &mut Arena, secs: u32| {
            for _ in 0..15 * secs {
                let gift = (3000 - a.bank(k)).max(0);
                a.ships[k].store += gift;
                a.score[0].mined += gift;
                a.step();
            }
        };
        // Only your own factory takes one: not the enemy's say-so, not a ship, not a king, not
        // a thing that isn't there.
        for c in [Cmd::Rally { team: 1, ship: f, at: (300, 300) }, Cmd::Rally { team: 0, ship: guards[0], at: (300, 300) }, Cmd::Rally { team: 0, ship: k, at: (300, 300) }, Cmd::Rally { team: 0, ship: 99_999, at: (300, 300) }] {
            a.command(c);
        }
        a.step();
        assert!(a.ships.iter().all(|s| s.rally.is_none()));
        assert!(a.ships[f].rallies(0) && !a.ships[f].rallies(1) && !a.ships[guards[0]].rallies(0) && !a.ships[k].rallies(0));
        // A rally point 150 px from the factory.
        let at = (fx + 150, fy - 30);
        a.command(Cmd::Rally { team: 0, ship: f, at });
        a.step();
        assert_eq!(a.ships[f].rally, Some(at));
        build(&mut a, 90);
        let guard = |a: &Arena, i: usize| {
            let s = &a.ships[i];
            s.alive && s.team == 0 && s.order == Order::Guard && s.role == Role::Ship && s.grow.is_none() && s.warp == 0 && s.st.thrust > 0
        };
        let there: Vec<usize> = (0..a.ships.len()).filter(|&i| guard(&a, i) && a.ships[i].home == at).collect();
        assert!(there.len() >= 2, "a minute and a half on, {} guards have the rally point for a home", there.len());
        // The ones built before it keep the factory for a home, and are still round it.
        for &g in guards.iter().filter(|&&g| a.ships[g].alive) {
            assert!(a.ships[g].home == (fx, fy) && cheb(a.ships[g].x, a.ships[g].y, fx, fy) <= fh + POST_GAP + 3 * POST_SPACE + 12);
        }
        // The new ones wait in a ring round the rally point, apart from each other, and sit
        // still there.
        build(&mut a, 20);
        for &g in &there {
            let s = &a.ships[g];
            let d = cheb(s.x, s.y, at.0, at.1);
            assert!(s.ring0 == POST_GAP && s.posted && d >= POST_GAP / 2 && d <= POST_GAP + s.half() + 4 + crate::pg::sim::near_enough(&s.st), "guard {g} is {d} px from the rally point");
            for &o in there.iter().filter(|&&o| o != g) {
                assert!(cheb(s.x, s.y, a.ships[o].x, a.ships[o].y) >= s.half() + a.ships[o].half() + 2, "guards {g} and {o} stand on each other");
            }
        }
        assert_eq!(restless(&mut a, &there, 5), 0, "guards at the rally point kept moving");
        // A place on the factory itself is no rally point: what it builds next stays round it.
        a.command(Cmd::Rally { team: 0, ship: f, at: (fx + fh - 1, fy) });
        a.step();
        assert_eq!(a.ships[f].rally, None);
        let before = a.ships.len();
        build(&mut a, 90);
        let after: Vec<usize> = (before..a.ships.len()).filter(|&i| guard(&a, i)).collect();
        assert!(!after.is_empty() && after.iter().all(|&i| a.ships[i].home == (fx, fy)), "{:?}", after.iter().map(|&i| a.ships[i].home).collect::<Vec<_>>());
        // A place off the map is the nearest place on it.
        a.command(Cmd::Rally { team: 0, ship: f, at: (-50, 99_999) });
        a.step();
        assert_eq!(a.ships[f].rally, Some((0, a.h - 1)));
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
        assert!(guards.len() >= 10, "only {} guards after six minutes", guards.len());
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
            // (A ship that keeps its speed stops as near its post as it can place itself, not
            // on the pixel: usually within a pixel or two, at worst `near_enough`.)
            let near = crate::pg::sim::near_enough(&s.st);
            assert!(far >= fh + POST_GAP - near && far <= fh + POST_GAP + 2 * POST_SPACE + s.half() + 2 + near, "{} waits {far} px from its factory", s.name);
            // Nothing is parked on top of anything else. (Clear water between every two hulls,
            // when ships stopped on the pixel. Now two neighbours may each be a few pixels
            // off their posts, towards each other.)
            for (m, &j) in guards.iter().enumerate().skip(n + 1) {
                let gap = cheb(at[n].0, at[n].1, at[m].0, at[m].1) - s.half() - a.ships[j].half();
                assert!(gap >= 1 - near, "{} and {} wait {gap} px apart", s.name, a.ships[j].name);
            }
            // At its post it looks outward: across or along, whichever its post is further
            // out by (either, where it's about as far out both ways: the ship may stand a
            // few pixels off its post).
            let (ax, ay) = ((s.x - fx).abs(), (s.y - fy).abs());
            let (out_x, out_y) = (if s.x > fx { 1 } else { 3 }, if s.y > fy { 2 } else { 0 });
            assert!((s.facing == out_x && ax + 2 * near >= ay) || (s.facing == out_y && ay + 2 * near >= ax), "{} at ({}, {}) from its factory faces {}", s.name, s.x - fx, s.y - fy, s.facing);
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
        for _ in 0..15 * 12 {
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
        let stepped = cheb(a.ships[g].x, a.ships[g].y, before[0].0, before[0].1);
        assert!((stepped - reach).abs() <= crate::pg::sim::near_enough(&a.ships[g].st), "the guard under the block didn't step just clear of it: {stepped} px, for {reach}");
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
            assert!(cheb(s.x, s.y, kx, ky) >= kh + s.half() - crate::pg::sim::near_enough(&s.st), "{} waits under the king", s.name);
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
        let back = spots(&a, &guards);
        // (To its post: as near it as it can place itself, coming from either side.)
        assert!((1..guards.len()).all(|n| cheb(back[n].0, back[n].1, before[n].0, before[n].1) <= 2 * crate::pg::sim::near_enough(&a.ships[guards[n]].st)), "a guard didn't go back to its post when the king left: {back:?}, from {before:?}");
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
        let home = spots(&a, &guards);
        assert!((0..guards.len()).all(|n| cheb(home[n].0, home[n].1, before[n].0, before[n].1) <= 2 * crate::pg::sim::near_enough(&a.ships[guards[n]].st)), "a guard came home to a different post: {home:?}, from {before:?}");
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
                assert!(gap >= -crate::pg::sim::near_enough(&a.ships[all[x]].st), "{} and {} wait {gap} px apart", a.ships[all[x]].name, a.ships[all[y]].name);
            }
            assert!(cheb(at[x].0, at[x].1, king.0, king.1) >= a.ships[k].half() + a.ships[all[x]].half() - crate::pg::sim::near_enough(&a.ships[all[x]].st), "a guard waits under the king");
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
            took |= cheb(a.ships[late].x, a.ships[late].y, before[2].0, before[2].1) <= 3;
            if !a.ships[bait].alive {
                break;
            }
        }
        assert!(!a.ships[bait].alive, "the guards never dealt with the enemy");
        // (Until 2026-11-16 the gunless fighter moved onto the post left empty beside it while
        // the others fought. A ship with an enemy in sight now picks its own course, and has
        // 24 px round its spot to do it in for next to nothing: it doesn't turn about twice
        // to move up 8 px.)
        let _ = took;
        restless(&mut a, &all, 20);
        let back = spots(&a, &all);
        assert!((0..all.len()).all(|n| cheb(back[n].0, back[n].1, at[n].0, at[n].1) <= 2 * crate::pg::sim::near_enough(&a.ships[all[n]].st)), "a guard came back from the fight to a different place: {back:?}, from {at:?}");
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
        let mut yard = Design::from_rows("YARD", &["hhhhhhh", "hfffffh", "hfhhhfh", "hfhchfh", "hfhhhfh", "hfffffh", "hhhhhhh"]);
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
        let hulk = Design::from_rows("HULK", &["ppppp", "pfffp", "pfcfp", "pfffp", "ppppp"]);
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
        let yard = Design::from_rows("YARD", &["ppppp", "pfffp", "pfcfp", "pfffp", "ppppp"]);
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
            let mut d = Design::from_rows("YARD", &["hhhhh", "hfffh", "hfcfh", "hfffh", "hhhhh"]);
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
            let mut d = Design::from_rows(name, &["hhhhh", "hfffh", "hfcfh", "hfffh", "hhhhh"]);
            d.seeded = true;
            d.build = build;
            d
        };
        let mut deck = starter_deck();
        deck[4] = yard("YARD A", vec![0]);
        deck[5] = yard("YARD B", vec![1]);
        // (What's printed guards where it was built: nothing wanders off to fight.)
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
        // (A card's own order is an AI's to give, so it's set here as an AI's command sets it.)
        a.set_order(0, 0, Order::Escort);
        run(&mut a, 20);
        assert!(fighters(&a).iter().all(|&i| a.ships[i].order == Order::Escort && far(&a, i) < 40), "an escort isn't with the king");
        a.command(Cmd::Steer { team: 0, dx: 1, dy: 0 });
        run(&mut a, 6);
        a.command(Cmd::Steer { team: 0, dx: 0, dy: 0 });
        run(&mut a, 8);
        assert!(fighters(&a).iter().all(|&i| far(&a, i) < 40), "an escort lost the king");
        // HUNT: off they go, and the next one printed is a hunter from the start.
        a.set_order(0, 0, Order::Hunt);
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

    #[test]
    fn a_persons_side_starts_on_guard_and_its_one_stance_sends_every_ship_out_or_home() {
        // The starter deck, with a second fighter in the seventh slot.
        let mut deck = starter_deck();
        deck[6] = deck[0].clone();
        deck[6].name = "PICKET".to_string();
        // What each card's ships start on follows from its drawing, on a person's side and on
        // an AI's alike: what flies with no scoop guards, the miner (a scoop) mines, and the
        // outpost, which can't fly, holds.
        let b = Arena::for_match(5, [deck.clone(), deck.clone()], [false, true]);
        for t in 0..2 {
            assert_eq!(b.eco().orders[t][..5], [Order::Guard, Order::Guard, Order::Guard, Order::Harvest, Order::Hold]);
            assert_eq!(b.eco().orders[t][6], Order::Guard);
        }
        assert!(b.eco().cards[0][6].stanced() && b.eco().cards[0][3].miner() && !b.eco().cards[0][3].stanced() && !b.eco().cards[0][4].stanced() && !b.eco().cards[0][4].miner());
        let mut a = Arena::for_match(5, [deck.clone(), deck.clone()], [false, false]);
        assert!(!a.attacking(0) && !a.attacking(1));
        let k = a.eco().king[0];
        let (kx, ky) = (a.ships[k].x, a.ships[k].y);
        // A ship put into a match is on its side's order for its card, as one printed is: not
        // on the order its drawing alone would give it (a fighter's drawing says fight).
        let built = |a: &mut Arena, slot: usize, dx: i32, dy: i32| {
            let d = a.eco().deck[0][slot].clone();
            let i = a.spawn(0, slot, &d, kx + dx, ky + dy, 1, false);
            assert_eq!(a.ships[i].order, a.eco().orders[0][slot], "a {} put into the match isn't on its card's order", d.name);
            i
        };
        let far = |a: &Arena, i: usize| cheb(a.ships[i].x, a.ships[i].y, a.ships[i].home.0, a.ships[i].home.1);
        // Two fighters, a lancer and a miner: the warships wait where they were built.
        let two = [built(&mut a, 0, 40, 0), built(&mut a, 0, 40, 14)];
        let lancer = built(&mut a, 1, 40, -14);
        let miner = built(&mut a, 3, 60, 0);
        let warships = [two[0], two[1], lancer];
        for _ in 0..30 {
            a.step();
        }
        assert!(warships.iter().all(|&i| a.ships[i].order == Order::Guard && far(&a, i) <= 6));
        // ATTACK: every card that flies is on HUNT, the picket's too, and what's flying goes at
        // once. The miner, which has a scoop, and the outpost are as they were, and so is red.
        a.command(Cmd::Stance { team: 0, attack: true });
        a.step();
        assert!(a.attacking(0) && !a.attacking(1));
        assert_eq!(a.eco().orders[0][..5], [Order::Hunt, Order::Hunt, Order::Hunt, Order::Harvest, Order::Hold]);
        assert_eq!(a.eco().orders[0][6], Order::Hunt);
        assert_eq!(a.eco().orders[1][..3], [Order::Guard, Order::Guard, Order::Guard]);
        assert!(warships.iter().all(|&i| a.ships[i].order == Order::Hunt));
        assert_eq!(a.ships[miner].order, Order::Harvest);
        for _ in 0..15 * 10 {
            a.step();
        }
        for i in warships {
            assert!(far(&a, i) > GUARD_RADIUS, "ten seconds on, ship {i} has gone {} px", far(&a, i));
        }
        // A fighter built after that goes too, as soon as it's built. (Ships used to gather
        // into groups first: group size went with the manual flight patterns.)
        let late = built(&mut a, 0, 40, 0);
        for _ in 0..15 * 10 {
            a.step();
        }
        assert!(a.ships[late].order == Order::Hunt && far(&a, late) > GUARD_RADIUS, "ten seconds on, the late fighter has gone {} px", far(&a, late));
        // A ship that's been sent somewhere stays sent, whatever the stance.
        a.command(Cmd::Send { team: 0, ship: lancer, at: (kx + 300, ky - 200), pace: 0 });
        // GUARD: they all come home.
        a.command(Cmd::Stance { team: 0, attack: false });
        a.step();
        assert!(!a.attacking(0));
        assert_eq!(a.eco().orders[0][..5], [Order::Guard, Order::Guard, Order::Guard, Order::Harvest, Order::Hold]);
        for _ in 0..15 * 60 {
            a.step();
        }
        for i in two {
            assert!(a.ships[i].alive && far(&a, i) <= 6, "a minute on, ship {i} is {} px from home", far(&a, i));
        }
        assert!(a.ships[lancer].sent == Some((kx + 300, ky - 200)) && cheb(a.ships[lancer].x, a.ships[lancer].y, kx + 300, ky - 200) <= crate::pg::sim::near_enough(&a.ships[lancer].st));
        // The same stance twice changes nothing, and tells nobody to gather.
        let orders = a.eco().tally[0].orders;
        a.command(Cmd::Stance { team: 0, attack: false });
        a.step();
        assert_eq!(a.eco().tally[0].orders, orders);
        // A card's own order is an AI's to give. From a person's side it's dropped: the card
        // stays as the stance left it.
        a.command(Cmd::Order { team: 0, slot: 1, order: Order::Hunt });
        a.command(Cmd::Order { team: 0, slot: 0, order: Order::Sab });
        a.step();
        assert!(a.eco().orders[0][..3] == [Order::Guard; 3] && !a.attacking(0) && a.eco().tally[0].orders == orders);
        // One card out of several isn't the side attacking. (Set here as an AI sets it.)
        a.set_order(0, 1, Order::Hunt);
        assert!(a.eco().orders[0][1] == Order::Hunt && !a.attacking(0));
        // A side with nothing that flies and fights is never attacking.
        let mut c = Arena::for_match(5, [vec![deck[3].clone(), deck[4].clone()], deck.clone()], [false, false]);
        c.command(Cmd::Stance { team: 0, attack: true });
        c.step();
        assert!(!c.attacking(0));
    }

    #[test]
    fn sab_passes_a_king_by_for_a_miner_and_goes_on_the_wire_like_any_order() {
        let deck = starter_deck();
        let mut a = Arena::for_match(5, [deck.clone(), starter_deck()], [false, false]);
        let k = a.eco().king[1];
        let (kx, ky) = (a.ships[k].x, a.ships[k].y);
        // A raider on SAB (an order an AI gives) 60 px from the red king, and a red miner 200 px
        // beyond: the miner.
        let raider = a.spawn(0, 0, &deck[0], kx - 60, ky, 1, false);
        a.ships[raider].order = Order::Sab;
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
        while a.tick < 15 * 240 && a.over.is_none() {
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
    fn a_ship_sent_somewhere_keeps_getting_nearer_until_its_there() {
        // Henry, 2026-11-16, of a match on his own deck: "this fighter seemed to be somehow
        // stuck for a while". His fighter takes 1.5 s a quarter turn. Two things held such a
        // ship up. Standing with a long leg and a short one to fly, it took the short one if
        // its engine happened to point that way, overshot it, came back and overshot again,
        // with the long leg still to fly: one miner here for 41 s (the test of the trap itself
        // is `a_ship_near_its_spot_gets_onto_it_without_fussing`). And having stopped a few
        // pixels off its spot it backed off and came in again from further out.
        //
        // His deck against the starter deck, both sides played by the AI so that there are
        // ships to send, three matches of four minutes. Every second one of blue's ships
        // that can fly is sent somewhere, half the time within 40 px and half within 300,
        // whatever it's doing. No ship on its way goes 25 s without getting nearer its spot
        // (the place it was sent to, or the nearest point clear of what stands there), by
        // one leg or the other, until it has stopped as near it as it can place itself.
        // (The longest now is 17 s: a fighter sent to a spot behind it while it's flying away
        // at speed, which stops dead and comes back. It was 9 s, with a limit of 15, until
        // 2026-11-17, when a ship in that state stopped steering for the spot, which could take
        // it round and round the spot for good, and when his fighter went from 30 px/s to 26.
        // A match is too chancy to pin one rule down: the tests that fail when a trap is put
        // back are `a_ship_near_its_spot_gets_onto_it_without_fussing` and
        // `a_ship_sent_somewhere_while_its_at_speed_comes_to_rest_there`.)
        // (His warships as he had drawn them that day, in the starter deck's slots, with the
        // starter deck's miner and factories. It read his saved team from its file until
        // 2026-11-17, so what it tested changed with whatever he had drawn last: that day three
        // of his cards weren't legal any more and his miner was a third short of power.)
        let mut deck = starter_deck();
        for (slot, line) in [
            (0, "FIGHTER|48,46|h.h/igi/ghg/hch/e.e|BUILT|"),
            (2, "FRIGATE|45,44|....p..../...php.../.ppplppp./phghlhghp/phghchghp/phhhlhhhp/phghlhghp/epppppppe/..e.e.e..|BUILT|"),
            (6, "BATTLESHIP|41,41|........p......../.......plp......./......phlhp....../.....phhlhhp...../....pnnnlnnnp..../...ppnnngnnnpp.../..phhnnnhnnnhhp../.gplhhhhphhhhlpg./.phnnnhhghhnnnhp./.phnnnhhghhnnnhp./gphnnnhhchhnnnhpg/.phhhhhnnnhhhhhp./.phhhhhnnnhhhhhp./gphhhhhnnnhhhhhpg/.phhhhhhghhhhhhp./.eeeeeeeeeeeeeee.|BUILT|"),
        ] {
            deck[slot] = Design::from_line(line).unwrap();
            assert!(deck[slot].check().ok(), "{}", deck[slot].name);
        }
        deck.push(Design::from_line("PICKET|43,45|.....hhh...../.....ttt...../.....ttt...../...hhttthh.../ehghhhchhhghe/..hhhhhhhhh../..eeee.eeee..|BUILT|").unwrap());
        const LIMIT: u32 = 15 * 25;
        let (mut sends, mut arrived, mut longest) = (0, 0, 0);
        for seed in [606u64, 607, 608] {
            let mut a = Arena::for_match(seed, [deck.clone(), starter_deck()], [true, true]);
            let mut rng = Rng::new(seed);
            // For each ship that's been sent: where to, how near it has been, and when it last got nearer.
            let mut track: std::collections::HashMap<usize, ((i32, i32), i32, u32)> = Default::default();
            while a.over.is_none() && a.tick < 15 * 60 * 4 {
                if a.tick % 15 == 0 {
                    let able: Vec<usize> = (0..a.ships.len())
                        .filter(|&i| {
                            let s = &a.ships[i];
                            s.alive && s.team == 0 && s.role == Role::Ship && s.st.thrust > 0 && s.warp == 0 && s.grow.is_none() && s.maker.is_none()
                        })
                        .collect();
                    if !able.is_empty() {
                        let i = able[rng.below(able.len() as u32) as usize];
                        let (s, far) = (&a.ships[i], if rng.chance(50) { 40 } else { 300 });
                        let to = ((s.x + rng.range(-far, far)).clamp(20, a.w - 21), (s.y + rng.range(-far, far)).clamp(20, a.h - 21));
                        a.command(Cmd::Send { team: 0, ship: i, at: to, pace: 0 });
                        track.insert(i, (to, i32::MAX, a.tick));
                        sends += 1;
                    }
                }
                a.step();
                let tick = a.tick;
                track.retain(|&i, t| {
                    let s = &a.ships[i];
                    if !s.alive || s.sent != Some(t.0) || s.st.thrust == 0 {
                        return false;
                    }
                    let spot = s.goal.filter(|_| s.doing == Doing::Sent).unwrap_or(t.0);
                    // (At rest within twice what it can place itself to counts: with an enemy in
                    // sight a ship picks its own course, and one that has stopped a pixel or two
                    // outside what the plain pilot settles for may judge the hop not worth it.
                    // What this test is for is a ship that keeps flying and doesn't get there.)
                    if cheb(s.x, s.y, spot.0, spot.1) <= 2 * crate::pg::sim::near_enough(&s.st) && (s.vx, s.vy) == (0, 0) {
                        arrived += 1;
                        return false;
                    }
                    let d = (s.x - spot.0).abs() + (s.y - spot.1).abs();
                    // (A ship fighting what's round the place is holding its range, not on its way.)
                    if d < t.1 || s.doing != Doing::Sent {
                        (t.1, t.2) = (d.min(t.1), tick);
                    }
                    longest = longest.max(tick - t.2);
                    assert!(tick - t.2 <= LIMIT, "seed {seed} at {}:{:02}: {} at {:?}, going {:?}, has been {} s no nearer its spot {spot:?} than {} px", tick / 900, tick / 15 % 60, s.name, (s.x, s.y), (s.vx, s.vy), (tick - t.2) / 15, t.1);
                    true
                });
            }
        }
        println!("{sends} sends, {arrived} came to rest on their spots; the longest any went without getting nearer: {} s", longest / 15);
        assert!(arrived >= 100, "only {arrived} of {sends} sent ships came to rest on their spots");
    }

    #[test]
    fn a_seed_sent_to_any_pixel_of_a_rock_comes_down_on_it() {
        // Every pixel of blue's home rock as the place a seed is sent, on the maps of Henry's
        // two matches of 2026-11-17 and on another. Each seed comes down within a minute, the
        // factory's core stands on the very pixel it was sent to, and it has the whole rock to
        // eat.
        //
        // (A seed that stops 3 px from its pixel has stopped as near as it can place itself.
        // While coming down took being within 2 px, 4 of 135 on the first of those maps never
        // did. And a rock was counted edge to edge: a pixel of its outline hanging on by a
        // corner was a rock of its own, so the factory that landed on one ate that pixel and
        // grew the rest out of the bank. Henry's outpost, in the second match.)
        for seed in [60921, 28259, 1] {
            let base = Arena::for_match(seed, [starter_deck(), starter_deck()], [false, false]);
            let (r, _) = home_rock(&base);
            let px = base.eco.as_ref().unwrap().rocks[r].px.clone();
            let mut tried = 0;
            for &target in px.iter() {
                let mut a = base.clone();
                a.command(Cmd::Print { team: 0, slot: 4, target: Some(target) });
                while a.eco.as_ref().unwrap().tally[0].planted == 0 {
                    a.step();
                    let s = a.ships.iter().find(|s| s.alive && s.team == 0 && s.role == Role::Seed);
                    assert!(a.tick < 15 * 60, "map {seed}: the seed sent to {target:?} never came down: it's {:?}", s.map(|s| ((s.x, s.y), (s.vx, s.vy))));
                }
                let f = factory(&a).unwrap();
                assert_eq!(a.mid_from(0, 4, target), Some((a.ships[f].x, a.ships[f].y)), "map {seed}: the factory sent to {target:?} stands somewhere else");
                // The whole rock is its to eat (but for a pixel or two it has eaten already).
                let food = a.ships[f].grow.as_ref().map_or(0, |g| g.food.len());
                assert!(food + 3 >= px.len(), "map {seed}: the factory sent to {target:?} has {food} px of a {} px rock to eat", px.len());
                tried += 1;
            }
            assert!(tried >= 90, "only {tried} places tried on map {seed}");
        }
    }

    #[test]
    fn a_seed_falls_on_one_of_four_layouts_and_always_the_same_one() {
        // Henry, 2026-11-17: "like an S, a donut, or two triangles, chosen randomly", and
        // "let's keep the existing map style as a fourth option." Over four hundred seeds each
        // of the four comes up about a quarter of the time, and a seed always gives the same.
        let mut times = [0; 4];
        for seed in 1..=400u64 {
            let layout = Layout::pick(seed);
            assert_eq!(layout, Layout::pick(seed));
            times[LAYOUTS.iter().position(|&l| l == layout).unwrap()] += 1;
        }
        assert!(times.iter().all(|&n| (70..=130).contains(&n)), "{times:?}");
        println!("of 400 seeds {times:?}; seeds 1 to 8: {:?}", (1..=8).map(|s| Layout::pick(s).name()).collect::<Vec<_>>());
        // The tools' words for them.
        for (word, layout) in [("scatter", Layout::Scatter), ("s", Layout::Ess), ("ring", Layout::Ring), ("triangles", Layout::Triangles)] {
            assert_eq!(Layout::from_name(word), Some(layout));
        }
        assert_eq!(Layout::from_name("square"), None);
        // The tests' matches are on the scattered map, as every match was.
        assert_eq!(quiet().eco.as_ref().unwrap().layout, Layout::Scatter);
    }

    #[test]
    fn every_layout_is_a_fair_map_with_rock_in_reach_of_both_kings() {
        let (w, h) = (MW, MH);
        let mut amounts = Vec::new();
        for layout in LAYOUTS {
            let (mut px, mut open) = (Vec::new(), Vec::new());
            for seed in 1..=8 {
                let a = Arena::for_map(seed, layout, [starter_deck(), starter_deck()], [Bot::None; 2]);
                let e = a.eco.as_ref().unwrap();
                assert_eq!(e.layout, layout);
                let all: Vec<(i32, i32)> = e.rocks.iter().flat_map(|r| r.px.iter().copied()).collect();
                px.push(all.len());
                // The same map turned half round about its middle: neither side has the
                // better of it.
                assert!(all.iter().all(|&(x, y)| a.rock_at(w - 1 - x, h - 1 - y)), "{} on seed {seed} isn't its own mirror", layout.name());
                // Every rock is one body, and well on the map.
                for r in &e.rocks {
                    assert_eq!(a.rock_blob(r.px[0]).len(), r.px.len(), "{} on seed {seed}: a rock at {:?} in pieces", layout.name(), (r.cx, r.cy));
                    assert!(r.px.iter().all(|&(x, y)| (10..w - 10).contains(&x) && (10..h - 10).contains(&y)));
                }
                // No rock crowds a king where it starts, and there's rock within reach of
                // its first factory (a king's link and a factory's: 200 px).
                for &(kx, ky) in &KING_STARTS {
                    let near = all.iter().map(|&(x, y)| isqrt(dist2(x, y, kx, ky))).min().unwrap();
                    assert!((KING_CLEAR - 6..=KING_RANGE + NODE_RANGE - 20).contains(&near), "{} on seed {seed}: the rock nearest a king is {near} px from it", layout.name());
                }
                // The figure is where it should be. The rock nearest the map's middle:
                let middle = all.iter().map(|&(x, y)| isqrt(dist2(x, y, w / 2, h / 2))).min().unwrap();
                match layout {
                    Layout::Ess => assert!(middle <= 2 && e.rocks.len() >= 40, "the S has no rock where its curves meet"),
                    Layout::Ring => assert!(middle >= 330, "the ring's middle isn't empty: a rock {middle} px from it"),
                    Layout::Triangles => assert!(middle >= 150, "there's no open way between the triangles: a rock {middle} px from the middle"),
                    Layout::Scatter => assert!(middle <= 2, "the scattered map has lost its giant"),
                }
                // And the open space: on a figure, two fifths of the map or more is further
                // than 110 px from any rock. On a scattered map under a third of it is.
                let far = (0..w).step_by(50).flat_map(|x| (0..h).step_by(50).map(move |y| (x, y))).filter(|&(x, y)| e.rocks.iter().all(|r| dist2(r.cx, r.cy, x, y) > 110 * 110)).count();
                let spots = ((w + 49) / 50 * ((h + 49) / 50)) as usize;
                open.push(far * 100 / spots);
                if layout == Layout::Scatter {
                    assert!(far * 10 <= spots * 3, "the scattered map on seed {seed} has {far} of {spots} places in open space");
                } else {
                    assert!(far * 5 >= spots * 2, "{} on seed {seed}: only {far} of {spots} places are in open space", layout.name());
                }
            }
            amounts.push((layout.name(), *px.iter().min().unwrap(), px.iter().sum::<usize>() / px.len(), *px.iter().max().unwrap()));
            println!("{}: percent of the map in open space on each of eight maps: {open:?}", layout.name());
        }
        // About as much rock whichever way it's laid out: every layout's average within a
        // fifth of the scattered map's.
        println!("rock on eight maps of each layout (least, average, most): {amounts:?}");
        let scatter = amounts[0].2;
        for &(name, _, avg, _) in &amounts[1..] {
            assert!(avg * 5 >= scatter * 4 && avg * 5 <= scatter * 6, "{name} holds {avg} px of rock on average, and a scattered map {scatter}");
        }
    }

    #[test]
    fn the_ai_plants_and_mines_on_every_layout() {
        // Eight minutes of the AI against itself on each figure. Each side has planted
        // factories and mined about what it does on a scattered map by then.
        let mut said = Vec::new();
        for layout in LAYOUTS {
            let mut a = Arena::for_map(1, layout, [starter_deck(), starter_deck()], [Bot::Ai; 2]);
            while a.tick < 15 * 60 * 8 && a.over.is_none() {
                a.step();
            }
            let e = a.eco.as_ref().unwrap();
            said.push((layout.name(), [e.tally[0].planted, e.tally[1].planted], [a.score[0].mined, a.score[1].mined]));
            for t in 0..2 {
                assert!(e.tally[t].planted >= 2 && a.score[t].mined >= 600, "{} by 8:00: side {t} planted {} and mined {}", layout.name(), e.tally[t].planted, a.score[t].mined);
            }
        }
        println!("by 8:00, factories planted and pixels mined: {said:?}");
    }

    #[test]
    fn every_rock_is_one_body_and_any_pixel_of_it_is_the_whole_rock() {
        // On four maps: whichever pixel of a rock is asked, the rock is all of that rock's
        // pixels, the ones on its outline that hang on by a corner included. (Counted edge to
        // edge, 278 of the 22,496 rock pixels on the map of Henry's match were rocks of their
        // own, a few on every rock. Twelve more stood clear of their rock altogether: rocks
        // are made without those now.)
        for seed in [28259, 60921, 1, 2] {
            let a = Arena::for_match(seed, [starter_deck(), starter_deck()], [false, false]);
            let rocks = &a.eco.as_ref().unwrap().rocks;
            assert!(rocks.len() >= 50, "{} rocks on map {seed}", rocks.len());
            for r in rocks {
                let mut whole = r.px.clone();
                whole.sort();
                for &p in &r.px {
                    let mut rock = a.rock_blob(p);
                    rock.sort();
                    assert!(rock == whole, "map {seed}: the rock at {p:?} is {} px of the {} px rock it's on", rock.len(), whole.len());
                }
            }
        }
    }

    /// A full miner of the starter deck's for blue, put down at a point.
    fn full_miner(a: &mut Arena, x: i32, y: i32) -> usize {
        let m = a.spawn(0, 3, &starter_deck()[3], x, y, 0, false);
        a.ships[m].carry = a.ships[m].st.scoops * SCOOP_CARRY;
        m
    }

    fn far(a: &Arena, i: usize, j: usize) -> i32 {
        isqrt(dist2(a.ships[i].x, a.ships[i].y, a.ships[j].x, a.ships[j].y))
    }
    fn far_from(a: &Arena, i: usize, j: usize) -> i32 {
        far(a, i, j)
    }

    /// The pixel of a rock nearest blue's king, for a rock whose middle is between `near` and
    /// `far` px from the king: the nearest such rock.
    fn rock_face(a: &Arena, near: i32, far: i32) -> (i32, i32) {
        let (kx, ky) = KING_STARTS[0];
        let e = a.eco.as_ref().unwrap();
        let r = e.rocks.iter().filter(|r| (near..far).contains(&isqrt(dist2(r.cx, r.cy, kx, ky)))).min_by_key(|r| dist2(r.cx, r.cy, kx, ky)).expect("no rock that far from the king");
        r.px.iter().copied().min_by_key(|&(x, y)| dist2(x, y, kx, ky)).unwrap()
    }

    /// Loose pixels put down at rest in a block 16 across, inside the one square of the map
    /// (`HEAP` px across) that (x, y) is in.
    fn strew(a: &mut Arena, (x, y): (i32, i32), n: i32) -> (i32, i32) {
        let cell = (x.div_euclid(HEAP), y.div_euclid(HEAP));
        for k in 0..n {
            a.debris.push(Debris { x: cell.0 * HEAP + 8 + k % 16, y: cell.1 * HEAP + 8 + k / 16, vx: 0, vy: 0, drift: 0, taken: false });
        }
        assert!(n <= 16 * 16);
        cell
    }

    fn example(name: &str) -> Design {
        part_examples().into_iter().find(|d| d.name == name).unwrap()
    }

    #[test]
    fn on_rock_a_vacuum_lump_bites_as_one_scoop_however_big_it_is() {
        // Henry, 2026-11-18: "specialized in picking up loose debris pixels but is less
        // efficient when used on asteroids". The SWEEPER's lump of 24, alone at the face of a
        // rock with nothing loose anywhere: a bite every 10 s, as one scoop bites. A ship
        // with two lumps of four bites as two scoops.
        let base = quiet();
        let face = rock_face(&base, 90, 400);
        let twin = Design::from_rows("TWIN", &["VV.VV", "VV.VV", "hhchh", "hhhhh", "ee.ee"]);
        for (d, lumps) in [(example("SWEEPER"), 1), (twin, 2)] {
            assert!(d.check().ok() && d.stats().biters() == lumps && d.stats().scoops == 0 && d.stats().power_pct() == 100, "{}", d.name);
            let mut a = base.clone();
            let m = a.spawn(0, 3, &d, face.0, face.1, 0, false);
            let mut bites = Vec::new();
            while bites.len() < 3 {
                let before = a.ships[m].carry;
                a.step();
                if a.ships[m].carry > before {
                    bites.push(a.tick);
                }
                assert!(a.tick < 15 * 90, "{} took {} bites in a minute and a half", d.name, bites.len());
            }
            assert_eq!((bites[2] - bites[0]) as i32, 2 * SCOOP_TICKS / lumps, "{}: its bites came at {bites:?}", d.name);
        }
    }

    #[test]
    fn a_vacuum_ship_goes_to_the_nearest_heap_that_would_fill_it_and_a_tractor_has_to_take_the_load() {
        // A SWEEPER by blue's king, which has room for 72. Twenty loose pixels 160 px off and
        // two hundred 330 px off: only the far heap would fill it, so that's where it goes. It
        // sits on the heap until it's full, takes the load into the reach of the king's
        // tractor, and waits there while the tractor takes it, a pixel a second. Then back,
        // until both heaps are gone.
        let mut a = quiet();
        let k = a.eco.as_ref().unwrap().king[0];
        let (kx, ky) = (a.ships[k].x, a.ships[k].y);
        let m = a.spawn(0, 3, &example("SWEEPER"), kx + 60, ky, 0, false);
        let (small, big) = (strew(&mut a, (kx + 160, ky - 60), 20), strew(&mut a, (kx + 330, ky + 40), 200));
        let (hold, mined) = (a.ships[m].st.hold(), a.score[0].mined);
        assert_eq!((hold, a.tractor_px(k)), (72, 8));
        a.step();
        assert_eq!(a.ships[m].heap, Some(big));
        // Its first load: full within 40 s, on the far heap, with the near one untouched.
        let in_cell = |a: &Arena, c: (i32, i32)| a.debris.iter().filter(|d| (d.x.div_euclid(HEAP), d.y.div_euclid(HEAP)) == c).count();
        while a.ships[m].carry < hold {
            a.step();
            assert!(a.tick < 15 * 40, "40 s on it carries {} and is {} px from the heap", a.ships[m].carry, isqrt(dist2(a.ships[m].x, a.ships[m].y, big.0 * HEAP + 16, big.1 * HEAP + 16)));
        }
        let full_at = a.tick;
        // (It stops where the nearest of the heap comes into its reach, 28 px from its middle.)
        assert!(cheb(a.ships[m].x, a.ships[m].y, big.0 * HEAP + 16, big.1 * HEAP + 16) <= HEAP + 16 && in_cell(&a, small) == 20);
        // Nothing is banked until a tractor takes it: the load is on the ship.
        assert_eq!(a.score[0].mined, mined);
        // Home, and emptied no faster than the king's eight tractor pixels pull: a pixel a
        // second, so the last of 72 comes off 71 s after the first.
        let (mut first, mut empty) = (None, None);
        while empty.is_none() {
            let before = a.ships[m].carry;
            a.step();
            assert!(a.ships[m].carry <= hold);
            if a.ships[m].carry < before && first.is_none() {
                first = Some(a.tick);
                assert!(a.ships[m].hauling && a.ships[m].dock == Some(k));
            }
            if a.ships[m].carry == 0 {
                empty = Some(a.tick);
            }
            assert!(a.tick < full_at + 15 * 150, "150 s after it filled it still carries {}", a.ships[m].carry);
        }
        let (first, empty) = (first.unwrap(), empty.unwrap());
        println!("full at {} s, the first pixel off at {} s, empty at {} s", full_at / 15, first / 15, empty / 15);
        assert!(((hold - 2) * 15..=hold * 15).contains(&((empty - first) as i32)), "it took {} ticks to empty", empty - first);
        assert_eq!(a.score[0].mined - mined, hold);
        // And so on until both heaps are home: four loads. The last isn't a full one, and it
        // brings that home all the same when there's nothing loose left to wait for.
        while a.score[0].mined - mined < 220 {
            a.step();
            assert!(a.ships[m].carry <= hold);
            assert!(a.tick < 15 * 60 * 10, "ten minutes on, {} loose pixels are left ({} and {} in the two heaps), it carries {}, and {} are home", a.debris.len(), in_cell(&a, small), in_cell(&a, big), a.ships[m].carry, a.score[0].mined - mined);
        }
        println!("both heaps home at {}:{:02}", a.tick / 900, a.tick / 15 % 60);
        assert!(a.debris.is_empty());
    }

    #[test]
    fn a_vacuum_ship_that_fits_a_pad_unloads_on_it_at_once_and_one_that_doesnt_needs_a_tractor() {
        // A station with a pad 6 by 6 and no tractor, as the PADPOST's is. The DUSTER, 5 by
        // 5, fits it: with its hold full it lands and all thirty pixels are in the station's
        // store at once. The SWEEPER, 9 by 8, doesn't: it would on a pad 9 by 8.
        let mut a = quiet();
        let (x, y) = (700, 500);
        let p = a.spawn(0, 4, &pad_block(6, 6), x, y, 0, false);
        let big = a.spawn(0, 4, &pad_block(9, 8), x + 200, y + 150, 0, false);
        let m = a.spawn(0, 3, &example("DUSTER"), x + 80, y + 10, 0, false);
        let sweeper = a.spawn(0, 3, &example("SWEEPER"), x + 200, y + 250, 0, false);
        assert_eq!((a.pad_room(m, p) > 0, a.pad_room(sweeper, p), a.pad_room(sweeper, big) > 0), (true, 0, true));
        a.ships[sweeper].alive = false;
        a.ships[big].alive = false;
        a.ships[m].carry = a.ships[m].st.hold();
        let (load, held) = (a.ships[m].carry, a.ships[p].store);
        assert_eq!(load, 30);
        let mut down = None;
        for t in 0..15 * 30 {
            let before = a.ships[m].carry;
            a.step();
            if a.ships[m].carry < before {
                assert!(a.ships[m].carry == 0 && on_the_pad(&a, m, p), "it unloaded {} of {before}", before - a.ships[m].carry);
                down = Some(t);
                break;
            }
        }
        assert!(down.is_some(), "half a minute on it still carries {}", a.ships[m].carry);
        assert_eq!((a.ships[p].store - held, a.score[0].landed), (load, load));
    }

    #[test]
    fn a_vacuum_ship_takes_the_nearer_of_two_heaps_that_would_fill_it_and_keeps_to_a_heap_till_its_gone() {
        let cell_of = |a: &Arena, m: usize| a.ships[m].heap;
        let in_cell = |a: &Arena, c: (i32, i32)| a.debris.iter().filter(|d| (d.x.div_euclid(HEAP), d.y.div_euclid(HEAP)) == c).count();
        // Eighty loose pixels 160 px from blue's king and two hundred 330 px off. Either would
        // fill a SWEEPER's 72, so it goes for the nearer, and not for the bigger.
        let mut a = quiet();
        let k = a.eco.as_ref().unwrap().king[0];
        let (kx, ky) = (a.ships[k].x, a.ships[k].y);
        let m = a.spawn(0, 3, &example("SWEEPER"), kx + 60, ky, 0, false);
        let (near, far) = (strew(&mut a, (kx + 160, ky - 60), 80), strew(&mut a, (kx + 330, ky + 40), 200));
        a.step();
        assert_eq!((cell_of(&a, m), near != far), (Some(near), true));
        // Fifty near and forty-five far: neither would fill it, so it goes for the bigger.
        // Six pixels in, the far heap is the bigger of the two, and it stays where it is:
        // it keeps to its heap until that's gone. (Choosing afresh every tick it set off for
        // the other, and then back.) Then, with room for 22 more, it goes for the far one.
        let mut a = quiet();
        let m = a.spawn(0, 3, &example("SWEEPER"), kx + 60, ky, 0, false);
        let (near, far) = (strew(&mut a, (kx + 160, ky - 60), 50), strew(&mut a, (kx + 330, ky + 40), 45));
        a.step();
        assert_eq!(cell_of(&a, m), Some(near));
        let mut fewest = 50;
        while in_cell(&a, near) > 0 {
            a.step();
            fewest = fewest.min(in_cell(&a, near));
            assert_eq!(cell_of(&a, m), Some(near), "with {} left of its heap it chose another", in_cell(&a, near));
            assert!(a.tick < 15 * 60, "a minute on, {} are left of the near heap", in_cell(&a, near));
        }
        assert!(fewest < 45 && in_cell(&a, far) == 45);
        a.step();
        assert_eq!((cell_of(&a, m), a.ships[m].hauling), (Some(far), false));
    }

    #[test]
    fn a_vacuum_ship_gets_every_pixel_of_a_heap_however_its_spread_and_however_small_its_vacuum() {
        // Heaps spread awkwardly over one square of the map: three pixels in each of two
        // opposite corners; a line along one edge; one pixel in a corner; and a ring. A ship
        // with three vacuum pixels, the DUSTER and the SWEEPER each get all of each within a
        // minute and a half, from 70 px off. (A ship places itself to within a few pixels,
        // turned whichever way it came: reaching from the lump's own box, the small ones sat
        // with the last of a heap just out of reach. And flying only ever to the middle of
        // what's left, a small one sits between two corners with neither in reach.)
        let tug = Design::from_rows("TUG", &["VVV", "huh", "hch", "e.e"]);
        let ring: Vec<(i32, i32)> = (0..8).map(|k| (16 + dir64(k * 8).0 * 14 / 1024, 16 + dir64(k * 8).1 * 14 / 1024)).collect();
        let shapes: [(&str, Vec<(i32, i32)>); 4] = [
            ("two corners", vec![(1, 1), (2, 1), (1, 2), (30, 30), (29, 30), (30, 29)]),
            ("a line", (0..8).map(|k| (4 * k, 0)).collect()),
            ("one pixel", vec![(31, 0)]),
            ("a ring", ring),
        ];
        let base = quiet();
        let k = base.eco.as_ref().unwrap().king[0];
        let (kx, ky) = (base.ships[k].x, base.ships[k].y);
        let cell = ((kx + 150).div_euclid(HEAP), (ky + 20).div_euclid(HEAP));
        for d in [tug, example("DUSTER"), example("SWEEPER")] {
            for (name, shape) in &shapes {
                let mut a = base.clone();
                for &(x, y) in shape {
                    a.loose_at(cell.0 * HEAP + x, cell.1 * HEAP + y);
                }
                let n = shape.len() as i32;
                assert!(n <= d.stats().hold() && a.debris.len() as i32 == n);
                let m = a.spawn(0, 3, &d, cell.0 * HEAP - 60, cell.1 * HEAP + 16, 0, false);
                while !a.debris.is_empty() {
                    a.step();
                    assert!(a.tick < 15 * 90, "{} and {name}: a minute and a half on, {} of {n} lie at {:?}; it's at {:?}, making for {:?}, and reaches {}", d.name, a.debris.len(), a.debris.iter().map(|p| (p.x - cell.0 * HEAP, p.y - cell.1 * HEAP)).collect::<Vec<_>>(), (a.ships[m].x - cell.0 * HEAP, a.ships[m].y - cell.1 * HEAP), a.ships[m].spot.map(|p| (p.0 - cell.0 * HEAP, p.1 - cell.1 * HEAP)), a.ships[m].st.vac_reach(a.ships[m].half()));
                }
                assert_eq!(a.ships[m].carry + a.ships[m].store + (a.score[0].mined - base.score[0].mined), n + a.ships[m].store, "{} and {name}", d.name);
            }
        }
    }

    #[test]
    fn a_vacuum_ship_with_a_tractor_of_its_own_is_emptied_by_it() {
        // Three vacuum pixels and one tractor pixel: it holds 9, it's a node, and it's no
        // collector, so it goes for the nine loose pixels beside it. Nothing else of its
        // side's is in reach. Its own tractor takes its load, a pixel every 8 s, into its own
        // store: the tractor's rule as it stands, with nothing added for the vacuum.
        let tug = Design::from_rows("TUG", &["VVV", "huh", "hch", "e.e"]);
        assert!(tug.check().ok() && tug.stats().power_pct() == 100 && tug.stats().hold() == 9);
        let mut a = quiet();
        let k = a.eco.as_ref().unwrap().king[0];
        let (kx, ky) = (a.ships[k].x, a.ships[k].y);
        let at = (kx + 150, ky + 20);
        let cell = strew(&mut a, at, 9);
        let m = a.spawn(0, 3, &tug, cell.0 * HEAP + 12, cell.1 * HEAP + 20, 0, false);
        assert!(a.is_node(m) && a.tractor_px(m) == 1 && far(&a, m, k) > a.tractor_reach(k) + 20);
        for _ in 0..15 * 20 {
            a.step();
        }
        // Twenty seconds on it has them all, aboard or in its store, and has gone nowhere.
        assert_eq!((a.debris.len(), a.ships[m].carry + a.ships[m].store, a.ships[m].dock), (0, 9, Some(m)));
        let store = a.ships[m].store;
        for _ in 0..TRACTOR_TICKS * 4 {
            a.step();
        }
        assert_eq!((a.ships[m].store - store, a.ships[m].carry + a.ships[m].store), (4, 9));
        assert!(far(&a, m, k) > a.tractor_reach(k) + 20);
    }

    #[test]
    fn a_collector_comes_to_a_vacuum_ship_at_work_and_empties_it_where_it_sits() {
        // Two hundred loose pixels 150 px from blue's king: out of the reach of the king's
        // tractor, and in link of it. A SWEEPER goes to them, and the starter deck's
        // collector, printed beside the king, comes and stands where its tractor covers the
        // heap. The SWEEPER never goes home: as the collector takes pixels off it, a pixel a
        // second, its vacuum takes more in.
        let mut a = quiet();
        let k = a.eco.as_ref().unwrap().king[0];
        let (kx, ky) = (a.ships[k].x, a.ships[k].y);
        let cell = strew(&mut a, (kx + 150, ky + 20), 200);
        let heap = (cell.0 * HEAP + 16, cell.1 * HEAP + 14);
        let m = a.spawn(0, 3, &example("SWEEPER"), kx + 60, ky, 0, false);
        let c = a.spawn(0, 7, &starter_deck()[7], kx + 40, ky + 30, 0, false);
        assert!(isqrt(dist2(heap.0, heap.1, kx, ky)) > a.tractor_reach(k) + 40 && a.tractor_px(c) == 8);
        for _ in 0..15 * 60 {
            a.step();
        }
        let (s, v) = (&a.ships[c], &a.ships[m]);
        assert!(s.doing == Doing::Collect && (s.vx, s.vy) == (0, 0), "after a minute the collector is {} at {:?}, going {:?}", s.doing.name(), (s.x - kx, s.y - ky), (s.vx, s.vy));
        assert!(far(&a, m, c) <= a.tractor_reach(c), "the sweeper is {} px from it, and it reaches {}", far(&a, m, c), a.tractor_reach(c));
        assert!(cheb(v.x, v.y, heap.0, heap.1) <= HEAP + 16, "the sweeper is at {:?} from the heap", (v.x - heap.0, v.y - heap.1));
        let e = a.eco.as_ref().unwrap();
        assert!(e.net[c] != usize::MAX && e.net[c] == e.net[k], "the collector stands out of link with its king");
        // A minute more: the collector banks a pixel a second, the sweeper stays on the heap
        // with its hold near full, and nothing has gone by way of the king.
        let (store, mined) = (a.ships[c].store, a.score[0].mined);
        for _ in 0..15 * 60 {
            a.step();
            assert!(cheb(a.ships[m].x, a.ships[m].y, heap.0, heap.1) <= HEAP + 16 && a.ships[m].dock != Some(k));
        }
        let banked = a.ships[c].store - store;
        println!("the collector banked {banked} px in a minute; the sweeper carries {}; {} loose pixels are left", a.ships[m].carry, a.debris.len());
        assert!((55..=61).contains(&banked) && a.score[0].mined - mined == banked, "it banked {banked} px in a minute");
        assert!(a.ships[m].carry >= 60);
    }

    /// A match with blue on the SCAVENGERS and red on the starter deck, and nobody playing.
    fn scavenging() -> (Arena, Vec<Design>) {
        let team = crate::pg::teams::scavengers();
        (Arena::for_match(1, [team.clone(), starter_deck()], [false, false]), team)
    }

    #[test]
    fn the_scavengers_miners_and_vacuum_ships_land_on_their_port_and_on_their_barge() {
        // The PORT's pad is 6 by 9: it has a place for a GLEANER (6 by 6), and two for
        // DIGGERs (5 by 4) at once. The BARGE's is 6 by 6: a place for either. A full DIGGER
        // and a full GLEANER by the PORT both land and unload everything at once: the PORT
        // has no tractor.
        let (mut a, team) = scavenging();
        let (x, y) = (700, 500);
        let port = a.spawn(0, 7, &team[7], x, y, 0, false);
        let barge = a.spawn(0, 8, &team[8], x, y + 160, 0, false);
        let digger = a.spawn(0, 5, &team[5], x + 70, y + 30, 0, false);
        let gleaner = a.spawn(0, 6, &team[6], x + 70, y - 30, 0, false);
        let rooms = (a.pad_room(digger, port), a.pad_room(gleaner, port), a.pad_room(digger, barge), a.pad_room(gleaner, barge));
        println!("places: a digger on the port {}, a gleaner on the port {}, a digger on the barge {}, a gleaner on the barge {}", rooms.0, rooms.1, rooms.2, rooms.3);
        assert!(rooms.0 >= 2 && rooms.1 >= 1 && rooms.2 >= 1 && rooms.3 >= 1, "{rooms:?}");
        assert_eq!((a.tractor_px(port), a.is_node(port), a.is_node(barge), a.ships[barge].st.collector()), (0, true, true, true));
        a.ships[barge].alive = false;
        (a.ships[digger].carry, a.ships[gleaner].carry) = (a.ships[digger].st.hold(), a.ships[gleaner].st.hold());
        let (held, load) = (a.ships[port].store, a.ships[digger].carry + a.ships[gleaner].carry);
        assert_eq!(load, 12 + 36);
        // (Each is away again as soon as it's empty, the DIGGER to the rock: so it's when each
        // was first empty that counts.)
        let (mut t, mut down) = (0, [None; 2]);
        while down.iter().any(|d| d.is_none()) {
            a.step();
            t += 1;
            for (k, &m) in [digger, gleaner].iter().enumerate() {
                if down[k].is_none() && a.ships[m].carry == 0 {
                    down[k] = Some(t);
                }
            }
            assert!(t < 15 * 60, "a minute on the digger carries {} and the gleaner {}: {down:?}", a.ships[digger].carry, a.ships[gleaner].carry);
        }
        println!("unloaded on the port's pad: the digger after {} s, the gleaner after {} s", down[0].unwrap() / 15 + 1, down[1].unwrap() / 15 + 1);
        // (The PORT spends from its store as it prints, so it's what landed that's counted.)
        let _ = held;
        assert!(a.score[0].landed >= load && a.score[0].mined >= load);
        // With the PORT gone and the BARGE there, a full GLEANER lands on the BARGE, which is
        // a node: the load is in its store.
        a.ships[port].alive = false;
        a.ships[barge].alive = true;
        a.ships[gleaner].carry = a.ships[gleaner].st.hold();
        let held = a.ships[barge].store;
        let mut t = 0;
        while a.ships[gleaner].carry > 0 {
            a.step();
            t += 1;
            assert!(t < 15 * 90, "a minute and a half on the gleaner still carries {}; it's {} px from the barge", a.ships[gleaner].carry, far(&a, gleaner, barge));
        }
        println!("unloaded on the barge within {} s", t / 15 + 1);
        // (All 36 of the GLEANER's, and whatever the DIGGER has brought it meanwhile.)
        assert!(a.ships[barge].store - held >= 36);
    }

    #[test]
    fn a_miner_with_no_guns_that_is_hit_runs_home_and_stops_there() {
        // A DIGGER 120 px from its PORT, hit once a second for 20 s. It runs to the PORT and
        // stops beside it: it's never further from the PORT than it started, and from 10 s on
        // it's beside it and all but still. Left alone, it goes back to work. (It used to fly
        // through the PORT flat out and on: 234 px out the far side at 20 s.)
        let (mut a, team) = scavenging();
        let k = a.eco.as_ref().unwrap().king[0];
        let (kx, ky) = (a.ships[k].x, a.ships[k].y);
        let port = a.spawn(0, 7, &team[7], kx + 260, ky - 120, 0, false);
        a.step();
        for d in [&team[5], &team[6], &team[8]] {
            let m = a.spawn(0, 5, d, kx + 260 + 120, ky - 120, 0, false);
            assert!(!a.ships[m].st.armed());
            let beside = a.ships[port].half() + 4 + a.ships[m].half() + 14;
            let mut worst = 0;
            for t in 0..15 * 20 {
                if t % 15 == 0 {
                    a.ships[m].last_hit = a.tick;
                }
                a.step();
                let s = &a.ships[m];
                let off = far(&a, m, port);
                assert!(s.doing == Doing::Flee && off <= 130, "{} at {} s: {}, {off} px from the port", d.name, t / 15, s.doing.name());
                if t >= 15 * 10 {
                    worst = worst.max(off);
                    assert!(off <= beside && s.speed_px_s() <= 8, "{} at {} s: {off} px from the port, going {} px/s", d.name, t / 15, s.speed_px_s());
                }
            }
            for _ in 0..SCARED + 2 {
                a.step();
            }
            assert!(a.ships[m].doing != Doing::Flee, "{} is still {}", d.name, a.ships[m].doing.name());
            println!("{}: beside the port from 10 s on, never more than {worst} px from its middle", d.name);
            a.ships[m].alive = false;
        }
    }

    /// Takes every rock pixel off the map and puts these there: for the tests of what a miner
    /// goes for.
    fn rocks_only(a: &mut Arena, px: &[(i32, i32)]) {
        a.rock.iter_mut().for_each(|r| *r = false);
        a.rock_px.clear();
        for &(x, y) in px {
            a.rock[(y * a.w + x) as usize] = true;
            a.rock_px.push((x, y));
        }
    }

    /// A miner put down at `at`, on a map with no rock but `rock` and with loose pixels lying
    /// at `loose`: the loose pixels it has taken, in the order it took them, when the first
    /// rock pixel goes (or, if none does, two minutes on); and the map at that moment.
    fn takes_before_rock(d: &Design, at: (i32, i32), rock: &[(i32, i32)], loose: &[(i32, i32)]) -> (Vec<(i32, i32)>, Arena, usize) {
        let mut a = quiet();
        rocks_only(&mut a, rock);
        for &(x, y) in loose {
            a.loose_at(x, y);
        }
        let m = a.spawn(0, 3, d, at.0, at.1, 0, false);
        let mut taken = Vec::new();
        while a.tick < 15 * 120 && rock.iter().all(|&(x, y)| a.rock_at(x, y)) {
            a.step();
            for &p in loose {
                if !taken.contains(&p) && !a.debris.iter().any(|d| (d.x, d.y) == p) {
                    taken.push(p);
                }
            }
        }
        (taken, a, m)
    }

    #[test]
    fn a_miner_goes_for_loose_pixels_only_with_no_rock_near_it() {
        // Henry, 2026-11-19: "They should prioritize rock and only mine loose pixels when
        // there is no rock anywhere near (within approx 1/3rd or 1/4th the map?)". Near is a
        // quarter of the map's width, from where the miner is: 350 px in a match, which is a
        // third of the map's height, and 120 px in the proving ground.
        assert_eq!((quiet().rock_close(), Arena::new().rock_close()), (350, 120));
        let (near, at) = (350, (500, 500));
        for d in [starter_deck()[3].clone(), crate::pg::teams::scavengers()[5].clone()] {
            // One rock pixel just near enough, 350 px east, and a loose pixel 100 px west:
            // the rock, and the loose pixel lies.
            let (rock, loose) = ((at.0 + near, at.1), (at.0 - 100, at.1));
            let (taken, a, _) = takes_before_rock(&d, at, &[rock], &[loose]);
            assert!(!a.rock_at(rock.0, rock.1) && taken.is_empty(), "{}, rock {near} px off: it took {taken:?} first, {} s", d.name, a.tick / 15);
            // The rock a pixel further: the loose pixel first, and then the rock.
            let rock = (at.0 + near + 1, at.1);
            let (taken, a, _) = takes_before_rock(&d, at, &[rock], &[loose]);
            assert!(!a.rock_at(rock.0, rock.1) && taken == [loose], "{}, rock {} px off: it took {taken:?} first, {} s", d.name, near + 1, a.tick / 15);
            // Near is as the crow flies. Rock 247 px east and 247 north is 349 px off: the
            // rock. 248 and 248 is 351 px off: the loose pixel first.
            for (side, first) in [(247, vec![]), (248, vec![(at.0 - 70, at.1 + 70)])] {
                let rock = (at.0 + side, at.1 - side);
                let (taken, a, _) = takes_before_rock(&d, at, &[rock], &[(at.0 - 70, at.1 + 70)]);
                assert!(!a.rock_at(rock.0, rock.1) && taken == first, "{}, rock {side} px east and north: it took {taken:?} first, {} s", d.name, a.tick / 15);
            }
            // No rock near, and the loose pixel further off than the rock is (rock 400 px
            // east, the loose pixel 450 px west): the rock. It goes no further for a loose
            // pixel than it would for rock.
            let (rock, loose) = ((at.0 + 400, at.1), (at.0 - 450, at.1));
            let (taken, a, _) = takes_before_rock(&d, at, &[rock], &[loose]);
            assert!(!a.rock_at(rock.0, rock.1) && taken.is_empty(), "{}, rock 400 px off and a loose pixel 450: it took {taken:?} first, {} s", d.name, a.tick / 15);
            // No rock on the map: it takes loose pixels wherever they lie, the nearest first.
            let loose = [(at.0 + 200, at.1 - 300), (at.0, at.1 - 300)];
            let (taken, _, _) = takes_before_rock(&d, at, &[], &loose);
            assert!(taken == [loose[1], loose[0]], "{}, no rock on the map: it took {taken:?}", d.name);
            // The last rock pixel on the map bitten, and a loose pixel 300 px off: it goes
            // for that, with no place of work for a collector to come to. With nothing left
            // of either, it takes its two pixels home.
            let (rock, loose) = ((at.0 + 5, at.1), (at.0, at.1 - 300));
            let mut a = quiet();
            rocks_only(&mut a, &[rock]);
            a.loose_at(loose.0, loose.1);
            let m = a.spawn(0, 3, &d, at.0, at.1, 0, false);
            while a.rock_at(rock.0, rock.1) {
                a.step();
                assert!(a.tick < 15 * 30, "{} hasn't bitten the pixel beside it in 30 s", d.name);
            }
            assert_eq!((a.ships[m].worked, a.ships[m].carry), (Some(rock), 1), "{}", d.name);
            a.step();
            assert_eq!((a.ships[m].worked, a.ships[m].goal, a.ships[m].doing), (None, Some(loose), Doing::Mine), "{}", d.name);
            while a.score[0].mined < 2 {
                a.step();
                assert!(a.tick < 15 * 150, "{}: {} loose, carrying {}, {}, mined {}", d.name, a.debris.len(), a.ships[m].carry, a.ships[m].doing.name(), a.score[0].mined);
            }
        }
    }

    #[test]
    fn a_miner_after_loose_pixels_turns_to_rock_as_soon_as_some_is_near() {
        // Rock A 400 px east of a miner and rock B 500 px west: neither near, and A the
        // nearer. Three loose pixels: 100 px west, 200 px west, and 150 px south of that one.
        // It goes west for the first. From 450 px east of B on, B is near: it flies on to B,
        // taking the second as it passes, and the third lies. (It looks for rock again once a
        // second while the rock it chose is far off. Keeping to A, it would never have had
        // rock near, and would have gone south for the third.)
        let at = (600, 500);
        let (rock_a, rock_b) = ((at.0 + 400, at.1), (at.0 - 500, at.1));
        let loose = [(at.0 - 100, at.1), (at.0 - 200, at.1), (at.0 - 200, at.1 + 150)];
        for d in [starter_deck()[3].clone(), crate::pg::teams::scavengers()[5].clone()] {
            let (taken, a, m) = takes_before_rock(&d, at, &[rock_a, rock_b], &loose);
            assert!(a.rock_at(rock_a.0, rock_a.1) && !a.rock_at(rock_b.0, rock_b.1), "{}: which rock it bit, {} s", d.name, a.tick / 15);
            assert!(taken == [loose[0], loose[1]] && a.ships[m].carry == 3, "{}: it took {taken:?} and carries {}", d.name, a.ships[m].carry);
        }
    }

    /// Blue's home rock in a quiet match with three starter MINERs and two DIGGERs at its
    /// face, and `strewn` loose pixels lying between them and the king, none on rock.
    fn miners_among_wreckage(seed: u64, strewn: i32) -> (Arena, Vec<usize>) {
        let mut a = Arena::for_match(seed, [starter_deck(), starter_deck()], [false, false]);
        let (kx, ky) = KING_STARTS[0];
        let e = a.eco.as_ref().unwrap();
        let r = e.rocks.iter().min_by_key(|r| dist2(r.cx, r.cy, kx, ky)).unwrap();
        let face = r.px.iter().copied().min_by_key(|&(x, y)| dist2(x, y, kx, ky)).unwrap();
        let digger = crate::pg::teams::scavengers()[5].clone();
        let mut miners = Vec::new();
        for k in 0..5 {
            let d = if k < 3 { starter_deck()[3].clone() } else { digger.clone() };
            miners.push(a.spawn(0, 3, &d, face.0 + (kx - face.0).signum() * 14 * (k + 1), face.1 + (ky - face.1).signum() * 14 * (k + 1), 0, false));
        }
        let mut k = 0;
        let mut put = 0;
        while put < strewn {
            let (x, y) = (face.0 + (k * 37) % 241 - 120, face.1 + (k * 53) % 239 - 119);
            k += 1;
            if (0..a.w).contains(&x) && (0..a.h).contains(&y) && !a.rock_at(x, y) && !a.debris.iter().any(|d| (d.x, d.y) == (x, y)) {
                a.loose_at(x, y);
                put += 1;
            }
        }
        (a, miners)
    }

    #[test]
    fn no_miner_heads_for_a_loose_pixel_with_rock_near_it_on_any_map() {
        // Henry, of his first match with the SCAVENGERS (2026-11-19): "It almost seems like
        // diggers are prioritizing debris over regular rock". They were: a loose pixel within
        // 30 px came before any rock, and each one taken put another within 30 px.
        //
        // On four maps, three MINERs and two DIGGERs at blue's home rock with 200 loose pixels
        // strewn within 120 px of its face, for three minutes. Every tick, every one of them
        // that's mining is making for a pixel that was rock when the tick began, or staying
        // where it is: never for a loose pixel. They bite as much rock as with no wreckage
        // there, less what their scoops took in passing (which is a load they didn't have to
        // bite).
        for seed in [1, 2, 28259, 48925] {
            let mut bitten = [0; 2];
            let mut scooped = 0;
            for (run, strewn) in [0, 200].into_iter().enumerate() {
                let (mut a, miners) = miners_among_wreckage(seed, strewn);
                let rock0 = a.rock_px.iter().filter(|&&(x, y)| a.rock_at(x, y)).count();
                while a.tick < 15 * 180 {
                    let (rock, at): (Vec<bool>, Vec<(i32, i32)>) = (a.rock.clone(), miners.iter().map(|&m| (a.ships[m].x, a.ships[m].y)).collect());
                    a.step();
                    for (k, &m) in miners.iter().enumerate() {
                        let s = &a.ships[m];
                        let Some(g) = s.goal.filter(|_| s.doing == Doing::Mine) else { continue };
                        assert!(g == at[k] || rock[(g.1 * a.w + g.0) as usize], "map {seed}, {} s: miner {k} at {:?} is making for {g:?}, which is no rock", a.tick / 15, at[k]);
                    }
                }
                bitten[run] = rock0 - a.rock_px.iter().filter(|&&(x, y)| a.rock_at(x, y)).count();
                if strewn > 0 {
                    scooped = strewn as usize - a.debris.len();
                }
            }
            println!("map {seed}: {} px of rock bitten in 3:00 with no wreckage, {} with 200 loose pixels about, of which their scoops took {scooped}", bitten[0], bitten[1]);
            assert!(bitten[1] + scooped >= bitten[0] * 9 / 10 && scooped < 200, "map {seed}: {bitten:?}, scooped {scooped}");
        }
    }

    /// The SCAVENGERS' cards as Henry had them kept on 2026-11-19, from before they were redrawn
    /// to turn. The PULSAR, the HERALD, the DIGGER, the GLEANER and the BARGE each take three
    /// seconds a quarter turn, the slowest there is, and the DIGGER flies at 26 px/s.
    fn slow_scavengers() -> Vec<Design> {
        [
            "DART|48,47|grg/hrh/hch/e.e|BUILT|",
            "PULSAR|46,46|..zzz../..zzz../ghhhhhg/hhhchhh/hhhhhhh/hh...hh/eee.eee|BUILT|",
            "MAUL|44,44|..ppppppp../.ppphhhppp./pnnntttnnnp/pnnntttnnnp/pnnntttnnnp/phhhhchhhhp/phhhhhhhhhp/phhhhhhhhhp/pghhhhhhhgp/ppppppppppp/y.e.e.e.e.y|BUILT|",
            "SNARE|42,42|x.............x/x.hhhhhhhhhhh.x/x.hhhhttthhhh.x/x.ghhhttthhhg.x/x.hhhhttthhhh.x/x.hhhhhhhhhhh.x/x.hhhhhhhhhhh.x/xxxxxxxxxxxxxxx/x.hhhhhhhhhhh.x/x.hhhhhchhhhh.x/x.ghhhhhhhhhg.x/x.hhhhhhhhhhh.x/x.hhhhhhhhhhh.x/x.yeeheheheey.x/x.............x|BUILT|",
            "HERALD|40,46|.......vvvvv......./...vvvvvvvvvvvvv.../vvvvvvhhhhhhhvvvvvv/hhhhhhhhhchhhhhhhhh/hhhhhhhhhhhhhhhhhhh/yeeeeeeee.eeeeeeeey|BUILT|",
            "DIGGER|47,47|ss.ss/hhchh/h.h.h/e.e.e|BUILT|",
            "GLEANER|47,46|VVVVVV/VVVVVV/VhhhhV/hhchhh/hhhhhh/eee.ee|BUILT|",
            "PORT|45,45|pphhghhppkkkkkk/pphhhhhppkkkkkk/hfffffffhkkkkkk/hfhhhhhfhkkkkkk/gfhhchhfgkkkkkk/hfhhhhhfhkkkkkk/hfffffffhkkkkkk/pphhhhhppkkkkkk/pphhghhppkkkkkk|SEEDED|5,5,5,5,6>0,0,1,5",
            "BARGE|47,45|kkkkkk/kkkkkk/kkkkkk/kkkkkk/kkkkkk/kkkkkk/..hch./eeeeee|BUILT|",
        ]
        .iter()
        .map(|l| Design::from_line(l).unwrap_or_else(|| panic!("{l}")))
        .collect()
    }

    /// A match with nobody playing blue and the SWARM's AI playing red, which raids at about
    /// 3:00. Blue has a PORT by its home rock (card 7 of `deck`) and these cards' ships about
    /// it, each put down so many px from the rock's face towards the king and so many across.
    /// The ships, in the order given.
    fn raided(deck: &[Design], seed: u64, layout: Layout, ships: &[(usize, i32, i32)]) -> (Arena, Vec<usize>) {
        let teams = crate::pg::teams::ready_teams();
        let swarm = teams.iter().find(|t| t.name == "SWARM").unwrap();
        let mut a = Arena::for_map(seed, layout, [deck.to_vec(), swarm.designs.clone()], [Bot::None, Bot::Ai]);
        a.eco.as_mut().unwrap().strategy = [None, swarm.strategy];
        let (kx, ky) = KING_STARTS[0];
        let face = {
            let e = a.eco.as_ref().unwrap();
            let r = e.rocks.iter().min_by_key(|r| dist2(r.cx, r.cy, kx, ky)).unwrap();
            r.px.iter().copied().min_by_key(|&(x, y)| dist2(x, y, kx, ky)).unwrap()
        };
        let (ux, uy) = toward(face.0, face.1, kx, ky, 64);
        let port = a.spawn(0, 7, &deck[7], face.0 + ux * 50 / 64, face.1 + uy * 50 / 64, 0, false);
        a.ships[port].node = true;
        let put = ships.iter().map(|&(card, back, across)| a.spawn(0, card, &deck[card], face.0 + ux * back / 64 - uy * across / 64, face.1 + uy * back / 64 + ux * across / 64, 0, false)).collect();
        (a, put)
    }

    /// Whether an enemy ship is near enough ship `i` for it to count as in a fight.
    fn enemy_in_sight(a: &Arena, i: usize) -> bool {
        let s = &a.ships[i];
        a.ships.iter().any(|o| o.alive && o.team != s.team && o.warp == 0 && cheb(o.x, o.y, s.x, s.y) <= 240)
    }

    #[test]
    fn no_miner_runs_off_when_the_enemy_comes_however_slowly_it_turns() {
        // Henry, 2026-11-19, of DIGGERs that took three seconds a quarter turn: "my miners are
        // running off the map when enemies came". A ship in a fight picked its own course,
        // weighed over four seconds. One that takes longer than that to turn about and stop
        // saw nothing gained by beginning to, so having once taken up a speed to get clear of
        // a gun it kept it. (On his map a GLEANER taking its load in passed its PORT at full
        // speed and stopped against the map's edge 184 px on; on another a DIGGER was 684 px
        // from home.) A ship with no guns flies as its plan says now.
        //
        // On five maps, his the first: his cards' four DIGGERs and GLEANER at work by a PORT,
        // two DARTs and a PULSAR by them, and the SWARM raiding, for six minutes. No ship of
        // blue's with no guns and an engine left ever touches the edge of the map, and none
        // flies away from the place it's making for for longer than it takes it to turn about:
        // three quarter turns, and for each of across and along a burn to stop and a burn to
        // get going the other way, at the most. (A ship with one engine turns one way round at
        // a time. Each pixel shot off it makes it another ship, with its own times.) Under the
        // old rule one touched the edge on every one of these maps, and they flew away from
        // where they were going for up to 28 s.
        let deck = slow_scavengers();
        let ships = [(5, 14, -24), (5, 14, -12), (5, 14, 0), (5, 14, 12), (6, 30, 30), (0, 80, 0), (0, 80, 14), (1, 80, 28)];
        for (seed, layout) in [(4258, Layout::Triangles), (1, Layout::Scatter), (2, Layout::Ess), (48925, Layout::Ess), (7, Layout::Ring)] {
            let (mut a, put) = raided(&deck, seed, layout, &ships);
            let miners = &put[..5];
            let (mut far, mut longest, mut run, mut run_goal, mut px, mut raid) = (0, 0u32, vec![0u32; 5], vec![(0, 0); 5], vec![0; 5], 0);
            while a.over.is_none() && a.tick < 15 * 60 * 6 {
                a.step();
                for (k, &m) in miners.iter().enumerate() {
                    let s = &a.ships[m];
                    if !s.alive || s.st.thrust == 0 {
                        run[k] = 0;
                        continue;
                    }
                    assert!(!s.st.armed() && s.course.is_none());
                    raid += enemy_in_sight(&a, m) as u32;
                    let h = s.half();
                    assert!(s.x > h && s.y > h && s.x < a.w - 1 - h && s.y < a.h - 1 - h, "map {seed}, {}: {} is against the edge of the map at {},{}, {}", crate::pg::play::clock(a.tick), s.name, s.x, s.y, s.doing.name());
                    far = far.max(a.ships.iter().filter(|o| o.alive && o.team == 0 && o.node).map(|o| isqrt(dist2(o.x, o.y, s.x, s.y))).min().unwrap_or(0));
                    // Flying away from where it's going: at a pixel every two ticks or more,
                    // and the place it's making for still about where it was when it began to.
                    let away = s.goal.is_some_and(|g| (g.0 - s.x) * s.vx + (g.1 - s.y) * s.vy < 0 && s.vx.abs().max(s.vy.abs()) >= 2 && (run[k] == 0 || cheb(g.0, g.1, run_goal[k].0, run_goal[k].1) <= 40));
                    let whole = std::mem::replace(&mut px[k], s.px_now()) == s.px_now();
                    if !away || !whole {
                        run[k] = 0;
                        continue;
                    }
                    if run[k] == 0 {
                        run_goal[k] = s.goal.unwrap();
                    }
                    run[k] += 1;
                    longest = longest.max(run[k]);
                    let burn = (s.st.vmax * PUSH_PER_WEIGHT * s.st.mass.max(1)) / s.st.push.iter().max().unwrap().max(&1) + 1;
                    let most = (3 * s.st.quarter + 4 * burn + 15) as u32;
                    assert!(run[k] <= most, "map {seed}, {}: {} at {},{} has been flying away from {:?} for {} ticks ({}); it turns about in {most} (a quarter turn in {}, top speed {}, push {:?}, weight {}, going {},{}, facing {})", crate::pg::play::clock(a.tick), s.name, s.x, s.y, s.goal, run[k], s.doing.name(), s.st.quarter, s.st.vmax, s.st.push, s.st.mass, s.vx, s.vy, s.facing);
                }
            }
            println!("map {seed}: with an engine left, never more than {far} px from a node; flying away from where it was going for {longest} ticks at the longest; {raid} ship-ticks with an enemy in sight");
            // (The raid came: the test would say nothing if it hadn't.)
            assert!(raid >= 15 * 60, "map {seed}: only {raid} ship-ticks with an enemy in sight");
        }
    }

    #[test]
    fn a_ship_with_no_guns_flies_as_its_plan_says_whatever_is_in_sight() {
        // The rule itself. A side set to fly by the plain pilot alone has no ship that picks a
        // course of its own (`Arena::plain`: only tools and tests set it). So for a side with
        // no armed ship that flies, a match played that way and the same match played as the
        // game plays it are the same match, tick for tick, by the number two machines compare
        // (`checksum`): however near the enemy comes, and whether a ship is mining, taking
        // its load in, running home, following its side's miners, with its group, or sent.
        //
        // Blue's cards here have no gun on anything that flies: both HERALDs, both BARGEs,
        // both DIGGERs, both GLEANERs and the starter deck's COLLECTOR, some that turn in a
        // third of a second and some that take three. One DIGGER has been sent to the rock.
        let (slow, quick, starter) = (slow_scavengers(), crate::pg::teams::scavengers(), starter_deck());
        let deck = vec![slow[4].clone(), slow[8].clone(), starter[7].clone(), quick[5].clone(), quick[6].clone(), slow[5].clone(), slow[6].clone(), slow[7].clone(), quick[4].clone()];
        assert!(deck.iter().all(|d| !d.stats().armed() || d.stats().thrust == 0));
        let ships = [(5, 14, -24), (3, 14, -12), (5, 14, 0), (3, 14, 12), (6, 30, 30), (4, 30, -30), (0, 70, 0), (8, 70, 20), (1, 70, -20), (2, 40, 0), (5, 40, 40)];
        for (seed, layout) in [(4258, Layout::Triangles), (48925, Layout::Ess)] {
            let (mut a, put) = raided(&deck, seed, layout, &ships);
            let sent = put[10];
            let at = (a.ships[put[0]].x, a.ships[put[0]].y);
            a.ships[sent].sent = Some(at);
            let mut b = a.clone();
            b.plain[0] = true;
            // Ship-ticks with an enemy in sight: of a miner, of a ship that isn't one, of a
            // ship that turns a quarter in a second or less, and of the one that was sent.
            let mut seen = [0u32; 4];
            while a.over.is_none() && a.tick < 15 * 60 * 5 {
                a.step();
                b.step();
                if a.checksum() != b.checksum() {
                    let s = (0..a.ships.len()).find(|&i| (a.ships[i].x, a.ships[i].y, a.ships[i].vx, a.ships[i].vy, a.ships[i].facing) != (b.ships[i].x, b.ships[i].y, b.ships[i].vx, b.ships[i].vy, b.ships[i].facing)).map(|i| &a.ships[i]);
                    panic!("map {seed}: the two matches part at {} ({:?})", crate::pg::play::clock(a.tick), s.map(|s| (s.name.clone(), s.team, s.x, s.y, s.doing.name())));
                }
                for i in 0..a.ships.len() {
                    let s = &a.ships[i];
                    if !s.alive || s.team != 0 || s.role != Role::Ship || s.st.thrust == 0 {
                        continue;
                    }
                    assert!(!s.st.armed() && s.course.is_none(), "{} has a gun or a course of its own", s.name);
                    if enemy_in_sight(&a, i) {
                        seen[0] += s.st.gathers() as u32;
                        seen[1] += !s.st.gathers() as u32;
                        seen[2] += (s.st.quarter <= 15) as u32;
                        seen[3] += (i == sent && s.sent.is_some()) as u32;
                    }
                }
            }
            println!("map {seed}: the same match on the plain pilot to {}; ship-ticks with an enemy in sight: miners {}, others {}, quick to turn {}, sent {}", crate::pg::play::clock(a.tick), seen[0], seen[1], seen[2], seen[3]);
            assert!(seen.iter().all(|&n| n >= 15 * 5), "map {seed}: {seen:?}");
        }
    }

    #[test]
    fn a_ship_whose_last_gun_is_shot_off_drops_the_course_it_was_on() {
        // Two FIGHTERs 70 px apart, one of each side: each picks a course of its own. With
        // its four guns shot off, blue's has none from the tick after its parts are counted
        // again, though the enemy is still there, and it never has one after.
        let mut a = quiet();
        let d = starter_deck()[0].clone();
        let (f, e) = (a.spawn(0, 0, &d, 700, 500, 0, false), a.spawn(1, 0, &d, 700, 430, 2, false));
        let mut t = 0;
        while a.ships[f].course.is_none() {
            a.step();
            t += 1;
            assert!(t < 15 * 20 && a.ships[f].alive && a.ships[e].alive, "after {t} ticks blue's fighter has no course of its own");
        }
        // (Red's guns go too, so that blue's lives to be looked at.)
        for i in [f, e] {
            for c in 0..a.ships[i].hp.len() {
                if a.ships[i].shape.cells[c] == Some(Part::Flak) {
                    a.ships[i].hp[c] = 0;
                }
            }
            a.ships[i].dirty = true;
        }
        a.step();
        assert!(!a.ships[f].st.armed() && a.ships[f].alive && a.ships[e].alive);
        for _ in 0..15 * 5 {
            a.step();
            assert!(a.ships[f].course.is_none() && a.ships[e].course.is_none(), "a ship with no guns is on a course of its own");
        }
    }

    #[test]
    fn a_stasis_lattice_that_has_been_sent_steps_aside_for_nothing() {
        // A SNARE has guns and a lattice, and its field is up only while it's still. Sent to a
        // spot 80 px off with three enemy FIGHTERs coming for it, it picks no course of its
        // own, on the way or there (any other ship with a gun does, in a fight), and once it
        // has stopped at the spot it doesn't move again.
        let mut a = quiet();
        let (x, y) = (700, 500);
        let s = a.spawn(0, 3, &crate::pg::teams::scavengers()[3], x, y, 0, false);
        let spot = (x, y - 80);
        a.ships[s].sent = Some(spot);
        for k in 0..3 {
            let r = a.spawn(1, 0, &starter_deck()[0], x - 40 + 40 * k, y - 300, 2, false);
            a.ships[r].sent = Some(spot);
        }
        let (mut stopped, mut fought) = (None, 0);
        for t in 0..15 * 40 {
            a.step();
            let me = &a.ships[s];
            assert!(me.alive && me.st.armed() && me.st.stasis > 0, "the SNARE is gone, or its guns or its lattice, at {} s", t / 15);
            assert!(me.course.is_none(), "at {} s the SNARE is on a course of its own", t / 15);
            fought += enemy_in_sight(&a, s) as u32;
            match stopped {
                Some(at) => assert_eq!((me.x, me.y), at, "it moved after it had stopped, at {} s", t / 15),
                None if (me.vx, me.vy) == (0, 0) && cheb(me.x, me.y, spot.0, spot.1) <= 20 => stopped = Some((me.x, me.y)),
                None => {}
            }
        }
        println!("stopped at {stopped:?}, {} px from where it was sent; {} s with an enemy in sight", stopped.map_or(-1, |p| cheb(p.0, p.1, spot.0, spot.1)), fought / 15);
        assert!(stopped.is_some() && fought >= 15 * 5, "{stopped:?}, {fought} ticks with an enemy in sight");
    }

    #[test]
    fn the_scavengers_open_with_a_port_that_prints_its_miners_and_they_bank_on_its_pad() {
        // The opening a person would play: the PORT planted on the home rock at once (135 of
        // the 190 px a king starts with), and then nothing but watching. It grows, prints
        // four DIGGERs and a GLEANER and then its repeat row, and the miners unload on its
        // pad. At 5:00, 150 px of wreckage are strewn 130 px from the port, and by 8:00 the
        // GLEANER has had them home.
        let (mut a, team) = scavenging();
        let (_, target) = home_rock(&a);
        a.command(Cmd::Print { team: 0, slot: 7, target: Some(target) });
        let alive = |a: &Arena, name: &str| a.ships.iter().filter(|s| s.alive && s.team == 0 && s.name == name && s.maker.is_none() && s.grow.is_none() && s.warp == 0).count();
        let mut grown = None;
        while a.tick < 15 * 300 {
            a.step();
            if grown.is_none() && alive(&a, "PORT") == 1 {
                grown = Some(a.tick);
            }
        }
        let port = (0..a.ships.len()).find(|&i| a.ships[i].alive && a.ships[i].name == "PORT").expect("no port at 5:00");
        println!("the port had grown by {} s; at 5:00: {} diggers, {} gleaners, {} darts, {} pulsars; mined {}, of it landed on a pad {}; bank {}", grown.map_or(0, |t| t / 15), alive(&a, "DIGGER"), alive(&a, "GLEANER"), alive(&a, "DART"), alive(&a, "PULSAR"), a.score[0].mined, a.score[0].landed, a.bank(port));
        assert!(grown.is_some_and(|t| t < 15 * 150) && a.ships[port].px_now() == team[7].stats().count);
        assert!(alive(&a, "DIGGER") >= 4 && alive(&a, "GLEANER") >= 1 && alive(&a, "DART") >= 1);
        assert!(a.score[0].landed >= 100 && a.score[0].mined >= 150, "mined {} and landed {} by 5:00", a.score[0].mined, a.score[0].landed);
        let (px, py) = (a.ships[port].x, a.ships[port].y);
        let cell = strew(&mut a, (px + 110, py - 70), 150);
        let (mined, landed) = (a.score[0].mined, a.score[0].landed);
        let in_cell = |a: &Arena| a.debris.iter().filter(|d| (d.x.div_euclid(HEAP), d.y.div_euclid(HEAP)) == cell).count();
        let mut clear = None;
        while a.tick < 15 * 480 {
            a.step();
            if clear.is_none() && in_cell(&a) == 0 {
                clear = Some(a.tick);
            }
        }
        println!("the wreckage was all taken by {:?} s after it fell; by 8:00 mined {} more, landed {} more", clear.map(|t| (t - 15 * 300) / 15), a.score[0].mined - mined, a.score[0].landed - landed);
        assert!(clear.is_some(), "at 8:00 {} of the 150 still lie there", in_cell(&a));
        assert!(a.score[0].landed - landed >= 150);
    }

    #[test]
    fn the_scavengers_pace_ship_speeds_their_big_ships_and_their_snares_own_guns_arent_held() {
        let (mut a, team) = scavenging();
        // A HERALD with a MAUL and a SNARE beside it: both are a step faster, 11 px/s for 7.
        let (x, y) = (700, 500);
        let pace = a.spawn(0, 4, &team[4], x, y, 0, false);
        let maul = a.spawn(0, 2, &team[2], x - 22, y + 4, 0, false);
        let snare = a.spawn(0, 3, &team[3], x + 26, y + 4, 0, false);
        for i in [pace, maul, snare] {
            a.ships[i].sent = Some((a.ships[i].x, a.ships[i].y));
        }
        a.step();
        assert_eq!((a.ships[maul].sped, a.ships[snare].sped, a.ships[pace].sped, a.ships[maul].st.vmax, a.ships[snare].st.vmax), (true, true, false, 3, 3));
        // The SNARE, parked, with red fighters come within its lattice's reach: they're held,
        // the MAUL beside it is held, and the SNARE isn't: its tracker and flak fire as ever.
        let fighters: Vec<usize> = (0..4).map(|n| a.spawn(1, 0, &starter_deck()[0], x + 26 + 40, y - 30 + 20 * n, 3, false)).collect();
        for _ in 0..STASIS_RISE + 3 {
            a.step();
        }
        assert!(a.stasis.iter().any(|f| f.ship == snare && f.up && f.reach == 60), "{:?}", a.stasis);
        assert!(!a.ships[snare].slowed && a.ships[maul].slowed && fighters.iter().all(|&i| !a.ships[i].alive || a.ships[i].slowed));
        let fired = a.score[0].fired;
        for _ in 0..15 * 4 {
            a.step();
        }
        assert!(a.score[0].fired > fired, "the snare and the maul fired nothing in four seconds");
    }

    #[test]
    fn every_scoop_bites() {
        // Miners with one, two, three and six scoops, each alone at the face of a rock with
        // no tractor in reach. From its first bite to its third is two bites' time, and a
        // bite's time is `SCOOP_TICKS` over the scoops: 10 s for one scoop, and a sixth of
        // that for six. (Until 2026-11-17 two scoops bit a pixel every 50 ticks, one took 100,
        // and three or six bit no faster than two.) The starter miner's three bite as they did.
        let base = quiet();
        let face = rock_face(&base, 90, 400);
        let starter = starter_deck()[3].clone();
        let drawings = [
            (Design::from_rows("ONE", &[".s.", "hch", "e.e"]), 1),
            (Design::from_rows("TWO", &["s.s", "hch", "e.e"]), 2),
            (starter, 3),
            (Design::from_rows("SIX", &["sss", "sss", "hch", "hhh", "hhh", "h.h", "e.e"]), 6),
        ];
        for (d, scoops) in drawings {
            assert!(d.check().ok() && d.stats().scoops == scoops && d.stats().power_pct() == 100, "{}", d.name);
            let mut a = base.clone();
            let m = a.spawn(0, 3, &d, face.0, face.1, 0, false);
            let mut bites = Vec::new();
            while bites.len() < 3 {
                let before = a.ships[m].carry;
                a.step();
                if a.ships[m].carry > before {
                    bites.push(a.tick);
                }
                assert!(a.tick < 15 * 60, "{} took {} bites in a minute", d.name, bites.len());
            }
            assert_eq!((bites[2] - bites[0]) as i32, 2 * SCOOP_TICKS / scoops, "{}: its bites came at {bites:?}", d.name);
        }
        assert_eq!((SCOOP_TICKS, SCOOP_TICKS / 3), (150, CHEW_TICKS));
    }

    #[test]
    fn a_miner_with_a_tractor_of_its_own_empties_itself_where_it_works() {
        // Henry, 2026-11-17: "What happens when tractor and scoop is on the same ship?" Three
        // scoops and four tractor pixels on one hull. With a scoop it's a miner, so it goes to
        // the rock and works; its tractor reaches its own hold, so what it bites is banked in
        // itself as it bites, it never fills and it never carries a load anywhere. It's a
        // node, so within link of its side's network what it holds can be spent.
        let d = Design::from_rows("HARVESTER", &["hsssh", "huhuh", "hhchh", "huhuh", "e...e"]);
        let st = d.stats();
        assert!(d.check().ok() && (st.scoops, st.tractor) == (3, 4) && st.power_pct() == 100 && !st.collector() && d.does() == "MINES");
        let link = KING_RANGE + NODE_RANGE;
        let work = |near: i32, far: i32| {
            let mut a = quiet();
            let k = a.eco.as_ref().unwrap().king[0];
            let (kx, ky) = (a.ships[k].x, a.ships[k].y);
            let face = rock_face(&a, near, far);
            let m = a.spawn(0, 3, &d, face.0, face.1, 0, false);
            // A collector beside the king, with nothing to do unless this miner needs one.
            let c = a.spawn(0, 7, &starter_deck()[7], kx + 40, ky, 0, false);
            let home = (a.ships[c].x, a.ships[c].y);
            let mut most = 0;
            for _ in 0..15 * 90 {
                a.step();
                most = most.max(a.ships[m].carry);
                assert!(a.ships[m].doing != Doing::Haul, "it set off with a load at {}", a.tick);
            }
            let linked = a.eco.as_ref().unwrap().net[m] == a.eco.as_ref().unwrap().net[k];
            (a.ships[m].store, most, linked, far_from(&a, m, k), cheb(a.ships[c].x, a.ships[c].y, home.0, home.1), a.bank(k), a.ships[k].store)
        };
        // In link of the king: a minute and a half of three scoops is 27 px, less the flight
        // to its first pixel. It never held more than a pixel or two, and the king's bank is
        // the king's own store and this ship's.
        let (store, most, linked, _, strayed, bank, kings) = work(90, link - 40);
        assert!((20..=27).contains(&store) && most <= 2 && linked, "it banked {store} px, held {most} at most, in link: {linked}");
        assert_eq!(bank, kings + store);
        // Its four tractor pixels have room for its own three scoops, so the collector has
        // nobody to stand by and stays where it was built.
        assert!(strayed <= 12, "the collector went {strayed} px off to a miner that empties itself");
        // Out of link it does just the same, and what it banks sits in it: nothing brings it
        // home. (Sent back into link by hand, it joins the network again.)
        let (store, most, linked, out, _, bank, kings) = work(link + 120, link + 300);
        assert!((20..=27).contains(&store) && most <= 2 && !linked && out > link, "{store} px, {most} held, linked {linked}, {out} px out");
        assert_eq!(bank, kings);
    }

    #[test]
    fn a_miners_load_comes_off_at_a_tractor_a_pixel_at_a_time() {
        // A full miner 30 px from its king. Nothing is banked the moment it's there, as it was
        // until 2026-11-16: the king's eight tractor pixels pull a pixel a second out of it.
        let mut a = quiet();
        let k = a.eco.as_ref().unwrap().king[0];
        let (kx, ky) = (a.ships[k].x, a.ships[k].y);
        let m = full_miner(&mut a, kx + 30, ky);
        let cap = a.ships[m].carry;
        assert_eq!((a.tractor_px(k), cap, TRACTOR_TICKS), (8, 9, 120));
        assert!(a.tractor_reach(k) >= 40, "the king's tractor reaches {} px", a.tractor_reach(k));
        let (bank, mined, made) = (a.ships[k].store, a.score[0].mined, a.eco.as_ref().unwrap().made[0]);
        let mut came = Vec::new();
        for _ in 0..15 * 10 {
            let before = a.ships[m].carry;
            a.step();
            let now = a.ships[m].carry;
            assert!(now == before || now == before - 1, "{before} px to {now} in a tick");
            if now < before {
                came.push(a.tick);
                assert_eq!(a.ships[m].pulled, Some((k, a.tick)), "the pull isn't marked for the picture");
            }
            if now > 0 {
                assert!(far(&a, m, k) <= a.tractor_reach(k) && a.ships[m].doing == Doing::Haul, "it left with {now} px still aboard");
            }
        }
        assert_eq!((a.ships[m].carry, came.len() as i32), (0, cap));
        assert!(came.windows(2).all(|w| w[1] - w[0] == 15), "a pixel a second: {came:?}");
        // Banked as it came: the king holds the load and what it made by itself, and the load
        // is what was mined.
        assert_eq!(a.score[0].mined - mined, cap);
        assert_eq!(a.ships[k].store - bank, cap + a.eco.as_ref().unwrap().made[0] - made);
        // Empty, it's off to the rock again.
        assert!(a.ships[m].doing == Doing::Mine && a.ships[m].dock.is_none(), "{}", a.ships[m].doing.name());
    }

    #[test]
    fn a_factory_with_no_tractor_takes_nothing_in() {
        // The starter deck's outpost as it was drawn before tractors, 300 px from the king, and
        // a full miner beside it. The factory is a node and takes nothing: the miner flies to
        // the king, whose tractor is the only one its side has.
        let mut a = quiet();
        let k = a.eco.as_ref().unwrap().king[0];
        let (kx, ky) = (a.ships[k].x, a.ships[k].y);
        let old = crate::pg::design::retired_starters().into_iter().filter(|(slot, _)| *slot == 4).last().unwrap().1;
        assert_eq!((old.stats().factory, old.stats().tractor), (28, 0));
        let f = a.spawn(0, 4, &old, kx + 300, ky, 0, false);
        let m = full_miner(&mut a, kx + 300, ky - 20);
        let cap = a.ships[m].carry;
        for _ in 0..15 * 4 {
            a.step();
        }
        assert!(a.ships[f].node && a.ships[f].store == 0 && a.ships[m].carry == cap);
        assert_eq!(a.ships[m].dock, Some(k));
        while a.ships[m].carry > 0 {
            a.step();
            assert!(a.tick < 15 * 60, "it never unloaded: it's {} px from its king", far(&a, m, k));
        }
        assert!(far(&a, m, k) <= a.tractor_reach(k) && a.ships[f].store == 0);
    }

    #[test]
    fn a_full_miner_goes_where_it_will_be_empty_soonest() {
        // Two outposts far from the king, one 60 px from a full miner and one 130 px the other
        // way. With nobody waiting it takes the nearer. With five full miners waiting at the
        // nearer (45 px, at a pixel every 2 s), it flies the 70 px further to the one that's
        // free. And once it has chosen it keeps to its choice.
        let pick = |waiting: i32| {
            let mut a = quiet();
            let (x, y) = (700, 500);
            let d = starter_deck();
            let near = a.spawn(0, 4, &d[4], x + 60, y, 0, false);
            let free = a.spawn(0, 4, &d[4], x - 130, y, 0, false);
            assert_eq!((a.tractor_px(near), a.tractor_px(free)), (4, 4));
            for n in 0..waiting {
                let w = full_miner(&mut a, x + 60, y - 30 + 6 * n);
                (a.ships[w].hauling, a.ships[w].dock) = (true, Some(near));
            }
            a.step();
            let m = full_miner(&mut a, x, y);
            a.step();
            let first = a.ships[m].dock;
            for _ in 0..30 {
                a.step();
            }
            assert_eq!(a.ships[m].dock, first, "it changed its mind on the way");
            (first, near, free)
        };
        let (dock, near, _) = pick(0);
        assert_eq!(dock, Some(near));
        let (dock, _, free) = pick(5);
        assert_eq!(dock, Some(free));
    }

    /// A station with nothing but a core, a strip of hull, and under it a dock pad drawn as
    /// `rows` (k where there's pad): no tractor.
    fn pad_station(rows: &[&str]) -> Design {
        let w = rows[0].len();
        let mut all = vec![format!("{}c{}", "h".repeat(w / 2), "h".repeat(w - w / 2 - 1))];
        all.extend(rows.iter().map(|r| r.to_string()));
        Design::from_rows("PAD", &all.iter().map(|r| r.as_str()).collect::<Vec<_>>())
    }

    /// A pad that's a solid block.
    fn pad_block(w: usize, h: usize) -> Design {
        let rows = vec!["k".repeat(w); h];
        pad_station(&rows.iter().map(|r| r.as_str()).collect::<Vec<_>>())
    }

    /// Whether every living pixel of ship `m` is over a living dock pixel of ship `p`.
    fn on_the_pad(a: &Arena, m: usize, p: usize) -> bool {
        (0..a.ships[m].hp.len()).filter(|&c| a.ships[m].hp[c] > 0).map(|c| a.ships[m].cell_xy(c)).all(|(x, y)| a.ships[p].cell_at(x, y).is_some_and(|c| a.ships[p].hp[c] > 0 && a.ships[p].shape.cells[c] == Some(Part::DockPad)))
    }

    fn boxes_touch(a: &Arena, i: usize, j: usize) -> bool {
        let ((l, t, r, b), (ol, ot, or, ob)) = (a.ships[i].bbox(), a.ships[j].bbox());
        !(or < l || ol > r || ob < t || ot > b)
    }

    #[test]
    fn a_pad_brings_a_miner_on_takes_its_whole_load_and_puts_it_off_clear() {
        // Henry, 2026-11-16: "it drains the miner's whole cargo instantly"; and 2026-11-17:
        // "It's like they're being tractored in ... the full on->tile->off sequence", "Miners
        // should clear the area of the pad once they're done offloading."
        //
        // A station with a pad 3 by 5 and no tractor, and a full starter miner (3 by 5) 80 px
        // off. It flies up and waits clear of the station; the pad takes hold of it and
        // brings it on; the tick it's down all nine pixels are in the station's store, with
        // pad under every pixel of the miner; and the pad puts it off again, clear of the
        // station.
        let mut a = quiet();
        let (x, y) = (700, 500);
        let p = a.spawn(0, 4, &pad_block(3, 5), x, y, 0, false);
        assert_eq!((a.tractor_px(p), a.is_node(p)), (0, true));
        let m = full_miner(&mut a, x + 80, y + 10);
        assert_eq!(a.pad_room(m, p), 1);
        let (load, mined, held) = (a.ships[m].carry, a.score[0].mined, a.ships[p].store);
        let (mut held_at, mut down, mut off) = (None, None, None);
        for t in 0..15 * 30 {
            let before = (a.ships[m].berth, a.ships[m].carry, (a.ships[m].x, a.ships[m].y));
            a.step();
            let s = &a.ships[m];
            if before.0.is_none() && s.berth.is_some() && down.is_none() {
                held_at = Some(t);
                assert!(!boxes_touch(&a, m, p) || before.0.is_some(), "the pad took hold of it over the station");
            }
            if let Some(b) = s.berth {
                // In the pad's hands it goes no faster than the pad moves it, and not of itself.
                assert!((s.vx, s.vy) == (0, 0) && cheb(s.x, s.y, before.2 .0, before.2 .1) <= DOCK_SPEED, "held, it went from {:?} to {:?} in a tick", before.2, (s.x, s.y));
                if before.1 > 0 && s.carry < before.1 {
                    assert!(s.carry == 0 && b.phase == 1 && on_the_pad(&a, m, p), "it unloaded {} of {} at {:?}, phase {}", before.1 - s.carry, before.1, (s.x - x, s.y - y), b.phase);
                    down = Some(t);
                }
            } else if before.0.is_some() && down.is_some() {
                assert!(!boxes_touch(&a, m, p), "the pad let go of it at {:?}, still over the station", (s.x - x, s.y - y));
                off = Some(t);
                break;
            }
            assert!(down.is_some() || s.carry == load, "it lost some of its load before it was down");
        }
        println!("held at {held_at:?}, down at {down:?}, off at {off:?}");
        assert!(held_at.is_some() && down.is_some() && off.is_some(), "half a minute on: held {held_at:?}, down {down:?}, off {off:?}; it's at {:?} going {:?}", (a.ships[m].x - x, a.ships[m].y - y), (a.ships[m].vx, a.ships[m].vy));
        assert_eq!((a.score[0].mined - mined, a.ships[p].store - held, a.score[0].landed), (load, load, load));
        assert!(!a.ships[m].hauling && a.ships[m].dock.is_none());
    }

    #[test]
    fn a_miner_fits_a_pad_that_matches_its_profile_whichever_way_its_turned() {
        // Henry, 2026-11-17: "I don't think the miner should need to be able to fit any
        // direction. Actually I think it should just need to fit on an exact match of its
        // profile". A starter miner is drawn .g. / sss / hch / hhh / e.e: thirteen pixels in a
        // box 3 by 5. How many of it a pad holds:
        let room = |pad: Design, shot: Option<(i32, i32)>| {
            let mut a = quiet();
            let p = a.spawn(0, 4, &pad, 700, 500, 0, false);
            let m = full_miner(&mut a, 780, 510);
            if let Some((dx, dy)) = shot {
                // A pad pixel shot out: so far across and down from the pad's top-left.
                let (l, t, _, _) = a.ships[p].bbox();
                let c = a.ships[p].cell_at(l + dx, t + 1 + dy).unwrap();
                assert_eq!(a.ships[p].shape.cells[c], Some(Part::DockPad));
                a.ships[p].hp[c] = 0;
            }
            a.pad_room(m, p)
        };
        // A pad of exactly its shape, and no more: one.
        assert_eq!(room(pad_station(&[".k.", "kkk", "kkk", "kkk", "k.k"]), None), 1);
        // The same block lying on its side: one, turned.
        assert_eq!(room(pad_block(5, 3), None), 1);
        // A block a pixel too narrow, or a pixel too short: none.
        assert_eq!((room(pad_block(2, 5), None), room(pad_block(3, 4), None)), (0, 0));
        // A block 3 by 5 with a corner shot out, where the miner has no pixel: still one, the
        // one way round. With its middle shot out: none.
        assert_eq!((room(pad_block(3, 5), Some((0, 0))), room(pad_block(3, 5), Some((1, 2)))), (1, 0));
        // And a block 6 by 5 holds two side by side; 9 by 5, three; 6 by 10, four.
        assert_eq!((room(pad_block(6, 5), None), room(pad_block(9, 5), None), room(pad_block(6, 10), None)), (2, 3, 4));

        // On the block lying on its side the pad turns the miner to fit, and takes its load.
        let mut a = quiet();
        let p = a.spawn(0, 4, &pad_block(5, 3), 700, 500, 0, false);
        let m = full_miner(&mut a, 780, 510);
        let mut down = false;
        for _ in 0..15 * 30 {
            let before = a.ships[m].carry;
            a.step();
            if before > 0 && a.ships[m].carry == 0 {
                assert!(a.ships[m].facing & 1 == 1 && on_the_pad(&a, m, p), "down facing {}, at {:?}", a.ships[m].facing, (a.ships[m].x - 700, a.ships[m].y - 500));
                down = true;
                break;
            }
        }
        assert!(down, "it never unloaded on a pad it fits turned on its side");
    }

    #[test]
    fn a_big_pad_takes_miners_side_by_side_and_where_one_sat_rests_three_seconds() {
        // Henry, 2026-11-17: "if there's extra room it would be cool if the miners tiled it
        // and multiple could offload at the same time", and "The three second cooldown can be
        // counted per pixel". A pad 6 by 5 holds two starter miners. Three full ones wait by
        // it: two are brought on and unload within a second of each other, side by side, and
        // the third has to wait for a place to have rested three seconds.
        let mut a = quiet();
        let (x, y) = (700, 500);
        let p = a.spawn(0, 4, &pad_block(6, 5), x, y, 0, false);
        let miners = [full_miner(&mut a, x + 24, y - 4), full_miner(&mut a, x + 24, y + 4), full_miner(&mut a, x - 24, y)];
        assert_eq!(a.pad_room(miners[0], p), 2);
        let mut downs: Vec<(u32, usize)> = Vec::new();
        for _ in 0..15 * 40 {
            let before: Vec<i32> = miners.iter().map(|&m| a.ships[m].carry).collect();
            a.step();
            for (n, &m) in miners.iter().enumerate() {
                if before[n] == 9 && a.ships[m].carry == 0 {
                    assert!(on_the_pad(&a, m, p));
                    // Nobody else is sitting where it sits.
                    for &o in miners.iter().filter(|&&o| o != m && a.ships[o].berth.is_some_and(|b| b.phase == 1)) {
                        assert!(!boxes_touch(&a, m, o), "two miners unloaded on one place");
                    }
                    downs.push((a.tick, n));
                }
            }
            // Never more than two in the pad's hands on their way on or sitting.
            assert!(miners.iter().filter(|&&m| a.ships[m].berth.is_some_and(|b| b.phase <= 1)).count() <= 2);
            if downs.len() == 3 {
                break;
            }
        }
        for _ in 0..15 * 4 {
            a.step();
        }
        println!("{downs:?}");
        assert_eq!(downs.len(), 3, "who unloaded, and when: {downs:?}");
        assert!(downs[1].0 - downs[0].0 < 15, "the first two should be down within a second of each other: {downs:?}");
        assert!(downs[2].0 >= downs[0].0 + DOCK_TURN as u32, "the third was down before a place had rested: {downs:?}");
        // And all three are put off clear of the station afterwards.
        assert!(miners.iter().all(|&m| a.ships[m].berth.is_none()), "a miner is still in the pad's hands");
    }

    #[test]
    fn a_flying_ship_with_a_pad_goes_to_the_miners_and_they_land_on_it() {
        // A carrier: a pad five wide, hull, engines, no tractor and no scoop. Like a collector
        // it goes to where its side's miners are at work, as far as still links, and they
        // bring their loads to its pad.
        let carrier = Design::from_rows("CARRIER", &["hkkkkkh", "hkkkkkh", "hkkkkkh", "hkkkkkh", "hkkkkkh", "hhhchhh", "e..e..e"]);
        assert!(carrier.check().problem().is_none() && carrier.stats().collector() && carrier.stats().tractor == 0);
        let mut a = quiet();
        let k = a.eco.as_ref().unwrap().king[0];
        let (kx, ky) = (a.ships[k].x, a.ships[k].y);
        let link = KING_RANGE + NODE_RANGE;
        let face = {
            let e = a.eco.as_ref().unwrap();
            let r = e.rocks.iter().filter(|r| (link + 120..link + 300).contains(&isqrt(dist2(r.cx, r.cy, kx, ky)))).min_by_key(|r| dist2(r.cx, r.cy, kx, ky)).expect("no rock well out of link of the king");
            r.px.iter().copied().min_by_key(|&(x, y)| dist2(x, y, kx, ky)).unwrap()
        };
        let d = starter_deck();
        for n in 0..3 {
            a.spawn(0, 3, &d[3], face.0 - 3 + 3 * n, face.1, 0, false);
        }
        let c = a.spawn(0, 7, &carrier, kx + 40, ky, 0, false);
        assert_eq!(a.tractor_px(c), 0);
        let mined = a.score[0].mined;
        for _ in 0..15 * 150 {
            a.step();
        }
        let out = far(&a, c, k);
        println!("the carrier is {out} px out and holds {}; mined {}", a.ships[c].store, a.score[0].mined - mined);
        assert!(out > link / 2, "the carrier stayed {out} px from its king");
        assert!(a.ships[c].store > 0, "nothing was landed on the carrier in two and a half minutes");
    }

    /// The miners the reach of a scoop is tried on: the starter's, the starter's before a
    /// scoop used power, Henry's of 2026-11-17 (the afternoon's, and the night's: 24 px
    /// weighing 31, engines one way, 3 s to a quarter turn), the example hauler, and one with
    /// a rotation engine and an engine astern.
    fn miners_of_every_sort() -> Vec<Design> {
        let mut all = vec![starter_deck()[3].clone(), Design::from_rows("OLD MINER", &["sss", "hch", "e.e"])];
        all.extend(["MINER PM|47,46|..h../..g../.sss./hhchh/.e.e.|BUILT|", "MINER NIGHT|47,46|hh.hh/shghs/sssss/hhchh/heheh|BUILT|"].iter().map(|l| Design::from_line(l).unwrap()));
        all.push(crate::pg::design::part_examples().into_iter().find(|d| d.name == "HAULER").unwrap());
        all.push(Design::from_rows("NIMBLE", &[".q.", "sss", "ych", "hhh", "e.e"]));
        all
    }

    #[test]
    fn a_scoop_reaches_eight_pixels_and_every_miner_gets_that_near_the_rock() {
        // Henry, 2026-11-17: "Let's just give scoops a short, fixed range of 8 px ... Ships
        // should be able to fly short distances, the flying algorithm might need to be
        // improved if they can't." Each miner alone at its king's home rock for four
        // minutes, where the king's tractor empties it as it works. Every pixel it bites is
        // within 8 px of its middle when it bites it; it's biting inside half a minute; and
        // it never goes a minute without a bite, however slow it is to turn.
        let mut said = Vec::new();
        for d in miners_of_every_sort() {
            assert!(d.check().problem().is_none(), "{} isn't legal: {:?}", d.name, d.check().problem());
            let mut a = quiet();
            let (_, at) = home_rock(&a);
            let k = a.eco.as_ref().unwrap().king[0];
            let (kx, ky) = (a.ships[k].x, a.ships[k].y);
            // It starts 50 px from the rock's middle, on the king's side.
            let far = isqrt(dist2(at.0, at.1, kx, ky)).max(1);
            let m = a.spawn(0, 3, &d, at.0 + (kx - at.0) * 50 / far, at.1 + (ky - at.1) * 50 / far, 0, false);
            let (mut bites, mut first, mut last, mut gap, mut furthest) = (0, None, 0u32, 0u32, 0);
            let mut was = a.ships[m].worked;
            for t in 0..15 * 240u32 {
                let rock = a.rock.iter().filter(|&&r| r).count();
                a.step();
                if a.rock.iter().filter(|&&r| r).count() < rock {
                    let (s, p) = (&a.ships[m], a.ships[m].worked.unwrap());
                    assert!(a.ships[m].worked != was || bites > 0);
                    let off = cheb(s.x, s.y, p.0, p.1);
                    assert!(off <= 8, "{} bit a pixel {off} px from its middle", d.name);
                    furthest = furthest.max(off);
                    if first.is_none() {
                        first = Some(t);
                    } else {
                        gap = gap.max(t - last);
                    }
                    (bites, last, was) = (bites + 1, t, a.ships[m].worked);
                }
            }
            gap = gap.max(15 * 240 - last);
            said.push(format!("{} ({} scoops, {} s a quarter turn): first bite at {} s, {bites} bites, longest wait {} s, furthest {furthest} px", d.name, d.stats().scoops, secs(d.stats().quarter), first.map_or(-1, |t| t as i32 / 15), gap / 15));
            assert!(first.is_some_and(|t| t <= 15 * 30), "{} hadn't bitten in half a minute: {:?}", d.name, said.last());
            assert!(gap <= 15 * 60, "{} went {} s without a bite: {:?}", d.name, gap / 15, said.last());
        }
        for l in &said {
            println!("{l}");
        }
    }

    #[test]
    fn a_tractor_empties_the_miners_waiting_before_the_ones_at_work_and_the_nearly_empty_first() {
        // Three miners in reach of one outpost's tractor: two waiting to be emptied, with 9 px
        // and 2 px, and one still at work with 5. The one with 2 goes first and is away; then
        // the one with 9; the one at work is left until nobody is waiting.
        let mut a = quiet();
        let (x, y) = (700, 500);
        let d = starter_deck();
        let f = a.spawn(0, 4, &d[4], x, y, 0, false);
        let full = full_miner(&mut a, x + 20, y);
        let two = full_miner(&mut a, x, y + 20);
        let busy = full_miner(&mut a, x - 20, y);
        (a.ships[two].carry, a.ships[busy].carry) = (2, 5);
        (a.ships[two].hauling, a.ships[busy].hauling) = (true, false);
        let mined = a.score[0].mined;
        let mut order = Vec::new();
        for _ in 0..15 * 2 * 8 {
            let before = [a.ships[full].carry, a.ships[two].carry, a.ships[busy].carry];
            a.step();
            for (n, &m) in [full, two, busy].iter().enumerate() {
                if a.ships[m].carry < before[n] {
                    order.push(n);
                }
            }
        }
        // (The one at work flies off to its rock, out of reach, long before its turn.)
        assert_eq!(order, vec![1, 1, 0, 0, 0, 0, 0, 0], "who was pulled from, pull by pull");
        // Eight pixels in sixteen seconds, through the outpost's four tractor pixels.
        assert_eq!((a.score[0].mined - mined, a.tractor_px(f)), (8, 4));
    }

    #[test]
    fn a_collector_stands_by_the_miners_in_link_with_its_network() {
        // Four miners at work on a rock some way from blue's king, and a collector printed
        // beside the king. It flies out and stands where its tractor covers their work: they
        // never have to leave the rock, and what they bite is banked through it.
        let mut a = quiet();
        let k = a.eco.as_ref().unwrap().king[0];
        let (kx, ky) = (a.ships[k].x, a.ships[k].y);
        let link = KING_RANGE + NODE_RANGE;
        // A rock out of the king's own reach and well inside link of it.
        let face = {
            let e = a.eco.as_ref().unwrap();
            let r = e.rocks.iter().filter(|r| (90..link - 40).contains(&isqrt(dist2(r.cx, r.cy, kx, ky)))).min_by_key(|r| dist2(r.cx, r.cy, kx, ky)).expect("no rock between 90 px and link of the king");
            r.px.iter().copied().min_by_key(|&(x, y)| dist2(x, y, kx, ky)).unwrap()
        };
        let d = starter_deck();
        assert!(d[7].stats().collector() && d[7].does() == "COLLECTS");
        let miners: Vec<usize> = (0..4).map(|n| a.spawn(0, 3, &d[3], face.0 - 4 + 3 * n, face.1, 0, false)).collect();
        let c = a.spawn(0, 7, &d[7], kx + 40, ky, 0, false);
        for _ in 0..15 * 60 {
            a.step();
        }
        let s = &a.ships[c];
        assert!(s.doing == Doing::Collect && (s.vx, s.vy) == (0, 0), "after a minute it's {} at {:?}, going {:?}", s.doing.name(), (s.x, s.y), (s.vx, s.vy));
        let e = a.eco.as_ref().unwrap();
        assert!(e.net[c] != usize::MAX && e.net[c] == e.net[k], "it stands out of link with its king");
        // Every miner is in its reach, at work, and has carried nothing home.
        for &m in &miners {
            assert!(far(&a, m, c) <= a.tractor_reach(c), "a miner is {} px from it, and it reaches {}", far(&a, m, c), a.tractor_reach(c));
            assert!(a.ships[m].doing == Doing::Mine, "a miner is {}", a.ships[m].doing.name());
        }
        // A minute more: it holds its place, and banks what four miners bite, less what they
        // still carry (a pixel every 50 ticks each: 72 in a minute).
        let (at, store) = ((s.x, s.y), s.store);
        for _ in 0..15 * 60 {
            a.step();
            assert!(cheb(a.ships[c].x, a.ships[c].y, at.0, at.1) <= STAND_REST, "it moved off to {:?} from {at:?}", (a.ships[c].x, a.ships[c].y));
        }
        let banked = a.ships[c].store - store;
        assert!((60..=72).contains(&banked), "it banked {banked} px in a minute from four miners");
    }

    /// How far a collector may shift about its place while the miners it stands by eat their
    /// rock back (px): one hop.
    const STAND_REST: i32 = 24;

    #[test]
    fn a_collector_goes_no_further_from_its_network_than_still_links() {
        // The miners work a rock far out of link of anything. The collector stands on the way
        // to them, as far out as still links to its king: they fly the rest, and unload there.
        let mut a = quiet();
        let k = a.eco.as_ref().unwrap().king[0];
        let (kx, ky) = (a.ships[k].x, a.ships[k].y);
        let link = KING_RANGE + NODE_RANGE;
        let face = {
            let e = a.eco.as_ref().unwrap();
            let r = e.rocks.iter().filter(|r| (link + 120..link + 300).contains(&isqrt(dist2(r.cx, r.cy, kx, ky)))).min_by_key(|r| dist2(r.cx, r.cy, kx, ky)).expect("no rock well out of link of the king");
            r.px.iter().copied().min_by_key(|&(x, y)| dist2(x, y, kx, ky)).unwrap()
        };
        let d = starter_deck();
        for n in 0..3 {
            a.spawn(0, 3, &d[3], face.0 - 3 + 3 * n, face.1, 0, false);
        }
        let c = a.spawn(0, 7, &d[7], kx + 40, ky, 0, false);
        for _ in 0..15 * 90 {
            a.step();
        }
        // A minute and a half on it's out there, and standing still most of the time: it
        // shifts a little as its miners move along the rock's face, so it's looked at over
        // the next half minute, and judged where it's next at rest.
        let (mut still, mut rested) = (0, None);
        for _ in 0..15 * 30 {
            a.step();
            if (a.ships[c].vx, a.ships[c].vy) == (0, 0) {
                still += 1;
                rested = rested.or(Some(far(&a, c, k)));
            }
            let out = far(&a, c, k);
            assert!((link - 24..=link).contains(&out), "it's {out} px from its king, and links at {link}");
        }
        println!("at rest {still} of 450 ticks");
        assert!(still >= 225, "it stood still for {still} ticks of 450");
        let s = &a.ships[c];
        let out = rested.unwrap();
        assert!((link - 24..=link).contains(&out), "it stands {out} px from its king, and links at {link}");
        let e = a.eco.as_ref().unwrap();
        assert!(e.net[c] == e.net[k], "it stands out of link with its king");
        // On the way to the rock: it has made up nearly all of the link's length towards it.
        let to_face = |x: i32, y: i32| isqrt(dist2(x, y, face.0, face.1));
        assert!(to_face(s.x, s.y) <= to_face(kx, ky) - link + 30, "it's {} px from the rock, and its king is {}", to_face(s.x, s.y), to_face(kx, ky));
        // The miners bring their loads to it, not to the king.
        assert!(a.ships[c].store > 0, "nothing was unloaded at it in a minute and a half");
    }

    #[test]
    fn what_a_flying_node_holds_is_spent_before_what_a_standing_one_holds() {
        // Henry, 2026-11-16: "Resources get used from mobile collectors first before getting
        // used from stationary outposts." A collector in link with the king, holding 20 px,
        // and the king holding 100: the king prints a fighter, and the collector pays.
        let mut a = quiet();
        let k = a.eco.as_ref().unwrap().king[0];
        let (kx, ky) = (a.ships[k].x, a.ships[k].y);
        let d = starter_deck();
        let c = a.spawn(0, 7, &d[7], kx + 60, ky, 0, false);
        (a.ships[k].store, a.ships[c].store) = (100, 20);
        a.eco.as_mut().unwrap().made[0] += 20 + 100 - START_BANK;
        let price = d[0].stats().cost;
        let made = a.eco.as_ref().unwrap().made[0];
        a.command(Cmd::Print { team: 0, slot: 0, target: None });
        while a.eco.as_ref().unwrap().tally[0].summoned == 0 {
            a.step();
            assert!(a.tick < 15 * 60, "the king never printed its fighter");
        }
        assert_eq!(a.ships[c].store, 20 - price, "the collector didn't pay for the fighter");
        assert_eq!(a.ships[k].store, 100 + a.eco.as_ref().unwrap().made[0] - made, "the king paid");
        // With the collector empty, the king pays.
        a.ships[c].store = 0;
        a.eco.as_mut().unwrap().made[0] -= 20 - price;
        let (bank, made) = (a.ships[k].store, a.eco.as_ref().unwrap().made[0]);
        a.command(Cmd::Print { team: 0, slot: 0, target: None });
        while a.eco.as_ref().unwrap().tally[0].summoned == 1 {
            a.step();
            assert!(a.tick < 15 * 120, "the king never printed its second fighter");
        }
        assert_eq!(a.ships[k].store, bank - price + a.eco.as_ref().unwrap().made[0] - made);
    }

    #[test]
    fn a_collector_coming_and_going_is_no_split() {
        // A collector in link with the king is sent out of link and back. Its network parts
        // and joins again, and nothing says the network has split: it flew off by itself.
        let mut a = quiet();
        let k = a.eco.as_ref().unwrap().king[0];
        let (kx, ky) = (a.ships[k].x, a.ships[k].y);
        let c = a.spawn(0, 7, &starter_deck()[7], kx + 60, ky, 0, false);
        a.step();
        assert_eq!(a.eco.as_ref().unwrap().net[c], a.eco.as_ref().unwrap().net[k]);
        a.command(Cmd::Send { team: 0, ship: c, at: (kx + 400, ky), pace: 0 });
        let mut parted = false;
        for _ in 0..15 * 40 {
            a.step();
            parted |= a.eco.as_ref().unwrap().net[c] != a.eco.as_ref().unwrap().net[k];
        }
        assert!(parted, "it never got out of link: it's {} px from its king", far(&a, c, k));
        assert_eq!(a.eco.as_ref().unwrap().split_at[0], 0, "a collector flying off was called a split");
    }

    #[test]
    fn the_ai_prints_a_collector_when_its_miners_bite_faster_than_its_tractors_pull() {
        // The AI's king and one outpost: twelve tractor pixels, which pull a pixel every 10
        // ticks. Miners bite one every 50 each, and fly as well: the AI wants tractors for two
        // thirds of their bite. So seven miners are within what it has, and eight are over.
        let first = |miners: i32, off: u32| {
            let mut a = ai_alone();
            a.eco.as_mut().unwrap().off[0] = off;
            let k = a.eco.as_ref().unwrap().king[0];
            let (kx, ky) = (a.ships[k].x, a.ships[k].y);
            let f = friend(&mut a, OUTPOST, kx + 80, ky - 60);
            (a.ships[f].rooted, a.ships[f].node) = (true, true);
            for n in 0..miners {
                friend(&mut a, MINER, kx - 60 + 6 * n, ky + 40);
            }
            a.ships[k].store = 2000;
            a.score[0].mined += 2000 - START_BANK;
            think(&mut a);
            first_ship_printed(&a)
        };
        const COLLECTOR: usize = 7;
        assert_ne!(first(7, 0), Some(COLLECTOR), "seven miners are within what twelve tractor pixels pull");
        assert_eq!(first(8, 0), Some(COLLECTOR), "eight miners out-bite twelve tractor pixels, and the AI printed no collector");
        assert_ne!(first(8, RULE_TRACTORS), Some(COLLECTOR), "with the rule off it printed one");
    }

    #[test]
    fn full_miners_get_home_in_the_time_the_flight_takes() {
        // A whole economy at work: a side that plants factories and prints miners without a
        // pause, against the AI, for six minutes. Miners heap up on the rock faces, and each one
        // that fills has to get home past the rest. Every trip into the reach of a tractor that stayed
        // where it was, with nothing solid in the way, takes what the miner's own flight takes
        // with nothing else on the map (getting up speed, turning about, stopping where it
        // can hand over) and no more than 10 s on top. (When friends pushed a full miner like any other
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
                // A trip is from the tick a miner sets off with its load to the tick it's in
                // the reach of the tractor it chose.
                let dock = s.dock.filter(|_| s.alive && s.doing == Doing::Haul);
                let there = dock.is_some_and(|j| far(&a, i, j) <= a.tractor_reach(j));
                if let (Some(j), false) = (dock, there) {
                    if out[i].is_none() {
                        let at = (a.ships[j].x, a.ships[j].y);
                        out[i] = Some((a.tick, a.flight_time(i, at, a.tractor_reach(j) * 2 / 3) as i32, j, at, false));
                    }
                    if let Some(trip) = out[i].as_mut() {
                        trip.4 |= s.bumped;
                    }
                } else if let Some((t0, flight, node, at, bumped)) = out[i].take() {
                    let o = &a.ships[node];
                    // (A miner whose engines were shot off on the way can't make the trip at
                    // all, and a collector that moved off isn't the trip that was measured.)
                    if s.alive && there && o.alive && (o.x, o.y) == at && !bumped && s.st.vmax > 0 {
                        trips += 1;
                        worst = worst.max((a.tick - t0) as i32 - flight);
                    }
                }
            }
        }
        println!("{trips} trips, the worst {} ticks over its flight", worst);
        // (Over a hundred until 2026-11-16: a miner in reach of a tractor where it works makes
        // no trip at all now.)
        assert!(trips >= 50, "only {trips} trips home in six minutes");
        // (10 s: a miner with an enemy in sight picks its own course home, and one that stops
        // just short of where it can hand over has to set off again.)
        assert!(worst <= 150, "a full miner took {} s longer to get home than the flight takes", worst / 15);
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
            // (Twelve seconds: the seed has to get up speed, turn about and stop on its rock.)
            for _ in 0..15 * 12 {
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
    fn orders_are_an_ais_to_give_and_it_gives_them_to_everything_of_its_that_flies_without_a_scoop() {
        // A deck with a tender in its seventh slot: a ship that flies with no guns and no scoop.
        let mut deck = starter_deck();
        deck[6] = Design::from_rows("TENDER", &["hhh", "hch", "e.e"]);
        assert!(deck[6].check().ok() && !deck[6].stats().armed());
        // Red is a scripted side that puts everything on HUNT, and blue a person.
        let mut a = Arena::for_bots(3, [deck.clone(), deck.clone()], [Bot::None, Bot::Planter]);
        // The miner has a flak gun and a scoop: it's a miner to the AI, and no warship.
        let miner = a.eco().cards[1][3];
        assert!(miner.st.armed() && miner.miner() && !miner.warship() && !miner.stanced());
        assert!(a.eco().cards[1][0].warship() && !a.eco().cards[1][6].warship() && a.eco().cards[1][6].stanced());
        // Before anyone thinks: what flies without a scoop guards, on both sides.
        for t in 0..2 {
            assert_eq!((a.eco().orders[t][0], a.eco().orders[t][6], a.eco().orders[t][3]), (Order::Guard, Order::Guard, Order::Harvest));
        }
        for _ in 0..40 {
            a.step();
        }
        // The script's fighters, lancers and frigates are on HUNT, and so is its tender, which
        // goes with the fleet. Its miner still mines and its outpost holds.
        assert_eq!(a.eco().orders[1][..5], [Order::Hunt, Order::Hunt, Order::Hunt, Order::Harvest, Order::Hold]);
        assert_eq!(a.eco().orders[1][6], Order::Hunt, "the script left its unarmed ship at home");
        // The person's side is as it started: nobody gives its cards orders, and one sent as a
        // command from that side is dropped.
        a.command(Cmd::Order { team: 0, slot: 0, order: Order::Sab });
        a.command(Cmd::Order { team: 0, slot: 6, order: Order::Hunt });
        a.step();
        assert_eq!((a.eco().orders[0][0], a.eco().orders[0][6], a.eco().tally[0].orders), (Order::Guard, Order::Guard, 0));
        // The same command from the script's side is taken.
        a.command(Cmd::Order { team: 1, slot: 6, order: Order::Guard });
        a.step();
        assert_eq!(a.eco().orders[1][6], Order::Guard);
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
        // (Under 80 until ships chose how they fly, 2026-11-14: its fights go differently now,
        // and it's 84 in this match. Still one change of a design's order every 20 s or so.)
        assert!(switches(0) <= 100, "the AI changed orders {} times in eight minutes", switches(0));
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
        // The AI's bank covers an outpost and 45 px more, short of the 150 px it used to keep
        // back under a raid. An enemy fighter is parked in reach of its home rocks. Where its
        // seed is sent within a second, if anywhere.
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
        // the enemy, and how much nearer the place a second factory stands (or would): off to
        // one side, or (`past`) on the far side of the enemy.
        let after = |slot: usize, refuge: bool, past: bool| {
            let mut a = ai_alone();
            for _ in 0..15 * 60 {
                a.step();
            }
            let k = a.eco.as_ref().unwrap().king[0];
            let f = factory(&a).unwrap();
            let (kx, ky) = (a.ships[k].x, a.ships[k].y);
            let far = if past { (kx + 400, ky - 400) } else { (kx + 400, ky) };
            if refuge {
                // A second factory of the AI's, far along the map, with nothing near it: off to
                // one side of the way it backs off, or up the map past the enemy.
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
        let (beside, _, _) = after(FIGHTER, false, false);
        assert!(beside, "raided by a single fighter, the AI's king isn't at its post");
        // A capital outweighs them both. With nowhere else to go the king backs straight away
        // from it.
        let (beside, further, _) = after(CAPITAL, false, false);
        assert!(!beside && further >= 20, "outweighed, with one factory: beside it {beside}, {further} px further from the capital");
        // With a second factory that nothing is near, it makes for that instead. (Backing away
        // from the capital took it the other way: see the line above.)
        let (_, _, wrong_way) = after(CAPITAL, false, false);
        let (beside, _, nearer) = after(CAPITAL, true, false);
        assert!(wrong_way < 20, "with no second factory the king still went {wrong_way} px towards where one would be");
        assert!(!beside && nearer >= 60, "outweighed, with a quiet factory to fall back on, the king is {nearer} px nearer it");
        // But not for one the enemy is nearer than it is: it would have to pass the enemy to
        // get there. It backs away, as it does with none. (Until 2026-11-17 it made for the
        // nearest quiet factory wherever that was.)
        let (_, further, nearer) = after(CAPITAL, true, true);
        assert!(further >= 20 && nearer < 20, "with its only quiet factory past the capital, the king is {further} px further from the capital and {nearer} px nearer the factory");
    }

    #[test]
    fn the_ais_king_keeps_out_of_reach_of_big_guns_and_doesnt_walk_back_into_them() {
        // Henry, 2026-11-17, of a match he won early on an S map: "the enemy king didn't seem
        // to run away correctly". Four minutes in, two of his BATTLESHIPS (cannon, 130 px of
        // reach, in armour the AI's starter ships can't break) come for the AI's king, at
        // 7.5 px/s to its 37.
        //
        // As it was, the king backed away only from an armed enemy within 90 px, so it stood
        // still while it was shelled from 130; and with the enemy 90 px off again it flew
        // back to its post, which was where the enemy was. It was dead in 144 s. Now it keeps
        // clear of the reach of the gun that reaches furthest, it doesn't go back to a post
        // that's in reach, and once it has lost a quarter of itself it stops trusting that it
        // outweighs them. It's alive two and a half minutes on, with most of itself.
        let battleship = Design::from_line("BATTLESHIP|41,41|........p......../.......plp......./......phlhp....../.....phhlhhp...../....pnnnlnnnp..../...ppnnngnnnpp.../..phhnnnhnnnhhp../.gplhhhhphhhhlpg./.phnnnhhghhnnnhp./.phnnnhhghhnnnhp./gphnnnhhchhnnnhpg/.phhhhhnnnhhhhhp./.phhhhhnnnhhhhhp./gphhhhhnnnhhhhhpg/.phhhhhhghhhhhhp./.eeeeeeeeeeeeeee.|BUILT|").unwrap();
        assert!(battleship.check().problem().is_none());
        let mut deck = starter_deck();
        deck[6] = battleship.clone();
        let mut a = Arena::for_bots(1, [starter_deck(), deck], [Bot::Ai, Bot::None]);
        for _ in 0..15 * 240 {
            a.step();
        }
        let k = a.eco.as_ref().unwrap().king[0];
        let (kx, ky, full) = (a.ships[k].x, a.ships[k].y, a.ships[k].px_now());
        // (Its side has a station a long way off, 56 px that print nothing and can't move, so
        // that losing the outpost beside the king doesn't leave the side beaten: a beaten
        // side's king flies at a tenth of its speed, and is meant to be caught. Without it
        // this test asked a slowed king to outrun a battleship, once the day's change to
        // mining had left the AI a little less standing at four minutes.)
        a.spawn(0, 4, &pad_block(7, 7), 1100, 400, 0, false);
        let raiders: Vec<usize> = (0..2).map(|j| a.spawn(1, 6, &battleship, kx + 380 + 60 * j, ky - 200, 0, false)).collect();
        for &i in &raiders {
            a.ships[i].order = Order::Hunt;
        }
        // Once it's hurt that much, how many ticks it then spends in a battleship's reach with
        // the battleship no further off than the tick before: walking into the guns, or
        // standing in them.
        let reach = CANNON_RANGE + a.ships[raiders[0]].half() + a.ships[k].half();
        let near = |a: &Arena| raiders.iter().filter(|&&i| a.ships[i].alive).map(|&i| cheb(a.ships[i].x, a.ships[i].y, a.ships[k].x, a.ships[k].y)).min();
        let (mut stood, mut was) = (0, near(&a));
        for _ in 0..15 * 150 {
            a.step();
            let now = near(&a);
            if let (Some(n), Some(w)) = (now, was) {
                stood += (a.ships[k].alive && a.ships[k].px_now() * 4 < full * 3 && n <= reach && n <= w) as i32;
            }
            was = now;
        }
        println!("the king has {} px of {full}; hurt, it spent {stood} ticks in a battleship's reach without getting further off", a.ships[k].px_now());
        assert!(a.ships[k].alive, "the AI's king was killed in two and a half minutes by two ships a fifth as fast");
        assert!(a.ships[k].px_now() * 2 >= full, "the king has {} px left of {full}", a.ships[k].px_now());
        // (Three seconds, at first. A whole match moves with every change to the game, and
        // this one has run from 0 ticks to 47 since; with the faults it's here for the king
        // stood 250 and 330. The test after this one stages the fault itself.)
        assert!(stood <= 90, "hurt, the king spent {stood} ticks in reach of a battleship's cannon without getting further off");
    }

    #[test]
    fn a_hurt_king_doesnt_fly_to_its_post_through_the_guns_in_between() {
        // The king's post is safe, and a battleship stands between the king and it. A king
        // that has lost a quarter of itself keeps clear of every gun, so it waits where it is:
        // it doesn't fly to the post past the battleship. (As first built on 2026-11-17 it
        // asked only whether the post itself was safe, and flew through the cannon's reach to
        // get there: 125 px of king down to 33 in fifteen seconds, in the test above.)
        let battleship = Design::from_line("BATTLESHIP|41,41|........p......../.......plp......./......phlhp....../.....phhlhhp...../....pnnnlnnnp..../...ppnnngnnnpp.../..phhnnnhnnnhhp../.gplhhhhphhhhlpg./.phnnnhhghhnnnhp./.phnnnhhghhnnnhp./gphnnnhhchhnnnhpg/.phhhhhnnnhhhhhp./.phhhhhnnnhhhhhp./gphhhhhnnnhhhhhpg/.phhhhhhghhhhhhp./.eeeeeeeeeeeeeee.|BUILT|").unwrap();
        let mut deck = starter_deck();
        deck[6] = battleship.clone();
        let mut a = Arena::for_bots(1, [starter_deck(), deck], [Bot::Ai, Bot::None]);
        for _ in 0..15 * 60 {
            a.step();
        }
        let k = a.eco.as_ref().unwrap().king[0];
        let post = KING_STARTS[0];
        // The king 360 px up the map's edge from its post, hurt; the battleship parked half
        // way between, 190 px from each: the post is out of its guns' reach, and so is the king.
        let cells: Vec<usize> = (0..a.ships[k].hp.len()).filter(|&c| a.ships[k].hp[c] > 0 && !a.ships[k].shape.is_core(c)).collect();
        for c in cells {
            if a.ships[k].px_now() * 4 < a.ships[k].px_full() * 3 {
                break;
            }
            let at = a.ships[k].cell_xy(c);
            a.kill_cell(k, c, at, (0, 0));
        }
        assert!(a.ships[k].alive && a.ships[k].px_now() * 4 < a.ships[k].px_full() * 3);
        (a.ships[k].x, a.ships[k].y) = (post.0 - 100, post.1 - 360);
        let e = a.spawn(1, 6, &battleship, post.0 - 50, post.1 - 180, 0, false);
        (a.ships[e].order, a.ships[e].home) = (Order::Hold, (post.0 - 50, post.1 - 180));
        let (px, far) = (a.ships[k].px_now(), |a: &Arena| cheb(a.ships[k].x, a.ships[k].y, a.ships[e].x, a.ships[e].y));
        let (start, mut nearest) = (far(&a), i32::MAX);
        for _ in 0..15 * 20 {
            a.step();
            nearest = nearest.min(far(&a));
        }
        println!("it began {start} px from the battleship and was never nearer than {nearest}; {} px of {px} left", a.ships[k].px_now());
        assert!(nearest >= start - 12, "the hurt king began {start} px from the battleship and came within {nearest} px of it on its way to its post");
        assert_eq!(a.ships[k].px_now(), px, "it was shot on the way");
    }

    #[test]
    fn the_ai_prints_miners_while_there_is_rock_in_reach_for_them() {
        // The AI's outpost is planted and its bank is full. What its king prints first, with so
        // many miners of its own already: a miner while the rock its stations reach holds
        // `MINER_ROCK` pixels for each one more, and not once it doesn't.
        let first = |miners: i32| {
            let mut a = ai_alone();
            let k = a.eco.as_ref().unwrap().king[0];
            let (kx, ky) = (a.ships[k].x, a.ships[k].y);
            let f = friend(&mut a, OUTPOST, kx + 80, ky - 60);
            (a.ships[f].rooted, a.ships[f].node) = (true, true);
            for n in 0..miners {
                friend(&mut a, MINER, kx - 60 + 6 * (n % 20), ky + 40 + 6 * (n / 20));
            }
            a.ships[k].store = 2000;
            a.score[0].mined += 2000 - START_BANK;
            let room = a.rock_in_reach(0) / MINER_ROCK;
            think(&mut a);
            (first_ship_printed(&a), room)
        };
        let (printed, room) = first(3);
        assert!(room > 3, "only {room} miners' worth of rock in reach of the AI's home");
        // Three is more than the one for each factory and one more it kept until 2026-11-15.
        assert_eq!(printed, Some(MINER), "with rock for {room} miners and three working, the AI printed no miner");
        let (printed, _) = first(room as i32 + 1);
        assert_ne!(printed, Some(MINER), "with its rock's worth of miners, the AI printed another");
        // Switched off, it keeps the old count.
        let mut a = ai_alone();
        a.eco.as_mut().unwrap().off[0] = RULE_MINERS;
        let k = a.eco.as_ref().unwrap().king[0];
        let (kx, ky) = (a.ships[k].x, a.ships[k].y);
        for n in 0..3 {
            friend(&mut a, MINER, kx - 60 + 6 * n, ky + 40);
        }
        a.ships[k].store = 0;
        think(&mut a);
        assert_ne!(first_ship_printed(&a), Some(MINER));
    }

    #[test]
    fn money_nothing_is_spending_plants_another_factory() {
        // A strategy that wants one factory, with three planted and a bank that keeps filling:
        // after `PILE_TICKS` of that it plants a fourth, and with the rule off it doesn't.
        let planted = |off: u32| {
            let mut a = ai_alone();
            a.eco.as_mut().unwrap().off[0] = off | RULE_MINERS;
            a.eco.as_mut().unwrap().strategy[0] = Some(Strategy { plant: &[(OUTPOST, 0)], most: 1, lean: 0, prints: &[], keep: 0, first: 9999, wave: 9999, bold: false, raid: &[] });
            let k = a.eco.as_ref().unwrap().king[0];
            let (kx, ky) = (a.ships[k].x, a.ships[k].y);
            let f = friend(&mut a, OUTPOST, kx + 80, ky - 60);
            (a.ships[f].rooted, a.ships[f].node) = (true, true);
            for n in 0..2 {
                friend(&mut a, MINER, kx - 40 + 20 * n, ky + 50);
            }
            for _ in 0..PILE_TICKS + 30 {
                let gift = (1000 - a.bank(k)).max(0);
                a.ships[k].store += gift;
                a.score[0].mined += gift;
                a.step();
            }
            first_seed_sent_to(&a).is_some()
        };
        assert!(planted(0), "a full bank for {} s planted nothing past the one factory it wants", PILE_TICKS / 15);
        assert!(!planted(RULE_SPEND), "with the rule off, it planted past the one factory it wants");
    }

    #[test]
    fn the_ai_plants_as_soon_as_its_bank_holds_the_price() {
        // A bank of an outpost's price and not a pixel more, nobody near: whether the AI sends
        // its seed within a second. With `RULE_SPARE` off it waits for 40 px more, as it did
        // until 2026-11-15.
        let sent = |off: u32| {
            let mut a = ai_alone();
            a.eco.as_mut().unwrap().off[0] = off;
            let k = a.eco.as_ref().unwrap().king[0];
            a.ships[k].store = a.eco.as_ref().unwrap().cards[0][OUTPOST].cost;
            think(&mut a);
            first_seed_sent_to(&a).is_some()
        };
        assert!(sent(0), "with the outpost's price in the bank, the AI didn't plant");
        assert!(!sent(RULE_SPARE), "with the rule off, the AI planted with nothing to spare");
    }

    #[test]
    fn the_ai_arms_while_the_enemys_fleet_has_the_edge() {
        // The AI has an outpost and its two miners, and a bank too small for the factory it's
        // saving for. The enemy has so many fighters parked at its own home, raiding nobody.
        // What the AI's king is printing a second later.
        let printing = |fighters: i32| {
            let mut a = ai_alone();
            // Its miners as they were before 2026-11-15, so it's guns it's choosing between.
            a.eco.as_mut().unwrap().off[0] = RULE_MINERS;
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
        let was = cheb(a.ships[k].x, a.ships[k].y, KING_STARTS[0].0, KING_STARTS[0].1);
        for _ in 0..15 * 45 {
            a.step();
        }
        // (The one lancer it had asked for fights on at the enemy's king, which nobody plays.
        // Since each ship has a way of flying of its own, 2026-11-15, it kills that king inside
        // the 45 s, some six seconds after its own king is home: the match is the rusher's, or
        // still going, and its king is home either way.)
        let (hx, hy) = KING_STARTS[0];
        assert!(a.over.is_none() || a.over == Some(Some(0)), "the rusher lost: {:?}", a.over);
        // (Since lances lead their targets, 2026-11-16, the lancer kills that king before the
        // rusher's is home, and the match stops with it on its way.)
        let now = cheb(a.ships[k].x, a.ships[k].y, hx, hy);
        assert!(now <= 6 || (a.over == Some(Some(0)) && now < was), "the hurt king is at ({}, {}), not home ({was} px off when it was hurt)", a.ships[k].x, a.ships[k].y);
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
        let slab = Design::from_rows("SLAB", &rows);
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
        // on the same ticks into a match where no AI thinks give the same match: commands are
        // the only thing that drives one.
        let mut a = Arena::for_match(4, [starter_deck(), starter_deck()], [true, true]);
        while a.over.is_none() && a.tick < 15 * 300 {
            a.step();
        }
        let log = a.eco.as_ref().unwrap().log.clone();
        assert!(log.len() >= 6, "only {} commands in five minutes", log.len());
        // The replay's sides are AIs' sides too (a card's own order is an AI's to give, and is
        // dropped from a person's side), but nobody thinks for them: every command comes from
        // the log.
        let mut b = Arena::for_match(4, [starter_deck(), starter_deck()], [true, true]);
        for t in 0..2 {
            b.eco_mut().mind[t].next = u32::MAX;
        }
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

    // ----- Seeds sent into open space -----

    /// Puts pixels in a king's store as if it had made them, so the books still balance.
    fn fund(a: &mut Arena, t: usize, n: i32) {
        let k = a.eco.as_ref().unwrap().king[t];
        a.ships[k].store += n;
        a.eco.as_mut().unwrap().made[t] += n;
    }

    /// A spot in open space beside blue's king where its outpost (slot 4) may be sown.
    fn open_spot(a: &Arena) -> (i32, i32) {
        let (kx, ky) = KING_STARTS[0];
        [(100, 0), (0, -100), (70, -70), (120, -40), (40, -120)].iter().map(|d| (kx + d.0, ky + d.1)).find(|&p| a.sow_links(0, 4, p).is_ok()).expect("no open space in reach of blue's king")
    }

    fn rock_count(a: &Arena) -> usize {
        a.rock.iter().filter(|&&r| r).count()
    }

    fn living(a: &Arena, i: usize) -> i32 {
        a.ships[i].hp.iter().filter(|&&h| h > 0).count() as i32
    }

    #[test]
    fn a_seed_sent_into_space_grows_from_its_network_and_costs_its_price_again() {
        let mut a = quiet();
        let k = a.eco.as_ref().unwrap().king[0];
        let (price, count) = (a.eco.as_ref().unwrap().cards[0][4].price(), a.eco.as_ref().unwrap().cards[0][4].count);
        fund(&mut a, 0, 2 * price);
        let (at, rock, bank) = (open_spot(&a), rock_count(&a), a.bank(k));
        assert_eq!(a.can_sow(0, 4, at), Ok(price), "the seed costs what a seed for a rock costs");
        a.command(Cmd::Sow { team: 0, slot: 4, at });
        a.step();
        assert_eq!(a.bank(k), bank - price, "the seed is paid for in full when it's sent");
        assert_eq!(a.ships.iter().filter(|s| s.alive && s.team == 0 && s.role == Role::Seed && s.sow).count(), 1);
        // It flies there, comes down on the very spot, and grows with nothing under it.
        let mut landed = None;
        for _ in 0..15 * 240 {
            a.step();
            a.audit().unwrap();
            if let Some(f) = factory(&a) {
                landed.get_or_insert(a.tick);
                assert!(a.ships[f].rooted, "growing in space but not planted");
                if a.ships[f].grow.is_none() {
                    break;
                }
            }
        }
        let f = factory(&a).expect("the seed never came down");
        assert!(a.ships[f].grow.is_none(), "it never finished growing");
        assert_eq!(living(&a, f), count, "it isn't whole");
        let core = a.ships[f].cell_at(at.0, at.1).expect("its core isn't on the spot it was sent to");
        assert!(a.ships[f].shape.is_core(core));
        assert_eq!(rock_count(&a), rock, "it ate rock, in open space");
        // Every pixel but the seed's own core came out of the bank: the price, as good as twice.
        let e = a.eco.as_ref().unwrap();
        assert_eq!(e.spent[0], price + count - 1);
        assert_eq!(a.bank(k), bank + (e.made[0] - 2 * price) - price - (count - 1));
        assert_eq!(e.tally[0].planted, 1);
    }

    #[test]
    fn a_factory_whose_rock_runs_out_finishes_from_the_bank() {
        let mut a = quiet();
        let (price, count) = (a.eco.as_ref().unwrap().cards[0][4].price(), a.eco.as_ref().unwrap().cards[0][4].count);
        fund(&mut a, 0, 2 * price);
        let (_, target) = home_rock(&a);
        let blob = a.rock_blob(target);
        a.command(Cmd::Print { team: 0, slot: 4, target: Some(target) });
        while factory(&a).is_none() {
            a.step();
            assert!(a.tick < 15 * 120, "the seed never landed");
        }
        // The rock is mined away from under it, all but thirty pixels.
        let left: Vec<(i32, i32)> = blob.iter().copied().filter(|&(x, y)| a.rock_at(x, y)).collect();
        for &(x, y) in &left[30..] {
            a.rock[(y * a.w + x) as usize] = false;
        }
        let f = factory(&a).unwrap();
        let had = living(&a, f);
        for _ in 0..15 * 240 {
            a.step();
            a.audit().unwrap();
            if a.ships[f].grow.is_none() {
                break;
            }
        }
        assert_eq!(living(&a, f), count, "it stopped short when its rock ran out");
        // It ate the thirty that were left, and its network paid for the rest.
        assert_eq!(blob.iter().filter(|&&(x, y)| a.rock_at(x, y)).count(), 0);
        assert_eq!(a.eco.as_ref().unwrap().spent[0], price + (count - had - 30));
    }

    #[test]
    fn something_with_no_factory_pixels_has_no_network_and_stops_when_its_rock_does() {
        // A seeded warship isn't a node. On a rock too small for it, it grows what the rock holds
        // and flies off as it is, as it always did: there's nothing to feed it.
        let mut blue = starter_deck();
        let mut d = blue[0].clone();
        d.name = "SPROUT".into();
        d.seeded = true;
        assert!(d.stats().factory == 0 && d.check().ok() && d.stats().count > 6);
        blue[4] = d;
        let mut a = Arena::for_match(1, [blue, starter_deck()], [false, false]);
        fund(&mut a, 0, 500);
        let k = a.eco.as_ref().unwrap().king[0];
        let (_, target) = home_rock(&a);
        // Into open space it can't go at all.
        let (kx, ky) = KING_STARTS[0];
        assert!(a.can_sow(0, 4, (kx + 100, ky)).unwrap_err().contains("NO FACTORY PIXELS"));
        // Its rock is mined down to four pixels: the one it's sent to and three more.
        let blob = a.rock_blob(target);
        for &(x, y) in blob.iter().filter(|&&p| p != target).skip(3) {
            a.rock[(y * a.w + x) as usize] = false;
        }
        a.command(Cmd::Print { team: 0, slot: 4, target: Some(target) });
        let sprout = |a: &Arena| (0..a.ships.len()).find(|&i| a.ships[i].alive && a.ships[i].role == Role::Ship && a.ships[i].name == "SPROUT");
        for _ in 0..15 * 120 {
            a.step();
            a.audit().unwrap();
        }
        let i = sprout(&a).expect("it never landed");
        let bank = a.bank(k);
        assert!(a.ships[i].grow.is_none(), "it's still waiting to be fed");
        // The seed's own core, and a pixel for each of the four the rock held.
        assert!(living(&a, i) <= 5, "it grew {} px on a rock of four", living(&a, i));
        for _ in 0..15 * 20 {
            a.step();
        }
        assert!(a.bank(k) >= bank, "it took pixels from a network it isn't in");
    }

    #[test]
    fn a_factory_growing_from_the_bank_waits_when_its_empty_and_shares_with_the_king() {
        let mut a = quiet();
        let k = a.eco.as_ref().unwrap().king[0];
        let (price, count) = (a.eco.as_ref().unwrap().cards[0][4].price(), a.eco.as_ref().unwrap().cards[0][4].count);
        // Just the seed's price in the bank: it lands and has nothing to grow with.
        let short = a.bank(k) - price;
        a.ships[k].store -= short;
        a.eco.as_mut().unwrap().made[0] -= short;
        let at = open_spot(&a);
        a.command(Cmd::Sow { team: 0, slot: 4, at });
        while factory(&a).is_none() {
            a.step();
            assert!(a.tick < 15 * 120, "the seed never landed");
        }
        let f = factory(&a).unwrap();
        for _ in 0..15 * 30 {
            a.step();
            a.audit().unwrap();
        }
        // It has grown only what the king has made since, a pixel every four seconds.
        let made = a.eco.as_ref().unwrap().made[0] + short;
        assert!(a.ships[f].grow.is_some() && living(&a, f) <= 1 + made, "it grew {} px on {made} px of income", living(&a, f));
        assert_eq!(a.bank(k), 0, "it's waiting with pixels in the bank");
        // With the king printing too and pixels coming in slower than the two could use them,
        // they take turns: the factory, which is much the hungrier, doesn't starve the king.
        a.command(Cmd::Print { team: 0, slot: 2, target: None });
        let had = living(&a, f);
        for tick in 0..15 * 20 {
            if tick % 5 == 0 {
                fund(&mut a, 0, 1);
            }
            a.step();
            a.audit().unwrap();
        }
        let job = a.eco.as_ref().unwrap().queue[0][0];
        assert!(living(&a, f) - had >= 30 && job.paid >= 8, "one starved the other: the factory grew {} px and the king's job has {} px", living(&a, f) - had, job.paid);
        // Fed, it finishes.
        fund(&mut a, 0, 2 * count);
        for _ in 0..15 * 120 {
            a.step();
            if a.ships[f].grow.is_none() {
                break;
            }
        }
        assert_eq!(living(&a, f), count);
    }

    #[test]
    fn a_seed_goes_into_space_only_where_it_fits_and_its_network_reaches() {
        let mut a = quiet();
        let k = a.eco.as_ref().unwrap().king[0];
        fund(&mut a, 0, 1000);
        let (kx, ky) = KING_STARTS[0];
        let why = |a: &Arena, slot: usize, at: (i32, i32)| a.can_sow(0, slot, at).unwrap_err();
        // On rock, or with any of it over rock: that's what planting on a rock is for.
        let (_, on_rock) = home_rock(&a);
        assert!(why(&a, 4, on_rock).contains("ROCK UNDER IT"));
        // Out past where a node of its own could link to it.
        let far = (kx + KING_RANGE + NODE_RANGE + 40, ky - 250);
        assert!(!a.rock_at(far.0, far.1));
        assert!(why(&a, 4, far).contains("TOO FAR"), "{}", why(&a, 4, far));
        // Hanging off the map.
        assert!(why(&a, 4, (2, ky)).contains("EDGE"));
        // A design that isn't seeded, and a card that isn't there.
        assert!(why(&a, 0, (kx + 100, ky)).contains("ONLY A SEEDED"));
        assert!(a.can_sow(0, 40, (kx + 100, ky)).is_err());
        // A command that isn't allowed is dropped, and the bank isn't touched.
        let bank = a.bank(k);
        a.command(Cmd::Sow { team: 0, slot: 4, at: far });
        a.command(Cmd::Sow { team: 0, slot: 0, at: (kx + 100, ky) });
        a.step();
        assert!(a.bank(k) >= bank, "a refused seed was paid for");
        assert_eq!(a.eco.as_ref().unwrap().spent[0], 0);
        assert_eq!(a.ships.iter().filter(|s| s.alive && s.role == Role::Seed).count(), 0);
        // One that's allowed, and while its seed is still flying a second can be sent on past it,
        // out of the king's own reach: the first counts as the node it will be.
        let at = open_spot(&a);
        let beyond = [(150, 0), (0, -150), (105, -105)].iter().map(|d| (at.0 + d.0, at.1 + d.1)).find(|&p| a.sow_links(0, 4, p).is_err() && !a.rock_at(p.0, p.1)).expect("everywhere near is in reach already");
        assert!(why(&a, 4, beyond).contains("TOO FAR"), "{}", why(&a, 4, beyond));
        a.command(Cmd::Sow { team: 0, slot: 4, at });
        a.step();
        let first = a.sow_links(0, 4, beyond).expect("a seed on its way wasn't counted as the node it will be");
        assert_eq!(first.to.len(), 1);
        // Once it's down, nothing else goes where it stands or where it will grow.
        while factory(&a).is_none() {
            a.step();
            assert!(a.tick < 15 * 120, "the seed never landed");
        }
        assert!(why(&a, 4, at).contains("IN THE WAY"));
        assert!(why(&a, 4, (at.0 + 5, at.1 + 5)).contains("IN THE WAY"), "what it has still to grow should count");
        a.audit().unwrap();
    }

    #[test]
    fn a_seed_for_open_space_crosses_the_wire() {
        let c = Cmd::Sow { team: 1, slot: 4, at: (700, 312) };
        assert_eq!(c.to_wire(), "W 1 4 700 312");
        assert_eq!(Cmd::from_wire(&c.to_wire()), Some(c));
        assert_eq!(c.team(), 1);
        for bad in ["W 1 4 700", "W 2 4 700 312", "W 1 64 700 312", "W 1 4 700 9999999", "W 1 4 700 312 5", "W 1 4 x 312"] {
            assert_eq!(Cmd::from_wire(bad), None, "{bad}");
        }
    }

    #[test]
    fn the_ai_sends_a_seed_into_space_only_when_no_rock_will_take_it() {
        let sown = |a: &Arena| a.eco.as_ref().unwrap().log.iter().find_map(|(_, c)| match c {
            Cmd::Sow { team: 0, slot, at } => Some((*slot, *at)),
            _ => None,
        });
        // With rocks to hand it plants on a rock, as it always has.
        let mut a = ai_alone();
        for _ in 0..15 * 60 {
            a.step();
        }
        assert!(first_seed_sent_to(&a).is_some(), "the test needs the AI to have planted");
        assert_eq!(sown(&a), None, "it sent a seed into space with rocks to hand");
        // Every rock gone and pixels to spend: its next factory goes into open space, in reach
        // of its network, and grows there from its bank.
        for r in a.rock.iter_mut() {
            *r = false;
        }
        let nodes = |a: &Arena| a.ships.iter().filter(|s| s.alive && s.team == 0 && s.role == Role::Ship && s.node && s.rooted).count();
        let had = nodes(&a);
        for tick in 0..15 * 240 {
            if tick % 15 == 0 {
                fund(&mut a, 0, 30);
            }
            a.step();
            a.audit().unwrap();
            if nodes(&a) > had && a.ships.iter().all(|s| !(s.alive && s.team == 0 && s.rooted && s.grow.is_some())) {
                break;
            }
        }
        let (slot, at) = sown(&a).expect("with no rock left it never sent a seed into open space");
        assert!(a.eco.as_ref().unwrap().cards[0][slot].st.factory > 0);
        assert!(nodes(&a) > had, "the seed it sent to {at:?} never grew into a factory");
        // The scripted openings never do: they're the yardstick, and stay as they were.
        let mut a = Arena::for_bots(1, [starter_deck(), starter_deck()], [Bot::Expander, Bot::None]);
        for _ in 0..15 * 60 {
            a.step();
        }
        for r in a.rock.iter_mut() {
            *r = false;
        }
        for tick in 0..15 * 120 {
            if tick % 15 == 0 {
                fund(&mut a, 0, 30);
            }
            a.step();
        }
        assert_eq!(sown(&a), None, "a scripted opening sent a seed into space");
    }
}
