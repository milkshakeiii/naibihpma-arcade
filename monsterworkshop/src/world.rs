//! The world as data: region maps (`data/*.map`) laid side by side, with their workshops,
//! bins, plants, project boards and the footbridge, plus the items lying around. No monsters
//! here; `game.rs` moves them and asks the world what is where.

use crate::data::{matches, Data};
use crate::econ::ItemData;
pub use crate::workshops::ShopKind;
use crate::palette::Style;
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
// Scenery: each kind's own character, in `PROPS` below (`Q` spare wheels, `x` fence, `^` the
// hillside...)
// `C` cotton bin and `m` its chute  `T` cloth store (a drop-off for fabric)  `ȋ` the assay
// office's bin (a drop-off for ingots, any metal)  `a` applicant
// stand in the gathering hall  `1` `2` monster starts
// Plants: `f` cotton  `h` hemlock  `k` oak  `e` elder  `r` rhododendron  `j` arrow bamboo
// `q` ramie  `n` indigofera  `l` limestone outcrop  `M` coal seam  `©` copper seam
// `ĝ` granite outcrop  `ĥ` hemp  `Ƥ` pine  `ƥ` a pine tapped for resin  `ʝ` juniper  `ɨ` iron,
// in a red bank
// `0` a gate into a province past the first, shut until its board's goal is met
// `░` paving  `╪` a jetty (planks over water)  `╬` a bridge, built  `▒` gravel: bare stony ground, as in a quarry or
// a pit (copper seams and granite outcrops lie on it too)  `≡` stone steps
// Work spots are marked by their kind's `map_char` in `data/workshops.json` (`B` workbench,
// `S` spinnery, `L` loom house...). Each `p` pad, `g` tool peg and `o` output mat belongs to the
// nearest work spot.

/// Every character the map legend above gives a meaning, so no workshop kind takes one.
pub const MAP_LEGEND: &str = "#.,W_DYwFsbXUPCmTa12fhkerjqnlpgovcuHGNM0░╪©ĝ▒≡ĥƤƥʝ╬ȋɨ";

/// A region's map, plus what its text-only features say, in the order they appear.
struct Region {
    map: &'static str,
    /// The province it's in: "" for the first, always open; another is closed off until one of
    /// its gates (`0`) opens.
    province: &'static str,
    /// Its look: its province's palette and building materials.
    style: Style,
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
    /// Racks of equipment, each showing one piece given for renown (region coordinates).
    racks: &'static [((i32, i32), &'static str)],
    /// Townsfolk placed by the region's data rather than the map (region coordinates).
    folk: &'static [((i32, i32), Folk)],
    /// Its vines (`ʃ` on the map): each one's foot, where goods are set to go, and the way it
    /// carries them (region coordinates).
    vines: &'static [((i32, i32), Dir)],
    /// Goods lying about when a world begins: where, the good, and what it's plainly made from.
    goods: &'static [((i32, i32), &'static str, &'static str)],
}

/// A working townsperson as a region's data gives them: a monster of the town's on a loop at a
/// bench or plants of their own (see `Game::found_town`).
pub struct WorkerSpec {
    pub name: &'static str,
    /// What they are: "WAGONWRIGHT".
    pub title: &'static str,
    /// The ramp their skin is drawn in instead of their kind's: leather, soot.
    pub skin: (u8, u8),
    /// The tools in their hands, each plainly made from a raw material (good, from). Their own
    /// tools never wear, as a town's workshops never do.
    pub holds: &'static [(&'static str, &'static str)],
    /// Their card's lines, and what they say, one at a time (after what their bench makes).
    pub about: &'static [&'static str],
    pub says: &'static [&'static str],
    pub skills: &'static [(&'static str, f64)],
    /// Their loop, in region coordinates.
    pub tasks: &'static [Task],
    /// Their trade's secret, written on a scroll on a stand by their work and given for renown
    /// like any piece: where it stands (region coordinates), and the piece.
    pub secret: ((i32, i32), &'static str),
}

/// One goal of a working townsperson's loop.
#[derive(Clone, Copy, Debug)]
pub enum Task {
    /// Get this good from where it lies to there.
    Move(&'static str, (i32, i32), (i32, i32)),
    /// Work the bench whose spot this is.
    Work((i32, i32)),
    /// Gather from the plant here, facing this way (where its harvest is set out).
    Gather((i32, i32), Dir),
}

/// A working townsperson in the world: their spec, where they start, and their loop in world
/// coordinates.
#[derive(Clone)]
pub struct Worker {
    pub name: &'static str,
    pub title: &'static str,
    pub skin: (u8, u8),
    pub holds: &'static [(&'static str, &'static str)],
    pub about: &'static [&'static str],
    pub says: &'static [&'static str],
    pub skills: &'static [(&'static str, f64)],
    pub at: (i32, i32),
    pub tasks: Vec<Task>,
    /// Their trade's secret: where its stand is, in world coordinates, and the piece.
    pub secret: ((i32, i32), &'static str),
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

/// Scenery: you walk round it, and examining it says what it is. Its kind's id (`PROPS`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Prop(pub &'static str);

impl Prop {
    pub fn kind(self) -> &'static PropKind {
        PROPS.iter().find(|k| k.id == self.0).unwrap_or_else(|| panic!("no prop kind {}", self.0))
    }
}

/// A vine (DESIGN.md, World highlights; PROVINCES.md, Vines straight): a living conveyor across
/// a cliff, carrying goods straight to where they're used while the way on foot goes round. A
/// good set on the tile at its foot is taken into its cradles and carried along, a tile at a
/// time, and set down on the tile past its head once that's clear.
#[derive(Clone, Debug)]
pub struct Lift {
    /// Where goods are set to go.
    pub foot: (i32, i32),
    /// Its own tiles, from its foot on: what it's carrying rides on them.
    pub vine: Vec<(i32, i32)>,
    /// Where it sets goods down, past its last tile (its head).
    pub top: (i32, i32),
    /// The way it carries: up a cliff, or down one.
    pub dir: Dir,
}

/// Ticks the vine takes to carry a good one tile up: half a second.
pub const LIFT_TICKS: u8 = 8;

/// A kind of scenery: what it's called and says, and how much of the map it takes.
pub struct PropKind {
    pub id: &'static str,
    /// Its map character, on every tile it covers: a bench is `--`.
    pub ch: char,
    pub name: &'static str,
    pub about: &'static str,
    /// A good its card tells how to make, the real thing it stands for (spare wheels: a wheel).
    pub shows: Option<&'static str>,
    /// Tiles across and down.
    pub size: (i32, i32),
    /// Hung on a building's wall, which stays a wall under it.
    pub on_wall: bool,
    /// How far its art reaches below its tiles, in pixels: a chain hanging from a wall into the
    /// room. Art taller than its tiles reaches up instead (a chimney over the roof).
    pub drop: i32,
    /// Afloat or growing in water, which stays water under it (lily pads, reeds, a boat).
    pub on_water: bool,
}

const fn prop(id: &'static str, ch: char, name: &'static str, about: &'static str, size: (i32, i32)) -> PropKind {
    PropKind { id, ch, name, about, shows: None, size, on_wall: false, drop: 0, on_water: false }
}

const fn wall_prop(id: &'static str, ch: char, name: &'static str, about: &'static str, drop: i32) -> PropKind {
    PropKind { id, ch, name, about, shows: None, size: (1, 1), on_wall: true, drop, on_water: false }
}

const fn water_prop(id: &'static str, ch: char, name: &'static str, about: &'static str, size: (i32, i32)) -> PropKind {
    PropKind { id, ch, name, about, shows: None, size, on_wall: false, drop: 0, on_water: true }
}

/// Every kind of scenery. Fences, stone walls, rails, washing and drying lines, gate posts, the
/// awning over a shop's door, the hillside, the cliff and its vine, hemp's drying racks, the
/// parapet and the yard wall join their neighbors of the same kind; the rest is drawn whole on its tiles. The map marks newer kinds with characters past
/// plain ASCII, which ran out (2026-11-04).
pub const PROPS: [PropKind; 99] = [
    PropKind { shows: Some("wagon_wheel"), ..prop("wheels", 'Q', "SPARE WAGON WHEELS", "FINISHED, WAITING FOR A CART.", (1, 1)) },
    PropKind { shows: Some("wagon_kit"), ..prop("cart", 'y', "A CART, HALF BUILT", "IT'S WAITING FOR ITS WHEELS.", (1, 1)) },
    prop("fence", 'x', "A FENCE", "AROUND A YARD. WALK IN BY ITS GATE.", (1, 1)),
    prop("barrel", 'i', "A BARREL", "RAINWATER, KEPT FOR WHATEVER NEEDS SOAKING OR COOLING.", (1, 1)),
    prop("chopping_block", 't', "A CHOPPING BLOCK", "AN AXE LEFT IN IT. GOOD SPOKES ARE SPLIT ALONG THE GRAIN, NOT SAWN.", (1, 1)),
    prop("sawhorse", 'z', "A SAWHORSE", "IT HOLDS A PLANK STEADY WHILE IT'S SAWN.", (1, 1)),
    prop("lumber", 'd', "STACKED PLANKS", "SEASONING IN THE AIR. GREEN WOOD WARPS, SO A WHEEL WANTS IT DRY.", (1, 1)),
    // Temple Town's (2026-11-03): the seam and the trader.
    prop("rock_face", '^', "THE HILLSIDE", "SOLID ROCK. THE COAL SEAM RUNS ALONG ITS FOOT.", (1, 1)),
    prop("mine_mouth", 'A', "AN OLD DRIFT", "A TUNNEL INTO THE HILL, SHORED UP WITH TIMBER. NOBODY DIGS IN THERE NOW.", (2, 2)),
    prop("rails", '=', "RAILS", "FOR THE MINE CART, OUT OF THE OLD DRIFT.", (1, 1)),
    prop("mine_cart", '&', "A MINE CART", "EMPTY. IT CARRIED COAL OUT OF THE DRIFT, WHEN THERE WAS COAL IN THERE.", (1, 1)),
    prop("lantern_post", '!', "A LANTERN POST", "IT LIGHTS THE WAY HOME AFTER DARK.", (1, 1)),
    prop("wheelbarrow", '%', "A WHEELBARROW", "FOR RUBBLE. COAL GETS PUSHED BY HAND, LIKE EVERYTHING HERE.", (1, 1)),
    prop("tool_shed", '$', "THE COLLIER'S SHED", "SPARE HANDLES, A LAMP, AND A KETTLE FOR HIS TEA.", (2, 3)),
    prop("crates", '[', "CRATES", "PACKED FOR THE ROAD, OR JUST UNPACKED.", (1, 1)),
    prop("awning", '~', "THE TRADER'S AWNING", "STRIPED, SO YOU CAN SEE HIS STALL FROM THE ROAD.", (3, 1)),
    prop("flowers", '*', "A FLOWER BED", "SOMEBODY TENDS IT EVERY MORNING.", (1, 1)),
    prop("boulder", '@', "A BOULDER", "IT ROLLED DOWN FROM THE HILLS A LONG TIME AGO.", (1, 1)),
    prop("big_tree", '(', "A GREAT TREE", "OLDER THAN THE TOWN. ITS SHADE IS FOR EVERYONE.", (3, 3)),
    prop("shade_tree", 'E', "A SHADE TREE", "PLANTED FOR ITS SHADE, NOT ITS WOOD.", (2, 2)),
    prop("bench", '-', "A BENCH", "FOR SITTING A WHILE.", (2, 1)),
    // The foundry's.
    prop("firewood", '+', "FIREWOOD", "SPLIT AND STACKED TO DRY, FOR SOMEBODY'S HEARTH.", (1, 1)),
    prop("trough", ':', "A WATER TROUGH", "HOT METAL IS QUENCHED IN IT.", (2, 1)),
    prop("anvil", ';', "AN ANVIL", "ON A STUMP, FOR TRIMMING CASTINGS. A SMITHY WOULD WANT ONE.", (1, 1)),
    prop("hand_cart", '<', "A HAND CART", "FOR WHEELING MOULDS TO THE FURNACE.", (2, 1)),
    wall_prop("chimney", '|', "THE FURNACE CHIMNEY", "IT DRAWS THE FURNACE HOT ENOUGH TO MELT BRONZE.", 0),
    wall_prop("hoist", '{', "A CHAIN HOIST", "FOR LIFTING A BELL OUT OF ITS MOULD.", 8),
    wall_prop("wall_lantern", '}', "A LANTERN", "IT LIGHTS THE WORK.", 0),
    prop("moulds", '?', "BELL MOULDS", "CLAY, BUILT ROUND A CORE AND BAKED. EACH IS BROKEN TO FREE ITS BELL.", (1, 1)),
    // The temple's garden.
    prop("stone_wall", '/', "A STONE WALL", "AROUND THE TEMPLE'S GARDEN. WALK IN BY ITS GATES.", (1, 1)),
    prop("gate_post", '"', "THE TEMPLE GATE", "IT MARKS WHERE THE TOWN ENDS AND THE TEMPLE'S GROUND BEGINS.", (1, 1)),
    prop("stone_lantern", '8', "A STONE LANTERN", "LIT FOR EVERYONE WHO COMES TO HEAR THE BELL.", (1, 1)),
    prop("incense_burner", '9', "AN INCENSE BURNER", "BRONZE, CAST IN THE FOUNDRY ACROSS THE ROAD.", (1, 1)),
    prop("pond", ')', "A LILY POND", "STILL WATER AND LILIES. THE BELL'S RINGING CARRIES ACROSS IT.", (3, 3)),
    prop("cherry_tree", '`', "A CHERRY TREE", "IN THE TEMPLE'S GARDEN IT BLOSSOMS ALL YEAR.", (2, 2)),
    wall_prop("banner", '7', "A BANNER", "FOR THE BELLS THAT HAVE HUNG HERE.", 10),
    prop("cushion", '3', "A CUSHION", "FOR SITTING AND LISTENING.", (1, 1)),
    prop("candle", '4', "A CANDLE STAND", "KEPT LIT.", (1, 1)),
    prop("drum", '5', "A DRUM", "IT ANSWERS THE BELL.", (1, 1)),
    prop("altar", '6', "A SMALL ALTAR", "FLOWERS AND A BOWL OF WATER, CHANGED EACH DAY.", (1, 1)),
    // The houses'.
    prop("well", '>', "THE WELL", "COLD WATER, FOR ANYONE.", (2, 2)),
    prop("washing_line", ']', "A WASHING LINE", "SOMEBODY'S LAUNDRY, DRYING IN THE SUN.", (1, 1)),
    // Tanglemere's (2026-11-04): the dyers' town, the outfitter's, the lake and the indigo field.
    prop("drying_line", 'ð', "A DRYING LINE", "CLOTH FRESH FROM THE VATS, HUNG OUT TO DRY. THE COLOR SETS IN THE SUN.", (1, 1)),
    prop("dye_barrel", 'ø', "A DYE BARREL", "INDIGO, MADDER, OR CLEAN WATER TO RINSE IN. MIND YOUR HANDS.", (1, 1)),
    prop("dye_tub", 'ö', "A DYE TUB", "A BATCH STEEPING. IT STAINS WHATEVER IT TOUCHES.", (1, 1)),
    prop("cloth_shelf", '¶', "SHELVES OF CLOTH", "FOLDED BOLTS, IN EVERY COLOR THE TOWN DYES.", (1, 1)),
    prop("cutting_table", '÷', "A CUTTING TABLE", "SHEARS, AND A BOLT HALF UNROLLED.", (2, 1)),
    wall_prop("hanging_cloth", '¤', "A WOVEN HANGING", "THE WEAVERS' FINEST, HUNG WHERE EVERYONE SEES IT.", 10),
    prop("dummy", 'ì', "A DRESSMAKER'S DUMMY", "A JACKET PINNED TO IT, HALF MADE.", (1, 1)),
    prop("mirror", 'ï', "A TALL MIRROR", "FOR TRYING THINGS ON. IT'S POLISHED EVERY MORNING.", (1, 1)),
    prop("shop_sign", '§', "THE OUTFITTER'S SIGN", "A NEEDLE AND RED THREAD: THINGS TO WEAR, WITHIN.", (1, 1)),
    wall_prop("shop_awning", '¬', "THE OUTFITTER'S AWNING", "STRIPED BLUE AND WHITE, OVER THE DOOR.", 0),
    prop("timber_gate", 'π', "TANGLEMERE'S GATE", "TIMBER POSTS AND A BEAM: THE DYERS' TOWN BEGINS HERE.", (1, 1)),
    prop("stone_pile", '•', "A PILE OF STONES", "LIMESTONE, WAITING TO BE BURNT.", (1, 1)),
    prop("stump", 'º', "A STUMP", "FOR SITTING ON, OR SPLITTING KINDLING.", (1, 1)),
    water_prop("reeds", '¥', "REEDS", "THEY GROW THICK IN THE SHALLOWS.", (1, 1)),
    water_prop("lily_pads", 'ò', "LILY PADS", "A FROG'S STEPPING STONES.", (1, 1)),
    water_prop("rowboat", 'ß', "A ROWING BOAT", "TIED UP AT THE JETTY. NOBODY'S FISHING TODAY.", (2, 1)),
    prop("field_shed", 'ħ', "THE FIELD SHED", "BASKETS FOR THE LEAVES, AND A HOE.", (2, 2)),
    prop("scarecrow", 'ŝ', "A SCARECROW", "IN AN INDIGO SHIRT, OF COURSE.", (1, 1)),
    prop("field_trough", '≈', "A WATER TROUGH", "FOR RINSING THE LEAVES BEFORE THEY GO TO THE VATS.", (2, 1)),
    prop("shrub", 'ŏ', "A BOX SHRUB", "CLIPPED ROUND. NOTHING ON IT TO PICK.", (1, 1)),
    prop("baskets", 'ŭ', "A PAIR OF BASKETS", "FOR THE ENDS OF A SHOULDER POLE. NOTHING IN THEM.", (1, 1)),
    prop("sacks", 'ş', "EMPTY SACKS", "FOLDED, FOR WHOEVER'S CARRYING SOMETHING HOME.", (1, 1)),
    // The Ore Hills' (2026-11-05): the way in.
    prop("cliff", 'Λ', "THE CLIFF", "THE HILLS BEGIN AT THE TOP. ON FOOT IT'S THE LONG WAY ROUND; GOODS GO UP BY THE VINE.", (1, 1)),
    prop("lift_vine", 'ʃ', "A VINE", "IT CARRIES WHATEVER IS SET AT ITS FOOT, STRAIGHT TO ITS HEAD.", (1, 1)),
    prop("pit_wall", 'Ш', "THE PIT WALL", "SHORED UP WITH TIMBER WHERE THE ROCK IS LOOSE.", (1, 1)),
    prop("cairn", 'ɑ', "A CAIRN", "EVERYONE WHO MAKES THE CLIMB ADDS A STONE.", (1, 1)),
    prop("scree", '∴', "SCREE", "LOOSE STONES, FALLEN FROM THE CLIFF.", (1, 1)),
    prop("cut_stone", '▪', "CUT STONE", "GRANITE BLOCKS, SQUARED FOR BUILDING. NOBODY'S IN A HURRY FOR THEM.", (1, 1)),
    prop("lean_to", 'Ł', "THE STONECUTTER'S LEAN-TO", "A ROOF OVER THE BENCH WHERE BLOCKS ARE DRESSED.", (3, 2)),
    prop("sledge", 'ʂ', "A STONE SLEDGE", "BLOCKS ARE DRAGGED ON IT. THE BIG ONES, ANYWAY.", (2, 1)),
    prop("miners_shed", 'Ĥ', "THE MINERS' SHED", "LAMPS, ROPE, AND A KETTLE THAT'S ALWAYS ON.", (2, 2)),
    prop("tool_rack", 'ŧ', "A TOOL RACK", "OLD PATINA'S SPARE PICKS AND SHOVELS. HE COUNTS THEM.", (1, 1)),
    prop("windlass", 'ω', "THE WINDLASS", "IT WINDS ORE UP FROM THE SHAFT, A BUCKET AT A TIME.", (4, 3)),
    prop("spoil_heap", 'ʬ', "A SPOIL HEAP", "BROKEN ROCK WITH NO COPPER IN IT. IT ONLY GROWS.", (3, 2)),
    // Hemp Hollow's.
    prop("hemp_rack", 'ɦ', "A DRYING RACK", "HEMP STALKS HUNG UP TO DRY, BEFORE THEY GO IN THE POND.", (1, 1)),
    prop("stook", 'ʈ', "A STOOK OF HEMP", "STALKS STOOD ON END TO DRY. THEY RATTLE IN THE WIND.", (1, 1)),
    water_prop("retting", 'ɽ', "HEMP SOAKING", "BUNDLES OF STALKS WEIGHTED UNDER THE WATER, ROTTING SOFT ENOUGH TO STRIP.", (1, 1)),
    prop("sack_scarecrow", 'ƨ', "A SCARECROW", "DRESSED IN AN OLD HEMP SACK. THE CROWS AREN'T FOOLED.", (1, 1)),
    // Charcoal Wood's.
    prop("log_stack", 'ɭ', "LOGS FOR THE MOUNDS", "PINE, STACKED TO SEASON BEFORE IT'S BURIED AND BURNT.", (1, 1)),
    prop("turf_heap", 'ʉ', "TURF AND EARTH", "CUT TO COVER THE NEXT MOUND, SO IT SMOULDERS INSTEAD OF BURNING.", (1, 1)),
    prop("shelter", 'ʊ', "THE BURNERS' SHELTER", "SOMEBODY HAS TO WATCH A MOUND ALL NIGHT. THIS IS WHERE.", (1, 1)),
    // Verdigris's.
    prop("cottage", 'ȼ', "A COTTAGE", "ITS ROOF IS COPPER, GONE GREEN IN THE WEATHER, LIKE EVERY ROOF HERE.", (3, 3)),
    prop("grindstone", 'ǥ', "A GRINDSTONE", "FOR PUTTING AN EDGE BACK ON. TURN THE HANDLE, MIND YOUR FINGERS.", (1, 1)),
    prop("parapet", 'ɐ', "THE PARAPET", "A LOW WALL ALONG THE CLIFF'S EDGE. THE PINE WOOD IS A LONG WAY DOWN.", (1, 1)),
    prop("notice_board", 'ņ', "THE NOTICE BOARD", "LOST: ONE PICK. FOUND: ONE MITTEN. WANTED: SOMEBODY WHO CAN SING.", (2, 1)),
    // Bellows's.
    prop("yard_wall", 'ǁ', "THE YARD WALL", "ROUND THE SMELTERS' YARD. IT KEEPS THE SPARKS IN, MOSTLY.", (1, 1)),
    prop("slag", 'ʚ', "SLAG", "WHAT'S LEFT WHEN THE METAL HAS RUN OUT OF THE ORE: GLASSY, SHARP, AND GOOD FOR NOTHING.", (1, 1)),
    prop("tong_rack", 'ʇ', "A RACK OF TONGS", "TONGS AND RAKES FOR THE FURNACES, EACH ON ITS OWN PEG.", (1, 1)),
    prop("scales", 'ʡ', "THE ASSAY SCALES", "EVERY INGOT IS WEIGHED HERE, AND ITS METAL JUDGED.", (1, 1)),
    prop("gear_shelf", 'ɧ', "SHELVES OF GEAR", "HATS, BOOTS AND COILS OF ROPE. THE BELT ON THE STAND IS THE ONE WORTH HAVING.", (1, 1)),
    // Beside a workshop whose recipes can take the same goods (Henry, 2026-11-06).
    prop("choice_board", 'ȵ', "THE CHOICE BOARD", "WHEN THE GOODS ON THE PADS BESIDE IT COULD MAKE SEVERAL THINGS, IT SAYS WHICH. SPACE CHANGES IT.", (1, 1)),
    // Iron Brook's.
    prop("sawyers_shed", 'Ɖ', "THE SAWYERS' SHED", "SEASONED LOGS UNDER COVER, OUT OF THE RAIN.", (3, 2)),
    prop("log_pile", 'ɮ', "LOGS FOR THE SAW", "PINE, BARKED AND WAITING ITS TURN OVER THE PIT.", (1, 1)),
    prop("sawdust", 'ʓ', "SAWDUST", "IT GETS EVERYWHERE. IT SMELLS OF PINE.", (1, 1)),
    prop("ferns", 'ɸ', "FERNS", "THEY LIKE IT DAMP BY THE WATER.", (1, 1)),
    water_prop("river_stones", 'ɵ', "STONES IN THE BROOK", "THE WATER CHATTERS OVER THEM.", (1, 1)),
    water_prop("fallen_log", 'ɤ', "A FALLEN LOG", "IT LIES ACROSS THE WATER. NOBODY TRUSTS IT TO WALK ON.", (1, 1)),
];


/// Each region's entities are listed after the previous region's, so adding a region never
/// reorders what an older save refers to: the road west to east, then Temple Town south of it.
const REGIONS: [Region; 12] = [
    Region {
        map: include_str!("../data/pinewater.map"),
        province: "",
        style: Style::Cloth,
        at: (0, 0),
        racks: &[],
        folk: &[],
        vines: &[],
        goods: &[],
        labels: &[((5, 19), "GATHERING HALL"), ((25, 27), "OLD SAWGRASS"), ((16, 18), "OLD SPOKE'S YARD")],
        signs: &[("TANGLEMERE", Dir::Right)],
        boards: &[("OLD SPOKE'S ORDER", "wagon_wheel", 4, Project::Order)],
        supplies: &[("oak_log", "oak_log"), ("planks", "hemlock_log"), ("elder_sticks", "elder_sticks")],
        workers: &[OLD_SPOKE],
    },
    Region {
        map: include_str!("../data/wilderness.map"),
        province: "",
        style: Style::Cloth,
        at: (50, 0),
        racks: &[],
        folk: &[],
        vines: &[],
        goods: &[],
        supplies: &[],
        workers: &[],
        labels: &[],
        signs: &[("TANGLEMERE", Dir::Right), ("TEMPLE TOWN", Dir::Down)],
        boards: &[("A ROPE FOR THE FOOTBRIDGE", "rope", 3, Project::Footbridge), ("A BRIDGE ACROSS THE RIVER", "log", 12, Project::Bridge)],
    },
    Region {
        map: include_str!("../data/tanglemere.map"),
        province: "",
        style: Style::Cloth,
        at: (125, 0),
        // The outfitter's: the plain pieces along the back wall, given for renown, and Old Thimble
        // between them; by the door, things to work with; outside, along the path, things for
        // hauling.
        racks: &[
            ((40, 14), "rope_belt"),
            ((41, 14), "cloth_gloves"),
            ((42, 14), "straw_sandals"),
            ((44, 14), "headband"),
            ((45, 14), "temple_charm"),
            ((46, 14), "bead_earrings"),
            ((40, 11), "towel"),
            ((40, 12), "whetstone"),
            ((42, 11), "hasty_apron"),
            // Outside its west wall, along the path south: things for hauling.
            ((37, 11), "backpack"),
            ((37, 12), "feathered_cap"),
            ((37, 13), "shoulder_pole"),
            ((37, 14), "tool_belt"),
        ],
        folk: &[((43, 13), Folk::Outfitter)],
        vines: &[],
        goods: &[],
        labels: &[((39, 2), "CLOTH HALL"), ((46, 10), "OUTFITTER")],
        signs: &[("PINEWATER", Dir::Left)],
        boards: &[],
        supplies: &[],
        workers: &[],
    },
    Region {
        map: include_str!("../data/temple_town.map"),
        province: "",
        style: Style::Cloth,
        at: (75, 32),
        // The temple's teachings, a scroll for each talent along its top wall; patience and the
        // lucky charm by the drum and the altar candle, and Old Quiet who keeps them; the work
        // song beside the bell frame.
        racks: &[
            ((7, 23), "teaching_mathematics"),
            ((8, 23), "teaching_science"),
            ((9, 23), "teaching_engineering"),
            ((10, 23), "teaching_writing"),
            ((11, 23), "teaching_visual_art"),
            ((13, 23), "teaching_music"),
            ((14, 23), "teaching_handcrafts"),
            ((15, 23), "teaching_athletics"),
            ((16, 23), "teaching_outdoorsmonstership"),
            ((17, 23), "teaching_social"),
            ((8, 26), "lucky_charm"),
            ((16, 26), "patience"),
            ((11, 26), "work_song"),
        ],
        folk: &[((15, 26), Folk::Keeper)],
        vines: &[],
        goods: &[],
        labels: &[((12, 22), "TEMPLE")],
        signs: &[],
        boards: &[("OLD SOOT'S ORDER", "coal", 6, Project::Order), ("THE TEMPLE BELL", "temple_bell", 1, Project::Bell)],
        supplies: &[],
        workers: &[OLD_SOOT],
    },
    // The way east out of Temple Town into the hills: barren country, a gate across it, and the
    // hills' first screen beyond, still to be built.
    Region {
        map: include_str!("../data/high_pass.map"),
        province: "THE HILLS",
        style: Style::Cloth,
        at: (125, 32),
        racks: &[],
        folk: &[],
        vines: &[],
        goods: &[],
        labels: &[],
        signs: &[("THE HILLS", Dir::Right)],
        boards: &[("THE PASS TO THE HILLS", "fabric", 20, Project::Gate)],
        supplies: &[],
        workers: &[],
    },
    // The Ore Hills, screen by screen (`PROVINCES.md`, the second draft): vines carry goods
    // straight to where they're used, and the road goes the long way round. The way in: the pass
    // arrives below a cliff, with a granite quarry at its foot, and the lift vine carries goods up
    // it to the high road into Verdigris; on foot, the road goes south, the long way.
    Region {
        map: include_str!("../data/cliff_lift.map"),
        province: "THE HILLS",
        style: Style::Hills,
        at: (150, 32),
        racks: &[],
        folk: &[],
        vines: &[((21, 10), Dir::Up)],
        // Until there's a smithy (Henry, 2026-11-05: for testing, lay some on the ground).
        goods: &[((20, 13), "stone_hammer", "granite"), ((21, 13), "metal_chisel", "pig_iron_ingot"), ((22, 13), "stone_hammer", "granite"), ((23, 13), "metal_chisel", "pig_iron_ingot")],
        labels: &[],
        signs: &[("TEMPLE TOWN", Dir::Left), ("VERDIGRIS", Dir::Down)],
        boards: &[],
        supplies: &[],
        workers: &[],
    },
    // The most copper: an open cut with two terraces of seams, a windlass over a shaft, and the
    // copper miner at his own seams by the road. Its ore vine goes over its south edge, down into
    // the smelters' yard in Bellows.
    Region {
        map: include_str!("../data/copper_cut.map"),
        province: "THE HILLS",
        style: Style::Hills,
        at: (200, 32),
        racks: &[],
        folk: &[],
        vines: &[((16, 13), Dir::Down)],
        goods: &[],
        labels: &[],
        signs: &[],
        boards: &[("OLD PATINA'S ORDER", "copper_ore", 6, Project::Order)],
        supplies: &[],
        workers: &[OLD_PATINA],
    },
    // Verdigris, the hills' town, at the top of the Stone Stair: the smithy at its west end, by
    // where the lift vine sets goods down; the smith at his own forge across the road; the
    // gathering hall and the square. A village, hedged round but where the roads and the stair
    // come in.
    Region {
        map: include_str!("../data/verdigris.map"),
        province: "THE HILLS",
        style: Style::Hills,
        at: (175, 32),
        racks: &[],
        folk: &[],
        vines: &[],
        goods: &[],
        labels: &[((21, 6), "GATHERING HALL")],
        signs: &[("COPPER CUT", Dir::Right), ("TEMPLE TOWN", Dir::Down)],
        boards: &[("OLD TEMPER'S ORDER", "knife", 6, Project::Order)],
        supplies: &[("copper_ingot", "copper_ore"), ("charcoal", "pine_log")],
        workers: &[OLD_TEMPER],
    },
    // The road the long way round: south from the Cliff Lift into Hemp Hollow, where hemp grows
    // by the Rust Brook and the hemp house stands on its retting pond.
    Region {
        map: include_str!("../data/hemp_hollow.map"),
        province: "THE HILLS",
        style: Style::Hills,
        at: (150, 48),
        racks: &[],
        folk: &[],
        vines: &[],
        // Until there's a smithy, a knife for bast fiber and a needle for sacks.
        goods: &[((16, 8), "knife", "pig_iron_ingot"), ((21, 8), "needle", "pig_iron_ingot")],
        labels: &[],
        signs: &[("TEMPLE TOWN", Dir::Up), ("VERDIGRIS", Dir::Right), ("CHARCOAL WOOD", Dir::Right)],
        boards: &[],
        supplies: &[],
        workers: &[],
    },
    // Charcoal Wood, a pine ridge: the charcoal mounds and the stonemason's yard, the Stone Stair
    // up its north cliff to Verdigris, and the charcoal vine over its east edge to Bellows.
    Region {
        map: include_str!("../data/charcoal_wood.map"),
        province: "THE HILLS",
        style: Style::Hills,
        at: (175, 48),
        racks: &[],
        folk: &[],
        vines: &[((21, 6), Dir::Right)],
        goods: &[],
        labels: &[],
        signs: &[("VERDIGRIS", Dir::Up)],
        boards: &[],
        supplies: &[],
        workers: &[],
    },
    // Bellows, the smelters' town, below the Copper Cut's cliff and Charcoal Wood's ridge: the ore
    // vine and the charcoal vine set their goods down in its smelters' yard. The assay office takes
    // ingots; the miners' outfitter gives the bronze-buckled belt. A town, hedged round but where
    // the road, the brook and the vines come in.
    Region {
        map: include_str!("../data/bellows.map"),
        province: "THE HILLS",
        style: Style::Hills,
        at: (200, 48),
        racks: &[((19, 8), "buckled_belt")],
        folk: &[],
        vines: &[],
        goods: &[],
        labels: &[((20, 2), "ASSAY OFFICE"), ((20, 7), "MINERS' OUTFITTER")],
        signs: &[("VERDIGRIS", Dir::Left)],
        boards: &[],
        supplies: &[],
        workers: &[],
    },
    // Iron Brook, south of Hemp Hollow: the Rust Brook comes down from the hollow and turns west
    // out of the hills, its banks red with iron; pines, and a sawpit at the road's end.
    Region {
        map: include_str!("../data/iron_brook.map"),
        province: "THE HILLS",
        style: Style::Hills,
        at: (150, 64),
        racks: &[],
        folk: &[],
        vines: &[],
        goods: &[],
        labels: &[],
        signs: &[("HEMP HOLLOW", Dir::Up)],
        boards: &[],
        supplies: &[],
        workers: &[],
    },
];

/// Pinewater's wagonwright, after Xi Zhong, who legend says made the first cart. He fetches his
/// wood to his bench, makes a wheel and takes it to his order board.
const OLD_SPOKE: WorkerSpec = WorkerSpec {
    name: "OLD SPOKE",
    title: "WAGONWRIGHT",
    skin: (crate::palette::MAT, 3),
    holds: &[("stone_hammer", "limestone")],
    about: &["HE MAKES WAGON WHEELS AT HIS OWN BENCH, FROM HIS OWN WOOD. WATCH HOW HE DOES IT.", "THE BENCH BY THE GATHERING HALL IS FOR ANYONE. HIS ORDER BOARD TAKES ANYONE'S WHEELS, FOR RENOWN.", "SPACE: ASK HIM SOMETHING."],
    says: &["A WHEEL WANTS HARDWOOD SPOKES: OAK, AROUND HERE. SOFT HEMLOCK WON'T DO.", "TWO WHEELS, PLANKS, A HARDWOOD LOG AND A PUMPKIN TO PULL IT: THAT'S A WAGON. THE BENCH BY THE HALL MAKES THOSE TOO.", "MY ORDERS TAKE ANYONE'S WHEELS. THERE'S ALWAYS A CART WANTING ONE."],
    skills: &[("wagon_wright", 0.8), ("carpentry", 0.6), ("hauling", 0.4)],
    tasks: &[
        Task::Move("oak_log", (21, 21), (19, 24)),
        Task::Move("planks", (17, 22), (18, 24)),
        Task::Move("planks", (17, 22), (20, 24)),
        Task::Move("elder_sticks", (19, 22), (18, 25)),
        Task::Move("elder_sticks", (19, 22), (18, 26)),
        Task::Work((19, 25)),
        Task::Move("wagon_wheel", (20, 25), (22, 25)),
    ],
    secret: ((21, 27), "secret_wagon_wright"),
};

/// Temple Town's collier: he digs coal at his own end of the seam, a pick and a shovel in hand,
/// and takes it to his order board for the foundry.
const OLD_SOOT: WorkerSpec = WorkerSpec {
    name: "OLD SOOT",
    title: "COLLIER",
    skin: (crate::palette::HUD_FILL, 3),
    holds: &[("pick", "pig_iron_ingot"), ("shovel", "copper_ingot")],
    about: &["HE DIGS COAL AT HIS OWN END OF THE SEAM, A PICK AND A SHOVEL IN HAND. WATCH HOW HE DOES IT.", "THE REST OF THE SEAM IS FOR ANYONE WITH THE TOOLS. HIS ORDER BOARD TAKES ANYONE'S COAL, FOR RENOWN.", "SPACE: ASK HIM SOMETHING."],
    says: &["THE TRADER HAS PICKS AND SHOVELS. TAKE ONE IN EACH HAND, THEN STAND ON THE SEAM AND DIG.", "TOOLS WEAR DOWN AS YOU DIG. TAKE YOURS FROM A PILE OF SPARES WHILE YOU RECORD, AND THE LOOP FETCHES A NEW ONE THERE WHEN IT BREAKS.", "COAL BURNS HOTTER THAN WOOD. THE KILN BAKES IT INTO COKE, FOR CASTING."],
    skills: &[("prospecting", 0.8), ("gathering", 0.6), ("hauling", 0.4)],
    tasks: &[
        Task::Gather((3, 2), Dir::Down),
        Task::Move("coal", (3, 3), (3, 5)),
        Task::Move("coal", (3, 3), (3, 5)),
        Task::Gather((4, 2), Dir::Down),
        Task::Move("coal", (4, 3), (3, 5)),
        Task::Move("coal", (4, 3), (3, 5)),
    ],
    secret: ((6, 5), "secret_prospecting"),
};

/// The Copper Cut's miner: he digs copper at his own end of the cut, as Old Soot digs coal,
/// and takes it to his order board.
const OLD_PATINA: WorkerSpec = WorkerSpec {
    name: "OLD PATINA",
    title: "COPPER MINER",
    skin: (crate::palette::THREAD, 3),
    holds: &[("pick", "bronze_ingot"), ("shovel", "copper_ingot")],
    about: &["HE DIGS COPPER AT HIS OWN END OF THE CUT, A PICK AND A SHOVEL IN HAND. WATCH HOW HE DOES IT.", "THE REST OF THE CUT IS FOR ANYONE WITH THE TOOLS. HIS ORDER BOARD TAKES ANYONE'S COPPER ORE, FOR RENOWN.", "SPACE: ASK HIM SOMETHING."],
    says: &["MORE COPPER COMES OUT OF THESE HILLS THAN OUT OF ANYWHERE ELSE. EVERYBODY KNOWS IT.", "THE GREEN IN THE ROCK IS HOW YOU KNOW IT'S COPPER. DIG WHERE IT'S GREENEST.", "A PICK IN ONE HAND, A SHOVEL IN THE OTHER. THEY WEAR DOWN, SO KEEP SPARES BY THE CUT."],
    skills: &[("prospecting", 0.8), ("gathering", 0.6), ("hauling", 0.4)],
    tasks: &[
        Task::Gather((3, 2), Dir::Down),
        Task::Move("copper_ore", (3, 3), (2, 3)),
        Task::Move("copper_ore", (3, 3), (2, 3)),
        Task::Gather((4, 2), Dir::Down),
        Task::Move("copper_ore", (4, 3), (2, 3)),
        Task::Move("copper_ore", (4, 3), (2, 3)),
    ],
    secret: ((1, 3), "secret_prospecting"),
};

/// Verdigris's smith: he forges knives at his own forge from his own copper and charcoal, and
/// takes them to his order board.
const OLD_TEMPER: WorkerSpec = WorkerSpec {
    name: "OLD TEMPER",
    title: "SMITH",
    skin: (crate::palette::WOOD, 3),
    holds: &[("metal_hammer", "steel_ingot"), ("forceps", "pig_iron_ingot")],
    about: &["HE FORGES KNIVES AT HIS OWN FORGE, FROM HIS OWN COPPER AND CHARCOAL. WATCH HOW HE DOES IT.", "THE SMITHY ACROSS THE ROAD IS FOR ANYONE. HIS ORDER BOARD TAKES ANYONE'S KNIVES, FOR RENOWN.", "SPACE: ASK HIM SOMETHING."],
    says: &["STEEL HOLDS AN EDGE TWICE AS LONG AS IRON. A COPPER KNIFE IS A KNIFE, BUT NOT FOR LONG.", "HEAT, THEN HAMMER: CHARCOAL OR COAL ON ONE PAD, THE METAL ON ANOTHER. THE FORGE LENDS THE TONGS AND THE ANVIL.", "CHISELS, FILES AND PICKS WANT HARD METAL: STEEL, PIG IRON OR BRONZE. COPPER WON'T DO."],
    skills: &[("blacksmithing", 0.8), ("casting", 0.5), ("hauling", 0.4)],
    tasks: &[
        Task::Move("copper_ingot", (2, 2), (4, 2)),
        Task::Move("charcoal", (8, 2), (6, 2)),
        Task::Work((5, 2)),
        Task::Move("knife", (5, 3), (8, 3)),
    ],
    secret: ((1, 1), "secret_blacksmithing"),
};

/// How long a working townsperson's supply takes to set out a fresh good once theirs is taken:
/// four seconds, so a second plank from the same stack is a short wait.
pub const SUPPLY_TICKS: u32 = 60;

/// Named stretches of the world by their northwest corner, announced on arrival: the road west
/// to east, then Temple Town south of it, then the hills row by row, each west to east (the last
/// one a tile lies past is its name, so the order matters).
const AREAS: [(i32, i32, &str); 14] = [(0, 0, "PINEWATER"), (50, 0, "TINLESS"), (75, 0, "THATCH HILLS"), (100, 0, "DYE MEADOWS"), (125, 0, "TANGLEMERE"), (75, 32, "TEMPLE TOWN"), (125, 32, "THE HIGH PASS"), (150, 32, "THE CLIFF LIFT"), (175, 32, "VERDIGRIS"), (200, 32, "COPPER CUT"), (150, 48, "HEMP HOLLOW"), (175, 48, "CHARCOAL WOOD"), (200, 48, "BELLOWS"), (150, 64, "IRON BROOK")];

/// The world's height, the rows of regions above together: a made-up world's copies repeat it.
/// A test checks it against the regions, since a region further south makes the world taller.
pub(crate) const WORLD_ROWS: i32 = 80;

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
        'M' => "coal",
        '©' => "copper_ore",
        'ĝ' => "granite",
        'ĥ' => "hemp_stalks",
        'Ƥ' => "pine_log",
        'ƥ' => "pine_resin",
        'ʝ' => "juniper_sticks",
        'ɨ' => "iron_ore",
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
    /// A gate into a closed province, shut: nobody, no good and no wagon gets past. Opened, it's
    /// road.
    Gate,
    /// A rack showing a piece of equipment, given for renown to whoever faces it (Space).
    Rack,
    /// Cobbles: a town square, walked and driven on like a road.
    Paving,
    /// Bare stony ground: a quarry's floor, a mine's pit. Walked like grass.
    Gravel,
    /// Stone steps, between a pit's terraces: walked, and goods pushed up and down them, but no
    /// wagon climbs them.
    Stair,
}

impl Tile {
    pub fn walkable(self) -> bool {
        !matches!(self, Tile::Hedge | Tile::Wall | Tile::Bin | Tile::Water | Tile::Tree | Tile::Sign | Tile::Gate | Tile::Rack)
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
    /// The working townsperson whose it is (a monster's id): only they gather it, and what it
    /// gives is theirs.
    #[serde(default)]
    pub owner: Option<u32>,
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

/// The outfitter a region's data places: Old Thimble, who keeps the town's things to wear.
pub const OUTFITTER: &str = "OLD THIMBLE";
/// The temple keeper's name, for now.
pub const KEEPER: &str = "OLD QUIET";

/// A rack of equipment: the piece it shows, given for renown to whoever faces it.
#[derive(Clone, Debug)]
pub struct Rack {
    pub x: i32,
    pub y: i32,
    pub piece: &'static str,
}

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
    /// Keeps the racks of things to wear beside them.
    Outfitter,
    /// Keeps the temple's teachings, and its lucky coins.
    Keeper,
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
    /// A province's gate: it opens for good once enough of what the board wants came in within
    /// the last hour (`GATE_WINDOW_TICKS`).
    Gate,
}

/// The stretch a gate's board counts over: an hour of play.
pub const GATE_WINDOW_TICKS: u32 = 15 * 60 * 60;

/// A gate into a province past the first: its tiles, the board whose goal opens it, and the
/// province it lets you into.
#[derive(Clone, Debug)]
pub struct Gate {
    pub tiles: Vec<(i32, i32)>,
    pub board: usize,
    pub province: &'static str,
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
    /// A gate's board: the frame each of its goods came in, over the last hour. What counts is
    /// how many came in that recently, a rate, not a total.
    #[serde(default)]
    pub log: Vec<u32>,
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
    /// What its choice board says, when the goods on its pads could make several things.
    #[serde(default)]
    pub chosen: Option<String>,
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

    /// What it rests on with nothing of use on its pads: what its choice board says, else what
    /// they last worked toward, else what it's set up for at first.
    pub fn resting<'a>(&'a self, data: &Data) -> &'a str {
        self.chosen.as_deref().or(self.aimed.as_deref()).unwrap_or_else(|| self.default_recipe(data))
    }
}

#[derive(Clone, Copy, Debug)]
pub enum PKind {
    Dust,
    Fluff,
    Spark,
    Leaf,
    /// A note rising from a work song's singer.
    Note,
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
    /// Racks of equipment.
    pub racks: Vec<Rack>,
    /// Gates into the provinces past the first.
    pub gates: Vec<Gate>,
    /// Cliff-lift vines.
    pub lifts: Vec<Lift>,
    /// Choice boards: where each stands, and the workshop it chooses for (the nearest).
    pub choices: Vec<((i32, i32), usize)>,
    /// Each region's box (x, y, width, height) and the province it's in.
    pub provinces: Vec<((i32, i32, i32, i32), &'static str)>,
    /// Each region's box and its look.
    pub looks: Vec<((i32, i32, i32, i32), Style)>,
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
        // Maps are read by character, not byte: any character can mark a kind of thing.
        let placed: Vec<(i32, i32, &Region, Vec<Vec<char>>)> = layout.iter().map(|&((ox, oy), r)| (ox, oy, r, r.map.lines().filter(|l| !l.is_empty()).map(|l| l.chars().collect()).collect())).collect();
        let w = placed.iter().map(|(ox, _, _, rows)| ox + rows[0].len() as i32).max().unwrap_or(0);
        let h = placed.iter().map(|(_, oy, _, rows)| oy + rows.len() as i32).max().unwrap_or(0);
        let mut tiles = vec![Tile::Hedge; (w * h) as usize];
        let (mut plants, mut bins, mut shops, mut starts) = (Vec::new(), Vec::new(), Vec::new(), [None; 2]);
        let (mut stands, mut boards, mut signs, mut labels, mut boat) = (Vec::new(), Vec::new(), Vec::new(), Vec::new(), Vec::new());
        let (mut townsfolk, mut plaques) = (Vec::new(), Vec::new());
        let (mut workers, mut supplies, mut props) = (Vec::new(), Vec::new(), Vec::new());
        let (mut bridge, mut wagon_at, mut pumpkin_at) = (Vec::new(), Vec::new(), Vec::new());
        let (mut mouths, mut parts) = (Vec::new(), Vec::new());
        let (mut gates, mut provinces, mut racks, mut looks) = (Vec::new(), Vec::new(), Vec::new(), Vec::new());
        let (mut vine_feet, mut items) = (Vec::new(), Vec::new());
        for (ox, oy, region, rows) in placed {
            let rw = rows[0].len() as i32;
            provinces.push(((ox, oy, rw, rows.len() as i32), region.province));
            looks.push(((ox, oy, rw, rows.len() as i32), region.style));
            let (first_board, mut gate_tiles) = (boards.len(), Vec::new());
            let (mut sign_texts, mut board_specs) = (region.signs.iter(), region.boards.iter());
            let (mut supply_specs, mut worker_specs) = (region.supplies.iter(), region.workers.iter());
            let at = |(x, y): (i32, i32)| (x + ox, y + oy);
            labels.extend(region.labels.iter().map(|&((x, y), text)| ((x + ox, y + oy), text)));
            for (y, row) in rows.iter().enumerate() {
                assert_eq!(row.len() as i32, rw, "map row {y} is {} wide", row.len());
                for (lx, &ch) in row.iter().enumerate() {
                    let (x, y) = (lx as i32 + ox, y as i32 + oy);
                    let street_next = row.get(lx + 1) == Some(&',') || (lx > 0 && row[lx - 1] == ',');
                    let shop_kind = ShopKind::by_map_char(ch);
                    let prop_kind = PROPS.iter().find(|k| k.ch == ch);
                    let tile = match ch {
                        _ if shop_kind.is_some() => Tile::Spot,
                        _ if prop_kind.is_some_and(|k| k.on_wall) => Tile::Wall,
                        _ if prop_kind.is_some_and(|k| k.on_water) => Tile::Water,
                        _ if prop_kind.is_some() => Tile::Sign,
                        '#' => Tile::Hedge,
                        ',' | 'm' => Tile::Path,
                        'W' | 'H' => Tile::Wall,
                        '_' | 'a' | 'c' => Tile::Floor,
                        'D' => Tile::Door,
                        'p' => Tile::Pad,
                        'g' => Tile::Peg,
                        'o' => Tile::OutMat,
                        'C' | 'T' | 'b' | 'ȋ' => Tile::Bin,
                        'w' | 'F' | 'X' => Tile::Water,
                        'Y' => Tile::Tree,
                        '0' => Tile::Gate,
                        '░' => Tile::Paving,
                    '▒' | '©' | 'ĝ' | 'ɨ' => Tile::Gravel,
                    '≡' => Tile::Stair,
                    '╬' => Tile::Bridge,
                        // A jetty: planks over the water, walked like the footbridge.
                        '╪' => Tile::Footbridge,
                        's' => Tile::Sign,
                        '1' | '2' if street_next => Tile::Path,
                        _ => Tile::Grass,
                    };
                    if let Some(good) = plant_good(ch) {
                        let regrow = data.good(good).production_time;
                        plants.push(Plant { x, y, good: good.into(), growth: regrow, regrow, stock: Vec::new(), harvest: None, drop: None, timer: 0, owner: None });
                    }
                    // A prop several tiles big is marked on each: it starts where its run of marks
                    // (across, and down) starts, or a whole one's width or height further on.
                    if let Some(k) = prop_kind {
                        let run = |dx: i32, dy: i32| {
                            let mut n = 0;
                            while rows.get((y - oy + dy * (n + 1)) as usize).and_then(|r| r.get((lx as i32 + dx * (n + 1)) as usize)).copied() == Some(ch) && lx as i32 + dx * (n + 1) >= 0 && y - oy + dy * (n + 1) >= 0 {
                                n += 1;
                            }
                            n
                        };
                        if run(-1, 0) % k.size.0 == 0 && run(0, -1) % k.size.1 == 0 {
                            props.push(((x, y), Prop(k.id)));
                        }
                    }
                    if let Some(kind) = shop_kind {
                        shops.push(Shop { kind, spot: (x, y), pads: Vec::new(), pegs: Vec::new(), outs: Vec::new(), quality: TOWN_WORKSHOP_QUALITY, job: None, pending: Vec::new(), pad_flash: Vec::new(), out_timer: 0, frontier: None, aimed: None, chosen: None, owner: None });
                    }
                    match ch {
                        'C' => bins.push(Bin { x, y, accepts: vec!["cotton_bolls".into()], stock: Vec::new(), mouth: None, timer: 0, bounce: 0, scores: false, frontier: None }),
                        'T' => bins.push(Bin { x, y, accepts: vec!["fabric".into()], stock: Vec::new(), mouth: None, timer: 0, bounce: 0, scores: true, frontier: None }),
                        'ȋ' => bins.push(Bin { x, y, accepts: vec!["ingot".into()], stock: Vec::new(), mouth: None, timer: 0, bounce: 0, scores: true, frontier: None }),
                        'b' => {
                            let &(title, wants, need, project) = board_specs.next().expect("a project for every board");
                            boards.push(Board { x, y, title: title.into(), wants: wants.into(), need, have: 0, project, bounce: 0, rest: 0, log: Vec::new() });
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
                                    Task::Gather(spot, facing) => Task::Gather(at(spot), facing),
                                })
                                .collect();
                            workers.push(Worker { name: w.name, title: w.title, skin: w.skin, holds: w.holds, about: w.about, says: w.says, skills: w.skills, at: (x, y), tasks, secret: (at(w.secret.0), w.secret.1) });
                        }
                        'm' => mouths.push((x, y)),
                        '0' => gate_tiles.push((x, y)),
                        'p' | 'g' | 'o' => parts.push((ch, x, y)),
                        '1' | '2' => _ = starts[(ch as u8 - b'1') as usize].get_or_insert((x, y)),
                        _ => {}
                    }
                    tiles[(y * w + x) as usize] = tile;
                }
            }
            for &((x, y), piece) in region.racks.iter().chain(region.workers.iter().map(|w| &w.secret)) {
                let (x, y) = at((x, y));
                assert!(data.try_piece(piece).is_some(), "a rack shows {piece}, which isn't a piece of equipment");
                tiles[(y * w + x) as usize] = Tile::Rack;
                racks.push(Rack { x, y, piece });
            }
            vine_feet.extend(region.vines.iter().map(|&(foot, dir)| (at(foot), dir)));
            items.extend(region.goods.iter().map(|&(spot, good, from)| {
                let (x, y) = at(spot);
                Item::new(x, y, crate::econ::plain(data, good, from))
            }));
            for &(spot, kind) in region.folk {
                let (x, y) = at(spot);
                let name = match kind {
                    Folk::Craftsman => CRAFTSMAN,
                    Folk::Trader => TRADER,
                    Folk::Outfitter => OUTFITTER,
                    Folk::Keeper => KEEPER,
                };
                townsfolk.push(Townsfolk { x, y, name: name.into(), kind });
            }
            if !gate_tiles.is_empty() {
                let board = (first_board..boards.len()).find(|&b| boards[b].project == Project::Gate).expect("a board for the gate");
                gates.push(Gate { tiles: gate_tiles, board, province: region.province });
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
        // Each choice board chooses for the workshop nearest it.
        let choices: Vec<((i32, i32), usize)> = props
            .iter()
            .filter(|(_, p)| p.0 == "choice_board")
            .filter_map(|&((x, y), _)| (0..shops.len()).min_by_key(|&s| (shops[s].spot.0 - x).abs() + (shops[s].spot.1 - y).abs()).map(|s| ((x, y), s)))
            .collect();
        // Each vine runs from its foot the way it carries, over its own tiles, to the tile past
        // them.
        let vine = |(x, y): (i32, i32)| props.iter().any(|&((px, py), p): &((i32, i32), Prop)| (px, py) == (x, y) && p.0 == "lift_vine");
        let lifts: Vec<Lift> = vine_feet
            .into_iter()
            .map(|(foot, dir)| {
                let (dx, dy) = dir.delta();
                let mut tiles = Vec::new();
                let mut at = (foot.0 + dx, foot.1 + dy);
                while vine(at) {
                    tiles.push(at);
                    at = (at.0 + dx, at.1 + dy);
                }
                assert!(!tiles.is_empty(), "a vine's foot at {foot:?} with no vine {dir:?} of it");
                Lift { foot, vine: tiles, top: at, dir }
            })
            .collect();
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
        Self { w, h, tiles, items, plants, bins, shops, particles: Vec::new(), boards, footbridge, bridge, wagons, signs, labels, starts, stands, townsfolk, plaques, workers, supplies, props, racks, gates, lifts, choices, provinces, looks, bells: Vec::new(), last_item_id: 0, seed: 0x9E37_79B9 }
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

    /// The look of the region this tile is in: the first province's outside every region.
    pub fn style_at(&self, x: i32, y: i32) -> Style {
        self.looks.iter().find(|&&((rx, ry, rw, rh), _)| (rx..rx + rw).contains(&x) && (ry..ry + rh).contains(&y)).map_or(Style::Cloth, |&(_, s)| s)
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
        self.walkable(x, y) && !matches!(self.tile(x, y), Tile::Footbridge | Tile::Stair)
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

    /// Gate `g` opens: its tiles are road from now on.
    pub fn open_gate(&mut self, g: usize) {
        for &(x, y) in &self.gates[g].tiles {
            self.tiles[(y * self.w + x) as usize] = Tile::Path;
        }
    }

    pub fn gate_open(&self, g: usize) -> bool {
        self.boards[self.gates[g].board].done()
    }

    /// The gate whose tile this is, if any.
    pub fn gate_at(&self, x: i32, y: i32) -> Option<usize> {
        self.gates.iter().position(|g| g.tiles.contains(&(x, y)))
    }

    /// Is `(x, y)` in a province still closed off, every gate into it shut? What grows or is
    /// made there doesn't count as made in the world yet.
    pub fn closed_at(&self, x: i32, y: i32) -> bool {
        let Some(&(_, province)) = self.provinces.iter().find(|&&((rx, ry, rw, rh), _)| (rx..rx + rw).contains(&x) && (ry..ry + rh).contains(&y)) else { return false };
        !province.is_empty() && !(0..self.gates.len()).any(|g| self.gates[g].province == province && self.gate_open(g))
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

    /// The prop covering tile `(x, y)`, and the tile it starts at.
    pub fn prop_at(&self, x: i32, y: i32) -> Option<((i32, i32), Prop)> {
        self.props.iter().copied().find(|&((px, py), p)| {
            let (w, h) = p.kind().size;
            (px..px + w).contains(&x) && (py..py + h).contains(&y)
        })
    }

    /// The workshop the choice board at this tile chooses for, if one stands there.
    pub fn choice_at(&self, x: i32, y: i32) -> Option<usize> {
        self.choices.iter().find(|&&(at, _)| at == (x, y)).map(|&(_, s)| s)
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

    /// A note rising just right of the head of a monster at tile `(tx, ty)` singing at its work,
    /// gone before it reaches a work bar over it, and clear of the marker over the one you steer.
    pub fn note(&mut self, tx: i32, ty: i32) {
        let ox = self.rand(2) as i32;
        self.particles.push(Particle { x: tx * TS + 16 + ox, y: ty * TS + 5, vx: 0, vy: 0, life: 20, kind: PKind::Note });
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
                // A note drifts up slowly, swaying.
                PKind::Note => (p.vy, p.vx) = (-((frame % 2 == 0) as i32), if frame % 8 < 4 { 0 } else { (frame / 8 % 2) as i32 * 2 - 1 }),
                _ => {}
            }
        }
        self.particles.retain(|p| p.life > 0);
    }
}
