//! Everything drawn: the side-view world, the colony, the panel, overlays and menus.

use crate::city::{Building, Job, Note, Order, Place};
use crate::framebuffer::{Framebuffer, Sprite};
use crate::game::*;
use crate::level::{Challenge, Goal, Tile};
use crate::palette::*;
use crate::path::{Move, FAR};
use crate::rules::*;
use crate::ui::{self, Rect};
use crate::world::World;

pub fn draw(g: &Game, fb: &mut Framebuffer) {
    fb.reset_view();
    match g.screen {
        Screen::Title => draw_title(g, fb),
        Screen::Select => draw_select(g, fb),
        Screen::Help => draw_help(g, fb),
        Screen::Play => draw_play(g, fb),
    }
    fb.reset_view();
}

fn hash(x: i32, y: i32) -> u32 {
    let mut h = (x as u32).wrapping_mul(374761393) ^ (y as u32).wrapping_mul(668265263);
    h = (h ^ (h >> 13)).wrapping_mul(1274126177);
    h ^ (h >> 16)
}

pub fn good_color(g: Good) -> u8 {
    match g {
        Good::Berry => BERRY,
        Good::Nut => SOIL_HI,
        Good::Fish => ROCK_HI,
        Good::Twig => WOOD,
        Good::Stone => ROCK,
        Good::Clay => CLAY,
        Good::Wicker => WOOD_HI,
        Good::Pot => POT,
        Good::Brick => BRICK,
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

pub fn draw_terrain(fb: &mut Framebuffer, w: &World, x0: i32, x1: i32, y0: i32, y1: i32) {
    for y in y0.max(0)..y1.min(w.h) {
        for x in x0.max(0)..x1.min(w.w) {
            draw_tile(fb, w, x, y);
        }
    }
}

// ----- Nodes -------------------------------------------------------------------------------------


// ----- Sources and the fortress ------------------------------------------------------------------

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
        Good::Nut => {
            // A leafy crown with nuts hanging in it.
            fb.rect_fill(px + 1, py + 1, 8, 6, LEAF_DK);
            fb.rect_fill(px, py + 2, 10, 4, LEAF_DK);
            fb.rect_fill(px + 2, py + 1, 5, 2, LEAF);
            fb.rect_fill(px + 4, py + 7, 2, 3, WOOD_DK);
            for (k, (bx, by)) in [(2, 4), (6, 3), (4, 5), (7, 5), (1, 3), (5, 2), (3, 2), (8, 4)].iter().enumerate() {
                if (k as i32) < stock {
                    fb.rect_fill(px + bx, py + by, 2, 2, SOIL_HI);
                    fb.pset(px + bx, py + by + 1, WOOD_DK);
                }
            }
        }
        Good::Fish => {
            // A mooring post with ripples and a fish when there's one to catch.
            fb.rect_fill(px + 1, py + 2, 2, 8, WOOD_DK);
            fb.rect_fill(px + 1, py + 2, 2, 1, WOOD_HI);
            fb.line(px + 3, py + 3, px + 9, py + 6, WOOD_HI);
            fb.pset(px + 9, py + 7, ROCK_HI);
            for k in 0..stock.min(3) {
                let fx = px + 4 + (k % 2) * 3;
                let fy = py + 7 + (k / 2) * 2 - k;
                fb.rect_fill(fx, fy, 3, 2, ROCK_HI);
                fb.pset(fx + 3, fy, ROCK);
                fb.pset(fx + 3, fy + 1, ROCK);
            }
        }
        _ => {
            let s = art.good(good);
            fb.blit(s, px + (TILE - s.w) / 2, py + TILE - s.h, false);
        }
    }
}

/// The fortress's shape, in pixels relative to its footprint's top-left (y may go above 0 for the
/// crest and tower tops): 0 = outside, 1 = stone, 2 = window or gate (dark), 3 = crest.

fn fortress_shape(x: i32, y: i32, pw: i32, ph: i32) -> u8 {
    if x < 0 || x >= pw {
        return 0;
    }
    let tw = 9;
    let tower = x < tw || x >= pw - tw;
    let keep_top = 6;
    // Crest: a bird with spread wings on top of the keep, centred.
    let cx = pw / 2;
    let dx = (x - cx).abs();
    if (-9..keep_top).contains(&y) && dx <= 9 {
        // Wings rise outward in a V; a narrow body with a head on top and a fanned tail below.
        let wing = dx >= 2 && y >= -1 - dx * 2 / 3 && y <= 1 - dx * 2 / 3;
        let body = dx <= 1 && (-7..keep_top).contains(&y);
        let head = dx == 0 && y == -8;
        let tail = dx <= 2 && (3..keep_top).contains(&y);
        let wing = wing || head || tail;
        if wing || body {
            return 3;
        }
    }
    if tower {
        if y < -4 || y >= ph {
            return 0;
        }
        if y < 0 {
            // Merlons.
            let lx = if x < tw { x } else { x - (pw - tw) };
            return if lx % 4 < 2 { 1 } else { 0 };
        }
        let lx = if x < tw { x } else { x - (pw - tw) };
        if (3..6).contains(&lx) && (4..8).contains(&y) {
            return 2;
        }
        return 1;
    }
    if y < keep_top || y >= ph {
        return 0;
    }
    // Gate arch.
    let gw = 5;
    if dx <= gw / 2 + 1 && y >= ph - 10 && !(y == ph - 10 && dx == gw / 2 + 1) {
        return 2;
    }
    // A row of slit windows.
    if y >= keep_top + 4 && y < keep_top + 7 && (x - tw) % 6 == 3 {
        return 2;
    }
    1
}

fn draw_fortress(fb: &mut Framebuffer, g: &Game) {
    let lv = g.level();
    let (fx, fy, fw, fh) = lv.fortress;
    let (px, py, pw, ph) = (fx * TILE, fy * TILE, fw * TILE, fh * TILE);
    let (got, total) = g.city.fortress_progress();
    let done = g.city.fortress_done();
    // Stone rises from the ground as goods arrive; the rest is a faint blueprint.
    let top = -9;
    let span = ph - top;
    let built_from = ph - span * got / total;
    for y in top..ph {
        for x in 0..pw {
            let s = fortress_shape(x, y, pw, ph);
            if s == 0 {
                continue;
            }
            let edge = [(1, 0), (-1, 0), (0, 1), (0, -1)].iter().any(|(dx, dy)| fortress_shape(x + dx, y + dy, pw, ph) == 0);
            let (sx, sy) = (px + x, py + y);
            if y >= built_from {
                let c = match s {
                    2 => PANEL_DEEP,
                    3 => {
                        if done {
                            if edge { OUTLINE } else if (x + y) % 3 == 0 { WHITE } else { GOLD }
                        } else if edge {
                            OUTLINE
                        } else {
                            ROCK_HI
                        }
                    }
                    _ => {
                        if edge {
                            OUTLINE
                        } else {
                            let row = (y - top) / 3;
                            let mortar = (y - top) % 3 == 2 || (x + if row % 2 == 0 { 0 } else { 3 }) % 6 == 0;
                            if mortar { ROCK_DK } else if (y - top) % 3 == 0 { ROCK_HI } else { ROCK }
                        }
                    }
                };
                fb.pset(sx, sy, c);
            } else if edge && (x + y) % 2 == 0 {
                // Blueprint outline: a dotted ghost of what's to come.
                fb.pset(sx, sy, ACCENT_HI);
            }
        }
    }
    // Scaffold while it's going up.
    if !done {
        for x in [1, pw - 2] {
            fb.rect_fill(px + x, py + built_from.max(0) - 2, 1, ph - built_from.max(0) + 2, WOOD_DK);
        }
        if built_from < ph {
            fb.rect_fill(px, py + built_from.max(top) - 1, pw, 1, WOOD);
        }
    } else {
        // Banners on both towers.
        for bx in [3, pw - 6] {
            fb.rect_fill(px + bx, py - 12, 1, 8, WOOD_DK);
            fb.rect_fill(px + bx + 1, py - 12, 4, 3, BANNER);
        }
    }
    // What the current stage still wants, above.
    if let Some(st) = lv.stages.get(g.city.stage) {
        let mut x = px;
        let y = py - 20;
        for (gd, n) in &st.need {
            let have = g.city.fort_stock[gd.index()].min(*n);
            let spr = g.art.good(*gd);
            fb.blit(spr, x, y + (8 - spr.h) / 2, false);
            let t = format!("{have}/{n}");
            fb.text(&t, x + spr.w + 1, y, if have >= *n { GOOD } else { TEXT }, Some(OUTLINE), 1);
            x += spr.w + 3 + Framebuffer::text_width(&t, 1);
        }
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


// ----- Buildings ---------------------------------------------------------------------------------

/// A home: its walls and roof show the tier, its door the species.
fn draw_home(fb: &mut Framebuffer, s: Species, tier: usize, px: i32, py: i32) {
    let (wall, wall_dk, roof, roof_hi) = match tier {
        0 => (WOOD, WOOD_DK, LEAF_DK, LEAF),
        1 => (WOOD_HI, WOOD, BEAK, GOLD),
        2 => (ROCK, ROCK_DK, BRICK, CLAY),
        _ => (ROCK_HI, ROCK, BANNER, GOLD),
    };
    match s {
        Species::Tribble => {
            // A round burrow mound with a hut front.
            fb.rect_fill(px + 1, py + 9, 18, 11, wall);
            fb.rect_fill(px + 3, py + 6, 14, 3, wall);
            fb.rect_fill(px + 6, py + 4, 8, 2, wall);
            for x in (3..18).step_by(5) {
                fb.rect_fill(px + x, py + 10, 1, 9, wall_dk);
            }
            fb.rect_fill(px + 3, py + 5, 14, 1, roof_hi);
            fb.rect_fill(px + 1, py + 8, 18, 1, roof);
            fb.rect_fill(px + 6, py + 3, 8, 1, roof);
            fb.rect_outline(px + 1, py + 8, 18, 12, OUTLINE);
            fb.rect_fill(px + 3, py + 4, 1, 4, OUTLINE);
            fb.rect_fill(px + 16, py + 4, 1, 4, OUTLINE);
            fb.rect_fill(px + 5, py + 2, 10, 1, OUTLINE);
            // Round door.
            fb.rect_fill(px + 7, py + 13, 6, 7, PANEL_DEEP);
            fb.rect_fill(px + 8, py + 12, 4, 1, PANEL_DEEP);
            fb.pset(px + 7, py + 13, wall);
            fb.pset(px + 12, py + 13, wall);
            fb.rect_fill(px + 8, py + 19, 4, 1, TRIB);
        }
        Species::Dodo => {
            // A woven nest-house: a wide bowl under a pointed roof.
            fb.rect_fill(px + 2, py + 8, 16, 12, wall);
            for y in (9..20).step_by(3) {
                fb.rect_fill(px + 3, py + y, 14, 1, wall_dk);
            }
            fb.rect_outline(px + 2, py + 8, 16, 12, OUTLINE);
            for k in 0..7 {
                let inset = 7 - k;
                fb.rect_fill(px + inset, py + 1 + k, 20 - inset * 2, 1, if k < 2 { roof_hi } else { roof });
                fb.pset(px + inset, py + 1 + k, OUTLINE);
                fb.pset(px + 19 - inset, py + 1 + k, OUTLINE);
            }
            fb.rect_fill(px + 7, py, 6, 1, OUTLINE);
            fb.rect_fill(px + 8, py + 13, 4, 7, PANEL_DEEP);
            fb.rect_fill(px + 8, py + 12, 4, 1, DODO);
        }
        Species::Ape => {
            // A lodge on stilts with a ladder up.
            fb.rect_fill(px + 1, py + 4, 18, 10, wall);
            for x in (3..18).step_by(4) {
                fb.rect_fill(px + x, py + 5, 1, 8, wall_dk);
            }
            fb.rect_outline(px + 1, py + 4, 18, 10, OUTLINE);
            fb.rect_fill(px, py + 1, 20, 3, roof);
            fb.rect_fill(px, py + 1, 20, 1, roof_hi);
            fb.rect_outline(px, py, 20, 4, OUTLINE);
            for x in [2, 16] {
                fb.rect_fill(px + x, py + 14, 2, 6, WOOD_DK);
            }
            fb.rect_fill(px + 8, py + 7, 4, 7, PANEL_DEEP);
            fb.rect_fill(px + 8, py + 6, 4, 1, APE);
            for y in [15, 17, 19] {
                fb.rect_fill(px + 8, py + y, 4, 1, WOOD);
            }
        }
    }
    if tier >= 2 {
        // A chimney.
        fb.rect_fill(px + 15, py - 2, 3, 5, BRICK);
        fb.rect_outline(px + 15, py - 2, 3, 5, OUTLINE);
    }
    if tier >= 3 {
        fb.rect_fill(px + 3, py - 6, 1, 7, WOOD_DK);
        fb.rect_fill(px + 4, py - 6, 4, 3, BANNER);
    }
}

fn hut_store(fb: &mut Framebuffer, px: i32, py: i32) {
    hut(fb, px, py, WOOD, WOOD_DK, WOOD_DK, WOOD_HI);
    // Crates by the door.
    fb.rect_fill(px + 13, py + 15, 5, 5, WOOD_HI);
    fb.rect_outline(px + 13, py + 15, 5, 5, OUTLINE);
}

fn draw_building(fb: &mut Framebuffer, g: &Game, b: &Building) {
    let (px, py) = (b.x * TILE, b.y * TILE);
    match b.kind {
        Kind::Burrow | Kind::Nest | Kind::Lodge => draw_home(fb, b.species, b.tier, px, py),
        Kind::Store => hut_store(fb, px, py),
        Kind::Mason => hut(fb, px, py, ROCK, ROCK_DK, SOIL_DK, ROCK_HI),
        Kind::Weaver => hut(fb, px, py, WOOD_HI, WOOD, BEAK, GOLD),
        Kind::Kiln => {
            fb.rect_fill(px + 2, py + 6, 16, 14, BRICK);
            fb.rect_fill(px + 4, py + 3, 12, 3, BRICK);
            fb.rect_fill(px + 6, py + 1, 8, 2, BRICK);
            fb.rect_fill(px + 6, py + 1, 8, 1, POT);
            fb.rect_outline(px + 2, py + 6, 16, 14, OUTLINE);
            fb.rect_fill(px + 8, py + 13, 4, 7, PANEL_DEEP);
            if b.progress > 0 {
                fb.rect_fill(px + 9, py + 16, 2, 3, GOLD);
                fb.pset(px + 9, py + 15, BAD);
            }
        }
        Kind::Garden => {
            fb.rect_fill(px + 1, py + 8, 8, 2, GRASS_DK);
            for (k, (fx, fy)) in [(2, 5), (5, 3), (7, 6), (4, 7)].iter().enumerate() {
                fb.rect_fill(px + fx, py + fy + 1, 1, 8 - fy, GRASS);
                let c = [BERRY, GOLD, WHITE, BANNER][k];
                fb.pset(px + fx, py + fy, c);
                fb.pset(px + fx - 1, py + fy, c);
                fb.pset(px + fx + 1, py + fy, c);
                fb.pset(px + fx, py + fy - 1, c);
            }
        }
        Kind::Camp => {
            // A pennant on a pole, in the colours of whoever works it.
            let (hi, dk) = species_colors(b.species);
            fb.rect_fill(px + 8, py, 1, 10, WOOD_DK);
            fb.rect_fill(px + 9, py, 1, 1, OUTLINE);
            fb.rect_fill(px + 9, py + 1, 4, 3, hi);
            fb.rect_fill(px + 9, py + 4, 2, 1, dk);
            fb.rect_outline(px + 9, py, 5, 5, OUTLINE);
            fb.rect_fill(px + 9, py + 1, 4, 3, hi);
        }
    }
    if let Some(r) = b.kind.recipe() {
        let s = g.art.good(r.output);
        fb.blit(s, px + 13, py + 9, false);
        if b.progress > 0 {
            let w = 16 * b.progress / r.ticks;
            fb.rect_fill(px + 2, py + 8, 16, 1, PANEL_DEEP);
            fb.rect_fill(px + 2, py + 8, w, 1, GOLD);
        }
    }
    if b.kind.slots() > 0 && b.kind != Kind::Camp {
        // Who works here: a species-coloured stripe under the eaves.
        let (hi, _) = species_colors(b.species);
        fb.rect_fill(px + 2, py + 7, 4, 1, hi);
    }
    // Stock pips: what's inside.
    if b.kind == Kind::Store || b.kind.recipe().is_some() {
        let mut pips: Vec<u8> = Vec::new();
        for gd in GOODS {
            for _ in 0..b.stock[gd.index()] {
                pips.push(good_color(gd));
            }
        }
        let per_row = if b.kind == Kind::Store { 4 } else { 3 };
        let max = if b.kind == Kind::Store { 16 } else { 6 };
        for (i, c) in pips.iter().take(max).enumerate() {
            let x = px + 2 + (i as i32 % per_row) * 3;
            let y = py + 9 + (i as i32 / per_row) * 3;
            fb.rect_fill(x, y, 2, 2, *c);
        }
    }
    draw_need_mark(fb, g, b);
}

/// A calm marker over a building that's stuck: the thing it lacks, in a red-edged box.
fn draw_need_mark(fb: &mut Framebuffer, g: &Game, b: &Building) {
    let (w, _) = b.kind.size();
    let (cx, top) = (b.x * TILE + w * TILE / 2, b.y * TILE - 2);
    let good = match b.note {
        Note::Hungry if b.residents > 0 => Some(Good::Berry),
        Note::Lacks(gd) => Some(gd),
        Note::NeedInput(gd) => Some(gd),
        Note::Stuck(gd, ..) => Some(gd),
        _ => None,
    };
    if let Some(gd) = good {
        let s = g.art.good(gd);
        let (x, y) = (cx - s.w / 2, top - s.h - 2);
        let edge = if matches!(b.note, Note::Stuck(..)) { ACCENT } else { BAD };
        fb.rect_fill(x - 2, y - 2, s.w + 4, s.h + 4, PANEL_DEEP);
        fb.rect_outline(x - 2, y - 2, s.w + 4, s.h + 4, edge);
        fb.blit(s, x, y, false);
        if matches!(b.note, Note::Stuck(..)) {
            fb.line(x - 1, y + s.h, x + s.w, y - 1, BAD);
        }
        return;
    }
    let who = match b.note {
        Note::NoWorkers(s) | Note::NoReach(s) => Some(s),
        Note::NoSettlers => Some(b.species),
        _ => None,
    };
    if let Some(s) = who {
        let spr = g.art.creature(s, 0);
        let (x, y) = (cx - spr.w / 2, top - spr.h - 2);
        fb.rect_fill(x - 2, y - 2, spr.w + 4, spr.h + 4, PANEL_DEEP);
        fb.rect_outline(x - 2, y - 2, spr.w + 4, spr.h + 4, BAD);
        fb.blit(spr, x, y, false);
        fb.text("?", x + spr.w - 2, y - 3, BAD, Some(OUTLINE), 1);
    }
}

fn draw_entrance(fb: &mut Framebuffer, g: &Game) {
    let (ex, ey) = g.level().entrance;
    signpost(fb, ex * TILE, ey * TILE);
}

fn signpost(fb: &mut Framebuffer, px: i32, py: i32) {
    fb.rect_fill(px + 4, py + 2, 1, 8, WOOD_DK);
    fb.rect_fill(px + 1, py + 1, 8, 4, WOOD_HI);
    fb.rect_outline(px + 1, py + 1, 8, 4, OUTLINE);
    fb.pset(px + 3, py + 3, OUTLINE);
    fb.pset(px + 5, py + 3, OUTLINE);
}

fn draw_world_things(fb: &mut Framebuffer, g: &Game) {
    let lv = g.level();
    for (i, s) in lv.sources.iter().enumerate() {
        let stock = if s.infinite() { 4 } else { g.city.src_stock[i] };
        draw_source(fb, &g.art, s.good, s.x * TILE, s.y * TILE, stock);
    }
    if lv.has_fortress() {
        draw_fortress(fb, g);
    }
    draw_entrance(fb, g);
    for b in &g.city.buildings {
        draw_building(fb, g, b);
    }
}

// ----- Creatures ---------------------------------------------------------------------------------

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
    let mut seen: Vec<((i32, i32), i32)> = Vec::new();
    for c in &g.city.creatures {
        let (mut px, mut py) = (c.x * TILE, c.y * TILE);
        let mut frame = 0;
        let mut cargo = c.cargo;
        match c.job {
            Job::Idle | Job::Wait(_) if c.cargo.is_none() => continue,
            Job::Fetch(_) => cargo = None,
            _ => {}
        }
        let at_work = matches!(c.job, Job::Harvest(_) | Job::Wait(_) | Job::Drop(..) | Job::Idle);
        if c.walking() {
            if let Some(st) = c.path.steps.get(c.step) {
                px += (st.x - c.x) * TILE * c.prog / st.t.max(1);
                py += (st.y - c.y) * TILE * c.prog / st.t.max(1);
                frame = match st.m {
                    Move::Walk => ((c.x + c.step as i32 + if c.prog * 2 >= st.t { 1 } else { 0 }) % 2) as usize,
                    Move::Rise | Move::Leap | Move::Fall => 2,
                    Move::Climb => 3,
                };
            }
        } else if at_work {
            // Workers at the same spot shuffle aside so they stay countable.
            let k = seen.iter().find(|(p, _)| *p == (c.x, c.y)).map(|(_, n)| *n).unwrap_or(0);
            if let Some(e) = seen.iter_mut().find(|(p, _)| *p == (c.x, c.y)) {
                e.1 += 1;
            } else {
                seen.push(((c.x, c.y), 1));
            }
            px += [0, -3, 3, -6, 6][(k as usize).min(4)];
            if let Job::Harvest(t) = c.job {
                frame = (t / 4 % 2) as usize;
            }
        }
        blit_creature(fb, g, c.species, frame, px, py, c.facing < 0, cargo);
        if c.job == Job::Settle {
            // Settlers carry a little bundle.
            fb.rect_fill(px + 3, py - 1, 4, 3, WOOD_HI);
            fb.rect_outline(px + 3, py - 1, 4, 3, OUTLINE);
        }
    }
}

// ----- Overlays in the world ---------------------------------------------------------------------

/// Dots on every tile a species can walk to within delivery range.
fn draw_reach_dots(fb: &mut Framebuffer, g: &Game, f: &crate::path::Field, s: Species, limit: i32) {
    let w = &g.city.world;
    let (hi, dk) = species_colors(s);
    let (x0, x1) = (g.cam.0 / TILE, (g.cam.0 + VIEW_W) / TILE + 1);
    let (y0, y1) = (g.cam.1 / TILE, (g.cam.1 + VIEW_H) / TILE + 1);
    for y in y0.max(0)..y1.min(w.h) {
        for x in x0.max(0)..x1.min(w.w) {
            let d = f.at(w, x, y);
            if d == FAR || d > limit {
                continue;
            }
            if !(w.standable_w(x, y, Weight::Empty) || w.climbable(x, y, s, Weight::Empty)) {
                continue;
            }
            // An outlined dot at standing height: bright within half the range, dark beyond.
            let (px, py) = (x * TILE + 3, y * TILE + 4);
            fb.rect_fill(px, py, 4, 4, OUTLINE);
            fb.rect_fill(px + 1, py + 1, 2, 2, if d <= limit / 2 { hi } else { dk });
        }
    }
}

fn draw_reach(fb: &mut Framebuffer, g: &Game) {
    if g.overlay_open() {
        return;
    }
    // A workplace about to be placed.
    if let (Some(k), Some((tx, ty))) = (g.tool.workplace(), g.hovered_tile()) {
        let (x, y) = g.place_at(k, tx, ty);
        if k == Kind::Camp || g.city.world.can_build(k, x, y).is_ok() {
            let f = g.ghost_reach(k, x, y, g.hire);
            draw_reach_dots(fb, g, &f, g.hire, RANGE);
        }
        return;
    }
    // The chosen workplace.
    if let Some(Sel::B(id)) = g.sel {
        if let Some(b) = g.city.building(id) {
            if b.kind.slots() > 0 {
                let f = g.city.field(Place::B(id), b.species, Load::EMPTY);
                draw_reach_dots(fb, g, &f, b.species, RANGE);
            }
        }
    }
}

fn draw_ghost(fb: &mut Framebuffer, g: &Game) {
    if g.overlay_open() {
        return;
    }
    let Some((tx, ty)) = g.hovered_tile() else { return };
    if let Some((r, _)) = g.card() {
        if r.contains(g.mouse.0, g.mouse.1) {
            return;
        }
    }
    match g.tool {
        Tool::Build(k) => {
            let (x, y) = g.place_at(k, tx, ty);
            let (w, h) = k.size();
            let mut ok = g.city.world.can_build(k, x, y).is_ok() && g.city.can_afford(k.cost());
            if let Some(s) = k.resident() {
                ok = ok && g.settlers_can_reach(k, x, y, s);
            }
            let c = if ok { GOOD } else { BAD };
            if ok && k.is_home() {
                draw_home(fb, k.resident().unwrap(), 0, x * TILE, y * TILE);
                fb.rect_checker(x * TILE, y * TILE, w * TILE, h * TILE, SHADOW, 0);
            }
            fb.corners(x * TILE, y * TILE, w * TILE, h * TILE, 4, c);
            // Pleasantness changes show for noisy buildings and gardens.
            let aura = if k == Kind::Camp {
                g.level().source_at(x, y).map(|i| camp_aura(g.level().sources[i].good)).unwrap_or(0)
            } else {
                k.aura()
            };
            if aura != 0 && g.level().kinds.contains(&Kind::Garden) {
                let col = if aura > 0 { GOOD } else { BAD };
                fb.rect_hatch((x - AURA) * TILE, (y - AURA) * TILE, (w + 2 * AURA) * TILE, (h + 2 * AURA) * TILE, col);
            }
        }
        Tool::Infra(i) => {
            let (ok, h) = match i {
                Infra::Spring => (g.city.world.can_spring(tx, ty).is_ok(), 1),
                Infra::Ladder => match g.city.world.can_ladder(tx, ty) {
                    Ok(n) => (true, n),
                    Err(_) => (false, 1),
                },
                Infra::Plank => (g.city.world.can_plank(tx, ty).is_ok(), 1),
            };
            let c = if ok { GOOD } else { BAD };
            fb.corners(tx * TILE, ty * TILE, 10, 10 * h, 3, c);
            if ok && i == Infra::Ladder {
                fb.rect_checker(tx * TILE + 2, ty * TILE, 6, 10 * h, WOOD_HI, 0);
            }
        }
        Tool::Demolish => fb.corners(tx * TILE, ty * TILE, 10, 10, 3, BAD),
        Tool::Select => {
            if let Some(s) = g.hovered_sel() {
                if let Some((x, y, w, h)) = g.sel_box(s) {
                    fb.corners(x * TILE - 1, y * TILE - 1, w * TILE + 2, h * TILE + 2, 3, ACCENT_HI);
                }
            }
        }
    }
}

fn draw_floaters(fb: &mut Framebuffer, g: &Game) {
    for f in &g.floaters {
        let w = Framebuffer::text_width(&f.text, 1);
        fb.text(&f.text, f.x - w / 2, f.y - 10, f.col, Some(OUTLINE), 1);
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

/// A good's icon with a count after it; returns the x after it.
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
    good_count(fb, g, r.output, 1, cx, y, TEXT)
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
    if s.climbs_vine() {
        fb.text("climbs vines", x + 120, y + 24, ACCENT, None, 1);
    }
}

/// A small green check mark, 7 wide and 6 tall.
fn check(fb: &mut Framebuffer, x: i32, y: i32, col: u8) {
    for (dx, dy) in [(0, 3), (1, 4), (2, 5), (3, 4), (4, 3), (5, 2), (6, 1), (6, 0)] {
        fb.rect_fill(x + dx, y + dy, 1, 1, col);
        fb.pset(x + dx, y + dy + 1, OUTLINE);
    }
}

/// The announcement across the top of the map when a goal is met.
fn draw_banner(fb: &mut Framebuffer, g: &Game) {
    let Some((title, line, _)) = &g.banner else { return };
    let w = Framebuffer::text_width(title, 1).max(Framebuffer::text_width(line, 1)) + 24;
    let h = if line.is_empty() { 18 } else { 28 };
    let x = 200 - w / 2;
    let y = 22;
    fb.rect_fill(x, y, w, h, OUTLINE);
    fb.rect_fill(x + 1, y + 1, w - 2, h - 2, PANEL_DEEP);
    fb.rect_fill(x + 1, y + 1, w - 2, 1, GOLD);
    fb.rect_fill(x + 1, y + h - 2, w - 2, 1, GOLD);
    fb.text_centered(title, 200, y + 5, GOLD, Some(OUTLINE), 1);
    if !line.is_empty() {
        fb.text_centered(line, 200, y + 16, TEXT, None, 1);
    }
}

fn medal_icon(fb: &mut Framebuffer, x: i32, y: i32, m: usize, big: bool) {
    let (c, hi) = match m {
        0 => (GOLD, WHITE),
        1 => (ROCK_HI, WHITE),
        2 => (CLAY, POT),
        _ => (PANEL_DEEP, TEXT_FAINT),
    };
    let r = if big { 7 } else { 3 };
    for dy in -r..=r {
        for dx in -r..=r {
            let d = dx * dx + dy * dy;
            if d <= r * r {
                let col = if d > (r - 1) * (r - 1) { OUTLINE } else if dx + dy < -r / 2 { hi } else { c };
                fb.pset(x + dx, y + dy, col);
            }
        }
    }
}

fn tool_icon(fb: &mut Framebuffer, g: &Game, t: Tool, r: Rect) {
    let (cx, cy) = (r.x + r.w / 2, r.y + r.h / 2);
    match t {
        Tool::Select => {
            let a = &g.art.pointer;
            fb.blit(a, r.x + (r.w - a.w) / 2 + 1, r.y + (r.h - a.h) / 2, false);
        }
        Tool::Build(k) if k.is_home() => {
            // A small home: walls, roof and the species at the door.
            let spr = g.art.creature(k.resident().unwrap(), 0);
            fb.rect_fill(cx - 7, cy - 2, 14, 9, WOOD);
            fb.rect_outline(cx - 7, cy - 2, 14, 9, OUTLINE);
            for k2 in 0..4 {
                fb.rect_fill(cx - 8 + k2 + 1, cy - 3 - k2, 14 - k2 * 2, 1, LEAF_DK);
            }
            fb.blit(spr, cx - spr.w / 2, cy + 7 - spr.h, false);
        }
        Tool::Build(Kind::Camp) => {
            fb.rect_fill(cx - 2, cy - 7, 1, 14, WOOD_DK);
            fb.rect_fill(cx - 1, cy - 7, 6, 4, BANNER);
            fb.rect_outline(cx - 1, cy - 7, 6, 4, OUTLINE);
            draw_source(fb, &g.art, Good::Berry, cx - 9, cy - 3, 3);
        }
        Tool::Build(Kind::Garden) => {
            for (k, (fx, fy)) in [(-4, -1), (0, -4), (4, 0)].iter().enumerate() {
                fb.rect_fill(cx + fx, cy + fy + 1, 1, 7 - fy, GRASS);
                let c = [BERRY, GOLD, WHITE][k];
                fb.rect_fill(cx + fx - 1, cy + fy - 1, 3, 3, c);
            }
            fb.rect_fill(cx - 7, cy + 7, 14, 2, GRASS_DK);
        }
        Tool::Build(k) => {
            let (wall, roof) = match k {
                Kind::Store => (WOOD, WOOD_DK),
                Kind::Mason => (ROCK, SOIL_DK),
                Kind::Weaver => (WOOD_HI, BEAK),
                _ => (BRICK, BRICK),
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
            } else if k == Kind::Store {
                fb.rect_fill(cx + 3, cy + 2, 4, 4, WOOD_HI);
                fb.rect_outline(cx + 3, cy + 2, 4, 4, OUTLINE);
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
        Tool::Demolish => {
            fb.line(cx - 4, cy - 4, cx + 4, cy + 4, BAD);
            fb.line(cx - 4, cy + 4, cx + 4, cy - 4, BAD);
            fb.line(cx - 3, cy - 4, cx + 5, cy + 4, BAD);
            fb.line(cx - 3, cy + 4, cx + 5, cy - 4, BAD);
        }
    }
}

pub fn tool_label(t: Tool) -> String {
    match t {
        Tool::Select => "Look. Click anything to see what it's doing.".into(),
        Tool::Build(k) => format!("{}, {} seeds. {}", k.title(), k.cost(), k.blurb()),
        Tool::Infra(i) => format!("{}, {} seeds{}. {}", i.title(), i.cost(), if i == Infra::Ladder { " a rung" } else { "" }, i.blurb()),
        Tool::Demolish => "Demolish. Half the seeds come back.".into(),
    }
}

/// A cream speech bubble with a pointer toward (ax, ay) on screen.
pub fn bubble(fb: &mut Framebuffer, text: &str, ax: i32, ay: i32) {
    let lines = crate::framebuffer::wrap(text, 30);
    let w = lines.iter().map(|l| Framebuffer::text_width(l, 1)).max().unwrap_or(0) + 12;
    let h = lines.len() as i32 * 10 + 8;
    let ay = ay.clamp(14, VIEW_H - 4);
    let above = ay > h + 24;
    let x = (ax - w / 2).clamp(2, 398 - w);
    let y = if above { ay - h - 8 } else { ay + 10 };
    fb.rect_fill(x, y, w, h, OUTLINE);
    fb.rect_fill(x + 1, y + 1, w - 2, h - 2, EGG);
    fb.rect_fill(x + 1, y + h - 2, w - 2, 1, CLAY);
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

// ----- Header, minimap, panel -------------------------------------------------------------------

fn draw_minimap(fb: &mut Framebuffer, g: &Game) {
    let r = R_MINI;
    let lv = g.level();
    let w = &g.city.world;
    fb.rect_fill(r.x - 1, r.y - 1, r.w + 2, r.h + 2, OUTLINE);
    for my in 0..r.h {
        for mx in 0..r.w {
            let tx = mx * lv.w / r.w;
            let ty = my * lv.h / r.h;
            // Sample the tile and the one below so thin floors still show.
            let t = w.tile(tx, ty);
            let c = match w.occ(tx, ty) {
                crate::world::Occ::Building(id) => g.city.building(id).map(|b| if b.kind.is_home() { species_colors(b.species).0 } else { WOOD_HI }).unwrap_or(WOOD),
                crate::world::Occ::Fortress => GOLD,
                crate::world::Occ::Source(i) => good_color(lv.sources[i].good),
                _ => match t {
                    Tile::Earth => GRASS_DK,
                    Tile::Rock | Tile::Bramble => ROCK_DK,
                    Tile::Water => WATER,
                    Tile::Branch | Tile::Ledge | Tile::Trunk => WOOD_DK,
                    Tile::Leaves | Tile::Vine => LEAF_DK,
                    Tile::Cloud => ROCK_HI,
                    _ => PANEL_DEEP,
                },
            };
            fb.pset(r.x + mx, r.y + my, c);
        }
    }
    // The camera's window.
    let cx0 = r.x + g.cam.0 / TILE * r.w / lv.w;
    let cx1 = r.x + ((g.cam.0 + VIEW_W) / TILE * r.w / lv.w).min(r.w);
    let cy0 = r.y + g.cam.1 / TILE * r.h / lv.h;
    let cy1 = r.y + ((g.cam.1 + VIEW_H) / TILE * r.h / lv.h).min(r.h);
    fb.rect_outline(cx0, cy0, (cx1 - cx0).max(3), (cy1 - cy0).max(3), WHITE);
}

fn draw_header(fb: &mut Framebuffer, g: &Game) {
    fb.rect_checker(0, 0, 400, 12, SHADOW, 0);
    let c = &g.city;
    let x = icon_num(fb, &g.art.seed, 2, 3, &format!("{}", c.seeds()), GOLD);
    let x = x.max(36) + 4;
    let date = format!("D{} {}", c.day(), SEASONS[c.season()]);
    let season_col = [GRASS_HI, GOLD, BEAK, ACCENT_HI][c.season()];
    fb.text(&date, x, 3, season_col, Some(OUTLINE), 1);
    let mut x = x + 66;
    for s in g.level().species.clone() {
        let (hi, _) = species_colors(s);
        fb.rect_fill(x, 4, 5, 5, hi);
        fb.rect_outline(x - 1, 3, 7, 7, OUTLINE);
        let t = format!("{}/{}", c.employed(s), c.pop_of(s));
        fb.text(&t, x + 8, 3, TEXT, Some(OUTLINE), 1);
        x += 10 + Framebuffer::text_width(&t, 1) + 4;
    }
    if g.scrolls() {
        draw_minimap(fb, g);
    } else {
        let t = format!("{} {}", g.level().id, g.level().name);
        fb.text(&t, R_MINI.x + R_MINI.w - Framebuffer::text_width(&t, 1), 3, TEXT_DIM, Some(OUTLINE), 1);
    }
    let (mx, my) = g.mouse;
    ui::button(fb, R_HELP, "?", R_HELP.contains(mx, my), false);
    ui::button(fb, R_MENU, "Menu", R_MENU.contains(mx, my), false);
}

fn draw_panel(fb: &mut Framebuffer, g: &Game) {
    ui::panel(fb, Rect::new(0, VIEW_H, 400, 300 - VIEW_H));
    let (mx, my) = g.mouse;
    let tools = g.tools();
    let mut hover_label: Option<String> = None;
    for (i, &t) in tools.iter().enumerate() {
        let r = tool_rect(i);
        let hover = r.contains(mx, my);
        let afford = match t {
            Tool::Build(k) => g.city.can_afford(k.cost()),
            Tool::Infra(i) => g.city.can_afford(i.cost()),
            _ => true,
        };
        ui::button_face(fb, r, hover, g.tool == t);
        tool_icon(fb, g, t, r);
        if !afford {
            fb.rect_checker(r.x + 1, r.y + 1, r.w - 2, r.h - 2, PANEL_DEEP, 0);
        }
        if hover {
            let key = match t {
                Tool::Demolish => "X".to_string(),
                _ if i < 9 => (i + 1).to_string(),
                _ => String::new(),
            };
            hover_label = Some(if key.is_empty() { tool_label(t) } else { format!("[{key}] {}", tool_label(t)) });
        }
    }
    // Clock controls.
    let hov = |r: Rect| r.contains(mx, my);
    ui::button_face(fb, R_PLAY, hov(R_PLAY), !g.paused);
    let (cx, cy) = (R_PLAY.x + 10, R_PLAY.y + 10);
    if !g.paused {
        fb.rect_fill(cx - 4, cy - 4, 3, 9, TEXT);
        fb.rect_fill(cx + 1, cy - 4, 3, 9, TEXT);
    } else {
        for k in 0..5 {
            fb.rect_fill(cx - 3 + k, cy - 4 + k, 1, 9 - 2 * k, GOOD);
        }
    }
    ui::button(fb, R_SPEED, &format!("x{}", SPEEDS[g.speed]), hov(R_SPEED), false);
    ui::button_face(fb, R_HOME, hov(R_HOME), false);
    signpost(fb, R_HOME.x + 5, R_HOME.y + 6);
    if hov(R_PLAY) {
        hover_label = Some("Run or pause. Space does it too.".into());
    } else if hov(R_SPEED) {
        hover_label = Some("Speed. Tab does it too.".into());
    } else if hov(R_HOME) {
        hover_label = Some("Back to the trail. Arrows, the screen edge or the map above scroll.".into());
    }
    // Status.
    ui::inset(fb, R_STATUS);
    let (text, bad) = match hover_label {
        Some(t) => (t, false),
        None => g.status_line(),
    };
    let wrap_w = if g.tool.workplace().is_some() { R_STATUS.w - 74 } else { R_STATUS.w - 8 };
    let lines = crate::framebuffer::wrap(&text, (wrap_w / 6) as usize);
    for (i, l) in lines.iter().take(3).enumerate() {
        fb.text(l, R_STATUS.x + 4, R_STATUS.y + 3 + i as i32 * 10, if bad { BAD } else { TEXT }, None, 1);
    }
    if g.tool.workplace().is_some() {
        fb.text("hire", R_STATUS.x + R_STATUS.w - 64, R_STATUS.y + 17, TEXT_DIM, None, 1);
        fb.text("T D A", R_STATUS.x + R_STATUS.w - 34, R_STATUS.y + 17, TEXT_FAINT, None, 1);
        for (i, s) in SPECIES.iter().enumerate() {
            let r = hire_rect(i);
            let free = g.city.pop_of(*s) - g.city.employed(*s);
            ui::button_face(fb, r, r.contains(mx, my), g.hire == *s);
            let (hi, _) = species_colors(*s);
            fb.rect_fill(r.x + 3, r.y + 3, 5, 5, hi);
            fb.rect_outline(r.x + 2, r.y + 2, 7, 7, OUTLINE);
            fb.text(&format!("{}", free.max(0)), r.x + 10, r.y + 2, if free > 0 { TEXT } else { TEXT_FAINT }, None, 1);
        }
    }
    // Goals.
    ui::inset(fb, R_GOALS);
    let goals = g.level().goals.clone();
    let mut y = R_GOALS.y + 3;
    for gl in goals.iter().take(2) {
        let (t, met) = g.goal_text(*gl);
        let icon_x = R_GOALS.x + 4;
        match gl {
            Goal::Fortress => {
                fb.rect_fill(icon_x, y + 1, 7, 6, ROCK);
                fb.rect_outline(icon_x, y + 1, 7, 6, OUTLINE);
                fb.pset(icon_x + 1, y, OUTLINE);
                fb.pset(icon_x + 5, y, OUTLINE);
            }
            _ => fb.blit(&g.art.paw, icon_x, y, false),
        }
        fb.text(&t, R_GOALS.x + 14, y, if met { GOOD } else { TEXT }, None, 1);
        if met {
            check(fb, R_GOALS.x + R_GOALS.w - 11, y, GOOD);
        }
        y += 10;
    }
    let lv = g.level();
    if goals.len() > 2 {
        let (t, met) = g.goal_text(goals[2]);
        fb.blit(&g.art.paw, R_GOALS.x + 4, y, false);
        fb.text(&t, R_GOALS.x + 14, y, if met { GOOD } else { TEXT }, None, 1);
        if met {
            check(fb, R_GOALS.x + R_GOALS.w - 11, y, GOOD);
        }
    } else if lv.medals[0] > 0 {
        let day = g.city.day();
        let m = lv.medal_for(day);
        let target = lv.medals.get(m).copied().unwrap_or(0);
        medal_icon(fb, R_GOALS.x + 7, y + 4, m, false);
        let t = if m < 3 { format!("by day {target}") } else { "no medal now".to_string() };
        fb.text(&t, R_GOALS.x + 14, y, TEXT_DIM, None, 1);
    }
}


// ----- Inspector card ----------------------------------------------------------------------------

fn card_frame(fb: &mut Framebuffer, r: Rect) {
    fb.rect_fill(r.x, r.y, r.w, r.h, OUTLINE);
    fb.rect_fill(r.x + 1, r.y + 1, r.w - 2, r.h - 2, PANEL_DEEP);
    fb.rect_fill(r.x + 1, r.y + 1, r.w - 2, 1, PANEL_HI);
}

fn small_button(fb: &mut Framebuffer, r: Rect, label: &str, hover: bool, on: bool) {
    let col = ui::button_face(fb, r, hover, on);
    fb.text_centered(label, r.x + r.w / 2, r.y + 2, col, None, 1);
}

/// A row of goods: icon and count for each that's non-zero (or all, with `all`).
fn goods_row(fb: &mut Framebuffer, g: &Game, x: i32, y: i32, stock: &[i32; NGOODS], only: &[Good], all: bool) -> i32 {
    let mut cx = x;
    for gd in only {
        let n = stock[gd.index()];
        if n == 0 && !all {
            continue;
        }
        let s = g.art.good(*gd);
        fb.blit(s, cx, y + (8 - s.h) / 2, false);
        let t = format!("{n}");
        fb.text(&t, cx + s.w + 1, y, if n > 0 { TEXT } else { BAD }, None, 1);
        cx += s.w + 3 + Framebuffer::text_width(&t, 1) + 2;
    }
    cx
}

fn draw_card(fb: &mut Framebuffer, g: &Game) {
    if g.overlay_open() {
        return;
    }
    let Some((r, hots)) = g.card() else { return };
    let Some(sel) = g.sel else { return };
    card_frame(fb, r);
    let (mx, my) = g.mouse;
    let (x, mut y) = (r.x + 5, r.y + 4);
    match sel {
        Sel::Fortress => {
            fb.text("The fortress", x, y, GOLD, None, 1);
            let (got, total) = g.city.fortress_progress();
            fb.text(&format!("{got}/{total}"), r.x + r.w - 40, y, TEXT_DIM, None, 1);
            y += 12;
            match g.level().stages.get(g.city.stage) {
                Some(st) => {
                    fb.text(&format!("Stage {} of {}: {}", g.city.stage + 1, g.level().stages.len(), st.name), x, y, TEXT, None, 1);
                    y += 11;
                    for (gd, n) in &st.need {
                        let have = g.city.fort_stock[gd.index()].min(*n);
                        let s = g.art.good(*gd);
                        fb.blit(s, x, y + (8 - s.h) / 2, false);
                        fb.text(&format!("{have}/{n} {}", gd.words()), x + 12, y, if have >= *n { GOOD } else { TEXT }, None, 1);
                        let bw = 60;
                        fb.rect_fill(r.x + r.w - bw - 6, y + 2, bw, 4, PANEL_LO);
                        fb.rect_fill(r.x + r.w - bw - 6, y + 2, bw * have / (*n).max(1), 4, GOLD);
                        y += 10;
                    }
                }
                None => fb.text("It stands!", x, y, GOOD, None, 1),
            }
        }
        Sel::Source(i) => {
            let s = &g.level().sources[i];
            fb.text(s.good.source_name(), x, y, GOLD, None, 1);
            y += 12;
            let stock = if s.infinite() { "Endless.".to_string() } else { format!("{} of {} on it.", g.city.src_stock[i], s.max) };
            fb.text(&stock, x, y, TEXT, None, 1);
            y += 10;
            let grows: Vec<&str> = (0..4).filter(|&k| s.good.season_growth(k) > 0).map(|k| SEASONS[k]).collect();
            let gtxt = if s.infinite() { format!("{} weigh {}.", crate::game::cap(s.good.words()), s.good.weight().word()) } else if grows.len() == 4 { "Grows all year.".to_string() } else { format!("Grows in {}.", grows.join(", ").to_lowercase()) };
            fb.text(&gtxt, x, y, TEXT_DIM, None, 1);
        }
        Sel::B(id) => {
            let Some(b) = g.city.building(id) else { return };
            fb.text(&g.building_title(b), x, y, GOLD, None, 1);
            if b.kind.slots() > 0 {
                fb.text("hire", r.x + r.w - 92, y, TEXT_DIM, None, 1);
            }
            y += 12;
            if b.kind.is_home() {
                let cap = residents(b.species, b.tier);
                let mut t = format!("{} of {} {}", b.residents, cap, b.species.plural());
                if b.coming > 0 {
                    t += &format!(", {} coming", b.coming);
                }
                fb.text(&t, x, y, TEXT, None, 1);
                for k in 0..TIERS {
                    let px = r.x + r.w - 40 + k as i32 * 9;
                    fb.rect_fill(px, y + 1, 7, 6, if k <= b.tier { GOLD } else { PANEL_LO });
                    fb.rect_outline(px, y + 1, 7, 6, OUTLINE);
                }
                y += 11;
                fb.text("has", x, y, TEXT_DIM, None, 1);
                let mut shown: Vec<Good> = FOODS.to_vec();
                shown.extend([Good::Wicker, Good::Pot, Good::Brick]);
                let present: Vec<Good> = shown.iter().copied().filter(|gd| b.stock[gd.index()] > 0).collect();
                if present.is_empty() {
                    fb.text("nothing", x + 24, y, BAD, None, 1);
                } else {
                    goods_row(fb, g, x + 24, y, &b.stock, &present, false);
                }
                y += 11;
                if b.tier + 1 < TIERS && g.city.tier_possible(b.tier + 1) {
                    let n = &TIER_NEEDS[b.tier + 1];
                    let mut cx = fb_text(fb, &format!("{}:", TIER_NAMES[b.tier + 1]), x, y, TEXT_DIM);
                    cx += 3;
                    fb.text(&format!("{} food{}", n.foods, if n.foods > 1 { "s" } else { "" }), cx, y, TEXT, None, 1);
                    cx += Framebuffer::text_width("2 foods", 1) + 3;
                    for gd in n.goods {
                        let s = g.art.good(*gd);
                        fb.blit(s, cx, y + (8 - s.h) / 2, false);
                        cx += s.w + 3;
                    }
                    if n.pleasant > 0 {
                        let p = g.city.pleasant(b);
                        fb.text(&format!("nice {p}/{}", n.pleasant), cx + 2, y, if p >= n.pleasant { GOOD } else { BAD }, None, 1);
                    }
                } else {
                    fb.text(&format!("Pays {}.{} seeds a day.", b.residents * TAX_TENTHS[b.tier] / 10, b.residents * TAX_TENTHS[b.tier] % 10), x, y, TEXT_DIM, None, 1);
                }
                y += 11;
            } else if b.kind.slots() > 0 {
                let n = g.city.workers_at(id);
                let mut t = format!("{} of {} at work", n, g.city.slots(b));
                if b.kind == Kind::Store {
                    t += &format!(", {} of {STORE_CAP} full", b.total());
                }
                fb.text(&t, x, y, TEXT, None, 1);
                y += 11;
                match b.kind {
                    Kind::Camp => {
                        if let Some(si) = b.source {
                            let s = &g.level().sources[si];
                            let t = if s.infinite() { format!("{}: endless", s.good.source_name()) } else { format!("{}: {} left", s.good.source_name(), g.city.src_stock[si]) };
                            fb.text(&t, x, y, TEXT_DIM, None, 1);
                        }
                    }
                    Kind::Store => {}
                    k => {
                        if let Some(rc) = k.recipe() {
                            let mut list: Vec<Good> = rc.inputs.iter().map(|(gd, _)| *gd).collect();
                            list.push(rc.output);
                            let cx = goods_row(fb, g, x, y, &b.stock, &list, true);
                            fb.rect_fill(cx + 4, y + 2, 40, 4, PANEL_LO);
                            fb.rect_fill(cx + 4, y + 2, 40 * b.progress / rc.ticks, 4, GOLD);
                        }
                    }
                }
                if b.kind != Kind::Store {
                    y += 11;
                }
            }
            for (hr, h) in &hots {
                let hover = hr.contains(mx, my);
                match h {
                    Hot::Species(s) => {
                        ui::button_face(fb, *hr, hover, b.species == *s);
                        let spr = g.art.creature(*s, 0);
                        fb.blit(spr, hr.x + (hr.w - spr.w) / 2, hr.y + hr.h - spr.h - 1, false);
                    }
                    Hot::Order(gd) => {
                        let o = b.orders[gd.index()];
                        let face = ui::button_face(fb, *hr, hover, o == Order::Get);
                        let s = g.art.good(*gd);
                        fb.blit(s, hr.x + 2, hr.y + 2, false);
                        fb.text(&format!("{}", b.stock[gd.index()]), hr.x + 13, hr.y + 2, face, None, 1);
                        let (t, c) = match o {
                            Order::Accept => ("take", TEXT_DIM),
                            Order::Get => ("GET", GOLD),
                            Order::Refuse => ("no", BAD),
                        };
                        fb.text_centered(t, hr.x + hr.w / 2, hr.y + 10, c, None, 1);
                        if o == Order::Refuse {
                            fb.line(hr.x + 2, hr.y + 9, hr.x + 10, hr.y + 2, BAD);
                        }
                    }
                    Hot::Pause => small_button(fb, *hr, if b.paused { "Go" } else { "Stop" }, hover, b.paused),
                    Hot::Demolish => small_button(fb, *hr, "Raze", hover, false),
                }
            }
            let (t, bad) = g.note_text(b);
            let ty = r.y + r.h - 13;
            let col = if bad { BAD } else { GOOD };
            let lines = g.card_note_lines(b);
            if lines.is_empty() {
                fb.text(&t, x, ty, col, None, 1);
            } else {
                // Too long for beside the buttons: wrapped over the full width, just above them.
                let top = ty - 10 * lines.len() as i32 - 2;
                for (i, l) in lines.iter().enumerate() {
                    fb.text(l, x, top + i as i32 * 10, col, None, 1);
                }
            }
        }
    }
}

fn fb_text(fb: &mut Framebuffer, t: &str, x: i32, y: i32, col: u8) -> i32 {
    fb.text(t, x, y, col, None, 1);
    x + Framebuffer::text_width(t, 1)
}

// ----- Site card and results ---------------------------------------------------------------------

fn new_thing(fb: &mut Framebuffer, g: &Game, token: &str, x: i32, y: i32) -> i32 {
    let f = Rect::new(x, y, 20, 20);
    let label: String;
    if let Some(k) = Kind::from_name(token) {
        ui::button_face(fb, f, false, false);
        tool_icon(fb, g, Tool::Build(k), f);
        label = k.title().to_string();
    } else if let Some(i) = Infra::from_name(token) {
        ui::button_face(fb, f, false, false);
        tool_icon(fb, g, Tool::Infra(i), f);
        label = i.title().to_string();
    } else if let Some(s) = Species::from_name(token) {
        let spr = g.art.creature(s, 0);
        fb.blit(spr, x + (20 - spr.w) / 2, y + 18 - spr.h, false);
        label = s.title().to_string();
    } else if let Some(gd) = Good::from_name(token) {
        let s = g.art.good(gd);
        fb.blit(s, x + (20 - s.w) / 2, y + (20 - s.h) / 2, false);
        label = crate::game::cap(gd.word());
    } else {
        let t = match token {
            "WINTER" => {
                fb.rect_fill(x + 4, y + 4, 12, 12, ACCENT_HI);
                fb.rect_outline(x + 4, y + 4, 12, 12, OUTLINE);
                "Winter"
            }
            "TIERS" => {
                for k in 0..4 {
                    fb.rect_fill(x + 1 + k * 5, y + 12 - k * 3, 4, 6 + k * 3, if k < 2 { GOLD } else { PANEL_LO });
                    fb.rect_outline(x + 1 + k * 5, y + 12 - k * 3, 4, 6 + k * 3, OUTLINE);
                }
                "Tiers"
            }
            "PLEASANT" => {
                tool_icon(fb, g, Tool::Build(Kind::Garden), f);
                "Pleasant spots"
            }
            "ORDERS" => {
                small_button(fb, Rect::new(x, y + 4, 20, 12), "GET", false, true);
                "Store orders"
            }
            "VINE" => {
                for k in 0..10 {
                    let dx = if k % 4 < 2 { 0 } else { 1 };
                    fb.rect_fill(x + 9 + dx, y + k * 2, 2, 2, VINE);
                    if k % 3 == 0 {
                        fb.pset(x + 8 + dx * 3, y + k * 2 + 1, LEAF_HI);
                    }
                }
                "Vine"
            }
            "CLOUD" => {
                fb.rect_fill(x + 2, y + 9, 16, 5, ROCK_HI);
                fb.rect_fill(x + 5, y + 6, 8, 3, WHITE);
                fb.rect_fill(x + 2, y + 13, 16, 1, ACCENT);
                "Cloud"
            }
            _ => token,
        };
        label = t.to_string();
    }
    fb.text(&label, x + 24, y + 6, TEXT, None, 1);
    x + 24 + Framebuffer::text_width(&label, 1) + 10
}

fn draw_intro(fb: &mut Framebuffer, g: &Game) {
    let r = R_INTRO;
    ui::panel(fb, r);
    let lv = g.level();
    fb.text(&format!("{} {}", lv.id, lv.name), r.x + 10, r.y + 8, GOLD, Some(OUTLINE), 2);
    let mut y = r.y + 30;
    let n = fb.text_wrapped(&lv.rule, r.x + 10, y, r.w - 20, 10, TEXT, None);
    y += n * 10 + 6;
    if !lv.new.is_empty() {
        let mut x = r.x + 10;
        fb.text("New", x, y + 6, ACCENT_HI, None, 1);
        x += 28;
        for t in &lv.new {
            if x > r.x + r.w - 80 {
                x = r.x + 38;
                y += 22;
            }
            x = new_thing(fb, g, t, x, y);
        }
        y += 26;
    }
    fb.text("Goals", r.x + 10, y, ACCENT_HI, None, 1);
    let mut gx = r.x + 50;
    for gl in &lv.goals {
        let t = match gl {
            Goal::Fortress => format!("raise the fortress ({} stage{})", lv.stages.len(), if lv.stages.len() == 1 { "" } else { "s" }),
            Goal::Pop(n) => format!("{n} residents"),
            Goal::Tier(t, n) => format!("{n} {}s", TIER_NAMES[*t].to_lowercase()),
        };
        if gx + Framebuffer::text_width(&t, 1) > r.x + r.w - 10 {
            gx = r.x + 50;
            y += 10;
        }
        fb.text(&t, gx, y, TEXT, None, 1);
        gx += Framebuffer::text_width(&t, 1) + 12;
    }
    y += 14;
    if lv.medals[0] > 0 {
        fb.text("Medals", r.x + 10, y, ACCENT_HI, None, 1);
        let mut mx = r.x + 56;
        for (k, d) in lv.medals.iter().enumerate() {
            medal_icon(fb, mx + 3, y + 3, k, false);
            fb.text(&format!("day {d}"), mx + 9, y, TEXT, None, 1);
            mx += 62;
        }
        y += 12;
    }
    if lv.challenge != Challenge::None {
        let rec = g.save.rec(&lv.id);
        ui::star(fb, r.x + 10, y, rec.challenge);
        fb.text(&format!("Challenge: {}", lv.challenge.text().to_lowercase()), r.x + 22, y, TEXT, None, 1);
        y += 12;
    }
    let rec = g.save.rec(&lv.id);
    if let Some(d) = rec.best_day {
        medal_icon(fb, r.x + 13, y + 3, lv.medal_for(d), false);
        fb.text(&format!("Your best: day {d}"), r.x + 22, y, GOOD, None, 1);
    }
    let [a, b] = intro_buttons();
    let (mx, my) = g.mouse;
    match g.resume_day {
        Some(d) => {
            ui::button(fb, a, &format!("Go on: day {d}"), a.contains(mx, my), false);
            ui::button(fb, b, "Start over", b.contains(mx, my), false);
        }
        None => ui::button(fb, Rect::new(r.x + r.w / 2 - 48, a.y, 96, 16), "Begin", a.contains(mx, my) || b.contains(mx, my), false),
    }
}

fn draw_results(fb: &mut Framebuffer, g: &Game) {
    let Overlay::Results { day, medal, challenge, best } = g.overlay.clone() else { return };
    let r = R_RESULTS;
    ui::panel(fb, r);
    let lv = g.level();
    let head = if lv.has_fortress() { "The fortress stands!" } else { "Every goal met!" };
    fb.text_centered(head, r.x + r.w / 2, r.y + 10, GOLD, Some(OUTLINE), 1);
    medal_icon(fb, r.x + r.w / 2, r.y + 38, medal, true);
    let mtxt = ["Gold", "Silver", "Bronze", "Finished"][medal];
    fb.text_centered(&format!("{mtxt}: day {day}"), r.x + r.w / 2, r.y + 52, TEXT, None, 1);
    let mut y = r.y + 68;
    let line = |fb: &mut Framebuffer, y: &mut i32, t: &str, c: u8| {
        fb.text_centered(t, r.x + r.w / 2, *y, c, None, 1);
        *y += 11;
    };
    if medal > 0 && medal < 4 && lv.medals[0] > 0 {
        let next = lv.medals[medal.min(3) - 1];
        line(fb, &mut y, &format!("{} needs day {next} or sooner.", ["Gold", "Silver", "Bronze"][medal - 1]), TEXT_DIM);
    }
    if best {
        line(fb, &mut y, "Your best yet.", GOOD);
    }
    if lv.challenge != Challenge::None {
        let t = format!("Challenge ({}): {}", lv.challenge.text().to_lowercase(), if challenge { "done!" } else { "not this time" });
        line(fb, &mut y, &t, if challenge { GOLD } else { TEXT_DIM });
    }
    line(fb, &mut y, &format!("Most residents: {}.  Seeds spent: {}.", g.city.peak_pop, g.city.spent), TEXT);
    if g.city.slipped > 0 {
        line(fb, &mut y, &format!("Homes slipped {} time{}.", g.city.slipped, if g.city.slipped == 1 { "" } else { "s" }), TEXT_DIM);
    }
    let [a, b, c] = result_buttons();
    let (mx, my) = g.mouse;
    ui::button(fb, a, "Next site", a.contains(mx, my), false);
    ui::button(fb, b, "Again", b.contains(mx, my), false);
    ui::button(fb, c, "Menu", c.contains(mx, my), false);
}

// ----- The play screen --------------------------------------------------------------------------

fn draw_play(g: &Game, fb: &mut Framebuffer) {
    g.gallery.backdrop(g.level().chapter).draw(fb, &BACKDROP, VIEW_H);
    fb.set_view(-g.cam.0, -g.cam.1, (0, 0, VIEW_W, VIEW_H));
    let (x0, x1) = (g.cam.0 / TILE, (g.cam.0 + VIEW_W) / TILE + 1);
    let (y0, y1) = (g.cam.1 / TILE, (g.cam.1 + VIEW_H) / TILE + 1);
    draw_terrain(fb, &g.city.world, x0, x1, y0, y1);
    draw_reach(fb, g);
    draw_world_things(fb, g);
    draw_creatures(fb, g);
    if let Some(s) = g.sel {
        if let Some((x, y, w, h)) = g.sel_box(s) {
            fb.corners(x * TILE - 2, y * TILE - 2, w * TILE + 4, h * TILE + 4, 4, WHITE);
        }
    }
    draw_ghost(fb, g);
    draw_floaters(fb, g);
    fb.reset_view();
    draw_header(fb, g);
    draw_card(fb, g);
    draw_panel(fb, g);
    match g.overlay {
        Overlay::Intro => draw_intro(fb, g),
        Overlay::Results { .. } => draw_results(fb, g),
        Overlay::None => {
            if let Some((t, x, y)) = g.coach() {
                let (sx, sy) = g.to_screen(x, y);
                bubble(fb, &t, sx, sy);
            } else if let Some(b) = &g.tip {
                let (sx, sy) = g.to_screen(b.x, b.y);
                bubble(fb, &b.text, sx, sy);
            }
            if g.banner.is_some() {
                draw_banner(fb, g);
            } else if g.paused {
                fb.text_centered("Paused", 200, 16, WHITE, Some(OUTLINE), 1);
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
    fb.text_centered("Grow a colony. Raise the fortress.", 200, 266, TEXT, Some(OUTLINE), 1);
    fb.text_centered("Click to begin", 200, 278, ACCENT_HI, Some(OUTLINE), 1);
    fb.text_centered("F1 rules   F2 scanlines   F3 music   Esc quit", 200, 290, TEXT_DIM, Some(OUTLINE), 1);
}

pub const CHAPTER_NAMES: [&str; 5] = ["Meadow", "Quarry", "Canopy", "Bastion", "Skyreach"];

fn draw_select(g: &Game, fb: &mut Framebuffer) {
    g.gallery.title.draw(fb, &title_slots(), 300);
    ui::dim(fb, Rect::new(0, 0, 400, 300));
    fb.text_centered("Choose a site", 200, 8, GOLD, Some(OUTLINE), 2);
    let (mx, my) = g.mouse;
    let mut medals = [0; 4];
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
                fb.text(&lv.id, r.x + 3, r.y + 3, if rec.done { GOOD } else { col }, None, 1);
                if rec.done {
                    fb.rect_outline(r.x - 1, r.y - 1, r.w + 2, r.h + 2, GOLD);
                    check(fb, r.x + 25, r.y + 3, GOOD);
                }
                let mut name = lv.name.clone();
                name.truncate(12);
                fb.text(&name, r.x + 3, r.y + 15, col, None, 1);
                if !rec.run.is_empty() {
                    fb.text("...", r.x + 30, r.y + 3, ACCENT_HI, None, 1);
                }
            } else {
                ui::button_off(fb, r, "");
                fb.text(&lv.id, r.x + 3, r.y + 3, TEXT_FAINT, None, 1);
                fb.text("locked", r.x + 3, r.y + 15, TEXT_FAINT, None, 1);
            }
            if let Some(d) = rec.best_day {
                let m = lv.medal_for(d);
                medal_icon(fb, r.x + r.w - 15, r.y + 7, m, false);
                medals[m] += 1;
            }
            if lv.challenge != Challenge::None {
                ui::star(fb, r.x + r.w - 9, r.y + 3, rec.challenge);
            }
        }
    }
    let done = g.levels.iter().filter(|l| g.save.rec(&l.id).done).count();
    fb.text_centered(&format!("Gold {}  Silver {}  Bronze {}    Sites done {done} of {}", medals[0], medals[1], medals[2], g.levels.len()), 200, 262, TEXT, Some(OUTLINE), 1);
    ui::button(fb, R_SEL_HELP, "Rules", R_SEL_HELP.contains(mx, my), false);
    ui::button(fb, R_SEL_QUIT, "Quit", R_SEL_QUIT.contains(mx, my), false);
}

fn draw_help(g: &Game, fb: &mut Framebuffer) {
    g.gallery.backdrop(1).draw(fb, &BACKDROP, 300);
    ui::panel(fb, Rect::new(4, 4, 392, 292));
    ui::inset(fb, Rect::new(10, 22, 380, 250));
    let titles = ["Homes", "Work", "Creatures", "Moving", "Goods and tiers", "Seasons and keys"];
    fb.text_centered(&format!("Rules {} of {}: {}", g.help_page + 1, HELP_PAGES, titles[g.help_page]), 200, 9, GOLD, Some(OUTLINE), 1);
    let (x, mut y) = (18, 30);
    let mut para = |fb: &mut Framebuffer, s: &str, col: u8, y: &mut i32| {
        let n = fb.text_wrapped(s, x, *y, 364, 10, col, None);
        *y += n * 10 + 5;
    };
    match g.help_page {
        0 => {
            for (i, s) in SPECIES.iter().enumerate() {
                let hx = x + 40 + i as i32 * 110;
                draw_home(fb, *s, i, hx, y + 2);
                fb.text(s.home().title(), hx + 26, y + 6, GOLD, None, 1);
                fb.text(s.plural(), hx + 26, y + 16, TEXT_DIM, None, 1);
            }
            y += 34;
            para(fb, "Build homes. Settlers walk in from the trail (the signpost), one at a time. A home its species can't walk to from the trail stays empty.", TEXT, &mut y);
            para(fb, "Every resident is a worker. Homes pay seeds in tax every day, and you build with seeds.", TEXT, &mut y);
            para(fb, "The first settlers come anyway. After that a home only fills up while it has food in it. Run out for two days and someone leaves.", TEXT, &mut y);
            para(fb, "Homes move up when they have what the next tier needs: more residents, more tax. They slip back when they run out.", TEXT, &mut y);
            para(fb, "Click a home to see what it has and what it wants next.", TEXT_DIM, &mut y);
        }
        1 => {
            let f = Rect::new(x, y, 20, 20);
            ui::button_face(fb, f, false, false);
            tool_icon(fb, g, Tool::Build(Kind::Camp), f);
            fb.text("Camp: goes on a bush, tree, pile, quarry, bank or pond.", x + 26, y + 6, TEXT, None, 1);
            y += 24;
            let f = Rect::new(x, y, 20, 20);
            ui::button_face(fb, f, false, false);
            tool_icon(fb, g, Tool::Build(Kind::Store), f);
            fb.text("Store: 24 goods, 8 of a kind. Feeds all in reach.", x + 26, y + 6, TEXT, None, 1);
            y += 26;
            para(fb, "Each workplace hires one kind of creature: you choose. A new hire walks over from a home of that kind, so it has to be able to get there.", TEXT, &mut y);
            para(fb, "Workers carry one thing at a time to the nearest place that wants it: homes, workshops and the fortress first, stores last. Stores never feed stores.", TEXT, &mut y);
            para(fb, "Nobody carries a thing further than 100 ticks of walking, by their own way of moving with that load. The dots show where they can go.", TEXT, &mut y);
            para(fb, "A store told to GET a good sends its carriers to fetch it from other stores, and takes nothing else: that is how goods travel far.", ACCENT_HI, &mut y);
        }
        2 => {
            for s in SPECIES {
                let spr = g.art.creature(s, 0);
                fb.blit(spr, x, y + 10 - spr.h, false);
                fb.text(s.title(), x + 14, y + 1, GOLD, None, 1);
                fb.text(s.blurb(), x + 70, y + 1, TEXT_DIM, None, 1);
                let r0 = residents(s, 0);
                let r3 = residents(s, TIERS - 1);
                fb.text(&format!("{r0} to {r3} a home"), x + 280, y + 1, TEXT, None, 1);
                jump_table(fb, g, s, x + 14, y + 12);
                y += 56;
            }
            para(fb, "Green bars: how many tiles up a jump goes, for each load. Pace: ticks per tile. Climbing is twice as slow; falling is quick.", TEXT, &mut y);
        }
        3 => {
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
            para(fb, "Pots are fragile: whoever carries one never falls more than a tile. Clouds hold up light loads only. Nobody swims.", BAD, &mut y);
        }
        4 => {
            let mut col = 0;
            for gd in GOODS {
                let cx = x + col * 186;
                let s = g.art.good(gd);
                fb.blit(s, cx, y + (8 - s.h) / 2, false);
                let fr = if gd.fragile() { ", fragile" } else if gd.food() { ", food" } else { "" };
                fb.text(&format!("{}: {}{fr}", crate::game::cap(gd.word()), gd.weight().word()), cx + 12, y, TEXT, None, 1);
                col += 1;
                if col == 2 {
                    col = 0;
                    y += 11;
                }
            }
            y += 6;
            for k in [Kind::Weaver, Kind::Kiln, Kind::Mason] {
                let f = Rect::new(x, y, 20, 20);
                ui::button_face(fb, f, false, false);
                tool_icon(fb, g, Tool::Build(k), f);
                fb.text(k.title(), x + 26, y + 1, GOLD, None, 1);
                recipe(fb, g, k, x + 26, y + 11);
                y += 23;
            }
            y += 2;
            for t in 0..TIERS {
                let n = &TIER_NEEDS[t];
                fb.text(TIER_NAMES[t], x, y, GOLD, None, 1);
                let mut cx = x + 60;
                fb.text(&format!("{} food{}", n.foods, if n.foods > 1 { "s" } else { "" }), cx, y, TEXT, None, 1);
                cx += 50;
                for gd in n.goods {
                    let s = g.art.good(*gd);
                    fb.blit(s, cx, y + (8 - s.h) / 2, false);
                    cx += s.w + 4;
                }
                if n.pleasant > 0 {
                    fb.text("+ a pleasant spot", cx + 4, y, TEXT, None, 1);
                }
                y += 11;
            }
            para(fb, "Kilns, masons, quarries and clay pits are noisy for 3 tiles around. Gardens are pleasant.", TEXT_DIM, &mut y);
        }
        _ => {
            for (k, s) in SEASONS.iter().enumerate() {
                let cx = x + k as i32 * 92;
                fb.text(s, cx, y, [GRASS_HI, GOLD, BEAK, ACCENT_HI][k], None, 1);
                let mut gx = cx;
                for gd in FOODS {
                    let s2 = g.art.good(gd);
                    fb.blit(s2, gx, y + 11, false);
                    if gd.season_growth(k) == 0 {
                        fb.line(gx, y + 19, gx + s2.w, y + 11, BAD);
                    }
                    gx += 14;
                }
            }
            y += 26;
            para(fb, "A year is four seasons of six days. Berries and fish stop in winter. Nut trees fruit only in autumn. Stock up.", TEXT, &mut y);
            let rows = [
                ("Left click", "build, or look at something"),
                ("Right click", "put the tool away, or close the card"),
                ("1 to 9", "pick a tool    X: demolish"),
                ("T D A", "hire tribbles, dodos or apes"),
                ("Space / Tab", "pause / speed"),
                ("Arrows, wheel", "scroll (or touch the screen's edge)"),
                ("H", "back to the trail"),
                ("Esc", "close, then back to the map"),
                ("F1 F2 F3", "rules, scanlines, music"),
            ];
            for (k, v) in rows {
                fb.text(k, x, y, GOLD, None, 1);
                fb.text(v, x + 96, y, TEXT, None, 1);
                y += 11;
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
