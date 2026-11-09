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

// ----- Proving ground -----

/// Plays every ready-made test plus a few extra matchups with the default designs and prints
/// who won, how fast, and how accurate each side was. The arena is deterministic, so one run
/// each is the whole story.
pub fn pg_sim() {
    use crate::pg::design::defaults;
    use crate::pg::sim::Arena;
    use crate::pg::{block, tests};
    let lib = defaults();
    // One replay can be luck, so every matchup runs on SEEDS rolls of the dice. Seed 1 is the
    // fight the window shows.
    const SEEDS: u64 = 12;
    let report = |name: &str, place: &[(usize, usize, i32, i32)]| {
        let (mut wins, mut secs, mut lost, mut px, mut window) = ([0; 3], 0, [0; 2], [0; 2], String::new());
        let (mut placed, mut stale, mut splits) = ([0; 2], 0, [0; 2]);
        for seed in 1..=SEEDS {
            let mut a = Arena::seeded(seed);
            for &(team, slot, x, y) in place {
                a.add(team, slot, &lib[slot], x, y);
            }
            while a.over.is_none() {
                a.step();
            }
            let k = match a.over.unwrap() {
                Some(0) => 0,
                Some(_) => 1,
                None => 2,
            };
            wins[k] += 1;
            stale += a.stalemate as i32;
            secs += a.tick / 15;
            for t in 0..2 {
                lost[t] += a.score[t].lost;
                px[t] += a.score[t].px_lost;
                placed[t] = a.score[t].placed - a.score[t].split;
                splits[t] += a.score[t].split;
            }
            if seed == 1 {
                let t = a.tick / 15;
                window = format!("{} {}:{:02}", ["BLUE", "RED", "DRAW"][k], t / 60, t % 60);
            }
        }
        let n = SEEDS as i32;
        let t = secs / SEEDS as u32;
        println!(
            "{name:34} blue {:2}  red {:2}  draw {:2} ({:2} stalemate)  avg {}:{:02}  lost: blue {}/{} ({} px)  red {}/{} ({} px)  splits {}/{}  window: {window}",
            wins[0],
            wins[1],
            wins[2],
            stale,
            t / 60,
            t % 60,
            lost[0] / n,
            placed[0],
            px[0] / n,
            lost[1] / n,
            placed[1],
            px[1] / n,
            splits[0],
            splits[1]
        );
    };
    println!("wins out of {SEEDS} seeds; losses are averages (ships lost can top ships placed when they split); splits are totals; \"window\" is the fight the window replays");
    println!("--- ready-made tests ---");
    for t in tests().iter().filter(|t| !t.place.is_empty()) {
        report(t.name, &t.place);
    }
    // Every pair of designs at (roughly) the same budget: the swarm question. Cost is pixels plus
    // extra cores.
    let budget = 600;
    println!("--- equal cost, about {budget} px a side ---");
    let gap = |s: usize| {
        let sh = lib[s].shape();
        sh.w.max(sh.h) + 4
    };
    let count = |s: usize| (budget / lib[s].stats().cost).max(1);
    for a in 0..5 {
        for b in a + 1..5 {
            let mut place = block(0, a, count(a), 130, 180, gap(a));
            place.extend(block(1, b, count(b), 350, 180, gap(b)));
            let name = format!("{} {} VS {} {}", count(a), lib[a].name, count(b), lib[b].name);
            report(&name, &place);
        }
    }
    // The titan against the bigger designs at its own cost (small ships would need hundreds).
    let titan = lib[6].stats().cost;
    println!("--- equal cost, the titan's {titan} px a side ---");
    for b in 2..5 {
        let n = (titan / lib[b].stats().cost).max(1);
        let mut place = block(0, 6, 1, 110, 180, 0);
        place.extend(block(1, b, n, 360, 180, gap(b)));
        report(&format!("1 TITAN VS {n} {}", lib[b].name), &place);
    }
}

/// Prints every ship's state every few ticks of one matchup: `blue` and `red` are (slot, count).
/// A blue slot of 99 plays ready-made test number `red.0` instead.
pub fn pg_trace(blue: (usize, i32), red: (usize, i32), every: u32, until: u32) {
    use crate::pg::design::defaults;
    use crate::pg::sim::Arena;
    use crate::pg::{block, tests};
    let lib = defaults();
    let mut a = Arena::new();
    let place = if blue.0 == 99 {
        tests()[red.0].place.clone()
    } else {
        let mut p = block(0, blue.0, blue.1, 130, 180, 12);
        p.extend(block(1, red.0, red.1, 350, 180, 12));
        p
    };
    for &(team, slot, x, y) in &place {
        a.add(team, slot, &lib[slot], x, y);
    }
    while a.over.is_none() && a.tick < until {
        a.step();
        if a.tick % every == 0 {
            println!(
                "tick {} shots {} | blue fired {} hit {} bounced {} | red fired {} hit {} bounced {}",
                a.tick,
                a.shots.len(),
                a.score[0].fired,
                a.score[0].hits,
                a.score[0].bounced,
                a.score[1].fired,
                a.score[1].hits,
                a.score[1].bounced
            );
            for s in a.ships.iter().filter(|s| s.alive) {
                println!(
                    "   {} {:9} ({:3},{:3}) v=({:3},{:3}) face {} {:16} px {}/{} flak {} lance {} cannon {}",
                    s.team,
                    s.name,
                    s.x,
                    s.y,
                    s.vx,
                    s.vy,
                    s.facing,
                    s.doing.name(),
                    s.px_now(),
                    s.px_full(),
                    s.st.flak,
                    s.st.lance,
                    s.st.cannon
                );
            }
        }
    }
    println!("over {:?} at tick {}", a.over, a.tick);
}

pub fn pg_screenshots(dir: &Path) {
    use crate::app::App;
    use crate::pg::{Pg, Screen};
    let _ = std::fs::create_dir_all(dir);
    let mut fb = Framebuffer::new();
    let save = |fb: &Framebuffer, pg: &Pg, name: &str| fb.save_png(&dir.join(name), pg.palette());
    let hover = |pg: &mut Pg, team: usize| {
        if let Some(s) = pg.arena.ships.iter().filter(|s| s.alive && s.team == team).max_by_key(|s| s.px_full()) {
            pg.mouse = (s.x - pg.cam.0, s.y - pg.cam.1);
        }
    };

    let mut pg = Pg::new(None, None);
    pg.draw(&mut fb);
    save(&fb, &pg, "01_rules.png");
    pg.help = false;
    pg.mouse = (150, 240);
    pg.draw(&mut fb);
    save(&fb, &pg, "02_setup.png");

    for (t, ticks, team, name) in [
        (0, 60, 1, "03_fighters_vs_frigate.png"),
        (1, 120, 1, "04_lancers_vs_frigate.png"),
        (3, 150, 0, "05_destroyer_vs_frigates.png"),
        (4, 75, 0, "06_capital_vs_fighters.png"),
        (5, 36, 0, "07_capital_vs_lancers.png"),
    ] {
        pg.load_test(t);
        pg.running = true;
        for _ in 0..ticks {
            pg.tick();
        }
        hover(&mut pg, team);
        pg.draw(&mut fb);
        save(&fb, &pg, name);
    }
    while pg.arena.over.is_none() {
        pg.tick();
    }
    pg.mouse = (0, 0);
    pg.draw(&mut fb);
    save(&fb, &pg, "08_result.png");

    use crate::pg::design::{Part, GRID};
    let c = GRID / 2;
    pg.running = false;
    pg.screen = Screen::Yard;
    pg.select_slot(4);
    pg.mouse = pg.cell_screen((c + 3, c - 6));
    pg.draw(&mut fb);
    save(&fb, &pg, "09_yard_capital.png");
    // A loose pixel, and a skirt of hull that buries both engines: flagged in red.
    pg.lib[0] = crate::pg::design::defaults()[0].clone();
    pg.lib[0].set(c + 2, c - 3, Some(Part::Hull));
    for (x, y) in [(-2, 0), (-2, 1), (-2, 2), (-1, 2), (0, 2), (1, 2), (2, 2), (2, 1), (2, 0)] {
        pg.lib[0].set(c + x, c + y, Some(Part::Hull));
    }
    pg.select_slot(0);
    pg.ed.part = Part::Cannon;
    pg.ed.cat = crate::pg::design::Cat::Weapons;
    pg.mouse = pg.cell_screen((c + 7, c - 6));
    pg.draw(&mut fb);
    save(&fb, &pg, "10_yard_loose_pixels.png");
    pg.lib[0] = crate::pg::design::defaults()[0].clone();
    // The titan: the whole 99x99 grid at 2x, then at its real size.
    pg.select_slot(6);
    pg.mouse = pg.cell_screen((c + 12, 62));
    pg.draw(&mut fb);
    save(&fb, &pg, "11_yard_titan.png");
    pg.ed.zoom = 1;
    pg.ed.pan = ((198 - GRID) / 2, (198 - GRID) / 2);
    pg.mouse = (300, 290);
    pg.draw(&mut fb);
    save(&fb, &pg, "12_yard_titan_real_size.png");
    // A stamp: the frigate picked up whole, about to land on a blank design, mirrored.
    pg.lib[5] = crate::pg::design::Design::blank("NEW SHIP");
    pg.select_slot(5);
    pg.ed.zoom = 4;
    pg.fit_view();
    pg.stamp_design(2);
    pg.mouse = pg.cell_screen((c - 9, c));
    pg.draw(&mut fb);
    save(&fb, &pg, "13_yard_stamp.png");
    pg.lib = crate::pg::design::defaults();
    pg.screen = Screen::Arena;
    pg.load_test(7);
    pg.running = true;
    for _ in 0..240 {
        pg.tick();
    }
    hover(&mut pg, 0);
    pg.draw(&mut fb);
    save(&fb, &pg, "14_titan_vs_capitals.png");
    println!("exported proving-ground screenshots to {}", dir.display());
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
