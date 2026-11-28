//! Headless snapshots for checking the art: runs a scripted scene and writes the exact
//! framebuffer at scale 3 as a PPM (convert with ffmpeg).
//! `cargo run --release --bin snap -- <scene> <out.ppm>`; scenes: title, start, watch, battle.
use hold::game::{Game, In, K_ENTER, K_TAB};
use hold::sim::*;

fn main() {
    let scene = std::env::args().nth(1).unwrap_or("start".into());
    let out = std::env::args().nth(2).unwrap_or("snap.ppm".into());
    let mut g = Game::new(7);
    let frames = |g: &mut Game, n: u32| {
        for _ in 0..n {
            g.frame();
        }
    };
    if scene != "title" {
        g.push(In::Key(K_ENTER, false));
        frames(&mut g, 95);
    }
    if scene == "watch" || scene == "battle" {
        g.push(In::Key(K_TAB, false));
        g.push(In::Key(K_TAB, false));
        g.push(In::Char('r'));
        frames(&mut g, 140);
    }
    if scene == "battle" {
        g.world.end_calm();
        let mut t = 0;
        for _ in 0..1500 {
            hold::bot::elder(&mut g.world, t);
            t += 1;
            g.frame();
        }
        // Select a nest with a reward on it to show the panel.
        let n = g
            .world
            .bldgs
            .iter()
            .find(|b| b.alive && b.kind == BKind::Nest)
            .map(|b| b.id);
        if let Some(n) = n {
            g.sel = hold::game::Sel::Bldg(n);
        }
        frames(&mut g, 1);
    }
    if scene == "end" || scene == "feedback" {
        g.skip_lesson();
        let mut t = 0;
        while g.world.over.is_none() && t < 15 * 60 * 10 {
            hold::bot::elder(&mut g.world, t);
            t += 1;
            g.frame();
        }
        frames(&mut g, 3);
        if scene == "feedback" {
            g.push(In::Char('n'));
            for c in "the herons are too quick".chars() {
                g.push(In::Char(c));
            }
            frames(&mut g, 2);
        }
    }
    if scene == "glade" || scene == "hero" {
        g.skip_lesson();
        let mut t = 0;
        for _ in 0..1500 {
            hold::bot::elder(&mut g.world, t);
            t += 1;
            g.frame();
        }
        if scene == "hero" {
            let h = g.world.units.iter().find(|u| u.kind.hero()).map(|u| u.id);
            if let Some(h) = h {
                g.sel = hold::game::Sel::Unit(h);
            }
            g.frame();
        }
    }
    g.scanlines = false;
    let n = 3;
    let px = g.present(n).to_vec();
    let (w, h) = (400 * n, 300 * n);
    let mut ppm = format!("P6\n{w} {h}\n255\n").into_bytes();
    for p in px.chunks(4) {
        ppm.extend_from_slice(&p[..3]);
    }
    std::fs::write(&out, ppm).unwrap();
}
