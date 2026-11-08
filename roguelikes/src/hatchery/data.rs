//! HATCHERY's bodies, genes and Grof's creatures. A familiar is a body plus one gene per
//! location (head, skin, legs, tail); it grows a new gene location at each level.

#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash, PartialOrd, Ord)]
pub enum Body {
    Frog,
    Toad,
    Newt,
    Salamander,
    Axolotl,
}

pub const ALL_BODIES: [Body; 5] = [Body::Frog, Body::Toad, Body::Newt, Body::Salamander, Body::Axolotl];

#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash, PartialOrd, Ord)]
pub enum Loc {
    Head,
    Skin,
    Legs,
    Tail,
}

impl Loc {
    pub fn name(self) -> &'static str {
        match self {
            Loc::Head => "HEAD",
            Loc::Skin => "SKIN",
            Loc::Legs => "LEGS",
            Loc::Tail => "TAIL",
        }
    }
}

pub struct BodyInfo {
    pub name: &'static str,
    pub hp: i32,
    /// Ticks per step.
    pub step: i32,
    pub bite: i32,
    /// Gene locations in the order they open up (one at hatching, one more per level).
    pub locs: &'static [Loc],
    pub note: &'static str,
}

impl Body {
    pub fn info(self) -> BodyInfo {
        use Loc::*;
        match self {
            Body::Frog => BodyInfo { name: "FROG", hp: 6, step: 3, bite: 2, locs: &[Head, Legs, Skin], note: "QUICK AND BALANCED. NO TAIL." },
            Body::Toad => BodyInfo { name: "TOAD", hp: 10, step: 5, bite: 2, locs: &[Skin, Head, Legs], note: "TOUGH AND SLOW. NO TAIL." },
            Body::Newt => BodyInfo { name: "NEWT", hp: 5, step: 2, bite: 1, locs: &[Tail, Head, Skin, Legs], note: "FAST AND FRAIL. ROOM FOR FOUR GENES." },
            Body::Salamander => BodyInfo { name: "SALAMANDER", hp: 7, step: 3, bite: 2, locs: &[Head, Tail, Skin, Legs], note: "A FIGHTER WITH ROOM FOR FOUR GENES." },
            Body::Axolotl => BodyInfo { name: "AXOLOTL", hp: 6, step: 3, bite: 1, locs: &[Tail, Skin, Head], note: "REGROWS 1 HP EVERY 6 TICKS IN WATER." },
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash, PartialOrd, Ord)]
pub enum Gene {
    LongTongue,
    WideMouth,
    VenomFangs,
    FireGland,
    ChorusSac,
    Warts,
    Slime,
    GlowSpots,
    BrightColors,
    BarkSkin,
    LeapingLegs,
    StickyPads,
    WebbedFeet,
    DiggingClaws,
    SplitTail,
    Regrowth,
    EggSac,
    ClubTail,
}

pub const ALL_GENES: [Gene; 18] = [
    Gene::LongTongue,
    Gene::WideMouth,
    Gene::VenomFangs,
    Gene::FireGland,
    Gene::ChorusSac,
    Gene::Warts,
    Gene::Slime,
    Gene::GlowSpots,
    Gene::BrightColors,
    Gene::BarkSkin,
    Gene::LeapingLegs,
    Gene::StickyPads,
    Gene::WebbedFeet,
    Gene::DiggingClaws,
    Gene::SplitTail,
    Gene::Regrowth,
    Gene::EggSac,
    Gene::ClubTail,
];

impl Gene {
    pub fn loc(self) -> Loc {
        use Gene::*;
        match self {
            LongTongue | WideMouth | VenomFangs | FireGland | ChorusSac => Loc::Head,
            Warts | Slime | GlowSpots | BrightColors | BarkSkin => Loc::Skin,
            LeapingLegs | StickyPads | WebbedFeet | DiggingClaws => Loc::Legs,
            SplitTail | Regrowth | EggSac | ClubTail => Loc::Tail,
        }
    }

    pub fn name(self) -> &'static str {
        match self {
            Gene::LongTongue => "LONG TONGUE",
            Gene::WideMouth => "WIDE MOUTH",
            Gene::VenomFangs => "VENOM FANGS",
            Gene::FireGland => "FIRE GLAND",
            Gene::ChorusSac => "CHORUS SAC",
            Gene::Warts => "WARTS",
            Gene::Slime => "SLIME",
            Gene::GlowSpots => "GLOW SPOTS",
            Gene::BrightColors => "BRIGHT COLORS",
            Gene::BarkSkin => "BARK SKIN",
            Gene::LeapingLegs => "LEAPING LEGS",
            Gene::StickyPads => "STICKY PADS",
            Gene::WebbedFeet => "WEBBED FEET",
            Gene::DiggingClaws => "DIGGING CLAWS",
            Gene::SplitTail => "SPLIT TAIL",
            Gene::Regrowth => "REGROWTH",
            Gene::EggSac => "EGG SAC",
            Gene::ClubTail => "CLUB TAIL",
        }
    }

    pub fn text(self) -> &'static str {
        match self {
            Gene::LongTongue => "BITES FROM 3 SQUARES AWAY, AND PULLS ITS PREY A SQUARE CLOSER.",
            Gene::WideMouth => "BITES 1 HARDER. ANYTHING LEFT WITH 3 HP OR LESS IS SWALLOWED WHOLE.",
            Gene::VenomFangs => "BITES POISON: 1 MORE DAMAGE EVERY 4 TICKS, 3 TIMES.",
            Gene::FireGland => "EVERY 12 TICKS, BREATHES FIRE: 2 TO EVERY FOE IN A LINE OF 3.",
            Gene::ChorusSac => "EVERY 10 TICKS, CROAKS: FAMILIARS WITHIN 2 SQUARES ACT AGAIN AT ONCE.",
            Gene::Warts => "WHATEVER BITES IT TAKES 1.",
            Gene::Slime => "THE FIRST HIT EACH NIGHT SLIDES OFF.",
            Gene::GlowSpots => "FAMILIARS WITHIN 2 BITE 1 HARDER. FOES NEARBY GO FOR IT FIRST.",
            Gene::BrightColors => "FOES WON'T BITE IT WHILE ANOTHER FAMILIAR IS IN REACH.",
            Gene::BarkSkin => "TAKES 1 LESS FROM EVERY BITE.",
            Gene::LeapingLegs => "LEAPS UP TO 3 SQUARES TOWARD ITS PREY IN ONE MOVE.",
            Gene::StickyPads => "CAN'T BE PUSHED OR PULLED, AND CLIMBS OVER OTHER FAMILIARS.",
            Gene::WebbedFeet => "MOVES TWICE AS FAST THROUGH WATER.",
            Gene::DiggingClaws => "STARTS THE NIGHT BURIED, WHERE NOTHING CAN REACH IT. ITS FIRST BITE DOES 2 MORE.",
            Gene::SplitTail => "ONCE A NIGHT, A KILLING BLOW LEAVES IT AT 1 HP INSTEAD.",
            Gene::Regrowth => "REGROWS 1 HP EVERY 4 TICKS.",
            Gene::EggSac => "WHEN IT FALLS, TWO TADPOLES HATCH WHERE IT WAS AND FIGHT ON.",
            Gene::ClubTail => "EACH BITE ALSO CLUBS ONE OTHER FOE NEXT TO IT FOR 1.",
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash, PartialOrd, Ord)]
pub enum Foe {
    Rat,
    Beetle,
    Crow,
    Hedgehog,
    Snake,
    Leech,
    Heron,
    Toadling,
    Grof,
}

pub struct FoeInfo {
    pub name: &'static str,
    pub hp: i32,
    pub step: i32,
    pub bite: i32,
    pub reach: i32,
    pub cooldown: i32,
    pub flies: bool,
    /// The gene you can learn by defeating it.
    pub gene: Gene,
    pub text: &'static str,
}

impl Foe {
    pub fn info(self) -> FoeInfo {
        let f = |name, hp, step, bite, reach, cooldown, flies, gene, text| FoeInfo { name, hp, step, bite, reach, cooldown, flies, gene, text };
        match self {
            Foe::Rat => f("RAT", 4, 2, 1, 1, 6, false, Gene::DiggingClaws, "QUICK. GOES FOR THE EGGS."),
            Foe::Beetle => f("BEETLE", 6, 4, 1, 1, 8, false, Gene::BarkSkin, "ARMORED: TAKES 1 LESS FROM EVERY BITE."),
            Foe::Crow => f("CROW", 4, 2, 1, 1, 6, true, Gene::LeapingLegs, "FLIES OVER YOUR FAMILIARS, STRAIGHT FOR THE EGGS."),
            Foe::Hedgehog => f("HEDGEHOG", 6, 4, 1, 1, 8, false, Gene::Warts, "SPINES: WHATEVER BITES IT TAKES 1."),
            Foe::Snake => f("WATER SNAKE", 7, 3, 2, 1, 8, false, Gene::VenomFangs, "VENOMOUS: ITS BITE KEEPS HURTING."),
            Foe::Leech => f("LEECH", 5, 3, 1, 1, 6, false, Gene::Regrowth, "REGROWS 1 HP EVERY 4 TICKS."),
            Foe::Heron => f("HERON", 10, 3, 3, 2, 10, false, Gene::LongTongue, "STABS FROM 2 SQUARES AWAY."),
            Foe::Toadling => f("TOADLING", 6, 3, 2, 1, 8, false, Gene::WideMouth, "ONE OF GROF'S OWN: A HUNGRY MOUTH."),
            Foe::Grof => f("GROF", 24, 4, 3, 2, 10, false, Gene::ChorusSac, "THE MALEVOLENT SORCERER. CALLS TOADLINGS EVERY 20 TICKS."),
        }
    }

    pub fn venom(self) -> bool {
        self == Foe::Snake
    }
}

/// Which foes come on each night (1-8), weighted.
pub fn night_foes(night: u32) -> Vec<Foe> {
    let mut v = vec![Foe::Rat, Foe::Beetle];
    if night >= 2 {
        v.push(Foe::Crow);
    }
    if night >= 3 {
        v.push(Foe::Hedgehog);
    }
    if night >= 4 {
        v.push(Foe::Snake);
    }
    if night >= 5 {
        v.push(Foe::Leech);
    }
    if night >= 6 {
        v.push(Foe::Heron);
    }
    if night >= 7 {
        v.push(Foe::Toadling);
    }
    v
}

pub const NIGHTS: u32 = 8;

pub fn night_size(night: u32) -> usize {
    [4, 5, 6, 7, 8, 9, 9, 6][(night.clamp(1, NIGHTS) - 1) as usize]
}

/// XP needed for levels 2, 3 and 4.
pub const LEVELS: [i32; 3] = [3, 7, 12];
pub const STAGES: [&str; 4] = ["TADPOLE", "FROGLET", "ADULT", "ELDER"];

pub const NAMES: [&str; 32] = [
    "PIP", "MOSS", "BLUB", "TWIG", "DEW", "FERN", "BOG", "REED", "PEBBLE", "SPROUT", "MUDDLE", "GLIM", "WISP", "BRAMBLE", "PUDDLE", "NETTLE", "SORREL",
    "BURDOCK", "CRESS", "TANSY", "LICHEN", "MALLOW", "RUSH", "SEDGE", "ALDER", "HAZEL", "TOADFLAX", "MINNOW", "CATKIN", "BRACKEN", "SILT", "MARSH",
];

/// A familiar as designed and raised.
#[derive(Clone, Debug, PartialEq)]
pub struct Familiar {
    pub name: &'static str,
    pub body: Body,
    pub genes: Vec<Gene>,
    pub xp: i32,
    pub level: i32,
}

impl Familiar {
    pub fn new(name: &'static str, body: Body, first: Gene) -> Self {
        Self { name, body, genes: vec![first], xp: 0, level: 1 }
    }

    pub fn has(&self, g: Gene) -> bool {
        self.genes.contains(&g)
    }

    pub fn max_hp(&self) -> i32 {
        self.body.info().hp + 2 * (self.level - 1)
    }

    /// The next gene location to open, if it has room to grow.
    pub fn next_loc(&self) -> Option<Loc> {
        self.body.info().locs.get(self.genes.len()).copied()
    }

    /// Level owed by its experience but not yet grown into.
    pub fn level_due(&self) -> bool {
        let lv = 1 + LEVELS.iter().filter(|&&x| self.xp >= x).count() as i32;
        lv > self.level && self.next_loc().is_some()
    }

    pub fn stage(&self) -> &'static str {
        STAGES[(self.level - 1).clamp(0, 3) as usize]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn familiars_grow_into_their_body_slots() {
        let mut f = Familiar::new("PIP", Body::Frog, Gene::LongTongue);
        assert_eq!(f.next_loc(), Some(Loc::Legs));
        f.xp = 3;
        assert!(f.level_due());
        f.genes.push(Gene::LeapingLegs);
        f.level = 2;
        f.genes.push(Gene::Warts);
        f.level = 3;
        f.xp = 99;
        assert!(!f.level_due(), "a frog has only three gene locations");
        for b in ALL_BODIES {
            for l in b.info().locs {
                assert!(ALL_GENES.iter().any(|g| g.loc() == *l));
            }
        }
    }
}
