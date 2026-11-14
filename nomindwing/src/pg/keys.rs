//! The keys page (F3): every key the screen in front of you takes, from one table a screen.
//! A test reads each screen's key handler and fails if a key it takes isn't in that screen's
//! table, so a key can't be added and left off the page.

use super::{library, Pg, Screen};
use crate::framebuffer::{wrap, Framebuffer};
use crate::palette::*;
use crate::ui::{self, Rect};

/// The button on a rules panel that goes on to the keys page.
pub(super) const BTN_KEYS: Rect = Rect::new(272, 16, 58, 13);
const PAGE: Rect = Rect::new(10, 2, 380, 296);
/// Where the rows start, how wide the keys' column is, and how many letters wide the other is.
const TOP: i32 = 24;
const KEY_W: i32 = 108;
const COLS: usize = 42;
/// The room above a heading.
const HEAD_GAP: i32 = 4;

/// A row of a keys page: the keys, parted by spaces, and what they do. A row with no keys is a
/// heading.
pub(super) type Row = (&'static str, &'static str);

pub(super) const MATCH_KEYS: &[Row] = &[
    ("W A S D ARROWS", "SCROLL THE VIEW"),
    ("K", "YOUR KING. TWICE: THE VIEW GOES TO IT"),
    ("TAB", "THE WHOLE MAP, AND BACK"),
    ("Z X C V B N M , .", "PRINT THAT CARD. A SEED THEN NEEDS A ROCK"),
    ("CTRL+CARD KEY", "SELECT EVERY SHIP OF THAT CARD"),
    ("R", "THE LAST JOB ASKED FOR REPEATS, OR STOPS"),
    ("BACKSPACE", "DROP WHAT YOUR KING IS PRINTING"),
    ("Q", "SELECT YOUR WARSHIPS IN VIEW"),
    ("CTRL+Q", "SELECT EVERY WARSHIP YOU HAVE"),
    ("1-9", "THAT GROUP. TWICE: THE VIEW GOES TO IT"),
    ("CTRL+1-9", "MAKE THAT GROUP OF WHAT'S SELECTED"),
    ("SHIFT+1-9", "ADD WHAT'S SELECTED TO THAT GROUP"),
    ("G", "LET GO: WHAT'S SELECTED IS UNSENT AGAIN"),
    ("T", "UNSENT SHIPS ALL GUARD, OR ALL ATTACK"),
    ("ESC", "STOP PLANTING. SELECT NOTHING. THEN: QUIT"),
    ("SPACE P", "PAUSE, AND RUN"),
    ("F", "SPEED: 1X, 2X, 4X"),
    ("L", "LAG: HOW LONG YOUR COMMANDS WAIT"),
    ("F1 F2 F3", "THE RULES. SCANLINES. THIS PAGE"),
    ("", "THE MOUSE"),
    ("CLICK", "SELECT A SHIP, YOUR KING OR A FACTORY"),
    ("DOUBLE-CLICK", "ALL OF THAT TYPE IN VIEW. CTRL+CLICK TOO"),
    ("SHIFT+CLICK", "ADD TO WHAT'S SELECTED, OR TAKE OUT"),
    ("DRAG", "A BOX: THE WARSHIPS INSIDE IT"),
    ("RIGHT-CLICK", "SEND WHAT'S SELECTED THERE, OR AFTER A SHIP. A FACTORY: ITS NEW SHIPS GATHER THERE. A CARD OR A JOB: IT REPEATS"),
    ("CTRL+CLICK", "WHILE PLANTING: THE SEED TO OPEN SPACE"),
];

pub(super) const YARD_KEYS: &[Row] = &[
    ("W A S D ARROWS", "SCROLL THE CANVAS"),
    ("Q E", "ZOOM OUT, ZOOM IN"),
    ("1-6", "A CATEGORY OF PARTS"),
    ("TAB", "THE NEXT PART IN THE CATEGORY"),
    ("P R F C", "PENCIL, RECTANGLE, FILL, COPY"),
    ("T", "TURN WHAT YOU'VE COPIED"),
    ("U", "UNDO"),
    ("M", "MIRROR, ON AND OFF"),
    ("O", "THE DESIGN'S ORDER"),
    ("Y", "HOW THE DESIGN FLIES"),
    ("G", "HOW MANY OF IT LEAVE TOGETHER"),
    ("L", "THE LIBRARY"),
    ("ENTER", "THE ARENA"),
    ("ESC", "QUIT"),
    ("F1 F2 F3", "THE RULES. SCANLINES. THIS PAGE"),
    ("", "THE MOUSE"),
    ("CLICK", "PAINT, OR PLACE WHAT YOU'VE COPIED"),
    ("RIGHT-CLICK", "ERASE. PUT A COPY DOWN. ON A DESIGN'S TAB: COPY THE WHOLE DESIGN"),
    ("WHEEL", "ZOOM"),
];

pub(super) const ARENA_KEYS: &[Row] = &[
    ("W A S D ARROWS", "SCROLL THE VIEW"),
    ("1-9", "THE DESIGN YOU PLACE"),
    ("T", "PLACE FOR THE OTHER SIDE"),
    ("SPACE", "RUN, AND PAUSE"),
    ("R", "THE FIGHT STARTS AGAIN"),
    ("F", "SPEED: 1X, 2X, 4X"),
    ("N", "THE NEXT READY-MADE FIGHT"),
    ("B V", "BLUE'S BRAIN, RED'S BRAIN"),
    ("TAB ENTER", "BACK TO THE YARD"),
    ("ESC", "QUIT"),
    ("F1 F2 F3", "THE RULES. SCANLINES. THIS PAGE"),
    ("", "THE MOUSE"),
    ("CLICK", "PLACE THE DESIGN, BEFORE THE FIGHT STARTS"),
    ("RIGHT-CLICK", "TAKE A PLACED SHIP AWAY"),
];

pub(super) const LIBRARY_KEYS: &[Row] = &[
    ("W S ARROWS", "UP AND DOWN THE LIST"),
    ("TAB", "SHIPS, OR TEAMS"),
    ("1-9", "THE SLOT A SHIP GOES IN"),
    ("ENTER", "TAKE THE SHIP, OR LOAD THE TEAM"),
    ("K", "KEEP YOUR SHIP, OR YOUR TEAM"),
    ("N", "NAME A TEAM OF YOURS"),
    ("C V", "COPY AS TEXT. PASTE TEXT"),
    ("X BACKSPACE", "DELETE ONE OF YOURS: PRESS IT TWICE"),
    ("ESC L", "BACK TO THE YARD"),
    ("F1 F2 F3", "THE RULES. SCANLINES. THIS PAGE"),
    ("", "THE MOUSE"),
    ("WHEEL", "SCROLL THE LIST"),
];

/// How tall a table draws, in pixels.
#[cfg(test)]
fn height(rows: &[Row]) -> i32 {
    rows.iter().map(|(keys, what)| if keys.is_empty() { HEAD_GAP + 9 } else { wrap(what, COLS).len() as i32 * 9 }).sum()
}

impl Pg {
    /// The table for the screen in front of you, and the screen's name.
    pub(super) fn keys_table(&self) -> (&'static str, &'static [Row]) {
        if library::open(self) {
            return ("THE LIBRARY", LIBRARY_KEYS);
        }
        match self.screen {
            Screen::Yard => ("THE YARD", YARD_KEYS),
            Screen::Arena => ("THE ARENA", ARENA_KEYS),
            Screen::Match => ("THE MATCH", MATCH_KEYS),
        }
    }

    pub(super) fn draw_keys(&self, fb: &mut Framebuffer) {
        let (name, rows) = self.keys_table();
        let r = PAGE;
        ui::panel(fb, r);
        fb.text(&format!("KEYS: {name}"), r.x + 8, r.y + 5, ACCENT, Some(OUTLINE), 2);
        let foot = "CLICK OR PRESS ANY KEY";
        fb.text(foot, r.x + r.w - 8 - Framebuffer::text_width(foot, 1), r.y + 9, TEXT_DIM, None, 1);
        let mut y = r.y + TOP;
        for (keys, what) in rows {
            if keys.is_empty() {
                y += HEAD_GAP;
                fb.text(what, r.x + 8, y, ACCENT, None, 1);
                y += 9;
                continue;
            }
            fb.text(keys, r.x + 8, y, GOLD, None, 1);
            let lines = wrap(what, COLS);
            for (i, l) in lines.iter().enumerate() {
                fb.text(l, r.x + 8 + KEY_W, y + i as i32 * 9, TEXT, None, 1);
            }
            y += lines.len() as i32 * 9;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::super::play::CARD_KEYS;
    use super::*;
    use crate::app::{App, Ev, Key};

    /// The keys a table names: its keys' columns, split at spaces and plus signs.
    fn named(rows: &[Row]) -> Vec<String> {
        rows.iter().flat_map(|(keys, _)| keys.split(|c| c == ' ' || c == '+')).filter(|t| !t.is_empty()).map(|t| t.to_string()).collect()
    }

    /// The source of a method: from its `fn name(` to the line that closes it (four spaces and
    /// a brace).
    fn body<'a>(src: &'a str, name: &str) -> &'a str {
        let from = src.find(&format!("fn {name}(")).unwrap_or_else(|| panic!("no fn {name}"));
        let rest = &src[from..];
        &rest[..rest.find("\n    }").expect("the method ends")]
    }

    /// The keys a piece of a key handler's source takes, by the names a keys page gives them:
    /// every `Key::Something` in it. (`Key::Letter(c)` with no letter written is a set of keys,
    /// checked apart.)
    fn taken(code: &str) -> Vec<String> {
        let mut out = Vec::new();
        let mut rest = code;
        while let Some(at) = rest.find("Key::") {
            rest = &rest[at + 5..];
            let word: String = rest.chars().take_while(|c| c.is_ascii_alphanumeric()).collect();
            let name = match word.as_str() {
                "Letter" => match rest[word.len()..].strip_prefix("('") {
                    Some(arg) => arg.chars().next().unwrap().to_string(),
                    None => continue,
                },
                "Num" => "1-9".to_string(),
                "ShiftNum" => "SHIFT".to_string(),
                "Tab" => "TAB".to_string(),
                "Space" => "SPACE".to_string(),
                "Esc" => "ESC".to_string(),
                "Help" => "F1".to_string(),
                "Keys" => "F3".to_string(),
                "Back" => "BACKSPACE".to_string(),
                "Enter" => "ENTER".to_string(),
                other => panic!("a key this check doesn't know: Key::{other}"),
            };
            if !out.contains(&name) {
                out.push(name);
            }
        }
        out
    }

    /// A table has a key if it names it. Digits are named as a range, "1-9" or "1-6".
    fn has(rows: &[Row], key: &str) -> bool {
        let names = named(rows);
        if key == "1-9" {
            return names.iter().any(|n| n.len() == 3 && n.starts_with('1') && n.as_bytes()[1] == b'-' && n.as_bytes()[2].is_ascii_digit());
        }
        names.iter().any(|n| n == key)
    }

    #[test]
    fn every_key_a_screen_takes_is_on_its_keys_page() {
        let read = |s: &str| s.replace("\r\n", "\n");
        let (play, yard, lib, top) = (read(include_str!("play.rs")), read(include_str!("yard.rs")), read(include_str!("library.rs")), read(include_str!("mod.rs")));
        // The arena's keys, and the keys every screen shares, are arms of the one `key` in
        // mod.rs: an arm is the arena's if it names the arena, and shared if it names no screen.
        let all = body(&top, "key");
        let arms = |want: &str| -> String {
            all.lines()
                .filter_map(|l| l.split("=>").next().filter(|_| l.contains("=>")))
                .flat_map(|pattern| pattern.split(" | "))
                .filter(|alt| alt.trim_start().starts_with(want))
                .collect::<Vec<_>>()
                .join("\n")
        };
        let (arena, shared) = (taken(&arms("(Screen::Arena,")), taken(&arms("(_,")));
        assert!(arena.len() >= 8 && shared == ["ESC", "F1", "F3", "ENTER"], "the arena takes {arena:?} and every screen {shared:?}");
        let mut checks: Vec<(&str, &[Row], Vec<String>)> = vec![
            ("the match", MATCH_KEYS, taken(body(&play, "key_match"))),
            ("the yard", YARD_KEYS, taken(body(&yard, "key_yard"))),
            ("the arena", ARENA_KEYS, arena),
            ("the library", LIBRARY_KEYS, taken(body(&lib, "key_library"))),
        ];
        for (name, _, keys) in checks.iter_mut() {
            // A match takes every key before the shared ENTER is reached, and the library has
            // its own arm for the rules and this page.
            let more: &[&str] = match *name {
                "the match" => &["ESC", "F1", "F3"],
                "the library" => &["F1", "F3"],
                _ => &["ESC", "F1", "F3", "ENTER"],
            };
            keys.extend(more.iter().map(|k| k.to_string()));
            // Read by the window itself: the scanline key everywhere, and the keys that are held.
            keys.push("F2".to_string());
            keys.extend(if *name == "the library" { vec!["W", "S", "ARROWS"] } else { vec!["W", "A", "S", "D", "ARROWS"] }.into_iter().map(|k| k.to_string()));
            if *name == "the match" {
                keys.extend(CARD_KEYS.iter().map(|k| k.to_string()));
            }
        }
        for (name, rows, keys) in &checks {
            assert!(keys.len() >= 10, "{name} takes only {keys:?}");
            for k in keys {
                assert!(has(rows, k), "{name} takes {k}, and its keys page doesn't say so");
            }
        }
        // And the check itself reads what it should: the match's handler has its stance key and
        // its groups, the yard's its tools.
        assert!(checks[0].2.contains(&"T".to_string()) && checks[0].2.contains(&"SHIFT".to_string()) && checks[0].2.contains(&"1-9".to_string()));
        assert!(["P", "R", "F", "C", "U", "M"].iter().all(|k| checks[1].2.contains(&k.to_string())));
    }

    #[test]
    fn every_keys_page_fits_its_panel() {
        for (name, rows) in [("match", MATCH_KEYS), ("yard", YARD_KEYS), ("arena", ARENA_KEYS), ("library", LIBRARY_KEYS)] {
            assert!(height(rows) <= PAGE.h - TOP - 4, "the {name}'s keys run {} px, and the page has {}", height(rows), PAGE.h - TOP - 4);
            for (keys, what) in rows {
                assert!(Framebuffer::text_width(keys, 1) <= KEY_W - 6, "{keys} is too wide for its column");
                assert!(wrap(what, COLS).iter().all(|l| Framebuffer::text_width(l, 1) <= PAGE.w - 16 - KEY_W), "{what}");
            }
        }
        // The title and the note beside it don't meet.
        let longest = Framebuffer::text_width("KEYS: THE LIBRARY", 2);
        assert!(PAGE.x + 8 + longest + 6 <= PAGE.x + PAGE.w - 8 - Framebuffer::text_width("CLICK OR PRESS ANY KEY", 1));
    }

    #[test]
    fn f3_shows_the_keys_of_the_screen_in_front_and_any_key_puts_them_away() {
        let mut pg = Pg::new(None, None);
        pg.help = false;
        // The game opens on the arena.
        pg.event(Ev::Key(Key::Keys));
        assert!(pg.keys && pg.keys_table().0 == "THE ARENA");
        // Any key puts the page away, and does nothing else: Enter doesn't go to the yard.
        pg.event(Ev::Key(Key::Enter));
        assert!(!pg.keys && pg.screen == Screen::Arena);
        // A click puts it away too, and presses nothing under it: on the arena's field, where a
        // click places a ship, none is placed.
        pg.event(Ev::Key(Key::Keys));
        let placed = pg.setup.len();
        pg.event(Ev::Press { x: 150, y: 150, right: false });
        assert!(!pg.keys && pg.setup.len() == placed, "a click on the keys page placed a ship under it");
        pg.event(Ev::Press { x: 150, y: 150, right: false });
        assert_eq!(pg.setup.len(), placed + 1, "the check has no teeth: a click on the field places nothing");
        // The yard's.
        pg.event(Ev::Key(Key::Enter));
        pg.event(Ev::Key(Key::Keys));
        assert!(pg.keys && pg.screen == Screen::Yard && pg.keys_table().0 == "THE YARD");
        pg.event(Ev::Key(Key::Letter('L')));
        assert!(!pg.keys && !library::open(&pg));
        // In the library the page is the library's.
        pg.event(Ev::Key(Key::Letter('L')));
        pg.event(Ev::Key(Key::Keys));
        assert!(pg.keys && library::open(&pg) && pg.keys_table().0 == "THE LIBRARY");
        // A click puts it away too, and presses nothing under it.
        pg.event(Ev::Press { x: 200, y: 150, right: false });
        assert!(!pg.keys && library::open(&pg));
        pg.event(Ev::Key(Key::Esc));
        assert!(!library::open(&pg));
        // From the rules, F3 goes on to the keys, and so does the button on the rules panel.
        pg.event(Ev::Key(Key::Help));
        assert!(pg.help);
        pg.event(Ev::Key(Key::Keys));
        assert!(pg.keys && !pg.help);
        pg.event(Ev::Key(Key::Space));
        pg.event(Ev::Key(Key::Help));
        pg.event(Ev::Press { x: BTN_KEYS.x + 3, y: BTN_KEYS.y + 3, right: false });
        assert!(pg.keys && !pg.help);
        pg.event(Ev::Key(Key::Space));
        // Anywhere else on the rules panel a click only puts the rules away.
        pg.event(Ev::Key(Key::Help));
        pg.event(Ev::Press { x: 200, y: 150, right: false });
        assert!(!pg.keys && !pg.help);
        // A match against the AI stands still while the page is up.
        pg.start_match(true, 5);
        pg.help = false;
        pg.tick();
        let at = pg.play.as_ref().unwrap().arena.tick;
        pg.event(Ev::Key(Key::Keys));
        assert!(pg.keys && pg.keys_table().0 == "THE MATCH");
        for _ in 0..5 {
            pg.tick();
        }
        assert_eq!(pg.play.as_ref().unwrap().arena.tick, at, "the match ran on under the keys page");
        pg.event(Ev::Key(Key::Keys));
        pg.tick();
        assert!(pg.play.as_ref().unwrap().arena.tick > at);
    }
}
