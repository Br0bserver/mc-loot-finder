pub use steel_utils::random::Random;
pub use steel_utils::random::legacy_random::LegacyRandom as LegacyRandom48;
pub use steel_utils::random::worldgen_random::WorldgenRandom;
pub use steel_utils::random::xoroshiro::Xoroshiro as Xoroshiro128PlusPlus;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn matches_known_java_random_values() {
        let mut random = LegacyRandom48::from_seed(0);
        assert_eq!(random.next_i32_bounded(16), 11);
        assert_eq!(random.next_i32_bounded(16), 13);
        assert_eq!(random.next_i32_bounded(16), 3);
        assert_eq!(random.next_i32_bounded(16), 9);
    }

    #[test]
    fn matches_known_minecraft_values() {
        let mut random = Xoroshiro128PlusPlus::from_seed(0);
        assert_eq!(random.next_i64(), 3_038_984_756_725_240_190);
        assert_eq!(random.next_i64(), -3_694_039_286_755_638_414);
    }
}
