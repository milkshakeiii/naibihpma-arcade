use std::fs;
use std::path::Path;

use towers::ai::search::{choose_action, AiDifficulty, BotArchetype};
use towers::content::challenges::all_challenges;
use towers::content::maps::build_maps;
use towers::gfx::render_game::{
    render_battle, render_challenges_menu, render_skirmish_setup, render_title, ActiveTool,
    BattleView, EnergyOverlayMode,
};
use towers::gfx::{Framebuffer, FB_H, FB_W};
use towers::sim::rng::SimRng;
use towers::sim::types::*;
use towers::sim::GameState;

fn crc32(data: &[u8]) -> u32 {
    let mut crc = 0xFFFF_FFFFu32;
    for &b in data {
        crc ^= b as u32;
        for _ in 0..8 {
            let mask = (crc & 1).wrapping_neg();
            crc = (crc >> 1) ^ (0xEDB8_8320 & mask);
        }
    }
    !crc
}

fn adler32(data: &[u8]) -> u32 {
    let mut a = 1u32;
    let mut b = 0u32;
    for &byte in data {
        a = (a + byte as u32) % 65521;
        b = (b + a) % 65521;
    }
    (b << 16) | a
}

fn write_chunk(out: &mut Vec<u8>, tag: &[u8; 4], data: &[u8]) {
    out.extend_from_slice(&(data.len() as u32).to_be_bytes());
    let start = out.len();
    out.extend_from_slice(tag);
    out.extend_from_slice(data);
    let crc = crc32(&out[start..]);
    out.extend_from_slice(&crc.to_be_bytes());
}

fn save_png_xrgb(path: &str, w: usize, h: usize, xrgb: &[u32]) {
    let mut raw = Vec::with_capacity(h * (1 + w * 3));
    for y in 0..h {
        raw.push(0); // PNG filter type 0 (None)
        let row = &xrgb[(y * w)..((y + 1) * w)];
        for &px in row {
            raw.push(((px >> 16) & 0xFF) as u8);
            raw.push(((px >> 8) & 0xFF) as u8);
            raw.push((px & 0xFF) as u8);
        }
    }

    // Wrap raw scanlines in uncompressed zlib DEFLATE blocks (max 65535 bytes/block)
    let mut zlib = Vec::with_capacity(raw.len() + 64);
    zlib.push(0x78); // CMF: Deflate, 32K window
    zlib.push(0x01); // FLG: no preset dict, fastest
    let mut pos = 0usize;
    while pos < raw.len() {
        let len = (raw.len() - pos).min(65535);
        let is_final = if pos + len >= raw.len() { 1u8 } else { 0u8 };
        zlib.push(is_final);
        let l16 = len as u16;
        zlib.extend_from_slice(&l16.to_le_bytes());
        zlib.extend_from_slice(&(!l16).to_le_bytes());
        zlib.extend_from_slice(&raw[pos..(pos + len)]);
        pos += len;
    }
    zlib.extend_from_slice(&adler32(&raw).to_be_bytes());

    let mut png = Vec::with_capacity(zlib.len() + 64);
    png.extend_from_slice(&[137, 80, 78, 71, 13, 10, 26, 10]);
    let mut ihdr = Vec::with_capacity(13);
    ihdr.extend_from_slice(&(w as u32).to_be_bytes());
    ihdr.extend_from_slice(&(h as u32).to_be_bytes());
    ihdr.push(8); // bit depth
    ihdr.push(2); // RGB
    ihdr.push(0);
    ihdr.push(0);
    ihdr.push(0);
    write_chunk(&mut png, b"IHDR", &ihdr);
    write_chunk(&mut png, b"IDAT", &zlib);
    write_chunk(&mut png, b"IEND", &[]);
    fs::write(path, png).expect("Failed to write PNG screenshot");
}

fn save_fb(path: &str, fb: &Framebuffer, scale: i32, scanlines: bool, tick: u32) {
    let mut buf = Vec::new();
    fb.present_integer_scaled(scale, scanlines, tick, &mut buf);
    save_png_xrgb(path, FB_W * (scale as usize), FB_H * (scale as usize), &buf);
}

fn main() {
    let dir = Path::new("target/screenshots");
    fs::create_dir_all(dir).expect("mkdir screenshots");

    let mut fb = Framebuffer::new();

    // 1 & 2. Title screen (1x and 3x physical CRT scanlines)
    render_title(&mut fb, 1, 19, 12);
    save_fb("target/screenshots/title.png", &fb, 1, false, 12);
    save_fb("target/screenshots/title_3x_crt.png", &fb, 3, true, 12);

    // 3. Challenges Menu (including Tier-0 Explanation Levels)
    let challenges = all_challenges();
    let mut stars = [0u8; 35];
    stars[0] = 3;
    stars[1] = 2;
    stars[5] = 3;
    stars[6] = 1;
    render_challenges_menu(&mut fb, 1, 0, &stars, &challenges);
    save_fb("target/screenshots/challenges.png", &fb, 1, false, 0);

    // 4. 1v1 Skirmish Setup (Multi-faction teams + minimap)
    let maps = build_maps();
    render_skirmish_setup(
        &mut fb,
        FactionTeam::three(Faction::Humans, Faction::Elves, Faction::Gnomes),
        FactionTeam::two(Faction::Orcs, Faction::Dracones),
        &maps[2],
        2,
    );
    save_fb("target/screenshots/skirmish_setup.png", &fb, 1, false, 0);

    // 5. Midgame Battle (Humans+Elves vs Orcs+Dracones)
    let mut s1 = GameState::new_skirmish(
        777,
        maps[2].tiles,
        FactionTeam::two(Faction::Humans, Faction::Elves),
        FactionTeam::two(Faction::Orcs, Faction::Dracones),
        maps[2].p1_citadel,
        maps[2].p2_citadel,
    );
    let mut r1 = SimRng::new(111);
    let mut r2 = SimRng::new(222);
    for _ in 0..520 {
        if s1.tick % 20 == 0 {
            let a = choose_action(&s1, TeamId::P1, BotArchetype::Adaptive(AiDifficulty::Normal), &mut r1);
            if a != Action::NoOp {
                let _ = s1.execute_action(TeamId::P1, a);
            }
            let b = choose_action(&s1, TeamId::P2, BotArchetype::Adaptive(AiDifficulty::Normal), &mut r2);
            if b != Action::NoOp {
                let _ = s1.execute_action(TeamId::P2, b);
            }
        }
        s1.step();
    }
    let view1 = BattleView {
        state: &s1,
        active_faction_tab: 1,
        tool: ActiveTool::Build(BuildingType::ElvenPrismSpire),
        selected_tile: Some(maps[2].p1_citadel),
        hover_tile: Some((8, 6)),
        overlay: EnergyOverlayMode::Off,
        paused: false,
        fast_speed: false,
        challenge: None,
        lesson_step: None,
        lesson_reading: false,
        status_msg: None,
        ui_tick: s1.tick,
    };
    render_battle(&mut fb, &view1);
    save_fb("target/screenshots/battle_midgame.png", &fb, 1, false, s1.tick);

    // 6. Energy & Polarity Overlay (Leones+Dwarves vs Dracones+Orcs)
    let mut s2 = GameState::new_skirmish(
        999,
        maps[4].tiles,
        FactionTeam::two(Faction::Leones, Faction::Dwarves),
        FactionTeam::two(Faction::Dracones, Faction::Orcs),
        maps[4].p1_citadel,
        maps[4].p2_citadel,
    );
    s2.spawn_building_raw(TeamId::P1, BuildingType::LeoneSunMonolith, 10, 7, true);
    s2.spawn_building_raw(TeamId::P1, BuildingType::LeoneSolarPillar, 11, 6, true);
    s2.spawn_building_raw(TeamId::P1, BuildingType::DwarfRuneArchive, 6, 4, true);
    s2.spawn_building_raw(TeamId::P1, BuildingType::DwarfTunnelBore, 9, 8, true);
    s2.spawn_building_raw(TeamId::P2, BuildingType::DraconeVoidBrazier, 14, 7, true);
    s2.spawn_building_raw(TeamId::P2, BuildingType::OrcBloodAltar, 15, 6, true);
    s2.terrain[tile_idx(11, 7)] = Terrain::RaisedStone;
    s2.raised_hp[tile_idx(11, 7)] = RAISED_STONE_MAX_HP;
    s2.refresh_caches();
    s2.step_n(120);

    let view2 = BattleView {
        state: &s2,
        active_faction_tab: 0,
        tool: ActiveTool::Select,
        selected_tile: Some((10, 7)),
        hover_tile: Some((12, 7)),
        overlay: EnergyOverlayMode::Polarity,
        paused: false,
        fast_speed: false,
        challenge: None,
        lesson_step: None,
        lesson_reading: false,
        status_msg: None,
        ui_tick: s2.tick,
    };
    render_battle(&mut fb, &view2);
    save_fb("target/screenshots/battle_overlay.png", &fb, 1, false, s2.tick);

    // 7. Interactive Explanation Level (H0: Human Basics Step 1 - Paused Advisor Box)
    let h0 = &challenges[0];
    let s3 = towers::content::challenges::create_challenge_state(h0, 555);
    let view3 = BattleView {
        state: &s3,
        active_faction_tab: 0,
        tool: ActiveTool::Build(BuildingType::HumanRelay),
        selected_tile: None,
        hover_tile: Some((6, 7)),
        overlay: EnergyOverlayMode::Team,
        paused: true,
        fast_speed: false,
        challenge: Some(h0),
        lesson_step: Some((0, h0.lesson_steps.len(), h0.lesson_steps[0])),
        lesson_reading: true,
        status_msg: None,
        ui_tick: s3.tick,
    };
    render_battle(&mut fb, &view3);
    save_fb("target/screenshots/explanation_level.png", &fb, 1, false, s3.tick);

    // 8. Human Relay Signal Link & Minimized Reminder Box (H0 Step 3)
    let mut s4 = towers::content::challenges::create_challenge_state(h0, 556);
    s4.spawn_building_raw(TeamId::P1, BuildingType::HumanRelay, 6, 7, true);
    s4.spawn_building_raw(TeamId::P1, BuildingType::HumanMediaArray, 9, 6, true);
    s4.refresh_caches();
    s4.step_n(30);
    let view4 = BattleView {
        state: &s4,
        active_faction_tab: 0,
        tool: ActiveTool::Build(BuildingType::HumanAutocannon),
        selected_tile: None,
        hover_tile: Some((9, 7)),
        overlay: EnergyOverlayMode::Team,
        paused: false,
        fast_speed: false,
        challenge: Some(h0),
        lesson_step: Some((2, h0.lesson_steps.len(), h0.lesson_steps[2])),
        lesson_reading: false,
        status_msg: None,
        ui_tick: s4.tick,
    };
    render_battle(&mut fb, &view4);
    save_fb("target/screenshots/human_relay_link.png", &fb, 1, false, s4.tick);

    println!("Saved 8 screenshots to target/screenshots/");
}
