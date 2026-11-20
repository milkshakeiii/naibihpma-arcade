use crate::sim::defs::{building_cost, building_def, unit_def};
use crate::sim::energy::media_array_income;
use crate::sim::types::*;
use crate::sim::GameState;

pub fn evaluate_state(state: &GameState, perspective: TeamId) -> i32 {
    match state.outcome {
        MatchOutcome::Victory(w) => {
            return if w == perspective {
                1_000_000 - (state.tick as i32)
            } else {
                -1_000_000 + (state.tick as i32)
            };
        }
        MatchOutcome::Draw => return 0,
        MatchOutcome::Ongoing => {}
    }

    let me = perspective.index();
    let foe = perspective.other().index();

    let mut score = [0i32; 2];
    let mut citadel_pos = [(2u8, 7u8), (22u8, 7u8)];
    let mut eco_count = [0i32; 2];

    for b in &state.buildings {
        if b.hp <= 0 {
            continue;
        }
        let ti = b.owner.index();
        if b.kind == BuildingType::Citadel {
            citadel_pos[ti] = (b.x, b.y);
            score[ti] += b.hp * 18;
            continue;
        }

        let def = building_def(b.kind);
        let cost = building_cost(b.kind, state.teams[ti].config);
        let hp_ratio_val = (cost * b.hp) / b.max_hp.max(1);
        let mut val = hp_ratio_val * 6;

        if b.level >= 2 {
            val += cost * 4;
        }
        if !b.powered {
            val -= cost * 3;
        } else {
            val += 25;
        }

        match b.kind.slot() {
            Some(BuildingSlot::Eco) => {
                eco_count[ti] += 1;
                let inc_est = match b.kind {
                    BuildingType::HumanMediaArray => media_array_income(&state.terrain, &state.buildings, b),
                    BuildingType::ElvenScriptorium => 10 + (b.net_links.min(3) as i32) * 5,
                    BuildingType::DwarfRuneArchive => 15,
                    BuildingType::DraconeHoardVault => 16,
                    _ => 12,
                };
                if b.powered {
                    val += inc_est * 18;
                }
            }
            Some(BuildingSlot::Infra) => {
                val += 40 + (b.net_links as i32) * 14;
                // Forward infra control bonus
                let advance = if ti == 0 { b.x as i32 } else { (MAP_W as i32 - 1) - b.x as i32 };
                val += advance * 4;
            }
            Some(BuildingSlot::Defense) => {
                val += 60 + (b.net_links as i32) * 16 + if b.gizmo_buff { 35 } else { 0 };
                let advance = if ti == 0 { b.x as i32 } else { (MAP_W as i32 - 1) - b.x as i32 };
                val += advance * 3;
            }
            Some(BuildingSlot::Factory) => {
                val += 100;
                if b.kind == BuildingType::DwarfTunnelBore {
                    let advance = if ti == 0 { b.x as i32 } else { (MAP_W as i32 - 1) - b.x as i32 };
                    val += advance * 5;
                }
            }
            Some(BuildingSlot::Apex) => {
                val += 240 + (b.fury_stacks as i32) * 90 + (b.net_links as i32) * 20;
            }
            None => {}
        }
        if let Some(ukind) = def.spawn_kind {
            if b.powered
                && b.build_ticks_left == 0
                && def.spawn_interval > 0
                && (b.kind != BuildingType::DraconeElderSanctum || state.teams[ti].resources >= 150)
            {
                let udef = unit_def(ukind);
                let unit_est = (udef.max_hp * 2 + udef.damage * 4).min(360);
                let elapsed = def.spawn_interval.saturating_sub(b.spawn_timer) as i32;
                val += (unit_est * elapsed) / (def.spawn_interval as i32 * 2);
            }
        }
        score[ti] += val;
    }

    // Units & Citadel Proximity Threat
    for u in &state.units {
        if u.hp <= 0 {
            continue;
        }
        let ti = u.owner.index();
        let opp = u.owner.other().index();
        let udef = unit_def(u.kind);
        let mut uval = (u.hp * 2) + udef.damage * 4;

        let target_cx = (citadel_pos[opp].0 as i32) * TILE_PX + 8;
        let target_cy = (citadel_pos[opp].1 as i32) * TILE_PX + 8;
        let d_px = ((u.px - target_cx).abs() + (u.py - target_cy).abs()) / TILE_PX;
        // Closer to enemy citadel is much higher pressure
        uval += (26 - d_px.min(26)) * 8;
        if d_px <= 5 {
            // Immediate threat to enemy citadel
            score[opp] -= 110;
        }
        score[ti] += uval;
    }

    // Resources, Knowledge Tiers, Hoard synergy
    for ti in 0..2 {
        let res_w = if state.teams[ti].config.has(Faction::Dracones) { 4 } else { 2 };
        score[ti] += state.teams[ti].resources * res_w;
        score[ti] += (state.teams[ti].knowledge_tier as i32) * 140;
        score[ti] += (state.teams[ti].kills as i32) * 12;
    }

    // Challenge goal adjustments for P1 solver
    if let Some(goal) = state.challenge_goal {
        if perspective == TeamId::P1 {
            match goal {
                ChallengeGoal::ReachResources(target) => {
                    let res = state.teams[0].resources.min(target);
                    let prog = res * 5 + (res * res) / target.max(1);
                    score[0] += prog + eco_count[0].min(3) * 220;
                }
                ChallengeGoal::SurviveTicks(_) => {
                    score[0] += state.citadel_hp(TeamId::P1) * 10;
                }
                ChallengeGoal::DestroyAllEnemyStructures => {
                    let foe_hp: i32 = state
                        .buildings
                        .iter()
                        .filter(|b| b.owner == TeamId::P2 && b.hp > 0)
                        .map(|b| b.hp)
                        .sum();
                    score[0] -= foe_hp * 24;
                }
                ChallengeGoal::DestroyEnemyCitadel => {}
            }
        }
    }

    score[me] - score[foe]
}
