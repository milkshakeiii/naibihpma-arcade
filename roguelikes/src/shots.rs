//! Headless 2x screenshots of every screen, for checking the look without playing.

use crate::app::{App, Ev, Out, Run};
use crate::changeling;
use crate::form::Form;
use crate::framebuffer::Framebuffer;
use crate::palette;
use crate::turncoat;
use std::path::Path;

fn keep_x(i: usize) -> i32 {
    turncoat::keep_rect(i).x + 10
}

pub fn export(dir: &Path) {
    let _ = std::fs::create_dir_all(dir);
    let mut fb = Framebuffer::new();
    let save = |fb: &Framebuffer, pal: &[u32; 64], name: &str| fb.save_png(&dir.join(name), pal);

    let app = App::new(None, None);
    app.draw(&mut fb);
    save(&fb, &palette::shell(), "00_title.png");

    // Changeling.
    let mut out = Out::default();
    let mut c = changeling::Game::new(11, true);
    for _ in 0..30 {
        c.tick(&mut out);
    }
    c.event(Ev::Move(changeling::BX + 3 * changeling::TILE + 10, changeling::BY + 6 * changeling::TILE + 10), &mut out);
    c.draw(&mut fb);
    save(&fb, c.palette(), "a01_lesson_first_steps.png");

    let mut c = changeling::Game::new(11, true);
    for l in 0..2 {
        // Walk lessons 1 and 2 by teleporting to the end state.
        let _ = l;
    }
    c.lesson = Some(2);
    c.floor = changeling::gen::lesson_floor(2, &c.carry);
    c.card = 0;
    c.refresh();
    let heron = c.floor.creatures.iter().find(|k| k.kind == changeling::rules::Kind::Heron).unwrap().pos;
    c.event(Ev::Move(changeling::BX + heron.0 * changeling::TILE + 10, changeling::BY + heron.1 * changeling::TILE + 10), &mut out);
    c.draw(&mut fb);
    save(&fb, c.palette(), "a02_lesson_danger_heron.png");
    let pl = c.floor.player;
    c.event(Ev::Move(changeling::BX + (pl.0 - 1) * changeling::TILE + 10, changeling::BY + (pl.1 - 1) * changeling::TILE + 10), &mut out);
    c.draw(&mut fb);
    save(&fb, c.palette(), "a03_lesson_danger_preview.png");

    for (b, name) in [(2u32, "a04_underbrush.png"), (5, "a05_dewfields.png"), (8, "a06_star_pond.png"), (10, "a07_grof.png")] {
        let mut c = changeling::Game::new(99 + b as u64, false);
        c.carry.dew = 9;
        c.carry.form = if b > 4 { changeling::rules::Kind::Frog } else { changeling::rules::Kind::Wanderer };
        c.carry.charms = vec![changeling::rules::Charm::Pounce, changeling::rules::Charm::Shell];
        let mut o = Out::default();
        for _ in 0..b {
            let _ = &mut o;
        }
        c.burrow = b - 1;
        c.floor = changeling::gen::generate(b, &mut c.rng, &c.carry);
        c.burrow = b;
        c.card = 0;
        c.refresh();
        let target = c.previews.first().map(|p| p.0).unwrap_or(c.floor.player);
        c.event(Ev::Move(changeling::BX + target.0 * changeling::TILE + 10, changeling::BY + target.1 * changeling::TILE + 10), &mut out);
        c.draw(&mut fb);
        save(&fb, c.palette(), name);
    }

    let mut c = changeling::Game::new(5, false);
    c.carry.dew = 12;
    c.carry.hearts = 2;
    c.burrow = 3;
    c.phase = changeling::Phase::Glade { offers: vec![changeling::rules::Charm::Pounce, changeling::rules::Charm::TwoSkins, changeling::rules::Charm::Startle], first: false };
    c.event(Ev::Move(100, changeling::GLADE_ITEM_Y + 40), &mut out);
    c.draw(&mut fb);
    save(&fb, c.palette(), "a08_glade.png");
    c.phase = changeling::Phase::Over { won: false };
    c.phase = changeling::Phase::Burrow;
    c.card = 0;
    c.refresh();
    c.phase = changeling::Phase::Over { won: false };
    c.draw(&mut fb);
    save(&fb, c.palette(), "a09_over.png");
    c.phase = changeling::Phase::Burrow;
    c.show_help = true;
    c.draw(&mut fb);
    save(&fb, c.palette(), "a10_help.png");

    // Turncoat.
    let mut t = turncoat::Game::new(21, true);
    t.tick(&mut out);
    t.event(Ev::Move(60, turncoat::BENCH_Y + 10), &mut out);
    t.draw(&mut fb);
    save(&fb, t.palette(), "b01_lesson_deploy.png");
    // Drag the first two onto the field, then point at a rival.
    for (slot, sq) in [(0usize, (2, 3)), (0, (4, 3)), (0, (3, 4)), (0, (3, 5))] {
        let r = turncoat::Game::bench_rect(slot);
        t.event(Ev::Press { x: r.x + 10, y: r.y + 10, right: false }, &mut out);
        let (x, y) = (turncoat::FX + sq.0 * turncoat::TILE + 10, turncoat::FY + sq.1 * turncoat::TILE + 10);
        t.event(Ev::Move(x, y), &mut out);
        t.event(Ev::Release { x, y, right: false }, &mut out);
    }
    t.event(Ev::Move(turncoat::FX + 3 * turncoat::TILE + 10, turncoat::FY + turncoat::TILE + 10), &mut out);
    t.draw(&mut fb);
    save(&fb, t.palette(), "b02_lesson_reach.png");
    t.event(Ev::Press { x: turncoat::START_BTN.x + 5, y: turncoat::START_BTN.y + 5, right: false }, &mut out);
    for _ in 0..200 {
        t.tick(&mut out);
        if t.note.is_some() {
            break;
        }
    }
    t.draw(&mut fb);
    save(&fb, t.palette(), "b03_lesson_note.png");
    t.note = None;
    for _ in 0..400 {
        t.tick(&mut out);
        if t.note.is_some() || matches!(t.playing.as_ref().map(|p| &p.act), Some(turncoat::battle::Act::Speak { .. })) {
            break;
        }
    }
    t.draw(&mut fb);
    save(&fb, t.palette(), "b04_battle.png");

    let mut t = turncoat::Game::new(8, false);
    t.event(Ev::Move(100, 120), &mut out);
    t.draw(&mut fb);
    save(&fb, t.palette(), "b05_route.png");

    // A mid-run deploy in the Golden Age, pointing at a rival.
    let mut t = turncoat::Game::new(13, false);
    t.index = 7;
    for k in [turncoat::data::Kind::Smith, turncoat::data::Kind::Rider, turncoat::data::Kind::Herald] {
        t.band.push(turncoat::Member { uid: 900 + k as u32, kind: k, career: 0, veteran: k == turncoat::data::Kind::Rider });
    }
    t.leaders_beaten = 2;
    t.ideas = vec![turncoat::data::Idea::Bronze, turncoat::data::Idea::Roads];
    let mut o = Out::default();
    t.encounters.clear();
    t.phase = turncoat::Phase::Route { options: vec![0, 1] };
    let e = turncoat::Encounter {
        rivals: vec![
            (turncoat::data::Kind::Scholar, (1, 1), false),
            (turncoat::data::Kind::Priest, (3, 2), false),
            (turncoat::data::Kind::Merchant, (4, 2), true),
            (turncoat::data::Kind::Herald, (5, 0), false),
            (turncoat::data::Kind::Smith, (2, 2), false),
        ],
        leader: false,
    };
    t.encounters.push(e.clone());
    t.encounters.push(e);
    t.event(Ev::Press { x: 30, y: 100, right: false }, &mut o);
    t.placed = vec![(0, (1, 3)), (4, (2, 3)), (5, (4, 4)), (6, (3, 5)), (3, (5, 5))];
    t.event(Ev::Move(turncoat::FX + 3 * turncoat::TILE + 10, turncoat::FY + 2 * turncoat::TILE + 10), &mut o);
    t.draw(&mut fb);
    save(&fb, t.palette(), "b06_golden_deploy.png");
    t.event(Ev::Key(crate::app::Key::Enter), &mut o);
    for _ in 0..90 {
        t.tick(&mut o);
        t.note = None;
    }
    t.event(Ev::Move(turncoat::FX + 3 * turncoat::TILE + 10, turncoat::FY + 3 * turncoat::TILE + 10), &mut o);
    t.draw(&mut fb);
    save(&fb, t.palette(), "b07_golden_battle.png");
    t.speed = 4;
    for _ in 0..4000 {
        if !matches!(t.phase, turncoat::Phase::Battle) {
            break;
        }
        t.tick(&mut o);
    }
    t.draw(&mut fb);
    save(&fb, t.palette(), "b08_after.png");
    for _ in 0..12 {
        t.tick(&mut o);
    }
    t.event(Ev::Press { x: 10, y: 10, right: false }, &mut o);
    t.event(Ev::Move(keep_x(1), 90), &mut o);
    t.draw(&mut fb);
    save(&fb, t.palette(), "b09_keep.png");
    t.phase = turncoat::Phase::Idea { offers: vec![turncoat::data::Idea::Writing, turncoat::data::Idea::Medicine, turncoat::data::Idea::Banners] };
    t.draw(&mut fb);
    save(&fb, t.palette(), "b10_idea.png");
    t.show_help = true;
    t.draw(&mut fb);
    save(&fb, t.palette(), "b11_help.png");

    let mut f = Form::new("CHANGELING - RUN 1 OVER. HOW WAS IT?".into(), "CAUGHT OUT IN THE DEWFIELDS, BURROW 5 OF 10. 23 CATCHES, STRUCK 4 TIMES, 82 TURNS. CHARMS: POUNCE.".into(), &changeling::FORM_QUESTION);
    let mut sfx = Vec::new();
    for ch in "4352".chars() {
        f.event(Ev::Char(ch), &mut sfx);
    }
    f.event(Ev::Press { x: 20, y: 160, right: false }, &mut sfx);
    for ch in "When the heron kept dodging me until I became a frog.".chars() {
        f.event(Ev::Char(ch), &mut sfx);
    }
    f.draw(&mut fb);
    save(&fb, &palette::shell(), "c01_form.png");
    println!("exported screenshots to {}", dir.display());
}
