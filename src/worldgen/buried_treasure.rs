use super::{Chest, Scan, Scanner, invalid_scan};
use crate::decoration_seed::container_loot_seed;
use crate::error::Error;
use crate::random::{LegacyRandom48, Random};
use steel_worldgen::structure::single_piece::BuriedTreasureStructure;
use steel_worldgen::structure::{ColumnBlock, Structure, StructureGenerationContext};

const FREQUENCY_SALT: u32 = 10_387_320;
const FREQUENCY: f32 = 0.01;
const LOOT_TABLE: &str = "minecraft:chests/buried_treasure";

pub(super) fn get_region_seed(seed: u64, chunk_x: i32, chunk_z: i32, salt: u32) -> u64 {
    let mut num = (i64::from(chunk_x) as u64)
        .wrapping_mul(3_418_731_287_123_u64)
        .wrapping_add((i64::from(chunk_z) as u64).wrapping_mul(132_897_987_541_u64));
    num = num.wrapping_add(seed).wrapping_add(salt as u64);
    num
}

pub(super) fn buried_treasure_frequency_passes(
    world_seed: i64,
    chunk_x: i32,
    chunk_z: i32,
) -> bool {
    let region_seed = get_region_seed(world_seed as u64, chunk_x, chunk_z, FREQUENCY_SALT);
    let mut random = LegacyRandom48::from_seed(region_seed);
    random.next_f32() < FREQUENCY
}

impl Scanner {
    pub(super) fn scan_buried_treasure(&self, chunk_x: i32, chunk_z: i32) -> Result<Scan, Error> {
        if !buried_treasure_frequency_passes(self.world_seed, chunk_x, chunk_z) {
            return Ok(invalid_scan());
        }

        let structure_data = self.structure_data().ok_or_else(|| {
            Error::Worldgen("buried treasure structure registry missing".to_owned())
        })?;
        let mut ctx = self.generation_context(chunk_x, chunk_z);
        let mut rng = self.feature_random(chunk_x, chunk_z);
        let Some(stub) =
            BuriedTreasureStructure.find_generation_point(&mut ctx, structure_data, &mut rng)
        else {
            return Ok(invalid_scan());
        };
        let chest_x = stub.position.0;
        let chest_z = stub.position.2;
        let ocean_floor_height = ctx.base_height(chest_x, chest_z, true);
        let top_y = ocean_floor_height;
        let mut chest_y = None;
        for y in (self.min_y()..=top_y).rev() {
            if ctx.column_state(chest_x, y - 1, chest_z) == ColumnBlock::Solid {
                chest_y = Some(y);
                break;
            }
        }

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
