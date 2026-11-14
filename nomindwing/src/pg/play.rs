//! The match screen: have your king print your deck, plant seeds on rocks, select your ships and
//! send them, and watch your factories grow and print. The AI plays the other
//! side with the starter deck, or a person on another machine plays it with their own (`link`).
//! Whichever side this machine plays is `me`: nothing here may take it for blue.

use super::design::*;
use super::link::{Link, Lobby, Met, Msg, Wire, PORT};
use super::scene;
use super::sim::*;
use super::{draw_icon, pc, team_name, Pg, Screen, CAM_STEP, FIELD, ROW_H, ROW_NAME};
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
/// The keys that print the nine cards: the bottom row of the keyboard, under the hand that
/// scrolls. (1 to 9 are the control groups, as in any RTS.) With Ctrl, a card's key selects
/// every ship of that card.
pub(super) const CARD_KEYS: [char; SLOTS] = ['Z', 'X', 'C', 'V', 'B', 'N', 'M', ',', '.'];
/// A second click on a ship, or a second press of a group's number, this many frames or fewer
/// after the first, is a double one.
const DOUBLE: u32 = 6;
/// The king's key counts as a tenth group when a key is pressed twice: after the nine groups.
const KING_KEY: usize = 9;

/// A card's key, as its hover text starts: "[Z] ".
fn card_key(i: usize) -> String {
    CARD_KEYS.get(i).map_or(String::new(), |k| format!("[{k}] "))
}

/// The stance: what every ship you haven't sent anywhere does, guard or attack. On the field's
/// top edge after the clock.
const BTN_STANCE: Rect = Rect::new(40, 1, 70, 11);
/// Lets the ships you've selected go: they're unsent again. After the stance, while any are
/// selected.
const BTN_HOME: Rect = Rect::new(112, 1, 52, 11);
/// The gap kept between neighbours standing in a formation, on top of their sizes: wide enough
/// that friends at rest don't push each other, and that the corners drawn round one selected
/// ship don't run into its neighbour's.
const FORM_GAP: i32 = 6;
/// The lag dial, on the field's top edge beside the network's state.
const BTN_LAG: Rect = Rect::new(258, 1, 44, 11);
const OVER: Rect = Rect::new(52, 100, 200, 88);
const OVER_BTNS: [Rect; 2] = [Rect::new(66, 162, 80, 16), Rect::new(158, 162, 80, 16)];
const MENU: Rect = Rect::new(80, 72, 240, 164);
const MENU_BTNS: [Rect; 5] = [Rect::new(92, 160, 70, 18), Rect::new(166, 160, 76, 18), Rect::new(246, 160, 62, 18), Rect::new(92, 206, 106, 18), Rect::new(202, 206, 106, 18)];
/// Finding the other machine: the panel, the address being typed, and its buttons.
const LOBBY: Rect = Rect::new(60, 86, 280, 128);
const LOBBY_FIELD: Rect = Rect::new(120, 140, 160, 13);
const LOBBY_BTNS: [Rect; 2] = [Rect::new(106, 186, 90, 18), Rect::new(204, 186, 90, 18)];
const NO_PAUSE: &str = "A NETWORK MATCH RUNS AT ONE SPEED: THE OTHER PLAYER'S CLOCK IS RUNNING TOO";

/// A card: one line each, so a team of nine fits above the queue. From the left: its key, its
/// icon, its name, and its price at the right end.
fn card_rect(i: usize) -> Rect {
    Rect::new(306, 14 + i as i32 * ROW_H, 90, ROW_H - 1)
}
const CARD_KEY_X: i32 = 3;
const CARD_ICON_X: i32 = 10;
const CARD_NAME_X: i32 = 23;
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

/// The match's rules panel, by heading.
const MATCH_HELP: [(&str, &str); 10] = [
    ("GOAL", "KILL THE ENEMY KING. LOSE YOURS AND YOU LOSE. A KING IS SLOW WHILE ITS SIDE HAS UNDER 40 PX LEFT."),
    ("KING", "RIGHT-CLICK MOVES IT. A CARD OR ITS KEY PRINTS A SHIP. RIGHT-CLICK REPEATS IT. BACKSPACE STOPS IT."),
    ("SEEDS", "A SEEDED CARD IS PAID FOR AT ONCE. ITS SEED GROWS IT OUT OF THE ROCK YOU CLICK, OR FROM ITS NETWORK IN OPEN SPACE (CTRL+CLICK), AT TWICE THE COST."),
    ("FACTORY", "IT PRINTS ITS BUILD ORDER ROUND AND ROUND, WAITING WHEN THE BANK IS EMPTY. SELECT IT AND RIGHT-CLICK: ITS NEW SHIPS GATHER THERE."),
    ("BANK", "MINERS HAUL TO THE NEAREST NODE (KING OR FACTORY). LINKED NODES SHARE A BANK; NODES CUT OFF FROM YOUR KING KEEP THEIR OWN."),
    ("FLEET", "DRAG A BOX OR CLICK TO SELECT; DOUBLE-CLICK A TYPE. RIGHT-CLICK SENDS THEM THERE TO FIGHT."),
    ("UNSENT", "UNSENT SHIPS GUARD WHERE THEY WERE BUILT. T, OR THE BUTTON BY THE CLOCK: THEY ALL ATTACK."),
    ("VIEW", "WASD OR ARROWS SCROLL. CLICK THE SMALL MAP TO JUMP. K TWICE: YOUR KING. TAB: WHOLE MAP."),
    ("SOLID", "PLANTED THINGS AND BIGGER ENEMY SHIPS ARE SOLID."),
    ("KEYS", "F3 LISTS EVERY KEY. SPACE PAUSES. ESC QUITS."),
];

/// A match in progress, and how you're looking at it.
pub struct Play {
    pub arena: Arena,
    /// How commands reach the match: after the lag, and in step with another machine if there
    /// is one. It also says which side this machine plays.
    pub link: Link,
    pub cam: (i32, i32),
    /// The field shows the whole map at quarter scale.
    pub tactical: bool,
    /// A seeded card picked, waiting for you to click the rock its seed should go to.
    pub planting: Option<usize>,
    /// The ships you've selected, to send somewhere (their places in the arena's list).
    pub picked: Vec<usize>,
    /// Where on the field a press began that may become a click on a ship or a box round several.
    box_from: Option<(i32, i32)>,
    /// The control groups, 1 to 9: the ships in each.
    pub groups: [Vec<usize>; 9],
    /// The ship last clicked and the group last called, and the frame it was: a second within
    /// `DOUBLE` frames is a double click, or a call to go and look.
    last_click: Option<(usize, u32)>,
    last_group: Option<(usize, u32)>,
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
    /// Whether the log has been told that the two machines parted, that the other left, and
    /// that their first check agreed.
    logged_net: [bool; 3],
}

impl Pg {
    // ----- Starting and leaving -----

    /// Starts a match with your team as your deck, or the starter deck. The AI always
    /// plays the starter deck.
    pub fn start_match(&mut self, starter: bool, seed: u64) {
        let mine = if starter { starter_deck() } else { self.lib.clone() };
        let arena = Arena::for_match(seed, [mine, starter_deck()], [false, true]);
        let mut p = Play { arena, link: Link::solo(), cam: (0, 0), tactical: false, planting: None, picked: Vec::new(), box_from: None, groups: Default::default(), last_click: None, last_group: None, running: true, speed: 1, starter, seed, map_drag: None, logged_over: false, next_log: 15 * 60, slowed: [false; 2], logged_net: [false; 3] };
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

    /// Starts a match against a person on another machine, now that the two have met: the host
    /// plays blue and the joiner red, each with their own team.
    pub fn start_net(&mut self, met: Met) {
        let me = if met.host { 0 } else { 1 };
        self.start_linked(me, met.seed, met.deck, Box::new(met.wire), met.backlog);
    }

    /// The same, over any wire.
    pub fn start_linked(&mut self, me: usize, seed: u64, theirs: Vec<Design>, wire: Box<dyn Wire>, backlog: Vec<Msg>) {
        let decks = if me == 0 { [self.lib.clone(), theirs] } else { [theirs, self.lib.clone()] };
        let arena = Arena::for_match(seed, decks, [false, false]);
        let mut p = Play { arena, link: Link::net(me, wire, backlog), cam: (0, 0), tactical: false, planting: None, picked: Vec::new(), box_from: None, groups: Default::default(), last_click: None, last_group: None, running: true, speed: 1, starter: false, seed, map_drag: None, logged_over: false, next_log: 15 * 60, slowed: [false; 2], logged_net: [false; 3] };
        p.look_at_king();
        self.play = Some(p);
        self.menu = false;
        self.lobby = None;
        self.screen = Screen::Match;
        // No rules panel first: the match is running on the other machine already.
        self.help = false;
        self.log(&format!("NET MATCH START seed {seed}, YOU ARE {}", team_name(me)));
        for d in self.lib.clone() {
            self.log(&format!("  DESIGN {}", d.to_line()));
        }
    }

    /// Hosts a network match: waits for the other player on a port. `local` listens to this
    /// machine only, for two games side by side.
    pub fn host(&mut self, port: u16, local: bool) {
        self.menu = false;
        self.lobby = Some(if local { Lobby::host_on("127.0.0.1", port) } else { Lobby::host(port) });
        self.log(&format!("NET HOST on port {port}{}", if local { ", this machine only" } else { "" }));
    }

    /// Joins a network match at an address, or asks for the address if none is given.
    pub fn join(&mut self, addr: Option<&str>) {
        self.menu = false;
        self.lobby = Some(match addr {
            Some(a) => Lobby::join(a),
            None => Lobby::Address("127.0.0.1".into()),
        });
    }

    /// One frame of finding the other machine. When both have said hello, the match starts.
    pub(super) fn step_lobby(&mut self) {
        let seed = crate::game::utc_stamp().bytes().fold(7u64, |h, b| h.wrapping_mul(31).wrapping_add(b as u64)) % 100_000 + 2;
        let Some(lobby) = self.lobby.as_mut() else { return };
        if let Some(met) = lobby.poll(seed, &self.lib) {
            self.start_net(met);
        }
    }

    /// A key while finding the other machine. True if it was used.
    pub(super) fn key_lobby(&mut self, k: Key) -> bool {
        match (self.lobby.as_mut(), k) {
            (None, _) => return false,
            (Some(Lobby::Address(text)), Key::Back) => {
                text.pop();
            }
            (Some(Lobby::Address(text)), Key::Enter) if !text.is_empty() => self.lobby = Some(Lobby::join(text)),
            (Some(_), Key::Esc) => self.lobby = None,
            _ => {}
        }
        true
    }

    /// A typed character goes into the address.
    pub(super) fn char_lobby(&mut self, c: char) {
        if let Some(Lobby::Address(text)) = self.lobby.as_mut() {
            if (c.is_ascii_alphanumeric() || ".:-".contains(c)) && text.len() < 26 {
                text.push(c.to_ascii_uppercase());
            }
        }
    }

    pub(super) fn press_lobby(&mut self, x: i32, y: i32) {
        let typing = matches!(self.lobby, Some(Lobby::Address(_)));
        if typing && LOBBY_BTNS[0].contains(x, y) {
            self.key_lobby(Key::Enter);
        } else if LOBBY_BTNS[1].contains(x, y) || (!typing && LOBBY_BTNS[0].contains(x, y)) {
            self.lobby = None;
        }
    }

    pub(super) fn draw_lobby(&self, fb: &mut Framebuffer) {
        let Some(lobby) = &self.lobby else { return };
        let r = LOBBY;
        ui::panel(fb, r);
        let m = self.mouse;
        let (head, lines): (&str, Vec<String>) = match lobby {
            Lobby::Address(_) => ("JOIN A MATCH", wrap("TYPE THE HOST'S ADDRESS. 127.0.0.1 IS THIS MACHINE.", 43)),
            Lobby::Hosting { port, .. } => {
                let port = if *port == PORT { String::new() } else { format!(" (PORT {port})") };
                ("HOSTING A MATCH", wrap(&format!("WAITING FOR THE OTHER PLAYER{port}. THEY CHOOSE JOIN A MATCH AND TYPE THIS MACHINE'S ADDRESS, OR 127.0.0.1 IF THEIR GAME IS ON THIS MACHINE TOO. YOU'LL PLAY BLUE."), 43))
            }
            Lobby::Joining { addr, .. } => ("JOINING A MATCH", wrap(&format!("CONNECTING TO {}...", addr.to_uppercase()), 43)),
            Lobby::Meeting { .. } => ("FOUND THEM", wrap("SWAPPING DECKS...", 43)),
            Lobby::Failed(why) => ("NO MATCH", wrap(why, 43)),
        };
        fb.text_centered(head, r.x + r.w / 2, r.y + 8, ACCENT, Some(OUTLINE), 2);
        for (i, l) in lines.iter().enumerate() {
            fb.text(l, r.x + 10, r.y + 30 + i as i32 * 9, TEXT, None, 1);
        }
        if let Lobby::Address(text) = lobby {
            let f = LOBBY_FIELD;
            fb.rect_fill(f.x, f.y, f.w, f.h, PANEL_DEEP);
            fb.rect_outline(f.x, f.y, f.w, f.h, ACCENT);
            fb.text(&format!("{text}_"), f.x + 4, f.y + 3, WHITE, None, 1);
            fb.text_centered("ENTER JOINS. ESC GOES BACK.", r.x + r.w / 2, f.y + 20, TEXT_DIM, None, 1);
            for (b, label) in LOBBY_BTNS.iter().zip(["JOIN", "BACK"]) {
                ui::button(fb, *b, label, b.contains(m.0, m.1), false);
            }
        } else {
            let b = LOBBY_BTNS[1];
            ui::button(fb, b, "BACK", b.contains(m.0, m.1), false);
        }
    }

    fn leave_match(&mut self) {
        if let Some(p) = self.play.as_mut() {
            p.link.bye();
        }
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
        let me = p.link.me;
        // Ships that are gone drop out of the selection.
        p.picked.retain(|&i| p.arena.ships.get(i).is_some_and(|s| s.selectable(me)));
        // WASD or the arrows scroll the view. (They flew the king until 2026-11-13: now it's
        // selected and sent like anything else.)
        let (dx, dy) = ((dirs[3] as i32 - dirs[2] as i32) * CAM_STEP, (dirs[1] as i32 - dirs[0] as i32) * CAM_STEP);
        p.cam = ((p.cam.0 + dx).clamp(0, MW - FIELD.w), (p.cam.1 + dy).clamp(0, MH - FIELD.h));
        // In step with another machine there's one speed and no pause: their clock runs too.
        let want = if p.link.net.is_some() {
            1
        } else if p.running {
            p.speed
        } else {
            0
        };
        p.link.advance(&mut p.arena, want);
        let mut lines = Vec::new();
        if let Some(net) = p.link.net.as_ref() {
            if let (Some(t), false) = (net.parted, p.logged_net[0]) {
                p.logged_net[0] = true;
                lines.push(format!("NET OUT OF STEP from {} (tick {t}); last agreed tick {}", clock(t), net.agreed));
            }
            if net.agreed > 0 && !p.logged_net[2] {
                p.logged_net[2] = true;
                lines.push(format!("NET IN STEP: the first check agreed, at tick {}", net.agreed));
            }
            if let (Some(why), false) = (&net.gone, p.logged_net[1]) {
                p.logged_net[1] = true;
                lines.push(format!("NET ENDED at {}: {why}", clock(p.arena.tick)));
            }
        }
        for t in 0..2 {
            let now = p.arena.over.is_none() && p.arena.king_slowed(t);
            if now != p.slowed[t] && p.arena.over.is_none() {
                p.slowed[t] = now;
                lines.push(format!("KING {} {} at {}, {} px standing", team_name(t), if now { "SLOWED" } else { "FREE" }, clock(p.arena.tick), p.arena.standing(t)));
            }
        }
        if p.arena.tick >= p.next_log && p.arena.over.is_none() {
            p.next_log += 15 * 60;
            let net = p.link.net.as_ref().map_or(String::new(), |n| format!(" | NET lag {}, in step to {}", p.link.delay, clock(n.agreed)));
            lines.push(format!("MATCH {} | {}{net}", clock(p.arena.tick), Self::match_state(&p.arena)));
        }
        if let Some(result) = p.arena.over {
            if !p.logged_over {
                p.logged_over = true;
                p.running = false;
                let other = if p.link.net.is_some() { "THEM" } else { "AI" };
                let who = match result {
                    Some(w) if w == me => format!("{} (YOU) WIN", team_name(w)),
                    Some(w) => format!("{} ({other}) WINS", team_name(w)),
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
        let (ctrl, shift) = (self.ctrl, self.shift);
        let Some(p) = self.play.as_mut() else { return };
        let (me, net) = (p.link.me, p.link.net.is_some());
        match k {
            // 1 to 9 are the control groups. The cards are on the bottom row of the keyboard.
            Key::Num(n) if (1..=9).contains(&n) => self.group_key(n as usize - 1, false),
            Key::ShiftNum(n) if (1..=9).contains(&n) => self.group_key(n as usize - 1, true),
            // A card's key prints it. With Ctrl, it selects every ship of that card.
            Key::Letter(c) if CARD_KEYS.contains(&c) => {
                let i = CARD_KEYS.iter().position(|&k| k == c).unwrap();
                if ctrl {
                    self.select_card(i, shift);
                } else {
                    self.card(i);
                }
            }
            Key::Space | Key::Letter('P') | Key::Letter('F') if net => self.say(NO_PAUSE),
            Key::Space | Key::Letter('P') => {
                if p.arena.over.is_none() {
                    p.running = !p.running;
                }
            }
            Key::Letter('F') => p.speed = if p.speed >= 4 { 1 } else { p.speed * 2 },
            Key::Letter('L') => self.turn_lag(),
            // Your warships in view. With Ctrl, every one you have, wherever it is.
            Key::Letter('Q') => p.select_fleet(ctrl, shift),
            Key::Letter('G') => self.let_go(),
            Key::Letter('T') => self.turn_stance(),
            Key::Letter('K') => self.king_key(),
            Key::Tab => self.toggle_tactical(),
            Key::Back => self.drop_job(0),
            Key::Letter('R') => {
                // The last job asked for repeats, or stops. After a card's number, that's a
                // repeating job of the card on the end of the queue: a right-click on the card.
                // The job may still be waiting out the lag, so the command names its design and
                // finds the job when it's applied.
                let queued = p.arena.eco.as_ref().and_then(|e| e.queue[me].last()).map(|j| j.slot);
                if let Some(slot) = p.link.print_waiting().or(queued) {
                    let line = format!("REPEAT LAST JOB at {}", clock(p.arena.tick));
                    p.link.give(&p.arena, Cmd::Repeat { team: me, slot, at: Some(LAST_JOB) });
                    self.log(&line);
                }
            }
            _ => {}
        }
    }

    /// The lag dial: how many ticks your commands wait before they're applied.
    fn turn_lag(&mut self) {
        let Some(p) = self.play.as_mut() else { return };
        p.link.next_delay();
        let line = format!("LAG {} ticks at {}", p.link.delay, clock(p.arena.tick));
        self.log(&line);
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
        let me = p.link.me;
        let Some(card) = p.arena.eco.as_ref().and_then(|e| e.cards[me].get(i).copied()) else { return };
        if p.arena.eco.as_ref().is_some_and(|e| e.deck[me][i].is_empty()) {
            self.say(&format!("SLOT {} IS EMPTY: DRAW A SHIP THERE IN THE YARD", i + 1));
            return;
        }
        if !card.ok {
            self.say("THAT DESIGN CAN'T FLY YET: FIX IT IN THE YARD");
            return;
        }
        if card.seeded {
            let bank = p.arena.bank(p.arena.eco.as_ref().unwrap().king[me]);
            if bank < card.price() {
                let s = format!("IT COSTS {} AND YOUR BANK HOLDS {bank}", card.price());
                self.say(&s);
                return;
            }
            p.planting = Some(i);
            self.say("CLICK A ROCK FOR THE SEED. RIGHT-CLICK CANCELS");
        } else {
            match p.arena.can_print(me, i, None) {
                Ok(_) => p.link.give(&p.arena, Cmd::Print { team: me, slot: i, target: None }),
                Err(why) => self.say(why),
            }
        }
    }

    /// The stance button: every ship you haven't sent anywhere attacks the nearest enemy, or
    /// guards where it was built.
    fn turn_stance(&mut self) {
        let Some(p) = self.play.as_mut() else { return };
        let me = p.link.me;
        let attack = !p.arena.attacking(me);
        let line = format!("STANCE {} at {}", if attack { "ATTACK" } else { "GUARD" }, clock(p.arena.tick));
        p.link.give(&p.arena, Cmd::Stance { team: me, attack });
        self.log(&line);
    }

    /// Ctrl and a card's key: every ship of that card you have, wherever it is. With `add`
    /// (Shift) they join what's selected.
    fn select_card(&mut self, i: usize, add: bool) {
        let Some(p) = self.play.as_mut() else { return };
        let me = p.link.me;
        let got: Vec<usize> = (0..p.arena.ships.len()).filter(|&j| p.arena.ships[j].selectable(me) && p.arena.ships[j].role == Role::Ship && p.arena.ships[j].slot == i).collect();
        if got.is_empty() {
            let name = p.arena.eco.as_ref().and_then(|e| e.deck[me].get(i)).filter(|d| !d.is_empty()).map(|d| d.name.clone());
            if let Some(name) = name {
                self.say(&format!("YOU HAVE NO {name} TO SELECT"));
            }
            return;
        }
        p.take(got, add);
    }

    /// Right-click on a card: a job of it goes on the end of the king's queue, on repeat. If one
    /// of its jobs repeats already, they all stop repeating instead.
    fn repeat(&mut self, i: usize) {
        let Some(p) = self.play.as_mut() else { return };
        let me = p.link.me;
        let Some(e) = p.arena.eco.as_ref() else { return };
        let Some(card) = e.cards[me].get(i).copied() else { return };
        if !card.ok || card.seeded {
            self.say("ONLY SHIPS YOUR KING PRINTS CAN REPEAT");
            return;
        }
        let on = e.queue[me].iter().any(|j| j.slot == i && j.repeat);
        let line = format!("REPEAT {} {} at {}", e.deck[me][i].name, if on { "OFF" } else { "ON" }, clock(p.arena.tick));
        p.link.give(&p.arena, Cmd::Repeat { team: me, slot: i, at: None });
        self.log(&line);
    }

    /// Right-click on a job in the queue: that job repeats, or stops.
    fn repeat_job(&mut self, at: usize) {
        let Some(p) = self.play.as_mut() else { return };
        let me = p.link.me;
        let Some(e) = p.arena.eco.as_ref() else { return };
        let Some(j) = e.queue[me].get(at).copied() else { return };
        let line = format!("REPEAT JOB {} {} at {}", e.deck[me].get(j.slot).map_or("?", |d| d.name.as_str()), if j.repeat { "OFF" } else { "ON" }, clock(p.arena.tick));
        p.link.give(&p.arena, Cmd::Repeat { team: me, slot: j.slot, at: Some(at) });
        self.log(&line);
    }

    /// Drops a job from the king's queue: 0 is the one it's printing.
    fn drop_job(&mut self, at: usize) {
        let Some(p) = self.play.as_mut() else { return };
        let me = p.link.me;
        let Some(e) = p.arena.eco.as_ref() else { return };
        let Some(j) = e.queue[me].get(at).copied() else { return };
        let line = format!("DROP {} at {}", e.deck[me].get(j.slot).map_or("?", |d| d.name.as_str()), clock(p.arena.tick));
        p.link.give(&p.arena, Cmd::Cancel { team: me, at, slot: j.slot });
        self.log(&line);
    }

    /// Esc while planting cancels the planting. True if it did.
    pub(super) fn cancel_planting(&mut self) -> bool {
        match self.play.as_mut() {
            Some(p) if p.planting.is_some() => {
                p.planting = None;
                true
            }
            // With no card picked, Esc lets a selection go.
            Some(p) if !p.picked.is_empty() => {
                p.picked.clear();
                true
            }
            _ => false,
        }
    }

    /// K, or a click on the note above the small map: your king is selected. Again straight
    /// away, the view goes to it, as with a group's number. (Henry, 2026-11-14: "K once should
    /// select your king and twice should jump the camera to it.") With Shift the king joins
    /// what's selected.
    fn king_key(&mut self) {
        let (shift, frame) = (self.shift, self.frames);
        let Some(p) = self.play.as_mut() else { return };
        let king = p.arena.eco.as_ref().unwrap().king[p.link.me];
        if p.arena.ships[king].alive {
            p.take(vec![king], shift);
        }
        if p.last_group.is_some_and(|(k, f)| k == KING_KEY && frame.saturating_sub(f) <= DOUBLE) {
            p.look_at_king();
        }
        p.last_group = Some((KING_KEY, frame));
    }

    /// A control group's key. With Ctrl held, the group becomes what's selected. With Shift,
    /// what's selected joins it. Alone, the group is selected; pressed again straight away, the
    /// view goes to it.
    fn group_key(&mut self, g: usize, add: bool) {
        let (ctrl, frame) = (self.ctrl, self.frames);
        let Some(p) = self.play.as_mut() else { return };
        let me = p.link.me;
        let live = |a: &Arena, v: &[usize]| -> Vec<usize> { v.iter().copied().filter(|&i| a.ships[i].selectable(me)).collect() };
        let mut note = None;
        if ctrl {
            p.groups[g] = live(&p.arena, &p.picked);
            note = Some(format!("GROUP {}: {} SHIPS", g + 1, p.groups[g].len()));
        } else if add {
            for i in live(&p.arena, &p.picked) {
                if !p.groups[g].contains(&i) {
                    p.groups[g].push(i);
                }
            }
            note = Some(format!("GROUP {}: {} SHIPS", g + 1, p.groups[g].len()));
        } else {
            p.groups[g] = live(&p.arena, &p.groups[g]);
            if !p.groups[g].is_empty() {
                p.picked = p.groups[g].clone();
                if p.last_group.is_some_and(|(k, f)| k == g && frame.saturating_sub(f) <= DOUBLE) {
                    let n = p.picked.len() as i32;
                    let mid = (p.picked.iter().map(|&i| p.arena.ships[i].x).sum::<i32>() / n, p.picked.iter().map(|&i| p.arena.ships[i].y).sum::<i32>() / n);
                    p.look_at(mid);
                }
                p.last_group = Some((g, frame));
            }
        }
        if let Some(n) = note {
            self.say(&n);
        }
    }

    /// Sends the ships you've selected to a place on the map, or after a thing (`target`): a
    /// command for each, and all at the pace of the slowest of them, so they arrive together.
    /// Sent to a place, each has its own spot in a formation there (`formation`). A factory
    /// can't be sent: the place becomes its rally point, where its new ships gather.
    fn send_picked(&mut self, at: (i32, i32), target: Option<usize>) {
        let Some(p) = self.play.as_mut() else { return };
        let me = p.link.me;
        let ships: Vec<usize> = p.picked.iter().copied().filter(|&i| p.arena.ships[i].commandable(me)).collect();
        let factories: Vec<usize> = p.picked.iter().copied().filter(|&i| p.arena.ships[i].rallies(me)).collect();
        // A king's own speed is its pace, whatever engines are drawn on it.
        let speed = |s: &Ship| if s.role == Role::King { KING_SPEED } else { s.st.vmax };
        let pace = if ships.len() > 1 { ships.iter().map(|&i| speed(&p.arena.ships[i])).min().unwrap_or(0).clamp(0, 255) } else { 0 };
        let spots = if target.is_none() { formation(&p.arena, &ships, at) } else { vec![at; ships.len()] };
        for (k, &ship) in ships.iter().enumerate() {
            let c = match target {
                Some(t) if t != ship => Cmd::Follow { team: me, ship, target: t, pace },
                _ => Cmd::Send { team: me, ship, at: spots[k], pace },
            };
            p.link.give(&p.arena, c);
        }
        let to = target.map_or(at, |t| (p.arena.ships[t].x, p.arena.ships[t].y));
        for &ship in &factories {
            p.link.give(&p.arena, Cmd::Rally { team: me, ship, at: to });
        }
        let mut lines = Vec::new();
        if !ships.is_empty() {
            let after = target.map_or(String::new(), |t| format!(" after {} {}", team_name(p.arena.ships[t].team), p.arena.ships[t].name));
            lines.push(format!("SEND {} ships to {},{}{after} at {}", ships.len(), at.0, at.1, clock(p.arena.tick)));
        }
        if !factories.is_empty() {
            lines.push(format!("RALLY {} factories to {},{} at {}", factories.len(), to.0, to.1, clock(p.arena.tick)));
        }
        for l in lines {
            self.log(&l);
        }
    }

    /// Lets the ships you've selected go: they're unsent again, and do as the stance says.
    fn let_go(&mut self) {
        let Some(p) = self.play.as_mut() else { return };
        let me = p.link.me;
        let ships: Vec<usize> = p.picked.iter().copied().filter(|&i| p.arena.ships[i].commandable(me)).collect();
        for &ship in &ships {
            p.link.give(&p.arena, Cmd::Home { team: me, ship });
        }
        if !ships.is_empty() {
            let line = format!("LET GO {} ships at {}", ships.len(), clock(p.arena.tick));
            self.log(&line);
        }
    }

    /// Plants the seed of `slot` on the rock nearest a world point, if there is one within `slack`.
    fn plant(&mut self, slot: usize, at: (i32, i32), slack: i32) {
        let Some(p) = self.play.as_mut() else { return };
        let me = p.link.me;
        match p.arena.rock_near(at, slack) {
            Some(target) => {
                p.planting = None;
                match p.arena.can_print(me, slot, Some(target)) {
                    Ok(cost) => {
                        let line = format!("PLANT {} for {cost} at {}", p.arena.eco.as_ref().unwrap().deck[me][slot].name, clock(p.arena.tick));
                        p.link.give(&p.arena, Cmd::Print { team: me, slot, target: Some(target) });
                        self.log(&line);
                    }
                    Err(why) => self.say(why),
                }
            }
            None => self.say("THAT'S NOT A ROCK: CLICK ON ONE"),
        }
    }

    /// Sends a seeded card's seed to a spot in open space. Refused, the card stays picked, to
    /// try another spot.
    fn sow(&mut self, slot: usize, at: (i32, i32)) {
        let Some(p) = self.play.as_mut() else { return };
        let me = p.link.me;
        match p.arena.can_sow(me, slot, at) {
            Ok(cost) => {
                p.planting = None;
                let line = format!("SOW {} for {cost} at {}, in open space", p.arena.eco.as_ref().unwrap().deck[me][slot].name, clock(p.arena.tick));
                p.link.give(&p.arena, Cmd::Sow { team: me, slot, at });
                self.log(&line);
            }
            Err(why) => self.say(why),
        }
    }

    pub(super) fn press_match(&mut self, x: i32, y: i32, right: bool) {
        let ctrl = self.ctrl;
        let Some(p) = self.play.as_mut() else { return };
        let net = p.link.net.is_some();
        if p.ended() && !right {
            // A network match has no second go: both players would have to want one.
            if !net && OVER_BTNS[0].contains(x, y) {
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
            } else if !p.picked.is_empty() && (MINI.contains(x, y) || (FIELD.contains(x, y) && !p.on_top_button(x, y))) {
                // With ships selected, a right-click on the map (either one) sends them there. On
                // the field, a right-click on a thing sends them after it.
                if let Some(at) = p.world_point(x, y) {
                    let target = if FIELD.contains(x, y) { p.ship_under((x, y), |s| s.warp == 0) } else { None };
                    self.send_picked(at, target);
                }
            } else if let Some(i) = (0..SLOTS).find(|&i| card_rect(i).contains(x, y)) {
                self.repeat(i);
            } else if let Some((k, _)) = p.job_at(x, y) {
                self.repeat_job(k);
            }
            return;
        }
        if BTN_LAG.contains(x, y) {
            self.turn_lag();
            return;
        }
        if BTN_STANCE.contains(x, y) {
            self.turn_stance();
            return;
        }
        if !p.picked.is_empty() && BTN_HOME.contains(x, y) {
            self.let_go();
            return;
        }
        if FIELD.contains(x, y) {
            match (p.planting, p.world_point(x, y)) {
                // Ctrl held: the seed goes to that spot in open space.
                (Some(slot), Some(at)) if ctrl => self.sow(slot, at),
                (Some(slot), Some(at)) => {
                    let slack = if p.tactical { 2 * TAC_SCALE } else { 6 };
                    self.plant(slot, at, slack);
                }
                // With no card picked, a press on the field begins a selection: a click on a ship,
                // or a box dragged round several. (On the whole map too. Looking round is the
                // small map's job.)
                (None, _) => p.box_from = Some((x, y)),
                _ => {}
            }
            return;
        }
        if let Some(i) = (0..SLOTS).find(|&i| card_rect(i).contains(x, y)) {
            self.card(i);
        } else if (306..400).contains(&x) && (VIEW_NOTE_Y - 1..VIEW_NOTE_Y + 8).contains(&y) {
            self.king_key();
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
        } else if net && (BTN_PAUSE.contains(x, y) || BTN_SPEED.contains(x, y)) {
            self.say(NO_PAUSE);
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

    pub(super) fn release_match(&mut self, x: i32, y: i32) {
        let (ctrl, shift, frame) = (self.ctrl, self.shift, self.frames);
        if let Some(p) = self.play.as_mut() {
            p.map_drag = None;
            if let Some(from) = p.box_from.take() {
                p.select(from, (x, y), ctrl, shift, frame);
            }
        }
    }

    pub(super) fn press_menu(&mut self, x: i32, y: i32) {
        if MENU_BTNS[0].contains(x, y) {
            let seed = crate::game::utc_stamp().bytes().fold(7u64, |h, b| h.wrapping_mul(31).wrapping_add(b as u64)) % 100_000 + 2;
            self.start_match(false, seed);
        } else if MENU_BTNS[1].contains(x, y) {
            let seed = crate::game::utc_stamp().bytes().fold(7u64, |h, b| h.wrapping_mul(31).wrapping_add(b as u64)) % 100_000 + 2;
            self.start_match(true, seed);
        } else if MENU_BTNS[3].contains(x, y) {
            self.host(PORT, false);
        } else if MENU_BTNS[4].contains(x, y) {
            self.join(None);
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
            fb.text(l, r.x + 10, r.y + 28 + i as i32 * 9, TEXT, None, 1);
        }
        fb.text("OR AGAINST A PERSON, EACH WITH THEIR", r.x + 10, MENU_BTNS[3].y - 22, TEXT, None, 1);
        fb.text("OWN DECK, ON THIS MACHINE OR ANOTHER:", r.x + 10, MENU_BTNS[3].y - 13, TEXT, None, 1);
        let m = self.mouse;
        for (b, label) in MENU_BTNS.iter().zip(["YOUR DECK", "STARTER DECK", "BACK", "HOST A MATCH", "JOIN A MATCH"]) {
            ui::button(fb, *b, label, b.contains(m.0, m.1), false);
        }
    }

    pub(super) fn draw_match(&self, fb: &mut Framebuffer) {
        let Some(p) = &self.play else { return };
        let a = &p.arena;
        let e = a.eco.as_ref().unwrap();
        let me = p.link.me;
        let m = self.mouse;
        let f = FIELD;
        // The world point under the pointer, on whichever view the field shows.
        let on_home = !p.picked.is_empty() && BTN_HOME.contains(m.0, m.1);
        let at = if f.contains(m.0, m.1) && !p.on_top_button(m.0, m.1) { p.world_point(m.0, m.1) } else { None };
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
        // What you've selected, in green corners. A cross where each ship you've sent is going
        // (small ones, each ship's own spot, for a group in formation), and for a factory a line
        // to the flag where its new ships gather.
        let mut going: Vec<(i32, i32)> = Vec::new();
        for &i in p.picked.iter().filter(|&&i| a.ships[i].alive) {
            let (l, t, r, b) = a.ships[i].bbox();
            let (x, y, w, h) = on_field(l, t, r, b);
            if x - 2 >= clip.x && y - 2 >= clip.y && x + w + 2 <= clip.x + clip.w && y + h + 2 <= clip.y + clip.h {
                fb.corners(x - 2, y - 2, w + 4, h + 4, 2, GOOD);
            }
            // (No cross under a ship that's standing on its spot.)
            if let Some(to) = a.ships[i].sent.filter(|to| !going.contains(to) && cheb(a.ships[i].x, a.ships[i].y, to.0, to.1) > 2) {
                going.push(to);
            }
            if let Some(to) = a.ships[i].rally {
                let (from, to) = (spot((a.ships[i].x, a.ships[i].y)), spot(to));
                crate::render::dotted(fb, clip, from.0, from.1, to.0, to.1, 3, GOOD);
                // The flag: a pole standing on the place, and a pennant at its top.
                for k in 0..6 {
                    pc(fb, clip, to.0, to.1 - k, GOOD);
                }
                for (dx, dy) in [(1, -5), (2, -5), (3, -4), (1, -4), (2, -4), (1, -3), (2, -3)] {
                    pc(fb, clip, to.0 + dx, to.1 + dy, GOOD);
                }
            }
        }
        let arm = if going.len() > 1 { 1 } else { 2 };
        for to in going {
            let (x, y) = spot(to);
            for k in -arm..=arm {
                pc(fb, clip, x + k, y + k, GOOD);
                pc(fb, clip, x + k, y - k, GOOD);
            }
        }
        // The box being dragged.
        if let Some((x0, y0)) = p.box_from {
            let (x1, y1) = (m.0.clamp(f.x, f.x + f.w - 1), m.1.clamp(f.y, f.y + f.h - 1));
            if (x1 - x0).abs() > 2 || (y1 - y0).abs() > 2 {
                fb.rect_outline(x0.min(x1), y0.min(y1), (x1 - x0).abs() + 1, (y1 - y0).abs() + 1, WHITE);
            }
        }
        // Planting: the rock under the pointer, whether the design fits on it, and what it would
        // link to there.
        let mut bar: Option<Vec<String>> = None;
        // A warning in the bar is gold: a seed in open space costs about twice.
        let mut bar_col = TEXT;
        if let (Some(slot), Some(at)) = (p.planting, at) {
            let need = e.cards[me][slot].count;
            let (name, price) = (&e.deck[me][slot].name, e.cards[me][slot].price());
            // Lines to what a new factory there would link to: green if that leads to your king,
            // gold if not. With nothing in reach, a dim one to the nearest node, and the words
            // for how far short it is.
            let linked = |fb: &mut Framebuffer, l: &crate::pg::sim::Links| {
                let from = spot(l.at);
                let mut line = |to: (i32, i32), col: u8| {
                    let to = spot(to);
                    crate::render::dotted(fb, clip, from.0, from.1, to.0, to.1, 2, col);
                };
                for &to in &l.to {
                    line(to, if l.king { GOOD } else { GOLD });
                }
                match (l.king, l.gap) {
                    (false, Some((to, short))) => {
                        line(to, TEXT_DIM);
                        format!(" {short} PX TOO FAR TO LINK.")
                    }
                    _ => String::new(),
                }
            };
            if self.ctrl {
                // The drawing's own box at the spot: gold where the seed may go, red where not.
                let sh = e.deck[me][slot].shape();
                let mid = a.mid_from(me, slot, at).unwrap_or(at);
                let (l, t) = (mid.0 - sh.w / 2, mid.1 - sh.h / 2);
                let (x, y, w, h) = on_field(l, t, l + sh.w - 1, t + sh.h - 1);
                bar_col = GOLD;
                match a.sow_links(me, slot, at) {
                    Ok(links) => {
                        fb.corners(x - 2, y - 2, w + 4, h + 4, 3, GOLD);
                        let link = linked(fb, &links);
                        bar = Some(wrap(&format!("{}{link} RIGHT-CLICK OR ESC CANCELS.", space_warning(name, price, need - 1)), 49));
                    }
                    Err(why) => {
                        fb.corners(x - 2, y - 2, w + 4, h + 4, 3, BAD);
                        bar = Some(wrap(&format!("NOT HERE. {why}. RIGHT-CLICK OR ESC CANCELS."), 49));
                    }
                }
            } else {
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
                    // A factory on a rock too small for it finishes from the bank; anything else
                    // has no network to feed it, and stops where the rock does.
                    let short = need - rock.len() as i32;
                    let fit = match here {
                        Some(s) => format!("{} {} IS PLANTED HERE ALREADY: THEY'LL WALL EACH OTHER IN.", team_name(s.team), s.name),
                        None if fits => "IT FITS.".to_string(),
                        None if e.cards[me][slot].st.factory > 0 => {
                            bar_col = GOLD;
                            format!("THE ROCK IS {short} PX SHORT: IT DRAWS THOSE FROM ITS NETWORK.")
                        }
                        None => format!("IT'LL STAY {}% GROWN.", rock.len() as i32 * 100 / need.max(1)),
                    };
                    let link = a.plant_links(me, slot, px).map_or(String::new(), |l| linked(fb, &l));
                    bar = Some(wrap(&format!("PLANT {name} HERE FOR {price} PX: ROCK {} PX, NEEDS {need}. {fit}{link} RIGHT-CLICK OR ESC CANCELS.", rock.len()), 49));
                }
                None => bar = Some(wrap(&format!("PLANT {name}: CLICK A ROCK. FOR A SPOT IN OPEN SPACE, HOLD CTRL. RIGHT-CLICK OR ESC CANCELS."), 49)),
            }
            }
        }
        fb.text(&clock(a.tick), 4, 3, TEXT_DIM, Some(OUTLINE), 1);
        // The lag dial, and beside it how the two machines stand: the time of the last hash
        // they agreed on.
        ui::button(fb, BTN_LAG, &format!("LAG {}", p.link.delay), BTN_LAG.contains(m.0, m.1), p.link.delay > 0);
        let attacking = a.attacking(me);
        ui::button(fb, BTN_STANCE, stance_label(attacking), BTN_STANCE.contains(m.0, m.1), attacking);
        if !p.picked.is_empty() {
            ui::button(fb, BTN_HOME, "LET GO", on_home, false);
        }
        let mut notes: Vec<(String, u8)> = Vec::new();
        if let Some(net) = p.link.net.as_ref() {
            match net.parted {
                Some(t) => notes.push((format!("OUT OF STEP WITH THE OTHER GAME SINCE {}", clock(t)), BAD)),
                None => {
                    let s = format!("IN STEP {}", clock(net.agreed));
                    fb.text(&s, BTN_LAG.x - 4 - Framebuffer::text_width(&s, 1), 3, TEXT_DIM, Some(OUTLINE), 1);
                }
            }
            if net.waiting >= 8 {
                notes.push(("WAITING FOR THE OTHER PLAYER".into(), GOLD));
            }
        }
        if e.split_at[me] > 0 && a.tick < e.split_at[me] + 15 * 4 {
            notes.push(("YOUR NETWORK SPLIT".into(), GOLD));
        }
        if a.over.is_none() {
            if a.king_slowed(me) {
                notes.push((format!("YOUR KING IS SLOWED: UNDER {KING_NEEDS} PX LEFT STANDING"), BAD));
            }
            if a.king_slowed(1 - me) {
                notes.push(("THE ENEMY KING IS SLOWED".into(), GOOD));
            }
        }
        for (i, (text, col)) in notes.iter().enumerate() {
            fb.text_centered(text, f.w / 2, 14 + i as i32 * 9, *col, Some(OUTLINE), 1);
        }
        if bar.is_none() {
            bar = if let Some(i) = hovered {
                let mut lines = ship_lines(a, i);
                // One of yours that can be sent, or a factory: how to pick it.
                if a.ships[i].commandable(me) && !p.picked.contains(&i) {
                    lines.push(SELECT_HINT.into());
                } else if a.ships[i].rallies(me) && !p.picked.contains(&i) {
                    lines.push(FACTORY_HINT.into());
                }
                Some(lines)
            } else if let Some(at) = at {
                a.rock_near(at, if p.tactical { TAC_SCALE } else { 2 }).map(|px| vec![format!("ROCK: {} PX LEFT. ANYONE CAN MINE IT, AND A SEED CAN GROW A FACTORY OUT OF IT.", a.rock_blob(px).len())])
            } else {
                self.match_tip().map(|t| wrap(&t, 49))
            };
        }
        // With ships selected and nothing else to say: what to do with them.
        if bar.is_none() && !p.picked.is_empty() {
            let factories = p.picked.iter().filter(|&&i| a.ships[i].rallies(me)).count();
            bar = Some(wrap(&selected_line(p.picked.len() - factories, factories), 49));
        }
        if let Some(lines) = bar {
            let h = lines.len() as i32 * 9 + 6;
            let r = Rect::new(0, f.h - h, f.w, h);
            ui::panel(fb, r);
            for (k, l) in lines.iter().enumerate() {
                fb.text(l, 4, r.y + 4 + k as i32 * 9, bar_col, None, 1);
            }
        }
        if p.ended() {
            let r = OVER;
            ui::panel(fb, r);
            let gone = p.link.net.as_ref().and_then(|n| n.gone.clone()).unwrap_or_default();
            let (head, col, why) = match a.over {
                Some(Some(w)) if w == me => ("YOU WIN", team_col(me, 2), "THE ENEMY KING IS LOOSE PIXELS"),
                Some(Some(w)) => ("YOU LOSE", team_col(w, 2), "YOUR KING IS LOOSE PIXELS"),
                Some(None) => ("DRAW", TEXT, "NOBODY'S KING FELL IN 40 MINUTES"),
                // No result: the other machine went away with the match unfinished.
                None => ("NO RESULT", TEXT, gone.as_str()),
            };
            fb.text_centered(head, r.x + r.w / 2, r.y + 10, col, Some(OUTLINE), 2);
            let why = wrap(why, 31);
            for (i, l) in why.iter().enumerate() {
                fb.text_centered(l, r.x + r.w / 2, r.y + 32 + i as i32 * 9, TEXT, None, 1);
            }
            fb.text_centered(&format!("AFTER {}", clock(a.tick)), r.x + r.w / 2, r.y + 35 + why.len() as i32 * 9, TEXT_DIM, None, 1);
            for (k, (b, label)) in OVER_BTNS.iter().zip(["AGAIN", "YARD"]).enumerate() {
                if k == 1 || p.link.net.is_none() {
                    ui::button(fb, *b, label, b.contains(m.0, m.1), false);
                }
            }
        }

        // Sidebar.
        ui::panel(fb, Rect::new(304, 0, 96, 300));
        let k = e.king[me];
        let bank = a.bank(k);
        fb.text(&format!("BANK {bank}"), 308, 4, GOLD, None, 1);
        // What nodes cut off from the king hold: yours, but not yours to plant with.
        let cut = cut_off(a, me);
        if cut > 0 {
            let s = format!("+{cut}");
            fb.text(&s, 396 - Framebuffer::text_width(&s, 1), 4, TEXT_DIM, None, 1);
        }
        for i in 0..SLOTS {
            let r = card_rect(i);
            let hov = r.contains(m.0, m.1);
            let busy = e.queue[me].first().is_some_and(|j| j.slot == i);
            let repeats = e.queue[me].iter().any(|j| j.slot == i && j.repeat);
            let picked = p.planting == Some(i);
            if picked || busy {
                fb.rect_bevel(r.x, r.y, r.w, r.h, PANEL_DEEP, PANEL_LO, PANEL_HI);
                fb.rect_outline(r.x + 1, r.y + 1, r.w - 2, r.h - 2, if picked { GOLD } else { ACCENT });
            } else if hov {
                fb.rect_bevel(r.x, r.y, r.w, r.h, PANEL_HI, WHITE, PANEL);
            } else {
                ui::panel(fb, r);
            }
            // A card the king keeps printing wears a blue frame, as in Toy 1 (and its job wears
            // the loop sign in the queue).
            if repeats {
                fb.rect_outline(r.x, r.y, r.w, r.h, ACCENT_HI);
            }
            let lit = hov && !(busy || picked);
            // Its key first, on the left; then its icon, its name, and its price at the right end.
            fb.text(&CARD_KEYS[i].to_string(), r.x + CARD_KEY_X, r.y + 3, if lit { PANEL_LO } else { ACCENT }, None, 1);
            let Some(d) = e.deck[me].get(i).filter(|d| !d.is_empty()) else {
                fb.text("EMPTY", r.x + CARD_NAME_X, r.y + 3, TEXT_FAINT, None, 1);
                continue;
            };
            let card = e.cards[me][i];
            // A seeded card you can't pay for yet is greyed, with its price in red.
            let short = card.ok && card.seeded && bank < card.price();
            draw_icon(fb, d, Rect::new(r.x + CARD_ICON_X, r.y + 1, 11, 11), me);
            // The price, in factory orange for a seed, keeps its room: a long name gives way.
            let what = format!("{}", card.price());
            let px = r.x + r.w - 3 - Framebuffer::text_width(&what, 1);
            let name: String = d.name.chars().take(ROW_NAME.min(((px - 2 - (r.x + CARD_NAME_X)) / 6) as usize)).collect();
            fb.text(&name, r.x + CARD_NAME_X, r.y + 3, if !card.ok { BAD } else if short { TEXT_DIM } else if lit { PANEL_DEEP } else { TEXT }, None, 1);
            fb.text(&what, px, r.y + 3, if short { BAD } else if card.seeded { FACTORY } else if lit { PANEL_LO } else { TEXT_DIM }, None, 1);
        }
        // The king's queue: what it's printing first, then what waits, each as its card's icon.
        // The bar is how much of the first is paid.
        let q = &e.queue[me];
        match q.first() {
            Some(first) => {
                let shown = jobs_shown(q.len());
                for (k, j) in q.iter().take(shown).enumerate() {
                    let r = job_rect(k);
                    // The icon in the top of its box, and under it the loop sign if it repeats.
                    fb.rect_fill(r.x, r.y, r.w, r.h, SPACE);
                    if let Some(d) = e.deck[me].get(j.slot) {
                        draw_icon(fb, d, Rect::new(r.x, r.y, r.w, r.h - 3), me);
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
        // The way back to the king, above the map.
        let on_note = (306..400).contains(&m.0) && (VIEW_NOTE_Y - 1..VIEW_NOTE_Y + 8).contains(&m.1);
        fb.text("K: YOUR KING", 308, VIEW_NOTE_Y, if on_note { TEXT } else { TEXT_DIM }, None, 1);
        p.draw_minimap(fb);
        for t in 0..2 {
            let y = 274 + t as i32 * 10;
            let ships = a.ships.iter().filter(|s| s.alive && s.team == t && s.role == Role::Ship && s.grow.is_none() && !s.node).count();
            let factories = a.ships.iter().filter(|s| s.alive && s.team == t && s.role == Role::Ship && s.node).count();
            fb.text(team_name(t), 308, y, team_col(t, 2), None, 1);
            let count = format!("{factories}F {ships}S");
            fb.text(&count, 340, y, TEXT, None, 1);
            if a.king_slowed(t) && 340 + Framebuffer::text_width(&count, 1) <= 371 {
                fb.text("SLOW", 373, y, if t == me { BAD } else { GOOD }, None, 1);
            }
        }
    }

    fn match_tip(&self) -> Option<String> {
        let m = self.mouse;
        let p = self.play.as_ref()?;
        let e = p.arena.eco.as_ref()?;
        let (me, net) = (p.link.me, p.link.net.is_some());
        if BTN_LAG.contains(m.0, m.1) {
            let ms = p.link.delay * 1000 / 15;
            return Some(if net {
                format!("[L] LAG: YOUR COMMANDS ARE APPLIED {} TICKS ({ms} MS) AFTER YOU GIVE THEM, SO THE OTHER GAME HAS THEM IN TIME. TOO LITTLE FOR THE NETWORK AND THE MATCH STUTTERS.", p.link.delay)
            } else {
                format!("[L] LAG: YOUR COMMANDS ARE APPLIED {} TICKS ({ms} MS) AFTER YOU GIVE THEM. A NETWORK MATCH NEEDS SOME: TURN THIS UP TO FEEL HOW ONE WOULD PLAY.", p.link.delay)
            });
        }
        if BTN_STANCE.contains(m.0, m.1) {
            return Some(STANCE_TIP.into());
        }
        if !p.picked.is_empty() && BTN_HOME.contains(m.0, m.1) {
            return Some(HOME_TIP.into());
        }
        if net && (BTN_PAUSE.contains(m.0, m.1) || BTN_SPEED.contains(m.0, m.1)) {
            return Some(format!("{NO_PAUSE}."));
        }
        if let Some(i) = (0..SLOTS).find(|&i| card_rect(i).contains(m.0, m.1)) {
            let d = e.deck[me].get(i).filter(|d| !d.is_empty());
            let Some(d) = d else {
                return Some(format!("{}AN EMPTY SLOT. A TEAM HOLDS NINE DESIGNS: DRAW ONE HERE IN THE YARD, OR TAKE ONE FROM THE LIBRARY.", card_key(i)));
            };
            let c = e.cards[me][i];
            let builds: Vec<String> = d.build.iter().filter_map(|&b| e.deck[me].get(b)).map(|o| o.name.clone()).collect();
            let what = if c.seeded {
                format!("{}{}: {} PX, PAID IN FULL WHEN YOU PICK ITS ROCK. ITS SEED FLIES THERE AND GROWS IT OUT OF THE ROCK. WHERE THE ROCK HOLDS UNDER {} PX, OR IN OPEN SPACE (CTRL), IT DRAWS THE REST FROM ITS NETWORK AS IT GROWS.", card_key(i), d.name, c.price(), c.count)
            } else {
                format!("{}{}: YOUR KING PRINTS IT FOR {} PX, A PIXEL AT A TIME. RIGHT-CLICK: ONE GOES ON THE END OF THE QUEUE, ON REPEAT.", card_key(i), d.name, c.cost)
            };
            let builds = if builds.is_empty() { String::new() } else { format!(" ITS FACTORY PIXELS PRINT {}.", builds.join(", ")) };
            return Some(what + &builds + &flying(d) + &format!(" CTRL+{} SELECTS EVERY ONE YOU HAVE.", CARD_KEYS[i]));
        }
        if let Some((k, j)) = p.job_at(m.0, m.1) {
            let name = e.deck[me].get(j.slot).map_or("?", |d| d.name.as_str());
            let state = if k == 0 { format!("YOUR KING IS PRINTING IT: {} OF {} PX PAID", j.paid, j.cost) } else { format!("NUMBER {} IN YOUR KING'S QUEUE", k + 1) };
            let key = if k + 1 == e.queue[me].len() { " OR [R]" } else { "" };
            let repeats = if j.repeat { " IT REPEATS: WHEN ONE IS DONE, ANOTHER GOES TO THE BACK OF THE QUEUE." } else { "" };
            return Some(format!("{name}: {state}.{repeats} CLICK DROPS IT; WHAT IT HAD PAID COMES BACK. RIGHT-CLICK{key}: REPEAT ON OR OFF."));
        }
        if BTN_CANCEL.contains(m.0, m.1) && !e.queue[me].is_empty() {
            return Some("[BACKSPACE] DROP WHAT YOUR KING IS PRINTING. YOU GET BACK WHAT IT HAD PAID.".into());
        }
        if BTN_PAUSE.contains(m.0, m.1) {
            return Some("[SPACE] PAUSE OR RUN THE MATCH.".into());
        }
        if BTN_SPEED.contains(m.0, m.1) {
            return Some("[F] 1X, 2X OR 4X. THE FRAMES STAY AT 15 FPS; FAST SKIPS AHEAD.".into());
        }
        if BTN_TAC.contains(m.0, m.1) {
            return Some("[TAB] THE WHOLE MAP IN THE FIELD. YOU CAN PLANT, SELECT AND SEND FROM IT.".into());
        }
        if BTN_LEAVE.contains(m.0, m.1) {
            return Some("GIVE UP THE MATCH AND GO BACK TO THE YARD.".into());
        }
        if (306..400).contains(&m.0) && (VIEW_NOTE_Y - 1..VIEW_NOTE_Y + 8).contains(&m.1) {
            return Some("[K] SELECTS YOUR KING. AGAIN STRAIGHT AWAY: THE VIEW GOES TO IT.".into());
        }
        if MINI.contains(m.0, m.1) {
            return Some("THE WHOLE MAP. CLICK OR DRAG ON IT TO MOVE THE VIEW; WASD OR THE ARROWS SCROLL IT. WHILE PLANTING, A CLICK PICKS THE ROCK. WITH SHIPS SELECTED, A RIGHT-CLICK SENDS THEM.".into());
        }
        if (306..400).contains(&m.0) && (0..12).contains(&m.1) {
            let cut = cut_off(&p.arena, me);
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
        ui::button(fb, super::keys::BTN_KEYS, "F3: KEYS", super::keys::BTN_KEYS.contains(self.mouse.0, self.mouse.1), false);
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

/// What the match screen says while a seed is about to be sent into open space: what the seed
/// costs, and that the growing costs as much again. Henry, 2026-11-12: twice the price "might be
/// a bit counterintuitive to players", so it's said where the seed is placed. And 2026-11-13:
/// say the one rule, that it draws from its network, and none of what follows from it.
/// What the bar says under one of your ships that can be selected.
const SELECT_HINT: &str = "CLICK SELECTS IT. DOUBLE-CLICK: ALL OF ITS TYPE.";
/// What the bar says under one of your factories.
const FACTORY_HINT: &str = "CLICK SELECTS IT, TO SET WHERE ITS SHIPS GATHER.";
/// What the LET GO button says when pointed at.
const HOME_TIP: &str = "[G] THE SHIPS YOU'VE SELECTED ARE UNSENT AGAIN: THEY DO WHAT THE BUTTON TO THE LEFT SAYS.";
/// What the stance button says when pointed at.
const STANCE_TIP: &str = "[T] WHAT EVERY SHIP YOU HAVEN'T SENT ANYWHERE DOES: GUARD WHERE IT WAS BUILT, OR ATTACK THE NEAREST ENEMY.";

/// The stance button's face.
fn stance_label(attacking: bool) -> &'static str {
    if attacking {
        "ALL: ATTACK"
    } else {
        "ALL: GUARD"
    }
}

/// What the bar says while things are selected and nothing else is being pointed at.
fn selected_line(ships: usize, factories: usize) -> String {
    match (ships, factories) {
        (n, 0) => format!("{n} SELECTED. RIGHT-CLICK SENDS THEM. G LETS GO."),
        (0, 1) => "A FACTORY. A RIGHT-CLICK SETS WHERE ITS NEW SHIPS GATHER. ON THE FACTORY ITSELF: ROUND IT AGAIN.".to_string(),
        (0, n) => format!("{n} FACTORIES. A RIGHT-CLICK SETS WHERE THEIR NEW SHIPS GATHER."),
        (n, k) => format!("{} SELECTED. A RIGHT-CLICK SENDS THE SHIPS, AND SETS WHERE THE FACTORIES' NEW SHIPS GATHER.", n + k),
    }
}

/// How far a ship's guns reach, for where it stands in a formation: the gun it lines up if it
/// has one, else its cannon, else its missiles, else its flak.
fn gun_range(st: &Stats) -> i32 {
    if st.aims() {
        st.fwd_reach()
    } else if st.cannon > 0 {
        CANNON_RANGE
    } else if st.missile > 0 {
        MISSILE_RANGE
    } else {
        FLAK_RANGE
    }
}

/// Where each of a group sent to a place stands once it's there: in ranks across the way they
/// came, the middle of the block on the place. There are four lines, front to back: ships with
/// only short guns (flak), ships with longer ones, what doesn't fight (the unarmed, and
/// miners), and a king. Each line has ranks of its own, as even as they'll go, with ships of a
/// type together. The block is about three times as wide as a square of the same ships would
/// be deep. Neighbours stand `FORM_GAP` apart, so nobody at rest pushes anybody. One ship
/// alone stands on the place itself.
fn formation(a: &Arena, ships: &[usize], at: (i32, i32)) -> Vec<(i32, i32)> {
    if ships.len() < 2 {
        return vec![at; ships.len()];
    }
    let n = ships.len() as i32;
    let mid = (ships.iter().map(|&i| a.ships[i].x).sum::<i32>() / n, ships.iter().map(|&i| a.ships[i].y).sum::<i32>() / n);
    // The way they face: the nearest of the eight compass points to the way they'll come. Sent
    // to where they already are, they face the middle of the map.
    let (dx, dy) = (at.0 - mid.0, at.1 - mid.1);
    let face = if dx.abs() < 8 && dy.abs() < 8 {
        (if at.0 * 2 > a.w { -1 } else { 1 }, 0)
    } else if dx.abs() >= 2 * dy.abs() {
        (dx.signum(), 0)
    } else if dy.abs() >= 2 * dx.abs() {
        (0, dy.signum())
    } else {
        (dx.signum(), dy.signum())
    };
    let side = (-face.1, face.0);
    let width = |i: usize| 2 * a.ships[i].half() + FORM_GAP;
    let line = |i: usize| {
        let s = &a.ships[i];
        if s.role == Role::King {
            3
        } else if s.miner() || !s.st.armed() {
            2
        } else if gun_range(&s.st) > FLAK_RANGE {
            1
        } else {
            0
        }
    };
    let mut order: Vec<usize> = ships.to_vec();
    order.sort_by_key(|&i| (line(i), gun_range(&a.ships[i].st), a.ships[i].slot, i));
    // The ranks. No rank is wider than the budget, and each line of battle splits into as few
    // as that allows, as evenly as they'll go.
    let budget = isqrt(3 * order.iter().map(|&i| width(i) * width(i)).sum::<i32>()).max(1);
    let mut rows: Vec<Vec<usize>> = Vec::new();
    for l in 0..4 {
        let of: Vec<usize> = order.iter().copied().filter(|&i| line(i) == l).collect();
        let wide: i32 = of.iter().map(|&i| width(i)).sum();
        if wide == 0 {
            continue;
        }
        let ranks = (wide + budget - 1) / budget;
        let each = (wide + ranks - 1) / ranks;
        let mut len = 0;
        rows.push(Vec::new());
        for &i in &of {
            if len > 0 && len + width(i) > each {
                rows.push(Vec::new());
                len = 0;
            }
            rows.last_mut().unwrap().push(i);
            len += width(i);
        }
    }
    let depth: Vec<i32> = rows.iter().map(|r| r.iter().map(|&i| width(i)).max().unwrap_or(0)).collect();
    let deep: i32 = depth.iter().sum();
    // Each spot: whose it is by rank, how far back from the middle of the block, and how far
    // along its rank from the middle.
    let mut spots: Vec<(usize, i32, i32)> = Vec::new();
    let mut back = 0;
    for (r, row) in rows.iter().enumerate() {
        let wide: i32 = row.iter().map(|&i| width(i)).sum();
        let mut along = -wide / 2;
        for &i in row {
            spots.push((i, back + depth[r] / 2 - deep / 2, along + width(i) / 2));
            along += width(i);
        }
        back += depth[r];
    }
    // Ships of one type are as good as each other: of the spots their type has, each takes the
    // one on its own side of the group, so they don't cross over on the way.
    let kind = |i: usize| (a.ships[i].role == Role::King, a.ships[i].slot);
    let across = |i: usize| (a.ships[i].x - mid.0) * side.0 + (a.ships[i].y - mid.1) * side.1;
    let mut out = vec![at; ships.len()];
    let mut kinds: Vec<(bool, usize)> = ships.iter().map(|&i| kind(i)).collect();
    kinds.sort();
    kinds.dedup();
    for k in kinds {
        let mut theirs: Vec<(i32, i32)> = spots.iter().filter(|s| kind(s.0) == k).map(|s| (s.2, s.1)).collect();
        theirs.sort();
        let mut who: Vec<usize> = (0..ships.len()).filter(|&j| kind(ships[j]) == k).collect();
        who.sort_by_key(|&j| (across(ships[j]), ships[j]));
        for (j, (along, back)) in who.into_iter().zip(theirs) {
            out[j] = (at.0 - back * face.0 + along * side.0, at.1 - back * face.1 + along * side.1);
        }
    }
    out
}

fn space_warning(name: &str, price: i32, grows: i32) -> String {
    format!("SEND {name}'S SEED HERE FOR {price} PX. NO ROCK FEEDS IT IN OPEN SPACE: IT DRAWS THE {grows} PX IT GROWS FROM ITS NETWORK, SO IT COSTS ABOUT TWICE.")
}

/// How a card's ships fly at what they fight and how many leave together, for its hover text:
/// " THESE ORBIT, AND LEAVE IN GROUPS OF 3." Nothing for a design with no engines.
fn flying(d: &Design) -> String {
    let Some(style) = d.flies() else { return String::new() };
    let how = match style {
        Style::Runs => "MAKE ATTACK RUNS",
        Style::Kite => "KITE",
        Style::Orbit => "ORBIT",
    };
    if d.group > 1 {
        format!(" THESE {how}, AND LEAVE IN GROUPS OF {}.", d.group)
    } else {
        format!(" THESE {how}.")
    }
}

/// Pixels held by a side's nodes that aren't in its king's network.
fn cut_off(a: &Arena, me: usize) -> i32 {
    let e = a.eco.as_ref().unwrap();
    let k = e.king[me];
    let net = e.net.get(k).copied().unwrap_or(usize::MAX);
    (0..a.ships.len()).filter(|&j| j != k && a.ships[j].alive && a.ships[j].team == me && e.net.get(j).is_some_and(|&n| n != net || n == usize::MAX)).map(|j| a.ships[j].store).sum()
}

impl Play {
    /// The job in the king's queue whose icon is under a screen point, and where it stands.
    fn job_at(&self, x: i32, y: i32) -> Option<(usize, Job)> {
        let q = &self.arena.eco.as_ref()?.queue[self.link.me];
        (0..jobs_shown(q.len())).find(|&k| job_rect(k).contains(x, y)).map(|k| (k, q[k]))
    }

    /// Whether the match is finished: a king has fallen, the time is up, or the other machine
    /// has gone.
    fn ended(&self) -> bool {
        self.arena.over.is_some() || self.link.net.as_ref().is_some_and(|n| n.gone.is_some())
    }

    fn look_at_king(&mut self) {
        let e = self.arena.eco.as_ref().unwrap();
        let k = &self.arena.ships[e.king[self.link.me]];
        self.cam = ((k.x - FIELD.w / 2).clamp(0, MW - FIELD.w), (k.y - FIELD.h / 2).clamp(0, MH - FIELD.h));
    }

    /// Centres the view on a world point.
    fn look_at(&mut self, (wx, wy): (i32, i32)) {
        self.cam = ((wx - FIELD.w / 2).clamp(0, MW - FIELD.w), (wy - FIELD.h / 2).clamp(0, MH - FIELD.h));
    }

    /// Whether a point is on one of the buttons along the field's top edge.
    fn on_top_button(&self, x: i32, y: i32) -> bool {
        BTN_LAG.contains(x, y) || BTN_STANCE.contains(x, y) || (!self.picked.is_empty() && BTN_HOME.contains(x, y))
    }

    /// Whether a ship is in view. On the whole map everything is.
    fn in_view(&self, i: usize) -> bool {
        let s = &self.arena.ships[i];
        self.tactical || ((self.cam.0..self.cam.0 + FIELD.w).contains(&s.x) && (self.cam.1..self.cam.1 + FIELD.h).contains(&s.y))
    }

    /// What's selected becomes `got`. With `add` (Shift) it gains them instead.
    fn take(&mut self, got: Vec<usize>, add: bool) {
        if add {
            let more: Vec<usize> = got.into_iter().filter(|i| !self.picked.contains(i)).collect();
            self.picked.extend(more);
        } else {
            self.picked = got;
        }
    }

    /// Selects your warships, which is everything of yours that can be sent and isn't a miner:
    /// the ones in view, or with `all` every one you have.
    fn select_fleet(&mut self, all: bool, add: bool) {
        let me = self.link.me;
        let got = (0..self.arena.ships.len()).filter(|&i| self.arena.ships[i].sendable(me) && !self.arena.ships[i].miner() && (all || self.in_view(i))).collect();
        self.take(got, add);
    }

    /// The world point a point on the field shows, in whichever view. In the whole-map view, a
    /// point off the map's picture counts as the nearest point of it.
    fn field_world(&self, (x, y): (i32, i32)) -> (i32, i32) {
        if self.tactical {
            let (x, y) = (x.clamp(TAC.x, TAC.x + TAC.w - 1), y.clamp(TAC.y, TAC.y + TAC.h - 1));
            ((x - TAC.x) * TAC_SCALE + TAC_SCALE / 2, (y - TAC.y) * TAC_SCALE + TAC_SCALE / 2)
        } else {
            (x + self.cam.0, y + self.cam.1)
        }
    }

    /// The thing at a point on the field that passes `ok`: on the field, the nearest whose
    /// outline (and two pixels round it) is under the point; on the whole map, where a ship is
    /// as small as a pixel, the nearest within its own size or a map pixel.
    fn ship_under(&self, at: (i32, i32), ok: impl Fn(&Ship) -> bool) -> Option<usize> {
        let a = &self.arena;
        let (wx, wy) = self.field_world(at);
        let on = |s: &Ship| {
            if self.tactical {
                cheb(s.x, s.y, wx, wy) <= s.half().max(TAC_SCALE)
            } else {
                let (l, t, r, b) = s.bbox();
                wx >= l - 2 && wy >= t - 2 && wx <= r + 2 && wy <= b + 2
            }
        };
        (0..a.ships.len()).filter(|&i| a.ships[i].alive && ok(&a.ships[i]) && on(&a.ships[i])).min_by_key(|&i| (dist2(a.ships[i].x, a.ships[i].y, wx, wy), i))
    }

    /// Selects by a press and a release on the field. Released where it was pressed, it's a
    /// click: the ship or the factory of yours under the pointer, or your king, or nobody. A
    /// second click on the same one straight after (or a click with Ctrl) takes every one of
    /// that design in view, which on the whole map is all of them. Dragged, it's a box: the
    /// warships of yours inside it; with none, the miners and the king inside it; with none
    /// of those either, the factories (so a box round a fleet leaves the miners working, the
    /// king where it is, and the base alone). With `add` (Shift) a click adds to what's
    /// selected, or takes out, and a box adds.
    fn select(&mut self, from: (i32, i32), to: (i32, i32), same_design: bool, add: bool, frame: u32) {
        let (a, me) = (&self.arena, self.link.me);
        let to = (to.0.clamp(FIELD.x, FIELD.x + FIELD.w - 1), to.1.clamp(FIELD.y, FIELD.y + FIELD.h - 1));
        let mine = |i: usize| a.ships[i].selectable(me);
        let fleet = |i: usize| a.ships[i].sendable(me) && !a.ships[i].miner();
        let mut clicked = None;
        let got: Vec<usize> = if (to.0 - from.0).abs() <= 2 && (to.1 - from.1).abs() <= 2 {
            match self.ship_under(to, |s| s.selectable(me)) {
                Some(i) => {
                    let double = self.last_click.is_some_and(|(j, f)| j == i && frame.saturating_sub(f) <= DOUBLE);
                    clicked = Some((i, frame));
                    if double || same_design {
                        (0..a.ships.len()).filter(|&j| mine(j) && self.in_view(j) && a.ships[j].slot == a.ships[i].slot && a.ships[j].role == a.ships[i].role).collect()
                    } else {
                        vec![i]
                    }
                }
                None => Vec::new(),
            }
        } else {
            let (p0, p1) = (self.field_world(from), self.field_world(to));
            let (l, r, t, b) = (p0.0.min(p1.0), p0.0.max(p1.0), p0.1.min(p1.1), p0.1.max(p1.1));
            let mut inside: Vec<usize> = (0..a.ships.len()).filter(|&i| mine(i) && (l..=r).contains(&a.ships[i].x) && (t..=b).contains(&a.ships[i].y)).collect();
            if inside.iter().any(|&i| fleet(i)) {
                inside.retain(|&i| fleet(i));
            } else if inside.iter().any(|&i| a.ships[i].commandable(me)) {
                inside.retain(|&i| a.ships[i].commandable(me));
            }
            inside
        };
        let picked = if !add {
            got
        } else if got.len() == 1 && clicked.is_some() && self.picked.contains(&got[0]) {
            // A Shift-click on a ship that's selected takes it out.
            self.picked.iter().copied().filter(|&i| i != got[0]).collect()
        } else {
            let mut all = self.picked.clone();
            all.extend(got.into_iter().filter(|i| !self.picked.contains(i)));
            all
        };
        self.picked = picked;
        self.last_click = clicked;
    }

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
        for s in a.ships.iter().filter(|s| s.alive && s.team == self.link.me && s.role == Role::Seed) {
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
        for s in a.ships.iter().filter(|s| s.alive && s.team == self.link.me && s.role == Role::Seed) {
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
    fn a_card_says_how_its_ships_fly_and_how_many_leave_together() {
        use crate::app::{App, Ev};
        let mut pg = Pg::new(None, None);
        pg.lib = starter_deck();
        (pg.lib[0].style, pg.lib[0].group) = (Some(Style::Orbit), 3);
        pg.start_match(false, 3);
        pg.help = false;
        let mut tip = |r: Rect| {
            pg.event(Ev::Move(r.x + 4, r.y + 4));
            pg.match_tip().unwrap_or_default()
        };
        // The fighter; the lancer as it's drawn; the outpost, which doesn't fly, says nothing of
        // it. Each ends with the key that selects every one of it.
        assert!(tip(card_rect(0)).ends_with(" THESE ORBIT, AND LEAVE IN GROUPS OF 3. CTRL+Z SELECTS EVERY ONE YOU HAVE."), "{}", tip(card_rect(0)));
        assert!(tip(card_rect(1)).ends_with(" THESE MAKE ATTACK RUNS. CTRL+X SELECTS EVERY ONE YOU HAVE."), "{}", tip(card_rect(1)));
        assert!(!tip(card_rect(4)).contains("THESE") && tip(card_rect(4)).ends_with(" CTRL+B SELECTS EVERY ONE YOU HAVE."), "{}", tip(card_rect(4)));
    }

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
        // The stance: one button by the clock, or T, for every ship you haven't sent anywhere.
        // The fighter, the lancer and the frigate all guard, then all attack, then all guard.
        // The miner goes on mining and the outpost holding. (A card had a tag for its own order
        // until 2026-11-13.)
        let stance = |pg: &Pg| pg.play.as_ref().unwrap().arena.attacking(0);
        assert_eq!((eco(&pg)[..5].to_vec(), stance(&pg)), (vec![Order::Guard, Order::Guard, Order::Guard, Order::Harvest, Order::Hold], false));
        click(&mut pg, BTN_STANCE, false);
        assert_eq!((eco(&pg)[..5].to_vec(), stance(&pg)), (vec![Order::Hunt, Order::Hunt, Order::Hunt, Order::Harvest, Order::Hold], true));
        assert!(queue(&pg).is_empty());
        pg.event(Ev::Key(Key::Letter('T')));
        pg.tick();
        assert_eq!((eco(&pg)[..5].to_vec(), stance(&pg)), (vec![Order::Guard, Order::Guard, Order::Guard, Order::Harvest, Order::Hold], false));
        pg.event(Ev::Move(BTN_STANCE.x + 3, BTN_STANCE.y + 3));
        assert!(pg.match_tip().is_some_and(|t| t.starts_with("[T] ")));
        // The right end of a card, where its tag was, is the card: a click there prints it.
        let end = Rect::new(card_rect(0).x + 80, card_rect(0).y + 2, 8, 6);
        click(&mut pg, end, false);
        assert_eq!((eco(&pg)[0], queue(&pg).len()), (Order::Guard, 1));
        click(&mut pg, BTN_CANCEL, false);
        // The numbers change no card's order and print nothing, with Shift or without.
        for n in 1..=9 {
            pg.event(Ev::Key(Key::ShiftNum(n)));
            pg.event(Ev::Key(Key::Num(n)));
        }
        pg.tick();
        assert_eq!((eco(&pg)[0], queue(&pg).len()), (Order::Guard, 0));
        // Right-click a card: it repeats. Again: it stops. Backspace or X: the king drops it.
        click(&mut pg, card_rect(1), true);
        assert!(queue(&pg).len() == 1 && queue(&pg)[0].slot == 1 && queue(&pg)[0].repeat);
        click(&mut pg, card_rect(1), true);
        assert!(queue(&pg).len() == 1 && !queue(&pg)[0].repeat);
        pg.event(Ev::Key(Key::Back));
        pg.tick();
        assert!(queue(&pg).is_empty());
        pg.event(Ev::Key(Key::Letter('C')));
        pg.tick();
        assert_eq!(queue(&pg).len(), 1);
        click(&mut pg, BTN_CANCEL, false);
        assert!(queue(&pg).is_empty());
        // The queue, as icons: a click on one drops that job, a right-click sets that job
        // repeating (or stops it), and R does that for the last one.
        let slots = |pg: &Pg| queue(pg).iter().map(|j| (j.slot, j.repeat)).collect::<Vec<_>>();
        // The cards' keys are the bottom row of letters: Z, X, C for the first three.
        for c in ['Z', 'X', 'C'] {
            pg.event(Ev::Key(Key::Letter(c)));
            pg.tick();
        }
        assert_eq!(slots(&pg), vec![(0, false), (1, false), (2, false)]);
        click(&mut pg, job_rect(1), false);
        assert_eq!(slots(&pg), vec![(0, false), (2, false)]);
        click(&mut pg, job_rect(0), true);
        assert_eq!(slots(&pg), vec![(0, true), (2, false)]);
        pg.event(Ev::Key(Key::Letter('R')));
        pg.tick();
        assert_eq!(slots(&pg), vec![(0, true), (2, true)]);
        pg.event(Ev::Key(Key::Letter('R')));
        pg.tick();
        assert_eq!(slots(&pg), vec![(0, true), (2, false)]);
        click(&mut pg, job_rect(0), true);
        assert_eq!(slots(&pg), vec![(0, false), (2, false)]);
        // A right-click on a card puts a job of it on the end of the queue, on repeat, whatever
        // of it is queued already: the fighter asked for first is still one fighter. A number
        // and then R does the same from the keys. A right-click on the card again stops its
        // repeating; the jobs stay.
        click(&mut pg, card_rect(0), true);
        assert_eq!(slots(&pg), vec![(0, false), (2, false), (0, true)]);
        pg.event(Ev::Key(Key::Letter('X')));
        pg.tick();
        pg.event(Ev::Key(Key::Letter('R')));
        pg.tick();
        assert_eq!(slots(&pg), vec![(0, false), (2, false), (0, true), (1, true)]);
        click(&mut pg, card_rect(0), true);
        assert_eq!(slots(&pg), vec![(0, false), (2, false), (0, false), (1, true)]);
        // There's no icon where there's no job, so a click there does nothing.
        click(&mut pg, job_rect(4), false);
        assert_eq!(slots(&pg).len(), 4);
        click(&mut pg, job_rect(2), false);
        click(&mut pg, job_rect(0), false);
        assert_eq!(slots(&pg), vec![(2, false), (1, true)]);
        click(&mut pg, job_rect(0), false);
        pg.event(Ev::Key(Key::Back));
        pg.tick();
        assert!(queue(&pg).is_empty());
        // The lag dial, by its button and by L: a command then waits that many ticks. With the
        // dial at 2, a card's job shows in the queue on the third tick, not the first.
        let lag = |pg: &Pg| pg.play.as_ref().unwrap().link.delay;
        assert_eq!(lag(&pg), 0);
        click(&mut pg, BTN_LAG, false);
        pg.event(Ev::Key(Key::Letter('L')));
        assert_eq!(lag(&pg), 2);
        // A number and R pressed together still mean "this, on repeat", though the job R is
        // for hasn't reached the queue when R is pressed.
        pg.event(Ev::Key(Key::Letter('Z')));
        pg.event(Ev::Key(Key::Letter('R')));
        pg.tick();
        pg.tick();
        assert!(queue(&pg).is_empty(), "the job was queued before the lag was up");
        pg.tick();
        assert_eq!(slots(&pg), vec![(0, true)]);
        while lag(&pg) != 0 {
            pg.event(Ev::Key(Key::Letter('L')));
        }
        pg.event(Ev::Key(Key::Back));
        pg.tick();
        assert!(queue(&pg).is_empty());
        // The shipyard costs more than the bank holds: its card won't start a planting.
        click(&mut pg, card_rect(5), false);
        assert_eq!(pg.play.as_ref().unwrap().planting, None);
        // The whole map: Tab and the button. A click on it is for selecting ships and leaves the
        // view where it is. A click on the small map moves the view and leaves it there, and
        // steering brings it back to the king.
        pg.event(Ev::Key(Key::Tab));
        assert!(pg.play.as_ref().unwrap().tactical);
        click(&mut pg, BTN_TAC, false);
        assert!(!pg.play.as_ref().unwrap().tactical);
        click(&mut pg, BTN_TAC, false);
        let home = pg.play.as_ref().unwrap().cam;
        click(&mut pg, Rect::new(TAC.x + TAC.w - 20, TAC.y + 10, 1, 1), false);
        assert_eq!(pg.play.as_ref().unwrap().cam, home, "a click on the whole map moved the view");
        let far = Rect::new(MINI.x + MINI.w - 8, MINI.y + 6, 1, 1);
        click(&mut pg, far, false);
        let p = pg.play.as_ref().unwrap();
        assert!(p.cam != home && p.cam.0 > MW / 2, "the view didn't move to the click: {:?}", p.cam);
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
        // The view stays where it was put. K selects the king and leaves the view; K again
        // straight away brings the view to the king. So do two clicks on the note above the
        // small map. The keys that used to fly the king scroll the view.
        let away = pg.play.as_ref().unwrap().cam;
        pg.tick();
        assert_eq!(pg.play.as_ref().unwrap().cam, away);
        let me = pg.play.as_ref().unwrap().arena.eco.as_ref().unwrap().king[0];
        pg.event(Ev::Key(Key::Letter('K')));
        assert_eq!((pg.play.as_ref().unwrap().cam, picked(&pg)), (away, vec![me]), "one K moved the view, or didn't select the king");
        // A while later, K again is a first press again.
        for _ in 0..DOUBLE + 2 {
            pg.tick();
        }
        pg.event(Ev::Key(Key::Letter('K')));
        assert_eq!(pg.play.as_ref().unwrap().cam, away, "a second K long after the first moved the view");
        pg.event(Ev::Key(Key::Letter('K')));
        let p = pg.play.as_ref().unwrap();
        let king = &p.arena.ships[p.arena.eco.as_ref().unwrap().king[0]];
        let on_king = ((king.x - FIELD.w / 2).clamp(0, MW - FIELD.w), (king.y - FIELD.h / 2).clamp(0, MH - FIELD.h));
        assert_eq!(p.cam, on_king, "K didn't bring the view back to the king");
        let (kx, ky) = (king.x, king.y);
        pg.hold([true, false, false, true]);
        pg.tick();
        pg.hold([false; 4]);
        let p = pg.play.as_ref().unwrap();
        assert_eq!(p.cam, (on_king.0 + CAM_STEP, (on_king.1 - CAM_STEP).max(0)));
        let king = &p.arena.ships[p.arena.eco.as_ref().unwrap().king[0]];
        assert_eq!((king.x, king.y), (kx, ky), "the keys flew the king");
        click(&mut pg, far, false);
        pg.event(Ev::Key(Key::Esc));
        for _ in 0..DOUBLE + 2 {
            pg.tick();
        }
        let away = pg.play.as_ref().unwrap().cam;
        click(&mut pg, Rect::new(310, VIEW_NOTE_Y, 60, 6), false);
        assert_eq!((pg.play.as_ref().unwrap().cam, picked(&pg)), (away, vec![me]), "one click on the note moved the view, or didn't select the king");
        click(&mut pg, Rect::new(310, VIEW_NOTE_Y, 60, 6), false);
        assert_eq!(pg.play.as_ref().unwrap().cam, on_king, "two clicks on the note didn't bring the view back");
        // A group's number and then K isn't K twice: the king as group 4, the view moved away,
        // then 4 and K.
        pg.ctrl(true);
        pg.event(Ev::Key(Key::Num(4)));
        pg.ctrl(false);
        click(&mut pg, far, false);
        for _ in 0..DOUBLE + 2 {
            pg.tick();
        }
        let away = pg.play.as_ref().unwrap().cam;
        pg.event(Ev::Key(Key::Num(4)));
        pg.event(Ev::Key(Key::Letter('K')));
        assert_eq!((pg.play.as_ref().unwrap().cam, picked(&pg)), (away, vec![me]));
    }

    #[test]
    fn a_network_match_plays_from_either_side_of_the_table() {
        use crate::app::{App, Ev};
        use crate::pg::link::tests::pipes;
        let (a, b, clock, _) = pipes();
        let (mut host, mut join) = (Pg::new(None, None), Pg::new(None, None));
        // Each is sent the other's team, empty slots and all, as the hellos carry it.
        let (theirs, ours) = (join.lib.clone(), host.lib.clone());
        host.start_linked(0, 11, theirs, Box::new(a), Vec::new());
        join.start_linked(1, 11, ours, Box::new(b), Vec::new());
        let frames = |host: &mut Pg, join: &mut Pg, n: u32| {
            for _ in 0..n {
                clock.set(clock.get() + 1);
                host.tick();
                join.tick();
            }
        };
        fn arena(pg: &Pg) -> &Arena {
            &pg.play.as_ref().unwrap().arena
        }
        fn net(pg: &Pg) -> &crate::pg::link::Net {
            pg.play.as_ref().unwrap().link.net.as_ref().unwrap()
        }
        let click = |pg: &mut Pg, r: Rect| {
            let (x, y) = (r.x + r.w / 2, r.y + r.h / 2);
            pg.event(Ev::Move(x, y));
            pg.event(Ev::Press { x, y, right: false });
            pg.event(Ev::Release { x, y, right: false });
        };
        // The joiner plays red, with the same keys and cards: its first card queues a fighter
        // for red's king, on both machines, and its right-click sends red's king and no other.
        // (Red's king is in the middle of the joiner's view, a pixel right of it.)
        join.event(Ev::Key(Key::Letter('Z')));
        for (x, y, right) in [(153, 149, false), (93, 149, true)] {
            join.event(Ev::Move(x, y));
            join.event(Ev::Press { x, y, right });
            join.event(Ev::Release { x, y, right });
        }
        frames(&mut host, &mut join, 30);
        for pg in [&host, &join] {
            let a = arena(pg);
            let e = a.eco.as_ref().unwrap();
            assert_eq!((e.queue[0].len(), e.queue[1].len()), (0, 1), "red's job should be in red's queue on both machines");
            assert!(a.ships[e.king[1]].x < KING_STARTS[1].0 - 20, "red's king didn't fly left");
            assert_eq!((a.ships[e.king[0]].x, a.ships[e.king[0]].y), KING_STARTS[0], "blue's king moved");
        }
        // Each window follows its own king, and the two matches are one match.
        assert!(join.play.as_ref().unwrap().cam.0 > MW / 2 && host.play.as_ref().unwrap().cam.0 < MW / 2);
        assert!(net(&host).agreed >= 15 && net(&join).agreed >= 15 && net(&host).parted.is_none() && net(&join).parted.is_none());
        // No pause and no fast forward: the keys and the buttons leave it running, at 1x.
        host.event(Ev::Key(Key::Space));
        host.event(Ev::Key(Key::Letter('F')));
        click(&mut host, BTN_PAUSE);
        click(&mut host, BTN_SPEED);
        let p = host.play.as_ref().unwrap();
        assert!(p.running && p.speed == 1);
        // Nor does a panel opened over it stop it: the rules, or the quit question.
        let before = arena(&host).tick;
        host.event(Ev::Key(Key::Help));
        assert!(host.help);
        frames(&mut host, &mut join, 15);
        host.event(Ev::Key(Key::Space));
        host.event(Ev::Key(Key::Esc));
        frames(&mut host, &mut join, 15);
        host.event(Ev::Key(Key::Space));
        assert!(!host.help && !host.quit());
        assert!(arena(&host).tick >= before + 25 && arena(&join).tick >= before + 25, "the match stopped under a panel");
        // The lag dial turns, but never to none: a command has to reach the other machine.
        for _ in 0..9 {
            click(&mut host, BTN_LAG);
            assert!(host.play.as_ref().unwrap().link.delay >= 1);
            frames(&mut host, &mut join, 3);
        }
        assert!(net(&host).parted.is_none() && net(&join).parted.is_none());
        // The joiner leaves. The host is told, and its match ends with no result and one way out.
        click(&mut join, BTN_LEAVE);
        assert!(join.play.is_none() && join.screen == Screen::Yard);
        frames(&mut host, &mut join, 5);
        assert!(net(&host).gone.is_some() && host.play.as_ref().unwrap().ended() && arena(&host).over.is_none());
        click(&mut host, OVER_BTNS[0]);
        assert!(host.play.is_some(), "a network match has no AGAIN");
        click(&mut host, OVER_BTNS[1]);
        assert!(host.play.is_none() && host.screen == Screen::Yard);
    }

    #[test]
    fn finding_the_other_machine_is_keys_and_buttons_too() {
        use crate::app::{App, Ev};
        let click = |pg: &mut Pg, r: Rect| {
            let (x, y) = (r.x + r.w / 2, r.y + r.h / 2);
            pg.event(Ev::Press { x, y, right: false });
        };
        let mut pg = Pg::new(None, None);
        pg.help = false;
        // JOIN A MATCH asks for an address, with this machine's own already typed. It's edited
        // with the keyboard, and Esc or BACK gives up.
        pg.menu = true;
        click(&mut pg, MENU_BTNS[4]);
        assert!(!pg.menu && matches!(&pg.lobby, Some(Lobby::Address(a)) if a == "127.0.0.1"));
        for _ in 0..3 {
            pg.event(Ev::Key(Key::Back));
        }
        for c in "9.42".chars() {
            // A digit comes as a key and as a character; only the character is typed.
            if let Some(d) = c.to_digit(10) {
                pg.event(Ev::Key(Key::Num(d as u8)));
            }
            pg.event(Ev::Char(c));
        }
        assert!(matches!(&pg.lobby, Some(Lobby::Address(a)) if a == "127.0.9.42"), "the address typed wrong");
        assert_eq!(pg.slot, 0, "typing an address picked a design in the yard behind it");
        pg.event(Ev::Key(Key::Esc));
        assert!(pg.lobby.is_none() && !pg.quit());
        pg.join(None);
        click(&mut pg, LOBBY_BTNS[1]);
        assert!(pg.lobby.is_none());
        // Enter, or JOIN, starts connecting (to nothing, here: it fails, and says so).
        pg.join(None);
        for _ in 0..9 {
            pg.event(Ev::Key(Key::Back));
        }
        for c in "0.0.0.0:1".chars() {
            pg.event(Ev::Char(c));
        }
        click(&mut pg, LOBBY_BTNS[0]);
        assert!(matches!(pg.lobby, Some(Lobby::Joining { .. })));
        let started = std::time::Instant::now();
        while !matches!(pg.lobby, Some(Lobby::Failed(_))) {
            assert!(started.elapsed() < std::time::Duration::from_secs(8), "connecting to nothing never failed");
            pg.tick();
            std::thread::sleep(std::time::Duration::from_millis(5));
        }
        pg.event(Ev::Key(Key::Esc));
        assert!(pg.lobby.is_none());
        // The panels' pieces sit inside them.
        for (panel, parts) in [(MENU, &MENU_BTNS[..]), (LOBBY, &LOBBY_BTNS[..]), (LOBBY, &[LOBBY_FIELD][..])] {
            for r in parts {
                assert!(r.x >= panel.x + 6 && r.y >= panel.y && r.x + r.w <= panel.x + panel.w - 6 && r.y + r.h <= panel.y + panel.h - 6, "{r:?} is outside {panel:?}");
            }
        }
        assert!(BTN_LAG.x + BTN_LAG.w <= FIELD.w && BTN_LAG.y + BTN_LAG.h <= 13, "the lag dial runs into the notes under it");
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

    #[test]
    fn nine_cards_fit_above_the_queue_each_with_its_key_its_icon_its_name_and_its_price() {
        for i in 0..SLOTS {
            let r = card_rect(i);
            assert!(r.y + r.h <= QUEUE.y, "card {} runs into the king's queue", i + 1);
            assert!(i == 0 || card_rect(i - 1).y + card_rect(i - 1).h <= r.y, "card {} overlaps the one above", i + 1);
            // One line of text, with a pixel clear above and below.
            assert!(r.h >= 9);
        }
        // Along a card: the key's letter, then the icon, then the name, each clear of the next,
        // and a name of six letters still fits beside a price of four figures.
        assert!(CARD_KEY_X >= 2 && CARD_KEY_X + 6 <= CARD_ICON_X && CARD_ICON_X + 11 + 1 <= CARD_NAME_X);
        let price = card_rect(0).w - 3 - Framebuffer::text_width("9999", 1);
        assert!((price - 2 - CARD_NAME_X) / 6 >= 6);
        // A key for every card, no two alike, and none of them a key the match has another use
        // for.
        assert_eq!(CARD_KEYS.len(), SLOTS);
        for (i, k) in CARD_KEYS.iter().enumerate() {
            assert!(!CARD_KEYS[..i].contains(k) && !"QGTKRFLPWASD".contains(*k), "{k}");
            assert_eq!(card_key(i), format!("[{k}] "));
        }
    }

    #[test]
    fn an_empty_slot_is_a_card_that_prints_nothing() {
        use crate::app::{App, Ev};
        // Your team: the starter deck and two empty slots.
        let mut pg = Pg::new(None, None);
        pg.start_match(false, 5);
        pg.help = false;
        let queue = |pg: &Pg| pg.play.as_ref().unwrap().arena.eco.as_ref().unwrap().queue[0].len();
        let click = |pg: &mut Pg, r: Rect, right: bool| {
            let (x, y) = (r.x + r.w / 2, r.y + r.h / 2);
            pg.event(Ev::Move(x, y));
            pg.event(Ev::Press { x, y, right });
            pg.event(Ev::Release { x, y, right });
            pg.tick();
        };
        for n in [8, 9] {
            click(&mut pg, card_rect(n as usize - 1), false);
            click(&mut pg, card_rect(n as usize - 1), true);
            assert_eq!(queue(&pg), 0, "slot {n} is empty and went on the king's queue");
            // The card says what it is.
            pg.event(Ev::Move(card_rect(n as usize - 1).x + 20, card_rect(n as usize - 1).y + 5));
            assert!(pg.match_tip().unwrap().contains("AN EMPTY SLOT"));
        }
        // Against the starter deck, which has seven: the same keys, and nothing breaks.
        let mut pg = Pg::new(None, None);
        pg.start_match(true, 5);
        pg.help = false;
        for n in [8, 9] {
            click(&mut pg, card_rect(n as usize - 1), false);
            click(&mut pg, card_rect(n as usize - 1), true);
            assert_eq!(queue(&pg), 0);
        }
        // And the eighth and ninth cards are cards like any other when their slots are filled:
        // clicked, or on the two keys after M, the comma and the full stop.
        let mut pg = Pg::new(None, None);
        pg.lib[7] = starter_deck()[1].clone();
        pg.lib[8] = starter_deck()[0].clone();
        pg.start_match(false, 5);
        pg.help = false;
        let slots = |pg: &Pg| pg.play.as_ref().unwrap().arena.eco.as_ref().unwrap().queue[0].iter().map(|j| j.slot).collect::<Vec<_>>();
        click(&mut pg, card_rect(8), false);
        assert_eq!(slots(&pg), vec![8], "the ninth card didn't print");
        pg.event(Ev::Key(Key::Letter(',')));
        pg.tick();
        pg.event(Ev::Key(Key::Letter('.')));
        pg.tick();
        assert_eq!(slots(&pg), vec![8, 7, 8]);
        // Each card's hover text starts with its key.
        pg.event(Ev::Move(card_rect(8).x + 20, card_rect(8).y + 5));
        assert!(pg.match_tip().is_some_and(|t| t.starts_with("[.] FIGHTER:")), "{:?}", pg.match_tip());
        pg.event(Ev::Move(card_rect(7).x + 20, card_rect(7).y + 5));
        assert!(pg.match_tip().is_some_and(|t| t.starts_with("[,] LANCER:")), "{:?}", pg.match_tip());
        pg.event(Ev::Move(card_rect(0).x + 20, card_rect(0).y + 5));
        assert!(pg.match_tip().is_some_and(|t| t.starts_with("[Z] ")), "{:?}", pg.match_tip());
    }

    #[test]
    fn a_seed_goes_to_open_space_with_ctrl_and_is_told_what_that_costs() {
        use crate::app::{App, Ev};
        let mut pg = Pg::new(None, None);
        pg.start_match(true, 3);
        pg.help = false;
        let click = |pg: &mut Pg, (x, y): (i32, i32)| {
            pg.event(Ev::Move(x, y));
            pg.event(Ev::Press { x, y, right: false });
            pg.event(Ev::Release { x, y, right: false });
            pg.tick();
        };
        let mid = |r: Rect| (r.x + r.w / 2, r.y + r.h / 2);
        let seeds = |pg: &Pg| pg.play.as_ref().unwrap().arena.ships.iter().filter(|s| s.alive && s.team == 0 && s.role == Role::Seed && s.sow).count();
        // Pixels to spend, counted as mined so the books still balance.
        {
            let a = &mut pg.play.as_mut().unwrap().arena;
            let k = a.eco.as_ref().unwrap().king[0];
            a.ships[k].store += 500;
            a.score[0].mined += 500;
        }
        // Two spots in open space beside the king, and where they are on the screen.
        let spots: Vec<(i32, i32)> = {
            let a = &pg.play.as_ref().unwrap().arena;
            let (kx, ky) = KING_STARTS[0];
            [(100, 0), (0, -100), (70, -70), (-70, -70), (40, -120)].iter().map(|d| (kx + d.0, ky + d.1)).filter(|&p| a.sow_links(0, 4, p).is_ok()).collect()
        };
        assert!(spots.len() >= 2, "the test needs two open spots by the king");
        let on_screen = |pg: &Pg, w: (i32, i32)| {
            let c = pg.play.as_ref().unwrap().cam;
            (w.0 - c.0, w.1 - c.1)
        };
        // The outpost's card. A plain click on open space is still not a rock: nothing is sent.
        click(&mut pg, mid(card_rect(4)));
        assert_eq!(pg.play.as_ref().unwrap().planting, Some(4));
        let at = on_screen(&pg, spots[0]);
        assert!(FIELD.contains(at.0, at.1));
        click(&mut pg, at);
        assert_eq!((pg.play.as_ref().unwrap().planting, seeds(&pg)), (Some(4), 0));
        // With Ctrl held, the same click sends the seed there, and the card is let go.
        pg.ctrl(true);
        let mut fb = Framebuffer::new();
        pg.event(Ev::Move(at.0, at.1));
        pg.draw(&mut fb);
        click(&mut pg, at);
        pg.ctrl(false);
        for _ in 0..4 {
            pg.tick();
        }
        assert_eq!((pg.play.as_ref().unwrap().planting, seeds(&pg)), (None, 1));
        // Somewhere it can't go, nothing is sent and the card stays picked; a spot that will
        // take it then does.
        click(&mut pg, mid(card_rect(4)));
        pg.ctrl(true);
        let far = on_screen(&pg, (KING_STARTS[0].0 + 60, KING_STARTS[0].1 - 140));
        let bad = {
            let a = &pg.play.as_ref().unwrap().arena;
            let c = pg.play.as_ref().unwrap().cam;
            (0..FIELD.w).step_by(7).flat_map(|x| (14..FIELD.h - 60).step_by(7).map(move |y| (x, y))).find(|&(x, y)| a.sow_links(0, 4, (x + c.0, y + c.1)).is_err()).unwrap_or(far)
        };
        click(&mut pg, bad);
        assert_eq!((pg.play.as_ref().unwrap().planting, seeds(&pg)), (Some(4), 1));
        pg.event(Ev::Move(bad.0, bad.1));
        pg.draw(&mut fb);
        let at2 = on_screen(&pg, spots[1]);
        click(&mut pg, at2);
        pg.ctrl(false);
        for _ in 0..4 {
            pg.tick();
        }
        assert_eq!((pg.play.as_ref().unwrap().planting, seeds(&pg)), (None, 2));
        // There's no button for it: the card's own hover text names the key.
        let card = mid(card_rect(4));
        pg.event(Ev::Move(card.0, card.1));
        let tip = pg.match_tip().unwrap();
        assert!(tip.contains("IN OPEN SPACE (CTRL)"), "{tip}");
        // The bar gives the one rule, that what grows with no rock under it is drawn from its
        // network, and that this costs about twice. With the dearest design there could be, it
        // still fits a bar on the field.
        let warn = space_warning("WWWWWWWWWW", 9999, 9998);
        assert!(warn.contains("TWICE") && warn.contains("IT DRAWS THE 9998 PX IT GROWS FROM ITS NETWORK"), "{warn}");
        assert!(!warn.contains("BANK"), "{warn}");
        assert!(wrap(&format!("{warn} RIGHT-CLICK OR ESC CANCELS."), 49).len() <= 7);
    }
    /// A match with the help put away, and blue ships set down by its king: three fighters and
    /// a miner close together, a lancer and a frigate apart, and a red fighter among the
    /// fighters. The view is on blue's king, so the map point (x, y) is on the field at
    /// (x, y - 750). Returns their places: fighters, miner, lancer, frigate, the red one.
    fn fleet_by_the_king() -> (Pg, [usize; 3], usize, usize, usize, usize) {
        let mut pg = Pg::new(None, None);
        pg.start_match(true, 3);
        pg.help = false;
        let d = starter_deck();
        let a = &mut pg.play.as_mut().unwrap().arena;
        let f = [a.spawn(0, 0, &d[0], 60, 800, 1, false), a.spawn(0, 0, &d[0], 80, 800, 1, false), a.spawn(0, 0, &d[0], 60, 820, 1, false)];
        let miner = a.spawn(0, 3, &d[3], 80, 820, 1, false);
        let lancer = a.spawn(0, 1, &d[1], 200, 800, 1, false);
        let frigate = a.spawn(0, 2, &d[2], 200, 840, 1, false);
        let foe = a.spawn(1, 0, &d[0], 70, 810, 3, false);
        assert_eq!(pg.play.as_ref().unwrap().cam, (0, 750));
        (pg, f, miner, lancer, frigate, foe)
    }

    fn drag(pg: &mut Pg, from: (i32, i32), to: (i32, i32), right: bool) {
        use crate::app::{App, Ev};
        pg.event(Ev::Move(from.0, from.1));
        pg.event(Ev::Press { x: from.0, y: from.1, right });
        pg.event(Ev::Move(to.0, to.1));
        pg.event(Ev::Release { x: to.0, y: to.1, right });
    }

    fn picked(pg: &Pg) -> Vec<usize> {
        let mut v = pg.play.as_ref().unwrap().picked.clone();
        v.sort();
        v
    }

    #[test]
    fn a_box_selects_your_warships_inside_it_and_a_click_selects_the_ship_under_it() {
        use crate::app::{App, Ev};
        let (mut pg, f, miner, lancer, frigate, foe) = fleet_by_the_king();
        // A box round the fighters, the miner and the red fighter: your fighters, and nothing else.
        drag(&mut pg, (40, 30), (100, 90), false);
        assert_eq!(picked(&pg), f.to_vec());
        // Dragged the other way, the same.
        drag(&mut pg, (100, 90), (40, 30), false);
        assert_eq!(picked(&pg), f.to_vec());
        // A box with only the miner in it selects the miner.
        drag(&mut pg, (75, 65), (90, 80), false);
        assert_eq!(picked(&pg), vec![miner]);
        // A click on a ship selects that one, and a click on empty space or on an enemy, nobody.
        drag(&mut pg, (200, 50), (200, 50), false);
        assert_eq!(picked(&pg), vec![lancer]);
        drag(&mut pg, (250, 200), (251, 201), false);
        assert_eq!(picked(&pg), Vec::<usize>::new());
        drag(&mut pg, (200, 90), (200, 90), false);
        assert_eq!(picked(&pg), vec![frigate]);
        let red = &pg.play.as_ref().unwrap().arena.ships[foe];
        let at = (red.x, red.y - 750);
        drag(&mut pg, at, at, false);
        assert!(!picked(&pg).contains(&foe));
        // With Ctrl, a click selects every ship of that design.
        pg.ctrl(true);
        drag(&mut pg, (60, 50), (60, 50), false);
        pg.ctrl(false);
        assert_eq!(picked(&pg), f.to_vec());
        // Q: every warship you have. The miner isn't one.
        pg.event(Ev::Key(Key::Letter('Q')));
        assert_eq!(picked(&pg), vec![f[0], f[1], f[2], lancer, frigate]);
        // Your king is selected by a click. A box round the king and the fleet takes the fleet
        // and leaves the king; a box round the king alone takes the king.
        let king = pg.play.as_ref().unwrap().arena.eco.as_ref().unwrap().king[0];
        drag(&mut pg, (150, 150), (150, 150), false);
        assert_eq!(picked(&pg), vec![king]);
        drag(&mut pg, (30, 30), (230, 190), false);
        assert_eq!(picked(&pg), vec![f[0], f[1], f[2], lancer, frigate]);
        drag(&mut pg, (130, 130), (170, 170), false);
        assert_eq!(picked(&pg), vec![king]);
        // Esc lets the selection go, and doesn't ask about quitting.
        pg.event(Ev::Key(Key::Esc));
        assert!(picked(&pg).is_empty() && !pg.confirm_quit);
        pg.event(Ev::Key(Key::Esc));
        assert!(pg.confirm_quit);
    }

    #[test]
    fn a_right_click_sends_the_selected_ships_there_together_and_let_go_frees_them() {
        use crate::app::{App, Ev};
        let (mut pg, f, _, lancer, frigate, _) = fleet_by_the_king();
        let sent = |pg: &Pg, i: usize| (pg.play.as_ref().unwrap().arena.ships[i].sent, pg.play.as_ref().unwrap().arena.ships[i].pace);
        let vmax = |pg: &Pg, i: usize| pg.play.as_ref().unwrap().arena.ships[i].st.vmax;
        // With nothing selected a right-click on the field sends nobody.
        drag(&mut pg, (250, 200), (250, 200), true);
        pg.tick();
        pg.tick();
        assert!([f[0], lancer, frigate].iter().all(|&i| sent(&pg, i).0.is_none()));
        // The lancer and the frigate, boxed, then a right-click: both are sent to that place on
        // the map, at the pace of the slower.
        drag(&mut pg, (190, 40), (215, 100), false);
        assert_eq!(picked(&pg), vec![lancer, frigate]);
        drag(&mut pg, (250, 200), (250, 200), true);
        pg.tick();
        pg.tick();
        let slow = vmax(&pg, lancer).min(vmax(&pg, frigate));
        assert!(slow < vmax(&pg, lancer), "the frigate is the slower");
        assert_eq!((sent(&pg, lancer).1, sent(&pg, frigate).1), (slow, slow));
        // Each has its own spot there, side by side across the way they came (they came from
        // above), with the place between them.
        let (l, g) = (sent(&pg, lancer).0.unwrap(), sent(&pg, frigate).0.unwrap());
        assert!(l.1 == 950 && g.1 == 950 && l.0 != g.0 && (l.0 + g.0 - 500).abs() <= 6, "the lancer to {l:?}, the frigate to {g:?}");
        assert!(sent(&pg, f[0]).0.is_none(), "a ship that wasn't selected was sent");
        // They stay selected. A right-click on the small map sends them too.
        assert_eq!(picked(&pg), vec![lancer, frigate]);
        let on_map = (MINI.x + 1 + 40, MINI.y + 1 + 20);
        drag(&mut pg, on_map, on_map, true);
        pg.tick();
        pg.tick();
        let to = pg.play.as_ref().unwrap().world_point(on_map.0, on_map.1).unwrap();
        let (l, g) = (sent(&pg, lancer).0.unwrap(), sent(&pg, frigate).0.unwrap());
        assert!(cheb(l.0, l.1, to.0, to.1) <= 12 && cheb(g.0, g.1, to.0, to.1) <= 12 && l != g, "{l:?} and {g:?}, sent to {to:?}");
        // One ship sent alone flies flat out.
        drag(&mut pg, (200, 50), (200, 50), false);
        drag(&mut pg, (250, 100), (250, 100), true);
        pg.tick();
        pg.tick();
        assert_eq!(sent(&pg, lancer), (Some((250, 850)), 0));
        // The LET GO button is there while ships are selected, and frees them. So does X.
        let home = (BTN_HOME.x + 3, BTN_HOME.y + 3);
        pg.event(Ev::Move(home.0, home.1));
        assert!(pg.match_tip().is_some_and(|t| t.starts_with("[G]")));
        drag(&mut pg, home, home, false);
        pg.tick();
        pg.tick();
        assert_eq!((sent(&pg, lancer), picked(&pg)), ((None, 0), vec![lancer]));
        assert!(sent(&pg, frigate).0.is_some(), "a ship that wasn't selected was let go");
        drag(&mut pg, (200, 90), (200, 90), false);
        pg.event(Ev::Key(Key::Letter('G')));
        pg.tick();
        pg.tick();
        assert_eq!(sent(&pg, frigate), (None, 0));
        // The king, selected and sent: it flies there with no key held, and stops when it's there.
        let king = pg.play.as_ref().unwrap().arena.eco.as_ref().unwrap().king[0];
        let at = |pg: &Pg| (pg.play.as_ref().unwrap().arena.ships[king].x, pg.play.as_ref().unwrap().arena.ships[king].y);
        drag(&mut pg, (150, 150), (150, 150), false);
        assert_eq!((picked(&pg), at(&pg)), (vec![king], (150, 900)));
        drag(&mut pg, (200, 150), (200, 150), true);
        for _ in 0..3 {
            pg.tick();
        }
        assert!(sent(&pg, king).0 == Some((200, 900)) && at(&pg).0 > 150, "{:?} at {:?}", sent(&pg, king), at(&pg));
        for _ in 0..60 {
            pg.tick();
        }
        assert!((at(&pg).0 - 200).abs() <= 1 && at(&pg).1 == 900 && sent(&pg, king).0.is_none(), "{:?} {:?}", at(&pg), sent(&pg, king));
        // Sent again, the keys that used to fly it don't take it back: they scroll the view, and
        // the king goes on.
        drag(&mut pg, (60, 150), (60, 150), true);
        pg.tick();
        pg.tick();
        assert!(sent(&pg, king).0.is_some());
        let cam = pg.play.as_ref().unwrap().cam;
        pg.hold([false, false, false, true]);
        pg.tick();
        pg.tick();
        pg.hold([false; 4]);
        assert!(sent(&pg, king).0.is_some() && pg.play.as_ref().unwrap().cam.0 > cam.0);
        pg.event(Ev::Key(Key::Letter('K')));
        pg.event(Ev::Key(Key::Letter('G')));
        pg.tick();
        pg.tick();
        assert!(sent(&pg, king).0.is_none());
        // With nothing selected the button isn't there: a click where it would be starts nothing.
        pg.event(Ev::Key(Key::Esc));
        pg.event(Ev::Move(home.0, home.1));
        assert!(pg.match_tip().map_or(true, |t| !t.starts_with("[G]")));
        // With a card picked for planting, a right-click lets the card go and sends nobody.
        pg.event(Ev::Key(Key::Letter('Q')));
        pg.play.as_mut().unwrap().planting = Some(4);
        drag(&mut pg, (250, 200), (250, 200), true);
        pg.tick();
        pg.tick();
        assert_eq!((pg.play.as_ref().unwrap().planting, sent(&pg, f[0]).0), (None, None));
        // And while a card is picked, a press on the field plants: it doesn't select.
        pg.event(Ev::Key(Key::Esc));
        pg.play.as_mut().unwrap().planting = Some(4);
        drag(&mut pg, (40, 30), (100, 90), false);
        assert!(picked(&pg).is_empty());
    }

    #[test]
    fn ships_are_selected_and_sent_on_the_whole_map_too_and_the_dead_drop_out() {
        use crate::app::{App, Ev};
        let (mut pg, f, _, lancer, _, _) = fleet_by_the_king();
        pg.event(Ev::Key(Key::Tab));
        let on = |x: i32, y: i32| (TAC.x + x / TAC_SCALE, TAC.y + y / TAC_SCALE);
        // A box round the fighters on the whole map.
        let (a, b) = (on(40, 780), on(100, 840));
        drag(&mut pg, (a.0 - 1, a.1 - 1), (b.0 + 1, b.1 + 1), false);
        assert_eq!(picked(&pg), f.to_vec());
        // A click on the lancer, which is a pixel or so there.
        drag(&mut pg, on(200, 800), on(200, 800), false);
        assert_eq!(picked(&pg), vec![lancer]);
        // A right-click on the whole map sends it to that place.
        let to = on(700, 500);
        drag(&mut pg, to, to, true);
        pg.tick();
        pg.tick();
        let want = pg.play.as_ref().unwrap().world_point(to.0, to.1).unwrap();
        assert_eq!(pg.play.as_ref().unwrap().arena.ships[lancer].sent, Some(want));
        assert!((want.0 - 700).abs() <= TAC_SCALE && (want.1 - 500).abs() <= TAC_SCALE);
        // A ship that dies drops out of the selection.
        pg.event(Ev::Key(Key::Letter('Q')));
        assert!(picked(&pg).contains(&f[0]));
        pg.play.as_mut().unwrap().arena.ships[f[0]].alive = false;
        pg.tick();
        assert!(!picked(&pg).contains(&f[0]) && picked(&pg).contains(&lancer));
    }

    #[test]
    fn the_words_for_selecting_fit_where_they_are_shown() {
        // The bar is 49 letters wide; a tip in it may run to seven lines.
        assert!(selected_line(999, 0).len() <= 49 && SELECT_HINT.len() <= 49 && FACTORY_HINT.len() <= 49);
        assert!([selected_line(0, 1), selected_line(0, 99), selected_line(999, 99)].iter().all(|l| wrap(l, 49).len() <= 2));
        assert!(wrap(HOME_TIP, 49).len() <= 7 && wrap(STANCE_TIP, 49).len() <= 7);
        // The stance button comes first after the clock, and both its faces fit it.
        assert!(BTN_STANCE.x >= 4 + Framebuffer::text_width("99:59", 1) + 2 && BTN_STANCE.x + BTN_STANCE.w < BTN_HOME.x && BTN_STANCE.y + BTN_STANCE.h <= 13);
        assert!([true, false].iter().all(|&on| Framebuffer::text_width(stance_label(on), 1) + 4 <= BTN_STANCE.w));
        // In a network match the note that the two games are in step sits before the lag dial:
        // LET GO ends before it starts.
        assert!(BTN_HOME.x + BTN_HOME.w <= BTN_LAG.x - 4 - Framebuffer::text_width("IN STEP 99:59", 1));
        // The button is on the field, clear of the clock before it and the lag dial after, and
        // its label fits.
        assert!(BTN_HOME.x >= 4 + Framebuffer::text_width("99:59", 1) + 2 && BTN_HOME.x + BTN_HOME.w < BTN_LAG.x && BTN_HOME.y + BTN_HOME.h <= 13);
        assert!(Framebuffer::text_width("LET GO", 1) + 4 <= BTN_HOME.w);
        // The rules say how.
        assert!(MATCH_HELP.iter().any(|(h, t)| *h == "FLEET" && t.contains("DOUBLE-CLICK A TYPE") && t.contains("RIGHT-CLICK SENDS THEM THERE TO FIGHT")));
        assert!(MATCH_HELP.iter().any(|(h, t)| *h == "UNSENT" && t.contains("GUARD WHERE THEY WERE BUILT") && t.contains("THEY ALL ATTACK")));
        assert!(MATCH_HELP.iter().any(|(h, t)| *h == "FACTORY" && t.contains("ITS NEW SHIPS GATHER THERE")));
        assert!(MATCH_HELP.iter().any(|(h, t)| *h == "VIEW" && t.contains("WASD OR ARROWS SCROLL") && t.contains("K TWICE: YOUR KING")));
        assert!(MATCH_HELP.iter().any(|(h, t)| *h == "KEYS" && t.contains("F3 LISTS EVERY KEY")));
    }
    fn kill(pg: &mut Pg, i: usize) {
        pg.play.as_mut().unwrap().arena.ships[i].alive = false;
    }

    /// A letter key, with Ctrl or Shift held or neither.
    fn letter(pg: &mut Pg, c: char, ctrl: bool, shift: bool) {
        use crate::app::{App, Ev};
        pg.ctrl(ctrl);
        pg.shift(shift);
        pg.event(Ev::Key(Key::Letter(c)));
        pg.ctrl(false);
        pg.shift(false);
    }

    #[test]
    fn q_takes_the_warships_in_view_ctrl_q_every_one_and_ctrl_with_a_cards_key_every_ship_of_that_card() {
        use crate::app::{App, Ev};
        let (mut pg, f, miner, lancer, frigate, foe) = fleet_by_the_king();
        kill(&mut pg, foe);
        let d = starter_deck();
        // One more fighter, far from the rest and out of view.
        let far = pg.play.as_mut().unwrap().arena.spawn(0, 0, &d[0], 900, 300, 1, false);
        let queued = |pg: &Pg| pg.play.as_ref().unwrap().arena.eco.as_ref().unwrap().queue[0].len();
        // Q: the warships in view. The miner isn't one, and the far fighter isn't in view.
        letter(&mut pg, 'Q', false, false);
        assert_eq!(picked(&pg), vec![f[0], f[1], f[2], lancer, frigate]);
        // Ctrl+Q: every one, wherever it is.
        letter(&mut pg, 'Q', true, false);
        assert_eq!(picked(&pg), vec![f[0], f[1], f[2], lancer, frigate, far]);
        // On the whole map everything is in view.
        pg.event(Ev::Key(Key::Esc));
        pg.event(Ev::Key(Key::Tab));
        letter(&mut pg, 'Q', false, false);
        assert_eq!(picked(&pg), vec![f[0], f[1], f[2], lancer, frigate, far]);
        pg.event(Ev::Key(Key::Tab));
        // Scrolled away from them all, Q takes nobody. With Shift it adds, so then it keeps
        // what's selected.
        pg.hold([false, false, false, true]);
        for _ in 0..40 {
            pg.tick();
        }
        pg.hold([false; 4]);
        assert!(cam(&pg).0 > 210);
        letter(&mut pg, 'Q', false, true);
        assert_eq!(picked(&pg).len(), 6);
        letter(&mut pg, 'Q', false, false);
        assert!(picked(&pg).is_empty());
        // Ctrl with a card's key: every ship of that card on the map. Nothing is printed.
        letter(&mut pg, 'Z', true, false);
        assert_eq!(picked(&pg), vec![f[0], f[1], f[2], far]);
        letter(&mut pg, 'X', true, false);
        assert_eq!(picked(&pg), vec![lancer]);
        // With Shift too, they join what's selected.
        letter(&mut pg, 'C', true, true);
        assert_eq!(picked(&pg), vec![lancer, frigate]);
        letter(&mut pg, 'V', true, false);
        assert_eq!(picked(&pg), vec![miner]);
        // A card you have none of: it says so, and what's selected stays. An empty slot's key
        // says nothing.
        letter(&mut pg, 'B', true, false);
        assert_eq!(picked(&pg), vec![miner]);
        assert!(pg.notice.as_ref().is_some_and(|(s, _)| s == "YOU HAVE NO OUTPOST TO SELECT"), "{:?}", pg.notice);
        pg.notice = None;
        letter(&mut pg, '.', true, false);
        assert!(picked(&pg) == vec![miner] && pg.notice.is_none());
        pg.tick();
        pg.tick();
        assert_eq!(queued(&pg), 0, "Ctrl and a card's key printed the card");
        // Without Ctrl the key prints, and selects nothing.
        letter(&mut pg, 'Z', false, false);
        pg.tick();
        pg.tick();
        assert_eq!((queued(&pg), picked(&pg)), (1, vec![miner]));
        // A right-click on the stance button, which is on the field, sends nobody.
        drag(&mut pg, (BTN_STANCE.x + 5, BTN_STANCE.y + 5), (BTN_STANCE.x + 5, BTN_STANCE.y + 5), true);
        pg.tick();
        pg.tick();
        assert!(pg.play.as_ref().unwrap().arena.ships[miner].sent.is_none());
    }

    #[test]
    fn a_factory_is_selected_by_a_click_and_a_right_click_sets_where_its_new_ships_gather() {
        use crate::app::{App, Ev};
        let (mut pg, f, _, lancer, frigate, foe) = fleet_by_the_king();
        kill(&mut pg, foe);
        let d = starter_deck();
        // An outpost of yours at (230, 960): on the field at (230, 210).
        let post = pg.play.as_mut().unwrap().arena.spawn(0, 4, &d[4], 230, 960, 1, false);
        let ship = |pg: &Pg, i: usize| {
            let s = &pg.play.as_ref().unwrap().arena.ships[i];
            (s.sent, s.rally)
        };
        assert!(pg.play.as_ref().unwrap().arena.ships[post].rallies(0));
        // A click selects it.
        drag(&mut pg, (230, 210), (230, 210), false);
        assert_eq!(picked(&pg), vec![post]);
        // A right-click on the field: that's where its new ships gather. It isn't sent anywhere.
        drag(&mut pg, (100, 250), (100, 250), true);
        pg.tick();
        pg.tick();
        assert_eq!(ship(&pg, post), (None, Some((100, 1000))));
        // It's drawn: a flag standing on the place.
        let mut fb = Framebuffer::new();
        pg.event(Ev::Move(350, 290));
        pg.draw(&mut fb);
        assert!((0..6).all(|k| fb.get(100, 250 - k) == GOOD) && fb.get(102, 245) == GOOD, "no flag at the rally point");
        // A right-click on the factory itself: no rally point, and no flag.
        drag(&mut pg, (232, 212), (232, 212), true);
        pg.tick();
        pg.tick();
        assert_eq!(ship(&pg, post), (None, None));
        pg.event(Ev::Move(350, 290));
        pg.draw(&mut fb);
        assert!(fb.get(100, 247) != GOOD);
        // A box round the fleet, the king and the factory takes the fleet. A box round the
        // factory alone takes the factory.
        drag(&mut pg, (30, 30), (250, 230), false);
        assert_eq!(picked(&pg), vec![f[0], f[1], f[2], lancer, frigate]);
        drag(&mut pg, (215, 195), (245, 225), false);
        assert_eq!(picked(&pg), vec![post]);
        // A box round the king and the factory, with no warship in it, takes the king.
        let king = pg.play.as_ref().unwrap().arena.eco.as_ref().unwrap().king[0];
        drag(&mut pg, (130, 130), (245, 225), false);
        assert_eq!(picked(&pg), vec![king]);
        drag(&mut pg, (215, 195), (245, 225), false);
        // With a fighter added (Shift and a click), a right-click sends the fighter there and
        // sets the factory's rally point there.
        pg.shift(true);
        drag(&mut pg, (60, 50), (60, 50), false);
        pg.shift(false);
        assert_eq!(picked(&pg), vec![f[0], post]);
        drag(&mut pg, (150, 60), (150, 60), true);
        pg.tick();
        pg.tick();
        assert_eq!((ship(&pg, f[0]).0, ship(&pg, post)), (Some((150, 810)), (None, Some((150, 810)))));
        // Let go frees the fighter and leaves the rally point.
        pg.event(Ev::Key(Key::Letter('G')));
        pg.tick();
        pg.tick();
        assert_eq!((ship(&pg, f[0]).0, ship(&pg, post).1), (None, Some((150, 810))));
        // A group keeps a factory, and Ctrl with the outpost's key takes every outpost.
        pg.ctrl(true);
        pg.event(Ev::Key(Key::Num(3)));
        pg.ctrl(false);
        pg.event(Ev::Key(Key::Esc));
        pg.event(Ev::Key(Key::Num(3)));
        assert_eq!(picked(&pg), vec![f[0], post]);
        letter(&mut pg, 'B', true, false);
        assert_eq!(picked(&pg), vec![post]);
        // A right-click on a ship puts the rally point where that ship is, not where the pointer
        // was on it.
        let red = pg.play.as_mut().unwrap().arena.spawn(1, 0, &d[0], 120, 880, 3, false);
        drag(&mut pg, (122, 131), (122, 131), true);
        pg.tick();
        pg.tick();
        assert_eq!(ship(&pg, post).1, Some((120, 880)));
        kill(&mut pg, red);
        // The words the bar has for it.
        assert!(selected_line(0, 1).starts_with("A FACTORY. A RIGHT-CLICK SETS WHERE ITS NEW SHIPS GATHER."));
    }

    #[test]
    fn ships_sent_together_stand_in_ranks_across_the_way_they_came_and_stand_still() {
        use crate::app::App;
        let (mut pg, f, miner, lancer, frigate, foe) = fleet_by_the_king();
        kill(&mut pg, foe);
        let d = starter_deck();
        let king = pg.play.as_ref().unwrap().arena.eco.as_ref().unwrap().king[0];
        // Nine more fighters and three more lancers, in a clump by the others.
        let mut fighters = f.to_vec();
        let mut lancers = vec![lancer];
        {
            let a = &mut pg.play.as_mut().unwrap().arena;
            for n in 0..9 {
                fighters.push(a.spawn(0, 0, &d[0], 60 + 12 * (n % 3), 850 + 12 * (n / 3), 1, false));
            }
            for n in 0..3 {
                lancers.push(a.spawn(0, 1, &d[1], 120, 800 + 14 * n, 1, false));
            }
        }
        let a = &pg.play.as_ref().unwrap().arena;
        let clear = |who: &[usize], spots: &[(i32, i32)]| {
            for i in 0..who.len() {
                for j in 0..i {
                    let need = a.ships[who[i]].half() + a.ships[who[j]].half() + 2;
                    assert!(cheb(spots[i].0, spots[i].1, spots[j].0, spots[j].1) >= need, "{:?} and {:?} are nearer than {need}", spots[i], spots[j]);
                }
            }
        };
        // Twelve fighters and four lancers sent east: two ranks of six fighters in front, and
        // the lancers, whose guns reach further, in a rank behind. Every rank is a straight
        // line across the way they came, and the block's middle is on the place.
        let group: Vec<usize> = fighters.iter().chain(&lancers).copied().collect();
        let at = (600, 830);
        let spots = formation(a, &group, at);
        clear(&group, &spots);
        let mut xs: Vec<i32> = spots[..12].iter().map(|p| p.0).collect();
        xs.sort();
        xs.dedup();
        assert_eq!(xs.len(), 2, "the fighters stand in {} ranks: {:?}", xs.len(), &spots[..12]);
        assert!(spots[..12].iter().filter(|p| p.0 == xs[0]).count() == 6);
        let back = spots[12].0;
        assert!(spots[12..].iter().all(|p| p.0 == back) && back < xs[0], "the lancers at {:?}, the fighters at x {xs:?}", &spots[12..]);
        let (top, bottom) = (spots.iter().map(|p| p.1).min().unwrap(), spots.iter().map(|p| p.1).max().unwrap());
        assert!(((top + bottom) / 2 - at.1).abs() <= 4 && ((back + xs[1]) / 2 - at.0).abs() <= 4, "{spots:?}");
        // Ships of a type don't cross over: the fighters highest up now take the highest spots.
        let (hi_now, hi_then) = ((0..12).min_by_key(|&j| (a.ships[group[j]].y, group[j])).unwrap(), (0..12).min_by_key(|&j| (spots[j].1, j)).unwrap());
        assert_eq!(spots[hi_now].1, spots[hi_then].1);
        // Whichever way they're sent (north, on a diagonal, to where they already are), and
        // whatever is sent (the frigate, the miner and the king with them), nobody's spot is
        // near enough a neighbour's to be pushed off it, and a king stands furthest back.
        let all: Vec<usize> = group.iter().copied().chain([frigate, miner, king]).collect();
        let n = all.len() as i32;
        let mid = (all.iter().map(|&i| a.ships[i].x).sum::<i32>() / n, all.iter().map(|&i| a.ships[i].y).sum::<i32>() / n);
        for at in [(600, 830), (110, 300), (500, 400), (mid.0 + 3, mid.1 - 2)] {
            let spots = formation(a, &all, at);
            clear(&all, &spots);
            let (l, r) = (spots.iter().map(|p| p.0).min().unwrap(), spots.iter().map(|p| p.0).max().unwrap());
            let (t, b) = (spots.iter().map(|p| p.1).min().unwrap(), spots.iter().map(|p| p.1).max().unwrap());
            // (On a diagonal the ranks are of different widths, so the box round the block isn't
            // square on the place: it's within a king's width of it.)
            assert!(((l + r) / 2 - at.0).abs() <= 20 && ((t + b) / 2 - at.1).abs() <= 20, "sent to {at:?}, they stand from {l},{t} to {r},{b}");
        }
        let east = formation(a, &all, (600, 830));
        let (frigate_at, miner_at, king_at) = (east[n as usize - 3], east[n as usize - 2], east[n as usize - 1]);
        assert!(east[..n as usize - 1].iter().all(|p| p.0 > king_at.0), "the king isn't at the back: {east:?}");
        // The miner, which doesn't fight, is behind every warship; the frigate, with long guns,
        // stands with the lancers.
        assert!(east[..n as usize - 2].iter().all(|p| p.0 > miner_at.0) && frigate_at.0 == east[12].0, "{east:?}");
        // One ship alone stands on the place itself, and nobody stands nowhere.
        assert_eq!(formation(a, &[lancer], (333, 444)), vec![(333, 444)]);
        assert!(formation(a, &[], (333, 444)).is_empty());
        // Sent with a right-click, they fly there, each to its own spot, and then stand still.
        letter(&mut pg, 'Q', true, false);
        assert_eq!(picked(&pg).len(), 17);
        drag(&mut pg, (280, 120), (280, 120), true);
        for _ in 0..15 * 30 {
            pg.tick();
        }
        let at_rest = |pg: &Pg| picked(pg).iter().map(|&i| (pg.play.as_ref().unwrap().arena.ships[i].x, pg.play.as_ref().unwrap().arena.ships[i].y)).collect::<Vec<_>>();
        let then = at_rest(&pg);
        for (k, &i) in picked(&pg).iter().enumerate() {
            let to = pg.play.as_ref().unwrap().arena.ships[i].sent.unwrap();
            assert!(cheb(then[k].0, then[k].1, to.0, to.1) <= 1, "ship {i} is at {:?}, and was sent to {to:?}", then[k]);
        }
        for _ in 0..30 {
            pg.tick();
            assert_eq!(at_rest(&pg), then, "ships in formation kept moving");
        }
    }

    #[test]
    fn a_card_shows_its_key_before_its_icon_and_the_stance_button_says_what_unsent_ships_do() {
        use crate::app::{App, Ev};
        let mut pg = Pg::new(None, None);
        pg.start_match(true, 3);
        pg.help = false;
        pg.event(Ev::Move(200, 150));
        let mut fb = Framebuffer::new();
        pg.draw(&mut fb);
        let lit = |fb: &Framebuffer, r: Rect, col: u8| (0..r.w * r.h).filter(|k| fb.get(r.x + k % r.w, r.y + k / r.w) == col).count();
        for i in 0..SLOTS {
            // The key's letter, in the card's first few pixels, and nothing of that colour where
            // the order's tag used to be. (A full stop is two pixels.)
            let r = card_rect(i);
            assert!(lit(&fb, Rect::new(r.x + CARD_KEY_X, r.y + 3, 5, 7), ACCENT) >= 2, "card {} shows no key", i + 1);
            assert_eq!(lit(&fb, Rect::new(r.x + 78, r.y + 1, 11, r.h - 2), PANEL_DEEP), 0, "card {} still has a tag", i + 1);
        }
        // The stance button's face is the stance, and it's lit while they attack.
        let face = |pg: &Pg| stance_label(pg.play.as_ref().unwrap().arena.attacking(0));
        assert_eq!(face(&pg), "ALL: GUARD");
        pg.event(Ev::Key(Key::Letter('T')));
        pg.tick();
        pg.tick();
        assert_eq!(face(&pg), "ALL: ATTACK");
        let before = fb.px.clone();
        pg.draw(&mut fb);
        let changed = (0..BTN_STANCE.w * BTN_STANCE.h).filter(|k| {
            let (x, y) = (BTN_STANCE.x + k % BTN_STANCE.w, BTN_STANCE.y + k / BTN_STANCE.w);
            fb.get(x, y) != before[y as usize * 400 + x as usize]
        });
        assert!(changed.count() > 20, "the stance button looks the same attacking as guarding");
    }

    fn cam(pg: &Pg) -> (i32, i32) {
        pg.play.as_ref().unwrap().cam
    }

    #[test]
    fn control_groups_are_set_with_ctrl_called_by_their_number_and_added_to_with_shift() {
        use crate::app::{App, Ev};
        let (mut pg, f, _, lancer, frigate, foe) = fleet_by_the_king();
        kill(&mut pg, foe);
        let queued = |pg: &Pg| pg.play.as_ref().unwrap().arena.eco.as_ref().unwrap().queue[0].len();
        // The fighters as group 1, the lancer and the frigate as group 2.
        drag(&mut pg, (40, 30), (100, 90), false);
        pg.ctrl(true);
        pg.event(Ev::Key(Key::Num(1)));
        pg.ctrl(false);
        drag(&mut pg, (190, 40), (215, 100), false);
        pg.ctrl(true);
        pg.event(Ev::Key(Key::Num(2)));
        pg.ctrl(false);
        // A number calls its group, whatever was selected. One nobody set calls nothing.
        pg.event(Ev::Key(Key::Num(1)));
        assert_eq!(picked(&pg), f.to_vec());
        pg.event(Ev::Key(Key::Num(2)));
        assert_eq!(picked(&pg), vec![lancer, frigate]);
        pg.event(Ev::Key(Key::Num(7)));
        assert_eq!(picked(&pg), vec![lancer, frigate]);
        // Shift and a number: what's selected joins that group.
        pg.event(Ev::Key(Key::ShiftNum(1)));
        pg.event(Ev::Key(Key::Num(1)));
        assert_eq!(picked(&pg), vec![f[0], f[1], f[2], lancer, frigate]);
        // The numbers print no cards any more.
        pg.tick();
        assert_eq!(queued(&pg), 0);
        // Scrolled away, a group's number called twice running takes the view to the group: the
        // lancer and the frigate are at (200, 800) and (200, 840).
        pg.hold([false, false, false, true]);
        for _ in 0..20 {
            pg.tick();
        }
        pg.hold([false; 4]);
        let away = cam(&pg);
        assert!(away.0 >= 100, "the keys didn't scroll the view: {away:?}");
        pg.event(Ev::Key(Key::Num(2)));
        assert_eq!((picked(&pg), cam(&pg)), (vec![lancer, frigate], away));
        pg.event(Ev::Key(Key::Num(2)));
        assert_eq!(cam(&pg), (200 - FIELD.w / 2, 820 - FIELD.h / 2));
        // Called once after a pause, it's only selected.
        pg.hold([false, false, false, true]);
        for _ in 0..20 {
            pg.tick();
        }
        pg.hold([false; 4]);
        let away = cam(&pg);
        pg.event(Ev::Key(Key::Num(2)));
        assert_eq!(cam(&pg), away);
        // A ship that dies leaves its group.
        kill(&mut pg, lancer);
        pg.tick();
        pg.event(Ev::Key(Key::Num(2)));
        assert_eq!(picked(&pg), vec![frigate]);
    }

    #[test]
    fn a_double_click_takes_every_ship_of_that_type_in_view_and_shift_adds_to_the_selection() {
        use crate::app::{App, Ev};
        let (mut pg, f, miner, lancer, frigate, foe) = fleet_by_the_king();
        kill(&mut pg, foe);
        // A fourth fighter, far off the screen.
        let far = pg.play.as_mut().unwrap().arena.spawn(0, 0, &starter_deck()[0], 900, 300, 1, false);
        // A click, and another straight after: every fighter in view, and not the one that isn't.
        drag(&mut pg, (60, 50), (60, 50), false);
        assert_eq!(picked(&pg), vec![f[0]]);
        drag(&mut pg, (60, 50), (60, 50), false);
        assert_eq!(picked(&pg), f.to_vec());
        // Two clicks with a pause between are two clicks.
        for _ in 0..DOUBLE + 2 {
            pg.tick();
        }
        drag(&mut pg, (60, 50), (60, 50), false);
        assert_eq!(picked(&pg), vec![f[0]]);
        // A click on one ship and then on another is no double click.
        drag(&mut pg, (80, 50), (80, 50), false);
        assert_eq!(picked(&pg), vec![f[1]]);
        // On the whole map everything is in view: a double click there takes all four.
        for _ in 0..DOUBLE + 2 {
            pg.tick();
        }
        pg.event(Ev::Key(Key::Tab));
        let at = (TAC.x + 900 / TAC_SCALE, TAC.y + 300 / TAC_SCALE);
        drag(&mut pg, at, at, false);
        drag(&mut pg, at, at, false);
        assert_eq!(picked(&pg), vec![f[0], f[1], f[2], far]);
        pg.event(Ev::Key(Key::Tab));
        for _ in 0..DOUBLE + 2 {
            pg.tick();
        }
        // With Shift, a click adds a ship to what's selected, and a click on one that's selected
        // takes it out. A box adds what's in it.
        drag(&mut pg, (200, 50), (200, 50), false);
        pg.shift(true);
        drag(&mut pg, (200, 90), (200, 90), false);
        assert_eq!(picked(&pg), vec![lancer, frigate]);
        for _ in 0..DOUBLE + 2 {
            pg.tick();
        }
        drag(&mut pg, (200, 50), (200, 50), false);
        assert_eq!(picked(&pg), vec![frigate]);
        drag(&mut pg, (40, 30), (100, 90), false);
        assert_eq!(picked(&pg), vec![f[0], f[1], f[2], frigate]);
        // The miner a box leaves out can be added by a click. (It has flown off to its rock by now.)
        let at = {
            let m = &pg.play.as_ref().unwrap().arena.ships[miner];
            (m.x - cam(&pg).0, m.y - cam(&pg).1)
        };
        assert!(FIELD.contains(at.0, at.1), "the miner has left the view: {at:?}");
        drag(&mut pg, at, at, false);
        assert!(picked(&pg).contains(&miner));
        // An empty click with Shift held keeps what's selected.
        drag(&mut pg, (250, 200), (250, 200), false);
        assert_eq!(picked(&pg).len(), 5);
        pg.shift(false);
        drag(&mut pg, (250, 200), (250, 200), false);
        assert!(picked(&pg).is_empty());
    }

    #[test]
    fn a_right_click_on_a_ship_sends_them_after_it_wherever_it_goes() {
        use crate::app::App;
        let (mut pg, f, _, lancer, frigate, foe) = fleet_by_the_king();
        let ship = |pg: &Pg, i: usize| {
            let s = &pg.play.as_ref().unwrap().arena.ships[i];
            (s.chase, s.sent, (s.x, s.y))
        };
        let near = |a: (i32, i32), b: (i32, i32)| (a.0 - b.0).abs() <= 4 && (a.1 - b.1).abs() <= 4;
        // The lancer, selected, and a right-click on the red fighter at (70, 810).
        drag(&mut pg, (200, 50), (200, 50), false);
        drag(&mut pg, (70, 60), (70, 60), true);
        pg.tick();
        pg.tick();
        let (chase, sent, _) = ship(&pg, lancer);
        assert!(chase == Some(foe) && sent.is_some_and(|p| near(p, ship(&pg, foe).2)), "{chase:?} {sent:?}");
        // The red fighter is somewhere else: so is where the lancer is sent.
        {
            let s = &mut pg.play.as_mut().unwrap().arena.ships[foe];
            (s.x, s.y) = (600, 500);
        }
        pg.tick();
        assert_eq!(ship(&pg, lancer).1, Some((600, 500)));
        // It dies: the order stands where it last was.
        kill(&mut pg, foe);
        pg.tick();
        pg.tick();
        assert_eq!((ship(&pg, lancer).0, ship(&pg, lancer).1), (None, Some((600, 500))));
        // A right-click on one of your own sends them after that: the fighters after the frigate.
        drag(&mut pg, (40, 30), (100, 90), false);
        assert_eq!(picked(&pg), f.to_vec());
        drag(&mut pg, (200, 90), (200, 90), true);
        pg.tick();
        pg.tick();
        assert!(f.iter().all(|&i| ship(&pg, i).0 == Some(frigate)));
        // A right-click on empty space after that is a place again.
        drag(&mut pg, (250, 250), (250, 250), true);
        pg.tick();
        pg.tick();
        let to: Vec<(i32, i32)> = f.iter().map(|&i| ship(&pg, i).1.unwrap()).collect();
        assert!(f.iter().all(|&i| ship(&pg, i).0.is_none()) && to.iter().all(|p| cheb(p.0, p.1, 250, 1000) <= 12), "{to:?}");
        assert!(to[0] != to[1] && to[1] != to[2] && to[0] != to[2], "{to:?}");
        // Sent after one of themselves, the rest go after it and it stays where it is.
        let at = ship(&pg, f[0]).2;
        let on = (at.0, at.1 - cam(&pg).1);
        drag(&mut pg, on, on, true);
        pg.tick();
        pg.tick();
        assert_eq!((ship(&pg, f[0]).0, ship(&pg, f[1]).0, ship(&pg, f[2]).0), (None, Some(f[0]), Some(f[0])));
        assert_eq!(ship(&pg, f[0]).1, Some(at), "the one they were sent after wasn't told to stay where it is");
    }
}
