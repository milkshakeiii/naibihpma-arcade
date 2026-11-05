#[cfg(windows)]
mod audio;
mod carry;
#[cfg_attr(not(windows), allow(dead_code))] // the player's side: only Windows has a window so far
mod client;
#[allow(dead_code)] // the schema mirrors the JSON; some fields are read in later stages
mod data;
#[allow(dead_code)] // tiers, value, weight and the forecast feed stages 3 and 4
mod econ;
#[allow(dead_code)] // shared copy of the arcade framebuffer; line/circle unused here
mod framebuffer;
mod frontier;
mod game;
mod gear;
mod inspect;
mod knot;
mod loadtest;
mod motion;
mod net;
mod palette;
mod recipes;
mod render;
mod save;
mod scenery;
mod sched;
mod server;
mod sprites;
mod tutorial;
mod wagon;
#[cfg(windows)]
mod window;
mod work;
mod workshops;
mod world;

use framebuffer::Framebuffer;
use game::{Game, Input};
use server::Server;
use sprites::Sprites;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

/// A world for trying what's new around the Temple Bell (`--temple-test-world`), saved as
/// "TEMPLE TEST" in the world list. Both river crossings are built and Mogg and Bruna stand at
/// the trader's stall in Temple Town. Laid out ready, all plain work with Mogg's share:
/// - by the stall, cloth, blue cloth, thread and rope, enough for each trade twice;
/// - by the foundry, everything a bell takes, and a wagon kit (to haul it) and a bin kit (which
///   won't set up in town);
/// - in the temple, a bell ready to hang, so the one cast next finds the frame resting;
/// - by Tanglemere's kiln, logs and sticks for charcoal;
/// - and 1500 renown, for the temple's teachings, patience and its lucky charm.
fn temple_test_world() {
    save_test_world(&temple_test_game(), "temple-test.json");
}

/// A world for trying equipment (`--outfitter-test-world`), saved as "OUTFITTER TEST" in the
/// world list. Both river crossings are built, and Mogg and Bruna stand in Tanglemere's
/// outfitter's with 1500 renown to spend. Mogg is ten renown short of seasoned, and a bolt of
/// his cloth lies just inside the cloth hall's door: push it up into the store.
fn outfitter_test_world() {
    save_test_world(&outfitter_test_game(), "outfitter-test.json");
}

fn save_test_world(g: &Game, file: &str) {
    let path = save::new_world_path().with_file_name(file);
    match save::save(g, &path) {
        Ok(()) => println!("saved {} as {}", path.display(), save::display_name(&path)),
        Err(e) => eprintln!("saving {} failed: {e}", path.display()),
    }
}

/// A new world with both river crossings built.
fn bridged_game() -> Game {
    let mut g = Game::new();
    for board in g.world.boards.iter_mut().filter(|b| matches!(b.project, world::Project::Footbridge | world::Project::Bridge)) {
        board.have = board.need;
    }
    g.world.build_footbridge();
    g.world.build_bridge();
    g
}

/// A plain, standard good made from `from`, with `owner`'s share, so whatever it goes into is
/// theirs too.
fn plain_work(g: &Game, owner: u32, id: &str, carried: &[&str], depth: u32, from: &[&str]) -> econ::ItemData {
    let raw: Vec<econ::RawMaterial> = from
        .iter()
        .map(|f| {
            let r = g.data.good(f);
            econ::RawMaterial { good: r.id.clone(), base_value: r.raw_material_base_value.unwrap_or(5.0), density: r.raw_material_density.unwrap_or(1.0), spent: false }
        })
        .collect();
    let value = econ::value(g.data.good(id), &raw, depth, 1.0, None);
    econ::ItemData { good: id.into(), quality: 1.0, carried_tags: carried.iter().map(|t| t.to_string()).collect(), raw, depth, value, shares: vec![econ::Share { monster: owner, count: 1.0 }], wear: 0.0, haul: Vec::new(), carted: None }
}

pub(crate) fn outfitter_test_game() -> Game {
    let mut g = bridged_game();
    let t = g.world.townsfolk.iter().position(|t| t.kind == world::Folk::Outfitter).expect("the outfitter");
    let (tx, ty) = (g.world.townsfolk[t].x, g.world.townsfolk[t].y);
    let ours: Vec<usize> = (0..g.monsters.len()).filter(|&i| g.monsters[i].commune == 0).collect();
    for (n, &i) in ours.iter().enumerate() {
        let m = &mut g.monsters[i];
        (m.x, m.y, m.facing) = (tx - 2 + 4 * n as i32, ty, world::Dir::Down);
    }
    g.communes[0].renown = 1500.0;
    let mogg = ours[0];
    g.monsters[mogg].earned = 990.0;
    // Bruna has a magnifying glass and a lucky charm in hand, from overseas and the temple, to see
    // what they do to a forecast: pieces not on any rack yet.
    if let Some(&bruna) = ours.get(1) {
        g.monsters[bruna].hands = vec![Some(game::Held::Piece("magnifying_glass".into())), Some(game::Held::Piece("lucky_charm".into()))];
    }
    let store = g.world.bins.iter().find(|b| b.scores && b.x >= 125).map(|b| (b.x, b.y)).expect("Tanglemere's cloth store");
    let cloth = plain_work(&g, g.monsters[mogg].id, "fabric", &["cotton"], 3, &["cotton_bolls"]);
    g.world.items.push(world::Item::new(store.0, store.1 + 2, cloth));
    g
}

pub(crate) fn temple_test_game() -> Game {
    let mut g = bridged_game();
    let stall = g.world.shops.iter().position(|s| s.kind.trades()).expect("the trader's stall");
    let (sx, sy) = g.world.shops[stall].spot;
    let ours: Vec<usize> = (0..g.monsters.len()).filter(|&i| g.monsters[i].commune == 0).collect();
    for (n, &i) in ours.iter().enumerate() {
        (g.monsters[i].x, g.monsters[i].y) = (sx + 2 * n as i32, sy + 2);
    }
    let mogg = g.monsters[ours[0]].id;
    // Plain, standard work made from `from`, with Mogg's share, so whatever it goes into is his too.
    let plain = |g: &Game, id: &str, carried: &[&str], depth: u32, from: &[&str]| plain_work(g, mogg, id, carried, depth, from);
    // Lays goods on free ground (`floor`: grass or a building's floor) in the box, row by row.
    let lay = |g: &mut Game, goods: Vec<econ::ItemData>, xs: std::ops::RangeInclusive<i32>, ys: std::ops::RangeInclusive<i32>, floor: world::Tile| {
        let free: Vec<(i32, i32)> = ys.flat_map(|y| xs.clone().map(move |x| (x, y))).filter(|&(x, y)| g.world.tile(x, y) == floor && g.world.item_at(x, y).is_none() && g.world.plant_at(x, y).is_none() && g.monster_at(x, y).is_none()).collect();
        assert!(free.len() >= goods.len(), "room for {} goods", goods.len());
        for (d, (x, y)) in goods.into_iter().zip(free) {
            g.world.items.push(world::Item::new(x, y, d));
        }
    };
    // By the stall, a row of each.
    let cotton = &["cotton_bolls"][..];
    for (k, (id, carried, depth, count)) in [("dyed_fabric", &["cotton", "indigo"][..], 4, 4), ("fabric", &["cotton"][..], 3, 6), ("thread", &["cotton"][..], 2, 6), ("rope", &["cotton"][..], 3, 6)].into_iter().enumerate() {
        let goods = (0..count).map(|_| plain(&g, id, carried, depth, cotton)).collect();
        let y = sy + 1 + k as i32;
        lay(&mut g, goods, sx - 15..=sx - 3, y..=y, world::Tile::Grass);
    }
    // By the foundry, a bell's worth, then the two kits.
    let foundry = g.world.shops.iter().position(|s| s.kind.label() == "FOUNDRY").expect("the foundry");
    let (fx, fy) = g.world.shops[foundry].spot;
    let bell_set: Vec<econ::ItemData> = [
        ("quicklime", "limestone"),
        ("mortar", "mortar"),
        ("beeswax", "beeswax"),
        ("oil", "oil"),
        ("red_clay", "red_clay"),
        ("charcoal", "hemlock_log"),
        ("hemlock_log", "hemlock_log"),
        ("elder_sticks", "elder_sticks"),
        ("coke", "coke"),
        ("copper_ingot", "copper_ingot"),
        ("tin_ingot", "tin_ingot"),
    ]
    .iter()
    .map(|&(id, from)| plain(&g, id, &[], (id != from) as u32, &[from]))
    .collect();
    lay(&mut g, bell_set, fx - 12..=fx + 8, fy + 6..=fy + 6, world::Tile::Grass);
    let kits = vec![plain(&g, "wagon_kit", &["oak"], 3, &["oak_log"; 12]), plain(&g, "bin_kit", &["hemlock"], 2, &["hemlock_log"; 6])];
    lay(&mut g, kits, fx + 1..=fx + 6, fy + 8..=fy + 8, world::Tile::Grass);
    // In the temple, three tiles in front of the frame: push it down into it.
    let frame = g.world.boards.iter().position(|b| b.project == world::Project::Bell).expect("the bell frame");
    let (bx, by) = (g.world.boards[frame].x, g.world.boards[frame].y);
    let bell = plain(&g, "temple_bell", &["bronze"], 3, &["copper_ingot", "tin_ingot", "limestone", "red_clay", "mortar", "beeswax", "oil"]);
    g.world.items.push(world::Item::new(bx, by - 3, bell));
    // By Tanglemere's kiln, two logs and two sticks: charcoal twice; and two coal, for coke.
    let kiln = g.world.shops.iter().position(|s| s.kind.label() == "KILN" && s.frontier.is_none()).expect("Tanglemere's kiln");
    let (kx, ky) = g.world.shops[kiln].spot;
    let wood = ["hemlock_log", "hemlock_log", "elder_sticks", "elder_sticks", "coal", "coal"].iter().map(|&id| plain(&g, id, &[], 0, &[id])).collect();
    lay(&mut g, wood, kx - 3..=kx + 3, ky + 1..=ky + 3, world::Tile::Floor);
    // Below the free end of the seam, two bins of spare tools, twelve picks and twelve
    // shovels, their chutes toward it with a tile between them, where Space reaches either:
    // record a digger taking its tools from the chutes, and its loop takes new ones there when
    // they wear out, with no trading.
    let seam: Vec<(i32, i32)> = g.world.plants.iter().filter(|p| p.good == "coal" && p.owner.is_none()).map(|p| (p.x, p.y)).collect();
    let east = *seam.iter().max().expect("a seam for anyone");
    for (k, tool) in ["pick", "shovel"].into_iter().enumerate() {
        let (x, y) = (east.0 + 2 * k as i32, east.1 + 4);
        let stock = (0..12).map(|_| plain(&g, tool, &[], 1, &["copper_ingot"])).collect();
        let frontier = world::FrontierBin { kit: "bin_kit".into(), material: "HEMLOCK".into(), room: 12.0, idle: 0, held: 12 };
        g.world.bins.push(world::Bin { x, y, accepts: vec![tool.into()], stock, mouth: Some((x, y - 1)), timer: 0, bounce: 0, scores: false, frontier: Some(frontier) });
        let w = g.world.w;
        g.world.tiles[(y * w + x) as usize] = world::Tile::Bin;
    }
    // The pass to the hills, three cloth short of opening: seventeen came in this hour, and
    // three lie by Temple Town's road east.
    let gate = g.world.gates.iter().position(|gt| gt.province == "THE HILLS").expect("the gate into the hills");
    let b = g.world.gates[gate].board;
    let board = &mut g.world.boards[b];
    board.log = vec![0; board.need as usize - 3];
    board.have = board.log.len() as u32;
    let cloth = (0..3).map(|_| plain(&g, "fabric", &["cotton"], 3, cotton)).collect();
    lay(&mut g, cloth, 75 + 40..=75 + 46, 32 + 13..=32 + 13, world::Tile::Grass);
    // Renown for the temple's teachings, its patience and its lucky charm.
    g.communes[0].renown = 1500.0;
    g
}

#[cfg(test)]
#[test]
fn the_outfitter_test_world_has_renown_to_spend_and_a_season_one_bolt_away() {
    let mut g = outfitter_test_game();
    assert!(g.communes[0].renown >= 1000.0);
    let mogg = g.communes[0].controlled;
    assert!(g.world.racks.iter().any(|r| (r.x - g.monsters[mogg].x).abs() <= 3 && r.y == g.monsters[mogg].y + 1), "racks beside Mogg");
    // Push the bolt up into the store: Mogg is seasoned.
    let store = g.world.bins.iter().find(|b| b.scores && b.x >= 125).map(|b| (b.x, b.y)).expect("the store");
    (g.monsters[mogg].x, g.monsters[mogg].y) = (store.0, store.1 + 3);
    g.script("UU");
    assert!(crate::gear::seasoning(g.monsters[mogg].earned) == 1, "earned {}", g.monsters[mogg].earned);
}

#[cfg(test)]
#[test]
fn the_temple_test_world_has_whats_new_ready_to_try() {
    let g = temple_test_game();
    let stall = g.world.shops.iter().position(|s| s.kind.trades()).expect("the trader's stall");
    let (sx, sy) = g.world.shops[stall].spot;
    assert!(g.monsters.iter().filter(|m| m.commune == 0).all(|m| (m.x - sx).abs() <= 2 && m.y == sy + 2), "Mogg and Bruna stand at the stall");
    assert!(g.world.footbridge_built(), "the river can be crossed");
    let lying = |group: &Vec<String>| (0..g.world.items.len()).filter(|&i| g.world.item_matches(&g.data, i, group)).count();
    for r in g.offers(stall).into_iter().chain(["temple_bell", "charcoal", "coke"]) {
        for (_, group) in g.data.good(r).input_groups() {
            assert!(lying(group) >= 2 || r == "temple_bell", "{r} wants {group:?}: {} lie about", lying(group));
            assert!(lying(group) >= 1, "{r} wants {group:?}");
        }
    }
    assert!(g.world.items.iter().any(|it| it.good() == "temple_bell"), "a bell to hang");
    let gate = g.world.gates.iter().find(|gt| gt.province == "THE HILLS").expect("the gate into the hills");
    let board = &g.world.boards[gate.board];
    assert_eq!(board.need - board.have, 3, "three cloth short of opening");
    assert!(g.world.items.iter().filter(|it| it.good() == "fabric" && crate::world::area_name(it.x, it.y) == "TEMPLE TOWN" && it.x >= 75 + 40).count() >= 3, "three to push there");
    for tool in ["pick", "shovel"] {
        let bin = g.world.bins.iter().find(|b| b.accepts == [tool]).unwrap_or_else(|| panic!("a bin of {tool}s"));
        assert_eq!(bin.stock.len(), 12, "twelve spare {tool}s");
        let (mx, my) = bin.mouth.expect("a chute");
        assert!(g.world.item_floor(mx, my) && g.world.item_at(mx, my).is_none(), "its chute is clear for the first {tool}");
        assert!(world::DIRS.iter().any(|d| g.world.walkable(mx + d.delta().0, my + d.delta().1) && (mx + d.delta().0, my + d.delta().1) != (bin.x, bin.y)), "a {tool} on the chute can be reached");
    }
    let loaded = save::from_json(&save::to_json(&g)).expect("it saves and loads");
    assert_eq!(loaded.world.items.len(), g.world.items.len());
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let after = |flag: &str| args.iter().position(|a| a == flag).and_then(|i| args.get(i + 1)).cloned();
    if args.iter().any(|a| a == "--export-screenshots") {
        export_screenshots(&after("--export-screenshots").map_or_else(|| PathBuf::from("screenshots"), PathBuf::from));
        return;
    }
    if args.iter().any(|a| a == "--export-places") {
        export_places(&after("--export-places").map_or_else(|| PathBuf::from("places"), PathBuf::from));
        return;
    }
    if args.iter().any(|a| a == "--temple-test-world") {
        temple_test_world();
        return;
    }
    if args.iter().any(|a| a == "--outfitter-test-world") {
        outfitter_test_world();
        return;
    }
    if args.iter().any(|a| a == "--server") {
        serve(after("--port"), after("--world"), args.iter().any(|a| a == "--busy"));
        return;
    }
    if let Some(n) = after("--bots") {
        bots(&n, after("--connect"));
        return;
    }
    #[cfg(windows)]
    {
        let lag = match after("--lag").map(|l| l.parse::<u64>()) {
            None => 0,
            Some(Ok(ms)) => ms,
            Some(Err(_)) => return eprintln!("--lag wants milliseconds"),
        };
        let lag = std::time::Duration::from_millis(lag);
        window::run(&mut audio::Audio::new(), args.iter().any(|a| a == "--windowed"), after("--connect"), after("--player"), lag);
    }
    #[cfg(not(windows))]
    eprintln!("Here the game runs only as a server: monster_workshop --server [--port N] [--world PATH]");
}

/// A server for players to connect to, until STOP is typed, reporting how it keeps up every
/// minute. The world is saved at `world` (by default `server.json` beside the saved worlds), and
/// started there if it's new. A `busy` one serves a full server's made-up world instead (see
/// `loadtest.rs`), reporting every ten seconds, with its saves thrown away.
fn serve(port: Option<String>, world: Option<String>, busy: bool) {
    let port = match port.map(|p| p.parse::<u16>()) {
        None => net::DEFAULT_PORT,
        Some(Ok(p)) => p,
        Some(Err(_)) => return eprintln!("--port wants a number"),
    };
    let (game, path, every) = if busy {
        (loadtest::busy_world(loadtest::FULL), std::env::temp_dir().join("monster_workshop_busy.json"), 10)
    } else {
        let path = world.map_or_else(|| save::worlds_dir().join("server.json"), PathBuf::from);
        let game = if path.exists() {
            match save::load(&path) {
                Ok(g) => g,
                Err(e) => return eprintln!("{}: {e}", path.display()),
            }
        } else {
            Game::new()
        };
        (game, path, 60)
    };
    let server = match std::net::TcpListener::bind(("0.0.0.0", port)).and_then(|l| Server::new(game, path.clone(), l)) {
        Ok(s) => s.reporting(std::time::Duration::from_secs(every)),
        Err(e) => return eprintln!("can't take players on port {port}: {e}"),
    };
    if busy {
        println!("Serving a busy made-up world on port {port}: {:?}. Type STOP to stop.", loadtest::FULL);
    } else {
        println!("Serving {} on port {port}. Type STOP to save and stop.", path.display());
    }
    let stop = stop_when_typed();
    server.run(&stop);
    if !busy {
        println!("Saved {}.", path.display());
    }
}

/// Set once STOP is typed.
fn stop_when_typed() -> Arc<AtomicBool> {
    let stop = Arc::new(AtomicBool::new(false));
    let typed = stop.clone();
    std::thread::spawn(move || {
        for line in std::io::stdin().lines().map_while(Result::ok) {
            if line.trim().eq_ignore_ascii_case("stop") {
                typed.store(true, Ordering::Relaxed);
                return;
            }
        }
    });
    stop
}

/// `n` bot players on the server at `addr` (this computer's by default) until STOP is typed,
/// reporting every ten seconds what they've been sent.
fn bots(n: &str, addr: Option<String>) {
    let Ok(n) = n.parse::<u64>() else { return eprintln!("--bots wants a number") };
    let addr = addr.unwrap_or_else(|| "127.0.0.1".into());
    let addr = if addr.contains(':') { addr } else { format!("{addr}:{}", net::DEFAULT_PORT) };
    let (seen, stop) = (Arc::new(loadtest::Seen::default()), stop_when_typed());
    for i in 0..n {
        let (addr, seen, stop) = (addr.clone(), seen.clone(), stop.clone());
        std::thread::spawn(move || {
            if let Err(e) = loadtest::play_bot(&addr, i, &seen, &stop) {
                eprintln!("bot {i}: {e}");
            }
        });
    }
    println!("{n} bots playing on {addr}. Type STOP to stop.");
    let every = std::time::Duration::from_secs(10);
    let mut last = [0; 4];
    while !stop.load(Ordering::Relaxed) {
        std::thread::sleep(every);
        let now = [&seen.frames, &seen.bytes, &seen.flips, &seen.flip_bytes].map(|a| a.load(Ordering::Relaxed));
        let playing = seen.playing.load(Ordering::Relaxed);
        let each = |k: usize| (now[k] - last[k]) as f64 / every.as_secs_f64() / playing.max(1) as f64;
        println!(
            "{playing} bots playing | each sent {:.1} frames/s, {:.1} KB/s, of which new screens {:.2}/s, {:.1} KB/s | {} garbled frames",
            each(0),
            each(1) / 1000.0,
            each(2),
            each(3) / 1000.0,
            seen.garbled.load(Ordering::Relaxed)
        );
        last = now;
        if playing == 0 {
            return;
        }
    }
}

/// Headless scripted playthrough of the Fabric chain, saving 2x PNGs at key moments.
/// The layout workflow's pictures (DESIGN.md, Layout by image model): every screen of the world
/// as a place, without the HUD, four times its size, half a minute into a new world so the town
/// is at work. Each comes plain and with its tile grid drawn on, named by its area and its
/// top-left tile.
fn export_places(dir: &Path) {
    const SCALE: usize = 4;
    let _ = std::fs::create_dir_all(dir);
    let (sprites, pal) = (Sprites::new(), palette::build());
    let mut fb = Framebuffer::new();
    let mut g = Game::new();
    g.run_ticks(450);
    let (sw, sh) = (world::SCREEN_W, world::SCREEN_H);
    let mut buf = Vec::new();
    let mut n = 0;
    for ty in (0..g.world.h).step_by(sh as usize) {
        for tx in (0..g.world.w).step_by(sw as usize) {
            // A screen of nothing but hedge is nowhere.
            if (ty..ty + sh).all(|y| (tx..tx + sw).all(|x| g.world.tile(x, y) == world::Tile::Hedge)) {
                continue;
            }
            render::draw_place(&g, &sprites, &mut fb, (tx, ty));
            let (w, _) = fb.present(&mut buf, SCALE, &pal, &pal, 0, false);
            let h = render::VIEW_H as usize * SCALE;
            let name = format!("{}_{tx}_{ty}", world::area_name(tx + sw / 2, ty + sh / 2).to_lowercase().replace(' ', "_"));
            framebuffer::save_argb_png(&dir.join(format!("{name}.png")), &buf[..w * h], w, h);
            // The grid: every tile's top and left edge a third darker.
            let tile = world::TS as usize * SCALE;
            for y in 0..h {
                for x in 0..w {
                    if x % tile == 0 || y % tile == 0 {
                        let p = buf[y * w + x];
                        let dim = |c: u32| ((c & 0xFF) * 2 / 3) & 0xFF;
                        buf[y * w + x] = 0xFF00_0000 | dim(p >> 16) << 16 | dim(p >> 8) << 8 | dim(p);
                    }
                }
            }
            framebuffer::save_argb_png(&dir.join(format!("{name}_grid.png")), &buf[..w * h], w, h);
            n += 1;
        }
    }
    println!("exported {n} places to {}", dir.display());
}

fn export_screenshots(dir: &Path) {
    let _ = std::fs::create_dir_all(dir);
    let sprites = Sprites::new();
    let pal = palette::build();
    let mut fb = Framebuffer::new();
    let mut g = Game::new();
    let mut save = |g: &Game, name: &str| {
        render::draw(g, 0, &sprites, &mut fb);
        fb.save_png(&dir.join(name), &pal, &pal, 0);
    };
    let use_now = |g: &mut Game| g.press(0, Input { use_pressed: true, ..Input::default() });

    g.run_ticks(2);
    save(&g, "01_fields.png");
    g.script("RRRRRR");
    use_now(&mut g);
    g.run_ticks(12);
    save(&g, "02_harvesting.png");
    g.run_ticks(30);
    g.communes[0].inspecting = true;
    save(&g, "03_examine_boll.png");
    g.communes[0].inspecting = false;
    g.script("RRR");
    g.script("t");
    g.run_ticks(10);
    g.script("RDRRU");
    use_now(&mut g);
    g.run_ticks(15);
    save(&g, "04_workbench.png");
    g.run_ticks(40);
    g.script(&format!("{}UR", "R".repeat(15)));
    g.communes[0].inspecting = true;
    save(&g, "05_spinnery_forecast.png");
    g.communes[0].card_page = 1;
    save(&g, "05b_spinnery_details.png");
    g.communes[0].card_page = 0;
    g.communes[0].inspecting = false;
    use_now(&mut g);
    g.run_ticks(84);
    save(&g, "06_thread_and_popup.png");
    g.script(&format!("D{}UR{}LD", "R".repeat(13), "D".repeat(12)));
    use_now(&mut g);
    g.run_ticks(200);
    save(&g, "07_loom_house.png");
    g.run_ticks(600);
    g.script(&format!("R{}", "D".repeat(6)));
    g.run_ticks(4);
    save(&g, "08_cloth_store_renown.png");
    g.monsters[g.communes[0].controlled].facing = game::Dir::Left;
    g.communes[0].inspecting = true;
    save(&g, "09_examine_bruna.png");
    g.communes[0].card_page = 1;
    save(&g, "09b_bruna_details.png");
    g.communes[0].card_page = 0;
    g.communes[0].inspecting = false;
    g.communes[0].controlled = 0;
    let stand = g.world.stands[1];
    (g.monsters[0].x, g.monsters[0].y) = (stand.0, stand.1 - 1);
    g.monsters[0].facing = game::Dir::Down;
    g.communes[0].renown = 60.0;
    save(&g, "10_gathering_hall.png");
    g.communes[0].help = true;
    save(&g, "11_help.png");
    g.communes[0].help = false;

    // M2: the footbridge's board, the footbridge, the wilderness and Tanglemere.
    g.communes[0].renown = 0.0;
    let ford = g.world.boards.iter().position(|b| b.project == world::Project::Footbridge).expect("the footbridge's board");
    let board = g.world.boards[ford].clone();
    (g.monsters[0].x, g.monsters[0].y) = (board.x, board.y + 1);
    g.monsters[0].facing = game::Dir::Up;
    g.world.boards[ford].have = board.need - 1;
    g.run_ticks(2);
    g.communes[0].inspecting = true;
    save(&g, "13_footbridge_board.png");
    g.communes[0].inspecting = false;
    let stored = g.world.bins.iter().flat_map(|b| &b.stock).find(|d| d.good == "fabric").cloned().expect("fabric in the cloth store");
    let mut rope = stored.clone();
    rope.good = "rope".into();
    g.world.items.push(world::Item::new(board.x, board.y + 1, rope));
    (g.monsters[0].x, g.monsters[0].y) = (board.x, board.y + 2);
    g.script("U");
    g.run_ticks(24);
    save(&g, "14_the_footbridge.png");
    let (fx, fy) = *g.world.footbridge.first().expect("a footbridge");
    let mut cloth = stored;
    cloth.quality = 1.1;
    g.world.items.push(world::Item::new(fx - 1, fy, cloth));
    (g.monsters[0].x, g.monsters[0].y) = (fx - 2, fy);
    g.script("RRR");
    g.run_ticks(12);
    save(&g, "15_crossing_the_footbridge.png");
    (g.monsters[0].x, g.monsters[0].y) = (80, 7);
    g.run_ticks(3);
    save(&g, "16_thatch_hills.png");
    (g.monsters[0].x, g.monsters[0].y) = (106, 9);
    g.run_ticks(60);
    save(&g, "17_dye_meadows.png");
    (g.monsters[0].x, g.monsters[0].y) = (164, 9);
    g.run_ticks(3);
    save(&g, "18_tanglemere.png");
    g.run_ticks(60);
    g.communes[0].roster = true;
    save(&g, "19_roster_and_edge_markers.png");
    g.communes[0].roster = false;

    // Rope at the gate bench, and a pull out of a corner.
    let mut thread = g.world.bins.iter().flat_map(|b| &b.stock).find(|d| d.good == "fabric").cloned().expect("fabric");
    thread.good = "thread".into();
    g.world.items.push(world::Item::new(40, 8, thread.clone()));
    g.world.items.push(world::Item::new(41, 8, thread));
    (g.monsters[0].x, g.monsters[0].y, g.monsters[0].facing) = (42, 8, game::Dir::Down);
    g.run_ticks(2);
    g.communes[0].inspecting = true;
    save(&g, "20_gate_bench.png");
    g.communes[0].inspecting = false;
    let mut stuck = g.world.bins.iter().flat_map(|b| &b.stock).find(|d| d.good == "fabric").cloned().expect("fabric");
    stuck.quality = 0.8;
    g.world.items.push(world::Item::new(1, 1, stuck));
    (g.monsters[0].x, g.monsters[0].y, g.monsters[0].facing) = (2, 1, game::Dir::Left);
    g.script("L");
    save(&g, "21_stuck.png");
    let mut inp = Input { pull: true, ..Input::default() };
    inp.pressed[3] = true;
    inp.held[3] = true;
    g.press(0, inp);
    g.run_ticks(14);
    save(&g, "22_pulling.png");

    // M2 stage 2: Tanglemere's dye works and the Thatch Hills' limestone.
    let base = g.world.bins.iter().flat_map(|b| &b.stock).find(|d| d.good == "fabric").cloned().expect("fabric");
    let good = |id: &str, tags: &[&str]| econ::ItemData { good: id.into(), carried_tags: tags.iter().map(|t| t.to_string()).collect(), quality: 0.9, ..base.clone() };
    let put = |g: &mut Game, (x, y): (i32, i32), id: &str, tags: &[&str]| g.world.items.push(world::Item::new(x, y, good(id, tags)));
    g.run_ticks(30);
    put(&mut g, (131, 21), "ramie_stalks", &[]);
    put(&mut g, (133, 21), "elder_sticks", &[]);
    (g.monsters[0].x, g.monsters[0].y, g.monsters[0].facing) = (132, 21, game::Dir::Up);
    g.run_ticks(3);
    g.communes[0].inspecting = true;
    save(&g, "23_kiln_forecast.png");
    g.communes[0].inspecting = false;
    use_now(&mut g);
    g.run_ticks(12);
    save(&g, "24_kiln_burning.png");
    (g.monsters[0].x, g.monsters[0].y, g.monsters[0].facing) = (154, 5, game::Dir::Up);
    g.run_ticks(3);
    save(&g, "25_indigo_house_empty.png");
    put(&mut g, (141, 4), "fabric", &["cotton"]);
    put(&mut g, (143, 4), "indigo", &["indigofera"]);
    (g.monsters[0].x, g.monsters[0].y, g.monsters[0].facing) = (142, 4, game::Dir::Down);
    use_now(&mut g);
    g.run_ticks(40);
    put(&mut g, (142, 6), "dyed_fabric", &["cotton", "indigo"]);
    (g.monsters[0].x, g.monsters[0].y, g.monsters[0].facing) = (142, 5, game::Dir::Down);
    g.communes[0].inspecting = true;
    save(&g, "26_dye_house.png");
    g.communes[0].inspecting = false;
    (g.monsters[0].x, g.monsters[0].y, g.monsters[0].facing) = (78, 19, game::Dir::Left);
    g.run_ticks(3);
    g.communes[0].inspecting = true;
    save(&g, "27_limestone.png");
    g.communes[0].inspecting = false;

    // M2 stage 3: the town wagon and its pumpkin, and the bridge.
    let (bed, pumpkin) = (g.world.wagons[0].at, g.world.wagons[0].pumpkin);
    (g.monsters[0].x, g.monsters[0].y, g.monsters[0].facing) = (pumpkin.0 + 1, pumpkin.1, game::Dir::Left);
    g.run_ticks(3);
    g.communes[0].inspecting = true;
    save(&g, "28_pumpkin.png");
    g.communes[0].inspecting = false;
    g.world.wagons[0].stock = (0..3).map(|_| (good("fabric", &["cotton"]), "PINEWATER".to_string())).collect();
    use_now(&mut g);
    g.script("UL");
    g.run_ticks(1);
    save(&g, "29_leading_the_wagon.png");
    use_now(&mut g);
    let back = g.world.wagons[0].behind();
    let w = g.world.wagons[0].at;
    let face = if back.0 < w.0 { game::Dir::Right } else if back.0 > w.0 { game::Dir::Left } else if back.1 < w.1 { game::Dir::Down } else { game::Dir::Up };
    (g.monsters[0].x, g.monsters[0].y, g.monsters[0].facing) = (back.0, back.1, face);
    g.run_ticks(2);
    g.communes[0].inspecting = true;
    save(&g, "30_wagon_card.png");
    g.communes[0].inspecting = false;
    let _ = bed;
    g.world.build_bridge();
    let &(bx, by) = g.world.bridge.first().expect("a bridge site");
    (g.monsters[0].x, g.monsters[0].y, g.monsters[0].facing) = (bx + 2, by + 1, game::Dir::Right);
    g.run_ticks(3);
    save(&g, "31_bridge.png");

    // A loop's next thread brought up ready beside the loom, still busy with the last one.
    let mut lg = Game::new();
    lg.monsters[0].bubble = None;
    lg.world.items.push(world::Item::new(44, 21, good("thread", &["cotton"])));
    (lg.monsters[0].x, lg.monsters[0].y, lg.monsters[0].facing) = (43, 21, game::Dir::Right);
    use_now(&mut lg);
    lg.world.items.push(world::Item::new(44, 17, good("thread", &["cotton"])));
    let m = &mut lg.monsters[1];
    (m.x, m.y, m.facing) = (44, 16, game::Dir::Down);
    m.routine = Some(game::Routine { flag: (44, 16), goals: vec![game::Goal::Move { good: "thread".into(), from: (44, 17), to: (44, 21) }] });
    (m.pc, m.playing) = (0, true);
    for _ in 0..300 {
        if lg.monsters[1].bubble.is_some() {
            break;
        }
        lg.run_ticks(1);
    }
    save(&lg, "32_ready_at_the_loom.png");
    let titles = ["WORLD 2".to_string(), "WORLD 1".to_string()];
    render::draw_title(&mut fb, &titles, 1, None, false);
    fb.save_png(&dir.join("12_title.png"), &pal, &pal, 0);
    render::draw_title(&mut fb, &titles, 2, Some("DEL AGAIN DELETES WORLD 1 FOR GOOD. ESC KEEPS IT."), true);
    fb.save_png(&dir.join("33_delete_world.png"), &pal, &pal, 0);

    // Every good with art of its own: how it lies in the world, then its icon, to check they
    // read as the same thing.
    let goods = [
        "cotton_bolls", "cotton_slivers", "thread", "fabric", "dyed_fabric", "rope", "spool", "cotton_gin", "cotton_bow", "spinning_wheel", "waist_loom", "wooden_tub",
        "hemlock_log", "oak_log", "elder_sticks", "rhododendron_sticks", "arrow_bamboo", "ramie_stalks", "indigofera_leaves", "lye", "limestone", "quicklime", "indigo",
        "workbench_kit", "kiln_kit",
    ];
    fb.rect_fill(0, 0, 400, 300, palette::PLANK + 1);
    for (k, id) in goods.iter().enumerate() {
        let (x, y) = ((k % 3) as i32 * 133 + 6, (k / 3) as i32 * 32 + 6);
        let s = sprites.item(id, None).0;
        fb.blit(s, x + (16 - s.w) / 2, y + 16 - s.h, false);
        match sprites.icon(id) {
            Some(icon) => fb.blit(icon, x + 22, y + 4, false),
            None => fb.text("-", x + 24, y + 4, palette::TEXT, Some(palette::OUTLINE), 1),
        }
        for (l, line) in tutorial::wrap(&g.good_name(id), 15).iter().enumerate() {
            fb.text(line, x + 34, y + 2 + l as i32 * 10, palette::TEXT, Some(palette::OUTLINE), 1);
        }
    }
    fb.save_png(&dir.join("34_sprites_and_icons.png"), &pal, &pal, 0);

    // A tool hangs on its peg as its icon; set down on the floor it's its full sprite.
    let mut tg = Game::new();
    tg.monsters[0].bubble = None;
    tg.world.items.push(world::Item::new(13, 8, good("cotton_gin", &[])));
    tg.world.items.push(world::Item::new(17, 10, good("cotton_bow", &[])));
    tg.world.items.push(world::Item::new(19, 10, good("cotton_gin", &[])));
    tg.world.items.push(world::Item::new(21, 10, good("spinning_wheel", &[])));
    tg.world.items.push(world::Item::new(23, 10, good("waist_loom", &[])));
    (tg.monsters[0].x, tg.monsters[0].y) = (16, 12);
    tg.run_ticks(2);
    render::draw(&tg, 0, &sprites, &mut fb);
    fb.save_png(&dir.join("35_tools.png"), &pal, &pal, 0);

    // M2 stage 4: the builders' bench, a kit's ghost (clear, then blocked by the road), and a
    // kiln unpacked in the Thatch Hills.
    let mut shot = |g: &Game, name: &str| {
        render::draw(g, 0, &sprites, &mut fb);
        fb.save_png(&dir.join(name), &pal, &pal, 0);
    };
    let mut fg = Game::new();
    fg.monsters[0].bubble = None;
    fg.communes[0].controlled = 1;
    (fg.monsters[1].x, fg.monsters[1].y, fg.monsters[1].facing) = (36, 28, game::Dir::Up);
    fg.run_ticks(2);
    fg.communes[0].inspecting = true;
    shot(&fg, "36_builders_bench.png");
    (fg.communes[0].inspecting, fg.communes[0].controlled) = (false, 0);
    fg.world.items.push(world::Item::new(66, 11, good("workbench_kit", &["oak"])));
    (fg.monsters[0].x, fg.monsters[0].y, fg.monsters[0].facing) = (66, 10, game::Dir::Down);
    fg.run_ticks(2);
    shot(&fg, "37_kit_ghost.png");
    fg.world.items.retain(|it| it.good() != "workbench_kit");
    fg.world.items.push(world::Item::new(62, 13, good("workbench_kit", &["oak"])));
    (fg.monsters[0].x, fg.monsters[0].y) = (62, 12);
    fg.run_ticks(2);
    shot(&fg, "38_kit_ghost_blocked.png");
    fg.world.items.push(world::Item::new(88, 10, good("kiln_kit", &["limestone", "oak"])));
    (fg.monsters[0].x, fg.monsters[0].y, fg.monsters[0].facing) = (88, 9, game::Dir::Down);
    fg.script("u");
    (fg.monsters[0].x, fg.monsters[0].y) = (88, 10);
    fg.run_ticks(40);
    fg.communes[0].inspecting = true;
    shot(&fg, "39_frontier_kiln.png");
    // The character sheet, a stone hammer in one hand, then its loop.
    let c = g.communes[0].controlled;
    let mut hammer = g.world.items.first().map(|it| it.data.clone()).unwrap_or_else(|| fg.world.items[0].data.clone());
    (hammer.good, hammer.carried_tags, hammer.quality) = ("stone_hammer".into(), vec!["oak".into()], 0.7);
    g.monsters[c].hands[0] = Some(game::Held::Tool(hammer));
    g.communes[0].sheet = true;
    shot(&g, "40_character_sheet.png");
    // Its loop: the whole fabric chain, partway round.
    g.monsters[c].routine = Some(loadtest::fabric_chain(&g, (0, 0)));
    (g.monsters[c].pc, g.monsters[c].playing) = (4, true);
    g.communes[0].sheet_page = 1;
    shot(&g, "41_character_sheet_loop.png");
    // The recipe book: what the commune has made, and a wagon kit heard of.
    g.communes[0].heard.insert("wagon_kit".into());
    g.communes[0].sheet_page = 2;
    shot(&g, "42_recipe_book.png");
    (g.communes[0].sheet, g.communes[0].sheet_page) = (false, 0);
    // Old Sawgrass, asked what could be made from two logs and a rope lying by his bench.
    (g.monsters[c].playing, g.monsters[c].x, g.monsters[c].y, g.monsters[c].facing) = (false, 28, 28, game::Dir::Left);
    for (x, id) in [(36, "hemlock_log"), (37, "hemlock_log"), (38, "rope")] {
        let mut d = g.world.items.first().map(|it| it.data.clone()).unwrap_or_else(|| fg.world.items[0].data.clone());
        (d.good, d.carried_tags) = (id.into(), Vec::new());
        g.world.items.push(world::Item::new(x, 20, d));
    }
    shot(&g, "43_old_sawgrass.png");
    // The builders' bench's card: a few of what else it makes, on its third page.
    (g.monsters[c].x, g.monsters[c].y) = (36, 28);
    g.communes[0].inspecting = true;
    g.communes[0].card_page = 2;
    shot(&g, "44_what_it_makes.png");
    // Esc with nothing open.
    (g.communes[0].inspecting, g.communes[0].leaving) = (false, true);
    shot(&g, "45_leaving.png");
    // A pumpkin on the builders' bench: it aims at a wagon kit, and its empty pads' ghosts show
    // what's still missing.
    g.communes[0].leaving = false;
    let bench = g.world.shops.iter().position(|s| s.spot == (36, 28)).expect("the builders' bench");
    let pad = g.world.shops[bench].pads[3];
    let mut d = g.world.items[0].data.clone();
    (d.good, d.carried_tags) = ("pumpkin".into(), Vec::new());
    g.world.items.push(world::Item::new(pad.0, pad.1, d));
    shot(&g, "46_aiming.png");
    // Temple Town: the trader at his stall says what he's got.
    let stall = g.world.shops.iter().position(|s| s.kind.trades()).expect("the trader's stall");
    let (sx, sy) = g.world.shops[stall].spot;
    (g.monsters[c].x, g.monsters[c].y, g.monsters[c].facing) = (sx, sy, game::Dir::Up);
    shot(&g, "47_trader.png");
    // The foundry, where the bell is cast: eleven pads, a chisel and a shovel lent.
    let foundry = g.world.shops.iter().position(|s| s.kind.label() == "FOUNDRY").expect("the foundry");
    (g.monsters[c].x, g.monsters[c].y) = g.world.shops[foundry].spot;
    g.communes[0].inspecting = true;
    g.communes[0].card_page = 0;
    shot(&g, "48_foundry.png");
    g.communes[0].inspecting = false;
    // A bell hung in the temple: it rings for everyone, and the plaque keeps it.
    let frame = g.world.boards.iter().position(|b| b.project == world::Project::Bell).expect("the bell frame");
    let (fx, fy) = (g.world.boards[frame].x, g.world.boards[frame].y);
    g.world.bells.push(world::Hung { quality: 1.4, metal: "BRONZE".into(), caster: "MOGG".into(), others: 12 });
    (g.world.boards[frame].have, g.world.boards[frame].rest) = (1, 1000);
    let line = game::Game::bell_line(g.world.bells.last().expect("a bell"));
    g.announcement = Some(game::Announcement { title: "THE TEMPLE BELL RINGS!".into(), line, t: 100 });
    (g.monsters[c].x, g.monsters[c].y, g.monsters[c].facing) = (fx, fy - 2, game::Dir::Down);
    shot(&g, "49_bell_rings.png");
    g.announcement = None;
    let (px, py) = g.world.plaques[0];
    (g.monsters[c].x, g.monsters[c].y, g.monsters[c].facing) = (px, py - 1, game::Dir::Down);
    g.communes[0].inspecting = true;
    shot(&g, "50_bell_plaque.png");
    g.communes[0].inspecting = false;
    // Old Spoke's yard: his woodpiles, his bench and order board, and the bench by his gate.
    (g.monsters[c].x, g.monsters[c].y, g.monsters[c].facing) = (17, 27, game::Dir::Up);
    shot(&g, "51_old_spokes_yard.png");
    // Facing him, he says what a wheel takes; his card says who he is.
    let spoke = g.monsters.iter().position(|m| m.worker.is_some()).expect("Old Spoke");
    (g.monsters[spoke].x, g.monsters[spoke].y, g.monsters[spoke].playing, g.monsters[spoke].slide) = (21, 26, false, None);
    (g.monsters[c].x, g.monsters[c].y, g.monsters[c].facing) = (21, 27, game::Dir::Up);
    shot(&g, "52_old_spoke_talks.png");
    g.communes[0].inspecting = true;
    shot(&g, "53_old_spoke_card.png");
    g.communes[0].inspecting = false;
    // The outfitter's things to work with, by its door: facing the towel, its card, then in hand.
    let mut og = outfitter_test_game();
    let oc = og.communes[0].controlled;
    let towel = og.world.racks.iter().find(|r| r.piece == "towel").map(|r| (r.x, r.y)).expect("the towel's rack");
    (og.monsters[oc].x, og.monsters[oc].y, og.monsters[oc].facing) = (towel.0 + 1, towel.1, game::Dir::Left);
    og.monsters[oc].bubble = None;
    shot(&og, "54_outfitter_towel.png");
    og.communes[0].inspecting = true;
    shot(&og, "55_towel_card.png");
    og.communes[0].inspecting = false;
    og.script("u");
    og.monsters[oc].facing = game::Dir::Down;
    shot(&og, "56_towel_in_hand.png");
    og.script("c");
    shot(&og, "57_sheet_with_a_towel.png");
    og.script("c");
    // Bruna at the spinnery, a magnifying glass and a lucky charm in hand: what helps.
    let bruna = og.monsters.iter().position(|m| m.commune == 0 && m.name == "BRUNA").expect("Bruna");
    let mut sg = Game::new();
    sg.monsters[1].hands = std::mem::take(&mut og.monsters[bruna].hands);
    sg.monsters[1].skills.specific.insert("thread".into(), 0.3);
    let mut slivers = plain_work(&sg, sg.monsters[1].id, "cotton_slivers", &["cotton"], 1, &["cotton_bolls"]);
    slivers.quality = 0.7;
    sg.world.items.push(world::Item::new(30, 9, slivers));
    sg.communes[0].controlled = 1;
    (sg.monsters[1].x, sg.monsters[1].y) = (30, 8);
    for k in ["spinning", "textiles", "weaving"] {
        sg.monsters[1].skills.applied.insert(k.into(), 0.4);
    }
    sg.communes[0].inspecting = true;
    sg.communes[0].card_page = 1;
    shot(&sg, "58_spinnery_what_helps.png");
    sg.communes[0].inspecting = false;
    sg.script("c");
    shot(&sg, "59_sheet_bruna_pieces.png");
    // The temple's teachings: facing a scroll, then the keeper beside patience.
    let mut tg = temple_test_game();
    let tc = tg.communes[0].controlled;
    let scroll = tg.world.racks.iter().find(|r| r.piece == "teaching_music").map(|r| (r.x, r.y)).expect("music's scroll");
    (tg.monsters[tc].x, tg.monsters[tc].y, tg.monsters[tc].facing) = (scroll.0, scroll.1 + 1, game::Dir::Up);
    tg.monsters[tc].bubble = None;
    shot(&tg, "60_temple_scroll.png");
    let keeper = tg.world.townsfolk.iter().find(|t| t.kind == world::Folk::Keeper).map(|t| (t.x, t.y)).expect("the keeper");
    (tg.monsters[tc].x, tg.monsters[tc].y, tg.monsters[tc].facing) = (keeper.0, keeper.1 - 1, game::Dir::Down);
    shot(&tg, "61_temple_keeper.png");
    // Each working townsperson's secret, on a stand by their work: facing it, then them.
    let mut wg = Game::new();
    let wc = wg.communes[0].controlled;
    wg.communes[0].renown = 150.0;
    for (n, w) in wg.world.workers.clone().iter().enumerate() {
        let (sx, sy) = w.secret.0;
        let open = |g: &Game, (x, y): (i32, i32)| g.world.walkable(x, y) && g.monster_at(x, y).is_none() && g.world.shop_at_spot(x, y).is_none();
        let d = [game::Dir::Down, game::Dir::Left, game::Dir::Right, game::Dir::Up].into_iter().find(|d| open(&wg, (sx - d.delta().0, sy - d.delta().1))).expect("open ground by the stand");
        (wg.monsters[wc].x, wg.monsters[wc].y, wg.monsters[wc].facing) = (sx - d.delta().0, sy - d.delta().1, d);
        wg.monsters[wc].bubble = None;
        shot(&wg, &format!("{}_{}_secret.png", 62 + 2 * n, w.name.to_lowercase().replace(' ', "_")));
        let j = wg.monsters.iter().position(|m| m.worker == Some(n)).expect("their monster");
        let (jx, jy) = (wg.monsters[j].x, wg.monsters[j].y);
        let d = [game::Dir::Left, game::Dir::Right, game::Dir::Down, game::Dir::Up].into_iter().find(|d| open(&wg, (jx - d.delta().0, jy - d.delta().1))).expect("open ground by them");
        (wg.monsters[wc].x, wg.monsters[wc].y, wg.monsters[wc].facing) = (jx - d.delta().0, jy - d.delta().1, d);
        wg.communes[0].idea_turn = (0..12).find(|&t| { wg.communes[0].idea_turn = t; wg.worker_says(j, 0).iter().any(|l| l.contains("SCROLL NEARBY")) }).expect("a word about the scroll");
        shot(&wg, &format!("{}_{}_card.png", 63 + 2 * n, w.name.to_lowercase().replace(' ', "_")));
    }
    // Things for hauling, on stands outside the outfitter along the path; then a goblin and a
    // cyclops with all three on, a pole in hand.
    let mut hg = Game::new();
    let hc = hg.communes[0].controlled;
    hg.communes[0].renown = 150.0;
    let pack = hg.world.racks.iter().find(|r| r.piece == "backpack").map(|r| (r.x, r.y)).expect("the backpack's stand");
    (hg.monsters[hc].x, hg.monsters[hc].y, hg.monsters[hc].facing) = (pack.0 - 1, pack.1, game::Dir::Right);
    hg.monsters[hc].bubble = None;
    shot(&hg, "66_hauling_stands.png");
    let cyclops = hg.monsters.iter().position(|m| m.species == game::Species::Cyclops && m.worker.is_none()).expect("a cyclops");
    for (i, at) in [(hc, (pack.0 - 1, pack.1 + 4)), (cyclops, (pack.0 - 2, pack.1 + 4))] {
        let m = &mut hg.monsters[i];
        (m.x, m.y, m.facing, m.flip) = (at.0, at.1, game::Dir::Right, false);
        m.gear = vec!["backpack".into(), "feathered_cap".into(), "tool_belt".into()];
        hg.fit_hands(i);
        let m = &mut hg.monsters[i];
        m.hands[0] = Some(game::Held::Piece("shoulder_pole".into()));
        m.hands[2] = Some(game::Held::Tool(econ::plain(&hg.data, "pick", "copper_ingot")));
    }
    shot(&hg, "67_hauling_worn.png");
    // The sheet of a monster with a tool belt: a pick and a shovel in hand, the belt's hand empty.
    let mut bg = Game::new();
    let bc = bg.communes[0].controlled;
    bg.monsters[bc].gear = vec!["tool_belt".into()];
    bg.fit_hands(bc);
    bg.monsters[bc].hands[0] = Some(game::Held::Tool(econ::plain(&bg.data, "pick", "copper_ingot")));
    bg.monsters[bc].hands[1] = Some(game::Held::Tool(econ::plain(&bg.data, "shovel", "copper_ingot")));
    bg.script("c");
    shot(&bg, "68_sheet_tool_belt.png");
    // The fullest sheet: three hands and every slot full, renown earned. What fits above the
    // keys is shown; how seasoned it is waits.
    bg.monsters[bc].gear = vec!["tool_belt".into(), "feathered_cap".into(), "headband".into(), "secret_prospecting".into()];
    bg.monsters[bc].hands[2] = Some(game::Held::Piece("shoulder_pole".into()));
    bg.monsters[bc].earned = 400.0;
    shot(&bg, "69_sheet_full.png");
    // The work song: its stand beside the temple's bell frame, then Bruna singing as she spins.
    let mut wsg = temple_test_game();
    let wc = wsg.communes[0].controlled;
    let song = wsg.world.racks.iter().find(|r| r.piece == "work_song").map(|r| (r.x, r.y)).expect("the work song's stand");
    (wsg.monsters[wc].x, wsg.monsters[wc].y, wsg.monsters[wc].facing) = (song.0, song.1 - 1, game::Dir::Down);
    wsg.monsters[wc].bubble = None;
    shot(&wsg, "70_work_song_stand.png");
    let mut ng = Game::new();
    let slivers = plain_work(&ng, ng.monsters[1].id, "cotton_slivers", &["cotton"], 1, &["cotton_bolls"]);
    ng.world.items.push(world::Item::new(30, 9, slivers));
    ng.communes[0].controlled = 1;
    (ng.monsters[1].x, ng.monsters[1].y, ng.monsters[1].facing) = (30, 8, game::Dir::Down);
    ng.monsters[1].gear = vec!["work_song".into()];
    ng.press(0, Input { use_pressed: true, ..Input::default() });
    for _ in 0..24 {
        ng.update(&[]);
    }
    shot(&ng, "71_singing_at_work.png");
    println!("exported screenshots to {} ({} fabric stored)", dir.display(), g.cloth_stored());
}
