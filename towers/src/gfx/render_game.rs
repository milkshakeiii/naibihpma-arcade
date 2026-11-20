use crate::content::challenges::{ChallengeDef, LessonStep};
use crate::content::maps::SkirmishMap;
use crate::gfx::sprites::{
    building_sprite, energy_icon, faction_icon, faction_ramp, terrain_tile, ui_icon, unit_sprite,
    UiIcon,
};
use crate::gfx::Framebuffer;
use crate::sim::defs::{building_cost, building_def, upgrade_cost};
use crate::sim::energy::{emitter_radius, has_line_of_sight, media_array_income};
use crate::sim::types::*;
use crate::sim::units::tile_center_px;
use crate::sim::GameState;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EnergyOverlayMode {
    Off = 0,
    Team = 1,
    Polarity = 2,
    All = 3,
}

impl EnergyOverlayMode {
    pub fn next(self) -> Self {
        match self {
            EnergyOverlayMode::Off => EnergyOverlayMode::Team,
            EnergyOverlayMode::Team => EnergyOverlayMode::Polarity,
            EnergyOverlayMode::Polarity => EnergyOverlayMode::All,
            EnergyOverlayMode::All => EnergyOverlayMode::Off,
        }
    }

    pub fn icon(self) -> UiIcon {
        match self {
            EnergyOverlayMode::Off => UiIcon::OverlayOff,
            EnergyOverlayMode::Team => UiIcon::OverlayGrid,
            EnergyOverlayMode::Polarity => UiIcon::OverlayPolarity,
            EnergyOverlayMode::All => UiIcon::OverlayAll,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ActiveTool {
    Select,
    Build(BuildingType),
    RelocateFrom(u8, u8),
    ShapeFrom(u8, u8),
}

pub struct BattleView<'a> {
    pub state: &'a GameState,
    pub active_faction_tab: usize,
    pub tool: ActiveTool,
    pub selected_tile: Option<(u8, u8)>,
    pub hover_tile: Option<(u8, u8)>,
    pub overlay: EnergyOverlayMode,
    pub paused: bool,
    pub fast_speed: bool,
    pub challenge: Option<&'a ChallengeDef>,
    pub lesson_step: Option<(usize, usize, LessonStep)>,
    pub lesson_reading: bool,
    pub status_msg: Option<(&'a str, u16)>,
    pub ui_tick: u32,
}

const ENERGY_ORDER: [EnergyMask; 5] = [
    EnergyMask::MECH,
    EnergyMask::COMPUTE,
    EnergyMask::MANA,
    EnergyMask::POS_FORCE,
    EnergyMask::NEG_FORCE,
];

fn count_energies(mask: EnergyMask) -> i32 {
    let mut c = 0;
    for &e in &ENERGY_ORDER {
        if mask.contains(e) {
            c += 1;
        }
    }
    c
}

fn draw_energy_icons(fb: &mut Framebuffer, mut x: i32, y: i32, mask: EnergyMask) -> i32 {
    for &e in &ENERGY_ORDER {
        if mask.contains(e) {
            let ic = energy_icon(e);
            fb.blit_sprite(x, y, 7, 7, &ic);
            x += 8;
        }
    }
    x
}

fn draw_stars(fb: &mut Framebuffer, x: i32, y: i32, earned: u8, max: u8) {
    for i in 0..max {
        let ic = ui_icon(if i < earned {
            UiIcon::StarFull
        } else {
            UiIcon::StarEmpty
        });
        fb.blit_sprite(x + (i as i32) * 8, y, 7, 7, &ic);
    }
}

fn slot_role_icon(slot: BuildingSlot) -> UiIcon {
    match slot {
        BuildingSlot::Infra => UiIcon::RoleInfra,
        BuildingSlot::Eco => UiIcon::RoleEco,
        BuildingSlot::Defense => UiIcon::RoleDefense,
        BuildingSlot::Factory => UiIcon::RoleFactory,
        BuildingSlot::Apex => UiIcon::RoleApex,
    }
}

fn blit_sprite_dither(
    fb: &mut Framebuffer,
    dx: i32,
    dy: i32,
    w: usize,
    h: usize,
    data: &[u8],
) {
    for sy in 0..h {
        let py = dy + sy as i32;
        if py < 0 || py >= crate::gfx::FB_H as i32 {
            continue;
        }
        for sx in 0..w {
            let px = dx + sx as i32;
            if px < 0 || px >= crate::gfx::FB_W as i32 || ((px + py) & 1) != 0 {
                continue;
            }
            let c = data[sy * w + sx];
            if c != 0 {
                fb.set_px(px, py, c);
            }
        }
    }
}

fn draw_lesson_tile_brackets(fb: &mut Framebuffer, gx: i32, gy: i32) {
    // Calm double gold brackets around guided placement tile
    let c_out = 43u8;
    let c_in = 42u8;
    // Outer corners (4px legs)
    fb.rect(gx - 1, gy - 1, 5, 1, c_out);
    fb.rect(gx - 1, gy - 1, 1, 5, c_out);
    fb.rect(gx + 12, gy - 1, 5, 1, c_out);
    fb.rect(gx + 16, gy - 1, 1, 5, c_out);
    fb.rect(gx - 1, gy + 16, 5, 1, c_out);
    fb.rect(gx - 1, gy + 12, 1, 5, c_out);
    fb.rect(gx + 12, gy + 16, 5, 1, c_out);
    fb.rect(gx + 16, gy + 12, 1, 5, c_out);
    fb.rect_outline(gx, gy, 16, 16, c_in);
}

fn energy_label_color(e: EnergyMask) -> (&'static str, u8) {
    if e == EnergyMask::MECH {
        ("MECH", 42)
    } else if e == EnergyMask::COMPUTE {
        ("COMPUTE", 27)
    } else if e == EnergyMask::MANA {
        ("MANA", 29)
    } else if e == EnergyMask::POS_FORCE {
        ("+FORCE", 42)
    } else {
        ("-FORCE", 51)
    }
}

fn draw_named_energies(fb: &mut Framebuffer, mut x: i32, y: i32, mask: EnergyMask) -> i32 {
    for &e in &ENERGY_ORDER {
        if mask.contains(e) {
            let (name, col) = energy_label_color(e);
            let ic = energy_icon(e);
            fb.blit_sprite(x, y, 7, 7, &ic);
            fb.text(x + 8, y, name, col, 1);
            x += 8 + (name.len() as i32) * 6 + 4;
        }
    }
    x
}

fn wrap_lines(text: &str, max_cols: usize) -> Vec<String> {
    let mut out = Vec::new();
    let mut cur = String::new();
    for word in text.split_whitespace() {
        if !cur.is_empty() && cur.len() + 1 + word.len() > max_cols {
            out.push(cur);
            cur = String::new();
        }
        if !cur.is_empty() {
            cur.push(' ');
        }
        cur.push_str(word);
    }
    if !cur.is_empty() {
        out.push(cur);
    }
    out
}

pub fn lesson_box_rect(step: &LessonStep, reading: bool) -> (i32, i32, i32, i32) {
    if !reading {
        let bw = 176;
        let bh = 34;
        let (bx, by) = if let Some((tx, ty)) = step.highlight_tile {
            let x = if tx <= 11 { 218 } else { 6 };
            let y = if ty <= 6 {
                PLAYFIELD_Y + PLAYFIELD_H - bh - 4
            } else {
                PLAYFIELD_Y + 4
            };
            (x, y)
        } else {
            (218, PLAYFIELD_Y + 4)
        };
        return (bx, by, bw, bh);
    }

    let bw = 202;
    let body_lines = wrap_lines(step.body, 31);
    let act_lines = wrap_lines(step.action_hint, 30);
    let key_h = if step.icon_mask != EnergyMask::NONE { 14 } else { 0 };
    let body_h = (body_lines.len() as i32) * 9 + 6;
    let act_h = (act_lines.len().max(1) as i32) * 9 + 6;
    let bh = (15 + key_h + body_h + act_h + 23).min(PLAYFIELD_H - 8);

    let bx = if let Some((tx, _)) = step.highlight_tile {
        if tx <= 10 { 192 } else { 4 }
    } else {
        99
    };
    let by = if let Some((_, ty)) = step.highlight_tile {
        let center_y = PLAYFIELD_Y + (ty as i32) * TILE_PX + 8;
        (center_y - bh / 2).clamp(PLAYFIELD_Y + 4, PLAYFIELD_Y + PLAYFIELD_H - bh - 4)
    } else {
        PLAYFIELD_Y + (PLAYFIELD_H - bh) / 2
    };
    (bx, by, bw, bh)
}

pub fn lesson_continue_btn_rect(step: &LessonStep) -> (i32, i32, i32, i32) {
    let (bx, by, bw, bh) = lesson_box_rect(step, true);
    (bx + 10, by + bh - 20, bw - 20, 15)
}

fn rects_overlap(ax: i32, ay: i32, aw: i32, ah: i32, bx: i32, by: i32, bw: i32, bh: i32) -> bool {
    ax < bx + bw && ax + aw > bx && ay < by + bh && ay + ah > by
}

fn draw_lesson_callout(fb: &mut Framebuffer, step_i: usize, total_n: usize, step: &LessonStep, reading: bool) {
    if !reading {
        let (bx, by, bw, bh) = lesson_box_rect(step, false);
        fb.rect_bevel(bx, by, bw, bh, 16, 42, 1);
        fb.blit_sprite(bx + 4, by + 3, 7, 7, &ui_icon(UiIcon::Book));
        let hdr = format!("{}/{}: {}", step_i + 1, total_n, step.title);
        let clipped = if hdr.len() > 23 { &hdr[..23] } else { &hdr };
        fb.text(bx + 13, by + 3, clipped, 43, 1);
        fb.text(bx + bw - 20, by + 3, "[?]", 42, 1);
        let act_lines = wrap_lines(step.action_hint, 27);
        for (i, line) in act_lines.iter().take(2).enumerate() {
            fb.text(bx + 6, by + 14 + (i as i32) * 9, line, 34, 1);
        }
        return;
    }

    let (bx, by, bw, bh) = lesson_box_rect(step, true);

    // Draw clean elbow leader line connecting box to highlighted tile brackets
    if let Some((tx, ty)) = step.highlight_tile {
        let t_left = (tx as i32) * TILE_PX - 1;
        let t_right = t_left + 18;
        let t_cy = PLAYFIELD_Y + (ty as i32) * TILE_PX + 8;
        let start_y = t_cy.clamp(by + 16, by + bh - 24);
        let box_x = if bx > t_right { bx } else { bx + bw };
        let edge_x = if bx > t_right { t_right } else { t_left };
        let mid_x = (box_x + edge_x) / 2;
        fb.line_bresenham(box_x, start_y, mid_x, start_y, 43);
        fb.line_bresenham(mid_x, start_y, mid_x, t_cy, 43);
        fb.line_bresenham(mid_x, t_cy, edge_x, t_cy, 43);
    }

    fb.rect_bevel(bx, by, bw, bh, 16, 42, 1);
    fb.rect(bx + 2, by + 2, bw - 4, 13, 24);
    fb.blit_sprite(bx + 5, by + 5, 7, 7, &ui_icon(UiIcon::Book));
    let hdr = format!("{}/{}: {}", step_i + 1, total_n, step.title);
    let max_hdr = ((bw - 52) / 6).max(8) as usize;
    let clipped = if hdr.len() > max_hdr { &hdr[..max_hdr] } else { &hdr };
    fb.text_shadow(bx + 15, by + 5, clipped, 43, 1, 1);
    fb.text(bx + bw - 34, by + 5, "PAUSE", 42, 1);

    let mut cur_y = by + 17;
    if step.icon_mask != EnergyMask::NONE {
        fb.rect(bx + 5, cur_y - 1, bw - 10, 12, 1);
        draw_named_energies(fb, bx + 8, cur_y + 1, step.icon_mask);
        cur_y += 14;
    } else {
        cur_y += 2;
    }

    let body_lines = wrap_lines(step.body, 31);
    for line in &body_lines {
        fb.text(bx + 7, cur_y, line, 22, 1);
        cur_y += 9;
    }
    cur_y += 2;

    let act_lines = wrap_lines(step.action_hint, 30);
    let act_h = (act_lines.len().max(1) as i32) * 9 + 5;
    fb.rect(bx + 5, cur_y, bw - 10, act_h, 1);
    fb.rect_outline(bx + 5, cur_y, bw - 10, act_h, 34);
    for (i, line) in act_lines.iter().enumerate() {
        fb.text(bx + 9, cur_y + 3 + (i as i32) * 9, line, 34, 1);
    }

    let (cbx, cby, cbw, cbh) = lesson_continue_btn_rect(step);
    fb.rect_bevel(cbx, cby, cbw, cbh, 32, 34, 1);
    fb.text_centered(cbx + cbw / 2, cby + 4, "GOT IT - CONTINUE [SPACE]", 43, 1, 1);
}

fn draw_overlay_legend(
    fb: &mut Framebuffer,
    mode: EnergyOverlayMode,
    req: EnergyMask,
    box_rect: Option<(i32, i32, i32, i32)>,
) {
    if mode == EnergyOverlayMode::Off {
        return;
    }
    let items: &[(&str, u8)] = match mode {
        EnergyOverlayMode::Team => {
            if req == EnergyMask::MECH.union(EnergyMask::COMPUTE) {
                &[("BOTH", 34), ("MECH", 47), ("COMPUTE", 27)]
            } else if req == EnergyMask::MANA.union(EnergyMask::COMPUTE) {
                &[("BOTH", 34), ("MANA", 29), ("COMPUTE", 27)]
            } else if req == EnergyMask::MECH.union(EnergyMask::NEG_FORCE) {
                &[("BOTH", 34), ("MECH", 47), ("-FORCE", 50)]
            } else if req == EnergyMask::MECH.union(EnergyMask::POS_FORCE) {
                &[("BOTH", 34), ("MECH", 47), ("+FORCE", 42)]
            } else {
                &[("FULL", 34), ("PARTIAL", 27), ("[V] NEXT", 22)]
            }
        }
        EnergyOverlayMode::Polarity => &[("+FORCE", 42), ("-FORCE", 50), ("STORM", 41)],
        EnergyOverlayMode::All => &[("YOURS", 58), ("ENEMY", 62), ("BOTH", 42)],
        EnergyOverlayMode::Off => &[],
    };
    let mut lw = 8;
    for (lbl, _) in items.iter() {
        let swatch = if *lbl == "[V] NEXT" { 0 } else { 7 };
        lw += swatch + (lbl.len() as i32) * 6 + 6;
    }
    let lh = 11;
    let candidates = [
        (4, PLAYFIELD_Y + PLAYFIELD_H - lh - 2),
        (400 - lw - 4, PLAYFIELD_Y + PLAYFIELD_H - lh - 2),
        (4, PLAYFIELD_Y + 2),
        (400 - lw - 4, PLAYFIELD_Y + 2),
    ];
    let (mut lx, mut ly) = candidates[0];
    if let Some((bx, by, bw, bh)) = box_rect {
        for &(cx, cy) in &candidates {
            if !rects_overlap(cx, cy, lw, lh, bx, by, bw, bh) {
                lx = cx;
                ly = cy;
                break;
            }
        }
    }
    fb.rect(lx, ly, lw, lh, 1);
    fb.rect_outline(lx, ly, lw, lh, 20);
    let mut cx = lx + 4;
    for &(lbl, col) in items.iter() {
        if lbl != "[V] NEXT" {
            fb.rect(cx, ly + 3, 5, 5, col);
            cx += 7;
        }
        fb.text(cx, ly + 2, lbl, col, 1);
        cx += (lbl.len() as i32) * 6 + 6;
    }
}

fn count_inline_ley_conduits(state: &GameState, team: TeamId, tx: u8, ty: u8) -> i32 {
    let mut links = 0i32;
    for b in &state.buildings {
        if b.hp > 0 && b.owner == team && b.build_ticks_left == 0 && b.kind == BuildingType::ElvenLeyConduit {
            let dx = b.x as i32 - tx as i32;
            let dy = b.y as i32 - ty as i32;
            let d = dx.abs().max(dy.abs());
            if d > 0
                && d <= 7
                && (dx == 0 || dy == 0 || dx.abs() == dy.abs())
                && has_line_of_sight(&state.terrain, tx as i32, ty as i32, b.x as i32, b.y as i32)
            {
                links += 1;
            }
        }
    }
    links
}

pub fn render_battle(fb: &mut Framebuffer, view: &BattleView<'_>) {
    let s = view.state;
    fb.clear(1);

    // 1. TOP STATUS BAR (y = 0..14)
    fb.rect_bevel(0, 0, 400, 14, 17, 19, 16);
    let p1 = &s.teams[0];
    let mut lx = 4;
    for &f in p1.config.as_slice() {
        fb.blit_sprite(lx, 3, 7, 7, &faction_icon(f));
        lx += 8;
    }
    fb.blit_sprite(lx, 3, 7, 7, &ui_icon(UiIcon::Coin));
    lx += 9;
    let res_str = format!("{} (+{})", p1.resources, p1.last_cycle_income);
    fb.text_shadow(lx, 3, &res_str, 42, 1, 1);

    // Extra faction telemetry in top bar (Book icon + tier, Greed icon + bonus)
    let mut rx = lx + (res_str.len() as i32) * 6 + 6;
    if p1.config.has(Faction::Dwarves) {
        fb.blit_sprite(rx, 3, 7, 7, &ui_icon(UiIcon::Book));
        let kt = format!("{}", p1.knowledge_tier);
        fb.text_shadow(rx + 8, 3, &kt, 46, 1, 1);
        rx += 17;
    }
    if p1.config.has(Faction::Dracones) {
        let greed = ((p1.resources / 100) * 10).clamp(0, 50);
        if greed > 0 {
            fb.blit_sprite(rx, 3, 7, 7, &ui_icon(UiIcon::Greed));
            let gs = format!("+{}%", greed);
            fb.text_shadow(rx + 8, 3, &gs, 54, 1, 1);
        }
    }

    // Center clock & objective with crisp icons
    let secs = s.tick / TICKS_PER_SEC;
    let time_s = format!("{:02}:{:02}", secs / 60, secs % 60);
    fb.blit_sprite(174, 3, 7, 7, &ui_icon(UiIcon::Clock));
    fb.text_shadow(183, 3, &time_s, 22, 1, 1);

    let (obj_icon, obj_txt, obj_col) = if let Some(goal) = s.challenge_goal {
        match goal {
            ChallengeGoal::SurviveTicks(t) => (
                UiIcon::Shield,
                format!("{}/{}S", secs, t / TICKS_PER_SEC),
                34,
            ),
            ChallengeGoal::ReachResources(amt) => (
                UiIcon::Coin,
                format!("{}/{}", p1.resources, amt),
                42,
            ),
            ChallengeGoal::DestroyAllEnemyStructures => {
                let left = s.buildings.iter().filter(|b| b.hp > 0 && b.owner == TeamId::P2).count();
                (UiIcon::Crosshair, format!("{}", left), 62)
            }
            ChallengeGoal::DestroyEnemyCitadel => (
                UiIcon::Citadel,
                format!("{}/{}", s.citadel_hp(TeamId::P1), s.citadel_hp(TeamId::P2)),
                22,
            ),
        }
    } else {
        (
            UiIcon::Citadel,
            format!("{}/{}", s.citadel_hp(TeamId::P1), s.citadel_hp(TeamId::P2)),
            22,
        )
    };
    fb.blit_sprite(218, 3, 7, 7, &ui_icon(obj_icon));
    fb.text_shadow(227, 3, &obj_txt, obj_col, 1, 1);

    // Right status pills: [V] + overlay icon, Speed icon + label, ?
    fb.text(302, 3, "[V]", 29, 1);
    fb.blit_sprite(322, 3, 7, 7, &ui_icon(view.overlay.icon()));

    let (spd_ic, spd_lbl, spd_col) = if view.paused {
        (UiIcon::Pause, "HOLD", 47)
    } else if view.fast_speed {
        (UiIcon::FastForward, "2X", 31)
    } else {
        (UiIcon::Play, "1X", 34)
    };
    fb.blit_sprite(339, 3, 7, 7, &ui_icon(spd_ic));
    fb.text(348, 3, spd_lbl, spd_col, 1);
    fb.text(388, 3, "?", 42, 1);

    // 2. PLAYFIELD TERRAIN (y = 14..238)
    for ty in 0..MAP_H {
        for tx in 0..MAP_W {
            let tile = s.terrain[tile_idx(tx, ty)];
            let spr = terrain_tile(tile, tx, ty);
            fb.blit_sprite(
                (tx as i32) * TILE_PX,
                PLAYFIELD_Y + (ty as i32) * TILE_PX,
                16,
                16,
                &spr,
            );
        }
    }

    // 3. ALWAYS-VISIBLE FLUX STORM POLARITY CLASH INDICATORS + OPTIONAL OVERLAY + PLACEMENT TERRITORY PIPS
    for ty in 0..MAP_H {
        for tx in 0..MAP_W {
            let idx = tile_idx(tx, ty);
            let px = (tx as i32) * TILE_PX;
            let py = PLAYFIELD_Y + (ty as i32) * TILE_PX;

            // Always show active +Force vs -Force Flux Storm tiles cleanly
            if s.energy_grid.flux_storm[idx] {
                let c = if ((tx + ty) & 1) == 0 { 42 } else { 50 };
                fb.rect(px + 1, py + 1, 3, 1, c);
                fb.rect(px + 1, py + 1, 1, 3, c);
                fb.rect(px + 12, py + 14, 3, 1, 51);
                fb.rect(px + 14, py + 12, 1, 3, 51);
            }

            if view.overlay != EnergyOverlayMode::Off {
                let req = p1.config.required_energies();
                match view.overlay {
                    EnergyOverlayMode::Team => {
                        let cell = s.energy_grid.net[0][idx];
                        if cell.satisfies(req) {
                            fb.rect_dither_sparse(px + 1, py + 1, 14, 14, 34);
                        } else if req.contains(EnergyMask::MECH) && cell.mech > 0 {
                            fb.rect_dither_sparse(px + 1, py + 1, 14, 14, 47);
                        } else if req.contains(EnergyMask::COMPUTE) && cell.compute > 0 {
                            fb.rect_dither_sparse(px + 1, py + 1, 14, 14, 27);
                        } else if req.contains(EnergyMask::MANA) && cell.mana > 0 {
                            fb.rect_dither_sparse(px + 1, py + 1, 14, 14, 29);
                        } else if req.contains(EnergyMask::POS_FORCE) && cell.pos_force > 0 {
                            fb.rect_dither_sparse(px + 1, py + 1, 14, 14, 42);
                        } else if req.contains(EnergyMask::NEG_FORCE) && cell.neg_force > 0 {
                            fb.rect_dither_sparse(px + 1, py + 1, 14, 14, 50);
                        }
                    }
                    EnergyOverlayMode::Polarity => {
                        let p_pos = s.energy_grid.net[0][idx].pos_force + s.energy_grid.net[1][idx].pos_force;
                        let p_neg = s.energy_grid.net[0][idx].neg_force + s.energy_grid.net[1][idx].neg_force;
                        if s.energy_grid.flux_storm[idx] {
                            fb.rect_dither(px + 1, py + 1, 14, 14, 41, 0);
                        } else if p_pos > 0 {
                            fb.rect_dither_sparse(px + 1, py + 1, 14, 14, 42);
                        } else if p_neg > 0 {
                            fb.rect_dither_sparse(px + 1, py + 1, 14, 14, 50);
                        }
                    }
                    EnergyOverlayMode::All => {
                        let c0 = s.energy_grid.net[0][idx];
                        let c1 = s.energy_grid.net[1][idx];
                        let p0 = c0.mech > 0 || c0.compute > 0 || c0.mana > 0 || c0.pos_force > 0 || c0.neg_force > 0;
                        let p1e = c1.mech > 0 || c1.compute > 0 || c1.mana > 0 || c1.pos_force > 0 || c1.neg_force > 0;
                        if p0 && p1e {
                            fb.rect_dither_sparse(px + 1, py + 1, 14, 14, 42);
                        } else if p0 {
                            fb.rect_dither_sparse(px + 1, py + 1, 14, 14, 58);
                        } else if p1e {
                            fb.rect_dither_sparse(px + 1, py + 1, 14, 14, 62);
                        }
                    }
                    EnergyOverlayMode::Off => {}
                }
            } else if let ActiveTool::Build(kind) = view.tool {
                // Show valid build territory dots when placing buildings even if overlay is off!
                let ux = tx as u8;
                let uy = ty as u8;
                if s.building_grid[idx] == GRID_EMPTY
                    && s.terrain[idx] != Terrain::Chasm
                    && (s.terrain[idx] != Terrain::RaisedStone || kind.faction() == Some(Faction::Leones))
                    && s.is_in_build_territory(TeamId::P1, ux, uy)
                {
                    let def = building_def(kind);
                    let pwr = s.energy_grid.net[0][idx].satisfies(def.req_energy);
                    fb.set_px(px + 7, py + 7, if pwr { 34 } else { 47 });
                }
            }
        }
    }

    // 4. RELAY LINKS, LEY LINKS, TUNNEL BORE WARP NETWORKS & GIZMO BRIDGES
    for i in 0..s.buildings.len() {
        let a = &s.buildings[i];
        if a.hp <= 0 || a.build_ticks_left > 0 {
            continue;
        }
        let (ax, ay) = tile_center_px(a.x, a.y);

        // Amber MECH power links (Power Relay -> Media Array) & Blue COMPUTE targeting wires (Compute emitters -> Human turrets)
        if a.kind == BuildingType::HumanRelay {
            let rad = emitter_radius(&s.terrain, a);
            for b in &s.buildings {
                if b.hp > 0
                    && b.build_ticks_left == 0
                    && b.owner == a.owner
                    && b.kind == BuildingType::HumanMediaArray
                    && chebyshev(a.x, a.y, b.x, b.y) <= rad
                {
                    let (bx, by) = tile_center_px(b.x, b.y);
                    fb.line_dashed(ax, PLAYFIELD_Y + ay, bx, PLAYFIELD_Y + by, 47, (s.tick / 3) as i32);
                }
            }
        }
        if a.powered && building_def(a.kind).emit_energy.contains(EnergyMask::COMPUTE) {
            let rad = emitter_radius(&s.terrain, a);
            for b in &s.buildings {
                if b.hp > 0
                    && b.build_ticks_left == 0
                    && b.owner == a.owner
                    && matches!(
                        b.kind,
                        BuildingType::HumanAutocannon | BuildingType::HumanOrbitalUplink
                    )
                    && chebyshev(a.x, a.y, b.x, b.y) <= rad
                {
                    let (bx, by) = tile_center_px(b.x, b.y);
                    fb.line_dashed(
                        ax,
                        PLAYFIELD_Y + ay,
                        bx,
                        PLAYFIELD_Y + by,
                        if a.owner == TeamId::P1 { 27 } else { 47 },
                        (s.tick / 3) as i32,
                    );
                }
            }
        }

        // Gnome Tinker Lab orthogonal Gizmo connectors
        if a.kind == BuildingType::GnomeTinkerLab && a.powered {
            for (dx, dy) in [(1i32, 0), (-1, 0), (0, 1), (0, -1)] {
                let nx = a.x as i32 + dx;
                let ny = a.y as i32 + dy;
                if nx >= 0 && nx < MAP_W as i32 && ny >= 0 && ny < MAP_H as i32 {
                    let g = s.building_grid[tile_idx(nx as usize, ny as usize)];
                    if g != GRID_EMPTY {
                        let ob = &s.buildings[g as usize];
                        if ob.hp > 0 && ob.build_ticks_left == 0 && ob.owner == a.owner {
                            let (bx, by) = tile_center_px(ob.x, ob.y);
                            let mx = (ax + bx) / 2;
                            let my = PLAYFIELD_Y + (ay + by) / 2;
                            fb.rect(mx - 1, my - 1, 3, 3, 35);
                        }
                    }
                }
            }
        }
    }

    // Dwarven Tunnel Bore Warp Lines (connect powered Bores to forward-most Bore per team)
    for ti in 0..2 {
        let owner = TeamId::from_index(ti);
        let goal = s
            .buildings
            .iter()
            .find(|b| b.hp > 0 && b.owner == owner.other() && b.kind == BuildingType::Citadel)
            .map_or(if owner == TeamId::P1 { (22u8, 7u8) } else { (2u8, 7u8) }, |c| (c.x, c.y));
        let mut front_bore: Option<(u8, u8, u8)> = None;
        for b in &s.buildings {
            if b.hp > 0
                && b.build_ticks_left == 0
                && b.powered
                && b.owner == owner
                && b.kind == BuildingType::DwarfTunnelBore
            {
                let d = chebyshev(b.x, b.y, goal.0, goal.1);
                if front_bore.map_or(true, |fb_best| d < fb_best.2) {
                    front_bore = Some((b.x, b.y, d));
                }
            }
        }
        if let Some((fx, fy, _)) = front_bore {
            let (fxp, fyp) = tile_center_px(fx, fy);
            for b in &s.buildings {
                if b.hp > 0
                    && b.build_ticks_left == 0
                    && b.powered
                    && b.owner == owner
                    && b.kind == BuildingType::DwarfTunnelBore
                    && (b.x != fx || b.y != fy)
                {
                    let (bxp, byp) = tile_center_px(b.x, b.y);
                    fb.line_dashed(bxp, PLAYFIELD_Y + byp, fxp, PLAYFIELD_Y + fyp, 46, (s.tick / 4) as i32);
                }
            }
        }
    }

    for link in &s.energy_grid.ley_links {
        let (x0, y0) = tile_center_px(link.x0, link.y0);
        let (x1, y1) = tile_center_px(link.x1, link.y1);
        let col = if link.owner == TeamId::P1 { 29 } else { 51 };
        fb.line_bresenham(x0, PLAYFIELD_Y + y0, x1, PLAYFIELD_Y + y1, col);
    }

    // 5. BUILDINGS
    for b in &s.buildings {
        if b.hp <= 0 {
            continue;
        }
        let px = (b.x as i32) * TILE_PX;
        let py = PLAYFIELD_Y + (b.y as i32) * TILE_PX;

        // Grounded drop shadow strip along bottom edge
        fb.rect_dither(px + 1, py + 14, 14, 2, 1, 0);

        let spr = building_sprite(b.kind, b.owner, s.tick, b.powered && b.build_ticks_left == 0);
        fb.blit_sprite(px, py, 16, 16, &spr);

        // Walker relocation trajectory
        if let Some((tx, ty)) = b.relocate_target {
            let (sx_px, sy_px) = tile_center_px(b.x, b.y);
            let (tx_px, ty_px) = tile_center_px(tx, ty);
            fb.line_dashed(sx_px, PLAYFIELD_Y + sy_px, tx_px, PLAYFIELD_Y + ty_px, 31, (s.tick / 2) as i32);
            fb.rect_outline((tx as i32) * TILE_PX + 2, PLAYFIELD_Y + (ty as i32) * TILE_PX + 2, 12, 12, 31);
        }

        if b.build_ticks_left > 0 {
            // Dithered construction scaffold overlay + progress bar
            fb.rect_dither(px, py, 16, 16, 47, 1);
            let total = building_def(b.kind).build_ticks.max(1) as i32;
            let done = total - (b.build_ticks_left as i32);
            let fill = ((done * 12) / total).clamp(1, 12);
            fb.rect(px + 2, py + 13, 12, 2, 1);
            fb.rect(px + 2, py + 13, fill, 2, 42);
        } else {
            let def = building_def(b.kind);
            // Mk.II Gold Badge (top-right)
            if b.level >= 2 {
                fb.rect(px + 12, py, 4, 4, 1);
                fb.rect(px + 13, py + 1, 2, 2, 42);
            }
            // Dwarf Hearth Armor Shield Pip (bottom-right-ish or top-mid-right)
            if b.hearth_armor > 0 {
                fb.rect(px + 12, py + 4, 4, 4, 1);
                fb.rect(px + 13, py + 5, 2, 2, 46);
            }
            // Gizmo buff spark pip (bottom-right)
            if b.gizmo_buff {
                fb.rect(px + 13, py + 12, 3, 3, 1);
                fb.set_px(px + 14, py + 13, 35);
            }
            // Tinker Bot slow debuff pip (top-left below banner)
            if b.slow_ticks > 0 {
                fb.rect(px, py + 4, 3, 3, 1);
                fb.set_px(px + 1, py + 5, 31);
            }
            // Fury pips for Doom Cauldron
            for f in 0..b.fury_stacks {
                fb.rect(px + 1 + (f as i32) * 3, py + 11, 2, 2, 41);
            }
            // Factory / Spawner Charge Bar along bottom row
            if b.powered && def.spawn_interval > 0 {
                if b.kind == BuildingType::DraconeElderSanctum && s.teams[b.owner.index()].resources < 150 {
                    fb.rect(px + 2, py + 14, 12, 2, 1);
                    fb.rect(px + 2, py + 14, 4, 1, 47);
                } else {
                    let done = def.spawn_interval.saturating_sub(b.spawn_timer) as i32;
                    let fill = ((done * 12) / (def.spawn_interval as i32)).clamp(1, 12);
                    fb.rect(px + 2, py + 14, 12, 2, 1);
                    fb.rect(px + 2, py + 14, fill, 1, 31);
                }
            }
            // Static BoltOff icon when unpowered
            if !b.powered {
                fb.blit_sprite(px + 9, py + 9, 7, 7, &ui_icon(UiIcon::BoltOff));
            }
        }

        // Compact HP bar when damaged
        if b.hp < b.max_hp {
            let w = ((b.hp * 14) / b.max_hp.max(1)).clamp(1, 14);
            fb.rect(px + 1, py - 2, 14, 2, 1);
            fb.rect(px + 1, py - 2, w, 1, if b.owner == TeamId::P1 { 34 } else { 61 });
        }
    }

    // 6. UNITS
    for u in &s.units {
        if u.hp <= 0 {
            continue;
        }
        let fly = u.kind.is_flying();
        let spr = unit_sprite(u.kind, u.owner, s.tick, u.hit_flash);
        let draw_x = u.px - 5;
        let draw_y = PLAYFIELD_Y + u.py - if fly { 8 } else { 5 };
        if fly {
            // 2x2 dithered ground shadow directly underneath
            fb.rect_dither(u.px - 4, PLAYFIELD_Y + u.py + 3, 8, 3, 1, 0);
        }
        fb.blit_sprite(draw_x, draw_y, 10, 10, &spr);

        // Corrosion / Slow status pips
        if u.corrode_ticks > 0 {
            fb.rect(draw_x, draw_y, 2, 2, 51);
        }
        if u.slow_ticks > 0 {
            fb.rect(draw_x + 8, draw_y, 2, 2, 31);
        }

        if u.hp < u.max_hp {
            let w = ((u.hp * 8) / u.max_hp.max(1)).clamp(1, 8);
            fb.rect(u.px - 4, draw_y - 2, 8, 2, 1);
            fb.rect(u.px - 4, draw_y - 2, w, 1, if u.owner == TeamId::P1 { 34 } else { 61 });
        }
    }

    // 7. PROJECTILES & VISUAL FX (Beams, Bursts, Popups)
    for p in &s.projectiles {
        let px = p.px;
        let py = PLAYFIELD_Y + p.py;
        let arc_off = if p.arcing {
            let half = (p.total_ticks as i32) / 2;
            let dist_mid = (p.ticks_left as i32 - half).abs();
            (half * half - dist_mid * dist_mid).max(0) / 2
        } else {
            0
        };
        fb.rect(px - 1, py - 1 - arc_off, 3, 3, 1);
        fb.set_px(px, py - arc_off, p.color);
    }

    for beam in &s.fx_beams {
        fb.line_bresenham(
            beam.x0 as i32,
            PLAYFIELD_Y + beam.y0 as i32,
            beam.x1 as i32,
            PLAYFIELD_Y + beam.y1 as i32,
            beam.color,
        );
    }

    for burst in &s.fx_bursts {
        fb.circle_int(
            burst.px as i32,
            PLAYFIELD_Y + burst.py as i32,
            burst.radius as i32,
            burst.color,
        );
    }

    for pop in &s.fx_popups {
        if let Ok(txt) = std::str::from_utf8(&pop.text[..pop.len as usize]) {
            fb.text_shadow(
                (pop.px as i32) - (txt.len() as i32) * 3,
                PLAYFIELD_Y + pop.py as i32,
                txt,
                pop.color,
                1,
                1,
            );
        }
    }

    // 8. SELECTION & PLACEMENT BLUEPRINT GHOST + LIVE SYNERGY PREVIEW
    if let Some((sx, sy)) = view.selected_tile {
        let gx = (sx as i32) * TILE_PX;
        let gy = PLAYFIELD_Y + (sy as i32) * TILE_PX;
        fb.rect_outline(gx, gy, 16, 16, 42);
        let occ = s.building_grid[tile_idx(sx as usize, sy as usize)];
        if occ != GRID_EMPTY {
            let b = &s.buildings[occ as usize];
            let def = building_def(b.kind);
            let r = if def.range_tiles > 0 && def.emit_energy == EnergyMask::NONE {
                def.range_tiles as i32 + if s.terrain[tile_idx(sx as usize, sy as usize)].is_high() { 1 } else { 0 }
            } else {
                emitter_radius(&s.terrain, b) as i32
            };
            let (cx, cy) = tile_center_px(sx, sy);
            if r > 0 {
                let r_col = if b.kind == BuildingType::HumanRelay {
                    47
                } else if b.kind == BuildingType::HumanMediaArray {
                    27
                } else if def.range_tiles > 0 {
                    42
                } else {
                    29
                };
                fb.circle_int(cx, PLAYFIELD_Y + cy, r * TILE_PX, r_col);
            }
            if def.min_range_tiles > 0 {
                fb.circle_int(cx, PLAYFIELD_Y + cy, (def.min_range_tiles as i32) * TILE_PX, 61);
            }
            if b.kind == BuildingType::LeoneSunMonolith {
                fb.circle_int(cx, PLAYFIELD_Y + cy, 3 * TILE_PX, 34);
            }
        }
    }

    // Guided Explanation Lesson Tile Brackets (calm gold outline + outer corners)
    if let Some((_, _, step)) = view.lesson_step {
        if let Some((lx, ly)) = step.highlight_tile {
            draw_lesson_tile_brackets(fb, (lx as i32) * TILE_PX, PLAYFIELD_Y + (ly as i32) * TILE_PX);
        }
    }

    if let Some((hx, hy)) = view.hover_tile {
        let gx = (hx as i32) * TILE_PX;
        let gy = PLAYFIELD_Y + (hy as i32) * TILE_PX;
        match view.tool {
            ActiveTool::Build(kind) => {
                let ok = s.can_execute(TeamId::P1, Action::Build { kind, x: hx, y: hy }).is_ok();
                let def = building_def(kind);
                let tidx = tile_idx(hx as usize, hy as usize);
                let powered_here = s.energy_grid.net[0][tidx].satisfies(def.req_energy);
                let col = if !ok {
                    61 // Red invalid
                } else if !powered_here && def.req_energy != EnergyMask::NONE {
                    47 // Amber warning
                } else {
                    34 // Green valid
                };

                // Dithered building silhouette ghost + crisp outline
                let spr = building_sprite(kind, TeamId::P1, view.ui_tick, powered_here || def.req_energy == EnergyMask::NONE);
                blit_sprite_dither(fb, gx, gy, 16, 16, &spr);
                fb.rect_outline(gx, gy, 16, 16, col);

                let (cx, cy) = tile_center_px(hx, hy);
                let r = if def.range_tiles > 0 {
                    def.range_tiles as i32 + if s.terrain[tidx].is_high() { 1 } else { 0 }
                } else {
                    def.emit_radius as i32
                };
                if r > 0 {
                    let dome_col = if col == 34 && kind == BuildingType::HumanRelay {
                        47
                    } else if col == 34 && kind == BuildingType::HumanMediaArray {
                        27
                    } else {
                        col
                    };
                    fb.circle_int(cx, PLAYFIELD_Y + cy, r * TILE_PX, dome_col);
                }
                if def.min_range_tiles > 0 {
                    fb.circle_int(cx, PLAYFIELD_Y + cy, (def.min_range_tiles as i32) * TILE_PX, 61);
                }
                if kind == BuildingType::LeoneSunMonolith {
                    fb.circle_int(cx, PLAYFIELD_Y + cy, 3 * TILE_PX, 34);
                }

                // Live Placement Synergy Previews
                match kind {
                    BuildingType::HumanRelay => {
                        for b in &s.buildings {
                            if b.hp > 0
                                && b.build_ticks_left == 0
                                && b.owner == TeamId::P1
                                && matches!(
                                    b.kind,
                                    BuildingType::HumanMediaArray
                                        | BuildingType::HumanMechFoundry
                                        | BuildingType::HumanAutocannon
                                        | BuildingType::HumanOrbitalUplink
                                )
                                && chebyshev(hx, hy, b.x, b.y) <= 4
                            {
                                let (bxp, byp) = tile_center_px(b.x, b.y);
                                fb.line_dashed(cx, PLAYFIELD_Y + cy, bxp, PLAYFIELD_Y + byp, 47, 0);
                            }
                        }
                    }
                    BuildingType::HumanAutocannon | BuildingType::HumanOrbitalUplink => {
                        for b in &s.buildings {
                            if b.hp > 0
                                && b.build_ticks_left == 0
                                && b.owner == TeamId::P1
                                && b.powered
                                && building_def(b.kind).emit_energy.contains(EnergyMask::COMPUTE)
                                && chebyshev(hx, hy, b.x, b.y) <= emitter_radius(&s.terrain, b)
                            {
                                let (bxp, byp) = tile_center_px(b.x, b.y);
                                fb.line_dashed(cx, PLAYFIELD_Y + cy, bxp, PLAYFIELD_Y + byp, 27, 0);
                            }
                        }
                    }
                    BuildingType::HumanMediaArray => {
                        let on_high = s.terrain[tidx].is_high();
                        let rad = 5i32 + if on_high { 1 } else { 0 };
                        for (dx, dy) in [(1i32, 0), (-1, 0), (0, 1), (0, -1), (1, 1), (1, -1), (-1, 1), (-1, -1)] {
                            let mut ex = hx as i32;
                            let mut ey = hy as i32;
                            for step in 1..=rad {
                                let nx = hx as i32 + dx * step;
                                let ny = hy as i32 + dy * step;
                                if nx < 0 || nx >= MAP_W as i32 || ny < 0 || ny >= MAP_H as i32 {
                                    break;
                                }
                                ex = nx;
                                ey = ny;
                                if !on_high && s.terrain[tile_idx(nx as usize, ny as usize)].blocks_line() {
                                    break;
                                }
                            }
                            let (epx, epy) = tile_center_px(ex as u8, ey as u8);
                            fb.line_dashed(cx, PLAYFIELD_Y + cy, epx, PLAYFIELD_Y + epy, 42, 0);
                        }
                        for b in &s.buildings {
                            if b.hp > 0
                                && b.build_ticks_left == 0
                                && b.owner == TeamId::P1
                                && matches!(
                                    b.kind,
                                    BuildingType::HumanAutocannon
                                        | BuildingType::HumanOrbitalUplink
                                        | BuildingType::HumanMechFoundry
                                )
                                && chebyshev(hx, hy, b.x, b.y) as i32 <= rad
                            {
                                let (bxp, byp) = tile_center_px(b.x, b.y);
                                fb.line_dashed(cx, PLAYFIELD_Y + cy, bxp, PLAYFIELD_Y + byp, 27, 0);
                            }
                        }
                    }
                    BuildingType::ElvenLeyConduit
                    | BuildingType::ElvenScriptorium
                    | BuildingType::ElvenPrismSpire
                    | BuildingType::ElvenWispSanctum
                    | BuildingType::ElvenAstralLens => {
                        for b in &s.buildings {
                            if b.hp > 0
                                && b.build_ticks_left == 0
                                && b.owner == TeamId::P1
                                && b.kind.faction() == Some(Faction::Elves)
                                && (kind == BuildingType::ElvenLeyConduit || b.kind == BuildingType::ElvenLeyConduit)
                            {
                                let dx = b.x as i32 - hx as i32;
                                let dy = b.y as i32 - hy as i32;
                                let d = dx.abs().max(dy.abs());
                                if d > 0
                                    && d <= 7
                                    && (dx == 0 || dy == 0 || dx.abs() == dy.abs())
                                    && has_line_of_sight(&s.terrain, hx as i32, hy as i32, b.x as i32, b.y as i32)
                                {
                                    let (bxp, byp) = tile_center_px(b.x, b.y);
                                    fb.line_dashed(cx, PLAYFIELD_Y + cy, bxp, PLAYFIELD_Y + byp, 30, 0);
                                }
                            }
                        }
                    }
                    BuildingType::DwarfRuneArchive => {
                        for oy in (hy as i32 - 1).max(0)..=(hy as i32 + 1).min(MAP_H as i32 - 1) {
                            for ox in (hx as i32 - 1).max(0)..=(hx as i32 + 1).min(MAP_W as i32 - 1) {
                                if s.terrain[tile_idx(ox as usize, oy as usize)] == Terrain::OreVein {
                                    let (opx, opy) = tile_center_px(ox as u8, oy as u8);
                                    fb.line_bresenham(cx, PLAYFIELD_Y + cy, opx, PLAYFIELD_Y + opy, 42);
                                }
                            }
                        }
                    }
                    BuildingType::DwarfRunicHearth => {
                        for oy in (hy as i32 - 1).max(0)..=(hy as i32 + 1).min(MAP_H as i32 - 1) {
                            for ox in (hx as i32 - 1).max(0)..=(hx as i32 + 1).min(MAP_W as i32 - 1) {
                                if ox != hx as i32 || oy != hy as i32 {
                                    fb.rect_outline(ox * TILE_PX + 1, PLAYFIELD_Y + oy * TILE_PX + 1, 14, 14, 46);
                                }
                            }
                        }
                    }
                    BuildingType::GnomeTinkerLab => {
                        for (dx, dy) in [(1i32, 0), (-1, 0), (0, 1), (0, -1)] {
                            let nx = hx as i32 + dx;
                            let ny = hy as i32 + dy;
                            if nx >= 0 && nx < MAP_W as i32 && ny >= 0 && ny < MAP_H as i32 {
                                fb.rect_outline(nx * TILE_PX + 1, PLAYFIELD_Y + ny * TILE_PX + 1, 14, 14, 35);
                            }
                        }
                    }
                    _ => {}
                }

                // Symbol-first placement hover pill
                let label_y = if gy > PLAYFIELD_Y + 12 { gy - 11 } else { gy + 18 };
                if !ok {
                    let cost = building_cost(kind, p1.config);
                    if p1.resources < cost {
                        let txt = format!("NEED {}", cost);
                        let w = 11 + (txt.len() as i32) * 6;
                        let lx = (gx - 6).clamp(2, 398 - w);
                        fb.rect(lx, label_y - 1, w, 9, 1);
                        fb.blit_sprite(lx + 1, label_y, 7, 7, &ui_icon(UiIcon::Coin));
                        fb.text(lx + 10, label_y, &txt, 61, 1);
                    } else {
                        let txt = "BLOCKED";
                        let w = 11 + (txt.len() as i32) * 6;
                        let lx = (gx - 6).clamp(2, 398 - w);
                        fb.rect(lx, label_y - 1, w, 9, 1);
                        fb.blit_sprite(lx + 1, label_y, 7, 7, &ui_icon(UiIcon::Cross));
                        fb.text(lx + 10, label_y, txt, 61, 1);
                    }
                } else if !powered_here && def.req_energy != EnergyMask::NONE {
                    let missing = EnergyMask(
                        def.req_energy.0
                            & !(EnergyMask(
                                (if s.energy_grid.net[0][tidx].mech > 0 { 1 } else { 0 })
                                    | (if s.energy_grid.net[0][tidx].compute > 0 { 2 } else { 0 })
                                    | (if s.energy_grid.net[0][tidx].mana > 0 { 4 } else { 0 })
                                    | (if s.energy_grid.net[0][tidx].pos_force > 0 { 8 } else { 0 })
                                    | (if s.energy_grid.net[0][tidx].neg_force > 0 { 16 } else { 0 }),
                            )
                            .0),
                    );
                    let m_cnt = count_energies(missing).max(1);
                    let msg = "NO POWER";
                    let w = 11 + (msg.len() as i32) * 6 + 3 + m_cnt * 8;
                    let lx = (gx - 10).clamp(2, 398 - w);
                    fb.rect(lx, label_y - 1, w, 9, 1);
                    fb.blit_sprite(lx + 1, label_y, 7, 7, &ui_icon(UiIcon::BoltOff));
                    fb.text(lx + 10, label_y, msg, 47, 1);
                    draw_energy_icons(fb, lx + 10 + (msg.len() as i32) * 6 + 3, label_y, missing);
                } else {
                    let compute_links = if matches!(
                        kind,
                        BuildingType::HumanAutocannon | BuildingType::HumanOrbitalUplink
                    ) {
                        s.buildings
                            .iter()
                            .filter(|b| {
                                b.hp > 0
                                    && b.build_ticks_left == 0
                                    && b.owner == TeamId::P1
                                    && b.powered
                                    && building_def(b.kind).emit_energy.contains(EnergyMask::COMPUTE)
                                    && chebyshev(hx, hy, b.x, b.y) <= emitter_radius(&s.terrain, b)
                            })
                            .count()
                            .min(4)
                    } else {
                        0
                    };
                    let foundry_compute = kind == BuildingType::HumanMechFoundry
                        && s.energy_grid.net[0][tidx].compute > 0;
                    let extra_txt = if kind == BuildingType::HumanMediaArray {
                        let dummy = Building {
                            id: 0, owner: TeamId::P1, kind, x: hx, y: hy, hp: 100, max_hp: 100,
                            armor: 0, level: 1, build_ticks_left: 0, cooldown: 0, spawn_timer: 0,
                            spawn_count: 0, powered: true, net_links: 0, gizmo_buff: false,
                            hearth_armor: 0, fury_stacks: 0, slow_ticks: 0, relocate_target: None,
                            relocate_timer: 0, ability_cd: 0, last_fire_tick: 0,
                        };
                        let inc = media_array_income(&s.terrain, &s.buildings, &dummy);
                        format!("+{}", inc)
                    } else if kind == BuildingType::ElvenScriptorium {
                        let links = count_inline_ley_conduits(s, TeamId::P1, hx, hy).min(3);
                        format!("+{}", 11 + links * 5)
                    } else if compute_links > 0 {
                        format!("+{}% FIRE", compute_links * 10)
                    } else if foundry_compute {
                        "+15% FAST".to_string()
                    } else {
                        format!("{}", building_cost(kind, p1.config))
                    };
                    let high_bonus = s.terrain[tidx].is_high() && def.range_tiles > 0;
                    let w = 11 + (extra_txt.len() as i32) * 6 + if high_bonus { 10 } else { 0 };
                    let lx = (gx - 6).clamp(2, 398 - w);
                    fb.rect(lx, label_y - 1, w, 9, 1);
                    let badge = if matches!(kind, BuildingType::HumanMediaArray | BuildingType::ElvenScriptorium) {
                        UiIcon::RoleEco
                    } else if compute_links > 0 || foundry_compute {
                        UiIcon::Link
                    } else {
                        UiIcon::Check
                    };
                    let txt_col = if compute_links > 0 || foundry_compute { 29 } else { col };
                    fb.blit_sprite(lx + 1, label_y, 7, 7, &ui_icon(badge));
                    fb.text(lx + 10, label_y, &extra_txt, txt_col, 1);
                    if high_bonus {
                        fb.blit_sprite(lx + 10 + (extra_txt.len() as i32) * 6 + 1, label_y, 7, 7, &ui_icon(UiIcon::Crosshair));
                    }
                }
            }
            ActiveTool::RelocateFrom(wx, wy) => {
                let ok = s.can_execute(TeamId::P1, Action::Relocate { x: wx, y: wy, tx: hx, ty: hy }).is_ok();
                let col = if ok { 31 } else { 61 };
                let (cx, cy) = tile_center_px(wx, wy);
                let (txp, typ) = tile_center_px(hx, hy);
                fb.line_dashed(cx, PLAYFIELD_Y + cy, txp, PLAYFIELD_Y + typ, col, (view.ui_tick / 2) as i32);
                fb.rect_outline(gx, gy, 16, 16, col);
            }
            ActiveTool::ShapeFrom(sx, sy) => {
                let ok = s.can_execute(TeamId::P1, Action::ShapeTerrain { shaper_x: sx, shaper_y: sy, tx: hx, ty: hy }).is_ok();
                let col = if ok { 42 } else { 61 };
                fb.rect_outline(gx, gy, 16, 16, col);
            }
            ActiveTool::Select => {
                fb.rect_outline(gx, gy, 16, 16, 21);
            }
        }
    }

    // Overlay Legend Pill + In-World Advisor Callout Box + Status Toast / Initial Briefing
    let active_box = view
        .lesson_step
        .map(|(_, _, st)| lesson_box_rect(&st, view.lesson_reading));
    draw_overlay_legend(
        fb,
        view.overlay,
        p1.config.required_energies(),
        active_box,
    );

    if let Some((step_i, total_n, step)) = view.lesson_step {
        draw_lesson_callout(fb, step_i, total_n, &step, view.lesson_reading);
    } else if let Some(ch) = view.challenge {
        if s.tick < 150 {
            fb.rect(8, 15, 384, 21, 1);
            fb.rect_outline(8, 15, 384, 21, 29);
            fb.text(12, 17, ch.briefing_1, 22, 1);
            fb.text(12, 26, ch.briefing_2, 27, 1);
        }
    }
    if let Some((msg, _)) = view.status_msg {
        fb.rect(40, 16, 320, 12, 1);
        fb.rect_outline(40, 16, 320, 12, 42);
        fb.text_centered(200, 18, msg, 42, 1, 1);
    }

    // 9. BOTTOM APOGEE / DUNE 2000 BEVELED HUD PANEL (y = 238..300)
    fb.rect_bevel(0, 238, 400, 62, 17, 20, 16);

    // Faction Tabs (x = 2..236, y = 240..253)
    let factions = p1.config.as_slice();
    let tab_idx = view.active_faction_tab % factions.len();
    let active_faction = factions[tab_idx];
    if factions.len() == 1 {
        fb.rect_bevel(2, 240, 236, 13, 18, 19, 16);
        fb.blit_sprite(6, 243, 7, 7, &faction_icon(active_faction));
        let hdr = format!("{}: {}", active_faction.name(), active_faction.resource_name());
        fb.text(15, 243, &hdr, 22, 1);
        let e_cnt = count_energies(active_faction.required_energies());
        draw_energy_icons(fb, 234 - e_cnt * 8, 243, active_faction.required_energies());
    } else {
        let tab_w = 234 / (factions.len() as i32);
        for (i, &f) in factions.iter().enumerate() {
            let tx = 2 + (i as i32) * tab_w;
            let sel = i == tab_idx;
            fb.rect_bevel(tx, 240, tab_w - 2, 13, if sel { 24 } else { 16 }, if sel { 42 } else { 19 }, 1);
            fb.blit_sprite(tx + 3, 243, 7, 7, &faction_icon(f));
            let key_tag = if i == 0 { "Q" } else if i == 1 { "E" } else { "T" };
            fb.text(tx + 12, 243, f.name(), if sel { 42 } else { 21 }, 1);
            fb.text(tx + tab_w - 10, 243, key_tag, if sel { 43 } else { 27 }, 1);
        }
    }

    // 5 Building Cards (x = 2 + i*47, y = 254, w = 45, h = 44)
    let guided_card = view.lesson_step.and_then(|(_, _, st)| st.highlight_card);
    for (i, slot) in BuildingSlot::ALL.iter().enumerate() {
        let kind = BuildingType::from_faction_slot(active_faction, *slot);
        let def = building_def(kind);
        let cost = building_cost(kind, p1.config);
        let locked = def.require_eco && !s.has_eco_building(TeamId::P1, active_faction);
        let affordable = p1.resources >= cost && !locked;
        let selected_card = view.tool == ActiveTool::Build(kind);
        let is_guided = guided_card == Some(i);

        let cx = 2 + (i as i32) * 47;
        let cy = 254;
        fb.rect_bevel(
            cx,
            cy,
            45,
            44,
            if selected_card { 24 } else { 16 },
            if selected_card || is_guided { 43 } else { 19 },
            1,
        );
        if is_guided {
            fb.rect_outline(cx + 1, cy + 1, 43, 42, 42);
        }
        // Key badge + 7x7 role icon
        let key_s = format!("{}", i + 1);
        fb.text(cx + 3, cy + 2, &key_s, 42, 1);
        fb.blit_sprite(cx + 35, cy + 2, 7, 7, &ui_icon(slot_role_icon(*slot)));

        // Centered 16x16 building sprite
        let spr = building_sprite(kind, TeamId::P1, view.ui_tick, affordable);
        fb.blit_sprite(cx + 15, cy + 5, 16, 16, &spr);

        // Energy symbol row (req -> emit for converters like Media Array, else + emit, else req)
        if def.req_energy != EnergyMask::NONE
            && def.emit_energy != EnergyMask::NONE
            && def.req_energy != def.emit_energy
        {
            let sx = cx + (45 - 19) / 2;
            fb.blit_sprite(sx, cy + 24, 7, 7, &energy_icon(def.req_energy));
            fb.rect(sx + 8, cy + 25, 1, 5, 34);
            fb.rect(sx + 9, cy + 26, 1, 3, 34);
            fb.set_px(sx + 10, cy + 27, 34);
            fb.blit_sprite(sx + 12, cy + 24, 7, 7, &energy_icon(def.emit_energy));
        } else {
            let card_mask = if def.emit_energy != EnergyMask::NONE {
                def.emit_energy
            } else {
                def.req_energy
            };
            if card_mask != EnergyMask::NONE {
                let n = count_energies(card_mask);
                let mw = n * 8 - 1;
                if def.emit_energy != EnergyMask::NONE {
                    let sx = cx + (45 - (mw + 5)) / 2;
                    fb.rect(sx, cy + 25, 1, 5, 34);
                    fb.rect(sx + 1, cy + 26, 1, 3, 34);
                    fb.set_px(sx + 2, cy + 27, 34);
                    draw_energy_icons(fb, sx + 5, cy + 24, card_mask);
                } else {
                    draw_energy_icons(fb, cx + (45 - mw) / 2, cy + 24, card_mask);
                }
            }
        }

        // Bottom cost / Eco Lock row
        if locked {
            fb.blit_sprite(cx + 14, cy + 34, 7, 7, &ui_icon(UiIcon::Lock));
            fb.blit_sprite(cx + 24, cy + 34, 7, 7, &ui_icon(UiIcon::RoleEco));
        } else {
            let cost_s = format!("{}", cost);
            let total_w = 9 + (cost_s.len() as i32) * 6;
            let sx = cx + (45 - total_w) / 2;
            fb.blit_sprite(sx, cy + 34, 7, 7, &ui_icon(UiIcon::Coin));
            fb.text(sx + 9, cy + 34, &cost_s, if affordable { 34 } else { 47 }, 1);
        }
        if !affordable {
            fb.rect_dither_sparse(cx + 1, cy + 1, 43, 22, 1);
        }
    }

    // Right Tactical Inspector (x = 240, y = 240, w = 158, h = 58)
    fb.rect_bevel(240, 240, 158, 58, 16, 19, 1);
    if let ActiveTool::Build(kind) = view.tool {
        let def = building_def(kind);
        fb.text(244, 243, def.name, 42, 1);
        let c = building_cost(kind, p1.config);
        let cs = format!("{}", c);
        fb.blit_sprite(393 - (cs.len() as i32) * 6 - 9, 243, 7, 7, &ui_icon(UiIcon::Coin));
        fb.text(394 - (cs.len() as i32) * 6, 243, &cs, 34, 1);

        if def.req_energy != EnergyMask::NONE
            && def.emit_energy != EnergyMask::NONE
            && def.req_energy != def.emit_energy
        {
            let nx = draw_named_energies(fb, 244, 253, def.req_energy);
            fb.rect(nx, 254, 1, 5, 34);
            fb.rect(nx + 1, 255, 1, 3, 34);
            fb.set_px(nx + 2, 256, 34);
            fb.text(nx + 5, 253, "EMITS", 34, 1);
            draw_named_energies(fb, nx + 38, 253, def.emit_energy);
        } else if def.emit_energy != EnergyMask::NONE {
            fb.text(244, 253, "EMITS", 34, 1);
            draw_named_energies(fb, 278, 253, def.emit_energy);
        } else if def.req_energy != EnergyMask::NONE {
            fb.text(244, 253, "NEEDS", 29, 1);
            draw_named_energies(fb, 278, 253, def.req_energy);
        }

        let mut sx = 244;
        fb.blit_sprite(sx, 263, 7, 7, &ui_icon(UiIcon::Heart));
        let hps = format!("{}", def.max_hp);
        fb.text(sx + 8, 263, &hps, 34, 1);
        sx += 8 + (hps.len() as i32) * 6 + 6;
        if def.damage > 0 {
            fb.blit_sprite(sx, 263, 7, 7, &ui_icon(UiIcon::Sword));
            let dmgs = format!("{}", def.damage);
            fb.text(sx + 8, 263, &dmgs, 41, 1);
            sx += 8 + (dmgs.len() as i32) * 6 + 6;
        }
        if def.range_tiles > 0 && sx + 14 <= 394 {
            fb.blit_sprite(sx, 263, 7, 7, &ui_icon(UiIcon::Crosshair));
            fb.text(sx + 8, 263, &format!("{}", def.range_tiles), 27, 1);
        }
        draw_wrapped_rows(fb, 244, 274, 25, 2, def.tip, 22);
    } else if let Some((sx, sy)) = view.selected_tile {
        let idx = tile_idx(sx as usize, sy as usize);
        let g = s.building_grid[idx];
        if g != GRID_EMPTY {
            let b = &s.buildings[g as usize];
            let def = building_def(b.kind);
            fb.text(244, 243, def.name, if b.owner == TeamId::P1 { 58 } else { 62 }, 1);
            if b.level >= 2 {
                fb.blit_sprite(386, 243, 7, 7, &ui_icon(UiIcon::Upgrade));
            }

            // Symbol stat row: Heart HP, BoltOn/Off, Sword DMG, Shield Armor
            fb.blit_sprite(244, 253, 7, 7, &ui_icon(UiIcon::Heart));
            let hp_s = format!("{}/{}", b.hp, b.max_hp);
            fb.text(253, 253, &hp_s, if b.powered { 34 } else { 47 }, 1);
            let mut bx = 253 + (hp_s.len() as i32) * 6 + 4;
            fb.blit_sprite(
                bx,
                253,
                7,
                7,
                &ui_icon(if b.powered { UiIcon::BoltOn } else { UiIcon::BoltOff }),
            );
            bx += 10;
            if def.damage > 0 && bx + 22 <= 394 {
                fb.blit_sprite(bx, 253, 7, 7, &ui_icon(UiIcon::Sword));
                let dmgs = format!("{}", def.damage);
                fb.text(bx + 8, 253, &dmgs, 41, 1);
                bx += 8 + (dmgs.len() as i32) * 6 + 4;
            }
            if b.hearth_armor > 0 && bx + 16 <= 394 {
                fb.blit_sprite(bx, 253, 7, 7, &ui_icon(UiIcon::Shield));
                fb.text(bx + 8, 253, &format!("+{}", b.hearth_armor), 46, 1);
            }

            let status_override: Option<(String, u8)> = if !b.powered {
                if b.kind == BuildingType::HumanMediaArray {
                    Some(("NEEDS MECH FROM [1] RELAY".to_string(), 47))
                } else {
                    Some(("NO POWER! BUILD [1] NEARBY".to_string(), 47))
                }
            } else if matches!(
                b.kind,
                BuildingType::HumanAutocannon | BuildingType::HumanOrbitalUplink
            ) && b.net_links > 0
            {
                Some((
                    format!("+{}% FASTER ({} COMPUTE)", b.net_links * 10, b.net_links),
                    29,
                ))
            } else if b.kind == BuildingType::HumanMechFoundry {
                if s.energy_grid.net[b.owner.index()][idx].compute > 0 {
                    Some(("COMPUTE LINK: +15% TANKS".to_string(), 29))
                } else {
                    Some(("PURE MECH: NORMAL SPEED".to_string(), 42))
                }
            } else if b.kind == BuildingType::HumanMediaArray {
                Some(("EMITS COMPUTE (+10% FIRE)".to_string(), 29))
            } else if b.kind.faction() == Some(Faction::Elves) && b.net_links > 0 {
                Some((format!("{} LEY LINKS CONNECTED", b.net_links), 29))
            } else if b.gizmo_buff {
                Some(("GIZMO: +25% FIRE + SPARK".to_string(), 35))
            } else if b.fury_stacks > 0 {
                Some((format!("FURY X{}: +{}% DAMAGE", b.fury_stacks, b.fury_stacks * 25), 41))
            } else {
                None
            };

            if b.owner == TeamId::P1 && b.kind != BuildingType::Citadel {
                if b.level == 1 {
                    fb.text(244, 265, "[U]", 42, 1);
                    fb.blit_sprite(263, 265, 7, 7, &ui_icon(UiIcon::Upgrade));
                    fb.text(272, 265, &format!("{}", upgrade_cost(b.kind, p1.config)), 42, 1);
                }
                fb.text(324, 265, "[X]", 62, 1);
                fb.blit_sprite(343, 265, 7, 7, &ui_icon(UiIcon::Coin));
                fb.text(352, 265, "SELL", 62, 1);

                if let Some((ref st, col)) = status_override {
                    draw_wrapped_rows(fb, 244, 276, 25, 2, st, col);
                } else {
                    let spec = match b.kind {
                        BuildingType::GnomeWalkerTurret | BuildingType::GnomeTeslaContraption => "[M] MOVE WALKER",
                        BuildingType::LeoneLithoShaper => "[T] RAISE/LOWER WALL",
                        BuildingType::LeoneTectonicRam => "[P] PUSH TOWER EAST",
                        BuildingType::OrcDoomCauldron => "[D] DEVOUR ORC NEARBY",
                        _ => def.tip,
                    };
                    draw_wrapped_rows(fb, 244, 276, 25, 2, spec, 21);
                }
            } else if let Some((ref st, col)) = status_override {
                draw_wrapped_rows(fb, 244, 266, 25, 3, st, col);
            } else {
                draw_wrapped(fb, 244, 266, 25, def.tip, 21);
            }
        } else {
            let t = s.terrain[idx];
            let cell = s.energy_grid.net[0][idx];
            let req = p1.config.required_energies();
            let powered_tile = cell.satisfies(req);
            fb.text(244, 243, t.label(), 22, 1);
            let p_lbl = if powered_tile { "POWERED" } else { "NO POWER" };
            let p_x = 394 - (p_lbl.len() as i32) * 6;
            fb.text(p_x, 243, p_lbl, if powered_tile { 34 } else { 47 }, 1);

            let val_of = |m: EnergyMask| -> i16 {
                if m == EnergyMask::MECH {
                    cell.mech
                } else if m == EnergyMask::COMPUTE {
                    cell.compute
                } else if m == EnergyMask::MANA {
                    cell.mana
                } else if m == EnergyMask::POS_FORCE {
                    cell.pos_force
                } else {
                    cell.neg_force
                }
            };
            let mut shown = [EnergyMask::NONE; 4];
            let mut scount = 0usize;
            for &m in &ENERGY_ORDER {
                if req.contains(m) && scount < 4 {
                    shown[scount] = m;
                    scount += 1;
                }
            }
            for &m in &ENERGY_ORDER {
                if scount < 4 && !shown[..scount].contains(&m) && val_of(m) > 0 {
                    shown[scount] = m;
                    scount += 1;
                }
            }
            for &m in &ENERGY_ORDER {
                if scount < 4 && !shown[..scount].contains(&m) {
                    shown[scount] = m;
                    scount += 1;
                }
            }
            for k in 0..scount {
                let m = shown[k];
                let v = val_of(m);
                let (name, col) = energy_label_color(m);
                let ex = if (k & 1) == 0 { 244 } else { 322 };
                let ey = if k < 2 { 254 } else { 264 };
                fb.blit_sprite(ex, ey, 7, 7, &energy_icon(m));
                let lbl = format!("{} {}", name, v);
                fb.text(ex + 8, ey, &lbl, if v > 0 { col } else { 21 }, 1);
            }

            if s.energy_grid.flux_storm[idx] {
                fb.blit_sprite(244, 277, 7, 7, &ui_icon(UiIcon::OverlayPolarity));
                fb.text(254, 277, "FLUX STORM! (-6 HP/S)", 41, 1);
            } else {
                let thint = match t {
                    Terrain::HighGround => "HIGH GROUND: +1 RANGE",
                    Terrain::OreVein => "ORE VEIN: +60% ARCHIVES",
                    Terrain::LeySpring => "LEY SPRING: FREE MANA",
                    Terrain::RaisedStone => "STONE: ONLY LEONES BUILD",
                    Terrain::Chasm => "CHASM: FLYERS ONLY",
                    Terrain::Open => "[V] TOGGLES POWER DOTS",
                };
                fb.text(244, 277, thint, 21, 1);
            }
        }
    } else {
        fb.text(244, 244, "[1-5] PICK BUILDING", 22, 1);
        fb.text(244, 256, "[Q/E] TAB  [V] GRID", 29, 1);
        fb.text(244, 268, "[U] UPGRADE [X] SELL", 21, 1);
        fb.text(244, 280, "[H] GUIDE  [F2] CRT", 42, 1);
    }
}

pub fn render_title(fb: &mut Framebuffer, selected_btn: usize, total_stars: usize, ui_tick: u32) {
    fb.clear(63);
    for y in (12..300).step_by(16) {
        for x in (12..400).step_by(16) {
            fb.set_px(x, y, 3);
        }
    }

    // Header Banner
    fb.rect_bevel(26, 8, 348, 38, 17, 20, 1);
    fb.text_centered(200, 14, "CITADEL GRID", 42, 1, 2);
    fb.text_centered(200, 34, "ASYMMETRIC TOWER-BUILDING RTS (MS-DOS EDITION)", 27, 1, 1);

    // Cosmic 7-Faction Wheel Diagram
    let cx = 112;
    let cy = 136;
    fb.circle_int(cx, cy, 66, 22);
    fb.circle_int(cx, cy, 65, 20);
    fb.line_bresenham(cx - 65, cy, cx - 22, cy, 20);
    fb.line_bresenham(cx + 22, cy, cx + 65, cy, 20);
    fb.line_bresenham(cx, cy - 65, cx, cy - 22, 20);
    fb.line_bresenham(cx, cy + 22, cx, cy + 65, 20);
    fb.rect(cx - 21, cy - 21, 42, 42, 63);
    fb.circle_int(cx, cy, 22, 44);

    // Full faction names & energy symbols on wheel (zero abbreviations!)
    fb.text_centered(cx, cy - 5, "HUMANS", 44, 1, 1);
    fb.text_centered(cx - 32, cy - 34, "ELVES", 29, 1, 1);
    fb.text_centered(cx + 32, cy - 34, "ORCS", 41, 1, 1);
    fb.text_centered(cx - 29, cy + 23, "DWARVES", 46, 1, 1);
    fb.text_centered(cx + 30, cy + 23, "GNOMES", 35, 1, 1);

    fb.text(154, 58, "LEONES", 42, 1);
    fb.blit_sprite(193, 58, 7, 7, &energy_icon(EnergyMask::POS_FORCE));
    fb.text(16, 206, "DRACONES", 50, 1);
    fb.blit_sprite(67, 206, 7, 7, &energy_icon(EnergyMask::NEG_FORCE));

    // Animated rotating energy icon in wheel center
    let ic = energy_icon(ENERGY_ORDER[((ui_tick / 12) as usize) % ENERGY_ORDER.len()]);
    fb.blit_sprite(cx - 3, cy + 5, 7, 7, &ic);

    // Right Menu Panel
    fb.rect_bevel(220, 58, 168, 162, 17, 20, 1);
    fb.text_centered(304, 66, "MAIN COMMAND", 22, 1, 1);
    for i in 0..4 {
        let by = 84 + (i as i32) * 32;
        let sel = i == selected_btn;
        fb.rect_bevel(
            228,
            by,
            152,
            24,
            if sel { 24 } else { 16 },
            if sel { 42 } else { 19 },
            1,
        );
        let col = if sel { 42 } else { 22 };
        if i == 0 {
            let s = format!("[1] CHALLENGES {}/105", total_stars);
            fb.text(233, by + 8, &s, col, 1);
            fb.blit_sprite(370, by + 8, 7, 7, &ui_icon(UiIcon::StarFull));
        } else {
            let lbl = match i {
                1 => "[2] 1V1 VS HARD AI",
                2 => "[3] HOW TO PLAY",
                _ => "[4] QUIT TO DOS",
            };
            fb.text_centered(304, by + 8, lbl, col, 1, 1);
        }
    }

    // Bottom Visual Symbol & Faction Legend Box (y = 226..294)
    fb.rect_bevel(10, 226, 380, 68, 16, 19, 1);
    // 7 Factions Row 1 (4 items) & Row 2 (3 items)
    for (i, &f) in Faction::ALL.iter().enumerate() {
        let (fx, fy) = if i < 4 {
            (16 + (i as i32) * 94, 231)
        } else {
            (16 + ((i - 4) as i32) * 124, 245)
        };
        fb.blit_sprite(fx, fy, 7, 7, &faction_icon(f));
        fb.text(fx + 9, fy, f.name(), faction_ramp(f)[2], 1);
        draw_energy_icons(fb, fx + 9 + (f.name().len() as i32) * 6 + 3, fy, f.required_energies());
    }
    // Row 3: Energy Symbol Legend
    let sym_names = [
        (EnergyMask::MECH, "MECH"),
        (EnergyMask::COMPUTE, "COMPUTE"),
        (EnergyMask::MANA, "MANA"),
        (EnergyMask::POS_FORCE, "+FORCE"),
        (EnergyMask::NEG_FORCE, "-FORCE"),
    ];
    let mut sx = 16;
    for &(mask, name) in &sym_names {
        fb.blit_sprite(sx, 262, 7, 7, &energy_icon(mask));
        fb.text(sx + 9, 262, name, 22, 1);
        sx += 9 + (name.len() as i32) * 6 + 14;
    }
    // Row 4: Controls
    fb.text(16, 279, "[F2] CRT SCANLINES   [F3] AUDIO MUTE   [F11] FULLSCREEN", 21, 1);
}

pub fn render_challenges_menu(
    fb: &mut Framebuffer,
    selected_faction: usize,
    selected_tier: usize,
    stars: &[u8; 35],
    challenges: &[ChallengeDef],
) {
    fb.clear(63);
    fb.rect_bevel(0, 0, 400, 22, 17, 20, 1);
    let total: usize = stars.iter().map(|&s| s as usize).sum();
    let hdr = format!("FACTION CHALLENGES - {}/105", total);
    fb.text_centered(192, 7, &hdr, 42, 1, 1);
    fb.blit_sprite(284, 7, 7, 7, &ui_icon(UiIcon::StarFull));

    // Left 7-Faction List
    fb.rect_bevel(4, 26, 118, 230, 16, 19, 1);
    for (i, &f) in Faction::ALL.iter().enumerate() {
        let fy = 30 + (i as i32) * 32;
        let sel = i == selected_faction;
        let f_stars: u8 = stars[(i * 5)..(i * 5 + 5)].iter().sum();
        fb.rect_bevel(8, fy, 110, 29, if sel { 24 } else { 17 }, if sel { 42 } else { 19 }, 1);
        let ramp = faction_ramp(f);
        fb.rect(10, fy + 4, 3, 21, ramp[1]);
        fb.blit_sprite(15, fy + 5, 7, 7, &faction_icon(f));
        fb.text(24, fy + 5, f.name(), if sel { 42 } else { 22 }, 1);
        draw_energy_icons(fb, 15, fy + 16, f.required_energies());
        let sub = format!("{}/15", f_stars);
        fb.text(78, fy + 16, &sub, 21, 1);
        fb.blit_sprite(107, fy + 16, 7, 7, &ui_icon(UiIcon::StarFull));
    }

    // Right 5 Missions for Selected Faction (Tier 0 Learn + Tiers 1..4)
    let fac = Faction::ALL[selected_faction];
    fb.rect_bevel(126, 26, 270, 230, 16, 19, 1);
    fb.blit_sprite(132, 31, 7, 7, &faction_icon(fac));
    let fhdr = format!("{}: {}", fac.name(), fac.resource_name());
    fb.text(142, 31, &fhdr, 29, 1);
    let e_cnt = count_energies(fac.required_energies());
    draw_energy_icons(fb, 390 - e_cnt * 8, 31, fac.required_energies());

    for t in 0..5 {
        let ch = &challenges[selected_faction * 5 + t];
        let my = 43 + (t as i32) * 42;
        let sel = t == selected_tier;
        fb.rect_bevel(131, my, 260, 40, if sel { 24 } else { 17 }, if sel { 42 } else { 19 }, 1);
        let st = stars[ch.id];
        if ch.tier == 0 {
            fb.rect(135, my + 4, 34, 9, 32);
            fb.text(137, my + 5, "LEARN", 35, 1);
            fb.text(173, my + 5, ch.title, if sel { 42 } else { 22 }, 1);
        } else {
            fb.text(136, my + 5, ch.title, if sel { 42 } else { 22 }, 1);
        }
        draw_stars(fb, 363, my + 4, st, 3);
        fb.text(136, my + 15, ch.goal_label, 34, 1);
        draw_wrapped_rows(fb, 136, my + 25, 41, 1, ch.briefing_1, 21);
    }

    // Bottom Bar
    let active_ch = &challenges[selected_faction * 5 + selected_tier];
    fb.rect_bevel(0, 260, 400, 40, 17, 20, 1);
    let tip_txt = if active_ch.tip.len() > 64 { &active_ch.tip[..64] } else { active_ch.tip };
    fb.text(8, 264, tip_txt, 42, 1);
    fb.rect_bevel(8, 276, 86, 19, 16, 19, 1);
    fb.text_centered(51, 282, "[ESC] BACK", 22, 1, 1);
    fb.rect_bevel(266, 276, 126, 19, 32, 34, 1);
    fb.text_centered(329, 282, "[ENTER] LAUNCH", 23, 1, 1);
}

pub fn render_skirmish_setup(
    fb: &mut Framebuffer,
    p1_team: FactionTeam,
    p2_team: FactionTeam,
    map: &SkirmishMap,
    diff_idx: usize,
) {
    fb.clear(63);
    fb.rect_bevel(0, 0, 400, 20, 17, 20, 1);
    fb.text_centered(200, 6, "1V1 VS AI - COMBINE 1 TO 3 FACTIONS PER TEAM", 42, 1, 1);

    // P1 Panel (x = 4..196, y = 24..176)
    fb.rect_bevel(4, 24, 192, 152, 16, 57, 1);
    let p1_hdr = format!("PLAYER ({} TEAM | {}% COST)", p1_team.count, p1_team.cost_pct());
    fb.text(10, 29, &p1_hdr, 58, 1);
    for (i, &f) in Faction::ALL.iter().enumerate() {
        let fy = 41 + (i as i32) * 18;
        let on = p1_team.has(f);
        fb.rect_bevel(10, fy, 180, 16, if on { 24 } else { 17 }, if on { 42 } else { 19 }, 1);
        let mark = if on { "[X]" } else { "[ ]" };
        fb.text(14, fy + 4, mark, if on { 42 } else { 21 }, 1);
        fb.blit_sprite(35, fy + 4, 7, 7, &faction_icon(f));
        fb.text(45, fy + 4, f.name(), if on { 42 } else { 21 }, 1);
        let e_cnt = count_energies(f.required_energies());
        draw_energy_icons(fb, 186 - e_cnt * 8, fy + 4, f.required_energies());
    }

    // P2 AI Panel (x = 204..396, y = 24..176)
    fb.rect_bevel(204, 24, 192, 152, 16, 61, 1);
    let p2_hdr = format!("AI ({} TEAM | {}% COST)", p2_team.count, p2_team.cost_pct());
    fb.text(210, 29, &p2_hdr, 62, 1);
    for (i, &f) in Faction::ALL.iter().enumerate() {
        let fy = 41 + (i as i32) * 18;
        let on = p2_team.has(f);
        fb.rect_bevel(210, fy, 180, 16, if on { 40 } else { 17 }, if on { 42 } else { 19 }, 1);
        let mark = if on { "[X]" } else { "[ ]" };
        fb.text(214, fy + 4, mark, if on { 42 } else { 21 }, 1);
        fb.blit_sprite(235, fy + 4, 7, 7, &faction_icon(f));
        fb.text(245, fy + 4, f.name(), if on { 42 } else { 21 }, 1);
        let e_cnt = count_energies(f.required_energies());
        draw_energy_icons(fb, 386 - e_cnt * 8, fy + 4, f.required_energies());
    }

    // Map & Difficulty Selector (y = 180..256)
    fb.rect_bevel(4, 180, 266, 76, 16, 19, 1);
    let m_hdr = format!("MAP: < {} > ({}/6)", map.name, map.id + 1);
    fb.text(12, 186, &m_hdr, 42, 1);
    draw_wrapped_rows(fb, 12, 197, 32, 2, map.desc, 22);
    fb.text(12, 219, "TOGGLE 1-3 FACTIONS ABOVE FOR", 21, 1);
    fb.text(12, 229, "SHARED-GRID TEAM SYNERGY!", 29, 1);
    fb.text(12, 243, "[< / >] MAP   [R] RANDOMIZE", 21, 1);

    // Minimap Preview (50x28 at 2x2 per tile)
    fb.rect(210, 194, 52, 30, 1);
    for y in 0..MAP_H {
        for x in 0..MAP_W {
            let col = match map.tiles[tile_idx(x, y)] {
                Terrain::Open => 5,
                Terrain::HighGround => 9,
                Terrain::OreVein => 42,
                Terrain::LeySpring => 29,
                Terrain::Chasm => 2,
                Terrain::RaisedStone => 12,
            };
            fb.rect(211 + (x as i32) * 2, 195 + (y as i32) * 2, 2, 2, col);
        }
    }
    fb.rect(211 + (map.p1_citadel.0 as i32) * 2, 195 + (map.p1_citadel.1 as i32) * 2, 3, 3, 57);
    fb.rect(211 + (map.p2_citadel.0 as i32) * 2, 195 + (map.p2_citadel.1 as i32) * 2, 3, 3, 61);

    // Difficulty Box (x = 274..396, y = 180..256)
    fb.rect_bevel(274, 180, 122, 76, 16, 19, 1);
    fb.text(282, 186, "AI DIFFICULTY:", 22, 1);
    for (i, lbl) in ["EASY", "NORMAL", "HARD (AI SEARCH)"].iter().enumerate() {
        let dy = 200 + (i as i32) * 17;
        let sel = i == diff_idx;
        fb.rect_bevel(280, dy, 110, 15, if sel { 24 } else { 17 }, if sel { 42 } else { 19 }, 1);
        fb.text(284, dy + 4, lbl, if sel { 42 } else { 21 }, 1);
    }

    // Bottom Bar
    fb.rect_bevel(0, 260, 400, 40, 17, 20, 1);
    fb.rect_bevel(8, 268, 86, 24, 16, 19, 1);
    fb.text_centered(51, 276, "[ESC] BACK", 22, 1, 1);
    fb.rect_bevel(106, 268, 124, 24, 16, 19, 1);
    fb.text_centered(168, 276, "[R] RANDOM TEAMS", 29, 1, 1);
    fb.rect_bevel(256, 268, 136, 24, 32, 34, 1);
    fb.text_centered(324, 276, "[ENTER] START 1V1", 23, 1, 1);
}

pub fn render_help_modal(fb: &mut Framebuffer) {
    fb.rect_dither(0, 0, 400, 300, 1, 0);
    fb.rect_bevel(14, 14, 372, 272, 17, 20, 1);
    fb.text_centered(200, 22, "CITADEL GRID - COMMANDER MANUAL", 42, 1, 1);

    let lines = [
        ("CONTROLS:", 29),
        (" [1]-[5]: PICK BUILDING | LEFT CLICK TILE TO PLACE", 22),
        (" [Q/E/TAB]: SWITCH FACTION TAB (IN MULTI-FACTION TEAMS)", 22),
        (" [U]: UPGRADE | [X]: SELL/SACRIFICE | [V]: GRID OVERLAY", 22),
        (" [M]: MOVE WALKER | [T]: STONE WALL | [P]: PUSH | [D]: DEVOUR", 22),
        (" [SPACE]: PAUSE | [F]: 2X SPEED | [F2]: CRT | [F11]: FULLSCREEN", 22),
        ("", 22),
        ("CORE RULES:", 42),
        (" - TOWERS NEED ENERGY FROM CITADEL OR [1] INFRASTRUCTURE", 21),
        (" - +FORCE & -FORCE CANCEL AND SPARK DAMAGE FLUX STORMS", 21),
        (" - HIGH GROUND ADDS +1 RANGE; ORE BOOSTS INCOME +60%", 21),
        (" - UNITS FIGHT AUTOMATICALLY - MASTER PLACEMENT & SYNERGY!", 21),
    ];
    for (i, (txt, col)) in lines.iter().enumerate() {
        fb.text(22, 38 + (i as i32) * 14, txt, *col, 1);
    }

    // Visual Symbol Key rows inside Manual
    fb.text(22, 208, "SYMBOLS:", 29, 1);
    let syms = [
        (energy_icon(EnergyMask::MECH), "MECH"),
        (energy_icon(EnergyMask::COMPUTE), "COMPUTE"),
        (energy_icon(EnergyMask::MANA), "MANA"),
        (energy_icon(EnergyMask::POS_FORCE), "+FORCE"),
        (energy_icon(EnergyMask::NEG_FORCE), "-FORCE"),
    ];
    let mut sx = 76;
    for (ic, lbl) in &syms {
        fb.blit_sprite(sx, 208, 7, 7, ic);
        fb.text(sx + 8, 208, lbl, 22, 1);
        sx += 8 + (lbl.len() as i32) * 6 + 6;
    }

    let uis = [
        (ui_icon(UiIcon::Heart), "HEALTH"),
        (ui_icon(UiIcon::Sword), "POWER"),
        (ui_icon(UiIcon::Crosshair), "RANGE"),
        (ui_icon(UiIcon::Shield), "ARMOR"),
        (ui_icon(UiIcon::Coin), "COST"),
    ];
    let mut ux = 76;
    for (ic, lbl) in &uis {
        fb.blit_sprite(ux, 224, 7, 7, ic);
        fb.text(ux + 8, 224, lbl, 21, 1);
        ux += 8 + (lbl.len() as i32) * 6 + 7;
    }

    fb.rect_bevel(136, 250, 128, 22, 24, 42, 1);
    fb.text_centered(200, 257, "[H / ESC] CLOSE", 42, 1, 1);
}

pub fn render_pause_modal(fb: &mut Framebuffer) {
    fb.rect_dither(0, 0, 400, 300, 1, 0);
    fb.rect_bevel(110, 82, 180, 134, 17, 20, 1);
    fb.text_centered(200, 92, "COMMAND PAUSED", 42, 1, 1);
    for (i, lbl) in ["[ESC] RESUME", "[R] RESTART", "[H] HOW TO PLAY", "[Q] QUIT TO MENU"].iter().enumerate() {
        let by = 114 + (i as i32) * 24;
        fb.rect_bevel(126, by, 148, 19, 16, 19, 1);
        fb.text_centered(200, by + 6, lbl, 22, 1, 1);
    }
}

pub fn render_debrief_modal(
    fb: &mut Framebuffer,
    won: bool,
    draw: bool,
    stars: u8,
    ticks: u32,
    state: &GameState,
) {
    fb.rect_dither(0, 0, 400, 300, 1, 0);
    fb.rect_bevel(72, 60, 256, 176, 17, if won { 34 } else { 61 }, 1);
    let title = if draw {
        "STALEMATE DRAW"
    } else if won {
        "VICTORY ACHIEVED!"
    } else {
        "CITADEL FALLEN"
    };
    fb.text_centered(200, 72, title, if won { 42 } else { 62 }, 1, 2);
    if won && stars > 0 {
        draw_stars(fb, 188, 98, stars, 3);
    }

    let secs = ticks / TICKS_PER_SEC;
    let l1 = format!("DURATION: {:02}:{:02}  KILLS: {}", secs / 60, secs % 60, state.teams[0].kills);
    let l2 = format!("EARNED: {}  BUILT: {}", state.teams[0].total_earned, state.teams[0].buildings_built);
    let l3 = format!("CITADEL HP: {} / {}", state.citadel_hp(TeamId::P1), CITADEL_MAX_HP);
    fb.text_centered(200, 120, &l1, 22, 1, 1);
    fb.text_centered(200, 136, &l2, 22, 1, 1);
    fb.text_centered(200, 152, &l3, 29, 1, 1);

    fb.rect_bevel(88, 192, 102, 24, 16, 19, 1);
    fb.text_centered(139, 200, "[R] REPLAY", 22, 1, 1);
    fb.rect_bevel(210, 192, 102, 24, 24, 42, 1);
    fb.text_centered(261, 200, "[ENTER] MENU", 42, 1, 1);
}

fn draw_wrapped(fb: &mut Framebuffer, x: i32, y: i32, max_cols: usize, text: &str, color: u8) {
    draw_wrapped_rows(fb, x, y, max_cols, 3, text, color);
}

fn draw_wrapped_rows(
    fb: &mut Framebuffer,
    x: i32,
    y: i32,
    max_cols: usize,
    max_rows: i32,
    text: &str,
    color: u8,
) {
    let mut row = 0i32;
    let mut cur = String::new();
    for word in text.split_whitespace() {
        if !cur.is_empty() && cur.len() + 1 + word.len() > max_cols {
            fb.text(x, y + row * 9, &cur, color, 1);
            row += 1;
            cur.clear();
            if row >= max_rows {
                return;
            }
        }
        if !cur.is_empty() {
            cur.push(' ');
        }
        cur.push_str(word);
    }
    if !cur.is_empty() && row < max_rows {
        fb.text(x, y + row * 9, &cur, color, 1);
    }
}
