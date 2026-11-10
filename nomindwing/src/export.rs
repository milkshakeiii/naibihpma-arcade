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
    use crate::pg::{block, reference_tests};
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
    for t in reference_tests().iter().filter(|t| !t.place.is_empty()) {
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

/// Plays `n` AI-vs-AI matches with the starter deck on seeds 1..=n and prints how each went,
/// with a line every two minutes of the first.
/// A deck for the headless runs: the designs in a save file (as the yard writes them), or the
/// starter deck.
fn deck_from(file: Option<&str>) -> Vec<crate::pg::design::Design> {
    use crate::pg::design::{starter_deck, Design};
    let Some(f) = file else { return starter_deck() };
    let text = std::fs::read_to_string(f).unwrap_or_else(|e| panic!("can't read {f}: {e}"));
    let deck: Vec<Design> = text.lines().skip(1).filter_map(Design::from_line).collect();
    assert!(!deck.is_empty(), "{f} holds no designs");
    deck
}

/// Prints what each card of a deck is: its size, cost, power, speed, guns and build order.
pub fn deck_sheet(file: Option<&str>) {
    let deck = deck_from(file);
    for (i, d) in deck.iter().enumerate() {
        let st = d.stats();
        let c = d.check();
        let kind = if d.seeded { "SEEDED" } else { "BUILT" };
        println!(
            "{} {:9} {:6} {:7} {:4} px, cost {:4}, mass {:4}, power {}/{} ({}%), thrust {} speed {} px/s turn {}, flak {} lance {} cannon {} scoops {} cores {}, factory {} (builds to {} px){}",
            i + 1,
            d.name,
            kind,
            d.order.name(),
            st.count,
            st.cost,
            st.mass,
            st.made,
            st.used,
            st.power_pct(),
            st.thrust,
            st.top_speed_px_s(),
            st.turn,
            st.flak,
            st.lance,
            st.cannon,
            st.scoops,
            st.cores,
            st.factory,
            st.max_build(),
            if c.ok() { "" } else { "  ** NOT LEGAL **" }
        );
        if !d.build.is_empty() {
            let names: Vec<String> = d
                .build
                .iter()
                .map(|&b| match deck.get(b) {
                    Some(o) if o.seeded => format!("{} (SEEDED: SKIPPED)", o.name),
                    Some(o) if o.stats().count > st.max_build() => format!("{} (TOO BIG)", o.name),
                    Some(o) => o.name.clone(),
                    None => "?".into(),
                })
                .collect();
            let round: i32 = d.build.iter().filter_map(|&b| deck.get(b)).filter(|o| !o.seeded && o.stats().count <= st.max_build()).map(|o| o.stats().cost).sum();
            println!("    builds {}  (one round {} px, {} s at full size)", names.join(", "), round, round * crate::pg::design::PRINT_TICKS / 15 / st.factory.max(1));
        }
    }
}

/// What one headless match came to: who won, how long it took, and when each side first had
/// each card finished (a factory fully grown, a ship printed or arrived).
pub struct Played {
    pub winner: Option<usize>,
    pub ticks: u32,
    pub first: [Vec<Option<u32>>; 2],
    /// Ships a side (miners included, as on the match screen) at 2:00 and 4:00.
    pub at2: [usize; 2],
    pub at4: [usize; 2],
    /// When each side planted each of its factories.
    pub planted: [Vec<u32>; 2],
    pub arena: crate::pg::sim::Arena,
}

/// Plays one match with nobody watching.
pub fn play_match(seed: u64, decks: [Vec<crate::pg::design::Design>; 2], bots: [crate::pg::sim::Bot; 2], mut each_tick: impl FnMut(&crate::pg::sim::Arena)) -> Played {
    use crate::pg::sim::{Arena, Role};
    let n = [decks[0].len(), decks[1].len()];
    let mut a = Arena::for_bots(seed, decks, bots);
    let mut first = [vec![None; n[0]], vec![None; n[1]]];
    let (mut at2, mut at4) = ([0; 2], [0; 2]);
    let mut planted = [Vec::new(), Vec::new()];
    let ships = |a: &Arena, t: usize| a.ships.iter().filter(|s| s.alive && s.team == t && s.role == Role::Ship && s.grow.is_none() && !s.node).count();
    while a.over.is_none() {
        a.step();
        each_tick(&a);
        for t in 0..2 {
            while planted[t].len() < a.eco.as_ref().unwrap().tally[t].planted as usize {
                planted[t].push(a.tick);
            }
        }
        if a.tick % 15 == 0 {
            for s in a.ships.iter().filter(|s| s.alive && s.role == Role::Ship && s.grow.is_none() && s.warp == 0) {
                if let Some(f) = first[s.team].get_mut(s.slot) {
                    f.get_or_insert(a.tick);
                }
            }
        }
        if a.tick == 15 * 120 {
            at2 = [ships(&a, 0), ships(&a, 1)];
        }
        if a.tick == 15 * 240 {
            at4 = [ships(&a, 0), ships(&a, 1)];
        }
    }
    Played { winner: a.over.unwrap(), ticks: a.tick, first, at2, at4, planted, arena: a }
}

/// Bot against bot, seeds 1 to `n`, each side with the deck in its file (or the starter deck).
pub fn match_sim(n: u64, blue: Option<&str>, red: Option<&str>, bots: [crate::pg::sim::Bot; 2]) {
    use crate::pg::play::{clock, state_line};
    let (mut wins, mut ticks) = ([0u32; 3], Vec::new());
    println!("BLUE {:?} against RED {:?}", bots[0], bots[1]);
    for seed in 1..=n {
        let t0 = std::time::Instant::now();
        let mut next = 15 * 120;
        let m = play_match(seed, [deck_from(blue), deck_from(red)], bots, |a| {
            if seed == 1 && a.tick >= next && a.over.is_none() {
                next += 15 * 120;
                println!("    {} {}", clock(a.tick), state_line(a));
            }
        });
        let a = &m.arena;
        let who = match m.winner {
            Some(0) => "BLUE",
            Some(_) => "RED",
            None => "DRAW",
        };
        wins[m.winner.unwrap_or(2)] += 1;
        ticks.push(m.ticks);
        println!("seed {seed}: {who} at {} ({} ms) | {}", clock(m.ticks), t0.elapsed().as_millis(), state_line(a));
        // What each side made of each card (everything that ever flew or stood), and when it
        // first had one finished.
        for t in 0..2 {
            let e = a.eco.as_ref().unwrap();
            let deck = &e.deck[t];
            let made: Vec<String> = (0..deck.len())
                .map(|i| (i, a.ships.iter().filter(|s| s.team == t && s.slot == i && s.role == crate::pg::sim::Role::Ship).count()))
                .filter(|&(_, n)| n > 0)
                .map(|(i, n)| format!("{} {}{}", n, deck[i].name, m.first[t][i].map_or(String::new(), |f| format!(" (first {})", clock(f)))))
                .collect();
            println!("      {} made {}", ["BLUE", "RED"][t], made.join(", "));
            let planted: Vec<String> = m.planted[t].iter().map(|&k| clock(k)).collect();
            println!("      {} had {} ships at 2:00, {} at 4:00; changed orders {} times; planted at {}", ["BLUE", "RED"][t], m.at2[t], m.at4[t], e.tally[t].orders, planted.join(" "));
        }
    }
    ticks.sort();
    println!("blue {} red {} draw {} | {} to {}, median {}", wins[0], wins[1], wins[2], clock(ticks[0]), clock(ticks[ticks.len() - 1]), clock(ticks[ticks.len() / 2]));
}

/// The AI against each scripted opening, `n` matches on each side of the map, starter deck
/// against starter deck. One line per opening: how many the AI won, lost and drew, and how long
/// the matches ran.
pub fn judge(n: u64) {
    use crate::pg::design::starter_deck;
    use crate::pg::play::clock;
    use crate::pg::sim::Bot;
    for other in [Bot::Clock, Bot::Planter, Bot::Expander, Bot::Summoner, Bot::Ai] {
        let (mut won, mut lost, mut drew, mut ticks) = (0, 0, 0, Vec::new());
        let mut printed = [0i32; 2];
        for seed in 1..=n {
            for side in 0..2 {
                let bots = if side == 0 { [Bot::Ai, other] } else { [other, Bot::Ai] };
                let m = play_match(seed, [starter_deck(), starter_deck()], bots, |_| {});
                match m.winner {
                    Some(w) if w == side => won += 1,
                    Some(_) => lost += 1,
                    None => drew += 1,
                }
                ticks.push(m.ticks);
                let e = m.arena.eco.as_ref().unwrap();
                printed[0] += e.tally[side].printed + e.tally[side].summoned;
                printed[1] += e.tally[1 - side].printed + e.tally[1 - side].summoned;
            }
        }
        ticks.sort();
        let games = (2 * n) as i32;
        println!(
            "AI v {:9} won {won:2} lost {lost:2} drew {drew:2} | {} to {}, median {} | ships made a match: AI {}, {:?} {}",
            format!("{other:?}").to_uppercase(),
            clock(ticks[0]),
            clock(ticks[ticks.len() - 1]),
            clock(ticks[ticks.len() / 2]),
            printed[0] / games,
            other,
            printed[1] / games
        );
    }
}

/// Prints every ship's state every few ticks of one matchup: `blue` and `red` are (slot, count).
/// A blue slot of 99 plays ready-made test number `red.0` instead.
pub fn pg_trace(blue: (usize, i32), red: (usize, i32), every: u32, until: u32) {
    use crate::pg::design::defaults;
    use crate::pg::sim::Arena;
    use crate::pg::{block, reference_tests};
    let lib = defaults();
    let mut a = Arena::new();
    let place = if blue.0 == 99 {
        reference_tests()[red.0].place.clone()
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

    let mut pg = Pg::reference();
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
    // Round 5: a cannon on a bare core, short of power, with the power gauge's hover.
    pg.lib[5] = crate::pg::design::Design::from_rows("CANNONBOAT", crate::pg::design::Order::Hunt, &["nnn", "nnn", "nnn", ".c.", ".e."]);
    pg.select_slot(5);
    pg.ed.stamp = None;
    pg.mouse = (240, 100);
    pg.draw(&mut fb);
    save(&fb, &pg, "15_yard_underpowered.png");
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
    // The match.
    pg.lib = crate::pg::design::starter_deck();
    pg.screen = Screen::Yard;
    pg.select_slot(4);
    pg.mouse = (262, 189);
    pg.draw(&mut fb);
    save(&fb, &pg, "16_yard_seeded_factory.png");
    pg.menu = true;
    pg.mouse = (200, 177);
    pg.draw(&mut fb);
    save(&fb, &pg, "17_match_menu.png");
    pg.menu = false;
    pg.start_match(true, 3);
    pg.help = false;
    // The king summons a fighter: it arrives by hyperspace.
    pg.play.as_mut().unwrap().arena.command(crate::pg::sim::Cmd::Print { team: 0, slot: 0, target: None });
    for _ in 0..15 * 5 {
        let a = &mut pg.play.as_mut().unwrap().arena;
        if a.ships.iter().any(|s| s.alive && s.team == 0 && s.warp > 0 && s.warp < 16) {
            break;
        }
        a.step();
    }
    pg.mouse = (350, 290);
    pg.draw(&mut fb);
    save(&fb, &pg, "18_match_hyperspace.png");
    pg.play.as_mut().unwrap().arena.eco.as_mut().unwrap().bot[0] = crate::pg::sim::Bot::Ai;
    // Look at something: the camera centred on a point, the pointer on a ship.
    let look = |pg: &mut Pg, (x, y): (i32, i32), hover: Option<(i32, i32)>| {
        let p = pg.play.as_mut().unwrap();
        p.follow = false;
        p.cam = ((x - 152).clamp(0, crate::pg::sim::MW - 304), (y - 150).clamp(0, crate::pg::sim::MH - 300));
        let cam = p.cam;
        pg.mouse = hover.map_or((350, 290), |(hx, hy)| (hx - cam.0, hy - cam.1));
    };
    let step_until = |pg: &mut Pg, max: u32, done: &dyn Fn(&crate::pg::sim::Arena) -> bool| {
        for _ in 0..max {
            let a = &mut pg.play.as_mut().unwrap().arena;
            if done(a) || a.over.is_some() {
                break;
            }
            a.step();
        }
    };
    // A seed's factory growing out of its rock.
    step_until(&mut pg, 15 * 120, &|a| a.ships.iter().any(|s| s.alive && s.team == 0 && s.grow.as_ref().is_some_and(|g| g.eats) && s.px_now() * 2 > s.px_full()));
    if let Some(s) = pg.play.as_ref().unwrap().arena.ships.iter().find(|s| s.alive && s.team == 0 && s.grow.as_ref().is_some_and(|g| g.eats)) {
        let (x, y) = (s.x, s.y);
        look(&mut pg, (x, y), Some((x, y)));
    }
    pg.draw(&mut fb);
    save(&fb, &pg, "19_match_factory_growing.png");
    // A factory printing a ship beside it.
    step_until(&mut pg, 15 * 120, &|a| a.ships.iter().any(|s| s.alive && s.team == 0 && s.maker.is_some() && s.px_now() * 3 > s.px_full()));
    if let Some(s) = pg.play.as_ref().unwrap().arena.ships.iter().find(|s| s.alive && s.team == 0 && s.maker.is_some()) {
        let m = &pg.play.as_ref().unwrap().arena.ships[s.maker.unwrap()];
        let (x, y, hx, hy) = (m.x, m.y, m.x, m.y);
        look(&mut pg, (x, y), Some((hx, hy)));
    }
    pg.draw(&mut fb);
    save(&fb, &pg, "20_match_factory_printing.png");
    // Planting: a seed card picked, the pointer on a rock.
    {
        let p = pg.play.as_mut().unwrap();
        let k = p.arena.eco.as_ref().unwrap().king[0];
        let (kx, ky) = (p.arena.ships[k].x, p.arena.ships[k].y);
        let rock = p.arena.eco.as_ref().unwrap().rocks.iter().filter(|r| p.arena.rock_left(r) > 300).min_by_key(|r| crate::geom::dist2(r.cx, r.cy, kx, ky + 200)).map(|r| (r.cx, r.cy));
        p.planting = Some(5);
        if let Some(c) = rock {
            look(&mut pg, c, Some(c));
        }
    }
    pg.draw(&mut fb);
    save(&fb, &pg, "22_match_planting.png");
    // Planting on a rock that already has a factory on it: the bar says they'll wall each other in.
    {
        let p = pg.play.as_mut().unwrap();
        let spot = p.arena.ships.iter().find(|s| s.alive && s.team == 0 && s.rooted).and_then(|s| p.arena.rock_near((s.x, s.y), 30).map(|r| ((s.x, s.y), r)));
        p.planting = Some(4);
        if let Some((c, r)) = spot {
            look(&mut pg, c, Some(r));
        }
    }
    pg.draw(&mut fb);
    save(&fb, &pg, "24_match_planting_taken.png");
    pg.play.as_mut().unwrap().planting = None;
    // The match's rules panel.
    pg.help = true;
    pg.draw(&mut fb);
    save(&fb, &pg, "25_match_rules.png");
    pg.help = false;
    // The thick of it: the camera on a fight.
    step_until(&mut pg, 15 * 300, &|a| a.shots.len() > 40);
    if let Some(sh) = pg.play.as_ref().unwrap().arena.shots.get(20) {
        let (x, y) = (sh.line.x, sh.line.y);
        look(&mut pg, (x, y), None);
    }
    pg.draw(&mut fb);
    save(&fb, &pg, "21_match_battle.png");
    // Round 2. The whole map in the field, the pointer on one of blue's factories.
    {
        let p = pg.play.as_mut().unwrap();
        p.tactical = true;
        let spot = p.arena.ships.iter().filter(|s| s.alive && s.team == 0 && s.rooted).map(|s| (s.x, s.y)).last();
        pg.mouse = spot.map_or((150, 150), |(x, y)| (2 + x / 4, 41 + y / 4));
    }
    pg.draw(&mut fb);
    save(&fb, &pg, "31_match_whole_map.png");
    pg.play.as_mut().unwrap().tactical = false;
    step_until(&mut pg, 15 * 60 * 30, &|_| false);
    pg.mouse = (0, 0);
    pg.draw(&mut fb);
    save(&fb, &pg, "23_match_over.png");
    // Round 2. Orders: the pointer on the frigate's tag, a fighter on repeat in the king's queue.
    pg.start_match(true, 5);
    pg.help = false;
    {
        use crate::pg::sim::Cmd;
        let a = &mut pg.play.as_mut().unwrap().arena;
        let k = a.eco.as_ref().unwrap().king[0];
        // Pixels to print with, on the books as mined.
        a.ships[k].store += 400;
        a.score[0].mined += 400;
        a.command(Cmd::Repeat { team: 0, slot: 0 });
        a.command(Cmd::Order { team: 0, slot: 0, order: crate::pg::design::Order::Escort });
        a.command(Cmd::Order { team: 0, slot: 2, order: crate::pg::design::Order::Hunt });
        for _ in 0..15 * 75 {
            a.step();
        }
    }
    pg.mouse = (383, 62);
    pg.draw(&mut fb);
    save(&fb, &pg, "30_match_orders.png");
    // The king flies off with its escort; the view was left behind on purpose, then catches up.
    {
        use crate::pg::sim::Cmd;
        let p = pg.play.as_mut().unwrap();
        p.arena.command(Cmd::Steer { team: 0, dx: 1, dy: -1 });
        for _ in 0..15 * 4 {
            p.arena.step();
        }
        let k = &p.arena.ships[p.arena.eco.as_ref().unwrap().king[0]];
        p.cam = ((k.x - 152).clamp(0, crate::pg::sim::MW - 304), (k.y - 150).clamp(0, crate::pg::sim::MH - 300));
        let cam = p.cam;
        let escort = p.arena.ships.iter().find(|s| s.alive && s.team == 0 && s.name == "FIGHTER" && s.warp == 0).map(|s| (s.x - cam.0, s.y - cam.1));
        pg.mouse = escort.unwrap_or((350, 290));
    }
    pg.draw(&mut fb);
    save(&fb, &pg, "32_match_escort.png");
    {
        // Looking away: the view held on the home rocks while the king flies on.
        let p = pg.play.as_mut().unwrap();
        p.follow = false;
        pg.mouse = (350, 230);
    }
    pg.draw(&mut fb);
    save(&fb, &pg, "33_match_looking_away.png");
    // Miners at work: scoops filling, and a spark where one has just taken a bite.
    pg.start_match(true, 7);
    pg.help = false;
    pg.play.as_mut().unwrap().arena.eco.as_mut().unwrap().bot[0] = crate::pg::sim::Bot::Ai;
    {
        let p = pg.play.as_mut().unwrap();
        let loaded = |a: &crate::pg::sim::Arena| a.ships.iter().filter(|s| s.alive && s.team == 0 && s.carry >= 4).count();
        while p.arena.tick < 15 * 240 && !(loaded(&p.arena) >= 3 && p.arena.sparks.iter().any(|sp| !sp.blast && sp.t >= 3)) {
            p.arena.step();
        }
        if let Some(s) = p.arena.ships.iter().filter(|s| s.alive && s.team == 0 && s.carry >= 4).max_by_key(|s| s.carry) {
            p.follow = false;
            p.cam = ((s.x - 152).clamp(0, crate::pg::sim::MW - 304), (s.y - 150).clamp(0, crate::pg::sim::MH - 300));
            pg.mouse = (s.x - p.cam.0, s.y - p.cam.1);
        }
    }
    pg.draw(&mut fb);
    save(&fb, &pg, "34_match_miners_at_work.png");
    // The game as it starts: the starter deck as your seven designs, and the fights made for it.
    let mut pg = Pg::new(None, None);
    pg.help = false;
    pg.mouse = (150, 240);
    pg.draw(&mut fb);
    save(&fb, &pg, "26_start_arena.png");
    for (t, ticks, name) in [(6, 150, "27_lancers_raid_outpost.png"), (8, 420, "28_capital_at_shipyard.png")] {
        pg.load_test(t);
        pg.running = true;
        for _ in 0..ticks {
            pg.tick();
        }
        hover(&mut pg, 1);
        pg.draw(&mut fb);
        save(&fb, &pg, name);
    }
    pg.running = false;
    pg.screen = Screen::Yard;
    pg.select_slot(5);
    pg.mouse = (262, 189);
    pg.draw(&mut fb);
    save(&fb, &pg, "29_start_yard_shipyard.png");
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
