//! The marsh's dangers (document 03): weather, lairs hatching and raiding, Gorf's Stones and their
//! curse, leech pools, snakes, Gorf, husks and wisps, friendships and the hints.

use super::fight::add_cond;
use super::*;

/// Hints shown once a game (document 05).
pub mod hint {
    pub const NOON: u8 = 0;
    pub const WATER: u8 = 1;
    pub const RAID: u8 = 2;
    pub const SNAKE: u8 = 3;
    pub const DREAM: u8 = 5;
    pub const SUN_ORB: u8 = 6;
    pub const STONE: u8 = 7;
    pub const SHORT: u8 = 8;
}

impl World {
    pub(crate) fn hint(&mut self, h: u8) {
        if self.hints_given & (1 << h) == 0 {
            self.hints_given |= 1 << h;
            self.events.push(Ev::Hint(h));
        }
    }

    pub(crate) fn weather_tick(&mut self) {
        let t = self.t;
        let day = t / DAY;
        let dt = day_tick(t);
        // Dawn mist on the first and third mornings.
        self.weather.mist = (day == 0 || day == 2) && dt < 900;
        let sched = [(22 * MINUTE, 3000u32, false), (30 * MINUTE, 1500, true)];
        for (at, len, rain) in sched {
            if t + MINUTE == at {
                self.say(if rain { "RAIN BY THE NEXT DUSK. NO ONE WILL DRY OUT." } else { "A DRY SPELL IS COMING." });
            }
            if t == at {
                if rain {
                    self.weather.rain = true;
                    self.say("RAIN.");
                } else {
                    self.weather.dry = true;
                    self.say("A DRY SPELL. OPEN FERNS WON'T RIPEN.");
                }
                self.weather_until = t + len;
            }
        }
        if self.weather_until != 0 && t >= self.weather_until {
            self.weather.rain = false;
            self.weather.dry = false;
            self.weather_until = 0;
            self.say("THE WEATHER CLEARS.");
        }
        if dt == NOON {
            self.hint(hint::NOON);
        }
    }

    pub(crate) fn marsh_tick(&mut self) {
        let t = self.t;
        let night = is_night(t);
        // Lairs hatch their creatures, for nothing, over time.
        for bi in 0..self.bs.len() {
            let b = &self.bs[bi];
            if b.dead || b.team == Team::Hold {
                continue;
            }
            let info = bld(b.kind);
            if info.houses.is_empty() {
                continue;
            }
            let id = b.id;
            if let Some((k, ready)) = b.hatching {
                if t >= ready {
                    self.bs[bi].hatching = None;
                    let team = self.bs[bi].team;
                    self.spawn(k, id, team);
                }
                continue;
            }
            let k = info.houses[0].0;
            let n = info.houses[0].1;
            if k == Kind::Wisp && !night {
                continue;
            }
            let have = self.cs.iter().filter(|c| c.home == Some(id) && c.kind == k && !c.dead).count() as i32;
            if have < n {
                self.bs[bi].hatching = Some((k, t + info.hatch as u32));
            }
        }
        // Raids.
        for bi in 0..self.bs.len() {
            if self.bs[bi].dead {
                continue;
            }
            let id = self.bs[bi].id;
            let waves = self.bs[bi].waves.clone();
            for (at, size) in waves {
                if t + MINUTE == at {
                    self.hint(hint::RAID);
                }
                if t == at {
                    let herons: Vec<usize> = self.cs.iter().enumerate().filter(|(_, c)| c.home == Some(id) && c.alive() && !c.wave).map(|(k, _)| k).take(size as usize).collect();
                    if herons.is_empty() {
                        continue;
                    }
                    for k in &herons {
                        self.release(*k);
                        self.cs[*k].wave = true;
                        self.cs[*k].wave_until = t + 2 * MINUTE;
                    }
                    self.events.push(Ev::Raid(id));
                    self.say(format!("{} HERONS ARE RAIDING FROM THE {}!", herons.len(), bld(self.bs[bi].kind).name));
                }
            }
        }
        if t % 15 == 0 {
            self.curse_tick();
            self.gorf_tick();
            self.husk_tick();
        }
        if t % MINUTE == 0 {
            self.stones_and_leeches();
            self.friendships();
        }
        // Snakes come in from the edges by day, every two or three minutes.
        if t >= self.next_snake && !night {
            self.next_snake = t + self.rng.r(2 * MINUTE as i32, 3 * MINUTE as i32) as u32;
            let edge = match self.rng.r(0, 3) {
                0 => (0, self.rng.r(8, MH - 2)),
                1 => (self.rng.r(2, 40), 0),
                _ => (self.rng.r(2, 40), MH - 1),
            };
            let p = self.free_near(edge);
            if let Some(roost) = self.find_bld(BKind::CrowRoost) {
                let s = self.spawn(Kind::Snake, roost, Team::Wild);
                let c = self.c_mut(s);
                c.home = None;
                c.pos = p;
                c.from = p;
                self.hint(hint::SNAKE);
            }
        }
        // A Sun Orb at the Hall.
        if t % 60 == 0 {
            if let Some(h) = self.hall() {
                if self.b(h).has(Item::SunOrb) > 0 {
                    self.hint(hint::SUN_ORB);
                }
            }
        }
    }

    fn curse_tick(&mut self) {
        for c in self.map.curse.iter_mut() {
            if *c != u32::MAX && *c > 0 {
                *c = c.saturating_sub(15);
            }
        }
        // Puddles on cursed ground dry up.
        for i in 0..self.map.puddle.len() {
            if self.map.curse[i] > 0 {
                self.map.puddle[i] = 0;
            }
        }
        let ley: Vec<(i32, i32)> = self.ley.iter().filter(|l| l.1 > self.t).map(|l| l.0).collect();
        for p in ley {
            let r = spell(Spell::LeyLight).radius;
            for y in p.1 - r..=p.1 + r {
                for x in p.0 - r..=p.0 + r {
                    if on_map(x, y) && self.map.curse[idx(x, y)] != u32::MAX {
                        self.map.curse[idx(x, y)] = 0;
                    }
                }
            }
        }
    }

    fn stones_and_leeches(&mut self) {
        let t = self.t;
        // Gorf's Stones rise as the Hold grows (document 03).
        let n = self.hold_count();
        if self.stones_risen < STONE_AT.len() && n >= STONE_AT[self.stones_risen] {
            let from = self.newest_bld;
            let mut spot = None;
            for _ in 0..200 {
                let d = self.rng.r(20, 31);
                let ang = self.rng.r(-60, 61);
                let x = from.0 + d * (100 - ang.abs()) / 100;
                let y = from.1 + d * ang / 100;
                if self.map.can_place(x, y, 1, 2, Place::Anywhere) && !self.map.cursed(x, y) {
                    spot = Some((x, y));
                    break;
                }
            }
            if let Some(p) = spot {
                let s = self.place_bld(BKind::GorfStone, p, true);
                self.b_mut(s).grown = t;
                self.stones_risen += 1;
                self.say("A GORF'S STONE HAS RISEN IN THE MARSH.");
                self.hint(hint::STONE);
            }
        }
        // Each standing stone curses the ground round it, a square wider every two minutes.
        let stones: Vec<(usize, (i32, i32))> = self.bs.iter().enumerate().filter(|(_, b)| b.kind == BKind::GorfStone && !b.dead).map(|(k, b)| (k, b.centre())).collect();
        for (k, c) in stones {
            let age = (t - self.bs[k].grown) / (2 * MINUTE);
            let r = (1 + age as i32).min(7);
            self.bs[k].curse_r = r;
            for y in c.1 - r..=c.1 + r {
                for x in c.0 - r..=c.0 + r {
                    if on_map(x, y) && dist((x, y), c) <= r {
                        let i = idx(x, y);
                        if self.map.curse[i] != u32::MAX {
                            self.map.curse[i] = self.map.curse[i].max(2 * MINUTE);
                        }
                    }
                }
            }
        }
        // Leech pools grow with the buildings by water: one for every four.
        let by_water = self.hold_blds().filter(|b| (b.pos.1 - 6..b.pos.1 + b.fp.1 + 6).any(|y| (b.pos.0 - 6..b.pos.0 + b.fp.0 + 6).any(|x| self.map.wet(x, y)))).count();
        let pools = self.bs.iter().filter(|b| b.kind == BKind::LeechPool && !b.dead).count();
        let recent_gone = self.bs.iter().any(|b| b.kind == BKind::LeechPool && b.dead && t < b.back_at);
        // The first five buildings don't count: the Hold starts with them.
        if by_water.saturating_sub(4) / 4 > pools && !recent_gone {
            let hall = self.hall().map(|h| self.b(h).centre()).unwrap_or(places::HALL);
            let mut best: Option<(i32, (i32, i32))> = None;
            for y in 0..MH - 1 {
                for x in 0..MW - 1 {
                    let ok = (0..2).all(|dy| (0..2).all(|dx| self.map.at(x + dx, y + dy) == Tile::Water && self.map.occ[idx(x + dx, y + dy)] == 0 && self.map.leech[idx(x + dx, y + dy)] == 0));
                    if ok {
                        let d = dist((x, y), hall);
                        if d > 8 && best.is_none_or(|b| d < b.0) {
                            best = Some((d, (x, y)));
                        }
                    }
                }
            }
            if let Some((_, p)) = best {
                let id = self.place_bld(BKind::LeechPool, p, true);
                for y in p.1 - 2..p.1 + 4 {
                    for x in p.0 - 2..p.0 + 4 {
                        if on_map(x, y) && self.map.at(x, y) == Tile::Water {
                            self.map.leech[idx(x, y)] = id + 1;
                        }
                    }
                }
                self.say("LEECHES HAVE COME INTO THE WATER NEAR THE HOLD.");
            }
        }
    }

    fn friendships(&mut self) {
        // Two heroes become friends when both are badly hurt in the same fight and both survive.
        let t = self.t;
        let hurt: Vec<usize> = self
            .cs
            .iter()
            .enumerate()
            .filter(|(_, c)| c.alive() && c.hero() && c.friend.is_none() && t < c.last_hit + MINUTE && c.hp() * 2 < c.max_hp())
            .map(|(k, _)| k)
            .collect();
        for a in 0..hurt.len() {
            for b in a + 1..hurt.len() {
                let (i, j) = (hurt[a], hurt[b]);
                if self.cs[i].friend.is_none() && self.cs[j].friend.is_none() && dist(self.cs[i].pos, self.cs[j].pos) <= 4 {
                    self.cs[i].friend = Some(self.cs[j].id);
                    self.cs[j].friend = Some(self.cs[i].id);
                    let (x, y) = (self.cs[i].name.clone(), self.cs[j].name.clone());
                    self.say(format!("{x} AND {y} ARE FRIENDS NOW."));
                }
            }
        }
    }

    fn gorf_tick(&mut self) {
        let Some(hol) = self.find_bld(BKind::GorfHollow) else { return };
        let hb = self.b(hol).clone();
        let Some(gi) = self.cs.iter().position(|c| c.kind == Kind::Gorf && !c.dead) else { return };
        if self.cs[gi].inside.is_some() {
            if hb.hp * 2 < hb.max_hp {
                let door = hb.door();
                let g = &mut self.cs[gi];
                g.inside = None;
                g.pos = door;
                g.from = door;
                self.say("GORF COMES OUT OF THE HOLLOW!");
            }
            return;
        }
        let t = self.t;
        let gp = self.cs[gi].pos;
        // Stay near the Hollow.
        if dist(gp, hb.centre()) > 15 {
            let c = &mut self.cs[gi];
            c.pos = hb.door();
            c.from = c.pos;
        }
        let ready = |w: &World, a: u32, slot: usize| -> bool { w.cs[gi].used.iter().find(|u| u.1 == slot as u32 + 1_000_000).is_none_or(|u| t >= u.0 as u32 + a) };
        let _ = ready;
        let spell_ready = |c: &Creature, slot: u32, every: u32| -> bool {
            c.used.iter().find(|u| matches!(u.0, Ability::Gathering) && u.1 / 10_000_000 == slot).is_none_or(|u| t >= u.1 % 10_000_000 + every)
        };
        let mark = |c: &mut Creature, slot: u32| {
            c.used.retain(|u| !(matches!(u.0, Ability::Gathering) && u.1 / 10_000_000 == slot));
            c.used.push((Ability::Gathering, slot * 10_000_000 + t));
        };
        // The croak.
        if spell_ready(&self.cs[gi], 1, 900) {
            let heroes: Vec<usize> = self.cs.iter().enumerate().filter(|(_, c)| c.alive() && c.team == Team::Hold && c.hero() && dist(c.pos, gp) <= 7).map(|(k, _)| k).collect();
            if !heroes.is_empty() {
                for k in heroes {
                    self.harm(gi, k, Cond::Frightened, 1, 1, Skill::Energy);
                }
                mark(&mut self.cs[gi], 1);
                self.say("GORF CROAKS. HEROES QUAIL.");
                self.events.push(Ev::Bolt);
                return;
            }
        }
        // Binding a husk.
        if spell_ready(&self.cs[gi], 2, 600) {
            if let Some(h) = self.husks.iter().position(|h| !h.gone && !h.home && dist(h.pos, gp) <= 15) {
                self.husk_to_wisp(h);
                mark(&mut self.cs[gi], 2);
                return;
            }
        }
        // The curse.
        if spell_ready(&self.cs[gi], 3, 1500) {
            let target = self.cs.iter().filter(|c| c.alive() && c.team == Team::Hold && dist(c.pos, gp) <= 20).map(|c| c.pos).next();
            if let Some(p) = target {
                for y in p.1 - 2..=p.1 + 2 {
                    for x in p.0 - 2..=p.0 + 2 {
                        if on_map(x, y) && self.map.curse[idx(x, y)] != u32::MAX {
                            self.map.curse[idx(x, y)] = self.map.curse[idx(x, y)].max(6000);
                        }
                    }
                }
                mark(&mut self.cs[gi], 3);
                self.say("GORF CURSES THE GROUND.");
                return;
            }
        }
        // Summoning the turtles back.
        if spell_ready(&self.cs[gi], 4, 1200) && self.danger(gi) > 0 {
            let ids: Vec<usize> = self.cs.iter().enumerate().filter(|(_, c)| c.alive() && c.kind == Kind::Turtle).map(|(k, _)| k).collect();
            for k in ids {
                self.cs[k].pos = self.free_near(gp);
                self.cs[k].from = self.cs[k].pos;
                self.cs[k].task = Task::Idle { until: 0 };
            }
            mark(&mut self.cs[gi], 4);
        }
        let _ = add_cond;
    }

    fn husk_to_wisp(&mut self, h: usize) {
        let pos = self.husks[h].pos;
        let purse = std::mem::take(&mut self.husks[h].was.purse);
        self.drop_item(pos, Item::Dew, purse);
        self.husks[h].gone = true;
        let name = self.husks[h].was.name.clone();
        let Some(hol) = self.find_bld(BKind::GorfHollow) else { return };
        let w = self.spawn(Kind::Wisp, hol, Team::Gorf);
        let c = self.c_mut(w);
        c.home = None;
        c.name = format!("SPIRIT OF {name}");
        self.say(format!("{name}'S SPIRIT HAS JOINED GORF'S PROCESSION."));
    }

    fn husk_tick(&mut self) {
        let t = self.t;
        let sp = self.find_bld(BKind::StarPond).filter(|b| self.b(*b).done());
        for h in 0..self.husks.len() {
            if self.husks[h].gone {
                continue;
            }
            // Axolotls go home alone (document 01).
            if !self.husks[h].home && self.husks[h].was.kind == Kind::Axolotl {
                if let Some(sp) = sp {
                    let door = self.b(sp).door();
                    let p = self.husks[h].pos;
                    if t % 30 == 0 {
                        self.husks[h].pos = (p.0 + (door.0 - p.0).signum(), p.1 + (door.1 - p.1).signum());
                    }
                    if dist(self.husks[h].pos, door) <= 1 {
                        self.husks[h].home = true;
                        self.husks[h].since = t;
                    }
                }
            }
            // Left out too long, a husk loses its spirit to Gorf.
            let life = HUSK_TICKS + self.husks[h].was.level as u32 * 300;
            if !self.husks[h].home && self.husks[h].claim.is_none() && t >= self.husks[h].since + life {
                self.husk_to_wisp(h);
                continue;
            }
            // At the Star Pond the axolotls regrow it, a level lower, when you've paid.
            if self.husks[h].home {
                let Some(sp) = sp else { continue };
                let cost = 10 * self.husks[h].was.level;
                let spi = self.bi(sp).unwrap();
                if self.bs[spi].has(Item::Dew) >= cost && t >= self.husks[h].since + 500 {
                    self.bs[spi].add(Item::Dew, -cost);
                    self.husks[h].gone = true;
                    let mut c = (*self.husks[h].was).clone();
                    c.id = self.new_id();
                    c.level = (c.level - 1).max(1);
                    c.xp = 0;
                    c.dead = false;
                    c.dmg = 0;
                    c.inside = None;
                    c.swallowed = None;
                    c.pos = self.free_near(self.b(sp).door());
                    c.from = c.pos;
                    c.task = Task::Idle { until: 0 };
                    c.regrown += 1;
                    c.used.clear();
                    c.path.clear();
                    if c.home.is_none_or(|hm| self.bi(hm).is_none_or(|bi| self.bs[bi].dead)) {
                        let k = c.kind;
                        c.home = self.bs.iter().find(|b| !b.dead && b.team == Team::Hold && b.info().houses.iter().any(|x| x.0 == k)).map(|b| b.id);
                    }
                    let name = c.name.clone();
                    self.cs.push(c);
                    self.say(format!("{name} HAS BEEN REGROWN AT THE STAR POND."));
                } else {
                    // Ask the Star Pond to keep the dew for it.
                    let want = cost;
                    let have = self.bs[spi].keep(Item::Dew);
                    if have < want {
                        self.bs[spi].set_keep(Item::Dew, want.min(limit(BKind::StarPond, Item::Dew)));
                    }
                }
            }
        }
    }
}
