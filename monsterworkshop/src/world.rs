//! The world as data: region maps (`data/*.map`) laid side by side, with their workshops,
//! bins, plants, project boards and the footbridge, plus the items lying around. No monsters
//! here; `game.rs` moves them and asks the world what is where.

use crate::data::{matches, Data};
use crate::econ::ItemData;
pub use crate::workshops::ShopKind;
use serde::{Deserialize, Serialize};

pub const TS: i32 = 16;
pub const SCREEN_W: i32 = 25;
pub const SCREEN_H: i32 = 16;
pub const DISPENSE_TICKS: u8 = 4;
/// Town workshops' built-in tools, used when nothing better hangs on the peg.
pub const BUILTIN_TOOL_QUALITY: f64 = 0.5;
pub const TOWN_WORKSHOP_QUALITY: f64 = 0.8;
/// Finished goods a workshop holds inside, beyond the one on its output mat.
pub const OUTPUT_BUFFER: usize = 9;
/// How far beyond what it's about (where a good is and where it goes, who's moving it) a search
/// for a way looks: two screens each way. A world of a hundred screens is too big to search
/// whole many times a tick, and a way that goes further round isn't found.
pub const SEARCH_MARGIN: (i32, i32) = (2 * SCREEN_W, 2 * SCREEN_H);

// `#` hedge  `.` grass  `,` street  `W` wall  `_` floor  `D` door  `Y` tree  `w` water
// `F` where the footbridge starts, at the west bank  `s` signpost  `b` project board
// `X` water where the bridge will go  `U` the town wagon  `P` its pumpkin
// `Z` the builders' bench's work spot (`B` an ordinary bench's)
// `C` cotton bin and `m` its chute  `T` cloth store (a drop-off for fabric)  `a` applicant
// stand in the gathering hall  `1` `2` monster starts
// Plants: `f` cotton  `h` hemlock  `k` oak  `e` elder  `r` rhododendron  `j` arrow bamboo
// `q` ramie  `n` indigofera  `l` limestone outcrop
// Work spots are marked by their kind's `map_char` in `data/workshops.json` (`B` workbench,
// `S` spinnery, `L` loom house...). Each `p` pad, `g` tool peg and `o` output mat belongs to the
// nearest work spot.

/// Every character the map legend above gives a meaning, so no workshop kind takes one.
pub const MAP_LEGEND: &str = "#.,W_DYwFsbXUPCmTa12fhkerjqnlpgovcuHGNQxy";

/// A region's map, plus what its text-only features say, in the order they appear.
struct Region {
    map: &'static str,
    /// Where its northwest corner lies in the world, in tiles. Between regions is hedge.
    at: (i32, i32),
    /// Building names, each placed by one tile of the building's top wall (region coordinates).
    labels: &'static [((i32, i32), &'static str)],
    /// Signpost texts and the way their arrows point.
    signs: &'static [(&'static str, Dir)],
    /// Project boards: the project, the good it wants, how many, and what finishing it does.
    boards: &'static [(&'static str, &'static str, u32, Project)],
    /// A working townsperson's supplies (`G`): the good each sets out, and the raw material it's
    /// plainly made from.
    supplies: &'static [(&'static str, &'static str)],
    /// Working townsfolk (`N`), where each starts.
    workers: &'static [WorkerSpec],
}

/// A working townsperson as a region's data gives them: a monster of the town's on a loop at a
/// bench of their own (see `Game::found_town`).
pub struct WorkerSpec {
    pub name: &'static str,
    /// What they are: "WAGONWRIGHT".
    pub title: &'static str,
    /// Their card's lines, and what they say, one at a time (after what their bench makes).
    pub about: &'static [&'static str],
    pub says: &'static [&'static str],
    pub skills: &'static [(&'static str, f64)],
    /// Their loop, in region coordinates.
    pub tasks: &'static [Task],
}

/// One goal of a working townsperson's loop.
#[derive(Clone, Copy, Debug)]
pub enum Task {
    /// Get this good from where it lies to there.
    Move(&'static str, (i32, i32), (i32, i32)),
    /// Work the bench whose spot this is.
    Work((i32, i32)),
}

/// A working townsperson in the world: their spec, where they start, and their loop in world
/// coordinates.
#[derive(Clone)]
pub struct Worker {
    pub name: &'static str,
    pub title: &'static str,
    pub about: &'static [&'static str],
    pub says: &'static [&'static str],
    pub skills: &'static [(&'static str, f64)],
    pub at: (i32, i32),
    pub tasks: Vec<Task>,
}

/// Where a working townsperson's goods come from: a good set out on its tile whenever theirs has
/// been taken, after a while.
#[derive(Clone, Debug)]
pub struct Supply {
    pub at: (i32, i32),
    pub good: &'static str,
    pub from: &'static str,
    /// The monster whose it is (`None` until the town is founded).
    pub owner: Option<u32>,
    pub timer: u32,
}

/// Scenery in a yard: you walk round it, and examining it says what it is.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Prop {
    Wheels,
    Cart,
    Fence,
}

/// Each region's entities are listed after the previous region's, so adding a region never
/// reorders what an older save refers to: the road west to east, then Temple Town south of it.
const REGIONS: [Region; 4] = [
    Region {
        map: include_str!("../data/pinewater.map"),
        at: (0, 0),
        labels: &[((4, 19), "GATHERING HALL"), ((25, 27), "OLD SAWGRASS"), ((30, 17), "OLD SPOKE'S YARD")],
        signs: &[("TANGLEMERE", Dir::Right)],
        boards: &[("OLD SPOKE'S ORDER", "wagon_wheel", 4, Project::Order)],
        supplies: &[("oak_log", "oak_log"), ("planks", "hemlock_log"), ("elder_sticks", "elder_sticks")],
        workers: &[OLD_SPOKE],
    },
    Region {
        map: include_str!("../data/wilderness.map"),
        at: (50, 0),
        supplies: &[],
        workers: &[],
        labels: &[],
        signs: &[("TANGLEMERE", Dir::Right), ("TEMPLE TOWN", Dir::Down)],
        boards: &[("A ROPE FOR THE FOOTBRIDGE", "rope", 3, Project::Footbridge), ("A BRIDGE ACROSS THE RIVER", "log", 12, Project::Bridge)],
    },
    Region {
        map: include_str!("../data/tanglemere.map"),
        at: (125, 0),
        labels: &[((34, 2), "CLOTH HALL")],
        signs: &[("PINEWATER", Dir::Left)],
        boards: &[],
        supplies: &[],
        workers: &[],
    },
    Region {
        map: include_str!("../data/temple_town.map"),
        at: (75, 32),
        labels: &[((12, 18), "TEMPLE")],
        signs: &[],
        boards: &[("THE TEMPLE BELL", "temple_bell", 1, Project::Bell)],
        supplies: &[],
        workers: &[],
    },
];

/// Pinewater's wagonwright, after Xi Zhong, who legend says made the first cart. He fetches his
/// wood to his bench, makes a wheel and takes it to his order board.
const OLD_SPOKE: WorkerSpec = WorkerSpec {
    name: "OLD SPOKE",
    title: "WAGONWRIGHT",
    about: &["HE MAKES WAGON WHEELS AT HIS OWN BENCH, FROM HIS OWN WOOD. WATCH HOW HE DOES IT.", "THE BENCH BY HIS GATE IS FOR ANYONE. HIS ORDER BOARD TAKES ANYONE'S WHEELS, FOR RENOWN.", "SPACE: ASK HIM SOMETHING."],
    says: &["OAK MAKES THE STRONGEST SPOKES. NOTHING ELSE WILL DO FOR A WHEEL.", "TWO WHEELS, PLANKS, AN OAK LOG AND A PUMPKIN TO PULL IT: THAT'S A WAGON. MY GATE BENCH MAKES THOSE TOO.", "MY ORDERS TAKE ANYONE'S WHEELS. THERE'S ALWAYS A CART WANTING ONE."],
    skills: &[("wagon_wright", 0.8), ("carpentry", 0.6), ("hauling", 0.4)],
    tasks: &[
        Task::Move("oak_log", (28, 19), (31, 20)),
        Task::Move("planks", (28, 21), (30, 21)),
        Task::Move("planks", (28, 21), (30, 20)),
        Task::Move("elder_sticks", (29, 23), (30, 22)),
        Task::Move("elder_sticks", (29, 23), (32, 20)),
        Task::Work((31, 21)),
        Task::Move("wagon_wheel", (32, 21), (34, 21)),
    ],
};

/// How long a working townsperson's supply takes to set out a fresh good once theirs is taken:
/// four seconds, so a second plank from the same stack is a short wait.
pub const SUPPLY_TICKS: u32 = 60;

/// Named stretches of the world by their northwest corner, announced on arrival: the road west
/// to east, then Temple Town south of it.
const AREAS: [(i32, i32, &str); 6] = [(0, 0, "PINEWATER"), (50, 0, "TINLESS"), (75, 0, "THATCH HILLS"), (100, 0, "DYE MEADOWS"), (125, 0, "TANGLEMERE"), (75, 32, "TEMPLE TOWN")];

/// The world's height, the rows of regions above together: a made-up world's copies repeat it.
const WORLD_ROWS: i32 = 64;

pub fn area_name(x: i32, y: i32) -> &'static str {
    let y = y.rem_euclid(WORLD_ROWS);
    AREAS.iter().rev().find(|&&(x0, y0, _)| x >= x0 && y >= y0).map_or(AREAS[0].2, |a| a.2)
}

fn plant_good(ch: char) -> Option<&'static str> {
    Some(match ch {
        'f' => "cotton_bolls",
        'h' => "hemlock_log",
        'k' => "oak_log",
        'e' => "elder_sticks",
        'r' => "rhododendron_sticks",
        'j' => "arrow_bamboo",
        'q' => "ramie_stalks",
        'n' => "indigofera_leaves",
        'l' => "limestone",
        'v' => "pumpkin",
        _ => return None,
    })
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub enum Dir {
    Up,
    Down,
    Left,
    Right,
}

pub const DIRS: [Dir; 4] = [Dir::Up, Dir::Down, Dir::Left, Dir::Right];

impl Dir {
    pub fn opposite(self) -> Dir {
        match self {
            Dir::Up => Dir::Down,
            Dir::Down => Dir::Up,
            Dir::Left => Dir::Right,
            Dir::Right => Dir::Left,
        }
    }

    pub fn delta(self) -> (i32, i32) {
        match self {
            Dir::Up => (0, -1),
            Dir::Down => (0, 1),
            Dir::Left => (-1, 0),
            Dir::Right => (1, 0),
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Tile {
    Grass,
    Path,
    Hedge,
    Wall,
    Floor,
    Door,
    Pad,
    Peg,
    OutMat,
    Spot,
    /// Bins and boards: you push goods into them.
    Bin,
    Water,
    Tree,
    Sign,
    /// Planks over the river, once the bridge project is done.
    Bridge,
    /// A rope footbridge over the river, once its project is done: monsters cross it and push
    /// goods over it, but wagons don't fit.
    Footbridge,
}

impl Tile {
    pub fn walkable(self) -> bool {
        !matches!(self, Tile::Hedge | Tile::Wall | Tile::Bin | Tile::Water | Tile::Tree | Tile::Sign)
    }

    /// Items slide over anything a monster can walk on except a work spot.
    pub fn item_floor(self) -> bool {
        self.walkable() && self != Tile::Spot
    }
}

/// A move in progress, drawn as an offset back toward the tile it came from.
#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub struct Slide {
    pub dir: Dir,
    pub left: u8,
    pub total: u8,
}

impl Slide {
    pub fn offset(s: Option<Slide>) -> (i32, i32) {
        match s {
            Some(s) => {
                let (dx, dy) = s.dir.delta();
                let k = TS * s.left as i32 / s.total as i32;
                (-dx * k, -dy * k)
            }
            None => (0, 0),
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Item {
    /// Its own, for keeping track of it wherever it goes; given out the first time something
    /// needs it (0 until then).
    #[serde(default)]
    pub id: u32,
    pub x: i32,
    pub y: i32,
    pub data: ItemData,
    pub slide: Option<Slide>,
    pub hop: u8,
    pub locked: bool, // on a pad that is being worked
    /// The working townsperson whose it is (a monster's id): only they move it.
    #[serde(default)]
    pub owner: Option<u32>,
}

impl Item {
    pub fn new(x: i32, y: i32, data: ItemData) -> Self {
        Self { id: 0, x, y, data, slide: None, hop: 0, locked: false, owner: None }
    }

    pub fn good(&self) -> &str {
        &self.data.good
    }
}

/// A timed job at a plant or a work spot.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Job {
    pub monster: usize,
    pub recipe: String,
    pub t: u32,
    pub total: u32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Plant {
    pub x: i32,
    pub y: i32,
    pub good: String,
    pub growth: u32,
    pub regrow: u32,
    /// Gathered goods waiting to come out, one at a time, like a chute.
    pub stock: Vec<ItemData>,
    pub harvest: Option<Job>,
    /// Where the goods come out: the tile the harvester faced, and that direction.
    pub drop: Option<((i32, i32), Dir)>,
    pub timer: u8,
}

impl Plant {
    pub fn ripe(&self) -> bool {
        self.growth >= self.regrow
    }

    pub fn stage(&self) -> u8 {
        if self.ripe() || !self.stock.is_empty() {
            2
        } else if self.growth >= self.regrow / 2 {
            1
        } else {
            0
        }
    }
}

/// The craftsman a map's `c` stands for: Old Sawgrass, after Lu Ban, the legendary master
/// carpenter, who's said to have invented the saw when a toothed blade of grass cut his hand.
pub const CRAFTSMAN: &str = "OLD SAWGRASS";

/// The trader a map's `u` stands for (temporary, for testing): a human, since only humans
/// barter, behind the counter of a trader's stall.
pub const TRADER: &str = "THE TRADER";

/// A townsperson who isn't anyone's monster (M3d): someone who stands in their place and
/// blocks the way like anyone.
#[derive(Clone, Debug)]
pub struct Townsfolk {
    pub x: i32,
    pub y: i32,
    pub name: String,
    pub kind: Folk,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Folk {
    /// Says what you could make from what's around him.
    Craftsman,
    /// Says what he'll give for what at the stall beside him.
    Trader,
}

/// A bell hung in the temple: how well it was made, who cast it and how many others had a
/// share in it. The temple's plaque lists them.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Hung {
    pub quality: f64,
    /// What it's cast in ("BRONZE"), if its metal is known.
    #[serde(default)]
    pub metal: String,
    pub caster: String,
    pub others: usize,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Bin {
    pub x: i32,
    pub y: i32,
    /// What it takes: each a good, or a type of good (a tag) it takes any good of.
    pub accepts: Vec<String>,
    pub stock: Vec<ItemData>,
    pub mouth: Option<(i32, i32)>,
    pub timer: u8,
    pub bounce: u8,
    /// A drop-off: goods pushed in are scored for renown.
    pub scores: bool,
    /// Built from a kit out in the wilds; a town bin is bottomless and lasts.
    #[serde(default)]
    pub frontier: Option<FrontierBin>,
}

/// What a bin got from its kit: it holds a limited room, takes one kind of good for life (the
/// first pushed in), and falls apart when nothing has gone in or out for a long while.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct FrontierBin {
    pub kit: String,
    /// Its wood, for its color and its name while it's empty: "HEMLOCK".
    pub material: String,
    /// How much it holds, in storage volume.
    pub room: f64,
    /// Ticks since anything went in or came out, and how many goods it held then.
    pub idle: u32,
    pub held: usize,
}

/// What finishing a project does to the world.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub enum Project {
    /// Ropes strung across the river make a footbridge.
    Footbridge,
    /// The bridge goes up where the river was, and wagons can cross.
    Bridge,
    /// A working townsperson's order: anyone's goods count toward it. Filled, it goes out, and
    /// after a rest a new one starts.
    Order,
    /// The Temple Bell hangs in its frame and rings for everyone. After a rest, a new bell can
    /// replace it.
    Bell,
}

/// A town project: push the goods it wants into the board. Each one pays renown like a
/// delivery, and finishing the project changes the world.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Board {
    pub x: i32,
    pub y: i32,
    pub title: String,
    pub wants: String,
    pub need: u32,
    pub have: u32,
    pub project: Project,
    pub bounce: u8,
    /// Ticks until it takes another (the bell frame, after a bell is hung).
    #[serde(default)]
    pub rest: u32,
}

impl Board {
    pub fn done(&self) -> bool {
        self.have >= self.need
    }
}

pub struct Sign {
    pub x: i32,
    pub y: i32,
    pub text: &'static str,
    pub dir: Dir,
}

/// The town lends its wagon until wagons can be built: its bed's room in storage volume, and
/// the weight it pulls at full speed per point of its leader's CHA.
pub const TOWN_WAGON_ROOM: f64 = 6.0;
pub const TOWN_WAGON_POWER: f64 = 25.0;

/// A wagon and the magical pumpkin that pulls it. The pumpkin walks a tile ahead of the wagon,
/// and a monster leading them walks a tile ahead of the pumpkin.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Wagon {
    pub at: (i32, i32),
    pub pumpkin: (i32, i32),
    pub slide: Option<Slide>,
    pub pumpkin_slide: Option<Slide>,
    /// The goods in its bed, first in first out, each with the area it was loaded in.
    pub stock: Vec<(ItemData, String)>,
    /// How much its bed holds, in storage volume.
    pub room: f64,
    /// Weight it pulls at full speed, per point of its leader's CHA.
    pub power: f64,
    /// Its tailgate: open, a parked wagon sets its goods out behind it.
    pub open: bool,
    /// The monster leading it, and whoever led it last (by id).
    pub leader: Option<u32>,
    pub last_leader: Option<u32>,
    pub timer: u8,
    pub bounce: u8,
    /// Its wood, for its color: "HEMLOCK" for a hemlock wagon from a kit; empty for the town's.
    #[serde(default)]
    pub material: String,
}

impl Wagon {
    /// The tile behind the wagon, away from its pumpkin: where its tailgate sets goods out.
    pub fn behind(&self) -> (i32, i32) {
        (2 * self.at.0 - self.pumpkin.0, 2 * self.at.1 - self.pumpkin.1)
    }

    /// The one kind of good it is carrying, if any.
    pub fn good(&self) -> Option<&str> {
        self.stock.first().map(|(d, _)| d.good.as_str())
    }
}

impl Bin {
    /// Does it take this good: the good itself, or any good of a type it takes?
    pub fn takes(&self, good: &crate::data::GoodType) -> bool {
        // A new bin from a kit takes whatever's pushed in first, and then only that.
        (self.accepts.is_empty() && self.frontier.is_some()) || self.accepts.iter().any(|a| *a == good.id || good.type_tags.contains(a))
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Shop {
    pub kind: ShopKind,
    pub spot: (i32, i32),
    pub pads: Vec<(i32, i32)>,
    pub pegs: Vec<(i32, i32)>,
    pub outs: Vec<(i32, i32)>,
    pub quality: f64,
    pub job: Option<Job>,
    /// Finished goods waiting for a free output mat.
    pub pending: Vec<ItemData>,
    pub pad_flash: Vec<u8>,
    pub out_timer: u8,
    /// Built from a kit out in the wilds (see `frontier.rs`); town workshops have none.
    #[serde(default)]
    pub frontier: Option<crate::frontier::Frontier>,
    /// What its pads last worked toward (`Game::aim`): with them empty, it stays set up for it.
    #[serde(default)]
    pub aimed: Option<String>,
    /// A working townsperson's own bench (their monster's id): only they work it, and what it
    /// makes is theirs.
    #[serde(skip)]
    pub owner: Option<u32>,
}

impl Shop {
    /// What the workshop is set up for while its pads are empty: the recipe whose inputs fill
    /// its pads exactly, else its first. Pinewater's west bench cards cotton; the gate bench
    /// twists rope.
    pub fn default_recipe(&self, data: &Data) -> &'static str {
        let recipes = self.kind.recipes();
        recipes.iter().copied().find(|r| data.good(r).input_groups().count() == self.pads.len()).unwrap_or(recipes[0])
    }

    /// What it rests on with nothing of use on its pads: what they last worked toward, else what
    /// it's set up for at first.
    pub fn resting<'a>(&'a self, data: &Data) -> &'a str {
        self.aimed.as_deref().unwrap_or_else(|| self.default_recipe(data))
    }
}

#[derive(Clone, Copy, Debug)]
pub enum PKind {
    Dust,
    Fluff,
    Spark,
    Leaf,
}

#[derive(Clone, Copy, Debug)]
pub struct Particle {
    pub x: i32,
    pub y: i32,
    pub vx: i32,
    pub vy: i32,
    pub life: u8,
    pub kind: PKind,
}

/// The part of the map a search for a way covers (see `World::search_area`), its tiles numbered
/// for the search's tables.
#[derive(Clone, Copy)]
pub struct Area {
    x0: i32,
    y0: i32,
    w: i32,
    h: i32,
}

impl Area {
    pub fn cells(self) -> usize {
        (self.w * self.h) as usize
    }

    pub fn inside(self, (x, y): (i32, i32)) -> bool {
        x >= self.x0 && y >= self.y0 && x < self.x0 + self.w && y < self.y0 + self.h
    }

    /// A tile's number; it must be inside.
    pub fn idx(self, (x, y): (i32, i32)) -> usize {
        ((y - self.y0) * self.w + x - self.x0) as usize
    }

    pub fn at(self, i: usize) -> (i32, i32) {
        (self.x0 + i as i32 % self.w, self.y0 + i as i32 / self.w)
    }
}

pub struct World {
    pub w: i32,
    pub h: i32,
    pub tiles: Vec<Tile>,
    pub items: Vec<Item>,
    pub plants: Vec<Plant>,
    pub bins: Vec<Bin>,
    pub shops: Vec<Shop>,
    pub particles: Vec<Particle>,
    pub boards: Vec<Board>,
    /// Where the footbridge goes up when its project is done.
    pub footbridge: Vec<(i32, i32)>,
    /// Where the bridge goes up when its project is done.
    pub bridge: Vec<(i32, i32)>,
    pub wagons: Vec<Wagon>,
    pub signs: Vec<Sign>,
    pub labels: Vec<((i32, i32), &'static str)>,
    pub starts: Vec<(i32, i32)>,
    /// Where applicants wait in the gathering hall.
    pub stands: Vec<(i32, i32)>,
    /// Townsfolk who aren't anyone's monster, standing in their places: Old Sawgrass first.
    /// They come from the map, never from a save.
    pub townsfolk: Vec<Townsfolk>,
    /// Plaques on walls: examining one reads the bells hung in the temple.
    pub plaques: Vec<(i32, i32)>,
    /// Working townsfolk, from the map; `Game::found_town` makes them monsters.
    pub workers: Vec<Worker>,
    pub supplies: Vec<Supply>,
    pub props: Vec<((i32, i32), Prop)>,
    /// Every bell hung in the temple, first first.
    pub bells: Vec<Hung>,
    /// The last id given to an item.
    pub last_item_id: u32,
    seed: u32,
}

impl World {
    pub fn new(data: &Data) -> Self {
        Self::build(data, &REGIONS.iter().map(|r| (r.at, r)).collect::<Vec<_>>())
    }

    /// A made-up world for load tests: `copies` of the whole world stacked north to south, with
    /// the hedges between them opened wherever there's ground on both sides, so it walks as one
    /// world, and every river bridged. Saves can't hold it, since a save keeps only what's on the
    /// map and the map comes from the game's data.
    pub fn made_up(data: &Data, copies: usize) -> Self {
        let placed: Vec<((i32, i32), &Region)> = (0..copies as i32).flat_map(|k| REGIONS.iter().map(move |r| ((r.at.0, r.at.1 + k * WORLD_ROWS), r))).collect();
        let mut world = Self::build(data, &placed);
        let rh = WORLD_ROWS;
        for seam in (1..copies as i32).map(|k| k * rh) {
            for x in 0..world.w {
                if world.walkable(x, seam - 2) && world.walkable(x, seam + 1) {
                    world.tiles[((seam - 1) * world.w + x) as usize] = Tile::Grass;
                    world.tiles[(seam * world.w + x) as usize] = Tile::Grass;
                }
            }
        }
        world.build_bridge();
        world
    }

    /// The world from region maps, each with where its northwest corner goes. Whatever no
    /// region covers is hedge.
    fn build(data: &Data, layout: &[((i32, i32), &Region)]) -> Self {
        let placed: Vec<(i32, i32, &Region, Vec<&str>)> = layout.iter().map(|&((ox, oy), r)| (ox, oy, r, r.map.lines().filter(|l| !l.is_empty()).collect())).collect();
        let w = placed.iter().map(|(ox, _, _, rows)| ox + rows[0].len() as i32).max().unwrap_or(0);
        let h = placed.iter().map(|(_, oy, _, rows)| oy + rows.len() as i32).max().unwrap_or(0);
        let mut tiles = vec![Tile::Hedge; (w * h) as usize];
        let (mut plants, mut bins, mut shops, mut starts) = (Vec::new(), Vec::new(), Vec::new(), [None; 2]);
        let (mut stands, mut boards, mut signs, mut labels, mut boat) = (Vec::new(), Vec::new(), Vec::new(), Vec::new(), Vec::new());
        let (mut townsfolk, mut plaques) = (Vec::new(), Vec::new());
        let (mut workers, mut supplies, mut props) = (Vec::new(), Vec::new(), Vec::new());
        let (mut bridge, mut wagon_at, mut pumpkin_at) = (Vec::new(), Vec::new(), Vec::new());
        let (mut mouths, mut parts) = (Vec::new(), Vec::new());
        for (ox, oy, region, rows) in placed {
            let rw = rows[0].len() as i32;
            let (mut sign_texts, mut board_specs) = (region.signs.iter(), region.boards.iter());
            let (mut supply_specs, mut worker_specs) = (region.supplies.iter(), region.workers.iter());
            let at = |(x, y): (i32, i32)| (x + ox, y + oy);
            labels.extend(region.labels.iter().map(|&((x, y), text)| ((x + ox, y + oy), text)));
            for (y, row) in rows.iter().enumerate() {
                assert_eq!(row.len() as i32, rw, "map row {y} is {} wide", row.len());
                for (lx, ch) in row.chars().enumerate() {
                    let (x, y) = (lx as i32 + ox, y as i32 + oy);
                    let street_next = row.as_bytes().get(lx + 1) == Some(&b',') || (lx > 0 && row.as_bytes()[lx - 1] == b',');
                    let shop_kind = ShopKind::by_map_char(ch);
                    let tile = match ch {
                        _ if shop_kind.is_some() => Tile::Spot,
                        '#' => Tile::Hedge,
                        ',' | 'm' => Tile::Path,
                        'W' | 'H' => Tile::Wall,
                        '_' | 'a' | 'c' => Tile::Floor,
                        'D' => Tile::Door,
                        'p' => Tile::Pad,
                        'g' => Tile::Peg,
                        'o' => Tile::OutMat,
                        'C' | 'T' | 'b' => Tile::Bin,
                        'w' | 'F' | 'X' => Tile::Water,
                        'Y' => Tile::Tree,
                        's' | 'Q' | 'x' | 'y' => Tile::Sign,
                        '1' | '2' if street_next => Tile::Path,
                        _ => Tile::Grass,
                    };
                    if let Some(good) = plant_good(ch) {
                        let regrow = data.good(good).production_time;
                        plants.push(Plant { x, y, good: good.into(), growth: regrow, regrow, stock: Vec::new(), harvest: None, drop: None, timer: 0 });
                    }
                    if let Some(kind) = shop_kind {
                        shops.push(Shop { kind, spot: (x, y), pads: Vec::new(), pegs: Vec::new(), outs: Vec::new(), quality: TOWN_WORKSHOP_QUALITY, job: None, pending: Vec::new(), pad_flash: Vec::new(), out_timer: 0, frontier: None, aimed: None, owner: None });
                    }
                    match ch {
                        'C' => bins.push(Bin { x, y, accepts: vec!["cotton_bolls".into()], stock: Vec::new(), mouth: None, timer: 0, bounce: 0, scores: false, frontier: None }),
                        'T' => bins.push(Bin { x, y, accepts: vec!["fabric".into()], stock: Vec::new(), mouth: None, timer: 0, bounce: 0, scores: true, frontier: None }),
                        'b' => {
                            let &(title, wants, need, project) = board_specs.next().expect("a project for every board");
                            boards.push(Board { x, y, title: title.into(), wants: wants.into(), need, have: 0, project, bounce: 0, rest: 0 });
                        }
                        's' => {
                            let &(text, dir) = sign_texts.next().expect("a text for every signpost");
                            signs.push(Sign { x, y, text, dir });
                        }
                        'F' => boat.push((x, y)),
                        'X' => bridge.push((x, y)),
                        'U' => wagon_at.push((x, y)),
                        'P' => pumpkin_at.push((x, y)),
                        'a' => stands.push((x, y)),
                        'c' => townsfolk.push(Townsfolk { x, y, name: CRAFTSMAN.into(), kind: Folk::Craftsman }),
                        'u' => townsfolk.push(Townsfolk { x, y, name: TRADER.into(), kind: Folk::Trader }),
                        'H' => plaques.push((x, y)),
                        'G' => {
                            let &(good, from) = supply_specs.next().expect("a good for every supply");
                            supplies.push(Supply { at: (x, y), good, from, owner: None, timer: SUPPLY_TICKS - 1 });
                        }
                        'N' => {
                            let w = worker_specs.next().expect("a worker for every N");
                            let tasks = w
                                .tasks
                                .iter()
                                .map(|t| match *t {
                                    Task::Move(good, from, to) => Task::Move(good, at(from), at(to)),
                                    Task::Work(spot) => Task::Work(at(spot)),
                                })
                                .collect();
                            workers.push(Worker { name: w.name, title: w.title, about: w.about, says: w.says, skills: w.skills, at: (x, y), tasks });
                        }
                        'Q' => props.push(((x, y), Prop::Wheels)),
                        'x' => props.push(((x, y), Prop::Fence)),
                        'y' => props.push(((x, y), Prop::Cart)),
                        'm' => mouths.push((x, y)),
                        'p' | 'g' | 'o' => parts.push((ch, x, y)),
                        '1' | '2' => _ = starts[(ch as u8 - b'1') as usize].get_or_insert((x, y)),
                        _ => {}
                    }
                    tiles[(y * w + x) as usize] = tile;
                }
            }
        }
        // The first copy's, in a made-up world.
        let starts = starts.map(|s| s.expect("both monster starts on the map")).to_vec();
        for (ch, x, y) in parts {
            let shop = shops.iter_mut().min_by_key(|s| (s.spot.0 - x).abs() + (s.spot.1 - y).abs()).expect("a work spot");
            match ch {
                'p' => {
                    shop.pads.push((x, y));
                    shop.pad_flash.push(0);
                }
                'g' => shop.pegs.push((x, y)),
                _ => shop.outs.push((x, y)),
            }
        }
        for m in mouths {
            if let Some(b) = bins.iter_mut().find(|b| (b.x - m.0).abs() + (b.y - m.1).abs() == 1) {
                b.mouth = Some(m);
            }
        }
        // The footbridge spans the stretch of water from its west end to the far bank.
        let footbridge: Vec<(i32, i32)> = boat.iter().min().map_or_else(Vec::new, |&(west, y)| {
            let mut far = west;
            while tiles[(y * w + far + 1) as usize] == Tile::Water {
                far += 1;
            }
            (west..=far).map(|x| (x, y)).collect()
        });
        let wagons = wagon_at
            .into_iter()
            .zip(pumpkin_at)
            .map(|(at, pumpkin)| Wagon {
                at,
                pumpkin,
                slide: None,
                pumpkin_slide: None,
                stock: Vec::new(),
                room: TOWN_WAGON_ROOM,
                power: TOWN_WAGON_POWER,
                open: false,
                leader: None,
                last_leader: None,
                timer: 0,
                bounce: 0,
                material: String::new(),
            })
            .collect();
        Self { w, h, tiles, items: Vec::new(), plants, bins, shops, particles: Vec::new(), boards, footbridge, bridge, wagons, signs, labels, starts, stands, townsfolk, plaques, workers, supplies, props, bells: Vec::new(), last_item_id: 0, seed: 0x9E37_79B9 }
    }

    /// Where a search for a way about these tiles looks: the box around them, `SEARCH_MARGIN`
    /// further each way, within the map.
    pub fn search_area(&self, about: &[(i32, i32)]) -> Area {
        let (mx, my) = SEARCH_MARGIN;
        let x0 = (about.iter().map(|p| p.0).min().unwrap_or(0) - mx).max(0);
        let y0 = (about.iter().map(|p| p.1).min().unwrap_or(0) - my).max(0);
        let x1 = (about.iter().map(|p| p.0).max().unwrap_or(0) + mx).min(self.w - 1);
        let y1 = (about.iter().map(|p| p.1).max().unwrap_or(0) + my).min(self.h - 1);
        Area { x0, y0, w: (x1 - x0 + 1).max(1), h: (y1 - y0 + 1).max(1) }
    }

    /// Which of an area's tiles have a good lying on them (those `skip` lets pass aside).
    pub fn goods_in(&self, area: Area, skip: impl Fn(&Item) -> bool) -> Vec<bool> {
        let mut taken = vec![false; area.cells()];
        for it in self.items.iter().filter(|it| area.inside((it.x, it.y)) && !skip(it)) {
            taken[area.idx((it.x, it.y))] = true;
        }
        taken
    }

    pub fn tile(&self, x: i32, y: i32) -> Tile {
        if x < 0 || y < 0 || x >= self.w || y >= self.h {
            Tile::Hedge
        } else {
            self.tiles[(y * self.w + x) as usize]
        }
    }

    /// Can a monster stand here?
    pub fn walkable(&self, x: i32, y: i32) -> bool {
        self.tile(x, y).walkable()
    }

    /// Can an item lie here?
    pub fn item_floor(&self, x: i32, y: i32) -> bool {
        self.tile(x, y).item_floor()
    }

    /// Can a wagon (or its pumpkin) roll here? Anywhere a monster walks, but the footbridge.
    pub fn wagon_way(&self, x: i32, y: i32) -> bool {
        self.walkable(x, y) && self.tile(x, y) != Tile::Footbridge
    }

    pub fn wagon_at(&self, x: i32, y: i32) -> Option<usize> {
        self.wagons.iter().position(|w| w.at == (x, y))
    }

    pub fn pumpkin_at(&self, x: i32, y: i32) -> Option<usize> {
        self.wagons.iter().position(|w| w.pumpkin == (x, y))
    }

    /// A wagon or a pumpkin stands here.
    pub fn cart_at(&self, x: i32, y: i32) -> bool {
        self.wagons.iter().any(|w| w.at == (x, y) || w.pumpkin == (x, y))
    }

    /// The bridge's planks replace the river.
    pub fn build_bridge(&mut self) {
        for &(x, y) in &self.bridge {
            self.tiles[(y * self.w + x) as usize] = Tile::Bridge;
        }
    }

    /// Ropes strung across the river: the footbridge.
    pub fn build_footbridge(&mut self) {
        for &(x, y) in &self.footbridge {
            self.tiles[(y * self.w + x) as usize] = Tile::Footbridge;
        }
    }

    pub fn footbridge_built(&self) -> bool {
        self.footbridge.first().is_some_and(|&(x, y)| self.tile(x, y) == Tile::Footbridge)
    }

    pub fn bridge_built(&self) -> bool {
        self.bridge.first().is_some_and(|&(x, y)| self.tile(x, y) == Tile::Bridge)
    }

    pub fn board_at(&self, x: i32, y: i32) -> Option<usize> {
        self.boards.iter().position(|b| b.x == x && b.y == y)
    }

    pub fn sign_at(&self, x: i32, y: i32) -> Option<&Sign> {
        self.signs.iter().find(|s| s.x == x && s.y == y)
    }

    /// Item `k`'s id, giving it one if it has none yet.
    pub fn item_id(&mut self, k: usize) -> u32 {
        if self.items[k].id == 0 {
            self.last_item_id += 1;
            self.items[k].id = self.last_item_id;
        }
        self.items[k].id
    }

    /// The item with this id, if it's still lying about.
    pub fn find_item(&self, id: u32) -> Option<usize> {
        (id != 0).then(|| self.items.iter().position(|it| it.id == id)).flatten()
    }

    pub fn item_at(&self, x: i32, y: i32) -> Option<usize> {
        self.items.iter().position(|it| it.x == x && it.y == y)
    }

    pub fn bin_at(&self, x: i32, y: i32) -> Option<usize> {
        self.bins.iter().position(|b| b.x == x && b.y == y)
    }

    pub fn plant_at(&self, x: i32, y: i32) -> Option<usize> {
        self.plants.iter().position(|p| p.x == x && p.y == y)
    }

    pub fn shop_at_spot(&self, x: i32, y: i32) -> Option<usize> {
        self.shops.iter().position(|s| s.spot == (x, y) && s.frontier.as_ref().is_none_or(|f| !f.fallen))
    }

    pub fn pad_owner(&self, x: i32, y: i32) -> Option<(usize, usize)> {
        self.shops.iter().enumerate().find_map(|(i, s)| s.pads.iter().position(|&p| p == (x, y)).map(|k| (i, k)))
    }

    /// The tool a peg offers: an item hanging on it, if any.
    pub fn peg_item(&self, x: i32, y: i32) -> Option<usize> {
        self.item_at(x, y)
    }

    /// Does this item's good (with its carried tags) satisfy a tag group?
    pub fn item_matches(&self, data: &Data, item: usize, group: &[String]) -> bool {
        matches(&self.items[item].data.tags(data), group)
    }

    pub fn rand(&mut self, n: u32) -> u32 {
        self.seed ^= self.seed << 13;
        self.seed ^= self.seed >> 17;
        self.seed ^= self.seed << 5;
        self.seed % n.max(1)
    }

    /// A small burst of particles from the center of a tile.
    pub fn burst(&mut self, tx: i32, ty: i32, kind: PKind, n: u32) {
        for _ in 0..n {
            let vx = self.rand(5) as i32 - 2;
            let vy = -(self.rand(3) as i32) - if matches!(kind, PKind::Spark) { 1 } else { 0 };
            let (ox, oy) = (self.rand(9) as i32 - 4, self.rand(7) as i32 - 3);
            let life = 6 + self.rand(8) as u8;
            self.particles.push(Particle { x: tx * TS + 8 + ox, y: ty * TS + 9 + oy, vx, vy, life, kind });
        }
    }

    /// Dust kicked up behind something sliding in `dir`.
    pub fn dust(&mut self, tx: i32, ty: i32, dir: Dir) {
        let (dx, dy) = dir.delta();
        for _ in 0..2 {
            let ox = self.rand(7) as i32 - 3;
            let vx = -dx + self.rand(3) as i32 - 1;
            let life = 4 + self.rand(4) as u8;
            let p = Particle { x: tx * TS + 8 - dx * 7 + ox, y: ty * TS + 14 - dy * 4, vx, vy: -(self.rand(2) as i32), life, kind: PKind::Dust };
            self.particles.push(p);
        }
    }

    pub fn tick_particles(&mut self, frame: u32) {
        for p in self.particles.iter_mut() {
            p.x += p.vx;
            p.y += p.vy;
            p.life = p.life.saturating_sub(1);
            match p.kind {
                PKind::Dust | PKind::Leaf if frame % 2 == 0 => p.vy += 1,
                PKind::Fluff if frame % 3 == 0 => p.vx = (p.vx + 1).min(1),
                PKind::Spark if frame % 2 == 0 => p.vx -= p.vx.signum(),
                _ => {}
            }
        }
        self.particles.retain(|p| p.life > 0);
    }
}
