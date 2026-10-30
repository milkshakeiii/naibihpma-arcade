//! Streams one WAV through winmm waveOut and reports the song clock from the device's own
//! playback position, so the world never drifts from the music over a 6-minute track.
//! Without a track (or an audio device) the clock falls back to wall time.

use crate::game::Sfx;
use std::fs::File;
use std::io::{BufReader, Read, Seek, SeekFrom};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};

const RATE: u64 = 44_100;
const NUM_BUFS: usize = 6;
const FRAMES_PER_BUF: usize = 1024; // ~23 ms each

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

#[repr(C)]
struct MmTime {
    w_type: u32,
    value: u32,
    pad: u32,
}

const WAVE_MAPPER: u32 = 0xFFFF_FFFF;
const WHDR_DONE: u32 = 0x1;
const TIME_SAMPLES: u32 = 0x2;
const TIME_BYTES: u32 = 0x4;

#[link(name = "winmm")]
extern "system" {
    fn waveOutOpen(phwo: *mut usize, dev: u32, fmt: *const WaveFormatEx, cb: usize, inst: usize, flags: u32) -> u32;
    fn waveOutPrepareHeader(hwo: usize, hdr: *mut WaveHdr, size: u32) -> u32;
    fn waveOutUnprepareHeader(hwo: usize, hdr: *mut WaveHdr, size: u32) -> u32;
    fn waveOutWrite(hwo: usize, hdr: *mut WaveHdr, size: u32) -> u32;
    fn waveOutGetPosition(hwo: usize, mmt: *mut MmTime, size: u32) -> u32;
    fn waveOutReset(hwo: usize) -> u32;
    fn waveOutClose(hwo: usize) -> u32;
}

struct Shared {
    device_ok: Option<bool>,
    play: Option<(u32, u64)>, // (generation, start frame)
    stop: bool,
    quit: bool,
    sfx: Vec<(Arc<Vec<i16>>, usize)>,
    pos_gen: u32,
    pos_frames: u64,
}

pub struct Audio {
    shared: Arc<Mutex<Shared>>,
    pub track: Option<PathBuf>,
    track_frames: u64,
    device_ok: bool,
    gen: u32,
    start_ms: u32,
    playing: bool,
    wall_origin: Instant,
    bank: Vec<Arc<Vec<i16>>>,
}

impl Audio {
    pub fn new(track: Option<PathBuf>) -> Self {
        let track = track.filter(|p| WavStream::open(p).is_some());
        let track_frames = track.as_ref().and_then(|p| WavStream::open(p)).map_or(0, |s| s.frames);
        let shared = Arc::new(Mutex::new(Shared {
            device_ok: None,
            play: None,
            stop: false,
            quit: false,
            sfx: Vec::new(),
            pos_gen: 0,
            pos_frames: 0,
        }));
        let (t, s) = (track.clone(), Arc::clone(&shared));
        thread::spawn(move || mixer_thread(t, s));
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
        Self {
            shared,
            track,
            track_frames,
            device_ok,
            gen: 0,
            start_ms: 0,
            playing: false,
            wall_origin: Instant::now(),
            bank: build_sfx_bank(),
        }
    }

    /// The song is audible and clocked by the device.
    pub fn has_music(&self) -> bool {
        self.track.is_some() && self.device_ok
    }

    pub fn track_len_ms(&self) -> Option<u32> {
        self.track.as_ref().map(|_| (self.track_frames * 1000 / RATE) as u32)
    }

    pub fn play_from(&mut self, ms: u32) {
        self.gen = self.gen.wrapping_add(1);
        self.start_ms = ms;
        self.playing = true;
        self.wall_origin = Instant::now();
        if let Ok(mut g) = self.shared.lock() {
            g.play = Some((self.gen, ms as u64 * RATE / 1000));
        }
    }

    pub fn stop(&mut self) {
        self.playing = false;
        if let Ok(mut g) = self.shared.lock() {
            g.stop = true;
        }
    }

    pub fn song_ms(&self) -> u32 {
        if !self.playing {
            return self.start_ms;
        }
        if !self.has_music() {
            return self.start_ms + self.wall_origin.elapsed().as_millis() as u32;
        }
        match self.shared.lock() {
            Ok(g) if g.pos_gen == self.gen => (g.pos_frames * 1000 / RATE) as u32,
            _ => self.start_ms,
        }
    }

    pub fn play_sfx(&self, sfx: Sfx) {
        let idx = match sfx {
            Sfx::Pluck { note, big } => (note % 5) as usize + if big { 5 } else { 0 },
            Sfx::Echo => 10,
        };
        if let (Some(buf), Ok(mut g)) = (self.bank.get(idx), self.shared.lock()) {
            if g.sfx.len() < 6 {
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

    fn seek_frame(&mut self, frame: u64) {
        self.pos = frame.min(self.frames);
        let _ = self.reader.seek(SeekFrom::Start(self.data_start + self.pos * self.frame_bytes() as u64));
    }

    /// Fills interleaved stereo i16; silence after the end (the song does not loop).
    fn read(&mut self, out: &mut [i16]) {
        out.fill(0);
        let want = (out.len() / 2).min((self.frames - self.pos) as usize);
        let fb = self.frame_bytes();
        self.raw.resize(want * fb, 0);
        if want == 0 || self.reader.read_exact(&mut self.raw).is_err() {
            self.pos = self.frames;
            return;
        }
        self.pos += want as u64;
        let sample = |raw: &[u8], at: usize| -> i16 {
            match (self.bits, self.float) {
                (16, _) => i16::from_le_bytes([raw[at], raw[at + 1]]),
                (24, _) => i16::from_le_bytes([raw[at + 1], raw[at + 2]]),
                _ => {
                    let v = f32::from_le_bytes([raw[at], raw[at + 1], raw[at + 2], raw[at + 3]]);
                    (v.clamp(-1.0, 1.0) * 32767.0) as i16
                }
            }
        };
        let bps = self.bits as usize / 8;
        for i in 0..want {
            let base = i * fb;
            let l = sample(&self.raw, base);
            let r = if self.channels > 1 { sample(&self.raw, base + bps) } else { l };
            out[i * 2] = l;
            out[i * 2 + 1] = r;
        }
    }
}

fn mixer_thread(track: Option<PathBuf>, shared: Arc<Mutex<Shared>>) {
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
        let (mut gen, mut base, mut playing) = (0u32, 0u64, false);
        loop {
            let (play, stop, quit) = match shared.lock() {
                Ok(mut g) => (g.play.take(), std::mem::take(&mut g.stop), g.quit),
                Err(_) => break,
            };
            if quit {
                break;
            }
            if let Some((g, from)) = play {
                waveOutReset(hwo);
                stream = track.as_deref().and_then(WavStream::open);
                if let Some(s) = stream.as_mut() {
                    s.seek_frame(from);
                }
                (gen, base, playing) = (g, from, true);
            } else if stop {
                waveOutReset(hwo);
                playing = false;
            }

            let mut queued = false;
            for i in 0..NUM_BUFS {
                if hdrs[i].dw_flags & WHDR_DONE == 0 {
                    continue;
                }
                match (playing, stream.as_mut()) {
                    (true, Some(s)) => s.read(&mut bufs[i]),
                    _ => bufs[i].fill(0),
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

            if playing {
                let mut mt = MmTime { w_type: TIME_SAMPLES, value: 0, pad: 0 };
                waveOutGetPosition(hwo, &mut mt, std::mem::size_of::<MmTime>() as u32);
                let frames = match mt.w_type {
                    TIME_SAMPLES => mt.value as u64,
                    TIME_BYTES => mt.value as u64 / 4,
                    _ => 0,
                };
                if let Ok(mut g) = shared.lock() {
                    g.pos_gen = gen;
                    g.pos_frames = base + frames;
                }
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

/// Soft plucks on a Bb major pentatonic (fits the F/Bb canopy) plus a quiet echo shimmer.
fn build_sfx_bank() -> Vec<Arc<Vec<i16>>> {
    const NOTES: [f32; 5] = [466.16, 523.25, 587.33, 698.46, 783.99];
    let mut bank = Vec::new();
    for big in [false, true] {
        for f in NOTES {
            let (freq, ms) = if big { (f / 2.0, 420) } else { (f, 240) };
            bank.push(Arc::new(synth(ms, |t| {
                let env = (1.0 - t / (ms as f32 / 1000.0)).max(0.0).powi(3);
                let body = (t * freq * std::f32::consts::TAU).sin() + 0.3 * (t * freq * 2.0 * std::f32::consts::TAU).sin();
                let fifth = if big { 0.35 * (t * freq * 1.5 * std::f32::consts::TAU).sin() } else { 0.0 };
                (body + fifth) * env * 0.55
            })));
        }
    }
    bank.push(Arc::new(synth(700, |t| {
        let env = (1.0 - t / 0.7).max(0.0).powi(2);
        let chord = [1396.91f32, 1864.66, 2349.32]
            .iter()
            .enumerate()
            .map(|(i, f)| if t >= i as f32 * 0.06 { (t * f * std::f32::consts::TAU).sin() } else { 0.0 })
            .sum::<f32>();
        chord * env * 0.22
    })));
    bank
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Needs an audio device and the Underbrush master: `cargo test -- --ignored`.
    #[test]
    #[ignore]
    fn device_clock_follows_playback_and_seeks() {
        let path = PathBuf::from(
            r"C:\Users\User\Desktop\albums\s3_thenechantedforestofnaibihpma\mastered_resonance\1_the_underbrush_master.wav",
        );
        let mut a = Audio::new(Some(path));
        assert!(a.has_music());
        assert_eq!(a.track_len_ms(), Some(352_000));
        a.play_from(0);
        thread::sleep(Duration::from_millis(1500));
        let t = a.song_ms();
        assert!((1200..=1700).contains(&t), "clock after 1.5s: {t}ms");
        a.play_from(200_000);
        assert_eq!(a.song_ms(), 200_000, "stale positions are ignored after a seek");
        thread::sleep(Duration::from_millis(500));
        let t = a.song_ms();
        assert!((200_300..=200_700).contains(&t), "clock after seek: {t}ms");
        a.stop();
    }
}

fn synth(ms: usize, gen: impl Fn(f32) -> f32) -> Vec<i16> {
    let n = RATE as usize * ms / 1000;
    let mut lp = 0.0f32;
    (0..n)
        .map(|i| {
            let raw = gen(i as f32 / RATE as f32).clamp(-1.0, 1.0);
            lp = lp * 0.6 + raw * 0.4;
            (lp * 2600.0) as i16
        })
        .collect()
}
