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
    let mut game = Game::new();
    let mut audio = Audio::new();
    run_window(&mut game, &mut audio, args.iter().any(|a| a == "--windowed"));
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

fn run_window(game: &mut Game, audio: &mut Audio, windowed: bool) {
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
        let title = wide("MONSTER WORKSHOP - FEEL PROTOTYPE [400x300 64-COLOR VGA | 15 FPS]");
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
                latched.use_pressed |= Edge::rose(&mut e_use, key_down(VK_SPACE));
                latched.inspect |= key_down('E' as i32);
                latched.rec_pressed |= Edge::rose(&mut e_rec, key_down('R' as i32));
                latched.play_pressed |= Edge::rose(&mut e_play, key_down('P' as i32));
                latched.tab_pressed |= Edge::rose(&mut e_tab, key_down(VK_TAB));
                latched.cancel_pressed |= Edge::rose(&mut e_cancel, key_down(VK_BACK));
                latched.help_pressed |= Edge::rose(&mut e_f1, key_down(VK_F1));
                latched.quit_pressed |= Edge::rose(&mut e_esc, key_down(VK_ESCAPE));
                if Edge::rose(&mut e_f2, key_down(VK_F2)) {
                    game.crt = !game.crt;
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

            game.update(&latched, &mut requests, &mut sfx);
            latched = Input::default();
            for r in requests.drain(..) {
                match r {
                    Request::Quit => break 'running,
                }
            }
            for s in sfx.drain(..) {
                audio.play_sfx(s);
            }

            render::draw(game, &sprites, &mut fb);
            let mut rc: RECT = zeroed();
            GetClientRect(hwnd, &mut rc);
            let (cw, ch) = ((rc.right - rc.left).max(1), (rc.bottom - rc.top).max(1));
            let scale = Framebuffer::integer_scale_for_viewport(cw, ch);
            let (dw, dh) = fb.present(&mut scaled, scale, &pal, &pal, 0, game.crt);
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
    save(&g, "08_cloth_store.png");
    g.monsters[g.controlled].facing = game::Dir::Left;
    g.inspecting = true;
    save(&g, "09_examine_bruna.png");
    g.inspecting = false;
    g.help = true;
    save(&g, "10_help.png");
    println!("exported screenshots to {} ({} fabric stored)", dir.display(), g.cloth_stored());
}
