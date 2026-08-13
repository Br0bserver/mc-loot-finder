const MULTIPLIER: u64 = 0x5DEECE66D;
const ADDEND: u64 = 0xB;
const MASK: u64 = (1_u64 << 48) - 1;

pub struct LegacyRandom48 {
    seed: u64,
}

impl LegacyRandom48 {
    pub fn new(seed: i64) -> Self {
        Self {
            seed: (seed as u64 ^ MULTIPLIER) & MASK,
        }
    }

    fn next(&mut self, bits: u32) -> i32 {
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
}
