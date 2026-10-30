use std::fs;
use std::path::Path;

pub const BESTIARY_SAVE_PATH: &str = "bestiary_save.cfg";
pub const NUM_SPECIES: usize = 14;
pub const NUM_RECRUITABLE_SPECIES: usize = 12;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Affinity {
    Verdant,   // Beats Tide (+25%), weak to Arcane
    Tide,      // Beats Construct (+25%), weak to Verdant
    Construct, // Beats Arcane (+25%), weak to Tide
    Arcane,    // Beats Verdant (+25%), weak to Construct
}

impl Affinity {
    pub fn label(self) -> &'static str {
        match self {
            Affinity::Verdant => "VERDANT",
            Affinity::Tide => "TIDE",
            Affinity::Construct => "CONSTRUCT",
            Affinity::Arcane => "ARCANE",
        }
    }

    pub fn short_tag(self) -> &'static str {
        match self {
            Affinity::Verdant => "VRD",
            Affinity::Tide => "TID",
            Affinity::Construct => "CNS",
            Affinity::Arcane => "ARC",
        }
    }

    pub fn color(self) -> u8 {
        match self {
            Affinity::Verdant => 25,   // Bright Bog-Fern Green
            Affinity::Tide => 34,      // Bright Aqua
            Affinity::Construct => 41, // Brass Gold
            Affinity::Arcane => 30,    // Orchid Spore Glow
        }
    }

    /// Returns damage multiplier in percent (125 = advantage, 85 = resisted, 100 = neutral)
    pub fn damage_mult_pct(self, defender: Affinity) -> i32 {
        match (self, defender) {
            (Affinity::Verdant, Affinity::Tide)
            | (Affinity::Tide, Affinity::Construct)
            | (Affinity::Construct, Affinity::Arcane)
            | (Affinity::Arcane, Affinity::Verdant) => 125,
            (Affinity::Tide, Affinity::Verdant)
            | (Affinity::Construct, Affinity::Tide)
            | (Affinity::Arcane, Affinity::Construct)
            | (Affinity::Verdant, Affinity::Arcane) => 85,
            _ => 100,
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
#[repr(usize)]
pub enum SpeciesId {
    DewNewt = 0,
    SporeMorel = 1,
    BriarToad = 2,
    VenomViper = 3,
    VoltaicEel = 4,
    DewFrog = 5,
    StarCarp = 6,
    LanternCrab = 7,
    RuneBeetle = 8,
    BogGolem = 9,
    GloomMoth = 10,
    CrownSalamander = 11,
    BossDaot = 12,
    BossGrof = 13,
}

impl SpeciesId {
    pub const ALL: [SpeciesId; NUM_SPECIES] = [
        SpeciesId::DewNewt,
        SpeciesId::SporeMorel,
        SpeciesId::BriarToad,
        SpeciesId::VenomViper,
        SpeciesId::VoltaicEel,
        SpeciesId::DewFrog,
        SpeciesId::StarCarp,
        SpeciesId::LanternCrab,
        SpeciesId::RuneBeetle,
        SpeciesId::BogGolem,
        SpeciesId::GloomMoth,
        SpeciesId::CrownSalamander,
        SpeciesId::BossDaot,
        SpeciesId::BossGrof,
    ];

    pub const RECRUITABLE: [SpeciesId; NUM_RECRUITABLE_SPECIES] = [
        SpeciesId::DewNewt,
        SpeciesId::SporeMorel,
        SpeciesId::BriarToad,
        SpeciesId::VenomViper,
        SpeciesId::VoltaicEel,
        SpeciesId::DewFrog,
        SpeciesId::StarCarp,
        SpeciesId::LanternCrab,
        SpeciesId::RuneBeetle,
        SpeciesId::BogGolem,
        SpeciesId::GloomMoth,
        SpeciesId::CrownSalamander,
    ];

    pub fn from_index(idx: usize) -> Option<Self> {
        Self::ALL.get(idx).copied()
    }

    pub fn meta(self) -> SpeciesMeta {
        match self {
            SpeciesId::DewNewt => SpeciesMeta {
                name: "TWEN DEW-NEWT",
                short_name: "DEWNEWT",
                affinity: Affinity::Verdant,
                habitat_hint: "FERN UNDERBRUSH",
                base_hp: 40,
                base_atk: 10,
                base_def: 12,
                base_int: 15,
                base_spd: 14,
                trait_name: "SOOTHING DEW",
                trait_desc_1: "AFTER ACTING OR DEFENDING: HEAL",
                trait_desc_2: "ADJACENT ALLIES FOR 22% MAX HP.",
                default_gem: SpellGemId::MistPulse,
                is_boss: false,
            },
            SpeciesId::SporeMorel => SpeciesMeta {
                name: "SPORE-CAP MOREL",
                short_name: "MOREL",
                affinity: Affinity::Verdant,
                habitat_hint: "FERN UNDERBRUSH",
                base_hp: 36,
                base_atk: 8,
                base_def: 11,
                base_int: 18,
                base_spd: 11,
                trait_name: "MYCELIAL BLOOM",
                trait_desc_1: "WHENEVER AN ALLY IS HEALED: DEAL",
                trait_desc_2: "45% INT DMG + POISON RANDOM FOE.",
                default_gem: SpellGemId::SporeVolley,
                is_boss: false,
            },
            SpeciesId::BriarToad => SpeciesMeta {
                name: "BRIAR-THORN TOAD",
                short_name: "BR-TOAD",
                affinity: Affinity::Verdant,
                habitat_hint: "FERN UNDERBRUSH",
                base_hp: 50,
                base_atk: 13,
                base_def: 18,
                base_int: 9,
                base_spd: 8,
                trait_name: "BRAMBLE HIDE",
                trait_desc_1: "WHEN HIT: 55% DEF THORNS TO FOE.",
                trait_desc_2: "ON PROVOKE/DEFEND: HEAL SELF 18%.",
                default_gem: SpellGemId::BrambleWard,
                is_boss: false,
            },
            SpeciesId::VenomViper => SpeciesMeta {
                name: "VENOM-LASH VIPER",
                short_name: "VIPER",
                affinity: Affinity::Verdant,
                habitat_hint: "FERN / GROTTO",
                base_hp: 35,
                base_atk: 18,
                base_def: 9,
                base_int: 13,
                base_spd: 17,
                trait_name: "NEUROTOXIN FANG",
                trait_desc_1: "ATTACKS DETONATE TARGET POISON.",
                trait_desc_2: "ON FOE POISONED: GAIN +20% ATK.",
                default_gem: SpellGemId::VenomLance,
                is_boss: false,
            },
            SpeciesId::VoltaicEel => SpeciesMeta {
                name: "VOLTAIC REED-EEL",
                short_name: "REEDEEL",
                affinity: Affinity::Tide,
                habitat_hint: "STAR-POND WATER",
                base_hp: 34,
                base_atk: 12,
                base_def: 9,
                base_int: 16,
                base_spd: 19,
                trait_name: "ARC CONDUIT",
                trait_desc_1: "WHENEVER ANY ALLY CASTS A SPELL:",
                trait_desc_2: "SHOCK 2 FOES FOR 45% SPD DMG.",
                default_gem: SpellGemId::ChainArc,
                is_boss: false,
            },
            SpeciesId::DewFrog => SpeciesMeta {
                name: "DEW-WEAVER FROG",
                short_name: "DEWFROG",
                affinity: Affinity::Tide,
                habitat_hint: "STAR-POND / FERN",
                base_hp: 38,
                base_atk: 9,
                base_def: 11,
                base_int: 17,
                base_spd: 15,
                trait_name: "ECHO CHORUS",
                trait_desc_1: "1ST TIME/TURN ADJACENT ALLY CASTS:",
                trait_desc_2: "AUTO-CAST OWN SPELL GEM FREE!",
                default_gem: SpellGemId::StarlightChoir,
                is_boss: false,
            },
            SpeciesId::StarCarp => SpeciesMeta {
                name: "STAR-SCALE CARP",
                short_name: "STRCARP",
                affinity: Affinity::Tide,
                habitat_hint: "STAR-POND WATER",
                base_hp: 42,
                base_atk: 10,
                base_def: 13,
                base_int: 14,
                base_spd: 16,
                trait_name: "ASTRAL CURRENT",
                trait_desc_1: "ON CAST OR DEFEND: GRANT SHIELD",
                trait_desc_2: "& +20% SPD TO ROW-MATE ALLY.",
                default_gem: SpellGemId::MistPulse,
                is_boss: false,
            },
            SpeciesId::LanternCrab => SpeciesMeta {
                name: "LANTERN HERMIT",
                short_name: "HERMIT",
                affinity: Affinity::Tide,
                habitat_hint: "STAR-POND WATER",
                base_hp: 48,
                base_atk: 13,
                base_def: 19,
                base_int: 10,
                base_spd: 9,
                trait_name: "TIDAL BULWARK",
                trait_desc_1: "START & PROVOKE: SHIELD FRONT ROW.",
                trait_desc_2: "ON ALLY SHIELD: 45% DEF WAVE DMG.",
                default_gem: SpellGemId::BrambleWard,
                is_boss: false,
            },
            SpeciesId::RuneBeetle => SpeciesMeta {
                name: "RUNE-CARAPACE",
                short_name: "R-BEETL",
                affinity: Affinity::Construct,
                habitat_hint: "CRYSTAL GROTTO",
                base_hp: 44,
                base_atk: 15,
                base_def: 17,
                base_int: 10,
                base_spd: 11,
                trait_name: "RESONANT PLATING",
                trait_desc_1: "WHEN OTHER ALLY GAINS STAT BUFF:",
                trait_desc_2: "COPY BUFF & HIT TOP-HP FOE (DEF).",
                default_gem: SpellGemId::MonolithSlam,
                is_boss: false,
            },
            SpeciesId::BogGolem => SpeciesMeta {
                name: "BOG-IDOL GOLEM",
                short_name: "IDOLGLM",
                affinity: Affinity::Construct,
                habitat_hint: "CRYSTAL GROTTO",
                base_hp: 54,
                base_atk: 14,
                base_def: 20,
                base_int: 8,
                base_spd: 7,
                trait_name: "MONOLITH AEGIS",
                trait_desc_1: "GUARDS BACK PARTNER (50% DMG).",
                trait_desc_2: "DEFEND/PROVOKE: +25% DEF FRONT.",
                default_gem: SpellGemId::MonolithSlam,
                is_boss: false,
            },
            SpeciesId::GloomMoth => SpeciesMeta {
                name: "GLOOM-WISP MOTH",
                short_name: "GLMMOTH",
                affinity: Affinity::Arcane,
                habitat_hint: "CRYSTAL GROTTO",
                base_hp: 34,
                base_atk: 8,
                base_def: 10,
                base_int: 20,
                base_spd: 16,
                trait_name: "SOUL SIPHON",
                trait_desc_1: "WHEN FOE TAKES SPELL DMG OR DIES:",
                trait_desc_2: "+20% INT ROW-MATE & HEAL LOWEST.",
                default_gem: SpellGemId::SiphonHex,
                is_boss: false,
            },
            SpeciesId::CrownSalamander => SpeciesMeta {
                name: "CROWN SALAMANDER",
                short_name: "CRNSALM",
                affinity: Affinity::Arcane,
                habitat_hint: "ALPHA / SANCTUM",
                base_hp: 42,
                base_atk: 17,
                base_def: 12,
                base_int: 17,
                base_spd: 16,
                trait_name: "ASTRAL FOLLOW-UP",
                trait_desc_1: "BASIC ATTACKS AUTO-CAST EQUIPPED",
                trait_desc_2: "SPELL GEM! ON KILL: +20% ATK ALL.",
                default_gem: SpellGemId::AncientNova,
                is_boss: false,
            },
            SpeciesId::BossDaot => SpeciesMeta {
                name: "DAOT ENCHANTER",
                short_name: "DAOT",
                affinity: Affinity::Arcane,
                habitat_hint: "ZONE 2 GUARDIAN",
                base_hp: 82,
                base_atk: 14,
                base_def: 16,
                base_int: 22,
                base_spd: 15,
                trait_name: "ENCHANTER'S HEX",
                trait_desc_1: "TURN START & ON ALLY FALL: CASTS",
                trait_desc_2: "ARCANE VOLLEY & SHIELDS RETINUE.",
                default_gem: SpellGemId::AncientNova,
                is_boss: true,
            },
            SpeciesId::BossGrof => SpeciesMeta {
                name: "GROF SORCERER",
                short_name: "GROF",
                affinity: Affinity::Arcane,
                habitat_hint: "ACT I BOSS",
                base_hp: 115,
                base_atk: 18,
                base_def: 18,
                base_int: 25,
                base_spd: 17,
                trait_name: "DARK DOMINION",
                trait_desc_1: "WHEN FOES CAST: +20% INT & FIRES",
                trait_desc_2: "HEX LANCE! TURN START: HEAL SELF.",
                default_gem: SpellGemId::AncientNova,
                is_boss: true,
            },
        }
    }
}

#[derive(Clone, Copy)]
pub struct SpeciesMeta {
    pub name: &'static str,
    pub short_name: &'static str, // <= 7 chars for 84px card headers!
    pub affinity: Affinity,
    pub habitat_hint: &'static str,
    pub base_hp: i32,
    pub base_atk: i32,
    pub base_def: i32,
    pub base_int: i32,
    pub base_spd: i32,
    pub trait_name: &'static str,
    pub trait_desc_1: &'static str, // <= 40 chars
    pub trait_desc_2: &'static str, // <= 40 chars
    pub default_gem: SpellGemId,
    pub is_boss: bool,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
#[repr(usize)]
pub enum SpellGemId {
    MistPulse = 0,
    SporeVolley = 1,
    ChainArc = 2,
    BrambleWard = 3,
    VenomLance = 4,
    StarlightChoir = 5,
    MonolithSlam = 6,
    SiphonHex = 7,
    PrecisionDart = 8,
    AncientNova = 9,
}

#[derive(Clone, Copy)]
pub struct SpellGemMeta {
    pub name: &'static str,
    pub short_name: &'static str,
    pub bonus_hp_pct: i32,
    pub bonus_atk_pct: i32,
    pub bonus_def_pct: i32,
    pub bonus_int_pct: i32,
    pub bonus_spd_pct: i32,
    pub needs_enemy_target: bool,
    pub desc_1: &'static str, // <= 40 chars
    pub desc_2: &'static str, // <= 40 chars
}

impl SpellGemId {
    pub const ALL: [SpellGemId; 10] = [
        SpellGemId::MistPulse,
        SpellGemId::SporeVolley,
        SpellGemId::ChainArc,
        SpellGemId::BrambleWard,
        SpellGemId::VenomLance,
        SpellGemId::StarlightChoir,
        SpellGemId::MonolithSlam,
        SpellGemId::SiphonHex,
        SpellGemId::PrecisionDart,
        SpellGemId::AncientNova,
    ];

    pub fn meta(self) -> SpellGemMeta {
        match self {
            SpellGemId::MistPulse => SpellGemMeta {
                name: "MIST PULSE GEM",
                short_name: "MIST PULSE",
                bonus_hp_pct: 10,
                bonus_atk_pct: 0,
                bonus_def_pct: 15,
                bonus_int_pct: 10,
                bonus_spd_pct: 0,
                needs_enemy_target: false,
                desc_1: "HEAL SELF & ADJACENT (65% INT) [+15%DEF]",
                desc_2: "PRE-LOADED: +10% HP / +15% DEF / +10% INT",
            },
            SpellGemId::SporeVolley => SpellGemMeta {
                name: "SPORE VOLLEY GEM",
                short_name: "SPORE VLY",
                bonus_hp_pct: 0,
                bonus_atk_pct: 0,
                bonus_def_pct: 0,
                bonus_int_pct: 22,
                bonus_spd_pct: 10,
                needs_enemy_target: false,
                desc_1: "80% INT DMG + POISON TO 2 FOES [+22%INT]",
                desc_2: "PRE-LOADED: +22% INT / +10% SPD",
            },
            SpellGemId::ChainArc => SpellGemMeta {
                name: "CHAIN ARC GEM",
                short_name: "CHAIN ARC",
                bonus_hp_pct: 0,
                bonus_atk_pct: 0,
                bonus_def_pct: 0,
                bonus_int_pct: 15,
                bonus_spd_pct: 22,
                needs_enemy_target: false,
                desc_1: "65% INT SHOCK TO 3 RANDOM FOES [+22%SPD]",
                desc_2: "PRE-LOADED: +15% INT / +22% SPD",
            },
            SpellGemId::BrambleWard => SpellGemMeta {
                name: "BRAMBLE WARD GEM",
                short_name: "BRMBL WARD",
                bonus_hp_pct: 20,
                bonus_atk_pct: 0,
                bonus_def_pct: 20,
                bonus_int_pct: 0,
                bonus_spd_pct: 0,
                needs_enemy_target: false,
                desc_1: "SHIELD & +25% DEF TO OWN COLUMN [+20%HP]",
                desc_2: "PRE-LOADED: +20% MAX HP / +20% DEF",
            },
            SpellGemId::VenomLance => SpellGemMeta {
                name: "VENOM LANCE GEM",
                short_name: "VENOM LNC",
                bonus_hp_pct: 0,
                bonus_atk_pct: 18,
                bonus_def_pct: 0,
                bonus_int_pct: 18,
                bonus_spd_pct: 0,
                needs_enemy_target: true,
                desc_1: "115% INT + POISON + DETONATE 1 FOE",
                desc_2: "PRE-LOADED: +18% ATK / +18% INT",
            },
            SpellGemId::StarlightChoir => SpellGemMeta {
                name: "STAR CHOIR GEM",
                short_name: "STAR CHOIR",
                bonus_hp_pct: 10,
                bonus_atk_pct: 0,
                bonus_def_pct: 0,
                bonus_int_pct: 20,
                bonus_spd_pct: 12,
                needs_enemy_target: false,
                desc_1: "HEAL ALL ALLIES (45% INT) & +20% SPD",
                desc_2: "PRE-LOADED: +20% INT / +12% SPD / +10%HP",
            },
            SpellGemId::MonolithSlam => SpellGemMeta {
                name: "MONOLITH SLAM GEM",
                short_name: "MONO SLAM",
                bonus_hp_pct: 10,
                bonus_atk_pct: 10,
                bonus_def_pct: 25,
                bonus_int_pct: 0,
                bonus_spd_pct: 0,
                needs_enemy_target: false,
                desc_1: "105% DEF CRUSH TO ENEMY FRONT COLUMN",
                desc_2: "PRE-LOADED: +25% DEF / +10% ATK / +10%HP",
            },
            SpellGemId::SiphonHex => SpellGemMeta {
                name: "SIPHON HEX GEM",
                short_name: "SIPHON HEX",
                bonus_hp_pct: 0,
                bonus_atk_pct: 0,
                bonus_def_pct: 10,
                bonus_int_pct: 24,
                bonus_spd_pct: 0,
                needs_enemy_target: true,
                desc_1: "105% INT 1 FOE; HEAL LOWEST ALLY 60%",
                desc_2: "PRE-LOADED: +24% INT / +10% DEF",
            },
            SpellGemId::PrecisionDart => SpellGemMeta {
                name: "SURGICAL DART GEM",
                short_name: "SURG DART",
                bonus_hp_pct: 0,
                bonus_atk_pct: 15,
                bonus_def_pct: 0,
                bonus_int_pct: 15,
                bonus_spd_pct: 25,
                needs_enemy_target: true,
                desc_1: "110% SPD STRIKE - LEAVES WILD AT >=1 HP!",
                desc_2: "IDEAL FOR <=40% CAPTURE! (+25% SPD)",
            },
            SpellGemId::AncientNova => SpellGemMeta {
                name: "ANCIENT NOVA GEM",
                short_name: "ANC. NOVA",
                bonus_hp_pct: 0,
                bonus_atk_pct: 0,
                bonus_def_pct: 0,
                bonus_int_pct: 26,
                bonus_spd_pct: 12,
                needs_enemy_target: false,
                desc_1: "72% INT TO ALL FOES + GAIN +20% INT",
                desc_2: "PRE-LOADED: +26% INT / +12% SPD",
            },
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
#[repr(usize)]
pub enum ArtifactId {
    DewdropVial = 0,
    CopperDynamo = 1,
    SporeCrown = 2,
    IronbarkBuckler = 3,
    AssassinFang = 4,
    ChorusChime = 5,
    ResonantCore = 6,
    LanternPrism = 7,
    TrapperLens = 8,
    AncientSigil = 9,
}

#[derive(Clone, Copy)]
pub struct ArtifactMeta {
    pub name: &'static str,
    pub short_name: &'static str,
    pub bonus_hp_pct: i32,
    pub bonus_atk_pct: i32,
    pub bonus_def_pct: i32,
    pub bonus_int_pct: i32,
    pub bonus_spd_pct: i32,
    pub desc_1: &'static str, // <= 40 chars
    pub desc_2: &'static str, // <= 40 chars
}

impl ArtifactId {
    pub const ALL: [ArtifactId; 10] = [
        ArtifactId::DewdropVial,
        ArtifactId::CopperDynamo,
        ArtifactId::SporeCrown,
        ArtifactId::IronbarkBuckler,
        ArtifactId::AssassinFang,
        ArtifactId::ChorusChime,
        ArtifactId::ResonantCore,
        ArtifactId::LanternPrism,
        ArtifactId::TrapperLens,
        ArtifactId::AncientSigil,
    ];

    pub fn meta(self) -> ArtifactMeta {
        match self {
            ArtifactId::DewdropVial => ArtifactMeta {
                name: "DEWDROP VIAL",
                short_name: "DEW VIAL",
                bonus_hp_pct: 22,
                bonus_atk_pct: 0,
                bonus_def_pct: 14,
                bonus_int_pct: 0,
                bonus_spd_pct: 0,
                desc_1: "+22% HP, +14% DEF | TURN START:",
                desc_2: "HEAL SELF 15% MAX HP (TRIGGERS BLOOM!)",
            },
            ArtifactId::CopperDynamo => ArtifactMeta {
                name: "COPPER DYNAMO",
                short_name: "DYNAMO",
                bonus_hp_pct: 0,
                bonus_atk_pct: 0,
                bonus_def_pct: 0,
                bonus_int_pct: 14,
                bonus_spd_pct: 25,
                desc_1: "+25% SPD, +14% INT | ON CAST SPELL:",
                desc_2: "GAIN +20% SPD BUFF (TRIGGERS RESONANCE)",
            },
            ArtifactId::SporeCrown => ArtifactMeta {
                name: "SPORE CROWN",
                short_name: "SPOR CROWN",
                bonus_hp_pct: 10,
                bonus_atk_pct: 0,
                bonus_def_pct: 0,
                bonus_int_pct: 26,
                bonus_spd_pct: 0,
                desc_1: "+26% INT, +10% HP | ON CAST SPELL:",
                desc_2: "INFLICT POISON ON A RANDOM ENEMY.",
            },
            ArtifactId::IronbarkBuckler => ArtifactMeta {
                name: "IRONBARK BUCKLER",
                short_name: "IRON BCKLR",
                bonus_hp_pct: 18,
                bonus_atk_pct: 0,
                bonus_def_pct: 28,
                bonus_int_pct: 0,
                bonus_spd_pct: 0,
                desc_1: "+28% DEF, +18% HP | BATTLE START:",
                desc_2: "GAIN SHIELD EQUAL TO 35% MAX HP.",
            },
            ArtifactId::AssassinFang => ArtifactMeta {
                name: "ASSASSIN FANG",
                short_name: "ASSN FANG",
                bonus_hp_pct: 0,
                bonus_atk_pct: 28,
                bonus_def_pct: 0,
                bonus_int_pct: 0,
                bonus_spd_pct: 14,
                desc_1: "+28% ATK, +14% SPD | BASIC ATTACKS:",
                desc_2: "+35% BONUS DMG AGAINST POISONED FOES.",
            },
            ArtifactId::ChorusChime => ArtifactMeta {
                name: "CHORUS CHIME",
                short_name: "CHOR CHIME",
                bonus_hp_pct: 10,
                bonus_atk_pct: 0,
                bonus_def_pct: 10,
                bonus_int_pct: 20,
                bonus_spd_pct: 14,
                desc_1: "+20% INT, +14% SPD | ON DEFEND:",
                desc_2: "HEAL ROW-MATE ALLY FOR 25% MAX HP.",
            },
            ArtifactId::ResonantCore => ArtifactMeta {
                name: "RESONANT CORE",
                short_name: "RESO CORE",
                bonus_hp_pct: 12,
                bonus_atk_pct: 20,
                bonus_def_pct: 20,
                bonus_int_pct: 0,
                bonus_spd_pct: 0,
                desc_1: "+20% ATK/DEF, +12% HP | BATTLE START:",
                desc_2: "GAIN +20% ATK & DEF BUFF IMMEDIATELY.",
            },
            ArtifactId::LanternPrism => ArtifactMeta {
                name: "LANTERN PRISM",
                short_name: "LNTN PRISM",
                bonus_hp_pct: 18,
                bonus_atk_pct: 0,
                bonus_def_pct: 15,
                bonus_int_pct: 22,
                bonus_spd_pct: 0,
                desc_1: "+22% INT, +18% HP | ON PROVOKE:",
                desc_2: "AUTOMATICALLY CAST EQUIPPED SPELL GEM!",
            },
            ArtifactId::TrapperLens => ArtifactMeta {
                name: "TRAPPER'S LENS",
                short_name: "TRAP LENS",
                bonus_hp_pct: 10,
                bonus_atk_pct: 18,
                bonus_def_pct: 14,
                bonus_int_pct: 0,
                bonus_spd_pct: 22,
                desc_1: "+22% SPD, +18% ATK | BASIC ATTACKS",
                desc_2: "NEVER DROP WILD FOES BELOW 1 HP!",
            },
            ArtifactId::AncientSigil => ArtifactMeta {
                name: "ANCIENT SIGIL",
                short_name: "ANC. SIGIL",
                bonus_hp_pct: 18,
                bonus_atk_pct: 18,
                bonus_def_pct: 18,
                bonus_int_pct: 18,
                bonus_spd_pct: 18,
                desc_1: "+18% TO ALL STATS | ON ENEMY DEFEAT:",
                desc_2: "HEAL SELF 25% HP & GAIN +20% SPD.",
            },
        }
    }
}

#[derive(Clone, Debug)]
pub struct CreatureInstance {
    pub species: SpeciesId,
    pub level: i32,
    pub xp: i32,
    pub is_alpha: bool,
    pub cur_hp: i32,
    pub spell_gem: SpellGemId,
    pub artifact: Option<ArtifactId>,
    pub last_cmd: u8,
}

impl CreatureInstance {
    pub fn new(species: SpeciesId, level: i32, is_alpha: bool) -> Self {
        let meta = species.meta();
        let mut inst = Self {
            species,
            level: level.max(1),
            xp: 0,
            is_alpha,
            cur_hp: 1,
            spell_gem: meta.default_gem,
            artifact: None,
            last_cmd: 0,
        };
        inst.cur_hp = inst.max_hp();
        inst
    }

    fn scale_stat(&self, base: i32, per_lv_pct: i32, gem_pct: i32, art_pct: i32) -> i32 {
        let lv_mult = 100 + (self.level - 1).max(0) * per_lv_pct;
        let alpha_mult = if self.is_alpha { 116 } else { 100 };
        let gear_mult = 100 + gem_pct + art_pct;
        let raw = (base as i64 * lv_mult as i64 * alpha_mult as i64 * gear_mult as i64)
            / (100 * 100 * 100);
        (raw as i32).max(1)
    }

    pub fn max_hp(&self) -> i32 {
        let m = self.species.meta();
        let gm = self.spell_gem.meta();
        let am_pct = self.artifact.map(|a| a.meta().bonus_hp_pct).unwrap_or(0);
        self.scale_stat(m.base_hp, 18, gm.bonus_hp_pct, am_pct)
    }

    pub fn atk(&self) -> i32 {
        let m = self.species.meta();
        let gm = self.spell_gem.meta();
        let am_pct = self.artifact.map(|a| a.meta().bonus_atk_pct).unwrap_or(0);
        self.scale_stat(m.base_atk, 16, gm.bonus_atk_pct, am_pct)
    }

    pub fn def(&self) -> i32 {
        let m = self.species.meta();
        let gm = self.spell_gem.meta();
        let am_pct = self.artifact.map(|a| a.meta().bonus_def_pct).unwrap_or(0);
        self.scale_stat(m.base_def, 16, gm.bonus_def_pct, am_pct)
    }

    pub fn int_stat(&self) -> i32 {
        let m = self.species.meta();
        let gm = self.spell_gem.meta();
        let am_pct = self.artifact.map(|a| a.meta().bonus_int_pct).unwrap_or(0);
        self.scale_stat(m.base_int, 16, gm.bonus_int_pct, am_pct)
    }

    pub fn spd(&self) -> i32 {
        let m = self.species.meta();
        let gm = self.spell_gem.meta();
        let am_pct = self.artifact.map(|a| a.meta().bonus_spd_pct).unwrap_or(0);
        self.scale_stat(m.base_spd, 14, gm.bonus_spd_pct, am_pct)
    }

    pub fn xp_to_next(&self) -> i32 {
        20 + self.level * 12
    }

    /// Grants XP; returns true if leveled up at least once.
    pub fn add_xp(&mut self, amount: i32) -> bool {
        let old_max = self.max_hp();
        self.xp += amount.max(0);
        let mut leveled = false;
        while self.xp >= self.xp_to_next() && self.level < 20 {
            self.xp -= self.xp_to_next();
            self.level += 1;
            leveled = true;
        }
        if leveled {
            let gained_hp = (self.max_hp() - old_max).max(4);
            self.cur_hp = (self.cur_hp + gained_hp).min(self.max_hp());
        }
        leveled
    }

    pub fn heal_full(&mut self) {
        self.cur_hp = self.max_hp();
    }
}

/// Side-view Pokemon-style terrain habitats!
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum HabitatKind {
    SafeStone,      // 0% encounters (Ancient Causeway / Camp)
    FernUnderbrush, // Verdant & Spore encounters
    StarPond,       // Tide & Conduit encounters
    CrystalGrotto,  // Construct & Arcane encounters
    AlphaGrove,     // Guaranteed +2 Lv Alpha leader + synergy pack!
}

impl HabitatKind {
    pub fn is_encounter_zone(self) -> bool {
        !matches!(self, HabitatKind::SafeStone)
    }

    pub fn hud_name(self) -> &'static str {
        match self {
            HabitatKind::SafeStone => "STONE CAUSEWAY (SAFE - 0% WILD)",
            HabitatKind::FernUnderbrush => "FERN UNDERBRUSH [VERDANT HABITAT]",
            HabitatKind::StarPond => "STAR-POND SHALLOWS [TIDE HABITAT]",
            HabitatKind::CrystalGrotto => "CRYSTAL GROTTO [CONSTRUCT/ARCANE]",
            HabitatKind::AlphaGrove => "*ALPHA SPORE-GROVE* [+2 LV ELITE!]",
        }
    }

    pub fn encounter_pool(self, zone_num: usize) -> &'static [SpeciesId] {
        match (self, zone_num) {
            (HabitatKind::SafeStone, _) => &[],
            (HabitatKind::FernUnderbrush, 1) => &[
                SpeciesId::DewNewt,
                SpeciesId::SporeMorel,
                SpeciesId::BriarToad,
                SpeciesId::VenomViper,
            ],
            (HabitatKind::FernUnderbrush, _) => &[
                SpeciesId::SporeMorel,
                SpeciesId::BriarToad,
                SpeciesId::VenomViper,
                SpeciesId::DewFrog,
            ],
            (HabitatKind::StarPond, 1) => &[
                SpeciesId::VoltaicEel,
                SpeciesId::StarCarp,
                SpeciesId::DewFrog,
                SpeciesId::LanternCrab,
            ],
            (HabitatKind::StarPond, _) => &[
                SpeciesId::VoltaicEel,
                SpeciesId::DewFrog,
                SpeciesId::StarCarp,
                SpeciesId::LanternCrab,
            ],
            (HabitatKind::CrystalGrotto, 1) => &[
                SpeciesId::RuneBeetle,
                SpeciesId::GloomMoth,
                SpeciesId::VenomViper,
                SpeciesId::BogGolem,
            ],
            (HabitatKind::CrystalGrotto, _) => &[
                SpeciesId::RuneBeetle,
                SpeciesId::BogGolem,
                SpeciesId::GloomMoth,
                SpeciesId::CrownSalamander,
            ],
            (HabitatKind::AlphaGrove, 1) => &[
                SpeciesId::BriarToad,
                SpeciesId::DewFrog,
                SpeciesId::LanternCrab,
                SpeciesId::VenomViper,
            ],
            (HabitatKind::AlphaGrove, _) => &[
                SpeciesId::CrownSalamander,
                SpeciesId::BogGolem,
                SpeciesId::GloomMoth,
                SpeciesId::VoltaicEel,
            ],
        }
    }
}

/// Persistent Xenobiology Bestiary across runs (unlocks starter draft options without stat bloat!)
#[derive(Clone)]
pub struct Bestiary {
    pub unlocked: [bool; NUM_SPECIES],
    pub captured_count: [u32; NUM_SPECIES],
    pub runs_cleared: u32,
}

impl Bestiary {
    pub fn default_new() -> Self {
        let mut unlocked = [false; NUM_SPECIES];
        // Start with 6 core species across all 4 affinities unlocked for Run 1 Starter Drafts!
        unlocked[SpeciesId::DewNewt as usize] = true;
        unlocked[SpeciesId::SporeMorel as usize] = true;
        unlocked[SpeciesId::BriarToad as usize] = true;
        unlocked[SpeciesId::VoltaicEel as usize] = true;
        unlocked[SpeciesId::StarCarp as usize] = true;
        unlocked[SpeciesId::RuneBeetle as usize] = true;
        Self {
            unlocked,
            captured_count: [0; NUM_SPECIES],
            runs_cleared: 0,
        }
    }

    pub fn load_or_default() -> Self {
        let mut b = Self::default_new();
        if let Ok(txt) = fs::read_to_string(Path::new(BESTIARY_SAVE_PATH)) {
            for line in txt.lines() {
                if let Some(rest) = line.strip_prefix("UNLOCK:") {
                    if let Ok(idx) = rest.trim().parse::<usize>() {
                        if idx < NUM_SPECIES {
                            b.unlocked[idx] = true;
                        }
                    }
                } else if let Some(rest) = line.strip_prefix("CAP:") {
                    let mut parts = rest.split(',');
                    if let (Some(a), Some(c)) = (parts.next(), parts.next()) {
                        if let (Ok(idx), Ok(cnt)) = (a.trim().parse::<usize>(), c.trim().parse::<u32>()) {
                            if idx < NUM_SPECIES {
                                b.captured_count[idx] = cnt;
                            }
                        }
                    }
                } else if let Some(rest) = line.strip_prefix("CLEARS:") {
                    if let Ok(c) = rest.trim().parse::<u32>() {
                        b.runs_cleared = c;
                    }
                }
            }
        }
        b
    }

    pub fn save(&self) {
        let mut out = String::new();
        out.push_str(&format!("CLEARS:{}\n", self.runs_cleared));
        for i in 0..NUM_SPECIES {
            if self.unlocked[i] {
                out.push_str(&format!("UNLOCK:{}\n", i));
            }
            if self.captured_count[i] > 0 {
                out.push_str(&format!("CAP:{},{}\n", i, self.captured_count[i]));
            }
        }
        let _ = fs::write(Path::new(BESTIARY_SAVE_PATH), out);
    }

    /// Marks species unlocked & increments capture count; returns true if newly unlocked!
    pub fn record_capture(&mut self, sp: SpeciesId) -> bool {
        let idx = sp as usize;
        let was_new = !self.unlocked[idx];
        self.unlocked[idx] = true;
        self.captured_count[idx] = self.captured_count[idx].saturating_add(1);
        self.save();
        was_new
    }

    pub fn record_sighting_boss(&mut self, sp: SpeciesId) {
        let idx = sp as usize;
        if !self.unlocked[idx] {
            self.unlocked[idx] = true;
            self.save();
        }
    }

    pub fn unlocked_recruitable(&self) -> Vec<SpeciesId> {
        SpeciesId::RECRUITABLE
            .iter()
            .copied()
            .filter(|&sp| self.unlocked[sp as usize])
            .collect()
    }

    pub fn total_unlocked(&self) -> usize {
        self.unlocked.iter().filter(|&&u| u).count()
    }
}
