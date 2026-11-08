//! Headless 2x screenshots of every screen, for checking the look without playing.

use crate::app::{App, Ev, Out, Run};
use crate::changeling;
use crate::form::Form;
use crate::framebuffer::Framebuffer;
use crate::palette;
use std::path::Path;

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
        c.carry.charms = vec![changeling::rules::Charm::Shove, changeling::rules::Charm::Shell];
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
    c.phase = changeling::Phase::Glade { offers: vec![changeling::rules::Charm::DewSense, changeling::rules::Charm::TwoSkins, changeling::rules::Charm::Shove], first: false };
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

    // Conduit.
    use crate::conduit::{self, data::*, From};
    let mut c = conduit::Game::new(21, true);
    c.tick(&mut out);
    c.event(Ev::Move(conduit::shelf_rect(0).x + 20, conduit::shelf_rect(0).y + 4), &mut out);
    c.draw(&mut fb);
    save(&fb, c.palette(), "b01_lesson_workshop.png");
    // Pick up the club and hover the orc's held slot.
    let club = c.shelf.iter().position(|p| *p == Part::Club).unwrap();
    let r = conduit::shelf_rect(club);
    c.event(Ev::Press { x: r.x + 20, y: r.y + 4, right: false }, &mut out);
    c.event(Ev::Release { x: r.x + 20, y: r.y + 4, right: false }, &mut out);
    let s = conduit::slot_rect(0, 0);
    c.event(Ev::Move(s.x + 30, s.y + 4), &mut out);
    c.draw(&mut fb);
    save(&fb, c.palette(), "b02_lesson_holding.png");
    c.event(Ev::Press { x: s.x + 30, y: s.y + 4, right: false }, &mut out);
    let sling = c.shelf.iter().position(|p| *p == Part::Sling).unwrap();
    c.held = Some((Part::Sling, From::Shelf(sling)));
    c.event(Ev::Press { x: conduit::slot_rect(1, 0).x + 30, y: conduit::slot_rect(1, 0).y + 4, right: false }, &mut out);
    let rr = conduit::rival_rect(1);
    c.event(Ev::Move(rr.x + 10, rr.y + 10), &mut out);
    c.draw(&mut fb);
    save(&fb, c.palette(), "b03_lesson_rival.png");
    c.event(Ev::Press { x: conduit::FIGHT_BTN.x + 5, y: conduit::FIGHT_BTN.y + 5, right: false }, &mut out);
    for _ in 0..300 {
        c.tick(&mut out);
        if c.note.is_some() {
            break;
        }
    }
    c.draw(&mut fb);
    save(&fb, c.palette(), "b04_lesson_note.png");

    // A mid-run workshop at the junction against a themed crew.
    let mut c = conduit::Game::new(13, false);
    c.contest = 4;
    let mut o = Out::default();
    c.rival = c.make_rival();
    c.crew = vec![Build::new(Body::Orc), Build::new(Body::Elf), Build::new(Body::Cyclops)];
    c.crew[0].parts = vec![Some(Part::Axe), Some(Part::Shield), Some(Part::Leather), Some(Part::Fang)];
    c.crew[1].parts = vec![Some(Part::SparkWand), Some(Part::WardCharm), Some(Part::HunterMind), None];
    c.crew[2].parts = vec![Some(Part::Club), Some(Part::Plate), None, Some(Part::SecondHeart)];
    c.shelf = vec![Part::Bow, Part::Boots, Part::Cloak, Part::MendingStaff, Part::Spikes, Part::QuickNerves];
    let m = conduit::card_rect(1);
    c.event(Ev::Move(m.x + 60, m.y + 10), &mut o);
    c.draw(&mut fb);
    save(&fb, c.palette(), "b05_junction_workshop.png");
    c.event(Ev::Key(crate::app::Key::Enter), &mut o);
    for _ in 0..70 {
        c.tick(&mut o);
    }
    c.draw(&mut fb);
    save(&fb, c.palette(), "b06_junction_battle.png");
    c.speed = 4;
    for _ in 0..2000 {
        if !matches!(c.phase, conduit::Phase::Battle) {
            break;
        }
        c.tick(&mut o);
    }
    c.draw(&mut fb);
    save(&fb, c.palette(), "b07_after.png");
    c.phase = conduit::Phase::Reward { offers: vec![conduit::Reward::Part(Part::Torch), conduit::Reward::Monster(Body::Troll), conduit::Reward::Crate(vec![Part::Bow, Part::StoneGut])] };
    c.event(Ev::Move(100, conduit::reward_rect(1).y + 10), &mut o);
    c.draw(&mut fb);
    save(&fb, c.palette(), "b08_spoils.png");
    c.phase = conduit::Phase::Over { won: false };
    c.draw(&mut fb);
    save(&fb, c.palette(), "b09_over.png");
    c.phase = conduit::Phase::Workshop;
    c.show_help = true;
    c.draw(&mut fb);
    save(&fb, c.palette(), "b10_help.png");

    let mut f = Form::new("CHANGELING - RUN 1 OVER. HOW WAS IT?".into(), "FELL IN THE DEWFIELDS, BURROW 5 OF 10. 23 CATCHES (LONGEST CHAIN 3), STRUCK 4 TIMES, 82 TURNS. CHARMS: SHELL.".into(), &changeling::FORM_QUESTION);
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
