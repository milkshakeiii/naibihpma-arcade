//! Drawing HATCHERY.

use super::art::{self, *};
use super::battle::{deploy_squares, Fx, Pos, Side, H, NEST, W};
use super::data::*;
use super::*;
use crate::framebuffer::{wrap, Framebuffer, IDENTITY};
use crate::palette::*;
use crate::ui::{self, Rect};

pub fn draw(g: &Game, fb: &mut Framebuffer) {
    fb.clear(BG);
    match &g.phase {
        Phase::Hatchery => draw_hatchery(g, fb),
        Phase::Deploy | Phase::Night | Phase::Dawn { .. } => draw_pond(g, fb),
        Phase::Over { won } => draw_over(g, fb, *won),
    }
    if g.dialog.is_some() {
        draw_dialog(g, fb);
    }
    if g.show_help {
        draw_help(fb);
    }
}

/// Draws a familiar from its body and genes, feet on `base_y`, centred on `cx`.
pub fn draw_familiar(g: &Game, fb: &mut Framebuffer, body: Body, genes: &[Gene], cx: i32, base_y: i32, ghost: bool) {
    let spr = g.art.body(body);
    let (sx, sy) = (cx - spr.w / 2, base_y - spr.h);
    let ramp = art::body_ramp(body);
    let map = if ghost {
        art::flat_map(PANEL_LO)
    } else if genes.contains(&Gene::BrightColors) {
        art::ramp_map(ramp, [BAD_DIM, BAD, GOLD])
    } else if genes.contains(&Gene::BarkSkin) {
        art::ramp_map(ramp, [HOG_R, HOG_R + 1, HOG_R + 2])
    } else {
        IDENTITY
    };
    // Legs go under the body.
    if genes.contains(&Gene::LeapingLegs) {
        fb.line(sx + 1, sy + spr.h - 1, sx + 5, sy + spr.h - 6, OUTLINE);
        fb.line(sx + 2, sy + spr.h - 1, sx + 6, sy + spr.h - 6, ramp);
    }
    fb.blit_map(spr, sx, sy, false, &map);
    if ghost {
        return;
    }
    let (w, h) = (spr.w, spr.h);
    let mouth = (sx + w - 2, sy + h / 2);
    for gene in genes {
        match gene {
            Gene::LongTongue => {
                fb.line(mouth.0, mouth.1, mouth.0 + 5, mouth.1 + 1, AXO_R + 1);
                fb.pset(mouth.0 + 6, mouth.1 + 1, AXO_R + 2);
            }
            Gene::WideMouth => fb.rect_fill(mouth.0 - 6, mouth.1, 7, 1, OUTLINE),
            Gene::VenomFangs => {
                fb.pset(mouth.0 - 1, mouth.1 + 1, WHITE);
                fb.pset(mouth.0 - 3, mouth.1 + 1, WHITE);
            }
            Gene::FireGland => fb.rect_fill(mouth.0 - 4, mouth.1 + 2, 3, 2, NEWT_R + 1),
            Gene::ChorusSac => {
                fb.rect_fill(mouth.0 - 6, mouth.1 + 1, 5, 4, OUTLINE);
                fb.rect_fill(mouth.0 - 5, mouth.1 + 2, 3, 2, GOLD);
            }
            Gene::Warts => {
                for k in 0..4 {
                    fb.pset(sx + 3 + k * (w / 5), sy + h / 3 + (k % 2) * 2, ramp);
                }
            }
            Gene::Slime => {
                fb.pset(sx + w / 3, sy + h / 3, WHITE);
                fb.pset(sx + w / 2, sy + h / 3 - 1, WHITE);
            }
            Gene::GlowSpots => {
                for k in 0..3 {
                    fb.pset(sx + 4 + k * (w / 4), sy + h / 2 - 1, GOLD);
                }
            }
            Gene::StickyPads => {
                fb.pset(sx + 2, sy + h - 1, GOLD);
                fb.pset(sx + w - 4, sy + h - 1, GOLD);
            }
            Gene::WebbedFeet => {
                fb.rect_fill(sx + 1, sy + h - 2, 3, 1, WATER + 2);
                fb.rect_fill(sx + w - 5, sy + h - 2, 3, 1, WATER + 2);
            }
            Gene::DiggingClaws => {
                fb.pset(sx + 1, sy + h - 1, WHITE);
                fb.pset(sx + w - 3, sy + h - 1, WHITE);
            }
            Gene::SplitTail => fb.pset(sx + 2, sy + h / 2 + 1, OUTLINE),
            Gene::Regrowth => fb.rect_fill(sx, sy + h / 2, 2, 2, GOOD),
            Gene::EggSac => {
                for k in 0..3 {
                    fb.rect_fill(sx + w / 3 + k * 3, sy + 1, 2, 2, EGG);
                }
            }
            Gene::ClubTail => {
                fb.rect_fill(sx - 2, sy + h / 2 - 1, 4, 4, OUTLINE);
                fb.rect_fill(sx - 1, sy + h / 2, 2, 2, HOG_R + 1);
            }
            _ => {}
        }
    }
    if body == Body::Axolotl {
        fb.pset(sx + w - 6, sy + 1, AXO_R + 2);
    }
}

fn top_bar(g: &Game, fb: &mut Framebuffer) {
    let title = match g.phase {
        Phase::Night | Phase::Deploy => format!("NIGHT {} OF {NIGHTS}{}", g.night, if g.night == NIGHTS { ": GROF COMES" } else { "" }),
        _ => format!("THE HATCHERY  -  BEFORE NIGHT {} OF {NIGHTS}", g.night),
    };
    fb.text(&title, 6, 5, TEXT, Some(OUTLINE), 1);
    let eggs = match (&g.phase, &g.battle) {
        (Phase::Night, Some(b)) => b.eggs,
        _ => g.eggs,
    };
    fb.text("EGGS", 300, 5, TEXT_DIM, None, 1);
    for k in 0..eggs.min(12) {
        draw_egg(fb, 330 + k * 6, 6);
    }
}

fn draw_egg(fb: &mut Framebuffer, x: i32, y: i32) {
    fb.rect_fill(x, y, 5, 6, OUTLINE);
    fb.rect_fill(x + 1, y + 1, 3, 4, EGG);
    fb.pset(x + 2, y + 2, EGG + 1);
}

fn draw_hatchery(g: &Game, fb: &mut Framebuffer) {
    top_bar(g, fb);
    for i in 0..MAX_ROSTER {
        let r = card_rect(i);
        let Some(f) = g.roster.get(i) else {
            ui::inset(fb, r);
            fb.text_centered("(EMPTY)", r.x + r.w / 2, r.y + r.h / 2 - 4, TEXT_FAINT, None, 1);
            continue;
        };
        let hover = g.hover_item == Some(i);
        if hover {
            fb.rect_bevel(r.x, r.y, r.w, r.h, PANEL_HI, WHITE, PANEL);
        } else {
            ui::panel(fb, r);
        }
        draw_familiar(g, fb, f.body, &f.genes, r.x + 18, r.y + 30, false);
        let (tc, dc) = if hover { (PANEL_DEEP, PANEL_DEEP) } else { (ACCENT_HI, TEXT_DIM) };
        fb.text(f.name, r.x + 36, r.y + 6, tc, None, 1);
        fb.text(f.body.info().name, r.x + 36, r.y + 16, dc, None, 1);
        fb.text(&format!("{} LV{} HP{}", f.stage(), f.level, f.max_hp()), r.x + 4, r.y + 34, dc, None, 1);
        // XP toward the next level.
        let next = LEVELS.get((f.level - 1) as usize).copied();
        if let Some(n) = next {
            let have = f.xp.min(n);
            for k in 0..n {
                fb.rect_fill(r.x + 4 + k * 4, r.y + 43, 3, 2, if k < have { GOOD } else { PANEL_DEEP });
            }
        }
        let locs = f.body.info().locs;
        for (k, loc) in locs.iter().enumerate() {
            let y = r.y + 48 + k as i32 * 10;
            let (t, col) = match f.genes.get(k) {
                Some(gene) => (format!("{} {}", &loc.name()[..1], gene.name()), if hover { PANEL_DEEP } else { TEXT }),
                None => (format!("{} (AT LEVEL {})", &loc.name()[..1], k + 1), if hover { PANEL_DEEP } else { TEXT_FAINT }),
            };
            fb.text(&t, r.x + 4, y, col, None, 1);
        }
        if f.level_due() {
            let gr = grow_rect(i);
            ui::button(fb, gr, "GROW!", gr.contains(g.mouse.0, g.mouse.1), true);
        }
    }
    // The clutch.
    ui::panel(fb, CLUTCH);
    fb.text("THE CLUTCH", CLUTCH.x + 6, CLUTCH.y + 5, ACCENT_HI, Some(OUTLINE), 1);
    for k in 0..g.eggs.min(16) {
        draw_egg(fb, CLUTCH.x + 6 + (k % 8) * 14, CLUTCH.y + 18 + (k / 8) * 10);
    }
    let can = g.can_hatch() && !(g.lesson && g.lesson_step > 0 && g.lesson_step != 3 && g.lesson_step < 4);
    ui::button(fb, HATCH_BTN, "HATCH (1 EGG)", HATCH_BTN.contains(g.mouse.0, g.mouse.1) && can, can);
    let night_ok = !(g.lesson && g.lesson_step < 4);
    ui::button(fb, NIGHT_BTN, "NIGHT FALLS", NIGHT_BTN.contains(g.mouse.0, g.mouse.1) && night_ok, night_ok);

    // Info: lesson, message, or what's pointed at.
    ui::inset(fb, INFO);
    let mut y = INFO.y + 4;
    let x0 = INFO.x + 5;
    let say = |fb: &mut Framebuffer, y: &mut i32, s: &str, col: u8| {
        for l in wrap(s, 41) {
            if *y < INFO.y + INFO.h - 8 {
                fb.text(&l, x0, *y, col, None, 1);
            }
            *y += 9;
        }
    };
    let foe_hover = tonight(g).into_iter().enumerate().find(|(k, _)| foe_icon_rect(*k).contains(g.mouse.0, g.mouse.1)).map(|(_, f)| f);
    if let Some(f) = foe_hover {
        let i = f.info();
        say(fb, &mut y, &format!("{}: {} HP {}, BITE {}.", i.name, i.text, i.hp, i.bite), TEXT);
        say(fb, &mut y, &format!("DEFEAT ONE TO LEARN {} ({}).", i.gene.name(), i.gene.loc().name()), GOLD);
    } else if g.lesson && g.lesson_step < LESSON_STEPS.len() {
        say(fb, &mut y, LESSON_STEPS[g.lesson_step], ACCENT_HI);
    } else if let Some(i) = g.hover_item.filter(|&i| i < g.roster.len()) {
        let f = &g.roster[i];
        say(fb, &mut y, f.body.info().note, TEXT_DIM);
        for gene in &f.genes {
            say(fb, &mut y, &format!("{}: {}", gene.name(), gene.text()), TEXT);
        }
    } else if let Some(m) = &g.message {
        say(fb, &mut y, m, ACCENT_HI);
    } else {
        let lib = format!("YOUR GENES: {}", g.library.iter().map(|g| g.name()).collect::<Vec<_>>().join(", "));
        say(fb, &mut y, &lib, TEXT_DIM);
    }
    // Tonight's foes.
    ui::panel(fb, FOES_BOX);
    fb.text("TONIGHT", FOES_BOX.x + 6, FOES_BOX.y + 5, BAD, Some(OUTLINE), 1);
    for (k, f) in tonight(g).iter().enumerate() {
        let spr = g.art.foe(*f);
        let r = foe_icon_rect(k);
        if r.contains(g.mouse.0, g.mouse.1) {
            fb.rect_fill(r.x, r.y, r.w, r.h, PANEL_HI);
        }
        fb.blit(spr, r.x + (r.w - spr.w) / 2, r.y + r.h - spr.h, true);
    }
    fb.text(&format!("{} OF THEM", night_size(g.night) + (g.night == NIGHTS) as usize), FOES_BOX.x + 6, FOES_BOX.y + FOES_BOX.h - 11, TEXT_DIM, None, 1);
}

fn tonight(g: &Game) -> Vec<Foe> {
    let mut v = night_foes(g.night);
    if g.night == NIGHTS {
        v.insert(0, Foe::Grof);
    }
    v.truncate(8);
    v
}

fn foe_icon_rect(k: usize) -> Rect {
    Rect::new(FOES_BOX.x + 4 + (k % 4) as i32 * 30, FOES_BOX.y + 16 + (k / 4) as i32 * 30, 28, 28)
}

fn draw_dialog(g: &Game, fb: &mut Framebuffer) {
    let Some(d) = &g.dialog else { return };
    fb.rect_checker(0, 0, 400, 300, OUTLINE, 0);
    let opts = g.options();
    let r = Rect::new(30, 24, 340, 40 + opts.len() as i32 * 34 + 20);
    ui::panel(fb, r);
    let head = match d {
        Dialog::Body => "HATCH: CHOOSE A BODY".to_string(),
        Dialog::Gene { body } => format!("HATCH A {}: ITS FIRST GENE ({})", body.info().name, body.info().locs[0].name()),
        Dialog::Grow { fam } => {
            let f = &g.roster[*fam];
            format!("{} GROWS: A {} GENE", f.name, f.next_loc().map_or("", |l| l.name()))
        }
        Dialog::Absorb { .. } => "ABSORB A GENE FROM WHAT YOU DEFEATED".to_string(),
    };
    fb.text_centered(&head, 200, 32, ACCENT_HI, Some(OUTLINE), 1);
    if g.lesson && g.lesson_step < LESSON_STEPS.len() && matches!(d, Dialog::Body | Dialog::Gene { .. }) {
        for (k, l) in wrap(LESSON_STEPS[g.lesson_step], 54).iter().take(2).enumerate() {
            fb.text(l, 40, 42 + k as i32 * 9, TEXT, None, 1);
        }
    }
    for (i, _) in opts.iter().enumerate() {
        let o = option_rect(i);
        let hover = g.hover_item == Some(i);
        if hover {
            fb.rect_bevel(o.x, o.y, o.w, o.h, PANEL_HI, WHITE, PANEL);
        } else {
            ui::inset(fb, o);
        }
        let (tc, dc) = if hover { (PANEL_DEEP, PANEL_DEEP) } else { (ACCENT_HI, TEXT_DIM) };
        match d {
            Dialog::Body => {
                let b = ALL_BODIES[i];
                let info = b.info();
                draw_familiar(g, fb, b, &[], o.x + 18, o.y + o.h - 3, false);
                fb.text(info.name, o.x + 40, o.y + 4, tc, None, 1);
                let locs: Vec<&str> = info.locs.iter().map(|l| l.name()).collect();
                fb.text(&format!("HP {} STEP {} BITE {}  GENES: {}", info.hp, info.step, info.bite, locs.join(" ")), o.x + 40, o.y + 13, dc, None, 1);
                fb.text(info.note, o.x + 40, o.y + 21, dc, None, 1);
            }
            _ => {
                let gene = match d {
                    Dialog::Gene { body } => g.genes_for(body.info().locs[0])[i],
                    Dialog::Grow { fam } => g.genes_for(g.roster[*fam].next_loc().unwrap())[i],
                    Dialog::Absorb { genes } => genes[i],
                    Dialog::Body => unreachable!(),
                };
                fb.text(&format!("{} ({})", gene.name(), gene.loc().name()), o.x + 8, o.y + 4, tc, None, 1);
                for (k, l) in wrap(gene.text(), 52).iter().take(2).enumerate() {
                    fb.text(l, o.x + 8, o.y + 13 + k as i32 * 8, dc, None, 1);
                }
            }
        }
    }
    let hint = match d {
        Dialog::Absorb { .. } => "CLICK ONE TO LEARN IT.",
        _ if g.lesson => "CLICK ONE.",
        _ => "CLICK ONE. CLICK OUTSIDE (OR ESC) TO CANCEL.",
    };
    fb.text_centered(hint, 200, r.y + r.h - 12, TEXT_FAINT, None, 1);
}

fn tile_xy(p: Pos) -> (i32, i32) {
    (AX + p.0 * AT, AY + p.1 * AT)
}

fn draw_pond(g: &Game, fb: &mut Framebuffer) {
    top_bar(g, fb);
    let water: Vec<Pos> = g.battle.as_ref().map(|b| b.water.clone()).unwrap_or_default();
    fb.rect_fill(AX - 2, AY - 2, W * AT + 4, H * AT + 4, OUTLINE);
    for y in 0..H {
        for x in 0..W {
            let (tx, ty) = tile_xy((x, y));
            if water.contains(&(x, y)) {
                fb.rect_fill(tx, ty, AT, AT, WATER + 1);
                let ph = ((g.frame / 8) as i32 + x * 3 + y * 5) % 7;
                fb.rect_fill(tx + 4 + ph * 3, ty + 8, 4, 1, WATER + 2);
            } else {
                fb.rect_fill(tx, ty, AT, AT, if (x + y) % 2 == 0 { GROUND_L } else { GROUND_D });
                if (x * 7 + y * 3) % 5 == 0 {
                    fb.pset(tx + 6, ty + 18, GRASS);
                    fb.pset(tx + 7, ty + 17, GRASS);
                }
            }
        }
    }
    // The nest with its eggs.
    let eggs = g.battle.as_ref().map_or(g.eggs, |b| b.eggs);
    let (nx, ny) = tile_xy(NEST[0]);
    fb.rect_fill(nx + 2, ny + 4, AT - 4, 2 * AT - 8, TWIGS);
    fb.rect_outline(nx + 2, ny + 4, AT - 4, 2 * AT - 8, OUTLINE);
    fb.rect_checker(nx + 3, ny + 5, AT - 6, 2 * AT - 10, TWIGS + 1, 0);
    for k in 0..eggs.min(12) {
        draw_egg(fb, nx + 4 + (k % 3) * 7, ny + 8 + (k / 3) * 9);
    }
    let deploying = g.phase == Phase::Deploy;
    if deploying {
        for p in deploy_squares() {
            let (tx, ty) = tile_xy(p);
            fb.rect_checker(tx + 1, ty + 1, AT - 2, AT - 2, LIT, 0);
        }
        for &(i, p) in &g.posts {
            if g.drag == Some(i) {
                continue;
            }
            let (tx, ty) = tile_xy(p);
            let f = &g.roster[i];
            draw_familiar(g, fb, f.body, &f.genes, tx + AT / 2, ty + AT - 3, false);
        }
        // Foes to come, waiting at the right edge.
        let kinds = night_foes(g.night);
        for (k, f) in kinds.iter().enumerate().take(H as usize) {
            let (tx, ty) = tile_xy((W - 1, k as i32));
            let spr = g.art.foe(*f);
            fb.blit_map(spr, tx + (AT - spr.w) / 2, ty + AT - 3 - spr.h, true, &art::flat_map(PANEL_LO));
        }
    }
    if let Some(b) = &g.battle {
        for u in b.units.iter().filter(|u| u.alive) {
            let (a, c) = (tile_xy(u.moved_from), tile_xy(u.pos));
            let n = u.step.max(1);
            let k = (b.tick.saturating_sub(u.moved_at) as i32).min(n);
            let (x, y) = (a.0 + (c.0 - a.0) * k / n, a.1 + (c.1 - a.1) * k / n);
            match (u.body, u.foe) {
                (Some(body), _) => {
                    if u.buried {
                        fb.rect_fill(x + 6, y + AT - 8, AT - 12, 4, MUD);
                        fb.text("~", x + 10, y + AT - 14, MUD, None, 1);
                    } else {
                        draw_familiar(g, fb, body, &u.genes, x + AT / 2, y + AT - 3, false);
                    }
                }
                (None, Some(f)) => {
                    let spr = g.art.foe(f);
                    fb.blit(spr, x + (AT - spr.w) / 2, y + AT - 3 - spr.h, true);
                }
                (None, None) => {
                    let spr = &g.art.tadpole;
                    fb.blit(spr, x + (AT - spr.w) / 2, y + AT - 3 - spr.h, false);
                }
            }
            if u.buried {
                continue;
            }
            let w = AT - 6;
            fb.rect_fill(x + 3, y, w, 3, OUTLINE);
            let fill = (u.hp.max(0) * (w - 2) + u.max - 1) / u.max.max(1);
            fb.rect_fill(x + 4, y + 1, fill.min(w - 2), 1, if u.side == Side::Ours { GOOD } else { BAD });
            if u.poison > 0 {
                fb.pset(x + AT - 4, y + 5, SNAKE_R + 2);
            }
        }
        for (ft, fx) in &b.fx {
            let age = b.tick.saturating_sub(*ft) as i32;
            let centre = |p: Pos| (AX + p.0 * AT + AT / 2, AY + p.1 * AT + AT / 2);
            match fx {
                Fx::Bite { from, to, dmg } if age <= 4 => {
                    let (a, c) = (centre(*from), centre(*to));
                    if a != c && age <= 1 {
                        fb.line(a.0, a.1, c.0, c.1, TEXT_FAINT);
                    }
                    fb.text(&format!("-{dmg}"), c.0 - 5, c.1 - 16 - age, WHITE, Some(OUTLINE), 1);
                }
                Fx::Word { at, text } if age <= 8 => {
                    let c = centre(*at);
                    let w = Framebuffer::text_width(text, 1);
                    fb.text(text, (c.0 - w / 2).clamp(2, 398 - w), c.1 - 22 - age, GOLD, Some(OUTLINE), 1);
                }
                Fx::Fire { squares } if age <= 3 => {
                    for q in squares {
                        let (tx, ty) = tile_xy(*q);
                        fb.rect_checker(tx + 2, ty + 2, AT - 4, AT - 4, NEWT_R + 1, age);
                    }
                }
                Fx::Died { at, .. } if age <= 6 => {
                    let c = centre(*at);
                    fb.circle(c.0, c.1, 2 + age * 2, TEXT_DIM);
                }
                Fx::Egg { at } if age <= 8 => {
                    let c = centre(*at);
                    fb.text("EGG TAKEN!", (c.0 - 30).max(2), c.1 - 22 - age, BAD, Some(OUTLINE), 1);
                }
                Fx::Heal { at } if age <= 4 => {
                    let c = centre(*at);
                    fb.text("+", c.0 + 6, c.1 - 12 - age, GOOD, Some(OUTLINE), 1);
                }
                Fx::Chorus { at } if age <= 4 => {
                    let c = centre(*at);
                    fb.circle(c.0, c.1, 6 + age * 6, GOLD);
                }
                _ => {}
            }
        }
    }

    // Bench and buttons.
    if deploying {
        fb.text("TO PLACE:", 6, 206, TEXT_DIM, None, 1);
        for (slot, i) in g.unplaced().into_iter().enumerate() {
            let r = bench_rect(slot);
            ui::inset(fb, r);
            if g.drag == Some(i) {
                continue;
            }
            let f = &g.roster[i];
            draw_familiar(g, fb, f.body, &f.genes, r.x + r.w / 2, r.y + r.h - 3, false);
        }
        ui::button(fb, START_BTN, "START", START_BTN.contains(g.mouse.0, g.mouse.1), !g.posts.is_empty());
        if let Some(i) = g.drag {
            let f = &g.roster[i];
            if let Some(p) = g.arena_square(g.mouse.0, g.mouse.1) {
                let (tx, ty) = tile_xy(p);
                fb.rect_outline(tx, ty, AT, AT, if deploy_squares().contains(&p) { WHITE } else { BAD });
            }
            draw_familiar(g, fb, f.body, &f.genes, g.mouse.0, g.mouse.1 + 8, false);
        }
    } else if g.phase == Phase::Night {
        ui::button(fb, SPEED_BTN, &format!("{}X", g.speed), SPEED_BTN.contains(g.mouse.0, g.mouse.1), true);
        ui::button(fb, SKIP_BTN, "SKIP", SKIP_BTN.contains(g.mouse.0, g.mouse.1), !g.lesson);
        if let Some(b) = &g.battle {
            fb.text(&format!("FOES LEFT {}", b.foes_left()), 6, 206, TEXT_DIM, None, 1);
            let names: Vec<String> = b.units.iter().filter(|u| u.side == Side::Ours && u.fam.is_some()).map(|u| format!("{} {}", u.name, if u.alive { u.hp.to_string() } else { "X".into() })).collect();
            for (k, l) in wrap(&names.join("  "), 42).iter().take(2).enumerate() {
                fb.text(l, 6, 216 + k as i32 * 9, TEXT, None, 1);
            }
        }
    }

    // Message bar.
    let bar = Rect::new(2, 240, 396, 58);
    ui::inset(fb, bar);
    let mut y = 244;
    // Point at a creature on the pond for what it is.
    let pointed = g.arena_square(g.mouse.0, g.mouse.1).and_then(|p| {
        if let Some(b) = &g.battle {
            b.units.iter().find(|u| u.alive && u.pos == p && !u.buried).map(|u| match u.foe {
                Some(f) => format!("{}: {} HP {}/{}.", f.info().name, f.info().text, u.hp, u.max),
                None if u.genes.is_empty() => format!("{}: A FRESH HATCHLING. HP {}/{}.", u.name, u.hp, u.max),
                None => format!("{}: {} HP {}/{}.", u.name, u.genes.iter().map(|g| g.name()).collect::<Vec<_>>().join(", "), u.hp, u.max),
            })
        } else {
            g.posts.iter().find(|(_, q)| *q == p).map(|(i, _)| {
                let f = &g.roster[*i];
                format!("{} THE {}: {}", f.name, f.body.info().name, f.genes.iter().map(|g| format!("{} ({})", g.name(), g.text())).collect::<Vec<_>>().join(" "))
            })
        }
    });
    let text = g.note.clone().or(pointed).or_else(|| if g.lesson && g.phase == Phase::Deploy { Some(LESSON_STEPS[5].to_string()) } else { g.message.clone() });
    if let Some(t) = text {
        for l in wrap(&t, 64).iter().take(4) {
            fb.text(l, 8, y, TEXT, None, 1);
            y += 9;
        }
    }
    let hint = if g.note.is_some() {
        "CLICK TO GO ON."
    } else {
        match g.phase {
            Phase::Deploy => "DRAG FAMILIARS ONTO LIT SQUARES. RIGHT-CLICK: BACK. F1: RULES",
            Phase::Night => "F OR THE BUTTON: SPEED.  F1: RULES",
            _ => "",
        }
    };
    fb.text(hint, 8, 287, if g.note.is_some() { ACCENT } else { TEXT_FAINT }, None, 1);

    if let Phase::Dawn { .. } = g.phase {
        let r = Rect::new(90, 70, 220, 80);
        ui::panel(fb, r);
        fb.text_centered("DAWN", 200, r.y + 8, GOLD, Some(OUTLINE), 2);
        let (eaten, defeated) = g.last_night.as_ref().map_or((0, 0), |l| (l.0, l.1.len()));
        fb.text_centered(&format!("{defeated} FOES DEFEATED, {eaten} EGGS TAKEN"), 200, r.y + 30, TEXT, None, 1);
        fb.text_centered("A NEW EGG IS LAID. CLICK TO GO ON.", 200, r.y + 44, TEXT_DIM, None, 1);
        fb.text_centered("SURVIVORS GAIN EXPERIENCE.", 200, r.y + 58, TEXT_DIM, None, 1);
    }
}

fn draw_over(g: &Game, fb: &mut Framebuffer, won: bool) {
    top_bar(g, fb);
    let r = Rect::new(30, 26, 340, 266);
    ui::panel(fb, r);
    let (head, sub) = if won {
        ("THE HATCHERY HOLDS", "GROF'S NIGHT IS OVER. THE STAR POND IS SAFE.".to_string())
    } else {
        ("THE LAST EGG IS GONE", format!("ON NIGHT {} OF {NIGHTS}.", g.night))
    };
    fb.text_centered(head, 200, 36, if won { GOLD } else { BAD }, Some(OUTLINE), 2);
    fb.text_centered(&sub, 200, 58, TEXT, None, 1);
    let s = &g.stats;
    fb.text_centered(&format!("HATCHED {}  FALLEN {}  FOES DEFEATED {}", s.hatched, s.fallen.len(), s.defeated), 200, 72, TEXT_DIM, None, 1);
    let mut y = 90;
    for f in &g.roster {
        draw_familiar(g, fb, f.body, &f.genes, 60, y + 22, false);
        fb.text(&format!("{} THE {} {}", f.name, f.stage(), f.body.info().name), 84, y + 2, TEXT, None, 1);
        let genes = f.genes.iter().map(|g| g.name()).collect::<Vec<_>>().join(", ");
        fb.text(&genes, 84, y + 12, TEXT_DIM, None, 1);
        y += 32;
    }
    if !s.fallen.is_empty() {
        fb.text(&format!("REMEMBERED: {}", s.fallen.join(", ")), 44, 248, TEXT_FAINT, None, 1);
    }
    let go = Rect::new(140, 264, 120, 20);
    ui::button(fb, go, "CONTINUE", go.contains(g.mouse.0, g.mouse.1), true);
}

fn draw_help(fb: &mut Framebuffer) {
    fb.rect_checker(0, 0, 400, 300, OUTLINE, 0);
    let r = Rect::new(12, 10, 376, 280);
    ui::panel(fb, r);
    fb.text_centered("HOW THE HATCHERY WORKS", 200, 18, ACCENT_HI, Some(OUTLINE), 1);
    let rules = [
        "EACH EGG IS A LIFE. LOSE THEM ALL AND THE RUN ENDS. A NEW EGG IS LAID EACH DAWN.",
        "HATCHING A FAMILIAR COSTS AN EGG: CHOOSE A BODY AND ITS FIRST GENE.",
        "A FAMILIAR GROWS GENES IN ITS BODY'S ORDER OF PLACES. EACH LEVEL OPENS THE NEXT PLACE, AND YOU CHOOSE THE GENE.",
        "SURVIVING A NIGHT GIVES 2 EXPERIENCE; EACH FOE IT FINISHES GIVES 1 MORE. FALLEN FAMILIARS ARE GONE.",
        "AFTER EACH NIGHT, LEARN ONE GENE FROM THE CREATURES YOU DEFEATED.",
        "AT NIGHT, PLACE YOUR FAMILIARS, THEN WATCH. FOES HEAD FOR THE NEST AND BITE WHAT STANDS NEXT TO THEM; FAMILIARS GO FOR THE NEAREST FOE.",
        "SURVIVE 8 NIGHTS. THE LAST ONE IS GROF'S.",
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
