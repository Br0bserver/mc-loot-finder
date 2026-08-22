use crate::catalog::{ContainerSeedShortcut, DecorationSeedSpec};
use crate::error::Error;
use crate::random::Xoroshiro128PlusPlus;

pub(crate) struct DecorationRandom {
    random: Xoroshiro128PlusPlus,
}

impl Default for DecorationRandom {
    fn default() -> Self {
        Self::new()
    }
}

impl DecorationRandom {
    fn new() -> Self {
        Self {
            random: Xoroshiro128PlusPlus::new(0),
        }
    }

    pub(crate) fn for_feature(
        world_seed: i64,
        chunk_x: i32,
        chunk_z: i32,
        spec: DecorationSeedSpec,
    ) -> Self {
        let mut random = Self::new();
        let decoration_seed = random.set_decoration_seed(
            world_seed,
            chunk_x.wrapping_mul(16),
            chunk_z.wrapping_mul(16),
        );
        random.set_feature_seed(decoration_seed, spec.structure_index, spec.step);
        random
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

    pub(crate) fn next_long(&mut self) -> i64 {
        let high = (self.random.next_long() >> 32) as i32;
        let low = (self.random.next_long() >> 32) as i32;
        (i64::from(high) << 32).wrapping_add(i64::from(low))
    }

    pub(crate) fn next_int(&mut self, bound: i32) -> i32 {
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
    spec: DecorationSeedSpec,
    ordinal: i32,
) -> Result<i64, Error> {
    if ordinal < 0 {
        return Err(Error::Usage(
            "container ordinal must be non-negative".to_owned(),
        ));
    }
    if spec.shortcut == ContainerSeedShortcut::Unavailable {
        return Err(Error::Usage(
            "container seed requires exact structure placement".to_owned(),
        ));
    }
    let effective_ordinal = ordinal
        .checked_add(spec.ordinal_offset)
        .ok_or_else(|| Error::Usage("container ordinal overflowed".to_owned()))?;

    let mut random = DecorationRandom::for_feature(world_seed, chunk_x, chunk_z, spec);
    if spec.shortcut == ContainerSeedShortcut::DesertPyramid {
        random.next_int(3);
    }
    let mut result = 0;
    for _ in 0..=effective_ordinal {
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
            container_loot_seed(
                0,
                0,
                0,
                DecorationSeedSpec {
                    structure_index: 0,
                    step: 7,
                    ordinal_offset: 0,
                    shortcut: ContainerSeedShortcut::Direct,
                },
                0,
            )
            .unwrap(),
            6_384_546_642_282_394_621
        );
    }

    #[test]
    fn matches_known_desert_pyramid_seed() {
        assert_eq!(
            container_loot_seed(
                0,
                -188,
                0,
                DecorationSeedSpec {
                    structure_index: 1,
                    step: 4,
                    ordinal_offset: 0,
                    shortcut: ContainerSeedShortcut::DesertPyramid,
                },
                2,
            )
            .unwrap(),
            3_899_282_274_470_656_331
        );
    }

    #[test]
    fn applies_igloo_template_ordinal_offset() {
        assert_eq!(
            container_loot_seed(
                0,
                98,
                192,
                DecorationSeedSpec {
                    structure_index: 3,
                    step: 4,
                    ordinal_offset: 1,
                    shortcut: ContainerSeedShortcut::Direct,
                },
                0,
            )
            .unwrap(),
            -7_862_992_963_971_781_551
        );
    }

    #[test]
    fn unavailable_shortcut_requires_structure_placement() {
        let error = container_loot_seed(
            0,
            14,
            8,
            DecorationSeedSpec {
                structure_index: 18,
                step: 4,
                ordinal_offset: 0,
                shortcut: ContainerSeedShortcut::Unavailable,
            },
            0,
        )
        .expect_err("shipwreck seed must require exact template placement");
        assert!(matches!(error, Error::Usage(_)));
    }

    #[test]
    fn matches_known_pillager_outpost_seed() {
        assert_eq!(
            container_loot_seed(
                0,
                -52,
                69,
                DecorationSeedSpec {
                    structure_index: 4,
                    step: 9,
                    ordinal_offset: 0,
                    shortcut: ContainerSeedShortcut::Direct,
                },
                1,
            )
            .unwrap(),
            -638_836_315_418_230_144
        );
    }
}
