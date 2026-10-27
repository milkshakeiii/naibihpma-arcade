use crate::audio::{ArcadeAudio, Sfx};
use crate::framebuffer::{Framebuffer, FB_H, FB_W};
use crate::palette::VgaPalette;
use crate::sprites::{AssetBank, TILE_SZ};

pub const PLAY_H: i32 = 236;
pub const WORLD_COLS: usize = 80;
pub const WORLD_ROWS: usize = 22;

const TS: i32 = TILE_SZ as i32; // 24x24 block size
const COSMO_W: i32 = 40;
const COSMO_H: i32 = 44;

const FALL_ARC: &[i32] = &[4, 6, 8, 10, 12, 12];

// Barstool X positions in the 400x236 quantized Naibihpma Galley interior
pub const STOOL_X: [i32; 4] = [194, 234, 274, 314];
const PASS_X: i32 = 136; // Kitchen Pass counter X where finished dishes appear

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum GameMode {
    TitleAttractor,
    PlanetaryDrop,
    MenuPrep,      // Pre-Service Dish Combiner (Combine ingredients into Tonight's Menu ahead of time!)
    GalleyService, // Dave the Diver-style Side-View Tavern Service!
    UpgradeShop,   // Purchase Restaurant & Foraging Rig Upgrades with your earnings!
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Tile {
    Empty,
    EarthTop,
    EarthFill,
    TrunkWall,
    LilyPad,
    GrappleAnchor, // Glowing overhead Grapple-Root Ring for Harpoon Winching!
    WaterSurface,  // Shimmering cyan Star-Pond surface meniscus
    WaterDeep,     // Flooded bioluminescent Star-Pond basin (8-Way Buoyant Swimming!)
    DropPod,       // Submersible Extraction Bell
    ShadowAlcove,  // Hollow-bark concealment alcove (hides player from Newts & Toad Orbs!)
    SporeVent,     // Rhythmic toxic spore-geyser fissure (vents O2 if touched while erupting!)
    OxygenReed,    // Bioluminescent bubble-reed cluster (+30% Suit O2 refill on contact!)
    SluiceLever,   // Interactive world-state puzzle switch (hit with [X] Harpoon!)
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum IngKind {
    DewNewt = 0,    // Twen the Dew-Newt (LUMINOUS)
    VocalRoe = 1,   // Doat's Golden Vocal-Roe (RICH-ROE)
    SporeLotus = 2, // Violet Spore-Lotus (SPORE-UMAMI)
    ReedEel = 3,    // Voltaic Reed-Eel & Pepper (VOLTAIC-SPICE)
}

impl IngKind {
    pub fn name(self) -> &'static str {
        match self {
            IngKind::DewNewt => "DEW-NEWT",
            IngKind::VocalRoe => "VOCAL-ROE",
            IngKind::SporeLotus => "SPORE-LOTUS",
            IngKind::ReedEel => "REED-EEL",
        }
    }

    /// Steady, non-flashing palette color for each ingredient tag
    pub fn steady_color(self) -> u8 {
        match self {
            IngKind::DewNewt => 34,    // Steady Bright Aqua
            IngKind::VocalRoe => 41,   // Steady Roe Gold
            IngKind::SporeLotus => 30, // Steady Orchid Violet
            IngKind::ReedEel => 38,    // Steady Coral Crimson
        }
    }

    pub fn base_value(self) -> i32 {
        match self {
            IngKind::DewNewt => 45,
            IngKind::VocalRoe => 55,
            IngKind::SporeLotus => 60,
            IngKind::ReedEel => 70,
        }
    }

    pub fn weight_kg(self) -> i32 {
        match self {
            IngKind::DewNewt => 2,
            IngKind::VocalRoe => 3,
            IngKind::SporeLotus => 2,
            IngKind::ReedEel => 4,
        }
    }
}

/// Harvest Grade determined by HOW you capture the ingredient in the field!
/// - `Fresh`      = 1* BRUISED (Punctured delicate prey with raw harpoon)
/// - `Tenderized` = 2* HARPOON (Clean harpoon reel on tough fauna/flora)
/// - `CharSeared` = 3* PRISTINE (Captured intact via Bait-Net Trap [C] or Tranq Dart [Z]!)
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum PrepState {
    Fresh,      // 1* BRUISED
    Tenderized, // 2* HARPOON
    CharSeared, // 3* PRISTINE
}

impl PrepState {
    pub fn label(self) -> &'static str {
        match self {
            PrepState::Fresh => "1* BRUISED",
            PrepState::Tenderized => "2* HARPOON",
            PrepState::CharSeared => "3* PRISTINE",
        }
    }

    pub fn star_bonus(self) -> usize {
        match self {
            PrepState::Fresh => 0,
            PrepState::Tenderized => 1,
            PrepState::CharSeared => 2,
        }
    }
}

#[derive(Clone, Copy)]
pub struct BagItem {
    pub kind: IngKind,
    pub prep: PrepState,
    pub ionized: bool,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum CreatureBehavior {
    SessileFlora, // Spore-Lotus: easy harpoon target
    SkittishNewt, // Twen Dew-Newt: hides in burrow if approached fast; loves Bait-Net Trap [C]!
    ArmoredToad,  // Doat Baron: frontal shield blocks harpoon; put to sleep with Tranq Dart [Z]!
    VoltaicEel,   // Reed-Eel: swims in Star-Pond & pulses electric arcs; Tranq [Z] or wait for cooldown!
}

#[derive(Clone)]
pub struct FieldEntity {
    pub kind: IngKind,
    pub behavior: CreatureBehavior,
    pub home_x: i32,
    pub home_y: i32,
    pub x: i32,
    pub y: i32,
    pub vx: i32,
    pub sleep_ticks: i32,   // > 0 when tranquilized by [Z]/[SPACE] Spore-Dart!
    pub burrowed: bool,     // True when skittish Newt hides in its root-burrow
    pub shock_phase: i32,   // > 30 when Voltaic Eel is discharging electricity
    pub alive: bool,
}

#[derive(Clone)]
pub struct TranqDart {
    pub x: i32,
    pub y: i32,
    pub vx: i32,
    pub vy: i32,
    pub life: i32,
}

#[derive(Clone)]
pub struct SporeOrb {
    pub x: i32,
    pub y: i32,
    pub vx: i32,
    pub vy: i32,
    pub life: i32,
}

#[derive(Clone)]
pub struct BaitTrap {
    pub x: i32,
    pub y: i32,
    pub snapped: bool,
    pub timer: i32,
}

#[derive(Clone)]
pub struct Particle {
    pub x: i32,
    pub y: i32,
    pub vx: i32,
    pub vy: i32,
    pub col: u8,
    pub life: i32,
}

#[derive(Clone)]
pub struct FloatingText {
    pub text: String,
    pub x: i32,
    pub y: i32,
    pub col: u8,
    pub life: i32,
}

/// A dish combined ahead of time on the Pre-Service Menu Board
#[derive(Clone)]
pub struct CuratedDish {
    pub name: String,
    pub primary_kind: IngKind,
    pub secondary_kind: IngKind,
    pub price: i32,
    pub cook_ticks_needed: i32,
    pub stars: usize, // 1..5 quality stars
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum PatronStage {
    WalkingIn,
    SeatedOrdering,
    Eating(i32),
    SatisfiedLeaving,
}

/// A patron in the side-view Dave the Diver-style bog tavern
#[derive(Clone)]
pub struct TavernPatron {
    pub stool_idx: usize,
    pub x: i32,
    pub stage: PatronStage,
    pub ordered_dish: CuratedDish,
    pub wants_tea: bool,
    pub tea_served: bool,
    pub patience: i32,
    pub max_patience: i32,
}

/// Restaurant & Foraging Rig Upgrades that determine hunting & serving success
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum RestaurantUpgrade {
    TwinBurnerHearth, // Cooks combined dishes 2x faster at the Kitchen Pass
    SousChefMastery,  // Sous-Chef adds +45% Dish Price & +1 Star to all dishes
    AutoTeaSporeUrn,  // Auto-pours Spore-Tea (+$45 tip) & +60% Patron Patience
    SuctionDashBoots, // Silk-Winch & Dive Fins: +50% Dive/Bar Speed & Harpoon never bruises!
}

impl RestaurantUpgrade {
    pub fn all() -> [RestaurantUpgrade; 4] {
        [
            RestaurantUpgrade::TwinBurnerHearth,
            RestaurantUpgrade::SousChefMastery,
            RestaurantUpgrade::AutoTeaSporeUrn,
            RestaurantUpgrade::SuctionDashBoots,
        ]
    }

    pub fn title(self) -> &'static str {
        match self {
            RestaurantUpgrade::TwinBurnerHearth => "TWIN-BURNER COPPER HEARTH",
            RestaurantUpgrade::SousChefMastery => "SOUS-CHEF GLAZE MASTERY",
            RestaurantUpgrade::AutoTeaSporeUrn => "AUTO SPORE-TEA URN & LAMPS",
            RestaurantUpgrade::SuctionDashBoots => "SILK-HARPOON & DIVE FINS",
        }
    }

    pub fn cost(self) -> i32 {
        match self {
            RestaurantUpgrade::TwinBurnerHearth => 120,
            RestaurantUpgrade::SousChefMastery => 160,
            RestaurantUpgrade::AutoTeaSporeUrn => 140,
            RestaurantUpgrade::SuctionDashBoots => 110,
        }
    }

    pub fn desc(self) -> &'static str {
        match self {
            RestaurantUpgrade::TwinBurnerHearth => "KITCHEN PASS COOKS COMBINED DISHES 2X FASTER",
            RestaurantUpgrade::SousChefMastery => "SOUS-CHEF ADDS +45% SYNERGY PRICE TO ALL DISHES",
            RestaurantUpgrade::AutoTeaSporeUrn => "AUTO-POURS SPORE-TEA (+$45 TIP) & +60% PATIENCE",
            RestaurantUpgrade::SuctionDashBoots => "+50% SWIM/BAR SPEED & HARPOON NEVER BRUISES PREY",
        }
    }
}

#[derive(Default, Clone, Copy)]
pub struct InputState {
    pub left: bool,
    pub right: bool,
    pub up: bool,
    pub down: bool,
    pub left_pressed: bool,
    pub right_pressed: bool,
    pub up_pressed: bool,
    pub down_pressed: bool,
    pub jump_pressed: bool,
    pub jump_held: bool,
    pub action_pressed: bool,
    pub bomb_pressed: bool,
    pub extract_pressed: bool,
    pub help_pressed: bool,
    pub warp_step: Option<usize>,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum AirborneMode {
    Grounded,
    Falling(usize),
    CrawlingOver {
        start_x: i32,
        start_y: i32,
        target_x: i32,
        target_y: i32,
        step: i32,
        total: i32,
    },
    GapLeap {
        start_x: i32,
        start_y: i32,
        target_x: i32,
        target_y: i32,
        step: i32,
        total: i32,
    },
    GrappleWinch {
        target_x: i32,
        target_y: i32,
        anchor_x: i32,
        anchor_y: i32,
    },
    Swimming, // 8-Way Buoyant Dave-the-Diver Swimming in the Flooded Star-Pond!
}

pub struct GameState {
    pub mode: GameMode,
    pub tick: u64,
    pub stage_num: i32,
    pub gold: i32,
    pub crt_scanlines: bool,
    pub show_help_overlay: bool,

    // Purchased Restaurant & Rig Upgrades
    pub upgrades: Vec<RestaurantUpgrade>,
    pub shop_cursor: usize,

    // Phase 1: Grounded Grapple-Hunter + Submerged Star-Pond Diver
    pub tiles: [[Tile; WORLD_COLS]; WORLD_ROWS],
    pub px: i32,
    pub py: i32,
    pub facing_right: bool,
    pub moving_x: bool,
    pub aim_dx: i32,
    pub aim_dy: i32,
    pub walk_anim_tick: u32,
    pub air_mode: AirborneMode,
    pub pluck_timer: i32,
    pub tether_target: Option<(i32, i32)>,
    pub cam_x: i32,
    pub cam_y: i32,
    pub suit_o2: i32, // Suit O2 / Health & Timer (0..100)
    pub tranq_darts_left: i32,
    pub portrait_mood: u8,
    pub mood_timer: i32,
    pub hurt_cooldown: i32,
    pub world_switch_pulled: bool,
    pub blackout_occurred: bool,

    pub entities: Vec<FieldEntity>,
    pub darts: Vec<TranqDart>,
    pub orbs: Vec<SporeOrb>,
    pub active_trap: Option<BaitTrap>,
    pub backpack: Vec<BagItem>,

    // Phase 2A: Pre-Service Dish Combiner (MenuPrep)
    pub prep_cursor: usize,
    pub selected_first_ing: Option<usize>,
    pub menu_dishes: Vec<CuratedDish>,

    // Phase 2B: Dave the Diver-style Side-View Tavern Floor (GalleyService)
    pub chef_bar_x: i32, // Cosmo's X position along the tavern floor (124..340)
    pub chef_facing_right: bool,
    pub chef_moving: bool,
    pub cooking_queue: Vec<CuratedDish>,
    pub active_cook_dish: Option<CuratedDish>,
    pub cook_progress: i32,
    pub ready_pass_dishes: Vec<CuratedDish>, // Finished dishes sitting on the Kitchen Pass counter
    pub carried_dish: Option<CuratedDish>,   // Dish currently carried in Cosmo's hands
    pub patrons: Vec<TavernPatron>,
    pub night_earnings: i32,
    pub dishes_served_tonight: usize,

    pub particles: Vec<Particle>,
    pub popups: Vec<FloatingText>,
}

impl GameState {
    pub fn new() -> Self {
        let mut gs = Self {
            mode: GameMode::TitleAttractor,
            tick: 0,
            stage_num: 1,
            gold: 120,
            crt_scanlines: true,
            show_help_overlay: false,
            upgrades: Vec::new(),
            shop_cursor: 0,
            tiles: [[Tile::Empty; WORLD_COLS]; WORLD_ROWS],
            px: 48,
            py: 12 * TS - COSMO_H,
            facing_right: true,
            moving_x: false,
            aim_dx: 1,
            aim_dy: 0,
            walk_anim_tick: 0,
            air_mode: AirborneMode::Grounded,
            pluck_timer: 0,
            tether_target: None,
            cam_x: 0,
            cam_y: 80,
            suit_o2: 100,
            tranq_darts_left: 9,
            portrait_mood: 0,
            mood_timer: 0,
            hurt_cooldown: 0,
            world_switch_pulled: false,
            blackout_occurred: false,
            entities: Vec::new(),
            darts: Vec::new(),
            orbs: Vec::new(),
            active_trap: None,
            backpack: Vec::new(),
            prep_cursor: 0,
            selected_first_ing: None,
            menu_dishes: Vec::new(),
            chef_bar_x: 156,
            chef_facing_right: true,
            chef_moving: false,
            cooking_queue: Vec::new(),
            active_cook_dish: None,
            cook_progress: 0,
            ready_pass_dishes: Vec::new(),
            carried_dish: None,
            patrons: Vec::new(),
            night_earnings: 0,
            dishes_served_tonight: 0,
            particles: Vec::new(),
            popups: Vec::new(),
        };
        gs.build_stage_for_current_level();
        gs
    }

    pub fn has_upgrade(&self, u: RestaurantUpgrade) -> bool {
        self.upgrades.contains(&u)
    }

    /// Metroidvania progression: Overhead Grapple-Winch & Tranq Darts unlock in Level 2 (or via Silk-Harpoon upgrade)
    pub fn has_grapple_and_tranq(&self) -> bool {
        self.stage_num >= 2 || self.has_upgrade(RestaurantUpgrade::SuctionDashBoots)
    }

    pub fn cargo_weight_kg(&self) -> i32 {
        self.backpack.iter().map(|b| b.kind.weight_kg()).sum()
    }

    pub fn max_cargo_kg(&self) -> i32 {
        if self.has_upgrade(RestaurantUpgrade::SuctionDashBoots) {
            24
        } else {
            16
        }
    }

    fn fill_ground(&mut self, c0: usize, c1: usize, r_top: usize) {
        for c in c0..=c1.min(WORLD_COLS - 1) {
            if r_top < WORLD_ROWS {
                self.tiles[r_top][c] = Tile::EarthTop;
            }
            for r in (r_top + 1)..WORLD_ROWS {
                self.tiles[r][c] = Tile::EarthFill;
            }
        }
    }

    fn fill_platform(&mut self, c0: usize, c1: usize, r_top: usize, thickness: usize) {
        for c in c0..=c1.min(WORLD_COLS - 1) {
            if r_top < WORLD_ROWS {
                self.tiles[r_top][c] = Tile::EarthTop;
            }
            for r in (r_top + 1)..(r_top + thickness).min(WORLD_ROWS) {
                self.tiles[r][c] = Tile::EarthFill;
            }
        }
    }

    pub fn build_tutorial_stage(&mut self) {
        self.build_stage_for_current_level();
    }

    pub fn build_stage_for_current_level(&mut self) {
        self.tiles = [[Tile::Empty; WORLD_COLS]; WORLD_ROWS];
        self.entities.clear();
        self.darts.clear();
        self.orbs.clear();
        self.active_trap = None;
        self.particles.clear();
        self.popups.clear();
        self.world_switch_pulled = false;
        self.blackout_occurred = false;
        self.hurt_cooldown = 0;

        if self.stage_num <= 1 {
            self.build_level_1_underbrush();
        } else {
            self.build_level_2_daot_enchanter();
        }
    }

    /// LEVEL 1: THE UNDERBRUSH (Act I, Track 1 - 05:52)
    /// Focus: Stealth Walk, Auto-Crawl, Auto-Leap, Shadow Alcove Concealment,
    /// Rhythmic Spore-Geysers, [X] Harpoon, [C] Lure-Trap, Bubble-Reed O2 Economy,
    /// and the Mutually-Exclusive Sluice Gate Ravine Puzzle!
    pub fn build_level_1_underbrush(&mut self) {
        // =========================================================================
        // ZONE 1 (cols 0..18): Mossy Root Terraces, Spore-Geysers & First Lotus
        // =========================================================================
        self.fill_ground(0, 11, 12);
        self.fill_ground(6, 6, 11); // 1-tile mossy bog-rock outcrop (auto-crawled!)
        self.tiles[12][9] = Tile::SporeVent; // Rhythmic toxic spore-geyser at col 9!
        self.fill_ground(12, 13, 16); // 2-tile open root chasm (auto-leaped across!)
        self.fill_ground(14, 18, 12);

        // =========================================================================
        // ZONE 2 (cols 19..35): Hollow-Bark Shadow Alcoves & Skittish Newt Burrow
        // =========================================================================
        self.fill_ground(19, 19, 11); // Step up onto root terrace
        self.fill_ground(20, 35, 10);
        // Bubble-Reed O2 Vent at col 21 (+30% O2!)
        self.tiles[9][21] = Tile::OxygenReed;
        // Two Hollow-Bark Shadow Alcoves at cols 23..24 and cols 30..31 (conceals player!)
        for r in 8..=9 {
            for c in [23, 24, 30, 31] {
                self.tiles[r][c] = Tile::ShadowAlcove;
            }
        }
        // Sluice Gate Lever at col 35, row 9 overlooking the Sunken Ravine!
        self.tiles[9][35] = Tile::SluiceLever;

        // =========================================================================
        // ZONE 3 (cols 36..59): The Sunken Ravine & Mutually-Exclusive Sluice Puzzle
        // - Default (Unflooded): Descend wide 2-tile steps (cols 36..41) to the dry Ravine Floor
        //   (cols 42..49, row 18) to trap the Deep Burrow Newt (col 44) & Deep Lotus (col 48),
        //   then climb the eastern steps (cols 50..53) to the Lower Pier (cols 54..59, row 15).
        // - Pulled Lever ([X] on SluiceLever at col 35): Floods cols 36..52 (rows 11..17)
        //   with 7-tile-deep Star-Pond water! Drowns the dry-floor burrow, awakens a Voltaic Eel,
        //   and lets you swim straight across to clamber onto the High Canopy Vault (row 11)!
        // =========================================================================
        self.fill_ground(36, 37, 12);
        self.fill_ground(38, 39, 14);
        self.fill_ground(40, 41, 16);
        self.fill_ground(42, 49, 18); // Dry Ravine Floor at row 18
        self.tiles[18][46] = Tile::SporeVent; // Ravine floor spore-geyser!
        self.fill_ground(50, 51, 17);
        self.fill_ground(52, 53, 16);
        self.fill_ground(54, 58, 15); // Lower Eastern Pier at row 15
        self.fill_ground(59, 59, 13); // Step connecting to Zone 4 Star-Basin

        // High Canopy Vault suspended above the eastern side of the ravine (cols 53..58, row 11)
        // Leaves 3 full tiles (72px) of empty headroom (rows 12..14) above the row 15 Lower Pier!
        self.fill_platform(53, 58, 11, 1);
        self.tiles[10][55] = Tile::OxygenReed;
        self.tiles[8][53] = Tile::GrappleAnchor; // Teases Metroidvania Grapple Ring for Level 2!

        // =========================================================================
        // ZONE 4 (cols 60..79): The Underbrush Star-Basin & Extraction Bell
        // =========================================================================
        self.fill_ground(60, 77, 20);
        self.fill_ground(78, 79, 14);
        for c in 60..=77 {
            self.tiles[14][c] = Tile::WaterSurface;
            for r in 15..=19 {
                self.tiles[r][c] = Tile::WaterDeep;
            }
        }
        self.tiles[17][63] = Tile::LilyPad;
        self.tiles[17][64] = Tile::LilyPad;
        self.tiles[19][65] = Tile::OxygenReed; // Underwater Bubble-Reed O2 Vent!

        // Submersible Extraction Bell at cols 75..76, row 18
        self.tiles[18][75] = Tile::DropPod;
        self.tiles[18][76] = Tile::DropPod;

        // =========================================================================
        // LEVEL 1 ECOLOGICAL FAUNA & FLORA
        // =========================================================================
        // 1. Zone 1: Spore-Lotus across the root gap (col 15, row 12)
        self.entities.push(FieldEntity {
            kind: IngKind::SporeLotus,
            behavior: CreatureBehavior::SessileFlora,
            home_x: 15 * TS + 4,
            home_y: 12 * TS - 34,
            x: 15 * TS + 4,
            y: 12 * TS - 34,
            vx: 0,
            sleep_ticks: 0,
            burrowed: false,
            shock_phase: 0,
            alive: true,
        });

        // 2. Zone 2: Skittish Twen Dew-Newt between the two Shadow Alcoves (col 27, row 10)
        self.entities.push(FieldEntity {
            kind: IngKind::DewNewt,
            behavior: CreatureBehavior::SkittishNewt,
            home_x: 27 * TS,
            home_y: 10 * TS - 34,
            x: 27 * TS,
            y: 10 * TS - 34,
            vx: -2,
            sleep_ticks: 0,
            burrowed: false,
            shock_phase: 0,
            alive: true,
        });

        // 3. Zone 3 (Route A - Dry Ravine Floor): Subterranean Twen Dew-Newt (col 43, row 18)
        self.entities.push(FieldEntity {
            kind: IngKind::DewNewt,
            behavior: CreatureBehavior::SkittishNewt,
            home_x: 43 * TS,
            home_y: 18 * TS - 34,
            x: 43 * TS,
            y: 18 * TS - 34,
            vx: 2,
            sleep_ticks: 0,
            burrowed: false,
            shock_phase: 0,
            alive: true,
        });

        // 4. Zone 3 (Route A - Dry Ravine Floor): Deep Ravine Spore-Lotus (col 48, row 18)
        self.entities.push(FieldEntity {
            kind: IngKind::SporeLotus,
            behavior: CreatureBehavior::SessileFlora,
            home_x: 48 * TS,
            home_y: 18 * TS - 34,
            x: 48 * TS,
            y: 18 * TS - 34,
            vx: 0,
            sleep_ticks: 0,
            burrowed: false,
            shock_phase: 0,
            alive: true,
        });

        // 5. Zone 3 (Route B - High Canopy Vault): Golden Vocal-Roe Sac (col 56, row 11)
        self.entities.push(FieldEntity {
            kind: IngKind::VocalRoe,
            behavior: CreatureBehavior::SessileFlora,
            home_x: 56 * TS,
            home_y: 11 * TS - 34,
            x: 56 * TS,
            y: 11 * TS - 34,
            vx: 0,
            sleep_ticks: 0,
            burrowed: false,
            shock_phase: 0,
            alive: true,
        });

        // 6. Zone 4 (Star-Basin Underwater): Patroling Voltaic Reed-Eel (col 68, row 16)
        self.entities.push(FieldEntity {
            kind: IngKind::ReedEel,
            behavior: CreatureBehavior::VoltaicEel,
            home_x: 68 * TS,
            home_y: 16 * TS,
            x: 68 * TS,
            y: 16 * TS,
            vx: 3,
            sleep_ticks: 0,
            burrowed: false,
            shock_phase: 0,
            alive: true,
        });

        // 7. Zone 4 (Star-Basin Floor): Deep Spore-Lotus (col 72, row 20)
        self.entities.push(FieldEntity {
            kind: IngKind::SporeLotus,
            behavior: CreatureBehavior::SessileFlora,
            home_x: 72 * TS,
            home_y: 20 * TS - 34,
            x: 72 * TS,
            y: 20 * TS - 34,
            vx: 0,
            sleep_ticks: 0,
            burrowed: false,
            shock_phase: 0,
            alive: true,
        });

        self.px = 2 * TS;
        self.py = 12 * TS - COSMO_H;
        self.facing_right = true;
        self.moving_x = false;
        self.aim_dx = 1;
        self.aim_dy = 0;
        self.air_mode = AirborneMode::Grounded;
        self.suit_o2 = 100;
        self.tranq_darts_left = if self.has_grapple_and_tranq() { 9 } else { 0 };
        self.update_camera_instant();
    }

    /// LEVEL 2: DAOT THE ENCHANTER (Act I, Track 2 - 04:30)
    /// Unlocks: [UP]+[X] Overhead Grapple-Winch & [Z]/[SPACE] Soporific Tranq Darts!
    /// Threats: Daot's Enchanter Toads firing hypnotic Spore-Orbs on the marimba pulse!
    /// Puzzle: The Counterweight Drawbridge & Sunken Star-Crypt (High Canopy Grapple vs. Flooded Crypt)!
    pub fn build_level_2_daot_enchanter(&mut self) {
        // =========================================================================
        // ZONE 1 (cols 0..17): Enchanter's Causeway, Spore-Orb Ambush & Grapple Cliff
        // =========================================================================
        self.fill_ground(0, 16, 14);
        self.fill_ground(4, 4, 13); // Mossy step
        // Shadow Alcove at cols 7..8 so player can duck Daot's hypnotic Spore-Orbs!
        for r in 12..=13 {
            self.tiles[r][7] = Tile::ShadowAlcove;
            self.tiles[r][8] = Tile::ShadowAlcove;
        }
        // Tall vertical cliff from row 14 up to row 8 at col 17!
        self.fill_ground(17, 37, 8);
        for r in 9..14 {
            self.tiles[r][17] = Tile::TrunkWall;
        }
        // Glowing Overhead Grapple-Root Anchor Rings at (cols 16..17, row 6)!
        self.tiles[6][16] = Tile::GrappleAnchor;
        self.tiles[6][17] = Tile::GrappleAnchor;

        // =========================================================================
        // ZONE 2 (cols 18..37): Suspended Root-Plateau, Spore-Geyser & Enchanter Guard
        // =========================================================================
        self.tiles[7][20] = Tile::OxygenReed; // O2 Bubble-Reed refill!
        self.tiles[8][23] = Tile::SporeVent;  // Rhythmic Spore-Geyser
        for r in 6..=7 {
            self.tiles[r][26] = Tile::ShadowAlcove;
            self.tiles[r][27] = Tile::ShadowAlcove;
        }
        // Counterweight Drawbridge Lever at col 37, row 7!
        self.tiles[7][37] = Tile::SluiceLever;

        // =========================================================================
        // ZONE 3 (cols 38..58): Counterweight Drawbridge & Sunken Star-Crypt Puzzle
        // - Route A (Default / Unpulled): Use Overhead Grapple Rings at (col 40, row 5),
        //   (col 45, row 5), and (col 50, row 5) to winch across the high Canopy Sanctum
        //   (cols 51..57, row 7) for a pristine Dew-Newt & Spore-Lotus!
        // - Route B (Pull Counterweight Lever [X] at col 37):
        //   Seals the Upper Canopy Gate at col 51, drops the Counterweight Timber Bridge
        //   across cols 38..45 at row 12, and opens the Sunken Star-Crypt (cols 46..57, rows 14..20)!
        // =========================================================================
        // High Canopy Sanctum platforms & Grapple Rings (Route A)
        self.tiles[5][40] = Tile::GrappleAnchor;
        self.fill_platform(41, 43, 7, 1);
        self.tiles[5][45] = Tile::GrappleAnchor;
        self.fill_platform(46, 48, 7, 1);
        self.tiles[5][50] = Tile::GrappleAnchor;
        self.fill_platform(51, 57, 7, 2);
        self.tiles[6][55] = Tile::OxygenReed;

        // Lower Chasm & Sunken Star-Crypt (Route B - sealed until Counterweight Lever is pulled,
        // or dropped into from col 38)
        self.fill_ground(38, 45, 16); // Chasm floor before drawbridge drops
        self.fill_ground(46, 57, 20); // Deep Star-Crypt floor at row 20
        // By default, a heavy stone counterweight slab seals row 12 across cols 46..57
        self.fill_platform(46, 57, 12, 2);
        for c in 46..=57 {
            self.tiles[14][c] = Tile::WaterSurface;
            for r in 15..=19 {
                self.tiles[r][c] = Tile::WaterDeep;
            }
        }
        self.tiles[19][49] = Tile::OxygenReed;

        // =========================================================================
        // ZONE 4 (cols 58..79): Daot's Sanctum Pier & Deep Extraction Bell
        // =========================================================================
        self.fill_ground(58, 62, 12);
        self.tiles[9][58] = Tile::GrappleAnchor;
        for r in 10..=11 {
            self.tiles[r][60] = Tile::ShadowAlcove;
        }
        self.fill_ground(63, 77, 20);
        self.fill_ground(78, 79, 12);
        for c in 63..=77 {
            self.tiles[13][c] = Tile::WaterSurface;
            for r in 14..=19 {
                self.tiles[r][c] = Tile::WaterDeep;
            }
        }
        self.tiles[16][66] = Tile::LilyPad;
        self.tiles[16][67] = Tile::LilyPad;
        self.tiles[19][68] = Tile::OxygenReed;

        // Submersible Extraction Bell at cols 75..76, row 18
        self.tiles[18][75] = Tile::DropPod;
        self.tiles[18][76] = Tile::DropPod;

        // =========================================================================
        // LEVEL 2 ECOLOGICAL FAUNA & ENCHANTER TOADS
        // =========================================================================
        // 1. Zone 1: Armored Daot Enchanter Toad firing Spore-Orbs down the causeway (col 12, row 14)
        self.entities.push(FieldEntity {
            kind: IngKind::VocalRoe,
            behavior: CreatureBehavior::ArmoredToad,
            home_x: 12 * TS,
            home_y: 14 * TS - 32,
            x: 12 * TS,
            y: 14 * TS - 32,
            vx: 0,
            sleep_ticks: 0,
            burrowed: false,
            shock_phase: 10,
            alive: true,
        });

        // 2. Zone 2: Skittish Twen Dew-Newt on the Upper Root-Plateau (col 30, row 8)
        self.entities.push(FieldEntity {
            kind: IngKind::DewNewt,
            behavior: CreatureBehavior::SkittishNewt,
            home_x: 30 * TS,
            home_y: 8 * TS - 34,
            x: 30 * TS,
            y: 8 * TS - 34,
            vx: -2,
            sleep_ticks: 0,
            burrowed: false,
            shock_phase: 0,
            alive: true,
        });

        // 3. Zone 2: Armored Enchanter Toad guarding the Counterweight Lever (col 35, row 8)
        self.entities.push(FieldEntity {
            kind: IngKind::VocalRoe,
            behavior: CreatureBehavior::ArmoredToad,
            home_x: 35 * TS,
            home_y: 8 * TS - 32,
            x: 35 * TS,
            y: 8 * TS - 32,
            vx: 0,
            sleep_ticks: 0,
            burrowed: false,
            shock_phase: 25,
            alive: true,
        });

        // 4. Zone 3 (Route A - High Canopy Sanctum): Canopy Dew-Newt (col 53, row 7)
        self.entities.push(FieldEntity {
            kind: IngKind::DewNewt,
            behavior: CreatureBehavior::SkittishNewt,
            home_x: 53 * TS,
            home_y: 7 * TS - 34,
            x: 53 * TS,
            y: 7 * TS - 34,
            vx: 2,
            sleep_ticks: 0,
            burrowed: false,
            shock_phase: 0,
            alive: true,
        });

        // 5. Zone 3 (Route A - High Canopy Sanctum): Canopy Spore-Lotus (col 56, row 7)
        self.entities.push(FieldEntity {
            kind: IngKind::SporeLotus,
            behavior: CreatureBehavior::SessileFlora,
            home_x: 56 * TS,
            home_y: 7 * TS - 34,
            x: 56 * TS,
            y: 7 * TS - 34,
            vx: 0,
            sleep_ticks: 0,
            burrowed: false,
            shock_phase: 0,
            alive: true,
        });

        // 6. Zone 3 (Route B - Sunken Star-Crypt): Crypt Voltaic Reed-Eel (col 51, row 17)
        self.entities.push(FieldEntity {
            kind: IngKind::ReedEel,
            behavior: CreatureBehavior::VoltaicEel,
            home_x: 51 * TS,
            home_y: 17 * TS,
            x: 51 * TS,
            y: 17 * TS,
            vx: 3,
            sleep_ticks: 0,
            burrowed: false,
            shock_phase: 0,
            alive: true,
        });

        // 7. Zone 4 (Sanctum Pier & Basin): Sanctum Enchanter Toad (col 61, row 12)
        self.entities.push(FieldEntity {
            kind: IngKind::VocalRoe,
            behavior: CreatureBehavior::ArmoredToad,
            home_x: 61 * TS,
            home_y: 12 * TS - 32,
            x: 61 * TS,
            y: 12 * TS - 32,
            vx: 0,
            sleep_ticks: 0,
            burrowed: false,
            shock_phase: 0,
            alive: true,
        });

        // 8. Zone 4 (Sanctum Basin Underwater): Deep Voltaic Reed-Eel (col 69, row 16)
        self.entities.push(FieldEntity {
            kind: IngKind::ReedEel,
            behavior: CreatureBehavior::VoltaicEel,
            home_x: 69 * TS,
            home_y: 16 * TS,
            x: 69 * TS,
            y: 16 * TS,
            vx: -3,
            sleep_ticks: 0,
            burrowed: false,
            shock_phase: 15,
            alive: true,
        });

        // 9. Zone 4 (Sanctum Basin Floor): Sanctum Spore-Lotus (col 72, row 20)
        self.entities.push(FieldEntity {
            kind: IngKind::SporeLotus,
            behavior: CreatureBehavior::SessileFlora,
            home_x: 72 * TS,
            home_y: 20 * TS - 34,
            x: 72 * TS,
            y: 20 * TS - 34,
            vx: 0,
            sleep_ticks: 0,
            burrowed: false,
            shock_phase: 0,
            alive: true,
        });

        self.px = 2 * TS;
        self.py = 14 * TS - COSMO_H;
        self.facing_right = true;
        self.moving_x = false;
        self.aim_dx = 1;
        self.aim_dy = 0;
        self.air_mode = AirborneMode::Grounded;
        self.suit_o2 = 100;
        self.tranq_darts_left = 9;
        self.update_camera_instant();
    }

    /// Toggles the interlocking World-State Switch (`Tile::SluiceLever`) in Level 1 or Level 2!
    pub fn toggle_world_switch(&mut self, audio: &ArcadeAudio) {
        if self.world_switch_pulled {
            return;
        }
        self.world_switch_pulled = true;
        audio.play_sfx(Sfx::ServiceBell);

        if self.stage_num <= 1 {
            // LEVEL 1: Flood the Sunken Ravine (cols 36..=52, rows 11..=17) with deep open Star-Pond water!
            // Clears the intermediate dry steps so the flooded basin is 7 full tiles deep everywhere,
            // connecting flush from the Lever Pier (col 35, row 10) across to the High Canopy Vault (col 53, row 11)!
            for c in 36..=52 {
                self.tiles[11][c] = Tile::WaterSurface;
                for r in 12..=17 {
                    self.tiles[r][c] = Tile::WaterDeep;
                }
            }
            // Drown the dry-floor skittish Newt if it wasn't captured yet, and spawn a Voltaic Reed-Eel!
            for e in &mut self.entities {
                if e.behavior == CreatureBehavior::SkittishNewt && e.x >= 38 * TS && e.x <= 50 * TS {
                    e.alive = false;
                }
            }
            self.entities.push(FieldEntity {
                kind: IngKind::ReedEel,
                behavior: CreatureBehavior::VoltaicEel,
                home_x: 44 * TS,
                home_y: 14 * TS,
                x: 44 * TS,
                y: 14 * TS,
                vx: 3,
                sleep_ticks: 0,
                burrowed: false,
                shock_phase: 0,
                alive: true,
            });
            self.popups.push(FloatingText {
                text: "SLUICE OPENED! RAVINE FLOODED TO VAULT!".to_string(),
                x: self.px - 40,
                y: self.py - 18,
                col: 34,
                life: 22,
            });
        } else {
            // LEVEL 2: Drop the Counterweight Drawbridge (cols 38..45, row 12),
            // open the entire Sunken Star-Crypt ceiling (cols 46..57, rows 12..13),
            // and seal the Upper Canopy Sanctum Gate (col 51, rows 4..6)!
            self.fill_platform(38, 45, 12, 1);
            for r in 12..=13 {
                for c in 46..=57 {
                    self.tiles[r][c] = Tile::Empty;
                }
            }
            for r in 4..=6 {
                self.tiles[r][51] = Tile::TrunkWall;
            }
            self.popups.push(FloatingText {
                text: "BRIDGE DROPPED! STAR-CRYPT OPENED (CANOPY SEALED)!".to_string(),
                x: self.px - 60,
                y: self.py - 18,
                col: 15,
                life: 22,
            });
        }
    }

    pub fn warp_to_tutorial_zone(&mut self, zone: usize, audio: &mut ArcadeAudio) {
        self.active_trap = None;
        match zone {
            1 => {
                // Key [1]: Play Level 1 (The Underbrush)
                self.stage_num = 1;
                audio.set_track(0);
                self.mode = GameMode::PlanetaryDrop;
                self.build_stage_for_current_level();
            }
            2 => {
                // Key [2]: Play Level 2 (Daot the Enchanter - unlocks Grapple-Winch & Tranq Darts!)
                self.stage_num = 2;
                audio.set_track(1);
                self.mode = GameMode::PlanetaryDrop;
                self.build_stage_for_current_level();
            }
            3 => {
                // Key [3]: Warp directly to the Mid-Level Interlocking Puzzle Gate of current level!
                self.mode = GameMode::PlanetaryDrop;
                if self.stage_num <= 1 {
                    self.px = 33 * TS;
                    self.py = 10 * TS - COSMO_H;
                } else {
                    self.px = 33 * TS;
                    self.py = 8 * TS - COSMO_H;
                }
                self.air_mode = AirborneMode::Grounded;
            }
            4 => {
                // Key [4]: Warp to Pre-Service Menu Prep
                self.enter_menu_prep(audio);
                return;
            }
            5 => {
                // Key [5]: Warp to Live Dave-the-Diver Tavern Service
                self.enter_menu_prep(audio);
                self.start_live_tavern_service(audio);
                return;
            }
            6 => {
                // Key [6]: Warp to Tavern Upgrade Shop
                self.mode = GameMode::UpgradeShop;
                self.shop_cursor = 0;
                audio.play_sfx(Sfx::CoinStart);
                return;
            }
            _ => {}
        }
        self.update_camera_instant();
        audio.play_sfx(Sfx::SuctionStick);
    }

    pub fn current_tutorial_zone(&self) -> (usize, &'static str, &'static str) {
        let col = (self.px / TS).max(0) as usize;
        if self.stage_num <= 1 {
            if col <= 18 {
                (
                    1,
                    "LV 1: THE UNDERBRUSH (STALK & VENTS)",
                    "AUTO-CRAWL & LEAP! TIME THE SPORE-VENT! [X]:HARPOON",
                )
            } else if col <= 35 {
                (
                    2,
                    "LV 1: SHADOW ALCOVES & SKITTISH NEWT",
                    "DROP [C] LURE & HIDE IN ALCOVE! [X]:SLUICE LEVER",
                )
            } else if col <= 58 {
                if self.world_switch_pulled {
                    (
                        3,
                        "LV 1: FLOODED RAVINE -> HIGH VAULT",
                        "SWIM ACROSS TO CLAMBER ONTO THE HIGH VAULT FOR ROE!",
                    )
                } else {
                    (
                        3,
                        "LV 1: SUNKEN RAVINE (DRY OR FLOOD)",
                        "TRAP FLOOR NEWT OR HIT [X] LEVER TO FLOOD RAVINE!",
                    )
                }
            } else {
                (
                    4,
                    "LV 1: STAR-BASIN & EXTRACTION BELL",
                    "TOUCH BUBBLE-REEDS (+30% O2) & REACH BELL ([E]/[X])!",
                )
            }
        } else if col <= 17 {
            (
                1,
                "LV 2: DAOT THE ENCHANTER (TRANQ+WINCH)",
                "HIDE IN ALCOVE! [Z/SPACE]:TRANQ TOAD  [UP+X]:WINCH!",
            )
        } else if col <= 37 {
            (
                2,
                "LV 2: CAUSEWAY & ENCHANTER GUARD",
                "TRANQ [Z] ARMORED TOADS FOR 3* ROE! LEVER AT COL 37!",
            )
        } else if col <= 58 {
            if self.world_switch_pulled {
                (
                    3,
                    "LV 2: BRIDGE DROPPED -> STAR-CRYPT",
                    "DIVE INTO THE OPENED STAR-CRYPT FOR REED-EELS!",
                )
            } else {
                (
                    3,
                    "LV 2: CANOPY GRAPPLE OR DRAWBRIDGE",
                    "[UP+X] SWING OVERHEAD RINGS, OR HIT [X] BRIDGE LEVER!",
                )
            }
        } else {
            (
                4,
                "LV 2: SANCTUM BASIN & EXTRACTION BELL",
                "BAG FINAL SPECIMENS & PRESS [E] OR [X] AT THE BELL!",
            )
        }
    }

    /// Transitions from the Forest into Phase 2A: Pre-Service Dish Combiner (MenuPrep)
    pub fn enter_menu_prep(&mut self, audio: &ArcadeAudio) {
        self.mode = GameMode::MenuPrep;
        self.air_mode = AirborneMode::Grounded;
        self.portrait_mood = 0;

        // Ensure the player has at least 4 ingredients to combine into 2 dishes!
        while self.backpack.len() < 4 {
            let fallback_kind = match self.backpack.len() % 4 {
                0 => IngKind::DewNewt,
                1 => IngKind::VocalRoe,
                2 => IngKind::SporeLotus,
                _ => IngKind::ReedEel,
            };
            self.backpack.push(BagItem {
                kind: fallback_kind,
                prep: PrepState::Tenderized,
                ionized: self.backpack.len() == 3,
            });
        }

        self.prep_cursor = 0;
        self.selected_first_ing = None;
        self.menu_dishes.clear();
        audio.play_sfx(Sfx::ServiceBell);
    }

    /// Combines two harvested ingredients into a Curated Menu Dish ahead of time,
    /// factoring in 1*-3* Harvest Grade AND owned Restaurant Upgrades!
    pub fn combine_pair_into_dish(&self, a: BagItem, b: BagItem) -> CuratedDish {
        let name = match (a.kind, b.kind) {
            (IngKind::DewNewt, IngKind::SporeLotus) | (IngKind::SporeLotus, IngKind::DewNewt) => {
                "STAR-POND LOTUS CONSOMME"
            }
            (IngKind::VocalRoe, IngKind::DewNewt) | (IngKind::DewNewt, IngKind::VocalRoe) => {
                "LUMINOUS ROE TARTARE"
            }
            (IngKind::SporeLotus, IngKind::ReedEel) | (IngKind::ReedEel, IngKind::SporeLotus) => {
                "VOLTAIC LOTUS TERRINE"
            }
            (IngKind::VocalRoe, IngKind::ReedEel) | (IngKind::ReedEel, IngKind::VocalRoe) => {
                "PEPPER-GLAZED GOLDEN ROE"
            }
            (IngKind::VocalRoe, IngKind::SporeLotus) | (IngKind::SporeLotus, IngKind::VocalRoe) => {
                "DOAT'S ENCHANTED ROE POT"
            }
            _ => "AGED BOG-DEW DELICACY",
        };

        let mut base = a.kind.base_value() + b.kind.base_value();
        let mut stars = 1usize + (a.prep.star_bonus() + b.prep.star_bonus() + 1) / 2;

        // Complementary pairing bonus
        if a.kind != b.kind {
            base += 30;
            stars += 1;
        }
        // Harvest grade value bonuses (2* Harpoon = +$25 each, 3* Pristine Trapped/Tranq'd = +$55 each!)
        base += (a.prep.star_bonus() as i32) * 28 + (b.prep.star_bonus() as i32) * 28;

        if a.ionized || b.ionized {
            base += 40;
            stars += 1;
        }
        // Restaurant Upgrade: Sous-Chef Glaze Mastery (+45% Dish Price & +1 Star!)
        if self.has_upgrade(RestaurantUpgrade::SousChefMastery) {
            base = (base * 145) / 100;
            stars += 1;
        }

        // Restaurant Upgrade: Twin-Burner Copper Hearth halves cooking time!
        let cook_ticks = if self.has_upgrade(RestaurantUpgrade::TwinBurnerHearth) {
            10 // 0.66s at 15 FPS
        } else {
            20 // 1.33s at 15 FPS
        };

        CuratedDish {
            name: name.to_string(),
            primary_kind: a.kind,
            secondary_kind: b.kind,
            price: base,
            cook_ticks_needed: cook_ticks,
            stars: stars.clamp(1, 5),
        }
    }

    /// Transitions from MenuPrep into the live Dave the Diver-style Side-View Tavern Service!
    pub fn start_live_tavern_service(&mut self, audio: &ArcadeAudio) {
        // Auto-combine any remaining pairs in the backpack so no ingredients are wasted!
        while self.backpack.len() >= 2 && self.menu_dishes.len() < 4 {
            let a = self.backpack.remove(0);
            let b = self.backpack.remove(0);
            let dish = self.combine_pair_into_dish(a, b);
            self.menu_dishes.push(dish);
        }
        if self.menu_dishes.is_empty() {
            let dish = self.combine_pair_into_dish(
                BagItem {
                    kind: IngKind::DewNewt,
                    prep: PrepState::CharSeared,
                    ionized: true,
                },
                BagItem {
                    kind: IngKind::SporeLotus,
                    prep: PrepState::CharSeared,
                    ionized: false,
                },
            );
            self.menu_dishes.push(dish);
        }

        self.mode = GameMode::GalleyService;
        self.chef_bar_x = 152;
        self.chef_facing_right = true;
        self.chef_moving = false;
        self.carried_dish = None;
        self.ready_pass_dishes.clear();
        self.cooking_queue.clear();
        self.active_cook_dish = None;
        self.cook_progress = 0;
        self.patrons.clear();
        self.night_earnings = 0;
        self.dishes_served_tonight = 0;

        let base_pat = if self.has_upgrade(RestaurantUpgrade::AutoTeaSporeUrn) {
            160
        } else {
            100
        };

        // Spawn 1 patron per curated menu dish (up to 4 barstools)
        let num_patrons = self.menu_dishes.len().clamp(2, 4);
        for i in 0..num_patrons {
            let dish = self.menu_dishes[i % self.menu_dishes.len()].clone();
            self.cooking_queue.push(dish.clone());
            self.patrons.push(TavernPatron {
                stool_idx: i,
                x: 368 + (i as i32) * 28, // Walk in from the right cave doorway!
                stage: PatronStage::WalkingIn,
                ordered_dish: dish,
                wants_tea: i % 2 == 1,
                tea_served: false,
                patience: base_pat,
                max_patience: base_pat,
            });
        }

        // Start cooking the first queued dish immediately at the Kitchen Pass!
        if !self.cooking_queue.is_empty() {
            self.active_cook_dish = Some(self.cooking_queue.remove(0));
            self.cook_progress = 0;
        }

        audio.play_sfx(Sfx::ServiceBell);
    }

    pub fn update_15fps_tick(
        &mut self,
        input: &InputState,
        audio: &mut ArcadeAudio,
        pal: &mut VgaPalette,
    ) {
        self.tick += 1;
        pal.update_cycler(self.tick);

        if input.help_pressed {
            self.show_help_overlay = !self.show_help_overlay;
        }
        if self.show_help_overlay {
            return;
        }

        if let Some(zone) = input.warp_step {
            self.warp_to_tutorial_zone(zone, audio);
            return;
        }

        for p in &mut self.particles {
            p.x += p.vx;
            p.y += p.vy;
            p.vy += 2;
            p.life -= 1;
        }
        self.particles.retain(|p| p.life > 0);

        for ft in &mut self.popups {
            ft.y -= 2;
            ft.life -= 1;
        }
        self.popups.retain(|ft| ft.life > 0);

        if self.mood_timer > 0 {
            self.mood_timer -= 1;
            if self.mood_timer == 0 {
                self.portrait_mood = 0;
            }
        }
        if self.hurt_cooldown > 0 {
            self.hurt_cooldown -= 1;
        }

        match self.mode {
            GameMode::TitleAttractor => {
                if input.jump_pressed || input.action_pressed || input.extract_pressed {
                    audio.play_sfx(Sfx::CoinStart);
                    self.stage_num = 1;
                    audio.set_track(0);
                    self.build_stage_for_current_level();
                    self.mode = GameMode::PlanetaryDrop;
                }
            }
            GameMode::PlanetaryDrop => self.update_planetary_drop_15fps(input, audio),
            GameMode::MenuPrep => self.update_menu_prep_15fps(input, audio),
            GameMode::GalleyService => self.update_galley_service_15fps(input, audio),
            GameMode::UpgradeShop => self.update_upgrade_shop_15fps(input, audio),
        }
    }

    fn is_in_star_pond(&self, px: i32, py: i32) -> bool {
        let cx = ((px + COSMO_W / 2) / TS).clamp(0, WORLD_COLS as i32 - 1) as usize;
        let cy_mid = ((py + 24) / TS).clamp(0, WORLD_ROWS as i32 - 1) as usize;
        matches!(
            self.tiles[cy_mid][cx],
            Tile::WaterSurface | Tile::WaterDeep
        )
    }

    fn find_water_surface_row(&self, px: i32, py: i32) -> usize {
        let cx = ((px + COSMO_W / 2) / TS).clamp(0, WORLD_COLS as i32 - 1) as usize;
        let mut r = ((py + 24) / TS).clamp(0, WORLD_ROWS as i32 - 1) as usize;
        while r > 0
            && matches!(
                self.tiles[r - 1][cx],
                Tile::WaterSurface | Tile::WaterDeep
            )
        {
            r -= 1;
        }
        r
    }

    pub fn is_hidden_in_shadow(&self) -> bool {
        let cx = ((self.px + COSMO_W / 2) / TS).clamp(0, WORLD_COLS as i32 - 1) as usize;
        let cy = ((self.py + 22) / TS).clamp(0, WORLD_ROWS as i32 - 1) as usize;
        self.tiles[cy][cx] == Tile::ShadowAlcove
    }

    pub fn is_spore_vent_erupting(&self, col: usize) -> bool {
        let phase = (self.tick + (col as u64) * 11) % 45;
        phase >= 32
    }

    fn vent_suit_o2(&mut self, amount: i32, reason: &str, audio: &ArcadeAudio) {
        if self.hurt_cooldown > 0 {
            return;
        }
        self.suit_o2 = (self.suit_o2 - amount).max(0);
        self.hurt_cooldown = 16;
        self.portrait_mood = 2;
        self.mood_timer = 10;
        audio.play_sfx(Sfx::BombExplode);
        self.popups.push(FloatingText {
            text: format!("{} (-{}% O2!)", reason, amount),
            x: self.px - 28,
            y: self.py - 14,
            col: 38,
            life: 16,
        });
        for i in -2..=2 {
            self.particles.push(Particle {
                x: self.px + 20,
                y: self.py + 18,
                vx: i * 3,
                vy: -5,
                col: 34,
                life: 8,
            });
        }
    }

    fn update_planetary_drop_15fps(&mut self, input: &InputState, audio: &ArcadeAudio) {
        if self.pluck_timer > 0 {
            self.pluck_timer -= 1;
            if self.pluck_timer == 0
                && !matches!(self.air_mode, AirborneMode::GrappleWinch { .. })
            {
                self.tether_target = None;
            }
        }

        // Update 8-Way Aim Vector from held directional inputs
        let mut adx = 0;
        let mut ady = 0;
        if input.left {
            adx -= 1;
        }
        if input.right {
            adx += 1;
        }
        if input.up {
            ady -= 1;
        }
        if input.down {
            ady += 1;
        }
        if adx != 0 || ady != 0 {
            self.aim_dx = adx;
            self.aim_dy = ady;
        } else {
            self.aim_dx = if self.facing_right { 1 } else { -1 };
            self.aim_dy = 0;
        }

        let in_water = self.is_in_star_pond(self.px, self.py);
        if in_water
            && !matches!(
                self.air_mode,
                AirborneMode::GrappleWinch { .. }
                    | AirborneMode::CrawlingOver { .. }
                    | AirborneMode::GapLeap { .. }
            )
        {
            self.air_mode = AirborneMode::Swimming;
        } else if !in_water && self.air_mode == AirborneMode::Swimming {
            if self.is_standing_on_ground(self.px, self.py) {
                self.air_mode = AirborneMode::Grounded;
            } else {
                self.air_mode = AirborneMode::Falling(0);
            }
        }

        // Deliberate stalking speed on land (4px/tick), fluid diving speed in water (6px/tick)
        let stalk_base = if self.has_upgrade(RestaurantUpgrade::SuctionDashBoots) {
            5
        } else {
            4
        };
        let stalk_step_x = if self.cargo_weight_kg() >= self.max_cargo_kg() {
            3
        } else {
            stalk_base
        };

        let swim_base = if self.has_upgrade(RestaurantUpgrade::SuctionDashBoots) {
            8
        } else {
            6
        };
        let swim_step_x = if self.cargo_weight_kg() >= self.max_cargo_kg() {
            (swim_base - 2).max(4)
        } else {
            swim_base
        };

        self.moving_x = false;

        match self.air_mode {
            AirborneMode::GrappleWinch {
                target_x,
                target_y,
                anchor_x,
                anchor_y,
            } => {
                // Smoothly winch along the cable up onto the ledge!
                self.portrait_mood = 1;
                self.mood_timer = 2;
                self.tether_target = Some((anchor_x, anchor_y));
                self.pluck_timer = 2;

                let dx = target_x - self.px;
                let dy = target_y - self.py;
                if dx.abs() <= 14 && dy.abs() <= 14 {
                    self.px = target_x;
                    self.py = target_y;
                    self.air_mode = AirborneMode::Grounded;
                    self.tether_target = None;
                    audio.play_sfx(Sfx::SuctionStick);
                } else {
                    self.px += dx.clamp(-14, 14);
                    self.py += dy.clamp(-14, 14);
                }
            }
            AirborneMode::CrawlingOver {
                start_x,
                start_y,
                target_x,
                target_y,
                step,
                total,
            } => {
                // Smooth diagonal step/mantle over low obstacles or onto the pier!
                self.moving_x = true;
                let next_step = step + 1;
                if next_step >= total {
                    self.px = target_x;
                    self.py = target_y;
                    self.air_mode = AirborneMode::Grounded;
                } else {
                    self.px = start_x + ((target_x - start_x) * next_step) / total;
                    self.py = start_y + ((target_y - start_y) * next_step) / total;
                    self.air_mode = AirborneMode::CrawlingOver {
                        start_x,
                        start_y,
                        target_x,
                        target_y,
                        step: next_step,
                        total,
                    };
                }
            }
            AirborneMode::GapLeap {
                start_x,
                start_y,
                target_x,
                target_y,
                step,
                total,
            } => {
                // Smooth automatic athletic leap across 1..3-tile gaps!
                self.moving_x = true;
                let next_step = step + 1;
                if next_step >= total {
                    self.px = target_x;
                    self.py = target_y;
                    self.air_mode = AirborneMode::Grounded;
                } else {
                    self.px = start_x + ((target_x - start_x) * next_step) / total;
                    let base_y = start_y + ((target_y - start_y) * next_step) / total;
                    let arc_lift = (4 * next_step * (total - next_step) * 12) / (total * total);
                    self.py = base_y - arc_lift;
                    self.air_mode = AirborneMode::GapLeap {
                        start_x,
                        start_y,
                        target_x,
                        target_y,
                        step: next_step,
                        total,
                    };
                }
            }
            AirborneMode::Swimming => {
                // =========================================================================
                // BUOYANT STAR-POND DIVER MODE (8-Way Swimming + Smooth Pier Clamber Exit!)
                // =========================================================================
                let mut moved = false;
                let mut clambered = false;
                if input.left && !input.right {
                    self.facing_right = false;
                    self.move_x_by(-swim_step_x, false);
                    moved = true;
                    clambered = self.try_clamber_out_of_water(-1, audio);
                } else if input.right && !input.left {
                    self.facing_right = true;
                    self.move_x_by(swim_step_x, false);
                    moved = true;
                    clambered = self.try_clamber_out_of_water(1, audio);
                }
                if !clambered {
                    let surf_row = self.find_water_surface_row(self.px, self.py);
                    let surface_min_y = (surf_row as i32) * TS - 10;
                    if input.up && !input.down {
                        if self.py <= surface_min_y + 12
                            && (self.try_clamber_out_of_water(-1, audio)
                                || self.try_clamber_out_of_water(1, audio))
                        {
                            clambered = true;
                        }
                        if !clambered {
                            let _ = self.move_y_by(-swim_step_x);
                            moved = true;
                        }
                    } else if input.down && !input.up {
                        let _ = self.move_y_by(swim_step_x);
                        moved = true;
                    }
                    // Keep the diver submerged beneath the dynamic Star-Pond surface meniscus
                    if !clambered && self.py < surface_min_y {
                        self.py = surface_min_y;
                    }
                }
                self.moving_x = moved;
                if moved {
                    self.walk_anim_tick = self.walk_anim_tick.wrapping_add(1);
                }
                if self.tick % 4 == 0 {
                    self.particles.push(Particle {
                        x: self.px + if self.facing_right { 28 } else { 10 },
                        y: self.py + 12,
                        vx: 0,
                        vy: -3,
                        col: 34,
                        life: 7,
                    });
                }
            }
            AirborneMode::Grounded | AirborneMode::Falling(_) => {
                // =========================================================================
                // GROUNDED STALKING HUNTER MODE (Stealth Walk + Auto-Crawl & Auto-Leap!)
                // =========================================================================
                let was_grounded = self.air_mode == AirborneMode::Grounded;
                let mut walk_dir = 0;

                if input.left && !input.right {
                    self.facing_right = false;
                    self.moving_x = true;
                    walk_dir = -1;
                    self.move_x_by(-stalk_step_x, false);
                } else if input.right && !input.left {
                    self.facing_right = true;
                    self.moving_x = true;
                    walk_dir = 1;
                    self.move_x_by(stalk_step_x, false);
                }

                // If move_x_by triggered an automatic obstacle step/crawl, play sound and advance!
                if matches!(self.air_mode, AirborneMode::CrawlingOver { .. }) {
                    audio.play_sfx(Sfx::SuctionStick);
                } else {
                    if self.moving_x && self.air_mode == AirborneMode::Grounded {
                        self.walk_anim_tick = self.walk_anim_tick.wrapping_add(1);
                    } else {
                        self.walk_anim_tick = 0;
                    }

                    if self.is_standing_on_ground(self.px, self.py) {
                        self.air_mode = AirborneMode::Grounded;
                    } else if was_grounded
                        && walk_dir != 0
                        && self.try_auto_leap_or_water_entry(walk_dir, audio)
                    {
                        // Automatically initiated a gap leap, 1-tile step down, or smooth dive!
                    } else {
                        let step = match self.air_mode {
                            AirborneMode::Falling(s) => s,
                            _ => 0,
                        };
                        let dy = FALL_ARC[step.min(FALL_ARC.len() - 1)];
                        if self.move_y_by(dy) {
                            self.air_mode = AirborneMode::Grounded;
                        } else {
                            self.air_mode =
                                AirborneMode::Falling((step + 1).min(FALL_ARC.len() - 1));
                        }
                    }
                }
            }
        }

        // =========================================================================
        // DAVE THE DIVER O2-IS-HEALTH & TIMER ECONOMY (Drain, Bubble-Reeds, Vents)
        // =========================================================================
        let o2_drain_interval = if self.air_mode == AirborneMode::Swimming
            || self.cargo_weight_kg() >= self.max_cargo_kg()
        {
            12
        } else {
            18
        };
        if self.tick % o2_drain_interval == 0 && self.suit_o2 > 0 {
            self.suit_o2 -= 1;
        }

        // Check Bubble-Reed O2 Vents (`Tile::OxygenReed`) & Spore-Geyser Plumes (`Tile::SporeVent`)
        let pc_col = ((self.px + 20) / TS).clamp(0, WORLD_COLS as i32 - 1) as usize;
        let pc_row = ((self.py + 22) / TS).clamp(0, WORLD_ROWS as i32 - 1) as usize;
        let foot_row = ((self.py + COSMO_H) / TS).clamp(0, WORLD_ROWS as i32 - 1) as usize;

        for r in pc_row.saturating_sub(1)..=(pc_row + 1).min(WORLD_ROWS - 1) {
            for c in pc_col.saturating_sub(1)..=(pc_col + 1).min(WORLD_COLS - 1) {
                if self.tiles[r][c] == Tile::OxygenReed {
                    let rx = (c as i32) * TS + 12;
                    let ry = (r as i32) * TS + 12;
                    if (rx - (self.px + 20)).abs() <= 22 && (ry - (self.py + 22)).abs() <= 26 {
                        let is_submerged = r > 0
                            && matches!(
                                self.tiles[r - 1][c],
                                Tile::WaterSurface | Tile::WaterDeep
                            );
                        self.tiles[r][c] = if is_submerged {
                            Tile::WaterDeep
                        } else {
                            Tile::Empty
                        };
                        self.suit_o2 = (self.suit_o2 + 30).min(100);
                        audio.play_sfx(Sfx::HarvestPluck);
                        self.popups.push(FloatingText {
                            text: "BUBBLE-REED +30% O2!".to_string(),
                            x: rx - 32,
                            y: ry - 16,
                            col: 34,
                            life: 16,
                        });
                        for b in -2..=2 {
                            self.particles.push(Particle {
                                x: rx + b * 4,
                                y: ry,
                                vx: b,
                                vy: -4,
                                col: 34,
                                life: 9,
                            });
                        }
                    }
                }
            }
        }

        // Check if standing on or directly above an erupting Spore-Vent (`Tile::SporeVent`)
        for check_r in [foot_row, pc_row, pc_row.saturating_add(1).min(WORLD_ROWS - 1)] {
            if self.tiles[check_r][pc_col] == Tile::SporeVent
                && self.is_spore_vent_erupting(pc_col)
            {
                self.vent_suit_o2(14, "TOXIC SPORE PLUME!", audio);
                break;
            }
        }

        // =========================================================================
        // ECOLOGICAL CREATURE BEHAVIORS (Skittish Newt, Armored Toad, Voltaic Eel)
        // =========================================================================
        let player_cx = self.px + 20;
        let player_cy = self.py + 22;
        let hidden_in_alcove = self.is_hidden_in_shadow();
        let trap_pos = self.active_trap.as_ref().map(|t| (t.x, t.y, t.snapped));
        let mut spawned_orbs: Vec<SporeOrb> = Vec::new();

        for i in 0..self.entities.len() {
            if !self.entities[i].alive {
                continue;
            }
            if self.entities[i].sleep_ticks > 0 {
                self.entities[i].sleep_ticks -= 1;
                // Sleeping creatures can be collected at 3* PRISTINE simply by walking/swimming over them!
                let dx = (self.entities[i].x + 17) - player_cx;
                let dy = (self.entities[i].y + 17) - player_cy;
                if dx * dx + dy * dy <= 34 * 34 {
                    let kind = self.entities[i].kind;
                    let ex = self.entities[i].x;
                    let ey = self.entities[i].y;
                    self.entities[i].alive = false;
                    audio.play_sfx(Sfx::HarvestPluck);
                    self.collect_ingredient(kind, PrepState::CharSeared, ex, ey);
                }
                continue;
            }

            match self.entities[i].behavior {
                CreatureBehavior::SessileFlora => {}
                CreatureBehavior::SkittishNewt => {
                    // If a Bait-Net Trap [C] is deployed and player stepped back OR hid in a Shadow Alcove, Newt emerges & walks to the lure!
                    if let Some((tx, ty, false)) = trap_pos {
                        let dist_player_to_trap = (player_cx - tx).abs();
                        let newt_near_trap = (self.entities[i].x - tx).abs() < 160
                            && (self.entities[i].y - ty).abs() < 48;
                        if newt_near_trap && (dist_player_to_trap >= 42 || hidden_in_alcove) {
                            self.entities[i].burrowed = false;
                            let diff = (tx - 6) - self.entities[i].x;
                            if diff.abs() <= 6 {
                                self.entities[i].x = tx - 6;
                                // Auto-snap the Bait-Net Trap when Twen feeds on the lure!
                                let kind = self.entities[i].kind;
                                let ex = self.entities[i].x;
                                let ey = self.entities[i].y;
                                self.entities[i].alive = false;
                                self.active_trap = None;
                                audio.play_sfx(Sfx::HarvestPluck);
                                self.popups.push(FloatingText {
                                    text: "NET SNAPPED! 3* PRISTINE!".to_string(),
                                    x: tx - 24,
                                    y: ty - 24,
                                    col: 25,
                                    life: 16,
                                });
                                self.collect_ingredient(kind, PrepState::CharSeared, ex, ey);
                            } else {
                                self.entities[i].x += diff.signum() * 4;
                            }
                            continue;
                        }
                    }

                    // Otherwise, if the player approaches within 78px AND is not hidden in a Shadow Alcove, Twen darts into its burrow!
                    let dist_p = (self.entities[i].x - self.px).abs()
                        + (self.entities[i].y - self.py).abs();
                    if dist_p < 78 && !hidden_in_alcove {
                        if !self.entities[i].burrowed {
                            self.entities[i].burrowed = true;
                            self.popups.push(FloatingText {
                                text: "SPOOKED! USE [C] LURE & ALCOVE!".to_string(),
                                x: self.entities[i].x - 36,
                                y: self.entities[i].y - 16,
                                col: 14,
                                life: 14,
                            });
                        }
                    } else {
                        self.entities[i].burrowed = false;
                        // Gentle patrol near burrow
                        let offset = self.entities[i].x - self.entities[i].home_x;
                        if offset.abs() > 24 {
                            self.entities[i].vx = -offset.signum() * 2;
                        }
                        self.entities[i].x += self.entities[i].vx;
                    }
                }
                CreatureBehavior::ArmoredToad => {
                    // Daot's Enchanter Toads rhythmically fire Hypnotic Spore-Orbs when player is in sight!
                    self.entities[i].shock_phase = (self.entities[i].shock_phase + 1) % 38;
                    let dx = player_cx - (self.entities[i].x + 17);
                    let dy = player_cy - (self.entities[i].y + 16);
                    if !hidden_in_alcove
                        && dx.abs() <= 210
                        && dy.abs() <= 48
                        && self.entities[i].shock_phase == 30
                    {
                        let dir = if dx < 0 { -1 } else { 1 };
                        spawned_orbs.push(SporeOrb {
                            x: self.entities[i].x + 16 + dir * 12,
                            y: self.entities[i].y + 14,
                            vx: dir * 9,
                            vy: 0,
                            life: 24,
                        });
                    }
                }
                CreatureBehavior::VoltaicEel => {
                    // Swims back and forth in the Star-Pond & cycles electric charge (0..45)
                    self.entities[i].shock_phase = (self.entities[i].shock_phase + 1) % 45;
                    let offset = self.entities[i].x - self.entities[i].home_x;
                    if offset.abs() > 42 {
                        self.entities[i].vx = -offset.signum() * 3;
                    }
                    self.entities[i].x += self.entities[i].vx;

                    // Contact with a charged Voltaic Eel vents Suit O2!
                    if self.entities[i].shock_phase >= 28 {
                        let dx = (self.entities[i].x + 17) - player_cx;
                        let dy = (self.entities[i].y + 17) - player_cy;
                        if dx * dx + dy * dy <= 24 * 24 {
                            self.vent_suit_o2(12, "VOLTAIC SHOCK!", audio);
                        }
                    }
                }
            }
        }
        self.orbs.extend(spawned_orbs);

        // =========================================================================
        // UPDATE DAOT'S HYPNOTIC SPORE-ORBS (Vents O2 unless hidden in Shadow Alcove!)
        // =========================================================================
        let mut orb_hit = false;
        for orb in &mut self.orbs {
            orb.x += orb.vx;
            orb.y += orb.vy;
            orb.life -= 1;

            let tc = (orb.x / TS).clamp(0, WORLD_COLS as i32 - 1) as usize;
            let tr = (orb.y / TS).clamp(0, WORLD_ROWS as i32 - 1) as usize;
            if matches!(
                self.tiles[tr][tc],
                Tile::EarthTop | Tile::EarthFill | Tile::TrunkWall
            ) {
                orb.life = 0;
                continue;
            }
            if !hidden_in_alcove
                && (orb.x - player_cx).abs() <= 16
                && (orb.y - player_cy).abs() <= 20
            {
                orb.life = 0;
                orb_hit = true;
            }
        }
        self.orbs.retain(|o| o.life > 0);
        if orb_hit {
            self.vent_suit_o2(15, "ENCHANTER SPORE-ORB!", audio);
        }

        // =========================================================================
        // CHECK FOR SUIT O2 BLACKOUT (Dave the Diver Emergency Drone Extraction!)
        // =========================================================================
        if self.suit_o2 <= 0 {
            self.blackout_occurred = true;
            if self.backpack.len() > 1 {
                self.backpack.truncate(1);
            }
            audio.play_sfx(Sfx::BombExplode);
            self.enter_menu_prep(audio);
            return;
        }

        // =========================================================================
        // UPDATE SOPORIFIC TRANQ DARTS ([Z] / [SPACE])
        // =========================================================================
        for d in &mut self.darts {
            d.x += d.vx;
            d.y += d.vy;
            d.life -= 1;

            let tc = (d.x / TS).clamp(0, WORLD_COLS as i32 - 1) as usize;
            let tr = (d.y / TS).clamp(0, WORLD_ROWS as i32 - 1) as usize;
            if matches!(
                self.tiles[tr][tc],
                Tile::EarthTop | Tile::EarthFill | Tile::TrunkWall
            ) {
                d.life = 0;
                continue;
            }

            for e in &mut self.entities {
                if e.alive && !e.burrowed {
                    let dx = (e.x + 17) - d.x;
                    let dy = (e.y + 17) - d.y;
                    if dx * dx + dy * dy <= 26 * 26 {
                        e.sleep_ticks = 95; // Put creature to sleep for ~6.3 seconds!
                        e.shock_phase = 0;
                        d.life = 0;
                        audio.play_sfx(Sfx::SuctionStick);
                        self.popups.push(FloatingText {
                            text: "TRANQ'D! [X] FOR 3* PRISTINE!".to_string(),
                            x: e.x - 24,
                            y: e.y - 16,
                            col: 34,
                            life: 16,
                        });
                        break;
                    }
                }
            }
        }
        self.darts.retain(|d| d.life > 0);

        // =========================================================================
        // TOOL 1: [X] PNEUMATIC TETHER-HARPOON (Grapple-Winch, Sluice Lever, or Prey!)
        // =========================================================================
        if input.action_pressed {
            let near_pod = (self.px - 75 * TS).abs() < 68 && self.py >= 14 * TS;
            if near_pod {
                self.enter_menu_prep(audio);
                return;
            }
            self.fire_pneumatic_harpoon(input.up, audio);
        }

        // =========================================================================
        // TOOL 2: [Z] OR [SPACE] SOPORIFIC SPORE-DART (TRANQ GUN - Unlocks in Level 2!)
        // =========================================================================
        if input.jump_pressed {
            if !self.has_grapple_and_tranq() {
                audio.play_sfx(Sfx::SuctionStick);
                self.popups.push(FloatingText {
                    text: "TRANQ DART UNLOCKS IN LV 2 (PRESS [2])!".to_string(),
                    x: self.px - 40,
                    y: self.py - 14,
                    col: 14,
                    life: 14,
                });
            } else if self.tranq_darts_left > 0 {
                self.tranq_darts_left -= 1;
                let mut vx = self.aim_dx * 16;
                let vy = self.aim_dy * 16;
                if vx == 0 && vy == 0 {
                    vx = if self.facing_right { 16 } else { -16 };
                }
                self.darts.push(TranqDart {
                    x: self.px + 20,
                    y: self.py + 18,
                    vx,
                    vy,
                    life: 16,
                });
                audio.play_sfx(Sfx::SuctionLaunch);
            }
        }

        // =========================================================================
        // TOOL 3: [C] BIOLUMINESCENT BAIT-NET TRAP
        // =========================================================================
        if input.bomb_pressed {
            if self.active_trap.is_some() {
                self.active_trap = None;
                audio.play_sfx(Sfx::SuctionStick);
            } else {
                let dir = if self.facing_right { 1 } else { -1 };
                self.active_trap = Some(BaitTrap {
                    x: self.px + 20 + dir * 24,
                    y: self.py + COSMO_H - 12,
                    snapped: false,
                    timer: 0,
                });
                audio.play_sfx(Sfx::SuctionStick);
                self.popups.push(FloatingText {
                    text: "SPORE-LURE SET! STEP BACK OR HIDE!".to_string(),
                    x: self.px - 18,
                    y: self.py - 12,
                    col: 25,
                    life: 14,
                });
            }
        }

        if input.extract_pressed {
            self.enter_menu_prep(audio);
            return;
        }

        self.update_camera_stepped();
    }

    fn find_grapple_anchor(&self, prefer_up: bool) -> Option<(i32, i32, i32, i32)> {
        if !self.has_grapple_and_tranq() {
            return None;
        }
        let mut best: Option<(i32, i32, i32, i32, i32)> = None;
        let pcx = self.px + 20;
        let pcy = self.py + 20;

        for r in 0..WORLD_ROWS {
            for c in 0..WORLD_COLS {
                if self.tiles[r][c] == Tile::GrappleAnchor {
                    let ax = (c as i32) * TS + 12;
                    let ay = (r as i32) * TS + 12;
                    let dx = ax - pcx;
                    let dy = ay - pcy;
                    let d2 = dx * dx + dy * dy;
                    let max_r = if prefer_up { 190 } else { 150 };
                    if d2 <= max_r * max_r && dy < 16 {
                        if best.map_or(true, |b| d2 < b.4) {
                            // Land on the upper ledge right next to the anchor!
                            let land_x = (c as i32 + 1) * TS;
                            let land_y = (r as i32 + 2) * TS - COSMO_H;
                            best = Some((land_x, land_y, ax, ay, d2));
                        }
                    }
                }
            }
        }
        best.map(|(lx, ly, ax, ay, _)| (lx, ly, ax, ay))
    }

    fn fire_pneumatic_harpoon(&mut self, holding_up: bool, audio: &ArcadeAudio) {
        let range = if self.has_upgrade(RestaurantUpgrade::SuctionDashBoots) {
            210
        } else {
            165
        };

        // 0. Check if a World-State Sluice / Counterweight Lever (`Tile::SluiceLever`) is within harpoon reach!
        if !self.world_switch_pulled {
            for r in 0..WORLD_ROWS {
                for c in 0..WORLD_COLS {
                    if self.tiles[r][c] == Tile::SluiceLever {
                        let lx = (c as i32) * TS + 12;
                        let ly = (r as i32) * TS + 12;
                        let dx = lx - (self.px + 20);
                        let dy = ly - (self.py + 22);
                        if dx * dx + dy * dy <= (range * range) {
                            self.tether_target = Some((lx, ly));
                            self.pluck_timer = 6;
                            self.toggle_world_switch(audio);
                            return;
                        }
                    }
                }
            }
        }

        // 1. If holding [UP] (or if an overhead Grapple-Root Anchor Ring is right above us), lock onto Grapple Ring!
        if holding_up {
            if let Some((target_x, target_y, anchor_x, anchor_y)) = self.find_grapple_anchor(true) {
                self.air_mode = AirborneMode::GrappleWinch {
                    target_x,
                    target_y,
                    anchor_x,
                    anchor_y,
                };
                self.tether_target = Some((anchor_x, anchor_y));
                self.pluck_timer = 8;
                audio.play_sfx(Sfx::SuctionLaunch);
                self.popups.push(FloatingText {
                    text: "GRAPPLE WINCH!".to_string(),
                    x: self.px,
                    y: self.py - 10,
                    col: 15,
                    life: 10,
                });
                return;
            } else if !self.has_grapple_and_tranq() {
                self.popups.push(FloatingText {
                    text: "GRAPPLE-WINCH UNLOCKS IN LV 2 (PRESS [2])!".to_string(),
                    x: self.px - 44,
                    y: self.py - 14,
                    col: 14,
                    life: 14,
                });
            }
        }

        // 2. Check for living creatures in range
        let mut best_idx = None;
        let mut best_d2 = range * range;

        for (i, e) in self.entities.iter().enumerate() {
            if !e.alive || e.burrowed {
                continue;
            }
            let dx = (e.x + 17) - (self.px + 20);
            let dy = (e.y + 17) - (self.py + 22);
            let d2 = dx * dx + dy * dy;
            if d2 <= best_d2 {
                best_d2 = d2;
                best_idx = Some(i);
            }
        }

        self.pluck_timer = 5;
        if let Some(idx) = best_idx {
            let ex = self.entities[idx].x;
            let ey = self.entities[idx].y;
            let kind = self.entities[idx].kind;
            let behavior = self.entities[idx].behavior;
            let asleep = self.entities[idx].sleep_ticks > 0;
            let charged =
                behavior == CreatureBehavior::VoltaicEel && self.entities[idx].shock_phase >= 28;

            self.tether_target = Some((ex + 17, ey + 17));

            if asleep {
                // Sleeping prey is always reeled in gently at 3* PRISTINE!
                self.entities[idx].alive = false;
                audio.play_sfx(Sfx::HarvestPluck);
                self.collect_ingredient(kind, PrepState::CharSeared, ex, ey);
            } else if behavior == CreatureBehavior::ArmoredToad {
                // Frontal spore-shield deflects raw harpoon!
                audio.play_sfx(Sfx::SuctionStick);
                self.popups.push(FloatingText {
                    text: "SHIELDED! PRESS [Z/SPACE] TO TRANQ!".to_string(),
                    x: ex - 36,
                    y: ey - 14,
                    col: 14,
                    life: 16,
                });
            } else if charged {
                // Harpooning a charged Voltaic Eel shocks the cable!
                self.vent_suit_o2(12, "HARPOON SHOCKED!", audio);
            } else {
                // Reel in creature! Delicate Dew-Newts get 1* Bruised unless Silk-Harpoon upgrade owned
                self.entities[idx].alive = false;
                audio.play_sfx(Sfx::SuctionLaunch);
                let grade = if behavior == CreatureBehavior::SkittishNewt
                    && !self.has_upgrade(RestaurantUpgrade::SuctionDashBoots)
                {
                    PrepState::Fresh // 1* BRUISED
                } else {
                    PrepState::Tenderized // 2* HARPOON
                };
                self.collect_ingredient(kind, grade, ex, ey);
            }
            return;
        }

        // 3. Even if [UP] wasn't held, if an overhead Grapple-Root Anchor Ring is nearby, lock onto it!
        if let Some((target_x, target_y, anchor_x, anchor_y)) = self.find_grapple_anchor(false) {
            self.air_mode = AirborneMode::GrappleWinch {
                target_x,
                target_y,
                anchor_x,
                anchor_y,
            };
            self.tether_target = Some((anchor_x, anchor_y));
            self.pluck_timer = 8;
            audio.play_sfx(Sfx::SuctionLaunch);
            self.popups.push(FloatingText {
                text: "GRAPPLE WINCH!".to_string(),
                x: self.px,
                y: self.py - 10,
                col: 15,
                life: 10,
            });
            return;
        }

        let tx = self.px + 20 + self.aim_dx * 64;
        let ty = self.py + 20 + self.aim_dy * 64;
        self.tether_target = Some((tx, ty));
        audio.play_sfx(Sfx::SuctionStick);
    }

    /// Phase 2A: Pre-Service Dish Combiner (Combine harvested ingredients ahead of time!)
    fn update_menu_prep_15fps(&mut self, input: &InputState, audio: &ArcadeAudio) {
        if !self.backpack.is_empty() {
            if input.up_pressed {
                self.prep_cursor =
                    (self.prep_cursor + self.backpack.len() - 1) % self.backpack.len();
                audio.play_sfx(Sfx::SuctionStick);
            } else if input.down_pressed {
                self.prep_cursor = (self.prep_cursor + 1) % self.backpack.len();
                audio.play_sfx(Sfx::SuctionStick);
            }
        }

        // [X] or [SPACE]: Select 1st ingredient, then select 2nd ingredient to combine into a Curated Dish!
        if (input.action_pressed || input.jump_pressed) && !self.backpack.is_empty() {
            let idx = self.prep_cursor.min(self.backpack.len() - 1);
            if let Some(first_idx) = self.selected_first_ing {
                if first_idx != idx && first_idx < self.backpack.len() {
                    let hi = first_idx.max(idx);
                    let lo = first_idx.min(idx);
                    let item_b = self.backpack.remove(hi);
                    let item_a = self.backpack.remove(lo);
                    let dish = self.combine_pair_into_dish(item_a, item_b);
                    self.menu_dishes.push(dish);
                    self.selected_first_ing = None;
                    self.prep_cursor = 0;
                    audio.play_sfx(Sfx::HarvestPluck);
                } else {
                    self.selected_first_ing = None;
                }
            } else if self.backpack.len() == 1 {
                let item_a = self.backpack.remove(0);
                let dish = self.combine_pair_into_dish(item_a, item_a);
                self.menu_dishes.push(dish);
                audio.play_sfx(Sfx::HarvestPluck);
            } else {
                self.selected_first_ing = Some(idx);
                audio.play_sfx(Sfx::CleaverChop);
            }
        }

        // [C]: Quick-combine next pair automatically
        if input.bomb_pressed && self.backpack.len() >= 2 {
            let a = self.backpack.remove(0);
            let b = self.backpack.remove(0);
            let dish = self.combine_pair_into_dish(a, b);
            self.menu_dishes.push(dish);
            self.selected_first_ing = None;
            self.prep_cursor = 0;
            audio.play_sfx(Sfx::HarvestPluck);
        }

        // [E] or [ENTER] (or when all ingredients are combined and player presses Space): Open Tavern Doors!
        if input.extract_pressed
            || (self.backpack.is_empty() && (input.jump_pressed || input.action_pressed))
        {
            self.start_live_tavern_service(audio);
        }
    }

    /// Phase 2B: Dave the Diver-style Side-View Tavern Service!
    fn update_galley_service_15fps(&mut self, input: &InputState, audio: &ArcadeAudio) {
        let walk_speed = if self.has_upgrade(RestaurantUpgrade::SuctionDashBoots) {
            12
        } else {
            8
        };
        self.chef_moving = false;
        if input.left && !input.right {
            self.chef_bar_x = (self.chef_bar_x - walk_speed).max(126);
            self.chef_facing_right = false;
            self.chef_moving = true;
            self.walk_anim_tick = self.walk_anim_tick.wrapping_add(1);
        } else if input.right && !input.left {
            self.chef_bar_x = (self.chef_bar_x + walk_speed).min(336);
            self.chef_facing_right = true;
            self.chef_moving = true;
            self.walk_anim_tick = self.walk_anim_tick.wrapping_add(1);
        }

        if let Some(ref dish) = self.active_cook_dish {
            self.cook_progress += 1;
            if self.has_upgrade(RestaurantUpgrade::SousChefMastery) && self.tick % 2 == 0 {
                self.cook_progress += 1;
            }
            if self.cook_progress >= dish.cook_ticks_needed {
                let finished = self.active_cook_dish.take().unwrap();
                self.ready_pass_dishes.push(finished);
                self.cook_progress = 0;
                audio.play_sfx(Sfx::ServiceBell);
                if !self.cooking_queue.is_empty() {
                    self.active_cook_dish = Some(self.cooking_queue.remove(0));
                }
            }
        } else if !self.cooking_queue.is_empty() {
            self.active_cook_dish = Some(self.cooking_queue.remove(0));
            self.cook_progress = 0;
        }

        let has_auto_tea = self.has_upgrade(RestaurantUpgrade::AutoTeaSporeUrn);
        for p in &mut self.patrons {
            match p.stage {
                PatronStage::WalkingIn => {
                    let target_x = STOOL_X[p.stool_idx];
                    if p.x > target_x {
                        p.x = (p.x - 6).max(target_x);
                    } else {
                        p.stage = PatronStage::SeatedOrdering;
                    }
                }
                PatronStage::SeatedOrdering => {
                    if p.wants_tea && !p.tea_served && has_auto_tea {
                        p.tea_served = true;
                        self.gold += 45;
                        self.night_earnings += 45;
                        self.popups.push(FloatingText {
                            text: "AUTO-TEA +$45".to_string(),
                            x: p.x - 16,
                            y: 78,
                            col: 34,
                            life: 12,
                        });
                    }
                    if self.tick % 4 == 0 && p.patience > 15 {
                        p.patience -= 1;
                    }
                }
                PatronStage::Eating(rem) => {
                    if rem <= 1 {
                        p.stage = PatronStage::SatisfiedLeaving;
                    } else {
                        p.stage = PatronStage::Eating(rem - 1);
                    }
                }
                PatronStage::SatisfiedLeaving => {
                    if p.x < 390 {
                        p.x += 6;
                    }
                }
            }
        }

        if input.action_pressed || input.jump_pressed {
            let near_pass = (self.chef_bar_x - PASS_X).abs() <= 24;
            if near_pass && self.carried_dish.is_none() {
                if !self.ready_pass_dishes.is_empty() {
                    self.carried_dish = Some(self.ready_pass_dishes.remove(0));
                    audio.play_sfx(Sfx::HarvestPluck);
                } else if self.active_cook_dish.is_some() {
                    self.cook_progress += 5;
                    audio.play_sfx(Sfx::WokSizzle);
                    self.popups.push(FloatingText {
                        text: "HEARTH STOKED!".to_string(),
                        x: 78,
                        y: 112,
                        col: 15,
                        life: 8,
                    });
                }
            } else if self.carried_dish.is_some() {
                let mut best_p = None;
                let mut best_dist = 34;
                for (i, p) in self.patrons.iter().enumerate() {
                    if p.stage == PatronStage::SeatedOrdering {
                        let d = (self.chef_bar_x - p.x).abs();
                        if d <= best_dist {
                            best_dist = d;
                            best_p = Some(i);
                        }
                    }
                }

                if let Some(pidx) = best_p {
                    let dish = self.carried_dish.take().unwrap();
                    let p = &mut self.patrons[pidx];
                    let tip = if p.tea_served { 35 } else { 0 };
                    let total_pay = dish.price + tip;
                    self.gold += total_pay;
                    self.night_earnings += total_pay;
                    self.dishes_served_tonight += 1;
                    p.stage = PatronStage::Eating(10);
                    self.portrait_mood = 3;
                    self.mood_timer = 10;
                    audio.play_sfx(Sfx::ServiceBell);

                    self.popups.push(FloatingText {
                        text: format!("+${} ({}*)", total_pay, dish.stars),
                        x: p.x - 16,
                        y: 84,
                        col: 15,
                        life: 16,
                    });
                }
            }
        }

        if input.bomb_pressed {
            for p in &mut self.patrons {
                if p.stage == PatronStage::SeatedOrdering
                    && p.wants_tea
                    && !p.tea_served
                    && (self.chef_bar_x - p.x).abs() <= 32
                {
                    p.tea_served = true;
                    self.gold += 45;
                    self.night_earnings += 45;
                    audio.play_sfx(Sfx::HarvestPluck);
                    self.popups.push(FloatingText {
                        text: "SPORE-TEA +$45!".to_string(),
                        x: p.x - 20,
                        y: 76,
                        col: 34,
                        life: 14,
                    });
                    break;
                }
            }
        }

        let all_done = self
            .patrons
            .iter()
            .all(|p| p.stage == PatronStage::SatisfiedLeaving && p.x >= 380);
        if input.extract_pressed || all_done {
            self.mode = GameMode::UpgradeShop;
            self.shop_cursor = 0;
            audio.play_sfx(Sfx::CoinStart);
        }
    }

    /// Phase 3: Restaurant & Galley Upgrade Shop
    fn update_upgrade_shop_15fps(&mut self, input: &InputState, audio: &mut ArcadeAudio) {
        let all_upgrades = RestaurantUpgrade::all();
        if input.up_pressed || input.left_pressed {
            self.shop_cursor = (self.shop_cursor + all_upgrades.len() - 1) % all_upgrades.len();
            audio.play_sfx(Sfx::SuctionStick);
        } else if input.down_pressed || input.right_pressed {
            self.shop_cursor = (self.shop_cursor + 1) % all_upgrades.len();
            audio.play_sfx(Sfx::SuctionStick);
        }

        if input.action_pressed || input.jump_pressed {
            let upg = all_upgrades[self.shop_cursor];
            if !self.has_upgrade(upg) && self.gold >= upg.cost() {
                self.gold -= upg.cost();
                self.upgrades.push(upg);
                audio.play_sfx(Sfx::ServiceBell);
            } else {
                audio.play_sfx(Sfx::SuctionStick);
            }
        }

        if input.extract_pressed {
            self.stage_num = if self.stage_num == 1 { 2 } else { 1 };
            audio.set_track((self.stage_num - 1) as usize);
            self.build_stage_for_current_level();
            self.mode = GameMode::PlanetaryDrop;
            audio.play_sfx(Sfx::CoinStart);
        }
    }

    fn move_x_by(&mut self, dx: i32, _allow_cling: bool) {
        let dir = dx.signum();
        let steps = dx.abs();
        let max_px = (WORLD_COLS as i32) * TS - COSMO_W;
        // When swimming horizontally, use a compact vertical profile (py+10..py+28) so submerged ledges never snag feet!
        let (y_top, y_bot) = if self.air_mode == AirborneMode::Swimming {
            (self.py + 10, self.py + 28)
        } else {
            (self.py + 6, self.py + COSMO_H - 4)
        };

        for _ in 0..steps {
            let next_px = (self.px + dir).clamp(0, max_px);
            let check_x = if dir < 0 {
                next_px + 12
            } else {
                next_px + 27
            };
            if self.solid_wall_at(check_x, y_top, y_bot).is_some() {
                // When grounded, automatically step/crawl over 1..2-tile obstacles!
                if self.air_mode == AirborneMode::Grounded && self.try_auto_crawl_obstacle(check_x) {
                    break;
                }
                break;
            } else {
                self.px = next_px;
            }
        }
    }

    /// Checks if the wall at `check_x` is a 1-tile or 2-tile obstacle with clear space on top;
    /// if so, initiates `AirborneMode::CrawlingOver` to automatically step up onto it!
    fn try_auto_crawl_obstacle(&mut self, check_x: i32) -> bool {
        let wall_col = (check_x / TS).clamp(0, WORLD_COLS as i32 - 1) as usize;
        let foot_row = ((self.py + COSMO_H) / TS).clamp(0, WORLD_ROWS as i32 - 1) as usize;

        for climb in 1..=2usize {
            if foot_row < climb + 2 {
                continue;
            }
            let ledge_row = foot_row - climb;
            let top_tile = self.tiles[ledge_row][wall_col];
            if !matches!(
                top_tile,
                Tile::EarthTop | Tile::TrunkWall | Tile::LilyPad | Tile::SporeVent
            ) {
                continue;
            }
            // Ensure the 2 tiles above the obstacle top are clear of solid walls
            let clear_above = !matches!(
                self.tiles[ledge_row - 1][wall_col],
                Tile::EarthTop | Tile::EarthFill | Tile::TrunkWall | Tile::SporeVent
            ) && !matches!(
                self.tiles[ledge_row - 2][wall_col],
                Tile::EarthTop | Tile::EarthFill | Tile::TrunkWall | Tile::SporeVent
            );
            if !clear_above {
                continue;
            }

            let target_y = (ledge_row as i32) * TS - COSMO_H;
            let target_x = (wall_col as i32) * TS - 8;

            if self
                .solid_wall_at(target_x + 12, target_y + 6, target_y + COSMO_H - 4)
                .is_none()
                && self
                    .solid_wall_at(target_x + 27, target_y + 6, target_y + COSMO_H - 4)
                    .is_none()
                && self.is_standing_on_ground(target_x, target_y)
            {
                self.air_mode = AirborneMode::CrawlingOver {
                    start_x: self.px,
                    start_y: self.py,
                    target_x,
                    target_y,
                    step: 0,
                    total: if climb == 1 { 3 } else { 5 },
                };
                return true;
            }
        }
        false
    }

    /// When walking off a ledge, checks if we are:
    /// 1) Stepping into Star-Pond water (smooth dive entry),
    /// 2) Stepping down a 1-tile rock/terrace (smooth 3-tick step down), OR
    /// 3) Crossing a deep 1..3-tile chasm with solid ground on the far side (automatic `GapLeap`)!
    fn try_auto_leap_or_water_entry(&mut self, dir: i32, audio: &ArcadeAudio) -> bool {
        let foot_row = ((self.py + COSMO_H) / TS).clamp(0, WORLD_ROWS as i32 - 1) as usize;
        let first_gap_col = if dir > 0 {
            ((self.px + 14) / TS).clamp(0, WORLD_COLS as i32 - 1)
        } else {
            ((self.px + 25) / TS).clamp(0, WORLD_COLS as i32 - 1)
        };
        let step_col = first_gap_col as usize;

        // 1. Check if the tile right below the ledge edge is Star-Pond water -> smooth water entry!
        let water_at_foot = matches!(
            self.tiles[foot_row][step_col],
            Tile::WaterSurface | Tile::WaterDeep
        );
        let water_below_foot = foot_row + 1 < WORLD_ROWS
            && matches!(
                self.tiles[foot_row + 1][step_col],
                Tile::WaterSurface | Tile::WaterDeep
            );
        if water_at_foot || water_below_foot {
            let water_row = if water_at_foot {
                foot_row
            } else {
                foot_row + 1
            };
            self.px = first_gap_col * TS - 4;
            self.py = (water_row as i32) * TS - 10;
            self.air_mode = AirborneMode::Swimming;
            audio.play_sfx(Sfx::SuctionStick);
            for i in -1..=1 {
                self.particles.push(Particle {
                    x: self.px + 20 + i * 6,
                    y: (water_row as i32) * TS,
                    vx: i * 2,
                    vy: -4,
                    col: 34,
                    life: 7,
                });
            }
            return true;
        }

        // 2. Check if stepping down a 1-tile rock or terrace -> smooth 3-tick step down (not a gap leap!)
        let step_clear = !matches!(
            self.tiles[foot_row][step_col],
            Tile::EarthTop | Tile::EarthFill | Tile::TrunkWall | Tile::SporeVent
        );
        if foot_row + 1 < WORLD_ROWS
            && step_clear
            && matches!(
                self.tiles[foot_row + 1][step_col],
                Tile::EarthTop | Tile::LilyPad | Tile::TrunkWall | Tile::SporeVent
            )
        {
            let target_x = first_gap_col * TS - 8;
            let target_y = ((foot_row + 1) as i32) * TS - COSMO_H;
            if self.is_standing_on_ground(target_x, target_y) {
                self.air_mode = AirborneMode::CrawlingOver {
                    start_x: self.px,
                    start_y: self.py,
                    target_x,
                    target_y,
                    step: 0,
                    total: 3,
                };
                return true;
            }
        }

        // 3. Only if `step_col` is an actual chasm (not solid ground at `foot_row` or `foot_row + 1`),
        // scan 1..=3 tiles ahead for solid ground across the chasm -> auto-leap!
        if !step_clear
            || (foot_row + 1 < WORLD_ROWS
                && matches!(
                    self.tiles[foot_row + 1][step_col],
                    Tile::EarthTop | Tile::EarthFill | Tile::TrunkWall | Tile::SporeVent
                ))
        {
            return false;
        }

        for gap_width in 1..=3i32 {
            let land_col_i32 = first_gap_col + dir * gap_width;
            if land_col_i32 < 0 || land_col_i32 >= WORLD_COLS as i32 {
                continue;
            }
            let land_col = land_col_i32 as usize;

            for &land_row in &[foot_row, foot_row.saturating_sub(1)] {
                if land_row < 2 {
                    continue;
                }
                let t = self.tiles[land_row][land_col];
                if !matches!(
                    t,
                    Tile::EarthTop | Tile::TrunkWall | Tile::LilyPad | Tile::SporeVent
                ) {
                    continue;
                }
                let clear_above = !matches!(
                    self.tiles[land_row - 1][land_col],
                    Tile::EarthTop | Tile::EarthFill | Tile::TrunkWall | Tile::SporeVent
                ) && !matches!(
                    self.tiles[land_row - 2][land_col],
                    Tile::EarthTop | Tile::EarthFill | Tile::TrunkWall | Tile::SporeVent
                );
                if !clear_above {
                    continue;
                }

                let target_y = (land_row as i32) * TS - COSMO_H;
                let target_x = (land_col as i32) * TS - 8;

                if self
                    .solid_wall_at(target_x + 12, target_y + 6, target_y + COSMO_H - 4)
                    .is_none()
                    && self
                        .solid_wall_at(target_x + 27, target_y + 6, target_y + COSMO_H - 4)
                        .is_none()
                    && self.is_standing_on_ground(target_x, target_y)
                {
                    self.air_mode = AirborneMode::GapLeap {
                        start_x: self.px,
                        start_y: self.py,
                        target_x,
                        target_y,
                        step: 0,
                        total: (4 + gap_width * 2).clamp(5, 8),
                    };
                    audio.play_sfx(Sfx::SuctionStick);
                    return true;
                }
            }
        }
        false
    }

    /// While swimming near the surface edge of any Star-Pond or Flooded Ravine, automatically clambers out onto the bank/pier!
    fn try_clamber_out_of_water(&mut self, dir: i32, audio: &ArcadeAudio) -> bool {
        let probe_x = if dir < 0 {
            self.px + 6
        } else {
            self.px + 33
        };
        let bank_col = (probe_x / TS).clamp(0, WORLD_COLS as i32 - 1) as usize;

        for ledge_row in 3..(WORLD_ROWS - 1) {
            let t = self.tiles[ledge_row][bank_col];
            if !matches!(t, Tile::EarthTop | Tile::TrunkWall | Tile::SporeVent) {
                continue;
            }
            let clear_above = !matches!(
                self.tiles[ledge_row - 1][bank_col],
                Tile::EarthTop
                    | Tile::EarthFill
                    | Tile::TrunkWall
                    | Tile::SporeVent
                    | Tile::WaterSurface
                    | Tile::WaterDeep
            ) && !matches!(
                self.tiles[ledge_row - 2][bank_col],
                Tile::EarthTop
                    | Tile::EarthFill
                    | Tile::TrunkWall
                    | Tile::SporeVent
                    | Tile::WaterSurface
                    | Tile::WaterDeep
            );
            if !clear_above {
                continue;
            }
            // Only clamber out when swimming in the upper portion of the water near the bank top
            if self.py > (ledge_row as i32 + 1) * TS
                || self.py < (ledge_row as i32 - 3) * TS
            {
                continue;
            }

            let target_y = (ledge_row as i32) * TS - COSMO_H;
            let target_x = (bank_col as i32) * TS - 8;

            if self
                .solid_wall_at(target_x + 12, target_y + 6, target_y + COSMO_H - 4)
                .is_none()
                && self
                    .solid_wall_at(target_x + 27, target_y + 6, target_y + COSMO_H - 4)
                    .is_none()
                && self.is_standing_on_ground(target_x, target_y)
            {
                self.air_mode = AirborneMode::CrawlingOver {
                    start_x: self.px,
                    start_y: self.py,
                    target_x,
                    target_y,
                    step: 0,
                    total: 4,
                };
                audio.play_sfx(Sfx::SuctionStick);
                return true;
            }
        }
        false
    }

    fn move_y_by(&mut self, dy: i32) -> bool {
        if dy == 0 {
            return false;
        }
        let dir = dy.signum();
        let steps = dy.abs();
        for _ in 0..steps {
            if dir > 0 && self.is_standing_on_ground(self.px, self.py) {
                return true;
            }
            let next_py = (self.py + dir).clamp(4, (WORLD_ROWS as i32 - 1) * TS - COSMO_H);
            if dir > 0 {
                if self.is_standing_on_ground(self.px, next_py) {
                    self.py = next_py;
                    return true;
                }
                // Also prevent swimming downward from passing through unaligned solid floors!
                let foot_y = next_py + COSMO_H;
                if self.solid_wall_at(self.px + 14, foot_y - 1, foot_y).is_some()
                    || self.solid_wall_at(self.px + 25, foot_y - 1, foot_y).is_some()
                {
                    let floor_row = foot_y / TS;
                    self.py = floor_row * TS - COSMO_H;
                    return true;
                }
            } else if self.solid_ceiling_at(self.px + 13, self.px + 26, next_py + 2) {
                return true;
            }
            self.py = next_py;
        }
        false
    }

    fn is_standing_on_ground(&self, px: i32, py: i32) -> bool {
        let foot_y = py + COSMO_H;
        if foot_y % TS != 0 {
            return false;
        }
        let r = (foot_y / TS) as usize;
        if r >= WORLD_ROWS {
            return true;
        }
        for x in [px + 14, px + 20, px + 25] {
            let c = (x / TS).clamp(0, WORLD_COLS as i32 - 1) as usize;
            let t = self.tiles[r][c];
            let above_clear = r == 0
                || !matches!(
                    self.tiles[r - 1][c],
                    Tile::EarthTop | Tile::EarthFill | Tile::TrunkWall | Tile::SporeVent
                );
            if above_clear
                && matches!(
                    t,
                    Tile::EarthTop | Tile::TrunkWall | Tile::LilyPad | Tile::SporeVent
                )
            {
                return true;
            }
        }
        false
    }

    fn solid_wall_at(&self, x: i32, y0: i32, y1: i32) -> Option<Tile> {
        let c = (x / TS).clamp(0, WORLD_COLS as i32 - 1) as usize;
        for y in [y0, (y0 + y1) / 2, y1] {
            if y < 0 {
                continue;
            }
            let r = (y / TS).clamp(0, WORLD_ROWS as i32 - 1) as usize;
            let t = self.tiles[r][c];
            if matches!(
                t,
                Tile::EarthTop | Tile::EarthFill | Tile::TrunkWall | Tile::SporeVent
            ) {
                return Some(t);
            }
        }
        None
    }

    fn solid_ceiling_at(&self, x0: i32, x1: i32, y: i32) -> bool {
        if y < 0 {
            return true;
        }
        let r = (y / TS).clamp(0, WORLD_ROWS as i32 - 1) as usize;
        for x in [x0, (x0 + x1) / 2, x1] {
            let c = (x / TS).clamp(0, WORLD_COLS as i32 - 1) as usize;
            let t = self.tiles[r][c];
            if matches!(
                t,
                Tile::EarthTop | Tile::EarthFill | Tile::TrunkWall | Tile::SporeVent
            ) {
                return true;
            }
        }
        false
    }

    fn update_camera_instant(&mut self) {
        let max_cam_x = (WORLD_COLS as i32) * TS - FB_W as i32;
        let max_cam_y = (WORLD_ROWS as i32) * TS - PLAY_H;
        self.cam_x = (self.px + 20 - (FB_W as i32) / 2).clamp(0, max_cam_x);
        self.cam_y = (self.py + 22 - PLAY_H / 2).clamp(0, max_cam_y);
    }

    fn update_camera_stepped(&mut self) {
        let max_cam_x = (WORLD_COLS as i32) * TS - FB_W as i32;
        let max_cam_y = (WORLD_ROWS as i32) * TS - PLAY_H;
        let target_cx = (self.px + 20 - (FB_W as i32) / 2).clamp(0, max_cam_x);
        let target_cy = (self.py + 22 - PLAY_H / 2).clamp(0, max_cam_y);

        let dx = target_cx - self.cam_x;
        let dy = target_cy - self.cam_y;
        if dx.abs() <= 6 {
            self.cam_x = target_cx;
        } else {
            self.cam_x += (dx / 2) & !1;
        }
        if dy.abs() <= 6 {
            self.cam_y = target_cy;
        } else {
            self.cam_y += (dy / 2) & !1;
        }
    }

    fn collect_ingredient(&mut self, kind: IngKind, prep: PrepState, x: i32, y: i32) {
        let start_ionized =
            (kind == IngKind::ReedEel) || self.backpack.iter().any(|b| b.kind == IngKind::ReedEel);

        if kind == IngKind::ReedEel {
            for item in &mut self.backpack {
                item.ionized = true;
            }
        }

        self.portrait_mood = 3;
        self.mood_timer = 8;

        if self.backpack.len() < 6 {
            self.backpack.push(BagItem {
                kind,
                prep,
                ionized: start_ionized,
            });
            self.popups.push(FloatingText {
                text: format!("+{} [{}]", kind.name(), prep.label()),
                x,
                y: y - 8,
                col: kind.steady_color(),
                life: 14,
            });
        }
    }

    // =========================================================================
    // RENDERER (Zero Flashing Text, Moody Terrain, Dave-the-Diver Galley)
    // =========================================================================
    pub fn render(&self, fb: &mut Framebuffer, assets: &AssetBank, track_title: &str) {
        fb.clear(16);
        match self.mode {
            GameMode::TitleAttractor => self.render_title(fb, assets, track_title),
            GameMode::PlanetaryDrop => self.render_planetary_drop(fb, assets, track_title),
            GameMode::MenuPrep => self.render_menu_prep(fb, assets, track_title),
            GameMode::GalleyService => self.render_galley_service(fb, assets, track_title),
            GameMode::UpgradeShop => self.render_upgrade_shop(fb, assets, track_title),
        }

        if self.show_help_overlay {
            self.render_help_overlay(fb);
        }
    }

    fn render_title(&self, fb: &mut Framebuffer, assets: &AssetBank, track_title: &str) {
        fb.blit_parallax_bg(&assets.bg_canopy, self.tick as i32, 0, FB_H);

        fb.rect_bevel(8, 6, 384, 38, 2, 4, 1, 1);
        fb.draw_text_centered("INTERSTELLAR FORAGER", 200, 12, 15, Some(1), 2);
        fb.draw_text_centered(
            "THE ENCHANTED FOREST OF NAIBIHPMA (15 FPS VGA)",
            200,
            31,
            34,
            Some(1),
            1,
        );

        fb.rect_bevel(14, 50, 372, 162, 2, 4, 1, 1);
        fb.draw_text_centered(
            "NAIBIHPMA CAST & INGREDIENTS (64-COLOR VGA LUT)",
            200,
            56,
            6,
            Some(1),
            1,
        );

        let labels = [
            "TWEN NEWT",
            "VOCAL-ROE",
            "SPORE-LOTUS",
            "REED-EEL",
            "THE FORAGER",
            "DOAT BARON",
            "PLASMA-WOK",
            "BOG CLOCHE",
        ];
        for i in 0..8 {
            let col = i % 4;
            let row = i / 4;
            let bx = 26 + (col as i32) * 88;
            let by = 68 + (row as i32) * 68;
            fb.rect_bevel(bx, by, 76, 52, 16, 3, 1, 1);
            if let Some(spr) = assets.ai_sprites_44.get(i) {
                fb.blit_sprite(spr, bx + 16, by + 4, false, false, None);
            }
            fb.draw_text_centered(labels[i], bx + 38, by + 55, 7, Some(1), 1);
        }

        fb.rect_bevel(14, 216, 372, 54, 2, 4, 1, 1);
        fb.draw_text_centered(
            &format!("NOW PLAYING: {} ([/] SKIP)", track_title),
            200,
            222,
            25,
            Some(1),
            1,
        );
        fb.draw_text_centered(
            "[1]:LV1 UNDERBRUSH  [2]:LV2 DAOT  [3]:PUZZLE  [4-6]:GALLEY",
            200,
            235,
            7,
            Some(1),
            1,
        );
        // Steady, calm prompt (zero flashing!)
        fb.draw_text_centered(
            "PRESS [SPACE] OR [ENTER] TO BEGIN",
            200,
            252,
            15,
            Some(1),
            1,
        );

        fb.rect_bevel(14, 274, 372, 22, 1, 3, 1, 1);
        fb.draw_text("64-COLOR VGA LUT:", 20, 278, 5, None, 1);
        for c in 0..64u8 {
            let x = 126 + (c as i32) * 4;
            fb.rect_fill(x, 278, 3, 14, c);
        }
    }

    fn render_planetary_drop(&self, fb: &mut Framebuffer, assets: &AssetBank, track_title: &str) {
        fb.blit_parallax_bg(&assets.bg_playfield, self.cam_x / 2, 32, PLAY_H as usize);

        let c0 = (self.cam_x / TS).max(0) as usize;
        let c1 = ((self.cam_x + FB_W as i32) / TS + 1).min(WORLD_COLS as i32) as usize;
        let r0 = (self.cam_y / TS).max(0) as usize;
        let r1 = ((self.cam_y + PLAY_H) / TS + 1).min(WORLD_ROWS as i32) as usize;

        for r in r0..r1 {
            let wy = (r as i32) * TS - self.cam_y;
            for c in c0..c1 {
                let wx = (c as i32) * TS - self.cam_x;
                let var = (c + r * 3) & 3;
                match self.tiles[r][c] {
                    Tile::Empty => {}
                    Tile::EarthTop => {
                        fb.blit_sprite(&assets.tile_earth_top[var], wx, wy, false, false, None)
                    }
                    Tile::EarthFill => {
                        fb.blit_sprite(&assets.tile_earth_fill[var], wx, wy, false, false, None)
                    }
                    Tile::TrunkWall => {
                        fb.blit_sprite(&assets.tile_trunk, wx, wy, false, false, None)
                    }
                    Tile::LilyPad => fb.blit_sprite(&assets.tile_lily, wx, wy, false, false, None),
                    Tile::GrappleAnchor => {
                        fb.blit_sprite(&assets.tile_grapple_anchor, wx, wy, false, false, None);
                    }
                    Tile::WaterSurface => {
                        fb.blit_sprite(&assets.tile_water_top, wx, wy, false, false, None);
                    }
                    Tile::WaterDeep => {
                        fb.blit_sprite(&assets.tile_water_deep, wx, wy, false, false, None);
                    }
                    Tile::DropPod => {
                        fb.rect_dither(wx + 4, 34, 16, wy + TS - 34, 33, 1);
                        fb.rect_bevel(wx, wy, TS, TS, 3, 6, 2, 1);
                        fb.rect_fill(wx + 6, wy + 6, 12, 12, 34);
                    }
                    Tile::ShadowAlcove => {
                        // Hollow-bark concealment alcove: deep shadow archway with mossy frame
                        fb.rect_fill(wx, wy, TS, TS, 16);
                        fb.rect_dither(wx + 2, wy + 2, TS - 4, TS - 4, 1, 16);
                        fb.rect_fill(wx, wy, 2, TS, 19);
                        fb.rect_fill(wx + TS - 2, wy, 2, TS, 19);
                    }
                    Tile::SporeVent => {
                        // Cracked toxic spore-geyser fissure
                        fb.blit_sprite(&assets.tile_earth_top[var], wx, wy, false, false, None);
                        fb.rect_bevel(wx + 5, wy - 3, 14, 7, 28, 30, 1, 1);
                        let phase = (self.tick + (c as u64) * 11) % 45;
                        if phase >= 32 {
                            // Erupting 2-tile violet spore plume!
                            fb.rect_dither(wx + 4, wy - 44, 16, 42, 30, 28);
                            fb.circle_fill(wx + 12, wy - 38, 6, 30);
                            fb.circle_fill(wx + 12, wy - 20, 5, 29);
                            fb.draw_text_centered("SPORES!", wx + 12, wy - 54, 30, Some(1), 1);
                        } else if phase >= 24 {
                            // Telegraph bubbling before eruption
                            fb.circle_fill(wx + 12, wy - 6, 3, 30);
                        }
                    }
                    Tile::OxygenReed => {
                        // Bioluminescent cyan bubble-reed cluster (+30% O2 refill!)
                        if r > 0
                            && matches!(
                                self.tiles[r - 1][c],
                                Tile::WaterSurface | Tile::WaterDeep
                            )
                        {
                            fb.blit_sprite(&assets.tile_water_deep, wx, wy, false, false, None);
                        }
                        fb.line(wx + 8, wy + 22, wx + 7, wy + 6, 24);
                        fb.line(wx + 12, wy + 22, wx + 12, wy + 4, 25);
                        fb.line(wx + 16, wy + 22, wx + 17, wy + 7, 24);
                        fb.circle_fill(wx + 7, wy + 6, 3, 34);
                        fb.circle_fill(wx + 12, wy + 4, 3, 15);
                        fb.circle_fill(wx + 17, wy + 7, 3, 34);
                        fb.draw_text_centered("+O2", wx + 12, wy - 8, 34, Some(1), 1);
                    }
                    Tile::SluiceLever => {
                        // Interactive world-state puzzle switch (Sluice Gate / Counterweight Bridge)
                        fb.rect_bevel(wx + 4, wy + 14, 16, 10, 19, 6, 1, 1);
                        if self.world_switch_pulled {
                            fb.line(wx + 12, wy + 14, wx + 20, wy + 3, 25);
                            fb.circle_fill(wx + 20, wy + 3, 3, 25);
                            fb.draw_text_centered("[PULLED]", wx + 12, wy - 10, 25, Some(1), 1);
                        } else {
                            fb.line(wx + 12, wy + 14, wx + 4, wy + 3, 15);
                            fb.circle_fill(wx + 4, wy + 3, 3, 38);
                            let lbl = if self.stage_num <= 1 {
                                "[X:FLOOD]"
                            } else {
                                "[X:BRIDGE]"
                            };
                            fb.draw_text_centered(lbl, wx + 12, wy - 10, 15, Some(1), 1);
                        }
                    }
                }
            }
        }

        // Highlight nearby Grapple-Root Anchor Ring if within harpoon winch range!
        if let Some((_, _, ax, ay)) = self.find_grapple_anchor(true) {
            let sx = ax - self.cam_x;
            let sy = ay - self.cam_y;
            fb.draw_text_centered("[UP+X:WINCH]", sx, sy - 18, 15, Some(1), 1);
        }

        // Render Active Bioluminescent Bait-Net Trap ([C])
        if let Some(ref trap) = self.active_trap {
            let tx = trap.x - self.cam_x;
            let ty = trap.y - self.cam_y;
            fb.rect_bevel(tx - 14, ty + 2, 28, 8, 18, 34, 1, 1);
            fb.circle_fill(tx, ty, 4, 25);
            fb.draw_text_centered("SPORE-LURE", tx, ty - 10, 34, Some(1), 1);
        }

        // Render Field Creatures & Ecological States
        for e in &self.entities {
            if !e.alive {
                continue;
            }
            let sx = e.x - self.cam_x;
            let sy = e.y - self.cam_y;
            if sx < -48 || sx > FB_W as i32 + 16 || sy < -48 || sy > PLAY_H {
                continue;
            }

            if e.burrowed {
                // Skittish Newt hiding in its hollow root-burrow!
                fb.rect_bevel(sx + 4, sy + 18, 26, 14, 19, 20, 1, 1);
                fb.draw_text_centered("[BURROWED]", sx + 17, sy + 6, 14, Some(1), 1);
                continue;
            }

            let bob = if e.sleep_ticks > 0 || ((self.tick / 4 + e.x as u64) & 1) == 0 {
                0
            } else {
                -2
            };

            if e.behavior == CreatureBehavior::ArmoredToad {
                let spr = if e.sleep_ticks > 0 {
                    &assets.toad_bounce
                } else {
                    &assets.toad_idle
                };
                fb.blit_sprite(spr, sx, sy, false, false, None);
                if let Some(icon) = assets.ai_sprites_22.get(e.kind as usize) {
                    fb.blit_sprite(icon, sx + 9, sy - 20 + bob, false, false, None);
                }
                if e.sleep_ticks == 0 {
                    // Draw Armored Spore-Shield arc in front of Doat!
                    fb.line(sx - 4, sy + 4, sx - 6, sy + 26, 15);
                    fb.line(sx - 3, sy + 4, sx - 5, sy + 26, 34);
                    fb.draw_text_centered("SHIELD [Z:TRANQ]", sx + 18, sy - 30, 14, Some(1), 1);
                }
            } else if let Some(icon) = assets.ai_sprites_34.get(e.kind as usize) {
                fb.blit_sprite(icon, sx, sy + bob, e.vx < 0, false, None);
                if e.behavior == CreatureBehavior::VoltaicEel && e.sleep_ticks == 0 {
                    if e.shock_phase >= 28 {
                        // Voltaic discharge arcs!
                        fb.line(sx - 4, sy + 8, sx + 38, sy + 24, 15);
                        fb.line(sx + 38, sy + 6, sx - 4, sy + 26, 34);
                        fb.draw_text_centered("[CHARGED!]", sx + 17, sy - 10, 38, Some(1), 1);
                    } else {
                        fb.draw_text_centered("[COOLED: X/Z]", sx + 17, sy - 10, 34, Some(1), 1);
                    }
                } else if e.behavior == CreatureBehavior::SkittishNewt && e.sleep_ticks == 0 {
                    fb.draw_text_centered("SKITTISH [C:TRAP]", sx + 17, sy - 10, 25, Some(1), 1);
                }
            }

            if e.sleep_ticks > 0 {
                fb.draw_text_centered("ZZZ... [X:3*]", sx + 17, sy - 12, 34, Some(1), 1);
            }
        }

        // Render Daot's Hypnotic Spore-Orbs
        for orb in &self.orbs {
            let sx = orb.x - self.cam_x;
            let sy = orb.y - self.cam_y;
            fb.circle_fill(sx, sy, 5, 30);
            fb.circle_fill(sx, sy, 2, 15);
        }

        // Render Soporific Tranq Darts ([Z] / [SPACE])
        for d in &self.darts {
            let sx = d.x - self.cam_x;
            let sy = d.y - self.cam_y;
            fb.rect_fill(sx - 3, sy - 2, 7, 4, 34);
            fb.rect_fill(sx - 1, sy - 1, 3, 2, 25);
        }

        let cosmo_sx = self.px - self.cam_x;
        let cosmo_sy = self.py - self.cam_y;

        if self.is_hidden_in_shadow() {
            fb.draw_text_centered("[HIDDEN]", cosmo_sx + 20, cosmo_sy - 10, 25, Some(1), 1);
        }

        // Render Subtle 8-Way Aim Reticle (3 steady cyan pixel dots)
        if self.pluck_timer == 0 {
            for step in 1..=3 {
                let rx = cosmo_sx + 20 + self.aim_dx * (step * 18);
                let ry = cosmo_sy + 20 + self.aim_dy * (step * 18);
                let col = if step == 3 { 15 } else { 34 };
                fb.rect_fill(rx - 1, ry - 1, 2, 2, col);
            }
        }

        // Render Pneumatic Harpoon & Winch Cable ([X])
        if let Some((tx, ty)) = self.tether_target {
            let tsx = tx - self.cam_x;
            let tsy = ty - self.cam_y;
            fb.line(cosmo_sx + 20, cosmo_sy + 20, tsx, tsy, 6);
            fb.line(cosmo_sx + 20, cosmo_sy + 21, tsx, tsy + 1, 34);
            fb.circle_fill(tsx, tsy, 5, 15);
            fb.circle_fill(tsx, tsy, 2, 34);
        }

        let (cosmo_spr, flip_x, y_nudge) = if self.pluck_timer > 0 {
            (&assets.cosmo_pluck, !self.facing_right, 2)
        } else {
            match self.air_mode {
                AirborneMode::CrawlingOver { .. } | AirborneMode::Falling(_) => {
                    (&assets.cosmo_crawl, !self.facing_right, 2)
                }
                AirborneMode::GapLeap { .. } | AirborneMode::GrappleWinch { .. } => {
                    (&assets.cosmo_jump, !self.facing_right, 0)
                }
                AirborneMode::Swimming => {
                    if self.is_standing_on_ground(self.px, self.py) {
                        if self.moving_x {
                            match (self.walk_anim_tick / 2) % 4 {
                                0 => (&assets.cosmo_run1, !self.facing_right, 2),
                                1 => (&assets.cosmo_run2, !self.facing_right, 2),
                                2 => (&assets.cosmo_run3, !self.facing_right, 2),
                                _ => (&assets.cosmo_run2, !self.facing_right, 2),
                            }
                        } else {
                            (&assets.cosmo_idle, !self.facing_right, 2)
                        }
                    } else {
                        let swim_bob = if self.moving_x {
                            ((self.walk_anim_tick / 2) % 2) as i32 * 2
                        } else {
                            0
                        };
                        (&assets.cosmo_cling_right, !self.facing_right, swim_bob)
                    }
                }
                AirborneMode::Grounded => {
                    if self.moving_x {
                        match (self.walk_anim_tick / 2) % 4 {
                            0 => (&assets.cosmo_run1, !self.facing_right, 2),
                            1 => (&assets.cosmo_run2, !self.facing_right, 2),
                            2 => (&assets.cosmo_run3, !self.facing_right, 2),
                            _ => (&assets.cosmo_run2, !self.facing_right, 2),
                        }
                    } else {
                        (&assets.cosmo_idle, !self.facing_right, 2)
                    }
                }
            }
        };
        fb.blit_sprite(
            cosmo_spr,
            cosmo_sx,
            cosmo_sy + y_nudge,
            flip_x,
            false,
            None,
        );

        for p in &self.particles {
            fb.rect_fill(p.x - self.cam_x, p.y - self.cam_y, 3, 3, p.col);
        }
        for ft in &self.popups {
            fb.draw_text(
                &ft.text,
                ft.x - self.cam_x,
                ft.y - self.cam_y,
                ft.col,
                Some(1),
                1,
            );
        }

        // Calm, non-flashing Top Level & Zone Banner
        let (_step_num, step_title, step_prompt) = self.current_tutorial_zone();
        fb.rect_bevel(6, 4, 388, 28, 2, 4, 1, 1);
        fb.draw_text(step_title, 12, 8, 15, Some(1), 1);
        fb.draw_text("[1:LV1 2:LV2]", 304, 8, 34, Some(1), 1);
        fb.draw_text(step_prompt, 12, 20, 7, Some(1), 1);

        self.render_apogee_status_bar(
            fb,
            assets,
            track_title,
            "ARROWS:WALK/SWIM  [X]:HARPOON/LEVER  [Z/SPACE]:TRANQ  [C]:TRAP",
        );
    }

    /// Renders Phase 2A: Pre-Service Dish Combiner (Combining ingredients into dishes ahead of time!)
    fn render_menu_prep(&self, fb: &mut Framebuffer, assets: &AssetBank, track_title: &str) {
        fb.blit_parallax_bg(&assets.bg_galley, 0, 0, PLAY_H as usize);
        fb.rect_dither(0, 0, FB_W as i32, PLAY_H, 1, 2);

        fb.rect_bevel(8, 4, 384, 24, 2, 4, 1, 1);
        if self.blackout_occurred {
            fb.draw_text(
                "O2 BLACKOUT! DRONE RECOVERY SAVED 1 SPECIMEN!",
                14,
                8,
                38,
                Some(1),
                1,
            );
        } else {
            fb.draw_text(
                "PRE-SERVICE PREP: COMBINE 2 ITEMS PER DISH",
                14,
                8,
                15,
                Some(1),
                1,
            );
        }
        fb.draw_text(
            "UP/DOWN+[X]:PAIR ITEMS   [C]:AUTO   [E]:SERVE!",
            14,
            18,
            34,
            Some(1),
            1,
        );

        // Left Column: Harvested Backpack Ingredients (x = 8..184)
        fb.rect_bevel(8, 32, 176, 198, 2, 4, 1, 1);
        fb.draw_text(
            &format!("HARVESTED HAUL ({})", self.backpack.len()),
            16,
            38,
            15,
            Some(1),
            1,
        );

        if self.backpack.is_empty() {
            fb.draw_text("ALL INGREDIENTS COMBINED!", 16, 96, 25, Some(1), 1);
            fb.draw_text("PRESS [E] OR [SPACE]", 16, 112, 15, Some(1), 1);
            fb.draw_text("TO OPEN THE TAVERN!", 16, 124, 15, Some(1), 1);
        } else {
            for (i, item) in self.backpack.iter().enumerate() {
                let ry = 52 + (i as i32) * 28;
                let is_cursor = self.prep_cursor == i;
                let is_first = self.selected_first_ing == Some(i);
                let row_bg = if is_first {
                    28
                } else if is_cursor {
                    17
                } else {
                    16
                };
                let border = if is_first || is_cursor { 15 } else { 3 };
                fb.rect_bevel(14, ry, 164, 25, row_bg, border, 1, 1);

                if let Some(spr) = assets.ai_sprites_22.get(item.kind as usize) {
                    fb.blit_sprite(spr, 18, ry + 1, false, false, None);
                }
                let prefix = if is_first { "[1ST] " } else { "" };
                fb.draw_text(
                    &format!("{}{}", prefix, item.kind.name()),
                    44,
                    ry + 4,
                    7,
                    Some(1),
                    1,
                );
                fb.draw_text(
                    &format!(
                        "{}{}",
                        item.prep.label(),
                        if item.ionized { " +IONIZED" } else { "" }
                    ),
                    44,
                    ry + 14,
                    item.kind.steady_color(),
                    Some(1),
                    1,
                );
            }
        }

        // Right Column: Curated Dishes & Active Restaurant Upgrades (x = 190..392)
        fb.rect_bevel(190, 32, 202, 128, 2, 4, 1, 1);
        fb.draw_text(
            &format!("TONIGHT'S CURATED MENU ({}/4)", self.menu_dishes.len()),
            198,
            38,
            15,
            Some(1),
            1,
        );

        if self.menu_dishes.is_empty() {
            fb.draw_text("SELECT 2 ITEMS ON THE LEFT", 198, 70, 6, None, 1);
            fb.draw_text("WITH [X] TO CREATE A DISH!", 198, 84, 6, None, 1);
        } else {
            for (i, d) in self.menu_dishes.iter().enumerate() {
                let dy = 52 + (i as i32) * 26;
                fb.rect_bevel(196, dy, 190, 24, 16, 4, 1, 1);
                if let Some(spr) = assets.ai_sprites_22.get(7) {
                    fb.blit_sprite(spr, 198, dy + 1, false, false, None);
                }
                fb.draw_text(&d.name[..d.name.len().min(20)], 222, dy + 4, 7, Some(1), 1);
                fb.draw_text(
                    &format!("PRICE: ${}  ({} STARS)", d.price, d.stars),
                    222,
                    dy + 14,
                    15,
                    Some(1),
                    1,
                );
            }
        }

        // Active Restaurant Upgrades Box (x = 190, y = 164..230)
        fb.rect_bevel(190, 164, 202, 66, 2, 4, 1, 1);
        fb.draw_text(
            &format!("ACTIVE TAVERN UPGRADES ({}/4)", self.upgrades.len()),
            198,
            170,
            34,
            Some(1),
            1,
        );
        if self.upgrades.is_empty() {
            fb.draw_text("NONE YET (BUY AFTER SERVICE", 198, 186, 5, None, 1);
            fb.draw_text("WITH YOUR TAVERN EARNINGS!)", 198, 198, 5, None, 1);
        } else {
            for (i, u) in self.upgrades.iter().take(3).enumerate() {
                fb.draw_text(
                    &format!("+ {}", u.title()),
                    198,
                    184 + (i as i32) * 12,
                    25,
                    Some(1),
                    1,
                );
            }
        }

        self.render_apogee_status_bar(
            fb,
            assets,
            track_title,
            "UP/DOWN:SELECT  [X]:PAIR ITEMS  [C]:AUTO-PAIR  [E]:START SERVICE",
        );
    }

    /// Renders Phase 2B: Dave the Diver-style Side-View Tavern Floor inside `bg_galley`!
    fn render_galley_service(&self, fb: &mut Framebuffer, assets: &AssetBank, track_title: &str) {
        // 1. Render the Quantized AI Side-View Naibihpma Bog-Tavern Interior!
        fb.blit_parallax_bg(&assets.bg_galley, 0, 0, PLAY_H as usize);

        // 2. Top Service Status Banner (Calm, non-flashing!)
        fb.rect_bevel(6, 4, 388, 22, 2, 4, 1, 1);
        let top_hint = if self.patrons.iter().all(|p| p.stage == PatronStage::SatisfiedLeaving) {
            "NIGHT DONE! PRESS [E] FOR UPGRADE SHOP!"
        } else if self.carried_dish.is_some() {
            "WALK TO MATCHING PATRON & PRESS [X] TO SERVE"
        } else if !self.ready_pass_dishes.is_empty() {
            "DISH READY AT LEFT PASS! WALK LEFT & PRESS [X]"
        } else {
            "COOKING AT LEFT PASS! TAP [X] TO STOKE HEARTH"
        };
        fb.draw_text(top_hint, 12, 8, 15, Some(1), 1);
        fb.draw_text(
            &format!(
                "TONIGHT: +${}    TOTAL GOLD: ${}",
                self.night_earnings, self.gold
            ),
            12,
            17,
            34,
            Some(1),
            1,
        );

        // 3. Kitchen Pass Cooking Progress & Ready Cloche Dishes (Left counter x = 88..154)
        if let Some(ref cooking) = self.active_cook_dish {
            fb.rect_bevel(68, 114, 84, 20, 2, 4, 1, 1);
            fb.draw_text("COOKING...", 74, 117, 15, Some(1), 1);
            let pct_w = ((self.cook_progress * 72) / cooking.cook_ticks_needed.max(1)).clamp(0, 72);
            fb.rect_fill(74, 126, 72, 5, 1);
            fb.rect_fill(74, 126, pct_w, 5, 25);
        }

        // Draw finished cloche dishes sitting on the Kitchen Pass counter (`x = 128, y = 136`)
        for (i, _d) in self.ready_pass_dishes.iter().enumerate() {
            if let Some(cloche) = assets.ai_sprites_34.get(7) {
                fb.blit_sprite(
                    cloche,
                    118 + (i as i32) * 12,
                    126,
                    false,
                    false,
                    None,
                );
            }
        }
        if !self.ready_pass_dishes.is_empty() {
            fb.draw_text("PASS:[X]", 112, 162, 15, Some(1), 1);
        }

        // 4. Render Amphibian Patrons Seated at the 4 Barstools (`x = 194, 234, 274, 314`)
        for p in &self.patrons {
            if p.x >= 384 && p.stage == PatronStage::SatisfiedLeaving {
                continue;
            }
            // Patron sits on the barstool (`y = 124`)
            if let Some(toad_spr) = assets.ai_sprites_44.get(5) {
                fb.blit_sprite(
                    toad_spr,
                    p.x - 20,
                    118,
                    p.stage != PatronStage::SatisfiedLeaving,
                    false,
                    None,
                );
            }

            // Order Thought Bubble above seated patron (`y = 58..112`)
            match p.stage {
                PatronStage::SeatedOrdering => {
                    let bx = p.x - 18;
                    let by = 56;
                    fb.rect_bevel(bx, by, 38, 52, 2, 6, 1, 1);
                    // Show the two ingredient icons that make up their ordered dish
                    if let Some(icon1) = assets.ai_sprites_22.get(p.ordered_dish.primary_kind as usize)
                    {
                        fb.blit_sprite(icon1, bx + 2, by + 3, false, false, None);
                    }
                    if let Some(icon2) =
                        assets.ai_sprites_22.get(p.ordered_dish.secondary_kind as usize)
                    {
                        fb.blit_sprite(icon2, bx + 15, by + 12, false, false, None);
                    }
                    // Patience bar
                    let pat_w = ((p.patience * 30) / p.max_patience.max(1)).clamp(2, 30);
                    fb.rect_fill(bx + 4, by + 36, 30, 4, 1);
                    fb.rect_fill(bx + 4, by + 36, pat_w, 4, 25);

                    if p.wants_tea && !p.tea_served {
                        fb.draw_text("C:TEA", bx + 4, by + 42, 34, Some(1), 1);
                    } else {
                        fb.draw_text(&format!("${}", p.ordered_dish.price), bx + 6, by + 42, 15, Some(1), 1);
                    }
                }
                PatronStage::Eating(_) => {
                    fb.rect_bevel(p.x - 18, 86, 38, 18, 2, 15, 1, 1);
                    fb.draw_text("YUM!!", p.x - 12, 91, 15, Some(1), 1);
                }
                _ => {}
            }
        }

        // 5. Render Chef Cosmo Walking Along the Tavern Bar Floor (`y = 152`)
        let cosmo_spr = if self.chef_moving {
            match (self.walk_anim_tick / 2) % 4 {
                0 => &assets.cosmo_run1,
                1 => &assets.cosmo_run2,
                2 => &assets.cosmo_run3,
                _ => &assets.cosmo_run2,
            }
        } else {
            &assets.cosmo_idle
        };
        fb.blit_sprite(
            cosmo_spr,
            self.chef_bar_x - 20,
            152,
            !self.chef_facing_right,
            false,
            None,
        );

        // If Cosmo is carrying a Plated Cloche Dish, draw it right above his hands!
        if let Some(ref dish) = self.carried_dish {
            if let Some(cloche) = assets.ai_sprites_34.get(7) {
                fb.blit_sprite(cloche, self.chef_bar_x - 17, 122, false, false, None);
            }
            fb.draw_text_centered(
                &dish.name[..dish.name.len().min(18)],
                self.chef_bar_x,
                200,
                15,
                Some(1),
                1,
            );
        }

        for ft in &self.popups {
            fb.draw_text(&ft.text, ft.x, ft.y, ft.col, Some(1), 1);
        }

        self.render_apogee_status_bar(
            fb,
            assets,
            track_title,
            "LEFT/RIGHT:WALK  [X]:GRAB/SERVE  [C]:TEA  [E]:SHOP",
        );
    }

    /// Renders Phase 3: The Restaurant & Galley Upgrade Shop
    fn render_upgrade_shop(&self, fb: &mut Framebuffer, assets: &AssetBank, track_title: &str) {
        fb.blit_parallax_bg(&assets.bg_galley, 0, 0, PLAY_H as usize);
        fb.rect_dither(0, 0, FB_W as i32, PLAY_H, 1, 2);

        fb.rect_bevel(12, 6, 376, 30, 2, 4, 1, 1);
        fb.draw_text(
            &format!(
                "NAIBIHPMA TAVERN UPGRADE SHOP   |   YOUR GOLD: ${}",
                self.gold
            ),
            20,
            11,
            15,
            Some(1),
            1,
        );
        fb.draw_text(
            "UPGRADES BOOST HOW FAST & PROFITABLY YOUR TAVERN RUNS!",
            20,
            22,
            34,
            Some(1),
            1,
        );

        let all_upgrades = RestaurantUpgrade::all();
        for (i, &upg) in all_upgrades.iter().enumerate() {
            let by = 40 + (i as i32) * 46;
            let selected = self.shop_cursor == i;
            let owned = self.has_upgrade(upg);
            let bg = if selected { 17 } else { 2 };
            let hi = if selected { 15 } else { 4 };
            fb.rect_bevel(12, by, 376, 42, bg, hi, 1, 1);

            if let Some(spr) = assets.ai_sprites_34.get(4 + (i % 4)) {
                fb.blit_sprite(spr, 18, by + 4, false, false, None);
            }

            let status = if owned {
                "[OWNED]".to_string()
            } else {
                format!("COST: ${}", upg.cost())
            };
            let status_col = if owned {
                25
            } else if self.gold >= upg.cost() {
                15
            } else {
                6
            };

            fb.draw_text(upg.title(), 58, by + 7, 7, Some(1), 1);
            fb.draw_text(&status, 304, by + 7, status_col, Some(1), 1);
            fb.draw_text(upg.desc(), 58, by + 22, 34, Some(1), 1);
        }

        self.render_apogee_status_bar(
            fb,
            assets,
            track_title,
            "UP/DOWN:SELECT  [X/SPACE]:BUY UPGRADE  [E/ENTER]:NEXT DAY STAGE",
        );
    }

    fn render_help_overlay(&self, fb: &mut Framebuffer) {
        fb.rect_dither(0, 0, FB_W as i32, FB_H as i32, 1, 2);
        fb.rect_bevel(20, 20, 360, 216, 2, 4, 1, 1);
        fb.draw_text_centered("CONTROLS & GAMEPLAY GUIDE", 200, 28, 15, Some(1), 2);

        let lines = [
            ("ARROWS / WASD :", "STALK ON LAND & 8-WAY SWIM IN STAR-POND"),
            ("HOLD UP + [X] :", "WINCH OVERHEAD GRAPPLE RING (LV 2)"),
            ("X OR J :", "FIRE HARPOON / PULL LEVER / SERVE DISH"),
            ("Z OR SPACE :", "FIRE SOPORIFIC TRANQ DART (LV 2)"),
            ("C OR K :", "DROP BAIT-NET TRAP (3*) / POUR SPORE-TEA"),
            ("SHADOW ALCOVE :", "STAND IN DARK ARCHWAY TO HIDE FROM PREY"),
            ("KEYS 1 TO 6 :", "1:LV1  2:LV2  3:GATE  4:PREP  5:BAR  6:SHOP"),
            ("[ AND ] / M :", "[ ] SKIP ALBUM TRACK   M: MUTE SFX"),
        ];
        for (i, (k, v)) in lines.iter().enumerate() {
            let y = 56 + (i as i32) * 18;
            fb.draw_text(k, 32, y, 34, Some(1), 1);
            fb.draw_text(v, 130, y, 7, Some(1), 1);
        }
        fb.draw_text_centered("PRESS [H] OR [F1] TO RESUME", 200, 214, 15, Some(1), 1);
    }

    fn render_apogee_status_bar(
        &self,
        fb: &mut Framebuffer,
        assets: &AssetBank,
        track_title: &str,
        controls_hint: &str,
    ) {
        fb.rect_bevel(0, PLAY_H, FB_W as i32, FB_H as i32 - PLAY_H, 2, 4, 1, 1);

        fb.rect_fill(4, PLAY_H + 3, FB_W as i32 - 8, 11, 1);
        fb.draw_text_centered(controls_hint, 200, PLAY_H + 5, 15, None, 1);

        let row_y = PLAY_H + 16;

        fb.rect_bevel(6, row_y, 48, 44, 16, 4, 1, 1);
        if let Some(cosmo_port) = assets.ai_sprites_44.get(4) {
            fb.blit_sprite(cosmo_port, 8, row_y, false, false, None);
        }
        let mood_str = match self.portrait_mood {
            1 => "WINCH!",
            2 => "-O2!!",
            3 => "3*BAG!",
            _ => {
                if self.is_hidden_in_shadow() {
                    "HIDDEN"
                } else {
                    "SUIT OK"
                }
            }
        };
        fb.draw_text_centered(mood_str, 30, row_y + 34, 15, Some(1), 1);

        fb.rect_bevel(58, row_y, 134, 44, 16, 4, 1, 1);
        let o2_col = if self.suit_o2 <= 25 { 38 } else { 34 };
        fb.draw_text(
            &format!("O2:{}%", self.suit_o2),
            64,
            row_y + 5,
            o2_col,
            Some(1),
            1,
        );
        fb.draw_text(
            &format!("TRANQ:{}", self.tranq_darts_left),
            124,
            row_y + 5,
            25,
            Some(1),
            1,
        );

        let state_label = match self.mode {
            GameMode::MenuPrep => "PHASE: MENU PREP",
            GameMode::GalleyService => "PHASE: TAVERN SERVICE",
            GameMode::UpgradeShop => "PHASE: UPGRADE SHOP",
            _ => match self.air_mode {
                AirborneMode::Grounded => {
                    if self.stage_num <= 1 {
                        "LV1: THE UNDERBRUSH"
                    } else {
                        "LV2: DAOT ENCHANTER"
                    }
                }
                AirborneMode::GrappleWinch { .. } => "MODE: GRAPPLE WINCH",
                AirborneMode::Swimming => "MODE: POND DIVER",
                _ => "MODE: DESCENDING",
            },
        };
        fb.draw_text(state_label, 64, row_y + 18, 15, Some(1), 1);
        fb.draw_text(
            &format!("BGM:{}", &track_title[..track_title.len().min(16)]),
            64,
            row_y + 31,
            34,
            Some(1),
            1,
        );

        fb.rect_bevel(196, row_y, 198, 44, 16, 4, 1, 1);
        fb.draw_text(
            &format!("GOLD:${:03}", self.gold),
            202,
            row_y + 6,
            15,
            Some(1),
            1,
        );
        fb.draw_text(
            &format!("KG:{}/{}", self.cargo_weight_kg(), self.max_cargo_kg()),
            202,
            row_y + 19,
            25,
            Some(1),
            1,
        );
        fb.draw_text("1-3:LV  H:HELP", 202, row_y + 32, 6, None, 1);

        for slot in 0..6 {
            let sc = slot % 3;
            let sr = slot / 3;
            let sx = 290 + (sc as i32) * 34;
            let sy = row_y + 3 + (sr as i32) * 20;
            let mut border = 4u8;
            if let Some(item) = self.backpack.get(slot) {
                border = match item.prep {
                    PrepState::CharSeared => 15, // Gold border for 3* PRISTINE!
                    PrepState::Tenderized => 34, // Cyan border for 2* HARPOON
                    PrepState::Fresh => 6,       // Grey border for 1* BRUISED
                };
            }
            fb.rect_bevel(sx, sy, 30, 18, 2, border, 1, 1);
            if let Some(item) = self.backpack.get(slot) {
                if let Some(spr) = assets.ai_sprites_22.get(item.kind as usize) {
                    fb.blit_sprite(spr, sx + 4, sy - 2, false, false, None);
                }
            }
        }
    }
}
