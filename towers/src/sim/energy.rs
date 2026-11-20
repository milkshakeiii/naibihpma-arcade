use super::defs::building_def;
use super::types::*;

#[derive(Clone, Debug)]
pub struct EnergyGrid {
    pub raw: [[EnergyCell; MAP_TILES]; 2],
    pub net: [[EnergyCell; MAP_TILES]; 2],
    pub flux_storm: [bool; MAP_TILES],
    pub ley_links: Vec<LeyLink>,
    pub ley_tiles: [[bool; MAP_TILES]; 2],
}

impl Default for EnergyGrid {
    fn default() -> Self {
        Self {
            raw: [[EnergyCell::default(); MAP_TILES]; 2],
            net: [[EnergyCell::default(); MAP_TILES]; 2],
            flux_storm: [false; MAP_TILES],
            ley_links: Vec::new(),
            ley_tiles: [[false; MAP_TILES]; 2],
        }
    }
}

pub fn has_line_of_sight(terrain: &[Terrain; MAP_TILES], x0: i32, y0: i32, x1: i32, y1: i32) -> bool {
    let mut cx = x0;
    let mut cy = y0;
    let dx = (x1 - x0).abs();
    let dy = -(y1 - y0).abs();
    let sx = if x0 < x1 { 1 } else { -1 };
    let sy = if y0 < y1 { 1 } else { -1 };
    let mut err = dx + dy;

    loop {
        if (cx != x0 || cy != y0) && (cx != x1 || cy != y1) {
            if cx >= 0 && cx < MAP_W as i32 && cy >= 0 && cy < MAP_H as i32 {
                if terrain[tile_idx(cx as usize, cy as usize)].blocks_line() {
                    return false;
                }
            }
        }
        if cx == x1 && cy == y1 {
            break;
        }
        let e2 = 2 * err;
        if e2 >= dy {
            err += dy;
            cx += sx;
        }
        if e2 <= dx {
            err += dx;
            cy += sy;
        }
    }
    true
}

fn is_elven_ley_node(kind: BuildingType) -> bool {
    matches!(
        kind,
        BuildingType::ElvenLeyConduit
            | BuildingType::ElvenScriptorium
            | BuildingType::ElvenPrismSpire
            | BuildingType::ElvenWispSanctum
            | BuildingType::ElvenAstralLens
    )
}

pub fn emitter_radius(terrain: &[Terrain; MAP_TILES], b: &Building) -> u8 {
    let def = building_def(b.kind);
    if b.kind == BuildingType::Citadel {
        return def.emit_radius;
    }
    if def.emit_energy == EnergyMask::NONE {
        return 0;
    }
    let bonus_r = if b.level >= 2 { 1 } else { 0 };
    let idx = tile_idx(b.x as usize, b.y as usize);
    let terrain_bonus = if (b.kind == BuildingType::ElvenLeyConduit
        && terrain[idx] == Terrain::LeySpring)
        || (b.kind == BuildingType::HumanMediaArray && terrain[idx].is_high())
    {
        1
    } else {
        0
    };
    def.emit_radius + bonus_r + terrain_bonus
}

pub fn recompute_energy_and_links(
    terrain: &[Terrain; MAP_TILES],
    team_configs: &[FactionTeam; 2],
    buildings: &[Building],
    building_grid: &[u16; MAP_TILES],
    grid: &mut EnergyGrid,
) -> Vec<(bool, u8, bool, u8)> {
    // Clear raw & ley_tiles
    for t in 0..2 {
        grid.raw[t].fill(EnergyCell::default());
        grid.ley_tiles[t].fill(false);
    }
    grid.flux_storm.fill(false);
    grid.ley_links.clear();

    // 1. Natural Ley Springs grant Mana locally (radius 1)
    for y in 0..MAP_H {
        for x in 0..MAP_W {
            if terrain[tile_idx(x, y)] == Terrain::LeySpring {
                for dy in -1i32..=1 {
                    for dx in -1i32..=1 {
                        let nx = x as i32 + dx;
                        let ny = y as i32 + dy;
                        if nx >= 0 && nx < MAP_W as i32 && ny >= 0 && ny < MAP_H as i32 {
                            let amt = if dx == 0 && dy == 0 { 2 } else { 1 };
                            let idx = tile_idx(nx as usize, ny as usize);
                            grid.raw[0][idx].mana += amt;
                            grid.raw[1][idx].mana += amt;
                        }
                    }
                }
            }
        }
    }

    // 2. Building Energy Emitters
    // Pass 1: Primary infrastructure (Citadel, req==NONE emitters, and Solstice Colossus)
    for b in buildings {
        if b.hp <= 0 || b.build_ticks_left > 0 {
            continue;
        }
        let def = building_def(b.kind);
        let (emit_mask, emit_amt, emit_rad) = if b.kind == BuildingType::Citadel {
            (
                team_configs[b.owner.index()].required_energies(),
                def.emit_amount,
                emitter_radius(terrain, b),
            )
        } else if def.emit_energy != EnergyMask::NONE
            && (def.req_energy == EnergyMask::NONE
                || b.kind == BuildingType::LeoneSolsticeColossus)
        {
            (
                def.emit_energy,
                def.emit_amount + if b.level >= 2 { 1 } else { 0 },
                emitter_radius(terrain, b),
            )
        } else {
            continue;
        };

        let r = emit_rad as i32;
        let bx = b.x as i32;
        let by = b.y as i32;
        let t_idx = b.owner.index();
        for ty in (by - r).max(0)..=(by + r).min(MAP_H as i32 - 1) {
            for tx in (bx - r).max(0)..=(bx + r).min(MAP_W as i32 - 1) {
                let idx = tile_idx(tx as usize, ty as usize);
                grid.raw[t_idx][idx].add_mask(emit_mask, emit_amt);
            }
        }
    }

    // Pass 2: Secondary powered emitters (Media Arrays broadcast Compute when powered by Mech)
    for b in buildings {
        if b.hp <= 0 || b.build_ticks_left > 0 || b.kind != BuildingType::HumanMediaArray {
            continue;
        }
        let t_idx = b.owner.index();
        let b_tile = tile_idx(b.x as usize, b.y as usize);
        if grid.raw[t_idx][b_tile].mech <= 0 {
            continue;
        }
        let def = building_def(b.kind);
        let emit_amt = def.emit_amount + if b.level >= 2 { 1 } else { 0 };
        let r = emitter_radius(terrain, b) as i32;
        let bx = b.x as i32;
        let by = b.y as i32;
        for ty in (by - r).max(0)..=(by + r).min(MAP_H as i32 - 1) {
            for tx in (bx - r).max(0)..=(bx + r).min(MAP_W as i32 - 1) {
                let idx = tile_idx(tx as usize, ty as usize);
                grid.raw[t_idx][idx].add_mask(EnergyMask::COMPUTE, emit_amt);
            }
        }
    }

    // 3. Polarity Clash (+Force vs -Force between opposing teams) & Flux Storms
    for idx in 0..MAP_TILES {
        let p0 = grid.raw[0][idx];
        let p1 = grid.raw[1][idx];

        let mut net0 = p0;
        let mut net1 = p1;
        net0.pos_force = (p0.pos_force - p1.neg_force).max(0);
        net0.neg_force = (p0.neg_force - p1.pos_force).max(0);
        net1.pos_force = (p1.pos_force - p0.neg_force).max(0);
        net1.neg_force = (p1.neg_force - p0.pos_force).max(0);

        grid.net[0][idx] = net0;
        grid.net[1][idx] = net1;

        let clash_a = p0.pos_force.min(p1.neg_force);
        let clash_b = p1.pos_force.min(p0.neg_force);
        if clash_a >= 2 || clash_b >= 2 {
            grid.flux_storm[idx] = true;
        }
    }

    // 4. Straight-line Elven Ley Links (cardinal or 45-degree diagonal, max dist 7, at least one Conduit)
    let mut node_link_counts = vec![0u8; buildings.len()];
    for i in 0..buildings.len() {
        let a = &buildings[i];
        if a.hp <= 0 || a.build_ticks_left > 0 || !is_elven_ley_node(a.kind) {
            continue;
        }
        for j in (i + 1)..buildings.len() {
            let b = &buildings[j];
            if b.hp <= 0 || b.build_ticks_left > 0 || b.owner != a.owner || !is_elven_ley_node(b.kind) {
                continue;
            }
            if a.kind != BuildingType::ElvenLeyConduit && b.kind != BuildingType::ElvenLeyConduit {
                continue;
            }
            let dx = b.x as i32 - a.x as i32;
            let dy = b.y as i32 - a.y as i32;
            let adx = dx.abs();
            let ady = dy.abs();
            let dist = adx.max(ady);
            if dist == 0 || dist > 7 {
                continue;
            }
            if !(dx == 0 || dy == 0 || adx == ady) {
                continue;
            }
            // Check unobstructed between a and b (allow passing through own Elven nodes, block on RaisedStone or enemy)
            let sx = dx.signum();
            let sy = dy.signum();
            let mut cx = a.x as i32 + sx;
            let mut cy = a.y as i32 + sy;
            let mut clear = true;
            while cx != b.x as i32 || cy != b.y as i32 {
                let cidx = tile_idx(cx as usize, cy as usize);
                if terrain[cidx].blocks_line() {
                    clear = false;
                    break;
                }
                let occ = building_grid[cidx];
                if occ != GRID_EMPTY {
                    let ob = &buildings[occ as usize];
                    if ob.owner != a.owner {
                        clear = false;
                        break;
                    }
                }
                cx += sx;
                cy += sy;
            }
            if clear {
                grid.ley_links.push(LeyLink {
                    owner: a.owner,
                    x0: a.x,
                    y0: a.y,
                    x1: b.x,
                    y1: b.y,
                });
                node_link_counts[i] = node_link_counts[i].saturating_add(1);
                node_link_counts[j] = node_link_counts[j].saturating_add(1);

                let t_idx = a.owner.index();
                let mut tx = a.x as i32;
                let mut ty = a.y as i32;
                loop {
                    grid.ley_tiles[t_idx][tile_idx(tx as usize, ty as usize)] = true;
                    if tx == b.x as i32 && ty == b.y as i32 {
                        break;
                    }
                    tx += sx;
                    ty += sy;
                }
            }
        }
    }

    // 5. Compute per-building power & network states
    let mut out = Vec::with_capacity(buildings.len());
    for (i, b) in buildings.iter().enumerate() {
        if b.hp <= 0 {
            out.push((false, 0, false, 0));
            continue;
        }
        let def = building_def(b.kind);
        let idx = tile_idx(b.x as usize, b.y as usize);
        let powered = if b.kind == BuildingType::Citadel || b.build_ticks_left > 0 {
            true
        } else {
            grid.net[b.owner.index()][idx].satisfies(def.req_energy)
        };

        // Human Compute targeting links / Elven Ley links
        let net_links = if is_elven_ley_node(b.kind) {
            node_link_counts[i].min(4)
        } else if matches!(
            b.kind,
            BuildingType::HumanAutocannon | BuildingType::HumanOrbitalUplink
        ) {
            let mut cnt = 0u8;
            for other in buildings {
                if other.id != b.id
                    && other.hp > 0
                    && other.build_ticks_left == 0
                    && other.owner == b.owner
                {
                    let odef = building_def(other.kind);
                    let oidx = tile_idx(other.x as usize, other.y as usize);
                    if odef.emit_energy.contains(EnergyMask::COMPUTE)
                        && grid.net[other.owner.index()][oidx].satisfies(odef.req_energy)
                        && chebyshev(b.x, b.y, other.x, other.y) <= emitter_radius(terrain, other)
                    {
                        cnt = cnt.saturating_add(1);
                    }
                }
            }
            cnt.min(4)
        } else if b.kind == BuildingType::HumanMediaArray {
            let rad = emitter_radius(terrain, b);
            let mut cnt = 0u8;
            for other in buildings {
                if other.id != b.id
                    && other.hp > 0
                    && other.build_ticks_left == 0
                    && other.owner == b.owner
                    && matches!(
                        other.kind,
                        BuildingType::HumanAutocannon | BuildingType::HumanOrbitalUplink
                    )
                    && chebyshev(b.x, b.y, other.x, other.y) <= rad
                {
                    cnt = cnt.saturating_add(1);
                }
            }
            cnt.min(4)
        } else {
            0
        };

        // Gnome Tinker Lab orthogonal Gizmo buff (works on ALL allied buildings!)
        let mut gizmo = false;
        let mut hearth_armor = 0u8;
        for other in buildings {
            if other.id == b.id || other.hp <= 0 || other.build_ticks_left > 0 || other.owner != b.owner {
                continue;
            }
            if other.kind == BuildingType::GnomeTinkerLab
                && manhattan(b.x, b.y, other.x, other.y) == 1
                && grid.net[other.owner.index()][tile_idx(other.x as usize, other.y as usize)]
                    .satisfies(building_def(BuildingType::GnomeTinkerLab).req_energy)
            {
                gizmo = true;
            }
            if other.kind == BuildingType::DwarfRunicHearth
                && chebyshev(b.x, b.y, other.x, other.y) <= 1
            {
                hearth_armor = (hearth_armor + 1).min(2);
            }
        }

        out.push((powered, net_links, gizmo, hearth_armor));
    }
    out
}

pub fn media_array_income(
    terrain: &[Terrain; MAP_TILES],
    buildings: &[Building],
    b: &Building,
) -> i32 {
    if !b.powered || b.build_ticks_left > 0 {
        return 0;
    }
    let def = building_def(b.kind);
    let mut base = def.base_income;
    if b.level >= 2 {
        base = (base * 135) / 100;
    }
    let on_high = terrain[tile_idx(b.x as usize, b.y as usize)].is_high();
    let rad = 5i32 + if on_high { 1 } else { 0 };
    let bx = b.x as i32;
    let by = b.y as i32;
    let mut score = 0i32;

    for ty in (by - rad).max(0)..=(by + rad).min(MAP_H as i32 - 1) {
        for tx in (bx - rad).max(0)..=(bx + rad).min(MAP_W as i32 - 1) {
            if on_high || has_line_of_sight(terrain, bx, by, tx, ty) {
                // Mid-map / forward contested tiles pay more
                let mid_dist = (tx - 12).abs();
                if mid_dist <= 4 {
                    score += 2;
                } else {
                    score += 1;
                }
            }
        }
    }
    // Bonus for seeing enemy buildings within sight radius
    for other in buildings {
        if other.hp > 0 && other.owner != b.owner && chebyshev(b.x, b.y, other.x, other.y) as i32 <= rad {
            if on_high || has_line_of_sight(terrain, bx, by, other.x as i32, other.y as i32) {
                score += 14;
            }
        }
    }
    // Max possible score around ~160 -> scales base up to 2.0x
    let bonus_pct = (score * 100 / 130).clamp(0, 100);
    base + (base * bonus_pct) / 100
}
