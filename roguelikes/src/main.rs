mod app;
mod audio;
mod changeling;
mod form;
mod framebuffer;
mod palette;
mod rng;
mod shots;
mod turncoat;
mod ui;

use app::{App, Ev, Key};
use audio::Audio;
use framebuffer::{Framebuffer, Sprite};
use std::ffi::OsStr;
use std::mem::{size_of, zeroed};
use std::os::windows::ffi::OsStrExt;
use std::path::PathBuf;
use std::ptr::null_mut;
use std::sync::atomic::{AtomicUsize, Ordering};
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

/// The pixel-art pointer as an OS cursor, rebuilt when the integer scale changes. Like a DOS
/// mouse driver's pointer, it moves independently of the 15 FPS frames.
static CURSOR: AtomicUsize = AtomicUsize::new(0);
/// Typed characters from WM_CHAR (the OS handles key repeat and keyboard layouts).
static TYPED: Mutex<Vec<char>> = Mutex::new(Vec::new());

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if let Some(i) = args.iter().position(|a| a == "--export-screenshots") {
        let dir = args.get(i + 1).map(PathBuf::from).unwrap_or_else(|| PathBuf::from("screenshots"));
        shots::export(&dir);
        return;
    }
    let mut app = App::new(Some(PathBuf::from("roguelikes_save.txt")), Some(PathBuf::from("playtest_log.txt")));
    let mut audio = Audio::new(!args.iter().any(|a| a == "--no-music"), app.music_level);
    app.log_line("");
    app.log_line(&format!("##### session start {} #####", app::utc_stamp(app::unix_now())));
    run_window(&mut app, &mut audio, args.iter().any(|a| a == "--windowed"));
    app.shutdown();
}

fn key_down(vk: i32) -> bool {
    unsafe { (GetAsyncKeyState(vk) as u16 & 0x8000) != 0 }
}

struct Poll {
    vks: &'static [i32],
    key: Key,
    repeat: bool,
    held: bool,
    next_repeat: Option<Instant>,
}

fn polls() -> Vec<Poll> {
    let p = |vks: &'static [i32], key: Key, repeat: bool| Poll { vks, key, repeat, held: false, next_repeat: None };
    vec![
        p(&[VK_UP], Key::Up, true),
        p(&[VK_DOWN], Key::Down, true),
        p(&[VK_LEFT], Key::Left, true),
        p(&[VK_RIGHT], Key::Right, true),
        p(&[VK_RETURN], Key::Enter, false),
        p(&[VK_SPACE], Key::Space, false),
        p(&[VK_ESCAPE], Key::Esc, false),
        p(&[VK_BACK], Key::Back, false),
        p(&[VK_TAB], Key::Tab, false),
        p(&[VK_F1], Key::Help, false),
        p(&['R' as i32], Key::Rest, false),
        p(&['F' as i32], Key::Fast, false),
        p(&['U' as i32], Key::Undo, false),
    ]
}

/// Builds a hardware cursor from the pointer sprite with every logical pixel an exact NxN block.
unsafe fn make_cursor(spr: &Sprite, pal: &[u32; 64], n: i32) -> HCURSOR {
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
    let mut info = ICONINFO { fIcon: 0, xHotspot: 0, yHotspot: 0, hbmMask: mask, hbmColor: color };
    let cursor = CreateIconIndirect(&mut info);
    DeleteObject(color as _);
    DeleteObject(mask as _);
    cursor
}

fn run_window(app: &mut App, audio: &mut Audio, windowed: bool) {
    unsafe {
        SetProcessDPIAware();
        let hinstance = GetModuleHandleW(null_mut());
        let wide = |s: &str| OsStr::new(s).encode_wide().chain(Some(0)).collect::<Vec<u16>>();
        let class_name = wide("TwoRoguelikesWindowClass");
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
        let title = wide("TWO ROGUELIKES: CHANGELING / TURNCOAT [400x300 64-COLOR VGA | 15 FPS]");
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

        let pointer = ui::pointer();
        let mut fb = Framebuffer::new();
        let mut scaled: Vec<u32> = Vec::new();
        let mut polls = polls();
        let (mut e_f2, mut e_f3) = (false, false);
        let (mut lmb, mut rmb) = (false, false);
        let mut last_mouse = (-1, -1);
        let mut view = (0i32, 0i32, 0i32);
        let tick = Duration::from_micros(66_667);
        let mut next_tick = Instant::now();
        let mut msg: MSG = zeroed();
        let mut cursor_pal = [0u32; 64];

        'running: loop {
            while PeekMessageW(&mut msg, null_mut(), 0, 0, PM_REMOVE) != 0 {
                if msg.message == WM_QUIT {
                    break 'running;
                }
                TranslateMessage(&msg);
                DispatchMessageW(&msg);
            }

            // 250 Hz input: edges apply immediately.
            if GetForegroundWindow() == hwnd {
                let now = Instant::now();
                for p in polls.iter_mut() {
                    let down = p.vks.iter().any(|&vk| key_down(vk));
                    if down && !p.held {
                        p.held = true;
                        p.next_repeat = Some(now + Duration::from_millis(260));
                        app.event(Ev::Key(p.key));
                    } else if down && p.repeat && p.next_repeat.is_some_and(|t| now >= t) {
                        p.next_repeat = Some(now + Duration::from_millis(90));
                        app.event(Ev::Key(p.key));
                    } else if !down {
                        p.held = false;
                    }
                }
                let typed: Vec<char> = TYPED.lock().map(|mut t| t.drain(..).collect()).unwrap_or_default();
                for c in typed {
                    app.event(Ev::Char(c));
                }
                let f2 = key_down(VK_F2);
                if f2 && !e_f2 {
                    app.toggle_crt();
                }
                e_f2 = f2;
                let f3 = key_down(VK_F3);
                if f3 && !e_f3 {
                    app.cycle_music();
                }
                e_f3 = f3;

                let mut pt = POINT { x: 0, y: 0 };
                GetCursorPos(&mut pt);
                ScreenToClient(hwnd, &mut pt);
                let (ox, oy, s) = view;
                let s = s.max(1);
                let (lx, ly) = ((pt.x - ox).div_euclid(s), (pt.y - oy).div_euclid(s));
                let in_view = (0..400).contains(&lx) && (0..300).contains(&ly);
                if in_view && (lx, ly) != last_mouse {
                    last_mouse = (lx, ly);
                    app.event(Ev::Move(lx, ly));
                }
                for (vk, held, right) in [(VK_LBUTTON, &mut lmb, false), (VK_RBUTTON, &mut rmb, true)] {
                    let down = key_down(vk);
                    if down != *held {
                        *held = down;
                        if down && in_view {
                            app.event(Ev::Press { x: lx, y: ly, right });
                        } else if !down {
                            app.event(Ev::Release { x: lx, y: ly, right });
                        }
                    }
                }
            } else if let Ok(mut t) = TYPED.lock() {
                t.clear();
            }
            if app.music_changed {
                app.music_changed = false;
                audio.set_level(app.music_level);
            }
            for s in app.out.sfx.drain(..) {
                audio.play_sfx(s);
            }
            if let Some(t) = app.out.music.take() {
                audio.music(t);
            }
            if app.quit {
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

            app.tick();
            for s in app.out.sfx.drain(..) {
                audio.play_sfx(s);
            }
            if let Some(t) = app.out.music.take() {
                audio.music(t);
            }

            app.draw(&mut fb);
            let pal = *app.palette();
            let mut rc: RECT = zeroed();
            GetClientRect(hwnd, &mut rc);
            let (cw, ch) = ((rc.right - rc.left).max(1), (rc.bottom - rc.top).max(1));
            let scale = Framebuffer::integer_scale_for_viewport(cw, ch);
            let (dw, dh) = fb.present(&mut scaled, scale, &pal, app.crt);
            let (dw, dh) = (dw as i32, dh as i32);
            let (dx, dy) = ((cw - dw) / 2, (ch - dh) / 2);
            // The pointer uses only the outline and white entries, which every palette shares.
            if scale as i32 != view.2 || CURSOR.load(Ordering::Relaxed) == 0 || cursor_pal[1] != pal[1] {
                cursor_pal = pal;
                let new = make_cursor(&pointer, &pal, scale as i32);
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
        WM_SETCURSOR if (lparam & 0xFFFF) as isize == HTCLIENT => {
            SetCursor(CURSOR.load(Ordering::Relaxed) as HCURSOR);
            1
        }
        _ => DefWindowProcW(hwnd, msg, wparam, lparam),
    }
}
