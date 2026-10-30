mod audio;
mod framebuffer;
mod game;
mod level;
mod palette;
mod render;
mod sprites;

use audio::Audio;
use framebuffer::{save_argb_png, Framebuffer, FB_W};
use game::{Game, Input, Request, VIEW_H, VIEW_W};
use level::{Level, TS};
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
    GetStockObject, PatBlt, SetStretchBltMode, StretchDIBits, BITMAPINFO, BITMAPINFOHEADER, BI_RGB,
    BLACKNESS, BLACK_BRUSH, COLORONCOLOR, DIB_RGB_COLORS, SRCCOPY,
};
use winapi::um::winuser::*;

const LEVEL_SRC: &str = include_str!("../levels/underbrush.lvl");
const DESKTOP_MASTERS: &str = r"C:\Users\User\Desktop\albums\s3_thenechantedforestofnaibihpma\mastered_resonance";

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let mut level = match Level::parse(LEVEL_SRC) {
        Ok(l) => l,
        Err(e) => {
            eprintln!("level error: {e}");
            return;
        }
    };
    if let Some(i) = args.iter().position(|a| a == "--export-screenshots") {
        let dir = args.get(i + 1).map(PathBuf::from).unwrap_or_else(|| PathBuf::from("screenshots"));
        export_screenshots(level, &dir);
        return;
    }

    let music = find_music(&level, &args);
    let mut audio = Audio::new(music);
    if let Some(len) = audio.track_len_ms() {
        level.len_ms = len;
    }
    let music_missing = !audio.has_music();
    let mut game = Game::new(level, music_missing, true);
    run_window(&mut game, &mut audio, args.iter().any(|a| a == "--windowed"));
}

/// `--music <file>` wins; otherwise the level's file names are looked up in the usual places.
fn find_music(level: &Level, args: &[String]) -> Option<PathBuf> {
    if let Some(i) = args.iter().position(|a| a == "--music") {
        return args.get(i + 1).map(PathBuf::from).filter(|p| p.is_file());
    }
    let exe_dir = std::env::current_exe().ok().and_then(|p| p.parent().map(Path::to_path_buf));
    let mut dirs: Vec<PathBuf> = vec![
        PathBuf::from("assets/music"),
        PathBuf::from("starpond/assets/music"),
        PathBuf::from(DESKTOP_MASTERS),
        PathBuf::from("../creaturecollector/assets/music"),
        PathBuf::from("creaturecollector/assets/music"),
    ];
    if let Some(d) = exe_dir {
        dirs.insert(0, d.join("assets/music"));
    }
    dirs.iter()
        .flat_map(|d| level.music.iter().map(move |name| d.join(name)))
        .find(|p| p.is_file())
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
        let class_name = wide("StarPondWindowClass");
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
        let title = wide("STAR POND - THE UNDERBRUSH [400x300 64-COLOR VGA | 15 FPS]");
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
        let mut fb = Framebuffer::new();
        let mut scaled: Vec<u32> = Vec::new();
        let mut requests = Vec::new();
        let mut sfx = Vec::new();
        let mut latched = Input::default();
        let (mut e_jump, mut e_down, mut e_restart, mut e_skip, mut e_esc, mut e_f2, mut e_f3) =
            (Edge::default(), Edge::default(), Edge::default(), Edge::default(), Edge::default(), Edge::default(), Edge::default());

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

            // 250 Hz polling: held keys are OR-ed and rising edges latched until the next tick.
            let left = key_down(VK_LEFT) || key_down('A' as i32);
            let right = key_down(VK_RIGHT) || key_down('D' as i32);
            let up = key_down(VK_UP) || key_down('W' as i32);
            let down = key_down(VK_DOWN) || key_down('S' as i32);
            let jump = key_down('Z' as i32) || key_down(VK_SPACE) || key_down('X' as i32) || key_down('K' as i32);
            latched.left |= left;
            latched.right |= right;
            latched.up |= up;
            latched.down |= down;
            latched.jump |= jump;
            latched.jump_pressed |= e_jump.rose(jump);
            latched.down_pressed |= e_down.rose(down);
            latched.restart_pressed |= e_restart.rose(key_down('R' as i32));
            latched.skip_pressed |= e_skip.rose(key_down(VK_NEXT) || key_down('N' as i32));
            latched.quit_pressed |= e_esc.rose(key_down(VK_ESCAPE));
            latched.debug_pressed |= e_f3.rose(key_down(VK_F3));
            if e_f2.rose(key_down(VK_F2)) {
                game.crt = !game.crt;
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

            game.update(&latched, audio.song_ms(), &mut requests, &mut sfx);
            latched = Input::default();
            for r in requests.drain(..) {
                match r {
                    Request::PlayFrom(ms) => audio.play_from(ms),
                    Request::Stop => audio.stop(),
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
            let (from, to, mix) = game.palette_pair();
            let (dw, dh) = fb.present(&mut scaled, scale, from, to, mix, game.crt);
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
        audio.stop();
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

/// Headless run of the whole song with scripted positions, saving 2x PNGs at key moments.
fn export_screenshots(level: Level, dir: &Path) {
    let _ = std::fs::create_dir_all(dir);
    let sprites = Sprites::new();
    let mut fb = Framebuffer::new();
    let mut game = Game::new(level, false, false);
    let save = |g: &Game, fb: &mut Framebuffer, name: &str| {
        render::draw(g, &sprites, fb);
        let (from, to, mix) = g.palette_pair();
        fb.save_png(&dir.join(name), from, to, mix);
    };
    save(&game, &mut fb, "01_title.png");

    let mut req = Vec::new();
    let mut sfx = Vec::new();
    game.start_run(&mut req);
    // (time, tile to stand Twen on, file)
    let shots: [(u32, (i32, i32), &str); 8] = [
        (40_000, (10, 3), "02_canopy_mist.png"),
        (100_000, (24, 16), "03_cavern_drone.png"),
        (125_000, (36, 2), "04_harp_clearing_sun.png"),
        (190_000, (47, 18), "05_swamp_night.png"),
        (210_000, (11, 18), "06_bog_flood.png"),
        (236_000, (24, 5), "07_canopy_lament_echo.png"),
        (290_000, (27, 8), "08_harp_finale.png"),
        (352_000, (10, 18), "09_tally.png"),
    ];
    let mut t: u64 = 0;
    for (ms, (tx, ty), name) in shots {
        loop {
            let now = (t * 200 / 3) as u32;
            if now > ms {
                break;
            }
            game.update(&Input::default(), now, &mut req, &mut sfx);
            t += 1;
        }
        if game.phase == game::Phase::Playing {
            game.player.x = tx * TS + (TS - game::PW) / 2;
            game.player.y = (ty + 1) * TS - game::PH;
            game.player.facing = 1;
            game.snap_camera();
        }
        save(&game, &mut fb, name);
    }

    // Whole map at the lament echo, stitched from playfield-sized views.
    let mut game2 = Game::new(Level::parse(LEVEL_SRC).expect("level"), false, false);
    game2.start_run(&mut req);
    let mut t: u64 = 0;
    while (t * 200 / 3) as u32 <= 237_000 {
        game2.update(&Input::default(), (t * 200 / 3) as u32, &mut req, &mut sfx);
        t += 1;
    }
    game2.mood_mix = 16;
    let (mw, mh) = ((game2.level.w * TS) as usize, (game2.level.h * TS) as usize);
    let mut map = vec![0u32; mw * mh];
    let mut buf = Vec::new();
    for cy in (0..mh as i32).step_by(VIEW_H as usize) {
        for cx in (0..mw as i32).step_by(VIEW_W as usize) {
            game2.cam_x = cx;
            game2.cam_y = cy;
            render::draw(&game2, &sprites, &mut fb);
            let (from, to, mix) = game2.palette_pair();
            fb.present(&mut buf, 1, from, to, mix, false);
            for y in 0..VIEW_H as usize {
                let src = &buf[y * FB_W..y * FB_W + VIEW_W as usize];
                let dst = (cy as usize + y) * mw + cx as usize;
                map[dst..dst + VIEW_W as usize].copy_from_slice(src);
            }
        }
    }
    save_argb_png(&dir.join("10_full_map_lament.png"), &map, mw, mh);
    println!("exported screenshots to {}", dir.display());
}
