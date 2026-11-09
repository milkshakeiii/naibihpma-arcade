//! The fixed rules: creatures, goods, buildings, homes, seasons and prices. Everything the player
//! has to learn lives here, so the help pages, tooltips and the colony all read the same numbers.

#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash, PartialOrd, Ord)]
pub enum Species {
    Tribble,
    Dodo,
    Ape,
}

pub const SPECIES: [Species; 3] = [Species::Tribble, Species::Dodo, Species::Ape];

#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash, PartialOrd, Ord)]
pub enum Weight {
    Empty,
    Light,
    Medium,
    Heavy,
}

pub const WEIGHTS: [Weight; 4] = [Weight::Empty, Weight::Light, Weight::Medium, Weight::Heavy];

impl Weight {
    pub fn word(self) -> &'static str {
        match self {
            Weight::Empty => "nothing",
            Weight::Light => "light",
            Weight::Medium => "medium",
            Weight::Heavy => "heavy",
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash, PartialOrd, Ord)]
pub enum Good {
    Berry,
    Nut,
    Fish,
    Twig,
    Stone,
    Clay,
    Wicker,
    Pot,
    Brick,
}

pub const NGOODS: usize = 9;
pub const GOODS: [Good; NGOODS] = [Good::Berry, Good::Nut, Good::Fish, Good::Twig, Good::Stone, Good::Clay, Good::Wicker, Good::Pot, Good::Brick];
pub const FOODS: [Good; 3] = [Good::Berry, Good::Nut, Good::Fish];

impl Good {
    pub fn weight(self) -> Weight {
        match self {
            Good::Berry | Good::Wicker | Good::Pot => Weight::Light,
            Good::Nut | Good::Fish | Good::Twig | Good::Clay | Good::Brick => Weight::Medium,
            Good::Stone => Weight::Heavy,
        }
    }

    /// Fragile goods break if their carrier falls more than `FRAGILE_FALL` tiles.
    pub fn fragile(self) -> bool {
        matches!(self, Good::Pot)
    }

    pub fn food(self) -> bool {
        matches!(self, Good::Berry | Good::Nut | Good::Fish)
    }

    pub fn name(self) -> &'static str {
        match self {
            Good::Berry => "BERRY",
            Good::Nut => "NUT",
            Good::Fish => "FISH",
            Good::Twig => "TWIG",
            Good::Stone => "STONE",
            Good::Clay => "CLAY",
            Good::Wicker => "WICKER",
            Good::Pot => "POT",
            Good::Brick => "BRICK",
        }
    }

    /// Lowercase display word ("twig") and plural ("twigs").
    pub fn word(self) -> &'static str {
        match self {
            Good::Berry => "berry",
            Good::Nut => "nut",
            Good::Fish => "fish",
            Good::Twig => "twig",
            Good::Stone => "stone",
            Good::Clay => "clay",
            Good::Wicker => "wicker",
            Good::Pot => "pot",
            Good::Brick => "brick",
        }
    }

    pub fn words(self) -> &'static str {
        match self {
            Good::Berry => "berries",
            Good::Nut => "nuts",
            Good::Fish => "fish",
            Good::Twig => "twigs",
            Good::Stone => "stone",
            Good::Clay => "clay",
            Good::Wicker => "wicker",
            Good::Pot => "pots",
            Good::Brick => "bricks",
        }
    }

    pub fn count(self, n: i32) -> String {
        format!("{n} {}", if n == 1 { self.word() } else { self.words() })
    }

    pub fn index(self) -> usize {
        self as usize
    }

    pub fn from_name(s: &str) -> Option<Good> {
        GOODS.iter().copied().find(|g| g.name().eq_ignore_ascii_case(s) || g.words().eq_ignore_ascii_case(s))
    }

    /// Ticks a camp worker spends gathering one at a source.
    pub fn harvest_ticks(self) -> i32 {
        match self {
            Good::Berry => 10,
            Good::Nut => 12,
            Good::Fish => 24,
            Good::Twig => 12,
            Good::Stone => 30,
            Good::Clay => 20,
            _ => 20,
        }
    }

    /// How much a source of this good grows back per season (spring, summer, autumn, winter):
    /// 0 = nothing that season, 1 = normal, 3 = three times as fast.
    pub fn season_growth(self, season: usize) -> i32 {
        match self {
            Good::Berry => [1, 1, 1, 0][season],
            Good::Nut => [0, 0, 3, 0][season],
            Good::Fish => [1, 1, 1, 0][season],
            _ => 1,
        }
    }

    /// The source's own name.
    pub fn source_name(self) -> &'static str {
        match self {
            Good::Berry => "Berry bush",
            Good::Nut => "Nut tree",
            Good::Fish => "Fishing spot",
            Good::Twig => "Twig pile",
            Good::Stone => "Quarry",
            Good::Clay => "Clay bank",
            _ => "Source",
        }
    }
}

/// What a creature is carrying, as far as its legs are concerned.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash, PartialOrd, Ord)]
pub struct Load {
    pub weight: Weight,
    pub fragile: bool,
}

impl Load {
    pub const EMPTY: Load = Load { weight: Weight::Empty, fragile: false };

    pub fn of(good: Good) -> Load {
        Load { weight: good.weight(), fragile: good.fragile() }
    }

    /// A small index for caching per-load data: empty, light, light fragile, medium, heavy.
    pub fn key(self) -> usize {
        match (self.weight, self.fragile) {
            (Weight::Empty, _) => 0,
            (Weight::Light, false) => 1,
            (Weight::Light, true) => 2,
            (Weight::Medium, _) => 3,
            (Weight::Heavy, _) => 4,
        }
    }
}

pub const NLOADS: usize = 5;

/// The highest fall (in tiles) a creature may take while carrying something fragile.
pub const FRAGILE_FALL: i32 = 1;
/// Ticks a creature spends handing over or taking one good.
pub const DROP_TICKS: i32 = 2;
/// A springboard adds this much to the jump of whoever leaps from it.
pub const SPRING_BONUS: i32 = 2;

impl Species {
    pub fn name(self) -> &'static str {
        match self {
            Species::Tribble => "TRIBBLE",
            Species::Dodo => "DODO",
            Species::Ape => "APE",
        }
    }

    pub fn title(self) -> &'static str {
        match self {
            Species::Tribble => "Tribble",
            Species::Dodo => "Dodo",
            Species::Ape => "Ape",
        }
    }

    pub fn plural(self) -> &'static str {
        match self {
            Species::Tribble => "tribbles",
            Species::Dodo => "dodos",
            Species::Ape => "apes",
        }
    }

    pub fn index(self) -> usize {
        self as usize
    }

    pub fn from_name(s: &str) -> Option<Species> {
        SPECIES.iter().copied().find(|g| g.name().eq_ignore_ascii_case(s))
    }

    /// The home this species lives in.
    pub fn home(self) -> Kind {
        match self {
            Species::Tribble => Kind::Burrow,
            Species::Dodo => Kind::Nest,
            Species::Ape => Kind::Lodge,
        }
    }

    /// How many tiles up a jump can rise (0 = can't jump). The jump also carries it up to
    /// `jump + 1` tiles sideways at the top before it drops.
    pub fn jump(self, w: Weight) -> i32 {
        use Weight::*;
        match (self, w) {
            (Species::Tribble, Empty | Light) => 3,
            (Species::Tribble, Medium) => 1,
            (Species::Tribble, Heavy) => 0,
            (Species::Dodo, Heavy) => 1,
            (Species::Dodo, _) => 2,
            (Species::Ape, Heavy) => 1,
            (Species::Ape, _) => 2,
        }
    }

    /// Ticks per tile walked, climbed (doubled) or jumped. Falling always costs 1 tick a tile.
    pub fn pace(self, w: Weight) -> i32 {
        use Weight::*;
        match (self, w) {
            (Species::Tribble, Empty | Light) => 1,
            (Species::Tribble, Medium) => 3,
            (Species::Tribble, Heavy) => 8,
            (Species::Dodo, Empty | Light) => 1,
            (Species::Dodo, Medium) => 2,
            (Species::Dodo, Heavy) => 4,
            (Species::Ape, Heavy) => 2,
            (Species::Ape, _) => 1,
        }
    }

    /// Apes climb vines and ladders with anything; everyone else climbs ladders only when
    /// carrying nothing or something light, and never vines.
    pub fn climbs_vine(self) -> bool {
        self == Species::Ape
    }

    pub fn climbs_ladder(self, w: Weight) -> bool {
        self == Species::Ape || w <= Weight::Light
    }

    pub fn blurb(self) -> &'static str {
        match self {
            Species::Tribble => "Many, cheap, bouncy. Hates weight.",
            Species::Dodo => "Steady. Doesn't mind a twig.",
            Species::Ape => "Few, strong. Climbs vines and ladders.",
        }
    }
}

// ----- Time --------------------------------------------------------------------------------------

/// Ticks in a day (8 seconds at 1x).
pub const DAY: i32 = 120;
pub const SEASON_DAYS: i32 = 6;
pub const YEAR_DAYS: i32 = SEASON_DAYS * 4;
pub const SEASONS: [&str; 4] = ["Spring", "Summer", "Autumn", "Winter"];

pub fn season_of(tick: i32) -> usize {
    ((tick / DAY / SEASON_DAYS) % 4) as usize
}

/// Day number, counting from 1.
pub fn day_of(tick: i32) -> i32 {
    tick / DAY + 1
}

// ----- Buildings ---------------------------------------------------------------------------------

#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash, PartialOrd, Ord)]
pub enum Kind {
    Burrow,
    Nest,
    Lodge,
    Camp,
    Store,
    Weaver,
    Kiln,
    Mason,
    Garden,
}

pub const KINDS: [Kind; 9] = [Kind::Burrow, Kind::Nest, Kind::Lodge, Kind::Camp, Kind::Store, Kind::Weaver, Kind::Kiln, Kind::Mason, Kind::Garden];

pub struct Recipe {
    pub inputs: &'static [(Good, i32)],
    pub output: Good,
    pub ticks: i32,
}

pub const WEAVER: Recipe = Recipe { inputs: &[(Good::Twig, 2)], output: Good::Wicker, ticks: 60 };
pub const KILN: Recipe = Recipe { inputs: &[(Good::Clay, 1), (Good::Twig, 1)], output: Good::Pot, ticks: 70 };
pub const MASON: Recipe = Recipe { inputs: &[(Good::Stone, 1)], output: Good::Brick, ticks: 40 };

/// A store holds this many goods in total...
pub const STORE_CAP: i32 = 24;
/// ...and no more than this many of any one good.
pub const STORE_PER_GOOD: i32 = 8;
/// A workshop holds up to this many of each input and of its output.
pub const SHOP_IN_CAP: i32 = 4;
pub const SHOP_OUT_CAP: i32 = 3;
/// A camp worker can't carry its harvest further than this many ticks of walking, and nor can
/// anyone else: deliveries look for places within this range.
pub const RANGE: i32 = 100;
/// Noise and pleasantness reach this many tiles (in a square) around their building.
pub const AURA: i32 = 3;

impl Kind {
    pub fn name(self) -> &'static str {
        match self {
            Kind::Burrow => "BURROW",
            Kind::Nest => "NEST",
            Kind::Lodge => "LODGE",
            Kind::Camp => "CAMP",
            Kind::Store => "STORE",
            Kind::Weaver => "WEAVER",
            Kind::Kiln => "KILN",
            Kind::Mason => "MASON",
            Kind::Garden => "GARDEN",
        }
    }

    pub fn title(self) -> &'static str {
        match self {
            Kind::Burrow => "Burrow",
            Kind::Nest => "Nest",
            Kind::Lodge => "Lodge",
            Kind::Camp => "Camp",
            Kind::Store => "Store",
            Kind::Weaver => "Weaver",
            Kind::Kiln => "Kiln",
            Kind::Mason => "Mason",
            Kind::Garden => "Garden",
        }
    }

    pub fn from_name(s: &str) -> Option<Kind> {
        KINDS.iter().copied().find(|g| g.name().eq_ignore_ascii_case(s))
    }

    pub fn cost(self) -> i32 {
        match self {
            Kind::Burrow => 5,
            Kind::Nest => 7,
            Kind::Lodge => 9,
            Kind::Camp => 4,
            Kind::Store => 10,
            Kind::Weaver => 12,
            Kind::Kiln => 14,
            Kind::Mason => 14,
            Kind::Garden => 6,
        }
    }

    /// Footprint in tiles. A camp sits on its source's tile.
    pub fn size(self) -> (i32, i32) {
        match self {
            Kind::Camp | Kind::Garden => (1, 1),
            _ => (2, 2),
        }
    }

    pub fn recipe(self) -> Option<&'static Recipe> {
        match self {
            Kind::Weaver => Some(&WEAVER),
            Kind::Kiln => Some(&KILN),
            Kind::Mason => Some(&MASON),
            _ => None,
        }
    }

    /// The species that lives here, for homes.
    pub fn resident(self) -> Option<Species> {
        match self {
            Kind::Burrow => Some(Species::Tribble),
            Kind::Nest => Some(Species::Dodo),
            Kind::Lodge => Some(Species::Ape),
            _ => None,
        }
    }

    pub fn is_home(self) -> bool {
        self.resident().is_some()
    }

    /// Worker slots (0 for homes and gardens).
    pub fn slots(self) -> i32 {
        match self {
            Kind::Camp | Kind::Store | Kind::Weaver | Kind::Kiln | Kind::Mason => 2,
            _ => 0,
        }
    }

    /// Noisy buildings spoil the spot for manors nearby; gardens make it pleasant.
    pub fn aura(self) -> i32 {
        match self {
            Kind::Mason | Kind::Kiln => -2,
            Kind::Garden => 2,
            _ => 0,
        }
    }

    pub fn blurb(self) -> &'static str {
        match self {
            Kind::Burrow => "Home for tribbles. Settlers walk in from the trail.",
            Kind::Nest => "Home for dodos. Settlers walk in from the trail.",
            Kind::Lodge => "Home for apes. Settlers walk in from the trail.",
            Kind::Camp => "Put it on a source. Its workers gather and carry.",
            Kind::Store => "Holds 24 goods, 8 of a kind. Its carriers supply all in reach.",
            Kind::Weaver => "2 twigs make 1 wicker.",
            Kind::Kiln => "Clay and a twig make 1 pot.",
            Kind::Mason => "1 stone makes 1 brick.",
            Kind::Garden => "Makes the spot pleasant. Manors need that.",
        }
    }
}

/// Workers a camp hires: one for bushes, trees, fish and twigs; two for heavy digging and the
/// autumn nut rush.
pub fn camp_slots(g: Good) -> i32 {
    match g {
        Good::Stone | Good::Clay | Good::Nut => 2,
        _ => 1,
    }
}

/// Camps on quarries and clay banks are noisy too.
pub fn camp_aura(g: Good) -> i32 {
    match g {
        Good::Stone | Good::Clay => -2,
        _ => 0,
    }
}

// ----- Homes -------------------------------------------------------------------------------------

pub const TIERS: usize = 4;
pub const TIER_NAMES: [&str; TIERS] = ["Shack", "Cottage", "House", "Manor"];

/// Residents per tier for each species' home.
pub fn residents(s: Species, tier: usize) -> i32 {
    match s {
        Species::Tribble => [3, 4, 6, 8][tier],
        Species::Dodo => [2, 3, 5, 6][tier],
        Species::Ape => [2, 3, 4, 5][tier],
    }
}

/// Tax per resident per day, in tenths of a seed.
pub const TAX_TENTHS: [i32; TIERS] = [3, 4, 6, 9];

/// What a home needs to be at a tier: kinds of food, then goods.
pub struct TierNeeds {
    pub foods: i32,
    pub goods: &'static [Good],
    pub pleasant: i32,
}

pub const TIER_NEEDS: [TierNeeds; TIERS] = [
    TierNeeds { foods: 1, goods: &[], pleasant: i32::MIN },
    TierNeeds { foods: 1, goods: &[Good::Wicker], pleasant: i32::MIN },
    TierNeeds { foods: 2, goods: &[Good::Wicker, Good::Pot], pleasant: i32::MIN },
    TierNeeds { foods: 2, goods: &[Good::Wicker, Good::Pot, Good::Brick], pleasant: 2 },
];

/// Each resident eats a food every this many ticks.
pub const EAT_TICKS: i32 = DAY * 4;
/// A home uses up one of each good it needs every this many ticks.
pub const WEAR_TICKS: i32 = DAY * 4;
/// How long a home goes without something it needs before it slips a tier (or, as a shack, loses
/// a resident).
pub const GRACE: i32 = DAY * 2;
/// How long a home must have everything for the next tier before it moves up.
pub const SETTLE: i32 = DAY / 2;
/// Gap between settlers heading to the same home.
pub const SETTLER_GAP: i32 = 40;
/// A home stocks up to this many of each food and each good.
pub const HOME_FOOD_CAP: i32 = 3;
pub const HOME_GOOD_CAP: i32 = 2;

// ----- Infrastructure ----------------------------------------------------------------------------

#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash, PartialOrd, Ord)]
pub enum Infra {
    Spring,
    Ladder,
    Plank,
}

pub const INFRAS: [Infra; 3] = [Infra::Spring, Infra::Ladder, Infra::Plank];

impl Infra {
    pub fn name(self) -> &'static str {
        match self {
            Infra::Spring => "SPRINGBOARD",
            Infra::Ladder => "LADDER",
            Infra::Plank => "PLANK",
        }
    }

    pub fn short(self) -> &'static str {
        match self {
            Infra::Spring => "SPRING",
            Infra::Ladder => "LADDER",
            Infra::Plank => "PLANK",
        }
    }

    pub fn title(self) -> &'static str {
        match self {
            Infra::Spring => "Springboard",
            Infra::Ladder => "Ladder",
            Infra::Plank => "Plank",
        }
    }

    pub fn from_name(s: &str) -> Option<Infra> {
        INFRAS.iter().copied().find(|g| g.name().eq_ignore_ascii_case(s) || g.short().eq_ignore_ascii_case(s))
    }

    /// Price per piece (a ladder is priced per rung tile, a plank per tile).
    pub fn cost(self) -> i32 {
        match self {
            Infra::Spring => 3,
            Infra::Ladder => 1,
            Infra::Plank => 2,
        }
    }

    pub fn blurb(self) -> &'static str {
        match self {
            Infra::Spring => "Jump 2 higher from here, even under a stone.",
            Infra::Ladder => "Click the top rung; it hangs to the floor. Apes climb with anything, others only light.",
            Infra::Plank => "A board to stand on or jump up through. It must touch ground or a plank.",
        }
    }
}

pub const MAX_LADDER: i32 = 12;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn loads_never_help() {
        for s in SPECIES {
            for w in [Weight::Light, Weight::Medium, Weight::Heavy] {
                let prev = WEIGHTS[w as usize - 1];
                assert!(s.jump(w) <= s.jump(prev), "{s:?} jumps higher under {w:?}");
                assert!(s.pace(w) >= s.pace(prev), "{s:?} walks faster under {w:?}");
            }
        }
    }

    #[test]
    fn tiers_grow() {
        for s in SPECIES {
            for t in 1..TIERS {
                assert!(residents(s, t) > residents(s, t - 1));
            }
        }
    }
}
