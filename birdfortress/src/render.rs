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
            if n % every == 0 {
                fb.pset(x0 + (x1 - x0) * k / steps, y0 + (y1 - y0) * k / steps, col);
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
            let a = &g.art.pointer;
            // A small arrow: the pointer's top 9 rows.
            for y in 0..10 {
                for x in 0..a.w {
                    let c = a.px[(y * a.w + x) as usize];
                    if c != 0 {
                        fb.pset(cx - 3 + x, cy - 5 + y, c);
                    }
                }
            }
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
        Tool::Select => "SELECT: CLICK SOMETHING TO SEE ITS ROUTES. RIGHT-CLICK ERASES.".into(),
        Tool::Species(s) => format!("{} ({} SEEDS): {}", s.name(), s.cost(), s.blurb()),
        Tool::Build(k) => format!("{} ({} SEEDS): {}", k.name(), k.cost(), k.blurb()),
        Tool::Infra(i) => format!("{}: {}", i.name(), i.blurb()),
        Tool::Erase => "ERASE (X): CLICK SOMETHING YOU BUILT TO REMOVE IT.".into(),
    }
}

fn draw_panel(fb: &mut Framebuffer, g: &Game) {
    ui::panel(fb, Rect::new(0, VIEW_H, 400, 300 - VIEW_H));
    let (mx, my) = g.mouse;
    let tools = g.tools();
    let mut hover_label: Option<String> = None;
    for (i, &t) in tools.iter().enumerate() {
        let r = tool_rect(i);
        let hover = r.contains(mx, my);
        ui::button_face(fb, r, hover, g.tool == t);
        tool_icon(fb, g, t, r);
        if hover {
            hover_label = Some(format!("[{}] {}", if t == Tool::Erase { "X".to_string() } else { (i + 1).to_string() }, tool_label(t)));
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
    ui::button(fb, R_SPEED, &format!("X{}", SPEEDS[g.speed]), hov(R_SPEED), false);
    ui::button_face(fb, R_RESET, hov(R_RESET), false);
    fb.rect_fill(R_RESET.x + 6, R_RESET.y + 6, 8, 8, if g.sim.is_some() { BAD } else { TEXT_FAINT });
    if hov(R_RUN) {
        hover_label = Some("RUN / PAUSE [SPACE]".into());
    } else if hov(R_STEP) {
        hover_label = Some("ADVANCE ONE TICK [S]".into());
    } else if hov(R_SPEED) {
        hover_label = Some("SPEED: TICKS PER FRAME [TAB]".into());
    } else if hov(R_RESET) {
        hover_label = Some("RESET THE RUN TO TICK 0 [R]".into());
    }
    // Status.
    ui::inset(fb, R_STATUS);
    let (text, bad) = match hover_label {
        Some(t) => (t, false),
        None => g.status_line(),
    };
    fb.text_wrapped(&text, R_STATUS.x + 4, R_STATUS.y + 3, R_STATUS.w - 8, 10, if bad { BAD } else { TEXT }, None);
    // Metrics.
    ui::inset(fb, R_METRICS);
    let rec = g.save.rec(&g.level().id);
    let ticks = g.sim.as_ref().map(|s| s.tick).unwrap_or(0);
    let cost = crate::world::layout_cost(g.level(), &g.layout);
    let crew = g.layout.crew();
    let tg = g.level().targets;
    let rows = [("TICKS", ticks, rec.ticks, tg.ticks), ("COST", cost, rec.cost, tg.cost), ("CREW", crew, rec.crew, tg.crew)];
    for (k, (name, v, best, _goal)) in rows.iter().enumerate() {
        let y = R_METRICS.y + 3 + k as i32 * 10;
        fb.text(name, R_METRICS.x + 4, y, TEXT_DIM, None, 1);
        fb.text(&format!("{v:>5}"), R_METRICS.x + 36, y, TEXT, None, 1);
        if let Some(b) = best {
            fb.text(&format!("BEST{b:>5}"), R_METRICS.x + 70, y, GOLD, None, 1);
        }
    }
}

fn draw_header(fb: &mut Framebuffer, g: &Game) {
    let lv = g.level();
    fb.text(&format!("{} {}", lv.id, lv.name), 3, 3, TEXT, Some(OUTLINE), 1);
    let (mx, my) = g.mouse;
    ui::button(fb, R_HELP, "?", R_HELP.contains(mx, my), false);
    ui::button(fb, R_ROUTES, "PATHS", R_ROUTES.contains(mx, my), g.show_routes);
    ui::button(fb, R_MENU, "MENU", R_MENU.contains(mx, my), false);
    if let Some(sim) = &g.sim {
        if g.running {
            fb.text(&format!("TICK {}", sim.tick), 230, 3, TEXT, Some(OUTLINE), 1);
        } else if sim.outcome == Outcome::Running {
            fb.text(&format!("PAUSED AT {}", sim.tick), 230, 3, GOLD, Some(OUTLINE), 1);
        }
    }
}

fn draw_card(fb: &mut Framebuffer, g: &Game) {
    let Some(card) = g.card_rect() else { return };
    let Some(n) = g.selected else { return };
    ui::panel(fb, card);
    fb.text(&g.node_name(n), card.x + 4, card.y + 4, GOLD, None, 1);
    // What's inside right now.
    let mut x = card.x + 4;
    let y = card.y + 14;
    let lv = g.level();
    let stock_line: Vec<(Good, String)> = match (&g.sim, n) {
        (Some(sim), _) => {
            let st = sim.node(n);
            GOODS
                .iter()
                .filter_map(|&gd| {
                    let s = st?;
                    if let NodeRef::Source(_) = n {
                        if s.max <= 0 {
                            return (s.good == Some(gd)).then(|| (gd, "ENDLESS".to_string()));
                        }
                    }
                    if n == NodeRef::Fortress {
                        let need = lv.need_of(gd);
                        return (need > 0).then(|| (gd, format!("{}/{need}", s.stock[gd.index()])));
                    }
                    (s.stock[gd.index()] > 0).then(|| (gd, format!("{}", s.stock[gd.index()])))
                })
                .collect()
        }
        (None, NodeRef::Source(i)) => {
            let s = &lv.sources[i];
            vec![(s.good, if s.infinite() { "ENDLESS".into() } else if s.regrow > 0 { format!("{} +1/{}T", s.max, s.regrow) } else { format!("{} ONLY", s.max) })]
        }
        (None, NodeRef::Fortress) => lv.need.iter().map(|(gd, c)| (*gd, format!("0/{c}"))).collect(),
        (None, NodeRef::Building(id)) => match g.layout.building(id).and_then(|b| b.kind.recipe()) {
            Some(r) => {
                let mut v: Vec<(Good, String)> = r.inputs.iter().map(|(gd, c)| (*gd, format!("{c}"))).collect();
                v.push((r.output, format!("> 1 IN {}T", r.ticks)));
                v
            }
            None => vec![],
        },
    };
    if stock_line.is_empty() {
        fb.text(if g.sim.is_some() { "EMPTY" } else { "HOLDS 6 OF ANYTHING" }, x, y, TEXT_FAINT, None, 1);
    }
    for (gd, t) in stock_line {
        let s = g.art.good(gd);
        if x + s.w + 4 + Framebuffer::text_width(&t, 1) > card.x + card.w - 2 {
            break;
        }
        fb.blit(s, x, y + (8 - s.h) / 2, false);
        fb.text(&t, x + s.w + 1, y, TEXT, None, 1);
        x += s.w + 6 + Framebuffer::text_width(&t, 1);
    }
    let routes = g.card_routes();
    if routes.is_empty() {
        fb.text("NO ROUTES. PICK A CREATURE.", card.x + 4, card.y + 27, TEXT_FAINT, None, 1);
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
        let (arrow, other) = if r.from == n { (">", r.to) } else { ("<", r.from) };
        let mut name = g.node_name(other);
        name.truncate(8);
        fb.text(&format!("{arrow}{name}"), row.x + 36, row.y + 3, TEXT_DIM, None, 1);
        match g.plans.get(ri).and_then(|p| if p.problem.is_none() { Some(p.cycle()) } else { None }) {
            Some(c) => fb.text(&format!("{c:>3}"), row.x + 92, row.y + 3, TEXT_FAINT, None, 1),
            None => fb.text(" !!", row.x + 92, row.y + 3, BAD, None, 1),
        }
        for (b, lab) in [(minus, "-"), (plus, "+"), (del, "X")] {
            let h = b.contains(mx, my);
            ui::button_face(fb, b, h, false);
            fb.text(lab, b.x + 2, b.y + 2, if lab == "X" { BAD } else if h { PANEL_DEEP } else { TEXT }, None, 1);
        }
    }
    // Tooltip line for the hovered row.
    for (i, ri) in g.card_routes().into_iter().enumerate() {
        let (row, cargo, ..) = g.card_row(card, i);
        if row.contains(mx, my) {
            let r = g.layout.routes[ri];
            let tip = match g.plans.get(ri).and_then(|p| p.problem.clone()) {
                Some(p) => p.text().to_string(),
                None if cargo.contains(mx, my) => "CLICK TO CHANGE WHAT IT CARRIES".to_string(),
                None => format!("{} X{} CARRY {}: {} TICKS A TRIP", r.species.name(), r.count, r.cargo.name(), g.plans[ri].cycle()),
            };
            let w = Framebuffer::text_width(&tip, 1) + 6;
            let tx = (card.x + card.w / 2 - w / 2).clamp(1, 399 - w);
            let ty = card.y + card.h + 1;
            fb.rect_bevel(tx, ty, w, 11, PANEL_DEEP, PANEL_LO, PANEL_LO);
            fb.text(&tip, tx + 3, ty + 2, TEXT, None, 1);
        }
    }
}

fn draw_rule(fb: &mut Framebuffer, g: &Game) {
    let lv = g.level();
    ui::dim(fb, Rect::new(0, 0, 400, VIEW_H));
    let lines = crate::framebuffer::wrap(&lv.rule, 46);
    let h = 70 + lines.len() as i32 * 10;
    let r = Rect::new(52, (VIEW_H - h) / 2, 296, h);
    ui::panel(fb, r);
    fb.text(&format!("{}  {}", lv.id, lv.name), r.x + 8, r.y + 7, GOLD, None, 1);
    for (i, l) in lines.iter().enumerate() {
        fb.text(l, r.x + 8, r.y + 22 + i as i32 * 10, TEXT, None, 1);
    }
    let y = r.y + 26 + lines.len() as i32 * 10;
    let need: Vec<String> = lv.need.iter().map(|(gd, c)| format!("{c} {}", if *c == 1 { gd.name() } else { gd.plural() })).collect();
    fb.text(&format!("BUILD THE FORTRESS: {}", need.join(", ")), r.x + 8, y, ACCENT, None, 1);
    let rec = g.save.rec(&lv.id);
    let tg = lv.targets;
    let star = |v: Option<i32>, t: i32| v.is_some_and(|v| v <= t);
    let mut sx = r.x + 8;
    for (name, v, t) in [("TICKS", rec.ticks, tg.ticks), ("COST", rec.cost, tg.cost), ("CREW", rec.crew, tg.crew)] {
        ui::star(fb, sx, y + 12, star(v, t));
        let s = format!("{name} <={t}");
        fb.text(&s, sx + 9, y + 12, TEXT_DIM, None, 1);
        sx += 18 + Framebuffer::text_width(&s, 1);
    }
    fb.text_centered("CLICK OR PRESS SPACE", r.x + r.w / 2, r.y + r.h - 12, TEXT_FAINT, None, 1);
}

fn draw_results(fb: &mut Framebuffer, g: &Game) {
    let r = R_RESULTS;
    let (mx, my) = g.mouse;
    let b = result_buttons();
    match &g.overlay {
        Overlay::Results { ticks, cost, crew, best } => {
            ui::dim(fb, Rect::new(0, 0, 400, VIEW_H));
            ui::panel(fb, r);
            fb.text_centered("THE FORTRESS STANDS!", r.x + r.w / 2, r.y + 8, GOLD, Some(OUTLINE), 1);
            let lv = g.level();
            let rec = g.save.rec(&lv.id);
            let tg = lv.targets;
            fb.text("SCORE", r.x + 64, r.y + 26, TEXT_DIM, None, 1);
            fb.text("BEST", r.x + 116, r.y + 26, TEXT_DIM, None, 1);
            fb.text("GOAL", r.x + 164, r.y + 26, TEXT_DIM, None, 1);
            let rows = [("TICKS", *ticks, rec.ticks, tg.ticks, best[0]), ("COST", *cost, rec.cost, tg.cost, best[1]), ("CREW", *crew, rec.crew, tg.crew, best[2])];
            for (k, (name, v, b2, goal, nb)) in rows.iter().enumerate() {
                let y = r.y + 40 + k as i32 * 16;
                fb.text(name, r.x + 14, y, TEXT, None, 1);
                fb.text(&format!("{v:>5}"), r.x + 58, y, if *nb { GOOD } else { TEXT }, None, 1);
                fb.text(&format!("{:>5}", b2.unwrap_or(*v)), r.x + 110, y, GOLD, None, 1);
                fb.text(&format!("<={goal}"), r.x + 160, y, TEXT_DIM, None, 1);
                ui::star(fb, r.x + 204, y, b2.unwrap_or(*v) <= *goal);
                if *nb {
                    fb.text("NEW!", r.x + 216, y, GOOD, None, 1);
                }
            }
            fb.text_centered("EACH SCORE IS KEPT ON ITS OWN. TRY ANOTHER SHAPE.", r.x + r.w / 2, r.y + 94, TEXT_FAINT, None, 1);
            let last = g.cur + 1 >= g.levels.len();
            for (k, lab) in [(0, if last { "CLOSE" } else { "NEXT SITE" }), (1, "KEEP TUNING"), (2, "MENU")] {
                ui::button(fb, b[k], lab, b[k].contains(mx, my), false);
            }
        }
        Overlay::Failed(msg) => {
            ui::panel(fb, Rect::new(r.x, r.y + 40, r.w, 80));
            fb.text_centered("THE COLONY GROUND TO A HALT", r.x + r.w / 2, r.y + 48, BAD, Some(OUTLINE), 1);
            fb.text_wrapped(msg, r.x + 10, r.y + 62, r.w - 20, 10, TEXT, None);
            for (k, lab) in [(0, "RESET"), (1, "LOOK"), (2, "MENU")] {
                ui::button(fb, b[k], lab, b[k].contains(mx, my), false);
            }
        }
        _ => {}
    }
}

fn draw_play(g: &Game, fb: &mut Framebuffer) {
    draw_sky(fb, VIEW_H);
    draw_terrain(fb, &g.world);
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
        Overlay::Rule => draw_rule(fb, g),
        Overlay::Results { .. } | Overlay::Failed(_) => draw_results(fb, g),
        Overlay::None => {}
    }
}

// ----- Menus -------------------------------------------------------------------------------------

fn draw_title(g: &Game, fb: &mut Framebuffer) {
    draw_sky(fb, 300);
    // Ground strip with a little colony at work.
    fb.rect_fill(0, 262, 400, 38, SOIL);
    fb.rect_fill(0, 262, 400, 3, GRASS);
    fb.rect_fill(0, 262, 400, 1, GRASS_HI);
    for k in 0..40 {
        let h = hash(k, 7);
        fb.pset(k * 10 + (h % 9) as i32, 270 + ((h >> 5) % 25) as i32, SOIL_DK);
    }
    // Fortress.
    for y in 0..36 {
        for x in 0..40 {
            let mortar = y % 3 == 2 || (x + if (y / 3) % 2 == 0 { 0 } else { 3 }) % 6 == 0;
            fb.pset(300 + x, 226 + y, if mortar { ROCK_DK } else { ROCK });
        }
    }
    for x in (0..40).step_by(6) {
        fb.rect_fill(300 + x, 222, 3, 4, ROCK);
    }
    fb.rect_fill(320, 204, 1, 18, WOOD_DK);
    fb.rect_fill(321, 204, 9, 5, BANNER);
    fb.rect_fill(321, 204, 9, 1, GOLD);
    // Walkers shuttling.
    let f = g.frame as i32;
    for (k, s) in SPECIES.iter().enumerate() {
        let period = 220 + k as i32 * 40;
        let t = (f * (2 + k as i32) / 2 + k as i32 * 70) % (period * 2);
        let (x, flip, carry) = if t < period { (40 + t, false, true) } else { (40 + period * 2 - t, true, false) };
        let cargo = if carry { Some([Good::Berry, Good::Twig, Good::Stone][k]) } else { None };
        blit_creature(fb, g, *s, ((f / 3) % 2) as usize, x, 252, flip, cargo);
    }
    fb.text_centered("BIRD FORTRESS", 202, 52, OUTLINE, None, 4);
    fb.text_centered("BIRD FORTRESS", 200, 50, GOLD, None, 4);
    fb.text_centered("A LOGISTICS PUZZLE FOR TRIBBLES, DODOS AND APES", 200, 92, TEXT, Some(OUTLINE), 1);
    let lines = [
        "EVERY CREATURE SEES A DIFFERENT MAP,",
        "AND IT CHANGES WITH WHAT IT CARRIES.",
        "",
        "LAY OUT DEPOTS, WORKSHOPS AND SPRINGBOARDS,",
        "SEND YOUR CREWS ON THEIR ROUTES, AND WATCH.",
        "THEN DO IT FASTER, CHEAPER, OR WITH FEWER PAWS.",
    ];
    for (i, l) in lines.iter().enumerate() {
        fb.text_centered(l, 200, 114 + i as i32 * 10, TEXT_DIM, Some(OUTLINE), 1);
    }
    fb.text_centered("CLICK OR PRESS ENTER", 200, 186, ACCENT_HI, Some(OUTLINE), 1);
    fb.text_centered("F1 RULES   F2 SCANLINES   F3 MUSIC   ESC QUIT", 200, 200, TEXT_FAINT, Some(OUTLINE), 1);
}

fn draw_select(g: &Game, fb: &mut Framebuffer) {
    draw_sky(fb, 300);
    ui::dim(fb, Rect::new(0, 0, 400, 300));
    fb.text_centered("CHOOSE A BUILDING SITE", 200, 6, GOLD, Some(OUTLINE), 2);
    let names = ["I. MEADOW", "II. QUARRY", "III. CANOPY", "IV. BASTION"];
    let (mx, my) = g.mouse;
    let mut stars = 0;
    let mut total = 0;
    for (c, list) in g.chapters().into_iter().enumerate() {
        let x = 6 + c as i32 * 98;
        fb.text_centered(names.get(c).copied().unwrap_or("?"), x + 47, 32, TEXT, Some(OUTLINE), 1);
        for (i, li) in list.into_iter().enumerate() {
            let r = select_rect(c, i);
            let lv = &g.levels[li];
            let rec = g.save.rec(&lv.id);
            let open = g.unlocked(li);
            if open {
                let col = ui::button_face(fb, r, r.contains(mx, my), false);
                fb.text(&lv.id, r.x + 4, r.y + 3, if rec.solved { GOOD } else { col }, None, 1);
                let mut name = lv.name.clone();
                name.truncate(15);
                fb.text(&name, r.x + 4, r.y + 15, col, None, 1);
            } else {
                ui::button_off(fb, r, "");
                fb.text(&lv.id, r.x + 4, r.y + 3, TEXT_FAINT, None, 1);
                fb.text("LOCKED", r.x + 4, r.y + 15, TEXT_FAINT, None, 1);
            }
            let tg = lv.targets;
            for (k, (v, t)) in [(rec.ticks, tg.ticks), (rec.cost, tg.cost), (rec.crew, tg.crew)].iter().enumerate() {
                let got = v.is_some_and(|v| v <= *t);
                if got {
                    stars += 1;
                }
                total += 1;
                ui::star(fb, r.x + 62 + k as i32 * 9, r.y + 3, got);
            }
        }
    }
    fb.text_centered(&format!("STARS {stars}/{total}   SOLVED {}/{}", g.levels.iter().filter(|l| g.save.rec(&l.id).solved).count(), g.levels.len()), 200, 262, TEXT_DIM, Some(OUTLINE), 1);
    ui::button(fb, R_SEL_HELP, "RULES", R_SEL_HELP.contains(mx, my), false);
    ui::button(fb, R_SEL_QUIT, "QUIT", R_SEL_QUIT.contains(mx, my), false);
}

fn draw_help(g: &Game, fb: &mut Framebuffer) {
    fb.clear(PANEL_DEEP);
    ui::panel(fb, Rect::new(0, 0, 400, 300));
    ui::inset(fb, Rect::new(6, 22, 388, 250));
    let titles = ["HOW IT WORKS", "THE CREATURES", "MOVING AROUND", "GOODS AND WORKSHOPS", "CONTROLS"];
    fb.text_centered(&format!("RULES {}/{}: {}", g.help_page + 1, HELP_PAGES, titles[g.help_page]), 200, 7, GOLD, Some(OUTLINE), 1);
    let (x, mut y) = (14, 30);
    let mut para = |fb: &mut Framebuffer, s: &str, col: u8, y: &mut i32| {
        let n = fb.text_wrapped(s, x, *y, 372, 10, col, None);
        *y += n * 10 + 4;
    };
    match g.help_page {
        0 => {
            para(fb, "EACH SITE ASKS FOR GOODS AT ITS FORTRESS. YOU NEVER CARRY ANYTHING YOURSELF: YOU HIRE CREWS.", TEXT, &mut y);
            para(fb, "A ROUTE IS A CREATURE SHUTTLING BETWEEN TWO PLACES: IT PICKS UP ONE GOOD, CARRIES IT TO THE DROP-OFF, HANDS IT OVER AND WALKS BACK EMPTY. IT ALWAYS TAKES THE FASTEST WAY IT CAN MANAGE, AND THE WAY BACK MAY BE A DIFFERENT ONE.", TEXT, &mut y);
            para(fb, "TO MAKE A ROUTE: PICK A CREATURE BELOW, CLICK WHERE IT PICKS UP (A SOURCE, A DEPOT OR A WORKSHOP), THEN WHERE IT DROPS OFF. CLICK A PLACE TO SEE ITS ROUTES, ADD OR REMOVE CREATURES, OR CHANGE THE CARGO.", TEXT, &mut y);
            para(fb, "EVERY PLACE HAS ONE DOCK (THE FORTRESS HAS TWO). TAKING OR HANDING OVER A GOOD USES IT FOR 2 TICKS, SO CROWDS QUEUE. A CREATURE ONLY PICKS UP WHEN ITS DROP-OFF HAS ROOM.", TEXT, &mut y);
            para(fb, "WHEN THE FORTRESS HAS EVERYTHING, YOU ARE SCORED THREE WAYS: TICKS TAKEN, SEEDS SPENT (CREATURES AND BUILDINGS) AND CREW SIZE. EACH BEST IS KEPT ON ITS OWN, AND EACH SITE HAS A GOAL FOR EACH.", ACCENT, &mut y);
        }
        1 => {
            fb.text("LOAD:", x, y, TEXT_DIM, None, 1);
            for (k, w) in WEIGHTS.iter().enumerate() {
                fb.text(w.name(), x + 96 + k as i32 * 66, y, TEXT_DIM, None, 1);
            }
            y += 14;
            for s in SPECIES {
                let spr = g.art.creature(s, 0);
                fb.blit(spr, x, y + 10 - spr.h, false);
                fb.text(&format!("{} {}S", s.name(), s.cost()), x + 12, y + 2, TEXT, None, 1);
                for (k, w) in WEIGHTS.iter().enumerate() {
                    let j = s.jump(*w);
                    let t = format!("J{} P{}", j, s.pace(*w));
                    fb.text(&t, x + 96 + k as i32 * 66, y + 2, if j == 0 { BAD } else { TEXT }, None, 1);
                }
                y += 16;
                fb.text_wrapped(s.blurb(), x + 12, y, 360, 10, TEXT_DIM, None);
                y += 22;
            }
            para(fb, "J = HOW MANY TILES UP A JUMP RISES. P = TICKS PER TILE WALKED. CLIMBING TAKES TWICE AS LONG. FALLING TAKES 1 TICK A TILE FOR EVERYONE.", ACCENT, &mut y);
            para(fb, "APES CLIMB VINES AND LADDERS CARRYING ANYTHING. TRIBBLES AND DODOS USE LADDERS ONLY WHEN EMPTY OR CARRYING SOMETHING LIGHT, AND NEVER VINES.", TEXT, &mut y);
        }
        2 => {
            para(fb, "WALK: ONE TILE SIDEWAYS ALONG THE GROUND. WALK OFF AN EDGE AND YOU FALL STRAIGHT DOWN, AS FAR AS IT GOES.", TEXT, &mut y);
            para(fb, "JUMP: RISE UP TO J TILES, MOVE UP TO J+1 TILES SIDEWAYS AT THE TOP, THEN FALL. A JUMP STOPS AS SOON AS IT IS ABOVE SOMETHING TO STAND ON, AND NOTHING SOLID MAY BE IN THE WAY.", TEXT, &mut y);
            // A little diagram: a jump-2 arc onto a ledge.
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
            fb.text("RISE 2", dx - 40, dy + 16, TEXT_DIM, None, 1);
            fb.text("ACROSS", dx + 72, dy + 6, TEXT_DIM, None, 1);
            y += 46;
            para(fb, "SPRINGBOARD: A JUMP FROM ONE RISES 2 HIGHER, EVEN FOR A TRIBBLE UNDER A STONE.", TEXT, &mut y);
            para(fb, "BRANCHES, LEDGES AND PLANKS ARE ONE-WAY: JUMP UP THROUGH THEM FROM BELOW, LAND ON THEM FROM ABOVE. WATER CAN'T BE ENTERED.", TEXT, &mut y);
            para(fb, "FRAGILE GOODS (EGGS, POTS) BREAK IF THEIR CARRIER FALLS MORE THAN 1 TILE, SO CREATURES CARRYING THEM WON'T.", BAD, &mut y);
        }
        3 => {
            for gd in GOODS {
                let s = g.art.good(gd);
                fb.blit(s, x, y + (8 - s.h) / 2, false);
                let fr = if gd.fragile() { ", FRAGILE" } else { "" };
                fb.text(&format!("{}: {}{}", gd.name(), gd.weight().name(), fr), x + 12, y, TEXT, None, 1);
                y += 11;
            }
            y += 4;
            for k in KINDS {
                fb.text(&format!("{} ({}S)", k.name(), k.cost()), x, y, GOLD, None, 1);
                fb.text_wrapped(k.blurb(), x + 90, y, 280, 10, TEXT, None);
                y += 22;
            }
            para(fb, "A WORKSHOP HOLDS UP TO 4 OF EACH INPUT AND 3 OF ITS PRODUCT. IT STARTS A NEW BATCH BY ITSELF WHENEVER IT HAS THE INPUTS.", TEXT_DIM, &mut y);
            para(fb, "SOURCES: SOME ARE ENDLESS, SOME HOLD A FEW AND REGROW, SOME RUN OUT. HOVER ONE TO SEE.", TEXT_DIM, &mut y);
        }
        _ => {
            let rows = [
                ("LEFT CLICK", "USE THE TOOL / SELECT"),
                ("RIGHT CLICK", "CANCEL, OR ERASE WHAT'S UNDER IT"),
                ("1-9", "PICK A TOOL FROM THE BAR"),
                ("X", "ERASE TOOL"),
                ("SPACE", "RUN / PAUSE"),
                ("S", "ONE TICK AT A TIME"),
                ("TAB / F", "SPEED: 1, 2, 4, 8 OR 32 TICKS A FRAME"),
                ("R", "RESET THE RUN"),
                ("Z / BACKSPACE", "UNDO"),
                ("V", "SHOW EVERY ROUTE'S PATH OR ONLY THE SELECTED"),
                ("I", "THE SITE'S NOTE AGAIN"),
                ("ESC", "CANCEL, DESELECT, THEN BACK TO THE MAP"),
                ("F1", "THESE RULES"),
                ("F2", "SCANLINES ON/OFF"),
                ("F3", "MUSIC LEVEL"),
            ];
            for (k, v) in rows {
                fb.text(k, x, y, GOLD, None, 1);
                fb.text(v, x + 96, y, TEXT, None, 1);
                y += 11;
            }
            y += 6;
            para(fb, "ROUTE PATHS: BRIGHT DOTS ARE THE LOADED TRIP, DARK DOTS THE WAY BACK. A RED X MEANS A ROUTE THAT CAN'T RUN.", TEXT_DIM, &mut y);
        }
    }
    let (mx, my) = g.mouse;
    if g.help_page > 0 {
        ui::button(fb, R_HELP_PREV, "< PREV", R_HELP_PREV.contains(mx, my), false);
    }
    if g.help_page + 1 < HELP_PAGES {
        ui::button(fb, R_HELP_NEXT, "NEXT >", R_HELP_NEXT.contains(mx, my), false);
    }
    ui::button(fb, R_HELP_CLOSE, "CLOSE", R_HELP_CLOSE.contains(mx, my), false);
}

#[allow(dead_code)]
pub fn sprite_center(s: &Sprite) -> (i32, i32) {
    (s.w / 2, s.h / 2)
}
