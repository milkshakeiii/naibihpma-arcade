//! The proving ground's arena. Ships are built from parts and fly with integer inertia. Each one
//! has a three-layer brain:
//! - the order picks where to go,
//! - the parts fire on reflex,
//! - mass and thrust decide how to fly.
//! Nothing here knows what kind of ship it is flying.

use super::design::*;
use crate::geom::{cheb, dist2, Line};
use crate::rng::Rng;

mod audit;
mod eco;
mod space;
pub use eco::*;

pub const AW: i32 = 480;
pub const AH: i32 = 360;
pub const TIME_LIMIT: u32 = 15 * 180;
/// Once something has been shot off, a fight where for this long nobody fires, nothing is shot off
/// and no armed ship gets any closer to an enemy is a stalemate: a draw.
pub const STALEMATE: u32 = 15 * 20;
/// Each team's home station: where hauls are delivered and the hurt run to.
pub const STATIONS: [(i32, i32); 2] = [(10, AH / 2), (AW - 11, AH / 2)];
const FLAK_SPEED: i32 = 6;
const LANCE_SPEED: i32 = 4;
const SHELL_SPEED: i32 = 4;
/// A ship on GUARD fights what comes this near the place it was built: further than any gun
/// reaches, so nothing can shell the factory beside it unanswered.
pub const GUARD_RADIUS: i32 = 160;
/// A ship with nothing to do waits at its post: its own place in the rings round what built it.
/// The first ring is this far outside the builder, clear of whatever it prints next; the rings,
/// and the posts along each ring, are about `POST_SPACE` apart.
pub const POST_GAP: i32 = 16;
pub const POST_SPACE: i32 = 16;
/// A ship that arrives beside its king this near posts other ships already keep joins theirs.
pub const POST_JOIN: i32 = 24;
/// How far from its post a waiting ship will look for a clear place.
pub const POST_SEARCH: i32 = 64;
/// A ship on ESCORT fights what comes this near its king.
pub const ESCORT_RADIUS: i32 = 110;
/// Ticks ahead an escort aims: where its place round the king will be by then.
const ESCORT_LEAD: i32 = 15;
/// Ticks after a hit during which an unarmed ship keeps fleeing.
const SCARED: u32 = 30;
/// Extra distance (px) a flak turret counts for each shot already on its way to a target, so a
/// ship's guns spread over several small targets instead of piling onto one.
const SPREAD: i32 = 12;
/// Ticks per 1/64 turn of a cannon turret: about 42 degrees a second.
const TURRET_TURN: u32 = 2;
/// Length of a drawn cannon barrel from the turret's centre.
pub const BARREL: i32 = 4;

pub fn isqrt(n: i32) -> i32 {
    if n < 2 {
        return n.max(0);
    }
    let (mut x, mut y) = (n, (n + 1) / 2);
    while y < x {
        x = y;
        y = (x + n / x) / 2;
    }
    x
}

/// A vector of length `speed` from (x, y) toward (tx, ty).
fn toward(x: i32, y: i32, tx: i32, ty: i32, speed: i32) -> (i32, i32) {
    let (dx, dy) = (tx - x, ty - y);
    let d = isqrt(dx * dx + dy * dy);
    if d == 0 {
        (0, 0)
    } else {
        (dx * speed / d, dy * speed / d)
    }
}

/// The facing (0 N, 1 E, 2 S, 3 W) nearest a direction.
fn axis_face(dx: i32, dy: i32) -> Option<u8> {
    if dx == 0 && dy == 0 {
        None
    } else if dx.abs() >= dy.abs() {
        Some(if dx > 0 { 1 } else { 3 })
    } else {
        Some(if dy > 0 { 2 } else { 0 })
    }
}

pub fn facing_vec(f: u8) -> (i32, i32) {
    [(0, -1), (1, 0), (0, 1), (-1, 0)][(f & 3) as usize]
}

/// sin of 0..=90 degrees in 16 steps, times 1024.
const SIN16: [i32; 17] = [0, 100, 200, 297, 392, 483, 569, 650, 724, 792, 851, 903, 946, 980, 1004, 1019, 1024];

fn sin64(a: i32) -> i32 {
    let a = a.rem_euclid(64);
    match a / 16 {
        0 => SIN16[a as usize],
        1 => SIN16[(32 - a) as usize],
        2 => -SIN16[(a - 32) as usize],
        _ => -SIN16[(64 - a) as usize],
    }
}

/// The unit vector (times 1024) of a turret angle in 64ths of a turn: 0 is north, clockwise.
pub fn dir64(a: i32) -> (i32, i32) {
    (sin64(a), -sin64(a + 16))
}

/// The turret angle nearest a direction.
fn angle_of(dx: i32, dy: i32) -> i32 {
    (0..64).max_by_key(|&a| {
        let (ux, uy) = dir64(a);
        dx * ux + dy * uy
    })
    .unwrap_or(0)
}

/// How far apart two turret angles are, in 64ths, signed the short way round.
fn angle_diff(from: i32, to: i32) -> i32 {
    let d = (to - from).rem_euclid(64);
    if d > 32 {
        d - 64
    } else {
        d
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Doing {
    Idle,
    LineUp,
    RunIn,
    RunOut,
    StandOff,
    Flee,
    Mine,
    Haul,
    Guard,
    Hold,
    Escort,
    WithKing,
    ToRock,
}

impl Doing {
    pub fn name(self) -> &'static str {
        match self {
            Doing::Idle => "IDLE",
            Doing::LineUp => "LINING UP",
            Doing::RunIn => "ATTACK RUN IN",
            Doing::RunOut => "ATTACK RUN OUT",
            Doing::StandOff => "HOLDING AT RANGE",
            Doing::Flee => "FLEEING HOME",
            Doing::Mine => "MINING",
            Doing::Haul => "HAULING HOME",
            Doing::Guard => "ON GUARD",
            Doing::Hold => "HOLDING",
            Doing::Escort => "NO GUNS: STAYING WITH THE GROUP",
            Doing::WithKing => "ESCORTING THE KING",
            Doing::ToRock => "FLYING TO ITS ROCK",
        }
    }
}

/// A cannon: a 3x3 block whose centre pixel is the breech. It works while that pixel survives.
#[derive(Clone, Debug)]
pub struct Turret {
    pub cell: usize,
    /// Where the barrel points, in 64ths of a turn (0 north, clockwise).
    pub aim: i32,
    cool: i32,
    turn_t: u32,
    aim_off: (i32, i32),
    aim_for: Option<usize>,
}

#[derive(Clone, Debug)]
pub struct Ship {
    pub team: usize,
    pub slot: usize,
    pub name: String,
    pub order: Order,
    pub shape: Shape,
    /// Stats from the pixels still alive, so shooting off engines slows a ship.
    pub st: Stats,
    pub x: i32,
    pub y: i32,
    /// Sub-pixel remainders (quarter pixels) for the integer DDA; positions stay whole pixels.
    sx: i32,
    sy: i32,
    /// Velocity in quarter pixels per tick.
    pub vx: i32,
    pub vy: i32,
    acc: i32,
    pub facing: u8,
    turn_t: i32,
    /// 1 while a pixel survives, 0 once it's gone.
    pub hp: Vec<u8>,
    cool: Vec<i32>,
    pub turrets: Vec<Turret>,
    pub alive: bool,
    pub doing: Doing,
    outbound: bool,
    run_t: i32,
    /// Where it was built: the middle of the factory that printed it, or of the king it arrived
    /// beside. For a ship nothing built, where it started.
    pub home: (i32, i32),
    /// How far from `home` its first ring of posts is, before its own half-size: the builder's
    /// half-size and `POST_GAP`. 0 for a ship nothing built: its post is `home` itself.
    pub ring0: i32,
    /// It spent this tick waiting at its post, or on its way there.
    pub posted: bool,
    pub carry: i32,
    hauling: bool,
    tpx: Option<(i32, i32)>,
    chew: i32,
    pub last_hit: u32,
    pub target: Option<usize>,
    pub goal: Option<(i32, i32)>,
    pub trail: Vec<(i32, i32)>,
    dirty: bool,
    /// The last move ran into a bigger enemy.
    bumped: bool,
    /// Where the ship was, and which way it faced, at the start of this tick: shots are tested
    /// against the path they take relative to the ship.
    pose0: (i32, i32, u8),
    /// The target pixel (relative to the target's centre) a lance ship is lining up on.
    aim_off: (i32, i32),
    aim_for: Option<usize>,
    aim_t: i32,
    /// What it is in a match: an ordinary ship, a king or a seed.
    pub role: Role,
    /// Pixels stored here. Nodes (kings, and anything with factory pixels) hold the bank.
    pub store: i32,
    pub node: bool,
    /// Its build order (deck slots), and where it is in it.
    pub build: Vec<usize>,
    pub build_i: usize,
    print_acc: i32,
    pub making: Making,
    make_off: (i32, i32),
    /// Cells still to grow: a factory growing out of its rock, or a ship being printed.
    pub grow: Option<Grow>,
    /// The factory printing this ship, while it's being printed.
    pub maker: Option<usize>,
    /// Ticks left of a hyperspace arrival.
    pub warp: u32,
    /// A seed's design (deck slot) and the rock pixel it's flying to.
    pub seed_for: Option<(usize, (i32, i32))>,
    /// Planted: grown out of a rock and still standing where its seed came down. A factory stays
    /// planted for good; a seeded design with engines stops being planted the moment it flies.
    /// Anything planted is solid to every enemy ship (see `sim/space.rs`).
    pub rooted: bool,
}

impl Ship {
    pub fn dims(&self) -> (i32, i32) {
        self.shape.dims(self.facing)
    }
    pub fn half(&self) -> i32 {
        let (w, h) = self.dims();
        w.max(h) / 2
    }
    pub fn bbox(&self) -> (i32, i32, i32, i32) {
        let (w, h) = self.dims();
        let (l, t) = (self.x - w / 2, self.y - h / 2);
        (l, t, l + w - 1, t + h - 1)
    }
    pub fn cell_xy(&self, i: usize) -> (i32, i32) {
        self.shape.cell_xy(self.x, self.y, self.facing, i)
    }
    pub fn cell_at(&self, px: i32, py: i32) -> Option<usize> {
        self.shape.cell_at(self.x, self.y, self.facing, px, py)
    }
    pub fn alive_cells(&self) -> Vec<Option<Part>> {
        self.shape.cells.iter().zip(&self.hp).map(|(c, &h)| if h > 0 { *c } else { None }).collect()
    }
    /// Pixels still standing.
    pub fn px_now(&self) -> i32 {
        self.hp.iter().map(|&h| h as i32).sum()
    }
    pub fn px_full(&self) -> i32 {
        self.shape.cells.iter().flatten().count() as i32
    }
    pub fn speed_px_s(&self) -> i32 {
        isqrt(self.vx * self.vx + self.vy * self.vy) * 15 / 4
    }
    /// A ship lives while any of its cores does.
    pub fn has_core(&self) -> bool {
        (0..self.hp.len()).any(|c| self.hp[c] > 0 && self.shape.is_core(c))
    }

    /// A new ship made of some of this one's cells (a cut-off piece with a core of its own). It
    /// keeps the same pixels in the same places, facing and drift, and takes along the guns,
    /// cannons and their reloads that sit in it.
    fn piece(&self, cells: &[usize], tick: u32) -> Ship {
        let w = self.shape.w;
        let (x0, x1) = (cells.iter().map(|&c| c as i32 % w).min().unwrap(), cells.iter().map(|&c| c as i32 % w).max().unwrap());
        let (y0, y1) = (cells.iter().map(|&c| c as i32 / w).min().unwrap(), cells.iter().map(|&c| c as i32 / w).max().unwrap());
        let (nw, nh) = (x1 - x0 + 1, y1 - y0 + 1);
        // Old cell index -> new cell index.
        let map = |c: usize| ((c as i32 / w - y0) * nw + (c as i32 % w - x0)) as usize;
        let mut new_cells = vec![None; (nw * nh) as usize];
        let mut cool = vec![0; (nw * nh) as usize];
        for &c in cells {
            new_cells[map(c)] = self.shape.cells[c];
            cool[map(c)] = self.cool[c];
        }
        let turrets: Vec<Turret> = self.turrets.iter().filter(|t| cells.contains(&t.cell)).map(|t| Turret { cell: map(t.cell), ..t.clone() }).collect();
        let shape = Shape { w: nw, h: nh, cells: new_cells, cannons: turrets.iter().map(|t| t.cell).collect() };
        // Place the new centre so every pixel stays exactly where it was.
        let c0 = cells[0];
        let (wx, wy) = self.cell_xy(c0);
        let (lx, ly) = shape.cell_xy(0, 0, self.facing, map(c0));
        let (x, y) = (wx - lx, wy - ly);
        let hp = shape.cells.iter().map(|c| c.is_some() as u8).collect();
        let working = turrets.len() as i32;
        let st = stats_of(&shape.cells, working);
        Ship {
            shape,
            st,
            x,
            y,
            hp,
            cool,
            turrets,
            // A piece told to hold stays where it broke off; the rest keep the old ship's home.
            home: if self.order == Order::Hold { (x, y) } else { self.home },
            ring0: if self.order == Order::Hold { 0 } else { self.ring0 },
            posted: false,
            carry: 0,
            hauling: false,
            tpx: None,
            chew: 0,
            last_hit: tick,
            target: None,
            goal: None,
            trail: Vec::new(),
            dirty: false,
            bumped: false,
            outbound: false,
            run_t: 0,
            doing: Doing::Idle,
            aim_for: None,
            aim_t: 0,
            pose0: (x, y, self.facing),
            role: Role::Ship,
            store: 0,
            node: cells.iter().any(|&c| self.shape.cells[c] == Some(Part::Factory)),
            build_i: 0,
            print_acc: 0,
            making: Making::None,
            make_off: (0, 0),
            grow: None,
            maker: None,
            warp: 0,
            seed_for: None,
            rooted: false,
            ..self.clone()
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Kind {
    Flak,
    Lance,
    Shell,
}

#[derive(Clone, Copy, Debug)]
pub struct Shot {
    pub team: usize,
    pub kind: Kind,
    pub line: Line,
    pub left: i32,
    pub speed: i32,
    dmg: i32,
    pen: i32,
    /// The ship it was fired at, so other guns can spread their fire.
    target: Option<usize>,
    pub prev: (i32, i32),
    /// Counted once in the shooter's hits, however many pixels it punches.
    connected: bool,
    /// The tick it was fired.
    born: u32,
}

#[derive(Clone, Copy, Debug)]
pub struct Debris {
    pub x: i32,
    pub y: i32,
    vx: i32,
    vy: i32,
    drift: i32,
    taken: bool,
}

/// A short-lived flash: a shot bouncing off plate, or a shell's blast.
#[derive(Clone, Copy, Debug)]
pub struct Spark {
    pub x: i32,
    pub y: i32,
    pub t: u8,
    pub blast: bool,
}

#[derive(Clone, Copy, Debug, Default)]
pub struct Score {
    pub placed: i32,
    pub lost: i32,
    pub px_lost: i32,
    /// Pixels that broke away when cut off from every core.
    pub broken: i32,
    /// Pieces that flew on as ships of their own (each one also counts as placed).
    pub split: i32,
    pub mined: i32,
    pub fired: i32,
    pub hits: i32,
    /// Hits that bounced off plate.
    pub bounced: i32,
}

#[derive(Clone, Copy, Debug)]
struct Snap {
    idx: usize,
    team: usize,
    x: i32,
    y: i32,
    vx: i32,
    vy: i32,
    half: i32,
    /// Pixels as built: its size class, which damage doesn't change.
    full: i32,
    /// Whether it was a miner at work last tick: after a rock pixel, or taking a full load home.
    /// It's part of the snapshot so that every ship gets the same answer, whatever order they
    /// think in.
    working: bool,
}

/// What a brain wants this tick.
struct Plan {
    dv: (i32, i32),
    face: Option<u8>,
    doing: Doing,
    target: Option<usize>,
    goal: Option<(i32, i32)>,
}

pub struct Arena {
    pub tick: u32,
    pub ships: Vec<Ship>,
    pub shots: Vec<Shot>,
    pub debris: Vec<Debris>,
    pub sparks: Vec<Spark>,
    pub rock: Vec<bool>,
    rock_px: Vec<(i32, i32)>,
    pub stars: Vec<(i32, i32, u8)>,
    rng: Rng,
    pub score: [Score; 2],
    /// Some(Some(team)) when a team won, Some(None) for a draw.
    pub over: Option<Option<usize>>,
    /// The draw came from nobody being able to hurt anybody, not from the time limit.
    pub stalemate: bool,
    started: [bool; 2],
    /// The tick a pixel was last shot off (0: none yet).
    last_loss: u32,
    /// The tick anyone last fired, or an armed ship got closer to an enemy.
    last_push: u32,
    /// The map's size: the proving ground's arena is AW x AH, a match's map bigger.
    pub w: i32,
    pub h: i32,
    /// A match's kings, factories, banks and networks; None in the proving ground.
    pub eco: Option<Eco>,
}

impl Arena {
    /// An empty map.
    fn blank(w: i32, h: i32, seed: u64) -> Arena {
        Arena {
            tick: 0,
            ships: Vec::new(),
            shots: Vec::new(),
            debris: Vec::new(),
            sparks: Vec::new(),
            rock: vec![false; (w * h) as usize],
            rock_px: Vec::new(),
            stars: Vec::new(),
            rng: Rng::new(seed),
            score: [Score::default(); 2],
            over: None,
            stalemate: false,
            started: [false; 2],
            last_loss: 0,
            last_push: 0,
            w,
            h,
            eco: None,
        }
    }

    /// Always the same arena, so the same setup replays the same fight.
    pub fn new() -> Arena {
        let mut a = Arena::blank(AW, AH, 1);
        for (cx, cy, r) in [(AW / 2, AH / 2, 18), (AW / 2, 70, 11), (AW / 2, AH - 71, 11)] {
            for y in -r..=r {
                for x in -r..=r {
                    let d = dist2(x, y, 0, 0);
                    let edge = d > (r - 2) * (r - 2);
                    if d <= r * r && !(edge && a.rng.chance(35)) {
                        a.rock[((cy + y) * AW + cx + x) as usize] = true;
                        a.rock_px.push((cx + x, cy + y));
                    }
                }
            }
        }
        for _ in 0..170 {
            let (x, y) = (a.rng.range(0, AW - 1), a.rng.range(0, AH - 1));
            let c = if a.rng.chance(15) { 2 } else if a.rng.chance(40) { 1 } else { 0 };
            a.stars.push((x, y, c));
        }
        a
    }

    /// The same rocks and stars with a different roll of the dice for the fight, so the headless
    /// runs can tell a real result from one lucky replay. Seed 1 is the fight the window shows.
    pub fn seeded(seed: u64) -> Arena {
        let mut a = Arena::new();
        if seed != 1 {
            a.rng = Rng::new(seed);
        }
        a
    }

    pub fn rock_at(&self, x: i32, y: i32) -> bool {
        (0..self.w).contains(&x) && (0..self.h).contains(&y) && self.rock[(y * self.w + x) as usize]
    }

    /// Places a ship; refuses a design that can't fly (no core, loose pixels, buried engines).
    pub fn add(&mut self, team: usize, slot: usize, d: &Design, x: i32, y: i32) -> bool {
        if !d.check().ok() {
            return false;
        }
        let i = self.spawn(team, slot, d, x, y, if team == 0 { 1 } else { 3 }, false);
        // A seeded design is placed as it would stand on its rock in a match: planted, so solid to
        // every enemy ship, until it flies (if it can).
        self.ships[i].rooted = d.seeded;
        // The arena is a fight, with no factory to guard and no king to escort: a design drawn on
        // GUARD or ESCORT (the order it starts a match on) hunts here.
        if self.eco.is_none() && matches!(d.order, Order::Guard | Order::Escort) {
            self.ships[i].order = Order::Hunt;
        }
        self.score[team].placed += 1;
        true
    }

    /// Puts a ship on the map facing one way, without checking the design; `empty` leaves every
    /// pixel still to grow. Returns its index.
    pub fn spawn(&mut self, team: usize, slot: usize, d: &Design, x: i32, y: i32, facing: u8, empty: bool) -> usize {
        let st = d.stats();
        let shape = d.shape();
        let n = shape.cells.len();
        let hp = shape.cells.iter().map(|c| (c.is_some() && !empty) as u8).collect();
        // Stagger the guns so a broadside ripples instead of firing as one.
        let cool = (0..n).map(|i| (i as i32 * 7) % FLAK_PERIOD).collect();
        let turrets = shape
            .cannons
            .iter()
            .enumerate()
            .map(|(k, &cell)| Turret { cell, aim: facing as i32 * 16, cool: (k as i32 * 11) % CANNON_PERIOD, turn_t: 0, aim_off: (0, 0), aim_for: None })
            .collect();
        let mut s = Ship {
            team,
            slot,
            name: d.name.clone(),
            order: d.order,
            shape,
            st,
            x,
            y,
            sx: 0,
            sy: 0,
            vx: 0,
            vy: 0,
            acc: 0,
            facing,
            turn_t: 0,
            hp,
            cool,
            turrets,
            alive: true,
            doing: Doing::Idle,
            outbound: false,
            run_t: 0,
            home: (x, y),
            ring0: 0,
            posted: false,
            carry: 0,
            hauling: false,
            tpx: None,
            chew: 0,
            last_hit: 0,
            target: None,
            goal: None,
            trail: Vec::new(),
            dirty: false,
            bumped: false,
            aim_off: (0, 0),
            aim_for: None,
            aim_t: 0,
            pose0: (x, y, facing),
            role: Role::Ship,
            store: 0,
            node: d.cells.contains(&Some(Part::Factory)),
            build: d.build.clone(),
            build_i: 0,
            print_acc: 0,
            making: Making::None,
            make_off: (0, 0),
            grow: None,
            maker: None,
            warp: 0,
            seed_for: None,
            rooted: false,
        };
        let h = s.half();
        s.x = x.clamp(h, self.w - 1 - h); // spawn
        s.y = y.clamp(h, self.h - 1 - h); // spawn
        s.home = (s.x, s.y);
        s.pose0 = (s.x, s.y, facing);
        if empty {
            s.st = stats_of(&s.alive_cells(), 0);
        }
        self.ships.push(s);
        self.ships.len() - 1
    }

    /// The living ship under a point (with a pixel of slack).
    pub fn ship_at(&self, x: i32, y: i32) -> Option<usize> {
        self.ships
            .iter()
            .enumerate()
            .filter(|(_, s)| {
                let (l, t, r, b) = s.bbox();
                s.alive && x >= l - 2 && y >= t - 2 && x <= r + 2 && y <= b + 2
            })
            .min_by_key(|(_, s)| dist2(s.x, s.y, x, y))
            .map(|(i, _)| i)
    }

    pub fn alive(&self, team: usize) -> usize {
        self.ships.iter().filter(|s| s.alive && s.team == team).count()
    }

    /// A team still in the fight with no working guns left anywhere.
    pub fn disarmed(&self, team: usize) -> bool {
        self.alive(team) > 0 && !self.ships.iter().any(|s| s.alive && s.team == team && s.st.armed())
    }

    pub fn step(&mut self) {
        if self.over.is_some() {
            return;
        }
        if self.tick == 0 {
            self.started = [self.alive(0) > 0, self.alive(1) > 0];
        }
        self.tick += 1;
        for s in &mut self.ships {
            s.pose0 = (s.x, s.y, s.facing);
        }
        if self.eco.is_some() {
            self.eco_pre();
        }
        // Ships still arriving by hyperspace aren't there yet.
        let snap: Vec<Snap> = self
            .ships
            .iter()
            .enumerate()
            .filter(|(_, s)| s.alive && s.warp == 0)
            .map(|(idx, s)| Snap { idx, team: s.team, x: s.x, y: s.y, vx: s.vx, vy: s.vy, half: s.half(), full: s.px_full(), working: matches!(s.doing, Doing::Mine | Doing::Haul) })
            .collect();
        let n = self.ships.len();
        // How close each armed ship that can move is to its nearest enemy, before and after
        // everyone moves: closing in is still a fight, not a stalemate.
        let pushing = |a: &Arena, i: usize| a.ships[i].alive && a.ships[i].st.armed() && a.ships[i].st.thrust > 0;
        let near = |a: &Arena, i: usize| {
            let s = &a.ships[i];
            snap.iter().filter(|e| e.team != s.team && a.ships[e.idx].alive).map(|e| dist2(a.ships[e.idx].x, a.ships[e.idx].y, s.x, s.y)).min().unwrap_or(i32::MAX)
        };
        let before: Vec<i32> = (0..n).map(|i| if pushing(self, i) { near(self, i) } else { i32::MAX }).collect();
        for i in 0..n {
            if self.ships[i].alive && self.moves(i) {
                let plan = if self.ships[i].role == Role::Seed { self.seed_plan(i) } else { self.brain(i, &snap) };
                self.fly(i, plan);
            }
        }
        if self.eco.is_some() {
            self.eco_follow();
        }
        if (0..n).any(|i| before[i] != i32::MAX && pushing(self, i) && near(self, i) < before[i]) {
            self.last_push = self.tick;
        }
        let mut pending = vec![0; n];
        for t in self.shots.iter().filter_map(|s| s.target) {
            pending[t] += 1;
        }
        let fired = self.score[0].fired + self.score[1].fired;
        for i in 0..n {
            if self.ships[i].alive && self.ships[i].warp == 0 && self.ships[i].maker.is_none() {
                self.weapons(i, &snap, &mut pending);
            }
        }
        if self.score[0].fired + self.score[1].fired != fired {
            self.last_push = self.tick;
        }
        self.shots_step();
        self.break_off();
        self.debris_step();
        self.deaths();
        if self.eco.is_some() {
            self.eco_post();
        }
        for s in self.ships.iter_mut().filter(|s| s.alive) {
            if s.dirty {
                s.dirty = false;
                let cannons = s.turrets.iter().filter(|t| s.hp[t.cell] > 0).count() as i32;
                s.st = stats_of(&s.alive_cells(), cannons);
            }
            if self.tick % 2 == 0 {
                s.trail.push((s.x, s.y));
                if s.trail.len() > 10 {
                    s.trail.remove(0);
                }
            }
        }
        for sp in &mut self.sparks {
            sp.t = sp.t.saturating_sub(1);
        }
        self.sparks.retain(|sp| sp.t > 0);
        if self.eco.is_some() {
            // A match ends when a king dies (eco_post decides).
            return;
        }
        let (a, b) = (self.alive(0), self.alive(1));
        if self.started[0] && self.started[1] {
            if a == 0 && b == 0 {
                self.over = Some(None);
            } else if a == 0 {
                self.over = Some(Some(1));
            } else if b == 0 {
                self.over = Some(Some(0));
            }
        }
        if self.over.is_none() && self.last_loss > 0 && self.tick - self.last_loss.max(self.last_push) >= STALEMATE {
            self.over = Some(None);
            self.stalemate = true;
        }
        if self.over.is_none() && self.tick >= TIME_LIMIT {
            self.over = Some(None);
        }
    }

    // ----- Layer 1: the order picks where to go -----

    fn brain(&mut self, i: usize, snap: &[Snap]) -> Plan {
        let (team, x, y, order, home, st, last_hit) = {
            let s = &self.ships[i];
            (s.team, s.x, s.y, s.order, s.home, s.st, s.last_hit)
        };
        let armed = st.armed();
        let hurt = last_hit > 0 && self.tick.saturating_sub(last_hit) < SCARED;
        let station = self.station(i).0;
        let nearest = |maxd: i32, from: (i32, i32)| {
            snap.iter().filter(|e| e.team != team && cheb(e.x, e.y, from.0, from.1) <= maxd).min_by_key(|e| dist2(e.x, e.y, x, y)).copied()
        };
        let mut plan = match order {
            Order::Hunt => {
                if !armed {
                    self.with_group(i, snap)
                } else {
                    match nearest(i32::MAX / 4, (x, y)) {
                        Some(e) => self.engage(i, &e, snap),
                        None => self.go(i, home, Doing::Idle),
                    }
                }
            }
            Order::Guard => match nearest(GUARD_RADIUS, home) {
                Some(e) if armed && cheb(x, y, home.0, home.1) <= GUARD_RADIUS + 50 => self.engage(i, &e, snap),
                _ if !armed && hurt => self.flee(i, station),
                _ => self.go(i, home, Doing::Guard),
            },
            // With its king: it fights what comes near the king while it's near the king itself,
            // and otherwise keeps its place in the rings round it. With no king to escort (the
            // arena), it hunts.
            Order::Escort => match self.escort_spot(i) {
                Some((king, spot)) => match nearest(ESCORT_RADIUS, king) {
                    Some(e) if armed && cheb(x, y, king.0, king.1) <= ESCORT_RADIUS + 50 => self.engage(i, &e, snap),
                    _ => self.go(i, spot, Doing::WithKing),
                },
                None if !armed => self.with_group(i, snap),
                None => match nearest(i32::MAX / 4, (x, y)) {
                    Some(e) => self.engage(i, &e, snap),
                    None => self.go(i, home, Doing::Idle),
                },
            },
            Order::Harvest => {
                if !armed && hurt {
                    self.flee(i, station)
                } else {
                    self.harvest(i)
                }
            }
            Order::Hold => {
                let mut p = self.go(i, home, Doing::Hold);
                if st.lance > 0 {
                    if let Some(e) = nearest(LANCE_RANGE + 10, (x, y)) {
                        p.face = axis_face(e.x - x, e.y - y);
                        p.target = Some(e.idx);
                    }
                } else {
                    p.face = None;
                }
                p
            }
        };
        // A built ship sent home with nothing to do waits at its post instead: its own place in
        // the rings round what built it.
        let posted = st.thrust > 0 && self.ships[i].ring0 > 0 && plan.goal == Some(home) && matches!(plan.doing, Doing::Guard | Doing::Idle | Doing::Hold);
        if posted {
            let spot = self.clear_post(i, self.post(i), snap);
            let mut p = self.go(i, spot, plan.doing);
            if plan.doing == Doing::Hold {
                // Holding, it turns only to bring a lance to bear.
                if plan.target.is_some() || st.lance == 0 {
                    p.face = plan.face;
                }
                p.target = plan.target;
            } else if (x, y) == spot {
                // At its post it looks outward.
                p.face = axis_face(spot.0 - home.0, spot.1 - home.1);
            }
            plan = p;
        }
        self.ships[i].posted = posted;
        // Friends keep a pixel or two apart instead of stacking into one sprite. A miner at work
        // takes no part in that, either way. After a rock pixel it goes to that pixel, whether its
        // own factory stands over it or a fleet is parked there (pushed off, it never gets its
        // bite), and with a full load it goes straight home (pushed back, it never unloads). Nor
        // does it push anyone: miners working one rock face sit on top of each other and give
        // way to nothing, so their pushes added up to a wall that nothing of their own side
        // could cross, a full miner on the far side of it least of all. A ship at its post is
        // the other exception: it has a place of its own and holds it, and whatever flies by
        // goes round. (Pushed off and pulled back, it never sat still.)
        if st.thrust > 0 && !matches!(plan.doing, Doing::Mine | Doing::Haul) && !posted {
            let h = self.ships[i].half();
            for f in snap.iter().filter(|f| f.team == team && f.idx != i && !f.working && cheb(f.x, f.y, x, y) < f.half + h + 2) {
                let (ax, ay) = if (f.x, f.y) == (x, y) { (if i < f.idx { -4 } else { 4 }, 0) } else { toward(f.x, f.y, x, y, 4) };
                plan.dv.0 += ax;
                plan.dv.1 += ay;
            }
        }
        plan
    }

    /// Where a built ship waits with nothing to do: its own place in the rings round what built
    /// it. Ships on GUARD fill the rings in the order they were built (the first ring nearest
    /// the builder, each ring holding as many as fit `POST_SPACE` apart) and close up when one is
    /// lost; a guard away fighting keeps its place. Ships waiting for any other reason come after
    /// the guards.
    fn post(&self, i: usize) -> (i32, i32) {
        let s = &self.ships[i];
        let same = |o: &Ship| o.alive && o.team == s.team && o.home == s.home && o.ring0 > 0 && o.role == Role::Ship && o.grow.is_none() && o.warp == 0 && o.st.thrust > 0;
        let guards = |v: &[Ship]| v.iter().filter(|o| same(o) && o.order == Order::Guard).count() as i32;
        let mut rank = if s.order == Order::Guard {
            guards(&self.ships[..i])
        } else {
            guards(&self.ships) + self.ships[..i].iter().filter(|o| same(o) && o.order != Order::Guard && o.posted).count() as i32
        };
        let (mut base, mut ring) = (s.ring0, 0);
        let room = |base: i32| (base * 2 / 5).clamp(6, 48);
        while rank >= room(base) {
            rank -= room(base);
            base += POST_SPACE;
            ring += 1;
        }
        // Each ring starts a little further round than the one inside it.
        let (ux, uy) = dir64((rank * 64 / room(base) + ring * 3) % 64);
        let (r, h) = (base + s.half(), s.half());
        ((s.home.0 + ux * r / 1024).clamp(h, self.w - 1 - h), (s.home.1 + uy * r / 1024).clamp(h, self.h - 1 - h))
    }

    /// A waiting ship gives way to what can't move aside for it (a factory, a ship still being
    /// printed, a king) and to ships already waiting that were built before it. Its place is
    /// then the nearest point to its post that is clear of them all: of two as near, the one
    /// further from home. Where they stand doesn't change from tick to tick, so neither does that
    /// point: the ship flies there and stops.
    ///
    /// (It used to step out of one thing at a time, through its nearer side. Between two things
    /// standing close together that stepped it out of each into the other, and it came to rest
    /// inside one of them.)
    fn clear_post(&self, i: usize, post: (i32, i32), snap: &[Snap]) -> (i32, i32) {
        let s = &self.ships[i];
        let h = s.half();
        // What's in the way: where each stands, and how near its middle this ship's may come.
        let blocks: Vec<((i32, i32), i32)> = snap
            .iter()
            .filter(|f| f.team == s.team && f.idx != i)
            .filter_map(|f| {
                let o = &self.ships[f.idx];
                let at = if o.st.thrust == 0 || o.grow.is_some() || o.role == Role::King {
                    (f.x, f.y)
                } else if f.idx < i && o.posted {
                    o.goal?
                } else {
                    return None;
                };
                let r = f.half + h + 3;
                (cheb(post.0, post.1, at.0, at.1) < r + POST_SEARCH).then_some((at, r))
            })
            .collect();
        let clear = |p: (i32, i32)| blocks.iter().all(|&((x, y), r)| cheb(p.0, p.1, x, y) >= r);
        if clear(post) {
            return post;
        }
        // Outward a ring at a time. A ring `d` out is at least `d` away, so the search is over
        // once a clear point nearer than that is in hand.
        let on_map = |p: (i32, i32)| (h..self.w - h).contains(&p.0) && (h..self.h - h).contains(&p.1);
        let mut best: Option<((i32, i32, i32, i32), (i32, i32))> = None;
        for d in 1..=POST_SEARCH {
            if best.is_some_and(|(k, _)| k.0 < d * d) {
                break;
            }
            let ring = (-d..=d).flat_map(|t| [(t, -d), (t, d)]).chain((1 - d..d).flat_map(|t| [(-d, t), (d, t)]));
            for (dx, dy) in ring {
                let p = (post.0 + dx, post.1 + dy);
                if on_map(p) && clear(p) {
                    let key = (dist2(p.0, p.1, post.0, post.1), -dist2(p.0, p.1, s.home.0, s.home.1), p.1, p.0);
                    if best.map_or(true, |(k, _)| key < k) {
                        best = Some((key, p));
                    }
                }
            }
        }
        best.map_or(post, |(_, p)| p)
    }

    /// Where a ship on ESCORT belongs: its king's position, and its own place in the rings round
    /// the king. Escorts fill the rings in the order they were built (6 in the first ring, 12 in
    /// the next, and so on) and close up when one is lost.
    fn escort_spot(&self, i: usize) -> Option<((i32, i32), (i32, i32))> {
        let s = &self.ships[i];
        let k = &self.ships[self.eco.as_ref()?.king[s.team]];
        if !k.alive {
            return None;
        }
        let mut rank = self.ships[..i].iter().filter(|o| o.alive && o.team == s.team && o.order == Order::Escort && o.role == Role::Ship && o.grow.is_none() && o.warp == 0).count() as i32;
        let (mut ring, mut room) = (0, 6);
        while rank >= room {
            rank -= room;
            ring += 1;
            room += 6;
        }
        // Each ring starts a little further round than the one inside it.
        let (ux, uy) = dir64((rank * 64 / room + ring * 5) % 64);
        let r = k.half() + 12 + ring * 14 + s.half();
        // It flies for where its place will be a second from now, so it keeps up with a king on
        // the move instead of braking for a spot the king has already left.
        let (lx, ly) = (k.vx * ESCORT_LEAD / 4, k.vy * ESCORT_LEAD / 4);
        let spot = ((k.x + ux * r / 1024 + lx).clamp(0, self.w - 1), (k.y + uy * r / 1024 + ly).clamp(0, self.h - 1));
        Some(((k.x, k.y), spot))
    }

    /// A hunter with no guns left stays with its group: beside the nearest friend that still has
    /// guns (or any friend, if none do), on the side it's already on. With no friends left, it
    /// stays where it is.
    fn with_group(&self, i: usize, snap: &[Snap]) -> Plan {
        let s = &self.ships[i];
        let friends = || snap.iter().filter(|f| f.team == s.team && f.idx != i);
        let armed = friends().filter(|f| self.ships[f.idx].st.armed()).min_by_key(|f| dist2(f.x, f.y, s.x, s.y));
        let Some(f) = armed.or_else(|| friends().min_by_key(|f| dist2(f.x, f.y, s.x, s.y))) else {
            return self.go(i, (s.x, s.y), Doing::Idle);
        };
        let gap = f.half + s.half() + 4;
        let (ux, uy) = if (s.x, s.y) == (f.x, f.y) { (64, 0) } else { toward(f.x, f.y, s.x, s.y, 64) };
        let spot = ((f.x + ux * gap / 64).clamp(0, self.w - 1), (f.y + uy * gap / 64).clamp(0, self.h - 1));
        let mut p = self.go(i, spot, Doing::Escort);
        p.target = Some(f.idx);
        p
    }

    /// Arrive at a point, braking in time for this ship's thrust and weight.
    fn arrive(&self, i: usize, gx: i32, gy: i32) -> (i32, i32) {
        let s = &self.ships[i];
        let d = isqrt(dist2(s.x, s.y, gx, gy));
        let brake = isqrt(8 * s.st.thrust * d / s.st.mass.max(1));
        toward(s.x, s.y, gx, gy, s.st.vmax.min(brake).min(d * 4))
    }

    fn go(&self, i: usize, (gx, gy): (i32, i32), doing: Doing) -> Plan {
        let dv = self.arrive(i, gx, gy);
        let face = if dv.0.abs() + dv.1.abs() > 2 { axis_face(dv.0, dv.1) } else { None };
        Plan { dv, face, doing, target: None, goal: Some((gx, gy)) }
    }

    fn flee(&self, i: usize, (gx, gy): (i32, i32)) -> Plan {
        let s = &self.ships[i];
        let dv = toward(s.x, s.y, gx, gy, s.st.vmax);
        Plan { dv, face: axis_face(dv.0, dv.1), doing: Doing::Flee, target: None, goal: Some((gx, gy)) }
    }

    // ----- Layer 3: mass and thrust decide how to fly -----

    fn engage(&mut self, i: usize, e: &Snap, snap: &[Snap]) -> Plan {
        match self.ships[i].st.flight {
            Flight::Runs => self.attack_run(i, e),
            Flight::StandOff => self.stand_off(i, e, snap),
            Flight::Still => Plan { dv: (0, 0), face: None, doing: Doing::Hold, target: Some(e.idx), goal: None },
        }
    }

    /// Dive at the target (leading it), fly through, keep going a little, then come about.
    fn attack_run(&mut self, i: usize, e: &Snap) -> Plan {
        let (x, y, vx, vy, vmax) = {
            let s = &self.ships[i];
            (s.x, s.y, s.vx, s.vy, s.st.vmax)
        };
        let d = isqrt(dist2(x, y, e.x, e.y));
        let t = d * 4 / vmax.max(1);
        let (tx, ty) = ((e.x + e.vx * t / 4).clamp(0, self.w - 1), (e.y + e.vy * t / 4).clamp(0, self.h - 1));
        // Flying through only works on ships smaller than you. If this course would hit a
        // bigger one before we could stop, peel off sideways: a strafing pass along its hull.
        let peel = self.impact(i).map(|b| self.peel(i, b));
        let lance_run = if !self.ships[i].outbound && self.ships[i].st.lance > 0 { Some(self.line_up(i, e, (tx, ty))) } else { None };
        let s = &mut self.ships[i];
        s.run_t += 1;
        if !s.outbound {
            let closing = (e.x - x) * vx + (e.y - y) * vy;
            if let Some(dv) = peel {
                s.outbound = true;
                s.run_t = 0;
                return Plan { dv, face: axis_face(dv.0, dv.1), doing: Doing::RunOut, target: Some(e.idx), goal: None };
            }
            if d <= e.half + 6 || s.bumped || (d < 36 + e.half && closing < 0 && s.run_t > 8) {
                s.outbound = true;
                s.run_t = 0;
            }
            if let Some(p) = lance_run {
                return p;
            }
            let dv = toward(x, y, tx, ty, vmax);
            Plan { dv, face: axis_face(dv.0, dv.1), doing: Doing::RunIn, target: Some(e.idx), goal: Some((tx, ty)) }
        } else {
            let dv = if let Some(p) = peel {
                p
            } else if vx == 0 && vy == 0 {
                toward(e.x, e.y, x, y, vmax)
            } else {
                toward(0, 0, vx, vy, vmax)
            };
            let near_wall = x < 16 || y < 16 || x > self.w - 17 || y > self.h - 17;
            if near_wall || (s.run_t > 8 && d > 48 + e.half) || s.run_t > 30 {
                s.outbound = false;
                s.run_t = 0;
            }
            Plan { dv, face: axis_face(dv.0, dv.1), doing: Doing::RunOut, target: Some(e.idx), goal: None }
        }
    }

    /// Hold at weapon range. Ships with lances line up on the target's row or column so the lances
    /// bear (they only fire straight ahead); cannon ships hold at cannon range and flak-only ships
    /// close to flak range from wherever they are.
    fn stand_off(&mut self, i: usize, e: &Snap, snap: &[Snap]) -> Plan {
        let (x, y, h, st, team) = {
            let s = &self.ships[i];
            (s.x, s.y, s.half(), s.st, s.team)
        };
        let pref = if st.lance > 0 {
            LANCE_RANGE * 3 / 4
        } else if st.cannon > 0 {
            CANNON_RANGE * 2 / 3
        } else {
            FLAK_RANGE * 2 / 3
        } + e.half;
        let (aw, ah) = (self.w, self.h);
        let inside = |(px, py): (i32, i32)| px >= h + 2 && py >= h + 2 && px <= aw - 3 - h && py <= ah - 3 - h;
        // Spots in line with the target, at a few distances; a spot a friend already holds costs
        // extra, so a group spreads into a firing line instead of stacking (own shots pass through
        // own ships, so a line behind a line still fires).
        let crowd = |(px, py): (i32, i32)| snap.iter().filter(|f| f.team == team && f.idx != i && cheb(f.x, f.y, px, py) <= f.half + h + 2).count() as i32;
        let lined_up = if st.lance > 0 {
            let aim = self.lance_aim(i, e);
            let mut spots = Vec::new();
            for d in [pref, pref - 14, pref + 14] {
                // East of the target facing west, west facing east, south facing north, north facing south.
                spots.push((e.x + d, self.lance_line(i, e, aim, 3, d)));
                spots.push((e.x - d, self.lance_line(i, e, aim, 1, d)));
                spots.push((self.lance_line(i, e, aim, 0, d), e.y + d));
                spots.push((self.lance_line(i, e, aim, 2, d), e.y - d));
            }
            spots.into_iter().filter(|&p| inside(p)).min_by_key(|&p| dist2(p.0, p.1, x, y) + crowd(p) * 20_000)
        } else {
            None
        };
        let point = lined_up.unwrap_or_else(|| {
            let (ux, uy) = toward(e.x, e.y, x, y, 64);
            let (ux, uy) = if (ux, uy) == (0, 0) { (-64, 0) } else { (ux, uy) };
            ((e.x + ux * pref / 64).clamp(h + 2, self.w - 3 - h), (e.y + uy * pref / 64).clamp(h + 2, self.h - 3 - h))
        });
        let dv = self.arrive(i, point.0, point.1);
        Plan { dv, face: axis_face(e.x - x, e.y - y), doing: Doing::StandOff, target: Some(e.idx), goal: Some(point) }
    }

    // ----- Lances: a gun that fires like a rook on a ship that flies freely -----
    // Any ship with a working lance, whatever its shape or flight style, gets where it needs to be
    // through these two: the pixel to aim at, and where the ship has to sit for its lance to bear.

    /// The pixel of target `e` (relative to its centre) that ship `i`'s lance lines up on: a
    /// surviving one, picked at random and re-picked after every shot, so a group works across
    /// the hull instead of drilling one tunnel that later shots fly straight through.
    fn lance_aim(&mut self, i: usize, e: &Snap) -> (i32, i32) {
        let stale = {
            let s = &self.ships[i];
            let ts = &self.ships[e.idx];
            s.aim_for != Some(e.idx) || s.aim_t <= 0 || ts.cell_at(e.x + s.aim_off.0, e.y + s.aim_off.1).map_or(true, |c| ts.hp[c] == 0)
        };
        if stale {
            let (px, py) = self.random_cell(e.idx);
            let s = &mut self.ships[i];
            s.aim_off = (px - e.x, py - e.y);
            s.aim_for = Some(e.idx);
            s.aim_t = LANCE_PERIOD;
        }
        let s = &mut self.ships[i];
        s.aim_t -= 1;
        s.aim_off
    }

    /// Where ship `i`'s centre has to be, across the line of fire, for its lance to bear on `aim`
    /// when the ship faces `f` from `dist` px away: a y when facing east or west, an x when facing
    /// north or south. It leads a moving target by the time a lance shot takes to cross `dist`.
    fn lance_line(&self, i: usize, e: &Snap, aim: (i32, i32), f: u8, dist: i32) -> i32 {
        let s = &self.ships[i];
        let lance = (0..s.shape.cells.len()).find(|&c| s.shape.cells[c] == Some(Part::Lance) && s.hp[c] > 0);
        let (ox, oy) = lance.map_or((0, 0), |c| s.shape.cell_xy(0, 0, f, c));
        let t = dist.abs() / LANCE_SPEED;
        if f % 2 == 1 {
            e.y + aim.1 + e.vy * t / 4 - oy
        } else {
            e.x + aim.0 + e.vx * t / 4 - ox
        }
    }

    /// The run in for a ship with a lance: slide onto the target's row or column, then dive along
    /// it with the lance pointed at the hull. Sliding across gets the speed it needs first; what's
    /// left closes the distance, so a ship far off the line slows its approach until it's on.
    fn line_up(&mut self, i: usize, e: &Snap, (tx, ty): (i32, i32)) -> Plan {
        let aim = self.lance_aim(i, e);
        let s = &self.ships[i];
        let (x, y, h, st) = (s.x, s.y, s.half(), s.st);
        let (ax, ay) = ((tx - x).abs(), (ty - y).abs());
        // Dive along a row (facing east or west) or a column. Keep the current axis unless the
        // other is much longer, so the choice doesn't flicker on a diagonal.
        let row = if s.facing % 2 == 1 { ax * 2 >= ay } else { ax > ay * 2 };
        let f = if row {
            if tx >= x { 1 } else { 3 }
        } else if ty >= y {
            2
        } else {
            0
        };
        let want = if row {
            self.lance_line(i, e, aim, f, ax).clamp(h + 2, self.h - 3 - h)
        } else {
            self.lance_line(i, e, aim, f, ay).clamp(h + 2, self.w - 3 - h)
        };
        let off = want - if row { y } else { x };
        let brake = isqrt(8 * st.thrust * off.abs() / st.mass.max(1));
        let side = off.signum() * st.vmax.min(brake).min(off.abs() * 4);
        let ahead = isqrt(st.vmax * st.vmax - side * side);
        let (dv, goal) = if row {
            (((tx - x).signum() * ahead, side), (tx, want))
        } else {
            ((side, (ty - y).signum() * ahead), (want, ty))
        };
        let doing = if off.abs() > 1 { Doing::LineUp } else { Doing::RunIn };
        Plan { dv, face: Some(f), doing, target: Some(e.idx), goal: Some(goal) }
    }

    fn harvest(&mut self, i: usize) -> Plan {
        let (team, x, y, carry, cap, home) = {
            let s = &self.ships[i];
            (s.team, s.x, s.y, s.carry, s.st.scoops * SCOOP_CARRY, s.home)
        };
        if cap == 0 {
            return self.go(i, home, Doing::Idle);
        }
        let (station, reach, node) = self.station(i);
        if self.ships[i].hauling || carry >= cap {
            self.ships[i].hauling = true;
            if cheb(x, y, station.0, station.1) <= reach {
                self.score[team].mined += carry;
                if let Some(j) = node {
                    self.ships[j].store += carry;
                }
                let s = &mut self.ships[i];
                s.carry = 0;
                s.hauling = false;
            }
            let mut p = self.flee(i, station);
            p.doing = Doing::Haul;
            return p;
        }
        if let Some(d) = self.debris.iter().filter(|d| !d.taken && cheb(d.x, d.y, x, y) <= 30).min_by_key(|d| dist2(d.x, d.y, x, y)) {
            let (dx, dy) = (d.x, d.y);
            return self.go(i, (dx, dy), Doing::Mine);
        }
        let valid = self.ships[i].tpx.map_or(false, |(px, py)| self.rock_at(px, py));
        if !valid {
            self.ships[i].tpx = self.rock_px.iter().copied().filter(|&(px, py)| self.rock_at(px, py)).min_by_key(|&(px, py)| dist2(px, py, x, y));
        }
        match self.ships[i].tpx {
            Some((px, py)) if cheb(x, y, px, py) <= 2 => {
                let scoops = self.ships[i].st.scoops;
                let s = &mut self.ships[i];
                s.chew += 1;
                if s.chew >= if scoops >= 2 { CHEW_TICKS } else { 2 * CHEW_TICKS } {
                    s.chew = 0;
                    s.tpx = None;
                    s.carry += 1;
                    self.rock[(py * self.w + px) as usize] = false;
                    // The bite shows: a spark where the pixel was. A miner at work hardly moves,
                    // so without it there's no telling a working miner from a stuck one.
                    self.sparks.push(Spark { x: px, y: py, t: 4, blast: false });
                }
                Plan { dv: (0, 0), face: None, doing: Doing::Mine, target: None, goal: Some((px, py)) }
            }
            Some(p) => self.go(i, p, Doing::Mine),
            None => {
                if carry > 0 {
                    self.ships[i].hauling = true;
                }
                self.go(i, home, Doing::Idle)
            }
        }
    }

    /// Integer inertia: thrust over weight buys a quarter-pixel of velocity change per tick in
    /// whole steps; facing turns a quarter at a time, as fast as the weight allows.
    fn fly(&mut self, i: usize, p: Plan) {
        let s = &mut self.ships[i];
        s.doing = p.doing;
        s.target = p.target;
        s.goal = p.goal;
        let st = s.st;
        if st.thrust > 0 {
            // Something grown out of a rock that can fly has lifted off: it's a ship like any other.
            s.rooted = false;
            s.acc += st.thrust;
            let budget = (s.acc / st.mass.max(1)).min(8);
            s.acc %= st.mass.max(1);
            s.vx += (p.dv.0 - s.vx).clamp(-budget, budget);
            s.vy += (p.dv.1 - s.vy).clamp(-budget, budget);
        } else {
            s.vx = 0;
            s.vy = 0;
        }
        let v = isqrt(s.vx * s.vx + s.vy * s.vy);
        if v > st.vmax && v > 0 {
            s.vx = s.vx * st.vmax / v;
            s.vy = s.vy * st.vmax / v;
        }
        let mut turn_to = None;
        match p.face {
            Some(f) if f != s.facing && st.turn > 0 => {
                s.turn_t += 1;
                if s.turn_t >= st.turn {
                    s.turn_t = 0;
                    turn_to = Some(if (f + 4 - s.facing) % 4 == 3 { (s.facing + 3) % 4 } else { (s.facing + 1) % 4 });
                }
            }
            _ => s.turn_t = 0,
        }
        if let Some(nf) = turn_to {
            self.try_turn(i, nf);
        }
        let s = &mut self.ships[i];
        s.sx += s.vx;
        s.sy += s.vy;
        let (dx, dy) = (s.sx.div_euclid(4), s.sy.div_euclid(4));
        s.sx = s.sx.rem_euclid(4);
        s.sy = s.sy.rem_euclid(4);
        let h = s.half();
        let (nx, ny) = (s.x + dx, s.y + dy);
        let (cx, cy) = (nx.clamp(h, self.w - 1 - h), ny.clamp(h, self.h - 1 - h));
        if cx != nx {
            s.vx = 0;
            s.sx = 0;
        }
        if cy != ny {
            s.vy = 0;
            s.sy = 0;
        }
        // The move itself, as far as whatever is solid to this ship allows.
        let (fx, fy) = self.shift(i, cx, cy);
        let s = &mut self.ships[i];
        s.bumped = (fx, fy) != (cx, cy);
        if fx != cx {
            s.vx = 0;
            s.sx = 0;
        }
        if fy != cy {
            s.vy = 0;
            s.sy = 0;
        }
    }

    /// The bigger enemy ship `i` would run into if it held its course, looking ahead about as
    /// many ticks as it needs to stop.
    fn impact(&self, i: usize) -> Option<usize> {
        let s = &self.ships[i];
        let v = isqrt(s.vx * s.vx + s.vy * s.vy);
        if v == 0 || s.st.thrust == 0 {
            return None;
        }
        let ahead = (v * s.st.mass / s.st.thrust + 3).min(12);
        (1..=ahead).find_map(|t| self.overlap(i, s.x + s.vx * t / 4, s.y + s.vy * t / 4, s.facing).1)
    }

    /// Full thrust square to the current course, on the side away from ship `big`'s centre.
    fn peel(&self, i: usize, big: usize) -> (i32, i32) {
        let (s, b) = (&self.ships[i], &self.ships[big]);
        let (ax, ay) = (-s.vy, s.vx);
        let away = (s.x - b.x) * ax + (s.y - b.y) * ay;
        let side = if away > 0 || (away == 0 && i % 2 == 0) { 1 } else { -1 };
        toward(0, 0, ax * side, ay * side, s.st.vmax)
    }

    /// Where a ship delivers what it mines and runs to when hurt: the proving ground's station,
    /// or in a match the nearest node of its own (with how close counts as there).
    fn station(&self, i: usize) -> ((i32, i32), i32, Option<usize>) {
        let s = &self.ships[i];
        if self.eco.is_some() {
            let best = self.ships.iter().enumerate().filter(|&(j, o)| j != i && o.alive && o.team == s.team && o.node && o.warp == 0 && o.maker.is_none()).min_by_key(|(_, o)| dist2(o.x, o.y, s.x, s.y));
            return match best {
                Some((j, o)) => ((o.x, o.y), o.half() + 4, Some(j)),
                None => ((s.x, s.y), 0, None),
            };
        }
        (STATIONS[s.team], 10, None)
    }

    /// Whether a ship flies itself this tick: kings are steered, and ships growing, being printed
    /// or arriving by hyperspace stay where they are.
    fn moves(&self, i: usize) -> bool {
        let s = &self.ships[i];
        s.role != Role::King && s.warp == 0 && s.grow.is_none()
    }

    // ----- Layer 2: the parts fire on reflex -----

    fn random_cell(&mut self, idx: usize) -> (i32, i32) {
        let alive: Vec<usize> = (0..self.ships[idx].hp.len()).filter(|&c| self.ships[idx].hp[c] > 0).collect();
        if alive.is_empty() {
            return (self.ships[idx].x, self.ships[idx].y);
        }
        let c = alive[self.rng.below(alive.len() as u32) as usize];
        self.ships[idx].cell_xy(c)
    }

    fn weapons(&mut self, i: usize, snap: &[Snap], pending: &mut [i32]) {
        let team = self.ships[i].team;
        for c in 0..self.ships[i].shape.cells.len() {
            let part = self.ships[i].shape.cells[c];
            if self.ships[i].hp[c] == 0 || !matches!(part, Some(Part::Flak) | Some(Part::Lance)) {
                continue;
            }
            if self.ships[i].cool[c] > 0 {
                self.ships[i].cool[c] -= 1;
                continue;
            }
            let (gx, gy) = self.ships[i].cell_xy(c);
            if part == Some(Part::Flak) {
                // The nearest enemy in range, counting each shot already on its way to a target
                // as extra distance, so several guns spread over several small targets.
                let tgt = snap
                    .iter()
                    .filter(|e| e.team != team && cheb(e.x, e.y, gx, gy) <= FLAK_RANGE + e.half)
                    .min_by_key(|e| isqrt(dist2(e.x, e.y, gx, gy)) + SPREAD * pending[e.idx])
                    .copied();
                if let Some(e) = tgt {
                    // Aim at a random surviving pixel (never the centre: that drills a tunnel later
                    // shots fly straight through), led by the target's velocity.
                    let (px, py) = self.random_cell(e.idx);
                    let t = cheb(px, py, gx, gy) / FLAK_SPEED;
                    let mut aim = (px + e.vx * t / 4, py + e.vy * t / 4);
                    if aim == (gx, gy) {
                        aim = (px, py + 1);
                    }
                    let line = Line::new(gx, gy, aim.0, aim.1);
                    self.shots.push(Shot { team, kind: Kind::Flak, line, left: FLAK_RANGE + 12, speed: FLAK_SPEED, dmg: FLAK_DMG, pen: 1, target: Some(e.idx), prev: (gx, gy), connected: false, born: self.tick });
                    pending[e.idx] += 1;
                    // A ship short of power reloads slower.
                    self.ships[i].cool[c] = self.ships[i].st.reload(FLAK_PERIOD) + self.rng.below(4) as i32;
                    self.score[team].fired += 1;
                }
            } else {
                // A lance only fires straight ahead, and only when its line would cross a pixel
                // still standing where the slow shot will meet it.
                let (dx, dy) = facing_vec(self.ships[i].facing);
                let ships = &self.ships;
                let in_line = snap.iter().any(|e| {
                    if e.team == team {
                        return false;
                    }
                    let along = (e.x - gx) * dx + (e.y - gy) * dy;
                    if along <= 0 || along > LANCE_RANGE + e.half {
                        return false;
                    }
                    let t = along / LANCE_SPEED;
                    let (ox, oy) = (e.vx * t / 4, e.vy * t / 4);
                    let ts = &ships[e.idx];
                    ((along - e.half - 1).max(1)..=along + e.half + 1).any(|k| ts.cell_at(gx + dx * k - ox, gy + dy * k - oy).map_or(false, |c| ts.hp[c] > 0))
                });
                if in_line {
                    self.ships[i].aim_t = 0;
                    let line = Line::new(gx, gy, gx + dx * 400, gy + dy * 400);
                    self.shots.push(Shot { team, kind: Kind::Lance, line, left: LANCE_RANGE + 12, speed: LANCE_SPEED, dmg: LANCE_DMG, pen: LANCE_PEN, target: None, prev: (gx, gy), connected: false, born: self.tick });
                    self.ships[i].cool[c] = self.ships[i].st.reload(LANCE_PERIOD);
                    self.score[team].fired += 1;
                }
            }
        }
        for k in 0..self.ships[i].turrets.len() {
            self.cannon(i, k, snap, pending);
        }
    }

    /// A cannon turret swings slowly toward its target and fires once its barrel is on it. Fast
    /// ships close in change bearing faster than it turns.
    ///
    /// It goes for the big ships: anything in range at least 2/3 the size (as built) of the
    /// biggest enemy in range. Of those it takes the one its barrel reaches soonest, so among
    /// ships of a size it stays on the one it's already aimed at. (It used to take whichever had
    /// the most pixels left, so every hit sent it swinging round to an undamaged twin: the
    /// titan's cannons changed target more often than they fired at five capitals.)
    fn cannon(&mut self, i: usize, k: usize, snap: &[Snap], pending: &mut [i32]) {
        let (team, cell) = (self.ships[i].team, self.ships[i].turrets[k].cell);
        if self.ships[i].hp[cell] == 0 {
            return;
        }
        let (gx, gy) = self.ships[i].cell_xy(cell);
        {
            let t = &mut self.ships[i].turrets[k];
            t.cool = (t.cool - 1).max(0);
        }
        let in_range = || snap.iter().filter(|e| e.team != team && cheb(e.x, e.y, gx, gy) <= CANNON_RANGE + e.half);
        let Some(big) = in_range().map(|e| e.full).max() else {
            return;
        };
        let barrel = self.ships[i].turrets[k].aim;
        let Some(e) = in_range().filter(|e| e.full * 3 >= big * 2).min_by_key(|e| (angle_diff(barrel, angle_of(e.x - gx, e.y - gy)).abs(), dist2(e.x, e.y, gx, gy))).copied() else {
            return;
        };
        let stale = {
            let t = &self.ships[i].turrets[k];
            let ts = &self.ships[e.idx];
            t.aim_for != Some(e.idx) || ts.cell_at(e.x + t.aim_off.0, e.y + t.aim_off.1).map_or(true, |c| ts.hp[c] == 0)
        };
        if stale {
            let (px, py) = self.random_cell(e.idx);
            let t = &mut self.ships[i].turrets[k];
            t.aim_off = (px - e.x, py - e.y);
            t.aim_for = Some(e.idx);
        }
        let reload = self.ships[i].st.reload(CANNON_PERIOD);
        let t = &mut self.ships[i].turrets[k];
        let (ax, ay) = (e.x + t.aim_off.0, e.y + t.aim_off.1);
        let travel = isqrt(dist2(ax, ay, gx, gy)) / SHELL_SPEED;
        let (lx, ly) = (ax + e.vx * travel / 4, ay + e.vy * travel / 4);
        let want = angle_of(lx - gx, ly - gy);
        let off = angle_diff(t.aim, want);
        if off != 0 {
            t.turn_t += 1;
            if t.turn_t >= TURRET_TURN {
                t.turn_t = 0;
                t.aim = (t.aim + off.signum()).rem_euclid(64);
            }
        }
        if t.cool == 0 && angle_diff(t.aim, want).abs() <= 1 && (lx, ly) != (gx, gy) {
            // The barrel only has 64 angles, about 8 px apart at 85 px, so a shell sent along
            // the barrel slips past small targets forever. Once the barrel is on target, the
            // shell flies at the aim point itself (a 4 px barrel can't show the difference).
            let line = Line::new(gx, gy, lx, ly);
            t.cool = reload;
            t.aim_for = None;
            self.shots.push(Shot { team, kind: Kind::Shell, line, left: CANNON_RANGE + 12, speed: SHELL_SPEED, dmg: CANNON_DMG, pen: 1, target: Some(e.idx), prev: (gx, gy), connected: false, born: self.tick });
            pending[e.idx] += 1;
            self.score[team].fired += 1;
        }
    }

    /// Knocks one pixel off a ship; it drifts away as a loose pixel.
    fn kill_cell(&mut self, id: usize, c: usize, (x, y): (i32, i32), (vx, vy): (i32, i32)) {
        let s = &mut self.ships[id];
        if s.hp[c] == 0 {
            return;
        }
        s.hp[c] = 0;
        s.last_hit = self.tick;
        s.dirty = true;
        let team = s.team;
        self.score[team].px_lost += 1;
        self.last_loss = self.tick;
        let drift = 2 + self.rng.below(4) as i32;
        self.debris.push(Debris { x, y, vx, vy, drift, taken: false });
    }

    /// Moves every shot and works out what it hits. A shot is tested against each enemy ship in
    /// that ship's own frame: the path it takes relative to the ship this tick, from where it was
    /// before the ship moved or turned to where it ends up, walked a pixel at a time. Walked like
    /// that, a shot always meets a ship's outer pixels first, so no move or turn can carry a shot
    /// past plate to what the plate guards.
    fn shots_step(&mut self) {
        let shots = std::mem::take(&mut self.shots);
        let boxes: Vec<(usize, usize, (i32, i32, i32, i32))> = self.ships.iter().enumerate().filter(|(_, s)| s.alive).map(|(i, s)| (i, s.team, s.bbox())).collect();
        let mut keep = Vec::with_capacity(shots.len());
        // In tests: the path of every shot that flew on, to check it skipped nothing.
        let mut flown: Vec<(Vec<(i32, i32)>, usize, u32)> = Vec::new();
        'shot: for mut sh in shots {
            // The shot's own flight this tick, a pixel at a time.
            let mut path = vec![(sh.line.x, sh.line.y)];
            let mut gone = false;
            for _ in 0..sh.speed {
                sh.line.step();
                sh.left -= 1;
                let (nx, ny) = (sh.line.x, sh.line.y);
                if sh.left <= 0 || nx < 0 || ny < 0 || nx >= self.w || ny >= self.h {
                    gone = true;
                    break;
                }
                path.push((nx, ny));
            }
            let m = path.len() as i32 - 1;
            let (s0, s1) = (path[0], path[m as usize]);
            let (pl, pt) = (path.iter().map(|p| p.0).min().unwrap(), path.iter().map(|p| p.1).min().unwrap());
            let (pr, pb) = (path.iter().map(|p| p.0).max().unwrap(), path.iter().map(|p| p.1).max().unwrap());
            // Every living enemy pixel the shot meets, with when it meets it (thousandths of the
            // tick). Diagonal steps also meet the two pixels they squeeze between, so a staircase
            // of plate (pixels touching only at corners) stops shots like the wall it looks.
            let mut meets: Vec<(i32, usize, usize)> = Vec::new();
            for (id, s) in self.ships.iter().enumerate() {
                if !s.alive || s.team == sh.team || s.warp > 0 {
                    continue;
                }
                // A shot fired this tick was fired after the ships moved.
                let (x0, y0, f0) = if sh.born == self.tick { (s.x, s.y, s.facing) } else { s.pose0 };
                let r = s.half() + 1;
                if pr < s.x.min(x0) - r || pb < s.y.min(y0) - r || pl > s.x.max(x0) + r || pt > s.y.max(y0) + r {
                    continue;
                }
                let mut meet = |t: i32, c: Option<usize>| {
                    if let Some(c) = c.filter(|&c| s.hp[c] > 0) {
                        meets.push((t, id, c));
                    }
                };
                // A shot that starts the tick inside a living pixel meets that pixel first. It can
                // only be there because the pixel grew or arrived round it, or because it was fired
                // from inside: either way it can't skip the pixel it's in.
                meet(0, s.shape.cell_at(x0, y0, f0, s0.0, s0.1));
                if (x0, y0, f0) == (s.x, s.y, s.facing) {
                    // The ship held still: the relative path is the shot's own.
                    for j in 1..=m {
                        let ((px, py), (nx, ny)) = (path[j as usize - 1], path[j as usize]);
                        let t = j * 1000 / m;
                        if px != nx && py != ny {
                            meet(t, s.cell_at(nx, py));
                            meet(t, s.cell_at(px, ny));
                        }
                        meet(t, s.cell_at(nx, ny));
                    }
                } else {
                    // The ship moved or turned: walk the relative path in the ship's own grid.
                    let a = s.shape.to_local(x0, y0, f0, s0.0, s0.1);
                    let b = s.shape.to_local(s.x, s.y, s.facing, s1.0, s1.1);
                    let n = (b.0 - a.0).abs().max((b.1 - a.1).abs());
                    let mut l = Line::new(a.0, a.1, b.0, b.1);
                    for j in 1..=n {
                        let (px, py) = (l.x, l.y);
                        l.step();
                        let (nx, ny) = (l.x, l.y);
                        let t = j * 1000 / n;
                        if px != nx && py != ny {
                            meet(t, s.shape.local_cell(nx, py));
                            meet(t, s.shape.local_cell(px, ny));
                        }
                        meet(t, s.shape.local_cell(nx, ny));
                    }
                }
            }
            meets.sort_by_key(|&(t, _, _)| t);
            for (t, id, c) in meets {
                if self.ships[id].hp[c] == 0 {
                    continue;
                }
                // How far along its own flight the shot is when it meets this pixel.
                let k = ((t * m + 999) / 1000).clamp(1, m.max(1)) as usize;
                let before = path[(k - 1).min(path.len() - 1)];
                let at = self.ships[id].cell_xy(c);
                if !sh.connected {
                    sh.connected = true;
                    self.score[sh.team].hits += 1;
                }
                self.ships[id].last_hit = self.tick;
                let armor = self.ships[id].shape.cells[c].map_or(0, |p| p.armor());
                if sh.dmg <= armor {
                    // Too weak for plate: it bounces off.
                    self.score[sh.team].bounced += 1;
                    self.sparks.push(Spark { x: before.0, y: before.1, t: 2, blast: false });
                    continue 'shot;
                }
                if sh.kind == Kind::Shell {
                    self.blast(at.0, at.1, sh.team, &boxes);
                    continue 'shot;
                }
                self.kill_cell(id, c, at, sh.line.dir());
                sh.pen -= 1 + armor;
                if sh.pen <= 0 {
                    continue 'shot;
                }
            }
            if gone {
                continue;
            }
            sh.prev = path[(m as usize).saturating_sub(1)];
            if cfg!(test) {
                flown.push((path, sh.team, sh.born));
            }
            keep.push(sh);
        }
        keep.append(&mut self.shots);
        self.shots = keep;
        if cfg!(test) {
            if let Err(what) = self.audit_shots().and_then(|_| self.audit_paths(&flown)) {
                panic!("tick {}: {what}", self.tick);
            }
        }
    }

    /// A shell bursts: every enemy pixel in the 3x3 around the hit goes, plate and all.
    fn blast(&mut self, x: i32, y: i32, team: usize, boxes: &[(usize, usize, (i32, i32, i32, i32))]) {
        self.sparks.push(Spark { x, y, t: 3, blast: true });
        for dy in -1..=1 {
            for dx in -1..=1 {
                let (px, py) = (x + dx, y + dy);
                for &(id, t, (l, tp, r, b)) in boxes {
                    if t == team || px < l || py < tp || px > r || py > b {
                        continue;
                    }
                    if let Some(c) = self.ships[id].cell_at(px, py) {
                        if self.ships[id].shape.cells[c].map_or(0, |p| p.armor()) < CANNON_DMG {
                            self.kill_cell(id, c, (px, py), (dx, dy));
                        }
                    }
                }
            }
        }
    }

    /// After a ship is cut apart: every piece that still holds a core flies on as a ship of its
    /// own (the biggest stays the original ship), and pieces with no core break away as loose
    /// pixels.
    fn break_off(&mut self) {
        for i in 0..self.ships.len() {
            let s = &self.ships[i];
            if !s.alive || !s.dirty || !s.has_core() {
                continue;
            }
            let (w, h) = (s.shape.w, s.shape.h);
            let mut seen = vec![false; s.hp.len()];
            let (mut cored, mut loose): (Vec<Vec<usize>>, Vec<usize>) = (Vec::new(), Vec::new());
            for c in 0..s.hp.len() {
                if s.hp[c] == 0 || seen[c] {
                    continue;
                }
                let reach = reach(w, h, c, |k| s.hp[k] > 0);
                let cells: Vec<usize> = (0..reach.len()).filter(|&k| reach[k]).collect();
                for &k in &cells {
                    seen[k] = true;
                }
                if cells.iter().any(|&k| s.shape.is_core(k)) {
                    cored.push(cells);
                } else {
                    loose.extend(cells);
                }
            }
            let (team, x, y) = (s.team, s.x, s.y);
            for c in loose {
                let (px, py) = self.ships[i].cell_xy(c);
                self.ships[i].hp[c] = 0;
                self.score[team].px_lost += 1;
                self.score[team].broken += 1;
                let drift = 3 + self.rng.below(5) as i32;
                self.debris.push(Debris { x: px, y: py, vx: (px - x).signum(), vy: (py - y).signum(), drift, taken: false });
            }
            if cored.len() > 1 {
                // The biggest piece stays this ship; every other cored piece is a new one.
                cored.sort_by_key(|p| std::cmp::Reverse(p.len()));
                for p in &cored[1..] {
                    let new = self.ships[i].piece(p, self.tick);
                    for &c in p {
                        self.ships[i].hp[c] = 0;
                    }
                    self.ships.push(new);
                    self.score[team].placed += 1;
                    self.score[team].split += 1;
                }
            }
        }
    }

    fn debris_step(&mut self) {
        for d in &mut self.debris {
            if d.drift > 0 {
                d.x = (d.x + d.vx).clamp(0, self.w - 1);
                d.y = (d.y + d.vy).clamp(0, self.h - 1);
                d.drift -= 1;
            }
        }
        // Scoops grab loose pixels they brush past, whatever the ship's order.
        for s in self.ships.iter_mut().filter(|s| s.alive && s.st.scoops > 0) {
            let cap = s.st.scoops * SCOOP_CARRY;
            for d in self.debris.iter_mut().filter(|d| !d.taken) {
                if s.carry < cap && cheb(d.x, d.y, s.x, s.y) <= s.half() + 1 {
                    d.taken = true;
                    s.carry += 1;
                }
            }
        }
        self.debris.retain(|d| !d.taken);
        if self.debris.len() > 3000 {
            let n = self.debris.len() - 3000;
            self.debris.drain(0..n);
        }
    }

    /// A ship whose last core is gone bursts into loose pixels.
    fn deaths(&mut self) {
        for i in 0..self.ships.len() {
            let s = &self.ships[i];
            if !s.alive || s.has_core() {
                continue;
            }
            let (team, x, y) = (s.team, s.x, s.y);
            let cells: Vec<(i32, i32)> = (0..s.hp.len()).filter(|&c| s.hp[c] > 0).map(|c| s.cell_xy(c)).collect();
            self.ships[i].alive = false;
            self.score[team].lost += 1;
            self.score[team].px_lost += cells.len() as i32;
            for (px, py) in cells {
                let drift = 2 + self.rng.below(5) as i32;
                self.debris.push(Debris { x: px, y: py, vx: (px - x).signum(), vy: (py - y).signum(), drift, taken: false });
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn run(a: &mut Arena, max: u32) {
        while a.over.is_none() && a.tick < max {
            a.step();
        }
    }

    fn fight(blue: &[(usize, i32, i32)], red: &[(usize, i32, i32)]) -> Arena {
        let lib = defaults();
        let mut a = Arena::new();
        for &(slot, x, y) in blue {
            assert!(a.add(0, slot, &lib[slot], x, y));
        }
        for &(slot, x, y) in red {
            assert!(a.add(1, slot, &lib[slot], x, y));
        }
        run(&mut a, TIME_LIMIT);
        a
    }

    #[test]
    fn turret_angles_point_the_right_way() {
        assert_eq!(dir64(0), (0, -1024));
        assert_eq!(dir64(16), (1024, 0));
        assert_eq!(dir64(32), (0, 1024));
        assert_eq!(dir64(48), (-1024, 0));
        assert_eq!(angle_of(10, 0), 16);
        assert_eq!(angle_of(-5, -5), 56);
        assert_eq!(angle_diff(62, 2), 4);
        assert_eq!(angle_diff(2, 62), -4);
    }

    #[test]
    fn fights_end() {
        let a = fight(&[(0, 150, 160), (0, 150, 180), (0, 150, 200), (0, 150, 220)], &[(1, 330, 160), (1, 330, 180), (1, 330, 200), (1, 330, 220)]);
        assert!(a.over.is_some() && a.tick < TIME_LIMIT, "fighters vs lancers timed out");
    }

    #[test]
    fn illegal_designs_are_refused() {
        let mut a = Arena::new();
        let mut d = Design::from_rows("LOOSE", Order::Hunt, &["g.g", "hch", "e.e"]);
        d.set(0, 0, Some(Part::Hull));
        assert!(!a.add(0, 0, &d, 100, 100));
    }

    #[test]
    fn flak_bounces_off_plate_and_lances_break_it() {
        let slab = Design::from_rows("SLAB", Order::Hold, &["ppppp", "ppppp", "ppcpp", "ppppp", "ppppp"]);
        let lib = defaults();
        let mut a = Arena::new();
        a.add(1, 0, &slab, 300, 180);
        for k in 0..4 {
            a.add(0, 0, &lib[0], 150, 150 + k * 20);
        }
        run(&mut a, 15 * 20);
        assert_eq!(a.score[1].px_lost, 0, "flak broke plate");
        assert!(a.score[0].bounced > 10, "only {} bounces", a.score[0].bounced);

        let mut a = Arena::new();
        a.add(1, 0, &slab, 300, 180);
        for k in 0..2 {
            a.add(0, 1, &lib[1], 150, 170 + k * 20);
        }
        run(&mut a, 15 * 60);
        assert_eq!(a.over, Some(Some(0)), "lancers never cracked the slab");
    }

    #[test]
    fn a_cannon_shell_blasts_a_hole_through_plate() {
        let slab = Design::from_rows("SLAB", Order::Hold, &["ppppppp", "ppppppp", "ppppppp", "pppcppp", "ppppppp", "ppppppp", "ppppppp"]);
        let gun = Design::from_rows("GUN", Order::Hold, &["nnn", "nnn", "nnn", ".c."]);
        let mut a = Arena::new();
        a.add(0, 0, &gun, 200, 180);
        a.add(1, 0, &slab, 290, 180);
        while a.score[0].hits == 0 && a.tick < 15 * 10 {
            a.step();
        }
        a.step();
        assert!(a.score[1].px_lost >= 4, "one shell took only {} px", a.score[1].px_lost);
    }

    #[test]
    fn a_cannon_hits_small_targets_that_sit_still() {
        // Gunless, engineless hulks at odd bearings: the cannon must not keep missing between
        // its 64 barrel angles.
        let gun = Design::from_rows("GUN", Order::Hold, &["nnn", "nnn", "nnn", ".c."]);
        let hulk = Design::from_rows("HULK", Order::Hold, &["hch"]);
        let mut a = Arena::new();
        a.add(0, 0, &gun, 240, 180);
        for (dx, dy) in [(90, 7), (70, 52), (-83, 31), (5, -101), (-61, -73), (47, -88)] {
            a.add(1, 0, &hulk, 240 + dx, 180 + dy);
        }
        run(&mut a, 15 * 90);
        assert_eq!(a.over, Some(Some(0)), "{} of 6 hulks still alive after 90 s", a.alive(1));
    }

    /// Runs `a` for `ticks` and counts, over every ship smaller than some enemy: ticks spent
    /// bumping into a bigger ship, ticks spent overlapping one, and the longest overlap in a row.
    fn crashes(a: &mut Arena, ticks: u32) -> (u32, u32, u32) {
        let n = a.ships.len();
        let (mut bumps, mut inside, mut longest) = (0, 0, 0);
        let mut run = vec![0; n];
        for _ in 0..ticks {
            if a.over.is_some() {
                break;
            }
            a.step();
            for i in 0..n {
                let s = &a.ships[i];
                if !s.alive {
                    continue;
                }
                bumps += s.bumped as u32;
                if a.overlap(i, s.x, s.y, s.facing).0 > 0 {
                    inside += 1;
                    run[i] += 1;
                    longest = longest.max(run[i]);
                } else {
                    run[i] = 0;
                }
            }
        }
        (bumps, inside, longest)
    }

    #[test]
    fn small_ships_never_stay_inside_big_ones() {
        let lib = defaults();
        let mut a = Arena::new();
        for &(team, slot, x, y) in &crate::pg::reference_tests()[4].place {
            a.add(team, slot, &lib[slot], x, y);
        }
        let (bumps, inside, longest) = crashes(&mut a, 15 * 60);
        println!("capital vs 30 fighters: {bumps} bump ticks, {inside} ticks inside, longest {longest}");
        assert!(longest <= 3, "a fighter spent {longest} ticks in a row inside the capital");
    }

    #[test]
    fn attack_runs_peel_off_before_the_hull() {
        // A big, unarmed slab: fighters dive at it again and again and should pass alongside,
        // not plough into the plate and skim.
        let mut rows = vec!["ppppppppppppppp"; 15];
        rows[7] = "pppppppcppppppp";
        let slab = Design::from_rows("SLAB", Order::Hold, &rows);
        let lib = defaults();
        let mut a = Arena::new();
        a.add(1, 0, &slab, 300, 180);
        for k in 0..6 {
            a.add(0, 0, &lib[0], 150, 130 + k * 20);
        }
        let (bumps, inside, longest) = crashes(&mut a, 15 * 40);
        println!("6 fighters vs slab: {bumps} bump ticks, {inside} ticks inside, longest {longest}");
        assert!(bumps <= 30, "fighters bumped the slab for {bumps} ticks");
        assert_eq!(inside, 0);
    }

    #[test]
    fn lance_ships_line_up_before_they_dive() {
        // A lance fires only straight ahead. Any drawing with one, wherever the lance sits, should
        // slide onto the target's row or column during its attack run and fire from range, not
        // dive at the centre and come into line only when it's about to pass.
        let slab = Design::from_rows("SLAB", Order::Hold, &["ppppp", "ppppp", "ppcpp", "ppppp", "ppppp"]);
        let ships = [
            Design::from_rows("LANCER", Order::Hunt, &[".h.", ".l.", "hch", "e.e"]),
            Design::from_rows("OFFSET", Order::Hunt, &["l..", "h..", "hch", "e.e"]),
            Design::from_rows("WIDE", Order::Hunt, &["..l..", ".hhh.", "hhchh", "e...e"]),
        ];
        for d in &ships {
            for (sx, sy) in [(200, 150), (200, 215), (330, 80)] {
                let mut a = Arena::new();
                a.add(1, 0, &slab, 300, 180);
                assert!(a.add(0, 1, d, sx, sy));
                let mut first = None;
                while a.tick < 15 * 10 && first.is_none() {
                    a.step();
                    if a.score[0].fired > 0 {
                        first = Some(isqrt(dist2(a.ships[1].x, a.ships[1].y, 300, 180)));
                    }
                }
                let first = first.unwrap_or_else(|| panic!("{} from ({sx},{sy}) never fired", d.name));
                let until = a.tick + 15 * 2;
                run(&mut a, until);
                println!("{} from ({sx},{sy}): first shot from {first} px, {} hits", d.name, a.score[0].hits);
                assert!(first >= 40, "{} from ({sx},{sy}) first fired from only {first} px", d.name);
                assert!(a.score[0].hits > 0, "{} from ({sx},{sy}) missed", d.name);
            }
        }
    }

    /// Where every living pixel of a team's ships is.
    fn pixels(a: &Arena, team: usize) -> Vec<(i32, i32)> {
        let mut v: Vec<(i32, i32)> = a.ships.iter().filter(|s| s.alive && s.team == team).flat_map(|s| (0..s.hp.len()).filter(|&c| s.hp[c] > 0).map(|c| s.cell_xy(c)).collect::<Vec<_>>()).collect();
        v.sort();
        v
    }

    fn dumbbell() -> Design {
        // Two gun pods, each with a core and engines, on a thin spine.
        Design::from_rows("DUMBBELL", Order::Hunt, &["g.g.....g.g", "hchhhhhhhch", "e.e.....e.e"])
    }

    #[test]
    fn a_cut_ship_with_a_core_in_each_piece_flies_on_as_two() {
        let d = dumbbell();
        assert!(d.check().ok(), "{:?}", d.check());
        assert_eq!(d.stats().cost, 19 + 8);
        let mut a = Arena::new();
        a.add(0, 0, &d, 200, 180);
        // Cut the spine in the middle.
        let s = &a.ships[0];
        let mid = (0..s.shape.cells.len()).find(|&c| c as i32 % s.shape.w == 5 && s.shape.cells[c].is_some()).unwrap();
        let at = s.cell_xy(mid);
        a.kill_cell(0, mid, at, (0, 1));
        let before = pixels(&a, 0);
        a.break_off();
        a.deaths();
        assert_eq!(a.alive(0), 2, "the two halves should fly on as two ships");
        assert_eq!(pixels(&a, 0), before, "no pixel moved or vanished in the split");
        assert_eq!((a.score[0].split, a.score[0].placed), (1, 2));
        for s in a.ships.iter().filter(|s| s.alive) {
            let st = stats_of(&s.alive_cells(), 0);
            assert_eq!((st.cores, st.flak, st.engines, st.count), (1, 2, 2, 9), "{:?}", s.alive_cells());
        }
        // The original refreshes its stats at the end of the tick; the new ship starts with its own.
        assert_eq!((a.ships[1].st.flak, a.ships[1].st.engines, a.ships[1].px_full()), (2, 2, 9));
        // Each half fights on: run it against a fighter and see both shoot.
        let lib = defaults();
        a.add(1, 0, &lib[0], 260, 180);
        run(&mut a, 15 * 20);
        assert!(a.score[0].fired >= 4, "the halves fired {} shots", a.score[0].fired);
    }

    /// A flak shot from team 0 at (x, y), flying toward (tx, ty), fired on an earlier tick.
    fn flak_at(a: &Arena, (x, y): (i32, i32), (tx, ty): (i32, i32)) -> Shot {
        let line = Line::new(x, y, tx, ty);
        Shot { team: 0, kind: Kind::Flak, line, left: FLAK_RANGE, speed: FLAK_SPEED, dmg: FLAK_DMG, pen: 1, target: None, prev: (x, y), connected: false, born: a.tick.saturating_sub(1) }
    }

    #[test]
    fn moving_onto_a_shot_doesnt_let_it_skip_the_plate() {
        // The case Henry saw: a flak shot rests just outside a pod's plate, heading for its gun,
        // and the pod moves 1 px onto it. The shot has to meet the plate and bounce.
        let pod = Design::from_rows("POD", Order::Hold, &["ppp", "pgp", "pcp", "ppp"]);
        let mut a = Arena::new();
        a.add(1, 0, &pod, 200, 180);
        a.tick = 5;
        let s = &a.ships[0];
        let gun = (0..s.hp.len()).find(|&c| s.shape.cells[c] == Some(Part::Flak)).unwrap();
        let (gx, gy) = s.cell_xy(gun);
        // The plate beside the gun, and the empty spot just beyond it.
        let (dx, dy) = [(1, 0), (-1, 0), (0, 1), (0, -1)].into_iter().find(|&(dx, dy)| s.cell_at(gx + dx, gy + dy).map_or(false, |c| s.shape.cells[c] == Some(Part::Plate)) && s.cell_at(gx + 2 * dx, gy + 2 * dy).is_none()).unwrap();
        let spot = (gx + 2 * dx, gy + 2 * dy);
        let shot = flak_at(&a, spot, (spot.0 - dx * 50, spot.1 - dy * 50));
        a.shots.push(shot);
        let s = &mut a.ships[0];
        s.pose0 = (s.x, s.y, s.facing);
        s.x += dx;
        s.y += dy;
        a.shots_step();
        assert_eq!(a.ships[0].hp[gun], 1, "the shot skipped the plate and killed the gun");
        assert_eq!(a.score[0].bounced, 1, "it should bounce off the plate");
    }

    #[test]
    fn flak_never_reaches_a_gun_sealed_in_plate() {
        // Flak can't break plate, so a gun and core sealed inside a ring of plate must survive any
        // flak shot: from every spot near the pod, flying any of 8 ways, whichever way the pod
        // faces, however it moves (up to 3 px) and whether or not it turns in the same tick.
        let pod = Design::from_rows("POD", Order::Hold, &["ppp", "pgp", "pcp", "ppp"]);
        let mut a = Arena::new();
        a.add(1, 0, &pod, 200, 180);
        a.tick = 5;
        let inner: Vec<usize> = (0..a.ships[0].hp.len()).filter(|&c| a.ships[0].shape.cells[c] != Some(Part::Plate)).collect();
        let mut tried = 0;
        for f0 in 0..4u8 {
            for turn in [0u8, 1, 3] {
                for (mx, my) in [(0, 0), (1, 0), (0, -1), (-2, 1), (3, 0), (0, 3), (-3, -2)] {
                    for sy in 172..=188 {
                        for sx in 192..=208 {
                            for (dx, dy) in [(1, 0), (-1, 0), (0, 1), (0, -1), (1, 1), (1, -1), (-1, 1), (-1, -1), (2, 1), (-1, 3)] {
                                let s = &mut a.ships[0];
                                s.hp.iter_mut().for_each(|h| *h = 1);
                                (s.x, s.y, s.facing) = (200, 180, f0);
                                // Shots never start inside a ship.
                                if s.cell_at(sx, sy).is_some() {
                                    continue;
                                }
                                s.pose0 = (200, 180, f0);
                                (s.x, s.y, s.facing) = (200 + mx, 180 + my, (f0 + turn) % 4);
                                a.shots.clear();
                                let shot = flak_at(&a, (sx, sy), (sx + dx * 50, sy + dy * 50));
                                a.shots.push(shot);
                                a.shots_step();
                                tried += 1;
                                for &c in &inner {
                                    assert_eq!(a.ships[0].hp[c], 1, "a shot from ({sx},{sy}) going ({dx},{dy}) reached the inside: facing {f0} turning {turn}, moving ({mx},{my})");
                                }
                            }
                        }
                    }
                }
            }
        }
        assert!(tried > 50_000, "only {tried} cases");
    }

    #[test]
    fn a_hunter_with_no_guns_stays_with_its_group() {
        // A gunless hunter far from its armed friend joins it instead of heading home, and stays
        // beside it even after being hit.
        let turret = Design::from_rows("TURRET", Order::Hold, &["ggg", "hch"]);
        let gunless = Design::from_rows("SPARE", Order::Hunt, &["hch", "e.e"]);
        let slab = Design::from_rows("SLAB", Order::Hold, &["ppp", "pcp", "ppp"]);
        let mut a = Arena::new();
        a.add(0, 0, &turret, 240, 100);
        a.add(0, 1, &gunless, 80, 280);
        a.add(1, 0, &slab, 440, 300);
        run(&mut a, 15 * 15);
        let (f, s) = (&a.ships[0], &a.ships[1]);
        assert_eq!(s.doing, Doing::Escort);
        assert!(cheb(s.x, s.y, f.x, f.y) <= f.half() + s.half() + 6, "still {} px away", cheb(s.x, s.y, f.x, f.y));
        a.ships[1].last_hit = a.tick;
        let until = a.tick + 15 * 3;
        run(&mut a, until);
        let (f, s) = (&a.ships[0], &a.ships[1]);
        assert!(cheb(s.x, s.y, f.x, f.y) <= f.half() + s.half() + 6, "ran off when hit");
    }

    /// Five miners put on one spot at the west face of the middle rock, between that rock and
    /// blue's station, and given a second to settle to their bites. Returns the spot.
    fn miners_at_work(a: &mut Arena) -> (i32, i32) {
        let miner = Design::from_rows("MINER", Order::Harvest, &["sss", "hch", "e.e"]);
        let at = (AW / 2 - 17, AH / 2);
        for _ in 0..5 {
            assert!(a.add(0, 0, &miner, at.0, at.1));
        }
        run(a, 15);
        assert!(a.ships.iter().all(|s| s.doing == Doing::Mine && (s.vx, s.vy) == (0, 0) && cheb(s.x, s.y, at.0, at.1) <= 2), "the miners should be at work in a heap");
        at
    }

    #[test]
    fn a_full_miner_flies_straight_home_through_its_friends() {
        // A miner fills up on the far side of a heap of miners still working, with three parked
        // blocks of its own side heaped on the same spot, and a picket holding a place further
        // along its line home. It takes no part in keeping apart: it flies the line through all
        // of them in the time the distance takes, and the picket it flies over doesn't stir.
        // (Pushed like any other ship, it hung behind the heap, a few pixels back and forth,
        // until the miners in it had filled up too: over a minute, in a match.)
        let mut a = Arena::new();
        let at = miners_at_work(&mut a);
        let block = Design::from_rows("BLOCK", Order::Hold, &["hch"]);
        for _ in 0..3 {
            assert!(a.add(0, 1, &block, at.0, at.1));
        }
        let picket = Design::from_rows("PICKET", Order::Hold, &["ggg", "hch", "e.e"]);
        assert!(a.add(0, 2, &picket, 60, at.1));
        let p = a.ships.len() - 1;
        let miner = Design::from_rows("MINER", Order::Harvest, &["sss", "hch", "e.e"]);
        assert!(a.add(0, 0, &miner, at.0 + 3, at.1));
        let m = a.ships.len() - 1;
        a.ships[m].carry = a.ships[m].st.scoops * SCOOP_CARRY;
        let far = at.0 + 3 - STATIONS[0].0 - 10;
        let until = a.tick + (far * 4 / a.ships[m].st.vmax + 30) as u32;
        while a.tick < until {
            a.step();
            assert_eq!((a.ships[p].x, a.ships[p].y), (60, at.1), "tick {}: the picket was pushed off its spot", a.tick);
        }
        assert_eq!(a.score[0].mined, 9, "the full miner isn't home after the flight and 2 s: it's at {:?}, {:?}", (a.ships[m].x, a.ships[m].y), a.ships[m].doing);
    }

    #[test]
    fn a_heap_of_miners_at_work_is_no_wall_to_their_own_side() {
        // Miners working one rock face sit on top of each other and give way to nothing. A slow,
        // heavy ship of their side sent to a spot beyond them flies through the heap and gets
        // there. (When each miner pushed it like any other friend, five pushes beat its engine:
        // it stopped short of them, was pushed back out, and came on again, for as long as they
        // worked. Something light and fast got through on its own momentum.)
        let mut a = Arena::new();
        let at = miners_at_work(&mut a);
        let tug = Design::from_rows("TUG", Order::Hold, &["ppp", "pcp", "ppp", ".e."]);
        assert!(a.add(0, 1, &tug, at.0 + 10, at.1));
        let r = a.ships.len() - 1;
        let spot = (at.0 - 20, at.1);
        a.ships[r].home = spot;
        let until = a.tick + 15 * 10;
        run(&mut a, until);
        assert!(cheb(a.ships[r].x, a.ships[r].y, spot.0, spot.1) <= 2, "the tug didn't get through the miners in 10 s: it's {} px short", a.ships[r].x - spot.0);
    }

    #[test]
    fn a_ship_lives_until_its_last_core_goes() {
        let d = Design::from_rows("TWIN", Order::Hold, &["hcch", "e..e"]);
        assert!(d.check().ok(), "{:?}", d.check());
        let mut a = Arena::new();
        a.add(0, 0, &d, 200, 180);
        let cores: Vec<usize> = (0..a.ships[0].hp.len()).filter(|&c| a.ships[0].shape.is_core(c)).collect();
        for (k, &c) in cores.iter().enumerate() {
            let at = a.ships[0].cell_xy(c);
            a.kill_cell(0, c, at, (0, 1));
            a.break_off();
            a.deaths();
            assert_eq!(a.ships[0].alive, k + 1 < cores.len(), "after losing {} of {} cores", k + 1, cores.len());
        }
    }

    #[test]
    fn a_stalemate_ends_early_as_a_draw() {
        // Two ships with no engines, out of each other's range, after a first hit: nobody can
        // fire and nothing gets shot off, so it ends 20 s later instead of at the 3-minute limit.
        let turret = Design::from_rows("TURRET", Order::Hold, &["ggg", "hch"]);
        let hulk = Design::from_rows("HULK", Order::Hold, &["hhh", "hch"]);
        let mut a = Arena::new();
        a.add(0, 0, &turret, 100, 180);
        a.add(1, 0, &hulk, 380, 180);
        let at = a.ships[1].cell_xy(0);
        a.step();
        a.kill_cell(1, 0, at, (0, 1));
        let hit = a.tick;
        run(&mut a, TIME_LIMIT);
        assert_eq!(a.over, Some(None));
        assert!(a.stalemate, "should end as a stalemate, not on the clock");
        assert_eq!(a.tick, hit + STALEMATE);
        assert!(a.disarmed(1) && !a.disarmed(0), "the hulk has no guns; the turret does");

        // Ships still shooting keep the fight going, even if every shot bounces off plate.
        let slab = Design::from_rows("SLAB", Order::Hold, &["..h..", "ppppp", "ppppp", "ppcpp", "ppppp", "ppppp"]);
        let lib = defaults();
        let mut a = Arena::new();
        a.add(1, 0, &slab, 300, 180);
        for k in 0..3 {
            a.add(0, 0, &lib[0], 150, 160 + k * 20);
        }
        run(&mut a, 15 * 60);
        assert!(a.score[1].px_lost > 0 && a.over.is_none(), "fighters plinking at plate aren't a stalemate yet");
    }

    #[test]
    fn a_severed_piece_breaks_away() {
        let stick = Design::from_rows("STICK", Order::Hold, &["h", "h", "h", "c"]);
        let mut a = Arena::new();
        a.add(0, 0, &stick, 200, 180);
        a.ships[0].hp[2] = 0;
        a.ships[0].dirty = true;
        a.step();
        assert_eq!(&a.ships[0].hp, &[0, 0, 0, 1]);
        assert_eq!(a.score[0].broken, 2);
        assert!(a.ships[0].alive);
    }

    #[test]
    fn flak_spreads_over_several_targets() {
        let battery = Design::from_rows("BATTERY", Order::Hold, &["ggg", "gcg", "ggg"]);
        let lib = defaults();
        let mut a = Arena::new();
        a.add(0, 0, &battery, 240, 180);
        for (x, y) in [(240, 145), (275, 180), (240, 215), (205, 180)] {
            a.add(1, 0, &lib[0], x, y);
            let s = a.ships.last_mut().unwrap();
            s.order = Order::Hold;
        }
        let mut targets = Vec::new();
        for _ in 0..12 {
            a.step();
            for sh in a.shots.iter().filter(|s| s.team == 0) {
                if let Some(t) = sh.target {
                    if !targets.contains(&t) {
                        targets.push(t);
                    }
                }
            }
        }
        assert!(targets.len() >= 3, "fire went to only {} of 4 targets", targets.len());
    }

    #[test]
    fn heavy_ships_turn_and_accelerate_slower() {
        let lib = defaults();
        let mut a = Arena::new();
        a.add(0, 0, &lib[0], 40, 40);
        a.add(0, 4, &lib[4], 40, 200);
        a.ships[0].order = Order::Hold;
        a.ships[1].order = Order::Hold;
        a.ships[0].home = (300, 40);
        a.ships[1].home = (300, 200);
        for _ in 0..15 {
            a.step();
        }
        assert!(a.ships[0].x - 40 > 3 * (a.ships[1].x - 40).max(1), "fighter {} capital {}", a.ships[0].x, a.ships[1].x);
    }

    #[test]
    fn an_armed_miner_shoots_back() {
        let a = fight(&[(0, 200, 180)], &[(5, 250, 180)]);
        assert!(a.score[1].hits > 0, "the gunminer never hit the fighter");
    }

    #[test]
    fn a_cannon_stays_on_its_target_among_equals() {
        // Henry, watching the titan against five capitals: its guns fired one volley at one ship,
        // then swung all the way round to another. A cannon facing two equal hulks must keep
        // shooting the one it's aimed at, however much the first hits take off it.
        let gun = Design::from_rows("GUN", Order::Hold, &["nnn", "nnn", "nnn", "hch"]);
        let hulk = Design::from_rows("HULK", Order::Hold, &["hhhhh", "hhhhh", "hhchh", "hhhhh", "hhhhh"]);
        let mut a = Arena::new();
        a.add(0, 0, &gun, 240, 180);
        a.add(1, 1, &hulk, 180, 180);
        a.add(1, 1, &hulk, 300, 180);
        let mut targets = Vec::new();
        while a.tick < 15 * 30 && a.ships[1..].iter().all(|s| s.alive) {
            a.step();
            targets.extend(a.shots.iter().filter(|s| s.kind == Kind::Shell && s.born == a.tick).map(|s| s.target));
        }
        assert!(targets.len() >= 3, "only {} shells", targets.len());
        assert!(targets.windows(2).all(|w| w[0] == w[1]), "it changed target: {targets:?}");
    }

    #[test]
    fn losing_hull_drains_power() {
        // Exactly powered: a cannon and an engine use 9; five hull and a core make 9. Shooting off
        // three hull pixels leaves 6 of 9, so the cannon reloads slower and the engine pushes less.
        let boat = Design::from_rows("BOAT", Order::Hold, &["nnn", "nnn", "nnn", "hch", "hhh", ".e."]);
        let mut a = Arena::new();
        a.add(0, 0, &boat, 240, 180);
        assert_eq!(a.ships[0].st.power_pct(), 100);
        let full = a.ships[0].st.thrust;
        for (x, y) in [(0, 3), (0, 4), (2, 4)] {
            let c = (y * a.ships[0].shape.w + x) as usize;
            assert_eq!(a.ships[0].shape.cells[c], Some(Part::Hull));
            let at = a.ships[0].cell_xy(c);
            a.kill_cell(0, c, at, (0, 0));
        }
        a.step();
        let st = a.ships[0].st;
        assert!(a.ships[0].alive);
        assert_eq!((st.made, st.used, st.power_pct()), (6, 9, 66));
        assert!(st.reload(CANNON_PERIOD) > CANNON_PERIOD && st.thrust < full, "{:?}", st);
    }

    #[test]
    fn a_shot_that_starts_inside_a_pixel_hits_that_pixel() {
        // A pixel can grow round a shot in flight, a ship can arrive round one, and a gun can
        // fire from inside an enemy. The shot can't skip the pixel it's in: on plate, flak bounces.
        let pod = Design::from_rows("POD", Order::Hold, &["ppp", "pgp", "pcp", "ppp"]);
        let mut a = Arena::new();
        a.add(1, 0, &pod, 200, 180);
        a.tick = 5;
        let s = &a.ships[0];
        let gun = (0..s.hp.len()).find(|&c| s.shape.cells[c] == Some(Part::Flak)).unwrap();
        let (gx, gy) = s.cell_xy(gun);
        let (dx, dy) = [(1, 0), (-1, 0), (0, 1), (0, -1)].into_iter().find(|&(dx, dy)| s.cell_at(gx + dx, gy + dy).map_or(false, |c| s.shape.cells[c] == Some(Part::Plate))).unwrap();
        // The shot sits on the plate beside the gun, heading through the gun.
        let shot = flak_at(&a, (gx + dx, gy + dy), (gx - dx * 50, gy - dy * 50));
        a.shots.push(shot);
        let s = &mut a.ships[0];
        s.pose0 = (s.x, s.y, s.facing);
        a.shots_step();
        assert_eq!(a.ships[0].hp[gun], 1, "the shot skipped the plate it started in and killed the gun");
        assert_eq!(a.score[0].bounced, 1, "it should bounce off the plate it's in");
    }
}
