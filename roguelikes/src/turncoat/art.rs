//! Turncoat's palettes (one per age) and people. Coats are drawn in a placeholder team ramp and
//! remapped to blue or red at blit time, so turning someone is a palette swap.

use super::data::Kind;
use crate::changeling::art::outlined;
use crate::framebuffer::Sprite;
use crate::palette::*;

pub const BG: u8 = 2; // 4
pub const GROUND_L: u8 = 6;
pub const GROUND_D: u8 = 7;
pub const GROUND_X: u8 = 8; // 2: details
pub const TEAM: u8 = 10; // 3 placeholder
pub const BLUE: u8 = 13; // 3
pub const RED: u8 = 16; // 3
pub const SKIN: u8 = 19; // 3
pub const HAIR: u8 = 22; // 3 (dark hair, hair, wood)
pub const METAL: u8 = 25; // 3
pub const CLOTH: u8 = 28; // 3
pub const GOLD_R: u8 = 31; // 3
pub const LEAF: u8 = 34; // 3
pub const DARK: u8 = 37; // 3
pub const VISOR: u8 = 40; // 3
pub const LIT: u8 = 43;
pub const MIDLINE: u8 = 44;

/// Ground colours per age: savanna, packed earth, marble, deck plating, starfield.
const GROUNDS: [((u8, u8, u8), (u8, u8, u8), (u8, u8, u8), (u8, u8, u8)); 5] = [
    ((112, 104, 56), (96, 88, 48), (132, 124, 64), (76, 104, 48)),
    ((100, 76, 52), (84, 64, 44), (120, 92, 64), (64, 56, 52)),
    ((168, 160, 144), (144, 136, 124), (188, 180, 164), (120, 112, 100)),
    ((72, 80, 92), (60, 68, 80), (96, 104, 116), (40, 200, 220)),
    ((28, 24, 52), (20, 18, 40), (60, 52, 100), (200, 180, 240)),
];

pub fn palette(age: usize) -> Pal {
    let mut p = [argb((0, 0, 0)); 64];
    let (l, d, x1, x2) = GROUNDS[age.min(4)];
    ramp(&mut p, BG, 4, (8, 8, 16), (34, 30, 44));
    set(&mut p, GROUND_L, l);
    set(&mut p, GROUND_D, d);
    set(&mut p, GROUND_X, x1);
    set(&mut p, GROUND_X + 1, x2);
    ramp3(&mut p, TEAM, (90, 90, 90), (150, 150, 150), (210, 210, 210));
    ramp3(&mut p, BLUE, (28, 56, 140), (56, 108, 220), (140, 184, 252));
    ramp3(&mut p, RED, (128, 28, 32), (208, 60, 52), (252, 144, 116));
    ramp3(&mut p, SKIN, (164, 104, 72), (216, 156, 112), (240, 196, 156));
    ramp3(&mut p, HAIR, (52, 32, 20), (100, 64, 36), (148, 100, 56));
    ramp3(&mut p, METAL, (84, 88, 100), (152, 156, 168), (216, 220, 228));
    ramp3(&mut p, CLOTH, (176, 172, 164), (220, 216, 208), (252, 252, 248));
    ramp3(&mut p, GOLD_R, (168, 120, 24), (232, 184, 48), (252, 232, 128));
    ramp3(&mut p, LEAF, (40, 104, 48), (80, 164, 72), (160, 220, 120));
    ramp3(&mut p, DARK, (36, 32, 40), (64, 56, 72), (96, 88, 104));
    ramp3(&mut p, VISOR, (24, 80, 120), (64, 160, 208), (176, 232, 252));
    set(&mut p, LIT, (l.0 / 2 + 110, l.1 / 2 + 110, l.2 / 2 + 110));
    set(&mut p, MIDLINE, (d.0 / 2, d.1 / 2, d.2 / 2));
    ui_block(
        &mut p,
        &UiTheme {
            panel: [(52, 40, 32), (84, 66, 50), (132, 108, 80), (30, 24, 20)],
            text: (244, 232, 204),
            text_dim: (184, 168, 136),
            text_faint: (124, 112, 92),
            accent: (248, 204, 96),
            accent_hi: (252, 236, 168),
        },
    );
    p
}

pub fn team_map(blue: bool) -> [u8; 64] {
    let mut m = crate::framebuffer::IDENTITY;
    let to = if blue { BLUE } else { RED };
    for s in 0..3 {
        m[(TEAM + s) as usize] = to + s;
    }
    m
}

/// Everything in one flat colour except the outline (for the dragged ghost).
pub fn flat(col: u8) -> [u8; 64] {
    let mut m = [col; 64];
    m[0] = 0;
    m[OUTLINE as usize] = OUTLINE;
    m
}

fn key(c: char) -> u8 {
    match c {
        't' => TEAM,
        'T' => TEAM + 1,
        'U' => TEAM + 2,
        'z' => SKIN,
        's' => SKIN + 1,
        'Z' => SKIN + 2,
        'h' => HAIR,
        'H' => HAIR + 1,
        'w' => HAIR + 2,
        'm' => METAL,
        'M' => METAL + 1,
        'N' => METAL + 2,
        'c' => CLOTH,
        'C' => CLOTH + 1,
        'y' => GOLD_R,
        'Y' => GOLD_R + 1,
        'g' => LEAF + 1,
        'd' => DARK,
        'D' => DARK + 1,
        'v' => VISOR,
        'V' => VISOR + 1,
        'e' | 'k' => OUTLINE,
        'W' => WHITE,
        _ => 0,
    }
}

fn person(rows: &[&str]) -> Sprite {
    outlined(Sprite::art(rows, key))
}

/// A leader: the base figure with a crown above its head.
fn crowned(rows: &[&str]) -> Sprite {
    let mut v: Vec<String> = vec!["....y.y.y.....".into(), "....yYyYy.....".into()];
    v.extend(rows.iter().map(|r| r.to_string()));
    let refs: Vec<&str> = v.iter().map(|s| s.as_str()).collect();
    person(&refs)
}

pub struct Art {
    pub people: Vec<(Kind, Sprite)>,
}

impl Art {
    pub fn new() -> Self {
        let people = vec![
            (Kind::Gatherer, person(&GATHERER)),
            (Kind::Hunter, person(&HUNTER)),
            (Kind::Elder, person(&ELDER)),
            (Kind::Shaman, person(&SHAMAN)),
            (Kind::Smith, person(&SMITH)),
            (Kind::Rider, person(&RIDER)),
            (Kind::Herald, person(&HERALD)),
            (Kind::Scholar, person(&SCHOLAR)),
            (Kind::Priest, person(&PRIEST)),
            (Kind::Merchant, person(&MERCHANT)),
            (Kind::Broadcaster, person(&BROADCASTER)),
            (Kind::Astronaut, person(&ASTRONAUT)),
            (Kind::Engineer, person(&ENGINEER)),
            (Kind::Chieftain, crowned(&SMITH)),
            (Kind::Warlord, crowned(&RIDER)),
            (Kind::Emperor, crowned(&ELDER)),
            (Kind::Director, crowned(&BROADCASTER)),
            (Kind::Myth, crowned(&MYTH)),
        ];
        Self { people }
    }

    pub fn of(&self, k: Kind) -> &Sprite {
        &self.people.iter().find(|(kk, _)| *kk == k).unwrap().1
    }
}

const GATHERER: [&str; 14] = [
    "....HHHH......",
    "...HHHHHH.....",
    "...sesses.....",
    "...ssssss.....",
    "....ssss......",
    "..tTTTTTTt....",
    ".tTTTTTTTTt...",
    ".tTTTTTTTTt...",
    ".sTTTTTTTTsww.",
    "..tTTTTTTtwwww",
    "..tTTttTTtwwww",
    "...dd..dd..ww.",
    "...dd..dd.....",
    "..ddd..ddd....",
];

const HUNTER: [&str; 15] = [
    "...........m..",
    "....HHHH..mMm.",
    "...HHHHHH..w..",
    "...sesses..w..",
    "...ssssss..w..",
    "....ssss...w..",
    "..tTTTTTTt.w..",
    ".tTTTTTTTTtw..",
    ".tTTTTTTTTsw..",
    ".sTTTTTTTT.w..",
    "..tTTTTTTt.w..",
    "..tTTttTTt.w..",
    "...dd..dd..w..",
    "...dd..dd..w..",
    "..ddd..ddd.w..",
];

const ELDER: [&str; 15] = [
    "..w...cccc....",
    ".www.cccccc...",
    "..w..sesses...",
    "..w..ssssss...",
    "..w..cccccc...",
    "..w...cccc....",
    "..w.tTccccTt..",
    "..wtTTTccTTTt.",
    "..sTTTTTTTTTt.",
    "..wtTTTTTTTTt.",
    "..w.tTTTTTTt..",
    "..w.tTTTTTTt..",
    "..w.tTTTTTTt..",
    "..w.tTTTTTTt..",
    "..w..dd..dd...",
];

const SHAMAN: [&str; 15] = [
    ".g.........g..",
    ".gg..yyyy..gg.",
    "..gwwwwwwwwg..",
    "...wkwwwkw....",
    "...wwwwwww....",
    "....wwwww.....",
    "..tTTTTTTt....",
    ".tTTTgTTTTt...",
    ".tTTgTgTTTt...",
    ".sTTTTTTTTs...",
    "..tTTTTTTt....",
    "..tTTttTTt....",
    "...dd..dd.....",
    "...dd..dd.....",
    "..ddd..ddd....",
];

const SMITH: [&str; 14] = [
    "....HHHH......",
    "...ssssss..mm.",
    "...sesses.mMMm",
    "...ssssss..w..",
    "...HHHHHH..w..",
    ".tTTTTTTTTsw..",
    "tTTTddddTTTw..",
    "tTTTddddTTT...",
    "sTTTddddTTs...",
    ".tTTddddTTt...",
    ".tTTddddTTt...",
    "..dddddddd....",
    "..dd....dd....",
    ".ddd....ddd...",
];

const RIDER: [&str; 15] = [
    ".....TT.......",
    "....TTT.......",
    "...mmmmmm.....",
    "...mMMMMm.....",
    "...sesses.....",
    "...ssssss.....",
    "..mtTTTTtm....",
    ".mMMTTTTMMm...",
    ".mMyMTTTTTt...",
    ".mMMmTTTTTs...",
    "..mm.TTTTt....",
    "..tTTttTTt....",
    "...dd..dd.....",
    "...dd..dd.....",
    "..mmm..mmm....",
];

const HERALD: [&str; 14] = [
    "....HHHH......",
    "...HHHHHH.....",
    "...sesses.yy..",
    "...ssssssyYy..",
    "....ssssyYy...",
    "..tTTTTsYy....",
    ".tTTTTTTs.....",
    ".tTTTTTTTt....",
    ".sTTTTTTTt....",
    "..tTTTTTTt....",
    "..tTTttTTt....",
    "...dd..dd.....",
    "...dd..dd.....",
    "..ddd..ddd....",
];

const SCHOLAR: [&str; 15] = [
    "..dddddddd....",
    "....dddd......",
    "...HHHHHH.....",
    "...sesses.....",
    "...ssssss.....",
    "....ssss......",
    "..tTTTTTTt....",
    ".tTTTTTTTTt...",
    ".tTTTTTTTTcc..",
    ".sTTTTTTTscCc.",
    "..tTTTTTTtcc..",
    "..tTTTTTTt....",
    "..tTTTTTTt....",
    "...dd..dd.....",
    "..ddd..ddd....",
];

const PRIEST: [&str; 15] = [
    ".....cc.......",
    "....cyyc......",
    "....cyyc......",
    "...ccyycc.....",
    "...sesses.....",
    "...ssssss.....",
    "..tTTccTTt....",
    ".tTTTccTTTt...",
    ".tTTTccTTTt...",
    ".sTTTccTTTs...",
    "..tTTccTTt....",
    "..tTTccTTt....",
    "..tTTccTTt....",
    "..tTTTTTTt....",
    "...dd..dd.....",
];

const MERCHANT: [&str; 14] = [
    "....HHHH......",
    "..HHHHHHHH....",
    "...sesses.....",
    "...ssssss.....",
    "....ssss......",
    "..tTTTTTTt....",
    ".tTTTTTTTTt...",
    ".tTTyTTTTTt...",
    ".sTTTTTTTTsyy.",
    "..tTTTTTTtyYYy",
    "..tTTttTTtyYYy",
    "...dd..dd..yy.",
    "...dd..dd.....",
    "..ddd..ddd....",
];

const BROADCASTER: [&str; 15] = [
    ".......m......",
    ".......m......",
    "....HHHHm.....",
    "...HHHHHHm....",
    "..msesses.....",
    "..mssssssm....",
    "...mssss.m....",
    "..tTTTTTTt....",
    ".tTTTTTTTTt...",
    ".tTTTTTTTTt...",
    ".sTTTTTTTTs...",
    "..tTTTTTTt....",
    "..tTTttTTt....",
    "...dd..dd.....",
    "..ddd..ddd....",
];

const ASTRONAUT: [&str; 14] = [
    "...cccccc.....",
    "..cvvvvvvc....",
    ".cvVsesVvc....",
    ".cvsssssvc....",
    ".cvvvvvvvc....",
    "..cccccccc....",
    ".ccTTTTTTcc...",
    "cccTTTTTTccc..",
    "cc.cccccc.cc..",
    "cc.cTTTTc.cc..",
    "...cccccc.....",
    "...cc..cc.....",
    "...cc..cc.....",
    "..mmm..mmm....",
];

const ENGINEER: [&str; 15] = [
    "....yyyy......",
    "...yYYYYy.....",
    "..yyyyyyyy....",
    "...sesses.....",
    "...ssssss.....",
    "....ssss...m.m",
    "..tTTTTTTt.mmm",
    ".tTTTTTTTTt.m.",
    ".tTTTTTTTTsm..",
    ".sTTTTTTTT.m..",
    "..tTTTTTTt....",
    "..tTTttTTt....",
    "...dd..dd.....",
    "...dd..dd.....",
    "..mmm..mmm....",
];

const MYTH: [&str; 15] = [
    "...yyyyyy.....",
    "..yYYYYYYy....",
    "..YsesesYY....",
    "..YssssssY....",
    "...YssssY.....",
    ".yYtTTTTtYy...",
    "yYtTTyyTTtYy..",
    "YtTTTyyTTTtY..",
    "YsTTTTTTTTsY..",
    ".ytTTTTTTty...",
    "..tTTTTTTt....",
    "..tTTyyTTt....",
    "..tTTTTTTt....",
    "...yy..yy.....",
    "..yyy..yyy....",
];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn palettes_are_6bit() {
        for age in 0..5 {
            for &c in palette(age).iter() {
                assert_eq!(c & 0x0003_0303, 0);
            }
        }
    }

    #[test]
    fn people_fit_and_stand_on_their_bottom_row() {
        let a = Art::new();
        for (k, s) in &a.people {
            assert!(s.w <= 18 && s.h <= 24, "{k:?} is {}x{}", s.w, s.h);
            assert_eq!(s.bottom_row(), s.h - 1, "{k:?} floats");
        }
    }
}
