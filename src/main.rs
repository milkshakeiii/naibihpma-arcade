mod audio;
mod framebuffer;
mod game;
mod palette;
mod sprites;

use audio::ArcadeAudio;
use framebuffer::{Framebuffer, WIN_H, WIN_W};
use game::{
    AirborneMode, BagItem, FloatingText, GameMode, GameState, IngKind, InputState, PatronStage,
    PrepState, RestaurantUpgrade, STOOL_X,
};
use palette::VgaPalette;
use sprites::AssetBank;
use std::ffi::OsStr;
use std::mem::{size_of, zeroed};
use std::os::windows::ffi::OsStrExt;
use std::path::Path;
use std::ptr::null_mut;
use std::time::{Duration, Instant};

use winapi::shared::minwindef::{LPARAM, LRESULT, UINT, WPARAM};
use winapi::shared::windef::{HDC, HWND, RECT};
use winapi::um::libloaderapi::GetModuleHandleW;
use winapi::um::wingdi::{
    GetStockObject, PatBlt, SetStretchBltMode, StretchDIBits, BITMAPINFO, BITMAPINFOHEADER,
    BI_RGB, BLACKNESS, BLACK_BRUSH, COLORONCOLOR, DIB_RGB_COLORS, SRCCOPY,
};
use winapi::um::winuser::*;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.iter().any(|a| a == "--export-screenshots") {
        export_verification_screenshots();
        return;
    }

    unsafe {
        SetProcessDPIAware();

        let hinstance = GetModuleHandleW(null_mut());
        let class_name: Vec<u16> = OsStr::new("NaibihpmaArcadeWindowClass")
            .encode_wide()
            .chain(Some(0))
            .collect();

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

        let screen_w = GetSystemMetrics(SM_CXSCREEN);
        let screen_h = GetSystemMetrics(SM_CYSCREEN);

        let style = WS_POPUP | WS_VISIBLE;

        let title: Vec<u16> = OsStr::new(
            "INTERSTELLAR FORAGER: 15 FPS TUTORIAL & VERTICAL SLICE [800x600 VGA]",
        )
        .encode_wide()
        .chain(Some(0))
        .collect();

        let hwnd = CreateWindowExW(
            0,
            class_name.as_ptr(),
            title.as_ptr(),
            style,
            0,
            0,
            screen_w,
            screen_h,
            null_mut(),
            null_mut(),
            hinstance,
            null_mut(),
        );

        if hwnd.is_null() {
            return;
        }

        let hdc: HDC = GetDC(hwnd);
        SetStretchBltMode(hdc, COLORONCOLOR);

        let mut bmi: BITMAPINFO = zeroed();
        bmi.bmiHeader.biSize = size_of::<BITMAPINFOHEADER>() as u32;
        bmi.bmiHeader.biWidth = WIN_W as i32;
        bmi.bmiHeader.biHeight = -(WIN_H as i32);
        bmi.bmiHeader.biPlanes = 1;
        bmi.bmiHeader.biBitCount = 32;
        bmi.bmiHeader.biCompression = BI_RGB;

        let mut pal = VgaPalette::new_naibihpma();
        let assets = AssetBank::load_and_compile(&pal);
        let mut audio = ArcadeAudio::new();
        let mut game = GameState::new();
        let mut fb = Framebuffer::new();
        let mut out_800x600 = vec![0u32; WIN_W * WIN_H];

        // High-frequency input edge trackers (sampled every 4ms so a tap is NEVER missed between 15 FPS ticks!)
        let mut prev_left = false;
        let mut prev_right = false;
        let mut prev_up = false;
        let mut prev_down = false;
        let mut prev_jump = false;
        let mut prev_action = false;
        let mut prev_bomb = false;
        let mut prev_extract = false;
        let mut prev_help = false;
        let mut prev_mute = false;
        let mut prev_f2 = false;
        let mut prev_lbracket = false;
        let mut prev_rbracket = false;
        let mut prev_num = [false; 6];

        // Latched events accumulated until the next 15 FPS game tick
        let mut latched = InputState::default();

        // Classic DOS / Apogee 15 FPS discrete tick duration (~66.6ms)
        let tick_dur = Duration::from_micros(66_667);
        let poll_sleep = Duration::from_millis(4);
        let mut last_tick = Instant::now();
        let mut msg: MSG = zeroed();

        // Render initial frame immediately
        let track_title = audio.current_track_title();
        game.render(&mut fb, &assets, &track_title);
        fb.present_to_800x600(&mut out_800x600, &pal, game.crt_scanlines);

        'running: loop {
            while PeekMessageW(&mut msg, null_mut(), 0, 0, PM_REMOVE) != 0 {
                if msg.message == WM_QUIT {
                    break 'running;
                }
                TranslateMessage(&msg);
                DispatchMessageW(&msg);
            }

            if key_down(VK_ESCAPE) {
                break 'running;
            }

            // Sample immediate buttons (F2, Mute, Track Skip)
            let f2 = key_down(VK_F2);
            if f2 && !prev_f2 {
                game.crt_scanlines = !game.crt_scanlines;
            }
            prev_f2 = f2;

            let mute_now = key_down('M' as i32);
            if mute_now && !prev_mute {
                audio.toggle_sfx_mute();
            }
            prev_mute = mute_now;

            let lbr = key_down(0xDB);
            if lbr && !prev_lbracket {
                audio.prev_track();
            }
            prev_lbracket = lbr;

            let rbr = key_down(0xDD);
            if rbr && !prev_rbracket {
                audio.next_track();
            }
            prev_rbracket = rbr;

            // Sample gameplay keys at 250Hz and latch rising edges for the next 15 FPS tick!
            let left_now = key_down(VK_LEFT) || key_down('A' as i32);
            let right_now = key_down(VK_RIGHT) || key_down('D' as i32);
            let up_now = key_down(VK_UP) || key_down('W' as i32);
            let down_now = key_down(VK_DOWN) || key_down('S' as i32);
            let jump_now = key_down(VK_SPACE) || key_down('Z' as i32);
            let action_now = key_down('X' as i32) || key_down('J' as i32);
            let bomb_now = key_down('C' as i32) || key_down('K' as i32);
            let extract_now = key_down('E' as i32) || key_down(VK_RETURN);
            let help_now = key_down('H' as i32) || key_down(VK_F1);

            if left_now && !prev_left {
                latched.left_pressed = true;
            }
            if right_now && !prev_right {
                latched.right_pressed = true;
            }
            if up_now && !prev_up {
                latched.up_pressed = true;
            }
            if down_now && !prev_down {
                latched.down_pressed = true;
            }
            if jump_now && !prev_jump {
                latched.jump_pressed = true;
            }
            if action_now && !prev_action {
                latched.action_pressed = true;
            }
            if bomb_now && !prev_bomb {
                latched.bomb_pressed = true;
            }
            if extract_now && !prev_extract {
                latched.extract_pressed = true;
            }
            if help_now && !prev_help {
                latched.help_pressed = true;
            }

            // Number keys 1..6 to warp directly to Tutorial Step 1..6
            for i in 0..6 {
                let num_down = key_down(('1' as i32) + i as i32);
                if num_down && !prev_num[i] {
                    latched.warp_step = Some(i + 1);
                }
                prev_num[i] = num_down;
            }

            // Also latch held states if pressed at any point during the tick window
            if left_now {
                latched.left = true;
            }
            if right_now {
                latched.right = true;
            }
            if up_now {
                latched.up = true;
            }
            if down_now {
                latched.down = true;
            }
            if jump_now {
                latched.jump_held = true;
            }

            prev_left = left_now;
            prev_right = right_now;
            prev_up = up_now;
            prev_down = down_now;
            prev_jump = jump_now;
            prev_action = action_now;
            prev_bomb = bomb_now;
            prev_extract = extract_now;
            prev_help = help_now;

            // Execute 15 FPS discrete tick when 66.6ms has elapsed!
            if last_tick.elapsed() >= tick_dur {
                last_tick = Instant::now();

                // Combine currently held keys with latched presses from the 66ms window
                latched.left = latched.left || left_now;
                latched.right = latched.right || right_now;
                latched.up = latched.up || up_now;
                latched.down = latched.down || down_now;
                latched.jump_held = latched.jump_held || jump_now;

                game.update_15fps_tick(&latched, &mut audio, &mut pal);
                latched = InputState::default();

                let track_title = audio.current_track_title();
                game.render(&mut fb, &assets, &track_title);
                fb.present_to_800x600(&mut out_800x600, &pal, game.crt_scanlines);

                let mut client_rect: RECT = zeroed();
                GetClientRect(hwnd, &mut client_rect);
                let cw = (client_rect.right - client_rect.left).max(1);
                let ch = (client_rect.bottom - client_rect.top).max(1);

                // Preserve crisp 4:3 square pixels at 100% monitor height, centered with black pillarboxes
                let dst_h = ch;
                let dst_w = ((ch * 4) / 3).min(cw);
                let dst_x = (cw - dst_w) / 2;

                if dst_x > 0 {
                    PatBlt(hdc, 0, 0, dst_x, ch, BLACKNESS);
                    PatBlt(hdc, dst_x + dst_w, 0, cw - (dst_x + dst_w), ch, BLACKNESS);
                }

                StretchDIBits(
                    hdc,
                    dst_x,
                    0,
                    dst_w,
                    dst_h,
                    0,
                    0,
                    WIN_W as i32,
                    WIN_H as i32,
                    out_800x600.as_ptr() as *const _,
                    &bmi,
                    DIB_RGB_COLORS,
                    SRCCOPY,
                );
            } else {
                std::thread::sleep(poll_sleep);
            }
        }

        ReleaseDC(hwnd, hdc);
    }
}

fn key_down(vk: i32) -> bool {
    unsafe { (GetAsyncKeyState(vk) as u16 & 0x8000) != 0 }
}

unsafe extern "system" fn wnd_proc(
    hwnd: HWND,
    msg: UINT,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    match msg {
        WM_DESTROY | WM_CLOSE => {
            PostQuitMessage(0);
            0
        }
        _ => DefWindowProcW(hwnd, msg, wparam, lparam),
    }
}

fn export_verification_screenshots() {
    let _ = std::fs::create_dir_all("screenshots");
    let mut pal = VgaPalette::new_naibihpma();
    let assets = AssetBank::load_and_compile(&pal);
    let mut game = GameState::new();
    let mut fb = Framebuffer::new();
    let track_label = "1/6 THE UNDERBRUSH";

    pal.update_cycler(16);
    game.mode = GameMode::TitleAttractor;
    game.render(&mut fb, &assets, track_label);
    fb.save_png_800x600(
        Path::new("screenshots/01_title_attractor.png"),
        &pal,
        true,
    );

    // Level 1: The Underbrush (Shadow Alcoves, Bubble-Reed O2 Vent, Sluice Lever & Skittish Newt!)
    game.stage_num = 1;
    game.build_stage_for_current_level();
    game.mode = GameMode::PlanetaryDrop;
    game.px = 23 * 24;
    game.py = 10 * 24 - 44;
    game.air_mode = AirborneMode::Grounded;
    game.cam_x = (game.px + 20 - 200).clamp(0, 80 * 24 - 400);
    game.cam_y = (game.py + 22 - 118).clamp(0, 22 * 24 - 236);
    game.render(&mut fb, &assets, "1/6 THE UNDERBRUSH");
    fb.save_png_800x600(
        Path::new("screenshots/02_naibihpma_harvest.png"),
        &pal,
        true,
    );

    // Level 2: Daot the Enchanter (Enchanter Toad firing Spore-Orb, Shadow Alcove & Grapple Cliff!)
    game.stage_num = 2;
    game.build_stage_for_current_level();
    game.mode = GameMode::PlanetaryDrop;
    game.px = 7 * 24;
    game.py = 14 * 24 - 44;
    game.air_mode = AirborneMode::Grounded;
    game.orbs.push(game::SporeOrb {
        x: 11 * 24,
        y: 14 * 24 - 18,
        vx: -9,
        vy: 0,
        life: 18,
    });
    game.cam_x = (game.px + 20 - 200).clamp(0, 80 * 24 - 400);
    game.cam_y = (game.py + 22 - 118).clamp(0, 22 * 24 - 236);
    game.render(&mut fb, &assets, "2/6 DAOT THE ENCHANTER");
    fb.save_png_800x600(
        Path::new("screenshots/03_level2_daot_enchanter.png"),
        &pal,
        true,
    );

    let dummy_audio = ArcadeAudio::new_muted();
    game.upgrades.push(RestaurantUpgrade::TwinBurnerHearth);
    game.backpack = vec![
        BagItem {
            kind: IngKind::DewNewt,
            prep: PrepState::Fresh,
            ionized: true,
        },
        BagItem {
            kind: IngKind::SporeLotus,
            prep: PrepState::CharSeared,
            ionized: false,
        },
        BagItem {
            kind: IngKind::VocalRoe,
            prep: PrepState::Tenderized,
            ionized: false,
        },
        BagItem {
            kind: IngKind::ReedEel,
            prep: PrepState::Fresh,
            ionized: true,
        },
    ];
    game.enter_menu_prep(&dummy_audio);
    let dish1 = game.combine_pair_into_dish(game.backpack[0], game.backpack[1]);
    game.menu_dishes.push(dish1);
    game.backpack.drain(0..2);
    game.selected_first_ing = Some(0);
    game.prep_cursor = 1;
    game.render(&mut fb, &assets, "3/6 STAR POND");
    fb.save_png_800x600(
        Path::new("screenshots/03_menu_prep.png"),
        &pal,
        true,
    );

    // Phase 2B: Dave the Diver-style Side-View Tavern Service
    game.start_live_tavern_service(&dummy_audio);
    // Advance patrons onto stools and put 1 dish on the pass & 1 in Cosmo's hands
    for (i, p) in game.patrons.iter_mut().enumerate() {
        p.x = STOOL_X[p.stool_idx];
        p.stage = if i == 0 {
            PatronStage::Eating(6)
        } else {
            PatronStage::SeatedOrdering
        };
    }
    if let Some(d) = game.cooking_queue.pop() {
        game.carried_dish = Some(d.clone());
        game.ready_pass_dishes.push(d.clone());
        game.active_cook_dish = Some(d);
        game.cook_progress = 7;
    }
    game.chef_bar_x = STOOL_X[1] - 6;
    game.chef_facing_right = true;
    game.gold = 215;
    game.night_earnings = 215;
    game.dishes_served_tonight = 1;
    game.popups.push(FloatingText {
        text: "+$215 SERVED!".to_string(),
        x: STOOL_X[0] - 18,
        y: 112,
        col: 15,
        life: 40,
    });
    game.render(&mut fb, &assets, "4/6 DOAT THE ENCHANTER");
    fb.save_png_800x600(
        Path::new("screenshots/04_orbital_galley.png"),
        &pal,
        true,
    );

    // Phase 3: Restaurant Upgrade Shop
    game.mode = GameMode::UpgradeShop;
    game.shop_cursor = 1;
    game.render(&mut fb, &assets, "5/6 ANCIENT MAGIC");
    fb.save_png_800x600(
        Path::new("screenshots/05_upgrade_shop.png"),
        &pal,
        true,
    );

    println!("Exported 15 FPS tutorial & Dave-the-Diver Galley screenshots to screenshots/");
}
