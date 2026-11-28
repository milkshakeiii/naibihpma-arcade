//! How creatures decide (documents 01 and 03) and how they carry their tasks out. Folk keep
//! RockCollector's fixed priority lists. Heroes add Majesty's chances to each line, shifted by
//! temperament. Enemies each have a motive, the first line of their list.

use super::fight::add_cond;
use super::*;

impl World {
    fn chance(&mut self, pct: i32) -> bool {
        self.rng.r(0, 100) < pct
    }

    fn task_done(&self, i: usize) -> bool {
        let t = self.t;
        match &self.cs[i].task {
            Task::Idle { until } => t >= *until,
            Task::Wander { until, .. } | Task::Patrol { until, .. } | Task::Moths { until } => t >= *until,
            Task::Follow { t: who, until } => t >= *until || self.ci(*who).is_none_or(|w| !self.cs[w].alive()),
            Task::Fight { t: who } | Task::Hunt { t: who } => self.ci(*who).is_none_or(|w| !self.cs[w].alive()),
            Task::Raid { b } => self.bi(*b).is_none_or(|bi| self.bs[bi].dead),
            Task::Reward { f } => self.flags.iter().find(|x| x.id == *f).is_none_or(|x| x.done),
            _ => false,
        }
    }

    /// Decide what to do next if the current task is over, or danger calls for it.
    pub(crate) fn think<const FORCE: bool>(&mut self, i: usize) {
        let t = self.t;
        let mut decide = FORCE || self.task_done(i);
        // Interrupt: danger has changed enough to drop what it's doing (RockCollector).
        if !decide && self.cs[i].team == Team::Hold && t >= self.cs[i].thought + 30 {
            self.cs[i].thought = t;
            let fleeing = matches!(self.cs[i].task, Task::Flee { .. } | Task::DigIn { .. } | Task::Fight { .. } | Task::Water { .. } | Task::Reward { .. });
            if !fleeing {
                let d = self.danger(i);
                if d > self.cs[i].enc() {
                    decide = true;
                }
            }
            if self.cs[i].stacks(Cond::Frightened) >= 1.0 && !matches!(self.cs[i].task, Task::Flee { .. }) {
                decide = true;
            }
            if self.cs[i].stacks(Cond::Dry) >= 4.0 && !matches!(self.cs[i].task, Task::Water { .. } | Task::Flee { .. }) && self.cs[i].hero() {
                decide = true;
            }
        }
        if !decide {
            return;
        }
        self.release(i);
        let task = match self.cs[i].role() {
            Role::Folk => self.choose_folk(i),
            Role::Hero => self.choose_hero(i),
            Role::Enemy => self.choose_enemy(i),
        };
        self.cs[i].task = task;
        if matches!(self.cs[i].task, Task::Deliver { .. }) {
            self.start_delivery(i);
        }
        let me = self.cs[i].id;
        match self.cs[i].task.clone() {
            Task::Harvest { s } | Task::Plant { s } => self.spots[s].claim = Some(me),
            Task::Tend { s, .. } => self.spots[s].tender = Some(me),
            Task::PickUp { g, .. } => self.ground[g].claim = Some(me),
            Task::CarryHusk { h, .. } => {
                if let Some(hk) = self.husks.iter_mut().find(|x| x.id == h) {
                    hk.claim = Some(me);
                }
            }
            Task::Steal { g, h } => {
                if let Some(g) = g {
                    self.ground[g].claim = Some(me);
                }
                if let Some(h) = h {
                    if let Some(hk) = self.husks.iter_mut().find(|x| x.id == h) {
                        hk.claim = Some(me);
                    }
                }
            }
            _ => {}
        }
    }

    fn home_door(&self, i: usize) -> Option<(i32, i32)> {
        self.cs[i].home.and_then(|h| self.bi(h)).filter(|&bi| !self.bs[bi].dead).map(|bi| self.bs[bi].door())
    }

    fn flee_task(&self, i: usize) -> Task {
        let to = self.home_door(i).or_else(|| self.hall().map(|h| self.b(h).door())).unwrap_or(self.cs[i].pos);
        Task::Flee { to, until: self.t + 300 }
    }

    fn should_flee(&self, i: usize) -> bool {
        let c = &self.cs[i];
        if c.stacks(Cond::Frightened) >= 1.0 {
            return true;
        }
        let d = self.danger(i);
        let nerve = if c.hero() { c.trait_.nerve() } else { 100 };
        let hurt_pct = 100 * c.hp() / c.max_hp().max(1);
        let flee_at = match c.trait_ {
            Trait::Bold => 30,
            Trait::Greedy | Trait::Steady => 45,
            Trait::Timid => 60,
        };
        let enemies_near = self.nearest_enemy(i, 6).is_some();
        // Folk run from any predator close by, unless they're armed and defending.
        let predator = self.nearest_enemy(i, 5).is_some_and(|e| self.c(e).kind != Kind::Crow);
        let defending = matches!(c.task, Task::Fight { .. }) && c.folk();
        d > c.enc() * 3 * nerve / 200 || (c.hero() && hurt_pct < flee_at && enemies_near) || (c.folk() && predator && !defending && d > 0)
    }

    fn rest_task(&self, i: usize) -> Option<Task> {
        let c = &self.cs[i];
        if c.dmg > 0 {
            let b = c.home.filter(|h| self.bi(*h).is_some_and(|bi| self.bs[bi].done()))?;
            return Some(Task::Rest { b });
        }
        None
    }

    fn wander_task(&mut self, i: usize, r: i32) -> Task {
        let base = self.home_door(i).unwrap_or(self.cs[i].pos);
        let to = (base.0 + self.rng.r(-r, r + 1), base.1 + self.rng.r(-r, r + 1));
        let to = (to.0.clamp(0, MW - 1), to.1.clamp(0, MH - 1));
        Task::Wander { to, until: self.t + 150 + self.rng.r(0, 150) as u32 }
    }

    fn equip_task(&self, i: usize, pay: bool) -> Option<Task> {
        let c = &self.cs[i];
        let mut slots: Vec<Slot> = c.info().slots.to_vec();
        if c.kind == Kind::Toad && c.trait_ == Trait::Timid {
            slots.sort_by_key(|s| *s != Slot::Sling);
        }
        for slot in slots {
            let cur = c.gear(slot).map_or(0, |it| item(it).lvl);
            let mut best: Option<(i32, Id, Item)> = None;
            for b in self.bs.iter().filter(|b| b.team == Team::Hold && b.done()) {
                if !pay && !matches!(b.kind, BKind::Hall | BKind::ChorusLog) {
                    continue;
                }
                for (it, n) in &b.stock {
                    let ii = item(*it);
                    if *n <= b.reserved(*it) || ii.slot != Some(slot.takes()) || ii.lvl <= cur {
                        continue;
                    }
                    if pay && ii.price > c.purse {
                        continue;
                    }
                    // Folk only take what's spare beyond nothing; heroes buy what's on sale.
                    if best.is_none_or(|x| ii.lvl > x.0) {
                        best = Some((ii.lvl, b.id, *it));
                    }
                }
            }
            if let Some((_, b, it)) = best {
                return Some(Task::Equip { b, item: it, slot, pay });
            }
        }
        None
    }

    fn choose_folk(&mut self, i: usize) -> Task {
        let k = self.cs[i].kind;
        if self.should_flee(i) {
            self.cs[i].said = "FLEEING HOME".into();
            return self.flee_task(i);
        }
        if matches!(k, Kind::Newt | Kind::ChorusToad) {
            if let Some(t) = self.equip_task(i, false) {
                self.cs[i].said = "TAKING A SHOVEL".into();
                return t;
            }
            // Defend the home against raiders, if not outnumbered. A newt needs its shovel and
            // level 5 (document 01: folk fight only when cornered or defending).
            let range = if k == Kind::ChorusToad { 40 } else { 20 };
            let armed = k == Kind::ChorusToad || self.cs[i].has_ability(Ability::ShovelBlow) && self.cs[i].has_shovel().is_some();
            if let Some(home) = self.home_door(i).filter(|_| armed) {
                if let Some(e) = self.enemy_near_point(home, range, self.cs[i].team).filter(|e| self.c(*e).wave || self.c(*e).kind == Kind::Snake) {
                    if self.danger(i) <= self.cs[i].enc() {
                        self.cs[i].said = "DEFENDING HOME".into();
                        return Task::Fight { t: e };
                    }
                }
            }
        }
        if k == Kind::Daot {
            if let Some((b, it)) = self.find_craft(i) {
                return Task::Craft { b, item: it, until: 0 };
            }
            return Task::Idle { until: self.t + 150 };
        }
        if matches!(k, Kind::Newt | Kind::TreeFrog | Kind::Salamander | Kind::ChorusToad) {
            if let Some(b) = self.find_repair(i, 100) {
                return Task::Repair { b };
            }
        }
        if k == Kind::Newt {
            if let Some(p) = self.puddles_marked.first().copied() {
                self.puddles_marked.remove(0);
                self.cs[i].said = "DIGGING A PUDDLE".into();
                return Task::DigPuddle { p };
            }
        }
        if matches!(k, Kind::Newt | Kind::TreeFrog | Kind::Salamander) {
            if let Some(t) = self.find_delivery(i) {
                self.cs[i].said = "CARRYING".into();
                return t;
            }
        }
        if matches!(k, Kind::TreeFrog | Kind::Salamander) {
            if let Some((b, it)) = self.find_craft(i) {
                return Task::Craft { b, item: it, until: 0 };
            }
        }
        if k == Kind::Caecilian {
            let me = &self.cs[i];
            let skill = me.skill(Skill::Fungiculture);
            let pos = me.pos;
            let mut best: Option<(i32, usize)> = None;
            for (si, s) in self.spots.iter().enumerate() {
                if s.state == 0 && s.claim.is_none() && spot(s.kind).plant.is_some_and(|p| skill >= p.0) {
                    if self.bi(s.owner).is_some_and(|bi| self.bs[bi].done()) {
                        let d = dist(pos, s.pos);
                        if best.is_none_or(|b| d < b.0) {
                            best = Some((d, si));
                        }
                    }
                }
            }
            if let Some((_, s)) = best {
                return Task::Plant { s };
            }
            let mut best: Option<(i32, usize)> = None;
            for (si, s) in self.spots.iter().enumerate() {
                if s.kind != SpotKind::Fern && s.state == 2 && s.claim.is_none() && self.total_want(spot(s.kind).gives) > 0 {
                    let d = dist(pos, s.pos);
                    if best.is_none_or(|b| d < b.0) {
                        best = Some((d, si));
                    }
                }
            }
            if let Some((_, s)) = best {
                return Task::Harvest { s };
            }
            // Tend a ripening plot (New); it teaches as repairing does.
            let mut best: Option<(i32, usize)> = None;
            for (si, s) in self.spots.iter().enumerate() {
                if s.kind != SpotKind::Fern && s.state == 1 && s.tender.is_none() && self.bi(s.owner).is_some_and(|bi| self.bs[bi].done()) {
                    let d = dist(pos, s.pos);
                    if d < 30 && best.is_none_or(|b| d < b.0) {
                        best = Some((d, si));
                    }
                }
            }
            if let Some((_, s)) = best {
                return Task::Tend { s, until: self.t + 300 };
            }
        }
        if k == Kind::Newt {
            let pos = self.cs[i].pos;
            let mut best: Option<(i32, usize)> = None;
            for (si, s) in self.spots.iter().enumerate() {
                if s.kind == SpotKind::Fern && s.state == 2 && s.claim.is_none() {
                    let d = dist(pos, s.pos);
                    if best.is_none_or(|b| d < b.0) {
                        best = Some((d, si));
                    }
                }
            }
            if let Some((_, s)) = best {
                return Task::Harvest { s };
            }
        }
        if k == Kind::ChorusToad {
            if self.cs[i].has_ability(Ability::Call) {
                if let Some(h) = self.hero_to_bolster(i) {
                    return Task::Follow { t: h, until: self.t + 120 };
                }
            }
        }
        if let Some(t) = self.rest_task(i) {
            return t;
        }
        let r = if k == Kind::ChorusToad { 8 } else { 3 };
        self.cs[i].said = if is_night(self.t) { "RESTING AT HOME".into() } else { "IDLE".into() };
        self.wander_task(i, r)
    }

    fn hero_to_bolster(&self, i: usize) -> Option<Id> {
        let c = &self.cs[i];
        if !c.ready(Ability::Call, self.t) {
            return None;
        }
        self.cs
            .iter()
            .filter(|o| o.alive() && o.hero() && o.team == c.team && dist(o.pos, c.pos) <= 10 && o.stacks(Cond::Bolstered) < 1.0)
            .min_by_key(|o| dist(o.pos, c.pos))
            .map(|o| o.id)
    }

    pub(crate) fn enemy_near_point(&self, p: (i32, i32), range: i32, team: Team) -> Option<Id> {
        self.cs
            .iter()
            .filter(|o| o.alive() && o.team != team && o.team != Team::Wild && dist(o.pos, p) <= range && o.kind != Kind::Wisp && !(o.kind == Kind::Gorf && o.inside.is_some()))
            .chain(self.cs.iter().filter(|o| o.alive() && o.team == Team::Wild && o.kind == Kind::Snake && dist(o.pos, p) <= range))
            .min_by_key(|o| dist(o.pos, p))
            .map(|o| o.id)
    }

    fn choose_hero(&mut self, i: usize) -> Task {
        let k = self.cs[i].kind;
        let t = self.t;
        // The Olm sleeps unless it senses danger near the Hold.
        if k == Kind::Olm && matches!(self.cs[i].task, Task::Sleep) {
            if !self.danger_near_hold(20) {
                return Task::Sleep;
            }
            self.say("THE OLM WAKES.");
        }
        // 1. Defend the Hold: raiders at any of its buildings, and anything at its own home.
        let me = self.cs[i].pos;
        let raider = self
            .cs
            .iter()
            .filter(|o| o.alive() && o.team != Team::Hold && o.kind != Kind::Wisp && dist(o.pos, me) <= 25)
            .filter(|o| (o.wave || o.kind == Kind::Snake) && self.hold_blds().any(|b| b.dist_to(o.pos) <= 8))
            .min_by_key(|o| dist(o.pos, me))
            .map(|o| o.id);
        let at_home = self.home_door(i).and_then(|home| self.enemy_near_point(home, 10, Team::Hold));
        if let Some(e) = raider.or(at_home) {
            if self.odds_ok(i, e) {
                self.cs[i].said = "DEFENDING THE HOLD".into();
                return Task::Fight { t: e };
            }
        }
        // 2. Flee, or dig in.
        if self.should_flee(i) {
            let far = self.home_door(i).is_none_or(|h| dist(h, self.cs[i].pos) > 15);
            if k == Kind::Toad && far && matches!(self.cs[i].trait_, Trait::Steady | Trait::Timid) && self.cs[i].stacks(Cond::Frightened) < 1.0 && self.chance(60) {
                self.cs[i].said = "DUG IN".into();
                return Task::DigIn { until: t + 900 };
            }
            self.cs[i].said = "FLEEING HOME".into();
            return self.flee_task(i);
        }
        // 3. Find water, if very dry (New).
        if self.cs[i].stacks(Cond::Dry) >= 4.0 {
            if let Some(w) = self.nearest_water(self.cs[i].pos) {
                self.hint(super::marsh::hint::WATER);
                self.cs[i].said = "GOING TO WATER: DRY".into();
                return Task::Water { to: w };
            }
        }
        // 4. Buy an upgrade it can afford.
        let buy_chance = if self.cs[i].gear.iter().any(|g| item(g.1).weapon.is_some()) { 80 } else { 95 };
        if self.chance(buy_chance) {
            if let Some(task) = self.equip_task(i, true) {
                if let Task::Equip { item: it, .. } = task {
                    self.cs[i].said = format!("BUYING A {}", item(it).name);
                }
                return task;
            }
        }
        // 5. Fight what it can see, if not outnumbered.

        if let Some(e) = self.nearest_enemy(i, 7) {
            if self.odds_ok(i, e) && self.level_ok(i, e) {
                self.cs[i].said = format!("FIGHTING A {}", self.c(e).info().name);
                return Task::Fight { t: e };
            }
        }
        // Goliath frogs mend.
        if k == Kind::Goliath && self.cs[i].has_shovel().is_some() && self.chance(75) {
            if let Some(b) = self.find_repair(i, 100) {
                self.cs[i].said = "SHIFTING STONES".into();
                return Task::Repair { b };
            }
        }
        // Axolotls learn at the Star Pond when an ability is due.
        if k == Kind::Axolotl {
            let (lvl, learned, purse) = (self.cs[i].level, self.cs[i].learned, self.cs[i].purse);
            let due = (lvl >= 3 && learned < 1) || (lvl >= 5 && learned < 2);
            if due && self.chance(90) {
                if let Some(sp) = self.find_bld(BKind::StarPond) {
                    let price = if learned == 0 { 10 } else { 20 };
                    if purse >= price {
                        self.cs[i].said = "LEARNING IN THE DEEP WATER".into();
                        return Task::Learn { b: sp };
                    }
                }
            }
            if self.cs[i].stacks(Cond::Dry) >= 2.0 && self.chance(90) {
                if let Some(sp) = self.find_bld(BKind::StarPond) {
                    let to = self.b(sp).door();
                    self.cs[i].said = "SOAKING".into();
                    return Task::Water { to };
                }
            }
        }
        // A friend carries a fallen friend's husk home, with no reward needed (document 01).
        if let (Some(f), Some(sp)) = (self.cs[i].friend, self.find_bld(BKind::StarPond)) {
            if let Some(h) = self.husks.iter().find(|h| !h.gone && !h.home && h.claim.is_none() && h.was.id == f).map(|h| h.id) {
                self.cs[i].said = "CARRYING A FRIEND HOME".into();
                return Task::CarryHusk { h, dst: sp };
            }
        }
        // 6-7. Keep at a reward it has taken, or take one worth more than its price.
        if let Some(f) = self.flags.iter().find(|f| !f.done && f.takers.contains(&self.cs[i].id)).map(|f| f.id) {
            return Task::Reward { f };
        }
        if let Some(f) = self.best_reward(i) {
            let me = self.cs[i].id;
            if let Some(fl) = self.flags.iter_mut().find(|x| x.id == f) {
                fl.takers.push(me);
            }
            let name = self.cs[i].name.clone();
            self.events.push(Ev::Took(me, f));
            self.say(format!("{name} TAKES THE REWARD."));
            return Task::Reward { f };
        }
        // Kind habits, by chance (document 01's draft table).
        match k {
            Kind::Toad => {
                if self.chance(85) {
                    if let Some(e) = self.nearest_enemy(i, 12) {
                        if self.level_ok(i, e) && self.odds_ok(i, e) {
                            self.cs[i].said = "HUNTING".into();
                            return Task::Fight { t: e };
                        }
                    }
                }
                if self.chance(70) {
                    if let Some(g) = self.ground_to_fetch(i, 15) {
                        self.cs[i].said = "CARRYING SOMETHING HOME".into();
                        return Task::PickUp { g, dst: self.hall().unwrap_or(0) };
                    }
                }
                if self.chance(70) {
                    if let Some(b) = self.lair_near(i, 3) {
                        self.cs[i].said = "RAIDING".into();
                        return Task::Raid { b };
                    }
                }
                if is_night(t) && self.chance(50) {
                    self.cs[i].said = "EATING MOTHS UNDER THE GLOWING LOGS".into();
                    return Task::Moths { until: t + 400 };
                }
                // Young toads tag along behind an older one from their hall.
                if self.cs[i].level <= 3 {
                    let me = &self.cs[i];
                    if let Some(o) = self.cs.iter().find(|o| o.alive() && o.kind == Kind::Toad && o.home == me.home && o.level >= me.level + 2 && !matches!(o.task, Task::Rest { .. })) {
                        let oid = o.id;
                        self.cs[i].said = format!("FOLLOWING {}", self.c(oid).name);
                        return Task::Follow { t: oid, until: t + 300 };
                    }
                }
            }
            Kind::Axolotl => {
                if self.chance(75) {
                    if let Some(h) = self.charge_to_follow(i, &[Kind::Toad]) {
                        self.cs[i].said = format!("FOLLOWING {} TO HEAL", self.c(h).name);
                        return Task::Follow { t: h, until: t + 600 };
                    }
                }
                if self.chance(50) {
                    if let Some(h) = self.charge_to_follow(i, &[Kind::Bullfrog]) {
                        self.cs[i].said = format!("FOLLOWING {} TO HEAL", self.c(h).name);
                        return Task::Follow { t: h, until: t + 600 };
                    }
                }
            }
            Kind::Bullfrog => {
                if let Some(home) = self.home_door(i) {
                    if let Some(e) = self.enemy_near_point(home, 8, Team::Hold) {
                        self.cs[i].said = "HOLDING ITS GROUND".into();
                        return Task::Fight { t: e };
                    }
                    if is_night(t) && self.chance(40) {
                        self.cs[i].said = "CALLING FROM THE POOL".into();
                        return Task::Patrol { to: home, until: t + 600 };
                    }
                    if self.chance(90) {
                        let to = (home.0 + self.rng.r(-6, 7), home.1 + self.rng.r(-6, 3));
                        self.cs[i].said = "PATROLLING ITS TERRITORY".into();
                        return Task::Patrol { to, until: t + 300 };
                    }
                }
            }
            Kind::Goliath => {
                if self.chance(95) {
                    if let Some(b) = self.lair_near(i, 50) {
                        self.cs[i].said = format!("OFF TO RAID THE {}", bld(self.b(b).kind).name);
                        return Task::Raid { b };
                    }
                }
                if self.cs[i].has_shovel().is_some() && self.chance(60) {
                    if let Some(p) = self.puddle_to_widen(i) {
                        self.cs[i].said = "BUILDING A POOL".into();
                        return Task::Widen { p };
                    }
                }
                if self.chance(50) {
                    let me = self.cs[i].pos;
                    if let Some(n) = self.cs.iter().filter(|o| o.alive() && o.kind == Kind::Newt && o.carry.is_some_and(|c| c.0 == Item::Dew) && dist(o.pos, me) < 25).min_by_key(|o| dist(o.pos, me)).map(|o| o.id) {
                        self.cs[i].said = "ESCORTING A NEWT".into();
                        return Task::Follow { t: n, until: t + 400 };
                    }
                }
            }
            Kind::Olm => {
                if !self.danger_near_hold(20) && self.chance(90) {
                    self.cs[i].said = "ASLEEP IN ITS CAVE".into();
                    return Task::Sleep;
                }
            }
            _ => {}
        }
        // Dreams at the Fairy Ring (New, approved).
        if let Some(s) = self.toadstool_nearby(i) {
            let wis = mod_of(self.cs[i].attr(WIS));
            let pct = (15 - 5 * wis).clamp(2, 40);
            if self.chance(pct) {
                self.spots[s].state = 0;
                self.spots[s].progress = 0;
                add_cond(&mut self.cs[i], Cond::Dreaming, 1.0, 1.0);
                let name = self.cs[i].name.clone();
                self.say(format!("{name} ATE A TOADSTOOL, AND DREAMS."));
                self.cs[i].said = "DREAMING".into();
                return Task::Dream { until: t + DREAM_TICKS as u32 };
            }
        }
        if let Some(g) = self.ground_to_fetch(i, 10) {
            return Task::PickUp { g, dst: self.hall().unwrap_or(0) };
        }
        if let Some(t) = self.rest_task(i) {
            self.cs[i].said = "RESTING".into();
            return t;
        }
        // A hero with a fat purse goes home, and leaves its house a share (Majesty's guilds).
        if self.cs[i].purse > 60 && self.chance(40) {
            if let Some(b) = self.cs[i].home.filter(|h| self.bi(*h).is_some_and(|bi| self.bs[bi].done())) {
                self.cs[i].said = "GOING HOME".into();
                return Task::Rest { b };
            }
        }
        self.cs[i].said = "WANDERING".into();
        self.wander_task(i, 5)
    }

    /// Whether a hero likes its odds against a target: the enemies round the target against
    /// itself and the Hold's heroes already near it, scaled by its nerve (company makes it braver).
    pub(crate) fn odds_ok(&self, i: usize, e: Id) -> bool {
        let c = &self.cs[i];
        let Some(ei) = self.ci(e) else { return false };
        let tp = self.cs[ei].pos;
        let foes: i32 = self.cs.iter().filter(|o| o.alive() && o.team != Team::Hold && o.kind != Kind::Wisp && dist(o.pos, tp) <= 4).map(|o| o.enc()).sum();
        let friends: i32 = self.cs.iter().filter(|o| o.alive() && o.team == Team::Hold && o.id != c.id && (o.hero() || o.kind == Kind::ChorusToad) && dist(o.pos, tp) <= 7).map(|o| o.enc()).sum::<i32>() + c.enc();
        foes * 100 <= friends * c.trait_.nerve()
    }

    pub(crate) fn danger_near_hold(&self, r: i32) -> bool {
        self.cs.iter().any(|o| o.alive() && o.team == Team::Gorf && o.kind != Kind::Wisp && self.hold_blds().any(|b| b.dist_to(o.pos) <= r))
    }

    /// The Hold's level limits for hunting (New): up to 2 levels above itself.
    fn level_ok(&self, i: usize, e: Id) -> bool {
        self.c(e).level <= self.cs[i].level + 2 || self.cs[i].kind == Kind::Olm
    }

    fn charge_to_follow(&self, i: usize, kinds: &[Kind]) -> Option<Id> {
        let me = &self.cs[i];
        let home = self.home_door(i)?;
        self.cs
            .iter()
            .filter(|o| o.alive() && kinds.contains(&o.kind) && dist(o.pos, home) > 8 && dist(o.pos, me.pos) < 30)
            .filter(|o| matches!(o.task, Task::Reward { .. } | Task::Fight { .. } | Task::Raid { .. }))
            .min_by_key(|o| dist(o.pos, me.pos))
            .map(|o| o.id)
    }

    fn lair_near(&self, i: usize, r: i32) -> Option<Id> {
        let p = self.cs[i].pos;
        self.bs
            .iter()
            .filter(|b| !b.dead && b.team != Team::Hold && b.kind != BKind::GorfHollow && b.dist_to(p) <= r && self.map.explored[idx(b.centre().0, b.centre().1)])
            .filter(|b| b.kind != BKind::CrowRoost || b.has(Item::Dew) + b.stock.len() as i32 > 0)
            .min_by_key(|b| b.dist_to(p))
            .map(|b| b.id)
    }

    fn ground_to_fetch(&self, i: usize, r: i32) -> Option<usize> {
        let p = self.cs[i].pos;
        self.ground
            .iter()
            .enumerate()
            .filter(|(_, g)| !g.gone && g.claim.is_none() && (g.item == Item::Dew || item(g.item).orb) && dist(g.pos, p) <= r)
            .filter(|(_, g)| self.bld_at(g.pos).is_none())
            .min_by_key(|(_, g)| dist(g.pos, p))
            .map(|(gi, _)| gi)
    }

    fn puddle_to_widen(&self, i: usize) -> Option<(i32, i32)> {
        let p = self.cs[i].pos;
        let mut best = None;
        for y in 0..MH {
            for x in 0..MW {
                if self.map.puddle[idx(x, y)] == 1 && dist((x, y), p) < 30 && best.is_none_or(|b: (i32, i32)| dist((x, y), p) < dist(b, p)) {
                    best = Some((x, y));
                }
            }
        }
        best
    }

    fn toadstool_nearby(&self, i: usize) -> Option<usize> {
        let p = self.cs[i].pos;
        self.spots.iter().position(|s| s.kind == SpotKind::Hybrid && s.state == 2 && s.claim.is_none() && dist(s.pos, p) <= 3)
    }

    pub fn nearest_water(&self, p: (i32, i32)) -> Option<(i32, i32)> {
        for r in 1..30 {
            let mut best = None;
            for y in p.1 - r..=p.1 + r {
                for x in p.0 - r..=p.0 + r {
                    if dist((x, y), p) == r && self.map.wet(x, y) && self.map.leech[idx(x, y)] == 0 {
                        best = Some((x, y));
                    }
                }
            }
            if best.is_some() {
                return best;
            }
        }
        None
    }

    fn choose_enemy(&mut self, i: usize) -> Task {
        let k = self.cs[i].kind;
        let t = self.t;
        let night = is_night(t);
        let home = self.cs[i].home.and_then(|h| self.bi(h)).filter(|bi| !self.bs[*bi].dead).map(|bi| self.bs[bi].door());
        match k {
            Kind::Heron => {
                self.cs[i].said = "WATCHING ITS NEST".into();
                let home = home.unwrap_or(self.cs[i].pos);
                // A raid is in and out: after two minutes, or badly hurt, a raider flies home.
                if self.cs[i].wave && (t >= self.cs[i].wave_until || self.cs[i].hp() * 3 < self.cs[i].max_hp()) {
                    self.cs[i].wave = false;
                    return Task::Roost;
                }
                if self.cs[i].wave {
                    if let Some(e) = self.nearest_enemy(i, 6) {
                        return Task::Hunt { t: e };
                    }
                    let p = self.cs[i].pos;
                    if let Some(b) = self.hold_blds().min_by_key(|b| b.dist_to(p)).map(|b| b.id) {
                        self.cs[i].said = "RAIDING THE HOLD".into();
                        return Task::Raid { b };
                    }
                    self.cs[i].wave = false;
                }
                // Defend the nest.
                if let Some(e) = self.enemy_near_point(home, 4, Team::Gorf) {
                    self.cs[i].said = "DEFENDING ITS NEST".into();
                    return Task::Hunt { t: e };
                }
                if self.cs[i].dmg > 0 && self.chance(50) {
                    return Task::Roost;
                }
                if night {
                    self.cs[i].said = "ROOSTING".into();
                    return Task::Roost;
                }
                // By day all but two brooders fly out to hunt folk within 12 of the nest.
                let lair = self.cs[i].home;
                let brooders: Vec<Id> = self.cs.iter().filter(|o| o.home == lair && o.kind == Kind::Heron && !o.dead).take(2).map(|o| o.id).collect();
                if brooders.contains(&self.cs[i].id) {
                    return Task::Roost;
                }
                let prey = self
                    .cs
                    .iter()
                    .filter(|o| o.alive() && o.team == Team::Hold && dist(o.pos, home) <= HERON_HUNT && dist(o.pos, self.cs[i].pos) <= HERON_LOOK && !o.dug_in && o.kind != Kind::Olm)
                    .min_by_key(|o| (o.carry.is_none() as i32, (o.role() != Role::Folk) as i32, dist(o.pos, self.cs[i].pos)))
                    .map(|o| o.id);
                if let Some(p) = prey {
                    self.cs[i].said = "HUNTING FOLK".into();
                    return Task::Hunt { t: p };
                }
                let to = (home.0 + self.rng.r(-8, 9), home.1 + self.rng.r(-8, 9));
                Task::Wander { to, until: t + 200 }
            }
            Kind::Snake => {
                self.cs[i].said = "HUNTING".into();
                let p = self.cs[i].pos;
                if let Some(d) = self.cs.iter().filter(|o| !o.dead && o.dug_in && dist(o.pos, p) <= 15).min_by_key(|o| dist(o.pos, p)).map(|o| o.id) {
                    return Task::Hunt { t: d };
                }
                if let Some(b) = self.bs.iter().filter(|b| b.team == Team::Hold && b.hatching.is_some() && b.dist_to(p) <= 20).min_by_key(|b| b.dist_to(p)).map(|b| b.id) {
                    return Task::Raid { b };
                }
                let lone = self
                    .cs
                    .iter()
                    .filter(|o| o.alive() && o.team == Team::Hold && o.info().small && dist(o.pos, p) <= 12)
                    .filter(|o| !self.cs.iter().any(|a| a.alive() && a.team == Team::Hold && a.id != o.id && a.hero() && dist(a.pos, o.pos) <= 3))
                    .min_by_key(|o| dist(o.pos, p))
                    .map(|o| o.id);
                if let Some(o) = lone {
                    return Task::Hunt { t: o };
                }
                if let Some(e) = self.nearest_enemy(i, 2) {
                    return Task::Hunt { t: e };
                }
                let hall = self.hall().map(|h| self.b(h).door()).unwrap_or(p);
                let to = (p.0 + (hall.0 - p.0).signum() * 5 + self.rng.r(-3, 4), p.1 + (hall.1 - p.1).signum() * 5 + self.rng.r(-3, 4));
                Task::Wander { to: (to.0.clamp(0, MW - 1), to.1.clamp(0, MH - 1)), until: t + 200 }
            }
            Kind::Crow => {
                self.cs[i].said = if self.cs[i].carry.is_some() { "FLYING HOME WITH LOOT".into() } else { "LOOKING FOR THINGS LEFT LYING".into() };
                let dt = day_tick(t);
                let flying = dt < 900 || (DUSK..DUSK + 900).contains(&dt);
                if self.cs[i].carry.is_some() || !flying {
                    return Task::Roost;
                }
                let p = self.cs[i].pos;
                let g = self
                    .ground
                    .iter()
                    .enumerate()
                    .filter(|(_, g)| !g.gone && g.claim.is_none() && t >= g.since + CROW_ITEM_AGE && self.bld_at(g.pos).is_none())
                    .min_by_key(|(_, g)| dist(g.pos, p))
                    .map(|(gi, _)| gi);
                if let Some(g) = g {
                    return Task::Steal { g: Some(g), h: None };
                }
                let h = self.husks.iter().filter(|h| !h.gone && !h.home && h.claim.is_none() && h.was.purse > 0).min_by_key(|h| dist(h.pos, p)).map(|h| h.id);
                if let Some(h) = h {
                    return Task::Steal { g: None, h: Some(h) };
                }
                Task::Roost
            }
            Kind::Wisp => {
                if self.cs[i].name.starts_with("SPIRIT OF ") {
                    let hol = self.find_bld(BKind::GorfHollow).map(|b| self.b(b).centre()).unwrap_or(self.cs[i].pos);
                    let to = (hol.0 + self.rng.r(-7, 8), hol.1 + self.rng.r(-7, 8));
                    return Task::Wander { to, until: t + 300 };
                }
                if !night {
                    return Task::Gone;
                }
                let p = self.cs[i].pos;
                // The bullfrog's call draws wisps from 20 squares.
                if let Some(b) = self.cs.iter().find(|o| o.alive() && o.kind == Kind::Bullfrog && o.said.starts_with("CALLING") && dist(o.pos, p) <= 20).map(|o| o.id) {
                    return Task::Hunt { t: b };
                }
                let fern = self.spots.iter().enumerate().filter(|(_, s)| s.kind == SpotKind::Fern && s.state == 2 && s.drained < spot(SpotKind::Fern).n && dist(s.pos, p) <= 30).min_by_key(|(_, s)| dist(s.pos, p)).map(|(si, _)| si);
                let bld_dew = self.bs.iter().filter(|b| b.team == Team::Hold && b.done() && b.has(Item::Dew) > 0 && b.dist_to(p) <= 30).min_by_key(|b| b.dist_to(p)).map(|b| b.id);
                match (fern, bld_dew) {
                    (Some(s), _) => Task::Drain { b: None, s: Some(s), until: t + 900 },
                    (None, Some(b)) => Task::Drain { b: Some(b), s: None, until: t + 900 },
                    _ => Task::Wander { to: (p.0 + self.rng.r(-5, 6), p.1 + self.rng.r(-5, 6)), until: t + 200 },
                }
            }
            Kind::Turtle => {
                self.cs[i].said = "GUARDING GORF'S HOLLOW".into();
                let hol = home.unwrap_or(self.cs[i].pos);
                if let Some(e) = self.enemy_near_point(hol, TURTLE_GUARD, Team::Gorf) {
                    return Task::Hunt { t: e };
                }
                if self.cs[i].dmg > 0 {
                    return Task::Roost;
                }
                Task::Guard
            }
            Kind::Gorf => {
                self.cs[i].said = if self.cs[i].inside.is_some() { "INSIDE THE HOLLOW".into() } else { "CASTING".into() };
                Task::Guard
            }
            _ => Task::Idle { until: t + 100 },
        }
    }

    // ---- doing it

    pub(crate) fn perform(&mut self, i: usize) {
        let task = self.cs[i].task.clone();
        let t = self.t;
        let done = match task {
            Task::Idle { .. } => {
                self.cs[i].busy = 50;
                false
            }
            Task::Wander { to, .. } => {
                if self.approach(i, to, 0) {
                    self.cs[i].busy = 100;
                }
                false
            }
            Task::Patrol { to, .. } => {
                if self.approach(i, to, 1) {
                    self.cs[i].busy = 100;
                }
                false
            }
            Task::Moths { .. } => {
                self.cs[i].busy = 100;
                false
            }
            Task::Repair { b } => self.do_repair(i, b),
            Task::Deliver { .. } => self.do_deliver(i),
            Task::Harvest { s } => self.do_harvest(i, s),
            Task::Plant { s } => self.do_plant(i, s),
            Task::Tend { s, until } => {
                if self.spots[s].state != 1 || t >= until {
                    self.spots[s].tender = None;
                    true
                } else {
                    if self.approach(i, self.spots[s].pos, 1) {
                        self.cs[i].said = "TENDING".into();
                        self.cs[i].busy = 150;
                        if t % 300 < 15 {
                            self.gain(i, Some(Skill::Fungiculture), 10);
                        }
                    }
                    false
                }
            }
            Task::Craft { .. } => self.do_craft(i),
            Task::Equip { b, item: it, slot, pay } => self.do_equip(i, b, it, slot, pay),
            Task::Rest { b } => {
                let Some(bi) = self.bi(b) else {
                    self.cs[i].task = Task::Idle { until: 0 };
                    return;
                };
                let door = self.bs[bi].door();
                if self.approach(i, door, 1) {
                    // Home again: a hero leaves a fifth of its purse above 30 dew.
                    if self.cs[i].hero() && self.cs[i].purse > 30 {
                        let share = (self.cs[i].purse - 30) / 5;
                        self.cs[i].purse -= share;
                        self.bs[bi].add(Item::Dew, share);
                    }
                    let (amt, cd) = bld(self.bs[bi].kind).rest;
                    let c = &mut self.cs[i];
                    c.dmg = (c.dmg - amt).max(0);
                    c.busy = cd * 10;
                    c.said = "RESTING".into();
                    c.dmg == 0
                } else {
                    false
                }
            }
            Task::Fight { t: target } | Task::Hunt { t: target } => self.do_fight(i, target, false),
            Task::Raid { b } => self.do_raid(i, b),
            Task::Flee { to, until } => {
                let there = self.approach(i, to, 1);
                if there {
                    self.cs[i].busy = 100;
                }
                there && t >= until.saturating_sub(200) || t >= until
            }
            Task::Water { to } => {
                if self.approach(i, to, 0) {
                    self.cs[i].busy = 100;
                    self.cs[i].stacks(Cond::Dry) < 1.0
                } else {
                    false
                }
            }
            Task::Reward { f } => self.do_reward(i, f),
            Task::Follow { t: who, .. } => self.do_follow(i, who),
            Task::Learn { b } => {
                let Some(bi) = self.bi(b) else { return self.end(i) };
                let door = self.bs[bi].door();
                if self.approach(i, door, 1) {
                    let c = &mut self.cs[i];
                    let price = if c.learned == 0 { 10 } else { 20 };
                    if c.purse >= price {
                        c.purse -= price;
                        c.learned += 1;
                        let (name, what) = (c.name.clone(), if c.learned == 1 { "POND LIGHT" } else { "REGROWTH" });
                        self.bs[bi].add(Item::Dew, price);
                        self.say(format!("{name} LEARNED {what} IN THE STAR POND'S DEEP WATER."));
                    }
                    self.cs[i].busy = 300;
                    true
                } else {
                    false
                }
            }
            Task::DigIn { until } => {
                let c = &mut self.cs[i];
                c.dug_in = true;
                c.busy = 100;
                if t % 100 < 10 {
                    c.dmg = (c.dmg - 1).max(0);
                }
                let well = c.dmg == 0;
                if well || t >= until {
                    c.dug_in = false;
                    true
                } else {
                    false
                }
            }
            Task::Dream { until } => {
                self.cs[i].busy = 50;
                t >= until && self.cs[i].stacks(Cond::Dreaming) < 1.0
            }
            Task::DigPuddle { p } => {
                if self.approach(i, p, 1) {
                    if self.map.cursed(p.0, p.1) || self.map.blocks_walk(p.0, p.1) {
                        true
                    } else {
                        self.cs[i].busy = if self.cs[i].has_shovel().is_some() { 600 } else { 1500 };
                        self.cs[i].said = "DIGGING A PUDDLE".into();
                        self.map.puddle[idx(p.0, p.1)] = self.map.puddle[idx(p.0, p.1)].max(1);
                        true
                    }
                } else {
                    false
                }
            }
            Task::Widen { p } => {
                if self.approach(i, p, 1) {
                    self.cs[i].busy = 900;
                    for (dx, dy) in [(0, 0), (1, 0), (-1, 0), (0, 1), (0, -1)] {
                        let (x, y) = (p.0 + dx, p.1 + dy);
                        if on_map(x, y) && !self.map.blocks_walk(x, y) && !self.map.cursed(x, y) {
                            self.map.puddle[idx(x, y)] = 2;
                        }
                    }
                    let name = self.cs[i].name.clone();
                    self.say(format!("{name} WIDENED A PUDDLE INTO A POOL."));
                    true
                } else {
                    false
                }
            }
            Task::CarryHusk { h, dst } => self.do_carry_husk(i, h, dst),
            Task::PickUp { g, dst } => self.do_pickup(i, g, dst),
            Task::Sleep => {
                if let Some(d) = self.home_door(i) {
                    self.approach(i, d, 0);
                }
                self.cs[i].busy = 300;
                self.cs[i].said = "ASLEEP IN ITS CAVE".into();
                self.danger_near_hold(20)
            }
            Task::Roost => {
                let Some(d) = self.home_door(i) else { return self.end(i) };
                if self.approach(i, d, 1) {
                    let hb = self.cs[i].home.unwrap();
                    if let Some(bi) = self.bi(hb) {
                        let (amt, cd) = bld(self.bs[bi].kind).rest;
                        let c = &mut self.cs[i];
                        c.dmg = (c.dmg - amt).max(0);
                        c.busy = cd * 10;
                        if let Some((it, n)) = c.carry.take() {
                            self.bs[bi].add(it, n);
                        }
                    }
                    self.cs[i].busy = self.cs[i].busy.max(200);
                    true
                } else {
                    false
                }
            }
            Task::Steal { g, h } => self.do_steal(i, g, h),
            Task::Drain { b, s, until } => self.do_drain(i, b, s, until),
            Task::Guard => {
                if let Some(d) = self.home_door(i) {
                    let to = (d.0 + self.rng.r(-3, 4), d.1 + self.rng.r(-2, 3));
                    self.approach(i, to, 1);
                }
                self.cs[i].busy = 300;
                true
            }
            Task::Gone => {
                if let Some(d) = self.home_door(i) {
                    if self.approach(i, d, 1) {
                        self.cs[i].dead = true;
                    }
                    false
                } else {
                    self.cs[i].dead = true;
                    true
                }
            }
        };
        if done {
            self.end(i);
        }
    }

    fn end(&mut self, i: usize) {
        if let Task::Deliver { .. } = self.cs[i].task {
            // already settled by do_deliver
        }
        self.cs[i].task = Task::Idle { until: 0 };
    }

    fn do_equip(&mut self, i: usize, b: Id, it: Item, slot: Slot, pay: bool) -> bool {
        let Some(bi) = self.bi(b) else { return true };
        let door = self.bs[bi].door();
        if !self.approach(i, door, 1) {
            return false;
        }
        let price = if pay { item(it).price } else { 0 };
        if self.bs[bi].has(it) - self.bs[bi].reserved(it) <= 0 || self.cs[i].purse < price {
            return true;
        }
        self.bs[bi].add(it, -1);
        self.bs[bi].add(Item::Dew, price);
        let c = &mut self.cs[i];
        c.purse -= price;
        let old = c.gear.iter().position(|g| g.0 == slot);
        if let Some(o) = old {
            let (_, was) = c.gear.remove(o);
            self.bs[bi].add(was, 1);
        }
        let c = &mut self.cs[i];
        c.gear.push((slot, it));
        c.busy = 100;
        if pay {
            let name = c.name.clone();
            self.say(format!("{name} BOUGHT A {}.", item(it).name));
            self.stats.note(&format!("first {} bought", item(it).name), self.t);
        }
        true
    }

    /// Fight a creature: close to reach, then strike when an attack is ready.
    pub(crate) fn do_fight(&mut self, i: usize, target: Id, rewarded: bool) -> bool {
        let Some(ti) = self.ci(target) else { return true };
        if !self.cs[ti].alive() {
            return true;
        }
        let tp = self.cs[ti].pos;
        let d = dist(self.cs[i].pos, tp);
        if d > 25 && self.cs[i].team == Team::Hold && !rewarded {
            return true;
        }
        // Herons let a fleeing hero go after a moment (demo); everyone gives up a long chase.
        if self.cs[i].team != Team::Hold && d > 14 {
            return true;
        }
        // Allies first: healing, the call, Alter.
        if self.support(i) {
            return false;
        }
        let reach = self.reach(i).max(1);
        if d <= reach {
            if let Some((a, w)) = self.pick_attack(i, d) {
                // A snake swallows the small and badly hurt (document 03).
                if self.cs[i].kind == Kind::Snake && self.cs[ti].info().small && self.cs[ti].hp() * 4 <= self.cs[ti].max_hp() {
                    let until = self.t + SWALLOW_TICKS;
                    let me = self.cs[i].id;
                    let name = self.cs[ti].name.clone();
                    self.release(ti);
                    self.cs[ti].swallowed = Some((me, until));
                    self.cs[ti].inside = Some(me);
                    self.say(format!("A SNAKE SWALLOWED {name}! KILL IT IN TIME."));
                    return true;
                }
                self.strike(i, a, w, target);
            } else {
                self.cs[i].busy = 50;
            }
            return false;
        }
        // Axolotls fight from the water's edge.
        if self.cs[i].kind == Kind::Axolotl && self.cs[i].stacks(Cond::Dry) >= 2.0 && d <= 7 {
            self.cs[i].busy = 50;
            self.cs[i].said = "WAITING AT THE WATER'S EDGE".into();
            return false;
        }
        self.approach(i, tp, reach);
        false
    }

    /// Use a helpful ability on an ally if one is ready and wanted; true if it did.
    fn support(&mut self, i: usize) -> bool {
        let c = &self.cs[i];
        let t = self.t;
        let pos = c.pos;
        let team = c.team;
        for a in [Ability::Regrowth, Ability::Alter, Ability::Call] {
            if !c.has_ability(a) || !c.ready(a, t) {
                continue;
            }
            let ai = ability(a);
            let (cnd, base, max) = ai.cond.unwrap();
            let mut targets: Vec<usize> = self
                .cs
                .iter()
                .enumerate()
                .filter(|(_, o)| o.alive() && o.team == team && dist(o.pos, pos) <= ai.range && o.role() != Role::Folk && o.stacks(cnd) < max as f32)
                .filter(|(_, o)| a != Ability::Regrowth || o.dmg > 0)
                .map(|(k, _)| k)
                .collect();
            targets.truncate(ai.targets as usize);
            if targets.is_empty() {
                continue;
            }
            for k in targets {
                add_cond(&mut self.cs[k], cnd, base as f32, max as f32);
            }
            let c = &mut self.cs[i];
            c.used.retain(|u| u.0 != a);
            c.used.push((a, t));
            c.busy = ai.cooldown.max(10) * 10;
            return true;
        }
        false
    }

    pub(crate) fn do_raid(&mut self, i: usize, b: Id) -> bool {
        let Some(bi) = self.bi(b) else { return true };
        if self.bs[bi].dead {
            return true;
        }
        // Snakes eat the eggs of a hatching building.
        if self.cs[i].kind == Kind::Snake {
            if self.bs[bi].hatching.is_none() {
                return true;
            }
            let door = self.bs[bi].door();
            if self.approach(i, door, 1) {
                self.bs[bi].hatching = None;
                self.bs[bi].back_at = self.t + 900;
                let n = bld(self.bs[bi].kind).name;
                self.say(format!("A SNAKE ATE THE EGGS AT {n}."));
                // A fed snake lies still for a while.
                self.cs[i].busy = 9000;
                return true;
            }
            return false;
        }
        // Fight what gets in the way.
        if let Some(e) = self.nearest_enemy(i, 2) {
            let d = dist(self.cs[i].pos, self.c(e).pos);
            if let Some((a, w)) = self.pick_attack(i, d) {
                self.strike(i, a, w, e);
                return false;
            }
        }
        let reach = self.reach(i).max(1);
        let target = {
            let bb = &self.bs[bi];
            let p = self.cs[i].pos;
            (p.0.clamp(bb.pos.0, bb.pos.0 + bb.fp.0 - 1), p.1.clamp(bb.pos.1, bb.pos.1 + bb.fp.1 - 1))
        };
        let d = self.bs[bi].dist_to(self.cs[i].pos);
        if d <= reach {
            if let Some((a, w)) = self.pick_attack(i, d) {
                self.strike_bld(i, a, w, b);
            } else {
                self.cs[i].busy = 50;
            }
            return false;
        }
        self.approach(i, target, reach);
        false
    }

    fn do_follow(&mut self, i: usize, who: Id) -> bool {
        let Some(wi) = self.ci(who) else { return true };
        if !self.cs[wi].alive() {
            return true;
        }
        if self.support(i) {
            return false;
        }
        let wp = self.cs[wi].pos;
        // Axolotls stop where the damp ground ends.
        if self.cs[i].kind == Kind::Axolotl && !self.map.damp(wp.0, wp.1) && self.cs[i].stacks(Cond::Dry) >= 1.0 && !self.cs[i].dry_half() {
            self.cs[i].busy = 60;
            self.cs[i].said = "WAITING AT THE WATER'S EDGE".into();
            return false;
        }
        if self.approach(i, wp, 2) {
            self.cs[i].busy = 60;
        }
        false
    }

    fn do_carry_husk(&mut self, i: usize, h: Id, dst: Id) -> bool {
        let Some(hk) = self.husks.iter().position(|x| x.id == h && !x.gone && !x.home) else { return true };
        let Some(di) = self.bi(dst) else { return true };
        if self.cs[i].carry.is_none() {
            let p = self.husks[hk].pos;
            if !self.approach(i, p, 0) {
                return false;
            }
            self.cs[i].carry = Some((Item::Dew, 0));
            self.cs[i].busy = 60;
            return false;
        }
        self.husks[hk].pos = self.cs[i].pos;
        let door = self.bs[di].door();
        if !self.approach(i, door, 1) {
            return false;
        }
        self.cs[i].carry = None;
        let hh = &mut self.husks[hk];
        hh.pos = door;
        hh.home = true;
        hh.claim = None;
        let name = hh.was.name.clone();
        self.say(format!("{name}'S HUSK IS AT THE STAR POND."));
        self.complete_fetch(RTarget::Husk(h), self.cs[i].id);
        true
    }

    fn do_pickup(&mut self, i: usize, g: usize, dst: Id) -> bool {
        if self.cs[i].carry.is_none() {
            if self.ground[g].gone {
                return true;
            }
            let p = self.ground[g].pos;
            if !self.approach(i, p, 0) {
                return false;
            }
            let gr = &mut self.ground[g];
            gr.gone = true;
            gr.claim = None;
            let (it, n) = (gr.item, gr.n);
            // Heroes keep dew they find; orbs and things go home.
            if it == Item::Dew && self.cs[i].hero() {
                self.cs[i].purse += n;
                self.complete_fetch(RTarget::Ground(g), self.cs[i].id);
                return true;
            }
            self.cs[i].carry = Some((it, n));
            self.cs[i].busy = 60;
            return false;
        }
        let Some(di) = self.bi(dst) else { return true };
        let door = self.bs[di].door();
        if !self.approach(i, door, 1) {
            return false;
        }
        let (it, n) = self.cs[i].carry.take().unwrap();
        self.bs[di].add(it, n);
        self.complete_fetch(RTarget::Ground(g), self.cs[i].id);
        true
    }

    fn do_steal(&mut self, i: usize, g: Option<usize>, h: Option<Id>) -> bool {
        if let Some(g) = g {
            if self.ground[g].gone {
                return true;
            }
            let p = self.ground[g].pos;
            if !self.approach(i, p, 0) {
                return false;
            }
            let gr = &mut self.ground[g];
            gr.gone = true;
            self.cs[i].carry = Some((gr.item, gr.n));
            return true;
        }
        if let Some(h) = h {
            let Some(hk) = self.husks.iter().position(|x| x.id == h && !x.gone) else { return true };
            let p = self.husks[hk].pos;
            if !self.approach(i, p, 0) {
                return false;
            }
            let purse = std::mem::take(&mut self.husks[hk].was.purse);
            self.husks[hk].claim = None;
            self.cs[i].carry = Some((Item::Dew, purse));
            return true;
        }
        true
    }

    fn do_drain(&mut self, i: usize, b: Option<Id>, s: Option<usize>, until: u32) -> bool {
        if !is_night(self.t) || self.t >= until {
            return true;
        }
        let p = match (b, s) {
            (_, Some(s)) => {
                if self.spots[s].state != 2 || self.spots[s].drained >= spot(SpotKind::Fern).n {
                    return true;
                }
                self.spots[s].pos
            }
            (Some(b), _) => {
                let Some(bi) = self.bi(b) else { return true };
                if self.bs[bi].has(Item::Dew) <= 0 {
                    return true;
                }
                self.bs[bi].door()
            }
            _ => return true,
        };
        if !self.approach(i, p, 1) {
            return false;
        }
        self.cs[i].busy = WISP_DRAIN_EVERY as i32 * 10;
        self.cs[i].said = "DRAINING DEW".into();
        match (b, s) {
            (_, Some(s)) => self.spots[s].drained += 1,
            (Some(b), _) => self.b_mut(b).add(Item::Dew, -1),
            _ => {}
        }
        self.cs[i].drained += 1;
        false
    }
}
