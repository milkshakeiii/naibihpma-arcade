//! Write a screenshot of the game after a scripted start, for checking the look headless.
//!     cargo run --release --bin snap -- <scene> out.ppm
//! Scenes: title, start (the guided start), hold (ten minutes in, no lesson), night, stock, info.

use hold::art::palette;
use hold::fb::{FB_H, FB_W};
use hold::game::{Game, In, Mode, Screen};

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let scene = args.get(1).map(String::as_str).unwrap_or("hold");
    let out = args.get(2).map(String::as_str).unwrap_or("snap.ppm");
    let mut g = Game::new(5);
    match scene {
        "title" => {}
        "start" => {
            g.push(In::Char('s'));
            for _ in 0..60 {
                g.frame();
            }
        }
        _ => {
            g.push(In::Char('s'));
            g.frame();
            g.lesson = hold::game::Lesson::Done;
            g.world.lesson = false;
            let mut bot = hold::bot::Bot::new(hold::bot::Style::Steady);
            let mins = match scene { "night" => 4, "day" => 2, "late" => 25, _ => 10 };
            while g.world.t < mins * 900 + 60 {
                bot.step(&mut g.world);
                g.world.tick();
                g.world.events.clear();
            }
            if scene == "stock" {
                let b = g.world.find_bld(hold::data::BKind::ToadHall).unwrap();
                g.sel = hold::game::Sel::B(b);
                g.mode = Mode::Stock(b);
            }
            if scene == "info" {
                g.mode = Mode::Info(hold::game::Info::Heroes);
            }
            g.frame();
        }
    }
    let _ = Screen::Play;
    let n = 2usize;
    let mut rgba = Vec::new();
    let pal = palette(g.frames, Default::default());
    let _ = pal;
    let px = g.present(n).to_vec();
    rgba.extend_from_slice(&px);
    let (w, h) = (FB_W as usize * n, FB_H as usize * n);
    let mut ppm = format!("P6\n{w} {h}\n255\n").into_bytes();
    for p in rgba.chunks(4) {
        ppm.extend_from_slice(&p[..3]);
    }
    std::fs::write(out, ppm).expect("write");
    println!("wrote {out}");
}
