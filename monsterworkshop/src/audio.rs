//! Sound effects through winmm waveOut. No music: a mixer thread keeps a few short buffers
//! queued and adds every playing effect into them, so a push sounds within ~25 ms.

use crate::game::Sfx;
use std::f32::consts::TAU;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

const RATE: u64 = 44_100;
const NUM_BUFS: usize = 4;
const FRAMES_PER_BUF: usize = 512; // ~12 ms each

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
    quit: bool,
    sfx: Vec<(Arc<Vec<i16>>, usize)>,
}

pub struct Audio {
    shared: Arc<Mutex<Shared>>,
    bank: Vec<Arc<Vec<i16>>>,
}

impl Audio {
    pub fn new() -> Self {
        let shared = Arc::new(Mutex::new(Shared { quit: false, sfx: Vec::new() }));
        let s = Arc::clone(&shared);
        thread::spawn(move || mixer_thread(s));
        Self { shared, bank: build_sfx_bank() }
    }

    pub fn play_sfx(&self, sfx: Sfx) {
        if let (Some(buf), Ok(mut g)) = (self.bank.get(sfx as usize), self.shared.lock()) {
            if g.sfx.len() < 8 {
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
        if waveOutOpen(&mut hwo, WAVE_MAPPER, &fmt, 0, 0, 0) != 0 || hwo == 0 {
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

fn sine(t: f32, f: f32) -> f32 {
    (t * f * TAU).sin()
}

/// Deterministic white noise from the sample index.
fn noise(t: f32) -> f32 {
    let mut h = (t * RATE as f32) as u32;
    h ^= h >> 16;
    h = h.wrapping_mul(0x7feb_352d);
    h ^= h >> 15;
    h = h.wrapping_mul(0x846c_a68b);
    h ^= h >> 16;
    (h as f32 / u32::MAX as f32) * 2.0 - 1.0
}

fn decay(t: f32, len: f32, pow: i32) -> f32 {
    (1.0 - t / len).max(0.0).powi(pow)
}

/// Notes in sequence, each `step` seconds apart, ringing until `len`.
fn arp(t: f32, notes: &[f32], step: f32, len: f32) -> f32 {
    notes
        .iter()
        .enumerate()
        .filter(|(i, _)| t >= *i as f32 * step)
        .map(|(i, &f)| {
            let lt = t - i as f32 * step;
            (sine(lt, f) + 0.25 * sine(lt, f * 2.0)) * decay(lt, len - i as f32 * step, 2)
        })
        .sum()
}

/// One buffer per `Sfx` variant, in declaration order.
fn build_sfx_bank() -> Vec<Arc<Vec<i16>>> {
    let bank: Vec<Vec<i16>> = vec![
        // Step: a soft padded footfall.
        synth(40, |t| (noise(t) * 0.25 + sine(t, 90.0)) * decay(t, 0.04, 3) * 0.35),
        // Push: a short scrape over a low thump.
        synth(110, |t| (noise(t) * 0.45 * decay(t, 0.11, 1) + sine(t, 120.0 - t * 300.0) * decay(t, 0.08, 2)) * 0.6),
        // Bump: dull thud.
        synth(130, |t| sine(t, 72.0 - t * 120.0) * decay(t, 0.13, 2) * 0.9),
        // Pop: harvesting a boll, a rising bloop.
        synth(130, |t| sine(t, 380.0 + t * 4200.0) * decay(t, 0.13, 2) * 0.5),
        // Plop: the chute drops the next item.
        synth(110, |t| sine(t, 620.0 - t * 3200.0) * decay(t, 0.11, 2) * 0.45),
        // Pad: an item lands on a workshop mat.
        synth(70, |t| (sine(t, 1180.0) + 0.5 * sine(t, 1770.0)) * decay(t, 0.07, 3) * 0.4),
        // Spin: one turn of the wheel.
        synth(90, |t| (sine(t, 210.0 + 30.0 * sine(t, 18.0)) * 0.6 + noise(t) * 0.15) * decay(t, 0.09, 1) * 0.3),
        // Chime: thread is done.
        synth(650, |t| arp(t, &[1046.5, 1318.5, 1568.0, 2093.0], 0.07, 0.65) * 0.22),
        // Deposit: two-note coin into a bin.
        synth(220, |t| arp(t, &[987.8, 1318.5], 0.06, 0.22) * 0.32),
        // RecOn
        synth(200, |t| arp(t, &[659.3, 659.3], 0.1, 0.2) * 0.3),
        // RecSave
        synth(330, |t| arp(t, &[523.3, 659.3, 784.0], 0.08, 0.33) * 0.3),
        // LoopOn
        synth(240, |t| arp(t, &[587.3, 880.0], 0.08, 0.24) * 0.3),
        // LoopOff
        synth(240, |t| arp(t, &[880.0, 587.3], 0.08, 0.24) * 0.3),
        // Switch
        synth(60, |t| sine(t, 1480.0) * decay(t, 0.06, 2) * 0.3),
        // Deny: soft low buzz.
        synth(170, |t| (sine(t, 150.0) + 0.5 * sine(t, 300.0)).signum() * decay(t, 0.17, 1) * 0.12),
    ];
    bank.into_iter().map(Arc::new).collect()
}

fn synth(ms: usize, gen: impl Fn(f32) -> f32) -> Vec<i16> {
    let n = RATE as usize * ms / 1000;
    let mut lp = 0.0f32;
    (0..n)
        .map(|i| {
            let raw = gen(i as f32 / RATE as f32).clamp(-1.0, 1.0);
            lp = lp * 0.55 + raw * 0.45;
            (lp * 2600.0) as i16
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bank_has_one_buffer_per_sfx() {
        assert_eq!(build_sfx_bank().len(), Sfx::ALL.len());
        for (i, s) in Sfx::ALL.iter().enumerate() {
            assert_eq!(*s as usize, i);
        }
    }
}
