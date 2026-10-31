//! Worlds saved as JSON under `%APPDATA%\MonsterWorkshop\worlds\`. Only simulation state is
//! saved; the base map comes from the game's data. A save is written to a temporary file and
//! renamed into place, and the two previous saves are kept as backups.
//!
//! While the game is in development there are no migrations: a save from another version
//! doesn't load, and the title screen says so.

use crate::econ::Rng;
use crate::game::{Applicant, Game, Monster};
use crate::tutorial::Tutorial;
use crate::world::{Bin, Board, Ferry, Item, Plant, Shop};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

/// Bump whenever the save format or the map changes.
pub const VERSION: u32 = 3;
/// Autosave every minute of play (15 ticks a second).
pub const AUTOSAVE_TICKS: u32 = 900;

/// What a save holds, borrowed from a running game.
#[derive(Serialize)]
struct SaveRef<'a> {
    version: u32,
    frame: u32,
    rng: &'a Rng,
    renown: f64,
    controlled: usize,
    tutorial: &'a Tutorial,
    made: Vec<&'a String>,
    examine_hinted: bool,
    items: &'a [Item],
    plants: &'a [Plant],
    bins: &'a [Bin],
    shops: &'a [Shop],
    boards: &'a [Board],
    ferry: &'a Option<Ferry>,
    monsters: &'a [Monster],
    applicants: &'a [Option<Applicant>],
    applicant_timers: &'a [u32],
    roster_timer: u32,
    next_id: u32,
}

/// Read first, so a save from another version is reported as such rather than as unreadable.
#[derive(Deserialize)]
struct Header {
    version: u32,
}

/// The same fields, owned, when loading.
#[derive(Deserialize)]
struct SaveData {
    frame: u32,
    rng: Rng,
    renown: f64,
    controlled: usize,
    tutorial: Tutorial,
    made: Vec<String>,
    examine_hinted: bool,
    items: Vec<Item>,
    plants: Vec<Plant>,
    bins: Vec<Bin>,
    shops: Vec<Shop>,
    boards: Vec<Board>,
    ferry: Option<Ferry>,
    monsters: Vec<Monster>,
    applicants: Vec<Option<Applicant>>,
    applicant_timers: Vec<u32>,
    roster_timer: u32,
    next_id: u32,
}

pub fn to_json(g: &Game) -> String {
    let save = SaveRef {
        version: VERSION,
        frame: g.frame,
        rng: &g.rng,
        renown: g.renown,
        controlled: g.controlled,
        tutorial: &g.tutorial,
        made: g.made.iter().collect(),
        examine_hinted: g.examine_hinted,
        items: &g.world.items,
        plants: &g.world.plants,
        bins: &g.world.bins,
        shops: &g.world.shops,
        boards: &g.world.boards,
        ferry: &g.world.ferry,
        monsters: &g.monsters,
        applicants: &g.applicants,
        applicant_timers: &g.applicant_timers,
        roster_timer: g.roster_timer,
        next_id: g.next_id,
    };
    serde_json::to_string(&save).expect("world state serializes")
}

pub fn from_json(json: &str) -> Result<Game, String> {
    let header: Header = serde_json::from_str(json).map_err(|e| format!("UNREADABLE SAVE: {e}"))?;
    if header.version != VERSION {
        return Err("SAVED BY ANOTHER VERSION OF THE GAME. START A NEW WORLD.".into());
    }
    let d: SaveData = serde_json::from_str(json).map_err(|e| format!("UNREADABLE SAVE: {e}"))?;
    let mut g = Game::new();
    let w = &g.world;
    if d.plants.len() != w.plants.len() || d.bins.len() != w.bins.len() || d.shops.len() != w.shops.len() || d.boards.len() != w.boards.len() {
        return Err("THE SAVE WAS MADE ON A DIFFERENT MAP.".into());
    }
    g.frame = d.frame;
    g.rng = d.rng;
    g.renown = d.renown;
    g.tutorial = d.tutorial;
    g.made = d.made.into_iter().collect();
    g.examine_hinted = d.examine_hinted;
    g.world.items = d.items;
    g.world.plants = d.plants;
    g.world.bins = d.bins;
    g.world.shops = d.shops;
    g.world.boards = d.boards;
    g.world.ferry = d.ferry;
    g.monsters = d.monsters;
    g.controlled = d.controlled.min(g.monsters.len().saturating_sub(1));
    g.applicants = d.applicants;
    g.applicant_timers = d.applicant_timers;
    g.roster_timer = d.roster_timer;
    g.next_id = d.next_id;
    Ok(g)
}

pub fn worlds_dir() -> PathBuf {
    let base = std::env::var_os("APPDATA").map(PathBuf::from).unwrap_or_else(|| PathBuf::from("."));
    base.join("MonsterWorkshop").join("worlds")
}

/// "world-3" shows as "WORLD 3".
pub fn display_name(path: &Path) -> String {
    path.file_stem().map_or_else(String::new, |s| s.to_string_lossy().replace('-', " ").to_uppercase())
}

/// Saved worlds, most recently played first.
pub fn list_worlds() -> Vec<PathBuf> {
    let Ok(dir) = std::fs::read_dir(worlds_dir()) else { return Vec::new() };
    let mut worlds: Vec<(std::time::SystemTime, PathBuf)> = dir
        .filter_map(|e| e.ok())
        .map(|e| e.path())
        .filter(|p| p.extension().is_some_and(|x| x == "json"))
        .map(|p| (std::fs::metadata(&p).and_then(|m| m.modified()).unwrap_or(std::time::UNIX_EPOCH), p))
        .collect();
    worlds.sort_by(|a, b| b.0.cmp(&a.0));
    worlds.into_iter().map(|(_, p)| p).collect()
}

pub fn new_world_path() -> PathBuf {
    let dir = worlds_dir();
    (1..).map(|n| dir.join(format!("world-{n}.json"))).find(|p| !p.exists()).expect("a free world name")
}

/// Writes atomically: temporary file, then rename. The two previous saves become backups.
pub fn save(g: &Game, path: &Path) -> std::io::Result<()> {
    if let Some(dir) = path.parent() {
        std::fs::create_dir_all(dir)?;
    }
    let tmp = path.with_extension("json.tmp");
    std::fs::write(&tmp, to_json(g))?;
    let (bak1, bak2) = (path.with_extension("bak1"), path.with_extension("bak2"));
    if bak1.exists() {
        let _ = std::fs::rename(&bak1, &bak2);
    }
    if path.exists() {
        let _ = std::fs::rename(path, &bak1);
    }
    std::fs::rename(&tmp, path)
}

pub fn load(path: &Path) -> Result<Game, String> {
    let json = std::fs::read_to_string(path).map_err(|e| format!("can't read {}: {e}", path.display()))?;
    from_json(&json)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game::routes::*;

    #[test]
    fn a_saved_world_replays_exactly_like_the_original() {
        let mut g = Game::new();
        g.script(MOGG_HARVEST);
        g.script(MOGG_HOME);
        g.script(&format!("r{MOGG_HARVEST}{MOGG_HOME}rp"));
        g.script("t");
        g.run_ticks(10);
        g.script(&format!("r{}{}rp", bruna_chain(), bruna_home()));
        g.run_ticks(777); // mid-loop: monsters sliding, crafts and harvests half done

        let mut copy = from_json(&to_json(&g)).expect("loads");
        let (a, b) = (to_json(&copy), to_json(&g));
        if a != b {
            let i = a.bytes().zip(b.bytes()).position(|(x, y)| x != y).unwrap_or(a.len().min(b.len()));
            panic!("saves differ at {i}: loaded {:?} vs original {:?}", &a[i.saturating_sub(80)..(i + 40).min(a.len())], &b[i.saturating_sub(80)..(i + 40).min(b.len())]);
        }
        g.run_ticks(3000);
        copy.run_ticks(3000);
        assert_eq!(to_json(&copy), to_json(&g), "both copies stay identical while they run");
        assert!(g.cloth_stored() > 0 && g.renown > 0.0);
    }

    #[test]
    fn a_save_from_another_version_says_so() {
        let json = to_json(&Game::new()).replacen(&format!("\"version\":{VERSION}"), "\"version\":1", 1);
        let err = from_json(&json).err().expect("an old save doesn't load");
        assert!(err.contains("ANOTHER VERSION"), "{err}");
    }

    #[test]
    fn saving_writes_atomically_and_keeps_two_backups() {
        let dir = std::env::temp_dir().join(format!("mw_save_test_{}", std::process::id()));
        let path = dir.join("world-1.json");
        let mut g = Game::new();
        for _ in 0..3 {
            g.run_ticks(5);
            save(&g, &path).expect("saves");
        }
        assert!(path.exists() && path.with_extension("bak1").exists() && path.with_extension("bak2").exists());
        assert!(!path.with_extension("json.tmp").exists());
        assert_eq!(load(&path).expect("loads").frame, g.frame);
        let _ = std::fs::remove_dir_all(&dir);
    }
}
