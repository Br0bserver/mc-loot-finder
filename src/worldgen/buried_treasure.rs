use super::{Chest, Scan, Scanner, invalid_scan, terrain::TerrainSampler};
use crate::decoration_seed::container_loot_seed;
use crate::error::Error;

use pumpkin_data::BlockId;
use pumpkin_util::{
    HeightMap,
    math::vector3::Vector3,
    random::{RandomImpl, get_region_seed, legacy_rand::LegacyRand},
};
use pumpkin_world::generation::noise::router::multi_noise_sampler::MultiNoiseSampler;

const FREQUENCY_SALT: u32 = 10_387_320;
const FREQUENCY: f32 = 0.01;
const LOOT_TABLE: &str = "minecraft:chests/buried_treasure";

pub(super) fn buried_treasure_frequency_passes(
    world_seed: i64,
    chunk_x: i32,
    chunk_z: i32,
) -> bool {
    let region_seed = get_region_seed(world_seed as u64, chunk_x, chunk_z, FREQUENCY_SALT);
    let mut random = LegacyRand::from_seed(region_seed);
    random.next_f32() < FREQUENCY
}

fn is_buried_treasure_support(block: BlockId) -> bool {
    matches!(
        block,
        BlockId::SANDSTONE
            | BlockId::STONE
            | BlockId::ANDESITE
            | BlockId::GRANITE
            | BlockId::DIORITE
    )
}

impl Scanner {
    pub(super) fn scan_buried_treasure(
        &self,
        chunk_x: i32,
        chunk_z: i32,
        sampler: &mut MultiNoiseSampler<'_>,
    ) -> Result<Scan, Error> {
        if !buried_treasure_frequency_passes(self.world_seed, chunk_x, chunk_z) {
            return Ok(invalid_scan());
        }

        let chest_x = chunk_x
            .checked_mul(16)
            .and_then(|value| value.checked_add(9))
            .ok_or_else(|| Error::Worldgen("buried treasure chest x overflowed".to_owned()))?;
        let chest_z = chunk_z
            .checked_mul(16)
            .and_then(|value| value.checked_add(9))
            .ok_or_else(|| Error::Worldgen("buried treasure chest z overflowed".to_owned()))?;
        let start = Vector3::new(chest_x, 90, chest_z);
        if !self.biome_is_valid(start, self.valid_biomes, sampler) {
            return Ok(invalid_scan());
        }

        let mut terrain = TerrainSampler::new(&self.generator);
        let top_y = terrain.height(HeightMap::OceanFloorWg, chest_x, chest_z);
        let chest_y = (self.kind.min_y()..=top_y).rev().find(|&y| {
            is_buried_treasure_support(terrain.block_state(chest_x, y - 1, chest_z).to_block_id())
        });
        let Some(chest_y) = chest_y else {
            return Ok(Scan {
                valid_structure: true,
                chests: Vec::new(),
            });
        };

        let loot_seed =
            container_loot_seed(self.world_seed, chunk_x, chunk_z, self.decoration()?, 0)?;
        Ok(Scan {
            valid_structure: true,
            chests: vec![Chest {
                structure_chunk_x: chunk_x,
                structure_chunk_z: chunk_z,
                x: chest_x,
                y: chest_y,
                z: chest_z,
                loot_table: LOOT_TABLE.to_owned(),
                ordinal: 0,
                loot_seed,
            }],
        })
    }
}
