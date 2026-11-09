//! The balance bot: a build plan is an ordered list of steps, each done as soon as it can be
//! afforded (or its wait is over). Plans live in the site book and check that every site can be
//! finished; the optimiser reorders them to set the medal days.

use crate::city::{Action, City, Order};
use crate::level::Level;
use crate::rules::*;

#[derive(Clone, Debug)]
pub enum Step {
    Do(Action),
    /// A store order, found by the store's top-left tile.
    Order(i32, i32, Good, Order),
    /// Change who a workplace hires (by its tile).
    Hire(i32, i32, Species),
    WaitDay(i32),
    WaitPop(i32),
    WaitSeeds(i32),
}

impl Step {
    pub fn text(&self) -> String {
        match self {
            Step::Do(a) => a.text().to_lowercase(),
            Step::Order(x, y, g, o) => format!("{} {x} {y} {}", match o {
                Order::Get => "get",
                Order::Refuse => "refuse",
                Order::Accept => "accept",
            }, g.word()),
            Step::Hire(x, y, s) => format!("hire {x} {y} {}", s.name().to_lowercase()),
            Step::WaitDay(d) => format!("wait day {d}"),
            Step::WaitPop(p) => format!("wait pop {p}"),
            Step::WaitSeeds(p) => format!("wait seeds {p}"),
        }
    }
}

pub fn parse_plan(text: &str) -> Result<Vec<Step>, String> {
    let mut out = Vec::new();
    for raw in text.split(';') {
        let t: Vec<&str> = raw.split_whitespace().collect();
        if t.is_empty() {
            continue;
        }
        let num = |i: usize| -> Result<i32, String> { t.get(i).and_then(|s| s.parse().ok()).ok_or_else(|| format!("bad number in '{raw}'")) };
        let word = t[0].to_ascii_uppercase();
        if word == "WAIT" {
            let what = t.get(1).copied().unwrap_or("");
            let n = num(2)?;
            out.push(match what {
                "day" => Step::WaitDay(n),
                "pop" => Step::WaitPop(n),
                "seeds" => Step::WaitSeeds(n),
                _ => return Err(format!("bad wait '{raw}'")),
            });
            continue;
        }
        if word == "GET" || word == "REFUSE" || word == "ACCEPT" {
            let g = t.get(3).and_then(|s| Good::from_name(s)).ok_or_else(|| format!("bad good in '{raw}'"))?;
            let o = match word.as_str() {
                "GET" => Order::Get,
                "REFUSE" => Order::Refuse,
                _ => Order::Accept,
            };
            out.push(Step::Order(num(1)?, num(2)?, g, o));
            continue;
        }
        if word == "HIRE" {
            let s = t.get(3).and_then(|s| Species::from_name(s)).ok_or_else(|| format!("bad species in '{raw}'"))?;
            out.push(Step::Hire(num(1)?, num(2)?, s));
            continue;
        }
        if word == "DEMOLISH" {
            out.push(Step::Do(Action::Demolish { x: num(1)?, y: num(2)? }));
            continue;
        }
        if let Some(i) = Infra::from_name(&word) {
            out.push(Step::Do(Action::Infra { infra: i, x: num(1)?, y: num(2)? }));
            continue;
        }
        if let Some(k) = Kind::from_name(&word) {
            let species = t.get(3).and_then(|s| Species::from_name(s)).unwrap_or(Species::Tribble);
            out.push(Step::Do(Action::Build { kind: k, x: num(1)?, y: num(2)?, species }));
            continue;
        }
        return Err(format!("unknown step '{raw}'"));
    }
    Ok(out)
}

pub fn plan_text(steps: &[Step]) -> String {
    steps.iter().map(|s| s.text()).collect::<Vec<_>>().join("; ")
}

#[derive(Clone, Debug)]
pub struct RunResult {
    /// The day every goal was met, if it was.
    pub won: Option<i32>,
    pub day: i32,
    pub pop: i32,
    pub stage: usize,
    pub error: Option<String>,
    pub slipped: i32,
    pub broke_challenge: bool,
    pub log: Vec<String>,
}

/// Runs a plan for up to `max_days`. With `verbose`, logs a line a day.
pub fn run(level: &Level, steps: &[Step], max_days: i32, verbose: bool) -> RunResult {
    let mut city = City::new(level);
    let mut next = 0usize;
    let mut log = Vec::new();
    let mut error = None;
    let limit = max_days * DAY;
    while city.tick < limit && city.won.is_none() {
        // Do as many steps as can be done right now.
        while next < steps.len() {
            let ok = match &steps[next] {
                Step::WaitDay(d) => city.day() >= *d,
                Step::WaitPop(p) => city.pop() >= *p,
                Step::WaitSeeds(s) => city.seeds() >= *s,
                Step::Order(x, y, g, o) => {
                    match city.building_at(*x, *y).map(|b| b.id) {
                        Some(id) => {
                            let _ = city.apply(&Action::SetOrder { id, good: *g, order: *o });
                        }
                        None => error = Some(format!("step {}: no store at {x} {y}", next + 1)),
                    }
                    true
                }
                Step::Hire(x, y, s) => {
                    match city.building_at(*x, *y).map(|b| b.id) {
                        Some(id) => {
                            let _ = city.apply(&Action::SetSpecies { id, species: *s });
                        }
                        None => error = Some(format!("step {}: nothing at {x} {y}", next + 1)),
                    }
                    true
                }
                Step::Do(a) => match city.check(a) {
                    Ok(_) => {
                        city.apply(a).ok();
                        if verbose {
                            log.push(format!("  d{:>3} {}", city.day(), a.text().to_lowercase()));
                        }
                        true
                    }
                    Err(e) if e.starts_with("It costs") => false,
                    Err(e) => {
                        error = Some(format!("step {} ({}): {e}", next + 1, a.text().to_lowercase()));
                        true
                    }
                },
            };
            if error.is_some() {
                break;
            }
            if !ok {
                break;
            }
            next += 1;
        }
        if error.is_some() {
            break;
        }
        city.step();
        city.events.clear();
        if verbose && city.tick % DAY == 0 {
            log.push(day_line(&city));
        }
    }
    RunResult {
        won: city.won,
        day: city.day(),
        pop: city.pop(),
        stage: city.stage,
        error,
        slipped: city.slipped,
        broke_challenge: city.broke_challenge,
        log,
    }
}

pub fn day_line(c: &City) -> String {
    let tiers: Vec<String> = (0..TIERS).map(|t| c.buildings.iter().filter(|b| b.kind.is_home() && b.tier == t && b.residents > 0).count().to_string()).collect();
    let (got, total) = c.fortress_progress();
    let work: Vec<String> = SPECIES.iter().map(|&s| format!("{}{}/{}", &s.name()[..1], c.employed(s), c.pop_of(s))).collect();
    let notes: Vec<String> = c
        .buildings
        .iter()
        .filter(|b| b.note != crate::city::Note::Fine)
        .map(|b| format!("{}@{},{}:{:?}", b.kind.name().to_lowercase(), b.x, b.y, b.note))
        .take(6)
        .collect();
    format!(
        "day {:>3} {:<6} seeds {:>4} pop {:>3} [{}] tiers {} fort {}/{} st{} | {}",
        c.day(),
        SEASONS[c.season()],
        c.seeds(),
        c.pop(),
        work.join(" "),
        tiers.join("/"),
        got,
        total,
        c.stage,
        notes.join(" ")
    )
}
