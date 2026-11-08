//! Turncoat's people, ideas and ages.

#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash, PartialOrd, Ord)]
pub enum Kind {
    Gatherer,
    Hunter,
    Elder,
    Shaman,
    Smith,
    Rider,
    Herald,
    Scholar,
    Priest,
    Merchant,
    Broadcaster,
    Astronaut,
    Engineer,
    Chieftain,
    Warlord,
    Emperor,
    Director,
    Myth,
}

/// Where someone's voice carries. "Ahead" is toward the other side.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Reach {
    /// Fixed offsets, mirrored every way (a leap; nothing blocks it).
    Steps(&'static [(i32, i32)]),
    /// Straight ahead, up to n squares, stopping at the first person.
    Ahead(i32),
    /// Lines in these directions up to n squares, stopping at the first person.
    Lines(&'static [(i32, i32)], i32),
    /// Every square within n steps (any direction).
    Radius(i32),
    /// The whole column ahead; persuades everyone in it at once.
    Column,
}

pub const ORTH: [(i32, i32); 4] = [(1, 0), (-1, 0), (0, 1), (0, -1)];
pub const ALL8: [(i32, i32); 8] = [(1, 0), (-1, 0), (0, 1), (0, -1), (1, 1), (-1, 1), (1, -1), (-1, -1)];

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Trait {
    None,
    Rooted,
    Kindred,
    Sturdy,
    Swift,
    Faithful,
    Trader,
    DropIn,
    Builder,
    Rally,
    Awe,
}

pub struct Info {
    pub name: &'static str,
    pub resolve: i32,
    pub voice: i32,
    pub pace: i32,
    pub reach: Reach,
    pub traits: &'static [Trait],
    pub age: usize,
    pub leader: bool,
}

impl Kind {
    pub fn info(self) -> Info {
        use Trait::*;
        let i = |name, resolve, voice, pace, reach, traits, age| Info { name, resolve, voice, pace, reach, traits, age, leader: false };
        let l = |name, resolve, voice, pace, reach, traits, age| Info { name, resolve, voice, pace, reach, traits, age, leader: true };
        match self {
            Kind::Gatherer => i("GATHERER", 4, 1, 2, Reach::Steps(&[(1, 0)]), &[], 0),
            Kind::Hunter => i("HUNTER", 3, 2, 3, Reach::Ahead(2), &[], 0),
            Kind::Elder => i("ELDER", 3, 1, 1, Reach::Radius(2), &[Rooted], 0),
            Kind::Shaman => i("SHAMAN", 3, 1, 2, Reach::Steps(&[(1, 1)]), &[Kindred], 0),
            Kind::Smith => i("SMITH", 6, 1, 1, Reach::Steps(&[(1, 0), (1, 1)]), &[Sturdy], 1),
            Kind::Rider => i("RIDER", 3, 2, 4, Reach::Steps(&[(1, 2)]), &[Swift], 1),
            Kind::Herald => i("HERALD", 3, 1, 3, Reach::Column, &[], 1),
            Kind::Scholar => i("SCHOLAR", 3, 3, 2, Reach::Lines(&ORTH, 2), &[], 2),
            Kind::Priest => i("PRIEST", 4, 1, 1, Reach::Steps(&[(1, 0), (1, 1)]), &[Faithful], 2),
            Kind::Merchant => i("MERCHANT", 4, 1, 3, Reach::Steps(&[(1, 0)]), &[Trader], 2),
            Kind::Broadcaster => i("BROADCASTER", 3, 1, 2, Reach::Lines(&ORTH, 9), &[], 3),
            Kind::Astronaut => i("ASTRONAUT", 4, 2, 5, Reach::Steps(&[(2, 0), (2, 2)]), &[DropIn], 3),
            Kind::Engineer => i("ENGINEER", 5, 2, 2, Reach::Steps(&[(1, 0), (1, 1)]), &[Builder], 3),
            Kind::Chieftain => l("CHIEFTAIN", 9, 2, 2, Reach::Steps(&[(1, 0), (1, 1)]), &[Rally], 0),
            Kind::Warlord => l("WARLORD", 8, 3, 4, Reach::Steps(&[(1, 2)]), &[Swift, Sturdy], 1),
            Kind::Emperor => l("EMPEROR", 11, 2, 2, Reach::Radius(2), &[Faithful], 2),
            Kind::Director => l("DIRECTOR", 9, 2, 3, Reach::Lines(&ALL8, 9), &[Rally], 3),
            Kind::Myth => l("A MYTH", 15, 3, 3, Reach::Radius(2), &[Awe, Sturdy], 4),
        }
    }

    pub fn has(self, t: Trait) -> bool {
        self.info().traits.contains(&t)
    }

    pub fn reach_text(self) -> &'static str {
        match self.info().reach {
            Reach::Steps(&[(1, 0)]) => "ONE STEP, STRAIGHT",
            Reach::Steps(&[(1, 1)]) => "ONE STEP, DIAGONAL",
            Reach::Steps(&[(1, 0), (1, 1)]) => "ONE STEP, ANY WAY",
            Reach::Steps(&[(1, 2)]) => "A TWO-AND-ONE LEAP",
            Reach::Steps(_) => "EXACTLY TWO AWAY, ANY WAY",
            Reach::Ahead(_) => "1-2 SQUARES STRAIGHT AHEAD",
            Reach::Lines(d, n) if d.len() == 8 && n > 6 => "LINES, ANY LENGTH, ANY WAY",
            Reach::Lines(_, n) if n > 6 => "STRAIGHT LINES, ANY LENGTH",
            Reach::Lines(_, _) => "STRAIGHT LINES, UP TO 2",
            Reach::Radius(2) => "ANYONE WITHIN 2 STEPS",
            Reach::Radius(_) => "ANYONE WITHIN 3 STEPS",
            Reach::Column => "EVERYONE IN THE COLUMN AHEAD",
        }
    }
}

impl Trait {
    pub fn text(self) -> &'static str {
        match self {
            Trait::None => "",
            Trait::Rooted => "ROOTED: NEVER WALKS.",
            Trait::Kindred => "KINDRED: THOSE IT TURNS JOIN AT FULL RESOLVE.",
            Trait::Sturdy => "STURDY: TAKES 1 LESS FROM EVERY VOICE.",
            Trait::Swift => "SWIFT: WALKS 2 STEPS AT A TIME.",
            Trait::Faithful => "FAITHFUL: ITS NEIGHBOURS CAN'T BE TURNED (THEY HOLD AT 1).",
            Trait::Trader => "TRADER: TURNING SOMEONE RESTORES ITS OWN RESOLVE.",
            Trait::DropIn => "DROPS IN: CAN BE PLACED ANYWHERE THAT'S EMPTY.",
            Trait::Builder => "BUILDER: ITS NEIGHBOURS TAKE 1 LESS.",
            Trait::Rally => "RALLY: ITS NEIGHBOURS SPEAK 1 LOUDER.",
            Trait::Awe => "AWE: THOSE IT TURNS JOIN AT FULL RESOLVE.",
        }
    }
}

pub const AGES: [&str; 5] = ["PREHISTORY", "THE IRON AGE", "THE GOLDEN AGE", "THE SPACE AGE", "A MYTH"];
pub const LEADERS: [Kind; 5] = [Kind::Chieftain, Kind::Warlord, Kind::Emperor, Kind::Director, Kind::Myth];

pub fn age_pool(age: usize) -> Vec<(Kind, u32)> {
    let all = [
        Kind::Gatherer,
        Kind::Hunter,
        Kind::Elder,
        Kind::Shaman,
        Kind::Smith,
        Kind::Rider,
        Kind::Herald,
        Kind::Scholar,
        Kind::Priest,
        Kind::Merchant,
        Kind::Broadcaster,
        Kind::Astronaut,
        Kind::Engineer,
    ];
    all.iter()
        .filter(|k| k.info().age <= age.min(3))
        .map(|&k| {
            let a = k.info().age;
            let w = if a == age.min(3) { 5 } else if a + 1 == age.min(3) { 3 } else { 1 };
            (k, w)
        })
        .collect()
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash, PartialOrd, Ord)]
pub enum Idea {
    OralTradition,
    Bronze,
    Roads,
    Writing,
    Banners,
    BigTent,
    Rhetoric,
    Walls,
    Medicine,
}

pub const ALL_IDEAS: [Idea; 9] =
    [Idea::OralTradition, Idea::Bronze, Idea::Roads, Idea::Writing, Idea::Banners, Idea::BigTent, Idea::Rhetoric, Idea::Walls, Idea::Medicine];

impl Idea {
    pub fn name(self) -> &'static str {
        match self {
            Idea::OralTradition => "ORAL TRADITION",
            Idea::Bronze => "BRONZE",
            Idea::Roads => "ROADS",
            Idea::Writing => "WRITING",
            Idea::Banners => "BANNERS",
            Idea::BigTent => "BIG TENT",
            Idea::Rhetoric => "RHETORIC",
            Idea::Walls => "WALLS",
            Idea::Medicine => "MEDICINE",
        }
    }
    pub fn text(self) -> &'static str {
        match self {
            Idea::OralTradition => "YOUR ELDERS REACH 3 STEPS. YOUR HERALDS AND BROADCASTERS SPEAK 1 LOUDER.",
            Idea::Bronze => "ALL YOUR PEOPLE HAVE 2 MORE RESOLVE.",
            Idea::Roads => "ALL YOUR PEOPLE HAVE 1 MORE PACE AND WALK 2 STEPS AT A TIME.",
            Idea::Writing => "PEOPLE YOU TURN JOIN AT FULL RESOLVE.",
            Idea::Banners => "ONE MORE PLACE ON THE FIELD.",
            Idea::BigTent => "YOUR BAND HOLDS 2 MORE.",
            Idea::Rhetoric => "YOUR PEOPLE AT FULL RESOLVE SPEAK 1 LOUDER.",
            Idea::Walls => "YOUR BACK ROW TAKES 1 LESS FROM EVERY VOICE.",
            Idea::Medicine => "EACH ROUND, YOUR MOST SHAKEN PERSON REGAINS 1 RESOLVE.",
        }
    }
}
