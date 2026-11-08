//! Drawing Turncoat.

use super::art::{self, *};
use super::battle::*;
use super::data::*;
use super::*;
use crate::framebuffer::{wrap, Framebuffer};
use crate::palette::*;
use crate::ui::{self, Rect};

const PX: i32 = 210;
const PW: i32 = 186;

fn tile_xy(p: Pos) -> (i32, i32) {
    (FX + p.0 * TILE, FY + p.1 * TILE)
}

pub fn draw(g: &Game, fb: &mut Framebuffer) {
    fb.clear(BG);
    match &g.phase {
        Phase::Route { options } => draw_route(g, fb, options.len()),
        Phase::Keep { cands, keep } => draw_keep(g, fb, cands, keep),
        Phase::Idea { offers } => draw_ideas(g, fb, offers),
        Phase::Over { won } => draw_over(g, fb, *won),
        _ => draw_field_screen(g, fb),
    }
    if g.show_help {
        draw_help(fb);
    }
}

fn top_bar(g: &Game, fb: &mut Framebuffer) {
    let title = if g.lesson {
        "THE LESSON: A FIRST CONTEST".to_string()
    } else {
        format!("{}  -  BATTLE {} OF {BATTLES}{}", AGES[g.age()], g.index + 1, if g.is_leader_battle() { "  (LEADER)" } else { "" })
    };
    fb.text(&title, 8, 6, TEXT, Some(OUTLINE), 1);
    // Chances as little banners.
    for i in 0..3 {
        let x = 396 - 12 - i * 10;
        let col = if (i as i32) < g.lives { BLUE + 1 } else { BG + 2 };
        fb.rect_fill(x, 3, 1, 11, HAIR + 2);
        fb.rect_fill(x + 1, 3, 6, 5, col);
        fb.rect_outline(x + 1, 3, 6, 5, OUTLINE);
    }
    let band = format!("BAND {}/{}", g.band.len(), g.band_limit());
    fb.text(&band, 396 - 36 - Framebuffer::text_width(&band, 1) - 4, 6, ACCENT_HI, Some(OUTLINE), 1);
}

fn draw_person(g: &Game, fb: &mut Framebuffer, kind: Kind, blue: bool, px: (i32, i32), resolve: i32, max: i32, veteran: bool) {
    let spr = g.art.of(kind);
    let (sx, sy) = (px.0 + (TILE - spr.w) / 2, px.1 + TILE - 4 - spr.h);
    fb.rect_checker(px.0 + 6, px.1 + TILE - 5, TILE - 12, 2, OUTLINE, 0);
    fb.blit_map(spr, sx, sy, !blue, &art::team_map(blue));
    // Resolve bar: one pip per point, in the side's colour.
    let max = max.max(1);
    let w = TILE - 4;
    fb.rect_fill(px.0 + 2, px.1 + TILE - 3, w, 3, OUTLINE);
    let fill = (resolve.max(0) * (w - 2) + max - 1) / max;
    fb.rect_fill(px.0 + 3, px.1 + TILE - 2, fill.min(w - 2), 1, if blue { BLUE + 2 } else { RED + 2 });
    if veteran {
        fb.text("*", px.0 + TILE - 8, px.1, GOLD, Some(OUTLINE), 1);
    }
}

fn reach_marks(fb: &mut Framebuffer, squares: &[Pos], col: u8) {
    for &q in squares {
        let (tx, ty) = tile_xy(q);
        fb.corners(tx + 2, ty + 2, TILE - 4, TILE - 4, 4, col);
        fb.rect_fill(tx + TILE / 2 - 1, ty + TILE / 2 - 1, 3, 3, col);
        fb.rect_outline(tx + TILE / 2 - 2, ty + TILE / 2 - 2, 5, 5, OUTLINE);
    }
}

fn draw_field_screen(g: &Game, fb: &mut Framebuffer) {
    top_bar(g, fb);
    fb.rect_fill(FX - 2, FY - 2, W * TILE + 4, H * TILE + 4, OUTLINE);
    for y in 0..H {
        for x in 0..W {
            let (tx, ty) = tile_xy((x, y));
            fb.rect_fill(tx, ty, TILE, TILE, if (x + y) % 2 == 0 { GROUND_L } else { GROUND_D });
            let h = (x * 7 + y * 13 + g.age() as i32 * 5) % 9;
            if h < 3 {
                fb.pset(tx + 5 + h * 6, ty + 6 + h * 4, GROUND_X);
                fb.pset(tx + 6 + h * 6, ty + 6 + h * 4, GROUND_X);
                fb.pset(tx + 18 - h * 3, ty + 19 - h, GROUND_X + 1);
            }
        }
    }
    // The line between the sides.
    for x in (FX..FX + W * TILE).step_by(4) {
        fb.rect_fill(x, FY + 3 * TILE - 1, 2, 2, MIDLINE);
    }

    let deploy = g.phase == Phase::Deploy;
    if deploy {
        let dragging = g.drag.as_ref().map(|d| d.member);
        for y in 0..H {
            for x in 0..W {
                let member = dragging.unwrap_or_else(|| g.bench().first().copied().unwrap_or(0));
                let ok = if g.band.is_empty() { false } else { g.can_stand(member, (x, y)) };
                let open_row = y >= 3 || dragging.is_some_and(|m| g.band[m].kind.has(Trait::DropIn));
                if ok && open_row {
                    let (tx, ty) = tile_xy((x, y));
                    fb.rect_checker(tx + 1, ty + 1, TILE - 2, TILE - 2, LIT, 0);
                }
            }
        }
    }

    // Reach of whoever is pointed at.
    let hovered = g.hovered();
    if let Some((kind, side, Some(p), idx)) = hovered {
        let squares = match (&g.battle, idx) {
            (Some(b), Some(i)) if matches!(g.phase, Phase::Battle | Phase::After { .. }) => b.reach_from(i, p),
            _ => {
                let b = g.build_battle();
                match b.at(p) {
                    Some(i) => b.reach_from(i, p),
                    None => Vec::new(),
                }
            }
        };
        let _ = kind;
        reach_marks(fb, &squares, if side == Side::Blue { BLUE + 2 } else { RED + 2 });
    }

    // People.
    match (&g.phase, &g.battle) {
        (Phase::Battle | Phase::After { .. }, Some(b)) => {
            for (i, p) in b.people.iter().enumerate() {
                let s = &g.shown[i];
                let mut px = tile_xy(s.pos);
                if let Some(pl) = &g.playing {
                    if let Act::Walk { who, from, to } = pl.act {
                        if who == i {
                            let (a, c) = (tile_xy(from), tile_xy(to));
                            let k = (pl.t + 1).min(pl.len) as i32;
                            let n = pl.len.max(1) as i32;
                            px = (a.0 + (c.0 - a.0) * k / n, a.1 + (c.1 - a.1) * k / n);
                        }
                    }
                    if let Act::Speak { actor, .. } = pl.act {
                        if actor == i && pl.t < 2 {
                            px.1 -= 2;
                        }
                    }
                }
                draw_person(g, fb, p.kind, s.side == Side::Blue, px, s.resolve, p.max, p.veteran);
            }
            if let Some(pl) = &g.playing {
                draw_act(g, fb, b, pl);
            }
        }
        _ => {
            if let Some(e) = &g.encounter {
                for (k, p, vet) in &e.rivals {
                    let max = max_resolve(*k, *vet, Side::Red, &[]);
                    draw_person(g, fb, *k, false, tile_xy(*p), max, max, *vet);
                }
            }
            for &(m, p) in &g.placed {
                if g.drag.as_ref().is_some_and(|d| d.member == m && d.from.is_some()) {
                    continue;
                }
                let mb = &g.band[m];
                let max = max_resolve(mb.kind, mb.veteran, Side::Blue, &g.ideas);
                draw_person(g, fb, mb.kind, true, tile_xy(p), max, max, mb.veteran);
            }
        }
    }
    for p in &g.particles {
        fb.pset(p.0, p.1, p.5);
        fb.pset(p.0 + 1, p.1, p.5);
    }

    // Bench.
    let label = if deploy {
        format!("YOUR BAND - ON THE FIELD {}/{}", g.placed.len(), g.deploy_limit().min(g.band.len()))
    } else {
        "WAITING ON THE BENCH".to_string()
    };
    fb.text(&label, 8, BENCH_Y - 10, TEXT_DIM, None, 1);
    let bench = g.bench();
    for slot in 0..g.band_limit() {
        let r = Game::bench_rect(slot);
        ui::inset(fb, r);
        if let Some(&m) = bench.get(slot) {
            if g.drag.as_ref().is_some_and(|d| d.member == m) {
                continue;
            }
            let mb = &g.band[m];
            let max = max_resolve(mb.kind, mb.veteran, Side::Blue, &g.ideas);
            draw_person(g, fb, mb.kind, true, (r.x, r.y), max, max, mb.veteran);
        }
    }
    // The person being dragged follows the pointer.
    if let Some(d) = &g.drag {
        let mb = &g.band[d.member];
        let spr = g.art.of(mb.kind);
        let (mx, my) = g.mouse;
        if let Some(p) = g.field_square(mx, my) {
            let (tx, ty) = tile_xy(p);
            let col = if g.can_stand(d.member, p) { WHITE } else { BAD };
            fb.rect_outline(tx, ty, TILE, TILE, col);
        }
        fb.blit_map(spr, mx - spr.w / 2, my - spr.h + 4, false, &art::team_map(true));
    }

    match &g.phase {
        Phase::Deploy => {
            let can = !(g.lesson && g.lesson_step < 2) && !g.placed.is_empty();
            ui::button(fb, START_BTN, "START", START_BTN.contains(g.mouse.0, g.mouse.1) && can, can);
        }
        Phase::Battle | Phase::After { .. } => {
            let sp = format!("{}X", g.speed);
            ui::button(fb, SPEED_BTN, &sp, SPEED_BTN.contains(g.mouse.0, g.mouse.1), true);
            ui::button(fb, SKIP_BTN, "SKIP", SKIP_BTN.contains(g.mouse.0, g.mouse.1), !g.lesson);
        }
        _ => {}
    }

    draw_panel(g, fb, hovered);
    draw_bar(g, fb);

    if let Phase::After { outcome, .. } = &g.phase {
        let r = Rect::new(FX + 10, FY + 60, W * TILE - 20, 50);
        ui::panel(fb, r);
        let (a, col) = match outcome {
            Outcome::Won => ("EVERYONE CARRIES YOUR BANNER", GOOD),
            Outcome::Lost => ("YOUR BAND IS WON OVER", BAD),
            Outcome::Draw => ("A STALEMATE", GOLD),
        };
        fb.text_centered(a, r.x + r.w / 2, r.y + 12, col, Some(OUTLINE), 1);
        fb.text_centered("CLICK TO CONTINUE", r.x + r.w / 2, r.y + 30, TEXT_DIM, None, 1);
    }
}

fn center(p: Pos) -> (i32, i32) {
    let (x, y) = tile_xy(p);
    (x + TILE / 2, y + TILE / 2 - 4)
}

fn draw_act(g: &Game, fb: &mut Framebuffer, _b: &Battle, pl: &Playing) {
    match &pl.act {
        Act::Speak { actor, hits } => {
            let from = center(g.shown[*actor].pos);
            let col = if g.shown[*actor].side == Side::Blue { BLUE + 2 } else { RED + 2 };
            for &(t, d) in hits {
                let to = center(g.shown[t].pos);
                // A dotted voice line that grows toward the listener.
                let k = (pl.t + 1).min(3) as i32;
                let end = (from.0 + (to.0 - from.0) * k / 3, from.1 + (to.1 - from.1) * k / 3);
                let steps = ((end.0 - from.0).abs().max((end.1 - from.1).abs()) / 3).max(1);
                for s in 0..=steps {
                    let x = from.0 + (end.0 - from.0) * s / steps;
                    let y = from.1 + (end.1 - from.1) * s / steps;
                    fb.rect_fill(x - 1, y - 1, 3, 3, OUTLINE);
                    fb.pset(x, y, col);
                }
                if pl.t >= 2 {
                    let (tx, ty) = tile_xy(g.shown[t].pos);
                    fb.text(&format!("-{d}"), tx + 8, ty - 2 - pl.t.min(4) as i32, col, Some(OUTLINE), 1);
                }
            }
            // A speech mark over the speaker.
            let (sx, sy) = tile_xy(g.shown[*actor].pos);
            fb.rect_fill(sx + TILE - 10, sy - 4, 9, 7, OUTLINE);
            fb.rect_fill(sx + TILE - 9, sy - 3, 7, 5, WHITE);
            fb.pset(sx + TILE - 7, sy - 1, OUTLINE);
            fb.pset(sx + TILE - 5, sy - 1, OUTLINE);
        }
        Act::Turned { who, .. } => {
            let (tx, ty) = tile_xy(g.shown[*who].pos);
            fb.corners(tx, ty, TILE, TILE, 6, GOLD);
            if pl.t < 4 {
                fb.text("TURNED!", tx - 6, ty - 8, GOLD, Some(OUTLINE), 1);
            }
        }
        Act::Held { who } => {
            let (tx, ty) = tile_xy(g.shown[*who].pos);
            fb.text("HELD", tx + 2, ty - 6, CLOTH + 2, Some(OUTLINE), 1);
        }
        Act::Veteran { who } => {
            let (tx, ty) = tile_xy(g.shown[*who].pos);
            fb.text("VETERAN", tx - 6, ty - 8, GOLD, Some(OUTLINE), 1);
        }
        _ => {}
    }
}

fn draw_panel(g: &Game, fb: &mut Framebuffer, hovered: Option<(Kind, Side, Option<Pos>, Option<usize>)>) {
    let r = Rect::new(PX, FY - 2, PW, H * TILE + 4);
    ui::panel(fb, r);
    let x0 = PX + 6;
    let cols = ((PW - 12) / 6) as usize;
    let mut y = FY + 3;
    if let Some((kind, side, pos, idx)) = hovered {
        let info = kind.info();
        let who = if side == Side::Blue { "YOURS" } else { "THEIRS" };
        fb.text(info.name, x0, y, if side == Side::Blue { BLUE + 2 } else { RED + 2 }, Some(OUTLINE), 1);
        fb.text(who, PX + PW - 6 - Framebuffer::text_width(who, 1), y, TEXT_DIM, None, 1);
        y += 12;
        let (resolve, max, voice, pace, vet) = match (&g.battle, idx) {
            (Some(b), Some(i)) => (g.shown[i].resolve, b.people[i].max, b.voice(i), b.pace(i), b.people[i].veteran),
            _ => {
                let b = g.build_battle();
                match pos.and_then(|p| b.at(p)) {
                    Some(i) => (b.people[i].resolve, b.people[i].max, b.voice(i), b.pace(i), b.people[i].veteran),
                    None => {
                        let m = max_resolve(kind, false, side, if side == Side::Blue { &g.ideas } else { &[] });
                        (m, m, info.voice, info.pace, false)
                    }
                }
            }
        };
        fb.text(&format!("RESOLVE {resolve}/{max}"), x0, y, TEXT, None, 1);
        y += 10;
        fb.text(&format!("VOICE {voice}   PACE {pace}"), x0, y, TEXT, None, 1);
        y += 12;
        // Reach diagram, facing the other side.
        let marks: Vec<(i32, i32)> = {
            let probe = Person {
                uid: 0,
                kind,
                side,
                origin: side,
                pos: (3, if side == Side::Blue { 3 } else { 2 }),
                resolve: 1,
                max: 1,
                veteran: false,
                career: 0,
                turned_here: 0,
            };
            let b = Battle::new(vec![probe], if side == Side::Blue { g.ideas.clone() } else { vec![] });
            b.reach_from(0, b.people[0].pos).iter().map(|q| (q.0 - 3, q.1 - b.people[0].pos.1)).filter(|(dx, dy)| dx.abs() <= 3 && dy.abs() <= 3).collect()
        };
        ui::pattern(fb, x0, y, 3, 7, &marks, if side == Side::Blue { BLUE + 1 } else { RED + 1 }, GOLD_R + 2);
        let tx = x0 + 7 * 7 + 6;
        let tcols = ((PX + PW - 4 - tx) / 6) as usize;
        let mut ty = y;
        for l in wrap(kind.reach_text(), tcols) {
            fb.text(&l, tx, ty, TEXT, None, 1);
            ty += 9;
        }
        fb.text(if side == Side::Blue { "(AHEAD = UP)" } else { "(AHEAD = DOWN)" }, tx, ty + 2, TEXT_FAINT, None, 1);
        y += 7 * 7 + 6;
        for t in info.traits {
            for l in wrap(t.text(), cols) {
                fb.text(&l, x0, y, GOLD, None, 1);
                y += 9;
            }
        }
        if vet {
            for l in wrap("VETERAN: SPEAKS 1 LOUDER, 2 MORE RESOLVE.", cols) {
                fb.text(&l, x0, y, GOLD, None, 1);
                y += 9;
            }
        }
        if info.leader {
            for l in wrap("A LEADER. WIN THEM OVER AND THEY CAN JOIN YOU.", cols) {
                fb.text(&l, x0, y, TEXT_DIM, None, 1);
                y += 9;
            }
        }
        return;
    }
    match (&g.phase, &g.battle) {
        (Phase::Battle | Phase::After { .. }, Some(b)) => {
            let blue = g.shown.iter().filter(|s| s.side == Side::Blue).count();
            let red = g.shown.len() - blue;
            fb.text(&format!("ROUND {} OF {}", b.round.clamp(1, ROUND_CAP), ROUND_CAP), x0, y, TEXT, None, 1);
            y += 14;
            fb.text(&format!("YOURS  {blue}"), x0, y, BLUE + 2, Some(OUTLINE), 1);
            y += 10;
            fb.text(&format!("THEIRS {red}"), x0, y, RED + 2, Some(OUTLINE), 1);
            y += 16;
            for l in wrap("POINT AT ANYONE TO SEE THEIR REACH AND RESOLVE.", cols) {
                fb.text(&l, x0, y, TEXT_DIM, None, 1);
                y += 9;
            }
        }
        _ => {
            fb.text("THEY BRING", x0, y, TEXT_DIM, None, 1);
            y += 11;
            if let Some(e) = &g.encounter {
                let mut names: Vec<(Kind, usize)> = Vec::new();
                for (k, _, _) in &e.rivals {
                    match names.iter_mut().find(|n| n.0 == *k) {
                        Some(n) => n.1 += 1,
                        None => names.push((*k, 1)),
                    }
                }
                for (k, n) in names {
                    let s = if n > 1 { format!("{n} {}S", k.info().name) } else { k.info().name.to_string() };
                    fb.text(&s, x0, y, RED + 2, None, 1);
                    y += 9;
                }
            }
            y += 6;
            for l in wrap("POINT AT ANYONE TO SEE WHERE THEIR VOICE REACHES.", cols) {
                fb.text(&l, x0, y, TEXT_DIM, None, 1);
                y += 9;
            }
            if !g.ideas.is_empty() {
                y += 4;
                fb.text("YOUR IDEAS", x0, y, TEXT_DIM, None, 1);
                y += 10;
                for i in &g.ideas {
                    fb.text(i.name(), x0, y, GOLD, None, 1);
                    y += 9;
                }
            }
        }
    }
}

fn draw_bar(g: &Game, fb: &mut Framebuffer) {
    draw_bar_hint(g, fb, None);
}

fn draw_bar_hint(g: &Game, fb: &mut Framebuffer, hint_override: Option<&str>) {
    let r = Rect::new(2, 238, 396, 60);
    ui::inset(fb, r);
    let mut y = 242;
    let say = |fb: &mut Framebuffer, s: &str, col: u8, y: &mut i32| {
        for l in wrap(s, 64).iter().take(4) {
            fb.text(l, 8, *y, col, None, 1);
            *y += 9;
        }
    };
    if let Some(n) = &g.note {
        say(fb, n, TEXT, &mut y);
        fb.text("CLICK TO GO ON", 8, 287, ACCENT, None, 1);
        return;
    }
    if g.lesson && g.lesson_step < LESSON_STEPS.len() {
        fb.text(&format!("LESSON (STEP {} OF {})", g.lesson_step + 1, LESSON_STEPS.len()), 8, y, ACCENT, None, 1);
        y += 10;
        say(fb, LESSON_STEPS[g.lesson_step], TEXT, &mut y);
    } else if let Some(m) = &g.message {
        say(fb, m, TEXT, &mut y);
    }
    let hint = hint_override.unwrap_or(match g.phase {
        Phase::Deploy => "DRAG ONTO LIT SQUARES. RIGHT-CLICK SENDS BACK. F1: RULES",
        Phase::Battle => "F OR THE 1X BUTTON: SPEED.  F1: RULES",
        Phase::Keep { .. } => "CLICK PEOPLE TO KEEP OR LET GO.  F1: RULES",
        _ => "F1: RULES",
    });
    fb.text(hint, 8, 287, TEXT_FAINT, None, 1);
}

fn draw_route(g: &Game, fb: &mut Framebuffer, n: usize) {
    top_bar(g, fb);
    let head = if n == 1 { "THE AGE'S LEADER AWAITS" } else { "WHO DO YOU MEET NEXT?" };
    fb.text_centered(head, 200, 30, ACCENT_HI, Some(OUTLINE), 2);
    fb.text_centered(&format!("MUSIC: {}", g.track().title()), 200, 52, TEXT_FAINT, None, 1);
    for i in 0..n {
        let r = route_rect(i, n);
        let hover = g.hover_route == Some(i);
        if hover {
            fb.rect_bevel(r.x, r.y, r.w, r.h, PANEL_HI, WHITE, PANEL);
        } else {
            ui::panel(fb, r);
        }
        let e = &g.encounters[i];
        let label = if e.leader {
            "A LEADER AND THEIR BAND".to_string()
        } else if i == 0 {
            "A SMALLER BAND".to_string()
        } else {
            "A LARGER BAND".to_string()
        };
        fb.text(&label, r.x + 8, r.y + 6, if hover { PANEL_DEEP } else { TEXT }, None, 1);
        // Their formation, in miniature.
        let (mx, my) = (r.x + (r.w - 7 * 24) / 2, r.y + 20);
        fb.rect_fill(mx - 1, my - 1, 7 * 24 + 2, 3 * 26 + 2, OUTLINE);
        for yy in 0..3 {
            for xx in 0..7 {
                fb.rect_fill(mx + xx * 24, my + yy * 26, 24, 26, if (xx + yy) % 2 == 0 { GROUND_L } else { GROUND_D });
            }
        }
        for (k, p, vet) in &e.rivals {
            let spr = g.art.of(*k);
            let (cx, cy) = (mx + p.0 * 24 + (24 - spr.w) / 2, my + p.1 * 26 + 26 - spr.h - 1);
            fb.blit_map(spr, cx, cy, true, &art::team_map(false));
            if *vet {
                fb.text("*", mx + p.0 * 24 + 17, my + p.1 * 26, GOLD, Some(OUTLINE), 1);
            }
        }
        let vets = e.rivals.iter().filter(|r| r.2).count();
        let mut line = format!("{} PEOPLE", e.rivals.len());
        if vets > 0 {
            line.push_str(&format!(", {vets} VETERAN{}", if vets > 1 { "S" } else { "" }));
        }
        let ty = r.y + 104;
        fb.text(&line, r.x + 8, ty, if hover { PANEL_DEEP } else { TEXT_DIM }, None, 1);
        let reward = if e.leader { "WIN: AN IDEA, AND ROOM FOR ONE MORE ON THE FIELD" } else { "WIN: ANY OF THEM MAY JOIN YOU" };
        for (k, l) in wrap(reward, 28).iter().enumerate() {
            fb.text(l, r.x + 8, ty + 12 + k as i32 * 9, if hover { PANEL_DEEP } else { GOLD }, None, 1);
        }
    }
    draw_bar_hint(g, fb, Some("CLICK A BAND TO MEET THEM.  F1: RULES"));
}

fn draw_keep(g: &Game, fb: &mut Framebuffer, cands: &[(Member, bool)], keep: &[bool]) {
    top_bar(g, fb);
    fb.text_centered("WHO STAYS?", 200, 24, ACCENT_HI, Some(OUTLINE), 2);
    let n = keep.iter().filter(|k| **k).count();
    fb.text_centered(&format!("STAYING: {n} OF {} PLACES", g.band_limit()), 200, 46, TEXT, None, 1);
    for (i, ((m, new), k)) in cands.iter().zip(keep).enumerate() {
        let r = keep_rect(i);
        let hover = g.hover_item == Some(i);
        if *k {
            fb.rect_bevel(r.x, r.y, r.w, r.h, if hover { PANEL_HI } else { PANEL }, WHITE, PANEL_LO);
        } else {
            ui::inset(fb, r);
        }
        let max = max_resolve(m.kind, m.veteran, Side::Blue, &g.ideas);
        draw_person(g, fb, m.kind, true, (r.x + (r.w - TILE) / 2, r.y + 2), max, max, m.veteran);
        let name = m.kind.info().name;
        let short: String = name.chars().take(9).collect();
        fb.text_centered(&short, r.x + r.w / 2, r.y + 34, if *k { TEXT } else { TEXT_FAINT }, None, 1);
        let tag = if *new { "NEW" } else { "" };
        fb.text_centered(tag, r.x + r.w / 2, r.y + 44, if *k { GOLD } else { TEXT_FAINT }, None, 1);
        if !*k {
            fb.rect_checker(r.x + 2, r.y + 2, r.w - 4, 30, PANEL_DEEP, 0);
        }
    }
    if let Some(i) = g.hover_item {
        if let Some((m, _)) = cands.get(i) {
            let info = m.kind.info();
            let s = format!("{}: RESOLVE {}, VOICE {}, PACE {}. {}.", info.name, max_resolve(m.kind, m.veteran, Side::Blue, &g.ideas), info.voice + m.veteran as i32, info.pace, m.kind.reach_text());
            for (k, l) in wrap(&s, 64).iter().take(2).enumerate() {
                fb.text(l, 8, 192 + k as i32 * 9, TEXT_DIM, None, 1);
            }
        }
    }
    draw_bar(g, fb);
    ui::button(fb, KEEP_DONE, "DONE", KEEP_DONE.contains(g.mouse.0, g.mouse.1), true);
}

fn draw_ideas(g: &Game, fb: &mut Framebuffer, offers: &[Idea]) {
    top_bar(g, fb);
    fb.text_centered("A NEW IDEA", 200, 26, ACCENT_HI, Some(OUTLINE), 2);
    fb.text_centered("CHOOSE ONE. IT LASTS THE WHOLE RUN.", 200, 48, TEXT_DIM, None, 1);
    for (i, idea) in offers.iter().enumerate() {
        let r = idea_rect(i);
        let hover = g.hover_item == Some(i);
        if hover {
            fb.rect_bevel(r.x, r.y, r.w, r.h, PANEL_HI, WHITE, PANEL);
        } else {
            ui::panel(fb, r);
        }
        fb.text(idea.name(), r.x + 10, r.y + 8, if hover { PANEL_DEEP } else { ACCENT_HI }, None, 1);
        for (k, l) in wrap(idea.text(), 52).iter().take(2).enumerate() {
            fb.text(l, r.x + 10, r.y + 20 + k as i32 * 9, if hover { PANEL_DEEP } else { TEXT }, None, 1);
        }
    }
}

fn draw_over(g: &Game, fb: &mut Framebuffer, won: bool) {
    top_bar(g, fb);
    let r = Rect::new(40, 40, 320, 250);
    ui::panel(fb, r);
    let (head, sub) = if won {
        ("HISTORY IS YOURS", "A MYTH CARRIES YOUR BANNER NOW.".to_string())
    } else {
        ("OUT OF CHANCES", format!("IN {}, BATTLE {} OF {BATTLES}.", AGES[g.age()], g.index + 1))
    };
    fb.text_centered(head, 200, 52, if won { GOLD } else { BAD }, Some(OUTLINE), 2);
    fb.text_centered(&sub, 200, 74, TEXT, None, 1);
    let s = &g.stats;
    let lines = [
        format!("BATTLES WON {}  LOST {}  DRAWN {}", s.won, s.lost, s.drawn),
        format!("TURNED TO YOUR SIDE: {}", s.turned_to_us),
        format!("TURNED AWAY FROM YOU: {}", s.turned_from_us),
    ];
    for (i, l) in lines.iter().enumerate() {
        fb.text_centered(l, 200, 92 + i as i32 * 11, TEXT_DIM, None, 1);
    }
    fb.text_centered("YOUR BAND AT THE END", 200, 132, TEXT, None, 1);
    let n = g.band.len() as i32;
    let x0 = 200 - n * 30 / 2;
    for (i, m) in g.band.iter().enumerate() {
        let max = max_resolve(m.kind, m.veteran, Side::Blue, &g.ideas);
        draw_person(g, fb, m.kind, true, (x0 + i as i32 * 30, 144), max, max, m.veteran);
    }
    if !g.ideas.is_empty() {
        let s = format!("IDEAS: {}", g.ideas.iter().map(|i| i.name()).collect::<Vec<_>>().join(", "));
        for (i, l) in wrap(&s, 48).iter().take(3).enumerate() {
            fb.text_centered(l, 200, 184 + i as i32 * 10, TEXT_DIM, None, 1);
        }
    }
    ui::button(fb, OVER_GO, "CONTINUE", OVER_GO.contains(g.mouse.0, g.mouse.1), true);
}

fn draw_help(fb: &mut Framebuffer) {
    fb.rect_checker(0, 0, 400, 300, OUTLINE, 0);
    let r = Rect::new(12, 10, 376, 280);
    ui::panel(fb, r);
    fb.text_centered("HOW THE CONTESTS WORK", 200, 18, ACCENT_HI, Some(OUTLINE), 1);
    let rules = [
        "BEFORE EACH BATTLE, DRAG YOUR PEOPLE ONTO THE LIT SQUARES. YOU SEE THE RIVALS FIRST.",
        "THE BATTLE THEN PLAYS ITSELF, IN ROUNDS. FASTEST (HIGHEST PACE) ACT FIRST.",
        "ON THEIR TURN, A PERSON PERSUADES THE RIVAL IN REACH WITH THE LEAST RESOLVE LEFT (TIES: THE NEAREST). IF NOBODY IS IN REACH, THEY WALK ONE STEP CLOSER.",
        "A VOICE TAKES THAT MUCH RESOLVE. AT 0, THEY JOIN THE SIDE THAT WON THEM OVER, WITH HALF THEIR RESOLVE BACK.",
        "WIN EVERYONE OVER TO WIN. AFTER 30 ROUNDS, THE SIDE WITH MORE PEOPLE WINS.",
        "AFTER A WIN, ANYONE ON YOUR SIDE MAY STAY. A LOSS COSTS ONE OF 3 CHANCES.",
        "TURN THREE PEOPLE AND YOU'RE A VETERAN: 1 LOUDER, 2 MORE RESOLVE.",
        "BEAT AN AGE'S LEADER FOR AN IDEA AND ONE MORE PLACE ON THE FIELD.",
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
    y += 2;
    for k in ["MOUSE: DRAG AND CLICK.  ENTER: START.  F: BATTLE SPEED.", "F1: THIS PAGE.  F2: SCANLINES.  F3: MUSIC.  ESC: PAUSE."] {
        fb.text(k, 22, y, TEXT_DIM, None, 1);
        y += 10;
    }
    fb.text_centered("PRESS ANY KEY OR CLICK TO GO BACK", 200, 276, ACCENT, None, 1);
}
