//! A player's side of a connection to a server (see `net.rs`): keys out, pictures and sounds
//! in. A thread reads what the server sends, so the window never waits on the network.
//!
//! A connection can be given lag, to feel a slow one on a fast one: what goes out and what
//! comes in are each held back half of it, so every round trip takes that much longer.

use crate::game::Input;
use crate::net::{self, ToPlayer, ToServer};
use std::io::{self, BufReader, Write};
use std::net::{TcpStream, ToSocketAddrs};
use std::sync::mpsc::{self, Receiver};
use std::time::{Duration, Instant};

pub struct Connection {
    send: Box<dyn FnMut(Vec<u8>) -> io::Result<()> + Send>,
    incoming: Receiver<io::Result<ToPlayer>>,
    /// What was held when keys were last sent.
    held: Input,
}

impl Connection {
    /// Connects and says hello with the player's key; the server's welcome arrives later.
    /// `lag` is added to every round trip.
    pub fn connect(addr: impl ToSocketAddrs, key: &str, lag: Duration) -> io::Result<Connection> {
        let stream = TcpStream::connect(addr)?;
        stream.set_nodelay(true)?;
        let (tx, incoming) = mpsc::channel();
        let mut arrive = delayed(lag / 2, move |m| tx.send(m).map_err(|_| closed()));
        let mut r = BufReader::new(stream.try_clone()?);
        std::thread::spawn(move || loop {
            let m = ToPlayer::read(&mut r);
            let end = m.is_err();
            if arrive(m).is_err() || end {
                return;
            }
        });
        let mut send = delayed(lag / 2, move |bytes: Vec<u8>| (&stream).write_all(&bytes));
        let mut hello = Vec::new();
        ToServer::Hello { protocol: net::PROTOCOL, key: key.into() }.write(&mut hello)?;
        send(hello)?;
        Ok(Connection { send, incoming, held: Input::default() })
    }

    /// Sends the keys if anything went down or what's held changed.
    pub fn keys(&mut self, inp: &Input) -> io::Result<()> {
        let held = net::held_part(inp);
        if !net::any_pressed(inp) && held == self.held {
            return Ok(());
        }
        self.held = held;
        let mut bytes = Vec::new();
        ToServer::Keys(*inp).write(&mut bytes)?;
        (self.send)(bytes)
    }

    /// Everything that has arrived since last asked. An error means the connection is gone.
    pub fn messages(&self) -> Vec<io::Result<ToPlayer>> {
        self.incoming.try_iter().collect()
    }
}

fn closed() -> io::Error {
    io::Error::new(io::ErrorKind::BrokenPipe, "the connection closed")
}

/// Hands each thing given to `out`, `lag` after it was given and in order: at once when there's
/// no lag, else from a thread of its own. Fails once `out` has.
fn delayed<T: Send + 'static>(lag: Duration, mut out: impl FnMut(T) -> io::Result<()> + Send + 'static) -> Box<dyn FnMut(T) -> io::Result<()> + Send> {
    if lag.is_zero() {
        return Box::new(out);
    }
    let (tx, rx) = mpsc::channel::<(Instant, T)>();
    std::thread::spawn(move || {
        for (due, t) in rx {
            std::thread::sleep(due.saturating_duration_since(Instant::now()));
            if out(t).is_err() {
                return;
            }
        }
    });
    Box::new(move |t| tx.send((Instant::now() + lag, t)).map_err(|_| closed()))
}

/// This player's key, made the first time it's needed and kept beside the saved worlds. It's
/// how servers know who's who: whoever has it plays this commune. A named player (`--player`)
/// has a key of their own, for playing several at once on one computer.
pub fn player_key(name: Option<&str>) -> String {
    let file = name.map_or_else(|| "player.key".to_string(), |n| format!("player-{n}.key"));
    let path = crate::save::worlds_dir().with_file_name(file);
    if let Ok(key) = std::fs::read_to_string(&path) {
        if !key.trim().is_empty() {
            return key.trim().to_string();
        }
    }
    use std::hash::{BuildHasher, Hasher};
    let part = |n: u64| {
        let mut h = std::collections::hash_map::RandomState::new().build_hasher();
        h.write_u64(n);
        h.write_u128(std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map_or(0, |d| d.as_nanos()));
        h.write_u32(std::process::id());
        h.finish()
    };
    let key = format!("{:016x}{:016x}", part(1), part(2));
    if let Some(dir) = path.parent() {
        let _ = std::fs::create_dir_all(dir);
    }
    let _ = std::fs::write(&path, &key);
    key
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lag_holds_things_back_in_order() {
        let (tx, rx) = mpsc::channel();
        let lag = Duration::from_millis(60);
        let mut line = delayed(lag, move |n: u32| tx.send((n, Instant::now())).map_err(|_| closed()));
        let sent = Instant::now();
        for n in 0..3 {
            line(n).unwrap();
        }
        let got: Vec<(u32, Instant)> = rx.iter().take(3).collect();
        assert_eq!(got.iter().map(|g| g.0).collect::<Vec<_>>(), vec![0, 1, 2]);
        assert!(got.iter().all(|g| g.1 - sent >= lag), "nothing arrives early");
    }
}
