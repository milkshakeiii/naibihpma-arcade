mod ai;
mod app;
mod design;
mod export;
mod framebuffer;
mod game;
mod geom;
mod palette;
mod pg;
mod render;
mod rng;
mod ui;
mod world;

use app::{App, Ev, Key};
use framebuffer::{Framebuffer, Sprite};
use game::Game;
use std::ffi::OsStr;
use std::mem::{size_of, zeroed};
use std::os::windows::ffi::OsStrExt;
use std::path::PathBuf;
use std::ptr::null_mut;
use std::sync::atomic::{AtomicI32, AtomicUsize, Ordering};
use std::sync::Mutex;
use std::time::{Duration, Instant};

use winapi::shared::minwindef::{LPARAM, LRESULT, UINT, WPARAM};
use winapi::shared::windef::{HCURSOR, HDC, HWND, POINT, RECT};
use winapi::um::libloaderapi::GetModuleHandleW;
use winapi::um::wingdi::{
    CreateBitmap, CreateDIBSection, DeleteObject, GetStockObject, PatBlt, SetStretchBltMode, StretchDIBits, BITMAPINFO,
    BITMAPINFOHEADER, BI_RGB, BLACKNESS, BLACK_BRUSH, COLORONCOLOR, DIB_RGB_COLORS, SRCCOPY,
};
use winapi::um::winuser::*;

/// The pixel-art pointer as an OS cursor, rebuilt when the integer scale or pointer changes. Like
/// a DOS mouse driver's pointer, it moves independently of the 15 FPS frames.
static CURSOR: AtomicUsize = AtomicUsize::new(0);
/// Typed characters from WM_CHAR (the OS handles key repeat and keyboard layouts).
static TYPED: Mutex<Vec<char>> = Mutex::new(Vec::new());
/// Mouse wheel movement from WM_MOUSEWHEEL, in Windows' units (120 a notch), drained on the tick.
static WHEEL: AtomicI32 = AtomicI32::new(0);

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let arg = |name: &str| args.iter().position(|a| a == name).map(|i| args.get(i + 1).cloned());
    if let Some(dir) = arg("--export-screenshots") {
        export::pg_screenshots(&PathBuf::from(dir.unwrap_or_else(|| "screenshots".into())));
        return;
    }
    if let Some(dir) = arg("--export-toy1") {
        export::screenshots(&PathBuf::from(dir.unwrap_or_else(|| "screenshots/toy1".into())));
        return;
    }
    if let Some(i) = args.iter().position(|a| a == "--pg-trace") {
        let n = |k: usize| args.get(i + k).and_then(|s| s.parse::<i32>().ok()).unwrap_or(1);
        export::pg_trace((n(1) as usize, n(2)), (n(3) as usize, n(4)), n(5).max(1) as u32, n(6).max(1) as u32);
        return;
    }
    if let Some(file) = arg("--deck") {
        export::deck_sheet(file.as_deref());
        return;
    }
    if let Some(i) = args.iter().position(|a| a == "--match-sim") {
        let n = args.get(i + 1).and_then(|s| s.parse().ok()).unwrap_or(4);
        // --bots BLUE,RED picks who plays: ai (the default), clock, planter, expander, summoner.
        let mut bots = [pg::sim::Bot::Ai; 2];
        if let Some(Some(names)) = arg("--bots") {
            for (k, name) in names.split(',').take(2).enumerate() {
                bots[k] = pg::sim::Bot::from_name(name).unwrap_or_else(|| panic!("no bot called {name}"));
            }
        }
        export::match_sim(n, arg("--blue").flatten().as_deref(), arg("--red").flatten().as_deref(), bots);
        return;
    }
    if let Some(i) = args.iter().position(|a| a == "--judge") {
        export::judge(args.get(i + 1).and_then(|s| s.parse().ok()).unwrap_or(5));
        return;
    }
    if arg("--pg-sim").is_some() {
        export::pg_sim();
        return;
    }
    if let Some(i) = args.iter().position(|a| a == "--trace") {
        let seed = args.get(i + 1).and_then(|s| s.parse().ok()).unwrap_or(1);
        export::trace(seed, args.iter().any(|a| a == "--both"));
        return;
    }
    if let Some(i) = args.iter().position(|a| a == "--sim") {
        let n = args.get(i + 1).and_then(|s| s.parse().ok()).unwrap_or(8);
        export::sim(n);
        return;
    }
    let windowed = args.iter().any(|a| a == "--windowed");
    if args.iter().any(|a| a == "--toy1") {
        let seed = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map(|d| d.as_secs()).unwrap_or(1);
        let mut game = Game::new(seed, Some(PathBuf::from("nomindwing_playtest.log")));
        run_window(&mut game, windowed, "NO MIND WING - TOY 1 [400x300 64-COLOR VGA | 15 FPS]");
    } else {
        let mut pg = pg::Pg::new(Some(PathBuf::from("proving_ground_designs.txt")), Some(PathBuf::from("proving_ground.log")));
        run_window(&mut pg, windowed, "NO MIND WING - PROVING GROUND [400x300 64-COLOR VGA | 15 FPS]");
    }
}

fn key_down(vk: i32) -> bool {
    unsafe { (GetAsyncKeyState(vk) as u16 & 0x8000) != 0 }
}

struct Poll {
    vks: &'static [i32],
    key: Key,
    held: bool,
}

fn polls() -> Vec<Poll> {
    let p = |vks: &'static [i32], key: Key| Poll { vks, key, held: false };
    static DIGITS: [[i32; 2]; 9] = [
        ['1' as i32, VK_NUMPAD1],
        ['2' as i32, VK_NUMPAD2],
        ['3' as i32, VK_NUMPAD3],
        ['4' as i32, VK_NUMPAD4],
        ['5' as i32, VK_NUMPAD5],
        ['6' as i32, VK_NUMPAD6],
        ['7' as i32, VK_NUMPAD7],
        ['8' as i32, VK_NUMPAD8],
        ['9' as i32, VK_NUMPAD9],
    ];
    static LETTERS: [[i32; 1]; 26] = {
        let mut l = [[0; 1]; 26];
        let mut i = 0;
        while i < 26 {
            l[i][0] = 'A' as i32 + i as i32;
            i += 1;
        }
        l
    };
    let mut v: Vec<Poll> = (0..9).map(|i| p(&DIGITS[i], Key::Num(i as u8 + 1))).collect();
    v.extend((0..26).map(|i| p(&LETTERS[i], Key::Letter((b'A' + i as u8) as char))));
    v.extend([
        p(&[VK_TAB], Key::Tab),
        p(&[VK_SPACE], Key::Space),
        p(&[VK_PAUSE], Key::Letter('P')),
        p(&[VK_ESCAPE], Key::Esc),
        p(&[VK_F1], Key::Help),
        p(&[VK_BACK], Key::Back),
        p(&[VK_RETURN], Key::Enter),
    ]);
    v
}

/// Builds a hardware cursor from a pointer sprite with every logical pixel an exact NxN block.
unsafe fn make_cursor(spr: &Sprite, pal: &[u32; 64], n: i32, hot: (i32, i32)) -> HCURSOR {
    let (w, h) = (spr.w * n, spr.h * n);
    let mut bmi: BITMAPINFO = zeroed();
    bmi.bmiHeader.biSize = size_of::<BITMAPINFOHEADER>() as u32;
    bmi.bmiHeader.biWidth = w;
    bmi.bmiHeader.biHeight = -h;
    bmi.bmiHeader.biPlanes = 1;
    bmi.bmiHeader.biBitCount = 32;
    bmi.bmiHeader.biCompression = BI_RGB;
    let screen = GetDC(null_mut());
    let mut bits: *mut winapi::ctypes::c_void = null_mut();
    let color = CreateDIBSection(screen, &bmi, DIB_RGB_COLORS, &mut bits, null_mut(), 0);
    ReleaseDC(null_mut(), screen);
    if color.is_null() || bits.is_null() {
        return null_mut();
    }
    let px = bits as *mut u32;
    for y in 0..h {
        for x in 0..w {
            let c = spr.px[((y / n) * spr.w + x / n) as usize];
            *px.add((y * w + x) as usize) = if c == 0 { 0 } else { pal[c as usize] | 0xFF00_0000 };
        }
    }
    let stride = ((w + 15) / 16 * 2) as usize;
    let mask_bits = vec![0u8; stride * h as usize];
    let mask = CreateBitmap(w, h, 1, 1, mask_bits.as_ptr() as *const _);
    let mut info = ICONINFO { fIcon: 0, xHotspot: (hot.0 * n + n / 2) as u32, yHotspot: (hot.1 * n + n / 2) as u32, hbmMask: mask, hbmColor: color };
    if hot == (0, 0) {
        info.xHotspot = 0;
        info.yHotspot = 0;
    }
    let cursor = CreateIconIndirect(&mut info);
    DeleteObject(color as _);
    DeleteObject(mask as _);
    cursor
}

fn run_window(game: &mut dyn App, windowed: bool, title: &str) {
    unsafe {
        SetProcessDPIAware();
        let hinstance = GetModuleHandleW(null_mut());
        let wide = |s: &str| OsStr::new(s).encode_wide().chain(Some(0)).collect::<Vec<u16>>();
        let class_name = wide("NoMindWingWindowClass");
        let wc = WNDCLASSW {
            style: CS_HREDRAW | CS_VREDRAW | CS_OWNDC,
            lpfnWndProc: Some(wnd_proc),
            cbClsExtra: 0,
            cbWndExtra: 0,
            hInstance: hinstance,
            hIcon: null_mut(),
            hCursor: null_mut(),
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
        let title = wide(title);
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

        let pointers = [(ui::pointer(), (0, 0)), (ui::crosshair(), (5, 5))];
        let mut fb = Framebuffer::new();
        let mut scaled: Vec<u32> = Vec::new();
        let mut polls = polls();
        let mut e_f2 = false;
        let (mut lmb, mut rmb) = (false, false);
        let mut last_mouse = (-1, -1);
        let mut view = (0i32, 0i32, 0i32);
        let mut shown_pointer = u8::MAX;
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

            // 250 Hz input: key edges apply immediately, held movement is latched for the tick.
            if GetForegroundWindow() == hwnd {
                for p in polls.iter_mut() {
                    let down = p.vks.iter().any(|&vk| key_down(vk));
                    if down && !p.held {
                        game.event(Ev::Key(match p.key {
                            Key::Num(n) if key_down(VK_SHIFT) => Key::ShiftNum(n),
                            k => k,
                        }));
                    }
                    p.held = down;
                }
                let typed: Vec<char> = TYPED.lock().map(|mut t| t.drain(..).collect()).unwrap_or_default();
                for c in typed.into_iter().filter(|c| !c.is_control()) {
                    game.event(Ev::Char(c));
                }
                let any = |vks: &[i32]| vks.iter().any(|&vk| key_down(vk));
                game.hold([any(&['W' as i32, VK_UP]), any(&['S' as i32, VK_DOWN]), any(&['A' as i32, VK_LEFT]), any(&['D' as i32, VK_RIGHT])]);
                let f2 = key_down(VK_F2);
                if f2 && !e_f2 {
                    game.toggle_crt();
                }
                e_f2 = f2;

                let mut pt = POINT { x: 0, y: 0 };
                GetCursorPos(&mut pt);
                ScreenToClient(hwnd, &mut pt);
                let (ox, oy, s) = view;
                let s = s.max(1);
                let (lx, ly) = ((pt.x - ox).div_euclid(s), (pt.y - oy).div_euclid(s));
                let in_view = (0..400).contains(&lx) && (0..300).contains(&ly);
                if in_view && (lx, ly) != last_mouse {
                    last_mouse = (lx, ly);
                    game.event(Ev::Move(lx, ly));
                }
                for (vk, held, right) in [(VK_LBUTTON, &mut lmb, false), (VK_RBUTTON, &mut rmb, true)] {
                    let down = key_down(vk);
                    if down != *held {
                        *held = down;
                        if down && in_view {
                            game.event(Ev::Press { x: lx, y: ly, right });
                        } else if !down {
                            game.event(Ev::Release { x: lx, y: ly, right });
                        }
                    }
                }
            } else {
                game.hold([false; 4]);
                if let Ok(mut t) = TYPED.lock() {
                    t.clear();
                }
            }
            if game.quit() {
                break 'running;
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

            // Whole wheel notches go to the game on the tick; any part-notch waits for the next.
            let notches = WHEEL.load(Ordering::Relaxed) / 120;
            if notches != 0 {
                WHEEL.fetch_sub(notches * 120, Ordering::Relaxed);
                game.event(Ev::Wheel(notches));
            }
            game.tick();
            game.draw(&mut fb);
            let pal = *game.palette();
            let mut rc: RECT = zeroed();
            GetClientRect(hwnd, &mut rc);
            let (cw, ch) = ((rc.right - rc.left).max(1), (rc.bottom - rc.top).max(1));
            let scale = Framebuffer::integer_scale_for_viewport(cw, ch);
            let (dw, dh) = fb.present(&mut scaled, scale, &pal, game.crt());
            let (dw, dh) = (dw as i32, dh as i32);
            let (dx, dy) = ((cw - dw) / 2, (ch - dh) / 2);
            let want = game.cursor_kind();
            if scale as i32 != view.2 || CURSOR.load(Ordering::Relaxed) == 0 || want != shown_pointer {
                shown_pointer = want;
                let (spr, hot) = &pointers[want as usize];
                let new = make_cursor(spr, &pal, scale as i32, *hot);
                let old = CURSOR.swap(new as usize, Ordering::Relaxed);
                if !new.is_null() && GetForegroundWindow() == hwnd {
                    SetCursor(new);
                }
                if old != 0 {
                    DestroyIcon(old as _);
                }
            }
            view = (dx, dy, scale as i32);
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
        WM_CHAR => {
            if let Some(c) = char::from_u32(wparam as u32) {
                if let Ok(mut t) = TYPED.lock() {
                    t.push(c);
                }
            }
            0
        }
        WM_MOUSEWHEEL => {
            WHEEL.fetch_add(((wparam >> 16) & 0xFFFF) as u16 as i16 as i32, Ordering::Relaxed);
            0
        }
        WM_SETCURSOR if (lparam & 0xFFFF) as isize == HTCLIENT => {
            SetCursor(CURSOR.load(Ordering::Relaxed) as HCURSOR);
            1
        }
        _ => DefWindowProcW(hwnd, msg, wparam, lparam),
    }
}
