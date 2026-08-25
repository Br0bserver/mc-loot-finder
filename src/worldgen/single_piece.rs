use super::{Chest, Scan, Scanner, invalid_scan};
use crate::decoration_seed::container_loot_seed;
use crate::error::Error;
use crate::random::Random;
use steel_utils::Direction;
use steel_worldgen::structure::desert_pyramid::DesertPyramidStructure;
use steel_worldgen::structure::igloo::IglooStructure;
use steel_worldgen::structure::{Structure, StructureGenerationContext, StructurePiecePayload};
const DESERT_PYRAMID_WIDTH: i32 = 21;
const DESERT_PYRAMID_DEPTH: i32 = 21;

const HORIZONTAL_DIRECTIONS: [Direction; 4] = [
    Direction::North,
    Direction::East,
    Direction::South,
    Direction::West,
];

fn random_horizontal_direction(rng: &mut impl Random) -> Direction {
    HORIZONTAL_DIRECTIONS[rng.next_i32_bounded(4) as usize]
}

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
        let structure_data = self.structure_data().ok_or_else(|| {
            Error::Worldgen("desert pyramid structure registry missing".to_owned())
        })?;
        let mut ctx = self.generation_context(chunk_x, chunk_z);
        let mut rng = self.feature_random(chunk_x, chunk_z);
        let Some(stub) =
            DesertPyramidStructure.find_generation_point(&mut ctx, structure_data, &mut rng)
        else {
            return Ok(invalid_scan());
        };

        let min_x = chunk_x
            .checked_mul(16)
            .ok_or_else(|| Error::Worldgen("desert pyramid chunk x overflowed".to_owned()))?;
        let min_z = chunk_z
            .checked_mul(16)
            .ok_or_else(|| Error::Worldgen("desert pyramid chunk z overflowed".to_owned()))?;

        let facing = stub.pieces[0].orientation.unwrap_or(Direction::North);
        let mut random = self.chunk_random(chunk_x, chunk_z);
        let _facing = random_horizontal_direction(&mut random);
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
        let structure_data = self
            .structure_data()
            .ok_or_else(|| Error::Worldgen("igloo structure registry missing".to_owned()))?;
        let mut ctx = self.generation_context(chunk_x, chunk_z);
        let mut rng = self.feature_random(chunk_x, chunk_z);
        let Some(stub) = IglooStructure.find_generation_point(&mut ctx, structure_data, &mut rng)
        else {
            return Ok(invalid_scan());
        };

        // If no basement piece was generated, there are no chests.
        let Some(bottom_piece) = stub.pieces.iter().find(|piece| {
            if let StructurePiecePayload::Template(data) = &piece.payload {
                data.template_id.path == "igloo/bottom"
            } else {
                false
            }
        }) else {
            return Ok(Scan {
                valid_structure: true,
                chests: Vec::new(),
            });
        };

        let StructurePiecePayload::Template(data) = &bottom_piece.payload else {
            return Ok(Scan {
                valid_structure: true,
                chests: Vec::new(),
            });
        };

        let surface_y = ctx.surface_y();
        let chest_local_pos = glam::IVec3::new(1, 1, 6);
        let transformed = super::transformed_position(
            data.rotation,
            chest_local_pos,
            glam::IVec3::new(
                data.rotation_pivot.0,
                data.rotation_pivot.1,
                data.rotation_pivot.2,
            ),
        );
        let chest_x = data.template_position.0 + transformed.x;
        let chest_z = data.template_position.2 + transformed.z;
        let chest_y = surface_y - (90 - data.template_position.1) + transformed.y;

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
                loot_table: "minecraft:chests/igloo_chest".to_owned(),
                ordinal: 0,
                loot_seed,
            }],
        })
    }
}
