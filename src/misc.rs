pub struct Prng {
    s: u64,
}

impl Prng {
    pub fn new(seed: u64) -> Self {
        assert!(seed != 0);
        Self {
            s: seed,
        }
    }

    pub fn rand64(&mut self) -> u64 {
        self.s ^= self.s >> 12;
        self.s ^= self.s << 25;
        self.s ^= self.s >> 27;
        self.s.wrapping_mul(2685821657736338717)
    }


    pub fn rand<T: From<u64>>(&mut self) -> T {
        T::from(self.rand64())
    }

    pub fn sparse_rand<T: From<u64>>(&mut self) -> T {
        T::from(self.rand64() & self.rand64() & self.rand64())
    }
}