//! Load tests (M3c): a made-up world as big as a full server's, busy with looping monsters and
//! with its players played by bots, to measure what a server can simulate and draw.
//!
//! Each copy's Pinewater has monsters looping the whole fabric chain, as many as one chain takes
//! without crowding (five: ten starting in a crowd jam, see `tangle_tests.rs`). The rest are
//! shuttlers, spread over the whole world, each pushing a log back and forth between two spots
//! a few screens' tiles apart: the same planning, pushing and walking a real loop does, without
//! the whole server's monsters queuing at seven cotton plants.

use crate::client::Connection;
use crate::data::Data;
use crate::econ::{ItemData, RawMaterial};
use crate::framebuffer::{FB_H, FB_W};
use crate::game::{Game, Goal, Input, Monster, Routine, Thing, NAMES};
use crate::net::{self, ToPlayer};
use crate::world::{Dir, Item, Tile, World, DIRS};
use std::collections::VecDeque;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::time::{Duration, Instant};

/// How busy a made-up world is.
#[derive(Clone, Copy, Debug)]
pub struct Busy {
    /// Copies of the whole world, 14 screens each (see `World::made_up`).
    pub copies: usize,
    /// Communes, nobody's yet, spread over the copies. Each has its first two monsters.
    pub players: usize,
    /// Monsters looping the fabric chain in each copy's Pinewater.
    pub chains: usize,
    /// Monsters shuttling a log between two spots, spread over the whole world.
    pub shuttlers: usize,
}

/// A full server: 98 screens, 50 players, five on each town's fabric chain and 400 shuttlers,
/// 535 monsters in all.
pub const FULL: Busy = Busy { copies: 7, players: 50, chains: 5, shuttlers: 400 };

/// The whole fabric chain as Henry recorded it, in the Pinewater whose northwest corner is
/// `(ox, oy)`: harvest and bin two bolls, two bolls from the chute through the workbench, the
/// slivers to the spinnery, the thread to the loom, the fabric to the cloth store.
pub fn fabric_chain(g: &Game, (ox, oy): (i32, i32)) -> Routine {
    let at = |(x, y): (i32, i32)| (x + ox, y + oy);
    let mv = |good: &str, from, to| Goal::Move { good: good.into(), from: at(from), to: at(to) };
    let work = |spot| {
        let (x, y) = at(spot);
        Goal::Activate(Thing::Shop(g.world.shop_at_spot(x, y).expect("a work spot")))
    };
    let goals = vec![
        Goal::Activate(Thing::Plant { at: at((8, 8)), facing: Dir::Right }),
        mv("cotton_bolls", (9, 8), (12, 8)),
        mv("cotton_bolls", (9, 8), (12, 8)),
        mv("cotton_bolls", (12, 9), (13, 9)),
        work((14, 9)),
        mv("cotton_bolls", (12, 9), (13, 9)),
        work((14, 9)),
        mv("cotton_slivers", (15, 9), (30, 9)),
        work((30, 8)),
        mv("thread", (31, 9), (44, 21)),
        work((43, 21)),
        mv("fabric", (44, 22), (44, 28)),
    ];
    Routine { flag: at((8, 10)), goals }
}

/// A made-up world as busy as `b` says, every monster's loop already running. Commune `p` lives
/// in copy `p % copies`, its first two monsters by that copy's monster starts. The loopers are
/// handed round the communes in turn.
pub fn busy_world(b: Busy) -> Game {
    let data = Data::load();
    let world = World::made_up(&data, b.copies);
    let rh = world.h / b.copies as i32;
    let mut g = Game::on(data, world);
    for p in 0..b.players {
        let k = if p == 0 { 0 } else { g.join() };
        let oy = (p % b.copies) as i32 * rh;
        let firsts: Vec<usize> = (0..g.monsters.len()).filter(|&i| g.monsters[i].commune == k).collect();
        for (n, i) in firsts.into_iter().enumerate() {
            let (sx, sy) = g.world.starts[n];
            (g.monsters[i].x, g.monsters[i].y) = g.free_tile_near((sx, sy + oy));
        }
    }
    let mut next = 0;
    let mut commune = || {
        next += 1;
        (next - 1) % b.players.max(1)
    };
    for copy in 0..b.copies as i32 {
        let chain = fabric_chain(&g, (0, copy * rh));
        // In a line along the road west of the flag, as five copies set off fine.
        for n in 0..b.chains as i32 {
            let at = g.free_tile_near((chain.flag.0 - 2 - n, chain.flag.1));
            hire(&mut g, commune(), at, chain.clone());
        }
    }
    let mut seed = 0x5EED_u64;
    let mut placed = 0;
    while placed < b.shuttlers {
        seed = seed.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
        let (x, y) = ((seed >> 33) as i32 % g.world.w, (seed >> 13) as i32 % g.world.h);
        let Some(there) = open(&g, (x, y)).then(|| far_spot(&g, (x, y), 8 + (seed % 12) as usize)).flatten() else { continue };
        g.world.items.push(Item::new(x, y, log(&g.data)));
        let goals = vec![Goal::Move { good: "hemlock_log".into(), from: (x, y), to: there }, Goal::Move { good: "hemlock_log".into(), from: there, to: (x, y) }];
        let flag = g.free_tile_near((x, y + 1));
        hire(&mut g, commune(), flag, Routine { flag, goals });
        placed += 1;
    }
    g
}

/// A new monster in commune `k`, standing at `at` and looping `routine`.
fn hire(g: &mut Game, k: usize, at: (i32, i32), routine: Routine) {
    let skills = g.monsters[0].skills.clone();
    let mut m = Monster::new(g.next_id, k, NAMES[g.next_id as usize % NAMES.len()], "goblin", &g.data, at, skills);
    (m.routine, m.pc, m.playing, m.bubble) = (Some(routine), 0, true, None);
    g.monsters.push(m);
    g.next_id += 1;
}

/// Open ground where a log can lie for good: grass, with nothing on it or beside it but grass,
/// so it's never in a doorway, on a road or by a workshop.
fn open(g: &Game, (x, y): (i32, i32)) -> bool {
    let clear = |(x, y): (i32, i32)| g.world.tile(x, y) == Tile::Grass && g.world.item_at(x, y).is_none() && g.monster_at(x, y).is_none() && g.world.plant_at(x, y).is_none();
    (-1..=1).all(|dy| (-1..=1).all(|dx| clear((x + dx, y + dy))))
}

/// An open spot `steps` steps' walk from `from`, if there is one.
fn far_spot(g: &Game, from: (i32, i32), steps: usize) -> Option<(i32, i32)> {
    let mut seen = std::collections::HashMap::from([(from, 0)]);
    let mut queue = VecDeque::from([from]);
    while let Some(at) = queue.pop_front() {
        let d = seen[&at];
        if d == steps {
            if open(g, at) {
                return Some(at);
            }
            continue;
        }
        for dir in DIRS {
            let (dx, dy) = dir.delta();
            let next = (at.0 + dx, at.1 + dy);
            if g.world.item_floor(next.0, next.1) && !seen.contains_key(&next) {
                seen.insert(next, d + 1);
                queue.push_back(next);
            }
        }
    }
    None
}

/// A hemlock log, as gathered.
fn log(data: &Data) -> ItemData {
    let gt = data.good("hemlock_log");
    let base = gt.raw_material_base_value.unwrap_or(1.0);
    let raw = vec![RawMaterial { good: gt.id.clone(), base_value: base, density: gt.raw_material_density.unwrap_or(1.0), spent: false }];
    ItemData { good: gt.id.clone(), quality: 0.5, carried_tags: Vec::new(), raw, depth: 0, value: base, shares: Vec::new(), wear: 0, haul: Vec::new(), carted: None }
}

/// A player played by a bot: it wanders, stopping now and then to use or examine what's in
/// front of it, or to switch monsters. It never records or plays loops.
pub struct Bot {
    seed: u64,
    /// What it's doing, and for how many more ticks.
    doing: Doing,
    left: u32,
}

#[derive(Clone, Copy, PartialEq)]
enum Doing {
    Walk(usize),
    Stand,
    Examine,
}

impl Bot {
    pub fn new(n: u64) -> Bot {
        Bot { seed: n.wrapping_add(1).wrapping_mul(0x9E37_79B9_7F4A_7C15), doing: Doing::Stand, left: 0 }
    }

    fn rand(&mut self, n: u32) -> u32 {
        self.seed ^= self.seed << 13;
        self.seed ^= self.seed >> 7;
        self.seed ^= self.seed << 17;
        (self.seed % n as u64) as u32
    }

    /// Its keys this tick.
    pub fn keys(&mut self) -> Input {
        let mut i = Input::default();
        if self.left == 0 {
            let before = self.doing;
            (self.doing, self.left) = match self.rand(100) {
                0..=64 => (Doing::Walk(self.rand(4) as usize), 5 + self.rand(40)),
                65..=84 => (Doing::Stand, 5 + self.rand(30)),
                85..=94 => {
                    i.use_pressed = true;
                    (Doing::Stand, 3)
                }
                95..=97 => (Doing::Examine, 15 + self.rand(30)),
                _ => {
                    i.tab_pressed = true;
                    (Doing::Stand, 3)
                }
            };
            if let Doing::Walk(d) = self.doing {
                i.pressed[d] = before != self.doing;
            }
        }
        self.left -= 1;
        match self.doing {
            Doing::Walk(d) => i.held[d] = true,
            Doing::Examine => i.inspect = true,
            Doing::Stand => {}
        }
        i
    }
}

/// A frame bigger than this is most of a new screen (a whole screen is about 34 KB).
const FLIP_BYTES: usize = 10_000;

/// What bots playing on a server have seen, all of them together.
#[derive(Default)]
pub struct Seen {
    pub playing: AtomicUsize,
    pub frames: AtomicUsize,
    pub bytes: AtomicUsize,
    /// Frames that were most of a new screen, and their bytes.
    pub flips: AtomicUsize,
    pub flip_bytes: AtomicUsize,
    /// Frames whose pictures didn't fit the screen they were for.
    pub garbled: AtomicUsize,
}

/// Bot `n` playing on the server at `addr` until `stop` is set or the server lets it go: keys out
/// every tick it has new ones, pictures in, checked against its screen.
pub fn play_bot(addr: &str, n: u64, seen: &Seen, stop: &AtomicBool) -> std::io::Result<()> {
    let mut conn = Connection::connect(addr, &format!("bot-{n}"), Duration::ZERO)?;
    let mut bot = Bot::new(n);
    let mut screen = vec![0u8; FB_W * FB_H];
    seen.playing.fetch_add(1, Ordering::Relaxed);
    let tick = Duration::from_micros(66_667);
    let mut next = Instant::now();
    let result = 'playing: loop {
        if stop.load(Ordering::Relaxed) {
            break Ok(());
        }
        if let Err(e) = conn.keys(&bot.keys()) {
            break Err(e);
        }
        for m in conn.messages() {
            match m {
                Ok(ToPlayer::Frame { pixels, .. }) => {
                    seen.frames.fetch_add(1, Ordering::Relaxed);
                    seen.bytes.fetch_add(pixels.len(), Ordering::Relaxed);
                    if pixels.len() > FLIP_BYTES {
                        seen.flips.fetch_add(1, Ordering::Relaxed);
                        seen.flip_bytes.fetch_add(pixels.len(), Ordering::Relaxed);
                    }
                    if net::apply_delta(&mut screen, &pixels).is_err() {
                        seen.garbled.fetch_add(1, Ordering::Relaxed);
                    }
                }
                Ok(ToPlayer::Welcome { .. }) => {}
                Ok(ToPlayer::Bye(_)) => break 'playing Ok(()),
                Err(e) => break 'playing Err(e),
            }
        }
        next += tick;
        std::thread::sleep(next.saturating_duration_since(Instant::now()));
    };
    seen.playing.fetch_sub(1, Ordering::Relaxed);
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Mean and worst of some durations, in milliseconds.
    fn ms(d: &[Duration]) -> (f64, f64) {
        let total: Duration = d.iter().sum();
        (total.as_secs_f64() * 1000.0 / d.len().max(1) as f64, d.iter().max().map_or(0.0, |m| m.as_secs_f64() * 1000.0))
    }

    #[test]
    fn a_busy_world_is_built_as_asked() {
        let g = busy_world(Busy { copies: 3, players: 4, chains: 2, shuttlers: 10 });
        assert_eq!(g.communes.iter().filter(|c| !c.town).count(), 4, "and the town's own");
        assert_eq!(g.monsters.iter().filter(|m| m.worker.is_none()).count(), 4 * 2 + 3 * 2 + 10);
        let one = crate::world::World::new(&g.data);
        assert_eq!(g.monsters.iter().filter(|m| m.worker.is_some()).count(), 3 * one.workers.len(), "the town's working folk in every copy");
        let one = one.h;
        assert_eq!(g.world.h, 3 * one);
        for (p, c) in g.communes.iter().filter(|c| !c.town).enumerate() {
            assert_eq!(g.monsters[c.controlled].y / one, (p % 3) as i32, "player {p} lives in copy {}", p % 3);
        }
        assert_eq!(g.monsters.iter().filter(|m| m.playing && m.worker.is_none()).count(), 3 * 2 + 10);
        assert_eq!(g.world.items.iter().filter(|it| it.good() == "hemlock_log").count(), 10);
    }

    /// `ticks` of a busy world with every player a bot: how long each tick took.
    fn run(b: Busy, ticks: u32) -> (Game, Vec<Duration>) {
        let mut g = busy_world(b);
        let mut bots: Vec<Bot> = (0..b.players as u64).map(Bot::new).collect();
        let mut took = Vec::new();
        for _ in 0..ticks {
            let inputs: Vec<Input> = bots.iter_mut().map(|b| b.keys()).collect();
            let t = Instant::now();
            g.update(&inputs);
            took.push(t.elapsed());
        }
        (g, took)
    }

    /// Prints how long ticks, and drawing the player's screen with their roster strip showing,
    /// take for one player with a huge commune: `cargo test --release -- --ignored --nocapture
    /// a_huge_commune`.
    #[test]
    #[ignore]
    fn a_huge_commune() {
        let b = Busy { copies: 7, players: 1, chains: 5, shuttlers: 600 };
        let (mut g, took) = run(b, 900);
        let (mean, worst) = ms(&took[450..]);
        let (sp, mut fb) = (crate::sprites::Sprites::new(), crate::framebuffer::Framebuffer::new());
        g.communes[0].roster = true;
        let drawn: Vec<Duration> = (0..100)
            .map(|_| {
                let t = Instant::now();
                crate::render::draw(&g, 0, &sp, &mut fb);
                t.elapsed()
            })
            .collect();
        let (draw, _) = ms(&drawn);
        println!("one commune of {} monsters: ticks {mean:.2} ms on average, {worst:.1} ms at worst; its screen with the roster strip {draw:.2} ms to draw", g.monsters.len());
    }

    /// Prints how long the world's ticks take as a full server's made-up world gets busier:
    /// `cargo test --release -- --ignored --nocapture ticks_as_the_world_fills`
    /// (`SHUTTLERS=0,400` picks how many shuttlers). A tick has 66 ms, and drawing players'
    /// screens comes on top.
    #[test]
    #[ignore]
    fn ticks_as_the_world_fills() {
        let counts: Vec<usize> = std::env::var("SHUTTLERS").map_or_else(|_| vec![0, 200, 400, 800, 1600], |s| s.split(',').filter_map(|n| n.parse().ok()).collect());
        for shuttlers in counts {
            let b = Busy { shuttlers, ..FULL };
            let (g, took) = run(b, 900);
            let (mean, worst) = ms(&took[450..]);
            let done: u32 = g.monsters.iter().filter(|m| m.playing).map(|m| m.steps).sum();
            let stuck = g.monsters.iter().filter(|m| m.playing && m.steps == 0).count();
            println!(
                "{} monsters ({shuttlers} shuttlers): the last half minute's ticks {mean:.2} ms on average, {worst:.1} ms at worst; {done} goals done, {stuck} loops never finished one, {} fabric",
                g.monsters.len(),
                g.cloth_stored()
            );
        }
    }
}
