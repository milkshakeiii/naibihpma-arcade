//! Equipment given for renown, and the abilities it adds to (DESIGN.md: Equipment given for
//! renown; Abilities that differ and grow). A monster's pieces are its own for good: nothing to
//! wear out or restock. One of a kind at a time (a new one replaces it), only as many as its type
//! has slots for, within its body's and mind's capacity, and the best bonus to each ability counts.

use crate::data::{Data, Piece, Slot, Stats};
use crate::econ::Knack;
use crate::game::{Game, Heard, Held, Monster, Sfx};
use crate::world::PKind;

/// Renown a monster has brought in to be seasoned: +1 to every ability at the first, +2 at the
/// second.
pub const SEASONED_AT: [f64; 2] = [1000.0, 5000.0];

/// How seasoned a monster that has brought in `earned` renown is: what it adds to every ability.
pub fn seasoning(earned: f64) -> i32 {
    SEASONED_AT.iter().filter(|&&t| earned >= t).count() as i32
}

/// A number from the world's dice for `seed` that leaves the world's own stream alone, so rolling
/// a newcomer's abilities changes nothing else that happens.
pub fn dice(seed: u64) -> u64 {
    let mut z = seed.wrapping_add(0x9E37_79B9_7F4A_7C15);
    z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
    z ^ (z >> 31)
}

/// A newcomer's abilities: its type's, each a roll from -2 to +2 off, 0 most often (1, 2, 3, 2
/// and 1 chances in 9).
pub fn rolled(base: &Stats, seed: u64) -> Stats {
    let mut s = base.clone();
    for a in 0..6 {
        let z = dice(seed.wrapping_mul(7).wrapping_add(a as u64));
        let d = (z % 3) as i32 - 1 + ((z / 3) % 3) as i32 - 1;
        s.set(a, base.get(a) + d);
    }
    s
}

impl Monster {
    /// Its abilities as they count: its own, plus how seasoned it is, plus the best bonus to each
    /// from what it has on (bonuses to the same ability don't stack).
    pub fn abilities(&self, data: &Data) -> Stats {
        let mut s = self.stats.clone();
        let seasoned = self.seasoned();
        for a in 0..6 {
            s.set(a, s.get(a) + seasoned + self.best_bonus(data, a, None));
        }
        s
    }

    /// What being seasoned adds to each of its abilities. A working townsperson stays as the
    /// town made them, whatever their orders bring in.
    pub fn seasoned(&self) -> i32 {
        if self.worker.is_some() { 0 } else { seasoning(self.earned) }
    }

    /// What its pieces do to a craft whose main skill is `skill` (or to practice at it): a
    /// craft's secret counts only at its own.
    pub fn knack(&self, data: &Data, skill: &str) -> Knack {
        self.helping(data, skill).iter().fold(crate::econ::PLAIN, |k, p| k.with(&p.does))
    }

    /// Its pieces that do something to a craft whose main skill is `skill`, for the forecast
    /// card ("MAGNIFYING GLASS: +10% QUALITY"). A teaching of a talent it has already does
    /// nothing.
    pub fn helping<'a>(&self, data: &'a Data, skill: &str) -> Vec<&'a Piece> {
        self.pieces()
            .filter_map(|id| data.try_piece(id))
            .filter(|p| p.does.at.as_deref().is_none_or(|at| at == skill))
            .filter(|p| !p.does.text().is_empty() && p.does.talent.as_ref().is_none_or(|t| !self.skills.transferable.contains(t)))
            .collect()
    }

    /// The best bonus to ability `a` from what it has on, leaving out the piece of kind
    /// `without` (one being replaced).
    pub(crate) fn best_bonus(&self, data: &Data, a: usize, without: Option<&str>) -> i32 {
        self.pieces().filter_map(|id| data.try_piece(id)).filter(|p| Some(p.kind.as_str()) != without).map(|p| p.bonus_to(a)).max().unwrap_or(0).max(0)
    }
}

/// A line of the character sheet you can choose: a hand (with a tool or a carried piece in it),
/// or a slot with what's in it.
#[derive(Clone, Debug, PartialEq)]
pub enum SheetLine {
    Hand(usize),
    Gear(Slot, Option<String>),
}

impl Game {
    /// What monster `i` has of its body's capacity used: the tools in its hands and what it has on.
    pub(crate) fn body_used(&self, i: usize) -> f64 {
        self.held_weight(i) + self.monsters[i].pieces().filter_map(|id| self.data.try_piece(id)).map(|p| p.body as f64).sum::<f64>()
    }

    /// What monster `i` has of its mind's capacity used.
    pub(crate) fn mind_used(&self, i: usize) -> u32 {
        self.monsters[i].pieces().filter_map(|id| self.data.try_piece(id)).map(|p| p.mind).sum()
    }

    /// Monster `i`'s pieces in `slot`, in the order it was given them.
    pub(crate) fn gear_in(&self, i: usize, slot: Slot) -> Vec<&Piece> {
        self.monsters[i].pieces().filter_map(|id| self.data.try_piece(id)).filter(|p| p.slot == slot).collect()
    }

    /// The character sheet's lines for monster `i`: its hands, then each slot it has to wear
    /// things and have them inside, with what's in it.
    pub(crate) fn sheet_lines(&self, i: usize) -> Vec<SheetLine> {
        let slots = self.data.monster_types[self.monsters[i].kind.as_str()].slots;
        let mut lines: Vec<SheetLine> = (0..self.monsters[i].hands.len()).map(SheetLine::Hand).collect();
        for slot in [Slot::Worn, Slot::Inside] {
            let on: Vec<String> = self.gear_in(i, slot).iter().map(|p| p.id.clone()).collect();
            for n in 0..slots.of(slot) as usize {
                lines.push(SheetLine::Gear(slot, on.get(n).cloned()));
            }
        }
        lines
    }

    /// Is a rack of equipment here?
    pub fn rack_at(&self, x: i32, y: i32) -> Option<usize> {
        self.world.racks.iter().position(|r| (r.x, r.y) == (x, y))
    }

    /// The rack monster `c` faces, if any.
    pub fn faced_rack(&self, c: usize) -> Option<usize> {
        let m = &self.monsters[c];
        let (dx, dy) = m.facing.delta();
        self.rack_at(m.x + dx, m.y + dy)
    }

    /// How many racks stand beside monster `c`.
    pub(crate) fn racks_beside(&self, c: usize) -> usize {
        let m = &self.monsters[c];
        self.racks_beside_tile((m.x, m.y))
    }

    /// How many racks stand beside a tile.
    pub(crate) fn racks_beside_tile(&self, (x, y): (i32, i32)) -> usize {
        crate::world::DIRS.iter().filter(|d| self.rack_at(x + d.delta().0, y + d.delta().1).is_some()).count()
    }

    /// The piece of the same kind monster `i` has on, which `piece` would replace.
    pub(crate) fn replaces(&self, i: usize, piece: &Piece) -> Option<&Piece> {
        self.monsters[i].pieces().filter_map(|id| self.data.try_piece(id)).find(|p| p.kind == piece.kind && p.slot == piece.slot)
    }

    /// Why monster `i` couldn't have `piece` (renown aside), in its own words, if it couldn't.
    pub(crate) fn piece_refusal(&self, i: usize, piece: &Piece) -> Option<String> {
        let m = &self.monsters[i];
        let ty = &self.data.monster_types[m.kind.as_str()];
        if m.pieces().any(|p| p == piece.id) {
            return Some(format!("I HAVE THE {} ALREADY.", piece.name.to_uppercase()));
        }
        // A teaching counts as a talent: not one it was born with.
        if let Some(t) = piece.does.talent.as_ref().filter(|t| m.skills.transferable.contains(t)) {
            return Some(format!("I HAVE A TALENT FOR {} ALREADY.", t.replace('_', " ").to_uppercase()));
        }
        let old = self.replaces(i, piece);
        let room = ty.slots.of(piece.slot) as usize;
        if room == 0 {
            return Some(format!("A {} HAS NO {} SLOTS.", m.kind.to_uppercase(), piece.slot.label()));
        }
        // Held, it takes a free hand, as a tool does.
        if piece.slot == Slot::Hand && old.is_none() && m.hands.iter().all(Option::is_some) {
            return Some("MY HANDS ARE FULL. PUT SOMETHING DOWN ON MY SHEET (C).".into());
        }
        if piece.slot != Slot::Hand && old.is_none() && self.gear_in(i, piece.slot).len() >= room {
            let slots = if room == 1 { format!("MY {} SLOT IS", piece.slot.label()) } else { format!("ALL {room} OF MY {} SLOTS ARE", piece.slot.label()) };
            return Some(format!("{slots} FULL. TAKE SOMETHING OFF ON MY SHEET (C)."));
        }
        let (old_body, old_mind) = old.map_or((0.0, 0), |p| (p.body as f64, p.mind));
        if self.body_used(i) - old_body + piece.body as f64 > ty.body_cap as f64 {
            return Some("THAT'S MORE THAN MY BODY CAN TAKE.".into());
        }
        if self.mind_used(i) - old_mind + piece.mind > ty.mind_cap {
            return Some("THAT'S MORE THAN MY MIND CAN TAKE.".into());
        }
        None
    }

    /// What monster `i` has on already that gives more of an ability `piece` adds to: the best
    /// counts, so `piece` would add nothing there. The other piece's name and the ability.
    pub(crate) fn bettered(&self, i: usize, piece: &Piece) -> Option<(String, &'static str)> {
        let m = &self.monsters[i];
        let old = self.replaces(i, piece).map(|p| p.kind.clone());
        (0..6).filter(|&a| piece.bonus_to(a) > 0).find_map(|a| {
            let best = m.best_bonus(&self.data, a, old.as_deref());
            (best >= piece.bonus_to(a)).then(|| {
                let by = m.pieces().filter_map(|id| self.data.try_piece(id)).filter(|p| Some(&p.kind) != old.as_ref()).max_by_key(|p| p.bonus_to(a)).map_or(String::new(), |p| p.name.to_uppercase());
                (by, crate::data::ABILITIES[a])
            })
        })
    }

    /// Monster `c` is given the piece on rack `r`, if its commune's renown reaches it and it has
    /// room. One of the same kind it has on gives way to it.
    pub(crate) fn give_piece(&mut self, c: usize, r: usize, sfx: &mut Vec<Sfx>) {
        if self.monsters[c].rec.is_some() {
            self.say(c, "FINISH RECORDING FIRST (R).", true);
            sfx.push(Sfx::Deny);
            return;
        }
        let piece = self.data.piece(self.world.racks[r].piece).clone();
        if let Some(why) = self.piece_refusal(c, &piece) {
            self.say(c, &why, true);
            sfx.push(Sfx::Deny);
            return;
        }
        let k = self.monsters[c].commune;
        if self.communes[k].renown < piece.renown as f64 {
            self.say(c, &format!("NOT ENOUGH RENOWN YET ({}).", piece.renown), true);
            sfx.push(Sfx::Deny);
            return;
        }
        self.communes[k].renown -= piece.renown as f64;
        let old = self.replaces(c, &piece).map(|p| p.id.clone());
        let m = &mut self.monsters[c];
        if piece.slot == Slot::Hand {
            // Into the hand that held the one it replaces, or a free one.
            let h = m.hands.iter().position(|h| matches!((h, &old), (Some(Held::Piece(p)), Some(o)) if p == o)).or_else(|| m.hands.iter().position(Option::is_none)).expect("a hand for it");
            m.hands[h] = Some(Held::Piece(piece.id.clone()));
        } else {
            if let Some(old) = &old {
                m.gear.retain(|id| id != old);
            }
            m.gear.push(piece.id.clone());
        }
        let verb = piece.verb();
        let text = match old {
            Some(old) => format!("{} {verb} THE {} IN PLACE OF THE {}. {}!", m.name, piece.name.to_uppercase(), self.data.piece(&old).name.to_uppercase(), piece.effect_text()),
            None => format!("{} {verb} THE {}. {}!", m.name, piece.name.to_uppercase(), piece.effect_text()),
        };
        let (x, y) = (m.x, m.y);
        self.say(c, &text, false);
        self.world.burst(x, y, PKind::Spark, 6);
        sfx.push(Sfx::RecSave);
    }

    /// Monster `i` takes off the piece `id` (or puts it down, if it's held), from its sheet. It's
    /// gone for good.
    pub(crate) fn take_off(&mut self, i: usize, id: &str, sfx: &mut Vec<Sfx>) {
        let m = &mut self.monsters[i];
        if !m.pieces().any(|g| g == id) {
            return;
        }
        m.gear.retain(|g| g != id);
        for h in m.hands.iter_mut().filter(|h| matches!(h, Some(Held::Piece(p)) if p == id)) {
            *h = None;
        }
        let p = self.data.piece(id);
        let text = format!("{} {} THE {}.", m.name, p.off_verb(), p.name.to_uppercase());
        self.say(i, &text, false);
        sfx.push(Sfx::Plop);
    }

    /// What the outfitter says to whoever faces them.
    pub(crate) fn outfitter_says(&self, c: usize) -> Vec<String> {
        let k = self.monsters[c].commune;
        let renown = self.communes[k].renown.floor();
        vec![format!("THINGS TO WEAR AND TO WORK WITH, FOR MONSTERS OF SOME RENOWN. YOURS HAS {renown}. FACE A RACK TO SEE WHAT ITS PIECE DOES; SPACE, AND IT'S YOURS FOR GOOD.")]
    }

    /// What the temple keeper says to whoever faces them.
    pub(crate) fn keeper_says(&self, c: usize) -> Vec<String> {
        let k = self.monsters[c].commune;
        let renown = self.communes[k].renown.floor();
        vec![format!("THE TEMPLE'S TEACHINGS, FOR MONSTERS OF SOME RENOWN. YOURS HAS {renown}. A TEACHING IS A TALENT YOU WEREN'T BORN WITH. FACE A SCROLL TO SEE WHAT IT TEACHES; SPACE, AND IT'S LEARNED FOR GOOD.")]
    }

    /// Credit to monster `i` that may season it: once its renown brought in passes a step, every
    /// ability goes up, and its player hears of it.
    pub(crate) fn season(&mut self, i: usize, before: f64) {
        let (was, now) = (seasoning(before), seasoning(self.monsters[i].earned));
        if now <= was || self.monsters[i].worker.is_some() {
            return;
        }
        let m = &self.monsters[i];
        let (x, y, k) = (m.x, m.y, m.commune);
        let text = if now == 1 { format!("{} IS SEASONED: +1 TO EVERY ABILITY.", m.name) } else { format!("{} IS SEASONED AGAIN: +{now} TO EVERY ABILITY.", m.name) };
        self.say(i, &text, false);
        self.world.burst(x, y, PKind::Spark, 10);
        self.sounds.push((Heard::By(k), Sfx::Chime));
    }
}
