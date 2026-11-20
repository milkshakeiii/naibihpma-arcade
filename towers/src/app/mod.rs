pub mod input;

use std::fs;
use std::path::Path;

use crate::ai::search::{AiDifficulty, LiveAiController};
use crate::content::challenges::{
    all_challenges, create_challenge_state, step_challenge_waves, ChallengeDef, LessonStep,
};
use crate::content::maps::{build_maps, SkirmishMap};
use crate::gfx::render_game::{
    lesson_box_rect, lesson_continue_btn_rect, render_battle, render_challenges_menu,
    render_debrief_modal, render_help_modal, render_pause_modal, render_skirmish_setup,
    render_title, ActiveTool, BattleView, EnergyOverlayMode,
};
use crate::gfx::Framebuffer;
use crate::platform::audio::SfxKind;
use crate::sim::defs::{building_cost, building_def};
use crate::sim::rng::SimRng;
use crate::sim::types::*;
use crate::sim::GameState;
use input::{InputState, Key};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Screen {
    Title,
    Challenges,
    SkirmishSetup,
    Playing,
}

const PROGRESS_FILE: &str = "progress.dat";

pub struct App {
    pub screen: Screen,
    pub title_btn: usize,
    pub ch_faction: usize,
    pub ch_tier: usize,
    pub stars: [u8; 35],
    pub lesson_progress: usize,
    pub lesson_reading: bool,
    pub p1_team: FactionTeam,
    pub p2_team: FactionTeam,
    pub map_id: usize,
    pub diff_idx: usize,
    pub state: Option<GameState>,
    pub active_challenge: Option<ChallengeDef>,
    pub ai: Option<LiveAiController>,
    pub active_faction_tab: usize,
    pub tool: ActiveTool,
    pub selected_tile: Option<(u8, u8)>,
    pub hover_tile: Option<(u8, u8)>,
    pub overlay: EnergyOverlayMode,
    pub paused: bool,
    pub fast_speed: bool,
    pub help_open: bool,
    pub pause_menu: bool,
    pub status_msg: Option<(String, u16)>,
    pub outcome_recorded: bool,
    pub ui_tick: u32,
    pub scanlines: bool,
    pub muted: bool,
    pub should_quit: bool,
    pub toggle_fullscreen_req: bool,
    pub pending_sfx: Vec<SfxKind>,
    pub maps: [SkirmishMap; 6],
    pub challenges: Vec<ChallengeDef>,
    pub rng: SimRng,
}

impl Default for App {
    fn default() -> Self {
        Self::new()
    }
}

impl App {
    pub fn new() -> Self {
        let mut app = Self {
            screen: Screen::Title,
            title_btn: 0,
            ch_faction: 0,
            ch_tier: 0,
            stars: [0; 35],
            lesson_progress: 0,
            lesson_reading: false,
            p1_team: FactionTeam::two(Faction::Humans, Faction::Gnomes),
            p2_team: FactionTeam::two(Faction::Orcs, Faction::Dracones),
            map_id: 0,
            diff_idx: 2, // Hard default
            state: None,
            active_challenge: None,
            ai: None,
            active_faction_tab: 0,
            tool: ActiveTool::Select,
            selected_tile: None,
            hover_tile: None,
            overlay: EnergyOverlayMode::Off,
            paused: false,
            fast_speed: false,
            help_open: false,
            pause_menu: false,
            status_msg: None,
            outcome_recorded: false,
            ui_tick: 0,
            scanlines: true,
            muted: false,
            should_quit: false,
            toggle_fullscreen_req: false,
            pending_sfx: Vec::new(),
            maps: build_maps(),
            challenges: all_challenges(),
            rng: SimRng::new(0x5EED_CAFE),
        };
        app.load_progress();
        app
    }

    fn load_progress(&mut self) {
        if let Ok(bytes) = fs::read(Path::new(PROGRESS_FILE)) {
            if bytes.len() >= 28 {
                for i in 0..bytes.len().min(35) {
                    self.stars[i] = bytes[i].min(3);
                }
            }
        }
    }

    fn save_progress(&self) {
        let _ = fs::write(Path::new(PROGRESS_FILE), &self.stars);
    }

    fn push_sfx(&mut self, s: SfxKind) {
        self.pending_sfx.push(s);
    }

    fn set_status(&mut self, msg: impl Into<String>) {
        self.status_msg = Some((msg.into(), 36));
    }

    pub fn start_challenge(&mut self, ch_idx: usize) {
        let ch = self.challenges[ch_idx % self.challenges.len()];
        let seed = 1000 + (ch.id as u64) * 97 + self.rng.next_u32() as u64;
        let s = create_challenge_state(&ch, seed);
        self.ai = if ch.uses_ai {
            Some(LiveAiController::new(TeamId::P2, AiDifficulty::Normal, seed ^ 0xA1))
        } else {
            None
        };
        self.state = Some(s);
        self.active_challenge = Some(ch);
        self.lesson_progress = 0;
        self.lesson_reading = !ch.lesson_steps.is_empty();
        self.screen = Screen::Playing;
        self.active_faction_tab = 0;
        self.tool = ActiveTool::Select;
        self.selected_tile = ch.p1_citadel;
        self.overlay = if ch.tier == 0 {
            if ch.faction == Faction::Leones {
                EnergyOverlayMode::Polarity
            } else {
                EnergyOverlayMode::Team
            }
        } else {
            EnergyOverlayMode::Off
        };
        self.paused = false;
        self.fast_speed = false;
        self.help_open = false;
        self.pause_menu = false;
        self.outcome_recorded = false;
        self.status_msg = None;
    }

    pub fn start_skirmish(&mut self) {
        let m = &self.maps[self.map_id % self.maps.len()];
        let seed = 4242 + self.rng.next_u32() as u64;
        let s = GameState::new_skirmish(
            seed,
            m.tiles,
            self.p1_team,
            self.p2_team,
            m.p1_citadel,
            m.p2_citadel,
        );
        let diff = match self.diff_idx {
            0 => AiDifficulty::Easy,
            1 => AiDifficulty::Normal,
            _ => AiDifficulty::Hard,
        };
        self.state = Some(s);
        self.active_challenge = None;
        self.lesson_progress = 0;
        self.lesson_reading = false;
        self.ai = Some(LiveAiController::new(TeamId::P2, diff, seed ^ 0xBEEF));
        self.screen = Screen::Playing;
        self.active_faction_tab = 0;
        self.tool = ActiveTool::Select;
        self.selected_tile = Some(m.p1_citadel);
        self.overlay = EnergyOverlayMode::Off;
        self.paused = false;
        self.fast_speed = false;
        self.help_open = false;
        self.pause_menu = false;
        self.outcome_recorded = false;
        self.status_msg = None;
    }

    pub fn tick(&mut self, input: &mut InputState) {
        self.ui_tick = self.ui_tick.wrapping_add(1);
        if let Some((_, ttl)) = &mut self.status_msg {
            if *ttl <= 1 {
                self.status_msg = None;
            } else {
                *ttl -= 1;
            }
        }

        // Global hardware toggles
        if input.just_pressed(Key::F2) {
            self.scanlines = !self.scanlines;
            self.push_sfx(SfxKind::Click);
        }
        if input.just_pressed(Key::F3) {
            self.muted = !self.muted;
        }
        if input.just_pressed(Key::F11) {
            self.toggle_fullscreen_req = true;
        }

        match self.screen {
            Screen::Title => self.tick_title(input),
            Screen::Challenges => self.tick_challenges(input),
            Screen::SkirmishSetup => self.tick_skirmish_setup(input),
            Screen::Playing => self.tick_playing(input),
        }

        input.end_tick();
    }

    fn tick_title(&mut self, input: &InputState) {
        if input.just_pressed(Key::Up) {
            self.title_btn = (self.title_btn + 3) % 4;
            self.push_sfx(SfxKind::Click);
        }
        if input.just_pressed(Key::Down) {
            self.title_btn = (self.title_btn + 1) % 4;
            self.push_sfx(SfxKind::Click);
        }
        if input.just_pressed(Key::Num1) {
            self.title_btn = 0;
            self.screen = Screen::Challenges;
            self.push_sfx(SfxKind::Click);
            return;
        }
        if input.just_pressed(Key::Num2) {
            self.title_btn = 1;
            self.screen = Screen::SkirmishSetup;
            self.push_sfx(SfxKind::Click);
            return;
        }
        if input.just_pressed(Key::Num3) {
            self.title_btn = 2;
            self.help_open = !self.help_open;
            self.push_sfx(SfxKind::Click);
            return;
        }
        if input.just_pressed(Key::Num4) {
            self.should_quit = true;
            return;
        }
        if self.help_open {
            if input.just_pressed(Key::H) || input.just_pressed(Key::Esc) || input.left_pressed {
                self.help_open = false;
                self.push_sfx(SfxKind::Click);
            }
            return;
        }
        if input.just_pressed(Key::Enter) || input.just_pressed(Key::Space) {
            self.activate_title_btn(self.title_btn);
            return;
        }
        if input.mouse_x >= 228 && input.mouse_x <= 380 {
            for i in 0..4 {
                let by = 84 + (i as i32) * 32;
                if input.mouse_y >= by && input.mouse_y < by + 24 {
                    self.title_btn = i;
                    if input.left_pressed {
                        self.activate_title_btn(i);
                        return;
                    }
                }
            }
        }
    }

    fn activate_title_btn(&mut self, idx: usize) {
        self.push_sfx(SfxKind::Click);
        match idx {
            0 => self.screen = Screen::Challenges,
            1 => self.screen = Screen::SkirmishSetup,
            2 => self.help_open = true,
            _ => self.should_quit = true,
        }
    }

    fn tick_challenges(&mut self, input: &InputState) {
        if input.just_pressed(Key::Esc) {
            self.screen = Screen::Title;
            self.push_sfx(SfxKind::Click);
            return;
        }
        if input.just_pressed(Key::Up) {
            if self.ch_tier > 0 {
                self.ch_tier -= 1;
            } else {
                self.ch_faction = (self.ch_faction + 6) % 7;
                self.ch_tier = 4;
            }
            self.push_sfx(SfxKind::Click);
        }
        if input.just_pressed(Key::Down) {
            if self.ch_tier < 4 {
                self.ch_tier += 1;
            } else {
                self.ch_faction = (self.ch_faction + 1) % 7;
                self.ch_tier = 0;
            }
            self.push_sfx(SfxKind::Click);
        }
        if input.just_pressed(Key::Left) {
            self.ch_faction = (self.ch_faction + 6) % 7;
            self.push_sfx(SfxKind::Click);
        }
        if input.just_pressed(Key::Right) {
            self.ch_faction = (self.ch_faction + 1) % 7;
            self.push_sfx(SfxKind::Click);
        }
        if input.just_pressed(Key::Enter) || input.just_pressed(Key::Space) {
            self.push_sfx(SfxKind::Click);
            self.start_challenge(self.ch_faction * 5 + self.ch_tier);
            return;
        }
        if input.left_pressed {
            let mx = input.mouse_x;
            let my = input.mouse_y;
            if mx >= 8 && mx <= 118 {
                for i in 0..7 {
                    let fy = 30 + (i as i32) * 32;
                    if my >= fy && my < fy + 29 {
                        self.ch_faction = i;
                        self.push_sfx(SfxKind::Click);
                        return;
                    }
                }
            }
            if mx >= 131 && mx <= 391 {
                for t in 0..5 {
                    let cy = 43 + (t as i32) * 42;
                    if my >= cy && my < cy + 40 {
                        if self.ch_tier == t {
                            self.push_sfx(SfxKind::Click);
                            self.start_challenge(self.ch_faction * 5 + t);
                        } else {
                            self.ch_tier = t;
                            self.push_sfx(SfxKind::Click);
                        }
                        return;
                    }
                }
            }
            if mx >= 8 && mx <= 94 && my >= 276 && my <= 295 {
                self.screen = Screen::Title;
                self.push_sfx(SfxKind::Click);
            } else if mx >= 266 && mx <= 392 && my >= 276 && my <= 295 {
                self.push_sfx(SfxKind::Click);
                self.start_challenge(self.ch_faction * 5 + self.ch_tier);
            }
        }
    }

    fn toggle_team_faction(team: &mut FactionTeam, f: Faction) {
        let slice = team.as_slice();
        if slice.contains(&f) {
            if slice.len() > 1 {
                let mut next = Vec::new();
                for &x in slice {
                    if x != f {
                        next.push(x);
                    }
                }
                *team = FactionTeam::from_slice(&next);
            }
        } else if slice.len() < 3 {
            let mut next = slice.to_vec();
            next.push(f);
            *team = FactionTeam::from_slice(&next);
        } else {
            // Rotate oldest out so clicking a 4th faction swaps cleanly
            let next = [slice[1], slice[2], f];
            *team = FactionTeam::from_slice(&next);
        }
    }

    fn randomize_skirmish(&mut self) {
        let c1 = self.rng.range_usize(1, 4);
        let mut l1 = Vec::new();
        for _ in 0..c1 {
            l1.push(Faction::ALL[self.rng.range_usize(0, 7)]);
        }
        self.p1_team = FactionTeam::from_slice(&l1);
        let c2 = self.rng.range_usize(1, 4);
        let mut l2 = Vec::new();
        for _ in 0..c2 {
            l2.push(Faction::ALL[self.rng.range_usize(0, 7)]);
        }
        self.p2_team = FactionTeam::from_slice(&l2);
        self.map_id = self.rng.range_usize(0, 6);
        self.push_sfx(SfxKind::Click);
    }

    fn tick_skirmish_setup(&mut self, input: &InputState) {
        if input.just_pressed(Key::Esc) {
            self.screen = Screen::Title;
            self.push_sfx(SfxKind::Click);
            return;
        }
        if input.just_pressed(Key::Left) {
            self.map_id = (self.map_id + 5) % 6;
            self.push_sfx(SfxKind::Click);
        }
        if input.just_pressed(Key::Right) {
            self.map_id = (self.map_id + 1) % 6;
            self.push_sfx(SfxKind::Click);
        }
        if input.just_pressed(Key::D) {
            self.diff_idx = (self.diff_idx + 1) % 3;
            self.push_sfx(SfxKind::Click);
        }
        if input.just_pressed(Key::Enter) {
            self.push_sfx(SfxKind::Click);
            self.start_skirmish();
            return;
        }
        if input.just_pressed(Key::R) || input.just_pressed(Key::Tab) {
            self.randomize_skirmish();
        }

        if input.left_pressed {
            let mx = input.mouse_x;
            let my = input.mouse_y;
            if mx >= 10 && mx <= 190 {
                for (i, &f) in Faction::ALL.iter().enumerate() {
                    let fy = 41 + (i as i32) * 18;
                    if my >= fy && my < fy + 16 {
                        Self::toggle_team_faction(&mut self.p1_team, f);
                        self.push_sfx(SfxKind::Click);
                        return;
                    }
                }
            }
            if mx >= 210 && mx <= 390 {
                for (i, &f) in Faction::ALL.iter().enumerate() {
                    let fy = 41 + (i as i32) * 18;
                    if my >= fy && my < fy + 16 {
                        Self::toggle_team_faction(&mut self.p2_team, f);
                        self.push_sfx(SfxKind::Click);
                        return;
                    }
                }
            }
            if mx >= 4 && mx <= 270 && my >= 180 && my <= 256 {
                if mx < 137 {
                    self.map_id = (self.map_id + 5) % 6;
                } else {
                    self.map_id = (self.map_id + 1) % 6;
                }
                self.push_sfx(SfxKind::Click);
                return;
            }
            if mx >= 280 && mx <= 390 {
                for i in 0..3 {
                    let dy = 200 + (i as i32) * 17;
                    if my >= dy && my < dy + 15 {
                        self.diff_idx = i;
                        self.push_sfx(SfxKind::Click);
                        return;
                    }
                }
            }
            if mx >= 8 && mx <= 94 && my >= 268 && my <= 292 {
                self.screen = Screen::Title;
                self.push_sfx(SfxKind::Click);
            } else if mx >= 106 && mx <= 230 && my >= 268 && my <= 292 {
                self.randomize_skirmish();
            } else if mx >= 256 && mx <= 392 && my >= 268 && my <= 292 {
                self.push_sfx(SfxKind::Click);
                self.start_skirmish();
            }
        }
    }

    fn tick_playing(&mut self, input: &InputState) {
        let Some(ref mut state) = self.state else {
            self.screen = Screen::Title;
            return;
        };

        // Check hover tile in playfield
        if input.mouse_y >= PLAYFIELD_Y && input.mouse_y < PLAYFIELD_Y + PLAYFIELD_H && input.mouse_x >= 0 && input.mouse_x < 400 {
            let tx = (input.mouse_x / TILE_PX).clamp(0, MAP_W as i32 - 1) as u8;
            let ty = ((input.mouse_y - PLAYFIELD_Y) / TILE_PX).clamp(0, MAP_H as i32 - 1) as u8;
            self.hover_tile = Some((tx, ty));
        } else {
            self.hover_tile = None;
        }

        // Handle Debrief if match ended
        if state.outcome != MatchOutcome::Ongoing {
            if !self.outcome_recorded {
                self.outcome_recorded = true;
                let won = state.outcome == MatchOutcome::Victory(TeamId::P1);
                if won {
                    self.pending_sfx.push(SfxKind::Victory);
                    if let Some(ch) = self.active_challenge {
                        let hp = state.citadel_hp(TeamId::P1);
                        let st = if state.tick <= ch.par_ticks && hp >= (CITADEL_MAX_HP * 85) / 100 {
                            3
                        } else if hp >= CITADEL_MAX_HP / 2 {
                            2
                        } else {
                            1
                        };
                        if st > self.stars[ch.id] {
                            self.stars[ch.id] = st;
                            self.save_progress();
                        }
                    }
                } else {
                    self.pending_sfx.push(SfxKind::Defeat);
                }
            }
            if input.just_pressed(Key::R) {
                if let Some(ch) = self.active_challenge {
                    self.start_challenge(ch.id);
                } else {
                    self.start_skirmish();
                }
                self.pending_sfx.push(SfxKind::Click);
                return;
            }
            if input.just_pressed(Key::Enter) || input.just_pressed(Key::Esc) {
                self.screen = if self.active_challenge.is_some() {
                    Screen::Challenges
                } else {
                    Screen::Title
                };
                self.state = None;
                self.pending_sfx.push(SfxKind::Click);
                return;
            }
            if input.left_pressed {
                let mx = input.mouse_x;
                let my = input.mouse_y;
                if my >= 192 && my <= 216 {
                    if mx >= 88 && mx <= 190 {
                        if let Some(ch) = self.active_challenge {
                            self.start_challenge(ch.id);
                        } else {
                            self.start_skirmish();
                        }
                        return;
                    } else if mx >= 210 && mx <= 312 {
                        self.screen = if self.active_challenge.is_some() {
                            Screen::Challenges
                        } else {
                            Screen::Title
                        };
                        self.state = None;
                        return;
                    }
                }
            }
            return;
        }

        // Modals (Help / Pause)
        if self.help_open {
            if input.just_pressed(Key::H) || input.just_pressed(Key::Esc)
                || (input.left_pressed && input.mouse_x >= 136 && input.mouse_x <= 264 && input.mouse_y >= 250 && input.mouse_y <= 272)
            {
                self.help_open = false;
                self.pending_sfx.push(SfxKind::Click);
            }
            return;
        }
        if self.pause_menu {
            if input.just_pressed(Key::Esc) {
                self.pause_menu = false;
                self.pending_sfx.push(SfxKind::Click);
                return;
            }
            if input.just_pressed(Key::R) {
                if let Some(ch) = self.active_challenge {
                    self.start_challenge(ch.id);
                } else {
                    self.start_skirmish();
                }
                self.pending_sfx.push(SfxKind::Click);
                return;
            }
            if input.just_pressed(Key::H) {
                self.pause_menu = false;
                self.help_open = true;
                self.pending_sfx.push(SfxKind::Click);
                return;
            }
            if input.just_pressed(Key::Q) {
                self.screen = if self.active_challenge.is_some() {
                    Screen::Challenges
                } else {
                    Screen::Title
                };
                self.state = None;
                self.pause_menu = false;
                return;
            }
            if input.left_pressed && input.mouse_x >= 126 && input.mouse_x <= 274 {
                for i in 0..4 {
                    let by = 114 + (i as i32) * 24;
                    if input.mouse_y >= by && input.mouse_y < by + 19 {
                        match i {
                            0 => self.pause_menu = false,
                            1 => {
                                if let Some(ch) = self.active_challenge {
                                    self.start_challenge(ch.id);
                                } else {
                                    self.start_skirmish();
                                }
                            }
                            2 => {
                                self.pause_menu = false;
                                self.help_open = true;
                            }
                            _ => {
                                self.screen = if self.active_challenge.is_some() {
                                    Screen::Challenges
                                } else {
                                    Screen::Title
                                };
                                self.state = None;
                                self.pause_menu = false;
                            }
                        }
                        self.pending_sfx.push(SfxKind::Click);
                        return;
                    }
                }
            }
            return;
        }

        // Tier-0 In-World Advisor Box controls (auto-paused while reading)
        let cur_lesson: Option<LessonStep> = self
            .active_challenge
            .and_then(|ch| ch.lesson_steps.get(self.lesson_progress).copied());
        if self.lesson_reading {
            if let Some(step) = cur_lesson {
                if input.just_pressed(Key::Space) || input.just_pressed(Key::Enter) {
                    self.lesson_reading = false;
                    self.paused = false;
                    self.pending_sfx.push(SfxKind::Click);
                    return;
                }
                if [Key::Num1, Key::Num2, Key::Num3, Key::Num4, Key::Num5]
                    .iter()
                    .any(|&k| input.just_pressed(k))
                {
                    self.lesson_reading = false;
                    self.paused = false;
                } else if input.left_pressed {
                    let mx = input.mouse_x;
                    let my = input.mouse_y;
                    let (cbx, cby, cbw, cbh) = lesson_continue_btn_rect(&step);
                    let (bx, by, bw, bh) = lesson_box_rect(&step, true);
                    if mx >= cbx && mx < cbx + cbw && my >= cby && my < cby + cbh {
                        self.lesson_reading = false;
                        self.paused = false;
                        self.pending_sfx.push(SfxKind::Click);
                        return;
                    } else if my >= 254 && my <= 298 && mx >= 2 && mx < 237 {
                        self.lesson_reading = false;
                        self.paused = false;
                    } else if mx >= bx && mx < bx + bw && my >= by && my < by + bh {
                        return;
                    }
                }
            } else {
                self.lesson_reading = false;
            }
        } else if let Some(step) = cur_lesson {
            if input.left_pressed {
                let (bx, by, bw, bh) = lesson_box_rect(&step, false);
                let mx = input.mouse_x;
                let my = input.mouse_y;
                if mx >= bx && mx < bx + bw && my >= by && my < by + bh {
                    self.lesson_reading = true;
                    self.pending_sfx.push(SfxKind::Click);
                    return;
                }
            }
        }

        // Player Commands
        if input.just_pressed(Key::H) {
            self.help_open = true;
            self.pending_sfx.push(SfxKind::Click);
            return;
        }
        if input.just_pressed(Key::V) {
            self.overlay = self.overlay.next();
            self.pending_sfx.push(SfxKind::Click);
        }
        if input.just_pressed(Key::Space) {
            self.paused = !self.paused;
            self.pending_sfx.push(SfxKind::Click);
        }
        if input.just_pressed(Key::F) {
            self.fast_speed = !self.fast_speed;
            self.pending_sfx.push(SfxKind::Click);
        }
        if input.right_pressed {
            self.tool = ActiveTool::Select;
            self.pending_sfx.push(SfxKind::Click);
        }
        if input.just_pressed(Key::Esc) {
            if self.tool != ActiveTool::Select {
                self.tool = ActiveTool::Select;
            } else {
                self.pause_menu = true;
            }
            self.pending_sfx.push(SfxKind::Click);
            return;
        }

        let num_factions = state.teams[0].config.as_slice().len();
        if num_factions > 1 {
            if input.just_pressed(Key::Q) {
                self.active_faction_tab = (self.active_faction_tab + num_factions - 1) % num_factions;
                self.tool = ActiveTool::Select;
                self.pending_sfx.push(SfxKind::Click);
            }
            if input.just_pressed(Key::E) || input.just_pressed(Key::Tab) {
                self.active_faction_tab = (self.active_faction_tab + 1) % num_factions;
                self.tool = ActiveTool::Select;
                self.pending_sfx.push(SfxKind::Click);
            }
        }

        let active_fac = state.teams[0].config.as_slice()[self.active_faction_tab % num_factions];
        for (i, key) in [Key::Num1, Key::Num2, Key::Num3, Key::Num4, Key::Num5].iter().enumerate() {
            if input.just_pressed(*key) {
                let kind = BuildingType::from_faction_slot(active_fac, BuildingSlot::ALL[i]);
                if self.tool == ActiveTool::Build(kind) {
                    self.tool = ActiveTool::Select;
                } else {
                    self.tool = ActiveTool::Build(kind);
                }
                self.pending_sfx.push(SfxKind::Click);
            }
        }

        // Selected building hotkeys: [U], [X], [M], [T], [P], [D]
        let mut msg_out: Option<&'static str> = None;
        let mut sfx_out: Option<SfxKind> = None;

        if let Some((sx, sy)) = self.selected_tile {
            if input.just_pressed(Key::U) {
                match state.execute_action(TeamId::P1, Action::Upgrade { x: sx, y: sy }) {
                    Ok(()) => sfx_out = Some(SfxKind::Upgrade),
                    Err(e) => msg_out = Some(e),
                }
            } else if input.just_pressed(Key::X) {
                match state.execute_action(TeamId::P1, Action::Salvage { x: sx, y: sy }) {
                    Ok(()) => {
                        sfx_out = Some(SfxKind::Salvage);
                        self.selected_tile = None;
                    }
                    Err(e) => msg_out = Some(e),
                }
            } else if input.just_pressed(Key::M) {
                let g = state.building_grid[tile_idx(sx as usize, sy as usize)];
                if g != GRID_EMPTY && state.buildings[g as usize].kind.is_walker() && state.buildings[g as usize].owner == TeamId::P1 {
                    self.tool = ActiveTool::RelocateFrom(sx, sy);
                    sfx_out = Some(SfxKind::Click);
                    msg_out = Some("CLICK DESTINATION TILE TO RELOCATE WALKER");
                }
            } else if input.just_pressed(Key::T) {
                let g = state.building_grid[tile_idx(sx as usize, sy as usize)];
                if g != GRID_EMPTY && state.buildings[g as usize].kind == BuildingType::LeoneLithoShaper && state.buildings[g as usize].owner == TeamId::P1 {
                    self.tool = ActiveTool::ShapeFrom(sx, sy);
                    sfx_out = Some(SfxKind::Click);
                    msg_out = Some("CLICK TILE WITHIN 3 TILES TO SHAPE STONE");
                }
            } else if input.just_pressed(Key::P) {
                // Auto-find adjacent Leone building to push away from Tectonic Ram
                let g = state.building_grid[tile_idx(sx as usize, sy as usize)];
                if g != GRID_EMPTY && state.buildings[g as usize].kind == BuildingType::LeoneTectonicRam && state.buildings[g as usize].owner == TeamId::P1 {
                    let mut best: Option<(u8, u8)> = None;
                    for ob in &state.buildings {
                        if ob.hp > 0 && ob.owner == TeamId::P1 && ob.kind != BuildingType::Citadel && ob.kind.faction() == Some(Faction::Leones) && (ob.x != sx || ob.y != sy) && chebyshev(sx, sy, ob.x, ob.y) <= 1 {
                            let act = Action::TectonicPush { ram_x: sx, ram_y: sy, target_x: ob.x, target_y: ob.y };
                            if state.can_execute(TeamId::P1, act).is_ok() {
                                best = Some((ob.x, ob.y));
                                break;
                            }
                        }
                    }
                    if let Some((tx, ty)) = best {
                        let _ = state.execute_action(TeamId::P1, Action::TectonicPush { ram_x: sx, ram_y: sy, target_x: tx, target_y: ty });
                        sfx_out = Some(SfxKind::Special);
                    } else {
                        msg_out = Some("NO ADJACENT PUSHABLE LEONE TOWER");
                    }
                }
            } else if input.just_pressed(Key::D) {
                // Auto-devour weakest adjacent Orc building
                let g = state.building_grid[tile_idx(sx as usize, sy as usize)];
                if g != GRID_EMPTY && state.buildings[g as usize].kind == BuildingType::OrcDoomCauldron && state.buildings[g as usize].owner == TeamId::P1 {
                    let mut best: Option<(u8, u8)> = None;
                    for ob in &state.buildings {
                        if ob.hp > 0 && ob.owner == TeamId::P1 && ob.kind != BuildingType::Citadel && ob.kind.faction() == Some(Faction::Orcs) && (ob.x != sx || ob.y != sy) && chebyshev(sx, sy, ob.x, ob.y) <= 1 {
                            let act = Action::Devour { cauldron_x: sx, cauldron_y: sy, victim_x: ob.x, victim_y: ob.y };
                            if state.can_execute(TeamId::P1, act).is_ok() {
                                best = Some((ob.x, ob.y));
                                break;
                            }
                        }
                    }
                    if let Some((vx, vy)) = best {
                        let _ = state.execute_action(TeamId::P1, Action::Devour { cauldron_x: sx, cauldron_y: sy, victim_x: vx, victim_y: vy });
                        sfx_out = Some(SfxKind::Special);
                    } else {
                        msg_out = Some("NO ADJACENT ORC BUILDING TO DEVOUR");
                    }
                }
            }
        }

        // Mouse Clicks during Play
        if input.left_pressed {
            let mx = input.mouse_x;
            let my = input.mouse_y;

            if my < PLAYFIELD_Y {
                // Top Bar controls
                if mx >= 300 && mx <= 336 {
                    self.overlay = self.overlay.next();
                    sfx_out = Some(SfxKind::Click);
                } else if mx >= 337 && mx <= 383 {
                    self.paused = !self.paused;
                    sfx_out = Some(SfxKind::Click);
                } else if mx >= 384 {
                    self.help_open = true;
                    sfx_out = Some(SfxKind::Click);
                }
            } else if my < PLAYFIELD_Y + PLAYFIELD_H {
                // Playfield Click
                if let Some((tx, ty)) = self.hover_tile {
                    match self.tool {
                        ActiveTool::Build(kind) => {
                            let act = Action::Build { kind, x: tx, y: ty };
                            match state.execute_action(TeamId::P1, act) {
                                Ok(()) => {
                                    sfx_out = Some(SfxKind::Build);
                                    self.selected_tile = Some((tx, ty));
                                    if state.teams[0].resources < building_cost(kind, state.teams[0].config) {
                                        self.tool = ActiveTool::Select;
                                    }
                                }
                                Err(e) => msg_out = Some(e),
                            }
                        }
                        ActiveTool::RelocateFrom(wx, wy) => {
                            let act = Action::Relocate { x: wx, y: wy, tx, ty };
                            match state.execute_action(TeamId::P1, act) {
                                Ok(()) => {
                                    sfx_out = Some(SfxKind::Special);
                                    self.tool = ActiveTool::Select;
                                    self.selected_tile = Some((tx, ty));
                                }
                                Err(e) => msg_out = Some(e),
                            }
                        }
                        ActiveTool::ShapeFrom(sx, sy) => {
                            let act = Action::ShapeTerrain { shaper_x: sx, shaper_y: sy, tx, ty };
                            match state.execute_action(TeamId::P1, act) {
                                Ok(()) => {
                                    sfx_out = Some(SfxKind::Special);
                                    self.tool = ActiveTool::Select;
                                }
                                Err(e) => msg_out = Some(e),
                            }
                        }
                        ActiveTool::Select => {
                            self.selected_tile = Some((tx, ty));
                            sfx_out = Some(SfxKind::Click);
                        }
                    }
                }
            } else if my >= 240 && my <= 253 && mx >= 2 && mx <= 236 && num_factions > 1 {
                // Faction tabs
                let tab_w = 234 / (num_factions as i32);
                let idx = ((mx - 2) / tab_w).clamp(0, num_factions as i32 - 1) as usize;
                self.active_faction_tab = idx;
                self.tool = ActiveTool::Select;
                sfx_out = Some(SfxKind::Click);
            } else if my >= 254 && my <= 298 && mx >= 2 && mx < 237 {
                // Building card clicks
                let slot_i = ((mx - 2) / 47).clamp(0, 4) as usize;
                let kind = BuildingType::from_faction_slot(active_fac, BuildingSlot::ALL[slot_i]);
                let def = building_def(kind);
                if def.require_eco && !state.has_eco_building(TeamId::P1, active_fac) {
                    msg_out = Some("BUILD FACTION ECO STRUCTURE FIRST");
                } else {
                    self.tool = if self.tool == ActiveTool::Build(kind) {
                        ActiveTool::Select
                    } else {
                        ActiveTool::Build(kind)
                    };
                    sfx_out = Some(SfxKind::Click);
                }
            } else if my >= 263 && my <= 294 && mx >= 244 && mx <= 396 {
                // Inspector Action button clicks
                if let Some((sx, sy)) = self.selected_tile {
                    if my < 275 {
                        if mx < 318 {
                            match state.execute_action(TeamId::P1, Action::Upgrade { x: sx, y: sy }) {
                                Ok(()) => sfx_out = Some(SfxKind::Upgrade),
                                Err(e) => msg_out = Some(e),
                            }
                        } else {
                            match state.execute_action(TeamId::P1, Action::Salvage { x: sx, y: sy }) {
                                Ok(()) => {
                                    sfx_out = Some(SfxKind::Salvage);
                                    self.selected_tile = None;
                                }
                                Err(e) => msg_out = Some(e),
                            }
                        }
                    } else {
                        // Trigger special button on selected building
                        let g = state.building_grid[tile_idx(sx as usize, sy as usize)];
                        if g != GRID_EMPTY && state.buildings[g as usize].owner == TeamId::P1 {
                            match state.buildings[g as usize].kind {
                                BuildingType::GnomeWalkerTurret | BuildingType::GnomeTeslaContraption => {
                                    self.tool = ActiveTool::RelocateFrom(sx, sy);
                                    msg_out = Some("CLICK DESTINATION TILE");
                                }
                                BuildingType::LeoneLithoShaper => {
                                    self.tool = ActiveTool::ShapeFrom(sx, sy);
                                    msg_out = Some("CLICK TILE WITHIN 3 TILES TO SHAPE");
                                }
                                _ => {}
                            }
                        }
                    }
                }
            }
        }

        if let Some(m) = msg_out {
            self.set_status(m);
        }
        if let Some(s) = sfx_out {
            self.pending_sfx.push(s);
        }

        // Advance simulation if unpaused and not reading advisor box
        if !self.paused && !self.lesson_reading {
            let steps = if self.fast_speed { 2 } else { 1 };
            for _ in 0..steps {
                if let Some(ref mut s) = self.state {
                    if s.outcome != MatchOutcome::Ongoing {
                        break;
                    }
                    if let Some(ch) = self.active_challenge {
                        step_challenge_waves(&ch, s);
                    }
                    if let Some(ref mut ai) = self.ai {
                        ai.update(s);
                    }
                    let prev_beams = s.fx_beams.len();
                    let prev_bursts = s.fx_bursts.len();
                    s.step();
                    if s.fx_bursts.len() > prev_bursts && (s.tick % 4 == 0) {
                        self.pending_sfx.push(SfxKind::Explosion);
                    } else if s.fx_beams.len() > prev_beams && (s.tick % 6 == 0) {
                        self.pending_sfx.push(SfxKind::Laser);
                    }
                }
            }
        }

        // Advance active Tier-0 lesson step as soon as trigger condition is met
        if let (Some(ch), Some(ref s)) = (self.active_challenge, &self.state) {
            if !ch.lesson_steps.is_empty() && s.outcome == MatchOutcome::Ongoing {
                let mut advanced = false;
                while self.lesson_progress < ch.lesson_steps.len()
                    && ch.lesson_steps[self.lesson_progress].trigger.is_met(s)
                {
                    self.lesson_progress += 1;
                    advanced = true;
                }
                if advanced {
                    self.pending_sfx.push(SfxKind::Upgrade);
                    if self.lesson_progress < ch.lesson_steps.len() {
                        self.lesson_reading = true;
                        if let ActiveTool::Build(_) = self.tool {
                            self.tool = ActiveTool::Select;
                        }
                    }
                }
            }
        }
    }

    pub fn render(&self, fb: &mut Framebuffer) {
        match self.screen {
            Screen::Title => {
                let total_stars: usize = self.stars.iter().map(|&s| s as usize).sum();
                render_title(fb, self.title_btn, total_stars, self.ui_tick);
                if self.help_open {
                    render_help_modal(fb);
                }
            }
            Screen::Challenges => {
                render_challenges_menu(fb, self.ch_faction, self.ch_tier, &self.stars, &self.challenges);
            }
            Screen::SkirmishSetup => {
                render_skirmish_setup(
                    fb,
                    self.p1_team,
                    self.p2_team,
                    &self.maps[self.map_id],
                    self.diff_idx,
                );
            }
            Screen::Playing => {
                if let Some(ref s) = self.state {
                    let lesson_step = self.active_challenge.and_then(|ch| {
                        ch.lesson_steps
                            .get(self.lesson_progress)
                            .copied()
                            .map(|st| (self.lesson_progress, ch.lesson_steps.len(), st))
                    });
                    let view = BattleView {
                        state: s,
                        active_faction_tab: self.active_faction_tab,
                        tool: self.tool,
                        selected_tile: self.selected_tile,
                        hover_tile: self.hover_tile,
                        overlay: self.overlay,
                        paused: self.paused || self.lesson_reading,
                        fast_speed: self.fast_speed,
                        challenge: self.active_challenge.as_ref(),
                        lesson_step,
                        lesson_reading: self.lesson_reading,
                        status_msg: self.status_msg.as_ref().map(|(m, t)| (m.as_str(), *t)),
                        ui_tick: self.ui_tick,
                    };
                    render_battle(fb, &view);
                    if self.help_open {
                        render_help_modal(fb);
                    } else if self.pause_menu {
                        render_pause_modal(fb);
                    } else if s.outcome != MatchOutcome::Ongoing {
                        let won = s.outcome == MatchOutcome::Victory(TeamId::P1);
                        let draw = s.outcome == MatchOutcome::Draw;
                        let st = if won {
                            self.active_challenge.map_or(0, |ch| self.stars[ch.id])
                        } else {
                            0
                        };
                        render_debrief_modal(fb, won, draw, st, s.tick, s);
                    }
                }
            }
        }
    }
}
