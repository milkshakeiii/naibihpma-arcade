use std::collections::VecDeque;

use super::defs::{calc_damage, unit_def};
use super::energy::EnergyGrid;
use super::types::*;

#[derive(Clone, Debug)]
pub struct FlowField {
    pub dist: [u16; MAP_TILES],
    pub wall_dist: [u16; MAP_TILES],
}

impl Default for FlowField {
    fn default() -> Self {
        Self {
            dist: [u16::MAX; MAP_TILES],
            wall_dist: [u16::MAX; MAP_TILES],
        }
    }
}

pub fn recompute_flow_fields(
    terrain: &[Terrain; MAP_TILES],
    buildings: &[Building],
    fields: &mut [FlowField; 2],
) {
    let mut q: VecDeque<(u8, u8)> = VecDeque::with_capacity(MAP_TILES);

    for team_idx in 0..2 {
        let enemy = if team_idx == 0 { TeamId::P2 } else { TeamId::P1 };
        let f = &mut fields[team_idx];
        f.dist.fill(u16::MAX);
        f.wall_dist.fill(u16::MAX);
        q.clear();

        // Goal: enemy Citadel if any, else any enemy building
        let mut any_citadel = false;
        for b in buildings {
            if b.hp > 0 && b.owner == enemy && b.kind == BuildingType::Citadel {
                let idx = tile_idx(b.x as usize, b.y as usize);
                f.dist[idx] = 0;
                f.wall_dist[idx] = 0;
                q.push_back((b.x, b.y));
                any_citadel = true;
            }
        }
        if !any_citadel {
            for b in buildings {
                if b.hp > 0 && b.owner == enemy {
                    let idx = tile_idx(b.x as usize, b.y as usize);
                    f.dist[idx] = 0;
                    f.wall_dist[idx] = 0;
                    q.push_back((b.x, b.y));
                }
            }
        }

        // Standard BFS avoiding Chasm & RaisedStone
        while let Some((cx, cy)) = q.pop_front() {
            let cur_d = f.dist[tile_idx(cx as usize, cy as usize)];
            for (dx, dy) in [(1i32, 0), (-1, 0), (0, 1), (0, -1)] {
                let nx = cx as i32 + dx;
                let ny = cy as i32 + dy;
                if nx < 0 || nx >= MAP_W as i32 || ny < 0 || ny >= MAP_H as i32 {
                    continue;
                }
                let ux = nx as u8;
                let uy = ny as u8;
                let nidx = tile_idx(ux as usize, uy as usize);
                if terrain[nidx].blocks_ground() {
                    continue;
                }
                if f.dist[nidx] > cur_d + 1 {
                    f.dist[nidx] = cur_d + 1;
                    q.push_back((ux, uy));
                }
            }
        }

        // Fallback path allowing bashing through RaisedStone (never Chasm)
        // Re-seed from goals
        q.clear();
        for idx in 0..MAP_TILES {
            if f.wall_dist[idx] == 0 {
                q.push_back(((idx % MAP_W) as u8, (idx / MAP_W) as u8));
            }
        }
        while let Some((cx, cy)) = q.pop_front() {
            let cur_d = f.wall_dist[tile_idx(cx as usize, cy as usize)];
            for (dx, dy) in [(1i32, 0), (-1, 0), (0, 1), (0, -1)] {
                let nx = cx as i32 + dx;
                let ny = cy as i32 + dy;
                if nx < 0 || nx >= MAP_W as i32 || ny < 0 || ny >= MAP_H as i32 {
                    continue;
                }
                let ux = nx as u8;
                let uy = ny as u8;
                let nidx = tile_idx(ux as usize, uy as usize);
                if terrain[nidx] == Terrain::Chasm {
                    continue;
                }
                let step_cost = if terrain[nidx] == Terrain::RaisedStone {
                    8
                } else {
                    1
                };
                if f.wall_dist[nidx] > cur_d.saturating_add(step_cost) {
                    f.wall_dist[nidx] = cur_d.saturating_add(step_cost);
                    q.push_back((ux, uy));
                }
            }
        }
    }
}

#[inline]
pub fn tile_center_px(tx: u8, ty: u8) -> (i32, i32) {
    ((tx as i32) * TILE_PX + TILE_PX / 2, (ty as i32) * TILE_PX + TILE_PX / 2)
}

#[inline]
pub fn px_to_tile(px: i32, py: i32) -> (usize, usize) {
    let tx = (px / TILE_PX).clamp(0, MAP_W as i32 - 1) as usize;
    let ty = (py / TILE_PX).clamp(0, MAP_H as i32 - 1) as usize;
    (tx, ty)
}

#[inline]
pub fn dist_sq(x0: i32, y0: i32, x1: i32, y1: i32) -> i32 {
    let dx = x1 - x0;
    let dy = y1 - y0;
    dx * dx + dy * dy
}

pub fn step_units(
    tick: u32,
    terrain: &mut [Terrain; MAP_TILES],
    raised_hp: &mut [i16; MAP_TILES],
    teams: &mut [TeamState; 2],
    buildings: &mut [Building],
    building_grid: &[u16; MAP_TILES],
    units: &mut Vec<Unit>,
    energy_grid: &EnergyGrid,
    flow_fields: &[FlowField; 2],
    flow_dirty: &mut bool,
    record_fx: bool,
    fx_beams: &mut Vec<FxBeam>,
    fx_bursts: &mut Vec<FxBurst>,
) {
    // Track per-unit attacks/healing without mutable aliasing
    let unit_count = units.len();
    for i in 0..unit_count {
        if units[i].hp <= 0 {
            continue;
        }
        if units[i].cooldown > 0 {
            units[i].cooldown -= 1;
        }
        if units[i].slow_ticks > 0 {
            units[i].slow_ticks -= 1;
        }
        if units[i].corrode_ticks > 0 {
            units[i].corrode_ticks -= 1;
        }
        if units[i].hit_flash > 0 {
            units[i].hit_flash -= 1;
        }

        let u = units[i];
        let def = unit_def(u.kind);
        let (utx, uty) = px_to_tile(u.px, u.py);
        let uidx = tile_idx(utx, uty);

        // Environmental auras every 15 ticks
        if tick % 15 == 0 {
            if energy_grid.flux_storm[uidx] {
                units[i].hp -= 6;
                units[i].hit_flash = 2;
                if units[i].hp <= 0 {
                    continue;
                }
            }
            // Enemy Void Brazier corrosion
            let enemy = u.owner.other();
            for b in buildings.iter() {
                if b.hp > 0
                    && b.build_ticks_left == 0
                    && b.owner == enemy
                    && b.kind == BuildingType::DraconeVoidBrazier
                    && chebyshev(utx as u8, uty as u8, b.x, b.y) <= 5
                {
                    units[i].corrode_ticks = 30;
                    units[i].hp -= 3;
                    break;
                }
            }
            if units[i].hp <= 0 {
                continue;
            }
        }

        // Special case: LivingBoulder rolls down a cardinal lane trampling units & smashing buildings/RaisedStone
        if u.kind == UnitKind::LivingBoulder {
            let spd = def.speed_px_per_tick;
            if u.px % TILE_PX == TILE_PX / 2 && u.py % TILE_PX == TILE_PX / 2 {
                let ff = &flow_fields[u.owner.index()];
                let cur_d = ff.wall_dist[tile_idx(utx, uty)];
                let ax = utx as i32 + u.dir_dx as i32;
                let ay = uty as i32 + u.dir_dy as i32;
                let ahead_ok = ax >= 0
                    && ax < MAP_W as i32
                    && ay >= 0
                    && ay < MAP_H as i32
                    && {
                        let aidx = tile_idx(ax as usize, ay as usize);
                        terrain[aidx] != Terrain::Chasm && ff.wall_dist[aidx] < cur_d
                    };
                if !ahead_ok {
                    let mut best_dir: Option<(i8, i8, u16)> = None;
                    for (dx, dy) in [(1i32, 0), (-1, 0), (0, 1), (0, -1)] {
                        let cx = utx as i32 + dx;
                        let cy = uty as i32 + dy;
                        if cx >= 0 && cx < MAP_W as i32 && cy >= 0 && cy < MAP_H as i32 {
                            let cidx = tile_idx(cx as usize, cy as usize);
                            if terrain[cidx] != Terrain::Chasm
                                && ff.wall_dist[cidx] < best_dir.map_or(u16::MAX, |b| b.2)
                            {
                                best_dir = Some((dx as i8, dy as i8, ff.wall_dist[cidx]));
                            }
                        }
                    }
                    if let Some((ddx, ddy, _)) = best_dir {
                        units[i].dir_dx = ddx;
                        units[i].dir_dy = ddy;
                    }
                }
            }
            let mut nx = u.px + (units[i].dir_dx as i32) * spd;
            let mut ny = u.py + (units[i].dir_dy as i32) * spd;
            let out_of_bounds = nx < 4 || nx >= PLAYFIELD_W - 4 || ny < 4 || ny >= PLAYFIELD_H - 4;
            let (mut ntx, mut nty) = px_to_tile(nx.clamp(4, PLAYFIELD_W - 5), ny.clamp(4, PLAYFIELD_H - 5));
            let mut nidx = tile_idx(ntx, nty);

            if out_of_bounds || terrain[nidx] == Terrain::Chasm {
                // Steer along flow field around Chasm / edge toward enemy base!
                let ff = &flow_fields[u.owner.index()];
                let mut best_dir: Option<(i8, i8, u16)> = None;
                for (dx, dy) in [(1i32, 0), (-1, 0), (0, 1), (0, -1)] {
                    let cx = utx as i32 + dx;
                    let cy = uty as i32 + dy;
                    if cx >= 0 && cx < MAP_W as i32 && cy >= 0 && cy < MAP_H as i32 {
                        let cidx = tile_idx(cx as usize, cy as usize);
                        if terrain[cidx] != Terrain::Chasm && ff.wall_dist[cidx] < best_dir.map_or(u16::MAX, |b| b.2) {
                            best_dir = Some((dx as i8, dy as i8, ff.wall_dist[cidx]));
                        }
                    }
                }
                if let Some((ddx, ddy, _)) = best_dir {
                    units[i].dir_dx = ddx;
                    units[i].dir_dy = ddy;
                    nx = u.px + (ddx as i32) * spd;
                    ny = u.py + (ddy as i32) * spd;
                    let t = px_to_tile(nx.clamp(4, PLAYFIELD_W - 5), ny.clamp(4, PLAYFIELD_H - 5));
                    ntx = t.0;
                    nty = t.1;
                    nidx = tile_idx(ntx, nty);
                } else {
                    units[i].hp = 0;
                    continue;
                }
            }
            if terrain[nidx] == Terrain::RaisedStone {
                raised_hp[nidx] -= 110;
                if raised_hp[nidx] <= 0 {
                    terrain[nidx] = Terrain::Open;
                    raised_hp[nidx] = 0;
                    *flow_dirty = true;
                }
                units[i].hp = 0;
                if record_fx {
                    fx_bursts.push(FxBurst {
                        px: nx as i16,
                        py: ny as i16,
                        radius: 9,
                        color: 42,
                        ttl: 6,
                    });
                }
                continue;
            }
            let occ = building_grid[nidx];
            if occ != GRID_EMPTY {
                let b = &mut buildings[occ as usize];
                if b.hp > 0 && b.owner != u.owner {
                    let dmg = calc_damage(def.damage * 2, DamageClass::Siege, ArmorClass::Fortified, b.armor);
                    b.hp -= dmg;
                    // Splash nearby enemy units & structures on boulder smash
                    for bi in 0..buildings.len() {
                        if bi != occ as usize && buildings[bi].hp > 0 && buildings[bi].owner != u.owner {
                            let (obx, oby) = tile_center_px(buildings[bi].x, buildings[bi].y);
                            if dist_sq(nx, ny, obx, oby) <= 22 * 22 {
                                buildings[bi].hp -= dmg / 2;
                            }
                        }
                    }
                    units[i].hp = 0;
                    if record_fx {
                        fx_bursts.push(FxBurst {
                            px: nx as i16,
                            py: ny as i16,
                            radius: 10,
                            color: 43,
                            ttl: 7,
                        });
                    }
                    continue;
                }
            }
            units[i].px = nx;
            units[i].py = ny;
            // Trample nearby enemy ground units
            if units[i].cooldown == 0 {
                let mut hit_any = false;
                for j in 0..unit_count {
                    if j == i || units[j].hp <= 0 || units[j].owner == u.owner {
                        continue;
                    }
                    if unit_def(units[j].kind).kind.is_flying() {
                        continue;
                    }
                    if dist_sq(nx, ny, units[j].px, units[j].py) <= 13 * 13 {
                        let ac = unit_def(units[j].kind).armor_class;
                        let arm = unit_def(units[j].kind).armor
                            - if units[j].corrode_ticks > 0 { 2 } else { 0 };
                        let dmg = calc_damage(def.damage, DamageClass::Siege, ac, arm.max(0));
                        units[j].hp -= dmg;
                        units[j].hit_flash = 3;
                        hit_any = true;
                    }
                }
                if hit_any {
                    units[i].cooldown = def.fire_cd;
                    if record_fx {
                        fx_bursts.push(FxBurst {
                            px: nx as i16,
                            py: ny as i16,
                            radius: 7,
                            color: 42,
                            ttl: 4,
                        });
                    }
                }
            }
            continue;
        }

        // Regular combat/support units: find target in range
        let rng_sq = def.range_px * def.range_px;

        // 1. Repairer support priority (ArcaneWisp & TinkerBot) if injured allied building within range
        let mut acted = false;
        if def.is_repairer && units[i].cooldown == 0 {
            let mut best_ally: Option<usize> = None;
            let mut best_missing = 0i32;
            for (bi, b) in buildings.iter().enumerate() {
                if b.hp > 0 && b.hp < b.max_hp && b.owner == u.owner {
                    let (bx, by) = tile_center_px(b.x, b.y);
                    if dist_sq(u.px, u.py, bx, by) <= (def.range_px + 10) * (def.range_px + 10) {
                        let missing = b.max_hp - b.hp;
                        if missing > best_missing {
                            best_missing = missing;
                            best_ally = Some(bi);
                        }
                    }
                }
            }
            if let Some(bi) = best_ally {
                let heal = if u.kind == UnitKind::ArcaneWisp { 14 } else { 11 };
                buildings[bi].hp = (buildings[bi].hp + heal).min(buildings[bi].max_hp);
                units[i].cooldown = def.fire_cd;
                acted = true;
                if record_fx {
                    let (bx, by) = tile_center_px(buildings[bi].x, buildings[bi].y);
                    fx_beams.push(FxBeam {
                        x0: u.px as i16,
                        y0: u.py as i16,
                        x1: bx as i16,
                        y1: by as i16,
                        color: 35,
                        ttl: 4,
                    });
                }
            }
        }

        // 2. Find enemy unit or enemy building in attack range
        let mut target_unit: Option<usize> = None;
        let mut best_u_d2 = rng_sq + 1;
        for j in 0..unit_count {
            if j == i || units[j].hp <= 0 || units[j].owner == u.owner {
                continue;
            }
            let target_fly = units[j].kind.is_flying();
            if target_fly && !def.hits_air {
                continue;
            }
            if !target_fly && !def.hits_ground {
                continue;
            }
            let d2 = dist_sq(u.px, u.py, units[j].px, units[j].py);
            if d2 <= rng_sq && d2 < best_u_d2 {
                best_u_d2 = d2;
                target_unit = Some(j);
            }
        }

        let mut target_bldg: Option<usize> = None;
        let mut best_b_d2 = (def.range_px + 7) * (def.range_px + 7) + 1;
        if target_unit.is_none() {
            for (bi, b) in buildings.iter().enumerate() {
                if b.hp <= 0 || b.owner == u.owner {
                    continue;
                }
                let (bx, by) = tile_center_px(b.x, b.y);
                let d2 = dist_sq(u.px, u.py, bx, by);
                if d2 <= (def.range_px + 7) * (def.range_px + 7) && d2 < best_b_d2 {
                    best_b_d2 = d2;
                    target_bldg = Some(bi);
                }
            }
        }

        // Compute damage modifiers (Orc -Force fury, Dwarven Knowledge tier, Dracone Greed, Escalation)
        let mut raw_dmg = def.damage;
        if tick >= SUDDEN_ESCALATION_TICK {
            raw_dmg = (raw_dmg * 135) / 100;
        }
        if matches!(u.kind, UnitKind::OrcRaider | UnitKind::OrcRam) {
            if energy_grid.net[u.owner.index()][uidx].neg_force > 0 {
                raw_dmg = (raw_dmg * 120) / 100;
            }
        }
        if u.kind == UnitKind::IroncladGolem {
            let kt = teams[u.owner.index()].knowledge_tier as i32;
            raw_dmg = (raw_dmg * (100 + kt * 8)) / 100;
        }
        if matches!(u.kind, UnitKind::SkyDrake | UnitKind::ElderWyrm) {
            let hoard_bonus = ((teams[u.owner.index()].resources / 100) * 10).clamp(0, 50);
            raw_dmg = (raw_dmg * (100 + hoard_bonus)) / 100;
        }

        if !acted && (target_unit.is_some() || target_bldg.is_some()) {
            if units[i].cooldown == 0 {
                if let Some(j) = target_unit {
                    let tdef = unit_def(units[j].kind);
                    let arm = (tdef.armor - if units[j].corrode_ticks > 0 { 2 } else { 0 }).max(0);
                    let dmg = calc_damage(raw_dmg, def.dmg_class, tdef.armor_class, arm);
                    units[j].hp -= dmg;
                    units[j].hit_flash = 3;
                    if u.kind == UnitKind::ElderWyrm {
                        units[j].corrode_ticks = 45;
                    }
                    units[i].cooldown = def.fire_cd;
                    if record_fx {
                        fx_beams.push(FxBeam {
                            x0: u.px as i16,
                            y0: u.py as i16,
                            x1: units[j].px as i16,
                            y1: units[j].py as i16,
                            color: unit_beam_color(u.kind),
                            ttl: 4,
                        });
                    }
                } else if let Some(bi) = target_bldg {
                    let b = &mut buildings[bi];
                    let scaled = (raw_dmg * def.building_mult_pct) / 100;
                    let arm = b.armor + b.hearth_armor as i32;
                    let dmg = calc_damage(scaled, def.dmg_class, ArmorClass::Fortified, arm);
                    b.hp -= dmg;
                    if u.kind == UnitKind::TinkerBot {
                        b.slow_ticks = 45;
                    }
                    units[i].cooldown = def.fire_cd;
                    if record_fx {
                        let (bx, by) = tile_center_px(b.x, b.y);
                        fx_beams.push(FxBeam {
                            x0: u.px as i16,
                            y0: u.py as i16,
                            x1: bx as i16,
                            y1: by as i16,
                            color: unit_beam_color(u.kind),
                            ttl: 4,
                        });
                    }
                }
            }
            // In combat range, hold position!
            continue;
        }

        // 3. Movement toward enemy objective
        let mut speed = def.speed_px_per_tick;
        if u.kind == UnitKind::ArcaneWisp && energy_grid.ley_tiles[u.owner.index()][uidx] {
            speed *= 2;
        }
        if matches!(u.kind, UnitKind::OrcRaider)
            && energy_grid.net[u.owner.index()][uidx].neg_force > 0
            && (tick % 2 == 0)
        {
            speed += 1;
        }
        if u.slow_ticks > 0 {
            speed = (speed / 2).max(if tick % 2 == 0 { 1 } else { 0 });
        }
        if speed == 0 {
            continue;
        }

        if u.kind.is_flying() {
            // Fly directly toward nearest enemy building (prefer Citadel if close, else nearest)
            let mut goal: Option<(i32, i32)> = None;
            let mut best_d = i32::MAX;
            for b in buildings.iter() {
                if b.hp > 0 && b.owner != u.owner {
                    let (bx, by) = tile_center_px(b.x, b.y);
                    let mut d = dist_sq(u.px, u.py, bx, by);
                    if b.kind == BuildingType::Citadel {
                        d = (d * 3) / 4;
                    }
                    if d < best_d {
                        best_d = d;
                        goal = Some((bx, by));
                    }
                }
            }
            if let Some((gx, gy)) = goal {
                let dx = (gx - u.px).signum();
                let dy = (gy - u.py).signum();
                units[i].px = (u.px + dx * speed).clamp(4, PLAYFIELD_W - 5);
                units[i].py = (u.py + dy * speed).clamp(4, PLAYFIELD_H - 5);
                if dx != 0 || dy != 0 {
                    units[i].dir_dx = dx as i8;
                    units[i].dir_dy = dy as i8;
                }
            }
            continue;
        }

        // Ground unit pathing via cached flow field
        let ff = &flow_fields[u.owner.index()];
        let use_wall_fallback = ff.dist[uidx] == u16::MAX;
        let table = if use_wall_fallback { &ff.wall_dist } else { &ff.dist };
        let cur_d = table[uidx];

        // Choose neighbor tile with smallest flow distance (ties broken toward center of tile / enemy side)
        let mut best_nt = (utx, uty);
        let mut best_nd = cur_d;
        let dirs: [(i32, i32); 4] = if u.owner == TeamId::P1 {
            [(1, 0), (0, 1), (0, -1), (-1, 0)]
        } else {
            [(-1, 0), (0, -1), (0, 1), (1, 0)]
        };

        for (dx, dy) in dirs {
            let nx = utx as i32 + dx;
            let ny = uty as i32 + dy;
            if nx < 0 || nx >= MAP_W as i32 || ny < 0 || ny >= MAP_H as i32 {
                continue;
            }
            let nidx = tile_idx(nx as usize, ny as usize);
            if !use_wall_fallback && terrain[nidx].blocks_ground() {
                continue;
            }
            if use_wall_fallback && terrain[nidx] == Terrain::Chasm {
                continue;
            }
            if table[nidx] < best_nd {
                best_nd = table[nidx];
                best_nt = (nx as usize, ny as usize);
            }
        }

        // If the best next tile is a RaisedStone wall (in fallback mode), bash it!
        let next_idx = tile_idx(best_nt.0, best_nt.1);
        if terrain[next_idx] == Terrain::RaisedStone {
            if units[i].cooldown == 0 {
                let dmg = ((raw_dmg * def.building_mult_pct) / 100).max(10);
                raised_hp[next_idx] -= dmg as i16;
                units[i].cooldown = def.fire_cd;
                if record_fx {
                    let (wx, wy) = tile_center_px(best_nt.0 as u8, best_nt.1 as u8);
                    fx_beams.push(FxBeam {
                        x0: u.px as i16,
                        y0: u.py as i16,
                        x1: wx as i16,
                        y1: wy as i16,
                        color: 42,
                        ttl: 4,
                    });
                }
                if raised_hp[next_idx] <= 0 {
                    terrain[next_idx] = Terrain::Open;
                    raised_hp[next_idx] = 0;
                    *flow_dirty = true;
                }
            }
            continue;
        }

        let (tx_px, ty_px) = tile_center_px(best_nt.0 as u8, best_nt.1 as u8);
        let dx = (tx_px - u.px).signum();
        let dy = (ty_px - u.py).signum();
        let mut npx = u.px + dx * speed;
        let mut npy = u.py + dy * speed;
        // Clamp overshoot when near target center
        if (dx > 0 && npx > tx_px) || (dx < 0 && npx < tx_px) {
            npx = tx_px;
        }
        if (dy > 0 && npy > ty_px) || (dy < 0 && npy < ty_px) {
            npy = ty_px;
        }
        units[i].px = npx.clamp(4, PLAYFIELD_W - 5);
        units[i].py = npy.clamp(4, PLAYFIELD_H - 5);
        if dx != 0 || dy != 0 {
            units[i].dir_dx = dx as i8;
            units[i].dir_dy = dy as i8;
        }
    }
}

#[inline]
fn unit_beam_color(kind: UnitKind) -> u8 {
    match kind {
        UnitKind::StriderTank => 43,
        UnitKind::ArcaneWisp => 29,
        UnitKind::OrcRaider | UnitKind::OrcRam => 40,
        UnitKind::IroncladGolem => 46,
        UnitKind::TinkerBot => 31,
        UnitKind::LivingBoulder | UnitKind::StoneGolem => 42,
        UnitKind::SkyDrake => 41,
        UnitKind::ElderWyrm => 54,
    }
}
