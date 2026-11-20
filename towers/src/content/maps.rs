use crate::sim::types::*;

#[derive(Clone, Copy)]
pub struct SkirmishMap {
    pub id: usize,
    pub name: &'static str,
    pub desc: &'static str,
    pub p1_citadel: (u8, u8),
    pub p2_citadel: (u8, u8),
    pub tiles: [Terrain; MAP_TILES],
}

const fn blank() -> [Terrain; MAP_TILES] {
    [Terrain::Open; MAP_TILES]
}

fn set_sym(tiles: &mut [Terrain; MAP_TILES], x: usize, y: usize, t: Terrain) {
    if x < MAP_W && y < MAP_H {
        tiles[tile_idx(x, y)] = t;
        tiles[tile_idx(MAP_W - 1 - x, y)] = t;
    }
}

pub fn build_maps() -> [SkirmishMap; 6] {
    // 0: Twin Bridges
    let mut m0 = blank();
    for y in 0..MAP_H {
        if !(2..=4).contains(&y) && !(9..=11).contains(&y) {
            m0[tile_idx(12, y)] = Terrain::Chasm;
        }
    }
    for (x, y, t) in [
        (9, 3, Terrain::HighGround),
        (9, 10, Terrain::HighGround),
        (10, 6, Terrain::HighGround),
        (5, 2, Terrain::OreVein),
        (5, 11, Terrain::OreVein),
        (8, 7, Terrain::LeySpring),
    ] {
        set_sym(&mut m0, x, y, t);
    }

    // 1: High Mesa
    let mut m1 = blank();
    for y in 4..=9 {
        for x in 9..=12 {
            set_sym(&mut m1, x, y, Terrain::HighGround);
        }
    }
    for (x, y, t) in [
        (7, 2, Terrain::OreVein),
        (7, 11, Terrain::OreVein),
        (11, 2, Terrain::LeySpring),
        (11, 11, Terrain::LeySpring),
        (8, 5, Terrain::Chasm),
        (8, 8, Terrain::Chasm),
    ] {
        set_sym(&mut m1, x, y, t);
    }

    // 2: Ley Crossroads
    let mut m2 = blank();
    for (x, y, t) in [
        (7, 3, Terrain::LeySpring),
        (7, 10, Terrain::LeySpring),
        (12, 7, Terrain::LeySpring),
        (10, 5, Terrain::Chasm),
        (10, 8, Terrain::Chasm),
        (9, 6, Terrain::HighGround),
        (9, 7, Terrain::HighGround),
        (4, 2, Terrain::OreVein),
        (4, 11, Terrain::OreVein),
    ] {
        set_sym(&mut m2, x, y, t);
    }

    // 3: Iron Canyon
    let mut m3 = blank();
    for y in 4..=9 {
        set_sym(&mut m3, 8, y, Terrain::Chasm);
    }
    for (x, y, t) in [
        (5, 2, Terrain::OreVein),
        (5, 11, Terrain::OreVein),
        (11, 4, Terrain::OreVein),
        (11, 9, Terrain::OreVein),
        (10, 2, Terrain::HighGround),
        (10, 11, Terrain::HighGround),
        (12, 6, Terrain::HighGround),
        (12, 7, Terrain::HighGround),
        (6, 7, Terrain::LeySpring),
    ] {
        set_sym(&mut m3, x, y, t);
    }

    // 4: Polarity Rift
    let mut m4 = blank();
    for (x, y, t) in [
        (10, 3, Terrain::HighGround),
        (11, 3, Terrain::HighGround),
        (10, 10, Terrain::HighGround),
        (11, 10, Terrain::HighGround),
        (12, 5, Terrain::LeySpring),
        (12, 8, Terrain::LeySpring),
        (6, 4, Terrain::OreVein),
        (6, 9, Terrain::OreVein),
        (9, 6, Terrain::Chasm),
        (9, 7, Terrain::Chasm),
    ] {
        set_sym(&mut m4, x, y, t);
    }

    // 5: Shattered Crown
    let mut m5 = blank();
    for y in [0, 3, 4, 9, 10, 13] {
        set_sym(&mut m5, 11, y, Terrain::Chasm);
    }
    for y in [3, 4, 9, 10] {
        set_sym(&mut m5, 12, y, Terrain::Chasm);
    }
    for (x, y, t) in [
        (12, 6, Terrain::HighGround),
        (12, 7, Terrain::HighGround),
        (8, 2, Terrain::HighGround),
        (8, 11, Terrain::HighGround),
        (5, 4, Terrain::OreVein),
        (5, 9, Terrain::OreVein),
        (9, 6, Terrain::LeySpring),
        (9, 7, Terrain::LeySpring),
    ] {
        set_sym(&mut m5, x, y, t);
    }

    [
        SkirmishMap {
            id: 0,
            name: "TWIN BRIDGES",
            desc: "TWO WIDE BRIDGES OVER A CENTRAL CHASM",
            p1_citadel: (2, 7),
            p2_citadel: (22, 7),
            tiles: m0,
        },
        SkirmishMap {
            id: 1,
            name: "HIGH MESA",
            desc: "CENTRAL ELEVATED PLATEAU FOR +1 RANGE",
            p1_citadel: (2, 7),
            p2_citadel: (22, 7),
            tiles: m1,
        },
        SkirmishMap {
            id: 2,
            name: "LEY CROSSROADS",
            desc: "FOUR MANA WELLS + OPEN DIAGONAL LANES",
            p1_citadel: (2, 7),
            p2_citadel: (22, 7),
            tiles: m2,
        },
        SkirmishMap {
            id: 3,
            name: "IRON CANYON",
            desc: "CLIFF BARRIERS FLANKED BY RICH ORE VEINS",
            p1_citadel: (2, 7),
            p2_citadel: (22, 7),
            tiles: m3,
        },
        SkirmishMap {
            id: 4,
            name: "POLARITY RIFT",
            desc: "CONTESTED MIDFIELD BUILT FOR CLASHES",
            p1_citadel: (2, 7),
            p2_citadel: (22, 7),
            tiles: m4,
        },
        SkirmishMap {
            id: 5,
            name: "SHATTERED CROWN",
            desc: "THREE LANE CROSSINGS AROUND HIGH CROWN",
            p1_citadel: (2, 7),
            p2_citadel: (22, 7),
            tiles: m5,
        },
    ]
}
