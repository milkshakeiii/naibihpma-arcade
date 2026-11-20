use super::candidates::{generate_candidates, ScoredAction, StrategicBias};
use super::eval::evaluate_state;
use crate::sim::rng::SimRng;
use crate::sim::types::*;
use crate::sim::GameState;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AiDifficulty {
    Easy,
    Normal,
    Hard,
}

impl AiDifficulty {
    pub const fn name(self) -> &'static str {
        match self {
            AiDifficulty::Easy => "EASY",
            AiDifficulty::Normal => "NORMAL",
            AiDifficulty::Hard => "HARD",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BotArchetype {
    Adaptive(AiDifficulty),
    Rush,
    Eco,
    TurtleTech,
    Greedy,
    RandomLegal,
    Passive,
}

impl BotArchetype {
    pub fn name(self) -> &'static str {
        match self {
            BotArchetype::Adaptive(d) => d.name(),
            BotArchetype::Rush => "RUSH",
            BotArchetype::Eco => "ECO",
            BotArchetype::TurtleTech => "TURTLE",
            BotArchetype::Greedy => "GREEDY",
            BotArchetype::RandomLegal => "RANDOM",
            BotArchetype::Passive => "PASSIVE",
        }
    }
}

pub fn fast_policy_action(state: &GameState, team: TeamId, bias: StrategicBias) -> Action {
    let cands = generate_candidates(state, team, bias, 4);
    if cands.is_empty() {
        return Action::NoOp;
    }
    // Never pick NoOp if there's an overwhelmingly high-prior immediate move unless NoOp beat it
    cands[0].action
}

fn simulate_rollout(
    state: &GameState,
    team: TeamId,
    action: Action,
    horizon_ticks: u32,
    delay_noop: bool,
) -> i32 {
    let mut sim = state.clone();
    sim.record_fx = false;
    sim.fx_beams.clear();
    sim.fx_bursts.clear();
    sim.fx_popups.clear();

    if action != Action::NoOp {
        let _ = sim.execute_action(team, action);
    }

    let foe = team.other();
    let mut t = 0u32;
    while t < horizon_ticks && sim.outcome == MatchOutcome::Ongoing {
        let step_chunk = 20.min(horizon_ticks - t);
        sim.step_n(step_chunk);
        t += step_chunk;
        if sim.outcome != MatchOutcome::Ongoing || t >= horizon_ticks {
            break;
        }
        // Enemy responds every 20 ticks
        let opp_act = fast_policy_action(&sim, foe, StrategicBias::Balanced);
        if opp_act != Action::NoOp {
            let _ = sim.execute_action(foe, opp_act);
        }
        // Own follow-up macro at exact 1:1 parity (after 40-tick saving window for NoOp)
        if !(delay_noop && t == 20) {
            let my_next = fast_policy_action(&sim, team, StrategicBias::Balanced);
            if my_next != Action::NoOp {
                let _ = sim.execute_action(team, my_next);
            }
        }
    }

    evaluate_state(&sim, team)
}

pub fn simulate_candidate(
    state: &GameState,
    team: TeamId,
    action: Action,
    horizon_ticks: u32,
) -> i32 {
    simulate_rollout(state, team, action, horizon_ticks, action == Action::NoOp)
}

pub fn choose_action(
    state: &GameState,
    team: TeamId,
    bot: BotArchetype,
    rng: &mut SimRng,
) -> Action {
    if state.outcome != MatchOutcome::Ongoing {
        return Action::NoOp;
    }

    match bot {
        BotArchetype::Passive => Action::NoOp,
        BotArchetype::RandomLegal => {
            let cands = generate_candidates(state, team, StrategicBias::Balanced, 20);
            if cands.is_empty() {
                Action::NoOp
            } else {
                let idx = rng.range_usize(0, cands.len());
                cands[idx].action
            }
        }
        BotArchetype::Rush => fast_policy_action(state, team, StrategicBias::Rush),
        BotArchetype::Eco => fast_policy_action(state, team, StrategicBias::Eco),
        BotArchetype::TurtleTech => fast_policy_action(state, team, StrategicBias::TurtleTech),
        BotArchetype::Greedy => fast_policy_action(state, team, StrategicBias::Balanced),
        BotArchetype::Adaptive(AiDifficulty::Easy) => {
            let cands = generate_candidates(state, team, StrategicBias::Balanced, 6);
            if cands.is_empty() {
                return Action::NoOp;
            }
            let top = cands.len().min(3);
            cands[rng.range_usize(0, top)].action
        }
        BotArchetype::Adaptive(AiDifficulty::Normal) => {
            let cands = generate_candidates(state, team, StrategicBias::Balanced, 4);
            let mut best = Action::NoOp;
            let mut best_val = i32::MIN;
            for c in cands {
                let v = simulate_candidate(state, team, c.action, 50);
                if v > best_val {
                    best_val = v;
                    best = c.action;
                }
            }
            best
        }
        BotArchetype::Adaptive(AiDifficulty::Hard) => {
            let mut cands = generate_candidates(state, team, StrategicBias::Balanced, 14);
            for bias in [StrategicBias::Rush, StrategicBias::Eco] {
                for extra in generate_candidates(state, team, bias, 1) {
                    if !cands.iter().any(|c| c.action == extra.action) {
                        cands.push(extra);
                    }
                }
            }
            // Force NoOp only when saving below 145, holding Dracone Hoard interest, or ReachResources challenge
            let ts = &state.teams[team.index()];
            let saving_goal = matches!(state.challenge_goal, Some(ChallengeGoal::ReachResources(_)));
            if (ts.resources < 145 || ts.config.has(Faction::Dracones) || saving_goal)
                && !cands.iter().any(|c| c.action == Action::NoOp)
            {
                cands.push(ScoredAction {
                    action: Action::NoOp,
                    prior: 10,
                });
            }

            let mut scored: Vec<(Action, i32)> = Vec::with_capacity(cands.len());
            for c in &cands {
                let val = simulate_candidate(state, team, c.action, 80);
                scored.push((c.action, val));
            }
            scored.sort_by(|a, b| b.1.cmp(&a.1));

            if scored.is_empty() {
                return Action::NoOp;
            }

            // 2-ply adversarial minimax refinement on top 3 candidates
            let refine_k = scored.len().min(3);
            let foe = team.other();
            let mut best_act = scored[0].0;
            let mut best_minimax = i32::MIN;

            for i in 0..refine_k {
                let act = scored[i].0;
                let mut base = state.clone();
                base.record_fx = false;
                if act != Action::NoOp {
                    let _ = base.execute_action(team, act);
                }
                base.step_n(20);
                let foe_cands = generate_candidates(&base, foe, StrategicBias::Balanced, 4);
                let mut worst_for_me = i32::MAX;
                if foe_cands.is_empty() {
                    worst_for_me = evaluate_state(&base, team);
                } else {
                    for fc in foe_cands {
                        let mut branch = base.clone();
                        if fc.action != Action::NoOp {
                            let _ = branch.execute_action(foe, fc.action);
                        }
                        let val = simulate_rollout(&branch, team, Action::NoOp, 60, false);
                        if val < worst_for_me {
                            worst_for_me = val;
                        }
                    }
                }
                // Blend 1-ply 80t + 2-ply minimax score
                let combined = (scored[i].1 * 2 + worst_for_me) / 3;
                if combined > best_minimax {
                    best_minimax = combined;
                    best_act = act;
                }
            }
            best_act
        }
    }
}

/// Incremental non-blocking AI controller for live 15 FPS play
pub struct LiveAiController {
    pub team: TeamId,
    pub difficulty: AiDifficulty,
    pub decision_interval: u32,
    pending_cands: Vec<ScoredAction>,
    next_cand_idx: usize,
    best_action: Action,
    best_score: i32,
    rng: SimRng,
}

impl LiveAiController {
    pub fn new(team: TeamId, difficulty: AiDifficulty, seed: u64) -> Self {
        let interval = match difficulty {
            AiDifficulty::Easy => 36,
            AiDifficulty::Normal => 28,
            AiDifficulty::Hard => 15,
        };
        Self {
            team,
            difficulty,
            decision_interval: interval,
            pending_cands: Vec::new(),
            next_cand_idx: 0,
            best_action: Action::NoOp,
            best_score: i32::MIN,
            rng: SimRng::new(seed),
        }
    }

    /// Steps incremental AI planning; executes at most 1 action when batch finishes.
    pub fn update(&mut self, state: &mut GameState) -> Option<Action> {
        if state.outcome != MatchOutcome::Ongoing {
            return None;
        }
        if self.difficulty == AiDifficulty::Easy {
            if state.tick % self.decision_interval == 0 {
                let act = choose_action(state, self.team, BotArchetype::Adaptive(AiDifficulty::Easy), &mut self.rng);
                if act != Action::NoOp && state.execute_action(self.team, act).is_ok() {
                    return Some(act);
                }
            }
            return None;
        }

        // Start a new batch every decision_interval ticks
        if self.pending_cands.is_empty() && state.tick % self.decision_interval == 0 {
            let max_c = if self.difficulty == AiDifficulty::Hard { 16 } else { 4 };
            let mut cands = generate_candidates(state, self.team, StrategicBias::Balanced, max_c);
            if !cands.iter().any(|c| c.action == Action::NoOp) {
                cands.push(ScoredAction {
                    action: Action::NoOp,
                    prior: 10,
                });
            }
            self.pending_cands = cands;
            self.next_cand_idx = 0;
            self.best_action = Action::NoOp;
            self.best_score = i32::MIN;
        }

        if !self.pending_cands.is_empty() {
            let horizon = if self.difficulty == AiDifficulty::Hard { 80 } else { 50 };
            let budget = if self.difficulty == AiDifficulty::Hard { 4 } else { 4 };
            let end = (self.next_cand_idx + budget).min(self.pending_cands.len());
            while self.next_cand_idx < end {
                let act = self.pending_cands[self.next_cand_idx].action;
                if act == Action::NoOp || state.can_execute(self.team, act).is_ok() {
                    let s = simulate_candidate(state, self.team, act, horizon);
                    if s > self.best_score {
                        self.best_score = s;
                        self.best_action = act;
                    }
                }
                self.next_cand_idx += 1;
            }
            if self.next_cand_idx >= self.pending_cands.len() {
                let chosen = self.best_action;
                self.pending_cands.clear();
                if chosen != Action::NoOp && state.execute_action(self.team, chosen).is_ok() {
                    return Some(chosen);
                }
            }
        }
        None
    }
}
