use crate::econ::{quality_level, EXCEPTIONAL, MASTERWORK};
use crate::inspect::{Icon, Line, Meter};
use crate::framebuffer::Framebuffer;
use crate::game::{Game, Species};
use crate::inspect;
use crate::palette::*;
use crate::sprites::{Sprite, Sprites};
use crate::tutorial::{self, wrap, HELP, STAGES};
use crate::world::*;

pub const VIEW_H: i32 = SCREEN_H * TS; // 256; the HUD fills the remaining 44 rows
const HOP: [i32; 7] = [0, 2, 4, 5, 5, 4, 2];

fn hash(x: i32, y: i32) -> u32 {
    let mut h = (x as u32).wrapping_mul(0x8da6_b343) ^ (y as u32).wrapping_mul(0xd816_3841);
    h ^= h >> 13;
    h = h.wrapping_mul(0x85eb_ca6b);
    h ^ (h >> 16)
}

/// Flip-screen camera: the top-left tile of the screen the controlled monster stands on.
pub fn screen_of(x: i32, y: i32) -> (i32, i32) {
    (x.div_euclid(SCREEN_W) * SCREEN_W, y.div_euclid(SCREEN_H) * SCREEN_H)
}

struct View {
    tx: i32,
    ty: i32,
}

impl View {
    fn of(g: &Game) -> Self {
        let m = &g.monsters[g.controlled];
        let (tx, ty) = screen_of(m.x, m.y);
        Self { tx, ty }
    }

    fn sees(&self, x: i32, y: i32) -> bool {
        x >= self.tx && y >= self.ty && x < self.tx + SCREEN_W && y < self.ty + SCREEN_H
    }

    /// Screen pixel of a tile's top-left corner.
    fn px(&self, x: i32, y: i32) -> (i32, i32) {
        ((x - self.tx) * TS, (y - self.ty) * TS)
    }
}

pub fn draw(g: &Game, sp: &Sprites, fb: &mut Framebuffer) {
    let v = View::of(g);
    fb.rect_fill(0, 0, 400, VIEW_H, OUTLINE);
    for ty in v.ty..v.ty + SCREEN_H {
        for tx in v.tx..v.tx + SCREEN_W {
            draw_tile(g, sp, fb, &v, tx, ty);
        }
    }
    draw_trees(g, sp, fb, &v);
    draw_building_labels(g, fb, &v);
    for p in g.world.plants.iter().filter(|p| v.sees(p.x, p.y)) {
        let (x, y) = v.px(p.x, p.y);
        let s = sp.plant(&p.good, p.stage());
        fb.blit(s, x + (TS - s.w) / 2, y + TS - s.h, false);
    }
    draw_ferry(g, sp, fb, &v);
    draw_boards_and_signs(g, sp, fb, &v);
    for s in g.world.shops.iter().filter(|s| v.sees(s.spot.0, s.spot.1)) {
        let (x, y) = v.px(s.spot.0, s.spot.1);
        match s.kind {
            ShopKind::Spinnery => fb.blit(&sp.wheel[if s.job.is_some() { (g.frame / 2 % 2) as usize } else { 0 }], x, y - 5, false),
            ShopKind::LoomHouse => fb.blit(&sp.loom, x, y - 5, false),
            ShopKind::Workbench => fb.blit(&sp.bench, x, y + TS - sp.bench.h - 4, false),
            ShopKind::Kiln => fb.blit(&sp.kiln[if s.job.is_some() { (g.frame / 3 % 2) as usize } else { 0 }], x, y - 5, false),
            ShopKind::IndigoHouse => fb.blit(&sp.indigo_vat, x, y - 4, false),
            ShopKind::DyeHouse => fb.blit(&sp.dye_vat, x, y - 4, false),
        }
    }
    for b in g.world.bins.iter().filter(|b| v.sees(b.x, b.y)) {
        draw_bin(sp, fb, &v, b);
    }
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
    draw_particles(g, fb, &v);
    draw_markers(g, sp, fb, &v);
    draw_edge_markers(g, fb, &v);
    draw_popups(g, fb, &v);
    // Lowest priority first, so the controlled monster's bubble ends up on top.
    for b in layout_bubbles(g, &v).iter().rev() {
        draw_bubble(fb, sp, b);
    }
    draw_progress(g, fb, &v);
    draw_applicant_pitch(g, sp, fb, &v);
    draw_floaters(g, fb, &v);
    draw_banner(g, fb);
    if g.inspecting && !g.help {
        draw_card(g, sp, fb, &v);
    }
    if g.roster && !g.help {
        draw_roster(g, sp, fb);
    }
    draw_hud(g, sp, fb);
    if g.help {
        draw_help(fb);
    }
}

fn draw_tile(g: &Game, sp: &Sprites, fb: &mut Framebuffer, v: &View, tx: i32, ty: i32) {
    let (x0, y0) = v.px(tx, ty);
    let (wx, wy) = (tx * TS, ty * TS);
    let tile = g.world.tile(tx, ty);
    let ground = match tile {
        Tile::Bin => g.world.tile(tx, ty + 1),
        Tile::Tree | Tile::Sign => Tile::Grass,
        t => t,
    };
    for y in 0..TS {
        for x in 0..TS {
            let h = hash(wx + x, wy + y);
            let c = match ground {
                Tile::Grass | Tile::Bin | Tile::Tree | Tile::Sign => grass(h),
                Tile::Water => {
                    // Slow ripples drifting downstream; a lighter lip where the bank begins.
                    let drift = (y + (g.frame / 10) as i32) % TS;
                    if y == 0 && g.world.tile(tx, ty - 1) != Tile::Water {
                        WATER + 2
                    } else if (x + drift * 5 + (h % 3) as i32) % 17 == 0 && drift % 4 == 1 {
                        WATER + 2
                    } else if (x * 3 + drift * 7) % 31 == 0 {
                        WATER
                    } else {
                        WATER + 1
                    }
                }
                Tile::Peg | Tile::Pad | Tile::OutMat | Tile::Spot if !indoors(g, tx, ty) => match h % 23 {
                    0 => GRASS + 3,
                    1 | 2 => GRASS + 2,
                    3 => GRASS,
                    _ => GRASS + 1,
                },
                Tile::Path => match h % 17 {
                    0 => PATH + 2,
                    1 | 2 => PATH,
                    _ => PATH + 1,
                },
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
                Tile::Wall => {
                    if y == 0 {
                        WOOD + 3
                    } else if y == TS - 1 {
                        WOOD
                    } else if (x + if ty % 2 == 0 { 0 } else { 4 }) % 8 == 0 {
                        WOOD
                    } else {
                        WOOD + 1 + (h % 9 == 0) as u8
                    }
                }
                _ => {
                    // Planks: a seam every 4 rows with staggered joints.
                    let row = y / 4;
                    if y % 4 == 3 || (x + row * 5 + tx * 3) % 16 == 0 {
                        PLANK
                    } else if y % 4 == 0 {
                        PLANK + 2
                    } else {
                        PLANK + 1 + (h % 13 == 0) as u8
                    }
                }
            };
            fb.pset(x0 + x, y0 + y, c);
        }
    }
    if ground == Tile::Grass && tile == Tile::Grass && hash(tx, ty) % 7 == 0 && g.world.plant_at(tx, ty).is_none() {
        let (fx, fy) = (x0 + 3 + (hash(ty, tx) % 9) as i32, y0 + 4 + (hash(tx + 7, ty) % 8) as i32);
        let col = FLOWER + (hash(tx, ty + 3) % 2) as u8;
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
    if tile == Tile::Door {
        if g.world.tile(tx, ty - 1) == Tile::Wall || g.world.tile(tx, ty + 1) == Tile::Wall {
            fb.rect_fill(x0 + 6, y0, 4, TS, PLANK);
            fb.rect_fill(x0 + 7, y0, 2, TS, PLANK + 2);
        } else {
            fb.rect_fill(x0, y0 + 6, TS, 4, PLANK);
            fb.rect_fill(x0, y0 + 7, TS, 2, PLANK + 2);
        }
    }
    if tile == Tile::Spot {
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
        for y in 2..TS - 2 {
            for x in 2..TS - 2 {
                fb.pset(x0 + x, y0 + y, if ((x / 2) + (y / 2)) % 2 == 0 { MAT + 1 } else { MAT });
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

fn grass(h: u32) -> u8 {
    match h % 23 {
        0 => GRASS + 3,
        1 | 2 => GRASS + 2,
        3 => GRASS,
        _ => GRASS + 1,
    }
}

/// Inside a building: walls or doors in all four directions within a few tiles. Mats there
/// sit on planks; outdoor mats sit on grass.
fn indoors(g: &Game, tx: i32, ty: i32) -> bool {
    let bounded = |dx: i32, dy: i32| (1..10).any(|k| matches!(g.world.tile(tx + dx * k, ty + dy * k), Tile::Wall | Tile::Door));
    bounded(-1, 0) && bounded(1, 0) && bounded(0, -1) && bounded(0, 1)
}

/// What belongs on a pad (that pad's input, in recipe order) or an output mat (the output).
/// A good the world grows or makes nearby is shown before a far-off one.
fn mat_hint(g: &Game, tx: i32, ty: i32, tile: Tile) -> Option<String> {
    let shop = g.world.shops.iter().find(|s| s.pads.contains(&(tx, ty)) || s.outs.contains(&(tx, ty)))?;
    let recipe = shop.default_recipe(&g.data);
    if tile == Tile::OutMat {
        return Some(recipe.to_string());
    }
    let k = shop.pads.iter().position(|&p| p == (tx, ty))?;
    let group = g.data.good(recipe).input_groups().nth(k).or_else(|| g.data.good(recipe).input_groups().next())?.1;
    let local = |id: &str| g.world.plants.iter().any(|p| p.good == id) || g.world.shops.iter().any(|s| s.kind.recipes().contains(&id));
    let fits: Vec<&crate::data::GoodType> = g.data.goods.iter().filter(|p| crate::data::matches(&p.type_tags, group)).collect();
    fits.iter().find(|p| local(&p.id)).or(fits.first()).map(|p| p.id.clone())
}

/// The built-in tool a peg stands in for.
fn peg_hint(g: &Game, tx: i32, ty: i32) -> Option<&'static str> {
    let shop = g.world.shops.iter().find(|s| s.pegs.contains(&(tx, ty)))?;
    let k = shop.pegs.iter().position(|&p| p == (tx, ty))?;
    shop.kind.builtin_tools().get(k).copied()
}

/// Facing an applicant: who they are, their talents and what they cost.
fn draw_applicant_pitch(g: &Game, sp: &Sprites, fb: &mut Framebuffer, v: &View) {
    let Some(k) = g.faced_applicant() else { return };
    let Some(a) = &g.applicants[k] else { return };
    let (sx, sy) = g.world.stands[k];
    if !v.sees(sx, sy) {
        return;
    }
    let talents = a.transferable.iter().map(|t| t.replace('_', " ").to_uppercase()).collect::<Vec<_>>().join(", ");
    let mut lines = vec![format!("{} THE {}", a.name, a.kind.to_uppercase()), format!("JOINS AT {} RENOWN", a.cost)];
    lines.extend(wrap(&format!("TALENTS: {talents}"), 24));
    let w = lines.iter().map(|l| l.len() as i32).max().unwrap_or(0) * 6 + 6;
    let h = lines.len() as i32 * 9 + 4;
    let (px, py) = v.px(sx, sy);
    let cx = px + 8;
    let b = BubbleBox { x: (cx - w / 2).clamp(2, 398 - w), y: (py + TS + 6).min(VIEW_H - h - 2), w, h, cx, above: false, lines, icon: None, warn: g.renown < a.cost as f64 };
    draw_bubble(fb, sp, &b);
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
/// A dyed good takes its dye's color: its cotton-white pixels move to the dye's ramp.
fn dye_ramp(item: &crate::econ::ItemData) -> Option<u8> {
    item.carried_tags.iter().any(|t| t == "indigo").then_some(WATER)
}

fn blit_quality(fb: &mut Framebuffer, s: &Sprite, x: i32, y: i32, level: i32, dye: Option<u8>) {
    for sy in 0..s.h {
        for sx in 0..s.w {
            let mut c = s.px[(sy * s.w + sx) as usize];
            if c == 0 {
                continue;
            }
            if let Some(ramp) = dye.filter(|_| (COTTON..COTTON + 3).contains(&c)) {
                c = ramp + (c - COTTON);
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
    let card = inspect::card(g);
    let page = g.card_page % card.pages.len();
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

/// Every other pixel of a sprite: a dithered "this goes here" hint, no alpha.
fn blit_ghost(fb: &mut Framebuffer, s: &Sprite, x: i32, y: i32) {
    for sy in 0..s.h {
        for sx in 0..s.w {
            let c = s.px[(sy * s.w + sx) as usize];
            if c > 1 && (x + sx + y + sy) % 2 == 0 {
                fb.pset(x + sx, y + sy, MAT + 2);
            }
        }
    }
}

fn draw_building_labels(g: &Game, fb: &mut Framebuffer, v: &View) {
    let mut named: Vec<((i32, i32), &str)> = Vec::new();
    for s in g.world.shops.iter().filter(|s| s.kind.is_building()) {
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
        let (x, y) = v.px(l, wy);
        let w = (r - l + 1) * TS;
        fb.text_centered(label, x + w / 2, y + 5, WOOD + 3, Some(OUTLINE), 1);
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

/// The ferry's rope across the river (once it is strung) and the boat on it.
fn draw_ferry(g: &Game, sp: &Sprites, fb: &mut Framebuffer, v: &View) {
    let Some(f) = &g.world.ferry else { return };
    if !(v.sees(f.west - 1, f.y) || v.sees(f.east + 2, f.y)) {
        return;
    }
    let (x0, y) = v.px(f.west - 1, f.y);
    let (x1, _) = v.px(f.east + 2, f.y);
    for x in [x0 + 11, x1 + 3] {
        fb.rect_fill(x, y - 3, 3, 12, OUTLINE);
        fb.rect_fill(x + 1, y - 2, 1, 10, WOOD + 2);
    }
    if f.rigged {
        fb.rect_fill(x0 + 13, y - 1, x1 - x0 - 10, 1, WOOD + 3);
        fb.rect_fill(x0 + 13, y, x1 - x0 - 10, 1, OUTLINE);
    }
    let (bx, by) = v.px(f.x, f.y);
    let ox = Slide::offset(f.slide).0;
    let bob = if f.moving && g.frame / 6 % 2 == 0 { 1 } else { 0 };
    fb.blit(&sp.boat, bx + ox, by + TS - 6 + bob, false);
}

/// Project boards with their progress, and signposts with where they point.
fn draw_boards_and_signs(g: &Game, sp: &Sprites, fb: &mut Framebuffer, v: &View) {
    for b in g.world.boards.iter().filter(|b| v.sees(b.x, b.y)) {
        let (x, y) = v.px(b.x, b.y);
        let y = y - (b.bounce > 1) as i32;
        fb.blit(&sp.board, x, y, false);
        let (text, col) = if b.done() { ("DONE".to_string(), TEXT_GOOD) } else { (format!("{}/{}", b.have, b.need), TEXT) };
        fb.text_centered(&text, x + 9, y - 8, col, Some(OUTLINE), 1);
    }
    for s in g.world.signs.iter().filter(|s| v.sees(s.x, s.y)) {
        let (x, y) = v.px(s.x, s.y);
        fb.blit(&sp.signpost, x + 1, y, false);
        let w = Framebuffer::text_width(s.text, 1);
        let lx = (x + 8 - w / 2).clamp(6, 394 - w);
        fb.text(s.text, lx, y - 8, TEXT, Some(OUTLINE), 1);
        // A small arrow beside the name, pointing the way.
        let (ax, dir) = if s.dir == Dir::Left { (lx - 5, -1) } else { (lx + w + 2, 1) };
        for (k, h) in [5, 3, 1].iter().enumerate() {
            let col = ax + if dir > 0 { k as i32 } else { 2 - k as i32 };
            fb.rect_fill(col, y - 8 + 3 - h / 2, 1, *h, SPARK + 2);
        }
    }
}

fn draw_bin(sp: &Sprites, fb: &mut Framebuffer, v: &View, b: &Bin) {
    let (x0, y0) = v.px(b.x, b.y);
    let y0 = y0 - (b.bounce > 1) as i32;
    fb.rect_bevel(x0 + 1, y0 + 2, 14, 14, WOOD + 1, WOOD + 3, WOOD);
    for y in [y0 + 6, y0 + 10] {
        fb.rect_fill(x0 + 2, y, 12, 1, WOOD);
    }
    if let Some(icon) = sp.icon(&b.accepts[0]) {
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
    for m in &g.monsters {
        let flag = if m.species == Species::Goblin { &sp.flag_goblin } else { &sp.flag_cyclops };
        if let Some((fx, fy)) = m.rec.as_ref().or(m.routine.as_ref()).map(|r| r.flag) {
            if v.sees(fx, fy) {
                let (x, y) = v.px(fx, fy);
                fb.blit(flag, x + 1, y + TS - flag.h, false);
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
                let s = sp.item(it.good());
                let (ox, oy) = Slide::offset(it.slide);
                let (tx, ty) = v.px(it.x, it.y);
                let jitter = if it.locked { (g.frame / 2 % 2) as i32 } else { 0 };
                let x = tx + ox + (TS - s.w) / 2 + jitter;
                let y = ty + TS - s.h + oy - HOP[it.hop.min(6) as usize];
                fb.rect_checker(tx + ox + 3, ty + TS + oy - 2, 10, 2, SHADOW);
                // Ungraded goods have no quality to show: no tint, no twinkle.
                let graded = g.data.try_good(it.good()).is_some_and(|gt| gt.has_quality);
                let q = it.data.quality;
                blit_quality(fb, s, x, y, if graded { quality_level(q) } else { 0 }, dye_ramp(&it.data));
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
            Body::Wagon(k) => {
                let w = &g.world.wagons[k];
                let s = &sp.wagon[w.open as usize];
                let (tx, ty) = v.px(w.at.0, w.at.1);
                let (ox, oy) = Slide::offset(w.slide);
                let (x, y) = (tx + ox + (TS - s.w) / 2, ty + oy + TS - s.h - (w.bounce > 1) as i32);
                fb.rect_checker(tx + ox + 2, ty + oy + TS - 2, 12, 2, SHADOW);
                // The tailgate is drawn on the left; face it away from the pumpkin.
                fb.blit(s, x, y, w.pumpkin.0 < w.at.0);
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
                fb.blit(s, x, y, flip);
            }
        }
    }
}

fn monster_pose<'a>(g: &Game, sp: &'a Sprites, v: &View, i: usize) -> (i32, i32, &'a Sprite, bool) {
    let m = &g.monsters[i];
    let frames = if m.species == Species::Goblin { &sp.goblin } else { &sp.cyclops };
    // Riders stand still while the ferry carries them.
    let riding = g.world.ferry.as_ref().is_some_and(|f| f.slide.is_some() && f.on_deck(m.x, m.y));
    let frame = if m.slide.is_some() && !riding {
        2 + m.walk_frame as usize
    } else if m.busy {
        (g.frame / 2 % 2) as usize
    } else {
        (g.frame / 10 % 2) as usize
    };
    let s = &frames[frame];
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
        if i == g.controlled {
            let bob = (g.frame / 8 % 2) as i32;
            fb.blit(&sp.marker, x + s.w / 2 - 3, top - 6 + bob, false);
        }
        let (bx, by) = (x + s.w - 3, top);
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
    for (i, m) in g.monsters.iter().enumerate().filter(|(_, m)| !v.sees(m.x, m.y)) {
        let (px, py) = v.px(m.x, m.y);
        let (mut x, mut y) = ((px + 8).clamp(9, 391), (py + 8).clamp(9, VIEW_H - 9));
        while placed.iter().any(|&(ox, oy)| (ox - x).abs() < 14 && (oy - y).abs() < 14) {
            if x == 9 || x == 391 {
                y = if y + 14 <= VIEW_H - 9 { y + 14 } else { y - 14 };
            } else {
                x = if x + 14 <= 391 { x + 14 } else { x - 14 };
            }
        }
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

/// Holding Tab: every monster, where it is and what it is doing.
fn draw_roster(g: &Game, sp: &Sprites, fb: &mut Framebuffer) {
    let n = g.monsters.len() as i32;
    let w = 96;
    let per_row = 4;
    let rows = (n + per_row - 1) / per_row;
    let h = rows * 26 + 6;
    let (x0, y0) = ((400 - per_row.min(n) * w - 6) / 2, VIEW_H - h - 4);
    fb.rect_bevel(x0, y0, per_row.min(n) * w + 6, h, HUD_FILL, HUD_HI, HUD_SHADOW);
    for (i, m) in g.monsters.iter().enumerate() {
        let (col, row) = (i as i32 % per_row, i as i32 / per_row);
        let (x, y) = (x0 + 3 + col * w, y0 + 3 + row * 26);
        if i == g.controlled {
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
fn draw_banner(g: &Game, fb: &mut Framebuffer) {
    let Some((name, t)) = g.banner else { return };
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
    warn: bool,
}

/// Places speech bubbles so they never cover each other: above the head if there is room,
/// otherwise below the feet, otherwise nudged upward until clear. Controlled monster first.
fn layout_bubbles(g: &Game, v: &View) -> Vec<BubbleBox> {
    let mut placed: Vec<BubbleBox> = Vec::new();
    let order = std::iter::once(g.controlled).chain((0..g.monsters.len()).filter(|&i| i != g.controlled));
    for i in order {
        let m = &g.monsters[i];
        let Some(b) = &m.bubble else { continue };
        if !v.sees(m.x, m.y) {
            continue;
        }
        // Monsters you are not steering only show what they wait for, not a sentence.
        let compact = b.waiting && i != g.controlled;
        let icon = if compact {
            g.data.goods.iter().find(|gd| m.wait_reason.contains(&gd.name.to_uppercase())).map(|gd| gd.id.clone())
        } else {
            None
        };
        let lines = match (compact, &icon) {
            (false, _) => wrap(&b.text, 24),
            (true, Some(_)) => vec!["?".to_string()],
            (true, None) => vec!["...".to_string()],
        };
        let icon_w = if icon.is_some() { 9 } else { 0 };
        let w = lines.iter().map(|l| l.len() as i32).max().unwrap_or(0) * 6 + 6 + icon_w;
        let h = lines.len() as i32 * 9 + 4;
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
        placed.push(BubbleBox { x, y, w, h, cx, above, lines, icon, warn: b.warn });
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
    for (k, line) in b.lines.iter().enumerate() {
        fb.text(line, tx, b.y + 3 + k as i32 * 9, INK, None, 1);
    }
}

fn draw_hud(g: &Game, sp: &Sprites, fb: &mut Framebuffer) {
    fb.rect_bevel(0, VIEW_H, 400, 300 - VIEW_H, HUD_FILL, HUD_HI, HUD_SHADOW);
    let done = g.tutorial.stage >= tutorial::DONE;
    let objective = STAGES[g.tutorial.stage.min(tutorial::DONE)];
    for (k, line) in wrap(objective, 64).iter().enumerate() {
        fb.text(line, 6, VIEW_H + 5 + k as i32 * 10, if done { TEXT_GOOD } else { TEXT }, Some(HUD_SHADOW), 1);
    }
    fb.rect_fill(4, VIEW_H + 26, 392, 1, HUD_SHADOW);
    fb.rect_fill(4, VIEW_H + 27, 392, 1, HUD_MID);
    let (status, warn) = g.status(g.controlled);
    fb.text(&status, 6, VIEW_H + 32, if warn { TEXT_WARN } else { TEXT_DIM }, Some(HUD_SHADOW), 1);
    let renown = format!("RENOWN {}", g.renown.floor());
    fb.text(&renown, 394 - Framebuffer::text_width(&renown, 1), VIEW_H + 32, SPARK + 2, Some(HUD_SHADOW), 1);
    let _ = sp;
}

/// The title screen: start a new world or pick a saved one.
pub fn draw_title(fb: &mut Framebuffer, worlds: &[String], sel: usize, error: Option<&str>) {
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
        fb.text(name, x + 22, ly, if k == sel { TEXT } else { TEXT_DIM }, Some(HUD_SHADOW), 1);
    }
    if let Some(e) = error {
        for (k, l) in wrap(e, 60).iter().enumerate() {
            fb.text_centered(l, 200, y + h + 10 + k as i32 * 10, TEXT_WARN, Some(OUTLINE), 1);
        }
    }
    fb.text_centered("ARROWS PICK  SPACE STARTS  ESC QUITS", 200, 282, TEXT, Some(OUTLINE), 1);
}

fn draw_help(fb: &mut Framebuffer) {
    let (w, h) = (252, HELP.len() as i32 * 11 + 34);
    let (x, y) = ((400 - w) / 2, (VIEW_H - h) / 2);
    fb.rect_checker(0, 0, 400, VIEW_H, OUTLINE);
    fb.rect_bevel(x, y, w, h, HUD_FILL, HUD_HI, HUD_SHADOW);
    fb.text_centered("CONTROLS", x + w / 2, y + 7, SPARK + 2, Some(HUD_SHADOW), 1);
    for (k, line) in HELP.iter().enumerate() {
        fb.text(line, x + 12, y + 20 + k as i32 * 11, TEXT, Some(HUD_SHADOW), 1);
    }
    fb.text_centered("F1 CLOSES", x + w / 2, y + h - 11, TEXT_DIM, Some(HUD_SHADOW), 1);
}
