//! Proving-ground designs: a 41x41 pixel grid where every colour is a part, and the stats that
//! drawing implies. Nothing here knows what "kind" of ship a design is: a fighter is just a
//! small drawing with engines to spare.
//!
//! Round 2 rules: every pixel must connect to the core through other pixels (edges, not
//! corners); plate shrugs off hits no stronger than it; the cannon is a 3x3 block.
//! Round 3: extra cores are allowed at a steeply rising cost, and every engine needs a 3-wide
//! way out of the ship for its exhaust (straight out since round 4, so it's always in the open).
//! Round 4: ships up to 99x99 (odd, so the mirror has a centre column), parts sorted into
//! categories, and the editing tools big ships need: rectangles, fills and stamps.

use crate::palette as pal;

pub const GRID: i32 = 99;
pub const FLAK_RANGE: i32 = 48;
pub const LANCE_RANGE: i32 = 100;
pub const CANNON_RANGE: i32 = 130;
/// Ticks between shots, per gun.
pub const FLAK_PERIOD: i32 = 20;
pub const LANCE_PERIOD: i32 = 45;
pub const CANNON_PERIOD: i32 = 45;
/// How hard each gun's shots pierce (2026-11-15; until then the same numbers were called
/// damage). A shot breaks a pixel only if it pierces more than the pixel's armour (bare 0, plate
/// 1, heavy plate 2); otherwise it bounces off and stops there. Pixels have no hit points, so
/// that's all a hit is: the pixel breaks or it doesn't.
pub const FLAK_PIERCE: i32 = 1;
pub const LANCE_PIERCE: i32 = 2;
pub const CANNON_PIERCE: i32 = 3;
/// The most a shot pierces: one more than the heaviest armour, so there's always an answer.
pub const PIERCE_TOP: i32 = 3;
/// How deep a lance shot goes: each pixel it breaks uses up 1 and the pixel's armour more (bare
/// 1, plate 2, heavy plate 3).
pub const LANCE_DEPTH: i32 = 6;
/// Ticks a ship with two or more scoops takes to bite a pixel out of a rock (one scoop takes
/// twice as long). 40 until 2026-11-11.
pub const CHEW_TICKS: i32 = 50;
/// Pixels each scoop carries.
pub const SCOOP_CARRY: i32 = 3;
/// The first line of a save file. Round 4 saves each design trimmed to its drawing, with where it
/// sits on the grid; round 2 and 3 files saved the whole 41x41 grid and still load, centred.
pub const SAVE_HEADER: &str = "#ROUND4";
pub const OLD_SAVE_HEADER: &str = "#ROUND2";
/// Ticks for one factory pixel to print one ship pixel (4 s).
pub const PRINT_TICKS: i32 = 400;
/// Ship pixels a factory can build per factory pixel: the biggest ship it can make.
pub const FACTORY_REACH: i32 = 10;
/// What the second core costs on top of the ship's pixels. Each core after that costs twice the
/// one before: +8, +16, +32 and so on.
pub const CORE_EXTRA: i32 = 8;
/// The beam (2026-11-12). Beam pixels one behind another in a column make one gun. It reaches
/// `BEAM_REACH0` px and `BEAM_REACH` more for each pixel of the gun, up to `BEAM_MAX` of them.
pub const BEAM_REACH0: i32 = 40;
pub const BEAM_REACH: i32 = 12;
pub const BEAM_MAX: i32 = 16;
/// Pixels a beam cuts for each pixel of the gun. Each uses up 1 and its armour more, as with a
/// lance. A beam pierces 3: it was 2 until 2026-11-15, when nothing had armour 2, so heavy plate
/// stops a small ship's lance but not a big ship's beam.
pub const BEAM_CUT: i32 = 4;
pub const BEAM_PIERCE: i32 = 3;
/// Ticks a beam shows its line before it cuts, and ticks it rests after.
pub const BEAM_WARM: i32 = 15;
pub const BEAM_PERIOD: i32 = 75;
/// The shield (2026-11-12): the radius of the circle each shield pixel throws, the shots it stops
/// before it's out, and the ticks after the last shot it stopped before it's full again.
pub const SHIELD_R: i32 = 10;
pub const SHIELD_CHARGES: u8 = 4;
pub const SHIELD_REST: i32 = 45;

/// The missile (2026-11-12). A launcher pixel sends one at the nearest enemy within
/// `MISSILE_RANGE` px, in any direction, every `MISSILE_PERIOD` ticks. It flies `MISSILE_SPEED`
/// thousandths of a pixel a tick, which is as fast as the fastest ship, straight at wherever its
/// target is now, for `MISSILE_LIFE` ticks, and bursts like a cannon's shell. Flak shoots it down.
pub const MISSILE_RANGE: i32 = 200;
pub const MISSILE_PERIOD: i32 = 90;
pub const MISSILE_SPEED: i32 = 3000;
pub const MISSILE_LIFE: u32 = 120;
/// A missile's burst pierces 2 (Henry, 2026-11-15: "I think missile should be piercing 2 to start
/// with"): it breaks bare pixels and plate, as it always did, and not heavy plate.
pub const MISSILE_PIERCE: i32 = 2;

/// How fast a missile flies, in thousandths of a pixel a tick, from how hard it pierces: one
/// beside a piercer pierces more than a missile does and flies a quarter slower, as any heavier
/// round does.
pub fn missile_rate(pierce: i32) -> i32 {
    rounds(MISSILE_PIERCE, MISSILE_SPEED, pierce > MISSILE_PIERCE).1
}

/// The repeater (2026-11-13). Repeater pixels one behind another in a column make one gun, as
/// beam pixels do. It fires small shots straight ahead: it reaches `REPEAT_REACH0` px and
/// `REPEAT_REACH` more for each pixel of the gun, up to `REPEAT_MAX` of them, and the longer it is
/// the faster it fires (`repeat_period`).
pub const REPEAT_REACH0: i32 = 60;
pub const REPEAT_REACH: i32 = 8;
pub const REPEAT_MAX: i32 = 16;
/// The shield projector (2026-11-13). A lump of projector pixels reaches `PROJ_REACH` px for each
/// pixel of the square root of its pixels, up to `PROJ_REACH_MAX`, and its ship holds a charge
/// for each projector pixel, full again `PROJ_REST` ticks after the last shot stopped. A pixel
/// needs `PROJ_NEAR` projector pixels among the eight round it, and its own row or column clear
/// to the edge of the ship in `PROJ_OPEN` of the four directions.
pub const PROJ_REACH: i32 = 10;
pub const PROJ_REACH_MAX: i32 = 150;
pub const PROJ_REST: i32 = 45;
pub const PROJ_NEAR: usize = 4;
pub const PROJ_OPEN: usize = 3;

/// The tracker (2026-11-15): a 3x3 turret that turns fast and goes for the small ships in reach,
/// putting quick shots into the middle of one. They pierce 1, so plate turns them. It uses twice
/// a cannon's power, and power comes from hull, so it's a big ship's gun.
pub const TRACKER_RANGE: i32 = 130;
pub const TRACKER_PERIOD: i32 = 6;
pub const TRACKER_PIERCE: i32 = 1;

/// A gun's shots as they leave it: how hard they pierce and how fast they fly. A gun touching a
/// standing piercer pixel (2026-11-15) fires heavier rounds: one level more piercing, up to
/// `PIERCE_TOP`, and a quarter slower, so a nimble ship is harder to hit. Touching two is no
/// better than one, and a gun already at the top isn't changed at all.
pub fn rounds(pierce: i32, speed: i32, heavy: bool) -> (i32, i32) {
    if heavy && pierce < PIERCE_TOP {
        (pierce + 1, speed - speed / 4)
    } else {
        (pierce, speed)
    }
}

/// How far a repeater gun of `n` pixels reaches (0 for none).
pub fn repeat_reach(n: i32) -> i32 {
    if n <= 0 {
        0
    } else {
        REPEAT_REACH0 + REPEAT_REACH * n.min(REPEAT_MAX)
    }
}

/// Ticks between a repeater gun's shots: 8 for one pixel, a tick less for every two more, down to 3.
pub fn repeat_period(n: i32) -> i32 {
    (9 - (n.clamp(1, REPEAT_MAX) + 1) / 2).max(3)
}

/// How far a lump of `px` projector pixels reaches.
pub fn proj_reach(px: i32) -> i32 {
    let mut r = 0;
    while (r + 1) * (r + 1) <= px {
        r += 1;
    }
    (PROJ_REACH * r).min(PROJ_REACH_MAX)
}

/// Projector pixels that don't work where they are: too thin (fewer than `PROJ_NEAR` projector
/// pixels among the eight round them) or hemmed in (their own row or column isn't clear to the
/// edge of the ship in `PROJ_OPEN` of the four directions; other projector pixels don't count as
/// in the way). Each pixel passes or fails where it stands, as an engine does. `part` says what a
/// cell holds (anything outside the drawing is empty). Gives the pixels that fail, and how many of
/// them fail for being thin.
pub fn hemmed_projectors(bbox: (i32, i32, i32, i32), part: impl Fn(i32, i32) -> Option<Part>) -> (Vec<(i32, i32)>, usize) {
    let (x0, y0, x1, y1) = bbox;
    let is = |x: i32, y: i32| part(x, y) == Some(Part::Projector);
    let (mut bad, mut thin) = (Vec::new(), 0);
    for y in y0..=y1 {
        for x in x0..=x1 {
            if !is(x, y) {
                continue;
            }
            let near = (-1..=1).flat_map(|dy| (-1..=1).map(move |dx| (dx, dy))).filter(|&(dx, dy)| (dx, dy) != (0, 0) && is(x + dx, y + dy)).count();
            let open = [(0, 1), (0, -1), (1, 0), (-1, 0)]
                .iter()
                .filter(|&&(dx, dy)| (1..).map(|k| (x + dx * k, y + dy * k)).take_while(|&(px, py)| (x0..=x1).contains(&px) && (y0..=y1).contains(&py)).all(|(px, py)| part(px, py).map_or(true, |p| p == Part::Projector)))
                .count();
            if near < PROJ_NEAR || open < PROJ_OPEN {
                bad.push((x, y));
                thin += (near < PROJ_NEAR) as usize;
            }
        }
    }
    (bad, thin)
}

/// How far a beam gun of `n` pixels reaches.
pub fn beam_reach(n: i32) -> i32 {
    if n <= 0 {
        0
    } else {
        BEAM_REACH0 + BEAM_REACH * n.min(BEAM_MAX)
    }
}

/// How many pixels a beam gun of `n` pixels cuts.
pub fn beam_cut(n: i32) -> i32 {
    BEAM_CUT * n.max(0)
}

/// The beam guns in a w x h grid of cells, north up (so up a column is forward): each unbroken
/// run of beam cells down a column is one gun. Gives each gun's front cell, its muzzle, and its
/// length. `beam` says whether a cell holds a beam pixel (a living one, on a ship).
pub fn beam_guns(w: i32, h: i32, beam: impl Fn(usize) -> bool) -> Vec<(usize, i32)> {
    let mut guns = Vec::new();
    for x in 0..w {
        let mut y = 0;
        while y < h {
            if beam((y * w + x) as usize) {
                let top = y;
                while y < h && beam((y * w + x) as usize) {
                    y += 1;
                }
                guns.push(((top * w + x) as usize, y - top));
            } else {
                y += 1;
            }
        }
    }
    guns
}

/// The extra cost of a ship's cores beyond the first.
pub fn core_cost(cores: i32) -> i32 {
    if cores <= 1 {
        0
    } else {
        CORE_EXTRA.saturating_mul((1 << (cores - 1).min(24)) - 1)
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash)]
pub enum Part {
    Hull,
    Plate,
    Flak,
    Lance,
    Cannon,
    Engine,
    Scoop,
    Core,
    /// Prints ships (match only): speed and the biggest ship it can build come from the count.
    Factory,
    /// A gun made of a column of these: the longer, the further it reaches and the more it cuts.
    Beam,
    /// Throws a circle round itself that stops enemy shots coming in.
    Shield,
    /// Launches missiles that chase their target and that flak can shoot down.
    Missile,
    /// A gun made of a column of these that fires a stream of small shots straight ahead.
    Repeater,
    /// A lump of these stops shots at every friend in reach. Never at its own ship, and never
    /// at a projector pixel.
    Projector,
    /// Armour 2, in dominoes two wide or two tall: only shots that pierce 3 break it. (The parts
    /// of 2026-11-15 come last so the older parts keep their numbers.)
    HeavyPlate,
    /// A gun touching it fires heavier rounds: one level more piercing, a quarter slower.
    Piercer,
    /// A 3x3 turret that turns fast and goes for the small ships in reach.
    Tracker,
}

/// The parts in the order the yard shows them.
pub const PARTS: [Part; 17] = [
    Part::Hull,
    Part::Plate,
    Part::HeavyPlate,
    Part::Shield,
    Part::Projector,
    Part::Flak,
    Part::Lance,
    Part::Beam,
    Part::Cannon,
    Part::Tracker,
    Part::Missile,
    Part::Repeater,
    Part::Piercer,
    Part::Engine,
    Part::Scoop,
    Part::Factory,
    Part::Core,
];

/// The parts that come in square blocks, which the editor only ever puts down or takes away
/// whole: the 3x3 cannon and tracker. (Heavy plate comes in dominoes: `Design::dominoes`.)
pub const BLOCK_PARTS: [Part; 2] = [Part::Cannon, Part::Tracker];

/// The yard's part categories, keys 1-6. UPGRADES (key 4) was POWER, which stayed empty (hull
/// is the power) until 2026-11-15, when an eighth weapon didn't fit the weapons' two rows of
/// buttons: parts that make other parts better go there, the piercer first.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Cat {
    Armor,
    Weapons,
    Drive,
    Upgrades,
    Industry,
    Command,
}

pub const CATS: [Cat; 6] = [Cat::Armor, Cat::Weapons, Cat::Drive, Cat::Upgrades, Cat::Industry, Cat::Command];

impl Cat {
    pub fn name(self) -> &'static str {
        match self {
            Cat::Armor => "HULL AND ARMOR",
            Cat::Weapons => "WEAPONS",
            Cat::Drive => "DRIVE",
            Cat::Upgrades => "UPGRADES",
            Cat::Industry => "INDUSTRY",
            Cat::Command => "COMMAND",
        }
    }
    /// The category's parts, in the yard's order.
    pub fn parts(self) -> Vec<Part> {
        PARTS.iter().copied().filter(|p| p.info().cat == self).collect()
    }
    pub fn of(p: Part) -> Cat {
        p.info().cat
    }
}

/// Everything about a part that isn't behaviour, in one place. A new part is a variant of `Part`,
/// a row here and a place in `PARTS`; the yard, the stats, the save format and the drawing all
/// read from this.
pub struct PartInfo {
    /// Its letter in designs and save files.
    pub ch: char,
    pub name: &'static str,
    pub cat: Cat,
    /// Weight of one pixel.
    pub mass: i32,
    /// Hits this strong or weaker bounce off.
    pub armor: i32,
    /// Power one pixel makes, and what one uses (round 5). A cannon's is for the whole block.
    pub makes: i32,
    pub uses: i32,
    /// Pixels in one of it: 9 for a 3x3 turret, 2 for heavy plate's domino, 1 for everything else.
    pub block: i32,
    /// Its palette colour. None is the team's own colour (hull).
    pub colour: Option<u8>,
    /// Which part shows when a shrunken icon has one pixel for a block of cells: highest wins.
    pub rank: i32,
    /// The 7x7 shape drawn inside its colour, so parts stay easy to tell apart.
    pub glyph: [&'static str; 7],
    /// What it does. Its numbers come from the constants; power and weight are added from above.
    pub about: fn() -> String,
}

/// A number of ticks as seconds, to a tenth: "1.3", or "3" when it's whole.
pub fn secs(ticks: i32) -> String {
    let tenths = ticks * 10 / 15;
    if tenths % 10 == 0 {
        format!("{}", tenths / 10)
    } else {
        format!("{}.{}", tenths / 10, tenths % 10)
    }
}

/// One row per part, in the order of the `Part` enum.
pub const PART_INFO: [PartInfo; 17] = [
    PartInfo {
        ch: 'h',
        name: "HULL",
        cat: Cat::Armor,
        mass: 1,
        armor: 0,
        makes: 1,
        uses: 0,
        block: 1,
        colour: None,
        rank: 0,
        glyph: [".......", ".......", ".......", ".......", ".......", ".......", "......."],
        about: || "HOLDS THE SHIP TOGETHER, AND IS WHERE ITS POWER COMES FROM. ANY HIT BREAKS IT.".into(),
    },
    PartInfo {
        ch: 'p',
        name: "PLATE",
        cat: Cat::Armor,
        mass: 2,
        armor: 1,
        makes: 0,
        uses: 0,
        block: 1,
        colour: Some(pal::STEEL),
        rank: 1,
        glyph: [".......", ".#...#.", ".......", ".......", ".......", ".#...#.", "......."],
        about: || "ARMOR 1: A SHOT THAT PIERCES ONLY 1 BOUNCES OFF IT (FLAK, REPEATERS, TRACKERS). A LANCE OR BEAM USES UP 2 OF ITS DEPTH BREAKING IT.".into(),
    },
    PartInfo {
        ch: 'g',
        name: "FLAK",
        cat: Cat::Weapons,
        mass: 1,
        armor: 0,
        makes: 0,
        uses: 1,
        block: 1,
        colour: Some(pal::GUN),
        rank: 4,
        glyph: [".......", "...#...", "...#...", "..###..", "..###..", ".......", "......."],
        about: || format!("A TURRET. SHOOTS THE NEAREST ENEMY WITHIN {FLAK_RANGE} PX, LEADING ITS AIM AND SPREADING FIRE OVER SEVERAL TARGETS. IT PIERCES {FLAK_PIERCE}, SO PLATE TURNS IT. ONE SHOT EVERY {} S.", secs(FLAK_PERIOD)),
    },
    PartInfo {
        ch: 'l',
        name: "LANCE",
        cat: Cat::Weapons,
        mass: 2,
        armor: 0,
        makes: 0,
        uses: 3,
        block: 1,
        colour: Some(pal::LANCE),
        rank: 5,
        glyph: ["...#...", "..###..", ".#.#.#.", "...#...", "...#...", "...#...", "......."],
        about: || format!("SHOOTS STRAIGHT AHEAD ONLY, UP TO {LANCE_RANGE} PX. PIERCES {LANCE_PIERCE} AND PUNCHES THROUGH {LANCE_DEPTH} PIXELS, OR {} OF PLATE. ONE SHOT EVERY {} S. IT HOLDS FIRE IF ITS TARGET COULD BE OFF ITS LINE WHEN THE SHOT ARRIVES.", LANCE_DEPTH / 2, secs(LANCE_PERIOD)),
    },
    PartInfo {
        ch: 'n',
        name: "CANNON",
        cat: Cat::Weapons,
        mass: 2,
        armor: 0,
        makes: 0,
        uses: 8,
        block: 9,
        colour: Some(pal::CANNON),
        rank: 6,
        glyph: [".......", "..###..", ".#...#.", ".#...#.", ".#...#.", "..###..", "......."],
        about: || format!("A BIG 3X3 TURRET, {CANNON_RANGE} PX. IT TURNS SLOWLY, SO FAST CLOSE SHIPS ESCAPE IT. ITS SHELL PIERCES {CANNON_PIERCE} AND BLASTS A 3X3 HOLE, ARMOR AND ALL. IT GOES FOR THE BIG SHIPS IN RANGE. ONE SHOT EVERY {} S.", secs(CANNON_PERIOD)),
    },
    PartInfo {
        ch: 'e',
        name: "ENGINE",
        cat: Cat::Drive,
        mass: 1,
        armor: 0,
        makes: 0,
        uses: 1,
        block: 1,
        colour: Some(pal::ENGINE),
        rank: 3,
        glyph: ["#.....#", ".#...#.", "..#.#..", "#..#..#", ".#...#.", "..#.#..", "...#..."],
        about: || "THRUST: MORE OF IT FOR THE WEIGHT MEANS FASTER AND QUICKER TO TURN. ITS EXHAUST NEEDS A STRAIGHT WAY OUT, 3 WIDE, IN ONE OF THE FOUR DIRECTIONS. A BURIED ENGINE SHOWS RED.".into(),
    },
    PartInfo {
        ch: 's',
        name: "SCOOP",
        cat: Cat::Industry,
        mass: 2,
        armor: 0,
        makes: 0,
        uses: 1,
        block: 1,
        colour: Some(pal::SCOOP),
        rank: 2,
        glyph: [".......", ".#...#.", ".#...#.", ".#...#.", "..###..", ".......", "......."],
        about: || format!("MINES ROCK AND PICKS UP LOOSE PIXELS. EACH SCOOP CARRIES {SCOOP_CARRY}."),
    },
    PartInfo {
        ch: 'c',
        name: "CORE",
        cat: Cat::Command,
        mass: 1,
        armor: 0,
        makes: 4,
        uses: 0,
        block: 1,
        colour: Some(pal::CORE),
        rank: 8,
        glyph: ["...#...", "..#.#..", ".#...#.", "#.....#", ".#...#.", "..#.#..", "...#..."],
        about: || format!("THE SHIP'S LIFE: IT DIES WHEN ITS LAST CORE GOES. EXTRA CORES COST +{}, +{}, +{}. CUT IN TWO, EVERY PIECE WITH A CORE FLIES ON AS A SHIP.", CORE_EXTRA, CORE_EXTRA * 2, CORE_EXTRA * 4),
    },
    PartInfo {
        ch: 'f',
        name: "FACTORY",
        cat: Cat::Industry,
        mass: 1,
        armor: 0,
        makes: 0,
        uses: 2,
        block: 1,
        colour: Some(pal::FACTORY),
        rank: 7,
        glyph: [".......", ".#.#.#.", ".......", ".#.#.#.", ".......", ".#.#.#.", "......."],
        about: || format!("IN A MATCH, PRINTS ITS BUILD ORDER FROM YOUR BANK: 1 PIXEL EVERY {} S EACH, AND SHIPS UP TO {FACTORY_REACH} PX BIGGER EACH. A SHIP WITH ANY STORES PIXELS AND LINKS YOUR NETWORK.", secs(PRINT_TICKS)),
    },
    PartInfo {
        ch: 'b',
        name: "BEAM",
        cat: Cat::Weapons,
        mass: 2,
        armor: 0,
        makes: 0,
        uses: 3,
        block: 1,
        colour: Some(pal::BEAM),
        rank: 5,
        glyph: ["..#.#..", "..#.#..", "..#.#..", "..#.#..", "..#.#..", "..###..", "......."],
        about: || {
            format!(
                "A COLUMN OF THESE IS ONE GUN, FIRING STRAIGHT AHEAD. ONE REACHES {} PX, EACH MORE ADDS {BEAM_REACH} (UP TO {BEAM_MAX}). IT PIERCES {BEAM_PIERCE} AND CUTS {BEAM_CUT} PIXELS FOR EACH, PLATE COUNTING DOUBLE AND HEAVY PLATE TRIPLE. IT SHOWS ITS LINE FOR {} S, CUTS, AND RESTS {} S.",
                beam_reach(1),
                secs(BEAM_WARM),
                secs(BEAM_PERIOD)
            )
        },
    },
    PartInfo {
        ch: 'o',
        name: "SHIELD",
        cat: Cat::Armor,
        mass: 2,
        armor: 0,
        makes: 0,
        uses: 4,
        block: 1,
        colour: Some(pal::SHIELD),
        rank: 2,
        glyph: [".......", "..###..", ".#...#.", "#.....#", "#..#..#", ".......", "......."],
        about: || format!("THROWS A CIRCLE {SHIELD_R} PX ROUND ITSELF. AN ENEMY SHOT OF ANY KIND COMING INTO THE CIRCLE STOPS AT ITS EDGE. AFTER {SHIELD_CHARGES} THE CIRCLE IS OUT, AND {} S AFTER THE LAST IT'S BACK. SHOTS FIRED FROM INSIDE IT PASS.", secs(SHIELD_REST)),
    },
    PartInfo {
        ch: 'm',
        name: "MISSILE",
        cat: Cat::Weapons,
        mass: 2,
        armor: 0,
        makes: 0,
        uses: 4,
        block: 1,
        colour: Some(pal::MISSILE),
        rank: 5,
        glyph: ["...#...", "..###..", "..###..", "..###..", "..###..", ".#.#.#.", "......."],
        about: || format!("A LAUNCHER. EVERY {} S IT SENDS A MISSILE AT THE NEAREST ENEMY WITHIN {MISSILE_RANGE} PX, ANY WAY IT FACES. THE MISSILE CHASES AS FAST AS THE FASTEST SHIP AND BLASTS A 3X3 HOLE, PIERCING {MISSILE_PIERCE}. FLAK SHOOTS IT DOWN. A SHIELD STOPS IT.", secs(MISSILE_PERIOD)),
    },
    PartInfo {
        ch: 'r',
        name: "REPEATER",
        cat: Cat::Weapons,
        mass: 1,
        armor: 0,
        makes: 0,
        uses: 2,
        block: 1,
        colour: Some(pal::REPEATER),
        rank: 5,
        glyph: ["...#...", ".......", "...#...", ".......", "...#...", "..###..", "..###.."],
        about: || format!("A STREAM OF SMALL SHOTS STRAIGHT AHEAD. PIXELS IN A COLUMN ARE ONE GUN: {} PX AND {} SHOTS A SECOND WITH ONE, {} PX AND {} WITH {REPEAT_MAX}. THEY PIERCE {FLAK_PIERCE}, SO PLATE TURNS THEM. EACH COSTS A SHIELD A CHARGE.", repeat_reach(1), (15 + repeat_period(1) / 2) / repeat_period(1), repeat_reach(REPEAT_MAX), 15 / repeat_period(REPEAT_MAX)),
    },
    PartInfo {
        ch: 'j',
        name: "PROJECTOR",
        cat: Cat::Armor,
        mass: 1,
        armor: 0,
        makes: 0,
        uses: 2,
        block: 1,
        colour: Some(pal::PROJECTOR),
        rank: 2,
        glyph: [".......", "..###..", ".#####.", ".#####.", ".#####.", "..###..", "......."],
        about: || format!("STOPS SHOTS AT FRIENDS IN REACH, NOT AT ITS OWN SHIP OR ANY PROJECTOR PIXEL: ONE A PIXEL, ALL BACK AFTER {} S WITH NONE. 16 PX REACH {}, 64 REACH {}. EACH NEEDS {PROJ_NEAR} PROJECTOR PIXELS ROUND IT AND {PROJ_OPEN} OPEN SIDES.", secs(PROJ_REST), proj_reach(16), proj_reach(64)),
    },
    PartInfo {
        ch: 'a',
        name: "HEAVY PLATE",
        cat: Cat::Armor,
        mass: 4,
        armor: 2,
        makes: 0,
        uses: 0,
        block: 2,
        colour: Some(pal::HEAVY),
        rank: 1,
        glyph: [".......", ".##.##.", ".##.##.", ".......", ".##.##.", ".##.##.", "......."],
        about: || format!("ARMOR 2, IN DOMINOES 2 WIDE OR 2 TALL ([T] TURNS IT): ONLY A SHOT THAT PIERCES {PIERCE_TOP} BREAKS IT (CANNONS, BEAMS, AND A LANCE OR MISSILE BESIDE A PIERCER). A LANCE OR BEAM USES UP 3 OF ITS DEPTH BREAKING IT."),
    },
    PartInfo {
        ch: 'i',
        name: "PIERCER",
        cat: Cat::Upgrades,
        mass: 1,
        armor: 0,
        makes: 0,
        uses: 2,
        block: 1,
        colour: Some(pal::PIERCER),
        rank: 4,
        glyph: ["...#...", "...#...", "..###..", "..###..", ".#####.", ".......", "......."],
        about: || format!("A GUN TOUCHING IT, EDGE TO EDGE, FIRES HEAVIER ROUNDS: THEY PIERCE ONE MORE, UP TO {PIERCE_TOP}, AND FLY A QUARTER SLOWER, SO NIMBLE SHIPS ARE HARDER TO HIT. TWO ON ONE GUN ARE NO BETTER THAN ONE."),
    },
    PartInfo {
        ch: 't',
        name: "TRACKER",
        cat: Cat::Weapons,
        mass: 2,
        armor: 0,
        makes: 0,
        uses: 16,
        block: 9,
        colour: Some(pal::TRACKER),
        rank: 6,
        glyph: [".......", "#.....#", "#.....#", ".#...#.", "..###..", "...#...", "..###.."],
        about: || format!("A BIG 3X3 TURRET, {TRACKER_RANGE} PX, THAT TURNS FAST AND GOES FOR THE SMALL SHIPS IN RANGE, PUTTING QUICK SHOTS INTO THE MIDDLE OF ONE. THEY PIERCE {TRACKER_PIERCE}, SO PLATE TURNS THEM. ONE SHOT EVERY {} S.", secs(TRACKER_PERIOD)),
    },
];

impl Part {
    pub fn info(self) -> &'static PartInfo {
        &PART_INFO[self as usize]
    }
    /// Hits this strong or weaker bounce off.
    pub fn armor(self) -> i32 {
        self.info().armor
    }
    pub fn mass(self) -> i32 {
        self.info().mass
    }
    pub fn ch(self) -> char {
        self.info().ch
    }
    pub fn from_ch(c: char) -> Option<Part> {
        PARTS.iter().copied().find(|p| p.ch() == c)
    }
    pub fn name(self) -> &'static str {
        self.info().name
    }
    /// The side of the square it comes in: 3 for a turret, 1 for everything else (heavy plate
    /// comes in dominoes, not squares).
    pub fn side(self) -> i32 {
        if self.info().block == 9 {
            3
        } else {
            1
        }
    }
    /// A 3x3 turret: a gun whose centre pixel is its breech.
    pub fn turret(self) -> bool {
        matches!(self, Part::Cannon | Part::Tracker)
    }
    /// The yard's description: what it does, then its power and weight from the table.
    pub fn blurb(self) -> String {
        let i = self.info();
        let power = match (i.makes, i.uses) {
            (0, 0) => "NO POWER".to_string(),
            (m, 0) => format!("MAKES {m} POWER"),
            (_, u) => format!("USES {u} POWER"),
        };
        format!("{} - {} {power}. WEIGHS {}.", i.name, (i.about)(), i.mass * i.block)
    }
    /// Who makes power and who uses it, for the help and the power gauge: "HULL 1, CORE 4". A part
    /// that comes as a block (a turret) says so, since its number is for the whole block.
    pub fn power_list(makers: bool) -> String {
        let of = |p: &Part| if makers { p.info().makes } else { p.info().uses };
        let block = |p: &Part| if p.side() > 1 { format!(" ({0}X{0})", p.side()) } else { String::new() };
        PARTS.iter().filter(|p| of(p) > 0).map(|p| format!("{}{} {}", p.name(), block(p), of(p))).collect::<Vec<_>>().join(", ")
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash)]
pub enum Order {
    Hunt,
    Guard,
    Escort,
    Harvest,
    Hold,
    /// Sabotage: goes for the other side's miners and factories.
    Sab,
}

/// Every order there is.
pub const ORDERS: [Order; 6] = [Order::Hunt, Order::Sab, Order::Guard, Order::Escort, Order::Harvest, Order::Hold];

impl Order {
    pub fn name(self) -> &'static str {
        match self {
            Order::Hunt => "HUNT",
            Order::Guard => "GUARD",
            Order::Escort => "ESCORT",
            Order::Harvest => "HARVEST",
            Order::Hold => "HOLD",
            Order::Sab => "SAB",
        }
    }
    fn from_name(s: &str) -> Option<Order> {
        ORDERS.iter().copied().find(|o| o.name() == s)
    }
}

/// How a ship flies, which follows from its drawing alone.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash)]
pub enum Flight {
    /// At least 1 thrust per weight: dives through the target and loops back.
    Runs,
    /// Heavier: holds at weapon range, lined up so its lances bear.
    StandOff,
    /// No engines: stays put and can't turn.
    Still,
}

impl Flight {
    pub fn name(self) -> &'static str {
        match self {
            Flight::Runs => "ATTACK RUNS",
            Flight::StandOff => "STAND-OFF",
            Flight::Still => "STAYS PUT",
        }
    }
}

/// How a ship flies at what it's fighting. It's set on the design in the yard and fixed for a
/// match. Until it's set the drawing decides: enough thrust for the weight runs, less kites.
/// Either way the drawing decides how well it works.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash)]
pub enum Style {
    /// Dives through the target and loops back.
    Runs,
    /// Holds at weapon range, and backs away from whatever comes closer.
    Kite,
    /// Circles the target at weapon range, flat out.
    Orbit,
}

pub const STYLES: [Style; 3] = [Style::Runs, Style::Kite, Style::Orbit];

impl Style {
    /// What a drawing flies as when nothing was chosen for it. None: it has no engines.
    pub fn of(f: Flight) -> Option<Style> {
        match f {
            Flight::Runs => Some(Style::Runs),
            Flight::StandOff => Some(Style::Kite),
            Flight::Still => None,
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash)]
pub struct Stats {
    pub count: i32,
    pub cores: i32,
    /// What the ship costs to build: its pixels plus its extra cores.
    pub cost: i32,
    pub mass: i32,
    pub engines: i32,
    pub flak: i32,
    pub lance: i32,
    pub cannon: i32,
    /// Working tracker turrets.
    pub tracker: i32,
    /// Beam pixels, and the length of its longest beam gun (a column of them).
    pub beam: i32,
    pub beam_len: i32,
    pub shields: i32,
    /// Missile launchers.
    pub missile: i32,
    /// Repeater pixels, and the length of the longest repeater gun.
    pub repeater: i32,
    pub rep_len: i32,
    /// Projector pixels.
    pub projector: i32,
    pub scoops: i32,
    pub factory: i32,
    /// Power made (hull and cores) and used (engines and guns).
    pub made: i32,
    pub used: i32,
    /// Thrust after any power shortage.
    pub thrust: i32,
    /// Top speed in quarter-pixels per tick.
    pub vmax: i32,
    /// Ticks per quarter turn (0 = can't turn).
    pub turn: i32,
    pub flight: Flight,
}

impl Stats {
    /// The order a ship of this drawing starts on. It follows from the drawing and nobody sets
    /// it: with engines and a scoop it mines, with engines and no scoop it fights, and with no
    /// engines it holds where it is. (A design carried an order chosen in the yard until
    /// 2026-11-14. Orders are now what a side's AI gives its cards, and what a person's stance
    /// gives every ship: nothing a drawing has.)
    pub fn order(&self) -> Order {
        if self.thrust == 0 {
            Order::Hold
        } else if self.scoops > 0 {
            Order::Harvest
        } else {
            Order::Hunt
        }
    }
    pub fn armed(&self) -> bool {
        self.flak + self.lance + self.cannon + self.tracker + self.beam + self.missile + self.repeater > 0
    }
    /// Whether it has a gun that fires straight ahead only, which the whole ship lines up.
    pub fn aims(&self) -> bool {
        self.lance + self.beam + self.repeater > 0
    }
    /// How far its longest beam gun reaches (0: it has none).
    pub fn beam_reach(&self) -> i32 {
        beam_reach(self.beam_len)
    }
    /// The reach of the gun it lines up: its longest beam or a lance, whichever reaches further.
    pub fn fwd_reach(&self) -> i32 {
        self.beam_reach().max(if self.lance > 0 { LANCE_RANGE } else { 0 }).max(repeat_reach(self.rep_len))
    }
    /// The biggest ship (in pixels) its factory pixels can build.
    pub fn max_build(&self) -> i32 {
        self.factory * FACTORY_REACH
    }
    /// How much of the power it needs the ship has, in percent (at most 100).
    pub fn power_pct(&self) -> i32 {
        if self.used <= self.made {
            100
        } else {
            self.made * 100 / self.used
        }
    }
    /// A gun's ticks between shots, stretched by any power shortage.
    pub fn reload(&self, period: i32) -> i32 {
        if self.used <= self.made {
            period
        } else {
            (period * self.used + self.made.max(1) - 1) / self.made.max(1)
        }
    }
    pub fn top_speed_px_s(&self) -> i32 {
        self.vmax * 15 / 4
    }
    /// Tenths of a second from standing still to top speed.
    pub fn zero_to_top_ds(&self) -> i32 {
        if self.thrust == 0 {
            0
        } else {
            self.vmax * self.mass * 10 / (self.thrust * 15)
        }
    }
    /// Tenths of a second per quarter turn.
    pub fn turn_ds(&self) -> i32 {
        self.turn * 10 / 15
    }
}

/// Stats from a set of cells. `cannons` and `trackers` are how many working turrets of each there
/// are (a turret is a block, so the cells alone can't say), and `beam_len` the length of the
/// longest beam gun (a column, which a list of cells can't say either).
pub fn stats_of(cells: &[Option<Part>], cannons: i32, trackers: i32, beam_len: i32, rep_len: i32) -> Stats {
    let n = |p: Part| cells.iter().filter(|c| **c == Some(p)).count() as i32;
    let count = cells.iter().flatten().count() as i32;
    let mass: i32 = cells.iter().flatten().map(|p| p.mass()).sum();
    let engines = n(Part::Engine);
    let (flak, lance, cores, factory) = (n(Part::Flak), n(Part::Lance), n(Part::Core), n(Part::Factory));
    // Power (round 5): what each pixel makes and uses comes from the part table. A turret's draw
    // is per working turret, not per pixel of its block.
    let made: i32 = PARTS.iter().map(|&p| n(p) * p.info().makes).sum();
    let used: i32 = PARTS
        .iter()
        .map(|&p| {
            let k = match p {
                Part::Cannon => cannons,
                Part::Tracker => trackers,
                _ => n(p),
            };
            k * p.info().uses
        })
        .sum();
    // Engines push in proportion to the power they get.
    let full = engines * 8;
    let thrust = if used > made { full * made / used } else { full };
    let vmax = if thrust == 0 { 0 } else { (thrust * 5 / mass.max(1) + 2).clamp(3, 12) };
    let turn = if thrust == 0 { 0 } else { (mass * 2 / thrust).clamp(1, 24) };
    let flight = if thrust == 0 {
        Flight::Still
    } else if thrust >= mass {
        Flight::Runs
    } else {
        Flight::StandOff
    };
    let cost = count + core_cost(cores);
    Stats { count, cores, cost, mass, engines, flak, lance, cannon: cannons, tracker: trackers, beam: n(Part::Beam), beam_len, shields: n(Part::Shield), missile: n(Part::Missile), repeater: n(Part::Repeater), rep_len, projector: n(Part::Projector), scoops: n(Part::Scoop), factory, made, used, thrust, vmax, turn, flight }
}

/// Which of a w x h grid's cells connect to `start` edge to edge, through cells where `on` holds.
pub fn reach(w: i32, h: i32, start: usize, on: impl Fn(usize) -> bool) -> Vec<bool> {
    let mut seen = vec![false; (w * h) as usize];
    if !on(start) {
        return seen;
    }
    seen[start] = true;
    let mut todo = vec![start];
    while let Some(i) = todo.pop() {
        let (x, y) = (i as i32 % w, i as i32 / w);
        for (dx, dy) in [(1, 0), (-1, 0), (0, 1), (0, -1)] {
            let (nx, ny) = (x + dx, y + dy);
            if nx < 0 || ny < 0 || nx >= w || ny >= h {
                continue;
            }
            let j = (ny * w + nx) as usize;
            if !seen[j] && on(j) {
                seen[j] = true;
                todo.push(j);
            }
        }
    }
    seen
}

/// A design trimmed to its bounding box, facing north (up is forward).
#[derive(Clone, Debug, Hash)]
pub struct Shape {
    pub w: i32,
    pub h: i32,
    pub cells: Vec<Option<Part>>,
    /// The centre cell of each turret block: the cannons', then the trackers'.
    pub turrets: Vec<usize>,
}

impl Shape {
    pub fn dims(&self, facing: u8) -> (i32, i32) {
        if facing & 1 == 1 {
            (self.h, self.w)
        } else {
            (self.w, self.h)
        }
    }
    /// Where cell `i` lands when the shape is centred on (x, y) with a facing (exact quarter turns).
    pub fn cell_xy(&self, x: i32, y: i32, facing: u8, i: usize) -> (i32, i32) {
        let (dx, dy) = (i as i32 % self.w, i as i32 / self.w);
        let (w, h, rx, ry) = match facing & 3 {
            0 => (self.w, self.h, dx, dy),
            1 => (self.h, self.w, self.h - 1 - dy, dx),
            2 => (self.w, self.h, self.w - 1 - dx, self.h - 1 - dy),
            _ => (self.h, self.w, dy, self.w - 1 - dx),
        };
        (x - w / 2 + rx, y - h / 2 + ry)
    }
    pub fn cell_at(&self, x: i32, y: i32, facing: u8, px: i32, py: i32) -> Option<usize> {
        let (dx, dy) = self.to_local(x, y, facing, px, py);
        self.local_cell(dx, dy)
    }
    /// A point in the arena in the shape's own grid (north up), for the shape centred on (x, y)
    /// with a facing. Points outside the shape map too, just outside the grid.
    pub fn to_local(&self, x: i32, y: i32, facing: u8, px: i32, py: i32) -> (i32, i32) {
        let (w, h) = self.dims(facing);
        let (rx, ry) = (px - (x - w / 2), py - (y - h / 2));
        match facing & 3 {
            0 => (rx, ry),
            1 => (ry, self.h - 1 - rx),
            2 => (self.w - 1 - rx, self.h - 1 - ry),
            _ => (self.w - 1 - ry, rx),
        }
    }
    /// The cell at a point of the shape's own grid, if there's a part there.
    pub fn local_cell(&self, dx: i32, dy: i32) -> Option<usize> {
        if dx < 0 || dy < 0 || dx >= self.w || dy >= self.h {
            return None;
        }
        let i = (dy * self.w + dx) as usize;
        self.cells[i].map(|_| i)
    }
    pub fn is_core(&self, i: usize) -> bool {
        self.cells[i] == Some(Part::Core)
    }
}

/// What's wrong with a design, if anything.
#[derive(Clone, Debug, Default)]
pub struct Check {
    pub cores: i32,
    /// Cells not joined to the cores: the whole drawing must be one piece.
    pub loose: Vec<(i32, i32)>,
    /// Engines with no straight 3-wide way out of the ship for their exhaust.
    pub buried: Vec<(i32, i32)>,
    /// A seeded design whose seed cell (the grid's centre) isn't a core.
    pub no_seed: bool,
    /// Projector pixels that are too thin or hemmed in (`hemmed_projectors`).
    pub hemmed: Vec<(i32, i32)>,
    /// How many of those are too thin. The rest are only hemmed in.
    pub thin: usize,
}

impl Check {
    pub fn ok(&self) -> bool {
        self.cores >= 1 && self.loose.is_empty() && self.buried.is_empty() && self.hemmed.is_empty() && !self.no_seed
    }
    pub fn problem(&self) -> Option<String> {
        let s = |n: usize| if n == 1 { "" } else { "S" };
        if self.cores == 0 {
            Some("NO CORE: IT CAN'T FLY".into())
        } else if self.no_seed {
            Some("NO CORE ON THE SEED".into())
        } else if !self.loose.is_empty() {
            let n = self.loose.len();
            Some(format!("{n} LOOSE PIXEL{}", s(n)))
        } else if !self.buried.is_empty() {
            let n = self.buried.len();
            Some(format!("{n} ENGINE{}: NO 3-WIDE EXIT", s(n)))
        } else if !self.hemmed.is_empty() {
            // The thin ones first: thickening a lump is what most often opens it up too.
            if self.thin > 0 {
                Some(format!("{} PROJECTOR PX TOO THIN", self.thin))
            } else {
                Some(format!("{} PROJECTOR PX SHUT IN", self.hemmed.len()))
            }
        } else {
            None
        }
    }
}

/// Engines whose exhaust has no straight way out. Like a rook's move: in one of the four
/// directions, the engine's own column (or row) and the ones either side of it must be empty all
/// the way out past the edge of the ship's bounding box. So there's always a straight line of
/// fire to every engine; plate can't fold round it. `filled` says whether a cell holds a pixel
/// (anything outside the drawing is empty).
pub fn buried_engines(engines: &[(i32, i32)], bbox: (i32, i32, i32, i32), filled: impl Fn(i32, i32) -> bool) -> Vec<(i32, i32)> {
    let (x0, y0, x1, y1) = bbox;
    let clear = |(ex, ey): (i32, i32), (dx, dy): (i32, i32)| {
        // Sideways from the line of the exhaust.
        let (sx, sy) = (dy.abs(), dx.abs());
        (1..)
            .map(|k| (ex + dx * k, ey + dy * k))
            .take_while(|&(x, y)| (x0..=x1).contains(&x) && (y0..=y1).contains(&y))
            .all(|(x, y)| (-1..=1).all(|s| !filled(x + sx * s, y + sy * s)))
    };
    engines.iter().copied().filter(|&e| ![(0, 1), (0, -1), (1, 0), (-1, 0)].iter().any(|&d| clear(e, d))).collect()
}

#[derive(Clone, Debug, Hash)]
pub struct Design {
    pub name: String,
    /// GRID x GRID, row-major, north up.
    pub cells: Vec<Option<Part>>,
    /// Grown from a seed on a rock (match only), rather than printed by a factory or the king.
    /// Its seed is a core at the grid's centre, which the yard keeps there.
    pub seeded: bool,
    /// What its factory pixels print, by deck slot, round and round (match only).
    pub build: Vec<usize>,
}

/// Where a seeded design's seed core sits on the grid.
pub const SEED_CELL: (i32, i32) = (GRID / 2, GRID / 2);
/// The longest build order the yard allows.
pub const BUILD_MAX: usize = 12;
/// How many designs a team holds (Henry, 2026-11-12: "Nine for now"). The yard's tabs, the
/// arena's roster and the match's cards are all laid out for this many.
pub const SLOTS: usize = 9;

/// A team from a list of designs: the first `SLOTS` of them, and empty slots after.
pub fn team_of(mut designs: Vec<Design>) -> Vec<Design> {
    designs.truncate(SLOTS);
    while designs.len() < SLOTS {
        designs.push(Design::blank(&format!("DESIGN {}", designs.len() + 1)));
    }
    designs
}

impl Design {
    pub fn blank(name: &str) -> Design {
        Design { name: name.into(), cells: vec![None; (GRID * GRID) as usize], seeded: false, build: Vec::new() }
    }

    /// Nothing drawn yet: an empty slot.
    pub fn is_empty(&self) -> bool {
        self.cells.iter().all(|c| c.is_none())
    }

    /// A design from art rows, centred in the grid.
    pub fn from_rows(name: &str, rows: &[&str]) -> Design {
        let mut d = Design::blank(name);
        let h = rows.len() as i32;
        let w = rows.iter().map(|r| r.len()).max().unwrap_or(0) as i32;
        let (ox, oy) = ((GRID - w) / 2, (GRID - h) / 2);
        for (y, r) in rows.iter().enumerate() {
            for (x, c) in r.chars().enumerate() {
                d.set(ox + x as i32, oy + y as i32, Part::from_ch(c));
            }
        }
        d
    }

    pub fn get(&self, x: i32, y: i32) -> Option<Part> {
        if (0..GRID).contains(&x) && (0..GRID).contains(&y) {
            self.cells[(y * GRID + x) as usize]
        } else {
            None
        }
    }

    /// Sets one cell with no regard for blocks; the editor uses `put`.
    pub fn set(&mut self, x: i32, y: i32, p: Option<Part>) {
        if (0..GRID).contains(&x) && (0..GRID).contains(&y) {
            self.cells[(y * GRID + x) as usize] = p;
        }
    }

    /// The top-left corner of every whole block of part `p` (a 3x3 turret, or a 2x2 slab of heavy
    /// plate), and any of its cells that don't make a whole block. Blocks never overlap (the
    /// editor sees to that), so the first unclaimed cell in reading order is always some block's
    /// top-left corner.
    pub fn blocks(&self, p: Part) -> (Vec<(i32, i32)>, Vec<(i32, i32)>) {
        let n = p.side();
        let mut claimed = vec![false; self.cells.len()];
        let (mut blocks, mut stray) = (Vec::new(), Vec::new());
        for y in 0..GRID {
            for x in 0..GRID {
                let i = (y * GRID + x) as usize;
                if claimed[i] || self.cells[i] != Some(p) {
                    continue;
                }
                let whole = (0..n).all(|dy| (0..n).all(|dx| self.get(x + dx, y + dy) == Some(p) && !claimed[((y + dy) * GRID + x + dx) as usize]));
                if whole {
                    for dy in 0..n {
                        for dx in 0..n {
                            claimed[((y + dy) * GRID + x + dx) as usize] = true;
                        }
                    }
                    blocks.push((x, y));
                } else {
                    claimed[i] = true;
                    stray.push((x, y));
                }
            }
        }
        (blocks, stray)
    }

    /// The cannon blocks (`blocks` for the cannon).
    #[cfg(test)]
    pub fn cannon_blocks(&self) -> (Vec<(i32, i32)>, Vec<(i32, i32)>) {
        self.blocks(Part::Cannon)
    }

    /// Every whole block of every part that comes in blocks, with its top-left corner.
    pub fn all_blocks(&self) -> Vec<(Part, (i32, i32))> {
        BLOCK_PARTS.iter().flat_map(|&p| self.blocks(p).0.into_iter().map(move |b| (p, b))).collect()
    }

    /// The centre of each turret, cannons first: where its breech is.
    pub fn breeches(&self) -> Vec<(Part, (i32, i32))> {
        [Part::Cannon, Part::Tracker].iter().flat_map(|&p| self.blocks(p).0.into_iter().map(move |(x, y)| (p, (x + 1, y + 1)))).collect()
    }

    /// The block covering a cell, if any: its part and its top-left corner.
    pub fn block_at(&self, x: i32, y: i32) -> Option<(Part, (i32, i32))> {
        self.all_blocks().into_iter().find(|&(p, (bx, by))| (bx..bx + p.side()).contains(&x) && (by..by + p.side()).contains(&y))
    }

    /// Heavy plate's dominoes: its pixels in pairs side by side, two wide or two tall, as many
    /// pairs as the drawing can be cut into, and the heavy pixels left over (loose). Any cut
    /// that leaves nothing over will do, since the pixels are the same whichever way it's cut, so
    /// this finds the most pairs there can be: a first pass in reading order, then each pixel
    /// left over on a chessboard's dark squares passes pairs along until one frees up (an
    /// augmenting path).
    pub fn dominoes(&self) -> (Vec<((i32, i32), (i32, i32))>, Vec<(i32, i32)>) {
        const NONE: usize = usize::MAX;
        let cells: Vec<(i32, i32)> = (0..GRID * GRID).map(|i| (i % GRID, i / GRID)).filter(|&(x, y)| self.get(x, y) == Some(Part::HeavyPlate)).collect();
        let mut id = vec![NONE; (GRID * GRID) as usize];
        for (k, &(x, y)) in cells.iter().enumerate() {
            id[(y * GRID + x) as usize] = k;
        }
        let near = |k: usize| -> Vec<usize> {
            let (x, y) = cells[k];
            [(1, 0), (0, 1), (-1, 0), (0, -1)]
                .iter()
                .map(|&(dx, dy)| (x + dx, y + dy))
                .filter(|&(nx, ny)| (0..GRID).contains(&nx) && (0..GRID).contains(&ny))
                .map(|(nx, ny)| id[(ny * GRID + nx) as usize])
                .filter(|&j| j != NONE)
                .collect()
        };
        let mut mate = vec![NONE; cells.len()];
        for k in 0..cells.len() {
            if mate[k] == NONE {
                if let Some(j) = near(k).into_iter().find(|&j| mate[j] == NONE) {
                    (mate[k], mate[j]) = (j, k);
                }
            }
        }
        for s in 0..cells.len() {
            if mate[s] != NONE || (cells[s].0 + cells[s].1) % 2 != 0 {
                continue;
            }
            // Breadth first from s: a light square is reached from a dark one, and goes on to
            // the dark one it's paired with, until a light square that's free turns up.
            let mut from = vec![NONE; cells.len()];
            let mut queued = vec![false; cells.len()];
            let mut queue = std::collections::VecDeque::from([s]);
            queued[s] = true;
            let mut free = NONE;
            'search: while let Some(b) = queue.pop_front() {
                for v in near(b) {
                    if from[v] != NONE {
                        continue;
                    }
                    from[v] = b;
                    if mate[v] == NONE {
                        free = v;
                        break 'search;
                    }
                    if !queued[mate[v]] {
                        queued[mate[v]] = true;
                        queue.push_back(mate[v]);
                    }
                }
            }
            // Pass the pairs along the path, back to s.
            let mut v = free;
            while v != NONE {
                let b = from[v];
                let next = mate[b];
                (mate[b], mate[v]) = (v, b);
                v = next;
            }
        }
        let pairs = (0..cells.len()).filter(|&k| mate[k] != NONE && mate[k] > k).map(|k| (cells[k], cells[mate[k]])).collect();
        let loose = (0..cells.len()).filter(|&k| mate[k] == NONE).map(|k| cells[k]).collect();
        (pairs, loose)
    }

    /// The domino a heavy pixel is half of, if it's paired.
    pub fn domino_at(&self, x: i32, y: i32) -> Option<((i32, i32), (i32, i32))> {
        self.dominoes().0.into_iter().find(|&(a, b)| a == (x, y) || b == (x, y))
    }

    /// Empties a cell, and the rest of the block or domino it's part of.
    pub fn clear_at(&mut self, (x, y): (i32, i32)) {
        match self.get(x, y) {
            Some(Part::HeavyPlate) => match self.domino_at(x, y) {
                Some((a, b)) => {
                    self.set(a.0, a.1, None);
                    self.set(b.0, b.1, None);
                }
                None => self.set(x, y, None),
            },
            Some(q) if q.side() > 1 => match self.block_at(x, y) {
                Some((bp, b)) => self.clear_block(bp, b),
                None => self.set(x, y, None),
            },
            _ => self.set(x, y, None),
        }
    }

    /// The editor's paint for heavy plate: a domino with its top-left on the cell, two wide, or
    /// two tall when `upright`. Whatever it lands on goes first, whole if it's a block or another
    /// domino. Returns whether anything changed.
    pub fn put_domino(&mut self, x: i32, y: i32, upright: bool) -> bool {
        let (x1, y1) = if upright { (x, y + 1) } else { (x + 1, y) };
        if x < 0 || y < 0 || x1 >= GRID || y1 >= GRID {
            return false;
        }
        if self.domino_at(x, y).is_some_and(|(a, b)| [a, b] == [(x, y), (x1, y1)] || [b, a] == [(x, y), (x1, y1)]) {
            return false;
        }
        for c in [(x, y), (x1, y1)] {
            self.clear_at(c);
        }
        self.set(x, y, Some(Part::HeavyPlate));
        self.set(x1, y1, Some(Part::HeavyPlate));
        true
    }

    fn clear_block(&mut self, p: Part, (bx, by): (i32, i32)) {
        for dy in 0..p.side() {
            for dx in 0..p.side() {
                self.set(bx + dx, by + dy, None);
            }
        }
    }

    /// The editor's paint: a block's or a domino's cell is only ever removed with the whole of
    /// it, and a turret stamps a whole 3x3 block centred on the cell (refused if it doesn't fit).
    /// Heavy plate puts down a domino two wide (`put_domino` turns it). Returns whether anything
    /// changed.
    pub fn put(&mut self, x: i32, y: i32, p: Option<Part>) -> bool {
        if !(0..GRID).contains(&x) || !(0..GRID).contains(&y) {
            return false;
        }
        if p == Some(Part::HeavyPlate) {
            return self.put_domino(x, y, false);
        }
        if let Some(q) = p.filter(|q| q.side() > 1) {
            let n = q.side();
            let (x0, y0) = (x - (n - 1) / 2, y - (n - 1) / 2);
            if x0 < 0 || y0 < 0 || x0 + n > GRID || y0 + n > GRID {
                return false;
            }
            // A block the new one lands on goes whole first, so no scraps are left.
            for (bp, (bx, by)) in self.all_blocks() {
                let m = bp.side();
                if bx < x0 + n && x0 < bx + m && by < y0 + n && y0 < by + m {
                    self.clear_block(bp, (bx, by));
                }
            }
            for dy in 0..n {
                for dx in 0..n {
                    if self.get(x0 + dx, y0 + dy) == Some(Part::HeavyPlate) {
                        self.clear_at((x0 + dx, y0 + dy));
                    }
                }
            }
            for dy in 0..n {
                for dx in 0..n {
                    self.set(x0 + dx, y0 + dy, Some(q));
                }
            }
            return true;
        }
        if self.get(x, y) == p {
            return false;
        }
        if self.get(x, y).is_some_and(|q| q.side() > 1 || q == Part::HeavyPlate) {
            self.clear_at((x, y));
        }
        self.set(x, y, p);
        true
    }

    /// A design is legal with at least one core, every pixel joined into one piece edge to edge
    /// (so all its cores are joined too), whole blocks, heavy plate in whole dominoes, and no
    /// buried engines.
    pub fn check(&self) -> Check {
        let cores: Vec<usize> = (0..self.cells.len()).filter(|&i| self.cells[i] == Some(Part::Core)).collect();
        let mut c = Check { cores: cores.len() as i32, no_seed: self.seeded && self.get(SEED_CELL.0, SEED_CELL.1) != Some(Part::Core), ..Default::default() };
        // The ship is the biggest piece with a core in it; anything else is loose.
        let ship = cores.iter().map(|&k| reach(GRID, GRID, k, |i| self.cells[i].is_some())).max_by_key(|seen| seen.iter().filter(|&&b| b).count());
        if let Some(seen) = ship {
            c.loose = (0..self.cells.len()).filter(|&i| self.cells[i].is_some() && !seen[i]).map(|i| (i as i32 % GRID, i as i32 / GRID)).collect();
        }
        for p in BLOCK_PARTS {
            c.loose.extend(self.blocks(p).1);
        }
        c.loose.extend(self.dominoes().1);
        if let Some(bbox) = self.used_box() {
            let (x0, y0, x1, y1) = bbox;
            let engines: Vec<(i32, i32)> = (y0..=y1).flat_map(|y| (x0..=x1).map(move |x| (x, y))).filter(|&(x, y)| self.get(x, y) == Some(Part::Engine)).collect();
            c.buried = buried_engines(&engines, bbox, |x, y| self.get(x, y).is_some());
            (c.hemmed, c.thin) = hemmed_projectors(bbox, |x, y| self.get(x, y));
        }
        c
    }

    pub fn stats(&self) -> Stats {
        let longest = |p: Part| beam_guns(GRID, GRID, |i| self.cells[i] == Some(p)).iter().map(|g| g.1).max().unwrap_or(0);
        stats_of(&self.cells, self.blocks(Part::Cannon).0.len() as i32, self.blocks(Part::Tracker).0.len() as i32, longest(Part::Beam), longest(Part::Repeater))
    }

    /// What its ships do, in a word or two, for a list. It follows from the drawing.
    pub fn does(&self) -> &'static str {
        let st = self.stats();
        match st.order() {
            Order::Harvest => "MINES",
            Order::Hold => "STAYS PUT",
            _ if st.armed() => "FIGHTS",
            _ => "FLIES, NO GUNS",
        }
    }

    /// How it flies at what it's fighting until its side has thought about it: what its
    /// drawing suggests. None: it has no engines, and stays put. (A design carried a way of
    /// flying chosen in the yard, and a group size, until 2026-11-14. Now ships choose.)
    #[cfg(test)]
    pub fn flies(&self) -> Option<Style> {
        let st = self.stats();
        if st.thrust == 0 {
            None
        } else {
            Style::of(st.flight)
        }
    }

    /// The box round everything drawn, as (left, top, right, bottom), inclusive.
    pub fn used_box(&self) -> Option<(i32, i32, i32, i32)> {
        let mut b: Option<(i32, i32, i32, i32)> = None;
        for i in 0..self.cells.len() {
            if self.cells[i].is_some() {
                let (x, y) = (i as i32 % GRID, i as i32 / GRID);
                b = Some(match b {
                    None => (x, y, x, y),
                    Some((l, t, r, bt)) => (l.min(x), t.min(y), r.max(x), bt.max(y)),
                });
            }
        }
        b
    }

    pub fn shape(&self) -> Shape {
        let Some((x0, y0, x1, y1)) = self.used_box() else {
            return Shape { w: 1, h: 1, cells: vec![None], turrets: Vec::new() };
        };
        let (w, h) = (x1 - x0 + 1, y1 - y0 + 1);
        let cells = (0..w * h).map(|i| self.get(x0 + i % w, y0 + i / w)).collect();
        let turrets = self.breeches().iter().map(|&(_, (bx, by))| ((by - y0) * w + bx - x0) as usize).collect();
        Shape { w, h, cells, turrets }
    }

    /// The whole drawing as a stamp.
    pub fn as_stamp(&self) -> Stamp {
        let sh = self.shape();
        Stamp { w: sh.w, h: sh.h, cells: sh.cells }
    }

    /// The cells in a box (corners inclusive, either order) as a stamp, trimmed to what's drawn.
    /// A block or domino the box cuts through is left out, so a stamp never holds part of one.
    pub fn copy_box(&self, a: (i32, i32), b: (i32, i32)) -> Stamp {
        let (x0, x1, y0, y1) = (a.0.min(b.0), a.0.max(b.0), a.1.min(b.1), a.1.max(b.1));
        let inside = |x: i32, y: i32| (x0..=x1).contains(&x) && (y0..=y1).contains(&y);
        let blocks = self.all_blocks();
        let pairs = self.dominoes().0;
        let keep = |x: i32, y: i32| match self.get(x, y) {
            Some(Part::HeavyPlate) => pairs.iter().any(|&(a, b)| (a == (x, y) || b == (x, y)) && inside(a.0, a.1) && inside(b.0, b.1)),
            Some(q) if q.side() > 1 => {
                let n = q.side();
                blocks.iter().any(|&(bp, (bx, by))| bp == q && (bx..bx + n).contains(&x) && (by..by + n).contains(&y) && inside(bx, by) && inside(bx + n - 1, by + n - 1))
            }
            p => p.is_some(),
        };
        let used: Vec<(i32, i32)> = (y0..=y1).flat_map(|y| (x0..=x1).map(move |x| (x, y))).filter(|&(x, y)| keep(x, y)).collect();
        if used.is_empty() {
            return Stamp { w: 1, h: 1, cells: vec![None] };
        }
        let (l, r) = (used.iter().map(|p| p.0).min().unwrap(), used.iter().map(|p| p.0).max().unwrap());
        let (t, bt) = (used.iter().map(|p| p.1).min().unwrap(), used.iter().map(|p| p.1).max().unwrap());
        let (w, h) = (r - l + 1, bt - t + 1);
        let cells = (0..w * h)
            .map(|i| {
                let (x, y) = (l + i % w, t + i / w);
                if keep(x, y) {
                    self.get(x, y)
                } else {
                    None
                }
            })
            .collect();
        Stamp { w, h, cells }
    }

    /// Writes a stamp's pixels with its top-left at (x, y); its empty cells leave what's there,
    /// and anything off the grid is dropped. A block or domino the stamp lands on is removed
    /// whole first, so no scraps are left. Returns whether anything changed.
    pub fn paste(&mut self, st: &Stamp, (x, y): (i32, i32)) -> bool {
        let mut changed = false;
        for (a, b) in self.dominoes().0 {
            if st.get(a.0 - x, a.1 - y).is_some() || st.get(b.0 - x, b.1 - y).is_some() {
                self.set(a.0, a.1, None);
                self.set(b.0, b.1, None);
                changed = true;
            }
        }
        for (bp, (bx, by)) in self.all_blocks() {
            let n = bp.side();
            if (0..n * n).any(|k| st.get(bx + k % n - x, by + k / n - y).is_some()) {
                self.clear_block(bp, (bx, by));
                changed = true;
            }
        }
        for sy in 0..st.h {
            for sx in 0..st.w {
                let (gx, gy) = (x + sx, y + sy);
                if let Some(p) = st.get(sx, sy) {
                    if (0..GRID).contains(&gx) && (0..GRID).contains(&gy) && self.get(gx, gy) != Some(p) {
                        self.set(gx, gy, Some(p));
                        changed = true;
                    }
                }
            }
        }
        changed
    }

    /// Fills a box (corners inclusive, either order) with a part, or empties it. A part that comes
    /// in blocks tiles the box in whole ones from its top-left corner, leaving any strip too thin
    /// for one; heavy plate fills a box with an even number of cells, leaving out the right-hand
    /// column (or, one column wide, the bottom row) of an odd one. A block or domino the box
    /// touches is removed whole. Returns whether anything changed.
    pub fn fill_box(&mut self, a: (i32, i32), b: (i32, i32), p: Option<Part>) -> bool {
        let (x0, x1) = (a.0.min(b.0).max(0), a.0.max(b.0).min(GRID - 1));
        let (y0, y1) = (a.1.min(b.1).max(0), a.1.max(b.1).min(GRID - 1));
        if x0 > x1 || y0 > y1 {
            return false;
        }
        let (mut w, mut h) = (x1 - x0 + 1, y1 - y0 + 1);
        if let Some(n) = p.map(|q| q.side()).filter(|&n| n > 1) {
            (w, h) = (w / n * n, h / n * n);
            if w == 0 || h == 0 {
                return false;
            }
        }
        if p == Some(Part::HeavyPlate) && w * h % 2 == 1 {
            if w > 1 {
                w -= 1;
            } else if h > 1 {
                h -= 1;
            } else {
                return false;
            }
        }
        match p {
            Some(_) => self.paste(&Stamp { w, h, cells: vec![p; (w * h) as usize] }, (x0, y0)),
            None => {
                let mut changed = false;
                for (bp, (bx, by)) in self.all_blocks() {
                    let n = bp.side();
                    if bx + n - 1 >= x0 && bx <= x1 && by + n - 1 >= y0 && by <= y1 {
                        self.clear_block(bp, (bx, by));
                        changed = true;
                    }
                }
                let inside = |(x, y): (i32, i32)| (x0..=x1).contains(&x) && (y0..=y1).contains(&y);
                for (a, b) in self.dominoes().0 {
                    if inside(a) || inside(b) {
                        self.set(a.0, a.1, None);
                        self.set(b.0, b.1, None);
                        changed = true;
                    }
                }
                for y in y0..=y1 {
                    for x in x0..=x1 {
                        changed |= self.get(x, y).is_some();
                        self.set(x, y, None);
                    }
                }
                changed
            }
        }
    }

    /// Flood-fills the patch joined edge to edge that matches the cell at (x, y): an empty hole,
    /// or a patch of one part, which this swaps for another part or erases. An empty area that
    /// reaches the edge of the grid isn't a hole, so it's refused, and so are turrets (the
    /// rectangle tiles them), and heavy plate where what it fills can't be cut into dominoes.
    /// Returns whether anything changed.
    pub fn flood(&mut self, (x, y): (i32, i32), p: Option<Part>) -> Result<bool, &'static str> {
        if !(0..GRID).contains(&x) || !(0..GRID).contains(&y) {
            return Ok(false);
        }
        match p {
            Some(Part::Cannon) => return Err("CANNONS DON'T FILL: USE THE RECTANGLE"),
            Some(Part::Tracker) => return Err("TRACKERS DON'T FILL: USE THE RECTANGLE"),
            _ => {}
        }
        if p == Some(Part::HeavyPlate) {
            let before = self.cells.clone();
            let filled = self.flood_patch((x, y), p)?;
            if !self.dominoes().1.is_empty() {
                self.cells = before;
                return Err("HEAVY PLATE COMES IN DOMINOES: THAT AREA DOESN'T CUT INTO THEM");
            }
            return Ok(filled);
        }
        self.flood_patch((x, y), p)
    }

    /// `flood` without the rules about blocks and dominoes.
    fn flood_patch(&mut self, (x, y): (i32, i32), p: Option<Part>) -> Result<bool, &'static str> {
        let was = self.get(x, y);
        if was == p {
            return Ok(false);
        }
        let seen = reach(GRID, GRID, (y * GRID + x) as usize, |i| self.cells[i] == was);
        let patch: Vec<usize> = (0..seen.len()).filter(|&i| seen[i]).collect();
        if was.is_none() && patch.iter().any(|&i| (i as i32 % GRID) % (GRID - 1) == 0 || (i as i32 / GRID) % (GRID - 1) == 0) {
            return Err("THAT SPACE ISN'T CLOSED IN: FILL A HOLE INSIDE THE SHIP");
        }
        for i in patch {
            self.cells[i] = p;
        }
        Ok(true)
    }

    /// One line for the save file: `NAME|X,Y|row/row/...|KIND|BUILD`. The drawing is trimmed to
    /// its box with the box's top-left on the grid and `.` for empty cells; KIND is BUILT or
    /// SEEDED, and BUILD the build order as deck slots. A design carries nothing else: what its
    /// ships do follows from the drawing (`Design::order`). Lines saved before 2026-11-14 have
    /// the order a design used to carry as their second field, and may end `|STYLE|GROUP`, the
    /// way of flying and the group size it carried too: all three are read past.
    pub fn to_line(&self) -> String {
        let kind = if self.seeded { "SEEDED" } else { "BUILT" };
        let build: Vec<String> = self.build.iter().map(|b| b.to_string()).collect();
        let Some((x0, y0, x1, y1)) = self.used_box() else {
            return format!("{}|0,0|.|{kind}|{}", self.name, build.join(","));
        };
        let rows: Vec<String> = (y0..=y1).map(|y| (x0..=x1).map(|x| self.get(x, y).map_or('.', |p| p.ch())).collect()).collect();
        format!("{}|{x0},{y0}|{}|{kind}|{}", self.name, rows.join("/"), build.join(","))
    }

    /// Reads a save line, today's or an older one. A round 2 or 3 line (the whole 41x41 grid,
    /// no position) is centred.
    pub fn from_line(s: &str) -> Option<Design> {
        let mut f: Vec<&str> = s.trim().split('|').collect();
        let name = *f.first()?;
        // An older line has the design's order second: read past it. Without one, the line is
        // today's, and has all five fields.
        let old = f.get(1).is_some_and(|o| Order::from_name(o).is_some());
        if old {
            f.remove(1);
        } else if f.len() != 5 {
            return None;
        }
        // Now: NAME, then X,Y and the rows (or the rows alone), then KIND and BUILD, then on
        // some older lines the way of flying and the group size.
        let (at, rows) = match f.len() {
            3 | 5 | 7 => {
                let (x, y) = f[1].split_once(',')?;
                ((x.parse::<i32>().ok()?, y.parse::<i32>().ok()?), f[2])
            }
            2 => {
                let n = f[1].split('/').count() as i32;
                if n > GRID {
                    return None;
                }
                (((GRID - n) / 2, (GRID - n) / 2), f[1])
            }
            _ => return None,
        };
        // A line may come from another machine or a clipboard: a corner off the grid isn't a
        // drawing.
        if !(0..GRID).contains(&at.0) || !(0..GRID).contains(&at.1) {
            return None;
        }
        let mut d = Design::blank(name);
        for (y, r) in rows.split('/').enumerate() {
            for (x, c) in r.chars().enumerate() {
                d.set(at.0 + x as i32, at.1 + y as i32, Part::from_ch(c));
            }
        }
        if f.len() >= 5 {
            d.seeded = f[3] == "SEEDED";
            d.build = f[4].split(',').filter_map(|b| b.trim().parse().ok()).take(BUILD_MAX).collect();
        }
        Some(d)
    }

    /// The shape's cells in the order they grow: outward from `start` edge to edge, so what's
    /// grown is one connected piece at every step. Ties go in reading order.
    pub fn growth_order(sh: &Shape, start: usize) -> Vec<usize> {
        let (w, h) = (sh.w, sh.h);
        let mut seen = vec![false; sh.cells.len()];
        let mut order = Vec::new();
        let mut queue = std::collections::VecDeque::new();
        if sh.cells.get(start).copied().flatten().is_some() {
            seen[start] = true;
            queue.push_back(start);
        }
        while let Some(c) = queue.pop_front() {
            order.push(c);
            let (x, y) = (c as i32 % w, c as i32 / w);
            for (dx, dy) in [(0, -1), (-1, 0), (1, 0), (0, 1)] {
                let (nx, ny) = (x + dx, y + dy);
                if nx >= 0 && ny >= 0 && nx < w && ny < h {
                    let k = (ny * w + nx) as usize;
                    if !seen[k] && sh.cells[k].is_some() {
                        seen[k] = true;
                        queue.push_back(k);
                    }
                }
            }
        }
        order
    }

    /// The shape cell where a seeded design's seed core sits, or the first core of any other.
    pub fn start_cell(&self) -> Option<usize> {
        let (x0, y0, _, _) = self.used_box()?;
        let sh = self.shape();
        if self.seeded {
            let c = ((SEED_CELL.1 - y0) * sh.w + SEED_CELL.0 - x0) as usize;
            return sh.is_core(c).then_some(c);
        }
        (0..sh.cells.len()).find(|&c| sh.is_core(c))
    }
}

/// A piece of a drawing picked up to place elsewhere: a whole design, or a box copied off the
/// grid. Placing it writes ordinary pixels; nothing stays linked to where it came from.
#[derive(Clone, Debug, PartialEq)]
pub struct Stamp {
    pub w: i32,
    pub h: i32,
    pub cells: Vec<Option<Part>>,
}

impl Stamp {
    pub fn get(&self, x: i32, y: i32) -> Option<Part> {
        if (0..self.w).contains(&x) && (0..self.h).contains(&y) {
            self.cells[(y * self.w + x) as usize]
        } else {
            None
        }
    }
    /// A quarter turn clockwise.
    pub fn turned(&self) -> Stamp {
        let (w, h) = (self.h, self.w);
        let cells = (0..w * h).map(|i| self.get(i / w, self.h - 1 - i % w)).collect();
        Stamp { w, h, cells }
    }
    /// Mirrored left to right.
    pub fn flipped(&self) -> Stamp {
        let cells = (0..self.w * self.h).map(|i| self.get(self.w - 1 - i % self.w, i / self.w)).collect();
        Stamp { w: self.w, h: self.h, cells }
    }
    pub fn is_empty(&self) -> bool {
        self.cells.iter().all(|c| c.is_none())
    }
    /// Where the top-left goes for the stamp to sit centred on a cell.
    pub fn origin(&self, (cx, cy): (i32, i32)) -> (i32, i32) {
        (cx - self.w / 2, cy - self.h / 2)
    }
}

/// The starting library. Slots are referred to by index in the test setups.
pub fn defaults() -> Vec<Design> {
    vec![
        Design::from_rows("FIGHTER", &["g.g", "hch", "e.e"]),
        // A hull pixel ahead of the lance soaks the first hit (own shots pass through own pixels).
        Design::from_rows("LANCER", &[".h.", ".l.", "hch", "e.e"]),
        // Flak and lance tucked inside plate (a ship's own shots pass through its own pixels);
        // only the engines are bare.
        Design::from_rows("FRIGATE", &["...ppp...", "..pplpp..", ".pphhhpp.", ".pghchgp.", ".pphhhpp.", ".pghhhgp.", "..ppppp..", "..e.e.e.."]),
        // One cannon behind a plated prow; its flak sponsons are bolted on outside the plate.
        Design::from_rows(
            "DESTROYER",
            &[
                ".....ppp.....",
                "....pphpp....",
                "...pphhhpp...",
                "..pphhhhhpp..",
                ".pphhnnnhhpp.",
                ".phhhnnnhhhp.",
                ".phhhnnnhhhp.",
                ".phhhhhhhhhp.",
                ".phhhhhhhhhp.",
                "gphhhhhhhhhpg",
                ".phhhhhhhhhp.",
                ".phhhhchhhhp.",
                ".phhhhhhhhhp.",
                "gphhhhhhhhhpg",
                ".phhhhhhhhhp.",
                "..phhhhhhhp..",
                "...ppppppp...",
                "..eeee.eeee..",
            ],
        ),
        // Sealed in plate all round: fighters can only strip its sponsons and engines. Three cores
        // (fore, middle, aft) for +24: cheap insurance on a ship this size.
        Design::from_rows(
            "CAPITAL",
            &[
                ".........ppp.........",
                "........pphpp........",
                ".......pphhhpp.......",
                "......pphhhhhpp......",
                ".....pphhhhhhhpp.....",
                "....pphhhhhhhhhpp....",
                "...pphhhhnnnhhhhpp...",
                "..pphhhhhnnnhhhhhpp..",
                ".pphhhhhhnnnhhhhhhpp.",
                ".phhhhhhhhhhhhhhhhhp.",
                ".phhhhhhhhchhhhhhhhp.",
                "gphhhhhhghhhghhhhhhpg",
                ".phhhhhhhhhhhhhhhhhp.",
                ".phhhhhhhhhhhhhhhhhp.",
                ".phhhnnnhhhhhnnnhhhp.",
                ".phhhnnnhhhhhnnnhhhp.",
                ".phhhnnnhhhhhnnnhhhp.",
                ".phhhhhhhhhhhhhhhhhp.",
                ".phhhhhhhhchhhhhhhhp.",
                "gphhhhhhhhhhhhhhhhhpg",
                ".phhhhhhhhhhhhhhhhhp.",
                ".phhhhhhhnnnhhhhhhhp.",
                ".phhhhhhhnnnhhhhhhhp.",
                ".phhhhhhhnnnhhhhhhhp.",
                ".phhhhhhhhhhhhhhhhhp.",
                ".phhhhhhhhhhhhhhhhhp.",
                ".phhhhhhhhchhhhhhhhp.",
                "gphhhhhhghhhghhhhhhpg",
                ".phhhhhhhhhhhhhhhhhp.",
                ".phhhhhhhhhhhhhhhhhp.",
                ".phhhhhhhhhhhhhhhhhp.",
                ".phhhhhhghhhghhhhhhp.",
                ".phhhhhhhhhhhhhhhhhp.",
                ".phhhhhhhhhhhhhhhhhp.",
                "..phhhhhhhhhhhhhhhp..",
                "...ppppppppppppppp...",
                "...e.e.e.e.e.e.e.e...",
            ],
        ),
        Design::from_rows("GUNMINER", &["s.s", "sgs", "hch", ".e."]),
        titan(),
    ]
}

/// A 97-pixel battleship, the round 4 scale test, drawn in code: plated two pixels deep, nine
/// cannons, flak on sponsons down both sides and more behind the plate, three lances in the prow,
/// three cores down the spine (+24) and a row of engines across the stern.
fn titan() -> Design {
    let mut d = Design::blank("TITAN");
    let c = GRID / 2;
    let half = |y: i32| if y < 40 { y / 2 } else if y <= 88 { 20 } else { 20 - (y - 88) };
    for y in 0..=95 {
        for x in c - half(y)..=c + half(y) {
            d.set(x, y, Some(Part::Hull));
        }
    }
    // Two pixels of plate all round: hull touching anything that isn't hull becomes plate, twice.
    for _ in 0..2 {
        let skin: Vec<(i32, i32)> = (0..GRID * GRID)
            .map(|i| (i % GRID, i / GRID))
            .filter(|&(x, y)| d.get(x, y) == Some(Part::Hull) && [(1, 0), (-1, 0), (0, 1), (0, -1)].iter().any(|&(dx, dy)| d.get(x + dx, y + dy) != Some(Part::Hull)))
            .collect();
        for (x, y) in skin {
            d.set(x, y, Some(Part::Plate));
        }
    }
    for y in [30, 55, 80] {
        d.set(c, y, Some(Part::Core));
    }
    for (x, y) in [(c, 18), (c, 42), (c, 67), (c - 12, 48), (c + 12, 48), (c - 12, 62), (c + 12, 62), (c - 12, 76), (c + 12, 76)] {
        d.put(x, y, Some(Part::Cannon));
    }
    for (x, y) in [(c, 8), (c - 4, 24), (c + 4, 24)] {
        d.set(x, y, Some(Part::Lance));
    }
    for s in [-1, 1] {
        // Sponsons just outside the plate, and guns behind it (a ship's own shots pass through it).
        for (x, y) in [(half(20) + 1, 20), (half(32) + 1, 32)].into_iter().chain((44..=84).step_by(8).map(|y| (21, y))).chain((44..=88).step_by(6).map(|y| (16, y))) {
            d.set(c + s * x, y, Some(Part::Flak));
        }
    }
    for x in (c - 12..=c + 12).step_by(2) {
        d.set(x, 96, Some(Part::Engine));
    }
    d
}

/// The king (match only): a fixed design for now. It flies like Toy 1's (straight from the keys,
/// no inertia), prints anything in the deck, and is the root node of its network.
pub fn king() -> Design {
    Design::from_rows(
        "KING",
        &[
            "......ppppp......",
            "....pphhhhhpp....",
            "...phhhhghhhhp...",
            "..phhhhhhhhhhhp..",
            ".phhhhhhhhhhhhhp.",
            ".phhhchhhhhchhhp.",
            "gphhhhhhhhhhhhhpg",
            "phhhhhhhhhhhhhhhp",
            "phhhhhhhchhhhhhhp",
            "phhhhhhhhhhhhhhhp",
            "gphhhhhhhhhhhhhpg",
            ".phhhhhhhhhhhhhp.",
            ".phhhhhhhhhhhhhp.",
            "..phhhhhhhhhhhp..",
            "...ppphhhhhppp...",
            ".....hhhhhhh.....",
            ".....e.e.e.e.....",
        ],
    )
}

/// A seed (match only): a core and an engine, unarmed. The king prints one for a seeded design;
/// it flies to a rock and grows the design out of it.
pub fn seed() -> Design {
    Design::from_rows("SEED", &["c", "e"])
}

/// A design drawn at an exact place in the grid, as the yard saves it. `build` makes it a seeded
/// factory with that build order.
pub(super) fn drawn(name: &str, at: (i32, i32), rows: &[&str], build: Option<&[usize]>) -> Design {
    let mut d = Design::blank(name);
    for (y, r) in rows.iter().enumerate() {
        for (x, c) in r.chars().enumerate() {
            d.set(at.0 + x as i32, at.1 + y as i32, Part::from_ch(c));
        }
    }
    if let Some(b) = build {
        d.seeded = true;
        d.build = b.to_vec();
    }
    d
}

/// Three ships that show the parts of 2026-11-12 at work, for the checks and the screenshots.
/// They're in nobody's deck. Each spends every pixel of power it makes.
/// - BEAMER: a siege ship. Nine beam pixels down its spine reach 148 px and cut 36.
/// - WARDEN: an escort inside four shield circles, with three lances in a column and four flak.
/// - NEEDLE: the smallest ship a beam fits, two beam pixels on a fighter's frame: 64 px.
///
/// Then the missile's, the repeater's and the projector's, and the 2026-11-15 parts' (PICKET,
/// BULWARK, AP LANCER, IRONLANCER, BASTION), which `teams::new_part_fights` fights.
pub fn part_examples() -> Vec<Design> {
    vec![
        drawn("BEAMER", (47, 44), &["..b..", ".hbh.", ".hbh.", "hhbhh", "hhbhh", "hhbhh", "hhbhh", "phbhp", "phbhp", "hhchh", "egege"], None),
        drawn("WARDEN", (45, 45), &["....o....", "...hlh...", "..hhlhh..", ".hghlhgh.", "ohhhchhho", ".hghhhgh.", "..hhhhh..", ".eehhhee.", "....o...."], None),
        drawn("NEEDLE", (48, 47), &[".b.", "hbh", "hch", "e.e"], None),
        // Two launchers and a flak gun against other people's missiles. (Its card said KITE
        // until designs stopped carrying a way of flying: as drawn it has the thrust to make
        // attack runs, and its side decides in the fight.)
        drawn("ARCHER", (47, 47), &[".m.m.", "hhghh", "hhchh", "e.e.e"], None),
        // A ten-pixel repeater down the spine of a long ship: 140 px of reach, a shot every
        // quarter of a second.
        drawn("RAKER", (47, 43), &["..r..", "..r..", "..r..", "..r..", ".hrh.", ".hrh.", ".hrh.", ".hrh.", "hhrhh", "hhrhh", "hhchh", "ee.ee"], None),
        // A projector knob of 38 pixels on a thick neck, on the bow of a wide hull: it reaches
        // 60 px and stops 38 shots. Its base is five wide, so one shell's 3x3 hole can't cut it
        // off. It has no guns, so it stays beside the nearest friend that has (as any unarmed
        // ship on HUNT does), and its fourteen engines keep it up with the other big ships.
        drawn(
            "AEGIS",
            (42, 43),
            &[
                ".....jjjjj.....",
                "....jjjjjjj....",
                "....jjjjjjj....",
                "....jjjjjjj....",
                "....jjjjjjj....",
                ".....jjjjj.....",
                ".....hhhhh.....",
                "....hhhhhhh....",
                "hhhhhhhhhhhhhhh",
                "hhhhhhhhhhhhhhh",
                "hhhhhhhchhhhhhh",
                "hhhhhhhhhhhhhhh",
                "hhhhhhhhhhhhhhh",
                "eeeeeee.eeeeeee",
            ],
            None,
        ),
        // The parts of 2026-11-15. A tracker escort: one tracker over a broad hull of the power
        // it needs, two flak, five engines.
        drawn("PICKET", (45, 46), &["...ttt...", "...ttt...", ".hhttthh.", "ghhhchhhg", "hhhhhhhhh", "e.e.e.e.e"], None),
        // A lance boat behind a prow of three heavy slabs: the lances fire through it.
        drawn("BULWARK", (46, 46), &["aaaaaa", "aaaaaa", "hlhhlh", "hlgglh", "hhhchh", "hhhhh.", "eeeeee"], None),
        // The starter lancer with a piercer between its two lances: both pierce 3.
        drawn("AP LANCER", (48, 47), &[".l.", "hih", "hlh", "hch", "e.e"], None),
        // The starter lancer behind plate, the swarm's answer to a tracker.
        drawn("IRONLANCER", (48, 47), &[".p.", "plp", "hlh", "hch", "e.e"], None),
        // All three on one big ship, redrawn (Henry, 2026-11-15: "redraw the bastion") after the
        // first, with heavy plate on its prow only, lost to lancers like the battleship: now a
        // skin of heavy plate a pixel thick all round, with everything inside it. Its guns fire
        // through it: a tracker on each flank, four lances with piercers between them, four flak.
        // Two cores, apart, so a cut down the middle leaves both halves flying. The engines sit
        // outside the skin, along the stern and two on each flank, the only thing a plain lance
        // or flak can break.
        drawn(
            "BASTION",
            (42, 43),
            &[
                ".aaaaaaaaaaaa.",
                ".ahhhhllhhhha.",
                ".attthiihttta.",
                ".attthllhttta.",
                "eattthhhhtttae",
                "eaghhhhhhhhgae",
                ".ahhhhhhhhhha.",
                ".ahhchhhhchha.",
                ".aghhhhhhhhga.",
                ".ahhhhhhhhhha.",
                ".aaaaaaaaaaaa.",
                ".eeeeeeeeeeee.",
            ],
            None,
        ),
    ]
}

/// The deck the AI plays, and the one you can borrow before you've drawn factories of your own.
/// Slots: 0 fighter, 1 lancer, 2 frigate, 3 miner, 4 outpost, 5 shipyard, 6 capital.
///
/// Henry's drawings (2026-11-10): the round 2 cards with every pixel of power put to use. None
/// carries hull that feeds nothing, so what a match shows is the rules and not a wasteful
/// drawing. The warships are drawn on GUARD: in a match they gather where they're built until
/// you send them, instead of trickling out one at a time.
pub fn starter_deck() -> Vec<Design> {
    vec![
        // Four flak round the nose: every pixel of power its hull makes is spent.
        drawn("FIGHTER", (48, 47), &[".g.", "ggg", "hch", "e.e"], None),
        // Two lances in a column: the rear one fires through the front one. Hull either side
        // of it since 2026-11-11, when a lance went from 2 power to 3.
        drawn("LANCER", (48, 47), &[".l.", "hlh", "hch", "e.e"], None),
        // Six flak and four lances down the spine, behind plate, with two more engines. Four
        // of its plates are hull now, to feed the lances.
        drawn(
            "FRIGATE",
            (45, 44),
            &[
                "....p....",
                "...php...",
                "..pplpp..",
                "phghlhghp",
                "phghchghp",
                "phhhlhhhp",
                "phghlhghp",
                "e.ppppp.e",
                "..e.e.e..",
            ],
            None,
        ),
        // Three scoops, a flak turret on the nose and three engines round the core: as fast as
        // a ship can fly, with all four of its core's power spent and no hull at all. A miner
        // bites a pixel every 2.7 s however many scoops it has past two, so the cheapest fast
        // one earns the most. Of eighteen drawings played against each other (MATCH.md, M2.14)
        // this came first; Henry's own first miner (below, with the retired cards) cost 14 px
        // and flew at a third of the speed.
        //
        // 2026-11-11: scoops use power and weigh 2, so it carries two hull pixels where it had
        // an engine, costs 9 px and flies at 30 px/s. Nobody has played drawings against each
        // other for it under these costs.
        drawn("MINER", (48, 47), &[".g.", "sss", "hch", "e.e"], None),
        // Twenty-eight factory pixels inside the plate, so it builds anything up to 280 px.
        // (Forty-two until a factory pixel went from 1 power to 2: the same size and price,
        // with a third of its factory turned to the hull the rest needs.)
        drawn(
            "OUTPOST",
            (43, 43),
            &[
                ".....ppp.....",
                "...pphhhpp...",
                "..phhhghhhp..",
                ".phhhhhhhhhp.",
                ".phhfffffhhp.",
                "phfffhhhfffhp",
                "pgfffhchfffgp",
                "phfffhhhfffhp",
                ".phhfffffhhp.",
                ".phhhhhhhhhp.",
                "..phhhghhhp..",
                "...pphhhpp...",
                ".....ppp.....",
            ],
            Some(&[3, 3, 0, 0, 1, 2]),
        ),
        // A plated block, 118 factory pixels in a ring and a heart, flak and lances on all four
        // sides. (184 in two rings until 2026-11-11: the outer ring is one pixel thick now, and
        // the rest of it is the hull that what's left needs.)
        drawn(
            "SHIPYARD",
            (36, 36),
            &[
                "...ppppppppppppppppppppp...",
                "...ppppppppppppppppppppp...",
                "...ppppphhhhlllhhhhppppp...",
                "ppppppphhhggggggghhhppppppp",
                "pppppphhhhhhhghhhhhhhpppppp",
                "ppppphhhhhhhhhhhhhhhhhppppp",
                "pppphhfffffffffffffffhhpppp",
                "ppphhhfghhhhhhhhhhhgfhhhppp",
                "pphhhhfhhhhhhhhhhhhhfhhhhpp",
                "pphhhhfhhhhhhhhhhhhhfhhhhpp",
                "pphhhhfhhfffffffffhhfhhhhpp",
                "pphhhhfhhfffffffffhhfhhhhpp",
                "pplhhhfhhfffffffffhhfhhhlpp",
                "pplghhfhhffffcffffhhfhhglpp",
                "pplhhhfhhfffffffffhhfhhhlpp",
                "pphhhhfhhfffffffffhhfhhhhpp",
                "pphhhhfhhfffffffffhhfhhhhpp",
                "pphhhhfhhhhhhhhhhhhhfhhhhpp",
                "pphhhhfhhhhhhhhhhhhhfhhhhpp",
                "ppphhhfghhhhhhhhhhhgfhhhppp",
                "pppphhfffffffffffffffhhpppp",
                "ppppphhhhhhhhhhhhhhhhhppppp",
                "pppppphhhhhhhhhhhhhhhpppppp",
                "ppppppphhggggggggghhppppppp",
                "...ppppphhhhlllhhhhppppp...",
                "...ppppppppppppppppppppp...",
                "...ppppppppppppppppppppp...",
            ],
            Some(&[3, 2, 2, 6]),
        ),
        // Twenty-seven cannons behind plate, and a full row of engines across the stern.
        drawn(
            "CAPITAL",
            (33, 31),
            &[
                "...............ppp...............",
                "..............pphpp..............",
                ".............pphhhpp.............",
                "............pphnnnhpp............",
                "...........pphhnnnhhpp...........",
                "..........pphhhnnnhhhpp..........",
                ".........ppnnnhnnnhnnnpp.........",
                "........pphnnnhnnnhnnnhpp........",
                ".......pphhnnnhnnnhnnnhhpp.......",
                ".......phhhhhhhhhhhhhhhhhp.......",
                ".......phhhhhhhhchhhhhhhhp.......",
                "......gpnnnnnnghhhgnnnnnnpg......",
                ".......pnnnnnnhhhhhnnnnnnp.......",
                ".......pnnnnnnhhhhhnnnnnnp.......",
                ".......pnnnnnnhhhhhnnnnnnp.......",
                ".......pnnnnnnhhhhhnnnnnnp.......",
                ".......pnnnnnnhhhhhnnnnnnp.......",
                ".......pnnnnnnhhhhhnnnnnnp.......",
                ".......pnnnnnnhhchhnnnnnnp.......",
                "......gpnnnnnnhhhhhnnnnnnpg......",
                ".......phhhhhhhhhhhhhhhhhp.......",
                ".......pnnnnnnhnnnhnnnnnnp.......",
                ".......pnnnnnnhnnnhnnnnnnp.......",
                ".......pnnnnnnhnnnhnnnnnnp.......",
                ".......pnnnnnnhhhhhnnnnnnp.......",
                ".......pnnnnnnhhhhhnnnnnnp.......",
                ".......pnnnnnnhhchhnnnnnnp.......",
                "......gphhhhhhghhhghhhhhhpg......",
                ".......phhhhhhhnnnhhhhhhhp.......",
                ".......phhhhhhhnnnhhhhhhhp.......",
                ".......phhhhhhhnnnhhhhhhhp.......",
                ".......phhhhhhghhhghhhhhhp.......",
                ".......phhhhhhhnnnhhhhhhhp.......",
                ".......phhhhhhhnnnhhhhhhhp.......",
                "........phhhhhhnnnhhhhhhp........",
                "ppppppppppppppppppppppppppppppppp",
                "peeeeeeeeeeeeeeeeeeeeeeeeeeeeeeep",
            ],
            None,
        ),
    ]
}

/// The starter deck's cards as they were before a round changed them, by slot. A saved design
/// still exactly like one of these was never edited, so loading swaps in the current card (match
/// round 2 drew the four warships on GUARD; after it Henry redrew all seven to use their power,
/// and the miner was redrawn once more).
pub fn retired_starters() -> Vec<(usize, Design)> {
    let d = defaults();
    let mut v = vec![(0, d[0].clone()), (1, d[1].clone()), (2, d[2].clone()), (6, d[4].clone())];
    v.extend(round2_deck().into_iter().enumerate());
    // Henry's first miner (2026-11-10): five scoops and five flak on one engine. Replaced
    // the same day at Henry's request.
    v.push((3, drawn("MINER", (48, 45), &[".g.", "ggg", "sss", "sgs", "hch", ".e."], None)));
    v.extend(cheap_parts_deck());
    v
}

/// The five starter cards that the costs of 2026-11-11 left short of power (a lance at 3, a
/// factory pixel at 2, a scoop at 1), as Henry drew them and as the miner tournament left the
/// miner. Kept for saves that still hold them.
fn cheap_parts_deck() -> Vec<(usize, Design)> {
    vec![
        (1, drawn("LANCER", (48, 47), &[".l.", ".l.", "hch", "e.e"], None)),
        (2, drawn("FRIGATE", (45, 44), &["....p....", "...php...", "..pplpp..", "ppghlhgpp", "phghchghp", "phhhlhhhp", "ppghlhgpp", "e.ppppp.e", "..e.e.e.."], None)),
        (3, drawn("MINER", (48, 47), &[".g.", "sss", "ece", ".e."], None)),
        (4, drawn("OUTPOST", (43, 43), &[".....ppp.....", "...pphhhpp...", "..phhhghhhp..", ".phhfffffhhp.", ".phfffffffhp.", "phfffhhhfffhp", "pgfffhchfffgp", "phfffhhhfffhp", ".phfffffffhp.", ".phhfffffhhp.", "..phhhghhhp..", "...pphhhpp...", ".....ppp....."], Some(&[3, 3, 0, 0, 1, 2]))),
        (
            5,
            drawn(
                "SHIPYARD",
                (36, 36),
                &[
                    "...ppppppppppppppppppppp...",
                    "...ppppppppppppppppppppp...",
                    "...ppppphhhhlllhhhhppppp...",
                    "ppppppphhhggggggghhhppppppp",
                    "pppppphhhhhhhhhhhhhhhpppppp",
                    "pppppfffffffffffffffffppppp",
                    "pppphfffffffffffffffffhpppp",
                    "ppphhffghhhhhfhhhhhgffhhppp",
                    "pphhhffhhhhhhhhhhhhhffhhhpp",
                    "pphhhffhhhhhhfhhhhhhffhhhpp",
                    "pphhhffhhfffffffffhhffhhhpp",
                    "pphhhffhhfffffffffhhffhhhpp",
                    "pplhhffhhfffffffffhhffhhlpp",
                    "pplghffhhffffcffffhhffhglpp",
                    "pplhhffhhfffffffffhhffhhlpp",
                    "pphhhffhhfffffffffhhffhhhpp",
                    "pphhhffhhfffffffffhhffhhhpp",
                    "pphhhffhhhhhhhhhhhhhffhhhpp",
                    "pphhhffhhhhhhhhhhhhhffhhhpp",
                    "ppphhffghhhhhhhhhhhgffhhppp",
                    "pppphfffffffffffffffffhpppp",
                    "pppppfffffffffffffffffppppp",
                    "pppppphhhhhhhhhhhhhhhpppppp",
                    "ppppppphhggggggggghhppppppp",
                    "...ppppphhhhlllhhhhppppp...",
                    "...ppppppppppppppppppppp...",
                    "...ppppppppppppppppppppp...",
                ],
                Some(&[3, 2, 2, 6]),
            ),
        ),
    ]
}

/// Match round 2's starter deck, as it was before Henry redrew it: cards with hull to spare,
/// and an outpost slow enough to leave its bank room to save. Kept for saves that still hold
/// its cards.
fn round2_deck() -> Vec<Design> {
    let d = defaults();
    let guard = |d: &Design| d.clone();
    let mut outpost = Design::from_rows(
        "OUTPOST",
        &[
            ".....ppp.....",
            "...pphhhpp...",
            "..phhhghhhp..",
            ".phhfffffhhp.",
            ".phfhhhhhfhp.",
            "phhfhhhhhfhhp",
            "pghfhhchhfhgp",
            "phhfhhhhhfhhp",
            ".phfhhhhhfhp.",
            ".phhfffffhhp.",
            "..phhhghhhp..",
            "...pphhhpp...",
            ".....ppp.....",
        ],
    );
    outpost.seeded = true;
    outpost.build = vec![3, 3, 0, 0, 1, 2];
    let miner = Design::from_rows("MINER", &["s.s", "sgs", "hch", ".e."]);
    vec![guard(&d[0]), guard(&d[1]), guard(&d[2]), miner, outpost, round2_shipyard(), guard(&d[4])]
}

/// Round 2's shipyard, kept so a save still holding it gets today's: an octagon plated all
/// round, with two rings of factory pixels (88 of them) and flak behind the plate.
fn round2_shipyard() -> Design {
    let mut d = Design::blank("SHIPYARD");
    let c = GRID / 2;
    for y in -12i32..=12 {
        for x in -12i32..=12 {
            if x.abs() + y.abs() <= 17 {
                d.set(c + x, c + y, Some(Part::Hull));
            }
        }
    }
    let skin: Vec<(i32, i32)> = (0..GRID * GRID)
        .map(|i| (i % GRID, i / GRID))
        .filter(|&(x, y)| d.get(x, y) == Some(Part::Hull) && [(1, 0), (-1, 0), (0, 1), (0, -1)].iter().any(|&(dx, dy)| d.get(x + dx, y + dy).is_none()))
        .collect();
    for (x, y) in skin {
        d.set(x, y, Some(Part::Plate));
    }
    for y in -8i32..=8 {
        for x in -8i32..=8 {
            let m = x.abs().max(y.abs());
            if m == 3 || m == 8 {
                d.set(c + x, c + y, Some(Part::Factory));
            }
        }
    }
    for (x, y) in [(10, 0), (-10, 0), (0, 10), (0, -10), (6, 6), (-6, 6), (6, -6), (-6, -6)] {
        d.set(c + x, c + y, Some(Part::Flak));
    }
    d.set(c, c, Some(Part::Core));
    d.seeded = true;
    d.build = vec![3, 2, 2, 6];
    d
}

/// Earlier versions of the defaults, by slot. A saved design still exactly like one of these was
/// never edited, so loading swaps in the current default (round 3 buried the old destroyer's
/// front engines and gave the capital extra cores).
pub fn retired_defaults() -> Vec<(usize, Design)> {
    let destroyer = Design::from_rows(
        "DESTROYER",
        &[
            ".....ppp.....",
            "....pphpp....",
            "...pphhhpp...",
            "..pphhhhhpp..",
            ".pphhnnnhhpp.",
            ".phhhnnnhhhp.",
            ".phhhnnnhhhp.",
            ".phhhhhhhhhp.",
            ".phhhhhhhhhp.",
            "gphhhhhhhhhpg",
            ".phhhhhhhhhp.",
            ".phhhhchhhhp.",
            ".phhhhhhhhhp.",
            "gphhhhhhhhhpg",
            ".phhhhhhhhhp.",
            "..phhhhhhhp..",
            "...ppppppp...",
            "...e.e.e.e...",
            "...e.e.e.e...",
        ],
    );
    let mut capital = defaults()[4].clone();
    let (cx, top) = (GRID / 2, (GRID - 37) / 2);
    for row in [10, 26] {
        capital.set(cx, top + row, Some(Part::Hull));
    }
    vec![(3, destroyer), (4, capital)]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_default_is_legal() {
        for d in defaults() {
            assert!(d.check().ok(), "{}: {:?}", d.name, d.check());
        }
    }

    #[test]
    fn a_line_saved_with_an_order_a_way_of_flying_or_a_group_size_loads_without_them() {
        let d = defaults()[0].clone();
        // A line has five fields: name, place, rows, kind and build order.
        let plain = d.to_line();
        assert_eq!(plain.split('|').count(), 5, "{plain}");
        assert_eq!(Design::from_line(&plain).unwrap().to_line(), plain);
        // Older lines had the design's order second, whichever it was, and the later of them
        // ended with a way of flying and a group size. Each loads as the same drawing, and is
        // saved again as today's line.
        let (name, rest) = plain.split_once('|').unwrap();
        for order in ["HUNT", "SAB", "GUARD", "ESCORT", "HARVEST", "HOLD"] {
            for tail in ["", "|ORBIT|1", "|-|6", "|KITE|12", "|RUNS|3", "|WARP|5"] {
                let old = format!("{name}|{order}|{rest}{tail}");
                let back = Design::from_line(&old).unwrap_or_else(|| panic!("{old} didn't load"));
                assert_eq!((&back.cells, back.to_line()), (&d.cells, plain.clone()), "{old}");
            }
        }
        // The oldest stopped after the rows.
        let (at, rows) = (plain.split('|').nth(1).unwrap(), plain.split('|').nth(2).unwrap());
        assert_eq!(Design::from_line(&format!("{name}|HUNT|{at}|{rows}")).unwrap().cells, d.cells);
        // Not a line: today's with fields missing, and a second field that's neither a place
        // nor any order there was.
        assert!(Design::from_line(&format!("{name}|{at}|{rows}")).is_none());
        assert!(Design::from_line(&format!("{name}|WANDER|{rest}")).is_none());
    }

    #[test]
    fn what_a_drawings_ships_do_follows_from_the_drawing() {
        // Engines and no scoop: it fights. Engines and a scoop: it mines, guns or no guns. No
        // engines: it holds where it is.
        let order = |rows: &[&str]| Design::from_rows("X", rows).stats().order();
        assert_eq!(order(&[".g.", "hch", "e.e"]), Order::Hunt);
        assert_eq!(order(&["hhh", "hch", "e.e"]), Order::Hunt);
        assert_eq!(order(&["sss", "hch", "e.e"]), Order::Harvest);
        assert_eq!(order(&["gsg", "hch", "e.e"]), Order::Harvest);
        assert_eq!(order(&["ggg", "hch"]), Order::Hold);
        assert_eq!(order(&["sss", "hch"]), Order::Hold);
        // The starter deck's cards, and the words a list has for each.
        let d = starter_deck();
        assert_eq!(d[..5].iter().map(|d| d.stats().order()).collect::<Vec<_>>(), vec![Order::Hunt, Order::Hunt, Order::Hunt, Order::Harvest, Order::Hold]);
        assert_eq!(d[..5].iter().map(|d| d.does()).collect::<Vec<_>>(), vec!["FIGHTS", "FIGHTS", "FIGHTS", "MINES", "STAYS PUT"]);
        assert_eq!(Design::from_rows("X", &["hhh", "hch", "e.e"]).does(), "FLIES, NO GUNS");
    }

    #[test]
    fn a_design_flies_as_its_drawing_suggests_until_its_side_decides() {
        let d = defaults();
        assert_eq!((d[0].flies(), d[4].flies()), (Some(Style::Runs), Some(Style::Kite)));
        // No engines: it stays put.
        assert_eq!(Design::from_rows("POST", &["ggg", "hch"]).flies(), None);
    }

    #[test]
    fn the_defaults_fly_the_way_their_drawings_suggest() {
        let d = defaults();
        assert_eq!(d[0].stats().flight, Flight::Runs, "fighter");
        assert_eq!(d[1].stats().flight, Flight::Runs, "lancer");
        assert_eq!(d[2].stats().flight, Flight::StandOff, "frigate");
        assert_eq!(d[4].stats().flight, Flight::StandOff, "capital");
        assert!(d[0].stats().vmax > d[2].stats().vmax);
        assert!(d[2].stats().turn < d[3].stats().turn && d[3].stats().turn < d[4].stats().turn, "bigger ships turn slower");
        assert_eq!(d[3].stats().cannon, 1);
        assert_eq!(d[4].stats().cannon, 4);
        assert_eq!(d[6].stats().cannon, 9);
        assert_eq!(d[6].stats().flight, Flight::StandOff, "titan");
        let sh = d[6].shape();
        assert!(sh.w <= GRID && sh.h <= GRID && sh.h >= 95, "the titan is {}x{}", sh.w, sh.h);
    }

    #[test]
    fn saves_from_the_41_grid_still_load_centred() {
        // A round 3 line: the whole 41x41 grid, no position.
        let mut rows = vec![".".repeat(41); 41];
        rows[19] = format!("{}g.g{}", ".".repeat(19), ".".repeat(19));
        rows[20] = format!("{}hch{}", ".".repeat(19), ".".repeat(19));
        rows[21] = format!("{}e.e{}", ".".repeat(19), ".".repeat(19));
        let d = Design::from_line(&format!("OLD|HUNT|{}", rows.join("/"))).expect("parses");
        assert_eq!(d.to_line(), Design::from_rows("OLD", &["g.g", "hch", "e.e"]).to_line());
        assert_eq!(d.get(GRID / 2, GRID / 2), Some(Part::Core));
    }

    #[test]
    fn boxes_fill_and_erase_and_tile_cannons() {
        let mut d = Design::blank("T");
        assert!(d.fill_box((10, 10), (14, 12), Some(Part::Hull)));
        assert_eq!(d.stats().count, 15);
        // Cannons tile in whole blocks: a 7x4 box takes two, side by side.
        let mut e = Design::blank("T");
        e.fill_box((20, 20), (26, 23), Some(Part::Cannon));
        assert_eq!(e.cannon_blocks(), (vec![(20, 20), (23, 20)], vec![]));
        // Erasing a box that clips a cannon takes the whole block.
        e.fill_box((22, 22), (22, 22), None);
        assert_eq!(e.cannon_blocks(), (vec![(23, 20)], vec![]));
        assert_eq!(e.stats().count, 9);
    }

    #[test]
    fn fills_take_holes_and_patches_but_not_open_space() {
        let mut d = Design::from_rows("T", &["ppppp", "p...p", "p.c.p", "p...p", "ppppp"]);
        let c = GRID / 2;
        // Outside the ring is open space.
        assert!(d.flood((0, 0), Some(Part::Hull)).is_err());
        // The hole inside fills, around the core.
        assert_eq!(d.flood((c - 1, c - 1), Some(Part::Hull)), Ok(true));
        assert_eq!(d.stats().count, 25);
        assert!(d.check().ok());
        // Clicking the ring swaps the whole ring.
        assert_eq!(d.flood((c - 2, c - 2), Some(Part::Hull)), Ok(true));
        assert!(!d.cells.contains(&Some(Part::Plate)));
        assert!(d.flood((c, c - 1), Some(Part::Cannon)).is_err());
    }

    #[test]
    fn stamps_copy_turn_and_paste_as_plain_pixels() {
        let src = Design::from_rows("T", &["g.g", "hch", "e.e"]);
        let st = src.as_stamp();
        assert_eq!((st.w, st.h), (3, 3));
        // Four quarter turns come back to the start; one turn puts the engines on the left.
        assert_eq!(st.turned().turned().turned().turned(), st);
        assert_eq!(st.turned().get(0, 0), Some(Part::Engine));
        assert_eq!(st.flipped().flipped(), st);
        let mut d = Design::blank("T");
        assert!(d.paste(&st, (5, 5)));
        assert_eq!(d.get(6, 6), Some(Part::Core));
        // Empty stamp cells leave what's under them.
        d.set(6, 5, Some(Part::Plate));
        d.paste(&st, (5, 5));
        assert_eq!(d.get(6, 5), Some(Part::Plate));
        // A copied box leaves out a cannon block it cuts through.
        let mut e = Design::blank("T");
        e.put(11, 11, Some(Part::Cannon));
        e.set(13, 11, Some(Part::Hull));
        let cut = e.copy_box((11, 10), (13, 12));
        assert_eq!((cut.w, cut.h), (1, 1));
        assert_eq!(cut.get(0, 0), Some(Part::Hull));
        let whole = e.copy_box((10, 10), (13, 12));
        assert_eq!(whole.cells.iter().filter(|c| **c == Some(Part::Cannon)).count(), 9);
        // Pasting over part of a cannon removes the whole block first.
        e.paste(&Stamp { w: 1, h: 1, cells: vec![Some(Part::Hull)] }, (10, 10));
        assert_eq!(e.stats().cannon, 0);
        assert_eq!(e.cannon_blocks().1, vec![]);
    }

    #[test]
    fn every_part_has_one_category() {
        for p in PARTS {
            assert_eq!(CATS.iter().filter(|c| c.parts().contains(&p)).count(), 1, "{:?}", p);
        }
    }

    #[test]
    fn loose_pixels_are_flagged_and_extra_cores_cost_more_each() {
        let mut d = Design::from_rows("T", &["g.g", "hch", "e.e"]);
        assert!(d.check().ok());
        d.set(0, 0, Some(Part::Hull));
        assert_eq!(d.check().loose, vec![(0, 0)]);
        d.set(0, 0, None);
        // A second core joined to the ship is legal, at a price.
        d.set(GRID / 2, GRID / 2 - 1, Some(Part::Core));
        assert!(d.check().ok(), "{:?}", d.check());
        assert_eq!(d.check().cores, 2);
        assert_eq!(d.stats().cost, d.stats().count + 8);
        // A core off on its own is a loose pixel, not a second ship.
        d.set(0, 0, Some(Part::Core));
        assert_eq!(d.check().loose, vec![(0, 0)]);
        assert_eq!([1, 2, 3, 4, 5].map(core_cost), [0, 8, 24, 56, 120]);
    }

    #[test]
    fn engines_need_a_straight_three_wide_way_out() {
        let buried = |rows: &[&str]| Design::from_rows("T", rows).check().buried.len();
        // Straight out the back, or out the side.
        assert_eq!(buried(&["hch", "e.e"]), 0);
        assert_eq!(buried(&["hhh", "ech", "hhh"]), 0);
        // A pixel diagonally behind is in the way: the line either side of the engine must be clear.
        assert_eq!(buried(&["hchh", "hhe.", "hh.."]), 1);
        // An engine stacked behind another is buried; the one at the back isn't.
        assert_eq!(buried(&["hhchh", "hheh.", "hheh."]), 1);
        // A 2-wide slot out the back is too narrow, and so is a 3-wide one that isn't centred on
        // the engine; a 3-wide one centred on it is enough.
        assert_eq!(buried(&["hhchhh", "hhehhh", "hh..hh", "hh..hh"]), 1);
        assert_eq!(buried(&["hhchhh", "hhehhh", "hh...h", "hh...h"]), 1);
        assert_eq!(buried(&["hhchhh", "hhehhh", "h...hh", "h...hh"]), 0);
        // A straight well, however deep in plate, leaves a line of fire to the engine.
        assert_eq!(buried(&["ppppppp", "pphchpp", "pp.e.pp", "pp...pp", "pp...pp", "pp...pp"]), 0);
        // A channel that turns a corner doesn't: plate wrapped round the engine (Henry's capital).
        let bent = ["ppppppppp", "pphchpppp", "pp.e.pppp", "pp...pppp", "pp...pppp", "pp...pppp", "pp.......", "pp.......", "pp.......", "ppppppppp"];
        assert_eq!(buried(&bent), 1);
    }

    #[test]
    fn untouched_old_defaults_are_recognised() {
        let now = defaults();
        for (slot, old) in retired_defaults() {
            assert_eq!(old.name, now[slot].name);
            assert_ne!(old.to_line(), now[slot].to_line());
        }
        assert!(!retired_defaults()[0].1.check().ok(), "the old destroyer's front engines are buried");
    }

    #[test]
    fn cannons_stamp_and_erase_as_whole_blocks() {
        let mut d = Design::blank("T");
        assert!(d.put(10, 10, Some(Part::Cannon)));
        assert_eq!(d.stats().cannon, 1);
        assert_eq!(d.stats().count, 9);
        // Stamping over part of it replaces it rather than leaving scraps.
        d.put(11, 10, Some(Part::Cannon));
        assert_eq!(d.cannon_blocks(), (vec![(10, 9)], vec![]));
        // Painting over any cell of a cannon removes the whole block.
        d.put(10, 9, Some(Part::Hull));
        assert_eq!(d.stats().cannon, 0);
        assert_eq!(d.stats().count, 1);
        // Too close to the edge to fit.
        assert!(!d.put(0, 5, Some(Part::Cannon)));
    }

    #[test]
    fn heavy_plate_and_trackers_go_down_and_come_up_whole() {
        let mut d = Design::blank("T");
        // Heavy plate is a domino with its top-left on the cell, two wide or two tall.
        assert!(d.put(10, 10, Some(Part::HeavyPlate)));
        assert_eq!((d.dominoes(), d.stats().count, d.stats().mass), ((vec![((10, 10), (11, 10))], vec![]), 2, 8));
        assert!(d.put_domino(10, 11, true));
        assert_eq!(d.dominoes(), (vec![((10, 10), (11, 10)), ((10, 11), (10, 12))], vec![]));
        assert!(!d.put_domino(10, 11, true), "the same domino again changed something");
        // A tracker stamped over one of the first domino's pixels takes that domino up whole,
        // and leaves the other.
        assert!(d.put(12, 9, Some(Part::Tracker)));
        assert_eq!((d.dominoes(), d.blocks(Part::Tracker).0), ((vec![((10, 11), (10, 12))], vec![]), vec![(11, 8)]));
        // Painting over a domino's pixel takes it up whole.
        assert!(d.put(10, 12, Some(Part::Hull)));
        assert_eq!((d.dominoes().0.len(), d.get(10, 11), d.stats().count), (0, None, 10));
        assert!(d.put(12, 9, Some(Part::Hull)));
        assert_eq!(d.stats().count, 2);
        let mut d = Design::blank("T");
        assert!(d.put(12, 12, Some(Part::Tracker)));
        assert_eq!(d.stats().count, 9);
        // Its power is for the whole block, and it's a gun.
        let st = d.stats();
        assert_eq!((st.tracker, st.used, st.armed()), (1, 16, true));
        // Painting over any pixel of it takes it up whole.
        assert!(d.put(13, 13, Some(Part::Hull)));
        assert_eq!((d.blocks(Part::Tracker).0.len(), d.stats().count), (0, 1));
        // Too near the edge to fit.
        assert!(!d.put(GRID - 1, 5, Some(Part::HeavyPlate)) && !d.put_domino(5, GRID - 1, true) && !d.put(0, 5, Some(Part::Tracker)));
        // The rectangle fills a box of an even number of cells, and leaves the last column of
        // an odd one out.
        let mut e = Design::blank("R");
        assert!(e.fill_box((20, 20), (22, 22), Some(Part::HeavyPlate)));
        assert_eq!((e.dominoes().0.len(), e.dominoes().1.len(), e.get(22, 20)), (3, 0, None));
        // Erasing a box that touches a domino takes it up whole.
        assert!(e.fill_box((21, 22), (21, 22), None));
        assert_eq!(e.dominoes().1.len(), 0);
        // Fill refuses a tracker, and heavy plate where it wouldn't cut into dominoes; it fills
        // a hole it does cut into.
        let mut ring = Design::from_rows("RING", &["hhhh", "h..h", "hhch"]);
        let (l, t, _, _) = ring.used_box().unwrap();
        assert!(ring.flood((l + 1, t + 1), Some(Part::Tracker)).is_err());
        assert_eq!(ring.flood((l + 1, t + 1), Some(Part::HeavyPlate)), Ok(true));
        assert_eq!(ring.dominoes().0.len(), 1);
        let mut odd = Design::from_rows("ODD", &["hhhhh", "h...h", "hhchh"]);
        let (l, t, _, _) = odd.used_box().unwrap();
        let before = odd.cells.clone();
        assert!(odd.flood((l + 1, t + 1), Some(Part::HeavyPlate)).is_err());
        assert_eq!(odd.cells, before, "a refused fill changed the drawing");
        // A heavy pixel left over (say, from a file) is loose; a shape a first pass in reading
        // order would cut wrong still cuts into dominoes.
        assert_eq!(Design::from_rows("F", &["aah", "aac", "ahh"]).check().loose.len(), 1);
        assert!(Design::from_rows("L", &["aaa", "ahc"]).check().ok());
        assert!(Design::from_rows("S", &["haah", "aaac", "ahhh"]).check().ok());
    }

    #[test]
    fn shapes_know_where_their_cannons_are() {
        let d = &defaults()[3];
        let s = d.shape();
        assert_eq!(s.turrets.len(), 1);
        let c = s.turrets[0];
        assert_eq!((c as i32 % s.w, c as i32 / s.w), (6, 5));
    }

    #[test]
    fn save_lines_round_trip() {
        for d in defaults() {
            let back = Design::from_line(&d.to_line()).expect("parses");
            assert_eq!(back.to_line(), d.to_line());
        }
    }

    #[test]
    fn rotated_cells_map_back() {
        for d in defaults() {
            let s = d.shape();
            for f in 0..4 {
                for i in 0..s.cells.len() {
                    if s.cells[i].is_some() {
                        let (x, y) = s.cell_xy(50, 50, f, i);
                        assert_eq!(s.cell_at(50, 50, f, x, y), Some(i));
                    }
                }
            }
        }
    }

    #[test]
    fn power_comes_from_hull_and_cores() {
        // A fighter: 2 hull and a core make 6; 2 flak and 2 engines use 4.
        let st = defaults()[0].stats();
        assert_eq!((st.made, st.used, st.power_pct()), (6, 4, 100));
        // Every default has the power it needs, so the old matchups don't move.
        for d in defaults() {
            let st = d.stats();
            assert!(st.used <= st.made, "{} uses {} of {}", d.name, st.used, st.made);
        }
        // A cannon on a bare core: 4 made, 9 used. Its gun reloads 9/4 as slow and its engine
        // pushes 4/9 as hard.
        let boat = Design::from_rows("BOAT", &["nnn", "nnn", "nnn", ".c.", ".e."]);
        let st = boat.stats();
        assert_eq!((st.made, st.used, st.power_pct()), (4, 9, 44));
        assert_eq!(st.reload(45), 102);
        assert_eq!(st.thrust, 8 * 4 / 9);
        // Five pixels of hull bring it up to full power.
        let fed = Design::from_rows("BOAT", &["nnn", "nnn", "nnn", "hch", "hhh", ".e."]);
        assert_eq!(fed.stats().power_pct(), 100);
        assert_eq!(fed.stats().reload(45), 45);
    }

    #[test]
    fn the_match_designs_are_legal() {
        assert!(king().check().ok(), "{:?}", king().check());
        assert!(seed().check().ok());
        let deck = starter_deck();
        assert_eq!(deck.len(), 7);
        for d in &deck {
            assert!(d.check().ok(), "{}: {:?}", d.name, d.check());
            assert!(d.stats().power_pct() == 100, "{} is short of power", d.name);
            if d.seeded {
                let c = d.start_cell().expect("a seed core");
                assert!(d.shape().is_core(c));
            }
            // Everything on a build order is a built design the full-grown factory can make.
            for &b in &d.build {
                assert!(!deck[b].seeded && deck[b].stats().count <= d.stats().max_build(), "{} can't build {}", d.name, deck[b].name);
            }
        }
        assert_eq!(deck[5].stats().factory, 118);
    }

    #[test]
    fn seeded_designs_and_build_orders_save() {
        let deck = starter_deck();
        for d in &deck {
            let back = Design::from_line(&d.to_line()).expect("parses");
            assert_eq!((back.seeded, &back.build), (d.seeded, &d.build));
            assert_eq!(back.to_line(), d.to_line());
        }
        // A seeded design whose seed cell isn't a core can't fly.
        let mut d = deck[4].clone();
        d.set(SEED_CELL.0, SEED_CELL.1, Some(Part::Hull));
        assert!(d.check().no_seed && !d.check().ok());
    }

    #[test]
    fn growth_goes_outward_and_stays_in_one_piece() {
        let d = &starter_deck()[5];
        let sh = d.shape();
        let order = Design::growth_order(&sh, d.start_cell().unwrap());
        assert_eq!(order.len() as i32, d.stats().count);
        assert!(sh.is_core(order[0]));
        // Every cell after the first touches one grown before it.
        let mut grown = vec![false; sh.cells.len()];
        for (k, &c) in order.iter().enumerate() {
            let (x, y) = (c as i32 % sh.w, c as i32 / sh.w);
            let touches = [(1, 0), (-1, 0), (0, 1), (0, -1)].iter().any(|&(dx, dy)| {
                let (nx, ny) = (x + dx, y + dy);
                nx >= 0 && ny >= 0 && nx < sh.w && ny < sh.h && grown[(ny * sh.w + nx) as usize]
            });
            assert!(k == 0 || touches, "cell {c} grew out of nowhere");
            grown[c] = true;
        }
    }

    #[test]
    fn the_part_table_lines_up_with_the_parts() {
        // Every part finds its own row: the table is in the order of the enum.
        assert_eq!(PARTS.len(), PART_INFO.len());
        for p in PARTS {
            let info = p.info();
            assert_eq!(format!("{p:?}").to_uppercase(), info.name.replace(' ', ""), "{:?} got {}'s row", p, info.name);
            assert_eq!(Part::from_ch(info.ch), Some(p));
            assert!(info.glyph.iter().all(|r| r.chars().count() == 7), "{}'s shape isn't 7x7", info.name);
        }
        // No two parts share a letter or a name.
        for a in &PART_INFO {
            assert_eq!(PART_INFO.iter().filter(|b| b.ch == a.ch || b.name == a.name).count(), 1, "{}", a.name);
        }
        assert_eq!(Part::Cannon.blurb().matches("WEIGHS 18.").count(), 1);
        assert_eq!(Part::power_list(true), "HULL 1, CORE 4");
        assert_eq!(secs(20), "1.3");
        assert_eq!(secs(45), "3");
    }

    #[test]
    fn beam_pixels_down_a_column_are_one_gun_and_a_gap_makes_two() {
        // Five in a column, with hull beside them to hold the ship together.
        let d = Design::from_rows("LONG", &["hb", "hb", "hb", "hb", "hb", "hc"]);
        let st = d.stats();
        assert_eq!((st.beam, st.beam_len, st.beam_reach()), (5, 5, 100));
        assert_eq!(beam_cut(st.beam_len), 20);
        assert!(st.armed() && st.aims());
        // A row of them is five guns of one, side by side: only a column fires as one.
        let wide = Design::from_rows("WIDE", &["bbbbb", "hhchh"]);
        assert_eq!((wide.stats().beam, wide.stats().beam_len, wide.stats().beam_reach()), (5, 1, 52));
        // The guns of a grid: each unbroken run down a column, by its front cell.
        let cells = ["b.b", "b..", ".b.", "b.b", "b.b"];
        let at = |i: usize| cells[i / 3].as_bytes()[i % 3] == b'b';
        assert_eq!(beam_guns(3, 5, at), vec![(0, 2), (9, 2), (7, 1), (2, 1), (11, 2)]);
        // Reach grows by the pixel up to sixteen of them, and no further; the cut has no such end.
        assert_eq!((beam_reach(0), beam_reach(1), beam_reach(16), beam_reach(40)), (0, 52, 232, 232));
        assert_eq!((beam_cut(1), beam_cut(40)), (4, 160));
    }

    #[test]
    fn the_part_examples_are_legal_spend_their_power_and_keep_their_letters() {
        for d in part_examples() {
            let st = d.stats();
            assert!(d.check().ok(), "{}: {:?}", d.name, d.check().problem());
            assert_eq!(st.used, st.made, "{} makes {} power and uses {}", d.name, st.made, st.used);
            // A design with the new parts in it comes back from its line as it went.
            let back = Design::from_line(&d.to_line()).unwrap_or_else(|| panic!("{} didn't load from its own line", d.name));
            assert_eq!(back.cells, d.cells, "{} changed on the way through its line", d.name);
        }
        let ex = part_examples();
        assert_eq!((ex[0].stats().beam_len, ex[0].stats().beam_reach()), (9, 148));
        assert_eq!(ex[1].stats().shields, 4);
        assert_eq!((ex[2].stats().beam_len, ex[2].stats().count), (2, 9));
    }

    #[test]
    fn a_line_from_outside_with_its_corner_off_the_grid_is_no_drawing() {
        let good = starter_deck()[0].to_line();
        assert!(Design::from_line(&good).is_some());
        for corner in ["-1,0", "0,-1", "99,0", "0,99", "2147483647,2147483647", "-2147483648,5"] {
            assert!(Design::from_line(&format!("X|HUNT|{corner}|c|BUILT|")).is_none(), "a drawing at {corner} was read");
        }
        // The far corner of the grid is on it; rows that run off the edge lose only what's off.
        let d = Design::from_line("X|HUNT|98,98|ch/hh|BUILT|").unwrap();
        assert_eq!(d.stats().count, 1);
        // A team is nine slots, and a list of any length makes one.
        assert_eq!(team_of(Vec::new()).len(), SLOTS);
        assert_eq!(team_of(starter_deck()).len(), SLOTS);
        assert!(team_of(starter_deck())[7].is_empty());
        let many: Vec<Design> = (0..12).flat_map(|_| starter_deck()).collect();
        assert_eq!(team_of(many).len(), SLOTS);
    }

    #[test]
    fn a_projector_pixel_needs_four_projector_neighbours_and_open_space_on_three_sides() {
        let hemmed = |rows: &[&str]| Design::from_rows("T", rows).check().hemmed.len();
        // The smallest lump that passes: a 4x4 with its corners off, standing on a neck. Its
        // inner pixels look out through the outer ones: projector pixels aren't in the way.
        let knob = [".jj.", "jjjj", "jjjj", ".jj.", ".hh.", "hhhh", "hchh"];
        assert_eq!(hemmed(&knob), 0);
        assert!(Design::from_rows("T", &knob).check().ok());
        // Too thin: a wire has two neighbours a pixel at most, a square's corners three, and a
        // plus's arms one or two (its middle pixel has its four).
        assert_eq!(hemmed(&["jjjj", ".hh.", "hchh"]), 4);
        assert_eq!(hemmed(&["jjjj", "jjjj", "jjjj", "jjjj", ".hh.", "hchh"]), 4);
        assert_eq!(hemmed(&[".j.", "jjj", ".j.", ".h.", "hch"]), 4);
        let thin = Design::from_rows("T", &["jjjj", "jjjj", "jjjj", "jjjj", ".hh.", "hchh"]).check();
        assert_eq!((thin.thin, thin.problem().unwrap().as_str()), (4, "4 PROJECTOR PX TOO THIN"));
        // A handle two pixels thick is a loop, and passes: a pixel looks out through its hole.
        assert_eq!(hemmed(&[".jjjjj.", "jjjjjjj", "jj...jj", "jj...jj", "jjjjjjj", ".jjjjj.", "..hhh..", "hhhhhhh", "hhhchhh"]), 0);
        // Shut in: the same knob with a wall of hull on one side of it has two open sides, not
        // three, and in a bay between two walls it has one. Every pixel of it fails.
        assert_eq!(hemmed(&["h.jj.", "hjjjj", "hjjjj", "h.jj.", "hhhhh", "hhchh"]), 12);
        assert_eq!(hemmed(&["h.jj.h", "hjjjjh", "hjjjjh", "h.jj.h", "hhhhhh", "hhchhh"]), 12);
        // A design with one that fails can't fly, and says why.
        let shut = Design::from_rows("T", &["h.jj.", "hjjjj", "hjjjj", "h.jj.", "hhhhh", "hhchh"]);
        assert!(!shut.check().ok());
        assert_eq!((shut.check().thin, shut.check().problem().unwrap().as_str()), (0, "12 PROJECTOR PX SHUT IN"));
        // Reach and what it holds come from how many pixels there are, nothing else.
        assert_eq!((proj_reach(12), proj_reach(25), proj_reach(37), proj_reach(81), proj_reach(225), proj_reach(900)), (30, 50, 60, 90, 150, 150));
        assert_eq!(Design::from_rows("T", &knob).stats().projector, 12);
    }

    #[test]
    fn a_repeater_column_is_one_gun_and_a_longer_one_reaches_further_and_fires_faster() {
        let st = |rows: &[&str]| Design::from_rows("T", rows).stats();
        // Three in a column are one gun three long. Three abreast are three guns one long.
        assert_eq!((st(&["hrh", "hrh", "hrh", "hch"]).repeater, st(&["hrh", "hrh", "hrh", "hch"]).rep_len), (3, 3));
        assert_eq!((st(&["rrr", "hch"]).repeater, st(&["rrr", "hch"]).rep_len), (3, 1));
        assert!(st(&["rrr", "hch"]).aims() && st(&["rrr", "hch"]).armed());
        // Each pixel adds reach, and every two take a tick off the wait, down to three ticks.
        assert_eq!((repeat_reach(0), repeat_reach(1), repeat_reach(6), repeat_reach(10), repeat_reach(16), repeat_reach(40)), (0, 68, 108, 140, 188, 188));
        assert_eq!((repeat_period(1), repeat_period(2), repeat_period(3), repeat_period(10), repeat_period(11), repeat_period(16)), (8, 8, 7, 4, 3, 3));
        for n in 1..REPEAT_MAX {
            assert!(repeat_reach(n + 1) > repeat_reach(n) && repeat_period(n + 1) <= repeat_period(n));
        }
        // The examples: a ten-pixel gun, and a knob of 38.
        let ex = part_examples();
        let (raker, aegis) = (ex.iter().find(|d| d.name == "RAKER").unwrap().stats(), ex.iter().find(|d| d.name == "AEGIS").unwrap().stats());
        assert_eq!((raker.rep_len, raker.fwd_reach()), (10, 140));
        assert_eq!((aegis.projector, proj_reach(aegis.projector)), (38, 60));
        // The AEGIS has no guns and keeps up with the big ships it's drawn to fly beside.
        assert!(!aegis.armed());
        for big in ["BEAMER", "WARDEN", "RAKER"] {
            let st = ex.iter().find(|d| d.name == big).unwrap().stats();
            assert!(aegis.vmax >= st.vmax, "the AEGIS ({}) can't keep up with the {big} ({})", aegis.vmax, st.vmax);
        }
    }
}
