use crate::catalog::ContainerSeedShortcut;
use crate::random::Xoroshiro128PlusPlus;

struct DecorationRandom {
    random: Xoroshiro128PlusPlus,
}

impl DecorationRandom {
    fn new() -> Self {
        Self {
            random: Xoroshiro128PlusPlus::new(0),
        }
    }

    fn set_decoration_seed(&mut self, world_seed: i64, block_x: i32, block_z: i32) -> i64 {
        self.random.set_seed(world_seed);
        let x_multiplier = self.next_long() | 1;
        let z_multiplier = self.next_long() | 1;
        let decoration_seed = i64::from(block_x)
            .wrapping_mul(x_multiplier)
            .wrapping_add(i64::from(block_z).wrapping_mul(z_multiplier))
            ^ world_seed;
        self.random.set_seed(decoration_seed);
        decoration_seed
    }

    fn set_feature_seed(&mut self, decoration_seed: i64, feature_index: i32, step: i32) {
        self.random.set_seed(
            decoration_seed
                .wrapping_add(i64::from(feature_index))
                .wrapping_add(10_000_i64.wrapping_mul(i64::from(step))),
        );
    }

    fn next_long(&mut self) -> i64 {
        let high = (self.random.next_long() >> 32) as i32;
        let low = (self.random.next_long() >> 32) as i32;
        (i64::from(high) << 32).wrapping_add(i64::from(low))
    }

    fn next_int(&mut self, bound: i32) -> i32 {
        assert!(bound > 0, "bound must be positive");
        if bound & (bound - 1) == 0 {
            return ((i64::from(bound) * i64::from(self.next_bits(31))) >> 31) as i32;
        }
        loop {
            let bits = self.next_bits(31);
            let value = bits % bound;
            if bits.wrapping_sub(value).wrapping_add(bound - 1) >= 0 {
                return value;
            }
        }
    }

    fn next_bits(&mut self, bits: u32) -> i32 {
        (self.random.next_long() >> (64 - bits)) as i32
    }
}

pub fn container_loot_seed(
    world_seed: i64,
    chunk_x: i32,
    chunk_z: i32,
    structure_index: i32,
    decoration_step: i32,
    ordinal: i32,
    shortcut: ContainerSeedShortcut,
) -> Result<i64, String> {
    if ordinal < 0 {
        return Err("container ordinal must be non-negative".to_owned());
    }
    if shortcut == ContainerSeedShortcut::None {
        return Err("container seed shortcut is unavailable".to_owned());
    }

    let mut random = DecorationRandom::new();
    let decoration_seed = random.set_decoration_seed(
        world_seed,
        chunk_x.wrapping_mul(16),
        chunk_z.wrapping_mul(16),
    );
    random.set_feature_seed(decoration_seed, structure_index, decoration_step);
    if shortcut == ContainerSeedShortcut::DesertPyramid {
        random.next_int(3);
    }
    let mut result = 0;
    for _ in 0..=ordinal {
        result = random.next_long();
    }
    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn matches_known_ancient_city_seed() {
        assert_eq!(
            container_loot_seed(0, 0, 0, 0, 7, 0, ContainerSeedShortcut::Direct).unwrap(),
            6_384_546_642_282_394_621
        );
    }

    #[test]
    fn matches_known_desert_pyramid_seed() {
        assert_eq!(
            container_loot_seed(0, -188, 0, 1, 4, 2, ContainerSeedShortcut::DesertPyramid).unwrap(),
            3_899_282_274_470_656_331
        );
    }
}
