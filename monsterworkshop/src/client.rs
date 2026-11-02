//! A player's side of a connection to a server (see `net.rs`): keys out, pictures and sounds
//! in. A thread reads what the server sends, so the window never waits on the network.

use crate::game::Input;
use crate::net::{self, ToPlayer, ToServer};
use std::io::{self, BufReader};
use std::net::{TcpStream, ToSocketAddrs};
use std::sync::mpsc::{self, Receiver};

pub struct Connection {
    stream: TcpStream,
    incoming: Receiver<io::Result<ToPlayer>>,
    /// What was held when keys were last sent.
    held: Input,
}

impl Connection {
    /// Connects and says hello with the player's key; the server's welcome arrives later.
    pub fn connect(addr: impl ToSocketAddrs, key: &str) -> io::Result<Connection> {
        let stream = TcpStream::connect(addr)?;
        stream.set_nodelay(true)?;
        ToServer::Hello { protocol: net::PROTOCOL, key: key.into() }.write(&mut &stream)?;
        let (tx, incoming) = mpsc::channel();
        let mut r = BufReader::new(stream.try_clone()?);
        std::thread::spawn(move || loop {
            let m = ToPlayer::read(&mut r);
            let end = m.is_err();
            if tx.send(m).is_err() || end {
                return;
            }
        });
        Ok(Connection { stream, incoming, held: Input::default() })
    }

    /// Sends the keys if anything went down or what's held changed.
    pub fn keys(&mut self, inp: &Input) -> io::Result<()> {
        let held = net::held_part(inp);
        if !net::any_pressed(inp) && held == self.held {
            return Ok(());
        }
        self.held = held;
        ToServer::Keys(*inp).write(&mut &self.stream)
    }

    /// Everything that has arrived since last asked. An error means the connection is gone.
    pub fn messages(&self) -> Vec<io::Result<ToPlayer>> {
        self.incoming.try_iter().collect()
    }
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
