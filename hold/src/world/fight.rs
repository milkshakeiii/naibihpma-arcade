//! Combat and conditions, as in RockCollector (document 01): d20 against defence, dodge or
//! resistance, damage dice, conditions with stacks that fade, death when health or any
//! attribute runs out, experience and skills. Plus Dry, Leeched, husks and drops.

use super::*;

impl World {
    /// Experience and practice from an activity at an encounter level (RockCollector's formulas).
    pub(crate) fn gain(&mut self, i: usize, skill: Option<Skill>, enc: i32) {
        let c = &mut self.cs[i];
        if let Some(s) = skill {
            let cur = c.skills[s as usize];
            c.skills[s as usize] += ((enc - 8).max(1) as f32).ln().max(0.0) / (cur + 1.0);
        }
        // Enemies don't grow past level 6 (Gorf excepted): blocking would make the shells
        // unbeatable, since in RockCollector every block teaches.
        if c.team != Team::Hold && (c.level >= 6 || skill.is_some_and(|s| matches!(s, Skill::Defense | Skill::Dodge | Skill::Resistance))) {
            return;
        }
        c.xp += (enc - 9).max(0).pow(2);
        if c.xp >= c.level * c.level {
            c.level += 1;
            c.xp = 0;
            let (name, lvl, kind) = (c.name.clone(), c.level, c.kind);
            let hero = c.role() == Role::Hero;
            if c.team == Team::Hold && (hero || kind == Kind::Newt && lvl == 5) {
                self.say(format!("{name} IS LEVEL {lvl}, AND SHEDS ITS SKIN."));
            }
        }
    }

    /// Danger around a creature: enemies' difficulty in sight, less its allies' (RockCollector).
    pub fn danger(&self, i: usize) -> i32 {
        let c = &self.cs[i];
        let mut d = 0;
        for o in self.cs.iter() {
            if !o.alive() || o.id == c.id || dist(o.pos, c.pos) > 10 || o.kind == Kind::Wisp && c.team == Team::Hold {
                continue;
            }
            if o.dug_in || o.stacks(Cond::Dreaming) >= 1.0 {
                continue;
            }
            if o.team == c.team {
                if o.role() != Role::Folk || c.folk() {
                    d -= o.enc();
                }
            } else if o.kind != Kind::Gorf || o.inside.is_none() {
                if o.role() == Role::Folk && c.team != Team::Hold {
                    continue;
                }
                d += o.enc();
            }
        }
        d
    }

    pub fn nearest_enemy(&self, i: usize, range: i32) -> Option<Id> {
        let c = &self.cs[i];
        self.cs
            .iter()
            .filter(|o| o.alive() && o.team != c.team && dist(o.pos, c.pos) <= range && !o.dug_in && !(o.kind == Kind::Gorf && o.inside.is_some()))
            .filter(|o| !(c.team == Team::Hold && o.team == Team::Wild && o.kind == Kind::Crow && dist(o.pos, c.pos) > 4))
            .min_by_key(|o| dist(o.pos, c.pos))
            .map(|o| o.id)
    }

    /// The attack this creature would use on something `d` squares away, if one is ready.
    pub(crate) fn pick_attack(&self, i: usize, d: i32) -> Option<(Ability, Option<Item>)> {
        let c = &self.cs[i];
        let mut best: Option<(Ability, Option<Item>, i32)> = None;
        for &(a, lvl) in c.info().abilities {
            if c.level < lvl || !c.has_ability(a) {
                continue;
            }
            let ai = ability(a);
            if ai.kind != AKind::Attack || !c.ready(a, self.t) {
                continue;
            }
            let (range, weapon, score) = if ai.weapons.is_empty() {
                (ai.range, None, ai.dmg.0 * ai.dmg.1)
            } else {
                // The spear up close, the sling beyond (document 01).
                let mut w = None;
                for (s, it) in &c.gear {
                    let ii = item(*it);
                    if let Some(ws) = ii.weapon {
                        if !ai.weapons.contains(&ws) || matches!(s, Slot::Shovel2) && a != Ability::DoubleSmash {
                            continue;
                        }
                        if ii.range >= d && w.is_none_or(|(_, r, sc): (Item, i32, i32)| (d <= 1 && ii.range == 1 && r > 1) || ii.dmg.0 * ii.dmg.1 > sc && r >= d) {
                            w = Some((*it, ii.range, ii.dmg.0 * ii.dmg.1));
                        }
                    }
                }
                match w {
                    Some((it, r, sc)) => (r, Some(it), sc),
                    None => continue,
                }
            };
            if range < d {
                continue;
            }
            if best.as_ref().is_none_or(|b| score > b.2) {
                best = Some((a, weapon, score));
            }
        }
        best.map(|b| (b.0, b.1))
    }

    /// The longest reach this creature has with any attack (ready or not).
    pub fn reach(&self, i: usize) -> i32 {
        let c = &self.cs[i];
        let mut r = 0;
        for &(a, lvl) in c.info().abilities {
            if c.level < lvl || !c.has_ability(a) || ability(a).kind != AKind::Attack {
                continue;
            }
            let ai = ability(a);
            if ai.weapons.is_empty() {
                r = r.max(ai.range);
            } else {
                for (_, it) in &c.gear {
                    if item(*it).weapon.is_some_and(|w| ai.weapons.contains(&w)) {
                        r = r.max(item(*it).range);
                    }
                }
            }
        }
        r
    }

    fn dice(&mut self, d: (i32, i32)) -> i32 {
        let mut s = 0;
        for _ in 0..d.0 {
            s += self.rng.r(1, d.1 + 1);
        }
        s
    }

    /// Strike a creature with an ability; returns the damage done.
    pub(crate) fn strike(&mut self, i: usize, a: Ability, weapon: Option<Item>, target: Id) -> i32 {
        let ai = ability(a);
        let t = self.t;
        let c = &mut self.cs[i];
        c.used.retain(|u| u.0 != a);
        c.used.push((a, t));
        c.busy = ai.cooldown.max(10) * 10;
        let me = c.id;
        let (skill, dmg, poison, ranged) = match weapon {
            Some(it) => {
                let ii = item(it);
                (weapon_skill(ii.weapon.unwrap()), ii.dmg, ii.poison, ii.range > 1)
            }
            None => (ai.skill, ai.dmg, (0, 0), false),
        };
        let atk = self.cs[i].skill(skill);
        let tpos = self.c(target).pos;
        let mut victims = vec![target];
        if ai.radius > 0 {
            let team = self.cs[i].team;
            victims = self.cs.iter().filter(|o| o.alive() && o.team != team && dist(o.pos, tpos) <= ai.radius).map(|o| o.id).collect();
        } else if ai.targets > 1 {
            let team = self.cs[i].team;
            let pos = self.cs[i].pos;
            let mut more: Vec<Id> = self.cs.iter().filter(|o| o.alive() && o.team != team && o.id != target && dist(o.pos, pos) <= 1).map(|o| o.id).collect();
            more.truncate(ai.targets as usize - 1);
            victims.extend(more);
        }
        if a == Ability::Lightning || a == Ability::PondLight {
            self.events.push(Ev::Bolt);
        } else {
            self.events.push(Ev::Hit);
        }
        let look = match a {
            Ability::Lightning => 2,
            Ability::PondLight => 3,
            _ if ranged => 1,
            _ => 0,
        };
        self.events.push(Ev::Shot(self.cs[i].pos, tpos, look));
        let mut total = 0;
        for v in victims {
            let Some(vi) = self.ci(v) else { continue };
            if !self.cs[vi].alive() {
                continue;
            }
            // A dreamer wakes when hit, with no vision.
            self.cs[vi].conds.retain(|c| c.0 != Cond::Dreaming);
            let roll = self.rng.r(1, 21);
            let def_skill = if weapon.is_some() {
                if ranged {
                    Skill::Dodge
                } else {
                    Skill::Defense
                }
            } else {
                Skill::Resistance
            };
            let def = 10 + self.cs[vi].skill(def_skill);
            if roll != 20 && roll + atk <= def {
                // Blocked: the defender learns from it (RockCollector).
                let enc = roll + atk;
                self.gain(vi, Some(def_skill), enc);
                continue;
            }
            let n = self.dice(dmg).max(if dmg.0 > 0 { 1 } else { 0 });
            total += n;
            if poison.0 > 0 {
                self.harm(i, vi, Cond::Poisoned, poison.0, poison.1, skill);
            }
            if let Some((cnd, base, max)) = ai.cond {
                self.harm(i, vi, cnd, base, max, ai.skill);
            }
            // Newt skin is poison (New): whatever bites a newt may be poisoned. Snakes are immune.
            if self.cs[vi].kind == Kind::Newt && !ranged && self.cs[i].kind != Kind::Snake && self.rng.r(0, 2) == 0 {
                add_cond(&mut self.cs[i], Cond::Poisoned, 2.0, 10.0);
            }
            // A heron that's hit calls its nest-mates within 15 squares (the demo, document 03).
            if self.cs[vi].kind == Kind::Heron {
                let (home, vp) = (self.cs[vi].home, self.cs[vi].pos);
                for k in 0..self.cs.len() {
                    let o = &self.cs[k];
                    if o.kind == Kind::Heron && o.alive() && o.home == home && k != vi && dist(o.pos, vp) <= HERON_CALL && !matches!(o.task, Task::Hunt { .. }) {
                        self.cs[k].task = Task::Hunt { t: me };
                    }
                }
            }
            self.hurt(vi, n, me);
        }
        total
    }

    /// Try to put a harmful condition on a creature (protection roll, RockCollector).
    pub(crate) fn harm(&mut self, i: usize, vi: usize, cnd: Cond, base: i32, max: i32, skill: Skill) {
        let ci = cond(cnd);
        let guard = ci.guard.map_or(0, |a| mod_of(self.cs[vi].attr(a)));
        let prot = 10 + self.cs[vi].level + guard;
        let roll = self.rng.r(1, 21) + self.cs[i].skill(skill);
        if roll >= prot {
            add_cond(&mut self.cs[vi], cnd, base as f32, max as f32);
        }
    }

    /// Damage a creature; it dies when its health runs out.
    pub(crate) fn hurt(&mut self, vi: usize, n: i32, by: Id) {
        if n <= 0 {
            return;
        }
        let t = self.t;
        let c = &mut self.cs[vi];
        c.dmg += n;
        c.last_hit = t;
        if let Some(h) = c.hits.iter_mut().find(|h| h.0 == by) {
            h.1 += n;
        } else {
            c.hits.push((by, n));
        }
        if c.dug_in && c.hp() > 0 {
            c.dug_in = false;
        }
        if c.hp() <= 0 {
            self.die(vi, Some(by));
        }
    }

    pub(crate) fn die(&mut self, vi: usize, by: Option<Id>) {
        if self.cs[vi].dead {
            return;
        }
        let c = self.cs[vi].clone();
        self.cs[vi].dead = true;
        // Let go of whatever it was doing.
        self.release(vi);
        let pos = c.pos;
        self.events.push(Ev::Died(c.kind, c.team == Team::Hold));
        // The killer's experience.
        if let Some(k) = by.and_then(|k| self.ci(k)) {
            let enc = 10 + c.level;
            let ws = self.cs[k].gear.iter().find_map(|g| item(g.1).weapon).map(weapon_skill);
            self.gain(k, ws, enc);
        }
        // Anyone it swallowed comes out alive.
        for o in self.cs.iter_mut() {
            if o.swallowed.is_some_and(|s| s.0 == c.id) {
                o.swallowed = None;
                o.inside = None;
                o.pos = pos;
                o.from = pos;
                o.said = "SAVED FROM THE SNAKE".into();
            }
        }
        if let Some(h) = c.swallowed_husk.clone() {
            let mut h = h;
            h.pos = pos;
            h.since = self.t;
            self.husks.push(h);
        }
        if c.team == Team::Hold {
            self.stats.deaths += 1;
            let killer = by.and_then(|b| self.cs.iter().find(|x| x.id == b)).map(|x| x.info().name);
            let place = if self.map.wet(pos.0, pos.1) { " IN THE WATER" } else if !self.map.damp(pos.0, pos.1) { " ON DRY GROUND" } else { "" };
            match killer {
                Some(k) => self.say(format!("A {k} TOOK {}{place}.", c.name)),
                None => self.say(format!("{} THE {} HAS FALLEN{place}.", c.name, c.info().name)),
            }
            if c.carry.is_some() {
                // Already dropped by release.
            }
            // Friends take it hard (document 01).
            let killer_kind = by.and_then(|b| self.cs.iter().find(|x| x.id == b)).map(|x| x.kind);
            for o in self.cs.iter_mut() {
                if o.friend == Some(c.id) {
                    o.friend = None;
                    if o.trait_ == Trait::Bold {
                        o.grudge = killer_kind;
                    }
                }
            }
            // A husk, with its purse still in it, unless a snake swallowed it.
            if c.swallowed.is_none() {
                let id = self.new_id();
                let mut was = c.clone();
                was.dead = false;
                was.dmg = 0;
                was.conds.clear();
                was.task = Task::Idle { until: 0 };
                self.husks.push(Husk { id, pos, was: Box::new(was), since: self.t, claim: None, gone: false, home: false });
            }
        } else {
            self.stats.kills += 1;
            if let Some(o) = c.orb {
                self.drop_item(pos, o, 1);
                self.say(format!("A {} FELL, CARRYING A {}.", c.info().name, item(o).name));
            }
            let dew = match c.kind {
                Kind::Heron => 3,
                Kind::Snake => 2,
                Kind::Turtle => 10,
                Kind::Wisp => c.drained,
                Kind::Crow => 0,
                _ => 0,
            };
            if let Some((it, n)) = c.carry {
                self.drop_item(pos, it, n);
            }
            if dew > 0 {
                self.drop_item(pos, Item::Dew, dew);
            }
            // A freed spirit goes home to the Star Pond to be regrown.
            if c.kind == Kind::Wisp && c.name.starts_with("SPIRIT OF ") {
                if let Some(h) = self.husks.iter_mut().find(|h| h.gone && h.was.id == c.drained as u32 + 0 && false) {
                    h.gone = false;
                }
                self.free_spirit(&c);
            }
            // Attack rewards on it are paid.
            self.pay_attack(RTarget::Creature(c.id), &c.hits);
        }
    }

    fn free_spirit(&mut self, c: &Creature) {
        if let Some(sp) = self.find_bld(BKind::StarPond) {
            let door = self.b(sp).door();
            if let Some(h) = self.husks.iter_mut().find(|h| h.gone && h.was.name == c.name.trim_start_matches("SPIRIT OF ")) {
                h.gone = false;
                h.pos = door;
                h.home = true;
                h.since = self.t;
                self.say(format!("{} IS FREE, AND GOES HOME TO THE STAR POND.", c.name.trim_start_matches("SPIRIT OF ")));
            }
        }
    }

    /// Drop claims and carried things when a task ends badly.
    pub(crate) fn release(&mut self, i: usize) {
        let me = self.cs[i].id;
        match self.cs[i].task.clone() {
            Task::Deliver { .. } => self.cancel_delivery(i),
            Task::Harvest { s } | Task::Plant { s } => {
                if self.spots[s].claim == Some(me) {
                    self.spots[s].claim = None;
                }
            }
            Task::Tend { s, .. } => {
                if self.spots[s].tender == Some(me) {
                    self.spots[s].tender = None;
                }
            }
            Task::Craft { b, until, .. } => {
                if let Some(bi) = self.bi(b) {
                    if self.bs[bi].crafter == Some(me) {
                        self.bs[bi].crafter = None;
                        // Inputs taken for an unfinished craft are lost.
                        let _ = until;
                    }
                }
            }
            Task::Repair { b } => {
                if let Some(bi) = self.bi(b) {
                    self.bs[bi].repairers.retain(|r| *r != me);
                }
            }
            Task::CarryHusk { h, .. } => {
                let pos = self.cs[i].pos;
                if let Some(hk) = self.husks.iter_mut().find(|x| x.id == h) {
                    hk.claim = None;
                    hk.pos = pos;
                }
            }
            Task::PickUp { g, .. } => {
                if self.ground[g].claim == Some(me) {
                    self.ground[g].claim = None;
                }
            }
            Task::Steal { g, h } => {
                if let Some(g) = g {
                    if self.ground[g].claim == Some(me) {
                        self.ground[g].claim = None;
                    }
                }
                if let Some(h) = h {
                    if let Some(hk) = self.husks.iter_mut().find(|x| x.id == h) {
                        hk.claim = None;
                    }
                }
            }
            _ => {}
        }
        if let Some((it, n)) = self.cs[i].carry.take() {
            let pos = self.cs[i].pos;
            self.drop_item(pos, it, n);
        }
        self.cs[i].task = Task::Idle { until: 0 };
        self.cs[i].path.clear();
    }

    /// Hit a building (RockCollector's demolition multiplier).
    pub(crate) fn strike_bld(&mut self, i: usize, a: Ability, weapon: Option<Item>, b: Id) {
        let ai = ability(a);
        let t = self.t;
        {
            let c = &mut self.cs[i];
            c.used.retain(|u| u.0 != a);
            c.used.push((a, t));
            c.busy = ai.cooldown.max(10) * 10;
        }
        let Some(bi) = self.bi(b) else { return };
        let diff = bld(self.bs[bi].kind).demolition;
        let roll = self.rng.r(1, 21);
        let r = roll + self.cs[i].skill(Skill::Demolition);
        let mut mult = match weapon {
            Some(it) if item(it).range <= 1 => 2,
            Some(_) => 1,
            None => 1,
        };
        for k in [0, 10, 20] {
            if r >= diff + k {
                mult += 1;
            }
        }
        if roll == 20 {
            mult += 1;
        }
        let dmg = weapon.map_or(ai.dmg, |it| item(it).dmg);
        let n = self.dice(dmg) * mult;
        self.events.push(Ev::Hit);
        let ranged = weapon.is_some_and(|it| item(it).range > 1);
        let to = self.bs[bi].centre();
        self.events.push(Ev::Shot(self.cs[i].pos, to, if ranged { 1 } else if weapon.is_none() && ai.range > 1 { 2 } else { 0 }));
        let me = self.cs[i].id;
        self.gain(i, Some(Skill::Demolition), diff);
        self.damage_bld(bi, n, Some(me));
    }

    pub(crate) fn damage_bld(&mut self, bi: usize, n: i32, by: Option<Id>) {
        if n <= 0 || self.bs[bi].dead {
            return;
        }
        let b = &mut self.bs[bi];
        b.hp -= n;
        if let Some(by) = by {
            if let Some(h) = b.hits.iter_mut().find(|h| h.0 == by) {
                h.1 += n;
            } else {
                b.hits.push((by, n));
            }
        }
        if b.hp > 0 {
            return;
        }
        b.hp = 0;
        b.dead = true;
        let b = b.clone();
        for y in b.pos.1..b.pos.1 + b.fp.1 {
            for x in b.pos.0..b.pos.0 + b.fp.0 {
                if on_map(x, y) {
                    self.map.occ[idx(x, y)] = 0;
                }
            }
        }
        self.events.push(Ev::Built(b.kind));
        let p = b.door();
        if b.team == Team::Hold {
            self.say(format!("{} HAS BEEN DESTROYED.", bld(b.kind).name));
            // Its folk lose their home; they move in elsewhere if there's room (document 01).
            let ids: Vec<Id> = self.cs.iter().filter(|c| c.home == Some(b.id)).map(|c| c.id).collect();
            for c in ids {
                let k = self.c(c).kind;
                let other = self.bs.iter().find(|o| o.id != b.id && !o.dead && o.team == Team::Hold && o.info().houses.iter().any(|h| h.0 == k)).map(|o| o.id);
                self.c_mut(c).home = other;
            }
            // Stock spills on the ground.
            for (it, n) in b.stock.clone() {
                self.drop_item(p, it, n);
            }
        } else {
            self.say(format!("{} IS DESTROYED.", bld(b.kind).name));
            for (it, n) in b.stock.clone() {
                self.drop_item(p, it, n);
            }
            if b.kind == BKind::Nest && self.rng.r(0, 2) == 0 {
                let o = [Item::SunOrb, Item::ClargentOrb, Item::MeteorOrb][self.rng.r(0, 3) as usize];
                self.drop_item(p, o, 1);
            }
            if b.kind == BKind::GorfStone {
                for i in 0..self.map.curse.len() {
                    if self.map.curse[i] != u32::MAX && self.map.curse[i] > 0 {
                        self.map.curse[i] = self.map.curse[i].min(MINUTE);
                    }
                }
            }
            if b.kind == BKind::LeechPool {
                for v in self.map.leech.iter_mut() {
                    if *v == b.id + 1 {
                        *v = 0;
                    }
                }
            }
            // Its creatures scatter: they lose their home.
            for c in self.cs.iter_mut() {
                if c.home == Some(b.id) {
                    c.home = None;
                    if c.inside == Some(b.id) {
                        c.inside = None;
                    }
                }
            }
            self.pay_attack(RTarget::Bldg(b.id), &b.hits);
        }
    }

    // ---- conditions

    pub(crate) fn conditions_tick(&mut self, i: usize) {
        let t = self.t;
        let pos = self.cs[i].pos;
        // Swallowed: out of play, and dies if the snake isn't killed in time.
        if let Some((snake, until)) = self.cs[i].swallowed {
            if t >= until {
                let mut was = self.cs[i].clone();
                self.cs[i].dead = true;
                self.cs[i].swallowed = None;
                was.inside = None;
                was.swallowed = None;
                was.dead = false;
                was.dmg = 0;
                was.conds.clear();
                let id = self.new_id();
                let h = Husk { id, pos, was: Box::new(was), since: t, claim: None, gone: false, home: false };
                if let Some(si) = self.ci(snake) {
                    self.cs[si].swallowed_husk = Some(h);
                }
                self.stats.deaths += 1;
            }
            return;
        }
        // Dry (New): gained on dry ground, lost on damp ground, cleared in water.
        let dry = self.cs[i].info().dry;
        if dry != DryClass::Never && self.cs[i].inside.is_none() {
            let wet = self.map.wet(pos.0, pos.1);
            let damp = self.map.damp(pos.0, pos.1) || self.raining_at(pos);
            let cursed = self.map.cursed(pos.0, pos.1);
            let mut per = dry_ticks(dry) as u32;
            if is_night(t) && !cursed {
                per *= 3;
            }
            if self.weather.dry {
                per /= 2;
            }
            if self.cs[i].dry_half() {
                per *= 2;
            }
            if self.cs[i].dug_in {
                per *= 4;
            }
            if wet {
                if t % 30 == 0 {
                    sub_cond(&mut self.cs[i], Cond::Dry, 1.0);
                }
            } else if damp && !cursed {
                if t % 400 == 0 {
                    sub_cond(&mut self.cs[i], Cond::Dry, 1.0);
                }
            } else if per > 0 && (t + self.cs[i].id) % per == 0 {
                add_cond(&mut self.cs[i], Cond::Dry, 1.0, 99.0);
            }
            // Leeches: picked up standing in infested water, shaken off on dry ground.
            let li = idx(pos.0, pos.1);
            if on_map(pos.0, pos.1) && self.map.leech[li] != 0 && wet && t % 150 == 0 {
                add_cond(&mut self.cs[i], Cond::Leeched, 1.0, 3.0);
            }
            // Dry ground shakes leeches off fast; out of the water they drop off slowly anyway.
            if !damp && t % 150 == 0 || !wet && t % 600 == 0 {
                sub_cond(&mut self.cs[i], Cond::Leeched, 1.0);
            }
        }
        // Stacks fade (RockCollector), damage and healing over time.
        let mut hurt = 0;
        let mut ended_dream = false;
        {
            let c = &mut self.cs[i];
            for e in c.conds.iter_mut() {
                let ci = cond(e.0);
                if ci.every > 0 && (t as i32) % ci.every == 0 {
                    let k = if e.0 == Cond::Leeched { e.1.floor() as i32 } else { 1 };
                    hurt += ci.dmg * k;
                }
                if ci.duration > 0 {
                    e.2 += e.1.ceil() as i32;
                    if e.2 >= ci.duration {
                        let gone = (e.2 / ci.duration) as f32;
                        e.2 %= ci.duration;
                        e.1 -= gone;
                        if e.0 == Cond::Dreaming && e.1 <= 0.0 {
                            ended_dream = true;
                        }
                    }
                }
            }
            c.conds.retain(|e| e.1 > 0.0);
        }
        if hurt > 0 {
            self.hurt(i, hurt, 0);
        } else if hurt < 0 {
            let c = &mut self.cs[i];
            c.dmg = (c.dmg + hurt).max(0);
        }
        if self.cs[i].dead {
            return;
        }
        if ended_dream {
            self.wake_with_vision(i);
        }
        // Death when any attribute reaches 0 (RockCollector): drying can kill.
        if (0..6).any(|a| self.cs[i].attr(a) <= 0) {
            let name = self.cs[i].name.clone();
            if self.cs[i].stacks(Cond::Dry) > 0.0 {
                self.say(format!("{name} DRIED OUT."));
            }
            self.die(i, None);
        }
    }

    fn wake_with_vision(&mut self, i: usize) {
        // An orb carrier, or the next raid (document 01).
        let pos = self.cs[i].pos;
        let carriers: Vec<Id> = self.cs.iter().filter(|c| c.alive() && c.orb.is_some() && c.team == Team::Gorf).map(|c| c.id).collect();
        let next_raid = self
            .bs
            .iter()
            .filter(|b| !b.dead && b.team == Team::Gorf)
            .flat_map(|b| b.waves.iter().filter(|w| w.0 > self.t).map(move |w| (w.0, b.id)))
            .min();
        let pick_orb = !carriers.is_empty() && (next_raid.is_none() || self.rng.r(0, 2) == 0);
        let name = self.cs[i].name.clone();
        if pick_orb {
            let c = *carriers.iter().min_by_key(|c| dist(self.c(**c).pos, pos)).unwrap();
            let (k, o) = (self.c(c).kind, self.c(c).orb.unwrap());
            self.cs[i].dreamt = Some(Vision::Orb(c));
            self.say(format!("{name} DREAMT OF A {} WITH A {}.", kind(k).name, item(o).name));
        } else if let Some((when, nest)) = next_raid {
            self.cs[i].dreamt = Some(Vision::Raid(nest, when));
            self.say(format!("{name} DREAMT OF A RAID FROM THE {} AT {}.", bld(self.b(nest).kind).name, clock(when)));
        }
        self.events.push(Ev::Hint(5));
    }
}

pub fn add_cond(c: &mut Creature, cnd: Cond, n: f32, max: f32) {
    if let Some(e) = c.conds.iter_mut().find(|e| e.0 == cnd) {
        e.1 = (e.1 + n).min(max.max(e.1));
    } else {
        c.conds.push((cnd, n.min(max), 0));
    }
}

pub fn sub_cond(c: &mut Creature, cnd: Cond, n: f32) {
    if let Some(e) = c.conds.iter_mut().find(|e| e.0 == cnd) {
        e.1 -= n;
    }
    c.conds.retain(|e| e.1 > 0.0);
}
