use super::chests::{collect_position_chests, dedup_and_seed_chests};
use super::{Scan, Scanner, invalid_scan, structure_biomes};
use crate::catalog::{ContainerSeedShortcut, DecorationSeedSpec, ScanKind, VILLAGE_PLACEMENT};
use crate::error::Error;
use crate::placement;
use crate::random::LegacyRandom48;
use crate::surface_height::ColumnHeightSampler;
use crate::village_jigsaw;
use pumpkin_data::structures::{Structure, StructureKeys};
use pumpkin_util::random::RandomImpl;
use pumpkin_world::generation::{
    noise::router::multi_noise_sampler::MultiNoiseSampler,
    structure::{
        generate_structure_position,
        structures::{StructureGeneratorContext, create_chunk_random},
    },
};
const PILLAGER_FREQUENCY: f32 = 0.2;
const VILLAGE_EXCLUSION_RADIUS: i32 = 10;

fn has_village_nearby(world_seed: i64, chunk_x: i32, chunk_z: i32) -> bool {
    let radius = VILLAGE_EXCLUSION_RADIUS;
    for dx in -radius..=radius {
        for dz in -radius..=radius {
            let other_x = chunk_x + dx;
            let other_z = chunk_z + dz;
            if placement::is_placement_chunk(world_seed, other_x, other_z, VILLAGE_PLACEMENT) {
                return true;
            }
        }
    }
    false
}

fn pillager_frequency_passes(world_seed: i64, chunk_x: i32, chunk_z: i32) -> bool {
    let i = chunk_x >> 4;
    let j = chunk_z >> 4;
    let combined = i64::from(i << 4 ^ j) ^ world_seed;
    let mut random = LegacyRandom48::new(combined);
    let _ = random.next_int_unbounded();
    let bound = (1.0 / PILLAGER_FREQUENCY) as i32;
    random.next_int(bound) == 0
}

impl Scanner {
    pub(super) fn scan_pumpkin_jigsaw(
        &self,
        chunk_x: i32,
        chunk_z: i32,
        sampler: &mut MultiNoiseSampler<'_>,
    ) -> Result<Scan, Error> {
        if self.kind == ScanKind::BastionRemnant
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
        let raw = collect_position_chests(&position, "jigsaw")?;
        let visible =
            dedup_and_seed_chests(self.world_seed, raw, (chunk_x, chunk_z), self.decoration()?)?;

        Ok(Scan {
            valid_structure: true,
            chests: visible,
        })
    }

    /// Scans a village candidate chunk.
    ///
    /// Mirrors the Java main's weighted structure-set selection for the
    /// five village variants: the placement random draws `nextInt(remaining)`,
    /// the candidate variant is probed with its own biome tag and, when
    /// invalid, removed before the next draw (biome fallback). Chest loot
    /// seeds use the `Direct` shortcut with the variant's decoration index.
    pub(super) fn scan_village(
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
            structure.start_pool.ok_or_else(|| {
                Error::Worldgen("village structures have a start pool".to_owned())
            })?,
            structure
                .size
                .ok_or_else(|| Error::Worldgen("village structures have a size".to_owned()))?,
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

        let raw = collect_position_chests(&position, "village")?;

        let visible = dedup_and_seed_chests(
            self.world_seed,
            raw,
            (chunk_x, chunk_z),
            DecorationSeedSpec {
                structure_index: index,
                step: 4,
                shortcut: ContainerSeedShortcut::Direct,
            },
        )?;

        Ok(Scan {
            valid_structure: true,
            chests: visible,
        })
    }
    pub(super) fn scan_pillager_outpost(
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
                .ok_or_else(|| Error::Worldgen("pillager outpost has a start pool".to_owned()))?,
            structure
                .size
                .ok_or_else(|| Error::Worldgen("pillager outpost has a size".to_owned()))?,
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

        let raw = collect_position_chests(&position, "pillager")?;

        let visible =
            dedup_and_seed_chests(self.world_seed, raw, (chunk_x, chunk_z), self.decoration()?)?;

        Ok(Scan {
            valid_structure: true,
            chests: visible,
        })
    }
}
