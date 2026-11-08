//! Conduit's monsters and parts. Built on the player's Creature Builder notes: every monster
//! has held, worn and inner slots, and a body and mind capacity that its parts use up.

#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash, PartialOrd, Ord)]
pub enum Body {
    Kobold,
    Orc,
    Elf,
    Cyclops,
    Troll,
}

pub const ALL_BODIES: [Body; 5] = [Body::Kobold, Body::Orc, Body::Elf, Body::Cyclops, Body::Troll];

pub struct BodyInfo {
    pub name: &'static str,
    pub hp: i32,
    /// Ticks to walk one square.
    pub step: i32,
    pub held: usize,
    pub worn: usize,
    pub inner: usize,
    pub body: i32,
    pub mind: i32,
    pub note: &'static str,
}

impl Body {
    pub fn info(self) -> BodyInfo {
        match self {
            Body::Kobold => BodyInfo { name: "KOBOLD", hp: 8, step: 3, held: 1, worn: 1, inner: 1, body: 3, mind: 3, note: "SMALL AND QUICK." },
            Body::Orc => BodyInfo { name: "ORC", hp: 12, step: 4, held: 2, worn: 1, inner: 1, body: 6, mind: 2, note: "TWO HANDS FOR HELD GEAR." },
            Body::Elf => BodyInfo { name: "ELF", hp: 8, step: 3, held: 1, worn: 1, inner: 2, body: 3, mind: 6, note: "A BIG MIND, A SLIGHT BODY." },
            Body::Cyclops => BodyInfo { name: "CYCLOPS", hp: 18, step: 6, held: 1, worn: 2, inner: 1, body: 9, mind: 1, note: "HUGE, SLOW, CARRIES A LOT." },
            Body::Troll => BodyInfo { name: "TROLL", hp: 14, step: 5, held: 1, worn: 1, inner: 2, body: 6, mind: 2, note: "REGROWS 1 HP EVERY 10 TICKS." },
        }
    }

    pub fn slots(self) -> Vec<Slot> {
        let i = self.info();
        let mut v = vec![Slot::Held; i.held];
        v.extend(vec![Slot::Worn; i.worn]);
        v.extend(vec![Slot::Inner; i.inner]);
        v
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash)]
pub enum Slot {
    Held,
    Worn,
    Inner,
}

impl Slot {
    pub fn letter(self) -> &'static str {
        match self {
            Slot::Held => "H",
            Slot::Worn => "W",
            Slot::Inner => "I",
        }
    }
    pub fn name(self) -> &'static str {
        match self {
            Slot::Held => "HELD",
            Slot::Worn => "WORN",
            Slot::Inner => "INNER",
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash, PartialOrd, Ord)]
pub enum Part {
    Club,
    Spear,
    Axe,
    Sling,
    Bow,
    SparkWand,
    Torch,
    MendingStaff,
    Shield,
    Leather,
    Plate,
    Cloak,
    WardCharm,
    Boots,
    Spikes,
    SecondHeart,
    Fang,
    QuickNerves,
    StoneGut,
    HunterMind,
    GuardMind,
    RaiderMind,
}

pub const ALL_PARTS: [Part; 22] = [
    Part::Club,
    Part::Spear,
    Part::Axe,
    Part::Sling,
    Part::Bow,
    Part::SparkWand,
    Part::Torch,
    Part::MendingStaff,
    Part::Shield,
    Part::Leather,
    Part::Plate,
    Part::Cloak,
    Part::WardCharm,
    Part::Boots,
    Part::Spikes,
    Part::SecondHeart,
    Part::Fang,
    Part::QuickNerves,
    Part::StoneGut,
    Part::HunterMind,
    Part::GuardMind,
    Part::RaiderMind,
];

#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash)]
pub enum Dmg {
    Melee,
    Ranged,
    Magic,
}

impl Dmg {
    pub fn name(self) -> &'static str {
        match self {
            Dmg::Melee => "MELEE",
            Dmg::Ranged => "RANGED",
            Dmg::Magic => "MAGIC",
        }
    }
    pub fn defense(self) -> &'static str {
        match self {
            Dmg::Melee => "ARMOR",
            Dmg::Ranged => "DODGE",
            Dmg::Magic => "WARD",
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Attack {
    pub dmg_type: Dmg,
    pub dmg: i32,
    pub range: i32,
    pub cooldown: i32,
    /// Also hits enemies next to the target.
    pub burst: bool,
    /// Also hits every other enemy next to the attacker.
    pub sweep: bool,
    /// Sets the target burning for this many points (1 HP every 4 ticks).
    pub burn: i32,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Brain {
    Brute,
    Hunter,
    Guard,
    Raider,
}

impl Brain {
    pub fn text(self) -> &'static str {
        match self {
            Brain::Brute => "FIGHTS THE NEAREST ENEMY, THEN THE CONDUIT.",
            Brain::Hunter => "GOES FOR THE ENEMY WITH THE LEAST HP.",
            Brain::Guard => "STAYS NEAR ITS OWN CONDUIT AND FIGHTS WHAT COMES CLOSE.",
            Brain::Raider => "HEADS STRAIGHT FOR THE ENEMY CONDUIT.",
        }
    }
}

pub struct PartInfo {
    pub name: &'static str,
    pub slot: Slot,
    pub body: i32,
    pub mind: i32,
    pub text: &'static str,
}

impl Part {
    pub fn info(self) -> PartInfo {
        use Slot::*;
        let p = |name, slot, body, mind, text| PartInfo { name, slot, body, mind, text };
        match self {
            Part::Club => p("CLUB", Held, 2, 0, "MELEE 3, REACH 1, EVERY 8 TICKS."),
            Part::Spear => p("SPEAR", Held, 2, 0, "MELEE 2, REACH 2, EVERY 7 TICKS."),
            Part::Axe => p("AXE", Held, 3, 0, "MELEE 3, REACH 1, EVERY 10 TICKS. SWEEP: ALSO HITS EVERY OTHER ENEMY NEXT TO IT."),
            Part::Sling => p("SLING", Held, 1, 0, "RANGED 2, REACH 4, EVERY 8 TICKS."),
            Part::Bow => p("BOW", Held, 1, 1, "RANGED 2, REACH 6, EVERY 8 TICKS."),
            Part::SparkWand => p("SPARK WAND", Held, 0, 3, "MAGIC 2, REACH 4, EVERY 12 TICKS. BURST: ALSO HITS ENEMIES NEXT TO THE TARGET."),
            Part::Torch => p("TORCH", Held, 1, 1, "MELEE 1, REACH 1, EVERY 8 TICKS. SETS THE TARGET BURNING: 3 MORE DAMAGE OVER 12 TICKS, NO DEFENSE."),
            Part::MendingStaff => p("MENDING STAFF", Held, 0, 3, "HEALS THE MOST HURT ALLY WITHIN 3 BY 3, EVERY 12 TICKS."),
            Part::Shield => p("SHIELD", Held, 2, 0, "+2 ARMOR."),
            Part::Leather => p("LEATHER", Worn, 1, 0, "+1 ARMOR."),
            Part::Plate => p("PLATE", Worn, 3, 0, "+3 ARMOR, BUT 1 TICK SLOWER PER STEP."),
            Part::Cloak => p("CLOAK", Worn, 1, 0, "+2 DODGE."),
            Part::WardCharm => p("WARD CHARM", Worn, 0, 1, "+2 WARD."),
            Part::Boots => p("BOOTS", Worn, 1, 0, "1 TICK FASTER PER STEP."),
            Part::Spikes => p("SPIKES", Worn, 2, 0, "WHOEVER HITS IT IN MELEE TAKES 1."),
            Part::SecondHeart => p("SECOND HEART", Inner, 2, 0, "+6 HP."),
            Part::Fang => p("FANG", Inner, 0, 2, "HEALS HALF THE MELEE DAMAGE IT DEALS."),
            Part::QuickNerves => p("QUICK NERVES", Inner, 1, 1, "EVERY ATTACK COMES 2 TICKS SOONER."),
            Part::StoneGut => p("STONE GUT", Inner, 3, 0, "TAKES HALF DAMAGE (ROUNDED UP)."),
            Part::HunterMind => p("HUNTER MIND", Inner, 0, 1, "MIND: GOES FOR THE ENEMY WITH THE LEAST HP."),
            Part::GuardMind => p("GUARD MIND", Inner, 0, 1, "MIND: STAYS NEAR ITS OWN CONDUIT AND FIGHTS WHAT COMES CLOSE."),
            Part::RaiderMind => p("RAIDER MIND", Inner, 0, 2, "MIND: HEADS STRAIGHT FOR THE ENEMY CONDUIT."),
        }
    }

    pub fn attack(self) -> Option<Attack> {
        let a = |dmg_type, dmg, range, cooldown| Attack { dmg_type, dmg, range, cooldown, burst: false, sweep: false, burn: 0 };
        Some(match self {
            Part::Club => a(Dmg::Melee, 3, 1, 8),
            Part::Spear => a(Dmg::Melee, 2, 2, 7),
            Part::Axe => Attack { sweep: true, ..a(Dmg::Melee, 3, 1, 10) },
            Part::Sling => a(Dmg::Ranged, 2, 4, 8),
            Part::Bow => a(Dmg::Ranged, 2, 6, 8),
            Part::SparkWand => Attack { burst: true, ..a(Dmg::Magic, 2, 4, 12) },
            Part::Torch => Attack { burn: 3, ..a(Dmg::Melee, 1, 1, 8) },
            _ => return None,
        })
    }
}

pub const FISTS: Attack = Attack { dmg_type: Dmg::Melee, dmg: 1, range: 1, cooldown: 8, burst: false, sweep: false, burn: 0 };

/// Everything a monster brings to a fight, from its body and parts.
#[derive(Clone, Debug)]
pub struct Stats {
    pub max_hp: i32,
    pub step: i32,
    pub armor: i32,
    pub dodge: i32,
    pub ward: i32,
    pub attacks: Vec<Attack>,
    pub heal: bool,
    pub lifelink: bool,
    pub spikes: bool,
    pub stone_gut: bool,
    pub regen: bool,
    pub quick: bool,
    pub brain: Brain,
}

pub fn stats(body: Body, parts: &[Part]) -> Stats {
    let b = body.info();
    let mut s = Stats {
        max_hp: b.hp,
        step: b.step,
        armor: 0,
        dodge: 0,
        ward: 0,
        attacks: Vec::new(),
        heal: false,
        lifelink: false,
        spikes: false,
        stone_gut: false,
        regen: body == Body::Troll,
        quick: false,
        brain: Brain::Brute,
    };
    for &p in parts {
        match p {
            Part::Shield => s.armor += 2,
            Part::Leather => s.armor += 1,
            Part::Plate => {
                s.armor += 3;
                s.step += 1;
            }
            Part::Cloak => s.dodge += 2,
            Part::WardCharm => s.ward += 2,
            Part::Boots => s.step -= 1,
            Part::Spikes => s.spikes = true,
            Part::SecondHeart => s.max_hp += 6,
            Part::Fang => s.lifelink = true,
            Part::QuickNerves => s.quick = true,
            Part::StoneGut => s.stone_gut = true,
            Part::MendingStaff => s.heal = true,
            Part::HunterMind => s.brain = Brain::Hunter,
            Part::GuardMind => s.brain = Brain::Guard,
            Part::RaiderMind => s.brain = Brain::Raider,
            _ => {}
        }
        if let Some(a) = p.attack() {
            s.attacks.push(a);
        }
    }
    s.step = s.step.max(1);
    if s.attacks.is_empty() {
        s.attacks.push(FISTS);
    }
    if s.quick {
        for a in s.attacks.iter_mut() {
            a.cooldown = (a.cooldown - 2).max(2);
        }
    }
    s
}

/// A monster as built: a body and the parts in its slots (in slot order).
#[derive(Clone, Debug, PartialEq)]
pub struct Build {
    pub body: Body,
    pub parts: Vec<Option<Part>>,
}

impl Build {
    pub fn new(body: Body) -> Self {
        Self { body, parts: vec![None; body.slots().len()] }
    }

    pub fn fitted(&self) -> Vec<Part> {
        self.parts.iter().flatten().copied().collect()
    }

    pub fn used(&self) -> (i32, i32) {
        self.fitted().iter().fold((0, 0), |(b, m), p| (b + p.info().body, m + p.info().mind))
    }

    /// Why `part` can't go in slot `i` (None if it can). Any part already there is swapped out.
    pub fn refusal(&self, i: usize, part: Part) -> Option<String> {
        let slots = self.body.slots();
        let info = part.info();
        let b = self.body.info();
        let slot = *slots.get(i)?;
        if slot != info.slot {
            return Some(format!("{} GOES IN A {} SLOT, NOT {}.", info.name, info.slot.name(), slot.name()));
        }
        if info.slot == Slot::Inner && part_is_mind(part) && self.parts.iter().enumerate().any(|(j, p)| j != i && p.is_some_and(part_is_mind)) {
            return Some(format!("THE {} ALREADY HAS A MIND PART.", b.name));
        }
        let (mut ub, mut um) = self.used();
        if let Some(old) = self.parts[i] {
            ub -= old.info().body;
            um -= old.info().mind;
        }
        if ub + info.body > b.body {
            return Some(format!("NOT ENOUGH BODY: {} NEEDS {}, THE {} HAS {} LEFT.", info.name, info.body, b.name, b.body - ub));
        }
        if um + info.mind > b.mind {
            return Some(format!("NOT ENOUGH MIND: {} NEEDS {}, THE {} HAS {} LEFT.", info.name, info.mind, b.name, b.mind - um));
        }
        None
    }

    pub fn stats(&self) -> Stats {
        stats(self.body, &self.fitted())
    }
}

pub fn part_is_mind(p: Part) -> bool {
    matches!(p, Part::HunterMind | Part::GuardMind | Part::RaiderMind)
}

/// The run's stages, each with a piece of The Marvelous Electrical World of Tomorrow.
pub const DISTRICTS: [&str; 5] = ["MAPLE STREET", "THE INVENTOR MUSEUM", "TRANSFORMER JUNCTION NO. 9", "THE STARBRIDGE", "THE TRANSOCEANIC RAILWAY"];
pub const CONTESTS: usize = 8;

pub fn district_of(contest: usize) -> usize {
    match contest {
        0 | 1 => 0,
        2 | 3 => 1,
        4 | 5 => 2,
        6 => 3,
        _ => 4,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fitting_respects_slots_and_capacity() {
        let mut k = Build::new(Body::Kobold); // H W I, body 3 mind 3
        assert!(k.refusal(0, Part::Leather).unwrap().contains("WORN SLOT, NOT HELD"));
        assert!(k.refusal(0, Part::Club).is_none());
        k.parts[0] = Some(Part::Club);
        assert!(k.refusal(1, Part::Plate).unwrap().contains("NOT ENOUGH BODY"));
        assert!(k.refusal(1, Part::Leather).is_none());
        // Swapping the club for a lighter sling frees body.
        k.parts[1] = Some(Part::Leather);
        assert!(k.refusal(0, Part::Sling).is_none());
        let mut e = Build::new(Body::Elf);
        e.parts[2] = Some(Part::HunterMind);
        assert!(e.refusal(3, Part::GuardMind).unwrap().contains("MIND PART"));
    }

    #[test]
    fn stats_add_up() {
        let s = stats(Body::Orc, &[Part::Club, Part::Shield, Part::Plate, Part::QuickNerves]);
        assert_eq!(s.armor, 5);
        assert_eq!(s.step, 5);
        assert_eq!(s.attacks[0].cooldown, 6);
        assert_eq!(stats(Body::Kobold, &[]).attacks, vec![FISTS]);
    }
}
