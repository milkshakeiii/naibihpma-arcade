//! Playing a match in step with another machine, and the lag that costs.
//!
//! A match is a sim that only commands drive (`Cmd`), each applied on a tick, and the same
//! commands on the same ticks always give the same match. So two machines playing one match send
//! each other nothing but their commands: no positions, no state. The price is that a command
//! can't take effect the moment it's given, because the other machine has to hear of it first.
//! Every command therefore waits `delay` ticks, and a machine plays a tick only once it holds
//! both sides' commands for it.
//!
//! - **Turns.** One side's commands for one tick are a turn. A side seals its turns in order, one
//!   for every tick (most are empty), and sends each as it's sealed. Sealed means final.
//! - **Stepping.** A tick is played when both sides' turns for it are in hand: blue's commands
//!   first, then red's, on both machines. A machine short of the other's turn waits for it.
//! - **The check.** Once a second each machine hashes its whole match (`Arena::checksum`) and
//!   sends the number. Two numbers for one tick that differ mean the machines have parted.
//! - **Alone.** With no other machine the same code runs with only your side's turns, and `delay`
//!   is a dial: the game as it would feel over a network of that lag, with no network.
//!
//! What's sent is lines of text (`Msg::to_text`). Nothing that arrives is trusted: a line that
//! isn't the protocol ends the match, and a command is only ever applied for the side whose turn
//! it came in.

use super::design::Design;
use super::sim::{Arena, Cmd};
use std::collections::BTreeMap;
use std::io::{ErrorKind, Read, Write};
use std::net::{TcpListener, TcpStream, ToSocketAddrs};
use std::sync::mpsc::{channel, Receiver, TryRecvError};
use std::time::Duration;

/// Two games talk only if this matches. It goes up whenever what's sent changes, and whenever
/// what a match's state holds does: the two machines compare a hash of it, so two versions with
/// different state would fall out of step at the first check. (3: a bot remembers one more
/// thing.)
pub const PROTO: u32 = 3;
/// The port a host listens on unless told another.
pub const PORT: u16 = 7777;
/// Ticks between hash checks: one a second.
pub const HASH_EVERY: u32 = 15;
/// The delay a network match starts with: two ticks, 133 ms.
pub const NET_DELAY: u32 = 2;
/// The settings of the lag dial, in ticks.
pub const DELAYS: [u32; 7] = [0, 1, 2, 3, 4, 6, 8];
/// Extra ticks a machine that has fallen behind plays in one frame, to catch up.
const CATCH_UP: u32 = 2;
/// The most commands one turn may carry, and the most turns a side may run ahead: more than any
/// play could need, and a limit on what a broken or hostile other end can make this one hold.
const TURN_MAX: usize = 64;
const AHEAD_MAX: u32 = 15 * 20;

/// What one machine sends another.
#[derive(Clone, PartialEq, Eq, Debug)]
pub enum Msg {
    /// The first thing each sends: its version, the match's seed (the host's counts), its deck.
    Hello { proto: u32, seed: u64, deck: Vec<String> },
    /// "These are all my commands for tick `turn`", sent while the sender was at `tick`.
    Turn { turn: u32, tick: u32, cmds: Vec<Cmd> },
    /// "After tick `tick` my match hashed to this."
    Hash { tick: u32, hash: u64 },
    /// "I've left."
    Bye,
}

impl Msg {
    /// The message as lines of text, each ending in a newline.
    pub fn to_text(&self) -> String {
        match self {
            Msg::Hello { proto, seed, deck } => {
                let mut s = format!("HELLO {proto} {seed} {}\n", deck.len());
                for d in deck {
                    s += &format!("D {}\n", d.replace(['\n', '\r'], " "));
                }
                s
            }
            Msg::Turn { turn, tick, cmds } if cmds.is_empty() => format!("T {turn} {tick}\n"),
            Msg::Turn { turn, tick, cmds } => format!("T {turn} {tick} {}\n", cmds.iter().map(Cmd::to_wire).collect::<Vec<_>>().join("|")),
            Msg::Hash { tick, hash } => format!("H {tick} {hash:016x}\n"),
            Msg::Bye => "B\n".into(),
        }
    }
}

/// Turns lines of text back into messages. A hello runs over several lines, so it keeps what it
/// has read of one.
#[derive(Default)]
pub struct Reader {
    hello: Option<(u32, u64, usize, Vec<String>)>,
}

impl Reader {
    /// One line in; a message out when one is complete. An error for anything that isn't the
    /// protocol: the match can't go on past a line it can't read.
    pub fn line(&mut self, line: &str) -> Result<Option<Msg>, String> {
        let line = line.trim_end_matches('\r');
        let bad = || "THE OTHER GAME SENT SOMETHING THIS ONE CAN'T READ".to_string();
        if let Some((proto, seed, n, mut deck)) = self.hello.take() {
            deck.push(line.strip_prefix("D ").ok_or_else(bad)?.to_string());
            if deck.len() == n {
                return Ok(Some(Msg::Hello { proto, seed, deck }));
            }
            self.hello = Some((proto, seed, n, deck));
            return Ok(None);
        }
        let (kind, rest) = line.split_once(' ').unwrap_or((line, ""));
        let mut words = rest.splitn(3, ' ');
        match kind {
            "HELLO" => {
                let proto = words.next().and_then(|w| w.parse().ok()).ok_or_else(bad)?;
                let seed = words.next().and_then(|w| w.parse().ok()).ok_or_else(bad)?;
                let n: usize = words.next().and_then(|w| w.parse().ok()).filter(|&n| n <= 16).ok_or_else(bad)?;
                if n == 0 {
                    return Ok(Some(Msg::Hello { proto, seed, deck: Vec::new() }));
                }
                self.hello = Some((proto, seed, n, Vec::new()));
                Ok(None)
            }
            "T" => {
                let turn = words.next().and_then(|w| w.parse().ok()).ok_or_else(bad)?;
                let tick = words.next().and_then(|w| w.parse().ok()).ok_or_else(bad)?;
                let cmds = match words.next() {
                    Some(list) => list.split('|').map(Cmd::from_wire).collect::<Option<Vec<Cmd>>>().filter(|c| c.len() <= TURN_MAX).ok_or_else(bad)?,
                    None => Vec::new(),
                };
                Ok(Some(Msg::Turn { turn, tick, cmds }))
            }
            "H" => {
                let tick = words.next().and_then(|w| w.parse().ok()).ok_or_else(bad)?;
                let hash = words.next().and_then(|w| u64::from_str_radix(w, 16).ok()).ok_or_else(bad)?;
                Ok(Some(Msg::Hash { tick, hash }))
            }
            "B" if rest.is_empty() => Ok(Some(Msg::Bye)),
            _ => Err(bad()),
        }
    }
}

/// Something messages go out on and come in from, in the order they were sent.
pub trait Wire {
    fn send(&mut self, m: &Msg);
    /// What has arrived since the last call. An error once the other end has gone, or has sent
    /// something that isn't the protocol.
    fn recv(&mut self) -> Result<Vec<Msg>, String>;
}

/// A TCP connection that never blocks: what can't be written yet waits, and reading takes what's
/// there.
pub struct Tcp {
    stream: TcpStream,
    inbuf: Vec<u8>,
    out: Vec<u8>,
    reader: Reader,
    dead: Option<String>,
}

impl Tcp {
    pub fn new(stream: TcpStream) -> Result<Tcp, String> {
        stream.set_nonblocking(true).and_then(|_| stream.set_nodelay(true)).map_err(|e| io_text("CAN'T SET UP THE CONNECTION", &e))?;
        Ok(Tcp { stream, inbuf: Vec::new(), out: Vec::new(), reader: Reader::default(), dead: None })
    }

    fn flush(&mut self) {
        while !self.out.is_empty() && self.dead.is_none() {
            match self.stream.write(&self.out) {
                Ok(0) => self.dead = Some(GONE.into()),
                Ok(n) => {
                    self.out.drain(..n);
                }
                Err(e) if e.kind() == ErrorKind::WouldBlock => break,
                Err(e) if e.kind() == ErrorKind::Interrupted => {}
                Err(_) => self.dead = Some(GONE.into()),
            }
        }
    }
}

const GONE: &str = "THE OTHER PLAYER HAS GONE";

impl Wire for Tcp {
    fn send(&mut self, m: &Msg) {
        self.out.extend_from_slice(m.to_text().as_bytes());
        self.flush();
    }

    fn recv(&mut self) -> Result<Vec<Msg>, String> {
        self.flush();
        let mut buf = [0u8; 4096];
        while self.dead.is_none() {
            match self.stream.read(&mut buf) {
                Ok(0) => self.dead = Some(GONE.into()),
                Ok(n) => self.inbuf.extend_from_slice(&buf[..n]),
                Err(e) if e.kind() == ErrorKind::WouldBlock => break,
                Err(e) if e.kind() == ErrorKind::Interrupted => {}
                Err(_) => self.dead = Some(GONE.into()),
            }
        }
        let mut msgs = Vec::new();
        while let Some(end) = self.inbuf.iter().position(|&b| b == b'\n') {
            let line: Vec<u8> = self.inbuf.drain(..=end).collect();
            match std::str::from_utf8(&line[..end]).map_err(|_| "THE OTHER GAME SENT SOMETHING THIS ONE CAN'T READ".to_string()).and_then(|l| self.reader.line(l)) {
                Ok(Some(m)) => msgs.push(m),
                Ok(None) => {}
                Err(e) => {
                    self.dead = Some(e);
                    self.inbuf.clear();
                }
            }
        }
        // A line that never ends isn't one of ours.
        if self.inbuf.len() > 1 << 16 {
            self.dead = Some("THE OTHER GAME SENT SOMETHING THIS ONE CAN'T READ".into());
            self.inbuf.clear();
        }
        match &self.dead {
            Some(why) if msgs.is_empty() => Err(why.clone()),
            _ => Ok(msgs),
        }
    }
}

/// An error from the system in the game's own letters.
fn io_text(what: &str, e: &std::io::Error) -> String {
    format!("{what}: {}", format!("{:?}", e.kind()).chars().flat_map(|c| if c.is_uppercase() { vec![' ', c] } else { vec![c.to_ascii_uppercase()] }).collect::<String>().trim())
}

/// The other machine, as this one knows it.
pub struct Net {
    wire: Box<dyn Wire>,
    /// Their turns that have arrived and aren't played yet, and the last one to arrive.
    theirs: BTreeMap<u32, Vec<Cmd>>,
    heard: u32,
    /// The tick they were at when they last sent a turn.
    their_tick: u32,
    my_hashes: BTreeMap<u32, u64>,
    their_hashes: BTreeMap<u32, u64>,
    started: bool,
    /// The last tick both machines hashed the same.
    pub agreed: u32,
    /// The first tick their hash and this machine's differed: from there on, two different games.
    pub parted: Option<u32>,
    /// Why the other machine can't be played with any more.
    pub gone: Option<String>,
    /// Frames in a row spent waiting for their turn.
    pub waiting: u32,
}

/// How a match's commands get to it: after the lag, and in step with another machine if there
/// is one.
pub struct Link {
    /// The side this machine plays.
    pub me: usize,
    /// Ticks a command waits between being given and being applied. Alone, it's the lag dial.
    pub delay: u32,
    /// My commands by the turn they're for. Turns up to `sealed` are final.
    mine: BTreeMap<u32, Vec<Cmd>>,
    sealed: u32,
    steer: (i32, i32),
    pub net: Option<Net>,
}

impl Link {
    /// A match on this machine alone.
    pub fn solo() -> Link {
        Link { me: 0, delay: 0, mine: BTreeMap::new(), sealed: 0, steer: (0, 0), net: None }
    }

    /// A match in step with another machine, which plays the other side. `backlog` is whatever
    /// arrived from it behind its hello.
    pub fn net(me: usize, wire: Box<dyn Wire>, backlog: Vec<Msg>) -> Link {
        let mut net = Net { wire, theirs: BTreeMap::new(), heard: 0, their_tick: 0, my_hashes: BTreeMap::new(), their_hashes: BTreeMap::new(), started: false, agreed: 0, parted: None, gone: None, waiting: 0 };
        for m in backlog {
            net.take(m);
        }
        Link { me, delay: NET_DELAY, mine: BTreeMap::new(), sealed: 0, steer: (0, 0), net: Some(net) }
    }

    /// Gives a command. It's applied `delay` ticks after the next tick, or on the first turn
    /// not yet sealed if the dial has just been turned down.
    pub fn give(&mut self, arena: &Arena, c: Cmd) {
        let turn = (arena.tick + 1 + self.delay).max(self.sealed + 1);
        self.mine.entry(turn).or_default().push(c);
    }

    /// The design of the last job this side has asked its king to print that's still waiting
    /// out the lag: it isn't in the queue to be seen yet, but it's the last job all the same.
    pub fn print_waiting(&self) -> Option<usize> {
        self.mine.values().flatten().filter_map(|c| match *c {
            Cmd::Print { slot, target: None, .. } => Some(slot),
            _ => None,
        }).last()
    }

    /// The king's keys as they are now. Only a change is a command.
    pub fn steer(&mut self, arena: &Arena, dx: i32, dy: i32) {
        if (dx, dy) != self.steer {
            self.steer = (dx, dy);
            self.give(arena, Cmd::Steer { team: self.me, dx, dy });
        }
    }

    /// Turns the lag dial to its next setting. A network match can't run at none: a command
    /// has to reach the other machine before the tick it's for.
    pub fn next_delay(&mut self) {
        let at = DELAYS.iter().position(|&d| d == self.delay).map_or(0, |k| (k + 1) % DELAYS.len());
        self.delay = DELAYS[at];
        if self.net.is_some() && self.delay == 0 {
            self.delay = DELAYS[1];
        }
    }

    /// Seals my turns as far as the delay reaches, sending each to the other machine.
    fn seal(&mut self, tick: u32) {
        let Some(net) = self.net.as_mut() else { return };
        while self.sealed < tick + 1 + self.delay {
            self.sealed += 1;
            let cmds = self.mine.get(&self.sealed).cloned().unwrap_or_default();
            net.wire.send(&Msg::Turn { turn: self.sealed, tick, cmds });
        }
    }

    /// One frame: plays up to `want` ticks of the match, each with the commands due on it. In
    /// step with another machine it plays a tick only when their turn for it has arrived, and
    /// a couple more than asked if it has fallen behind them. Returns how many it played.
    pub fn advance(&mut self, arena: &mut Arena, want: u32) -> u32 {
        if let Some(net) = self.net.as_mut() {
            net.listen();
            if !net.started {
                net.started = true;
                net.hashed(arena);
            }
        }
        let mut played = 0;
        while arena.over.is_none() {
            let turn = arena.tick + 1;
            match self.net.as_ref() {
                Some(net) => {
                    let behind = net.their_tick > arena.tick + 1;
                    if net.gone.is_some() || played >= want + if behind { CATCH_UP } else { 0 } {
                        break;
                    }
                    self.seal(arena.tick);
                    if self.net.as_ref().is_some_and(|n| n.heard < turn) {
                        break;
                    }
                }
                None => {
                    if played >= want {
                        break;
                    }
                    self.sealed = turn;
                }
            }
            // Blue's commands, then red's, whichever of them is mine. Only a side's own commands
            // count in its turn, on both machines alike.
            for team in 0..2 {
                let cmds = if team == self.me { self.mine.remove(&turn) } else { self.net.as_mut().and_then(|n| n.theirs.remove(&turn)) };
                for c in cmds.unwrap_or_default().into_iter().filter(|c| c.team() == team) {
                    arena.command(c);
                }
            }
            arena.step();
            played += 1;
            if let Some(net) = self.net.as_mut() {
                if arena.tick % HASH_EVERY == 0 || arena.over.is_some() {
                    net.hashed(arena);
                }
            }
        }
        if let Some(net) = self.net.as_mut() {
            net.waiting = if want > 0 && played == 0 && arena.over.is_none() && net.gone.is_none() { net.waiting + 1 } else { 0 };
        }
        played
    }

    /// Tells the other machine this one is leaving.
    pub fn bye(&mut self) {
        if let Some(net) = self.net.as_mut() {
            net.wire.send(&Msg::Bye);
        }
    }
}

impl Net {
    /// Takes in whatever has arrived.
    fn listen(&mut self) {
        if self.gone.is_some() {
            return;
        }
        match self.wire.recv() {
            Ok(msgs) => {
                for m in msgs {
                    self.take(m);
                }
            }
            Err(why) => self.gone = Some(why),
        }
    }

    fn take(&mut self, m: Msg) {
        match m {
            // Turns come once each and in order, and never further ahead than a lag could put them.
            Msg::Turn { turn, tick, cmds } if turn == self.heard + 1 && turn <= tick.saturating_add(AHEAD_MAX) => {
                self.heard = turn;
                self.their_tick = self.their_tick.max(tick);
                if !cmds.is_empty() {
                    self.theirs.insert(turn, cmds);
                }
            }
            Msg::Hash { tick, hash } => {
                self.their_hashes.insert(tick, hash);
                self.compare();
            }
            Msg::Bye => self.gone = Some(GONE.into()),
            Msg::Turn { .. } | Msg::Hello { .. } => self.gone = Some("THE OTHER GAME SENT ITS TURNS OUT OF ORDER".into()),
        }
    }

    /// Hashes the match as it stands and sends the number.
    fn hashed(&mut self, arena: &Arena) {
        let hash = arena.checksum();
        self.my_hashes.insert(arena.tick, hash);
        self.wire.send(&Msg::Hash { tick: arena.tick, hash });
        self.compare();
    }

    /// Sets every tick both machines have hashed side by side.
    fn compare(&mut self) {
        let both: Vec<u32> = self.my_hashes.keys().copied().filter(|t| self.their_hashes.contains_key(t)).collect();
        for t in both {
            if self.my_hashes.remove(&t) == self.their_hashes.remove(&t) {
                self.agreed = self.agreed.max(t);
            } else if self.parted.map_or(true, |p| t < p) {
                self.parted = Some(t);
            }
        }
    }
}

/// Finding the other machine, before there's a match.
pub enum Lobby {
    /// Typing the address to join.
    Address(String),
    /// Listening for the other player.
    Hosting { listener: TcpListener, port: u16 },
    /// Connecting to a host.
    Joining { rx: Receiver<Result<TcpStream, String>>, addr: String },
    /// Connected: hello sent, theirs awaited.
    Meeting { wire: Tcp, host: bool },
    Failed(String),
}

/// Both sides have said hello: all a network match needs to start.
pub struct Met {
    /// Whether this machine is the host, who plays blue.
    pub host: bool,
    pub seed: u64,
    /// The other player's deck.
    pub deck: Vec<Design>,
    pub wire: Tcp,
    pub backlog: Vec<Msg>,
}

impl Lobby {
    /// Listens on every address this machine has.
    pub fn host(port: u16) -> Lobby {
        Lobby::host_on("0.0.0.0", port)
    }

    /// Listens on one address: `127.0.0.1` hears only this machine.
    pub fn host_on(addr: &str, port: u16) -> Lobby {
        match TcpListener::bind((addr, port)).and_then(|l| l.set_nonblocking(true).map(|_| l)) {
            Ok(listener) => {
                // Asked for port 0, the system picks one: say which.
                let port = listener.local_addr().map_or(port, |a| a.port());
                Lobby::Hosting { listener, port }
            }
            Err(e) => Lobby::Failed(io_text(&format!("CAN'T LISTEN ON PORT {port}"), &e)),
        }
    }

    /// Connects to a host at `addr` (a name or number, with `:PORT` if it isn't the usual one).
    /// The connecting is done on a thread of its own, so the window never waits on it.
    pub fn join(addr: &str) -> Lobby {
        let addr = addr.trim().to_string();
        let full = if addr.contains(':') { addr.clone() } else { format!("{addr}:{PORT}") };
        let (tx, rx) = channel();
        std::thread::spawn(move || {
            let found = full.to_socket_addrs().map_err(|e| io_text("NO SUCH ADDRESS", &e)).and_then(|mut a| a.next().ok_or_else(|| "NO SUCH ADDRESS".to_string()));
            let _ = tx.send(found.and_then(|a| TcpStream::connect_timeout(&a, Duration::from_secs(5)).map_err(|e| io_text("CAN'T CONNECT", &e))));
        });
        Lobby::Joining { rx, addr }
    }

    /// One frame's work. `seed` and `deck` are what this machine would bring to the match.
    /// Gives back everything a match needs once both sides have said hello.
    pub fn poll(&mut self, seed: u64, deck: &[Design]) -> Option<Met> {
        let hello = |seed| Msg::Hello { proto: PROTO, seed, deck: deck.iter().map(|d| d.to_line()).collect() };
        let connected = match self {
            Lobby::Hosting { listener, .. } => match listener.accept() {
                Ok((stream, _)) => Some((stream, true)),
                Err(e) if e.kind() == ErrorKind::WouldBlock => None,
                Err(e) => {
                    *self = Lobby::Failed(io_text("CAN'T TAKE THE CONNECTION", &e));
                    None
                }
            },
            Lobby::Joining { rx, .. } => match rx.try_recv() {
                Ok(Ok(stream)) => Some((stream, false)),
                Ok(Err(why)) => {
                    *self = Lobby::Failed(why);
                    None
                }
                Err(TryRecvError::Empty) => None,
                Err(TryRecvError::Disconnected) => {
                    *self = Lobby::Failed("CAN'T CONNECT".into());
                    None
                }
            },
            _ => None,
        };
        if let Some((stream, host)) = connected {
            *self = match Tcp::new(stream) {
                Ok(mut wire) => {
                    wire.send(&hello(if host { seed } else { 0 }));
                    Lobby::Meeting { wire, host }
                }
                Err(why) => Lobby::Failed(why),
            };
            return None;
        }
        let Lobby::Meeting { wire, host } = self else { return None };
        let host = *host;
        let mut msgs = match wire.recv() {
            Ok(m) if m.is_empty() => return None,
            Ok(m) => m,
            Err(why) => {
                *self = Lobby::Failed(why);
                return None;
            }
        };
        let fail = match msgs.remove(0) {
            Msg::Hello { proto, .. } if proto != PROTO => "THE OTHER GAME IS A DIFFERENT VERSION OF THIS ONE".to_string(),
            Msg::Hello { seed: theirs, deck: lines, .. } => match lines.iter().map(|l| Design::from_line(l)).collect::<Option<Vec<Design>>>() {
                Some(deck) if !deck.is_empty() => {
                    let Lobby::Meeting { wire, .. } = std::mem::replace(self, Lobby::Failed(String::new())) else { unreachable!() };
                    return Some(Met { host, seed: if host { seed } else { theirs }, deck, wire, backlog: msgs });
                }
                _ => "THE OTHER PLAYER'S DECK CAN'T BE READ".to_string(),
            },
            _ => "THE OTHER GAME DIDN'T SAY HELLO".to_string(),
        };
        *self = Lobby::Failed(fail);
        None
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;
    use crate::pg::design::{starter_deck, Order};
    use crate::rng::Rng;
    use std::cell::{Cell, RefCell};
    use std::collections::VecDeque;
    use std::rc::Rc;

    type Line = Rc<RefCell<VecDeque<(u32, String)>>>;

    /// One end of a pretend network: what's sent arrives `lag` frames of the shared clock later,
    /// in order, as the text a real wire would carry.
    pub struct Pipe {
        out: Line,
        inq: Line,
        clock: Rc<Cell<u32>>,
        pub lag: Rc<Cell<u32>>,
        reader: Reader,
    }

    /// Two ends joined to each other, their shared clock, and the lag (in frames) each way.
    pub fn pipes() -> (Pipe, Pipe, Rc<Cell<u32>>, Rc<Cell<u32>>) {
        let (ab, ba): (Line, Line) = Default::default();
        let (clock, lag) = (Rc::new(Cell::new(0)), Rc::new(Cell::new(0)));
        let end = |out: &Line, inq: &Line| Pipe { out: out.clone(), inq: inq.clone(), clock: clock.clone(), lag: lag.clone(), reader: Reader::default() };
        (end(&ab, &ba), end(&ba, &ab), clock.clone(), lag.clone())
    }

    impl Wire for Pipe {
        fn send(&mut self, m: &Msg) {
            // Never sooner than what was sent before it: a wire keeps its order.
            let due = (self.clock.get() + self.lag.get()).max(self.out.borrow().back().map_or(0, |b| b.0));
            self.out.borrow_mut().push_back((due, m.to_text()));
        }
        fn recv(&mut self) -> Result<Vec<Msg>, String> {
            let mut msgs = Vec::new();
            while self.inq.borrow().front().is_some_and(|f| f.0 <= self.clock.get()) {
                let (_, text) = self.inq.borrow_mut().pop_front().unwrap();
                for line in text.lines() {
                    if let Some(m) = self.reader.line(line)? {
                        msgs.push(m);
                    }
                }
            }
            Ok(msgs)
        }
    }

    /// Two machines set up to play one match in step: blue's first.
    pub struct Table {
        pub arena: [Arena; 2],
        pub link: [Link; 2],
        pub clock: Rc<Cell<u32>>,
        pub lag: Rc<Cell<u32>>,
    }

    impl Table {
        pub fn new(seed: u64) -> Table {
            let (a, b, clock, lag) = pipes();
            let arena = [0, 1].map(|_| Arena::for_match(seed, [starter_deck(), starter_deck()], [false, false]));
            Table { arena, link: [Link::net(0, Box::new(a), Vec::new()), Link::net(1, Box::new(b), Vec::new())], clock, lag }
        }

        /// One frame on both machines. `who` says which of them get one: a machine left out has
        /// hung for a frame.
        pub fn frame(&mut self, who: [bool; 2]) {
            self.clock.set(self.clock.get() + 1);
            for k in 0..2 {
                if who[k] {
                    self.link[k].advance(&mut self.arena[k], 1);
                }
            }
        }

        fn net(&self, k: usize) -> &Net {
            self.link[k].net.as_ref().unwrap()
        }

        /// Fails if the two matches differ at a tick both have reached, by the test's own
        /// reckoning, or if either machine's check says so.
        fn in_step(&self, seen: &mut BTreeMap<u32, u64>) {
            for k in 0..2 {
                assert_eq!(self.net(k).parted, None, "machine {k} says they parted");
                assert_eq!(self.net(k).gone, None, "machine {k} lost the other");
                let sum = self.arena[k].checksum();
                assert_eq!(*seen.entry(self.arena[k].tick).or_insert(sum), sum, "the two matches differ at tick {}", self.arena[k].tick);
            }
        }
    }

    /// What a restless player might do on one tick: fly about, queue ships, plant a factory,
    /// switch orders, set jobs repeating and drop them.
    fn fidget(rng: &mut Rng, a: &Arena, team: usize) -> Vec<Cmd> {
        let e = a.eco.as_ref().unwrap();
        let mut out = Vec::new();
        if rng.chance(10) {
            out.push(Cmd::Steer { team, dx: rng.range(-1, 1), dy: rng.range(-1, 1) });
        }
        if rng.chance(4) && e.queue[team].len() < 4 {
            out.push(Cmd::Print { team, slot: rng.below(4) as usize, target: None });
        }
        if rng.chance(1) {
            let k = &a.ships[e.king[team]];
            if let Some(rock) = a.rock_near((k.x, k.y), 150) {
                out.push(Cmd::Print { team, slot: 4, target: Some(rock) });
            }
        }
        if rng.chance(2) {
            out.push(Cmd::Order { team, slot: rng.below(4) as usize, order: [Order::Hunt, Order::Guard, Order::Escort, Order::Sab][rng.below(4) as usize] });
        }
        if rng.chance(1) {
            out.push(Cmd::Repeat { team, slot: rng.below(4) as usize, at: if rng.chance(50) { None } else { Some(rng.below(3) as usize) } });
        }
        if rng.chance(1) {
            out.push(Cmd::Cancel { team, at: 0, slot: e.queue[team].first().map_or(0, |j| j.slot) });
        }
        out
    }

    /// Three minutes of two restless players on one machine, and every command that was
    /// applied: something for two machines to play back at each other.
    fn script(seed: u64) -> (Arena, Vec<(u32, Cmd)>) {
        let mut a = Arena::for_match(seed, [starter_deck(), starter_deck()], [false, false]);
        let mut rng = Rng::new(seed);
        while a.tick < 15 * 180 {
            for team in 0..2 {
                for c in fidget(&mut rng, &a, team) {
                    a.command(c);
                }
            }
            a.step();
        }
        let log = a.eco.as_ref().unwrap().log.clone();
        assert!(log.len() >= 400 && a.ships.len() >= 10, "a thin script: {} commands, {} ships", log.len(), a.ships.len());
        (a, log)
    }

    /// Each machine gives its own side's scripted commands as it comes to the tick before
    /// theirs, the way a player would press them.
    fn press(t: &mut Table, log: &[(u32, Cmd)], given: &mut [usize; 2]) {
        for k in 0..2 {
            let mine: Vec<&(u32, Cmd)> = log.iter().filter(|c| c.1.team() == k).collect();
            while given[k] < mine.len() && mine[given[k]].0 <= t.arena[k].tick + 1 {
                t.link[k].give(&t.arena[k], mine[given[k]].1);
                given[k] += 1;
            }
        }
    }

    #[test]
    fn alone_a_command_waits_out_the_lag_and_not_a_tick_more() {
        // The dial at 0: a command given now is applied on the very next tick, as it always was.
        // At 3 it's applied three ticks after that, and at each setting in between likewise.
        for delay in [0, 1, 3, 8] {
            let mut a = Arena::for_match(1, [starter_deck(), starter_deck()], [false, false]);
            let mut link = Link::solo();
            link.delay = delay;
            link.advance(&mut a, 10);
            link.give(&a, Cmd::Print { team: 0, slot: 0, target: None });
            for waited in 0..=delay {
                assert!(a.eco.as_ref().unwrap().queue[0].is_empty(), "lag {delay}: the job was queued {waited} ticks in");
                link.advance(&mut a, 1);
            }
            assert_eq!(a.eco.as_ref().unwrap().queue[0].len(), 1, "lag {delay}: the job wasn't queued when the lag was up");
            assert_eq!(a.eco.as_ref().unwrap().log.last().unwrap().0, 11 + delay);
        }
    }

    #[test]
    fn alone_the_kings_keys_are_a_command_only_when_they_change() {
        let mut a = Arena::for_match(1, [starter_deck(), starter_deck()], [false, false]);
        let mut link = Link::solo();
        link.delay = 2;
        for frame in 0..30 {
            // Held right for ten frames, let go for ten, held down for ten.
            let (dx, dy) = [(1, 0), (0, 0), (0, 1)][frame / 10];
            link.steer(&a, dx, dy);
            link.advance(&mut a, 1);
        }
        let log = &a.eco.as_ref().unwrap().log;
        assert_eq!(log.iter().map(|c| c.0).collect::<Vec<_>>(), vec![3, 13, 23], "{log:?}");
    }

    #[test]
    fn the_kings_keys_cross_the_wire_only_when_they_change() {
        // A key held down is one command, not one a frame: thirty frames of holding right,
        // letting go and holding down put three commands on the wire, and every turn is sent
        // all the same, most of them empty.
        let (a, mut b, clock, _) = pipes();
        let mut arena = Arena::for_match(3, [starter_deck(), starter_deck()], [false, false]);
        let mut link = Link::net(0, Box::new(a), Vec::new());
        let (mut turns, mut cmds) = (0, Vec::new());
        for frame in 0..30u32 {
            clock.set(clock.get() + 1);
            b.send(&Msg::Turn { turn: frame + 1, tick: frame, cmds: Vec::new() });
            let (dx, dy) = [(1, 0), (0, 0), (0, 1)][frame as usize / 10];
            link.steer(&arena, dx, dy);
            link.advance(&mut arena, 1);
            for m in b.recv().unwrap() {
                if let Msg::Turn { cmds: c, .. } = m {
                    turns += 1;
                    cmds.extend(c);
                }
            }
        }
        assert_eq!(arena.tick, 30);
        assert_eq!(turns, 30 + NET_DELAY, "a turn for every tick, as far ahead as the lag");
        assert_eq!(cmds, vec![Cmd::Steer { team: 0, dx: 1, dy: 0 }, Cmd::Steer { team: 0, dx: 0, dy: 0 }, Cmd::Steer { team: 0, dx: 0, dy: 1 }]);
    }

    #[test]
    fn two_machines_with_no_lag_play_the_very_match_the_commands_made() {
        // The commands of an AI match, split by side and given on two machines joined by a wire,
        // each machine pressing only its own side's. With no delay and nothing lost on the wire
        // they play the match those commands came from, tick for tick, and each machine's hash
        // of it is the other's.
        let (reference, log) = script(4);
        let mut t = Table::new(4);
        for l in &mut t.link {
            l.delay = 0;
        }
        let (mut given, mut seen) = ([0; 2], BTreeMap::new());
        let mut frames = 0;
        while t.arena.iter().any(|a| a.tick < reference.tick) {
            press(&mut t, &log, &mut given);
            t.frame([true, true]);
            t.in_step(&mut seen);
            frames += 1;
            assert!(frames < 15 * 180 * 4, "the machines stopped: at ticks {} and {}", t.arena[0].tick, t.arena[1].tick);
        }
        for k in 0..2 {
            assert_eq!(t.arena[k].checksum(), reference.checksum(), "machine {k} played a different match");
            assert_eq!(t.arena[k].eco.as_ref().unwrap().log, log);
            assert!(t.net(k).agreed >= reference.tick - 2 * HASH_EVERY, "machine {k} last agreed at tick {}", t.net(k).agreed);
        }
    }

    #[test]
    fn two_machines_stay_in_step_through_lag_hangs_and_a_turning_dial() {
        // The same, as a real network would have it: messages take one to three frames, each
        // machine hangs now and then (a window being dragged), and both players turn their lag
        // dials as they go. The match that results isn't the scripted one, since the commands
        // land later, but it's one match: every tick both machines reach hashes the same, no
        // turn is lost or sent twice, and a machine that fell behind comes level again.
        let (_, log) = script(5);
        let mut t = Table::new(5);
        let (mut given, mut seen) = ([0; 2], BTreeMap::new());
        let mut rng = Rng::new(9);
        let mut worst_gap = 0;
        for frame in 0..15 * 150u32 {
            t.lag.set(1 + (frame / 97) % 3);
            press(&mut t, &log, &mut given);
            if frame % 211 == 0 {
                t.link[(frame / 211 % 2) as usize].next_delay();
            }
            // Each machine misses about one frame in twenty, and one of them hangs for two
            // seconds every half minute.
            let hung = (frame % 450) < 30;
            let who = [!rng.chance(5) && !(hung && frame / 450 % 2 == 0), !rng.chance(5) && !(hung && frame / 450 % 2 == 1)];
            t.frame(who);
            t.in_step(&mut seen);
            if frame % 450 == 449 {
                worst_gap = worst_gap.max((t.arena[0].tick as i32 - t.arena[1].tick as i32).abs());
            }
        }
        let sent: usize = given.iter().sum();
        assert!(sent >= 300, "only {sent} commands were given");
        assert!(t.arena[0].tick > 15 * 80, "the match crawled: tick {} after 150 seconds of frames", t.arena[0].tick);
        assert!(worst_gap <= 4, "a machine that fell behind stayed {worst_gap} ticks behind");
        for k in 0..2 {
            assert!(t.net(k).agreed + 4 * HASH_EVERY >= t.arena[k].tick, "machine {k} last agreed at tick {} of {}", t.net(k).agreed, t.arena[k].tick);
            // Landing late, some of the script's commands no longer apply (a seed the bank can't
            // pay for yet), but most still do.
            let applied = t.arena[k].eco.as_ref().unwrap().log.len();
            assert!(applied * 2 >= sent, "machine {k} applied {applied} of the {sent} commands given");
        }
    }

    #[test]
    fn the_check_catches_two_matches_that_have_parted() {
        // One pixel moves from one bank to nowhere on one machine only: a difference too small
        // to see. Within a second both machines know, and know the tick.
        let mut t = Table::new(6);
        let mut seen = BTreeMap::new();
        for _ in 0..100 {
            t.frame([true, true]);
            t.in_step(&mut seen);
        }
        let k = t.arena[1].eco.as_ref().unwrap().king[0];
        t.arena[1].ships[k].store -= 1;
        let broke = t.arena[1].tick;
        for _ in 0..HASH_EVERY + 6 {
            t.frame([true, true]);
        }
        let due = (broke / HASH_EVERY + 1) * HASH_EVERY;
        for k in 0..2 {
            assert_eq!(t.net(k).parted, Some(due), "machine {k}");
            assert!(t.net(k).agreed < due);
        }
    }

    #[test]
    fn a_machine_plays_no_tick_without_the_others_turn_for_it() {
        // Blue's machine runs alone for a while: it plays as far as red's turns reach (none
        // have come, so nowhere) and waits. When red's machine starts, both go on together.
        let mut t = Table::new(7);
        for _ in 0..20 {
            t.frame([true, false]);
        }
        assert_eq!(t.arena[0].tick, 0, "blue played ticks red never sent a turn for");
        assert_eq!(t.net(0).waiting, 20);
        for _ in 0..40 {
            t.frame([true, true]);
        }
        assert!(t.arena[0].tick >= 30 && t.net(0).waiting == 0, "blue is at tick {}", t.arena[0].tick);
    }

    #[test]
    fn a_turn_missing_or_sent_twice_ends_the_match_and_is_never_played_round() {
        // Each turn comes once, in order. A game that skips one, or sends one again, isn't
        // playing the same match any more: this machine stops, saying so, and plays no tick on
        // what it was sent.
        for turns in [[1, 2, 4, 5, 6], [1, 2, 2, 3, 4]] {
            let (a, mut b, clock, _) = pipes();
            let mut arena = Arena::for_match(9, [starter_deck(), starter_deck()], [false, false]);
            let mut link = Link::net(0, Box::new(a), Vec::new());
            for turn in turns {
                b.send(&Msg::Turn { turn, tick: turn.saturating_sub(3), cmds: Vec::new() });
            }
            for _ in 0..10 {
                clock.set(clock.get() + 1);
                link.advance(&mut arena, 1);
            }
            let net = link.net.as_ref().unwrap();
            assert!(net.gone.is_some(), "turns {turns:?} were let through");
            assert!(arena.tick <= 2, "turns {turns:?}: it played on to tick {}", arena.tick);
        }
    }

    #[test]
    fn a_command_counts_only_for_the_side_that_sent_it() {
        // Red's machine sends a command in blue's name: "blue's king prints a fighter". Neither
        // machine applies it, so they stay in step and blue's queue stays empty.
        let mut t = Table::new(8);
        let mut seen = BTreeMap::new();
        for _ in 0..10 {
            t.frame([true, true]);
        }
        let forged = Cmd::Print { team: 0, slot: 0, target: None };
        let (red, arena) = (&mut t.link[1], &t.arena[1]);
        red.give(arena, forged);
        red.give(arena, Cmd::Print { team: 1, slot: 0, target: None });
        for _ in 0..30 {
            t.frame([true, true]);
            t.in_step(&mut seen);
        }
        for k in 0..2 {
            let e = t.arena[k].eco.as_ref().unwrap();
            assert!(e.queue[0].is_empty(), "machine {k} let red print for blue");
            assert_eq!(e.queue[1].len(), 1, "machine {k} lost red's own command");
        }
    }

    #[test]
    fn every_message_comes_back_off_the_wire_as_it_went_on() {
        let cmds = vec![
            Cmd::Steer { team: 1, dx: -1, dy: 1 },
            Cmd::Print { team: 0, slot: 6, target: None },
            Cmd::Print { team: 1, slot: 4, target: Some((1399, 2)) },
            Cmd::Order { team: 0, slot: 2, order: Order::Escort },
            Cmd::Repeat { team: 1, slot: 3, at: None },
            Cmd::Repeat { team: 0, slot: 3, at: Some(5) },
            Cmd::Cancel { team: 1, at: 2, slot: 0 },
        ];
        for c in &cmds {
            assert_eq!(Cmd::from_wire(&c.to_wire()), Some(*c), "{}", c.to_wire());
        }
        let msgs = vec![
            Msg::Hello { proto: PROTO, seed: 39_456, deck: starter_deck().iter().map(|d| d.to_line()).collect() },
            Msg::Turn { turn: 7, tick: 4, cmds: Vec::new() },
            Msg::Turn { turn: 8, tick: 5, cmds },
            Msg::Hash { tick: 15, hash: 0x0123_4567_89ab_cdef },
            Msg::Bye,
        ];
        let mut r = Reader::default();
        let mut back = Vec::new();
        for m in &msgs {
            for line in m.to_text().lines() {
                back.extend(r.line(line).unwrap());
            }
        }
        assert_eq!(back, msgs);
        // And nothing that isn't a message gets in: a side that isn't one, a slot off the deck, a
        // word too many, a number that isn't one, a turn with a broken command in it.
        for bad in ["S 2 0 0", "P 0 64", "P 0 1 5", "O 0 1 9", "R 0", "C 0 1", "X 0 0", "S 0 a 0", "P -1 0", ""] {
            assert_eq!(Cmd::from_wire(bad), None, "{bad:?} was read as a command");
        }
        for bad in ["T 1", "T 1 0 S 0 0 0|nonsense", "H 1 xyz", "HELLO 1 2", "B now", "GET / HTTP/1.1"] {
            assert!(Reader::default().line(bad).is_err(), "{bad:?} was read as a message");
        }
    }

    #[test]
    fn two_games_find_each_other_and_play_in_step_over_a_real_socket() {
        // The whole way through the real thing on this machine: a host listening, a joiner
        // connecting, hellos with both decks, then four seconds of match over TCP.
        let mut host = Lobby::host_on("127.0.0.1", 0);
        let Lobby::Hosting { port, .. } = &host else { panic!("couldn't listen") };
        let port = *port;
        let mut join = Lobby::join(&format!("127.0.0.1:{port}"));
        let (deck, mut met) = (starter_deck(), [None, None]);
        let started = std::time::Instant::now();
        while met.iter().any(|m| m.is_none()) {
            assert!(started.elapsed() < Duration::from_secs(10), "the two never met");
            if met[0].is_none() {
                met[0] = host.poll(77, &deck);
            }
            if met[1].is_none() {
                met[1] = join.poll(1, &deck);
            }
            std::thread::sleep(Duration::from_millis(2));
        }
        let [h, j] = met.map(|m| m.unwrap());
        assert!(h.host && !j.host && (h.seed, j.seed) == (77, 77), "the host's seed is the match's");
        assert_eq!(j.deck.len(), 7);
        let mut arena = [0, 1].map(|_| Arena::for_match(77, [starter_deck(), starter_deck()], [false, false]));
        let mut link = [Link::net(0, Box::new(h.wire), h.backlog), Link::net(1, Box::new(j.wire), j.backlog)];
        link[1].give(&arena[1], Cmd::Print { team: 1, slot: 0, target: None });
        let started = std::time::Instant::now();
        while arena.iter().any(|a| a.tick < 60) {
            assert!(started.elapsed() < Duration::from_secs(10), "stuck at ticks {} and {}", arena[0].tick, arena[1].tick);
            for k in 0..2 {
                link[k].advance(&mut arena[k], 1);
            }
            std::thread::sleep(Duration::from_millis(1));
        }
        for k in 0..2 {
            let net = link[k].net.as_ref().unwrap();
            assert_eq!((net.parted, net.gone.clone()), (None, None));
            assert!(net.agreed >= 30, "machine {k} agreed only as far as tick {}", net.agreed);
            assert_eq!(arena[k].eco.as_ref().unwrap().queue[1].len(), 1, "machine {k} never got red's job");
        }
        // The joiner leaves, and the host is told.
        link[1].bye();
        let started = std::time::Instant::now();
        while link[0].net.as_ref().unwrap().gone.is_none() {
            assert!(started.elapsed() < Duration::from_secs(5), "the host never heard the joiner leave");
            link[0].advance(&mut arena[0], 1);
            std::thread::sleep(Duration::from_millis(1));
        }
    }
}
