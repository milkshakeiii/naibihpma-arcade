use crate::econ::{quality_level, EXCEPTIONAL, MASTERWORK};
use crate::inspect::{Icon, Line, Meter};
use crate::framebuffer::Framebuffer;
use crate::game::{Game, Species};
use crate::inspect;
use crate::palette::*;
use crate::sprites::{Sprite, Sprites};
use crate::tutorial::{self, wrap, HELP, STAGES};
use crate::world::*;

/// The sprites a workshop kind's `look` can name (`data/workshops.json`).
pub const LOOKS: [&str; 12] = ["bench", "wheel", "loom", "kiln", "indigo_vat", "dye_vat", "wagon_bench", "mound", "forge", "smelter", "sawpit", "salt_pan"];

pub const VIEW_H: i32 = SCREEN_H * TS; // 256; the HUD fills the remaining 44 rows
const HOP: [i32; 7] = [0, 2, 4, 5, 5, 4, 2];

fn hash(x: i32, y: i32) -> u32 {
    let mut h = (x as u32).wrapping_mul(0x8da6_b343) ^ (y as u32).wrapping_mul(0xd816_3841);
    h ^= h >> 13;
    h = h.wrapping_mul(0x85eb_ca6b);
    h ^ (h >> 16)
}

/// Smooth noise over the world's pixels, 0 to 255: a hashed value at every `cell`-th pixel each
/// way, blended between them.
fn smooth(px: i32, py: i32, cell: i32) -> i32 {
    let (gx, gy, fx, fy) = (px.div_euclid(cell), py.div_euclid(cell), px.rem_euclid(cell), py.rem_euclid(cell));
    let at = |x: i32, y: i32| (hash(x * 31 + cell, y) % 256) as i32;
    (at(gx, gy) * (cell - fx) * (cell - fy) + at(gx + 1, gy) * fx * (cell - fy) + at(gx, gy + 1) * (cell - fx) * fy + at(gx + 1, gy + 1) * fx * fy) / (cell * cell)
}

/// Flip-screen camera: the top-left tile of the screen the controlled monster stands on.
pub fn screen_of(x: i32, y: i32) -> (i32, i32) {
    (x.div_euclid(SCREEN_W) * SCREEN_W, y.div_euclid(SCREEN_H) * SCREEN_H)
}

/// One player's view: the screen their controlled monster is on.
struct View {
    tx: i32,
    ty: i32,
    /// The look of the province it shows: its land's colors and its buildings' materials.
    style: Style,
    /// Whose view: the commune, and the monster its player controls.
    k: usize,
    c: usize,
}

impl View {
    fn of(g: &Game, k: usize) -> Self {
        let c = g.communes[k].controlled;
        let m = &g.monsters[c];
        let (tx, ty) = screen_of(m.x, m.y);
        Self { tx, ty, style: g.style.unwrap_or_else(|| g.world.style_at(tx, ty)), k, c }
    }

    fn sees(&self, x: i32, y: i32) -> bool {
        x >= self.tx && y >= self.ty && x < self.tx + SCREEN_W && y < self.ty + SCREEN_H
    }

    /// Screen pixel of a tile's top-left corner.
    fn px(&self, x: i32, y: i32) -> (i32, i32) {
        ((x - self.tx) * TS, (y - self.ty) * TS)
    }
}

/// The look commune `k`'s player sees, whose palette shows their picture: their screen's
/// province's.
pub fn style_of(g: &Game, k: usize) -> Style {
    View::of(g, k).style
}

/// What commune `k`'s player sees.
pub fn draw(g: &Game, k: usize, sp: &Sprites, fb: &mut Framebuffer) {
    let v = View::of(g, k);
    let me = &g.communes[k];
    draw_ground(g, sp, fb, &v);
    draw_kit_ghost(g, fb, &v);
    // How many more goods wait inside a workshop or a harvested plant.
    for s in &g.world.shops {
        if let Some(&(x, y)) = s.outs.first().filter(|_| !s.pending.is_empty()) {
            if v.sees(x, y) {
                let (px, py) = v.px(x, y);
                badge(fb, px, py, s.pending.len());
            }
        }
    }
    for p in g.world.plants.iter().filter(|p| !p.stock.is_empty() && v.sees(p.x, p.y)) {
        let (px, py) = v.px(p.x, p.y);
        badge(fb, px, py, p.stock.len());
    }
    draw_flags(g, sp, fb, &v);
    draw_bodies(g, sp, fb, &v);
    draw_overhead(g, fb, &v);
    draw_particles(g, fb, &v);
    draw_markers(g, sp, fb, &v);
    draw_edge_markers(g, fb, &v);
    draw_popups(g, fb, &v);
    draw_progress(g, fb, &v);
    for b in bubbles(g, &v) {
        draw_bubble(fb, sp, &b);
    }
    draw_floaters(g, fb, &v);
    draw_banner(me, fb);
    draw_announcement(g, fb);
    if me.inspecting && !me.help {
        draw_card(g, sp, fb, &v);
    }
    if me.roster && !me.help {
        draw_roster(g, sp, fb, &v);
    }
    draw_hud(g, sp, fb, &v);
    if me.sheet {
        draw_sheet(g, sp, fb, k);
    }
    if me.help {
        draw_help(fb);
    }
    if me.leaving {
        draw_leaving(fb);
    }
}

/// The screen whose top-left tile is `(tx, ty)` as a place, for the layout workflow's pictures
/// (DESIGN.md, Layout by image model): its ground, what stands on it and whoever is about, with
/// nobody's flags, bubbles or HUD.
pub fn draw_place(g: &Game, sp: &Sprites, fb: &mut Framebuffer, (tx, ty): (i32, i32)) {
    let v = View { tx, ty, style: g.style.unwrap_or_else(|| g.world.style_at(tx, ty)), k: 0, c: g.communes[0].controlled };
    draw_ground(g, sp, fb, &v);
    draw_bodies(g, sp, fb, &v);
    draw_overhead(g, fb, &v);
}

/// The ground of `v`'s screen and what stands on it: tiles, trees, buildings' names, plants,
/// boards and signs, workshops and bins.
fn draw_ground(g: &Game, sp: &Sprites, fb: &mut Framebuffer, v: &View) {
    fb.rect_fill(0, 0, 400, VIEW_H, OUTLINE);
    for ty in v.ty..v.ty + SCREEN_H {
        for tx in v.tx..v.tx + SCREEN_W {
            draw_tile(g, sp, fb, v, tx, ty);
        }
    }
    draw_trees(g, sp, fb, v);
    draw_building_labels(g, fb, v);
    for p in g.world.plants.iter().filter(|p| v.sees(p.x, p.y)) {
        let (x, y) = v.px(p.x, p.y);
        let s = sp.plant(&p.good, g.data.try_good(&p.good), p.stage());
        fb.blit(s, x + (TS - s.w) / 2, y + TS - s.h, false);
    }
    draw_boards_and_signs(g, sp, fb, v);
    draw_vine_feet(g, fb, v);
    for s in g.world.shops.iter().filter(|s| v.sees(s.spot.0, s.spot.1) && s.frontier.as_ref().is_none_or(|f| !f.fallen)) {
        let (x, y) = v.px(s.spot.0, s.spot.1);
        let (art, dy) = match s.kind.look() {
            "wheel" => (&sp.wheel[if s.job.is_some() { (g.frame / 2 % 2) as usize } else { 0 }], -5),
            "loom" => (&sp.loom, -5),
            "kiln" => (&sp.kiln[if s.job.is_some() { (g.frame / 3 % 2) as usize } else { 0 }], -5),
            "mound" => (&sp.mound[if s.job.is_some() { 1 + (g.frame / 5 % 2) as usize } else { 0 }], TS - 22),
            "forge" => (&sp.forge[if s.job.is_some() { 1 + (g.frame / 2 % 2) as usize } else { 0 }], TS - 24),
            "sawpit" => (&sp.sawpit, 0),
            "salt_pan" => (&sp.salt_pan[if s.job.is_some() { 1 + (g.frame / 3 % 2) as usize } else { 0 }], TS - 20),
            "smelter" => (&sp.smelter[if s.job.is_some() { 1 + (g.frame / 4 % 2) as usize } else { 0 }], TS - 26),
            "indigo_vat" => (&sp.indigo_vat, -4),
            "dye_vat" => (&sp.dye_vat, -4),
            "wagon_bench" => (&sp.wagon_bench, TS - sp.wagon_bench.h - 4),
            _ => (&sp.bench, TS - sp.bench.h - 4),
        };
        // A frontier workshop in its material's tones: a hemlock one lighter than an oak one.
        let wood = s.frontier.as_ref().and_then(|f| g.data.variety(&f.material.to_lowercase())).and_then(|v| v.color.as_deref()).and_then(ramp_named);
        blit_quality(fb, art, x, y + dy, 0, wood_tint(wood));
    }
    for b in g.world.bins.iter().filter(|b| v.sees(b.x, b.y)) {
        draw_bin(g, sp, fb, v, b);
    }
}

/// The character sheet (C), the one full-screen page, kept by tradition: the monster itself,
/// then (Tab) its loop. The world goes on behind it.
fn draw_sheet(g: &Game, sp: &Sprites, fb: &mut Framebuffer, k: usize) {
    let me = &g.communes[k];
    let m = &g.monsters[me.controlled];
    fb.rect_bevel(0, 0, 400, 300, HUD_FILL, HUD_HI, HUD_SHADOW);
    let title = match me.sheet_page {
        0 => format!("{} THE {}", m.name, m.kind.to_uppercase()),
        1 => format!("{}'S LOOP", m.name),
        _ => "THE COMMUNE'S RECIPE BOOK".into(),
    };
    fb.text_centered(&title, 200, 8, SPARK + 2, Some(HUD_SHADOW), 1);
    fb.rect_fill(8, 20, 384, 1, HUD_SHADOW);
    match me.sheet_page {
        0 => sheet_monster(g, sp, fb, me.controlled, me.sheet_hand, me.sheet_confirm),
        1 => sheet_loop(g, fb, me.controlled),
        _ => sheet_book(g, fb, k),
    }
    fb.rect_fill(8, SHEET_KEYS_Y, 384, 1, HUD_SHADOW);
    let keys = match me.sheet_page {
        0 => "UP/DOWN: CHOOSE  SPACE: PUT DOWN, TAKE OFF  TAB: LOOP  C: CLOSE",
        1 => "TAB: RECIPE BOOK   C: CLOSE",
        _ => "UP/DOWN: SCROLL   TAB: BACK   C: CLOSE",
    };
    fb.text_centered(keys, 200, 287, TEXT_DIM, Some(HUD_SHADOW), 1);
}

/// Where the sheet's keys begin, under a rule: nothing on a page may reach it.
const SHEET_KEYS_Y: i32 = 282;

/// Page 1: a big portrait, talents, hands, its slots and what's in them, its capacities, how
/// seasoned it is, its abilities and what each is for, and its skills as bars.
fn sheet_monster(g: &Game, sp: &Sprites, fb: &mut Framebuffer, i: usize, chosen: usize, confirm: Option<usize>) {
    let m = &g.monsters[i];
    let t = |fb: &mut Framebuffer, x: i32, y: i32, text: &str, col: u8| fb.text(text, x, y, col, Some(HUD_SHADOW), 1);
    let dressed = dressed(g, i);
    let s = match &dressed {
        Some(frames) => &frames[0],
        None => &(if m.species == Species::Goblin { &sp.goblin } else { &sp.cyclops })[0],
    };
    // Every pixel three across and three down: whole pixels only.
    for sy in 0..s.h {
        for sx in 0..s.w {
            let c = s.px[(sy * s.w + sx) as usize];
            if c != 0 {
                fb.rect_fill(16 + sx * 3, 28 + sy * 3, 3, 3, c);
            }
        }
    }
    t(fb, 76, 28, "TALENTS", SPARK + 1);
    // Its own talents, then each one taught, closer together if there are many (an elf with
    // three teachings) so they stay clear of its hands.
    let own = m.skills.transferable.iter().map(|t| (t.replace('_', " ").to_uppercase(), TEXT));
    let taught = m.knack(&g.data, "").talents.into_iter().map(|t| (format!("{}, TAUGHT", t.replace('_', " ").to_uppercase()), TEXT_GOOD));
    let talents: Vec<(String, u8)> = own.chain(taught).collect();
    let pitch = if talents.len() > 5 { 9 } else { 10 };
    for (n, (name, col)) in talents.iter().enumerate() {
        t(fb, 76, 40 + n as i32 * pitch, name, *col);
    }
    // A small arrow at the chosen line.
    let arrow = |fb: &mut Framebuffer, y: i32| {
        for (dx, dy) in [(0, 0), (0, 1), (1, 1), (0, 2), (1, 2), (2, 2), (0, 3), (1, 3), (0, 4)] {
            fb.pset(17 + dx, y + dy, TEXT_GOOD);
        }
    };
    // Each section's heading, then its lines, each two rows: what's there, and its details.
    let (mut y, mut section) = (84, None);
    for (n, l) in g.sheet_lines(i).iter().enumerate() {
        let (slot, held) = match l {
            crate::gear::SheetLine::Hand(h) => (crate::data::Slot::Hand, m.hands[*h].as_ref()),
            crate::gear::SheetLine::Gear(slot, _) => (*slot, None),
        };
        if section != Some(slot) {
            section = Some(slot);
            t(fb, 16, y, slot.label(), SPARK + 1);
            y += 12;
        }
        if n == chosen {
            arrow(fb, y);
        }
        let piece = match (l, held) {
            (_, Some(crate::game::Held::Piece(id))) | (crate::gear::SheetLine::Gear(_, Some(id)), _) => g.data.try_piece(id),
            _ => None,
        };
        match (held.and_then(crate::game::Held::tool), piece) {
            (Some(d), _) => {
                t(fb, 26, y, &g.good_name(&d.good), TEXT);
                t(fb, 26, y + 9, &format!("QUALITY {:.2}, DURABILITY {}/{}", d.quality, d.durability_left(&g.data), d.durability(&g.data)), TEXT_DIM);
            }
            (None, Some(p)) => {
                t(fb, 26, y, &p.name.to_uppercase(), TEXT);
                if confirm == Some(n) {
                    t(fb, 26, y + 9, "SPACE AGAIN: GONE FOR GOOD", TEXT_WARN);
                } else {
                    t(fb, 26, y + 9, &p.effect_text(), TEXT_GOOD);
                }
            }
            // A tool belt's hand, empty, says whose it is.
            (None, None) => match l {
                crate::gear::SheetLine::Hand(h) if *h >= g.data.monster_types[m.kind.as_str()].slots.hands as usize => {
                    let belt = m.gear.iter().filter_map(|id| g.data.try_piece(id)).find(|p| p.does.hands > 0).map_or("THE BELT".to_string(), |p| format!("THE {}", p.name.to_uppercase()));
                    t(fb, 26, y, &format!("EMPTY: ON {belt}"), TEXT_DIM)
                }
                _ => t(fb, 26, y, "EMPTY", TEXT_DIM),
            },
        }
        // Two rows for what's there and its details, one for an empty slot: room for a tool
        // belt's third hand and every slot full above the capacities.
        y += if held.is_some() || piece.is_some() { 19 } else { 11 };
    }
    // Its capacities, what it has earned and how seasoned it is, a line each, as many as fit
    // above the keys: with every slot full and renown earned, how seasoned it is waits (Henry).
    let ty = &g.data.monster_types[m.kind.as_str()];
    let mut below = vec![(format!("BODY {:.0} OF {}   MIND {} OF {}", g.body_used(i), ty.body_cap, g.mind_used(i), ty.mind_cap), TEXT)];
    if m.earned > 0.0 {
        below.push((format!("EARNED {} RENOWN", m.earned.floor()), TEXT));
    }
    let seasoned = m.seasoned();
    let next = crate::gear::SEASONED_AT.get(seasoned as usize).map_or(String::new(), |at| format!("{}AT {at}", if seasoned > 0 { "AGAIN " } else { "" }));
    below.push(match seasoned {
        0 => (format!("SEASONED {next}"), TEXT_DIM),
        n => (format!("SEASONED: +{n} TO ALL{}", if next.is_empty() { String::new() } else { format!(", {next}") }), TEXT_GOOD),
    });
    for (n, (text, col)) in below.iter().enumerate() {
        let ly = y + 2 + n as i32 * 11;
        if ly + 8 > SHEET_KEYS_Y - 1 {
            break;
        }
        t(fb, 16, ly, text, *col);
    }
    t(fb, 200, 28, "ABILITIES", SPARK + 1);
    let st = m.abilities(&g.data);
    for (a, name) in crate::data::ABILITIES.into_iter().enumerate() {
        let y = 40 + a as i32 * 10;
        let v = st.get(a);
        t(fb, 200, y, &format!("{name} {v:>2}"), if v > m.stats.get(a) { TEXT_GOOD } else { TEXT });
        t(fb, 242, y, &inspect::ability_uses(g, a, m.leads_by(&g.data)), TEXT_DIM);
    }
    // Its type's ability, by name and what it does, then its skills below.
    let skills_y = match m.ability(&g.data) {
        Some(ab) => {
            t(fb, 200, 100, &ab.name.to_uppercase(), SPARK + 1);
            t(fb, 200, 110, &ab.does.text().join(", "), TEXT_GOOD);
            124
        }
        None => 108,
    };
    t(fb, 200, skills_y, "SKILLS", SPARK + 1);
    let mut skills: Vec<(String, f64)> = m.skills.applied.keys().map(|k| (k.clone(), m.skills.applied(k))).filter(|(_, v)| *v > 0.0).collect();
    skills.sort_by(|a, b| b.1.total_cmp(&a.1));
    if skills.is_empty() {
        t(fb, 200, skills_y + 12, "NONE YET", TEXT_DIM);
    }
    // As many as fit above the keys.
    let fit = ((SHEET_KEYS_Y - 1 - 8 - (skills_y + 12)) / 10 + 1) as usize;
    for (n, (k, v)) in skills.into_iter().take(fit).enumerate() {
        let y = skills_y + 12 + n as i32 * 10;
        t(fb, 200, y, &k.replace('_', " ").to_uppercase(), TEXT);
        fb.rect_fill(330, y, 52, 6, OUTLINE);
        fb.rect_fill(331, y + 1, ((v.clamp(0.0, 1.0) * 50.0) as i32).max(1), 4, SPARK + 1);
    }
}

/// Page 3: the commune's recipe book. By workshop, everything its monsters have made, and in
/// grey what a craftsman has told of, each with what it takes.
fn sheet_book(g: &Game, fb: &mut Framebuffer, k: usize) {
    let c = &g.communes[k];
    let mut rows: Vec<(String, u8)> = Vec::new();
    for kind in crate::world::ShopKind::all() {
        let known: Vec<&str> = kind.recipes().iter().copied().filter(|r| c.made.contains(*r) || c.heard.contains(*r)).collect();
        if known.is_empty() {
            continue;
        }
        rows.push((kind.label().to_string(), SPARK + 1));
        for r in known {
            let gt = g.data.good(r);
            // Heard of but not made yet: in grey.
            let col = if c.made.contains(r) { TEXT } else { TEXT_DIM };
            for (n, row) in wrap(&format!("{}: {}", gt.name.to_uppercase(), g.inputs_text(gt)), 58).into_iter().enumerate() {
                rows.push((if n == 0 { format!("  {row}") } else { format!("    {row}") }, col));
            }
        }
    }
    if rows.is_empty() {
        for (n, row) in wrap("NOTHING YET. WHAT YOUR MONSTERS MAKE, AND WHAT A CRAFTSMAN TELLS YOU OF, IS WRITTEN HERE.", 58).iter().enumerate() {
            fb.text(row, 16, 30 + n as i32 * 10, TEXT_DIM, Some(HUD_SHADOW), 1);
        }
        return;
    }
    const FIT: usize = 24;
    let first = c.sheet_scroll.min(rows.len().saturating_sub(FIT));
    for (n, (row, col)) in rows.iter().skip(first).take(FIT).enumerate() {
        fb.text(row, 16, 30 + n as i32 * 10, *col, Some(HUD_SHADOW), 1);
    }
    if first > 0 {
        fb.text("MORE ABOVE", 316, 24, TEXT_DIM, Some(HUD_SHADOW), 1);
    }
    if first + FIT < rows.len() {
        fb.text("MORE BELOW", 316, 272, TEXT_DIM, Some(HUD_SHADOW), 1);
    }
}

/// Page 2: its loop, goal by goal, the one it's on picked out while it's looping.
fn sheet_loop(g: &Game, fb: &mut Framebuffer, i: usize) {
    let m = &g.monsters[i];
    let Some(r) = m.routine.as_ref() else {
        fb.text("NO LOOP RECORDED YET. R RECORDS ONE.", 16, 30, TEXT_DIM, Some(HUD_SHADOW), 1);
        return;
    };
    let mut y = 30;
    for (n, goal) in r.goals.iter().enumerate() {
        let col = if m.playing && n == m.pc { TEXT_GOOD } else { TEXT };
        for (k, row) in wrap(&inspect::goal_text(g, goal), 56).iter().enumerate() {
            if y > 270 {
                return;
            }
            let lead = if k == 0 { format!("{:>2}. ", n + 1) } else { "    ".into() };
            fb.text(&format!("{lead}{row}"), 16, y, col, Some(HUD_SHADOW), 1);
            y += 10;
        }
    }
}

fn draw_tile(g: &Game, sp: &Sprites, fb: &mut Framebuffer, v: &View, tx: i32, ty: i32) {
    let (x0, y0) = v.px(tx, ty);
    let (wx, wy) = (tx * TS, ty * TS);
    let tile = g.world.tile(tx, ty);
    let ground = ground(g, tx, ty);
    for y in 0..TS {
        for x in 0..TS {
            let h = hash(wx + x, wy + y);
            let c = match ground {
                Tile::Grass | Tile::Bin | Tile::Tree | Tile::Sign => grass(h),
                Tile::Water | Tile::Footbridge => {
                    // Slow ripples drifting downstream; a lighter lip where the bank begins.
                    let drift = (y + (g.frame / 10) as i32) % TS;
                    if y == 0 && g.world.tile(tx, ty - 1) != Tile::Water {
                        LAND_WATER + 2
                    } else if (x + drift * 5 + (h % 3) as i32) % 17 == 0 && drift % 4 == 1 {
                        LAND_WATER + 2
                    } else if (x * 3 + drift * 7) % 31 == 0 {
                        LAND_WATER
                    } else {
                        LAND_WATER + 1
                    }
                }
                Tile::Peg | Tile::Pad | Tile::OutMat | Tile::Spot if !indoors(g, tx, ty) => match h % 23 {
                    0 => GRASS + 3,
                    1 | 2 => GRASS + 2,
                    3 => GRASS,
                    _ => GRASS + 1,
                },
                Tile::Paving => {
                    // Cobbles set in the dirt: small rounded stones in staggered rows, each lit
                    // at its top left and shaded at its bottom right.
                    let off = if (y / 4) % 2 == 0 { 0 } else { 2 };
                    let (sx, sy) = ((x + off + tx * 16) % 4, y % 4);
                    if sx == 3 || sy == 3 {
                        LAND_PATH + 1
                    } else if (sx == 0 && sy == 0) || (sx == 2 && sy == 2) {
                        if sx == 0 { TEXT_DIM } else { HUD_MID }
                    } else if h % 11 == 0 {
                        HUD_MID
                    } else {
                        HUD_HI
                    }
                }
                Tile::Path => match h % 17 {
                    0 => LAND_PATH + 2,
                    1 | 2 => LAND_PATH,
                    _ => LAND_PATH + 1,
                },
                Tile::Gravel => {
                    // A path's earth, a little darker, with a pebble here and there, lit on top.
                    let (cx, cy) = ((wx + x) / 4, (wy + y) / 4);
                    if hash(cx, cy) % 13 == 0 && x % 4 < 2 && y % 4 < 2 {
                        if x % 4 == 0 && y % 4 == 0 { LAND_PATH + 2 } else { WALL + 2 }
                    } else {
                        match h % 13 {
                            0 => LAND_PATH + 2,
                            1..=5 => LAND_PATH,
                            _ => LAND_PATH + 1,
                        }
                    }
                }
                Tile::Stair => {
                    // Steps climbing north: each a lit nosing, its tread, and a shadowed riser.
                    let edge = (x == 0 && g.world.tile(tx - 1, ty) != Tile::Stair) || (x == TS - 1 && g.world.tile(tx + 1, ty) != Tile::Stair);
                    match y % 4 {
                        _ if edge => HUD_FILL,
                        0 => TEXT_DIM,
                        1 | 2 => if h % 13 == 0 { HUD_HI } else { HUD_MID },
                        _ => HUD_FILL,
                    }
                }
                Tile::Hedge => {
                    let lit = (x + y + (h % 3) as i32) % 5 == 0;
                    if y < 2 && g.world.tile(tx, ty - 1) != Tile::Hedge {
                        HEDGE + 2
                    } else if lit {
                        HEDGE + 1
                    } else if h % 7 == 0 {
                        HEDGE + 2
                    } else {
                        HEDGE
                    }
                }
                Tile::Boards => {
                    // Planks laid across the way, a dark gap between each, their sides edged.
                    let across = g.world.tile(tx - 1, ty) == Tile::Boards || g.world.tile(tx + 1, ty) == Tile::Boards;
                    let (k, side, before, after) = if across { (x, y, (0, -1), (0, 1)) } else { (y, x, (-1, 0), (1, 0)) };
                    let edge = (side == 0 && g.world.tile(tx + before.0, ty + before.1) != Tile::Boards) || (side == TS - 1 && g.world.tile(tx + after.0, ty + after.1) != Tile::Boards);
                    if edge {
                        WOOD
                    } else if (k + 1) % 4 == 0 {
                        PLANK
                    } else if (k + 1) % 4 == 1 {
                        PLANK + 3
                    } else if h % 9 == 0 {
                        PLANK + 1
                    } else {
                        PLANK + 2
                    }
                }
                Tile::Sinter => {
                    // The springs' crust: pale, rippled in rims where the overflow ran and
                    // dried, each rim lit along its lip; ragged where it gives out into grass.
                    let (px, py) = (wx + x, wy + y);
                    let n = (smooth(px, py, 16) * 2 + smooth(px, py, 7)) / 3;
                    let open_side = |dx: i32, dy: i32| !matches!(g.world.tile(tx + dx, ty + dy), Tile::Sinter | Tile::Boards) && self::ground(g, tx + dx, ty + dy) != Tile::Sinter;
                    let fringe = [(-1, 0, x), (1, 0, TS - 1 - x), (0, -1, y), (0, 1, TS - 1 - y)].iter().filter(|&&(dx, dy, _)| open_side(dx, dy)).map(|&(_, _, d)| d).min();
                    if fringe.is_some_and(|d| d < smooth(px, py, 9) / 36) {
                        grass(h)
                    } else if n % 30 == 0 {
                        LAND_PATH + 1
                    } else if n % 30 == 1 {
                        COTTON + 1
                    } else if h % 29 == 0 {
                        LAND_PATH + 1
                    } else {
                        // Soft patches where it's older and duller, dithered at their edges.
                        let m = smooth(px, py, 24);
                        let duller = if (m - 150).abs() < 8 { (px + py) % 2 == 0 } else { m > 150 };
                        if duller { LAND_PATH + 2 } else { COTTON }
                    }
                }
                Tile::Wall => wall_px(v.style.walls(), x, y, (tx, ty), g.world.tile(tx, ty + 1) != Tile::Wall, h),
                _ => floor_px(v.style.floors(), x, y, tx, h),
            };
            fb.pset(x0 + x, y0 + y, c);
        }
    }
    if ground == Tile::Grass && tile == Tile::Grass && hash(tx, ty) % 7 == 0 && g.world.plant_at(tx, ty).is_none() {
        let (fx, fy) = (x0 + 3 + (hash(ty, tx) % 9) as i32, y0 + 4 + (hash(tx + 7, ty) % 8) as i32);
        let col = LAND_FLOWER + (hash(tx, ty + 3) % 2) as u8;
        for (dx, dy) in [(0, -1), (-1, 0), (1, 0), (0, 1)] {
            fb.pset(fx + dx, fy + dy, col);
        }
        fb.pset(fx, fy, SPARK + 2);
    }
    if tile == Tile::Bridge {
        for (dy, row) in [(-1, 0), (1, TS - 2)] {
            if g.world.tile(tx, ty + dy) != Tile::Bridge {
                fb.rect_fill(x0, y0 + row, TS, 2, WOOD);
                fb.rect_fill(x0, y0 + row + if dy < 0 { 1 } else { 0 }, TS, 1, WOOD + 2);
            }
        }
    }
    if tile == Tile::Footbridge {
        // Planks laid across between two ropes, and a post where each end meets the bank.
        for x in 0..TS {
            let seam = (x + tx * 2) % 4 == 3;
            fb.rect_fill(x0 + x, y0 + 4, 1, 8, if seam { PLANK } else { PLANK + 1 });
            fb.pset(x0 + x, y0 + 4, PLANK + 2);
        }
        for row in [3, 12] {
            fb.rect_fill(x0, y0 + row, TS, 1, OUTLINE);
        }
        for (dx, px) in [(-1, 0), (1, TS - 2)] {
            if g.world.tile(tx + dx, ty) != Tile::Footbridge {
                fb.rect_fill(x0 + px, y0 + 1, 2, 13, OUTLINE);
                fb.rect_fill(x0 + px, y0 + 2, 1, 11, WOOD + 2);
            }
        }
    }
    if tile == Tile::Wall {
        for (dx, dy, edge) in [(0, -1, 0), (0, 1, 1), (-1, 0, 2), (1, 0, 3)] {
            if g.world.tile(tx + dx, ty + dy) != Tile::Wall {
                match edge {
                    0 => fb.rect_fill(x0, y0, TS, 1, OUTLINE),
                    1 => fb.rect_fill(x0, y0 + TS - 1, TS, 1, OUTLINE),
                    2 => fb.rect_fill(x0, y0, 1, TS, OUTLINE),
                    _ => fb.rect_fill(x0 + TS - 1, y0, 1, TS, OUTLINE),
                }
            }
        }
    }
    if tile == Tile::Rack {
        if let Some(r) = g.rack_at(tx, ty) {
            draw_rack(g, fb, g.world.racks[r].piece, x0, y0);
        }
    }
    if tile == Tile::Gate {
        // A gate shut across the way: upright boards banded with iron, a ring to pull on the
        // middle one, and a squat stone tower at each end.
        let gate = |dy: i32| g.world.tile(tx, ty + dy) == Tile::Gate;
        fb.rect_fill(x0 + 2, y0, 12, TS, OUTLINE);
        fb.rect_fill(x0 + 3, y0, 10, TS, WOOD + 1);
        fb.rect_fill(x0 + 3, y0, 1, TS, WOOD + 2);
        for bx in [6, 9, 12] {
            fb.rect_fill(x0 + bx, y0, 1, TS, WOOD);
        }
        for by in [4, 11] {
            fb.rect_fill(x0 + 2, y0 + by, 12, 2, HUD_MID);
            fb.rect_fill(x0 + 2, y0 + by, 12, 1, HUD_HI);
            for rx in [4, 7, 10] {
                fb.pset(x0 + rx, y0 + by + 1, TEXT_DIM);
            }
        }
        if gate(-1) && gate(1) {
            fb.rect_fill(x0 + 6, y0 + 7, 4, 3, OUTLINE);
            fb.rect_fill(x0 + 7, y0 + 8, 2, 1, SPARK);
        }
        for (dy, py) in [(-1, 0), (1, TS - 7)] {
            if !gate(dy) {
                fb.rect_fill(x0, y0 + py, TS, 7, OUTLINE);
                fb.rect_fill(x0 + 1, y0 + py + 1, TS - 2, 5, HUD_HI);
                fb.rect_fill(x0 + 1, y0 + py + 1, TS - 2, 1, TEXT_DIM);
                for jx in [5, 10] {
                    fb.rect_fill(x0 + jx, y0 + py + 2, 1, 4, HUD_MID);
                }
            }
        }
    }
    if tile == Tile::Door {
        if g.world.tile(tx, ty - 1) == Tile::Wall || g.world.tile(tx, ty + 1) == Tile::Wall {
            fb.rect_fill(x0 + 6, y0, 4, TS, PLANK);
            fb.rect_fill(x0 + 7, y0, 2, TS, PLANK + 2);
        } else {
            fb.rect_fill(x0, y0 + 6, TS, 4, PLANK);
            fb.rect_fill(x0, y0 + 7, TS, 2, PLANK + 2);
        }
        // A shop's awning runs on over its door.
        if [-1, 1].iter().all(|&d| g.world.prop_at(tx + d, ty).is_some_and(|(_, p)| p.0 == "shop_awning")) {
            canopy(fb, x0, y0, tx);
        }
    }
    // A charcoal mound sits on bare earth, burning.
    let mound = || g.world.shop_at_spot(tx, ty).is_some_and(|s| g.world.shops[s].kind.look() == "mound");
    if tile == Tile::Spot && !mound() {
        // A rug under the work spot so it reads at a glance.
        fb.rect_fill(x0 + 1, y0 + 1, TS - 2, TS - 2, THREAD + 1);
        fb.rect_fill(x0 + 2, y0 + 2, TS - 4, TS - 4, THREAD);
        for k in (3..TS - 3).step_by(3) {
            fb.pset(x0 + k, y0 + 3, THREAD + 2);
            fb.pset(x0 + k, y0 + TS - 4, THREAD + 2);
        }
    }
    if matches!(tile, Tile::Pad | Tile::OutMat) {
        let pad = g.world.pad_owner(tx, ty);
        let flash = pad.is_some_and(|(s, k)| g.world.shops[s].pad_flash[k] > 0);
        let edge = if flash { SPARK + 2 } else if tile == Tile::OutMat { MAT + 2 } else { MAT };
        fb.rect_fill(x0 + 1, y0 + 1, TS - 2, TS - 2, edge);
        // Plain, so a ghost reads on it, with a dotted weave just inside its edge.
        fb.rect_fill(x0 + 2, y0 + 2, TS - 4, TS - 4, MAT);
        for y in 2..TS - 2 {
            for x in 2..TS - 2 {
                if (x == 2 || y == 2 || x == TS - 3 || y == TS - 3) && (x + y) % 2 == 0 {
                    fb.pset(x0 + x, y0 + y, MAT + 1);
                }
            }
        }
        if g.world.item_at(tx, ty).is_none() {
            if let Some(icon) = mat_hint(g, tx, ty, tile).and_then(|id| sp.icon(&id)) {
                blit_ghost(fb, icon, x0 + 4, y0 + 4);
            }
        }
    }
    if tile == Tile::Peg {
        fb.rect_fill(x0 + 2, y0 + 2, TS - 4, TS - 4, WOOD);
        fb.rect_fill(x0 + 3, y0 + 3, TS - 6, TS - 6, WOOD + 1);
        fb.rect_fill(x0 + 7, y0 + 3, 2, 2, WOOD + 3);
        if g.world.item_at(tx, ty).is_none() {
            if let Some(icon) = peg_hint(g, tx, ty).and_then(|id| sp.icon(id)) {
                blit_ghost(fb, icon, x0 + 4, y0 + 5);
            }
        }
    }
}

/// A pixel of a wall, at `(x, y)` in tile `(tx, ty)`, in its style's material: the lit top
/// edge, the shaded foot and the seams the same in all of them. `foot`: the wall ends below
/// this tile, where a plinth shows.
fn wall_px(walls: Walls, x: i32, y: i32, (tx, ty): (i32, i32), foot: bool, h: u32) -> u8 {
    if y == 0 {
        return WALL + 3;
    }
    if y == TS - 1 {
        return WALL;
    }
    match walls {
        // Boards with staggered joints.
        Walls::Planks => {
            if (x + if ty % 2 == 0 { 0 } else { 4 }) % 8 == 0 {
                WALL
            } else {
                WALL + 1 + (h % 9 == 0) as u8
            }
        }
        // Courses of dry stone, each block lit along its top, the joints staggered.
        Walls::Stone => {
            let course = (y - 1) / 5;
            let off = (course * 5 + tx * 3 + ty) % 7;
            if (y - 1) % 5 == 4 || (x + off) % 7 == 0 {
                WALL
            } else if (y - 1) % 5 == 0 {
                WALL + 2
            } else {
                WALL + 1 + (h % 11 == 0) as u8
            }
        }
        // Small bricks in running bond, pale mortar between.
        Walls::Brick => {
            let course = (y - 1) / 3;
            if (y - 1) % 3 == 2 || (x + course % 2 * 3) % 6 == 5 {
                WALL + 3
            } else if h % 7 == 0 {
                WALL
            } else {
                WALL + 1 + ((y - 1) % 3 == 0) as u8
            }
        }
        // Pale daub between dark timbers: a post in each tile, and a beam across the middle.
        Walls::Daub => {
            if (x + tx * 4) % 16 < 2 || (7..=8).contains(&y) {
                WALL
            } else if h % 13 == 0 {
                WALL + 1
            } else {
                WALL + 2 + (h % 5 == 0) as u8
            }
        }
        // Whitewash, over a dark stone plinth where the wall meets the ground.
        Walls::Plaster => {
            if foot && y >= TS - 4 {
                if y == TS - 4 || (x + tx * 3) % 6 == 0 { WALL } else { WALL + 1 }
            } else if h % 17 == 0 {
                WALL + 2
            } else {
                WALL + 3
            }
        }
    }
}

/// A pixel of a floor, in its style's material.
fn floor_px(floors: Floors, x: i32, y: i32, tx: i32, h: u32) -> u8 {
    match floors {
        // Planks: a seam every 4 rows with staggered joints.
        Floors::Planks => {
            let row = y / 4;
            if y % 4 == 3 || (x + row * 5 + tx * 3) % 16 == 0 {
                FLOOR
            } else if y % 4 == 0 {
                FLOOR + 2
            } else {
                FLOOR + 1 + (h % 13 == 0) as u8
            }
        }
        // Flagstones, staggered, each lit along its top.
        Floors::Flags => {
            let row = y / 8;
            if y % 8 == 7 || (x + row * 4 + tx * 5) % 8 == 7 {
                FLOOR
            } else if y % 8 == 0 {
                FLOOR + 2
            } else {
                FLOOR + 1 + (h % 13 == 0) as u8
            }
        }
        // Square terracotta tiles in a grid, each lit at its top and left.
        Floors::Terracotta => {
            if x % 8 == 7 || y % 8 == 7 {
                FLOOR
            } else if x % 8 == 0 || y % 8 == 0 {
                FLOOR + 2
            } else {
                FLOOR + 1 + (h % 17 == 0) as u8
            }
        }
        // Beaten earth: flat, flecked.
        Floors::Earth => match h % 29 {
            0 | 1 => FLOOR,
            2 => FLOOR + 3,
            3..=5 => FLOOR + 2,
            _ => FLOOR + 1,
        },
        // Pale tiles, offset in rows.
        Floors::Tiles => {
            let row = y / 8;
            if y % 8 == 7 || (x + row * 4) % 8 == 7 {
                FLOOR + 1
            } else if (x + row * 4) % 8 == 0 || y % 8 == 0 {
                FLOOR + 3
            } else {
                FLOOR + 2 + (h % 19 == 0) as u8
            }
        }
    }
}

fn grass(h: u32) -> u8 {
    match h % 23 {
        0 => GRASS + 3,
        1 | 2 => GRASS + 2,
        3 => GRASS,
        _ => GRASS + 1,
    }
}

/// The ground drawn on a tile, under whatever stands there: a building's floor indoors, otherwise
/// grass, or under a bin what it stands in (a path). A stand for a piece too: outdoors, on grass.
fn ground(g: &Game, tx: i32, ty: i32) -> Tile {
    // Scenery stands on planks indoors; outdoors, on gravel where the ground around is gravel, on
    // the springs' sinter where it's most of the ground around, on cobbles where all the open
    // ground around is cobbled (a well in a square), otherwise on grass.
    let around = || DIRS.iter().map(|d| {
        let (dx, dy) = d.delta();
        g.world.tile(tx + dx, ty + dy)
    });
    let gravel = || around().any(|t| t == Tile::Gravel);
    let sinter = || around().filter(|&t| matches!(t, Tile::Sinter | Tile::Boards)).count() >= 2;
    let paved = || around().any(|t| t == Tile::Paving) && around().filter(|t| t.walkable()).all(|t| t == Tile::Paving);
    let under = || if indoors(g, tx, ty) {
        Tile::Floor
    } else if gravel() {
        Tile::Gravel
    } else if sinter() {
        Tile::Sinter
    } else if paved() {
        Tile::Paving
    } else {
        Tile::Grass
    };
    match g.world.tile(tx, ty) {
        Tile::Bin => match g.world.tile(tx, ty + 1) {
            Tile::Wall | Tile::Door | Tile::Sign | Tile::Tree | Tile::Bin => under(),
            t => t,
        },
        Tile::Tree | Tile::Sign | Tile::Rack => under(),
        // A workshop's spot, pads, pegs and mat outdoors: on the stony ground, if that's where
        // it stands.
        Tile::Peg | Tile::Pad | Tile::OutMat | Tile::Spot if !indoors(g, tx, ty) && gravel() => Tile::Gravel,
        Tile::Gate => Tile::Path,
        t => t,
    }
}

/// Inside a building: a wall or door each way across its floor, however wide the room, before
/// any grass, path or water. Mats and scenery there sit on planks; outdoors, on grass.
fn indoors(g: &Game, tx: i32, ty: i32) -> bool {
    let bounded = |dx: i32, dy: i32| {
        for k in 1..SCREEN_W {
            match g.world.tile(tx + dx * k, ty + dy * k) {
                Tile::Wall | Tile::Door => return true,
                Tile::Floor | Tile::Pad | Tile::Peg | Tile::OutMat | Tile::Spot | Tile::Sign | Tile::Bin | Tile::Rack => {}
                _ => return false,
            }
        }
        false
    };
    bounded(-1, 0) && bounded(1, 0) && bounded(0, -1) && bounded(0, 1)
}

/// What belongs on an empty pad or an output mat, for what the workshop's pads are working
/// toward (`Game::aim`, the same as its card and its monsters say, so best fit never
/// surprises): the output on its mat, and on the empty pads, in order, the inputs it still
/// needs. A good the world grows or makes nearby is shown before a far-off one.
fn mat_hint(g: &Game, tx: i32, ty: i32, tile: Tile) -> Option<String> {
    let s = g.world.shops.iter().position(|s| s.pads.contains(&(tx, ty)) || s.outs.contains(&(tx, ty)))?;
    let shop = &g.world.shops[s];
    let aim = g.aim(s, &|_| true);
    if tile == Tile::OutMat {
        return Some(g.data.good(&aim.recipe).output().to_string());
    }
    let k = shop.pads.iter().filter(|&&(x, y)| g.world.item_at(x, y).is_none()).position(|&p| p == (tx, ty))?;
    ghost_good(g, aim.missing.get(k)?)
}

/// The good a ghost shows for an input: one the world grows or makes nearby before a far-off one.
fn ghost_good(g: &Game, group: &[String]) -> Option<String> {
    // What grows here first (a stone pad shows a stone, not the stone anvil a mason's bench
    // makes), then what's made here, then anything that fits.
    let grows = |id: &str| g.world.plants.iter().any(|p| p.good == id);
    let made = |id: &str| g.world.shops.iter().any(|s| s.kind.recipes().contains(&id));
    let fits: Vec<&crate::data::GoodType> = g.data.goods.iter().filter(|p| crate::data::matches(&p.type_tags, group)).collect();
    fits.iter().find(|p| grows(&p.id)).or_else(|| fits.iter().find(|p| made(&p.id))).or(fits.first()).map(|p| p.id.clone())
}

/// The built-in tool a peg stands in for.
fn peg_hint(g: &Game, tx: i32, ty: i32) -> Option<&'static str> {
    let shop = g.world.shops.iter().find(|s| s.pegs.contains(&(tx, ty)))?;
    let k = shop.pegs.iter().position(|&p| p == (tx, ty))?;
    shop.kind.builtin_tools().get(k).copied()
}

/// Every bubble on screen, oldest first, so newer ones are drawn over older ones (Henry): what
/// monsters say, and what the one you steer is facing shows (a rack, a townsperson, an
/// applicant), which is as old as the facing. On the same frame, what's said goes on top, and
/// of two said then, the steered monster's.
fn bubbles(g: &Game, v: &View) -> Vec<BubbleBox> {
    let mut all = layout_bubbles(g, v);
    let faced = g.monsters[v.c].faced.1;
    for mut b in [applicant_pitch(g, v), rack_pitch(g, v), townsfolk_talk(g, v), worker_talk(g, v)].into_iter().flatten() {
        (b.born, b.rank) = (faced, 0);
        all.push(b);
    }
    all.sort_by_key(|b| (b.born, b.rank));
    all
}

/// Facing a townsperson: a craftsman says what he'd make from what's around, just for this
/// player (Space asks him for another idea; his card says so); the trader, what he's got.
fn townsfolk_talk(g: &Game, v: &View) -> Option<BubbleBox> {
    let t = g.faced_townsfolk(v.c)?;
    let tf = &g.world.townsfolk[t];
    if !v.sees(tf.x, tf.y) {
        return None;
    }
    let says = match tf.kind {
        crate::world::Folk::Craftsman => g.craftsman_says(t, v.k),
        crate::world::Folk::Trader => g.trader_says(t),
        crate::world::Folk::Outfitter => g.outfitter_says(v.c),
        crate::world::Folk::Keeper => g.keeper_says(v.c),
    };
    talk_bubble(v, (tf.x, tf.y), says)
}

/// Facing a working townsperson: what their bench makes from what, or their own words.
fn worker_talk(g: &Game, v: &View) -> Option<BubbleBox> {
    let j = g.faced_worker(v.c)?;
    let m = &g.monsters[j];
    if !v.sees(m.x, m.y) || m.slide.is_some() {
        return None;
    }
    talk_bubble(v, (m.x, m.y), g.worker_says(j, v.k))
}

/// A speech bubble over someone at `at`, saying `says`.
fn talk_bubble(v: &View, at: (i32, i32), says: Vec<String>) -> Option<BubbleBox> {
    let mut lines = Vec::new();
    for said in says {
        lines.extend(wrap(&said, 30));
    }
    let w = lines.iter().map(|l| l.len() as i32).max().unwrap_or(0) * 6 + 6;
    let h = lines.len() as i32 * 9 + 4;
    if lines.is_empty() {
        return None;
    }
    let (px, py) = v.px(at.0, at.1);
    let cx = px + 8;
    Some(BubbleBox { x: (cx - w / 2).clamp(2, 398 - w), y: (py - h - 6).max(2), w, h, cx, above: true, lines, icon: None, mark: None, warn: false, born: 0, rank: 0 })
}

/// Facing a rack: its piece, what it does and the renown it's given at. Below the rack if
/// there's room, otherwise over whoever faces it.
fn rack_pitch(g: &Game, v: &View) -> Option<BubbleBox> {
    let r = g.faced_rack(v.c)?;
    let rack = &g.world.racks[r];
    if !v.sees(rack.x, rack.y) {
        return None;
    }
    let p = g.data.piece(rack.piece);
    let lines = vec![p.name.to_uppercase(), p.effect_text(), format!("({} RENOWN)", p.renown)];
    let w = lines.iter().map(|l| l.len() as i32).max().unwrap_or(0) * 6 + 6;
    let h = lines.len() as i32 * 9 + 4;
    let (px, py) = v.px(rack.x, rack.y);
    let cx = px + 8;
    let warn = g.communes[v.k].renown < p.renown as f64 || g.piece_refusal(v.c, p).is_some();
    Some(if py + TS + 6 + h <= VIEW_H - 2 {
        BubbleBox { x: (cx - w / 2).clamp(2, 398 - w), y: py + TS + 6, w, h, cx, above: false, lines, icon: None, mark: None, warn, born: 0, rank: 0 }
    } else {
        let m = &g.monsters[v.c];
        let my = v.px(m.x, m.y).1;
        BubbleBox { x: (cx - w / 2).clamp(2, 398 - w), y: (my - h - 6).max(2), w, h, cx, above: true, lines, icon: None, mark: None, warn, born: 0, rank: 0 }
    })
}

/// Facing an applicant: who they are, their talents and what they cost.
fn applicant_pitch(g: &Game, v: &View) -> Option<BubbleBox> {
    let k = g.faced_applicant(v.c)?;
    let a = g.applicants[k].as_ref()?;
    let (sx, sy) = g.world.stands[k];
    if !v.sees(sx, sy) {
        return None;
    }
    let talents = a.transferable.iter().map(|t| t.replace('_', " ").to_uppercase()).collect::<Vec<_>>().join(", ");
    let mut lines = vec![format!("{} THE {}", a.name, a.kind.to_uppercase()), format!("JOINS AT {} RENOWN", a.cost)];
    lines.extend(wrap(&format!("TALENTS: {talents}"), 24));
    // Its type's ability: its name, then what it does, whole.
    if let Some(ab) = g.data.monster_types.get(a.kind.as_str()).and_then(|t| t.ability.as_ref()) {
        lines.push(format!("{}:", ab.name.to_uppercase()));
        lines.extend(wrap(&ab.does.text().join(", "), 30));
    }
    let w = lines.iter().map(|l| l.len() as i32).max().unwrap_or(0) * 6 + 6;
    let h = lines.len() as i32 * 9 + 4;
    let (px, py) = v.px(sx, sy);
    let cx = px + 8;
    Some(BubbleBox { x: (cx - w / 2).clamp(2, 398 - w), y: (py + TS + 6).min(VIEW_H - h - 2), w, h, cx, above: false, lines, icon: None, mark: None, warn: g.communes[v.k].renown < a.cost as f64, born: 0, rank: 0 })
}

/// Renown rising from a drop-off, and credits flying to each contributor.
fn draw_floaters(g: &Game, fb: &mut Framebuffer, v: &View) {
    let (cx, cy) = (v.tx * TS, v.ty * TS);
    for f in g.floaters.iter().filter(|f| f.t >= f.delay) {
        let k = (f.t - f.delay) as i32;
        let n = (f.total - f.delay).max(1) as i32;
        let x = f.from.0 + (f.to.0 - f.from.0) * k / n - cx;
        let y = f.from.1 + (f.to.1 - f.from.1) * k / n - cy;
        if (0..400).contains(&x) && (0..VIEW_H).contains(&y) {
            fb.text_centered(&f.text, x, y, f.color, Some(OUTLINE), 1);
        }
    }
}

/// A small count in a tile's top-right corner.
fn badge(fb: &mut Framebuffer, x: i32, y: i32, n: usize) {
    let text = n.to_string();
    let w = Framebuffer::text_width(&text, 1) + 3;
    fb.rect_fill(x + TS - w, y - 2, w, 9, OUTLINE);
    fb.text(&text, x + TS - w + 2, y - 1, TEXT, None, 1);
}

/// A sprite brightened or dimmed by quality in half steps: odd levels shift every other
/// pixel (ordered dither), even levels shift them all. The dark hull never changes.
/// A good's colors from its varieties: a dyed good's cotton-white pixels take its dye's color,
/// and its wood takes the color of the wood it's made of (a kit's, of its material, as its
/// workshop will).
fn tint(data: &crate::data::Data, item: &crate::econ::ItemData) -> Option<Tint> {
    let color = |tag: &str| data.variety(tag).and_then(|v| v.color.as_deref()).and_then(ramp_named);
    let dye = item.carried_tags.iter().find_map(|t| data.variety(t).filter(|v| v.is("dye")).and_then(|v| color(&v.tag)));
    if let Some(to) = dye {
        return Some(Tint { from: ramp_named("cotton")?, to });
    }
    let of = crate::frontier::named_by(&item.good).unwrap_or("wood");
    wood_tint(color(data.variety_in(&item.carried_tags, of)?))
}

/// Wood drawn in another wood's ramp, unless it's the wood the art is drawn in already.
fn wood_tint(to: Option<(u8, u8)>) -> Option<Tint> {
    let from = ramp_named("wood")?;
    to.filter(|&to| to != from).map(|to| Tint { from, to })
}

/// A sprite, maybe flipped, recolored by `tint`.
fn blit_tinted(fb: &mut Framebuffer, s: &Sprite, x: i32, y: i32, flip: bool, tint: Option<Tint>) {
    let Some(t) = tint else { return fb.blit(s, x, y, flip) };
    for sy in 0..s.h {
        for sx in 0..s.w {
            let c = s.px[(sy * s.w + if flip { s.w - 1 - sx } else { sx }) as usize];
            if c != 0 {
                fb.pset(x + sx, y + sy, t.apply(c));
            }
        }
    }
}

fn blit_quality(fb: &mut Framebuffer, s: &Sprite, x: i32, y: i32, level: i32, tint: Option<Tint>) {
    for sy in 0..s.h {
        for sx in 0..s.w {
            let mut c = s.px[(sy * s.w + sx) as usize];
            if c == 0 {
                continue;
            }
            if let Some(t) = tint {
                c = t.apply(c);
            }
            let half = level % 2 != 0 && (x + sx + y + sy) % 2 == 0;
            let k = level / 2 + if half { level.signum() } else { 0 };
            fb.pset(x + sx, y + sy, if c > 1 { shift(c, k) } else { c });
        }
    }
}

/// "+SPINNING" rises over a monster that just learned.
fn draw_popups(g: &Game, fb: &mut Framebuffer, v: &View) {
    for m in g.monsters.iter().filter(|m| v.sees(m.x, m.y)) {
        let Some((recipe, _, ttl)) = &m.learned else { continue };
        let Some(r) = g.data.try_good(recipe) else { continue };
        let text = format!("+{}", r.primary_applied_skill.replace('_', " ").to_uppercase());
        let (ox, oy) = Slide::offset(m.slide);
        let (mx, my) = v.px(m.x, m.y);
        let rise = (45 - *ttl as i32).max(0) / 3;
        fb.text_centered(&text, mx + ox + 8, my + oy - 10 - rise, TEXT_GOOD, Some(OUTLINE), 1);
    }
}

/// The hold-E card, with corner brackets on what it describes. It shows one page at a time;
/// a footer says Tab turns to the other.
fn draw_card(g: &Game, sp: &Sprites, fb: &mut Framebuffer, v: &View) {
    let card = inspect::card(g, v.k);
    let page = g.communes[v.k].card_page % card.pages.len();
    let lines = &card.pages[page];
    let paged = card.pages.len() > 1;
    let bar_w = if lines.iter().any(|l| l.bar.is_some()) { 46 } else { 0 };
    // Each line becomes one or more rows; only a line's first row carries its icons and bar.
    let layout = |chars: usize| {
        let mut rows: Vec<(&Line, String, bool)> = Vec::new();
        for l in lines {
            if l.text.is_empty() {
                rows.push((l, String::new(), true));
                continue;
            }
            let chars = if l.icon.is_some() { chars.saturating_sub(2) } else { chars };
            for (k, t) in wrap(&l.text, chars.max(8)).into_iter().enumerate() {
                rows.push((l, t, k == 0));
            }
        }
        let row_w = |(l, t, first): &(&Line, String, bool)| {
            let icon = if l.icon.is_some() { ICON_STEP } else { 0 };
            let then = match (&l.then, first) {
                (Some((_, s)), true) => 6 + ICON_STEP + s.len() as i32 * 6,
                _ => 0,
            };
            let meter = if l.meter.is_some() { ICON_STEP + METER_W } else { 0 };
            icon + t.len() as i32 * 6 + then + meter
        };
        let widest = rows.iter().map(row_w).max().unwrap_or(0).max(card.title.len() as i32 * 6).max(if paged { 90 } else { 0 });
        (rows, widest + 12 + bar_w)
    };
    let at = v.px(card.at.0, card.at.1);
    let (mut rows, mut w) = layout(38);
    let (right, room) = inspect::side(at, w);
    if w > room {
        (rows, w) = layout((((room - 12 - bar_w) / 6).max(14)) as usize);
    }
    let footer = if paged { 13 } else { 0 };
    let h = 24 + rows.len() as i32 * 10 + footer;
    if v.sees(card.at.0, card.at.1) {
        let (tx, ty) = at;
        for (cx, cy, dx, dy) in [(tx, ty, 1, 1), (tx + TS - 1, ty, -1, 1), (tx, ty + TS - 1, 1, -1), (tx + TS - 1, ty + TS - 1, -1, -1)] {
            fb.rect_fill(cx.min(cx + dx * 3), cy, 4, 1, SPARK + 2);
            fb.rect_fill(cx, cy.min(cy + dy * 3), 1, 4, SPARK + 2);
        }
    }
    let (x, y) = inspect::place(at, w, h, VIEW_H, right);
    fb.rect_bevel(x, y, w, h, HUD_FILL, HUD_HI, HUD_SHADOW);
    fb.text(&card.title, x + 6, y + 6, SPARK + 2, Some(HUD_SHADOW), 1);
    fb.rect_fill(x + 4, y + 16, w - 8, 1, HUD_SHADOW);
    for (k, (l, t, first)) in rows.iter().enumerate() {
        let ly = y + 20 + k as i32 * 10;
        let mut tx = x + 6;
        if let (Some(icon), true) = (&l.icon, *first) {
            draw_icon(fb, sp, icon, tx, ly);
        }
        if l.icon.is_some() {
            tx += ICON_STEP;
        }
        if let Some(m) = l.meter {
            draw_meter(fb, tx + ICON_STEP, ly, m);
            continue;
        }
        fb.text(t, tx, ly, l.color, Some(HUD_SHADOW), 1);
        if let (Some((icon, s)), true) = (&l.then, *first) {
            let ax = tx + t.len() as i32 * 6 + 6;
            draw_icon(fb, sp, icon, ax, ly);
            fb.text(s, ax + ICON_STEP, ly, TEXT, Some(HUD_SHADOW), 1);
        }
        if let (Some(b), true) = (l.bar, *first) {
            let bx = x + w - 46;
            fb.rect_fill(bx, ly, 40, 6, OUTLINE);
            fb.rect_fill(bx + 1, ly + 1, ((b.clamp(0.0, 1.0) * 38.0) as i32).max(1), 4, SPARK + 1);
        }
    }
    if paged {
        let fy = y + h - 13;
        fb.rect_fill(x + 4, fy - 2, w - 8, 1, HUD_SHADOW);
        fb.text(if page == 0 { "TAB: MORE" } else { "TAB: BACK" }, x + 6, fy + 1, TEXT_DIM, Some(HUD_SHADOW), 1);
        let n = card.pages.len() as i32;
        for p in 0..n {
            let dx = x + w - 8 - (n - p) * 7;
            fb.rect_fill(dx, fy + 2, 5, 5, OUTLINE);
            fb.rect_fill(dx + 1, fy + 3, 3, 3, if p as usize == page { TEXT } else { HUD_MID });
        }
    }
}

/// Width an icon takes in a card line, with its gap.
const ICON_STEP: i32 = 10;
/// The quality meter spans 0 to 2 across this many pixels.
const METER_W: i32 = 100;

/// A quality meter: the usual range as a band, the expected value as a bright tick, and small
/// marks at standard (1.0), exceptional and masterwork.
fn draw_meter(fb: &mut Framebuffer, x: i32, y: i32, m: Meter) {
    let at = |q: f64| x + ((q / 2.0).clamp(0.0, 1.0) * (METER_W - 1) as f64).round() as i32;
    fb.rect_fill(x, y + 2, METER_W, 4, OUTLINE);
    let (a, b) = (at(m.lo), at(m.hi));
    fb.rect_fill(a, y + 3, (b - a).max(1), 2, HUD_HI);
    for (q, c) in [(1.0, TEXT_DIM), (EXCEPTIONAL, SPARK + 1), (MASTERWORK, SPARK + 2)] {
        fb.rect_fill(at(q), y + 1, 1, 6, c);
    }
    let mx = at(m.mid);
    fb.rect_fill(mx - 1, y - 1, 3, 10, OUTLINE);
    fb.rect_fill(mx, y, 1, 8, TEXT);
}

/// 7x7 card icons: `a` is the main color, `b` the second.
fn icon_art(icon: &Icon) -> ([&'static str; 7], u8, u8) {
    match icon {
        Icon::Quality => (["...a...", "...a...", "aaaaaaa", ".aaaaa.", "..aaa..", ".aa.aa.", "a.....a"], SPARK + 2, SPARK),
        Icon::Time => (["aaaaaaa", ".abbba.", "..aba..", "...a...", "..a.a..", ".abbba.", "aaaaaaa"], WOOD + 3, SPARK + 1),
        Icon::Warn => (["...a...", "..aaa..", "..aba..", ".aabaa.", ".aaaaa.", "aaabaaa", "aaaaaaa"], TEXT_WARN, OUTLINE),
        Icon::Learn => (["...a...", "..aaa..", ".aaaaa.", "aaaaaaa", "..aaa..", "..aaa..", "..aaa.."], TEXT_GOOD, TEXT_GOOD),
        Icon::Forget => (["..aaa..", "..aaa..", "..aaa..", "aaaaaaa", ".aaaaa.", "..aaa..", "...a..."], TEXT_DIM, TEXT_DIM),
        Icon::Renown => (["...a...", "..aaa..", ".aabaa.", "aabbbaa", ".aabaa.", "..aaa..", "...a..."], SPARK + 1, TEXT),
        Icon::Weight => (["..aaa..", ".a...a.", ".aaaaa.", "aaaaaaa", "aaaaaaa", "aaaaaaa", ".aaaaa."], HUD_HI, HUD_HI),
        Icon::Wear => (["aaaaa..", "aaaaa..", "..b....", "..b....", "..b....", "..b....", "..b...."], HUD_HI, WOOD + 2),
        Icon::Good(_) => (["aaaaaaa", "abbbbba", "aaaaaaa", "abbbbba", "abbbbba", "abbbbba", "aaaaaaa"], WOOD + 3, WOOD + 1),
    }
}

fn draw_icon(fb: &mut Framebuffer, sp: &Sprites, icon: &Icon, x: i32, y: i32) {
    if let Icon::Good(id) = icon {
        if let Some(s) = sp.icon(id) {
            fb.blit(s, x, y - 1, false);
            return;
        }
    }
    let (art, a, b) = icon_art(icon);
    for (dy, row) in art.iter().enumerate() {
        for (dx, ch) in row.chars().enumerate() {
            if ch != '.' {
                fb.pset(x + dx as i32, y + dy as i32, if ch == 'a' { a } else { b });
            }
        }
    }
}

/// A sprite in the mat's two light tones, its own lights and darks kept: a "this goes here"
/// hint that reads clearly but is plainly not the good itself (no alpha, no outline).
fn blit_ghost(fb: &mut Framebuffer, s: &Sprite, x: i32, y: i32) {
    for sy in 0..s.h {
        for sx in 0..s.w {
            let c = s.px[(sy * s.w + sx) as usize];
            if c > 1 {
                fb.pset(x + sx, y + sy, if crate::palette::is_light(c) { MAT + 2 } else { MAT + 1 });
            }
        }
    }
}

fn draw_building_labels(g: &Game, fb: &mut Framebuffer, v: &View) {
    let mut named: Vec<((i32, i32), &str)> = Vec::new();
    // Town workshops' buildings; frontier ones are open-air, with no walls to put a name on.
    // One out in the open (a charcoal mound, the smith's own forge) has no wall to put a name on.
    for s in g.world.shops.iter().filter(|s| s.kind.is_building() && s.frontier.is_none() && indoors(g, s.spot.0, s.spot.1)) {
        // Up from the work spot to the building's top wall; a door in that wall counts too.
        let (sx, mut wy) = s.spot;
        while wy > 0 && !matches!(g.world.tile(sx, wy), Tile::Wall | Tile::Door) {
            wy -= 1;
        }
        named.push(((sx, wy), s.kind.label()));
    }
    named.extend(g.world.labels.iter().copied());
    for ((sx, wy), label) in named.into_iter().filter(|((x, y), _)| v.sees(*x, *y)) {
        let (mut l, mut r) = (sx, sx);
        while matches!(g.world.tile(l - 1, wy), Tile::Wall | Tile::Door) {
            l -= 1;
        }
        while g.world.tile(r + 1, wy) == Tile::Wall || g.world.tile(r + 1, wy) == Tile::Door {
            r += 1;
        }
        // Centered on the wall, unless that would cover a door or something hung on the wall
        // (an awning, a woven hanging): then on the widest stretch of wall clear of them, if
        // the name fits there.
        let busy = |x: i32| g.world.tile(x, wy) == Tile::Door || g.world.prop_at(x, wy).is_some();
        let half = Framebuffer::text_width(label, 1) / 2;
        let covers = |(a, b): (i32, i32)| {
            let mid = (a * TS + (b + 1) * TS) / 2;
            ((mid - half).div_euclid(TS)..=(mid + half - 1).div_euclid(TS)).any(busy)
        };
        let mut span = (l, r);
        if covers(span) {
            let mut runs: Vec<(i32, i32)> = Vec::new();
            for x in (l..=r).filter(|&x| !busy(x)) {
                match runs.last_mut() {
                    Some(run) if run.1 == x - 1 => run.1 = x,
                    _ => runs.push((x, x)),
                }
            }
            let fits = |&(a, b): &(i32, i32)| (b - a + 1) * TS >= 2 * half + 4;
            span = runs.into_iter().filter(fits).max_by_key(|&(a, b)| (b - a, -(a - l))).unwrap_or(span);
        }
        let (x, y) = v.px(span.0, wy);
        let w = (span.1 - span.0 + 1) * TS;
        let (ink, shadow) = v.style.label();
        fb.text_centered(label, x + w / 2, y + 5, ink, Some(shadow), 1);
    }
}

/// Trees that are not for gathering: conifers and broadleaves, picked per tile.
fn draw_trees(g: &Game, sp: &Sprites, fb: &mut Framebuffer, v: &View) {
    for ty in v.ty..v.ty + SCREEN_H {
        for tx in v.tx..v.tx + SCREEN_W {
            if g.world.tile(tx, ty) == Tile::Tree {
                let (x, y) = v.px(tx, ty);
                fb.blit(&sp.trees[(hash(tx, ty) % 2) as usize], x, y, hash(ty, tx) % 2 == 0);
            }
        }
    }
}

/// Project boards with their progress, and signposts with where they point.
fn draw_boards_and_signs(g: &Game, sp: &Sprites, fb: &mut Framebuffer, v: &View) {
    for b in g.world.boards.iter().filter(|b| v.sees(b.x, b.y)) {
        let (x, y) = v.px(b.x, b.y);
        if b.project == Project::Bell {
            // The temple's frame, with the newest bell hanging from its beam.
            fb.blit(&sp.bell_frame, x, y, false);
            if !g.world.bells.is_empty() {
                fb.rect_fill(x + 7, y + 3, 2, 1, WOOD);
                fb.blit(&sp.bell_hung, x + 3, y + 4, false);
            }
            continue;
        }
        let y = y - (b.bounce > 1) as i32;
        fb.blit(&sp.board, x, y, false);
        let (text, col) = match (b.done(), b.project) {
            (true, Project::Gate) => ("OPEN".to_string(), TEXT_GOOD),
            (true, _) => ("DONE".to_string(), TEXT_GOOD),
            _ => (format!("{}/{}", b.have, b.need), TEXT),
        };
        fb.text_centered(&text, x + 9, y - 8, col, Some(OUTLINE), 1);
    }
    // A working townsperson's supplies: a low pallet, so the woodpile reads as one when empty.
    for s in g.world.supplies.iter().filter(|s| v.sees(s.at.0, s.at.1)) {
        let (x, y) = v.px(s.at.0, s.at.1);
        fb.rect_fill(x + 1, y + 12, 14, 2, WOOD + 1);
        for k in [2, 7, 12] {
            fb.rect_fill(x + k, y + 14, 2, 1, WOOD);
        }
    }
    for &((px, py), prop) in g.world.props.iter().filter(|((x, y), _)| v.sees(*x, *y)) {
        draw_prop(g, sp, fb, v, (px, py), prop);
    }
    for &(px, py) in g.world.plaques.iter().filter(|&&(x, y)| v.sees(x, y)) {
        let (x, y) = v.px(px, py);
        fb.blit(&sp.plaque, x + 2, y + 4, false);
    }
    for s in g.world.signs.iter().filter(|s| v.sees(s.x, s.y)) {
        let (x, y) = v.px(s.x, s.y);
        fb.blit(&sp.signpost, x + 1, y, false);
        let w = Framebuffer::text_width(s.text, 1);
        let lx = (x + 8 - w / 2).clamp(6, 394 - w);
        fb.text(s.text, lx, y - 8, TEXT, Some(OUTLINE), 1);
        // A small arrow beside the name, pointing the way.
        if s.dir == Dir::Down || s.dir == Dir::Up {
            for (k, h) in [5, 3, 1].iter().enumerate() {
                let row = if s.dir == Dir::Down { k as i32 } else { 2 - k as i32 };
                fb.rect_fill(lx + w + 4 - h / 2, y - 8 + 1 + row, *h, 1, SPARK + 2);
            }
            continue;
        }
        let (ax, dir) = if s.dir == Dir::Left { (lx - 5, -1) } else { (lx + w + 2, 1) };
        for (k, h) in [5, 3, 1].iter().enumerate() {
            let col = ax + if dir > 0 { k as i32 } else { 2 - k as i32 };
            fb.rect_fill(col, y - 8 + 3 - h / 2, 1, *h, SPARK + 2);
        }
    }
}

/// A tile of cliff face, by which of its neighbors (up, down, left, right) are cliff too: rounded
/// columns of rock in the province's stone (its walls' ramp: the hills' cool grey), each lit on
/// its left and shaded on its right, broken across here and there, with dark clefts between; a
/// lip of grass where the high ground above ends, and shadow and scree at its foot.
fn draw_cliff(fb: &mut Framebuffer, x: i32, y: i32, (px, py): (i32, i32), [u, d, l, r]: [bool; 4]) {
    fb.rect_fill(x, y, TS, TS, WALL);
    let h = hash(px, py);
    // Three columns, their widths and breaks set by the tile.
    let a = 4 + (h % 3) as i32;
    let b = a + 5 + (h >> 2) as i32 % 2;
    for (k, (x0, x1)) in [(0, a), (a, b), (b, TS)].into_iter().enumerate() {
        let w = x1 - x0 - 1;
        if w < 2 {
            continue;
        }
        let brk = 4 + (h >> (5 + k * 3)) as i32 % 9;
        for (y0, y1) in [(0, brk), (brk, TS)] {
            let ht = y1 - y0 - 1;
            if ht < 2 {
                continue;
            }
            for cx in 0..w {
                // Lit down its left, shaded down its right: a rounded column.
                let c = match (cx * 4) / w {
                    0 => WALL + 3,
                    1 => WALL + 2,
                    2 => WALL + 2,
                    _ => WALL + 1,
                };
                fb.rect_fill(x + x0 + 1 + cx, y + y0 + 1, 1, ht, c);
            }
            // A rounded top to each piece: its corners cut back into the cleft.
            fb.rect_fill(x + x0 + 1, y + y0 + 1, 1, 1, WALL);
            fb.rect_fill(x + x0 + w, y + y0 + 1, 1, 1, WALL);
            fb.rect_fill(x + x0 + 2, y + y0 + 1, (w - 2).max(0), 1, WALL + 3);
        }
    }
    // Now and then a boulder bulging out of the face, lit on its upper left, or a deep cleft
    // down the whole tile, so the face doesn't read as courses.
    if h % 4 == 0 {
        let (cx, cy, rr) = (4 + (h >> 7) as i32 % 8, 5 + (h >> 11) as i32 % 6, 4 + (h >> 13) as i32 % 2);
        for yy in -rr..=rr {
            for xx in -rr..=rr {
                let d2 = xx * xx + yy * yy;
                if d2 > rr * rr + 1 {
                    continue;
                }
                let c = if d2 > (rr - 1) * (rr - 1) + 1 && (xx > 0 || yy > 0) {
                    WALL
                } else if xx + yy < -rr / 2 {
                    WALL + 3
                } else if xx + yy < rr / 2 {
                    WALL + 2
                } else {
                    WALL + 1
                };
                let (qx, qy) = (cx + xx, cy + yy);
                if (0..TS).contains(&qx) && (0..TS).contains(&qy) {
                    fb.pset(x + qx, y + qy, c);
                }
            }
        }
    } else if h % 4 == 1 {
        let cx = [a, b][(h >> 9) as usize % 2];
        fb.rect_fill(x + cx - 1, y, 2, TS, OUTLINE);
        fb.rect_fill(x + cx + 1, y, 1, TS, WALL);
    }
    // A dark pit or two, and moss in a cleft.
    for k in 0..2 {
        let (mx, my) = ((h >> (18 + k * 5)) as i32 % 13 + 1, (h >> (21 + k * 3)) as i32 % 11 + 3);
        fb.rect_fill(x + mx, y + my, 1, 2, WALL);
    }
    if (h >> 28) % 3 == 0 {
        let mx = (h >> 24) as i32 % 12 + 2;
        fb.rect_fill(x + mx, y + 7, 2, 1, HEDGE + 2);
        fb.rect_fill(x + mx + 1, y + 8, 1, 1, HEDGE + 1);
    }
    if !u {
        // The lip: the high ground's grass, ragged where it hangs over the edge.
        for gx in 0..TS {
            let hang = 1 + (hash(px * TS + gx, py) % 3) as i32;
            fb.rect_fill(x + gx, y, 1, hang, if gx % 3 == 0 { GRASS + 3 } else { GRASS + 2 });
            fb.rect_fill(x + gx, y + hang, 1, 1, OUTLINE);
        }
    }
    if !d {
        // The foot: deeper in shadow (dithered), with fallen scree.
        for yy in 9..TS {
            for xx in 0..TS {
                if (xx + yy) % 2 == 0 || yy >= 13 {
                    fb.rect_fill(x + xx, y + yy, 1, 1, WALL);
                }
            }
        }
        for k in 0..4 {
            let sx = (h >> (k * 4 + 2)) as i32 % 13;
            fb.rect_fill(x + sx, y + 13, 3, 2, WALL + 1);
            fb.rect_fill(x + sx, y + 13, 2, 1, WALL + 2);
        }
        fb.rect_fill(x, y + 15, TS, 1, OUTLINE);
    }
    if !l {
        fb.rect_fill(x, y, 1, TS, OUTLINE);
    }
    if !r {
        fb.rect_fill(x + TS - 1, y, 1, TS, OUTLINE);
    }
}

/// A vine on its cliff: a twisting stem in a mass of leaves, a woven cradle hanging off it here
/// and there, a leafy crown where it tops the cliff and roots where it stands.
fn draw_lift_vine(fb: &mut Framebuffer, x: i32, y: i32, (px, py): (i32, i32), up: bool, down: bool) {
    // Leaves either side, in clumps, darker toward the rock.
    let h = hash(px, py);
    for (k, &(lx, ly)) in [(1, 1), (10, 3), (2, 7), (11, 9), (0, 12), (9, 13)].iter().enumerate() {
        let (lx, ly) = (lx + (h >> (k * 2)) as i32 % 2, ly);
        fb.rect_fill(x + lx, y + ly, 6, 4, OUTLINE);
        fb.rect_fill(x + lx + 1, y + ly + 1, 4, 2, LEAF);
        fb.rect_fill(x + lx + 1, y + ly + 1, 2, 1, LEAF + 1);
        fb.pset(x + lx + 1, y + ly + 1, LEAF + 2);
    }
    // The stem, twisting as it goes.
    for yy in 0..TS {
        let sx = 6 + [0, 1, 1, 0, -1, -1, 0, 1][((yy + py * TS) / 2 % 8) as usize];
        fb.rect_fill(x + sx - 1, y + yy, 4, 1, OUTLINE);
        fb.rect_fill(x + sx, y + yy, 2, 1, LEAF);
        fb.pset(x + sx, y + yy, LEAF + 1);
    }
    // A cradle, on every other tile.
    if py % 2 == 0 {
        fb.rect_fill(x + 9, y + 1, 1, 4, LEAF);
        fb.rect_fill(x + 8, y + 5, 7, 6, OUTLINE);
        fb.rect_fill(x + 9, y + 5, 5, 5, MAT + 1);
        for wx in [9, 11, 13] {
            fb.rect_fill(x + wx, y + 6, 1, 4, MAT);
        }
        fb.rect_fill(x + 9, y + 5, 5, 1, MAT + 2);
    }
    if !up {
        // Its crown, spilling over the lip.
        fb.rect_fill(x - 1, y - 5, 18, 8, OUTLINE);
        fb.rect_fill(x, y - 4, 16, 6, LEAF);
        fb.rect_fill(x + 1, y - 4, 8, 2, LEAF + 1);
        fb.rect_fill(x + 10, y - 3, 4, 2, LEAF + 1);
        fb.pset(x + 3, y - 4, LEAF + 2);
        fb.pset(x + 12, y - 3, LEAF + 2);
    }
    if !down {
        // Its roots, spread on the ground at its foot.
        for (rx, rw) in [(1, 4), (11, 4), (5, 6)] {
            fb.rect_fill(x + rx, y + 14, rw, 2, WOOD + 1);
            fb.pset(x + rx, y + 14, WOOD + 2);
        }
    }
}

/// Where each vine takes goods on, plain to see: an empty woven cradle on the ground at its
/// foot, a tendril reaching to it from the vine, so you know where to set a good to send it.
fn draw_vine_feet(g: &Game, fb: &mut Framebuffer, v: &View) {
    for l in g.world.lifts.iter().filter(|l| v.sees(l.foot.0, l.foot.1)) {
        let (x, y) = v.px(l.foot.0, l.foot.1);
        // The tendril, from the cradle's edge to the vine's side of the tile.
        let (rx, ry, rw, rh) = match l.dir {
            Dir::Up => (7, 0, 2, 8),
            Dir::Down => (7, 14, 2, 2),
            Dir::Left => (0, 10, 3, 2),
            Dir::Right => (13, 10, 3, 2),
        };
        fb.rect_fill(x + rx - 1, y + ry, rw + 2, rh, OUTLINE);
        fb.rect_fill(x + rx, y + ry, rw, rh, LEAF);
        // The cradle, open and waiting.
        fb.rect_fill(x + 2, y + 7, 12, 8, OUTLINE);
        fb.rect_fill(x + 3, y + 8, 10, 6, MAT + 1);
        fb.rect_fill(x + 4, y + 9, 8, 3, MAT);
        for wx in (4..12).step_by(2) {
            fb.pset(x + wx, y + 13, MAT + 2);
        }
        fb.rect_fill(x + 3, y + 8, 10, 1, MAT + 2);
    }
}

/// A vine running across a cliff, sideways: its stem along the rock, leaves above and below,
/// and a cradle hanging from it on every other tile.
fn draw_vine_across(fb: &mut Framebuffer, x: i32, y: i32, (px, py): (i32, i32)) {
    let h = hash(px, py);
    for (k, &(lx, ly)) in [(0, 2), (6, 1), (11, 3), (2, 10), (9, 11)].iter().enumerate() {
        let lx = lx + (h >> (k * 2)) as i32 % 2;
        fb.rect_fill(x + lx, y + ly, 6, 4, OUTLINE);
        fb.rect_fill(x + lx + 1, y + ly + 1, 4, 2, LEAF);
        fb.rect_fill(x + lx + 1, y + ly + 1, 2, 1, LEAF + 1);
    }
    for xx in 0..TS {
        let sy = 7 + [0, 1, 1, 0, -1, -1, 0, 1][((xx + px * TS) / 2 % 8) as usize];
        fb.rect_fill(x + xx, y + sy - 1, 1, 4, OUTLINE);
        fb.rect_fill(x + xx, y + sy, 1, 2, LEAF);
        fb.pset(x + xx, y + sy, LEAF + 1);
    }
    if px % 2 == 0 {
        fb.rect_fill(x + 6, y + 9, 1, 2, LEAF);
        fb.rect_fill(x + 3, y + 10, 7, 6, OUTLINE);
        fb.rect_fill(x + 4, y + 10, 5, 5, MAT + 1);
        fb.rect_fill(x + 4, y + 10, 5, 1, MAT + 2);
    }
}

/// A piece of scenery: whole on its tiles, or joined to its neighbors of the same kind.
fn draw_prop(g: &Game, sp: &Sprites, fb: &mut Framebuffer, v: &View, (px, py): (i32, i32), prop: Prop) {
    let k = prop.kind();
    let (x, y) = v.px(px, py);
    let joins = |id: &str, dx: i32, dy: i32| g.world.prop_at(px + dx, py + dy).is_some_and(|(_, p)| p.0 == id || (id == "rails" && p.0 == "mine_cart"));
    match k.id {
        "cliff" | "lift_vine" | "pit_wall" => {
            let cliff = |dx: i32, dy: i32| ["cliff", "lift_vine", "pit_wall"].iter().any(|id| joins(id, dx, dy));
            draw_cliff(fb, x, y, (px, py), [cliff(0, -1), cliff(0, 1), cliff(-1, 0), cliff(1, 0)]);
            if k.id == "lift_vine" && (joins("lift_vine", -1, 0) || joins("lift_vine", 1, 0)) {
                draw_vine_across(fb, x, y, (px, py));
            } else if k.id == "lift_vine" {
                draw_lift_vine(fb, x, y, (px, py), joins("lift_vine", 0, -1), joins("lift_vine", 0, 1));
            }
            // A pit's wall is shored with timber here and there: a post, and a cap across it.
            if k.id == "pit_wall" && (px + py * 2) % 3 == 0 {
                fb.rect_fill(x + 6, y, 5, TS, OUTLINE);
                fb.rect_fill(x + 7, y, 3, TS, WOOD + 1);
                fb.rect_fill(x + 7, y, 1, TS, WOOD + 2);
                if !cliff(0, -1) {
                    fb.rect_fill(x + 3, y + 1, 11, 4, OUTLINE);
                    fb.rect_fill(x + 4, y + 2, 9, 2, WOOD + 2);
                }
            }
        }
        "fence" => {
            // A post, with rails to the fence on either side.
            fb.rect_fill(x + 6, y + 3, 4, 12, WOOD + 2);
            fb.rect_fill(x + 6, y + 3, 4, 1, WOOD + 3);
            for (dx, rx) in [(-1, 0), (1, 10)] {
                if joins("fence", dx, 0) {
                    fb.rect_fill(x + rx, y + 6, 6, 2, WOOD + 1);
                    fb.rect_fill(x + rx, y + 11, 6, 2, WOOD + 1);
                }
            }
            // Up and down, a rail joins the posts.
            if joins("fence", 0, -1) {
                fb.rect_fill(x + 7, y, 2, 3, WOOD + 1);
            }
            // A rail down to the next post south, which the next row's post overlaps.
            if joins("fence", 0, 1) {
                fb.rect_fill(x + 7, y + 15, 2, 4, WOOD + 1);
            }
        }
        "stone_wall" | "parapet" | "yard_wall" => {
            // A low wall of dressed stone: a block here, and courses out to each neighbor.
            let (l, r, u, d) = (joins(k.id, -1, 0), joins(k.id, 1, 0), joins(k.id, 0, -1), joins(k.id, 0, 1));
            if u || d {
                let (a, b) = (if u { 0 } else { 4 }, if d { 16 } else { 13 });
                fb.rect_fill(x + 4, y + a, 8, b - a, OUTLINE);
                fb.rect_fill(x + 5, y + a, 6, b - a, HUD_HI);
                fb.rect_fill(x + 9, y + a, 2, b - a, HUD_MID);
                for jy in (a..b).filter(|jy| (jy + py * TS) % 6 == 0) {
                    fb.rect_fill(x + 5, y + jy, 6, 1, HUD_FILL);
                }
            }
            if l || r || !(u || d) {
                let (a, b) = (if l { 0 } else { 3 }, if r { 16 } else { 13 });
                fb.rect_fill(x + a, y + 4, b - a, 9, OUTLINE);
                fb.rect_fill(x + a, y + 5, b - a, 3, HUD_HI);
                fb.rect_fill(x + a, y + 8, b - a, 4, HUD_MID);
                for jx in (a..b).filter(|jx| (jx + px * TS) % 6 == 0) {
                    fb.rect_fill(x + jx, y + 8, 1, 4, HUD_FILL);
                }
            }
        }
        "rails" | "mine_cart" => {
            // Two iron rails on wooden sleepers, with a stop block where they end.
            let (l, r) = (joins("rails", -1, 0), joins("rails", 1, 0));
            let (a, b) = (if l { 0 } else { 2 }, if r { 16 } else { 14 });
            for sx in (a..b).filter(|sx| sx % 4 == 1) {
                fb.rect_fill(x + sx, y + 6, 2, 8, WOOD + 1);
            }
            for ry in [7, 12] {
                fb.rect_fill(x + a, y + ry, b - a, 1, HUD_HI);
                fb.rect_fill(x + a, y + ry + 1, b - a, 1, HUD_FILL);
            }
            for (end, bx) in [(l, 0), (r, 13)] {
                if !end {
                    fb.rect_fill(x + bx, y + 6, 3, 9, OUTLINE);
                    fb.rect_fill(x + bx + 1, y + 7, 1, 7, WOOD + 2);
                }
            }
            if k.id == "mine_cart" {
                if let Some(s) = sp.prop(k.id, 0) {
                    fb.blit(s, x + (TS - s.w) / 2, y + TS - 2 - s.h, false);
                }
            }
        }
        "choice_board" => {
            fb.rect_fill(x + 7, y + 9, 2, 7, WOOD + 1);
            fb.rect_fill(x + 1, y - 1, 14, 12, OUTLINE);
            fb.rect_fill(x + 2, y, 12, 10, WOOD + 2);
            fb.rect_fill(x + 2, y, 12, 1, WOOD + 3);
            let chosen = g.world.choice_at(px, py).and_then(|s| g.world.shops[s].chosen.as_deref());
            match chosen.and_then(|r| sp.icon(g.data.good(r).output())) {
                Some(icon) => fb.blit(icon, x + 4, y + 1, false),
                None => fb.text_centered("?", x + 8, y + 2, TEXT, Some(OUTLINE), 1),
            }
        }
        "hemp_rack" => {
            // An A-frame at each end, a pole across, and bundles of hemp hung over it to dry.
            let (l, r) = (joins(k.id, -1, 0), joins(k.id, 1, 0));
            for (end, ex) in [(l, 3), (r, 12)] {
                if !end {
                    for yy in 1..TS {
                        let spread = (yy - 1) / 4;
                        for lx in [ex - spread, ex + spread] {
                            fb.rect_fill(x + lx - 1, y + yy, 3, 1, OUTLINE);
                        }
                    }
                    for yy in 1..TS {
                        let spread = (yy - 1) / 4;
                        for lx in [ex - spread, ex + spread] {
                            fb.pset(x + lx, y + yy, WOOD + 2);
                        }
                    }
                }
            }
            let (a, b) = (if l { 0 } else { 2 }, if r { 16 } else { 14 });
            fb.rect_fill(x + a, y + 1, b - a, 3, OUTLINE);
            fb.rect_fill(x + a, y + 2, b - a, 1, WOOD + 3);
            for bx in (a + 1..b - 2).step_by(3) {
                let long = 9 + (hash(px * TS + bx, py) % 3) as i32;
                fb.rect_fill(x + bx - 1, y + 3, 4, long + 1, OUTLINE);
                fb.rect_fill(x + bx, y + 3, 2, long, MAT + 2);
                fb.pset(x + bx + 1, y + 3 + long / 2, MAT + 1);
                fb.rect_fill(x + bx, y + 3, 2, 2, MAT);
                fb.pset(x + bx, y + 2 + long, LEAF + 1);
            }
        }
        "washing_line" | "drying_line" => {
            // A post at each end, the line between, and laundry pegged along it: in the dyers'
            // town, lengths of cloth fresh from the vats.
            let (l, r) = (joins(k.id, -1, 0), joins(k.id, 1, 0));
            for (end, ex) in [(l, 2), (r, 12)] {
                if !end {
                    fb.rect_fill(x + ex - 1, y + 1, 4, 15, OUTLINE);
                    fb.rect_fill(x + ex, y + 2, 2, 13, WOOD + 2);
                }
            }
            let (a, b) = (if l { 0 } else { 4 }, if r { 16 } else { 12 });
            fb.rect_fill(x + a, y + 3, b - a, 1, TEXT_DIM);
            if l || r {
                let (cx, cy) = (x + if l && r { 5 } else if r { 6 } else { 4 }, y + 4);
                if k.id == "drying_line" {
                    let c = [THREAD + 1, WATER + 1, WATER, THREAD][(hash(px, py) % 4) as usize];
                    fb.rect_fill(cx - 1, cy - 1, 7, 11, OUTLINE);
                    fb.rect_fill(cx, cy, 5, 9, c);
                    fb.rect_fill(cx, cy, 1, 9, shift(c, 1));
                    fb.rect_fill(cx, cy + 8, 5, 1, shift(c, -1));
                    return;
                }
                match hash(px, py) % 3 {
                    0 => {
                        // A shirt.
                        fb.rect_fill(cx - 1, cy - 1, 8, 8, OUTLINE);
                        fb.rect_fill(cx, cy, 6, 6, COTTON + 2);
                        fb.rect_fill(cx, cy, 6, 1, COTTON + 1);
                    }
                    1 => {
                        // Trousers.
                        fb.rect_fill(cx - 1, cy - 1, 8, 10, OUTLINE);
                        fb.rect_fill(cx, cy, 6, 3, WATER + 1);
                        fb.rect_fill(cx, cy + 3, 2, 5, WATER + 1);
                        fb.rect_fill(cx + 4, cy + 3, 2, 5, WATER + 1);
                    }
                    _ => {
                        // A dyed cloth.
                        fb.rect_fill(cx - 1, cy - 1, 7, 9, OUTLINE);
                        fb.rect_fill(cx, cy, 5, 7, THREAD + 2);
                        fb.rect_fill(cx, cy, 1, 7, COTTON + 2);
                    }
                }
            }
        }
        "timber_gate" | "village_gate" => {
            // A rough timber post on a stone; the beam over the way is overhead.
            fb.rect_fill(x + 5, y, 6, 16, OUTLINE);
            fb.rect_fill(x + 6, y, 4, 13, WOOD + 1);
            fb.rect_fill(x + 6, y, 1, 13, WOOD + 2);
            fb.rect_fill(x + 9, y, 1, 13, WOOD);
            fb.rect_fill(x + 4, y + 12, 8, 4, HUD_MID);
            fb.rect_fill(x + 4, y + 12, 8, 1, HUD_HI);
        }
        "shop_awning" => canopy(fb, x, y, px),
        "gate_post" => {
            // A lacquered red post on a stone foot; the beam over the gate is overhead.
            fb.rect_fill(x + 4, y, 8, 16, OUTLINE);
            fb.rect_fill(x + 5, y, 6, 13, THREAD + 1);
            fb.rect_fill(x + 5, y, 1, 13, THREAD + 2);
            fb.rect_fill(x + 10, y, 1, 13, THREAD);
            fb.rect_fill(x + 4, y + 12, 8, 4, HUD_MID);
            fb.rect_fill(x + 4, y + 12, 8, 1, HUD_HI);
        }
        _ => {
            let Some(s) = sp.prop(k.id, hash(px, py)) else { return };
            let (fw, fh) = (k.size.0 * TS, k.size.1 * TS);
            if !k.on_wall && !k.on_water && k.size == (1, 1) && s.h < TS {
                fb.rect_checker(x + 3, y + TS - 2, 10, 2, SHADOW);
            }
            fb.blit(s, x + (fw - s.w) / 2, y + fh + k.drop - s.h, false);
        }
    }
}

/// A striped canopy across one tile of a shop's front, blue and white with a scalloped edge,
/// reaching up over the way in.
fn canopy(fb: &mut Framebuffer, x: i32, y: i32, tx: i32) {
    fb.rect_fill(x, y - 4, TS, 1, OUTLINE);
    for dx in 0..TS {
        let wx = dx + tx * TS;
        let c = if (wx / 4) % 2 == 0 { WATER + 1 } else { COTTON + 2 };
        let bottom = 6 + (wx % 4) / 2;
        fb.rect_fill(x + dx, y - 3, 1, bottom + 3, c);
        fb.pset(x + dx, y + bottom, OUTLINE);
    }
    fb.rect_fill(x, y - 3, TS, 1, COTTON + 2);
}

/// What hangs over everyone's heads: a gate's beam (and roof) between its two posts, two tiles
/// apart, so whoever walks through the gate walks under it.
fn draw_overhead(g: &Game, fb: &mut Framebuffer, v: &View) {
    for &((px, py), prop) in g.world.props.iter().filter(|((x, y), p)| matches!(p.0, "gate_post" | "timber_gate" | "village_gate") && v.sees(*x, *y)) {
        if g.world.prop_at(px + 2, py).is_none_or(|(_, p)| p != prop) {
            continue;
        }
        let (x, y) = v.px(px, py);
        if matches!(prop.0, "timber_gate" | "village_gate") {
            // A rough-hewn beam on the posts, its cut ends overhanging, a crosspiece below.
            fb.rect_fill(x - 3, y - 7, 54, 7, OUTLINE);
            fb.rect_fill(x - 2, y - 6, 52, 5, WOOD + 1);
            fb.rect_fill(x - 2, y - 6, 52, 1, WOOD + 3);
            fb.rect_fill(x - 2, y - 2, 52, 1, WOOD);
            for ex in [x - 2, x + 49] {
                fb.rect_fill(ex, y - 5, 1, 3, PLANK + 3);
            }
            fb.rect_fill(x + 10, y + 1, 28, 3, OUTLINE);
            fb.rect_fill(x + 10, y + 2, 28, 1, WOOD + 2);
            continue;
        }
        // The tie beam between the posts, the lintel above with a gold plaque, and a tiled roof
        // whose eaves turn up at the ends.
        fb.rect_fill(x + 10, y + 1, 28, 4, OUTLINE);
        fb.rect_fill(x + 10, y + 2, 28, 2, THREAD + 1);
        fb.rect_fill(x - 2, y - 8, 52, 7, OUTLINE);
        fb.rect_fill(x - 1, y - 7, 50, 5, THREAD + 1);
        fb.rect_fill(x - 1, y - 7, 50, 1, THREAD + 2);
        fb.rect_fill(x + 19, y - 9, 10, 9, OUTLINE);
        fb.rect_fill(x + 20, y - 8, 8, 7, SPARK + 1);
        fb.rect_fill(x + 21, y - 6, 6, 1, SPARK);
        fb.rect_fill(x + 21, y - 4, 6, 1, SPARK);
        fb.rect_fill(x - 4, y - 15, 56, 7, OUTLINE);
        fb.rect_fill(x - 3, y - 14, 54, 5, HUD_FILL);
        fb.rect_fill(x - 3, y - 14, 54, 1, HUD_MID);
        for tx in (0..54).step_by(4) {
            fb.rect_fill(x - 3 + tx, y - 12, 1, 3, HUD_SHADOW);
        }
        for (ex, dir) in [(x - 4, -1), (x + 51, 1)] {
            fb.rect_fill(ex, y - 18, 1, 4, OUTLINE);
            fb.rect_fill(ex + dir, y - 19, 1, 3, OUTLINE);
            fb.pset(ex, y - 17, HUD_MID);
        }
    }
}

fn draw_bin(g: &Game, sp: &Sprites, fb: &mut Framebuffer, v: &View, b: &Bin) {
    let (x0, y0) = v.px(b.x, b.y);
    let y0 = y0 - (b.bounce > 1) as i32;
    // One from a kit in its wood's tones.
    let wood = b.frontier.as_ref().and_then(|f| g.data.variety(&f.material.to_lowercase())).and_then(|v| v.color.as_deref()).and_then(ramp_named).map_or(WOOD, |r| r.0);
    fb.rect_bevel(x0 + 1, y0 + 2, 14, 14, wood + 1, wood + 3, wood);
    for y in [y0 + 6, y0 + 10] {
        fb.rect_fill(x0 + 2, y, 12, 1, wood);
    }
    if let Some(icon) = b.accepts.first().and_then(|a| sp.icon(a)) {
        fb.blit(icon, x0 + 4, y0 + 5, false);
    }
    if let Some((mx, my)) = b.mouth {
        if my > b.y {
            fb.rect_fill(x0 + 4, y0 + 14, 8, 2, OUTLINE);
        }
        if !b.stock.is_empty() {
            let (ax, ay) = v.px(mx, my);
            for (i, w) in [3, 2, 1].iter().enumerate() {
                fb.rect_fill(ax + 8 - w, ay + 3 + i as i32, w * 2 + 1, 1, PATH);
            }
        }
    }
    let label = b.stock.len().to_string();
    let col = if b.stock.is_empty() && b.mouth.is_some() { TEXT_WARN } else { TEXT };
    fb.text_centered(&label, x0 + 9, y0 - 8, col, Some(OUTLINE), 1);
}

fn draw_flags(g: &Game, sp: &Sprites, fb: &mut Framebuffer, v: &View) {
    // Flags sharing a tile (a copied loop) stand side by side.
    let mut drawn: Vec<(i32, i32)> = Vec::new();
    // A working townsperson's loop is their trade, not a flag in the ground.
    for m in g.monsters.iter().filter(|m| m.worker.is_none()) {
        let flag = if m.species == Species::Goblin { &sp.flag_goblin } else { &sp.flag_cyclops };
        if let Some((fx, fy)) = m.rec.as_ref().or(m.routine.as_ref()).map(|r| r.flag) {
            let n = drawn.iter().filter(|&&p| p == (fx, fy)).count() as i32;
            drawn.push((fx, fy));
            if v.sees(fx, fy) {
                let (x, y) = v.px(fx, fy);
                fb.blit(flag, x + 1 + n * 5, y + TS - flag.h, false);
            }
        }
    }
}

/// Items and monsters on this screen, back to front by their on-screen feet.
fn draw_bodies(g: &Game, sp: &Sprites, fb: &mut Framebuffer, v: &View) {
    enum Body {
        Item(usize),
        Monster(usize),
        Applicant(usize),
        Townsfolk(usize),
        Wagon(usize),
        Pumpkin(usize),
    }
    let mut bodies: Vec<(i32, Body)> = Vec::new();
    for (i, it) in g.world.items.iter().enumerate().filter(|(_, it)| v.sees(it.x, it.y)) {
        bodies.push((it.y * TS + Slide::offset(it.slide).1, Body::Item(i)));
    }
    for (i, m) in g.monsters.iter().enumerate().filter(|(_, m)| v.sees(m.x, m.y)) {
        bodies.push((m.y * TS + Slide::offset(m.slide).1 + 1, Body::Monster(i)));
    }
    for (k, &(x, y)) in g.world.stands.iter().enumerate() {
        if g.applicants[k].is_some() && v.sees(x, y) {
            bodies.push((y * TS + 1, Body::Applicant(k)));
        }
    }
    for (k, t) in g.world.townsfolk.iter().enumerate().filter(|(_, t)| v.sees(t.x, t.y)) {
        bodies.push((t.y * TS + 1, Body::Townsfolk(k)));
    }
    for (k, w) in g.world.wagons.iter().enumerate() {
        if v.sees(w.at.0, w.at.1) {
            bodies.push((w.at.1 * TS + Slide::offset(w.slide).1, Body::Wagon(k)));
        }
        if v.sees(w.pumpkin.0, w.pumpkin.1) {
            bodies.push((w.pumpkin.1 * TS + Slide::offset(w.pumpkin_slide).1, Body::Pumpkin(k)));
        }
    }
    bodies.sort_by_key(|b| b.0);
    for (_, b) in bodies {
        match b {
            Body::Item(i) => {
                let it = &g.world.items[i];
                // A tool on a peg hangs there as its icon, where the empty peg's ghost was.
                let hung = g.world.tile(it.x, it.y) == Tile::Peg && it.slide.is_none();
                let gt = g.data.try_good(it.good());
                let (s, typed) = match sp.icon(it.good()).filter(|_| hung) {
                    Some(icon) => (icon, None),
                    None => sp.item(it.good(), gt),
                };
                let (ox, oy) = Slide::offset(it.slide);
                let (tx, ty) = v.px(it.x, it.y);
                let jitter = if it.locked { (g.frame / 2 % 2) as i32 } else { 0 };
                let x = tx + ox + (TS - s.w) / 2 + jitter;
                let y = if hung { ty + 5 } else { ty + TS - s.h + oy - HOP[it.hop.min(6) as usize] };
                if !hung {
                    fb.rect_checker(tx + ox + 3, ty + TS + oy - 2, 10, 2, SHADOW);
                }
                // Ungraded goods have no quality to show: no tint, no twinkle.
                let graded = gt.is_some_and(|gt| gt.has_quality);
                let q = it.data.quality;
                // Drawn with its type's art, it shows its own variety's color.
                let own = typed.zip(gt.and_then(|gt| g.data.own_color(gt)).and_then(ramp_named)).map(|(from, to)| Tint { from, to });
                blit_quality(fb, s, x, y, if graded { quality_level(q) } else { 0 }, tint(&g.data, &it.data).or(own));
                if graded && q >= EXCEPTIONAL {
                    // A slow twinkle: exceptional goods every two seconds, masterworks twice as often.
                    let period = if q >= MASTERWORK { 15 } else { 30 };
                    let phase = (g.frame + (it.x * 7 + it.y * 13) as u32) % period;
                    let (gx, gy) = (x + s.w - 2, y + 1);
                    if phase < 3 {
                        fb.pset(gx, gy, SPARK + 2);
                    }
                    if phase == 1 {
                        for (dx, dy) in [(-1, 0), (1, 0), (0, -1), (0, 1)] {
                            fb.pset(gx + dx, gy + dy, SPARK + 1);
                        }
                    }
                }
            }
            Body::Applicant(k) => {
                let Some(a) = &g.applicants[k] else { continue };
                let frames = if a.kind == "goblin" { &sp.goblin } else { &sp.cyclops };
                let s = &frames[(g.frame / 12 + k as u32) as usize % 2];
                let (tx, ty) = v.px(g.world.stands[k].0, g.world.stands[k].1);
                fb.rect_checker(tx + 3, ty + TS - 2, 10, 2, SHADOW);
                fb.blit(s, tx + (TS - s.w) / 2, ty + TS - s.h, k % 2 == 1);
            }
            Body::Townsfolk(k) => {
                let t = &g.world.townsfolk[k];
                let (tx, ty) = v.px(t.x, t.y);
                fb.rect_checker(tx + 3, ty + TS - 2, 10, 2, SHADOW);
                match t.kind {
                    crate::world::Folk::Craftsman => {
                        // An old carpenter: weathered tan where a goblin is green, a saw on his back.
                        let s = &sp.goblin[(g.frame / 12) as usize % 2];
                        let (x, y) = (tx + (TS - s.w) / 2, ty + TS - s.h);
                        blit_tinted(fb, s, x, y, true, ramp_named("plank").map(|to| Tint { from: (GOBLIN, 3), to }));
                        fb.rect_fill(x + s.w - 2, y + 2, 1, 3, WOOD + 2);
                        for k in 0..6 {
                            fb.pset(x + s.w - 1, y + 5 + k, if k % 2 == 0 { TEXT_DIM } else { TEXT });
                        }
                    }
                    crate::world::Folk::Outfitter => {
                        // A goblin dyed indigo, as befits the dyers' town, a tape measure round
                        // the neck.
                        let s = &sp.goblin[(g.frame / 12) as usize % 2];
                        let (x, y) = (tx + (TS - s.w) / 2, ty + TS - s.h);
                        blit_tinted(fb, s, x, y, false, ramp_named("water").map(|to| Tint { from: (GOBLIN, 3), to }));
                        fb.rect_fill(x + 5, y + 11, 6, 1, SPARK + 1);
                        fb.rect_fill(x + 5, y + 12, 1, 2, SPARK + 1);
                        fb.pset(x + 10, y + 12, SPARK + 1);
                    }
                    crate::world::Folk::Keeper => {
                        // A cyclops in saffron, as quiet as the temple, prayer beads at the neck.
                        let s = &sp.cyclops[(g.frame / 16) as usize % 2];
                        let (x, y) = (tx + (TS - s.w) / 2, ty + TS - s.h);
                        blit_tinted(fb, s, x, y, true, Some(Tint { from: (CYCLOPS, 3), to: (SPARK, 3) }));
                        for bx in [5, 7, 9, 11] {
                            fb.pset(x + bx, y + 11, THREAD);
                        }
                        fb.pset(x + 8, y + 12, THREAD);
                    }
                    crate::world::Folk::Trader => {
                        // Now and then he lifts a hand to passers-by.
                        let s = &sp.trader[((g.frame / 20) % 6 == 5) as usize];
                        fb.blit(s, tx + (TS - s.w) / 2, ty + TS - s.h, false);
                    }
                }
            }
            Body::Wagon(k) => {
                let w = &g.world.wagons[k];
                let s = &sp.wagon[w.open as usize];
                let (tx, ty) = v.px(w.at.0, w.at.1);
                let (ox, oy) = Slide::offset(w.slide);
                let (x, y) = (tx + ox + (TS - s.w) / 2, ty + oy + TS - s.h - (w.bounce > 1) as i32);
                fb.rect_checker(tx + ox + 2, ty + oy + TS - 2, 12, 2, SHADOW);
                // The tailgate is drawn on the left; face it away from the pumpkin. One from a
                // kit in its wood's tones.
                let wood = g.data.variety(&w.material.to_lowercase()).and_then(|v| v.color.as_deref()).and_then(ramp_named);
                blit_tinted(fb, s, x, y, w.pumpkin.0 < w.at.0, wood_tint(wood));
                if let Some(icon) = w.good().and_then(|good| sp.icon(good)) {
                    fb.blit(icon, x + 1, y - 2, false);
                }
                if !w.stock.is_empty() {
                    badge(fb, tx + ox, ty + oy, w.stock.len());
                }
            }
            Body::Pumpkin(k) => {
                let w = &g.world.wagons[k];
                let moving = w.pumpkin_slide.is_some();
                let s = &sp.pumpkin[if moving { (g.frame / 2 % 2) as usize } else { (g.frame / 15 % 2) as usize }];
                let (tx, ty) = v.px(w.pumpkin.0, w.pumpkin.1);
                let (ox, oy) = Slide::offset(w.pumpkin_slide);
                fb.rect_checker(tx + ox + 3, ty + oy + TS - 2, 10, 2, SHADOW);
                fb.blit(s, tx + ox + (TS - s.w) / 2, ty + oy + TS - s.h, w.at.0 > w.pumpkin.0);
                // The lead rope from the pumpkin's stem to its leader's hand.
                if let Some(m) = w.leader.and_then(|id| g.monsters.iter().find(|m| m.id == id)) {
                    let (mx, my) = v.px(m.x, m.y);
                    let (mox, moy) = Slide::offset(m.slide);
                    let (ax, ay) = (tx + ox + 8, ty + oy + TS - s.h + 1);
                    let (bx, by) = (mx + mox + 8, my + moy + 9);
                    let n = (bx - ax).abs().max((by - ay).abs()).max(1);
                    for k in 0..=n {
                        fb.pset(ax + (bx - ax) * k / n, ay + (by - ay) * k / n + if k > 0 && k < n { 1 } else { 0 }, WOOD + 3);
                    }
                }
            }
            Body::Monster(i) => {
                let (x, y, s, flip) = monster_pose(g, sp, v, i);
                let m = &g.monsters[i];
                let (ox, oy) = Slide::offset(m.slide);
                let (tx, ty) = v.px(m.x, m.y);
                fb.rect_checker(tx + ox + 3, ty + TS + oy - 2, 10, 2, SHADOW);
                if let Some(w) = m.worker.and_then(|n| g.world.workers.get(n)) {
                    // A working townsperson: weathered, in their trade's own tones (leather, soot).
                    blit_tinted(fb, &s, x, y, flip, Some(Tint { from: (GOBLIN, 3), to: w.skin }));
                } else {
                    fb.blit(&s, x, y, flip);
                }
                // What it holds in each hand that's full: a tool's handle and its head (a
                // shovel's blade at the bottom), or a carried piece. A hand past its type's is a
                // tool belt's: what's in it hangs at the front of the waist, shorter.
                let base = g.data.monster_types[m.kind.as_str()].slots.hands as usize;
                for (h, held) in m.hands.iter().enumerate() {
                    let belt = h >= base;
                    let hx = match (belt, (h == 0) != flip) {
                        (true, _) => x + if flip { s.w / 2 - 3 } else { s.w / 2 + 2 },
                        (false, true) => x + s.w - 3,
                        (false, false) => x + 2,
                    };
                    let (top, len) = if belt { (y + s.h - 6, 4) } else { (y + s.h - 9, 6) };
                    match held {
                        Some(crate::game::Held::Tool(held)) => {
                            fb.rect_fill(hx, top, 1, len, WOOD + 2);
                            if g.data.try_good(&held.good).is_some_and(|t| t.type_tags.iter().any(|t| t == "shovel")) {
                                fb.rect_fill(hx - 1, top + len - 1, 3, 2, TEXT_DIM);
                                fb.pset(hx - 1, top + len - 1, HUD_HI);
                            } else {
                                fb.rect_fill(hx - 1, top - 1, 3, 1, TEXT_DIM);
                            }
                        }
                        Some(crate::game::Held::Piece(id)) => {
                            if let Some(p) = g.data.try_piece(id) {
                                draw_carried(fb, p, hx, top);
                            }
                        }
                        None => {}
                    }
                }
            }
        }
    }
}

/// A piece carried in a hand, beside a monster at `hx`, from row `top` down to the hand: a towel
/// hanging from it, a whetstone, a magnifying glass, a watch on the wrist, a brush, a lucky coin on
/// its string. Outlined, as everything in the foreground is.
fn draw_carried(fb: &mut Framebuffer, p: &crate::data::Piece, hx: i32, top: i32) {
    let c = p.color.as_ref().and_then(|(n, s)| color_named(n, *s)).unwrap_or(TEXT);
    let blob = |fb: &mut Framebuffer, x: i32, y: i32, w: i32, h: i32, c: u8| {
        fb.rect_fill(x - 1, y - 1, w + 2, h + 2, OUTLINE);
        fb.rect_fill(x, y, w, h, c);
        fb.rect_fill(x, y + h - 1, w, 1, shift(c, -1));
    };
    match p.look() {
        // Upright like a staff, from above the head to the ground, a rope loop round its top.
        "pole" => {
            fb.rect_fill(hx - 1, top - 6, 3, 15, OUTLINE);
            fb.rect_fill(hx, top - 5, 1, 13, c);
            fb.rect_fill(hx - 2, top - 4, 5, 4, OUTLINE);
            fb.rect_fill(hx - 1, top - 3, 3, 2, MAT + 2);
            fb.pset(hx, top - 3, c);
        }
        "towel" => {
            blob(fb, hx - 1, top + 3, 3, 5, c);
            fb.rect_fill(hx - 1, top + 5, 3, 1, THREAD + 1);
        }
        "whetstone" => blob(fb, hx - 1, top + 5, 3, 2, c),
        "magnifier" => {
            fb.rect_fill(hx - 1, top + 4, 3, 4, OUTLINE);
            fb.rect_fill(hx, top + 5, 1, 2, WOOD + 1);
            fb.rect_fill(hx - 2, top, 5, 5, OUTLINE);
            fb.rect_fill(hx - 1, top + 1, 3, 3, SPARK + 1);
            fb.pset(hx, top + 2, c);
        }
        "watch" => {
            fb.rect_fill(hx - 2, top + 4, 5, 3, OUTLINE);
            fb.rect_fill(hx - 1, top + 5, 3, 1, c);
            fb.pset(hx, top + 5, COTTON + 2);
        }
        "pen" => {
            fb.rect_fill(hx - 1, top - 1, 3, 8, OUTLINE);
            fb.rect_fill(hx, top + 1, 1, 5, c);
            fb.pset(hx, top, INK);
        }
        "lucky_charm" => {
            fb.rect_fill(hx - 1, top + 3, 3, 2, OUTLINE);
            fb.pset(hx, top + 4, THREAD + 1);
            blob(fb, hx, top + 5, 2, 2, c);
        }
        _ => blob(fb, hx - 1, top + 4, 3, 3, c),
    }
}

/// A rack: a little wooden stand with its piece hung on it (or, shoes, set at its foot), in the
/// piece's color.
fn draw_rack(g: &Game, fb: &mut Framebuffer, piece: &str, x0: i32, y0: i32) {
    let p = g.data.piece(piece);
    let c = p.color.as_ref().and_then(|(n, s)| color_named(n, *s)).unwrap_or(TEXT);
    let dark = shift(c, -1);
    // The stand: a post on a foot, and a crossbar.
    fb.rect_fill(x0 + 3, y0 + 13, 10, 2, OUTLINE);
    fb.rect_fill(x0 + 4, y0 + 13, 8, 1, WOOD + 2);
    fb.rect_fill(x0 + 7, y0 + 4, 2, 10, OUTLINE);
    fb.rect_fill(x0 + 7, y0 + 4, 1, 9, WOOD + 2);
    fb.rect_fill(x0 + 2, y0 + 3, 12, 2, OUTLINE);
    fb.rect_fill(x0 + 2, y0 + 3, 12, 1, WOOD + 3);
    let outlined = |fb: &mut Framebuffer, x: i32, y: i32, w: i32, h: i32| {
        fb.rect_fill(x0 + x - 1, y0 + y - 1, w + 2, h + 2, OUTLINE);
        fb.rect_fill(x0 + x, y0 + y, w, h, c);
        fb.rect_fill(x0 + x, y0 + y + h - 1, w, 1, dark);
    };
    match p.look() {
        "belt" => {
            outlined(fb, 3, 6, 10, 2);
            fb.rect_fill(x0 + 7, y0 + 6, 2, 2, SPARK + 1);
        }
        "gloves" => {
            for gx in [3, 10] {
                outlined(fb, gx, 6, 3, 4);
                fb.pset(x0 + gx + if gx < 8 { -1 } else { 3 }, y0 + 7, c);
            }
        }
        "shoes" => {
            outlined(fb, 2, 11, 4, 2);
            outlined(fb, 10, 11, 4, 2);
        }
        "headband" => {
            outlined(fb, 4, 5, 8, 1);
            outlined(fb, 4, 9, 8, 1);
            fb.rect_fill(x0 + 4, y0 + 5, 1, 5, c);
            fb.rect_fill(x0 + 11, y0 + 5, 1, 5, c);
        }
        "charm" | "earrings" => {
            let at: &[i32] = if p.look() == "charm" { &[7] } else { &[4, 11] };
            for &bx in at {
                fb.rect_fill(x0 + bx, y0 + 5, 1, 3, TEXT_DIM);
                outlined(fb, bx, 8, 2, 2);
                fb.pset(x0 + bx, y0 + 8, shift(c, 1));
            }
        }
        "towel" => {
            // Folded over the bar, a red stripe near its hem.
            outlined(fb, 4, 4, 8, 7);
            fb.rect_fill(x0 + 4, y0 + 8, 8, 1, THREAD + 1);
            fb.rect_fill(x0 + 4, y0 + 4, 8, 1, shift(c, 1));
        }
        "whetstone" => {
            // On the stand's foot, a little water on it.
            outlined(fb, 4, 10, 8, 2);
            fb.rect_fill(x0 + 5, y0 + 10, 3, 1, WATER + 2);
        }
        "magnifier" => {
            fb.rect_fill(x0 + 7, y0 + 5, 2, 1, WOOD + 1);
            fb.rect_fill(x0 + 4, y0 + 5, 8, 7, OUTLINE);
            fb.rect_fill(x0 + 5, y0 + 6, 6, 5, SPARK + 1);
            fb.rect_fill(x0 + 6, y0 + 7, 4, 3, c);
            fb.pset(x0 + 6, y0 + 7, COTTON + 2);
        }
        "watch" => {
            // Hung by its strap, its face in the middle.
            fb.rect_fill(x0 + 6, y0 + 5, 4, 8, OUTLINE);
            fb.rect_fill(x0 + 7, y0 + 5, 2, 7, WOOD + 1);
            outlined(fb, 6, 7, 4, 3);
            fb.rect_fill(x0 + 7, y0 + 8, 2, 1, COTTON + 2);
        }
        "pen" => {
            fb.rect_fill(x0 + 7, y0 + 4, 3, 9, OUTLINE);
            fb.rect_fill(x0 + 8, y0 + 5, 1, 6, c);
            fb.pset(x0 + 8, y0 + 11, INK);
        }
        "lucky_charm" => {
            fb.rect_fill(x0 + 7, y0 + 5, 1, 3, THREAD + 1);
            outlined(fb, 6, 8, 3, 3);
            fb.pset(x0 + 7, y0 + 9, shift(c, -1));
        }
        "apron" => {
            // Its ties over the bar, a pocket across the front.
            outlined(fb, 5, 5, 6, 7);
            fb.rect_fill(x0 + 4, y0 + 5, 1, 2, c);
            fb.rect_fill(x0 + 11, y0 + 5, 1, 2, c);
            fb.rect_fill(x0 + 6, y0 + 9, 4, 1, shift(c, -1));
        }
        "tool_belt" => {
            // A belt with a buckle and two pouches, a hammer's handle up out of the first.
            fb.rect_fill(x0 + 4, y0 + 1, 3, 7, OUTLINE);
            fb.rect_fill(x0 + 3, y0 + 1, 5, 3, OUTLINE);
            outlined(fb, 2, 6, 12, 2);
            fb.rect_fill(x0 + 7, y0 + 6, 2, 2, SPARK + 1);
            for (px, w, h) in [(3, 4, 4), (9, 4, 3)] {
                outlined(fb, px, 8, w, h);
                fb.rect_fill(x0 + px, y0 + 8, w, 1, shift(c, 1));
            }
            fb.rect_fill(x0 + 5, y0 + 2, 1, 6, WOOD + 2);
            fb.rect_fill(x0 + 4, y0 + 2, 3, 1, TEXT_DIM);
        }
        "backpack" => {
            // Hung by its straps, a flap over the top and a pocket on the front.
            fb.rect_fill(x0 + 4, y0 + 2, 3, 4, OUTLINE);
            fb.rect_fill(x0 + 9, y0 + 2, 3, 4, OUTLINE);
            fb.rect_fill(x0 + 5, y0 + 3, 1, 2, shift(c, -1));
            fb.rect_fill(x0 + 10, y0 + 3, 1, 2, shift(c, -1));
            outlined(fb, 4, 5, 8, 7);
            fb.rect_fill(x0 + 4, y0 + 5, 8, 2, shift(c, 1));
            fb.pset(x0 + 7, y0 + 7, SPARK + 1);
            fb.rect_fill(x0 + 5, y0 + 9, 6, 1, shift(c, -1));
        }
        "cap" => {
            // Hung on the bar, its peak to the right, a red feather up and back from its crown.
            for (fx, fy) in [(4, 5), (3, 4), (3, 3)] {
                fb.rect_fill(x0 + fx - 1, y0 + fy - 1, 3, 3, OUTLINE);
            }
            for (fx, fy) in [(4, 5), (3, 4), (3, 3)] {
                fb.pset(x0 + fx, y0 + fy, THREAD + 1);
            }
            outlined(fb, 5, 5, 6, 3);
            fb.rect_fill(x0 + 4, y0 + 8, 10, 2, OUTLINE);
            fb.rect_fill(x0 + 5, y0 + 8, 8, 1, shift(c, -1));
            fb.rect_fill(x0 + 6, y0 + 5, 4, 1, shift(c, 1));
        }
        "pole" => {
            // Leant across the stand corner to corner, a rope ring round each end.
            let along = |k: i32| (x0 + 2 + k, y0 + 13 - k);
            for k in 0..12 {
                let (px, py) = along(k);
                fb.rect_fill(px - 1, py - 1, 3, 3, OUTLINE);
            }
            for k in [1, 10] {
                let (px, py) = along(k);
                fb.rect_fill(px - 2, py - 2, 5, 5, OUTLINE);
                fb.rect_fill(px - 1, py - 1, 3, 3, MAT + 2);
            }
            for k in 0..12 {
                let (px, py) = along(k);
                fb.pset(px, py, c);
            }
        }
        "notebook" => {
            outlined(fb, 5, 6, 6, 6);
            fb.rect_fill(x0 + 10, y0 + 6, 1, 6, COTTON + 2);
        }
        "seal" => {
            // A stone seal on the stand's foot, its red mark beside it.
            outlined(fb, 6, 8, 3, 4);
            fb.rect_fill(x0 + 10, y0 + 11, 2, 1, THREAD + 1);
        }
        "song" => {
            // A scroll of music: two notes on its paper.
            fb.rect_fill(x0 + 4, y0 + 5, 8, 8, OUTLINE);
            fb.rect_fill(x0 + 5, y0 + 6, 6, 6, COTTON + 2);
            for (nx, ny) in [(6, 8), (9, 7)] {
                fb.rect_fill(x0 + nx - 1, y0 + ny + 1, 2, 2, INK);
                fb.rect_fill(x0 + nx, y0 + ny - 2, 1, 3, INK);
            }
            fb.rect_fill(x0 + 6, y0 + 6, 4, 1, INK);
            fb.rect_fill(x0 + 4, y0 + 5, 8, 1, c);
            fb.rect_fill(x0 + 4, y0 + 12, 8, 1, shift(c, -1));
        }
        k if k == "teaching" || k == "patience" || k.starts_with("secret_") => {
            // A scroll hung from the bar: rolled top and bottom, its paper between.
            fb.rect_fill(x0 + 4, y0 + 5, 8, 8, OUTLINE);
            fb.rect_fill(x0 + 5, y0 + 6, 6, 6, COTTON + 2);
            for (row, w) in [(7, 4), (9, 3)] {
                fb.rect_fill(x0 + 6, y0 + row, w, 1, INK);
            }
            fb.rect_fill(x0 + 4, y0 + 5, 8, 1, c);
            fb.rect_fill(x0 + 4, y0 + 12, 8, 1, shift(c, -1));
        }
        _ => outlined(fb, 4, 5, 8, 6),
    }
}

/// Monster `i`'s frames with what it has on drawn in, if it has anything on that shows.
fn dressed(g: &Game, i: usize) -> Option<[Sprite; 4]> {
    let m = &g.monsters[i];
    let look: Vec<(&str, u8)> = m.gear.iter().filter_map(|id| g.data.try_piece(id)).filter_map(|p| Some((p.look(), p.color.as_ref().and_then(|(n, s)| color_named(n, *s))?))).collect();
    (!look.is_empty()).then(|| crate::sprites::dressed_frames(m.species == Species::Goblin, &look))
}

fn monster_pose<'a>(g: &Game, sp: &'a Sprites, v: &View, i: usize) -> (i32, i32, std::borrow::Cow<'a, Sprite>, bool) {
    let m = &g.monsters[i];
    let frames = if m.species == Species::Goblin { &sp.goblin } else { &sp.cyclops };
    let frame = if m.slide.is_some() {
        2 + m.walk_frame as usize
    } else if m.busy {
        (g.frame / 2 % 2) as usize
    } else {
        (g.frame / 10 % 2) as usize
    };
    let s: std::borrow::Cow<'a, Sprite> = match dressed(g, i) {
        Some(worn) => std::borrow::Cow::Owned(worn[frame].clone()),
        None => std::borrow::Cow::Borrowed(&frames[frame]),
    };
    let (mut ox, mut oy) = Slide::offset(m.slide);
    if let Some((d, t)) = m.bump {
        let (dx, dy) = d.delta();
        let k = if t >= 2 { 2 } else { 1 };
        ox += dx * k;
        oy += dy * k;
    }
    let (tx, ty) = v.px(m.x, m.y);
    (tx + ox + (TS - s.w) / 2, ty + TS - s.h + oy, s, m.flip)
}

fn draw_particles(g: &Game, fb: &mut Framebuffer, v: &View) {
    let (cx, cy) = (v.tx * TS, v.ty * TS);
    for p in &g.world.particles {
        let (x, y) = (p.x - cx, p.y - cy);
        if y >= VIEW_H {
            continue;
        }
        match p.kind {
            PKind::Dust => fb.pset(x, y, DUST),
            PKind::Leaf => fb.pset(x, y, LEAF + 2),
            PKind::Fluff => {
                fb.pset(x, y, if p.life > 4 { COTTON + 2 } else { COTTON + 1 });
                if p.life > 6 {
                    fb.pset(x + 1, y, COTTON + 1);
                }
            }
            // An eighth note, outlined as everything in the foreground is: its stem and flag,
            // its head at the foot.
            PKind::Note => {
                const NOTE: [(i32, i32); 9] = [(0, -3), (0, -2), (1, -2), (0, -1), (0, 0), (-1, 1), (0, 1), (-1, 2), (0, 2)];
                for (dx, dy) in NOTE {
                    fb.rect_fill(x + dx - 1, y + dy - 1, 3, 3, OUTLINE);
                }
                for (dx, dy) in NOTE {
                    fb.pset(x + dx, y + dy, COTTON + 2);
                }
            }
            PKind::Spark => {
                fb.pset(x, y, SPARK + 2);
                if p.life > 6 {
                    for (dx, dy) in [(-1, 0), (1, 0), (0, -1), (0, 1)] {
                        fb.pset(x + dx, y + dy, SPARK + 1);
                    }
                }
            }
        }
    }
}

fn draw_markers(g: &Game, sp: &Sprites, fb: &mut Framebuffer, v: &View) {
    for (i, m) in g.monsters.iter().enumerate().filter(|(_, m)| v.sees(m.x, m.y)) {
        let (x, _, s, _) = monster_pose(g, sp, v, i);
        let (_, oy) = Slide::offset(m.slide);
        let top = v.px(m.x, m.y).1 + oy + 2;
        if i == v.c {
            let bob = (g.frame / 8 % 2) as i32;
            fb.blit(&sp.marker, x + s.w / 2 - 3, top - 6 + bob, false);
        }
        let (bx, by) = (x + s.w - 3, top);
        if m.worker.is_some() {
            continue;
        }
        if m.rec.is_some() {
            fb.rect_fill(bx, by, 5, 5, OUTLINE);
            fb.rect_fill(bx + 1, by + 1, 3, 3, REC + 1);
            fb.pset(bx + 1, by + 1, TEXT);
        } else if m.playing {
            fb.blit(&sp.loop_icon, bx - 1, by - 2, false);
        }
    }
}

/// What a monster is doing, in a word and a color, for edge markers and the roster.
fn state_of(g: &Game, i: usize) -> (&'static str, u8) {
    let m = &g.monsters[i];
    if m.busy {
        ("WORKING", SPARK + 1)
    } else if m.playing && m.wait >= 12 {
        ("WAITING", TEXT_WARN)
    } else if m.playing {
        ("LOOPING", LOOP + 1)
    } else if m.rec.is_some() {
        ("RECORDING", REC + 1)
    } else {
        ("IDLE", TEXT_DIM)
    }
}

/// Monsters on other screens get a marker at the edge pointing their way, colored by what
/// they are doing, so a stuck one is noticeable from anywhere.
fn draw_edge_markers(g: &Game, fb: &mut Framebuffer, v: &View) {
    let mut placed: Vec<(i32, i32)> = Vec::new();
    // Nobody's marker for a working townsperson: they're the town's, not anyone's to find.
    for (i, m) in g.monsters.iter().enumerate().filter(|(_, m)| !v.sees(m.x, m.y) && m.worker.is_none()) {
        let (px, py) = v.px(m.x, m.y);
        let (x, y) = ((px + 8).clamp(9, 391), (py + 8).clamp(9, VIEW_H - 9));
        // Markers sharing a spot slide along the edge to the nearest clear one; if the edge is
        // full, they overlap.
        let clear = |(x, y): (i32, i32)| !placed.iter().any(|&(ox, oy)| (ox - x).abs() < 14 && (oy - y).abs() < 14);
        let along = x == 9 || x == 391;
        let (x, y) = (0..30)
            .flat_map(|k| [k * 14, -k * 14])
            .map(|o| if along { (x, y + o) } else { (x + o, y) })
            .filter(|&(x, y)| (9..=391).contains(&x) && (9..=VIEW_H - 9).contains(&y))
            .find(|&p| clear(p))
            .unwrap_or((x, y));
        placed.push((x, y));
        let (_, col) = state_of(g, i);
        fb.rect_fill(x - 6, y - 6, 13, 13, OUTLINE);
        fb.rect_fill(x - 5, y - 5, 11, 11, col);
        fb.rect_fill(x - 4, y - 4, 9, 9, HUD_FILL);
        let initial = &m.name[..1];
        fb.text(initial, x - 2, y - 3, TEXT, None, 1);
        // A notch on the side facing the monster.
        let (nx, ny) = (if px + 8 < 9 { -8 } else if px + 8 > 391 { 7 } else { 0 }, if py + 8 < 9 { -8 } else if py + 8 > VIEW_H - 9 { 7 } else { 0 });
        if nx != 0 || ny != 0 {
            fb.rect_fill(x + nx - if nx == 0 { 1 } else { 0 }, y + ny - if ny == 0 { 1 } else { 0 }, if nx == 0 { 3 } else { 1 }, if ny == 0 { 3 } else { 1 }, col);
        }
    }
}

/// Holding Tab: every monster of the player's commune, and what it is doing.
fn draw_roster(g: &Game, sp: &Sprites, fb: &mut Framebuffer, v: &View) {
    let own: Vec<usize> = (0..g.monsters.len()).filter(|&i| g.monsters[i].commune == v.k).collect();
    let n = own.len() as i32;
    let w = 96;
    let per_row = 4;
    let rows = (n + per_row - 1) / per_row;
    let h = rows * 26 + 6;
    let (x0, y0) = ((400 - per_row.min(n) * w - 6) / 2, VIEW_H - h - 4);
    fb.rect_bevel(x0, y0, per_row.min(n) * w + 6, h, HUD_FILL, HUD_HI, HUD_SHADOW);
    for (n, &i) in own.iter().enumerate() {
        let m = &g.monsters[i];
        let (col, row) = (n as i32 % per_row, n as i32 / per_row);
        let (x, y) = (x0 + 3 + col * w, y0 + 3 + row * 26);
        if i == v.c {
            fb.rect_fill(x, y, w, 26, HUD_MID);
        }
        let frames = if m.species == Species::Goblin { &sp.goblin } else { &sp.cyclops };
        fb.blit(&frames[0], x + 2, y + 25 - frames[0].h, false);
        let (state, color) = state_of(g, i);
        fb.text(&m.name, x + 20, y + 3, TEXT, Some(HUD_SHADOW), 1);
        fb.text(state, x + 20, y + 14, color, Some(HUD_SHADOW), 1);
    }
}

/// The area's name, shown for a moment on arrival.
/// News for everyone, such as a bell hung in the temple: a headline in big letters and a line
/// under it, below the area banner.
fn draw_announcement(g: &Game, fb: &mut Framebuffer) {
    let Some(a) = &g.announcement else { return };
    let w = (Framebuffer::text_width(&a.title, 2).max(Framebuffer::text_width(&a.line, 1)) + 24).min(396);
    let (x, y) = ((400 - w) / 2, 40);
    fb.rect_bevel(x, y, w, 40, HUD_FILL, HUD_HI, HUD_SHADOW);
    fb.text_centered(&a.title, 200, y + 6, SPARK + 2, Some(HUD_SHADOW), 2);
    fb.text_centered(&a.line, 200, y + 26, TEXT, Some(HUD_SHADOW), 1);
}

fn draw_banner(me: &crate::game::Commune, fb: &mut Framebuffer) {
    let Some((name, t)) = me.banner else { return };
    let drop = (45 - t as i32).min(4) * 2 - 8;
    let w = Framebuffer::text_width(name, 2) + 20;
    let (x, y) = ((400 - w) / 2, 10 + drop);
    fb.rect_bevel(x, y, w, 24, HUD_FILL, HUD_HI, HUD_SHADOW);
    fb.text_centered(name, 200, y + 5, SPARK + 2, Some(HUD_SHADOW), 2);
}

fn jobs(g: &Game) -> impl Iterator<Item = ((i32, i32), &Job)> {
    g.world
        .shops
        .iter()
        .filter_map(|s| s.job.as_ref().map(|j| (s.spot, j)))
        .chain(g.world.plants.iter().filter_map(|p| p.harvest.as_ref().map(|j| ((p.x, p.y), j))))
}

fn progress_spots(g: &Game) -> impl Iterator<Item = (i32, i32)> + '_ {
    jobs(g).map(|(at, _)| at)
}

/// Progress bars over work spots and plants being harvested.
fn draw_progress(g: &Game, fb: &mut Framebuffer, v: &View) {
    for ((tx, ty), j) in jobs(g).filter(|((x, y), _)| v.sees(*x, *y)) {
        let (x, y) = v.px(tx, ty);
        let (x, y) = (x - 1, y - 13);
        let fill = (j.t as i32 * 16 / j.total.max(1) as i32).clamp(1, 16);
        fb.rect_fill(x, y, 18, 5, OUTLINE);
        fb.rect_fill(x + 1, y + 1, 16, 3, HUD_SHADOW);
        fb.rect_fill(x + 1, y + 1, fill, 3, SPARK + 1);
        fb.rect_fill(x + 1, y + 1, fill, 1, SPARK + 2);
    }
}

struct BubbleBox {
    x: i32,
    y: i32,
    w: i32,
    h: i32,
    cx: i32,
    above: bool, // bubble sits above the monster, tail points down
    lines: Vec<String>,
    icon: Option<String>,
    /// A card icon after the good's, such as the hourglass of a ready wait.
    mark: Option<Icon>,
    warn: bool,
    /// The frame it appeared on, and on the same frame which goes on top: what's faced (0),
    /// what's said (1), what the steered monster says (2).
    born: u32,
    rank: u8,
}

/// Places speech bubbles so they never cover each other: above the head if there is room,
/// otherwise below the feet, otherwise nudged upward until clear. Controlled monster first.
fn layout_bubbles(g: &Game, v: &View) -> Vec<BubbleBox> {
    let mut placed: Vec<BubbleBox> = Vec::new();
    let order = std::iter::once(v.c).chain((0..g.monsters.len()).filter(|&i| i != v.c));
    for i in order {
        let m = &g.monsters[i];
        let Some(b) = &m.bubble else { continue };
        // A working townsperson says nothing unasked: facing them shows their talk.
        if !v.sees(m.x, m.y) || m.worker.is_some() {
            continue;
        }
        // Monsters you are not steering only show what they wait for, not a sentence.
        let compact = b.waiting && i != v.c;
        let icon = if compact {
            g.data.goods.iter().find(|gd| m.wait_reason.contains(&gd.name.to_uppercase())).map(|gd| gd.id.clone())
        } else {
            None
        };
        // Brought up beside its goal, waiting its turn: the good and an hourglass, not "?".
        let mark = (compact && icon.is_some() && m.wait_reason.starts_with("READY")).then_some(Icon::Time);
        let lines = match (compact, &icon) {
            (false, _) => wrap(&b.text, 24),
            (true, Some(_)) if mark.is_some() => Vec::new(),
            (true, Some(_)) => vec!["?".to_string()],
            (true, None) => vec!["...".to_string()],
        };
        let icon_w = if icon.is_some() { 9 } else { 0 } + if mark.is_some() { 7 } else { 0 };
        let w = lines.iter().map(|l| l.len() as i32).max().unwrap_or(0) * 6 + 6 + icon_w;
        let h = lines.len().max(1) as i32 * 9 + 4;
        let (ox, oy) = Slide::offset(m.slide);
        let (mx, my) = v.px(m.x, m.y);
        let cx = mx + ox + 8;
        let (head, feet) = (my + oy + 2, my + TS + oy);
        let x = (cx - w / 2).clamp(2, 398 - w);
        let mut bodies: Vec<(i32, i32, i32, i32)> = (g.monsters.iter().enumerate())
            .filter(|(j, o)| *j != i && v.sees(o.x, o.y))
            .map(|(_, o)| {
                let (px, py) = v.px(o.x, o.y);
                (px + Slide::offset(o.slide).0, py + Slide::offset(o.slide).1, TS, TS)
            })
            .collect();
        bodies.extend(progress_spots(g).filter(|(x, y)| v.sees(*x, *y)).map(|(x, y)| {
            let (px, py) = v.px(x, y);
            (px - 1, py - 13, 18, 5)
        }));
        let clear = |y: i32, placed: &[BubbleBox]| {
            let hits = |(px, py, pw, ph): (i32, i32, i32, i32)| x <= px + pw && px <= x + w && y <= py + ph && py <= y + h;
            y >= 2 && y + h <= VIEW_H - 2 && !placed.iter().any(|p| hits((p.x, p.y, p.w, p.h))) && !bodies.iter().any(|&r| hits(r))
        };
        let mut y = head - h - 5;
        let mut above = true;
        if !clear(y, &placed) {
            if clear(feet + 4, &placed) {
                y = feet + 4;
                above = false;
            } else {
                while y > 2 && !clear(y, &placed) {
                    y -= 1;
                }
                y = y.max(2);
            }
        }
        placed.push(BubbleBox { x, y, w, h, cx, above, lines, icon, mark, warn: b.warn, born: b.born, rank: if i == v.c { 2 } else { 1 } });
    }
    placed
}

fn draw_bubble(fb: &mut Framebuffer, sp: &Sprites, b: &BubbleBox) {
    let edge = if b.warn { TEXT_WARN } else { OUTLINE };
    fb.rect_fill(b.x, b.y, b.w, b.h, edge);
    fb.rect_fill(b.x + 1, b.y + 1, b.w - 2, b.h - 2, BUBBLE);
    // Three-row tail pointing at the speaker.
    let (base, dir) = if b.above { (b.y + b.h - 1, 1) } else { (b.y, -1) };
    let cx = b.cx.clamp(b.x + 4, b.x + b.w - 5);
    fb.rect_fill(cx - 2, base, 5, 1, BUBBLE);
    fb.rect_fill(cx - 1, base + dir, 3, 1, BUBBLE);
    for (dx, dy) in [(-3, 0), (3, 0), (-2, 1), (2, 1), (-1, 2), (0, 2), (1, 2)] {
        fb.pset(cx + dx, base + dy * dir, edge);
    }
    let mut tx = b.x + 4;
    if let Some(icon) = b.icon.as_deref().and_then(|id| sp.icon(id)) {
        fb.blit(icon, b.x + 3, b.y + 2, false);
        tx += 9;
    }
    if let Some(mark) = &b.mark {
        draw_icon(fb, sp, mark, tx - 1, b.y + 3);
    }
    for (k, line) in b.lines.iter().enumerate() {
        fb.text(line, tx, b.y + 3 + k as i32 * 9, INK, None, 1);
    }
}

fn draw_hud(g: &Game, sp: &Sprites, fb: &mut Framebuffer, v: &View) {
    fb.rect_bevel(0, VIEW_H, 400, 300 - VIEW_H, HUD_FILL, HUD_HI, HUD_SHADOW);
    let me = &g.communes[v.k];
    let done = me.tutorial.stage >= tutorial::DONE;
    let objective = STAGES[me.tutorial.stage.min(tutorial::DONE)];
    for (k, line) in wrap(objective, 64).iter().enumerate() {
        fb.text(line, 6, VIEW_H + 5 + k as i32 * 10, if done { TEXT_GOOD } else { TEXT }, Some(HUD_SHADOW), 1);
    }
    fb.rect_fill(4, VIEW_H + 26, 392, 1, HUD_SHADOW);
    fb.rect_fill(4, VIEW_H + 27, 392, 1, HUD_MID);
    let (status, warn) = g.status(v.c);
    let renown = format!("RENOWN {}", me.renown.floor());
    let renown_x = 394 - Framebuffer::text_width(&renown, 1);
    // The status gets the rest of the line, clear of the renown: without the monster's kind if
    // that isn't enough, then cut short (its bubble says it all).
    let kind = format!(" THE {}", g.monsters[v.c].kind.to_uppercase());
    let status = fit_line(&status, &kind, ((renown_x - 12 - 6) / 6).max(0) as usize);
    fb.text(&status, 6, VIEW_H + 32, if warn { TEXT_WARN } else { TEXT_DIM }, Some(HUD_SHADOW), 1);
    fb.text(&renown, renown_x, VIEW_H + 32, SPARK + 2, Some(HUD_SHADOW), 1);
    let _ = sp;
}

/// `text` in at most `room` characters: without `drop` (once) if it's too long, then cut short
/// with "..." if it still is.
fn fit_line(text: &str, drop: &str, room: usize) -> String {
    let mut text = text.to_string();
    if text.chars().count() > room {
        text = text.replacen(drop, "", 1);
    }
    if text.chars().count() > room {
        text = text.chars().take(room.saturating_sub(3)).collect::<String>() + "...";
    }
    text
}

#[cfg(test)]
#[test]
fn a_long_status_fits_beside_the_renown() {
    let status = "MOGG THE GOBLIN: WAITING FOR COTTON BOLLS AT THE COTTON BOLLS BIN'S CHUTE.";
    assert_eq!(fit_line("MOGG THE GOBLIN: IDLE", " THE GOBLIN", 50), "MOGG THE GOBLIN: IDLE", "short ones stay whole");
    assert_eq!(fit_line(status, " THE GOBLIN", 64), "MOGG: WAITING FOR COTTON BOLLS AT THE COTTON BOLLS BIN'S CHUTE.");
    let cut = fit_line(status, " THE GOBLIN", 50);
    assert!(cut.len() == 50 && cut.starts_with("MOGG: WAITING") && cut.ends_with("..."), "{cut}");
}

#[cfg(test)]
#[test]
fn every_good_a_workshop_makes_or_takes_has_an_icon_for_its_ghosts_and_cards() {
    let (g, sp) = (Game::new(), Sprites::new());
    let mut missing = Vec::new();
    for kind in crate::workshops::ShopKind::all() {
        for &r in kind.recipes() {
            if sp.icon(g.data.good(r).output()).is_none() {
                missing.push(r.to_string());
            }
            for (_, group) in g.data.good(r).input_groups() {
                if ghost_good(&g, group).is_none_or(|id| sp.icon(&id).is_none()) {
                    missing.push(format!("{r}'s {group:?}"));
                }
            }
        }
        // And the tools a town one lends, ghosted on its pegs.
        for &t in kind.builtin_tools() {
            if sp.icon(t).is_none() {
                missing.push(format!("{}'s {t}", kind.id()));
            }
        }
    }
    assert!(missing.is_empty(), "no icon: {missing:?}");
}

#[cfg(test)]
#[test]
fn a_benchs_mats_show_what_its_pads_are_working_toward() {
    let mut g = Game::new();
    let bench = g.world.shops.iter().position(|s| s.kind.label() == "BUILDERS' BENCH").expect("the builders' bench");
    let (pads, out) = (g.world.shops[bench].pads.clone(), g.world.shops[bench].outs[0]);
    let hint = |g: &Game, (x, y): (i32, i32), tile| mat_hint(g, x, y, tile);
    assert_eq!(hint(&g, out, Tile::OutMat).as_deref(), Some("kiln_kit"), "resting on what fills its pads");
    // Standing at it doesn't turn the ghosts through its recipes: they show what the card says.
    let c = g.communes[0].controlled;
    (g.monsters[c].x, g.monsters[c].y) = g.world.shops[bench].spot;
    g.frame = 30;
    assert_eq!(hint(&g, out, Tile::OutMat).as_deref(), Some("kiln_kit"), "still");
    let cloth = crate::econ::ItemData { good: "fabric".into(), quality: 0.5, carried_tags: Vec::new(), raw: Vec::new(), depth: 0, value: 1.0, shares: Vec::new(), wear: 0.0, haul: Vec::new(), carted: None };
    g.world.items.push(crate::world::Item::new(pads[0].0, pads[0].1, cloth));
    assert_eq!(hint(&g, out, Tile::OutMat).as_deref(), Some("loom_house_kit"), "cloth goes into a loom house kit");
    assert_eq!(hint(&g, pads[0], Tile::Pad), None, "a pad with something on it has no ghost");
    assert_eq!(hint(&g, pads[1], Tile::Pad).as_deref(), Some("planks"), "the empty pads show what's still missing, in order");
    let rest: Vec<Option<String>> = pads[2..].iter().map(|&p| hint(&g, p, Tile::Pad)).collect();
    assert_eq!(rest[0].as_deref(), Some("planks"));
    assert!(rest[1..].iter().all(|h| h.is_some()) && !rest.iter().any(|h| h.as_deref() == Some("fabric")), "{rest:?}");
}

/// Facing a kit: the workshop it would set up, as a ghost on the ground (a checkerboard, so
/// what's under it still shows), red on the tiles where something's in the way.
fn draw_kit_ghost(g: &Game, fb: &mut Framebuffer, v: &View) {
    let c = v.c;
    if g.monsters[c].slide.is_some() || g.monsters[c].playing {
        return;
    }
    let Some(fp) = g.kit_ahead(c).and_then(|k| g.footprint(c, k)) else { return };
    for t in fp.tiles().filter(|&(x, y)| v.sees(x, y)) {
        let col = if fp.blocked.contains(&t) { TEXT_WARN } else { TEXT_GOOD };
        let (x0, y0) = v.px(t.0, t.1);
        for y in 0..TS {
            for x in 0..TS {
                let edge = x == 0 || y == 0 || x == TS - 1 || y == TS - 1;
                if edge || (x + y) % 4 == 0 {
                    fb.pset(x0 + x, y0 + y, col);
                }
            }
        }
    }
}

/// The title screen: start a new world or pick a saved one. `deleting`: the picked world is
/// one Del away from being deleted.
pub fn draw_title(fb: &mut Framebuffer, worlds: &[String], sel: usize, error: Option<&str>, deleting: bool) {
    fb.rect_fill(0, 0, 400, 300, HEDGE);
    fb.rect_checker(0, 0, 400, 300, GRASS);
    fb.text_centered("MONSTER WORKSHOP", 200, 40, SPARK + 2, Some(OUTLINE), 3);
    let items: Vec<String> = std::iter::once("NEW WORLD".to_string()).chain(worlds.iter().cloned()).collect();
    let h = items.len().min(10) as i32 * 14 + 16;
    let (w, x, y) = (200, 100, 100);
    fb.rect_bevel(x, y, w, h, HUD_FILL, HUD_HI, HUD_SHADOW);
    let first = sel.saturating_sub(9);
    for (k, name) in items.iter().enumerate().skip(first).take(10) {
        let ly = y + 9 + (k - first) as i32 * 14;
        if k == sel {
            fb.rect_fill(x + 4, ly - 3, w - 8, 13, HUD_MID);
            for (k, h) in [1, 3, 5, 3, 1].iter().enumerate() {
                fb.rect_fill(x + 10 + k as i32, ly + 3 - h / 2, 1, *h, SPARK + 2);
            }
        }
        let col = if k == sel && deleting { TEXT_WARN } else if k == sel { TEXT } else { TEXT_DIM };
        fb.text(name, x + 22, ly, col, Some(HUD_SHADOW), 1);
    }
    if let Some(e) = error {
        for (k, l) in wrap(e, 60).iter().enumerate() {
            fb.text_centered(l, 200, y + h + 10 + k as i32 * 10, TEXT_WARN, Some(OUTLINE), 1);
        }
    }
    fb.text_centered("ARROWS PICK  SPACE STARTS  DEL DELETES  ESC QUITS", 200, 282, TEXT, Some(OUTLINE), 1);
}

/// Esc with nothing open: whether to leave the world.
fn draw_leaving(fb: &mut Framebuffer) {
    let lines = ["ESC AGAIN: TO THE TITLE SCREEN", "ANY OTHER KEY: STAY"];
    let w = lines.iter().map(|l| Framebuffer::text_width(l, 1)).max().unwrap_or(0) + 32;
    let h = lines.len() as i32 * 11 + 28;
    let (x, y) = ((400 - w) / 2, (VIEW_H - h) / 2);
    fb.rect_checker(0, 0, 400, VIEW_H, OUTLINE);
    fb.rect_bevel(x, y, w, h, HUD_FILL, HUD_HI, HUD_SHADOW);
    fb.text_centered("LEAVE THE WORLD?", x + w / 2, y + 7, SPARK + 2, Some(HUD_SHADOW), 1);
    for (k, line) in lines.iter().enumerate() {
        fb.text_centered(line, x + w / 2, y + 20 + k as i32 * 11, TEXT, Some(HUD_SHADOW), 1);
    }
}

fn draw_help(fb: &mut Framebuffer) {
    let w = HELP.iter().map(|l| Framebuffer::text_width(l, 1)).max().unwrap_or(0) + 24;
    let h = HELP.len() as i32 * 11 + 34;
    let (x, y) = ((400 - w) / 2, (VIEW_H - h) / 2);
    fb.rect_checker(0, 0, 400, VIEW_H, OUTLINE);
    fb.rect_bevel(x, y, w, h, HUD_FILL, HUD_HI, HUD_SHADOW);
    fb.text_centered("CONTROLS", x + w / 2, y + 7, SPARK + 2, Some(HUD_SHADOW), 1);
    for (k, line) in HELP.iter().enumerate() {
        fb.text(line, x + 12, y + 20 + k as i32 * 11, TEXT, Some(HUD_SHADOW), 1);
    }
    fb.text_centered("F1 CLOSES", x + w / 2, y + h - 11, TEXT_DIM, Some(HUD_SHADOW), 1);
}

#[cfg(test)]
#[test]
fn a_stand_sits_on_the_ground_around_it() {
    let g = Game::new();
    // Planks in the outfitter's and the temple, where you stand on planks to face it; grass
    // outdoors, in a yard, by the seam or beside a path; gravel in the Copper Cut.
    for r in &g.world.racks {
        let fronts: Vec<Tile> = crate::world::DIRS.iter().map(|d| (r.x + d.delta().0, r.y + d.delta().1)).filter(|&(x, y)| g.world.tile(x, y).walkable()).map(|(x, y)| ground(&g, x, y)).collect();
        let on = ground(&g, r.x, r.y);
        assert!(matches!(on, Tile::Floor | Tile::Grass | Tile::Gravel) && (on == Tile::Floor) == fronts.contains(&Tile::Floor), "{} at ({}, {}) on {on:?}, beside {fronts:?}", r.piece, r.x, r.y);
    }
    let on = |piece: &str| g.world.racks.iter().find(|r| r.piece == piece).map(|r| ground(&g, r.x, r.y));
    assert_eq!(on("secret_wagon_wright"), Some(Tile::Grass));
    assert_eq!(on("feathered_cap"), Some(Tile::Grass));
    assert_eq!(on("teaching_music"), Some(Tile::Floor));
    assert_eq!(on("secret_prospecting"), Some(Tile::Grass), "Old Soot's, by the coal seam");
    assert_eq!(on("rope_belt"), Some(Tile::Floor));
}

#[cfg(test)]
#[test]
fn newer_bubbles_are_drawn_over_older_ones() {
    use crate::world::Dir;
    let mut g = Game::new();
    let r = g.world.racks.iter().position(|r| r.piece == "towel").expect("the towel's rack");
    let (x, y) = (g.world.racks[r].x, g.world.racks[r].y);
    (g.monsters[0].x, g.monsters[0].y, g.monsters[0].facing) = (x + 1, y, Dir::Left);
    g.communes[0].renown = 0.0;
    for _ in 0..5 {
        g.update(&[]);
    }
    // Facing the towel's rack shows its bubble; Space there, too poor, says why: newer, on top.
    g.script("u");
    let top = |g: &Game| bubbles(g, &View::of(g, 0)).last().map(|b| b.lines.join(" ")).unwrap_or_default();
    assert!(bubbles(&g, &View::of(&g, 0)).len() >= 2);
    assert!(top(&g).contains("NOT ENOUGH RENOWN"), "{}", top(&g));
    // Said first, then turning to face another rack: that rack's bubble is newer, on top.
    g.say(0, "HELLO", false);
    for _ in 0..3 {
        g.update(&[]);
    }
    g.monsters[0].facing = Dir::Right;
    g.update(&[]);
    assert!(top(&g).contains("HASTY APRON"), "{}", top(&g));
}
