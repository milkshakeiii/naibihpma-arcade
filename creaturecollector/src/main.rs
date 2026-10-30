mod audio;
mod battle;
mod data;
mod framebuffer;
mod game;
mod palette;
mod sprites;

use audio::{ArcadeAudio, Sfx};
use battle::{BattleCmd, BattlePhase, BattleState, PopBadge, Side, VisualArc};
use data::{ArtifactId, CreatureInstance, SpeciesId, SpellGemId};
use framebuffer::Framebuffer;
use game::{GameMode, GameState, InputState, SquadSubMode, TS};
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
        let class_name: Vec<u16> = OsStr::new("NaibihpmaRogueWindowClass")
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
            "NAIBIHPMA: XENOBIOLOGY ROGUELIKE [400x300 64-COLOR VGA | 15 FPS]",
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
        bmi.bmiHeader.biPlanes = 1;
        bmi.bmiHeader.biBitCount = 32;
        bmi.bmiHeader.biCompression = BI_RGB;

        let mut pal = VgaPalette::new_naibihpma();
        let assets = AssetBank::load_and_compile(&pal);
        let mut audio = ArcadeAudio::new();
        let mut game = GameState::new();
        let mut fb = Framebuffer::new();
        let mut out_scaled: Vec<u32> = Vec::new();
        let mut sfx_buf: Vec<Sfx> = Vec::with_capacity(8);

        // 250 Hz high-frequency edge trackers so quick taps are never dropped between 15 FPS ticks!
        let mut prev_left = false;
        let mut prev_right = false;
        let mut prev_up = false;
        let mut prev_down = false;
        let mut prev_confirm = false;
        let mut prev_cancel = false;
        let mut prev_special = false;
        let mut prev_squad = false;
        let mut prev_help = false;
        let mut prev_retry = false;
        let mut prev_esc = false;
        let mut prev_y = false;
        let mut prev_mute = false;
        let mut prev_f2 = false;
        let mut prev_lbracket = false;
        let mut prev_rbracket = false;
        let mut prev_num = [false; 3];

        let mut latched = InputState::default();

        let tick_dur = Duration::from_micros(66_667);
        let poll_sleep = Duration::from_millis(4);
        let mut last_tick = Instant::now()
            .checked_sub(tick_dur)
            .unwrap_or_else(Instant::now);
        let mut msg: MSG = zeroed();

        'running: loop {
            while PeekMessageW(&mut msg, null_mut(), 0, 0, PM_REMOVE) != 0 {
                if msg.message == WM_QUIT {
                    break 'running;
                }
                TranslateMessage(&msg);
                DispatchMessageW(&msg);
            }

            // Immediate display & audio toggles (F2 scanlines, M mute SFX, [ / ] track skip)
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

            // Sample gameplay keys at 250 Hz (both Arrows + Z/X/C and WASD + Enter/Space/E/Tab!)
            let left_now = key_down(VK_LEFT) || key_down('A' as i32);
            let right_now = key_down(VK_RIGHT) || key_down('D' as i32);
            let up_now = key_down(VK_UP) || key_down('W' as i32);
            let down_now = key_down(VK_DOWN) || key_down('S' as i32);
            let confirm_now = key_down(VK_RETURN)
                || key_down(VK_SPACE)
                || key_down('Z' as i32)
                || key_down('E' as i32)
                || key_down('J' as i32);
            let cancel_now =
                key_down('X' as i32) || key_down(VK_BACK) || key_down('K' as i32);
            let special_now = key_down('T' as i32) || key_down('C' as i32);
            let squad_now = key_down(VK_TAB) || key_down('P' as i32);
            let help_now = key_down('H' as i32) || key_down(VK_F1);
            let retry_now = key_down('R' as i32);
            let esc_now = key_down(VK_ESCAPE);
            let y_now = key_down('Y' as i32);

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
            if confirm_now && !prev_confirm {
                latched.confirm_pressed = true;
            }
            if cancel_now && !prev_cancel {
                latched.cancel_pressed = true;
            }
            if special_now && !prev_special {
                latched.special_pressed = true;
            }
            if squad_now && !prev_squad {
                latched.squad_pressed = true;
            }
            if help_now && !prev_help {
                latched.help_pressed = true;
            }
            if retry_now && !prev_retry {
                latched.retry_pressed = true;
            }
            if esc_now && !prev_esc {
                latched.esc_pressed = true;
            }
            if y_now && !prev_y {
                latched.quit_yes_pressed = true;
            }

            for i in 0..3 {
                let num_down = key_down(('1' as i32) + i as i32);
                if num_down && !prev_num[i] {
                    latched.warp_zone = Some(i + 1);
                }
                prev_num[i] = num_down;
            }

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

            prev_left = left_now;
            prev_right = right_now;
            prev_up = up_now;
            prev_down = down_now;
            prev_confirm = confirm_now;
            prev_cancel = cancel_now;
            prev_special = special_now;
            prev_squad = squad_now;
            prev_help = help_now;
            prev_retry = retry_now;
            prev_esc = esc_now;
            prev_y = y_now;

            // Advance discrete 15 FPS simulation tick every 66.667 ms
            if last_tick.elapsed() >= tick_dur {
                last_tick = Instant::now();

                latched.left = latched.left || left_now;
                latched.right = latched.right || right_now;
                latched.up = latched.up || up_now;
                latched.down = latched.down || down_now;

                sfx_buf.clear();
                game.update(&latched, &mut sfx_buf);
                latched = InputState::default();

                if game.should_exit_app {
                    break 'running;
                }

                if let Some(track_idx) = game.desired_bgm_track.take() {
                    audio.set_track(track_idx);
                }
                for sfx in sfx_buf.drain(..) {
                    audio.play_sfx(sfx);
                }
                pal.update_cycler(game.tick_count);

                let track_title = audio.current_track_title();
                game.render(&mut fb, &assets, &track_title);

                let mut client_rect: RECT = zeroed();
                GetClientRect(hwnd, &mut client_rect);
                let cw = (client_rect.right - client_rect.left).max(1);
                let ch = (client_rect.bottom - client_rect.top).max(1);

                let scale = Framebuffer::integer_scale_for_viewport(cw, ch);
                let (scaled_w, scaled_h) =
                    fb.present_integer_scaled(&mut out_scaled, scale, &pal, game.crt_scanlines);

                let dst_w = scaled_w as i32;
                let dst_h = scaled_h as i32;
                let dst_x = (cw - dst_w) / 2;
                let dst_y = (ch - dst_h) / 2;

                bmi.bmiHeader.biWidth = dst_w;
                bmi.bmiHeader.biHeight = -dst_h;

                if dst_y > 0 {
                    PatBlt(hdc, 0, 0, cw, dst_y, BLACKNESS);
                }
                let bottom_y = dst_y + dst_h;
                if bottom_y < ch {
                    PatBlt(hdc, 0, bottom_y, cw, ch - bottom_y, BLACKNESS);
                }
                if dst_x > 0 {
                    PatBlt(hdc, 0, dst_y.max(0), dst_x, dst_h.min(ch), BLACKNESS);
                }
                let right_x = dst_x + dst_w;
                if right_x < cw {
                    PatBlt(hdc, right_x, dst_y.max(0), cw - right_x, dst_h.min(ch), BLACKNESS);
                }

                StretchDIBits(
                    hdc,
                    dst_x,
                    dst_y,
                    dst_w,
                    dst_h,
                    0,
                    0,
                    dst_w,
                    dst_h,
                    out_scaled.as_ptr() as *const _,
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
    pal.update_cycler(16);

    // 1. Roguelike Starter Draft Screen (Pick 1 of 3 unlocked Bestiary specimens!)
    game.draft_species_offers = [
        SpeciesId::DewNewt,
        SpeciesId::SporeMorel,
        SpeciesId::VoltaicEel,
    ];
    game.render(&mut fb, &assets, "1/6 THE UNDERBRUSH");
    fb.save_png_800x600(Path::new("screenshots/01_starter_draft.png"), &pal, true);

    // 2. Side-View Pokemon Habitat Exploration (Fern Underbrush, Star-Pond Shallows & Crystal Grotto!)
    game.squad[0] = Some({
        let mut c = CreatureInstance::new(SpeciesId::BriarToad, 4, false);
        c.artifact = Some(ArtifactId::IronbarkBuckler);
        c.heal_full();
        c
    });
    game.squad[1] = Some(CreatureInstance::new(SpeciesId::DewNewt, 4, false));
    game.squad[3] = Some({
        let mut c = CreatureInstance::new(SpeciesId::SporeMorel, 4, false);
        c.artifact = Some(ArtifactId::SporeCrown);
        c.heal_full();
        c
    });
    game.squad[4] = Some(CreatureInstance::new(SpeciesId::VoltaicEel, 4, false));
    game.reserve[0] = Some(CreatureInstance::new(SpeciesId::RuneBeetle, 4, false));

    game.load_zone(1);
    game.mode = GameMode::SideWorld;
    game.px = 14 * TS;
    game.py = 6 * TS;
    game.cam_x = 6 * TS;
    game.encounter_meter = 64;
    game.render(&mut fb, &assets, "1/6 THE UNDERBRUSH");
    fb.save_png_800x600(
        Path::new("screenshots/02_sideview_habitats.png"),
        &pal,
        true,
    );

    // 3. 3x2 vs 3x2 Siralim Ultimate Cascading Trait-Chain Battle + <=40% HP Tactical Capture Window!
    let mut enemies: [Option<CreatureInstance>; 6] = [None, None, None, None, None, None];
    enemies[0] = Some(CreatureInstance::new(SpeciesId::LanternCrab, 4, false));
    enemies[1] = Some(CreatureInstance::new(SpeciesId::CrownSalamander, 5, true)); // +2 Lv Alpha!
    enemies[3] = Some(CreatureInstance::new(SpeciesId::DewFrog, 4, false));
    enemies[4] = Some(CreatureInstance::new(SpeciesId::GloomMoth, 4, false));

    let mut bs = BattleState::new(
        &game.squad,
        enemies,
        false,
        1,
        "ALPHA GROVE PACK".to_string(),
        99,
    );
    // Showcase Alpha Crown Salamander chipped down to 31% HP (<=40% Capture Ready!) + active chain arc!
    if let Some(eu) = bs.unit_mut(Side::Enemy, 1) {
        eu.cur_hp = (eu.max_hp * 31) / 100;
        eu.poison = 2;
    }
    if let Some(au) = bs.unit_mut(Side::Ally, 3) {
        au.defending_hold_fire = true;
    }
    bs.chain_depth = 3;
    bs.log_line_1 = "> [SOOTHING DEW] HEALS MOREL +11 HP!".to_string();
    bs.log_line_2 = "> [MYCELIAL BLOOM] DEALS 14 DMG + PSN TO *CRNSALM!".to_string();
    bs.arcs.push(VisualArc {
        from: (Side::Ally, 1),
        to: (Side::Ally, 3),
        col: 25,
        ticks: 5,
    });
    bs.arcs.push(VisualArc {
        from: (Side::Ally, 3),
        to: (Side::Enemy, 1),
        col: 30,
        ticks: 5,
    });
    bs.pops.push(PopBadge {
        side: Side::Enemy,
        slot: 1,
        text: "-14".to_string(),
        col: 15,
        ticks: 9,
    });
    bs.cmd_cursor = 4; // Highlight CAPTURE (4)
    bs.target_cursor = 1;
    bs.phase = BattlePhase::WaitingEnemyTarget {
        ally_slot: 1,
        cmd: BattleCmd::Capture,
    };
    game.active_battle = Some(bs);
    game.mode = GameMode::Battle;
    game.render(&mut fb, &assets, "3/6 STAR POND");
    fb.save_png_800x600(
        Path::new("screenshots/03_siralim_battle_chain.png"),
        &pal,
        true,
    );

    // 4. 3x2 Formation, 2-Slot Reserve & Pre-Loaded Gear Manager ([TAB])
    game.gem_bag = vec![SpellGemId::PrecisionDart, SpellGemId::StarlightChoir];
    game.artifact_bag = vec![ArtifactId::TrapperLens, ArtifactId::CopperDynamo];
    game.mode = GameMode::SquadMenu {
        cursor: 0,
        sub: SquadSubMode::BrowseSlots,
        bag_cursor: 0,
    };
    game.render(&mut fb, &assets, "1/6 THE UNDERBRUSH");
    fb.save_png_800x600(Path::new("screenshots/04_squad_and_gear.png"), &pal, true);

    println!("Exported Roguelike Siralim x Side-View Pokemon verification screenshots to screenshots/");
}
