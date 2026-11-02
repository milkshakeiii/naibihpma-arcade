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
mod inspect;
mod knot;
mod motion;
mod net;
mod palette;
mod render;
mod save;
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

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let after = |flag: &str| args.iter().position(|a| a == flag).and_then(|i| args.get(i + 1)).cloned();
    if args.iter().any(|a| a == "--export-screenshots") {
        export_screenshots(&after("--export-screenshots").map_or_else(|| PathBuf::from("screenshots"), PathBuf::from));
        return;
    }
    if args.iter().any(|a| a == "--server") {
        serve(after("--port"), after("--world"));
        return;
    }
    #[cfg(windows)]
    window::run(&mut audio::Audio::new(), args.iter().any(|a| a == "--windowed"), after("--connect"), after("--player"));
    #[cfg(not(windows))]
    eprintln!("Here the game runs only as a server: monster_workshop --server [--port N] [--world PATH]");
}

/// A server for players to connect to, until STOP is typed. The world is saved at `world`
/// (by default `server.json` beside the saved worlds), and started there if it's new.
fn serve(port: Option<String>, world: Option<String>) {
    let port = match port.map(|p| p.parse::<u16>()) {
        None => net::DEFAULT_PORT,
        Some(Ok(p)) => p,
        Some(Err(_)) => return eprintln!("--port wants a number"),
    };
    let path = world.map_or_else(|| save::worlds_dir().join("server.json"), PathBuf::from);
    let game = if path.exists() {
        match save::load(&path) {
            Ok(g) => g,
            Err(e) => return eprintln!("{}: {e}", path.display()),
        }
    } else {
        Game::new()
    };
    let server = match std::net::TcpListener::bind(("0.0.0.0", port)).and_then(|l| Server::new(game, path.clone(), l)) {
        Ok(s) => s,
        Err(e) => return eprintln!("can't take players on port {port}: {e}"),
    };
    println!("Serving {} on port {port}. Type STOP to save and stop.", path.display());
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
    server.run(&stop);
    println!("Saved {}.", path.display());
}

/// Headless scripted playthrough of the Fabric chain, saving 2x PNGs at key moments.
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
    let board = g.world.boards[0].clone();
    (g.monsters[0].x, g.monsters[0].y) = (board.x, board.y + 1);
    g.monsters[0].facing = game::Dir::Up;
    g.world.boards[0].have = board.need - 1;
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
    (g.monsters[0].x, g.monsters[0].y, g.monsters[0].facing) = (154, 6, game::Dir::Up);
    g.run_ticks(3);
    save(&g, "25_indigo_house_empty.png");
    put(&mut g, (140, 4), "fabric", &["cotton"]);
    put(&mut g, (142, 4), "indigo", &["indigofera"]);
    (g.monsters[0].x, g.monsters[0].y, g.monsters[0].facing) = (141, 4, game::Dir::Down);
    use_now(&mut g);
    g.run_ticks(40);
    put(&mut g, (141, 6), "dyed_fabric", &["cotton", "indigo"]);
    (g.monsters[0].x, g.monsters[0].y, g.monsters[0].facing) = (141, 5, game::Dir::Down);
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
        let s = sprites.item(id);
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
    println!("exported screenshots to {} ({} fabric stored)", dir.display(), g.cloth_stored());
}
