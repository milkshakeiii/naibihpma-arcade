use crate::framebuffer::{Framebuffer, BAYER4};
use crate::game::{hash, Game, PState, Phase, PH, PW, VIEW_H, VIEW_W};
use crate::level::{Level, Tile, TS};
use crate::palette::*;
use crate::sprites::Sprites;

pub fn draw(g: &Game, sp: &Sprites, fb: &mut Framebuffer) {
    draw_background(g, fb);
    draw_tiles(g, fb, false);
    draw_motes(g, fb);
    draw_drops(g, sp, fb);
    draw_player(g, sp, fb);
    draw_water(g, fb);
    draw_tiles(g, fb, true);
    draw_effects(g, fb);
    if g.debug {
        draw_debug(g, fb);
    }
    draw_hud(g, sp, fb);
    match g.phase {
        Phase::Title => draw_title(g, fb),
        Phase::Tally => draw_tally(g, fb),
        Phase::Playing => {}
    }
}

fn tri(x: i32, period: i32, amp: i32) -> i32 {
    let m = x.rem_euclid(period);
    let half = period / 2;
    (if m < half { m } else { period - m }) * amp / half.max(1)
}

fn draw_background(g: &Game, fb: &mut Framebuffer) {
    let (cx, cy) = (g.cam_x, g.cam_y);
    let mood = g.level.sections[g.section].mood;
    for sy in 0..VIEW_H {
        let grad = ((sy + cy / 3) * 64 / (VIEW_H + 80)).clamp(0, 63);
        let (band, frac) = ((grad / 16) as u8, (grad % 16) as u8);
        let bayer = &BAYER4[(sy & 3) as usize];
        for sx in 0..VIEW_W {
            let c = if bayer[(sx & 3) as usize] < frac { SKY + band + 1 } else { SKY + band };
            fb.pset(sx, sy, c);
        }
    }

    // Two parallax layers of distant forest.
    for sx in 0..VIEW_W {
        let f1 = sx + cx / 4;
        let top1 = 120 - cy / 4 - tri(f1, 70, 26) - tri(f1 + 31, 26, 8);
        let f2 = sx + cx / 2;
        let top2 = 176 - cy / 2 - tri(f2, 52, 30) - tri(f2 + 13, 19, 9);
        let trunk = f2.rem_euclid(48) / 3 == 7;
        for sy in top1.max(0)..VIEW_H {
            fb.pset(sx, sy, FAR + 2);
        }
        for sy in top2.max(0)..VIEW_H {
            fb.pset(sx, sy, if trunk && sy > top2 + 12 { FAR + 1 } else { FAR });
        }
    }

    // Ambient atmosphere: drifting mist bands at night, still sun shafts in the clearing.
    let misty = matches!(mood, Mood::Mist | Mood::Swamp | Mood::Bog);
    let sunny = mood == Mood::Clearing;
    if misty || sunny {
        for sy in 0..VIEW_H {
            let wy = sy + cy;
            for sx in 0..VIEW_W {
                let wx = sx + cx;
                let b = BAYER4[(sy & 3) as usize][(sx & 3) as usize] as i32;
                if misty {
                    let k = wy.div_euclid(96);
                    let dy = (wy.rem_euclid(96) - 48).abs();
                    let drift = (g.tick / 6) as i32 + k * 37;
                    let density = (5 - dy / 3).max(0) + tri(wx + drift, 140, 4) - 2;
                    if dy < 15 && b < density {
                        fb.pset(sx, sy, if dy < 6 { MIST + 1 } else { MIST });
                    }
                } else {
                    let d = (sx + sy / 2 + cx / 3).rem_euclid(110);
                    if d < 18 && b < if (6..12).contains(&d) { 5 } else { 3 } {
                        fb.pset(sx, sy, if (6..12).contains(&d) { SUN + 1 } else { SUN });
                    }
                }
            }
        }
    }

    // Cavern back wall and the murky bog below the water line.
    let lvl = &g.level;
    for sy in 0..VIEW_H {
        let wy = sy + cy;
        for sx in 0..VIEW_W {
            let wx = sx + cx;
            let (tx, ty) = (wx.div_euclid(TS), wy.div_euclid(TS));
            if wy >= g.water_top && !lvl.tile(tx, ty).solid() {
                fb.pset(sx, sy, WATER);
            } else if lvl.in_cave(tx, ty) && lvl.tile(tx, ty) == Tile::Empty {
                let h = hash(wx / 4, wy / 3);
                let c = if wy % 9 == 0 {
                    CAVE + 2
                } else if h % 6 == 0 {
                    CAVE
                } else if h % 11 == 0 {
                    CAVE + 2
                } else {
                    CAVE + 1
                };
                fb.pset(sx, sy, c);
            }
        }
    }
}

/// Draws every visible tile; lily pads go in a second pass above the water.
fn draw_tiles(g: &Game, fb: &mut Framebuffer, lilies: bool) {
    let lvl = &g.level;
    for ty in g.cam_y.div_euclid(TS)..=(g.cam_y + VIEW_H).div_euclid(TS) {
        for tx in g.cam_x.div_euclid(TS)..=(g.cam_x + VIEW_W).div_euclid(TS) {
            let t = lvl.tile(tx, ty);
            if (t == Tile::Lily) != lilies || t == Tile::Empty {
                continue;
            }
            for ly in 0..TS {
                let sy = ty * TS + ly - g.cam_y;
                if !(0..VIEW_H).contains(&sy) {
                    continue;
                }
                for lx in 0..TS {
                    let sx = tx * TS + lx - g.cam_x;
                    if !(0..VIEW_W).contains(&sx) {
                        continue;
                    }
                    if let Some(c) = tile_px(lvl, t, tx, ty, lx, ly) {
                        fb.pset(sx, sy, c);
                    }
                }
            }
        }
    }
}

fn tile_px(lvl: &Level, t: Tile, tx: i32, ty: i32, lx: i32, ly: i32) -> Option<u8> {
    let (wx, wy) = (tx * TS + lx, ty * TS + ly);
    match t {
        Tile::Soil => {
            let up = lvl.tile(tx, ty - 1).solid();
            let edge = (!lvl.tile(tx - 1, ty).solid() && lx == 0)
                || (!lvl.tile(tx + 1, ty).solid() && lx == TS - 1)
                || (!lvl.tile(tx, ty + 1).solid() && ly == TS - 1);
            if !up && ly == 0 {
                return Some(OUTLINE);
            }
            if edge {
                return Some(OUTLINE);
            }
            if !up {
                let drip = 3 + (hash(wx / 2, ty) % 5) as i32;
                if ly == 1 {
                    return Some(MOSS + 2);
                }
                if ly == 2 || (ly <= drip && hash(wx / 2, ty) % 3 == 0) {
                    return Some(MOSS + 1);
                }
                if ly == 3 {
                    return Some(MOSS);
                }
            }
            let h = hash(wx / 3, wy / 3);
            Some(match h % 9 {
                0 => SOIL,
                1 => SOIL + 2,
                2 if (wy / 3) % 2 == 0 => SOIL + 3,
                _ => SOIL + 1,
            })
        }
        Tile::Bark => {
            let bark = |t: Tile| t == Tile::Bark;
            if (!bark(lvl.tile(tx - 1, ty)) && !lvl.tile(tx - 1, ty).solid() && lx == 0)
                || (!bark(lvl.tile(tx + 1, ty)) && !lvl.tile(tx + 1, ty).solid() && lx == TS - 1)
            {
                return Some(OUTLINE);
            }
            if !lvl.tile(tx, ty - 1).solid() {
                if ly == 0 {
                    return Some(OUTLINE);
                }
                if ly == 1 {
                    return Some(BARK + 3);
                }
            }
            let knot = hash(tx, ty) % 4 == 0 && (lx - 10).pow(2) + (ly - 9).pow(2) <= 5;
            if knot {
                return Some(if (lx - 10).pow(2) + (ly - 9).pow(2) <= 1 { BARK + 3 } else { BARK });
            }
            if (wy + (hash(wx, 0) % 23) as i32) % 23 == 0 {
                return Some(BARK);
            }
            const STRIPES: [u8; 9] = [0, 1, 2, 2, 1, 3, 2, 1, 0];
            Some(BARK + STRIPES[wx.rem_euclid(9) as usize])
        }
        Tile::Branch => {
            let joins = |t: Tile| t == Tile::Branch || t.solid();
            let left_end = !joins(lvl.tile(tx - 1, ty)) && lx == 0;
            let right_end = !joins(lvl.tile(tx + 1, ty)) && lx == TS - 1;
            if ly <= 6 {
                if left_end || right_end || ly == 0 || ly == 6 {
                    return Some(OUTLINE);
                }
                if (2..5).contains(&ly) && hash(wx / 4, ty) % 5 == 0 {
                    return Some(BARK);
                }
                return Some(match ly {
                    1 => BARK + 3,
                    2 => BARK + 2,
                    5 => BARK,
                    _ => BARK + 1,
                });
            }
            // A few leaves dangling under the branch.
            let h = hash(tx, ty * 7);
            let lx0 = 4 + (h % 12) as i32;
            let len = 3 + (h / 16 % 6) as i32;
            if h % 3 != 0 && ly < 7 + len {
                let d = (lx - lx0).abs();
                if d == 0 {
                    return Some(LEAF + 1);
                }
                if d == 1 && ly < 6 + len {
                    return Some(if lx > lx0 { LEAF } else { LEAF + 2 });
                }
                if d == 2 && ly < 6 + len {
                    return Some(OUTLINE);
                }
                if d <= 1 && ly == 7 + len - 1 {
                    return Some(OUTLINE);
                }
            }
            None
        }
        Tile::Lily => {
            if ly > 4 || (lx == 0 || lx == TS - 1) && (ly == 0 || ly == 4) || (lx == 9 || lx == 10) && ly <= 2 {
                return None;
            }
            Some(match ly {
                0 | 4 => OUTLINE,
                _ if lx == 0 || lx == TS - 1 => OUTLINE,
                1 => LEAF + 2,
                _ => LILY,
            })
        }
        Tile::Frond => {
            let leaf = |x: i32, y: i32| frond_leaf(lvl, x, y);
            if leaf(wx, wy) {
                // Each 3px cell is a small leaf lit from above: light top row, dark bottom row.
                let h = hash(wx.div_euclid(3), wy.div_euclid(3));
                let row = wy.rem_euclid(3);
                Some(if !leaf(wx, wy - 1) {
                    LEAF + 3
                } else if !leaf(wx, wy + 1) || (row == 2 && h % 3 == 0) {
                    LEAF
                } else if row == 0 && h % 2 == 0 {
                    LEAF + 2
                } else {
                    LEAF + 1
                })
            } else if leaf(wx - 1, wy) || leaf(wx + 1, wy) || leaf(wx, wy - 1) || leaf(wx, wy + 1) {
                Some(OUTLINE)
            } else {
                None
            }
        }
        Tile::Reed => {
            for (i, sx) in [5, 9, 14].into_iter().enumerate() {
                let height = 11 + (hash(tx * 3 + i as i32, ty) % 7) as i32;
                let top = TS - height;
                if lx == sx && ly >= top {
                    return Some(REED + 1);
                }
                if lx == sx + 1 && ly >= top + 4 {
                    return Some(REED);
                }
                if (lx - sx).abs() <= 1 && ly >= top - 4 && ly < top {
                    return Some(if (lx - sx).abs() == 1 { OUTLINE } else { BARK + 1 });
                }
            }
            None
        }
        Tile::Empty => None,
    }
}

/// Chunky 3px leaf cells inside frond tiles, ragged where the frond meets open air.
fn frond_leaf(lvl: &Level, wx: i32, wy: i32) -> bool {
    let (tx, ty) = (wx.div_euclid(TS), wy.div_euclid(TS));
    if lvl.tile(tx, ty) != Tile::Frond {
        return false;
    }
    let (lx, ly) = (wx.rem_euclid(TS), wy.rem_euclid(TS));
    let open = |dx: i32, dy: i32| lvl.tile(tx + dx, ty + dy) != Tile::Frond;
    if (open(-1, 0) && lx == 0) || (open(1, 0) && lx == TS - 1) || (open(0, -1) && ly == 0) || (open(0, 1) && ly == TS - 1) {
        return false;
    }
    let h = hash(wx.div_euclid(3), wy.div_euclid(3));
    let near_edge = (open(-1, 0) && lx < 4) || (open(1, 0) && lx > TS - 5) || (open(0, -1) && ly < 4) || (open(0, 1) && ly > TS - 5);
    if near_edge {
        h % 2 == 0
    } else {
        h % 13 != 0
    }
}

/// Spores in the mist and cavern, fireflies over the swamp: slow drifting points, no blinking.
fn draw_motes(g: &Game, fb: &mut Framebuffer) {
    let mood = g.level.sections[g.section].mood;
    if mood == Mood::Clearing {
        return;
    }
    let (count, col) = match mood {
        Mood::Swamp => (40, GLOW + 2),
        Mood::Cavern => (26, GLOW + 1),
        _ => (22, GLOW),
    };
    let world_w = g.level.w * TS;
    let world_h = g.level.h * TS;
    for i in 0..count {
        let h = hash(i, 911);
        let phase = (h % 64) as i32;
        let t = (g.tick / 4) as i32 + phase;
        let x = (h % world_w as u32) as i32 + tri(t, 40, 12) - 6;
        let y = ((h / 1024) % world_h as u32) as i32 + tri(t + 9, 28, 8) - 4;
        let (sx, sy) = (x - g.cam_x, y - g.cam_y);
        if (0..VIEW_W).contains(&sx) && (0..VIEW_H).contains(&sy) && !g.level.tile(x / TS, y / TS).solid() {
            fb.pset(sx, sy, col);
        }
    }
}

fn draw_drops(g: &Game, sp: &Sprites, fb: &mut Framebuffer) {
    for (site, slot) in g.drops.iter().enumerate() {
        let Some(d) = slot else { continue };
        let shown = d.size.saturating_sub((d.swell + 2) / 4).max(1);
        let (x, y, _, _) = g.drop_rect(site, shown);
        fb.blit(sp.drop(shown, g.is_remembered(d.id)), x - g.cam_x, y - g.cam_y, false);
    }
}

fn draw_player(g: &Game, sp: &Sprites, fb: &mut Framebuffer) {
    let p = &g.player;
    let (x, y) = (p.x - g.cam_x, p.y - g.cam_y);
    if p.state == PState::Cling {
        let spr = &sp.cling[((p.anim / 2) % 2) as usize];
        let sx = if p.cling_dir > 0 { x + PW - spr.w } else { x };
        fb.blit(spr, sx, y + PH / 2 - spr.h / 2, p.cling_dir < 0);
        return;
    }
    let spr = match p.state {
        PState::Air => &sp.air,
        PState::Swim => &sp.walk[((g.tick / 4) % 2) as usize],
        _ if p.moving => &sp.walk[((p.anim / 2) % 2) as usize],
        _ => &sp.walk[0],
    };
    fb.blit(spr, x + PW / 2 - spr.w / 2, y + PH - spr.h, p.facing < 0);
}

/// Water is a dithered overlay (checkerboard), so whatever is submerged stays visible.
fn draw_water(g: &Game, fb: &mut Framebuffer) {
    const RIPPLE: [i32; 8] = [0, 0, 1, 1, 0, 0, -1, -1];
    let top = g.water_top;
    for sy in 0..VIEW_H {
        let wy = sy + g.cam_y;
        if wy < top - 1 {
            continue;
        }
        for sx in 0..VIEW_W {
            let wx = sx + g.cam_x;
            let tx = wx.div_euclid(TS);
            // Ripples may lift the surface 1px, but never above a floor that is dry at rest.
            if g.level.tile(tx, wy.div_euclid(TS)).solid() || (wy < top && g.level.tile(tx, top.div_euclid(TS)).solid()) {
                continue;
            }
            let surf = top + RIPPLE[((wx / 6 + (g.tick / 3) as i32) & 7) as usize];
            let checker = (sx + sy) & 1 == 0;
            if wy == surf {
                fb.pset(sx, sy, WATER + 3);
            } else if wy > surf && checker {
                fb.pset(sx, sy, if wy - top <= 2 { WATER + 2 } else { WATER + 1 });
            }
        }
    }
}

fn draw_effects(g: &Game, fb: &mut Framebuffer) {
    for p in &g.particles {
        fb.pset(p.x - g.cam_x, p.y - g.cam_y, p.col);
    }
    for r in &g.rings {
        fb.circle(r.x - g.cam_x, r.y - g.cam_y, r.r, DEW + 3);
    }
}

fn draw_debug(g: &Game, fb: &mut Framebuffer) {
    for (i, s) in g.level.sites.iter().enumerate() {
        let (sx, sy) = (s.tx * TS - g.cam_x, s.ty * TS - g.cam_y);
        let label = if s.exposed { "E" } else { "S" };
        fb.text(label, sx + 1, sy + 1, if s.exposed { SUN + 1 } else { TEXT_DIM }, Some(OUTLINE), 1);
        if let Some(d) = g.drops[i] {
            if s.exposed && d.exposure > 0 {
                fb.rect_fill(sx + 1, sy + TS - 2, (d.exposure as i32 * 18) / crate::game::EVAP_TICKS as i32, 1, SUN + 1);
            }
        }
    }
    let s = format!("{}.{}S SEC {} WATER {}", g.song_ms / 1000, g.song_ms / 100 % 10, g.section, g.water_top);
    fb.text(&s, 4, 4, TEXT, Some(OUTLINE), 1);
}

fn mmss(ms: u32) -> String {
    format!("{}:{:02}", ms / 60_000, ms / 1000 % 60)
}

fn draw_hud(g: &Game, sp: &Sprites, fb: &mut Framebuffer) {
    fb.rect_bevel(0, VIEW_H, 400, 300 - VIEW_H, HUD_FILL, HUD_HI, HUD_SHADOW);

    // Dew count in a recessed box.
    fb.rect_bevel(6, 247, 82, 46, HUD_SHADOW, HUD_SHADOW, HUD_MID);
    fb.blit(&sp.hud_drop, 14, 258, false);
    fb.text(&g.stats.gathered.to_string(), 36, 263, TEXT, Some(OUTLINE), 2);

    // Song-shape strip: one segment per section, tinted by its mood.
    let lvl = &g.level;
    let (x0, w, y) = (100, 288, 271);
    let len = lvl.len_ms.max(1) as i64;
    let px = |ms: u32| x0 + (ms as i64 * w as i64 / len) as i32;
    let sec = &lvl.sections[g.section];
    fb.text(&sec.name, x0, 253, TEXT, Some(OUTLINE), 1);
    let time = format!("{} / {}", mmss(g.song_ms), mmss(lvl.len_ms));
    fb.text(&time, x0 + w - Framebuffer::text_width(&time, 1), 253, TEXT_DIM, Some(OUTLINE), 1);
    fb.rect_fill(x0 - 1, y - 1, w + 2, 11, OUTLINE);
    for (i, s) in lvl.sections.iter().enumerate() {
        let (a, b) = (px(s.start_ms), px(lvl.section_end_ms(i)));
        fb.rect_fill(a, y, b - a, 9, s.mood.swatch());
        fb.rect_fill(a, y, 1, 9, OUTLINE);
    }
    let head = px(g.song_ms);
    if g.phase != Phase::Title {
        fb.rect_checker(x0, y, head - x0, 9, OUTLINE);
    }

    // Brackets joining each echo to the section it remembers.
    let mut depth = 0;
    for (i, s) in lvl.sections.iter().enumerate() {
        let Some(src) = s.echo_of else { continue };
        let mid = |k: usize| (px(lvl.sections[k].start_ms) + px(lvl.section_end_ms(k))) / 2;
        let (a, b) = (mid(src), mid(i));
        let by = y + 13 + depth * 4;
        fb.line(a, y + 10, a, by, TEXT_DIM);
        fb.line(a, by, b, by, TEXT_DIM);
        fb.line(b, by, b, y + 10, TEXT_DIM);
        depth += 1;
    }

    if g.phase != Phase::Title {
        fb.rect_fill(head, y - 3, 1, 15, HUD_DEW_LIGHT);
        fb.rect_fill(head - 1, y - 4, 3, 1, HUD_DEW_LIGHT);
        fb.rect_fill(head - 2, y - 5, 5, 1, HUD_DEW_LIGHT);
    }
}

fn overlay_box(fb: &mut Framebuffer, x: i32, y: i32, w: i32, h: i32) {
    fb.rect_checker(0, 0, VIEW_W, VIEW_H, OUTLINE);
    fb.rect_bevel(x, y, w, h, HUD_FILL, HUD_HI, HUD_SHADOW);
}

fn draw_title(g: &Game, fb: &mut Framebuffer) {
    overlay_box(fb, 40, 30, 320, 180);
    let c = 200;
    fb.text_centered("STAR POND", c, 42, TEXT, Some(OUTLINE), 3);
    fb.text_centered("EPISODE 1: THE ENCHANTED FOREST", c, 72, TEXT_DIM, None, 1);
    fb.text_centered("OF NAIBIHPMA", c, 82, TEXT_DIM, None, 1);
    fb.text_centered(&format!("1. {}", g.level.title), c, 100, HUD_DEW_LIGHT, Some(OUTLINE), 1);
    fb.text_centered("GATHER DEW UNTIL THE SONG ENDS", c, 122, TEXT, None, 1);
    fb.text_centered("ARROWS MOVE    Z / SPACE JUMP", c, 140, TEXT, None, 1);
    fb.text_centered("JUMP AT A WALL TO CLING, UP TO CLIMB", c, 152, TEXT, None, 1);
    if g.music_missing {
        fb.text_centered("MUSIC NOT FOUND - PLAYING SILENT", c, 170, SWATCH + 2, None, 1);
    }
    fb.text_centered("PRESS JUMP TO BEGIN", c, 190, HUD_DEW_LIGHT, Some(OUTLINE), 1);
}

fn draw_tally(g: &Game, fb: &mut Framebuffer) {
    overlay_box(fb, 60, 22, 280, 196);
    let s = &g.stats;
    fb.text_centered(&g.level.title, 200, 34, TEXT, Some(OUTLINE), 2);
    let rows: [(&str, u32, u8); 6] = [
        ("DEW GATHERED", s.gathered, TEXT),
        ("  GROWN BY ECHOES", s.echo_bonus, HUD_DEW_LIGHT),
        ("LOST TO THE SUN", s.lost_sun, SWATCH + 2),
        ("LOST TO THE BOG", s.lost_bog, SWATCH + 4),
        ("LEFT IN THE FOREST", s.left_behind, TEXT_DIM),
        ("BEST", g.best, TEXT),
    ];
    for (i, (label, v, col)) in rows.iter().enumerate() {
        let y = 66 + i as i32 * 16;
        fb.text(label, 80, y, *col, Some(OUTLINE), 1);
        let n = v.to_string();
        fb.text(&n, 320 - Framebuffer::text_width(&n, 1), y, *col, Some(OUTLINE), 1);
    }
    if g.new_best {
        fb.text_centered("NEW BEST", 200, 166, HUD_DEW_LIGHT, Some(OUTLINE), 1);
    }
    fb.text_centered("JUMP: PLAY AGAIN    ESC: QUIT", 200, 196, TEXT, Some(OUTLINE), 1);
}
