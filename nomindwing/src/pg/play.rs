//! The match screen: fly your king, have it print your deck, plant seeds on rocks, and watch your
//! factories grow and print. The AI plays the other side with the starter deck.

use super::design::*;
use super::scene;
use super::sim::*;
use super::{draw_icon, pc, team_name, Pg, Screen, FIELD};
use crate::app::Key;
use crate::framebuffer::{wrap, Framebuffer};
use crate::geom::dist2;
use crate::palette::*;
use crate::ui::{self, Rect};

/// The whole map at exactly 1/13 scale.
const MINI_SCALE: i32 = 13;
const MINI: Rect = Rect::new(306, 198, MW / MINI_SCALE + 2, MH / MINI_SCALE + 2);
const BTN_PAUSE: Rect = Rect::new(306, 163, 44, 15);
const BTN_SPEED: Rect = Rect::new(352, 163, 44, 15);
const BTN_LEAVE: Rect = Rect::new(306, 180, 90, 15);
const OVER: Rect = Rect::new(52, 100, 200, 88);
const OVER_BTNS: [Rect; 2] = [Rect::new(66, 162, 80, 16), Rect::new(158, 162, 80, 16)];
const MENU: Rect = Rect::new(80, 80, 240, 130);
const MENU_BTNS: [Rect; 3] = [Rect::new(92, 168, 70, 18), Rect::new(166, 168, 76, 18), Rect::new(246, 168, 62, 18)];

fn card_rect(i: usize) -> Rect {
    Rect::new(306, 14 + i as i32 * 18, 90, 17)
}

/// The match's rules panel, by heading.
const MATCH_HELP: [(&str, &str); 7] = [
    ("GOAL", "KILL THE ENEMY KING. LOSE YOURS AND YOU LOSE."),
    ("KING", "WASD OR THE ARROWS FLY IT. 1-7 OR A CARD: IT PRINTS THAT DESIGN FROM YOUR BANK, AND IT ARRIVES BY HYPERSPACE."),
    ("SEEDS", "A SEEDED CARD MAKES A SEED: CLICK A ROCK FOR IT. IT FLIES THERE AND GROWS ITS DESIGN OUT OF THE ROCK, PIXEL BY PIXEL. ONE WITH ENGINES FLIES OFF WHEN IT'S WHOLE."),
    ("FACTORY", "IT PRINTS ITS BUILD ORDER ROUND AND ROUND, A PIXEL AT A TIME, SKIPPING WHAT IT'S TOO SMALL FOR AND WAITING WHEN THE BANK IS EMPTY."),
    ("BANK", "MINERS HAUL TO THE NEAREST NODE (KING OR FACTORY). LINKED NODES SHARE A BANK; CUT A LINK AND IT SPLITS."),
    ("SOLID", "WHAT'S PLANTED ON A ROCK IS SOLID TO EVERY ENEMY SHIP. AN ENEMY SHIP BIGGER THAN YOURS IS SOLID TO YOU. ROCKS STOP NOTHING."),
    ("KEYS", "SPACE PAUSE  F SPEED  ESC CANCEL/QUIT  F1 HELP"),
];

/// A match in progress, and how you're looking at it.
pub struct Play {
    pub arena: Arena,
    pub cam: (i32, i32),
    pub follow: bool,
    /// A seeded card picked, waiting for you to click the rock its seed should go to.
    pub planting: Option<usize>,
    pub running: bool,
    pub speed: u32,
    pub starter: bool,
    pub seed: u64,
    map_drag: bool,
    logged_over: bool,
    next_log: u32,
}

impl Pg {
    // ----- Starting and leaving -----

    /// Starts a match with your seven designs as your deck, or the starter deck. The AI always
    /// plays the starter deck.
    pub fn start_match(&mut self, starter: bool, seed: u64) {
        let mine = if starter { starter_deck() } else { self.lib.clone() };
        let arena = Arena::for_match(seed, [mine, starter_deck()], [false, true]);
        let mut p = Play { arena, cam: (0, 0), follow: true, planting: None, running: true, speed: 1, starter, seed, map_drag: false, logged_over: false, next_log: 15 * 60 };
        p.look_at_king();
        self.play = Some(p);
        self.menu = false;
        self.screen = Screen::Match;
        // The rules first: Toy 1's were hard to work out from play alone.
        self.help = true;
        let deck = if starter { "STARTER DECK" } else { "YOUR DECK" };
        self.log(&format!("MATCH START seed {seed}, {deck}"));
        if !starter {
            for d in self.lib.clone() {
                self.log(&format!("  DESIGN {}", d.to_line()));
            }
        }
    }

    fn leave_match(&mut self) {
        if let Some(p) = &self.play {
            if p.arena.over.is_none() {
                let line = format!("MATCH LEFT at {} | {}", clock(p.arena.tick), Self::match_state(&p.arena));
                self.log(&line);
            }
        }
        self.play = None;
        self.screen = Screen::Yard;
        self.frame_if_new();
    }

    fn match_state(a: &Arena) -> String {
        state_line(a)
    }

    // ----- Ticks -----

    pub(super) fn step_match(&mut self, dirs: &[bool]) {
        let Some(p) = self.play.as_mut() else { return };
        let steer = (dirs[3] as i32 - dirs[2] as i32, dirs[1] as i32 - dirs[0] as i32);
        if steer != (0, 0) {
            p.follow = true;
        }
        if p.arena.eco.as_ref().is_some_and(|e| !e.ai[0]) {
            p.arena.command(Cmd::Steer { team: 0, dx: steer.0, dy: steer.1 });
        }
        if p.running && p.arena.over.is_none() {
            for _ in 0..p.speed {
                p.arena.step();
            }
        }
        if p.follow {
            p.look_at_king();
        }
        let mut lines = Vec::new();
        if p.arena.tick >= p.next_log && p.arena.over.is_none() {
            p.next_log += 15 * 60;
            lines.push(format!("MATCH {} | {}", clock(p.arena.tick), Self::match_state(&p.arena)));
        }
        if let Some(result) = p.arena.over {
            if !p.logged_over {
                p.logged_over = true;
                p.running = false;
                let who = match result {
                    Some(0) => "BLUE (YOU) WIN".to_string(),
                    Some(_) => "RED (AI) WINS".to_string(),
                    None => "DRAW".to_string(),
                };
                lines.push(format!("MATCH RESULT {who} at {} | {}", clock(p.arena.tick), Self::match_state(&p.arena)));
            }
        }
        for l in lines {
            self.log(&l);
        }
    }

    // ----- Input -----

    pub(super) fn key_match(&mut self, k: Key) {
        let Some(p) = self.play.as_mut() else { return };
        match k {
            Key::Num(n) if (1..=7).contains(&n) => self.card(n as usize - 1),
            Key::Space | Key::Letter('P') => {
                if p.arena.over.is_none() {
                    p.running = !p.running;
                }
            }
            Key::Letter('F') => p.speed = if p.speed >= 4 { 1 } else { p.speed * 2 },
            _ => {}
        }
    }

    /// A card: the king prints a built design straight away; a seeded one waits for a rock.
    fn card(&mut self, i: usize) {
        let Some(p) = self.play.as_mut() else { return };
        let Some(card) = p.arena.eco.as_ref().and_then(|e| e.cards[0].get(i).copied()) else { return };
        if !card.ok {
            self.say("THAT DESIGN CAN'T FLY YET: FIX IT IN THE YARD");
            return;
        }
        if card.seeded {
            p.planting = Some(i);
            self.say("CLICK A ROCK FOR THE SEED. RIGHT-CLICK CANCELS");
        } else {
            match p.arena.can_print(0, i, None) {
                Ok(_) => p.arena.command(Cmd::Print { team: 0, slot: i, target: None }),
                Err(why) => self.say(why),
            }
        }
    }

    /// Esc while planting cancels the planting. True if it did.
    pub(super) fn cancel_planting(&mut self) -> bool {
        match self.play.as_mut() {
            Some(p) if p.planting.is_some() => {
                p.planting = None;
                true
            }
            _ => false,
        }
    }

    pub(super) fn press_match(&mut self, x: i32, y: i32, right: bool) {
        let Some(p) = self.play.as_mut() else { return };
        if p.arena.over.is_some() && !right {
            if OVER_BTNS[0].contains(x, y) {
                let (starter, seed) = (p.starter, p.seed + 1);
                self.start_match(starter, seed);
                return;
            }
            if OVER_BTNS[1].contains(x, y) {
                self.leave_match();
                return;
            }
        }
        if right {
            p.planting = None;
            return;
        }
        if FIELD.contains(x, y) {
            if let Some(slot) = p.planting {
                let at = (x + p.cam.0, y + p.cam.1);
                match p.arena.rock_near(at, 6) {
                    Some(target) => {
                        p.planting = None;
                        match p.arena.can_print(0, slot, Some(target)) {
                            Ok(_) => p.arena.command(Cmd::Print { team: 0, slot, target: Some(target) }),
                            Err(why) => self.say(why),
                        }
                    }
                    None => self.say("THAT'S NOT A ROCK: CLICK ON ONE"),
                }
            }
            return;
        }
        if let Some(i) = (0..7).find(|&i| card_rect(i).contains(x, y)) {
            self.card(i);
        } else if MINI.contains(x, y) {
            if let Some(slot) = p.planting {
                // Planting from the minimap: the rock nearest the click.
                let at = ((x - MINI.x - 1) * MINI_SCALE, (y - MINI.y - 1) * MINI_SCALE);
                match p.arena.rock_near(at, MINI_SCALE) {
                    Some(target) => {
                        p.planting = None;
                        match p.arena.can_print(0, slot, Some(target)) {
                            Ok(_) => p.arena.command(Cmd::Print { team: 0, slot, target: Some(target) }),
                            Err(why) => self.say(why),
                        }
                    }
                    None => self.say("THAT'S NOT A ROCK: CLICK ON ONE"),
                }
                return;
            }
            p.map_drag = true;
            p.cam_from_map(x, y);
        } else if BTN_PAUSE.contains(x, y) {
            if p.arena.over.is_none() {
                p.running = !p.running;
            }
        } else if BTN_SPEED.contains(x, y) {
            p.speed = if p.speed >= 4 { 1 } else { p.speed * 2 };
        } else if BTN_LEAVE.contains(x, y) {
            self.leave_match();
        }
    }

    pub(super) fn drag_match(&mut self, x: i32, y: i32) {
        if let Some(p) = self.play.as_mut() {
            if p.map_drag {
                p.cam_from_map(x.clamp(MINI.x + 1, MINI.x + MINI.w - 2), y.clamp(MINI.y + 1, MINI.y + MINI.h - 2));
            }
        }
    }

    pub(super) fn release_match(&mut self) {
        if let Some(p) = self.play.as_mut() {
            p.map_drag = false;
        }
    }

    pub(super) fn press_menu(&mut self, x: i32, y: i32) {
        if MENU_BTNS[0].contains(x, y) {
            let seed = crate::game::utc_stamp().bytes().fold(7u64, |h, b| h.wrapping_mul(31).wrapping_add(b as u64)) % 100_000 + 2;
            self.start_match(false, seed);
        } else if MENU_BTNS[1].contains(x, y) {
            let seed = crate::game::utc_stamp().bytes().fold(7u64, |h, b| h.wrapping_mul(31).wrapping_add(b as u64)) % 100_000 + 2;
            self.start_match(true, seed);
        } else if MENU_BTNS[2].contains(x, y) || !MENU.contains(x, y) {
            self.menu = false;
        }
    }

    // ----- Drawing -----

    pub(super) fn draw_menu(&self, fb: &mut Framebuffer) {
        let r = MENU;
        ui::panel(fb, r);
        fb.text_centered("PLAY A MATCH", r.x + r.w / 2, r.y + 8, ACCENT, Some(OUTLINE), 2);
        let has_seed = self.lib.iter().any(|d| d.seeded && d.check().ok());
        let mut body = "1 V 1 AGAINST THE AI. KILL ITS KING. YOUR KING PRINTS YOUR DECK; SEEDED DESIGNS GROW INTO FACTORIES ON ROCKS.".to_string();
        if !has_seed {
            body += " YOUR DECK HAS NO SEEDED FACTORY YET, SO ONLY YOUR KING WOULD BUILD.";
        }
        for (i, l) in wrap(&body, 37).iter().enumerate() {
            fb.text(l, r.x + 10, r.y + 30 + i as i32 * 9, TEXT, None, 1);
        }
        let m = self.mouse;
        for (b, label) in MENU_BTNS.iter().zip(["YOUR DECK", "STARTER DECK", "BACK"]) {
            ui::button(fb, *b, label, b.contains(m.0, m.1), false);
        }
    }

    pub(super) fn draw_match(&self, fb: &mut Framebuffer) {
        let Some(p) = &self.play else { return };
        let a = &p.arena;
        let e = a.eco.as_ref().unwrap();
        let m = self.mouse;
        let (cx, cy) = p.cam;
        let f = FIELD;
        scene::backdrop(fb, f, a, p.cam);
        // Network links: dotted lines between linked nodes.
        let nodes: Vec<usize> = (0..a.ships.len()).filter(|&i| a.is_node(i)).collect();
        for (k, &i) in nodes.iter().enumerate() {
            for &j in &nodes[k + 1..] {
                let (si, sj) = (&a.ships[i], &a.ships[j]);
                let r = reach(si) + reach(sj);
                if si.team == sj.team && e.net.get(i) == e.net.get(j) && dist2(si.x, si.y, sj.x, sj.y) <= r * r {
                    crate::render::dotted(fb, f, si.x - cx, si.y - cy, sj.x - cx, sj.y - cy, 4, team_col(si.team, 0));
                }
            }
        }
        scene::drift(fb, f, a, p.cam);
        let (wx, wy) = (m.0 + cx, m.1 + cy);
        let hovered = if f.contains(m.0, m.1) && p.planting.is_none() { a.ship_at(wx, wy) } else { None };
        // Our seeds' courses.
        for s in a.ships.iter().filter(|s| s.alive && s.team == 0 && s.role == Role::Seed) {
            if let Some((_, (tx, ty))) = s.seed_for {
                crate::render::dotted(fb, f, s.x - cx, s.y - cy, tx - cx, ty - cy, 3, TEXT_FAINT);
            }
        }
        for s in a.ships.iter().filter(|s| s.alive) {
            scene::ship(fb, f, s, p.cam);
        }
        if let Some(i) = hovered {
            scene::corners(fb, &a.ships[i], p.cam);
        }
        scene::fire(fb, f, a, p.cam);
        // Planting: the rock under the pointer and whether the design fits on it.
        let mut bar: Option<Vec<String>> = None;
        if let Some(slot) = p.planting {
            if f.contains(m.0, m.1) {
                let need = e.cards[0][slot].count;
                match a.rock_near((wx, wy), 6) {
                    Some(px) => {
                        let rock = a.rock_blob(px);
                        let (l, t) = (rock.iter().map(|q| q.0).min().unwrap(), rock.iter().map(|q| q.1).min().unwrap());
                        let (r, b) = (rock.iter().map(|q| q.0).max().unwrap(), rock.iter().map(|q| q.1).max().unwrap());
                        let fits = rock.len() as i32 >= need;
                        // Something already planted on this rock, whoever's: the two will grow up
                        // against each other, so neither is likely to finish.
                        let here = a.ships.iter().find(|s| {
                            let (sl, st, sr, sb) = s.bbox();
                            s.alive && s.rooted && sl <= r + 2 && sr >= l - 2 && st <= b + 2 && sb >= t - 2
                        });
                        fb.corners(l - 2 - cx, t - 2 - cy, r - l + 5, b - t + 5, 3, if fits && here.is_none() { GOOD } else { GOLD });
                        let fit = match here {
                            Some(s) => format!("{} {} IS PLANTED HERE ALREADY: THEY'LL WALL EACH OTHER IN.", team_name(s.team), s.name),
                            None if fits => "IT FITS.".to_string(),
                            None => format!("IT'LL STAY {}% GROWN.", rock.len() as i32 * 100 / need.max(1)),
                        };
                        bar = Some(wrap(&format!("PLANT {} HERE: ROCK {} PX, NEEDS {}. {fit} RIGHT-CLICK OR ESC CANCELS.", e.deck[0][slot].name, rock.len(), need), 49));
                    }
                    None => bar = Some(vec![format!("PLANT {}: CLICK A ROCK", e.deck[0][slot].name), "RIGHT-CLICK OR ESC CANCELS".into()]),
                }
            }
        }
        fb.text(&clock(a.tick), 4, 3, TEXT_DIM, Some(OUTLINE), 1);
        if e.split_at[0] > 0 && a.tick < e.split_at[0] + 15 * 4 {
            fb.text_centered("YOUR NETWORK SPLIT", f.w / 2, 14, GOLD, Some(OUTLINE), 1);
        }
        if bar.is_none() {
            bar = if let Some(i) = hovered {
                Some(ship_lines(a, i))
            } else if f.contains(m.0, m.1) {
                a.rock_near((wx, wy), 2).map(|px| vec![format!("ROCK: {} PX LEFT. ANYONE CAN MINE IT, AND A SEED CAN GROW A FACTORY OUT OF IT.", a.rock_blob(px).len())])
            } else {
                self.match_tip().map(|t| wrap(&t, 49))
            };
        }
        if let Some(lines) = bar {
            let h = lines.len() as i32 * 9 + 6;
            let r = Rect::new(0, f.h - h, f.w, h);
            ui::panel(fb, r);
            for (k, l) in lines.iter().enumerate() {
                fb.text(l, 4, r.y + 4 + k as i32 * 9, TEXT, None, 1);
            }
        }
        if let Some(result) = a.over {
            let r = OVER;
            ui::panel(fb, r);
            let (head, col) = match result {
                Some(0) => ("YOU WIN", team_col(0, 2)),
                Some(_) => ("YOU LOSE", team_col(1, 2)),
                None => ("DRAW", TEXT),
            };
            fb.text_centered(head, r.x + r.w / 2, r.y + 10, col, Some(OUTLINE), 2);
            let why = match result {
                Some(0) => "THE ENEMY KING IS LOOSE PIXELS",
                Some(_) => "YOUR KING IS LOOSE PIXELS",
                None => "NOBODY'S KING FELL IN 40 MINUTES",
            };
            fb.text_centered(why, r.x + r.w / 2, r.y + 32, TEXT, None, 1);
            fb.text_centered(&format!("AFTER {}", clock(a.tick)), r.x + r.w / 2, r.y + 44, TEXT_DIM, None, 1);
            for (b, label) in OVER_BTNS.iter().zip(["AGAIN", "YARD"]) {
                ui::button(fb, *b, label, b.contains(m.0, m.1), false);
            }
        }

        // Sidebar.
        ui::panel(fb, Rect::new(304, 0, 96, 300));
        let k = e.king[0];
        fb.text(&format!("BANK {}", a.bank(k)), 308, 4, GOLD, None, 1);
        for i in 0..7 {
            let r = card_rect(i);
            let hov = r.contains(m.0, m.1);
            let busy = e.queue[0].first().is_some_and(|j| j.slot == i);
            let picked = p.planting == Some(i);
            if picked || busy {
                fb.rect_bevel(r.x, r.y, r.w, r.h, PANEL_DEEP, PANEL_LO, PANEL_HI);
                fb.rect_outline(r.x + 1, r.y + 1, r.w - 2, r.h - 2, if picked { GOLD } else { ACCENT });
            } else if hov {
                fb.rect_bevel(r.x, r.y, r.w, r.h, PANEL_HI, WHITE, PANEL);
            } else {
                ui::panel(fb, r);
            }
            let Some(d) = e.deck[0].get(i) else { continue };
            let card = e.cards[0][i];
            draw_icon(fb, d, Rect::new(r.x + 2, r.y + 1, 14, 14), 0);
            let name: String = d.name.chars().take(10).collect();
            fb.text(&name, r.x + 19, r.y + 1, if !card.ok { BAD } else if hov && !(busy || picked) { PANEL_DEEP } else { TEXT }, None, 1);
            let what = if card.seeded { format!("SEED {}", card.price()) } else { format!("{} PX", card.price()) };
            fb.text(&what, r.x + 19, r.y + 9, if hov && !(busy || picked) { PANEL_LO } else if card.seeded { FACTORY } else { TEXT_DIM }, None, 1);
            fb.text(&format!("{}", i + 1), r.x + r.w - 8, r.y + 1, TEXT_FAINT, None, 1);
        }
        // What the king is printing.
        match e.queue[0].first() {
            Some(j) => {
                let name = e.deck[0].get(j.slot).map_or("?".into(), |d| if e.cards[0][j.slot].seeded { format!("SEED: {}", d.name) } else { d.name.clone() });
                let name: String = name.chars().take(15).collect();
                fb.text(&name, 308, 142, TEXT, None, 1);
                fb.rect_fill(308, 151, 86, 4, PANEL_DEEP);
                fb.rect_fill(308, 151, 86 * j.paid / j.cost.max(1), 4, ACCENT);
                if e.queue[0].len() > 1 {
                    fb.text(&format!("+{}", e.queue[0].len() - 1), 380, 142, TEXT_DIM, None, 1);
                }
            }
            None => fb.text("KING IDLE", 308, 142, TEXT_FAINT, None, 1),
        }
        ui::button(fb, BTN_PAUSE, if p.running { "PAUSE" } else { "RUN" }, BTN_PAUSE.contains(m.0, m.1), !p.running);
        ui::button(fb, BTN_SPEED, &format!("{}X", p.speed), BTN_SPEED.contains(m.0, m.1), p.speed > 1);
        ui::button(fb, BTN_LEAVE, "LEAVE", BTN_LEAVE.contains(m.0, m.1), false);
        p.draw_minimap(fb);
        for t in 0..2 {
            let y = 274 + t as i32 * 11;
            let ships = a.ships.iter().filter(|s| s.alive && s.team == t && s.role == Role::Ship && s.grow.is_none() && !s.node).count();
            let factories = a.ships.iter().filter(|s| s.alive && s.team == t && s.role == Role::Ship && s.node).count();
            fb.text(team_name(t), 308, y, team_col(t, 2), None, 1);
            fb.text(&format!("{factories}F {ships}S"), 340, y, TEXT, None, 1);
        }
    }

    fn match_tip(&self) -> Option<String> {
        let m = self.mouse;
        let p = self.play.as_ref()?;
        let e = p.arena.eco.as_ref()?;
        if let Some(i) = (0..7).find(|&i| card_rect(i).contains(m.0, m.1)) {
            let d = e.deck[0].get(i)?;
            let c = e.cards[0][i];
            let builds: Vec<String> = d.build.iter().filter_map(|&b| e.deck[0].get(b)).map(|o| o.name.clone()).collect();
            let what = if c.seeded {
                format!("[{}] {}: YOUR KING PRINTS A SEED ({} PX) THAT GROWS IT OUT OF A ROCK YOU CLICK. IT NEEDS {} PX OF ROCK.", i + 1, d.name, SEED_COST, c.count)
            } else {
                format!("[{}] {}: YOUR KING PRINTS IT FOR {} PX AND IT ARRIVES BY HYPERSPACE.", i + 1, d.name, c.cost)
            };
            let builds = if builds.is_empty() { String::new() } else { format!(" ITS FACTORY PIXELS PRINT {}.", builds.join(", ")) };
            return Some(what + &builds);
        }
        if BTN_PAUSE.contains(m.0, m.1) {
            return Some("[SPACE] PAUSE OR RUN THE MATCH.".into());
        }
        if BTN_SPEED.contains(m.0, m.1) {
            return Some("[F] 1X, 2X OR 4X. THE FRAMES STAY AT 15 FPS; FAST SKIPS AHEAD.".into());
        }
        if BTN_LEAVE.contains(m.0, m.1) {
            return Some("GIVE UP THE MATCH AND GO BACK TO THE YARD.".into());
        }
        if MINI.contains(m.0, m.1) {
            return Some("THE WHOLE MAP. CLICK OR DRAG TO LOOK AROUND; FLYING YOUR KING BRINGS THE VIEW BACK. WHILE PLANTING, A CLICK PICKS THE ROCK.".into());
        }
        if (306..400).contains(&m.0) && (0..12).contains(&m.1) {
            return Some("BANK: THE PIXELS STORED IN YOUR KING'S NETWORK. YOUR MINERS HAUL TO THE NEAREST NODE; YOUR KING AND FACTORIES PRINT FROM THEIR OWN NETWORK.".into());
        }
        if (306..400).contains(&m.0) && (140..158).contains(&m.1) {
            return Some("WHAT YOUR KING IS PRINTING, PAID A PIXEL AT A TIME FROM ITS NETWORK.".into());
        }
        None
    }

    pub(super) fn draw_match_help(&self, fb: &mut Framebuffer) {
        let r = Rect::new(10, 8, 380, 284);
        ui::panel(fb, r);
        fb.text("THE MATCH", r.x + 10, r.y + 8, ACCENT, Some(OUTLINE), 2);
        fb.text("ROUND 1", r.x + r.w - 52, r.y + 12, GOLD, None, 1);
        let mut y = r.y + 34;
        for (head, text) in MATCH_HELP {
            fb.text(head, r.x + 10, y, GOLD, None, 1);
            let lines = wrap(text, 49);
            for (i, l) in lines.iter().enumerate() {
                fb.text(l, r.x + 64, y + i as i32 * 9, TEXT, None, 1);
            }
            y += lines.len() as i32 * 9 + 6;
        }
        fb.text_centered("CLICK OR PRESS ANY KEY", r.x + r.w / 2, r.y + r.h - 18, ACCENT, None, 1);
    }
}

impl Play {
    fn look_at_king(&mut self) {
        let e = self.arena.eco.as_ref().unwrap();
        let k = &self.arena.ships[e.king[0]];
        self.cam = ((k.x - FIELD.w / 2).clamp(0, MW - FIELD.w), (k.y - FIELD.h / 2).clamp(0, MH - FIELD.h));
    }

    fn cam_from_map(&mut self, x: i32, y: i32) {
        let (wx, wy) = ((x - MINI.x - 1) * MINI_SCALE, (y - MINI.y - 1) * MINI_SCALE);
        self.follow = false;
        self.cam = ((wx - FIELD.w / 2).clamp(0, MW - FIELD.w), (wy - FIELD.h / 2).clamp(0, MH - FIELD.h));
    }

    fn draw_minimap(&self, fb: &mut Framebuffer) {
        let a = &self.arena;
        let r = MINI;
        let inner = scene::minimap_ground(fb, r, a, MINI_SCALE);
        let (ox, oy) = (inner.x, inner.y);
        for s in a.ships.iter().filter(|s| s.alive && s.warp == 0) {
            let n = if s.node || s.role == Role::King { 2 } else { 1 };
            let (x, y) = (ox + s.x / MINI_SCALE, oy + s.y / MINI_SCALE);
            for k in 0..n * n {
                pc(fb, inner, x + k % n, y + k / n, if s.role == Role::King { WHITE } else { team_col(s.team, 2) });
            }
        }
        scene::minimap_view(fb, inner, FIELD, self.cam, MINI_SCALE);
    }
}

fn reach(s: &Ship) -> i32 {
    if s.role == Role::King {
        KING_RANGE
    } else {
        NODE_RANGE
    }
}

/// One line of how both sides stand, for the log and the headless runs.
pub fn state_line(a: &Arena) -> String {
    let e = a.eco.as_ref().unwrap();
    let side = |t: usize| {
        let ships = a.ships.iter().filter(|s| s.alive && s.team == t && s.role == Role::Ship && s.grow.is_none() && !s.node).count();
        let factories = a.ships.iter().filter(|s| s.alive && s.team == t && s.role == Role::Ship && s.node).count();
        let tl = e.tally[t];
        format!(
            "{} bank {} (peak {}), {} factories, {} ships; planted {}, printed {}, summoned {}, mined {}, lost {} ships",
            team_name(t),
            a.bank(e.king[t]),
            tl.bank_peak,
            factories,
            ships,
            tl.planted,
            tl.printed,
            tl.summoned,
            a.score[t].mined,
            a.score[t].lost
        )
    };
    format!("{} | {}", side(0), side(1))
}

pub fn clock(tick: u32) -> String {
    let t = tick / 15;
    format!("{}:{:02}", t / 60, t % 60)
}

/// The inspector's lines for a ship.
fn ship_lines(a: &Arena, i: usize) -> Vec<String> {
    let s = &a.ships[i];
    let e = a.eco.as_ref().unwrap();
    let who = team_name(s.team);
    let status = if s.warp > 0 {
        "ARRIVING BY HYPERSPACE".to_string()
    } else if s.role == Role::King {
        "THE KING".to_string()
    } else if s.maker.is_some() {
        format!("BEING PRINTED: {} OF {} PX", s.px_now(), s.px_full())
    } else if s.grow.as_ref().is_some_and(|g| g.eats) {
        format!("GROWING OUT OF ITS ROCK: {} OF {} PX", s.px_now(), s.px_full())
    } else {
        format!("{} - {}", s.order.name(), s.doing.name())
    };
    let mut lines = vec![format!("{who} {} - {status}", s.name)];
    let mut parts = vec![format!("{}/{} PX", s.px_now(), s.px_full())];
    if s.node {
        parts.push(format!("STORE {} NET {}", s.store, a.bank(i)));
    }
    if s.st.factory > 0 {
        let next = match s.making {
            Making::Child(c) => format!("PRINTING {}", a.ships[c].name),
            Making::Pending(slot) => format!("NEXT {}", e.deck[s.team].get(slot).map_or("?", |d| d.name.as_str())),
            Making::None if s.grow.is_some() => "GROWING".to_string(),
            Making::None => "NOTHING IT CAN BUILD".to_string(),
        };
        parts.push(next);
    }
    if s.st.power_pct() < 100 {
        parts.push(format!("POWER {}%", s.st.power_pct()));
    }
    if s.carry > 0 {
        parts.push(format!("CARRYING {}", s.carry));
    }
    lines.push(parts.join("  "));
    lines
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_match_rules_fit_their_panel() {
        // Headings start 34 px down a 284 px panel; the last line of the panel is 18 px from its foot.
        let lines: i32 = MATCH_HELP.iter().map(|(_, t)| wrap(t, 49).len() as i32 * 9 + 6).sum();
        assert!(34 + lines <= 284 - 22, "the rules run {} px too far", 34 + lines - (284 - 22));
    }
}
