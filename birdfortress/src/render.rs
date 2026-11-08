//! Everything drawn: the side-view world, the colony, the panel, overlays and menus.

use crate::framebuffer::{Framebuffer, Sprite};
use crate::game::*;
use crate::layout::NodeRef;
use crate::level::Tile;
use crate::palette::*;
use crate::path::{Move, Path};
use crate::rules::*;
use crate::sim::{CState, Outcome};
use crate::ui::{self, Rect};
use crate::world::World;

pub fn draw(g: &Game, fb: &mut Framebuffer) {
    match g.screen {
        Screen::Title => draw_title(g, fb),
        Screen::Select => draw_select(g, fb),
        Screen::Help => draw_help(g, fb),
        Screen::Play => draw_play(g, fb),
    }
}

fn hash(x: i32, y: i32) -> u32 {
    let mut h = (x as u32).wrapping_mul(374761393) ^ (y as u32).wrapping_mul(668265263);
    h = (h ^ (h >> 13)).wrapping_mul(1274126177);
    h ^ (h >> 16)
}

pub fn good_color(g: Good) -> u8 {
    match g {
        Good::Berry => BERRY,
        Good::Twig => WOOD,
        Good::Stone => ROCK,
        Good::Clay => CLAY,
        Good::Egg => EGG,
        Good::Brick => BRICK,
        Good::Wicker => WOOD_HI,
        Good::Pot => POT,
    }
}

pub fn species_colors(s: Species) -> (u8, u8) {
    match s {
        Species::Tribble => (TRIB_HI, TRIB_DK),
        Species::Dodo => (DODO_HI, DODO_DK),
        Species::Ape => (APE_FACE, APE),
    }
}

// ----- Sky and terrain --------------------------------------------------------------------------

pub fn draw_sky(fb: &mut Framebuffer, h: i32) {
    // Five bands with a 2x2 checker seam between each (dithering, no blending).
    let band = h / 5;
    for y in 0..h {
        let b = (y / band).min(4);
        let next = (b + 1).min(4);
        let seam = y % band >= band - 3 && b < 4;
        for x in 0..400 {
            let c = if seam && (x + y) % 2 == 0 { SKY + next as u8 } else { SKY + b as u8 };
            fb.pset(x, y, c);
        }
    }
    // Distant hills.
    for x in 0..400 {
        let a = ((x as f32 * 0.021).sin() * 14.0 + (x as f32 * 0.047 + 1.3).sin() * 7.0) as i32;
        let top = h - 46 + a;
        for y in top.max(0)..h {
            fb.pset(x, y, HILL + 1);
        }
        let b = ((x as f32 * 0.033 + 2.0).sin() * 10.0 + (x as f32 * 0.071).sin() * 5.0) as i32;
        for y in (h - 26 + b).max(0)..h {
            fb.pset(x, y, HILL);
        }
    }
}

fn solidish(w: &World, x: i32, y: i32) -> bool {
    w.inside(x, y) && w.tile(x, y).solid()
}

fn draw_tile(fb: &mut Framebuffer, w: &World, x: i32, y: i32) {
    let (px, py) = (x * TILE, y * TILE);
    let t = w.tile(x, y);
    let hsh = hash(x, y);
    match t {
        Tile::Earth | Tile::Rock | Tile::Bramble => {
            let (base, dk, hi) = match t {
                Tile::Earth => (SOIL, SOIL_DK, SOIL_HI),
                Tile::Rock => (ROCK, ROCK_DK, ROCK_HI),
                _ => (THORN_DK, OUTLINE, THORN),
            };
            fb.rect_fill(px, py, TILE, TILE, base);
            if t == Tile::Rock {
                // Strata: broken horizontal seams, offset per tile.
                for sy in [3, 7] {
                    let off = ((hsh >> sy) % 5) as i32;
                    for sx in 0..TILE {
                        if (sx + off) % 7 < 4 {
                            fb.pset(px + sx, py + sy, ROCK_DK);
                        }
                    }
                }
                fb.pset(px + (hsh % 8) as i32 + 1, py + 2, ROCK_HI);
                fb.pset(px + ((hsh >> 4) % 8) as i32 + 1, py + 6, ROCK_HI);
            }
            for k in 0..5 {
                let hh = hash(x * 7 + k, y * 13 - k);
                let (sx, sy) = ((hh % 9) as i32, ((hh >> 8) % 9) as i32);
                fb.pset(px + sx, py + sy, if k % 2 == 0 { dk } else { hi });
                if t == Tile::Rock && k == 0 {
                    fb.pset(px + sx + 1, py + sy, dk);
                }
                if t == Tile::Bramble {
                    fb.pset(px + sx, py + sy + 1, hi);
                    fb.pset(px + sx + 1, py + sy, hi);
                }
            }
            let open_above = !solidish(w, x, y - 1);
            if !solidish(w, x - 1, y) {
                fb.rect_fill(px, py, 1, TILE, dk);
            }
            if !solidish(w, x + 1, y) {
                fb.rect_fill(px + TILE - 1, py, 1, TILE, dk);
            }
            if !solidish(w, x, y + 1) && w.inside(x, y + 1) {
                fb.rect_fill(px, py + TILE - 1, TILE, 1, dk);
            }
            if open_above {
                match t {
                    Tile::Earth => {
                        fb.rect_fill(px, py, TILE, 3, GRASS);
                        fb.rect_fill(px, py, TILE, 1, GRASS_HI);
                        for k in 0..TILE {
                            if (hsh >> k) & 3 == 0 {
                                fb.pset(px + k, py + 3, GRASS_DK);
                            }
                            if (hsh >> (k + 3)) & 7 == 0 {
                                fb.pset(px + k, py - 1, GRASS);
                            }
                        }
                    }
                    Tile::Rock => fb.rect_fill(px, py, TILE, 1, ROCK_HI),
                    _ => {
                        for k in (1..TILE).step_by(3) {
                            fb.pset(px + k, py - 1, THORN);
                        }
                    }
                }
            }
        }
        Tile::Branch => {
            fb.rect_fill(px, py, TILE, 4, WOOD);
            fb.rect_fill(px, py, TILE, 1, WOOD_HI);
            fb.rect_fill(px, py + 3, TILE, 1, WOOD_DK);
            fb.rect_fill(px, py + 4, TILE, 1, OUTLINE);
            if hsh % 3 == 0 {
                fb.pset(px + (hsh % 7) as i32 + 1, py + 2, WOOD_DK);
            }
            let left = w.tile(x - 1, y) == Tile::Branch || solidish(w, x - 1, y) || w.tile(x - 1, y) == Tile::Trunk;
            let right = w.tile(x + 1, y) == Tile::Branch || solidish(w, x + 1, y) || w.tile(x + 1, y) == Tile::Trunk;
            if !left {
                fb.rect_fill(px, py, 1, 5, OUTLINE);
            }
            if !right {
                fb.rect_fill(px + TILE - 1, py, 1, 5, OUTLINE);
            }
        }
        Tile::Ledge => {
            fb.rect_fill(px, py, TILE, 2, WOOD_DK);
            fb.rect_fill(px, py, TILE, 1, WOOD);
            for k in [2, 6] {
                if (hsh >> k) & 1 == 0 {
                    fb.pset(px + k, py + 2, LEAF);
                    fb.pset(px + k + 1, py + 2, LEAF_HI);
                }
            }
        }
        Tile::Trunk => {
            fb.rect_fill(px + 1, py, TILE - 2, TILE, WOOD_DK);
            for k in 0..3 {
                let sx = px + 2 + ((hsh >> (k * 4)) % 6) as i32;
                fb.rect_fill(sx, py + (k * 3) as i32, 1, 3, WOOD);
            }
            fb.rect_fill(px, py, 1, TILE, OUTLINE);
            fb.rect_fill(px + TILE - 1, py, 1, TILE, OUTLINE);
        }
        Tile::Leaves => {
            for yy in 0..TILE {
                for xx in 0..TILE {
                    let c = if (xx + yy) % 2 == 0 { LEAF } else { LEAF_DK };
                    fb.pset(px + xx, py + yy, c);
                }
            }
            for k in 0..3 {
                let hh = hash(x * 3 + k, y * 5);
                fb.pset(px + (hh % 9) as i32, py + ((hh >> 4) % 9) as i32, LEAF_HI);
            }
        }
        Tile::Vine => {
            for yy in 0..TILE {
                let off = if (yy / 3) % 2 == 0 { 4 } else { 5 };
                fb.pset(px + off, py + yy, VINE);
            }
            fb.pset(px + 3, py + 2, LEAF_HI);
            fb.pset(px + 6, py + 7, LEAF_HI);
            fb.pset(px + 2, py + 2, LEAF);
            fb.pset(px + 7, py + 7, LEAF);
        }
        Tile::Water => {
            for yy in 0..TILE {
                for xx in 0..TILE {
                    let c = WATER + (((px + xx) / 3 + (py + yy) / 2) % 4) as u8;
                    fb.pset(px + xx, py + yy, c);
                }
            }
            if w.tile(x, y - 1) != Tile::Water {
                for xx in 0..TILE {
                    if (px + xx) % 5 != 0 {
                        fb.pset(px + xx, py, FOAM);
                    }
                }
            }
        }
        Tile::Ladder => {
            fb.rect_fill(px + 1, py, 1, TILE, WOOD_DK);
            fb.rect_fill(px + 2, py, 1, TILE, WOOD);
            fb.rect_fill(px + 7, py, 1, TILE, WOOD);
            fb.rect_fill(px + 8, py, 1, TILE, WOOD_DK);
            for ry in [2, 7] {
                fb.rect_fill(px + 3, py + ry, 4, 1, WOOD_HI);
            }
        }
        Tile::Cloud => {
            // A puffy bank: bright scalloped top, softer dithered underside.
            let left = w.tile(x - 1, y) == Tile::Cloud;
            let right = w.tile(x + 1, y) == Tile::Cloud;
            for yy in 0..6 {
                for xx in 0..TILE {
                    let bump = [1, 0, 0, 1, 2, 1, 0, 0, 1, 2][((xx + x * 3) % 10) as usize];
                    let edge = (!left && xx < 2 - yy.min(2)) || (!right && xx > TILE - 3 + yy.min(2));
                    if yy < bump || edge {
                        continue;
                    }
                    let c = if yy == bump { WHITE } else if yy < 4 { ACCENT_HI } else if (xx + yy) % 2 == 0 { ACCENT } else { 0 };
                    if c != 0 {
                        fb.pset(px + xx, py + yy, c);
                    }
                }
            }
        }
        Tile::Air => {}
    }
    if w.is_plank(x, y) {
        fb.rect_fill(px, py, TILE, 3, WOOD_HI);
        fb.rect_fill(px, py + 2, TILE, 1, WOOD);
        fb.rect_fill(px, py + 3, TILE, 1, OUTLINE);
        fb.pset(px + 1, py + 1, ROCK_DK);
        fb.pset(px + 8, py + 1, ROCK_DK);
    }
    if w.is_spring(x, y) {
        draw_spring(fb, px, py);
    }
}

pub fn draw_spring(fb: &mut Framebuffer, px: i32, py: i32) {
    fb.rect_fill(px + 1, py + 8, 8, 2, ROCK_DK);
    for (k, xx) in [(5, 3), (6, 5), (7, 3)] {
        fb.rect_fill(px + xx, py + k, 3, 1, SPRING);
    }
    fb.rect_fill(px + 1, py + 3, 8, 2, BANNER);
    fb.rect_fill(px + 1, py + 3, 8, 1, GOLD);
}

pub fn draw_terrain(fb: &mut Framebuffer, w: &World) {
    for y in 0..w.h {
        for x in 0..w.w {
            draw_tile(fb, w, x, y);
        }
    }
}

// ----- Nodes -------------------------------------------------------------------------------------

fn draw_source(fb: &mut Framebuffer, art: &crate::art::Art, good: Good, px: i32, py: i32, stock: i32) {
    match good {
        Good::Berry => {
            fb.rect_fill(px + 2, py + 3, 6, 6, LEAF);
            fb.rect_fill(px + 1, py + 5, 8, 4, LEAF);
            fb.rect_fill(px + 3, py + 2, 4, 2, LEAF_HI);
            fb.rect_fill(px + 1, py + 9, 8, 1, LEAF_DK);
            for (k, (bx, by)) in [(2, 5), (6, 4), (4, 7), (7, 7), (3, 3), (5, 5)].iter().enumerate() {
                if (k as i32) < stock {
                    fb.pset(px + bx, py + by, BERRY);
                    fb.pset(px + bx + 1, py + by, BERRY);
                }
            }
        }
        Good::Twig => {
            let n = stock.min(4);
            for k in 0..n.max(1) {
                let y = py + 9 - k * 2;
                let c = if k < n { WOOD } else { WOOD_DK };
                fb.line(px + 1, y, px + 8, y - 1, c);
                fb.pset(px + 1 + k * 2, y - 1, WOOD_HI);
            }
            if n == 0 {
                fb.pset(px + 4, py + 9, WOOD_DK);
            }
        }
        Good::Stone => {
            fb.rect_fill(px + 1, py + 3, 8, 7, ROCK);
            fb.rect_fill(px + 2, py + 1, 6, 3, ROCK);
            fb.rect_fill(px + 2, py + 1, 3, 1, ROCK_HI);
            fb.pset(px + 4, py + 5, ROCK_DK);
            fb.pset(px + 5, py + 6, ROCK_DK);
            fb.rect_fill(px + 1, py + 3, 1, 7, ROCK_DK);
            fb.line(px + 6, py, px + 9, py + 3, WOOD);
            fb.rect_fill(px + 7, py, 3, 1, ROCK_HI);
        }
        Good::Clay => {
            fb.rect_fill(px + 1, py + 5, 8, 5, CLAY);
            fb.rect_fill(px + 2, py + 3, 6, 2, CLAY);
            fb.rect_fill(px + 3, py + 3, 3, 1, POT);
            fb.rect_fill(px + 1, py + 9, 8, 1, BRICK);
        }
        Good::Egg => {
            fb.rect_fill(px, py + 7, 10, 3, WOOD);
            fb.rect_fill(px + 1, py + 7, 8, 1, WOOD_HI);
            fb.rect_fill(px, py + 9, 10, 1, WOOD_DK);
            for k in 0..stock.min(3) {
                let ex = px + 1 + k * 3;
                fb.rect_fill(ex, py + 4, 3, 4, EGG);
                fb.pset(ex, py + 4, 0);
                fb.pset(ex + 2, py + 4, 0);
                fb.pset(ex + 1, py + 5, WHITE);
            }
        }
        _ => {
            let s = art.good(good);
            fb.blit(s, px + (TILE - s.w) / 2, py + TILE - s.h, false);
        }
    }
}

fn draw_fortress(fb: &mut Framebuffer, g: &Game) {
    let lv = g.level();
    let (fx, fy, fw, fh) = lv.fortress;
    let (px, py, pw, ph) = (fx * TILE, fy * TILE, fw * TILE, fh * TILE);
    let total = lv.total_need().max(1);
    let got: i32 = match &g.sim {
        Some(s) => lv.need.iter().map(|(gd, n)| s.delivered(*gd).min(*n)).sum(),
        None => 0,
    };
    let done = got >= total;
    // Footing and scaffold posts.
    fb.rect_fill(px, py + ph - 2, pw, 2, ROCK_DK);
    let wall_h = ph - 4;
    let filled = wall_h * got / total;
    for y in 0..filled {
        let yy = py + ph - 2 - 1 - y;
        let row = y / 3;
        for x in 2..pw - 2 {
            let mortar = y % 3 == 2 || (x + if row % 2 == 0 { 0 } else { 3 }) % 6 == 0;
            fb.pset(px + x, yy, if mortar { ROCK_DK } else if y % 3 == 0 { ROCK_HI } else { ROCK });
        }
        fb.pset(px + 1, yy, OUTLINE);
        fb.pset(px + pw - 2, yy, OUTLINE);
    }
    if done {
        // Crenellations and the banner.
        let top = py + 2;
        for x in (2..pw - 2).step_by(4) {
            fb.rect_fill(px + x, top - 2, 2, 2, ROCK);
        }
        fb.rect_fill(px + pw / 2, top - 14, 1, 12, WOOD_DK);
        fb.rect_fill(px + pw / 2 + 1, top - 14, 7, 4, BANNER);
        fb.rect_fill(px + pw / 2 + 1, top - 14, 7, 1, GOLD);
    } else {
        for x in [0, pw - 1] {
            fb.rect_fill(px + x, py, 1, ph - 2, WOOD_DK);
        }
        for k in 0..fw {
            fb.rect_fill(px + k * TILE + TILE - 1, py + 2, 1, ph - 4, WOOD);
        }
        for y in (py + 2..py + ph - 2).step_by(8) {
            fb.rect_fill(px, y, pw, 1, WOOD);
        }
        fb.rect_fill(px, py, pw, 1, WOOD_HI);
    }
    // Needs, above.
    let mut x = px;
    let y = py - 10;
    for (gd, n) in &lv.need {
        let have = g.sim.as_ref().map(|s| s.delivered(*gd)).unwrap_or(0).min(*n);
        let spr = g.art.good(*gd);
        fb.blit(spr, x, y + (8 - spr.h) / 2, false);
        let t = format!("{have}/{n}");
        fb.text(&t, x + spr.w + 1, y, if have >= *n { GOOD } else { TEXT }, Some(OUTLINE), 1);
        x += spr.w + 3 + Framebuffer::text_width(&t, 1);
    }
}

fn hut(fb: &mut Framebuffer, px: i32, py: i32, wall: u8, wall_dk: u8, roof: u8, roof_hi: u8) {
    // Body.
    fb.rect_fill(px + 1, py + 7, 18, 13, wall);
    for x in (3..18).step_by(4) {
        fb.rect_fill(px + x, py + 8, 1, 12, wall_dk);
    }
    fb.rect_outline(px + 1, py + 7, 18, 13, OUTLINE);
    // Roof (stepped).
    for k in 0..6 {
        let inset = 5 - k;
        fb.rect_fill(px + inset, py + 1 + k, 20 - inset * 2, 1, if k == 0 { roof_hi } else { roof });
        fb.pset(px + inset, py + 1 + k, OUTLINE);
        fb.pset(px + 19 - inset, py + 1 + k, OUTLINE);
    }
    fb.rect_fill(px + 5, py, 10, 1, OUTLINE);
    fb.rect_fill(px, py + 7, 20, 1, OUTLINE);
    // Door.
    fb.rect_fill(px + 8, py + 13, 4, 7, PANEL_DEEP);
}

fn draw_building(fb: &mut Framebuffer, g: &Game, kind: Kind, px: i32, py: i32, node: Option<&crate::sim::NodeState>) {
    match kind {
        Kind::Depot => hut(fb, px, py, WOOD, WOOD_DK, WOOD_DK, WOOD_HI),
        Kind::Mason => hut(fb, px, py, ROCK, ROCK_DK, SOIL_DK, ROCK_HI),
        Kind::Weaver => hut(fb, px, py, WOOD_HI, WOOD, BEAK, GOLD),
        Kind::Kiln => {
            fb.rect_fill(px + 2, py + 6, 16, 14, BRICK);
            fb.rect_fill(px + 4, py + 3, 12, 3, BRICK);
            fb.rect_fill(px + 6, py + 1, 8, 2, BRICK);
            fb.rect_fill(px + 6, py + 1, 8, 1, POT);
            fb.rect_outline(px + 2, py + 6, 16, 14, OUTLINE);
            fb.rect_fill(px + 8, py + 13, 4, 7, PANEL_DEEP);
            if node.is_some_and(|n| n.craft_t > 0) {
                fb.rect_fill(px + 9, py + 16, 2, 3, GOLD);
                fb.pset(px + 9, py + 15, BAD);
            }
        }
    }
    // Sign with the product, for workshops.
    if let Some(r) = kind.recipe() {
        let s = g.art.good(r.output);
        fb.blit(s, px + 13, py + 9, false);
    }
    // Stock pips: what's inside.
    if let Some(n) = node {
        let mut pips: Vec<u8> = Vec::new();
        for gd in GOODS {
            for _ in 0..n.stock[gd.index()].min(6) {
                pips.push(good_color(gd));
            }
        }
        for (i, c) in pips.iter().take(6).enumerate() {
            let x = px + 2 + (i as i32 % 3) * 3;
            let y = py + 9 + (i as i32 / 3) * 3;
            fb.rect_fill(x, y, 2, 2, *c);
        }
        if let Some(r) = kind.recipe() {
            if n.craft_t > 0 {
                let w = 16 * (r.ticks - n.craft_t) / r.ticks;
                fb.rect_fill(px + 2, py + 8, 16, 1, PANEL_DEEP);
                fb.rect_fill(px + 2, py + 8, w, 1, GOLD);
            }
        }
    }
}

fn draw_nodes(fb: &mut Framebuffer, g: &Game) {
    let lv = g.level();
    for (i, s) in lv.sources.iter().enumerate() {
        let stock = match &g.sim {
            Some(sim) => sim.node(NodeRef::Source(i)).map(|n| if n.max <= 0 { 4 } else { n.stock[s.good.index()] }).unwrap_or(0),
            None => if s.infinite() { 4 } else { s.max },
        };
        draw_source(fb, &g.art, s.good, s.x * TILE, s.y * TILE, stock);
    }
    draw_fortress(fb, g);
    for b in &g.layout.buildings {
        let node = g.sim.as_ref().and_then(|s| s.node(NodeRef::Building(b.id)));
        draw_building(fb, g, b.kind, b.x * TILE, b.y * TILE, node);
    }
}

// ----- Routes and creatures ----------------------------------------------------------------------

fn path_points(p: &Path) -> Vec<(i32, i32)> {
    let mut v = vec![p.start];
    v.extend(p.steps.iter().map(|s| (s.x, s.y)));
    v
}

fn dotted(fb: &mut Framebuffer, pts: &[(i32, i32)], col: u8, dy: i32, every: i32, phase: i32) {
    let mut n = phase;
    for w in pts.windows(2) {
        let (x0, y0) = (w[0].0 * TILE + 5, w[0].1 * TILE + 5 + dy);
        let (x1, y1) = (w[1].0 * TILE + 5, w[1].1 * TILE + 5 + dy);
        let steps = (x1 - x0).abs().max((y1 - y0).abs()).max(1);
        for k in 0..steps {
            if n.rem_euclid(every) == 0 {
                let (x, y) = (x0 + (x1 - x0) * k / steps, y0 + (y1 - y0) * k / steps);
                fb.pset(x, y + 1, OUTLINE);
                fb.pset(x, y, col);
            }
            n += 1;
        }
    }
}

fn draw_routes(fb: &mut Framebuffer, g: &Game) {
    for (i, r) in g.layout.routes.iter().enumerate() {
        let Some(p) = g.plans.get(i) else { continue };
        let hot = g.selected.is_some_and(|n| r.from == n || r.to == n);
        if p.problem.is_some() {
            let (x, y, w, _) = g.node_box(r.from);
            let (cx, cy) = (x * TILE + w * TILE / 2, y * TILE - 4);
            fb.line(cx - 2, cy - 2, cx + 2, cy + 2, BAD);
            fb.line(cx - 2, cy + 2, cx + 2, cy - 2, BAD);
            continue;
        }
        if !hot && !g.show_routes {
            continue;
        }
        let (hi, dk) = species_colors(r.species);
        let (every_l, every_e) = if hot { (2, 3) } else { (4, 6) };
        let phase = if hot && g.sim.is_none() { (g.frame / 2) as i32 } else { 0 };
        dotted(fb, &path_points(&p.empty), dk, 1, every_e, -phase);
        dotted(fb, &path_points(&p.loaded), hi, -1, every_l, -phase);
    }
    if let Some((_, _, p)) = &g.preview {
        if p.problem.is_none() {
            let ph = -((g.frame / 2) as i32);
            dotted(fb, &path_points(&p.empty), ACCENT, 1, 2, ph);
            dotted(fb, &path_points(&p.loaded), GOLD, -1, 2, ph);
        }
    }
}

fn blit_creature(fb: &mut Framebuffer, g: &Game, s: Species, frame: usize, px: i32, py: i32, flip: bool, cargo: Option<Good>) {
    let spr = g.art.creature(s, frame);
    let x = px + (TILE - spr.w) / 2;
    let y = py + TILE - spr.h;
    fb.blit(spr, x, y, flip);
    if let Some(gd) = cargo {
        let c = g.art.good(gd);
        fb.blit(c, px + (TILE - c.w) / 2, y - c.h + 2, false);
    }
}

fn draw_creatures(fb: &mut Framebuffer, g: &Game) {
    match &g.sim {
        Some(sim) => {
            let mut seen: Vec<((i32, i32), i32)> = Vec::new();
            for c in &sim.creatures {
                let plan = &sim.plans[c.route];
                let (mut px, mut py) = (c.x * TILE, c.y * TILE);
                let mut frame = 0;
                let path = match c.state {
                    CState::Carry => Some(&plan.loaded),
                    CState::Return => Some(&plan.empty),
                    _ => None,
                };
                if let Some(st) = path.and_then(|p| p.steps.get(c.step)) {
                    px += (st.x - c.x) * TILE * c.prog / st.t.max(1);
                    py += (st.y - c.y) * TILE * c.prog / st.t.max(1);
                    frame = match st.m {
                        Move::Walk => ((c.x + c.step as i32 + if c.prog * 2 >= st.t { 1 } else { 0 }) % 2) as usize,
                        Move::Rise | Move::Leap | Move::Fall => 2,
                        Move::Climb => 3,
                    };
                    if st.m == Move::Climb {
                        frame = 3;
                    }
                } else {
                    // Waiting creatures shuffle aside so a queue stays countable.
                    let k = seen.iter().find(|(p, _)| *p == (c.x, c.y)).map(|(_, n)| *n).unwrap_or(0);
                    if let Some(e) = seen.iter_mut().find(|(p, _)| *p == (c.x, c.y)) {
                        e.1 += 1;
                    } else {
                        seen.push(((c.x, c.y), 1));
                    }
                    px += [0, -3, 3, -6, 6][(k as usize).min(4)];
                }
                blit_creature(fb, g, c.species, frame, px, py, c.facing < 0, c.cargo);
            }
        }
        None => {
            // Before the run: the crews stand at their pickups.
            let mut seen: Vec<((i32, i32), i32)> = Vec::new();
            for (i, r) in g.layout.routes.iter().enumerate() {
                let Some(p) = g.plans.get(i) else { continue };
                if p.problem.is_some() {
                    continue;
                }
                let (x, y) = p.loaded.start;
                for _ in 0..r.count {
                    let k = seen.iter().find(|(pp, _)| *pp == (x, y)).map(|(_, n)| *n).unwrap_or(0);
                    if let Some(e) = seen.iter_mut().find(|(pp, _)| *pp == (x, y)) {
                        e.1 += 1;
                    } else {
                        seen.push(((x, y), 1));
                    }
                    let off = [0, -3, 3, -6, 6, -9, 9, -2, 2][(k as usize).min(8)];
                    let flip = p.loaded.steps.first().is_some_and(|s| s.x < x);
                    blit_creature(fb, g, r.species, 0, x * TILE + off, y * TILE, flip, None);
                }
            }
        }
    }
    for &(x, y, _, c) in &g.sparks {
        fb.pset(x, y, c);
        fb.pset(x - 1, y, c);
        fb.pset(x + 1, y, c);
        fb.pset(x, y - 1, c);
    }
}

// ----- Placement ghost ---------------------------------------------------------------------------

fn draw_ghost(fb: &mut Framebuffer, g: &Game) {
    let Some((tx, ty)) = g.hovered_tile() else { return };
    if !matches!(g.overlay, Overlay::None) {
        return;
    }
    if let Some(card) = g.card_rect() {
        if card.contains(g.mouse.0, g.mouse.1) {
            return;
        }
    }
    match g.tool {
        Tool::Build(_) => {
            let ok = g.world.can_build(tx, ty - 1).is_ok();
            let c = if ok { GOOD } else { BAD };
            fb.corners(tx * TILE, (ty - 1) * TILE, 20, 20, 4, c);
        }
        Tool::Infra(i) => {
            let (ok, h) = match i {
                Infra::Spring => (g.world.can_spring(tx, ty).is_ok(), 1),
                Infra::Ladder => match g.world.can_ladder(tx, ty) {
                    Ok(n) => (true, n),
                    Err(_) => (false, 1),
                },
                Infra::Plank => (g.world.can_plank(tx, ty).is_ok(), 1),
            };
            let c = if ok { GOOD } else { BAD };
            fb.corners(tx * TILE, ty * TILE, 10, 10 * h, 3, c);
            if ok && i == Infra::Ladder {
                fb.rect_checker(tx * TILE + 2, ty * TILE, 6, 10 * h, WOOD_HI, 0);
            }
        }
        Tool::Erase => fb.corners(tx * TILE, ty * TILE, 10, 10, 3, BAD),
        Tool::Species(_) | Tool::Select => {
            if let Some(n) = g.hovered_node() {
                let (x, y, w, h) = g.node_box(n);
                fb.corners(x * TILE - 1, y * TILE - 1, w * TILE + 2, h * TILE + 2, 3, ACCENT_HI);
            }
        }
    }
}

// ----- Panel, card, overlays ---------------------------------------------------------------------

fn tool_icon(fb: &mut Framebuffer, g: &Game, t: Tool, r: Rect) {
    let (cx, cy) = (r.x + r.w / 2, r.y + r.h / 2);
    match t {
        Tool::Select => {
            // The whole 10x16 pointer, inside the button's 1px bevel (rows 2..17).
            let a = &g.art.pointer;
            fb.blit(a, r.x + (r.w - a.w) / 2 + 1, r.y + (r.h - a.h) / 2, false);
        }
        Tool::Species(s) => {
            let spr = g.art.creature(s, 0);
            fb.blit(spr, cx - spr.w / 2, cy - spr.h / 2, false);
        }
        Tool::Build(k) => {
            let (wall, roof) = match k {
                Kind::Depot => (WOOD, WOOD_DK),
                Kind::Mason => (ROCK, SOIL_DK),
                Kind::Weaver => (WOOD_HI, BEAK),
                Kind::Kiln => (BRICK, BRICK),
            };
            fb.rect_fill(cx - 6, cy - 1, 12, 7, wall);
            fb.rect_outline(cx - 6, cy - 1, 12, 7, OUTLINE);
            for k2 in 0..4 {
                fb.rect_fill(cx - 4 + k2 - 2, cy - 2 - k2, 12 - (k2 * 2) as i32, 1, roof);
            }
            fb.rect_fill(cx - 1, cy + 2, 2, 4, PANEL_DEEP);
            if let Some(rc) = k.recipe() {
                let s = g.art.good(rc.output);
                fb.blit(s, cx + 3, cy - 7, false);
            }
        }
        Tool::Infra(Infra::Spring) => draw_spring(fb, cx - 5, cy - 5),
        Tool::Infra(Infra::Ladder) => {
            fb.rect_fill(cx - 4, cy - 7, 1, 14, WOOD);
            fb.rect_fill(cx + 3, cy - 7, 1, 14, WOOD);
            for y in [-5, -1, 3] {
                fb.rect_fill(cx - 3, cy + y, 6, 1, WOOD_HI);
            }
        }
        Tool::Infra(Infra::Plank) => {
            fb.rect_fill(cx - 7, cy - 2, 14, 3, WOOD_HI);
            fb.rect_fill(cx - 7, cy + 1, 14, 1, OUTLINE);
            fb.pset(cx - 6, cy - 1, ROCK_DK);
            fb.pset(cx + 5, cy - 1, ROCK_DK);
        }
        Tool::Erase => {
            fb.line(cx - 4, cy - 4, cx + 4, cy + 4, BAD);
            fb.line(cx - 4, cy + 4, cx + 4, cy - 4, BAD);
            fb.line(cx - 3, cy - 4, cx + 5, cy + 4, BAD);
            fb.line(cx - 3, cy + 4, cx + 5, cy - 4, BAD);
        }
    }
}

pub fn tool_label(t: Tool) -> String {
    match t {
        Tool::Select => "Select. Click a place to see its routes. Right-click takes away what you built.".into(),
        Tool::Species(s) => format!("{}. {}", s.title(), s.blurb()),
        Tool::Build(k) => format!("{}. {}", k.title(), k.blurb()),
        Tool::Infra(i) => format!("{}. {}", i.title(), i.blurb()),
        Tool::Erase => "Erase. Click something you built to take it away.".into(),
    }
}

// ----- Small graphics -----------------------------------------------------------------------------

/// A number next to an icon; returns the x after it.
fn icon_num(fb: &mut Framebuffer, icon: &Sprite, x: i32, y: i32, n: &str, col: u8) -> i32 {
    fb.blit(icon, x, y + (8 - icon.h) / 2, false);
    let tx = x + icon.w + 2;
    fb.text(n, tx, y, col, None, 1);
    tx + Framebuffer::text_width(n, 1)
}

/// A good's icon with "xN" after it; returns the x after it.
fn good_count(fb: &mut Framebuffer, g: &Game, good: Good, n: i32, x: i32, y: i32, col: u8) -> i32 {
    let s = g.art.good(good);
    fb.blit(s, x, y + (8 - s.h) / 2, false);
    if n > 1 {
        let t = format!("x{n}");
        fb.text(&t, x + s.w + 1, y, col, None, 1);
        return x + s.w + 2 + Framebuffer::text_width(&t, 1);
    }
    x + s.w + 1
}

/// A small right arrow, 7 wide.
fn arrow(fb: &mut Framebuffer, x: i32, y: i32, col: u8) {
    fb.rect_fill(x, y + 3, 5, 1, col);
    fb.pset(x + 4, y + 2, col);
    fb.pset(x + 4, y + 4, col);
    fb.pset(x + 3, y + 1, col);
    fb.pset(x + 3, y + 5, col);
}

/// A recipe as icons: inputs, arrow, output. Returns the x after it.
fn recipe(fb: &mut Framebuffer, g: &Game, k: Kind, x: i32, y: i32) -> i32 {
    let Some(r) = k.recipe() else { return x };
    let mut cx = x;
    for (i, (good, n)) in r.inputs.iter().enumerate() {
        if i > 0 {
            fb.text("+", cx, y, TEXT_DIM, None, 1);
            cx += 7;
        }
        cx = good_count(fb, g, *good, *n, cx, y, TEXT) + 2;
    }
    arrow(fb, cx, y, GOLD);
    cx += 9;
    cx = good_count(fb, g, r.output, 1, cx, y, TEXT);
    fb.text(&format!("{}t", r.ticks), cx + 4, y, TEXT_DIM, None, 1);
    cx + 4 + Framebuffer::text_width(&format!("{}t", r.ticks), 1)
}

/// The jump table of a creature drawn as columns of blocks: one column per load.
fn jump_table(fb: &mut Framebuffer, g: &Game, s: Species, x: i32, y: i32) {
    let loads: [(Option<Good>, Weight); 4] = [(None, Weight::Empty), (Some(Good::Berry), Weight::Light), (Some(Good::Twig), Weight::Medium), (Some(Good::Stone), Weight::Heavy)];
    fb.text("jump", x, y + 12, TEXT_DIM, None, 1);
    fb.text("pace", x, y + 24, TEXT_DIM, None, 1);
    for (k, (good, w)) in loads.iter().enumerate() {
        let cx = x + 30 + k as i32 * 22;
        match good {
            Some(gd) => {
                let sp = g.art.good(*gd);
                fb.blit(sp, cx + (8 - sp.w) / 2, y, false);
            }
            None => fb.blit(&g.art.empty, cx + 1, y + 1, false),
        }
        let j = s.jump(*w);
        for b in 0..j {
            fb.rect_fill(cx, y + 18 - b * 4, 8, 3, GOOD);
        }
        if j == 0 {
            fb.text("x", cx + 1, y + 13, BAD, None, 1);
        }
        let p = s.pace(*w);
        fb.text(&format!("{p}"), cx + 1, y + 24, if p >= 4 { BAD } else { TEXT }, None, 1);
    }
    let climbs = if s.climbs_vine() { "climbs vines" } else { "" };
    if !climbs.is_empty() {
        fb.text(climbs, x + 120, y + 24, ACCENT, None, 1);
    }
}

/// The card that pops up above a tool button: pictures first, a few words after.
fn tool_card(fb: &mut Framebuffer, g: &Game, t: Tool, anchor_x: i32) {
    let (w, h) = match t {
        Tool::Species(_) => (204, 62),
        _ => (204, 40),
    };
    let x = (anchor_x - w / 2).clamp(2, 398 - w);
    let y = VIEW_H - h - 2;
    ui::panel(fb, Rect::new(x, y, w, h));
    let ix = x + 6;
    match t {
        Tool::Species(s) => {
            let spr = g.art.creature(s, 0);
            fb.blit(spr, ix, y + 13 - spr.h, false);
            fb.text(s.title(), ix + 14, y + 5, GOLD, None, 1);
            icon_num(fb, &g.art.seed, x + w - 30, y + 5, &format!("{}", s.cost()), TEXT);
            jump_table(fb, g, s, ix, y + 15);
            fb.text(s.blurb(), ix, y + h - 10, TEXT_DIM, None, 1);
        }
        Tool::Build(k) => {
            fb.text(k.title(), ix, y + 5, GOLD, None, 1);
            icon_num(fb, &g.art.seed, x + w - 30, y + 5, &format!("{}", k.cost()), TEXT);
            if k == Kind::Depot {
                for i in 0..6 {
                    fb.rect_fill(ix + i * 6, y + 18, 4, 4, good_color(GOODS[i as usize % 4]));
                }
                fb.text("any 6 goods", ix + 40, y + 16, TEXT, None, 1);
            } else {
                recipe(fb, g, k, ix, y + 16);
            }
            fb.text(k.blurb(), ix, y + 28, TEXT_DIM, None, 1);
        }
        Tool::Infra(i) => {
            fb.text(i.title(), ix, y + 5, GOLD, None, 1);
            let price = match i {
                Infra::Spring => "2".to_string(),
                Infra::Ladder => "1 a rung".to_string(),
                Infra::Plank => "2 a tile".to_string(),
            };
            icon_num(fb, &g.art.seed, x + w - 60, y + 5, &price, TEXT);
            fb.text_wrapped(i.blurb(), ix, y + 16, w - 12, 10, TEXT_DIM, None);
        }
        Tool::Select | Tool::Erase => {
            fb.text_wrapped(&tool_label(t), ix, y + 6, w - 12, 10, TEXT, None);
        }
    }
}

/// A cream speech bubble with a pointer toward (ax, ay).
pub fn bubble(fb: &mut Framebuffer, text: &str, ax: i32, ay: i32) {
    let lines = crate::framebuffer::wrap(text, 30);
    let w = lines.iter().map(|l| Framebuffer::text_width(l, 1)).max().unwrap_or(0) + 12;
    let h = lines.len() as i32 * 10 + 8;
    let above = ay > h + 16;
    let x = (ax - w / 2).clamp(2, 398 - w);
    let y = if above { ay - h - 8 } else { ay + 10 };
    fb.rect_fill(x, y, w, h, OUTLINE);
    fb.rect_fill(x + 1, y + 1, w - 2, h - 2, EGG);
    fb.rect_fill(x + 1, y + h - 2, w - 2, 1, CLAY);
    // Pointer.
    let px = ax.clamp(x + 6, x + w - 7);
    for k in 0..6 {
        let yy = if above { y + h - 1 + k } else { y - k };
        let half = 5 - k;
        if half > 0 {
            fb.rect_fill(px - half + 1, yy, half * 2 - 1, 1, EGG);
            fb.pset(px - half, yy, OUTLINE);
            fb.pset(px + half, yy, OUTLINE);
        } else {
            fb.pset(px, yy, OUTLINE);
        }
    }
    for (i, l) in lines.iter().enumerate() {
        fb.text(l, x + 6, y + 5 + i as i32 * 10, PANEL_DEEP, None, 1);
    }
}

// ----- Panel, header, card --------------------------------------------------------------------------

fn draw_panel(fb: &mut Framebuffer, g: &Game) {
    ui::panel(fb, Rect::new(0, VIEW_H, 400, 300 - VIEW_H));
    let (mx, my) = g.mouse;
    let tools = g.tools();
    let mut hover_label: Option<String> = None;
    let mut hover_tool: Option<(Tool, i32)> = None;
    for (i, &t) in tools.iter().enumerate() {
        let r = tool_rect(i);
        let hover = r.contains(mx, my);
        ui::button_face(fb, r, hover, g.tool == t);
        tool_icon(fb, g, t, r);
        if hover {
            let key = if t == Tool::Erase { "X".to_string() } else { (i + 1).to_string() };
            hover_label = Some(format!("Key {key}. {}", tool_label(t)));
            hover_tool = Some((t, r.x + r.w / 2));
        }
    }
    // Run controls.
    let running = g.running;
    let hov = |r: Rect| r.contains(mx, my);
    ui::button_face(fb, R_RUN, hov(R_RUN), running);
    let (cx, cy) = (R_RUN.x + 10, R_RUN.y + 10);
    if running {
        fb.rect_fill(cx - 4, cy - 4, 3, 9, TEXT);
        fb.rect_fill(cx + 1, cy - 4, 3, 9, TEXT);
    } else {
        for k in 0..5 {
            fb.rect_fill(cx - 3 + k, cy - 4 + k, 1, 9 - 2 * k, GOOD);
        }
    }
    ui::button_face(fb, R_STEP, hov(R_STEP), false);
    let (cx, cy) = (R_STEP.x + 9, R_STEP.y + 10);
    for k in 0..4 {
        fb.rect_fill(cx - 3 + k, cy - 3 + k, 1, 7 - 2 * k, TEXT);
    }
    fb.rect_fill(cx + 2, cy - 3, 2, 7, TEXT);
    ui::button(fb, R_SPEED, &format!("x{}", SPEEDS[g.speed]), hov(R_SPEED), false);
    ui::button_face(fb, R_RESET, hov(R_RESET), false);
    fb.rect_fill(R_RESET.x + 6, R_RESET.y + 6, 8, 8, if g.sim.is_some() { BAD } else { TEXT_FAINT });
    if hov(R_RUN) {
        hover_label = Some("Run or pause. Space does it too.".into());
    } else if hov(R_STEP) {
        hover_label = Some("One tick at a time. Key S.".into());
    } else if hov(R_SPEED) {
        hover_label = Some("Speed: ticks per frame. Tab does it too.".into());
    } else if hov(R_RESET) {
        hover_label = Some("Back to tick 0. Key R.".into());
    }
    // Status.
    ui::inset(fb, R_STATUS);
    let (text, bad) = match hover_label {
        Some(t) => (t, false),
        None => g.status_line(),
    };
    fb.text_wrapped(&text, R_STATUS.x + 4, R_STATUS.y + 3, R_STATUS.w - 8, 10, if bad { BAD } else { TEXT }, None);
    // Scores: now, against the goal, and your best.
    ui::inset(fb, R_METRICS);
    let rec = g.save.rec(&g.level().id);
    let ticks = g.sim.as_ref().map(|s| s.tick).unwrap_or(0);
    let cost = crate::world::layout_cost(g.level(), &g.layout);
    let crew = g.layout.crew();
    let tg = g.level().targets;
    let rows = [(&g.art.clock, ticks, rec.ticks, tg.ticks), (&g.art.seed, cost, rec.cost, tg.cost), (&g.art.paw, crew, rec.crew, tg.crew)];
    for (k, (icon, v, best, goal)) in rows.iter().enumerate() {
        let y = R_METRICS.y + 3 + k as i32 * 10;
        fb.blit(icon, R_METRICS.x + 4, y, false);
        let live_ok = *v <= *goal && (k != 0 || g.sim.as_ref().is_some_and(|s| matches!(s.outcome, Outcome::Done(_))));
        fb.text(&format!("{v:>4}"), R_METRICS.x + 13, y, if live_ok { GOOD } else { TEXT }, None, 1);
        fb.text(&format!("goal {goal}"), R_METRICS.x + 42, y, TEXT_DIM, None, 1);
        if let Some(b) = best {
            ui::star(fb, R_METRICS.x + 112, y, *b <= *goal);
        }
    }
    if let Some((t, ax)) = hover_tool {
        if matches!(g.overlay, Overlay::None) {
            tool_card(fb, g, t, ax);
        }
    }
}

/// The site's goals as icon chips: met ones are gold with a star.
fn goal_chips(fb: &mut Framebuffer, g: &Game, x: i32, y: i32) -> i32 {
    let lv = g.level();
    let rec = g.save.rec(&lv.id);
    let tg = lv.targets;
    let mut cx = x;
    for (icon, goal, best) in [(&g.art.clock, tg.ticks, rec.ticks), (&g.art.seed, tg.cost, rec.cost), (&g.art.paw, tg.crew, rec.crew)] {
        let met = best.is_some_and(|b| b <= goal);
        cx = icon_num(fb, icon, cx, y, &format!("{goal}"), if met { GOLD } else { TEXT }) + 6;
    }
    if tg.combo_ticks > 0 {
        ui::star(fb, cx, y, rec.combo);
        cx += 9;
        cx = icon_num(fb, &g.art.clock, cx, y, &format!("{}", tg.combo_ticks), if rec.combo { GOLD } else { TEXT }) + 2;
        cx = icon_num(fb, &g.art.seed, cx, y, &format!("{}", tg.combo_cost), if rec.combo { GOLD } else { TEXT });
    }
    cx
}

fn draw_header(fb: &mut Framebuffer, g: &Game) {
    let lv = g.level();
    let title = format!("{} {}", lv.id, lv.name);
    fb.rect_checker(0, 0, 400, 12, SHADOW, 0);
    fb.text(&title, 3, 3, TEXT, Some(OUTLINE), 1);
    goal_chips(fb, g, 12 + Framebuffer::text_width(&title, 1), 3);
    let (mx, my) = g.mouse;
    ui::button(fb, R_HELP, "?", R_HELP.contains(mx, my), false);
    ui::button(fb, R_ROUTES, "Paths", R_ROUTES.contains(mx, my), g.show_routes);
    ui::button(fb, R_MENU, "Menu", R_MENU.contains(mx, my), false);
}

fn draw_card(fb: &mut Framebuffer, g: &Game) {
    let Some(card) = g.card_rect() else { return };
    let Some(n) = g.selected else { return };
    ui::panel(fb, card);
    fb.text(&cap(&g.node_name(n)), card.x + 4, card.y + 4, GOLD, None, 1);
    let mut x = card.x + 4;
    let y = card.y + 14;
    let lv = g.level();
    // What's inside right now (or, before a run, what this place does).
    let mut items: Vec<(Good, String)> = Vec::new();
    match (&g.sim, n) {
        (Some(sim), _) => {
            if let Some(st) = sim.node(n) {
                for gd in GOODS {
                    if let NodeRef::Source(_) = n {
                        if st.max <= 0 && st.good == Some(gd) {
                            items.push((gd, "endless".into()));
                        } else if st.good == Some(gd) {
                            items.push((gd, format!("{} left", st.stock[gd.index()])));
                        }
                    } else if n == NodeRef::Fortress {
                        let need = lv.need_of(gd);
                        if need > 0 {
                            items.push((gd, format!("{}/{need}", st.stock[gd.index()])));
                        }
                    } else if st.stock[gd.index()] > 0 {
                        items.push((gd, format!("{}", st.stock[gd.index()])));
                    }
                }
            }
        }
        (None, NodeRef::Source(i)) => {
            let s = &lv.sources[i];
            items.push((s.good, if s.infinite() { "endless".into() } else if s.regrow > 0 { format!("{}, +1 every {}t", s.max, s.regrow) } else { format!("only {}", s.max) }));
        }
        (None, NodeRef::Fortress) => items = lv.need.iter().map(|(gd, c)| (*gd, format!("0/{c}"))).collect(),
        (None, NodeRef::Building(id)) => {
            if let Some(b) = g.layout.building(id) {
                if b.kind.recipe().is_some() {
                    recipe(fb, g, b.kind, x, y);
                } else {
                    fb.text("holds any 6 goods", x, y, TEXT_DIM, None, 1);
                }
            }
        }
    }
    if items.is_empty() && g.sim.is_some() {
        fb.text("empty", x, y, TEXT_FAINT, None, 1);
    }
    for (gd, t) in items {
        let s = g.art.good(gd);
        if x + s.w + 4 + Framebuffer::text_width(&t, 1) > card.x + card.w - 2 {
            break;
        }
        fb.blit(s, x, y + (8 - s.h) / 2, false);
        fb.text(&t, x + s.w + 2, y, TEXT, None, 1);
        x += s.w + 8 + Framebuffer::text_width(&t, 1);
    }
    let routes = g.card_routes();
    if routes.is_empty() {
        fb.text("No routes yet.", card.x + 4, card.y + 27, TEXT_FAINT, None, 1);
    }
    let (mx, my) = g.mouse;
    for (i, ri) in routes.into_iter().enumerate() {
        let r = g.layout.routes[ri];
        let (row, cargo, minus, plus, del) = g.card_row(card, i);
        if row.contains(mx, my) {
            fb.rect_fill(row.x, row.y, row.w, row.h, PANEL_LO);
        }
        let spr = g.art.creature(r.species, 0);
        fb.blit(spr, row.x + 2, row.y + 11 - spr.h, false);
        fb.text(&format!("{}", r.count), row.x + 13, row.y + 3, TEXT, None, 1);
        let gs = g.art.good(r.cargo);
        fb.blit(gs, cargo.x + (10 - gs.w) / 2, cargo.y + (10 - gs.h) / 2, false);
        let (to_here, other) = if r.from == n { (false, r.to) } else { (true, r.from) };
        let mut name = g.node_name(other);
        name.truncate(7);
        if to_here {
            fb.text("<", row.x + 36, row.y + 3, TEXT_DIM, None, 1);
        } else {
            arrow(fb, row.x + 36, row.y + 2, TEXT_DIM);
        }
        fb.text(&name, row.x + 44, row.y + 3, TEXT_DIM, None, 1);
        match g.plans.get(ri).and_then(|p| if p.problem.is_none() { Some(p.cycle()) } else { None }) {
            Some(c) => fb.text(&format!("{c:>3}"), row.x + 94, row.y + 3, TEXT_DIM, None, 1),
            None => fb.text(" !!", row.x + 94, row.y + 3, BAD, None, 1),
        }
        for (b, lab) in [(minus, "-"), (plus, "+"), (del, "x")] {
            let h = b.contains(mx, my);
            ui::button_face(fb, b, h, false);
            fb.text(lab, b.x + 2, b.y + 1, if lab == "x" { BAD } else if h { PANEL_DEEP } else { TEXT }, None, 1);
        }
    }
    // A one-line explanation for the hovered row.
    for (i, ri) in g.card_routes().into_iter().enumerate() {
        let (row, cargo, minus, plus, del) = g.card_row(card, i);
        if row.contains(mx, my) {
            let r = g.layout.routes[ri];
            let tip = match g.plans.get(ri).and_then(|p| p.problem.clone()) {
                Some(p) => cap(p.text()),
                None if cargo.contains(mx, my) => "Click to change what this route carries.".to_string(),
                None if minus.contains(mx, my) => "One fewer.".to_string(),
                None if plus.contains(mx, my) => "One more.".to_string(),
                None if del.contains(mx, my) => "Fire this whole crew.".to_string(),
                None => format!("{} {}, {} ticks a round trip.", r.count, if r.count == 1 { r.species.title().to_lowercase() } else { r.species.plural().to_string() }, g.plans[ri].cycle()),
            };
            let w = Framebuffer::text_width(&tip, 1) + 6;
            let tx = (card.x + card.w / 2 - w / 2).clamp(1, 399 - w);
            let ty = card.y + card.h + 1;
            fb.rect_bevel(tx, ty, w, 11, PANEL_DEEP, PANEL_LO, PANEL_LO);
            fb.text(&tip, tx + 3, ty + 2, TEXT, None, 1);
        }
    }
}

// ----- Site card, results ---------------------------------------------------------------------------

/// What a "new" token on the site card looks like: an icon, a name and a line.
fn new_thing(fb: &mut Framebuffer, g: &Game, token: &str, x: i32, y: i32) {
    let frame = Rect::new(x, y, 20, 20);
    ui::inset(fb, frame);
    let (name, line): (String, String) = if let Some(s) = Species::from_name(token) {
        tool_icon(fb, g, Tool::Species(s), frame);
        (s.title().into(), s.blurb().into())
    } else if let Some(k) = Kind::from_name(token) {
        tool_icon(fb, g, Tool::Build(k), frame);
        (k.title().into(), k.blurb().into())
    } else if let Some(i) = Infra::from_name(token) {
        tool_icon(fb, g, Tool::Infra(i), frame);
        (i.title().into(), i.blurb().into())
    } else if let Some(gd) = Good::from_name(token) {
        let s = g.art.good(gd);
        fb.blit(s, x + (20 - s.w) / 2, y + (20 - s.h) / 2, false);
        let line = match gd {
            Good::Berry => "Light. Tribbles hardly notice it.",
            Good::Twig => "Medium. Slows a tribble right down.",
            Good::Stone => "Heavy. Nobody jumps far with one.",
            Good::Egg => "Fragile: never dropped more than a tile.",
            Good::Clay => "Medium, like a twig.",
            _ => "",
        };
        (cap(gd.word()), line.into())
    } else {
        // Terrain: draw a sample tile.
        let (tile, line) = match token {
            "VINE" => (Tile::Vine, "Apes climb vines, carrying anything."),
            "WATER" => (Tile::Water, "Nobody swims. Jump it or bridge it."),
            "CLOUD" => (Tile::Cloud, "Holds you only if you carry light or nothing."),
            "BRAMBLE" => (Tile::Bramble, "Solid, but nothing can be built on it."),
            _ => (Tile::Air, ""),
        };
        let mut w = crate::world::World::new(g.level(), &crate::layout::Layout::default());
        w.tiles = vec![tile; (w.w * w.h) as usize];
        let mut tmp = Framebuffer::new();
        draw_tile(&mut tmp, &w, 1, 1);
        for yy in 0..TILE {
            for xx in 0..TILE {
                let c = tmp.get(TILE + xx, TILE + yy);
                if c != 0 {
                    fb.pset(x + 5 + xx, y + 5 + yy, c);
                }
            }
        }
        (cap(&token.to_lowercase()), line.into())
    };
    let line = match token {
        "LADDER" => "Apes climb with anything, others light only.".to_string(),
        "PLANK" => "A board to stand on or jump up through.".to_string(),
        "SPRING" => "Jump 2 higher from it, even with a stone.".to_string(),
        _ => line,
    };
    fb.text(&name, x + 26, y + 2, GOLD, None, 1);
    fb.text(&line, x + 26, y + 12, TEXT, None, 1);
}

/// The card shown on arriving at a site: what it wants, what's new, what to aim for.
fn draw_site_card(fb: &mut Framebuffer, g: &Game) {
    let lv = g.level();
    ui::dim(fb, Rect::new(0, 0, 400, VIEW_H));
    let intro = crate::framebuffer::wrap(&lv.rule, 50);
    let news = lv.new.len() as i32;
    let h = 64 + intro.len() as i32 * 10 + news * 24;
    let r = Rect::new(40, ((VIEW_H - h) / 2).max(2), 320, h);
    ui::panel(fb, r);
    fb.text(&lv.id, r.x + 8, r.y + 6, TEXT_DIM, None, 1);
    fb.text(&lv.name, r.x + 32, r.y + 6, GOLD, Some(OUTLINE), 1);
    let mut y = r.y + 20;
    for l in &intro {
        fb.text(l, r.x + 8, y, TEXT, None, 1);
        y += 10;
    }
    y += 4;
    for t in &lv.new {
        new_thing(fb, g, t, r.x + 8, y);
        y += 24;
    }
    // Wants.
    fb.text("Wants", r.x + 8, y + 1, TEXT_DIM, None, 1);
    let mut x = r.x + 50;
    for (gd, n) in &lv.need {
        x = good_count(fb, g, *gd, *n, x, y + 1, TEXT) + 8;
    }
    y += 14;
    fb.text("Goals", r.x + 8, y + 1, TEXT_DIM, None, 1);
    goal_chips(fb, g, r.x + 50, y + 1);
    fb.text_centered("Click to start", r.x + r.w / 2, r.y + r.h - 11, ACCENT_HI, None, 1);
}

fn draw_results(fb: &mut Framebuffer, g: &Game) {
    let r = R_RESULTS;
    let (mx, my) = g.mouse;
    let b = result_buttons();
    match &g.overlay {
        Overlay::Results { ticks, cost, crew, best, combo } => {
            ui::dim(fb, Rect::new(0, 0, 400, VIEW_H));
            ui::panel(fb, r);
            fb.text_centered("The fortress stands!", r.x + r.w / 2, r.y + 7, GOLD, Some(OUTLINE), 1);
            let lv = g.level();
            let rec = g.save.rec(&lv.id);
            let tg = lv.targets;
            fb.text("this run", r.x + 30, r.y + 22, TEXT_DIM, None, 1);
            fb.text("best", r.x + 88, r.y + 22, TEXT_DIM, None, 1);
            fb.text("goal", r.x + 130, r.y + 22, TEXT_DIM, None, 1);
            fb.text("everyone else", r.x + 236, r.y + 22, TEXT_DIM, None, 1);
            let histo = g.histos.iter().find(|(id, _)| *id == lv.id).map(|(_, h)| h);
            let rows = [(&g.art.clock, *ticks, rec.ticks, tg.ticks, best[0]), (&g.art.seed, *cost, rec.cost, tg.cost, best[1]), (&g.art.paw, *crew, rec.crew, tg.crew, best[2])];
            for (k, (icon, v, b2, goal, nb)) in rows.iter().enumerate() {
                let y = r.y + 36 + k as i32 * 20;
                fb.blit(icon, r.x + 12, y, false);
                fb.text(&format!("{v:>5}"), r.x + 30, y, if *nb { GOOD } else { TEXT }, None, 1);
                fb.text(&format!("{:>5}", b2.unwrap_or(*v)), r.x + 76, y, GOLD, None, 1);
                fb.text(&format!("{goal}"), r.x + 132, y, TEXT_DIM, None, 1);
                ui::star(fb, r.x + 160, y, b2.unwrap_or(*v) <= *goal);
                if *nb {
                    fb.text("new best", r.x + 172, y, GOOD, None, 1);
                }
                if let Some((lo, hi, bins)) = histo.and_then(|h| h.m.get(k)) {
                    draw_histo(fb, r.x + 236, y - 6, *lo, *hi, bins, *v);
                }
            }
            if tg.combo_ticks > 0 {
                let y = r.y + 98;
                ui::star(fb, r.x + 12, y, rec.combo);
                let t = format!("Both at once: {} ticks or less and {} seeds or less", tg.combo_ticks, tg.combo_cost);
                fb.text(&t, r.x + 24, y, if *combo { GOLD } else { TEXT }, None, 1);
            }
            fb.text_centered("Every star is its own puzzle.", r.x + r.w / 2, r.y + 116, TEXT_DIM, None, 1);
            let last = g.cur + 1 >= g.levels.len();
            for (k, lab) in [(0, if last { "Close" } else { "Next site" }), (1, "Keep tuning"), (2, "Menu")] {
                ui::button(fb, b[k], lab, b[k].contains(mx, my), false);
            }
        }
        Overlay::Failed(msg) => {
            ui::panel(fb, Rect::new(r.x, r.y + 40, r.w, 80));
            fb.text_centered("The colony ground to a halt", r.x + r.w / 2, r.y + 48, BAD, Some(OUTLINE), 1);
            fb.text_wrapped(msg, r.x + 10, r.y + 62, r.w - 20, 10, TEXT, None);
            for (k, lab) in [(0, "Reset"), (1, "Look"), (2, "Menu")] {
                ui::button(fb, b[k], lab, b[k].contains(mx, my), false);
            }
        }
        _ => {}
    }
}

/// A 16-bar histogram, 96x14, lower scores to the left; the bar holding `you` is gold.
fn draw_histo(fb: &mut Framebuffer, x: i32, y: i32, lo: i32, hi: i32, bins: &[u32], you: i32) {
    let max = bins.iter().copied().max().unwrap_or(1).max(1);
    let yours = crate::harness::histo_bin(you, lo, hi);
    fb.rect_fill(x - 1, y + 14, 98, 1, PANEL_LO);
    for (i, &n) in bins.iter().enumerate() {
        let h = if n == 0 { 0 } else { 1 + (13 * n / max) as i32 };
        let col = if i == yours { GOLD } else { PANEL_HI };
        if h > 0 {
            fb.rect_fill(x + i as i32 * 6, y + 14 - h, 5, h, col);
        }
        if i == yours {
            fb.rect_fill(x + i as i32 * 6 + 2, y + 16, 1, 2, GOLD);
        }
    }
    if you < lo {
        fb.text("<", x - 7, y + 4, GOOD, None, 1);
    }
}

/// With a creature chosen and a pickup clicked: every spot it can reach carrying that pickup's
/// goods, so missing links (a step too high, a gap too wide) show before a route is made.
fn draw_reach(fb: &mut Framebuffer, g: &Game) {
    let (Tool::Species(s), Some(f)) = (g.tool, g.from) else { return };
    let good = match f {
        NodeRef::Source(i) => Some(g.level().sources[i].good),
        NodeRef::Building(id) => match g.layout.building(id).map(|b| b.kind) {
            Some(Kind::Depot) => g.layout.routes.iter().find(|r| r.to == f).map(|r| r.cargo),
            Some(k) => k.recipe().map(|r| r.output),
            None => None,
        },
        NodeRef::Fortress => None,
    };
    let Some(good) = good else { return };
    let load = Load::of(good);
    let w = &g.world;
    let starts = w.access(g.level(), &g.layout, f);
    let mut seen = vec![false; (w.w * w.h) as usize];
    for &(x, y) in &starts {
        for (i, r) in crate::path::reach(w, s, load, x, y).into_iter().enumerate() {
            seen[i] |= r;
        }
    }
    let home = crate::path::reach_back(w, s, Load::EMPTY, &starts);
    for y in 0..w.h {
        for x in 0..w.w {
            let i = w.idx(x, y);
            if seen[i] && w.standable_w(x, y, load.weight) {
                let (px, py) = (x * TILE + 4, y * TILE + 8);
                fb.rect_fill(px, py + 1, 2, 1, OUTLINE);
                fb.rect_fill(px, py, 2, 1, if home[i] { GOLD } else { BAD });
            }
        }
    }
}

fn draw_play(g: &Game, fb: &mut Framebuffer) {
    g.gallery.backdrop(g.level().chapter).draw(fb, &BACKDROP, VIEW_H);
    draw_terrain(fb, &g.world);
    draw_reach(fb, g);
    draw_nodes(fb, g);
    draw_routes(fb, g);
    draw_creatures(fb, g);
    if let Some(f) = g.from {
        let (x, y, w, h) = g.node_box(f);
        fb.rect_outline(x * TILE - 2, y * TILE - 2, w * TILE + 4, h * TILE + 4, GOLD);
    }
    if let Some(n) = g.selected {
        let (x, y, w, h) = g.node_box(n);
        fb.corners(x * TILE - 2, y * TILE - 2, w * TILE + 4, h * TILE + 4, 4, WHITE);
    }
    draw_ghost(fb, g);
    draw_header(fb, g);
    draw_card(fb, g);
    draw_panel(fb, g);
    match g.overlay {
        Overlay::Rule => draw_site_card(fb, g),
        Overlay::Results { .. } | Overlay::Failed(_) => draw_results(fb, g),
        Overlay::None => {
            if let Some((t, x, y)) = g.coach() {
                bubble(fb, &t, x, y);
            } else if let Some(b) = &g.tip {
                bubble(fb, &b.text, b.x, b.y);
            }
        }
    }
}

// ----- Menus -------------------------------------------------------------------------------------

fn title_slots() -> Vec<u8> {
    (0..TITLE_SLOTS as u8).map(|k| TITLE_FIRST + k).collect()
}

fn draw_title(g: &Game, fb: &mut Framebuffer) {
    g.gallery.title.draw(fb, &title_slots(), 300);
    fb.text_centered("BIRD FORTRESS", 202, 12, OUTLINE, None, 3);
    fb.text_centered("BIRD FORTRESS", 200, 10, GOLD, None, 3);
    fb.rect_checker(0, 262, 400, 38, SHADOW, 0);
    fb.rect_checker(0, 262, 400, 38, PANEL_DEEP, 1);
    fb.text_centered("Every creature sees a different map.", 200, 266, TEXT, Some(OUTLINE), 1);
    fb.text_centered("Click to begin", 200, 278, ACCENT_HI, Some(OUTLINE), 1);
    fb.text_centered("F1 rules   F2 scanlines   F3 music   Esc quit", 200, 290, TEXT_DIM, Some(OUTLINE), 1);
}

pub const CHAPTER_NAMES: [&str; 5] = ["Meadow", "Quarry", "Canopy", "Bastion", "Skyreach"];

fn draw_select(g: &Game, fb: &mut Framebuffer) {
    g.gallery.title.draw(fb, &title_slots(), 300);
    ui::dim(fb, Rect::new(0, 0, 400, 300));
    fb.text_centered("Choose a site", 200, 8, GOLD, Some(OUTLINE), 2);
    let (mx, my) = g.mouse;
    let mut stars = 0;
    let mut total = 0;
    for (c, list) in g.chapters().into_iter().enumerate() {
        let r0 = select_rect(c, 0);
        fb.text_centered(CHAPTER_NAMES.get(c).copied().unwrap_or("?"), r0.x + r0.w / 2, 32, TEXT, Some(OUTLINE), 1);
        for (i, li) in list.into_iter().enumerate() {
            let r = select_rect(c, i);
            let lv = &g.levels[li];
            let rec = g.save.rec(&lv.id);
            let open = g.unlocked(li);
            if open {
                let col = ui::button_face(fb, r, r.contains(mx, my), false);
                fb.text(&lv.id, r.x + 3, r.y + 3, if rec.solved { GOOD } else { col }, None, 1);
                let mut name = lv.name.clone();
                name.truncate(12);
                fb.text(&name, r.x + 3, r.y + 15, col, None, 1);
            } else {
                ui::button_off(fb, r, "");
                fb.text(&lv.id, r.x + 3, r.y + 3, TEXT_FAINT, None, 1);
                fb.text("locked", r.x + 3, r.y + 15, TEXT_FAINT, None, 1);
            }
            let tg = lv.targets;
            let got = [rec.ticks.is_some_and(|v| v <= tg.ticks), rec.cost.is_some_and(|v| v <= tg.cost), rec.crew.is_some_and(|v| v <= tg.crew), rec.combo];
            for (k, s) in got.iter().enumerate() {
                if k == 3 && tg.combo_ticks == 0 {
                    continue;
                }
                stars += *s as i32;
                total += 1;
                ui::star(fb, r.x + 38 + k as i32 * 9, r.y + 3, *s);
            }
        }
    }
    let solved = g.levels.iter().filter(|l| g.save.rec(&l.id).solved).count();
    fb.text_centered(&format!("Stars {stars} of {total}    Sites done {solved} of {}", g.levels.len()), 200, 262, TEXT, Some(OUTLINE), 1);
    ui::button(fb, R_SEL_HELP, "Rules", R_SEL_HELP.contains(mx, my), false);
    ui::button(fb, R_SEL_QUIT, "Quit", R_SEL_QUIT.contains(mx, my), false);
}

fn draw_help(g: &Game, fb: &mut Framebuffer) {
    g.gallery.backdrop(1).draw(fb, &BACKDROP, 300);
    ui::panel(fb, Rect::new(4, 4, 392, 292));
    ui::inset(fb, Rect::new(10, 22, 380, 250));
    let titles = ["Routes", "Creatures", "Moving", "Goods and workshops", "Keys"];
    fb.text_centered(&format!("Rules {} of {}: {}", g.help_page + 1, HELP_PAGES, titles[g.help_page]), 200, 9, GOLD, Some(OUTLINE), 1);
    let (x, mut y) = (18, 30);
    let mut para = |fb: &mut Framebuffer, s: &str, col: u8, y: &mut i32| {
        let n = fb.text_wrapped(s, x, *y, 364, 10, col, None);
        *y += n * 10 + 5;
    };
    match g.help_page {
        0 => {
            // A route, drawn: pickup, loaded trip, drop-off, trip back.
            let (ax, bx, ry) = (60, 300, 60);
            draw_source(fb, &g.art, Good::Twig, ax, ry, 4);
            for k in 0..3 {
                fb.rect_fill(bx + k * 8, ry - 10 + k * 2, 8, 20 - k * 2, ROCK);
            }
            fb.text("pickup", ax - 8, ry + 13, TEXT_DIM, None, 1);
            fb.text("drop-off", bx - 8, ry + 13, TEXT_DIM, None, 1);
            for k in (ax + 16..bx - 6).step_by(3) {
                fb.pset(k, ry - 2, GOLD);
                fb.pset(k, ry + 8, ACCENT);
            }
            blit_creature(fb, g, Species::Tribble, 0, 150, ry - 4, false, Some(Good::Twig));
            blit_creature(fb, g, Species::Tribble, 1, 210, ry + 6, true, None);
            y = 100;
            para(fb, "A route is a creature going back and forth between two places. It picks up one thing, carries it over, hands it over and comes back empty.", TEXT, &mut y);
            para(fb, "Gold dots show the trip there, blue dots the trip back. Loaded and empty, a creature can jump different heights, so the way back is often different.", TEXT, &mut y);
            para(fb, "Pick a creature, click where it picks up, then where it drops off. Click a place to add or remove creatures.", TEXT, &mut y);
            para(fb, "Each place has one dock (the fortress has two). Picking up or handing over takes 2 ticks, so a crowd queues.", TEXT_DIM, &mut y);
        }
        1 => {
            for s in SPECIES {
                let spr = g.art.creature(s, 0);
                fb.blit(spr, x, y + 10 - spr.h, false);
                fb.text(s.title(), x + 14, y + 1, GOLD, None, 1);
                icon_num(fb, &g.art.seed, x + 70, y + 1, &format!("{}", s.cost()), TEXT);
                fb.text(s.blurb(), x + 110, y + 1, TEXT_DIM, None, 1);
                jump_table(fb, g, s, x + 14, y + 12);
                y += 56;
            }
            para(fb, "Green bars: how many tiles up a jump goes, for each load. Pace: ticks per tile walked. Climbing is twice as slow. Falling is quick for everyone.", TEXT, &mut y);
        }
        2 => {
            para(fb, "Walk along the ground. Step off an edge and you fall, as far as it goes.", TEXT, &mut y);
            para(fb, "A jump rises up to its height, carries sideways up to one tile more than that, then falls. It lands on the first ledge it passes over.", TEXT, &mut y);
            let (dx, dy) = (150, y);
            for k in 0..8 {
                fb.rect_fill(dx + k * 8, dy + 32, 8, 8, SOIL);
            }
            fb.rect_fill(dx + 40, dy + 16, 24, 16, SOIL);
            fb.rect_fill(dx + 40, dy + 16, 24, 2, GRASS);
            fb.rect_fill(dx, dy + 32, 40, 2, GRASS);
            fb.line(dx + 12, dy + 28, dx + 12, dy + 12, GOLD);
            fb.line(dx + 12, dy + 12, dx + 44, dy + 12, GOLD);
            fb.blit(g.art.creature(Species::Dodo, 0), dx + 8, dy + 22, false);
            fb.blit(g.art.creature(Species::Dodo, 2), dx + 44, dy + 6, false);
            y += 46;
            let rows: [(Tool, &str); 3] = [
                (Tool::Infra(Infra::Spring), "Springboard: jump 2 higher from it, even with a stone."),
                (Tool::Infra(Infra::Ladder), "Ladder: apes climb it with anything, others only light."),
                (Tool::Infra(Infra::Plank), "Plank, branch, ledge: stand on it, jump up through it."),
            ];
            for (t, line) in rows {
                let f = Rect::new(x, y, 20, 20);
                ui::button_face(fb, f, false, false);
                tool_icon(fb, g, t, f);
                fb.text(line, x + 26, y + 6, TEXT, None, 1);
                y += 23;
            }
            para(fb, "Eggs and pots are fragile: whoever carries one never falls more than a tile. Clouds hold up light loads only. Nobody swims.", BAD, &mut y);
        }
        3 => {
            let mut col = 0;
            for gd in GOODS {
                let cx = x + col * 186;
                let s = g.art.good(gd);
                fb.blit(s, cx, y + (8 - s.h) / 2, false);
                let fr = if gd.fragile() { ", fragile" } else { "" };
                fb.text(&format!("{}: {}{fr}", cap(gd.word()), gd.weight().word()), cx + 12, y, TEXT, None, 1);
                col += 1;
                if col == 2 {
                    col = 0;
                    y += 12;
                }
            }
            y += 10;
            for k in KINDS {
                let f = Rect::new(x, y, 20, 20);
                ui::button_face(fb, f, false, false);
                tool_icon(fb, g, Tool::Build(k), f);
                fb.text(k.title(), x + 26, y + 1, GOLD, None, 1);
                icon_num(fb, &g.art.seed, x + 80, y + 1, &format!("{}", k.cost()), TEXT);
                if k == Kind::Depot {
                    fb.text("holds any 6 goods", x + 26, y + 11, TEXT_DIM, None, 1);
                } else {
                    recipe(fb, g, k, x + 26, y + 11);
                }
                y += 25;
            }
            para(fb, "A workshop starts by itself whenever it has what it needs. It holds 4 of each input and 3 of what it makes.", TEXT_DIM, &mut y);
        }
        _ => {
            let rows = [
                ("Left click", "use the tool, or pick a place"),
                ("Right click", "take away what you built, or cancel"),
                ("1 to 9", "pick a tool"),
                ("X", "the eraser"),
                ("Space", "run or pause"),
                ("S", "one tick at a time"),
                ("Tab", "speed"),
                ("R", "back to tick 0"),
                ("Z", "undo"),
                ("V", "show every path, or only the chosen place's"),
                ("I", "the site card again"),
                ("Esc", "cancel, then back to the map"),
                ("F1 F2 F3", "rules, scanlines, music"),
            ];
            for (k, v) in rows {
                fb.text(k, x, y, GOLD, None, 1);
                fb.text(v, x + 90, y, TEXT, None, 1);
                y += 12;
            }
        }
    }
    let (mx, my) = g.mouse;
    if g.help_page > 0 {
        ui::button(fb, R_HELP_PREV, "< Back", R_HELP_PREV.contains(mx, my), false);
    }
    if g.help_page + 1 < HELP_PAGES {
        ui::button(fb, R_HELP_NEXT, "Next >", R_HELP_NEXT.contains(mx, my), false);
    }
    ui::button(fb, R_HELP_CLOSE, "Close", R_HELP_CLOSE.contains(mx, my), false);
}
