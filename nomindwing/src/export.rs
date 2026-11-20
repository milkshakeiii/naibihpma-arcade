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

/// The arena's reference fights: every ready-made test, then every pair of warships at equal
/// cost, with who won, how fast, and what each side lost. They're fought with the starter deck
/// (Henry's drawings, with no hull that feeds nothing) and the fights made for it, so what's
/// measured is the rules and not a wasteful drawing. `old` fights them with the proving ground's
/// own fleet instead (`defaults()`), as its five rounds were judged: those drawings carry spare
/// hull, and are kept so the rounds' numbers can be had again.
pub fn pg_sim(which: &str) {
    use crate::pg::design::{defaults, starter_deck};
    use crate::pg::sim::Arena;
    use crate::pg::{block, part_tests, parts_library, reference_tests, tests};
    let (old, parts) = (which == "old", which == "parts" || which == "teams");
    let (lib, fights) = match which {
        "old" => (defaults(), reference_tests()),
        "parts" => (parts_library(), part_tests()),
        // The ready-made teams' fights, each with its own ships (`--team-fights`).
        "teams" => (Vec::new(), crate::pg::teams::team_fights()),
        // Your own team as the window has it, read from its save (nothing is written): the
        // fights the window shows, with the orders and styles your cards carry.
        "mine" => (crate::pg::Pg::new(Some(std::path::PathBuf::from("proving_ground_designs.txt")), None).lib, tests()),
        _ => (starter_deck(), tests()),
    };
    if which == "mine" {
        println!("your team, from proving_ground_designs.txt:");
        for (i, d) in lib.iter().enumerate().filter(|(_, d)| !d.is_empty()) {
            println!("  {} {}: {} px, {}", i + 1, d.name, d.stats().cost, d.does().to_lowercase());
        }
    }
    // One replay can be luck, so every matchup runs on SEEDS rolls of the dice. Seed 1 is the
    // fight the window shows.
    const SEEDS: u64 = 12;
    let report = |name: &str, lib: [&[crate::pg::design::Design]; 2], place: &[(usize, usize, i32, i32)]| {
        let (mut wins, mut secs, mut lost, mut px, mut window) = ([0; 3], 0, [0; 2], [0; 2], String::new());
        let (mut placed, mut stale, mut splits, mut stopped, mut downed) = ([0; 2], 0, [0; 2], [0; 2], [0; 2]);
        // Shots fired and shots that hit something, each side, over all the seeds.
        let (mut fired, mut hit) = ([0; 2], [0; 2]);
        let n = SEEDS as i32;
        for seed in 1..=SEEDS {
            let mut a = Arena::seeded(seed);
            for &(team, slot, x, y) in place {
                a.add(team, slot, &lib[team][slot], x, y);
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
                stopped[t] += a.score[t].shielded;
                downed[t] += a.score[t].downed;
                fired[t] += a.score[t].fired;
                hit[t] += a.score[t].hits;
            }
            if seed == 1 {
                let t = a.tick / 15;
                window = format!("{} {}:{:02}", ["BLUE", "RED", "DRAW"][k], t / 60, t % 60);
            }
        }
        let t = secs / SEEDS as u32;
        // Shots of each side's that the other's shields stopped, a fight: only said where there
        // were any.
        let pct = |t: usize| if fired[t] > 0 { format!("{}%", hit[t] * 100 / fired[t]) } else { "-".to_string() };
        let aim = format!("  shots that hit: blue {} of {} ({})  red {} of {} ({})", hit[0], fired[0], pct(0), hit[1], fired[1], pct(1));
        let shields = if stopped[0] + stopped[1] > 0 { format!("  stopped at shields: blue's {} red's {}", stopped[0] / n, stopped[1] / n) } else { String::new() };
        // Missiles each side shot or cut down, in all the seeds together: only said where there were any.
        let shields = if downed[0] + downed[1] > 0 { format!("{shields}  missiles downed: by blue {} by red {}", downed[0], downed[1]) } else { shields };
        println!(
            "{name:34} blue {:2}  red {:2}  draw {:2} ({:2} stalemate)  avg {}:{:02}  lost: blue {}/{} ({} px)  red {}/{} ({} px)  splits {}/{}  window: {window}{aim}{shields}",
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
    for t in fights.iter().filter(|t| !t.place.is_empty()) {
        // A team fight's sides each have their own team's slots.
        let sides = t.ships.as_ref().map_or([&lib[..], &lib[..]], |s| [&s.teams[0][..], &s.teams[1][..]]);
        report(t.name, sides, &t.place);
    }
    // Every pair of warships at (roughly) the same budget: the swarm question. Cost is pixels plus
    // extra cores. The old fleet's five at 600 px; the starter deck's four at what its dearest
    // costs.
    if parts {
        return;
    }
    let warships: Vec<usize> = if old { (0..5).collect() } else { (0..lib.len()).filter(|&s| crate::pg::sim::Card::of(&lib[s]).warship()).collect() };
    let budget = if old { 600 } else { warships.iter().map(|&s| lib[s].stats().cost).max().unwrap_or(600) };
    println!("--- equal cost, about {budget} px a side ---");
    let gap = |s: usize| {
        let sh = lib[s].shape();
        sh.w.max(sh.h) + 4
    };
    let count = |s: usize| (budget / lib[s].stats().cost).max(1);
    for (k, &a) in warships.iter().enumerate() {
        for &b in &warships[k + 1..] {
            let mut place = block(0, a, count(a), 130, 180, gap(a));
            place.extend(block(1, b, count(b), 350, 180, gap(b)));
            let name = format!("{} {} VS {} {}", count(a), lib[a].name, count(b), lib[b].name);
            report(&name, [&lib, &lib], &place);
        }
    }
    if !old {
        return;
    }
    // The titan against the bigger designs at its own cost (small ships would need hundreds).
    let titan = lib[6].stats().cost;
    println!("--- equal cost, the titan's {titan} px a side ---");
    for b in 2..5 {
        let n = (titan / lib[b].stats().cost).max(1);
        let mut place = block(0, 6, 1, 110, 180, 0);
        place.extend(block(1, b, n, 360, 180, gap(b)));
        report(&format!("1 TITAN VS {n} {}", lib[b].name), [&lib, &lib], &place);
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
            "{} {:9} {:6} {:7} {:4} px, cost {:4}, mass {:4}, power {}/{} ({}%), thrust {} speed {} px/s turn {} (nose-first: push {}/{}/{}/{} ahead/right/astern/left, quarter turn {} ticks), flak {} lance {} cannon {} beam {} (longest {}, {} px) missile {} repeater {} (longest {}, {} px) shields {} projector {} ({} px) scoops {} cores {}, factory {} (builds to {} px){}",
            i + 1,
            d.name,
            kind,
            d.does(),
            st.count,
            st.cost,
            st.mass,
            st.made,
            st.used,
            st.power_pct(),
            st.thrust,
            st.top_speed_px_s(),
            st.turn,
            st.push[0],
            st.push[1],
            st.push[2],
            st.push[3],
            st.quarter,
            st.flak,
            st.lance,
            st.cannon,
            st.beam,
            st.beam_len,
            st.beam_reach(),
            st.missile,
            st.repeater,
            st.rep_len,
            crate::pg::design::repeat_reach(st.rep_len),
            st.shields,
            st.projector,
            crate::pg::design::proj_reach(st.projector),
            st.scoops,
            st.cores,
            st.factory,
            st.max_build(),
            c.problem().map_or(String::new(), |p| format!("  ** NOT LEGAL: {p} **"))
        );
        if !d.first.is_empty() {
            println!("    builds first, once: {}", d.first.iter().map(|&b| deck.get(b).map_or("?".to_string(), |o| o.name.clone())).collect::<Vec<_>>().join(", "));
        }
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
    /// The tick each side's king was last slowed and stayed slowed to the end (None: it wasn't).
    pub beaten: [Option<u32>; 2],
    pub arena: crate::pg::sim::Arena,
}

/// The layout the tools' matches are played on, if `--map` named one: otherwise each match's
/// is picked by its seed, as the game's are.
pub static MAP: std::sync::OnceLock<crate::pg::sim::Layout> = std::sync::OnceLock::new();

fn map_for(seed: u64) -> crate::pg::sim::Layout {
    MAP.get().copied().unwrap_or_else(|| crate::pg::sim::Layout::pick(seed))
}

/// Plays one match with nobody watching.
pub fn play_match(seed: u64, decks: [Vec<crate::pg::design::Design>; 2], bots: [crate::pg::sim::Bot; 2], each_tick: impl FnMut(&crate::pg::sim::Arena)) -> Played {
    play_match_with(seed, decks, bots, [false; 2], each_tick)
}

/// The same, saying which sides' AIs send their warships by playing the match forward (`sends`).
pub fn play_match_with(seed: u64, decks: [Vec<crate::pg::design::Design>; 2], bots: [crate::pg::sim::Bot; 2], sends: [bool; 2], mut each_tick: impl FnMut(&crate::pg::sim::Arena)) -> Played {
    use crate::pg::sim::{Arena, Role};
    let n = [decks[0].len(), decks[1].len()];
    let mut a = Arena::for_map(seed, map_for(seed), decks, bots);
    for t in 0..2 {
        if sends[t] {
            a.send_by_looking(t);
        }
    }
    let mut first = [vec![None; n[0]], vec![None; n[1]]];
    let (mut at2, mut at4) = ([0; 2], [0; 2]);
    let mut planted = [Vec::new(), Vec::new()];
    let mut beaten = [None; 2];
    let ships = |a: &Arena, t: usize| a.ships.iter().filter(|s| s.alive && s.team == t && s.role == Role::Ship && s.grow.is_none() && !s.node).count();
    while a.over.is_none() {
        a.step();
        each_tick(&a);
        for t in 0..2 {
            while planted[t].len() < a.eco.as_ref().unwrap().tally[t].planted as usize {
                planted[t].push(a.tick);
            }
            if a.over.is_none() {
                beaten[t] = if a.king_slowed(t) { beaten[t].or(Some(a.tick)) } else { None };
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
    Played { winner: a.over.unwrap(), ticks: a.tick, first, at2, at4, planted, beaten, arena: a }
}

/// Bot against bot, seeds 1 to `n`, each side with the deck in its file (or the starter deck).
pub fn match_sim(n: u64, blue: Option<&str>, red: Option<&str>, bots: [crate::pg::sim::Bot; 2], sends: [bool; 2]) {
    use crate::pg::play::{clock, state_line};
    let (mut wins, mut ticks) = ([0u32; 3], Vec::new());
    let how = |t: usize| if sends[t] { " (sending by playing the match forward)" } else { "" };
    println!("BLUE {:?}{} against RED {:?}{}", bots[0], how(0), bots[1], how(1));
    for seed in 1..=n {
        let t0 = std::time::Instant::now();
        let mut next = 15 * 120;
        let m = play_match_with(seed, [deck_from(blue), deck_from(red)], bots, sends, |a| {
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
        println!("seed {seed}, rocks {}: {who} at {} ({} ms) | {}", map_for(seed).name(), clock(m.ticks), t0.elapsed().as_millis(), state_line(a));
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
            if let Some(d) = &e.dispatch[t] {
                println!("      {} thought {} times, changed its plan {} times, and played the match forward {} times", ["BLUE", "RED"][t], d.thoughts, d.switches, d.looked);
            }
        }
        // The loser's last stretch with next to nothing standing: how long its king lasted.
        if let Some(l) = m.winner.map(|w| 1 - w) {
            match m.beaten[l] {
                Some(at) => println!("      {} had under {} px standing from {}: its king lasted {} s more", ["BLUE", "RED"][l], crate::pg::sim::KING_NEEDS, clock(at), (m.ticks - at) / 15),
                None => println!("      {} lost its king with {} px still standing", ["BLUE", "RED"][l], a.standing(l)),
            }
        }
    }
    ticks.sort();
    println!("blue {} red {} draw {} | {} to {}, median {}", wins[0], wins[1], wins[2], clock(ticks[0]), clock(ticks[ticks.len() - 1]), clock(ticks[ticks.len() / 2]));
}

/// Every ready-made team against every other, the AI playing both by their strategies: `seeds`
/// seeds a pairing, with each team on each side of the map. The matches run side by side on every
/// core (each is deterministic, so that changes nothing in them). Prints who beat whom, and for
/// each team how its matches went: how long, when its first wave went, what it planted, how big
/// its side got and how big the fights got. `only` names teams to play (commas between); the
/// rest are left out. With `minutes`, each match also has a line a minute: each side's
/// factories, miners, warship pixels, king pixels, bank and pixels mined.
/// One match between two ready-made teams, every five seconds from `from` to `to` seconds: for
/// each side, its warships (how many, their pixels, where their middle is), its king (where, how
/// whole, hunting or not), its factories, its miners, and the enemy warships within 300 px of its
/// king. `old` names a team that plays with every rule of 2026-11-15 off.
pub fn match_trace(blue: &str, red: &str, seed: u64, from: u32, to: u32, off: u32, old: Option<&str>) {
    use crate::pg::play::clock;
    use crate::pg::sim::{Arena, Role, Ship};
    use crate::pg::teams::ready_teams;
    let teams = ready_teams();
    let find = |n: &str| teams.iter().position(|t| t.name.eq_ignore_ascii_case(n)).unwrap_or_else(|| panic!("no team called {n}"));
    let (i, j) = (find(blue), find(red));
    let mut a = Arena::for_map(seed, map_for(seed), [teams[i].designs.clone(), teams[j].designs.clone()], [crate::pg::sim::Bot::Ai; 2]);
    let was = |n: usize| old.is_some_and(|o| o.split(',').any(|w| w.trim().eq_ignore_ascii_case(teams[n].name)));
    {
        let e = a.eco.as_mut().unwrap();
        e.strategy = [teams[i].strategy, teams[j].strategy];
        e.off = [i, j].map(|n| if was(n) { u32::MAX } else { off });
    }
    println!("{} (blue) v {} (red), seed {seed}", teams[i].name, teams[j].name);
    while a.over.is_none() && a.tick <= to * 15 {
        a.step();
        if a.tick % 75 != 0 || a.tick < from * 15 {
            continue;
        }
        let e = a.eco.as_ref().unwrap();
        let mut line = clock(a.tick);
        for t in 0..2 {
            let war: Vec<&Ship> = a.ships.iter().filter(|s| s.alive && s.team == t && s.role == Role::Ship && !s.rooted && s.grow.is_none() && s.st.armed() && !s.st.gathers()).collect();
            let px: i32 = war.iter().map(|s| s.px_now()).sum();
            let n = war.len().max(1) as i32;
            let (cx, cy) = (war.iter().map(|s| s.x).sum::<i32>() / n, war.iter().map(|s| s.y).sum::<i32>() / n);
            let k = &a.ships[e.king[t]];
            let near = a.ships.iter().filter(|s| s.alive && s.team != t && s.role == Role::Ship && !s.rooted && s.st.armed() && !s.st.gathers() && crate::geom::cheb(s.x, s.y, k.x, k.y) <= 300).map(|s| s.px_now()).sum::<i32>();
            let fact: Vec<String> = a.ships.iter().filter(|s| s.alive && s.team == t && s.rooted && s.st.factory > 0).map(|s| format!("{}", s.px_now())).collect();
            let miners = a.ships.iter().filter(|s| s.alive && s.team == t && s.role == Role::Ship && s.st.gathers()).count();
            line += &format!(" | {} war {:2} ships {:3} px at {:4},{:4} | king {:3} px at {:4},{:4}{} | fact [{}] miners {:2} bank {:3} | enemy near king {:3}", ["B", "R"][t], war.len(), px, cx, cy, k.px_now() * k.alive as i32, k.x, k.y, if k.sent.is_some() { " SENT" } else { "" }, fact.join(" "), miners, a.bank(e.king[t]), near);
        }
        println!("{line}");
    }
    println!("over: {:?} at {}", a.over, clock(a.tick));
}

pub fn teams(seeds: u64, only: Option<&str>, minutes: bool, off: u32, old: Option<&str>) {
    use crate::pg::play::clock;
    use crate::pg::sim::{Arena, Role};
    use crate::pg::teams::ready_teams;
    use std::sync::atomic::{AtomicUsize, Ordering};
    let all = ready_teams();
    let picked: Vec<usize> = (0..all.len()).filter(|&i| only.map_or(true, |o| o.split(',').any(|n| n.trim().eq_ignore_ascii_case(all[i].name)))).collect();
    assert!(picked.len() >= 2, "pick two teams or more");
    // One match: blue team, red team, seed.
    let jobs: Vec<(usize, usize, u64)> = picked.iter().flat_map(|&i| picked.iter().filter(move |&&j| j != i).flat_map(move |&j| (1..=seeds).map(move |s| (i, j, s)))).collect();
    struct Row {
        teams: [usize; 2],
        winner: Option<usize>,
        ticks: u32,
        wave: [u32; 2],
        planted: [i32; 2],
        /// The most ships a side had at once, its king and factories left out.
        ships: [usize; 2],
        /// The most ships in a fight at once, both sides.
        fight: usize,
        /// When ships of the two sides first fought.
        contact: u32,
        seed: u64,
        /// Pixels of warships a side had at 5:00 and at 10:00 (0 if the match was over), and the
        /// most it had at once.
        army: [[i32; 3]; 2],
        /// When a side first lost a factory (0: never).
        lost: [u32; 2],
        /// A line a minute, with `minutes`.
        lines: Vec<String>,
    }
    let next = AtomicUsize::new(0);
    let rows = std::sync::Mutex::new(Vec::new());
    let started = std::time::Instant::now();
    let cores = std::thread::available_parallelism().map_or(4, |n| n.get());
    std::thread::scope(|scope| {
        for _ in 0..cores {
            scope.spawn(|| loop {
                let k = next.fetch_add(1, Ordering::Relaxed);
                let Some(&(i, j, seed)) = jobs.get(k) else { break };
                let teams = ready_teams();
                let mut a = Arena::for_map(seed, map_for(seed), [teams[i].designs.clone(), teams[j].designs.clone()], [crate::pg::sim::Bot::Ai; 2]);
                a.eco.as_mut().unwrap().strategy = [teams[i].strategy, teams[j].strategy];
                // A team named in `old` plays with every rule of 2026-11-15 off: as it played before.
                let was = |n: usize| old.is_some_and(|o| o.split(',').any(|w| w.trim().eq_ignore_ascii_case(teams[n].name)));
                a.eco.as_mut().unwrap().off = [i, j].map(|n| if was(n) { u32::MAX } else { off });
                let (mut ships, mut fight, mut contact) = ([0usize; 2], 0usize, 0u32);
                let (mut army, mut lost) = ([[0i32; 3]; 2], [0u32; 2]);
                let factories = |a: &Arena, t: usize| a.ships.iter().filter(|s| s.alive && s.team == t && s.rooted && s.st.factory > 0).count();
                let mut had = [0usize; 2];
                let mut lines = Vec::new();
                while a.over.is_none() {
                    a.step();
                    if minutes && a.tick % (15 * 60) == 0 {
                        let e = a.eco.as_ref().unwrap();
                        let side = |t: usize| {
                            let mine = |s: &&crate::pg::sim::Ship| s.alive && s.team == t;
                            let miners = a.ships.iter().filter(|s| mine(s) && s.role == Role::Ship && s.grow.is_none() && s.st.gathers()).count();
                            let war: i32 = a.ships.iter().filter(|s| mine(s) && s.role == Role::Ship && !s.rooted && s.grow.is_none() && s.st.armed() && !s.st.gathers()).map(|s| s.px_now()).sum();
                            format!("{} fact {:2} miners {:3} war px {:4} king {:3} bank {:4} mined {:5} rock {:5}", ["BLUE", "RED"][t], factories(&a, t), miners, war, a.ships[e.king[t]].px_now() * a.ships[e.king[t]].alive as i32, a.bank(e.king[t]), a.score[t].mined, a.rock_in_reach(t))
                        };
                        lines.push(format!("{} | {} | {}", clock(a.tick), side(0), side(1)));
                    }
                    if a.tick % 15 == 0 {
                        for t in 0..2 {
                            let px: i32 = a.ships.iter().filter(|s| s.alive && s.team == t && s.role == Role::Ship && !s.rooted && s.grow.is_none() && s.st.armed() && !s.st.gathers()).map(|s| s.px_now()).sum();
                            army[t][2] = army[t][2].max(px);
                            if a.tick == 15 * 300 {
                                army[t][0] = px;
                            }
                            if a.tick == 15 * 600 {
                                army[t][1] = px;
                            }
                            let f = factories(&a, t);
                            if f < had[t] && lost[t] == 0 {
                                lost[t] = a.tick;
                            }
                            had[t] = f;
                        }
                        for t in 0..2 {
                            let n = a.ships.iter().filter(|s| s.alive && s.team == t && s.role == Role::Ship && !s.rooted && s.grow.is_none()).count();
                            ships[t] = ships[t].max(n);
                        }
                        // Ships with an enemy within 300 px of them.
                        let there: Vec<&crate::pg::sim::Ship> = a.ships.iter().filter(|s| s.alive && s.warp == 0).collect();
                        let n = there.iter().filter(|s| there.iter().any(|o| o.team != s.team && crate::geom::cheb(o.x, o.y, s.x, s.y) <= 300)).count();
                        if n > 0 && contact == 0 {
                            contact = a.tick;
                        }
                        fight = fight.max(n);
                    }
                }
                let e = a.eco.as_ref().unwrap();
                let row = Row { teams: [i, j], winner: a.over.unwrap(), ticks: a.tick, wave: [e.tally[0].first_wave, e.tally[1].first_wave], planted: [e.tally[0].planted, e.tally[1].planted], ships, fight, contact, seed, army, lost, lines };
                rows.lock().unwrap().push(row);
            });
        }
    });
    let rows = rows.into_inner().unwrap();
    println!("{} matches, {} seeds a pairing and each team on each side, in {:.0} s on {cores} cores{}", rows.len(), seeds, started.elapsed().as_secs_f64(), if off > 0 { format!(", AI rules off: {off}") } else { String::new() } + &old.map_or(String::new(), |o| format!(", playing as before: {o}")));
    let med = |mut v: Vec<u32>| -> u32 {
        v.sort();
        if v.is_empty() {
            0
        } else {
            v[v.len() / 2]
        }
    };
    let w = 12;
    // Who beat whom: the row team's wins, losses and draws against the column team.
    println!();
    println!("WINS-LOSSES-DRAWS of the team on the left against the team on top:");
    print!("{:w$}", "");
    for &j in &picked {
        print!(" {:>w$}", all[j].name);
    }
    println!(" {:>w$}", "ALL");
    for &i in &picked {
        print!("{:w$}", all[i].name);
        let mut total = [0; 3];
        for &j in &picked {
            if i == j {
                print!(" {:>w$}", "-");
                continue;
            }
            let mut wld = [0; 3];
            for r in rows.iter().filter(|r| (r.teams == [i, j]) || (r.teams == [j, i])) {
                let me = if r.teams[0] == i { 0 } else { 1 };
                wld[match r.winner {
                    Some(t) if t == me => 0,
                    Some(_) => 1,
                    None => 2,
                }] += 1;
            }
            for k in 0..3 {
                total[k] += wld[k];
            }
            print!(" {:>w$}", format!("{}-{}-{}", wld[0], wld[1], wld[2]));
        }
        println!(" {:>w$}", format!("{}-{}-{}", total[0], total[1], total[2]));
    }
    // How long each pairing's matches ran.
    println!();
    println!("MEDIAN LENGTH of the matches between the two:");
    print!("{:w$}", "");
    for &j in &picked {
        print!(" {:>w$}", all[j].name);
    }
    println!();
    for &i in &picked {
        print!("{:w$}", all[i].name);
        for &j in &picked {
            let v: Vec<u32> = rows.iter().filter(|r| i != j && (r.teams == [i, j] || r.teams == [j, i])).map(|r| r.ticks).collect();
            print!(" {:>w$}", if v.is_empty() { "-".to_string() } else { clock(med(v)) });
        }
        println!();
    }
    // Each team over all its matches.
    println!();
    println!("EACH TEAM over all its matches (medians; a wave that never went counts as never):");
    println!("{:w$} {:>8} {:>8} {:>11} {:>9} {:>9} {:>9} {:>10}", "", "length", "contact", "first wave", "no wave", "planted", "ships", "big fight");
    for &i in &picked {
        let mine: Vec<(&Row, usize)> = rows.iter().filter_map(|r| r.teams.iter().position(|&t| t == i).map(|side| (r, side))).collect();
        let waves: Vec<u32> = mine.iter().map(|(r, s)| r.wave[*s]).filter(|&t| t > 0).collect();
        let never = mine.iter().filter(|(r, s)| r.wave[*s] == 0).count();
        println!(
            "{:w$} {:>8} {:>8} {:>11} {:>9} {:>9} {:>9} {:>10}",
            all[i].name,
            clock(med(mine.iter().map(|(r, _)| r.ticks).collect())),
            clock(med(mine.iter().map(|(r, _)| r.contact).collect())),
            if waves.is_empty() { "-".to_string() } else { clock(med(waves)) },
            format!("{never} of {}", mine.len()),
            med(mine.iter().map(|(r, s)| r.planted[*s].max(0) as u32).collect()),
            med(mine.iter().map(|(r, s)| r.ships[*s] as u32).collect()),
            med(mine.iter().map(|(r, _)| r.fight as u32).collect()),
        );
    }
    // Every match, for looking into one.
    println!();
    println!("EVERY MATCH: blue v red, seed, who won and when; then each side's first wave, factories planted, first factory lost, warship pixels at 5:00 and 10:00 and at most, and most ships:");
    let mut order: Vec<&Row> = rows.iter().collect();
    order.sort_by_key(|r| (r.teams, r.seed));
    let at = |t: u32| if t == 0 { "-".to_string() } else { clock(t) };
    for r in order {
        let who = match r.winner {
            Some(t) => all[r.teams[t]].name,
            None => "DRAW",
        };
        let side = |t: usize| format!("wave {:>5} planted {} lost {:>5} army {}/{}/{} ships {}", at(r.wave[t]), r.planted[t], at(r.lost[t]), r.army[t][0], r.army[t][1], r.army[t][2], r.ships[t]);
        println!("  {} v {} {}: {} at {} | {} | {}", all[r.teams[0]].name, all[r.teams[1]].name, r.seed, who, clock(r.ticks), side(0), side(1));
        for l in &r.lines {
            println!("      {l}");
        }
    }
    let fights: Vec<u32> = rows.iter().map(|r| r.fight as u32).collect();
    let mut sorted = fights.clone();
    sorted.sort();
    println!();
    println!("The most ships in a fight at once, over every match: median {}, 90% {}, most {}", med(fights), sorted[sorted.len() * 9 / 10], sorted[sorted.len() - 1]);
}

/// The AI against each scripted opening (or just those named in `only`, with commas between),
/// `n` matches on each side of the map, both sides with the deck in `file` (or the starter
/// deck). One line per opening: how many the AI won, lost and drew, how long the matches ran,
/// and when the AI was first beaten down to a slowed king.
pub fn judge(n: u64, file: Option<&str>, only: Option<&str>, whatif: bool) {
    use crate::pg::play::clock;
    use crate::pg::sim::Bot;
    println!("{}{}", file.unwrap_or("STARTER DECK"), if whatif { ": THE AI SENDS ITS WARSHIPS BY PLAYING THE MATCH FORWARD (against Ai: today's)" } else { "" });
    for other in [Bot::Clock, Bot::Planter, Bot::Expander, Bot::Summoner, Bot::Boomer, Bot::Rusher, Bot::Saboteur, Bot::Ai] {
        if only.is_some_and(|names| !names.split(',').any(|w| Bot::from_name(w) == Some(other))) {
            continue;
        }
        let (mut won, mut lost, mut drew, mut ticks) = (0, 0, 0, Vec::new());
        let mut printed = [0i32; 2];
        // Seconds each losing king lasted once its side had next to nothing standing.
        let mut lasted: Vec<u32> = Vec::new();
        // When the AI's king was first slowed, in the matches where it ever was.
        let mut down: Vec<u32> = Vec::new();
        // The dispatcher's doings over these matches, and how long they took to play.
        let (mut thoughts, mut switches, mut looked, t0) = (0, 0, 0, std::time::Instant::now());
        for seed in 1..=n {
            for side in 0..2 {
                let bots = if side == 0 { [Bot::Ai, other] } else { [other, Bot::Ai] };
                let mut sends = [false; 2];
                sends[side] = whatif;
                let mut slowed = None;
                let m = play_match_with(seed, [deck_from(file), deck_from(file)], bots, sends, |a| {
                    if slowed.is_none() && a.king_slowed(side) {
                        slowed = Some(a.tick);
                    }
                });
                if let Some(at) = slowed {
                    down.push(at);
                }
                match m.winner {
                    Some(w) if w == side => won += 1,
                    Some(_) => lost += 1,
                    None => drew += 1,
                }
                ticks.push(m.ticks);
                if let Some(at) = m.winner.and_then(|w| m.beaten[1 - w]) {
                    lasted.push((m.ticks - at) / 15);
                }
                let e = m.arena.eco.as_ref().unwrap();
                if let Some(d) = &e.dispatch[side] {
                    (thoughts, switches, looked) = (thoughts + d.thoughts, switches + d.switches, looked + d.looked);
                }
                printed[0] += e.tally[side].printed + e.tally[side].summoned;
                printed[1] += e.tally[1 - side].printed + e.tally[1 - side].summoned;
            }
        }
        ticks.sort();
        let games = (2 * n) as i32;
        println!(
            "AI v {:9} won {won:2} lost {lost:2} drew {drew:2} | {} to {}, median {} | ships made a match: AI {}, {:?} {} | {} | {}",
            format!("{other:?}").to_uppercase(),
            clock(ticks[0]),
            clock(ticks[ticks.len() - 1]),
            clock(ticks[ticks.len() / 2]),
            printed[0] / games,
            other,
            printed[1] / games,
            {
                lasted.sort();
                match lasted.len() {
                    0 => "no king fell beaten".to_string(),
                    k => format!("{k} kings fell beaten, lasting {} to {} s, median {}", lasted[0], lasted[k - 1], lasted[k / 2]),
                }
            },
            {
                down.sort();
                match down.len() {
                    0 => "the AI's king was never slowed".to_string(),
                    k => format!("the AI's king was slowed in {k}, first at {} to {}", clock(down[0]), clock(down[k - 1])),
                }
            }
        );
        if whatif {
            println!("             it thought {thoughts} times, changed its plan {switches} times, played the match forward {looked} times; these {games} matches took {} s to play", t0.elapsed().as_secs());
        }
    }
}

pub fn pace(n: u64) {
    use crate::pg::design::starter_deck;
    use crate::pg::play::clock;
    use crate::pg::sim::Bot;
    const MARKS: [u32; 3] = [3, 5, 7];
    let median = |v: &mut Vec<i32>| {
        v.sort();
        v[v.len() / 2]
    };
    for (name, bots) in [("BOOM v AI", [Bot::Boomer, Bot::Ai]), ("AI v AI", [Bot::Ai, Bot::Ai])] {
        let (mut wins, mut ticks) = ([0; 3], Vec::new());
        // Per side and mark: what each match had mined and planted by then.
        let mut mined = [[Vec::new(), Vec::new(), Vec::new()], [Vec::new(), Vec::new(), Vec::new()]];
        let mut planted = mined.clone();
        let (mut second, mut ships) = ([Vec::new(), Vec::new()], [Vec::new(), Vec::new()]);
        for seed in 1..=n {
            let mut at: [[Option<i32>; 3]; 2] = [[None; 3]; 2];
            let m = play_match(seed, [starter_deck(), starter_deck()], bots, |a| {
                for (k, &min) in MARKS.iter().enumerate() {
                    if a.tick == 15 * 60 * min {
                        for t in 0..2 {
                            at[t][k] = Some(a.score[t].mined);
                        }
                    }
                }
            });
            wins[m.winner.unwrap_or(2)] += 1;
            ticks.push(m.ticks as i32);
            let e = m.arena.eco.as_ref().unwrap();
            for t in 0..2 {
                for (k, &min) in MARKS.iter().enumerate() {
                    mined[t][k].push(at[t][k].unwrap_or(m.arena.score[t].mined));
                    planted[t][k].push(m.planted[t].iter().filter(|&&p| p <= 15 * 60 * min).count() as i32);
                }
                // A side that never planted a second factory counts as the match's length.
                second[t].push(m.planted[t].get(1).map_or(m.ticks, |&p| p) as i32);
                ships[t].push(e.tally[t].printed + e.tally[t].summoned);
            }
        }
        let (lo, hi, mid) = (*ticks.iter().min().unwrap(), *ticks.iter().max().unwrap(), median(&mut ticks));
        println!("{name}: blue {} red {} draw {} | {} to {}, median {}", wins[0], wins[1], wins[2], clock(lo as u32), clock(hi as u32), clock(mid as u32));
        for t in 0..2 {
            let row = |v: &mut [Vec<i32>; 3]| (0..3).map(|k| format!("{}", median(&mut v[k]))).collect::<Vec<_>>().join("/");
            println!(
                "    {} {:?}: factories planted by 3/5/7 min {} | mined {} | second factory {} | ships a match {}",
                ["BLUE", "RED"][t],
                bots[t],
                row(&mut planted[t]),
                row(&mut mined[t]),
                clock(median(&mut second[t]) as u32),
                median(&mut ships[t])
            );
        }
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
    pg.lib[5] = crate::pg::design::Design::from_rows("CANNONBOAT", &["nnn", "nnn", "nnn", ".c.", ".e."]);
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
    // The same seed about to go into open space: Ctrl held, the pointer clear of the rock.
    {
        let p = pg.play.as_ref().unwrap();
        let k = p.arena.eco.as_ref().unwrap().king[0];
        let (kx, ky) = (p.arena.ships[k].x, p.arena.ships[k].y);
        let open = [(110, -30), (0, -110), (80, -80), (-80, -80), (110, 40)].iter().map(|d| (kx + d.0, ky + d.1)).find(|&q| p.arena.sow_links(0, 4, q).is_ok());
        pg.play.as_mut().unwrap().planting = Some(4);
        crate::app::App::ctrl(&mut pg, true);
        if let Some(c) = open {
            look(&mut pg, (kx, ky), Some(c));
        }
    }
    pg.draw(&mut fb);
    save(&fb, &pg, "58_match_seed_into_open_space.png");
    crate::app::App::ctrl(&mut pg, false);
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
        pg.mouse = spot.map_or((150, 150), |(x, y)| (12 + x / 5, 49 + y / 5));
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
        a.command(Cmd::Repeat { team: 0, slot: 0, at: None });
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
    // Looking away: the view held on the home rocks while the king flies on.
    pg.mouse = (350, 230);
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
            p.cam = ((s.x - 152).clamp(0, crate::pg::sim::MW - 304), (s.y - 150).clamp(0, crate::pg::sim::MH - 300));
            pg.mouse = (s.x - p.cam.0, s.y - p.cam.1);
        }
    }
    pg.draw(&mut fb);
    save(&fb, &pg, "34_match_miners_at_work.png");
    // A beaten side: the enemy king slowed, with its engines out, and what hovering it says.
    pg.start_match(true, 3);
    pg.help = false;
    {
        let p = pg.play.as_mut().unwrap();
        let e = p.arena.eco.as_mut().unwrap();
        e.bot = [crate::pg::sim::Bot::Ai, crate::pg::sim::Bot::Clock];
        while p.arena.over.is_none() && !(p.arena.king_slowed(1) && p.arena.tick > 15 * 120) {
            p.arena.step();
        }
        for _ in 0..15 * 3 {
            p.arena.step();
        }
        let k = &p.arena.ships[p.arena.eco.as_ref().unwrap().king[1]];
        p.cam = ((k.x - 152).clamp(0, crate::pg::sim::MW - 304), (k.y - 150).clamp(0, crate::pg::sim::MH - 300));
        pg.mouse = (k.x - p.cam.0, k.y - p.cam.1);
    }
    pg.draw(&mut fb);
    save(&fb, &pg, "35_match_enemy_king_slowed.png");
    // Your own king, slowed: a side that has built nothing by the end of the first minute.
    pg.start_match(true, 3);
    pg.help = false;
    {
        let p = pg.play.as_mut().unwrap();
        p.arena.eco.as_mut().unwrap().bot[1] = crate::pg::sim::Bot::None;
        for _ in 0..15 * 62 {
            p.arena.step();
        }
        pg.mouse = (350, 230);
    }
    pg.draw(&mut fb);
    save(&fb, &pg, "36_match_your_king_slowed.png");
    // Guards at their posts: the rings round an outpost in a quiet minute, one of them hovered.
    // Blue opens as the AI does and then just lets its outpost print, with pixels to print with.
    pg.start_match(true, 3);
    pg.help = false;
    {
        let p = pg.play.as_mut().unwrap();
        // Red does nothing, and has an outpost standing so its king isn't the story here.
        p.arena.eco.as_mut().unwrap().bot = [crate::pg::sim::Bot::Ai, crate::pg::sim::Bot::None];
        let rk = p.arena.eco.as_ref().unwrap().king[1];
        let (rx, ry) = (p.arena.ships[rk].x, p.arena.ships[rk].y);
        p.arena.spawn(1, 4, &crate::pg::design::starter_deck()[4], rx - 60, ry, 0, false);
        let waiting = |a: &crate::pg::sim::Arena| a.ships.iter().filter(|s| s.alive && s.team == 0 && s.posted && s.vx == 0 && s.vy == 0 && s.goal == Some((s.x, s.y))).count();
        while p.arena.over.is_none() && p.arena.tick < 15 * 360 && !(p.arena.tick > 15 * 120 && waiting(&p.arena) >= 14) {
            if p.arena.tick == 15 * 40 {
                p.arena.eco.as_mut().unwrap().bot[0] = crate::pg::sim::Bot::None;
            }
            let k = p.arena.eco.as_ref().unwrap().king[0];
            if p.arena.tick > 15 * 40 && p.arena.ships[k].store < 60 {
                p.arena.ships[k].store += 60;
                p.arena.score[0].mined += 60;
            }
            p.arena.step();
        }
        let a = &p.arena;
        if let Some(g) = a.ships.iter().filter(|s| s.alive && s.team == 0 && s.posted && s.name == "FRIGATE").last().or(a.ships.iter().filter(|s| s.alive && s.team == 0 && s.posted).last()) {
            let (hx, hy, gx, gy) = (g.home.0, g.home.1, g.x, g.y);
            p.cam = ((hx - 152).clamp(0, crate::pg::sim::MW - 304), (hy - 150).clamp(0, crate::pg::sim::MH - 300));
            pg.mouse = (gx - p.cam.0, gy - p.cam.1);
        }
    }
    pg.draw(&mut fb);
    save(&fb, &pg, "37_match_guards_at_their_posts.png");
    // Picking a rock for a seed: an outpost stands on the home rock, and the pointer is on the
    // nearest rock that would link to it but not to the king. Then, on the whole map, the
    // nearest rock that's out of reach of everything.
    pg.start_match(true, 3);
    pg.help = false;
    {
        use crate::pg::sim::{Arena, Cmd, Links};
        let p = pg.play.as_mut().unwrap();
        p.arena.eco.as_mut().unwrap().bot[1] = crate::pg::sim::Bot::None;
        let a = &mut p.arena;
        let k = a.eco.as_ref().unwrap().king[0];
        a.ships[k].store += 600;
        a.score[0].mined += 600;
        let king = (a.ships[k].x, a.ships[k].y);
        let home = a.rock_near((king.0 + 125, king.1 + 35), 40).unwrap();
        a.command(Cmd::Print { team: 0, slot: 4, target: Some(home) });
        for _ in 0..15 * 40 {
            a.step();
        }
        let rocks: Vec<(i32, i32)> = a.eco.as_ref().unwrap().rocks.iter().filter_map(|r| a.rock_near((r.cx, r.cy), 12)).collect();
        let nearest = |a: &Arena, want: &dyn Fn(&Links) -> bool| rocks.iter().copied().filter(|&px| a.plant_links(0, 4, px).is_some_and(|l| want(&l))).min_by_key(|&(x, y)| crate::geom::dist2(x, y, king.0, king.1));
        let chained = nearest(a, &|l| l.king && !l.to.contains(&king)).or_else(|| nearest(a, &|l| l.king && l.at != home)).unwrap();
        let far = nearest(a, &|l| l.gap.is_some()).unwrap();
        p.planting = Some(4);
        p.cam = (((chained.0 + home.0) / 2 - 152).clamp(0, crate::pg::sim::MW - 304), ((chained.1 + home.1) / 2 - 150).clamp(0, crate::pg::sim::MH - 300));
        pg.mouse = (chained.0 - p.cam.0, chained.1 - p.cam.1);
        pg.draw(&mut fb);
        save(&fb, &pg, "39_match_seed_links.png");
        let p = pg.play.as_mut().unwrap();
        p.tactical = true;
        pg.mouse = (12 + far.0 / 5, 49 + far.1 / 5);
        pg.draw(&mut fb);
        save(&fb, &pg, "40_match_seed_too_far.png");
    }
    // The king's queue: five jobs, the miner on repeat, the pointer on it.
    pg.start_match(true, 3);
    pg.help = false;
    {
        use crate::pg::sim::Cmd;
        let p = pg.play.as_mut().unwrap();
        p.arena.eco.as_mut().unwrap().bot[1] = crate::pg::sim::Bot::None;
        let a = &mut p.arena;
        let k = a.eco.as_ref().unwrap().king[0];
        a.ships[k].store += 600;
        a.score[0].mined += 600;
        for slot in [0, 1] {
            a.command(Cmd::Print { team: 0, slot, target: None });
        }
        a.command(Cmd::Repeat { team: 0, slot: 3, at: None });
        for slot in [2, 6] {
            a.command(Cmd::Print { team: 0, slot, target: None });
        }
        for _ in 0..15 * 9 {
            a.step();
        }
    }
    pg.mouse = (307 + 2 * 15 + 7, 147);
    pg.draw(&mut fb);
    save(&fb, &pg, "41_match_kings_queue.png");
    // The lag dial turned to three ticks, the pointer on it.
    {
        use crate::app::{Ev, Key};
        for _ in 0..3 {
            pg.event(Ev::Key(Key::Letter('L')));
        }
        pg.mouse = (280, 6);
        pg.draw(&mut fb);
        save(&fb, &pg, "42_match_lag_dial.png");
    }
    // The match menu with its two network buttons; then joining, the address being typed; then
    // hosting, waiting.
    pg.screen = Screen::Yard;
    pg.play = None;
    pg.menu = true;
    pg.mouse = (255, 225);
    pg.draw(&mut fb);
    save(&fb, &pg, "43_menu_host_or_join.png");
    pg.join(None);
    pg.draw(&mut fb);
    save(&fb, &pg, "44_net_join_address.png");
    pg.lobby = Some(crate::pg::link::Lobby::host_on("127.0.0.1", crate::pg::link::PORT));
    pg.draw(&mut fb);
    save(&fb, &pg, "45_net_hosting.png");
    pg.lobby = None;
    // A network match, played for real between two games on this machine, seen from the
    // joiner's side: red, top right, with red's bank and queue and its own cards.
    {
        use crate::app::{Ev, Key};
        use crate::pg::link::Lobby;
        let (mut host, mut join) = (Pg::new(None, None), Pg::new(None, None));
        host.lobby = Some(Lobby::host_on("127.0.0.1", 0));
        if let Some(Lobby::Hosting { port, .. }) = &host.lobby {
            join.join(Some(&format!("127.0.0.1:{port}")));
        }
        let started = std::time::Instant::now();
        let mut frames = 0;
        while started.elapsed() < std::time::Duration::from_secs(20) && frames < 15 * 30 {
            host.tick();
            join.tick();
            std::thread::sleep(std::time::Duration::from_millis(2));
            if join.screen != Screen::Match || host.screen != Screen::Match {
                continue;
            }
            frames += 1;
            match frames {
                1 => {
                    join.event(Ev::Key(Key::Num(4)));
                    join.event(Ev::Key(Key::Letter('R')));
                    host.event(Ev::Key(Key::Num(1)));
                }
                20 => join.hold([false, true, true, false]),
                60 => join.hold([false; 4]),
                _ => {}
            }
        }
        if join.screen == Screen::Match {
            join.mouse = (200, 6);
            join.draw(&mut fb);
            save(&fb, &join, "46_net_match_as_red.png");
        } else {
            println!("the two games never met: no 46_net_match_as_red.png");
        }
    }
    // The yard: Tab has stepped WEAPONS on to its second part, and the category says how.
    pg.screen = Screen::Yard;
    pg.play = None;
    pg.select_slot(2);
    pg.event(crate::app::Ev::Key(crate::app::Key::Num(2)));
    pg.event(crate::app::Ev::Key(crate::app::Key::Tab));
    // The pointer on the WEAPONS button (the second of the six, at 6 + 33, 228).
    pg.mouse = (6 + 33 + 16, 228 + 8);
    pg.draw(&mut fb);
    save(&fb, &pg, "38_yard_tab_next_part.png");
    // The game as it starts: the starter deck as your team, on an open field.
    let mut pg = Pg::new(None, None);
    pg.help = false;
    pg.mouse = (150, 240);
    pg.draw(&mut fb);
    save(&fb, &pg, "26_start_arena.png");
    // The fights the window had for the starter deck's slots until 2026-11-14.
    let mut pg = Pg::starter_fights();
    pg.help = false;
    pg.mouse = (150, 240);
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
    // Thirty fighters round a capital, flying as their side decides.
    pg.load_test(4);
    pg.running = true;
    for _ in 0..120 {
        pg.tick();
    }
    hover(&mut pg, 1);
    pg.draw(&mut fb);
    save(&fb, &pg, "47_fighters_orbit_a_capital.png");
    pg.running = false;
    pg.screen = Screen::Yard;
    pg.select_slot(5);
    pg.mouse = (262, 189);
    pg.draw(&mut fb);
    save(&fb, &pg, "29_start_yard_shipyard.png");
    // The library, open over the yard: single drawings, then whole teams.
    pg.select_slot(7);
    pg.event(crate::app::Ev::Key(crate::app::Key::Letter('L')));
    pg.mouse = (300, 196);
    pg.draw(&mut fb);
    save(&fb, &pg, "55_library_drawings.png");
    pg.event(crate::app::Ev::Key(crate::app::Key::Enter));
    pg.mouse = (30, 14);
    pg.draw(&mut fb);
    save(&fb, &pg, "56_yard_drawing_taken_into_slot_8.png");
    pg.event(crate::app::Ev::Key(crate::app::Key::Letter('L')));
    pg.event(crate::app::Ev::Key(crate::app::Key::Tab));
    pg.event(crate::app::Ev::Key(crate::app::Key::Letter('K')));
    pg.mouse = (300, 196);
    pg.draw(&mut fb);
    save(&fb, &pg, "57_library_teams.png");
    pg.event(crate::app::Ev::Key(crate::app::Key::Esc));
    // The beam and the shield, from the examples (`--parts`).
    let mut pp = Pg::parts();
    pp.help = false;
    let hover_slot = |pp: &mut Pg, slot: usize| {
        if let Some(s) = pp.arena.ships.iter().find(|s| s.alive && s.slot == slot) {
            pp.mouse = (s.x - pp.cam.0, s.y - pp.cam.1);
        }
    };
    // A beamer warming on an outpost.
    pp.load_test(0);
    pp.running = true;
    while pp.arena.warming().is_empty() && pp.arena.tick < 15 * 60 {
        pp.tick();
    }
    for _ in 0..8 {
        pp.tick();
    }
    hover_slot(&mut pp, 0);
    pp.draw(&mut fb);
    save(&fb, &pp, "48_beam_warming.png");
    // The tick a beam cuts, at a warden: it stops at the circle's edge.
    pp.load_test(7);
    pp.running = true;
    while pp.arena.cuts.is_empty() && pp.arena.tick < 15 * 120 {
        pp.tick();
    }
    hover_slot(&mut pp, 1);
    pp.draw(&mut fb);
    save(&fb, &pp, "49_beam_stopped_at_a_shield.png");
    // Fighters getting off two beamers' lines.
    pp.load_test(1);
    pp.running = true;
    while !pp.arena.ships.iter().any(|s| s.doing == crate::pg::sim::Doing::Dodge) && pp.arena.tick < 15 * 60 {
        pp.tick();
    }
    pp.tick();
    if let Some(s) = pp.arena.ships.iter().find(|s| s.alive && s.doing == crate::pg::sim::Doing::Dodge) {
        pp.mouse = (s.x - pp.cam.0, s.y - pp.cam.1);
    }
    pp.draw(&mut fb);
    save(&fb, &pp, "50_fighters_dodge_beams.png");
    // A warden's circles under lances, and under flak.
    for (t, name) in [(4, "51_lancers_vs_warden.png"), (5, "52_fighters_vs_warden.png")] {
        pp.load_test(t);
        pp.running = true;
        while pp.arena.score[0].shielded < 6 && pp.arena.tick < 15 * 60 {
            pp.tick();
        }
        hover_slot(&mut pp, 1);
        pp.draw(&mut fb);
        save(&fb, &pp, name);
    }
    // The two parts in the yard.
    pp.running = false;
    pp.screen = Screen::Yard;
    for (slot, cat, part, name) in [(0, crate::pg::design::Cat::Weapons, Part::Beam, "53_yard_beamer.png"), (1, crate::pg::design::Cat::Armor, Part::Shield, "54_yard_warden.png")] {
        pp.select_slot(slot);
        pp.ed.cat = cat;
        pp.ed.part = part;
        pp.mouse = pp.cell_screen((c + 6, c - 8));
        pp.draw(&mut fb);
        save(&fb, &pp, name);
    }
    // The missile: its example ship in the yard, with the weapons on two rows, and a volley
    // on its way to an outpost whose flak has started on it.
    pp.select_slot(7);
    pp.ed.cat = crate::pg::design::Cat::Weapons;
    pp.ed.part = Part::Missile;
    pp.mouse = (60, 262);
    pp.draw(&mut fb);
    save(&fb, &pp, "59_yard_archer.png");
    pp.screen = Screen::Arena;
    pp.load_test(11);
    pp.running = true;
    while pp.arena.score[1].downed < 2 && pp.arena.tick < 15 * 60 {
        pp.tick();
    }
    hover_slot(&mut pp, 7);
    pp.draw(&mut fb);
    save(&fb, &pp, "60_missiles_at_an_outpost.png");
    // The repeater and the projector: their example ships in the yard, each with the pointer
    // on its part's button.
    pp.screen = Screen::Yard;
    for (slot, part, name) in [(2, Part::Repeater, "61_yard_raker.png"), (8, Part::Projector, "63_yard_aegis.png")] {
        pp.select_slot(slot);
        pp.ed.cat = crate::pg::design::Cat::of(part);
        pp.ed.part = part;
        pp.mouse = pp.part_button(part);
        pp.draw(&mut fb);
        save(&fb, &pp, name);
    }
    // A projector drawn where it can't work: thin at its corners, and walled in on one side.
    let kept = pp.lib[8].clone();
    pp.lib[8] = crate::pg::design::Design::from_rows("SHUT IN", &["hjjjj.", "hjjjj.", "hjjjj.", "hjjjj.", "hhhhhh", "hhchhh", "hhhhhh", "e.ee.e"]);
    pp.select_slot(8);
    pp.mouse = (300, 150);
    pp.draw(&mut fb);
    save(&fb, &pp, "65_yard_projector_that_cant_work.png");
    pp.lib[8] = kept;
    pp.select_slot(8);
    // Two rakers' streams on a warden's circles.
    pp.screen = Screen::Arena;
    pp.load_test(8);
    pp.running = true;
    while pp.arena.score[0].shielded < 8 && pp.arena.tick < 15 * 60 {
        pp.tick();
    }
    hover_slot(&mut pp, 2);
    pp.draw(&mut fb);
    save(&fb, &pp, "62_rakers_stream_at_a_warden.png");
    // An AEGIS beside two beamers, with lancers' shots stopping at the skin it throws.
    pp.load_test(15);
    pp.running = true;
    while pp.arena.score[1].shielded < 6 && pp.arena.tick < 15 * 60 {
        pp.tick();
    }
    hover_slot(&mut pp, 8);
    pp.draw(&mut fb);
    save(&fb, &pp, "64_aegis_covers_two_beamers.png");
    // A mixed fleet against its twin, blue flown by the arch-AI and red flying as drawn.
    let mut pb = Pg::starter_fights();
    pb.help = false;
    pb.load_named(crate::pg::TWIN);
    pb.running = true;
    for _ in 0..15 * 5 {
        pb.tick();
    }
    hover_slot(&mut pb, 1);
    pb.draw(&mut fb);
    save(&fb, &pb, "66_arena_arch_ai_flies_a_mixed_fleet.png");
    // Selecting and sending in a match: a box dragged round ships by the king, then the same
    // ships sent across the field.
    {
        use crate::app::{App, Ev, Key};
        let mut ps = Pg::new(None, None);
        ps.start_match(true, 3);
        ps.help = false;
        let d = crate::pg::design::starter_deck();
        {
            let a = &mut ps.play.as_mut().unwrap().arena;
            for (slot, x, y) in [(0, 60, 820), (0, 75, 812), (0, 62, 838), (0, 80, 830), (1, 100, 800), (1, 112, 815), (1, 104, 832), (2, 60, 870), (2, 95, 866)] {
                a.spawn(0, slot, &d[slot], x, y, 1, false);
            }
        }
        ps.event(Ev::Move(40, 30));
        ps.event(Ev::Press { x: 40, y: 30, right: false });
        ps.event(Ev::Move(128, 136));
        ps.draw(&mut fb);
        save(&fb, &ps, "67_match_a_box_dragged_round_ships.png");
        ps.event(Ev::Release { x: 128, y: 136, right: false });
        ps.event(Ev::Press { x: 250, y: 60, right: true });
        ps.event(Ev::Release { x: 250, y: 60, right: true });
        for _ in 0..40 {
            ps.tick();
        }
        ps.event(Ev::Move(150, 200));
        ps.draw(&mut fb);
        save(&fb, &ps, "68_match_ships_selected_and_sent.png");
        // The same ships arrived, each on its own spot: fighters in front, long guns behind.
        for _ in 0..260 {
            ps.tick();
        }
        ps.draw(&mut fb);
        save(&fb, &ps, "69_match_ships_in_formation.png");
        // A factory selected, with the flag where its new ships gather; and the stance turned
        // to ATTACK.
        let post = ps.play.as_mut().unwrap().arena.spawn(0, 4, &d[4], 60, 960, 1, false);
        ps.event(Ev::Key(Key::Esc));
        ps.play.as_mut().unwrap().picked = vec![post];
        ps.event(Ev::Press { x: 170, y: 250, right: true });
        ps.event(Ev::Release { x: 170, y: 250, right: true });
        ps.event(Ev::Key(Key::Letter('T')));
        for _ in 0..3 {
            ps.tick();
        }
        ps.event(Ev::Move(BTN_STANCE_AT.0, BTN_STANCE_AT.1));
        ps.draw(&mut fb);
        save(&fb, &ps, "70_match_rally_point_and_all_attack.png");
        // The keys page, for each screen.
        ps.event(Ev::Key(Key::Keys));
        ps.draw(&mut fb);
        save(&fb, &ps, "71_keys_the_match.png");
        let mut pk = Pg::new(None, None);
        pk.help = false;
        pk.event(Ev::Key(Key::Keys));
        pk.draw(&mut fb);
        save(&fb, &pk, "72_keys_the_arena.png");
        pk.event(Ev::Key(Key::Keys));
        pk.event(Ev::Key(Key::Enter));
        pk.event(Ev::Key(Key::Keys));
        pk.draw(&mut fb);
        save(&fb, &pk, "73_keys_the_yard.png");
        pk.event(Ev::Key(Key::Keys));
        pk.event(Ev::Key(Key::Letter('L')));
        pk.event(Ev::Key(Key::Keys));
        pk.draw(&mut fb);
        save(&fb, &pk, "74_keys_the_library.png");
    }
    // The teams' fights, after your own: two armies as they start and once they've met, and the
    // biggest of them a while in.
    {
        let mut pt = Pg::new(None, None);
        pt.help = false;
        pt.load_named("SWARM VS STARTER DECK");
        pt.draw(&mut fb);
        save(&fb, &pt, "75_team_fights_swarm_v_starter.png");
        for (name, ticks, file) in [("SWARM VS STARTER DECK", 150, "76_team_fight_swarm_meets_starter.png"), ("BIG GUNS VS SIEGE", 300, "77_team_fight_big_guns_v_siege.png"), ("RAIDERS VS STARTER DECK", 200, "78_team_fight_raiders_v_starter.png")] {
            pt.load_named(name);
            pt.running = true;
            for _ in 0..ticks {
                pt.tick();
            }
            pt.draw(&mut fb);
            save(&fb, &pt, file);
        }
    }
    new_parts_screenshots(dir);
    println!("exported proving-ground screenshots to {}", dir.display());
}

/// The parts of 2026-11-15 on screen: the BASTION in the yard with the pointer on the piercer's
/// button, and its fight against seventeen lancers four seconds in.
pub fn new_parts_screenshots(dir: &Path) {
    use crate::app::App;
    use crate::pg::design::{part_examples, Cat, Part};
    use crate::pg::{Pg, Screen};
    let _ = std::fs::create_dir_all(dir);
    let mut fb = Framebuffer::new();
    let save = |fb: &Framebuffer, pg: &Pg, name: &str| fb.save_png(&dir.join(name), pg.palette());
    let mut pg = Pg::new(None, None);
    pg.help = false;
    pg.running = false;
    pg.screen = Screen::Yard;
    pg.lib[5] = part_examples().into_iter().find(|d| d.name == "BASTION").unwrap();
    pg.select_slot(5);
    pg.ed.cat = Cat::Upgrades;
    pg.ed.part = Part::Piercer;
    pg.mouse = pg.part_button(Part::Piercer);
    pg.draw(&mut fb);
    save(&fb, &pg, "60_yard_new_parts.png");
    pg.screen = Screen::Arena;
    if pg.load_named("1 BASTION VS 23 LANCERS") {
        pg.running = true;
        for _ in 0..15 * 4 {
            pg.tick();
        }
        pg.mouse = (0, 0);
        pg.draw(&mut fb);
        save(&fb, &pg, "61_bastion_vs_lancers.png");
    }
    // The parts of 2026-11-17 in the yard: the dock pad, and the energy ball in the seventh
    // category.
    for (name, cat, part, file) in [("PADPOST", Cat::Industry, Part::DockPad, "62_yard_dock_pad.png"), ("SAPPER", Cat::Heavy, Part::EnergyBall, "63_yard_energy_ball.png"), ("HAULER", Cat::Industry, Part::Scoop, "64_yard_hauler.png")] {
        pg.screen = Screen::Yard;
        pg.lib[5] = part_examples().into_iter().find(|d| d.name == name).unwrap();
        pg.select_slot(5);
        pg.ed.cat = cat;
        pg.ed.part = part;
        pg.mouse = pg.part_button(part);
        pg.draw(&mut fb);
        save(&fb, &pg, file);
    }
    // A pad at work in a match: the example outpost with its pad, six full miners by it, at
    // the moment one's load is going, and a second and a half later.
    {
        let mut pg = Pg::new(None, None);
        pg.lib = crate::pg::design::starter_deck();
        pg.start_match(true, 3);
        pg.help = false;
        let d = crate::pg::design::starter_deck();
        let post = {
            let a = &mut pg.play.as_mut().unwrap().arena;
            let k = a.eco.as_ref().unwrap().king[0];
            let (kx, ky) = (a.ships[k].x, a.ships[k].y);
            let post = a.spawn(0, 4, &part_examples().into_iter().find(|d| d.name == "PADPOST").unwrap(), kx + 80, ky - 40, 0, false);
            for n in 0..6 {
                let m = a.spawn(0, 3, &d[3], kx + 118 + 7 * (n % 3), ky - 62 + 22 * (n / 3), 0, false);
                a.ships[m].carry = 9;
            }
            post
        };
        let (mut shown, mut after) = (false, 0);
        for _ in 0..15 * 30 {
            pg.tick();
            let p = pg.play.as_ref().unwrap();
            let going = p.arena.ships.iter().any(|s| s.alive && s.berth.is_some_and(|b| b.phase == 1 && b.t == crate::pg::design::DOCK_SHOW / 2));
            after += shown as i32;
            if (going && !shown) || after == 22 {
                let s = &p.arena.ships[post];
                pg.mouse = (s.x - p.cam.0 - 8, s.y - p.cam.1);
                pg.draw(&mut fb);
                save(&fb, &pg, if shown { "66_dock_pad_later.png" } else { "65_dock_pad_unloading.png" });
                if shown {
                    break;
                }
                shown = true;
            }
        }
    }
    // The stasis lattice (2026-11-18): its example in the yard, and in a match with its field
    // up: parked by blue's king, three blue fighters beside it and six red ones come at it.
    {
        pg.screen = Screen::Yard;
        pg.lib[5] = part_examples().into_iter().find(|d| d.name == "ANCHOR").unwrap();
        pg.select_slot(5);
        pg.ed.cat = Cat::Drive;
        pg.ed.part = Part::Stasis;
        pg.mouse = pg.part_button(Part::Stasis);
        pg.draw(&mut fb);
        save(&fb, &pg, "67_yard_stasis.png");
        let mut pg = Pg::new(None, None);
        pg.lib = crate::pg::design::starter_deck();
        pg.start_match(true, 3);
        pg.help = false;
        let d = crate::pg::design::starter_deck();
        let anchor = {
            let a = &mut pg.play.as_mut().unwrap().arena;
            let k = a.eco.as_ref().unwrap().king[0];
            let (kx, ky) = (a.ships[k].x, a.ships[k].y);
            let anchor = a.spawn(0, 4, &part_examples().into_iter().find(|d| d.name == "ANCHOR").unwrap(), kx + 110, ky - 30, 0, false);
            a.ships[anchor].sent = Some((kx + 110, ky - 30));
            for n in 0..3 {
                let f = a.spawn(0, 0, &d[0], kx + 70, ky - 50 + 20 * n, 1, false);
                a.ships[f].sent = Some((kx + 70, ky - 50 + 20 * n));
            }
            for n in 0..6 {
                a.spawn(1, 0, &d[0], kx + 190 + 12 * (n % 2), ky - 70 + 16 * (n / 2), 3, false);
            }
            anchor
        };
        for t in 0..15 * 12 {
            pg.tick();
            let p = pg.play.as_ref().unwrap();
            let up = p.arena.stasis.iter().any(|f| f.ship == anchor && f.up);
            let held = p.arena.ships.iter().filter(|s| s.alive && s.slowed).count();
            if t == 15 * 12 - 1 || (up && held >= 4 && t >= 15 * 4) {
                let s = &p.arena.ships[anchor];
                pg.mouse = (s.x - p.cam.0, s.y - p.cam.1 + 3);
                pg.draw(&mut fb);
                save(&fb, &pg, "68_stasis_field_up.png");
                println!("68: tick {t}, the field is {}, {held} ships held", if up { "up" } else { "not up" });
                break;
            }
        }
    }
    // The slipstream (2026-11-18): its example in the yard, and in a match: a pace ship sent
    // off with two frigates and a capital, which leave green exhaust while it has them.
    {
        pg.screen = Screen::Yard;
        pg.lib[5] = part_examples().into_iter().find(|d| d.name == "PACER").unwrap();
        pg.select_slot(5);
        pg.ed.cat = Cat::Drive;
        pg.ed.part = Part::Slipstream;
        pg.mouse = pg.part_button(Part::Slipstream);
        pg.draw(&mut fb);
        save(&fb, &pg, "69_yard_slipstream.png");
        let mut pg = Pg::new(None, None);
        pg.lib = crate::pg::design::starter_deck();
        pg.start_match(true, 3);
        pg.help = false;
        let d = crate::pg::design::starter_deck();
        let (pacer, wards) = {
            let a = &mut pg.play.as_mut().unwrap().arena;
            let k = a.eco.as_ref().unwrap().king[0];
            let (kx, ky) = (a.ships[k].x, a.ships[k].y);
            let pacer = a.spawn(0, 4, &part_examples().into_iter().find(|d| d.name == "PACER").unwrap(), kx + 80, ky - 40, 1, false);
            let wards = vec![a.spawn(0, 2, &d[2], kx + 84, ky - 68, 1, false), a.spawn(0, 2, &d[2], kx + 84, ky - 12, 1, false), a.spawn(0, 6, &d[6], kx + 40, ky - 40, 1, false)];
            a.step();
            let all: Vec<usize> = wards.iter().copied().chain([pacer]).collect();
            let pace = all.iter().map(|&i| a.ships[i].st.vmax).min().unwrap();
            for &i in &all {
                (a.ships[i].sent, a.ships[i].pace) = (Some((a.ships[i].x + 500, a.ships[i].y)), pace);
            }
            println!("70: the wards' own speeds {:?}, the pace {pace}", wards.iter().map(|&i| a.ships[i].vmax_own()).collect::<Vec<_>>());
            (pacer, wards)
        };
        for _ in 0..15 * 5 {
            pg.tick();
        }
        let p = pg.play.as_ref().unwrap();
        let s = &p.arena.ships[pacer];
        println!("70: {} of {} sped; the pace ship at {:?}, the camera {:?}", wards.iter().filter(|&&i| p.arena.ships[i].sped).count(), wards.len(), (s.x, s.y), p.cam);
        pg.mouse = (s.x - p.cam.0, s.y - p.cam.1 + 3);
        pg.draw(&mut fb);
        save(&fb, &pg, "70_slipstream_fleet.png");
    }
    // The vacuum (2026-11-18): its example in the yard, and in a match: a sweeper come to a
    // field of wreckage by blue's king, the loose pixels streaming in to it.
    {
        pg.screen = Screen::Yard;
        pg.lib[5] = part_examples().into_iter().find(|d| d.name == "SWEEPER").unwrap();
        pg.select_slot(5);
        pg.ed.cat = Cat::Industry;
        pg.ed.part = Part::Vacuum;
        pg.mouse = pg.part_button(Part::Vacuum);
        pg.draw(&mut fb);
        save(&fb, &pg, "71_yard_vacuum.png");
        let mut pg = Pg::new(None, None);
        pg.lib = crate::pg::design::starter_deck();
        pg.start_match(true, 3);
        pg.help = false;
        let sweeper = {
            let a = &mut pg.play.as_mut().unwrap().arena;
            let k = a.eco.as_ref().unwrap().king[0];
            let (kx, ky) = (a.ships[k].x, a.ships[k].y);
            // Wreckage as a fight leaves it: some 200 loose pixels strewn over a patch 80 px
            // across.
            let mut rng = crate::rng::Rng::new(7);
            for _ in 0..260 {
                let (dx, dy) = (rng.range(-40, 40), rng.range(-40, 40));
                if dx * dx + dy * dy <= 40 * 40 {
                    a.loose_at(kx + 85 + dx, ky - 55 + dy);
                }
            }
            // (In the miner's slot of the deck: a ship does as its slot's card does.)
            a.spawn(0, 3, &part_examples().into_iter().find(|d| d.name == "SWEEPER").unwrap(), kx + 20, ky - 90, 1, false)
        };
        let mut shot = false;
        for t in 0..15 * 30 {
            pg.tick();
            let p = pg.play.as_ref().unwrap();
            let s = &p.arena.ships[sweeper];
            if !shot && s.carry >= 30 {
                println!("72: tick {t}, it carries {} of {}, {} loose pixels left", s.carry, s.st.hold(), p.arena.debris.len());
                pg.mouse = (s.x - p.cam.0, s.y - p.cam.1 + 3);
                pg.draw(&mut fb);
                save(&fb, &pg, "72_vacuum_at_work.png");
                shot = true;
            }
        }
        let p = pg.play.as_ref().unwrap();
        let s = &p.arena.ships[sweeper];
        println!("72: 30 s on it carries {}, is {}, and {} loose pixels are left", s.carry, s.doing.name(), p.arena.debris.len());
        assert!(shot);
    }
    // Two decks for the headless matches: the starter deck with the pad on its outpost, and
    // that with the hauler for its miner.
    let mut deck = crate::pg::design::starter_deck();
    let find = |name: &str| part_examples().into_iter().find(|d| d.name == name).unwrap();
    deck[4] = find("PADPOST");
    let text = |deck: &[crate::pg::design::Design]| format!("{}\n{}\n", crate::pg::design::SAVE_HEADER, deck.iter().map(|d| d.to_line()).collect::<Vec<_>>().join("\n"));
    let _ = std::fs::write(dir.join("deck_pad.txt"), text(&deck));
    let _ = std::fs::write(dir.join("deck_scavengers.txt"), text(&crate::pg::teams::scavengers()));
    {
        pg.screen = Screen::Yard;
        pg.lib = crate::pg::teams::scavengers();
        for slot in 0..pg.lib.len() {
            pg.select_slot(slot);
            pg.mouse = (0, 0);
            pg.draw(&mut fb);
            let name = format!("8{slot}_scavengers_{}.png", pg.lib[slot].name.to_lowercase());
            save(&fb, &pg, &name);
        }
    }
    deck[3] = find("HAULER");
    let _ = std::fs::write(dir.join("deck_pad_hauler.txt"), text(&deck));
    for d in [find("PADPOST"), find("HAULER"), find("SAPPER"), find("ANCHOR"), find("PACER"), find("SWEEPER"), find("DUSTER")] {
        let st = d.stats();
        println!("{}: {} px, cost {}, power {}%, {} px/s, problem {:?}", d.name, st.count, st.cost, st.power_pct(), st.top_speed_px_s(), d.check().problem());
    }
}

/// The referee for the rollout pilot (`--nose-referee [N]`): the flight toy's fights on N rolls of
/// the dice each (16 if not given), with a side that picks its own courses against a side on the
/// plain pilot, each as blue and as red, and with both sides picking. For each fight and over all
/// of them: how the thinking side did against the plain one, the pixels each lost, how long the
/// fights ran, and how many quarter turns a thinking ship makes in ten seconds.
pub fn nose_referee(seeds: u64, parts: bool) {
    use crate::geom::cheb;
    use crate::pg::design::{CANNON_RANGE, FLAK_RANGE};
    use crate::pg::sim::Arena;
    /// What one side's ships did in a fight, or in many.
    #[derive(Clone, Copy, Default)]
    struct Side {
        /// Ship-ticks alive; with an enemy inside flak range; with one inside cannon range.
        alive: i64,
        close: i64,
        near: i64,
        /// Quarter turns made.
        turns: i64,
        /// How far round their nearest enemy they went while it was inside cannon range, in
        /// thousandths of a circle.
        round: i64,
        /// Shots fired; shots fired at them, and the ones that hit; pixels lost.
        fired: i64,
        shot_at: i64,
        hit: i64,
        px_lost: i64,
    }
    impl Side {
        fn add(&mut self, o: &Side) {
            for (mine, theirs) in [
                (&mut self.alive, o.alive),
                (&mut self.close, o.close),
                (&mut self.near, o.near),
                (&mut self.turns, o.turns),
                (&mut self.round, o.round),
                (&mut self.fired, o.fired),
                (&mut self.shot_at, o.shot_at),
                (&mut self.hit, o.hit),
                (&mut self.px_lost, o.px_lost),
            ] {
                *mine += theirs;
            }
        }
        /// Its ships' behaviour in a line.
        fn line(&self) -> String {
            let per = |n: i64, of: i64| n as f64 / of.max(1) as f64;
            format!(
                "in flak range {:2.0}% of the time, {:.2} shots a ship-second, {:.2} pixel hits taken a shot at them, {:.2} circles of their nearest enemy in 10 s near it, {:.1} quarter turns in 10 s",
                100.0 * per(self.close, self.alive),
                15.0 * per(self.fired, self.alive),
                per(self.hit, self.shot_at),
                0.15 * per(self.round, self.near),
                150.0 * per(self.turns, self.alive)
            )
        }
    }
    let (lib, tests) = if parts { (crate::pg::parts_library(), crate::pg::part_tests()) } else { (crate::pg::flight_library(), crate::pg::flight_tests()) };
    let lib = &lib;
    // One fight: who won (0 blue, 1 red, 2 nobody), its ticks, and what each side's ships did.
    let fight = |place: &Vec<(usize, usize, i32, i32)>, plain: [bool; 2], seed: u64| {
        let mut a = Arena::seeded(seed);
        a.plain = plain;
        for &(team, slot, x, y) in place {
            a.add(team, slot, &lib[slot], x, y);
        }
        let n = a.ships.len();
        let mut facing: Vec<u8> = a.ships.iter().map(|s| s.facing).collect();
        // Each ship's nearest enemy a tick ago, and the bearing it had from it.
        let mut bearing: Vec<Option<(usize, f64)>> = vec![None; n];
        let mut sides = [Side::default(); 2];
        while a.over.is_none() {
            a.step();
            for k in 0..n {
                let s = &a.ships[k];
                if !s.alive {
                    continue;
                }
                let side = &mut sides[s.team];
                side.alive += 1;
                side.turns += (s.facing != facing[k]) as i64;
                facing[k] = s.facing;
                let nearest = a.ships.iter().enumerate().filter(|(_, o)| o.alive && o.team != s.team).min_by_key(|(j, o)| ((o.x - s.x).pow(2) + (o.y - s.y).pow(2), *j));
                let was = bearing[k].take();
                if let Some((j, o)) = nearest {
                    let d = cheb(o.x, o.y, s.x, s.y);
                    side.close += (d <= FLAK_RANGE) as i64;
                    if d <= CANNON_RANGE {
                        side.near += 1;
                        let now = ((s.y - o.y) as f64).atan2((s.x - o.x) as f64);
                        if let Some((j0, then)) = was {
                            if j0 == j {
                                let turned = (now - then).rem_euclid(std::f64::consts::TAU);
                                side.round += (turned.min(std::f64::consts::TAU - turned) / std::f64::consts::TAU * 1000.0).round() as i64;
                            }
                        }
                        bearing[k] = Some((j, now));
                    }
                }
            }
        }
        for t in 0..2 {
            sides[t].fired = a.score[t].fired as i64;
            sides[t].px_lost = a.score[t].px_lost as i64;
            sides[t].shot_at = a.score[1 - t].fired as i64;
            sides[t].hit = (a.score[1 - t].hits + a.score[1 - t].bounced) as i64;
        }
        let won = match a.over.unwrap() {
            Some(0) => 0usize,
            Some(_) => 1,
            None => 2,
        };
        (won, a.tick, sides)
    };
    let clock = std::time::Instant::now();
    // Thinking ships against plain ones, plain against thinking, thinking against thinking,
    // plain against plain: over every fight.
    let mut every = [Side::default(); 4];
    let (mut all, mut ticks) = ([0i32; 3], 0u64);
    println!("the {} toy's fights, flown nose-first, {seeds} rolls each, with neither side picking its own courses (both on the plain pilot), with blue picking, with red, and with both. Blue's wins, red's, draws, seconds a fight; then for each side the px it lost a fight, the pixel hits it took for each shot fired at it, the circles its ships made of their nearest enemy in 10 s near it, and the share of the time they had an enemy in flak range", if parts { "parts" } else { "flight" });
    // (A fight that places nothing, as the open field does, isn't one.)
    for t in tests.into_iter().filter(|t| !t.place.is_empty()) {
        let place = &t.place;
        println!("  {}", t.name);
        for (mode, (name, plain)) in [("neither thinks", [true, true]), ("blue thinks", [false, true]), ("red thinks", [true, false]), ("both think", [false, false])].into_iter().enumerate() {
            let rs: Vec<(usize, u32, [Side; 2])> = std::thread::scope(|scope| {
                let going: Vec<_> = (1..=seeds).map(|seed| scope.spawn(move || fight(place, plain, seed))).collect();
                going.into_iter().map(|g| g.join().unwrap()).collect()
            });
            let won = |k: usize| rs.iter().filter(|r| r.0 == k).count() as i32;
            let mut sides = [Side::default(); 2];
            for r in &rs {
                sides[0].add(&r.2[0]);
                sides[1].add(&r.2[1]);
            }
            let short = |s: &Side| {
                let per = |n: i64, of: i64| n as f64 / of.max(1) as f64;
                format!("lost {:3} px, {:.2} hits a shot, {:.2} circles, {:2.0}% in flak range", s.px_lost / seeds.max(1) as i64, per(s.hit, s.shot_at), 0.15 * per(s.round, s.near), 100.0 * per(s.close, s.alive))
            };
            println!("    {name:15} {:2} {:2} {:2}, {:3} s | blue {} | red {}", won(0), won(1), won(2), rs.iter().map(|r| r.1 as u64).sum::<u64>() / 15 / seeds.max(1), short(&sides[0]), short(&sides[1]));
            match mode {
                0 | 3 => {
                    let k = if mode == 0 { 3 } else { 2 };
                    every[k].add(&sides[0]);
                    every[k].add(&sides[1]);
                }
                _ => {
                    let me = mode - 1;
                    every[0].add(&sides[me]);
                    every[1].add(&sides[1 - me]);
                    all[0] += won(me);
                    all[1] += won(1 - me);
                    all[2] += won(2);
                    ticks += rs.iter().map(|r| r.1 as u64).sum::<u64>();
                }
            }
        }
    }
    println!("=== all fights, a thinking side against a plain one: won {}, lost {}, drew {}; lost {} px and took {} ===", all[0], all[1], all[2], every[0].px_lost, every[1].px_lost);
    for (k, who) in ["thinking ships against plain ones", "plain ships against thinking ones", "thinking ships against thinking ones", "plain ships against plain ones"].iter().enumerate() {
        println!("  {who:36}: {}", every[k].line());
    }
    println!("({} ticks of thinking-against-plain fights; the whole run took {:.1} s)", ticks, clock.elapsed().as_secs_f64());
}

/// Where the ships went (`--nose-trails [DIR]`): each of the flight toy's fights, on one roll of
/// the dice, on the plain pilot and with every ship picking its own course, as a picture of its
/// first twenty seconds. A dot a tick for each ship in its side's colour, a bright one each
/// second (so beads far apart are a ship going fast), a white dot where it began, a gold square
/// where it died and a white box round it where it was at the end.
pub fn nose_trails(dir: &Path) {
    use crate::app::App;
    use crate::palette as pal;
    use crate::pg::sim::Arena;
    let _ = std::fs::create_dir_all(dir);
    let lib = crate::pg::flight_library();
    let pg = crate::pg::Pg::flight();
    // The middle 400 x 300 of the map, pixel for pixel.
    let (ox, oy) = (40, 30);
    for (k, t) in crate::pg::flight_tests().into_iter().enumerate() {
        for (mode, plain) in [("PLAIN PILOT", [true; 2]), ("OWN COURSES", [false; 2])] {
            let mut a = Arena::seeded(1);
            a.plain = plain;
            for &(team, slot, x, y) in &t.place {
                a.add(team, slot, &lib[slot], x, y);
            }
            let n = a.ships.len();
            let began: Vec<(i32, i32)> = a.ships.iter().map(|s| (s.x, s.y)).collect();
            let mut last = began.clone();
            let mut died = vec![false; n];
            let mut fb = Framebuffer::new();
            fb.clear(pal::SPACE);
            let mut ticks = 0;
            while ticks < 300 && a.over.is_none() {
                a.step();
                ticks += 1;
                for i in 0..n {
                    let s = &a.ships[i];
                    if died[i] {
                        continue;
                    }
                    if !s.alive {
                        died[i] = true;
                        fb.rect_fill(last[i].0 - ox - 1, last[i].1 - oy - 1, 3, 3, pal::GOLD);
                        continue;
                    }
                    last[i] = (s.x, s.y);
                    fb.pset(s.x - ox, s.y - oy, pal::team_col(s.team, if ticks % 15 == 0 { 3 } else { 1 }));
                }
            }
            for i in 0..n {
                fb.pset(began[i].0 - ox, began[i].1 - oy, pal::WHITE);
                if !died[i] {
                    fb.rect_outline(last[i].0 - ox - 3, last[i].1 - oy - 3, 7, 7, pal::WHITE);
                }
            }
            fb.text(&format!("{}, {mode}, {} S", t.name, ticks / 15), 4, 4, pal::TEXT, None, 1);
            fb.save_png(&dir.join(format!("{k}_{}_{}.png", t.name.to_lowercase().replace(' ', "_"), mode.to_lowercase().replace(' ', "_"))), pg.palette());
        }
    }
}

/// Frames of the nose-first flight toy (`--nose`): each of its fights at a few moments, and the
/// yard with the rotation engine chosen, and with an engine chosen and turned.
pub fn nose_screenshots(dir: &Path) {
    use crate::app::App;
    use crate::pg::design::{Cat, Part};
    use crate::pg::{Pg, Screen};
    let _ = std::fs::create_dir_all(dir);
    let mut fb = Framebuffer::new();
    let save = |fb: &Framebuffer, pg: &Pg, name: &str| fb.save_png(&dir.join(name), pg.palette());
    let mut pg = Pg::flight();
    pg.help = false;
    let names: Vec<&'static str> = crate::pg::flight_tests().iter().map(|t| t.name).collect();
    for (k, name) in names.iter().enumerate() {
        if !pg.load_named(name) {
            continue;
        }
        pg.running = true;
        let mut at = 0;
        for secs in [1, 3, 6, 10] {
            while at < secs * 15 {
                pg.tick();
                at += 1;
            }
            pg.mouse = (0, 0);
            pg.draw(&mut fb);
            save(&fb, &pg, &format!("7{k}_{}_{secs:02}s.png", name.to_lowercase().replace(' ', "_")));
        }
    }
    pg.running = false;
    pg.screen = Screen::Yard;
    pg.select_slot(0);
    pg.ed.cat = Cat::Drive;
    pg.ed.part = Part::Gyro;
    pg.mouse = pg.part_button(Part::Gyro);
    pg.draw(&mut fb);
    save(&fb, &pg, "79_yard_rotation_engine.png");
    // The skater, with the engine chosen and turned once: its flank engines push across.
    pg.select_slot(1);
    pg.ed.part = Part::Engine;
    pg.ed.engine_way = 1;
    pg.mouse = pg.part_button(Part::Engine);
    pg.draw(&mut fb);
    save(&fb, &pg, "78_yard_engine_turned.png");
    println!("exported nose-first flight screenshots to {}", dir.display());
}

/// Screenshots of the tractor at work (2026-11-16): full miners being emptied at the king, a
/// collector gone out to the rock they work, and the yard with the part and the collector.
pub fn tractor_screenshots(dir: &Path) {
    use crate::app::App;
    use crate::pg::design::{starter_deck, Cat, Part};
    use crate::pg::{Pg, Screen};
    let _ = std::fs::create_dir_all(dir);
    let mut fb = Framebuffer::new();
    let save = |fb: &Framebuffer, pg: &Pg, name: &str| fb.save_png(&dir.join(name), pg.palette());
    let mut pg = Pg::new(None, None);
    pg.lib = starter_deck();
    pg.start_match(true, 3);
    pg.help = false;
    let d = starter_deck();
    let collector = {
        let a = &mut pg.play.as_mut().unwrap().arena;
        let k = a.eco.as_ref().unwrap().king[0];
        let (kx, ky) = (a.ships[k].x, a.ships[k].y);
        for n in 0..7 {
            let m = a.spawn(0, 3, &d[3], kx + 30 + 8 * (n % 4), ky - 34 + 6 * n, 0, false);
            a.ships[m].carry = 9;
        }
        a.spawn(0, 7, &d[7], kx + 60, ky + 24, 0, false)
    };
    let mut at = 0;
    for secs in [2, 6, 45, 90, 150] {
        while at < secs * 15 {
            pg.tick();
            at += 1;
        }
        // The pointer on the collector, so the bar says what it's doing.
        let p = pg.play.as_ref().unwrap();
        let s = &p.arena.ships[collector];
        pg.mouse = (s.x - p.cam.0, s.y - p.cam.1);
        pg.draw(&mut fb);
        save(&fb, &pg, &format!("tractor_{secs:03}s.png"));
    }
    pg.screen = Screen::Yard;
    pg.select_slot(7);
    pg.ed.cat = Cat::Industry;
    pg.ed.part = Part::Tractor;
    pg.mouse = pg.part_button(Part::Tractor);
    pg.draw(&mut fb);
    save(&fb, &pg, "tractor_yard.png");
    // The outpost's build order: a start row and a repeat row, the start row lit.
    pg.select_slot(4);
    pg.lib[4].first = vec![3, 3, 7];
    pg.ed.add_to_start = true;
    pg.mouse = (214, 166);
    pg.draw(&mut fb);
    save(&fb, &pg, "tractor_yard_builds.png");
    println!("exported tractor screenshots to {}", dir.display());
}

/// A picture of the whole map for each way its rocks can be laid out, on the first seed that
/// falls on each.
pub fn map_screenshots(dir: &Path) {
    use crate::app::{App, Ev, Key};
    use crate::pg::design::starter_deck;
    use crate::pg::sim::{Layout, LAYOUTS};
    use crate::pg::Pg;
    let _ = std::fs::create_dir_all(dir);
    let mut fb = Framebuffer::new();
    for layout in LAYOUTS {
        let seed = (1..).find(|&s| Layout::pick(s) == layout).unwrap();
        let mut pg = Pg::new(None, None);
        pg.lib = starter_deck();
        pg.start_match(true, seed);
        pg.help = false;
        pg.event(Ev::Key(Key::Tab));
        pg.mouse = (399, 299);
        pg.draw(&mut fb);
        let name = format!("map_{}_seed{seed}.png", layout.name().to_lowercase().replace(' ', "_"));
        fb.save_png(&dir.join(&name), pg.palette());
        let a = &pg.play.as_ref().unwrap().arena;
        println!("{name}: {} rocks, {} px of rock", a.eco.as_ref().unwrap().rocks.len(), a.eco.as_ref().unwrap().rocks.iter().map(|r| r.px.len()).sum::<usize>());
    }
}

#[cfg(test)]
mod new_parts_shots {
    /// Run by hand: `cargo test --release new_parts_shots -- --ignored`.
    #[test]
    #[ignore]
    fn new_parts_shots() {
        super::new_parts_screenshots(&std::env::temp_dir().join("nomindwing_new_parts"));
    }
}

/// A point on the match's stance button, for the screenshot that points at it.
const BTN_STANCE_AT: (i32, i32) = (60, 6);

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
