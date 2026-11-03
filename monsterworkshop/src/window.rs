//! The player's game on Windows: a window showing the title screen's worlds, then a server's
//! pictures. Playing a world alone runs its server inside the game; `--connect` plays on
//! someone else's. Keys are polled at 250 Hz and sent the moment one goes down or what's held
//! changes, and each picture is shown the moment it arrives. `--lag MS` slows every connection
//! the window makes by that much each round trip, to feel a slow one.

use crate::audio::Audio;
use crate::client::{self, Connection};
use crate::framebuffer::Framebuffer;
use crate::game::{Game, Input};
use crate::net::{self, ToPlayer};
use crate::server::Server;
use crate::{palette, render, save};
use std::ffi::OsStr;
use std::mem::{size_of, zeroed};
use std::os::windows::ffi::OsStrExt;
use std::path::PathBuf;
use std::ptr::null_mut;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

use winapi::shared::minwindef::{LPARAM, LRESULT, UINT, WPARAM};
use winapi::shared::windef::{HDC, HWND, RECT};
use winapi::um::libloaderapi::GetModuleHandleW;
use winapi::um::wingdi::{
    GetStockObject, PatBlt, SetStretchBltMode, StretchDIBits, BITMAPINFO, BITMAPINFOHEADER, BI_RGB, BLACKNESS,
    BLACK_BRUSH, COLORONCOLOR, DIB_RGB_COLORS, SRCCOPY,
};
use winapi::um::winuser::*;

/// The title screen's list of worlds, or a world being played.
enum Screen {
    /// `deleting`: Del was pressed once on the picked world; Del again deletes it.
    Title { worlds: Vec<PathBuf>, sel: usize, error: Option<String>, deleting: bool },
    /// `local`: the world's own server, when playing alone.
    Play { conn: Connection, local: Option<Local> },
}

impl Screen {
    fn title(error: Option<String>) -> Self {
        Screen::Title { worlds: save::list_worlds(), sel: 0, error, deleting: false }
    }

    /// Leaves a world: its server, if it runs here, saves and stops.
    fn leave(&mut self, to: Screen) {
        if let Screen::Play { local: Some(local), .. } = std::mem::replace(self, to) {
            local.finish();
        }
    }
}

/// A world played alone: its server, running inside the game.
struct Local {
    stop: Arc<AtomicBool>,
    thread: std::thread::JoinHandle<()>,
}

impl Local {
    fn finish(self) {
        self.stop.store(true, Ordering::Relaxed);
        let _ = self.thread.join();
    }
}

/// Starts a world's server on a port only this computer can reach, and connects to it.
fn play_alone(game: Game, path: PathBuf, key: &str, lag: Duration) -> std::io::Result<Screen> {
    let listener = std::net::TcpListener::bind("127.0.0.1:0")?;
    let addr = listener.local_addr()?;
    let server = Server::new(game, path, listener)?;
    let stop = Arc::new(AtomicBool::new(false));
    let running = stop.clone();
    let thread = std::thread::spawn(move || server.run(&running));
    let conn = Connection::connect(addr, key, lag)?;
    Ok(Screen::Play { conn, local: Some(Local { stop, thread }) })
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

/// Every key the game reads, polled at 250 Hz.
#[derive(Default)]
struct Keys {
    dirs: [Edge; 4],
    space: Edge,
    rec: Edge,
    play: Edge,
    tab: Edge,
    cancel: Edge,
    f1: Edge,
    f2: Edge,
    esc: Edge,
    sheet: Edge,
    del: Edge,
}

impl Keys {
    /// What's held right now and what went down since the last poll, and whether F2 (the
    /// screen's scanlines, never sent anywhere) went down.
    fn poll(&mut self) -> (Input, bool) {
        let held = [
            key_down(VK_UP) || key_down('W' as i32),
            key_down(VK_DOWN) || key_down('S' as i32),
            key_down(VK_LEFT) || key_down('A' as i32),
            key_down(VK_RIGHT) || key_down('D' as i32),
        ];
        let mut i = Input { held, inspect: key_down('E' as i32), pull: key_down(VK_SHIFT), tab_held: key_down(VK_TAB), ..Input::default() };
        for k in 0..4 {
            i.pressed[k] = self.dirs[k].rose(held[k]);
        }
        i.use_pressed = self.space.rose(key_down(VK_SPACE) || key_down(VK_RETURN));
        i.rec_pressed = self.rec.rose(key_down('R' as i32));
        i.play_pressed = self.play.rose(key_down('P' as i32));
        i.tab_pressed = self.tab.rose(key_down(VK_TAB));
        i.cancel_pressed = self.cancel.rose(key_down(VK_BACK));
        i.help_pressed = self.f1.rose(key_down(VK_F1));
        i.sheet_pressed = self.sheet.rose(key_down('C' as i32));
        i.quit_pressed = self.esc.rose(key_down(VK_ESCAPE));
        i.delete_pressed = self.del.rose(key_down(VK_DELETE));
        (i, self.f2.rose(key_down(VK_F2)))
    }
}

/// Opens the window: the title screen, or straight onto a server at `connect`, as `player`
/// (see `client::player_key`). Every connection gets `lag` added to its round trips.
pub fn run(audio: &mut Audio, windowed: bool, connect: Option<String>, player: Option<String>, lag: Duration) {
    let key = client::player_key(player.as_deref());
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
        let lagged = if lag.is_zero() { String::new() } else { format!(" | {} MS LAG", lag.as_millis()) };
        let title = wide(&format!("MONSTER WORKSHOP [400x300 64-COLOR VGA | 15 FPS{lagged}]"));
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

        let pal = palette::build();
        let mut fb = Framebuffer::new();
        let mut scaled: Vec<u32> = Vec::new();
        let mut present = |fb: &Framebuffer, crt: bool| {
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
        };

        let mut keys = Keys::default();
        let mut latched = Input::default();
        let mut crt = true;
        // Whether the character sheet is open, as far as this game knows: then Esc closes it
        // rather than leaving the world.
        let mut sheet_open = false;
        let mut screen = match connect {
            Some(addr) => {
                let addr = if addr.contains(':') { addr } else { format!("{addr}:{}", net::DEFAULT_PORT) };
                match Connection::connect(addr.as_str(), &key, lag) {
                    Ok(conn) => Screen::Play { conn, local: None },
                    Err(e) => Screen::title(Some(format!("COULDN'T REACH {addr}: {e}").to_uppercase())),
                }
            }
            None => Screen::title(None),
        };

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

            // Keys only count while the window has focus.
            let (polled, f2) = if GetForegroundWindow() == hwnd { keys.poll() } else { (Input::default(), false) };
            let mut redraw = f2;
            crt ^= f2;

            // Playing: keys go out as they change, and pictures are shown as they arrive.
            if let Screen::Play { conn, .. } = &mut screen {
                let mut polled = polled;
                sheet_open ^= polled.sheet_pressed;
                if polled.quit_pressed && sheet_open {
                    (polled.quit_pressed, polled.close_pressed, sheet_open) = (false, true, false);
                }
                if polled.quit_pressed {
                    break 'running;
                }
                let mut lost = conn.keys(&polled).err().map(|e| format!("LOST THE CONNECTION: {e}"));
                for m in conn.messages() {
                    match m {
                        Ok(ToPlayer::Frame { sounds, pixels }) => {
                            if let Err(e) = net::apply_delta(&mut fb.px, &pixels) {
                                lost = Some(format!("THE SERVER SENT A BAD PICTURE: {e}"));
                            }
                            for s in sounds {
                                audio.play_sfx(s);
                            }
                            redraw = true;
                        }
                        Ok(ToPlayer::Welcome { .. }) => {}
                        Ok(ToPlayer::Bye(why)) => lost = Some(why),
                        Err(e) => lost = lost.or(Some(format!("LOST THE CONNECTION: {e}"))),
                    }
                }
                if let Some(why) = lost {
                    screen.leave(Screen::title(Some(why.to_uppercase())));
                }
            }
            latched = latched.or(&polled);

            let now = Instant::now();
            if now >= next_tick {
                next_tick += tick;
                if now > next_tick + tick * 4 {
                    next_tick = now + tick;
                }
                let inp = std::mem::take(&mut latched);
                if let Screen::Title { worlds, sel, error, deleting } = &mut screen {
                    let count = worlds.len() + 1; // "NEW WORLD" first, then saved worlds
                    // Anything but a second Del keeps the world.
                    let kept = *deleting && (inp.pressed.iter().any(|&p| p) || inp.use_pressed || inp.quit_pressed);
                    if kept {
                        (*deleting, *error) = (false, None);
                    } else if inp.quit_pressed {
                        break 'running;
                    }
                    if inp.pressed[0] {
                        *sel = (*sel + count - 1) % count;
                    }
                    if inp.pressed[1] {
                        *sel = (*sel + 1) % count;
                    }
                    if inp.delete_pressed && *sel > 0 {
                        let path = worlds[*sel - 1].clone();
                        if *deleting {
                            let failed = save::delete(&path).err().map(|e| format!("COULDN'T DELETE IT: {e}"));
                            let left = save::list_worlds();
                            *sel = (*sel).min(left.len());
                            (*worlds, *error, *deleting) = (left, failed, false);
                        } else {
                            *deleting = true;
                            *error = Some(format!("DEL AGAIN DELETES {} FOR GOOD. ESC KEEPS IT.", save::display_name(&path)));
                        }
                    }
                    let names: Vec<String> = worlds.iter().map(|p| save::display_name(p)).collect();
                    render::draw_title(&mut fb, &names, *sel, error.as_deref(), *deleting);
                    if inp.use_pressed && !kept {
                        let started = if *sel == 0 {
                            Ok((Game::new(), save::new_world_path()))
                        } else {
                            let path = worlds[*sel - 1].clone();
                            save::load(&path).map(|g| (g, path))
                        };
                        screen = match started.and_then(|(g, path)| play_alone(g, path, &key, lag).map_err(|e| format!("COULDN'T START THE WORLD: {e}").to_uppercase())) {
                            // The server's pictures start from a blank screen.
                            Ok(play) => {
                                fb.px.fill(0);
                                play
                            }
                            Err(e) => Screen::title(Some(e)),
                        };
                    }
                }
                // Every tick, so a resized window is redrawn.
                redraw = true;
            }
            if redraw {
                present(&fb, crt);
            } else {
                std::thread::sleep(Duration::from_millis(4));
            }
        }
        // Closing the window saves and stops a world played alone.
        screen.leave(Screen::title(None));
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
