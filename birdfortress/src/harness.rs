//! Headless tools for designing and balancing levels: reference checks, ASCII route views, a
//! randomized solver that hill-climbs each score, and screenshot export.

use crate::layout::{Layout, NodeRef};
use crate::level::{Level, Tile};
use crate::rules::*;
use crate::sim::{self, Score};
use crate::world::World;
use std::collections::HashSet;

pub const HISTO_BINS: usize = 16;

pub fn histo_bin(v: i32, lo: i32, hi: i32) -> usize {
    if hi <= lo {
        return 0;
    }
    (((v - lo).clamp(0, hi - lo) as i64 * (HISTO_BINS as i64 - 1) + (hi - lo) as i64 / 2) / (hi - lo) as i64) as usize
}

/// Per-level histograms of solver-found scores, for the results screen.
#[derive(Clone, Debug, Default)]
pub struct Histo {
    /// [metric] = (lo, hi, bins)
    pub m: Vec<(i32, i32, Vec<u32>)>,
}

pub fn parse_histos(text: &str) -> Vec<(String, Histo)> {
    let mut out: Vec<(String, Histo)> = Vec::new();
    for line in text.lines() {
        let p: Vec<&str> = line.split_whitespace().collect();
        if p.first() != Some(&"histo") || p.len() < 5 + HISTO_BINS {
            continue;
        }
        let id = p[1].to_string();
        let lo = p[3].parse().unwrap_or(0);
        let hi = p[4].parse().unwrap_or(0);
        let bins: Vec<u32> = p[5..5 + HISTO_BINS].iter().map(|s| s.parse().unwrap_or(0)).collect();
        if let Some(e) = out.iter_mut().find(|(i, _)| *i == id) {
            e.1.m.push((lo, hi, bins));
        } else {
            out.push((id, Histo { m: vec![(lo, hi, bins)] }));
        }
    }
    out
}

pub struct Rng(u64);

impl Rng {
    pub fn new(seed: u64) -> Rng {
        Rng(seed.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407) | 1)
    }
    pub fn next(&mut self) -> u64 {
        let mut x = self.0;
        x ^= x << 13;
        x ^= x >> 7;
        x ^= x << 17;
        self.0 = x;
        x
    }
    pub fn below(&mut self, n: usize) -> usize {
        if n == 0 {
            0
        } else {
            (self.next() % n as u64) as usize
        }
    }
    pub fn chance(&mut self, p: f64) -> bool {
        (self.next() % 10_000) as f64 / 10_000.0 < p
    }
    pub fn pick<T: Copy>(&mut self, v: &[T]) -> Option<T> {
        if v.is_empty() {
            None
        } else {
            Some(v[self.below(v.len())])
        }
    }
}

pub fn tile_char(lv: &Level, w: &World, x: i32, y: i32) -> char {
    if let Some(n) = w.node_at(x, y) {
        return match n {
            NodeRef::Fortress => 'F',
            NodeRef::Source(i) => match lv.sources[i].good {
                Good::Berry => 'b',
                Good::Twig => 't',
                Good::Stone => 'q',
                Good::Clay => 'c',
                _ => 'e',
            },
            NodeRef::Building(_) => 'D',
        };
    }
    if w.is_spring(x, y) {
        return '^';
    }
    if w.is_plank(x, y) {
        return '_';
    }
    match w.tile(x, y) {
        Tile::Earth => '#',
        Tile::Rock => '%',
        Tile::Bramble => 'X',
        Tile::Branch => '=',
        Tile::Ledge => '-',
        Tile::Trunk => '|',
        Tile::Leaves => ':',
        Tile::Vine => 'v',
        Tile::Water => '~',
        Tile::Ladder => 'H',
        Tile::Cloud => '@',
        Tile::Air => '.',
    }
}

pub fn show(lv: &Level, l: &Layout) {
    let w = World::new(lv, l);
    let mut grid: Vec<Vec<char>> = (0..lv.h).map(|y| (0..lv.w).map(|x| tile_char(lv, &w, x, y)).collect()).collect();
    for (ri, r) in l.routes.iter().enumerate() {
        let p = sim::plan_route(lv, l, &w, r);
        match &p.problem {
            Some(pr) => println!("route {ri}: {} {:?}->{:?} {} x{}: {}", r.species.name(), r.from, r.to, r.cargo.name(), r.count, pr.text()),
            None => {
                println!("route {ri}: {} {:?}->{:?} {} x{}: there {} back {} cycle {}", r.species.name(), r.from, r.to, r.cargo.name(), r.count, p.loaded.ticks, p.empty.ticks, p.cycle());
                for s in &p.empty.steps {
                    if grid[s.y as usize][s.x as usize] == '.' {
                        grid[s.y as usize][s.x as usize] = 'o';
                    }
                }
                for s in &p.loaded.steps {
                    let c = &mut grid[s.y as usize][s.x as usize];
                    if *c == '.' || *c == 'o' {
                        *c = '*';
                    }
                }
            }
        }
    }
    println!("    {}", (0..lv.w).map(|x| char::from(b'0' + (x % 10) as u8)).collect::<String>());
    for (y, row) in grid.into_iter().enumerate() {
        println!("{y:2}  {}", row.into_iter().collect::<String>());
    }
    match sim::evaluate(lv, l) {
        Ok(s) => println!("ticks {} cost {} crew {}", s.ticks, s.cost, s.crew),
        Err(e) => println!("FAIL {e}"),
    }
}

/// Every top-left where a 2x2 building could go right now.
fn build_spots(lv: &Level, l: &Layout) -> Vec<(i32, i32)> {
    let w = World::new(lv, l);
    let mut v = Vec::new();
    for y in 0..lv.h - 2 {
        for x in 0..lv.w - 1 {
            if w.can_build(x, y).is_ok() {
                v.push((x, y));
            }
        }
    }
    v
}

fn infra_spots(lv: &Level, l: &Layout, i: Infra) -> Vec<(i32, i32)> {
    let w = World::new(lv, l);
    let mut v = Vec::new();
    for y in 0..lv.h {
        for x in 0..lv.w {
            let ok = match i {
                Infra::Spring => w.can_spring(x, y).is_ok(),
                Infra::Ladder => w.can_ladder(x, y).is_ok(),
                Infra::Plank => w.can_plank(x, y).is_ok(),
            };
            if ok {
                v.push((x, y));
            }
        }
    }
    v
}

/// Adds one leg with a species that can actually run it; false if none can.
fn add_leg(lv: &Level, l: &mut Layout, rng: &mut Rng, from: NodeRef, to: NodeRef, g: Good) -> bool {
    let w = World::new(lv, l);
    let mut sp = lv.species.clone();
    for i in (1..sp.len()).rev() {
        let j = rng.below(i + 1);
        sp.swap(i, j);
    }
    for s in sp {
        let r = crate::layout::Route { species: s, from, to, cargo: g, count: 1 };
        if sim::plan_route(lv, l, &w, &r).problem.is_none() {
            let n = 1 + rng.below(3) as i32;
            l.add_route(s, from, to, g, n);
            return true;
        }
    }
    false
}

fn chain(lv: &Level, l: &mut Layout, rng: &mut Rng, from: NodeRef, to: NodeRef, g: Good, max_depots: usize) -> bool {
    let mut nodes = vec![from];
    if lv.kinds.contains(&Kind::Depot) {
        let k = rng.below(max_depots + 1);
        for _ in 0..k {
            let spots = build_spots(lv, l);
            if let Some((x, y)) = rng.pick(&spots) {
                let id = l.add_building(Kind::Depot, x, y);
                nodes.push(NodeRef::Building(id));
            }
        }
    }
    nodes.push(to);
    nodes.windows(2).all(|p| add_leg(lv, l, rng, p[0], p[1], g))
}

fn deliver(lv: &Level, l: &mut Layout, rng: &mut Rng, g: Good, to: NodeRef, depth: usize) -> bool {
    if depth > 3 {
        return false;
    }
    let sources: Vec<usize> = lv.sources.iter().enumerate().filter(|(_, s)| s.good == g).map(|(i, _)| i).collect();
    if !sources.is_empty() {
        let n = 1 + rng.below(sources.len().min(2));
        let mut ok = false;
        for _ in 0..n {
            let si = rng.pick(&sources).unwrap();
            ok |= chain(lv, l, rng, NodeRef::Source(si), to, g, 2);
        }
        return ok;
    }
    let Some(&kind) = lv.kinds.iter().find(|k| k.recipe().is_some_and(|r| r.output == g)) else { return false };
    let existing: Vec<u32> = l.buildings.iter().filter(|b| b.kind == kind).map(|b| b.id).collect();
    let shop = if !existing.is_empty() && rng.chance(0.5) {
        rng.pick(&existing).unwrap()
    } else {
        let spots = build_spots(lv, l);
        let Some((x, y)) = rng.pick(&spots) else { return false };
        l.add_building(kind, x, y)
    };
    let r = kind.recipe().unwrap();
    for (input, _) in r.inputs {
        if !deliver(lv, l, rng, *input, NodeRef::Building(shop), depth + 1) {
            return false;
        }
    }
    chain(lv, l, rng, NodeRef::Building(shop), to, g, 1)
}

pub fn random_layout(lv: &Level, rng: &mut Rng) -> Layout {
    let mut l = Layout::default();
    for &i in &lv.infra {
        let n = rng.below(3);
        for _ in 0..n {
            let spots = infra_spots(lv, &l, i);
            if let Some((x, y)) = rng.pick(&spots) {
                match i {
                    Infra::Spring => l.springs.push((x, y)),
                    Infra::Ladder => l.ladders.push((x, y)),
                    Infra::Plank => l.planks.push((x, y)),
                }
            }
        }
    }
    for &(g, _) in &lv.need {
        deliver(lv, &mut l, rng, g, NodeRef::Fortress, 0);
    }
    l
}

fn mutate(lv: &Level, base: &Layout, rng: &mut Rng) -> Layout {
    let mut l = base.clone();
    let n = 1 + rng.below(2);
    for _ in 0..n {
        match rng.below(9) {
            0 | 1 => {
                if let Some(i) = (!l.routes.is_empty()).then(|| rng.below(l.routes.len())) {
                    if let Some(s) = rng.pick(&lv.species) {
                        l.routes[i].species = s;
                    }
                }
            }
            2 | 3 => {
                if let Some(i) = (!l.routes.is_empty()).then(|| rng.below(l.routes.len())) {
                    let d = if rng.chance(0.5) { 1 } else { -1 };
                    l.routes[i].count = (l.routes[i].count + d).clamp(1, 9);
                }
            }
            4 => {
                if !l.buildings.is_empty() {
                    let i = rng.below(l.buildings.len());
                    let b = l.buildings[i];
                    let mut tmp = l.clone();
                    tmp.buildings.remove(i);
                    let spots = build_spots(lv, &tmp);
                    let near: Vec<(i32, i32)> = spots.iter().copied().filter(|(x, y)| (x - b.x).abs() <= 4 && (y - b.y).abs() <= 4).collect();
                    let pick = if rng.chance(0.7) { rng.pick(&near) } else { rng.pick(&spots) };
                    if let Some((x, y)) = pick {
                        l.buildings[i].x = x;
                        l.buildings[i].y = y;
                    }
                }
            }
            5 => {
                if let Some(i) = rng.pick(&lv.infra) {
                    let spots = infra_spots(lv, &l, i);
                    if let Some(p) = rng.pick(&spots) {
                        match i {
                            Infra::Spring => l.springs.push(p),
                            Infra::Ladder => l.ladders.push(p),
                            Infra::Plank => l.planks.push(p),
                        }
                    }
                }
            }
            6 => {
                let total = l.springs.len() + l.ladders.len() + l.planks.len();
                if total > 0 {
                    let mut k = rng.below(total);
                    if k < l.springs.len() {
                        l.springs.remove(k);
                    } else {
                        k -= l.springs.len();
                        if k < l.ladders.len() {
                            l.ladders.remove(k);
                        } else {
                            l.planks.remove(k - l.ladders.len());
                        }
                    }
                }
            }
            7 => {
                // Pull from a different source of the same good.
                if let Some(i) = (!l.routes.is_empty()).then(|| rng.below(l.routes.len())) {
                    if let NodeRef::Source(_) = l.routes[i].from {
                        let g = l.routes[i].cargo;
                        let alts: Vec<usize> = lv.sources.iter().enumerate().filter(|(_, s)| s.good == g).map(|(k, _)| k).collect();
                        if let Some(k) = rng.pick(&alts) {
                            if rng.chance(0.5) {
                                l.routes[i].from = NodeRef::Source(k);
                            } else {
                                let mut r = l.routes[i];
                                r.from = NodeRef::Source(k);
                                r.count = 1;
                                l.routes.push(r);
                            }
                        }
                    }
                }
            }
            _ => {
                if l.routes.len() > 1 {
                    let i = rng.below(l.routes.len());
                    l.routes.remove(i);
                }
            }
        }
    }
    l
}

fn key(s: &Score, m: usize) -> (i32, i32, i32) {
    match m {
        0 => (s.ticks, s.cost, s.crew),
        1 => (s.cost, s.ticks, s.crew),
        _ => (s.crew, s.cost, s.ticks),
    }
}

pub struct SolveReport {
    pub best: [Option<(Score, Layout)>; 3],
    pub distinct: usize,
    pub valid: usize,
    pub tried: usize,
    pub all: Vec<(i32, i32, i32)>,
}

/// Random starts plus a hill-climb on each score.
pub fn solve(lv: &Level, starts: usize, climb: usize, seed: u64) -> SolveReport {
    let mut rng = Rng::new(seed);
    let mut best: [Option<(Score, Layout)>; 3] = [None, None, None];
    let mut seen: HashSet<String> = HashSet::new();
    let mut all = Vec::new();
    let mut valid = 0;
    let mut tried = 0;
    let mut pool: Vec<Layout> = Vec::new();
    if !lv.reference.is_empty() {
        pool.push(Layout::from_text(&lv.reference));
    }
    for _ in 0..starts {
        pool.push(random_layout(lv, &mut rng));
    }
    let mut record = |l: &Layout, s: &Score, best: &mut [Option<(Score, Layout)>; 3], all: &mut Vec<(i32, i32, i32)>| {
        let t = l.to_text();
        if seen.insert(t) {
            all.push((s.ticks, s.cost, s.crew));
        }
        for m in 0..3 {
            if best[m].as_ref().map_or(true, |(b, _)| key(s, m) < key(b, m)) {
                best[m] = Some((Score { ticks: s.ticks, cost: s.cost, crew: s.crew }, l.clone()));
            }
        }
    };
    for start in pool {
        tried += 1;
        let Ok(s0) = sim::evaluate(lv, &start) else { continue };
        valid += 1;
        record(&start, &s0, &mut best, &mut all);
        for m in 0..3 {
            let (mut cur, mut cs) = (start.clone(), Score { ticks: s0.ticks, cost: s0.cost, crew: s0.crew });
            for _ in 0..climb {
                let cand = mutate(lv, &cur, &mut rng);
                tried += 1;
                if let Ok(s) = sim::evaluate(lv, &cand) {
                    valid += 1;
                    record(&cand, &s, &mut best, &mut all);
                    if key(&s, m) <= key(&cs, m) {
                        cur = cand;
                        cs = s;
                    }
                }
            }
        }
    }
    SolveReport { best, distinct: all.len(), valid, tried, all }
}

pub fn cli(args: &[String], levels: &[Level]) -> bool {
    let arg = |name: &str| args.iter().position(|a| a == name);
    let find = |id: &str| levels.iter().find(|l| l.id == id);
    if let Some(i) = arg("--show") {
        let Some(lv) = args.get(i + 1).and_then(|id| find(id)) else { return true };
        let l = Layout::from_text(args.get(i + 2).map(|s| s.as_str()).unwrap_or(&lv.reference));
        show(lv, &l);
        return true;
    }
    if arg("--check").is_some() {
        for lv in levels {
            let l = Layout::from_text(&lv.reference);
            let t = lv.targets;
            match sim::evaluate(lv, &l) {
                Ok(s) => println!("{:5} {:22} ref ticks {:5} cost {:3} crew {:2}   goals {:5} {:3} {:2}", lv.id, lv.name, s.ticks, s.cost, s.crew, t.ticks, t.cost, t.crew),
                Err(e) => println!("{:5} {:22} FAIL {}", lv.id, lv.name, e),
            }
        }
        return true;
    }
    if let Some(i) = arg("--solve") {
        let which = args.get(i + 1).cloned().unwrap_or_else(|| "all".into());
        let starts: usize = args.get(i + 2).and_then(|s| s.parse().ok()).unwrap_or(60);
        let climb: usize = args.get(i + 3).and_then(|s| s.parse().ok()).unwrap_or(150);
        for lv in levels.iter().filter(|l| which == "all" || l.id == which) {
            let rep = solve(lv, starts, climb, 12345);
            let t = lv.targets;
            println!("== {} {}  (goals ticks {} cost {} crew {})  valid {}/{} distinct {}", lv.id, lv.name, t.ticks, t.cost, t.crew, rep.valid, rep.tried, rep.distinct);
            for (m, name) in ["TICKS", "COST", "CREW"].iter().enumerate() {
                if let Some((s, l)) = &rep.best[m] {
                    println!("  best {name:5}: ticks {:5} cost {:3} crew {:2} | {}", s.ticks, s.cost, s.crew, l.to_text());
                }
            }
            // Pareto front size over all distinct solutions.
            let front = rep.all.iter().filter(|a| !rep.all.iter().any(|b| b.0 <= a.0 && b.1 <= a.1 && b.2 <= a.2 && b != *a)).count();
            println!("  pareto front: {front}");
        }
        return true;
    }
    if let Some(i) = arg("--histo") {
        // Solves every level and writes the score histograms the results screen draws.
        let out = args.get(i + 1).cloned().unwrap_or_else(|| "src/levels/histo.txt".into());
        let starts: usize = args.get(i + 2).and_then(|s| s.parse().ok()).unwrap_or(80);
        let climb: usize = args.get(i + 3).and_then(|s| s.parse().ok()).unwrap_or(300);
        let prefix = args.get(i + 4).cloned().unwrap_or_default();
        let mut text = String::new();
        for lv in levels.iter().filter(|l| l.id.starts_with(&prefix)) {
            let rep = solve(lv, starts, climb, 777);
            for (m, name) in ["TICKS", "COST", "CREW"].iter().enumerate() {
                let vals: Vec<i32> = rep.all.iter().map(|a| [a.0, a.1, a.2][m]).collect();
                let (lo, hi) = (vals.iter().copied().min().unwrap_or(0), vals.iter().copied().max().unwrap_or(0));
                let mut bins = [0u32; HISTO_BINS];
                for v in &vals {
                    bins[histo_bin(*v, lo, hi)] += 1;
                }
                let b: Vec<String> = bins.iter().map(|n| n.to_string()).collect();
                text.push_str(&format!("histo {} {} {} {} {}\n", lv.id, name, lo, hi, b.join(" ")));
            }
            // The ticks-cost frontier, for choosing the combo goal.
            let mut front: Vec<(i32, i32)> = rep.all.iter().map(|a| (a.0, a.1)).collect();
            front.sort();
            front.dedup();
            let front: Vec<(i32, i32)> = front.iter().copied().filter(|&(t, c)| !rep.all.iter().any(|a| a.0 <= t && a.1 <= c && (a.0, a.1) != (t, c))).collect();
            for (t, c) in &front {
                text.push_str(&format!("front {} {} {}\n", lv.id, t, c));
            }
            let b = |m: usize| rep.best[m].as_ref().map(|(s, _)| [s.ticks, s.cost, s.crew][m]).unwrap_or(0);
            text.push_str(&format!("best {} {} {} {}\n", lv.id, b(0), b(1), b(2)));
            for m in 0..3 {
                if let Some((s, l)) = &rep.best[m] {
                    text.push_str(&format!("# {} best {}: {} {} {} | {}\n", lv.id, ["TICKS", "COST", "CREW"][m], s.ticks, s.cost, s.crew, l.to_text()));
                }
            }
            println!("{} done: {} distinct", lv.id, rep.distinct);
        }
        let _ = std::fs::write(&out, text);
        return true;
    }
    if let Some(i) = arg("--shots") {
        let dir = args.get(i + 1).cloned().unwrap_or_else(|| "screenshots".into());
        crate::harness::shots(levels, &dir);
        return true;
    }
    false
}

/// Renders a set of screens to PNGs for review.
pub fn shots(levels: &[Level], dir: &str) {
    use crate::app::{App, Ev, Key};
    use crate::framebuffer::Framebuffer;
    let _ = std::fs::create_dir_all(dir);
    let save = crate::save::Save::load(None);
    let log = crate::save::Log::open(None);
    let mut g = crate::game::Game::new(levels.to_vec(), save, log, None);
    g.unlock_all = true;
    let mut fb = Framebuffer::new();
    let mut snap = |g: &mut crate::game::Game, name: &str| {
        g.tick();
        g.draw(&mut fb);
        fb.save_png(&std::path::Path::new(dir).join(format!("{name}.png")), g.palette());
    };
    snap(&mut g, "00_title");
    g.event(Ev::Key(Key::Enter));
    snap(&mut g, "01_select");
    // The guided first site, and the picture cards over tool buttons.
    g.enter_level(0);
    g.overlay = crate::game::Overlay::None;
    g.mouse = (200, 100);
    snap(&mut g, "01b_coach");
    if let Some(i) = levels.iter().position(|l| l.id == "2-4") {
        g.enter_level(i);
        g.overlay = crate::game::Overlay::None;
        for (k, name) in [(2usize, "01c_card_dodo"), (5, "01d_card_mason"), (7, "01e_card_ladder")] {
            let r = crate::game::tool_rect(k);
            g.mouse = (r.x + 5, r.y + 5);
            snap(&mut g, name);
        }
    }
    let at = |id: &str| levels.iter().position(|l| l.id == id).unwrap_or(0);
    for (k, idx) in [at("1-1"), at("1-4"), at("2-2"), at("3-2"), at("4-3"), at("5-1"), at("5-2")].iter().enumerate() {
        if *idx >= levels.len() {
            continue;
        }
        g.enter_level(*idx);
        snap(&mut g, &format!("{:02}_{}_rule", 2 + k * 3, levels[*idx].id));
        g.event(Ev::Key(Key::Space));
        if k == 1 {
            // The reach marks: a dodo picking up at the twig pile.
            g.layout = Layout::default();
            g.rebuild();
            g.tool = crate::game::Tool::Species(Species::Dodo);
            g.from = Some(NodeRef::Source(0));
            g.mouse = (200, 100);
            snap(&mut g, &format!("{:02}_{}_reach", 2 + k * 3, levels[*idx].id));
            g.tool = crate::game::Tool::Select;
            g.from = None;
        }
        g.layout = Layout::from_text(&levels[*idx].reference);
        g.rebuild();
        if let Some(r) = g.layout.routes.first() {
            g.selected = Some(r.from);
        }
        snap(&mut g, &format!("{:02}_{}_plan", 3 + k * 3, levels[*idx].id));
        g.event(Ev::Key(Key::Space));
        for _ in 0..40 {
            g.tick();
        }
        snap(&mut g, &format!("{:02}_{}_run", 4 + k * 3, levels[*idx].id));
        if k == 1 || k == 4 || k == 5 {
            g.speed = 4;
            for _ in 0..2000 {
                if !g.running {
                    break;
                }
                g.tick();
            }
            snap(&mut g, &format!("{:02}_{}_result", 4 + k * 3, levels[*idx].id));
        }
    }
    // The fortress going up, then finished.
    g.enter_level(0);
    g.overlay = crate::game::Overlay::None;
    g.layout = Layout::from_text("R TRIBBLE s0 F TWIG 3");
    g.rebuild();
    g.speed = 0;
    g.event(Ev::Key(Key::Space));
    for _ in 0..400 {
        g.tick();
        if g.sim.as_ref().is_some_and(|s| s.delivered_total() >= 2) {
            break;
        }
    }
    g.running = false;
    snap(&mut g, "30_fortress_rising");
    g.running = true;
    for _ in 0..400 {
        if !g.running {
            break;
        }
        g.tick();
    }
    g.overlay = crate::game::Overlay::None;
    snap(&mut g, "31_fortress_done");
    g.event(Ev::Key(Key::Help));
    for p in 0..crate::game::HELP_PAGES {
        g.help_page = p;
        snap(&mut g, &format!("90_help_{}", p + 1));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_level_parses_and_its_reference_solves() {
        let levels = crate::level::parse_book(crate::BOOK);
        assert_eq!(levels.len(), 22);
        for lv in &levels {
            assert!(lv.fortress.2 > 0, "{} has no fortress", lv.id);
            assert!(!lv.rule.is_empty(), "{} has no rule card", lv.id);
            let l = Layout::from_text(&lv.reference);
            let s = sim::evaluate(lv, &l).unwrap_or_else(|e| panic!("{} reference fails: {e}", lv.id));
            assert!(s.cost >= lv.targets.cost && s.crew >= lv.targets.crew, "{} goals are looser than its reference", lv.id);
            // Every building in a reference must be legal to place.
            let mut placed = Layout::default();
            for b in &l.buildings {
                let w = World::new(lv, &placed);
                assert!(w.can_build(b.x, b.y).is_ok(), "{} reference building at {},{} is illegal", lv.id, b.x, b.y);
                placed.add_building(b.kind, b.x, b.y);
            }
        }
    }

    #[test]
    fn sources_sit_on_floors() {
        for lv in crate::level::parse_book(crate::BOOK) {
            let w = World::new(&lv, &Layout::default());
            for s in &lv.sources {
                assert!(w.standable(s.x, s.y), "{} source at {},{} floats", lv.id, s.x, s.y);
            }
            let (fx, fy, fw, fh) = lv.fortress;
            for x in fx..fx + fw {
                assert!(w.floor_below(x, fy + fh - 1), "{} fortress floats at x {x}", lv.id);
            }
        }
    }
}
