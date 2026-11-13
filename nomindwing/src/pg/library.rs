//! The design library: every drawing you've kept, the ready-made ones, and whole teams of nine.
//! A drawing carries its order, how it flies and its group size with it (Henry, 2026-11-12), and
//! taking one into a slot makes a copy the yard edits like any other. Yours live in one text
//! file beside the save, and a drawing passes between players as a line of text on the
//! clipboard. The screen only ever asks a shelf for its list, for one entry, and to keep one:
//! that's all a shelf on another machine would be asked for too.

use super::design::*;
use super::{draw_icon, part_col, parts_library, Pg, Screen};
use crate::app::Key;
use crate::framebuffer::{wrap, Framebuffer};
use crate::palette::*;
use crate::ui::{self, Rect};
use std::path::PathBuf;

pub const LIB_HEADER: &str = "#LIBRARY1";
/// The most drawings and teams of your own the library keeps.
pub const SHIPS_MAX: usize = 200;
pub const TEAMS_MAX: usize = 50;
const TEAM_NAME_MAX: usize = 12;
/// The most of a clipboard that's read as drawings, and the most it may add at once.
const PASTE_CHARS: usize = 400_000;
const PASTE_MAX: usize = 20;

/// Where a drawing in the library comes from.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Shelf {
    Mine,
    Parts,
    Starter,
    Reference,
}

impl Shelf {
    fn from(self) -> &'static str {
        match self {
            Shelf::Mine => "ONE OF YOURS",
            Shelf::Parts => "A PART EXAMPLE",
            Shelf::Starter => "FROM THE STARTER DECK",
            Shelf::Reference => "FROM THE REFERENCE FLEET",
        }
    }
    fn tag(self) -> &'static str {
        match self {
            Shelf::Mine => "YOURS",
            Shelf::Parts => "PARTS",
            Shelf::Starter => "START",
            Shelf::Reference => "REF",
        }
    }
}

#[derive(Clone, Debug)]
pub struct Team {
    pub name: String,
    pub designs: Vec<Design>,
}

impl Team {
    fn lines(&self) -> Vec<String> {
        self.designs.iter().map(|d| d.to_line()).collect()
    }
    /// A team as text for the clipboard: its name, then its nine drawings, a line each.
    pub fn text(&self) -> String {
        format!("TEAM {}\n{}\n", self.name, self.lines().join("\n"))
    }
}

/// A name someone typed or passed along, made safe to show: capitals, digits, spaces and
/// dashes, and no longer than there's room for.
fn clean_name(s: &str, max: usize, or: &str) -> String {
    let n: String = s.trim().chars().map(|c| c.to_ascii_uppercase()).filter(|c| c.is_ascii_alphanumeric() || *c == ' ' || *c == '-').take(max).collect();
    if n.trim().is_empty() {
        or.to_string()
    } else {
        n.trim().to_string()
    }
}

/// A drawing from a line of text that came from outside the game.
fn read_line(line: &str) -> Option<Design> {
    let mut d = Design::from_line(line)?;
    d.name = clean_name(&d.name, 10, "PASTED");
    Some(d)
}

/// What a paste may hold.
#[derive(Debug)]
pub enum Pasted {
    Ship(Design),
    Team(Team),
}

/// Reads text someone passed along: every line that is a drawing, and after a TEAM line the
/// drawings that follow it as that team's slots. Anything else is skipped, so a paste out of a
/// chat with words round it still works, and rubbish adds nothing.
pub fn read_shared(text: &str) -> Vec<Pasted> {
    let mut out: Vec<Pasted> = Vec::new();
    let end = text.char_indices().nth(PASTE_CHARS).map_or(text.len(), |(i, _)| i);
    for line in text[..end].lines().map(str::trim).filter(|l| !l.is_empty()) {
        if out.len() >= PASTE_MAX {
            break;
        }
        if let Some(name) = line.strip_prefix("TEAM ") {
            out.push(Pasted::Team(Team { name: clean_name(name, TEAM_NAME_MAX, "TEAM"), designs: Vec::new() }));
        } else if let Some(d) = read_line(line) {
            match out.last_mut() {
                // A team's slot may be empty. On its own, a line with nothing drawn is nothing.
                Some(Pasted::Team(t)) if t.designs.len() < SLOTS => t.designs.push(d),
                _ if d.is_empty() => {}
                _ => out.push(Pasted::Ship(d)),
            }
        }
    }
    out.retain(|p| !matches!(p, Pasted::Team(t) if t.designs.iter().all(|d| d.is_empty())));
    for p in &mut out {
        if let Pasted::Team(t) = p {
            t.designs = team_of(std::mem::take(&mut t.designs));
        }
    }
    out
}

pub struct Library {
    /// Your own, newest first.
    pub ships: Vec<Design>,
    pub teams: Vec<Team>,
    /// The ready-made ones, which follow the game's rules as they change and are never saved.
    ready_ships: Vec<(Shelf, Design)>,
    ready_teams: Vec<Team>,
    path: Option<PathBuf>,
}

impl Library {
    pub fn load(path: Option<PathBuf>) -> Library {
        let ready_teams = vec![
            Team { name: "STARTER DECK".into(), designs: team_of(starter_deck()) },
            Team { name: "NEW PARTS".into(), designs: team_of(parts_library()) },
            Team { name: "REF FLEET".into(), designs: team_of(defaults()) },
        ];
        let mut ready_ships: Vec<(Shelf, Design)> = Vec::new();
        for (shelf, designs) in [(Shelf::Parts, part_examples()), (Shelf::Starter, starter_deck()), (Shelf::Reference, defaults())] {
            for d in designs {
                if !ready_ships.iter().any(|(_, o)| o.to_line() == d.to_line()) {
                    ready_ships.push((shelf, d));
                }
            }
        }
        let mut lib = Library { ships: Vec::new(), teams: Vec::new(), ready_ships, ready_teams, path };
        if let Some(text) = lib.path.as_ref().and_then(|p| std::fs::read_to_string(p).ok()) {
            lib.read(&text);
        }
        lib
    }

    /// The file: SHIP and a drawing's line for each of yours, then TEAM and a name for each team
    /// with a SLOT line for each of its nine.
    pub fn text(&self) -> String {
        let mut s = format!("{LIB_HEADER}\n");
        for d in &self.ships {
            s += &format!("SHIP {}\n", d.to_line());
        }
        for t in &self.teams {
            s += &format!("TEAM {}\n", t.name);
            for l in t.lines() {
                s += &format!("SLOT {l}\n");
            }
        }
        s
    }

    pub fn read(&mut self, text: &str) {
        if text.lines().next().map(str::trim) != Some(LIB_HEADER) {
            return;
        }
        for line in text.lines().skip(1) {
            if let Some(d) = line.strip_prefix("SHIP ").and_then(Design::from_line) {
                self.ships.push(d);
            } else if let Some(name) = line.strip_prefix("TEAM ") {
                self.teams.push(Team { name: clean_name(name, TEAM_NAME_MAX, "TEAM"), designs: Vec::new() });
            } else if let (Some(d), Some(t)) = (line.strip_prefix("SLOT ").and_then(Design::from_line), self.teams.last_mut()) {
                t.designs.push(d);
            }
        }
        self.ships.truncate(SHIPS_MAX);
        self.teams.truncate(TEAMS_MAX);
        for t in &mut self.teams {
            t.designs = team_of(std::mem::take(&mut t.designs));
        }
    }

    /// Written whole to a file beside it and then swapped in, so a crash part-way can't leave
    /// half a library.
    fn save(&self) {
        if let Some(p) = &self.path {
            let tmp = p.with_extension("tmp");
            if std::fs::write(&tmp, self.text()).is_ok() {
                let _ = std::fs::rename(&tmp, p);
            }
        }
    }

    // ----- What's on the shelves: yours first, then the ready-made -----

    pub fn ship_count(&self) -> usize {
        self.ships.len() + self.ready_ships.len()
    }
    pub fn ship(&self, i: usize) -> Option<(Shelf, &Design)> {
        match self.ships.get(i) {
            Some(d) => Some((Shelf::Mine, d)),
            None => self.ready_ships.get(i - self.ships.len()).map(|(s, d)| (*s, d)),
        }
    }
    pub fn team_count(&self) -> usize {
        self.teams.len() + self.ready_teams.len()
    }
    /// A team, and whether it's one of yours.
    pub fn team(&self, i: usize) -> Option<(bool, &Team)> {
        match self.teams.get(i) {
            Some(t) => Some((true, t)),
            None => self.ready_teams.get(i - self.teams.len()).map(|t| (false, t)),
        }
    }

    /// Whether a drawing just like this one is on a shelf already: same name, pixels and orders.
    pub fn has(&self, d: &Design) -> bool {
        let line = d.to_line();
        self.ships.iter().chain(self.ready_ships.iter().map(|(_, d)| d)).any(|o| o.to_line() == line)
    }

    /// Keeps a drawing as one of yours. It says why not if it doesn't.
    pub fn keep(&mut self, d: &Design) -> Result<(), &'static str> {
        if d.is_empty() {
            Err("THERE'S NOTHING DRAWN IN THAT SLOT TO KEEP")
        } else if self.has(d) {
            Err("ONE JUST LIKE IT IS IN THE LIBRARY ALREADY")
        } else if self.ships.len() >= SHIPS_MAX {
            Err("THE LIBRARY IS FULL: DELETE ONE OF YOURS FIRST")
        } else {
            self.ships.insert(0, d.clone());
            self.save();
            Ok(())
        }
    }

    pub fn has_team(&self, designs: &[Design]) -> bool {
        let lines: Vec<String> = designs.iter().map(|d| d.to_line()).collect();
        self.teams.iter().chain(&self.ready_teams).any(|t| t.lines() == lines)
    }

    /// Keeps a team of nine as one of yours, under the name given or the next free TEAM number.
    pub fn keep_team(&mut self, designs: &[Design], name: Option<&str>) -> Result<String, &'static str> {
        if designs.iter().all(|d| d.is_empty()) {
            Err("THERE'S NOTHING IN THAT TEAM TO KEEP")
        } else if self.has_team(designs) {
            Err("A TEAM JUST LIKE IT IS IN THE LIBRARY ALREADY")
        } else if self.teams.len() >= TEAMS_MAX {
            Err("THE LIBRARY IS FULL OF TEAMS: DELETE ONE OF YOURS FIRST")
        } else {
            let free = || (1..).map(|n| format!("TEAM {n}")).find(|n| !self.teams.iter().any(|t| &t.name == n)).unwrap();
            let name = name.map_or_else(free, |n| n.to_string());
            self.teams.insert(0, Team { name: name.clone(), designs: team_of(designs.to_vec()) });
            self.save();
            Ok(name)
        }
    }

    pub fn delete_ship(&mut self, i: usize) {
        if i < self.ships.len() {
            self.ships.remove(i);
            self.save();
        }
    }
    pub fn delete_team(&mut self, i: usize) {
        if i < self.teams.len() {
            self.teams.remove(i);
            self.save();
        }
    }
}

// ----- The screen, open over the yard -----

const BOX: Rect = Rect::new(6, 4, 388, 292);
const TAB_BTNS: [Rect; 2] = [Rect::new(120, 9, 50, 15), Rect::new(172, 9, 50, 15)];
const BTN_CLOSE: Rect = Rect::new(338, 9, 50, 15);
pub const ROWS: usize = 13;
const ROW: i32 = 16;
const LIST: Rect = Rect::new(12, 30, 204, ROWS as i32 * ROW);
const BTN_UP: Rect = Rect::new(218, 30, 14, 16);
const BTN_DOWN: Rect = Rect::new(218, 222, 14, 16);
const PREVIEW: Rect = Rect::new(236, 30, 152, 84);
const INFO_Y: i32 = 118;
const TEAM_Y: i32 = 157;
const BTN_TAKE: Rect = Rect::new(236, 188, 152, 16);
const BTN_KEEP: Rect = Rect::new(236, 206, 75, 16);
const BTN_NAME: Rect = Rect::new(313, 206, 75, 16);
const BTN_COPY: Rect = Rect::new(236, 224, 49, 16);
const BTN_PASTE: Rect = Rect::new(287, 224, 50, 16);
const BTN_DEL: Rect = Rect::new(339, 224, 49, 16);
const TIP: Rect = Rect::new(12, 244, 376, 46);
/// Characters to a line of the tip box.
const TIP_COLS: usize = ((TIP.w - 8) / 6) as usize;

fn row_rect(k: usize) -> Rect {
    Rect::new(LIST.x, LIST.y + k as i32 * ROW, LIST.w, ROW - 1)
}
/// Your team's nine slots, under the preview: the one a drawing is taken into is lit.
fn slot_rect(i: usize) -> Rect {
    Rect::new(236 + i as i32 * 17, TEAM_Y + 10, 16, 16)
}

/// The library as it's being looked at.
pub struct Browse {
    /// Looking at teams, not single drawings.
    pub teams: bool,
    /// The entry picked, and the first one on screen.
    pub at: usize,
    pub top: usize,
    /// DELETE has been pressed once on this entry: once more removes it.
    armed: Option<usize>,
    /// A team of yours is being renamed.
    pub naming: bool,
    /// What the last thing done has to say, until the next thing is done.
    pub note: Option<String>,
    /// Ticks an arrow has been held, for stepping through the list at a steady pace.
    held: u32,
}

/// The drawing as big as whole pixels allow in a box: each of its pixels an exact square.
fn draw_preview(fb: &mut Framebuffer, d: &Design, r: Rect, team: usize) {
    fb.rect_fill(r.x, r.y, r.w, r.h, SPACE);
    if d.is_empty() {
        return;
    }
    let sh = d.shape();
    let z = ((r.w - 4) / sh.w).min((r.h - 4) / sh.h).min(8);
    if z < 1 {
        draw_icon(fb, d, r, team);
        return;
    }
    let (ox, oy) = (r.x + (r.w - sh.w * z) / 2, r.y + (r.h - sh.h * z) / 2);
    for cy in 0..sh.h {
        for cx in 0..sh.w {
            if let Some(p) = sh.cells[(cy * sh.w + cx) as usize] {
                fb.rect_fill(ox + cx * z, oy + cy * z, z, z, if p == Part::Cannon { GOLD } else { part_col(p, team) });
            }
        }
    }
}

impl Pg {
    pub fn open_library(&mut self) {
        self.end_naming();
        self.menu = false;
        self.ed.drag = None;
        self.browse = Some(Browse { teams: false, at: 0, top: 0, armed: None, naming: false, note: None, held: 0 });
    }

    fn close_library(&mut self) {
        self.end_team_naming();
        self.browse = None;
    }

    fn end_team_naming(&mut self) {
        let Some(b) = self.browse.as_mut() else { return };
        if b.naming {
            b.naming = false;
            if let Some(t) = self.library.teams.get_mut(b.at) {
                t.name = clean_name(&t.name, TEAM_NAME_MAX, "TEAM");
            }
            self.library.save();
        }
    }

    fn count(&self) -> usize {
        match &self.browse {
            Some(b) if b.teams => self.library.team_count(),
            _ => self.library.ship_count(),
        }
    }

    /// Picks an entry and keeps it on screen. Anything but DELETE twice running lets go of a
    /// delete that was waiting to be confirmed.
    fn pick(&mut self, at: usize) {
        let n = self.count();
        self.end_team_naming();
        if let Some(b) = self.browse.as_mut() {
            b.at = at.min(n.saturating_sub(1));
            b.top = b.top.min(b.at).max((b.at + 1).saturating_sub(ROWS)).min(n.saturating_sub(ROWS));
            b.armed = None;
        }
    }

    fn note(&mut self, s: &str) {
        if let Some(b) = self.browse.as_mut() {
            b.note = Some(s.to_string());
        }
    }

    fn set_tab(&mut self, teams: bool) {
        self.end_team_naming();
        if let Some(b) = self.browse.as_mut() {
            if b.teams != teams {
                (b.teams, b.at, b.top, b.armed) = (teams, 0, 0, None);
            }
        }
    }

    /// TAKE: the picked drawing goes into the open slot as a copy, and the yard opens on it. The
    /// drawing it replaces isn't lost: if it's nowhere in the library, it's kept there first.
    /// On the teams tab, the same for all nine slots at once.
    fn library_take(&mut self) {
        let Some(b) = self.browse.as_ref() else { return };
        if b.teams {
            let Some((_, team)) = self.library.team(b.at) else { return };
            let team = team.clone();
            let kept = self.library.keep_team(&self.lib, None).ok();
            self.lib = team_of(team.designs.clone());
            self.log(&format!("LIBRARY LOAD TEAM {}", team.name));
            for d in self.lib.clone() {
                self.log(&format!("  DESIGN {}", d.to_line()));
            }
            self.took(match kept {
                Some(name) => format!("{} LOADED. THE TEAM IT REPLACED IS KEPT AS {name}", team.name),
                None => format!("{} LOADED", team.name),
            });
        } else {
            let Some((_, d)) = self.library.ship(b.at) else { return };
            let d = d.clone();
            let old = self.lib[self.slot].clone();
            let kept = self.library.keep(&old).is_ok();
            self.lib[self.slot] = d.clone();
            self.log(&format!("LIBRARY TAKE INTO SLOT {}: {}", self.slot + 1, d.to_line()));
            self.took(if kept { format!("{} IS IN SLOT {}. {} WAS KEPT IN THE LIBRARY", d.name, self.slot + 1, old.name) } else { format!("{} IS IN SLOT {}", d.name, self.slot + 1) });
        }
    }

    /// The team has changed under the yard: save it, forget what undo knew, and show it.
    fn took(&mut self, said: String) {
        self.ed.forget();
        self.edited = true;
        self.save();
        self.close_library();
        self.fit_view();
        self.say(&said);
    }

    /// KEEP: the open slot's drawing (or the whole team) goes into the library as one of yours.
    fn library_keep(&mut self) {
        let Some(b) = self.browse.as_ref() else { return };
        let teams = b.teams;
        let (said, kept) = if teams {
            match self.library.keep_team(&self.lib, None) {
                Ok(name) => {
                    self.log(&format!("LIBRARY KEEP TEAM {name}"));
                    (format!("YOUR TEAM IS KEPT AS {name}. [N] RENAMES IT"), true)
                }
                Err(why) => (why.to_string(), false),
            }
        } else {
            let d = self.lib[self.slot].clone();
            match self.library.keep(&d) {
                Ok(()) => {
                    self.log(&format!("LIBRARY KEEP {}", d.to_line()));
                    (format!("{} IS KEPT IN THE LIBRARY", d.name), true)
                }
                Err(why) => (why.to_string(), false),
            }
        };
        // What was just kept is the newest, so it's first.
        if kept {
            self.pick(0);
            if let Some(b) = self.browse.as_mut() {
                b.top = 0;
            }
        }
        self.note(&said);
    }

    /// COPY: the picked drawing, or team, as text on the clipboard.
    fn library_copy(&mut self) {
        let Some(b) = self.browse.as_ref() else { return };
        let (text, name) = if b.teams {
            let Some((_, t)) = self.library.team(b.at) else { return };
            (t.text(), t.name.clone())
        } else {
            let Some((_, d)) = self.library.ship(b.at) else { return };
            (d.to_line() + "\n", d.name.clone())
        };
        let said = if crate::clip::set(&text) { format!("{name} IS ON THE CLIPBOARD AS TEXT. PASTE IT TO SOMEONE, AND THEY PASTE IT IN HERE") } else { "THE CLIPBOARD WOULDN'T TAKE IT".to_string() };
        self.note(&said);
    }

    /// PASTE: whatever on the clipboard reads as drawings or teams is kept as yours.
    fn library_paste(&mut self) {
        let text = crate::clip::get().unwrap_or_default();
        self.paste_text(&text);
    }

    pub fn paste_text(&mut self, text: &str) {
        let (mut ships, mut teams, mut had) = (0, 0, 0);
        // Kept in the order they came, so the first pasted ends up first.
        for p in read_shared(text).into_iter().rev() {
            match p {
                Pasted::Ship(d) => match self.library.keep(&d) {
                    Ok(()) => ships += 1,
                    Err(_) => had += 1,
                },
                Pasted::Team(t) => match self.library.keep_team(&t.designs, Some(&t.name)) {
                    Ok(_) => teams += 1,
                    Err(_) => had += 1,
                },
            }
        }
        let s = |n: i32| if n == 1 { "" } else { "S" };
        let said = match (ships, teams, had) {
            (0, 0, 0) => "NOTHING ON THE CLIPBOARD READS AS A DRAWING. COPY ONE HERE, OR A LINE SOMEONE SENT YOU".to_string(),
            (0, 0, _) => "WHAT'S ON THE CLIPBOARD IS IN THE LIBRARY ALREADY".to_string(),
            (n, 0, _) => format!("PASTED {n} DRAWING{}", s(n)),
            (0, n, _) => format!("PASTED {n} TEAM{}", s(n)),
            (a, b, _) => format!("PASTED {a} DRAWING{} AND {b} TEAM{}", s(a), s(b)),
        };
        if ships + teams > 0 {
            self.log(&format!("LIBRARY PASTE: {ships} drawings, {teams} teams"));
            // Show what came in: the tab it's on, from the top.
            let on_teams = self.browse.as_ref().is_some_and(|b| b.teams);
            if (on_teams && teams == 0) || (!on_teams && ships == 0) {
                self.set_tab(!on_teams);
            }
            self.pick(0);
            if let Some(b) = self.browse.as_mut() {
                b.top = 0;
            }
        }
        self.note(&said);
    }

    /// DELETE: one of yours, for good, on the second press. The ready-made ones stay.
    fn library_delete(&mut self) {
        let Some(b) = self.browse.as_mut() else { return };
        let (at, teams) = (b.at, b.teams);
        let mine = if teams { at < self.library.teams.len() } else { at < self.library.ships.len() };
        if !mine {
            b.armed = None;
            b.note = Some("THE READY-MADE ONES STAY. ONLY YOURS CAN BE DELETED".into());
            return;
        }
        let name = if teams { self.library.teams[at].name.clone() } else { self.library.ships[at].name.clone() };
        if b.armed != Some(at) {
            b.armed = Some(at);
            b.note = Some(format!("DELETE AGAIN TO REMOVE {name} FROM THE LIBRARY FOR GOOD"));
            return;
        }
        if teams {
            self.library.delete_team(at);
        } else {
            self.library.delete_ship(at);
        }
        self.log(&format!("LIBRARY DELETE {name}"));
        self.pick(at);
        self.note(&format!("{name} IS GONE FROM THE LIBRARY"));
    }

    /// RENAME: type a new name for a team of yours.
    fn library_rename(&mut self) {
        let Some(b) = self.browse.as_mut() else { return };
        b.armed = None;
        if !b.teams {
            b.note = Some("A DRAWING IS RENAMED IN THE YARD: TAKE IT INTO A SLOT AND CLICK ITS NAME".into());
        } else if b.at >= self.library.teams.len() {
            b.note = Some("THE READY-MADE TEAMS KEEP THEIR NAMES. LOAD ONE AND KEEP IT TO HAVE YOUR OWN".into());
        } else {
            b.naming = true;
            b.note = None;
        }
    }

    // ----- Input -----

    pub(super) fn key_library(&mut self, k: Key) {
        let Some(b) = self.browse.as_mut() else { return };
        if b.naming {
            match k {
                Key::Back => {
                    if let Some(t) = self.library.teams.get_mut(b.at) {
                        t.name.pop();
                    }
                }
                Key::Enter | Key::Esc | Key::Tab => self.end_team_naming(),
                _ => {}
            }
            return;
        }
        b.note = None;
        let (teams, at) = (b.teams, b.at);
        // Anything but a second DELETE lets go of the first.
        if !matches!(k, Key::Letter('X') | Key::Back) {
            b.armed = None;
        }
        match k {
            Key::Esc | Key::Letter('L') => self.close_library(),
            Key::Tab => self.set_tab(!teams),
            Key::Enter => self.library_take(),
            Key::Letter('K') => self.library_keep(),
            Key::Letter('C') => self.library_copy(),
            Key::Letter('V') => self.library_paste(),
            Key::Letter('X') | Key::Back => self.library_delete(),
            Key::Letter('N') => self.library_rename(),
            Key::Num(n) if (1..=SLOTS as u8).contains(&n) => self.select_slot(n as usize - 1),
            Key::Letter('W') => self.pick(at.saturating_sub(1)),
            Key::Letter('S') => self.pick(at + 1),
            _ => {}
        }
    }

    pub(super) fn char_library(&mut self, c: char) {
        let Some(b) = self.browse.as_ref() else { return };
        if b.naming {
            let c = c.to_ascii_uppercase();
            if let Some(t) = self.library.teams.get_mut(b.at) {
                if (c.is_ascii_alphanumeric() || c == ' ' || c == '-') && t.name.chars().count() < TEAM_NAME_MAX {
                    t.name.push(c);
                }
            }
        }
    }

    pub(super) fn wheel_library(&mut self, notches: i32) {
        let n = self.count();
        if let Some(b) = self.browse.as_mut() {
            b.top = (b.top as i32 - notches).clamp(0, n.saturating_sub(ROWS) as i32) as usize;
        }
    }

    /// The arrows step through the list: once when pressed, then steadily while held.
    pub(super) fn nav_library(&mut self, dirs: &[bool]) {
        let Some(b) = self.browse.as_mut() else { return };
        let step = dirs[1] as i32 - dirs[0] as i32;
        if step == 0 || b.naming {
            b.held = 0;
            return;
        }
        b.held += 1;
        if b.held == 1 || (b.held > 5 && b.held % 2 == 0) {
            let at = (b.at as i32 + step).max(0) as usize;
            b.note = None;
            self.pick(at);
        }
    }

    pub(super) fn press_library(&mut self, x: i32, y: i32, right: bool) {
        let Some(b) = self.browse.as_mut() else { return };
        if right {
            return;
        }
        let (teams, top) = (b.teams, b.top);
        if b.naming && !BTN_NAME.contains(x, y) {
            self.end_team_naming();
        }
        let Some(b) = self.browse.as_mut() else { return };
        b.note = None;
        if !BTN_DEL.contains(x, y) {
            b.armed = None;
        }
        let n = self.count();
        if BTN_CLOSE.contains(x, y) || !BOX.contains(x, y) {
            self.close_library();
        } else if let Some(t) = (0..2).find(|&t| TAB_BTNS[t].contains(x, y)) {
            self.set_tab(t == 1);
        } else if let Some(k) = (0..ROWS).find(|&k| row_rect(k).contains(x, y) && top + k < n) {
            self.pick(top + k);
        } else if BTN_UP.contains(x, y) {
            self.wheel_library(3);
        } else if BTN_DOWN.contains(x, y) {
            self.wheel_library(-3);
        } else if let Some(i) = (0..SLOTS).find(|&i| slot_rect(i).contains(x, y)) {
            self.select_slot(i);
        } else if BTN_TAKE.contains(x, y) {
            self.library_take();
        } else if BTN_KEEP.contains(x, y) {
            self.library_keep();
        } else if BTN_NAME.contains(x, y) && teams {
            self.library_rename();
        } else if BTN_COPY.contains(x, y) {
            self.library_copy();
        } else if BTN_PASTE.contains(x, y) {
            self.library_paste();
        } else if BTN_DEL.contains(x, y) {
            self.library_delete();
        }
    }

    // ----- Drawing -----

    /// What the pointer is over, in words, or what the picked entry is.
    pub(super) fn library_tip(&self) -> String {
        let Some(b) = self.browse.as_ref() else { return String::new() };
        let m = self.mouse;
        let at = |r: Rect| r.contains(m.0, m.1);
        let slot = self.slot + 1;
        if at(BTN_TAKE) {
            return if b.teams {
                "[ENTER] LOAD THIS TEAM INTO ALL NINE SLOTS. THE TEAM YOU HAVE NOW IS KEPT IN THE LIBRARY FIRST, IF IT ISN'T THERE ALREADY.".into()
            } else {
                format!("[ENTER] PUT A COPY OF THIS DRAWING IN SLOT {slot}, TO FLY AS IT IS OR REDRAW. WHAT'S IN THE SLOT NOW IS KEPT IN THE LIBRARY FIRST, IF IT ISN'T THERE ALREADY.")
            };
        }
        if at(BTN_KEEP) {
            return if b.teams { "[K] KEEP YOUR WHOLE TEAM OF NINE IN THE LIBRARY, AS IT STANDS NOW.".into() } else { format!("[K] KEEP THE DRAWING IN SLOT {slot} IN THE LIBRARY, WITH ITS ORDER AND HOW IT FLIES.") };
        }
        if at(BTN_NAME) && b.teams {
            return "[N] RENAME A TEAM OF YOURS. ENTER WHEN DONE.".into();
        }
        if at(BTN_COPY) {
            return "[C] COPY IT TO THE CLIPBOARD AS A LINE OF TEXT. SEND THAT TO ANYONE: THEY PASTE IT INTO THEIR LIBRARY.".into();
        }
        if at(BTN_PASTE) {
            return "[V] KEEP WHATEVER DRAWINGS AND TEAMS ARE ON THE CLIPBOARD AS TEXT. ANYTHING ELSE THERE IS IGNORED.".into();
        }
        if at(BTN_DEL) {
            return "[X] DELETE ONE OF YOURS FROM THE LIBRARY, FOR GOOD. PRESS IT TWICE. THE READY-MADE ONES STAY.".into();
        }
        if at(BTN_CLOSE) {
            return "[ESC OR L] BACK TO THE YARD.".into();
        }
        if (0..2).any(|t| at(TAB_BTNS[t])) {
            return "[TAB] SINGLE DRAWINGS, OR WHOLE TEAMS OF NINE.".into();
        }
        if at(BTN_UP) || at(BTN_DOWN) {
            return "SCROLL THE LIST. THE WHEEL DOES TOO, AND THE ARROWS OR W AND S STEP THROUGH IT.".into();
        }
        if (0..SLOTS).any(|i| at(slot_rect(i))) {
            return format!("[1-9] YOUR TEAM. THE LIT SLOT, {slot}, IS WHERE A DRAWING IS TAKEN TO AND KEPT FROM.");
        }
        if b.teams {
            "WHOLE TEAMS: YOURS FIRST, THEN THE READY-MADE. LOAD ONE TO PLAY IT OR CHANGE IT; KEEP SAVES THE TEAM YOU HAVE NOW.".into()
        } else {
            "EVERY DRAWING: YOURS FIRST, THEN THE READY-MADE. TAKE ONE INTO A SLOT TO FLY IT OR REDRAW IT; THE COPY IN THE LIBRARY STAYS AS IT IS.".into()
        }
    }

    pub(super) fn draw_library(&self, fb: &mut Framebuffer) {
        let Some(b) = self.browse.as_ref() else { return };
        let m = self.mouse;
        let lib = &self.library;
        ui::panel(fb, BOX);
        fb.text("LIBRARY", BOX.x + 8, BOX.y + 5, ACCENT, Some(OUTLINE), 2);
        for (t, label) in ["SHIPS", "TEAMS"].iter().enumerate() {
            ui::button(fb, TAB_BTNS[t], label, TAB_BTNS[t].contains(m.0, m.1), b.teams == (t == 1));
        }
        ui::button(fb, BTN_CLOSE, "YARD", BTN_CLOSE.contains(m.0, m.1), false);

        // The list.
        ui::inset(fb, Rect::new(LIST.x - 1, LIST.y - 1, LIST.w + 2, LIST.h + 1));
        let n = self.count();
        for k in 0..ROWS {
            let i = b.top + k;
            if i >= n {
                break;
            }
            let r = row_rect(k);
            let (sel, hov) = (i == b.at, r.contains(m.0, m.1));
            if sel {
                fb.rect_fill(r.x, r.y, r.w, r.h, PANEL_LO);
                fb.rect_outline(r.x, r.y, r.w, r.h, ACCENT);
            } else if hov {
                fb.rect_fill(r.x, r.y, r.w, r.h, PANEL);
            }
            if b.teams {
                let Some((mine, t)) = lib.team(i) else { continue };
                let name = if sel && b.naming { format!("{}_", t.name) } else { t.name.chars().take(TEAM_NAME_MAX).collect() };
                fb.text(&name, r.x + 3, r.y + 4, if sel && b.naming { ACCENT } else if mine { TEXT } else { TEXT_DIM }, None, 1);
                for (s, d) in t.designs.iter().enumerate().filter(|(_, d)| !d.is_empty()) {
                    draw_icon(fb, d, Rect::new(r.x + 84 + s as i32 * 13, r.y + 1, 12, 13), 0);
                }
            } else {
                let Some((shelf, d)) = lib.ship(i) else { continue };
                draw_icon(fb, d, Rect::new(r.x + 1, r.y + 1, 13, 13), 0);
                let ok = d.check().ok();
                fb.text(&d.name, r.x + 18, r.y + 4, if !ok { BAD } else if sel { WHITE } else { TEXT }, None, 1);
                let cost = format!("{}", d.stats().cost);
                fb.text(&cost, r.x + 106 - Framebuffer::text_width(&cost, 1), r.y + 4, GOLD, None, 1);
                fb.text(&d.order.name()[..4], r.x + 112, r.y + 4, TEXT_DIM, None, 1);
                fb.text(shelf.tag(), r.x + r.w - 3 - Framebuffer::text_width(shelf.tag(), 1), r.y + 4, if shelf == Shelf::Mine { ACCENT } else { TEXT_FAINT }, None, 1);
            }
        }
        ui::button(fb, BTN_UP, "^", BTN_UP.contains(m.0, m.1), false);
        ui::button(fb, BTN_DOWN, "V", BTN_DOWN.contains(m.0, m.1), false);
        // Where the list is: first and last shown, of how many.
        fb.text(&format!("{}-{} OF {n}", (b.top + 1).min(n), (b.top + ROWS).min(n)), TAB_BTNS[1].x + TAB_BTNS[1].w + 8, TAB_BTNS[1].y + 4, TEXT_FAINT, None, 1);

        // The picked entry, large, and what it is.
        let y = |k: i32| INFO_Y + k * 9;
        if b.teams {
            fb.rect_fill(PREVIEW.x, PREVIEW.y, PREVIEW.w, PREVIEW.h, SPACE);
            if let Some((mine, t)) = lib.team(b.at) {
                for (s, d) in t.designs.iter().enumerate().filter(|(_, d)| !d.is_empty()) {
                    draw_icon(fb, d, Rect::new(PREVIEW.x + 1 + (s as i32 % 3) * 50, PREVIEW.y + (s as i32 / 3) * 28, 50, 28), 0);
                }
                let full: Vec<&Design> = t.designs.iter().filter(|d| !d.is_empty()).collect();
                fb.text(&t.name, PREVIEW.x, y(0), TEXT, None, 1);
                fb.text(&format!("{} OF {SLOTS} SLOTS FILLED", full.len()), PREVIEW.x, y(1), GOLD, None, 1);
                let bad = full.iter().filter(|d| !d.check().ok()).count();
                let (flies, col) = if bad == 0 { ("EVERY ONE CAN FLY".to_string(), GOOD) } else { (format!("{bad} CAN'T FLY YET"), BAD) };
                fb.text(&flies, PREVIEW.x, y(2), col, None, 1);
                fb.text(if mine { "ONE OF YOURS" } else { "READY-MADE" }, PREVIEW.x, y(3), TEXT_DIM, None, 1);
            }
        } else if let Some((shelf, d)) = lib.ship(b.at) {
            draw_preview(fb, d, PREVIEW, 0);
            let st = d.stats();
            fb.text(&d.name, PREVIEW.x, y(0), TEXT, None, 1);
            let cost = format!("{} PX", st.cost);
            fb.text(&cost, PREVIEW.x + PREVIEW.w - Framebuffer::text_width(&cost, 1), y(0), GOLD, None, 1);
            let how = match d.flies() {
                Some(style) if d.group > 1 => format!("{}, {}, IN {}S", d.order.name(), style.name(), d.group),
                Some(style) => format!("{}, {}", d.order.name(), style.name()),
                None => format!("{}, STAYS PUT", d.order.name()),
            };
            fb.text(&how, PREVIEW.x, y(1), TEXT_DIM, None, 1);
            let (why, col) = match d.check().problem() {
                None => ("READY TO FLY".to_string(), GOOD),
                Some(p) => (p.chars().take(25).collect(), BAD),
            };
            fb.text(&why, PREVIEW.x, y(2), col, None, 1);
            fb.text(shelf.from(), PREVIEW.x, y(3), TEXT_DIM, None, 1);
        } else {
            fb.rect_fill(PREVIEW.x, PREVIEW.y, PREVIEW.w, PREVIEW.h, SPACE);
        }

        // Your team, and the slot things are taken into and kept from.
        fb.text(&format!("YOUR TEAM: SLOT {}", self.slot + 1), PREVIEW.x, TEAM_Y, TEXT_DIM, None, 1);
        for (i, d) in self.lib.iter().enumerate() {
            let r = slot_rect(i);
            draw_icon(fb, d, r, 0);
            fb.rect_outline(r.x, r.y, r.w, r.h, if i == self.slot && !b.teams { ACCENT } else if r.contains(m.0, m.1) { WHITE } else { PANEL_LO });
        }
        let slot = self.slot + 1;
        let mine = if b.teams { b.at < lib.teams.len() } else { b.at < lib.ships.len() };
        ui::button(fb, BTN_TAKE, &if b.teams { "LOAD THIS TEAM".to_string() } else { format!("TAKE INTO SLOT {slot}") }, BTN_TAKE.contains(m.0, m.1), false);
        ui::button(fb, BTN_KEEP, &if b.teams { "KEEP TEAM".to_string() } else { format!("KEEP SLOT {slot}") }, BTN_KEEP.contains(m.0, m.1), false);
        if b.teams {
            ui::button(fb, BTN_NAME, "RENAME", BTN_NAME.contains(m.0, m.1), b.naming);
        }
        ui::button(fb, BTN_COPY, "COPY", BTN_COPY.contains(m.0, m.1), false);
        ui::button(fb, BTN_PASTE, "PASTE", BTN_PASTE.contains(m.0, m.1), false);
        ui::button(fb, BTN_DEL, "DELETE", BTN_DEL.contains(m.0, m.1) && mine, b.armed.is_some());

        // What was just done, or what the pointer is over.
        ui::inset(fb, TIP);
        let (text, col) = match &b.note {
            Some(note) => (note.clone(), GOLD),
            None => (self.library_tip(), TEXT),
        };
        for (i, l) in wrap(&text, TIP_COLS).iter().enumerate().take(4) {
            fb.text(l, TIP.x + 4, TIP.y + 5 + i as i32 * 9, col, None, 1);
        }
    }
}

/// Whether the library is what the yard's keys and clicks should go to.
pub fn open(pg: &Pg) -> bool {
    pg.browse.is_some() && pg.screen == Screen::Yard
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app::{App, Ev};

    fn yard() -> Pg {
        let mut pg = Pg::new(None, None);
        pg.help = false;
        pg.screen = Screen::Yard;
        pg
    }
    fn key(pg: &mut Pg, k: Key) {
        pg.event(Ev::Key(k));
    }
    fn click(pg: &mut Pg, r: Rect) {
        let (x, y) = (r.x + r.w / 2, r.y + r.h / 2);
        pg.event(Ev::Move(x, y));
        pg.event(Ev::Press { x, y, right: false });
        pg.event(Ev::Release { x, y, right: false });
    }
    fn lines(designs: &[Design]) -> Vec<String> {
        designs.iter().map(|d| d.to_line()).collect()
    }
    fn at(pg: &Pg) -> usize {
        pg.browse.as_ref().unwrap().at
    }
    fn note(pg: &Pg) -> String {
        pg.browse.as_ref().unwrap().note.clone().unwrap_or_default()
    }
    /// A drawing of your own: the starter's fighter under another name.
    fn own(name: &str) -> Design {
        let mut d = starter_deck()[0].clone();
        d.name = name.into();
        d
    }

    #[test]
    fn a_drawing_taken_from_the_library_fills_the_slot_and_what_it_replaced_is_kept() {
        let mut pg = yard();
        pg.lib[0] = own("MINE");
        pg.select_slot(0);
        // One pencil stroke on it, so undo has something of this drawing's to remember.
        let c = crate::pg::yard::CANVAS;
        let before = pg.lib[0].stats().count;
        pg.event(Ev::Press { x: c.x + 12, y: c.y + 12, right: false });
        pg.event(Ev::Release { x: c.x + 12, y: c.y + 12, right: false });
        assert_eq!(pg.lib[0].stats().count, before + 2, "the stroke (mirrored) should have added two pixels");
        let mine = pg.lib[0].to_line();
        key(&mut pg, Key::Letter('L'));
        assert!(pg.browse.is_some());
        // With nothing of yours kept yet, the first drawing listed is the beamer.
        let (shelf, beamer) = pg.library.ship(0).map(|(s, d)| (s, d.clone())).unwrap();
        assert_eq!((shelf, beamer.name.as_str()), (Shelf::Parts, "BEAMER"));
        key(&mut pg, Key::Enter);
        assert!(pg.browse.is_none(), "taking a drawing should go back to the yard");
        assert_eq!(pg.lib[0].to_line(), beamer.to_line(), "the slot doesn't hold the drawing taken");
        assert_eq!(pg.lib[0].order, beamer.order, "it didn't bring its order with it");
        assert_eq!(lines(&pg.library.ships), vec![mine.clone()], "the drawing it replaced wasn't kept");
        // Undo doesn't reach back into the drawing that was there.
        key(&mut pg, Key::Letter('U'));
        assert_eq!(pg.lib[0].to_line(), beamer.to_line(), "undo brought back pixels of the drawing that was replaced");
        // Taking yours back over the beamer keeps nothing more: the library has the beamer.
        key(&mut pg, Key::Letter('L'));
        assert_eq!(pg.library.ship(0).unwrap().0, Shelf::Mine);
        key(&mut pg, Key::Enter);
        assert_eq!(pg.lib[0].to_line(), mine);
        assert_eq!(pg.library.ships.len(), 1, "a drawing the library already had was kept again");
        // It's a copy: redrawn in the yard, the one in the library stays as it was.
        pg.lib[0].set(0, 0, Some(Part::Hull));
        pg.lib[0].order = Order::Sab;
        assert_eq!(pg.library.ships[0].to_line(), mine);
        // An empty slot takes a drawing and has nothing to keep.
        pg.select_slot(8);
        key(&mut pg, Key::Letter('L'));
        key(&mut pg, Key::Enter);
        assert_eq!(pg.lib[8].to_line(), mine);
        assert_eq!(pg.library.ships.len(), 1);
    }

    #[test]
    fn keeping_a_drawing_puts_it_first_and_only_once() {
        let mut pg = yard();
        pg.lib[2] = own("KEEPER");
        pg.select_slot(2);
        key(&mut pg, Key::Letter('L'));
        key(&mut pg, Key::Letter('K'));
        assert_eq!(lines(&pg.library.ships), vec![pg.lib[2].to_line()]);
        assert_eq!(at(&pg), 0, "what was just kept should be the one picked");
        assert!(note(&pg).contains("KEPT"));
        key(&mut pg, Key::Letter('K'));
        assert_eq!(pg.library.ships.len(), 1, "the same drawing was kept twice");
        assert!(note(&pg).contains("ALREADY"));
        // A ready-made drawing is in the library already, and an empty slot holds nothing.
        key(&mut pg, Key::Num(1));
        key(&mut pg, Key::Letter('K'));
        assert_eq!(pg.library.ships.len(), 1);
        key(&mut pg, Key::Num(9));
        key(&mut pg, Key::Letter('K'));
        assert_eq!(pg.library.ships.len(), 1);
        assert!(note(&pg).contains("NOTHING DRAWN"));
        // The same pixels on another order are another drawing: its order goes with it.
        key(&mut pg, Key::Num(3));
        pg.lib[2].order = Order::Sab;
        key(&mut pg, Key::Letter('K'));
        assert_eq!(pg.library.ships.len(), 2);
        assert_eq!(pg.library.ships[0].order, Order::Sab);
    }

    #[test]
    fn deleting_takes_two_presses_and_only_yours_go() {
        let mut pg = yard();
        pg.lib[0] = own("DOOMED");
        pg.select_slot(0);
        key(&mut pg, Key::Letter('L'));
        key(&mut pg, Key::Letter('K'));
        let n = pg.library.ship_count();
        key(&mut pg, Key::Letter('X'));
        assert_eq!(pg.library.ships.len(), 1, "one press deleted it");
        assert!(note(&pg).contains("AGAIN"));
        // Doing anything else lets go: the next press is a first press again.
        key(&mut pg, Key::Letter('S'));
        key(&mut pg, Key::Letter('W'));
        key(&mut pg, Key::Letter('X'));
        assert_eq!(pg.library.ships.len(), 1, "a delete that was let go of still went through");
        key(&mut pg, Key::Letter('X'));
        assert_eq!((pg.library.ships.len(), pg.library.ship_count()), (0, n - 1));
        // The ready-made ones stay, however often it's pressed.
        for _ in 0..3 {
            click(&mut pg, BTN_DEL);
        }
        assert_eq!(pg.library.ship_count(), n - 1);
        assert!(note(&pg).contains("READY-MADE"));
    }

    #[test]
    fn loading_a_team_swaps_all_nine_and_keeps_the_team_it_replaced() {
        let mut pg = yard();
        pg.lib[7] = part_examples()[0].clone();
        let mine = lines(&pg.lib);
        key(&mut pg, Key::Letter('L'));
        key(&mut pg, Key::Tab);
        assert_eq!((pg.library.teams.len(), pg.library.team_count()), (0, 3));
        // The second ready-made team is the new parts' examples.
        key(&mut pg, Key::Letter('S'));
        let parts = pg.library.team(at(&pg)).unwrap().1.clone();
        assert_eq!(parts.name, "NEW PARTS");
        key(&mut pg, Key::Enter);
        assert!(pg.browse.is_none());
        assert_eq!(lines(&pg.lib), lines(&team_of(parts_library())));
        assert_eq!(pg.lib.len(), SLOTS);
        assert_eq!(pg.library.teams.len(), 1, "the team it replaced wasn't kept");
        assert_eq!((pg.library.teams[0].name.as_str(), pg.library.teams[0].lines()), ("TEAM 1", mine.clone()));
        // Loading over a team the library has keeps nothing more.
        key(&mut pg, Key::Letter('L'));
        key(&mut pg, Key::Tab);
        key(&mut pg, Key::Enter);
        assert_eq!(lines(&pg.lib), mine);
        assert_eq!(pg.library.teams.len(), 1);
        // KEEP refuses a team just like one that's there, and takes one that isn't.
        key(&mut pg, Key::Letter('L'));
        key(&mut pg, Key::Tab);
        key(&mut pg, Key::Letter('K'));
        assert_eq!(pg.library.teams.len(), 1);
        assert!(note(&pg).contains("ALREADY"));
        pg.lib[8] = own("EXTRA");
        click(&mut pg, BTN_KEEP);
        assert_eq!(pg.library.teams.len(), 2);
        assert_eq!(pg.library.teams[0].name, "TEAM 2");
    }

    #[test]
    fn a_team_of_yours_is_renamed_by_typing() {
        let mut pg = yard();
        pg.lib[8] = own("EXTRA");
        key(&mut pg, Key::Letter('L'));
        key(&mut pg, Key::Tab);
        key(&mut pg, Key::Letter('K'));
        key(&mut pg, Key::Letter('N'));
        assert!(pg.browse.as_ref().unwrap().naming);
        for _ in 0..6 {
            key(&mut pg, Key::Back);
        }
        // While a name is typed, its letters are letters: K keeps nothing and L closes nothing.
        for c in "kill-swarm 2!?".chars() {
            pg.event(Ev::Char(c));
            if let Some(l) = c.to_ascii_uppercase().is_ascii_uppercase().then(|| c.to_ascii_uppercase()) {
                key(&mut pg, Key::Letter(l));
            }
        }
        assert!(pg.browse.is_some() && pg.library.teams.len() == 1);
        key(&mut pg, Key::Enter);
        assert!(!pg.browse.as_ref().unwrap().naming);
        assert_eq!(pg.library.teams[0].name, "KILL-SWARM 2", "twelve characters, and only letters, digits, spaces and dashes");
        // A name wiped out gets one back, and a ready-made team keeps its own.
        click(&mut pg, BTN_NAME);
        for _ in 0..12 {
            key(&mut pg, Key::Back);
        }
        key(&mut pg, Key::Enter);
        assert_eq!(pg.library.teams[0].name, "TEAM");
        key(&mut pg, Key::Letter('S'));
        key(&mut pg, Key::Letter('N'));
        assert!(!pg.browse.as_ref().unwrap().naming);
        assert!(note(&pg).contains("READY-MADE"));
    }

    #[test]
    fn a_drawing_goes_out_as_text_and_comes_back_the_same() {
        let mut pg = yard();
        pg.lib[0] = own("SENDER");
        pg.lib[0].order = Order::Sab;
        (pg.lib[0].style, pg.lib[0].group) = (Some(Style::Orbit), 6);
        let sent = pg.lib[0].to_line();
        pg.select_slot(0);
        key(&mut pg, Key::Letter('L'));
        key(&mut pg, Key::Letter('K'));
        key(&mut pg, Key::Letter('C'));
        assert_eq!(crate::clip::get().unwrap().trim(), sent, "what's on the clipboard isn't the drawing's line");
        // The other player's game: nothing kept, the same clipboard.
        let mut theirs = yard();
        key(&mut theirs, Key::Letter('L'));
        click(&mut theirs, BTN_PASTE);
        assert_eq!(lines(&theirs.library.ships), vec![sent.clone()]);
        assert!(note(&theirs).contains("PASTED 1 DRAWING"));
        key(&mut theirs, Key::Letter('V'));
        assert_eq!(theirs.library.ships.len(), 1, "pasting the same thing twice kept it twice");
        // A whole team the same way: its name and all nine slots.
        pg.lib[8] = part_examples()[1].clone();
        let team = lines(&pg.lib);
        key(&mut pg, Key::Tab);
        key(&mut pg, Key::Letter('K'));
        click(&mut pg, BTN_COPY);
        key(&mut theirs, Key::Letter('V'));
        assert_eq!(theirs.library.teams.len(), 1);
        assert_eq!((theirs.library.teams[0].name.as_str(), theirs.library.teams[0].lines()), ("TEAM 1", team));
        assert!(theirs.browse.as_ref().unwrap().teams, "a pasted team should be shown on the teams tab");
    }

    #[test]
    fn text_from_outside_becomes_drawings_and_rubbish_adds_nothing() {
        let fighter = starter_deck()[0].to_line();
        let ships = |text: &str| read_shared(text).iter().filter(|p| matches!(p, Pasted::Ship(_))).count();
        // A line out of a chat, with words round it.
        assert_eq!(ships(&format!("try this one:\r\n\r\n  {fighter}  \r\nsee what you think")), 1);
        match &read_shared(&fighter)[0] {
            Pasted::Ship(d) => assert_eq!(d.to_line(), fighter),
            other => panic!("{other:?}"),
        }
        // Rubbish: none of it is a drawing, and none of it stops the game.
        let long_row = format!("A|HUNT|40,40|{}|BUILT|", "c".repeat(100_000));
        let many_rows = format!("A|HUNT|0,0|{}|BUILT|", vec!["c"; 5000].join("/"));
        for text in ["", "\n\n\n", "TEAM", "TEAM NOBODY", "hello|there", "A|NOPE|0,0|c|BUILT|", "A|HUNT|x,y|c|BUILT|", "A|HUNT|999999999999,0|c|BUILT|", "A|HUNT|2147483647,2147483647|c|BUILT|", "A|HUNT|-5,3|c|BUILT|", "|||||||", "A|HUNT", "\u{0}\u{1}\u{fffd}|HUNT|\u{202e}"] {
            assert_eq!(read_shared(text).len(), 0, "{text:?} was read as something");
        }
        // Lines that are too long or too many are still only drawings on the grid.
        for text in [long_row.as_str(), many_rows.as_str()] {
            for p in read_shared(text) {
                match p {
                    Pasted::Ship(d) => assert!(d.cells.len() == (GRID * GRID) as usize),
                    Pasted::Team(_) => panic!("a team from one line"),
                }
            }
        }
        // A name is made fit to show, and one that's nothing gets a plain one.
        let named = |name: &str| match &read_shared(&format!("{name}|HUNT|48,48|c|BUILT|"))[0] {
            Pasted::Ship(d) => d.name.clone(),
            other => panic!("{other:?}"),
        };
        assert_eq!(named("<b>evil one</b>"), "BEVIL ONEB");
        assert_eq!(named("a name far too long to show"), "A NAME FAR");
        assert_eq!(named("%%%"), "PASTED");
        // One paste adds so many at most, and a team takes nine of what follows it.
        let flood: String = (0..100).map(|i| format!("S{i}|HUNT|48,48|c|BUILT|\n")).collect();
        assert_eq!(read_shared(&flood).len(), PASTE_MAX);
        let team: String = std::iter::once("TEAM big one".to_string()).chain((0..12).map(|i| format!("S{i}|HUNT|48,48|c|BUILT|"))).collect::<Vec<_>>().join("\n");
        let got = read_shared(&team);
        match &got[0] {
            Pasted::Team(t) => assert_eq!((t.name.as_str(), t.designs.len(), t.designs[8].name.as_str()), ("BIG ONE", SLOTS, "S8")),
            other => panic!("{other:?}"),
        }
        assert_eq!(got.len(), 4, "the three past the ninth are drawings on their own");
        // Through the screen: rubbish says so and keeps nothing.
        let mut pg = yard();
        key(&mut pg, Key::Letter('L'));
        pg.paste_text("nothing to see here");
        assert_eq!((pg.library.ships.len(), pg.library.teams.len()), (0, 0));
        assert!(note(&pg).contains("NOTHING ON THE CLIPBOARD"));
        pg.paste_text(&flood);
        assert_eq!(pg.library.ships.len(), PASTE_MAX);
        assert_eq!(pg.library.ships[0].name, "S0", "the first pasted should be listed first");
    }

    #[test]
    fn the_library_file_holds_your_drawings_and_teams_and_reads_back_the_same() {
        let path = std::env::temp_dir().join(format!("nmw_library_{}.txt", std::process::id()));
        let _ = std::fs::remove_file(&path);
        let mut lib = Library::load(Some(path.clone()));
        assert_eq!((lib.ships.len(), lib.teams.len()), (0, 0));
        let mut orbiter = own("ORBITER");
        (orbiter.style, orbiter.group, orbiter.order) = (Some(Style::Orbit), 3, Order::Guard);
        lib.keep(&orbiter).unwrap();
        lib.keep(&own("SECOND")).unwrap();
        let mut team = team_of(starter_deck());
        team[8] = orbiter.clone();
        assert_eq!(lib.keep_team(&team, None).unwrap(), "TEAM 1");
        assert_eq!(lib.keep_team(&team_of(vec![orbiter.clone()]), Some("SOLO")).unwrap(), "SOLO");
        let again = Library::load(Some(path.clone()));
        assert_eq!(again.text(), lib.text());
        assert_eq!(lines(&again.ships), vec![own("SECOND").to_line(), orbiter.to_line()], "newest first");
        assert_eq!(again.teams.iter().map(|t| t.name.as_str()).collect::<Vec<_>>(), vec!["SOLO", "TEAM 1"]);
        assert_eq!(again.teams[1].lines(), lines(&team));
        assert!(!path.with_extension("tmp").exists(), "the half-written copy was left behind");
        // Deleting is saved too.
        lib.delete_ship(0);
        lib.delete_team(1);
        let again = Library::load(Some(path.clone()));
        assert_eq!((again.ships.len(), again.teams.len()), (1, 1));
        // A file that isn't a library, or has lines that aren't drawings, gives what it can.
        std::fs::write(&path, "not a library\nSHIP junk\n").unwrap();
        assert_eq!(Library::load(Some(path.clone())).ships.len(), 0);
        std::fs::write(&path, format!("{LIB_HEADER}\nSHIP junk\nSLOT {}\nSHIP {}\nTEAM \nSLOT {}\nwhat\n", orbiter.to_line(), orbiter.to_line(), orbiter.to_line())).unwrap();
        let odd = Library::load(Some(path.clone()));
        assert_eq!(lines(&odd.ships), vec![orbiter.to_line()], "a SLOT line before any TEAM belongs to nothing");
        assert_eq!((odd.teams.len(), odd.teams[0].name.as_str(), odd.teams[0].designs.len()), (1, "TEAM", SLOTS));
        let _ = std::fs::remove_file(&path);
        // The ready-made shelves are never written: they follow the game.
        assert!(!lib.text().contains("BEAMER"));
        assert!(lib.has(&part_examples()[0]) && lib.has_team(&team_of(starter_deck())));
    }

    #[test]
    fn everything_in_the_library_is_a_key_and_a_button() {
        let mut pg = yard();
        // In and out: the L key and the LIBRARY button; L, ESC, the YARD button and a click
        // outside the panel.
        click(&mut pg, crate::pg::yard::BTN_LIBRARY);
        assert!(pg.browse.is_some());
        click(&mut pg, BTN_CLOSE);
        assert!(pg.browse.is_none());
        key(&mut pg, Key::Letter('L'));
        key(&mut pg, Key::Letter('L'));
        assert!(pg.browse.is_none());
        key(&mut pg, Key::Letter('L'));
        key(&mut pg, Key::Esc);
        assert!(pg.browse.is_none() && !pg.confirm_quit, "ESC should close the library, not ask to quit");
        key(&mut pg, Key::Letter('L'));
        click(&mut pg, Rect::new(0, 140, 4, 4));
        assert!(pg.browse.is_none());
        // The two tabs.
        key(&mut pg, Key::Letter('L'));
        key(&mut pg, Key::Tab);
        assert!(pg.browse.as_ref().unwrap().teams);
        click(&mut pg, TAB_BTNS[0]);
        assert!(!pg.browse.as_ref().unwrap().teams);
        click(&mut pg, TAB_BTNS[1]);
        assert!(pg.browse.as_ref().unwrap().teams);
        key(&mut pg, Key::Tab);
        // Picking: a click on a row, W and S, and the arrows (one step a tap, then steadily).
        let n = pg.library.ship_count();
        assert!(n > ROWS, "the ready-made drawings should overflow the list, to test scrolling");
        click(&mut pg, row_rect(2));
        assert_eq!(at(&pg), 2);
        key(&mut pg, Key::Letter('S'));
        key(&mut pg, Key::Letter('S'));
        key(&mut pg, Key::Letter('W'));
        assert_eq!(at(&pg), 3);
        pg.hold([false, true, false, false]);
        pg.tick();
        pg.hold([false; 4]);
        pg.tick();
        pg.tick();
        assert_eq!(at(&pg), 4, "a tap on the down arrow should step once");
        pg.hold([true, false, false, false]);
        for _ in 0..40 {
            pg.tick();
        }
        pg.hold([false; 4]);
        pg.tick();
        pg.tick();
        assert_eq!(at(&pg), 0, "holding the up arrow should run to the top and stop there");
        // The list follows what's picked, and the wheel and the two arrows scroll it.
        for _ in 0..n + 3 {
            key(&mut pg, Key::Letter('S'));
        }
        assert_eq!((at(&pg), pg.browse.as_ref().unwrap().top), (n - 1, n - ROWS));
        pg.event(Ev::Wheel(1));
        assert_eq!(pg.browse.as_ref().unwrap().top, n - ROWS - 1);
        click(&mut pg, BTN_UP);
        assert_eq!(pg.browse.as_ref().unwrap().top, (n - ROWS).saturating_sub(4));
        pg.event(Ev::Wheel(-50));
        assert_eq!(pg.browse.as_ref().unwrap().top, n - ROWS);
        click(&mut pg, BTN_UP);
        click(&mut pg, BTN_DOWN);
        assert_eq!(pg.browse.as_ref().unwrap().top, n - ROWS);
        // A click on a row that's scrolled picks what's shown there.
        click(&mut pg, row_rect(0));
        assert_eq!(at(&pg), n - ROWS);
        // The slot: the number keys and the nine boxes.
        key(&mut pg, Key::Num(9));
        assert_eq!(pg.slot, 8);
        click(&mut pg, slot_rect(4));
        assert_eq!(pg.slot, 4);
        // TAKE: ENTER and the button.
        let want = pg.library.ship(at(&pg)).unwrap().1.to_line();
        click(&mut pg, BTN_TAKE);
        assert!(pg.browse.is_none());
        assert_eq!(pg.lib[4].to_line(), want);
        // KEEP, COPY, PASTE and DELETE as buttons (their keys are in the tests above).
        pg.lib[4] = own("BUTTONS");
        key(&mut pg, Key::Letter('L'));
        click(&mut pg, BTN_KEEP);
        // (The outpost that TAKE replaced is the starter deck's, so it wasn't kept again.)
        assert_eq!(pg.library.ships.len(), 1);
        click(&mut pg, BTN_COPY);
        assert_eq!(crate::clip::get().unwrap().trim(), pg.lib[4].to_line());
        click(&mut pg, BTN_DEL);
        click(&mut pg, BTN_DEL);
        assert_eq!(pg.library.ships.len(), 0);
        click(&mut pg, BTN_PASTE);
        assert_eq!(pg.library.ships.len(), 1);
        // F1 still opens the rules over it, and closing them leaves the library open.
        key(&mut pg, Key::Help);
        assert!(pg.help);
        key(&mut pg, Key::Space);
        assert!(!pg.help && pg.browse.is_some());
        // The yard's own keys don't reach through: 2 picks a slot here, not a part category.
        let cat = pg.ed.cat;
        key(&mut pg, Key::Num(2));
        assert!(pg.ed.cat == cat && pg.slot == 1);
    }

    #[test]
    fn every_line_of_the_library_fits_where_its_shown() {
        let fits = |text: &str| assert!(wrap(text, TIP_COLS).len() <= 4, "{} lines for a box of 4: {text}", wrap(text, TIP_COLS).len());
        let mut pg = yard();
        pg.select_slot(8);
        key(&mut pg, Key::Letter('L'));
        // The hover text for everything, on both tabs.
        let spots: Vec<Rect> = [BTN_TAKE, BTN_KEEP, BTN_NAME, BTN_COPY, BTN_PASTE, BTN_DEL, BTN_CLOSE, TAB_BTNS[0], TAB_BTNS[1], BTN_UP, BTN_DOWN, slot_rect(8), row_rect(0), TIP].into();
        for teams in [false, true] {
            pg.set_tab(teams);
            for r in &spots {
                pg.event(Ev::Move(r.x + 2, r.y + 2));
                let tip = pg.library_tip();
                assert!(!tip.is_empty());
                fits(&tip);
            }
        }
        // What it says back, with the longest names there can be.
        pg.set_tab(false);
        pg.lib[8] = own("WWWWWWWWWW");
        let mut said: Vec<String> = Vec::new();
        let mut say = |pg: &Pg| said.push(note(pg));
        for k in ['K', 'K', 'C', 'X', 'X', 'N', 'V'] {
            key(&mut pg, Key::Letter(k));
            say(&pg);
        }
        crate::clip::set("nothing");
        key(&mut pg, Key::Letter('V'));
        say(&pg);
        key(&mut pg, Key::Tab);
        for k in ['K', 'K', 'C', 'X', 'X'] {
            key(&mut pg, Key::Letter(k));
            say(&pg);
        }
        key(&mut pg, Key::Letter('S'));
        for k in ['X', 'N'] {
            key(&mut pg, Key::Letter(k));
            say(&pg);
        }
        assert!(said.iter().all(|s| !s.is_empty()), "something said nothing: {said:?}");
        for s in &said {
            fits(s);
        }
        // The buttons' own words, at their longest, and the four lines under the preview.
        let room = |r: Rect, label: &str| assert!(Framebuffer::text_width(label, 1) + 4 <= r.w, "{label} is too wide for its button");
        room(BTN_TAKE, "TAKE INTO SLOT 9");
        room(BTN_TAKE, "LOAD THIS TEAM");
        room(BTN_KEEP, "KEEP SLOT 9");
        room(BTN_KEEP, "KEEP TEAM");
        room(BTN_NAME, "RENAME");
        room(BTN_COPY, "COPY");
        room(BTN_PASTE, "PASTE");
        room(BTN_DEL, "DELETE");
        for shelf in [Shelf::Mine, Shelf::Parts, Shelf::Starter, Shelf::Reference] {
            assert!(Framebuffer::text_width(shelf.from(), 1) <= PREVIEW.w, "{}", shelf.from());
        }
        for o in ORDERS {
            for st in STYLES {
                assert!(Framebuffer::text_width(&format!("{}, {}, IN 12S", o.name(), st.name()), 1) <= PREVIEW.w);
            }
        }
        // The panel holds everything, the nine slot boxes too, and the rows keep clear of the tip.
        for r in [BTN_TAKE, BTN_KEEP, BTN_NAME, BTN_COPY, BTN_PASTE, BTN_DEL, BTN_CLOSE, PREVIEW, TIP, LIST, BTN_UP, BTN_DOWN, slot_rect(8), TAB_BTNS[1]] {
            assert!(r.x >= BOX.x + 2 && r.y >= BOX.y + 2 && r.x + r.w <= BOX.x + BOX.w - 2 && r.y + r.h <= BOX.y + BOX.h - 2, "{r:?} is off the panel");
        }
        assert!(row_rect(ROWS - 1).y + ROW <= TIP.y && BTN_DEL.y + BTN_DEL.h <= TIP.y && slot_rect(0).y + 16 <= BTN_TAKE.y);
        // A team's name at its longest stops short of its row of icons, and a row of nine fits.
        assert!(3 + Framebuffer::text_width(&"W".repeat(TEAM_NAME_MAX + 1), 1) <= 84 && 84 + 9 * 13 <= LIST.w);
        for t in 0..pg.library.team_count() {
            assert!(pg.library.team(t).unwrap().1.name.chars().count() <= TEAM_NAME_MAX);
        }
    }
}
