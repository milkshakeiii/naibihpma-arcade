#![allow(non_snake_case, clippy::too_many_arguments)]

use std::ffi::c_void;
use std::mem::{size_of, zeroed};
use std::ptr;
use std::time::{Duration, Instant};

use crate::app::input::{InputState, Key};
use crate::app::App;
use crate::gfx::{Framebuffer, FB_H, FB_W};
use crate::platform::audio::AudioEngine;

type HWND = isize;
type HINSTANCE = isize;
type HDC = isize;
type HBRUSH = isize;
type HICON = isize;
type HCURSOR = isize;
type HMENU = isize;
type LRESULT = isize;
type WPARAM = usize;
type LPARAM = isize;
type ATOM = u16;

#[repr(C)]
struct WndClassExW {
    cb_size: u32,
    style: u32,
    lpfn_wnd_proc: extern "system" fn(HWND, u32, WPARAM, LPARAM) -> LRESULT,
    cb_cls_extra: i32,
    cb_wnd_extra: i32,
    h_instance: HINSTANCE,
    h_icon: HICON,
    h_cursor: HCURSOR,
    hbr_background: HBRUSH,
    lpsz_menu_name: *const u16,
    lpsz_class_name: *const u16,
    h_icon_sm: HICON,
}

#[repr(C)]
#[derive(Default)]
struct Point {
    x: i32,
    y: i32,
}

#[repr(C)]
#[derive(Default)]
struct Msg {
    hwnd: HWND,
    message: u32,
    w_param: WPARAM,
    l_param: LPARAM,
    time: u32,
    pt: Point,
}

#[repr(C)]
#[derive(Default, Clone, Copy)]
struct Rect {
    left: i32,
    top: i32,
    right: i32,
    bottom: i32,
}

#[repr(C)]
struct PaintStruct {
    hdc: HDC,
    f_erase: i32,
    rc_paint: Rect,
    f_restore: i32,
    f_inc_update: i32,
    rgb_reserved: [u8; 32],
}

#[repr(C)]
struct BitmapInfoHeader {
    bi_size: u32,
    bi_width: i32,
    bi_height: i32,
    bi_planes: u16,
    bi_bit_count: u16,
    bi_compression: u32,
    bi_size_image: u32,
    bi_x_pels_per_meter: i32,
    bi_y_pels_per_meter: i32,
    bi_clr_used: u32,
    bi_clr_important: u32,
}

#[repr(C)]
struct BitmapInfo {
    bmi_header: BitmapInfoHeader,
    bmi_colors: [u32; 1],
}

#[repr(C)]
struct WindowPlacement {
    length: u32,
    flags: u32,
    show_cmd: u32,
    pt_min_position: Point,
    pt_max_position: Point,
    rc_normal_position: Rect,
}

const WS_OVERLAPPEDWINDOW: u32 = 0x00CF_0000;
const WS_POPUP: u32 = 0x8000_0000;
const WS_VISIBLE: u32 = 0x1000_0000;
const CS_OWNDC: u32 = 0x0020;
const CW_USEDEFAULT: i32 = 0x8000_0000u32 as i32;

const WM_DESTROY: u32 = 0x0002;
const WM_ERASEBKGND: u32 = 0x0014;
const WM_PAINT: u32 = 0x000F;
const WM_KEYDOWN: u32 = 0x0100;
const WM_KEYUP: u32 = 0x0101;
const WM_SYSKEYDOWN: u32 = 0x0104;
const WM_SYSKEYUP: u32 = 0x0105;
const WM_MOUSEMOVE: u32 = 0x0200;
const WM_LBUTTONDOWN: u32 = 0x0201;
const WM_LBUTTONUP: u32 = 0x0202;
const WM_RBUTTONDOWN: u32 = 0x0204;

const PM_REMOVE: u32 = 0x0001;
const SM_CXSCREEN: i32 = 0;
const SM_CYSCREEN: i32 = 1;
const IDC_ARROW: usize = 32512;
const BLACK_BRUSH: i32 = 4;
const COLORONCOLOR: i32 = 3;
const BI_RGB: u32 = 0;
const DIB_RGB_COLORS: u32 = 0;
const BLACKNESS: u32 = 0x0000_0042;
const GWL_STYLE: i32 = -16;
const SWP_NOOWNERZORDER: u32 = 0x0200;
const SWP_FRAMECHANGED: u32 = 0x0020;
const SW_SHOW: i32 = 5;

const VK_TAB: usize = 0x09;
const VK_RETURN: usize = 0x0D;
const VK_MENU: i32 = 0x12;
const VK_ESCAPE: usize = 0x1B;
const VK_SPACE: usize = 0x20;
const VK_LEFT: usize = 0x25;
const VK_UP: usize = 0x26;
const VK_RIGHT: usize = 0x27;
const VK_DOWN: usize = 0x28;
const VK_F2: usize = 0x71;
const VK_F3: usize = 0x72;
const VK_F11: usize = 0x7A;

#[link(name = "user32")]
extern "system" {
    fn SetProcessDPIAware() -> i32;
    fn GetModuleHandleW(lp_module_name: *const u16) -> HINSTANCE;
    fn LoadCursorW(h_instance: HINSTANCE, lp_cursor_name: *const u16) -> HCURSOR;
    fn RegisterClassExW(lpwcx: *const WndClassExW) -> ATOM;
    fn CreateWindowExW(
        dw_ex_style: u32,
        lp_class_name: *const u16,
        lp_window_name: *const u16,
        dw_style: u32,
        x: i32,
        y: i32,
        n_width: i32,
        n_height: i32,
        h_wnd_parent: HWND,
        h_menu: HMENU,
        h_instance: HINSTANCE,
        lp_param: *mut c_void,
    ) -> HWND;
    fn ShowWindow(h_wnd: HWND, n_cmd_show: i32) -> i32;
    fn UpdateWindow(h_wnd: HWND) -> i32;
    fn DefWindowProcW(h_wnd: HWND, msg: u32, w_param: WPARAM, l_param: LPARAM) -> LRESULT;
    fn PostQuitMessage(n_exit_code: i32);
    fn PeekMessageW(
        lp_msg: *mut Msg,
        h_wnd: HWND,
        w_msg_filter_min: u32,
        w_msg_filter_max: u32,
        w_remove_msg: u32,
    ) -> i32;
    fn TranslateMessage(lp_msg: *const Msg) -> i32;
    fn DispatchMessageW(lp_msg: *const Msg) -> LRESULT;
    fn GetSystemMetrics(n_index: i32) -> i32;
    fn AdjustWindowRect(lp_rect: *mut Rect, dw_style: u32, b_menu: i32) -> i32;
    fn GetClientRect(h_wnd: HWND, lp_rect: *mut Rect) -> i32;
    fn GetDC(h_wnd: HWND) -> HDC;
    fn ReleaseDC(h_wnd: HWND, h_dc: HDC) -> i32;
    fn BeginPaint(h_wnd: HWND, lp_paint: *mut PaintStruct) -> HDC;
    fn EndPaint(h_wnd: HWND, lp_paint: *const PaintStruct) -> i32;
    fn GetKeyState(n_virt_key: i32) -> i16;
    fn GetWindowLongW(h_wnd: HWND, n_index: i32) -> i32;
    fn SetWindowLongW(h_wnd: HWND, n_index: i32, dw_new_long: i32) -> i32;
    fn GetWindowPlacement(h_wnd: HWND, lpwndpl: *mut WindowPlacement) -> i32;
    fn SetWindowPlacement(h_wnd: HWND, lpwndpl: *const WindowPlacement) -> i32;
    fn SetWindowPos(
        h_wnd: HWND,
        h_wnd_insert_after: HWND,
        x: i32,
        y: i32,
        cx: i32,
        cy: i32,
        u_flags: u32,
    ) -> i32;
    fn DestroyWindow(h_wnd: HWND) -> i32;
}

#[link(name = "gdi32")]
extern "system" {
    fn GetStockObject(i: i32) -> HBRUSH;
    fn SetStretchBltMode(hdc: HDC, mode: i32) -> i32;
    fn PatBlt(hdc: HDC, x: i32, y: i32, w: i32, h: i32, rop: u32) -> i32;
    fn SetDIBitsToDevice(
        hdc: HDC,
        x_dest: i32,
        y_dest: i32,
        w: u32,
        h: u32,
        x_src: i32,
        y_src: i32,
        start_scan: u32,
        c_lines: u32,
        lpv_bits: *const c_void,
        lpbmi: *const BitmapInfo,
        color_use: u32,
    ) -> i32;
}

static mut QUIT_FLAG: bool = false;

extern "system" fn wnd_proc(hwnd: HWND, msg: u32, w_param: WPARAM, l_param: LPARAM) -> LRESULT {
    match msg {
        WM_DESTROY => {
            unsafe {
                QUIT_FLAG = true;
                PostQuitMessage(0);
            }
            0
        }
        WM_ERASEBKGND => 1,
        WM_PAINT => unsafe {
            let mut ps: PaintStruct = zeroed();
            BeginPaint(hwnd, &mut ps);
            EndPaint(hwnd, &ps);
            0
        },
        _ => unsafe { DefWindowProcW(hwnd, msg, w_param, l_param) },
    }
}

fn to_wide(s: &str) -> Vec<u16> {
    s.encode_utf16().chain(std::iter::once(0)).collect()
}

fn map_vk(vk: usize) -> Option<Key> {
    match vk {
        0x31 => Some(Key::Num1),
        0x32 => Some(Key::Num2),
        0x33 => Some(Key::Num3),
        0x34 => Some(Key::Num4),
        0x35 => Some(Key::Num5),
        0x36 => Some(Key::Num6),
        0x37 => Some(Key::Num7),
        0x51 => Some(Key::Q),
        0x45 => Some(Key::E),
        0x52 => Some(Key::R),
        0x55 => Some(Key::U),
        0x58 => Some(Key::X),
        0x4D => Some(Key::M),
        0x54 => Some(Key::T),
        0x50 => Some(Key::P),
        0x44 => Some(Key::D),
        0x56 => Some(Key::V),
        0x48 => Some(Key::H),
        0x46 => Some(Key::F),
        VK_SPACE => Some(Key::Space),
        VK_TAB => Some(Key::Tab),
        VK_RETURN => Some(Key::Enter),
        VK_ESCAPE => Some(Key::Esc),
        VK_UP => Some(Key::Up),
        VK_DOWN => Some(Key::Down),
        VK_LEFT => Some(Key::Left),
        VK_RIGHT => Some(Key::Right),
        VK_F2 => Some(Key::F2),
        VK_F3 => Some(Key::F3),
        VK_F11 => Some(Key::F11),
        _ => None,
    }
}

struct FullscreenToggle {
    is_fullscreen: bool,
    placement: WindowPlacement,
}

impl FullscreenToggle {
    fn new() -> Self {
        Self {
            is_fullscreen: false,
            placement: unsafe { zeroed() },
        }
    }

    fn toggle(&mut self, hwnd: HWND) {
        unsafe {
            let style = GetWindowLongW(hwnd, GWL_STYLE) as u32;
            if !self.is_fullscreen {
                self.placement.length = size_of::<WindowPlacement>() as u32;
                GetWindowPlacement(hwnd, &mut self.placement);
                let sw = GetSystemMetrics(SM_CXSCREEN);
                let sh = GetSystemMetrics(SM_CYSCREEN);
                SetWindowLongW(
                    hwnd,
                    GWL_STYLE,
                    ((style & !WS_OVERLAPPEDWINDOW) | WS_POPUP) as i32,
                );
                SetWindowPos(
                    hwnd,
                    0,
                    0,
                    0,
                    sw,
                    sh,
                    SWP_NOOWNERZORDER | SWP_FRAMECHANGED,
                );
                self.is_fullscreen = true;
            } else {
                SetWindowLongW(
                    hwnd,
                    GWL_STYLE,
                    ((style & !WS_POPUP) | WS_OVERLAPPEDWINDOW) as i32,
                );
                SetWindowPlacement(hwnd, &self.placement);
                SetWindowPos(
                    hwnd,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0x0001 | 0x0002 | SWP_NOOWNERZORDER | SWP_FRAMECHANGED,
                );
                self.is_fullscreen = false;
            }
        }
    }
}

fn viewport_metrics(hwnd: HWND) -> (i32, i32, i32, i32, i32) {
    let mut rc = Rect::default();
    unsafe {
        GetClientRect(hwnd, &mut rc);
    }
    let cw = (rc.right - rc.left).max(1);
    let ch = (rc.bottom - rc.top).max(1);
    let scale = Framebuffer::integer_scale_for_viewport(cw, ch);
    let vp_w = (FB_W as i32) * scale;
    let vp_h = (FB_H as i32) * scale;
    let ox = (cw - vp_w) / 2;
    let oy = (ch - vp_h) / 2;
    (cw, ch, scale, ox, oy)
}

fn present_to_window(
    hwnd: HWND,
    fb: &Framebuffer,
    scanlines: bool,
    ui_tick: u32,
    dib_buf: &mut Vec<u32>,
) {
    let (cw, ch, scale, ox, oy) = viewport_metrics(hwnd);
    let vp_w = (FB_W as i32) * scale;
    let vp_h = (FB_H as i32) * scale;
    fb.present_integer_scaled(scale, scanlines, ui_tick, dib_buf);

    let bmi = BitmapInfo {
        bmi_header: BitmapInfoHeader {
            bi_size: size_of::<BitmapInfoHeader>() as u32,
            bi_width: vp_w,
            bi_height: -vp_h, // Top-down DIB
            bi_planes: 1,
            bi_bit_count: 32,
            bi_compression: BI_RGB,
            bi_size_image: 0,
            bi_x_pels_per_meter: 0,
            bi_y_pels_per_meter: 0,
            bi_clr_used: 0,
            bi_clr_important: 0,
        },
        bmi_colors: [0],
    };

    unsafe {
        let hdc = GetDC(hwnd);
        if hdc != 0 {
            SetStretchBltMode(hdc, COLORONCOLOR);
            // Clear pure black CRT bezel margins around centered integer-scaled viewport
            if oy > 0 {
                PatBlt(hdc, 0, 0, cw, oy, BLACKNESS);
                PatBlt(hdc, 0, oy + vp_h, cw, ch - (oy + vp_h), BLACKNESS);
            }
            if ox > 0 {
                PatBlt(hdc, 0, oy, ox, vp_h, BLACKNESS);
                PatBlt(hdc, ox + vp_w, oy, cw - (ox + vp_w), vp_h, BLACKNESS);
            }
            SetDIBitsToDevice(
                hdc,
                ox,
                oy,
                vp_w as u32,
                vp_h as u32,
                0,
                0,
                0,
                vp_h as u32,
                dib_buf.as_ptr() as *const c_void,
                &bmi,
                DIB_RGB_COLORS,
            );
            ReleaseDC(hwnd, hdc);
        }
    }
}

pub fn run() {
    unsafe {
        SetProcessDPIAware();
    }

    let hinst = unsafe { GetModuleHandleW(ptr::null()) };
    let class_name = to_wide("CitadelGridClass");
    let title = to_wide("CITADEL GRID - Retro MS-DOS Tower RTS");

    let wc = WndClassExW {
        cb_size: size_of::<WndClassExW>() as u32,
        style: CS_OWNDC,
        lpfn_wnd_proc: wnd_proc,
        cb_cls_extra: 0,
        cb_wnd_extra: 0,
        h_instance: hinst,
        h_icon: 0,
        h_cursor: unsafe { LoadCursorW(0, IDC_ARROW as *const u16) },
        hbr_background: unsafe { GetStockObject(BLACK_BRUSH) },
        lpsz_menu_name: ptr::null(),
        lpsz_class_name: class_name.as_ptr(),
        h_icon_sm: 0,
    };

    if unsafe { RegisterClassExW(&wc) } == 0 {
        return;
    }

    let (sw, sh) = unsafe { (GetSystemMetrics(SM_CXSCREEN), GetSystemMetrics(SM_CYSCREEN)) };
    let init_scale = Framebuffer::integer_scale_for_viewport((sw - 40).max(400), (sh - 80).max(300)).min(3);
    let client_w = (FB_W as i32) * init_scale;
    let client_h = (FB_H as i32) * init_scale;

    let mut wr = Rect {
        left: 0,
        top: 0,
        right: client_w,
        bottom: client_h,
    };
    unsafe {
        AdjustWindowRect(&mut wr, WS_OVERLAPPEDWINDOW, 0);
    }
    let win_w = wr.right - wr.left;
    let win_h = wr.bottom - wr.top;
    let win_x = ((sw - win_w) / 2).max(CW_USEDEFAULT.min(0));
    let win_y = ((sh - win_h) / 2).max(0);

    let hwnd = unsafe {
        CreateWindowExW(
            0,
            class_name.as_ptr(),
            title.as_ptr(),
            WS_OVERLAPPEDWINDOW | WS_VISIBLE,
            win_x,
            win_y,
            win_w,
            win_h,
            0,
            0,
            hinst,
            ptr::null_mut(),
        )
    };
    if hwnd == 0 {
        return;
    }

    unsafe {
        ShowWindow(hwnd, SW_SHOW);
        UpdateWindow(hwnd);
    }

    let mut app = App::new();
    let mut input = InputState::default();
    let mut fb = Framebuffer::new();
    let mut dib_buf: Vec<u32> = Vec::new();
    let mut audio = AudioEngine::new();
    let mut fs = FullscreenToggle::new();

    app.render(&mut fb);
    present_to_window(hwnd, &fb, app.scanlines, app.ui_tick, &mut dib_buf);

    // 15 FPS discrete tick (66.667 ms) + 250 Hz (4 ms) input polling
    let tick_dur = Duration::from_nanos(66_666_667);
    let poll_dur = Duration::from_millis(4);
    let mut last_time = Instant::now();
    let mut acc = Duration::ZERO;

    loop {
        unsafe {
            if QUIT_FLAG {
                break;
            }
            let mut msg: Msg = zeroed();
            while PeekMessageW(&mut msg, 0, 0, 0, PM_REMOVE) != 0 {
                TranslateMessage(&msg);
                match msg.message {
                    WM_KEYDOWN | WM_SYSKEYDOWN => {
                        let vk = msg.w_param;
                        let alt_down = GetKeyState(VK_MENU) < 0;
                        if vk == VK_RETURN && alt_down {
                            fs.toggle(hwnd);
                        } else if let Some(k) = map_vk(vk) {
                            input.set_key(k, true);
                        }
                    }
                    WM_KEYUP | WM_SYSKEYUP => {
                        if let Some(k) = map_vk(msg.w_param) {
                            input.set_key(k, false);
                        }
                    }
                    WM_MOUSEMOVE | WM_LBUTTONDOWN | WM_LBUTTONUP | WM_RBUTTONDOWN => {
                        let px = (msg.l_param & 0xFFFF) as i16 as i32;
                        let py = ((msg.l_param >> 16) & 0xFFFF) as i16 as i32;
                        let (_, _, scale, ox, oy) = viewport_metrics(hwnd);
                        let lx = (px - ox).div_euclid(scale);
                        let ly = (py - oy).div_euclid(scale);
                        if lx >= 0 && lx < FB_W as i32 && ly >= 0 && ly < FB_H as i32 {
                            input.set_mouse_pos(lx, ly);
                        } else {
                            input.set_mouse_pos(-1, -1);
                        }
                        if msg.message == WM_LBUTTONDOWN {
                            input.set_left_button(true);
                        } else if msg.message == WM_LBUTTONUP {
                            input.set_left_button(false);
                        } else if msg.message == WM_RBUTTONDOWN {
                            input.latch_right_click();
                        }
                    }
                    _ => {}
                }
                DispatchMessageW(&msg);
            }
        }

        let now = Instant::now();
        let dt = now.saturating_duration_since(last_time);
        last_time = now;
        acc += dt;
        if acc > Duration::from_millis(200) {
            acc = tick_dur;
        }

        if acc >= tick_dur {
            acc -= tick_dur;
            app.tick(&mut input);
            if app.toggle_fullscreen_req {
                app.toggle_fullscreen_req = false;
                fs.toggle(hwnd);
            }
            audio.muted = app.muted;
            for sfx in app.pending_sfx.drain(..) {
                audio.play(sfx);
            }
            if app.should_quit {
                unsafe {
                    DestroyWindow(hwnd);
                }
                break;
            }
            app.render(&mut fb);
            present_to_window(hwnd, &fb, app.scanlines, app.ui_tick, &mut dib_buf);
        }

        std::thread::sleep(poll_dur);
    }
}
