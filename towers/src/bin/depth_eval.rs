use std::time::Instant;

use towers::ai::search::{choose_action, simulate_candidate, AiDifficulty, BotArchetype};
use towers::content::challenges::{all_challenges, create_challenge_state, step_challenge_waves};
use towers::content::maps::build_maps;
use towers::sim::rng::SimRng;
use towers::sim::types::*;
use towers::sim::GameState;

fn play_match(
    map_id: usize,
    p1_team: FactionTeam,
    p2_team: FactionTeam,
    b1: BotArchetype,
    b2: BotArchetype,
    seed: u64,
    max_ticks: u32,
) -> (MatchOutcome, u32) {
    let maps = build_maps();
    let m = &maps[map_id % maps.len()];
    let mut state = GameState::new_skirmish(seed, m.tiles, p1_team, p2_team, m.p1_citadel, m.p2_citadel);
    state.record_fx = false;

    let mut r1 = SimRng::new(seed ^ 0xA11CE);
    let mut r2 = SimRng::new(seed ^ 0xB0B0);
    let cad1 = match b1 {
        BotArchetype::Adaptive(AiDifficulty::Easy) => 36,
        BotArchetype::Adaptive(AiDifficulty::Normal) => 28,
        _ => 15,
    };
    let cad2 = match b2 {
        BotArchetype::Adaptive(AiDifficulty::Easy) => 36,
        BotArchetype::Adaptive(AiDifficulty::Normal) => 28,
        _ => 15,
    };

    while state.outcome == MatchOutcome::Ongoing && state.tick < max_ticks {
        if state.tick % cad1 == 0 {
            let a = choose_action(&state, TeamId::P1, b1, &mut r1);
            if a != Action::NoOp {
                let _ = state.execute_action(TeamId::P1, a);
            }
        }
        if state.tick % cad2 == 0 {
            let a = choose_action(&state, TeamId::P2, b2, &mut r2);
            if a != Action::NoOp {
                let _ = state.execute_action(TeamId::P2, a);
            }
        }
        state.step();
    }
    (state.outcome, state.tick)
}

fn eval_pair(b1: BotArchetype, b2: BotArchetype, games: usize) -> (usize, usize, usize, u32) {
    let pairs = [
        (Faction::Humans, Faction::Orcs),
        (Faction::Elves, Faction::Dwarves),
        (Faction::Gnomes, Faction::Leones),
        (Faction::Dracones, Faction::Humans),
        (Faction::Orcs, Faction::Leones),
        (Faction::Dwarves, Faction::Dracones),
        (Faction::Gnomes, Faction::Elves),
    ];
    let mut w1 = 0usize;
    let mut w2 = 0usize;
    let mut draws = 0usize;
    let mut sum_ticks = 0u64;

    for g in 0..games {
        let (f1, f2) = pairs[g % pairs.len()];
        let map_id = (g / 2) % 6;
        let swap = g % 2 == 1;
        let (t1, t2, a1, a2) = if !swap {
            (FactionTeam::one(f1), FactionTeam::one(f2), b1, b2)
        } else {
            (FactionTeam::one(f2), FactionTeam::one(f1), b2, b1)
        };
        let (out, ticks) = play_match(map_id, t1, t2, a1, a2, 1000 + g as u64 * 17, 4500);
        sum_ticks += ticks as u64;
        match out {
            MatchOutcome::Victory(winner) => {
                let b1_won = (!swap && winner == TeamId::P1) || (swap && winner == TeamId::P2);
                if b1_won {
                    w1 += 1;
                } else {
                    w2 += 1;
                }
            }
            _ => draws += 1,
        }
    }
    (w1, w2, draws, (sum_ticks / games.max(1) as u64) as u32)
}

fn main() {
    println!("=============================================================");
    println!("  CITADEL GRID - STRATEGIC DEPTH & AI BALANCE EVALUATION");
    println!("=============================================================\n");

    // 0. Rollout speed benchmark
    let maps = build_maps();
    let bench_state = GameState::new_skirmish(
        1,
        maps[0].tiles,
        FactionTeam::one(Faction::Humans),
        FactionTeam::one(Faction::Orcs),
        maps[0].p1_citadel,
        maps[0].p2_citadel,
    );
    let t0 = Instant::now();
    let mut sink = 0i64;
    for _ in 0..100 {
        sink += simulate_candidate(&bench_state, TeamId::P1, Action::NoOp, 180) as i64;
    }
    let us_per_rollout = t0.elapsed().as_micros() / 100;
    println!("[0] 180-tick forward rollout speed: {} us/rollout (checksum {})\n", us_per_rollout, sink);

    // 1. Search-Depth Scaling
    println!("[1] SEARCH-DEPTH SCALING (Side-swapped across factions & maps)");
    for (name, opp) in [
        ("Hard vs RandomLegal", BotArchetype::RandomLegal),
        ("Hard vs Greedy     ", BotArchetype::Greedy),
        ("Hard vs Easy       ", BotArchetype::Adaptive(AiDifficulty::Easy)),
        ("Hard vs Normal     ", BotArchetype::Adaptive(AiDifficulty::Normal)),
    ] {
        let games = 14;
        let (w1, w2, d, avg_t) = eval_pair(BotArchetype::Adaptive(AiDifficulty::Hard), opp, games);
        let pct = (w1 * 100) / games;
        println!("  {} : {:2}W / {:2}L / {:2}D ({:3}% win, avg {} ticks)", name, w1, w2, d, pct, avg_t);
    }
    println!();

    // 2. Strategic Archetype Triangle
    println!("[2] STRATEGIC ARCHETYPE TRIANGLE");
    for (name, a, b) in [
        ("Rush vs Eco         ", BotArchetype::Rush, BotArchetype::Eco),
        ("Eco vs TurtleTech   ", BotArchetype::Eco, BotArchetype::TurtleTech),
        ("TurtleTech vs Rush  ", BotArchetype::TurtleTech, BotArchetype::Rush),
        ("Hard vs Rush        ", BotArchetype::Adaptive(AiDifficulty::Hard), BotArchetype::Rush),
        ("Hard vs Eco         ", BotArchetype::Adaptive(AiDifficulty::Hard), BotArchetype::Eco),
        ("Hard vs TurtleTech  ", BotArchetype::Adaptive(AiDifficulty::Hard), BotArchetype::TurtleTech),
    ] {
        let games = 14;
        let (w1, w2, d, _) = eval_pair(a, b, games);
        let pct = (w1 * 100) / games;
        println!("  {} : {:2}W / {:2}L / {:2}D ({:3}% win)", name, w1, w2, d, pct);
    }
    println!();

    // 3. 7x7 Faction Round-Robin Balance Matrix (Normal vs Normal)
    println!("[3] 7x7 FACTION ROUND-ROBIN (Normal AI, 4 games/pair, side-swapped)");
    let mut fac_wins = [0usize; 7];
    let mut fac_games = [0usize; 7];
    let mut matrix = [[0usize; 7]; 7];

    for i in 0..7 {
        for j in (i + 1)..7 {
            let fi = Faction::ALL[i];
            let fj = Faction::ALL[j];
            for g in 0..4 {
                let swap = g % 2 == 1;
                let map_id = (i + j + g) % 6;
                let (t1, t2) = if !swap {
                    (FactionTeam::one(fi), FactionTeam::one(fj))
                } else {
                    (FactionTeam::one(fj), FactionTeam::one(fi))
                };
                let (out, _) = play_match(
                    map_id,
                    t1,
                    t2,
                    BotArchetype::Adaptive(AiDifficulty::Normal),
                    BotArchetype::Adaptive(AiDifficulty::Normal),
                    5000 + (i * 31 + j * 13 + g) as u64,
                    5000,
                );
                fac_games[i] += 1;
                fac_games[j] += 1;
                if let MatchOutcome::Victory(w) = out {
                    let i_won = (!swap && w == TeamId::P1) || (swap && w == TeamId::P2);
                    if i_won {
                        fac_wins[i] += 1;
                        matrix[i][j] += 1;
                    } else {
                        fac_wins[j] += 1;
                        matrix[j][i] += 1;
                    }
                }
            }
        }
    }
    print!("          ");
    for f in Faction::ALL {
        print!(" {:>4}", f.short_name());
    }
    println!(" |  WIN%");
    for i in 0..7 {
        print!("  {:>7} ", Faction::ALL[i].name());
        for j in 0..7 {
            if i == j {
                print!("   --");
            } else {
                print!("  {:>2}/4", matrix[i][j]);
            }
        }
        let pct = (fac_wins[i] * 100) / fac_games[i].max(1);
        println!(" | {:3}%", pct);
    }
    println!();

    // 4. Multi-Faction Synergy (Pure vs Dual vs Triad)
    println!("[4] MULTI-FACTION TEAM COMPARISON (Pure vs Dual vs Triad)");
    let pure_teams = [
        FactionTeam::one(Faction::Humans),
        FactionTeam::one(Faction::Elves),
        FactionTeam::one(Faction::Orcs),
        FactionTeam::one(Faction::Dwarves),
        FactionTeam::one(Faction::Gnomes),
        FactionTeam::one(Faction::Leones),
        FactionTeam::one(Faction::Dracones),
    ];
    let dual_teams = [
        FactionTeam::two(Faction::Humans, Faction::Gnomes),
        FactionTeam::two(Faction::Elves, Faction::Gnomes),
        FactionTeam::two(Faction::Dwarves, Faction::Leones),
        FactionTeam::two(Faction::Orcs, Faction::Dracones),
        FactionTeam::two(Faction::Humans, Faction::Leones),
        FactionTeam::two(Faction::Elves, Faction::Dracones),
    ];
    let triad_teams = [
        FactionTeam::three(Faction::Humans, Faction::Gnomes, Faction::Dwarves),
        FactionTeam::three(Faction::Elves, Faction::Gnomes, Faction::Leones),
        FactionTeam::three(Faction::Orcs, Faction::Dracones, Faction::Humans),
        FactionTeam::three(Faction::Dwarves, Faction::Leones, Faction::Elves),
    ];

    for (label, a_pool, b_pool) in [
        ("Dual vs Pure ", &dual_teams[..], &pure_teams[..]),
        ("Triad vs Pure", &triad_teams[..], &pure_teams[..]),
        ("Triad vs Dual", &triad_teams[..], &dual_teams[..]),
    ] {
        let mut wa = 0usize;
        let mut wb = 0usize;
        let mut dr = 0usize;
        let games = 16;
        for g in 0..games {
            let ta = a_pool[g % a_pool.len()];
            let tb = b_pool[(g * 3) % b_pool.len()];
            let swap = g % 2 == 1;
            let (t1, t2) = if !swap { (ta, tb) } else { (tb, ta) };
            let (out, _) = play_match(
                g % 6,
                t1,
                t2,
                BotArchetype::Adaptive(AiDifficulty::Normal),
                BotArchetype::Adaptive(AiDifficulty::Normal),
                8800 + g as u64,
                5000,
            );
            match out {
                MatchOutcome::Victory(w) => {
                    if (!swap && w == TeamId::P1) || (swap && w == TeamId::P2) {
                        wa += 1;
                    } else {
                        wb += 1;
                    }
                }
                _ => dr += 1,
            }
        }
        println!(
            "  {} : {:2}W / {:2}L / {:2}D ({:3}%)",
            label,
            wa,
            wb,
            dr,
            (wa * 100) / games
        );
    }
    println!();

    // 5. All Challenges Verification
    let challenges = all_challenges();
    println!("[5] ALL {} CHALLENGES VERIFICATION (Hard Solver vs Passive NoOp)", challenges.len());
    let mut passive_wins = 0usize;
    let mut ai_wins = 0usize;
    for ch in &challenges {
        // Test Passive
        let mut s_pass = create_challenge_state(ch, 777);
        s_pass.record_fx = false;
        let mut r_foe = SimRng::new(999);
        while s_pass.outcome == MatchOutcome::Ongoing && s_pass.tick < 4500 {
            step_challenge_waves(ch, &mut s_pass);
            if ch.uses_ai && s_pass.tick % 28 == 0 {
                let a = choose_action(&s_pass, TeamId::P2, BotArchetype::Adaptive(AiDifficulty::Normal), &mut r_foe);
                if a != Action::NoOp {
                    let _ = s_pass.execute_action(TeamId::P2, a);
                }
            }
            s_pass.step();
        }
        let pass_won = s_pass.outcome == MatchOutcome::Victory(TeamId::P1);
        if pass_won {
            passive_wins += 1;
        }

        // Test Hard AI Solver
        let mut s_ai = create_challenge_state(ch, 777);
        s_ai.record_fx = false;
        let mut r_p1 = SimRng::new(333);
        let mut r_p2 = SimRng::new(999);
        while s_ai.outcome == MatchOutcome::Ongoing && s_ai.tick < 4500 {
            step_challenge_waves(ch, &mut s_ai);
            if s_ai.tick % 15 == 0 {
                let a = choose_action(&s_ai, TeamId::P1, BotArchetype::Adaptive(AiDifficulty::Hard), &mut r_p1);
                if a != Action::NoOp {
                    let _ = s_ai.execute_action(TeamId::P1, a);
                }
            }
            if ch.uses_ai && s_ai.tick % 28 == 0 {
                let a = choose_action(&s_ai, TeamId::P2, BotArchetype::Adaptive(AiDifficulty::Normal), &mut r_p2);
                if a != Action::NoOp {
                    let _ = s_ai.execute_action(TeamId::P2, a);
                }
            }
            s_ai.step();
        }
        let ai_won = s_ai.outcome == MatchOutcome::Victory(TeamId::P1);
        if ai_won {
            ai_wins += 1;
        }
        println!(
            "  [{:2}] {:<24} | Passive: {:<4} | Hard AI: {:<4} ({} ticks)",
            ch.id + 1,
            ch.title,
            if pass_won { "PASS" } else { "FAIL" },
            if ai_won { "PASS" } else { "FAIL" },
            s_ai.tick
        );
    }
    println!(
        "\nSUMMARY: Passive wins {}/{} (want 0/{}) | Hard AI wins {}/{} (want {}/{})",
        passive_wins,
        challenges.len(),
        challenges.len(),
        ai_wins,
        challenges.len(),
        challenges.len(),
        challenges.len()
    );
}
