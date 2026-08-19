const MULTIPLIER: u64 = 0x5DEECE66D;
const ADDEND: u64 = 0xB;
const MASK: u64 = (1_u64 << 48) - 1;

pub struct LegacyRandom48 {
    seed: u64,
}

#[allow(dead_code)]
impl LegacyRandom48 {
    pub fn new(seed: i64) -> Self {
        Self {
            seed: (seed as u64 ^ MULTIPLIER) & MASK,
        }
    }

    pub fn set_seed(&mut self, seed: i64) {
        self.seed = (seed as u64 ^ MULTIPLIER) & MASK;
    }

    pub fn next(&mut self, bits: u32) -> i32 {
        self.seed = self.seed.wrapping_mul(MULTIPLIER).wrapping_add(ADDEND) & MASK;
        (self.seed >> (48 - bits)) as i32
    }

    pub fn next_int(&mut self, bound: i32) -> i32 {
        assert!(bound > 0, "bound must be positive");
        if bound & -bound == bound {
            return ((i64::from(bound) * i64::from(self.next(31))) >> 31) as i32;
        }
        loop {
            let bits = self.next(31);
            let value = bits % bound;
            if bits.wrapping_sub(value).wrapping_add(bound - 1) >= 0 {
                return value;
            }
        }
    }

    pub fn next_int_unbounded(&mut self) -> i32 {
        self.next(32)
    }

    pub fn next_long(&mut self) -> i64 {
        ((self.next(32) as i64) << 32) + (self.next(32) as i64)
    }

    pub fn next_double(&mut self) -> f64 {
        let high = self.next(26) as i64;
        let low = self.next(27) as i64;
        ((high << 27) + low) as f64 / (1i64 << 53) as f64
    }

    pub fn next_float(&mut self) -> f32 {
        self.next(24) as f32 * 2_f32.powi(-24)
    }

    pub fn set_large_feature_seed(&mut self, seed: i64, chunk_x: i32, chunk_z: i32) {
        self.set_seed(seed);
        let a = self.next_long();
        let b = self.next_long();
        let c = (i64::from(chunk_x).wrapping_mul(a)) ^ (i64::from(chunk_z).wrapping_mul(b)) ^ seed;
        self.set_seed(c);
    }
}

const GOLDEN_RATIO_64: u64 = 0x9E37_79B9_7F4A_7C15;
const SILVER_RATIO_64: u64 = 0x6A09_E667_F3BC_C909;
const STAFFORD_MIX_1: u64 = 0xBF58_476D_1CE4_E5B9;
const STAFFORD_MIX_2: u64 = 0x94D0_49BB_1331_11EB;

pub struct Xoroshiro128PlusPlus {
    seed_lo: u64,
    seed_hi: u64,
}

impl Xoroshiro128PlusPlus {
    pub fn new(seed: i64) -> Self {
        let mut random = Self {
            seed_lo: 0,
            seed_hi: 0,
        };
        random.set_seed(seed);
        random
    }

    pub fn set_seed(&mut self, seed: i64) {
        let low_bits = seed as u64 ^ SILVER_RATIO_64;
        let high_bits = low_bits.wrapping_add(GOLDEN_RATIO_64);
        self.seed_lo = mix_stafford_13(low_bits);
        self.seed_hi = mix_stafford_13(high_bits);
        if self.seed_lo | self.seed_hi == 0 {
            self.seed_lo = GOLDEN_RATIO_64;
            self.seed_hi = SILVER_RATIO_64;
        }
    }

    pub fn next_long(&mut self) -> u64 {
        let seed_lo = self.seed_lo;
        let mut seed_hi = self.seed_hi;
        let result = seed_lo
            .wrapping_add(seed_hi)
            .rotate_left(17)
            .wrapping_add(seed_lo);
        seed_hi ^= seed_lo;
        self.seed_lo = seed_lo.rotate_left(49) ^ seed_hi ^ seed_hi.wrapping_shl(21);
        self.seed_hi = seed_hi.rotate_left(28);
        result
    }
}

fn mix_stafford_13(mut value: u64) -> u64 {
    value = (value ^ (value >> 30)).wrapping_mul(STAFFORD_MIX_1);
    value = (value ^ (value >> 27)).wrapping_mul(STAFFORD_MIX_2);
    value ^ (value >> 31)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn matches_known_java_random_values() {
        let mut random = LegacyRandom48::new(0);
        assert_eq!(random.next_int(16), 11);
        assert_eq!(random.next_int(16), 13);
        assert_eq!(random.next_int(16), 3);
        assert_eq!(random.next_int(16), 9);
    }

    #[test]
    fn matches_known_minecraft_values() {
        let mut random = Xoroshiro128PlusPlus::new(0);
        assert_eq!(random.next_long() as i64, 3_038_984_756_725_240_190);
        assert_eq!(random.next_long() as i64, -3_694_039_286_755_638_414);
    }
}
