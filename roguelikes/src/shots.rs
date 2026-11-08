//! Headless 2x screenshots of every screen, for checking the look without playing.

use crate::app::{App, Ev, Out, Run};
use crate::form::Form;
use crate::framebuffer::Framebuffer;
use crate::gulp;
use crate::hatchery;
use crate::palette;
use std::path::Path;

pub fn export(dir: &Path) {
    let _ = std::fs::create_dir_all(dir);
    let mut fb = Framebuffer::new();
    let save = |fb: &Framebuffer, pal: &[u32; 64], name: &str| fb.save_png(&dir.join(name), pal);

    let app = App::new(None, None);
    app.draw(&mut fb);
    save(&fb, &palette::shell(), "00_title.png");

    // GULP.
    let mut out = Out::default();
    let at = |p: (i32, i32)| (gulp::BX + p.0 * gulp::TILE + 12, gulp::BY + p.1 * gulp::TILE + 12);
    let mut g = gulp::Game::new(5, true);
    for _ in 0..25 {
        g.tick(&mut out);
    }
    let (x, y) = at((2, 6));
    g.event(Ev::Move(x, y), &mut out);
    g.draw(&mut fb);
    save(&fb, g.palette(), "a01_lesson_hop.png");

    // Lesson 2 in tongue mode, pointing up a line at a fly.
    let mut g = gulp::Game::new(5, true);
    g.lesson = Some(1);
    g.pond = gulp::gen::lesson_pond(1, &g.frog);
    g.pond.exit_locked = true;
    g.card = 0;
    g.event(Ev::Char('t'), &mut out);
    let (x, y) = at((3, 4));
    g.event(Ev::Move(x, y), &mut out);
    g.draw(&mut fb);
    save(&fb, g.palette(), "a02_lesson_tongue.png");

    // Lesson 3: strikes and the egg clutch.
    let mut g = gulp::Game::new(5, true);
    g.lesson = Some(2);
    g.frog.size = 2;
    g.pond = gulp::gen::lesson_pond(2, &g.frog);
    g.pond.exit_locked = true;
    g.card = 0;
    let (x, y) = at((3, 6));
    g.event(Ev::Move(x, y), &mut out);
    g.draw(&mut fb);
    save(&fb, g.palette(), "a03_lesson_strikes.png");

    for (n, name, size) in [(2u32, "a04_underbrush.png", 1), (5, "a05_star_pond.png", 2), (8, "a06_deep_pool.png", 3), (10, "a07_grof.png", 3)] {
        let mut g = gulp::Game::new(30 + n as u64, false);
        g.frog.size = size;
        g.frog.max_hearts = 2 + size;
        g.frog.hearts = 2 + size;
        g.frog.muts = vec![gulp::rules::Mutation::PoisonSkin, gulp::rules::Mutation::Gills];
        g.frog.throat = vec![gulp::rules::Kind::Beetle];
        g.pond_no = n;
        g.pond = gulp::gen::generate(n, &mut g.rng, &g.frog);
        g.card = 0;
        let f = g.pond.frog.pos;
        let (x, y) = at((f.0, (f.1 - 2).max(0)));
        g.event(Ev::Move(x, y), &mut out);
        g.draw(&mut fb);
        save(&fb, g.palette(), name);
    }

    let mut g = gulp::Game::new(9, false);
    g.pond_no = 3;
    g.frog.hearts = 2;
    g.phase = gulp::Phase::Bank {
        offers: vec![Some(gulp::rules::Mutation::LongTongue), Some(gulp::rules::Mutation::BigThroat), None],
    };
    let r = gulp::mutation_rect(1);
    g.event(Ev::Move(r.x + 10, r.y + 10), &mut out);
    g.draw(&mut fb);
    save(&fb, g.palette(), "a08_bank.png");
    g.phase = gulp::Phase::Over { won: false };
    g.draw(&mut fb);
    save(&fb, g.palette(), "a09_over.png");
    g.phase = gulp::Phase::Pond;
    g.show_help = true;
    g.draw(&mut fb);
    save(&fb, g.palette(), "a10_help.png");

    export_hatchery(dir, &mut fb);

    let mut f = Form::new("GULP - RUN 1 OVER. HOW WAS IT?".into(), "THE FROG FELL IN STAR POND, POND 5 OF 10. A MEDIUM FROG. ATE 12, DEFEATED 9, HURT 6 TIMES OVER 61 TURNS.".into(), &gulp::FORM_QUESTION);
    let mut sfx = Vec::new();
    for ch in "4352".chars() {
        f.event(Ev::Char(ch), &mut sfx);
    }
    f.draw(&mut fb);
    save(&fb, &palette::shell(), "c01_form.png");
    let _ = hatchery::VERSION;
    println!("exported screenshots to {}", dir.display());
}

fn export_hatchery(dir: &Path, fb: &mut Framebuffer) {
    use hatchery::data::*;
    let save = |fb: &Framebuffer, pal: &[u32; 64], name: &str| fb.save_png(&dir.join(name), pal);
    let mut out = Out::default();
    let mut h = hatchery::Game::new(3, true);
    h.tick(&mut out);
    h.draw(fb);
    save(fb, h.palette(), "b01_lesson_hatchery.png");
    h.event(Ev::Press { x: hatchery::HATCH_BTN.x + 5, y: hatchery::HATCH_BTN.y + 5, right: false }, &mut out);
    let o = hatchery::option_rect(0);
    h.event(Ev::Move(o.x + 10, o.y + 10), &mut out);
    h.draw(fb);
    save(fb, h.palette(), "b02_lesson_bodies.png");
    h.event(Ev::Press { x: o.x + 10, y: o.y + 10, right: false }, &mut out);
    h.draw(fb);
    save(fb, h.palette(), "b03_lesson_genes.png");

    // A mid-run hatchery with grown familiars.
    let mut h = hatchery::Game::new(4, false);
    h.night = 5;
    h.eggs = 5;
    h.library = ALL_GENES.to_vec();
    h.roster = vec![
        Familiar { name: "PIP", body: Body::Frog, genes: vec![Gene::LongTongue, Gene::LeapingLegs, Gene::GlowSpots], xp: 9, level: 3 },
        Familiar { name: "BOG", body: Body::Toad, genes: vec![Gene::Warts, Gene::WideMouth], xp: 7, level: 2 },
        Familiar { name: "DEW", body: Body::Newt, genes: vec![Gene::SplitTail, Gene::VenomFangs, Gene::Slime, Gene::WebbedFeet], xp: 14, level: 4 },
        Familiar { name: "EMBER", body: Body::Salamander, genes: vec![Gene::FireGland], xp: 3, level: 1 },
        Familiar { name: "WISP", body: Body::Axolotl, genes: vec![Gene::EggSac, Gene::BrightColors], xp: 4, level: 2 },
    ];
    h.hover_item = Some(0);
    h.mouse = (60, 60);
    h.draw(fb);
    save(fb, h.palette(), "b04_hatchery.png");
    h.hover_item = None;
    h.dialog = Some(hatchery::Dialog::Grow { fam: 3 });
    h.draw(fb);
    save(fb, h.palette(), "b05_grow.png");
    h.dialog = None;
    h.event(Ev::Press { x: hatchery::NIGHT_BTN.x + 5, y: hatchery::NIGHT_BTN.y + 5, right: false }, &mut out);
    h.event(Ev::Press { x: hatchery::NIGHT_BTN.x + 5, y: hatchery::NIGHT_BTN.y + 5, right: false }, &mut out);
    h.draw(fb);
    save(fb, h.palette(), "b06_deploy.png");
    h.event(Ev::Key(crate::app::Key::Enter), &mut out);
    for _ in 0..70 {
        h.tick(&mut out);
    }
    h.draw(fb);
    save(fb, h.palette(), "b07_night.png");
    h.speed = 4;
    for _ in 0..3000 {
        if !matches!(h.phase, hatchery::Phase::Night) {
            break;
        }
        h.tick(&mut out);
    }
    h.draw(fb);
    save(fb, h.palette(), "b08_dawn.png");
    h.phase = hatchery::Phase::Over { won: false };
    h.draw(fb);
    save(fb, h.palette(), "b09_over.png");
}
