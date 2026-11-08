//! Drawing Changeling: the board, the shape panel, the message bar, the glade and the help page.

use super::art::{self, *};
use super::gen::{AREAS, BURROWS, LESSONS};
use super::rules::*;
use super::*;
use crate::framebuffer::{wrap, Framebuffer};
use crate::palette::*;
use crate::ui::{self, Rect};

const PANEL_X: i32 = 238;
const PANEL_W: i32 = 158;

fn tile_xy(p: Pos) -> (i32, i32) {
    (BX + p.0 * TILE, BY + p.1 * TILE)
}

fn lerp(a: (i32, i32), b: (i32, i32), k: u32, n: u32) -> (i32, i32) {
    let (k, n) = (k.min(n) as i32, n.max(1) as i32);
    (a.0 + (b.0 - a.0) * k / n, a.1 + (b.1 - a.1) * k / n)
}

/// Deterministic per-square noise for grass tufts.
fn hash(x: i32, y: i32, salt: u32) -> u32 {
    let mut h = (x as u32).wrapping_mul(0x9E37_79B1) ^ (y as u32).wrapping_mul(0x85EB_CA77) ^ salt.wrapping_mul(0xC2B2_AE3D);
    h ^= h >> 15;
    h = h.wrapping_mul(0x2C1B_3C6D);
    h ^ (h >> 12)
}

pub fn draw(g: &Game, fb: &mut Framebuffer) {
    fb.clear(BG);
    match &g.phase {
        Phase::Glade { offers, first } => draw_glade(g, fb, offers, *first),
        _ => {
            draw_play(g, fb);
            if let Phase::Over { won } = g.phase {
                draw_over(g, fb, won);
            }
        }
    }
    if g.show_help {
        draw_help(fb);
    }
}

fn sprite_pos(s: &crate::framebuffer::Sprite, px: (i32, i32)) -> (i32, i32) {
    (px.0 + (TILE - s.w) / 2, px.1 + TILE - 2 - s.h)
}

fn draw_play(g: &Game, fb: &mut Framebuffer) {
    let anim = g.anim.as_ref();
    let landed = anim.map_or(true, |a| a.t >= PH1);
    let shown: &Floor = match anim {
        Some(a) if !landed => &a.prev,
        _ => &g.floor,
    };
    let salt = g.burrow * 7 + g.lesson.map_or(0, |l| l as u32 + 100);

    // Top bar.
    let title = match g.lesson {
        Some(l) => LESSONS[l].title.to_string(),
        None => format!("{}  -  BURROW {}/{BURROWS}", g.area_name(), g.burrow),
    };
    fb.text(&title, 8, 6, TEXT, Some(OUTLINE), 1);
    let f = &g.floor;
    ui::hearts(fb, 396 - f.max_hearts * 9, 5, f.hearts, f.max_hearts);
    let dew = format!("DEW {}", f.dew_total);
    fb.text(&dew, 396 - f.max_hearts * 9 - 8 - Framebuffer::text_width(&dew, 1), 6, ACCENT_HI, Some(OUTLINE), 1);

    // Ground.
    fb.rect_fill(BX - 2, BY - 2, N * TILE + 4, N * TILE + 4, OUTLINE);
    for y in 0..N {
        for x in 0..N {
            let (tx, ty) = tile_xy((x, y));
            let base = if (x + y) % 2 == 0 { GROUND_L } else { GROUND_D };
            fb.rect_fill(tx, ty, TILE, TILE, base);
            let h = hash(x, y, salt);
            for k in 0..3 {
                let hx = (h >> (k * 8)) % (TILE as u32 - 6);
                let hy = (h >> (k * 8 + 4)) % (TILE as u32 - 4);
                if (h >> (24 + k)) & 1 == 1 {
                    fb.blit(&g.art.tuft, tx + 1 + hx as i32, ty + 1 + hy as i32, false);
                }
            }
            match shown.cell((x, y)) {
                Cell::Rock => {
                    let r = &g.art.rock;
                    fb.blit(&r, tx + (TILE - r.w) / 2, ty + TILE - 2 - r.h, h & 1 == 0);
                }
                Cell::Burrow => {
                    fb.rect_fill(tx + 4, ty + 6, TILE - 8, TILE - 10, ROOT);
                    fb.rect_fill(tx + 5, ty + 7, TILE - 10, TILE - 12, HOLE);
                    fb.rect_fill(tx + 3, ty + 8, 2, TILE - 14, ROOT + 1);
                    fb.rect_fill(tx + TILE - 5, ty + 9, 2, TILE - 16, ROOT + 1);
                    fb.rect_fill(tx + 7, ty + 5, TILE - 14, 2, ROOT + 1);
                    fb.rect_fill(tx + 9, ty + 12, 3, 2, ROOT);
                    fb.rect_fill(tx + 15, ty + 15, 4, 2, ROOT);
                    if shown.seal > 0 {
                        // A lattice of roots over the hole, and how many catches it still wants.
                        for k in 0..4 {
                            fb.line(tx + 5 + k * 5, ty + 7, tx + 5 + k * 5 + 4, ty + TILE - 6, ROOT + 1);
                            fb.line(tx + TILE - 6 - k * 5, ty + 7, tx + TILE - 10 - k * 5, ty + TILE - 6, ROOT);
                        }
                        fb.text(&shown.seal.to_string(), tx + TILE / 2 - 2, ty + TILE / 2 - 3, GOLD, Some(OUTLINE), 1);
                    }
                }
                Cell::Ground => {}
            }
        }
    }
    for &d in &shown.dew {
        let (tx, ty) = tile_xy(d);
        fb.blit(&g.art.dew, tx + 4, ty + 4, false);
    }
    // Rustling grass where something will arrive.
    for s in &shown.spawns {
        let (tx, ty) = tile_xy(s.pos);
        let sway = ((g.frame / 6) % 2) as i32;
        for k in 0..4 {
            let x = tx + 4 + k * 6 + sway;
            fb.line(x, ty + TILE - 4, x + 2 - sway * 2, ty + TILE - 12, GRASS_HI);
            fb.line(x + 1, ty + TILE - 4, x + 3 - sway * 2, ty + TILE - 10, GRASS);
        }
        let spr = g.art.of(s.kind);
        let (sx, sy) = sprite_pos(spr, (tx, ty));
        fb.blit_map(spr, sx, sy, false, &art::flat_map(GRASS));
        fb.text(&s.turns.max(1).to_string(), tx + TILE - 8, ty + 2, HILITE, Some(OUTLINE), 1);
    }

    let interactive = anim.is_none() && g.card == 0 && matches!(g.phase, Phase::Lesson | Phase::Burrow);
    let pointed = g.pointed();
    if interactive {
        for &t in &g.threatened {
            if t == f.player || g.previews.iter().any(|(p, _)| *p == t) {
                continue;
            }
            let (tx, ty) = tile_xy(t);
            fb.rect_hatch(tx + 1, ty + 1, TILE - 2, TILE - 2, BAD_DIM);
        }
        for (t, pv) in &g.previews {
            let (tx, ty) = tile_xy(*t);
            if pv.hits > 0 || pv.toxic || pv.dies || pv.blocked {
                fb.rect_hatch(tx + 1, ty + 1, TILE - 2, TILE - 2, BAD);
                fb.rect_hatch(tx + 2, ty + 1, TILE - 3, TILE - 2, BAD);
            }
            let col = if pv.catch.is_some() { GOLD } else { HILITE };
            fb.corners(tx + 1, ty + 1, TILE - 2, TILE - 2, 5, col);
            fb.corners(tx + 2, ty + 2, TILE - 4, TILE - 4, 3, OUTLINE);
            if pv.hits > 1 {
                fb.text(&pv.hits.to_string(), tx + TILE - 8, ty + TILE - 9, BAD, Some(OUTLINE), 1);
            }
        }
    }

    // Creatures.
    let mid = anim.map(|a| &a.mid);
    for c in &shown.creatures {
        let mut px = tile_xy(c.pos);
        if let (Some(a), true) = (anim, landed) {
            let k = a.t.saturating_sub(PH1) + 1;
            let from = mid.and_then(|m| m.creatures.iter().find(|m| m.id == c.id)).map(|m| m.pos);
            if let Some(from) = from {
                px = lerp(tile_xy(from), tile_xy(c.pos), k, PH2);
            }
            for e in &a.ev_enemy {
                if let Event::Strike { id, .. } | Event::Blocked { id, .. } = e {
                    if *id == c.id && k == 2 {
                        let pl = tile_xy(g.floor.player);
                        px.0 += (pl.0 - px.0).signum() * 4;
                        px.1 += (pl.1 - px.1).signum() * 4;
                    }
                }
            }
            if a.ev_enemy.iter().any(|e| matches!(e, Event::Arrived { id, .. } if *id == c.id)) && k < PH2 {
                continue;
            }
        }
        let spr = g.art.of(c.kind);
        let (sx, sy) = sprite_pos(spr, px);
        let flip = c.pos.0 > shown.player.0;
        if c.dazed {
            fb.blit_map(spr, sx, sy, flip, &art::flat_map(TEXT_FAINT));
            fb.text("*", px.0 + TILE - 9, px.1 + 1, GOLD, Some(OUTLINE), 1);
        } else {
            fb.blit(spr, sx, sy, flip);
        }
        if c.kind == Kind::Grof {
            for w in 0..c.wards {
                fb.rect_fill(px.0 + 3 + w * 5, px.1 + 1, 3, 3, GOLD);
                fb.rect_outline(px.0 + 2 + w * 5, px.1, 5, 5, OUTLINE);
            }
        }
    }

    // The changeling.
    let (from, to, form) = match anim {
        Some(a) if !landed => (tile_xy(a.prev.player), tile_xy(a.mid.player), a.prev.form),
        _ => (tile_xy(g.floor.player), tile_xy(g.floor.player), g.floor.form),
    };
    let px = match anim {
        Some(a) if !landed => lerp(from, to, a.t + 1, PH1),
        _ => to,
    };
    let spr = g.art.of(form);
    let (sx, sy) = sprite_pos(spr, px);
    let hop = if anim.is_some_and(|a| !landed && a.t == 1) { 3 } else { 0 };
    // A soft ring of light under the changeling.
    fb.rect_checker(px.0 + 5, px.1 + TILE - 5, TILE - 10, 3, SPIRIT, 0);
    if g.hurt_flash > 0 {
        fb.blit_map(spr, sx, sy - hop, false, &art::flat_map(BAD));
    } else if form == Kind::Wanderer {
        fb.blit(spr, sx, sy - hop, false);
    } else {
        fb.blit_map(spr, sx, sy - hop, false, &art::spirit_map());
    }

    for p in &g.particles {
        fb.pset(p.x, p.y, p.col);
        fb.pset(p.x + 1, p.y, p.col);
    }

    if interactive {
        if let Some(p) = pointed {
            let (tx, ty) = tile_xy(p);
            fb.rect_outline(tx, ty, TILE, TILE, WHITE);
        }
    }

    draw_panel(g, fb, pointed.filter(|_| interactive));
    draw_bar(g, fb);

    if g.card > 0 {
        fb.rect_checker(BX, BY, N * TILE, N * TILE, OUTLINE, 0);
        let r = Rect::new(BX + 20, BY + 82, N * TILE - 40, 56);
        ui::panel(fb, r);
        let (a, b) = match g.lesson {
            Some(l) => ("DOAT'S LESSONS".to_string(), LESSONS[l].title.split(": ").nth(1).unwrap_or("").to_string()),
            None => (AREAS[gen::area_of(g.burrow)].name.to_string(), format!("BURROW {} OF {BURROWS}", g.burrow)),
        };
        fb.text_centered(&a, r.x + r.w / 2, r.y + 12, ACCENT_HI, Some(OUTLINE), 1);
        fb.text_centered(&b, r.x + r.w / 2, r.y + 26, TEXT, None, 1);
        fb.text_centered(&format!("MUSIC: {}", g.track().title()), r.x + r.w / 2, r.y + 40, TEXT_FAINT, None, 1);
    }
}

fn diagram_size(kind: Kind, extra: i32) -> i32 {
    kind.diagram(extra).iter().map(|(x, y)| x.abs().max(y.abs())).max().unwrap_or(1).clamp(2, 5)
}

fn draw_diagram(fb: &mut Framebuffer, x: i32, y: i32, kind: Kind, extra: i32, max_px: i32) -> i32 {
    let r = diagram_size(kind, extra);
    let cell = (max_px / (2 * r + 1)).clamp(3, 8);
    ui::pattern(fb, x, y, r, cell, &kind.diagram(extra), art::ramp_of(kind) + 1, art::SPIRIT + 2);
    (2 * r + 1) * cell + 1
}

fn draw_panel(g: &Game, fb: &mut Framebuffer, pointed: Option<Pos>) {
    let f = &g.floor;
    ui::panel(fb, Rect::new(PANEL_X, BY - 2, PANEL_W, N * TILE + 4));
    let x0 = PANEL_X + 6;
    fb.text("YOUR SHAPE", x0, BY + 3, TEXT_DIM, None, 1);
    let name = if f.form == Kind::Wanderer { "YOURSELF".to_string() } else { format!("{} SHAPE", f.form.name()) };
    fb.text(&name, x0, BY + 13, ACCENT_HI, Some(OUTLINE), 1);
    let extra = if f.has(Charm::LongShadow) { 2 } else { 0 };
    let w = draw_diagram(fb, x0, BY + 24, f.form, extra, 50);
    let tx = x0 + w + 5;
    let cols = ((PANEL_X + PANEL_W - 4 - tx) / 6).max(6) as usize;
    let mut y = BY + 25;
    for l in wrap(f.form.shape_text(), cols) {
        fb.text(&l, tx, y, TEXT, None, 1);
        y += 9;
    }
    if let Some(p) = f.prev_form.filter(|p| f.has(Charm::TwoSkins) && *p != f.form) {
        y += 2;
        for l in wrap(&format!("ALSO {}", p.name()), cols) {
            fb.text(&l, tx, y, TEXT_DIM, None, 1);
            y += 9;
        }
    }
    if f.pouncing {
        fb.text("POUNCE!", tx, BY + 66, GOLD, Some(OUTLINE), 1);
    }

    let sep = BY + 80;
    fb.rect_fill(PANEL_X + 4, sep, PANEL_W - 8, 1, PANEL_LO);
    fb.rect_fill(PANEL_X + 4, sep + 1, PANEL_W - 8, 1, PANEL_HI);
    let mut y = sep + 6;
    let cols = ((PANEL_W - 12) / 6) as usize;
    let mut say = |fb: &mut Framebuffer, y: &mut i32, s: &str, col: u8| {
        for l in wrap(s, cols) {
            if *y < BY + 178 {
                fb.text(&l, x0, *y, col, None, 1);
            }
            *y += 9;
        }
    };
    if let Some(p) = pointed {
        let preview = g.previews.iter().find(|(t, _)| *t == p).map(|(_, pv)| *pv);
        let creature = f.creature_at(p).map(|i| &f.creatures[i]);
        if p == f.player {
            fb.text("YOU", x0, y, ACCENT_HI, Some(OUTLINE), 1);
            y += 11;
            if f.pouncing {
                say(fb, &mut y, "CLICK YOURSELF TO STOP POUNCING. THE CREATURES THEN MOVE.", TEXT);
            } else {
                say(fb, &mut y, "CLICK YOURSELF TO REST: BECOME YOURSELF AGAIN. THE CREATURES GET A TURN.", TEXT);
            }
            outcome(fb, &mut y, &mut say, &g.rest_preview);
        } else if let Some(pv) = preview {
            let head = match (pv.catch, pv.descend) {
                (Some(k), _) => format!("CATCH THE {}", k.name()),
                (None, true) => "GO DOWN THE BURROW".to_string(),
                _ => "MOVE HERE".to_string(),
            };
            fb.text(&head, x0, y, GOLD, Some(OUTLINE), 1);
            y += 11;
            if let Some(k) = pv.catch {
                if k == Kind::Grof && !pv.wins {
                    say(fb, &mut y, "HE LOSES A WARD AND FLEES. YOU TAKE HIS SHAPE.", TEXT);
                } else if !pv.wins {
                    say(fb, &mut y, &format!("YOU WILL MOVE LIKE A {}: {}.", k.name(), k.shape_text()), TEXT);
                }
            }
            if pv.dew > 0 {
                say(fb, &mut y, &format!("+{} DEW", pv.dew), ACCENT_HI);
            }
            if pv.opens {
                say(fb, &mut y, "THE BURROW OPENS.", ACCENT_HI);
            }
            if pv.descend {
                say(fb, &mut y, "ON TO THE GLADE, THEN DEEPER.", TEXT);
            }
            outcome(fb, &mut y, &mut say, &pv);
        } else if let Some(c) = creature {
            fb.text(c.kind.name(), x0, y, ACCENT_HI, Some(OUTLINE), 1);
            y += 11;
            let w = draw_diagram(fb, x0, y, c.kind, 0, 44);
            let tx = x0 + w + 5;
            let tcols = ((PANEL_X + PANEL_W - 4 - tx) / 6) as usize;
            let mut ty = y;
            for l in wrap(c.kind.shape_text(), tcols) {
                fb.text(&l, tx, ty, TEXT, None, 1);
                ty += 9;
            }
            if c.kind.dew() > 0 {
                fb.text(&format!("{} DEW", c.kind.dew()), tx, ty + 2, ACCENT_HI, None, 1);
            }
            y += w + 4;
            if let Some(t) = c.kind.trait_text() {
                say(fb, &mut y, t, GOLD);
            }
            if c.dazed {
                say(fb, &mut y, "DAZED: SKIPS ITS NEXT TURN.", TEXT_DIM);
            }
        } else if let Some(s) = f.spawns.iter().find(|s| s.pos == p) {
            fb.text("RUSTLING", x0, y, ACCENT_HI, Some(OUTLINE), 1);
            y += 11;
            say(fb, &mut y, &format!("A {} ARRIVES HERE IN {} TURN{}.", s.kind.name(), s.turns.max(1), if s.turns > 1 { "S" } else { "" }), TEXT);
            say(fb, &mut y, s.kind.shape_text(), TEXT_DIM);
        } else {
            let sealed = format!("SEALED BY ROOTS. CATCH {} MORE CREATURE{} HERE TO OPEN IT.", f.seal, if f.seal == 1 { "" } else { "S" });
            let (head, text) = match f.cell(p) {
                Cell::Rock => ("ROCK", "BLOCKS LINES. LEAPS GO OVER IT."),
                Cell::Burrow if f.seal > 0 => ("THE BURROW", sealed.as_str()),
                Cell::Burrow => ("THE BURROW", "THE WAY DEEPER. IT'S OPEN."),
                Cell::Ground if f.dew.contains(&p) => ("DEW", "+1 DEW WHEN YOU STEP HERE."),
                Cell::Ground => ("GROUND", "YOUR SHAPE CAN'T REACH THIS SQUARE NOW."),
            };
            fb.text(head, x0, y, ACCENT_HI, Some(OUTLINE), 1);
            y += 11;
            say(fb, &mut y, text, TEXT);
            if g.threatened.contains(&p) {
                say(fb, &mut y, "A CREATURE COULD STRIKE HERE.", BAD);
            }
        }
    } else {
        fb.text("POINT AT ANYTHING", x0, y, TEXT_DIM, None, 1);
    }

    // Charms.
    let cy = BY + 184;
    fb.rect_fill(PANEL_X + 4, cy - 4, PANEL_W - 8, 1, PANEL_LO);
    fb.rect_fill(PANEL_X + 4, cy - 3, PANEL_W - 8, 1, PANEL_HI);
    if f.charms.is_empty() {
        fb.text("NO CHARMS YET", x0, cy, TEXT_FAINT, None, 1);
    } else {
        for (i, c) in f.charms.iter().enumerate() {
            let (cx, yy) = (x0 + (i as i32 % 2) * 74, cy + (i as i32 / 2) * 9);
            let col = if *c == Charm::Shell && !f.shell_ready { TEXT_FAINT } else { TEXT_DIM };
            fb.text(c.name(), cx, yy, col, None, 1);
        }
    }
}

fn outcome(fb: &mut Framebuffer, y: &mut i32, say: &mut impl FnMut(&mut Framebuffer, &mut i32, &str, u8), pv: &Preview) {
    if pv.wins {
        say(fb, y, "THE LAST CATCH. GROF IS YOURS.", GOOD);
        return;
    }
    if pv.toxic {
        say(fb, y, "TOXIC: CATCHING IT COSTS A HEART.", BAD);
    }
    if pv.pounce {
        say(fb, y, "POUNCE: YOU MAY CATCH AGAIN FIRST.", GOLD);
    }
    if pv.blocked {
        say(fb, y, "A STRIKE COMES, BUT YOUR SHELL BLOCKS IT (ONCE PER BURROW).", GOLD);
    }
    if pv.dies {
        say(fb, y, "YOU WOULD NOT SURVIVE THIS.", BAD);
    } else if pv.descend {
    } else if pv.hits == 0 && !pv.blocked {
        say(fb, y, "SAFE: NOTHING CAN STRIKE YOU THERE.", GOOD);
    } else if pv.hits > 0 {
        let s = format!("STRUCK {} TIME{}: YOU LOSE {} HEART{}.", pv.hits, if pv.hits > 1 { "S" } else { "" }, pv.hits, if pv.hits > 1 { "S" } else { "" });
        say(fb, y, &s, BAD);
    }
}

fn draw_bar(g: &Game, fb: &mut Framebuffer) {
    let r = Rect::new(2, 248, 396, 50);
    ui::inset(fb, r);
    let mut y = 252;
    if let Some(l) = g.lesson {
        if let Some((text, _)) = LESSONS[l].steps.get(g.lesson_step) {
            fb.text(&format!("DOAT: (STEP {} OF {})", g.lesson_step + 1, LESSONS[l].steps.len()), 8, y, ACCENT, None, 1);
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
    } else if g.floor.pouncing {
        fb.text("POUNCE: CATCH AGAIN, OR CLICK YOURSELF TO STOP.", 8, y, GOLD, None, 1);
    }
    let hint = "CLICK A LIT SQUARE: MOVE.  CLICK YOURSELF: REST.  F1: RULES";
    fb.text(hint, 8, 287, TEXT_FAINT, None, 1);
}

fn draw_glade(g: &Game, fb: &mut Framebuffer, offers: &[Charm], first: bool) {
    for y in (0..300).step_by(2) {
        fb.rect_checker(0, y, 400, 1, BG + 1, y);
    }
    fb.text_centered("THE GLADE", 200, 8, ACCENT_HI, Some(OUTLINE), 2);
    let sub = if g.burrow == 0 { "BEFORE THE FIRST BURROW".to_string() } else { format!("AFTER BURROW {} OF {BURROWS}", g.burrow) };
    fb.text_centered(&sub, 200, 28, TEXT_DIM, None, 1);
    let c = &g.carry;
    ui::hearts(fb, 20, 44, c.hearts, c.max_hearts);
    fb.text(&format!("DEW {}", c.dew), 24 + c.max_hearts * 9, 45, ACCENT_HI, Some(OUTLINE), 1);
    let intro = if first {
        "SPEND DEW ON CHARMS, OR MEND A HEART. CHARMS LAST THE WHOLE RUN. YOU KEEP YOUR SHAPE GOING DOWN."
    } else {
        "SPEND DEW, OR SAVE IT. YOU KEEP YOUR SHAPE GOING DOWN."
    };
    for (i, l) in wrap(intro, 62).iter().enumerate() {
        fb.text(l, 20, 60 + i as i32 * 9, TEXT, None, 1);
    }
    let items = g.glade_items();
    for (i, item) in items.iter().enumerate() {
        let r = glade_item_rect(i);
        let can = g.glade_can_buy(*item);
        let hover = g.glade_hover == Some(i);
        if hover && can {
            fb.rect_bevel(r.x, r.y, r.w, r.h, PANEL_HI, WHITE, PANEL);
        } else {
            ui::panel(fb, r);
        }
        let (name, text) = match item {
            Some(ch) => (ch.name(), ch.text()),
            None => ("MEND", "MEND ONE HEART."),
        };
        let cost = g.glade_cost(*item);
        let owned = item.is_some_and(|ch| c.charms.contains(&ch));
        let tcol = if hover && can { PANEL_DEEP } else if can { TEXT } else { TEXT_FAINT };
        fb.text(name, r.x + 8, r.y + 5, if hover && can { PANEL_DEEP } else if can { ACCENT_HI } else { TEXT_DIM }, None, 1);
        let tag = if owned {
            "OWNED".to_string()
        } else if item.is_none() && c.hearts >= c.max_hearts {
            "HEARTS FULL".to_string()
        } else {
            format!("{cost} DEW")
        };
        fb.text(&tag, r.x + r.w - 8 - Framebuffer::text_width(&tag, 1), r.y + 5, if can { tcol } else { TEXT_FAINT }, None, 1);
        for (k, l) in wrap(text, 58).iter().take(2).enumerate() {
            fb.text(l, r.x + 8, r.y + 15 + k as i32 * 9, tcol, None, 1);
        }
    }
    let _ = offers;
    if !c.charms.is_empty() {
        let s = format!("YOURS: {}", c.charms.iter().map(|c| c.name()).collect::<Vec<_>>().join(", "));
        for (i, l) in wrap(&s, 62).iter().take(2).enumerate() {
            fb.text(l, 20, 246 + i as i32 * 9, TEXT_DIM, None, 1);
        }
    }
    ui::button(fb, GLADE_GO, "GO DEEPER", g.glade_hover == Some(items.len()), true);
    fb.text("CLICK A CHARM TO BUY IT.", 20, 274, TEXT_FAINT, None, 1);
}

fn draw_over(g: &Game, fb: &mut Framebuffer, won: bool) {
    fb.rect_checker(0, 0, 400, 300, OUTLINE, 0);
    let r = Rect::new(60, 70, 280, 210);
    ui::panel(fb, r);
    let (head, sub) = if won {
        ("GROF IS CAUGHT", "THE FOREST OF NAIBIHPMA IS QUIET.".to_string())
    } else {
        ("YOU ARE CAUGHT OUT", format!("IN {}, BURROW {} OF {BURROWS}.", g.area_name(), g.burrow))
    };
    fb.text_centered(head, 200, 80, if won { GOLD } else { BAD }, Some(OUTLINE), 2);
    fb.text_centered(&sub, 200, 102, TEXT, None, 1);
    let s = &g.stats;
    let lines = [
        format!("CATCHES: {}", s.catches),
        format!("STRUCK: {} TIMES", s.hits),
        format!("TURNS: {}   RESTS: {}", s.turns, s.rests),
        format!("DEW LEFT: {}", g.carry.dew),
    ];
    for (i, l) in lines.iter().enumerate() {
        fb.text_centered(l, 200, 122 + i as i32 * 11, TEXT_DIM, None, 1);
    }
    if !g.carry.charms.is_empty() {
        let c = format!("CHARMS: {}", g.carry.charms.iter().map(|c| c.name()).collect::<Vec<_>>().join(", "));
        for (i, l) in wrap(&c, 42).iter().take(3).enumerate() {
            fb.text_centered(l, 200, 172 + i as i32 * 10, TEXT_DIM, None, 1);
        }
    }
    let spr = g.art.of(if won { Kind::Grof } else { g.floor.form });
    fb.blit_map(spr, 200 - spr.w / 2, 240 - spr.h, false, &if won { crate::framebuffer::IDENTITY } else { art::spirit_map() });
    let hover = OVER_GO.contains(g.mouse.0, g.mouse.1);
    ui::button(fb, OVER_GO, "CONTINUE", hover, true);
}

fn draw_help(fb: &mut Framebuffer) {
    fb.rect_checker(0, 0, 400, 300, OUTLINE, 0);
    let r = Rect::new(12, 10, 376, 280);
    ui::panel(fb, r);
    fb.text_centered("HOW THE FOREST WORKS", 200, 18, ACCENT_HI, Some(OUTLINE), 1);
    let rules = [
        "YOU MOVE TO A LIT SQUARE. YOUR SHAPE DECIDES WHICH SQUARES ARE LIT.",
        "MOVE ONTO A CREATURE TO CATCH IT: YOU GET ITS DEW AND TAKE ITS SHAPE.",
        "CLICK YOURSELF TO REST: YOU BECOME YOURSELF AGAIN (ONE STEP, ANY WAY).",
        "AFTER EVERY MOVE OR REST, EACH CREATURE THAT CAN REACH YOU STRIKES. THE OTHERS MOVE CLOSER.",
        "A STRIKE COSTS A HEART. RED SQUARES ARE WHERE YOU WOULD BE STRUCK.",
        "POINT AT A LIT SQUARE TO SEE EXACTLY WHAT WOULD HAPPEN THERE.",
        "LINES STOP AT ROCKS AND AT ANYTHING IN THEM. LEAPS GO OVER EVERYTHING.",
        "RUSTLING GRASS: A NEW CREATURE ARRIVES THERE SOON.",
        "ROOTS SEAL EACH BURROW UNTIL YOU'VE CAUGHT A FEW CREATURES IN IT. BETWEEN BURROWS, SPEND DEW IN THE GLADE.",
    ];
    let mut y = 34;
    for rule in rules {
        for (i, l) in wrap(rule, 58).iter().enumerate() {
            fb.text(if i == 0 { "-" } else { " " }, 22, y, ACCENT, None, 1);
            fb.text(l, 32, y, TEXT, None, 1);
            y += 9;
        }
        y += 3;
    }
    y += 4;
    let keys = [
        "MOUSE: POINT AND CLICK.  ARROWS + ENTER: THE SAME.",
        "R: REST.  F1: THIS PAGE.  F2: SCANLINES.  F3: MUSIC LEVEL.",
        "ESC: PAUSE (END THE RUN OR QUIT FROM THERE).",
    ];
    for k in keys {
        fb.text(k, 22, y, TEXT_DIM, None, 1);
        y += 10;
    }
    fb.text_centered("PRESS ANY KEY OR CLICK TO GO BACK", 200, 276, ACCENT, None, 1);
}
