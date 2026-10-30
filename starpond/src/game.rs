use crate::level::{DewMode, Level, TS};
use crate::palette::{self, Mood, DEW, MIST, MOODS, WATER};
use crate::sprites::drop_dims;

pub const VIEW_W: i32 = 400;
pub const VIEW_H: i32 = 240;
pub const PW: i32 = 14; // player hitbox
pub const PH: i32 = 10;
pub const MAX_SIZE: u8 = 5;
const ECHO_GROWTH: u8 = 2;
/// Ticks of direct sun (15 per second) before an exposed drop shrinks by one.
pub const EVAP_TICKS: u16 = 120;
const BEST_FILE: &str = "starpond_best.txt";

#[derive(Default, Clone, Copy, Debug)]
pub struct Input {
    pub left: bool,
    pub right: bool,
    pub up: bool,
    pub down: bool,
    pub jump: bool,
    pub jump_pressed: bool,
    pub down_pressed: bool,
    pub restart_pressed: bool,
    pub skip_pressed: bool,
    pub quit_pressed: bool,
    pub debug_pressed: bool,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Phase {
    Title,
    Playing,
    Tally,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum PState {
    Ground,
    Air,
    Cling,
    Swim,
}

#[derive(Debug, PartialEq, Eq)]
pub enum Request {
    PlayFrom(u32),
    Stop,
    Quit,
}

#[derive(Clone, Copy, Debug)]
pub enum Sfx {
    Pluck { note: u8, big: bool },
    Echo,
}

#[derive(Clone, Copy, Debug)]
pub struct Player {
    pub x: i32,
    pub y: i32,
    pub vy: i32,
    pub facing: i32,
    pub state: PState,
    pub cling_dir: i32,
    pub moving: bool,
    pub anim: u32,
    coyote: i32,
    jump_buf: i32,
    kick: i32,
    kick_dir: i32,
    drop_through: i32,
    hop: bool,
}

impl Player {
    fn at_tile(tx: i32, ty: i32) -> Self {
        Player {
            x: tx * TS + (TS - PW) / 2,
            y: (ty + 1) * TS - PH,
            vy: 0,
            facing: 1,
            state: PState::Ground,
            cling_dir: 1,
            moving: false,
            anim: 0,
            coyote: 0,
            jump_buf: 0,
            kick: 0,
            kick_dir: 0,
            drop_through: 0,
            hop: false,
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Drop {
    pub id: u32,
    pub size: u8,
    pub exposure: u16,
    pub grown: u8,
    pub swell: u8,
}

#[derive(Clone, Copy)]
pub struct Particle {
    pub x: i32,
    pub y: i32,
    pub vx: i32,
    pub vy: i32,
    pub life: i32,
    pub col: u8,
}

#[derive(Clone, Copy)]
pub struct Ring {
    pub x: i32,
    pub y: i32,
    pub r: i32,
    pub life: i32,
}

#[derive(Default, Clone, Copy, Debug)]
pub struct Stats {
    pub gathered: u32,
    pub echo_bonus: u32,
    pub lost_sun: u32,
    pub lost_bog: u32,
    pub left_behind: u32,
}

pub struct Game {
    pub level: Level,
    pub phase: Phase,
    pub song_ms: u32,
    pub section: usize,
    pub player: Player,
    pub drops: Vec<Option<Drop>>, // one slot per level site
    next_drop_id: u32,
    /// Ids of drops alive when each echoed section ended; consumed when its echo begins.
    snapshots: Vec<Option<Vec<u32>>>,
    schedule: Vec<(usize, u32)>, // (site, condense time ms)
    pub stats: Stats,
    pub best: u32,
    pub new_best: bool,
    pub particles: Vec<Particle>,
    pub rings: Vec<Ring>,
    pub palettes: [[u32; 64]; 5],
    pub mood_from: Mood,
    pub mood_to: Mood,
    pub mood_mix: u8,
    pub tick: u32,
    pub water_top: i32,
    pub cam_x: i32,
    pub cam_y: i32,
    pub crt: bool,
    pub debug: bool,
    pub music_missing: bool,
    note: u8,
    persist_best: bool,
}

impl Game {
    pub fn new(level: Level, music_missing: bool, persist_best: bool) -> Self {
        let mood = level.sections[0].mood;
        let best = if persist_best {
            std::fs::read_to_string(BEST_FILE).ok().and_then(|s| s.trim().parse().ok()).unwrap_or(0)
        } else {
            0
        };
        let mut g = Game {
            phase: Phase::Title,
            song_ms: 0,
            section: 0,
            player: Player::at_tile(level.start.0, level.start.1),
            drops: vec![None; level.sites.len()],
            next_drop_id: 1,
            snapshots: vec![None; level.sections.len()],
            schedule: Vec::new(),
            stats: Stats::default(),
            best,
            new_best: false,
            particles: Vec::new(),
            rings: Vec::new(),
            palettes: MOODS.map(palette::build),
            mood_from: mood,
            mood_to: mood,
            mood_mix: 16,
            tick: 0,
            water_top: level.water_top_px(0),
            cam_x: 0,
            cam_y: 0,
            crt: true,
            debug: false,
            music_missing,
            note: 0,
            persist_best,
            level,
        };
        g.snap_camera();
        g
    }

    pub fn palette_pair(&self) -> (&[u32; 64], &[u32; 64], u8) {
        (&self.palettes[self.mood_from.index()], &self.palettes[self.mood_to.index()], self.mood_mix)
    }

    /// True while a later echo section still remembers this drop.
    pub fn is_remembered(&self, id: u32) -> bool {
        self.snapshots.iter().flatten().any(|ids| ids.contains(&id))
    }

    fn reset_world(&mut self) {
        let mood = self.level.sections[0].mood;
        self.player = Player::at_tile(self.level.start.0, self.level.start.1);
        self.drops.iter_mut().for_each(|d| *d = None);
        self.snapshots.iter_mut().for_each(|s| *s = None);
        self.schedule.clear();
        self.particles.clear();
        self.rings.clear();
        self.stats = Stats::default();
        self.new_best = false;
        self.song_ms = 0;
        self.section = 0;
        self.note = 0;
        self.mood_from = mood;
        self.mood_to = mood;
        self.mood_mix = 16;
        self.water_top = self.level.water_top_px(0);
        self.snap_camera();
    }

    pub fn start_run(&mut self, req: &mut Vec<Request>) {
        self.reset_world();
        self.phase = Phase::Playing;
        let mut sfx = Vec::new();
        self.begin_section(0, &mut sfx);
        req.push(Request::PlayFrom(0));
    }

    pub fn update(&mut self, inp: &Input, song_ms: u32, req: &mut Vec<Request>, sfx: &mut Vec<Sfx>) {
        self.tick = self.tick.wrapping_add(1);
        if inp.quit_pressed {
            req.push(Request::Quit);
            return;
        }
        if inp.debug_pressed {
            self.debug = !self.debug;
        }
        if self.mood_mix < 16 {
            self.mood_mix += 1;
        }
        match self.phase {
            Phase::Title => {
                if inp.jump_pressed {
                    self.start_run(req);
                } else {
                    step_player(&mut self.player, &self.level, self.water_top, inp);
                }
            }
            Phase::Playing => {
                if inp.restart_pressed {
                    self.start_run(req);
                    return;
                }
                if inp.skip_pressed {
                    if let Some(next) = self.level.sections.get(self.section + 1) {
                        req.push(Request::PlayFrom(next.start_ms));
                    }
                }
                self.advance_song(song_ms, sfx);
                step_player(&mut self.player, &self.level, self.water_top, inp);
                self.collect(sfx);
                if self.song_ms >= self.level.len_ms {
                    self.finish(req);
                }
            }
            Phase::Tally => {
                if inp.jump_pressed {
                    self.start_run(req);
                }
            }
        }
        self.update_effects();
        self.follow_camera();
    }

    /// Moves the world to `now`: crosses section boundaries, condenses, evaporates and floods.
    pub fn advance_song(&mut self, now: u32, sfx: &mut Vec<Sfx>) {
        let now = now.min(self.level.len_ms);
        self.song_ms = now;
        while self.section + 1 < self.level.sections.len() && now >= self.level.sections[self.section + 1].start_ms {
            self.end_section(self.section);
            self.section += 1;
            self.begin_section(self.section, sfx);
        }

        let mut i = 0;
        while i < self.schedule.len() {
            let (site, at) = self.schedule[i];
            if at <= now {
                self.schedule.swap_remove(i);
                if self.drops[site].is_none() {
                    self.spawn_drop(site);
                }
            } else {
                i += 1;
            }
        }

        if self.level.sections[self.section].dew == DewMode::Evaporate {
            for (site, slot) in self.drops.iter_mut().enumerate() {
                let s = self.level.sites[site];
                let Some(d) = slot else { continue };
                if !s.exposed {
                    continue;
                }
                d.exposure += 1;
                let (cx, top) = (s.tx * TS + TS / 2, (s.ty + 1) * TS - drop_dims(d.size).1);
                if d.exposure > EVAP_TICKS / 2 && d.exposure % 10 == 0 {
                    self.particles.push(Particle { x: cx, y: top - 1, vx: 0, vy: -1, life: 8, col: MIST + 1 });
                }
                if d.exposure >= EVAP_TICKS {
                    d.exposure = 0;
                    d.size -= 1;
                    self.stats.lost_sun += 1;
                    if d.size == 0 {
                        *slot = None;
                    }
                }
            }
        }

        self.water_top = self.level.water_top_px(now);
        for (site, slot) in self.drops.iter_mut().enumerate() {
            let s = self.level.sites[site];
            if let Some(d) = slot {
                if self.water_top <= s.ty * TS + TS / 2 {
                    self.stats.lost_bog += d.size as u32;
                    for k in 0..3 {
                        self.particles.push(Particle {
                            x: s.tx * TS + 6 + k * 4,
                            y: self.water_top,
                            vx: k - 1,
                            vy: -1,
                            life: 5,
                            col: WATER + 2,
                        });
                    }
                    *slot = None;
                }
            }
        }
    }

    fn end_section(&mut self, i: usize) {
        if self.level.sections.iter().any(|s| s.echo_of == Some(i)) {
            self.snapshots[i] = Some(self.drops.iter().flatten().map(|d| d.id).collect());
        }
    }

    fn begin_section(&mut self, i: usize, sfx: &mut Vec<Sfx>) {
        let sec = self.level.sections[i].clone();
        if sec.mood != self.mood_to {
            self.mood_from = self.mood_to;
            self.mood_to = sec.mood;
            self.mood_mix = 0;
        }

        if let Some(src) = sec.echo_of {
            if let Some(ids) = self.snapshots[src].take() {
                let mut grew = false;
                for (site, slot) in self.drops.iter_mut().enumerate() {
                    let Some(d) = slot else { continue };
                    if !ids.contains(&d.id) {
                        continue;
                    }
                    let before = d.size;
                    d.size = (d.size + ECHO_GROWTH).min(MAX_SIZE);
                    d.grown += d.size - before;
                    d.swell = 9;
                    grew = true;
                    let s = self.level.sites[site];
                    self.rings.push(Ring { x: s.tx * TS + TS / 2, y: s.ty * TS + TS / 2, r: 3, life: 7 });
                }
                if grew {
                    sfx.push(Sfx::Echo);
                }
            }
        }

        if let DewMode::Condense(tag) = sec.dew {
            let start = sec.start_ms;
            let span = (self.level.section_end_ms(i) - start) * 7 / 10;
            let mut sites: Vec<usize> = (0..self.level.sites.len()).filter(|&s| self.level.sites[s].tag == tag).collect();
            sites.sort_by_key(|&s| hash(s as i32, i as i32));
            let n = sites.len() as u32;
            for (k, site) in sites.into_iter().enumerate() {
                self.schedule.push((site, start + 2000 + (k as u32) * span / n.max(1)));
            }
        }
    }

    fn spawn_drop(&mut self, site: usize) {
        self.drops[site] = Some(Drop { id: self.next_drop_id, size: 1, exposure: 0, grown: 0, swell: 0 });
        self.next_drop_id += 1;
        let s = self.level.sites[site];
        let (cx, cy) = (s.tx * TS + TS / 2, s.ty * TS + TS / 2);
        for (vx, vy) in [(-1, -1), (1, -1), (0, -2)] {
            self.particles.push(Particle { x: cx + vx * 3, y: cy + vy * 2, vx: 0, vy: 0, life: 4, col: DEW + 2 });
        }
    }

    /// Pixel rect of the drop sprite at a site; also the pickup area.
    pub fn drop_rect(&self, site: usize, size: u8) -> (i32, i32, i32, i32) {
        let s = self.level.sites[site];
        let (w, h) = drop_dims(size);
        let x = s.tx * TS + (TS - w) / 2;
        let y = if s.hanging { s.ty * TS } else { (s.ty + 1) * TS - h };
        (x, y, w, h)
    }

    fn collect(&mut self, sfx: &mut Vec<Sfx>) {
        let p = self.player;
        for site in 0..self.drops.len() {
            let Some(d) = self.drops[site] else { continue };
            let (x, y, w, h) = self.drop_rect(site, d.size);
            if p.x < x + w && p.x + PW > x && p.y < y + h && p.y + PH > y {
                self.drops[site] = None;
                self.stats.gathered += d.size as u32;
                if d.grown > 0 {
                    self.stats.echo_bonus += d.size as u32 - 1;
                }
                sfx.push(Sfx::Pluck { note: self.note % 5, big: d.size >= 3 });
                self.note = self.note.wrapping_add(1);
                let (cx, cy) = (x + w / 2, y + h / 2);
                for (vx, vy) in [(-2, -2), (2, -2), (-3, 0), (3, 0), (0, -3), (-1, 1), (1, 1)] {
                    self.particles.push(Particle { x: cx, y: cy, vx, vy, life: 5, col: DEW + 3 });
                }
            }
        }
    }

    fn finish(&mut self, req: &mut Vec<Request>) {
        self.stats.left_behind = self.drops.iter().flatten().map(|d| d.size as u32).sum();
        self.phase = Phase::Tally;
        if self.stats.gathered > self.best {
            self.best = self.stats.gathered;
            self.new_best = true;
            if self.persist_best {
                let _ = std::fs::write(BEST_FILE, self.best.to_string());
            }
        }
        req.push(Request::Stop);
    }

    fn update_effects(&mut self) {
        for d in self.drops.iter_mut().flatten() {
            d.swell = d.swell.saturating_sub(1);
        }
        for p in &mut self.particles {
            p.x += p.vx;
            p.y += p.vy;
            p.life -= 1;
        }
        self.particles.retain(|p| p.life > 0);
        for r in &mut self.rings {
            r.r += 2;
            r.life -= 1;
        }
        self.rings.retain(|r| r.life > 0);
    }

    fn camera_target(&self) -> (i32, i32) {
        let p = &self.player;
        let tx = p.x + PW / 2 - VIEW_W / 2 + p.facing * 24;
        let ty = p.y + PH / 2 - VIEW_H / 2 - 10;
        (
            tx.clamp(0, self.level.w * TS - VIEW_W),
            ty.clamp(0, self.level.h * TS - VIEW_H),
        )
    }

    pub fn snap_camera(&mut self) {
        (self.cam_x, self.cam_y) = self.camera_target();
    }

    fn follow_camera(&mut self) {
        let (tx, ty) = self.camera_target();
        self.cam_x += (tx - self.cam_x).clamp(-8, 8);
        self.cam_y += (ty - self.cam_y).clamp(-8, 8);
    }
}

pub fn hash(x: i32, y: i32) -> u32 {
    let mut h = (x as u32).wrapping_mul(0x27d4_eb2d) ^ (y as u32).wrapping_mul(0x1656_67b1);
    h ^= h >> 15;
    h = h.wrapping_mul(0x85eb_ca6b);
    h ^= h >> 13;
    h
}

fn rect_solid(lvl: &Level, x: i32, y: i32, w: i32, h: i32) -> bool {
    for ty in y.div_euclid(TS)..=(y + h - 1).div_euclid(TS) {
        for tx in x.div_euclid(TS)..=(x + w - 1).div_euclid(TS) {
            if lvl.tile(tx, ty).solid() {
                return true;
            }
        }
    }
    false
}

/// Standing exactly on top of a one-way platform.
fn oneway_under(lvl: &Level, x: i32, y: i32, w: i32, h: i32) -> bool {
    let bottom = y + h;
    if bottom.rem_euclid(TS) != 0 {
        return false;
    }
    let ty = bottom / TS;
    (x.div_euclid(TS)..=(x + w - 1).div_euclid(TS)).any(|tx| lvl.tile(tx, ty).oneway())
}

fn grounded(p: &Player, lvl: &Level) -> bool {
    rect_solid(lvl, p.x, p.y + 1, PW, PH) || (p.drop_through == 0 && oneway_under(lvl, p.x, p.y, PW, PH))
}

/// One 15 FPS step of Twen's movement: walk, jump, cling and climb walls, swim.
pub fn step_player(p: &mut Player, lvl: &Level, water_top: i32, inp: &Input) {
    let dir = inp.right as i32 - inp.left as i32;
    p.drop_through = (p.drop_through - 1).max(0);
    p.jump_buf = (p.jump_buf - 1).max(0);
    if inp.jump_pressed {
        p.jump_buf = 3;
    }
    let swimming = p.y + PH / 2 >= water_top;
    p.moving = false;

    if p.state == PState::Cling {
        let d = p.cling_dir;
        let wall_top = rect_solid(lvl, p.x + d, p.y, PW, 2);
        let wall_low = rect_solid(lvl, p.x + d, p.y + PH - 2, PW, 2);
        let ledge_close = !rect_solid(lvl, p.x + d, p.y - 8, PW, 2);
        if swimming {
            p.state = PState::Swim;
        } else if p.jump_buf > 0 {
            p.jump_buf = 0;
            p.state = PState::Air;
            p.vy = -13;
            p.kick = 5;
            p.kick_dir = -d;
            p.facing = -d;
        } else if dir == -d || (!wall_top && !wall_low) {
            p.state = PState::Air;
            p.vy = 0;
        } else if (inp.up && !wall_top) || (dir == d && ledge_close) {
            // At the top edge (or pressing into a wall just below its lip): hop up and over.
            p.state = PState::Air;
            p.vy = -10;
            p.kick = 4;
            p.kick_dir = d;
        } else if inp.up {
            for _ in 0..3 {
                if rect_solid(lvl, p.x, p.y - 1, PW, PH) {
                    break;
                }
                p.y -= 1;
                p.moving = true;
            }
        } else if inp.down {
            for _ in 0..3 {
                if grounded(p, lvl) {
                    p.state = PState::Ground;
                    break;
                }
                p.y += 1;
                p.moving = true;
            }
        }
        if p.state == PState::Cling {
            p.vy = 0;
            if p.moving {
                p.anim += 1;
            }
            return;
        }
    }

    // Horizontal movement.
    let vx = if p.kick > 0 {
        p.kick -= 1;
        p.kick_dir * 4
    } else {
        if dir != 0 {
            p.facing = dir;
        }
        dir * if swimming { 3 } else { 4 }
    };
    for _ in 0..vx.abs() {
        if rect_solid(lvl, p.x + vx.signum(), p.y, PW, PH) {
            break;
        }
        p.x += vx.signum();
        p.moving = true;
    }

    // Sticky feet: pressing into a wall in mid-air, or pressing up beside one, starts a cling.
    let on_ground = grounded(p, lvl);
    let wall_dir = if dir != 0 && rect_solid(lvl, p.x + dir, p.y, PW, PH) {
        dir
    } else if inp.up && rect_solid(lvl, p.x + 1, p.y, PW, PH) {
        1
    } else if inp.up && rect_solid(lvl, p.x - 1, p.y, PW, PH) {
        -1
    } else {
        0
    };
    if !swimming && p.kick == 0 && wall_dir != 0 && (!on_ground || inp.up) {
        p.state = PState::Cling;
        p.cling_dir = wall_dir;
        p.facing = wall_dir;
        p.vy = 0;
        return;
    }

    // Vertical movement.
    if swimming {
        if p.jump_buf > 0 {
            p.jump_buf = 0;
            p.vy = -10; // stroke: strong enough to leap out at the surface
        } else if p.vy < -3 {
            p.vy += 1;
        } else if inp.jump {
            p.vy = -2;
        } else {
            p.vy = (p.vy + 1).min(2);
        }
    } else {
        if on_ground {
            p.coyote = 3;
        } else {
            p.coyote = (p.coyote - 1).max(0);
        }
        if p.jump_buf > 0 && p.coyote > 0 && p.vy >= 0 {
            p.jump_buf = 0;
            p.coyote = 0;
            p.vy = -15;
            p.hop = true;
        } else if on_ground && p.vy >= 0 {
            p.vy = 0;
            if inp.down_pressed && !rect_solid(lvl, p.x, p.y + 1, PW, PH) {
                p.drop_through = 4;
                p.vy = 2;
            }
        } else {
            p.vy = (p.vy + 2).min(12);
        }
        if p.hop && !inp.jump && p.vy < -5 {
            p.vy = -5;
        }
    }

    for _ in 0..p.vy.abs() {
        let blocked = if p.vy > 0 {
            grounded(p, lvl)
        } else {
            rect_solid(lvl, p.x, p.y - 1, PW, PH)
        };
        if blocked {
            p.vy = 0;
            break;
        }
        p.y += p.vy.signum();
    }

    let now_swimming = p.y + PH / 2 >= water_top;
    if now_swimming {
        p.state = PState::Swim;
    } else if grounded(p, lvl) && p.vy >= 0 {
        p.state = PState::Ground;
        p.hop = false;
    } else {
        p.state = PState::Air;
    }
    if p.moving {
        p.anim += 1;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const UNDERBRUSH: &str = include_str!("../levels/underbrush.lvl");

    fn game() -> Game {
        Game::new(Level::parse(UNDERBRUSH).expect("level parses"), true, false)
    }

    fn sites_with(g: &Game, tag: char) -> Vec<usize> {
        (0..g.level.sites.len()).filter(|&i| g.level.sites[i].tag == tag).collect()
    }

    /// Runs the song from 0 to `until_ms` in 15 FPS ticks with Twen standing still at the start.
    fn run_idle(g: &mut Game, until_ms: u32) {
        let mut req = Vec::new();
        if g.phase != Phase::Playing {
            g.start_run(&mut req);
        }
        let mut sfx = Vec::new();
        let mut t: u64 = (g.song_ms as u64 * 3).div_ceil(200);
        loop {
            let ms = (t * 200 / 3) as u32;
            if ms > until_ms {
                break;
            }
            g.update(&Input::default(), ms, &mut req, &mut sfx);
            t += 1;
        }
    }

    #[test]
    fn underbrush_parses_with_expected_sites_and_echoes() {
        let g = game();
        let l = &g.level;
        assert_eq!((l.w, l.h), (60, 24));
        assert_eq!(l.len_ms, 352_000);
        assert_eq!(sites_with(&g, 'c').len(), 16);
        assert_eq!(sites_with(&g, 'v').len(), 10);
        assert_eq!(sites_with(&g, 'r').len(), 6);
        let exposed = sites_with(&g, 'c').iter().filter(|&&s| l.sites[s].exposed).count();
        assert_eq!(exposed, 6);
        assert_eq!(l.sections.len(), 7);
        assert_eq!(l.sections[5].echo_of, Some(0));
        assert_eq!(l.sections[6].echo_of, Some(2));
        assert!(sites_with(&g, 'v').iter().filter(|&&s| l.sites[s].hanging).count() == 4);
    }

    #[test]
    fn mist_dew_condenses_before_the_cavern() {
        let mut g = game();
        run_idle(&mut g, 53_000);
        let present = sites_with(&g, 'c').iter().filter(|&&s| g.drops[s].is_some()).count();
        assert_eq!(present, 16);
    }

    #[test]
    fn sun_evaporates_exposed_dew_but_spares_shade() {
        let mut g = game();
        run_idle(&mut g, 2 * 60_000 + 44_000);
        for s in sites_with(&g, 'c') {
            assert_eq!(g.drops[s].is_some(), !g.level.sites[s].exposed, "site {s}");
        }
        assert_eq!(g.stats.lost_sun, 6);
    }

    #[test]
    fn flood_dissolves_low_cavern_and_reed_dew() {
        let mut g = game();
        run_idle(&mut g, 3 * 60_000 + 40_000);
        for s in sites_with(&g, 'v') {
            let low = g.level.sites[s].ty >= 17;
            assert_eq!(g.drops[s].is_none(), low, "cavern site {s}");
        }
        assert!(sites_with(&g, 'r').iter().all(|&s| g.drops[s].is_none()));
        assert_eq!(g.stats.lost_bog, 4 + 6);
    }

    #[test]
    fn echoes_grow_dew_left_through_its_source_section() {
        let mut g = game();
        run_idle(&mut g, 3 * 60_000 + 56_000);
        for s in sites_with(&g, 'c') {
            let d = g.drops[s];
            if g.level.sites[s].exposed {
                assert!(d.map_or(true, |d| d.size == 1 && d.grown == 0), "re-condensed exposed site {s}");
            } else {
                assert_eq!(d.map(|d| d.size), Some(3), "shaded canopy site {s} after lament echo");
            }
        }
        run_idle(&mut g, 4 * 60_000 + 49_000);
        for s in sites_with(&g, 'c').into_iter().filter(|&s| !g.level.sites[s].exposed) {
            assert_eq!(g.drops[s].map(|d| d.size), Some(5), "shaded canopy site {s} after finale echo");
        }
        for s in sites_with(&g, 'v').into_iter().filter(|&s| g.level.sites[s].ty < 17) {
            assert_eq!(g.drops[s].map(|d| d.size), Some(3), "high cavern site {s} after finale echo");
        }
    }

    #[test]
    fn song_end_tallies_and_requests_stop() {
        let mut g = game();
        let mut req = Vec::new();
        g.start_run(&mut req);
        run_idle(&mut g, 352_000);
        assert_eq!(g.phase, Phase::Tally);
        assert_eq!(g.stats.gathered, 0);
        assert!(g.stats.left_behind > 0);
    }

    #[test]
    fn collecting_a_grown_drop_counts_echo_bonus() {
        let mut g = game();
        run_idle(&mut g, 4 * 60_000);
        let site = sites_with(&g, 'c').into_iter().find(|&s| g.drops[s].map_or(false, |d| d.size == 3)).unwrap();
        let (x, y, w, h) = g.drop_rect(site, 3);
        g.player.x = x + w / 2 - PW / 2;
        g.player.y = y + h - PH;
        let mut sfx = Vec::new();
        g.collect(&mut sfx);
        assert_eq!(g.stats.gathered, 3);
        assert_eq!(g.stats.echo_bonus, 2);
    }

    fn steps(p: &mut Player, g: &Game, inp: Input, n: usize) {
        for _ in 0..n {
            step_player(p, &g.level, g.water_top, &inp);
        }
    }

    #[test]
    fn twen_climbs_a_trunk_and_mantles_onto_its_top() {
        let g = game();
        let mut p = Player::at_tile(6, 18);
        p.x = 6 * TS; // flush against the right side of trunk A (cols 4-5)
        steps(&mut p, &g, Input { up: true, ..Default::default() }, 1);
        assert_eq!((p.state, p.cling_dir), (PState::Cling, -1));
        steps(&mut p, &g, Input { up: true, ..Default::default() }, 120);
        steps(&mut p, &g, Input::default(), 20);
        assert_eq!((p.y + PH, p.state), (3 * TS, PState::Ground), "standing on the trunk top");
    }

    #[test]
    fn twen_leaps_out_of_the_bog_onto_a_hummock() {
        let g = game();
        let mut p = Player::at_tile(45, 20);
        p.y = 20 * TS - PH / 2; // floating at the surface beside hummock 1 (cols 46-48)
        let leap = Input { right: true, jump: true, jump_pressed: true, ..Default::default() };
        steps(&mut p, &g, leap, 1);
        steps(&mut p, &g, Input { right: true, ..Default::default() }, 20);
        assert_eq!((p.y + PH, p.state), (19 * TS, PState::Ground));
    }

    /// A seeded random-input explorer must be able to touch every dew site at normal water.
    #[test]
    fn every_dew_site_is_reachable() {
        let g = game();
        let mut seen = vec![false; g.level.sites.len()];
        let mut rng: u32 = 0x5EED;
        let mut next = |n: u32| {
            rng = rng.wrapping_mul(1_664_525).wrapping_add(1_013_904_223);
            (rng >> 8) % n
        };
        for _run in 0..40 {
            let mut p = g.player;
            for _ in 0..400 {
                let pick = next(9);
                let hold = 3 + next(30);
                let inp = Input {
                    left: matches!(pick, 0 | 2 | 4),
                    right: matches!(pick, 1 | 3 | 5),
                    up: matches!(pick, 2 | 3 | 6),
                    down: pick == 7,
                    jump: matches!(pick, 4 | 5 | 8),
                    ..Default::default()
                };
                for t in 0..hold {
                    let mut i = inp;
                    i.jump_pressed = inp.jump && t == 0;
                    i.down_pressed = inp.down && t == 0;
                    step_player(&mut p, &g.level, g.water_top, &i);
                    for (s, hit) in seen.iter_mut().enumerate() {
                        let (x, y, w, h) = g.drop_rect(s, 1);
                        if p.x < x + w && p.x + PW > x && p.y < y + h && p.y + PH > y {
                            *hit = true;
                        }
                    }
                }
            }
        }
        let missed: Vec<(i32, i32)> = (0..seen.len()).filter(|&s| !seen[s]).map(|s| (g.level.sites[s].tx, g.level.sites[s].ty)).collect();
        assert!(missed.is_empty(), "unreached sites (tx, ty): {missed:?}");
    }

    #[test]
    fn twen_walks_jumps_and_lands_on_the_floor() {
        let g = game();
        let mut p = g.player;
        let floor_y = p.y;
        let right = Input { right: true, ..Default::default() };
        for _ in 0..5 {
            step_player(&mut p, &g.level, g.water_top, &right);
        }
        assert_eq!(p.y, floor_y);
        assert!(p.x > g.player.x);
        let jump = Input { jump: true, jump_pressed: true, ..Default::default() };
        step_player(&mut p, &g.level, g.water_top, &jump);
        assert!(p.y < floor_y);
        let hold = Input { jump: true, ..Default::default() };
        for _ in 0..30 {
            step_player(&mut p, &g.level, g.water_top, &hold);
        }
        assert_eq!((p.y, p.state), (floor_y, PState::Ground));
    }
}
