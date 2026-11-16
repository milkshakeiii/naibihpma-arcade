#![allow(dead_code)]
//! A small deterministic generator (SplitMix64), so every run replays from its logged seed.

#[derive(Clone, Debug, Hash)]
pub struct Rng(u64);

impl Rng {
    pub fn new(seed: u64) -> Self {
        Self(seed ^ 0x9E37_79B9_7F4A_7C15)
    }

    /// A generator for one roll that has a name: the same parts always give the same numbers,
    /// and asking uses nothing up anywhere else.
    pub fn keyed(parts: [u64; 4]) -> Self {
        let mut z = 0x243F_6A88_85A3_08D3u64;
        for p in parts {
            z = Rng(z ^ p).next_u64();
        }
        Self(z)
    }

    pub fn next_u64(&mut self) -> u64 {
        self.0 = self.0.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut z = self.0;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        z ^ (z >> 31)
    }

    /// Uniform in `0..n` (n > 0).
    pub fn below(&mut self, n: u32) -> u32 {
        ((self.next_u64() >> 32) * n as u64 >> 32) as u32
    }

    pub fn range(&mut self, lo: i32, hi_incl: i32) -> i32 {
        lo + self.below((hi_incl - lo + 1) as u32) as i32
    }

    pub fn chance(&mut self, pct: u32) -> bool {
        self.below(100) < pct
    }

    pub fn pick<'a, T>(&mut self, v: &'a [T]) -> &'a T {
        &v[self.below(v.len() as u32) as usize]
    }

    pub fn shuffle<T>(&mut self, v: &mut [T]) {
        for i in (1..v.len()).rev() {
            let j = self.below(i as u32 + 1) as usize;
            v.swap(i, j);
        }
    }
}
