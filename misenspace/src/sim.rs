//! The live restaurant, one beat at a time: the kitchen on the left, the dining room on the
//! right, and chutes joining the same square in both.
//!
//! Kitchen: pantry hatches put out ingredients; belts move items one tile per beat and back
//! up when blocked; a completely full loop of belts still turns. Grabber arms lift an item
//! from behind and set it down in front, and only lift what the tile in front can take, so
//! they filter for free.
//!
//! Stations are stops on a line, like a sushi belt. Machines and plating counters let through
//! what they never use, and make anything they do use wait for room. Machines cook one item
//! at a time; counters assemble a dish once every part is in and an order ticket is open for
//! it, then send it out of their output side; chutes send
//! up a dish that someone seated at their table is waiting for (promising it to them) and let
//! every other dish go by.
//!
//! Guests walk in, take the nearest free seat, order, wait, eat, pay, and leave; a guest who
//! waits too long leaves hungry.

use crate::geom::*;
use crate::model::*;
use std::collections::VecDeque;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Ev {
    Hatch,
    Slice,
    Bake,
    Plate,
    Arm,
    ChuteUp,
    Arrive,
    Served(u32),
    Hungry,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum GState {
    Queue,
    Walking,
    Waiting,
    Eating,
    Leaving { happy: bool },
    Gone,
}

#[derive(Clone, Debug)]
pub struct Guest {
    pub species: Species,
    pub x: i32,
    pub y: i32,
    pub from: (i32, i32),
    pub to: (i32, i32),
    pub state: GState,
    pub order: Item,
    pub patience: u16,
    pub max_patience: u16,
    pub seat: Option<(i32, i32)>,
    pub table: Option<(i32, i32)>,
    pub claimed: bool,
    pub timer: u16,
    path: VecDeque<(i32, i32)>,
    /// Coins paid (shown as a pop-up while eating).
    pub paid: u32,
}

#[derive(Clone)]
pub struct Sim {
    pub cells: [KCell; 64],
    /// Belt item, machine input, chute dish on its way up, or the item an arm holds.
    pub slot: [Option<Item>; 64],
    /// Machine or counter output waiting to leave.
    pub out: [Option<Item>; 64],
    /// An item passing straight through a station, and where it's heading.
    pub pass: [Option<(Item, Dir)>; 64],
    pub timer: [u8; 64],
    pub parts: [[u8; ITEM_COUNT]; 64],
    /// Arm phase: 0 back, 1 swinging out, 2 front, 3 swinging back.
    pub arm: [u8; 64],
    /// Where the item now in `slot`/`pass` came from this beat (for sliding animation).
    pub slot_from: [Option<(i32, i32)>; 64],
    pub dining: [DCell; 64],
    pub table_dish: [Option<Item>; 64],
    pub table_pop: [u8; 64],
    pub guests: Vec<Guest>,
    pub beat: u32,
    pub coins: u32,
    pub fed: u32,
    pub hungry: u32,
    pub arrived: u32,
    pub events: Vec<Ev>,
    service: Service,
    rng: u32,
    door: (i32, i32),
    door_out: Dir,
}

impl Sim {
    pub fn new(r: &Restaurant, kitchen: &[KCell; 64]) -> Self {
        let door = (0..64).find(|&i| r.dining[i] == DCell::Door).map_or((0, 7), |i| (i as i32 % N, i as i32 / N));
        let door_out = if door.1 == N - 1 {
            Dir::S
        } else if door.1 == 0 {
            Dir::N
        } else if door.0 == 0 {
            Dir::W
        } else {
            Dir::E
        };
        Self {
            cells: *kitchen,
            slot: [None; 64],
            out: [None; 64],
            pass: [None; 64],
            timer: [0; 64],
            parts: [[0; ITEM_COUNT]; 64],
            arm: [0; 64],
            slot_from: [None; 64],
            dining: r.dining,
            table_dish: [None; 64],
            table_pop: [0; 64],
            guests: Vec::new(),
            beat: 0,
            coins: 0,
            fed: 0,
            hungry: 0,
            arrived: 0,
            events: Vec::new(),
            service: r.service.clone(),
            rng: 0x2545_F491,
            door,
            door_out,
        }
    }

    pub fn service_beats(&self) -> u32 {
        self.service.beats
    }

    pub fn over(&self) -> bool {
        self.beat >= self.service.beats && self.guests.iter().all(|g| g.state == GState::Gone)
    }

    pub fn total_guests(&self) -> u32 {
        self.service.arrivals.len() as u32
    }

    pub fn stars(&self) -> u32 {
        let pct = self.fed * 100 / self.total_guests().max(1);
        self.service.stars.iter().filter(|&&s| pct >= s).count() as u32
    }

    /// The player built something mid-service: keep what still makes sense.
    pub fn rebuild(&mut self, kitchen: &[KCell; 64]) {
        for i in 0..64 {
            if self.cells[i] != kitchen[i] {
                let same_kind = Tool::of(self.cells[i]).is_some() && Tool::of(self.cells[i]) == Tool::of(kitchen[i]);
                if !same_kind {
                    self.slot[i] = None;
                    self.out[i] = None;
                    self.pass[i] = None;
                    self.timer[i] = 0;
                    self.parts[i] = [0; ITEM_COUNT];
                    self.arm[i] = 0;
                }
                self.cells[i] = kitchen[i];
            }
        }
    }

    fn rand(&mut self) -> u32 {
        let mut x = self.rng;
        x ^= x << 13;
        x ^= x >> 17;
        x ^= x << 5;
        self.rng = x;
        x
    }

    // -----------------------------------------------------------------------------------------
    // Kitchen

    /// Could the station at `i` use `item` right now (rather than letting it pass)?
    fn can_use(&self, i: usize, item: Item) -> bool {
        let (x, y) = (i as i32 % N, i as i32 / N);
        match self.cells[i] {
            KCell::Machine(m, _) => self.slot[i].is_none() && m.product(item).is_some(),
            KCell::Counter(dish, _) => dish.recipe().iter().any(|&(p, n)| p == item && self.parts[i][p.index()] < n),
            KCell::Chute => {
                item.is_dish() && self.slot[i].is_none() && self.table_dish[i].is_none() && self.wanted_at(x, y, item).is_some()
            }
            _ => false,
        }
    }

    /// Guests who have ordered `dish` and haven't been served yet: the ticket rail.
    pub fn open_tickets(&self, dish: Item) -> usize {
        self.guests
            .iter()
            .filter(|g| g.order == dish && matches!(g.state, GState::Queue | GState::Walking | GState::Waiting))
            .count()
    }

    /// Plated `dish`es already on their way: anywhere in the kitchen or on a table.
    pub fn in_flight(&self, dish: Item) -> usize {
        let mut n = 0;
        for i in 0..64 {
            n += (self.slot[i] == Some(dish)) as usize + (self.out[i] == Some(dish)) as usize;
            n += (self.pass[i].map(|p| p.0) == Some(dish)) as usize + (self.table_dish[i] == Some(dish)) as usize;
        }
        n
    }

    /// Will the station at `i` let `item` pass straight through? Machines and counters pass
    /// only what they never use; chutes pass any dish nobody at their table is waiting for.
    fn passes(&self, i: usize, item: Item) -> bool {
        match self.cells[i] {
            KCell::Machine(m, _) => m.product(item).is_none(),
            KCell::Counter(dish, _) => !dish.recipe().iter().any(|&(p, _)| p == item),
            KCell::Chute => true,
            _ => false,
        }
    }

    /// Can `item`, travelling in direction `d`, enter kitchen cell (x, y) right now? With
    /// `vacated`, pretend the cell's moving content has already left (for turning full loops).
    fn accepts_ex(&self, x: i32, y: i32, item: Item, d: Dir, vacated: bool) -> bool {
        if !inside(x, y) {
            return false;
        }
        let i = idx(x, y);
        match self.cells[i] {
            KCell::Belt(bd) => (vacated || self.slot[i].is_none()) && bd != d.opposite(),
            KCell::Machine(_, od) | KCell::Counter(_, od) => {
                d != od.opposite() && (self.can_use(i, item) || (self.passes(i, item) && (vacated || self.pass[i].is_none())))
            }
            KCell::Chute => self.can_use(i, item) || vacated || self.pass[i].is_none(),
            _ => false,
        }
    }

    pub fn accepts(&self, x: i32, y: i32, item: Item, d: Dir) -> bool {
        self.accepts_ex(x, y, item, d, false)
    }

    /// Arms load stations only with what they'll use (so arms filter); onto belts, anything.
    pub fn arm_accepts(&self, x: i32, y: i32, item: Item, d: Dir) -> bool {
        if !inside(x, y) {
            return false;
        }
        let i = idx(x, y);
        match self.cells[i] {
            KCell::Belt(_) => self.accepts(x, y, item, d),
            KCell::Machine(_, od) | KCell::Counter(_, od) => d != od.opposite() && self.can_use(i, item),
            KCell::Chute => self.can_use(i, item),
            _ => false,
        }
    }

    /// A seated guest at the table on (x, y) waiting for `item` and not yet promised one.
    fn wanted_at(&self, x: i32, y: i32, item: Item) -> Option<usize> {
        self.guests.iter().position(|g| g.state == GState::Waiting && g.table == Some((x, y)) && g.order == item && !g.claimed)
    }

    /// Moves `item` (travelling `d`) into (x, y), already checked with `accepts`.
    fn put(&mut self, x: i32, y: i32, item: Item, d: Dir, from: (i32, i32)) {
        let i = idx(x, y);
        self.slot_from[i] = Some(from);
        if self.can_use(i, item) {
            match self.cells[i] {
                KCell::Counter(..) => self.parts[i][item.index()] += 1,
                KCell::Chute => {
                    if let Some(g) = self.wanted_at(x, y, item) {
                        self.guests[g].claimed = true;
                    }
                    self.slot[i] = Some(item);
                }
                _ => {
                    self.slot[i] = Some(item);
                    self.timer[i] = 0;
                }
            }
            return;
        }
        match self.cells[i] {
            KCell::Belt(_) => self.slot[i] = Some(item),
            _ => self.pass[i] = Some((item, d)),
        }
    }

    /// What wants to move out of cell `i` this beat: (item, direction, is it the pass slot).
    fn mover(&self, i: usize) -> Option<(Item, Dir, bool)> {
        match self.cells[i] {
            KCell::Belt(d) => self.slot[i].map(|it| (it, d, false)),
            KCell::Machine(..) | KCell::Counter(..) | KCell::Chute => self.pass[i].map(|(it, d)| (it, d, true)),
            _ => None,
        }
    }

    fn clear_mover(&mut self, i: usize, is_pass: bool) {
        if is_pass {
            self.pass[i] = None;
        } else {
            self.slot[i] = None;
        }
    }

    fn step_kitchen(&mut self) {
        self.slot_from = [None; 64];
        // Machines cook, counters plate, hatches refill, and passing items get used if a
        // station now wants them.
        for i in 0..64 {
            if let Some((item, d)) = self.pass[i] {
                if self.can_use(i, item) {
                    self.pass[i] = None;
                    let (x, y) = (i as i32 % N, i as i32 / N);
                    let c = cell_center(Board::Kitchen, x, y);
                    self.put(x, y, item, d, c);
                }
            }
            match self.cells[i] {
                KCell::Machine(m, _) => {
                    if let Some(input) = self.slot[i] {
                        self.timer[i] = (self.timer[i] + 1).min(m.beats());
                        if self.timer[i] >= m.beats() && self.out[i].is_none() {
                            self.out[i] = m.product(input);
                            self.slot[i] = None;
                            self.timer[i] = 0;
                            self.events.push(if m == Machine::Slicer { Ev::Slice } else { Ev::Bake });
                        }
                    }
                }
                KCell::Counter(dish, _) => {
                    let ready = dish.recipe().iter().all(|&(p, n)| self.parts[i][p.index()] >= n);
                    if ready && self.out[i].is_none() && self.open_tickets(dish) > self.in_flight(dish) {
                        for &(p, n) in dish.recipe() {
                            self.parts[i][p.index()] -= n;
                        }
                        self.out[i] = Some(dish);
                        self.events.push(Ev::Plate);
                    }
                }
                KCell::Hatch { period, .. } => self.timer[i] = (self.timer[i] + 1).min(period),
                _ => {}
            }
        }

        // Things move, downstream first: repeat until nothing more can move this beat.
        let mut moved = [false; 64];
        let mut out_moved = [false; 64];
        loop {
            let mut progress = false;
            for i in 0..64 {
                let (x, y) = (i as i32 % N, i as i32 / N);
                let here = cell_center(Board::Kitchen, x, y);
                if !moved[i] {
                    if let Some((item, d, is_pass)) = self.mover(i) {
                        let (tx, ty) = (x + d.dx(), y + d.dy());
                        if self.accepts(tx, ty, item, d) {
                            self.clear_mover(i, is_pass);
                            self.put(tx, ty, item, d, here);
                            moved[idx(tx, ty)] = true;
                            progress = true;
                        }
                    }
                }
                match self.cells[i] {
                    KCell::Machine(_, d) | KCell::Counter(_, d) if !out_moved[i] => {
                        if let Some(item) = self.out[i] {
                            let (tx, ty) = (x + d.dx(), y + d.dy());
                            if self.accepts(tx, ty, item, d) {
                                self.out[i] = None;
                                out_moved[i] = true;
                                self.put(tx, ty, item, d, (here.0 + d.dx() * 6, here.1 + d.dy() * 6));
                                moved[idx(tx, ty)] = true;
                                progress = true;
                            }
                        }
                    }
                    KCell::Hatch { item, dir, period } if self.timer[i] >= period => {
                        let (tx, ty) = (x + dir.dx(), y + dir.dy());
                        if self.accepts(tx, ty, item, dir) {
                            self.timer[i] = 0;
                            self.put(tx, ty, item, dir, here);
                            moved[idx(tx, ty)] = true;
                            self.events.push(Ev::Hatch);
                            progress = true;
                        }
                    }
                    _ => {}
                }
            }
            if !progress {
                break;
            }
        }
        self.turn_full_loops(&mut moved);

        // Arms: lift only what the tile in front can take right now.
        for i in 0..64 {
            let KCell::Arm(d) = self.cells[i] else { continue };
            let (x, y) = (i as i32 % N, i as i32 / N);
            let (bx, by) = (x - d.dx(), y - d.dy());
            let (fx, fy) = (x + d.dx(), y + d.dy());
            match self.arm[i] {
                0 => {
                    if let Some(item) = self.source_item(bx, by) {
                        if self.arm_accepts(fx, fy, item, d) {
                            self.take(bx, by);
                            self.slot[i] = Some(item);
                            self.arm[i] = 1;
                            self.events.push(Ev::Arm);
                        }
                    }
                }
                1 => self.arm[i] = 2,
                2 => {
                    if let Some(item) = self.slot[i] {
                        if self.arm_accepts(fx, fy, item, d) {
                            self.slot[i] = None;
                            let c = cell_center(Board::Kitchen, x, y);
                            self.put(fx, fy, item, d, (c.0 + d.dx() * 8, c.1 + d.dy() * 8));
                            self.arm[i] = 3;
                        }
                    } else {
                        self.arm[i] = 3;
                    }
                }
                _ => self.arm[i] = 0,
            }
        }

        // Chutes send promised dishes up to their table.
        for i in 0..64 {
            if self.cells[i] == KCell::Chute {
                if let Some(item) = self.slot[i] {
                    if self.dining[i] == DCell::Table && self.table_dish[i].is_none() {
                        self.table_dish[i] = Some(item);
                        self.table_pop[i] = 4;
                        self.slot[i] = None;
                        self.events.push(Ev::ChuteUp);
                    }
                }
            }
        }
    }

    /// A loop of belts and stations packed so full that nothing can move still turns, one
    /// tile per beat, as long as every item in it wants to go to the next cell of the loop.
    fn turn_full_loops(&mut self, moved: &mut [bool; 64]) {
        let next_of = |s: &Sim, i: usize| -> Option<usize> {
            let (item, d, _) = s.mover(i)?;
            let (x, y) = (i as i32 % N + d.dx(), i as i32 / N + d.dy());
            if !inside(x, y) {
                return None;
            }
            let j = idx(x, y);
            (s.mover(j).is_some() && s.accepts_ex(x, y, item, d, true)).then_some(j)
        };
        for start in 0..64 {
            if moved[start] || self.mover(start).is_none() {
                continue;
            }
            let mut chain = vec![start];
            let mut at = start;
            let cycle = loop {
                match next_of(self, at) {
                    Some(j) if j == start => break true,
                    Some(j) if !chain.contains(&j) && !moved[j] && chain.len() < 64 => {
                        chain.push(j);
                        at = j;
                    }
                    _ => break false,
                }
            };
            if !cycle {
                continue;
            }
            let carried: Vec<(Item, Dir, bool)> = chain.iter().map(|&i| self.mover(i).unwrap()).collect();
            for (&i, &(_, _, is_pass)) in chain.iter().zip(&carried) {
                self.clear_mover(i, is_pass);
            }
            for (k, &i) in chain.iter().enumerate() {
                let (item, d, _) = carried[k];
                let j = chain[(k + 1) % chain.len()];
                let from = cell_center(Board::Kitchen, i as i32 % N, i as i32 / N);
                self.put(j as i32 % N, j as i32 / N, item, d, from);
                moved[j] = true;
            }
        }
    }

    /// The item an arm could lift from (x, y): a belt's item, a machine or counter's output,
    /// or a hatch's ready ingredient.
    fn source_item(&self, x: i32, y: i32) -> Option<Item> {
        if !inside(x, y) {
            return None;
        }
        let i = idx(x, y);
        match self.cells[i] {
            KCell::Belt(_) => self.slot[i],
            KCell::Machine(..) | KCell::Counter(..) => self.out[i],
            KCell::Hatch { item, period, .. } if self.timer[i] >= period => Some(item),
            _ => None,
        }
    }

    fn take(&mut self, x: i32, y: i32) {
        let i = idx(x, y);
        match self.cells[i] {
            KCell::Belt(_) => self.slot[i] = None,
            KCell::Machine(..) | KCell::Counter(..) => self.out[i] = None,
            KCell::Hatch { .. } => {
                self.timer[i] = 0;
                self.events.push(Ev::Hatch);
            }
            _ => {}
        }
    }

    // -----------------------------------------------------------------------------------------
    // Dining room

    fn walkable(&self, x: i32, y: i32) -> bool {
        inside(x, y) && matches!(self.dining[idx(x, y)], DCell::Floor | DCell::Door)
    }

    /// Shortest path from `start` to `goal` over open floor; `goal` itself may be a seat.
    fn path(&self, start: (i32, i32), goal: (i32, i32)) -> Option<VecDeque<(i32, i32)>> {
        let mut prev = [None; 64];
        let mut seen = [false; 64];
        let mut q = VecDeque::new();
        seen[idx(start.0, start.1)] = true;
        q.push_back(start);
        while let Some(p) = q.pop_front() {
            if p == goal {
                let mut out = VecDeque::new();
                let mut at = p;
                while at != start {
                    out.push_front(at);
                    at = prev[idx(at.0, at.1)].unwrap();
                }
                return Some(out);
            }
            for d in Dir::ALL {
                let n = (p.0 + d.dx(), p.1 + d.dy());
                if inside(n.0, n.1) && !seen[idx(n.0, n.1)] && (self.walkable(n.0, n.1) || n == goal) {
                    seen[idx(n.0, n.1)] = true;
                    prev[idx(n.0, n.1)] = Some(p);
                    q.push_back(n);
                }
            }
        }
        None
    }

    fn seat_taken(&self, s: (i32, i32)) -> bool {
        self.guests.iter().any(|g| g.seat == Some(s) && !matches!(g.state, GState::Leaving { .. } | GState::Gone))
    }

    /// Sends guest `k` to the nearest free seat, if there is one.
    fn find_seat(&mut self, k: usize) -> bool {
        let start = (self.guests[k].x, self.guests[k].y);
        let mut best: Option<(i32, (i32, i32), VecDeque<(i32, i32)>)> = None;
        for i in 0..64 {
            let DCell::Seat(_) = self.dining[i] else { continue };
            let s = (i as i32 % N, i as i32 / N);
            if self.seat_taken(s) {
                continue;
            }
            if let Some(p) = self.path(start, s) {
                let len = p.len() as i32;
                if best.as_ref().map_or(true, |b| len < b.0) {
                    best = Some((len, s, p));
                }
            }
        }
        let Some((_, s, p)) = best else { return false };
        let DCell::Seat(d) = self.dining[idx(s.0, s.1)] else { return false };
        let g = &mut self.guests[k];
        g.seat = Some(s);
        g.table = Some((s.0 + d.dx(), s.1 + d.dy()));
        g.path = p;
        g.state = GState::Walking;
        true
    }

    fn leave(&mut self, k: usize, happy: bool) {
        let start = (self.guests[k].x, self.guests[k].y);
        let mut p = self.path(start, self.door).unwrap_or_default();
        p.push_back((self.door.0 + self.door_out.dx(), self.door.1 + self.door_out.dy()));
        let g = &mut self.guests[k];
        g.path = p;
        g.claimed = false;
        g.state = GState::Leaving { happy };
    }

    fn step_dining(&mut self) {
        for i in 0..64 {
            self.table_pop[i] = self.table_pop[i].saturating_sub(1);
        }
        // Dishes on tables go to whoever there ordered them (a promised guest first).
        for i in 0..64 {
            let Some(dish) = self.table_dish[i] else { continue };
            let t = (i as i32 % N, i as i32 / N);
            let waiting = |g: &Guest| g.state == GState::Waiting && g.table == Some(t) && g.order == dish;
            let pick = self.guests.iter().position(|g| waiting(g) && g.claimed).or_else(|| self.guests.iter().position(waiting));
            if let Some(k) = pick {
                let g = &mut self.guests[k];
                let tip = (3 * g.patience as u32 / g.max_patience.max(1) as u32).min(2);
                g.paid = dish.price() + tip;
                g.state = GState::Eating;
                g.timer = self.service.eat_beats;
                g.claimed = false;
                let paid = g.paid;
                self.coins += paid;
                self.fed += 1;
                self.table_dish[i] = None;
                self.events.push(Ev::Served(paid));
            }
        }

        for k in 0..self.guests.len() {
            let beat = self.beat;
            let g = &mut self.guests[k];
            g.from = g.to;
            match g.state {
                GState::Walking | GState::Leaving { .. } => {
                    if let Some(next) = g.path.pop_front() {
                        g.x = next.0;
                        g.y = next.1;
                        g.to = cell_center(Board::Dining, next.0, next.1);
                    } else if g.state == GState::Walking {
                        g.state = GState::Waiting;
                    } else {
                        g.state = GState::Gone;
                    }
                }
                GState::Waiting => {
                    g.patience = g.patience.saturating_sub(1);
                    if g.patience == 0 {
                        self.hungry += 1;
                        self.events.push(Ev::Hungry);
                        self.leave(k, false);
                    }
                }
                GState::Queue => {
                    if beat % 2 == 0 {
                        g.patience = g.patience.saturating_sub(1);
                    }
                    if self.guests[k].patience == 0 {
                        self.hungry += 1;
                        self.events.push(Ev::Hungry);
                        self.leave(k, false);
                    } else {
                        self.find_seat(k);
                    }
                }
                GState::Eating => {
                    g.timer = g.timer.saturating_sub(1);
                    if g.timer == 0 {
                        self.leave(k, true);
                    }
                }
                GState::Gone => {}
            }
        }

        // New arrivals, ordering from tonight's menu so far.
        let arrivals: Vec<Species> = self.service.arrivals.iter().filter(|(b, _)| *b == self.beat).map(|(_, s)| *s).collect();
        for species in arrivals {
            let menu: Vec<Item> = self.service.menu.iter().filter(|(b, _)| *b <= self.beat).map(|(_, d)| *d).collect();
            let order = menu[(self.rand() % menu.len() as u32) as usize];
            let door = cell_center(Board::Dining, self.door.0, self.door.1);
            let outside = (door.0 + self.door_out.dx() * 12, door.1 + self.door_out.dy() * 12);
            self.guests.push(Guest {
                species,
                x: self.door.0,
                y: self.door.1,
                from: outside,
                to: door,
                state: GState::Queue,
                order,
                patience: self.service.patience,
                max_patience: self.service.patience,
                seat: None,
                table: None,
                claimed: false,
                timer: 0,
                path: VecDeque::new(),
                paid: 0,
            });
            self.arrived += 1;
            self.events.push(Ev::Arrive);
            let k = self.guests.len() - 1;
            self.find_seat(k);
        }
    }

    pub fn step(&mut self) {
        self.beat += 1;
        self.step_kitchen();
        self.step_dining();
    }
}

/// Kitchen layouts written as text, for tests, tuning and screenshots.
#[allow(dead_code)]
pub mod lab {
    use super::*;

    /// Builds a kitchen from 8 rows of 8 two-character tokens over the restaurant's fixed
    /// cells: `..` keep, `B>` belt, `A>` arm, `S>` slicer, `O>` oven, `T>` toast counter,
    /// `W>` bowl counter, `P>` bean-plate counter, `CH` chute (arrow = direction).
    pub fn kitchen_from(r: &Restaurant, rows: [&str; 8]) -> [KCell; 64] {
        let mut k = r.kitchen;
        for (y, row) in rows.iter().enumerate() {
            for (x, tok) in row.split_whitespace().enumerate() {
                let b = tok.as_bytes();
                let d = match b[1] {
                    b'^' => Dir::N,
                    b'>' => Dir::E,
                    b'v' => Dir::S,
                    _ => Dir::W,
                };
                let cell = match b[0] {
                    b'.' => continue,
                    b'B' => KCell::Belt(d),
                    b'A' => KCell::Arm(d),
                    b'S' => KCell::Machine(Machine::Slicer, d),
                    b'O' => KCell::Machine(Machine::Oven, d),
                    b'T' => KCell::Counter(Item::Toast, d),
                    b'W' => KCell::Counter(Item::Bowl, d),
                    b'P' => KCell::Counter(Item::BeanPlate, d),
                    b'C' => KCell::Chute,
                    _ => panic!("bad token {tok}"),
                };
                k[idx(x as i32, y as i32)] = cell;
            }
        }
        k
    }

    pub fn reference_kitchen(r: &Restaurant) -> [KCell; 64] {
        kitchen_from(r, REFERENCE)
    }

    /// Separate ingredient lines feed three counters; dishes ride a loop that passes under
    /// all three tables (G2, H5, F8).
    pub const REFERENCE: [&str; 8] = [
        ".. S> B> Bv .. .. .. ..",
        ".. .. .. T> B> B> CH Bv",
        ".. O> B> B^ B^ .. .. Bv",
        ".. .. .. Av B^ .. .. Bv",
        ".. B> B> W> B^ .. .. CH",
        ".. .. Av .. B^ .. .. Bv",
        ".. O> P> B> B^ .. .. Bv",
        ".. .. .. B^ B< CH B< B<",
    ];

    /// Plays a whole service; returns the finished sim.
    pub fn play(r: &Restaurant, k: &[KCell; 64]) -> Sim {
        let mut s = Sim::new(r, k);
        while !s.over() && s.beat < 5000 {
            s.step();
        }
        s
    }

    pub fn dump(s: &Sim) -> String {
        let mut out = String::new();
        for y in 0..N {
            for x in 0..N {
                let i = idx(x, y);
                let c = match s.cells[i] {
                    KCell::Floor => "..".to_string(),
                    KCell::Wall => "##".to_string(),
                    KCell::Hatch { item, .. } => format!("H{}", &item.name()[..1]),
                    KCell::Belt(_) => format!("b{}", s.slot[i].map_or(".".into(), |it| it.name()[..1].to_string())),
                    KCell::Arm(_) => format!("a{}", s.slot[i].map_or(".".into(), |it| it.name()[..1].to_string())),
                    KCell::Machine(..) => format!("m{}", s.out[i].map_or(".".into(), |it| it.name()[..1].to_string())),
                    KCell::Counter(..) => format!("c{}", s.out[i].map_or(".".into(), |it| it.name()[..1].to_string())),
                    KCell::Chute => format!("C{}", s.slot[i].map_or(".".into(), |it| it.name()[..1].to_string())),
                };
                out.push_str(&c);
                out.push(' ');
            }
            out.push('\n');
        }
        out
    }

}

#[cfg(test)]
pub mod tests {
    use super::lab::*;
    use super::*;

    #[test]
    fn belt_carries_hatch_items_and_backs_up() {
        let r = object25();
        let k = kitchen_from(&r, [
            ".. B> B> B> B> B> B> B>",
            ".. .. .. .. .. .. .. ..",
            ".. .. .. .. .. .. .. ..",
            ".. .. .. .. .. .. .. ..",
            ".. .. .. .. .. .. .. ..",
            ".. .. .. .. .. .. .. ..",
            ".. .. .. .. .. .. .. ..",
            ".. .. .. .. .. .. .. ..",
        ]);
        let mut s = Sim::new(&r, &k);
        for _ in 0..40 {
            s.step();
        }
        // The belt ends at the east wall, so berries pile up behind the last tile.
        let on_belt = (1..8).filter(|&x| s.slot[idx(x, 0)] == Some(Item::Berry)).count();
        assert_eq!(on_belt, 7);
    }

    #[test]
    fn a_counter_gathers_parts_from_a_merged_line() {
        let mut r = object25();
        r.service.arrivals.clear();
        let k = kitchen_from(&r, [
            ".. .. .. .. .. .. .. ..",
            ".. .. W> .. .. .. .. ..",
            ".. O> B^ .. .. .. .. ..",
            ".. .. B^ .. .. .. .. ..",
            ".. B> B^ .. .. .. .. ..",
            ".. .. .. .. .. .. .. ..",
            ".. .. .. .. .. .. .. ..",
            ".. .. .. .. .. .. .. ..",
        ]);
        let mut s = Sim::new(&r, &k);
        for _ in 0..40 {
            s.step();
        }
        // No guest has ordered yet, so the counter holds its parts ready and waits for a ticket.
        let c = idx(2, 1);
        assert_eq!(s.parts[c][Item::Bread.index()], 1, "{}", dump(&s));
        assert_eq!(s.parts[c][Item::Moss.index()], 1);
        assert_eq!(s.out[c], None);
    }

    #[test]
    fn arms_only_load_stations_with_what_they_use() {
        let r = object25();
        let rows = |m: &'static str| -> [&'static str; 8] {
            [".. B> B> B> .. .. .. ..", ".. Av .. .. .. .. .. ..", m, ".. .. .. .. .. .. .. ..", ".. .. .. .. .. .. .. ..", ".. .. .. .. .. .. .. ..", ".. .. .. .. .. .. .. ..", ".. .. .. .. .. .. .. .."]
        };
        // An oven doesn't use berries, so the arm never lifts one.
        let mut s = Sim::new(&r, &kitchen_from(&r, rows(".. O> .. .. .. .. .. ..")));
        for _ in 0..30 {
            s.step();
        }
        assert_eq!(s.slot[idx(1, 1)], None);
        // A slicer does.
        let mut s = Sim::new(&r, &kitchen_from(&r, rows(".. S> .. .. .. .. .. ..")));
        let mut sliced = false;
        for _ in 0..30 {
            s.step();
            sliced |= s.events.contains(&Ev::Slice);
        }
        assert!(sliced);
    }

    #[test]
    fn stations_let_what_they_cannot_use_pass_through() {
        let r = object25();
        // Berries run east through an oven (which can't use them) and on along the belt.
        let k = kitchen_from(&r, [
            ".. B> O> B> B> .. .. ..",
            ".. .. .. .. .. .. .. ..",
            ".. .. .. .. .. .. .. ..",
            ".. .. .. .. .. .. .. ..",
            ".. .. .. .. .. .. .. ..",
            ".. .. .. .. .. .. .. ..",
            ".. .. .. .. .. .. .. ..",
            ".. .. .. .. .. .. .. ..",
        ]);
        let mut s = Sim::new(&r, &k);
        for _ in 0..20 {
            s.step();
        }
        assert_eq!(s.slot[idx(4, 0)], Some(Item::Berry));
    }

    #[test]
    fn a_full_loop_keeps_turning() {
        let r = object25();
        // A 2x2 loop at E2..F3 filled by hand.
        let k = kitchen_from(&r, [
            ".. .. .. .. .. .. .. ..",
            ".. .. .. .. B> Bv .. ..",
            ".. .. .. .. B^ B< .. ..",
            ".. .. .. .. .. .. .. ..",
            ".. .. .. .. .. .. .. ..",
            ".. .. .. .. .. .. .. ..",
            ".. .. .. .. .. .. .. ..",
            ".. .. .. .. .. .. .. ..",
        ]);
        let mut s = Sim::new(&r, &k);
        let ring = [idx(4, 1), idx(5, 1), idx(5, 2), idx(4, 2)];
        let items = [Item::Toast, Item::Bowl, Item::BeanPlate, Item::Bread];
        for (i, it) in ring.iter().zip(items) {
            s.slot[*i] = Some(it);
        }
        s.step();
        assert_eq!(s.slot[idx(5, 1)], Some(Item::Toast), "everything moved one step round");
        assert_eq!(s.slot[idx(4, 1)], Some(Item::Bread));
    }

    #[test]
    #[ignore]
    fn reference_kitchen_report() {
        let r = object25();
        let k = reference_kitchen(&r);
        let mut s = Sim::new(&r, &k);
        for b in 0..5000 {
            if s.over() {
                break;
            }
            s.step();
            if b == 300 || b == 700 {
                println!("beat {b}: fed {} hungry {}\n{}", s.fed, s.hungry, dump(&s));
            }
        }
        println!("END fed {}/{} hungry {} coins {} stars {}", s.fed, s.total_guests(), s.hungry, s.coins, s.stars());
    }

    #[test]
    fn the_reference_kitchen_earns_three_stars() {
        let r = object25();
        let s = play(&r, &reference_kitchen(&r));
        assert_eq!(s.stars(), 3, "fed {}/{} hungry {}", s.fed, s.total_guests(), s.hungry);
    }

    #[test]
    fn counters_plate_only_against_open_tickets() {
        let r = object25();
        let mut s = Sim::new(&r, &reference_kitchen(&r));
        // Bean plates join the menu at beat 320; none may be plated before anyone orders one.
        for _ in 0..300 {
            s.step();
            assert_eq!(s.in_flight(Item::BeanPlate), 0);
        }
    }

    #[test]
    fn guests_seat_order_and_leave_hungry_without_a_kitchen() {
        let r = object25();
        let mut s = Sim::new(&r, &r.kitchen);
        while !s.over() {
            s.step();
            assert!(s.beat < 5000, "service never ended");
        }
        assert_eq!(s.fed, 0);
        assert_eq!(s.hungry, s.total_guests());
        assert_eq!(s.stars(), 0);
    }
}
