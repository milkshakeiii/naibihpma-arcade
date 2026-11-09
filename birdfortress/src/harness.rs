//! Headless tools: run a site's bot plan with a day-by-day log, check every site, print a map
//! with coordinates, optimise plan order, and export screenshots.

use crate::bot;
use crate::level::Level;

pub fn find<'a>(levels: &'a [Level], id: &str) -> Option<&'a Level> {
    levels.iter().find(|l| l.id == id)
}

/// Returns true if a harness command ran (and the game shouldn't open).
pub fn cli(args: &[String], levels: &[Level]) -> bool {
    let arg = |i: usize| args.get(i).map(|s| s.as_str()).unwrap_or("");
    match arg(1) {
        "--plan" => {
            let Some(lv) = find(levels, arg(2)) else {
                println!("no site {}", arg(2));
                return true;
            };
            let idx: usize = arg(3).parse().unwrap_or(0);
            let days: i32 = arg(4).parse().unwrap_or(200);
            let Some(text) = lv.plans.get(idx) else {
                println!("{} has no plan {idx}", lv.id);
                return true;
            };
            match bot::parse_plan(text) {
                Ok(steps) => {
                    let r = bot::run(lv, &steps, days, true);
                    for l in &r.log {
                        println!("{l}");
                    }
                    if let Some(e) = &r.error {
                        println!("ERROR {e}");
                    }
                    println!("{} plan {idx}: {}", lv.id, match r.won {
                        Some(d) => format!("WON day {d}"),
                        None => format!("unfinished at day {} (pop {}, stage {})", r.day, r.pop, r.stage),
                    });
                }
                Err(e) => println!("plan error: {e}"),
            }
            true
        }
        "--check" => {
            for lv in levels {
                let mut line = format!("{:<5} {:<18} {}x{}", lv.id, lv.name, lv.w, lv.h);
                for (i, p) in lv.plans.iter().enumerate() {
                    match bot::parse_plan(p) {
                        Ok(steps) => {
                            let r = bot::run(lv, &steps, 400, false);
                            line += &match (&r.error, r.won) {
                                (Some(e), _) => format!(" | plan{i} ERROR {e}"),
                                (None, Some(d)) => format!(" | plan{i} day {d}{}", if r.slipped > 0 { format!(" ({} slips)", r.slipped) } else { String::new() }),
                                (None, None) => format!(" | plan{i} unfinished (pop {} stage {})", r.pop, r.stage),
                            };
                        }
                        Err(e) => line += &format!(" | plan{i} PARSE {e}"),
                    }
                }
                line += &format!(" | medals {:?}", lv.medals);
                println!("{line}");
            }
            true
        }
        "--shot" => {
            let days: i32 = arg(3).parse().unwrap_or(10);
            let cam = if !arg(5).is_empty() { Some((arg(5).parse().unwrap_or(0), arg(6).parse().unwrap_or(0))) } else { None };
            let sel = if !arg(7).is_empty() { Some((arg(7).parse().unwrap_or(0), arg(8).parse().unwrap_or(0))) } else { None };
            shot(levels, arg(2), days, arg(4), cam, sel);
            true
        }
        "--dump" => {
            let Some(lv) = find(levels, arg(2)) else { return true };
            let days: i32 = arg(3).parse().unwrap_or(10);
            let steps = bot::parse_plan(&lv.plans[0]).unwrap_or_default();
            let mut g = crate::game::Game::new(levels.to_vec(), crate::save::Save::load(None), crate::save::Log::open(None), None);
            let i = levels.iter().position(|l| l.id == lv.id).unwrap();
            g.enter_level(i);
            g.begin(false);
            drive(&mut g, &steps, days);
            println!("{}", bot::day_line(&g.city));
            for b in &g.city.buildings {
                let stock: Vec<String> = crate::rules::GOODS.iter().filter(|gd| b.stock[gd.index()] > 0 || b.incoming[gd.index()] > 0).map(|gd| format!("{}{}+{}", gd.word(), b.stock[gd.index()], b.incoming[gd.index()])).collect();
                println!(
                    "#{:<3} {:<7} {:>3},{:<3} {:?} tier{} res{} work{}/{} {:?} [{}]",
                    b.id,
                    b.kind.name(),
                    b.x,
                    b.y,
                    b.species,
                    b.tier,
                    b.residents,
                    g.city.workers_at(b.id),
                    g.city.slots(b),
                    b.note,
                    stock.join(" ")
                );
            }
            for c in &g.city.creatures {
                println!("  {:?} base#{} at {},{} {:?} cargo {:?}", c.species, c.base, c.x, c.y, c.job, c.cargo);
            }
            true
        }
        "--opt" => {
            let Some(lv) = find(levels, arg(2)) else { return true };
            let iters: usize = arg(3).parse().unwrap_or(200);
            let seed: u64 = arg(4).parse().unwrap_or(7);
            let steps = bot::parse_plan(&lv.plans[0]).unwrap_or_default();
            let before = bot::run(lv, &steps, 400, false).won;
            let (day, best) = bot::optimize(lv, &steps, iters, seed);
            println!("{} before {:?} after day {day}", lv.id, before);
            println!("PLAN {} {}", lv.id, bot::plan_text(&best));
            true
        }
        "--shots" => {
            shots(levels, if arg(2).is_empty() { "screenshots" } else { arg(2) });
            true
        }
        "--spread" => {
            let n: usize = arg(2).parse().unwrap_or(40);
            for lv in levels {
                let steps = bot::parse_plan(&lv.plans[0]).unwrap_or_default();
                let best = bot::run(lv, &steps, 400, false).won.unwrap_or(0);
                let (days, lost) = bot::spread(lv, &steps, n, 5);
                let q = |f: f32| days.get(((days.len() as f32 - 1.0) * f).round() as usize).copied().unwrap_or(0);
                println!(
                    "{:<5} best {:>3} | random orders: min {:>3} median {:>3} max {:>3} | {:>2} of {n} never finish",
                    lv.id,
                    best,
                    q(0.0),
                    q(0.5),
                    q(1.0),
                    lost
                );
            }
            true
        }
        "--map" => {
            let Some(lv) = find(levels, arg(2)) else { return true };
            print_map(lv);
            true
        }
        _ => false,
    }
}

pub fn print_map(lv: &Level) {
    let mut header = String::from("    ");
    for x in 0..lv.w {
        header.push(char::from(b'0' + (x % 10) as u8));
    }
    println!("{header}");
    for y in 0..lv.h {
        let mut row = format!("{y:>3} ");
        for x in 0..lv.w {
            use crate::level::Tile::*;
            let c = if let Some(s) = lv.sources.iter().find(|s| s.x == x && s.y == y) {
                match s.good {
                    crate::rules::Good::Berry => 'b',
                    crate::rules::Good::Nut => 'n',
                    crate::rules::Good::Fish => 'f',
                    crate::rules::Good::Twig => 't',
                    crate::rules::Good::Stone => 'q',
                    crate::rules::Good::Clay => 'c',
                    _ => '?',
                }
            } else if lv.entrance == (x, y) {
                'E'
            } else {
                let (fx, fy, fw, fh) = lv.fortress;
                if x >= fx && x < fx + fw && y >= fy && y < fy + fh {
                    'F'
                } else {
                    match lv.tile(x, y) {
                        Air => '.',
                        Earth => '#',
                        Rock => '%',
                        Bramble => 'X',
                        Branch => '=',
                        Ledge => '-',
                        Trunk => '|',
                        Leaves => ':',
                        Vine => 'v',
                        Water => '~',
                        Ladder => 'H',
                        Cloud => '@',
                    }
                }
            };
            row.push(c);
        }
        println!("{row}");
    }
}

/// Runs a plan inside a real game (so the screen shows it), for `days` days.
pub fn drive(g: &mut crate::game::Game, steps: &[bot::Step], days: i32) {
    use crate::city::Action;
    let mut next = 0usize;
    let limit = days * crate::rules::DAY;
    while g.city.tick < limit && g.city.won.is_none() {
        while next < steps.len() {
            let ok = match &steps[next] {
                bot::Step::WaitDay(d) => g.city.day() >= *d,
                bot::Step::WaitPop(p) => g.city.pop() >= *p,
                bot::Step::WaitSeeds(s) => g.city.seeds() >= *s,
                bot::Step::Order(x, y, gd, o) => {
                    if let Some(id) = g.city.building_at(*x, *y).map(|b| b.id) {
                        g.act(Action::SetOrder { id, good: *gd, order: *o });
                    }
                    true
                }
                bot::Step::Hire(x, y, s) => {
                    if let Some(id) = g.city.building_at(*x, *y).map(|b| b.id) {
                        g.act(Action::SetSpecies { id, species: *s });
                    }
                    true
                }
                bot::Step::Do(a) => match g.city.check(a) {
                    Ok(_) => {
                        g.act(a.clone());
                        true
                    }
                    Err(e) if e.starts_with("It costs") => false,
                    Err(_) => true,
                },
            };
            if !ok {
                break;
            }
            next += 1;
        }
        g.advance(1);
    }
}

/// One screenshot of a site after its first plan has run for some days.
pub fn shot(levels: &[Level], id: &str, days: i32, out: &str, cam: Option<(i32, i32)>, sel: Option<(i32, i32)>) {
    use crate::app::App;
    use crate::framebuffer::Framebuffer;
    let Some(i) = levels.iter().position(|l| l.id == id) else { return };
    let mut g = crate::game::Game::new(levels.to_vec(), crate::save::Save::load(None), crate::save::Log::open(None), None);
    g.unlock_all = true;
    g.enter_level(i);
    g.begin(false);
    if let Some(p) = levels[i].plans.first() {
        if let Ok(steps) = bot::parse_plan(p) {
            drive(&mut g, &steps, days);
        }
    }
    g.tip = None;
    g.paused = true;
    if let Some(c) = cam {
        g.cam = c;
        g.clamp_cam();
    }
    if let Some((x, y)) = sel {
        if let Some(b) = g.city.building_at(x, y) {
            g.sel = Some(crate::game::Sel::B(b.id));
        } else if let Some(s) = g.level().source_at(x, y) {
            g.sel = Some(crate::game::Sel::Source(s));
        }
    }
    g.mouse = (200, 120);
    g.tick();
    let mut fb = Framebuffer::new();
    g.draw(&mut fb);
    fb.save_png(std::path::Path::new(out), g.palette());
    println!("{}", bot::day_line(&g.city));
}

/// The screenshot set for the README and for looking over the presentation.
pub fn shots(levels: &[Level], dir: &str) {
    use crate::app::{App, Ev, Key};
    use crate::framebuffer::Framebuffer;
    let _ = std::fs::create_dir_all(dir);
    let new_game = || {
        let mut g = crate::game::Game::new(levels.to_vec(), crate::save::Save::load(None), crate::save::Log::open(None), None);
        g.unlock_all = true;
        g
    };
    let mut fb = Framebuffer::new();
    let mut snap = |g: &mut crate::game::Game, name: &str| {
        g.tick();
        g.draw(&mut fb);
        fb.save_png(&std::path::Path::new(dir).join(format!("{name}.png")), g.palette());
    };
    let at = |id: &str| levels.iter().position(|l| l.id == id).unwrap_or(0);
    let mut g = new_game();
    snap(&mut g, "00_title");
    g.event(Ev::Key(Key::Enter));
    g.mouse = (200, 150);
    snap(&mut g, "01_select");
    // The first site: its card, then a moment of play with a burrow being placed.
    g.enter_level(at("1-1"));
    snap(&mut g, "02_1-1_card");
    g.begin(false);
    let steps = crate::bot::parse_plan(&levels[at("1-1")].plans[0]).unwrap_or_default();
    drive(&mut g, &steps, 2);
    g.tip = None;
    g.tool = crate::game::Tool::Build(crate::rules::Kind::Burrow);
    g.mouse = (200, 165);
    g.paused = false;
    snap(&mut g, "03_1-1_play");
    // Mid-game views of bigger sites, each with something selected.
    let views: [(&str, i32, (i32, i32), (i32, i32), &str); 7] = [
        ("1-2", 12, (0, 0), (30, 16), "04_1-2_store"),
        ("2-1", 22, (300, 0), (68, 18), "05_2-1_mesa"),
        ("2-2", 26, (440, 0), (82, 18), "06_2-2_relay"),
        ("2-3", 22, (240, 0), (52, 16), "07_2-3_kiln"),
        ("3-2", 26, (0, 80), (12, 11), "08_3-2_canopy"),
        ("4-1", 24, (0, 0), (6, 17), "09_4-1_manors"),
        ("5-2", 50, (580, 0), (88, 12), "10_5-2_skyreach"),
    ];
    for (id, days, cam, sel, name) in views {
        let mut g = new_game();
        g.enter_level(at(id));
        g.begin(false);
        let steps = crate::bot::parse_plan(&levels[at(id)].plans[0]).unwrap_or_default();
        drive(&mut g, &steps, days);
        g.tip = None;
        g.msg = None;
        g.floaters.clear();
        g.overlay = crate::game::Overlay::None;
        g.paused = true;
        g.cam = cam;
        g.clamp_cam();
        if let Some(b) = g.city.building_at(sel.0, sel.1) {
            g.sel = Some(crate::game::Sel::B(b.id));
        } else if let Some(s) = g.level().source_at(sel.0, sel.1) {
            g.sel = Some(crate::game::Sel::Source(s));
        }
        g.mouse = (390, 120);
        snap(&mut g, name);
    }
    // A workplace about to be placed: the reach dots.
    let mut g = new_game();
    g.enter_level(at("2-1"));
    g.begin(false);
    let steps = crate::bot::parse_plan(&levels[at("2-1")].plans[0]).unwrap_or_default();
    drive(&mut g, &steps, 20);
    g.tip = None;
    g.msg = None;
    g.floaters.clear();
    g.city.seeds_tenths = 400;
    g.paused = true;
    g.cam = (0, 0);
    g.clamp_cam();
    g.tool = crate::game::Tool::Build(crate::rules::Kind::Weaver);
    g.hire = crate::rules::Species::Dodo;
    g.mouse = (275, 195);
    snap(&mut g, "11_2-1_reach");
    // A finished site.
    let mut g = new_game();
    g.enter_level(at("1-1"));
    g.begin(false);
    let steps = crate::bot::parse_plan(&levels[at("1-1")].plans[0]).unwrap_or_default();
    drive(&mut g, &steps, 60);
    g.mouse = (200, 100);
    snap(&mut g, "12_results");
    // A goal met with one still to go: the banner says what's left.
    let mut g = new_game();
    g.enter_level(at("1-1"));
    g.begin(false);
    g.act(crate::city::Action::Build { kind: crate::rules::Kind::Burrow, x: 2, y: 16, species: crate::rules::Species::Tribble });
    if let Some(t) = levels[at("1-1")].sources.iter().find(|s| s.good == crate::rules::Good::Twig) {
        g.act(crate::city::Action::Build { kind: crate::rules::Kind::Camp, x: t.x, y: t.y, species: crate::rules::Species::Tribble });
    }
    g.speed = 3;
    for _ in 0..3000 {
        g.tick();
        if g.banner.is_some() {
            break;
        }
    }
    g.tip = None;
    g.msg = None;
    g.mouse = (200, 100);
    snap(&mut g, "13_goal_banner");
    // The site list with a finished site.
    let mut g = new_game();
    let rec = g.save.rec_mut("1-1");
    rec.done = true;
    rec.best_day = Some(8);
    g.event(Ev::Key(Key::Enter));
    g.mouse = (200, 200);
    snap(&mut g, "14_select_done");
    // The rules pages.
    let mut g = new_game();
    g.event(Ev::Key(Key::Help));
    for p in 0..crate::game::HELP_PAGES {
        g.help_page = p;
        snap(&mut g, &format!("90_help_{}", p + 1));
    }
}

#[cfg(test)]
mod tests {
    use crate::level::parse_book;

    #[test]
    fn every_site_parses_and_its_plan_wins() {
        let levels = parse_book(crate::BOOK);
        assert_eq!(levels.len(), 12);
        for lv in &levels {
            assert!(!lv.plans.is_empty(), "{} has no plan", lv.id);
            let steps = crate::bot::parse_plan(&lv.plans[0]).unwrap_or_else(|e| panic!("{}: {e}", lv.id));
            let r = crate::bot::run(lv, &steps, 300, false);
            assert!(r.error.is_none(), "{}: {:?}", lv.id, r.error);
            let day = r.won.unwrap_or_else(|| panic!("{} plan never wins (pop {}, stage {})", lv.id, r.pop, r.stage));
            if lv.medals[2] > 0 {
                assert!(day <= lv.medals[2], "{} plan finishes day {day}, after bronze {}", lv.id, lv.medals[2]);
            }
        }
    }

    #[test]
    fn sources_entrances_and_fortresses_stand_on_floors() {
        for lv in parse_book(crate::BOOK) {
            let w = crate::world::World::new(&lv);
            for s in &lv.sources {
                assert!(w.standable(s.x, s.y), "{}: source at {},{} floats", lv.id, s.x, s.y);
            }
            let (ex, ey) = lv.entrance;
            assert!(w.standable(ex, ey), "{}: the trail at {ex},{ey} floats", lv.id);
            let (fx, fy, fw, fh) = lv.fortress;
            for x in fx..fx + fw {
                assert!(w.standable(x, fy + fh - 1), "{}: the fortress floats at {x}", lv.id);
            }
        }
    }
}
