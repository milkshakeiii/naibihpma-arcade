//! A server: runs a world at 15 ticks a second for players connecting over TCP, draws each
//! player's screen and sends them what changed (see `net.rs`). Playing alone runs one inside the
//! game, on a port only this computer can reach.
//!
//! Each connection gets a thread that reads its keys and one that writes its frames, so the
//! world never waits on a player. A player who falls behind has frames skipped: the next one
//! sent covers everything they missed.

use crate::framebuffer::{Framebuffer, FB_H, FB_W};
use crate::game::{Game, Input, Sfx};
use crate::net::{self, ToPlayer, ToServer};
use crate::sprites::Sprites;
use std::io::{self, BufReader};
use std::net::{Shutdown, SocketAddr, TcpListener, TcpStream};
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{self, Receiver, Sender, SyncSender, TrySendError};
use std::time::{Duration, Instant};

/// Frames waiting for a player before the server stops queueing more.
const QUEUE: usize = 4;
/// Sounds kept for a player who's behind.
const MAX_SOUNDS: usize = 16;
/// A player whose connection takes longer than this to accept a frame is let go.
const WRITE_TIMEOUT: Duration = Duration::from_secs(10);

enum Event {
    Hello { protocol: u32, key: String },
    Keys(Input),
    Gone,
}

struct Player {
    id: u64,
    out: SyncSender<Vec<u8>>,
    /// Theirs once they've said hello.
    commune: Option<usize>,
    /// Keys gathered since the last tick, and what's held now.
    keys: Input,
    held: Input,
    /// The screen they'll have once everything queued reaches them.
    screen: Vec<u8>,
    /// Sounds not sent yet, while they're behind.
    sounds: Vec<Sfx>,
}

pub struct Server {
    pub game: Game,
    path: PathBuf,
    listener: TcpListener,
    players: Vec<Player>,
    events: (Sender<(u64, Event)>, Receiver<(u64, Event)>),
    next_id: u64,
    sprites: Sprites,
    fb: Framebuffer,
    since_save: u32,
}

impl Server {
    /// A server for this world, saved at `path`, taking players on `listener`.
    pub fn new(game: Game, path: PathBuf, listener: TcpListener) -> io::Result<Server> {
        listener.set_nonblocking(true)?;
        Ok(Server { game, path, listener, players: Vec::new(), events: mpsc::channel(), next_id: 0, sprites: Sprites::new(), fb: Framebuffer::new(), since_save: 0 })
    }

    #[cfg_attr(not(test), allow(dead_code))]
    pub fn addr(&self) -> SocketAddr {
        self.listener.local_addr().expect("a bound listener")
    }

    /// Runs at 15 ticks a second until `stop` is set, then lets everyone go and saves.
    pub fn run(mut self, stop: &AtomicBool) {
        let tick = Duration::from_micros(66_667);
        let mut next = Instant::now();
        while !stop.load(Ordering::Relaxed) {
            let now = Instant::now();
            if now < next {
                std::thread::sleep((next - now).min(Duration::from_millis(4)));
                continue;
            }
            next += tick;
            if now > next + tick * 4 {
                next = now + tick;
            }
            self.tick();
        }
        for p in &self.players {
            let _ = p.out.try_send(ToPlayer::Bye("THE SERVER STOPPED.".into()).encode());
        }
        self.players.clear();
        if let Err(e) = crate::save::save(&self.game, &self.path) {
            eprintln!("saving {} failed: {e}", self.path.display());
        }
    }

    /// One tick: new players, everyone's keys, the world, everyone's screens, and an autosave
    /// now and then.
    pub fn tick(&mut self) {
        self.accept();
        self.hear();
        let mut inputs = vec![Input::default(); self.game.communes.len()];
        for p in &mut self.players {
            if let Some(k) = p.commune {
                inputs[k] = p.keys.or(&p.held);
                p.keys = Input::default();
            }
        }
        self.game.update(&inputs);
        self.show();
        self.since_save += 1;
        if self.since_save >= crate::save::AUTOSAVE_TICKS {
            self.since_save = 0;
            if let Err(e) = crate::save::save(&self.game, &self.path) {
                eprintln!("saving {} failed: {e}", self.path.display());
            }
        }
    }

    fn accept(&mut self) {
        while let Ok((stream, _)) = self.listener.accept() {
            let ready = stream.set_nonblocking(false).and_then(|_| stream.set_nodelay(true)).and_then(|_| stream.set_write_timeout(Some(WRITE_TIMEOUT)));
            let (Ok(()), Ok(reader)) = (ready, stream.try_clone()) else { continue };
            let id = self.next_id;
            self.next_id += 1;
            let events = self.events.0.clone();
            std::thread::spawn(move || listen(id, reader, events));
            let (out, queue) = mpsc::sync_channel(QUEUE);
            std::thread::spawn(move || deliver(stream, queue));
            self.players.push(Player { id, out, commune: None, keys: Input::default(), held: Input::default(), screen: vec![0; FB_W * FB_H], sounds: Vec::new() });
        }
    }

    fn hear(&mut self) {
        while let Ok((id, e)) = self.events.1.try_recv() {
            let Some(i) = self.players.iter().position(|p| p.id == id) else { continue };
            match e {
                Event::Hello { protocol, .. } if protocol != net::PROTOCOL => self.let_go(i, "THIS SERVER RUNS ANOTHER VERSION OF THE GAME."),
                Event::Hello { key, .. } if self.players[i].commune.is_none() => {
                    let k = self.game.player(&key);
                    // The same player from somewhere else: the older connection goes.
                    if let Some(old) = self.players.iter().position(|p| p.commune == Some(k)) {
                        self.let_go(old, "PLAYING FROM SOMEWHERE ELSE NOW.");
                    }
                    let Some(p) = self.players.iter_mut().find(|p| p.id == id) else { continue };
                    p.commune = Some(k);
                    let _ = p.out.try_send(ToPlayer::Welcome { commune: k }.encode());
                }
                Event::Hello { .. } => {}
                Event::Keys(inp) => {
                    let p = &mut self.players[i];
                    p.keys = p.keys.or(&inp);
                    p.held = net::held_part(&inp);
                }
                Event::Gone => self.let_go(i, ""),
            }
        }
    }

    /// Draws each player's screen and sends what changed, with the sounds they hear.
    fn show(&mut self) {
        let mut gone = Vec::new();
        for (i, p) in self.players.iter_mut().enumerate() {
            let Some(k) = p.commune else { continue };
            crate::render::draw(&self.game, k, &self.sprites, &mut self.fb);
            p.sounds.extend(self.game.sounds_for(k));
            let excess = p.sounds.len().saturating_sub(MAX_SOUNDS);
            p.sounds.drain(..excess);
            let pixels = net::pixel_delta(&p.screen, &self.fb.px);
            if pixels.is_empty() && p.sounds.is_empty() {
                continue;
            }
            match p.out.try_send(ToPlayer::Frame { sounds: p.sounds.clone(), pixels }.encode()) {
                Ok(()) => {
                    p.screen.copy_from_slice(&self.fb.px);
                    p.sounds.clear();
                }
                // Behind: this one isn't sent, and the next one covers it.
                Err(TrySendError::Full(_)) => {}
                Err(TrySendError::Disconnected(_)) => gone.push(i),
            }
        }
        for i in gone.into_iter().rev() {
            self.let_go(i, "");
        }
    }

    /// Lets a player go, telling them why if there's a reason. Their writer sends what's queued,
    /// then closes the connection.
    fn let_go(&mut self, i: usize, why: &str) {
        let p = self.players.remove(i);
        if !why.is_empty() {
            let _ = p.out.try_send(ToPlayer::Bye(why.into()).encode());
        }
    }
}

/// Reads a player's messages until they go.
fn listen(id: u64, stream: TcpStream, events: Sender<(u64, Event)>) {
    let mut r = BufReader::new(stream);
    loop {
        let e = match ToServer::read(&mut r) {
            Ok(ToServer::Hello { protocol, key }) => Event::Hello { protocol, key },
            Ok(ToServer::Keys(inp)) => Event::Keys(inp),
            Err(_) => Event::Gone,
        };
        let gone = matches!(e, Event::Gone);
        if events.send((id, e)).is_err() || gone {
            return;
        }
    }
}

/// Writes a player's frames as they're queued; closes the connection once the server lets them
/// go or they stop taking them.
fn deliver(mut stream: TcpStream, queue: Receiver<Vec<u8>>) {
    use std::io::Write;
    for msg in queue {
        if stream.write_all(&msg).is_err() {
            break;
        }
    }
    let _ = stream.shutdown(Shutdown::Both);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::client::Connection;

    /// Ticks the server until `done`, a second at most.
    fn tick_until(s: &mut Server, mut done: impl FnMut(&mut Server) -> bool) {
        for _ in 0..200 {
            s.tick();
            if done(s) {
                return;
            }
            std::thread::sleep(Duration::from_millis(5));
        }
        panic!("timed out");
    }

    fn server() -> Server {
        let dir = std::env::temp_dir().join(format!("mw_server_test_{}_{}", std::process::id(), rand_suffix()));
        Server::new(Game::new(), dir.join("world.json"), TcpListener::bind("127.0.0.1:0").unwrap()).unwrap()
    }

    fn rand_suffix() -> u64 {
        use std::hash::{BuildHasher, Hasher};
        let mut h = std::collections::hash_map::RandomState::new().build_hasher();
        h.write_u8(0);
        h.finish()
    }

    /// Everything that's arrived for a player: their commune once welcomed, their screen and
    /// sounds from every frame so far.
    struct Seen {
        commune: Option<usize>,
        screen: Vec<u8>,
        sounds: Vec<Sfx>,
        bye: Option<String>,
    }

    fn seen() -> Seen {
        Seen { commune: None, screen: vec![0; FB_W * FB_H], sounds: Vec::new(), bye: None }
    }

    fn take(c: &Connection, s: &mut Seen) {
        // An error is the connection closing, after a bye.
        for m in c.messages().into_iter().flatten() {
            match m {
                ToPlayer::Welcome { commune } => s.commune = Some(commune),
                ToPlayer::Frame { sounds, pixels } => {
                    net::apply_delta(&mut s.screen, &pixels).unwrap();
                    s.sounds.extend(sounds);
                }
                ToPlayer::Bye(why) => s.bye = Some(why),
            }
        }
    }

    #[test]
    fn two_players_each_see_their_own_screen_as_the_server_draws_it() {
        let mut s = server();
        let (mut a, b) = (Connection::connect(s.addr(), "key-a", Duration::ZERO).unwrap(), Connection::connect(s.addr(), "key-b", Duration::ZERO).unwrap());
        let (mut sa, mut sb) = (seen(), seen());
        tick_until(&mut s, |_| {
            take(&a, &mut sa);
            take(&b, &mut sb);
            sa.commune.is_some() && sb.commune.is_some()
        });
        let (ka, kb) = (sa.commune.unwrap(), sb.commune.unwrap());
        assert_eq!(ka.min(kb), 0, "the first to arrive gets Mogg and Bruna");
        assert_eq!(ka.max(kb), 1, "the other a new commune");
        assert_eq!(s.game.communes[ka].key, "key-a");

        // A presses up; only A's monster moves.
        let a_monster = s.game.communes[ka].controlled;
        let b_monster = s.game.communes[kb].controlled;
        let (ay, bpos) = (s.game.monsters[a_monster].y, (s.game.monsters[b_monster].x, s.game.monsters[b_monster].y));
        a.keys(&Input { held: [true, false, false, false], pressed: [true, false, false, false], ..Input::default() }).unwrap();
        a.keys(&Input::default()).unwrap();
        tick_until(&mut s, |s| s.game.monsters[a_monster].y != ay || s.game.monsters[a_monster].bump.is_some());
        assert_eq!((s.game.monsters[b_monster].x, s.game.monsters[b_monster].y), bpos);

        // Once the frames arrive, each screen is exactly what the server draws for that player.
        for _ in 0..5 {
            s.tick();
        }
        std::thread::sleep(Duration::from_millis(50));
        take(&a, &mut sa);
        take(&b, &mut sb);
        let mut fb = Framebuffer::new();
        crate::render::draw(&s.game, ka, &s.sprites, &mut fb);
        assert!(sa.screen == fb.px, "A's screen is the server's picture");
        crate::render::draw(&s.game, kb, &s.sprites, &mut fb);
        assert!(sb.screen == fb.px, "B's screen is the server's picture");
    }

    #[test]
    fn a_player_who_comes_back_gets_their_own_commune_and_the_old_connection_goes() {
        let mut s = server();
        let first = Connection::connect(s.addr(), "key-a", Duration::ZERO).unwrap();
        let other = Connection::connect(s.addr(), "key-b", Duration::ZERO).unwrap();
        let (mut s1, mut so) = (seen(), seen());
        tick_until(&mut s, |_| {
            take(&first, &mut s1);
            take(&other, &mut so);
            s1.commune.is_some() && so.commune.is_some()
        });
        let again = Connection::connect(s.addr(), "key-a", Duration::ZERO).unwrap();
        let mut s2 = seen();
        tick_until(&mut s, |_| {
            take(&again, &mut s2);
            take(&first, &mut s1);
            s2.commune.is_some() && s1.bye.is_some()
        });
        assert_eq!(s2.commune, s1.commune, "the same commune as before");
        assert!(s1.bye.as_deref().unwrap().contains("SOMEWHERE ELSE"));
        assert_eq!(s.game.communes.len(), 2, "nobody new joined");
    }

    #[test]
    fn a_player_hears_their_own_actions() {
        let mut s = server();
        let mut a = Connection::connect(s.addr(), "key-a", Duration::ZERO).unwrap();
        let mut sa = seen();
        tick_until(&mut s, |_| {
            take(&a, &mut sa);
            sa.commune.is_some()
        });
        a.keys(&Input { rec_pressed: true, ..Input::default() }).unwrap();
        tick_until(&mut s, |_| {
            take(&a, &mut sa);
            sa.sounds.contains(&Sfx::RecOn)
        });
    }

    #[test]
    fn a_lagged_player_waits_a_round_trip_and_their_keys_arrive_half_of_one_late() {
        let mut s = server();
        let lag = Duration::from_millis(200);
        let started = Instant::now();
        let mut a = Connection::connect(s.addr(), "key-a", lag).unwrap();
        let mut sa = seen();
        tick_until(&mut s, |_| {
            take(&a, &mut sa);
            sa.commune.is_some()
        });
        assert!(started.elapsed() >= lag, "welcomed after {:?}", started.elapsed());
        let m = s.game.communes[sa.commune.unwrap()].controlled;
        let (y, pressed) = (s.game.monsters[m].y, Instant::now());
        a.keys(&Input { held: [true, false, false, false], pressed: [true, false, false, false], ..Input::default() }).unwrap();
        a.keys(&Input::default()).unwrap();
        tick_until(&mut s, |s| s.game.monsters[m].y != y || s.game.monsters[m].bump.is_some());
        assert!(pressed.elapsed() >= lag / 2, "the key arrived after {:?}", pressed.elapsed());
    }

    #[test]
    fn stopping_lets_everyone_go_and_saves_the_world_with_its_players() {
        let mut s = server();
        let a = Connection::connect(s.addr(), "key-a", Duration::ZERO).unwrap();
        let b = Connection::connect(s.addr(), "key-b", Duration::ZERO).unwrap();
        let (mut sa, mut sb) = (seen(), seen());
        tick_until(&mut s, |_| {
            take(&a, &mut sa);
            take(&b, &mut sb);
            sa.commune.is_some() && sb.commune.is_some()
        });
        let path = s.path.clone();
        s.run(&AtomicBool::new(true));
        let loaded = crate::save::load(&path).expect("saved");
        let keys: Vec<&str> = loaded.communes.iter().map(|c| c.key.as_str()).collect();
        assert!(keys.contains(&"key-a") && keys.contains(&"key-b"), "{keys:?}");
        std::thread::sleep(Duration::from_millis(100));
        take(&a, &mut sa);
        assert_eq!(sa.bye.as_deref(), Some("THE SERVER STOPPED."));
        let _ = std::fs::remove_dir_all(path.parent().unwrap());
    }
}
