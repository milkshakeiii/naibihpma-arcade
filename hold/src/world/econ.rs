//! The economy (documents 01 and 02): spots ripen and are picked, folk carry things to stock
//! levels, makers craft, sites gather their materials and are dug up, and buildings hatch.

use super::*;

impl World {
    // ---- spots

    pub(crate) fn grow_spots(&mut self) {
        let t = self.t;
        let dt = day_tick(t);
        let rain = self.weather.rain;
        let dry_spell = self.weather.dry;
        for si in 0..self.spots.len() {
            let owner = self.spots[si].owner;
            let Some(bi) = self.bi(owner) else { continue };
            if !self.bs[bi].done() {
                continue;
            }
            let s = &self.spots[si];
            let pos = s.pos;
            let info = spot(s.kind);
            let open = !self.map.damp(pos.0, pos.1) && !self.raining_at(pos);
            let cursed = self.map.cursed(pos.0, pos.1);
            let night = is_night(t);
            let s = &mut self.spots[si];
            if s.state == 3 && t >= s.gone_until {
                s.state = 1;
                s.progress = 0;
            }
            if s.state == 1 && !cursed {
                // A spot ripens only while its ground is damp (New), and puffballs only at night.
                let grows = match info.when {
                    When::Night => night,
                    When::Damp => rain || !open || night,
                } && !(dry_spell && open && s.kind == SpotKind::Fern);
                if grows {
                    s.progress += if s.tender.is_some() { 15 * TEND_PCT / 100 } else { 15 };
                    if s.progress >= info.ripen {
                        s.state = 2;
                        s.ripe_since = t;
                    }
                }
            }
            if s.state == 2 && s.claim.is_none() {
                // Dew dries at noon on open ground; puffballs burst at dawn.
                if s.kind == SpotKind::Fern && open && dt / 15 == NOON / 15 {
                    s.state = 1;
                    s.progress = 0;
                }
                if s.kind == SpotKind::Puff && dt < 15 {
                    s.state = 1;
                    s.progress = 0;
                }
            }
        }
    }

    pub fn raining_at(&self, p: (i32, i32)) -> bool {
        self.weather.rain || self.rain_at.iter().any(|r| r.1 > self.t && dist(r.0, p) <= spell(Spell::RainCall).radius)
    }

    // ---- stock levels

    pub fn want(&self, b: &Building, i: Item) -> i32 {
        if b.kind == BKind::Hall && i == Item::Dew {
            return 0;
        }
        b.keep(i) + b.reserved(i) - b.has(i) - b.coming(i)
    }

    pub fn spare(&self, b: &Building, i: Item) -> i32 {
        if !b.done() {
            return 0;
        }
        if b.kind == BKind::Hall && i == Item::Dew {
            return b.has(i) - b.reserved(i);
        }
        b.has(i) - b.keep(i) - b.reserved(i)
    }

    /// How many of an item the Hold wants delivered, less what's lying ready to carry.
    pub fn total_want(&self, i: Item) -> i32 {
        let w: i32 = self.bs.iter().filter(|b| b.team == Team::Hold && !b.dead && (b.done() || !b.res.is_empty())).map(|b| self.want(b, i).max(0)).sum();
        let lying: i32 = self.ground.iter().filter(|g| !g.gone && g.item == i && g.claim.is_none()).map(|g| g.n).sum();
        w - lying
    }

    /// The nearest source for a building's want: on the ground, a building's spare stock, or along
    /// a transport route.
    fn source_for(&self, item: Item, dst: &Building, pos: (i32, i32)) -> Option<(Src, i32, (i32, i32))> {
        let mut best: Option<(i32, Src, i32, (i32, i32))> = None;
        for (gi, g) in self.ground.iter().enumerate() {
            if g.gone || g.item != item || g.claim.is_some() || self.map.cursed(g.pos.0, g.pos.1) && false {
                continue;
            }
            let d = dist(pos, g.pos) + dist(g.pos, dst.door());
            if best.as_ref().is_none_or(|b| d < b.0) {
                best = Some((d, Src::Ground(gi), g.n, g.pos));
            }
        }
        for b in self.bs.iter() {
            if b.id == dst.id || b.team != Team::Hold || !b.done() {
                continue;
            }
            let mut have = self.spare(b, item);
            if b.routes.contains(&dst.id) {
                have = have.max(b.has(item) - b.reserved(item));
            }
            if have > 0 {
                let d = dist(pos, b.door()) + dist(b.door(), dst.door());
                if best.as_ref().is_none_or(|x| d < x.0) {
                    best = Some((d, Src::Stock(b.id), have, b.door()));
                }
            }
        }
        best.map(|b| (b.1, b.2, b.3))
    }

    /// What a carrier should take where: the first unmet want, then spare dew to the Hall.
    pub(crate) fn find_delivery(&self, i: usize) -> Option<Task> {
        let pos = self.cs[i].pos;
        let mut order: Vec<&Building> = self
            .bs
            .iter()
            .filter(|b| b.team == Team::Hold && !b.dead && (b.done() || !b.res.is_empty()))
            .collect();
        order.sort_by_key(|b| (b.res.is_empty() as i32, dist(pos, b.door())));
        for dst in order {
            let items: Vec<Item> = dst.keeps.iter().map(|k| k.0).chain(dst.res.iter().map(|r| r.0)).collect();
            for item in items {
                let w = self.want(dst, item);
                if w <= 0 {
                    continue;
                }
                if let Some((src, have, _)) = self.source_for(item, dst, pos) {
                    let cap = if item == Item::Dew { 10 } else { 1 };
                    let n = w.min(have).min(cap);
                    if n > 0 {
                        return Some(Task::Deliver { item, n, src, dst: dst.id, picked: false });
                    }
                }
            }
        }
        // Spare dew goes to the Hall, which keeps whatever no one else wants.
        let hall = self.hall()?;
        let mut best: Option<(i32, Src, i32)> = None;
        for (gi, g) in self.ground.iter().enumerate() {
            if !g.gone && g.item == Item::Dew && g.claim.is_none() {
                let d = dist(pos, g.pos);
                if best.as_ref().is_none_or(|b| d < b.0) {
                    best = Some((d, Src::Ground(gi), g.n));
                }
            }
        }
        for b in self.bs.iter() {
            if b.id != hall && b.team == Team::Hold && b.done() {
                let s = self.spare(b, Item::Dew);
                if s > 0 {
                    let d = dist(pos, b.door());
                    if best.as_ref().is_none_or(|x| d < x.0) {
                        best = Some((d, Src::Stock(b.id), s));
                    }
                }
            }
        }
        // Orbs and the like lying about go to the Hall too.
        if best.is_none() {
            for (gi, g) in self.ground.iter().enumerate() {
                if !g.gone && item(g.item).orb && g.claim.is_none() {
                    return Some(Task::Deliver { item: g.item, n: g.n, src: Src::Ground(gi), dst: hall, picked: false });
                }
            }
        }
        best.map(|(_, src, n)| Task::Deliver { item: Item::Dew, n: n.min(10), src, dst: hall, picked: false })
    }

    /// Claim the source and promise the destination (as the task starts).
    pub(crate) fn start_delivery(&mut self, i: usize) {
        let me = self.cs[i].id;
        if let Task::Deliver { item, n, src, dst, .. } = self.cs[i].task.clone() {
            match src {
                Src::Ground(g) => self.ground[g].claim = Some(me),
                Src::Stock(b) => self.b_mut(b).reserve(item, n),
            }
            self.b_mut(dst).expect(item, n);
        }
    }

    pub(crate) fn cancel_delivery(&mut self, i: usize) {
        if let Task::Deliver { item, n, src, dst, picked } = self.cs[i].task.clone() {
            if let Some(bi) = self.bi(dst) {
                self.bs[bi].expect(item, -n);
            }
            if picked {
                let pos = self.cs[i].pos;
                self.cs[i].carry = None;
                self.drop_item(pos, item, n);
            } else {
                match src {
                    Src::Ground(g) => self.ground[g].claim = None,
                    Src::Stock(b) => {
                        if let Some(bi) = self.bi(b) {
                            self.bs[bi].reserve(item, -n);
                        }
                    }
                }
            }
        }
    }

    pub(crate) fn do_deliver(&mut self, i: usize) -> bool {
        let Task::Deliver { item, n, src, dst, picked } = self.cs[i].task.clone() else { return true };
        if self.bi(dst).is_none_or(|bi| self.bs[bi].dead) {
            self.cancel_delivery(i);
            return true;
        }
        if !picked {
            let at = match src {
                Src::Ground(g) => {
                    if self.ground[g].gone {
                        self.b_mut(dst).expect(item, -n);
                        return true;
                    }
                    self.ground[g].pos
                }
                Src::Stock(b) => {
                    let Some(bi) = self.bi(b) else {
                        self.b_mut(dst).expect(item, -n);
                        return true;
                    };
                    self.bs[bi].door()
                }
            };
            let adj = if matches!(src, Src::Ground(_)) { 0 } else { 1 };
            if !self.approach(i, at, adj) {
                return false;
            }
            let got = match src {
                Src::Ground(g) => {
                    let gr = &mut self.ground[g];
                    let k = gr.n.min(n);
                    gr.n -= k;
                    gr.claim = None;
                    if gr.n <= 0 {
                        gr.gone = true;
                    }
                    k
                }
                Src::Stock(b) => {
                    let bb = self.b_mut(b);
                    bb.reserve(item, -n);
                    let k = bb.has(item).min(n);
                    bb.add(item, -k);
                    k
                }
            };
            if got < n {
                self.b_mut(dst).expect(item, got - n);
            }
            if got <= 0 {
                return true;
            }
            self.cs[i].carry = Some((item, got));
            self.cs[i].task = Task::Deliver { item, n: got, src, dst, picked: true };
            self.cs[i].busy = 100;
            return false;
        }
        let door = self.b(dst).door();
        if !self.approach(i, door, 1) {
            return false;
        }
        self.cs[i].carry = None;
        let hall = self.hall();
        let b = self.b_mut(dst);
        b.expect(item, -n);
        b.add(item, n);
        if Some(dst) == hall && item == Item::Dew {
            self.hall_income(n);
        }
        self.cs[i].busy = 100;
        true
    }

    // ---- picking, planting and tending

    pub(crate) fn do_harvest(&mut self, i: usize, si: usize) -> bool {
        let s = self.spots[si].clone();
        if s.state != 2 {
            self.spots[si].claim = None;
            return true;
        }
        if !self.approach(i, s.pos, 1) {
            return false;
        }
        let info = spot(s.kind);
        let (ab, amt) = if s.kind == SpotKind::Fern { (Ability::Gathering, GATHER_AMOUNT) } else { (Ability::Fungiculture, FUNGI_AMOUNT) };
        let m = self.cs[i].skill(info.skill) + if s.kind == SpotKind::Fern && self.twen_alive() { 2 } else { 0 };
        let r = self.rng.r(1, 21);
        let sp = &mut self.spots[si];
        if r == 20 || r + m >= info.diff {
            sp.progress += amt * 1000;
        }
        self.cs[i].busy = ability(ab).cooldown * 10;
        self.cs[i].said = if s.kind == SpotKind::Fern { "PICKING DEW".into() } else { "PICKING FUNGUS".into() };
        if self.spots[si].progress >= info.work * 1000 + info.ripen {
            // Done: what it gives drops at the spot, and the spot goes back or is used up.
            let n = (info.n - self.spots[si].drained).max(0);
            let pos = self.spots[si].pos;
            let sp = &mut self.spots[si];
            sp.claim = None;
            sp.drained = 0;
            if sp.bonus {
                if matches!(sp.kind, SpotKind::Fern | SpotKind::Puff) {
                    sp.state = 1;
                    sp.progress = 0;
                } else {
                    sp.state = 3;
                    sp.gone_until = self.t + BONUS_REGROW as u32;
                }
            } else {
                sp.state = 0;
                sp.progress = 0;
            }
            self.drop_item(pos, info.gives, n);
            self.gain(i, Some(info.skill), info.diff);
            self.stats.note(&format!("first {} picked", item(info.gives).name), self.t);
            return true;
        }
        false
    }

    pub(crate) fn do_plant(&mut self, i: usize, si: usize) -> bool {
        let s = self.spots[si].clone();
        if s.state != 0 {
            self.spots[si].claim = None;
            return true;
        }
        if !self.approach(i, s.pos, 1) {
            return false;
        }
        let (lvl, diff) = spot(s.kind).plant.unwrap_or((0, 10));
        let m = self.cs[i].skill(Skill::Fungiculture);
        if m < lvl {
            self.spots[si].claim = None;
            return true;
        }
        self.cs[i].busy = ability(Ability::Fungiculture).cooldown * 10;
        self.cs[i].said = "PLANTING".into();
        let r = self.rng.r(1, 21);
        if r == 20 || r + m >= diff {
            let sp = &mut self.spots[si];
            sp.state = 1;
            sp.progress = 0;
            sp.claim = None;
            self.gain(i, Some(Skill::Fungiculture), diff);
            return true;
        }
        false
    }

    pub fn twen_alive(&self) -> bool {
        self.cs.iter().any(|c| c.name == "TWEN" && !c.dead)
    }

    // ---- making

    /// Something this maker can make at a workshop, with its inputs there.
    pub(crate) fn find_craft(&self, i: usize) -> Option<(Id, Item)> {
        let c = &self.cs[i];
        let skill = match c.kind {
            Kind::TreeFrog => Skill::Woodcrafting,
            Kind::Salamander => Skill::Ironcrafting,
            Kind::Daot => Skill::Enchanting,
            _ => return None,
        };
        let mut bs: Vec<&Building> = self.bs.iter().filter(|b| b.team == Team::Hold && b.done() && b.crafter.is_none()).collect();
        bs.sort_by_key(|b| dist(c.pos, b.door()));
        for b in bs {
            for it in b.info().makes {
                let ii = item(*it);
                if ii.craft != Some(skill) || c.skill(skill) < ii.craft_lvl {
                    continue;
                }
                if b.keep(*it) <= b.has(*it) {
                    continue;
                }
                if ii.inputs.iter().all(|(inp, n)| b.has(*inp) - b.reserved(*inp) >= *n) {
                    return Some((b.id, *it));
                }
            }
        }
        None
    }

    pub(crate) fn do_craft(&mut self, i: usize) -> bool {
        let Task::Craft { b, item: it, until } = self.cs[i].task.clone() else { return true };
        let Some(bi) = self.bi(b) else { return true };
        if until == 0 {
            let door = self.bs[bi].door();
            if !self.approach(i, door, 1) {
                return false;
            }
            let ii = item(it);
            let bb = &mut self.bs[bi];
            if bb.crafter.is_some_and(|x| x != self.cs[i].id) || !ii.inputs.iter().all(|(inp, n)| bb.has(*inp) - bb.reserved(*inp) >= *n) {
                return true;
            }
            for (inp, n) in ii.inputs {
                bb.add(*inp, -n);
            }
            bb.crafter = Some(self.cs[i].id);
            self.cs[i].task = Task::Craft { b, item: it, until: self.t + ii.time as u32 };
            self.cs[i].said = format!("MAKING {}", ii.name);
            self.cs[i].busy = 100;
            return false;
        }
        if self.t < until {
            self.cs[i].busy = 100;
            return false;
        }
        let ii = item(it);
        let bb = &mut self.bs[bi];
        bb.add(it, 1);
        bb.crafter = None;
        self.gain(i, ii.craft, ii.craft_lvl + 10);
        self.stats.note(&format!("first {} made", ii.name), self.t);
        true
    }

    // ---- building and mending

    pub(crate) fn find_repair(&self, i: usize, range: i32) -> Option<Id> {
        let c = &self.cs[i];
        self.bs
            .iter()
            .filter(|b| b.team == Team::Hold && !b.dead && !b.site && b.hp < b.max_hp && b.repairers.len() < BUILDERS && b.dist_to(c.pos) <= range)
            .min_by_key(|b| b.dist_to(c.pos))
            .map(|b| b.id)
    }

    pub(crate) fn do_repair(&mut self, i: usize, b: Id) -> bool {
        let Some(bi) = self.bi(b) else { return true };
        if self.bs[bi].dead || self.bs[bi].hp >= self.bs[bi].max_hp {
            self.bs[bi].repairers.retain(|r| *r != self.cs[i].id);
            return true;
        }
        let me = self.cs[i].id;
        if !self.bs[bi].repairers.contains(&me) {
            if self.bs[bi].repairers.len() >= BUILDERS {
                return true;
            }
            self.bs[bi].repairers.push(me);
        }
        let target = {
            let bb = &self.bs[bi];
            let p = self.cs[i].pos;
            (p.0.clamp(bb.pos.0, bb.pos.0 + bb.fp.0 - 1), p.1.clamp(bb.pos.1, bb.pos.1 + bb.fp.1 - 1))
        };
        if !self.approach(i, target, 1) {
            return false;
        }
        let c = &self.cs[i];
        let shovel = c.has_shovel();
        let (amt, cd) = if c.has_ability(Ability::HeavyRepair) && shovel.is_some() {
            (item(shovel.unwrap()).mend, 3)
        } else if c.has_ability(Ability::Digging) && shovel.is_some() {
            (item(shovel.unwrap()).mend, 10)
        } else {
            (1, ability(Ability::Mending).cooldown)
        };
        let site = self.bs[bi].hp < self.bs[bi].max_hp / 2 && self.bs[bi].grown == 0;
        let bb = &mut self.bs[bi];
        bb.hp = (bb.hp + amt).min(bb.max_hp);
        self.cs[i].busy = cd * 10;
        self.cs[i].said = if site { "DIGGING A NEW BUILDING".into() } else { "MENDING".into() };
        self.gain(i, None, 10);
        if self.bs[bi].hp >= self.bs[bi].max_hp {
            let k = self.bs[bi].kind;
            self.bs[bi].repairers.clear();
            if self.bs[bi].grown == 0 {
                self.bs[bi].grown = self.t;
                self.events.push(Ev::Built(k));
                self.say(format!("{} IS BUILT.", bld(k).name));
                self.stats.note(&format!("{} built", bld(k).name), self.t);
                self.newest_bld = self.bs[bi].pos;
            }
            return true;
        }
        false
    }

    /// Sites gather their cost at the building they're built from; buildings hatch.
    pub(crate) fn sites_and_hatching(&mut self) {
        let t = self.t;
        for bi in 0..self.bs.len() {
            let b = &self.bs[bi];
            if b.dead {
                continue;
            }
            if b.site {
                let Some(src) = b.src else { continue };
                let Some(si) = self.bi(src) else {
                    continue;
                };
                if !self.bs[si].done() {
                    continue;
                }
                let cost = bld(self.bs[bi].kind).cost;
                if cost.iter().all(|(it, n)| self.bs[si].has(*it) >= *n) {
                    for (it, n) in cost {
                        self.bs[si].add(*it, -n);
                        self.bs[si].reserve(*it, -n);
                    }
                    let b = &mut self.bs[bi];
                    b.site = false;
                    b.hp = (b.max_hp / 10).max(1);
                    let k = b.kind;
                    self.say(format!("THE {} SITE HAS ITS MATERIALS.", bld(k).name.trim_start_matches("THE ")));
                }
                continue;
            }
            if self.bs[bi].team != Team::Hold || !self.bs[bi].done() || self.lesson {
                continue;
            }
            if let Some((k, ready)) = self.bs[bi].hatching {
                if t >= ready {
                    self.bs[bi].hatching = None;
                    let id = self.bs[bi].id;
                    let c = self.spawn(k, id, Team::Hold);
                    if kind(k).role == Role::Hero {
                        self.c_mut(c).purse = HATCH_PURSE;
                    }
                    self.events.push(Ev::Hatched(k));
                    let name = self.c(c).name.clone();
                    self.say(format!("{} HATCHED: {}.", kind(k).name, name));
                    self.stats.note(&format!("first {} hatched", kind(k).name), t);
                }
                continue;
            }
            if !self.bs[bi].auto_hatch || t < self.bs[bi].back_at {
                continue;
            }
            let id = self.bs[bi].id;
            let houses = bld(self.bs[bi].kind).houses;
            let mut next = None;
            for (k, n) in houses {
                let have = self.cs.iter().filter(|c| c.home == Some(id) && c.kind == *k && !c.dead).count() as i32
                    + self.husks.iter().filter(|h| !h.gone && h.was.home == Some(id) && h.was.kind == *k).count() as i32;
                if have < *n {
                    next = Some(*k);
                    break;
                }
            }
            let Some(k) = next else {
                if let Some(old) = self.bs[bi].hatch_res.take() {
                    self.unreserve_hatch(bi, old);
                }
                continue;
            };
            // The Olm needs one of each orb; axolotls a Sun Orb each.
            if k == Kind::Axolotl && self.find_bld(BKind::OlmsCave).is_some() && !self.cs.iter().any(|c| c.kind == Kind::Olm) {
                continue;
            }
            if self.bs[bi].hatch_res != Some(k) {
                if let Some(old) = self.bs[bi].hatch_res.take() {
                    self.unreserve_hatch(bi, old);
                }
                for (it, n) in hatch_cost(k) {
                    self.bs[bi].reserve(it, n);
                }
                self.bs[bi].hatch_res = Some(k);
            }
            let cost = hatch_cost(k);
            if cost.iter().all(|(it, n)| self.bs[bi].has(*it) >= *n) {
                for (it, n) in &cost {
                    self.bs[bi].add(*it, -n);
                    self.bs[bi].reserve(*it, -n);
                }
                self.bs[bi].hatch_res = None;
                let time = bld(self.bs[bi].kind).hatch as u32;
                self.bs[bi].hatching = Some((k, t + time));
            }
        }
    }

    fn unreserve_hatch(&mut self, bi: usize, k: Kind) {
        for (it, n) in hatch_cost(k) {
            self.bs[bi].reserve(it, -n);
        }
    }
}

/// What a hatching costs, including the hero's starting purse (document 02).
pub fn hatch_cost(k: Kind) -> Vec<(Item, i32)> {
    let mut v = match k {
        Kind::Caecilian | Kind::TreeFrog | Kind::Salamander => vec![(Item::Dew, 5)],
        Kind::Axolotl => vec![(Item::SunOrb, 1)],
        Kind::Olm => vec![(Item::SunOrb, 1), (Item::ClargentOrb, 1), (Item::MeteorOrb, 1)],
        _ => vec![],
    };
    if kind(k).role == Role::Hero {
        v.push((Item::Dew, HATCH_PURSE));
    }
    v
}
