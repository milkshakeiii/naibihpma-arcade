//! Sound effects through winmm waveOut. Small buffers keep the delay from key press to click
//! short; every sound is synthesized at startup (no audio files).

use crate::game::Sfx;
use std::f32::consts::TAU;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

const RATE: usize = 44_100;
const NUM_BUFS: usize = 4;
const FRAMES_PER_BUF: usize = 512; // ~11.6 ms each

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

struct Shared {
    voices: Vec<(Arc<Vec<i16>>, usize)>,
    quit: bool,
}

pub struct Audio {
    shared: Arc<Mutex<Shared>>,
    bank: Vec<Arc<Vec<i16>>>,
}

impl Audio {
    pub fn new() -> Self {
        let shared = Arc::new(Mutex::new(Shared { voices: Vec::new(), quit: false }));
        let s = Arc::clone(&shared);
        thread::spawn(move || mixer_thread(s));
        Self { shared, bank: build_bank() }
    }

    pub fn play(&self, sfx: Sfx) {
        let i = match sfx {
            Sfx::Tick => 0,
            Sfx::Place => 1,
            Sfx::Rotate => 2,
            Sfx::Remove => 3,
            Sfx::Deny => 4,
            Sfx::Draw => 5,
            Sfx::RunStart => 6,
            Sfx::Emit => 7,
            Sfx::Paint => 8,
            Sfx::Switch => 9,
            Sfx::Clink(n) => 10 + (n as usize % 5),
            Sfx::Lock => 15,
            Sfx::Release => 16,
            Sfx::Note(n) => 17 + (n as usize % 5),
            Sfx::Bad => 22,
            Sfx::Won => 23,
            Sfx::Turn => 24,
        };
        if let (Some(buf), Ok(mut g)) = (self.bank.get(i), self.shared.lock()) {
            if g.voices.len() < 8 {
                g.voices.push((Arc::clone(buf), 0));
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

fn mixer_thread(shared: Arc<Mutex<Shared>>) {
    unsafe {
        let fmt = WaveFormatEx {
            w_format_tag: 1,
            n_channels: 1,
            n_samples_per_sec: RATE as u32,
            n_avg_bytes_per_sec: RATE as u32 * 2,
            n_block_align: 2,
            w_bits_per_sample: 16,
            cb_size: 0,
        };
        let mut hwo = 0usize;
        if waveOutOpen(&mut hwo, WAVE_MAPPER, &fmt, 0, 0, 0) != 0 || hwo == 0 {
            return;
        }
        let hdr_size = std::mem::size_of::<WaveHdr>() as u32;
        let mut bufs: Vec<Vec<i16>> = (0..NUM_BUFS).map(|_| vec![0i16; FRAMES_PER_BUF]).collect();
        let mut hdrs: Vec<WaveHdr> = bufs
            .iter_mut()
            .map(|b| WaveHdr {
                lp_data: b.as_mut_ptr() as *mut u8,
                dw_buffer_length: (FRAMES_PER_BUF * 2) as u32,
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
        loop {
            if shared.lock().map_or(true, |g| g.quit) {
                break;
            }
            let mut queued = false;
            for i in 0..NUM_BUFS {
                if hdrs[i].dw_flags & WHDR_DONE == 0 {
                    continue;
                }
                bufs[i].fill(0);
                if let Ok(mut g) = shared.lock() {
                    for (samples, cursor) in g.voices.iter_mut() {
                        for f in 0..FRAMES_PER_BUF {
                            let Some(&v) = samples.get(*cursor) else { break };
                            *cursor += 1;
                            let o = &mut bufs[i][f];
                            *o = (*o as i32 + v as i32).clamp(-32000, 32000) as i16;
                        }
                    }
                    g.voices.retain(|(s, c)| *c < s.len());
                }
                hdrs[i].dw_flags &= !WHDR_DONE;
                waveOutWrite(hwo, &mut hdrs[i], hdr_size);
                queued = true;
            }
            if !queued {
                thread::sleep(Duration::from_millis(2));
            }
        }
        waveOutReset(hwo);
        for h in hdrs.iter_mut() {
            waveOutUnprepareHeader(hwo, h, hdr_size);
        }
        waveOutClose(hwo);
    }
}

/// Deterministic white noise in -1..1.
fn noise(i: usize) -> f32 {
    let mut x = (i as u32).wrapping_mul(0x9E37_79B9) ^ 0x85EB_CA6B;
    x ^= x >> 15;
    x = x.wrapping_mul(0x2C1B_3C6D);
    x ^= x >> 12;
    (x & 0xFFFF) as f32 / 32768.0 - 1.0
}

fn synth(ms: usize, gain: f32, gen: impl Fn(f32, usize) -> f32) -> Arc<Vec<i16>> {
    let n = RATE * ms / 1000;
    let mut lp = 0.0f32;
    Arc::new(
        (0..n)
            .map(|i| {
                let raw = gen(i as f32 / RATE as f32, i).clamp(-1.0, 1.0);
                lp = lp * 0.5 + raw * 0.5;
                (lp * gain * 9000.0) as i16
            })
            .collect(),
    )
}

fn env(t: f32, len: f32, pow: i32) -> f32 {
    (1.0 - t / len).max(0.0).powi(pow)
}

/// C major pentatonic, so any run of bins rings without clashing.
const NOTES: [f32; 5] = [523.25, 587.33, 659.25, 783.99, 880.0];

fn build_bank() -> Vec<Arc<Vec<i16>>> {
    let mut b = Vec::new();
    b.push(synth(12, 0.25, |t, i| noise(i) * env(t, 0.012, 2))); // tick
    b.push(synth(80, 0.9, |t, i| ((t * 170.0 * TAU).sin() * 0.8 + noise(i) * 0.25 * env(t, 0.02, 1)) * env(t, 0.08, 3))); // place
    b.push(synth(30, 0.45, |t, _| (t * 1250.0 * TAU).sin() * env(t, 0.03, 3))); // rotate
    b.push(synth(90, 0.5, |t, _| (t * (600.0 - t * 3000.0) * TAU).sin() * env(t, 0.09, 2))); // remove
    b.push(synth(130, 0.45, |t, _| ((t * 110.0 * TAU).sin().signum() * 0.4) * env(t, 0.13, 2))); // deny
    b.push(synth(18, 0.3, |t, _| (t * 900.0 * TAU).sin() * env(t, 0.018, 2))); // draw
    b.push(synth(260, 0.5, |t, _| (t * (220.0 + t * 900.0) * TAU).sin() * (1.0 - (t * 30.0 * TAU).sin() * 0.3) * env(t, 0.26, 1))); // run
    b.push(synth(25, 0.2, |t, _| (t * 300.0 * TAU).sin() * env(t, 0.025, 2))); // emit
    b.push(synth(110, 0.35, |t, i| noise(i) * env(t, 0.11, 2) * (0.5 + 0.5 * (t * 40.0 * TAU).sin()))); // paint
    b.push(synth(45, 0.4, |t, _| ((t * 2200.0 * TAU).sin() + (t * 3150.0 * TAU).sin()) * 0.5 * env(t, 0.045, 4))); // switch
    for k in 0..5 {
        let f = 2300.0 + k as f32 * 220.0;
        b.push(synth(140, 0.35, move |t, _| ((t * f * TAU).sin() + 0.4 * (t * f * 2.7 * TAU).sin()) * env(t, 0.14, 3))); // clink
    }
    b.push(synth(170, 0.9, |t, i| ((t * 80.0 * TAU).sin() + noise(i) * 0.3) * env(t, 0.17, 3))); // lock
    b.push(synth(40, 0.4, |t, _| (t * 480.0 * TAU).sin() * env(t, 0.04, 2))); // release
    for f in NOTES {
        b.push(synth(420, 0.55, move |t, _| ((t * f * TAU).sin() + 0.3 * (t * f * 2.0 * TAU).sin() + 0.15 * (t * f * 3.0 * TAU).sin()) * env(t, 0.42, 3))); // bin note
    }
    b.push(synth(240, 0.8, |t, i| ((t * (90.0 - t * 120.0) * TAU).sin() + noise(i) * 0.2) * env(t, 0.24, 2))); // bad
    b.push(synth(900, 0.45, |t, _| {
        let mut v = 0.0;
        for (k, f) in [523.25f32, 659.25, 783.99, 1046.5].iter().enumerate() {
            let start = k as f32 * 0.09;
            if t >= start {
                v += (t * f * TAU).sin() * env(t - start, 0.8, 2);
            }
        }
        v * 0.5
    })); // won
    b.push(synth(14, 0.18, |t, _| (t * 1600.0 * TAU).sin() * env(t, 0.014, 2))); // turn
    b
}
