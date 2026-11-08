//! Drawing GULP.

use super::art::{self, *};
use super::gen::{AREAS, LESSONS, PONDS};
use super::rules::*;
use super::*;
use crate::framebuffer::{wrap, Framebuffer, Sprite};
use crate::palette::*;
use crate::ui::{self, Rect};

fn tile_xy(p: Pos) -> (i32, i32) {
    (BX + p.0 * TILE, BY + p.1 * TILE)
}

fn centre(p: Pos) -> (i32, i32) {
    (BX + p.0 * TILE + TILE / 2, BY + p.1 * TILE + TILE / 2)
}

fn lerp(a: (i32, i32), b: (i32, i32), k: u32, n: u32) -> (i32, i32) {
    let (k, n) = (k.min(n) as i32, n.max(1) as i32);
    (a.0 + (b.0 - a.0) * k / n, a.1 + (b.1 - a.1) * k / n)
}

fn hash(x: i32, y: i32, salt: u32) -> u32 {
    let mut h = (x as u32).wrapping_mul(0x9E37_79B1) ^ (y as u32).wrapping_mul(0x85EB_CA77) ^ salt.wrapping_mul(0xC2B2_AE3D);
    h ^= h >> 15;
    h = h.wrapping_mul(0x2C1B_3C6D);
    h ^ (h >> 12)
}

pub fn draw(g: &Game, fb: &mut Framebuffer) {
    fb.clear(BG);
    match &g.phase {
        Phase::Bank { offers } => draw_bank(g, fb, offers),
        _ => {
            draw_pond(g, fb);
            if let Phase::Over { won } = g.phase {
                draw_over(g, fb, won);
            }
        }
    }
    if g.show_help {
        draw_help(fb);
    }
}

fn draw_tile(fb: &mut Framebuffer, p: &Pond, at: Pos, frame: u32, salt: u32) {
    let (tx, ty) = tile_xy(at);
    let (x, y) = at;
    let h = hash(x, y, salt);
    match p.tile(at) {
        Tile::Bank | Tile::Thorns => {
            fb.rect_fill(tx, ty, TILE, TILE, if (x + y) % 2 == 0 { BANK_L } else { BANK_D });
            for k in 0..3 {
                if (h >> (20 + k)) & 1 == 1 {
                    let gx = tx + 3 + ((h >> (k * 6)) % 20) as i32;
                    let gy = ty + 4 + ((h >> (k * 6 + 3)) % 18) as i32;
                    fb.pset(gx, gy, GRASS);
                    fb.pset(gx + 1, gy - 1, GRASS_HI);
                    fb.pset(gx + 2, gy, GRASS);
                }
            }
            if p.tile(at) == Tile::Thorns {
                for k in 0..5 {
                    let bx = tx + 3 + k * 5;
                    let by = ty + 8 + ((k * 7) % 11);
                    fb.line(bx, by + 6, bx + 2, by, THORN + 1);
                    fb.line(bx + 2, by, bx + 4, by + 6, THORN);
                    fb.pset(bx + 2, by - 1, THORN + 1);
                }
            }
        }
        Tile::Water | Tile::Lily => {
            fb.rect_fill(tx, ty, TILE, TILE, WATER + 1);
            // Slow glints drifting across the water (ambience).
            let phase = ((frame / 8) as i32 + x * 3 + y * 5) % 9;
            for k in 0..2 {
                let gx = tx + 4 + ((phase + k * 4) * 5) % (TILE - 8);
                let gy = ty + 6 + k * 11;
                fb.rect_fill(gx, gy, 4, 1, WATER + 2);
            }
            fb.rect_fill(tx, ty + TILE - 2, TILE, 2, WATER);
            if p.tile(at) == Tile::Lily {
                fb.circle(tx + TILE / 2, ty + TILE / 2, 10, OUTLINE);
                for r in 0..10 {
                    fb.circle(tx + TILE / 2, ty + TILE / 2, r, if r > 7 { LILY } else { LILY + 1 });
                }
                // The notch.
                fb.line(tx + TILE / 2, ty + TILE / 2, tx + TILE / 2 + 9, ty + TILE / 2 - 3, WATER + 1);
                fb.line(tx + TILE / 2, ty + TILE / 2, tx + TILE / 2 + 9, ty + TILE / 2 - 2, WATER + 1);
            }
        }
        Tile::Reeds => {
            fb.rect_fill(tx, ty, TILE, TILE, if (x + y) % 2 == 0 { BANK_L } else { BANK_D });
            for k in 0..6 {
                let rx = tx + 3 + k * 4;
                let top = ty + 2 + ((h >> (k * 3)) % 8) as i32;
                fb.rect_fill(rx - 1, top, 3, TILE - 2 - (top - ty), OUTLINE);
                fb.rect_fill(rx, top + 1, 1, TILE - 4 - (top - ty), if k % 2 == 0 { REEDS + 1 } else { REEDS });
                fb.rect_fill(rx - 1, top, 3, 4, MUD);
            }
        }
        Tile::Stone => {
            fb.rect_fill(tx, ty, TILE, TILE, if (x + y) % 2 == 0 { BANK_L } else { BANK_D });
            fb.rect_fill(tx + 3, ty + 6, TILE - 6, TILE - 9, OUTLINE);
            fb.rect_fill(tx + 4, ty + 7, TILE - 8, TILE - 11, STONE + 1);
            fb.rect_fill(tx + 5, ty + 8, TILE - 14, 4, STONE + 2);
            fb.rect_fill(tx + 4, ty + TILE - 7, TILE - 8, 3, STONE);
        }
    }
}

fn sprite_at(spr: &Sprite, px: (i32, i32)) -> (i32, i32) {
    (px.0 + (TILE - spr.w) / 2, px.1 + TILE - 3 - spr.h)
}

fn draw_frog(g: &Game, fb: &mut Framebuffer, f: &Frog, px: (i32, i32), hop: i32) {
    let spr = &g.art.frogs[(f.size - 1).clamp(0, 2) as usize];
    let (sx, sy) = sprite_at(spr, px);
    let sy = sy - hop;
    let map = if g.hurt_flash > 0 {
        art::flat_map(BAD)
    } else if f.has(Mutation::PoisonSkin) {
        art::poison_map()
    } else {
        crate::framebuffer::IDENTITY
    };
    fb.rect_checker(px.0 + 6, px.1 + TILE - 4, TILE - 12, 2, OUTLINE, 0);
    // Spring legs: longer back legs.
    if f.has(Mutation::SpringLegs) {
        fb.line(sx - 2, sy + spr.h - 1, sx + 2, sy + spr.h - 5, OUTLINE);
        fb.line(sx + spr.w + 1, sy + spr.h - 1, sx + spr.w - 3, sy + spr.h - 5, OUTLINE);
    }
    fb.blit_map(spr, sx, sy, false, &map);
    if f.has(Mutation::Gills) {
        fb.rect_fill(sx - 1, sy + spr.h / 3, 2, 3, PINK);
        fb.rect_fill(sx + spr.w - 1, sy + spr.h / 3, 2, 3, PINK);
    }
    if f.has(Mutation::Camouflage) {
        for k in 0..3 {
            fb.pset(sx + 3 + k * (spr.w / 3), sy + spr.h / 2 + (k % 2), FROG);
        }
    }
    if !f.throat.is_empty() || f.has(Mutation::BigThroat) {
        // A bulging throat sac while something's inside.
        let full = !f.throat.is_empty();
        let w = if full { spr.w / 2 } else { spr.w / 3 };
        let (cx, cy) = (sx + spr.w / 2 - w / 2, sy + spr.h / 2 + 1);
        fb.rect_fill(cx - 1, cy - 1, w + 2, 5, OUTLINE);
        fb.rect_fill(cx, cy, w, 3, if full { GOLD } else { FROG + 2 });
    }
}

fn draw_pond(g: &Game, fb: &mut Framebuffer) {
    let p = &g.pond;
    let anim = g.anim.as_ref();
    let mine = anim.map_or(false, |a| a.t < Anim::MINE);
    let salt = g.pond_no * 13 + g.lesson.map_or(0, |l| l as u32 + 50);

    // Top bar.
    let title = match g.lesson {
        Some(l) => LESSONS[l].title.to_string(),
        None => format!("{}  -  POND {}/{PONDS}", g.area_name(), g.pond_no),
    };
    fb.text(&title, 8, 5, TEXT, Some(OUTLINE), 1);
    let f = &p.frog;
    ui::hearts(fb, 396 - f.max_hearts * 9, 4, f.hearts, f.max_hearts);

    // Tiles (the board as it was, while the frog's own action plays).
    let tiles_from = match anim {
        Some(a) if mine => &a.prev,
        _ => p,
    };
    fb.rect_fill(BX - 2, BY - 2, N * TILE + 4, N * TILE + 4, OUTLINE);
    for y in 0..N {
        for x in 0..N {
            draw_tile(fb, tiles_from, (x, y), g.frame, salt);
        }
    }
    // The exit.
    if inb(p.exit) {
        let (tx, ty) = tile_xy(p.exit);
        if p.cleared && !p.exit_locked {
            fb.rect_outline(tx + 2, ty + 2, TILE - 4, TILE - 4, GOOD);
            fb.rect_outline(tx + 3, ty + 3, TILE - 6, TILE - 6, OUTLINE);
            fb.text("EXIT", tx + 2, ty + 10, GOOD, Some(OUTLINE), 1);
        } else {
            fb.corners(tx + 2, ty + 2, TILE - 4, TILE - 4, 4, TEXT_FAINT);
        }
    }
    // The frog's spawn: a jelly clutch of eggs.
    for e in &tiles_from.eggs {
        let (tx, ty) = tile_xy(*e);
        for (k, (ox, oy)) in [(8, 9), (14, 8), (19, 11), (10, 15), (16, 14), (12, 20), (18, 18)].iter().enumerate() {
            let (cx, cy) = (tx + ox, ty + oy);
            fb.circle(cx, cy, 3, OUTLINE);
            fb.rect_fill(cx - 2, cy - 2, 5, 5, WATER + 2);
            fb.rect_checker(cx - 2, cy - 2, 5, 5, HILITE, k as i32);
            fb.pset(cx, cy, OUTLINE);
        }
    }
    // Arrivals: ripples where something comes next turn.
    for a in &p.arrivals {
        let (cx, cy) = centre(a.pos);
        let r = 4 + ((g.frame / 4) % 3) as i32 * 3;
        fb.circle(cx, cy, r, WATER + 2);
        fb.circle(cx, cy, r + 4, WATER);
        let spr = g.art.of(a.kind);
        let (sx, sy) = sprite_at(spr, tile_xy(a.pos));
        fb.blit_map(spr, sx, sy, false, &art::flat_map(PANEL_LO));
    }

    let interactive = anim.is_none() && g.card == 0 && matches!(g.phase, Phase::Lesson | Phase::Pond);
    let hovered = g.board_square(g.mouse.0, g.mouse.1);

    if interactive {
        // Telegraphed strikes.
        for (q, _) in p.danger() {
            let (tx, ty) = tile_xy(q);
            fb.rect_hatch(tx + 1, ty + 1, TILE - 2, TILE - 2, BAD);
            fb.rect_hatch(tx + 2, ty + 1, TILE - 3, TILE - 2, BAD);
        }
        match g.mode {
            Mode::Hop => {
                for h in p.hop_targets() {
                    let (tx, ty) = tile_xy(h);
                    fb.corners(tx + 1, ty + 1, TILE - 2, TILE - 2, 5, HILITE);
                    fb.corners(tx + 2, ty + 2, TILE - 4, TILE - 4, 3, OUTLINE);
                }
            }
            Mode::Tongue | Mode::Spit => {
                let range = if g.mode == Mode::Tongue { f.tongue_range() } else { 4 };
                let col = if g.mode == Mode::Tongue { PINK } else { GOLD };
                let hover_dir = hovered.and_then(|h| g.line_dir(h));
                for d in DIRS8 {
                    let (end, _, _) = p.line_hit(d, range);
                    let solid = Some(d) == hover_dir;
                    let mut k = 1;
                    loop {
                        let q = (f.pos.0 + d.0 * k, f.pos.1 + d.1 * k);
                        if !inb(q) || k > range {
                            break;
                        }
                        let (cx, cy) = centre(q);
                        if solid {
                            fb.rect_fill(cx - 2, cy - 2, 5, 5, OUTLINE);
                            fb.rect_fill(cx - 1, cy - 1, 3, 3, col);
                        } else {
                            fb.rect_fill(cx - 1, cy - 1, 3, 3, OUTLINE);
                            fb.pset(cx, cy, col);
                        }
                        if q == end {
                            break;
                        }
                        k += 1;
                    }
                    if solid {
                        let (tx, ty) = tile_xy(end);
                        fb.rect_outline(tx, ty, TILE, TILE, col);
                    }
                }
            }
        }
    }

    // Creatures.
    let shown: Vec<(Creature, (i32, i32))> = match anim {
        Some(a) if mine => a
            .prev
            .creatures
            .iter()
            .map(|c| {
                let to = a.mid.creatures.iter().find(|m| m.id == c.id).map(|m| m.pos).unwrap_or(c.pos);
                (c.clone(), lerp(tile_xy(c.pos), tile_xy(to), a.t + 1, Anim::MINE))
            })
            .filter(|(c, _)| a.mid.creatures.iter().any(|m| m.id == c.id) || a.t < 2)
            .collect(),
        Some(a) => p
            .creatures
            .iter()
            .map(|c| {
                let from = a.mid.creatures.iter().find(|m| m.id == c.id).map(|m| m.pos).unwrap_or(c.pos);
                (c.clone(), lerp(tile_xy(from), tile_xy(c.pos), a.t - Anim::MINE + 1, Anim::THEIRS - 2))
            })
            .collect(),
        None => p.creatures.iter().map(|c| (c.clone(), tile_xy(c.pos))).collect(),
    };
    for (c, px) in &shown {
        let spr = g.art.of(c.kind);
        let (sx, sy) = sprite_at(spr, *px);
        let flip = c.pos.0 > p.frog.pos.0;
        if c.stunned {
            fb.blit_map(spr, sx, sy, flip, &art::flat_map(TEXT_FAINT));
            fb.text("*", px.0 + TILE - 8, px.1, GOLD, Some(OUTLINE), 1);
        } else {
            fb.blit(spr, sx, sy, flip);
        }
        if c.hp > 1 || c.kind.info().hp > 1 {
            for k in 0..c.hp {
                fb.rect_fill(px.0 + 2 + k * 4, px.1 + 1, 3, 3, OUTLINE);
                fb.pset(px.0 + 3 + k * 4, px.1 + 2, HEART);
            }
        }
        // Which way it will strike.
        if interactive && !c.stunned {
            if let Some(aim) = c.aim {
                let (cx, cy) = centre(c.pos);
                let (ax, ay) = (cx + aim.0 * 11, cy + aim.1 * 11);
                fb.rect_fill(ax - 2, ay - 2, 5, 5, OUTLINE);
                fb.rect_fill(ax - 1, ay - 1, 3, 3, BAD);
            }
        }
    }

    // The frog.
    let (fpx, hop) = match anim {
        Some(a) if mine => {
            let (from, to) = (tile_xy(a.prev.frog.pos), tile_xy(a.mid.frog.pos));
            (lerp(from, to, a.t + 1, Anim::MINE), if a.prev.frog.pos != a.mid.frog.pos && a.t < 3 { 4 - a.t as i32 } else { 0 })
        }
        Some(a) => (lerp(tile_xy(a.mid.frog.pos), tile_xy(p.frog.pos), a.t - Anim::MINE + 1, 2), 0),
        None => (tile_xy(p.frog.pos), 0),
    };
    let frog_now = match anim {
        Some(a) if mine => &a.prev.frog,
        _ => &p.frog,
    };
    draw_frog(g, fb, frog_now, fpx, hop);

    // Action effects.
    if let Some(a) = anim {
        if mine {
            for e in &a.mine {
                match e {
                    Ev::Tongue { end, .. } => {
                        let (fx, fy) = centre(a.prev.frog.pos);
                        let (ex, ey) = centre(*end);
                        let k = if a.t <= 1 { a.t as i32 + 1 } else { (Anim::MINE as i32 - a.t as i32).max(0) };
                        let (tx, ty) = (fx + (ex - fx) * k / 2, fy + (ey - fy) * k / 2);
                        fb.line(fx, fy - 2, tx, ty, OUTLINE);
                        fb.line(fx, fy - 1, tx, ty + 1, PINK);
                        fb.rect_fill(tx - 1, ty - 1, 3, 3, PINK);
                    }
                    Ev::Spit { end, kind, .. } => {
                        let (fx, fy) = centre(a.prev.frog.pos);
                        let (ex, ey) = centre(*end);
                        let (sx, sy) = (fx + (ex - fx) * (a.t as i32 + 1) / Anim::MINE as i32, fy + (ey - fy) * (a.t as i32 + 1) / Anim::MINE as i32);
                        let spr = g.art.of(*kind);
                        fb.blit(spr, sx - spr.w / 2, sy - spr.h / 2, false);
                    }
                    Ev::Croak => {
                        let (cx, cy) = centre(a.prev.frog.pos);
                        fb.circle(cx, cy, 8 + a.t as i32 * 6, ACCENT_HI);
                    }
                    Ev::Died { at, .. } if a.t >= 2 => {
                        let (cx, cy) = centre(*at);
                        fb.circle(cx, cy, 3 + a.t as i32 * 2, WATER + 2);
                    }
                    _ => {}
                }
            }
        } else {
            let k = a.t - Anim::MINE;
            for e in &a.theirs {
                match e {
                    Ev::Strike { squares, .. } if k < 3 => {
                        for q in squares {
                            let (tx, ty) = tile_xy(*q);
                            fb.rect_outline(tx, ty, TILE, TILE, WHITE);
                            fb.rect_outline(tx + 1, ty + 1, TILE - 2, TILE - 2, BAD);
                        }
                    }
                    Ev::Died { at, .. } => {
                        let (cx, cy) = centre(*at);
                        fb.circle(cx, cy, 3 + k as i32 * 2, WATER + 2);
                    }
                    _ => {}
                }
            }
        }
    }

    if interactive {
        if let Some(h) = hovered {
            let (tx, ty) = tile_xy(h);
            fb.rect_outline(tx, ty, TILE, TILE, WHITE);
        }
    }

    draw_panel(g, fb, hovered.filter(|_| interactive));
    for (i, (b, label)) in BUTTONS.iter().enumerate() {
        let r = button_rect(i);
        let active = (*b == Button::Tongue && g.mode == Mode::Tongue) || (*b == Button::Spit && g.mode == Mode::Spit);
        if active {
            fb.rect_bevel(r.x, r.y, r.w, r.h, ACCENT, ACCENT_HI, PANEL_LO);
            fb.text_centered(label, r.x + r.w / 2, r.y + 7, PANEL_DEEP, None, 1);
        } else {
            let label = if *b == Button::Croak && p.frog.croak_cd > 0 { format!("CROAK ({})", p.frog.croak_cd) } else { label.to_string() };
            ui::button(fb, r, &label, r.contains(g.mouse.0, g.mouse.1), g.can(*b));
        }
    }
    draw_bar(g, fb);

    if g.card > 0 {
        fb.rect_checker(BX, BY, N * TILE, N * TILE, OUTLINE, 0);
        let r = Rect::new(BX + 20, BY + 82, N * TILE - 40, 56);
        ui::panel(fb, r);
        let (a, b) = match g.lesson {
            Some(l) => ("LESSONS".to_string(), LESSONS[l].title.split(": ").nth(1).unwrap_or("").to_string()),
            None => (AREAS[gen::area_of(g.pond_no)].name.to_string(), format!("POND {} OF {PONDS}", g.pond_no)),
        };
        fb.text_centered(&a, r.x + r.w / 2, r.y + 12, ACCENT_HI, Some(OUTLINE), 1);
        fb.text_centered(&b, r.x + r.w / 2, r.y + 26, TEXT, None, 1);
        fb.text_centered(&format!("MUSIC: {}", g.track().title()), r.x + r.w / 2, r.y + 40, TEXT_FAINT, None, 1);
    }
}

fn tongue_preview(g: &Game, d: Pos, spit: bool) -> (String, u8) {
    let p = &g.pond;
    let f = &p.frog;
    if spit {
        let Some(&k) = f.throat.first() else { return ("NOTHING TO SPIT.".into(), TEXT_DIM) };
        let (end, who, _) = p.line_hit(d, 4);
        return match who {
            Some(i) => {
                let c = &p.creatures[i];
                let mut dmg = 1 + k.info().size + if f.has(Mutation::AcidSpit) { 1 } else { 0 };
                if c.kind == Kind::Crayfish {
                    dmg -= 1;
                }
                (format!("SPIT THE {} AT THE {}: {} DAMAGE, AND IT'S SHOVED ONE SQUARE.", k.info().name, c.kind.info().name, dmg), GOLD)
            }
            None => (format!("SPIT THE {}: IT LANDS AT {}, DAZED.", k.info().name, sq(end).to_uppercase()), TEXT),
        };
    }
    let (end, who, solid) = p.line_hit(d, f.tongue_range());
    match who {
        Some(i) => {
            let c = &p.creatures[i];
            let size = c.kind.info().size;
            if size < f.size && f.throat.len() < f.throat_cap() {
                (format!("SWALLOW THE {}.", c.kind.info().name), GOOD)
            } else if size < f.size {
                (format!("YOUR THROAT IS FULL: YANK THE {} NEXT TO YOU.", c.kind.info().name), TEXT)
            } else if size == f.size {
                let to = (f.pos.0 + d.0, f.pos.1 + d.1);
                let drown = p.tile(to) == Tile::Water && !c.kind.info().swims && !c.kind.info().flies;
                (format!("THE {} IS YOUR SIZE: YANK IT NEXT TO YOU{}", c.kind.info().name, if drown { ", INTO THE WATER." } else { "." }), TEXT)
            } else {
                (format!("THE {} IS BIGGER: IT PULLS YOU OVER TO IT.", c.kind.info().name), TEXT)
            }
        }
        None if solid => (format!("GRAB THE {} AND PULL YOURSELF OVER.", p.tile(end).name()), TEXT),
        None => ("NOTHING IN REACH THAT WAY.".into(), TEXT_DIM),
    }
}

fn draw_panel(g: &Game, fb: &mut Framebuffer, hovered: Option<Pos>) {
    let p = &g.pond;
    let f = &p.frog;
    ui::panel(fb, Rect::new(PX, BY - 2, PW, 150));
    let x0 = PX + 6;
    let cols = ((PW - 12) / 6) as usize;
    fb.text(&format!("{} FROG", f.size_name()), x0, BY + 3, ACCENT_HI, Some(OUTLINE), 1);
    // Growth toward the next size.
    if f.size < 3 {
        let need = GROW_AT[(f.size - 1) as usize];
        let have = f.growth.min(need);
        fb.text("GROW", x0, BY + 14, TEXT_DIM, None, 1);
        for k in 0..need {
            let c = if k < have { GOOD } else { PANEL_DEEP };
            fb.rect_fill(x0 + 28 + k * 6, BY + 14, 5, 7, OUTLINE);
            fb.rect_fill(x0 + 29 + k * 6, BY + 15, 3, 5, c);
        }
    } else {
        fb.text("FULLY GROWN", x0, BY + 14, TEXT_DIM, None, 1);
    }
    let throat = if f.throat.is_empty() { "EMPTY".to_string() } else { f.throat.iter().map(|k| k.info().name).collect::<Vec<_>>().join(", ") };
    fb.text(&format!("THROAT: {throat}"), x0, BY + 25, TEXT, None, 1);
    let turn = format!("{}{}", if f.hop_left { "HOP " } else { "" }, if f.act_left { "ACT" } else { "" });
    fb.text(&format!("LEFT THIS TURN: {}", if turn.is_empty() { "-".into() } else { turn }), x0, BY + 35, TEXT_DIM, None, 1);
    let mut y = BY + 47;
    fb.rect_fill(PX + 4, y - 3, PW - 8, 1, PANEL_LO);
    let say = |fb: &mut Framebuffer, y: &mut i32, s: &str, col: u8| {
        for l in wrap(s, cols) {
            if *y < BY + 142 {
                fb.text(&l, x0, *y, col, None, 1);
            }
            *y += 9;
        }
    };
    let Some(h) = hovered else {
        if !f.muts.is_empty() {
            let m = f.muts.iter().map(|m| m.name()).collect::<Vec<_>>().join(", ");
            say(fb, &mut y, &m, GOLD);
        } else {
            say(fb, &mut y, "POINT AT ANYTHING.", TEXT_FAINT);
        }
        return;
    };
    if matches!(g.mode, Mode::Tongue | Mode::Spit) {
        if let Some(d) = g.line_dir(h) {
            let (t, col) = tongue_preview(g, d, g.mode == Mode::Spit);
            say(fb, &mut y, &t, col);
            return;
        }
    }
    if h == f.pos {
        say(fb, &mut y, "YOU. CLICK YOURSELF TO END YOUR TURN.", TEXT);
        let hits = p.preview_hop(f.pos);
        if hits > 0 {
            say(fb, &mut y, &format!("STAYING HERE: YOU'D BE HIT FOR {hits}."), BAD);
        }
        return;
    }
    if let Some(i) = p.at(h) {
        let c = &p.creatures[i];
        let info = c.kind.info();
        fb.text(info.name, x0, y, ACCENT_HI, Some(OUTLINE), 1);
        y += 11;
        let rel = if info.size < f.size {
            "SMALLER THAN YOU: SWALLOWABLE."
        } else if info.size == f.size {
            "YOUR SIZE: YOUR TONGUE YANKS IT."
        } else {
            "BIGGER: YOUR TONGUE PULLS YOU TO IT."
        };
        say(fb, &mut y, info.text, TEXT);
        say(fb, &mut y, rel, TEXT_DIM);
        if c.stunned {
            say(fb, &mut y, "DAZED: IT SKIPS ITS NEXT STRIKE.", GOLD);
        }
        return;
    }
    if p.eggs.contains(&h) {
        say(fb, &mut y, "YOUR FROGSPAWN. PREDATORS GO FOR IT TOO. A STRIKE ON IT, OR A CREATURE SHOVED INTO IT, COSTS YOU A HEART. EACH CLUTCH THAT SURVIVES THE POND HEALS YOU 1.", TEXT);
        if p.danger().iter().any(|(q, _)| *q == h) {
            say(fb, &mut y, "IT WILL BE STRUCK WHEN YOUR TURN ENDS!", BAD);
        }
        return;
    }
    if let Some(a) = p.arrivals.iter().find(|a| a.pos == h) {
        say(fb, &mut y, &format!("RIPPLES: A {} ARRIVES HERE NEXT TURN.", a.kind.info().name), TEXT);
        return;
    }
    if p.hop_targets().contains(&h) {
        let hits = p.preview_hop(h);
        if hits == 0 {
            say(fb, &mut y, "HOP HERE: SAFE WHEN YOUR TURN ENDS.", GOOD);
        } else {
            say(fb, &mut y, &format!("HOP HERE: YOU'D BE HIT FOR {hits} WHEN YOUR TURN ENDS."), BAD);
        }
    }
    if h == p.exit {
        let t = if p.exit_locked {
            "THE EXIT. IT OPENS WHEN THE LESSON IS DONE."
        } else if p.cleared {
            "THE EXIT. HOP HERE TO LEAVE."
        } else {
            "THE EXIT. IT OPENS WHEN NO PREDATORS ARE LEFT."
        };
        say(fb, &mut y, t, TEXT);
    } else {
        say(fb, &mut y, &format!("{}: {}", p.tile(h).name(), p.tile(h).text()), TEXT_DIM);
    }
    if p.danger().iter().any(|(q, _)| *q == h) {
        say(fb, &mut y, "STRUCK WHEN YOUR TURN ENDS.", BAD);
    }
}

fn draw_bar(g: &Game, fb: &mut Framebuffer) {
    let r = Rect::new(2, 248, 396, 50);
    ui::inset(fb, r);
    let mut y = 252;
    if let Some(l) = g.lesson {
        if let Some((text, _)) = LESSONS[l].steps.get(g.lesson_step) {
            fb.text(&format!("STEP {} OF {}", g.lesson_step + 1, LESSONS[l].steps.len()), 8, y, ACCENT, None, 1);
            y += 10;
            for line in wrap(text, 64).iter().take(3) {
                fb.text(line, 8, y, TEXT, None, 1);
                y += 9;
            }
        }
    } else if let Some(m) = &g.message {
        for line in wrap(m, 64).iter().take(3) {
            fb.text(line, 8, y, TEXT, None, 1);
            y += 9;
        }
    }
    let hint = match g.mode {
        Mode::Hop => "CLICK: HOP.  RIGHT-CLICK A LINE: TONGUE.  F1: RULES",
        Mode::Tongue => "CLICK A SQUARE IN ONE OF THE 8 LINES TO SHOOT YOUR TONGUE.",
        Mode::Spit => "CLICK A SQUARE IN ONE OF THE 8 LINES TO SPIT.",
    };
    if y <= 280 {
        fb.text(hint, 8, 287, TEXT_FAINT, None, 1);
    }
}

fn draw_bank(g: &Game, fb: &mut Framebuffer, offers: &[Option<Mutation>]) {
    fb.text_centered("THE BANK", 200, 10, ACCENT_HI, Some(OUTLINE), 2);
    let sub = if g.pond_no == 0 { "BEFORE THE FIRST POND".to_string() } else { format!("AFTER POND {} OF {PONDS}", g.pond_no) };
    fb.text_centered(&sub, 200, 30, TEXT_DIM, None, 1);
    let f = &g.frog;
    ui::hearts(fb, 20, 46, f.hearts, f.max_hearts);
    fb.text(&format!("{} FROG", f.size_name()), 30 + f.max_hearts * 9, 47, TEXT, None, 1);
    let spr = &g.art.frogs[(f.size - 1).clamp(0, 2) as usize];
    let fake = (330, 40);
    draw_frog(g, fb, f, fake, 0);
    let _ = spr;
    fb.text("CHOOSE ONE MUTATION. IT LASTS THE WHOLE RUN.", 20, 64, TEXT, None, 1);
    for (i, o) in offers.iter().enumerate() {
        let r = mutation_rect(i);
        let hover = g.hover_item == Some(i);
        if hover {
            fb.rect_bevel(r.x, r.y, r.w, r.h, PANEL_HI, WHITE, PANEL);
        } else {
            ui::panel(fb, r);
        }
        let (name, text) = match o {
            Some(m) => (m.name(), m.text()),
            None => ("MEND", "HEAL TO FULL HEARTS INSTEAD."),
        };
        fb.text(name, r.x + 10, r.y + 8, if hover { PANEL_DEEP } else { ACCENT_HI }, None, 1);
        for (k, l) in wrap(text, 52).iter().take(2).enumerate() {
            fb.text(l, r.x + 10, r.y + 20 + k as i32 * 9, if hover { PANEL_DEEP } else { TEXT }, None, 1);
        }
    }
    if let Some(m) = &g.message {
        for (k, l) in wrap(m, 62).iter().take(2).enumerate() {
            fb.text(l, 20, 284 - 9 + k as i32 * 9, TEXT_DIM, None, 1);
        }
    }
}

fn draw_over(g: &Game, fb: &mut Framebuffer, won: bool) {
    fb.rect_checker(0, 0, 400, 300, OUTLINE, 0);
    let r = Rect::new(60, 60, 280, 220);
    ui::panel(fb, r);
    let (head, sub) = if won {
        ("GROF IS DEFEATED", "THE POND IS QUIET. THE POND IS YOURS.".to_string())
    } else {
        ("THE FROG FELL", format!("IN {}, POND {} OF {PONDS}.", g.area_name(), g.pond_no))
    };
    fb.text_centered(head, 200, 72, if won { GOLD } else { BAD }, Some(OUTLINE), 2);
    fb.text_centered(&sub, 200, 94, TEXT, None, 1);
    let s = &g.stats;
    for (i, l) in [format!("ATE {}   DEFEATED {}", s.eaten, s.defeated), format!("HURT {} TIMES OVER {} TURNS", s.hurt, s.turns), format!("PONDS {}   CLUTCHES SAVED {}  LOST {}", s.ponds, s.eggs_saved, s.eggs_lost)].iter().enumerate() {
        fb.text_centered(l, 200, 112 + i as i32 * 11, TEXT_DIM, None, 1);
    }
    let f = &g.frog;
    if !f.muts.is_empty() {
        let m = format!("{} FROG: {}", f.size_name(), f.muts.iter().map(|m| m.name()).collect::<Vec<_>>().join(", "));
        for (i, l) in wrap(&m, 42).iter().take(3).enumerate() {
            fb.text_centered(l, 200, 152 + i as i32 * 10, TEXT_DIM, None, 1);
        }
    }
    draw_frog(g, fb, f, (186, 196), 0);
    ui::button(fb, OVER_GO, "CONTINUE", OVER_GO.contains(g.mouse.0, g.mouse.1), true);
}

fn draw_help(fb: &mut Framebuffer) {
    fb.rect_checker(0, 0, 400, 300, OUTLINE, 0);
    let r = Rect::new(12, 10, 376, 280);
    ui::panel(fb, r);
    fb.text_centered("HOW THE POND WORKS", 200, 18, ACCENT_HI, Some(OUTLINE), 1);
    let rules = [
        "EACH TURN: ONE HOP AND ONE ACTION, IN EITHER ORDER. CLICK YOURSELF OR PRESS SPACE TO END THE TURN.",
        "HOP: ONE SQUARE ANY WAY. YOU SWIM. YOUR TONGUE CAN ALSO PULL YOU ACROSS (THAT USES YOUR ACTION).",
        "TONGUE (T): THE FIRST THING IN A LINE. SMALLER: SWALLOWED. YOUR SIZE: YANKED NEXT TO YOU. BIGGER, OR REEDS AND STONE: PULLS YOU OVER.",
        "SPIT (S): SHOOT WHAT YOU SWALLOWED. 1 DAMAGE PLUS ITS SIZE, AND A SHOVE. GULP (G): DIGEST IT TO GROW. GROWING A SIZE HEALS 1.",
        "CROAK (C): SHOVE EVERYTHING NEXT TO YOU A SQUARE. SHOVED INTO WATER, MOST THINGS DROWN; INTO A WALL OR EACH OTHER, THEY TAKE 1.",
        "RED SQUARES: STRIKES THAT LAND WHEN YOUR TURN ENDS, FROM WHEREVER THE STRIKER STANDS THEN.",
        "YOUR FROGSPAWN: A STRIKE ON IT, OR A CREATURE SHOVED INTO IT, COSTS A HEART. EACH CLUTCH THAT SURVIVES A POND HEALS 1.",
        "CLEAR THE PREDATORS, THEN HOP TO THE EXIT. BETWEEN PONDS, CHOOSE A MUTATION.",
    ];
    let mut y = 32;
    for rule in rules {
        for (i, l) in wrap(rule, 58).iter().enumerate() {
            fb.text(if i == 0 { "-" } else { " " }, 22, y, ACCENT, None, 1);
            fb.text(l, 32, y, TEXT, None, 1);
            y += 9;
        }
        y += 3;
    }
    fb.text("F2 SCANLINES  F3 MUSIC  ESC PAUSE", 22, y + 4, TEXT_DIM, None, 1);
    fb.text_centered("PRESS ANY KEY OR CLICK TO GO BACK", 200, 276, ACCENT, None, 1);
}
