use crate::sim::types::*;

/// Map ASCII art char to palette index given faction ramp:
/// '.' = 0 (transparent)
/// '#' = 1 (1px dark hull outline)
/// '1' = dark, '2' = mid, '3' = light, '4' = accent, '5' = glow, 'W' = white, 'G' = gold
const fn map_char(ch: u8, ramp: [u8; 5]) -> u8 {
    match ch {
        b'.' => 0,
        b'#' => 1,
        b'1' => ramp[0],
        b'2' => ramp[1],
        b'3' => ramp[2],
        b'4' => ramp[3],
        b'5' => ramp[4],
        b'W' => 22,
        b'G' => 42,
        _ => 1,
    }
}

pub const fn faction_ramp(f: Faction) -> [u8; 5] {
    match f {
        // [dark, mid, light, accent, glow]
        Faction::Humans => [18, 45, 44, 26, 27],
        Faction::Elves => [28, 29, 30, 39, 31],
        Faction::Orcs => [14, 40, 41, 38, 43],
        Faction::Dwarves => [36, 37, 46, 42, 47],
        Faction::Gnomes => [37, 38, 20, 31, 35],
        Faction::Leones => [13, 12, 42, 43, 22],
        Faction::Dracones => [48, 49, 50, 42, 54],
    }
}

pub const fn parse_16x16(rows: [&str; 16], ramp: [u8; 5]) -> [u8; 256] {
    let mut out = [0u8; 256];
    let mut y = 0;
    while y < 16 {
        let bytes = rows[y].as_bytes();
        let mut x = 0;
        while x < 16 {
            out[y * 16 + x] = map_char(bytes[x], ramp);
            x += 1;
        }
        y += 1;
    }
    out
}

pub const fn parse_10x10(rows: [&str; 10], ramp: [u8; 5]) -> [u8; 100] {
    let mut out = [0u8; 100];
    let mut y = 0;
    while y < 10 {
        let bytes = rows[y].as_bytes();
        let mut x = 0;
        while x < 10 {
            out[y * 10 + x] = map_char(bytes[x], ramp);
            x += 1;
        }
        y += 1;
    }
    out
}

const CITADEL_ART: [&str; 16] = [
    "....#..##..#....",
    "...#G##GG##G#...",
    "...#333GG333#...",
    "..##22255222##..",
    ".#33#255552#33#.",
    ".#22#25WW52#22#.",
    ".#22#225522#22#.",
    ".#112222222211#.",
    "#33221155112233#",
    "#2211#1551#1122#",
    "#2211#1221#1122#",
    "#21111122111112#",
    "#1111#1111#1111#",
    "#1111#1111#1111#",
    "#11111111111111#",
    "################",
];

// Slot 0: Infra (Pylon / Conduit / Furnace / Hearth / Cog / Monolith / Brazier)
const INFRA_ART: [&str; 16] = [
    ".......##.......",
    "......#55#......",
    ".....#5WW5#.....",
    "......#55#......",
    ".....##44##.....",
    "....#332233#....",
    ".....#2222#.....",
    "....#225522#....",
    "...#32255223#...",
    "....#122221#....",
    "....#122221#....",
    "...#21122112#...",
    "..#2211111122#..",
    "..#1111111111#..",
    ".#111111111111#.",
    ".##############.",
];

// Slot 1: Eco (Array / Scriptorium / Altar / Archive / TinkerLab / Shaper / Vault)
const ECO_ART: [&str; 16] = [
    "....########....",
    "...#33333333#...",
    "..#3224554223#..",
    "..#2245WW5422#..",
    ".#322455554223#.",
    ".#222244442222#.",
    ".#21122GG22112#.",
    ".#1112GGGG2111#.",
    "#32112GGGG21123#",
    "#22111222211122#",
    "#21111111111112#",
    "#111#22GG22#111#",
    "#111#2GGGG2#111#",
    "#111#111111#111#",
    "#11111111111111#",
    "################",
];

// Slot 2: Defense (Turret / Prism / Spike / Bastion / Walker / SolarPillar / TaxObelisk)
const DEFENSE_ART: [&str; 16] = [
    "......####......",
    ".....#3333#.....",
    "....#35WW53#....",
    "....#255552#....",
    "...##224422##...",
    "..#3322222233#..",
    "..#2221111222#..",
    "...##112211##...",
    "...#21122112#...",
    "...#21122112#...",
    "..#3211221123#..",
    "..#2211111122#..",
    ".#321111111123#.",
    ".#211111111112#.",
    ".#111111111111#.",
    ".##############.",
];

// Distinct Gnome Walker Turret with legs!
const WALKER_ART: [&str; 16] = [
    ".....######.....",
    "....#333333#....",
    "...#32255223#...",
    "..#225WW522#####",
    "..#222442223345#",
    "...#11222211####",
    "....#112211#....",
    "...##111111##...",
    "..#32#....#23#..",
    "..#21#....#12#..",
    ".#21#......#12#.",
    ".#11#......#11#.",
    "#21#........#12#",
    "#11#........#11#",
    "#1#..........#1#",
    "###..........###",
];

// Slot 3: Factory (Foundry / Sanctum / WarDen / TunnelBore / Hive / TectonicRam / Roost)
const FACTORY_ART: [&str; 16] = [
    "...##########...",
    "..#3333333333#..",
    ".#322224422223#.",
    ".#224445544422#.",
    ".#211222222112#.",
    "#3211######1123#",
    "#221#555555#122#",
    "#211#5WWWW5#112#",
    "#211#225522#112#",
    "#111#222222#111#",
    "#111#111111#111#",
    "#211#11##11#112#",
    "#211#1#..#1#112#",
    "#111#1#..#1#111#",
    "#11111#..#11111#",
    "#######..#######",
];

// Slot 4: Apex (Uplink / AstralLens / Cauldron / Howitzer / Tesla / Colossus / Elder)
const APEX_ART: [&str; 16] = [
    "..#...####...#..",
    ".#5#.#5WW5#.#5#.",
    ".#4#.#3553#.#4#.",
    "..##.#2442#.##..",
    "..#3332222333#..",
    ".#322555555223#.",
    ".#225WWGGWW522#.",
    ".#225WGGGGW522#.",
    "#32224555542223#",
    "#22112244221122#",
    "#21111222211112#",
    "#11111122111111#",
    "#32111111111123#",
    "#2111#1111#1112#",
    "#11111111111111#",
    "################",
];

// Distinct Human Orbital Dish
const UPLINK_ART: [&str; 16] = [
    "..........####..",
    "........##3355#.",
    "......##3225WW#.",
    ".....#3224555#..",
    "....#3224422#...",
    "....#222221#....",
    "....#21111#.....",
    ".....##11##.....",
    ".....#2222#.....",
    "....#322223#....",
    "...#22111122#...",
    "...#21155112#...",
    "..#3211551123#..",
    "..#2111111112#..",
    ".#111111111111#.",
    ".##############.",
];

// Distinct Dwarf Howitzer Artillery
const HOWITZER_ART: [&str; 16] = [
    "...........###..",
    "..........#433#.",
    "........##322#..",
    "......##3221#...",
    "....##3221##....",
    "...#3221##......",
    "..#322442#......",
    "..#224GG42#.....",
    "..#2122221#.....",
    ".#3211221123#...",
    ".#2211111122#...",
    "#3211122111123#.",
    "#2111122111112#.",
    "#1111111111111#.",
    "#1111111111111#.",
    "################",
];

// Distinct Orc Doom Cauldron
const CAULDRON_ART: [&str; 16] = [
    "....#..##..#....",
    "....#5#55#5#....",
    "...#555WW555#...",
    "..############..",
    ".#333333333333#.",
    ".#221111111122#.",
    "#32115555551123#",
    "#21155GGGG55112#",
    "#21115555551112#",
    "#21111222211112#",
    "#11111111111111#",
    ".#111111111111#.",
    ".#2111#11#1112#.",
    ".#111#....#111#.",
    "#211#......#112#",
    "#####......#####",
];

// Distinct Gnome Tesla Contraption
const TESLA_ART: [&str; 16] = [
    ".....######.....",
    "....#5WWWW5#....",
    "...#55555555#...",
    "....#55##55#....",
    ".....#3333#.....",
    "....#324423#....",
    ".....#2222#.....",
    "....#325523#....",
    "...#32255223#...",
    "...#21122112#...",
    "..#3211111123#..",
    "..#21#....#12#..",
    ".#21#......#12#.",
    ".#11#......#11#.",
    "#21#........#12#",
    "####........####",
];

// Distinct Leone Sun Monolith
const MONOLITH_ART: [&str; 16] = [
    "......####......",
    ".....#5WW5#.....",
    "....#5GGGG5#....",
    "....#35GG53#....",
    "....#324423#....",
    "....#322223#....",
    "...#322GG223#...",
    "...#222GG222#...",
    "...#21222212#...",
    "...#21244212#...",
    "..#3212222123#..",
    "..#2111221112#..",
    "..#2111111112#..",
    ".#311111111113#.",
    ".#111111111111#.",
    ".##############.",
];

// Distinct Elven Prism Spire
const PRISM_ART: [&str; 16] = [
    ".......##.......",
    "......#55#......",
    ".....#5WW5#.....",
    ".....#3553#.....",
    "....#334433#....",
    "....#25WW52#....",
    "...#32555523#...",
    "...#22244222#...",
    "..#3212222123#..",
    "..#2112552112#..",
    "..#2112552112#..",
    ".#321112211123#.",
    ".#211111111112#.",
    ".#1111#11#1111#.",
    "#21111111111112#",
    "################",
];

// Distinct Dracone Hoard Vault
const VAULT_ART: [&str; 16] = [
    "...##########...",
    "..#3333333333#..",
    ".#322GGGGGG223#.",
    ".#22GGWWWWGG22#.",
    "#321GGGGGGGG123#",
    "#21122GGGG22112#",
    "#211211GG112112#",
    "#111111GG111111#",
    "#211#GGGGGG#112#",
    "#211#GWWWWG#112#",
    "#111#GGGGGG#111#",
    "#111#22GG22#111#",
    "#21111111111112#",
    "#1111#1111#1111#",
    "#11111111111111#",
    "################",
];

// Distinct Dracone Wyrm Roost
const ROOST_ART: [&str; 16] = [
    ".#............#.",
    ".#3#..####..#3#.",
    "..#3##3333##3#..",
    "...#32255223#...",
    "..#3225WW5223#..",
    "..#2212552122#..",
    ".#321112211123#.",
    ".#211#GGGG#112#.",
    "#321#GGWWGG#123#",
    "#211#G4444G#112#",
    "#211#111111#112#",
    "#11111222211111#",
    ".#211111111112#.",
    ".#1111#11#1111#.",
    "#21111111111112#",
    "################",
];

// Distinct Human Grid Relay (Twin Signal Mast)
const RELAY_ART: [&str; 16] = [
    "...#........#...",
    "..#5#......#5#..",
    ".#4W4#....#4W4#.",
    "..#3##....##3#..",
    "...#2######2#...",
    "....#35WW53#....",
    "...#32255223#...",
    "....#214412#....",
    "....#212212#....",
    "...#32122123#...",
    "...#21122112#...",
    "..#3211111123#..",
    "..#211#GG#112#..",
    ".#311111111113#.",
    ".#111111111111#.",
    ".##############.",
];

// Distinct Human Media Array (Sweeping Radar Dish)
const MEDIA_ART: [&str; 16] = [
    ".......####.....",
    ".....##3355##...",
    "....#3325WW53#..",
    "...#322455422#..",
    "...#22244422#...",
    "....#212222#....",
    ".....#1122#.....",
    "....##3223##....",
    "...#32244223#...",
    "..#2212GG2122#..",
    "..#2111221112#..",
    ".#3211#22#1123#.",
    ".#2111#GG#1112#.",
    ".#111111111111#.",
    "#21111111111112#",
    "################",
];

// Distinct Elven Ley Conduit (Floating Diamond Obelisk)
const CONDUIT_ART: [&str; 16] = [
    ".......##.......",
    "......#55#......",
    ".....#5WW5#.....",
    "....#35WW53#....",
    "...#32555523#...",
    "....#245542#....",
    ".....#2442#.....",
    "......#11#......",
    "....##.##.##....",
    "...#32#55#23#...",
    "...#21255212#...",
    "..#3211221123#..",
    "..#2111111112#..",
    ".#311112211113#.",
    ".#111111111111#.",
    ".##############.",
];

// Distinct Elven Astral Lens (Focused Celestial Ring Cannon)
const LENS_ART: [&str; 16] = [
    "....#..##..#....",
    "...#5##55##5#...",
    "..#355WWWW553#..",
    "..#25WWGGWW52#..",
    "..#355WWWW553#..",
    "...#23355332#...",
    "....#224422#....",
    "...#32155123#...",
    "..#3211551123#..",
    "..#2112442112#..",
    ".#321122221123#.",
    ".#211112211112#.",
    ".#2111#11#1112#.",
    "#31111111111113#",
    "#11111111111111#",
    "################",
];

// Distinct Orc Blood Altar (Horned Flame Basin)
const ALTAR_ART: [&str; 16] = [
    ".#............#.",
    "#4#...####...#4#",
    "#32#.#5WW5#.#23#",
    ".#32#555555#23#.",
    "..############..",
    "..#3322GG2233#..",
    "..#2215555122#..",
    "...#11244211#...",
    "...#21122112#...",
    "..#3211221123#..",
    "..#2111111112#..",
    ".#3211#44#1123#.",
    ".#211111111112#.",
    "#31111111111113#",
    "#11111111111111#",
    "################",
];

// Distinct Dwarf Tunnel Bore (Heavy Rotary Drill Engine)
const BORE_ART: [&str; 16] = [
    "...##########...",
    "..#3333333333#..",
    ".#32244GG44223#.",
    ".#221122221122#.",
    "#321########123#",
    "#21#44455444#12#",
    "#21#334WW433#12#",
    "#11#22344322#11#",
    "#21##223322##12#",
    ".#211#2222#112#.",
    ".#111##22##111#.",
    "#21111#44#11112#",
    "#1111#1##1#1111#",
    "#2111#1111#1112#",
    "#11111111111111#",
    "################",
];

// Distinct Gnome Tinker Lab (4-Port Brass Gizmo Bench)
const TINKER_ART: [&str; 16] = [
    "......####......",
    "....##5WW5##....",
    "...#33455433#...",
    ".###22222222###.",
    "#55211GGGG11255#",
    "#WW21GWWWWG12WW#",
    ".###11GGGG11###.",
    "...#21122112#...",
    "..#3212552123#..",
    "..#2112552112#..",
    ".#321112211123#.",
    ".#211#1111#112#.",
    ".#111#4224#111#.",
    "#21111111111112#",
    "#11111111111111#",
    "################",
];

// Distinct Leone Litho Shaper (Levitated Solar Stone)
const SHAPER_ART: [&str; 16] = [
    "....########....",
    "...#33322333#...",
    "..#322GGGG223#..",
    "..#22GWWWWG22#..",
    "...#11GGGG11#...",
    "....###55###....",
    "..#....55....#..",
    ".#5#.#2GG2#.#5#.",
    ".#32##3223##23#.",
    "..#2112442112#..",
    "..#2112222112#..",
    ".#321111111123#.",
    ".#2111#GG#1112#.",
    ".#111111111111#.",
    "#21111111111112#",
    "################",
];

// Distinct Leone Tectonic Ram (Piston Boulder Chute)
const RAM_BLDG_ART: [&str; 16] = [
    "..############..",
    ".#333333333333#.",
    "#32244GGGG44223#",
    "#2211######1122#",
    "#211#333223#112#",
    "#211#324521#112#",
    "#111#225521#111#",
    "#321#122411#123#",
    "#211##1111##112#",
    "#211#111111#112#",
    "#11112244221111#",
    ".#2111#GG#1112#.",
    ".#211#1..1#112#.",
    "#3111#1..1#1113#",
    "#11111#..#11111#",
    "#######..#######",
];

// Distinct Dracone Void Brazier (Clawed Dark Flame Chalice)
const BRAZIER_ART: [&str; 16] = [
    ".......##.......",
    "....#.#55#.#....",
    "...#5##WW##5#...",
    "..#4555WW5554#..",
    "..############..",
    "...#33255233#...",
    "....#214412#....",
    ".....#1221#.....",
    "....#212212#....",
    "...#32144123#...",
    "...#21122112#...",
    "..#3211111123#..",
    "..#211#55#112#..",
    ".#311111111113#.",
    ".#111111111111#.",
    ".##############.",
];

pub fn building_sprite(kind: BuildingType, owner: TeamId, tick: u32, powered: bool) -> [u8; 256] {
    let ramp = if kind == BuildingType::Citadel {
        if owner == TeamId::P1 {
            [56, 57, 58, 42, 27]
        } else {
            [60, 61, 62, 42, 43]
        }
    } else {
        faction_ramp(kind.faction().unwrap_or(Faction::Humans))
    };

    let art = match kind {
        BuildingType::Citadel => &CITADEL_ART,
        BuildingType::HumanRelay => &RELAY_ART,
        BuildingType::HumanMediaArray => &MEDIA_ART,
        BuildingType::HumanOrbitalUplink => &UPLINK_ART,
        BuildingType::ElvenLeyConduit => &CONDUIT_ART,
        BuildingType::ElvenPrismSpire => &PRISM_ART,
        BuildingType::ElvenAstralLens => &LENS_ART,
        BuildingType::OrcBloodAltar => &ALTAR_ART,
        BuildingType::OrcDoomCauldron => &CAULDRON_ART,
        BuildingType::DwarfTunnelBore => &BORE_ART,
        BuildingType::DwarfHowitzer => &HOWITZER_ART,
        BuildingType::GnomeTinkerLab => &TINKER_ART,
        BuildingType::GnomeWalkerTurret => &WALKER_ART,
        BuildingType::GnomeTeslaContraption => &TESLA_ART,
        BuildingType::LeoneSunMonolith => &MONOLITH_ART,
        BuildingType::LeoneLithoShaper => &SHAPER_ART,
        BuildingType::LeoneTectonicRam => &RAM_BLDG_ART,
        BuildingType::DraconeVoidBrazier => &BRAZIER_ART,
        BuildingType::DraconeHoardVault => &VAULT_ART,
        BuildingType::DraconeWyrmRoost => &ROOST_ART,
        _ => match kind.slot().unwrap_or(BuildingSlot::Infra) {
            BuildingSlot::Infra => &INFRA_ART,
            BuildingSlot::Eco => &ECO_ART,
            BuildingSlot::Defense => &DEFENSE_ART,
            BuildingSlot::Factory => &FACTORY_ART,
            BuildingSlot::Apex => &APEX_ART,
        },
    };

    let mut px = parse_16x16(*art, ramp);

    // Subtle 2-frame pulse on glow pixels (ramp[4] or white) when powered
    let phase = (tick / 6) & 1;
    if !powered {
        // Dim glow pixels to dark hull/slate when offline
        for p in px.iter_mut() {
            if *p == ramp[4] || *p == 22 {
                *p = 18;
            }
        }
    } else if phase == 1 {
        for p in px.iter_mut() {
            if *p == ramp[4] {
                *p = 22; // crisp core shimmer
            }
        }
    }

    // Team ownership banner pip in top-left corner (3x3 with 1px hull)
    let (tb, td) = if owner == TeamId::P1 { (57u8, 56u8) } else { (61u8, 60u8) };
    px[0] = 1;
    px[1] = 1;
    px[2] = 1;
    px[16] = 1;
    px[17] = tb;
    px[18] = td;
    px[32] = 1;
    px[33] = 1;
    px[34] = 1;

    px
}

// ================= UNIT 10x10 SPRITES (Grounded on row 9; Flyers hover) =================

const STRIDER_ART: [&str; 10] = [
    "...####...",
    "..#3333#..",
    ".#322455#.",
    ".#222222#.",
    ".#112211#.",
    "..#1##1#..",
    ".#21##12#.",
    ".#1#..#1#.",
    "#21#..#12#",
    "####..####",
];

const RAIDER_ART: [&str; 10] = [
    "...####...",
    "..#2222#..",
    "..#2525#..",
    ".##3223##.",
    "#43222234#",
    ".##1221##.",
    "..#1111#..",
    "..#2##2#..",
    ".#21##12#.",
    ".###..###.",
];

const RAM_ART: [&str; 10] = [
    "..######..",
    ".#333333#.",
    "#43222235#",
    "#43244235#",
    "#22111122#",
    ".#111111#.",
    ".#21##12#.",
    "#21#..#12#",
    "#11#..#11#",
    "####..####",
];

const GOLEM_ART: [&str; 10] = [
    "..######..",
    ".#333333#.",
    "#32255223#",
    "#22222222#",
    "#32122123#",
    ".#112211#.",
    ".#211112#.",
    ".#21##12#.",
    "#21#..#12#",
    "####..####",
];

const BOT_ART: [&str; 10] = [
    "....##....",
    "...#55#...",
    "..#3333#..",
    ".#325523#.",
    ".#222222#.",
    "..#1221#..",
    "..#2112#..",
    ".#21##12#.",
    ".#11##11#.",
    ".###..###.",
];

const BOULDER_ART: [&str; 10] = [
    "..######..",
    ".#333322#.",
    "#33224522#",
    "#32245521#",
    "#22224221#",
    "#22122111#",
    "#12111111#",
    ".#111111#.",
    "..#1111#..",
    "..######..",
];

const WISP_ART: [&str; 10] = [
    "....##....",
    "...#55#...",
    "..#5WW5#..",
    ".#35WW53#.",
    ".#325523#.",
    "..#2222#..",
    "...#11#...",
    "....##....",
    "..........",
    "..........",
];

const DRAKE_ART: [&str; 10] = [
    "#........#",
    "#3#.##.#3#",
    "#23#33#32#",
    ".#235532#.",
    "..#2222#..",
    "..#1441#..",
    "...#11#...",
    "....##....",
    "..........",
    "..........",
];

pub fn unit_sprite(kind: UnitKind, owner: TeamId, tick: u32, hit_flash: u8) -> [u8; 100] {
    let mut ramp = match kind {
        UnitKind::StriderTank => faction_ramp(Faction::Humans),
        UnitKind::ArcaneWisp => faction_ramp(Faction::Elves),
        UnitKind::OrcRaider | UnitKind::OrcRam => faction_ramp(Faction::Orcs),
        UnitKind::IroncladGolem => faction_ramp(Faction::Dwarves),
        UnitKind::TinkerBot => faction_ramp(Faction::Gnomes),
        UnitKind::LivingBoulder | UnitKind::StoneGolem => faction_ramp(Faction::Leones),
        UnitKind::SkyDrake => faction_ramp(Faction::Dracones),
        UnitKind::ElderWyrm => [48, 53, 42, 43, 54],
    };
    if owner == TeamId::P1 {
        ramp[3] = 57;
    } else {
        ramp[3] = 61;
    }

    let art = match kind {
        UnitKind::StriderTank => &STRIDER_ART,
        UnitKind::ArcaneWisp => &WISP_ART,
        UnitKind::OrcRaider => &RAIDER_ART,
        UnitKind::OrcRam => &RAM_ART,
        UnitKind::IroncladGolem | UnitKind::StoneGolem => &GOLEM_ART,
        UnitKind::TinkerBot => &BOT_ART,
        UnitKind::LivingBoulder => &BOULDER_ART,
        UnitKind::SkyDrake | UnitKind::ElderWyrm => &DRAKE_ART,
    };

    let mut px = parse_10x10(*art, ramp);
    // 2-frame march animation (subtle foot swap on row 8/9 for grounded, wing flap for flyers)
    let frame = (tick / 4) & 1;
    if frame == 1 && !kind.is_flying() && kind != UnitKind::LivingBoulder {
        // Alternate foot highlight on row 8
        for x in 0..5 {
            if px[8 * 10 + x] == 1 {
                px[8 * 10 + x] = ramp[0];
            }
        }
    }
    if hit_flash > 0 {
        for p in px.iter_mut() {
            if *p != 0 && *p != 1 {
                *p = 23;
            }
        }
    }
    px
}

// ================= 7x7 ENERGY, FACTION & UI ICONS =================

const fn parse_7x7_pal(rows: [&str; 7], c1: u8, c2: u8, c3: u8) -> [u8; 49] {
    let mut out = [0u8; 49];
    let mut y = 0;
    while y < 7 {
        let bytes = rows[y].as_bytes();
        let mut x = 0;
        while x < 7 {
            out[y * 7 + x] = match bytes[x] {
                b'.' => 0,
                b'#' => 1,
                b'W' => 22,
                b'G' => 42,
                b'1' => c1,
                b'2' => c2,
                b'3' => c3,
                _ => c1,
            };
            x += 1;
        }
        y += 1;
    }
    out
}

const ICON_MECH: [&str; 7] = [
    ".#.#.#.",
    "#1#2#1#",
    ".#2W2#.",
    "#2W#W2#",
    ".#2W2#.",
    "#1#2#1#",
    ".#.#.#.",
];
const ICON_COMP: [&str; 7] = [
    "..#####",
    ".#2WW#.",
    "#WW2###",
    "###2WW#",
    ".#1W2#.",
    ".#W1#..",
    ".###...",
];
const ICON_MANA: [&str; 7] = [
    "...#...",
    "..#W#..",
    ".#2W2#.",
    "#1WWW1#",
    ".#121#.",
    "..#1#..",
    "...#...",
];
const ICON_POS: [&str; 7] = [
    "..###..",
    "..#2#..",
    "###W###",
    "#2WGW2#",
    "###W###",
    "..#2#..",
    "..###..",
];
const ICON_NEG: [&str; 7] = [
    ".......",
    "#######",
    "#2WWW2#",
    "#12221#",
    "#11111#",
    "#######",
    ".......",
];

pub fn energy_icon(mask: EnergyMask) -> [u8; 49] {
    match mask.0 {
        1 => parse_7x7_pal(ICON_MECH, 47, 42, 43),   // Amber/Gold Gear (Mech)
        2 => parse_7x7_pal(ICON_COMP, 26, 27, 22),   // Electric Blue Bolt (Compute)
        4 => parse_7x7_pal(ICON_MANA, 28, 29, 30),   // Arcane Cyan Crystal (Mana)
        8 => parse_7x7_pal(ICON_POS, 42, 43, 22),    // Sun Gold + (+Force)
        _ => parse_7x7_pal(ICON_NEG, 49, 50, 51),    // Void Violet - (-Force)
    }
}

pub fn faction_icon(f: Faction) -> [u8; 49] {
    let r = faction_ramp(f);
    match f {
        Faction::Humans => parse_7x7_pal(
            [
                "..###..",
                ".#3W3#.",
                "#2#3#2#",
                "#2WW22#",
                "#12221#",
                ".#111#.",
                "..###..",
            ],
            r[0],
            r[2],
            r[4],
        ),
        Faction::Elves => parse_7x7_pal(
            [
                "...#...",
                "..#3#..",
                ".#2W2#.",
                "#23W32#",
                ".#131#.",
                ".#1#1#.",
                "..###..",
            ],
            r[0],
            r[2],
            r[4],
        ),
        Faction::Orcs => parse_7x7_pal(
            [
                "#.....#",
                "#2###2#",
                ".#232#.",
                "#2W#W2#",
                "#11111#",
                ".#W#W#.",
                "..###..",
            ],
            r[0],
            r[1],
            r[4],
        ),
        Faction::Dwarves => parse_7x7_pal(
            [
                ".#####.",
                "#2WWW2#",
                "#13331#",
                "..#2#..",
                "..#1#..",
                ".#212#.",
                ".#####.",
            ],
            r[0],
            r[2],
            r[4],
        ),
        Faction::Gnomes => parse_7x7_pal(
            [
                ".#.#.#.",
                "#23W32#",
                ".#121#.",
                "#23W32#",
                ".#121#.",
                "#2#.#2#",
                ".#...#.",
            ],
            r[0],
            r[2],
            r[4],
        ),
        Faction::Leones => parse_7x7_pal(
            [
                "#.#.#.#",
                ".#GGW#.",
                "#G2W2G#",
                "#G131G#",
                ".#222#.",
                "#.#1#.#",
                "..###..",
            ],
            r[0],
            r[2],
            r[4],
        ),
        Faction::Dracones => parse_7x7_pal(
            [
                "#.....#",
                "#2#.#2#",
                "#12#21#",
                ".#232#.",
                ".#1G1#.",
                "..#1#..",
                "...#...",
            ],
            r[0],
            r[2],
            r[4],
        ),
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum UiIcon {
    Heart,
    Sword,
    Crosshair,
    Shield,
    Coin,
    StarFull,
    StarEmpty,
    Citadel,
    Clock,
    BoltOn,
    BoltOff,
    Upgrade,
    Lock,
    Play,
    FastForward,
    Pause,
    OverlayOff,
    OverlayGrid,
    OverlayPolarity,
    OverlayAll,
    RoleInfra,
    RoleEco,
    RoleDefense,
    RoleFactory,
    RoleApex,
    Book,
    Greed,
    Check,
    Cross,
    Link,
}

impl UiIcon {
    pub const ALL: [UiIcon; 30] = [
        UiIcon::Heart,
        UiIcon::Sword,
        UiIcon::Crosshair,
        UiIcon::Shield,
        UiIcon::Coin,
        UiIcon::StarFull,
        UiIcon::StarEmpty,
        UiIcon::Citadel,
        UiIcon::Clock,
        UiIcon::BoltOn,
        UiIcon::BoltOff,
        UiIcon::Upgrade,
        UiIcon::Lock,
        UiIcon::Play,
        UiIcon::FastForward,
        UiIcon::Pause,
        UiIcon::OverlayOff,
        UiIcon::OverlayGrid,
        UiIcon::OverlayPolarity,
        UiIcon::OverlayAll,
        UiIcon::RoleInfra,
        UiIcon::RoleEco,
        UiIcon::RoleDefense,
        UiIcon::RoleFactory,
        UiIcon::RoleApex,
        UiIcon::Book,
        UiIcon::Greed,
        UiIcon::Check,
        UiIcon::Cross,
        UiIcon::Link,
    ];
}

pub fn ui_icon(icon: UiIcon) -> [u8; 49] {
    match icon {
        UiIcon::Heart => parse_7x7_pal(
            [
                ".##.##.",
                "#W2#21#",
                "#22221#",
                "#12221#",
                ".#121#.",
                "..#1#..",
                "...#...",
            ],
            40,
            61,
            62,
        ),
        UiIcon::Sword => parse_7x7_pal(
            [
                "....###",
                "...#W2#",
                "..#W2#.",
                ".##2#..",
                "#G1#...",
                "#1#....",
                "##.....",
            ],
            44,
            45,
            42,
        ),
        UiIcon::Crosshair => parse_7x7_pal(
            [
                "..###..",
                ".#2#2#.",
                "#2.#.2#",
                "##WGW##",
                "#2.#.2#",
                ".#2#2#.",
                "..###..",
            ],
            42,
            27,
            43,
        ),
        UiIcon::Shield => parse_7x7_pal(
            [
                ".#####.",
                "#W2221#",
                "#22G21#",
                "#21111#",
                ".#111#.",
                "..#1#..",
                "...#...",
            ],
            45,
            46,
            42,
        ),
        UiIcon::Coin => parse_7x7_pal(
            [
                "..###..",
                ".#W22#.",
                "#22122#",
                "#22122#",
                "#12121#",
                ".#111#.",
                "..###..",
            ],
            38,
            42,
            43,
        ),
        UiIcon::StarFull => parse_7x7_pal(
            [
                "...#...",
                "..#W#..",
                "##222##",
                "#2WGW2#",
                ".#121#.",
                "#1#.#1#",
                "##...##",
            ],
            38,
            42,
            43,
        ),
        UiIcon::StarEmpty => parse_7x7_pal(
            [
                "...#...",
                "..#1#..",
                "##111##",
                "#12121#",
                ".#111#.",
                "#1#.#1#",
                "##...##",
            ],
            18,
            19,
            20,
        ),
        UiIcon::Citadel => parse_7x7_pal(
            [
                "#.#.#.#",
                "#2#G#2#",
                "#22222#",
                "#12W21#",
                "#11211#",
                "#11111#",
                "#######",
            ],
            56,
            57,
            58,
        ),
        UiIcon::Clock => parse_7x7_pal(
            [
                "..###..",
                ".#1W1#.",
                "#11W11#",
                "#11WW1#",
                "#11111#",
                ".#111#.",
                "..###..",
            ],
            19,
            27,
            42,
        ),
        UiIcon::BoltOn => parse_7x7_pal(
            [
                "...###.",
                "..#W2#.",
                ".#22#..",
                "#2WWW2#",
                "..#22#.",
                ".#21#..",
                ".###...",
            ],
            33,
            34,
            35,
        ),
        UiIcon::BoltOff => parse_7x7_pal(
            [
                "#..###.",
                "##1#1#.",
                ".#1##..",
                "#1#2##.",
                "..##1#.",
                ".#1#.##",
                ".###..#",
            ],
            60,
            61,
            47,
        ),
        UiIcon::Upgrade => parse_7x7_pal(
            [
                "...#...",
                "..#W#..",
                ".#222#.",
                "#2#W#2#",
                "..#2#..",
                ".#222#.",
                "#######",
            ],
            38,
            42,
            43,
        ),
        UiIcon::Lock => parse_7x7_pal(
            [
                "..###..",
                ".#2.2#.",
                ".#...#.",
                "#######",
                "#22W21#",
                "#21111#",
                "#######",
            ],
            36,
            47,
            42,
        ),
        UiIcon::Play => parse_7x7_pal(
            [
                ".##....",
                ".#W#...",
                ".#22#..",
                ".#222#.",
                ".#22#..",
                ".#1#...",
                ".##....",
            ],
            33,
            34,
            35,
        ),
        UiIcon::FastForward => parse_7x7_pal(
            [
                "#..#...",
                "#2.#2..",
                "#W2#W2.",
                "#22#22#",
                "#W2#W2.",
                "#1.#1..",
                "#..#...",
            ],
            29,
            31,
            22,
        ),
        UiIcon::Pause => parse_7x7_pal(
            [
                "###.###",
                "#W#.#W#",
                "#2#.#2#",
                "#2#.#2#",
                "#1#.#1#",
                "#1#.#1#",
                "###.###",
            ],
            38,
            47,
            42,
        ),
        UiIcon::OverlayOff => parse_7x7_pal(
            [
                "#######",
                "#11111#",
                "#1...1#",
                "#1.1.1#",
                "#1...1#",
                "#11111#",
                "#######",
            ],
            19,
            21,
            22,
        ),
        UiIcon::OverlayGrid => parse_7x7_pal(
            [
                "#######",
                "#22#22#",
                "#W2#12#",
                "#######",
                "#21#2W#",
                "#11#22#",
                "#######",
            ],
            33,
            34,
            35,
        ),
        UiIcon::OverlayPolarity => parse_7x7_pal(
            [
                "..###..",
                ".#1#2#.",
                "#11#22#",
                "#1W#W2#",
                "#11#22#",
                ".#1#2#.",
                "..###..",
            ],
            42,
            50,
            22,
        ),
        UiIcon::OverlayAll => parse_7x7_pal(
            [
                "#######",
                "#11#22#",
                "#1W#W2#",
                "#######",
                "#11#22#",
                "#11#22#",
                "#######",
            ],
            57,
            61,
            42,
        ),
        UiIcon::RoleInfra => parse_7x7_pal(
            [
                "...#...",
                "..#W#..",
                ".#222#.",
                "..#1#..",
                ".#111#.",
                "#11111#",
                "#######",
            ],
            26,
            27,
            22,
        ),
        UiIcon::RoleEco => parse_7x7_pal(
            [
                "..###..",
                ".#2W2#.",
                ".#####.",
                "#2G2G2#",
                "#12121#",
                "#11111#",
                "#######",
            ],
            34,
            42,
            43,
        ),
        UiIcon::RoleDefense => parse_7x7_pal(
            [
                "#.#.#.#",
                "#22222#",
                ".#2W2#.",
                ".#121#.",
                ".#111#.",
                "#11111#",
                "#######",
            ],
            45,
            44,
            41,
        ),
        UiIcon::RoleFactory => parse_7x7_pal(
            [
                "#..###.",
                "#2#222#",
                "#222W2#",
                "#11111#",
                "#12221#",
                "#12.21#",
                "##...##",
            ],
            38,
            47,
            42,
        ),
        UiIcon::RoleApex => parse_7x7_pal(
            [
                "#..#..#",
                "#2#W#2#",
                "#22322#",
                "#13G31#",
                ".#232#.",
                ".#111#.",
                ".#####.",
            ],
            49,
            51,
            43,
        ),
        UiIcon::Book => parse_7x7_pal(
            [
                "###.###",
                "#22#22#",
                "#W2#2W#",
                "#22#22#",
                "#11#11#",
                ".##.##.",
                ".......",
            ],
            38,
            46,
            22,
        ),
        UiIcon::Greed => parse_7x7_pal(
            [
                "..###..",
                ".#3W3#.",
                "#32223#",
                "#21112#",
                ".#212#.",
                "..#2#..",
                "...#...",
            ],
            53,
            54,
            42,
        ),
        UiIcon::Check => parse_7x7_pal(
            [
                ".......",
                ".....##",
                "....#W#",
                "##.#2#.",
                "#W#2#..",
                ".#2#...",
                "..#....",
            ],
            33,
            34,
            35,
        ),
        UiIcon::Cross => parse_7x7_pal(
            [
                "##...##",
                "#2#.#2#",
                ".#2#2#.",
                "..#W#..",
                ".#1#1#.",
                "#1#.#1#",
                "##...##",
            ],
            40,
            61,
            62,
        ),
        UiIcon::Link => parse_7x7_pal(
            [
                "...###.",
                "..#222#",
                ".###1#.",
                "#22W2#.",
                "#2#1#..",
                "#222#..",
                ".###...",
            ],
            28,
            29,
            30,
        ),
    }
}

/// Deterministic 16x16 terrain tile generator
pub fn terrain_tile(t: Terrain, tx: usize, ty: usize) -> [u8; 256] {
    let mut out = [4u8; 256];
    let hash = ((tx * 73 + ty * 137) ^ (tx * ty * 19)) & 31;

    match t {
        Terrain::Open => {
            for y in 0..16 {
                for x in 0..16 {
                    let h = ((tx * 16 + x) * 31 + (ty * 16 + y) * 17) & 15;
                    let c = if x == 0 || y == 0 {
                        5
                    } else if h == 3 && ((x + y) & 1 == 0) {
                        6
                    } else if h == 11 {
                        3
                    } else {
                        4
                    };
                    out[y * 16 + x] = c;
                }
            }
            if hash < 5 {
                out[7 * 16 + 8] = 6;
                out[8 * 16 + 7] = 5;
            }
        }
        Terrain::HighGround => {
            for y in 0..16 {
                for x in 0..16 {
                    let c = if y == 0 || x == 0 {
                        9
                    } else if y == 15 || x == 15 {
                        1
                    } else if y >= 13 || x >= 14 {
                        7
                    } else if ((x + y) & 3) == 0 {
                        7
                    } else {
                        8
                    };
                    out[y * 16 + x] = c;
                }
            }
        }
        Terrain::OreVein => {
            for y in 0..16 {
                for x in 0..16 {
                    let c = if x == 0 || y == 0 { 5 } else { 4 };
                    out[y * 16 + x] = c;
                }
            }
            // Craggy ore rock cluster with gold crystals
            for y in 3..13 {
                for x in 3..13 {
                    let dx = (x as i32 - 7).abs();
                    let dy = (y as i32 - 8).abs();
                    if dx + dy <= 5 {
                        out[y * 16 + x] = if dx + dy == 5 { 1 } else if y < 8 { 11 } else { 10 };
                    }
                }
            }
            for &(ox, oy, c) in &[
                (6usize, 6usize, 42u8),
                (7, 6, 43),
                (6, 7, 39),
                (9, 8, 42),
                (10, 8, 43),
                (5, 10, 46),
                (8, 10, 42),
            ] {
                out[oy * 16 + ox] = c;
            }
        }
        Terrain::LeySpring => {
            for y in 0..16 {
                for x in 0..16 {
                    let dx = x as i32 - 8;
                    let dy = y as i32 - 8;
                    let r2 = dx * dx + dy * dy;
                    let c = if r2 <= 10 {
                        55 // Ambient palette-cycled shimmer
                    } else if r2 <= 22 {
                        29
                    } else if r2 <= 34 {
                        28
                    } else if r2 <= 46 {
                        8
                    } else if r2 <= 56 {
                        1
                    } else {
                        4
                    };
                    out[y * 16 + x] = c;
                }
            }
        }
        Terrain::Chasm => {
            for y in 0..16 {
                for x in 0..16 {
                    let edge = x == 0 || y == 0 || x == 15 || y == 15;
                    let inner = x == 1 || y == 1 || x == 14 || y == 14;
                    let c = if edge {
                        10
                    } else if inner {
                        1
                    } else if ((x * 5 + y * 9 + tx + ty) & 15) == 0 {
                        15
                    } else if ((x + y) & 3) == 0 {
                        63
                    } else {
                        2
                    };
                    out[y * 16 + x] = c;
                }
            }
        }
        Terrain::RaisedStone => {
            for y in 0..16 {
                for x in 0..16 {
                    let c = if x == 0 || y == 0 || x == 15 || y == 15 {
                        1
                    } else if y <= 2 || x <= 2 {
                        12
                    } else if y >= 13 || x >= 13 {
                        14
                    } else {
                        13
                    };
                    out[y * 16 + x] = c;
                }
            }
            // Leone carved solar rune in center
            for &(rx, ry) in &[(7usize, 5usize), (8, 5), (6, 7), (7, 7), (8, 7), (9, 7), (7, 9), (8, 9)] {
                out[ry * 16 + rx] = 42;
            }
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_all_grounded_sprites_touch_bottom_row_and_have_hull() {
        for &kind in &BuildingType::ALL {
            let spr = building_sprite(kind, TeamId::P1, 0, true);
            let bottom_has_pixel = (0..16).any(|x| spr[15 * 16 + x] != 0);
            let has_hull = spr.iter().any(|&c| c == 1);
            assert!(bottom_has_pixel, "Building {:?} must touch row 15", kind);
            assert!(has_hull, "Building {:?} must have 1px dark hull outline (1)", kind);
        }

        let grounded_units = [
            UnitKind::StriderTank,
            UnitKind::OrcRaider,
            UnitKind::OrcRam,
            UnitKind::IroncladGolem,
            UnitKind::TinkerBot,
            UnitKind::LivingBoulder,
            UnitKind::StoneGolem,
        ];
        for kind in grounded_units {
            let spr = unit_sprite(kind, TeamId::P1, 0, 0);
            let bottom_has_pixel = (0..10).any(|x| spr[9 * 10 + x] != 0);
            let has_hull = spr.iter().any(|&c| c == 1);
            assert!(bottom_has_pixel, "Grounded unit {:?} must touch bottom row 9", kind);
            assert!(has_hull, "Unit {:?} must have 1px dark hull outline (1)", kind);
        }
    }

    #[test]
    fn test_all_icons_have_hull_and_content() {
        for &mask in &[
            EnergyMask::MECH,
            EnergyMask::COMPUTE,
            EnergyMask::MANA,
            EnergyMask::POS_FORCE,
            EnergyMask::NEG_FORCE,
        ] {
            let ic = energy_icon(mask);
            assert!(ic.iter().any(|&c| c == 1), "Energy icon {:?} must have dark hull (1)", mask);
            assert!(ic.iter().any(|&c| c > 1), "Energy icon {:?} must have foreground pixels", mask);
        }
        for &f in &Faction::ALL {
            let ic = faction_icon(f);
            assert!(ic.iter().any(|&c| c == 1), "Faction icon {:?} must have dark hull (1)", f);
            assert!(ic.iter().any(|&c| c > 1), "Faction icon {:?} must have foreground pixels", f);
        }
        for &ui in &UiIcon::ALL {
            let ic = ui_icon(ui);
            assert!(ic.iter().any(|&c| c == 1), "UI icon {:?} must have dark hull (1)", ui);
            assert!(ic.iter().any(|&c| c > 1), "UI icon {:?} must have foreground pixels", ui);
        }
    }
}
