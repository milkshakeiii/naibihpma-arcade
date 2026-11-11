//! The match screen: fly your king, have it print your deck, plant seeds on rocks, give your
//! designs their standing orders, and watch your factories grow and print. The AI plays the other
//! side with the starter deck.

use super::design::*;
use super::scene;
use super::sim::*;
use super::{draw_icon, pc, team_name, Pg, Screen, FIELD};
use crate::app::Key;
use crate::framebuffer::{wrap, Framebuffer};
use crate::geom::{cheb, dist2};
use crate::palette::*;
use crate::ui::{self, Rect};

/// The whole map at exactly 1/16 scale.
const MINI_SCALE: i32 = 16;
const MINI: Rect = Rect::new(306, 200, MW / MINI_SCALE + 2, MH / MINI_SCALE + 2);
/// The tactical map: the whole map at exactly 1/5 scale, in the field.
const TAC_SCALE: i32 = 5;
const TAC: Rect = Rect::new((FIELD.w - MW / TAC_SCALE) / 2, (FIELD.h - MH / TAC_SCALE) / 2 + 4, MW / TAC_SCALE, MH / TAC_SCALE);
const BTN_CANCEL: Rect = Rect::new(385, 148, 11, 9);
const BTN_PAUSE: Rect = Rect::new(306, 159, 44, 14);
const BTN_SPEED: Rect = Rect::new(352, 159, 44, 14);
const BTN_TAC: Rect = Rect::new(306, 175, 44, 14);
const BTN_LEAVE: Rect = Rect::new(352, 175, 44, 14);
const VIEW_NOTE_Y: i32 = 191;
const OVER: Rect = Rect::new(52, 100, 200, 88);
const OVER_BTNS: [Rect; 2] = [Rect::new(66, 162, 80, 16), Rect::new(158, 162, 80, 16)];
const MENU: Rect = Rect::new(80, 80, 240, 130);
const MENU_BTNS: [Rect; 3] = [Rect::new(92, 168, 70, 18), Rect::new(166, 168, 76, 18), Rect::new(246, 168, 62, 18)];

fn card_rect(i: usize) -> Rect {
    Rect::new(306, 14 + i as i32 * 18, 90, 17)
}
/// The king's queue, under the cards: its first jobs as icons, the one being printed first, each
/// with room under it for the loop sign. Five fit; a longer queue shows four and a count of the
/// rest.
const QUEUE: Rect = Rect::new(306, 140, 90, 18);
fn job_rect(k: usize) -> Rect {
    Rect::new(307 + k as i32 * 15, 141, 14, 13)
}
fn jobs_shown(queued: usize) -> usize {
    if queued > 5 {
        4
    } else {
        queued
    }
}

/// The loop sign, 7 by 3: what the king keeps printing wears it, on its card and in the queue.
fn loop_mark(fb: &mut Framebuffer, x: i32, y: i32, col: u8) {
    for (dy, row) in [".##.##.", "#..#..#", ".##.##."].iter().enumerate() {
        for (dx, c) in row.bytes().enumerate() {
            if c == b'#' {
                fb.pset(x + dx as i32, y + dy as i32, col);
            }
        }
    }
}

/// The order tag on a card: a click there switches the design's standing order.
fn tag_rect(i: usize) -> Rect {
    let r = card_rect(i);
    Rect::new(r.x + 67, r.y + 8, 21, 8)
}

fn order_col(o: Order) -> u8 {
    match o {
        Order::Hunt => BAD,
        Order::Guard => ACCENT_HI,
        Order::Escort => GOLD,
        Order::Harvest => GOOD,
        Order::Hold => TEXT_DIM,
    }
}

/// The match's rules panel, by heading.
const MATCH_HELP: [(&str, &str); 9] = [
    ("GOAL", "KILL THE ENEMY KING. LOSE YOURS AND YOU LOSE. A KING IS SLOW WHILE ITS SIDE HAS UNDER 40 PX LEFT."),
    ("KING", "WASD OR ARROWS FLY IT. 1-7 OR A CARD: IT PRINTS THAT SHIP, A PIXEL AT A TIME. RIGHT-CLICK OR R: IT KEEPS PRINTING IT. BACKSPACE: IT STOPS."),
    ("SEEDS", "A SEEDED CARD COSTS ALL IT GROWS INTO, PAID AT ONCE. CLICK A ROCK: ITS SEED FLIES THERE AND GROWS IT OUT OF THE ROCK, PIXEL BY PIXEL."),
    ("FACTORY", "IT PRINTS ITS BUILD ORDER ROUND AND ROUND, A PIXEL AT A TIME, WAITING WHEN THE BANK IS EMPTY."),
    ("BANK", "MINERS HAUL TO THE NEAREST NODE (KING OR FACTORY). LINKED NODES SHARE A BANK; NODES CUT OFF FROM YOUR KING KEEP THEIR OWN."),
    ("ORDERS", "EVERY DESIGN THAT FLIES HAS A STANDING ORDER, ON ITS CARD: HUNT THE NEAREST ENEMY, GUARD WHERE IT WAS BUILT, OR ESCORT YOUR KING. CLICK THE TAG OR SHIFT+NUMBER: EVERY SHIP OF IT OBEYS AT ONCE."),
    ("VIEW", "HOLD THE MOUSE ON THE SMALL MAP TO LOOK AROUND; STEERING BRINGS THE VIEW BACK. TAB: WHOLE MAP."),
    ("SOLID", "WHAT'S PLANTED IS SOLID TO ENEMY SHIPS. SO IS AN ENEMY SHIP BIGGER THAN YOURS."),
    ("KEYS", "SPACE PAUSE  F SPEED  ESC CANCEL/QUIT  F1 HELP"),
];

/// A match in progress, and how you're looking at it.
pub struct Play {
    pub arena: Arena,
    pub cam: (i32, i32),
    pub follow: bool,
    /// The field shows the whole map at quarter scale.
    pub tactical: bool,
    /// A seeded card picked, waiting for you to click the rock its seed should go to.
    pub planting: Option<usize>,
    pub running: bool,
    pub speed: u32,
    pub starter: bool,
    pub seed: u64,
    /// A drag that moves the view is under way, on the whole map (true) or the small one.
    map_drag: Option<bool>,
    logged_over: bool,
    next_log: u32,
    /// Whether each king was slowed at the last look, for the log.
    slowed: [bool; 2],
}

impl Pg {
    // ----- Starting and leaving -----

    /// Starts a match with your seven designs as your deck, or the starter deck. The AI always
    /// plays the starter deck.
    pub fn start_match(&mut self, starter: bool, seed: u64) {
        let mine = if starter { starter_deck() } else { self.lib.clone() };
        let arena = Arena::for_match(seed, [mine, starter_deck()], [false, true]);
        let mut p = Play { arena, cam: (0, 0), follow: true, tactical: false, planting: None, running: true, speed: 1, starter, seed, map_drag: None, logged_over: false, next_log: 15 * 60, slowed: [false; 2] };
        p.look_at_king();
        self.play = Some(p);
        self.menu = false;
        self.screen = Screen::Match;
        // The rules first: Toy 1's were hard to work out from play alone.
        self.help = true;
        let deck = if starter { "STARTER DECK" } else { "YOUR DECK" };
        self.log(&format!("MATCH START round 2, seed {seed}, {deck}"));
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
        if p.arena.eco.as_ref().is_some_and(|e| e.bot[0] == Bot::None) {
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
        for t in 0..2 {
            let now = p.arena.over.is_none() && p.arena.king_slowed(t);
            if now != p.slowed[t] && p.arena.over.is_none() {
                p.slowed[t] = now;
                lines.push(format!("KING {} {} at {}, {} px standing", team_name(t), if now { "SLOWED" } else { "FREE" }, clock(p.arena.tick), p.arena.standing(t)));
            }
        }
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
            Key::ShiftNum(n) if (1..=7).contains(&n) => self.next_order(n as usize - 1),
            Key::Space | Key::Letter('P') => {
                if p.arena.over.is_none() {
                    p.running = !p.running;
                }
            }
            Key::Letter('F') => p.speed = if p.speed >= 4 { 1 } else { p.speed * 2 },
            Key::Tab => self.toggle_tactical(),
            Key::Back => self.drop_job(0),
            Key::Letter('R') => {
                // What the king is printing repeats, or stops repeating.
                if let Some(slot) = p.arena.eco.as_ref().and_then(|e| e.queue[0].first()).map(|j| j.slot) {
                    self.repeat(slot);
                }
            }
            _ => {}
        }
    }

    fn toggle_tactical(&mut self) {
        let Some(p) = self.play.as_mut() else { return };
        p.tactical = !p.tactical;
        let line = format!("TACTICAL {} at {}", if p.tactical { "ON" } else { "OFF" }, clock(p.arena.tick));
        self.log(&line);
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
            let bank = p.arena.bank(p.arena.eco.as_ref().unwrap().king[0]);
            if bank < card.price() {
                let s = format!("IT COSTS {} AND YOUR BANK HOLDS {bank}", card.price());
                self.say(&s);
                return;
            }
            p.planting = Some(i);
            self.say("CLICK A ROCK FOR THE SEED. RIGHT-CLICK CANCELS");
        } else {
            match p.arena.can_print(0, i, None) {
                Ok(_) => p.arena.command(Cmd::Print { team: 0, slot: i, target: None }),
                Err(why) => self.say(why),
            }
        }
    }

    /// A card's order tag: the design's next standing order.
    fn next_order(&mut self, i: usize) {
        let Some(p) = self.play.as_mut() else { return };
        let Some(e) = p.arena.eco.as_ref() else { return };
        let Some(card) = e.cards[0].get(i).copied().filter(|c| c.takes_orders()) else { return };
        let (now, choices) = (e.orders[0][i], Order::in_match(card.order));
        let next = choices[(choices.iter().position(|&o| o == now).map_or(0, |k| k + 1)) % choices.len()];
        let line = format!("ORDER {} {} at {}", e.deck[0][i].name, next.name(), clock(p.arena.tick));
        p.arena.command(Cmd::Order { team: 0, slot: i, order: next });
        self.log(&line);
    }

    /// Right-click on a card, or on one of its jobs in the queue: the king keeps printing it (or
    /// stops).
    fn repeat(&mut self, i: usize) {
        let Some(p) = self.play.as_mut() else { return };
        let Some(e) = p.arena.eco.as_ref() else { return };
        let Some(card) = e.cards[0].get(i).copied() else { return };
        if !card.ok || card.seeded {
            self.say("ONLY SHIPS YOUR KING PRINTS CAN REPEAT");
            return;
        }
        let on = e.queue[0].iter().any(|j| j.slot == i && j.repeat);
        let line = format!("REPEAT {} {} at {}", e.deck[0][i].name, if on { "OFF" } else { "ON" }, clock(p.arena.tick));
        p.arena.command(Cmd::Repeat { team: 0, slot: i });
        self.log(&line);
    }

    /// Drops a job from the king's queue: 0 is the one it's printing.
    fn drop_job(&mut self, at: usize) {
        let Some(p) = self.play.as_mut() else { return };
        let Some(e) = p.arena.eco.as_ref() else { return };
        let Some(j) = e.queue[0].get(at).copied() else { return };
        let line = format!("DROP {} at {}", e.deck[0].get(j.slot).map_or("?", |d| d.name.as_str()), clock(p.arena.tick));
        p.arena.command(Cmd::Cancel { team: 0, at, slot: j.slot });
        self.log(&line);
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

    /// Plants the seed of `slot` on the rock nearest a world point, if there is one within `slack`.
    fn plant(&mut self, slot: usize, at: (i32, i32), slack: i32) {
        let Some(p) = self.play.as_mut() else { return };
        match p.arena.rock_near(at, slack) {
            Some(target) => {
                p.planting = None;
                match p.arena.can_print(0, slot, Some(target)) {
                    Ok(cost) => {
                        let line = format!("PLANT {} for {cost} at {}", p.arena.eco.as_ref().unwrap().deck[0][slot].name, clock(p.arena.tick));
                        p.arena.command(Cmd::Print { team: 0, slot, target: Some(target) });
                        self.log(&line);
                    }
                    Err(why) => self.say(why),
                }
            }
            None => self.say("THAT'S NOT A ROCK: CLICK ON ONE"),
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
            if p.planting.is_some() {
                p.planting = None;
            } else if let Some(i) = (0..7).find(|&i| card_rect(i).contains(x, y)) {
                self.repeat(i);
            } else if let Some(slot) = p.job_at(x, y).map(|(_, j)| j.slot) {
                self.repeat(slot);
            }
            return;
        }
        if FIELD.contains(x, y) {
            match (p.planting, p.world_point(x, y)) {
                (Some(slot), Some(at)) => {
                    let slack = if p.tactical { 2 * TAC_SCALE } else { 6 };
                    self.plant(slot, at, slack);
                }
                // A click on the tactical map moves the view, as on the small map.
                (None, Some(at)) if p.tactical => {
                    p.map_drag = Some(true);
                    p.look_at(at);
                }
                _ => {}
            }
            return;
        }
        if let Some(i) = (0..7).find(|&i| tag_rect(i).contains(x, y) && p.arena.eco.as_ref().is_some_and(|e| e.cards[0].get(i).is_some_and(|c| c.takes_orders()))) {
            self.next_order(i);
        } else if let Some(i) = (0..7).find(|&i| card_rect(i).contains(x, y)) {
            self.card(i);
        } else if MINI.contains(x, y) {
            match (p.planting, p.world_point(x, y)) {
                // Planting from the small map: the rock nearest the click.
                (Some(slot), Some(at)) => self.plant(slot, at, MINI_SCALE),
                (None, Some(at)) => {
                    p.map_drag = Some(false);
                    p.look_at(at);
                }
                _ => {}
            }
        } else if let Some((k, _)) = p.job_at(x, y) {
            self.drop_job(k);
        } else if BTN_CANCEL.contains(x, y) {
            self.drop_job(0);
        } else if BTN_PAUSE.contains(x, y) {
            if p.arena.over.is_none() {
                p.running = !p.running;
            }
        } else if BTN_SPEED.contains(x, y) {
            p.speed = if p.speed >= 4 { 1 } else { p.speed * 2 };
        } else if BTN_TAC.contains(x, y) {
            self.toggle_tactical();
        } else if BTN_LEAVE.contains(x, y) {
            self.leave_match();
        }
    }

    pub(super) fn drag_match(&mut self, x: i32, y: i32) {
        if let Some(p) = self.play.as_mut() {
            if let Some(whole) = p.map_drag {
                // The drag stays on the map it began on, however far the pointer strays.
                let r = if whole && p.tactical { TAC } else { Rect::new(MINI.x + 1, MINI.y + 1, MINI.w - 2, MINI.h - 2) };
                if let Some(at) = p.world_point(x.clamp(r.x, r.x + r.w - 1), y.clamp(r.y, r.y + r.h - 1)) {
                    p.look_at(at);
                }
            }
        }
    }

    pub(super) fn release_match(&mut self) {
        if let Some(p) = self.play.as_mut() {
            p.map_drag = None;
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
        let f = FIELD;
        // The world point under the pointer, on whichever view the field shows.
        let at = if f.contains(m.0, m.1) { p.world_point(m.0, m.1) } else { None };
        let hovered = match at {
            Some((wx, wy)) if p.planting.is_none() && !p.tactical => a.ship_at(wx, wy),
            Some((wx, wy)) if p.planting.is_none() => (0..a.ships.len()).filter(|&i| a.ships[i].alive && a.ships[i].warp == 0 && cheb(a.ships[i].x, a.ships[i].y, wx, wy) <= a.ships[i].half() + 2 * TAC_SCALE).min_by_key(|&i| dist2(a.ships[i].x, a.ships[i].y, wx, wy)),
            _ => None,
        };
        // Where a world box lands on the field.
        let on_field = |l: i32, t: i32, r: i32, b: i32| {
            if p.tactical {
                (TAC.x + l / TAC_SCALE, TAC.y + t / TAC_SCALE, (r - l) / TAC_SCALE + 1, (b - t) / TAC_SCALE + 1)
            } else {
                (l - p.cam.0, t - p.cam.1, r - l + 1, b - t + 1)
            }
        };
        if p.tactical {
            p.draw_tactical(fb);
        } else {
            p.draw_field(fb);
        }
        if let Some(i) = hovered {
            let (l, t, r, b) = a.ships[i].bbox();
            let (x, y, w, h) = on_field(l, t, r, b);
            fb.corners(x - 2, y - 2, w + 4, h + 4, 2, WHITE);
        }
        // Where a world point lands on the field, and what clips a line drawn there.
        let spot = |(x, y): (i32, i32)| if p.tactical { (TAC.x + x / TAC_SCALE, TAC.y + y / TAC_SCALE) } else { (x - p.cam.0, y - p.cam.1) };
        let clip = if p.tactical { TAC } else { f };
        // Planting: the rock under the pointer, whether the design fits on it, and what it would
        // link to there.
        let mut bar: Option<Vec<String>> = None;
        if let (Some(slot), Some(at)) = (p.planting, at) {
            let need = e.cards[0][slot].count;
            match a.rock_near(at, if p.tactical { 2 * TAC_SCALE } else { 6 }) {
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
                    let (x, y, w, h) = on_field(l, t, r, b);
                    fb.corners(x - 2, y - 2, w + 4, h + 4, 3, if fits && here.is_none() { GOOD } else { GOLD });
                    let fit = match here {
                        Some(s) => format!("{} {} IS PLANTED HERE ALREADY: THEY'LL WALL EACH OTHER IN.", team_name(s.team), s.name),
                        None if fits => "IT FITS.".to_string(),
                        None => format!("IT'LL STAY {}% GROWN.", rock.len() as i32 * 100 / need.max(1)),
                    };
                    // A line to every node it would link to, green if that leads to the king's
                    // bank; with nothing in reach, a dim one to the nearest and how far short it is.
                    let link = match a.plant_links(0, slot, px) {
                        Some(l) => {
                            let from = spot(l.at);
                            let mut line = |to: (i32, i32), col: u8| {
                                let to = spot(to);
                                crate::render::dotted(fb, clip, from.0, from.1, to.0, to.1, 2, col);
                            };
                            for &to in &l.to {
                                line(to, if l.king { GOOD } else { GOLD });
                            }
                            match (l.king, l.gap) {
                                (true, _) => " IT JOINS YOUR KING'S BANK.".to_string(),
                                (false, Some((to, short))) => {
                                    line(to, TEXT_DIM);
                                    format!(" {short} PX TOO FAR TO LINK: IT'LL KEEP ITS OWN BANK.")
                                }
                                (false, None) => " IT LINKS, BUT NOT TO YOUR KING'S BANK.".to_string(),
                            }
                        }
                        None => String::new(),
                    };
                    bar = Some(wrap(&format!("PLANT {} HERE FOR {} PX: ROCK {} PX, NEEDS {}. {fit}{link} RIGHT-CLICK OR ESC CANCELS.", e.deck[0][slot].name, e.cards[0][slot].price(), rock.len(), need), 49));
                }
                None => bar = Some(vec![format!("PLANT {}: CLICK A ROCK", e.deck[0][slot].name), "RIGHT-CLICK OR ESC CANCELS".into()]),
            }
        }
        fb.text(&clock(a.tick), 4, 3, TEXT_DIM, Some(OUTLINE), 1);
        let mut notes: Vec<(String, u8)> = Vec::new();
        if e.split_at[0] > 0 && a.tick < e.split_at[0] + 15 * 4 {
            notes.push(("YOUR NETWORK SPLIT".into(), GOLD));
        }
        if a.over.is_none() {
            if a.king_slowed(0) {
                notes.push((format!("YOUR KING IS SLOWED: UNDER {KING_NEEDS} PX LEFT STANDING"), BAD));
            }
            if a.king_slowed(1) {
                notes.push(("THE ENEMY KING IS SLOWED".into(), GOOD));
            }
        }
        for (i, (text, col)) in notes.iter().enumerate() {
            fb.text_centered(text, f.w / 2, 14 + i as i32 * 9, *col, Some(OUTLINE), 1);
        }
        if bar.is_none() {
            bar = if let Some(i) = hovered {
                Some(ship_lines(a, i))
            } else if let Some(at) = at {
                a.rock_near(at, if p.tactical { TAC_SCALE } else { 2 }).map(|px| vec![format!("ROCK: {} PX LEFT. ANYONE CAN MINE IT, AND A SEED CAN GROW A FACTORY OUT OF IT.", a.rock_blob(px).len())])
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
        let bank = a.bank(k);
        fb.text(&format!("BANK {bank}"), 308, 4, GOLD, None, 1);
        // What nodes cut off from the king hold: yours, but not yours to plant with.
        let cut = cut_off(a);
        if cut > 0 {
            let s = format!("+{cut}");
            fb.text(&s, 396 - Framebuffer::text_width(&s, 1), 4, TEXT_DIM, None, 1);
        }
        for i in 0..7 {
            let r = card_rect(i);
            let hov = r.contains(m.0, m.1);
            let busy = e.queue[0].first().is_some_and(|j| j.slot == i);
            let repeats = e.queue[0].iter().any(|j| j.slot == i && j.repeat);
            let picked = p.planting == Some(i);
            if picked || busy {
                fb.rect_bevel(r.x, r.y, r.w, r.h, PANEL_DEEP, PANEL_LO, PANEL_HI);
                fb.rect_outline(r.x + 1, r.y + 1, r.w - 2, r.h - 2, if picked { GOLD } else { ACCENT });
            } else if hov {
                fb.rect_bevel(r.x, r.y, r.w, r.h, PANEL_HI, WHITE, PANEL);
            } else {
                ui::panel(fb, r);
            }
            // A card the king keeps printing wears a blue frame, as in Toy 1, and the loop sign
            // its job wears in the queue.
            if repeats {
                fb.rect_outline(r.x, r.y, r.w, r.h, ACCENT_HI);
                loop_mark(fb, r.x + 58, r.y + 11, ACCENT_HI);
            }
            let Some(d) = e.deck[0].get(i) else { continue };
            let card = e.cards[0][i];
            let lit = hov && !(busy || picked);
            // A seeded card you can't pay for yet is greyed, with its price in red.
            let short = card.ok && card.seeded && bank < card.price();
            draw_icon(fb, d, Rect::new(r.x + 2, r.y + 1, 14, 14), 0);
            let name: String = d.name.chars().take(10).collect();
            fb.text(&name, r.x + 19, r.y + 1, if !card.ok { BAD } else if short { TEXT_DIM } else if lit { PANEL_DEEP } else { TEXT }, None, 1);
            let what = if card.seeded { format!("SEED {}", card.price()) } else { format!("{} PX", card.price()) };
            fb.text(&what, r.x + 19, r.y + 9, if short { BAD } else if lit { PANEL_LO } else if card.seeded { FACTORY } else { TEXT_DIM }, None, 1);
            fb.text(&format!("{}", i + 1), r.x + r.w - 8, r.y + 1, TEXT_FAINT, None, 1);
            if card.takes_orders() {
                let t = tag_rect(i);
                let o = e.orders[0][i];
                fb.rect_fill(t.x, t.y, t.w, t.h, if t.contains(m.0, m.1) { OUTLINE } else { PANEL_DEEP });
                fb.text(o.tag(), t.x + 2, t.y + 1, order_col(o), None, 1);
            }
        }
        // The king's queue: what it's printing first, then what waits, each as its card's icon.
        // The bar is how much of the first is paid.
        let q = &e.queue[0];
        match q.first() {
            Some(first) => {
                let shown = jobs_shown(q.len());
                for (k, j) in q.iter().take(shown).enumerate() {
                    let r = job_rect(k);
                    // The icon in the top of its box, and under it the loop sign if it repeats.
                    fb.rect_fill(r.x, r.y, r.w, r.h, SPACE);
                    if let Some(d) = e.deck[0].get(j.slot) {
                        draw_icon(fb, d, Rect::new(r.x, r.y, r.w, r.h - 3), 0);
                    }
                    if j.repeat {
                        loop_mark(fb, r.x + 3, r.y + r.h - 4, ACCENT_HI);
                    }
                    fb.rect_outline(r.x, r.y, r.w, r.h, if r.contains(m.0, m.1) { WHITE } else if k == 0 { ACCENT } else { PANEL_LO });
                }
                if q.len() > shown {
                    let more = format!("+{}", q.len() - shown);
                    fb.text(&more, 396 - Framebuffer::text_width(&more, 1), 141, TEXT_DIM, None, 1);
                }
                fb.rect_fill(308, 155, 74, 3, PANEL_DEEP);
                fb.rect_fill(308, 155, 74 * first.paid / first.cost.max(1), 3, if first.repeat { ACCENT_HI } else { ACCENT });
                ui::button(fb, BTN_CANCEL, "X", BTN_CANCEL.contains(m.0, m.1), false);
            }
            None => fb.text("KING IDLE", 308, 145, TEXT_FAINT, None, 1),
        }
        ui::button(fb, BTN_PAUSE, if p.running { "PAUSE" } else { "RUN" }, BTN_PAUSE.contains(m.0, m.1), !p.running);
        ui::button(fb, BTN_SPEED, &format!("{}X", p.speed), BTN_SPEED.contains(m.0, m.1), p.speed > 1);
        ui::button(fb, BTN_TAC, "MAP", BTN_TAC.contains(m.0, m.1), p.tactical);
        ui::button(fb, BTN_LEAVE, "LEAVE", BTN_LEAVE.contains(m.0, m.1), false);
        // How the view works, where the eye goes to look at the map.
        if p.follow {
            fb.text("HOLD TO LOOK", 308, VIEW_NOTE_Y, TEXT_DIM, None, 1);
        } else {
            fb.text("STEER TO RETURN", 306, VIEW_NOTE_Y, GOLD, None, 1);
        }
        p.draw_minimap(fb);
        for t in 0..2 {
            let y = 274 + t as i32 * 10;
            let ships = a.ships.iter().filter(|s| s.alive && s.team == t && s.role == Role::Ship && s.grow.is_none() && !s.node).count();
            let factories = a.ships.iter().filter(|s| s.alive && s.team == t && s.role == Role::Ship && s.node).count();
            fb.text(team_name(t), 308, y, team_col(t, 2), None, 1);
            let count = format!("{factories}F {ships}S");
            fb.text(&count, 340, y, TEXT, None, 1);
            if a.king_slowed(t) && 340 + Framebuffer::text_width(&count, 1) <= 371 {
                fb.text("SLOW", 373, y, if t == 0 { BAD } else { GOOD }, None, 1);
            }
        }
    }

    fn match_tip(&self) -> Option<String> {
        let m = self.mouse;
        let p = self.play.as_ref()?;
        let e = p.arena.eco.as_ref()?;
        if let Some(i) = (0..7).find(|&i| card_rect(i).contains(m.0, m.1)) {
            let d = e.deck[0].get(i)?;
            let c = e.cards[0][i];
            if c.takes_orders() && tag_rect(i).contains(m.0, m.1) {
                let (now, choices) = (e.orders[0][i], Order::in_match(c.order));
                let next = choices[(choices.iter().position(|&o| o == now).map_or(0, |k| k + 1)) % choices.len()];
                return Some(format!("[SHIFT+{}] EVERY {} IS ON {} CLICK FOR {}.", i + 1, d.name, match_blurb(now), next.name()));
            }
            let builds: Vec<String> = d.build.iter().filter_map(|&b| e.deck[0].get(b)).map(|o| o.name.clone()).collect();
            let what = if c.seeded {
                format!("[{}] {}: {} PX, PAID IN FULL WHEN YOU PICK ITS ROCK. ITS SEED FLIES THERE AND GROWS IT OUT OF THE ROCK, WHICH HAS TO HOLD {} PX.", i + 1, d.name, c.price(), c.count)
            } else {
                format!("[{}] {}: YOUR KING PRINTS IT FOR {} PX, A PIXEL AT A TIME. RIGHT-CLICK: IT KEEPS PRINTING THEM.", i + 1, d.name, c.cost)
            };
            let builds = if builds.is_empty() { String::new() } else { format!(" ITS FACTORY PIXELS PRINT {}.", builds.join(", ")) };
            return Some(what + &builds);
        }
        if let Some((k, j)) = p.job_at(m.0, m.1) {
            let name = e.deck[0].get(j.slot).map_or("?", |d| d.name.as_str());
            let (state, key) = if k == 0 { (format!("YOUR KING IS PRINTING IT: {} OF {} PX PAID", j.paid, j.cost), " OR [R]") } else { (format!("NUMBER {} IN YOUR KING'S QUEUE", k + 1), "") };
            let repeats = if j.repeat { " IT REPEATS: WHEN ONE IS DONE, ANOTHER GOES TO THE BACK OF THE QUEUE." } else { "" };
            return Some(format!("{name}: {state}.{repeats} CLICK DROPS IT; WHAT IT HAD PAID COMES BACK. RIGHT-CLICK{key}: REPEAT ON OR OFF."));
        }
        if BTN_CANCEL.contains(m.0, m.1) && !e.queue[0].is_empty() {
            return Some("[BACKSPACE] DROP WHAT YOUR KING IS PRINTING. YOU GET BACK WHAT IT HAD PAID.".into());
        }
        if BTN_PAUSE.contains(m.0, m.1) {
            return Some("[SPACE] PAUSE OR RUN THE MATCH.".into());
        }
        if BTN_SPEED.contains(m.0, m.1) {
            return Some("[F] 1X, 2X OR 4X. THE FRAMES STAY AT 15 FPS; FAST SKIPS AHEAD.".into());
        }
        if BTN_TAC.contains(m.0, m.1) {
            return Some("[TAB] THE WHOLE MAP IN THE FIELD. YOUR KING STILL FLIES, AND YOU CAN PLANT AND GIVE ORDERS FROM IT.".into());
        }
        if BTN_LEAVE.contains(m.0, m.1) {
            return Some("GIVE UP THE MATCH AND GO BACK TO THE YARD.".into());
        }
        if MINI.contains(m.0, m.1) || (306..400).contains(&m.0) && (VIEW_NOTE_Y - 1..VIEW_NOTE_Y + 8).contains(&m.1) {
            return Some("THE WHOLE MAP. HOLD THE MOUSE BUTTON ON IT TO LOOK ELSEWHERE; FLYING YOUR KING BRINGS THE VIEW BACK. WHILE PLANTING, A CLICK PICKS THE ROCK.".into());
        }
        if (306..400).contains(&m.0) && (0..12).contains(&m.1) {
            let cut = cut_off(&p.arena);
            let more = if cut > 0 { format!(" +{cut}: HELD BY NODES OF YOURS CUT OFF FROM YOUR KING. FLY NEAR THEM, OR PLANT A LINK, TO SPEND IT.") } else { String::new() };
            return Some(format!("BANK: THE PIXELS STORED IN YOUR KING'S NETWORK. YOUR KING MAKES 1 EVERY {} S; MINERS HAUL TO THE NEAREST NODE.{more}", secs(KING_MAKES_EVERY as i32)));
        }
        if QUEUE.contains(m.0, m.1) {
            return Some("YOUR KING'S QUEUE: IT PRINTS THE FIRST, PAID A PIXEL AT A TIME FROM ITS NETWORK, AND THE BAR IS HOW FAR THAT HAS GOT. THE LOOP SIGN MARKS ONE IT KEEPS PRINTING.".into());
        }
        None
    }

    pub(super) fn draw_match_help(&self, fb: &mut Framebuffer) {
        let r = Rect::new(10, 8, 380, 284);
        ui::panel(fb, r);
        fb.text("THE MATCH", r.x + 10, r.y + 8, ACCENT, Some(OUTLINE), 2);
        fb.text("ROUND 2", r.x + r.w - 52, r.y + 12, GOLD, None, 1);
        let mut y = r.y + 32;
        for (head, text) in MATCH_HELP {
            fb.text(head, r.x + 10, y, GOLD, None, 1);
            let lines = wrap(text, 49);
            for (i, l) in lines.iter().enumerate() {
                fb.text(l, r.x + 64, y + i as i32 * 9, TEXT, None, 1);
            }
            y += lines.len() as i32 * 9 + 4;
        }
        fb.text_centered("CLICK OR PRESS ANY KEY", r.x + r.w / 2, r.y + r.h - 14, ACCENT, None, 1);
    }
}

/// An order's blurb as it reads in a match: without what it does in the arena.
fn match_blurb(o: Order) -> String {
    o.blurb().replace("IN A MATCH, ", "").replace(" IN THE ARENA IT HUNTS.", "")
}

/// Pixels held by a side's nodes that aren't in its king's network.
fn cut_off(a: &Arena) -> i32 {
    let e = a.eco.as_ref().unwrap();
    let k = e.king[0];
    let net = e.net.get(k).copied().unwrap_or(usize::MAX);
    (0..a.ships.len()).filter(|&j| j != k && a.ships[j].alive && a.ships[j].team == 0 && e.net.get(j).is_some_and(|&n| n != net || n == usize::MAX)).map(|j| a.ships[j].store).sum()
}

impl Play {
    /// The job in the king's queue whose icon is under a screen point, and where it stands.
    fn job_at(&self, x: i32, y: i32) -> Option<(usize, Job)> {
        let q = &self.arena.eco.as_ref()?.queue[0];
        (0..jobs_shown(q.len())).find(|&k| job_rect(k).contains(x, y)).map(|k| (k, q[k]))
    }

    fn look_at_king(&mut self) {
        let e = self.arena.eco.as_ref().unwrap();
        let k = &self.arena.ships[e.king[0]];
        self.cam = ((k.x - FIELD.w / 2).clamp(0, MW - FIELD.w), (k.y - FIELD.h / 2).clamp(0, MH - FIELD.h));
    }

    /// Centres the view on a world point and leaves it there until the king is steered.
    fn look_at(&mut self, (wx, wy): (i32, i32)) {
        self.follow = false;
        self.cam = ((wx - FIELD.w / 2).clamp(0, MW - FIELD.w), (wy - FIELD.h / 2).clamp(0, MH - FIELD.h));
    }

    /// The world point under a screen point: on the field (or the tactical map, when that's
    /// showing) or on the small map.
    fn world_point(&self, x: i32, y: i32) -> Option<(i32, i32)> {
        let inner = Rect::new(MINI.x + 1, MINI.y + 1, MINI.w - 2, MINI.h - 2);
        if inner.contains(x, y) {
            Some(((x - inner.x) * MINI_SCALE + MINI_SCALE / 2, (y - inner.y) * MINI_SCALE + MINI_SCALE / 2))
        } else if !FIELD.contains(x, y) {
            None
        } else if !self.tactical {
            Some((x + self.cam.0, y + self.cam.1))
        } else if TAC.contains(x, y) {
            Some(((x - TAC.x) * TAC_SCALE + TAC_SCALE / 2, (y - TAC.y) * TAC_SCALE + TAC_SCALE / 2))
        } else {
            None
        }
    }

    /// The field: the map at full size, where the camera is.
    fn draw_field(&self, fb: &mut Framebuffer) {
        let a = &self.arena;
        let e = a.eco.as_ref().unwrap();
        let (f, (cx, cy)) = (FIELD, self.cam);
        scene::backdrop(fb, f, a, self.cam);
        // Network links: dotted lines between linked nodes.
        for (i, j) in links(a) {
            let (si, sj) = (&a.ships[i], &a.ships[j]);
            crate::render::dotted(fb, f, si.x - cx, si.y - cy, sj.x - cx, sj.y - cy, 4, team_col(si.team, 0));
        }
        scene::drift(fb, f, a, self.cam);
        // Our seeds' courses.
        for s in a.ships.iter().filter(|s| s.alive && s.team == 0 && s.role == Role::Seed) {
            if let Some((_, (tx, ty))) = s.seed_for {
                crate::render::dotted(fb, f, s.x - cx, s.y - cy, tx - cx, ty - cy, 3, TEXT_FAINT);
            }
        }
        for s in a.ships.iter().filter(|s| s.alive) {
            scene::ship(fb, f, s, self.cam);
        }
        // A slowed king's engines are out.
        for t in 0..2 {
            let k = &a.ships[e.king[t]];
            if k.alive && a.king_slowed(t) {
                for c in (0..k.hp.len()).filter(|&c| k.hp[c] > 0 && k.shape.cells[c] == Some(Part::Engine)) {
                    let (px, py) = k.cell_xy(c);
                    pc(fb, f, px - cx, py - cy, STEEL_DK);
                }
            }
        }
        scene::fire(fb, f, a, self.cam);
    }

    /// The tactical map: everything on the map at a fifth of its size. A ship is a block as big
    /// as it is (at least a pixel), so a capital reads as a capital and a swarm as a swarm.
    fn draw_tactical(&self, fb: &mut Framebuffer) {
        let a = &self.arena;
        let (f, r, sc) = (FIELD, TAC, TAC_SCALE);
        fb.rect_fill(f.x, f.y, f.w, f.h, PANEL_DEEP);
        fb.rect_checker(f.x, f.y, f.w, f.h, PANEL_LO, 0);
        fb.rect_bevel(r.x - 2, r.y - 2, r.w + 4, r.h + 4, SPACE, PANEL_LO, PANEL_HI);
        // Rock, shaded by how much of each block of the map is rock: three quarters, a third, less.
        for ty in 0..r.h {
            for tx in 0..r.w {
                let n = (0..sc * sc).filter(|k| a.rock_at(tx * sc + k % sc, ty * sc + k / sc)).count() as i32;
                if n > 0 {
                    fb.pset(r.x + tx, r.y + ty, if n * 4 >= sc * sc * 3 { ROCK + 2 } else if n * 3 >= sc * sc { ROCK + 1 } else { ROCK });
                }
            }
        }
        let at = |x: i32, y: i32| (r.x + x / sc, r.y + y / sc);
        for d in &a.debris {
            let (x, y) = at(d.x, d.y);
            pc(fb, r, x, y, LOOSE);
        }
        for (i, j) in links(a) {
            let ((x0, y0), (x1, y1)) = (at(a.ships[i].x, a.ships[i].y), at(a.ships[j].x, a.ships[j].y));
            crate::render::dotted(fb, r, x0, y0, x1, y1, 3, team_col(a.ships[i].team, 0));
        }
        for s in a.ships.iter().filter(|s| s.alive && s.team == 0 && s.role == Role::Seed) {
            if let Some((_, (tx, ty))) = s.seed_for {
                let ((x0, y0), (x1, y1)) = (at(s.x, s.y), at(tx, ty));
                crate::render::dotted(fb, r, x0, y0, x1, y1, 2, TEXT_FAINT);
            }
        }
        for s in a.ships.iter().filter(|s| s.alive && s.warp == 0) {
            let (l, t, rr, b) = s.bbox();
            let ((x, y), w, h) = (at(l, t), (rr - l + 1 + sc / 2) / sc, (b - t + 1 + sc / 2) / sc);
            let (w, h) = (w.max(1), h.max(1));
            let c = team_col(s.team, 2);
            if s.role == Role::King {
                let (kx, ky) = at(s.x, s.y);
                fb.rect_fill(kx - 3, ky - 3, 7, 7, OUTLINE);
                fb.rect_fill(kx - 2, ky - 2, 5, 5, team_col(s.team, 3));
                fb.pset(kx, ky, WHITE);
            } else if s.rooted {
                // Planted: outlined, and dithered until it has finished growing.
                fb.rect_fill(x - 1, y - 1, w + 2, h + 2, OUTLINE);
                if s.grow.is_some() {
                    fb.rect_fill(x, y, w, h, team_col(s.team, 0));
                    fb.rect_checker(x, y, w, h, c, 0);
                } else {
                    fb.rect_fill(x, y, w, h, c);
                }
            } else {
                fb.rect_fill(x, y, w, h, if s.role == Role::Seed { team_col(s.team, 3) } else { c });
            }
        }
        for sh in &a.shots {
            let (x, y) = at(sh.line.x, sh.line.y);
            pc(fb, r, x, y, team_col(sh.team, 3));
        }
        let (vx, vy) = at(self.cam.0, self.cam.1);
        fb.rect_outline(vx, vy, f.w / sc, f.h / sc, TEXT_DIM);
        fb.text("THE WHOLE MAP", r.x, r.y - 14, ACCENT, Some(OUTLINE), 1);
        fb.text("TAB RETURNS", r.x + r.w - 66, r.y - 14, TEXT_DIM, Some(OUTLINE), 1);
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

/// Every pair of linked nodes: same side, same network, reaches overlapping.
fn links(a: &Arena) -> Vec<(usize, usize)> {
    let e = a.eco.as_ref().unwrap();
    let nodes: Vec<usize> = (0..a.ships.len()).filter(|&i| a.is_node(i)).collect();
    let mut out = Vec::new();
    for (k, &i) in nodes.iter().enumerate() {
        for &j in &nodes[k + 1..] {
            let (si, sj) = (&a.ships[i], &a.ships[j]);
            let r = reach(si) + reach(sj);
            if si.team == sj.team && e.net.get(i) == e.net.get(j) && dist2(si.x, si.y, sj.x, sj.y) <= r * r {
                out.push((i, j));
            }
        }
    }
    out
}

/// One line of how both sides stand, for the log and the headless runs.
pub fn state_line(a: &Arena) -> String {
    let e = a.eco.as_ref().unwrap();
    let side = |t: usize| {
        let ships = a.ships.iter().filter(|s| s.alive && s.team == t && s.role == Role::Ship && s.grow.is_none() && !s.node).count();
        let factories = a.ships.iter().filter(|s| s.alive && s.team == t && s.role == Role::Ship && s.node).count();
        let tl = e.tally[t];
        format!(
            "{} bank {} (peak {}, {} in all its nodes), {} factories, {} ships, {} px standing{}; planted {}, printed {}, summoned {}, mined {}, lost {} ships",
            team_name(t),
            a.bank(e.king[t]),
            tl.bank_peak,
            a.ships.iter().filter(|s| s.alive && s.team == t).map(|s| s.store).sum::<i32>(),
            factories,
            ships,
            a.standing(t),
            if a.king_slowed(t) { " (king slowed)" } else { "" },
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
        if a.king_slowed(s.team) { "THE KING, SLOWED" } else { "THE KING" }.to_string()
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
    if s.role == Role::King {
        lines.extend(wrap(&format!("ITS SIDE HAS {} PX OF SHIPS AND FACTORIES STANDING. WITH UNDER {KING_NEEDS}, AFTER THE FIRST MINUTE, A KING FLIES AT A FIFTH OF ITS SPEED.", a.standing(s.team)), 49));
    }
    lines
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_match_screen_drives_everything_through_its_keys_and_buttons() {
        use crate::app::{App, Ev};
        let mut pg = Pg::new(None, None);
        pg.start_match(true, 3);
        pg.help = false;
        let click = |pg: &mut Pg, r: Rect, right: bool| {
            let (x, y) = (r.x + r.w / 2, r.y + r.h / 2);
            pg.event(Ev::Move(x, y));
            pg.event(Ev::Press { x, y, right });
            pg.event(Ev::Release { x, y, right });
            pg.tick();
        };
        let eco = |pg: &Pg| pg.play.as_ref().unwrap().arena.eco.as_ref().unwrap().orders[0].clone();
        let queue = |pg: &Pg| pg.play.as_ref().unwrap().arena.eco.as_ref().unwrap().queue[0].clone();
        // Orders: the tag, then Shift+number, step a fighter GUARD, ESCORT, HUNT, GUARD.
        assert_eq!(eco(&pg)[0], Order::Guard);
        click(&mut pg, tag_rect(0), false);
        assert_eq!(eco(&pg)[0], Order::Escort);
        assert!(queue(&pg).is_empty(), "a click on the tag also printed the card");
        pg.event(Ev::Key(Key::ShiftNum(1)));
        pg.tick();
        assert_eq!(eco(&pg)[0], Order::Hunt);
        pg.event(Ev::Key(Key::ShiftNum(1)));
        pg.tick();
        assert_eq!(eco(&pg)[0], Order::Guard);
        // A miner steps through four, and comes back to HARVEST. A factory has no tag to click:
        // a click there is a click on the card.
        for want in [Order::Hunt, Order::Guard, Order::Escort, Order::Harvest] {
            pg.event(Ev::Key(Key::ShiftNum(4)));
            pg.tick();
            assert_eq!(eco(&pg)[3], want);
        }
        pg.event(Ev::Key(Key::ShiftNum(5)));
        pg.tick();
        assert_eq!(eco(&pg)[4], Order::Hold);
        // Right-click a card: it repeats. Again: it stops. Backspace or X: the king drops it.
        click(&mut pg, card_rect(1), true);
        assert!(queue(&pg).len() == 1 && queue(&pg)[0].slot == 1 && queue(&pg)[0].repeat);
        click(&mut pg, card_rect(1), true);
        assert!(queue(&pg).len() == 1 && !queue(&pg)[0].repeat);
        pg.event(Ev::Key(Key::Back));
        pg.tick();
        assert!(queue(&pg).is_empty());
        pg.event(Ev::Key(Key::Num(3)));
        pg.tick();
        assert_eq!(queue(&pg).len(), 1);
        click(&mut pg, BTN_CANCEL, false);
        assert!(queue(&pg).is_empty());
        // The queue, as icons: a click on one drops that job, a right-click sets its design
        // repeating, and R does that for the one being printed.
        let slots = |pg: &Pg| queue(pg).iter().map(|j| (j.slot, j.repeat)).collect::<Vec<_>>();
        for n in [1, 2, 3] {
            pg.event(Ev::Key(Key::Num(n)));
            pg.tick();
        }
        assert_eq!(slots(&pg), vec![(0, false), (1, false), (2, false)]);
        click(&mut pg, job_rect(1), false);
        assert_eq!(slots(&pg), vec![(0, false), (2, false)]);
        click(&mut pg, job_rect(1), true);
        assert_eq!(slots(&pg), vec![(0, false), (2, true)]);
        pg.event(Ev::Key(Key::Letter('R')));
        pg.tick();
        assert_eq!(slots(&pg), vec![(0, true), (2, true)]);
        pg.event(Ev::Key(Key::Letter('R')));
        pg.tick();
        assert_eq!(slots(&pg), vec![(0, false), (2, true)]);
        // There's no icon where there's no job, so a click there does nothing.
        click(&mut pg, job_rect(2), false);
        assert_eq!(slots(&pg).len(), 2);
        click(&mut pg, job_rect(0), false);
        assert_eq!(slots(&pg), vec![(2, true)]);
        pg.event(Ev::Key(Key::Back));
        pg.tick();
        assert!(queue(&pg).is_empty());
        // The shipyard costs more than the bank holds: its card won't start a planting.
        click(&mut pg, card_rect(5), false);
        assert_eq!(pg.play.as_ref().unwrap().planting, None);
        // The whole map: Tab and the button. A click on it moves the view and leaves it there,
        // and steering brings it back to the king.
        pg.event(Ev::Key(Key::Tab));
        assert!(pg.play.as_ref().unwrap().tactical);
        click(&mut pg, BTN_TAC, false);
        assert!(!pg.play.as_ref().unwrap().tactical);
        click(&mut pg, BTN_TAC, false);
        let home = pg.play.as_ref().unwrap().cam;
        let far = Rect::new(TAC.x + TAC.w - 20, TAC.y + 10, 1, 1);
        click(&mut pg, far, false);
        let p = pg.play.as_ref().unwrap();
        assert!(!p.follow && p.cam != home && p.cam.0 > MW / 2, "the view didn't move to the click: {:?}", p.cam);
        // Planting from the whole map: the outpost's card, then the home rock.
        click(&mut pg, card_rect(4), false);
        assert_eq!(pg.play.as_ref().unwrap().planting, Some(4));
        let rock = {
            let a = &pg.play.as_ref().unwrap().arena;
            let (kx, ky) = KING_STARTS[0];
            a.rock_near((kx + 125, ky + 35), 40).unwrap()
        };
        click(&mut pg, Rect::new(TAC.x + rock.0 / TAC_SCALE, TAC.y + rock.1 / TAC_SCALE, 1, 1), false);
        let p = pg.play.as_ref().unwrap();
        assert_eq!(p.planting, None);
        assert!(p.arena.ships.iter().any(|s| s.alive && s.team == 0 && s.role == Role::Seed), "no seed set off");
        pg.hold([false, false, false, true]);
        pg.tick();
        let p = pg.play.as_ref().unwrap();
        assert!(p.follow && p.cam.0 < MW / 2, "steering didn't bring the view back");
    }

    #[test]
    fn the_match_rules_fit_their_panel() {
        // Headings start 32 px down a 284 px panel; the last line of the panel is 14 px from its foot.
        let lines: i32 = MATCH_HELP.iter().map(|(_, t)| wrap(t, 49).len() as i32 * 9 + 4).sum();
        assert!(32 + lines <= 284 - 18, "the rules run {} px too far", 32 + lines - (284 - 18));
        // The queue's icons and its drop button sit inside its strip, clear of the cards above
        // and the buttons below.
        let strip = Rect::new(306, card_rect(6).y + card_rect(6).h, 90, BTN_PAUSE.y - card_rect(6).y - card_rect(6).h);
        for r in (0..5).map(job_rect).chain([BTN_CANCEL, QUEUE]) {
            assert!(r.x >= strip.x && r.y >= strip.y && r.x + r.w <= strip.x + strip.w && r.y + r.h <= strip.y + strip.h, "{r:?} is outside the queue's strip {strip:?}");
        }
        assert!(job_rect(4).x + job_rect(4).w <= BTN_CANCEL.x, "the fifth job runs under the drop button");
        // The number in the rules is the number in the sim.
        assert!(MATCH_HELP[0].1.contains(&format!("UNDER {KING_NEEDS} PX")));
        assert_eq!(KING_SPEED, 5 * KING_SLOW, "the hover text says a fifth");
    }
}
