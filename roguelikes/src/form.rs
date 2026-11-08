//! The end-of-run feedback form: six 1-5 ratings and two optional free-text answers.

use crate::app::{Ev, Key};
use crate::audio::Sfx;
use crate::framebuffer::{wrap, Framebuffer};
use crate::palette::*;
use crate::ui::{self, Rect};

pub struct Question {
    pub id: &'static str,
    pub text: &'static str,
    pub low: &'static str,
    pub high: &'static str,
}

pub const COMMON: [Question; 5] = [
    Question { id: "fun", text: "HOW MUCH FUN WAS IT?", low: "NONE", high: "LOTS" },
    Question { id: "clarity", text: "DID OUTCOMES MAKE SENSE?", low: "NEVER", high: "ALWAYS" },
    Question { id: "decisions", text: "WERE YOUR CHOICES INTERESTING?", low: "DULL", high: "RICH" },
    Question { id: "difficulty", text: "HOW HARD WAS IT?", low: "TOO EASY", high: "TOO HARD" },
    Question { id: "again", text: "WANT ANOTHER RUN OF THIS ONE?", low: "NO", high: "RIGHT NOW" },
];

pub const TEXT_PROMPTS: [(&str, &str); 2] = [
    ("best", "THE BEST OR MOST INTERESTING MOMENT:"),
    ("change", "WHAT BUGGED YOU, OR WHAT WOULD YOU CHANGE?"),
];

const MAX_TEXT: usize = 900;
const ROW_Y: i32 = 46;
const ROW_H: i32 = 15;
const BOX_X: i32 = 196;
const TEXT_COLS: usize = 62;
const TEXT_LINES: usize = 4;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Focus {
    Rating(usize),
    Text(usize),
    Save,
}

pub struct Form {
    pub heading: String,
    pub summary: String,
    pub questions: Vec<&'static Question>,
    pub ratings: Vec<Option<u8>>,
    pub texts: [String; 2],
    focus: Focus,
    mouse: (i32, i32),
    pub done: bool,
}

impl Form {
    pub fn new(heading: String, summary: String, special: &'static Question) -> Self {
        let mut questions: Vec<&'static Question> = COMMON.iter().collect();
        questions.push(special);
        let n = questions.len();
        Self {
            heading,
            summary,
            questions,
            ratings: vec![None; n],
            texts: [String::new(), String::new()],
            focus: Focus::Rating(0),
            mouse: (-1, -1),
            done: false,
        }
    }

    fn order(&self) -> Vec<Focus> {
        let mut v: Vec<Focus> = (0..self.questions.len()).map(Focus::Rating).collect();
        v.extend([Focus::Text(0), Focus::Text(1), Focus::Save]);
        v
    }

    fn step_focus(&mut self, d: i32) {
        let o = self.order();
        let i = o.iter().position(|f| *f == self.focus).unwrap_or(0) as i32;
        self.focus = o[(i + d).clamp(0, o.len() as i32 - 1) as usize];
    }

    fn box_rect(i: usize, v: usize) -> Rect {
        Rect::new(BOX_X + v as i32 * 17, ROW_Y + i as i32 * ROW_H - 2, 15, 12)
    }

    fn text_rect(&self, i: usize) -> Rect {
        let y0 = ROW_Y + self.questions.len() as i32 * ROW_H + 10;
        Rect::new(8, y0 + 10 + i as i32 * 62, 384, 46)
    }

    fn save_rect() -> Rect {
        Rect::new(292, 272, 100, 20)
    }

    pub fn event(&mut self, ev: Ev, sfx: &mut Vec<Sfx>) {
        match ev {
            Ev::Move(x, y) => self.mouse = (x, y),
            Ev::Press { x, y, right: false } => {
                for i in 0..self.questions.len() {
                    for v in 0..5 {
                        if Self::box_rect(i, v).contains(x, y) {
                            self.ratings[i] = Some(v as u8 + 1);
                            self.focus = Focus::Rating(i);
                            sfx.push(Sfx::Click);
                        }
                    }
                }
                for i in 0..2 {
                    if self.text_rect(i).contains(x, y) {
                        self.focus = Focus::Text(i);
                        sfx.push(Sfx::Click);
                    }
                }
                if Self::save_rect().contains(x, y) {
                    self.done = true;
                    sfx.push(Sfx::Buy);
                }
            }
            Ev::Key(k) => {
                let in_text = matches!(self.focus, Focus::Text(_));
                match k {
                    Key::Tab | Key::Down => self.step_focus(1),
                    Key::Up => self.step_focus(-1),
                    Key::Enter => {
                        if self.focus == Focus::Save {
                            self.done = true;
                            sfx.push(Sfx::Buy);
                        } else {
                            self.step_focus(1);
                        }
                    }
                    Key::Left | Key::Right if !in_text => {
                        if let Focus::Rating(i) = self.focus {
                            let cur = self.ratings[i].unwrap_or(3) as i32;
                            let next = if k == Key::Left { cur - 1 } else { cur + 1 };
                            self.ratings[i] = Some(next.clamp(1, 5) as u8);
                            sfx.push(Sfx::Click);
                        }
                    }
                    _ => {}
                }
            }
            Ev::Char(c) => match self.focus {
                Focus::Text(i) => {
                    if c == '\u{8}' {
                        self.texts[i].pop();
                        sfx.push(Sfx::Key);
                    } else if (' '..='~').contains(&c) && self.texts[i].len() < MAX_TEXT {
                        self.texts[i].push(c);
                        sfx.push(Sfx::Key);
                    }
                }
                Focus::Rating(i) => {
                    if let Some(d) = c.to_digit(10).filter(|d| (1..=5).contains(d)) {
                        self.ratings[i] = Some(d as u8);
                        sfx.push(Sfx::Click);
                        self.step_focus(1);
                    }
                }
                Focus::Save => {}
            },
            _ => {}
        }
    }

    /// The answers as log lines.
    pub fn report(&self) -> Vec<String> {
        let mut v = Vec::new();
        for (q, r) in self.questions.iter().zip(&self.ratings) {
            let val = r.map_or("-".to_string(), |r| r.to_string());
            v.push(format!("rating {:<10} {val}  ({}  1={} 5={})", q.id, q.text.to_lowercase(), q.low.to_lowercase(), q.high.to_lowercase()));
        }
        for ((id, _), t) in TEXT_PROMPTS.iter().zip(&self.texts) {
            v.push(format!("text   {id:<10} {}", if t.trim().is_empty() { "-" } else { t.trim() }));
        }
        v
    }

    pub fn draw(&self, fb: &mut Framebuffer) {
        fb.clear(2);
        for y in (0..300).step_by(4) {
            fb.rect_fill(0, y, 400, 1, 3);
        }
        ui::panel(fb, Rect::new(2, 2, 396, 296));
        fb.text(&self.heading, 10, 8, ACCENT_HI, Some(OUTLINE), 1);
        let lines = wrap(&self.summary, 63);
        for (i, l) in lines.iter().take(2).enumerate() {
            fb.text(l, 10, 20 + i as i32 * 10, TEXT_DIM, None, 1);
        }
        for (i, q) in self.questions.iter().enumerate() {
            let y = ROW_Y + i as i32 * ROW_H;
            let focused = self.focus == Focus::Rating(i);
            if focused {
                fb.text("\u{25B6}", 2, y, ACCENT, None, 1);
            }
            fb.text(q.text, 10, y, if focused { WHITE } else { TEXT }, None, 1);
            for v in 0..5 {
                let r = Self::box_rect(i, v);
                let chosen = self.ratings[i] == Some(v as u8 + 1);
                let hover = r.contains(self.mouse.0, self.mouse.1);
                if chosen {
                    fb.rect_bevel(r.x, r.y, r.w, r.h, ACCENT, ACCENT_HI, PANEL_LO);
                } else {
                    fb.rect_bevel(r.x, r.y, r.w, r.h, if hover { PANEL_HI } else { PANEL_DEEP }, PANEL_LO, PANEL_HI);
                }
                let col = if chosen { PANEL_DEEP } else { TEXT_DIM };
                fb.text(&(v + 1).to_string(), r.x + 5, r.y + 2, col, None, 1);
            }
            fb.text(&format!("{}-{}", q.low, q.high), BOX_X + 90, y, TEXT_DIM, None, 1);
        }
        for i in 0..2 {
            let r = self.text_rect(i);
            let focused = self.focus == Focus::Text(i);
            fb.text(TEXT_PROMPTS[i].1, 10, r.y - 10, if focused { WHITE } else { TEXT }, None, 1);
            if focused {
                fb.text("\u{25B6}", 2, r.y - 10, ACCENT, None, 1);
            }
            ui::inset(fb, r);
            let mut lines = wrap(&self.texts[i], TEXT_COLS);
            if focused {
                // A steady (never blinking) block cursor after the last character.
                let last = lines.pop().unwrap_or_default();
                if last.chars().count() >= TEXT_COLS {
                    lines.push(last);
                    lines.push("\u{2588}".to_string());
                } else {
                    lines.push(format!("{last}\u{2588}"));
                }
            }
            let skip = lines.len().saturating_sub(TEXT_LINES);
            for (k, l) in lines.iter().skip(skip).enumerate() {
                fb.text(l, r.x + 5, r.y + 4 + k as i32 * 10, TEXT, None, 1);
            }
            if self.texts[i].is_empty() && !focused {
                fb.text("(OPTIONAL. CLICK HERE AND TYPE.)", r.x + 5, r.y + 4, TEXT_FAINT, None, 1);
            }
        }
        let s = Self::save_rect();
        let hover = s.contains(self.mouse.0, self.mouse.1) || self.focus == Focus::Save;
        ui::button(fb, s, "SAVE", hover, true);
        let hint = match self.focus {
            Focus::Rating(_) => "CLICK A NUMBER, OR PRESS 1-5. TAB MOVES ON.",
            Focus::Text(_) => "TYPE YOUR ANSWER. ENTER OR TAB MOVES ON.",
            Focus::Save => "ENTER SAVES YOUR ANSWERS.",
        };
        fb.text(hint, 10, 278, TEXT_DIM, None, 1);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    static SPECIAL: Question = Question { id: "special", text: "TEST?", low: "A", high: "B" };

    #[test]
    fn typing_digits_rates_and_text_is_recorded() {
        let mut f = Form::new("H".into(), "S".into(), &SPECIAL);
        let mut sfx = Vec::new();
        for c in "435215".chars() {
            f.event(Ev::Char(c), &mut sfx);
        }
        assert_eq!(f.ratings, vec![Some(4), Some(3), Some(5), Some(2), Some(1), Some(5)]);
        for c in "hello world\u{8}D".chars() {
            f.event(Ev::Char(c), &mut sfx);
        }
        f.event(Ev::Key(Key::Enter), &mut sfx);
        f.event(Ev::Key(Key::Enter), &mut sfx);
        f.event(Ev::Key(Key::Enter), &mut sfx);
        assert!(f.done);
        let r = f.report();
        assert!(r.iter().any(|l| l.contains("hello worlD")));
        assert!(r.iter().any(|l| l.starts_with("rating special    5")));
    }
}
