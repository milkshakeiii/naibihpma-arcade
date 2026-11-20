#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SfxKind {
    Click,
    Build,
    Upgrade,
    Salvage,
    Laser,
    Explosion,
    Special,
    Victory,
    Defeat,
}

#[cfg(target_os = "windows")]
mod win_audio {
    use super::SfxKind;
    use std::ffi::c_void;
    use std::ptr;

    #[repr(C)]
    #[derive(Clone, Copy)]
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
        lp_next: *mut c_void,
        reserved: usize,
    }

    const WAVE_MAPPER: u32 = 0xFFFF_FFFF;
    const WAVE_FORMAT_PCM: u16 = 1;
    const WHDR_DONE: u32 = 0x0000_0001;
    const SAMPLE_RATE: u32 = 11025;

    #[link(name = "winmm")]
    extern "system" {
        fn waveOutOpen(
            phwo: *mut isize,
            u_device_id: u32,
            pwfx: *const WaveFormatEx,
            dw_callback: usize,
            dw_instance: usize,
            fdw_open: u32,
        ) -> u32;
        fn waveOutPrepareHeader(hwo: isize, pwh: *mut WaveHdr, cbwh: u32) -> u32;
        fn waveOutUnprepareHeader(hwo: isize, pwh: *mut WaveHdr, cbwh: u32) -> u32;
        fn waveOutWrite(hwo: isize, pwh: *mut WaveHdr, cbwh: u32) -> u32;
        fn waveOutReset(hwo: isize) -> u32;
        fn waveOutClose(hwo: isize) -> u32;
    }

    struct Slot {
        hdr: Box<WaveHdr>,
        buf: Vec<u8>,
        prepared: bool,
    }

    pub struct AudioEngine {
        hwo: isize,
        slots: Vec<Slot>,
        next_slot: usize,
        pub muted: bool,
    }

    impl AudioEngine {
        pub fn new() -> Self {
            let wfx = WaveFormatEx {
                w_format_tag: WAVE_FORMAT_PCM,
                n_channels: 1,
                n_samples_per_sec: SAMPLE_RATE,
                n_avg_bytes_per_sec: SAMPLE_RATE,
                n_block_align: 1,
                w_bits_per_sample: 8,
                cb_size: 0,
            };
            let mut hwo: isize = 0;
            let res = unsafe { waveOutOpen(&mut hwo, WAVE_MAPPER, &wfx, 0, 0, 0) };
            let mut slots = Vec::with_capacity(4);
            for _ in 0..4 {
                slots.push(Slot {
                    hdr: Box::new(WaveHdr {
                        lp_data: ptr::null_mut(),
                        dw_buffer_length: 0,
                        dw_bytes_recorded: 0,
                        dw_user: 0,
                        dw_flags: 0,
                        dw_loops: 0,
                        lp_next: ptr::null_mut(),
                        reserved: 0,
                    }),
                    buf: Vec::new(),
                    prepared: false,
                });
            }
            Self {
                hwo: if res == 0 { hwo } else { 0 },
                slots,
                next_slot: 0,
                muted: false,
            }
        }

        pub fn play(&mut self, sfx: SfxKind) {
            if self.muted || self.hwo == 0 {
                return;
            }
            // Pick a free slot or recycle oldest done slot
            let mut chosen = None;
            for i in 0..self.slots.len() {
                let idx = (self.next_slot + i) % self.slots.len();
                let s = &mut self.slots[idx];
                if !s.prepared || (s.hdr.dw_flags & WHDR_DONE) != 0 {
                    if s.prepared {
                        unsafe {
                            waveOutUnprepareHeader(
                                self.hwo,
                                s.hdr.as_mut() as *mut WaveHdr,
                                std::mem::size_of::<WaveHdr>() as u32,
                            );
                        }
                        s.prepared = false;
                    }
                    chosen = Some(idx);
                    break;
                }
            }
            let Some(idx) = chosen else {
                return;
            };
            self.next_slot = (idx + 1) % self.slots.len();

            let samples = synthesize_8bit_pcm(sfx);
            let slot = &mut self.slots[idx];
            slot.buf = samples;
            slot.hdr.lp_data = slot.buf.as_mut_ptr();
            slot.hdr.dw_buffer_length = slot.buf.len() as u32;
            slot.hdr.dw_flags = 0;
            slot.hdr.dw_loops = 0;

            unsafe {
                if waveOutPrepareHeader(
                    self.hwo,
                    slot.hdr.as_mut() as *mut WaveHdr,
                    std::mem::size_of::<WaveHdr>() as u32,
                ) == 0
                {
                    slot.prepared = true;
                    waveOutWrite(
                        self.hwo,
                        slot.hdr.as_mut() as *mut WaveHdr,
                        std::mem::size_of::<WaveHdr>() as u32,
                    );
                }
            }
        }
    }

    impl Drop for AudioEngine {
        fn drop(&mut self) {
            if self.hwo != 0 {
                unsafe {
                    waveOutReset(self.hwo);
                    for s in &mut self.slots {
                        if s.prepared {
                            waveOutUnprepareHeader(
                                self.hwo,
                                s.hdr.as_mut() as *mut WaveHdr,
                                std::mem::size_of::<WaveHdr>() as u32,
                            );
                        }
                    }
                    waveOutClose(self.hwo);
                }
            }
        }
    }

    fn synthesize_8bit_pcm(sfx: SfxKind) -> Vec<u8> {
        let len = match sfx {
            SfxKind::Click => 450,
            SfxKind::Build => 1400,
            SfxKind::Upgrade => 2800,
            SfxKind::Salvage => 1500,
            SfxKind::Laser => 1100,
            SfxKind::Explosion => 1800,
            SfxKind::Special => 2200,
            SfxKind::Victory => 5200,
            SfxKind::Defeat => 4800,
        };
        let mut out = Vec::with_capacity(len);
        let mut lfsr: u32 = 0xACE1;
        let mut phase: u32 = 0;

        for i in 0..len {
            let t_norm = (i * 256) / len;
            let env = if i + 100 > len {
                ((len - i) * 48) / 100
            } else {
                48
            } as i32;

            let sample_signed: i32 = match sfx {
                SfxKind::Click => {
                    let freq = if i < 200 { 880 } else { 1180 };
                    phase = phase.wrapping_add((freq * 65536) / SAMPLE_RATE);
                    if (phase & 0x8000) != 0 { env } else { -env }
                }
                SfxKind::Build => {
                    let freq = if i < 600 { 240 } else { 420 };
                    phase = phase.wrapping_add((freq * 65536) / SAMPLE_RATE);
                    if (phase & 0x8000) != 0 { env } else { -env }
                }
                SfxKind::Upgrade => {
                    let freq = match i / 900 {
                        0 => 440,
                        1 => 554,
                        _ => 659,
                    };
                    phase = phase.wrapping_add((freq * 65536) / SAMPLE_RATE);
                    if (phase & 0x8000) != 0 { env } else { -env }
                }
                SfxKind::Salvage => {
                    let freq = if i < 700 { 520 } else { 360 };
                    phase = phase.wrapping_add((freq * 65536) / SAMPLE_RATE);
                    if (phase & 0x8000) != 0 { env } else { -env }
                }
                SfxKind::Laser => {
                    let freq = 1200usize.saturating_sub(t_norm * 4).max(160) as u32;
                    phase = phase.wrapping_add((freq * 65536) / SAMPLE_RATE);
                    if (phase & 0x8000) != 0 { (env * 3) / 4 } else { -(env * 3) / 4 }
                }
                SfxKind::Explosion => {
                    if i % 3 == 0 {
                        let bit = ((lfsr >> 0) ^ (lfsr >> 2) ^ (lfsr >> 3) ^ (lfsr >> 5)) & 1;
                        lfsr = (lfsr >> 1) | (bit << 15);
                    }
                    let decay = ((len - i) * env as usize) / len;
                    if (lfsr & 1) != 0 { decay as i32 } else { -(decay as i32) }
                }
                SfxKind::Special => {
                    let freq = 320 + (t_norm as u32) * 2;
                    phase = phase.wrapping_add((freq * 65536) / SAMPLE_RATE);
                    if (phase & 0x8000) != 0 { env } else { -env }
                }
                SfxKind::Victory => {
                    let freq = match i / 1300 {
                        0 => 392,
                        1 => 494,
                        2 => 587,
                        _ => 784,
                    };
                    phase = phase.wrapping_add((freq * 65536) / SAMPLE_RATE);
                    if (phase & 0x8000) != 0 { env } else { -env }
                }
                SfxKind::Defeat => {
                    let freq = match i / 1600 {
                        0 => 294,
                        1 => 247,
                        _ => 196,
                    };
                    phase = phase.wrapping_add((freq * 65536) / SAMPLE_RATE);
                    if (phase & 0x8000) != 0 { env } else { -env }
                }
            };
            out.push((128 + sample_signed).clamp(0, 255) as u8);
        }
        out
    }
}

#[cfg(not(target_os = "windows"))]
mod win_audio {
    use super::SfxKind;
    pub struct AudioEngine {
        pub muted: bool,
    }
    impl AudioEngine {
        pub fn new() -> Self {
            Self { muted: false }
        }
        pub fn play(&mut self, _sfx: SfxKind) {}
    }
}

pub use win_audio::AudioEngine;
