use crate::sim::defs::{building_cost, building_def, upgrade_cost};
use crate::sim::energy::{emitter_radius, has_line_of_sight, media_array_income};
use crate::sim::types::*;
use crate::sim::units::px_to_tile;
use crate::sim::GameState;

#[derive(Clone, Copy, Debug)]
pub struct ScoredAction {
    pub action: Action,
    pub prior: i32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StrategicBias {
    Balanced,
    Rush,
    Eco,
    TurtleTech,
}

pub fn generate_candidates(
    state: &GameState,
    team: TeamId,
    bias: StrategicBias,
    max_candidates: usize,
) -> Vec<ScoredAction> {
    let mut out: Vec<ScoredAction> = Vec::with_capacity(48);
    let ts = &state.teams[team.index()];
    let enemy = team.other();

    // Always include NoOp (saving for tech / compound interest!)
    let mut noop_prior = 15;
    if ts.config.has(Faction::Dracones)
        && state.buildings.iter().any(|b| {
            b.hp > 0 && b.owner == team && b.kind == BuildingType::DraconeHoardVault
        })
        && ts.resources < 240
    {
        noop_prior += 35;
    }
    if ts.resources >= 50 && ts.resources < 135 {
        // Might be saving for Factory or Apex
        noop_prior += 20;
    }
    out.push(ScoredAction {
        action: Action::NoOp,
        prior: noop_prior,
    });

    // Analyze tactical context
    let mut my_citadel = if team == TeamId::P1 { (2u8, 7u8) } else { (22u8, 7u8) };
    let mut enemy_citadel = if team == TeamId::P1 { (22u8, 7u8) } else { (2u8, 7u8) };
    let mut has_enemy_citadel = false;

    let mut my_counts = [0i32; 5]; // Infra, Eco, Def, Fac, Apex
    let mut my_unpowered_near = Vec::new();
    let mut my_unpowered_need_compute = Vec::new();
    let mut enemy_front_x = if team == TeamId::P1 { 24i32 } else { 0i32 };
    let mut nearest_threat_to_citadel: Option<(u8, u8, i32)> = None;

    for b in &state.buildings {
        if b.hp <= 0 {
            continue;
        }
        if b.owner == team {
            if b.kind == BuildingType::Citadel {
                my_citadel = (b.x, b.y);
            } else if let Some(slot) = b.kind.slot() {
                my_counts[slot as usize] += 1;
            }
            if !b.powered && b.build_ticks_left == 0 {
                let cell = state.energy_grid.net[team.index()][tile_idx(b.x as usize, b.y as usize)];
                let req = building_def(b.kind).req_energy;
                let missing_compute = req.contains(EnergyMask::COMPUTE) && cell.compute <= 0;
                let missing_other = (req.contains(EnergyMask::MECH) && cell.mech <= 0)
                    || (req.contains(EnergyMask::MANA) && cell.mana <= 0)
                    || (req.contains(EnergyMask::POS_FORCE) && cell.pos_force <= 0)
                    || (req.contains(EnergyMask::NEG_FORCE) && cell.neg_force <= 0);
                if missing_compute && !missing_other {
                    my_unpowered_need_compute.push((b.x, b.y));
                } else {
                    my_unpowered_near.push((b.x, b.y));
                }
            }
        } else {
            if b.kind == BuildingType::Citadel {
                enemy_citadel = (b.x, b.y);
                has_enemy_citadel = true;
            }
            if team == TeamId::P1 {
                enemy_front_x = enemy_front_x.min(b.x as i32);
            } else {
                enemy_front_x = enemy_front_x.max(b.x as i32);
            }
        }
    }
    let _ = has_enemy_citadel;

    let mut enemy_units_count = 0i32;
    let mut enemy_flying_count = 0i32;
    for u in &state.units {
        if u.hp <= 0 || u.owner != enemy {
            continue;
        }
        enemy_units_count += 1;
        if u.kind.is_flying() {
            enemy_flying_count += 1;
        }
        let (utx, uty) = px_to_tile(u.px, u.py);
        let d = chebyshev(utx as u8, uty as u8, my_citadel.0, my_citadel.1) as i32;
        if nearest_threat_to_citadel.map_or(true, |t| d < t.2) {
            nearest_threat_to_citadel = Some((utx as u8, uty as u8, d));
        }
    }

    let under_immediate_threat = nearest_threat_to_citadel.map_or(false, |t| t.2 <= 7);
    let forward_dir: i32 = if team == TeamId::P1 { 1 } else { -1 };

    // Precompute valid build tiles in territory
    let mut valid_tiles: Vec<(u8, u8)> = Vec::with_capacity(96);
    for y in 0..MAP_H {
        for x in 0..MAP_W {
            let ux = x as u8;
            let uy = y as u8;
            let idx = tile_idx(x, y);
            if state.building_grid[idx] == GRID_EMPTY
                && state.terrain[idx] != Terrain::Chasm
                && state.is_in_build_territory(team, ux, uy)
            {
                valid_tiles.push((ux, uy));
            }
        }
    }

    // 1. Building candidates per owned faction & slot
    for &faction in ts.config.as_slice() {
        for slot in BuildingSlot::ALL {
            let kind = BuildingType::from_faction_slot(faction, slot);
            let cost = building_cost(kind, ts.config);
            if ts.resources < cost {
                continue;
            }
            let def = building_def(kind);
            if def.require_eco && !state.has_eco_building(team, faction) {
                continue;
            }

            // Macro priority weights
            let mut macro_w = match slot {
                BuildingSlot::Infra => {
                    let mut w = if !my_unpowered_near.is_empty() {
                        95
                    } else if my_counts[0] < 3 {
                        58 - my_counts[0] * 10
                    } else {
                        26
                    };
                    if bias == StrategicBias::Rush && my_counts[0] < 2 {
                        w += 24;
                    }
                    w
                }
                BuildingSlot::Eco => {
                    let target_eco = match bias {
                        StrategicBias::Eco => 4,
                        StrategicBias::Rush => 0,
                        StrategicBias::TurtleTech => 2,
                        StrategicBias::Balanced => 2,
                    };
                    let need_media_compute = kind == BuildingType::HumanMediaArray
                        && !my_unpowered_need_compute.is_empty();
                    if need_media_compute {
                        88
                    } else if bias == StrategicBias::Rush {
                        -20
                    } else if under_immediate_threat && my_counts[2] == 0 {
                        15
                    } else if my_counts[1] < target_eco {
                        let mut w = 92 - my_counts[1] * 16;
                        if kind == BuildingType::HumanMediaArray
                            && my_counts[0] >= 1
                            && my_counts[1] < 2
                        {
                            w += 22;
                        }
                        w
                    } else {
                        28
                    }
                }
                BuildingSlot::Defense => {
                    let mut w = 68 - my_counts[2] * 5;
                    if under_immediate_threat {
                        w += 55;
                    }
                    if enemy_units_count > 2 {
                        w += 25;
                    }
                    if enemy_flying_count > 0 && def.hits_air {
                        w += 20;
                    }
                    if bias == StrategicBias::TurtleTech {
                        w += 32;
                    } else if bias == StrategicBias::Rush {
                        w -= 10;
                    }
                    w
                }
                BuildingSlot::Factory => {
                    let mut w = 78 - my_counts[3] * 14;
                    if bias == StrategicBias::Rush {
                        w += 50;
                    }
                    if bias == StrategicBias::Eco && my_counts[1] < 2 {
                        w -= 20;
                    }
                    w
                }
                BuildingSlot::Apex => {
                    let mut w = 88 - my_counts[4] * 18;
                    if bias == StrategicBias::TurtleTech {
                        w += 25;
                    }
                    if under_immediate_threat && my_counts[2] < 2 {
                        w -= 25;
                    }
                    w
                }
            };

            if under_immediate_threat
                && slot == BuildingSlot::Eco
                && !(kind == BuildingType::HumanMediaArray && !my_unpowered_need_compute.is_empty())
            {
                macro_w -= 25;
            }

            // Pick top 2-3 placement tiles for this building type
            let mut best_tiles: [(u8, u8, i32); 3] = [(0, 0, i32::MIN); 3];
            for &(tx, ty) in &valid_tiles {
                let tidx = tile_idx(tx as usize, ty as usize);
                let terr = state.terrain[tidx];
                if terr == Terrain::RaisedStone && faction != Faction::Leones {
                    continue;
                }
                let cell = state.energy_grid.net[team.index()][tidx];
                let powered_here = cell.satisfies(def.req_energy);
                if slot != BuildingSlot::Infra && !powered_here {
                    // Strongly avoid building unpowered unless no choice
                    continue;
                }

                let mut pos_score = if ts.config.count > 1 && powered_here && slot != BuildingSlot::Infra {
                    10
                } else {
                    0
                };
                let fwd = (tx as i32 - my_citadel.0 as i32) * forward_dir;
                let mid_closeness = 12 - (tx as i32 - 12).abs();

                match slot {
                    BuildingSlot::Infra => {
                        // Power unpowered buildings, claim Ore/Ley/HighGround, expand forward
                        for &(ux, uy) in &my_unpowered_near {
                            if chebyshev(tx, ty, ux, uy) <= def.emit_radius {
                                pos_score += 55;
                            }
                        }
                        if terr == Terrain::LeySpring && faction == Faction::Elves {
                            pos_score += 38;
                        }
                        if terr == Terrain::OreVein && faction == Faction::Dwarves {
                            pos_score += 24;
                        }
                        // Avoid stacking right on top of existing same-kind infra
                        let mut too_close = false;
                        for b in &state.buildings {
                            if b.hp > 0 && b.owner == team && b.kind == kind {
                                let d = chebyshev(tx, ty, b.x, b.y);
                                if d <= 2 {
                                    too_close = true;
                                } else if faction == Faction::Humans && d <= 4 {
                                    pos_score += 18;
                                } else if faction == Faction::Elves {
                                    let dx = b.x as i32 - tx as i32;
                                    let dy = b.y as i32 - ty as i32;
                                    if d <= 7 && (dx == 0 || dy == 0 || dx.abs() == dy.abs()) {
                                        pos_score += 20;
                                    }
                                }
                            }
                        }
                        if too_close {
                            pos_score -= 35;
                        }
                        // Push forward carefully without suiciding in front of enemy towers
                        let safe_fwd = fwd.clamp(-2, 10);
                        pos_score += safe_fwd * 6;
                        if (tx as i32 - enemy_front_x) * forward_dir > -2 {
                            pos_score -= 35;
                        }
                        // Polarity clash counter: push SunMonolith/VoidBrazier near enemy force
                        if matches!(
                            kind,
                            BuildingType::LeoneSunMonolith | BuildingType::DraconeVoidBrazier
                        ) {
                            pos_score += mid_closeness * 3;
                        }
                    }
                    BuildingSlot::Eco => {
                        match kind {
                            BuildingType::HumanMediaArray => {
                                let dummy = Building {
                                    id: 0,
                                    owner: team,
                                    kind,
                                    x: tx,
                                    y: ty,
                                    hp: 100,
                                    max_hp: 100,
                                    armor: 0,
                                    level: 1,
                                    build_ticks_left: 0,
                                    cooldown: 0,
                                    spawn_timer: 0,
                                    spawn_count: 0,
                                    powered: true,
                                    net_links: 0,
                                    gizmo_buff: false,
                                    hearth_armor: 0,
                                    fury_stacks: 0,
                                    slow_ticks: 0,
                                    relocate_target: None,
                                    relocate_timer: 0,
                                    ability_cd: 0,
                                    last_fire_tick: 0,
                                };
                                let inc = media_array_income(&state.terrain, &state.buildings, &dummy);
                                pos_score += inc * 4;
                                if terr.is_high() {
                                    pos_score += 14;
                                }
                                let rad = 5 + if terr.is_high() { 1 } else { 0 };
                                for &(ux, uy) in &my_unpowered_need_compute {
                                    if chebyshev(tx, ty, ux, uy) <= rad {
                                        pos_score += 55;
                                    }
                                }
                                for b in &state.buildings {
                                    if b.hp > 0 && b.owner == team && chebyshev(tx, ty, b.x, b.y) <= rad {
                                        if matches!(
                                            b.kind,
                                            BuildingType::HumanAutocannon | BuildingType::HumanOrbitalUplink
                                        ) {
                                            pos_score += 14;
                                        } else if b.kind == BuildingType::HumanMechFoundry {
                                            pos_score += 10;
                                        }
                                    }
                                }
                                if cell.mech > 0 && cell.compute == 0 {
                                    pos_score += 18;
                                }
                            }
                            BuildingType::ElvenScriptorium => {
                                let mut links = 0i32;
                                for b in &state.buildings {
                                    if b.hp > 0
                                        && b.owner == team
                                        && b.kind == BuildingType::ElvenLeyConduit
                                    {
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
                                pos_score += links.min(3) * 26;
                            }
                            BuildingType::DwarfRuneArchive => {
                                let mut near_ore = false;
                                for oy in (ty as i32 - 1).max(0)..=(ty as i32 + 1).min(MAP_H as i32 - 1) {
                                    for ox in (tx as i32 - 1).max(0)..=(tx as i32 + 1).min(MAP_W as i32 - 1) {
                                        if state.terrain[tile_idx(ox as usize, oy as usize)] == Terrain::OreVein {
                                            near_ore = true;
                                        }
                                    }
                                }
                                if near_ore {
                                    pos_score += 48;
                                }
                                pos_score -= fwd.max(0) * 2; // Safer near base
                            }
                            BuildingType::GnomeTinkerLab => {
                                // Orthogonal adjacency to own towers/factories!
                                let mut adj = 0i32;
                                for (dx, dy) in [(1i32, 0), (-1, 0), (0, 1), (0, -1)] {
                                    let nx = tx as i32 + dx;
                                    let ny = ty as i32 + dy;
                                    if nx >= 0 && nx < MAP_W as i32 && ny >= 0 && ny < MAP_H as i32 {
                                        let g = state.building_grid[tile_idx(nx as usize, ny as usize)];
                                        if g != GRID_EMPTY {
                                            let ob = &state.buildings[g as usize];
                                            if ob.owner == team
                                                && matches!(
                                                    ob.kind.slot(),
                                                    Some(BuildingSlot::Defense)
                                                        | Some(BuildingSlot::Apex)
                                                )
                                            {
                                                adj += 1;
                                            }
                                        }
                                    }
                                }
                                pos_score += adj * 26;
                            }
                            BuildingType::OrcBloodAltar => {
                                pos_score += fwd.clamp(0, 7) * 6 + mid_closeness * 2;
                            }
                            BuildingType::LeoneLithoShaper => {
                                let mut stone = 0i32;
                                for oy in (ty as i32 - 1).max(0)..=(ty as i32 + 1).min(MAP_H as i32 - 1) {
                                    for ox in (tx as i32 - 1).max(0)..=(tx as i32 + 1).min(MAP_W as i32 - 1) {
                                        if matches!(
                                            state.terrain[tile_idx(ox as usize, oy as usize)],
                                            Terrain::HighGround | Terrain::OreVein | Terrain::RaisedStone
                                        ) {
                                            stone += 1;
                                        }
                                    }
                                }
                                pos_score += stone * 8 + fwd.clamp(0, 6) * 3;
                            }
                            BuildingType::DraconeHoardVault => {
                                // Safe backline!
                                pos_score -= fwd * 4;
                            }
                            _ => {}
                        }
                        if kind != BuildingType::OrcBloodAltar && kind != BuildingType::HumanMediaArray {
                            // Prefer safer placements
                            pos_score -= (fwd - 4).max(0) * 4;
                        }
                    }
                    BuildingSlot::Defense => {
                        if let Some((thx, thy, _)) = nearest_threat_to_citadel {
                            let td = chebyshev(tx, ty, thx, thy) as i32;
                            if td <= 4 {
                                pos_score += 42;
                            } else {
                                pos_score += (12 - td).max(0) * 3;
                            }
                        }
                        if terr.is_high() {
                            pos_score += 22;
                        }
                        if fwd >= 1 && fwd <= 8 {
                            pos_score += 18;
                        } else if fwd < 0 {
                            pos_score -= 10;
                        }
                        // Synergies
                        for b in &state.buildings {
                            if b.hp > 0 && b.owner == team {
                                if b.kind == BuildingType::GnomeTinkerLab && manhattan(tx, ty, b.x, b.y) == 1 {
                                    pos_score += 24;
                                }
                                if b.kind == BuildingType::DwarfRunicHearth && chebyshev(tx, ty, b.x, b.y) <= 1 {
                                    pos_score += 16;
                                }
                                if kind == BuildingType::HumanAutocannon && b.build_ticks_left == 0 && b.powered {
                                    let odef = building_def(b.kind);
                                    if odef.emit_energy.contains(EnergyMask::COMPUTE)
                                        && chebyshev(tx, ty, b.x, b.y) <= emitter_radius(&state.terrain, b)
                                    {
                                        pos_score += 14;
                                    }
                                }
                                if kind == BuildingType::ElvenPrismSpire && b.kind == BuildingType::ElvenLeyConduit {
                                    let dx = b.x as i32 - tx as i32;
                                    let dy = b.y as i32 - ty as i32;
                                    if dx.abs().max(dy.abs()) <= 7 && (dx == 0 || dy == 0 || dx.abs() == dy.abs()) {
                                        pos_score += 18;
                                    }
                                }
                            }
                        }
                    }
                    BuildingSlot::Factory => {
                        if kind == BuildingType::DwarfTunnelBore {
                            let bore_cnt = state
                                .buildings
                                .iter()
                                .filter(|b| b.hp > 0 && b.owner == team && b.kind == BuildingType::DwarfTunnelBore)
                                .count();
                            if bore_cnt >= 1 {
                                // Second bore wants forward placement!
                                pos_score += fwd.clamp(0, 12) * 5;
                            } else {
                                pos_score += fwd.clamp(0, 4) * 3;
                            }
                        } else if kind == BuildingType::LeoneTectonicRam {
                            // Align Y with enemy citadel or enemy structures!
                            let dy = (ty as i32 - enemy_citadel.1 as i32).abs();
                            pos_score += (8 - dy).max(0) * 5 + fwd.clamp(1, 7) * 3;
                        } else {
                            pos_score += fwd.clamp(0, 6) * 4;
                            if kind == BuildingType::HumanMechFoundry && cell.compute > 0 {
                                pos_score += 12;
                            }
                        }
                        // Keep back from immediate suicide
                        if (tx as i32 - enemy_front_x) * forward_dir > -2 {
                            pos_score -= 40;
                        }
                    }
                    BuildingSlot::Apex => {
                        let d_foe = chebyshev(tx, ty, enemy_citadel.0, enemy_citadel.1) as i32;
                        if matches!(
                            kind,
                            BuildingType::HumanOrbitalUplink | BuildingType::DwarfHowitzer | BuildingType::ElvenAstralLens
                        ) {
                            // Loves range 5..8 to enemy structures!
                            if d_foe <= def.range_tiles as i32 {
                                pos_score += 45;
                            }
                            pos_score += fwd.clamp(2, 9) * 5;
                        } else {
                            pos_score += fwd.clamp(2, 8) * 5;
                        }
                        if kind == BuildingType::HumanOrbitalUplink {
                            for b in &state.buildings {
                                if b.hp > 0 && b.owner == team && b.build_ticks_left == 0 && b.powered {
                                    let odef = building_def(b.kind);
                                    if odef.emit_energy.contains(EnergyMask::COMPUTE)
                                        && chebyshev(tx, ty, b.x, b.y) <= emitter_radius(&state.terrain, b)
                                    {
                                        pos_score += 14;
                                    }
                                }
                            }
                        }
                        if kind == BuildingType::OrcDoomCauldron {
                            // Likes being adjacent to cheap Orc building for Devour!
                            for (dx, dy) in [(1i32, 0), (-1, 0), (0, 1), (0, -1), (1, 1), (-1, -1), (1, -1), (-1, 1)] {
                                let nx = tx as i32 + dx;
                                let ny = ty as i32 + dy;
                                if nx >= 0 && nx < MAP_W as i32 && ny >= 0 && ny < MAP_H as i32 {
                                    let g = state.building_grid[tile_idx(nx as usize, ny as usize)];
                                    if g != GRID_EMPTY && state.buildings[g as usize].owner == team {
                                        pos_score += 12;
                                    }
                                }
                            }
                        }
                        if terr.is_high() {
                            pos_score += 16;
                        }
                    }
                }

                // Keep top 2 spots
                if pos_score > best_tiles[0].2 {
                    best_tiles[1] = best_tiles[0];
                    best_tiles[0] = (tx, ty, pos_score);
                } else if pos_score > best_tiles[1].2 {
                    best_tiles[1] = (tx, ty, pos_score);
                }
            }

            for k in 0..2 {
                if best_tiles[k].2 > i32::MIN {
                    let act = Action::Build {
                        kind,
                        x: best_tiles[k].0,
                        y: best_tiles[k].1,
                    };
                    if state.can_execute(team, act).is_ok() {
                        out.push(ScoredAction {
                            action: act,
                            prior: macro_w + best_tiles[k].2 - (k as i32) * 38,
                        });
                    }
                }
            }
        }
    }

    // 2. Upgrades, Devour, Relocate, ShapeTerrain, TectonicPush
    for b in &state.buildings {
        if b.hp <= 0 || b.owner != team || b.build_ticks_left > 0 || b.kind == BuildingType::Citadel {
            continue;
        }
        // Upgrade candidate
        if b.level == 1 && ts.resources >= upgrade_cost(b.kind, ts.config) {
            let upg_act = Action::Upgrade { x: b.x, y: b.y };
            if state.can_execute(team, upg_act).is_ok() {
                let mut up_p = match b.kind.slot() {
                    Some(BuildingSlot::Eco) => 72,
                    Some(BuildingSlot::Apex) => 85,
                    Some(BuildingSlot::Defense) => 65,
                    Some(BuildingSlot::Factory) => 60,
                    _ => 40,
                };
                if !b.powered {
                    up_p -= 40;
                }
                out.push(ScoredAction {
                    action: upg_act,
                    prior: up_p,
                });
            }
        }

        // Orc Doom Cauldron Devour
        if b.kind == BuildingType::OrcDoomCauldron && b.fury_stacks < 3 {
            for other in &state.buildings {
                if other.hp > 0
                    && other.owner == team
                    && other.id != b.id
                    && other.kind != BuildingType::Citadel
                    && other.kind.faction() == Some(Faction::Orcs)
                    && chebyshev(b.x, b.y, other.x, other.y) <= 1
                {
                    let dev_act = Action::Devour {
                        cauldron_x: b.x,
                        cauldron_y: b.y,
                        victim_x: other.x,
                        victim_y: other.y,
                    };
                    if state.can_execute(team, dev_act).is_ok() {
                        let p = if b.hp < b.max_hp / 2 { 105 } else { 55 };
                        out.push(ScoredAction {
                            action: dev_act,
                            prior: p,
                        });
                        break;
                    }
                }
            }
        }

        // Gnome Walker Relocation if idle / far from combat
        if b.kind.is_walker() && b.relocate_target.is_none() {
            let has_target_near = state.units.iter().any(|u| {
                u.hp > 0
                    && u.owner == enemy
                    && chebyshev(b.x, b.y, px_to_tile(u.px, u.py).0 as u8, px_to_tile(u.px, u.py).1 as u8) <= 5
            }) || state.buildings.iter().any(|ob| {
                ob.hp > 0 && ob.owner == enemy && chebyshev(b.x, b.y, ob.x, ob.y) <= 5
            });
            if !has_target_near {
                let target_x = (b.x as i32 + forward_dir * 4).clamp(1, MAP_W as i32 - 2) as u8;
                let rel_act = Action::Relocate {
                    x: b.x,
                    y: b.y,
                    tx: target_x,
                    ty: b.y,
                };
                if state.can_execute(team, rel_act).is_ok() {
                    out.push(ScoredAction {
                        action: rel_act,
                        prior: 62,
                    });
                }
            }
        }

        // Leone Litho Shaper ShapeTerrain to raise defensive stone wall in front of approaching enemy
        if b.kind == BuildingType::LeoneLithoShaper && b.powered && b.ability_cd == 0 && ts.resources >= 55 {
            if let Some((thx, thy, td)) = nearest_threat_to_citadel {
                if td <= 8 {
                    let wx = (b.x as i32 + (thx as i32 - b.x as i32).signum()).clamp(0, MAP_W as i32 - 1) as u8;
                    let wy = (b.y as i32 + (thy as i32 - b.y as i32).signum()).clamp(0, MAP_H as i32 - 1) as u8;
                    let blocks_ram = state.buildings.iter().any(|ob| {
                        ob.hp > 0 && ob.owner == team && ob.kind == BuildingType::LeoneTectonicRam && ob.y == wy
                    });
                    if !blocks_ram && state.terrain[tile_idx(wx as usize, wy as usize)] == Terrain::Open {
                        let shape_act = Action::ShapeTerrain {
                            shaper_x: b.x,
                            shaper_y: b.y,
                            tx: wx,
                            ty: wy,
                        };
                        if state.can_execute(team, shape_act).is_ok() {
                            out.push(ScoredAction {
                                action: shape_act,
                                prior: 64,
                            });
                        }
                    }
                }
            }
        }

        // Leone Tectonic Ram Push
        if b.kind == BuildingType::LeoneTectonicRam && b.powered && b.ability_cd == 0 && ts.resources >= 25 {
            for other in &state.buildings {
                if other.hp > 0
                    && other.owner == team
                    && other.id != b.id
                    && other.kind == BuildingType::LeoneSolarPillar
                    && chebyshev(b.x, b.y, other.x, other.y) <= 1
                {
                    let dx = (other.x as i32 - b.x as i32) * forward_dir;
                    if dx > 0 {
                        let push_act = Action::TectonicPush {
                            ram_x: b.x,
                            ram_y: b.y,
                            target_x: other.x,
                            target_y: other.y,
                        };
                        if state.can_execute(team, push_act).is_ok() {
                            out.push(ScoredAction {
                                action: push_act,
                                prior: 58,
                            });
                            break;
                        }
                    }
                }
            }
        }
    }

    // Deterministic sort by descending prior, then action tie-breaker
    out.sort_by(|a, b| b.prior.cmp(&a.prior));
    if out.len() > max_candidates {
        out.truncate(max_candidates);
    }
    out
}
