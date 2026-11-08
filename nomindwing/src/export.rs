//! Headless tools: 2x screenshots of the main screens, and AI-vs-AI match statistics for tuning.

use crate::ai::Ai;
use crate::design::Kind;
use crate::framebuffer::Framebuffer;
use crate::game::Game;
use crate::render::VIEW;
use crate::world::{Cmd, World, WORLD_H, WORLD_W};
use std::path::Path;

fn run_until(g: &mut Game, max_tick: u32, stop: impl Fn(&Game) -> bool) {
    while g.w.tick < max_tick && g.w.winner.is_none() && !stop(g) {
        g.tick();
    }
}

fn look_at(g: &mut Game, x: i32, y: i32) {
    g.follow = false;
    g.cam = ((x - VIEW.w / 2).clamp(0, WORLD_W - VIEW.w), (y - VIEW.h / 2).clamp(0, WORLD_H - VIEW.h));
}

pub fn screenshots(dir: &Path) {
    let _ = std::fs::create_dir_all(dir);
    let mut fb = Framebuffer::new();
    let save = |fb: &Framebuffer, g: &Game, name: &str| fb.save_png(&dir.join(name), g.palette());

    let mut g = Game::new(7, None);
    g.draw(&mut fb);
    save(&fb, &g, "01_rules.png");

    g.help = false;
    g.autopilot = Some(Ai::new(0, &g.w));
    run_until(&mut g, 15 * 40, |_| false);
    g.mouse = (crate::game::card_rect(2).x + 20, crate::game::card_rect(2).y + 10);
    g.draw(&mut fb);
    save(&fb, &g, "02_opening.png");

    // The awe shot: a seed eating its rock into a base.
    run_until(&mut g, 15 * 300, |g| g.w.ships.iter().any(|s| s.alive && s.team == 0 && s.kind.is_base() && s.grown * 2 > s.design().count && !s.complete()));
    if let Some(b) = g.w.ships.iter().find(|s| s.alive && s.team == 0 && s.kind.is_base() && !s.complete()) {
        let (x, y) = (b.x, b.y);
        look_at(&mut g, x, y);
        g.mouse = (VIEW.w / 2, VIEW.h / 2);
    }
    g.draw(&mut fb);
    save(&fb, &g, "03_growing.png");
    g.follow = true;

    run_until(&mut g, 15 * 200, |_| false);
    g.tactical = true;
    g.mouse = (0, 0);
    g.draw(&mut fb);
    save(&fb, &g, "04_tactical.png");
    g.tactical = false;

    // Planting: the crosshair over the free rock nearest the king.
    let (kx, ky) = (g.w.king(0).x, g.w.king(0).y);
    if let Some(a) = g.w.free_rock_near(kx, ky, 2000, 0, None) {
        let (rx, ry) = (g.w.rocks[a].cx, g.w.rocks[a].cy);
        look_at(&mut g, (rx + kx) / 2, (ry + ky) / 2);
        g.targeting = Some(Kind::OutpostSeed);
        g.mouse = (rx - g.cam.0, ry - g.cam.1);
        g.draw(&mut fb);
        save(&fb, &g, "05_planting.png");
        g.targeting = None;
    }
    g.follow = true;

    run_until(&mut g, 15 * 900, |g| g.w.shots.len() >= 14);
    if let Some(s) = g.w.shots.first() {
        let (x, y) = (s.line.x, s.line.y);
        look_at(&mut g, x, y);
    }
    g.mouse = (0, 0);
    g.draw(&mut fb);
    save(&fb, &g, "06_battle.png");
    g.follow = true;

    run_until(&mut g, 15 * 60 * 25, |_| false);
    g.draw(&mut fb);
    save(&fb, &g, "07_end.png");
    println!("exported screenshots to {} (match ended at tick {}, winner {:?})", dir.display(), g.w.tick, g.w.winner);
}

pub struct Outcome {
    pub winner: Option<usize>,
    pub ticks: u32,
    pub w: World,
}

/// Plays one headless match. `left` drives the player's side, or leaves it idle when `None`.
pub fn play(seed: u64, left: bool, max_ticks: u32) -> Outcome {
    let mut w = World::new(seed);
    let mut a0 = Ai::new(0, &w);
    let mut a1 = Ai::new(1, &w);
    let mut cmds: Vec<Cmd> = Vec::new();
    while w.winner.is_none() && w.tick < max_ticks {
        cmds.clear();
        w.steer[0] = if left { a0.step(&w, &mut cmds) } else { (0, 0) };
        w.steer[1] = a1.step(&w, &mut cmds);
        w.tick(&cmds);
        w.events.clear();
    }
    Outcome { winner: w.winner, ticks: w.tick, w }
}

/// Prints a line every 20 seconds of one headless match, for finding out why a match stalls.
pub fn trace(seed: u64, left: bool) {
    let mut w = World::new(seed);
    let mut a0 = Ai::new(0, &w);
    let mut a1 = Ai::new(1, &w);
    let mut cmds: Vec<Cmd> = Vec::new();
    while w.winner.is_none() && w.tick < 15 * 60 * 8 {
        cmds.clear();
        w.steer[0] = if left { a0.step(&w, &mut cmds) } else { (0, 0) };
        w.steer[1] = a1.step(&w, &mut cmds);
        w.tick(&cmds);
        w.events.clear();
        if w.tick % (15 * 20) == 0 {
            for t in 0..2 {
                let n = |k: Kind| w.ships.iter().filter(|s| s.alive && s.team == t && s.kind == k).count();
                let hunting = w.ships.iter().filter(|s| s.alive && s.team == t && s.kind == Kind::Raider && s.hunting).count();
                let k = w.king(t);
                let foe = w.king(1 - t);
                let near = w.ships.iter().filter(|s| s.alive && s.team == t && s.kind == Kind::Raider).map(|s| crate::geom::cheb(s.x, s.y, foe.x, foe.y)).min();
                println!(
                    "{:>5}s T{t} mode {:?} px {:4} king {:3}% at ({},{}) col {} esc {} raid {} (hunting {}, nearest to foe king {:?}) out {} fwd {} seeds {}",
                    w.tick / 15,
                    if t == 0 { a0.mode } else { a1.mode },
                    w.teams[t].bank,
                    k.hp_pct(),
                    k.x,
                    k.y,
                    n(Kind::Collector),
                    n(Kind::Escort),
                    n(Kind::Raider),
                    hunting,
                    near,
                    n(Kind::Outpost),
                    n(Kind::Forward),
                    n(Kind::OutpostSeed) + n(Kind::ForwardSeed)
                );
            }
        }
    }
    println!("winner {:?} at {}s", w.winner, w.tick / 15);
}

pub fn sim(n: u64) {
    let fmt = |t: u32| format!("{:02}:{:02}", t / 15 / 60, t / 15 % 60);
    println!("AI vs AI");
    for seed in 1..=n {
        let o = play(seed, true, 15 * 60 * 25);
        let (a, b) = (&o.w.teams[0], &o.w.teams[1]);
        let bases = |t: usize| o.w.ships.iter().filter(|s| s.alive && s.team == t && s.kind.is_base()).count();
        println!(
            "  seed {seed:2}: winner {:?} at {}  mined {}/{}  planted {}/{}  bases left {}/{}  kills {}/{}",
            o.winner,
            fmt(o.ticks),
            a.mined,
            b.mined,
            a.planted,
            b.planted,
            bases(0),
            bases(1),
            a.kills,
            b.kills
        );
    }
    println!("AI vs an idle king");
    for seed in 1..=n {
        let o = play(seed, false, 15 * 60 * 25);
        println!("  seed {seed:2}: winner {:?} at {}  enemy planted {}", o.winner, fmt(o.ticks), o.w.teams[1].planted);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn an_idle_king_falls() {
        let o = play(1, false, 15 * 60 * 20);
        assert_eq!(o.winner, Some(1), "the AI never killed an idle king");
    }

    #[test]
    fn ai_matches_end() {
        let o = play(2, true, 15 * 60 * 30);
        assert!(o.winner.is_some(), "AI vs AI stalled for 30 minutes");
    }
}
