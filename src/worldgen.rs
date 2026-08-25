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
use std::cell::RefCell;
use std::sync::{LazyLock, Once};
use steel_registry::biome::BiomeRef;
use steel_registry::structure::StructureData;
use steel_registry::template_pool::{TemplateData, TemplatePoolData};
use steel_registry::vanilla_template_pools::{vanilla_template_pools, vanilla_templates};
use steel_registry::{REGISTRY, Registry, RegistryExt};
use steel_utils::Identifier;
use steel_utils::random::legacy_random::LegacyRandom;
use steel_utils::random::{Random, RandomSplitter};
use steel_worldgen::biomes::{BiomeSourceKind, ChunkBiomeSampler};
use steel_worldgen::density_functions::nether::{NetherColumnCache, NetherNoises};
use steel_worldgen::density_functions::overworld::{OverworldColumnCache, OverworldNoises};
use steel_worldgen::noise::LazyAquifer;
use steel_worldgen::noise_parameters::get_noise_parameters;
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

static VANILLA_REGISTRY_INIT: Once = Once::new();

pub(crate) fn ensure_vanilla_registry() {
    VANILLA_REGISTRY_INIT.call_once(|| {
        let mut registry = Registry::new_vanilla();
        registry.freeze();
        let _ = REGISTRY.init(registry);
    });
}

fn transformed_position(rotation: steel_utils::Rotation, position: IVec3, pivot: IVec3) -> IVec3 {
    let (x, y, z) = rotation.transform_pos(position.x, position.y, position.z, pivot.x, pivot.z);
    IVec3::new(x, y, z)
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
    biome_sampler: RefCell<ChunkBiomeSampler<'src>>,
    height_cache: RefCell<OverworldColumnCache>,
    aquifer: RefCell<LazyAquifer<'src, OverworldNoises>>,
    surface_y_cache: RefCell<Option<i32>>,
    height_cache_grid_ready: RefCell<bool>,
}

impl<'src> OverworldScannerContext<'src> {
    fn with_context<R>(
        &self,
        f: impl FnOnce(&mut GenerationContext<'_, 'src, OverworldNoises>) -> R,
    ) -> R {
        let mut biome_sampler = self.biome_sampler.borrow_mut();
        let mut height_cache = self.height_cache.borrow_mut();
        let mut aquifer = self.aquifer.borrow_mut();
        let mut surface_y_cache = self.surface_y_cache.borrow_mut();
        let mut height_cache_grid_ready = self.height_cache_grid_ready.borrow_mut();
        let mut ctx = GenerationContext::new(
            self.seed,
            self.chunk_x,
            self.chunk_z,
            self.sea_level,
            self.noises,
            self.splitter,
            &TEMPLATE_POOLS,
            &TEMPLATES,
            &mut biome_sampler,
            &mut height_cache,
            &mut aquifer,
            &mut surface_y_cache,
            &mut height_cache_grid_ready,
        );
        f(&mut ctx)
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
        let in_chunk = x >= self.chunk_min_x()
            && x < self.chunk_min_x() + 16
            && z >= self.chunk_min_z()
            && z < self.chunk_min_z() + 16;
        if in_chunk {
            self.with_context(|ctx| ctx.base_height(x, z, ocean_floor))
        } else {
            self.with_context(|ctx| ctx.terrain_surface_height(x, z, ocean_floor))
        }
    }
    fn base_height_full(&mut self, x: i32, z: i32, ocean_floor: bool) -> i32 {
        self.with_context(|ctx| ctx.base_height_full(x, z, ocean_floor))
    }
    fn biome_at(&mut self, block_x: i32, block_y: i32, block_z: i32) -> BiomeRef {
        self.with_context(|ctx| ctx.biome_at(block_x, block_y, block_z))
    }
    fn column_state(&mut self, x: i32, y: i32, z: i32) -> ColumnBlock {
        self.with_context(|ctx| ctx.column_state(x, y, z))
    }
    fn solid_block_below_air(
        &mut self,
        x: i32,
        z: i32,
        start_y: i32,
        min_solid_y: i32,
    ) -> Option<i32> {
        self.with_context(|ctx| ctx.solid_block_below_air(x, z, start_y, min_solid_y))
    }
    fn surface_y(&mut self) -> i32 {
        self.with_context(|ctx| ctx.surface_y())
    }
    fn terrain_surface_height(&self, x: i32, z: i32, ocean_floor: bool) -> i32 {
        self.with_context(|ctx| ctx.terrain_surface_height(x, z, ocean_floor))
    }
    fn terrain_is_opaque(&self, x: i32, y: i32, z: i32, ocean_floor: bool) -> bool {
        self.with_context(|ctx| ctx.terrain_is_opaque(x, y, z, ocean_floor))
    }
}

pub struct NetherScannerContext<'src> {
    seed: i64,
    chunk_x: i32,
    chunk_z: i32,
    sea_level: i32,
    noises: &'src NetherNoises,
    splitter: &'src RandomSplitter,
    biome_sampler: RefCell<ChunkBiomeSampler<'src>>,
    height_cache: RefCell<NetherColumnCache>,
    aquifer: RefCell<LazyAquifer<'src, NetherNoises>>,
    surface_y_cache: RefCell<Option<i32>>,
    height_cache_grid_ready: RefCell<bool>,
}

impl<'src> NetherScannerContext<'src> {
    fn with_context<R>(
        &self,
        f: impl FnOnce(&mut GenerationContext<'_, 'src, NetherNoises>) -> R,
    ) -> R {
        let mut biome_sampler = self.biome_sampler.borrow_mut();
        let mut height_cache = self.height_cache.borrow_mut();
        let mut aquifer = self.aquifer.borrow_mut();
        let mut surface_y_cache = self.surface_y_cache.borrow_mut();
        let mut height_cache_grid_ready = self.height_cache_grid_ready.borrow_mut();
        let mut ctx = GenerationContext::new(
            self.seed,
            self.chunk_x,
            self.chunk_z,
            self.sea_level,
            self.noises,
            self.splitter,
            &TEMPLATE_POOLS,
            &TEMPLATES,
            &mut biome_sampler,
            &mut height_cache,
            &mut aquifer,
            &mut surface_y_cache,
            &mut height_cache_grid_ready,
        );
        f(&mut ctx)
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
        self.with_context(|ctx| ctx.base_height(x, z, ocean_floor))
    }
    fn base_height_full(&mut self, x: i32, z: i32, ocean_floor: bool) -> i32 {
        self.with_context(|ctx| ctx.base_height_full(x, z, ocean_floor))
    }
    fn biome_at(&mut self, block_x: i32, block_y: i32, block_z: i32) -> BiomeRef {
        self.with_context(|ctx| ctx.biome_at(block_x, block_y, block_z))
    }
    fn column_state(&mut self, x: i32, y: i32, z: i32) -> ColumnBlock {
        self.with_context(|ctx| ctx.column_state(x, y, z))
    }
    fn solid_block_below_air(
        &mut self,
        x: i32,
        z: i32,
        start_y: i32,
        min_solid_y: i32,
    ) -> Option<i32> {
        self.with_context(|ctx| ctx.solid_block_below_air(x, z, start_y, min_solid_y))
    }
    fn surface_y(&mut self) -> i32 {
        self.with_context(|ctx| ctx.surface_y())
    }
    fn terrain_surface_height(&self, x: i32, z: i32, ocean_floor: bool) -> i32 {
        self.with_context(|ctx| ctx.terrain_surface_height(x, z, ocean_floor))
    }
    fn terrain_is_opaque(&self, x: i32, y: i32, z: i32, ocean_floor: bool) -> bool {
        self.with_context(|ctx| ctx.terrain_is_opaque(x, y, z, ocean_floor))
    }
}

pub enum ScannerContext<'src> {
    Overworld(Box<OverworldScannerContext<'src>>),
    Nether(Box<NetherScannerContext<'src>>),
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
            Self::Nether(c) => c.chunk_x(),
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
    fn solid_block_below_air(
        &mut self,
        x: i32,
        z: i32,
        start_y: i32,
        min_solid_y: i32,
    ) -> Option<i32> {
        match self {
            Self::Overworld(c) => c.solid_block_below_air(x, z, start_y, min_solid_y),
            Self::Nether(c) => c.solid_block_below_air(x, z, start_y, min_solid_y),
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
        ensure_vanilla_registry();
        let dimension = structure.dimension;
        let is_nether = dimension == "minecraft:the_nether";
        let is_end = dimension == "minecraft:the_end";

        let splitter = LegacyRandom::from_seed(world_seed as u64).next_positional();
        let params = get_noise_parameters();

        let (overworld_noises, nether_noises, biome_source) = if is_nether {
            (
                None,
                Some(NetherNoises::create(world_seed as u64, &splitter, &params)),
                BiomeSourceKind::nether(world_seed as u64),
            )
        } else if is_end {
            (None, None, BiomeSourceKind::end(world_seed as u64))
        } else {
            (
                Some(OverworldNoises::create(
                    world_seed as u64,
                    &splitter,
                    &params,
                )),
                None,
                BiomeSourceKind::overworld(world_seed as u64),
            )
        };

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
        let biome_sampler = self.biome_source.chunk_sampler();

        if let Some(noises) = &self.overworld_noises {
            let mut height_cache = OverworldColumnCache::new();
            height_cache.init_grid(chunk_min_x, chunk_min_z, noises);
            let aquifer = LazyAquifer::<OverworldNoises>::new(
                chunk_min_x,
                chunk_min_z,
                &self.splitter,
                noises,
            );
            ScannerContext::Overworld(Box::new(OverworldScannerContext {
                seed: self.world_seed,
                chunk_x,
                chunk_z,
                sea_level: 63,
                noises,
                splitter: &self.splitter,
                biome_sampler: RefCell::new(biome_sampler),
                height_cache: RefCell::new(height_cache),
                aquifer: RefCell::new(aquifer),
                surface_y_cache: RefCell::new(None),
                height_cache_grid_ready: RefCell::new(true),
            }))
        } else if let Some(noises) = &self.nether_noises {
            let mut height_cache = NetherColumnCache::new();
            height_cache.init_grid(chunk_min_x, chunk_min_z, noises);
            let aquifer =
                LazyAquifer::<NetherNoises>::new(chunk_min_x, chunk_min_z, &self.splitter, noises);
            ScannerContext::Nether(Box::new(NetherScannerContext {
                seed: self.world_seed,
                chunk_x,
                chunk_z,
                sea_level: 32,
                noises,
                splitter: &self.splitter,
                biome_sampler: RefCell::new(biome_sampler),
                height_cache: RefCell::new(height_cache),
                aquifer: RefCell::new(aquifer),
                surface_y_cache: RefCell::new(None),
                height_cache_grid_ready: RefCell::new(true),
            }))
        } else {
            panic!("unsupported dimension for scanner context");
        }
    }

    pub(crate) fn chunk_random(&self, chunk_x: i32, chunk_z: i32) -> LegacyRandom {
        let mut random = LegacyRandom::from_seed(self.world_seed as u64);
        let x_mult = random.next_i64();
        let z_mult = random.next_i64();
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
