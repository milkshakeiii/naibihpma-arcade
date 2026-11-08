//! HATCHERY: design familiars gene by gene, raise them, and watch them defend the eggs.

pub mod art;
pub mod battle;
pub mod data;
mod render;

use crate::app::{Ev, Key, Out, Run, Summary};
use crate::audio::{Sfx, Track};
use crate::form::Question;
use crate::framebuffer::Framebuffer;
use crate::palette::Pal;
use crate::rng::Rng;
use crate::ui::Rect;
use battle::*;
use data::*;

pub const VERSION: u32 = 1;
pub const LESSON_VERSION: u32 = 1;
pub static FORM_QUESTION: Question = Question { id: "familiars", text: "DESIGNING YOUR FAMILIARS WAS...", low: "BUSYWORK", high: "THE BEST PART" };

pub const MAX_ROSTER: usize = 5;
pub const START_EGGS: i32 = 7;
pub const START_GENES: [Gene; 5] = [Gene::LongTongue, Gene::Warts, Gene::Slime, Gene::LeapingLegs, Gene::SplitTail];

// Hatchery layout.
pub fn card_rect(i: usize) -> Rect {
    Rect::new(4 + (i % 3) as i32 * 132, 18 + (i / 3) as i32 * 98, 128, 94)
}
pub fn grow_rect(i: usize) -> Rect {
    let c = card_rect(i);
    Rect::new(c.x + c.w - 44, c.y + 4, 40, 14)
}
pub const CLUTCH: Rect = Rect::new(268, 116, 128, 94);
pub const HATCH_BTN: Rect = Rect::new(274, 160, 116, 20);
pub const NIGHT_BTN: Rect = Rect::new(274, 184, 116, 22);
pub const INFO: Rect = Rect::new(4, 216, 260, 82);
pub const FOES_BOX: Rect = Rect::new(268, 216, 128, 82);
pub fn option_rect(i: usize) -> Rect {
    Rect::new(40, 64 + i as i32 * 34, 320, 30)
}

// Night layout.
pub const AT: i32 = 28;
pub const AX: i32 = 58;
pub const AY: i32 = 22;
pub fn bench_rect(i: usize) -> Rect {
    Rect::new(AX + i as i32 * 34, 198, 32, 30)
}
pub const START_BTN: Rect = Rect::new(274, 200, 118, 26);
pub const SPEED_BTN: Rect = Rect::new(274, 200, 56, 26);
pub const SKIP_BTN: Rect = Rect::new(336, 200, 56, 26);

#[derive(Clone, Debug, PartialEq)]
pub enum Dialog {
    Body,
    Gene { body: Body },
    Grow { fam: usize },
    Absorb { genes: Vec<Gene> },
}

#[derive(Clone, Debug, PartialEq)]
pub enum Phase {
    Hatchery,
    Deploy,
    Night,
    Dawn { t: u32 },
    Over { won: bool },
}

#[derive(Default, Clone)]
pub struct Stats {
    pub hatched: u32,
    pub fallen: Vec<String>,
    pub defeated: u32,
    pub eggs_eaten: i32,
    pub levels: u32,
}

pub const LESSON_STEPS: [&str; 6] = [
    "EACH EGG IN YOUR CLUTCH IS A LIFE: GROF'S CREATURES COME AT NIGHT TO EAT THEM. AN EGG CAN ALSO HATCH INTO A FAMILIAR. PRESS HATCH.",
    "PICK A BODY. EACH BODY GROWS GENES IN ITS OWN ORDER OF PLACES (HEAD, SKIN, LEGS, TAIL).",
    "NOW ITS FIRST GENE. GENES ARE HOW YOU DESIGN A FAMILIAR. IT GROWS ANOTHER AT EACH LEVEL, AND YOU CHOOSE IT.",
    "GOOD. HATCH ONE MORE (EACH HATCHING COSTS AN EGG).",
    "WHEN YOU'RE READY, PRESS NIGHT FALLS.",
    "DRAG YOUR FAMILIARS ONTO THE LIT SQUARES NEAR THE NEST, THEN PRESS START. THEY FIGHT ON THEIR OWN.",
];

pub struct Game {
    pub phase: Phase,
    pub dialog: Option<Dialog>,
    pub roster: Vec<Familiar>,
    pub library: Vec<Gene>,
    pub eggs: i32,
    pub night: u32,
    pub posts: Vec<(usize, Pos)>,
    pub battle: Option<Battle>,
    pub speed: u32,
    pub drag: Option<usize>,
    pub mouse: (i32, i32),
    pub hover_item: Option<usize>,
    pub lesson: bool,
    pub lesson_step: usize,
    pub note: Option<String>,
    pub tips: Vec<&'static str>,
    pub message: Option<String>,
    pub rng: Rng,
    pub art: art::Art,
    pub pal: Pal,
    pub stats: Stats,
    pub ended: bool,
    pub show_help: bool,
    pub frame: u32,
    pub think: u32,
    pub names_used: Vec<&'static str>,
    pub last_night: Option<(i32, Vec<Foe>)>,
}

impl Game {
    pub fn new(seed: u64, lesson: bool) -> Self {
        let mut g = Self {
            phase: Phase::Hatchery,
            dialog: None,
            roster: Vec::new(),
            library: START_GENES.to_vec(),
            eggs: START_EGGS,
            night: 1,
            posts: Vec::new(),
            battle: None,
            speed: 1,
            drag: None,
            mouse: (-1, -1),
            hover_item: None,
            lesson,
            lesson_step: 0,
            note: None,
            tips: Vec::new(),
            message: None,
            rng: Rng::new(seed),
            art: art::Art::new(),
            pal: art::palette(),
            stats: Stats::default(),
            ended: false,
            show_help: false,
            frame: 0,
            think: 0,
            names_used: Vec::new(),
            last_night: None,
        };
        if !lesson {
            // Without the lesson, start with two hatched familiars to design further.
            let (a, b) = (g.name(), g.name());
            g.roster.push(Familiar::new(a, Body::Frog, Gene::LongTongue));
            g.roster.push(Familiar::new(b, Body::Toad, Gene::Warts));
            g.eggs -= 2;
            g.message = Some("HATCH, GROW AND PLACE YOUR FAMILIARS, THEN LET NIGHT FALL. POINT AT ANYTHING FOR DETAILS.".into());
        }
        g
    }

    fn name(&mut self) -> &'static str {
        let free: Vec<&'static str> = NAMES.iter().copied().filter(|n| !self.names_used.contains(n)).collect();
        let n = if free.is_empty() { "SPAWN" } else { *self.rng.pick(&free) };
        self.names_used.push(n);
        n
    }

    pub fn track(&self) -> Track {
        match self.phase {
            Phase::Over { won: true } => Track::StarPond,
            Phase::Night | Phase::Deploy if self.night == NIGHTS => Track::Grof,
            Phase::Night => Track::Twen,
            _ => Track::Doat,
        }
    }

    fn tip(&mut self, id: &'static str, text: &str) {
        if !self.tips.contains(&id) {
            self.tips.push(id);
            self.message = Some(text.to_string());
        }
    }

    fn lesson_to(&mut self, step: usize, out: &mut Out) {
        if self.lesson && self.lesson_step < step {
            self.lesson_step = step;
            out.sfx(Sfx::Lesson);
        }
    }

    pub fn can_hatch(&self) -> bool {
        self.eggs > 1 && self.roster.len() < MAX_ROSTER
    }

    /// Genes in the library that fit a location.
    pub fn genes_for(&self, loc: Loc) -> Vec<Gene> {
        self.library.iter().copied().filter(|g| g.loc() == loc).collect()
    }

    /// The options shown by the open dialog.
    pub fn options(&self) -> Vec<String> {
        match &self.dialog {
            Some(Dialog::Body) => ALL_BODIES.iter().map(|b| b.info().name.to_string()).collect(),
            Some(Dialog::Gene { body }) => self.genes_for(body.info().locs[0]).iter().map(|g| g.name().to_string()).collect(),
            Some(Dialog::Grow { fam }) => self.roster[*fam].next_loc().map_or(Vec::new(), |l| self.genes_for(l).iter().map(|g| g.name().to_string()).collect()),
            Some(Dialog::Absorb { genes }) => genes.iter().map(|g| g.name().to_string()).collect(),
            None => Vec::new(),
        }
    }

    fn pick_option(&mut self, i: usize, out: &mut Out) {
        let Some(d) = self.dialog.clone() else { return };
        match d {
            Dialog::Body => {
                let body = ALL_BODIES[i];
                self.dialog = Some(Dialog::Gene { body });
                out.sfx(Sfx::Click);
                self.lesson_to(2, out);
            }
            Dialog::Gene { body } => {
                let genes = self.genes_for(body.info().locs[0]);
                let Some(&g) = genes.get(i) else { return };
                let name = self.name();
                self.roster.push(Familiar::new(name, body, g));
                self.eggs -= 1;
                self.stats.hatched += 1;
                self.dialog = None;
                out.sfx(Sfx::Hatch);
                out.log(format!("hatched {} the {} with {} ({} eggs left)", name.to_lowercase(), body.info().name.to_lowercase(), g.name().to_lowercase(), self.eggs));
                if self.lesson {
                    if self.roster.len() >= 2 {
                        self.lesson_to(4, out);
                    } else {
                        self.lesson_to(3, out);
                    }
                }
            }
            Dialog::Grow { fam } => {
                let Some(loc) = self.roster[fam].next_loc() else { return };
                let genes = self.genes_for(loc);
                let Some(&g) = genes.get(i) else { return };
                let f = &mut self.roster[fam];
                f.genes.push(g);
                f.level += 1;
                self.stats.levels += 1;
                out.sfx(Sfx::Level);
                out.log(format!("{} grows into a {} (level {}): {} on its {}", f.name.to_lowercase(), f.stage().to_lowercase(), f.level, g.name().to_lowercase(), loc.name().to_lowercase()));
                self.dialog = None;
            }
            Dialog::Absorb { genes } => {
                let Some(&g) = genes.get(i) else { return };
                self.library.push(g);
                out.sfx(Sfx::Gene);
                out.log(format!("absorbed {}", g.name().to_lowercase()));
                self.dialog = None;
            }
        }
    }

    pub fn arena_square(&self, x: i32, y: i32) -> Option<Pos> {
        let (gx, gy) = ((x - AX).div_euclid(AT), (y - AY).div_euclid(AT));
        (x >= AX && y >= AY && inb((gx, gy))).then_some((gx, gy))
    }

    pub fn unplaced(&self) -> Vec<usize> {
        (0..self.roster.len()).filter(|i| !self.posts.iter().any(|(p, _)| p == i)).collect()
    }

    fn to_deploy(&mut self, out: &mut Out) {
        if self.roster.is_empty() {
            out.sfx(Sfx::Deny);
            self.message = Some("HATCH AT LEAST ONE FAMILIAR FIRST.".into());
            return;
        }
        if self.roster.iter().any(|f| f.level_due() && !self.genes_for(f.next_loc().unwrap()).is_empty()) {
            self.tip("grow_first", "SOMEONE IS READY TO GROW: CLICK GROW! ON THEIR CARD FIRST (OR PRESS NIGHT FALLS AGAIN TO SKIP).");
            if self.message.as_deref().is_some_and(|m| m.starts_with("SOMEONE IS READY")) && !self.tips.contains(&"grow_warned") {
                self.tips.push("grow_warned");
                out.sfx(Sfx::Deny);
                return;
            }
        }
        // Keep last night's posts where they still make sense; place the rest in a column.
        self.posts.retain(|(i, _)| *i < self.roster.len());
        let squares = deploy_squares();
        for i in self.unplaced() {
            let free = [(3, 2), (3, 3), (2, 1), (2, 4), (3, 0), (3, 5)].into_iter().find(|p| !self.posts.iter().any(|(_, q)| q == p) && squares.contains(p));
            if let Some(p) = free {
                if !self.lesson {
                    self.posts.push((i, p));
                }
            }
        }
        self.phase = Phase::Deploy;
        self.message = None;
        out.music = Some(self.track());
        out.sfx(Sfx::Click);
        self.lesson_to(5, out);
    }

    fn start_night(&mut self, out: &mut Out) {
        if self.posts.is_empty() {
            out.sfx(Sfx::Deny);
            self.message = Some("DRAG AT LEAST ONE FAMILIAR ONTO THE POND FIRST.".into());
            return;
        }
        let seed = self.rng.next_u64();
        let b = Battle::new(&self.roster, &self.posts, self.night, self.eggs, seed);
        let fams: Vec<String> = self
            .posts
            .iter()
            .map(|(i, p)| {
                let f = &self.roster[*i];
                format!("{}({} lv{} [{}])@{}{}", f.name.to_lowercase(), f.body.info().name.to_lowercase(), f.level, f.genes.iter().map(|g| g.name().to_lowercase()).collect::<Vec<_>>().join("+"), p.0, p.1)
            })
            .collect();
        out.log(format!(
            "--- night {} ({:.0}s in the hatchery): eggs {}, posted [{}], foes [{}]",
            self.night,
            self.think as f32 / 15.0,
            self.eggs,
            fams.join(", "),
            b.schedule.iter().map(|s| s.1.info().name.to_lowercase()).collect::<Vec<_>>().join(", ")
        ));
        self.think = 0;
        self.battle = Some(b);
        self.phase = Phase::Night;
        self.message = None;
        self.drag = None;
        out.music = Some(self.track());
        out.sfx(Sfx::Fight);
    }

    fn night_tick(&mut self, out: &mut Out) {
        if self.note.is_some() {
            return;
        }
        let n = match self.speed {
            1 => 1,
            2 => 2,
            _ => 4,
        };
        for _ in 0..n {
            let Some(b) = self.battle.as_mut() else { return };
            b.step();
            let t = b.tick;
            let mut word = false;
            let mut egg = false;
            let mut fell = false;
            let mut bite = false;
            for (ft, fx) in &b.fx {
                if *ft != t {
                    continue;
                }
                match fx {
                    Fx::Word { .. } | Fx::Chorus { .. } | Fx::Fire { .. } => word = true,
                    Fx::Egg { .. } => egg = true,
                    Fx::Died { .. } => fell = true,
                    Fx::Bite { .. } => bite = true,
                    _ => {}
                }
            }
            if egg {
                out.sfx(Sfx::Hurt);
            } else if fell {
                out.sfx(Sfx::Fall);
            } else if word {
                out.sfx(Sfx::Gene);
            } else if bite {
                out.sfx(Sfx::Tap);
            }
            if self.lesson {
                if word && !self.tips.contains(&"note_word") {
                    self.tips.push("note_word");
                    self.note = Some("WHEN A GENE DOES SOMETHING, ITS NAME FLOATS UP. THE WAY YOU DESIGN EACH FAMILIAR SHAPES HOW IT FIGHTS.".into());
                    return;
                }
                if egg && !self.tips.contains(&"note_egg") {
                    self.tips.push("note_egg");
                    self.note = Some("AN EGG WAS CARRIED OFF: ONE LIFE GONE. FOES HEAD FOR THE NEST AND FIGHT WHATEVER STANDS NEXT TO THEM.".into());
                    return;
                }
            }
            if b.over {
                self.end_night(out);
                return;
            }
        }
    }

    fn end_night(&mut self, out: &mut Out) {
        let Some(b) = self.battle.take() else { return };
        self.eggs = b.eggs;
        self.stats.eggs_eaten += b.eggs_eaten;
        self.stats.defeated += b.defeated.len() as u32;
        // Experience; the fallen are gone for good.
        let mut fallen = Vec::new();
        let mut alive = vec![false; self.roster.len()];
        for u in &b.units {
            if let Some(i) = u.fam {
                if u.alive {
                    alive[i] = true;
                    self.roster[i].xp += 2 + u.kills;
                }
            }
        }
        let mut keep = Vec::new();
        for (i, f) in self.roster.iter().enumerate() {
            let posted = self.posts.iter().any(|(p, _)| *p == i);
            if posted && !alive[i] {
                fallen.push(f.name.to_string());
            } else {
                keep.push(i);
            }
        }
        let remap: Vec<(usize, usize)> = keep.iter().enumerate().map(|(new, &old)| (old, new)).collect();
        self.roster = keep.iter().map(|&i| self.roster[i].clone()).collect();
        self.posts = self.posts.iter().filter_map(|(i, p)| remap.iter().find(|(o, _)| o == i).map(|(_, n)| (*n, *p))).collect();
        self.stats.fallen.extend(fallen.iter().cloned());
        out.log(format!(
            "night {} over at tick {}: eggs eaten {}, eggs left {}, foes defeated {}, fallen [{}]",
            self.night,
            b.tick,
            b.eggs_eaten,
            self.eggs,
            b.defeated.len(),
            fallen.join(", ").to_lowercase()
        ));
        self.last_night = Some((b.eggs_eaten, b.defeated.clone()));
        if self.eggs <= 0 {
            out.sfx(Sfx::Lose);
            self.phase = Phase::Over { won: false };
            return;
        }
        if self.night >= NIGHTS {
            out.sfx(Sfx::Win);
            out.log("WON: the hatchery survived grof's night");
            self.phase = Phase::Over { won: true };
            out.music = Some(Track::StarPond);
            return;
        }
        out.sfx(Sfx::Clear);
        self.phase = Phase::Dawn { t: 0 };
        if !fallen.is_empty() {
            self.tip("fallen", "A FAMILIAR THAT FALLS AT NIGHT IS GONE FOR GOOD.");
        }
    }

    fn dawn_done(&mut self, out: &mut Out) {
        // A new egg is laid each dawn.
        self.eggs += 1;
        self.night += 1;
        self.phase = Phase::Hatchery;
        out.music = Some(Track::Doat);
        // Learn a gene from what was defeated.
        let defeated = self.last_night.as_ref().map(|l| l.1.clone()).unwrap_or_default();
        let mut new: Vec<Gene> = Vec::new();
        for f in defeated {
            let g = f.info().gene;
            if !self.library.contains(&g) && !new.contains(&g) {
                new.push(g);
            }
        }
        self.rng.shuffle(&mut new);
        new.truncate(3);
        if !new.is_empty() {
            self.dialog = Some(Dialog::Absorb { genes: new });
            self.tip("absorb", "YOU CAN LEARN ONE GENE FROM THE CREATURES YOU DEFEATED. PICK ONE FOR YOUR LIBRARY.");
        }
        if self.roster.iter().any(|f| f.level_due()) {
            self.tip("grow", "A FAMILIAR HAS GROWN ENOUGH TO LEVEL UP: CLICK GROW! ON ITS CARD AND CHOOSE ITS NEW GENE.");
        }
        self.lesson = false;
    }
}

impl Run for Game {
    fn event(&mut self, ev: Ev, out: &mut Out) {
        if let Ev::Move(x, y) | Ev::Press { x, y, .. } | Ev::Release { x, y, .. } = ev {
            self.mouse = (x, y);
        }
        if self.show_help {
            if matches!(ev, Ev::Key(_) | Ev::Press { .. }) {
                self.show_help = false;
                out.sfx(Sfx::Click);
            }
            return;
        }
        if ev == Ev::Key(Key::Help) {
            self.show_help = true;
            out.sfx(Sfx::Click);
            return;
        }
        if self.dialog.is_some() {
            let n = self.options().len();
            match ev {
                Ev::Move(x, y) => self.hover_item = (0..n).find(|&i| option_rect(i).contains(x, y)),
                Ev::Press { x, y, right: false } => {
                    if let Some(i) = (0..n).find(|&i| option_rect(i).contains(x, y)) {
                        self.pick_option(i, out);
                    } else if !matches!(self.dialog, Some(Dialog::Absorb { .. })) && !option_rect(n).contains(x, y) {
                        // Clicking outside closes a hatch or grow dialog.
                        if !self.lesson {
                            self.dialog = None;
                            out.sfx(Sfx::Click);
                        }
                    }
                }
                Ev::Key(Key::Esc) if !self.lesson => self.dialog = None,
                _ => {}
            }
            return;
        }
        match self.phase.clone() {
            Phase::Hatchery => match ev {
                Ev::Press { x, y, right: false } => {
                    if HATCH_BTN.contains(x, y) {
                        if self.can_hatch() {
                            self.dialog = Some(Dialog::Body);
                            out.sfx(Sfx::Click);
                            self.lesson_to(1, out);
                        } else {
                            out.sfx(Sfx::Deny);
                            self.message = Some(if self.roster.len() >= MAX_ROSTER {
                                "YOUR HATCHERY HOLDS FIVE FAMILIARS.".into()
                            } else {
                                "YOU CAN'T HATCH YOUR LAST EGG.".into()
                            });
                        }
                    } else if NIGHT_BTN.contains(x, y) {
                        if self.lesson && self.lesson_step < 4 {
                            out.sfx(Sfx::Deny);
                        } else {
                            self.to_deploy(out);
                        }
                    } else if let Some(i) = (0..self.roster.len()).find(|&i| grow_rect(i).contains(x, y) && self.roster[i].level_due()) {
                        if self.roster[i].next_loc().is_some_and(|l| !self.genes_for(l).is_empty()) {
                            self.dialog = Some(Dialog::Grow { fam: i });
                            out.sfx(Sfx::Click);
                        } else {
                            out.sfx(Sfx::Deny);
                            self.message = Some("YOUR LIBRARY HAS NO GENE FOR THAT PLACE YET. ABSORB ONE FROM A DEFEATED FOE.".into());
                        }
                    }
                }
                Ev::Move(x, y) => self.hover_item = (0..self.roster.len()).find(|&i| card_rect(i).contains(x, y)),
                _ => {}
            },
            Phase::Deploy => match ev {
                Ev::Press { x, y, right } => {
                    if START_BTN.contains(x, y) && !right {
                        self.start_night(out);
                        return;
                    }
                    if let Some(p) = self.arena_square(x, y) {
                        if let Some(k) = self.posts.iter().position(|(_, q)| *q == p) {
                            if right {
                                self.posts.remove(k);
                                out.sfx(Sfx::Click);
                            } else {
                                self.drag = Some(self.posts[k].0);
                                out.sfx(Sfx::Click);
                            }
                        } else if let Some(i) = self.drag.take() {
                            self.place(i, p, out);
                        }
                        return;
                    }
                    for (slot, i) in self.unplaced().into_iter().enumerate() {
                        if bench_rect(slot).contains(x, y) {
                            self.drag = Some(i);
                            out.sfx(Sfx::Click);
                            return;
                        }
                    }
                }
                Ev::Release { x, y, right: false } => {
                    if let Some(i) = self.drag {
                        let posted = self.posts.iter().find(|(j, _)| *j == i).map(|(_, q)| *q);
                        match self.arena_square(x, y) {
                            // Released where it already stands: a click, keep holding.
                            Some(p) if posted == Some(p) => {}
                            Some(p) => {
                                self.drag = None;
                                self.place(i, p, out);
                            }
                            // Back onto the bench: a posted familiar is taken off the pond; a
                            // familiar picked up from the bench stays in hand.
                            None if (196..230).contains(&y) => {
                                if posted.is_some() {
                                    self.posts.retain(|(j, _)| *j != i);
                                    self.drag = None;
                                }
                            }
                            None => self.drag = None,
                        }
                    }
                }
                Ev::Key(Key::Enter) => self.start_night(out),
                _ => {}
            },
            Phase::Night => match ev {
                Ev::Press { x, y, right: false } => {
                    if self.note.is_some() {
                        self.note = None;
                        out.sfx(Sfx::Click);
                    } else if SPEED_BTN.contains(x, y) {
                        self.speed = match self.speed {
                            1 => 2,
                            2 => 4,
                            _ => 1,
                        };
                        out.sfx(Sfx::Click);
                    } else if SKIP_BTN.contains(x, y) && !self.lesson {
                        out.sfx(Sfx::Click);
                        if let Some(b) = self.battle.as_mut() {
                            b.run();
                        }
                        out.log("night skipped to the end");
                        self.end_night(out);
                    }
                }
                Ev::Key(Key::Enter) | Ev::Key(Key::Space) if self.note.is_some() => {
                    self.note = None;
                    out.sfx(Sfx::Click);
                }
                Ev::Key(Key::Fast) => {
                    self.speed = match self.speed {
                        1 => 2,
                        2 => 4,
                        _ => 1,
                    };
                }
                _ => {}
            },
            Phase::Dawn { t } => {
                if t > 8 && matches!(ev, Ev::Press { right: false, .. } | Ev::Key(Key::Enter) | Ev::Key(Key::Space)) {
                    out.sfx(Sfx::Click);
                    self.dawn_done(out);
                }
            }
            Phase::Over { .. } => {
                if matches!(ev, Ev::Press { right: false, .. } | Ev::Key(Key::Enter)) {
                    out.sfx(Sfx::Click);
                    self.ended = true;
                }
            }
        }
    }

    fn tick(&mut self, out: &mut Out) {
        self.frame += 1;
        if self.frame == 1 {
            out.music = Some(self.track());
        }
        match &mut self.phase {
            Phase::Hatchery | Phase::Deploy => self.think += 1,
            Phase::Dawn { t } => *t += 1,
            Phase::Night => self.night_tick(out),
            _ => {}
        }
    }

    fn draw(&self, fb: &mut Framebuffer) {
        render::draw(self, fb);
    }

    fn palette(&self) -> &Pal {
        &self.pal
    }

    fn ended(&self) -> bool {
        self.ended
    }

    fn summary(&self) -> Summary {
        let headline = match self.phase {
            Phase::Over { won: true } => "THE HATCHERY SURVIVED GROF'S NIGHT.".to_string(),
            Phase::Over { won: false } => format!("THE LAST EGG WAS TAKEN ON NIGHT {} OF {NIGHTS}.", self.night),
            _ => format!("STOPPED BEFORE NIGHT {} OF {NIGHTS}.", self.night),
        };
        let s = &self.stats;
        let fam: Vec<String> = self.roster.iter().map(|f| format!("{} THE {} LV{} ({})", f.name, f.body.info().name, f.level, f.genes.iter().map(|g| g.name()).collect::<Vec<_>>().join(", "))).collect();
        let detail = format!(
            "HATCHED {}, LOST {} TO THE NIGHT, {} EGGS EATEN, {} FOES DEFEATED, {} LEVELS GROWN. FAMILIARS: {}.",
            s.hatched,
            s.fallen.len(),
            s.eggs_eaten,
            s.defeated,
            s.levels,
            if fam.is_empty() { "NONE".to_string() } else { fam.join("; ") }
        );
        let won = matches!(self.phase, Phase::Over { won: true });
        Summary { headline, detail, best_key: self.night + if won { 100 } else { 0 } }
    }

    fn takes_esc(&self) -> bool {
        self.show_help || (self.dialog.is_some() && !self.lesson)
    }
}

impl Game {
    fn place(&mut self, i: usize, p: Pos, out: &mut Out) {
        if !deploy_squares().contains(&p) {
            out.sfx(Sfx::Deny);
            self.message = Some("FAMILIARS START ON THE LIT SQUARES NEAR THE NEST.".into());
            return;
        }
        // Swap with whoever stands there.
        let old = self.posts.iter().find(|(j, _)| *j == i).map(|(_, q)| *q);
        if let Some(k) = self.posts.iter().position(|(_, q)| *q == p) {
            match old {
                Some(o) => self.posts[k].1 = o,
                None => {
                    self.posts.remove(k);
                }
            }
        }
        self.posts.retain(|(j, _)| *j != i);
        self.posts.push((i, p));
        out.sfx(Sfx::Hop);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn drive(g: &mut Game, out: &mut Out) {
        for _ in 0..20_000 {
            if g.ended {
                return;
            }
            if let Some(d) = g.dialog.clone() {
                let _ = d;
                g.pick_option(0, out);
                continue;
            }
            match g.phase.clone() {
                Phase::Hatchery => {
                    for i in 0..g.roster.len() {
                        if g.roster[i].level_due() && g.roster[i].next_loc().is_some_and(|l| !g.genes_for(l).is_empty()) {
                            g.dialog = Some(Dialog::Grow { fam: i });
                            g.pick_option(0, out);
                        }
                    }
                    if g.can_hatch() && (g.roster.len() < 3 || g.eggs > 4) {
                        g.dialog = Some(Dialog::Body);
                        let b = g.rng.below(5) as usize;
                        g.pick_option(b, out);
                        g.pick_option(0, out);
                        continue;
                    }
                    g.lesson_step = 4;
                    g.tips.push("grow_warned");
                    g.to_deploy(out);
                    if g.phase == Phase::Hatchery {
                        g.to_deploy(out);
                    }
                }
                Phase::Deploy => {
                    for i in g.unplaced() {
                        let p = (3, (i as i32 + 1) % H);
                        g.place(i, p, out);
                    }
                    g.start_night(out);
                }
                Phase::Night => {
                    g.note = None;
                    g.speed = 4;
                    g.tick(out);
                }
                Phase::Dawn { .. } => g.dawn_done(out),
                Phase::Over { .. } => g.ended = true,
            }
        }
    }

    #[test]
    fn whole_runs_reach_an_end() {
        let mut res = Vec::new();
        for seed in 0..12 {
            let mut g = Game::new(seed, seed == 0);
            let mut out = Out::default();
            drive(&mut g, &mut out);
            assert!(g.ended, "seed {seed} stuck in {:?} {:?}", g.phase, g.dialog);
            res.push((g.night, matches!(g.phase, Phase::Over { won: true }), g.stats.hatched, g.stats.fallen.len(), g.stats.eggs_eaten));
        }
        println!("hatchery bot (night, won, hatched, fallen, eggs eaten): {res:?}");
    }
}
