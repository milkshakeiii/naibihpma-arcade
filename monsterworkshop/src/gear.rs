//! Equipment given for renown, and the abilities it adds to (DESIGN.md: Equipment given for
//! renown; Abilities that differ and grow). A monster's pieces are its own for good: nothing to
//! wear out or restock. Only as many as its type has slots for, a free one for each (none takes
//! another's place), within its body's and mind's capacity, and the best bonus to each ability
//! counts.

use crate::data::{Data, Piece, Slot, Stats};
use crate::econ::Knack;
use crate::game::{Game, Heard, Held, Monster, Sfx};
use crate::world::PKind;

/// Why a monster can't give up its tool belt: what it holds would have nowhere to go.
pub(crate) const TOO_FULL: &str = "MY HANDS ARE TOO FULL FOR THAT. PUT SOMETHING DOWN ON MY SHEET (C).";

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
            s.set(a, s.get(a) + seasoned + self.best_bonus(data, a));
        }
        s
    }

    /// What being seasoned adds to each of its abilities. A working townsperson stays as the
    /// town made them, whatever their orders bring in.
    pub fn seasoned(&self) -> i32 {
        if self.worker.is_some() { 0 } else { seasoning(self.earned) }
    }

    /// How many hands it has: its type's, and one more for a tool belt.
    pub fn hand_count(&self, data: &Data) -> usize {
        data.monster_types[self.kind.as_str()].slots.hands as usize + self.gear.iter().filter_map(|id| data.try_piece(id)).map(|p| p.does.hands as usize).sum::<usize>()
    }

    /// How many of its hands hold something.
    pub fn holding(&self) -> usize {
        self.hands.iter().flatten().count()
    }

    /// What its type's ability and its pieces do to a craft whose main skill is `skill` (or to
    /// practice at it, or to hauling): a craft's secret counts only at its own.
    pub fn knack(&self, data: &Data, skill: &str) -> Knack {
        let base = self.ability(data).map_or(crate::econ::PLAIN, |a| crate::econ::PLAIN.with(&a.does));
        self.helping(data, skill).iter().fold(base, |k, p| k.with(&p.does))
    }

    /// Its type's unique ability, if it has one.
    pub fn ability<'a>(&self, data: &'a Data) -> Option<&'a crate::data::TypeAbility> {
        data.monster_types.get(self.kind.as_str())?.ability.as_ref()
    }

    /// The ability it leads a wagon by: CHA, or an orc's STR.
    pub fn leads_by(&self, data: &Data) -> usize {
        self.knack(data, "wagon_driving").lead_by.unwrap_or(crate::data::CHA)
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

    /// The best bonus to ability `a` from what it has on.
    pub(crate) fn best_bonus(&self, data: &Data, a: usize) -> i32 {
        self.pieces().filter_map(|id| data.try_piece(id)).map(|p| p.bonus_to(a)).max().unwrap_or(0).max(0)
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
        let room = ty.slots.of(piece.slot) as usize;
        if room == 0 {
            return Some(format!("A {} HAS NO {} SLOTS.", m.kind.to_uppercase(), piece.slot.label()));
        }
        // Held, it takes a free hand, as a tool does.
        if piece.slot == Slot::Hand && m.hands.iter().all(Option::is_some) {
            return Some("MY HANDS ARE FULL. PUT SOMETHING DOWN ON MY SHEET (C).".into());
        }
        if piece.slot != Slot::Hand && self.gear_in(i, piece.slot).len() >= room {
            let slots = if room == 1 { format!("MY {} SLOT IS", piece.slot.label()) } else { format!("ALL {room} OF MY {} SLOTS ARE", piece.slot.label()) };
            return Some(format!("{slots} FULL. TAKE SOMETHING OFF ON MY SHEET (C)."));
        }
        None
    }

    /// What monster `i` has on already that gives more of an ability `piece` adds to: the best
    /// counts, so `piece` would add nothing there. The other piece's name and the ability.
    pub(crate) fn bettered(&self, i: usize, piece: &Piece) -> Option<(String, &'static str)> {
        let m = &self.monsters[i];
        (0..6).filter(|&a| piece.bonus_to(a) > 0).find_map(|a| {
            let best = m.best_bonus(&self.data, a);
            (best >= piece.bonus_to(a)).then(|| {
                let by = m.pieces().filter_map(|id| self.data.try_piece(id)).max_by_key(|p| p.bonus_to(a)).map_or(String::new(), |p| p.name.to_uppercase());
                (by, crate::data::ABILITIES[a])
            })
        })
    }

    /// Monster `c` is given the piece on rack `r`, if its commune's renown reaches it and it has
    /// room.
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
        let m = &mut self.monsters[c];
        if piece.slot == Slot::Hand {
            let h = m.hands.iter().position(Option::is_none).expect("a free hand for it");
            m.hands[h] = Some(Held::Piece(piece.id.clone()));
        } else {
            m.gear.push(piece.id.clone());
        }
        self.fit_hands(c);
        let m = &self.monsters[c];
        let text = format!("{} {} THE {}. {}!", m.name, piece.verb(), piece.name.to_uppercase(), piece.effect_text());
        let (x, y) = (m.x, m.y);
        self.say(c, &text, false);
        self.world.burst(x, y, PKind::Spark, 6);
        sfx.push(Sfx::RecSave);
    }

    /// Monster `i` takes off the piece `id` (or puts it down, if it's held), from its sheet. It's
    /// gone for good.
    pub(crate) fn take_off(&mut self, i: usize, id: &str, sfx: &mut Vec<Sfx>) {
        let m = &self.monsters[i];
        if !m.pieces().any(|g| g == id) {
            return;
        }
        // A tool belt comes off only if what it holds has a free hand to go to.
        let extra = self.data.piece(id).does.hands as usize;
        if extra > 0 && m.holding() > m.hand_count(&self.data) - extra {
            self.say(i, TOO_FULL, true);
            sfx.push(Sfx::Deny);
            return;
        }
        let m = &mut self.monsters[i];
        m.gear.retain(|g| g != id);
        for h in m.hands.iter_mut().filter(|h| matches!(h, Some(Held::Piece(p)) if p == id)) {
            *h = None;
        }
        self.fit_hands(i);
        let m = &self.monsters[i];
        let p = self.data.piece(id);
        let text = format!("{} {} THE {}.", m.name, p.off_verb(), p.name.to_uppercase());
        self.say(i, &text, false);
        sfx.push(Sfx::Plop);
    }

    /// Monster `i`'s hands, as many as it has now: a tool belt put on adds one; taken off, what
    /// was in it goes to a free hand. Never fewer hands than it's holding things (a belt that
    /// can't come off isn't let off).
    pub(crate) fn fit_hands(&mut self, i: usize) {
        let n = self.monsters[i].hand_count(&self.data);
        let m = &mut self.monsters[i];
        if m.hands.len() > n && m.holding() <= n {
            let held: Vec<Held> = m.hands.drain(..).flatten().collect();
            m.hands = held.into_iter().map(Some).collect();
        }
        if m.hands.len() < n {
            m.hands.resize(n, None);
        }
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
