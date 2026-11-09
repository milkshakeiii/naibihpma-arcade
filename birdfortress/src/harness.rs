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
