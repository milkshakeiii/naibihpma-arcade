use std::fs::File;
use std::io::{BufReader, Read, Seek, SeekFrom};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

#[derive(Clone, Copy)]
pub enum Sfx {
    SuctionStick,
    SuctionLaunch,
    ToadBounce,
    HarvestPluck,
    BombExplode,
    ElectricZap,
    CleaverChop,
    WokSizzle,
    ServiceBell,
    CoinStart,
    Hurt,
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

const WAVE_FORMAT_PCM: u16 = 1;
const WAVE_MAPPER: u32 = 0xFFFF_FFFF;
const WHDR_DONE: u32 = 0x0000_0001;

#[link(name = "winmm")]
extern "system" {
    fn waveOutOpen(
        phwo: *mut usize,
        u_device_id: u32,
        pwfx: *const WaveFormatEx,
        dw_callback: usize,
        dw_instance: usize,
        fdw_open: u32,
    ) -> u32;
    fn waveOutPrepareHeader(hwo: usize, pwh: *mut WaveHdr, cbwh: u32) -> u32;
    fn waveOutWrite(hwo: usize, pwh: *mut WaveHdr, cbwh: u32) -> u32;
    fn waveOutUnprepareHeader(hwo: usize, pwh: *mut WaveHdr, cbwh: u32) -> u32;
    fn waveOutReset(hwo: usize) -> u32;
    fn waveOutClose(hwo: usize) -> u32;
}

struct SharedAudioCommand {
    desired_track: usize,
    track_changed: bool,
    sfx_queue: Vec<(Vec<i16>, usize)>, // (mono 44.1kHz samples, cursor)
    sfx_muted: bool,
    stop: bool,
}

pub struct ArcadeAudio {
    pub tracks: Vec<PathBuf>,
    pub current_track: usize,
    pub muted: bool,
    shared: Option<Arc<Mutex<SharedAudioCommand>>>,
    // Pre-synthesized soft, low-passed 44.1kHz mono SFX buffers
    sfx_bank: Vec<Vec<i16>>,
}

impl ArcadeAudio {
    pub fn new_muted() -> Self {
        Self {
            tracks: Vec::new(),
            current_track: 0,
            muted: true,
            shared: None,
            sfx_bank: Vec::new(),
        }
    }

    pub fn new() -> Self {
        let mut tracks = Vec::new();
        let search_dirs = [
            r"assets\music",
            r"creaturecollector\assets\music",
            r"C:\Users\User\Desktop\theenchantedforestofnaibihpma",
        ];
        for dir in &search_dirs {
            let p = Path::new(dir);
            if let Ok(entries) = std::fs::read_dir(p) {
                for entry in entries.flatten() {
                    let path = entry.path();
                    if path.is_file() {
                        if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
                            if ext.eq_ignore_ascii_case("wav") {
                                // Avoid duplicates if present in both folders
                                let fname = path.file_name().map(|f| f.to_os_string());
                                let already = tracks.iter().any(|t: &PathBuf| {
                                    t.file_name().map(|f| f.to_os_string()) == fname
                                });
                                if !already {
                                    tracks.push(path);
                                }
                            }
                        }
                    }
                }
            }
        }

        // Order tracks in canonical Bandcamp album order for The Enchanted Forest of Naibihpma
        let stage_order = [
            "the_underbrush",
            "doat_the_enchanter",
            "twen_dew_collector",
            "star_pond",
            "grof_malevolent_sorcerer",
            "ancient_magic",
        ];
        tracks.sort_by_key(|p| {
            let stem = p
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("")
                .to_ascii_lowercase();
            stage_order
                .iter()
                .position(|&s| stem.contains(s))
                .unwrap_or(99)
        });

        let sfx_bank = build_gentle_sfx_bank();

        let shared = Arc::new(Mutex::new(SharedAudioCommand {
            desired_track: 0,
            track_changed: true,
            sfx_queue: Vec::new(),
            sfx_muted: false,
            stop: false,
        }));

        let tracks_clone = tracks.clone();
        let shared_clone = Arc::clone(&shared);
        thread::spawn(move || {
            run_waveout_mixer_thread(tracks_clone, shared_clone);
        });

        Self {
            tracks,
            current_track: 0,
            muted: false,
            shared: Some(shared),
            sfx_bank,
        }
    }

    pub fn current_track_title(&self) -> String {
        if self.tracks.is_empty() {
            "NO WAV IN ASSETS/MUSIC".to_string()
        } else {
            let name = self.tracks[self.current_track]
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("TRACK")
                .replace('_', " ");
            let mut up = name.to_ascii_uppercase();
            if up.len() > 24 {
                up.truncate(24);
            }
            format!("{}/{} {}", self.current_track + 1, self.tracks.len(), up)
        }
    }

    pub fn set_track(&mut self, idx: usize) {
        if !self.tracks.is_empty() {
            let target = idx % self.tracks.len();
            if self.current_track != target {
                self.current_track = target;
                if let Some(ref s) = self.shared {
                    if let Ok(mut guard) = s.lock() {
                        guard.desired_track = self.current_track;
                        guard.track_changed = true;
                    }
                }
            }
        }
    }

    pub fn next_track(&mut self) {
        if !self.tracks.is_empty() {
            self.current_track = (self.current_track + 1) % self.tracks.len();
            if let Some(ref s) = self.shared {
                if let Ok(mut guard) = s.lock() {
                    guard.desired_track = self.current_track;
                    guard.track_changed = true;
                }
            }
        }
    }

    pub fn prev_track(&mut self) {
        if !self.tracks.is_empty() {
            self.current_track = (self.current_track + self.tracks.len() - 1) % self.tracks.len();
            if let Some(ref s) = self.shared {
                if let Ok(mut guard) = s.lock() {
                    guard.desired_track = self.current_track;
                    guard.track_changed = true;
                }
            }
        }
    }

    pub fn toggle_sfx_mute(&mut self) {
        self.muted = !self.muted;
        if let Some(ref s) = self.shared {
            if let Ok(mut guard) = s.lock() {
                guard.sfx_muted = self.muted;
            }
        }
    }

    pub fn play_sfx(&self, sfx: Sfx) {
        if self.muted {
            return;
        }
        let idx = sfx as usize;
        if let Some(buf) = self.sfx_bank.get(idx) {
            if let Some(ref s) = self.shared {
                if let Ok(mut guard) = s.lock() {
                    if guard.sfx_queue.len() < 4 {
                        guard.sfx_queue.push((buf.clone(), 0));
                    }
                }
            }
        }
    }
}

impl Drop for ArcadeAudio {
    fn drop(&mut self) {
        if let Some(ref s) = self.shared {
            if let Ok(mut guard) = s.lock() {
                guard.stop = true;
            }
        }
    }
}

struct WavStreamReader {
    reader: BufReader<File>,
    data_start: u64,
    data_len: u64,
    data_pos: u64,
    channels: u16,
    bits_per_sample: u16,
    raw_buf: Vec<u8>,
}

impl WavStreamReader {
    fn open(path: &Path) -> Option<Self> {
        let mut f = File::open(path).ok()?;
        let mut header = [0u8; 12];
        f.read_exact(&mut header).ok()?;
        if &header[0..4] != b"RIFF" || &header[8..12] != b"WAVE" {
            return None;
        }

        let mut channels = 2u16;
        let mut bits_per_sample = 24u16;
        let mut data_start = 0u64;
        let mut data_len = 0u64;

        let mut pos = 12u64;
        loop {
            f.seek(SeekFrom::Start(pos)).ok()?;
            let mut chunk_hdr = [0u8; 8];
            if f.read_exact(&mut chunk_hdr).is_err() {
                break;
            }
            let cid = &chunk_hdr[0..4];
            let csize = u32::from_le_bytes([
                chunk_hdr[4],
                chunk_hdr[5],
                chunk_hdr[6],
                chunk_hdr[7],
            ]) as u64;

            if cid == b"fmt " && csize >= 16 {
                let mut fmt_buf = [0u8; 16];
                f.read_exact(&mut fmt_buf).ok()?;
                channels = u16::from_le_bytes([fmt_buf[2], fmt_buf[3]]);
                bits_per_sample = u16::from_le_bytes([fmt_buf[14], fmt_buf[15]]);
            } else if cid == b"data" {
                data_start = pos + 8;
                data_len = csize;
                break;
            }

            pos += 8 + csize + (csize & 1);
        }

        if data_start == 0 || data_len == 0 {
            return None;
        }

        f.seek(SeekFrom::Start(data_start)).ok()?;
        Some(Self {
            reader: BufReader::with_capacity(32768, f),
            data_start,
            data_len,
            data_pos: 0,
            channels,
            bits_per_sample,
            raw_buf: Vec::new(),
        })
    }

    /// Fills `out_stereo_i16` (interleaved L, R at 44.1kHz) with decoded PCM audio, looping seamlessly
    fn fill_stereo_frames(&mut self, out_stereo: &mut [i16]) {
        let frames = out_stereo.len() / 2;
        let bytes_per_sample = (self.bits_per_sample / 8).max(1) as usize;
        let frame_bytes = bytes_per_sample * (self.channels.max(1) as usize);
        let needed_bytes = frames * frame_bytes;

        if self.raw_buf.len() < needed_bytes {
            self.raw_buf.resize(needed_bytes, 0);
        }

        let mut filled = 0usize;
        while filled < needed_bytes {
            let rem_in_file = (self.data_len.saturating_sub(self.data_pos)) as usize;
            if rem_in_file == 0 {
                let _ = self.reader.seek(SeekFrom::Start(self.data_start));
                self.data_pos = 0;
                continue;
            }
            let to_read = (needed_bytes - filled).min(rem_in_file);
            match self.reader.read(&mut self.raw_buf[filled..filled + to_read]) {
                Ok(0) => {
                    let _ = self.reader.seek(SeekFrom::Start(self.data_start));
                    self.data_pos = 0;
                }
                Ok(n) => {
                    filled += n;
                    self.data_pos += n as u64;
                }
                Err(_) => break,
            }
        }

        for i in 0..frames {
            let base = i * frame_bytes;
            let (l, r) = match (self.bits_per_sample, self.channels) {
                (24, 2) => {
                    // 24-bit signed little-endian: take upper 16 bits (bytes 1 and 2)
                    let left = i16::from_le_bytes([self.raw_buf[base + 1], self.raw_buf[base + 2]]);
                    let right =
                        i16::from_le_bytes([self.raw_buf[base + 4], self.raw_buf[base + 5]]);
                    (left, right)
                }
                (16, 2) => {
                    let left = i16::from_le_bytes([self.raw_buf[base], self.raw_buf[base + 1]]);
                    let right =
                        i16::from_le_bytes([self.raw_buf[base + 2], self.raw_buf[base + 3]]);
                    (left, right)
                }
                (24, 1) => {
                    let mono = i16::from_le_bytes([self.raw_buf[base + 1], self.raw_buf[base + 2]]);
                    (mono, mono)
                }
                (16, 1) => {
                    let mono = i16::from_le_bytes([self.raw_buf[base], self.raw_buf[base + 1]]);
                    (mono, mono)
                }
                _ => (0, 0),
            };
            out_stereo[i * 2] = l;
            out_stereo[i * 2 + 1] = r;
        }
    }
}

fn run_waveout_mixer_thread(tracks: Vec<PathBuf>, shared: Arc<Mutex<SharedAudioCommand>>) {
    unsafe {
        let fmt = WaveFormatEx {
            w_format_tag: WAVE_FORMAT_PCM,
            n_channels: 2,
            n_samples_per_sec: 44100,
            n_avg_bytes_per_sec: 44100 * 4,
            n_block_align: 4,
            w_bits_per_sample: 16,
            cb_size: 0,
        };

        let mut hwo: usize = 0;
        if waveOutOpen(&mut hwo, WAVE_MAPPER, &fmt, 0, 0, 0) != 0 || hwo == 0 {
            return;
        }

        const NUM_BUFS: usize = 4;
        const FRAMES_PER_BUF: usize = 2048; // ~46ms per buffer at 44.1kHz
        const SAMPLES_PER_BUF: usize = FRAMES_PER_BUF * 2;

        let mut audio_bufs: Vec<Vec<i16>> = (0..NUM_BUFS)
            .map(|_| vec![0i16; SAMPLES_PER_BUF])
            .collect();
        let mut hdrs: Vec<WaveHdr> = (0..NUM_BUFS)
            .map(|i| WaveHdr {
                lp_data: audio_bufs[i].as_mut_ptr() as *mut u8,
                dw_buffer_length: (SAMPLES_PER_BUF * 2) as u32,
                dw_bytes_recorded: 0,
                dw_user: 0,
                dw_flags: WHDR_DONE,
                dw_loops: 0,
                lp_next: std::ptr::null_mut(),
                reserved: 0,
            })
            .collect();

        for i in 0..NUM_BUFS {
            waveOutPrepareHeader(hwo, &mut hdrs[i], std::mem::size_of::<WaveHdr>() as u32);
            hdrs[i].dw_flags |= WHDR_DONE;
        }

        let mut current_stream: Option<WavStreamReader> = None;

        loop {
            {
                let mut guard = match shared.lock() {
                    Ok(g) => g,
                    Err(_) => break,
                };
                if guard.stop {
                    break;
                }
                if guard.track_changed {
                    guard.track_changed = false;
                    if !tracks.is_empty() {
                        let idx = guard.desired_track % tracks.len();
                        current_stream = WavStreamReader::open(&tracks[idx]);
                    }
                }
            }

            let mut any_queued = false;
            for i in 0..NUM_BUFS {
                if (hdrs[i].dw_flags & WHDR_DONE) != 0 {
                    // 1. Fill buffer with music from current 24-bit WAV track
                    if let Some(ref mut stream) = current_stream {
                        stream.fill_stereo_frames(&mut audio_bufs[i]);
                    } else {
                        audio_bufs[i].fill(0);
                    }

                    // 2. Mix any active gentle SFX on top
                    if let Ok(mut guard) = shared.lock() {
                        if !guard.sfx_muted && !guard.sfx_queue.is_empty() {
                            for f in 0..FRAMES_PER_BUF {
                                let mut sfx_sum = 0i32;
                                for (samples, cursor) in &mut guard.sfx_queue {
                                    if *cursor < samples.len() {
                                        sfx_sum += samples[*cursor] as i32;
                                        *cursor += 1;
                                    }
                                }
                                if sfx_sum != 0 {
                                    let l = (audio_bufs[i][f * 2] as i32 + sfx_sum)
                                        .clamp(-32760, 32760) as i16;
                                    let r = (audio_bufs[i][f * 2 + 1] as i32 + sfx_sum)
                                        .clamp(-32760, 32760) as i16;
                                    audio_bufs[i][f * 2] = l;
                                    audio_bufs[i][f * 2 + 1] = r;
                                }
                            }
                            guard.sfx_queue.retain(|(s, c)| *c < s.len());
                        }
                    }

                    hdrs[i].dw_flags &= !WHDR_DONE;
                    waveOutWrite(hwo, &mut hdrs[i], std::mem::size_of::<WaveHdr>() as u32);
                    any_queued = true;
                }
            }

            if !any_queued {
                thread::sleep(Duration::from_millis(8));
            }
        }

        waveOutReset(hwo);
        for i in 0..NUM_BUFS {
            waveOutUnprepareHeader(hwo, &mut hdrs[i], std::mem::size_of::<WaveHdr>() as u32);
        }
        waveOutClose(hwo);
    }
}

/// Synthesizes soft, low-volume (~-22dB), warm low-passed sound effects so they never overpower the music
fn build_gentle_sfx_bank() -> Vec<Vec<i16>> {
    vec![
        // 0: SuctionStick (soft muted woodblock/rubber pop)
        synth_soft_sfx(55, |t, _| {
            let f = if t < 0.025 { 160.0 + t * 2200.0 } else { 290.0 - (t - 0.025) * 1800.0 };
            (t * f * 6.283).sin() * 0.6
        }),
        // 1: SuctionLaunch (gentle elastic whoosh)
        synth_soft_sfx(110, |t, _| {
            let f = 130.0 + t * 1400.0;
            (t * f * 6.283).sin() * (1.0 - t * 7.5).max(0.0) * 0.55
        }),
        // 2: ToadBounce (warm bassy frog boing)
        synth_soft_sfx(150, |t, _| {
            let f = 95.0 + (t * 28.0).sin() * 25.0 + t * 380.0;
            (t * f * 6.283).sin() * (1.0 - t * 5.5).max(0.0) * 0.65
        }),
        // 3: HarvestPluck (soft marimba two-note chime)
        synth_soft_sfx(120, |t, _| {
            let f = if t < 0.055 { 392.0 } else { 587.33 };
            (t * f * 6.283).sin() * (1.0 - t * 6.5).max(0.0) * 0.5
        }),
        // 4: BombExplode (warm low thud)
        synth_soft_sfx(200, |t, i| {
            let noise = (((i.wrapping_mul(1103515245).wrapping_add(12345)) >> 16) & 0xFF) as f32 / 128.0 - 1.0;
            let sub = (t * 58.0 * 6.283).sin();
            (noise * 0.25 + sub * 0.65) * (1.0 - t * 4.5).max(0.0) * 0.65
        }),
        // 5: ElectricZap (gentle hum)
        synth_soft_sfx(90, |t, _| {
            ((t * 330.0 * 6.283).sin() * 0.5 + (t * 660.0 * 6.283).sin() * 0.25)
                * (1.0 - t * 9.0).max(0.0)
                * 0.45
        }),
        // 6: CleaverChop (soft wooden cutting-board tap)
        synth_soft_sfx(45, |t, _| {
            (t * 195.0 * 6.283).sin() * (1.0 - t * 18.0).max(0.0) * 0.6
        }),
        // 7: WokSizzle (very quiet simmer bubble)
        synth_soft_sfx(80, |t, i| {
            let noise = (((i.wrapping_mul(1664525).wrapping_add(1013904223)) >> 16) & 0xFF) as f32 / 128.0 - 1.0;
            noise * (1.0 - t * 10.0).max(0.0) * 0.25
        }),
        // 8: ServiceBell (warm rhodes-style chord ding)
        synth_soft_sfx(220, |t, _| {
            let f = if t < 0.08 { 523.25 } else { 659.25 };
            (t * f * 6.283).sin() * (1.0 - t * 4.0).max(0.0) * 0.5
        }),
        // 9: CoinStart (gentle start chime)
        synth_soft_sfx(160, |t, _| {
            let f = if t < 0.07 { 440.0 } else { 587.33 };
            (t * f * 6.283).sin() * (1.0 - t * 5.0).max(0.0) * 0.5
        }),
        // 10: Hurt (low thump)
        synth_soft_sfx(130, |t, _| {
            let f = 180.0 - t * 600.0;
            (t * f.max(55.0) * 6.283).sin() * (1.0 - t * 6.5).max(0.0) * 0.55
        }),
    ]
}

fn synth_soft_sfx<F: Fn(f32, usize) -> f32>(duration_ms: usize, gen_sample: F) -> Vec<i16> {
    let sample_rate = 44100usize;
    let num_samples = (sample_rate * duration_ms) / 1000;
    let mut out = Vec::with_capacity(num_samples);
    let mut lp_state = 0.0f32;
    for i in 0..num_samples {
        let t = i as f32 / sample_rate as f32;
        let raw = gen_sample(t, i).clamp(-1.0, 1.0);
        // 1-pole low-pass filter so SFX stay warm and never clash with the music
        lp_state = lp_state * 0.72 + raw * 0.28;
        // Max amplitude ~2100 out of 32767 (~-24 dBFS, gentle background level)
        out.push((lp_state * 2100.0) as i16);
    }
    out
}
