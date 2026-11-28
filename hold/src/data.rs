//! Every number from design documents 01 to 05 (`hold/design/`), in one place. The rules in
//! `world` read these tables and nothing else, so tuning is editing this file.
//!
//! One tick is one 15 FPS frame. A map square is 16 px. A creature walks a square in 10 ticks.

pub const TPS: u32 = 15;
pub const DAY: u32 = 5400;
pub const DUSK: u32 = 2700;
pub const NOON: u32 = 1350;
pub const WALK: i32 = 10;
pub const MINUTE: u32 = 60 * TPS;

// ---- items ---------------------------------------------------------------------------------

#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash, PartialOrd, Ord)]
pub enum Item {
    Dew,
    Fungiwood,
    CapIngot,
    SpottedCap,
    GlowSpores,
    SunOrb,
    ClargentOrb,
    MeteorOrb,
    WoodenShovel,
    IronShovel,
    VenomwoodShovel,
    Slingshot,
    Catapult,
    IronSpear,
    VenomwoodSpear,
    VenomironSpear,
    Chanfron,
    Peytral,
    Croupiere,
    VenomironChanfron,
    DewBead,
    GlowCharm,
    StarPearl,
}

pub const ITEMS: [Item; 23] = [
    Item::Dew,
    Item::Fungiwood,
    Item::CapIngot,
    Item::SpottedCap,
    Item::GlowSpores,
    Item::SunOrb,
    Item::ClargentOrb,
    Item::MeteorOrb,
    Item::WoodenShovel,
    Item::IronShovel,
    Item::VenomwoodShovel,
    Item::Slingshot,
    Item::Catapult,
    Item::IronSpear,
    Item::VenomwoodSpear,
    Item::VenomironSpear,
    Item::Chanfron,
    Item::Peytral,
    Item::Croupiere,
    Item::VenomironChanfron,
    Item::DewBead,
    Item::GlowCharm,
    Item::StarPearl,
];

#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash)]
pub enum Slot {
    Shovel,
    Shovel2,
    Spear,
    Sling,
    Head,
    Chest,
    Rump,
    Bauble,
}

impl Slot {
    /// The kind of item a slot takes (a second shovel slot takes shovels).
    pub fn takes(self) -> Slot {
        if self == Slot::Shovel2 {
            Slot::Shovel
        } else {
            self
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash)]
pub enum WSkill {
    Shovels,
    Spears,
    Slingshots,
}

pub struct ItemInfo {
    pub name: &'static str,
    pub slot: Option<Slot>,
    pub lvl: i32,
    pub inputs: &'static [(Item, i32)],
    pub craft: Option<Skill>,
    pub craft_lvl: i32,
    pub time: i32,
    pub price: i32,
    /// Weapon dice (count, sides), range in squares, and the skill it rolls on.
    pub dmg: (i32, i32),
    pub range: i32,
    pub weapon: Option<WSkill>,
    pub mend: i32,
    /// Poisoned stacks a hit, and the most it can build to.
    pub poison: (i32, i32),
    pub def: i32,
    pub dodge: i32,
    pub res: i32,
    pub energy: i32,
    pub healing: i32,
    pub dry_half: bool,
    pub orb: bool,
}

const NO: ItemInfo = ItemInfo {
    name: "",
    slot: None,
    lvl: 0,
    inputs: &[],
    craft: None,
    craft_lvl: 0,
    time: 150,
    price: 0,
    dmg: (0, 0),
    range: 0,
    weapon: None,
    mend: 0,
    poison: (0, 0),
    def: 0,
    dodge: 0,
    res: 0,
    energy: 0,
    healing: 0,
    dry_half: false,
    orb: false,
};

pub fn item(i: Item) -> &'static ItemInfo {
    use Item::*;
    use Skill::{Enchanting, Ironcrafting, Woodcrafting};
    match i {
        Dew => &ItemInfo { name: "DEW", ..NO },
        Fungiwood => &ItemInfo { name: "FUNGIWOOD", ..NO },
        CapIngot => &ItemInfo { name: "CAP INGOT", ..NO },
        SpottedCap => &ItemInfo { name: "SPOTTED CAP", ..NO },
        GlowSpores => &ItemInfo { name: "GLOW SPORES", ..NO },
        SunOrb => &ItemInfo { name: "SUN ORB", orb: true, ..NO },
        ClargentOrb => &ItemInfo { name: "CLARGENT ORB", orb: true, ..NO },
        MeteorOrb => &ItemInfo { name: "METEOR ORB", orb: true, ..NO },
        WoodenShovel => &ItemInfo {
            name: "WOODEN SHOVEL",
            slot: Some(Slot::Shovel),
            lvl: 1,
            inputs: &[(Fungiwood, 1)],
            craft: Some(Woodcrafting),
            price: 6,
            dmg: (2, 2),
            range: 1,
            weapon: Some(WSkill::Shovels),
            mend: 2,
            ..NO
        },
        IronShovel => &ItemInfo {
            name: "IRON SHOVEL",
            slot: Some(Slot::Shovel),
            lvl: 2,
            inputs: &[(Fungiwood, 1), (CapIngot, 1)],
            craft: Some(Ironcrafting),
            price: 15,
            dmg: (3, 4),
            range: 1,
            weapon: Some(WSkill::Shovels),
            mend: 3,
            ..NO
        },
        VenomwoodShovel => &ItemInfo {
            name: "VENOMWOOD SHOVEL",
            slot: Some(Slot::Shovel),
            lvl: 3,
            inputs: &[(Fungiwood, 1), (SpottedCap, 1)],
            craft: Some(Woodcrafting),
            craft_lvl: 2,
            price: 30,
            dmg: (3, 4),
            range: 1,
            weapon: Some(WSkill::Shovels),
            mend: 4,
            poison: (1, 8),
            ..NO
        },
        Slingshot => &ItemInfo {
            name: "SLINGSHOT",
            slot: Some(Slot::Sling),
            lvl: 1,
            inputs: &[(Fungiwood, 1), (CapIngot, 1)],
            craft: Some(Woodcrafting),
            price: 10,
            dmg: (1, 6),
            range: 4,
            weapon: Some(WSkill::Slingshots),
            ..NO
        },
        Catapult => &ItemInfo {
            name: "VENOMIRON CATAPULT",
            slot: Some(Slot::Sling),
            lvl: 2,
            inputs: &[(Fungiwood, 1), (CapIngot, 1), (SpottedCap, 1)],
            craft: Some(Woodcrafting),
            craft_lvl: 2,
            price: 40,
            dmg: (2, 6),
            range: 4,
            weapon: Some(WSkill::Slingshots),
            poison: (2, 10),
            ..NO
        },
        IronSpear => &ItemInfo {
            name: "IRON SPEAR",
            slot: Some(Slot::Spear),
            lvl: 2,
            inputs: &[(Fungiwood, 1), (CapIngot, 1)],
            craft: Some(Ironcrafting),
            craft_lvl: 2,
            price: 25,
            dmg: (3, 6),
            range: 1,
            weapon: Some(WSkill::Spears),
            ..NO
        },
        VenomwoodSpear => &ItemInfo {
            name: "VENOMWOOD SPEAR",
            slot: Some(Slot::Spear),
            lvl: 3,
            inputs: &[(Fungiwood, 1), (SpottedCap, 1)],
            craft: Some(Woodcrafting),
            craft_lvl: 3,
            price: 45,
            dmg: (3, 6),
            range: 1,
            weapon: Some(WSkill::Spears),
            poison: (1, 8),
            ..NO
        },
        VenomironSpear => &ItemInfo {
            name: "VENOMIRON SPEAR",
            slot: Some(Slot::Spear),
            lvl: 4,
            inputs: &[(Fungiwood, 1), (CapIngot, 1), (SpottedCap, 1), (MeteorOrb, 1)],
            craft: Some(Ironcrafting),
            craft_lvl: 5,
            price: 70,
            dmg: (4, 6),
            range: 1,
            weapon: Some(WSkill::Spears),
            poison: (2, 10),
            ..NO
        },
        Chanfron => &ItemInfo {
            name: "CHANFRON",
            slot: Some(Slot::Head),
            lvl: 1,
            inputs: &[(CapIngot, 1)],
            craft: Some(Ironcrafting),
            craft_lvl: 3,
            time: 1000,
            price: 20,
            def: 4,
            dodge: 4,
            ..NO
        },
        Peytral => &ItemInfo {
            name: "PEYTRAL",
            slot: Some(Slot::Chest),
            lvl: 1,
            inputs: &[(CapIngot, 2)],
            craft: Some(Ironcrafting),
            craft_lvl: 3,
            time: 1000,
            price: 30,
            def: 4,
            dodge: 4,
            ..NO
        },
        Croupiere => &ItemInfo {
            name: "CROUPIERE",
            slot: Some(Slot::Rump),
            lvl: 1,
            inputs: &[(CapIngot, 2)],
            craft: Some(Ironcrafting),
            craft_lvl: 3,
            time: 1000,
            price: 30,
            def: 4,
            dodge: 4,
            ..NO
        },
        VenomironChanfron => &ItemInfo {
            name: "VENOMIRON CHANFRON",
            slot: Some(Slot::Head),
            lvl: 2,
            inputs: &[(CapIngot, 1), (SpottedCap, 1)],
            craft: Some(Ironcrafting),
            craft_lvl: 4,
            time: 1000,
            price: 60,
            def: 4,
            dodge: 4,
            res: 4,
            ..NO
        },
        DewBead => &ItemInfo {
            name: "DEW BEAD",
            slot: Some(Slot::Bauble),
            lvl: 1,
            inputs: &[(Fungiwood, 1), (Dew, 2)],
            craft: Some(Woodcrafting),
            craft_lvl: 1,
            price: 12,
            dry_half: true,
            ..NO
        },
        GlowCharm => &ItemInfo {
            name: "GLOW CHARM",
            slot: Some(Slot::Bauble),
            lvl: 2,
            inputs: &[(Fungiwood, 1), (GlowSpores, 1), (Dew, 2)],
            craft: Some(Woodcrafting),
            craft_lvl: 3,
            price: 30,
            dry_half: true,
            energy: 2,
            ..NO
        },
        StarPearl => &ItemInfo {
            name: "STAR PEARL",
            slot: Some(Slot::Bauble),
            lvl: 3,
            inputs: &[(GlowSpores, 2), (MeteorOrb, 1)],
            craft: Some(Enchanting),
            craft_lvl: 8,
            time: 300,
            price: 50,
            dry_half: true,
            energy: 3,
            healing: 3,
            res: 4,
            ..NO
        },
    }
}

// ---- skills, conditions, abilities ----------------------------------------------------------

#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash, PartialOrd, Ord)]
pub enum Skill {
    Gathering,
    Fungiculture,
    Woodcrafting,
    Ironcrafting,
    Enchanting,
    Biting,
    Shovels,
    Spears,
    Slingshots,
    Energy,
    Healing,
    Motivation,
    Alteration,
    Defense,
    Dodge,
    Resistance,
    Repair,
    Demolition,
}

pub const STR: usize = 0;
pub const DEX: usize = 1;
pub const CON: usize = 2;
pub const INT: usize = 3;
pub const WIS: usize = 4;
pub const CHA: usize = 5;

/// The attribute each skill rolls on (RockCollector's skills files, and document 02).
pub fn key_attr(s: Skill) -> usize {
    use Skill::*;
    match s {
        Gathering | Woodcrafting | Slingshots | Dodge => DEX,
        Fungiculture | Healing | Resistance => WIS,
        Ironcrafting | Enchanting | Energy | Alteration | Repair => INT,
        Biting | Shovels | Spears | Demolition => STR,
        Defense => CON,
        Motivation => CHA,
    }
}

pub fn weapon_skill(w: WSkill) -> Skill {
    match w {
        WSkill::Shovels => Skill::Shovels,
        WSkill::Spears => Skill::Spears,
        WSkill::Slingshots => Skill::Slingshots,
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash)]
pub enum Cond {
    Bolstered,
    Poisoned,
    Blinded,
    Regenerating,
    Altered,
    Dry,
    Dreaming,
    Leeched,
    Frightened,
}

pub struct CondInfo {
    pub name: &'static str,
    /// Ticks for a stack to fade (0: fades only by its own rule).
    pub duration: i32,
    pub attrs: [i32; 6],
    pub def: i32,
    pub dodge: i32,
    /// Healing (negative) or harm every `every` ticks, per stack.
    pub every: i32,
    pub dmg: i32,
    /// Tenths: -1 is ten percent slower actions per stack.
    pub speed: i32,
    pub move_ticks: i32,
    pub harmful: bool,
    pub guard: Option<usize>,
}

pub fn cond(c: Cond) -> &'static CondInfo {
    const Z: CondInfo = CondInfo {
        name: "",
        duration: 400,
        attrs: [0; 6],
        def: 0,
        dodge: 0,
        every: 0,
        dmg: 0,
        speed: 0,
        move_ticks: 0,
        harmful: false,
        guard: None,
    };
    match c {
        Cond::Bolstered => &CondInfo { name: "BOLSTERED", duration: 2000, attrs: [4, 0, 4, 0, 0, 2], ..Z },
        Cond::Poisoned => &CondInfo {
            name: "POISONED",
            duration: 400,
            attrs: [0, 0, -1, 0, 0, 0],
            harmful: true,
            guard: Some(CON),
            ..Z
        },
        Cond::Blinded => &CondInfo { name: "BLINDED", duration: 300, def: -1, dodge: -1, harmful: true, ..Z },
        Cond::Regenerating => &CondInfo { name: "REGROWING", duration: 500, every: 50, dmg: -1, ..Z },
        Cond::Altered => &CondInfo {
            name: "ALTERED",
            duration: 1000,
            attrs: [4, 4, 4, 4, 0, 0],
            speed: -1,
            move_ticks: -2,
            ..Z
        },
        Cond::Dry => &CondInfo { name: "DRY", duration: 0, attrs: [0, 0, -1, 0, 0, 0], move_ticks: 1, ..Z },
        Cond::Dreaming => &CondInfo { name: "DREAMING", duration: 1000, ..Z },
        Cond::Leeched => &CondInfo { name: "LEECHED", duration: 0, every: 300, dmg: 1, ..Z },
        Cond::Frightened => &CondInfo { name: "FRIGHTENED", duration: 200, harmful: true, guard: Some(WIS), ..Z },
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash)]
pub enum Ability {
    Gathering,
    Fungiculture,
    Bite,
    ShovelBlow,
    SpearSling,
    DoubleSmash,
    Smash,
    Mending,
    Digging,
    HeavyRepair,
    Woodcrafting,
    Ironcrafting,
    Enchanting,
    Call,
    Lightning,
    PondLight,
    Regrowth,
    Alter,
    Beak,
    SnakeBite,
    Peck,
    Jaws,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum AKind {
    Attack,
    Ally,
    Work,
}

pub struct AbilityInfo {
    pub name: &'static str,
    pub kind: AKind,
    pub cooldown: i32,
    pub recharge: i32,
    pub range: i32,
    pub targets: i32,
    pub radius: i32,
    pub dmg: (i32, i32),
    pub skill: Skill,
    pub cond: Option<(Cond, i32, i32)>,
    pub weapons: &'static [WSkill],
}

pub fn ability(a: Ability) -> &'static AbilityInfo {
    use Ability::*;
    const W: AbilityInfo = AbilityInfo {
        name: "",
        kind: AKind::Work,
        cooldown: 0,
        recharge: 0,
        range: 1,
        targets: 1,
        radius: 0,
        dmg: (0, 0),
        skill: Skill::Repair,
        cond: None,
        weapons: &[],
    };
    match a {
        Gathering => &AbilityInfo { name: "GATHERING", cooldown: 60, skill: Skill::Gathering, ..W },
        Fungiculture => &AbilityInfo { name: "FUNGICULTURE", cooldown: 100, skill: Skill::Fungiculture, ..W },
        Mending => &AbilityInfo { name: "MENDING", cooldown: 20, ..W },
        Digging => &AbilityInfo { name: "DIGGING", cooldown: 10, weapons: &[WSkill::Shovels], ..W },
        HeavyRepair => &AbilityInfo { name: "HEAVY REPAIR", cooldown: 3, weapons: &[WSkill::Shovels], ..W },
        Woodcrafting => &AbilityInfo { name: "WOODCRAFTING", skill: Skill::Woodcrafting, ..W },
        Ironcrafting => &AbilityInfo { name: "IRONCRAFTING", skill: Skill::Ironcrafting, ..W },
        Enchanting => &AbilityInfo { name: "ENCHANTING", skill: Skill::Enchanting, ..W },
        Bite => &AbilityInfo {
            name: "BITE",
            kind: AKind::Attack,
            cooldown: 15,
            recharge: 20,
            dmg: (1, 2),
            skill: Skill::Biting,
            ..W
        },
        ShovelBlow => &AbilityInfo {
            name: "SHOVEL BLOW",
            kind: AKind::Attack,
            cooldown: 30,
            recharge: 30,
            skill: Skill::Shovels,
            weapons: &[WSkill::Shovels],
            ..W
        },
        SpearSling => &AbilityInfo {
            name: "SPEAR AND SLING",
            kind: AKind::Attack,
            cooldown: 20,
            recharge: 30,
            skill: Skill::Spears,
            weapons: &[WSkill::Spears, WSkill::Slingshots],
            ..W
        },
        DoubleSmash => &AbilityInfo {
            name: "DOUBLE SMASH",
            kind: AKind::Attack,
            cooldown: 25,
            recharge: 25,
            targets: 2,
            skill: Skill::Shovels,
            weapons: &[WSkill::Shovels],
            ..W
        },
        Smash => &AbilityInfo {
            name: "SMASH",
            kind: AKind::Attack,
            cooldown: 20,
            recharge: 20,
            skill: Skill::Shovels,
            weapons: &[WSkill::Shovels],
            ..W
        },
        Call => &AbilityInfo {
            name: "THE CALL",
            kind: AKind::Ally,
            recharge: 100,
            range: 7,
            skill: Skill::Motivation,
            cond: Some((Cond::Bolstered, 1, 1)),
            ..W
        },
        Lightning => &AbilityInfo {
            name: "LIGHTNING",
            kind: AKind::Attack,
            cooldown: 40,
            recharge: 100,
            range: 7,
            dmg: (1, 12),
            skill: Skill::Energy,
            ..W
        },
        PondLight => &AbilityInfo {
            name: "POND LIGHT",
            kind: AKind::Attack,
            cooldown: 40,
            recharge: 100,
            range: 7,
            radius: 3,
            targets: 1,
            dmg: (1, 6),
            skill: Skill::Energy,
            cond: Some((Cond::Blinded, 2, 10)),
            ..W
        },
        Regrowth => &AbilityInfo {
            name: "REGROWTH",
            kind: AKind::Ally,
            cooldown: 10,
            recharge: 100,
            range: 7,
            skill: Skill::Healing,
            cond: Some((Cond::Regenerating, 1, 10)),
            ..W
        },
        Alter => &AbilityInfo {
            name: "ALTER",
            kind: AKind::Ally,
            cooldown: 10,
            recharge: 100,
            range: 7,
            targets: 2,
            skill: Skill::Alteration,
            cond: Some((Cond::Altered, 1, 2)),
            ..W
        },
        Beak => &AbilityInfo {
            name: "BEAK",
            kind: AKind::Attack,
            cooldown: 25,
            recharge: 25,
            dmg: (1, 4),
            skill: Skill::Spears,
            ..W
        },
        SnakeBite => &AbilityInfo {
            name: "BITE",
            kind: AKind::Attack,
            cooldown: 20,
            recharge: 20,
            dmg: (1, 6),
            skill: Skill::Biting,
            ..W
        },
        Peck => &AbilityInfo {
            name: "PECK",
            kind: AKind::Attack,
            cooldown: 15,
            recharge: 15,
            dmg: (1, 3),
            skill: Skill::Biting,
            ..W
        },
        Jaws => &AbilityInfo {
            name: "JAWS",
            kind: AKind::Attack,
            cooldown: 40,
            recharge: 40,
            dmg: (2, 8),
            skill: Skill::Biting,
            ..W
        },
    }
}

// ---- creature kinds ---------------------------------------------------------------------------

#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash, PartialOrd, Ord)]
pub enum Kind {
    Newt,
    Caecilian,
    TreeFrog,
    Salamander,
    ChorusToad,
    Toad,
    Axolotl,
    Bullfrog,
    Goliath,
    Olm,
    Daot,
    Heron,
    Snake,
    Crow,
    Wisp,
    Turtle,
    Gorf,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Role {
    Folk,
    Hero,
    Enemy,
}

/// How fast a kind dries on dry ground (ticks per stack by day; three times as long at night).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum DryClass {
    Frog,
    Toad,
    Axolotl,
    Never,
}

pub fn dry_ticks(d: DryClass) -> i32 {
    match d {
        DryClass::Frog => 300,
        DryClass::Toad => 900,
        DryClass::Axolotl => 225,
        DryClass::Never => 0,
    }
}

pub struct KindInfo {
    pub name: &'static str,
    pub role: Role,
    pub hp: (i32, i32),
    pub attrs: [i32; 6],
    pub abilities: &'static [(Ability, i32)],
    pub slots: &'static [Slot],
    pub dry: DryClass,
    pub walk: i32,
    pub flies: bool,
    pub size: i32,
    pub sight: i32,
    /// Bonus to defence, dodge and resistance (the turtle's shell).
    pub shell: i32,
    /// Small enough for a snake to swallow.
    pub small: bool,
    pub names: &'static [&'static str],
}

const K: KindInfo = KindInfo {
    name: "",
    role: Role::Folk,
    hp: (5, 3),
    attrs: [0; 6],
    abilities: &[],
    slots: &[],
    dry: DryClass::Frog,
    walk: WALK,
    flies: false,
    size: 1,
    sight: 6,
    shell: 0,
    small: false,
    names: &[],
};

pub fn kind(k: Kind) -> &'static KindInfo {
    use Ability as A;
    use Kind::*;
    match k {
        Newt => &KindInfo {
            name: "NEWT",
            hp: (3, 3),
            attrs: [-4, 0, 2, 0, 0, 0],
            abilities: &[(A::Mending, 1), (A::Gathering, 1), (A::Bite, 1), (A::ShovelBlow, 5), (A::Digging, 5)],
            slots: &[Slot::Shovel],
            small: true,
            names: &["WREN", "FEN", "SEDGE", "RIPPLE", "TANSY", "MOSS", "REED", "SORREL", "DUCKWEED", "BRACKEN",
                "LICHEN", "PEAT", "BURDOCK", "CRESS"],
            ..K
        },
        Caecilian => &KindInfo {
            name: "CAECILIAN",
            hp: (5, 3),
            attrs: [-4, 0, 2, 0, 2, 0],
            abilities: &[(A::Fungiculture, 1)],
            small: true,
            names: &["LOAM", "DELVE", "HUMUS", "MYCEL", "TUBER", "GRUB", "MULCH", "SPORE"],
            ..K
        },
        TreeFrog => &KindInfo {
            name: "TREE FROG",
            hp: (5, 3),
            attrs: [-2, 2, 0, 0, 0, 0],
            abilities: &[(A::Woodcrafting, 1)],
            small: true,
            names: &["TWIG", "KNOT", "BUD", "BARK", "SAP"],
            ..K
        },
        Salamander => &KindInfo {
            name: "SALAMANDER",
            hp: (5, 3),
            attrs: [0, 2, 0, 2, 0, 0],
            abilities: &[(A::Ironcrafting, 1)],
            dry: DryClass::Toad,
            names: &["CINDER", "EMBER", "SOOT", "FLINT", "ASH"],
            ..K
        },
        ChorusToad => &KindInfo {
            name: "CHORUS TOAD",
            hp: (6, 6),
            attrs: [0, 0, 2, 0, 0, 2],
            abilities: &[(A::ShovelBlow, 1), (A::Digging, 1), (A::Call, 5)],
            slots: &[Slot::Shovel],
            dry: DryClass::Toad,
            names: &["OLD CROAK"],
            ..K
        },
        Toad => &KindInfo {
            name: "TOAD",
            role: Role::Hero,
            hp: (8, 4),
            attrs: [2, 0, 2, -2, -2, 0],
            abilities: &[(A::SpearSling, 1)],
            slots: &[Slot::Spear, Slot::Sling],
            dry: DryClass::Toad,
            names: &["MOTT", "GRISTLE", "UMBER", "BRACK", "OLD LURK", "WART", "KNOBBLE", "CLOD", "TUSSOCK",
                "BOULDER"],
            ..K
        },
        Axolotl => &KindInfo {
            name: "AXOLOTL",
            role: Role::Hero,
            hp: (4, 2),
            attrs: [-2, -2, -2, 2, 2, 2],
            abilities: &[(A::Lightning, 1), (A::PondLight, 3), (A::Regrowth, 5)],
            slots: &[Slot::Bauble],
            dry: DryClass::Axolotl,
            small: true,
            names: &["XOCHI", "TLALOC", "ATL", "CITLALI", "NAHUI", "ILHUI"],
            ..K
        },
        Bullfrog => &KindInfo {
            name: "BULLFROG",
            role: Role::Hero,
            hp: (20, 10),
            attrs: [6, 0, 8, 0, 0, 0],
            abilities: &[(A::DoubleSmash, 1)],
            slots: &[Slot::Shovel, Slot::Shovel2, Slot::Head, Slot::Chest, Slot::Rump],
            size: 2,
            names: &["RUMBLE", "BELLOW", "GORGE", "THUNDER"],
            ..K
        },
        Goliath => &KindInfo {
            name: "GOLIATH FROG",
            role: Role::Hero,
            hp: (20, 10),
            attrs: [4, 0, 4, 2, 2, 2],
            abilities: &[(A::Smash, 1), (A::HeavyRepair, 1)],
            slots: &[Slot::Shovel],
            names: &["CAIRN", "MENHIR", "SLAB", "QUARRY"],
            ..K
        },
        Olm => &KindInfo {
            name: "THE OLM",
            role: Role::Hero,
            hp: (20, 10),
            attrs: [6, 6, 6, 6, 6, 6],
            abilities: &[(A::Bite, 1), (A::SpearSling, 1), (A::PondLight, 1), (A::Alter, 1)],
            slots: &[Slot::Bauble, Slot::Spear, Slot::Head],
            dry: DryClass::Axolotl,
            sight: 8,
            names: &["THE OLM"],
            ..K
        },
        Daot => &KindInfo {
            name: "DAOT",
            hp: (10, 3),
            attrs: [0, 0, 0, 4, 4, 2],
            abilities: &[(A::Enchanting, 1)],
            dry: DryClass::Toad,
            names: &["DAOT"],
            ..K
        },
        Heron => &KindInfo {
            name: "HERON",
            role: Role::Enemy,
            hp: (3, 3),
            attrs: [0, 2, 0, 0, 2, 0],
            abilities: &[(A::Beak, 1)],
            dry: DryClass::Never,
            walk: 8,
            flies: true,
            sight: 7,
            ..K
        },
        Snake => &KindInfo {
            name: "SNAKE",
            role: Role::Enemy,
            hp: (7, 3),
            attrs: [2, 2, 0, 0, 0, 0],
            abilities: &[(A::SnakeBite, 1)],
            dry: DryClass::Never,
            ..K
        },
        Crow => &KindInfo {
            name: "CROW",
            role: Role::Enemy,
            hp: (3, 2),
            attrs: [0, 6, 0, 0, 0, 0],
            abilities: &[(A::Peck, 1)],
            dry: DryClass::Never,
            walk: 6,
            flies: true,
            ..K
        },
        Wisp => &KindInfo {
            name: "WISP",
            role: Role::Enemy,
            hp: (4, 2),
            attrs: [0, 0, 0, 2, 0, 0],
            dry: DryClass::Never,
            walk: 12,
            flies: true,
            ..K
        },
        Turtle => &KindInfo {
            name: "SNAPPING TURTLE",
            role: Role::Enemy,
            hp: (27, 3),
            attrs: [8, 0, 8, 0, 0, 0],
            abilities: &[(A::Jaws, 1)],
            dry: DryClass::Never,
            walk: 20,
            shell: 6,
            ..K
        },
        Gorf => &KindInfo {
            name: "GORF",
            role: Role::Enemy,
            hp: (90, 3),
            attrs: [2, 2, 4, 6, 6, 4],
            dry: DryClass::Never,
            sight: 12,
            ..K
        },
    }
}

// ---- spots ---------------------------------------------------------------------------------

#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash)]
pub enum SpotKind {
    Fern,
    Red,
    White,
    Hybrid,
    Puff,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum When {
    Damp,
    Night,
}

pub struct SpotInfo {
    pub ripen: i32,
    pub work: i32,
    pub diff: i32,
    pub gives: Item,
    pub n: i32,
    pub skill: Skill,
    /// Planting skill and difficulty, for planted kinds.
    pub plant: Option<(i32, i32)>,
    pub when: When,
}

pub fn spot(s: SpotKind) -> &'static SpotInfo {
    match s {
        SpotKind::Fern => &SpotInfo {
            ripen: 1000,
            work: 10,
            diff: 10,
            gives: Item::Dew,
            n: 20,
            skill: Skill::Gathering,
            plant: None,
            when: When::Damp,
        },
        SpotKind::Red => &SpotInfo {
            ripen: 1000,
            work: 6,
            diff: 15,
            gives: Item::Fungiwood,
            n: 2,
            skill: Skill::Fungiculture,
            plant: Some((0, 10)),
            when: When::Damp,
        },
        SpotKind::White => &SpotInfo {
            ripen: 1000,
            work: 9,
            diff: 20,
            gives: Item::CapIngot,
            n: 1,
            skill: Skill::Fungiculture,
            plant: Some((5, 15)),
            when: When::Damp,
        },
        SpotKind::Hybrid => &SpotInfo {
            ripen: 1000,
            work: 12,
            diff: 25,
            gives: Item::SpottedCap,
            n: 1,
            skill: Skill::Fungiculture,
            plant: Some((8, 25)),
            when: When::Damp,
        },
        SpotKind::Puff => &SpotInfo {
            ripen: 1000,
            work: 6,
            diff: 15,
            gives: Item::GlowSpores,
            n: 1,
            skill: Skill::Fungiculture,
            plant: None,
            when: When::Night,
        },
    }
}

pub const BONUS_REGROW: i32 = 600;
pub const FUNGI_AMOUNT: i32 = 2;
pub const GATHER_AMOUNT: i32 = 1;
pub const TEND_PCT: i32 = 150;

// ---- buildings and lairs --------------------------------------------------------------------

#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash, PartialOrd, Ord)]
pub enum BKind {
    Hall,
    Burrow,
    TwigLoft,
    MorelCave,
    FairyRing,
    ChorusLog,
    EmberLog,
    IroncapCave,
    ToadHall,
    ShellForge,
    Cauldron,
    StarPond,
    NewtHouse,
    SpawningPool,
    DaotsGrotto,
    OlmsCave,
    Nest,
    LoneNest,
    GorfStone,
    LeechPool,
    CrowRoost,
    GorfHollow,
}

pub const HOLD_BUILDINGS: [BKind; 16] = [
    BKind::Hall,
    BKind::Burrow,
    BKind::TwigLoft,
    BKind::MorelCave,
    BKind::FairyRing,
    BKind::ChorusLog,
    BKind::EmberLog,
    BKind::IroncapCave,
    BKind::ToadHall,
    BKind::ShellForge,
    BKind::Cauldron,
    BKind::StarPond,
    BKind::NewtHouse,
    BKind::SpawningPool,
    BKind::DaotsGrotto,
    BKind::OlmsCave,
];

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Place {
    Anywhere,
    Rock,
    Pond,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Team {
    Hold,
    Gorf,
    Wild,
}

pub struct BInfo {
    pub name: &'static str,
    pub fp: (i32, i32),
    pub hp: i32,
    pub cost: &'static [(Item, i32)],
    pub src: Option<BKind>,
    pub houses: &'static [(Kind, i32)],
    pub hatch: i32,
    /// Default stock levels, and the limits they can be raised to.
    pub keeps: &'static [(Item, i32)],
    pub limits: &'static [(Item, i32)],
    pub makes: &'static [Item],
    pub spots: &'static [(SpotKind, i32)],
    pub plot: Option<(SpotKind, i32)>,
    pub place: Place,
    pub team: Team,
    pub demolition: i32,
    pub difficulty: i32,
    /// Health a resting creature regains, every so many ticks.
    pub rest: (i32, i32),
}

const B0: BInfo = BInfo {
    name: "",
    fp: (2, 2),
    hp: 60,
    cost: &[],
    src: None,
    houses: &[],
    hatch: 500,
    keeps: &[],
    limits: &[],
    makes: &[],
    spots: &[],
    plot: None,
    place: Place::Anywhere,
    team: Team::Hold,
    demolition: 15,
    difficulty: 10,
    rest: (1, 100),
};

pub fn bld(b: BKind) -> &'static BInfo {
    use BKind::*;
    use Item::*;
    match b {
        Hall => &BInfo {
            name: "THE HALL",
            fp: (4, 3),
            hp: 1000,
            houses: &[(Kind::Newt, 8)],
            keeps: &[(WoodenShovel, 2)],
            limits: &[(Fungiwood, 10), (SunOrb, 2), (WoodenShovel, 4)],
            spots: &[(SpotKind::Fern, 4)],
            ..B0
        },
        Burrow => &BInfo {
            name: "THE BURROW",
            hp: 150,
            cost: &[(Dew, 20), (Fungiwood, 4)],
            src: Some(Hall),
            houses: &[(Kind::Caecilian, 2)],
            keeps: &[(Dew, 10)],
            limits: &[(Dew, 25), (Fungiwood, 4), (CapIngot, 1)],
            spots: &[(SpotKind::Red, 1)],
            ..B0
        },
        TwigLoft => &BInfo {
            name: "TWIG LOFT",
            hp: 150,
            cost: &[(Dew, 10)],
            src: Some(Hall),
            houses: &[(Kind::TreeFrog, 2)],
            keeps: &[(Fungiwood, 3), (WoodenShovel, 3)],
            limits: &[(Dew, 20), (Fungiwood, 6), (WoodenShovel, 6)],
            makes: &[WoodenShovel],
            spots: &[(SpotKind::Red, 1)],
            ..B0
        },
        MorelCave => &BInfo {
            name: "MOREL CAVE",
            hp: 150,
            cost: &[(Dew, 20), (Fungiwood, 4)],
            src: Some(Burrow),
            houses: &[(Kind::Caecilian, 2)],
            keeps: &[(Dew, 10)],
            limits: &[(Dew, 20)],
            plot: Some((SpotKind::Red, 4)),
            place: Place::Rock,
            ..B0
        },
        FairyRing => &BInfo {
            name: "FAIRY RING",
            hp: 100,
            cost: &[(Fungiwood, 1), (CapIngot, 1)],
            src: Some(Burrow),
            plot: Some((SpotKind::Hybrid, 2)),
            ..B0
        },
        ChorusLog => &BInfo {
            name: "CHORUS LOG",
            fp: (3, 2),
            hp: 500,
            cost: &[(Dew, 50), (Fungiwood, 10)],
            src: Some(Hall),
            houses: &[(Kind::ChorusToad, 1)],
            hatch: 1500,
            keeps: &[(WoodenShovel, 3), (IronShovel, 2)],
            limits: &[(Dew, 25), (Fungiwood, 5), (CapIngot, 5), (WoodenShovel, 5), (IronShovel, 5)],
            rest: (2, 100),
            ..B0
        },
        EmberLog => &BInfo {
            name: "EMBER LOG",
            hp: 150,
            cost: &[(Dew, 10)],
            src: Some(ChorusLog),
            houses: &[(Kind::Salamander, 2)],
            keeps: &[(Dew, 10), (Fungiwood, 1), (CapIngot, 1), (IronShovel, 2)],
            limits: &[(Dew, 20), (Fungiwood, 3), (CapIngot, 3), (IronShovel, 4)],
            makes: &[IronShovel],
            ..B0
        },
        IroncapCave => &BInfo {
            name: "IRONCAP CAVE",
            hp: 150,
            cost: &[(Dew, 20), (Fungiwood, 4)],
            src: Some(ChorusLog),
            houses: &[(Kind::Caecilian, 2)],
            keeps: &[(Dew, 10)],
            limits: &[(Dew, 20)],
            plot: Some((SpotKind::White, 4)),
            place: Place::Rock,
            ..B0
        },
        ToadHall => &BInfo {
            name: "TOAD HALL",
            fp: (3, 2),
            hp: 350,
            cost: &[(Fungiwood, 5)],
            src: Some(ChorusLog),
            houses: &[(Kind::Toad, 5)],
            hatch: 700,
            keeps: &[(Dew, 10), (Fungiwood, 2), (CapIngot, 2), (Slingshot, 3), (IronSpear, 2)],
            limits: &[(Dew, 30), (Fungiwood, 5), (CapIngot, 5), (Slingshot, 5), (IronSpear, 5)],
            makes: &[Slingshot, IronSpear],
            ..B0
        },
        ShellForge => &BInfo {
            name: "SHELL FORGE",
            fp: (3, 2),
            hp: 350,
            cost: &[(CapIngot, 5)],
            src: Some(ToadHall),
            keeps: &[(CapIngot, 3), (Chanfron, 1), (Peytral, 1), (Croupiere, 1)],
            limits: &[(CapIngot, 10), (Chanfron, 5), (Peytral, 5), (Croupiere, 5)],
            makes: &[Chanfron, Peytral, Croupiere],
            ..B0
        },
        Cauldron => &BInfo {
            name: "CAULDRON",
            hp: 300,
            cost: &[(CapIngot, 5)],
            src: Some(ToadHall),
            keeps: &[(Fungiwood, 2), (SpottedCap, 2), (VenomwoodShovel, 1), (VenomwoodSpear, 2)],
            limits: &[(Fungiwood, 5), (SpottedCap, 5), (VenomwoodShovel, 5), (VenomwoodSpear, 5)],
            makes: &[VenomwoodShovel, VenomwoodSpear],
            ..B0
        },
        StarPond => &BInfo {
            name: "STAR POND",
            fp: (3, 3),
            hp: 500,
            cost: &[(SunOrb, 1)],
            src: Some(Hall),
            houses: &[(Kind::Axolotl, 4)],
            keeps: &[(Dew, 14), (Fungiwood, 2), (GlowSpores, 2), (DewBead, 2), (GlowCharm, 1)],
            limits: &[(Dew, 250), (Fungiwood, 50), (GlowSpores, 5), (DewBead, 5), (GlowCharm, 5), (SunOrb, 5),
                (ClargentOrb, 5), (MeteorOrb, 5)],
            makes: &[DewBead, GlowCharm],
            spots: &[(SpotKind::Puff, 2)],
            place: Place::Pond,
            ..B0
        },
        NewtHouse => &BInfo {
            name: "NEWT HOUSE",
            hp: 150,
            cost: &[(Dew, 10), (Fungiwood, 2)],
            src: Some(StarPond),
            houses: &[(Kind::Newt, 4)],
            spots: &[(SpotKind::Fern, 2)],
            limits: &[(Dew, 25)],
            ..B0
        },
        SpawningPool => &BInfo {
            name: "SPAWNING POOL",
            fp: (3, 3),
            hp: 500,
            cost: &[(ClargentOrb, 1), (Fungiwood, 10), (Dew, 150)],
            src: Some(StarPond),
            houses: &[(Kind::Bullfrog, 2), (Kind::Goliath, 2)],
            keeps: &[(Dew, 20)],
            limits: &[(Dew, 250), (ClargentOrb, 5)],
            ..B0
        },
        DaotsGrotto => &BInfo {
            name: "DAOT'S GROTTO",
            fp: (3, 2),
            hp: 350,
            cost: &[(Dew, 50), (Fungiwood, 5), (CapIngot, 2)],
            src: Some(StarPond),
            houses: &[(Kind::Daot, 1)],
            keeps: &[(Fungiwood, 1), (CapIngot, 2), (SpottedCap, 2), (GlowSpores, 2), (MeteorOrb, 1), (Catapult, 1),
                (VenomironSpear, 1), (VenomironChanfron, 1), (StarPearl, 1)],
            limits: &[(Fungiwood, 5), (CapIngot, 5), (SpottedCap, 5), (GlowSpores, 5), (MeteorOrb, 5), (Catapult, 5),
                (VenomironSpear, 5), (VenomironChanfron, 5), (StarPearl, 5)],
            makes: &[Catapult, VenomironSpear, VenomironChanfron, StarPearl],
            place: Place::Rock,
            ..B0
        },
        OlmsCave => &BInfo {
            name: "THE OLM'S CAVE",
            fp: (3, 2),
            hp: 500,
            cost: &[(ClargentOrb, 1), (Dew, 120)],
            src: Some(SpawningPool),
            houses: &[(Kind::Olm, 1)],
            limits: &[(SunOrb, 5), (ClargentOrb, 5), (MeteorOrb, 5)],
            place: Place::Rock,
            ..B0
        },
        Nest => &BInfo {
            name: "HERON NEST",
            hp: 150,
            houses: &[(Kind::Heron, 4)],
            hatch: 1800,
            team: Team::Gorf,
            demolition: 11,
            difficulty: 11,
            rest: (3, 20),
            ..B0
        },
        LoneNest => &BInfo {
            name: "LONE NEST",
            hp: 60,
            houses: &[(Kind::Heron, 1)],
            hatch: 600,
            team: Team::Gorf,
            demolition: 10,
            difficulty: 10,
            rest: (3, 20),
            ..B0
        },
        GorfStone => &BInfo {
            name: "GORF'S STONE",
            fp: (1, 2),
            hp: 200,
            houses: &[(Kind::Wisp, 3)],
            hatch: 600,
            team: Team::Gorf,
            demolition: 14,
            difficulty: 14,
            ..B0
        },
        LeechPool => &BInfo {
            name: "LEECH POOL",
            hp: 80,
            team: Team::Wild,
            demolition: 10,
            difficulty: 10,
            ..B0
        },
        CrowRoost => &BInfo {
            name: "CROWS' ROOST",
            hp: 60,
            houses: &[(Kind::Crow, 3)],
            hatch: 900,
            team: Team::Wild,
            demolition: 10,
            difficulty: 10,
            ..B0
        },
        GorfHollow => &BInfo {
            name: "GORF'S HOLLOW",
            fp: (4, 3),
            hp: 400,
            houses: &[(Kind::Turtle, 5), (Kind::Gorf, 1)],
            hatch: 1200,
            team: Team::Gorf,
            demolition: 20,
            difficulty: 20,
            rest: (3, 20),
            ..B0
        },
    }
}

/// The limit a building's stock level can be raised to.
pub fn limit(b: BKind, i: Item) -> i32 {
    let info = bld(b);
    let l = info.limits.iter().find(|x| x.0 == i).map_or(0, |x| x.1);
    let k = info.keeps.iter().find(|x| x.0 == i).map_or(0, |x| x.1);
    let input = info.makes.iter().any(|m| item(*m).inputs.iter().any(|x| x.0 == i));
    l.max(k).max(if input { 5 } else { 0 })
}

// ---- spells -----------------------------------------------------------------------------------

#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash)]
pub enum Spell {
    RainCall,
    OldSong,
    Starlight,
    LeyLight,
    DeepCalling,
}

pub const SPELLS: [Spell; 5] = [Spell::RainCall, Spell::OldSong, Spell::Starlight, Spell::LeyLight, Spell::DeepCalling];

pub struct SpellInfo {
    pub name: &'static str,
    pub from: BKind,
    pub cost: i32,
    pub recharge: u32,
    pub radius: i32,
    /// Whether it needs a place on the map.
    pub aimed: bool,
}

pub fn spell(s: Spell) -> &'static SpellInfo {
    match s {
        Spell::RainCall => &SpellInfo { name: "RAIN-CALL", from: BKind::Hall, cost: 40, recharge: DAY, radius: 7, aimed: true },
        Spell::OldSong => {
            &SpellInfo { name: "THE OLD SONG", from: BKind::ChorusLog, cost: 30, recharge: DAY / 2, radius: 7, aimed: false }
        }
        Spell::Starlight => {
            &SpellInfo { name: "STARLIGHT", from: BKind::StarPond, cost: 25, recharge: DAY / 2, radius: 6, aimed: true }
        }
        Spell::LeyLight => &SpellInfo { name: "LEY-LIGHT", from: BKind::DaotsGrotto, cost: 40, recharge: DAY, radius: 3, aimed: true },
        Spell::DeepCalling => {
            &SpellInfo { name: "THE DEEP CALLING", from: BKind::OlmsCave, cost: 50, recharge: DAY, radius: 0, aimed: false }
        }
    }
}

// ---- rewards ----------------------------------------------------------------------------------

pub const REWARD_FIRST: i32 = 20;
pub const REWARD_STEP: i32 = 10;
pub const MAX_REWARDS: usize = 6;
pub const TITHE_PCT: i32 = 10;
pub const HATCH_PURSE: i32 = 10;
pub const BUILDERS: usize = 4;
pub const PUDDLE_COST: i32 = 5;

// ---- the marsh --------------------------------------------------------------------------------

pub const HERON_HUNT: i32 = 10;
/// How far a heron notices prey, as Testario's skeletons did (hunt look distance 5).
pub const HERON_LOOK: i32 = 6;
pub const HERON_CALL: i32 = 8;
pub const SNAKE_FROM_MIN: u32 = 8;
pub const CROW_ITEM_AGE: u32 = 450;
pub const WISP_DRAIN_EVERY: u32 = 300;
pub const TURTLE_GUARD: i32 = 8;
pub const SWALLOW_TICKS: u32 = 1000;
pub const HUSK_TICKS: u32 = DAY / 2;
pub const DREAM_TICKS: i32 = 1000;
pub const STONE_AT: [usize; 3] = [10, 13, 16];
