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
/// How far a lance shot flies before it's spent: a little past the range the gun is given, so a
/// shot at something at the end of that range gets there. The gun fires at what the shot will
/// reach, which is this.
const LANCE_LIFE: i32 = LANCE_RANGE + 12;
const SHELL_SPEED: i32 = 4;
/// A repeater's shot: as quick as flak's.
const REPEAT_SPEED: i32 = 6;
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
/// A ship keeps the enemy it's after until another is this many pixels nearer.
pub const KEEP: i32 = 30;
/// How far round its circle an orbiting ship aims, in pixels.
const ORBIT_LEAD: i32 = 12;
/// How far beyond clear of a warming beam's line a ship still won't fly back towards it.
const DODGE_ROOM: i32 = 4;
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

#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash)]
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
    Orbit,
    /// Waiting at its post for the rest of its group.
    Muster,
    /// Getting off the line of an enemy beam that's warming.
    Dodge,
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
            Doing::Escort => "NO GUNS: WITH ITS GROUP",
            Doing::WithKing => "ESCORTING THE KING",
            Doing::ToRock => "FLYING TO WHERE IT'LL GROW",
            Doing::Orbit => "ORBITING",
            Doing::Muster => "WAITING FOR ITS GROUP",
            Doing::Dodge => "DODGING A BEAM",
        }
    }
}

/// A cannon: a 3x3 block whose centre pixel is the breech. It works while that pixel survives.
#[derive(Clone, Debug, Hash)]
pub struct Turret {
    pub cell: usize,
    /// Where the barrel points, in 64ths of a turn (0 north, clockwise).
    pub aim: i32,
    cool: i32,
    turn_t: u32,
    aim_off: (i32, i32),
    aim_for: Option<usize>,
}

#[derive(Clone, Debug, Hash)]
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
    /// Per pixel: a gun's ticks until it can fire. A beam gun's muzzle counts up from below zero
    /// while it warms. A shield pixel's is its ticks of rest still to go.
    cool: Vec<i32>,
    /// Per shield pixel: the shots its circle will still stop. 0 everywhere else.
    pub charge: Vec<u8>,
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
    /// The speed it's trying to fly at, in quarter pixels a tick: what its brain (or its king's
    /// keys) last asked for, no faster than it can go. While that's the speed it has, it's flying
    /// steadily. While it isn't, it's speeding up, slowing or turning towards it.
    want: (i32, i32),
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
    /// A seed sent to a spot in open space, not to a rock: it comes down exactly there.
    pub sow: bool,
    /// Its projector's pool: the shots its skin can still stop, and the ticks of rest left
    /// before the pool is full again (a charge for each projector pixel it has standing).
    pub skin: (i32, i32),
    /// Planted: grown out of a rock and still standing where its seed came down. A factory stays
    /// planted for good; a seeded design with engines stops being planted the moment it flies.
    /// Anything planted is solid to every enemy ship (see `sim/space.rs`).
    pub rooted: bool,
    /// How it flies at what it's fighting, if its design had that chosen. None: as its drawing
    /// (what's left of it) suggests.
    pub style: Option<Style>,
    /// The enemy it's after. It keeps it until that one is gone, or out of what it may go for, or
    /// another is `KEEP` px nearer.
    pub prey: Option<usize>,
    /// Which way round it orbits: 1 or -1, and 0 until it has picked.
    spin: i8,
    /// On HUNT or SAB in a match, still waiting at its post for the rest of its group.
    pub waiting: bool,
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
    /// Its beam guns as they stand: the muzzle cell and the length of each.
    pub fn beam_guns(&self) -> Vec<(usize, i32)> {
        beam_guns(self.shape.w, self.shape.h, |c| self.hp[c] > 0 && self.shape.cells[c] == Some(Part::Beam))
    }
    /// Its repeater guns as they stand: the muzzle cell and the length of each column.
    pub fn repeat_guns(&self) -> Vec<(usize, i32)> {
        beam_guns(self.shape.w, self.shape.h, |c| self.hp[c] > 0 && self.shape.cells[c] == Some(Part::Repeater))
    }
    /// Its projector lumps as they stand: the living projector pixels of each, joined edge to edge.
    pub fn proj_lumps(&self) -> Vec<Vec<usize>> {
        let on = |c: usize| self.hp[c] > 0 && self.shape.cells[c] == Some(Part::Projector);
        let mut seen = vec![false; self.hp.len()];
        let mut lumps = Vec::new();
        for c in 0..self.hp.len() {
            if on(c) && !seen[c] {
                let got = reach(self.shape.w, self.shape.h, c, on);
                let cells: Vec<usize> = (0..got.len()).filter(|&k| got[k]).collect();
                for &k in &cells {
                    seen[k] = true;
                }
                lumps.push(cells);
            }
        }
        lumps
    }
    /// The shield pixels whose circles are up: standing, with a shot still to stop.
    pub fn live_shields(&self) -> Vec<usize> {
        (0..self.hp.len()).filter(|&c| self.hp[c] > 0 && self.charge[c] > 0 && self.shape.cells[c] == Some(Part::Shield)).collect()
    }
    /// Its stats from the pixels it has left.
    fn stats_now(&self) -> Stats {
        let cannons = self.turrets.iter().filter(|t| self.hp[t.cell] > 0).count() as i32;
        stats_of(&self.alive_cells(), cannons, self.beam_guns().iter().map(|g| g.1).max().unwrap_or(0), self.repeat_guns().iter().map(|g| g.1).max().unwrap_or(0))
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
        let mut charge = vec![0; (nw * nh) as usize];
        for &c in cells {
            new_cells[map(c)] = self.shape.cells[c];
            cool[map(c)] = self.cool[c];
            charge[map(c)] = self.charge[c];
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
        let longest = |p: Part| beam_guns(nw, nh, |c| shape.cells[c] == Some(p)).iter().map(|g| g.1).max().unwrap_or(0);
        let st = stats_of(&shape.cells, working, longest(Part::Beam), longest(Part::Repeater));
        Ship {
            shape,
            st,
            x,
            y,
            hp,
            cool,
            charge,
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
            sow: false,
            skin: (0, 0),
            rooted: false,
            prey: None,
            spin: 0,
            ..self.clone()
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash)]
pub enum Kind {
    Flak,
    Lance,
    Shell,
    /// It steers at its target every tick (`missiles_steer`), and flak can shoot it down.
    Missile,
    /// A repeater's shot: small, like flak's, fired straight ahead.
    Repeat,
}

#[derive(Clone, Copy, Debug, Hash)]
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
    /// A missile only: thousandths of a pixel of flight carried over from tick to tick, so that
    /// it flies as fast along a diagonal as along a row. Its `speed` is what one step along its
    /// line costs in those, and its `left` the steps it takes this tick.
    acc: i32,
    /// A missile only: flak shots on their way to it. The tick the last of them gets there, and
    /// how many there are, so a whole fleet's flak doesn't turn on one missile.
    cover: (u32, u8),
}

#[derive(Clone, Copy, Debug, Hash)]
pub struct Debris {
    pub x: i32,
    pub y: i32,
    vx: i32,
    vy: i32,
    drift: i32,
    taken: bool,
}

/// A beam's line as it cut, shown for a few ticks: from its muzzle to where it stopped.
#[derive(Clone, Copy, Debug, Hash)]
pub struct Cut {
    pub team: usize,
    pub from: (i32, i32),
    pub to: (i32, i32),
    pub t: u8,
}

/// A beam gun that's warming: where its line runs, and the ticks until it cuts.
#[derive(Clone, Copy, Debug)]
pub struct Warm {
    pub team: usize,
    pub from: (i32, i32),
    pub dir: (i32, i32),
    pub reach: i32,
    pub left: i32,
}

/// A short-lived flash: a shot bouncing off plate or stopped at a shield, or a shell's blast.
#[derive(Clone, Copy, Debug, Hash)]
pub struct Spark {
    pub x: i32,
    pub y: i32,
    pub t: u8,
    pub blast: bool,
}

#[derive(Clone, Copy, Debug, Default, Hash)]
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
    /// Shots stopped at an enemy shield's circle.
    pub shielded: i32,
    /// Enemy missiles this side shot down or cut down.
    pub downed: i32,
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
    /// What SAB goes for: a ship with a scoop or a factory pixel still on it, or anything planted.
    /// Never a king.
    worker: bool,
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
    /// Beams that have just cut.
    pub cuts: Vec<Cut>,
    /// For each ship, the ships whose projectors cover it this tick, nearest first (itself
    /// among them, if it has one). Worked out afresh each tick from where everything is, so it
    /// isn't part of the state the two machines compare.
    pub skins: Vec<Vec<usize>>,
    /// The beams warming at the start of this tick, for ships to get out of the way of.
    warm: Vec<Warm>,
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

/// FNV-1a: a hash that comes out the same on every machine and every run, which the standard
/// library's doesn't promise.
pub struct Fnv(pub u64);

impl Default for Fnv {
    fn default() -> Fnv {
        Fnv(0xcbf2_9ce4_8422_2325)
    }
}

impl std::hash::Hasher for Fnv {
    fn finish(&self) -> u64 {
        self.0
    }
    fn write(&mut self, bytes: &[u8]) {
        for &b in bytes {
            self.0 = (self.0 ^ b as u64).wrapping_mul(0x0000_0100_0000_01b3);
        }
    }
}

/// Everything about a fight that decides what happens next, and nothing else (the stars are
/// scenery). Every field is named here, so a field added to the arena has to be put in the hash
/// or left out on purpose before this compiles; the types inside it derive theirs, so their new
/// fields are in it without anyone remembering. Two machines playing one match in step compare
/// this number: see `Arena::checksum`.
impl std::hash::Hash for Arena {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        // `warm` is worked out afresh from the ships at the start of every tick.
        let Arena { tick, ships, shots, debris, sparks, cuts, skins: _, warm: _, rock, rock_px, stars: _, rng, score, over, stalemate, started, last_loss, last_push, w, h, eco } = self;
        tick.hash(state);
        ships.hash(state);
        shots.hash(state);
        debris.hash(state);
        sparks.hash(state);
        cuts.hash(state);
        // A million rock pixels, 64 to a word.
        rock.len().hash(state);
        for chunk in rock.chunks(64) {
            state.write_u64(chunk.iter().fold(0, |word, &px| word << 1 | px as u64));
        }
        rock_px.hash(state);
        rng.hash(state);
        score.hash(state);
        over.hash(state);
        stalemate.hash(state);
        started.hash(state);
        last_loss.hash(state);
        last_push.hash(state);
        w.hash(state);
        h.hash(state);
        eco.hash(state);
    }
}

impl Arena {
    /// One number for the whole state of the fight (see the `Hash` above). Two machines that
    /// have played the same commands on the same ticks get the same number; if they ever don't,
    /// their matches have parted and nothing after that is the same game.
    pub fn checksum(&self) -> u64 {
        use std::hash::{Hash, Hasher};
        let mut h = Fnv::default();
        self.hash(&mut h);
        h.finish()
    }

    /// An empty map.
    fn blank(w: i32, h: i32, seed: u64) -> Arena {
        Arena {
            tick: 0,
            ships: Vec::new(),
            shots: Vec::new(),
            debris: Vec::new(),
            sparks: Vec::new(),
            cuts: Vec::new(),
            skins: Vec::new(),
            warm: Vec::new(),
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
        // Stagger the guns so a broadside ripples instead of firing as one. A shield starts full.
        let cool = (0..n).map(|i| if shape.cells[i] == Some(Part::Shield) { 0 } else { (i as i32 * 7) % FLAK_PERIOD }).collect();
        let charge = shape.cells.iter().map(|c| if *c == Some(Part::Shield) { SHIELD_CHARGES } else { 0 }).collect();
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
            charge,
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
            want: (0, 0),
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
            sow: false,
            skin: (0, 0),
            rooted: false,
            style: d.style,
            prey: None,
            spin: 0,
            // Only a match has posts to gather at; in the arena everything is there at once.
            waiting: self.eco.is_some() && d.group > 1,
        };
        let h = s.half();
        s.x = x.clamp(h, self.w - 1 - h); // spawn
        s.y = y.clamp(h, self.h - 1 - h); // spawn
        s.home = (s.x, s.y);
        s.pose0 = (s.x, s.y, facing);
        if empty {
            s.st = s.stats_now();
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
        self.warm = self.warming();
        // Ships still arriving by hyperspace aren't there yet.
        let snap: Vec<Snap> = self
            .ships
            .iter()
            .enumerate()
            .filter(|(_, s)| s.alive && s.warp == 0)
            .map(|(idx, s)| Snap { idx, team: s.team, x: s.x, y: s.y, vx: s.vx, vy: s.vy, half: s.half(), full: s.px_full(), working: matches!(s.doing, Doing::Mine | Doing::Haul), worker: s.role == Role::Ship && (s.st.scoops > 0 || s.st.factory > 0 || s.rooted) })
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
        self.skins_refresh();
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
                s.st = s.stats_now();
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
        for c in &mut self.cuts {
            c.t = c.t.saturating_sub(1);
        }
        self.cuts.retain(|c| c.t > 0);
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
        // The enemy it goes for: the nearest of those it may. But it keeps the one it's after
        // until that one is gone, or no longer one it may go for, or another is `KEEP` px nearer.
        // Picked afresh every tick, a ship between two enemies about as far off as each other
        // turned from one to the other and back.
        let prey = self.ships[i].prey;
        let pick = |maxd: i32, from: (i32, i32), workers: bool| -> Option<Snap> {
            let pool = || snap.iter().filter(move |e| e.team != team && cheb(e.x, e.y, from.0, from.1) <= maxd && (!workers || e.worker));
            let near = pool().min_by_key(|e| dist2(e.x, e.y, x, y)).copied()?;
            let kept = prey.and_then(|j| pool().find(|e| e.idx == j).copied());
            Some(match kept {
                Some(k) if isqrt(dist2(near.x, near.y, x, y)) + KEEP > isqrt(dist2(k.x, k.y, x, y)) => k,
                _ => near,
            })
        };
        let mut chosen = None;
        // A ship still waiting for the rest of its group guards where it was built.
        let waiting = self.ships[i].waiting && matches!(order, Order::Hunt | Order::Sab);
        let mut plan = match if waiting { Order::Guard } else { order } {
            Order::Hunt | Order::Sab => {
                if !armed {
                    self.with_group(i, snap)
                } else {
                    // On SAB it's miners and factories first. With none left, it hunts.
                    let found = if order == Order::Sab { pick(i32::MAX / 4, (x, y), true) } else { None };
                    match found.or_else(|| pick(i32::MAX / 4, (x, y), false)) {
                        Some(e) => {
                            chosen = Some(e.idx);
                            self.engage(i, &e, snap)
                        }
                        None => self.go(i, home, Doing::Idle),
                    }
                }
            }
            Order::Guard => match pick(GUARD_RADIUS, home, false) {
                Some(e) if armed && cheb(x, y, home.0, home.1) <= GUARD_RADIUS + 50 => {
                    chosen = Some(e.idx);
                    self.engage(i, &e, snap)
                }
                _ if !armed && hurt => self.flee(i, station),
                _ => self.go(i, home, if waiting { Doing::Muster } else { Doing::Guard }),
            },
            // With its king: it fights what comes near the king while it's near the king itself,
            // and otherwise keeps its place in the rings round it. With no king to escort (the
            // arena), it hunts.
            Order::Escort => match self.escort_spot(i) {
                Some((king, spot)) => match pick(ESCORT_RADIUS, king, false) {
                    Some(e) if armed && cheb(x, y, king.0, king.1) <= ESCORT_RADIUS + 50 => {
                        chosen = Some(e.idx);
                        self.engage(i, &e, snap)
                    }
                    _ => self.go(i, spot, Doing::WithKing),
                },
                None if !armed => self.with_group(i, snap),
                None => match pick(i32::MAX / 4, (x, y), false) {
                    Some(e) => {
                        chosen = Some(e.idx);
                        self.engage(i, &e, snap)
                    }
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
                if st.aims() {
                    if let Some(e) = nearest(st.fwd_reach() + 10, (x, y)) {
                        p.face = axis_face(e.x - x, e.y - y);
                        p.target = Some(e.idx);
                    }
                } else {
                    p.face = None;
                }
                p
            }
        };
        self.ships[i].prey = chosen;
        // A built ship sent home with nothing to do waits at its post instead: its own place in
        // the rings round what built it.
        let posted = st.thrust > 0 && self.ships[i].ring0 > 0 && plan.goal == Some(home) && matches!(plan.doing, Doing::Guard | Doing::Idle | Doing::Hold | Doing::Muster);
        if posted {
            let spot = self.clear_post(i, self.post(i), snap);
            let mut p = self.go(i, spot, plan.doing);
            if plan.doing == Doing::Hold {
                // Holding, it turns only to bring a lance or a beam to bear.
                if plan.target.is_some() || !st.aims() {
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
        // Whatever it was doing, a ship that can get off the line of a warming beam does, and
        // stays off it until the beam has cut.
        if let Some(dv) = self.dodge(i, plan.dv) {
            plan.dv = dv;
            plan.doing = Doing::Dodge;
        }
        plan
    }

    /// The beam guns that are warming, as things stand: where each one's line runs.
    pub fn warming(&self) -> Vec<Warm> {
        let mut out = Vec::new();
        for s in self.ships.iter().filter(|s| s.alive && s.warp == 0 && s.st.beam > 0) {
            for (c, n) in s.beam_guns() {
                if s.cool[c] < 0 {
                    out.push(Warm { team: s.team, from: s.cell_xy(c), dir: facing_vec(s.facing), reach: beam_reach(n), left: -s.cool[c] });
                }
            }
        }
        out
    }

    /// How ship `i` flies to keep off the line of an enemy beam that's warming, when what it
    /// wanted was to fly at `want`. On the line, it gets off: flat out, square to the line, to
    /// the side it's already on (or the other, at the map's edge). That's only if it can be clear
    /// by the time the beam cuts: a ship too wide or too slow for that carries on with what it
    /// was doing. Just clear of the line, it doesn't fly back onto it until the beam has cut:
    /// whatever of `want` would take it towards the line is left out. None: no beam concerns it.
    /// A beam's line moves with its ship, so against a gun that's sliding or turning to follow,
    /// getting clear is a race.
    fn dodge(&self, i: usize, want: (i32, i32)) -> Option<(i32, i32)> {
        let s = &self.ships[i];
        if s.st.thrust == 0 {
            return None;
        }
        let clear = s.half() + 2;
        let (mut kept, mut held) = (want, false);
        let mut out: Option<(i32, (i32, i32))> = None;
        for w in self.warm.iter().filter(|w| w.team != s.team) {
            let (rx, ry) = (s.x - w.from.0, s.y - w.from.1);
            let along = rx * w.dir.0 + ry * w.dir.1;
            // Across the line, on the side of the vector (-dy, dx).
            let (px, py) = (-w.dir.1, w.dir.0);
            let across = rx * px + ry * py;
            if along < -clear || along > w.reach + clear {
                continue;
            }
            if across.abs() < clear {
                if (clear - across.abs()) * 4 > s.st.vmax * w.left {
                    continue;
                }
                let mut side = if across > 0 || (across == 0 && i % 2 == 0) { 1 } else { -1 };
                let to = (s.x + px * side * clear, s.y + py * side * clear);
                if to.0 < clear || to.1 < clear || to.0 > self.w - 1 - clear || to.1 > self.h - 1 - clear {
                    side = -side;
                }
                if out.map_or(true, |(a, _)| along < a) {
                    out = Some((along, (px * side * s.st.vmax, py * side * s.st.vmax)));
                }
            } else if across.abs() < clear + DODGE_ROOM {
                let towards = kept.0 * px + kept.1 * py;
                if towards * across.signum() < 0 {
                    kept = (kept.0 - px * towards, kept.1 - py * towards);
                    held = true;
                }
            }
        }
        match out {
            Some((_, dv)) => Some(dv),
            None if held => Some(kept),
            None => None,
        }
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
        // What's left of the drawing decides whether it can fly at all, and how it flies when
        // nothing was chosen for it.
        let s = &self.ships[i];
        let style = match s.st.flight {
            Flight::Still => None,
            f => s.style.or(Style::of(f)),
        };
        match style {
            Some(Style::Runs) => self.attack_run(i, e),
            Some(Style::Kite) => self.kite(i, e, snap),
            Some(Style::Orbit) => self.orbit(i, e),
            None => Plan { dv: (0, 0), face: None, doing: Doing::Hold, target: Some(e.idx), goal: None },
        }
    }

    /// How far from target `e` ship `i` holds, centre to centre: most of the range of its guns
    /// (the gun it lines up if it has one, a lance or a beam; else its cannon; else its flak),
    /// from the target's edge.
    fn gun_reach(&self, i: usize, e: &Snap) -> i32 {
        let st = &self.ships[i].st;
        let gun = if st.aims() {
            st.fwd_reach() * 3 / 4
        } else if st.cannon > 0 {
            CANNON_RANGE * 2 / 3
        } else if st.missile > 0 {
            MISSILE_RANGE * 2 / 3
        } else {
            FLAK_RANGE * 2 / 3
        };
        gun + e.half
    }

    /// Hold at weapon range from the target, and back away from anything else that comes nearer
    /// than that: while some other enemy is inside its range, the nearest of all is the one it
    /// keeps its distance from.
    fn kite(&mut self, i: usize, e: &Snap, snap: &[Snap]) -> Plan {
        let (x, y, team) = (self.ships[i].x, self.ships[i].y, self.ships[i].team);
        let nearest = snap.iter().filter(|o| o.team != team).min_by_key(|o| dist2(o.x, o.y, x, y)).copied();
        match nearest {
            Some(t) if t.idx != e.idx && isqrt(dist2(t.x, t.y, x, y)) < self.gun_reach(i, &t) => self.stand_off(i, &t, snap),
            _ => self.stand_off(i, e, snap),
        }
    }

    /// Circle the target at weapon range, flat out, facing it. It makes for the place on that
    /// circle a little way round from where it is, so far off it flies straight for the circle and
    /// on it it goes round. It keeps going the way it was already going, and turns back the other
    /// way where the circle leaves the map or something solid stops it. How tight a circle it can
    /// hold is down to its thrust for its weight.
    fn orbit(&mut self, i: usize, e: &Snap) -> Plan {
        let (x, y, vx, vy, vmax, h, bumped) = {
            let s = &self.ships[i];
            (s.x, s.y, s.vx, s.vy, s.st.vmax, s.half(), s.bumped)
        };
        let want = self.gun_reach(i, e);
        // From the target out to the ship. Right on top of it, any way out will do.
        let (rx, ry) = if (x, y) == (e.x, e.y) { (1, 0) } else { (x - e.x, y - e.y) };
        if self.ships[i].spin == 0 {
            let cross = rx * vy - ry * vx;
            self.ships[i].spin = if cross > 0 || (cross == 0 && i % 2 == 0) { 1 } else { -1 };
        }
        let (w, hh) = (self.w, self.h);
        // The place on the circle `ORBIT_LEAD` px round from the ship, and whether the map's edge
        // cut it short. Worked in quarter pixels, like speeds, so short steps keep their angle.
        let ahead = |spin: i32| {
            let out = toward(0, 0, rx, ry, want * 4);
            let round = toward(0, 0, -ry * spin, rx * spin, ORBIT_LEAD * 4);
            let p = toward(0, 0, out.0 + round.0, out.1 + round.1, want * 4);
            let p = (e.x + p.0 / 4, e.y + p.1 / 4);
            let q = (p.0.clamp(h + 2, w - 3 - h), p.1.clamp(h + 2, hh - 3 - h));
            (q, q != p)
        };
        let (mut to, cut) = ahead(self.ships[i].spin as i32);
        if bumped || (cut && cheb(x, y, to.0, to.1) <= 6) {
            self.ships[i].spin = -self.ships[i].spin;
            to = ahead(self.ships[i].spin as i32).0;
        }
        let dv = toward(x, y, to.0, to.1, vmax);
        Plan { dv, face: axis_face(e.x - x, e.y - y), doing: Doing::Orbit, target: Some(e.idx), goal: Some(to) }
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
        let lance_run = if !self.ships[i].outbound && self.ships[i].st.aims() { Some(self.line_up(i, e, (tx, ty))) } else { None };
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
        let pref = self.gun_reach(i, e);
        let (aw, ah) = (self.w, self.h);
        let inside = |(px, py): (i32, i32)| px >= h + 2 && py >= h + 2 && px <= aw - 3 - h && py <= ah - 3 - h;
        // Spots in line with the target, at a few distances; a spot a friend already holds costs
        // extra, so a group spreads into a firing line instead of stacking (own shots pass through
        // own ships, so a line behind a line still fires).
        let crowd = |(px, py): (i32, i32)| snap.iter().filter(|f| f.team == team && f.idx != i && cheb(f.x, f.y, px, py) <= f.half + h + 2).count() as i32;
        let lined_up = if st.aims() {
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
    /// north or south. It leads a moving target by the time a lance shot takes to reach it: the
    /// `dist` to cross, against how fast the target is opening or closing that as it flies, and
    /// never longer than the shot lasts. A ship with a beam lines up its longest beam instead, and
    /// leads by the time it warms.
    fn lance_line(&self, i: usize, e: &Snap, aim: (i32, i32), f: u8, dist: i32) -> i32 {
        let s = &self.ships[i];
        let beam = s.beam_guns().into_iter().max_by_key(|g| g.1).map(|g| g.0);
        let lance = (0..s.shape.cells.len()).find(|&c| s.shape.cells[c] == Some(Part::Lance) && s.hp[c] > 0);
        let rep = s.repeat_guns().into_iter().max_by_key(|g| g.1);
        let (ox, oy) = beam.or(lance).or(rep.map(|g| g.0)).map_or((0, 0), |c| s.shape.cell_xy(0, 0, f, c));
        // A target that's lining up on this ship is flying to this ship's row or column itself.
        // To lead it there is to run from the very line it's coming to, and two ships doing
        // that to each other chase the length of the map and back, with whichever is faster
        // getting its shots in. So a ship doesn't lead a target that's lining a gun up on it:
        // it holds that target's line as it stands, and the two settle on one row or column.
        let ts = &self.ships[e.idx];
        let met = ts.st.aims() && ts.target == Some(i);
        let t = if met {
            0
        } else if beam.is_some() {
            BEAM_WARM
        } else if let (None, Some((_, n))) = (lance, rep) {
            // A repeater's shot is quicker than a lance's and spent sooner.
            let (fx, fy) = facing_vec(f);
            let (v, life) = (e.vx * fx + e.vy * fy, repeat_reach(n) / REPEAT_SPEED);
            if 4 * REPEAT_SPEED - v <= 0 {
                life
            } else {
                (dist.abs() * 4 / (4 * REPEAT_SPEED - v)).min(life)
            }
        } else {
            let (fx, fy) = facing_vec(f);
            let (v, life) = (e.vx * fx + e.vy * fy, LANCE_LIFE / LANCE_SPEED);
            if 4 * LANCE_SPEED - v <= 0 {
                life
            } else {
                (dist.abs() * 4 / (4 * LANCE_SPEED - v)).min(life)
            }
        };
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
            // What it's making for: the speed its plan asked for, cut to what it can do.
            let w = isqrt(p.dv.0 * p.dv.0 + p.dv.1 * p.dv.1);
            s.want = if w > st.vmax && w > 0 { (p.dv.0 * st.vmax / w, p.dv.1 * st.vmax / w) } else { p.dv };
        } else {
            s.vx = 0;
            s.vy = 0;
            s.want = (0, 0);
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
        // The edge of the map stops what flies into it, with room to turn. It never moves what
        // can't fly: a piece shot off a ship beside the edge stays where its pixels are, though
        // its box, turned, would hang over. (It was shoved in from the edge the tick after it
        // broke off.)
        let (cx, cy) = if st.thrust > 0 { (nx.clamp(h, self.w - 1 - h), ny.clamp(h, self.h - 1 - h)) } else { (nx, ny) };
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
        if self.ships[i].st.shields > 0 {
            // A circle that has stopped a shot is full again once it has had its rest.
            let s = &mut self.ships[i];
            for c in 0..s.charge.len() {
                if s.hp[c] > 0 && s.shape.cells[c] == Some(Part::Shield) && s.charge[c] < SHIELD_CHARGES {
                    s.cool[c] -= 1;
                    if s.cool[c] <= 0 {
                        (s.charge[c], s.cool[c]) = (SHIELD_CHARGES, 0);
                    }
                }
            }
        }
        if self.ships[i].st.projector > 0 || self.ships[i].skin != (0, 0) {
            // A projector's pool holds a charge for each projector pixel standing, and is full
            // again once it has had its rest since the last shot it stopped.
            let s = &mut self.ships[i];
            let max = s.st.projector;
            s.skin.0 = s.skin.0.min(max);
            if s.skin.0 < max {
                s.skin.1 -= 1;
                if s.skin.1 <= 0 {
                    s.skin = (max, 0);
                }
            }
        }
        if self.ships[i].st.beam > 0 {
            self.beams(i, snap);
        }
        if self.ships[i].st.repeater > 0 {
            self.repeaters(i, snap);
        }
        for c in 0..self.ships[i].shape.cells.len() {
            let part = self.ships[i].shape.cells[c];
            if self.ships[i].hp[c] == 0 || !matches!(part, Some(Part::Flak) | Some(Part::Lance) | Some(Part::Missile)) {
                continue;
            }
            if self.ships[i].cool[c] > 0 {
                self.ships[i].cool[c] -= 1;
                continue;
            }
            let (gx, gy) = self.ships[i].cell_xy(c);
            if part == Some(Part::Flak) {
                // A missile in reach comes before any ship, unless two shots are on their way to
                // it already.
                if let Some(aim) = self.missile_in_reach(team, (gx, gy)) {
                    let line = Line::new(gx, gy, aim.0, aim.1);
                    self.shots.push(Shot { team, kind: Kind::Flak, line, left: FLAK_RANGE + 12, speed: FLAK_SPEED, dmg: FLAK_DMG, pen: 1, target: None, prev: (gx, gy), connected: false, born: self.tick, acc: 0, cover: (0, 0) });
                    self.ships[i].cool[c] = self.ships[i].st.reload(FLAK_PERIOD) + self.rng.below(4) as i32;
                    self.score[team].fired += 1;
                    continue;
                }
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
                    self.shots.push(Shot { team, kind: Kind::Flak, line, left: FLAK_RANGE + 12, speed: FLAK_SPEED, dmg: FLAK_DMG, pen: 1, target: Some(e.idx), prev: (gx, gy), connected: false, born: self.tick, acc: 0, cover: (0, 0) });
                    pending[e.idx] += 1;
                    // A ship short of power reloads slower.
                    self.ships[i].cool[c] = self.ships[i].st.reload(FLAK_PERIOD) + self.rng.below(4) as i32;
                    self.score[team].fired += 1;
                }
            } else if part == Some(Part::Missile) {
                // A launcher needs no lining up: the nearest enemy in reach, wherever it is.
                let tgt = snap.iter().filter(|e| e.team != team && cheb(e.x, e.y, gx, gy) <= MISSILE_RANGE + e.half).min_by_key(|e| dist2(e.x, e.y, gx, gy)).copied();
                if let Some(e) = tgt.filter(|e| (e.x, e.y) != (gx, gy)) {
                    let line = Line::new(gx, gy, e.x, e.y);
                    self.shots.push(Shot { team, kind: Kind::Missile, line, left: 0, speed: 1000, dmg: CANNON_DMG, pen: 1, target: Some(e.idx), prev: (gx, gy), connected: false, born: self.tick, acc: 0, cover: (0, 0) });
                    self.ships[i].cool[c] = self.ships[i].st.reload(MISSILE_PERIOD);
                    self.score[team].fired += 1;
                }
            } else {
                // A lance only fires straight ahead, and only when its slow shot would meet a
                // pixel still standing, within its range, on an enemy, whatever that enemy does
                // of the three things `shot_meets` asks about.
                let dir = facing_vec(self.ships[i].facing);
                let (dx, dy) = dir;
                let in_line = snap.iter().any(|e| e.team != team && self.shot_meets(e.idx, (gx, gy), dir, LANCE_SPEED, LANCE_LIFE - 1));
                if in_line {
                    self.ships[i].aim_t = 0;
                    let line = Line::new(gx, gy, gx + dx * 400, gy + dy * 400);
                    self.shots.push(Shot { team, kind: Kind::Lance, line, left: LANCE_LIFE, speed: LANCE_SPEED, dmg: LANCE_DMG, pen: LANCE_PEN, target: None, prev: (gx, gy), connected: false, born: self.tick, acc: 0, cover: (0, 0) });
                    self.ships[i].cool[c] = self.ships[i].st.reload(LANCE_PERIOD);
                    self.score[team].fired += 1;
                }
            }
        }
        for k in 0..self.ships[i].turrets.len() {
            self.cannon(i, k, snap, pending);
        }
    }

    /// Whether a gun that fires straight ahead should fire now at ship `j`: a shot from `g` along
    /// `dir`, at `speed` px a tick and good for `range` px. Only if the shot would land three
    /// ways: with the ship flying on as it is, with it flying as it's trying to, and with it
    /// stopped dead where it is. A lance shot takes 1.7 s to cross its range, and in that time a
    /// ship may hold its course, finish the turn it has begun, or pull up. The shot needn't
    /// hit the same pixel each time, only some pixel: a ship is wider than a shot, and one
    /// flying along the line stays on it at any speed. A shot that needs the ship to do exactly
    /// one of the three is a guess, and it's held, so the gun is loaded when a shot that
    /// doesn't comes. A crossing ship is fired at only if it moves less than its own width
    /// while the shot is in flight. In practice: it fires
    /// at what's flying along its line, towards it or away, at what's sitting still, and at
    /// what's crossing so slowly or so near that it can't get off the line in time.
    fn shot_meets(&self, j: usize, g: (i32, i32), dir: (i32, i32), speed: i32, range: i32) -> bool {
        let ts = &self.ships[j];
        let (now, want) = ((ts.vx, ts.vy), ts.want);
        self.meets_at(j, g, dir, speed, range, now) && (want == now || self.meets_at(j, g, dir, speed, range, want)) && (now == (0, 0) || self.meets_at(j, g, dir, speed, range, (0, 0)))
    }

    /// Whether a shot fired now from `(gx, gy)` along `(dx, dy)`, at `speed` px a tick, would meet
    /// a living pixel of ship `j` within `range` px, if `j` flew on at `(vx, vy)` quarter-pixels a
    /// tick. The shot is `k` px down its line `k / speed` ticks from now, and the pixel that will
    /// be at that place then is the one that's now where the place will have come from. So every
    /// place the ship could be met is asked about with its own moment: a ship coming straight
    /// down the line is met sooner and nearer than where it is now, one flying off later and
    /// further, and one flying off as fast as the shot never.
    fn meets_at(&self, j: usize, (gx, gy): (i32, i32), (dx, dy): (i32, i32), speed: i32, range: i32, (vx, vy): (i32, i32)) -> bool {
        let ts = &self.ships[j];
        // Where down the line the ship's middle will be when the shot gets to it: `along` px
        // off now, and that changing by `v` quarter-pixels a tick while the shot makes
        // `4 * speed`.
        let along = (ts.x - gx) * dx + (ts.y - gy) * dy;
        let v = vx * dx + vy * dy;
        if 4 * speed - v <= 0 {
            return false;
        }
        let (mid, half) = (along * 4 * speed / (4 * speed - v), ts.half());
        // (Nothing would be found below either: this only saves the looking.)
        if mid + half < 1 || mid - half > range {
            return false;
        }
        // The shot is at place `k` from `(k - 1) / speed` to `k / speed` ticks from now, and the
        // ship has moved between so much and so much by then: every whole pixel of that.
        let moved = |v: i32, k: i32| {
            let (a, b) = (v * (k - 1) / (4 * speed), v * k / (4 * speed));
            a.min(b)..=a.max(b)
        };
        ((mid - half - 2).max(1)..=(mid + half + 2).min(range)).any(|k| moved(vx, k).any(|ox| moved(vy, k).any(|oy| ts.cell_at(gx + dx * k - ox, gy + dy * k - oy).map_or(false, |c| ts.hp[c] > 0))))
    }

    /// Repeater guns. Each column of repeater pixels is one gun at its front pixel, firing
    /// straight ahead whenever an enemy pixel will be on its line, within its reach, when the
    /// shot gets there if the enemy flies on as it's flying. It doesn't wait for a sure thing
    /// as a lance does: its shots are small and many.
    fn repeaters(&mut self, i: usize, snap: &[Snap]) {
        let team = self.ships[i].team;
        let dir = facing_vec(self.ships[i].facing);
        for (c, n) in self.ships[i].repeat_guns() {
            // A repeater's wait is exact: a gun with a wait of seven fires on every seventh tick.
            if self.ships[i].cool[c] > 0 {
                self.ships[i].cool[c] -= 1;
            }
            if self.ships[i].cool[c] > 0 {
                continue;
            }
            let (gx, gy) = self.ships[i].cell_xy(c);
            let reach = repeat_reach(n);
            let in_line = snap.iter().any(|e| e.team != team && self.meets_at(e.idx, (gx, gy), dir, REPEAT_SPEED, reach, (e.vx, e.vy)));
            if in_line {
                let line = Line::new(gx, gy, gx + dir.0 * 400, gy + dir.1 * 400);
                self.shots.push(Shot { team, kind: Kind::Repeat, line, left: reach + 1, speed: REPEAT_SPEED, dmg: FLAK_DMG, pen: 1, target: None, prev: (gx, gy), connected: false, born: self.tick, acc: 0, cover: (0, 0) });
                let s = &mut self.ships[i];
                s.cool[c] = s.st.reload(repeat_period(n));
                s.aim_t = 0;
                self.score[team].fired += 1;
            }
        }
    }

    /// Every working lump of projector pixels on the map: its ship, its side, its box, and how
    /// far from that box it reaches.
    pub fn proj_fields(&self) -> Vec<(usize, usize, (i32, i32, i32, i32), i32)> {
        let mut lumps = Vec::new();
        for (a, s) in self.ships.iter().enumerate() {
            if !s.alive || s.warp > 0 || s.st.projector == 0 {
                continue;
            }
            for cells in s.proj_lumps() {
                let at: Vec<(i32, i32)> = cells.iter().map(|&c| s.cell_xy(c)).collect();
                let bbox = (at.iter().map(|p| p.0).min().unwrap(), at.iter().map(|p| p.1).min().unwrap(), at.iter().map(|p| p.0).max().unwrap(), at.iter().map(|p| p.1).max().unwrap());
                lumps.push((a, s.team, bbox, proj_reach(cells.len() as i32)));
            }
        }
        lumps
    }

    /// Which ships' projectors cover which ships, this tick. A lump of projector pixels reaches
    /// `proj_reach` of its pixels from its edge, and covers every other ship of its side any
    /// part of whose box is within that of the lump's box. Never its own ship (Henry, 2026-11-11
    /// and again 2026-11-13): a projector is for its friends, and its ship is how you get at it.
    fn skins_refresh(&mut self) {
        let n = self.ships.len();
        let lumps = self.proj_fields();
        let mut table = vec![Vec::new(); n];
        if !lumps.is_empty() {
            for (b, s) in self.ships.iter().enumerate() {
                if !s.alive || s.warp > 0 {
                    continue;
                }
                let (l, t, r, bt) = s.bbox();
                let mut near: Vec<(i32, usize)> = lumps
                    .iter()
                    .filter(|lump| lump.1 == s.team && lump.0 != b)
                    .filter_map(|&(a, _, (ll, lt, lr, lb), reach)| {
                        let (dx, dy) = ((ll - r).max(l - lr).max(0), (lt - bt).max(t - lb).max(0));
                        let d2 = dx * dx + dy * dy;
                        (d2 <= reach * reach).then_some((d2, a))
                    })
                    .collect();
                near.sort();
                for (_, a) in near {
                    if !table[b].contains(&a) {
                        table[b].push(a);
                    }
                }
            }
        }
        self.skins = table;
    }

    /// The ship whose projector would stop a shot at pixel `c` of ship `id`: the nearest one
    /// covering it that has a charge left. Never for a projector pixel: those are always bare.
    pub fn skin_for(&self, id: usize, c: usize) -> Option<usize> {
        if self.ships[id].shape.cells[c] == Some(Part::Projector) {
            return None;
        }
        self.skins.get(id)?.iter().copied().find(|&a| self.ships[a].alive && self.ships[a].skin.0 > 0)
    }

    /// A shot of team `by` is stopped by ship `a`'s projector at `at`: its pool has one shot
    /// fewer to stop, and its rest starts again.
    fn skin_stop(&mut self, a: usize, at: (i32, i32), by: usize) {
        let s = &mut self.ships[a];
        s.skin = (s.skin.0 - 1, s.st.reload(PROJ_REST));
        self.score[by].shielded += 1;
        self.sparks.push(Spark { x: at.0, y: at.1, t: 2, blast: false });
    }

    /// The enemy missile a flak gun at `(gx, gy)` should shoot at: the nearest in the gun's range
    /// that doesn't have two shots on their way to it already. It gives back where to aim, which
    /// is where the missile will be when the shot gets there (it flies straight at its target),
    /// and counts the shot against the missile.
    fn missile_in_reach(&mut self, team: usize, (gx, gy): (i32, i32)) -> Option<(i32, i32)> {
        let tick = self.tick;
        let k = (0..self.shots.len())
            .filter(|&k| {
                let m = &self.shots[k];
                m.kind == Kind::Missile && m.team != team && cheb(m.line.x, m.line.y, gx, gy) <= FLAK_RANGE && (tick > m.cover.0 || m.cover.1 < 2)
            })
            .min_by_key(|&k| dist2(self.shots[k].line.x, self.shots[k].line.y, gx, gy))?;
        let (mx, my) = (self.shots[k].line.x, self.shots[k].line.y);
        let t = cheb(mx, my, gx, gy) / FLAK_SPEED + 1;
        let aim = match self.shots[k].target.filter(|&j| self.ships[j].alive) {
            Some(j) => {
                let (dx, dy) = (self.ships[j].x - mx, self.ships[j].y - my);
                let len = isqrt(dx * dx + dy * dy).max(1);
                let far = (MISSILE_SPEED * t / 1000).min(len);
                (mx + dx * far / len, my + dy * far / len)
            }
            None => (mx, my),
        };
        let m = &mut self.shots[k];
        m.cover = (tick + t as u32 + 1, if tick > m.cover.0 { 1 } else { m.cover.1 + 1 });
        Some(if aim == (gx, gy) { (mx, my + 1) } else { aim })
    }

    /// Missiles steer, before anything flies. Each turns straight at where its target is now and
    /// works out its steps for this tick: `MISSILE_SPEED` thousandths of a pixel, measured along
    /// its line, so it's as fast on a diagonal as along a row. One whose target is gone flies on
    /// the way it was going. One that has flown `MISSILE_LIFE` ticks is spent.
    fn missiles_steer(&mut self, shots: &mut Vec<Shot>) {
        let tick = self.tick;
        shots.retain(|s| s.kind != Kind::Missile || tick.saturating_sub(s.born) < MISSILE_LIFE);
        for sh in shots.iter_mut().filter(|s| s.kind == Kind::Missile) {
            let (x, y) = (sh.line.x, sh.line.y);
            if let Some(t) = sh.target.filter(|&t| self.ships[t].alive && self.ships[t].warp == 0) {
                let (dx, dy) = (self.ships[t].x - x, self.ships[t].y - y);
                let major = dx.abs().max(dy.abs());
                if major > 0 {
                    let k = 600 / major + 1;
                    sh.line = Line::new(x, y, x + dx * k, y + dy * k);
                    // What one step along this line covers: 1 px along a row, 1.41 on a diagonal.
                    let slope = dx.abs().min(dy.abs()) * 1000 / major;
                    sh.speed = isqrt(1_000_000 + slope * slope);
                }
            }
            sh.acc += MISSILE_SPEED;
            sh.left = sh.acc / sh.speed.max(1000);
            sh.acc -= sh.left * sh.speed.max(1000);
        }
    }

    /// Flak against missiles, before either flies on. A flak shot that comes within a pixel of an
    /// enemy missile at any moment of this tick's flight takes it down, and is spent doing it.
    /// The two are walked together at half-pixel steps, so neither can skip past the other.
    fn intercept(&mut self, shots: &mut Vec<Shot>) {
        if !shots.iter().any(|s| s.kind == Kind::Missile) {
            return;
        }
        // Where a shot is through this tick, pixel by pixel.
        let path = |s: &Shot| -> Vec<(i32, i32)> {
            let (mut l, steps) = (s.line, if s.kind == Kind::Missile { s.left } else { s.speed.min(s.left) });
            let mut p = vec![(l.x, l.y)];
            for _ in 0..steps {
                l.step();
                p.push((l.x, l.y));
            }
            p
        };
        let mut dead = vec![false; shots.len()];
        for mi in 0..shots.len() {
            if shots[mi].kind != Kind::Missile {
                continue;
            }
            let pm = path(&shots[mi]);
            for fi in 0..shots.len() {
                if dead[fi] || !matches!(shots[fi].kind, Kind::Flak | Kind::Repeat) || shots[fi].team == shots[mi].team || cheb(shots[fi].line.x, shots[fi].line.y, pm[0].0, pm[0].1) > FLAK_SPEED + 6 {
                    continue;
                }
                let pf = path(&shots[fi]);
                let n = 2 * (pm.len().max(pf.len()) as i32 - 1).max(1);
                let met = (0..=n).any(|j| {
                    let (a, b) = (pm[(j * (pm.len() as i32 - 1) / n) as usize], pf[(j * (pf.len() as i32 - 1) / n) as usize]);
                    cheb(a.0, a.1, b.0, b.1) <= 1
                });
                if met {
                    (dead[mi], dead[fi]) = (true, true);
                    self.score[shots[fi].team].downed += 1;
                    self.sparks.push(Spark { x: pm[0].0, y: pm[0].1, t: 3, blast: true });
                    break;
                }
            }
        }
        let mut k = 0;
        shots.retain(|_| {
            k += 1;
            !dead[k - 1]
        });
    }

    /// Beam guns. A rested gun starts to warm when an enemy pixel will be on its line, within
    /// its reach, by the time it has warmed. Warming, it shows its line (which moves with the
    /// ship); warmed, it cuts along that line whatever is or isn't on it by then, and rests.
    fn beams(&mut self, i: usize, snap: &[Snap]) {
        let team = self.ships[i].team;
        let (dx, dy) = facing_vec(self.ships[i].facing);
        for (c, n) in self.ships[i].beam_guns() {
            let (gx, gy) = self.ships[i].cell_xy(c);
            let cool = self.ships[i].cool[c];
            if cool > 0 {
                self.ships[i].cool[c] -= 1;
            } else if cool < 0 {
                self.ships[i].cool[c] += 1;
                if self.ships[i].cool[c] == 0 {
                    self.cut(i, (gx, gy), (dx, dy), n);
                    let s = &mut self.ships[i];
                    s.cool[c] = s.st.reload(BEAM_PERIOD);
                    s.aim_t = 0;
                    self.score[team].fired += 1;
                }
            } else {
                let reach = beam_reach(n);
                let ships = &self.ships;
                let in_line = snap.iter().any(|e| {
                    if e.team == team {
                        return false;
                    }
                    // Where the target will be when the beam cuts, if it flies on as it's
                    // flying: so far along the line, and each of its pixels moved by so much.
                    let (ox, oy) = (e.vx * BEAM_WARM / 4, e.vy * BEAM_WARM / 4);
                    let then = (e.x + ox - gx) * dx + (e.y + oy - gy) * dy;
                    if then + e.half < 1 || then - e.half > reach {
                        return false;
                    }
                    let ts = &ships[e.idx];
                    ((then - e.half - 1).max(1)..=(then + e.half + 1).min(reach)).any(|k| ts.cell_at(gx + dx * k - ox, gy + dy * k - oy).map_or(false, |c| ts.hp[c] > 0))
                });
                if in_line {
                    self.ships[i].cool[c] = -BEAM_WARM;
                }
            }
        }
    }

    /// A beam gun of `n` pixels fires from its muzzle at `(gx, gy)`: every enemy pixel on its line
    /// goes at once, nearest first, until its cut is used up or its reach ends. Plate uses up two
    /// of the cut. A shield's circle that the line comes into from outside stops it at the edge.
    fn cut(&mut self, i: usize, (gx, gy): (i32, i32), (dx, dy): (i32, i32), n: i32) {
        let team = self.ships[i].team;
        let (reach, mut pen) = (beam_reach(n), beam_cut(n));
        // The enemy's circles, and whether the line is inside each as it goes: it starts inside
        // any circle the muzzle is in, and that circle doesn't stop it.
        let mut rings: Vec<(usize, usize, (i32, i32), bool)> = Vec::new();
        let mut boxes: Vec<(usize, (i32, i32, i32, i32))> = Vec::new();
        for (id, s) in self.ships.iter().enumerate() {
            if !s.alive || s.team == team || s.warp > 0 {
                continue;
            }
            boxes.push((id, s.bbox()));
            for c in s.live_shields() {
                let at = s.cell_xy(c);
                rings.push((id, c, at, dist2(gx, gy, at.0, at.1) <= SHIELD_R * SHIELD_R));
            }
        }
        let (mut end, mut hit) = ((gx, gy), false);
        'walk: for k in 1..=reach {
            let (px, py) = (gx + dx * k, gy + dy * k);
            if px < 0 || py < 0 || px >= self.w || py >= self.h {
                break;
            }
            for r in rings.iter_mut() {
                let inside = dist2(px, py, r.2 .0, r.2 .1) <= SHIELD_R * SHIELD_R;
                if inside && !r.3 {
                    let (id, c) = (r.0, r.1);
                    self.shield_stop(id, c, end, team);
                    break 'walk;
                }
                r.3 = inside;
            }
            end = (px, py);
            for &(id, (l, t, r, b)) in &boxes {
                if px < l || py < t || px > r || py > b {
                    continue;
                }
                let Some(c) = self.ships[id].cell_at(px, py).filter(|&c| self.ships[id].hp[c] > 0) else { continue };
                // A projector's skin stops the cut at the first covered pixel, for one charge.
                if let Some(a) = self.skin_for(id, c) {
                    self.skin_stop(a, end, team);
                    break 'walk;
                }
                hit = true;
                let armor = self.ships[id].shape.cells[c].map_or(0, |p| p.armor());
                if BEAM_DMG <= armor {
                    self.score[team].bounced += 1;
                    self.sparks.push(Spark { x: px - dx, y: py - dy, t: 2, blast: false });
                    break 'walk;
                }
                self.kill_cell(id, c, (px, py), (dx, dy));
                pen -= 1 + armor;
                if pen <= 0 {
                    break 'walk;
                }
            }
        }
        if hit {
            self.score[team].hits += 1;
        }
        self.cuts.push(Cut { team, from: (gx, gy), to: end, t: 3 });
        // An enemy missile on the line, as far as the beam cut, is cut down with everything else.
        let len = (end.0 - gx).abs().max((end.1 - gy).abs());
        let before = self.shots.len();
        self.shots.retain(|s| {
            let (rx, ry) = (s.line.x - gx, s.line.y - gy);
            let (along, across) = (rx * dx + ry * dy, (rx * dy - ry * dx).abs());
            !(s.kind == Kind::Missile && s.team != team && (1..=len).contains(&along) && across <= 1)
        });
        self.score[team].downed += (before - self.shots.len()) as i32;
    }

    /// A shot of team `by` is stopped at the edge of ship `id`'s shield circle `c`, at `at`: the
    /// circle has one shot fewer to stop, and its rest starts again.
    fn shield_stop(&mut self, id: usize, c: usize, at: (i32, i32), by: usize) {
        let s = &mut self.ships[id];
        s.charge[c] = s.charge[c].saturating_sub(1);
        s.cool[c] = s.st.reload(SHIELD_REST);
        self.score[by].shielded += 1;
        self.sparks.push(Spark { x: at.0, y: at.1, t: 2, blast: false });
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
            self.shots.push(Shot { team, kind: Kind::Shell, line, left: CANNON_RANGE + 12, speed: SHELL_SPEED, dmg: CANNON_DMG, pen: 1, target: Some(e.idx), prev: (gx, gy), connected: false, born: self.tick, acc: 0, cover: (0, 0) });
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
        let mut shots = std::mem::take(&mut self.shots);
        self.missiles_steer(&mut shots);
        self.intercept(&mut shots);
        let boxes: Vec<(usize, usize, (i32, i32, i32, i32))> = self.ships.iter().enumerate().filter(|(_, s)| s.alive).map(|(i, s)| (i, s.team, s.bbox())).collect();
        let mut keep = Vec::with_capacity(shots.len());
        // In tests: the path of every shot that flew on, to check it skipped nothing.
        let mut flown: Vec<(Vec<(i32, i32)>, usize, u32)> = Vec::new();
        'shot: for mut sh in shots {
            // The shot's own flight this tick, a pixel at a time.
            let mut path = vec![(sh.line.x, sh.line.y)];
            let mut gone = false;
            // A missile's steps for this tick were worked out when it steered, and it's spent by
            // the clock, not by how far it has flown.
            let missile = sh.kind == Kind::Missile;
            for _ in 0..if missile { sh.left } else { sh.speed } {
                sh.line.step();
                if !missile {
                    sh.left -= 1;
                }
                let (nx, ny) = (sh.line.x, sh.line.y);
                if (!missile && sh.left <= 0) || nx < 0 || ny < 0 || nx >= self.w || ny >= self.h {
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
            // Every enemy shield circle the shot comes into from outside, with when. Like the
            // pixels, a circle is met along the path the shot takes relative to its ship.
            let mut stops: Vec<(i32, usize, usize)> = Vec::new();
            for (id, s) in self.ships.iter().enumerate() {
                if !s.alive || s.team == sh.team || s.warp > 0 {
                    continue;
                }
                // A shot fired this tick was fired after the ships moved.
                let (x0, y0, f0) = if sh.born == self.tick { (s.x, s.y, s.facing) } else { s.pose0 };
                let shields = if s.st.shields > 0 { s.live_shields() } else { Vec::new() };
                let r = s.half() + 1 + if shields.is_empty() { 0 } else { SHIELD_R };
                if pr < s.x.min(x0) - r || pb < s.y.min(y0) - r || pl > s.x.max(x0) + r || pt > s.y.max(y0) + r {
                    continue;
                }
                // The shot's path relative to the ship, in the ship's own grid, with when it's at
                // each point: only worked out for a ship with a circle up.
                let mut walk: Vec<(i32, (i32, i32))> = Vec::new();
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
                    if !shields.is_empty() {
                        walk.push((0, s.shape.to_local(s.x, s.y, s.facing, s0.0, s0.1)));
                    }
                    for j in 1..=m {
                        let ((px, py), (nx, ny)) = (path[j as usize - 1], path[j as usize]);
                        let t = j * 1000 / m;
                        if px != nx && py != ny {
                            meet(t, s.cell_at(nx, py));
                            meet(t, s.cell_at(px, ny));
                        }
                        meet(t, s.cell_at(nx, ny));
                        if !shields.is_empty() {
                            walk.push((t, s.shape.to_local(s.x, s.y, s.facing, nx, ny)));
                        }
                    }
                } else {
                    // The ship moved or turned: walk the relative path in the ship's own grid.
                    let a = s.shape.to_local(x0, y0, f0, s0.0, s0.1);
                    let b = s.shape.to_local(s.x, s.y, s.facing, s1.0, s1.1);
                    let n = (b.0 - a.0).abs().max((b.1 - a.1).abs());
                    let mut l = Line::new(a.0, a.1, b.0, b.1);
                    if !shields.is_empty() {
                        walk.push((0, a));
                    }
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
                        if !shields.is_empty() {
                            walk.push((t, (nx, ny)));
                        }
                    }
                }
                // A circle stops the shot where the shot comes into it: outside it at one point
                // of the walk, inside at the next. A shot that starts the tick inside a circle
                // (fired from in there, say) isn't stopped by that one.
                for &c in &shields {
                    let (cx, cy) = (c as i32 % s.shape.w, c as i32 / s.shape.w);
                    let inside = |p: (i32, i32)| dist2(p.0, p.1, cx, cy) <= SHIELD_R * SHIELD_R;
                    let mut was = walk.first().map_or(true, |&(_, p)| inside(p));
                    for &(t, p) in walk.iter().skip(1) {
                        let now = inside(p);
                        if now && !was {
                            stops.push((t, id, c));
                            break;
                        }
                        was = now;
                    }
                }
            }
            meets.sort_by_key(|&(t, _, _)| t);
            // The first circle it comes into. A pixel met at the same moment is inside the
            // circle, so the circle comes first.
            let stop = stops.iter().min().copied();
            for (t, id, c) in meets {
                if stop.is_some_and(|(ts, _, _)| t >= ts) {
                    break;
                }
                if self.ships[id].hp[c] == 0 {
                    continue;
                }
                // How far along its own flight the shot is when it meets this pixel.
                let k = ((t * m + 999) / 1000).clamp(1, m.max(1)) as usize;
                let before = path[(k - 1).min(path.len() - 1)];
                let at = self.ships[id].cell_xy(c);
                // A projector's skin: the shot stops at the pixel it would have hit, whatever
                // kind of shot it is, and costs the projector a charge.
                if let Some(a) = self.skin_for(id, c) {
                    self.skin_stop(a, before, sh.team);
                    continue 'shot;
                }
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
                if sh.kind == Kind::Shell || sh.kind == Kind::Missile {
                    self.blast(at.0, at.1, sh.team, &boxes);
                    continue 'shot;
                }
                self.kill_cell(id, c, at, sh.line.dir());
                sh.pen -= 1 + armor;
                if sh.pen <= 0 {
                    continue 'shot;
                }
            }
            if let Some((t, id, c)) = stop {
                // Stopped at the circle's edge: where the shot was just before it came in.
                let k = ((t * m + 999) / 1000).clamp(1, m.max(1)) as usize;
                let before = path[(k - 1).min(path.len() - 1)];
                self.shield_stop(id, c, before, sh.team);
                continue 'shot;
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
        // A burst on a bare pixel (a projector's own) spares the covered pixels round it, for
        // one charge in all.
        let mut spared: Option<usize> = None;
        for dy in -1..=1 {
            for dx in -1..=1 {
                let (px, py) = (x + dx, y + dy);
                for &(id, t, (l, tp, r, b)) in boxes {
                    if t == team || px < l || py < tp || px > r || py > b {
                        continue;
                    }
                    if let Some(c) = self.ships[id].cell_at(px, py) {
                        if self.ships[id].hp[c] > 0 {
                            if let Some(a) = self.skin_for(id, c) {
                                spared.get_or_insert(a);
                                continue;
                            }
                        }
                        if self.ships[id].shape.cells[c].map_or(0, |p| p.armor()) < CANNON_DMG {
                            self.kill_cell(id, c, (px, py), (dx, dy));
                        }
                    }
                }
            }
        }
        if let Some(a) = spared {
            self.skin_stop(a, (x, y), team);
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

    /// A fighter, and a slab of plate round a core that its flak can't hurt.
    fn fighter_and_slab(order: Order, style: Option<Style>) -> (Design, Design) {
        let mut f = Design::from_rows("FIGHTER", order, &["g.g", "hch", "e.e"]);
        f.style = style;
        (f, Design::from_rows("SLAB", Order::Hold, &["ppp", "pcp", "ppp"]))
    }

    /// One tick with ship 0 put back at rest in the middle of the arena first, so what it's after
    /// can be asked about at known distances.
    fn step_from_the_middle(a: &mut Arena) -> Option<usize> {
        let s = &mut a.ships[0];
        (s.x, s.y, s.vx, s.vy, s.sx, s.sy) = (240, 180, 0, 0, 0, 0);
        a.step();
        a.ships[0].prey
    }

    #[test]
    fn a_ship_keeps_the_enemy_its_after_until_another_is_clearly_nearer() {
        let (fighter, slab) = fighter_and_slab(Order::Hunt, None);
        let mut a = Arena::new();
        assert!(a.add(0, 0, &fighter, 240, 180));
        assert!(a.add(1, 0, &slab, 140, 180));
        assert!(a.add(1, 1, &slab, 345, 180));
        // The nearer of the two, to start with.
        assert_eq!(step_from_the_middle(&mut a), Some(1));
        // The other brought nearer than it, but not by much: it keeps the one it has.
        a.ships[2].x = 240 + 100 - KEEP + 1;
        assert_eq!(step_from_the_middle(&mut a), Some(1), "it turned to one only {} px nearer", KEEP - 1);
        // A pixel more, and that's clearly nearer.
        a.ships[2].x = 240 + 100 - KEEP;
        assert_eq!(step_from_the_middle(&mut a), Some(2));
        // Now the first would have to be that much nearer again to win it back.
        a.ships[1].x = 240 - (100 - KEEP) + 5;
        assert_eq!(step_from_the_middle(&mut a), Some(2));
        // What it's after gone, it takes the nearest left.
        a.ships[2].alive = false;
        assert_eq!(step_from_the_middle(&mut a), Some(1));
        a.ships[1].alive = false;
        assert_eq!(step_from_the_middle(&mut a), None);
    }

    #[test]
    fn a_light_ship_told_to_kite_holds_its_range_and_backs_off_what_comes_closer() {
        let (kiter, slab) = fighter_and_slab(Order::Hunt, Some(Style::Kite));
        // Left to its drawing, it would dive straight through.
        assert_eq!(fighter_and_slab(Order::Hunt, None).0.flies(), Some(Style::Runs));
        assert_eq!(kiter.flies(), Some(Style::Kite));
        let mut a = Arena::new();
        assert!(a.add(0, 0, &kiter, 100, 180));
        assert!(a.add(1, 0, &slab, 300, 180));
        let apart = |a: &Arena, j: usize| isqrt(dist2(a.ships[0].x, a.ships[0].y, a.ships[j].x, a.ships[j].y));
        let want = FLAK_RANGE * 2 / 3 + a.ships[1].half();
        let mut nearest = i32::MAX;
        for _ in 0..15 * 12 {
            a.step();
            nearest = nearest.min(apart(&a, 1));
        }
        assert!(nearest >= want - 6, "it went in to {nearest} px, and should hold at {want}");
        assert!(apart(&a, 1) <= want + 6, "it never closed to its range: {} px", apart(&a, 1));
        assert_eq!(a.ships[0].doing, Doing::StandOff);
        // Something else turns up closer than its guns reach, off to one side. It gets clear of
        // that, and what it was after is still what it's after.
        let (x, y) = (a.ships[0].x, a.ships[0].y);
        assert!(a.add(1, 1, &slab, x, y - 12));
        assert!(apart(&a, 2) < want - 6);
        for _ in 0..15 * 4 {
            a.step();
        }
        assert!(apart(&a, 2) >= want - 6, "still {} px from what came close", apart(&a, 2));
        assert_eq!(a.ships[0].prey, Some(1));
    }

    /// Which quarter of the circle round ship `j` ship 0 is in: 0 to 3, clockwise from north-east.
    fn quarter(a: &Arena, j: usize) -> i32 {
        let (dx, dy) = (a.ships[0].x - a.ships[j].x, a.ships[0].y - a.ships[j].y);
        match (dx >= 0, dy >= 0) {
            (true, false) => 0,
            (true, true) => 1,
            (false, true) => 2,
            (false, false) => 3,
        }
    }

    #[test]
    fn a_ship_told_to_orbit_circles_its_target_at_the_range_of_its_guns() {
        let (orbiter, slab) = fighter_and_slab(Order::Hunt, Some(Style::Orbit));
        let mut a = Arena::new();
        assert!(a.add(0, 0, &orbiter, 100, 180));
        assert!(a.add(1, 0, &slab, 240, 180));
        let want = FLAK_RANGE * 2 / 3 + a.ships[1].half();
        run(&mut a, 15 * 6);
        // Twenty seconds on the circle: never far off it, and round and round the one way.
        let (mut lo, mut hi, mut quarters) = (i32::MAX, 0, vec![quarter(&a, 1)]);
        for _ in 0..15 * 20 {
            a.step();
            let d = isqrt(dist2(a.ships[0].x, a.ships[0].y, a.ships[1].x, a.ships[1].y));
            (lo, hi) = (lo.min(d), hi.max(d));
            if quarters.last() != Some(&quarter(&a, 1)) {
                quarters.push(quarter(&a, 1));
            }
            assert_eq!(a.ships[0].doing, Doing::Orbit);
        }
        assert!(lo >= want - 8 && hi <= want + 8, "it flew between {lo} and {hi} px out, and should hold {want}");
        let turns: Vec<i32> = quarters.windows(2).map(|q| (q[1] - q[0]).rem_euclid(4)).collect();
        assert!(turns.len() >= 8, "it only got through {quarters:?}");
        assert!(turns[0] != 2 && turns.iter().all(|&t| t == turns[0]), "it didn't keep going round one way: {quarters:?}");
    }

    #[test]
    fn an_orbit_that_runs_into_the_maps_edge_turns_back_the_other_way() {
        // The target sits so near the west edge that most of the circle round it is off the map.
        let (orbiter, slab) = fighter_and_slab(Order::Hunt, Some(Style::Orbit));
        let mut a = Arena::new();
        assert!(a.add(0, 0, &orbiter, 120, 100));
        assert!(a.add(1, 0, &slab, 14, 180));
        run(&mut a, 15 * 6);
        let (mut flips, mut flown, mut spin, mut at) = (0, 0, a.ships[0].spin, (a.ships[0].x, a.ships[0].y));
        let mut seen = [false; 4];
        for _ in 0..15 * 30 {
            a.step();
            let s = &a.ships[0];
            flips += (s.spin != spin) as i32;
            flown += cheb(s.x, s.y, at.0, at.1);
            (spin, at) = (s.spin, (s.x, s.y));
            seen[quarter(&a, 1) as usize] = true;
            assert!(s.x >= s.half() && s.y >= s.half());
        }
        assert!(flips >= 2, "it turned back {flips} times in half a minute");
        assert!(flown >= 15 * 30, "it mostly sat still: {flown} px flown in half a minute");
        assert!(seen[0] && seen[1], "it didn't sweep the part of the circle that's on the map");
    }

    #[test]
    fn a_ship_on_sab_goes_for_miners_and_factories_past_what_guards_them() {
        let (raider, slab) = fighter_and_slab(Order::Sab, None);
        let miner = Design::from_rows("MINER", Order::Hold, &["sss", "hch", "e.e"]);
        let works = Design::from_rows("WORKS", Order::Hold, &["f", "c"]);
        let mut a = Arena::new();
        assert!(a.add(0, 0, &raider, 240, 180));
        assert!(a.add(1, 0, &slab, 200, 180));
        assert!(a.add(1, 1, &miner, 400, 180));
        assert!(a.add(1, 2, &works, 60, 180));
        // A guard is 40 px off, a miner 160 and a factory 180: the miner.
        assert_eq!(step_from_the_middle(&mut a), Some(2));
        // On HUNT it would take the guard.
        a.ships[0].order = Order::Hunt;
        a.ships[0].prey = None;
        assert_eq!(step_from_the_middle(&mut a), Some(1));
        a.ships[0].order = Order::Sab;
        // The miner gone, the factory. But never a king, whatever is drawn on it: were that
        // factory a king, there'd be nothing left to sabotage, and it would hunt.
        a.ships[2].alive = false;
        assert_eq!(step_from_the_middle(&mut a), Some(3));
        a.ships[3].role = Role::King;
        assert_eq!(step_from_the_middle(&mut a), Some(1));
        // A factory again, it comes first again: what it was hunting is no reason to stay.
        a.ships[3].role = Role::Ship;
        assert_eq!(step_from_the_middle(&mut a), Some(3));
        // That gone too, it hunts.
        a.ships[3].alive = false;
        assert_eq!(step_from_the_middle(&mut a), Some(1));
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
            let st = stats_of(&s.alive_cells(), 0, 0, 0);
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
        Shot { team: 0, kind: Kind::Flak, line, left: FLAK_RANGE, speed: FLAK_SPEED, dmg: FLAK_DMG, pen: 1, target: None, prev: (x, y), connected: false, born: a.tick.saturating_sub(1), acc: 0, cover: (0, 0) }
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
    fn the_maps_edge_never_moves_what_cant_fly() {
        // A hulk twelve wide and six tall, lying along the top edge of the map: every pixel of it
        // is on the map, but the square it would need to turn in isn't. It can't turn, or fly,
        // so it stays exactly where it is. The same shape with engines is kept a turn's room from
        // the edge, as any ship is.
        let hulk = Design::from_rows("HULK", Order::Hold, &["hhhhhhhhhhhh", "hhhhhhhhhhhh", "hhhhhchhhhhh", "hhhhhhhhhhhh", "hhhhhhhhhhhh", "hhhhhhhhhhhh"]);
        let barge = Design::from_rows("BARGE", Order::Hold, &["hhhhhhhhhhhh", "hhhhhhhhhhhh", "hhhhhchhhhhh", "hhhhhhhhhhhh", "hhhhhhhhhhhh", "eeeeeeeeeeee"]);
        let mut a = Arena::new();
        let h = a.spawn(0, 0, &hulk, 100, 20, 0, false);
        let b = a.spawn(0, 1, &barge, 300, 20, 0, false);
        assert!(a.ships[h].st.thrust == 0 && a.ships[b].st.thrust > 0 && a.ships[h].half() == 6);
        // Nothing is placed that close to the edge, so they're put there as a piece shot off a
        // bigger ship would be left: where its pixels were.
        for i in [h, b] {
            a.ships[i].y = 3;
        }
        for _ in 0..30 {
            a.step();
            assert_eq!((a.ships[h].x, a.ships[h].y), (100, 3), "tick {}: the hulk was moved", a.tick);
            assert!(a.ships[b].y >= 6, "tick {}: a ship that flies keeps room to turn at the edge", a.tick);
        }
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

    // ----- A lance's aim -----

    /// Blue's lance on a hulk with no engines, facing east, and a red hull frame with engines
    /// `far` px down its line and `side` px off it. Gives the arena, where the lance is, and the
    /// frame's index. The frame flies and tries to fly as the test sets it.
    fn lance_and_frame(far: i32, side: i32) -> (Arena, (i32, i32), usize) {
        let gun = Design::from_rows("GUN", Order::Hold, &["hlh", "hch"]);
        let frame = Design::from_rows("FRAME", Order::Hold, &[".h.", "hhh", "hch", "e.e"]);
        let mut a = Arena::new();
        a.add(0, 0, &gun, 150, 120);
        let s = &a.ships[0];
        let l = (0..s.hp.len()).find(|&c| s.shape.cells[c] == Some(Part::Lance)).unwrap();
        let g = s.cell_xy(l);
        a.add(1, 0, &frame, g.0 + far, g.1 + side);
        (a, g, 1)
    }

    /// Whether the lance would fire at the frame, with the frame flying at `v` and trying to fly
    /// at `want` (quarter pixels a tick).
    fn fires(far: i32, side: i32, v: (i32, i32), want: (i32, i32)) -> bool {
        let (mut a, g, j) = lance_and_frame(far, side);
        ((a.ships[j].vx, a.ships[j].vy), a.ships[j].want) = (v, want);
        a.shot_meets(j, g, (1, 0), LANCE_SPEED, LANCE_LIFE - 1)
    }

    #[test]
    fn a_lance_fires_where_its_shot_and_the_ship_will_both_be() {
        // Sitting still: anywhere on the line the shot will reach, and not beyond.
        assert!(fires(90, 0, (0, 0), (0, 0)));
        assert!(!fires(120, 0, (0, 0), (0, 0)), "it fired at a ship sitting further off than its shot flies");
        assert!(!fires(60, 6, (0, 0), (0, 0)), "it fired at a ship sitting beside its line");
        // Coming straight down the line at 3 px a tick, 90 px off: the shot meets it at about
        // 50 px. The old check looked for it 90 px off with a 22 tick lead, found nothing there,
        // and held.
        assert!(fires(90, 0, (-12, 0), (-12, 0)), "it held its fire at a ship flying straight down its line at it");
        // Flying straight off at 3 px a tick, the shot gains a pixel a tick. From 20 px it
        // catches it at 80. From 30 px it would need 120, further than it flies, and it holds.
        assert!(fires(20, 0, (12, 0), (12, 0)));
        assert!(!fires(30, 0, (12, 0), (12, 0)), "it fired at a ship it can't catch before the shot is spent");
        // Flying off as fast as the shot flies, never.
        assert!(!fires(4, 0, (16, 0), (16, 0)));
    }

    #[test]
    fn a_lance_holds_a_shot_that_needs_the_ship_to_do_one_thing_exactly() {
        // The frame is 3 px wide. Crossing the line at 2 px a tick, 60 px out, it will be 30 px
        // away by the time the shot is there: on the line now, the shot misses if it flies on.
        assert!(!fires(60, 0, (0, 8), (0, 8)), "it fired at where a crossing ship is, not where it will be");
        // 30 px short of the line and crossing towards it, the shot lands if it flies on and
        // misses if it stops. A guess: held.
        assert!(!fires(60, -30, (0, 8), (0, 8)), "it fired a shot that only lands if the ship doesn't stop");
        // Flying down the line but trying to turn across it: lands if it carries on, misses once
        // it has turned. Held.
        assert!(!fires(90, 0, (-12, 0), (0, 12)), "it fired a shot that only lands if the ship doesn't finish its turn");
        // The same ship with nothing else in mind: fired at.
        assert!(fires(90, 0, (-12, 0), (-12, 0)));
        // Crossing right under the gun, 8 px out, it can't get off the line in the two ticks
        // the shot takes: fired at.
        assert!(fires(8, 0, (0, 2), (0, 2)));
    }

    #[test]
    fn a_lance_fires_at_a_crossing_ship_that_cant_clear_its_line_in_time() {
        // A ship 13 px across, crossing the line at 1 px a tick, 40 px off. The shot takes about
        // 10 ticks and the ship moves about 10 px, so some 3 px of it are on the line whether it
        // flies on or stops: not the same pixels, but a shot down a line through them lands
        // either way. A gun on such a line fires. One on a line through its middle holds (the
        // ship would have flown clear), and so does one on a line it hasn't reached yet.
        let crossing = |rows: &[&str], side: i32| {
            let gun = Design::from_rows("GUN", Order::Hold, &["hlh", "hch"]);
            let mut a = Arena::new();
            a.add(0, 0, &gun, 150, 120);
            let s = &a.ships[0];
            let l = (0..s.hp.len()).find(|&c| s.shape.cells[c] == Some(Part::Lance)).unwrap();
            let g = s.cell_xy(l);
            a.add(1, 0, &Design::from_rows("WALL", Order::Hold, rows), g.0 + 40, g.1 + side);
            ((a.ships[1].vx, a.ships[1].vy), a.ships[1].want) = ((0, 4), (0, 4));
            a.shot_meets(1, g, (1, 0), LANCE_SPEED, LANCE_LIFE - 1)
        };
        // (Drawn 13 wide facing up; placed facing west, that's 13 px across the gun's line.)
        let wide = ["hhhhhhhhhhhhh", "hhhhhhchhhhhh", "hhhhhhhhhhhhh"];
        assert!(crossing(&wide, -5), "it held a shot that lands on a wide ship whether it flies on or stops");
        assert!(!crossing(&wide, 0), "it fired down a line the ship will have flown clear of");
        assert!(!crossing(&wide, -12), "it fired down a line the ship only reaches if it flies on");
        // The same crossing by a ship 3 px across: it moves three times its width while the shot
        // is in flight, and no line catches it both ways.
        let narrow = ["hhh", "hch", "hhh"];
        assert!((-14..=4).all(|side| !crossing(&narrow, side)), "it fired at a narrow ship crossing far off");
    }

    #[test]
    fn a_lance_counts_only_the_pixels_it_can_reach() {
        // A 9x9 slab whose near edge is on the last pixel a lance shot reaches is fired at. One
        // pixel further out it isn't. The old check counted pixels up to the far side of whatever
        // had its middle in range, and against a big ship those shots were spent in flight.
        let reach = |centre: i32| {
            let gun = Design::from_rows("GUN", Order::Hold, &["hlh", "hch"]);
            let mut a = Arena::new();
            a.add(0, 0, &gun, 150, 120);
            let s = &a.ships[0];
            let l = (0..s.hp.len()).find(|&c| s.shape.cells[c] == Some(Part::Lance)).unwrap();
            let g = s.cell_xy(l);
            a.add(1, 0, &slab('h'), g.0 + centre, g.1);
            a.shot_meets(1, g, (1, 0), LANCE_SPEED, LANCE_LIFE - 1)
        };
        assert!(reach(LANCE_LIFE - 1 + 4), "a slab with its near edge on the last pixel the shot reaches wasn't fired at");
        assert!(!reach(LANCE_LIFE + 4), "a slab with its near edge a pixel past where the shot is spent was fired at");
        // A ship with a hollow nose: on the gun's line its two near cells are empty and its core
        // is the third. With the core one pixel past where the shot is spent, the nose is in
        // reach and nothing of it on the line is. Not fired at.
        let hollow = Design::from_rows("HOLLOW", Order::Hold, &["h.h", "h.h", "hch"]);
        let core_at = |k: i32| {
            let gun = Design::from_rows("GUN", Order::Hold, &["hlh", "hch"]);
            let mut a = Arena::new();
            a.add(0, 0, &gun, 150, 120);
            let s = &a.ships[0];
            let l = (0..s.hp.len()).find(|&c| s.shape.cells[c] == Some(Part::Lance)).unwrap();
            let g = s.cell_xy(l);
            a.add(1, 0, &hollow, g.0 + k - 1, g.1);
            let t = &a.ships[1];
            let core = (0..t.hp.len()).find(|&c| t.shape.is_core(c)).unwrap();
            assert_eq!(t.cell_xy(core), (g.0 + k, g.1), "the hollow ship's core isn't where this test puts it");
            a.shot_meets(1, g, (1, 0), LANCE_SPEED, LANCE_LIFE - 1)
        };
        assert!(core_at(LANCE_LIFE - 1), "a core on the last pixel the shot reaches wasn't fired at");
        assert!(!core_at(LANCE_LIFE), "a ship with its nose in reach and nothing on the line in reach was fired at");
    }

    #[test]
    fn a_lance_shot_reaches_everything_the_gun_fires_at() {
        // A slab with its near edge on the last pixel the gun will fire at. It fires, and the
        // shot gets there: what the gun counts as in reach and how far the shot flies are one
        // number.
        let gun = Design::from_rows("GUN", Order::Hold, &["hlh", "hch"]);
        let mut a = Arena::new();
        a.add(0, 0, &gun, 150, 120);
        let l = (0..a.ships[0].hp.len()).find(|&c| a.ships[0].shape.cells[c] == Some(Part::Lance)).unwrap();
        a.ships[0].cool[l] = 0;
        let g = a.ships[0].cell_xy(l);
        a.add(1, 0, &slab('h'), g.0 + LANCE_LIFE - 1 + 4, g.1);
        let full = a.ships[1].px_now();
        for _ in 0..LANCE_LIFE / LANCE_SPEED + 3 {
            a.step();
        }
        assert_eq!(a.score[0].fired, 1, "the lance should have fired once");
        assert!(a.ships[1].px_now() < full, "the shot was spent before it reached what it was fired at");
    }

    #[test]
    fn a_ship_shows_the_speed_its_trying_to_fly_at() {
        // A lancer told to hunt a slab far off. A tick in, it's still gathering speed, and what
        // it's trying for is flat out. A hulk with no engines is trying for nothing.
        let lancer = Design::from_rows("LANCER", Order::Hunt, &[".l.", "hlh", "hch", "e.e"]);
        let mut a = Arena::new();
        a.add(0, 0, &lancer, 100, 180);
        a.add(1, 0, &slab('h'), 380, 180);
        a.step();
        let s = &a.ships[0];
        let (v, w) = (isqrt(s.vx * s.vx + s.vy * s.vy), isqrt(s.want.0 * s.want.0 + s.want.1 * s.want.1));
        assert!(v < s.st.vmax && w >= s.st.vmax - 1 && w <= s.st.vmax, "a tick in it's flying at {v} and trying for {w}, of a top speed of {}", s.st.vmax);
        assert!(s.want.0 > 0, "it's trying to fly away from its target");
        assert_eq!(a.ships[1].want, (0, 0));
        // Up to speed, what it's trying for is what it's doing.
        for _ in 0..20 {
            a.step();
        }
        let s = &a.ships[0];
        assert_eq!((s.vx, s.vy), s.want, "at full speed on a straight course, it isn't flying as it's trying to");
    }

    #[test]
    fn a_lance_ship_leads_by_the_time_its_shot_really_takes() {
        // Where a lancer has to sit, up or down the map, to bear on a target 80 px east of it
        // that's drifting down the map at 2 px a tick.
        let lancer = Design::from_rows("LANCER", Order::Hunt, &[".l.", "hlh", "hch", "e.e"]);
        let mut a = Arena::new();
        a.add(0, 0, &lancer, 150, 180);
        a.add(1, 0, &slab('h'), 230, 180);
        let line = |a: &Arena, vx: i32, dist: i32| {
            let e = Snap { idx: 1, team: 1, x: 230, y: 180, vx, vy: 8, half: 4, full: 81, working: false, worker: false };
            a.lance_line(0, &e, (0, 0), 1, dist) - 180
        };
        // Holding its distance, the shot takes 80 / 4 = 20 ticks, and the target is 40 px on.
        assert_eq!(line(&a, 0, 80), 40);
        // Coming on at 1 px a tick, the shot meets it in 80 / 5 = 16 ticks: 32 px.
        assert_eq!(line(&a, -4, 80), 32);
        // Drawing off at 2 px a tick, in 80 / 2 = 40 ticks, but the shot only lasts 28: 56 px.
        assert_eq!(line(&a, 8, 80), 56);
        // From 400 px away it used to lead by 100 ticks. The shot it's lining up for lasts 28.
        assert_eq!(line(&a, 0, 400), 56);
        // A target that's lining a gun of its own up on this ship isn't led at all.
        a.ships[1].st.lance = 1;
        a.ships[1].target = Some(0);
        assert_eq!(line(&a, 0, 80), 0, "it led a ship that's coming to its line");
    }

    #[test]
    fn two_lance_ships_lining_up_on_each_other_settle_and_trade_shots() {
        // Two of the starter deck's frigates. Each used to lead the other's sideways speed, so
        // each ran ahead of the other and they flew the length of the map side by side, firing
        // when one got ahead. Now each holds the other's line as it stands.
        let frigate = crate::pg::design::starter_deck().remove(2);
        let mut a = Arena::new();
        a.add(0, 0, &frigate, 150, 180);
        a.add(1, 0, &frigate, 330, 180);
        let (mut lo, mut hi) = (a.ships[0].y, a.ships[0].y);
        while a.tick < 15 * 30 && a.over.is_none() {
            a.step();
            lo = lo.min(a.ships[0].y);
            hi = hi.max(a.ships[0].y);
        }
        assert!(hi - lo <= 90, "blue's frigate ranged {} px up and down the map", hi - lo);
        // They do shoot: four lances each, a volley every few seconds. (How many land is another
        // matter: a lance shot takes 23 ticks to cross the gap they hold, and a frigate shuffles
        // further than its own width in that time.)
        assert!(a.score[0].fired >= 12 && a.score[1].fired >= 12, "in thirty seconds they fired {} and {} shots", a.score[0].fired, a.score[1].fired);
    }

    // ----- The beam and the shield -----

    /// A beam gun of three on a ship with no engines, so it stays where it's put and fires the way
    /// it was placed facing: 76 px of reach, and a cut of 12.
    fn beam_turret() -> Design {
        Design::from_rows("TURRET", Order::Hold, &["hbh", "hbh", "hbh", "hch"])
    }

    /// A 9x9 slab of one part round a core. It stays put.
    fn slab(part: char) -> Design {
        let row: String = std::iter::repeat(part).take(9).collect();
        let mid: String = (0..9).map(|x| if x == 4 { 'c' } else { part }).collect();
        let rows: Vec<&str> = (0..9).map(|y| if y == 4 { mid.as_str() } else { row.as_str() }).collect();
        Design::from_rows("SLAB", Order::Hold, &rows)
    }

    /// Blue's beam turret, rested and ready, facing east: the arena, and where its muzzle is.
    fn turret_ready() -> (Arena, (i32, i32)) {
        let mut a = Arena::new();
        a.add(0, 0, &beam_turret(), 150, 120);
        let (gun, n) = a.ships[0].beam_guns()[0];
        assert_eq!(n, 3);
        a.ships[0].cool[gun] = 0;
        let at = a.ships[0].cell_xy(gun);
        (a, at)
    }

    #[test]
    fn a_beam_shows_its_line_for_a_second_then_cuts_along_it_all_at_once() {
        // A hull slab 50 px out, its middle 2 px to one side of the line: the line runs along one
        // whole row of nine, two rows from its edge.
        let (mut a, (mx, my)) = turret_ready();
        a.add(1, 0, &slab('h'), mx + 50, my + 2);
        let full = a.ships[1].px_now();
        a.step();
        let w = a.warming();
        assert_eq!(w.len(), 1, "with a slab on its line the gun isn't warming");
        assert_eq!((w[0].from, w[0].dir, w[0].reach, w[0].left), ((mx, my), (1, 0), beam_reach(3), BEAM_WARM));
        for _ in 0..BEAM_WARM - 1 {
            a.step();
            assert_eq!(a.ships[1].px_now(), full, "it cut before it had warmed");
            assert_eq!(a.warming().len(), 1);
        }
        a.step();
        // The row of nine is gone in the one tick, and the two rows beyond it, cut off from the
        // core, have broken away.
        assert_eq!(a.ships[1].px_now(), full - 9 - 18, "a cut of 12 through a row of nine hull pixels");
        assert_eq!(a.cuts.len(), 1);
        assert_eq!((a.cuts[0].from, a.cuts[0].to), ((mx, my), (mx + beam_reach(3), my)), "with cut to spare, the line runs to the end of its reach");
        assert_eq!((a.score[0].fired, a.score[0].hits), (1, 1));
        // With nothing left on its line, it doesn't warm again.
        for _ in 0..BEAM_PERIOD + 20 {
            a.step();
            assert!(a.warming().is_empty(), "it warmed with nothing on its line");
        }
    }

    #[test]
    fn plate_uses_up_twice_the_cut_and_the_gun_rests_before_it_warms_again() {
        let (mut a, (mx, my)) = turret_ready();
        a.add(1, 0, &slab('p'), mx + 50, my + 2);
        let (full, left) = (a.ships[1].px_now(), a.ships[1].bbox().0);
        for _ in 0..BEAM_WARM + 1 {
            a.step();
        }
        // Twelve of cut is six plates. It stops in the sixth, with three still standing behind.
        assert_eq!(a.ships[1].px_now(), full - 6);
        assert_eq!(a.cuts[0].to, (left + 5, my), "the line ends where the cut ran out");
        // It rests its five seconds with three plates still on its line, then warms again.
        for _ in 0..BEAM_PERIOD - 1 {
            a.step();
            assert!(a.warming().is_empty(), "it warmed again before it had rested");
        }
        a.step();
        a.step();
        assert_eq!(a.warming().len(), 1, "rested, with plate still on its line, it didn't warm again");
    }

    #[test]
    fn a_beam_reaches_as_far_as_its_length_says_and_no_further() {
        // The slab's near edge is 2 px beyond a gun of three's 76 px: it never warms.
        let (mut a, (mx, my)) = turret_ready();
        a.add(1, 0, &slab('h'), mx + beam_reach(3) + 2 + 4, my);
        for _ in 0..40 {
            a.step();
            assert!(a.warming().is_empty(), "it warmed at a slab out of its reach");
        }
        // Two pixels nearer, the edge is on the last pixel of its reach, and that pixel is cut.
        let (mut a, (mx, my)) = turret_ready();
        a.add(1, 0, &slab('h'), mx + beam_reach(3) + 4, my + 2);
        let full = a.ships[1].px_now();
        for _ in 0..BEAM_WARM + 1 {
            a.step();
        }
        assert_eq!(a.ships[1].px_now(), full - 1);
    }

    #[test]
    fn a_beam_gun_shot_through_the_middle_is_two_shorter_guns() {
        let d = Design::from_rows("LONG", Order::Hold, &["hb", "hb", "hb", "hb", "hb", "hc"]);
        let mut a = Arena::new();
        a.add(0, 0, &d, 150, 120);
        a.tick = 5;
        assert_eq!(a.ships[0].beam_guns(), vec![(1, 5)]);
        assert_eq!(a.ships[0].st.beam_reach(), 100);
        // The third pixel of the column goes. The two in front stay on the ship by the hull
        // beside them, as a gun of their own.
        let at = a.ships[0].cell_xy(5);
        a.kill_cell(0, 5, at, (0, 1));
        a.step();
        assert_eq!(a.ships[0].beam_guns(), vec![(1, 2), (7, 2)]);
        assert_eq!((a.ships[0].st.beam, a.ships[0].st.beam_len, a.ships[0].st.beam_reach()), (4, 2, 64));
    }

    #[test]
    fn a_ship_that_can_get_off_a_warming_beams_line_does_and_one_too_wide_is_cut() {
        // A fighter's frame parked on the line, told to hold: it sidesteps while the beam warms,
        // and the beam cuts where it was. (It has no guns: 50 px out, a fighter's flak would have
        // shot the muzzle off first.)
        let fighter = Design::from_rows("FIGHTER", Order::Hold, &[".h.", "hhh", "hch", "e.e"]);
        let (mut a, (mx, my)) = turret_ready();
        a.add(1, 0, &fighter, mx + 50, my);
        let full = a.ships[1].px_now();
        let mut dodged = false;
        for _ in 0..BEAM_WARM + 1 {
            a.step();
            dodged |= a.ships[1].doing == Doing::Dodge;
        }
        assert_eq!(a.cuts.len(), 1, "the beam should have fired");
        assert!(dodged, "the fighter never tried to get off the line");
        assert_eq!(a.ships[1].px_now(), full, "the fighter was cut all the same");
        // A hull 25 px across with two engines can't be clear of the line in a second, so it
        // doesn't try, and it's cut.
        let wide: String = std::iter::repeat('h').take(25).collect();
        let mid: String = (0..25).map(|x| if x == 12 { 'c' } else { 'h' }).collect();
        let tail = format!("e{}e", ".".repeat(23));
        let barge = Design::from_rows("BARGE", Order::Hold, &[wide.as_str(), wide.as_str(), mid.as_str(), wide.as_str(), tail.as_str()]);
        let (mut a, (mx, my)) = turret_ready();
        a.add(1, 0, &barge, mx + 50, my);
        let full = a.ships[1].px_now();
        for _ in 0..BEAM_WARM + 1 {
            a.step();
            assert!(a.ships[1].doing != Doing::Dodge, "the barge tried a dodge it couldn't make");
        }
        assert!(a.ships[1].px_now() < full, "the barge on the line wasn't cut");
    }

    #[test]
    fn a_ship_with_a_beam_holds_off_and_lines_it_up_like_a_lance() {
        // The beamer of the examples hunts a slab that sits still, 80 px off its row: flying
        // straight at the slab it would never have it on its line. It makes for a place in line
        // with the slab at about three quarters of its beam's 148 px, never dives in, and within
        // forty seconds it has cut it.
        let beamer = crate::pg::design::part_examples().remove(0);
        let mut a = Arena::new();
        a.add(0, 0, &beamer, 100, 120);
        a.add(1, 0, &slab('h'), 360, 200);
        let (full, mut nearest, mut cuts) = (a.ships[1].px_now(), i32::MAX, 0);
        while a.tick < 15 * 40 && a.ships[1].alive {
            a.step();
            assert!(a.ships[0].doing != Doing::Dodge, "the beamer dodged its own beam");
            if let (Doing::StandOff, Some(g)) = (a.ships[0].doing, a.ships[0].goal) {
                let d = isqrt(dist2(g.0, g.1, a.ships[1].x, a.ships[1].y));
                assert!(d >= 97, "holding off, it made for a place {d} px from its target");
            }
            nearest = nearest.min(isqrt(dist2(a.ships[0].x, a.ships[0].y, a.ships[1].x, a.ships[1].y)));
            // A cut is shown for three ticks: it has two left at the end of the tick it's made.
            cuts += a.cuts.iter().filter(|c| c.t == 2).count();
        }
        assert!(cuts >= 1 && a.ships[1].px_now() < full, "in forty seconds the beamer fired {cuts} times and the slab lost {} px", full - a.ships[1].px_now());
        assert!(nearest >= 90, "a ship whose beam reaches 148 px came within {nearest} px of its target");
    }

    #[test]
    fn a_beam_starts_to_warm_on_a_target_that_will_be_on_its_line_when_it_cuts() {
        // A hull frame 24 px beyond a gun of three's reach, on its line. Parked there, it's out
        // of reach and the gun stays cold. Flying down the line at 3 px a tick, it will be well
        // inside the reach a second from now, and the gun starts to warm at once.
        let frame = Design::from_rows("FRAME", Order::Hold, &[".h.", "hhh", "hch", "e.e"]);
        let warming_after_a_tick = |vx: i32| {
            let (mut a, (mx, my)) = turret_ready();
            a.add(1, 0, &frame, mx + beam_reach(3) + 24, my);
            a.ships[1].vx = vx;
            a.step();
            a.warming().len()
        };
        assert_eq!(warming_after_a_tick(0), 0, "it warmed at a ship parked out of its reach");
        assert_eq!(warming_after_a_tick(-12), 1, "it didn't warm at a ship flying down its line into its reach");
    }

    #[test]
    fn a_beam_ship_and_a_lance_ship_lining_up_on_each_other_dont_chase_each_other_about() {
        // The beamer and the warden of the examples. Each lines its gun up on the other. Leading
        // each other, they ran side by side from one edge of the map to the other and back, and
        // the beamer fired three times in a minute. Holding each other's line, they stay near
        // the row they met on and get on with it.
        let ex = crate::pg::design::part_examples();
        let mut a = Arena::new();
        a.add(0, 0, &ex[0], 150, 180);
        a.add(1, 1, &ex[1], 330, 180);
        let (mut lo, mut hi) = (a.ships[0].y, a.ships[0].y);
        while a.tick < 15 * 40 && a.over.is_none() {
            a.step();
            lo = lo.min(a.ships[0].y);
            hi = hi.max(a.ships[0].y);
        }
        assert!(hi - lo <= 90, "the beamer ranged {} px up and down the map", hi - lo);
        assert!(a.score[0].shielded >= 4, "in forty seconds only {} of the beamer's cuts reached the warden's circles", a.score[0].shielded);
    }

    /// A hull hulk with one shield pixel on its nose, and no engines or guns.
    fn egg() -> Design {
        Design::from_rows("EGG", Order::Hold, &["hoh", "hhh", "hch"])
    }

    /// Red's egg, where its shield pixel is, and which way out from the egg that is (-1: west).
    fn egg_alone() -> (Arena, usize, (i32, i32), i32) {
        let mut a = Arena::new();
        a.add(1, 0, &egg(), 300, 120);
        a.tick = 5;
        let s = &a.ships[0];
        let c = (0..s.hp.len()).find(|&c| s.shape.cells[c] == Some(Part::Shield)).unwrap();
        let (sx, sy) = s.cell_xy(c);
        assert_eq!(sy, s.y);
        let out = (sx - s.x).signum();
        assert!(out != 0);
        (a, c, (sx, sy), out)
    }

    /// Fires a flak shot of blue's from `from`, flying along the row, and steps until it's gone.
    /// Returns the tick it went on.
    fn flak_from(a: &mut Arena, from: (i32, i32), dx: i32) -> u32 {
        let shot = flak_at(a, from, (from.0 + dx * 60, from.1));
        a.shots.push(shot);
        while !a.shots.is_empty() {
            a.step();
        }
        a.tick
    }

    #[test]
    fn a_shield_stops_four_shots_coming_in_and_is_back_three_seconds_after_the_last() {
        let (mut a, c, (sx, sy), out) = egg_alone();
        let full = a.ships[0].px_now();
        assert_eq!(a.ships[0].charge[c], SHIELD_CHARGES);
        let mut last = 0;
        for n in 1..=SHIELD_CHARGES {
            last = flak_from(&mut a, (sx + out * 30, sy), -out);
            assert_eq!(a.ships[0].charge[c], SHIELD_CHARGES - n, "shot {n} didn't cost the circle a charge");
            assert_eq!((a.ships[0].px_now(), a.score[0].shielded as u8), (full, n), "shot {n} wasn't stopped at the circle");
        }
        assert!(a.ships[0].live_shields().is_empty(), "with no charge left the circle should be out");
        // The fifth, at the hull beside the shield, goes through.
        flak_from(&mut a, (sx + out * 30, sy + 1), -out);
        assert_eq!((a.ships[0].px_now(), a.score[0].shielded as u8), (full - 1, SHIELD_CHARGES), "with the circle out, a shot was still stopped");
        // It comes back full, all at once, when it has gone its three seconds since the last
        // shot it stopped.
        while a.tick < last + SHIELD_REST as u32 - 1 {
            a.step();
            assert_eq!(a.ships[0].charge[c], 0, "it came back before it had rested");
        }
        a.step();
        assert_eq!(a.ships[0].charge[c], SHIELD_CHARGES, "three seconds after the last shot it stopped, it isn't back");
    }

    #[test]
    fn a_shot_stopped_starts_the_circles_rest_again() {
        // Two shots stopped two seconds apart: it's full three seconds after the second, not
        // the first.
        let (mut a, c, (sx, sy), out) = egg_alone();
        let first = flak_from(&mut a, (sx + out * 30, sy), -out);
        while a.tick < first + 30 {
            a.step();
        }
        let second = flak_from(&mut a, (sx + out * 30, sy), -out);
        while a.tick < first + SHIELD_REST as u32 + 5 {
            a.step();
        }
        assert_eq!(a.ships[0].charge[c], SHIELD_CHARGES - 2, "it was full three seconds after the first shot, with a second stopped since");
        while a.tick < second + SHIELD_REST as u32 {
            a.step();
        }
        assert_eq!(a.ships[0].charge[c], SHIELD_CHARGES);
    }

    #[test]
    fn a_shot_fired_from_inside_a_circle_isnt_stopped_by_it() {
        // From 5 px out, inside the 10 px circle, at the shield pixel itself.
        let (mut a, c, (sx, sy), out) = egg_alone();
        let full = a.ships[0].px_now();
        flak_from(&mut a, (sx + out * 5, sy), -out);
        assert_eq!((a.score[0].shielded, a.ships[0].hp[c], a.ships[0].px_now()), (0, 0, full - 1), "a shot from inside the circle should hit what it's aimed at");
    }

    #[test]
    fn a_circle_covers_a_friend_inside_it() {
        // A one-pixel friend 6 px out from the shield pixel, shot at from beyond the circle.
        let dot = Design::from_rows("DOT", Order::Hold, &["c"]);
        let fired_at_the_dot = |charged: bool| {
            let (mut a, c, (sx, sy), out) = egg_alone();
            a.add(1, 1, &dot, sx + out * 6, sy);
            if !charged {
                // Out, with its rest still to come.
                (a.ships[0].charge[c], a.ships[0].cool[c]) = (0, SHIELD_REST);
            }
            flak_from(&mut a, (sx + out * 30, sy), -out);
            (a.ships[1].alive, a.score[0].shielded)
        };
        assert_eq!(fired_at_the_dot(true), (true, 1), "a friend inside the circle was hit");
        assert_eq!(fired_at_the_dot(false), (false, 0), "with the circle out, the friend should have been hit");
    }

    #[test]
    fn a_circle_stops_a_beam_and_a_shell_as_it_stops_flak() {
        // The beam turret, with the egg on its line 50 px out.
        let (mut a, (mx, my)) = turret_ready();
        a.add(1, 0, &egg(), mx + 50, my);
        let s = &a.ships[1];
        let c = (0..s.hp.len()).find(|&c| s.shape.cells[c] == Some(Part::Shield)).unwrap();
        let ((sx, sy), full) = (s.cell_xy(c), s.px_now());
        assert_eq!(sy, my);
        for _ in 0..BEAM_WARM + 1 {
            a.step();
        }
        assert_eq!(a.cuts.len(), 1, "the beam should have fired");
        assert_eq!(a.cuts[0].to, (sx - SHIELD_R - 1, my), "the beam should end at the circle's edge");
        assert_eq!((a.ships[1].px_now(), a.ships[1].charge[c], a.score[0].shielded, a.score[0].hits), (full, SHIELD_CHARGES - 1, 1, 0));
        // A shell at the same egg: stopped, with no blast.
        let line = Line::new(sx - 30, sy, sx + 30, sy);
        a.shots.push(Shot { team: 0, kind: Kind::Shell, line, left: CANNON_RANGE, speed: SHELL_SPEED, dmg: CANNON_DMG, pen: 1, target: None, prev: (sx - 30, sy), connected: false, born: a.tick, acc: 0, cover: (0, 0) });
        while !a.shots.is_empty() {
            a.step();
        }
        assert_eq!((a.ships[1].px_now(), a.ships[1].charge[c], a.score[0].shielded), (full, SHIELD_CHARGES - 2, 2));
        assert!(a.sparks.iter().all(|s| !s.blast), "a shell stopped at a circle burst");
    }

    #[test]
    fn a_ship_that_moves_onto_a_shot_doesnt_take_it_inside_its_circle() {
        // The egg has just moved 3 px towards a shot that's now 9 px from its shield pixel. Where
        // the egg stands now the shot is inside the circle already; where the egg stood when the
        // tick began it was 12 px off, outside. It came into the circle, so it's stopped.
        let (mut a, c, (sx, sy), out) = egg_alone();
        let full = a.ships[0].px_now();
        let shot = flak_at(&a, (sx + out * 9, sy), (sx - out * 60, sy));
        a.shots.push(shot);
        let s = &mut a.ships[0];
        s.pose0 = (s.x - out * 3, s.y, s.facing);
        a.shots_step();
        assert_eq!((a.score[0].shielded, a.ships[0].charge[c], a.ships[0].px_now()), (1, SHIELD_CHARGES - 1, full), "the shot got inside the circle by the ship moving onto it");
        // The other way about: the egg has just moved 3 px away from a shot that was 9 px from
        // its shield pixel, inside the circle. The shot is 12 px off now and flies 6 px nearer.
        // It never came into the circle: it was in it. So it isn't stopped.
        let (mut a, _, (sx, sy), out) = egg_alone();
        let shot = flak_at(&a, (sx + out * 12, sy), (sx - out * 60, sy));
        a.shots.push(shot);
        let s = &mut a.ships[0];
        s.pose0 = (s.x + out * 3, s.y, s.facing);
        a.shots_step();
        assert_eq!((a.score[0].shielded, a.shots.len()), (0, 1), "a shot already inside the circle was stopped when the ship moved off it");
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

    // ----- The missile -----

    /// A launcher on a stand: one missile pixel, no engines, on HOLD.
    fn launcher() -> Design {
        Design::from_rows("LAUNCHER", Order::Hold, &["hmh", "hch"])
    }

    /// Blue's launcher at (100, 180), ready to fire, and where its missile pixel is.
    fn launcher_ready() -> (Arena, (i32, i32)) {
        let mut a = Arena::new();
        a.add(0, 0, &launcher(), 100, 180);
        let s = &a.ships[0];
        let m = (0..s.hp.len()).find(|&c| s.shape.cells[c] == Some(Part::Missile)).unwrap();
        let at = s.cell_xy(m);
        a.ships[0].cool[m] = 0;
        (a, at)
    }

    fn missiles(a: &Arena) -> Vec<Shot> {
        a.shots.iter().copied().filter(|s| s.kind == Kind::Missile).collect()
    }

    /// A missile of blue's at a point, after a ship.
    fn missile_at(a: &mut Arena, (x, y): (i32, i32), target: usize) {
        let (tx, ty) = (a.ships[target].x, a.ships[target].y);
        a.shots.push(Shot { team: 0, kind: Kind::Missile, line: Line::new(x, y, tx, ty), left: 0, speed: 1000, dmg: CANNON_DMG, pen: 1, target: Some(target), prev: (x, y), connected: false, born: a.tick, acc: 0, cover: (0, 0) });
    }

    #[test]
    fn a_launcher_sends_a_missile_at_the_nearest_enemy_in_reach_whichever_way_it_faces() {
        // Above, below, behind: a launcher needs no lining up. Out of reach it holds its fire.
        for (dx, dy, fires) in [(150, 0, true), (0, -150, true), (-80, 0, true), (120, 120, true), (MISSILE_RANGE + 10, 150, false), (MISSILE_RANGE + 20, 0, false)] {
            let (mut a, at) = launcher_ready();
            a.add(1, 0, &slab('h'), at.0 + dx, at.1 + dy);
            a.step();
            assert_eq!(missiles(&a).len(), fires as usize, "a slab {dx},{dy} away");
        }
        // Of two in reach it's the nearer, and it reloads before the next.
        let (mut a, at) = launcher_ready();
        a.add(1, 0, &slab('h'), at.0 + 150, at.1);
        a.add(1, 0, &slab('h'), at.0, at.1 + 90);
        a.step();
        assert_eq!(missiles(&a)[0].target, Some(2));
        for _ in 0..MISSILE_PERIOD - 2 {
            a.step();
        }
        assert_eq!(a.score[0].fired, 1, "it fired again before it had reloaded");
        for _ in 0..3 {
            a.step();
        }
        assert_eq!(a.score[0].fired, 2);
    }

    #[test]
    fn a_missile_blasts_a_hole_like_a_shells_and_plate_goes_with_it() {
        for part in ['h', 'p'] {
            let (mut a, at) = launcher_ready();
            a.add(1, 0, &slab(part), at.0 + 120, at.1);
            let full = a.ships[1].px_now();
            for _ in 0..15 * 6 {
                a.step();
                if a.ships[1].px_now() < full {
                    break;
                }
            }
            let lost = full - a.ships[1].px_now();
            assert!((3..=9).contains(&lost), "one missile took {lost} px of a slab of '{part}'");
            assert_eq!(a.score[0].hits, 1);
            assert!(missiles(&a).is_empty(), "the missile flew on after it burst");
        }
    }

    #[test]
    fn a_missile_flies_as_fast_on_a_diagonal_as_along_a_row_and_no_faster_than_the_fastest_ship() {
        // Thirty ticks at a far target: 90 px, give or take the last step, whichever way.
        for (dx, dy) in [(200, 0), (0, 150), (150, 150), (150, -150), (200, 100), (-85, 150)] {
            let mut a = Arena::new();
            a.add(1, 0, &slab('h'), 140 + dx, 180 + dy);
            assert_eq!((a.ships[0].x, a.ships[0].y), (140 + dx, 180 + dy));
            missile_at(&mut a, (140, 180), 0);
            for _ in 0..30 {
                a.tick += 1;
                a.shots_step();
            }
            let m = missiles(&a)[0];
            let flown = isqrt(dist2(m.line.x, m.line.y, 140, 180));
            assert!((88..=91).contains(&flown), "toward {dx},{dy} it flew {flown} px in 30 ticks");
        }
        // A ship flying off at 3 px a tick, the fastest there is, is never caught from behind.
        // One at 2 px a tick is.
        let chase = |speed: i32| {
            let mut a = Arena::new();
            a.add(1, 0, &slab('h'), 105, 180);
            let full = a.ships[0].px_now();
            missile_at(&mut a, (55, 180), 0);
            for _ in 0..MISSILE_LIFE + 2 {
                let s = &mut a.ships[0];
                s.pose0 = (s.x, s.y, s.facing);
                s.x += speed;
                a.tick += 1;
                a.shots_step();
            }
            (full - a.ships[0].px_now(), missiles(&a).len())
        };
        assert_eq!(chase(3), (0, 0), "a missile caught a ship as fast as itself, or outlived its time");
        assert!(chase(2).0 > 0, "a missile never caught a ship slower than itself");
    }

    #[test]
    fn a_missile_turns_after_a_target_that_moves_and_flies_on_when_its_target_is_gone() {
        // The target jumps from ahead to above: the missile's next steps go up, not on.
        let mut a = Arena::new();
        a.add(1, 0, &slab('h'), 400, 300);
        missile_at(&mut a, (200, 300), 0);
        for _ in 0..10 {
            a.tick += 1;
            a.shots_step();
        }
        let before = missiles(&a)[0];
        assert!(before.line.x > 225 && before.line.y == 300);
        (a.ships[0].x, a.ships[0].y) = (before.line.x, 100);
        a.ships[0].pose0 = (a.ships[0].x, a.ships[0].y, a.ships[0].facing);
        for _ in 0..10 {
            a.tick += 1;
            a.shots_step();
        }
        let after = missiles(&a)[0];
        assert!(after.line.x == before.line.x && after.line.y < 275, "it didn't turn: {:?}", (after.line.x, after.line.y));
        // Its target dies: it flies on the way it was going, and is spent when its time is up.
        a.ships[0].alive = false;
        for _ in 0..10 {
            a.tick += 1;
            a.shots_step();
        }
        let on = missiles(&a)[0];
        assert!(on.line.x == after.line.x && on.line.y < after.line.y - 25);
        // It's spent by the clock: one that flies its whole time without reaching anything is
        // gone the tick its time is up, and not before.
        let mut a = Arena::new();
        a.add(1, 0, &slab('h'), 60, 180);
        a.ships[0].x = 30 + 3 * MISSILE_LIFE as i32 + 40;
        a.ships[0].pose0 = (a.ships[0].x, a.ships[0].y, a.ships[0].facing);
        assert!(a.ships[0].x < a.w - 10, "the test needs the target on the map");
        missile_at(&mut a, (30, 180), 0);
        for _ in 0..MISSILE_LIFE - 1 {
            a.tick += 1;
            a.shots_step();
        }
        assert_eq!(missiles(&a).len(), 1, "a missile was spent before its time");
        a.tick += 1;
        a.shots_step();
        assert!(missiles(&a).is_empty(), "a missile outlived its time");
    }

    #[test]
    fn flak_shoots_a_missile_down_before_it_gets_there() {
        // A slab with flak along the side the missile comes from, against one with none.
        let screened = Design::from_rows("SCREENED", Order::Hold, &["ghhhh", "ghhhh", "ghchh", "ghhhh", "ghhhh"]);
        let bare = Design::from_rows("BARE", Order::Hold, &["hhhhh", "hhhhh", "hhchh", "hhhhh", "hhhhh"]);
        let run = |target: &Design| {
            let (mut a, at) = launcher_ready();
            a.add(1, 0, target, at.0 + 150, at.1);
            let full = a.ships[1].px_now();
            for _ in 0..MISSILE_PERIOD - 1 {
                a.step();
            }
            (full - a.ships[1].px_now(), a.score[1].downed, a.score[0].fired)
        };
        let (lost, downed, fired) = run(&bare);
        assert!(fired == 1 && lost >= 3 && downed == 0, "with no flak: lost {lost}, downed {downed}");
        let (lost, downed, fired) = run(&screened);
        assert_eq!((fired, lost, downed), (1, 0, 1), "five flak guns let a single missile through");
    }

    #[test]
    fn flak_sends_two_shots_at_a_missile_and_keeps_the_rest_for_the_next() {
        // Twenty flak guns with a missile coming: two shots go its way, not twenty.
        let wall = Design::from_rows("WALL", Order::Hold, &["gggg", "gggg", "ggcg", "gggg", "gggg"]);
        let mut a = Arena::new();
        a.add(1, 0, &wall, 300, 180);
        for c in 0..a.ships[0].cool.len() {
            a.ships[0].cool[c] = 0;
        }
        let far = Arena::new().ships.len();
        assert_eq!(far, 0);
        // Blue has nothing on the field but the missile, so nothing else draws their fire.
        a.shots.push(Shot { team: 0, kind: Kind::Missile, line: Line::new(300 - FLAK_RANGE + 4, 180, 300, 180), left: 0, speed: 1000, dmg: CANNON_DMG, pen: 1, target: Some(0), prev: (0, 0), connected: false, born: a.tick, acc: 0, cover: (0, 0) });
        a.step();
        let at_it = a.shots.iter().filter(|s| s.kind == Kind::Flak).count();
        assert_eq!(at_it, 2, "{at_it} flak shots went at one missile");
        // A flak gun with a ship in reach and no missile fires at the ship, as ever.
        let mut a = Arena::new();
        a.add(1, 0, &wall, 300, 180);
        a.add(0, 0, &slab('h'), 300 - 30, 180);
        for c in 0..a.ships[0].cool.len() {
            a.ships[0].cool[c] = 0;
        }
        a.step();
        assert!(a.shots.iter().filter(|s| s.kind == Kind::Flak).count() >= 10);
    }

    #[test]
    fn a_shield_stops_a_missile_for_one_charge_and_a_beam_cuts_one_down() {
        // Into a circle from outside: stopped at its edge for a charge, like any shot.
        let egg = Design::from_rows("EGG", Order::Hold, &["hhh", "hoh", "hch"]);
        let (mut a, at) = launcher_ready();
        a.add(1, 0, &egg, at.0 + 120, at.1);
        let full = a.ships[1].px_now();
        for _ in 0..MISSILE_PERIOD - 1 {
            a.step();
        }
        assert_eq!((full - a.ships[1].px_now(), a.score[0].shielded), (0, 1), "a missile got through a full shield");
        assert!(missiles(&a).is_empty());
        // On a beam's line when it cuts: gone, and counted to the beam's side. One beside the
        // line, and one of the beam's own side on it, fly on.
        let (mut a, (mx, my)) = turret_ready();
        a.add(1, 0, &slab('h'), mx + 300, my + 200);
        for (team, dx, dy) in [(1, 30, 0), (1, 30, 5), (0, 40, 0)] {
            a.shots.push(Shot { team, kind: Kind::Missile, line: Line::new(mx + dx, my + dy, mx + 300, my + 200), left: 0, speed: 1000, dmg: CANNON_DMG, pen: 1, target: None, prev: (0, 0), connected: false, born: a.tick, acc: 0, cover: (0, 0) });
        }
        a.cut(0, (mx, my), (1, 0), 3);
        assert_eq!((missiles(&a).len(), a.score[0].downed), (2, 1));
        assert!(missiles(&a).iter().any(|m| m.team == 0) && missiles(&a).iter().any(|m| m.line.y == my + 5));
    }

    #[test]
    fn a_ship_with_missiles_is_armed_and_holds_off_at_their_range_when_its_told_to_kite() {
        let archer = part_examples().into_iter().find(|d| d.name == "ARCHER").unwrap();
        let st = archer.stats();
        assert!(st.armed() && st.missile == 2 && !st.aims());
        // Launchers alone make a ship armed: it's a warship to the AI, and it's flown as one.
        assert!(launcher().stats().armed() && launcher().stats().flak == 0);
        assert_eq!((st.made, st.used), (12, 12), "an example ship should spend the power it makes");
        assert!(archer.check().ok());
        // Its card says KITE: it holds off at two thirds of a missile's reach, from the target's edge.
        assert_eq!(archer.flies(), Some(Style::Kite));
        let mut a = Arena::new();
        a.add(0, 0, &archer, 100, 180);
        a.add(1, 0, &slab('p'), 100 + MISSILE_RANGE + 80, 180);
        for _ in 0..15 * 20 {
            a.step();
        }
        let gap = (a.ships[1].x - a.ships[0].x).abs() - a.ships[1].half();
        assert!((MISSILE_RANGE * 2 / 3 - 12..=MISSILE_RANGE * 2 / 3 + 12).contains(&gap), "it holds off at {gap} px");
        assert!(a.score[0].fired > 0, "it never launched from there");
        // Left as drawn (it has the thrust for it) the same ship makes attack runs: it follows
        // what its card says, whatever its guns are.
        let mut runner = archer.clone();
        runner.style = None;
        assert_eq!(runner.flies(), Some(Style::Runs));
    }

    // ----- The repeater -----

    /// A repeater gun three long on a stand, on HOLD.
    fn rake() -> Design {
        Design::from_rows("RAKE", Order::Hold, &["hrh", "hrh", "hrh", "hch"])
    }

    /// Blue's rake at (100, 180) facing east, its gun ready, and where its muzzle is.
    fn rake_ready(d: &Design) -> (Arena, (i32, i32)) {
        let mut a = Arena::new();
        a.add(0, 0, d, 100, 180);
        let guns = a.ships[0].repeat_guns();
        for &(c, _) in &guns {
            a.ships[0].cool[c] = 0;
        }
        let at = a.ships[0].cell_xy(guns[0].0);
        (a, at)
    }

    #[test]
    fn a_repeater_fires_a_stream_straight_ahead_at_what_will_be_on_its_line() {
        // Ahead and in reach (three pixels: 84 px): a shot every seven ticks. Out of reach,
        // or off its line, nothing.
        for (dx, dy, fires) in [(60, 0, true), (84 + 3, 0, true), (84 + 6, 0, false), (60, 20, false), (0, 60, false), (-60, 0, false)] {
            let (mut a, at) = rake_ready(&rake());
            a.add(1, 0, &slab('p'), at.0 + dx, at.1 + dy);
            for _ in 0..70 {
                a.step();
            }
            let fired = a.score[0].fired;
            assert_eq!(fired > 0, fires, "a slab {dx},{dy} from the muzzle: {fired} shots");
            if fires {
                assert_eq!(fired, 10, "a three-pixel gun should fire every {} ticks", repeat_period(3));
            }
        }
        // A shot that meets nothing is spent at the end of its gun's reach. The gun fires at a
        // lone core 50 px out, which the first shot to land kills with the next already on its
        // way. A hull slab stands behind it, its near edge 96 px out: past the 84 the gun reaches.
        let (mut a, at) = rake_ready(&rake());
        a.add(1, 0, &Design::from_rows("DOT", Order::Hold, &["c"]), at.0 + 50, at.1);
        a.add(1, 0, &slab('h'), at.0 + 100, at.1);
        let full = a.ships[2].px_now();
        for _ in 0..60 {
            a.step();
        }
        assert!(!a.ships[1].alive && a.score[0].fired >= 2, "the core should be dead with a second shot fired: {} fired", a.score[0].fired);
        assert_eq!(a.ships[2].px_now(), full, "a shot flew on past its gun's reach and hit the slab behind");
        // Three abreast are three guns, each on its own line: a target on one line is fired at
        // by that one alone, and slower.
        let (mut a, _) = rake_ready(&Design::from_rows("ROW", Order::Hold, &["rrr", "hch"]));
        let guns: Vec<(i32, i32)> = a.ships[0].repeat_guns().iter().map(|g| a.ships[0].cell_xy(g.0)).collect();
        assert_eq!(guns.len(), 3);
        let top = *guns.iter().min_by_key(|g| g.1).unwrap();
        a.add(1, 0, &Design::from_rows("DOT", Order::Hold, &["c"]), top.0 + 50, top.1);
        for _ in 0..8 {
            a.step();
        }
        assert_eq!((a.score[0].fired, a.shots.iter().filter(|s| s.kind == Kind::Repeat).all(|s| s.line.y == top.1)), (1, true));
    }

    #[test]
    fn a_repeaters_shots_break_hull_one_pixel_each_and_plate_turns_them() {
        let run = |part: char| {
            let (mut a, at) = rake_ready(&rake());
            a.add(1, 0, &slab(part), at.0 + 60, at.1);
            let full = a.ships[1].px_now();
            for _ in 0..15 * 4 {
                a.step();
            }
            (full - a.ships[1].px_now(), a.score[0].hits, a.score[0].bounced)
        };
        let (lost, hits, bounced) = run('h');
        assert!(lost >= 5 && lost == hits && bounced == 0, "against hull: {lost} px for {hits} hits, {bounced} bounced");
        let (lost, _, bounced) = run('p');
        assert!(lost == 0 && bounced >= 5, "against plate: {lost} px lost, {bounced} bounced");
    }

    #[test]
    fn a_stream_empties_a_shield_and_keeps_it_empty() {
        // Red's egg, its shield toward the gun. One lance shot would cost it a charge and be
        // gone. The stream costs it all four inside two seconds and then every shot lands.
        let (mut a, at) = rake_ready(&rake());
        a.add(1, 0, &Design::from_rows("EGG", Order::Hold, &["hoh", "hhh", "hhh", "hch"]), at.0 + 60, at.1);
        let o = (0..a.ships[1].hp.len()).find(|&c| a.ships[1].shape.cells[c] == Some(Part::Shield)).unwrap();
        let full = a.ships[1].px_now();
        let mut emptied = None;
        for t in 0..15 * 8 {
            a.step();
            if a.ships[1].charge[o] == 0 && emptied.is_none() {
                emptied = Some(t);
            }
            if let Some(t0) = emptied {
                assert!(a.ships[1].charge[o] == 0 || a.ships[1].hp[o] == 0, "the circle came back {} ticks after the stream emptied it", t - t0);
            }
        }
        assert!(emptied.is_some_and(|t| t <= 15 * 3), "the stream never emptied the circle: {emptied:?}");
        assert_eq!(a.score[0].shielded, SHIELD_CHARGES as i32, "the circle stopped more than its four");
        assert!(a.ships[1].px_now() < full, "nothing got through once the circle was down");
    }

    #[test]
    fn a_repeater_shot_takes_a_missile_down_and_its_ship_lines_up_like_a_lance_ship() {
        // A red missile crossing a blue repeater shot's path: both are gone.
        let mut a = Arena::new();
        a.add(0, 0, &slab('h'), 60, 60);
        a.tick = 3;
        a.shots.push(Shot { team: 1, kind: Kind::Missile, line: Line::new(200, 150, 200, 300), left: 0, speed: 1000, dmg: CANNON_DMG, pen: 1, target: None, prev: (200, 150), connected: false, born: a.tick, acc: 0, cover: (0, 0) });
        a.shots.push(Shot { team: 0, kind: Kind::Repeat, line: Line::new(195, 153, 400, 153), left: 80, speed: REPEAT_SPEED, dmg: FLAK_DMG, pen: 1, target: None, prev: (195, 153), connected: false, born: a.tick, acc: 0, cover: (0, 0) });
        a.shots_step();
        assert_eq!((a.shots.len(), a.score[0].downed), (0, 1));
        // The example ship flies as lance ships do: onto its target's row, holding off at
        // three quarters of its gun's reach, and fires from there.
        let raker = part_examples().into_iter().find(|d| d.name == "RAKER").unwrap();
        let mut a = Arena::new();
        a.add(0, 0, &raker, 80, 120);
        a.add(1, 0, &slab('p'), 380, 220);
        for _ in 0..15 * 30 {
            a.step();
        }
        assert!((a.ships[0].y - a.ships[1].y).abs() <= 5, "it never got onto its target's row: {} against {}", a.ships[0].y, a.ships[1].y);
        let gap = a.ships[1].x - a.ships[0].x - a.ships[1].half();
        let want = repeat_reach(10) * 3 / 4;
        assert!((want - 20..=want + 20).contains(&gap), "it holds off at {gap} px, not about {want}");
        assert!(a.score[0].fired > 20, "it fired {} shots in thirty seconds", a.score[0].fired);
    }

    // ----- The projector -----

    /// A projector knob of twelve pixels on a neck, on a base with the power for it. It stays put.
    fn tower() -> Design {
        Design::from_rows("TOWER", Order::Hold, &["..jj..", ".jjjj.", ".jjjj.", "..jj..", "..hh..", "hhhhhh", "hhhhhh", "hhchhh", "hhhhhh"])
    }

    /// Red's tower at (300, 180) and a red slab `gap` px clear of its knob, above it. Team 0
    /// has nothing: its shots are put in by hand. Gives the arena after one tick, so that what
    /// covers what has been worked out.
    fn tower_and_friend(gap: i32) -> Arena {
        let mut a = Arena::new();
        a.add(1, 0, &tower(), 300, 180);
        let lump: Vec<(i32, i32)> = a.ships[0].proj_lumps()[0].iter().map(|&c| a.ships[0].cell_xy(c)).collect();
        let (top, mid) = (lump.iter().map(|p| p.1).min().unwrap(), lump.iter().map(|p| p.0).sum::<i32>() / lump.len() as i32);
        a.add(1, 0, &slab('h'), mid, top - gap - 5);
        assert_eq!(top - (a.ships[1].y + 4), gap + 1, "the slab's bottom edge should be {gap} px clear of the knob");
        a.step();
        a
    }

    /// A flak shot of team 0's fired along a row from the left at ship `i`, flown until it's gone.
    fn flak_into(a: &mut Arena, i: usize, row: i32) {
        let (l, _, _, _) = a.ships[i].bbox();
        let shot = flak_at(a, (l - 20, row), (l + 40, row));
        a.shots.push(shot);
        while !a.shots.is_empty() {
            a.step();
        }
    }

    #[test]
    fn a_projector_stops_shots_at_a_friend_in_its_reach_and_not_at_one_beyond_it() {
        // Twelve pixels reach 30 px. A friend 25 px clear of the knob is covered: the shot stops
        // at the pixel it would have hit, and the pool is one down.
        let mut a = tower_and_friend(25);
        assert_eq!((a.ships[0].st.projector, a.ships[0].skin.0, proj_reach(12)), (12, 12, 30));
        assert_eq!(a.skins[1], vec![0]);
        let full = a.ships[1].px_now();
        let row = a.ships[1].y;
        flak_into(&mut a, 1, row);
        assert_eq!((a.ships[1].px_now(), a.ships[0].skin.0, a.score[0].shielded, a.score[0].hits), (full, 11, 1, 0));
        // One 35 px clear isn't: the shot lands.
        let mut a = tower_and_friend(35);
        assert!(a.skins[1].is_empty());
        let row = a.ships[1].y;
        flak_into(&mut a, 1, row);
        assert_eq!((a.ships[1].px_now(), a.ships[0].skin.0, a.score[0].shielded), (full - 1, 12, 0));
        // The enemy's ships are never covered, however near.
        let mut a = tower_and_friend(25);
        a.add(0, 0, &slab('h'), a.ships[0].x + 30, a.ships[0].y);
        a.step();
        assert!(a.skins[2].is_empty(), "a projector covered an enemy ship");
    }

    #[test]
    fn a_projector_never_covers_its_own_ship_and_no_projector_pixel_is_ever_covered() {
        // A tower with a friend in reach: the friend is covered, the tower isn't. A shot at the
        // tower's base lands, and its pool, which is full, is never asked.
        let mut a = tower_and_friend(25);
        let (full, knob) = (a.ships[0].px_now(), a.ships[0].st.projector);
        assert_eq!((a.skins[0].len(), a.skins[1].clone()), (0, vec![0]), "the friend should be covered and the tower not");
        // Team 1 faces west, so the knob is the tower's west end and its base is to the east.
        let (row, (_, _, r, _)) = (a.ships[0].y, a.ships[0].bbox());
        let shot = flak_at(&a, (r + 20, row + 2), (r - 40, row + 2));
        a.shots.push(shot);
        while !a.shots.is_empty() {
            a.step();
        }
        assert_eq!((a.ships[0].px_now(), a.ships[0].st.projector, a.ships[0].skin.0, a.score[0].shielded), (full - 1, knob, knob, 0), "a projector covered its own ship");
        // Two towers side by side cover each other, and still neither itself.
        let mut a = Arena::new();
        a.add(1, 0, &tower(), 300, 180);
        a.add(1, 0, &tower(), 300, 192);
        a.step();
        assert_eq!((a.skins[0].clone(), a.skins[1].clone()), (vec![1], vec![0]), "two towers side by side should each cover the other and not itself");
        // A shot at the first one's base is stopped, and it's the second that pays.
        let (row, (_, _, r, _)) = (a.ships[0].y, a.ships[0].bbox());
        let shot = flak_at(&a, (r + 20, row - 2), (r - 40, row - 2));
        a.shots.push(shot);
        while !a.shots.is_empty() {
            a.step();
        }
        assert_eq!((a.ships[0].px_now(), a.ships[0].skin.0, a.ships[1].skin.0, a.score[0].shielded), (full, knob, knob - 1, 1), "the tower beside it should have stopped the shot");
        // A shot at the first one's knob lands all the same (from the west, along its middle
        // row, the knob is what it meets): no projector pixel is covered, whoever's skin is near.
        flak_into(&mut a, 0, row);
        assert_eq!((a.ships[0].st.projector, a.ships[1].skin.0, a.score[0].shielded), (knob - 1, knob - 1, 1), "a projector pixel was covered");
    }

    #[test]
    fn a_projectors_pool_is_a_charge_a_pixel_and_is_full_again_after_its_rest() {
        let mut a = tower_and_friend(25);
        let full = a.ships[1].px_now();
        // Twelve shots are stopped, one after another. The thirteenth lands.
        for k in 0..12 {
            let row = a.ships[1].y - 4 + (k % 9);
            flak_into(&mut a, 1, row);
            assert_eq!((a.ships[1].px_now(), a.ships[0].skin.0), (full, 11 - k), "shot {} of twelve", k + 1);
        }
        let row = a.ships[1].y;
        flak_into(&mut a, 1, row);
        assert_eq!((a.ships[1].px_now(), a.score[0].shielded), (full - 1, 12));
        // Three seconds after the last it stopped, the pool is whole again: not a tick sooner.
        let mut a = tower_and_friend(25);
        let row = a.ships[1].y;
        flak_into(&mut a, 1, row);
        let stopped = a.tick;
        while a.tick < stopped + PROJ_REST as u32 - 2 {
            a.step();
            assert_eq!(a.ships[0].skin.0, 11, "the pool came back {} ticks after a shot was stopped", a.tick - stopped);
        }
        for _ in 0..4 {
            a.step();
        }
        assert_eq!(a.ships[0].skin.0, 12);
    }

    #[test]
    fn a_friend_is_covered_by_one_projector_at_a_time_the_nearest_with_a_charge() {
        // The first tower's knob is 26 px from the slab. A second tower, put down later, has its
        // knob 6 px from it. Both reach it, and a shot costs the nearer one: the second.
        let mut a = tower_and_friend(25);
        a.add(1, 0, &tower(), a.ships[1].x + 14, a.ships[1].y);
        a.step();
        assert_eq!(a.skins[1], vec![2, 0], "both towers should reach the slab, the nearer one first");
        let (near, other) = (2, 0);
        let row = a.ships[1].y;
        flak_into(&mut a, 1, row);
        assert_eq!((a.ships[near].skin.0, a.ships[other].skin.0), (11, 12));
        // With the nearer one's pool empty, the other pays.
        a.ships[near].skin = (0, 1000);
        flak_into(&mut a, 1, row);
        assert_eq!((a.ships[near].skin.0, a.ships[other].skin.0), (0, 11));
    }

    #[test]
    fn a_beam_and_a_burst_each_cost_a_skin_one_charge() {
        // A beam's cut through a covered ship stops at its first pixel, for one charge.
        // (The turret is only there to own the cut: its gun is kept resting, so the one cut is
        // the one made by hand.)
        let (mut a, (mx, my)) = turret_ready();
        let gun = a.ships[0].beam_guns()[0].0;
        a.ships[0].cool[gun] = 10_000;
        a.add(1, 0, &tower(), mx + 40, my + 30);
        a.add(1, 0, &slab('h'), mx + 40, my);
        a.step();
        assert_eq!(a.skins[2], vec![1], "the slab on the beam's line should be covered by the tower below it");
        let full = a.ships[2].px_now();
        a.cut(0, (mx, my), (1, 0), 3);
        assert_eq!((a.ships[2].px_now(), a.ships[1].skin.0, a.score[0].shielded), (full, 11, 1));
        let to = a.cuts.last().unwrap().to;
        assert_eq!(to.0, a.ships[2].x - 4, "the cut should end on the covered ship's first pixel");
        // A shell that would have burst on it: stopped, no hole, one charge.
        let (l, _, _, _) = a.ships[2].bbox();
        a.shots.push(Shot { team: 0, kind: Kind::Shell, line: Line::new(l - 20, my, l + 40, my), left: CANNON_RANGE, speed: SHELL_SPEED, dmg: CANNON_DMG, pen: 1, target: None, prev: (l - 20, my), connected: false, born: a.tick, acc: 0, cover: (0, 0) });
        while !a.shots.is_empty() {
            a.step();
        }
        assert_eq!((a.ships[2].px_now(), a.ships[1].skin.0, a.score[0].shielded), (full, 10, 2));
    }

    #[test]
    fn a_projector_that_loses_pixels_reaches_less_and_holds_less() {
        // Twelve pixels reach 30 px and cover a friend 25 px off. With four shot away, eight
        // reach 20: the friend isn't covered any more, and the pool holds eight.
        let mut a = tower_and_friend(25);
        assert_eq!(a.skins[1], vec![0]);
        // The four furthest from the neck go (the tower faces west, so they're its westmost),
        // which leaves the other eight joined to each other and to the ship.
        let mut lump = a.ships[0].proj_lumps()[0].clone();
        lump.sort_by_key(|&c| a.ships[0].cell_xy(c));
        for &c in lump.iter().take(4) {
            a.ships[0].hp[c] = 0;
        }
        a.ships[0].dirty = true;
        a.step();
        a.step();
        assert_eq!((a.ships[0].st.projector, a.ships[0].skin.0, proj_reach(8)), (8, 8, 20));
        assert!(a.skins[1].is_empty(), "eight pixels still covered a friend 25 px off");
    }

    #[test]
    fn a_burst_on_a_bare_projector_pixel_spares_the_covered_hull_beside_it_for_one_charge() {
        // Two towers side by side, each covering the other. A shell comes down on the first
        // one's knob pixel next to its neck. Projector pixels are bare, so it bursts there: the
        // knob pixels in its 3x3 go, and the two neck pixels in it, which the second tower's
        // skin covers, are spared for one of the second tower's charges between them.
        let mut a = Arena::new();
        a.add(1, 0, &tower(), 300, 180);
        a.add(1, 0, &tower(), 300, 192);
        a.step();
        assert_eq!(a.skins[0], vec![1]);
        let s = &a.ships[0];
        let hull = |a: &Arena| (0..a.ships[0].hp.len()).filter(|&c| a.ships[0].hp[c] > 0 && a.ships[0].shape.cells[c] == Some(Part::Hull)).count();
        let by_hull = |c: usize| {
            let (x, y) = s.cell_xy(c);
            [(1, 0), (-1, 0), (0, 1), (0, -1)].iter().any(|&(dx, dy)| s.cell_at(x + dx, y + dy).is_some_and(|n| s.shape.cells[n] == Some(Part::Hull)))
        };
        let (x, y) = s.proj_lumps()[0].iter().copied().filter(|&c| by_hull(c)).map(|c| s.cell_xy(c)).min_by_key(|p| p.1).unwrap();
        let (base, knob) = (hull(&a), a.ships[0].st.projector);
        a.shots.push(Shot { team: 0, kind: Kind::Shell, line: Line::new(x, y - 30, x, y + 30), left: CANNON_RANGE, speed: SHELL_SPEED, dmg: CANNON_DMG, pen: 1, target: None, prev: (x, y - 30), connected: false, born: a.tick, acc: 0, cover: (0, 0) });
        while !a.shots.is_empty() {
            a.step();
        }
        assert!(a.ships[0].st.projector <= knob - 5, "the burst should have taken five knob pixels: {} of {knob} left", a.ships[0].st.projector);
        assert_eq!((hull(&a), a.ships[1].skin.0, a.score[0].shielded), (base, 11, 1), "the covered neck beside the burst wasn't spared for one charge");
    }

    #[test]
    fn two_lumps_on_one_ship_each_reach_by_their_own_size_and_share_the_ships_pool() {
        // A knob of twelve at each end: each reaches 30 px, where 24 in one lump would reach 40.
        let twin = Design::from_rows(
            "TWIN",
            Order::Hold,
            &["..jj..", ".jjjj.", ".jjjj.", "..jj..", "..hh..", "hhhhhh", "hhhhhh", "hhhhhh", "hhchhh", "hhhhhh", "hhhhhh", "hhhhhh", "hhhhhh", "..hh..", "..jj..", ".jjjj.", ".jjjj.", "..jj.."],
        );
        assert!(twin.check().ok(), "{:?}", twin.check().problem());
        assert_eq!((proj_reach(12), proj_reach(24)), (30, 40));
        for (gap, covered) in [(25, true), (34, false)] {
            let mut a = Arena::new();
            a.add(1, 0, &twin, 300, 180);
            assert_eq!(a.ships[0].proj_lumps().len(), 2);
            let west = a.ships[0].proj_lumps().iter().flatten().map(|&c| a.ships[0].cell_xy(c).0).min().unwrap();
            a.add(1, 0, &slab('h'), west - gap - 5, 180);
            a.step();
            assert_eq!(a.skins[1] == vec![0], covered, "a friend {} px from the nearer knob", gap + 1);
            assert_eq!(a.ships[0].skin.0, 24, "the two knobs fill one pool");
        }
    }

    #[test]
    fn a_covered_ships_own_circle_stops_a_shot_before_the_skin_is_asked() {
        // Red's tower, and red's egg above its knob and in its reach: covered. A shot at the
        // egg's shield side stops at the egg's own circle, which loses a charge. The
        // projector's pool isn't touched.
        let mut a = Arena::new();
        a.add(1, 0, &tower(), 300, 180);
        let lump: Vec<(i32, i32)> = a.ships[0].proj_lumps()[0].iter().map(|&c| a.ships[0].cell_xy(c)).collect();
        let (top, mid) = (lump.iter().map(|p| p.1).min().unwrap(), lump.iter().map(|p| p.0).sum::<i32>() / lump.len() as i32);
        a.add(1, 0, &egg(), mid, top - 22);
        a.step();
        assert_eq!(a.skins[1], vec![0], "the egg should be covered by the tower");
        let o = (0..a.ships[1].hp.len()).find(|&c| a.ships[1].shape.cells[c] == Some(Part::Shield)).unwrap();
        let (ox, oy) = a.ships[1].cell_xy(o);
        let full = a.ships[1].px_now();
        // The egg faces west, so its circle's pixel is its west end: the shot comes from there.
        let shot = flak_at(&a, (ox - 30, oy), (ox + 10, oy));
        a.shots.push(shot);
        while !a.shots.is_empty() {
            a.step();
        }
        assert_eq!((a.ships[1].charge[o], a.ships[0].skin.0, a.ships[1].px_now()), (SHIELD_CHARGES - 1, 12, full));
    }
}
