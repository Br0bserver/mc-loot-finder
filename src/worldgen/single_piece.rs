use super::{Chest, Scan, Scanner, invalid_scan};
use crate::decoration_seed::container_loot_seed;
use crate::error::Error;
use crate::surface_height::ColumnHeightSampler;
use pumpkin_util::{
    BlockDirection,
    math::{block_box::BlockBox, vector3::Vector3},
    random::RandomImpl,
};
use pumpkin_world::generation::structure::structures::{
    StructurePieceBase, desert_pyramid::DesertPyramidPiece,
};
use pumpkin_world::generation::{
    noise::router::multi_noise_sampler::MultiNoiseSampler,
    structure::{generate_structure_position, structures::create_chunk_random},
};
const DESERT_PYRAMID_WIDTH: i32 = 21;
const DESERT_PYRAMID_HEIGHT: i32 = 15;
const DESERT_PYRAMID_DEPTH: i32 = 21;
/// Vanilla `StructurePiece.getWorldX`: local XZ rotated by the piece facing.
fn chest_world_x(facing: BlockDirection, box_: &BlockBox, local_x: i32, local_z: i32) -> i32 {
    match facing {
        BlockDirection::North | BlockDirection::South => box_.min.x + local_x,
        BlockDirection::West => box_.max.x - local_z,
        BlockDirection::East => box_.min.x + local_z,
        // Vanilla's switch default: the desert pyramid facing is always horizontal.
        BlockDirection::Down | BlockDirection::Up => local_x,
    }
}

/// Vanilla `StructurePiece.getWorldZ`: local XZ rotated by the piece facing.
fn chest_world_z(facing: BlockDirection, box_: &BlockBox, local_x: i32, local_z: i32) -> i32 {
    match facing {
        BlockDirection::North => box_.max.z - local_z,
        BlockDirection::South => box_.min.z + local_z,
        BlockDirection::West | BlockDirection::East => box_.min.z + local_x,
        // Vanilla's switch default: the desert pyramid facing is always horizontal.
        BlockDirection::Down | BlockDirection::Up => local_z,
    }
}

/// Vanilla `StructureTemplate.transform` rotation of an XZ offset around a
/// pivot. Rotation indexes follow the vanilla enum order: 0 = NONE,
/// 1 = CLOCKWISE_90, 2 = CLOCKWISE_180, 3 = COUNTERCLOCKWISE_90.
fn rotate_around_pivot(
    rotation_index: i32,
    x: i32,
    z: i32,
    pivot_x: i32,
    pivot_z: i32,
) -> (i32, i32) {
    match rotation_index {
        0 => (x, z),
        1 => (pivot_x - z + pivot_z, pivot_z + x - pivot_x),
        2 => (2 * pivot_x - x, 2 * pivot_z - z),
        _ => (pivot_x + z - pivot_z, pivot_z - x + pivot_x),
    }
}

impl Scanner {
    /// Scans a desert pyramid candidate chunk.
    ///
    /// Mirrors vanilla 26.1.2 `SinglePieceStructure.findGenerationPoint` +
    /// `DesertPyramidPiece`:
    /// 1. the lowest `getFirstOccupiedHeight(WORLD_SURFACE_WG)` at the four
    ///    bounding box corners must be at least the sea level;
    /// 2. the biome at the chunk center block, sampled at the
    ///    `getFirstOccupiedHeight` world surface height, must be in the
    ///    structure's biome tag;
    /// 3. the piece is anchored at `(minBlockX, 64, minBlockZ)` with a
    ///    horizontal facing drawn from the placement random;
    /// 4. `postProcess` draws `nextInt(3)` and shifts the piece so its base sits
    ///    at the lowest `MOTION_BLOCKING_NO_LEAVES` heightmap value
    ///    (`getBaseHeight`) in the 21x21 area plus the (non-positive) ground
    ///    offset;
    /// 5. four chests are placed in NORTH/EAST/SOUTH/WEST order at local
    ///    `(10 +- 2, -11, 10 +- 2)`; each consumes one `nextLong` from the
    ///    decoration random, which is exactly the
    ///    `ContainerSeedShortcut::DesertPyramid` shortcut.
    pub(super) fn scan_desert_pyramid(
        &self,
        chunk_x: i32,
        chunk_z: i32,
        sampler: &mut MultiNoiseSampler<'_>,
    ) -> Result<Scan, Error> {
        let min_x = chunk_x
            .checked_mul(16)
            .ok_or_else(|| Error::Worldgen("desert pyramid chunk x overflowed".to_owned()))?;
        let min_z = chunk_z
            .checked_mul(16)
            .ok_or_else(|| Error::Worldgen("desert pyramid chunk z overflowed".to_owned()))?;
        let mut heights = ColumnHeightSampler::new(&self.generator, min_x, min_z);

        let corner_lowest = [
            (min_x, min_z),
            (min_x, min_z + DESERT_PYRAMID_DEPTH),
            (min_x + DESERT_PYRAMID_WIDTH, min_z),
            (min_x + DESERT_PYRAMID_WIDTH, min_z + DESERT_PYRAMID_DEPTH),
        ]
        .into_iter()
        .map(|(x, z)| heights.first_occupied_height(x, z))
        .min()
        .ok_or_else(|| Error::Worldgen("desert pyramid corner list was empty".to_owned()))?;
        if corner_lowest < self.kind.sea_level() {
            return Ok(invalid_scan());
        }

        let mid_x = min_x + 8;
        let mid_z = min_z + 8;
        let mid_y = heights.first_occupied_height(mid_x, mid_z);
        if !self.biome_is_valid(
            Vector3::new(mid_x, mid_y, mid_z),
            self.valid_biomes,
            sampler,
        ) {
            return Ok(invalid_scan());
        }

        let structure = self.kind.structure();
        let position = generate_structure_position(
            &self.kind.structure_key(),
            &structure,
            self.context(chunk_x, chunk_z),
        )
        .ok_or_else(|| Error::Worldgen("desert pyramid failed full placement".to_owned()))?;

        let collector = position.collector.lock().map_err(|_| {
            Error::Worldgen("desert pyramid piece collector was poisoned".to_owned())
        })?;
        let piece = collector
            .pieces
            .iter()
            .find_map(|piece| piece.as_any().downcast_ref::<DesertPyramidPiece>())
            .ok_or_else(|| Error::Worldgen("desert pyramid piece is missing".to_owned()))?;
        let structure_piece = piece.get_structure_piece();
        let facing = structure_piece
            .facing
            .ok_or_else(|| Error::Worldgen("desert pyramid piece has no facing".to_owned()))?;
        let bounding_box = structure_piece.bounding_box;

        // The placement random draws the horizontal facing (nextInt(4)) and then
        // the ground offset (nextInt(3)); only the latter value is needed here.
        let mut random = create_chunk_random(self.world_seed, chunk_x, chunk_z);
        random.next_bounded_i32(4);
        let ground_offset = -random.next_bounded_i32(3);

        let mut lowest = i32::MAX;
        for x in min_x..=min_x + DESERT_PYRAMID_WIDTH - 1 {
            for z in min_z..=min_z + DESERT_PYRAMID_DEPTH - 1 {
                lowest = lowest.min(heights.base_height(x, z));
            }
        }
        let base_y = lowest + ground_offset;
        let adjusted_box = BlockBox {
            min: Vector3::new(bounding_box.min.x, base_y, bounding_box.min.z),
            max: Vector3::new(
                bounding_box.max.x,
                base_y + DESERT_PYRAMID_HEIGHT - 1,
                bounding_box.max.z,
            ),
        };

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
                x: chest_world_x(facing, &adjusted_box, local_x, local_z),
                y: base_y - 11,
                z: chest_world_z(facing, &adjusted_box, local_x, local_z),
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
    ///
    /// Mirrors vanilla 26.1.2 `IglooStructure` + `IglooPieces`:
    /// 1. the biome at the chunk center block, sampled at
    ///    `getFirstOccupiedHeight(WORLD_SURFACE_WG)`, must be in the biome tag;
    /// 2. the placement random draws the template rotation (`nextInt(4)`), the
    ///    basement chance (`nextDouble() < 0.5`) and the ladder segment count
    ///    (`nextInt(8) + 4`);
    /// 3. with a basement, the bottom template's reference column
    ///    (`getHeight(WORLD_SURFACE_WG)`) anchors the piece so the chest sits at
    ///    template local (1, 1, 6) rotated around the bottom pivot XZ (3, 7),
    ///    offset from the chunk by `OFFSETS[bottom] = (0, -3, -2)`;
    /// 4. the chest loot seed is the second `nextLong` of the decoration stream
    ///    after `setFeatureSeed` (the first is consumed by template placement),
    ///    which is `ContainerSeedShortcut::Direct` with ordinal 1.
    pub(super) fn scan_igloo(
        &self,
        chunk_x: i32,
        chunk_z: i32,
        sampler: &mut MultiNoiseSampler<'_>,
    ) -> Result<Scan, Error> {
        let min_x = chunk_x
            .checked_mul(16)
            .ok_or_else(|| Error::Worldgen("igloo chunk x overflowed".to_owned()))?;
        let min_z = chunk_z
            .checked_mul(16)
            .ok_or_else(|| Error::Worldgen("igloo chunk z overflowed".to_owned()))?;
        let mut heights = ColumnHeightSampler::new(&self.generator, min_x, min_z);

        let mid_x = min_x + 8;
        let mid_z = min_z + 8;
        let mid_y = heights.first_occupied_height(mid_x, mid_z);
        if !self.biome_is_valid(
            Vector3::new(mid_x, mid_y, mid_z),
            self.valid_biomes,
            sampler,
        ) {
            return Ok(invalid_scan());
        }

        // The placement random draws the template rotation, the basement chance
        // and the ladder segment count, in that order.
        let mut random = create_chunk_random(self.world_seed, chunk_x, chunk_z);
        let rotation_index = random.next_bounded_i32(4);
        if random.next_f64() >= 0.5 {
            return Ok(Scan {
                valid_structure: true,
                chests: Vec::new(),
            });
        }
        let ladder_segments = random.next_bounded_i32(8) + 4;

        // Bottom template: `OFFSETS[bottom] = (0, -3, -2)`, pivot XZ (3, 7).
        // The sink reference is template local (3, 0, 2); the chest is at
        // template local (1, 1, 6) and ends up `ladder_segments * 3` blocks
        // below the reference column `MOTION_BLOCKING_NO_LEAVES` height, plus
        // the OFFSETS Y component (-3) of the bottom template anchor
        // (template position starts at 90 - 3 - ladder_segments * 3).
        let (ref_x, ref_z) = rotate_around_pivot(rotation_index, 3, 2, 3, 7);
        let surface_y = heights.motion_blocking_no_leaves_height(min_x + ref_x, min_z - 2 + ref_z);
        let chest_y = surface_y - ladder_segments * 3 - 3;
        let (chest_rel_x, chest_rel_z) = rotate_around_pivot(rotation_index, 1, 6, 3, 7);

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
