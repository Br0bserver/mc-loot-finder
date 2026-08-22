mod chests;
mod jigsaw_scan;
mod profile;
mod single_piece;
use crate::catalog::{CandidateStructure, DecorationSeedSpec, ScanKind, ScanSupport};
use crate::error::Error;
use pumpkin_data::{
    structures::Structure,
    tag::{RegistryKey, get_tag_values},
};
use pumpkin_util::{math::vector3::Vector3, random::RandomImpl, world_seed::Seed};
use pumpkin_world::{
    biome::BiomeSupplier,
    generation::{
        biome_coords,
        generator::{GeneratorInit, VanillaGenerator},
        noise::router::multi_noise_sampler::{MultiNoiseSampler, MultiNoiseSamplerBuilderOptions},
        structure::structures::{StructureGeneratorContext, create_chunk_random},
    },
};

const OVERWORLD_MIN_Y: i32 = -64;
const NETHER_MIN_Y: i32 = 0;
const OVERWORLD_SEA_LEVEL: i32 = 63;
const NETHER_SEA_LEVEL: i32 = 32;

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
    structure: &'static CandidateStructure,
    kind: ScanKind,
    generator: VanillaGenerator,
    valid_biomes: &'static [&'static str],
    fortress_biomes: &'static [&'static str],
}

impl Scanner {
    pub fn for_structure(
        structure: &'static CandidateStructure,
        world_seed: i64,
    ) -> Result<Self, Error> {
        let ScanSupport::Full(kind) = structure.support else {
            return Err(Error::Structure(format!(
                "Rust chests and find do not support {} yet",
                structure.name
            )));
        };
        Ok(Self::from_structure(world_seed, structure, kind))
    }

    #[cfg(test)]
    #[must_use]
    pub fn new(world_seed: i64, kind: ScanKind) -> Self {
        let structure = crate::catalog::CANDIDATE_STRUCTURES
            .iter()
            .find(|structure| structure.support == ScanSupport::Full(kind))
            .expect("every scan kind must have one catalog entry");
        Self::from_structure(world_seed, structure, kind)
    }

    fn from_structure(
        world_seed: i64,
        structure: &'static CandidateStructure,
        kind: ScanKind,
    ) -> Self {
        let runtime_structure = kind.structure();
        Self {
            world_seed,
            structure,
            kind,
            generator: VanillaGenerator::new(Seed(world_seed as u64), kind.dimension()),
            valid_biomes: structure_biomes(&runtime_structure),
            fortress_biomes: if kind == ScanKind::BastionRemnant {
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
        match self.kind {
            ScanKind::AncientCity | ScanKind::BastionRemnant => {
                self.scan_pumpkin_jigsaw(chunk_x, chunk_z, sampler)
            }
            ScanKind::DesertPyramid => self.scan_desert_pyramid(chunk_x, chunk_z, sampler),
            ScanKind::Igloo => self.scan_igloo(chunk_x, chunk_z, sampler),
            ScanKind::Village => self.scan_village(chunk_x, chunk_z, sampler),
            ScanKind::PillagerOutpost => self.scan_pillager_outpost(chunk_x, chunk_z, sampler),
        }
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

    fn decoration(&self) -> Result<DecorationSeedSpec, Error> {
        self.structure.decoration.ok_or_else(|| {
            Error::Worldgen(format!(
                "{} scanner has no static decoration seed specification",
                self.structure.name
            ))
        })
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

#[cfg(test)]
mod tests;
