//! The fixed rules: creatures, goods, weights, recipes and prices. Everything the player has to
//! learn lives here, so the rules page, tooltips and the simulation all read the same numbers.

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
    pub fn name(self) -> &'static str {
        match self {
            Weight::Empty => "EMPTY",
            Weight::Light => "LIGHT",
            Weight::Medium => "MEDIUM",
            Weight::Heavy => "HEAVY",
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash, PartialOrd, Ord)]
pub enum Good {
    Berry,
    Twig,
    Stone,
    Clay,
    Egg,
    Brick,
    Wicker,
    Pot,
}

pub const GOODS: [Good; 8] = [Good::Berry, Good::Twig, Good::Stone, Good::Clay, Good::Egg, Good::Brick, Good::Wicker, Good::Pot];

impl Good {
    pub fn weight(self) -> Weight {
        match self {
            Good::Berry | Good::Egg | Good::Wicker | Good::Pot => Weight::Light,
            Good::Twig | Good::Clay | Good::Brick => Weight::Medium,
            Good::Stone => Weight::Heavy,
        }
    }

    /// Fragile goods break if their carrier falls more than `FRAGILE_FALL` tiles.
    pub fn fragile(self) -> bool {
        matches!(self, Good::Egg | Good::Pot)
    }

    pub fn name(self) -> &'static str {
        match self {
            Good::Berry => "BERRY",
            Good::Twig => "TWIG",
            Good::Stone => "STONE",
            Good::Clay => "CLAY",
            Good::Egg => "EGG",
            Good::Brick => "BRICK",
            Good::Wicker => "WICKER",
            Good::Pot => "POT",
        }
    }

    pub fn plural(self) -> &'static str {
        match self {
            Good::Berry => "BERRIES",
            Good::Twig => "TWIGS",
            Good::Stone => "STONES",
            Good::Clay => "CLAY",
            Good::Egg => "EGGS",
            Good::Brick => "BRICKS",
            Good::Wicker => "WICKER",
            Good::Pot => "POTS",
        }
    }

    pub fn index(self) -> usize {
        self as usize
    }

    pub fn from_name(s: &str) -> Option<Good> {
        GOODS.iter().copied().find(|g| g.name().eq_ignore_ascii_case(s))
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
}

/// The highest fall (in tiles) a creature may take while carrying something fragile.
pub const FRAGILE_FALL: i32 = 1;
/// Ticks a creature spends at a dock to take or hand over one good.
pub const PICKUP_TICKS: i32 = 2;
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

    pub fn from_name(s: &str) -> Option<Species> {
        SPECIES.iter().copied().find(|g| g.name().eq_ignore_ascii_case(s))
    }

    pub fn cost(self) -> i32 {
        match self {
            Species::Tribble => 2,
            Species::Dodo => 3,
            Species::Ape => 5,
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
            Species::Tribble => "CHEAP AND QUICK. LEAPS HIGH WHEN LIGHT, BUT A TWIG BOGS IT DOWN AND A STONE NEARLY STOPS IT.",
            Species::Dodo => "STEADY. KEEPS ITS JUMP WITH A TWIG ON ITS BACK.",
            Species::Ape => "STRONG AND DEAR. CLIMBS VINES AND LADDERS WITH ANY LOAD.",
        }
    }
}

/// Buildings the player can place. Each is two tiles wide and two tall.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash, PartialOrd, Ord)]
pub enum Kind {
    Depot,
    Mason,
    Weaver,
    Kiln,
}

pub const KINDS: [Kind; 4] = [Kind::Depot, Kind::Mason, Kind::Weaver, Kind::Kiln];

pub struct Recipe {
    pub inputs: &'static [(Good, i32)],
    pub output: Good,
    pub ticks: i32,
}

pub const MASON: Recipe = Recipe { inputs: &[(Good::Stone, 1)], output: Good::Brick, ticks: 8 };
pub const WEAVER: Recipe = Recipe { inputs: &[(Good::Twig, 2)], output: Good::Wicker, ticks: 10 };
pub const KILN: Recipe = Recipe { inputs: &[(Good::Clay, 1), (Good::Twig, 1)], output: Good::Pot, ticks: 12 };

/// A depot holds this many goods in total, of any mix.
pub const DEPOT_CAP: i32 = 6;
/// A workshop holds up to this many of each input and of its output.
pub const SHOP_IN_CAP: i32 = 4;
pub const SHOP_OUT_CAP: i32 = 3;

impl Kind {
    pub fn name(self) -> &'static str {
        match self {
            Kind::Depot => "DEPOT",
            Kind::Mason => "MASON",
            Kind::Weaver => "WEAVER",
            Kind::Kiln => "KILN",
        }
    }

    pub fn from_name(s: &str) -> Option<Kind> {
        KINDS.iter().copied().find(|g| g.name().eq_ignore_ascii_case(s))
    }

    pub fn cost(self) -> i32 {
        match self {
            Kind::Depot => 3,
            Kind::Mason => 5,
            Kind::Weaver => 4,
            Kind::Kiln => 6,
        }
    }

    pub fn recipe(self) -> Option<&'static Recipe> {
        match self {
            Kind::Depot => None,
            Kind::Mason => Some(&MASON),
            Kind::Weaver => Some(&WEAVER),
            Kind::Kiln => Some(&KILN),
        }
    }

    pub fn blurb(self) -> &'static str {
        match self {
            Kind::Depot => "HOLDS UP TO 6 GOODS OF ANY KIND. A HANDOFF POINT BETWEEN ROUTES.",
            Kind::Mason => "DRESSES 1 STONE INTO 1 BRICK IN 8 TICKS.",
            Kind::Weaver => "WEAVES 2 TWIGS INTO 1 WICKER IN 10 TICKS.",
            Kind::Kiln => "FIRES 1 CLAY WITH 1 TWIG INTO 1 POT IN 12 TICKS.",
        }
    }
}

/// Terrain the player can add.
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

    pub fn from_name(s: &str) -> Option<Infra> {
        INFRAS.iter().copied().find(|g| g.name().eq_ignore_ascii_case(s) || g.short().eq_ignore_ascii_case(s))
    }

    /// Price per piece (a ladder is priced per rung tile, a plank per tile).
    pub fn cost(self) -> i32 {
        match self {
            Infra::Spring => 2,
            Infra::Ladder => 1,
            Infra::Plank => 2,
        }
    }

    pub fn blurb(self) -> &'static str {
        match self {
            Infra::Spring => "ANYONE WHO JUMPS FROM IT JUMPS 2 HIGHER, EVEN UNDER A STONE.",
            Infra::Ladder => "HANGS DOWN TO THE GROUND. 1 SEED PER RUNG. APES CLIMB IT WITH ANYTHING, OTHERS ONLY LIGHT.",
            Infra::Plank => "A ONE-WAY BOARD: JUMP UP THROUGH IT, STAND ON IT. MUST TOUCH GROUND OR ANOTHER PLANK.",
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
}
