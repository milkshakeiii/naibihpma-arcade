use crate::audio::Sfx;
use crate::battle::{BattlePhase, BattleState};
use crate::data::{
    ArtifactId, Bestiary, CreatureInstance, HabitatKind, SpeciesId, SpellGemId, NUM_SPECIES,
};
use crate::framebuffer::{Framebuffer, FB_H, FB_W};
use crate::sprites::{AssetBank, MON_SZ, TILE_SZ};

pub const PLAY_H: i32 = 240;
pub const MAP_W: usize = 56;
pub const MAP_H: usize = 12;
pub const TS: i32 = TILE_SZ as i32; // 20x20 1:1 pixel tile grid (12 * 20 = 240 playfield height)

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
    pub confirm_pressed: bool, // [ENTER] / [SPACE] / [Z] / [E] / [J]
    pub cancel_pressed: bool,  // [X] / [BACKSPACE] / [K]
    pub special_pressed: bool, // [T] / [C] -> Turbo Auto in Battle | Instant Hunt Lure in Habitat!
    pub squad_pressed: bool,   // [TAB] / [P] -> Toggle 3x2 Squad, Reserve & Pre-Loaded Gear Menu
    pub help_pressed: bool,    // [H] / [F1] -> Toggle Xenobiology Bestiary & Field Guide
    pub retry_pressed: bool,   // [R] -> Start Fresh Run Draft (from Victory/Defeat)
    pub esc_pressed: bool,     // [ESC] -> Leave open sub-menu/overlay, or open Quit Confirmation!
    pub quit_yes_pressed: bool,// [Y] -> Confirm Exit to Desktop when Quit dialog is open
    pub warp_zone: Option<usize>,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum WorldTile {
    Empty,
    StoneRoad,     // Solid floor: 0% wild encounters (safe causeway!)
    EarthSub,      // Solid subsoil block
    Ladder,        // Climbable vine/wood ladder
    FernPatch,     // Passable overlay above solid floor: Verdant encounters!
    StarWater,     // Passable wading pool above solid floor: Tide encounters!
    CrystalPatch,  // Passable grotto cluster above solid floor: Construct/Arcane encounters!
    AlphaPatch,    // Passable golden grove above solid floor: +2 Lv Alpha Elite encounters!
}

impl WorldTile {
    pub fn is_solid(self) -> bool {
        matches!(self, WorldTile::StoneRoad | WorldTile::EarthSub)
    }

    pub fn habitat(self) -> HabitatKind {
        match self {
            WorldTile::FernPatch => HabitatKind::FernUnderbrush,
            WorldTile::StarWater => HabitatKind::StarPond,
            WorldTile::CrystalPatch => HabitatKind::CrystalGrotto,
            WorldTile::AlphaPatch => HabitatKind::AlphaGrove,
            _ => HabitatKind::SafeStone,
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum GearReward {
    Gem(SpellGemId),
    Artifact(ArtifactId),
}

#[derive(Clone)]
pub struct SupplyCache {
    pub col: usize,
    pub row: usize,
    pub opened: bool,
    pub reward: GearReward,
    pub bonus_pod: bool,
}

#[derive(Clone)]
pub struct MedBeacon {
    pub col: usize,
    pub row: usize,
    pub used: bool,
}

#[derive(Clone)]
pub struct ZoneGuardian {
    pub col: usize,
    pub row: usize,
    pub lead_species: SpeciesId,
    pub title: &'static str,
    pub defeated: bool,
}

#[derive(Clone)]
pub struct FloatingToast {
    pub x: i32,
    pub y: i32,
    pub text: String,
    pub col: u8,
    pub ticks: i32,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum DraftStep {
    StarterOne,
    StarterTwo,
    StarterArtifact,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum SquadSubMode {
    BrowseSlots,
    SwapWith { first_slot: usize },
    PickGearSlot { slot: usize, choosing_artifact: bool },
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum GameMode {
    StarterDraft {
        step: DraftStep,
        cursor: usize,
    },
    SideWorld,
    Battle,
    CaptureOverflow {
        cursor: usize,
    },
    LootDraft {
        cursor: usize,
    },
    SquadMenu {
        cursor: usize,
        sub: SquadSubMode,
        bag_cursor: usize,
    },
    RunVictory,
    RunDefeat,
}

pub struct GameState {
    pub crt_scanlines: bool,
    pub tick_count: u64,
    pub rng_state: u32,

    pub bestiary: Bestiary,
    pub show_bestiary_overlay: bool,
    pub quit_confirm_open: bool,
    pub should_exit_app: bool,
    pub bestiary_tab: usize, // 0 = Guide, 1 = 14-Species Codex
    pub bestiary_cursor: usize,

    pub mode: GameMode,
    pub draft_species_offers: [SpeciesId; 3],
    pub draft_artifact_offers: [ArtifactId; 3],
    pub loot_offers: [GearReward; 3],
    pub pending_captures: Vec<CreatureInstance>,

    // Run Roster: 6 Active (3x2: 0..3 Front Col, 3..6 Back Col) + 2 Field Reserve + Gear Bags!
    pub squad: [Option<CreatureInstance>; 6],
    pub reserve: [Option<CreatureInstance>; 2],
    pub gem_bag: Vec<SpellGemId>,
    pub artifact_bag: Vec<ArtifactId>,
    pub pods_left: usize,
    pub battles_won: u32,

    // Side-View Pokemon World State
    pub zone_num: usize, // 1 = Underbrush, 2 = Star-Pond & Daot, 3 = Ancient Sanctum & Grof
    pub tiles: [[WorldTile; MAP_W]; MAP_H],
    pub caches: Vec<SupplyCache>,
    pub beacon: Option<MedBeacon>,
    pub guardian: Option<ZoneGuardian>,
    pub exit_col: usize,
    pub exit_row: usize,

    // Integer-Only Blue Alien Coordinates (20x20 sprite, boots on py + 19!)
    pub px: i32,
    pub py: i32,
    pub face_right: bool,
    pub climbing: bool,
    pub walk_frame: usize,
    pub cam_x: i32,
    pub encounter_meter: i32, // 0..100 step tension gauge in tall grass / water / grotto!
    pub encounter_grace_ticks: i32,
    pub world_banner: String,
    pub toasts: Vec<FloatingToast>,

    pub active_battle: Option<BattleState>,
    pub desired_bgm_track: Option<usize>,
}

impl GameState {
    pub fn new() -> Self {
        let seed = 0x51A7_1E29u32;
        let bestiary = Bestiary::load_or_default();
        let mut gs = Self {
            crt_scanlines: true,
            tick_count: 0,
            rng_state: seed,
            bestiary,
            show_bestiary_overlay: false,
            quit_confirm_open: false,
            should_exit_app: false,
            bestiary_tab: 0,
            bestiary_cursor: 0,
            mode: GameMode::StarterDraft {
                step: DraftStep::StarterOne,
                cursor: 0,
            },
            draft_species_offers: [
                SpeciesId::DewNewt,
                SpeciesId::VoltaicEel,
                SpeciesId::BriarToad,
            ],
            draft_artifact_offers: [
                ArtifactId::DewdropVial,
                ArtifactId::CopperDynamo,
                ArtifactId::TrapperLens,
            ],
            loot_offers: [
                GearReward::Gem(SpellGemId::PrecisionDart),
                GearReward::Artifact(ArtifactId::SporeCrown),
                GearReward::Gem(SpellGemId::StarlightChoir),
            ],
            pending_captures: Vec::new(),
            squad: [None, None, None, None, None, None],
            reserve: [None, None],
            gem_bag: Vec::new(),
            artifact_bag: Vec::new(),
            pods_left: 4,
            battles_won: 0,
            zone_num: 1,
            tiles: [[WorldTile::Empty; MAP_W]; MAP_H],
            caches: Vec::new(),
            beacon: None,
            guardian: None,
            exit_col: 53,
            exit_row: 7,
            px: 40,
            py: 120,
            face_right: true,
            climbing: false,
            walk_frame: 0,
            cam_x: 0,
            encounter_meter: 0,
            encounter_grace_ticks: 0,
            world_banner: String::new(),
            toasts: Vec::new(),
            active_battle: None,
            desired_bgm_track: Some(0),
        };
        gs.begin_new_run_draft();
        gs
    }

    fn next_rand(&mut self) -> u32 {
        let mut x = self.rng_state.wrapping_add((self.tick_count as u32).wrapping_mul(7919));
        if x == 0 {
            x = 0x9E37_79B9;
        }
        x ^= x << 13;
        x ^= x >> 17;
        x ^= x << 5;
        self.rng_state = x.max(1);
        self.rng_state
    }

    pub fn begin_new_run_draft(&mut self) {
        self.squad = [None, None, None, None, None, None];
        self.reserve = [None, None];
        self.gem_bag = vec![SpellGemId::PrecisionDart]; // Start with 1 Surgical Dart Gem in bag for capture builds!
        self.artifact_bag.clear();
        self.pending_captures.clear();
        self.active_battle = None;
        self.pods_left = 4;
        self.battles_won = 0;
        self.zone_num = 1;
        self.roll_species_draft_offers(None);
        self.roll_artifact_draft_offers();
        self.mode = GameMode::StarterDraft {
            step: DraftStep::StarterOne,
            cursor: 0,
        };
        self.desired_bgm_track = Some(0);
    }

    fn roll_species_draft_offers(&mut self, exclude: Option<SpeciesId>) {
        let mut pool = self.bestiary.unlocked_recruitable();
        if let Some(ex) = exclude {
            pool.retain(|&s| s != ex);
        }
        if pool.len() < 3 {
            pool = SpeciesId::RECRUITABLE.to_vec();
        }
        // Fisher-Yates shuffle 3 distinct offers
        for i in 0..3 {
            let rem = pool.len() - i;
            let pick = i + (self.next_rand() as usize % rem);
            pool.swap(i, pick);
            self.draft_species_offers[i] = pool[i];
        }
    }

    fn roll_artifact_draft_offers(&mut self) {
        let mut pool = ArtifactId::ALL.to_vec();
        for i in 0..3 {
            let rem = pool.len() - i;
            let pick = i + (self.next_rand() as usize % rem);
            pool.swap(i, pick);
            self.draft_artifact_offers[i] = pool[i];
        }
    }

    fn roll_post_battle_loot_offers(&mut self) {
        let g1 = SpellGemId::ALL[(self.next_rand() as usize) % SpellGemId::ALL.len()];
        let mut g2 = SpellGemId::ALL[(self.next_rand() as usize) % SpellGemId::ALL.len()];
        if g2 == g1 {
            g2 = SpellGemId::ALL[(g1 as usize + 3) % SpellGemId::ALL.len()];
        }
        let a1 = ArtifactId::ALL[(self.next_rand() as usize) % ArtifactId::ALL.len()];
        self.loot_offers = [
            GearReward::Gem(g1),
            GearReward::Artifact(a1),
            GearReward::Gem(g2),
        ];
    }

    pub fn zone_wild_level(&self) -> i32 {
        match self.zone_num {
            1 => 3,
            2 => 6,
            _ => 9,
        }
    }

    pub fn zone_name(&self) -> &'static str {
        match self.zone_num {
            1 => "ZONE 1: THE UNDERBRUSH & TWEN GLADE",
            2 => "ZONE 2: STAR-POND & MYCELIUM GROTTO",
            _ => "ZONE 3: ANCIENT MAGIC SANCTUM",
        }
    }

    // =========================================================================
    // SIDE-VIEW POKEMON HABITAT MAP BUILDER (ZONES 1, 2, AND 3)
    // =========================================================================
    pub fn load_zone(&mut self, zone: usize) {
        self.zone_num = zone.clamp(1, 3);
        self.tiles = [[WorldTile::Empty; MAP_W]; MAP_H];
        self.caches.clear();
        self.toasts.clear();
        self.encounter_meter = 0;
        self.encounter_grace_ticks = 18;
        self.climbing = false;

        let ascii_rows: [&str; MAP_H] = match self.zone_num {
            1 => [
                "........................................................",
                "....AAAA..FFFF........FFFFFFFF..........................",
                "...######H#####......#########H.........................",
                ".........H....................H.........................",
                ".........H....................H.........CCCC...AAAA.....",
                ".........H....................H........#####H#######....",
                ".........H..FFFFF.............H...FFF.......H...........",
                ".########H########..WWWWWWWW..###H##########H##########.",
                ".========H========############===H==========H##########.",
                ".========H========.==========.===H==========H==========.",
                ".....CCCCH..CCCCC................CCCCCCC....H...........",
                "#########H##################################H###########",
            ],
            2 => [
                "........................................................",
                "...................AAAA.......FFFFFF....................",
                "..................######H....########H..................",
                "........................H............H..................",
                "...FFFFF................H............H..........AAAA....",
                "..#######H..............H............H.........######...",
                ".........H..WWWWWWW.....H..WWWWWWW...H..FFFF...H........",
                ".########H.#########..##H##########..H########.H#######.",
                ".========H.=========#===H==========..H========.H========",
                ".========H..........====H............H.........H========",
                ".....CCCCH..CCCCCC......H...CCCCCC...H..CCCCC..H........",
                "#########H##############H############H#########H########",
            ],
            _ => [
                "........................................................",
                "...AAAA...........FFFFFF............AAAA................",
                "..######H........########H.........######H..............",
                "........H................H...............H..............",
                "........H................H...............H..............",
                "........H...WWWWWW.......H...CCCCCC......H..............",
                "........H..########..####H#########..FFFFH..............",
                ".#######H..========.=====H=========######H############..",
                ".=======H................H.........======H============..",
                ".=======H................H...............H..............",
                "....CCCCH...CCCCCC.......H..AAAA....CCCCCH..............",
                "########H################H###############H##############",
            ],
        };

        for (r, line) in ascii_rows.iter().enumerate() {
            for (c, ch) in line.chars().take(MAP_W).enumerate() {
                self.tiles[r][c] = match ch {
                    '#' => WorldTile::StoneRoad,
                    '=' => WorldTile::EarthSub,
                    'H' => WorldTile::Ladder,
                    'F' => WorldTile::FernPatch,
                    'W' => WorldTile::StarWater,
                    'C' => WorldTile::CrystalPatch,
                    'A' => WorldTile::AlphaPatch,
                    _ => WorldTile::Empty,
                };
            }
        }

        // Place Player cleanly on West Stone Causeway (boots grounded on StoneRoad!)
        self.px = 2 * TS;
        self.py = 6 * TS;
        self.snap_player_to_ground();
        self.face_right = true;
        self.cam_x = 0;

        // Populate Zone Landmarks (Supply Caches, Med-Beacon, and East Guardian Boss!)
        match self.zone_num {
            1 => {
                self.caches.push(SupplyCache {
                    col: 15,
                    row: 10,
                    opened: false,
                    reward: GearReward::Artifact(ArtifactId::SporeCrown),
                    bonus_pod: true,
                });
                self.caches.push(SupplyCache {
                    col: 28,
                    row: 1,
                    opened: false,
                    reward: GearReward::Gem(SpellGemId::ChainArc),
                    bonus_pod: false,
                });
                self.beacon = Some(MedBeacon {
                    col: 43,
                    row: 6,
                    used: false,
                });
                self.guardian = Some(ZoneGuardian {
                    col: 47,
                    row: 6,
                    lead_species: SpeciesId::BriarToad,
                    title: "ALPHA BRIAR-WARDEN",
                    defeated: false,
                });
                self.exit_col = 53;
                self.exit_row = 6;
                self.desired_bgm_track = Some(0);
            }
            2 => {
                self.caches.push(SupplyCache {
                    col: 15,
                    row: 10,
                    opened: false,
                    reward: GearReward::Artifact(ArtifactId::LanternPrism),
                    bonus_pod: true,
                });
                self.caches.push(SupplyCache {
                    col: 33,
                    row: 1,
                    opened: false,
                    reward: GearReward::Gem(SpellGemId::StarlightChoir),
                    bonus_pod: false,
                });
                self.beacon = Some(MedBeacon {
                    col: 49,
                    row: 6,
                    used: false,
                });
                self.guardian = Some(ZoneGuardian {
                    col: 51,
                    row: 6,
                    lead_species: SpeciesId::BossDaot,
                    title: "DAOT THE ENCHANTER",
                    defeated: false,
                });
                self.exit_col = 54;
                self.exit_row = 6;
                self.desired_bgm_track = Some(1);
            }
            _ => {
                self.caches.push(SupplyCache {
                    col: 4,
                    row: 1,
                    opened: false,
                    reward: GearReward::Artifact(ArtifactId::AncientSigil),
                    bonus_pod: true,
                });
                self.caches.push(SupplyCache {
                    col: 32,
                    row: 10,
                    opened: false,
                    reward: GearReward::Gem(SpellGemId::AncientNova),
                    bonus_pod: false,
                });
                self.beacon = Some(MedBeacon {
                    col: 46,
                    row: 6,
                    used: false,
                });
                self.guardian = Some(ZoneGuardian {
                    col: 50,
                    row: 6,
                    lead_species: SpeciesId::BossGrof,
                    title: "GROF, MALEVOLENT SORCERER",
                    defeated: false,
                });
                self.exit_col = 53;
                self.exit_row = 6;
                self.desired_bgm_track = Some(4);
            }
        }

        self.world_banner = self.zone_name().to_string();
    }

    fn tile_at(&self, col: i32, row: i32) -> WorldTile {
        if col < 0 || col >= MAP_W as i32 || row < 0 || row >= MAP_H as i32 {
            WorldTile::StoneRoad
        } else {
            self.tiles[row as usize][col as usize]
        }
    }

    fn is_solid_at(&self, col: i32, row: i32) -> bool {
        if col < 0 || col >= MAP_W as i32 || row >= MAP_H as i32 {
            true
        } else if row < 0 {
            false
        } else {
            self.tiles[row as usize][col as usize].is_solid()
        }
    }

    fn snap_player_to_ground(&mut self) {
        let cx = (self.px + 10) / TS;
        let mut r = (self.py / TS).clamp(0, MAP_H as i32 - 1);
        while r > 0 && self.is_solid_at(cx, r) {
            r -= 1;
        }
        while r + 1 < MAP_H as i32 && !self.is_solid_at(cx, r + 1) {
            r += 1;
        }
        self.py = (r + 1) * TS - TS;
    }

    pub fn current_player_habitat(&self) -> HabitatKind {
        let col = ((self.px + 10) / TS).clamp(0, MAP_W as i32 - 1);
        let row = ((self.py + 14) / TS).clamp(0, MAP_H as i32 - 1);
        self.tile_at(col, row).habitat()
    }

    fn add_toast(&mut self, x: i32, y: i32, text: String, col: u8) {
        self.toasts.push(FloatingToast {
            x,
            y,
            text,
            col,
            ticks: 22,
        });
    }

    // =========================================================================
    // MAIN 15 FPS DISCRETE SIMULATION UPDATE
    // =========================================================================
    pub fn update(&mut self, input: &InputState, sfx_out: &mut Vec<Sfx>) {
        self.tick_count = self.tick_count.wrapping_add(1);

        for t in &mut self.toasts {
            t.ticks -= 1;
            if t.ticks % 2 == 0 {
                t.y -= 1;
            }
        }
        self.toasts.retain(|t| t.ticks > 0);

        // Protective Quit Confirmation Modal (prevents accidental Esc run loss!)
        if self.quit_confirm_open {
            if input.quit_yes_pressed || input.confirm_pressed {
                self.should_exit_app = true;
            } else if input.esc_pressed || input.cancel_pressed {
                self.quit_confirm_open = false;
                sfx_out.push(Sfx::SuctionStick);
            }
            return;
        }

        // Smart [ESC] Navigation: backs out of open overlays, Army Management, or target selection first!
        if input.esc_pressed {
            if self.show_bestiary_overlay {
                self.show_bestiary_overlay = false;
                sfx_out.push(Sfx::SuctionStick);
                return;
            }
            match self.mode {
                GameMode::SquadMenu { cursor, sub, .. } => {
                    match sub {
                        SquadSubMode::BrowseSlots => {
                            self.mode = GameMode::SideWorld;
                        }
                        SquadSubMode::SwapWith { .. } | SquadSubMode::PickGearSlot { .. } => {
                            self.mode = GameMode::SquadMenu {
                                cursor,
                                sub: SquadSubMode::BrowseSlots,
                                bag_cursor: 0,
                            };
                        }
                    }
                    sfx_out.push(Sfx::SuctionStick);
                    return;
                }
                GameMode::Battle => {
                    if let Some(ref mut bs) = self.active_battle {
                        if let BattlePhase::WaitingEnemyTarget { ally_slot, .. } = bs.phase {
                            bs.phase = BattlePhase::WaitingAllyCommand { slot: ally_slot };
                            sfx_out.push(Sfx::SuctionStick);
                            return;
                        }
                    }
                    self.quit_confirm_open = true;
                    sfx_out.push(Sfx::SuctionStick);
                    return;
                }
                _ => {
                    self.quit_confirm_open = true;
                    sfx_out.push(Sfx::SuctionStick);
                    return;
                }
            }
        }

        if input.help_pressed {
            self.show_bestiary_overlay = !self.show_bestiary_overlay;
            sfx_out.push(Sfx::SuctionStick);
            return;
        }

        if self.show_bestiary_overlay {
            if input.left_pressed || input.right_pressed {
                self.bestiary_tab = 1 - self.bestiary_tab;
                sfx_out.push(Sfx::SuctionStick);
            } else if self.bestiary_tab == 1 {
                if input.up_pressed {
                    self.bestiary_cursor = (self.bestiary_cursor + NUM_SPECIES - 1) % NUM_SPECIES;
                    sfx_out.push(Sfx::SuctionStick);
                } else if input.down_pressed {
                    self.bestiary_cursor = (self.bestiary_cursor + 1) % NUM_SPECIES;
                    sfx_out.push(Sfx::SuctionStick);
                }
            }
            if input.cancel_pressed {
                self.show_bestiary_overlay = false;
                sfx_out.push(Sfx::SuctionStick);
            }
            return;
        }

        if let Some(wz) = input.warp_zone {
            if self.squad.iter().all(|s| s.is_none()) {
                // Ensure a test squad exists if warping directly from draft!
                self.squad[0] = Some(CreatureInstance::new(SpeciesId::DewNewt, 3 * wz as i32, false));
                self.squad[1] = Some(CreatureInstance::new(SpeciesId::SporeMorel, 3 * wz as i32, false));
                self.squad[3] = Some(CreatureInstance::new(SpeciesId::VoltaicEel, 3 * wz as i32, false));
            }
            self.load_zone(wz);
            self.mode = GameMode::SideWorld;
            sfx_out.push(Sfx::CoinStart);
            return;
        }

        match self.mode {
            GameMode::StarterDraft { step, cursor } => {
                self.update_starter_draft(step, cursor, input, sfx_out);
            }
            GameMode::SideWorld => {
                if input.squad_pressed {
                    self.mode = GameMode::SquadMenu {
                        cursor: 0,
                        sub: SquadSubMode::BrowseSlots,
                        bag_cursor: 0,
                    };
                    sfx_out.push(Sfx::SuctionStick);
                } else {
                    self.update_side_world(input, sfx_out);
                }
            }
            GameMode::SquadMenu {
                cursor,
                sub,
                bag_cursor,
            } => {
                self.update_squad_menu(cursor, sub, bag_cursor, input, sfx_out);
            }
            GameMode::Battle => {
                self.update_battle_mode(input, sfx_out);
            }
            GameMode::CaptureOverflow { cursor } => {
                self.update_capture_overflow(cursor, input, sfx_out);
            }
            GameMode::LootDraft { cursor } => {
                self.update_loot_draft(cursor, input, sfx_out);
            }
            GameMode::RunVictory | GameMode::RunDefeat => {
                if input.confirm_pressed || input.retry_pressed {
                    self.begin_new_run_draft();
                    sfx_out.push(Sfx::CoinStart);
                }
            }
        }
    }

    fn update_starter_draft(
        &mut self,
        step: DraftStep,
        mut cursor: usize,
        input: &InputState,
        sfx_out: &mut Vec<Sfx>,
    ) {
        if input.left_pressed {
            cursor = (cursor + 2) % 3;
            sfx_out.push(Sfx::SuctionStick);
        } else if input.right_pressed {
            cursor = (cursor + 1) % 3;
            sfx_out.push(Sfx::SuctionStick);
        }
        self.mode = GameMode::StarterDraft { step, cursor };

        if input.confirm_pressed {
            match step {
                DraftStep::StarterOne => {
                    let sp1 = self.draft_species_offers[cursor];
                    self.squad[0] = Some(CreatureInstance::new(sp1, 3, false));
                    self.roll_species_draft_offers(Some(sp1));
                    self.mode = GameMode::StarterDraft {
                        step: DraftStep::StarterTwo,
                        cursor: 0,
                    };
                    sfx_out.push(Sfx::HarvestPluck);
                }
                DraftStep::StarterTwo => {
                    let sp2 = self.draft_species_offers[cursor];
                    // Place tankier creatures in Front Row (slot 1), ranged casters in Back Row (slot 3)!
                    let meta = sp2.meta();
                    let slot = if meta.base_def >= meta.base_int { 1 } else { 3 };
                    self.squad[slot] = Some(CreatureInstance::new(sp2, 3, false));
                    self.mode = GameMode::StarterDraft {
                        step: DraftStep::StarterArtifact,
                        cursor: 0,
                    };
                    sfx_out.push(Sfx::HarvestPluck);
                }
                DraftStep::StarterArtifact => {
                    let art = self.draft_artifact_offers[cursor];
                    if let Some(ref mut c0) = self.squad[0] {
                        c0.artifact = Some(art);
                        c0.heal_full();
                    }
                    self.load_zone(1);
                    self.mode = GameMode::SideWorld;
                    sfx_out.push(Sfx::CoinStart);
                }
            }
        }
    }

    fn update_side_world(&mut self, input: &InputState, sfx_out: &mut Vec<Sfx>) {
        if self.encounter_grace_ticks > 0 {
            self.encounter_grace_ticks -= 1;
        }

        let center_col = ((self.px + 10) / TS).clamp(0, MAP_W as i32 - 1);
        let center_row = ((self.py + 10) / TS).clamp(0, MAP_H as i32 - 1);
        let foot_row = ((self.py + 19) / TS).clamp(0, MAP_H as i32 - 1);

        let on_ladder = self.tile_at(center_col, center_row) == WorldTile::Ladder
            || self.tile_at(center_col, foot_row) == WorldTile::Ladder;
        let ladder_below = self.tile_at(center_col, foot_row + 1) == WorldTile::Ladder;

        // Ladder engagement (`Up` or `Down` cleanly climbs without any tricky alignment!)
        if !self.climbing {
            if input.up && on_ladder {
                self.climbing = true;
                self.px = center_col * TS;
            } else if input.down && (on_ladder || ladder_below) {
                self.climbing = true;
                self.px = center_col * TS;
                if ladder_below && !on_ladder {
                    self.py += 6;
                }
            }
        }

        let mut moved_horiz = false;
        if self.climbing {
            let mut dy = 0;
            if input.up {
                dy -= 4;
            }
            if input.down {
                dy += 4;
            }
            if dy != 0 {
                self.walk_frame = (self.walk_frame + 1) % 8;
                let next_y = (self.py + dy).clamp(0, (MAP_H as i32 - 1) * TS);
                let check_row = if dy < 0 {
                    next_y / TS
                } else {
                    (next_y + 19) / TS
                };
                if dy > 0 && self.is_solid_at(center_col, check_row)
                    && self.tile_at(center_col, check_row) != WorldTile::Ladder
                {
                    self.py = check_row * TS - TS;
                    self.climbing = false;
                } else {
                    self.py = next_y;
                }
            }
            if input.left || input.right {
                let dir = if input.right { 1 } else { -1 };
                let side_col = center_col + dir;
                let r_body = (self.py + 10) / TS;
                if !self.is_solid_at(side_col, r_body) {
                    self.climbing = false;
                    self.px += dir * 4;
                    self.face_right = dir > 0;
                }
            }
            let now_col = (self.px + 10) / TS;
            let now_r1 = (self.py + 6) / TS;
            let now_r2 = (self.py + 19) / TS;
            if self.tile_at(now_col, now_r1) != WorldTile::Ladder
                && self.tile_at(now_col, now_r2) != WorldTile::Ladder
            {
                self.climbing = false;
            }
        } else {
            // Smooth Side-View Walking + Automatic 1-Tile Step-Up (zero jump frustration!)
            let mut dx = 0;
            if input.left {
                dx -= 4;
                self.face_right = false;
            }
            if input.right {
                dx += 4;
                self.face_right = true;
            }

            if dx != 0 {
                let next_x = (self.px + dx).clamp(2, (MAP_W as i32 - 2) * TS);
                let lead_x = if dx > 0 { next_x + 15 } else { next_x + 4 };
                let lead_col = lead_x / TS;
                let body_row = (self.py + 14) / TS;
                let head_row = (self.py + 2) / TS;

                // Stop before undefeated Zone Guardian so you don't walk past the Boss Gate!
                let blocked_by_boss = self
                    .guardian
                    .as_ref()
                    .filter(|g| !g.defeated)
                    .map(|g| dx > 0 && next_x + 16 > (g.col as i32) * TS - 4 && (body_row - g.row as i32).abs() <= 1)
                    .unwrap_or(false);

                if !blocked_by_boss {
                    if !self.is_solid_at(lead_col, body_row) && !self.is_solid_at(lead_col, head_row)
                    {
                        self.px = next_x;
                        moved_horiz = true;
                    } else if self.is_solid_at(lead_col, body_row)
                        && !self.is_solid_at(lead_col, body_row - 1)
                        && (body_row < 2 || !self.is_solid_at(lead_col, body_row - 2))
                    {
                        // Auto-step smoothly up a 1-tile stone ledge!
                        self.px = next_x;
                        self.py = body_row * TS - TS;
                        moved_horiz = true;
                    }
                }
            }

            // Gravity onto solid floor or ladder top
            let support_l = (self.px + 5) / TS;
            let support_r = (self.px + 14) / TS;
            let below_row = (self.py + 20) / TS;
            let supported = self.is_solid_at(support_l, below_row)
                || self.is_solid_at(support_r, below_row)
                || (self.tile_at((self.px + 10) / TS, below_row) == WorldTile::Ladder
                    && (self.py + 20) % TS < 6);

            if !supported {
                self.py += 5;
                let land_row = (self.py + 19) / TS;
                if self.is_solid_at(support_l, land_row) || self.is_solid_at(support_r, land_row) {
                    self.py = land_row * TS - TS;
                }
            }

            if moved_horiz {
                self.walk_frame = (self.walk_frame + 1) % 12;
            } else {
                self.walk_frame = 0;
            }
        }

        // Smooth integer camera tracking
        let max_cam = (MAP_W as i32) * TS - FB_W as i32;
        self.cam_x = (self.px + 10 - (FB_W as i32) / 2).clamp(0, max_cam);

        // Interactive Landmarks ([ENTER] / [SPACE] / [Z] / [E])
        if input.confirm_pressed {
            if self.try_interact_landmark(sfx_out) {
                return;
            }
        }

        // Instant Habitat Hunt Button ([C] / [T]) when standing inside an encounter terrain!
        let hab = self.current_player_habitat();
        if input.special_pressed && hab.is_encounter_zone() {
            self.encounter_meter = 0;
            self.trigger_wild_encounter(hab, sfx_out);
            return;
        }

        // Step-based Pokemon Terrain Encounter Roll!
        if moved_horiz && hab.is_encounter_zone() && self.encounter_grace_ticks == 0 {
            let step_gain = 14 + (self.next_rand() % 9) as i32;
            self.encounter_meter += step_gain;
            if self.encounter_meter >= 100 {
                self.encounter_meter = 0;
                self.trigger_wild_encounter(hab, sfx_out);
            }
        } else if !hab.is_encounter_zone() && self.encounter_meter > 0 {
            self.encounter_meter = (self.encounter_meter - 4).max(0);
        }
    }

    fn try_interact_landmark(&mut self, sfx_out: &mut Vec<Sfx>) -> bool {
        let p_cx = self.px + 10;
        let p_cy = self.py + 10;

        // 1. Supply Caches
        for i in 0..self.caches.len() {
            if !self.caches[i].opened {
                let cx = (self.caches[i].col as i32) * TS + 10;
                let cy = (self.caches[i].row as i32) * TS + 10;
                if (p_cx - cx).abs() <= 24 && (p_cy - cy).abs() <= 24 {
                    self.caches[i].opened = true;
                    let reward = self.caches[i].reward;
                    let bonus_pod = self.caches[i].bonus_pod;
                    let msg = match reward {
                        GearReward::Gem(g) => {
                            self.gem_bag.push(g);
                            format!("FOUND {}!", g.meta().short_name)
                        }
                        GearReward::Artifact(a) => {
                            self.artifact_bag.push(a);
                            // Auto-equip if any active ally has an empty artifact slot!
                            let mut equipped_on = None;
                            for slot in 0..6 {
                                if let Some(ref mut c) = self.squad[slot] {
                                    if c.artifact.is_none() {
                                        c.artifact = Some(a);
                                        if let Some(pos) =
                                            self.artifact_bag.iter().rposition(|&x| x == a)
                                        {
                                            self.artifact_bag.remove(pos);
                                        }
                                        equipped_on = Some(c.species.meta().short_name);
                                        break;
                                    }
                                }
                            }
                            if let Some(who) = equipped_on {
                                format!("{} -> {}!", a.meta().short_name, who)
                            } else {
                                format!("FOUND {}!", a.meta().short_name)
                            }
                        }
                    };
                    if bonus_pod {
                        self.pods_left += 1;
                    }
                    self.add_toast(cx, cy - 12, msg, 15);
                    sfx_out.push(Sfx::ServiceBell);
                    return true;
                }
            }
        }

        // 2. Field Med-Beacon
        if let Some(ref mut b) = self.beacon {
            if !b.used {
                let bx = (b.col as i32) * TS + 10;
                let by = (b.row as i32) * TS + 10;
                if (p_cx - bx).abs() <= 26 && (p_cy - by).abs() <= 26 {
                    b.used = true;
                    for s in 0..6 {
                        if let Some(ref mut c) = self.squad[s] {
                            c.heal_full();
                        }
                    }
                    for r in 0..2 {
                        if let Some(ref mut c) = self.reserve[r] {
                            c.heal_full();
                        }
                    }
                    self.pods_left += 2;
                    self.add_toast(bx, by - 14, "SQUAD HEALED & +2 PODS!".to_string(), 25);
                    sfx_out.push(Sfx::CoinStart);
                    return true;
                }
            }
        }

        // 3. Zone Guardian Boss Challenge!
        if let Some(ref g) = self.guardian {
            if !g.defeated {
                let gx = (g.col as i32) * TS + 10;
                let gy = (g.row as i32) * TS + 10;
                if (p_cx - gx).abs() <= 34 && (p_cy - gy).abs() <= 28 {
                    self.trigger_guardian_battle(sfx_out);
                    return true;
                }
            }
        }

        // 4. Unlocked East Zone Exit Archway!
        let boss_clear = self.guardian.as_ref().map(|g| g.defeated).unwrap_or(true);
        if boss_clear {
            let ex = (self.exit_col as i32) * TS + 10;
            let ey = (self.exit_row as i32) * TS + 10;
            if (p_cx - ex).abs() <= 28 && (p_cy - ey).abs() <= 28 {
                if self.zone_num < 3 {
                    let next_z = self.zone_num + 1;
                    self.load_zone(next_z);
                    sfx_out.push(Sfx::CoinStart);
                } else {
                    self.bestiary.runs_cleared = self.bestiary.runs_cleared.saturating_add(1);
                    self.bestiary.save();
                    self.mode = GameMode::RunVictory;
                    sfx_out.push(Sfx::CoinStart);
                }
                return true;
            }
        }

        false
    }

    pub fn trigger_wild_encounter(&mut self, hab: HabitatKind, sfx_out: &mut Vec<Sfx>) {
        let pool = hab.encounter_pool(self.zone_num);
        if pool.is_empty() {
            return;
        }
        let base_lv = self.zone_wild_level();
        let is_alpha_zone = hab == HabitatKind::AlphaGrove;
        let pack_count = match self.zone_num {
            1 => 2 + (self.next_rand() as usize % 2), // 2..=3
            2 => 3 + (self.next_rand() as usize % 2), // 3..=4
            _ => 4 + (self.next_rand() as usize % 2), // 4..=5
        };

        let mut enemies: [Option<CreatureInstance>; 6] = [None, None, None, None, None, None];
        let slot_order = [0usize, 1, 3, 2, 4, 5];

        for i in 0..pack_count.min(6) {
            let sp = pool[(self.next_rand() as usize) % pool.len()];
            let is_lead_alpha = (i == 0 && is_alpha_zone)
                || (!is_alpha_zone && i == 0 && (self.next_rand() % 100) < 18);
            let lv = if is_lead_alpha {
                base_lv + 2
            } else {
                base_lv + ((self.next_rand() % 2) as i32)
            };
            let mut inst = CreatureInstance::new(sp, lv, is_lead_alpha);
            if is_lead_alpha {
                let art = ArtifactId::ALL[(self.next_rand() as usize) % ArtifactId::ALL.len()];
                inst.artifact = Some(art);
                inst.heal_full();
            }
            enemies[slot_order[i]] = Some(inst);
        }

        let title = if is_alpha_zone {
            "ALPHA GROVE PACK".to_string()
        } else {
            match hab {
                HabitatKind::FernUnderbrush => "WILD UNDERBRUSH PACK".to_string(),
                HabitatKind::StarPond => "WILD STAR-POND PACK".to_string(),
                HabitatKind::CrystalGrotto => "WILD GROTTO PACK".to_string(),
                _ => "WILD SPECIMEN PACK".to_string(),
            }
        };

        let seed = self.next_rand();
        self.active_battle = Some(BattleState::new(
            &self.squad,
            enemies,
            false,
            self.zone_num,
            title,
            seed,
        ));
        self.mode = GameMode::Battle;
        sfx_out.push(Sfx::ToadBounce);
    }

    pub fn trigger_guardian_battle(&mut self, sfx_out: &mut Vec<Sfx>) {
        let mut enemies: [Option<CreatureInstance>; 6] = [None, None, None, None, None, None];
        let (title, is_true_boss) = match self.zone_num {
            1 => {
                // Zone 1 Guardian: Capturable Alpha Briar-Toad + Synergy Pack!
                let mut lead = CreatureInstance::new(SpeciesId::BriarToad, 5, true);
                lead.artifact = Some(ArtifactId::IronbarkBuckler);
                lead.heal_full();
                enemies[1] = Some(lead);
                enemies[3] = Some(CreatureInstance::new(SpeciesId::SporeMorel, 4, false));
                enemies[4] = Some(CreatureInstance::new(SpeciesId::DewNewt, 4, false));
                ("ALPHA BRIAR-WARDEN", false)
            }
            2 => {
                self.bestiary.record_sighting_boss(SpeciesId::BossDaot);
                let mut boss = CreatureInstance::new(SpeciesId::BossDaot, 8, true);
                boss.artifact = Some(ArtifactId::SporeCrown);
                boss.heal_full();
                enemies[4] = Some(boss);
                enemies[0] = Some(CreatureInstance::new(SpeciesId::BogGolem, 7, false));
                enemies[1] = Some(CreatureInstance::new(SpeciesId::LanternCrab, 7, false));
                enemies[3] = Some(CreatureInstance::new(SpeciesId::DewFrog, 7, false));
                enemies[5] = Some(CreatureInstance::new(SpeciesId::GloomMoth, 7, false));
                self.desired_bgm_track = Some(1);
                ("DAOT THE ENCHANTER", true)
            }
            _ => {
                self.bestiary.record_sighting_boss(SpeciesId::BossGrof);
                let mut boss = CreatureInstance::new(SpeciesId::BossGrof, 11, true);
                boss.artifact = Some(ArtifactId::AncientSigil);
                boss.heal_full();
                enemies[4] = Some(boss);
                enemies[0] = Some(CreatureInstance::new(SpeciesId::BogGolem, 10, false));
                enemies[1] = Some(CreatureInstance::new(SpeciesId::BriarToad, 10, true));
                enemies[2] = Some(CreatureInstance::new(SpeciesId::RuneBeetle, 10, false));
                enemies[3] = Some(CreatureInstance::new(SpeciesId::CrownSalamander, 10, false));
                enemies[5] = Some(CreatureInstance::new(SpeciesId::GloomMoth, 10, false));
                self.desired_bgm_track = Some(4);
                ("GROF, MALEVOLENT SORCERER", true)
            }
        };

        let seed = self.next_rand();
        self.active_battle = Some(BattleState::new(
            &self.squad,
            enemies,
            is_true_boss,
            self.zone_num,
            title.to_string(),
            seed,
        ));
        self.mode = GameMode::Battle;
        sfx_out.push(Sfx::BombExplode);
    }

    fn update_battle_mode(&mut self, input: &InputState, sfx_out: &mut Vec<Sfx>) {
        let mut pods = self.pods_left;
        let mut outcome: Option<bool> = None;

        if let Some(ref mut bs) = self.active_battle {
            bs.handle_player_input(
                input.up_pressed,
                input.down_pressed,
                input.left_pressed,
                input.right_pressed,
                input.confirm_pressed,
                input.cancel_pressed,
                input.special_pressed,
                &mut pods,
                sfx_out,
            );
            bs.tick_animations_and_flow(&mut pods, sfx_out);

            match bs.phase {
                BattlePhase::Victory { ticks } if ticks == 0 || input.confirm_pressed => {
                    outcome = Some(true);
                }
                BattlePhase::Defeat { ticks } if ticks == 0 || input.confirm_pressed => {
                    outcome = Some(false);
                }
                _ => {}
            }
        }
        self.pods_left = pods;

        if let Some(won) = outcome {
            if won {
                self.finish_battle_victory();
            } else {
                self.active_battle = None;
                self.mode = GameMode::RunDefeat;
            }
        }
    }

    fn finish_battle_victory(&mut self) {
        let Some(bs) = self.active_battle.take() else {
            self.mode = GameMode::SideWorld;
            return;
        };

        bs.sync_back_to_squad(&mut self.squad);
        self.battles_won += 1;
        self.encounter_grace_ticks = 24;

        // Grant XP + 25% post-battle breather recovery to active 3x2 squad!
        for s in 0..6 {
            if let Some(ref mut c) = self.squad[s] {
                c.add_xp(bs.xp_reward);
                let rec = (c.max_hp() * 20) / 100 + 3;
                c.cur_hp = (c.cur_hp + rec).min(c.max_hp());
            }
        }

        // Check if this victory defeated the Zone Guardian!
        let mut beat_guardian = false;
        if let Some(ref mut g) = self.guardian {
            if !g.defeated && bs.banner_title == g.title {
                g.defeated = true;
                beat_guardian = true;
                self.pods_left += 2;
                let gx = (g.col as i32) * TS;
                let gy = (g.row as i32) * TS - 12;
                self.add_toast(gx, gy, "GATE UNLOCKED! +2 PODS!".to_string(), 15);
            }
        }

        // Process Captured Wild Creatures -> Bestiary Unlock + Active 3x2 / 2-Slot Reserve!
        for mut cap in bs.captured_units {
            let is_new = self.bestiary.record_capture(cap.species);
            cap.heal_full();
            if is_new {
                let px = self.px;
                let py = self.py - 12;
                self.add_toast(
                    px,
                    py,
                    format!("NEW BESTIARY: {}!", cap.species.meta().short_name),
                    15,
                );
            }

            if let Some(empty_active) = (0..6).find(|&i| self.squad[i].is_none()) {
                self.squad[empty_active] = Some(cap);
            } else if let Some(empty_res) = (0..2).find(|&i| self.reserve[i].is_none()) {
                self.reserve[empty_res] = Some(cap);
            } else {
                self.pending_captures.push(cap);
            }
        }

        if !self.pending_captures.is_empty() {
            self.mode = GameMode::CaptureOverflow { cursor: 0 };
        } else if beat_guardian || self.battles_won % 2 == 1 {
            self.roll_post_battle_loot_offers();
            self.mode = GameMode::LootDraft { cursor: 0 };
        } else {
            self.mode = GameMode::SideWorld;
        }
    }

    fn slot_ref(&self, idx: usize) -> &Option<CreatureInstance> {
        if idx < 6 {
            &self.squad[idx]
        } else {
            &self.reserve[idx - 6]
        }
    }

    fn slot_mut(&mut self, idx: usize) -> &mut Option<CreatureInstance> {
        if idx < 6 {
            &mut self.squad[idx]
        } else {
            &mut self.reserve[idx - 6]
        }
    }

    fn return_creature_gear_to_bag(&mut self, c: &CreatureInstance) {
        if c.spell_gem != c.species.meta().default_gem {
            self.gem_bag.push(c.spell_gem);
        }
        if let Some(art) = c.artifact {
            self.artifact_bag.push(art);
        }
    }

    fn update_capture_overflow(
        &mut self,
        mut cursor: usize,
        input: &InputState,
        sfx_out: &mut Vec<Sfx>,
    ) {
        if self.pending_captures.is_empty() {
            self.mode = GameMode::SideWorld;
            return;
        }
        // 0..8 = replace slot 0..8 (returning gear!), 8 = release new capture for +20 Squad XP
        if input.up_pressed || input.left_pressed {
            cursor = (cursor + 8) % 9;
            sfx_out.push(Sfx::SuctionStick);
        } else if input.down_pressed || input.right_pressed {
            cursor = (cursor + 1) % 9;
            sfx_out.push(Sfx::SuctionStick);
        }
        self.mode = GameMode::CaptureOverflow { cursor };

        if input.confirm_pressed {
            let incoming = self.pending_captures.remove(0);
            if cursor < 8 {
                if let Some(old) = self.slot_mut(cursor).take() {
                    self.return_creature_gear_to_bag(&old);
                }
                *self.slot_mut(cursor) = Some(incoming);
            } else {
                self.return_creature_gear_to_bag(&incoming);
                for s in 0..6 {
                    if let Some(ref mut c) = self.squad[s] {
                        c.add_xp(18);
                    }
                }
            }
            sfx_out.push(Sfx::HarvestPluck);
            if self.pending_captures.is_empty() {
                self.roll_post_battle_loot_offers();
                self.mode = GameMode::LootDraft { cursor: 0 };
            }
        }
    }

    fn update_loot_draft(
        &mut self,
        mut cursor: usize,
        input: &InputState,
        sfx_out: &mut Vec<Sfx>,
    ) {
        if input.left_pressed {
            cursor = (cursor + 2) % 3;
            sfx_out.push(Sfx::SuctionStick);
        } else if input.right_pressed {
            cursor = (cursor + 1) % 3;
            sfx_out.push(Sfx::SuctionStick);
        }
        self.mode = GameMode::LootDraft { cursor };

        if input.confirm_pressed {
            match self.loot_offers[cursor] {
                GearReward::Gem(g) => {
                    self.gem_bag.push(g);
                }
                GearReward::Artifact(a) => {
                    // Auto-equip onto first active creature lacking an artifact, else add to bag!
                    let mut placed = false;
                    for s in 0..6 {
                        if let Some(ref mut c) = self.squad[s] {
                            if c.artifact.is_none() {
                                c.artifact = Some(a);
                                c.heal_full();
                                placed = true;
                                break;
                            }
                        }
                    }
                    if !placed {
                        self.artifact_bag.push(a);
                    }
                }
            }
            sfx_out.push(Sfx::ServiceBell);
            self.mode = GameMode::SideWorld;
        } else if input.cancel_pressed {
            self.mode = GameMode::SideWorld;
        }
    }

    fn update_squad_menu(
        &mut self,
        mut cursor: usize,
        sub: SquadSubMode,
        mut bag_cursor: usize,
        input: &InputState,
        sfx_out: &mut Vec<Sfx>,
    ) {
        if input.squad_pressed {
            self.mode = GameMode::SideWorld;
            sfx_out.push(Sfx::SuctionStick);
            return;
        }

        match sub {
            SquadSubMode::BrowseSlots | SquadSubMode::SwapWith { .. } => {
                if input.cancel_pressed {
                    if matches!(sub, SquadSubMode::SwapWith { .. }) {
                        self.mode = GameMode::SquadMenu {
                            cursor,
                            sub: SquadSubMode::BrowseSlots,
                            bag_cursor: 0,
                        };
                    } else {
                        self.mode = GameMode::SideWorld;
                    }
                    sfx_out.push(Sfx::SuctionStick);
                    return;
                }

                if input.up_pressed {
                    cursor = (cursor + 7) % 8;
                    sfx_out.push(Sfx::SuctionStick);
                } else if input.down_pressed {
                    cursor = (cursor + 1) % 8;
                    sfx_out.push(Sfx::SuctionStick);
                } else if input.left_pressed || input.right_pressed {
                    cursor = match cursor {
                        0..=2 => cursor + 3,
                        3..=5 => cursor - 3,
                        6 => 7,
                        _ => 6,
                    };
                    sfx_out.push(Sfx::SuctionStick);
                }

                if input.confirm_pressed {
                    if let SquadSubMode::SwapWith { first_slot } = sub {
                        if first_slot != cursor {
                            let a = self.slot_mut(first_slot).take();
                            let b = self.slot_mut(cursor).take();
                            *self.slot_mut(first_slot) = b;
                            *self.slot_mut(cursor) = a;
                            // Ensure at least 1 active creature remains in 0..6!
                            if self.squad.iter().all(|s| s.is_none()) {
                                let a = self.slot_mut(first_slot).take();
                                let b = self.slot_mut(cursor).take();
                                *self.slot_mut(first_slot) = b;
                                *self.slot_mut(cursor) = a;
                            }
                        }
                        self.mode = GameMode::SquadMenu {
                            cursor,
                            sub: SquadSubMode::BrowseSlots,
                            bag_cursor: 0,
                        };
                        sfx_out.push(Sfx::HarvestPluck);
                        return;
                    } else {
                        self.mode = GameMode::SquadMenu {
                            cursor,
                            sub: SquadSubMode::SwapWith { first_slot: cursor },
                            bag_cursor: 0,
                        };
                        sfx_out.push(Sfx::SuctionStick);
                        return;
                    }
                }

                // [C] / [T] opens Pre-Loaded Gear Swapper for the highlighted creature!
                if input.special_pressed && self.slot_ref(cursor).is_some() {
                    let choose_art = !self.artifact_bag.is_empty() && self.gem_bag.is_empty();
                    self.mode = GameMode::SquadMenu {
                        cursor,
                        sub: SquadSubMode::PickGearSlot {
                            slot: cursor,
                            choosing_artifact: choose_art,
                        },
                        bag_cursor: 0,
                    };
                    sfx_out.push(Sfx::SuctionStick);
                    return;
                }

                self.mode = GameMode::SquadMenu {
                    cursor,
                    sub,
                    bag_cursor,
                };
            }

            SquadSubMode::PickGearSlot {
                slot,
                mut choosing_artifact,
            } => {
                if input.cancel_pressed {
                    self.mode = GameMode::SquadMenu {
                        cursor: slot,
                        sub: SquadSubMode::BrowseSlots,
                        bag_cursor: 0,
                    };
                    sfx_out.push(Sfx::SuctionStick);
                    return;
                }
                if input.left_pressed || input.right_pressed {
                    choosing_artifact = !choosing_artifact;
                    bag_cursor = 0;
                    sfx_out.push(Sfx::SuctionStick);
                }
                let len = if choosing_artifact {
                    self.artifact_bag.len()
                } else {
                    self.gem_bag.len()
                };
                if len > 0 {
                    if input.up_pressed {
                        bag_cursor = (bag_cursor + len - 1) % len;
                        sfx_out.push(Sfx::SuctionStick);
                    } else if input.down_pressed {
                        bag_cursor = (bag_cursor + 1) % len;
                        sfx_out.push(Sfx::SuctionStick);
                    } else if input.confirm_pressed {
                        if choosing_artifact {
                            let new_art = self.artifact_bag.remove(bag_cursor);
                            let mut old_art = None;
                            if let Some(ref mut c) = self.slot_mut(slot) {
                                old_art = c.artifact.replace(new_art);
                                c.cur_hp = c.cur_hp.min(c.max_hp());
                            }
                            if let Some(oa) = old_art {
                                self.artifact_bag.push(oa);
                            }
                        } else {
                            let new_gem = self.gem_bag.remove(bag_cursor);
                            let mut old_gem = None;
                            if let Some(ref mut c) = self.slot_mut(slot) {
                                old_gem = Some(std::mem::replace(&mut c.spell_gem, new_gem));
                                c.cur_hp = c.cur_hp.min(c.max_hp());
                            }
                            if let Some(og) = old_gem {
                                self.gem_bag.push(og);
                            }
                        }
                        sfx_out.push(Sfx::ServiceBell);
                        bag_cursor = 0;
                    }
                }
                self.mode = GameMode::SquadMenu {
                    cursor: slot,
                    sub: SquadSubMode::PickGearSlot {
                        slot,
                        choosing_artifact,
                    },
                    bag_cursor,
                };
            }
        }
    }

    // =========================================================================
    // 400x300 64-COLOR VGA MASTER RENDERER
    // =========================================================================
    pub fn render(&self, fb: &mut Framebuffer, assets: &AssetBank, track_title: &str) {
        match self.mode {
            GameMode::StarterDraft { step, cursor } => {
                self.render_starter_draft(fb, assets, step, cursor);
            }
            GameMode::SideWorld => {
                self.render_side_world(fb, assets, track_title);
            }
            GameMode::Battle => {
                if let Some(ref bs) = self.active_battle {
                    bs.render(fb, assets, self.pods_left);
                }
            }
            GameMode::CaptureOverflow { cursor } => {
                self.render_capture_overflow(fb, assets, cursor);
            }
            GameMode::LootDraft { cursor } => {
                self.render_loot_draft(fb, assets, cursor);
            }
            GameMode::SquadMenu {
                cursor,
                sub,
                bag_cursor,
            } => {
                self.render_squad_menu(fb, assets, cursor, sub, bag_cursor);
            }
            GameMode::RunVictory => {
                self.render_end_screen(fb, assets, true);
            }
            GameMode::RunDefeat => {
                self.render_end_screen(fb, assets, false);
            }
        }

        if self.show_bestiary_overlay {
            self.render_bestiary_overlay(fb, assets);
        }

        if self.quit_confirm_open {
            fb.rect_dither(0, 0, FB_W as i32, FB_H as i32, 1, 2);
            fb.rect_bevel(64, 108, 272, 80, 2, 15, 1, 1);
            fb.draw_text_centered("EXIT GAME TO DESKTOP?", 200, 122, 15, Some(1), 1);
            fb.draw_text_centered(
                "[ENTER / Y]: QUIT TO DESKTOP",
                200,
                144,
                38,
                Some(1),
                1,
            );
            fb.draw_text_centered(
                "[ESC / X]: RESUME EXPEDITION",
                200,
                162,
                25,
                Some(1),
                1,
            );
        }
    }

    fn render_starter_draft(
        &self,
        fb: &mut Framebuffer,
        assets: &AssetBank,
        step: DraftStep,
        cursor: usize,
    ) {
        fb.blit_parallax_bg(&assets.bg_canopy, 0, 0, FB_H);
        fb.rect_dither(0, 0, FB_W as i32, FB_H as i32, 1, 2);

        fb.rect_bevel(6, 4, 388, 28, 2, 4, 1, 1);
        let title = match step {
            DraftStep::StarterOne => "EXPEDITION DRAFT 1/3: PICK VANGUARD STARTER",
            DraftStep::StarterTwo => "EXPEDITION DRAFT 2/3: PICK SYNERGY PARTNER",
            DraftStep::StarterArtifact => "EXPEDITION DRAFT 3/3: PICK STARTER ARTIFACT",
        };
        fb.draw_text_centered(title, 200, 8, 15, Some(1), 1);
        let sub_hint = format!(
            "LEFT/RIGHT: INSPECT   [ENTER/Z]: DRAFT   BESTIARY: {}/{}",
            self.bestiary.total_unlocked(),
            NUM_SPECIES
        );
        fb.draw_text_centered(&sub_hint, 200, 19, 34, Some(1), 1);

        if step != DraftStep::StarterArtifact {
            for i in 0..3 {
                let sp = self.draft_species_offers[i];
                let meta = sp.meta();
                let inst = CreatureInstance::new(sp, 3, false);
                let cx = 10 + (i as i32) * 128;
                let cy = 38;
                let sel = cursor == i;
                fb.rect_bevel(
                    cx,
                    cy,
                    124,
                    152,
                    if sel { 18 } else { 2 },
                    if sel { 15 } else { 4 },
                    1,
                    1,
                );

                fb.draw_text_centered(meta.name, cx + 62, cy + 7, if sel { 15 } else { 7 }, Some(1), 1);
                fb.draw_text_centered(
                    &format!("[{}]", meta.affinity.label()),
                    cx + 62,
                    cy + 19,
                    meta.affinity.color(),
                    Some(1),
                    1,
                );

                // Grounded specimen pedestal (row 23 sits directly on pedestal top at cy + 62!)
                fb.rect_bevel(cx + 38, cy + 62, 48, 8, 3, 6, 1, 1);
                fb.blit_sprite(&assets.monsters[sp as usize], cx + 50, cy + 38, false, false, None);

                fb.draw_text(
                    &format!("HP :{:2}  ATK:{:2}", inst.max_hp(), inst.atk()),
                    cx + 12,
                    cy + 78,
                    7,
                    Some(1),
                    1,
                );
                fb.draw_text(
                    &format!("DEF:{:2}  INT:{:2}", inst.def(), inst.int_stat()),
                    cx + 12,
                    cy + 90,
                    7,
                    Some(1),
                    1,
                );
                fb.draw_text(
                    &format!("SPD:{:2}", inst.spd()),
                    cx + 12,
                    cy + 102,
                    34,
                    Some(1),
                    1,
                );
                fb.draw_text("TRAIT:", cx + 12, cy + 118, 15, Some(1), 1);
                fb.draw_text(meta.trait_name, cx + 12, cy + 129, 25, Some(1), 1);
            }

            // Bottom Full Synergy Inspector (y = 196..294)
            let sp = self.draft_species_offers[cursor];
            let m = sp.meta();
            let gm = m.default_gem.meta();
            fb.rect_bevel(6, 196, 388, 98, 2, 4, 1, 1);
            if let Some(ref pick1) = self.squad[0] {
                fb.draw_text(
                    &format!("PICK #1 LOCKED: {} [{}]", pick1.species.meta().name, pick1.species.meta().trait_name),
                    14,
                    202,
                    42,
                    Some(1),
                    1,
                );
            } else {
                fb.draw_text(
                    "TIP: PAIR HEALERS, CASTERS, OR BUFFERS TO SPARK TRAIT CHAINS!",
                    14,
                    202,
                    6,
                    Some(1),
                    1,
                );
            }
            fb.draw_text(&format!("INNATE TRAIT - {}:", m.trait_name), 14, 217, 15, Some(1), 1);
            fb.draw_text(m.trait_desc_1, 14, 229, 7, Some(1), 1);
            fb.draw_text(m.trait_desc_2, 14, 240, 7, Some(1), 1);
            fb.draw_text(&format!("STARTING GEM - {}:", gm.name), 14, 256, 34, Some(1), 1);
            fb.draw_text(gm.desc_1, 14, 268, 25, Some(1), 1);
            fb.draw_text("[H/F1]: OPEN FULL 14-SPECIES BESTIARY", 14, 281, 5, Some(1), 1);
        } else {
            for i in 0..3 {
                let art = self.draft_artifact_offers[i];
                let am = art.meta();
                let cx = 10 + (i as i32) * 128;
                let cy = 46;
                let sel = cursor == i;
                fb.rect_bevel(
                    cx,
                    cy,
                    124,
                    136,
                    if sel { 28 } else { 2 },
                    if sel { 15 } else { 4 },
                    1,
                    1,
                );
                fb.draw_text_centered("PRE-LOADED", cx + 62, cy + 12, 34, Some(1), 1);
                fb.draw_text_centered("ARTIFACT", cx + 62, cy + 23, 34, Some(1), 1);
                fb.rect_bevel(cx + 46, cy + 44, 32, 32, 3, 15, 1, 1);
                fb.blit_sprite(&assets.spr_pod_12, cx + 56, cy + 54, false, false, None);
                fb.draw_text_centered(am.name, cx + 62, cy + 94, if sel { 15 } else { 7 }, Some(1), 1);
                fb.draw_text_centered(
                    if sel { "> EQUIP & DROP <" } else { "READY TO EQUIP" },
                    cx + 62,
                    cy + 114,
                    25,
                    Some(1),
                    1,
                );
            }

            let am = self.draft_artifact_offers[cursor].meta();
            fb.rect_bevel(6, 196, 388, 98, 2, 4, 1, 1);
            fb.draw_text(&format!("ARTIFACT: {}", am.name), 14, 206, 15, Some(1), 1);
            fb.draw_text(am.desc_1, 14, 224, 25, Some(1), 1);
            fb.draw_text(am.desc_2, 14, 238, 7, Some(1), 1);
            fb.draw_text(
                "EQUIPS ON STARTER #1 (SWAP FREELY ANYTIME VIA [TAB] SQUAD MENU!)",
                14,
                262,
                34,
                Some(1),
                1,
            );
        }
    }

    fn render_side_world(&self, fb: &mut Framebuffer, assets: &AssetBank, track_title: &str) {
        fb.blit_parallax_bg(&assets.bg_playfield, self.cam_x, 0, PLAY_H as usize);

        let first_col = (self.cam_x / TS).max(0) as usize;
        let last_col = ((self.cam_x + FB_W as i32) / TS + 1).min(MAP_W as i32) as usize;
        let sway_frame = ((self.tick_count / 10) % 2) as usize;

        // Pass 1: Solid terrain, ladders, crystal grottos, caches, beacons & exit arch
        for r in 0..MAP_H {
            let sy = (r as i32) * TS;
            for c in first_col..last_col {
                let sx = (c as i32) * TS - self.cam_x;
                let variant = (c * 3 + r * 7) & 3;
                match self.tiles[r][c] {
                    WorldTile::StoneRoad => {
                        fb.blit_sprite(&assets.tile_stone_road[variant], sx, sy, false, false, None);
                    }
                    WorldTile::EarthSub => {
                        fb.blit_sprite(&assets.tile_earth_sub[variant], sx, sy, false, false, None);
                    }
                    WorldTile::Ladder => {
                        fb.blit_sprite(&assets.tile_ladder, sx, sy, false, false, None);
                    }
                    WorldTile::CrystalPatch => {
                        fb.blit_sprite(
                            &assets.tile_crystal_grotto[sway_frame],
                            sx,
                            sy,
                            false,
                            false,
                            None,
                        );
                    }
                    WorldTile::StarWater => {
                        fb.blit_sprite(
                            &assets.tile_star_water[sway_frame],
                            sx,
                            sy,
                            false,
                            false,
                            None,
                        );
                    }
                    _ => {}
                }
            }
        }

        // East Sanctum Exit Archway (grounded cleanly on floor!)
        let ex_x = (self.exit_col as i32) * TS - self.cam_x;
        let ex_y = (self.exit_row as i32) * TS - 16;
        let boss_clear = self.guardian.as_ref().map(|g| g.defeated).unwrap_or(true);
        fb.rect_bevel(
            ex_x - 4,
            ex_y,
            28,
            36,
            if boss_clear { 33 } else { 16 },
            if boss_clear { 15 } else { 4 },
            1,
            1,
        );
        fb.draw_text(
            if boss_clear { "EXIT" } else { "LOCK" },
            ex_x - 2,
            ex_y + 4,
            if boss_clear { 15 } else { 5 },
            Some(1),
            1,
        );

        // Supply Caches (20x20 grounded on row * TS)
        for cache in &self.caches {
            let cx = (cache.col as i32) * TS - self.cam_x;
            let cy = (cache.row as i32) * TS;
            let spr = if cache.opened {
                &assets.spr_cache_open
            } else {
                &assets.spr_cache_closed
            };
            fb.blit_sprite(spr, cx, cy, false, false, None);
            if !cache.opened
                && (self.px - (cache.col as i32) * TS).abs() <= 26
                && (self.py - (cache.row as i32) * TS).abs() <= 26
            {
                fb.draw_text("[E]:CACHE", cx - 14, cy - 10, 15, Some(1), 1);
            }
        }

        // Field Med-Beacon
        if let Some(ref b) = self.beacon {
            let bx = (b.col as i32) * TS - self.cam_x;
            let by = (b.row as i32) * TS;
            fb.blit_sprite(&assets.spr_med_beacon, bx, by, false, false, None);
            if !b.used {
                fb.draw_text("HEAL", bx - 2, by - 9, 25, Some(1), 1);
                if (self.px - (b.col as i32) * TS).abs() <= 28
                    && (self.py - (b.row as i32) * TS).abs() <= 28
                {
                    fb.draw_text("[E]:HEAL+2POD", bx - 26, by - 19, 15, Some(1), 1);
                }
            }
        }

        // Zone Guardian Boss Sprite (24x24 grounded strictly at (row + 1) * TS - 24!)
        if let Some(ref g) = self.guardian {
            if !g.defeated {
                let gx = (g.col as i32) * TS - self.cam_x - 2;
                let gy = ((g.row + 1) as i32) * TS - (MON_SZ as i32);
                fb.blit_sprite(
                    &assets.monsters[g.lead_species as usize],
                    gx,
                    gy,
                    true,
                    false,
                    None,
                );
                fb.draw_text("BOSS", gx + 1, gy - 10, 38, Some(1), 1);
                if (self.px - (g.col as i32) * TS).abs() <= 38
                    && (self.py - (g.row as i32) * TS).abs() <= 28
                {
                    fb.draw_text("[E]:FIGHT!", gx - 16, gy - 20, 15, Some(1), 1);
                }
            }
        }

        // Blue Alien Xenobiologist (20x20 grounded strictly on py + 19!)
        let alien_spr = if self.climbing {
            if (self.walk_frame / 4) % 2 == 0 {
                &assets.alien_climb1
            } else {
                &assets.alien_climb2
            }
        } else {
            match self.walk_frame / 3 {
                1 => &assets.alien_run1,
                2 => &assets.alien_run2,
                3 => &assets.alien_run3,
                _ => &assets.alien_idle,
            }
        };
        fb.blit_sprite(
            alien_spr,
            self.px - self.cam_x,
            self.py,
            !self.face_right && !self.climbing,
            false,
            None,
        );

        // Pass 2: Foreground Pokemon-Style Tall Fern Grass, Alpha Spores & Waist-Deep Star-Pond Water!
        let p_col = ((self.px + 10) / TS) as usize;
        let p_row = ((self.py + 14) / TS) as usize;
        for r in 0..MAP_H {
            let sy = (r as i32) * TS;
            for c in first_col..last_col {
                let sx = (c as i32) * TS - self.cam_x;
                let rustle = if c == p_col && r == p_row && self.walk_frame > 0 {
                    1 - sway_frame
                } else {
                    sway_frame
                };
                match self.tiles[r][c] {
                    WorldTile::FernPatch => {
                        fb.blit_sprite(&assets.tile_fern_grass[rustle], sx, sy, false, false, None);
                    }
                    WorldTile::AlphaPatch => {
                        fb.blit_sprite(&assets.tile_alpha_grove[rustle], sx, sy, false, false, None);
                    }
                    WorldTile::StarWater => {
                        // Classic DOS 2x2 Bayer wading shimmer across boots (zero RGBA alpha!)
                        fb.rect_dither(sx, sy + 9, TS, TS - 9, 33, 2);
                        fb.rect_fill(
                            sx,
                            sy + 8,
                            TS,
                            1,
                            if rustle == 0 { 35 } else { 34 },
                        );
                    }
                    _ => {}
                }
            }
        }

        // Top Zone & Habitat Banner (y = 3..25)
        let hab = self.current_player_habitat();
        fb.rect_bevel(4, 3, 392, 22, 2, 4, 1, 1);
        fb.draw_text(&self.world_banner, 10, 6, 15, Some(1), 1);
        fb.draw_text(
            &format!("LV.{}", self.zone_wild_level()),
            238,
            6,
            34,
            Some(1),
            1,
        );
        fb.draw_text("[TAB]SQUAD [H]DEX", 286, 6, 6, Some(1), 1);
        fb.draw_text(
            hab.hud_name(),
            10,
            15,
            if hab.is_encounter_zone() { 25 } else { 6 },
            Some(1),
            1,
        );
        if hab.is_encounter_zone() {
            fb.draw_text("[C]:HUNT NOW!", 310, 15, 15, Some(1), 1);
        }

        // Floating World Toasts
        for t in &self.toasts {
            let tx = (t.x - self.cam_x - (t.text.len() as i32) * 3).clamp(4, 396 - (t.text.len() as i32) * 6);
            let ty = t.y.clamp(28, 224);
            fb.rect_fill(tx - 2, ty - 1, (t.text.len() as i32) * 6 + 4, 10, 1);
            fb.draw_text(&t.text, tx, ty, t.col, None, 1);
        }

        // Bottom 60px Classic Apogee VGA Status Bar (y = 240..300)
        self.render_world_status_bar(fb, assets, track_title, hab);
    }

    fn render_world_status_bar(
        &self,
        fb: &mut Framebuffer,
        assets: &AssetBank,
        track_title: &str,
        hab: HabitatKind,
    ) {
        fb.rect_bevel(0, PLAY_H, FB_W as i32, FB_H as i32 - PLAY_H, 2, 4, 1, 1);

        // Controls Ribbon (y = 243..254)
        fb.rect_fill(4, PLAY_H + 3, FB_W as i32 - 8, 11, 1);
        fb.draw_text_centered(
            "ARROWS:WALK/CLIMB  [E/ENT]:ACT  [C]:HUNT HABITAT  [TAB]:3X2 SQUAD",
            200,
            PLAY_H + 5,
            15,
            None,
            1,
        );

        let row_y = PLAY_H + 16;

        // 1. 2x Integer Blue Alien Portrait (x = 6..50)
        fb.rect_bevel(6, row_y, 44, 41, 16, 4, 1, 1);
        fb.blit_sprite(&assets.alien_portrait_36, 10, row_y + 2, false, false, None);

        // 2. Habitat Encounter Radar, Containment Pods & BGM (x = 54..216)
        fb.rect_bevel(54, row_y, 162, 41, 16, 4, 1, 1);
        fb.blit_sprite(&assets.spr_pod_12, 58, row_y + 3, false, false, None);
        fb.draw_text(
            &format!("PODS:{}  BAG:{}", self.pods_left, self.gem_bag.len() + self.artifact_bag.len()),
            73,
            row_y + 5,
            15,
            Some(1),
            1,
        );

        // Step Encounter Tension Bar!
        fb.draw_text("RADAR:", 60, row_y + 17, 6, Some(1), 1);
        fb.rect_fill(98, row_y + 18, 110, 6, 1);
        let fill = ((108 * self.encounter_meter.clamp(0, 100)) / 100).max(0);
        let r_col = if hab == HabitatKind::AlphaGrove {
            15
        } else if hab.is_encounter_zone() {
            25
        } else {
            4
        };
        if fill > 0 {
            fb.rect_fill(99, row_y + 19, fill, 4, r_col);
        }

        let clean_bgm: String = track_title.chars().take(20).collect();
        fb.draw_text(&format!("BGM:{}", clean_bgm), 60, row_y + 29, 34, Some(1), 1);

        // 3. Live 3x2 Mini-Squad Formation Readout (x = 220..394, 3 cols x 2 rows or Back/Front!)
        fb.rect_bevel(220, row_y, 174, 41, 16, 4, 1, 1);
        for slot in 0..6 {
            // Top row = Front (0..3), Bottom row = Back (3..6)
            let col_idx = (slot % 3) as i32;
            let row_idx = (slot / 3) as i32;
            let sx = 224 + col_idx * 56;
            let sy = row_y + 3 + row_idx * 18;
            if let Some(ref c) = self.squad[slot] {
                let m = c.species.meta();
                fb.rect_bevel(sx, sy, 54, 16, 2, m.affinity.color(), 1, 1);
                let tag: String = m.short_name.chars().take(4).collect();
                fb.draw_text(
                    &format!("{} L{}", tag, c.level),
                    sx + 3,
                    sy + 3,
                    7,
                    Some(1),
                    1,
                );
                let hp_w = ((48 * c.cur_hp.max(0)) / c.max_hp().max(1)).clamp(1, 48);
                fb.rect_fill(sx + 3, sy + 12, 48, 2, 36);
                fb.rect_fill(sx + 3, sy + 12, hp_w, 2, 25);
            } else {
                fb.rect_dither(sx, sy, 54, 16, 1, 1);
                fb.draw_text(
                    if slot < 3 { "FRNT -" } else { "BACK -" },
                    sx + 9,
                    sy + 4,
                    4,
                    None,
                    1,
                );
            }
        }
    }

    fn render_squad_menu(
        &self,
        fb: &mut Framebuffer,
        assets: &AssetBank,
        cursor: usize,
        sub: SquadSubMode,
        bag_cursor: usize,
    ) {
        fb.clear(16);
        fb.rect_bevel(4, 4, 392, 24, 2, 4, 1, 1);
        fb.draw_text("3X2 FORMATION, RESERVE & PRE-LOADED GEAR", 10, 8, 15, Some(1), 1);
        fb.draw_text(
            "ARROWS:SELECT  [Z/ENT]:MOVE SLOT  [C]:EQUIP GEAR  [TAB/X]:EXIT",
            10,
            17,
            34,
            Some(1),
            1,
        );

        // Left Panel: 3x2 Active Grid (Back Col x=8, Front Col x=104) + 2 Reserve Slots (y=164)
        fb.draw_text("BACK COLUMN", 20, 31, 6, Some(1), 1);
        fb.draw_text("FRONT COLUMN", 114, 31, 15, Some(1), 1);

        for idx in 0..8 {
            let (sx, sy) = if idx < 3 {
                (104, 41 + (idx as i32) * 40) // Front Col (0..3)
            } else if idx < 6 {
                (6, 41 + ((idx - 3) as i32) * 40) // Back Col (3..6)
            } else {
                (6 + ((idx - 6) as i32) * 98, 173) // 2-Slot Field Reserve (6..8)
            };
            if idx == 6 {
                fb.draw_text("2-SLOT FIELD RESERVE:", 8, 163, 6, Some(1), 1);
            }

            let is_cur = cursor == idx;
            let is_swap_src = matches!(sub, SquadSubMode::SwapWith { first_slot } if first_slot == idx);
            let fill = if is_swap_src {
                28
            } else if is_cur {
                18
            } else {
                2
            };
            let hi = if is_cur || is_swap_src { 15 } else { 4 };
            fb.rect_bevel(sx, sy, 96, 37, fill, hi, 1, 1);

            if let Some(ref c) = self.slot_ref(idx) {
                let m = c.species.meta();
                fb.blit_sprite(
                    &assets.monsters[c.species as usize],
                    sx + 2,
                    sy + 10,
                    false,
                    false,
                    None,
                );
                let pfx = if c.is_alpha { "*" } else { "" };
                fb.draw_text(
                    &format!("{}{} L{}", pfx, m.short_name, c.level),
                    sx + 29,
                    sy + 4,
                    if c.is_alpha { 15 } else { m.affinity.color() },
                    Some(1),
                    1,
                );
                fb.draw_text(
                    &format!("HP:{}/{}", c.cur_hp, c.max_hp()),
                    sx + 29,
                    sy + 15,
                    25,
                    Some(1),
                    1,
                );
                let art_tag = c.artifact.map(|a| a.meta().short_name).unwrap_or("NO ART.");
                let at_trim: String = art_tag.chars().take(10).collect();
                fb.draw_text(&at_trim, sx + 29, sy + 26, 34, Some(1), 1);
            } else {
                fb.draw_text_centered("- EMPTY -", sx + 48, sy + 15, 5, None, 1);
            }
        }

        // Right Panel (x = 204..396, y = 32..210): Selected Creature Trait & Gear Or Bag Picker!
        fb.rect_bevel(204, 32, 192, 178, 2, 4, 1, 1);
        if let SquadSubMode::PickGearSlot {
            slot,
            choosing_artifact,
        } = sub
        {
            fb.draw_text(
                if choosing_artifact {
                    "< ARTIFACT BAG > (L/R:GEMS)"
                } else {
                    "< SPELL GEM BAG > (L/R:ART)"
                },
                210,
                38,
                15,
                Some(1),
                1,
            );
            if choosing_artifact {
                if self.artifact_bag.is_empty() {
                    fb.draw_text("NO SPARE ARTIFACTS IN BAG!", 210, 62, 5, Some(1), 1);
                } else {
                    for (i, &a) in self.artifact_bag.iter().take(6).enumerate() {
                        let y = 54 + (i as i32) * 16;
                        let sel = i == bag_cursor;
                        fb.draw_text(
                            &format!("{}{}", if sel { "> " } else { "  " }, a.meta().name),
                            210,
                            y,
                            if sel { 15 } else { 7 },
                            Some(1),
                            1,
                        );
                    }
                    if let Some(&a) = self.artifact_bag.get(bag_cursor) {
                        let d1: String = a.meta().desc_1.chars().take(30).collect();
                        let d2: String = a.meta().desc_2.chars().take(30).collect();
                        fb.draw_text(&d1, 210, 166, 25, Some(1), 1);
                        fb.draw_text(&d2, 210, 178, 6, Some(1), 1);
                    }
                }
            } else if self.gem_bag.is_empty() {
                fb.draw_text("NO SPARE SPELL GEMS IN BAG!", 210, 62, 5, Some(1), 1);
            } else {
                for (i, &g) in self.gem_bag.iter().take(6).enumerate() {
                    let y = 54 + (i as i32) * 16;
                    let sel = i == bag_cursor;
                    fb.draw_text(
                        &format!("{}{}", if sel { "> " } else { "  " }, g.meta().name),
                        210,
                        y,
                        if sel { 15 } else { 7 },
                        Some(1),
                        1,
                    );
                }
                if let Some(&g) = self.gem_bag.get(bag_cursor) {
                    let d1: String = g.meta().desc_1.chars().take(30).collect();
                    let d2: String = g.meta().desc_2.chars().take(30).collect();
                    fb.draw_text(&d1, 210, 166, 34, Some(1), 1);
                    fb.draw_text(&d2, 210, 178, 6, Some(1), 1);
                }
            }
            let _ = slot;
        } else if let Some(ref c) = self.slot_ref(cursor) {
            let m = c.species.meta();
            fb.draw_text(m.name, 210, 38, 15, Some(1), 1);
            fb.draw_text(
                &format!("LV.{} [{}] XP:{}/{}", c.level, m.affinity.label(), c.xp, c.xp_to_next()),
                210,
                50,
                m.affinity.color(),
                Some(1),
                1,
            );
            fb.draw_text(
                &format!("ATK:{} DEF:{} INT:{} SPD:{}", c.atk(), c.def(), c.int_stat(), c.spd()),
                210,
                64,
                7,
                Some(1),
                1,
            );
            fb.draw_text(&format!("GEM: {}", c.spell_gem.meta().name), 210, 82, 34, Some(1), 1);
            let art_name = c.artifact.map(|a| a.meta().name).unwrap_or("- NONE -");
            fb.draw_text(&format!("ART: {}", art_name), 210, 96, 42, Some(1), 1);
            fb.draw_text(
                &format!("BAG: {} GEMS, {} ARTIFACTS", self.gem_bag.len(), self.artifact_bag.len()),
                210,
                114,
                25,
                Some(1),
                1,
            );
            fb.draw_text("PRESS [C] TO SWAP GEM/ART!", 210, 128, 15, Some(1), 1);
        }

        // Bottom Wide Trait & Gear Description Box (y = 214..296)
        fb.rect_bevel(4, 214, 392, 82, 2, 4, 1, 1);
        if let Some(ref c) = self.slot_ref(cursor) {
            let m = c.species.meta();
            let gm = c.spell_gem.meta();
            fb.draw_text(
                &format!("TRAIT [{}]: {}", m.trait_name, m.trait_desc_1),
                10,
                220,
                15,
                Some(1),
                1,
            );
            fb.draw_text(m.trait_desc_2, 10, 232, 7, Some(1), 1);
            fb.draw_text(&format!("GEM: {}", gm.desc_1), 10, 248, 34, Some(1), 1);
            if let Some(a) = c.artifact {
                fb.draw_text(
                    &format!("ART [{}]: {}", a.meta().short_name, a.meta().desc_2),
                    10,
                    262,
                    42,
                    Some(1),
                    1,
                );
            } else {
                fb.draw_text("ARTIFACT: NONE EQUIPPED ([C] TO EQUIP FROM BAG)", 10, 262, 5, Some(1), 1);
            }
            fb.draw_text(
                "TIP: FRONT ROW BLOCKS FOES | ADJACENT = UP/DOWN + ROW-MATE!",
                10,
                279,
                25,
                Some(1),
                1,
            );
        }
    }

    fn render_capture_overflow(&self, fb: &mut Framebuffer, assets: &AssetBank, cursor: usize) {
        fb.clear(16);
        let Some(incoming) = self.pending_captures.first() else {
            return;
        };
        let im = incoming.species.meta();

        fb.rect_bevel(8, 6, 384, 48, 28, 15, 1, 1);
        fb.blit_sprite(
            &assets.monsters[incoming.species as usize],
            16,
            18,
            false,
            false,
            None,
        );
        fb.draw_text(
            &format!("ROSTER FULL (8/8)! ASSIGN {} LV.{}", im.name, incoming.level),
            48,
            12,
            15,
            Some(1),
            1,
        );
        fb.draw_text(
            &format!("TRAIT [{}]: {}", im.trait_name, im.trait_desc_1),
            48,
            25,
            34,
            Some(1),
            1,
        );
        fb.draw_text("REPLACING AUTO-RETURNS EQUIPPED GEAR TO YOUR BAG!", 48, 38, 25, Some(1), 1);

        for i in 0..8 {
            let y = 60 + (i as i32) * 22;
            let sel = cursor == i;
            fb.rect_bevel(12, y, 376, 20, if sel { 18 } else { 2 }, if sel { 15 } else { 4 }, 1, 1);
            let slot_lbl = if i < 3 {
                format!("FRONT {}", i + 1)
            } else if i < 6 {
                format!("BACK  {}", i - 2)
            } else {
                format!("RESRV {}", i - 5)
            };
            if let Some(ref c) = self.slot_ref(i) {
                fb.draw_text(
                    &format!(
                        "{} [{}]: {} LV.{} ({})",
                        if sel { ">" } else { " " },
                        slot_lbl,
                        c.species.meta().name,
                        c.level,
                        c.species.meta().trait_name
                    ),
                    18,
                    y + 6,
                    if sel { 15 } else { 7 },
                    Some(1),
                    1,
                );
            }
        }

        let sel_rel = cursor == 8;
        fb.rect_bevel(
            12,
            242,
            376,
            24,
            if sel_rel { 36 } else { 2 },
            if sel_rel { 15 } else { 4 },
            1,
            1,
        );
        fb.draw_text_centered(
            "> RELEASE CAPTURE FOR +18 SQUAD XP (& KEEP ITS GEM) <",
            200,
            250,
            if sel_rel { 15 } else { 6 },
            Some(1),
            1,
        );
    }

    fn render_loot_draft(&self, fb: &mut Framebuffer, assets: &AssetBank, cursor: usize) {
        fb.blit_parallax_bg(&assets.bg_playfield, self.cam_x, 0, FB_H);
        fb.rect_dither(0, 0, FB_W as i32, FB_H as i32, 1, 2);

        fb.rect_bevel(10, 12, 380, 28, 2, 15, 1, 1);
        fb.draw_text_centered(
            "SPECIMEN SPOILS: DRAFT 1 PRE-LOADED GEAR PIECE",
            200,
            17,
            15,
            Some(1),
            1,
        );
        fb.draw_text_centered(
            "LEFT/RIGHT: SELECT   [ENTER/Z]: CLAIM   [X]: SKIP",
            200,
            28,
            34,
            Some(1),
            1,
        );

        for i in 0..3 {
            let cx = 10 + (i as i32) * 128;
            let cy = 50;
            let sel = cursor == i;
            fb.rect_bevel(
                cx,
                cy,
                124,
                130,
                if sel { 18 } else { 2 },
                if sel { 15 } else { 4 },
                1,
                1,
            );
            match self.loot_offers[i] {
                GearReward::Gem(g) => {
                    fb.draw_text_centered("SPELL GEM", cx + 62, cy + 12, 34, Some(1), 1);
                    fb.rect_bevel(cx + 46, cy + 36, 32, 32, 27, 34, 1, 1);
                    fb.circle_fill(cx + 62, cy + 52, 8, 34);
                    fb.draw_text_centered(g.meta().name, cx + 62, cy + 84, 15, Some(1), 1);
                }
                GearReward::Artifact(a) => {
                    fb.draw_text_centered("ARTIFACT", cx + 62, cy + 12, 42, Some(1), 1);
                    fb.rect_bevel(cx + 46, cy + 36, 32, 32, 19, 15, 1, 1);
                    fb.blit_sprite(&assets.spr_pod_12, cx + 56, cy + 46, false, false, None);
                    fb.draw_text_centered(a.meta().name, cx + 62, cy + 84, 15, Some(1), 1);
                }
            }
            fb.draw_text_centered(
                if sel { "> CLAIM <" } else { "PRE-LOADED" },
                cx + 62,
                cy + 108,
                25,
                Some(1),
                1,
            );
        }

        fb.rect_bevel(10, 192, 380, 88, 2, 4, 1, 1);
        match self.loot_offers[cursor] {
            GearReward::Gem(g) => {
                let gm = g.meta();
                fb.draw_text(&format!("SPELL GEM: {}", gm.name), 18, 202, 34, Some(1), 1);
                fb.draw_text(gm.desc_1, 18, 220, 15, Some(1), 1);
                fb.draw_text(gm.desc_2, 18, 234, 25, Some(1), 1);
                fb.draw_text("EQUIP ON ANY ALLY VIA [TAB] SQUAD MENU!", 18, 256, 6, Some(1), 1);
            }
            GearReward::Artifact(a) => {
                let am = a.meta();
                fb.draw_text(&format!("ARTIFACT: {}", am.name), 18, 202, 42, Some(1), 1);
                fb.draw_text(am.desc_1, 18, 220, 15, Some(1), 1);
                fb.draw_text(am.desc_2, 18, 234, 25, Some(1), 1);
                fb.draw_text(
                    "AUTO-EQUIPS ON FIRST OPEN ALLY SLOT (OR SAVES TO BAG)!",
                    18,
                    256,
                    6,
                    Some(1),
                    1,
                );
            }
        }
    }

    fn render_bestiary_overlay(&self, fb: &mut Framebuffer, assets: &AssetBank) {
        fb.rect_dither(0, 0, FB_W as i32, FB_H as i32, 1, 3);
        fb.rect_bevel(12, 10, 376, 280, 2, 15, 1, 1);

        let t0_col = if self.bestiary_tab == 0 { 15 } else { 5 };
        let t1_col = if self.bestiary_tab == 1 { 15 } else { 5 };
        fb.draw_text("[< GUIDE & CONTROLS]", 22, 18, t0_col, Some(1), 1);
        fb.draw_text(
            &format!(
                "[XENOBIOLOGY BESTIARY {}/{} >]",
                self.bestiary.total_unlocked(),
                NUM_SPECIES
            ),
            196,
            18,
            t1_col,
            Some(1),
            1,
        );
        fb.rect_fill(20, 30, 360, 1, 4);

        if self.bestiary_tab == 0 {
            let rows = [
                ("SIDE-VIEW HABITATS :", "STONE = 0% SAFE | FERNS/WATER/GROTTO = WILD!"),
                ("ALPHA SPORE-GROVES :", "GOLDEN PATCHES SPAWN +2 LV ALPHA LEADERS!"),
                ("[C] IN HABITAT     :", "TRIGGER INSTANT WILD HUNT (ZERO PACING!)"),
                ("CAPTURING (0% RNG) :", "CHIP WILD FOE TO 1%-40% HP (GOLD BAR) + POD!"),
                ("DEFEND / HOLD FIRE :", "HALVES DMG TAKEN & PAUSES OFFENSIVE TRAITS!"),
                ("[T] IN BATTLE      :", "TOGGLE TURBO AUTO-BATTLE ON / OFF ANYTIME"),
                ("[TAB] / [P]        :", "ARRANGE 3X2 FRONT/BACK & SWAP GEMS/ARTIFACTS"),
                ("AFFINITY CYCLE     :", "VERDANT > TIDE > CONSTRUCT > ARCANE > VERDANT"),
                ("AUDIO & CRT        :", "[ ] TRACK SKIP   [M] MUTE SFX   [F2] SCANLINES"),
            ];
            for (i, (k, v)) in rows.iter().enumerate() {
                let y = 42 + (i as i32) * 23;
                fb.draw_text(k, 22, y, 34, Some(1), 1);
                fb.draw_text(v, 146, y, 7, Some(1), 1);
            }
        } else {
            // Left list of 14 species (y = 38..264)
            for i in 0..NUM_SPECIES {
                let sp = SpeciesId::ALL[i];
                let unlocked = self.bestiary.unlocked[i];
                let sel = self.bestiary_cursor == i;
                let y = 38 + (i as i32) * 16;
                if sel {
                    fb.rect_fill(18, y - 1, 152, 12, 18);
                }
                let label = if unlocked {
                    format!("{:02}. {}", i + 1, sp.meta().name)
                } else {
                    format!("{:02}. ??? UNDISCOVERED", i + 1)
                };
                let trim: String = label.chars().take(24).collect();
                fb.draw_text(
                    &trim,
                    22,
                    y + 1,
                    if sel {
                        15
                    } else if unlocked {
                        7
                    } else {
                        5
                    },
                    Some(1),
                    1,
                );
            }

            // Right Detail Card (x = 176..378, y = 38..262)
            let sp = SpeciesId::ALL[self.bestiary_cursor];
            let m = sp.meta();
            let unlocked = self.bestiary.unlocked[self.bestiary_cursor];
            fb.rect_bevel(176, 38, 202, 224, 16, 4, 1, 1);

            fb.rect_bevel(253, 82, 48, 8, 3, 6, 1, 1);
            let flash = if unlocked { None } else { Some(1) };
            fb.blit_sprite(&assets.monsters[sp as usize], 265, 58, false, false, flash);

            if unlocked {
                fb.draw_text_centered(m.name, 277, 44, 15, Some(1), 1);
                fb.draw_text(
                    &format!("CLASS: {} | {}", m.affinity.label(), m.habitat_hint),
                    184,
                    98,
                    m.affinity.color(),
                    Some(1),
                    1,
                );
                fb.draw_text(
                    &format!(
                        "BASE HP:{} ATK:{} DEF:{} INT:{} SPD:{}",
                        m.base_hp, m.base_atk, m.base_def, m.base_int, m.base_spd
                    ),
                    184,
                    114,
                    7,
                    Some(1),
                    1,
                );
                fb.draw_text(
                    &format!("CAPTURED: {}", self.bestiary.captured_count[self.bestiary_cursor]),
                    184,
                    130,
                    25,
                    Some(1),
                    1,
                );
                fb.draw_text(&format!("TRAIT: {}", m.trait_name), 184, 152, 15, Some(1), 1);
                let td1: String = m.trait_desc_1.chars().take(31).collect();
                let td2: String = m.trait_desc_2.chars().take(31).collect();
                fb.draw_text(&td1, 184, 166, 34, Some(1), 1);
                fb.draw_text(&td2, 184, 180, 34, Some(1), 1);
                fb.draw_text(
                    &format!("GEM: {}", m.default_gem.meta().name),
                    184,
                    204,
                    42,
                    Some(1),
                    1,
                );
            } else {
                fb.draw_text_centered("UNDISCOVERED SPECIMEN", 277, 108, 5, Some(1), 1);
                fb.draw_text(
                    &format!("HABITAT CLUE: {}", m.habitat_hint),
                    184,
                    136,
                    34,
                    Some(1),
                    1,
                );
                fb.draw_text("CAPTURE IN THE WILD TO UNLOCK", 184, 160, 6, Some(1), 1);
                fb.draw_text("FOR FUTURE STARTER DRAFTS!", 184, 174, 6, Some(1), 1);
            }
        }

        fb.draw_text_centered(
            "LEFT/RIGHT: SWITCH TAB   UP/DOWN: BROWSE   [H/X]: RESUME",
            200,
            274,
            15,
            Some(1),
            1,
        );
    }

    fn render_end_screen(&self, fb: &mut Framebuffer, assets: &AssetBank, victory: bool) {
        fb.blit_parallax_bg(&assets.bg_canopy, 0, 0, FB_H);
        fb.rect_dither(0, 0, FB_W as i32, FB_H as i32, 1, 2);
        fb.rect_bevel(20, 24, 360, 252, 2, if victory { 15 } else { 37 }, 1, 1);

        if victory {
            fb.draw_text_centered("ACT I EXPEDITION CLEARED!", 200, 38, 15, Some(1), 2);
            fb.draw_text_centered(
                "GROF DEFEATED - NAIBIHPMA SANCTUM SECURED!",
                200,
                64,
                25,
                Some(1),
                1,
            );
        } else {
            fb.draw_text_centered("EXPEDITION OVERRUN", 200, 38, 38, Some(1), 2);
            fb.draw_text_centered(
                "ALL BESTIARY CAPTURES PERSIST FOR YOUR NEXT DRAFT!",
                200,
                64,
                34,
                Some(1),
                1,
            );
        }

        // Showcase Hall-of-Fame 3x2 Squad!
        for slot in 0..6 {
            let cx = 36 + ((slot % 3) as i32) * 112;
            let cy = 92 + ((slot / 3) as i32) * 62;
            fb.rect_bevel(cx, cy, 104, 52, 16, 4, 1, 1);
            if let Some(ref c) = self.squad[slot] {
                fb.blit_sprite(
                    &assets.monsters[c.species as usize],
                    cx + 6,
                    cy + 16,
                    false,
                    false,
                    None,
                );
                fb.draw_text(
                    &format!("{} L{}", c.species.meta().short_name, c.level),
                    cx + 34,
                    cy + 12,
                    15,
                    Some(1),
                    1,
                );
                fb.draw_text(
                    c.species.meta().affinity.short_tag(),
                    cx + 34,
                    cy + 26,
                    c.species.meta().affinity.color(),
                    Some(1),
                    1,
                );
            }
        }

        fb.draw_text_centered(
            &format!(
                "BATTLES WON: {}   BESTIARY UNLOCKED: {}/{}   CLEARS: {}",
                self.battles_won,
                self.bestiary.total_unlocked(),
                NUM_SPECIES,
                self.bestiary.runs_cleared
            ),
            200,
            228,
            7,
            Some(1),
            1,
        );
        fb.draw_text_centered(
            "PRESS [ENTER] OR [R] TO START NEW EXPEDITION DRAFT",
            200,
            250,
            15,
            Some(1),
            1,
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_all_three_zones_load_with_grounded_spawn_and_habitats() {
        let mut gs = GameState::new();
        for z in 1..=3 {
            gs.load_zone(z);
            gs.mode = GameMode::SideWorld;
            let col = ((gs.px + 10) / TS) as usize;
            let body_row = (gs.py / TS) as usize;
            let below_row = ((gs.py + 20) / TS) as usize;
            assert!(
                !gs.tiles[body_row][col].is_solid(),
                "Zone {} player spawn must never start embedded inside solid terrain",
                z
            );
            assert!(
                gs.tiles[below_row][col].is_solid(),
                "Zone {} player spawn must stand planted directly on solid stone floor",
                z
            );
            // Verify walking right from spawn moves smoothly along the flat causeway without Y-teleporting!
            let spawn_py = gs.py;
            let mut sfx = Vec::new();
            let mut walk_inp = InputState::default();
            walk_inp.right = true;
            gs.update(&walk_inp, &mut sfx);
            assert_eq!(
                gs.py, spawn_py,
                "Zone {} player must not teleport vertically on first step from spawn",
                z
            );
            gs.load_zone(z);

            // Verify every zone contains SafeStone + FernPatch + StarWater + CrystalPatch + AlphaPatch!
            let mut has_fern = false;
            let mut has_water = false;
            let mut has_crystal = false;
            let mut has_alpha = false;
            for r in 0..MAP_H {
                for c in 0..MAP_W {
                    match gs.tiles[r][c] {
                        WorldTile::FernPatch => has_fern = true,
                        WorldTile::StarWater => has_water = true,
                        WorldTile::CrystalPatch => has_crystal = true,
                        WorldTile::AlphaPatch => has_alpha = true,
                        _ => {}
                    }
                }
            }
            assert!(
                has_fern && has_water && has_crystal && has_alpha,
                "Zone {} must include all 4 Pokemon-style encounter habitats",
                z
            );

            // Every landmark (caches, med-beacon, boss guardian, exit) must sit in open/passable space directly above solid stone!
            for cache in &gs.caches {
                assert!(
                    !gs.tiles[cache.row][cache.col].is_solid(),
                    "Zone {} cache at ({},{}) must not be embedded inside solid block",
                    z,
                    cache.col,
                    cache.row
                );
                assert!(
                    gs.tiles[cache.row + 1][cache.col].is_solid(),
                    "Zone {} cache at ({},{}) must sit on solid floor",
                    z,
                    cache.col,
                    cache.row
                );
            }
            if let Some(ref b) = gs.beacon {
                assert!(
                    !gs.tiles[b.row][b.col].is_solid(),
                    "Zone {} beacon at ({},{}) must not be embedded inside solid block",
                    z,
                    b.col,
                    b.row
                );
                assert!(
                    gs.tiles[b.row + 1][b.col].is_solid(),
                    "Zone {} beacon at ({},{}) must sit on solid floor",
                    z,
                    b.col,
                    b.row
                );
            }
            if let Some(ref g) = gs.guardian {
                assert!(
                    !gs.tiles[g.row][g.col].is_solid(),
                    "Zone {} guardian at ({},{}) must not be embedded inside solid block",
                    z,
                    g.col,
                    g.row
                );
                assert!(
                    gs.tiles[g.row + 1][g.col].is_solid(),
                    "Zone {} guardian at ({},{}) must sit on solid floor",
                    z,
                    g.col,
                    g.row
                );
            }
            assert!(
                !gs.tiles[gs.exit_row][gs.exit_col].is_solid(),
                "Zone {} exit at ({},{}) must not be embedded inside solid block",
                z,
                gs.exit_col,
                gs.exit_row
            );
            assert!(
                gs.tiles[gs.exit_row + 1][gs.exit_col].is_solid(),
                "Zone {} exit at ({},{}) must sit on solid floor",
                z,
                gs.exit_col,
                gs.exit_row
            );
        }
    }

    #[test]
    fn test_safe_stone_never_triggers_random_encounters() {
        let mut gs = GameState::new();
        gs.squad[0] = Some(CreatureInstance::new(SpeciesId::DewNewt, 3, false));
        gs.load_zone(1);
        gs.mode = GameMode::SideWorld;
        gs.encounter_grace_ticks = 0;

        let mut sfx = Vec::new();
        let mut inp = InputState::default();
        inp.right = true;
        // Walk several steps on initial Safe Stone Causeway (cols 2..6)
        for _ in 0..8 {
            gs.update(&inp, &mut sfx);
        }
        assert_eq!(gs.current_player_habitat(), HabitatKind::SafeStone);
        assert_eq!(gs.encounter_meter, 0, "Safe stone causeway must keep 0% encounter meter");
        assert!(matches!(gs.mode, GameMode::SideWorld));
    }
}
