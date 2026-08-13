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
    fn matches_known_minecraft_values() {
        let mut random = Xoroshiro128PlusPlus::new(0);
        assert_eq!(random.next_long() as i64, 3_038_984_756_725_240_190);
        assert_eq!(random.next_long() as i64, -3_694_039_286_755_638_414);
    }
}
