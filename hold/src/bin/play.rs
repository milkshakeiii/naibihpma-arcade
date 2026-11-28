//! Play the game turn by turn from a script, for testing without a browser. Each run replays the
//! whole script from the start (the game is deterministic for a seed) and writes a 2x screenshot
//! of where it ends, so a player can add a line, run it again, and look.
//!
//!     cargo run --release --bin play -- moves.txt shot.ppm
//!
//! One command a line (`#` starts a comment). Coordinates are the game's own 400 x 300 pixels:
//!     wait N            let N frames pass (15 a second)
//!     click X Y         left click        rclick X Y    right click
//!     move X Y          move the pointer (the view scrolls when it's at an edge)
//!     key NAME          tab, shift-tab, enter, esc, backspace, up, down, left, right,
//!                       home, space, f2 ... f9
//!     type TEXT         type these characters (letters are button keys)

use hold::game::{Game, In, K_BACK, K_DOWN, K_ENTER, K_ESC, K_F2, K_F3, K_F4, K_F5, K_F6, K_F7, K_F8, K_F9, K_HOME, K_LEFT, K_RIGHT, K_TAB, K_UP};

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let script = std::fs::read_to_string(args.get(1).expect("a moves file")).unwrap_or_default();
    let out = args.get(2).map(String::as_str).unwrap_or("shot.ppm");
    let mut g = Game::new(11);
    g.frame();
    for (n, line) in script.lines().enumerate() {
        let line = line.split('#').next().unwrap().trim();
        if line.is_empty() {
            continue;
        }
        let mut parts = line.splitn(2, ' ');
        let cmd = parts.next().unwrap().to_lowercase();
        let rest = parts.next().unwrap_or("").trim();
        let nums: Vec<i32> = rest.split_whitespace().filter_map(|s| s.parse().ok()).collect();
        match cmd.as_str() {
            "wait" => {
                for _ in 0..nums.first().copied().unwrap_or(15).clamp(0, 15 * 600) {
                    g.frame();
                }
            }
            "click" | "rclick" if nums.len() >= 2 => {
                let b = if cmd == "click" { 0 } else { 2 };
                g.push(In::Move(nums[0], nums[1]));
                g.push(In::Down(b, nums[0], nums[1]));
                g.push(In::Up(b, nums[0], nums[1]));
                g.frame();
            }
            "move" if nums.len() >= 2 => {
                g.push(In::Move(nums[0], nums[1]));
                g.frame();
            }
            "key" => {
                let (k, shift) = match rest.to_lowercase().as_str() {
                    "tab" => (K_TAB, false),
                    "shift-tab" => (K_TAB, true),
                    "enter" => (K_ENTER, false),
                    "esc" | "escape" => (K_ESC, false),
                    "backspace" => (K_BACK, false),
                    "up" => (K_UP, false),
                    "down" => (K_DOWN, false),
                    "left" => (K_LEFT, false),
                    "right" => (K_RIGHT, false),
                    "home" => (K_HOME, false),
                    "f2" => (K_F2, false),
                    "f3" => (K_F3, false),
                    "f4" => (K_F4, false),
                    "f5" => (K_F5, false),
                    "f6" => (K_F6, false),
                    "f7" => (K_F7, false),
                    "f8" => (K_F8, false),
                    "f9" => (K_F9, false),
                    "space" => {
                        g.push(In::Char(' '));
                        g.frame();
                        continue;
                    }
                    other => {
                        eprintln!("line {}: unknown key {other}", n + 1);
                        continue;
                    }
                };
                g.push(In::Key(k, shift));
                g.frame();
            }
            "type" => {
                for c in rest.chars() {
                    g.push(In::Char(c));
                    g.frame();
                }
            }
            _ => eprintln!("line {}: didn't understand \"{line}\"", n + 1),
        }
    }
    let n = 2usize;
    let px = g.present(n).to_vec();
    let (w, h) = (400 * n, 300 * n);
    let mut ppm = format!("P6\n{w} {h}\n255\n").into_bytes();
    for p in px.chunks(4) {
        ppm.extend_from_slice(&p[..3]);
    }
    std::fs::write(out, ppm).expect("write the screenshot");
    let s = g.frames / 15;
    println!("{} frames played ({}:{:02} of game time)", g.frames, s / 60, s % 60);
}
