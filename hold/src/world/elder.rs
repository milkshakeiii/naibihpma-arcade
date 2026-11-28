//! The Elder's hand (document 04): rewards and how heroes weigh them, stock levels, building,
//! hatching, transport routes, puddles and the Ancient Magic. The game and the bots call these.

use super::fight::add_cond;
use super::*;

/// Why a hero won't take a reward, or what it wants.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Answer {
    Wants(i32),
    TooDry,
    TooWeak,
    Busy,
    Hurt,
    Asleep,
    Taken,
}

impl Answer {
    pub fn text(self) -> String {
        match self {
            Answer::Wants(n) => format!("WANTS {n}"),
            Answer::TooDry => "TOO DRY".into(),
            Answer::TooWeak => "TOO WEAK".into(),
            Answer::Busy => "BUSY".into(),
            Answer::Hurt => "HURT".into(),
            Answer::Asleep => "ASLEEP".into(),
            Answer::Taken => "TAKEN".into(),
        }
    }
}

impl World {
    // ---- rewards

    pub fn flag(&self, f: Id) -> Option<&Flag> {
        self.flags.iter().find(|x| x.id == f && !x.done)
    }

    pub fn target_pos(&self, t: RTarget) -> Option<(i32, i32)> {
        match t {
            RTarget::Creature(c) => self.ci(c).filter(|&k| self.cs[k].alive()).map(|k| self.cs[k].pos),
            RTarget::Bldg(b) => self.bi(b).filter(|&k| !self.bs[k].dead).map(|k| self.bs[k].centre()),
            RTarget::Square(p) => Some(p),
            RTarget::Husk(h) => self.husks.iter().find(|x| x.id == h && !x.gone && !x.home).map(|x| x.pos),
            RTarget::Ground(g) => self.ground.get(g).filter(|x| !x.gone).map(|x| x.pos),
        }
    }

    fn target_danger(&self, t: RTarget) -> i32 {
        let near = |p: (i32, i32), r: i32| -> i32 { self.cs.iter().filter(|c| c.alive() && c.team == Team::Gorf && c.kind != Kind::Wisp && dist(c.pos, p) <= r).map(|c| c.enc()).sum() };
        match t {
            RTarget::Creature(c) => {
                let Some(k) = self.ci(c) else { return 0 };
                let p = self.cs[k].pos;
                self.cs[k].enc() + near(p, 4) - self.cs[k].enc().min(near(p, 4))
            }
            RTarget::Bldg(b) => {
                let Some(k) = self.bi(b) else { return 0 };
                let bb = &self.bs[k];
                if bb.team == Team::Hold {
                    return 15;
                }
                bld(bb.kind).difficulty + self.cs.iter().filter(|c| c.alive() && c.home == Some(b) && dist(c.pos, bb.centre()) <= 3).map(|c| c.enc()).sum::<i32>()
            }
            _ => {
                let p = self.target_pos(t).unwrap_or((0, 0));
                5 + near(p, 5)
            }
        }
    }

    /// What a hero says to a reward: its price, or why not (documents 01 and 04).
    pub fn asking(&self, i: usize, f: &Flag) -> Answer {
        let c = &self.cs[i];
        if f.takers.contains(&c.id) {
            return Answer::Taken;
        }
        if matches!(c.task, Task::Sleep) || c.stacks(Cond::Dreaming) >= 1.0 {
            return Answer::Asleep;
        }
        if self.flags.iter().any(|o| !o.done && o.id != f.id && o.takers.contains(&c.id)) {
            return Answer::Busy;
        }
        if c.hp() * 2 < c.max_hp() {
            return Answer::Hurt;
        }
        let Some(tp) = self.target_pos(f.target) else { return Answer::TooWeak };
        let d = self.target_danger(f.target);
        let company: i32 = f.takers.iter().filter_map(|t| self.ci(*t)).map(|k| self.cs[k].enc()).sum();
        let s = c.enc() + company;
        let ratio = d * 100 / s.max(1);
        let nerve = c.trait_.nerve();
        if let RTarget::Creature(e) = f.target {
            if let Some(k) = self.ci(e) {
                if self.cs[k].level > c.level + 2 && c.kind != Kind::Olm {
                    return Answer::TooWeak;
                }
            }
        }
        if ratio > 2 * nerve && f.kind == RKind::Attack {
            return Answer::TooWeak;
        }
        // The dry ground on the way, there and back.
        let raining = self.weather.rain;
        let dry_sq = if raining { 0 } else { self.map.dry_between(c.pos, tp) };
        let per = dry_ticks(c.info().dry) * if is_night(self.t) { 3 } else { 1 } * if c.dry_half() { 2 } else { 1 };
        let stacks = if per > 0 { dry_sq * 2 * WALK / per } else { 0 };
        if stacks + c.stacks(Cond::Dry) as i32 > 6 {
            return Answer::TooDry;
        }
        let factor = match c.info().dry {
            DryClass::Toad => 1,
            DryClass::Frog => 2,
            DryClass::Axolotl => 4,
            DryClass::Never => 0,
        };
        let far = dist(c.pos, tp);
        let mut price = (8 + d * ratio / 300 + dry_sq * factor / 2 + far / 6) * 100 / nerve * c.trait_.greed() / 100;
        let dream = match (c.dreamt, f.target) {
            (Some(Vision::Orb(x)), RTarget::Creature(y)) => x == y,
            (Some(Vision::Raid(n, _)), RTarget::Bldg(b)) => n == b,
            _ => false,
        };
        let grudge = match f.target {
            RTarget::Creature(e) => self.ci(e).is_some_and(|k| Some(self.cs[k].kind) == c.grudge),
            RTarget::Bldg(b) => c.grudge == Some(Kind::Heron) && self.bi(b).is_some_and(|k| matches!(self.bs[k].kind, BKind::Nest | BKind::LoneNest)),
            _ => false,
        };
        if dream || grudge {
            price /= 2;
        }
        Answer::Wants(((price + 4) / 5 * 5).max(5))
    }

    pub fn candidates(&self, f: Id) -> Vec<(Id, Answer)> {
        let Some(fl) = self.flag(f) else { return Vec::new() };
        self.cs.iter().enumerate().filter(|(_, c)| c.alive() && c.hero() && c.team == Team::Hold).map(|(k, c)| (c.id, self.asking(k, fl))).collect()
    }

    /// A short summary for the panel: "3 TOO DRY, 2 WANT MORE".
    pub fn candidate_summary(&self, f: Id) -> String {
        let Some(fl) = self.flag(f) else { return String::new() };
        let mut counts: Vec<(String, i32)> = Vec::new();
        for (_, a) in self.candidates(f) {
            let key = match a {
                Answer::Wants(n) if n > fl.dew => "WANT MORE".to_string(),
                Answer::Wants(_) => continue,
                Answer::Taken => "TAKEN".to_string(),
                other => other.text(),
            };
            if let Some(e) = counts.iter_mut().find(|e| e.0 == key) {
                e.1 += 1;
            } else {
                counts.push((key, 1));
            }
        }
        counts.iter().map(|(k, n)| format!("{n} {k}")).collect::<Vec<_>>().join(", ")
    }

    pub(crate) fn best_reward(&self, i: usize) -> Option<Id> {
        let mut best: Option<(i32, Id)> = None;
        for f in self.flags.iter().filter(|f| !f.done) {
            if let Answer::Wants(p) = self.asking(i, f) {
                if f.dew >= p {
                    let v = f.dew - p;
                    if best.is_none_or(|b| v > b.0) {
                        best = Some((v, f.id));
                    }
                }
            }
        }
        best.map(|b| b.1)
    }

    pub fn post_reward(&mut self, kind: RKind, target: RTarget, dew: i32) -> Result<Id, &'static str> {
        if self.flags.iter().filter(|f| !f.done).count() >= MAX_REWARDS {
            return Err("SIX REWARDS ARE ALREADY POSTED");
        }
        let hall = self.hall().ok_or("NO HALL")?;
        let ok = match (kind, target) {
            (RKind::Attack, RTarget::Creature(c)) => self.ci(c).is_some_and(|k| self.cs[k].team != Team::Hold && self.cs[k].alive()),
            (RKind::Attack, RTarget::Bldg(b)) => self.bi(b).is_some_and(|k| self.bs[k].team != Team::Hold && !self.bs[k].dead),
            (RKind::Explore, RTarget::Square(_)) => true,
            (RKind::Guard, RTarget::Bldg(b)) => self.bi(b).is_some_and(|k| self.bs[k].team == Team::Hold),
            (RKind::Fetch, RTarget::Husk(_)) | (RKind::Fetch, RTarget::Ground(_)) => self.target_pos(target).is_some(),
            _ => false,
        };
        if !ok {
            return Err("THAT REWARD CAN'T GO THERE");
        }
        if self.flags.iter().any(|f| !f.done && f.target == target && f.kind == kind) {
            return Err("THERE'S ALREADY A REWARD ON IT");
        }
        let have = self.b(hall).has(Item::Dew) - self.b(hall).reserved(Item::Dew);
        if have < dew {
            return Err("NOT ENOUGH DEW IN THE HALL");
        }
        self.b_mut(hall).add(Item::Dew, -dew);
        let id = self.new_id();
        let until = if kind == RKind::Guard { self.t + (DAY - day_tick(self.t)) } else { u32::MAX };
        self.flags.push(Flag { id, kind, target, dew, takers: Vec::new(), posted: self.t, until, done: false });
        self.wake_heroes();
        Ok(id)
    }

    /// Raise (or lower, with a negative step) a reward's dew.
    pub fn change_reward(&mut self, f: Id, by: i32) -> Result<(), &'static str> {
        let hall = self.hall().ok_or("NO HALL")?;
        let cur = self.flag(f).ok_or("NO SUCH REWARD")?.dew;
        let new = (cur + by).max(REWARD_STEP);
        let delta = new - cur;
        if delta > 0 && self.b(hall).has(Item::Dew) - self.b(hall).reserved(Item::Dew) < delta {
            return Err("NOT ENOUGH DEW IN THE HALL");
        }
        self.b_mut(hall).add(Item::Dew, -delta);
        if let Some(fl) = self.flags.iter_mut().find(|x| x.id == f) {
            fl.dew = new;
        }
        if delta > 0 {
            self.wake_heroes();
        }
        Ok(())
    }

    /// Idle and wandering heroes look at the rewards again straight away.
    fn wake_heroes(&mut self) {
        for c in self.cs.iter_mut() {
            if c.alive() && c.hero() && c.team == Team::Hold && matches!(c.task, Task::Idle { .. } | Task::Wander { .. } | Task::Moths { .. } | Task::Patrol { .. } | Task::Rest { .. }) {
                c.task = Task::Idle { until: 0 };
                c.busy = c.busy.min(10);
            }
        }
    }

    pub fn cancel_reward(&mut self, f: Id) {
        let Some(k) = self.flags.iter().position(|x| x.id == f && !x.done) else { return };
        let dew = self.flags[k].dew;
        self.flags[k].done = true;
        if let Some(h) = self.hall() {
            self.b_mut(h).add(Item::Dew, dew);
        }
    }

    fn pay(&mut self, hero: Id, n: i32) {
        let Some(k) = self.ci(hero) else { return };
        let tithe = n * TITHE_PCT / 100;
        self.cs[k].purse += n - tithe;
        if let Some(h) = self.cs[k].home {
            if let Some(bi) = self.bi(h) {
                self.bs[bi].add(Item::Dew, tithe);
            }
        }
        self.events.push(Ev::Paid(n));
    }

    /// An attack target fell: its rewards are split by damage among the heroes who hit it.
    pub(crate) fn pay_attack(&mut self, target: RTarget, hits: &[(Id, i32)]) {
        let flags: Vec<usize> = self.flags.iter().enumerate().filter(|(_, f)| !f.done && f.kind == RKind::Attack && f.target == target).map(|(k, _)| k).collect();
        for k in flags {
            let dew = self.flags[k].dew;
            self.flags[k].done = true;
            let heroes: Vec<(Id, i32)> = hits.iter().filter(|(h, _)| self.ci(*h).is_some_and(|x| self.cs[x].team == Team::Hold && self.cs[x].hero())).copied().collect();
            let total: i32 = heroes.iter().map(|h| h.1).sum();
            if total > 0 {
                let mut left = dew;
                for (n, (h, d)) in heroes.iter().enumerate() {
                    let share = if n + 1 == heroes.len() { left } else { dew * d / total };
                    left -= share;
                    self.pay(*h, share);
                }
                self.say(format!("{dew} DEW PAID FOR THE REWARD."));
            } else if !self.flags[k].takers.is_empty() {
                let takers = self.flags[k].takers.clone();
                let share = dew / takers.len() as i32;
                for h in takers {
                    self.pay(h, share);
                }
            } else if let Some(h) = self.hall() {
                self.b_mut(h).add(Item::Dew, dew);
            }
        }
    }

    pub(crate) fn complete_fetch(&mut self, target: RTarget, carrier: Id) {
        let flags: Vec<usize> = self.flags.iter().enumerate().filter(|(_, f)| !f.done && f.kind == RKind::Fetch && f.target == target).map(|(k, _)| k).collect();
        for k in flags {
            let dew = self.flags[k].dew;
            self.flags[k].done = true;
            if self.ci(carrier).is_some_and(|x| self.cs[x].hero()) {
                self.pay(carrier, dew);
            } else if let Some(h) = self.hall() {
                self.b_mut(h).add(Item::Dew, dew);
            }
        }
    }

    pub(crate) fn flags_tick(&mut self) {
        if self.t % 15 != 0 {
            return;
        }
        let t = self.t;
        for k in 0..self.flags.len() {
            if self.flags[k].done {
                continue;
            }
            let f = self.flags[k].clone();
            match f.kind {
                RKind::Explore => {
                    if let RTarget::Square(p) = f.target {
                        if let Some(h) = self.cs.iter().find(|c| c.alive() && c.hero() && dist(c.pos, p) <= 1).map(|c| c.id) {
                            self.flags[k].done = true;
                            self.pay(h, f.dew);
                            self.say("A HERO REACHED THE PLACE YOU MARKED.");
                        }
                    }
                }
                RKind::Guard => {
                    if t >= f.until {
                        self.flags[k].done = true;
                        let RTarget::Bldg(b) = f.target else { continue };
                        let Some(bi) = self.bi(b) else { continue };
                        let c = self.bs[bi].centre();
                        let near: Vec<Id> = f.takers.iter().copied().filter(|h| self.ci(*h).is_some_and(|x| dist(self.cs[x].pos, c) <= 6)).collect();
                        if near.is_empty() {
                            if let Some(h) = self.hall() {
                                self.b_mut(h).add(Item::Dew, f.dew);
                            }
                        } else {
                            let share = f.dew / near.len() as i32;
                            for h in near {
                                self.pay(h, share);
                            }
                        }
                    }
                }
                _ => {
                    if self.target_pos(f.target).is_none() {
                        // The target's gone some other way: the dew comes back.
                        self.flags[k].done = true;
                        if let Some(h) = self.hall() {
                            self.b_mut(h).add(Item::Dew, f.dew);
                        }
                    }
                }
            }
        }
        self.flags.retain(|f| !f.done || t < f.posted + 10 * MINUTE);
    }

    pub(crate) fn do_reward(&mut self, i: usize, f: Id) -> bool {
        let Some(fl) = self.flag(f).cloned() else { return true };
        match (fl.kind, fl.target) {
            (RKind::Attack, RTarget::Creature(c)) => {
                self.cs[i].said = format!("ON A REWARD: {}", self.ci(c).map_or("", |k| self.cs[k].info().name));
                self.do_fight_pub(i, c)
            }
            (RKind::Attack, RTarget::Bldg(b)) => {
                self.cs[i].said = format!("ON A REWARD: {}", bld(self.b(b).kind).name);
                self.do_raid_pub(i, b)
            }
            (RKind::Explore, RTarget::Square(p)) => {
                self.cs[i].said = "EXPLORING FOR A REWARD".into();
                if self.approach(i, p, 0) {
                    self.cs[i].busy = 60;
                }
                false
            }
            (RKind::Guard, RTarget::Bldg(b)) => {
                self.cs[i].said = "STANDING GUARD".into();
                if let Some(e) = self.nearest_enemy(i, 6) {
                    return self.do_fight_pub(i, e) && false;
                }
                let door = self.b(b).door();
                if self.approach(i, door, 3) {
                    self.cs[i].busy = 100;
                }
                false
            }
            (RKind::Fetch, RTarget::Husk(h)) => {
                self.cs[i].said = "CARRYING A HUSK HOME".into();
                match self.find_bld(BKind::StarPond) {
                    Some(sp) => self.do_carry_husk_pub(i, h, sp),
                    None => true,
                }
            }
            (RKind::Fetch, RTarget::Ground(g)) => {
                self.cs[i].said = "FETCHING".into();
                let hall = self.hall().unwrap_or(0);
                if self.ground[g].claim.is_none() || self.ground[g].claim == Some(self.cs[i].id) {
                    self.ground[g].claim = Some(self.cs[i].id);
                    self.do_pickup_pub(i, g, hall)
                } else {
                    true
                }
            }
            _ => true,
        }
    }

    // ---- stock levels, building, hatching, routes, puddles

    pub fn set_keep(&mut self, b: Id, it: Item, n: i32) {
        let k = self.b(b).kind;
        let n = n.clamp(0, limit(k, it).max(if k == BKind::Hall && it == Item::Dew { 0 } else { 0 }));
        self.b_mut(b).set_keep(it, n);
    }

    /// What can be built from a building (document 02's tree).
    pub fn buildable(src: BKind) -> Vec<BKind> {
        HOLD_BUILDINGS.iter().copied().filter(|k| bld(*k).src == Some(src)).collect()
    }

    /// Why a building can't be ordered yet, if it can't.
    pub fn build_blocker(&self, src: Id, k: BKind) -> Option<String> {
        let s = self.b(src);
        if !s.done() {
            return Some("NOT BUILT YET".into());
        }
        if bld(k).src != Some(s.kind) {
            return Some("NOT FROM HERE".into());
        }
        if self.bs.iter().any(|b| b.kind == k && !b.dead) && matches!(k, BKind::StarPond | BKind::SpawningPool | BKind::OlmsCave | BKind::DaotsGrotto | BKind::ChorusLog) {
            return Some("ONLY ONE".into());
        }
        for (it, _) in bld(k).cost {
            if item(*it).orb && !self.bs.iter().any(|b| b.team == Team::Hold && b.has(*it) > 0) && !self.ground.iter().any(|g| !g.gone && g.item == *it) {
                return Some(format!("NEEDS A {}", item(*it).name));
            }
        }
        None
    }

    pub fn order_build(&mut self, src: Id, k: BKind, pos: (i32, i32)) -> Result<Id, String> {
        if let Some(why) = self.build_blocker(src, k) {
            return Err(why);
        }
        let info = bld(k);
        if !self.map.can_place(pos.0, pos.1, info.fp.0, info.fp.1, info.place) {
            return Err(match info.place {
                Place::Rock => "CAVES NEED ROCK ABOVE THEM".into(),
                Place::Pond => "THE STAR POND NEEDS THE DEEP POND".into(),
                Place::Anywhere => "THERE'S NO ROOM THERE".into(),
            });
        }
        let near = self.hold_blds().any(|b| b.dist_to(pos) <= 20);
        if !near {
            return Err("TOO FAR FROM THE HOLD".into());
        }
        let id = self.place_bld(k, pos, false);
        self.b_mut(id).src = Some(src);
        for (it, n) in info.cost {
            self.b_mut(src).reserve(*it, *n);
        }
        self.say(format!("A SITE FOR THE {}. IT WAITS FOR ITS MATERIALS.", info.name.trim_start_matches("THE ")));
        Ok(id)
    }

    pub fn cancel_site(&mut self, b: Id) {
        let Some(bi) = self.bi(b) else { return };
        if !self.bs[bi].site {
            return;
        }
        let k = self.bs[bi].kind;
        if let Some(src) = self.bs[bi].src {
            for (it, n) in bld(k).cost {
                self.b_mut(src).reserve(*it, -n);
            }
        }
        self.bs[bi].dead = true;
        let (p, fp) = (self.bs[bi].pos, self.bs[bi].fp);
        for y in p.1..p.1 + fp.1 {
            for x in p.0..p.0 + fp.0 {
                if on_map(x, y) {
                    self.map.occ[idx(x, y)] = 0;
                }
            }
        }
        self.spots.retain(|s| s.owner != b);
    }

    pub fn toggle_auto_hatch(&mut self, b: Id) {
        let v = !self.b(b).auto_hatch;
        self.b_mut(b).auto_hatch = v;
    }

    pub fn route(&mut self, a: Id, b: Id) {
        if a != b && !self.b(a).routes.contains(&b) {
            self.b_mut(a).routes.push(b);
        }
    }

    pub fn unroute(&mut self, a: Id) {
        self.b_mut(a).routes.clear();
        for b in self.bs.iter_mut() {
            b.routes.retain(|r| *r != a);
        }
    }

    pub fn mark_puddle(&mut self, p: (i32, i32)) -> Result<(), &'static str> {
        if !on_map(p.0, p.1) || self.map.blocks_walk(p.0, p.1) || self.map.wet(p.0, p.1) {
            return Err("NOT THERE");
        }
        if self.map.cursed(p.0, p.1) {
            return Err("NOT ON CURSED GROUND");
        }
        if !self.hold_blds().any(|b| b.dist_to(p) <= 20) {
            return Err("TOO FAR FROM THE HOLD");
        }
        let hall = self.hall().ok_or("NO HALL")?;
        if self.b(hall).has(Item::Dew) < PUDDLE_COST {
            return Err("NOT ENOUGH DEW");
        }
        self.b_mut(hall).add(Item::Dew, -PUDDLE_COST);
        self.puddles_marked.push(p);
        Ok(())
    }

    // ---- the Ancient Magic

    pub fn spell_state(&self, s: Spell) -> Result<(), String> {
        let si = spell(s);
        if !self.has_done(si.from) {
            return Err(format!("NEEDS THE {}", bld(si.from).name.trim_start_matches("THE ")));
        }
        if s == Spell::OldSong && !self.cs.iter().any(|c| c.kind == Kind::ChorusToad && c.alive() && c.level >= 5) {
            return Err("THE CHORUS TOAD ISN'T LEVEL 5".into());
        }
        if s == Spell::DeepCalling && !self.cs.iter().any(|c| c.kind == Kind::Olm && c.alive()) {
            return Err("THERE IS NO OLM".into());
        }
        let slot = SPELLS.iter().position(|x| *x == s).unwrap();
        if self.t < self.spells_ready[slot] {
            return Err(format!("READY IN {}", clock(self.spells_ready[slot] - self.t)));
        }
        let hall = self.hall().ok_or("NO HALL")?;
        if self.b(hall).has(Item::Dew) < si.cost {
            return Err("NOT ENOUGH DEW".into());
        }
        Ok(())
    }

    pub fn cast(&mut self, s: Spell, at: (i32, i32)) -> Result<(), String> {
        self.spell_state(s)?;
        let si = spell(s);
        let hall = self.hall().unwrap();
        self.b_mut(hall).add(Item::Dew, -si.cost);
        let slot = SPELLS.iter().position(|x| *x == s).unwrap();
        self.spells_ready[slot] = self.t + si.recharge;
        let t = self.t;
        match s {
            Spell::RainCall => {
                self.rain_at.push((at, t + 900));
                self.say("THE ELDER CALLS THE RAIN.");
            }
            Spell::OldSong => {
                let c = self.find_bld(BKind::ChorusLog).map(|b| self.b(b).centre()).unwrap_or(at);
                for k in 0..self.cs.len() {
                    if self.cs[k].alive() && self.cs[k].hero() && dist(self.cs[k].pos, c) <= si.radius {
                        add_cond(&mut self.cs[k], Cond::Bolstered, 1.0, 1.0);
                    }
                }
                self.say("THE OLD SONG. EVERY HERO BY THE CHORUS LOG IS BOLSTERED.");
            }
            Spell::Starlight => {
                self.starlight.push((at, t + DAY));
                self.say("STARLIGHT SHOWS THE MARSH.");
            }
            Spell::LeyLight => {
                self.ley.push((at, t + DAY));
                self.say("LEY-LIGHT LIFTS GORF'S CURSE.");
            }
            Spell::DeepCalling => {
                if let Some(k) = self.cs.iter().position(|c| c.kind == Kind::Olm && c.alive()) {
                    self.cs[k].task = Task::Idle { until: 0 };
                    self.think::<true>(k);
                    if matches!(self.cs[k].task, Task::Sleep) {
                        self.cs[k].task = Task::Wander { to: at, until: t + 600 };
                    }
                }
                self.say("THE DEEP CALLING. THE OLM WAKES.");
            }
        }
        Ok(())
    }

    // thin wrappers so rewards can reuse the task code in `think`
    pub(crate) fn do_fight_pub(&mut self, i: usize, t: Id) -> bool {
        self.do_fight(i, t, true)
    }
    pub(crate) fn do_raid_pub(&mut self, i: usize, b: Id) -> bool {
        self.do_raid(i, b)
    }
    pub(crate) fn do_carry_husk_pub(&mut self, i: usize, h: Id, dst: Id) -> bool {
        self.perform_as(i, Task::CarryHusk { h, dst })
    }
    pub(crate) fn do_pickup_pub(&mut self, i: usize, g: usize, dst: Id) -> bool {
        self.perform_as(i, Task::PickUp { g, dst })
    }

    /// Do one step of a task as if it were the creature's own, keeping the reward task.
    fn perform_as(&mut self, i: usize, task: Task) -> bool {
        let keep = std::mem::replace(&mut self.cs[i].task, task);
        self.perform(i);
        let after = std::mem::replace(&mut self.cs[i].task, keep);
        matches!(after, Task::Idle { until: 0 })
    }
}
