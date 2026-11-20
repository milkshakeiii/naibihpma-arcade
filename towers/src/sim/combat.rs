use super::defs::{building_def, calc_damage, unit_def};
use super::energy::EnergyGrid;
use super::types::*;
use super::units::{dist_sq, px_to_tile, tile_center_px};

pub fn step_buildings_and_combat(
    tick: u32,
    terrain: &[Terrain; MAP_TILES],
    teams: &mut [TeamState; 2],
    buildings: &mut Vec<Building>,
    building_grid: &mut [u16; MAP_TILES],
    units: &mut Vec<Unit>,
    projectiles: &mut Vec<SimProjectile>,
    energy_grid: &EnergyGrid,
    next_id: &mut u32,
    energy_dirty: &mut bool,
    flow_dirty: &mut bool,
    record_fx: bool,
    fx_beams: &mut Vec<FxBeam>,
    fx_bursts: &mut Vec<FxBurst>,
    fx_popups: &mut Vec<FxPopup>,
) {
    // Pre-read team buffs: does team have an active HoardVault, AetherCogs, etc.?
    let mut has_vault = [false; 2];
    let mut enemy_citadel_pos = [(22u8, 7u8), (2u8, 7u8)];
    for b in buildings.iter() {
        if b.hp <= 0 {
            continue;
        }
        if b.kind == BuildingType::Citadel {
            enemy_citadel_pos[b.owner.other().index()] = (b.x, b.y);
        }
        if b.build_ticks_left == 0 && b.powered && b.kind == BuildingType::DraconeHoardVault {
            has_vault[b.owner.index()] = true;
        }
    }

    // 1. Update projectiles
    let mut proj_i = 0;
    while proj_i < projectiles.len() {
        if projectiles[proj_i].ticks_left > 1 {
            let p = &mut projectiles[proj_i];
            let rem = p.ticks_left as i32;
            p.px += (p.tx - p.px) / rem;
            p.py += (p.ty - p.py) / rem;
            p.ticks_left -= 1;
            proj_i += 1;
            continue;
        }

        let p = projectiles.swap_remove(proj_i);
        let impact_x = p.tx;
        let impact_y = p.ty;
        let (itx, ity) = px_to_tile(impact_x, impact_y);
        let impact_idx = tile_idx(itx, ity);

        if record_fx {
            fx_bursts.push(FxBurst {
                px: impact_x as i16,
                py: impact_y as i16,
                radius: if p.splash_px > 0 {
                    (p.splash_px / 2).clamp(4, 14) as u8
                } else {
                    4
                },
                color: p.color,
                ttl: 5,
            });
        }

        let splash_r = p.splash_px.max(10);
        let splash_sq = splash_r * splash_r;

        // Damage units
        let mut hit_unit = false;
        for u in units.iter_mut() {
            if u.hp <= 0 || u.owner == p.owner {
                continue;
            }
            let d2 = dist_sq(impact_x, impact_y, u.px, u.py);
            if d2 <= splash_sq {
                if p.splash_px == 0 && hit_unit {
                    continue;
                }
                let udef = unit_def(u.kind);
                let arm = (udef.armor - if u.corrode_ticks > 0 { 2 } else { 0 }).max(0);
                let falloff = if p.splash_px > 0 && d2 > 144 { 75 } else { 100 };
                let dmg = calc_damage((p.dmg * falloff) / 100, p.dmg_class, udef.armor_class, arm);
                u.hp -= dmg;
                u.hit_flash = 3;
                hit_unit = true;
                if p.siphon > 0 {
                    teams[p.owner.index()].resources += p.siphon as i32;
                    teams[p.owner.index()].total_earned += p.siphon as i32;
                }
            }
        }

        // Damage buildings if splash or direct target tile has enemy building
        for bi in 0..buildings.len() {
            if buildings[bi].hp <= 0 || buildings[bi].owner == p.owner {
                continue;
            }
            let bx_t = buildings[bi].x;
            let by_t = buildings[bi].y;
            let (bx, by) = tile_center_px(bx_t, by_t);
            let d2 = dist_sq(impact_x, impact_y, bx, by);
            let direct = tile_idx(bx_t as usize, by_t as usize) == impact_idx && !hit_unit;
            if (p.splash_px > 0 && d2 <= (splash_r + 8) * (splash_r + 8)) || direct {
                let arm = buildings[bi].armor + buildings[bi].hearth_armor as i32;
                let dmg = calc_damage(p.dmg, p.dmg_class, ArmorClass::Fortified, arm);
                buildings[bi].hp -= dmg;
                if p.siphon > 0 {
                    teams[p.owner.index()].resources += p.siphon as i32;
                    teams[p.owner.index()].total_earned += p.siphon as i32;
                }
                trigger_blood_altars_on_siege(bx_t, by_t, p.owner, buildings, teams);
            }
        }
    }

    // 2. Step each building (construction, relocation, healing, firing, spawning)
    // Snapshot building count so newly spawned units don't affect iteration
    let b_count = buildings.len();
    for bi in 0..b_count {
        if buildings[bi].hp <= 0 {
            continue;
        }
        if buildings[bi].slow_ticks > 0 {
            buildings[bi].slow_ticks -= 1;
        }
        if buildings[bi].ability_cd > 0 {
            buildings[bi].ability_cd -= 1;
        }
        if buildings[bi].cooldown > 0 {
            buildings[bi].cooldown -= 1;
        }

        // Construction progress
        if buildings[bi].build_ticks_left > 0 {
            let mut step = 1u16;
            let bx = buildings[bi].x;
            let by = buildings[bi].y;
            let owner = buildings[bi].owner;
            for other in buildings.iter() {
                if other.hp > 0
                    && other.build_ticks_left == 0
                    && other.owner == owner
                    && other.kind == BuildingType::GnomeAetherCog
                    && chebyshev(bx, by, other.x, other.y) <= 4
                {
                    step = 2;
                    break;
                }
            }
            buildings[bi].build_ticks_left = buildings[bi].build_ticks_left.saturating_sub(step);
            if buildings[bi].build_ticks_left == 0 {
                *energy_dirty = true;
            }
            continue;
        }

        // Walker relocation step
        if let Some((tx, ty)) = buildings[bi].relocate_target {
            if buildings[bi].x == tx && buildings[bi].y == ty {
                buildings[bi].relocate_target = None;
                buildings[bi].relocate_timer = 0;
            } else {
                let mut near_cog = false;
                let owner = buildings[bi].owner;
                let bx = buildings[bi].x;
                let by = buildings[bi].y;
                for other in buildings.iter() {
                    if other.hp > 0
                        && other.build_ticks_left == 0
                        && other.owner == owner
                        && other.kind == BuildingType::GnomeAetherCog
                        && chebyshev(bx, by, other.x, other.y) <= 4
                    {
                        near_cog = true;
                        break;
                    }
                }
                let step_cd = if near_cog { 6 } else { 10 };
                buildings[bi].relocate_timer += 1;
                if buildings[bi].relocate_timer >= step_cd {
                    buildings[bi].relocate_timer = 0;
                    // Step 1 tile toward (tx, ty)
                    let mut best_step: Option<(u8, u8, i32)> = None;
                    let cur_dist = (tx as i32 - bx as i32).abs() + (ty as i32 - by as i32).abs();
                    for (dx, dy) in [(1i32, 0), (-1, 0), (0, 1), (0, -1), (1, 1), (1, -1), (-1, 1), (-1, -1)] {
                        let nx = bx as i32 + dx;
                        let ny = by as i32 + dy;
                        if nx < 0 || nx >= MAP_W as i32 || ny < 0 || ny >= MAP_H as i32 {
                            continue;
                        }
                        let ux = nx as u8;
                        let uy = ny as u8;
                        let nidx = tile_idx(ux as usize, uy as usize);
                        if terrain[nidx].blocks_ground() || building_grid[nidx] != GRID_EMPTY {
                            continue;
                        }
                        let nd = (tx as i32 - nx).abs() + (ty as i32 - ny).abs();
                        if nd < cur_dist {
                            let tie = if best_step.map_or(true, |b| nd < b.2) { true } else { false };
                            if tie {
                                best_step = Some((ux, uy, nd));
                            }
                        }
                    }
                    if let Some((nx, ny, _)) = best_step {
                        let old_idx = tile_idx(bx as usize, by as usize);
                        let new_idx = tile_idx(nx as usize, ny as usize);
                        let g_val = building_grid[old_idx];
                        building_grid[old_idx] = GRID_EMPTY;
                        building_grid[new_idx] = g_val;
                        buildings[bi].x = nx;
                        buildings[bi].y = ny;
                        *energy_dirty = true;
                        *flow_dirty = true;
                        if nx == tx && ny == ty {
                            buildings[bi].relocate_target = None;
                        }
                    } else {
                        // Blocked: stop relocation
                        buildings[bi].relocate_target = None;
                    }
                }
                continue;
            }
        }

        let b = buildings[bi];
        let def = building_def(b.kind);

        // Leone Sun Monolith passive repair aura every 15 ticks
        if b.kind == BuildingType::LeoneSunMonolith && tick % 15 == 0 {
            let (sx, sy) = tile_center_px(b.x, b.y);
            for other in buildings.iter_mut() {
                if other.hp > 0
                    && other.hp < other.max_hp
                    && other.owner == b.owner
                    && chebyshev(b.x, b.y, other.x, other.y) <= 3
                {
                    let heal = if b.level >= 2 { 10 } else { 7 };
                    other.hp = (other.hp + heal).min(other.max_hp);
                    if record_fx {
                        let (ox, oy) = tile_center_px(other.x, other.y);
                        fx_beams.push(FxBeam {
                            x0: sx as i16,
                            y0: sy as i16,
                            x1: ox as i16,
                            y1: oy as i16,
                            color: 43,
                            ttl: 4,
                        });
                    }
                }
            }
        }

        // Unit Spawning (only when powered)
        if let Some(ukind) = def.spawn_kind {
            if b.powered {
                if buildings[bi].spawn_timer > 0 {
                    buildings[bi].spawn_timer -= 1;
                }
                if buildings[bi].spawn_timer == 0 {
                    // Elder Sanctum requires stored resources >= 150 to hatch ElderWyrm
                    if b.kind == BuildingType::DraconeElderSanctum
                        && teams[b.owner.index()].resources < 150
                    {
                        buildings[bi].spawn_timer = 15;
                    } else {
                        let mut interval = if b.level >= 2 {
                            (def.spawn_interval * 80) / 100
                        } else {
                            def.spawn_interval
                        };
                        if b.kind == BuildingType::HumanMechFoundry
                            && energy_grid.net[b.owner.index()]
                                [tile_idx(b.x as usize, b.y as usize)]
                                .compute
                                > 0
                        {
                            interval = (interval * 85) / 100;
                        }
                        buildings[bi].spawn_timer = interval;
                        buildings[bi].spawn_count = buildings[bi].spawn_count.saturating_add(1);

                        // Spawn location (Tunnel Bore warp check!)
                        let (mut sx, mut sy) = (b.x, b.y);
                        if b.kind == BuildingType::DwarfTunnelBore {
                            let goal = enemy_citadel_pos[b.owner.index()];
                            let mut best_d = chebyshev(sx, sy, goal.0, goal.1);
                            let mut warped = false;
                            for other in buildings.iter() {
                                if other.hp > 0
                                    && other.build_ticks_left == 0
                                    && other.powered
                                    && other.owner == b.owner
                                    && other.kind == BuildingType::DwarfTunnelBore
                                {
                                    let d = chebyshev(other.x, other.y, goal.0, goal.1);
                                    if d < best_d {
                                        best_d = d;
                                        sx = other.x;
                                        sy = other.y;
                                        warped = true;
                                    }
                                }
                            }
                            if warped && record_fx {
                                let (ox, oy) = tile_center_px(b.x, b.y);
                                let (wx, wy) = tile_center_px(sx, sy);
                                fx_beams.push(FxBeam {
                                    x0: ox as i16,
                                    y0: oy as i16,
                                    x1: wx as i16,
                                    y1: wy as i16,
                                    color: 46,
                                    ttl: 6,
                                });
                            }
                        }

                        let (px, py) = tile_center_px(sx, sy);
                        let udef = unit_def(ukind);
                        let mut hp_max = udef.max_hp;
                        if ukind == UnitKind::IroncladGolem {
                            let kt = teams[b.owner.index()].knowledge_tier as i32;
                            hp_max = (hp_max * (100 + kt * 7)) / 100;
                        }
                        if b.level >= 2 {
                            hp_max = (hp_max * 120) / 100;
                        }
                        let (dir_dx, dir_dy) = if ukind == UnitKind::LivingBoulder {
                            let goal = enemy_citadel_pos[b.owner.index()];
                            let dx = goal.0 as i32 - sx as i32;
                            let dy = goal.1 as i32 - sy as i32;
                            if dx.abs() >= dy.abs() {
                                (dx.signum() as i8, 0)
                            } else {
                                (0, dy.signum() as i8)
                            }
                        } else if b.owner == TeamId::P1 {
                            (1, 0)
                        } else {
                            (-1, 0)
                        };

                        *next_id += 1;
                        units.push(Unit {
                            id: *next_id,
                            owner: b.owner,
                            kind: ukind,
                            px,
                            py,
                            hp: hp_max,
                            max_hp: hp_max,
                            cooldown: 0,
                            slow_ticks: 0,
                            corrode_ticks: 0,
                            dir_dx,
                            dir_dy,
                            hit_flash: 0,
                        });

                        // Orc War Den spawns an extra Siege Ram every 3rd wave
                        if b.kind == BuildingType::OrcWarDen && buildings[bi].spawn_count % 3 == 0 {
                            let ram_def = unit_def(UnitKind::OrcRam);
                            *next_id += 1;
                            units.push(Unit {
                                id: *next_id,
                                owner: b.owner,
                                kind: UnitKind::OrcRam,
                                px: (px - 4).max(4),
                                py,
                                hp: ram_def.max_hp,
                                max_hp: ram_def.max_hp,
                                cooldown: 0,
                                slow_ticks: 0,
                                corrode_ticks: 0,
                                dir_dx,
                                dir_dy,
                                hit_flash: 0,
                            });
                        }
                    }
                }
            }
        }

        // Tower Weapon Firing
        if def.damage <= 0 || buildings[bi].cooldown > 0 {
            continue;
        }

        // Range computation
        let b_idx_tile = tile_idx(b.x as usize, b.y as usize);
        let on_high = terrain[b_idx_tile].is_high();
        let mut range_t = def.range_tiles as i32;
        if on_high {
            range_t += 1;
        }
        let range_px = range_t * TILE_PX + 6;
        let min_range_px = (def.min_range_tiles as i32) * TILE_PX;
        let (bx_px, by_px) = tile_center_px(b.x, b.y);

        // Compute effective damage
        let mut raw_dmg = def.damage;
        if b.level >= 2 {
            raw_dmg = (raw_dmg * 135) / 100;
        }
        if b.kind.faction() == Some(Faction::Dwarves) {
            let kt = teams[b.owner.index()].knowledge_tier as i32;
            raw_dmg = (raw_dmg * (100 + kt * 8)) / 100;
        }
        if b.fury_stacks > 0 {
            raw_dmg = (raw_dmg * (100 + (b.fury_stacks as i32) * 25)) / 100;
        }
        if b.kind == BuildingType::LeoneSolarPillar && on_high {
            raw_dmg = (raw_dmg * 120) / 100;
        }
        if b.kind.faction() == Some(Faction::Dracones) && has_vault[b.owner.index()] {
            let greed = ((teams[b.owner.index()].resources / 100) * 10).clamp(0, 50);
            raw_dmg = (raw_dmg * (100 + greed)) / 100;
        }
        if b.kind == BuildingType::ElvenAstralLens {
            raw_dmg += (b.net_links as i32) * 10;
        }

        // Select target (Units prioritized unless OrbitalUplink sniper)
        let mut target_unit: Option<usize> = None;
        let mut target_bldg: Option<usize> = None;

        if b.kind == BuildingType::HumanOrbitalUplink {
            // Precision sniper: prefers Eco / Factory / Apex enemy buildings!
            let mut best_score = i32::MIN;
            for (obi, ob) in buildings.iter().enumerate() {
                if ob.hp <= 0 || ob.owner == b.owner {
                    continue;
                }
                let (ox, oy) = tile_center_px(ob.x, ob.y);
                let d2 = dist_sq(bx_px, by_px, ox, oy);
                if d2 <= range_px * range_px && d2 >= min_range_px * min_range_px {
                    let prio = match building_def(ob.kind).slot_priority() {
                        Some(BuildingSlot::Apex) => 400,
                        Some(BuildingSlot::Factory) => 320,
                        Some(BuildingSlot::Eco) => 280,
                        Some(BuildingSlot::Defense) => 200,
                        Some(BuildingSlot::Infra) => 140,
                        None => 100,
                    };
                    let score = prio - d2 / 256;
                    if score > best_score {
                        best_score = score;
                        target_bldg = Some(obi);
                    }
                }
            }
        }

        if target_bldg.is_none() {
            let mut best_d2 = range_px * range_px + 1;
            for (ui, u) in units.iter().enumerate() {
                if u.hp <= 0 || u.owner == b.owner {
                    continue;
                }
                let fly = u.kind.is_flying();
                if fly && !def.hits_air {
                    continue;
                }
                if !fly && !def.hits_ground {
                    continue;
                }
                let d2 = dist_sq(bx_px, by_px, u.px, u.py);
                if d2 <= range_px * range_px && d2 >= min_range_px * min_range_px && d2 < best_d2 {
                    best_d2 = d2;
                    target_unit = Some(ui);
                }
            }
        }

        if target_unit.is_none() && target_bldg.is_none() && def.hits_buildings {
            let mut best_d2 = range_px * range_px + 1;
            for (obi, ob) in buildings.iter().enumerate() {
                if ob.hp <= 0 || ob.owner == b.owner {
                    continue;
                }
                let (ox, oy) = tile_center_px(ob.x, ob.y);
                let d2 = dist_sq(bx_px, by_px, ox, oy);
                if d2 <= range_px * range_px && d2 >= min_range_px * min_range_px && d2 < best_d2 {
                    best_d2 = d2;
                    target_bldg = Some(obi);
                }
            }
        }

        // Special: Elven Prism Spire refraction if no direct target or even with bounce!
        let mut fired = false;
        let target_pos: Option<(i32, i32)> = if let Some(ui) = target_unit {
            Some((units[ui].px, units[ui].py))
        } else if let Some(obi) = target_bldg {
            Some(tile_center_px(buildings[obi].x, buildings[obi].y))
        } else {
            None
        };

        if let Some((tx_px, ty_px)) = target_pos {
            fired = true;
            match b.kind {
                BuildingType::ElvenAstralLens => {
                    // Piercing ray up to r=9 hitting ALL enemies along the line!
                    let dx = tx_px - bx_px;
                    let dy = ty_px - by_px;
                    let len_sq = dx * dx + dy * dy;
                    let max_len = range_px;
                    if len_sq > 0 {
                        let dist_i = isqrt_i64(len_sq as i64).max(1);
                        let ex = bx_px + (dx * max_len) / dist_i;
                        let ey = by_px + (dy * max_len) / dist_i;
                        for u in units.iter_mut() {
                            if u.hp <= 0 || u.owner == b.owner {
                                continue;
                            }
                            if point_near_segment(u.px, u.py, bx_px, by_px, ex, ey, 14) {
                                let udef = unit_def(u.kind);
                                let arm = (udef.armor - if u.corrode_ticks > 0 { 2 } else { 0 }).max(0);
                                let dmg = calc_damage(raw_dmg, DamageClass::Arcane, udef.armor_class, arm);
                                u.hp -= dmg;
                                u.hit_flash = 3;
                            }
                        }
                        for obi in 0..buildings.len() {
                            if buildings[obi].hp <= 0 || buildings[obi].owner == b.owner {
                                continue;
                            }
                            let (ox, oy) = tile_center_px(buildings[obi].x, buildings[obi].y);
                            if point_near_segment(ox, oy, bx_px, by_px, ex, ey, 14) {
                                let arm = buildings[obi].armor + buildings[obi].hearth_armor as i32;
                                let dmg = calc_damage(raw_dmg, DamageClass::Arcane, ArmorClass::Fortified, arm);
                                buildings[obi].hp -= dmg;
                            }
                        }
                        if record_fx {
                            fx_beams.push(FxBeam {
                                x0: bx_px as i16,
                                y0: by_px as i16,
                                x1: ex.clamp(0, PLAYFIELD_W - 1) as i16,
                                y1: ey.clamp(0, PLAYFIELD_H - 1) as i16,
                                color: 29,
                                ttl: 5,
                            });
                        }
                    }
                }
                BuildingType::ElvenPrismSpire => {
                    // Direct hit + refraction chain through up to 2 allied Ley nodes/Prisms if available!
                    let dmg_boost = raw_dmg + (raw_dmg * (b.net_links.min(2) as i32) * 30) / 100;
                    apply_direct_hit(
                        b.owner,
                        dmg_boost,
                        DamageClass::Arcane,
                        0,
                        false,
                        target_unit,
                        target_bldg,
                        units,
                        buildings,
                        teams,
                        energy_grid,
                    );
                    if record_fx {
                        // Draw refraction visual via linked Ley neighbor if present
                        let mut linked_pt: Option<(i32, i32)> = None;
                        for link in &energy_grid.ley_links {
                            if link.owner == b.owner {
                                if link.x0 == b.x && link.y0 == b.y {
                                    linked_pt = Some(tile_center_px(link.x1, link.y1));
                                    break;
                                } else if link.x1 == b.x && link.y1 == b.y {
                                    linked_pt = Some(tile_center_px(link.x0, link.y0));
                                    break;
                                }
                            }
                        }
                        if let Some((lx, ly)) = linked_pt {
                            fx_beams.push(FxBeam {
                                x0: lx as i16,
                                y0: ly as i16,
                                x1: bx_px as i16,
                                y1: by_px as i16,
                                color: 28,
                                ttl: 4,
                            });
                        }
                        fx_beams.push(FxBeam {
                            x0: bx_px as i16,
                            y0: by_px as i16,
                            x1: tx_px as i16,
                            y1: ty_px as i16,
                            color: 29,
                            ttl: 4,
                        });
                    }
                }
                BuildingType::GnomeTeslaContraption => {
                    // Chain lightning: hits primary + up to 3 jumps within 48px
                    apply_direct_hit(
                        b.owner,
                        raw_dmg,
                        DamageClass::Arcane,
                        0,
                        false,
                        target_unit,
                        target_bldg,
                        units,
                        buildings,
                        teams,
                        energy_grid,
                    );
                    if record_fx {
                        fx_beams.push(FxBeam {
                            x0: bx_px as i16,
                            y0: by_px as i16,
                            x1: tx_px as i16,
                            y1: ty_px as i16,
                            color: 31,
                            ttl: 4,
                        });
                    }
                    let mut last_x = tx_px;
                    let mut last_y = ty_px;
                    let mut hit_ids: [u32; 4] = [0; 4];
                    if let Some(ui) = target_unit {
                        hit_ids[0] = units[ui].id;
                    } else if let Some(obi) = target_bldg {
                        hit_ids[0] = buildings[obi].id;
                    }
                    let mut jump_dmg = (raw_dmg * 70) / 100;
                    for hop in 1..4 {
                        let mut next_u: Option<usize> = None;
                        let mut best_d2 = 52 * 52;
                        for (ui, u) in units.iter().enumerate() {
                            if u.hp <= 0 || u.owner == b.owner || hit_ids.contains(&u.id) {
                                continue;
                            }
                            let d2 = dist_sq(last_x, last_y, u.px, u.py);
                            if d2 < best_d2 {
                                best_d2 = d2;
                                next_u = Some(ui);
                            }
                        }
                        if let Some(nui) = next_u {
                            hit_ids[hop] = units[nui].id;
                            let udef = unit_def(units[nui].kind);
                            let arm = (udef.armor - if units[nui].corrode_ticks > 0 { 2 } else { 0 }).max(0);
                            units[nui].hp -= calc_damage(jump_dmg, DamageClass::Arcane, udef.armor_class, arm);
                            units[nui].hit_flash = 3;
                            if record_fx {
                                fx_beams.push(FxBeam {
                                    x0: last_x as i16,
                                    y0: last_y as i16,
                                    x1: units[nui].px as i16,
                                    y1: units[nui].py as i16,
                                    color: 31,
                                    ttl: 4,
                                });
                            }
                            last_x = units[nui].px;
                            last_y = units[nui].py;
                            jump_dmg = (jump_dmg * 75) / 100;
                        } else {
                            break;
                        }
                    }
                }
                BuildingType::DwarfHowitzer | BuildingType::OrcDoomCauldron | BuildingType::LeoneSolsticeColossus => {
                    // Projectile with splash
                    let travel = if b.kind == BuildingType::DwarfHowitzer { 7 } else { 5 };
                    let color = match b.kind {
                        BuildingType::DwarfHowitzer => 43,
                        BuildingType::OrcDoomCauldron => 40,
                        _ => 42,
                    };
                    projectiles.push(SimProjectile {
                        owner: b.owner,
                        source_kind: b.kind,
                        dmg: raw_dmg,
                        dmg_class: def.dmg_class,
                        splash_px: def.splash_px,
                        siphon: 0,
                        px: bx_px,
                        py: by_px,
                        tx: tx_px,
                        ty: ty_px,
                        ticks_left: travel,
                        total_ticks: travel,
                        color,
                        arcing: true,
                    });
                }
                BuildingType::OrcSpikeBastion => {
                    // Instant close-range AoE flak burst
                    let splash_sq = def.splash_px * def.splash_px;
                    for u in units.iter_mut() {
                        if u.hp <= 0 || u.owner == b.owner {
                            continue;
                        }
                        if dist_sq(tx_px, ty_px, u.px, u.py) <= splash_sq {
                            let udef = unit_def(u.kind);
                            let arm = (udef.armor - if u.corrode_ticks > 0 { 2 } else { 0 }).max(0);
                            u.hp -= calc_damage(raw_dmg, DamageClass::Kinetic, udef.armor_class, arm);
                            u.hit_flash = 2;
                        }
                    }
                    if target_unit.is_none() {
                        if let Some(obi) = target_bldg {
                            let arm = buildings[obi].armor + buildings[obi].hearth_armor as i32;
                            buildings[obi].hp -= calc_damage(raw_dmg, DamageClass::Kinetic, ArmorClass::Fortified, arm);
                        }
                    }
                    if record_fx {
                        fx_beams.push(FxBeam {
                            x0: bx_px as i16,
                            y0: by_px as i16,
                            x1: tx_px as i16,
                            y1: ty_px as i16,
                            color: 40,
                            ttl: 3,
                        });
                        fx_bursts.push(FxBurst {
                            px: tx_px as i16,
                            py: ty_px as i16,
                            radius: 8,
                            color: 41,
                            ttl: 4,
                        });
                    }
                }
                _ => {
                    // Direct beams & bolts (Autocannon, OrbitalUplink, BastionCannon, WalkerTurret, SolarPillar, TaxObelisk, ElderSanctum, Citadel)
                    let mut final_dmg = raw_dmg;
                    if b.kind == BuildingType::LeoneSolarPillar {
                        if let Some(ui) = target_unit {
                            let (utx, uty) = px_to_tile(units[ui].px, units[ui].py);
                            if units[ui].kind.is_flying()
                                || energy_grid.net[units[ui].owner.index()][tile_idx(utx, uty)].neg_force > 0
                            {
                                final_dmg = (final_dmg * 130) / 100;
                            }
                        } else if let Some(obi) = target_bldg {
                            let ot = tile_idx(buildings[obi].x as usize, buildings[obi].y as usize);
                            if energy_grid.net[buildings[obi].owner.index()][ot].neg_force > 0 {
                                final_dmg = (final_dmg * 130) / 100;
                            }
                        }
                    }
                    let siphon = if b.kind == BuildingType::DraconeTaxObelisk { 4 } else { 0 };
                    let corrode = b.kind == BuildingType::DraconeElderSanctum;
                    apply_direct_hit(
                        b.owner,
                        final_dmg,
                        def.dmg_class,
                        siphon,
                        corrode,
                        target_unit,
                        target_bldg,
                        units,
                        buildings,
                        teams,
                        energy_grid,
                    );
                    if record_fx {
                        let col = match b.kind {
                            BuildingType::HumanOrbitalUplink => 43,
                            BuildingType::HumanAutocannon => 44,
                            BuildingType::DwarfBastionCannon => 46,
                            BuildingType::GnomeWalkerTurret => 31,
                            BuildingType::LeoneSolarPillar => 42,
                            BuildingType::DraconeTaxObelisk => 54,
                            BuildingType::DraconeElderSanctum => 53,
                            _ => 59,
                        };
                        fx_beams.push(FxBeam {
                            x0: bx_px as i16,
                            y0: by_px as i16,
                            x1: tx_px as i16,
                            y1: ty_px as i16,
                            color: col,
                            ttl: if b.kind == BuildingType::HumanOrbitalUplink { 6 } else { 3 },
                        });
                        if siphon > 0 {
                            push_popup(fx_popups, bx_px, by_px - 8, "+4", 42);
                        }
                    }
                }
            }
        } else if b.kind == BuildingType::ElvenPrismSpire {
            // Relay refraction shot! If Prism has no direct target, check if an inline connected Conduit/Prism has a target!
            'refract: for link in &energy_grid.ley_links {
                if link.owner != b.owner {
                    continue;
                }
                let other_tile = if link.x0 == b.x && link.y0 == b.y {
                    Some((link.x1, link.y1))
                } else if link.x1 == b.x && link.y1 == b.y {
                    Some((link.x0, link.y0))
                } else {
                    None
                };
                if let Some((ox, oy)) = other_tile {
                    let (opx, opy) = tile_center_px(ox, oy);
                    for u in units.iter_mut() {
                        if u.hp > 0 && u.owner != b.owner && dist_sq(opx, opy, u.px, u.py) <= 58 * 58 {
                            let udef = unit_def(u.kind);
                            let arm = (udef.armor - if u.corrode_ticks > 0 { 2 } else { 0 }).max(0);
                            let dmg = calc_damage((raw_dmg * 130) / 100, DamageClass::Arcane, udef.armor_class, arm);
                            u.hp -= dmg;
                            u.hit_flash = 3;
                            fired = true;
                            if record_fx {
                                fx_beams.push(FxBeam {
                                    x0: bx_px as i16,
                                    y0: by_px as i16,
                                    x1: opx as i16,
                                    y1: opy as i16,
                                    color: 28,
                                    ttl: 4,
                                });
                                fx_beams.push(FxBeam {
                                    x0: opx as i16,
                                    y0: opy as i16,
                                    x1: u.px as i16,
                                    y1: u.py as i16,
                                    color: 29,
                                    ttl: 4,
                                });
                            }
                            break 'refract;
                        }
                    }
                }
            }
        }

        if fired {
            buildings[bi].last_fire_tick = tick;
            let mut cd = def.fire_cd as i32;
            if !b.powered {
                cd *= 3;
            }
            if b.kind.faction() == Some(Faction::Humans) && b.net_links > 0 {
                cd = (cd * (100 - (b.net_links as i32) * 10)) / 100;
            }
            if b.gizmo_buff {
                cd = (cd * 80) / 100;
                // Mini chain-lightning spark to 1 nearby enemy within 44px!
                if let Some((sx, sy)) = target_pos {
                    for u in units.iter_mut() {
                        if u.hp > 0 && u.owner != b.owner && dist_sq(sx, sy, u.px, u.py) <= 44 * 44 {
                            let udef = unit_def(u.kind);
                            let arm = (udef.armor - if u.corrode_ticks > 0 { 2 } else { 0 }).max(0);
                            u.hp -= calc_damage(7, DamageClass::Arcane, udef.armor_class, arm);
                            u.hit_flash = 2;
                            if record_fx {
                                fx_beams.push(FxBeam {
                                    x0: sx as i16,
                                    y0: sy as i16,
                                    x1: u.px as i16,
                                    y1: u.py as i16,
                                    color: 35,
                                    ttl: 3,
                                });
                            }
                            break;
                        }
                    }
                }
                // Trigger adjacent Tinker Lab R&D resource!
                for other in buildings.iter_mut() {
                    if other.hp > 0
                        && other.build_ticks_left == 0
                        && other.owner == b.owner
                        && other.kind == BuildingType::GnomeTinkerLab
                        && other.ability_cd == 0
                        && manhattan(b.x, b.y, other.x, other.y) == 1
                    {
                        other.ability_cd = 15;
                        teams[b.owner.index()].resources += 1;
                        teams[b.owner.index()].total_earned += 1;
                    }
                }
            }
            if b.slow_ticks > 0 {
                cd = (cd * 150) / 100;
            }
            buildings[bi].cooldown = cd.max(4) as u16;
        }
    }

    // Check Blood Altar triggers on unit deaths & remove dead units
    let mut ui = 0;
    while ui < units.len() {
        if units[ui].hp <= 0 {
            let dead = units.swap_remove(ui);
            teams[dead.owner.other().index()].kills =
                teams[dead.owner.other().index()].kills.saturating_add(1);
            let (utx, uty) = px_to_tile(dead.px, dead.py);
            for b in buildings.iter() {
                if b.hp > 0
                    && b.build_ticks_left == 0
                    && b.powered
                    && b.kind == BuildingType::OrcBloodAltar
                    && chebyshev(utx as u8, uty as u8, b.x, b.y) <= 6
                {
                    teams[b.owner.index()].resources += 4;
                    teams[b.owner.index()].total_earned += 4;
                    if record_fx {
                        let (bx, by) = tile_center_px(b.x, b.y);
                        fx_beams.push(FxBeam {
                            x0: dead.px as i16,
                            y0: dead.py as i16,
                            x1: bx as i16,
                            y1: by as i16,
                            color: 41,
                            ttl: 5,
                        });
                        push_popup(fx_popups, bx, by - 6, "+4", 40);
                    }
                }
            }
        } else {
            ui += 1;
        }
    }
}

fn apply_direct_hit(
    attacker: TeamId,
    raw_dmg: i32,
    dmg_class: DamageClass,
    siphon: i16,
    corrode: bool,
    target_unit: Option<usize>,
    target_bldg: Option<usize>,
    units: &mut [Unit],
    buildings: &mut [Building],
    teams: &mut [TeamState; 2],
    energy_grid: &EnergyGrid,
) {
    if let Some(ui) = target_unit {
        let udef = unit_def(units[ui].kind);
        let (utx, uty) = px_to_tile(units[ui].px, units[ui].py);
        let in_neg = energy_grid.net[attacker.index()][tile_idx(utx, uty)].neg_force > 0;
        let arm_pen = if units[ui].corrode_ticks > 0 { 2 } else { 0 } + if in_neg { 1 } else { 0 };
        let arm = (udef.armor - arm_pen).max(0);
        let dmg = calc_damage(raw_dmg, dmg_class, udef.armor_class, arm);
        units[ui].hp -= dmg;
        units[ui].hit_flash = 3;
        if corrode {
            units[ui].corrode_ticks = 45;
        }
        if siphon > 0 {
            teams[attacker.index()].resources += siphon as i32;
            teams[attacker.index()].total_earned += siphon as i32;
        }
    } else if let Some(obi) = target_bldg {
        let in_neg = energy_grid.net[attacker.index()]
            [tile_idx(buildings[obi].x as usize, buildings[obi].y as usize)]
            .neg_force
            > 0;
        let arm = (buildings[obi].armor + buildings[obi].hearth_armor as i32
            - if in_neg { 1 } else { 0 })
        .max(0);
        let dmg = calc_damage(raw_dmg, dmg_class, ArmorClass::Fortified, arm);
        buildings[obi].hp -= dmg;
        if siphon > 0 {
            teams[attacker.index()].resources += siphon as i32;
            teams[attacker.index()].total_earned += siphon as i32;
        }
        trigger_blood_altars_on_siege(buildings[obi].x, buildings[obi].y, attacker, buildings, teams);
    }
}

#[inline]
fn isqrt_i64(n: i64) -> i32 {
    if n <= 0 {
        return 0;
    }
    let mut x = n as u64;
    let mut y = (x + 1) >> 1;
    while y < x {
        x = y;
        y = (x + (n as u64) / x) >> 1;
    }
    x as i32
}

fn trigger_blood_altars_on_siege(
    bx: u8,
    by: u8,
    attacker: TeamId,
    buildings: &mut [Building],
    teams: &mut [TeamState; 2],
) {
    for b in buildings.iter_mut() {
        if b.hp > 0
            && b.build_ticks_left == 0
            && b.powered
            && b.owner == attacker
            && b.kind == BuildingType::OrcBloodAltar
            && b.ability_cd == 0
            && chebyshev(bx, by, b.x, b.y) <= 6
        {
            b.ability_cd = 15;
            teams[attacker.index()].resources += 3;
            teams[attacker.index()].total_earned += 3;
        }
    }
}

fn point_near_segment(px: i32, py: i32, x0: i32, y0: i32, x1: i32, y1: i32, tol: i32) -> bool {
    let dx = x1 - x0;
    let dy = y1 - y0;
    let len_sq = dx * dx + dy * dy;
    if len_sq == 0 {
        return dist_sq(px, py, x0, y0) <= tol * tol;
    }
    let dot = (px - x0) * dx + (py - y0) * dy;
    if dot < 0 || dot > len_sq {
        return false;
    }
    // Perpendicular distance squared = ((px-x0)*dy - (py-y0)*dx)^2 / len_sq <= tol^2
    let cross = (px - x0) as i64 * (dy as i64) - (py - y0) as i64 * (dx as i64);
    (cross * cross) <= (tol as i64) * (tol as i64) * (len_sq as i64)
}

pub fn push_popup(fx_popups: &mut Vec<FxPopup>, px: i32, py: i32, s: &str, color: u8) {
    let mut text = [b' '; 8];
    let bytes = s.as_bytes();
    let len = bytes.len().min(8);
    text[..len].copy_from_slice(&bytes[..len]);
    fx_popups.push(FxPopup {
        px: px as i16,
        py: py as i16,
        text,
        len: len as u8,
        color,
        ttl: 18,
    });
}

trait SlotPrio {
    fn slot_priority(&self) -> Option<BuildingSlot>;
}

impl SlotPrio for super::defs::BuildingDef {
    fn slot_priority(&self) -> Option<BuildingSlot> {
        self.kind.slot()
    }
}
