#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SimRng {
    state: u64,
}

impl SimRng {
    pub fn new(seed: u64) -> Self {
        let mut s = seed.wrapping_add(0x9E37_79B9_7F4A_7C15);
        s = (s ^ (s >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        s = (s ^ (s >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        s ^= s >> 31;
        if s == 0 {
            s = 0xCAFE_BABE_1234_5678;
        }
        Self { state: s }
    }

    #[inline]
    pub fn next_u64(&mut self) -> u64 {
        let mut x = self.state;
        x ^= x << 13;
        x ^= x >> 7;
        x ^= x << 17;
        self.state = x;
        x
    }

    #[inline]
    pub fn next_u32(&mut self) -> u32 {
        (self.next_u64() >> 32) as u32
    }

    #[inline]
    pub fn range_i32(&mut self, lo: i32, hi_exclusive: i32) -> i32 {
        if hi_exclusive <= lo {
            return lo;
        }
        let span = (hi_exclusive - lo) as u64;
        lo + (self.next_u64() % span) as i32
    }

    #[inline]
    pub fn range_usize(&mut self, lo: usize, hi_exclusive: usize) -> usize {
        if hi_exclusive <= lo {
            return lo;
        }
        let span = (hi_exclusive - lo) as u64;
        lo + (self.next_u64() % span) as usize
    }
}
