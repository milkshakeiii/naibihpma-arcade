mod audio;
#[allow(dead_code)] // the schema mirrors the JSON; some fields are read in later stages
mod data;
#[allow(dead_code)] // tiers, value, weight and the forecast feed stages 3 and 4
mod econ;
#[allow(dead_code)] // shared copy of the arcade framebuffer; line/circle unused here
mod framebuffer;
mod game;
mod inspect;
mod palette;
mod render;
mod save;
mod sprites;
mod tutorial;
mod work;
mod world;

use audio::Audio;
use framebuffer::Framebuffer;
use game::{Game, Input, Request};
use sprites::Sprites;
use std::ffi::OsStr;
use std::mem::{size_of, zeroed};
use std::os::windows::ffi::OsStrExt;
use std::path::{Path, PathBuf};
use std::ptr::null_mut;
use std::time::{Duration, Instant};

use winapi::shared::minwindef::{LPARAM, LRESULT, UINT, WPARAM};
use winapi::shared::windef::{HDC, HWND, RECT};
use winapi::um::libloaderapi::GetModuleHandleW;
use winapi::um::wingdi::{
    GetStockObject, PatBlt, SetStretchBltMode, StretchDIBits, BITMAPINFO, BITMAPINFOHEADER, BI_RGB, BLACKNESS,
    BLACK_BRUSH, COLORONCOLOR, DIB_RGB_COLORS, SRCCOPY,
};
use winapi::um::winuser::*;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if let Some(i) = args.iter().position(|a| a == "--export-screenshots") {
        let dir = args.get(i + 1).map(PathBuf::from).unwrap_or_else(|| PathBuf::from("screenshots"));
        export_screenshots(&dir);
        return;
    }
    let mut audio = Audio::new();
    run_window(&mut audio, args.iter().any(|a| a == "--windowed"));
}

/// The title screen's list of worlds, or the world being played.
enum Screen {
    Title { worlds: Vec<PathBuf>, sel: usize, error: Option<String> },
    Play { game: Box<Game>, path: PathBuf, since_save: u32 },
}

impl Screen {
    fn title(error: Option<String>) -> Self {
        Screen::Title { worlds: save::list_worlds(), sel: 0, error }
    }
}

fn key_down(vk: i32) -> bool {
    unsafe { (GetAsyncKeyState(vk) as u16 & 0x8000) != 0 }
}

/// Rising-edge tracker for one logical button polled at 250 Hz.
#[derive(Default)]
struct Edge(bool);

impl Edge {
    fn rose(&mut self, now: bool) -> bool {
        let r = now && !self.0;
        self.0 = now;
        r
    }
}

fn run_window(audio: &mut Audio, windowed: bool) {
    unsafe {
        SetProcessDPIAware();
        let hinstance = GetModuleHandleW(null_mut());
        let wide = |s: &str| OsStr::new(s).encode_wide().chain(Some(0)).collect::<Vec<u16>>();
        let class_name = wide("MonsterWorkshopWindowClass");
        let wc = WNDCLASSW {
            style: CS_HREDRAW | CS_VREDRAW | CS_OWNDC,
            lpfnWndProc: Some(wnd_proc),
            cbClsExtra: 0,
            cbWndExtra: 0,
            hInstance: hinstance,
            hIcon: null_mut(),
            hCursor: LoadCursorW(null_mut(), IDC_ARROW),
            hbrBackground: GetStockObject(BLACK_BRUSH as i32) as _,
            lpszMenuName: null_mut(),
            lpszClassName: class_name.as_ptr(),
        };
        RegisterClassW(&wc);

        let (sw, sh) = (GetSystemMetrics(SM_CXSCREEN), GetSystemMetrics(SM_CYSCREEN));
        let (style, x, y, w, h) = if windowed {
            let n = Framebuffer::integer_scale_for_viewport(sw * 4 / 5, sh * 4 / 5) as i32;
            let mut r = RECT { left: 0, top: 0, right: 400 * n, bottom: 300 * n };
            AdjustWindowRect(&mut r, WS_OVERLAPPEDWINDOW, 0);
            let (w, h) = (r.right - r.left, r.bottom - r.top);
            (WS_OVERLAPPEDWINDOW | WS_VISIBLE, (sw - w) / 2, (sh - h) / 2, w, h)
        } else {
            (WS_POPUP | WS_VISIBLE, 0, 0, sw, sh)
        };
        let title = wide("MONSTER WORKSHOP [400x300 64-COLOR VGA | 15 FPS]");
        let hwnd = CreateWindowExW(0, class_name.as_ptr(), title.as_ptr(), style, x, y, w, h, null_mut(), null_mut(), hinstance, null_mut());
        if hwnd.is_null() {
            return;
        }
        let hdc: HDC = GetDC(hwnd);
        SetStretchBltMode(hdc, COLORONCOLOR);
        let mut bmi: BITMAPINFO = zeroed();
        bmi.bmiHeader.biSize = size_of::<BITMAPINFOHEADER>() as u32;
        bmi.bmiHeader.biPlanes = 1;
        bmi.bmiHeader.biBitCount = 32;
        bmi.bmiHeader.biCompression = BI_RGB;

        let sprites = Sprites::new();
        let pal = palette::build();
        let mut fb = Framebuffer::new();
        let mut scaled: Vec<u32> = Vec::new();
        let mut requests = Vec::new();
        let mut sfx = Vec::new();
        let mut latched = Input::default();
        let mut screen = Screen::title(None);
        let mut crt = true;
        let mut dirs: [Edge; 4] = Default::default();
        let (mut e_use, mut e_rec, mut e_play, mut e_tab, mut e_cancel, mut e_f1, mut e_f2, mut e_esc) = Default::default();

        let tick = Duration::from_micros(66_667);
        let mut next_tick = Instant::now();
        let mut msg: MSG = zeroed();

        'running: loop {
            while PeekMessageW(&mut msg, null_mut(), 0, 0, PM_REMOVE) != 0 {
                if msg.message == WM_QUIT {
                    break 'running;
                }
                TranslateMessage(&msg);
                DispatchMessageW(&msg);
            }

            // 250 Hz polling, only while the window has focus: held keys are OR-ed and rising
            // edges latched until the next tick, so a quick tap between ticks is never lost.
            if GetForegroundWindow() == hwnd {
                let held = [
                    key_down(VK_UP) || key_down('W' as i32),
                    key_down(VK_DOWN) || key_down('S' as i32),
                    key_down(VK_LEFT) || key_down('A' as i32),
                    key_down(VK_RIGHT) || key_down('D' as i32),
                ];
                for k in 0..4 {
                    latched.held[k] |= held[k];
                    latched.pressed[k] |= dirs[k].rose(held[k]);
                }
                latched.use_pressed |= Edge::rose(&mut e_use, key_down(VK_SPACE) || key_down(VK_RETURN));
                latched.inspect |= key_down('E' as i32);
                latched.pull |= key_down(VK_SHIFT);
                latched.rec_pressed |= Edge::rose(&mut e_rec, key_down('R' as i32));
                latched.play_pressed |= Edge::rose(&mut e_play, key_down('P' as i32));
                latched.tab_pressed |= Edge::rose(&mut e_tab, key_down(VK_TAB));
                latched.tab_held |= key_down(VK_TAB);
                latched.cancel_pressed |= Edge::rose(&mut e_cancel, key_down(VK_BACK));
                latched.help_pressed |= Edge::rose(&mut e_f1, key_down(VK_F1));
                latched.quit_pressed |= Edge::rose(&mut e_esc, key_down(VK_ESCAPE));
                if Edge::rose(&mut e_f2, key_down(VK_F2)) {
                    crt = !crt;
                }
            }

            let now = Instant::now();
            if now < next_tick {
                std::thread::sleep(Duration::from_millis(4));
                continue;
            }
            next_tick += tick;
            if now > next_tick + tick * 4 {
                next_tick = now + tick;
            }

            let inp = std::mem::take(&mut latched);
            match &mut screen {
                Screen::Title { worlds, sel, error } => {
                    let count = worlds.len() + 1; // "NEW WORLD" first, then saved worlds
                    if inp.pressed[0] {
                        *sel = (*sel + count - 1) % count;
                    }
                    if inp.pressed[1] {
                        *sel = (*sel + 1) % count;
                    }
                    if inp.quit_pressed {
                        break 'running;
                    }
                    let names: Vec<String> = worlds.iter().map(|p| save::display_name(p)).collect();
                    render::draw_title(&mut fb, &names, *sel, error.as_deref());
                    if inp.use_pressed {
                        let started = if *sel == 0 {
                            Ok((Game::new(), save::new_world_path()))
                        } else {
                            let path = worlds[*sel - 1].clone();
                            save::load(&path).map(|g| (g, path))
                        };
                        screen = match started {
                            Ok((game, path)) => Screen::Play { game: Box::new(game), path, since_save: 0 },
                            Err(e) => Screen::title(Some(e)),
                        };
                    }
                }
                Screen::Play { game, path, since_save } => {
                    game.update(&inp, &mut requests, &mut sfx);
                    *since_save += 1;
                    let quitting = requests.drain(..).any(|r| matches!(r, Request::Quit));
                    if quitting || *since_save >= save::AUTOSAVE_TICKS {
                        *since_save = 0;
                        if let Err(e) = save::save(game, path) {
                            eprintln!("save failed: {e}");
                        }
                    }
                    if quitting {
                        break 'running;
                    }
                    for s in sfx.drain(..) {
                        audio.play_sfx(s);
                    }
                    render::draw(game, &sprites, &mut fb);
                }
            }
            let mut rc: RECT = zeroed();
            GetClientRect(hwnd, &mut rc);
            let (cw, ch) = ((rc.right - rc.left).max(1), (rc.bottom - rc.top).max(1));
            let scale = Framebuffer::integer_scale_for_viewport(cw, ch);
            let (dw, dh) = fb.present(&mut scaled, scale, &pal, &pal, 0, crt);
            let (dw, dh) = (dw as i32, dh as i32);
            let (dx, dy) = ((cw - dw) / 2, (ch - dh) / 2);
            bmi.bmiHeader.biWidth = dw;
            bmi.bmiHeader.biHeight = -dh;
            if dy > 0 {
                PatBlt(hdc, 0, 0, cw, dy, BLACKNESS);
                PatBlt(hdc, 0, dy + dh, cw, ch - dy - dh, BLACKNESS);
            }
            if dx > 0 {
                PatBlt(hdc, 0, 0, dx, ch, BLACKNESS);
                PatBlt(hdc, dx + dw, 0, cw - dx - dw, ch, BLACKNESS);
            }
            StretchDIBits(hdc, dx, dy, dw, dh, 0, 0, dw, dh, scaled.as_ptr() as *const _, &bmi, DIB_RGB_COLORS, SRCCOPY);
        }
        // Closing the window also saves the world being played.
        if let Screen::Play { game, path, .. } = &screen {
            let _ = save::save(game, path);
        }
        ReleaseDC(hwnd, hdc);
    }
}

unsafe extern "system" fn wnd_proc(hwnd: HWND, msg: UINT, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    match msg {
        WM_DESTROY | WM_CLOSE => {
            PostQuitMessage(0);
            0
        }
        _ => DefWindowProcW(hwnd, msg, wparam, lparam),
    }
}

/// Headless scripted playthrough of the Fabric chain, saving 2x PNGs at key moments.
fn export_screenshots(dir: &Path) {
    let _ = std::fs::create_dir_all(dir);
    let sprites = Sprites::new();
    let pal = palette::build();
    let mut fb = Framebuffer::new();
    let mut g = Game::new();
    let mut save = |g: &Game, name: &str| {
        render::draw(g, &sprites, &mut fb);
        fb.save_png(&dir.join(name), &pal, &pal, 0);
    };
    let use_now = |g: &mut Game| g.update(&Input { use_pressed: true, ..Input::default() }, &mut Vec::new(), &mut Vec::new());

    g.run_ticks(2);
    save(&g, "01_fields.png");
    g.script("RRRRRR");
    use_now(&mut g);
    g.run_ticks(12);
    save(&g, "02_harvesting.png");
    g.run_ticks(30);
    g.inspecting = true;
    save(&g, "03_examine_boll.png");
    g.inspecting = false;
    g.script("RRR");
    g.script("t");
    g.run_ticks(10);
    g.script("RDRRU");
    use_now(&mut g);
    g.run_ticks(15);
    save(&g, "04_workbench.png");
    g.run_ticks(40);
    g.script(&format!("{}UR", "R".repeat(15)));
    g.inspecting = true;
    save(&g, "05_spinnery_forecast.png");
    g.card_page = 1;
    save(&g, "05b_spinnery_details.png");
    g.card_page = 0;
    g.inspecting = false;
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
    g.monsters[g.controlled].facing = game::Dir::Left;
    g.inspecting = true;
    save(&g, "09_examine_bruna.png");
    g.card_page = 1;
    save(&g, "09b_bruna_details.png");
    g.card_page = 0;
    g.inspecting = false;
    g.controlled = 0;
    let stand = g.world.stands[1];
    (g.monsters[0].x, g.monsters[0].y) = (stand.0, stand.1 - 1);
    g.monsters[0].facing = game::Dir::Down;
    g.renown = 60.0;
    save(&g, "10_gathering_hall.png");
    g.help = true;
    save(&g, "11_help.png");
    g.help = false;

    // M2: the ferry's board, the ferry running, the wilderness and Tanglemere.
    g.renown = 0.0;
    let board = g.world.boards[0].clone();
    (g.monsters[0].x, g.monsters[0].y) = (board.x, board.y + 1);
    g.monsters[0].facing = game::Dir::Up;
    g.world.boards[0].have = board.need - 1;
    g.run_ticks(2);
    g.inspecting = true;
    save(&g, "13_ferry_board.png");
    g.inspecting = false;
    let stored = g.world.bins.iter().flat_map(|b| &b.stock).find(|d| d.good == "fabric").cloned().expect("fabric in the cloth store");
    let mut rope = stored.clone();
    rope.good = "rope".into();
    g.world.items.push(world::Item::new(board.x, board.y + 1, rope));
    (g.monsters[0].x, g.monsters[0].y) = (board.x, board.y + 2);
    g.script("U");
    g.run_ticks(24);
    save(&g, "14_the_ferry_runs.png");
    let f = g.world.ferry.clone().expect("a ferry");
    let mut cloth = stored;
    cloth.quality = 1.1;
    g.world.items.push(world::Item::new(f.west - 1, f.y, cloth));
    (g.monsters[0].x, g.monsters[0].y) = (f.west - 2, f.y);
    g.script("RR");
    for _ in 0..200 {
        if g.world.ferry.as_ref().is_some_and(|f| f.moving && f.x == f.west + 2) {
            break;
        }
        g.run_ticks(1);
    }
    save(&g, "15_ferry_crossing.png");
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
    g.roster = true;
    save(&g, "19_roster_and_edge_markers.png");
    g.roster = false;

    // Rope at the gate bench, and a pull out of a corner.
    let mut thread = g.world.bins.iter().flat_map(|b| &b.stock).find(|d| d.good == "fabric").cloned().expect("fabric");
    thread.good = "thread".into();
    g.world.items.push(world::Item::new(40, 8, thread.clone()));
    g.world.items.push(world::Item::new(41, 8, thread));
    (g.monsters[0].x, g.monsters[0].y, g.monsters[0].facing) = (42, 8, game::Dir::Down);
    g.run_ticks(2);
    g.inspecting = true;
    save(&g, "20_gate_bench.png");
    g.inspecting = false;
    let mut stuck = g.world.bins.iter().flat_map(|b| &b.stock).find(|d| d.good == "fabric").cloned().expect("fabric");
    stuck.quality = 0.8;
    g.world.items.push(world::Item::new(1, 1, stuck));
    (g.monsters[0].x, g.monsters[0].y, g.monsters[0].facing) = (2, 1, game::Dir::Left);
    g.script("L");
    save(&g, "21_stuck.png");
    let mut inp = Input { pull: true, ..Input::default() };
    inp.pressed[3] = true;
    inp.held[3] = true;
    g.update(&inp, &mut Vec::new(), &mut Vec::new());
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
    g.inspecting = true;
    save(&g, "23_kiln_forecast.png");
    g.inspecting = false;
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
    g.inspecting = true;
    save(&g, "26_dye_house.png");
    g.inspecting = false;
    (g.monsters[0].x, g.monsters[0].y, g.monsters[0].facing) = (78, 19, game::Dir::Left);
    g.run_ticks(3);
    g.inspecting = true;
    save(&g, "27_limestone.png");
    g.inspecting = false;
    render::draw_title(&mut fb, &["WORLD 2".to_string(), "WORLD 1".to_string()], 1, None);
    fb.save_png(&dir.join("12_title.png"), &pal, &pal, 0);
    println!("exported screenshots to {} ({} fabric stored)", dir.display(), g.cloth_stored());
}
