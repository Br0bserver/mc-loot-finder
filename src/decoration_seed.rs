use crate::catalog::{ContainerSeedShortcut, DecorationSeedSpec};
use crate::error::Error;
use steel_utils::random::Random;
use steel_utils::random::worldgen_random::WorldgenRandom;

pub(crate) struct DecorationRandom {
    random: WorldgenRandom,
}

impl Default for DecorationRandom {
    fn default() -> Self {
        Self::new()
    }
}

impl DecorationRandom {
    pub(crate) fn new() -> Self {
        Self {
            random: WorldgenRandom::from_seed(0),
        }
    }

    pub(crate) fn for_feature(
        world_seed: i64,
        chunk_x: i32,
        chunk_z: i32,
        spec: DecorationSeedSpec,
    ) -> Self {
        let mut result = Self::new();
        let decoration_seed = result.random.set_decoration_seed(
            world_seed,
            chunk_x.wrapping_mul(16),
            chunk_z.wrapping_mul(16),
        );
        result
            .random
            .set_feature_seed(decoration_seed, spec.structure_index, spec.step);
        result
    }

    pub(crate) fn next_long(&mut self) -> i64 {
        self.random.next_i64()
    }

    pub(crate) fn next_int(&mut self, bound: i32) -> i32 {
        self.random.next_i32_bounded(bound)
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
