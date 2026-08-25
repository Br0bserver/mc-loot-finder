use super::template_scan::rotate_around_pivot;
use super::{Chest, Scan, Scanner, invalid_scan};
use crate::decoration_seed::container_loot_seed;
use crate::error::Error;
use crate::random::{LegacyRandom48, Random};
use steel_utils::{Direction, Rotation};
use steel_worldgen::structure::random_horizontal_direction;

const DESERT_PYRAMID_WIDTH: i32 = 21;
const DESERT_PYRAMID_HEIGHT: i32 = 15;
const DESERT_PYRAMID_DEPTH: i32 = 21;

/// Vanilla `StructurePiece.getWorldX`: local XZ rotated by the piece facing.
fn chest_world_x(facing: Direction, min_x: i32, max_x: i32, local_x: i32, local_z: i32) -> i32 {
    match facing {
        Direction::North | Direction::South => min_x + local_x,
        Direction::West => max_x - local_z,
        Direction::East => min_x + local_z,
        Direction::Down | Direction::Up => local_x,
    }
}

/// Vanilla `StructurePiece.getWorldZ`: local XZ rotated by the piece facing.
fn chest_world_z(facing: Direction, min_z: i32, max_z: i32, local_x: i32, local_z: i32) -> i32 {
    match facing {
        Direction::North => max_z - local_z,
        Direction::South => min_z + local_z,
        Direction::West | Direction::East => min_z + local_x,
        Direction::Down | Direction::Up => local_z,
    }
}

impl Scanner {
    /// Scans a desert pyramid candidate chunk.
    pub(super) fn scan_desert_pyramid(&self, chunk_x: i32, chunk_z: i32) -> Result<Scan, Error> {
        let min_x = chunk_x
            .checked_mul(16)
            .ok_or_else(|| Error::Worldgen("desert pyramid chunk x overflowed".to_owned()))?;
        let min_z = chunk_z
            .checked_mul(16)
            .ok_or_else(|| Error::Worldgen("desert pyramid chunk z overflowed".to_owned()))?;

        let mut ctx = self.generation_context(chunk_x, chunk_z);

        let h0 = ctx.base_height(min_x, min_z, false) - 1;
        let h1 = ctx.base_height(min_x, min_z + DESERT_PYRAMID_DEPTH, false) - 1;
        let h2 = ctx.base_height(min_x + DESERT_PYRAMID_WIDTH, min_z, false) - 1;
        let h3 = ctx.base_height(
            min_x + DESERT_PYRAMID_WIDTH,
            min_z + DESERT_PYRAMID_DEPTH,
            false,
        ) - 1;
        if h0.min(h1).min(h2).min(h3) < self.sea_level() {
            return Ok(invalid_scan());
        }

        let mid_x = min_x + 8;
        let mid_z = min_z + 8;
        let mid_y = ctx.surface_y();
        let biome = ctx.biome_at(mid_x, mid_y, mid_z);
        if !self.is_valid_biome(&biome.key) {
            return Ok(invalid_scan());
        }

        let mut random = self.chunk_random(chunk_x, chunk_z);
        let facing = random_horizontal_direction(&mut random);
        let ground_offset = -random.next_i32_bounded(3);

        let mut lowest = i32::MAX;
        for x in min_x..=min_x + DESERT_PYRAMID_WIDTH - 1 {
            for z in min_z..=min_z + DESERT_PYRAMID_DEPTH - 1 {
                lowest = lowest.min(ctx.base_height(x, z, false));
            }
        }
        let base_y = lowest + ground_offset;

        let z_axis = matches!(facing, Direction::North | Direction::South);
        let (box_width, box_depth) = if z_axis {
            (DESERT_PYRAMID_WIDTH, DESERT_PYRAMID_DEPTH)
        } else {
            (DESERT_PYRAMID_DEPTH, DESERT_PYRAMID_WIDTH)
        };
        let max_x = min_x + box_width - 1;
        let max_z = min_z + box_depth - 1;

        let decoration = self.decoration()?;
        let mut chests = Vec::with_capacity(4);
        for (ordinal, (local_x, local_z)) in [(10, 8), (12, 10), (10, 12), (8, 10)]
            .into_iter()
            .enumerate()
        {
            let loot_seed = container_loot_seed(
                self.world_seed,
                chunk_x,
                chunk_z,
                decoration,
                ordinal as i32,
            )?;
            chests.push(Chest {
                structure_chunk_x: chunk_x,
                structure_chunk_z: chunk_z,
                x: chest_world_x(facing, min_x, max_x, local_x, local_z),
                y: base_y - 11,
                z: chest_world_z(facing, min_z, max_z, local_x, local_z),
                loot_table: "minecraft:chests/desert_pyramid".to_owned(),
                ordinal: ordinal as i32,
                loot_seed,
            });
        }

        Ok(Scan {
            valid_structure: true,
            chests,
        })
    }

    /// Scans an igloo candidate chunk.
    pub(super) fn scan_igloo(&self, chunk_x: i32, chunk_z: i32) -> Result<Scan, Error> {
        let min_x = chunk_x
            .checked_mul(16)
            .ok_or_else(|| Error::Worldgen("igloo chunk x overflowed".to_owned()))?;
        let min_z = chunk_z
            .checked_mul(16)
            .ok_or_else(|| Error::Worldgen("igloo chunk z overflowed".to_owned()))?;

        let mut ctx = self.generation_context(chunk_x, chunk_z);
        let mid_x = min_x + 8;
        let mid_z = min_z + 8;
        let mid_y = ctx.surface_y();
        let biome = ctx.biome_at(mid_x, mid_y, mid_z);
        if !self.is_valid_biome(&biome.key) {
            return Ok(invalid_scan());
        }

        let mut random = self.chunk_random(chunk_x, chunk_z);
        let rotation = Rotation::get_random(&mut random);
        if random.next_f64() >= 0.5 {
            return Ok(Scan {
                valid_structure: true,
                chests: Vec::new(),
            });
        }
        let ladder_segments = random.next_i32_bounded(8) + 4;

        let (ref_x, ref_z) = rotate_around_pivot(rotation, 3, 2, 3, 7);
        let surface_y = ctx.base_height(min_x + ref_x, min_z - 2 + ref_z, false);
        let chest_y = surface_y - ladder_segments * 3 - 3;
        let (chest_rel_x, chest_rel_z) = rotate_around_pivot(rotation, 1, 6, 3, 7);

        let loot_seed =
            container_loot_seed(self.world_seed, chunk_x, chunk_z, self.decoration()?, 0)?;

        Ok(Scan {
            valid_structure: true,
            chests: vec![Chest {
                structure_chunk_x: chunk_x,
                structure_chunk_z: chunk_z,
                x: min_x + chest_rel_x,
                y: chest_y,
                z: min_z - 2 + chest_rel_z,
                loot_table: "minecraft:chests/igloo_chest".to_owned(),
                ordinal: 0,
                loot_seed,
            }],
        })
    }
}
