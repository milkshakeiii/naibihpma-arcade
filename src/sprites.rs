use crate::palette::{IndexedSprite, VgaPalette};
use std::path::Path;

pub const TILE_SZ: usize = 24;

pub struct AssetBank {
    pub bg_canopy: IndexedSprite,
    pub bg_playfield: IndexedSprite, // Deep-recessed twilight forest backdrop
    pub bg_galley: IndexedSprite,    // Quantized side-view Naibihpma Bog-Tavern interior!

    // 22x22 for compact slots, 34x34 for field creatures & dishes, 44x44 for portraits
    pub ai_sprites_22: Vec<IndexedSprite>,
    pub ai_sprites_diced_22: Vec<IndexedSprite>,
    pub ai_sprites_34: Vec<IndexedSprite>,
    pub ai_sprites_diced_34: Vec<IndexedSprite>,
    pub ai_sprites_44: Vec<IndexedSprite>,

    // Slender Blue Alien Forager Animations (20x22 -> 40x44 EPX)
    pub cosmo_idle: IndexedSprite,
    pub cosmo_run1: IndexedSprite, // Stalking stride 1 (low forward prowl, front leg reaching)
    pub cosmo_run2: IndexedSprite, // Stalking stride 2 (crouched weight transfer)
    pub cosmo_run3: IndexedSprite, // Stalking stride 3 (back leg push-off)
    pub cosmo_crawl: IndexedSprite, // Hands-down crawling over obstacles & clambering out of water!
    pub cosmo_jump: IndexedSprite,  // Athletic horizontal gap-leap across small crevices!
    pub cosmo_cling_right: IndexedSprite, // Streamlined Star-Pond diving/swimming posture!
    pub cosmo_pluck: IndexedSprite, // Braced harpoon/tranq firing stance!

    pub toad_idle: IndexedSprite,
    pub toad_bounce: IndexedSprite,
    pub bomb_1: IndexedSprite,
    pub bomb_2: IndexedSprite,

    // 24x24 Moody Organic Bog-Stone, Root Strata, & Hollow-Bark Tiles (4 variants to avoid grid repetition)
    pub tile_earth_top: [IndexedSprite; 4],
    pub tile_earth_fill: [IndexedSprite; 4],
    pub tile_trunk: IndexedSprite,
    pub tile_lily: IndexedSprite,
    pub tile_grapple_anchor: IndexedSprite,
    pub tile_water_top: IndexedSprite,
    pub tile_water_deep: IndexedSprite,
}

impl AssetBank {
    pub fn load_and_compile(pal: &VgaPalette) -> Self {
        let bg_path = Path::new("assets/raw_ai/naibihpma_bg.png");
        let bg_canopy = pal
            .quantize_background_png(bg_path, 400, 252)
            .unwrap_or_else(|| build_fallback_bg(400, 252));
        let bg_playfield = pal.make_recessed_playfield_bg(&bg_canopy);

        let galley_path = Path::new("assets/raw_ai/galley_bg.png");
        let bg_galley = pal
            .quantize_background_png(galley_path, 400, 236)
            .unwrap_or_else(|| build_fallback_bg(400, 236));

        let sp_path = Path::new("assets/raw_ai/sprites_raw.png");
        let ai_sprites_22 = pal
            .quantize_sprite_sheet_png(sp_path, 4, 2, 22)
            .unwrap_or_else(|| (0..8).map(|_| IndexedSprite::new(22, 22)).collect());
        let ai_sprites_diced_22 = ai_sprites_22
            .iter()
            .map(|s| s.make_diced_variant())
            .collect();

        let mut ai_sprites_34 = pal
            .quantize_sprite_sheet_png(sp_path, 4, 2, 34)
            .unwrap_or_else(|| (0..8).map(|_| IndexedSprite::new(34, 34)).collect());
        let ai_sprites_diced_34 = ai_sprites_34
            .iter()
            .map(|s| s.make_diced_variant())
            .collect();

        let mut ai_sprites_44 = pal
            .quantize_sprite_sheet_png(sp_path, 4, 2, 44)
            .unwrap_or_else(|| (0..8).map(|_| IndexedSprite::new(44, 44)).collect());

        let cosmo_idle = compile_ascii_sprite(COSMO_IDLE, 20, 22).scale_epx_2x();
        if ai_sprites_44.len() > 4 {
            ai_sprites_44[4] = build_blue_alien_portrait_44(&cosmo_idle);
        }
        if ai_sprites_34.len() > 4 {
            ai_sprites_34[4] = build_blue_alien_portrait_34(&cosmo_idle);
        }

        Self {
            bg_canopy,
            bg_playfield,
            bg_galley,
            ai_sprites_22,
            ai_sprites_diced_22,
            ai_sprites_34,
            ai_sprites_diced_34,
            ai_sprites_44,
            cosmo_idle,
            cosmo_run1: compile_ascii_sprite(COSMO_RUN1, 20, 22).scale_epx_2x(),
            cosmo_run2: compile_ascii_sprite(COSMO_RUN2, 20, 22).scale_epx_2x(),
            cosmo_run3: compile_ascii_sprite(COSMO_RUN3, 20, 22).scale_epx_2x(),
            cosmo_crawl: compile_ascii_sprite(COSMO_CRAWL, 20, 22).scale_epx_2x(),
            cosmo_jump: compile_ascii_sprite(COSMO_JUMP, 20, 22).scale_epx_2x(),
            cosmo_cling_right: compile_ascii_sprite(COSMO_CLING_R, 20, 22).scale_epx_2x(),
            cosmo_pluck: compile_ascii_sprite(COSMO_PLUCK, 20, 22).scale_epx_2x(),
            toad_idle: compile_ascii_sprite(TOAD_IDLE, 20, 16).scale_epx_2x(),
            toad_bounce: compile_ascii_sprite(TOAD_BOUNCE, 20, 16).scale_epx_2x(),
            bomb_1: compile_ascii_sprite(BOMB_1, 12, 12).scale_epx_2x(),
            bomb_2: compile_ascii_sprite(BOMB_2, 12, 12).scale_epx_2x(),
            tile_earth_top: [
                build_moody_bog_earth(true, 0),
                build_moody_bog_earth(true, 1),
                build_moody_bog_earth(true, 2),
                build_moody_bog_earth(true, 3),
            ],
            tile_earth_fill: [
                build_moody_bog_earth(false, 0),
                build_moody_bog_earth(false, 1),
                build_moody_bog_earth(false, 2),
                build_moody_bog_earth(false, 3),
            ],
            tile_trunk: build_moody_trunk(),
            tile_lily: build_moody_shelf_fungus(),
            tile_grapple_anchor: build_grapple_anchor_tile(),
            tile_water_top: build_star_pond_water(true),
            tile_water_deep: build_star_pond_water(false),
        }
    }
}

fn char_to_pal(ch: char) -> u8 {
    match ch {
        '.' => 0,
        '#' => 1,
        'W' => 7,
        'S' => 6,
        'b' => 32, // Deep Cerulean-Blue Alien Skin Shadow
        'B' => 33, // Vibrant Azure-Blue Alien Skin Midtone
        'C' => 34, // Bright Aqua-Cyan Alien Highlight / Visor Glint
        'A' => 35, // Pale Ice-Cyan Antenna Tip / Glass Reflection
        'g' => 32,
        'G' => 33,
        'L' => 34,
        'M' => 35,
        'r' => 12,
        'O' => 13, // Amber-Orange Protective Hazard Trim / Harness
        'Y' => 14, // Bright Gold Hazard Seal / Buckle
        'E' => 15, // Luminous Gold Alien Eyes
        't' => 45,
        'T' => 46,
        'R' => 37,
        'P' => 38,
        'D' => 2,  // Dark Slate Protective Suit Shadow
        'd' => 4,  // Mid Slate-Grey Protective Suit Armor Plate
        'F' => 14,
        _ => 0,
    }
}

fn compile_ascii_sprite(rows: &[&str], w: usize, h: usize) -> IndexedSprite {
    let mut spr = IndexedSprite::new(w, h);
    for (y, row) in rows.iter().take(h).enumerate() {
        for (x, ch) in row.chars().take(w).enumerate() {
            spr.set(x, y, char_to_pal(ch));
        }
    }
    spr.enforce_1px_outline();
    spr
}

fn build_blue_alien_portrait_44(idle_40x44: &IndexedSprite) -> IndexedSprite {
    let mut p = IndexedSprite::new(44, 44);
    for y in 0..idle_40x44.h.min(44) {
        for x in 0..idle_40x44.w.min(40) {
            p.set(x + 2, y, idle_40x44.get(x, y));
        }
    }
    p
}

fn build_blue_alien_portrait_34(idle_40x44: &IndexedSprite) -> IndexedSprite {
    let mut p = IndexedSprite::new(34, 34);
    for y in 0..34 {
        let sy = (y * idle_40x44.h) / 34;
        for x in 0..34 {
            let sx = (x * idle_40x44.w) / 34;
            p.set(x, y, idle_40x44.get(sx, sy));
        }
    }
    p
}

// =========================================================================
// SLENDER, LITHE BLUE ALIEN IN PROTECTIVE GEAR (20x22 -> 40x44 EPX)
// - Grounded on row 21 (y = 43) with ZERO empty bottom rows so feet never float!
// - Upright, poised stealthy walk cycle (smooth creeping stride, no crouching!)
// =========================================================================
const COSMO_IDLE: &[&str] = &[
    ".....#CA#...#AC#....",
    "......#bB###Bb#.....",
    "......#dSSSSSd#.....",
    ".....#dSbBBBbSd#....",
    ".....#dBCCCCCBd#....",
    ".....#BC#E#E#CB#....",
    "......#dBCCCbd#.....",
    "......#OOSSSOO#.....",
    "....##dDDYOYDDd##...",
    "...#Dd#dDOYODd#dD#..",
    "...#bB##dDDDd##Bb#..",
    "...#Od#.#dSOd#.#dO#.",
    "....##..#dDDd#..##..",
    ".......#bB##Bb#.....",
    ".......#bB..Bb#.....",
    "......#dBb..bBd#....",
    "......#dB#..#Bd#....",
    "......#dB#..#Bd#....",
    "......#dB#..#Bd#....",
    ".....#OOd#..#dOO#...",
    ".....#ddd#..#ddd#...",
    ".....#####..#####...",
];

// Stealth Walk Frame 1: Upright creeping stride, front foot planted at row 21, back heel lifting 1px
const COSMO_RUN1: &[&str] = &[
    ".....#CA#...#AC#....",
    "......#bB###Bb#.....",
    "......#dSSSSSd#.....",
    ".....#dSbBBBbSd#....",
    ".....#dBCCCCCBd#....",
    ".....#BC#E#E#CB#....",
    "......#dBCCCbd#.....",
    "......#OOSSSOO#.....",
    "....##dDDYOYDDd##...",
    "...#Dd#dDOYODd#dD#..",
    "..#bB#.#dDDDd#.#bB#.",
    "..#Od#.#dSSOd#..#Od#",
    "...##..#dDDDDd#..##.",
    ".......#bB##Bb#.....",
    "......#bB#..#bB#....",
    ".....#dBb#..#dBb#...",
    ".....#dB#....#dB#...",
    "....#dB#.....#dBb#..",
    "...#OOd#......#dB#..",
    "...#ddd#......#OOd#.",
    "...#####......#ddd#.",
    "..............#####.",
];

// Stealth Walk Frame 2: Upright passing stride, support foot planted flat on row 21
const COSMO_RUN2: &[&str] = &[
    ".....#CA#...#AC#....",
    "......#bB###Bb#.....",
    "......#dSSSSSd#.....",
    ".....#dSbBBBbSd#....",
    ".....#dBCCCCCBd#....",
    ".....#BC#E#E#CB#....",
    "......#dBCCCbd#.....",
    "......#OOSSSOO#.....",
    "....##dDDYOYDDd##...",
    "...#Dd#dDOYODd#dD#..",
    "...#bB##dDDDd##Bb#..",
    "...#Od#.#dSOd#.#dO#.",
    "....##..#dDDd#..##..",
    ".......#bB##Bb#.....",
    ".......#bB##bB#.....",
    "......#dBb##bBd#....",
    "......#dB#..#Bd#....",
    "......#dB#..#Bd#....",
    ".....#OOd#..#dB#....",
    ".....#ddd#..#OOd#...",
    ".....#####..#ddd#...",
    "............#####...",
];

// Stealth Walk Frame 3: Opposite upright creeping stride, front foot planted at row 21
const COSMO_RUN3: &[&str] = &[
    ".....#CA#...#AC#....",
    "......#bB###Bb#.....",
    "......#dSSSSSd#.....",
    ".....#dSbBBBbSd#....",
    ".....#dBCCCCCBd#....",
    ".....#BC#E#E#CB#....",
    "......#dBCCCbd#.....",
    "......#OOSSSOO#.....",
    "....##dDDYOYDDd##...",
    "...#Dd#dDOYODd#dD#..",
    "..#bB#.#dDDDd#.#bB#.",
    ".#dO#..#dSSOd#.#dO#.",
    ".##....#dDDDDd#..##.",
    ".......#bB##Bb#.....",
    "......#bB#..#bB#....",
    ".....#dBb#..#dBb#...",
    ".....#dB#....#dB#...",
    "....#dBb#.....#dB#..",
    "....#dB#......#OOd#.",
    "...#OOd#......#ddd#.",
    "...#ddd#......#####.",
    "...#####............",
];

// Step-Up / Mantle Pose: Back foot pushing off row 21, front foot raised onto the ledge
const COSMO_CRAWL: &[&str] = &[
    "......#CA#...#AC#...",
    ".......#bB###Bb#....",
    ".......#dSSSSSd#....",
    "......#dSbBBBbSd#...",
    "......#dBCCCCCBd#...",
    "......#BC#E#E#CB#...",
    ".......#dBCCCbd#....",
    ".......#OOSSSOO#....",
    "....###dDDYOYDDd##..",
    "...#Dd#dDOYODd#bBd#.",
    "...#Od##dDDDDd##Od#.",
    "....##.#bB###bB####.",
    "......#bBd#..#bBd#..",
    "......#dBb#...#dBb#.",
    ".....#dBb#....#OOd#.",
    ".....#dB#.....#ddd#.",
    "....#dB#......#####.",
    "....#dB#............",
    "...#dB#.............",
    "..#OOd#.............",
    "..#ddd#.............",
    "..#####.............",
];

// Athletic Horizontal Gap-Leap across small crevices!
const COSMO_JUMP: &[&str] = &[
    "......#CA#...#AC#...",
    ".......#bB###Bb#....",
    ".......#dSSSSSd#....",
    "......#dSbBBBbSd#...",
    "......#dBCCCCCBd#...",
    "......#BC#E#E#CB#...",
    ".......#dBCCCbd#....",
    "....###OOSSSSOO###..",
    "..#bBd#dDDYOYDDd#bB#",
    ".#Od###dDOYCYODd##Od",
    ".###...#dDDDDd#..###",
    ".......#bB###Bb#....",
    "......#bBd#.#dBb#...",
    ".....#dBb#...#dBb#..",
    "....#dBb#.....#dBb#.",
    "...#dBb#.......#dBb#",
    "..#OOd#.........#OOd",
    "..#ddd#.........#ddd",
    "..#####.........####",
    "....................",
    "....................",
    "....................",
];

// Streamlined Horizontal Diving / Swimming Posture in the Star-Pond!
// Positioned at rows 8..18 (y = 16..37) so the diver stays cleanly beneath the water meniscus!
const COSMO_CLING_R: &[&str] = &[
    "....................",
    "....................",
    "....................",
    "....................",
    "....................",
    "....................",
    "....................",
    "....................",
    ".........#CA#..#AC#.",
    "..........#bB##Bb#..",
    "..........#dSSSSSd#.",
    "....#####.#dSbBBBbSd",
    "..#dDDDDd##dBCCCCCBd",
    ".#bB###dDOYOO#E#E#CB",
    "#OOd#.#dDDDDdSSSSOO#",
    "#ddd#..#bB####bB##..",
    "#####...#bBd#.#bBd#.",
    ".........#Od#..#Od#.",
    ".........####..####.",
    "....................",
    "....................",
    "....................",
];

// Braced Harpoon / Tranq Firing Stance (boots planted flat on row 21!)
const COSMO_PLUCK: &[&str] = &[
    ".....#CA#...#AC#....",
    "......#bB###Bb#.....",
    "......#dSSSSSd#.....",
    ".....#dSbBBBbSd#....",
    ".....#dBCCCCCBd#....",
    ".....#BC#E#E#CB#....",
    "......#dBCCCbd#.....",
    "......#OOSSSOO#.....",
    "....##dDDYOYDDd###..",
    "...#Dd#dDOYODd#OdD#.",
    "...#bB##dDDDDdS#OYO#",
    "...#Od#.#dSOd######.",
    "....##..#dDDd#......",
    ".......#bB##Bb#.....",
    ".......#bB..Bb#.....",
    "......#bBd..dBb#....",
    ".....#dBb....bBd#...",
    ".....#dB#....#Bd#...",
    ".....#dB#....#Bd#...",
    "....#OOd#....#dOO#..",
    "....#ddd#....#ddd#..",
    "....#####....#####..",
];

const TOAD_IDLE: &[&str] = &[
    "....................",
    "....................",
    "....###....###......",
    "...#E#t####t#E#.....",
    "..#tTTTTTTTTTTt#....",
    ".#tTTTTTTTTTTTTt#...",
    ".#tTT########TTt#...",
    ".#tT#RRRRRRRR#Tt#...",
    ".#tT#RPPPPPPR#Tt#...",
    "..#t#RRRRRRRR#t#....",
    ".#tTTTTTTTTTTTTt#...",
    "#tTTt########tTTt#..",
    "#ttt#........#ttt#..",
    "#####........#####..",
    "....................",
    "....................",
];

const TOAD_BOUNCE: &[&str] = &[
    "....###....###......",
    "...#E#t####t#E#.....",
    "..#tTTTTTTTTTTt#....",
    ".#tTT########TTt#...",
    "#tT##RRRRRRRR##Tt#..",
    "#t#RRRRPPPPPPPPR#t#.",
    "#t#RRPPPPPPPPPPPR#t#",
    "#t#RRPPPPPPPPPPPR#t#",
    "#t#RRRRPPPPPPPPR#t#.",
    ".#t##RRRRRRRR##t#...",
    ".#tTTTTTTTTTTTTt#...",
    "#tTTt########tTTt#..",
    "#ttt#........#ttt#..",
    "#####........#####..",
    "....................",
    "....................",
];

const BOMB_1: &[&str] = &[
    ".....##F#...",
    "....#..#F...",
    "...######...",
    "..#DDDDDD#..",
    ".#DDddDDDD#.",
    ".#DDddDDDD#.",
    ".#DDDDDDDD#.",
    ".#DDDDDDDD#.",
    "..#DDDDDD#..",
    "...######...",
    "............",
    "............",
];

const BOMB_2: &[&str] = &[
    "......#F#...",
    "....##FFE...",
    "...######...",
    "..#DDDDDD#..",
    ".#DDddDDDD#.",
    ".#DDddDDDD#.",
    ".#DDDDDDDD#.",
    ".#DDDDDDDD#.",
    "..#DDDDDD#..",
    "...######...",
    "............",
    "............",
];

/// Builds a moody, nocturnal bog-stone & tangled root-strata block (NO square Mario bricks or neon grass!).
/// Uses deep cavern indigo/slate (1, 16, 17, 27), dark umber roots (19, 20), and damp twilight moss (22, 23, 18).
fn build_moody_bog_earth(moss_cap: bool, variant: usize) -> IndexedSprite {
    let mut t = IndexedSprite::new(TILE_SZ, TILE_SZ);
    let v = variant * 7;

    for y in 0..TILE_SZ {
        for x in 0..TILE_SZ {
            // Winding organic root-vein and fractured bog-slate math
            let wave1 = ((x + v) * 3 + (y * 2)) % 11;
            let wave2 = ((x * 2 + y * 5 + v * 3) ^ (y * 3)) % 13;
            let root_vein = (y + (x + v) / 3) % 7 == 0 || (x + y / 2 + v) % 9 == 0;

            let mut col = if root_vein {
                if (x + y + v) % 3 == 0 {
                    20 // Damp root ridge
                } else {
                    19 // Dark umber root
                }
            } else if wave1 <= 2 {
                1 // Deep bog fissure shadow
            } else if wave2 <= 3 {
                27 // Bruised cavern violet stone
            } else if (x + y + v) % 2 == 0 {
                16 // Midnight slate
            } else {
                17 // Damp marsh slate
            };

            // Occasional tiny bioluminescent cyan spore-speck deep in the rock
            if !moss_cap && (x * 13 + y * 17 + variant * 19) % 97 == 0 {
                col = 32;
            }

            if moss_cap {
                let overhang = 4 + ((x * 3 + v) % 3) + ((x / 4 + variant) % 2);
                if y == 0 {
                    // Moody dark teal / forest moss highlight right at the top surface (y == 0)!
                    col = if (x + v) % 3 == 0 { 24 } else { 23 };
                } else if y <= overhang {
                    col = if (x + y + v) % 2 == 0 { 22 } else { 18 };
                } else if y == overhang + 1 {
                    col = 1; // Irregular dark root shadow under the moss cushion
                }
            }
            t.set(x, y, col);
        }
    }
    t
}

/// Builds a moody, gnarled ancient hollow-bark trunk tile (twisted dark bark fissures & damp moss hollows)
fn build_moody_trunk() -> IndexedSprite {
    let mut t = IndexedSprite::new(TILE_SZ, TILE_SZ);
    for y in 0..TILE_SZ {
        for x in 0..TILE_SZ {
            let twist = (x as i32 + ((y as i32 * 2) / 5) - 2).rem_euclid(7);
            let knot = ((x as i32 - 12).pow(2) + (y as i32 - 11).pow(2)) <= 10;

            let col = if x == 0 || x == TILE_SZ - 1 {
                1
            } else if x == 1 || x == TILE_SZ - 2 {
                // Subtle damp moss/bark grip edge
                if y % 3 == 0 { 22 } else { 20 }
            } else if knot {
                if (x + y) % 2 == 0 { 1 } else { 19 }
            } else if twist == 0 {
                1 // Deep bark fissure
            } else if twist <= 2 {
                19 // Dark umber bark
            } else if twist == 3 && (y % 4 != 0) {
                20 // Mid bark grain
            } else {
                22 // Shadowy moss-tinted bark
            };
            t.set(x, y, col);
        }
    }
    t
}

/// Builds a moody twilight shelf-fungus / dark bog-frond one-way platform (instead of bright green T-bars)
fn build_moody_shelf_fungus() -> IndexedSprite {
    let mut t = IndexedSprite::new(TILE_SZ, TILE_SZ);
    for y in 0..TILE_SZ {
        for x in 0..TILE_SZ {
            let arch = ((x as i32 - 12).abs()) / 5;
            let col = if y == 0 {
                1
            } else if y <= 2 {
                if (x + y) % 3 == 0 { 33 } else { 18 } // Muted cerulean/teal fungal cap rim
            } else if y <= (6 - arch as usize).max(3) {
                if (x + y) % 2 == 0 { 28 } else { 27 } // Twilight purple-violet shelf body
            } else if y == (7 - arch as usize).max(4) {
                1
            } else if y <= 11 && x >= 7 && x <= 16 && (x % 2 == 0) {
                27 // Dark spore gills underneath
            } else {
                0
            };
            t.set(x, y, col);
        }
    }
    t
}

/// Builds a glowing amber-gold & cyan bioluminescent Grapple-Root Anchor Ring (24x24)
/// that the Pneumatic Harpoon ([UP/DIAG] + [X]) locks onto to winch across chasms & cliffs!
fn build_grapple_anchor_tile() -> IndexedSprite {
    let mut t = IndexedSprite::new(TILE_SZ, TILE_SZ);
    for y in 0..TILE_SZ {
        for x in 0..TILE_SZ {
            let dx = x as i32 - 12;
            let dy = y as i32 - 13;
            let dist_sq = dx * dx + dy * dy;

            let col = if y < 5 && dx.abs() <= 3 {
                // Twisted overhead root mount
                if dx.abs() == 3 { 1 } else if (x + y) % 2 == 0 { 20 } else { 19 }
            } else if dist_sq >= 20 && dist_sq <= 52 {
                // Glowing amber-gold & cyan grapple ring
                if dist_sq >= 42 || dist_sq <= 24 {
                    1
                } else if dy < 0 {
                    15 // Bright gold upper rim
                } else {
                    14 // Warm amber lower rim
                }
            } else if dist_sq < 12 {
                // Bioluminescent cyan target core
                if dist_sq <= 4 { 35 } else { 33 }
            } else {
                0
            };
            t.set(x, y, col);
        }
    }
    t
}

/// Builds a moody, bioluminescent Star-Pond water tile (surface shimmer + deep twilight basin)
fn build_star_pond_water(surface: bool) -> IndexedSprite {
    let mut t = IndexedSprite::new(TILE_SZ, TILE_SZ);
    for y in 0..TILE_SZ {
        for x in 0..TILE_SZ {
            if surface && y < 3 {
                t.set(x, y, 0);
                continue;
            }
            let col = if surface && y == 3 {
                if ((x + 2) % 6) < 4 { 34 } else { 35 } // Calm cyan star-pond surface meniscus
            } else if surface && y == 4 {
                33
            } else {
                // Deep twilight pond water with calm horizontal ripples & rare glowing spore-motes
                let mote = (x * 13 + y * 29) % 89 == 0;
                let horiz_ripple = (y % 7 == 2) && ((x + y * 3) % 5 < 3);
                if mote {
                    33
                } else if horiz_ripple {
                    18
                } else if (x + y) % 2 == 0 {
                    16
                } else {
                    1
                }
            };
            t.set(x, y, col);
        }
    }
    t
}

fn build_fallback_bg(w: usize, h: usize) -> IndexedSprite {
    let mut bg = IndexedSprite::new(w, h);
    for y in 0..h {
        for x in 0..w {
            let c = if y < h / 3 {
                16
            } else if y < (h * 2) / 3 {
                17
            } else {
                18
            };
            bg.set(x, y, c);
        }
    }
    bg
}
