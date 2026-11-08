//! Drawing Conduit.

use super::art::{self, *};
use super::battle::*;
use super::data::*;
use super::*;
use crate::framebuffer::{wrap, Framebuffer};
use crate::palette::*;
use crate::ui::{self, Rect};

pub fn draw(g: &Game, fb: &mut Framebuffer) {
    fb.clear(BG);
    match &g.phase {
        Phase::Workshop => draw_workshop(g, fb),
        Phase::Battle | Phase::After { .. } => draw_battle(g, fb),
        Phase::Reward { offers } => draw_rewards(g, fb, offers),
        Phase::Over { won } => draw_over(g, fb, *won),
    }
    if g.show_help {
        draw_help(fb);
    }
}

fn top_bar(g: &Game, fb: &mut Framebuffer) {
    let title = if g.lesson {
        "THE PRACTICE CONTEST".to_string()
    } else {
        format!("{}  -  CONTEST {} OF {CONTESTS}", DISTRICTS[g.district()], g.contest + 1)
    };
    fb.text(&title, 6, 5, TEXT, Some(OUTLINE), 1);
    // Fuses: little glass tubes, lit while unblown.
    for i in 0..FUSES {
        let x = 394 - 10 - i * 10;
        let lit = (FUSES - 1 - i) < g.fuses;
        fb.rect_fill(x, 3, 7, 11, OUTLINE);
        fb.rect_fill(x + 1, 4, 5, 9, if lit { SPARK } else { SLOT_BG });
        fb.rect_fill(x + 3, 5, 1, 7, if lit { SPARK + 2 } else { SLOT_HI });
        fb.rect_fill(x + 1, 3, 5, 1, METAL + 1);
        fb.rect_fill(x + 1, 13, 5, 1, METAL + 1);
    }
    fb.text("FUSES", 394 - 34 - 34, 5, TEXT_DIM, None, 1);
}

fn stat_line(s: &data::Stats) -> String {
    let mut v = format!("HP {}  STEP {}", s.max_hp, s.step);
    if s.armor > 0 {
        v.push_str(&format!("  AR {}", s.armor));
    }
    if s.dodge > 0 {
        v.push_str(&format!("  DG {}", s.dodge));
    }
    if s.ward > 0 {
        v.push_str(&format!("  WD {}", s.ward));
    }
    v
}

fn bar(fb: &mut Framebuffer, x: i32, y: i32, label: &str, used: i32, cap: i32, col: u8) {
    fb.text(label, x, y, TEXT_DIM, None, 1);
    for k in 0..cap.max(0) {
        let c = if k < used { col } else { SLOT_BG };
        fb.rect_fill(x + 8 + k * 5, y, 4, 7, OUTLINE);
        fb.rect_fill(x + 9 + k * 5, y + 1, 2, 5, c);
    }
    fb.text(&format!("{used}/{cap}"), x + 10 + cap * 5, y, if used > cap { BAD } else { TEXT_DIM }, None, 1);
}

fn draw_workshop(g: &Game, fb: &mut Framebuffer) {
    top_bar(g, fb);
    let held = g.held;
    for (m, b) in g.crew.iter().enumerate() {
        let r = card_rect(m);
        ui::panel(fb, r);
        let spr = g.art.of(b.body);
        fb.blit_map(spr, r.x + 3 + (22 - spr.w) / 2, r.y + 3 + (24 - spr.h), false, &art::team_map(true));
        let info = b.body.info();
        fb.text(info.name, r.x + 28, r.y + 4, ACCENT_HI, Some(OUTLINE), 1);
        let s = b.stats();
        let line = stat_line(&s);
        for (k, l) in wrap(&line, 14).iter().take(2).enumerate() {
            fb.text(l, r.x + 28, r.y + 14 + k as i32 * 9, TEXT_DIM, None, 1);
        }
        let (ub, um) = b.used();
        bar(fb, r.x + 3, r.y + 30, "B", ub, info.body, WOOD + 2);
        bar(fb, r.x + 3, r.y + 38, "M", um, info.mind, SPARK + 1);
        for (si, slot) in b.body.slots().iter().enumerate() {
            let sr = slot_rect(m, si);
            let hover = sr.contains(g.mouse.0, g.mouse.1);
            let fits = held.is_some_and(|(p, from)| from != From::Slot(m, si) && b.refusal(si, p).is_none() && p.info().slot == *slot);
            let fill = if hover { SLOT_HI } else { SLOT_BG };
            fb.rect_fill(sr.x, sr.y, sr.w, sr.h - 1, fill);
            if fits {
                fb.corners(sr.x, sr.y, sr.w, sr.h - 1, 3, GOOD);
            }
            fb.text(slot.letter(), sr.x + 2, sr.y + 1, ACCENT, None, 1);
            match b.parts[si] {
                Some(p) if held.map(|h| h.1) != Some(From::Slot(m, si)) => {
                    fb.text(p.info().name, sr.x + 10, sr.y + 1, TEXT, None, 1);
                }
                _ => fb.text("-", sr.x + 10, sr.y + 1, TEXT_FAINT, None, 1),
            }
        }
    }
    for m in g.crew.len()..MAX_CREW {
        let r = card_rect(m);
        ui::inset(fb, r);
        fb.text_centered("(EMPTY PLACE)", r.x + r.w / 2, r.y + r.h / 2 - 4, TEXT_FAINT, None, 1);
    }

    // Spare parts.
    ui::panel(fb, SHELF);
    fb.text("SPARE PARTS", SHELF.x + 4, SHELF.y + 3, ACCENT_HI, Some(OUTLINE), 1);
    let shelf_hover = SHELF.contains(g.mouse.0, g.mouse.1);
    if held.is_some_and(|h| matches!(h.1, From::Slot(..))) && shelf_hover {
        fb.rect_outline(SHELF.x + 1, SHELF.y + 1, SHELF.w - 2, SHELF.h - 2, GOOD);
    }
    for (k, p) in g.shelf.iter().enumerate().take(SHELF_ROWS) {
        let r = shelf_rect(k);
        if held.map(|h| h.1) == Some(From::Shelf(k)) {
            continue;
        }
        if r.contains(g.mouse.0, g.mouse.1) {
            fb.rect_fill(r.x, r.y, r.w, r.h - 1, SLOT_HI);
        }
        let i = p.info();
        fb.text(i.slot.letter(), r.x + 1, r.y + 2, ACCENT, None, 1);
        fb.text(i.name, r.x + 9, r.y + 2, TEXT, None, 1);
        let cost = format!("{}{}", if i.body > 0 { format!("B{}", i.body) } else { String::new() }, if i.mind > 0 { format!("M{}", i.mind) } else { String::new() });
        fb.text(&cost, r.x + r.w - 1 - Framebuffer::text_width(&cost, 1), r.y + 2, TEXT_DIM, None, 1);
    }
    if g.shelf.len() > SHELF_ROWS {
        fb.text(&format!("+{} MORE", g.shelf.len() - SHELF_ROWS), SHELF.x + 4, SHELF.y + SHELF.h - 10, TEXT_FAINT, None, 1);
    }

    // Rivals.
    ui::panel(fb, RIVAL_BOX);
    fb.text("NEXT RIVALS", RIVAL_BOX.x + 4, RIVAL_BOX.y + 4, BAD, Some(OUTLINE), 1);
    let name: String = g.rival.name.chars().take(20).collect();
    fb.text(&name, RIVAL_BOX.x + 4, RIVAL_BOX.y + 13, TEXT, None, 1);
    for (k, b) in g.rival.crew.iter().enumerate() {
        let r = rival_rect(k);
        if r.contains(g.mouse.0, g.mouse.1) {
            fb.rect_fill(r.x, r.y, r.w, r.h, SLOT_HI);
        }
        let spr = g.art.of(b.body);
        fb.blit_map(spr, r.x + (r.w - spr.w) / 2, r.y + r.h - spr.h, true, &art::team_map(false));
    }
    let can = !(g.lesson && g.lesson_step < 2);
    ui::button(fb, FIGHT_BTN, "FIGHT", FIGHT_BTN.contains(g.mouse.0, g.mouse.1) && can, can);

    draw_info(g, fb);

    // The held part follows the pointer.
    if let Some((p, _)) = held {
        let name = p.info().name;
        let w = Framebuffer::text_width(name, 1) + 6;
        let (x, y) = (g.mouse.0 + 8, g.mouse.1 + 4);
        fb.rect_fill(x, y, w, 11, OUTLINE);
        fb.rect_fill(x + 1, y + 1, w - 2, 9, ACCENT);
        fb.text(name, x + 3, y + 2, PANEL_DEEP, None, 1);
    }
}

fn describe_build(b: &Build) -> Vec<(String, u8)> {
    let s = b.stats();
    let mut v = vec![(format!("{}: {}", b.body.info().name, stat_line(&s)), TEXT)];
    for a in &s.attacks {
        let mut t = format!("{} {} REACH {} EVERY {}", a.dmg_type.name(), a.dmg, a.range, a.cooldown);
        if a.sweep {
            t.push_str(" SWEEP");
        }
        if a.burst {
            t.push_str(" BURST");
        }
        if a.burn > 0 {
            t.push_str(" BURN");
        }
        v.push((t, TEXT));
    }
    let mut extra: Vec<&str> = Vec::new();
    if s.heal {
        extra.push("HEALS");
    }
    if s.lifelink {
        extra.push("FANG");
    }
    if s.spikes {
        extra.push("SPIKES");
    }
    if s.stone_gut {
        extra.push("STONE GUT");
    }
    if s.regen {
        extra.push("REGROWS");
    }
    if !extra.is_empty() {
        v.push((extra.join(", "), GOLD));
    }
    v.push((s.brain.text().to_string(), TEXT_DIM));
    v
}

fn draw_info(g: &Game, fb: &mut Framebuffer) {
    ui::inset(fb, INFO);
    let x0 = INFO.x + 5;
    let cols = ((INFO.w - 10) / 6) as usize;
    let mut y = INFO.y + 4;
    // Lesson or message first.
    let top: Option<(String, u8)> = if g.lesson && g.lesson_step < LESSON_STEPS.len() {
        Some((format!("LESSON {}/{}: {}", g.lesson_step + 1, LESSON_STEPS.len(), LESSON_STEPS[g.lesson_step]), ACCENT_HI))
    } else {
        g.message.clone().map(|m| (m, ACCENT_HI))
    };
    let mut lines_left = 9;
    if let Some((t, col)) = top {
        for l in wrap(&t, cols).iter().take(4) {
            fb.text(l, x0, y, col, None, 1);
            y += 9;
            lines_left -= 1;
        }
        y += 2;
    }
    let (mx, my) = g.mouse;
    let mut lines: Vec<(String, u8)> = Vec::new();
    if let Some((p, _)) = g.held {
        let i = p.info();
        lines.push((format!("{} ({} SLOT, BODY {}, MIND {})", i.name, i.slot.name(), i.body, i.mind), TEXT));
        lines.push((i.text.to_string(), TEXT_DIM));
        lines.push(("CLICK A LIT SLOT TO FIT IT, OR THE SHELF TO PUT IT BACK.".into(), TEXT_FAINT));
    } else if let Some(f) = g.spot_at(mx, my) {
        if let Some(p) = g.part_at(f) {
            let i = p.info();
            lines.push((format!("{} ({} SLOT, BODY {}, MIND {})", i.name, i.slot.name(), i.body, i.mind), TEXT));
            lines.push((i.text.to_string(), TEXT_DIM));
        } else if let From::Slot(m, _) = f {
            lines.extend(describe_build(&g.crew[m]));
        }
    } else if let Some(m) = (0..g.crew.len()).find(|&m| card_rect(m).contains(mx, my)) {
        lines.extend(describe_build(&g.crew[m]));
        lines.push((g.crew[m].body.info().note.to_string(), TEXT_FAINT));
    } else if let Some(k) = (0..g.rival.crew.len()).find(|&k| rival_rect(k).contains(mx, my)) {
        let b = &g.rival.crew[k];
        let parts: Vec<&str> = b.fitted().iter().map(|p| p.info().name).collect();
        lines.push((format!("RIVAL {}: {}", b.body.info().name, if parts.is_empty() { "NO GEAR".into() } else { parts.join(", ") }), BAD));
        lines.extend(describe_build(b).into_iter().skip(0));
    } else if RIVAL_BOX.contains(mx, my) {
        lines.push((format!("{}: {}", g.rival.name, g.rival.text), TEXT));
    } else if lines_left == 9 {
        lines.push(("POINT AT A MONSTER, A PART OR A RIVAL FOR DETAILS. DRAG PARTS INTO SLOTS.".into(), TEXT_DIM));
    }
    for (t, col) in lines {
        for l in wrap(&t, cols) {
            if lines_left <= 0 {
                return;
            }
            fb.text(&l, x0, y, col, None, 1);
            y += 9;
            lines_left -= 1;
        }
    }
}

fn tile_xy(p: Pos) -> (i32, i32) {
    (AX + p.0 * AT, AY + p.1 * AT)
}

fn unit_px(b: &Battle, u: &Unit) -> (i32, i32) {
    let (a, c) = (tile_xy(u.moved_from), tile_xy(u.pos));
    let n = u.stats.step.max(1) as i32;
    let k = (b.tick.saturating_sub(u.moved_at) as i32).min(n);
    (a.0 + (c.0 - a.0) * k / n, a.1 + (c.1 - a.1) * k / n)
}

fn draw_conduit(fb: &mut Framebuffer, side: Side, hp: i32, tick: u32) {
    let tiles = conduit_tiles(side);
    let (x, y) = tile_xy(tiles[0]);
    let team = if side == Side::Blue { BLUE } else { RED };
    // Base, mast, coil rings and a glowing ball.
    fb.rect_fill(x + 2, y + 3 * AT - 8, AT - 4, 7, OUTLINE);
    fb.rect_fill(x + 3, y + 3 * AT - 7, AT - 6, 5, METAL);
    fb.rect_fill(x + 9, y + 14, 6, 3 * AT - 22, OUTLINE);
    fb.rect_fill(x + 10, y + 14, 4, 3 * AT - 22, METAL + 1);
    for k in 0..6 {
        let ry = y + 20 + k * 8;
        fb.rect_fill(x + 5, ry, 14, 4, OUTLINE);
        fb.rect_fill(x + 6, ry + 1, 12, 2, if k % 2 == 0 { WOOD + 2 } else { WOOD + 1 });
    }
    let alive = hp > 0;
    fb.rect_fill(x + 6, y + 3, 12, 12, OUTLINE);
    fb.rect_fill(x + 7, y + 4, 10, 10, if alive { team + 1 } else { SLOT_BG });
    fb.rect_fill(x + 9, y + 6, 4, 3, if alive { team + 2 } else { SLOT_HI });
    if alive && (tick / 5) % 3 == 0 {
        // A slow crackle around the ball (ambience, not a UI flash).
        fb.pset(x + 4, y + 6, SPARK + 2);
        fb.pset(x + 19, y + 10, SPARK + 2);
    }
    // HP bar under it.
    let w = AT - 2;
    fb.rect_fill(x + 1, y + 3 * AT - 2, w, 3, OUTLINE);
    let fill = (hp.max(0) * (w - 2) + CONDUIT_HP - 1) / CONDUIT_HP;
    fb.rect_fill(x + 2, y + 3 * AT - 1, fill, 1, team + 2);
}

fn draw_battle(g: &Game, fb: &mut Framebuffer) {
    top_bar(g, fb);
    let Some(b) = &g.battle else { return };
    fb.rect_fill(AX - 2, AY - 2, W * AT + 4, H * AT + 4, OUTLINE);
    for y in 0..H {
        for x in 0..W {
            let (tx, ty) = tile_xy((x, y));
            fb.rect_fill(tx, ty, AT, AT, if (x + y) % 2 == 0 { GROUND_L } else { GROUND_D });
            if (x * 5 + y * 3) % 7 == 0 {
                fb.rect_fill(tx + 4, ty + AT - 6, 5, 1, GROUND_X);
            }
        }
    }
    draw_conduit(fb, Side::Blue, b.conduit_hp[0], b.tick);
    draw_conduit(fb, Side::Red, b.conduit_hp[1], b.tick);
    // Units: dead ones leave a scorch.
    for u in b.units.iter().filter(|u| !u.alive) {
        let (x, y) = tile_xy(u.pos);
        fb.rect_checker(x + 4, y + AT - 6, AT - 8, 3, OUTLINE, 0);
    }
    for u in b.units.iter().filter(|u| u.alive) {
        let (x, y) = unit_px(b, u);
        let spr = g.art.of(u.build.body);
        let blue = u.side == Side::Blue;
        fb.rect_checker(x + 4, y + AT - 4, AT - 8, 2, if blue { BLUE } else { RED }, 0);
        fb.blit_map(spr, x + (AT - spr.w) / 2, y + AT - 3 - spr.h, !blue, &art::team_map(blue));
        let w = AT - 4;
        fb.rect_fill(x + 2, y - 1, w, 3, OUTLINE);
        let fill = (u.hp.max(0) * (w - 2) + u.stats.max_hp - 1) / u.stats.max_hp.max(1);
        fb.rect_fill(x + 3, y, fill.min(w - 2), 1, if blue { BLUE + 2 } else { RED + 2 });
        if u.burn > 0 {
            fb.pset(x + AT - 5, y + 4, FIRE + 1);
            fb.pset(x + AT - 4, y + 5, FIRE);
        }
    }
    // Effects from the last few ticks.
    for (ft, fx) in &b.fx {
        let age = b.tick.saturating_sub(*ft) as i32;
        let centre = |p: Pos| (AX + p.0 * AT + AT / 2, AY + p.1 * AT + AT / 2);
        match *fx {
            Fx::Hit { from, to, dmg_type, dmg, primary } => {
                let (a, c) = (centre(from), centre(to));
                if age <= 3 {
                    match dmg_type {
                        Dmg::Melee => {
                            fb.line(c.0 - 4, c.1 - 4, c.0 + 4, c.1 + 4, WHITE);
                            fb.line(c.0 - 3, c.1 - 4, c.0 + 5, c.1 + 4, OUTLINE);
                        }
                        Dmg::Ranged => {
                            // The shot flies over 3 ticks.
                            let k = (age + 1).min(3);
                            let (px, py) = (a.0 + (c.0 - a.0) * k / 3, a.1 + (c.1 - a.1) * k / 3);
                            fb.rect_fill(px - 1, py - 1, 3, 3, OUTLINE);
                            fb.pset(px, py, WOOD + 2);
                        }
                        Dmg::Magic => {
                            fb.line(a.0, a.1, c.0, c.1, SPARK + 1);
                            fb.circle(c.0, c.1, 3 + age, SPARK + 2);
                        }
                    }
                }
                if primary && age <= 6 {
                    let col = if dmg_type == Dmg::Magic { SPARK + 2 } else { WHITE };
                    fb.text(&format!("-{dmg}"), c.0 - 5, c.1 - 14 - age, col, Some(OUTLINE), 1);
                }
            }
            Fx::Heal { from, to, amount } if age <= 6 => {
                let (a, c) = (centre(from), centre(to));
                fb.line(a.0, a.1, c.0, c.1, HEAL);
                fb.text(&format!("+{amount}"), c.0 - 5, c.1 - 14 - age, HEAL, Some(OUTLINE), 1);
            }
            Fx::Burn { at } if age <= 4 => {
                let c = centre(at);
                fb.pset(c.0, c.1 - 6 - age, FIRE + 1);
                fb.pset(c.0 + 2, c.1 - 4 - age, FIRE);
            }
            Fx::Died { at, .. } if age <= 6 => {
                let c = centre(at);
                fb.circle(c.0, c.1, 2 + age * 2, TEXT_DIM);
            }
            Fx::ConduitHit { side, dmg } if age <= 6 => {
                let (x, y) = tile_xy(conduit_tiles(side)[0]);
                fb.text(&format!("-{dmg}"), x + 4, y - 2 - age, GOLD, Some(OUTLINE), 1);
            }
            _ => {}
        }
    }

    // Crew status below the arena.
    for (side, x0) in [(Side::Blue, 4), (Side::Red, 202)] {
        let r = Rect::new(x0, 192, 194, 46);
        ui::inset(fb, r);
        let head = if side == Side::Blue { "YOUR CREW" } else { g.rival.name };
        fb.text(head, r.x + 4, r.y + 3, if side == Side::Blue { BLUE + 2 } else { RED + 2 }, None, 1);
        fb.text(&format!("CONDUIT {}", b.conduit_hp[side.idx()].max(0)), r.x + r.w - 70, r.y + 3, GOLD, None, 1);
        for (k, u) in b.units.iter().filter(|u| u.side == side).enumerate() {
            let (cx, cy) = (r.x + 4 + (k % 2) as i32 * 95, r.y + 14 + (k / 2) as i32 * 15);
            let name: String = u.build.body.info().name.chars().take(7).collect();
            fb.text(&name, cx, cy, if u.alive { TEXT } else { TEXT_FAINT }, None, 1);
            let w = 40;
            fb.rect_fill(cx + 44, cy + 1, w + 2, 5, OUTLINE);
            let fill = (u.hp.max(0) * w + u.stats.max_hp - 1) / u.stats.max_hp.max(1);
            fb.rect_fill(cx + 45, cy + 2, fill.min(w), 3, if side == Side::Blue { BLUE + 1 } else { RED + 1 });
        }
    }

    // Message bar and speed.
    let bar = Rect::new(4, 242, 266, 56);
    ui::inset(fb, bar);
    let text = g.note.clone().or_else(|| g.message.clone());
    if let Some(t) = text {
        for (k, l) in wrap(&t, 42).iter().take(5).enumerate() {
            fb.text(l, bar.x + 5, bar.y + 4 + k as i32 * 9, TEXT, None, 1);
        }
        if g.note.is_some() {
            fb.text("CLICK TO GO ON", bar.x + 5, bar.y + bar.h - 10, ACCENT, None, 1);
        }
    } else {
        fb.text(&format!("TICK {} OF {}", b.tick, TICK_CAP), bar.x + 5, bar.y + 4, TEXT_DIM, None, 1);
        fb.text("F OR THE BUTTON: SPEED.  F1: RULES", bar.x + 5, bar.y + 40, TEXT_FAINT, None, 1);
    }
    ui::button(fb, SPEED_BTN, &format!("{}X", g.speed), SPEED_BTN.contains(g.mouse.0, g.mouse.1), true);
    ui::button(fb, SKIP_BTN, "SKIP", SKIP_BTN.contains(g.mouse.0, g.mouse.1), !g.lesson);

    if let Phase::After { outcome, .. } = &g.phase {
        let r = Rect::new(100, 70, 200, 52);
        ui::panel(fb, r);
        let (a, col) = match outcome {
            Outcome::Won => ("THEIR CONDUIT IS DOWN", GOOD),
            Outcome::Lost => ("YOUR CONDUIT IS DOWN", BAD),
            Outcome::Draw => ("TIME: A DRAW", GOLD),
        };
        let a = if b.tick >= TICK_CAP && *outcome != Outcome::Draw {
            if *outcome == Outcome::Won {
                "TIME: YOURS HELD BETTER"
            } else {
                "TIME: THEIRS HELD BETTER"
            }
        } else {
            a
        };
        fb.text_centered(a, 200, r.y + 12, col, Some(OUTLINE), 1);
        fb.text_centered("CLICK TO CONTINUE", 200, r.y + 32, TEXT_DIM, None, 1);
    }
}

fn draw_rewards(g: &Game, fb: &mut Framebuffer, offers: &[Reward]) {
    top_bar(g, fb);
    fb.text_centered("SPOILS", 200, 24, ACCENT_HI, Some(OUTLINE), 2);
    fb.text_centered("KEEP ONE.", 200, 48, TEXT_DIM, None, 1);
    for (i, r) in offers.iter().enumerate() {
        let rr = reward_rect(i);
        let hover = g.hover_item == Some(i);
        if hover {
            fb.rect_bevel(rr.x, rr.y, rr.w, rr.h, PANEL_HI, WHITE, PANEL);
        } else {
            ui::panel(fb, rr);
        }
        let tc = if hover { PANEL_DEEP } else { TEXT };
        match r {
            Reward::Part(p) => {
                let i = p.info();
                fb.text(&format!("{}  ({} SLOT, BODY {}, MIND {})", i.name, i.slot.name(), i.body, i.mind), rr.x + 8, rr.y + 6, if hover { PANEL_DEEP } else { ACCENT_HI }, None, 1);
                for (k, l) in wrap(i.text, 56).iter().take(3).enumerate() {
                    fb.text(l, rr.x + 8, rr.y + 18 + k as i32 * 9, tc, None, 1);
                }
            }
            Reward::Crate(ps) => {
                fb.text("A CRATE OF TWO PARTS", rr.x + 8, rr.y + 6, if hover { PANEL_DEEP } else { ACCENT_HI }, None, 1);
                for (k, p) in ps.iter().enumerate() {
                    let i = p.info();
                    fb.text(&format!("{} ({}): {}", i.name, i.slot.letter(), i.text.chars().take(36).collect::<String>()), rr.x + 8, rr.y + 18 + k as i32 * 10, tc, None, 1);
                }
            }
            Reward::Monster(b) => {
                let info = b.info();
                let spr = g.art.of(*b);
                fb.blit_map(spr, rr.x + 6, rr.y + rr.h - spr.h - 4, false, &art::team_map(true));
                fb.text(&format!("A NEW MONSTER: {}", info.name), rr.x + 34, rr.y + 6, if hover { PANEL_DEEP } else { ACCENT_HI }, None, 1);
                fb.text(
                    &format!("HP {}, STEP {}, SLOTS {}H {}W {}I, BODY {}, MIND {}", info.hp, info.step, info.held, info.worn, info.inner, info.body, info.mind),
                    rr.x + 34,
                    rr.y + 18,
                    tc,
                    None,
                    1,
                );
                fb.text(info.note, rr.x + 34, rr.y + 28, tc, None, 1);
            }
        }
    }
    if let Some(m) = &g.message {
        for (k, l) in wrap(m, 62).iter().take(2).enumerate() {
            fb.text(l, 14, 252 + k as i32 * 9, ACCENT_HI, None, 1);
        }
    }
}

fn draw_over(g: &Game, fb: &mut Framebuffer, won: bool) {
    top_bar(g, fb);
    let r = Rect::new(40, 30, 320, 262);
    ui::panel(fb, r);
    let (head, sub) = if won {
        ("THE LEAGUE IS YOURS", "THE RAILWAY BARONS' CONDUIT IS DOWN.".to_string())
    } else {
        ("OUT OF FUSES", format!("AT {}, CONTEST {} OF {CONTESTS}.", DISTRICTS[g.district()], g.contest + 1))
    };
    fb.text_centered(head, 200, 42, if won { GOLD } else { BAD }, Some(OUTLINE), 2);
    fb.text_centered(&sub, 200, 64, TEXT, None, 1);
    let s = &g.stats;
    fb.text_centered(&format!("CONTESTS WON {}  LOST {}  DRAWN {}", s.won, s.lost, s.drawn), 200, 80, TEXT_DIM, None, 1);
    fb.text_centered("YOUR CREW AT THE END", 200, 98, TEXT, None, 1);
    let mut y = 112;
    for b in &g.crew {
        let spr = g.art.of(b.body);
        fb.blit_map(spr, 56, y + 22 - spr.h, false, &art::team_map(true));
        let parts: Vec<&str> = b.fitted().iter().map(|p| p.info().name).collect();
        let t = format!("{}: {}", b.body.info().name, if parts.is_empty() { "NO GEAR".into() } else { parts.join(", ") });
        for (k, l) in wrap(&t, 42).iter().take(2).enumerate() {
            fb.text(l, 84, y + 4 + k as i32 * 9, TEXT_DIM, None, 1);
        }
        y += 34;
    }
    ui::button(fb, OVER_GO, "CONTINUE", OVER_GO.contains(g.mouse.0, g.mouse.1), true);
}

fn draw_help(fb: &mut Framebuffer) {
    fb.rect_checker(0, 0, 400, 300, OUTLINE, 0);
    let r = Rect::new(12, 10, 376, 280);
    ui::panel(fb, r);
    fb.text_centered("HOW THE CONTESTS WORK", 200, 18, ACCENT_HI, Some(OUTLINE), 1);
    let rules = [
        "EACH MONSTER HAS HELD (H), WORN (W) AND INNER (I) SLOTS. EACH PART FITS ONE KIND OF SLOT AND USES SOME OF THE MONSTER'S BODY AND MIND.",
        "YOU SEE THE RIVAL CREW BEFORE YOU FIGHT. POINT AT THEM TO SEE THEIR GEAR.",
        "THE CONTEST PLAYS ITSELF IN TICKS. EVERY ATTACK OR STEP TAKES SOME TICKS BEFORE THAT MONSTER CAN ACT AGAIN.",
        "A HIT DOES ITS DAMAGE MINUS THE MATCHING DEFENSE: MELEE VS ARMOR, RANGED VS DODGE, MAGIC VS WARD. AT LEAST 1.",
        "MONSTERS WALK STRAIGHT (NO DIAGONALS). REACH COUNTS STRAIGHT STEPS.",
        "BREAK THE RIVAL CONDUIT (30 HP) TO WIN. AT 900 TICKS, THE HEALTHIER CONDUIT WINS.",
        "A LOSS BLOWS ONE OF 3 FUSES. AFTER A WIN, KEEP ONE OF THREE SPOILS.",
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
    for k in ["MOUSE: DRAG OR CLICK PARTS. RIGHT-CLICK: BACK TO THE SHELF.", "ENTER: FIGHT.  F: CONTEST SPEED.  F1: THIS PAGE.  ESC: PAUSE."] {
        fb.text(k, 22, y, TEXT_DIM, None, 1);
        y += 10;
    }
    fb.text_centered("PRESS ANY KEY OR CLICK TO GO BACK", 200, 276, ACCENT, None, 1);
}
