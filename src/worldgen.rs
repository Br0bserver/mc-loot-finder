mod buried_treasure;
mod chests;
mod jigsaw_scan;
mod shipwreck;
mod single_piece;
mod template_data;
mod template_scan;

#[cfg(test)]
mod tests;

use glam::IVec3;
use rustc_hash::FxHashMap;
use std::sync::LazyLock;
use steel_registry::biome::BiomeRef;
use steel_registry::template_pool::{TemplateData, TemplatePoolData};
use steel_registry::vanilla_template_pools::{vanilla_template_pools, vanilla_templates};
use steel_registry::{REGISTRY, RegistryExt, init_vanilla_registry};
use steel_utils::Identifier;
use steel_utils::random::legacy_random::LegacyRandom;
use steel_utils::random::{PositionalRandom, Random, RandomSplitter};
use steel_worldgen::biomes::{BiomeSourceKind, ChunkBiomeSampler};
use steel_worldgen::density::traits::DimensionNoises;
use steel_worldgen::density_functions::nether::{NetherColumnCache, NetherNoises};
use steel_worldgen::density_functions::overworld::{OverworldColumnCache, OverworldNoises};
use steel_worldgen::noise::LazyAquifer;
use steel_worldgen::structure::{ColumnBlock, GenerationContext, StructureGenerationContext};

use crate::catalog::{CandidateStructure, DecorationSeedSpec, ScanKind, ScanSupport};
use crate::error::Error;

static TEMPLATE_POOLS: LazyLock<FxHashMap<Identifier, TemplatePoolData>> = LazyLock::new(|| {
    vanilla_template_pools()
        .into_iter()
        .map(|pool| (pool.key.clone(), pool))
        .collect()
});

static TEMPLATES: LazyLock<FxHashMap<Identifier, TemplateData>> =
    LazyLock::new(|| vanilla_templates().into_iter().collect());

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
    dimension: &'static str,
    overworld_noises: Option<OverworldNoises>,
    nether_noises: Option<NetherNoises>,
    biome_source: BiomeSourceKind,
    splitter: RandomSplitter,
}

pub struct OverworldScannerContext<'src> {
    seed: i64,
    chunk_x: i32,
    chunk_z: i32,
    sea_level: i32,
    noises: &'src OverworldNoises,
    splitter: &'src RandomSplitter,
    biome_sampler: ChunkBiomeSampler<'src>,
    height_cache: OverworldColumnCache,
    aquifer: LazyAquifer<'src, OverworldNoises>,
    surface_y_cache: Option<i32>,
    height_cache_grid_ready: bool,
}

impl<'src> OverworldScannerContext<'src> {
    fn as_generation_context(&mut self) -> GenerationContext<'_, 'src, OverworldNoises> {
        GenerationContext::new(
            self.seed,
            self.chunk_x,
            self.chunk_z,
            self.sea_level,
            self.noises,
            self.splitter,
            &TEMPLATE_POOLS,
            &TEMPLATES,
            &mut self.biome_sampler,
            &mut self.height_cache,
            &mut self.aquifer,
            &mut self.surface_y_cache,
            &mut self.height_cache_grid_ready,
        )
    }
}

impl StructureGenerationContext for OverworldScannerContext<'_> {
    fn seed(&self) -> i64 {
        self.seed
    }
    fn chunk_x(&self) -> i32 {
        self.chunk_x
    }
    fn chunk_z(&self) -> i32 {
        self.chunk_z
    }
    fn chunk_min_x(&self) -> i32 {
        self.chunk_x * 16
    }
    fn chunk_min_z(&self) -> i32 {
        self.chunk_z * 16
    }
    fn center_block_x(&self) -> i32 {
        self.chunk_x * 16 + 8
    }
    fn center_block_z(&self) -> i32 {
        self.chunk_z * 16 + 8
    }
    fn sea_level(&self) -> i32 {
        self.sea_level
    }
    fn min_y(&self) -> i32 {
        -64
    }
    fn height(&self) -> i32 {
        384
    }
    fn template_pools(&self) -> &FxHashMap<Identifier, TemplatePoolData> {
        &TEMPLATE_POOLS
    }
    fn templates(&self) -> &FxHashMap<Identifier, TemplateData> {
        &TEMPLATES
    }
    fn base_height(&mut self, x: i32, z: i32, ocean_floor: bool) -> i32 {
        self.as_generation_context().base_height(x, z, ocean_floor)
    }
    fn base_height_full(&mut self, x: i32, z: i32, ocean_floor: bool) -> i32 {
        self.as_generation_context()
            .base_height_full(x, z, ocean_floor)
    }
    fn biome_at(&mut self, block_x: i32, block_y: i32, block_z: i32) -> BiomeRef {
        self.as_generation_context()
            .biome_at(block_x, block_y, block_z)
    }
    fn column_state(&mut self, x: i32, y: i32, z: i32) -> ColumnBlock {
        self.as_generation_context().column_state(x, y, z)
    }
    fn surface_y(&mut self) -> i32 {
        self.as_generation_context().surface_y()
    }
    fn terrain_surface_height(&self, x: i32, z: i32, ocean_floor: bool) -> i32 {
        let mut cache = OverworldColumnCache::default();
        cache.init_grid(x & !15, z & !15, self.noises);
        let mut aq =
            LazyAquifer::<OverworldNoises>::new(x & !15, z & !15, self.splitter, self.noises);
        let aquifer = aq.ensure(&mut cache);
        steel_worldgen::utils::column_base_height::<OverworldNoises>(
            &mut cache,
            self.noises,
            aquifer,
            x,
            z,
            ocean_floor,
        )
    }
    fn terrain_is_opaque(&self, x: i32, y: i32, z: i32, ocean_floor: bool) -> bool {
        let mut cache = OverworldColumnCache::default();
        cache.init_grid(x & !15, z & !15, self.noises);
        let mut aq =
            LazyAquifer::<OverworldNoises>::new(x & !15, z & !15, self.splitter, self.noises);
        let aquifer = aq.ensure(&mut cache);
        let density = steel_worldgen::utils::column_interpolated_density::<OverworldNoises>(
            &mut cache,
            self.noises,
            x,
            y,
            z,
            4,
            8,
        );
        match aquifer.compute_substance(self.noises, x, y, z, density) {
            steel_worldgen::noise::AquiferResult::Solid => true,
            steel_worldgen::noise::AquiferResult::Fluid(_) => !ocean_floor,
            steel_worldgen::noise::AquiferResult::Air => false,
        }
    }
}

pub struct NetherScannerContext<'src> {
    seed: i64,
    chunk_x: i32,
    chunk_z: i32,
    sea_level: i32,
    noises: &'src NetherNoises,
    splitter: &'src RandomSplitter,
    biome_sampler: ChunkBiomeSampler<'src>,
    height_cache: NetherColumnCache,
    aquifer: LazyAquifer<'src, NetherNoises>,
    surface_y_cache: Option<i32>,
    height_cache_grid_ready: bool,
}

impl<'src> NetherScannerContext<'src> {
    fn as_generation_context(&mut self) -> GenerationContext<'_, 'src, NetherNoises> {
        GenerationContext::new(
            self.seed,
            self.chunk_x,
            self.chunk_z,
            self.sea_level,
            self.noises,
            self.splitter,
            &TEMPLATE_POOLS,
            &TEMPLATES,
            &mut self.biome_sampler,
            &mut self.height_cache,
            &mut self.aquifer,
            &mut self.surface_y_cache,
            &mut self.height_cache_grid_ready,
        )
    }
}

impl StructureGenerationContext for NetherScannerContext<'_> {
    fn seed(&self) -> i64 {
        self.seed
    }
    fn chunk_x(&self) -> i32 {
        self.chunk_x
    }
    fn chunk_z(&self) -> i32 {
        self.chunk_z
    }
    fn chunk_min_x(&self) -> i32 {
        self.chunk_x * 16
    }
    fn chunk_min_z(&self) -> i32 {
        self.chunk_z * 16
    }
    fn center_block_x(&self) -> i32 {
        self.chunk_x * 16 + 8
    }
    fn center_block_z(&self) -> i32 {
        self.chunk_z * 16 + 8
    }
    fn sea_level(&self) -> i32 {
        self.sea_level
    }
    fn min_y(&self) -> i32 {
        0
    }
    fn height(&self) -> i32 {
        128
    }
    fn template_pools(&self) -> &FxHashMap<Identifier, TemplatePoolData> {
        &TEMPLATE_POOLS
    }
    fn templates(&self) -> &FxHashMap<Identifier, TemplateData> {
        &TEMPLATES
    }
    fn base_height(&mut self, x: i32, z: i32, ocean_floor: bool) -> i32 {
        self.as_generation_context().base_height(x, z, ocean_floor)
    }
    fn base_height_full(&mut self, x: i32, z: i32, ocean_floor: bool) -> i32 {
        self.as_generation_context()
            .base_height_full(x, z, ocean_floor)
    }
    fn biome_at(&mut self, block_x: i32, block_y: i32, block_z: i32) -> BiomeRef {
        self.as_generation_context()
            .biome_at(block_x, block_y, block_z)
    }
    fn column_state(&mut self, x: i32, y: i32, z: i32) -> ColumnBlock {
        self.as_generation_context().column_state(x, y, z)
    }
    fn surface_y(&mut self) -> i32 {
        self.as_generation_context().surface_y()
    }
    fn terrain_surface_height(&self, x: i32, z: i32, ocean_floor: bool) -> i32 {
        let mut cache = NetherColumnCache::default();
        cache.init_grid(x & !15, z & !15, self.noises);
        let mut aq = LazyAquifer::<NetherNoises>::new(x & !15, z & !15, self.splitter, self.noises);
        let aquifer = aq.ensure(&mut cache);
        steel_worldgen::utils::column_base_height::<NetherNoises>(
            &mut cache,
            self.noises,
            aquifer,
            x,
            z,
            ocean_floor,
        )
    }
    fn terrain_is_opaque(&self, x: i32, y: i32, z: i32, ocean_floor: bool) -> bool {
        let mut cache = NetherColumnCache::default();
        cache.init_grid(x & !15, z & !15, self.noises);
        let mut aq = LazyAquifer::<NetherNoises>::new(x & !15, z & !15, self.splitter, self.noises);
        let aquifer = aq.ensure(&mut cache);
        let density = steel_worldgen::utils::column_interpolated_density::<NetherNoises>(
            &mut cache,
            self.noises,
            x,
            y,
            z,
            4,
            8,
        );
        match aquifer.compute_substance(self.noises, x, y, z, density) {
            steel_worldgen::noise::AquiferResult::Solid => true,
            steel_worldgen::noise::AquiferResult::Fluid(_) => !ocean_floor,
            steel_worldgen::noise::AquiferResult::Air => false,
        }
    }
}

pub enum ScannerContext<'src> {
    Overworld(OverworldScannerContext<'src>),
    Nether(NetherScannerContext<'src>),
}

impl StructureGenerationContext for ScannerContext<'_> {
    fn seed(&self) -> i64 {
        match self {
            Self::Overworld(c) => c.seed(),
            Self::Nether(c) => c.seed(),
        }
    }
    fn chunk_x(&self) -> i32 {
        match self {
            Self::Overworld(c) => c.chunk_x(),
            Self::Nether(c) => c.chunk_z(),
        }
    }
    fn chunk_z(&self) -> i32 {
        match self {
            Self::Overworld(c) => c.chunk_z(),
            Self::Nether(c) => c.chunk_z(),
        }
    }
    fn chunk_min_x(&self) -> i32 {
        match self {
            Self::Overworld(c) => c.chunk_min_x(),
            Self::Nether(c) => c.chunk_min_x(),
        }
    }
    fn chunk_min_z(&self) -> i32 {
        match self {
            Self::Overworld(c) => c.chunk_min_z(),
            Self::Nether(c) => c.chunk_min_z(),
        }
    }
    fn center_block_x(&self) -> i32 {
        match self {
            Self::Overworld(c) => c.center_block_x(),
            Self::Nether(c) => c.center_block_x(),
        }
    }
    fn center_block_z(&self) -> i32 {
        match self {
            Self::Overworld(c) => c.center_block_z(),
            Self::Nether(c) => c.center_block_z(),
        }
    }
    fn sea_level(&self) -> i32 {
        match self {
            Self::Overworld(c) => c.sea_level(),
            Self::Nether(c) => c.sea_level(),
        }
    }
    fn min_y(&self) -> i32 {
        match self {
            Self::Overworld(c) => c.min_y(),
            Self::Nether(c) => c.min_y(),
        }
    }
    fn height(&self) -> i32 {
        match self {
            Self::Overworld(c) => c.height(),
            Self::Nether(c) => c.height(),
        }
    }
    fn template_pools(&self) -> &FxHashMap<Identifier, TemplatePoolData> {
        match self {
            Self::Overworld(c) => c.template_pools(),
            Self::Nether(c) => c.template_pools(),
        }
    }
    fn templates(&self) -> &FxHashMap<Identifier, TemplateData> {
        match self {
            Self::Overworld(c) => c.templates(),
            Self::Nether(c) => c.templates(),
        }
    }
    fn base_height(&mut self, x: i32, z: i32, ocean_floor: bool) -> i32 {
        match self {
            Self::Overworld(c) => c.base_height(x, z, ocean_floor),
            Self::Nether(c) => c.base_height(x, z, ocean_floor),
        }
    }
    fn base_height_full(&mut self, x: i32, z: i32, ocean_floor: bool) -> i32 {
        match self {
            Self::Overworld(c) => c.base_height_full(x, z, ocean_floor),
            Self::Nether(c) => c.base_height_full(x, z, ocean_floor),
        }
    }
    fn biome_at(&mut self, block_x: i32, block_y: i32, block_z: i32) -> BiomeRef {
        match self {
            Self::Overworld(c) => c.biome_at(block_x, block_y, block_z),
            Self::Nether(c) => c.biome_at(block_x, block_y, block_z),
        }
    }
    fn column_state(&mut self, x: i32, y: i32, z: i32) -> ColumnBlock {
        match self {
            Self::Overworld(c) => c.column_state(x, y, z),
            Self::Nether(c) => c.column_state(x, y, z),
        }
    }
    fn surface_y(&mut self) -> i32 {
        match self {
            Self::Overworld(c) => c.surface_y(),
            Self::Nether(c) => c.surface_y(),
        }
    }
    fn terrain_surface_height(&self, x: i32, z: i32, ocean_floor: bool) -> i32 {
        match self {
            Self::Overworld(c) => c.terrain_surface_height(x, z, ocean_floor),
            Self::Nether(c) => c.terrain_surface_height(x, z, ocean_floor),
        }
    }
    fn terrain_is_opaque(&self, x: i32, y: i32, z: i32, ocean_floor: bool) -> bool {
        match self {
            Self::Overworld(c) => c.terrain_is_opaque(x, y, z, ocean_floor),
            Self::Nether(c) => c.terrain_is_opaque(x, y, z, ocean_floor),
        }
    }
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
        init_vanilla_registry();
        let dimension = structure.dimension;
        let is_nether = dimension == "minecraft:the_nether";
        let is_end = dimension == "minecraft:the_end";

        let (overworld_noises, nether_noises, biome_source) = if is_nether {
            (
                None,
                Some(NetherNoises::new(world_seed as u64)),
                BiomeSourceKind::nether(world_seed as u64),
            )
        } else if is_end {
            (None, None, BiomeSourceKind::end(world_seed as u64))
        } else {
            (
                Some(OverworldNoises::new(world_seed as u64)),
                None,
                BiomeSourceKind::overworld(world_seed as u64),
            )
        };

        let splitter = LegacyRandom::from_seed(world_seed as u64).next_positional();

        Self {
            world_seed,
            structure,
            kind,
            dimension,
            overworld_noises,
            nether_noises,
            biome_source,
            splitter,
        }
    }

    pub fn scan_many(
        &self,
        chunks: impl IntoIterator<Item = (i32, i32)>,
    ) -> Result<Vec<Scan>, Error> {
        chunks
            .into_iter()
            .map(|(chunk_x, chunk_z)| self.scan_chunk(chunk_x, chunk_z))
            .collect()
    }

    fn scan_chunk(&self, chunk_x: i32, chunk_z: i32) -> Result<Scan, Error> {
        match self.kind {
            ScanKind::AncientCity | ScanKind::BastionRemnant => {
                self.scan_jigsaw_structure(chunk_x, chunk_z)
            }
            ScanKind::DesertPyramid => self.scan_desert_pyramid(chunk_x, chunk_z),
            ScanKind::Igloo => self.scan_igloo(chunk_x, chunk_z),
            ScanKind::Village => self.scan_village(chunk_x, chunk_z),
            ScanKind::PillagerOutpost => self.scan_pillager_outpost(chunk_x, chunk_z),
            ScanKind::BuriedTreasure => self.scan_buried_treasure(chunk_x, chunk_z),
            ScanKind::Shipwreck => self.scan_shipwreck(chunk_x, chunk_z),
        }
    }

    pub(crate) fn generation_context(&self, chunk_x: i32, chunk_z: i32) -> ScannerContext<'_> {
        let chunk_min_x = chunk_x * 16;
        let chunk_min_z = chunk_z * 16;
        let mut biome_sampler = self.biome_source.chunk_sampler();
        biome_sampler.init_grid(chunk_min_x, chunk_min_z);

        if let Some(noises) = &self.overworld_noises {
            let mut height_cache = OverworldColumnCache::new();
            height_cache.init_grid(chunk_min_x, chunk_min_z, noises);
            let aquifer = LazyAquifer::<OverworldNoises>::new(
                chunk_min_x,
                chunk_min_z,
                &self.splitter,
                noises,
            );
            ScannerContext::Overworld(OverworldScannerContext {
                seed: self.world_seed,
                chunk_x,
                chunk_z,
                sea_level: 63,
                noises,
                splitter: &self.splitter,
                biome_sampler,
                height_cache,
                aquifer,
                surface_y_cache: None,
                height_cache_grid_ready: true,
            })
        } else if let Some(noises) = &self.nether_noises {
            let mut height_cache = NetherColumnCache::new();
            height_cache.init_grid(chunk_min_x, chunk_min_z, noises);
            let aquifer =
                LazyAquifer::<NetherNoises>::new(chunk_min_x, chunk_min_z, &self.splitter, noises);
            ScannerContext::Nether(NetherScannerContext {
                seed: self.world_seed,
                chunk_x,
                chunk_z,
                sea_level: 32,
                noises,
                splitter: &self.splitter,
                biome_sampler,
                height_cache,
                aquifer,
                surface_y_cache: None,
                height_cache_grid_ready: true,
            })
        } else {
            panic!("unsupported dimension for scanner context");
        }
    }

    pub(crate) fn chunk_random(&self, chunk_x: i32, chunk_z: i32) -> LegacyRandom {
        let mut random = LegacyRandom::from_seed(self.world_seed as u64);
        let x_mult = random.next_i64() | 1;
        let z_mult = random.next_i64() | 1;
        let seed = (i64::from(chunk_x).wrapping_mul(x_mult)
            ^ i64::from(chunk_z).wrapping_mul(z_mult)
            ^ self.world_seed) as u64;
        LegacyRandom::from_seed(seed)
    }

    pub(crate) fn feature_random(&self, chunk_x: i32, chunk_z: i32) -> LegacyRandom {
        let mut random = LegacyRandom::from_seed(self.world_seed as u64);
        let x_mult = random.next_i64();
        let z_mult = random.next_i64();
        let seed = (i64::from(chunk_x).wrapping_mul(x_mult)
            ^ i64::from(chunk_z).wrapping_mul(z_mult)
            ^ self.world_seed) as u64;
        LegacyRandom::from_seed(seed)
    }

    pub(crate) fn structure_data(&self) -> Option<&'static StructureData> {
        let id = self.kind.identifier();
        REGISTRY.structures.by_key(&id)
    }

    pub(crate) fn is_valid_biome(&self, biome_id: &Identifier) -> bool {
        if let Some(structure_data) = self.structure_data() {
            structure_data.allowed_biomes.contains(biome_id)
        } else {
            false
        }
    }

    pub(crate) fn sea_level(&self) -> i32 {
        if self.dimension == "minecraft:the_nether" {
            32
        } else {
            63
        }
    }

    pub(crate) fn min_y(&self) -> i32 {
        if self.dimension == "minecraft:the_nether" {
            0
        } else {
            -64
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
}

impl ScanKind {
    pub const fn identifier(self) -> Identifier {
        match self {
            Self::AncientCity => Identifier::new_static("minecraft", "ancient_city"),
            Self::BastionRemnant => Identifier::new_static("minecraft", "bastion_remnant"),
            Self::DesertPyramid => Identifier::new_static("minecraft", "desert_pyramid"),
            Self::Igloo => Identifier::new_static("minecraft", "igloo"),
            Self::Village => Identifier::new_static("minecraft", "village_plains"),
            Self::PillagerOutpost => Identifier::new_static("minecraft", "pillager_outpost"),
            Self::BuriedTreasure => Identifier::new_static("minecraft", "buried_treasure"),
            Self::Shipwreck => Identifier::new_static("minecraft", "shipwreck"),
        }
    }
}

const fn invalid_scan() -> Scan {
    Scan {
        valid_structure: false,
        chests: Vec::new(),
    }
}
