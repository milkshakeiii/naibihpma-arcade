//! What a server and a player's game say to each other over TCP. The server runs the world and
//! draws each player's screen; a player's game sends its keys and shows the pictures.
//!
//! Every message is a 4-byte little-endian length, then a tag byte, then its payload.
//! - To the server: `Hello` (protocol version and the player's key), then `Keys` whenever a key
//!   goes down or anything held changes.
//! - To a player: `Welcome` (which commune is theirs), then a `Frame` each tick something
//!   changed: the sounds they hear and their screen's changed pixels, compressed. `Bye` when the
//!   server lets them go.
//!
//! Frames are deltas: the changed pixels are the old screen XOR-ed with the new, deflated. TCP
//! keeps them in order, so a player's screen is always the server's picture.

use crate::framebuffer::{FB_H, FB_W};
use crate::game::{Input, Sfx};
use std::io::{self, Read, Write};

/// Bump when messages change; a player's game and a server must agree on it.
pub const PROTOCOL: u32 = 2;
pub const DEFAULT_PORT: u16 = 7777;
const MAX_MESSAGE: usize = 4 << 20;

pub enum ToServer {
    Hello { protocol: u32, key: String },
    Keys(Input),
}

pub enum ToPlayer {
    Welcome { commune: usize },
    Frame { sounds: Vec<Sfx>, pixels: Vec<u8> },
    Bye(String),
}

fn write_message(w: &mut impl Write, tag: u8, payload: &[u8]) -> io::Result<()> {
    let mut out = Vec::with_capacity(5 + payload.len());
    out.extend_from_slice(&(1 + payload.len() as u32).to_le_bytes());
    out.push(tag);
    out.extend_from_slice(payload);
    w.write_all(&out)
}

fn read_message(r: &mut impl Read) -> io::Result<(u8, Vec<u8>)> {
    let mut len = [0; 4];
    r.read_exact(&mut len)?;
    let len = u32::from_le_bytes(len) as usize;
    if len == 0 || len > MAX_MESSAGE {
        return Err(io::Error::new(io::ErrorKind::InvalidData, format!("a message {len} bytes long")));
    }
    let mut body = vec![0; len];
    r.read_exact(&mut body)?;
    let tag = body.remove(0);
    Ok((tag, body))
}

fn bad(what: &str) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, what.to_string())
}

impl ToServer {
    #[cfg_attr(not(windows), allow(dead_code))]
    pub fn write(&self, w: &mut impl Write) -> io::Result<()> {
        match self {
            ToServer::Hello { protocol, key } => {
                let mut p = protocol.to_le_bytes().to_vec();
                p.extend_from_slice(key.as_bytes());
                write_message(w, 1, &p)
            }
            ToServer::Keys(inp) => write_message(w, 2, &key_bits(inp).to_le_bytes()),
        }
    }

    pub fn read(r: &mut impl Read) -> io::Result<Self> {
        let (tag, p) = read_message(r)?;
        match tag {
            1 if p.len() >= 4 => {
                let protocol = u32::from_le_bytes(p[..4].try_into().unwrap());
                let key = String::from_utf8(p[4..].to_vec()).map_err(|_| bad("a key that isn't text"))?;
                Ok(ToServer::Hello { protocol, key })
            }
            2 if p.len() == 4 => Ok(ToServer::Keys(keys_from_bits(u32::from_le_bytes(p[..4].try_into().unwrap())))),
            _ => Err(bad("an unknown message to the server")),
        }
    }
}

impl ToPlayer {
    /// The message, ready to write (servers queue them for a writer thread).
    pub fn encode(&self) -> Vec<u8> {
        let mut out = Vec::new();
        let _ = match self {
            ToPlayer::Welcome { commune } => write_message(&mut out, 1, &(*commune as u32).to_le_bytes()),
            ToPlayer::Frame { sounds, pixels } => {
                let mut p = (sounds.len() as u16).to_le_bytes().to_vec();
                p.extend(sounds.iter().map(|&s| s as u8));
                p.extend_from_slice(pixels);
                write_message(&mut out, 2, &p)
            }
            ToPlayer::Bye(why) => write_message(&mut out, 3, why.as_bytes()),
        };
        out
    }

    #[cfg_attr(not(windows), allow(dead_code))]
    pub fn read(r: &mut impl Read) -> io::Result<Self> {
        let (tag, p) = read_message(r)?;
        match tag {
            1 if p.len() == 4 => Ok(ToPlayer::Welcome { commune: u32::from_le_bytes(p[..4].try_into().unwrap()) as usize }),
            2 if p.len() >= 2 => {
                let n = u16::from_le_bytes([p[0], p[1]]) as usize;
                let sounds = p.get(2..2 + n).ok_or_else(|| bad("a frame cut short"))?;
                let sounds = sounds.iter().map(|&b| Sfx::ALL.get(b as usize).copied().ok_or_else(|| bad("an unknown sound"))).collect::<io::Result<_>>()?;
                Ok(ToPlayer::Frame { sounds, pixels: p[2 + n..].to_vec() })
            }
            3 => Ok(ToPlayer::Bye(String::from_utf8_lossy(&p).into_owned())),
            _ => Err(bad("an unknown message to a player")),
        }
    }
}

/// Keys as bits: what's held, and what went down since the last message. Quitting and deleting
/// worlds never reach a server.
fn key_bits(i: &Input) -> u32 {
    let flags = [i.use_pressed, i.rec_pressed, i.play_pressed, i.tab_pressed, i.tab_held, i.cancel_pressed, i.help_pressed, i.inspect, i.pull, i.sheet_pressed, i.close_pressed];
    let dirs = (0..4).map(|k| (i.held[k] as u32) << k | (i.pressed[k] as u32) << (4 + k));
    dirs.chain(flags.iter().enumerate().map(|(k, &f)| (f as u32) << (8 + k))).fold(0, |a, b| a | b)
}

fn keys_from_bits(b: u32) -> Input {
    let bit = |k: u32| b & (1 << k) != 0;
    Input {
        held: [bit(0), bit(1), bit(2), bit(3)],
        pressed: [bit(4), bit(5), bit(6), bit(7)],
        use_pressed: bit(8),
        rec_pressed: bit(9),
        play_pressed: bit(10),
        tab_pressed: bit(11),
        tab_held: bit(12),
        cancel_pressed: bit(13),
        help_pressed: bit(14),
        inspect: bit(15),
        pull: bit(16),
        sheet_pressed: bit(17),
        close_pressed: bit(18),
        ..Input::default()
    }
}

/// What's held right now, with nothing that went down.
pub fn held_part(i: &Input) -> Input {
    Input { held: i.held, tab_held: i.tab_held, inspect: i.inspect, pull: i.pull, ..Input::default() }
}

/// Did anything go down?
pub fn any_pressed(i: &Input) -> bool {
    key_bits(i) != key_bits(&held_part(i))
}

/// What a screen needs to go from `had` to `now`: the changed pixels, compressed. Empty when
/// nothing changed.
pub fn pixel_delta(had: &[u8], now: &[u8]) -> Vec<u8> {
    if had == now {
        return Vec::new();
    }
    let xor: Vec<u8> = had.iter().zip(now).map(|(a, b)| a ^ b).collect();
    miniz_oxide::deflate::compress_to_vec(&xor, 6)
}

/// Applies a frame's changed pixels to the screen it was made for.
#[cfg_attr(not(windows), allow(dead_code))]
pub fn apply_delta(screen: &mut [u8], delta: &[u8]) -> io::Result<()> {
    if delta.is_empty() {
        return Ok(());
    }
    let xor = miniz_oxide::inflate::decompress_to_vec_with_limit(delta, FB_W * FB_H).map_err(|_| bad("pixels that don't inflate"))?;
    if xor.len() != screen.len() {
        return Err(bad("pixels for a different screen size"));
    }
    for (p, x) in screen.iter_mut().zip(xor) {
        *p ^= x;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keys_survive_the_trip() {
        let i = Input { held: [true, false, false, true], pressed: [false, false, true, false], use_pressed: true, tab_held: true, inspect: true, pull: true, cancel_pressed: true, sheet_pressed: true, close_pressed: true, ..Input::default() };
        let mut wire = Vec::new();
        ToServer::Keys(i).write(&mut wire).unwrap();
        let ToServer::Keys(back) = ToServer::read(&mut wire.as_slice()).unwrap() else { panic!("not keys") };
        assert_eq!(key_bits(&back), key_bits(&i));
        assert!(any_pressed(&i) && !any_pressed(&held_part(&i)));
        let quit = Input { quit_pressed: true, delete_pressed: true, ..Input::default() };
        assert_eq!(key_bits(&quit), 0, "quitting never reaches a server");
    }

    #[test]
    fn a_hello_and_a_frame_survive_the_trip() {
        let mut wire = Vec::new();
        ToServer::Hello { protocol: PROTOCOL, key: "abc123".into() }.write(&mut wire).unwrap();
        let ToServer::Hello { protocol, key } = ToServer::read(&mut wire.as_slice()).unwrap() else { panic!("not a hello") };
        assert_eq!((protocol, key.as_str()), (PROTOCOL, "abc123"));

        let had = vec![0u8; FB_W * FB_H];
        let mut now = had.clone();
        now[1234] = 17;
        now[99_999] = 63;
        let wire = ToPlayer::Frame { sounds: vec![Sfx::Chime, Sfx::Deny], pixels: pixel_delta(&had, &now) }.encode();
        let ToPlayer::Frame { sounds, pixels } = ToPlayer::read(&mut wire.as_slice()).unwrap() else { panic!("not a frame") };
        assert_eq!(sounds, vec![Sfx::Chime, Sfx::Deny]);
        let mut screen = had.clone();
        apply_delta(&mut screen, &pixels).unwrap();
        assert_eq!(screen, now);
        assert!(pixel_delta(&now, &now).is_empty(), "nothing changed, nothing sent");
    }

    #[test]
    fn a_garbled_message_is_an_error_not_a_crash() {
        let wire = [5u8, 0, 0, 0, 9, 1, 2, 3, 4];
        assert!(ToServer::read(&mut wire.as_slice()).is_err());
        let huge = [255u8, 255, 255, 255];
        assert!(ToPlayer::read(&mut huge.as_slice()).is_err());
        assert!(apply_delta(&mut vec![0; FB_W * FB_H], &[1, 2, 3]).is_err());
    }
}
