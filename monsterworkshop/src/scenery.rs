//! Scenery's art: the props towns are dressed with (`world::PROPS`), drawn in code in the
//! game's palette, lit from the upper left, each with its dark outline. A kind may have a few
//! variants (flower beds of different colors), picked per tile. Props that join their
//! neighbors (fences, stone walls, rails, washing lines, gate posts) are drawn in `render.rs`.

use crate::palette::*;
use crate::sprites::{circle_fill, Sprite, STONE};

fn rect(s: &mut Sprite, x0: i32, y0: i32, x1: i32, y1: i32, c: u8) {
    for y in y0..=y1 {
        for x in x0..=x1 {
            s.set(x, y, c);
        }
    }
}

/// Decorative trees' leaves, darker than those you can gather from.
const CANOPY: [u8; 3] = [HEDGE, HEDGE + 1, HEDGE + 2];
/// Cast bronze.
const BRONZE: [u8; 3] = [WOOD + 2, SPARK, SPARK + 1];
/// Fired clay, a dull brick red.
const TERRACOTTA: [u8; 3] = [WOOD + 1, WOOD + 2, THREAD + 2];
/// Cherry blossom.
const BLOSSOM: [u8; 3] = [THREAD + 1, FLOWER, COTTON + 1];

/// Every variant of a kind's art, or none if the kind is drawn by joining its neighbors.
pub fn variants(id: &str) -> Vec<Sprite> {
    match id {
        "rock_face" => (0..3).map(rock_face).collect(),
        "mine_mouth" => vec![mine_mouth()],
        "mine_cart" => vec![mine_cart()],
        "lantern_post" => vec![lantern_post()],
        "wheelbarrow" => vec![wheelbarrow()],
        "tool_shed" => vec![tool_shed()],
        "crates" => vec![crates()],
        "awning" => vec![awning()],
        "flowers" => [(FLOWER, FLOWER + 1), (THREAD + 2, COTTON + 2), (WATER + 2, SPARK + 1)].into_iter().map(flowers).collect(),
        "boulder" => (0..2).map(boulder).collect(),
        "big_tree" => vec![big_tree()],
        "shade_tree" => vec![shade_tree()],
        "bench" => vec![bench()],
        "firewood" => vec![firewood()],
        "trough" => vec![trough()],
        "anvil" => vec![anvil()],
        "hand_cart" => vec![hand_cart()],
        "chimney" => vec![chimney()],
        "hoist" => vec![hoist()],
        "wall_lantern" => vec![wall_lantern()],
        "moulds" => vec![moulds()],
        "stone_lantern" => vec![stone_lantern()],
        "incense_burner" => vec![incense_burner()],
        "pond" => vec![pond()],
        "cherry_tree" => vec![cherry_tree()],
        "banner" => [THREAD + 1, WATER + 1].into_iter().map(banner).collect(),
        "cushion" => [THREAD + 1, WATER + 1, LEAF + 1, SPARK, CYCLOPS + 1].into_iter().map(cushion).collect(),
        "candle" => vec![candle()],
        "drum" => vec![drum()],
        "altar" => vec![altar()],
        "well" => vec![well()],
        "dye_barrel" => [INDIGO, MADDER, [WATER, WATER + 1, WATER + 2]].into_iter().map(dye_barrel).collect(),
        "dye_tub" => [MADDER, INDIGO].into_iter().map(dye_tub).collect(),
        "cloth_shelf" => vec![cloth_shelf()],
        "cutting_table" => vec![cutting_table()],
        "hanging_cloth" => [THREAD + 1, WATER + 1, LEAF + 1].into_iter().map(hanging_cloth).collect(),
        "dummy" => vec![dummy()],
        "mirror" => vec![mirror()],
        "shop_sign" => vec![shop_sign()],
        "stone_pile" => vec![stone_pile()],
        "stump" => vec![stump()],
        "reeds" => (0..2).map(reeds).collect(),
        "lily_pads" => (0..2).map(lily_pads).collect(),
        "rowboat" => vec![rowboat()],
        "field_shed" => vec![field_shed()],
        "scarecrow" => vec![scarecrow()],
        "field_trough" => vec![field_trough()],
        "shrub" => vec![shrub()],
        "baskets" => vec![baskets()],
        "sacks" => vec![sacks()],
        _ => Vec::new(),
    }
}

/// The hillside: grey boulders packed tight, a few mossy, in three arrangements.
fn rock_face(k: u8) -> Sprite {
    let mut s = Sprite::blank(16, 16);
    let rocks: &[(i32, i32, i32)] = match k {
        0 => &[(4, 4, 4), (12, 4, 4), (4, 12, 4), (12, 12, 4), (8, 8, 3)],
        1 => &[(8, 4, 5), (3, 11, 4), (11, 11, 5), (14, 3, 2)],
        _ => &[(4, 5, 5), (12, 6, 4), (7, 13, 3), (13, 13, 3), (1, 13, 2)],
    };
    for &(cx, cy, r) in rocks {
        circle_fill(&mut s, cx, cy, r, STONE);
        // A dark crack between it and the next.
        s.set(cx + r, cy, HUD_FILL);
        s.set(cx, cy + r, HUD_FILL);
    }
    for (x, y) in [(3, 2), (4, 2), (11, 1)] {
        s.set(x + k as i32, y, LEAF);
    }
    s.outline()
}

/// An old drift into the hillside: rock all round, a timber frame, darkness, and the rails
/// running in.
fn mine_mouth() -> Sprite {
    let mut s = Sprite::blank(32, 32);
    for &(cx, cy, r) in &[(4, 4, 5), (14, 3, 4), (24, 4, 5), (29, 12, 4), (3, 14, 4), (2, 25, 3), (29, 25, 3)] {
        circle_fill(&mut s, cx, cy, r, STONE);
    }
    // The dark inside, deepest in the middle.
    rect(&mut s, 6, 8, 25, 31, HUD_SHADOW);
    rect(&mut s, 10, 12, 21, 31, OUTLINE);
    // The rails running in, fading into the dark.
    for x in [12, 19] {
        rect(&mut s, x, 22, x, 31, HUD_FILL);
    }
    for y in [24, 28] {
        rect(&mut s, 11, y, 20, y, WOOD);
    }
    // Two posts and a lintel, lit on the left.
    for x0 in [3, 25] {
        rect(&mut s, x0, 7, x0 + 3, 31, WOOD + 1);
        rect(&mut s, x0, 7, x0, 31, WOOD + 2);
        rect(&mut s, x0 + 3, 7, x0 + 3, 31, WOOD);
    }
    rect(&mut s, 1, 3, 30, 7, WOOD + 2);
    rect(&mut s, 1, 3, 30, 3, WOOD + 3);
    rect(&mut s, 1, 7, 30, 7, WOOD);
    for x in [6, 15, 24] {
        s.set(x, 5, WOOD);
    }
    s.outline()
}

/// An empty mine cart, standing on its rails (which `render.rs` draws under it).
fn mine_cart() -> Sprite {
    let mut s = Sprite::blank(16, 14);
    for y in 2..=9 {
        let inset = (y - 2) / 4;
        rect(&mut s, 1 + inset, y, 14 - inset, y, if y <= 3 { HUD_HI } else { HUD_MID });
    }
    rect(&mut s, 3, 3, 12, 5, HUD_SHADOW);
    rect(&mut s, 1, 2, 14, 2, TEXT_DIM);
    for x in [4, 11] {
        circle_fill(&mut s, x, 11, 2, [OUTLINE, HUD_FILL, HUD_MID]);
    }
    s.outline()
}

/// A lantern hung from a post's arm, glowing.
fn lantern_post() -> Sprite {
    let mut s = Sprite::blank(16, 16);
    rect(&mut s, 5, 2, 6, 15, WOOD + 2);
    rect(&mut s, 6, 2, 6, 15, WOOD);
    rect(&mut s, 4, 14, 7, 15, WOOD + 1);
    rect(&mut s, 5, 2, 11, 2, WOOD + 2);
    s.set(11, 3, HUD_HI);
    rect(&mut s, 9, 4, 13, 9, OUTLINE);
    rect(&mut s, 10, 5, 12, 8, SPARK + 1);
    s.set(10, 5, SPARK + 2);
    rect(&mut s, 9, 4, 13, 4, HUD_MID);
    s.outline()
}

/// A wheelbarrow: an iron tray on a wheel, handles behind.
fn wheelbarrow() -> Sprite {
    let mut s = Sprite::blank(16, 12);
    for y in 3..=7 {
        let inset = (y - 3) / 2;
        rect(&mut s, 2 + inset, y, 11 - inset, y, if y == 3 { HUD_HI } else { HUD_MID });
    }
    rect(&mut s, 4, 4, 9, 5, HUD_FILL);
    rect(&mut s, 0, 2, 3, 2, WOOD + 2);
    rect(&mut s, 3, 8, 3, 11, WOOD + 1);
    circle_fill(&mut s, 12, 9, 2, [OUTLINE, HUD_FILL, HUD_MID]);
    s.outline()
}

/// A lean-to shed, two tiles wide and three deep: a plank roof sloping to the front, and inside
/// the open front a spare pick and shovel and a lamp.
fn tool_shed() -> Sprite {
    let mut s = Sprite::blank(32, 46);
    // The roof, its planks running down the slope.
    for y in 2..=17 {
        for x in 0..=31 {
            s.set(x, y, if x % 5 == 0 { PLANK } else if y < 5 { PLANK + 3 } else { PLANK + 2 });
        }
    }
    rect(&mut s, 0, 17, 31, 17, PLANK);
    // The walls, boards upright.
    for y in 18..=45 {
        for x in 1..=30 {
            s.set(x, y, if x % 4 == 0 { WOOD } else { WOOD + 1 });
        }
    }
    // The open front, and what's inside.
    rect(&mut s, 5, 22, 26, 45, HUD_SHADOW);
    rect(&mut s, 5, 22, 26, 22, WOOD + 3);
    rect(&mut s, 9, 25, 9, 38, WOOD + 2);
    rect(&mut s, 7, 24, 11, 25, HUD_HI);
    rect(&mut s, 15, 25, 15, 34, WOOD + 2);
    rect(&mut s, 14, 35, 16, 38, HUD_HI);
    rect(&mut s, 21, 28, 23, 31, SPARK + 1);
    s.set(21, 28, SPARK + 2);
    rect(&mut s, 18, 41, 25, 45, WOOD + 2);
    s.outline()
}

/// Two crates stacked, a smaller on a bigger, each braced with an X.
fn crates() -> Sprite {
    let mut s = Sprite::blank(16, 16);
    for &(x0, y0, x1, y1) in &[(1, 6, 14, 15), (3, 0, 12, 6)] {
        // The lid, lit, then the front: light boards in a dark frame, braced corner to corner.
        rect(&mut s, x0, y0, x1, y0 + 1, MAT + 2);
        rect(&mut s, x0, y0 + 2, x1, y1, PLANK + 3);
        rect(&mut s, x0, y0 + 2, x1, y0 + 2, WOOD + 1);
        rect(&mut s, x0, y1, x1, y1, WOOD + 1);
        rect(&mut s, x0, y0 + 2, x0, y1, WOOD + 1);
        rect(&mut s, x1, y0 + 2, x1, y1, WOOD + 1);
        let (w, h) = (x1 - x0, y1 - y0 - 2);
        for k in 1..w {
            s.set(x0 + k, y0 + 2 + k * h / w, WOOD + 2);
        }
    }
    s.outline()
}

/// The trader's awning, three tiles wide: bright stripes with a scalloped edge, on two posts.
fn awning() -> Sprite {
    let mut s = Sprite::blank(48, 16);
    let stripes = [THREAD + 1, SPARK + 1, WATER + 1, SPARK + 1];
    for x in 0..48 {
        let c = stripes[(x / 6) as usize % stripes.len()];
        let bottom = 7 + ((x % 6) / 3) as i32;
        for y in 0..=bottom {
            s.set(x, y, if y == 0 { COTTON + 2 } else { c });
        }
    }
    for x0 in [2, 44] {
        rect(&mut s, x0, 9, x0 + 1, 15, WOOD + 2);
    }
    s.outline()
}

/// A flower bed: a low border, leaves, and blossoms in two colors.
fn flowers((a, b): (u8, u8)) -> Sprite {
    let mut s = Sprite::blank(16, 12);
    rect(&mut s, 1, 7, 14, 11, PATH);
    rect(&mut s, 1, 11, 14, 11, WOOD + 1);
    for &(x, y) in &[(3, 5), (7, 4), (11, 5), (5, 8), (9, 8), (13, 8)] {
        s.set(x, y + 1, LEAF + 1);
        s.set(x - 1, y + 1, LEAF);
    }
    for (k, &(x, y)) in [(3, 4), (7, 3), (11, 4), (5, 7), (9, 7), (13, 7)].iter().enumerate() {
        let c = if k % 2 == 0 { a } else { b };
        rect(&mut s, x - 1, y - 1, x, y, c);
        s.set(x, y - 1, SPARK + 2);
    }
    s.outline()
}

/// A boulder, a little moss on it.
fn boulder(k: u8) -> Sprite {
    let mut s = Sprite::blank(16, 12);
    if k == 0 {
        circle_fill(&mut s, 8, 7, 5, STONE);
        rect(&mut s, 3, 11, 13, 11, STONE[0]);
    } else {
        circle_fill(&mut s, 6, 8, 4, STONE);
        circle_fill(&mut s, 11, 9, 3, STONE);
    }
    for (x, y) in [(5, 3), (6, 3), (4, 4)] {
        s.set(x + k as i32, y + k as i32, LEAF);
    }
    s.outline()
}

/// A crown of leaves: overlapping round clumps, the light on the upper left.
fn crown(s: &mut Sprite, clumps: &[(i32, i32, i32)], ramp: [u8; 3]) {
    for &(cx, cy, r) in clumps {
        circle_fill(s, cx, cy, r, ramp);
    }
    // Leaves catching the light along the top of each clump.
    for &(cx, cy, r) in clumps {
        for dx in (-r / 2..=r / 2).step_by(3) {
            s.set(cx + dx - 1, cy - r + 2, ramp[2]);
        }
    }
}

/// A trunk from `top` to the bottom row, `w` wide, centered on `cx`, with roots at its foot.
fn trunk(s: &mut Sprite, cx: i32, top: i32, w: i32) {
    let bottom = s.h - 1;
    rect(s, cx - w / 2, top, cx - w / 2 + w - 1, bottom, WOOD + 1);
    rect(s, cx - w / 2, top, cx - w / 2, bottom, WOOD + 2);
    rect(s, cx - w / 2 + w - 1, top, cx - w / 2 + w - 1, bottom, WOOD);
    rect(s, cx - w / 2 - 2, bottom, cx - w / 2 + w + 1, bottom, WOOD);
}

/// The great tree, three tiles square: an old broad crown on a thick trunk.
fn big_tree() -> Sprite {
    let mut s = Sprite::blank(48, 48);
    trunk(&mut s, 24, 30, 6);
    crown(&mut s, &[(24, 20, 16), (12, 24, 10), (36, 24, 10), (18, 10, 9), (31, 10, 9)], CANOPY);
    s.outline()
}

/// A shade tree, two tiles square.
fn shade_tree() -> Sprite {
    let mut s = Sprite::blank(32, 32);
    trunk(&mut s, 16, 22, 4);
    crown(&mut s, &[(16, 13, 11), (9, 16, 7), (23, 16, 7)], CANOPY);
    s.outline()
}

/// A cherry tree in blossom, two tiles square, a few petals fallen.
fn cherry_tree() -> Sprite {
    let mut s = Sprite::blank(32, 32);
    trunk(&mut s, 16, 21, 3);
    crown(&mut s, &[(16, 12, 10), (8, 15, 7), (24, 15, 7)], BLOSSOM);
    for (x, y) in [(6, 10), (13, 6), (21, 9), (25, 14), (10, 17)] {
        s.set(x, y, COTTON + 2);
    }
    for (x, y) in [(4, 30), (9, 31), (24, 30), (27, 31)] {
        s.set(x, y, FLOWER);
    }
    s.outline()
}

/// A bench, two tiles wide: a backrest, two planks, and legs.
fn bench() -> Sprite {
    let mut s = Sprite::blank(32, 14);
    rect(&mut s, 2, 1, 29, 2, WOOD + 2);
    for y in [5, 7] {
        rect(&mut s, 1, y, 30, y + 1, if y == 5 { WOOD + 3 } else { WOOD + 2 });
    }
    for x in [3, 27] {
        rect(&mut s, x, 3, x + 1, 4, WOOD + 1);
        rect(&mut s, x, 9, x + 1, 13, WOOD);
    }
    s.outline()
}

/// Firewood stacked, cut ends out.
fn firewood() -> Sprite {
    let mut s = Sprite::blank(16, 16);
    // Each log's end: bark round a pale cut face with a dark heart.
    let end = |s: &mut Sprite, x0: i32, y0: i32| {
        for (dy, row) in [".BBB.", "BcccB", "BchcB", "BcccB", ".BBB."].iter().enumerate() {
            for (dx, ch) in row.chars().enumerate() {
                let c = match ch {
                    'B' => WOOD,
                    'c' => PLANK + 3,
                    'h' => PLANK + 1,
                    _ => continue,
                };
                s.set(x0 + dx as i32, y0 + dy as i32, c);
            }
        }
    };
    for x0 in [0, 5, 10] {
        end(&mut s, x0, 11);
    }
    for x0 in [2, 7] {
        end(&mut s, x0, 7);
    }
    end(&mut s, 5, 3);
    s.outline()
}

/// A stone trough, two tiles long, full of water.
fn trough() -> Sprite {
    let mut s = Sprite::blank(32, 13);
    rect(&mut s, 1, 2, 30, 12, STONE[1]);
    rect(&mut s, 1, 2, 30, 2, STONE[2]);
    rect(&mut s, 1, 8, 30, 12, STONE[0]);
    rect(&mut s, 3, 3, 28, 6, WATER + 1);
    for x in [6, 7, 17, 23] {
        s.set(x, 4, WATER + 2);
    }
    s.outline()
}

/// An anvil on a stump.
fn anvil() -> Sprite {
    let mut s = Sprite::blank(16, 16);
    rect(&mut s, 4, 10, 11, 15, WOOD + 1);
    rect(&mut s, 4, 10, 4, 15, WOOD + 2);
    rect(&mut s, 4, 9, 11, 9, PLANK + 2);
    rect(&mut s, 1, 3, 13, 4, HUD_FILL);
    rect(&mut s, 1, 3, 13, 3, HUD_HI);
    rect(&mut s, 0, 3, 2, 3, HUD_MID);
    rect(&mut s, 5, 5, 10, 6, HUD_SHADOW);
    rect(&mut s, 3, 7, 12, 8, HUD_FILL);
    s.outline()
}

/// A hand cart, two tiles long: a plank bed on one wheel, its handles out front.
fn hand_cart() -> Sprite {
    let mut s = Sprite::blank(32, 16);
    rect(&mut s, 8, 2, 29, 8, WOOD + 2);
    rect(&mut s, 8, 2, 29, 2, WOOD + 3);
    for x in [14, 20, 26] {
        rect(&mut s, x, 3, x, 8, WOOD + 1);
    }
    rect(&mut s, 0, 6, 8, 6, WOOD + 2);
    rect(&mut s, 0, 9, 8, 9, WOOD + 1);
    circle_fill(&mut s, 21, 11, 4, [WOOD, WOOD + 1, WOOD + 2]);
    s.set(21, 11, WOOD + 3);
    rect(&mut s, 10, 9, 10, 15, WOOD);
    s.outline()
}

/// The furnace's chimney, stone, standing up out of the foundry's wall, a wisp of smoke above.
fn chimney() -> Sprite {
    let mut s = Sprite::blank(16, 30);
    for y in 5..=29 {
        for x in 3..=12 {
            let course = (y - 5) / 3;
            let joint = y % 3 == 1 || (x + course * 3) % 6 == 0;
            s.set(x, y, if joint { STONE[0] } else if x <= 5 { STONE[2] } else { STONE[1] });
        }
    }
    rect(&mut s, 2, 5, 13, 7, STONE[1]);
    rect(&mut s, 2, 5, 13, 5, STONE[2]);
    rect(&mut s, 5, 6, 10, 6, OUTLINE);
    let mut s = s.outline();
    for (x, y) in [(8, 3), (9, 2), (9, 1), (10, 0), (7, 2)] {
        s.set(x, y, TEXT_DIM);
    }
    s
}

/// A chain hoist hanging from the wall's beam into the room, a hook at the bottom.
fn hoist() -> Sprite {
    let mut s = Sprite::blank(16, 24);
    rect(&mut s, 4, 2, 11, 4, WOOD + 2);
    rect(&mut s, 6, 5, 9, 8, HUD_MID);
    s.set(7, 6, HUD_HI);
    for y in 9..=18 {
        s.set(7 + (y % 2), y, if y % 2 == 0 { HUD_HI } else { HUD_MID });
    }
    for (x, y) in [(8, 19), (9, 20), (9, 21), (8, 22), (7, 22), (6, 21)] {
        s.set(x, y, HUD_HI);
    }
    s.outline()
}

/// A lantern on a bracket on the wall.
fn wall_lantern() -> Sprite {
    let mut s = Sprite::blank(16, 16);
    rect(&mut s, 7, 3, 8, 4, HUD_MID);
    rect(&mut s, 5, 5, 10, 11, OUTLINE);
    rect(&mut s, 6, 6, 9, 10, SPARK + 1);
    s.set(6, 6, SPARK + 2);
    rect(&mut s, 5, 5, 10, 5, HUD_MID);
    s.outline()
}

/// Bell moulds of fired clay, a small one stacked on a big one.
fn moulds() -> Sprite {
    let mut s = Sprite::blank(16, 16);
    // Each a dome of baked clay with a band round it, the small one sat on the big one.
    for &(cx, top, r, bottom) in &[(8, 7, 6, 15), (8, 1, 4, 7)] {
        for y in top..=bottom {
            let half = match y - top {
                0 => r - 2,
                1 => r - 1,
                _ => r,
            };
            for x in cx - half..=cx + half {
                let c = if x <= cx - half + 1 { TERRACOTTA[2] } else if x >= cx + half - 1 { TERRACOTTA[0] } else { TERRACOTTA[1] };
                s.set(x, y, c);
            }
        }
        rect(&mut s, cx - r, bottom - 2, cx + r, bottom - 2, TERRACOTTA[0]);
    }
    s.outline()
}

/// A stone lantern: a base, a post, a light box glowing, and a roof.
fn stone_lantern() -> Sprite {
    let mut s = Sprite::blank(16, 16);
    rect(&mut s, 4, 13, 11, 15, STONE[0]);
    rect(&mut s, 4, 13, 11, 13, STONE[1]);
    rect(&mut s, 6, 9, 9, 12, STONE[1]);
    rect(&mut s, 4, 5, 11, 8, STONE[1]);
    rect(&mut s, 6, 6, 9, 7, SPARK + 1);
    s.set(6, 6, SPARK + 2);
    rect(&mut s, 2, 3, 13, 4, STONE[2]);
    rect(&mut s, 2, 4, 13, 4, STONE[1]);
    rect(&mut s, 7, 1, 8, 2, STONE[2]);
    s.outline()
}

/// A bronze incense burner on a stone base, smoke rising.
fn incense_burner() -> Sprite {
    let mut s = Sprite::blank(16, 16);
    rect(&mut s, 2, 12, 13, 15, STONE[1]);
    rect(&mut s, 2, 12, 13, 12, STONE[2]);
    for y in 6..=10 {
        let inset = if y == 10 { 1 } else { 0 };
        rect(&mut s, 4 + inset, y, 11 - inset, y, if y == 6 { BRONZE[2] } else if y <= 7 { BRONZE[1] } else { BRONZE[0] });
    }
    for x in [5, 10] {
        s.set(x, 11, BRONZE[0]);
    }
    s.set(3, 7, BRONZE[1]);
    s.set(12, 7, BRONZE[1]);
    let mut s = s.outline();
    for (x, y) in [(7, 4), (8, 3), (8, 2), (7, 1), (9, 0)] {
        s.set(x, y, TEXT_DIM);
    }
    s
}

/// A lily pond, three tiles square: a rim of stones, still water, lilies, a koi.
fn pond() -> Sprite {
    let mut s = Sprite::blank(48, 46);
    for y in 0..46 {
        for x in 0..48 {
            let (dx, dy) = ((x - 24) as f64 / 23.0, (y - 23) as f64 / 22.0);
            let d = dx * dx + dy * dy;
            if d <= 1.0 {
                let c = if d > 0.8 {
                    if (x + y) % 5 == 0 { STONE[0] } else if y < 23 { STONE[2] } else { STONE[1] }
                } else if d > 0.68 && y < 23 {
                    WATER
                } else {
                    WATER + 1
                };
                s.set(x, y, c);
            }
        }
    }
    for &(cx, cy) in &[(15, 16), (30, 26), (20, 31), (33, 13)] {
        circle_fill(&mut s, cx, cy, 3, [LEAF, LEAF + 1, LEAF + 2]);
        s.set(cx + 1, cy - 1, WATER + 1);
        s.set(cx + 2, cy - 1, WATER + 1);
    }
    rect(&mut s, 14, 15, 15, 16, FLOWER);
    rect(&mut s, 29, 25, 30, 26, COTTON + 2);
    for (x, y) in [(24, 20), (25, 20), (26, 21)] {
        s.set(x, y, SPARK + 1);
    }
    for (x, y) in [(10, 27), (37, 20), (22, 9)] {
        s.set(x, y, WATER + 2);
    }
    s.outline()
}

/// A banner hanging from a rod, a bell sewn on it.
fn banner(cloth: u8) -> Sprite {
    let mut s = Sprite::blank(16, 26);
    rect(&mut s, 2, 1, 13, 2, WOOD + 2);
    rect(&mut s, 4, 3, 11, 22, cloth);
    rect(&mut s, 4, 3, 4, 22, shift(cloth, 1));
    for x in [4, 6, 8, 10] {
        s.set(x, 23, cloth);
    }
    // The bell: a dome, its lip, and its clapper.
    rect(&mut s, 7, 8, 8, 8, SPARK + 1);
    rect(&mut s, 6, 9, 9, 12, SPARK + 1);
    rect(&mut s, 5, 13, 10, 13, SPARK);
    s.set(7, 14, SPARK);
    s.outline()
}

/// A square floor cushion, buttoned in the middle.
fn cushion(color: u8) -> Sprite {
    let mut s = Sprite::blank(16, 10);
    rect(&mut s, 3, 2, 12, 8, color);
    rect(&mut s, 3, 2, 12, 2, shift(color, 1));
    rect(&mut s, 3, 2, 3, 8, shift(color, 1));
    rect(&mut s, 3, 8, 12, 8, shift(color, -1));
    s.set(7, 5, shift(color, -1));
    s.set(8, 5, shift(color, -1));
    s.outline()
}

/// A candle on a tall stand.
fn candle() -> Sprite {
    let mut s = Sprite::blank(16, 16);
    rect(&mut s, 5, 14, 10, 15, BRONZE[0]);
    rect(&mut s, 7, 6, 8, 13, BRONZE[1]);
    rect(&mut s, 5, 5, 10, 5, BRONZE[2]);
    rect(&mut s, 7, 2, 8, 4, COTTON + 2);
    s.set(7, 1, SPARK + 2);
    s.set(7, 0, SPARK + 1);
    s.outline()
}

/// A temple drum on its stand: a red body, a pale head, brass studs.
fn drum() -> Sprite {
    let mut s = Sprite::blank(16, 16);
    for x in [3, 12] {
        rect(&mut s, x, 11, x, 15, WOOD);
    }
    rect(&mut s, 2, 5, 13, 11, THREAD + 1);
    rect(&mut s, 2, 5, 3, 11, THREAD + 2);
    rect(&mut s, 12, 5, 13, 11, THREAD);
    rect(&mut s, 3, 1, 12, 4, COTTON + 1);
    rect(&mut s, 4, 1, 11, 1, COTTON + 2);
    for x in (3..=12).step_by(3) {
        s.set(x, 5, SPARK + 1);
        s.set(x, 11, SPARK + 1);
    }
    s.outline()
}

/// A small altar table with flowers and a bowl of water.
fn altar() -> Sprite {
    let mut s = Sprite::blank(16, 16);
    rect(&mut s, 1, 8, 14, 9, PLANK + 2);
    rect(&mut s, 1, 10, 14, 11, WOOD + 1);
    for x in [2, 13] {
        rect(&mut s, x, 12, x, 15, WOOD);
    }
    rect(&mut s, 3, 4, 5, 7, COTTON + 1);
    for (x, y) in [(3, 2), (5, 1), (4, 3)] {
        s.set(x, y, FLOWER);
    }
    s.set(4, 1, LEAF + 1);
    rect(&mut s, 9, 5, 12, 7, BRONZE[1]);
    rect(&mut s, 10, 5, 11, 5, WATER + 2);
    s.outline()
}

/// The well, two tiles square: a ring of stones, dark water, two posts and a little roof with
/// its crank and rope.
fn well() -> Sprite {
    let mut s = Sprite::blank(32, 32);
    for y in 14..=31 {
        for x in 3..=28 {
            let (dx, dy) = ((x - 16) as f64 / 13.0, (y - 22) as f64 / 9.0);
            if dx * dx + dy * dy <= 1.0 || y >= 22 {
                let course = (y - 14) / 3;
                let joint = y % 3 == 2 || (x + course * 3) % 6 == 0;
                s.set(x, y, if joint { STONE[0] } else if x < 10 { STONE[2] } else { STONE[1] });
            }
        }
    }
    for y in 16..=21 {
        for x in 7..=24 {
            let (dx, dy) = ((x - 16) as f64 / 9.0, (y - 18) as f64 / 3.5);
            if dx * dx + dy * dy <= 1.0 {
                s.set(x, y, if y <= 17 { WATER } else { HUD_SHADOW });
            }
        }
    }
    for x0 in [5, 25] {
        rect(&mut s, x0, 4, x0 + 1, 22, WOOD + 1);
        s.set(x0, 4, WOOD + 2);
    }
    for y in 0..=5 {
        rect(&mut s, 3 + (5 - y), y, 28 - (5 - y), y, if y < 2 { PLANK + 3 } else { PLANK + 2 });
    }
    rect(&mut s, 7, 9, 24, 9, WOOD + 2);
    rect(&mut s, 15, 10, 15, 16, HUD_HI);
    rect(&mut s, 26, 9, 28, 10, HUD_MID);
    s.outline()
}

// Tanglemere's (2026-11-04): the dyers' town, the outfitter's, the lake and the indigo field.

/// Indigo, darkest to lightest.
const INDIGO: [u8; 3] = [OUTLINE, WATER, WATER + 1];
/// Madder red.
const MADDER: [u8; 3] = [WOOD, THREAD, THREAD + 1];

/// A barrel of dye, its top open: indigo, madder red, or clear water for rinsing.
fn dye_barrel(dye: [u8; 3]) -> Sprite {
    let mut s = Sprite::blank(16, 16);
    // Staves, lit on the left, with two iron hoops.
    rect(&mut s, 2, 4, 13, 15, WOOD + 1);
    rect(&mut s, 2, 4, 4, 15, WOOD + 2);
    rect(&mut s, 12, 4, 13, 15, WOOD);
    for x in [6, 9] {
        rect(&mut s, x, 5, x, 15, WOOD);
    }
    for y in [7, 12] {
        rect(&mut s, 2, y, 13, y, HUD_MID);
    }
    // The open top: a rim, and the dye inside, catching the light.
    rect(&mut s, 2, 1, 13, 4, WOOD + 2);
    rect(&mut s, 3, 2, 12, 3, dye[1]);
    rect(&mut s, 3, 2, 12, 2, dye[0]);
    s.set(5, 3, dye[2]);
    s.set(6, 3, dye[2]);
    s.outline()
}

/// A low tub of dye steeping, a stain run down its side.
fn dye_tub(dye: [u8; 3]) -> Sprite {
    let mut s = Sprite::blank(16, 11);
    rect(&mut s, 1, 3, 14, 10, WOOD + 1);
    rect(&mut s, 1, 3, 2, 10, WOOD + 2);
    rect(&mut s, 13, 3, 14, 10, WOOD);
    rect(&mut s, 1, 7, 14, 7, HUD_MID);
    rect(&mut s, 1, 0, 14, 3, WOOD + 2);
    rect(&mut s, 2, 1, 13, 2, dye[1]);
    rect(&mut s, 3, 1, 6, 1, dye[2]);
    rect(&mut s, 9, 3, 10, 6, dye[1]);
    s.set(9, 7, dye[0]);
    s.outline()
}

/// Shelves of folded cloth, every color the town dyes.
fn cloth_shelf() -> Sprite {
    let mut s = Sprite::blank(16, 16);
    rect(&mut s, 1, 0, 14, 15, WOOD);
    rect(&mut s, 2, 1, 13, 14, WOOD + 1);
    let colors = [[THREAD + 1, WATER + 1, COTTON + 2, LEAF + 1], [WATER, SPARK + 1, THREAD + 1, FLOWER], [COTTON + 2, LEAF + 1, WATER + 1, THREAD]];
    for (row, cols) in colors.iter().enumerate() {
        let y = 2 + row as i32 * 4;
        for (k, &c) in cols.iter().enumerate() {
            let x = 2 + k as i32 * 3;
            rect(&mut s, x, y, x + 2, y + 2, c);
            s.set(x, y, shift(c, 1));
        }
        rect(&mut s, 2, y + 3, 13, y + 3, WOOD + 2);
    }
    s.outline()
}

/// A cutting table, two tiles long: a bolt of cloth half unrolled, and the shears.
fn cutting_table() -> Sprite {
    let mut s = Sprite::blank(32, 14);
    rect(&mut s, 1, 1, 30, 6, PLANK + 2);
    rect(&mut s, 1, 1, 30, 1, PLANK + 3);
    rect(&mut s, 1, 7, 30, 8, WOOD + 1);
    for x in [3, 27] {
        rect(&mut s, x, 9, x + 1, 13, WOOD);
    }
    // The bolt, and its cloth laid out flat.
    rect(&mut s, 3, 2, 5, 6, WATER + 1);
    rect(&mut s, 3, 2, 3, 6, WATER + 2);
    rect(&mut s, 6, 3, 15, 5, WATER + 2);
    rect(&mut s, 6, 5, 15, 5, WATER + 1);
    // The shears, open.
    for k in 0..5 {
        s.set(20 + k, 2 + k / 2, HUD_HI);
        s.set(20 + k, 5 - k / 2, HUD_HI);
    }
    rect(&mut s, 25, 3, 26, 4, THREAD + 1);
    s.set(27, 2, THREAD + 1);
    s.set(27, 5, THREAD + 1);
    s.outline()
}

/// A woven cloth hung on the wall, its pattern in the town's dyes.
fn hanging_cloth(c: u8) -> Sprite {
    let mut s = Sprite::blank(12, 16);
    rect(&mut s, 0, 0, 11, 1, WOOD + 2);
    rect(&mut s, 1, 2, 10, 14, c);
    for y in (3..14).step_by(3) {
        for x in (2..10).step_by(2) {
            s.set(x + (y / 3) % 2, y, COTTON + 2);
        }
    }
    rect(&mut s, 1, 2, 1, 14, shift(c, 1));
    for x in (1..11).step_by(2) {
        s.set(x, 15, c);
    }
    s.outline()
}

/// A dressmaker's dummy on its stand, a half-made jacket pinned to it.
fn dummy() -> Sprite {
    let mut s = Sprite::blank(16, 16);
    rect(&mut s, 7, 0, 8, 1, WOOD + 2);
    // The torso, shoulders wide, waist in.
    for (y, (x0, x1)) in [(2, (3, 12)), (3, (3, 12)), (4, (4, 11)), (5, (4, 11)), (6, (5, 10)), (7, (5, 10)), (8, (4, 11)), (9, (4, 11))] {
        rect(&mut s, x0, y, x1, y, MAT + 2);
    }
    rect(&mut s, 3, 2, 6, 7, THREAD + 1);
    rect(&mut s, 3, 2, 3, 6, THREAD + 2);
    rect(&mut s, 7, 3, 7, 8, HUD_HI);
    rect(&mut s, 7, 10, 8, 13, WOOD + 1);
    rect(&mut s, 4, 14, 11, 15, WOOD);
    rect(&mut s, 4, 14, 11, 14, WOOD + 2);
    s.outline()
}

/// A tall mirror in a wooden frame, for trying things on.
fn mirror() -> Sprite {
    let mut s = Sprite::blank(16, 16);
    rect(&mut s, 3, 0, 12, 13, WOOD + 2);
    rect(&mut s, 4, 1, 11, 12, WATER + 2);
    rect(&mut s, 4, 1, 11, 2, COTTON + 2);
    for k in 0..4 {
        s.set(5 + k, 9 - k * 2, COTTON + 2);
        s.set(5 + k, 10 - k * 2, COTTON + 2);
    }
    rect(&mut s, 3, 14, 4, 15, WOOD + 1);
    rect(&mut s, 11, 14, 12, 15, WOOD + 1);
    s.outline()
}

/// The outfitter's sign: a board on a bracket, a needle and its red thread painted on it.
fn shop_sign() -> Sprite {
    let mut s = Sprite::blank(16, 16);
    rect(&mut s, 13, 0, 14, 15, WOOD + 1);
    rect(&mut s, 2, 1, 14, 1, WOOD + 2);
    for x in [3, 11] {
        s.set(x, 2, HUD_MID);
    }
    rect(&mut s, 1, 3, 12, 11, PLANK + 2);
    rect(&mut s, 1, 3, 12, 3, PLANK + 3);
    for k in 0..7 {
        s.set(3 + k, 9 - k, HUD_HI);
    }
    s.set(9, 3, HUD_HI);
    for (x, y) in [(10, 4), (10, 5), (9, 6), (8, 7), (8, 8), (9, 9), (10, 9)] {
        s.set(x, y, THREAD + 1);
    }
    s.outline()
}

/// A heap of stones, ready to be burnt for lime.
fn stone_pile() -> Sprite {
    let mut s = Sprite::blank(16, 13);
    for &(cx, cy, r) in &[(4, 9, 3), (11, 9, 3), (7, 5, 3), (8, 10, 2)] {
        circle_fill(&mut s, cx, cy, r, STONE);
    }
    s.outline()
}

/// A tree stump, sawn flat: rings on its face, bark down its sides, roots at its foot.
fn stump() -> Sprite {
    let mut s = Sprite::blank(16, 13);
    rect(&mut s, 3, 4, 12, 10, WOOD + 1);
    rect(&mut s, 3, 4, 4, 10, WOOD + 2);
    rect(&mut s, 11, 4, 12, 10, WOOD);
    for x in [1, 2, 13, 14] {
        s.set(x, 11, WOOD);
    }
    rect(&mut s, 2, 10, 13, 11, WOOD);
    for y in 1..=6 {
        let (dy, half) = ((y - 3) as f64 / 2.6, 5.0);
        let w = (half * (1.0 - dy * dy).max(0.0).sqrt()).round() as i32;
        rect(&mut s, 8 - w, y, 7 + w, y, PLANK + 3);
    }
    for (x, y) in [(5, 3), (6, 2), (7, 2), (8, 2), (9, 2), (10, 3), (10, 4), (5, 4), (6, 5), (9, 5)] {
        s.set(x, y, PLANK + 1);
    }
    s.set(7, 4, PLANK + 1);
    s.set(8, 4, PLANK + 1);
    s.outline()
}

/// Reeds in the shallows, cattails nodding.
fn reeds(k: u8) -> Sprite {
    let mut s = Sprite::blank(16, 16);
    let stems: &[(i32, i32)] = if k == 0 { &[(3, 4), (6, 1), (9, 3), (12, 6)] } else { &[(4, 2), (8, 5), (11, 1)] };
    for &(x, top) in stems {
        rect(&mut s, x, top + 3, x, 15, LEAF + 1);
        s.set(x - 1, 13, LEAF);
        rect(&mut s, x, top, x, top + 2, WOOD + 1);
    }
    s.set(1, 12, LEAF + 2);
    s.set(14, 11, LEAF + 2);
    s
}

/// Lily pads afloat, one in flower.
fn lily_pads(k: u8) -> Sprite {
    let mut s = Sprite::blank(16, 16);
    let pads: &[(i32, i32, i32)] = if k == 0 { &[(5, 5, 3), (11, 10, 3)] } else { &[(9, 4, 2), (4, 11, 3)] };
    for &(cx, cy, r) in pads {
        circle_fill(&mut s, cx, cy, r, [LEAF, LEAF + 1, LEAF + 2]);
        s.set(cx + 1, cy, WATER + 1);
        s.set(cx + 2, cy, WATER + 1);
    }
    let (fx, fy) = (pads[0].0, pads[0].1);
    s.set(fx, fy - 1, FLOWER);
    s.set(fx - 1, fy, FLOWER);
    s.set(fx, fy, SPARK + 2);
    s
}

/// A rowing boat, two tiles long, tied up with its oars shipped.
fn rowboat() -> Sprite {
    let mut s = Sprite::blank(32, 14);
    for x in 1..31 {
        let taper = ((x - 1).min(30 - x) as f64).min(5.0) as i32;
        let (top, bottom) = (6 - taper, 7 + taper);
        rect(&mut s, x, top, x, bottom, WOOD + 1);
        s.set(x, top, WOOD + 3);
        s.set(x, bottom, WOOD);
        if bottom - top > 3 {
            rect(&mut s, x, top + 2, x, bottom - 2, PLANK + 1);
        }
    }
    for x in [11, 19] {
        rect(&mut s, x, 3, x + 1, 10, WOOD + 2);
    }
    rect(&mut s, 6, 6, 26, 6, PLANK + 3);
    s.outline()
}

/// A small field shed, two tiles each way: baskets and a hoe inside.
fn field_shed() -> Sprite {
    let mut s = Sprite::blank(32, 32);
    // The roof, boards running down it.
    for y in 1..15 {
        let inset = (15 - y).min(4) / 2;
        rect(&mut s, 1 + inset, y, 30 - inset, y, if y < 4 { WOOD + 3 } else { WOOD + 2 });
    }
    for x in (4..29).step_by(4) {
        rect(&mut s, x, 3, x, 14, WOOD + 1);
    }
    rect(&mut s, 1, 15, 30, 16, WOOD);
    // The front wall and its open door.
    rect(&mut s, 3, 17, 28, 31, WOOD + 1);
    for x in (5..28).step_by(4) {
        rect(&mut s, x, 17, x, 31, WOOD);
    }
    rect(&mut s, 12, 20, 19, 31, OUTLINE);
    rect(&mut s, 13, 21, 18, 31, HUD_SHADOW);
    rect(&mut s, 14, 27, 17, 30, MAT + 1);
    s.outline()
}

/// A scarecrow in a straw hat, arms out, keeping birds off the indigo.
fn scarecrow() -> Sprite {
    let mut s = Sprite::blank(16, 20);
    rect(&mut s, 7, 8, 8, 19, WOOD + 1);
    rect(&mut s, 1, 9, 14, 10, WATER + 1);
    rect(&mut s, 5, 8, 10, 14, WATER + 1);
    rect(&mut s, 5, 8, 5, 14, WATER + 2);
    for x in [1, 14] {
        s.set(x, 11, MAT + 2);
    }
    rect(&mut s, 5, 3, 10, 7, MAT + 2);
    s.set(6, 5, OUTLINE);
    s.set(9, 5, OUTLINE);
    rect(&mut s, 3, 2, 12, 3, SPARK + 1);
    rect(&mut s, 5, 0, 10, 2, SPARK);
    s.outline()
}

/// A wooden trough of clean water, two tiles long, for rinsing the leaves.
fn field_trough() -> Sprite {
    let mut s = Sprite::blank(32, 12);
    rect(&mut s, 1, 2, 30, 11, WOOD + 1);
    rect(&mut s, 1, 2, 30, 2, WOOD + 3);
    rect(&mut s, 1, 8, 30, 8, WOOD);
    rect(&mut s, 3, 3, 28, 6, WATER + 1);
    for x in [7, 8, 18, 24] {
        s.set(x, 4, WATER + 2);
    }
    for x in [2, 28] {
        rect(&mut s, x, 9, x + 1, 11, WOOD);
    }
    s.outline()
}

/// A shrub clipped round: nothing on it to pick.
fn shrub() -> Sprite {
    let mut s = Sprite::blank(16, 14);
    circle_fill(&mut s, 8, 7, 6, CANOPY);
    for &(x, y) in &[(5, 4), (9, 3), (11, 7), (6, 9)] {
        s.set(x, y, HEDGE + 2);
    }
    s.outline()
}

/// A pair of woven baskets, the back one higher: each a lit rim round its dark open top, then
/// the weave, narrowing a little to its foot.
fn baskets() -> Sprite {
    let mut s = Sprite::blank(16, 13);
    for &(x0, y0, x1) in &[(8, 2, 14), (1, 4, 7)] {
        rect(&mut s, x0, y0, x1, y0, MAT + 2);
        rect(&mut s, x0, y0 + 1, x1, y0 + 1, MAT + 2);
        rect(&mut s, x0 + 1, y0 + 1, x1 - 1, y0 + 1, WOOD);
        rect(&mut s, x0, y0 + 2, x1, y0 + 2, MAT + 2);
        for y in y0 + 3..=12 {
            let inset = if y == 12 { 1 } else { 0 };
            for x in x0 + inset..=x1 - inset {
                s.set(x, y, if (x + y) % 2 == 0 { MAT + 1 } else { MAT });
            }
        }
    }
    s.outline()
}

/// Empty sacks folded flat and piled, a tie round the top one: burlap, not a sack of anything.
fn sacks() -> Sprite {
    let mut s = Sprite::blank(16, 12);
    for &(x0, y0, x1, y1) in &[(1, 8, 14, 11), (2, 5, 13, 8), (3, 2, 12, 5)] {
        rect(&mut s, x0, y0, x1, y1, MAT + 1);
        rect(&mut s, x0, y0, x1, y0, MAT + 2);
        rect(&mut s, x0, y1, x1, y1, MAT);
        // A fold line down the middle.
        s.set((x0 + x1) / 2, y0 + 1, MAT);
    }
    rect(&mut s, 7, 2, 8, 5, WOOD + 2);
    s.outline()
}
