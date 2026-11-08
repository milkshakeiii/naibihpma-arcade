//! Background music streamed from the album masters through winmm waveOut, with synthesized
//! sound effects mixed on top. The music bus sits well under the effects so every click, catch
//! and turn stays audible. F3 steps the music level.

use std::fs::File;
use std::io::{BufReader, Read, Seek, SeekFrom};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};

const RATE: u64 = 44_100;
const NUM_BUFS: usize = 6;
const FRAMES_PER_BUF: usize = 1024; // ~23 ms each

/// Music gain levels out of 256 (F3 cycles). The default is `MUSIC_LEVELS[2]`, about -11 dB.
pub const MUSIC_LEVELS: [i32; 4] = [0, 40, 72, 108];
pub const MUSIC_LEVEL_NAMES: [&str; 4] = ["OFF", "LOW", "MEDIUM", "HIGH"];
/// Effects are synthesized at this peak (out of 32767).
const SFX_PEAK: f32 = 19000.0;

const ALBUMS: &str = r"C:\Users\User\Desktop\albums";

/// Every piece of music the games use, all from the player's albums.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Track {
    /// s1-04 Delta: beatless, for the launcher and the feedback form.
    Menu,
    /// s3-02 Doat, the Enchanter: Changeling's lessons.
    Doat,
    /// s3-01 The Underbrush.
    Underbrush,
    /// s3-03 Twen, Dew Collector.
    Twen,
    /// s3-04 Star Pond.
    StarPond,
    /// s3-05 Grof, Malevolent Sorcerer.
    Grof,
    /// s3-06 Ancient Magic: Changeling's victory.
    AncientMagic,
    /// s5-01 .. s5-06 Virtual History, one per age of Turncoat.
    Prehistory,
    IronAge,
    GoldenAge,
    SpaceAge,
    AMyth,
    Posthistory,
}

impl Track {
    fn file(self) -> (&'static str, &'static str) {
        const S1: &str = r"s1_abstractobjects\mastered";
        const S3: &str = r"s3_thenechantedforestofnaibihpma\mastered_resonance";
        const S5: &str = r"s5_virtualhistory\Virtual_History-Mastered";
        match self {
            Track::Menu => (S1, "4_delta_master.wav"),
            Track::Doat => (S3, "2_doat_the_enchanter_master.wav"),
            Track::Underbrush => (S3, "1_the_underbrush_master.wav"),
            Track::Twen => (S3, "3_twen_dew_collector_master.wav"),
            Track::StarPond => (S3, "4_star_pond_master.wav"),
            Track::Grof => (S3, "5_grof_malevolent_sorcerer_master.wav"),
            Track::AncientMagic => (S3, "6_ancient_magic_master.wav"),
            Track::Prehistory => (S5, "1_Prehistory-Mastered-v2.wav"),
            Track::IronAge => (S5, "2_IronAge-Mastered-v2.wav"),
            Track::GoldenAge => (S5, "3_GoldenAge-Mastered-v2.wav"),
            Track::SpaceAge => (S5, "4_SpaceAge-Mastered-v1.wav"),
            Track::AMyth => (S5, "5_AMyth-Mastered-v1.wav"),
            Track::Posthistory => (S5, "6_Posthistory-Mastered-v2.wav"),
        }
    }

    pub fn title(self) -> &'static str {
        match self {
            Track::Menu => "DELTA",
            Track::Doat => "DOAT, THE ENCHANTER",
            Track::Underbrush => "THE UNDERBRUSH",
            Track::Twen => "TWEN, DEW COLLECTOR",
            Track::StarPond => "STAR POND",
            Track::Grof => "GROF, MALEVOLENT SORCERER",
            Track::AncientMagic => "ANCIENT MAGIC",
            Track::Prehistory => "PREHISTORY",
            Track::IronAge => "IRON AGE",
            Track::GoldenAge => "GOLDEN AGE",
            Track::SpaceAge => "SPACE AGE",
            Track::AMyth => "A MYTH",
            Track::Posthistory => "POSTHISTORY",
        }
    }

    /// Looks next to the exe, in the working directory's `assets/music`, then the album masters.
    pub fn path(self) -> Option<PathBuf> {
        let (dir, file) = self.file();
        let mut dirs = vec![PathBuf::from("assets/music")];
        if let Some(d) = std::env::current_exe().ok().and_then(|p| p.parent().map(Path::to_path_buf)) {
            dirs.insert(0, d.join("assets/music"));
        }
        dirs.push(Path::new(ALBUMS).join(dir));
        dirs.into_iter().map(|d| d.join(file)).find(|p| p.is_file())
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Sfx {
    Click,
    Step,
    Hop,
    Catch,
    Hurt,
    Strike,
    Deny,
    Dew,
    Rest,
    Rustle,
    Arrive,
    Descend,
    Buy,
    Die,
    Win,
    Place,
    Lift,
    Persuade,
    Turn,
    BattleStart,
    BattleWin,
    BattleLose,
    Key,
    Lesson,
    Ward,
    Block,
}

struct Shared {
    device_ok: Option<bool>,
    play: Option<Option<PathBuf>>,
    gain: i32,
    quit: bool,
    sfx: Vec<(Arc<Vec<i16>>, usize)>,
}

pub struct Audio {
    shared: Arc<Mutex<Shared>>,
    device_ok: bool,
    current: Option<Track>,
    enabled: bool,
    bank: Vec<Arc<Vec<i16>>>,
}

impl Audio {
    pub fn new(music_enabled: bool, level: usize) -> Self {
        let shared = Arc::new(Mutex::new(Shared {
            device_ok: None,
            play: None,
            gain: MUSIC_LEVELS[level.min(3)],
            quit: false,
            sfx: Vec::new(),
        }));
        let s = Arc::clone(&shared);
        thread::spawn(move || mixer_thread(s));
        let begin = Instant::now();
        let device_ok = loop {
            if let Some(ok) = shared.lock().ok().and_then(|g| g.device_ok) {
                break ok;
            }
            if begin.elapsed() > Duration::from_millis(1000) {
                break false;
            }
            thread::sleep(Duration::from_millis(2));
        };
        Self { shared, device_ok, current: None, enabled: music_enabled, bank: build_sfx_bank() }
    }

    /// Switches to `track` (a no-op if it's already playing). Tracks loop.
    pub fn music(&mut self, track: Track) {
        if self.current == Some(track) || !self.device_ok {
            return;
        }
        self.current = Some(track);
        let path = if self.enabled { track.path() } else { None };
        if let Ok(mut g) = self.shared.lock() {
            g.play = Some(path);
        }
    }

    pub fn set_level(&mut self, level: usize) {
        if let Ok(mut g) = self.shared.lock() {
            g.gain = MUSIC_LEVELS[level.min(3)];
        }
    }

    pub fn play_sfx(&self, sfx: Sfx) {
        let idx = sfx as usize;
        if let (Some(buf), Ok(mut g)) = (self.bank.get(idx), self.shared.lock()) {
            if g.sfx.len() < 10 {
                g.sfx.push((Arc::clone(buf), 0));
            }
        }
    }
}

impl Drop for Audio {
    fn drop(&mut self) {
        if let Ok(mut g) = self.shared.lock() {
            g.quit = true;
        }
    }
}

struct WavStream {
    reader: BufReader<File>,
    data_start: u64,
    frames: u64,
    pos: u64,
    channels: usize,
    bits: u16,
    float: bool,
    raw: Vec<u8>,
}

impl WavStream {
    fn open(path: &Path) -> Option<Self> {
        let mut f = File::open(path).ok()?;
        let mut hdr = [0u8; 12];
        f.read_exact(&mut hdr).ok()?;
        if &hdr[0..4] != b"RIFF" || &hdr[8..12] != b"WAVE" {
            return None;
        }
        let (mut channels, mut bits, mut float, mut rate) = (0usize, 0u16, false, 0u32);
        let mut pos = 12u64;
        loop {
            f.seek(SeekFrom::Start(pos)).ok()?;
            let mut ch = [0u8; 8];
            f.read_exact(&mut ch).ok()?;
            let size = u32::from_le_bytes([ch[4], ch[5], ch[6], ch[7]]) as u64;
            if &ch[0..4] == b"fmt " {
                let mut fmt = [0u8; 16];
                f.read_exact(&mut fmt).ok()?;
                let tag = u16::from_le_bytes([fmt[0], fmt[1]]);
                channels = u16::from_le_bytes([fmt[2], fmt[3]]) as usize;
                rate = u32::from_le_bytes([fmt[4], fmt[5], fmt[6], fmt[7]]);
                bits = u16::from_le_bytes([fmt[14], fmt[15]]);
                float = tag == 3;
                if tag == 0xFFFE && size >= 26 {
                    let mut ext = [0u8; 10];
                    f.read_exact(&mut ext).ok()?;
                    float = ext[8] == 3; // first bytes of the subformat GUID
                }
            } else if &ch[0..4] == b"data" {
                let supported = matches!((bits, float), (16, false) | (24, false) | (32, true));
                if channels == 0 || rate != RATE as u32 || !supported {
                    return None;
                }
                let frames = size / (channels as u64 * bits as u64 / 8);
                f.seek(SeekFrom::Start(pos + 8)).ok()?;
                return Some(Self {
                    reader: BufReader::with_capacity(64 * 1024, f),
                    data_start: pos + 8,
                    frames,
                    pos: 0,
                    channels,
                    bits,
                    float,
                    raw: Vec::new(),
                });
            }
            pos += 8 + size + (size & 1);
        }
    }

    fn frame_bytes(&self) -> usize {
        self.channels * self.bits as usize / 8
    }

    fn rewind(&mut self) {
        self.pos = 0;
        let _ = self.reader.seek(SeekFrom::Start(self.data_start));
    }

    /// Fills interleaved stereo i16 at full scale, looping at the end.
    fn read(&mut self, out: &mut [i16]) {
        let mut done = 0;
        let fb = self.frame_bytes();
        let bps = self.bits as usize / 8;
        while done < out.len() / 2 {
            if self.pos >= self.frames {
                self.rewind();
            }
            let want = (out.len() / 2 - done).min((self.frames - self.pos) as usize);
            self.raw.resize(want * fb, 0);
            if want == 0 || self.reader.read_exact(&mut self.raw).is_err() {
                out[done * 2..].fill(0);
                self.pos = self.frames;
                return;
            }
            self.pos += want as u64;
            let (bits, float) = (self.bits, self.float);
            let sample = |raw: &[u8], at: usize| -> i16 {
                match (bits, float) {
                    (16, _) => i16::from_le_bytes([raw[at], raw[at + 1]]),
                    (24, _) => i16::from_le_bytes([raw[at + 1], raw[at + 2]]),
                    _ => {
                        let v = f32::from_le_bytes([raw[at], raw[at + 1], raw[at + 2], raw[at + 3]]);
                        (v.clamp(-1.0, 1.0) * 32767.0) as i16
                    }
                }
            };
            for i in 0..want {
                let base = i * fb;
                let l = sample(&self.raw, base);
                let r = if self.channels > 1 { sample(&self.raw, base + bps) } else { l };
                out[(done + i) * 2] = l;
                out[(done + i) * 2 + 1] = r;
            }
            done += want;
        }
    }
}

fn mixer_thread(shared: Arc<Mutex<Shared>>) {
    unsafe {
        let fmt = WaveFormatEx {
            w_format_tag: 1,
            n_channels: 2,
            n_samples_per_sec: RATE as u32,
            n_avg_bytes_per_sec: RATE as u32 * 4,
            n_block_align: 4,
            w_bits_per_sample: 16,
            cb_size: 0,
        };
        let mut hwo = 0usize;
        let ok = waveOutOpen(&mut hwo, WAVE_MAPPER, &fmt, 0, 0, 0) == 0 && hwo != 0;
        if let Ok(mut g) = shared.lock() {
            g.device_ok = Some(ok);
        }
        if !ok {
            return;
        }

        let hdr_size = std::mem::size_of::<WaveHdr>() as u32;
        let mut bufs: Vec<Vec<i16>> = (0..NUM_BUFS).map(|_| vec![0i16; FRAMES_PER_BUF * 2]).collect();
        let mut hdrs: Vec<WaveHdr> = bufs
            .iter_mut()
            .map(|b| WaveHdr {
                lp_data: b.as_mut_ptr() as *mut u8,
                dw_buffer_length: (FRAMES_PER_BUF * 4) as u32,
                dw_bytes_recorded: 0,
                dw_user: 0,
                dw_flags: 0,
                dw_loops: 0,
                lp_next: std::ptr::null_mut(),
                reserved: 0,
            })
            .collect();
        for h in hdrs.iter_mut() {
            waveOutPrepareHeader(hwo, h, hdr_size);
            h.dw_flags |= WHDR_DONE;
        }

        let mut stream: Option<WavStream> = None;
        // Fade-in ramp after a track change, in frames (0..FADE).
        const FADE: i64 = RATE as i64 / 2;
        let mut fade: i64 = FADE;
        let mut music = vec![0i16; FRAMES_PER_BUF * 2];
        loop {
            let (play, quit, gain) = match shared.lock() {
                Ok(mut g) => (g.play.take(), g.quit, g.gain),
                Err(_) => break,
            };
            if quit {
                break;
            }
            if let Some(path) = play {
                stream = path.as_deref().and_then(WavStream::open);
                fade = 0;
            }

            let mut queued = false;
            for i in 0..NUM_BUFS {
                if hdrs[i].dw_flags & WHDR_DONE == 0 {
                    continue;
                }
                match stream.as_mut() {
                    Some(s) if gain > 0 => s.read(&mut music),
                    _ => music.fill(0),
                }
                for f in 0..FRAMES_PER_BUF {
                    let g = gain as i64 * fade.min(FADE) / FADE;
                    fade += 1;
                    for c in 0..2 {
                        bufs[i][f * 2 + c] = ((music[f * 2 + c] as i64 * g) >> 8) as i16;
                    }
                }
                if let Ok(mut g) = shared.lock() {
                    for (samples, cursor) in g.sfx.iter_mut() {
                        for f in 0..FRAMES_PER_BUF {
                            let Some(&v) = samples.get(*cursor) else { break };
                            *cursor += 1;
                            for c in 0..2 {
                                let o = &mut bufs[i][f * 2 + c];
                                *o = (*o as i32 + v as i32).clamp(-32760, 32760) as i16;
                            }
                        }
                    }
                    g.sfx.retain(|(s, c)| *c < s.len());
                }
                hdrs[i].dw_flags &= !WHDR_DONE;
                waveOutWrite(hwo, &mut hdrs[i], hdr_size);
                queued = true;
            }
            if !queued {
                thread::sleep(Duration::from_millis(3));
            }
        }

        waveOutReset(hwo);
        for h in hdrs.iter_mut() {
            waveOutUnprepareHeader(hwo, h, hdr_size);
        }
        waveOutClose(hwo);
    }
}

/// Short synthesized effects, in the order of `Sfx`.
fn build_sfx_bank() -> Vec<Arc<Vec<i16>>> {
    use std::f32::consts::TAU;
    let noise = |i: f32| -> f32 {
        let mut x = (i * 7919.0) as u32 ^ 0x9E37_79B9;
        x ^= x << 13;
        x ^= x >> 17;
        x ^= x << 5;
        (x & 0xFFFF) as f32 / 32768.0 - 1.0
    };
    let env = |t: f32, len: f32, p: i32| (1.0 - t / len).max(0.0).powi(p);
    let tone = |t: f32, f: f32| (t * f * TAU).sin();
    let sq = |t: f32, f: f32| if (t * f).fract() < 0.5 { 1.0 } else { -1.0 };
    let arp = |t: f32, notes: &[f32], gap: f32, len: f32| {
        let mut v = 0.0;
        for (k, f) in notes.iter().enumerate() {
            let st = k as f32 * gap;
            if t >= st {
                v += (t * f * TAU).sin() * (1.0 - (t - st) / len).max(0.0).powi(2);
            }
        }
        v
    };
    let mut b: Vec<Vec<i16>> = Vec::new();
    // Click
    b.push(synth(28, |t| tone(t, 1400.0) * env(t, 0.028, 3) * 0.35));
    // Step: soft footfall
    b.push(synth(70, |t| (tone(t, 220.0 - t * 900.0) * 0.6 + noise(t * 44100.0) * 0.25) * env(t, 0.07, 3) * 0.55));
    // Hop: a creature moves
    b.push(synth(60, |t| tone(t, 520.0 + t * 2600.0) * env(t, 0.06, 2) * 0.22));
    // Catch: snap and a bright rising pair
    b.push(synth(260, |t| {
        let snap = noise(t * 44100.0) * env(t, 0.02, 2) * 0.6;
        snap + arp(t, &[659.25, 987.77], 0.06, 0.2) * 0.4
    }));
    // Hurt: low thud with a sour square
    b.push(synth(300, |t| (tone(t, 150.0 - t * 200.0) * 0.8 + sq(t, 233.0) * 0.25) * env(t, 0.3, 2) * 0.6));
    // Strike: a whoosh
    b.push(synth(140, |t| noise(t * 44100.0) * env(t, 0.14, 2) * (t / 0.14).min(1.0) * 0.45));
    // Deny
    b.push(synth(130, |t| sq(t, 110.0) * 0.22 * env(t, 0.13, 2)));
    // Dew: a droplet
    b.push(synth(180, |t| tone(t, 1800.0 + 900.0 * (-t * 30.0).exp()) * env(t, 0.18, 3) * 0.35));
    // Rest: a soft settling chord
    b.push(synth(360, |t| arp(t, &[392.0, 523.25], 0.05, 0.34) * 0.28));
    // Rustle: leaves
    b.push(synth(260, |t| noise(t * 44100.0) * (0.5 + 0.5 * tone(t, 18.0)) * env(t, 0.26, 1) * 0.3));
    // Arrive
    b.push(synth(200, |t| (tone(t, 300.0 + t * 600.0) * 0.5 + noise(t * 44100.0) * 0.2) * env(t, 0.2, 2) * 0.4));
    // Descend: falling arpeggio
    b.push(synth(700, |t| arp(t, &[783.99, 659.25, 523.25, 392.0], 0.11, 0.4) * 0.3));
    // Buy
    b.push(synth(260, |t| (tone(t, 1975.5) + tone(t, 2637.0) * if t > 0.06 { 1.0 } else { 0.0 }) * env(t, 0.26, 3) * 0.28));
    // Die
    b.push(synth(1300, |t| arp(t, &[392.0, 369.99, 349.23, 293.66], 0.22, 0.6) * 0.32));
    // Win
    b.push(synth(1500, |t| arp(t, &[523.25, 659.25, 783.99, 1046.5, 783.99, 1046.5, 1318.5], 0.13, 0.6) * 0.22));
    // Place
    b.push(synth(90, |t| (tone(t, 180.0) * 0.7 + noise(t * 44100.0) * 0.2 * env(t, 0.02, 1)) * env(t, 0.09, 3) * 0.6));
    // Lift
    b.push(synth(70, |t| tone(t, 600.0 + t * 3000.0) * env(t, 0.07, 2) * 0.3));
    // Persuade: a short vocal-ish blip
    b.push(synth(90, |t| (sq(t, 330.0 + 80.0 * tone(t, 40.0)) * 0.3 + tone(t, 660.0) * 0.3) * env(t, 0.09, 2) * 0.32));
    // Turn: someone changes sides
    b.push(synth(420, |t| arp(t, &[523.25, 698.46, 880.0, 1046.5], 0.07, 0.3) * 0.3));
    // Battle start: a horn
    b.push(synth(700, |t| (sq(t, 196.0) * 0.4 + tone(t, 392.0) * 0.4) * env(t, 0.7, 1) * (t / 0.05).min(1.0) * 0.3));
    // Battle won
    b.push(synth(1100, |t| arp(t, &[523.25, 659.25, 783.99, 1046.5], 0.12, 0.6) * 0.27));
    // Battle lost
    b.push(synth(1000, |t| arp(t, &[440.0, 415.3, 392.0, 329.63], 0.18, 0.55) * 0.28));
    // Key: a typing tick
    b.push(synth(14, |t| noise(t * 44100.0) * env(t, 0.014, 2) * 0.3));
    // Lesson: a gentle two-note chime
    b.push(synth(500, |t| arp(t, &[880.0, 1318.5], 0.1, 0.4) * 0.26));
    // Ward breaks
    b.push(synth(600, |t| (sq(t, 98.0) * 0.3 + noise(t * 44100.0) * 0.3) * env(t, 0.6, 2) * 0.5));
    // Block: shell rings
    b.push(synth(300, |t| (tone(t, 1200.0) + tone(t, 1810.0) * 0.6) * env(t, 0.3, 3) * 0.3));
    b.into_iter().map(Arc::new).collect()
}

fn synth(ms: usize, gen: impl Fn(f32) -> f32) -> Vec<i16> {
    let n = RATE as usize * ms / 1000;
    let mut lp = 0.0f32;
    (0..n)
        .map(|i| {
            let raw = gen(i as f32 / RATE as f32).clamp(-1.0, 1.0);
            lp = lp * 0.55 + raw * 0.45;
            (lp * SFX_PEAK) as i16
        })
        .collect()
}

#[repr(C)]
struct WaveFormatEx {
    w_format_tag: u16,
    n_channels: u16,
    n_samples_per_sec: u32,
    n_avg_bytes_per_sec: u32,
    n_block_align: u16,
    w_bits_per_sample: u16,
    cb_size: u16,
}

#[repr(C)]
struct WaveHdr {
    lp_data: *mut u8,
    dw_buffer_length: u32,
    dw_bytes_recorded: u32,
    dw_user: usize,
    dw_flags: u32,
    dw_loops: u32,
    lp_next: *mut WaveHdr,
    reserved: usize,
}

const WAVE_MAPPER: u32 = 0xFFFF_FFFF;
const WHDR_DONE: u32 = 0x1;

#[link(name = "winmm")]
extern "system" {
    fn waveOutOpen(phwo: *mut usize, dev: u32, fmt: *const WaveFormatEx, cb: usize, inst: usize, flags: u32) -> u32;
    fn waveOutPrepareHeader(hwo: usize, hdr: *mut WaveHdr, size: u32) -> u32;
    fn waveOutUnprepareHeader(hwo: usize, hdr: *mut WaveHdr, size: u32) -> u32;
    fn waveOutWrite(hwo: usize, hdr: *mut WaveHdr, size: u32) -> u32;
    fn waveOutReset(hwo: usize) -> u32;
    fn waveOutClose(hwo: usize) -> u32;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sfx_bank_matches_enum() {
        assert_eq!(build_sfx_bank().len(), Sfx::Block as usize + 1);
    }

    #[test]
    fn every_track_names_a_wav() {
        for t in [Track::Menu, Track::Doat, Track::Grof, Track::Prehistory, Track::Posthistory] {
            assert!(t.file().1.ends_with(".wav"));
        }
    }
}
