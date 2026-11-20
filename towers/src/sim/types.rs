use std::fmt;

pub const MAP_W: usize = 25;
pub const MAP_H: usize = 14;
pub const MAP_TILES: usize = MAP_W * MAP_H;
pub const TILE_PX: i32 = 16;
pub const PLAYFIELD_W: i32 = (MAP_W as i32) * TILE_PX; // 400
pub const PLAYFIELD_H: i32 = (MAP_H as i32) * TILE_PX; // 224
pub const PLAYFIELD_Y: i32 = 14; // Top bar is 0..14

pub const TICKS_PER_SEC: u32 = 15;
pub const ECO_CYCLE_TICKS: u32 = 45; // 3.0 seconds
pub const MAX_MATCH_TICKS: u32 = 9000; // 10 minutes at 15 FPS
pub const SUDDEN_ESCALATION_TICK: u32 = 7200; // 8 minutes
pub const CITADEL_MAX_HP: i32 = 1500;
pub const STARTING_RESOURCES: i32 = 140;
pub const RAISED_STONE_MAX_HP: i16 = 220;
pub const GRID_EMPTY: u16 = u16::MAX;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(u8)]
pub enum TeamId {
    P1 = 0,
    P2 = 1,
}

impl TeamId {
    #[inline]
    pub const fn index(self) -> usize {
        self as usize
    }

    #[inline]
    pub const fn other(self) -> Self {
        match self {
            TeamId::P1 => TeamId::P2,
            TeamId::P2 => TeamId::P1,
        }
    }

    #[inline]
    pub const fn from_index(idx: usize) -> Self {
        if idx == 0 {
            TeamId::P1
        } else {
            TeamId::P2
        }
    }

    #[inline]
    pub const fn label(self) -> &'static str {
        match self {
            TeamId::P1 => "P1",
            TeamId::P2 => "AI",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(u8)]
pub enum Faction {
    Humans = 0,
    Elves = 1,
    Orcs = 2,
    Dwarves = 3,
    Gnomes = 4,
    Leones = 5,
    Dracones = 6,
}

impl Faction {
    pub const ALL: [Faction; 7] = [
        Faction::Humans,
        Faction::Elves,
        Faction::Orcs,
        Faction::Dwarves,
        Faction::Gnomes,
        Faction::Leones,
        Faction::Dracones,
    ];

    #[inline]
    pub const fn index(self) -> usize {
        self as usize
    }

    #[inline]
    pub const fn from_index(i: usize) -> Self {
        Self::ALL[i % 7]
    }

    pub const fn name(self) -> &'static str {
        match self {
            Faction::Humans => "HUMANS",
            Faction::Elves => "ELVES",
            Faction::Orcs => "ORCS",
            Faction::Dwarves => "DWARVES",
            Faction::Gnomes => "GNOMES",
            Faction::Leones => "LEONES",
            Faction::Dracones => "DRACONES",
        }
    }

    pub const fn short_name(self) -> &'static str {
        match self {
            Faction::Humans => "HUM",
            Faction::Elves => "ELV",
            Faction::Orcs => "ORC",
            Faction::Dwarves => "DWA",
            Faction::Gnomes => "GNO",
            Faction::Leones => "LEO",
            Faction::Dracones => "DRA",
        }
    }

    pub const fn resource_name(self) -> &'static str {
        match self {
            Faction::Humans => "MEDIA",
            Faction::Elves => "TOMES",
            Faction::Orcs => "GRIMOIRES",
            Faction::Dwarves => "BOOKS",
            Faction::Gnomes => "MANUALS",
            Faction::Leones => "ROCKS",
            Faction::Dracones => "TREASURE",
        }
    }

    pub const fn resource_tag(self) -> &'static str {
        match self {
            Faction::Humans => "DAT",
            Faction::Elves => "ARC",
            Faction::Orcs => "RIT",
            Faction::Dwarves => "KNW",
            Faction::Gnomes => "TEC",
            Faction::Leones => "LIF",
            Faction::Dracones => "WLT",
        }
    }

    pub const fn energy_summary(self) -> &'static str {
        match self {
            Faction::Humans => "MECH+COMP",
            Faction::Elves => "MANA+COMP",
            Faction::Orcs => "MECH+-FRC",
            Faction::Dwarves => "MECH++FRC",
            Faction::Gnomes => "MANA+COMP",
            Faction::Leones => "+FORCE",
            Faction::Dracones => "-FORCE",
        }
    }

    pub const fn required_energies(self) -> EnergyMask {
        match self {
            Faction::Humans => EnergyMask::MECH.union(EnergyMask::COMPUTE),
            Faction::Elves => EnergyMask::MANA.union(EnergyMask::COMPUTE),
            Faction::Orcs => EnergyMask::MECH.union(EnergyMask::NEG_FORCE),
            Faction::Dwarves => EnergyMask::MECH.union(EnergyMask::POS_FORCE),
            Faction::Gnomes => EnergyMask::MANA.union(EnergyMask::COMPUTE),
            Faction::Leones => EnergyMask::POS_FORCE,
            Faction::Dracones => EnergyMask::NEG_FORCE,
        }
    }

    pub const fn tagline(self) -> &'static str {
        match self {
            Faction::Humans => "RELAY NET + SIGHTLINES + SIEGE",
            Faction::Elves => "LEY LINES + PRISM REFRACT + WISPS",
            Faction::Orcs => "BLOOD RITUAL + DEVOUR + SWARM",
            Faction::Dwarves => "KNOWLEDGE TIERS + TUNNEL BORES",
            Faction::Gnomes => "GIZMO BUFFS + WALKING TURRETS",
            Faction::Leones => "TERRAFORMING + PUSH + +FORCE",
            Faction::Dracones => "COMPOUND HOARD + FLYING DRAKES",
        }
    }
}

impl fmt::Display for Faction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.name())
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct FactionTeam {
    pub count: u8,
    pub slots: [Faction; 3],
}

impl FactionTeam {
    pub const fn one(a: Faction) -> Self {
        Self {
            count: 1,
            slots: [a, a, a],
        }
    }

    pub const fn two(a: Faction, b: Faction) -> Self {
        Self {
            count: 2,
            slots: [a, b, a],
        }
    }

    pub const fn three(a: Faction, b: Faction, c: Faction) -> Self {
        Self {
            count: 3,
            slots: [a, b, c],
        }
    }

    pub fn from_slice(list: &[Faction]) -> Self {
        let mut out = [Faction::Humans; 3];
        let mut count = 0usize;
        for &f in list {
            if count < 3 && !out[..count].contains(&f) {
                out[count] = f;
                count += 1;
            }
        }
        if count == 0 {
            count = 1;
        }
        Self {
            count: count as u8,
            slots: out,
        }
    }

    #[inline]
    pub fn as_slice(&self) -> &[Faction] {
        &self.slots[..(self.count as usize).clamp(1, 3)]
    }

    #[inline]
    pub fn has(&self, f: Faction) -> bool {
        self.as_slice().contains(&f)
    }

    #[inline]
    pub fn primary(&self) -> Faction {
        self.slots[0]
    }

    pub fn required_energies(&self) -> EnergyMask {
        let mut mask = EnergyMask::NONE;
        for &f in self.as_slice() {
            mask = mask.union(f.required_energies());
        }
        mask
    }

    /// Cost multiplier in percent: Pure=98%, Dual=100%, Triad=102%
    pub const fn cost_pct(&self) -> i32 {
        match self.count {
            1 => 98,
            2 => 100,
            _ => 102,
        }
    }

    /// Passive Citadel income per economy cycle
    pub const fn citadel_income(&self) -> i32 {
        14
    }

    pub fn resource_label(&self) -> &'static str {
        if self.count == 1 {
            self.slots[0].resource_name()
        } else {
            "STOCK"
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct EnergyMask(pub u8);

impl EnergyMask {
    pub const NONE: Self = Self(0);
    pub const MECH: Self = Self(1 << 0);
    pub const COMPUTE: Self = Self(1 << 1);
    pub const MANA: Self = Self(1 << 2);
    pub const POS_FORCE: Self = Self(1 << 3);
    pub const NEG_FORCE: Self = Self(1 << 4);

    #[inline]
    pub const fn union(self, other: Self) -> Self {
        Self(self.0 | other.0)
    }

    #[inline]
    pub const fn contains(self, other: Self) -> bool {
        (self.0 & other.0) == other.0
    }

    #[inline]
    pub const fn has_any(self, other: Self) -> bool {
        (self.0 & other.0) != 0
    }

    pub fn short_label(self) -> &'static str {
        match self.0 {
            0 => "-",
            1 => "M",
            2 => "C",
            3 => "M+C",
            4 => "N",
            6 => "N+C",
            8 => "+F",
            9 => "M+F",
            16 => "-F",
            17 => "M-F",
            _ => "MIX",
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct EnergyCell {
    pub mech: i16,
    pub compute: i16,
    pub mana: i16,
    pub pos_force: i16,
    pub neg_force: i16,
}

impl EnergyCell {
    #[inline]
    pub fn add_mask(&mut self, mask: EnergyMask, amount: i16) {
        if mask.contains(EnergyMask::MECH) {
            self.mech += amount;
        }
        if mask.contains(EnergyMask::COMPUTE) {
            self.compute += amount;
        }
        if mask.contains(EnergyMask::MANA) {
            self.mana += amount;
        }
        if mask.contains(EnergyMask::POS_FORCE) {
            self.pos_force += amount;
        }
        if mask.contains(EnergyMask::NEG_FORCE) {
            self.neg_force += amount;
        }
    }

    #[inline]
    pub fn satisfies(&self, req: EnergyMask) -> bool {
        if req.contains(EnergyMask::MECH) && self.mech <= 0 {
            return false;
        }
        if req.contains(EnergyMask::COMPUTE) && self.compute <= 0 {
            return false;
        }
        if req.contains(EnergyMask::MANA) && self.mana <= 0 {
            return false;
        }
        if req.contains(EnergyMask::POS_FORCE) && self.pos_force <= 0 {
            return false;
        }
        if req.contains(EnergyMask::NEG_FORCE) && self.neg_force <= 0 {
            return false;
        }
        true
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum Terrain {
    Open = 0,
    HighGround = 1,
    OreVein = 2,
    LeySpring = 3,
    Chasm = 4,
    RaisedStone = 5,
}

impl Terrain {
    #[inline]
    pub const fn blocks_ground(self) -> bool {
        matches!(self, Terrain::Chasm | Terrain::RaisedStone)
    }

    #[inline]
    pub const fn blocks_line(self) -> bool {
        matches!(self, Terrain::RaisedStone)
    }

    #[inline]
    pub const fn is_high(self) -> bool {
        matches!(self, Terrain::HighGround | Terrain::RaisedStone)
    }

    pub const fn label(self) -> &'static str {
        match self {
            Terrain::Open => "PLAIN",
            Terrain::HighGround => "HIGH GROUND",
            Terrain::OreVein => "ORE VEIN",
            Terrain::LeySpring => "LEY SPRING",
            Terrain::Chasm => "CHASM",
            Terrain::RaisedStone => "RAISED STONE",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
#[repr(u8)]
pub enum BuildingSlot {
    Infra = 0,
    Eco = 1,
    Defense = 2,
    Factory = 3,
    Apex = 4,
}

impl BuildingSlot {
    pub const ALL: [BuildingSlot; 5] = [
        BuildingSlot::Infra,
        BuildingSlot::Eco,
        BuildingSlot::Defense,
        BuildingSlot::Factory,
        BuildingSlot::Apex,
    ];
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(u8)]
pub enum BuildingType {
    Citadel = 0,
    // Humans
    HumanRelay = 1,
    HumanMediaArray = 2,
    HumanAutocannon = 3,
    HumanMechFoundry = 4,
    HumanOrbitalUplink = 5,
    // Elves
    ElvenLeyConduit = 6,
    ElvenScriptorium = 7,
    ElvenPrismSpire = 8,
    ElvenWispSanctum = 9,
    ElvenAstralLens = 10,
    // Orcs
    OrcRiftFurnace = 11,
    OrcBloodAltar = 12,
    OrcSpikeBastion = 13,
    OrcWarDen = 14,
    OrcDoomCauldron = 15,
    // Dwarves
    DwarfRunicHearth = 16,
    DwarfRuneArchive = 17,
    DwarfBastionCannon = 18,
    DwarfTunnelBore = 19,
    DwarfHowitzer = 20,
    // Gnomes
    GnomeAetherCog = 21,
    GnomeTinkerLab = 22,
    GnomeWalkerTurret = 23,
    GnomeClockworkHive = 24,
    GnomeTeslaContraption = 25,
    // Leones
    LeoneSunMonolith = 26,
    LeoneLithoShaper = 27,
    LeoneSolarPillar = 28,
    LeoneTectonicRam = 29,
    LeoneSolsticeColossus = 30,
    // Dracones
    DraconeVoidBrazier = 31,
    DraconeHoardVault = 32,
    DraconeTaxObelisk = 33,
    DraconeWyrmRoost = 34,
    DraconeElderSanctum = 35,
}

impl BuildingType {
    pub const COUNT: usize = 36;

    #[inline]
    pub const fn index(self) -> usize {
        self as usize
    }

    pub const fn from_faction_slot(faction: Faction, slot: BuildingSlot) -> Self {
        let idx = 1 + (faction as usize) * 5 + (slot as usize);
        Self::ALL[idx]
    }

    pub const fn faction(self) -> Option<Faction> {
        let i = self as usize;
        if i == 0 {
            None
        } else {
            Some(Faction::ALL[(i - 1) / 5])
        }
    }

    pub const fn slot(self) -> Option<BuildingSlot> {
        let i = self as usize;
        if i == 0 {
            None
        } else {
            Some(BuildingSlot::ALL[(i - 1) % 5])
        }
    }

    pub const fn is_walker(self) -> bool {
        matches!(
            self,
            BuildingType::GnomeWalkerTurret | BuildingType::GnomeTeslaContraption
        )
    }

    pub const ALL: [BuildingType; 36] = [
        BuildingType::Citadel,
        BuildingType::HumanRelay,
        BuildingType::HumanMediaArray,
        BuildingType::HumanAutocannon,
        BuildingType::HumanMechFoundry,
        BuildingType::HumanOrbitalUplink,
        BuildingType::ElvenLeyConduit,
        BuildingType::ElvenScriptorium,
        BuildingType::ElvenPrismSpire,
        BuildingType::ElvenWispSanctum,
        BuildingType::ElvenAstralLens,
        BuildingType::OrcRiftFurnace,
        BuildingType::OrcBloodAltar,
        BuildingType::OrcSpikeBastion,
        BuildingType::OrcWarDen,
        BuildingType::OrcDoomCauldron,
        BuildingType::DwarfRunicHearth,
        BuildingType::DwarfRuneArchive,
        BuildingType::DwarfBastionCannon,
        BuildingType::DwarfTunnelBore,
        BuildingType::DwarfHowitzer,
        BuildingType::GnomeAetherCog,
        BuildingType::GnomeTinkerLab,
        BuildingType::GnomeWalkerTurret,
        BuildingType::GnomeClockworkHive,
        BuildingType::GnomeTeslaContraption,
        BuildingType::LeoneSunMonolith,
        BuildingType::LeoneLithoShaper,
        BuildingType::LeoneSolarPillar,
        BuildingType::LeoneTectonicRam,
        BuildingType::LeoneSolsticeColossus,
        BuildingType::DraconeVoidBrazier,
        BuildingType::DraconeHoardVault,
        BuildingType::DraconeTaxObelisk,
        BuildingType::DraconeWyrmRoost,
        BuildingType::DraconeElderSanctum,
    ];
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum UnitKind {
    StriderTank = 0,
    ArcaneWisp = 1,
    OrcRaider = 2,
    OrcRam = 3,
    IroncladGolem = 4,
    TinkerBot = 5,
    LivingBoulder = 6,
    StoneGolem = 7,
    SkyDrake = 8,
    ElderWyrm = 9,
}

impl UnitKind {
    pub const COUNT: usize = 10;

    #[inline]
    pub const fn index(self) -> usize {
        self as usize
    }

    #[inline]
    pub const fn is_flying(self) -> bool {
        matches!(
            self,
            UnitKind::ArcaneWisp | UnitKind::SkyDrake | UnitKind::ElderWyrm
        )
    }

    pub const fn name(self) -> &'static str {
        match self {
            UnitKind::StriderTank => "STRIDER TANK",
            UnitKind::ArcaneWisp => "ARCANE WISP",
            UnitKind::OrcRaider => "ORC RAIDER",
            UnitKind::OrcRam => "SIEGE RAM",
            UnitKind::IroncladGolem => "IRONCLAD",
            UnitKind::TinkerBot => "TINKER BOT",
            UnitKind::LivingBoulder => "BOULDER",
            UnitKind::StoneGolem => "STONE GOLEM",
            UnitKind::SkyDrake => "SKY DRAKE",
            UnitKind::ElderWyrm => "ELDER WYRM",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum DamageClass {
    Kinetic = 0,
    Arcane = 1,
    Fire = 2,
    Siege = 3,
    Flux = 4,
    Void = 5,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum ArmorClass {
    Light = 0,
    Armored = 1,
    Fortified = 2,
    Flying = 3,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Building {
    pub id: u32,
    pub owner: TeamId,
    pub kind: BuildingType,
    pub x: u8,
    pub y: u8,
    pub hp: i32,
    pub max_hp: i32,
    pub armor: i32,
    pub level: u8,
    pub build_ticks_left: u16,
    pub cooldown: u16,
    pub spawn_timer: u16,
    pub spawn_count: u16,
    pub powered: bool,
    pub net_links: u8,
    pub gizmo_buff: bool,
    pub hearth_armor: u8,
    pub fury_stacks: u8,
    pub slow_ticks: u16,
    pub relocate_target: Option<(u8, u8)>,
    pub relocate_timer: u8,
    pub ability_cd: u16,
    pub last_fire_tick: u32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Unit {
    pub id: u32,
    pub owner: TeamId,
    pub kind: UnitKind,
    pub px: i32,
    pub py: i32,
    pub hp: i32,
    pub max_hp: i32,
    pub cooldown: u16,
    pub slow_ticks: u16,
    pub corrode_ticks: u16,
    pub dir_dx: i8,
    pub dir_dy: i8,
    pub hit_flash: u8,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SimProjectile {
    pub owner: TeamId,
    pub source_kind: BuildingType,
    pub dmg: i32,
    pub dmg_class: DamageClass,
    pub splash_px: i32,
    pub siphon: i16,
    pub px: i32,
    pub py: i32,
    pub tx: i32,
    pub ty: i32,
    pub ticks_left: u8,
    pub total_ticks: u8,
    pub color: u8,
    pub arcing: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct LeyLink {
    pub owner: TeamId,
    pub x0: u8,
    pub y0: u8,
    pub x1: u8,
    pub y1: u8,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TeamState {
    pub config: FactionTeam,
    pub resources: i32,
    pub total_earned: i32,
    pub total_spent: i32,
    pub knowledge_xp: i32,
    pub knowledge_tier: u8, // 0..5 for Dwarves
    pub last_cycle_income: i32,
    pub kills: u16,
    pub buildings_built: u16,
    pub buildings_lost: u16,
}

impl TeamState {
    pub fn new(config: FactionTeam, starting_resources: i32) -> Self {
        Self {
            config,
            resources: starting_resources,
            total_earned: starting_resources,
            total_spent: 0,
            knowledge_xp: 0,
            knowledge_tier: 0,
            last_cycle_income: config.citadel_income(),
            kills: 0,
            buildings_built: 0,
            buildings_lost: 0,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Action {
    NoOp,
    Build {
        kind: BuildingType,
        x: u8,
        y: u8,
    },
    Upgrade {
        x: u8,
        y: u8,
    },
    Salvage {
        x: u8,
        y: u8,
    },
    Devour {
        cauldron_x: u8,
        cauldron_y: u8,
        victim_x: u8,
        victim_y: u8,
    },
    Relocate {
        x: u8,
        y: u8,
        tx: u8,
        ty: u8,
    },
    ShapeTerrain {
        shaper_x: u8,
        shaper_y: u8,
        tx: u8,
        ty: u8,
    },
    TectonicPush {
        ram_x: u8,
        ram_y: u8,
        target_x: u8,
        target_y: u8,
    },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MatchOutcome {
    Ongoing,
    Victory(TeamId),
    Draw,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ChallengeGoal {
    DestroyEnemyCitadel,
    DestroyAllEnemyStructures,
    SurviveTicks(u32),
    ReachResources(i32),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct FxBeam {
    pub x0: i16,
    pub y0: i16,
    pub x1: i16,
    pub y1: i16,
    pub color: u8,
    pub ttl: u8,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct FxBurst {
    pub px: i16,
    pub py: i16,
    pub radius: u8,
    pub color: u8,
    pub ttl: u8,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct FxPopup {
    pub px: i16,
    pub py: i16,
    pub text: [u8; 8],
    pub len: u8,
    pub color: u8,
    pub ttl: u8,
}

#[inline]
pub const fn tile_idx(x: usize, y: usize) -> usize {
    y * MAP_W + x
}

#[inline]
pub const fn chebyshev(x0: u8, y0: u8, x1: u8, y1: u8) -> u8 {
    let dx = if x0 > x1 { x0 - x1 } else { x1 - x0 };
    let dy = if y0 > y1 { y0 - y1 } else { y1 - y0 };
    if dx > dy {
        dx
    } else {
        dy
    }
}

#[inline]
pub const fn manhattan(x0: u8, y0: u8, x1: u8, y1: u8) -> u8 {
    let dx = if x0 > x1 { x0 - x1 } else { x1 - x0 };
    let dy = if y0 > y1 { y0 - y1 } else { y1 - y0 };
    dx + dy
}
