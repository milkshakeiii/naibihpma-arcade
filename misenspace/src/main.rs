mod audio;
mod framebuffer;
mod game;
mod geom;
mod model;
mod palette;
mod render;
mod sim;
mod sprites;

use audio::Audio;
use framebuffer::Framebuffer;
use game::{Game, Key, Phase};
use model::{idx, KCell};
use sprites::Sprites;
use std::ffi::OsStr;
use std::mem::{size_of, zeroed};
use std::os::windows::ffi::OsStrExt;
use std::path::{Path, PathBuf};
use std::ptr::null_mut;
use std::sync::atomic::{AtomicI32, AtomicUsize, Ordering};
use std::time::{Duration, Instant};

use winapi::shared::minwindef::{LPARAM, LRESULT, UINT, WPARAM};
use winapi::shared::windef::{HCURSOR, HDC, HWND, POINT, RECT};
use winapi::um::libloaderapi::GetModuleHandleW;
use winapi::um::wingdi::{
    CreateBitmap, CreateDIBSection, DeleteObject, GetStockObject, PatBlt, SetStretchBltMode, StretchDIBits, BITMAPINFO,
    BITMAPINFOHEADER, BI_RGB, BLACKNESS, BLACK_BRUSH, COLORONCOLOR, DIB_RGB_COLORS, SRCCOPY,
};
use winapi::um::winuser::*;

const TRACK: &str = "1-Star_2_Object_25-Mastered-v2-LRsweepfix.wav";
const DESKTOP_MASTERS: &str = r"C:\Users\User\Desktop\albums\s4_stellarcartography\Stellar_Cartography-Mastered-v2";

static WHEEL: AtomicI32 = AtomicI32::new(0);
/// The pixel-art pointer as an OS cursor, rebuilt when the integer scale changes. Like a DOS
/// mouse driver's pointer, it moves independently of the 15 FPS frames.
static CURSOR: AtomicUsize = AtomicUsize::new(0);

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if let Some(i) = args.iter().position(|a| a == "--export-screenshots") {
        let dir = args.get(i + 1).map(PathBuf::from).unwrap_or_else(|| PathBuf::from("screenshots"));
        export_screenshots(&dir);
        return;
    }
    let music = if args.iter().any(|a| a == "--no-music") { None } else { find_music(&args) };
    let mut audio = Audio::new(music);
    let mut game = Game::new(Some(PathBuf::from("misenspace_save.txt")));
    if audio.has_music() {
        audio.play_from(0);
    }
    run_window(&mut game, &mut audio, args.iter().any(|a| a == "--windowed"));
    audio.stop();
}

/// `--music <file>` wins; otherwise the track is looked for next to the exe, in the working
/// directory, and among the Desktop masters.
fn find_music(args: &[String]) -> Option<PathBuf> {
    if let Some(i) = args.iter().position(|a| a == "--music") {
        return args.get(i + 1).map(PathBuf::from).filter(|p| p.is_file());
    }
    let exe_dir = std::env::current_exe().ok().and_then(|p| p.parent().map(Path::to_path_buf));
    let mut dirs = vec![PathBuf::from("assets/music"), PathBuf::from(DESKTOP_MASTERS)];
    if let Some(d) = exe_dir {
        dirs.insert(0, d.join("assets/music"));
    }
    dirs.into_iter().map(|d| d.join(TRACK)).find(|p| p.is_file())
}

fn key_down(vk: i32) -> bool {
    unsafe { (GetAsyncKeyState(vk) as u16 & 0x8000) != 0 }
}

struct Poll {
    vks: &'static [i32],
    key: Key,
    repeat: bool,
    held_since: Option<Instant>,
    next_repeat: Option<Instant>,
}

impl Poll {
    fn new(vks: &'static [i32], key: Key, repeat: bool) -> Self {
        Self { vks, key, repeat, held_since: None, next_repeat: None }
    }
}

fn polls() -> Vec<Poll> {
    const DIGITS: [i32; 9] = ['1' as i32, '2' as i32, '3' as i32, '4' as i32, '5' as i32, '6' as i32, '7' as i32, '8' as i32, '9' as i32];
    let mut p = vec![
        Poll::new(&[VK_UP, 'W' as i32], Key::Up, true),
        Poll::new(&[VK_DOWN, 'S' as i32], Key::Down, true),
        Poll::new(&[VK_LEFT, 'A' as i32], Key::Left, true),
        Poll::new(&[VK_RIGHT, 'D' as i32], Key::Right, true),
        Poll::new(&[VK_SPACE], Key::Act, false),
        Poll::new(&['R' as i32], Key::Rotate, false),
        Poll::new(&['X' as i32, VK_DELETE], Key::Remove, false),
        Poll::new(&[VK_RETURN], Key::Run, false),
        Poll::new(&[VK_F5], Key::Restart, false),
        Poll::new(&[VK_BACK, 'U' as i32], Key::Undo, false),
        Poll::new(&['F' as i32], Key::Fast, false),
        Poll::new(&['C' as i32], Key::Clear, false),
        Poll::new(&[VK_F1], Key::Help, false),
        Poll::new(&[VK_ESCAPE], Key::Back, false),
        Poll::new(&[VK_TAB], Key::Tab, false),
        Poll::new(&['Q' as i32], Key::PrevTool, false),
        Poll::new(&['E' as i32], Key::NextTool, false),
    ];
    for i in 0..9 {
        p.push(Poll::new(std::slice::from_ref(&DIGITS[i]), Key::Tool(i), false));
    }
    p
}

/// Builds a hardware cursor from the pointer sprite with every logical pixel an exact NxN block.
unsafe fn make_cursor(spr: &sprites::Sprite, pal: &[u32; 64], n: i32) -> HCURSOR {
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

fn run_window(game: &mut Game, audio: &mut Audio, windowed: bool) {
    unsafe {
        SetProcessDPIAware();
        let hinstance = GetModuleHandleW(null_mut());
        let wide = |s: &str| OsStr::new(s).encode_wide().chain(Some(0)).collect::<Vec<u16>>();
        let class_name = wide("MiseEnSpaceWindowClass");
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
        let title = wide("MISE EN SPACE - OBJECT 25 CANTEEN [400x300 64-COLOR VGA | 15 FPS]");
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
        let mut polls = polls();
        let mut e_f2 = false;
        let (mut lmb, mut rmb) = (false, false);
        let mut last_mouse = (-1, -1);
        let mut view = (0i32, 0i32, 0i32);
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

            // 250 Hz input: edges apply immediately and their clicks play right away.
            if GetForegroundWindow() == hwnd {
                let now = Instant::now();
                for p in polls.iter_mut() {
                    let down = p.vks.iter().any(|&vk| key_down(vk));
                    match (down, p.held_since) {
                        (true, None) => {
                            p.held_since = Some(now);
                            p.next_repeat = Some(now + Duration::from_millis(230));
                            game.key(p.key);
                        }
                        (true, Some(_)) if p.repeat => {
                            if p.next_repeat.is_some_and(|t| now >= t) {
                                p.next_repeat = Some(now + Duration::from_millis(80));
                                game.key(p.key);
                            }
                        }
                        (false, Some(_)) => {
                            p.held_since = None;
                            if p.key == Key::Act {
                                game.key(Key::ActUp);
                            }
                        }
                        _ => {}
                    }
                }
                let f2 = key_down(VK_F2);
                if f2 && !e_f2 {
                    game.crt = !game.crt;
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
                    game.mouse_move(lx, ly);
                }
                for (vk, held, left) in [(VK_LBUTTON, &mut lmb, true), (VK_RBUTTON, &mut rmb, false)] {
                    let down = key_down(vk);
                    if down != *held {
                        *held = down;
                        if in_view || !down {
                            game.mouse_button(left, down, lx, ly);
                        }
                    }
                }
                let wh = WHEEL.swap(0, Ordering::Relaxed);
                if wh != 0 {
                    game.wheel((wh / 120).max(1));
                }
            } else {
                WHEEL.store(0, Ordering::Relaxed);
            }
            for s in game.sfx.drain(..) {
                audio.play_sfx(s);
            }
            if game.quit {
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

            game.update();
            for s in game.sfx.drain(..) {
                audio.play_sfx(s);
            }
            // Background music loops; nothing in the game listens to it.
            if let Some(len) = audio.track_len_ms() {
                if audio.has_music() && audio.song_ms() + 120 >= len {
                    audio.play_from(0);
                }
            }

            render::draw(game, &sprites, &mut fb);
            let mut rc: RECT = zeroed();
            GetClientRect(hwnd, &mut rc);
            let (cw, ch) = ((rc.right - rc.left).max(1), (rc.bottom - rc.top).max(1));
            let scale = Framebuffer::integer_scale_for_viewport(cw, ch);
            let (dw, dh) = fb.present(&mut scaled, scale, &pal, &pal, 0, game.crt);
            let (dw, dh) = (dw as i32, dh as i32);
            let (dx, dy) = ((cw - dw) / 2, (ch - dh) / 2);
            if scale as i32 != view.2 || CURSOR.load(Ordering::Relaxed) == 0 {
                let new = make_cursor(&sprites.pointer, &pal, scale as i32);
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
        WM_MOUSEWHEEL => {
            WHEEL.fetch_add(GET_WHEEL_DELTA_WPARAM(wparam) as i32, Ordering::Relaxed);
            0
        }
        WM_SETCURSOR if (lparam & 0xFFFF) as isize == HTCLIENT => {
            SetCursor(CURSOR.load(Ordering::Relaxed) as HCURSOR);
            1
        }
        _ => DefWindowProcW(hwnd, msg, wparam, lparam),
    }
}

/// Headless frames at 2x for checking the look.
fn export_screenshots(dir: &Path) {
    let _ = std::fs::create_dir_all(dir);
    let sprites = Sprites::new();
    let pal = palette::build();
    let mut fb = Framebuffer::new();
    let save = |g: &Game, fb: &mut Framebuffer, name: &str| {
        render::draw(g, &sprites, fb);
        fb.save_png(&dir.join(name), &pal, &pal, 0);
    };

    let mut g = Game::new(None);
    g.cursor = (1, 0);
    save(&g, &mut fb, "01_prep_empty.png");

    // A kitchen part-built, with the chute tool picked over a table square.
    let k = sim::lab::reference_kitchen(&g.r);
    for i in 0..24 {
        if k[i] != g.r.kitchen[i] {
            g.layout[i] = Some(k[i]);
        }
    }
    g.restart();
    g.message = None;
    g.tool = g.r.shelf.iter().position(|s| s.0 == model::Tool::Chute).unwrap();
    g.cursor = (6, 1);
    save(&g, &mut fb, "02_building.png");

    for i in 0..64 {
        if k[i] != g.r.kitchen[i] {
            g.layout[i] = Some(k[i]);
        }
    }
    g.restart();
    g.tool = 0;
    g.cursor = (12, 4);
    g.key(Key::Run);
    let run_to = |g: &mut Game, beat: u32| {
        while g.sim.beat < beat && g.phase != Phase::Closed {
            g.update();
        }
        g.update();
        g.update();
    };
    run_to(&mut g, 120);
    g.message = None;
    save(&g, &mut fb, "03_first_guests.png");
    run_to(&mut g, 690);
    g.cursor = (3, 1);
    save(&g, &mut fb, "04_rush.png");
    g.speed = 4;
    while g.phase != Phase::Closed {
        g.update();
    }
    save(&g, &mut fb, "05_closing_time.png");

    // The same evening with no kitchen at all, partway in.
    let mut g2 = Game::new(None);
    g2.layout[idx(1, 0)] = Some(KCell::Belt(model::Dir::E));
    g2.restart();
    g2.key(Key::Run);
    run_to(&mut g2, 260);
    g2.cursor = (14, 4);
    g2.message = None;
    save(&g2, &mut fb, "06_hungry_guests.png");

    g.show_help = true;
    save(&g, &mut fb, "07_help.png");
    println!("exported screenshots to {}", dir.display());
}
