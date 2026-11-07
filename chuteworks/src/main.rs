mod audio;
mod framebuffer;
mod game;
mod geom;
mod levels;
mod model;
mod palette;
mod render;
mod sim;
mod sprites;

use audio::Audio;
use framebuffer::Framebuffer;
use game::{Game, Key, Phase};
use model::{Board, Cell, Dir, Layout};
use sprites::Sprites;
use std::ffi::OsStr;
use std::mem::{size_of, zeroed};
use std::os::windows::ffi::OsStrExt;
use std::path::{Path, PathBuf};
use std::ptr::null_mut;
use std::sync::atomic::{AtomicI32, Ordering};
use std::time::{Duration, Instant};

use winapi::shared::minwindef::{LPARAM, LRESULT, UINT, WPARAM};
use winapi::shared::windef::{HDC, HWND, POINT, RECT};
use winapi::um::libloaderapi::GetModuleHandleW;
use winapi::um::wingdi::{
    GetStockObject, PatBlt, SetStretchBltMode, StretchDIBits, BITMAPINFO, BITMAPINFOHEADER, BI_RGB, BLACKNESS, BLACK_BRUSH,
    COLORONCOLOR, DIB_RGB_COLORS, SRCCOPY,
};
use winapi::um::winuser::*;

static WHEEL: AtomicI32 = AtomicI32::new(0);

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if let Some(i) = args.iter().position(|a| a == "--export-screenshots") {
        let dir = args.get(i + 1).map(PathBuf::from).unwrap_or_else(|| PathBuf::from("screenshots"));
        export_screenshots(&dir);
        return;
    }
    let mut game = match Game::new(Some(PathBuf::from("chuteworks_save.txt")), Some(PathBuf::from("chuteworks_playtest.log"))) {
        Ok(g) => g,
        Err(e) => {
            eprintln!("level error: {e}");
            return;
        }
    };
    let unix = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map_or(0, |d| d.as_secs());
    game.log(&format!("=== session start (unix time {unix}) ==="));
    let audio = Audio::new();
    run_window(&mut game, &audio, args.iter().any(|a| a == "--windowed"));
    game.end_session();
}

fn key_down(vk: i32) -> bool {
    unsafe { (GetAsyncKeyState(vk) as u16 & 0x8000) != 0 }
}

/// One logical key polled at 250 Hz: rising edges, plus auto-repeat for cursor keys.
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

const VK_W: i32 = 'W' as i32;
const VK_A: i32 = 'A' as i32;
const VK_S: i32 = 'S' as i32;
const VK_D: i32 = 'D' as i32;

fn polls() -> Vec<Poll> {
    let mut p = vec![
        Poll::new(&[VK_UP, VK_W], Key::Up, true),
        Poll::new(&[VK_DOWN, VK_S], Key::Down, true),
        Poll::new(&[VK_LEFT, VK_A], Key::Left, true),
        Poll::new(&[VK_RIGHT, VK_D], Key::Right, true),
        Poll::new(&[VK_SPACE], Key::Act, false),
        Poll::new(&['R' as i32], Key::Rotate, false),
        Poll::new(&['T' as i32], Key::Flip, false),
        Poll::new(&['X' as i32, VK_DELETE], Key::Remove, false),
        Poll::new(&[VK_RETURN], Key::Run, false),
        Poll::new(&[VK_BACK, 'U' as i32], Key::Undo, false),
        Poll::new(&['F' as i32], Key::Fast, false),
        Poll::new(&['H' as i32], Key::Hint, false),
        Poll::new(&['C' as i32], Key::Clear, false),
        Poll::new(&[VK_F1], Key::Help, false),
        Poll::new(&[VK_ESCAPE], Key::Back, false),
        Poll::new(&[VK_TAB], Key::Tab, false),
        Poll::new(&['Q' as i32], Key::PrevTool, false),
        Poll::new(&['E' as i32], Key::NextTool, false),
    ];
    const DIGITS: [i32; 9] = ['1' as i32, '2' as i32, '3' as i32, '4' as i32, '5' as i32, '6' as i32, '7' as i32, '8' as i32, '9' as i32];
    for i in 0..9 {
        p.push(Poll::new(std::slice::from_ref(&DIGITS[i]), Key::Tool(i), false));
    }
    p
}

fn run_window(game: &mut Game, audio: &Audio, windowed: bool) {
    unsafe {
        SetProcessDPIAware();
        let hinstance = GetModuleHandleW(null_mut());
        let wide = |s: &str| OsStr::new(s).encode_wide().chain(Some(0)).collect::<Vec<u16>>();
        let class_name = wide("ChuteworksWindowClass");
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
        let title = wide("CHUTEWORKS [400x300 64-COLOR VGA | 15 FPS]");
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
        let pals = [palette::build(1), palette::build(2)];
        let mut fb = Framebuffer::new();
        let mut scaled: Vec<u32> = Vec::new();
        let mut polls = polls();
        let mut e_f2 = false;
        let (mut lmb, mut rmb) = (false, false);
        let mut last_mouse = (-1, -1);
        // Viewport mapping from the last present: (offset x, offset y, scale).
        let mut view = (0i32, 0i32, 1i32);

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

            // 250 Hz input: edges apply immediately; their clicks play right away.
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
                let (lx, ly) = ((pt.x - ox).div_euclid(s), (pt.y - oy).div_euclid(s));
                let in_view = (0..400).contains(&lx) && (0..300).contains(&ly);
                if in_view && (lx, ly) != last_mouse {
                    last_mouse = (lx, ly);
                    game.mouse_move(lx, ly);
                    game.pointer = Some((lx, ly));
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
                let w = WHEEL.swap(0, Ordering::Relaxed);
                if w != 0 {
                    game.wheel((w / 120).max(1) * w.signum().abs());
                }
            } else {
                WHEEL.store(0, Ordering::Relaxed);
            }
            for s in game.sfx.drain(..) {
                audio.play(s);
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
                audio.play(s);
            }

            render::draw(game, &sprites, &mut fb);
            let mut rc: RECT = zeroed();
            GetClientRect(hwnd, &mut rc);
            let (cw, ch) = ((rc.right - rc.left).max(1), (rc.bottom - rc.top).max(1));
            let scale = Framebuffer::integer_scale_for_viewport(cw, ch);
            let (a, b, mix) = game.hall_pal;
            let from = &pals[(a.max(1) - 1).min(1) as usize];
            let to = &pals[(b.max(1) - 1).min(1) as usize];
            let (dw, dh) = fb.present(&mut scaled, scale, from, to, mix, game.crt);
            let (dw, dh) = (dw as i32, dh as i32);
            let (dx, dy) = ((cw - dw) / 2, (ch - dh) / 2);
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
        // The game draws its own pixel pointer inside the client area.
        WM_SETCURSOR if (lparam & 0xFFFF) as isize == HTCLIENT => {
            SetCursor(null_mut());
            1
        }
        _ => DefWindowProcW(hwnd, msg, wparam, lparam),
    }
}

/// Headless frames at 2x for checking the look, saved as PNGs.
fn export_screenshots(dir: &Path) {
    let _ = std::fs::create_dir_all(dir);
    let sprites = Sprites::new();
    let pals = [palette::build(1), palette::build(2)];
    let mut fb = Framebuffer::new();
    let mut g = Game::new(None, None).expect("levels");
    let save = |g: &Game, fb: &mut Framebuffer, name: &str| {
        render::draw(g, &sprites, fb);
        let p = &pals[(g.hall().max(1) - 1).min(1) as usize];
        fb.save_png(&dir.join(name), p, p, 0);
    };
    let run_until = |g: &mut Game, stop: &dyn Fn(&Game) -> bool| {
        for _ in 0..4000 {
            if stop(g) {
                return;
            }
            g.update();
        }
    };
    let find = |g: &Game, id: &str| g.levels.iter().position(|l| l.id == id).unwrap();

    for i in 0..6 {
        g.solved[i] = true;
    }
    g.sel = 6;
    save(&g, &mut fb, "01_halls.png");

    // Drawing the first belt of First Coat, cursor on the way to the booth.
    g.open_level(find(&g, "1-1"));
    for (x, y, d) in [(1, 3, Dir::E), (2, 3, Dir::N), (2, 2, Dir::E), (3, 2, Dir::E)] {
        g.layout.set(Board::Left, x, y, Some(Cell::Belt(d)));
    }
    g.cursor = (5, 2);
    save(&g, &mut fb, "02_first_coat_building.png");

    let lv = find(&g, "1-5");
    g.open_level(lv);
    g.layout = Layout::from_solution(&g.levels[lv]);
    g.cursor = (3, 4);
    g.start_run();
    for _ in 0..34 {
        g.update();
    }
    save(&g, &mut fb, "03_through_the_booth_making.png");
    run_until(&mut g, &|g| g.phase == Phase::Deliver && g.deliver.as_ref().is_some_and(|d| d.trails.len() >= 3));
    for _ in 0..7 {
        g.update();
    }
    save(&g, &mut fb, "04_through_the_booth_delivering.png");

    // The natural first try: straight lines, so grey reaches the orange bin.
    g.open_level(lv);
    g.layout = Layout::default();
    for x in 1..8 {
        g.layout.set(Board::Left, x, 1, Some(Cell::Belt(Dir::E)));
        g.layout.set(Board::Left, x, 6, Some(Cell::Belt(Dir::E)));
    }
    g.start_run();
    run_until(&mut g, &|g| !g.phase.running());
    save(&g, &mut fb, "05_through_the_booth_first_try.png");

    let wf = find(&g, "2-6");
    g.open_level(wf);
    g.hall_pal = (2, 2, 16);
    g.layout = Layout::from_solution(&g.levels[wf]);
    g.cursor = (12, 3);
    g.start_run();
    for _ in 0..30 {
        g.update();
    }
    save(&g, &mut fb, "06_wrong_foot_zipper.png");
    run_until(&mut g, &|g| g.deliver.as_ref().is_some_and(|d| d.got.iter().map(|v| v.len()).sum::<usize>() >= 2));
    for _ in 0..3 {
        g.update();
    }
    save(&g, &mut fb, "07_wrong_foot_flip_flop.png");

    let mud = find(&g, "1-6");
    g.open_level(mud);
    g.layout = Layout::default();
    for x in 1..8 {
        g.layout.set(Board::Left, x, 1, Some(Cell::Belt(Dir::E)));
    }
    for (x, y, d) in [(1, 6, Dir::E), (2, 6, Dir::E), (3, 6, Dir::E), (4, 6, Dir::E), (6, 6, Dir::N), (6, 5, Dir::N), (6, 4, Dir::W), (5, 4, Dir::W), (4, 4, Dir::W)] {
        g.layout.set(Board::Left, x, y, Some(Cell::Belt(d)));
    }
    g.cursor = (3, 4);
    g.start_run();
    run_until(&mut g, &|g| !g.phase.running());
    save(&g, &mut fb, "08_mud_jam.png");

    let bf = find(&g, "2-9");
    g.open_level(bf);
    g.layout = Layout::from_solution(&g.levels[bf]);
    g.cursor = (13, 3);
    g.start_run();
    run_until(&mut g, &|g| !g.phase.running());
    save(&g, &mut fb, "09_blue_goes_first_solved.png");

    // Stuck in Through the Booth: the first hint ghosts the tiles around the booth.
    g.open_level(lv);
    g.layout = Layout::default();
    g.hall_pal = (1, 1, 16);
    g.key(game::Key::Hint);
    g.cursor = (1, 1);
    save(&g, &mut fb, "10_through_the_booth_hint.png");

    g.show_help = true;
    save(&g, &mut fb, "11_keys.png");
    println!("exported screenshots to {}", dir.display());
}
