pub mod combat;
pub mod defs;
pub mod energy;
pub mod rng;
pub mod types;
pub mod units;

use combat::{push_popup, step_buildings_and_combat};
use defs::{building_cost, building_def, upgrade_cost};
use energy::{media_array_income, recompute_energy_and_links, EnergyGrid};
use rng::SimRng;
use types::*;
use units::{px_to_tile, recompute_flow_fields, step_units, tile_center_px, FlowField};

#[derive(Clone, Debug)]
pub struct GameState {
    pub tick: u32,
    pub rng: SimRng,
    pub terrain: [Terrain; MAP_TILES],
    pub raised_hp: [i16; MAP_TILES],
    pub building_grid: [u16; MAP_TILES],
    pub teams: [TeamState; 2],
    pub buildings: Vec<Building>,
    pub units: Vec<Unit>,
    pub projectiles: Vec<SimProjectile>,
    pub energy_grid: EnergyGrid,
    pub flow_fields: [FlowField; 2],
    pub energy_dirty: bool,
    pub flow_dirty: bool,
    pub next_id: u32,
    pub outcome: MatchOutcome,
    pub challenge_goal: Option<ChallengeGoal>,
    pub record_fx: bool,
    pub fx_beams: Vec<FxBeam>,
    pub fx_bursts: Vec<FxBurst>,
    pub fx_popups: Vec<FxPopup>,
}

impl GameState {
    pub fn new_skirmish(
        seed: u64,
        terrain: [Terrain; MAP_TILES],
        p1_team: FactionTeam,
        p2_team: FactionTeam,
        p1_citadel: (u8, u8),
        p2_citadel: (u8, u8),
    ) -> Self {
        let mut s = Self {
            tick: 0,
            rng: SimRng::new(seed),
            terrain,
            raised_hp: [0; MAP_TILES],
            building_grid: [GRID_EMPTY; MAP_TILES],
            teams: [
                TeamState::new(p1_team, STARTING_RESOURCES),
                TeamState::new(p2_team, STARTING_RESOURCES),
            ],
            buildings: Vec::with_capacity(48),
            units: Vec::with_capacity(64),
            projectiles: Vec::with_capacity(32),
            energy_grid: EnergyGrid::default(),
            flow_fields: [FlowField::default(), FlowField::default()],
            energy_dirty: true,
            flow_dirty: true,
            next_id: 1,
            outcome: MatchOutcome::Ongoing,
            challenge_goal: None,
            record_fx: true,
            fx_beams: Vec::new(),
            fx_bursts: Vec::new(),
            fx_popups: Vec::new(),
        };

        for (i, &t) in s.terrain.iter().enumerate() {
            if t == Terrain::RaisedStone {
                s.raised_hp[i] = RAISED_STONE_MAX_HP;
            }
        }

        s.spawn_building_raw(TeamId::P1, BuildingType::Citadel, p1_citadel.0, p1_citadel.1, true);
        s.spawn_building_raw(TeamId::P2, BuildingType::Citadel, p2_citadel.0, p2_citadel.1, true);
        s.refresh_caches();
        s
    }

    pub fn new_empty(
        seed: u64,
        terrain: [Terrain; MAP_TILES],
        p1_team: FactionTeam,
        p2_team: FactionTeam,
        p1_res: i32,
        p2_res: i32,
        goal: Option<ChallengeGoal>,
    ) -> Self {
        let mut s = Self {
            tick: 0,
            rng: SimRng::new(seed),
            terrain,
            raised_hp: [0; MAP_TILES],
            building_grid: [GRID_EMPTY; MAP_TILES],
            teams: [
                TeamState::new(p1_team, p1_res),
                TeamState::new(p2_team, p2_res),
            ],
            buildings: Vec::with_capacity(48),
            units: Vec::with_capacity(64),
            projectiles: Vec::with_capacity(32),
            energy_grid: EnergyGrid::default(),
            flow_fields: [FlowField::default(), FlowField::default()],
            energy_dirty: true,
            flow_dirty: true,
            next_id: 1,
            outcome: MatchOutcome::Ongoing,
            challenge_goal: goal,
            record_fx: true,
            fx_beams: Vec::new(),
            fx_bursts: Vec::new(),
            fx_popups: Vec::new(),
        };
        for (i, &t) in s.terrain.iter().enumerate() {
            if t == Terrain::RaisedStone {
                s.raised_hp[i] = RAISED_STONE_MAX_HP;
            }
        }
        s
    }

    pub fn spawn_building_raw(
        &mut self,
        owner: TeamId,
        kind: BuildingType,
        x: u8,
        y: u8,
        instant: bool,
    ) -> Option<usize> {
        if x as usize >= MAP_W || y as usize >= MAP_H {
            return None;
        }
        let idx = tile_idx(x as usize, y as usize);
        if self.building_grid[idx] != GRID_EMPTY {
            return None;
        }
        let def = building_def(kind);
        let mut hp = def.max_hp;
        let mut armor = def.armor;
        if kind.faction() == Some(Faction::Dwarves) {
            let kt = self.teams[owner.index()].knowledge_tier as i32;
            hp = (hp * (100 + kt * 7)) / 100;
            armor += kt / 3;
        }
        self.next_id += 1;
        let b_idx = self.buildings.len();
        self.buildings.push(Building {
            id: self.next_id,
            owner,
            kind,
            x,
            y,
            hp,
            max_hp: hp,
            armor,
            level: 1,
            build_ticks_left: if instant { 0 } else { def.build_ticks },
            cooldown: 0,
            spawn_timer: if def.spawn_interval > 0 {
                def.spawn_interval / 2
            } else {
                0
            },
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
        });
        self.building_grid[idx] = b_idx as u16;
        self.energy_dirty = true;
        self.flow_dirty = true;
        Some(b_idx)
    }

    pub fn spawn_unit_raw(&mut self, owner: TeamId, kind: UnitKind, tx: u8, ty: u8) {
        let (px, py) = tile_center_px(tx, ty);
        let def = defs::unit_def(kind);
        self.next_id += 1;
        let dir_dx = if owner == TeamId::P1 { 1 } else { -1 };
        self.units.push(Unit {
            id: self.next_id,
            owner,
            kind,
            px,
            py,
            hp: def.max_hp,
            max_hp: def.max_hp,
            cooldown: 0,
            slow_ticks: 0,
            corrode_ticks: 0,
            dir_dx,
            dir_dy: 0,
            hit_flash: 0,
        });
    }

    pub fn refresh_caches(&mut self) {
        if self.energy_dirty {
            let team_configs = [self.teams[0].config, self.teams[1].config];
            let states = recompute_energy_and_links(
                &self.terrain,
                &team_configs,
                &self.buildings,
                &self.building_grid,
                &mut self.energy_grid,
            );
            for (i, (pow, links, gizmo, hearth)) in states.into_iter().enumerate() {
                if i < self.buildings.len() {
                    self.buildings[i].powered = pow;
                    self.buildings[i].net_links = links;
                    self.buildings[i].gizmo_buff = gizmo;
                    self.buildings[i].hearth_armor = hearth;
                }
            }
            self.energy_dirty = false;
        }
        if self.flow_dirty {
            recompute_flow_fields(&self.terrain, &self.buildings, &mut self.flow_fields);
            self.flow_dirty = false;
        }
    }

    pub fn is_in_build_territory(&self, team: TeamId, x: u8, y: u8) -> bool {
        for b in &self.buildings {
            if b.hp > 0 && b.owner == team && b.build_ticks_left == 0 {
                if chebyshev(b.x, b.y, x, y) <= 5 {
                    return true;
                }
            }
        }
        false
    }

    pub fn has_eco_building(&self, team: TeamId, faction: Faction) -> bool {
        let eco_kind = BuildingType::from_faction_slot(faction, BuildingSlot::Eco);
        self.buildings
            .iter()
            .any(|b| b.hp > 0 && b.owner == team && b.kind == eco_kind && b.build_ticks_left == 0)
    }

    pub fn can_execute(&self, team: TeamId, action: Action) -> Result<(), &'static str> {
        if self.outcome != MatchOutcome::Ongoing {
            return Err("MATCH ALREADY ENDED");
        }
        let ts = &self.teams[team.index()];
        match action {
            Action::NoOp => Ok(()),
            Action::Build { kind, x, y } => {
                if kind == BuildingType::Citadel {
                    return Err("CANNOT BUILD CITADEL");
                }
                let Some(faction) = kind.faction() else {
                    return Err("INVALID BUILDING");
                };
                if !ts.config.has(faction) {
                    return Err("FACTION NOT IN TEAM");
                }
                if x as usize >= MAP_W || y as usize >= MAP_H {
                    return Err("OUT OF BOUNDS");
                }
                let idx = tile_idx(x as usize, y as usize);
                if self.building_grid[idx] != GRID_EMPTY {
                    return Err("TILE OCCUPIED");
                }
                let t = self.terrain[idx];
                if t == Terrain::Chasm {
                    return Err("CANNOT BUILD ON CHASM");
                }
                if t == Terrain::RaisedStone && faction != Faction::Leones {
                    return Err("ONLY LEONES BUILD ON RAISED STONE");
                }
                if !self.is_in_build_territory(team, x, y) {
                    return Err("OUTSIDE GRID RANGE (R5)");
                }
                let def = building_def(kind);
                if def.require_eco && !self.has_eco_building(team, faction) {
                    return Err("REQUIRES FACTION ECO BUILDING");
                }
                let cost = building_cost(kind, ts.config);
                if ts.resources < cost {
                    return Err("INSUFFICIENT RESOURCES");
                }
                Ok(())
            }
            Action::Upgrade { x, y } => {
                if x as usize >= MAP_W || y as usize >= MAP_H {
                    return Err("OUT OF BOUNDS");
                }
                let g = self.building_grid[tile_idx(x as usize, y as usize)];
                if g == GRID_EMPTY {
                    return Err("NO BUILDING HERE");
                }
                let b = &self.buildings[g as usize];
                if b.owner != team || b.hp <= 0 {
                    return Err("NOT YOUR BUILDING");
                }
                if b.kind == BuildingType::Citadel {
                    return Err("CANNOT UPGRADE CITADEL");
                }
                if b.build_ticks_left > 0 {
                    return Err("STILL CONSTRUCTING");
                }
                if b.level >= 2 {
                    return Err("ALREADY MK.II");
                }
                let cost = upgrade_cost(b.kind, ts.config);
                if ts.resources < cost {
                    return Err("INSUFFICIENT RESOURCES");
                }
                Ok(())
            }
            Action::Salvage { x, y } => {
                if x as usize >= MAP_W || y as usize >= MAP_H {
                    return Err("OUT OF BOUNDS");
                }
                let g = self.building_grid[tile_idx(x as usize, y as usize)];
                if g == GRID_EMPTY {
                    return Err("NO BUILDING HERE");
                }
                let b = &self.buildings[g as usize];
                if b.owner != team || b.hp <= 0 {
                    return Err("NOT YOUR BUILDING");
                }
                if b.kind == BuildingType::Citadel {
                    return Err("CANNOT SALVAGE CITADEL");
                }
                Ok(())
            }
            Action::Devour {
                cauldron_x,
                cauldron_y,
                victim_x,
                victim_y,
            } => {
                if cauldron_x as usize >= MAP_W
                    || cauldron_y as usize >= MAP_H
                    || victim_x as usize >= MAP_W
                    || victim_y as usize >= MAP_H
                {
                    return Err("OUT OF BOUNDS");
                }
                let cg = self.building_grid[tile_idx(cauldron_x as usize, cauldron_y as usize)];
                let vg = self.building_grid[tile_idx(victim_x as usize, victim_y as usize)];
                if cg == GRID_EMPTY || vg == GRID_EMPTY || cg == vg {
                    return Err("INVALID DEVOUR TARGET");
                }
                let c = &self.buildings[cg as usize];
                let v = &self.buildings[vg as usize];
                if c.owner != team || c.kind != BuildingType::OrcDoomCauldron || c.build_ticks_left > 0 {
                    return Err("NEED ACTIVE DOOM CAULDRON");
                }
                if c.fury_stacks >= 3 {
                    return Err("MAX FURY REACHED");
                }
                if v.owner != team || v.kind == BuildingType::Citadel || v.kind.faction() != Some(Faction::Orcs) {
                    return Err("VICTIM MUST BE ORC STRUCTURE");
                }
                if chebyshev(cauldron_x, cauldron_y, victim_x, victim_y) > 1 {
                    return Err("VICTIM MUST BE ADJACENT");
                }
                Ok(())
            }
            Action::Relocate { x, y, tx, ty } => {
                if x as usize >= MAP_W
                    || y as usize >= MAP_H
                    || tx as usize >= MAP_W
                    || ty as usize >= MAP_H
                {
                    return Err("OUT OF BOUNDS");
                }
                let g = self.building_grid[tile_idx(x as usize, y as usize)];
                if g == GRID_EMPTY {
                    return Err("NO WALKER HERE");
                }
                let b = &self.buildings[g as usize];
                if b.owner != team || !b.kind.is_walker() || b.build_ticks_left > 0 {
                    return Err("NOT AN ACTIVE WALKER");
                }
                let tidx = tile_idx(tx as usize, ty as usize);
                if self.terrain[tidx].blocks_ground() || self.building_grid[tidx] != GRID_EMPTY {
                    return Err("DESTINATION BLOCKED");
                }
                Ok(())
            }
            Action::ShapeTerrain {
                shaper_x,
                shaper_y,
                tx,
                ty,
            } => {
                if shaper_x as usize >= MAP_W
                    || shaper_y as usize >= MAP_H
                    || tx as usize >= MAP_W
                    || ty as usize >= MAP_H
                {
                    return Err("OUT OF BOUNDS");
                }
                let g = self.building_grid[tile_idx(shaper_x as usize, shaper_y as usize)];
                if g == GRID_EMPTY {
                    return Err("NO SHAPER HERE");
                }
                let s = &self.buildings[g as usize];
                if s.owner != team
                    || s.kind != BuildingType::LeoneLithoShaper
                    || s.build_ticks_left > 0
                    || !s.powered
                {
                    return Err("NEED POWERED LITHO SHAPER");
                }
                if s.ability_cd > 0 {
                    return Err("SHAPER ON COOLDOWN");
                }
                if chebyshev(shaper_x, shaper_y, tx, ty) > 3 {
                    return Err("TARGET OUT OF R3 RANGE");
                }
                let tidx = tile_idx(tx as usize, ty as usize);
                if self.building_grid[tidx] != GRID_EMPTY {
                    return Err("TILE OCCUPIED BY STRUCTURE");
                }
                let t = self.terrain[tidx];
                if t == Terrain::Open {
                    if ts.resources < 20 {
                        return Err("NEED 20 ROCKS");
                    }
                } else if t == Terrain::RaisedStone {
                    if ts.resources < 10 {
                        return Err("NEED 10 ROCKS");
                    }
                } else {
                    return Err("MUST TARGET OPEN OR RAISED STONE");
                }
                Ok(())
            }
            Action::TectonicPush {
                ram_x,
                ram_y,
                target_x,
                target_y,
            } => {
                if ram_x as usize >= MAP_W
                    || ram_y as usize >= MAP_H
                    || target_x as usize >= MAP_W
                    || target_y as usize >= MAP_H
                {
                    return Err("OUT OF BOUNDS");
                }
                let rg = self.building_grid[tile_idx(ram_x as usize, ram_y as usize)];
                let tg = self.building_grid[tile_idx(target_x as usize, target_y as usize)];
                if rg == GRID_EMPTY || tg == GRID_EMPTY || rg == tg {
                    return Err("INVALID PUSH TARGET");
                }
                let r = &self.buildings[rg as usize];
                let t = &self.buildings[tg as usize];
                if r.owner != team
                    || r.kind != BuildingType::LeoneTectonicRam
                    || r.build_ticks_left > 0
                    || !r.powered
                {
                    return Err("NEED POWERED TECTONIC RAM");
                }
                if r.ability_cd > 0 {
                    return Err("RAM ON COOLDOWN");
                }
                if t.owner != team || t.kind == BuildingType::Citadel || t.kind.faction() != Some(Faction::Leones) {
                    return Err("TARGET MUST BE LEONE BUILDING");
                }
                if chebyshev(ram_x, ram_y, target_x, target_y) > 1 {
                    return Err("TARGET MUST BE ADJACENT");
                }
                if ts.resources < 10 {
                    return Err("NEED 10 ROCKS");
                }
                let dx = (target_x as i32 - ram_x as i32).signum();
                let dy = (target_y as i32 - ram_y as i32).signum();
                let nx = target_x as i32 + dx;
                let ny = target_y as i32 + dy;
                if nx < 0 || nx >= MAP_W as i32 || ny < 0 || ny >= MAP_H as i32 {
                    return Err("PUSH DESTINATION OFF MAP");
                }
                let nidx = tile_idx(nx as usize, ny as usize);
                if self.terrain[nidx] == Terrain::Chasm || self.building_grid[nidx] != GRID_EMPTY {
                    return Err("PUSH DESTINATION BLOCKED");
                }
                Ok(())
            }
        }
    }

    pub fn execute_action(&mut self, team: TeamId, action: Action) -> Result<(), &'static str> {
        self.can_execute(team, action)?;
        match action {
            Action::NoOp => Ok(()),
            Action::Build { kind, x, y } => {
                let cost = building_cost(kind, self.teams[team.index()].config);
                self.teams[team.index()].resources -= cost;
                self.teams[team.index()].total_spent += cost;
                self.teams[team.index()].buildings_built += 1;
                self.spawn_building_raw(team, kind, x, y, false);
                self.refresh_caches();
                Ok(())
            }
            Action::Upgrade { x, y } => {
                let g = self.building_grid[tile_idx(x as usize, y as usize)] as usize;
                let kind = self.buildings[g].kind;
                let cost = upgrade_cost(kind, self.teams[team.index()].config);
                self.teams[team.index()].resources -= cost;
                self.teams[team.index()].total_spent += cost;
                let bonus_hp = (self.buildings[g].max_hp * 45) / 100;
                self.buildings[g].max_hp += bonus_hp;
                self.buildings[g].hp += bonus_hp;
                self.buildings[g].level = 2;
                self.energy_dirty = true;
                self.refresh_caches();
                if self.record_fx {
                    let (px, py) = tile_center_px(x, y);
                    push_popup(&mut self.fx_popups, px, py - 8, "MK.II!", 42);
                }
                Ok(())
            }
            Action::Salvage { x, y } => {
                let g = self.building_grid[tile_idx(x as usize, y as usize)] as usize;
                let kind = self.buildings[g].kind;
                let base_cost = building_cost(kind, self.teams[team.index()].config);
                let near_altar = self.buildings.iter().any(|b| {
                    b.hp > 0
                        && b.owner == team
                        && b.build_ticks_left == 0
                        && b.kind == BuildingType::OrcBloodAltar
                        && chebyshev(b.x, b.y, x, y) <= 6
                });
                let refund_pct = if near_altar { 80 } else { 50 };
                let refund = (base_cost * refund_pct) / 100;
                self.teams[team.index()].resources += refund;
                self.teams[team.index()].total_earned += refund;
                self.buildings[g].hp = 0;
                self.cleanup_dead_buildings();
                self.refresh_caches();
                if self.record_fx {
                    let (px, py) = tile_center_px(x, y);
                    let tag = format!("+{}", refund);
                    push_popup(&mut self.fx_popups, px, py - 6, &tag, if near_altar { 40 } else { 42 });
                }
                Ok(())
            }
            Action::Devour {
                cauldron_x,
                cauldron_y,
                victim_x,
                victim_y,
            } => {
                let cg = self.building_grid[tile_idx(cauldron_x as usize, cauldron_y as usize)] as usize;
                let vg = self.building_grid[tile_idx(victim_x as usize, victim_y as usize)] as usize;
                self.buildings[vg].hp = 0;
                self.buildings[cg].hp = self.buildings[cg].max_hp;
                self.buildings[cg].fury_stacks = (self.buildings[cg].fury_stacks + 1).min(3);
                // Altar ritual bonus
                if self.buildings.iter().any(|b| {
                    b.hp > 0
                        && b.owner == team
                        && b.build_ticks_left == 0
                        && b.kind == BuildingType::OrcBloodAltar
                        && chebyshev(b.x, b.y, victim_x, victim_y) <= 6
                }) {
                    self.teams[team.index()].resources += 14;
                    self.teams[team.index()].total_earned += 14;
                }
                self.cleanup_dead_buildings();
                self.refresh_caches();
                if self.record_fx {
                    let (px, py) = tile_center_px(cauldron_x, cauldron_y);
                    push_popup(&mut self.fx_popups, px, py - 8, "FURY!", 40);
                }
                Ok(())
            }
            Action::Relocate { x, y, tx, ty } => {
                let g = self.building_grid[tile_idx(x as usize, y as usize)] as usize;
                self.buildings[g].relocate_target = Some((tx, ty));
                self.buildings[g].relocate_timer = 0;
                Ok(())
            }
            Action::ShapeTerrain {
                shaper_x,
                shaper_y,
                tx,
                ty,
            } => {
                let sg = self.building_grid[tile_idx(shaper_x as usize, shaper_y as usize)] as usize;
                let tidx = tile_idx(tx as usize, ty as usize);
                if self.terrain[tidx] == Terrain::Open {
                    self.teams[team.index()].resources -= 20;
                    self.teams[team.index()].total_spent += 20;
                    self.terrain[tidx] = Terrain::RaisedStone;
                    self.raised_hp[tidx] = RAISED_STONE_MAX_HP;
                    self.buildings[sg].ability_cd = 30;
                    // Nudge ground units off newly raised stone tile
                    let (cx, cy) = tile_center_px(tx, ty);
                    for u in self.units.iter_mut() {
                        if !u.kind.is_flying() {
                            let (ux, uy) = px_to_tile(u.px, u.py);
                            if ux == tx as usize && uy == ty as usize {
                                u.px = (cx + 14).min(PLAYFIELD_W - 6);
                                u.py = cy;
                            }
                        }
                    }
                } else {
                    self.teams[team.index()].resources -= 10;
                    self.teams[team.index()].total_spent += 10;
                    self.terrain[tidx] = Terrain::Open;
                    self.raised_hp[tidx] = 0;
                    self.buildings[sg].ability_cd = 20;
                }
                self.energy_dirty = true;
                self.flow_dirty = true;
                self.refresh_caches();
                Ok(())
            }
            Action::TectonicPush {
                ram_x,
                ram_y,
                target_x,
                target_y,
            } => {
                let rg = self.building_grid[tile_idx(ram_x as usize, ram_y as usize)] as usize;
                let tg = self.building_grid[tile_idx(target_x as usize, target_y as usize)] as usize;
                let dx = (target_x as i32 - ram_x as i32).signum();
                let dy = (target_y as i32 - ram_y as i32).signum();
                let nx = (target_x as i32 + dx) as u8;
                let ny = (target_y as i32 + dy) as u8;

                self.teams[team.index()].resources -= 10;
                self.teams[team.index()].total_spent += 10;
                self.buildings[rg].ability_cd = 30;

                let old_idx = tile_idx(target_x as usize, target_y as usize);
                let new_idx = tile_idx(nx as usize, ny as usize);
                self.building_grid[old_idx] = GRID_EMPTY;
                self.building_grid[new_idx] = tg as u16;
                self.buildings[tg].x = nx;
                self.buildings[tg].y = ny;

                // Slam enemy units near destination
                let (npx, npy) = tile_center_px(nx, ny);
                for u in self.units.iter_mut() {
                    if u.hp > 0 && u.owner != team && units::dist_sq(npx, npy, u.px, u.py) <= 18 * 18 {
                        u.hp -= 28;
                        u.hit_flash = 3;
                    }
                }

                self.energy_dirty = true;
                self.flow_dirty = true;
                self.refresh_caches();
                Ok(())
            }
        }
    }

    pub fn step_n(&mut self, n: u32) {
        for _ in 0..n {
            if self.outcome != MatchOutcome::Ongoing {
                break;
            }
            self.step();
        }
    }

    pub fn step(&mut self) {
        if self.outcome != MatchOutcome::Ongoing {
            return;
        }
        self.tick += 1;

        // Tick down visual FX
        if self.record_fx {
            self.fx_beams.retain_mut(|b| {
                b.ttl = b.ttl.saturating_sub(1);
                b.ttl > 0
            });
            self.fx_bursts.retain_mut(|b| {
                b.ttl = b.ttl.saturating_sub(1);
                b.ttl > 0
            });
            self.fx_popups.retain_mut(|p| {
                p.py -= 1;
                p.ttl = p.ttl.saturating_sub(1);
                p.ttl > 0
            });
        }

        // 1. Refresh spatial energy / ley links / flow fields if dirty
        self.refresh_caches();

        // 2. Economy cycle every ECO_CYCLE_TICKS (45 ticks = 3s)
        if self.tick % ECO_CYCLE_TICKS == 0 {
            self.run_economy_cycle();
        }

        // 3. Step buildings & combat
        step_buildings_and_combat(
            self.tick,
            &self.terrain,
            &mut self.teams,
            &mut self.buildings,
            &mut self.building_grid,
            &mut self.units,
            &mut self.projectiles,
            &self.energy_grid,
            &mut self.next_id,
            &mut self.energy_dirty,
            &mut self.flow_dirty,
            self.record_fx,
            &mut self.fx_beams,
            &mut self.fx_bursts,
            &mut self.fx_popups,
        );

        // 4. Step units (with sudden escalation dmg boost if >= 7200)
        step_units(
            self.tick,
            &mut self.terrain,
            &mut self.raised_hp,
            &mut self.teams,
            &mut self.buildings,
            &self.building_grid,
            &mut self.units,
            &self.energy_grid,
            &self.flow_fields,
            &mut self.flow_dirty,
            self.record_fx,
            &mut self.fx_beams,
            &mut self.fx_bursts,
        );

        // 5. Clean up destroyed buildings
        let any_dead = self.buildings.iter().any(|b| b.hp <= 0);
        if any_dead {
            self.cleanup_dead_buildings();
        }

        // 6. Check victory / loss / challenge goal
        self.check_outcome();
    }

    fn run_economy_cycle(&mut self) {
        let escalation = self.tick >= SUDDEN_ESCALATION_TICK;
        let mut vault_interest = [0i32; 2];
        let mut cycle_inc = [0i32; 2];

        for bi in 0..self.buildings.len() {
            let b = self.buildings[bi];
            if b.hp <= 0 || b.build_ticks_left > 0 {
                continue;
            }
            let ti = b.owner.index();
            if b.kind == BuildingType::Citadel {
                let base = self.teams[ti].config.citadel_income();
                let inc = if escalation { (base * 150) / 100 } else { base };
                cycle_inc[ti] += inc;
                self.teams[ti].knowledge_xp += inc / 3;
                continue;
            }
            if !b.powered {
                continue;
            }
            let def = building_def(b.kind);
            if def.base_income == 0 {
                continue;
            }
            let mut inc = match b.kind {
                BuildingType::HumanMediaArray => media_array_income(&self.terrain, &self.buildings, &b),
                BuildingType::ElvenScriptorium => {
                    let base = if b.level >= 2 { (def.base_income * 135) / 100 } else { def.base_income };
                    let links = b.net_links.min(3) as i32;
                    (base * (100 + links * 50)) / 100
                }
                BuildingType::OrcBloodAltar => {
                    if b.level >= 2 { (def.base_income * 135) / 100 } else { def.base_income }
                }
                BuildingType::DwarfRuneArchive => {
                    let mut base = if b.level >= 2 { (def.base_income * 135) / 100 } else { def.base_income };
                    let mut near_ore = false;
                    for ty in (b.y as i32 - 1).max(0)..=(b.y as i32 + 1).min(MAP_H as i32 - 1) {
                        for tx in (b.x as i32 - 1).max(0)..=(b.x as i32 + 1).min(MAP_W as i32 - 1) {
                            if self.terrain[tile_idx(tx as usize, ty as usize)] == Terrain::OreVein {
                                near_ore = true;
                            }
                        }
                    }
                    if near_ore {
                        base = (base * 160) / 100;
                    }
                    self.teams[ti].knowledge_xp += base;
                    base
                }
                BuildingType::GnomeTinkerLab => {
                    if b.level >= 2 { (def.base_income * 135) / 100 } else { def.base_income }
                }
                BuildingType::LeoneLithoShaper => {
                    let mut base = if b.level >= 2 { (def.base_income * 135) / 100 } else { def.base_income };
                    let mut adj_stone = 0i32;
                    for ty in (b.y as i32 - 1).max(0)..=(b.y as i32 + 1).min(MAP_H as i32 - 1) {
                        for tx in (b.x as i32 - 1).max(0)..=(b.x as i32 + 1).min(MAP_W as i32 - 1) {
                            if tx == b.x as i32 && ty == b.y as i32 {
                                continue;
                            }
                            let t = self.terrain[tile_idx(tx as usize, ty as usize)];
                            if matches!(t, Terrain::HighGround | Terrain::OreVein | Terrain::RaisedStone) {
                                adj_stone += 1;
                            }
                        }
                    }
                    base += (adj_stone * 2).min(8);
                    base
                }
                BuildingType::DraconeHoardVault => {
                    let base = if b.level >= 2 { (def.base_income * 135) / 100 } else { def.base_income };
                    let rate = if b.level >= 2 { 5 } else { 4 };
                    let cap = if b.level >= 2 { 30 } else { 24 };
                    let interest = ((self.teams[ti].resources * rate) / 100).clamp(0, cap);
                    let remaining_cap = (48 - vault_interest[ti]).max(0);
                    let actual_int = interest.min(remaining_cap);
                    vault_interest[ti] += actual_int;
                    base + actual_int
                }
                _ => 0,
            };

            if self.terrain[tile_idx(b.x as usize, b.y as usize)] == Terrain::OreVein
                && b.kind != BuildingType::DwarfRuneArchive
            {
                inc += 2;
            }
            cycle_inc[ti] += inc;

            if self.record_fx && inc > 0 {
                let (px, py) = tile_center_px(b.x, b.y);
                let s = format!("+{}", inc);
                push_popup(&mut self.fx_popups, px, py - 8, &s, 42);
            }
        }

        for ti in 0..2 {
            self.teams[ti].resources += cycle_inc[ti];
            self.teams[ti].total_earned += cycle_inc[ti];
            self.teams[ti].last_cycle_income = cycle_inc[ti];

            // Level up Dwarven Knowledge Tier (0..5) every 115 XP
            if self.teams[ti].config.has(Faction::Dwarves) {
                let new_tier = (self.teams[ti].knowledge_xp / 115).clamp(0, 5) as u8;
                if new_tier > self.teams[ti].knowledge_tier {
                    let diff = (new_tier - self.teams[ti].knowledge_tier) as i32;
                    self.teams[ti].knowledge_tier = new_tier;
                    // Upgrade existing Dwarven buildings HP & armor immediately
                    for b in self.buildings.iter_mut() {
                        if b.hp > 0 && b.owner.index() == ti && b.kind.faction() == Some(Faction::Dwarves) {
                            let add = (b.max_hp * 7 * diff) / 100;
                            b.max_hp += add;
                            b.hp += add;
                            b.armor += diff / 3;
                        }
                    }
                }
            }
        }
    }

    fn cleanup_dead_buildings(&mut self) {
        self.building_grid.fill(GRID_EMPTY);
        let mut i = 0;
        while i < self.buildings.len() {
            if self.buildings[i].hp <= 0 {
                let dead = self.buildings.swap_remove(i);
                self.teams[dead.owner.index()].buildings_lost += 1;
                if self.record_fx {
                    let (px, py) = tile_center_px(dead.x, dead.y);
                    self.fx_bursts.push(FxBurst {
                        px: px as i16,
                        py: py as i16,
                        radius: 10,
                        color: 40,
                        ttl: 8,
                    });
                }
            } else {
                let idx = tile_idx(self.buildings[i].x as usize, self.buildings[i].y as usize);
                self.building_grid[idx] = i as u16;
                i += 1;
            }
        }
        self.energy_dirty = true;
        self.flow_dirty = true;
    }

    fn check_outcome(&mut self) {
        let p1_citadel_hp = self
            .buildings
            .iter()
            .filter(|b| b.owner == TeamId::P1 && b.kind == BuildingType::Citadel && b.hp > 0)
            .map(|b| b.hp)
            .sum::<i32>();
        let p2_citadel_hp = self
            .buildings
            .iter()
            .filter(|b| b.owner == TeamId::P2 && b.kind == BuildingType::Citadel && b.hp > 0)
            .map(|b| b.hp)
            .sum::<i32>();

        if let Some(goal) = self.challenge_goal {
            // In challenges, losing P1 Citadel (if P1 has one) or all P1 buildings is defeat
            let p1_alive = self.buildings.iter().any(|b| b.owner == TeamId::P1 && b.hp > 0);
            if !p1_alive || (goal != ChallengeGoal::DestroyAllEnemyStructures && p1_citadel_hp == 0) {
                // Only fail on citadel==0 if P1 actually had a citadel; wait: if challenge gives P1 a Citadel, check p1_citadel_hp == 0!
                // Wait: what if a challenge gave P1 no Citadel? We'll make sure all P1 challenges have a Citadel (or at least buildings)!
                if !p1_alive {
                    self.outcome = MatchOutcome::Victory(TeamId::P2);
                    return;
                }
            }
            match goal {
                ChallengeGoal::DestroyEnemyCitadel => {
                    if p1_citadel_hp == 0 {
                        self.outcome = MatchOutcome::Victory(TeamId::P2);
                    } else if p2_citadel_hp == 0 {
                        self.outcome = MatchOutcome::Victory(TeamId::P1);
                    }
                }
                ChallengeGoal::DestroyAllEnemyStructures => {
                    let p2_any = self.buildings.iter().any(|b| b.owner == TeamId::P2 && b.hp > 0);
                    if p1_citadel_hp == 0 {
                        self.outcome = MatchOutcome::Victory(TeamId::P2);
                    } else if !p2_any {
                        self.outcome = MatchOutcome::Victory(TeamId::P1);
                    }
                }
                ChallengeGoal::SurviveTicks(target) => {
                    if p1_citadel_hp == 0 {
                        self.outcome = MatchOutcome::Victory(TeamId::P2);
                    } else if self.tick >= target {
                        self.outcome = MatchOutcome::Victory(TeamId::P1);
                    }
                }
                ChallengeGoal::ReachResources(target) => {
                    if p1_citadel_hp == 0 {
                        self.outcome = MatchOutcome::Victory(TeamId::P2);
                    } else if self.teams[0].resources >= target {
                        self.outcome = MatchOutcome::Victory(TeamId::P1);
                    }
                }
            }
            if self.outcome == MatchOutcome::Ongoing && self.tick >= MAX_MATCH_TICKS {
                self.outcome = MatchOutcome::Victory(TeamId::P2);
            }
            return;
        }

        // Standard Skirmish Victory
        if p1_citadel_hp == 0 && p2_citadel_hp == 0 {
            self.outcome = MatchOutcome::Draw;
        } else if p2_citadel_hp == 0 {
            self.outcome = MatchOutcome::Victory(TeamId::P1);
        } else if p1_citadel_hp == 0 {
            self.outcome = MatchOutcome::Victory(TeamId::P2);
        } else if self.tick >= MAX_MATCH_TICKS {
            if p1_citadel_hp > p2_citadel_hp {
                self.outcome = MatchOutcome::Victory(TeamId::P1);
            } else if p2_citadel_hp > p1_citadel_hp {
                self.outcome = MatchOutcome::Victory(TeamId::P2);
            } else {
                self.outcome = MatchOutcome::Draw;
            }
        }
    }

    pub fn citadel_hp(&self, team: TeamId) -> i32 {
        self.buildings
            .iter()
            .filter(|b| b.owner == team && b.kind == BuildingType::Citadel && b.hp > 0)
            .map(|b| b.hp)
            .sum()
    }

    pub fn state_hash(&self) -> u64 {
        let mut h: u64 = 0xCBF2_9CE4_8422_2325;
        let mix = |h: &mut u64, v: u64| {
            *h ^= v;
            *h = h.wrapping_mul(0x1000_0000_01B3);
        };
        mix(&mut h, self.tick as u64);
        for t in &self.teams {
            mix(&mut h, t.resources as u64);
            mix(&mut h, t.knowledge_xp as u64);
            mix(&mut h, t.kills as u64);
        }
        for b in &self.buildings {
            mix(&mut h, b.id as u64);
            mix(&mut h, b.hp as u64);
            mix(&mut h, ((b.x as u64) << 8) | (b.y as u64));
        }
        for u in &self.units {
            mix(&mut h, u.id as u64);
            mix(&mut h, u.hp as u64);
            mix(&mut h, ((u.px as u64) << 16) | (u.py as u64));
        }
        h
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn flat_map() -> [Terrain; MAP_TILES] {
        [Terrain::Open; MAP_TILES]
    }

    #[test]
    fn test_determinism() {
        let m = flat_map();
        let mut s1 = GameState::new_skirmish(
            12345,
            m,
            FactionTeam::one(Faction::Humans),
            FactionTeam::one(Faction::Orcs),
            (2, 7),
            (22, 7),
        );
        let mut s2 = s1.clone();

        s1.execute_action(TeamId::P1, Action::Build { kind: BuildingType::HumanMechFoundry, x: 5, y: 7 }).unwrap();
        s1.execute_action(TeamId::P2, Action::Build { kind: BuildingType::OrcWarDen, x: 19, y: 7 }).unwrap();
        s2.execute_action(TeamId::P1, Action::Build { kind: BuildingType::HumanMechFoundry, x: 5, y: 7 }).unwrap();
        s2.execute_action(TeamId::P2, Action::Build { kind: BuildingType::OrcWarDen, x: 19, y: 7 }).unwrap();

        s1.step_n(450);
        s2.step_n(450);
        assert_eq!(s1.state_hash(), s2.state_hash());
    }

    #[test]
    fn test_polarity_clash_and_flux_storm() {
        let m = flat_map();
        let mut s = GameState::new_skirmish(
            42,
            m,
            FactionTeam::one(Faction::Leones),
            FactionTeam::one(Faction::Dracones),
            (2, 7),
            (22, 7),
        );
        s.teams[0].resources = 500;
        s.teams[1].resources = 500;
        // Put Leone SunMonolith (+F+4 r=5) at (10,7) and Dracone VoidBrazier (-F+4 r=5) at (14,7)
        s.spawn_building_raw(TeamId::P1, BuildingType::LeoneSunMonolith, 10, 7, true);
        s.spawn_building_raw(TeamId::P2, BuildingType::DraconeVoidBrazier, 14, 7, true);
        s.refresh_caches();

        // At midpoint (12,7), +4 and -4 clash -> flux storm and 0 net!
        let mid = tile_idx(12, 7);
        assert!(s.energy_grid.flux_storm[mid]);
        assert_eq!(s.energy_grid.net[0][mid].pos_force, 0);
        assert_eq!(s.energy_grid.net[1][mid].neg_force, 0);
    }

    #[test]
    fn test_multi_faction_energy_sharing_and_leone_dracone_no_self_cancel() {
        let m = flat_map();
        let s = GameState::new_skirmish(
            99,
            m,
            FactionTeam::two(Faction::Leones, Faction::Dracones),
            FactionTeam::one(Faction::Humans),
            (2, 7),
            (22, 7),
        );
        // Combined Leone+Dracone Citadel must provide BOTH +Force and -Force in starter radius
        let idx = tile_idx(4, 7);
        assert!(s.energy_grid.net[0][idx].pos_force > 0);
        assert!(s.energy_grid.net[0][idx].neg_force > 0);
    }

    #[test]
    fn test_all_seven_faction_signature_mechanics() {
        let mut m = flat_map();
        m[tile_idx(4, 5)] = Terrain::OreVein;
        m[tile_idx(5, 5)] = Terrain::HighGround;

        // 1. Humans: Split Mech (Power Relay / Foundry) vs Compute (Media Array / Turrets)
        let mut s = GameState::new_skirmish(
            1,
            m,
            FactionTeam::one(Faction::Humans),
            FactionTeam::one(Faction::Humans),
            (2, 7),
            (22, 7),
        );
        s.teams[0].resources = 500;
        // Relay at (6,7) puts pure Mech (no Compute) onto tiles (8,7) and (9,7)
        s.execute_action(TeamId::P1, Action::Build { kind: BuildingType::HumanRelay, x: 6, y: 7 }).unwrap();
        s.step_n(20);
        assert!(s.energy_grid.net[0][tile_idx(9, 7)].mech > 0);
        assert_eq!(s.energy_grid.net[0][tile_idx(9, 7)].compute, 0);

        // Foundry on pure Mech is powered, while Autocannon without Compute is unpowered
        s.spawn_building_raw(TeamId::P1, BuildingType::HumanMechFoundry, 8, 7, true);
        s.spawn_building_raw(TeamId::P1, BuildingType::HumanAutocannon, 9, 7, true);
        s.refresh_caches();
        let fg = s.building_grid[tile_idx(8, 7)] as usize;
        let ag = s.building_grid[tile_idx(9, 7)] as usize;
        assert!(s.buildings[fg].powered);
        assert!(!s.buildings[ag].powered);

        // Powered Media Array at (9,6) emits Compute onto (9,7), powering Autocannon with +1 Compute link
        s.execute_action(TeamId::P1, Action::Build { kind: BuildingType::HumanMediaArray, x: 9, y: 6 }).unwrap();
        s.step_n(50);
        let ag = s.building_grid[tile_idx(9, 7)] as usize;
        assert!(s.buildings[ag].powered);
        assert_eq!(s.buildings[ag].net_links, 1);
        assert!(s.teams[0].last_cycle_income > 14);

        // Removing the Power Relay cascades Media Array and Autocannon unpowered, while Citadel covers (4,7)
        s.execute_action(TeamId::P1, Action::Salvage { x: 6, y: 7 }).unwrap();
        let mg = s.building_grid[tile_idx(9, 6)] as usize;
        let ag = s.building_grid[tile_idx(9, 7)] as usize;
        assert!(!s.buildings[mg].powered);
        assert!(!s.buildings[ag].powered);

        // 2. Elves: Ley Links multiply Scriptorium
        let mut s_elv = GameState::new_skirmish(
            2,
            m,
            FactionTeam::one(Faction::Elves),
            FactionTeam::one(Faction::Humans),
            (2, 7),
            (22, 7),
        );
        s_elv.teams[0].resources = 500;
        s_elv.spawn_building_raw(TeamId::P1, BuildingType::ElvenLeyConduit, 5, 7, true);
        s_elv.spawn_building_raw(TeamId::P1, BuildingType::ElvenScriptorium, 5, 4, true);
        s_elv.refresh_caches();
        assert!(!s_elv.energy_grid.ley_links.is_empty());

        // 3. Orcs: Devour adds fury stack
        let mut s_orc = GameState::new_skirmish(
            3,
            m,
            FactionTeam::one(Faction::Orcs),
            FactionTeam::one(Faction::Humans),
            (2, 7),
            (22, 7),
        );
        s_orc.spawn_building_raw(TeamId::P1, BuildingType::OrcSpikeBastion, 5, 7, true);
        s_orc.spawn_building_raw(TeamId::P1, BuildingType::OrcDoomCauldron, 4, 7, true);
        s_orc.refresh_caches();
        s_orc.execute_action(
            TeamId::P1,
            Action::Devour {
                cauldron_x: 4,
                cauldron_y: 7,
                victim_x: 5,
                victim_y: 7,
            },
        )
        .unwrap();
        let cg = s_orc.building_grid[tile_idx(4, 7)] as usize;
        assert_eq!(s_orc.buildings[cg].fury_stacks, 1);

        // 4. Dwarves: Knowledge tier + Tunnel Bore warp
        let mut s_dwa = GameState::new_skirmish(
            4,
            m,
            FactionTeam::one(Faction::Dwarves),
            FactionTeam::one(Faction::Humans),
            (2, 7),
            (22, 7),
        );
        s_dwa.spawn_building_raw(TeamId::P1, BuildingType::DwarfRuneArchive, 4, 5, true);
        s_dwa.spawn_building_raw(TeamId::P1, BuildingType::DwarfTunnelBore, 4, 7, true);
        s_dwa.spawn_building_raw(TeamId::P1, BuildingType::DwarfTunnelBore, 15, 7, true);
        s_dwa.spawn_building_raw(TeamId::P1, BuildingType::DwarfRunicHearth, 14, 7, true);
        s_dwa.step_n(320);
        assert!(s_dwa.teams[0].knowledge_tier >= 1);
        assert!(!s_dwa.units.is_empty());

        // 5. Gnomes: Tinker Gizmo + Walker relocation
        let mut s_gno = GameState::new_skirmish(
            5,
            m,
            FactionTeam::one(Faction::Gnomes),
            FactionTeam::one(Faction::Humans),
            (2, 7),
            (22, 7),
        );
        s_gno.spawn_building_raw(TeamId::P1, BuildingType::GnomeTinkerLab, 4, 7, true);
        s_gno.spawn_building_raw(TeamId::P1, BuildingType::GnomeWalkerTurret, 5, 7, true);
        s_gno.refresh_caches();
        let wg = s_gno.building_grid[tile_idx(5, 7)] as usize;
        assert!(s_gno.buildings[wg].gizmo_buff);
        s_gno.execute_action(
            TeamId::P1,
            Action::Relocate {
                x: 5,
                y: 7,
                tx: 7,
                ty: 7,
            },
        )
        .unwrap();
        s_gno.step_n(30);
        assert!(s_gno.building_grid[tile_idx(7, 7)] != GRID_EMPTY);

        // 6. Leones: ShapeTerrain RaisedStone + TectonicPush + Fallback wall bashing
        let mut s_leo = GameState::new_skirmish(
            6,
            m,
            FactionTeam::one(Faction::Leones),
            FactionTeam::one(Faction::Humans),
            (2, 7),
            (22, 7),
        );
        s_leo.teams[0].resources = 200;
        s_leo.spawn_building_raw(TeamId::P1, BuildingType::LeoneLithoShaper, 5, 7, true);
        s_leo.spawn_building_raw(TeamId::P1, BuildingType::LeoneTectonicRam, 4, 6, true);
        s_leo.spawn_building_raw(TeamId::P1, BuildingType::LeoneSolarPillar, 5, 6, true);
        s_leo.refresh_caches();
        s_leo
            .execute_action(
                TeamId::P1,
                Action::ShapeTerrain {
                    shaper_x: 5,
                    shaper_y: 7,
                    tx: 7,
                    ty: 7,
                },
            )
            .unwrap();
        assert_eq!(s_leo.terrain[tile_idx(7, 7)], Terrain::RaisedStone);
        s_leo
            .execute_action(
                TeamId::P1,
                Action::TectonicPush {
                    ram_x: 4,
                    ram_y: 6,
                    target_x: 5,
                    target_y: 6,
                },
            )
            .unwrap();
        assert!(s_leo.building_grid[tile_idx(6, 6)] != GRID_EMPTY);

        // 7. Dracones: Compound Hoard interest & Elder Wyrm threshold
        let mut s_dra = GameState::new_skirmish(
            7,
            m,
            FactionTeam::one(Faction::Dracones),
            FactionTeam::one(Faction::Humans),
            (2, 7),
            (22, 7),
        );
        s_dra.teams[0].resources = 300;
        s_dra.spawn_building_raw(TeamId::P1, BuildingType::DraconeHoardVault, 4, 7, true);
        s_dra.spawn_building_raw(TeamId::P1, BuildingType::DraconeElderSanctum, 5, 7, true);
        s_dra.step_n(200);
        assert!(s_dra.units.iter().any(|u| u.kind == UnitKind::ElderWyrm));
    }
}
