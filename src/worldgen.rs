use std::collections::HashMap;

use crate::catalog::ContainerSeedShortcut;
use crate::decoration_seed::container_loot_seed;
use crate::error::Error;
use crate::random::LegacyRandom48;
use crate::surface_height::ColumnHeightSampler;
use crate::village_jigsaw;
use pumpkin_data::{
    dimension::Dimension,
    structures::{Structure, StructureKeys},
    tag::{RegistryKey, get_tag_values},
};
use pumpkin_util::{
    BlockDirection,
    math::{block_box::BlockBox, vector3::Vector3},
    random::RandomImpl,
    world_seed::Seed,
};
use pumpkin_world::generation::structure::structures::{
    StructurePieceBase, desert_pyramid::DesertPyramidPiece, jigsaw::PoolElementStructurePiece,
};
use pumpkin_world::{
    biome::{BiomeSupplier, MultiNoiseBiomeSupplier},
    generation::{
        biome_coords,
        generator::{GeneratorInit, VanillaGenerator},
        noise::router::multi_noise_sampler::{MultiNoiseSampler, MultiNoiseSamplerBuilderOptions},
        structure::{
            generate_structure_position,
            structures::{StructureGeneratorContext, create_chunk_random},
        },
    },
};

const DESERT_PYRAMID_WIDTH: i32 = 21;
const DESERT_PYRAMID_HEIGHT: i32 = 15;
const DESERT_PYRAMID_DEPTH: i32 = 21;

const OVERWORLD_MIN_Y: i32 = -64;
const NETHER_MIN_Y: i32 = 0;
const OVERWORLD_SEA_LEVEL: i32 = 63;
const NETHER_SEA_LEVEL: i32 = 32;

#[allow(dead_code)]
const PILLAGER_FREQUENCY: f32 = 0.2;
const VILLAGE_SPACING: i32 = 34;
const VILLAGE_SEPARATION: i32 = 8;
const VILLAGE_SALT: i64 = 10_387_312;
const VILLAGE_EXCLUSION_RADIUS: i32 = 10;
const REGION_X_MULTIPLIER: i64 = 341_873_128_712;
const REGION_Z_MULTIPLIER: i64 = 132_897_987_541;

#[allow(dead_code)]
fn is_village_placement_chunk(world_seed: i64, chunk_x: i32, chunk_z: i32) -> bool {
    let spacing = VILLAGE_SPACING;
    let separation = VILLAGE_SEPARATION;
    let salt = VILLAGE_SALT;
    let limit = spacing - separation;
    let region_x = chunk_x.div_euclid(spacing);
    let region_z = chunk_z.div_euclid(spacing);
    let placement_seed = world_seed
        .wrapping_add(i64::from(region_x).wrapping_mul(REGION_X_MULTIPLIER))
        .wrapping_add(i64::from(region_z).wrapping_mul(REGION_Z_MULTIPLIER))
        .wrapping_add(salt);
    let mut random = LegacyRandom48::new(placement_seed);
    let offset_x = random.next_int(limit);
    let offset_z = random.next_int(limit);
    let potential_x = region_x * spacing + offset_x;
    let potential_z = region_z * spacing + offset_z;
    chunk_x == potential_x && chunk_z == potential_z
}

#[allow(dead_code)]
fn has_village_nearby(world_seed: i64, chunk_x: i32, chunk_z: i32) -> bool {
    let radius = VILLAGE_EXCLUSION_RADIUS;
    for dx in -radius..=radius {
        for dz in -radius..=radius {
            let other_x = chunk_x + dx;
            let other_z = chunk_z + dz;
            if is_village_placement_chunk(world_seed, other_x, other_z) {
                return true;
            }
        }
    }
    false
}

#[allow(dead_code)]
fn pillager_frequency_passes(world_seed: i64, chunk_x: i32, chunk_z: i32) -> bool {
    let i = chunk_x >> 4;
    let j = chunk_z >> 4;
    let combined = i64::from(i << 4 ^ j) ^ world_seed;
    let mut random = LegacyRandom48::new(combined);
    let _ = random.next_int_unbounded();
    let bound = (1.0 / PILLAGER_FREQUENCY) as i32;
    random.next_int(bound) == 0
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Kind {
    AncientCity,
    BastionRemnant,
    DesertPyramid,
    Igloo,
    Village,
    PillagerOutpost,
}
impl Kind {
    fn structure(self) -> Structure {
        match self {
            Self::AncientCity => Structure::ANCIENT_CITY,
            Self::BastionRemnant => Structure::BASTION_REMNANT,
            Self::DesertPyramid => Structure::DESERT_PYRAMID,
            Self::Igloo => Structure::IGLOO,
            // The village variant is selected per candidate in `scan_village`.
            Self::Village => Structure::VILLAGE_PLAINS,
            Self::PillagerOutpost => Structure::PILLAGER_OUTPOST,
        }
    }

    const fn structure_key(self) -> StructureKeys {
        match self {
            Self::AncientCity => StructureKeys::AncientCity,
            Self::BastionRemnant => StructureKeys::BastionRemnant,
            Self::DesertPyramid => StructureKeys::DesertPyramid,
            Self::Igloo => StructureKeys::Igloo,
            Self::Village => StructureKeys::VillagePlains,
            Self::PillagerOutpost => StructureKeys::PillagerOutpost,
        }
    }

    const fn dimension(self) -> Dimension {
        match self {
            Self::AncientCity
            | Self::DesertPyramid
            | Self::Igloo
            | Self::Village
            | Self::PillagerOutpost => Dimension::OVERWORLD,
            Self::BastionRemnant => Dimension::THE_NETHER,
        }
    }

    const fn min_y(self) -> i32 {
        match self {
            Self::AncientCity
            | Self::DesertPyramid
            | Self::Igloo
            | Self::Village
            | Self::PillagerOutpost => OVERWORLD_MIN_Y,
            Self::BastionRemnant => NETHER_MIN_Y,
        }
    }

    const fn sea_level(self) -> i32 {
        match self {
            Self::AncientCity
            | Self::DesertPyramid
            | Self::Igloo
            | Self::Village
            | Self::PillagerOutpost => OVERWORLD_SEA_LEVEL,
            Self::BastionRemnant => NETHER_SEA_LEVEL,
        }
    }

    const fn decoration_coordinates(self) -> (i32, i32) {
        match self {
            Self::AncientCity => (7, 0),
            Self::BastionRemnant => (4, 0),
            Self::DesertPyramid => (1, 4),
            Self::Igloo => (3, 4),
            // The village variant index is resolved per candidate.
            Self::Village => (22, 4),
            Self::PillagerOutpost => (9, 4),
        }
    }

    const fn biome_supplier(self) -> MultiNoiseBiomeSupplier {
        match self {
            Self::AncientCity
            | Self::DesertPyramid
            | Self::Igloo
            | Self::Village
            | Self::PillagerOutpost => MultiNoiseBiomeSupplier::OVERWORLD,
            Self::BastionRemnant => MultiNoiseBiomeSupplier::NETHER,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Chest {
    pub structure_chunk_x: i32,
    pub structure_chunk_z: i32,
    pub x: i32,
    pub y: i32,
    pub z: i32,
    pub loot_table: String,
    pub ordinal: i32,
    pub loot_seed: i64,
}

#[derive(Debug, Eq, PartialEq)]
pub struct Scan {
    pub valid_structure: bool,
    pub chests: Vec<Chest>,
}

pub struct Scanner {
    world_seed: i64,
    kind: Kind,
    generator: VanillaGenerator,
    valid_biomes: &'static [&'static str],
    fortress_biomes: &'static [&'static str],
}

impl Scanner {
    /// Build a scanner for a structure name that supports full chest scanning.
    pub fn for_structure(structure_name: &str, world_seed: i64) -> Result<Self, Error> {
        let kind = match structure_name {
            "ancient_city" => Kind::AncientCity,
            "bastion_remnant" => Kind::BastionRemnant,
            "desert_pyramid" => Kind::DesertPyramid,
            "igloo" => Kind::Igloo,
            "village" => Kind::Village,
            "pillager_outpost" => Kind::PillagerOutpost,
            _ => {
                return Err(Error::Structure(format!(
                    "Rust chests and find do not support {structure_name} yet"
                )));
            }
        };
        Ok(Self::new(world_seed, kind))
    }

    #[must_use]
    pub fn new(world_seed: i64, kind: Kind) -> Self {
        let structure = kind.structure();
        Self {
            world_seed,
            kind,
            generator: VanillaGenerator::new(Seed(world_seed as u64), kind.dimension()),
            valid_biomes: structure_biomes(&structure),
            fortress_biomes: if kind == Kind::BastionRemnant {
                structure_biomes(&Structure::FORTRESS)
            } else {
                &[]
            },
        }
    }

    pub fn scan_many(
        &self,
        chunks: impl IntoIterator<Item = (i32, i32)>,
    ) -> Result<Vec<Scan>, Error> {
        let mut sampler = MultiNoiseSampler::generate(
            &self.generator.base_router.multi_noise,
            &MultiNoiseSamplerBuilderOptions::new(0, 0, 0),
        );
        chunks
            .into_iter()
            .map(|(chunk_x, chunk_z)| self.scan_with_sampler(chunk_x, chunk_z, &mut sampler))
            .collect()
    }

    fn scan_with_sampler(
        &self,
        chunk_x: i32,
        chunk_z: i32,
        sampler: &mut MultiNoiseSampler<'_>,
    ) -> Result<Scan, Error> {
        if self.kind == Kind::DesertPyramid {
            return self.scan_desert_pyramid(chunk_x, chunk_z, sampler);
        }
        if self.kind == Kind::Igloo {
            return self.scan_igloo(chunk_x, chunk_z, sampler);
        }
        if self.kind == Kind::Village {
            return self.scan_village(chunk_x, chunk_z, sampler);
        }
        if self.kind == Kind::PillagerOutpost {
            return self.scan_pillager_outpost(chunk_x, chunk_z, sampler);
        }
        if self.kind == Kind::BastionRemnant
            && !self.bastion_reached_in_weighted_selection(chunk_x, chunk_z, sampler)?
        {
            return Ok(invalid_scan());
        }
        let structure = self.kind.structure();
        let probe_structure = Structure {
            size: Some(0),
            ..structure
        };
        let Some(probe) = generate_structure_position(
            &self.kind.structure_key(),
            &probe_structure,
            self.context(chunk_x, chunk_z),
        ) else {
            return Ok(invalid_scan());
        };
        if !self.biome_is_valid(probe.start_pos.0, self.valid_biomes, sampler) {
            return Ok(invalid_scan());
        }

        let position = generate_structure_position(
            &self.kind.structure_key(),
            &structure,
            self.context(chunk_x, chunk_z),
        )
        .ok_or_else(|| {
            Error::Worldgen("validated jigsaw structure failed full placement".to_owned())
        })?;

        let collector = position
            .collector
            .lock()
            .map_err(|_| Error::Worldgen("jigsaw piece collector was poisoned".to_owned()))?;
        let mut raw = Vec::new();
        for piece in &collector.pieces {
            let Some(piece) = piece.as_any().downcast_ref::<PoolElementStructurePiece>() else {
                continue;
            };
            collect_piece_chests(piece, &mut raw);
        }

        let (decoration_step, structure_index) = self.kind.decoration_coordinates();
        let mut next_ordinal_by_chunk = HashMap::<(i32, i32), i32>::new();
        let mut visible = Vec::<Chest>::new();
        let mut index_by_position = HashMap::<(i32, i32, i32), usize>::new();
        for chest in raw {
            let chest_chunk_x = chest.x.div_euclid(16);
            let chest_chunk_z = chest.z.div_euclid(16);
            let ordinal = next_ordinal_by_chunk
                .entry((chest_chunk_x, chest_chunk_z))
                .or_insert(0);
            let current_ordinal = *ordinal;
            *ordinal += 1;
            let loot_seed = container_loot_seed(
                self.world_seed,
                chest_chunk_x,
                chest_chunk_z,
                structure_index,
                decoration_step,
                current_ordinal,
                ContainerSeedShortcut::Direct,
            )?;
            let prediction = Chest {
                structure_chunk_x: chunk_x,
                structure_chunk_z: chunk_z,
                x: chest.x,
                y: chest.y,
                z: chest.z,
                loot_table: chest.loot_table,
                ordinal: current_ordinal,
                loot_seed,
            };
            let key = (prediction.x, prediction.y, prediction.z);
            if let Some(index) = index_by_position.get(&key).copied() {
                visible[index] = prediction;
            } else {
                index_by_position.insert(key, visible.len());
                visible.push(prediction);
            }
        }

        Ok(Scan {
            valid_structure: true,
            chests: visible,
        })
    }

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
    fn scan_desert_pyramid(
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

        let (structure_index, decoration_step) = self.kind.decoration_coordinates();
        let mut chests = Vec::with_capacity(4);
        for (ordinal, (local_x, local_z)) in [(10, 8), (12, 10), (10, 12), (8, 10)]
            .into_iter()
            .enumerate()
        {
            let loot_seed = container_loot_seed(
                self.world_seed,
                chunk_x,
                chunk_z,
                structure_index,
                decoration_step,
                ordinal as i32,
                ContainerSeedShortcut::DesertPyramid,
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
    fn scan_igloo(
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

        let (structure_index, decoration_step) = self.kind.decoration_coordinates();
        let loot_seed = container_loot_seed(
            self.world_seed,
            chunk_x,
            chunk_z,
            structure_index,
            decoration_step,
            1,
            ContainerSeedShortcut::Direct,
        )?;

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

    /// Scans a village candidate chunk.
    ///
    /// Mirrors the Java main's weighted structure-set selection for the
    /// five village variants: the placement random draws `nextInt(remaining)`,
    /// the candidate variant is probed with its own biome tag and, when
    /// invalid, removed before the next draw (biome fallback). Chest loot
    /// seeds use the `Direct` shortcut with the variant's decoration index.
    fn scan_village(
        &self,
        chunk_x: i32,
        chunk_z: i32,
        sampler: &mut MultiNoiseSampler<'_>,
    ) -> Result<Scan, Error> {
        let min_x = chunk_x
            .checked_mul(16)
            .ok_or_else(|| Error::Worldgen("village chunk x overflowed".to_owned()))?;
        let min_z = chunk_z
            .checked_mul(16)
            .ok_or_else(|| Error::Worldgen("village chunk z overflowed".to_owned()))?;

        let mut random = create_chunk_random(self.world_seed, chunk_x, chunk_z);
        let mut remaining: Vec<(Structure, StructureKeys, i32, &'static [&'static str])> = vec![
            (
                Structure::VILLAGE_DESERT,
                StructureKeys::VillageDesert,
                21,
                structure_biomes(&Structure::VILLAGE_DESERT),
            ),
            (
                Structure::VILLAGE_PLAINS,
                StructureKeys::VillagePlains,
                22,
                structure_biomes(&Structure::VILLAGE_PLAINS),
            ),
            (
                Structure::VILLAGE_SAVANNA,
                StructureKeys::VillageSavanna,
                23,
                structure_biomes(&Structure::VILLAGE_SAVANNA),
            ),
            (
                Structure::VILLAGE_SNOWY,
                StructureKeys::VillageSnowy,
                24,
                structure_biomes(&Structure::VILLAGE_SNOWY),
            ),
            (
                Structure::VILLAGE_TAIGA,
                StructureKeys::VillageTaiga,
                25,
                structure_biomes(&Structure::VILLAGE_TAIGA),
            ),
        ];
        let mut selected: Option<(Structure, StructureKeys, i32)> = None;
        let mut probe_heights = ColumnHeightSampler::new(&self.generator, min_x, min_z);
        while !remaining.is_empty() {
            let choice = random.next_bounded_i32(remaining.len() as i32) as usize;
            let (structure, key, index, biomes) = remaining.swap_remove(choice);
            let probe_structure = Structure {
                size: Some(0),
                ..structure
            };
            let mut probe_context = StructureGeneratorContext {
                seed: self.world_seed,
                chunk_x,
                chunk_z,
                random: create_chunk_random(self.world_seed, chunk_x, chunk_z),
                sea_level: self.kind.sea_level(),
                min_y: self.kind.min_y(),
                height_sampler: Some(&mut probe_heights),
                structure_key: Some(key),
            };
            let Some(probe) = village_jigsaw::generate_village_position(
                probe_structure
                    .start_pool
                    .expect("village structures have a start pool"),
                0,
                i32::from(
                    probe_structure
                        .start_height
                        .unwrap_or(self.kind.sea_level() as i16),
                ),
                probe_structure.project_start_to_heightmap.is_some(),
                probe_structure.max_distance_from_center.unwrap_or(80),
                true,
                &mut probe_context,
            ) else {
                continue;
            };
            if !self.biome_is_valid(probe.start_pos.0, biomes, sampler) {
                continue;
            }
            selected = Some((structure, key, index));
            break;
        }
        let Some((structure, key, index)) = selected else {
            return Ok(invalid_scan());
        };
        let mut heights = ColumnHeightSampler::new(&self.generator, min_x, min_z);
        let position = village_jigsaw::generate_village_position(
            structure
                .start_pool
                .expect("village structures have a start pool"),
            structure.size.expect("village structures have a size"),
            i32::from(
                structure
                    .start_height
                    .unwrap_or(self.kind.sea_level() as i16),
            ),
            structure.project_start_to_heightmap.is_some(),
            structure.max_distance_from_center.unwrap_or(80),
            true,
            &mut StructureGeneratorContext {
                seed: self.world_seed,
                chunk_x,
                chunk_z,
                random: create_chunk_random(self.world_seed, chunk_x, chunk_z),
                sea_level: self.kind.sea_level(),
                min_y: self.kind.min_y(),
                height_sampler: Some(&mut heights),
                structure_key: Some(key),
            },
        )
        .ok_or_else(|| Error::Worldgen("village failed full placement".to_owned()))?;

        let collector = position
            .collector
            .lock()
            .map_err(|_| Error::Worldgen("village piece collector was poisoned".to_owned()))?;
        let mut raw = Vec::new();
        for piece in &collector.pieces {
            let Some(piece) = piece.as_any().downcast_ref::<PoolElementStructurePiece>() else {
                continue;
            };
            collect_piece_chests(piece, &mut raw);
        }

        let mut next_ordinal_by_chunk = HashMap::<(i32, i32), i32>::new();
        let mut visible = Vec::<Chest>::new();
        let mut index_by_position = HashMap::<(i32, i32, i32), usize>::new();
        for chest in raw {
            let chest_chunk_x = chest.x.div_euclid(16);
            let chest_chunk_z = chest.z.div_euclid(16);
            let ordinal = next_ordinal_by_chunk
                .entry((chest_chunk_x, chest_chunk_z))
                .or_insert(0);
            let current_ordinal = *ordinal;
            *ordinal += 1;
            let loot_seed = container_loot_seed(
                self.world_seed,
                chest_chunk_x,
                chest_chunk_z,
                index,
                4,
                current_ordinal,
                ContainerSeedShortcut::Direct,
            )?;
            let prediction = Chest {
                structure_chunk_x: chunk_x,
                structure_chunk_z: chunk_z,
                x: chest.x,
                y: chest.y,
                z: chest.z,
                loot_table: chest.loot_table,
                ordinal: current_ordinal,
                loot_seed,
            };
            let key = (prediction.x, prediction.y, prediction.z);
            if let Some(position) = index_by_position.get(&key).copied() {
                visible[position] = prediction;
            } else {
                index_by_position.insert(key, visible.len());
                visible.push(prediction);
            }
        }

        Ok(Scan {
            valid_structure: true,
            chests: visible,
        })
    }

    fn scan_pillager_outpost(
        &self,
        chunk_x: i32,
        chunk_z: i32,
        sampler: &mut MultiNoiseSampler<'_>,
    ) -> Result<Scan, Error> {
        if !pillager_frequency_passes(self.world_seed, chunk_x, chunk_z) {
            return Ok(invalid_scan());
        }
        if has_village_nearby(self.world_seed, chunk_x, chunk_z) {
            return Ok(invalid_scan());
        }
        let min_x = chunk_x
            .checked_mul(16)
            .ok_or_else(|| Error::Worldgen("pillager outpost chunk x overflowed".to_owned()))?;
        let min_z = chunk_z
            .checked_mul(16)
            .ok_or_else(|| Error::Worldgen("pillager outpost chunk z overflowed".to_owned()))?;

        let structure = self.kind.structure();
        let mut probe_heights = ColumnHeightSampler::new(&self.generator, min_x, min_z);
        let mut probe_context = StructureGeneratorContext {
            seed: self.world_seed,
            chunk_x,
            chunk_z,
            random: create_chunk_random(self.world_seed, chunk_x, chunk_z),
            sea_level: self.kind.sea_level(),
            min_y: self.kind.min_y(),
            height_sampler: Some(&mut probe_heights),
            structure_key: Some(self.kind.structure_key()),
        };
        let probe_structure = Structure {
            size: Some(0),
            ..structure
        };
        let Some(probe) = village_jigsaw::generate_village_position(
            probe_structure
                .start_pool
                .expect("pillager outpost has a start pool"),
            0,
            i32::from(
                probe_structure
                    .start_height
                    .unwrap_or(self.kind.sea_level() as i16),
            ),
            probe_structure.project_start_to_heightmap.is_some(),
            probe_structure.max_distance_from_center.unwrap_or(80),
            true,
            &mut probe_context,
        ) else {
            return Ok(invalid_scan());
        };
        if !self.biome_is_valid(probe.start_pos.0, self.valid_biomes, sampler) {
            return Ok(invalid_scan());
        }
        let mut heights = ColumnHeightSampler::new(&self.generator, min_x, min_z);
        let position = village_jigsaw::generate_village_position(
            structure
                .start_pool
                .expect("pillager outpost has a start pool"),
            structure.size.expect("pillager outpost has a size"),
            i32::from(
                structure
                    .start_height
                    .unwrap_or(self.kind.sea_level() as i16),
            ),
            structure.project_start_to_heightmap.is_some(),
            structure.max_distance_from_center.unwrap_or(80),
            true,
            &mut StructureGeneratorContext {
                seed: self.world_seed,
                chunk_x,
                chunk_z,
                random: create_chunk_random(self.world_seed, chunk_x, chunk_z),
                sea_level: self.kind.sea_level(),
                min_y: self.kind.min_y(),
                height_sampler: Some(&mut heights),
                structure_key: Some(self.kind.structure_key()),
            },
        )
        .ok_or_else(|| Error::Worldgen("pillager outpost failed full placement".to_owned()))?;

        let collector = position
            .collector
            .lock()
            .map_err(|_| Error::Worldgen("pillager piece collector was poisoned".to_owned()))?;
        let mut raw = Vec::new();
        for piece in &collector.pieces {
            let Some(piece) = piece.as_any().downcast_ref::<PoolElementStructurePiece>() else {
                continue;
            };
            collect_piece_chests(piece, &mut raw);
        }

        let (decoration_step, structure_index) = self.kind.decoration_coordinates();
        let mut next_ordinal_by_chunk = HashMap::<(i32, i32), i32>::new();
        let mut visible = Vec::<Chest>::new();
        let mut index_by_position = HashMap::<(i32, i32, i32), usize>::new();
        for chest in raw {
            let chest_chunk_x = chest.x.div_euclid(16);
            let chest_chunk_z = chest.z.div_euclid(16);
            let ordinal = next_ordinal_by_chunk
                .entry((chest_chunk_x, chest_chunk_z))
                .or_insert(0);
            let current_ordinal = *ordinal;
            *ordinal += 1;
            let loot_seed = container_loot_seed(
                self.world_seed,
                chest_chunk_x,
                chest_chunk_z,
                structure_index,
                decoration_step,
                current_ordinal,
                ContainerSeedShortcut::Direct,
            )?;
            let prediction = Chest {
                structure_chunk_x: chunk_x,
                structure_chunk_z: chunk_z,
                x: chest.x,
                y: chest.y,
                z: chest.z,
                loot_table: chest.loot_table,
                ordinal: current_ordinal,
                loot_seed,
            };
            let key = (prediction.x, prediction.y, prediction.z);
            if let Some(existing) = index_by_position.get(&key).copied() {
                visible[existing] = prediction;
            } else {
                index_by_position.insert(key, visible.len());
                visible.push(prediction);
            }
        }

        Ok(Scan {
            valid_structure: true,
            chests: visible,
        })
    }

    fn context(&self, chunk_x: i32, chunk_z: i32) -> StructureGeneratorContext<'_> {
        StructureGeneratorContext {
            seed: self.world_seed,
            chunk_x,
            chunk_z,
            random: create_chunk_random(self.world_seed, chunk_x, chunk_z),
            sea_level: self.kind.sea_level(),
            min_y: self.kind.min_y(),
            height_sampler: None,
            structure_key: Some(self.kind.structure_key()),
        }
    }

    fn bastion_reached_in_weighted_selection(
        &self,
        chunk_x: i32,
        chunk_z: i32,
        sampler: &mut MultiNoiseSampler<'_>,
    ) -> Result<bool, Error> {
        let mut selection_random = create_chunk_random(self.world_seed, chunk_x, chunk_z);
        let bastion_selected_first = selection_random.next_bounded_i32(5) >= 2;
        if bastion_selected_first {
            return Ok(true);
        }

        let block_x = chunk_x.checked_mul(16).ok_or_else(|| {
            Error::Worldgen("fortress biome probe x coordinate overflowed".to_owned())
        })?;
        let block_z = chunk_z.checked_mul(16).ok_or_else(|| {
            Error::Worldgen("fortress biome probe z coordinate overflowed".to_owned())
        })?;
        let fortress_start = Vector3::new(block_x, 64, block_z);
        Ok(!self.biome_is_valid(fortress_start, self.fortress_biomes, sampler))
    }

    fn biome_is_valid(
        &self,
        position: Vector3<i32>,
        valid_biomes: &[&str],
        sampler: &mut MultiNoiseSampler<'_>,
    ) -> bool {
        let biome = self.kind.biome_supplier().biome(
            biome_coords::from_block(position.x),
            biome_coords::from_block(position.y),
            biome_coords::from_block(position.z),
            sampler,
        );
        // Compare registry ids, not numeric ids: the fork's generated tag
        // table carries off-by-one biome ids for a few tags (e.g. taiga is
        // listed as 56 while its enum index is 55).
        valid_biomes.contains(&biome.registry_id)
    }
}

fn structure_biomes(structure: &Structure) -> &'static [&'static str] {
    let biome_tag = structure
        .biomes
        .strip_prefix('#')
        .unwrap_or(structure.biomes);
    get_tag_values(RegistryKey::WorldgenBiome, biome_tag)
        .expect("vanilla structure biome tag must exist")
}

const fn invalid_scan() -> Scan {
    Scan {
        valid_structure: false,
        chests: Vec::new(),
    }
}

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

struct RawChest {
    x: i32,
    y: i32,
    z: i32,
    loot_table: String,
}

fn collect_piece_chests(piece: &PoolElementStructurePiece, output: &mut Vec<RawChest>) {
    let origin = piece.pos.0;
    piece.element.for_each_template(|_, _, _, template| {
        let (corner_x, corner_z) = piece.rotation.rotate_offset(
            template.size.x.saturating_sub(1),
            template.size.z.saturating_sub(1),
        );
        let placement_origin = Vector3::new(
            origin.x + corner_x.min(0),
            origin.y,
            origin.z + corner_z.min(0),
        );
        for block in &template.blocks {
            let palette = &template.palette[block.state as usize];
            if palette.name != "minecraft:chest" {
                continue;
            }
            let local = piece.rotation.transform_pos(block.pos, template.size);
            let world = Vector3::new(
                placement_origin.x + local.x,
                placement_origin.y + local.y,
                placement_origin.z + local.z,
            );
            let loot_table = block
                .nbt
                .as_ref()
                .and_then(|nbt| nbt.get_string("LootTable"))
                .unwrap_or_default()
                .to_owned();
            output.push(RawChest {
                x: world.x,
                y: world.y,
                z: world.z,
                loot_table,
            });
        }
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    #[test]
    fn scans_known_26_1_2_cities() {
        let scanner = Scanner::new(114514, Kind::AncientCity);
        let scans = scanner
            .scan_many([(96, 5), (244, 171)])
            .expect("scan known cities");
        let first = &scans[0];
        assert!(first.valid_structure);
        assert!(first.chests.iter().any(|chest| {
            chest.x == 1450
                && chest.y == -35
                && chest.z == 137
                && chest.loot_table == "minecraft:chests/ancient_city"
                && chest.loot_seed == 1_392_286_922_750_350_146
                && chest.ordinal == 0
        }));

        let second = &scans[1];
        assert!(second.valid_structure);
        assert!(second.chests.iter().any(|chest| {
            chest.x == 3965
                && chest.y == -37
                && chest.z == 2755
                && chest.loot_table == "minecraft:chests/ancient_city"
                && chest.loot_seed == -5_503_126_436_529_563_106
        }));
    }

    #[test]
    fn scans_known_26_1_2_bastions() {
        let scanner = Scanner::new(0, Kind::BastionRemnant);
        let scans = scanner
            .scan_many([(11, -14), (-27, -10), (62, 32)])
            .expect("scan known bastions");
        assert!(scans.iter().all(|scan| scan.valid_structure));
        assert_eq!(
            scans
                .iter()
                .map(|scan| scan.chests.len())
                .collect::<Vec<_>>(),
            [3, 11, 6]
        );
        assert_eq!(
            scans[0].chests.first(),
            Some(&Chest {
                structure_chunk_x: 11,
                structure_chunk_z: -14,
                x: 180,
                y: 80,
                z: -233,
                loot_table: "minecraft:chests/bastion_bridge".to_owned(),
                ordinal: 0,
                loot_seed: 1_335_123_538_721_756_194,
            })
        );
        assert_eq!(
            scans[1].chests.first(),
            Some(&Chest {
                structure_chunk_x: -27,
                structure_chunk_z: -10,
                x: -428,
                y: 35,
                z: -189,
                loot_table: "minecraft:chests/bastion_other".to_owned(),
                ordinal: 0,
                loot_seed: -5_513_880_696_554_537_352,
            })
        );
        assert_eq!(
            scans[2].chests.first(),
            Some(&Chest {
                structure_chunk_x: 62,
                structure_chunk_z: 32,
                x: 1011,
                y: 35,
                z: 496,
                loot_table: "minecraft:chests/bastion_treasure".to_owned(),
                ordinal: 0,
                loot_seed: -6_403_023_197_147_397_919,
            })
        );

        let tables = scans
            .iter()
            .flat_map(|scan| scan.chests.iter())
            .map(|chest| chest.loot_table.as_str())
            .collect::<HashSet<_>>();
        assert_eq!(
            tables,
            HashSet::from([
                "minecraft:chests/bastion_bridge",
                "minecraft:chests/bastion_hoglin_stable",
                "minecraft:chests/bastion_other",
                "minecraft:chests/bastion_treasure",
            ])
        );
    }

    #[test]
    fn scans_known_26_1_2_desert_pyramids() {
        let scanner = Scanner::new(0, Kind::DesertPyramid);
        // Seed 0: three valid pyramids and six candidates rejected by the
        // vanilla sea-level corner check (some corner below the sea level).
        let scans = scanner
            .scan_many([
                (0, -188),
                (77, -213),
                (81, -254),
                (22, -146),
                (-30, -160),
                (-10, -178),
                (98, -155),
                (102, -224),
                (105, -246),
            ])
            .expect("scan desert pyramid candidates");
        for scan in &scans[3..] {
            assert!(!scan.valid_structure, "candidate must be rejected");
            assert!(scan.chests.is_empty(), "rejected candidate has chests");
        }
        let scans = &scans[..3];
        let expected = [
            (
                "minecraft:chests/desert_pyramid",
                [
                    (10, 59, -2996, -5_568_029_752_813_165_272),
                    (12, 59, -2998, 8_612_763_612_274_328_067),
                    (10, 59, -3000, 410_913_108_922_281_890),
                    (8, 59, -2998, -6_529_954_051_122_263_735),
                ],
            ),
            (
                "minecraft:chests/desert_pyramid",
                [
                    (1244, 60, -3398, 192_079_748_099_134_926),
                    (1242, 60, -3396, -369_207_723_137_014_054),
                    (1240, 60, -3398, 1_366_626_509_293_417_282),
                    (1242, 60, -3400, 2_864_047_697_517_889_560),
                ],
            ),
            (
                "minecraft:chests/desert_pyramid",
                [
                    (1304, 52, -4054, 8_475_396_442_896_426_591),
                    (1306, 52, -4052, -164_227_586_464_969_558),
                    (1308, 52, -4054, -6_884_729_539_475_924_943),
                    (1306, 52, -4056, 5_000_275_533_034_043_386),
                ],
            ),
        ];
        assert_eq!(scans.len(), expected.len());
        for (scan, (loot_table, chests)) in scans.iter().zip(expected) {
            assert!(scan.valid_structure);
            let actual = scan
                .chests
                .iter()
                .map(|chest| {
                    (
                        chest.loot_table.as_str(),
                        (chest.x, chest.y, chest.z, chest.loot_seed),
                    )
                })
                .collect::<Vec<_>>();
            let wanted = chests
                .iter()
                .map(|(x, y, z, seed)| (loot_table, (*x, *y, *z, *seed)))
                .collect::<Vec<_>>();
            assert_eq!(actual, wanted);
        }
    }

    #[test]
    fn scans_known_26_1_2_igloos() {
        let scanner = Scanner::new(0, Kind::Igloo);
        // Seed 0: three igloos with basements (chest vectors from the vanilla
        // 26.1.2 placement run) and three valid igloos without a basement.
        let scans = scanner
            .scan_many([
                (98, 192),
                (238, -29),
                (-110, 246),
                (-12, 231),
                (-46, 242),
                (-214, 141),
            ])
            .expect("scan igloo candidates");
        let expected_chests = [
            ((1569, 122, 3076), -7_862_992_963_971_781_551),
            ((3813, 48, -458), -3_865_222_752_920_655_871),
            ((-1755, 50, 3942), 1_861_016_387_536_410_190),
        ];
        for (scan, ((x, y, z), seed)) in scans.iter().take(3).zip(expected_chests) {
            assert!(scan.valid_structure);
            assert_eq!(scan.chests.len(), 1);
            let chest = &scan.chests[0];
            assert_eq!((chest.x, chest.y, chest.z), (x, y, z));
            assert_eq!(chest.loot_seed, seed);
            assert_eq!(chest.loot_table, "minecraft:chests/igloo_chest");
            assert_eq!(chest.ordinal, 0);
        }
        for scan in &scans[3..] {
            assert!(scan.valid_structure);
            assert!(
                scan.chests.is_empty(),
                "igloo without basement must have no chests"
            );
        }
    }

    #[test]
    fn scans_known_26_1_2_snowy_village() {
        // Chunk (-114,290): a snowy village on a ravine edge. The free-space
        // region must use the village's max-distance (80) Y extent so ravine
        // candidates at y24 are rejected; the interior-expansion house
        // snowy_small_house_6 (y72-80) then attaches and its chest appears.
        let scanner = Scanner::new(0, Kind::Village);
        let scans = scanner
            .scan_many([(-114i32, 290i32)])
            .expect("scan village");
        assert_eq!(scans.len(), 1);
        assert!(scans[0].valid_structure);
        // Chest order follows the vanilla piece/block scan order.
        let expected_positions = [
            (-1823, 133, 4655),
            (-1803, 131, 4655),
            (-1787, 131, 4681),
            (-1845, 73, 4613),
        ];
        let known_seeds = [
            8_698_000_211_807_766_173,
            -6_504_443_412_966_562_273,
            2_903_910_160_224_306_417,
        ];
        assert_eq!(scans[0].chests.len(), expected_positions.len());
        for (chest, (x, y, z)) in scans[0].chests.iter().zip(expected_positions) {
            assert_eq!((chest.x, chest.y, chest.z), (x, y, z));
            assert_eq!(
                chest.loot_table,
                "minecraft:chests/village/village_snowy_house"
            );
            assert_eq!(chest.ordinal, 0);
        }
        for (chest, seed) in scans[0].chests.iter().zip(known_seeds) {
            assert_eq!(chest.loot_seed, seed);
        }
    }

    #[test]
    fn scans_known_26_1_2_villages() {
        let scanner = Scanner::new(0, Kind::Village);
        // Seed 0: a savanna village at (38,45) with five chests and a plains
        // village at (17,59) with two chests (vectors from the vanilla
        // 26.1.2 placement run, including variant indices 23 and 22).
        let scans = scanner
            .scan_many([(38, 45), (17, 59)])
            .expect("scan villages");
        // Chest order follows the vanilla piece/block scan order (not
        // coordinate order): (613,75,715) is the first cartographer chest.
        let expected = [
            vec![
                (613, 75, 715, 7_894_754_405_246_038_683),
                (629, 74, 724, 7_967_509_563_249_290_458),
                (634, 74, 736, 412_383_135_729_192_107),
                (613, 72, 749, -5_741_561_169_428_246_725),
                (566, 72, 715, -8_360_261_121_396_786_299),
            ],
            vec![
                (283, 69, 944, 1_415_882_058_948_058_970),
                (290, 75, 900, -8_102_066_785_630_644_298),
            ],
        ];
        for (scan, chests) in scans.iter().zip(expected) {
            assert!(scan.valid_structure);
            let actual = scan
                .chests
                .iter()
                .map(|chest| (chest.x, chest.y, chest.z, chest.loot_seed))
                .collect::<Vec<_>>();
            assert_eq!(actual, chests);
        }
    }

    #[test]
    fn scans_known_26_1_2_pillager_outpost() {
        // World-level valid pillager for seed 0: chunk (-51,70) passes
        // frequency (legacy_type_1, 0.2) and the 10-chunk village
        // exclusion, then generates one chest. This vector was captured
        // from the Rust scanner with placement filters enabled and matches
        // the Java direct generation for the same chunk (except the
        // placement filters).
        let scanner = Scanner::new(0, Kind::PillagerOutpost);
        let scans = scanner.scan_many([(-51, 70)]).expect("scan pillager");
        assert_eq!(scans.len(), 1);
        let scan = &scans[0];
        assert!(scan.valid_structure, "pillager should be valid");
        assert_eq!(scan.chests.len(), 1);
        let chest = &scan.chests[0];
        assert_eq!(chest.loot_table, "minecraft:chests/pillager_outpost");
        assert_eq!(
            (chest.x, chest.y, chest.z, chest.loot_seed, chest.ordinal),
            (-826, 77, 1110, -638836315418230144, 1),
            "pillager chest vector: {:?}",
            chest
        );
    }

    #[test]
    fn pillager_outpost_36_103_is_filtered_by_frequency() {
        // Java's `VillageAndFortressIntegrationTest` probes pillager at
        // 36,103 via direct `generateSelectedStructure`, bypassing the
        // placement frequency check, and finds one chest at (566,84,1657).
        // For world scans `isStructurePlacementChunk` rejects 36,103
        // (legacy_type_1), so the Rust scanner must report it as invalid.
        let scanner = Scanner::new(0, Kind::PillagerOutpost);
        let scans = scanner.scan_many([(36, 103)]).expect("scan pillager");
        assert_eq!(scans.len(), 1);
        assert!(
            !scans[0].valid_structure,
            "36,103 should be invalid for world scan due to frequency"
        );
        assert!(scans[0].chests.is_empty());
    }
}
