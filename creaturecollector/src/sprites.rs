use crate::data::NUM_SPECIES;
use crate::palette::{IndexedSprite, VgaPalette};
use std::path::Path;

pub const TILE_SZ: usize = 20;
pub const MON_SZ: usize = 24;

pub struct AssetBank {
    pub bg_canopy: IndexedSprite,
    pub bg_playfield: IndexedSprite, // 400x240 deep-recessed twilight forest/ruin backdrop

    pub alien_portrait_36: IndexedSprite,

    // 1:1 Pixel-Pure Slender Blue Alien Xenobiologist (20x20, boots planted at row 19!)
    pub alien_idle: IndexedSprite,
    pub alien_run1: IndexedSprite,
    pub alien_run2: IndexedSprite,
    pub alien_run3: IndexedSprite,
    pub alien_climb1: IndexedSprite,
    pub alien_climb2: IndexedSprite,

    // 14 Bespoke 24x24 1:1 Pixel-Grounded Monster Sprites (feet planted strictly at row 23!)
    pub monsters: [IndexedSprite; NUM_SPECIES],

    // Side-View Pokemon-Style Habitat & World Tiles (20x20 1:1)
    pub tile_stone_road: [IndexedSprite; 4],
    pub tile_earth_sub: [IndexedSprite; 4],
    pub tile_ladder: IndexedSprite,
    pub tile_fern_grass: [IndexedSprite; 2],
    pub tile_star_water: [IndexedSprite; 2],
    pub tile_crystal_grotto: [IndexedSprite; 2],
    pub tile_alpha_grove: [IndexedSprite; 2],

    // Interactive Landmarks & Battle Props
    pub spr_cache_closed: IndexedSprite,
    pub spr_cache_open: IndexedSprite,
    pub spr_med_beacon: IndexedSprite,
    pub spr_pod_12: IndexedSprite,
}

impl AssetBank {
    pub fn load_and_compile(pal: &VgaPalette) -> Self {
        let bg_local = Path::new("assets/raw_ai/naibihpma_bg.png");
        let bg_nested = Path::new("creaturecollector/assets/raw_ai/naibihpma_bg.png");
        let bg_path = if bg_local.exists() { bg_local } else { bg_nested };
        let bg_canopy = pal
            .quantize_background_png(bg_path, 400, 300)
            .unwrap_or_else(|| build_fallback_bg(400, 300));
        let bg_playfield_src = pal
            .quantize_background_png(bg_path, 400, 240)
            .unwrap_or_else(|| build_fallback_bg(400, 240));
        let bg_playfield = pal.make_recessed_playfield_bg(&bg_playfield_src);

        let alien_idle = compile_ascii_sprite(ALIEN_IDLE, 20, 20);
        let alien_portrait_36 = build_portrait_36(&alien_idle);

        let monsters = [
            compile_ascii_sprite(MON_00_DEW_NEWT, MON_SZ, MON_SZ),
            compile_ascii_sprite(MON_01_SPORE_MOREL, MON_SZ, MON_SZ),
            compile_ascii_sprite(MON_02_BRIAR_TOAD, MON_SZ, MON_SZ),
            compile_ascii_sprite(MON_03_VENOM_VIPER, MON_SZ, MON_SZ),
            compile_ascii_sprite(MON_04_VOLTAIC_EEL, MON_SZ, MON_SZ),
            compile_ascii_sprite(MON_05_DEW_FROG, MON_SZ, MON_SZ),
            compile_ascii_sprite(MON_06_STAR_CARP, MON_SZ, MON_SZ),
            compile_ascii_sprite(MON_07_LANTERN_CRAB, MON_SZ, MON_SZ),
            compile_ascii_sprite(MON_08_RUNE_BEETLE, MON_SZ, MON_SZ),
            compile_ascii_sprite(MON_09_BOG_GOLEM, MON_SZ, MON_SZ),
            compile_ascii_sprite(MON_10_GLOOM_MOTH, MON_SZ, MON_SZ),
            compile_ascii_sprite(MON_11_CROWN_SALAMANDER, MON_SZ, MON_SZ),
            compile_ascii_sprite(MON_12_BOSS_DAOT, MON_SZ, MON_SZ),
            compile_ascii_sprite(MON_13_BOSS_GROF, MON_SZ, MON_SZ),
        ];

        Self {
            bg_canopy,
            bg_playfield,
            alien_portrait_36,
            alien_idle,
            alien_run1: compile_ascii_sprite(ALIEN_RUN1, 20, 20),
            alien_run2: compile_ascii_sprite(ALIEN_RUN2, 20, 20),
            alien_run3: compile_ascii_sprite(ALIEN_RUN3, 20, 20),
            alien_climb1: compile_ascii_sprite(ALIEN_CLIMB1, 20, 20),
            alien_climb2: compile_ascii_sprite(ALIEN_CLIMB2, 20, 20),
            monsters,
            tile_stone_road: [
                build_stone_road_tile(0),
                build_stone_road_tile(1),
                build_stone_road_tile(2),
                build_stone_road_tile(3),
            ],
            tile_earth_sub: [
                build_earth_sub_tile(0),
                build_earth_sub_tile(1),
                build_earth_sub_tile(2),
                build_earth_sub_tile(3),
            ],
            tile_ladder: build_ladder_tile(),
            tile_fern_grass: [build_fern_grass_tile(0), build_fern_grass_tile(1)],
            tile_star_water: [build_star_water_tile(0), build_star_water_tile(1)],
            tile_crystal_grotto: [build_crystal_grotto_tile(0), build_crystal_grotto_tile(1)],
            tile_alpha_grove: [build_alpha_grove_tile(0), build_alpha_grove_tile(1)],
            spr_cache_closed: build_cache_chest(false),
            spr_cache_open: build_cache_chest(true),
            spr_med_beacon: build_med_beacon(),
            spr_pod_12: build_containment_pod_12(),
        }
    }
}

fn char_to_pal(ch: char) -> u8 {
    match ch {
        '.' => 0,
        '#' => 1,  // 1px Dark Bog Ink Hull Outline
        'W' => 7,  // Crisp White Glint
        'S' => 6,  // Silver Plate / Trim
        's' => 5,  // Steel Mid
        'D' => 2,  // Dark Slate Shadow
        'd' => 4,  // Mid Slate Armor
        'b' => 32, // Deep Cerulean Shadow
        'B' => 33, // Vibrant Cyan Body
        'C' => 34, // Bright Aqua Highlight
        'A' => 35, // Bioluminescent Ice Highlight
        'k' => 22, // Deep Bog Moss Shadow
        'g' => 23, // Forest Canopy Green
        'j' => 24, // Lush Jade Green
        'G' => 25, // Bright Fern Green
        'Z' => 26, // Chartreuse Accent
        't' => 45, // Toad Olive Mid
        'L' => 46, // Toad Olive Highlight
        'u' => 27, // Deep Cavern Violet
        'p' => 28, // Twilight Purple
        'v' => 29, // Bruised Lotus Magenta
        'V' => 30, // Orchid Spore Glow
        'r' => 36, // Dark Crimson
        'R' => 37, // Ruby Red
        'P' => 38, // Coral Pink
        'm' => 19, // Dark Bark Brown
        'n' => 20, // Mid Bark Brown
        'N' => 21, // Light Bark Ridge
        'o' => 39, // Umber Bronze
        'O' => 13, // Bright Orange-Red
        'Y' => 14, // Tangerine Highlight
        'q' => 40, // Copper Ochre
        'E' => 15, // Arcade Gold
        'K' => 42, // Cream Gold Highlight
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

/// Exact 2x integer block portrait (36x36) for HUD status panel (zero fractional scaling!)
fn build_portrait_36(idle_20: &IndexedSprite) -> IndexedSprite {
    let mut p = IndexedSprite::new(36, 36);
    for y in 0..18 {
        for x in 0..18 {
            let c = idle_20.get(x + 1, y);
            p.set(x * 2, y * 2, c);
            p.set(x * 2 + 1, y * 2, c);
            p.set(x * 2, y * 2 + 1, c);
            p.set(x * 2 + 1, y * 2 + 1, c);
        }
    }
    p
}

// =========================================================================
// 1:1 PIXEL-PURE SLENDER BLUE ALIEN XENOBIOLOGIST (20x20, grounded row 19)
// =========================================================================
const ALIEN_IDLE: &[&str] = &[
    ".....#CA#..#AC#.....",
    "......#bB##Bb#......",
    "......#dSSSSd#......",
    ".....#dBCCCCBd#.....",
    ".....#BC#E##CB#.....",
    "......#dBCCbd#......",
    "......#OOSSOO#......",
    "....##dDDYODDd##....",
    "...#Dd#dDOYDd#dD#...",
    "...#bB##dDDd##Bb#...",
    "...#Od#.#dSd#.#dO#..",
    "....##..#dDd#..##...",
    ".......#bB#Bb#......",
    "......#dBb.bBd#.....",
    "......#dB#.#Bd#.....",
    "......#dB#.#Bd#.....",
    "......#dB#.#Bd#.....",
    ".....#OOd#.#dOO#....",
    ".....#ddd#.#ddd#....",
    ".....#####.#####....",
];

const ALIEN_RUN1: &[&str] = &[
    ".....#CA#..#AC#.....",
    "......#bB##Bb#......",
    "......#dSSSSd#......",
    ".....#dBCCCCBd#.....",
    ".....#BC#E##CB#.....",
    "......#dBCCbd#......",
    "......#OOSSOO#......",
    "....##dDDYODDd##....",
    "...#bB#dDOYDd#bB#...",
    "..#Od#.#dDDd#.#Od#..",
    "...##..#dSDd#..##...",
    ".......#bB#Bb#......",
    "......#bB#.#bB#.....",
    ".....#dBb#.#dBb#....",
    "....#dB#....#dB#....",
    "...#OOd#.....#dB#...",
    "...#ddd#.....#OOd#..",
    "...#####.....#ddd#..",
    ".............#ddd#..",
    ".............#####..",
];

const ALIEN_RUN2: &[&str] = &[
    ".....#CA#..#AC#.....",
    "......#bB##Bb#......",
    "......#dSSSSd#......",
    ".....#dBCCCCBd#.....",
    ".....#BC#E##CB#.....",
    "......#dBCCbd#......",
    "......#OOSSOO#......",
    "....##dDDYODDd##....",
    "...#Dd#dDOYDd#dD#...",
    "...#bB##dDDd##Bb#...",
    "...#Od#.#dSd#.#dO#..",
    "....##..#dDd#..##...",
    ".......#bB#bB#......",
    "......#dBb#bBd#.....",
    "......#dB#.#Bd#.....",
    ".....#OOd#.#Bd#.....",
    ".....#ddd#.#dOO#....",
    ".....#####.#ddd#....",
    "...........#ddd#....",
    "...........#####....",
];

const ALIEN_RUN3: &[&str] = &[
    ".....#CA#..#AC#.....",
    "......#bB##Bb#......",
    "......#dSSSSd#......",
    ".....#dBCCCCBd#.....",
    ".....#BC#E##CB#.....",
    "......#dBCCbd#......",
    "......#OOSSOO#......",
    "....##dDDYODDd##....",
    "...#bB#dDOYDd#bB#...",
    "..#dO#.#dDDd#.#dO#..",
    "..##...#dSDd#...##..",
    ".......#bB#Bb#......",
    "......#bB#.#bB#.....",
    ".....#dBb#.#dBb#....",
    "....#dBb#....#dB#...",
    "....#dB#.....#OOd#..",
    "...#OOd#.....#ddd#..",
    "...#ddd#.....#####..",
    "...#ddd#............",
    "...#####............",
];

const ALIEN_CLIMB1: &[&str] = &[
    "....#Od#............",
    "....#bB##CA##AC#....",
    ".....#dB#bB##Bb#....",
    "......#d#dSSSSd#....",
    ".......##dBCCBd##...",
    "........#OOSSOO#dO#.",
    ".......#dDDYODDd#bB#",
    ".......#dDOYODd##d#.",
    ".......#bB#dDDd#....",
    "......#dB##bB#Bb#...",
    "......#OOd#.#dBb#...",
    "......#ddd#..#dB#...",
    "......#####..#dB#...",
    ".............#dB#...",
    "............#OOd#...",
    "............#ddd#...",
    "............#####...",
    "....................",
    "....................",
    "....................",
];

const ALIEN_CLIMB2: &[&str] = &[
    "............#dO#....",
    "....#CA##AC##Bb#....",
    "....#bB##Bb#Bd#.....",
    "....#dSSSSd#d#......",
    "...##dBCCBd##.......",
    ".#Od#OOSSOO#........",
    "#bB#dDDYODDd#.......",
    ".#d##dDOYODd#.......",
    "....#dDDd#Bb#.......",
    "...#bB#Bb##Bd#......",
    "...#bBd#.#dOO#......",
    "...#Bd#..#ddd#......",
    "...#Bd#..#####......",
    "...#Bd#.............",
    "...#dOO#............",
    "...#ddd#............",
    "...#####............",
    "....................",
    "....................",
    "....................",
];

// =========================================================================
// 14 BESPOKE 24x24 1:1 PIXEL-GROUNDED MONSTER SPRITES (grounded on row 23!)
// =========================================================================

// 0: TWEN DEW-NEWT (Bioluminescent cyan/emerald frilled salamander)
const MON_00_DEW_NEWT: &[&str] = &[
    "........................",
    ".........#A#.#A#........",
    "........#CA###AC#.......",
    ".......#CBBjGjBBC#......",
    "......#BBjjGGGjjBB#.....",
    ".....#AC#GG#E#GG#CA#....",
    "......##jGGGGGGGj##.....",
    ".......#jGGZZZGGj#......",
    "...###..#jjCCCjj#.......",
    "..#CAA#.#gBCCCBBg#......",
    ".#CBjB#.#gBBACBBgg#.....",
    ".#Bjg#..#ggBBBBjjg#.....",
    ".#Bgg#..#ggjBBjGGGg#....",
    "..#jg###kggjjjGGGGg#....",
    "...#jggkgggGGGGZZGg#....",
    "....#kggggGGGGGjjgk#....",
    ".....##kggjjjjjgkk#.....",
    "......#jgg#...#jgg#.....",
    ".....#jGGg#...#jGGg#....",
    ".....#jGg#.....#jGg#....",
    "....#CBjg#.....#CBjg#...",
    "....#CCBg#.....#CCBg#...",
    "...#ACCCB#.....#ACCCB#..",
    "...#######.....#######..",
];

// 1: SPORE-CAP MOREL (Spotted orchid-magenta mushroom caster)
const MON_01_SPORE_MOREL: &[&str] = &[
    "........................",
    "........########........",
    "......##vvVVVVvv##......",
    "....##vvVVAAVVVVvv##....",
    "...#pvVVAAWWAAVVAAvp#...",
    "..#pvvVVVAAAAVVAWWAvp#..",
    ".#ppvvvVVVVVVVVVAAvvpp#.",
    ".#pAAvvvvVVVVVVvvvvvpp#.",
    "#ppWWAvvvvVVVVvvvvAAupp#",
    "#pppuvvvvvvvvvvvvvuuupp#",
    ".###uuuuuppppppuuuuu###.",
    "....####KKKKKKKK####....",
    "......#qKKE##EKKq#......",
    "......#qKK#WW#KKq#......",
    ".....#qqKKKKKKKKqq#.....",
    "....#VqqqKKKKKKqqqV#....",
    "...#AV#qqqKKKKqqq#VA#...",
    "...##.#oqqKKKKqqo#.##...",
    "......#ooqqqqqqoo#......",
    ".....#oooqq##qqooo#.....",
    ".....#ooqq#..#qqoo#.....",
    "....#mooqq#..#qqoom#....",
    "...#mmmooq#..#qoommm#...",
    "...########..########...",
];

// 2: BRIAR-THORN TOAD (Armored olive/bramble frontline toad with ruby vocal sac)
const MON_02_BRIAR_TOAD: &[&str] = &[
    "........................",
    "......#Z#......#Z#......",
    ".....#GZG#....#GZG#.....",
    "....#kttt######tttk#....",
    "...#ktLLttttttttLLtk#...",
    "..#ktL#E#tttttt#E#Ltk#..",
    "..#ktt###tLLLLt###ttk#..",
    ".#Z#ttttLLLLLLLLtttt#Z#.",
    "#GZttrrRRRRRRRRRRrrttZG#",
    ".#ktrRRPPPPPPPPPRRrtk#..",
    ".#ktRRPPPPWWWWPPPPRRtk#.",
    "#ZktRRPPPPPPPPPPPPRRtkZ#",
    "#GktrrRRRRPPPPPRRRRrtkG#",
    ".#ktttrrrrRRRRrrrrtttk#.",
    ".#ktttttttLLLLtttttttk#.",
    "..#kkttttttttttttttkk#..",
    "..#ttkkttttttttttkktt#..",
    ".#tLLt#kkkkkkkkkk#tLLt#.",
    ".#tLt#.#ktt##ttk#.#tLt#.",
    ".#tt#..#ktt##ttk#..#tt#.",
    "#tLLt#.#tt#..#tt#.#tLLt#",
    "#tttt#.#tLt##tLt#.#tttt#",
    "#kkkk#.#kkkkkkkk#.#kkkk#",
    "######.##########.######",
];

// 3: VENOM-LASH VIPER (Coiled horned bog-cobra with dripping fangs)
const MON_03_VENOM_VIPER: &[&str] = &[
    "........................",
    "....#V#..........#V#....",
    "...#vVv##......##vVv#...",
    "...#ukGGG######GGGku#...",
    "..#kGGZZZZGGGGZZZZGGk#..",
    "..#kGZ#R#GGGGGG#R#ZGk#..",
    "..#kGGGGGGGGGGGGGGGGk#..",
    "...#kGG#W#rrrr#W#GGk#...",
    "....#kG#W#PPPP#W#Gk#....",
    ".....##V##kGGk##V##.....",
    "........#kGZZGk#........",
    ".......#kGGZZGGk#.......",
    "......#kGGjZZjGGk#......",
    ".....#kGGjjZZjjGGk#.....",
    "....#kGGjjkggkjjGGk#....",
    "...#kGGjjk#..#kjjGGk#...",
    "..#kGGjjk##..##kjjGGk#..",
    "..#kGZZGGGG##GGGGZZGk#..",
    ".#kGGZZZZGGGGGGZZZZGGk#.",
    ".#kGGGjjjjGGGGjjjjGGGk#.",
    ".#kkGGGGGGGGGGGGGGGGkk#.",
    "..#kkkGGGGZZZZGGGGkkk#..",
    "..#kkkkjjjjjjjjjjkkkk#..",
    "...##################...",
];

// 4: VOLTAIC REED-EEL (Crackling electric cyan/gold dynamo serpent)
const MON_04_VOLTAIC_EEL: &[&str] = &[
    "........................",
    ".........#E##E#.........",
    ".......##KKEEEK##.......",
    ".....##BBCCCCCCBB##.....",
    "....#BBCCAAWWAAcCBB#....",
    "...#BBC#E#CCCC#E#CBB#...",
    "...#BBCCCCCCCCCCCCBB#...",
    "....#bBBBCCCCCCBBBb#....",
    ".....##bbBBBBBBbb##.#E#.",
    ".......#bbBBBBbb#..#KEK#",
    "......#EbbBCCBbb#..#CEC#",
    ".....#KEbBCCCCBb#.#bCCB#",
    ".....#EbBCCCCCCBb##bCBB#",
    "......#bBCCAAcCCBBbbBBb#",
    ".....#E#bBBCCBBBBBBBBb#.",
    "....#KEK#bbBBBBbb#bbb#..",
    ".....#bBBbbbbbb##.......",
    "....#bBCCCCBBBBbb#......",
    "...#bBCCCAACCCCBBb##....",
    "..#bBBCCCCCCCCCCBBBb#...",
    "..#bBBBBCCCCCCCCBBBBb#..",
    ".#EbbBBBBBBBBBBBBBBbbE#.",
    ".#KKbbbbbbbbbbbbbbbbKK#.",
    "..####################..",
];

// 5: DEW-WEAVER FROG (Azure/gold chorus tree-frog with glowing throat sac)
const MON_05_DEW_FROG: &[&str] = &[
    "........................",
    "........................",
    ".....####......####.....",
    "....#CCAA#....#AACC#....",
    "...#CB#E#B####B#E#BC#...",
    "...#CBBBBCCCCCCBBBBC#...",
    "..#bBBBBCCCCCCCCBBBBb#..",
    "..#bBBBCCCCCCCCCCBBBb#..",
    "...#bBBKKKKKKKKKKBBb#...",
    "..#bBBKKEEEEEEEEKKBBb#..",
    ".#bBBKKEEWWWWWWEEKKBBb#.",
    ".#bBBKKEEEEEEEEEKKKBBb#.",
    ".#bBBBKKKKKKKKKKKKBBBb#.",
    "..#bbBBBBEEEEEEBBBBbb#..",
    "..#BBbbbbBBBBBBbbbbBB#..",
    ".#BCCB#bbBBBBBBbb#BCCB#.",
    ".#BCB#.#bbBBBBbb#.#BCB#.",
    ".#bB#..#bb#..#bb#..#Bb#.",
    "#CBC#.#bBB#..#BBb#.#CBC#",
    "#ACC#.#bBb#..#bBb#.#CCA#",
    "####.#CBb#....#bBC#.####",
    "....#ACCB#....#BCCA#....",
    "...#EEAAB#....#BAAEE#...",
    "...#######....#######...",
];

// 6: STAR-SCALE CARP (Astral crowned koi spirit grounded on water-crest)
const MON_06_STAR_CARP: &[&str] = &[
    "........................",
    "..........#EE#..........",
    ".........#EKKE#.........",
    "........#EEWWEE#........",
    "......##CCCCCCCC##......",
    "....##BBCCAAAAACCB##....",
    "...#BBCCAA#E#AAACCCBB#..",
    "..#BBCCAAA###AAAACCCB#..",
    "..#KBBCCCCEEEEECCCCBBK#.",
    ".#EKBBCCCEEKKEEECCCBBE#.",
    "..#bBBBCCCEEEEECCCBBBb#.",
    "...#bbBBBBCCCCCBBBBbb#..",
    "..#CAAbbBBBBBBBBbbAAC#..",
    ".#CCAAAbbBBBBBBbbAAACC#.",
    ".#BCAA#.#bbBBBBbb#.#ACB#",
    "..###...#bBCCCCBb#..###.",
    ".......#bBCCEECCBb#.....",
    "......#bBBCEKKECBBb#....",
    ".....#bBBCCCAACCCBBb#...",
    "....#bBBCCA#..#ACCBBb#..",
    "...#CBBCAA#....#AACBBC#.",
    "..#ACBCA##......##ACBCA#",
    "..#AAA##..........##AAA#",
    "...###..............###.",
];

// 7: LANTERN HERMIT (Heavy spiral-shell crab with glowing angler lantern)
const MON_07_LANTERN_CRAB: &[&str] = &[
    "....#####...............",
    "...#CAAAC#..............",
    "..#CAWWWAC#...######....",
    "..#CAAWAC#..##qqqqqq##..",
    "...#CAAC#.##qqKKKKKKqq#.",
    "....#bb#.#qqKKooooooKKq#",
    "....#bb#.#qKKooqqqqooKq#",
    "...#bBB#.#qKooqKKKKqooKq",
    "...#bB#..#qKoqKKooKKqoKq",
    "..#RRb#..#qKoqKoqqoKqoKq",
    ".#RPPPR##qqKoqKooooKqoKq",
    "#RPP#PPRRqqKKoqKKKKqooKq",
    "#RPP##PRRqqqKKooooooKKq#",
    ".#RRRPRR#bbqqqKKKKKKqq#.",
    "..#####.#bBBqqqqqqqqq#..",
    ".......#bBC#E#CC#E#CBb#.",
    "......#bBBCCCCCCCCCCBBb#",
    ".....#RRbBBBBBBBBBBBBbRR",
    "....#RPPR#bbbbbbbbbb#RPR",
    "....#RPR#.#RR#..#RR#.#RR",
    "...#RPR#..#RP#..#PR#..##",
    "...#RR#..#RPR#..#RPR#...",
    "..#RR#...#RR#....#RR#...",
    "..####...####....####...",
];

// 8: RUNE-CARAPACE (Brass & steel scarab construct with glowing cyan runes)
const MON_08_RUNE_BEETLE: &[&str] = &[
    "........................",
    ".........#E#.#E#........",
    "........#qK#.#Kq#.......",
    "........#qK###Kq#.......",
    "......##dSSSSSSSd##.....",
    "....##ddSSSCCCSSSdd##...",
    "...#ddSSSCCAAACCCSSdd#..",
    "..#ddSS#C#dd#dd#C#SSdd#.",
    "..#qddS###ddCdd###Sddq#.",
    ".#KqddSSSSddAddSSSSddqK#",
    ".#qddSSqqqqdAdqqqqSSddq#",
    "#ddSSqqKKKKdCdKKKKqqSSdd",
    "#dSSqKKoooo#A#ooooKKqSSd",
    "#dSSqKooCCC#A#CCCooKqSSd",
    ".#dSqKoCAAAdAdAAACoKqSd#",
    ".#ddqKKoooo#C#ooooKKqdd#",
    "..#ddqqKKKKdSdKKKKqqdd#.",
    "..#sddqqqqqdSddqqqqdds#.",
    ".#SSd######ddd######dSS#",
    ".#Sd#....#dd#dd#....#dS#",
    "#SSd#...#dd#.#dd#...#dSS",
    "#dd#....#Sd#.#dS#....#dd",
    "#qq#...#qSd#.#dSq#...#qq",
    "####...#####.#####...###",
];

// 9: BOG-IDOL GOLEM (Mossy stone monolith guardian with glowing amber runes)
const MON_09_BOG_GOLEM: &[&str] = &[
    "........................",
    "......############......",
    "....##ggjjGGGGjjgg##....",
    "...#kkggddddddddggkk#...",
    "..#dddddSSSSSSSSddddd#..",
    "..#ddSSddddddddddSSdd#..",
    "..#ddSdd#EE##EE#ddSdd#..",
    ".#ggdddd#KK##KK#ddddgg#.",
    "#jjgddddd######dddddgjj#",
    "#GjgdddSSddEEddSSdddgjG#",
    "#jgddSSddEEWWEEddSSddgj#",
    "#ddSSddddEEWWEEddddSSdd#",
    "#ddSdd.##dEEEEd##.ddSdd#",
    "#dddd#.#dddddddd#.#dddd#",
    "#dSSd#.#ddSSSSdd#.#dSSd#",
    "#qKKq#.#dddddddd#.#qKKq#",
    "#oooo#.#dd####dd#.#oooo#",
    ".####..#dd#..#dd#..####.",
    "......#ddd#..#ddd#......",
    ".....#dSSd#..#dSSd#.....",
    ".....#dddd#..#dddd#.....",
    "....#ggddd#..#dddgg#....",
    "....#kdddd#..#ddddk#....",
    "....#######..#######....",
];

// 10: GLOOM-WISP MOTH (Twilight arcane moth spirit with golden eyespots)
const MON_10_GLOOM_MOTH: &[&str] = &[
    "........................",
    ".......#E#....#E#.......",
    "......#KE#....#EK#......",
    "..####.#V######V#.####..",
    ".#vvvv##pVVVVVVp##vvvv#.",
    "#vVVVVvp#pVAAVpp#pVVVVv#",
    "#vVEEVvvp#C##C#pvvVEEVv#",
    "#vEWWEEvvpVVVVpvvEEWWEv#",
    "#vVEEVVvvppVVppvvVVEEVv#",
    "#vvVVvvuppvvvvppuvvVVvv#",
    ".#vvvvupppvAAvpppuuvvv#.",
    "..####upppvWWvpppuu####.",
    "...##uupppvAAvpppuuu##..",
    "..#vVVupppvvvvpppuVVv#..",
    ".#vVAAvuppvvvvppuvAAVv#.",
    ".#vVvvuu#ppvvpp#uuvvVv#.",
    "..#vvuu#.#pppp#.#uuvv#..",
    "...###...#puup#...###...",
    "........#pp##pp#........",
    "........#p#..#p#........",
    ".......#Vp#..#pV#.......",
    ".......#vp#..#pv#.......",
    "......#AAp#..#pAA#......",
    "......#####..#####......",
];

// 11: CROWN SALAMANDER (Regal arcane drake with orchid flame-crest & gold armor)
const MON_11_CROWN_SALAMANDER: &[&str] = &[
    ".........#V##V#.........",
    "........#VAVVAV#........",
    "......##E#VVVV#E##......",
    ".....#KKE#pVVp#EKK#.....",
    "....#KEppVVVVVVppEK#....",
    "...#ppVVAA#EE#AAVVpp#...",
    "...#pVVAAA####AAAVVp#...",
    "....#ppVVVVVVVVVVpp#....",
    ".....##ppqqKKqqpp##.....",
    "...#V#.#qKKEEEKq#.#V#...",
    "..#VAV##qKEWWEEKq##VAV#.",
    "..#pVpppqKKEEEKKqpppVp#.",
    "...#ppppqqKKKKKqqpppp#..",
    "....##ppuvvvvvvupp##.#V#",
    ".....#puuvVVVVvuup#.#VAV",
    "....#ppuvvVAAVvvupp##Vp#",
    "....#puvvvVVVVvvvupppu#.",
    "....#puuuuvvvvuuuuppu#..",
    ".....#ppu######uupp##...",
    ".....#pVp#....#pVp#.....",
    "....#qKVp#....#pVKq#....",
    "....#EEKp#....#pKEE#....",
    "...#KKEEp#....#pEEKK#...",
    "...#######....#######...",
];

// 12: BOSS DAOT THE ENCHANTER (Hooded amphibian warlock with star-orb staff)
const MON_12_BOSS_DAOT: &[&str] = &[
    "...#AA#...######........",
    "..#CAWAC##ppVVpp##......",
    "..#CAWAC#pVVAAVVp#......",
    "...#CAC#pVVVVVVVVp#.....",
    "....#q##pV#EE##EE#p#....",
    "....#q#ppu#WW##WW#p#....",
    "....#q#ppuutLLtuupp#....",
    "....#q##putLLLLtup##....",
    "...#KqK##ppqqqqpp##.....",
    "...#qqq#ppqKKKKqpp##....",
    "....#q##pVVqEEqVVpp#....",
    "....#q#pVAAvvvvAAVp#....",
    "....#q#tLLpvvvvpLLt#....",
    "....#q##ttpvVVvptt#.....",
    "....#q#.#ppvAAvpp#......",
    "....#q#.#ppvWWvpp#......",
    "....#q#.#ppvAAvpp#......",
    "....#q#.#ppvvvvpp#......",
    "....#q#.#pppVVppp#......",
    "....#q##ppppVVpppp#.....",
    "....#q#ppuuqKKquupp#....",
    "....#q#puuqqEEqquup#....",
    "...#KqK#tttt##tttt#.....",
    "...################.....",
];

// 13: BOSS GROF, MALEVOLENT SORCERER (Horned warlord with twin dark-spore braziers)
const MON_13_BOSS_GROF: &[&str] = &[
    "..#E#..............#E#..",
    ".#KE#....######....#EK#.",
    ".#KE#..##rrRRrr##..#EK#.",
    "..#EE##rrRPVVPRrr##EE#..",
    "...#KKrrR#W##W#RrrKK#...",
    "....##rrR##VV##Rrr##....",
    "..#V#.#rrutLLturr#.#V#..",
    ".#VAV##qqKKKKKKqq##VAV#.",
    ".#vVvvqKKEEVVEEKKqvvVv#.",
    "..#uuu#rrRPVVPRrr#uuu#..",
    "..#dd##rrRPWWPRrr##dd#..",
    ".#dSSd#rrRPVVPRrr#dSSd#.",
    ".#dddd#qKKEEEEEKq#dddd#.",
    "..#tt##qqqKKKKqqq##tt#..",
    "......#rrpuuuuprr#......",
    ".....#rrppuVVuprrr#.....",
    ".....#rpppuAAuppppr#....",
    "....#rrpppuVVupppprr#...",
    "....#rpppuu##uuppprr#...",
    "...#rrppuu#..#uupprrr#..",
    "...#qKKq##....##qKKq#...",
    "...#qEEq#......#qEEq#...",
    "..#ddqqdd#....#ddqqdd#..",
    "..########....########..",
];

// =========================================================================
// SIDE-VIEW POKEMON-STYLE HABITAT & ARCHITECTURE TILES (20x20 1:1)
// =========================================================================

/// Safe Ancient Stone Causeway (`0%` wild encounters!)
fn build_stone_road_tile(variant: usize) -> IndexedSprite {
    let mut t = IndexedSprite::new(TILE_SZ, TILE_SZ);
    for y in 0..TILE_SZ {
        for x in 0..TILE_SZ {
            let seam_y = y == 0 || y == 10 || y == 19;
            let off = if y < 10 {
                (variant * 5) % 10
            } else {
                (variant * 5 + 9) % 18
            };
            let seam_x = x == off || x == (off + 11) % 20;
            let col = if seam_y || seam_x {
                1
            } else if y == 1 {
                6 // Crisp silver-stone top walking surface!
            } else if y == 2 || y == 11 {
                4 // Bevel stone highlight
            } else if y == 9 || y == 18 {
                2 // Recessed stone shadow
            } else if ((x + y + variant) % 7) == 0 {
                4
            } else {
                3
            };
            t.set(x, y, col);
        }
    }
    t
}

/// Deep Subsoil / Ruin Foundation Block (`20x20`)
fn build_earth_sub_tile(variant: usize) -> IndexedSprite {
    let mut t = IndexedSprite::new(TILE_SZ, TILE_SZ);
    for y in 0..TILE_SZ {
        for x in 0..TILE_SZ {
            let root = ((x * 3 + y * 5 + variant * 7) % 19) == 0;
            let pebble = ((x * 7 + y * 11 + variant * 3) % 23) == 0;
            let col = if y == 0 {
                1
            } else if root {
                20
            } else if pebble {
                22
            } else if (x + y) % 2 == 0 {
                19
            } else {
                16
            };
            t.set(x, y, col);
        }
    }
    t
}

/// Climbable Mossy Wood & Iron Ladder (`20x20`)
fn build_ladder_tile() -> IndexedSprite {
    let mut t = IndexedSprite::new(TILE_SZ, TILE_SZ);
    for y in 0..TILE_SZ {
        for x in 3..17 {
            let rail = x == 4 || x == 5 || x == 14 || x == 15;
            let rail_outline = x == 3 || x == 6 || x == 13 || x == 16;
            let rung_y = y % 5;
            let on_rung = (6..=13).contains(&x) && (rung_y == 1 || rung_y == 2);
            let rung_outline = (6..=13).contains(&x) && (rung_y == 0 || rung_y == 3);

            if rail {
                t.set(x, y, if x == 4 || x == 14 { 21 } else { 20 });
            } else if on_rung {
                t.set(x, y, if rung_y == 1 { 41 } else { 20 });
            } else if rail_outline || rung_outline {
                t.set(x, y, 1);
            }
        }
    }
    t
}

/// Rustling Waist-High Fern Underbrush (`20x20` foreground habitat blades - Verdant encounters!)
/// Kept cleanly at rows 10..19 (waist/knee height) so the Blue Alien's visor, eyes & torso stay 100% visible!
fn build_fern_grass_tile(frame: usize) -> IndexedSprite {
    let mut t = IndexedSprite::new(TILE_SZ, TILE_SZ);
    let sway: i32 = if frame == 0 { 0 } else { 1 };
    let blades: [(i32, i32, u8); 6] = [
        (2, 12, 24),
        (5, 10, 25),
        (9, 11, 26),
        (12, 10, 25),
        (15, 11, 24),
        (18, 13, 26),
    ];
    // Ground moss bed at rows 17..19
    for y in 17..20 {
        for x in 0..20 {
            t.set(
                x,
                y,
                if y == 19 {
                    1
                } else if (x + y) % 2 == 0 {
                    22
                } else {
                    23
                },
            );
        }
    }
    for &(bx, top_y, tip_col) in &blades {
        for y in top_y..18 {
            let bend = if y <= top_y + 2 { sway } else { 0 };
            let px = (bx + bend).clamp(1, 18) as usize;
            let py = y as usize;
            let c = if y <= top_y + 1 {
                tip_col
            } else if y <= top_y + 4 {
                24
            } else {
                23
            };
            t.set(px, py, c);
            if y > top_y + 2 && px + 1 < 19 {
                t.set(px + 1, py, 22);
            }
        }
    }
    t.enforce_1px_outline();
    t
}

/// Star-Pond Wading Shallows (`20x20` - Tide encounters!)
fn build_star_water_tile(frame: usize) -> IndexedSprite {
    let mut t = IndexedSprite::new(TILE_SZ, TILE_SZ);
    // Waist-deep wading pool occupying rows 8..19 so boots wade visibly through cyan water!
    for y in 8..20 {
        for x in 0..20 {
            let col = if y == 8 {
                let crest = ((x + frame * 4) % 7) < 3;
                if crest { 35 } else { 34 }
            } else if y == 9 {
                33
            } else if y == 19 {
                1
            } else if ((x + y * 3 + frame * 2) % 11) == 0 {
                34 // Starlight shimmer fleck
            } else if (x + y) % 2 == 0 {
                32
            } else {
                18
            };
            t.set(x, y, col);
        }
    }
    t
}

/// Mycelium & Crystal Grotto Patch (`20x20` - Construct & Arcane encounters!)
fn build_crystal_grotto_tile(frame: usize) -> IndexedSprite {
    let mut t = IndexedSprite::new(TILE_SZ, TILE_SZ);
    for y in 17..20 {
        for x in 0..20 {
            t.set(x, y, if y == 19 { 1 } else if (x + y) % 2 == 0 { 27 } else { 28 });
        }
    }
    let shards: [(usize, usize, usize, u8, u8); 3] = [
        (4, 8, 3, 30, 29),
        (10, 4, 4, 35, 30),
        (15, 9, 3, 30, 28),
    ];
    for &(cx, top_y, hw, hi, mid) in &shards {
        for y in top_y..18 {
            let grow = ((y - top_y) / 3).min(hw / 2);
            for dx in 0..=(grow + 1) {
                if cx >= dx {
                    let c = if dx == 0 && (y == top_y || frame == 1) {
                        7
                    } else if dx == 0 {
                        hi
                    } else {
                        mid
                    };
                    t.set(cx - dx, y, c);
                }
                if cx + dx < 19 {
                    t.set(cx + dx, y, mid);
                }
            }
        }
    }
    t.enforce_1px_outline();
    t
}

/// Golden Alpha Spore-Grove (`20x20` - +2 Lv Alpha Elite encounters!)
fn build_alpha_grove_tile(frame: usize) -> IndexedSprite {
    let mut t = build_fern_grass_tile(frame);
    // Crown the waist-high fern tips with golden & orchid alpha spore-lanterns!
    let orbs: [(usize, usize); 3] = [(5, 10), (12, 9), (17, 12)];
    for &(ox, oy) in &orbs {
        let core = if frame == 0 { 15 } else { 42 };
        t.set(ox, oy, core);
        if ox > 0 {
            t.set(ox - 1, oy, 41);
        }
        if ox + 1 < 20 {
            t.set(ox + 1, oy, 41);
        }
        if oy > 0 {
            t.set(ox, oy - 1, 30);
        }
    }
    t.enforce_1px_outline();
    t
}

/// Supply Cache Chest (`20x20`, grounded strictly on row 19!)
fn build_cache_chest(opened: bool) -> IndexedSprite {
    let mut s = IndexedSprite::new(20, 20);
    let top_y = if opened { 8 } else { 10 };
    for y in top_y..20 {
        for x in 3..17 {
            let border = x == 3 || x == 16 || y == top_y || y == 19 || y == 13;
            let col = if border {
                1
            } else if opened && y < 13 {
                16 // Open hollow interior
            } else if (x == 9 || x == 10) && (12..=15).contains(&y) {
                if opened { 5 } else { 15 } // Gold clasp!
            } else if y < 13 {
                34 // Cyan tech lid
            } else if y == 14 {
                6
            } else {
                4
            };
            s.set(x, y, col);
        }
    }
    s
}

/// Field Med-Beacon (`20x20`, grounded strictly on row 19!)
fn build_med_beacon() -> IndexedSprite {
    let mut s = IndexedSprite::new(20, 20);
    // Base pedestal rows 15..19
    for y in 15..20 {
        for x in 4..16 {
            let c = if y == 15 || y == 19 || x == 4 || x == 15 {
                1
            } else if y == 16 {
                6
            } else {
                4
            };
            s.set(x, y, c);
        }
    }
    // Glowing green/cyan bio-column rows 4..15
    for y in 4..15 {
        for x in 7..13 {
            let c = if x == 7 || x == 12 || y == 4 {
                1
            } else if (x == 9 || x == 10) && (7..=11).contains(&y)
                || (y == 9 && (8..=11).contains(&x))
            {
                7 // Crisp white med-cross!
            } else if x == 8 {
                35
            } else {
                25
            };
            s.set(x, y, c);
        }
    }
    s
}

/// 12x12 Containment Pod Capsule Icon
fn build_containment_pod_12() -> IndexedSprite {
    let rows = [
        "....####....",
        "..##CCCC##..",
        ".#CCAWWACC#.",
        ".#CCCCCCCC#.",
        "#ddddEEdddd#",
        "#dddEWWEdDd#",
        "#ddddEEdddd#",
        ".#OOOOOOOO#.",
        ".#OOYYYYOO#.",
        "..##OOOO##..",
        "....####....",
        "............",
    ];
    compile_ascii_sprite(&rows, 12, 12)
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_all_grounded_sprites_touch_bottom_row() {
        let pal = VgaPalette::new_naibihpma();
        let assets = AssetBank::load_and_compile(&pal);

        // Alien standing & running frames must plant boots strictly on row 19 (zero floating feet!)
        for (label, spr) in [
            ("alien_idle", &assets.alien_idle),
            ("alien_run1", &assets.alien_run1),
            ("alien_run2", &assets.alien_run2),
            ("alien_run3", &assets.alien_run3),
            ("cache_closed", &assets.spr_cache_closed),
            ("med_beacon", &assets.spr_med_beacon),
        ] {
            let bottom_y = spr.h - 1;
            let has_bottom_pixel = (0..spr.w).any(|x| spr.get(x, bottom_y) != 0);
            assert!(
                has_bottom_pixel,
                "{} must have grounded pixels on bottom row {}",
                label, bottom_y
            );
        }

        // Every single 24x24 monster sprite (0..14) must touch bottom row 23!
        for (idx, mon) in assets.monsters.iter().enumerate() {
            let bottom_y = mon.h - 1;
            let has_bottom_pixel = (0..mon.w).any(|x| mon.get(x, bottom_y) != 0);
            assert!(
                has_bottom_pixel,
                "Monster sprite #{} must touch bottom row {}",
                idx, bottom_y
            );
        }
    }
}
