use std::cell::Cell;

use rustc_hash::FxHashMap;
use steel_registry::biome::TemperatureModifier;
use steel_registry::blocks::block_state_ext::BlockStateExt;
use steel_registry::vanilla_blocks;
use steel_registry::{REGISTRY, RegistryEntry, RegistryExt};
use steel_utils::BlockStateId;
use steel_utils::random::legacy_random::LegacyRandom;
use steel_utils::random::name_hash::NameHash;
use steel_utils::random::xoroshiro::Xoroshiro;
use steel_utils::random::{PositionalRandom, Random, RandomSource, RandomSplitter};
use steel_worldgen::biomes::{BiomeSourceKind, ChunkBiomeSampler, obfuscate_biome_seed};
use steel_worldgen::density::DimensionNoises;
use steel_worldgen::density_functions::overworld::{OverworldColumnCache, OverworldNoises};
use steel_worldgen::noise::{
    LazyAquifer, NormalNoise, PerlinSimplexNoise, preliminary_surface_level,
};
use steel_worldgen::structure::{ColumnBlock, GenerationContext};
use steel_worldgen::surface::{
    SurfaceBiomeProvider, SurfaceConditionNoiseCache, SurfaceNoiseProvider, SurfaceRuleContext,
};

use super::{TEMPLATE_POOLS, TEMPLATES};

const MIN_Y: i32 = -64;
const HEIGHT: i32 = 384;
const SEA_LEVEL: i32 = 63;
const CLAY_BAND_LENGTH: usize = 192;

pub(super) struct SurfaceTerrainSampler {
    world_seed: i64,
    noises: OverworldNoises,
    splitter: RandomSplitter,
    biome_source: BiomeSourceKind,
    surface_rules: SurfaceRules,
    columns: FxHashMap<(i32, i32), Column>,
}

struct Column {
    states: Box<[BlockStateId]>,
    world_surface_height: i32,
    ocean_floor_height: i32,
    motion_blocking_no_leaves_height: i32,
}

impl SurfaceTerrainSampler {
    pub(super) fn new(world_seed: i64) -> Self {
        let seed = world_seed as u64;
        Self::with_splitter(world_seed, Xoroshiro::from_seed(seed).next_positional())
    }

    pub(super) fn new_legacy(world_seed: i64) -> Self {
        let seed = world_seed as u64;
        Self::with_splitter(world_seed, LegacyRandom::from_seed(seed).next_positional())
    }

    fn with_splitter(world_seed: i64, splitter: RandomSplitter) -> Self {
        let seed = world_seed as u64;
        let params = steel_worldgen::noise_parameters::get_noise_parameters();
        let noises = OverworldNoises::create(seed, &splitter, &params);
        let surface_rules = SurfaceRules::new(&splitter, &params);
        Self {
            world_seed,
            noises,
            splitter,
            biome_source: BiomeSourceKind::overworld(seed),
            surface_rules,
            columns: FxHashMap::default(),
        }
    }

    pub(super) fn height(&mut self, x: i32, z: i32, ocean_floor: bool) -> i32 {
        let column = self.column(x, z);
        if ocean_floor {
            column.ocean_floor_height
        } else {
            column.world_surface_height
        }
    }
    pub(super) fn motion_blocking_no_leaves_height(&mut self, x: i32, z: i32) -> i32 {
        self.column(x, z).motion_blocking_no_leaves_height
    }
    #[cfg(test)]
    pub(super) fn debug_block_counts(&mut self, x: i32, z: i32) -> FxHashMap<String, usize> {
        let mut counts = FxHashMap::default();
        for state in self.column(x, z).states.iter() {
            let name = state.get_block().key.to_string();
            *counts.entry(name).or_insert(0) += 1;
        }
        counts
    }

    pub(super) fn is_buried_treasure_support(&mut self, x: i32, y: i32, z: i32) -> bool {
        if !(MIN_Y..MIN_Y + HEIGHT).contains(&y) {
            return false;
        }
        let state = self.column(x, z).states[(y - MIN_Y) as usize];
        let block = state.get_block();
        block == &vanilla_blocks::SANDSTONE
            || block == &vanilla_blocks::STONE
            || block == &vanilla_blocks::ANDESITE
            || block == &vanilla_blocks::GRANITE
            || block == &vanilla_blocks::DIORITE
    }
    #[cfg(test)]
    pub(super) fn debug_surface_metrics(&self, x: i32, z: i32) -> (i32, i32, i32, i32, i32, u16) {
        let chunk_x = x.div_euclid(16);
        let chunk_z = z.div_euclid(16);
        let chunk_min_x = chunk_x * 16;
        let chunk_min_z = chunk_z * 16;
        let mut cache = OverworldColumnCache::new();
        let p00 = preliminary_surface_level(&self.noises, &mut cache, chunk_min_x, chunk_min_z);
        let p10 =
            preliminary_surface_level(&self.noises, &mut cache, chunk_min_x + 16, chunk_min_z);
        let p01 =
            preliminary_surface_level(&self.noises, &mut cache, chunk_min_x, chunk_min_z + 16);
        let p11 =
            preliminary_surface_level(&self.noises, &mut cache, chunk_min_x + 16, chunk_min_z + 16);
        let tx = f64::from(x - chunk_min_x) / 16.0;
        let tz = f64::from(z - chunk_min_z) / 16.0;
        let interpolated = (1.0 - tx) * (1.0 - tz) * f64::from(p00)
            + tx * (1.0 - tz) * f64::from(p10)
            + (1.0 - tx) * tz * f64::from(p01)
            + tx * tz * f64::from(p11);
        let depth = self.surface_rules.surface_depth(x, z);
        let min_surface = interpolated.floor() as i32 + depth - 8;
        let mut sampler = self.biome_source.chunk_sampler();
        let biome_id = sampler.sample(x >> 2, 140 >> 2, z >> 2).id() as u16;
        (p00, p10, p01, p11, min_surface, biome_id)
    }

    fn column(&mut self, x: i32, z: i32) -> &Column {
        let key = (x, z);
        if !self.columns.contains_key(&key) {
            let column = self.generate_column(x, z);
            self.columns.insert(key, column);
        }
        self.columns
            .get(&key)
            .expect("generated surface column must be cached")
    }

    fn generate_column(&self, x: i32, z: i32) -> Column {
        let chunk_x = x.div_euclid(16);
        let chunk_z = z.div_euclid(16);
        let mut biome_sampler = self.biome_source.chunk_sampler();
        let mut height_cache = OverworldColumnCache::new();
        let mut aquifer = LazyAquifer::<OverworldNoises>::new(
            chunk_x * 16,
            chunk_z * 16,
            &self.splitter,
            &self.noises,
        );
        let mut surface_y_cache = None;
        let mut height_cache_grid_ready = false;
        let mut ctx = GenerationContext::new(
            self.world_seed,
            chunk_x,
            chunk_z,
            SEA_LEVEL,
            &self.noises,
            &self.splitter,
            &TEMPLATE_POOLS,
            &TEMPLATES,
            &mut biome_sampler,
            &mut height_cache,
            &mut aquifer,
            &mut surface_y_cache,
            &mut height_cache_grid_ready,
        );

        let air = vanilla_blocks::AIR.default_state();
        let default_block = <OverworldNoises as DimensionNoises>::Settings::default_block_id();
        let default_fluid = <OverworldNoises as DimensionNoises>::Settings::default_fluid_id();
        let mut base_states = vec![air; HEIGHT as usize];
        for y in MIN_Y..MIN_Y + HEIGHT {
            base_states[(y - MIN_Y) as usize] = match ctx.column_state(x, y, z) {
                ColumnBlock::Air => air,
                ColumnBlock::Solid => default_block,
                ColumnBlock::Fluid => default_fluid,
            };
        }

        let world_surface_height = first_non_air_height(&base_states);
        let ocean_floor_height = first_solid_height(&base_states);
        let mut states = base_states.clone();
        self.apply_surface_rules(
            x,
            z,
            chunk_x,
            chunk_z,
            &base_states,
            &mut states,
            default_block,
        );
        let motion_blocking_no_leaves_height = first_motion_blocking_no_leaves_height(&states);

        Column {
            states: states.into_boxed_slice(),
            world_surface_height,
            ocean_floor_height,
            motion_blocking_no_leaves_height,
        }
    }

    #[expect(
        clippy::too_many_arguments,
        reason = "surface rule context mirrors vanilla's per-column inputs"
    )]
    fn apply_surface_rules(
        &self,
        x: i32,
        z: i32,
        chunk_x: i32,
        chunk_z: i32,
        base_states: &[BlockStateId],
        states: &mut [BlockStateId],
        default_block: BlockStateId,
    ) {
        let uses_biome = <OverworldNoises as DimensionNoises>::surface_rule_uses_biome();
        let uses_preliminary =
            <OverworldNoises as DimensionNoises>::surface_rule_uses_preliminary_surface();
        let uses_secondary =
            <OverworldNoises as DimensionNoises>::surface_rule_uses_surface_secondary();
        let uses_steep = <OverworldNoises as DimensionNoises>::surface_rule_uses_steep();
        let surface_depth = self.surface_rules.surface_depth(x, z);
        let surface_secondary = if uses_secondary {
            self.surface_rules.surface_secondary(x, z)
        } else {
            0.0
        };
        let min_surface_level = if uses_preliminary {
            let mut preliminary_cache = OverworldColumnCache::new();
            let chunk_min_x = chunk_x * 16;
            let chunk_min_z = chunk_z * 16;
            let p00 = preliminary_surface_level(
                &self.noises,
                &mut preliminary_cache,
                chunk_min_x,
                chunk_min_z,
            );
            let p10 = preliminary_surface_level(
                &self.noises,
                &mut preliminary_cache,
                chunk_min_x + 16,
                chunk_min_z,
            );
            let p01 = preliminary_surface_level(
                &self.noises,
                &mut preliminary_cache,
                chunk_min_x,
                chunk_min_z + 16,
            );
            let p11 = preliminary_surface_level(
                &self.noises,
                &mut preliminary_cache,
                chunk_min_x + 16,
                chunk_min_z + 16,
            );
            let local_x = x - chunk_min_x;
            let local_z = z - chunk_min_z;
            let tx = f64::from(local_x) / 16.0;
            let tz = f64::from(local_z) / 16.0;
            let interpolated = (1.0 - tx) * (1.0 - tz) * f64::from(p00)
                + tx * (1.0 - tz) * f64::from(p10)
                + (1.0 - tx) * tz * f64::from(p01)
                + tx * tz * f64::from(p11);
            interpolated.floor() as i32 + surface_depth - 8
        } else {
            0
        };
        let steep = uses_steep && self.is_steep(x, z);
        let condition_count = <OverworldNoises as DimensionNoises>::surface_noise_ids().len();
        let condition_values = (0..condition_count)
            .map(|_| Cell::new(0.0))
            .collect::<Vec<_>>();
        let condition_initialized = (0..condition_count)
            .map(|_| Cell::new(false))
            .collect::<Vec<_>>();
        let condition_cache =
            SurfaceConditionNoiseCache::new(&condition_values, &condition_initialized);
        let mut biome_column = BiomeColumn {
            sampler: self.biome_source.chunk_sampler(),
            zoom_seed: obfuscate_biome_seed(self.world_seed),
            x,
            z,
        };
        let lazy_biome = uses_biome && uses_preliminary;
        let mut stone_depth_above = 0;
        let mut water_height = i32::MIN;
        let mut next_ceiling_stone_y = i32::MAX;
        let start_height = first_non_air_height(base_states);

        for y in (MIN_Y..=start_height).rev() {
            let index = (y - MIN_Y) as usize;
            let state = base_states[index];
            if state.is_air() {
                stone_depth_above = 0;
                water_height = i32::MIN;
                continue;
            }
            if state.get_block().config.liquid {
                if water_height == i32::MIN {
                    water_height = y + 1;
                }
                continue;
            }
            if next_ceiling_stone_y >= y {
                next_ceiling_stone_y = i32::MIN;
                for lookahead_y in (MIN_Y - 1..y).rev() {
                    if lookahead_y < MIN_Y {
                        next_ceiling_stone_y = lookahead_y + 1;
                        break;
                    }
                    let lookahead = base_states[(lookahead_y - MIN_Y) as usize];
                    if lookahead.is_air() || lookahead.get_block().config.liquid {
                        next_ceiling_stone_y = lookahead_y + 1;
                        break;
                    }
                }
            }
            stone_depth_above += 1;
            let stone_depth_below = y - next_ceiling_stone_y + 1;
            if state != default_block {
                continue;
            }

            let eager_biome_id = if uses_biome && !lazy_biome {
                Some(biome_column.biome_id(y))
            } else {
                None
            };
            let biome_provider = if lazy_biome {
                Some(&mut biome_column as &mut dyn SurfaceBiomeProvider)
            } else {
                None
            };
            let mut rule_context = SurfaceRuleContext::new(
                x,
                z,
                surface_depth,
                surface_secondary,
                min_surface_level,
                steep,
                y,
                stone_depth_above,
                stone_depth_below,
                water_height,
                eager_biome_id,
                biome_provider,
                &self.surface_rules,
                &condition_cache,
                <OverworldNoises as DimensionNoises>::surface_rule_block_states(),
            );
            if let Some(new_state) = OverworldNoises::try_apply_surface_rule(&mut rule_context) {
                states[index] = new_state;
            }
        }
    }

    fn is_steep(&self, x: i32, z: i32) -> bool {
        let north = self.base_height(x, z - 1);
        let south = self.base_height(x, z + 1);
        if south >= north + 4 {
            return true;
        }
        let west = self.base_height(x - 1, z);
        let east = self.base_height(x + 1, z);
        west >= east + 4
    }

    fn base_height(&self, x: i32, z: i32) -> i32 {
        let chunk_x = x.div_euclid(16);
        let chunk_z = z.div_euclid(16);
        let mut biome_sampler = self.biome_source.chunk_sampler();
        let mut height_cache = OverworldColumnCache::new();
        let mut aquifer = LazyAquifer::<OverworldNoises>::new(
            chunk_x * 16,
            chunk_z * 16,
            &self.splitter,
            &self.noises,
        );
        let mut surface_y_cache = None;
        let mut height_cache_grid_ready = false;
        let mut ctx = GenerationContext::new(
            self.world_seed,
            chunk_x,
            chunk_z,
            SEA_LEVEL,
            &self.noises,
            &self.splitter,
            &TEMPLATE_POOLS,
            &TEMPLATES,
            &mut biome_sampler,
            &mut height_cache,
            &mut aquifer,
            &mut surface_y_cache,
            &mut height_cache_grid_ready,
        );
        ctx.base_height(x, z, false)
    }
}

fn first_non_air_height(states: &[BlockStateId]) -> i32 {
    states
        .iter()
        .rposition(|state| !state.is_air())
        .map_or(MIN_Y, |index| MIN_Y + index as i32 + 1)
}

fn first_solid_height(states: &[BlockStateId]) -> i32 {
    states
        .iter()
        .rposition(|state| !state.is_air() && !state.get_block().config.liquid)
        .map_or(MIN_Y, |index| MIN_Y + index as i32 + 1)
}

fn first_motion_blocking_no_leaves_height(states: &[BlockStateId]) -> i32 {
    states
        .iter()
        .rposition(|state| {
            let block = state.get_block();
            block != &vanilla_blocks::SNOW
                && block != &vanilla_blocks::POWDER_SNOW
                && (state.blocks_motion() || block.config.liquid)
        })
        .map_or(MIN_Y, |index| MIN_Y + index as i32 + 1)
}

fn lcg_next(mut value: i64, c: i64) -> i64 {
    value = value.wrapping_mul(
        value
            .wrapping_mul(6_364_136_223_846_793_005)
            .wrapping_add(1_442_695_040_888_963_407),
    );
    value.wrapping_add(c)
}

fn get_fiddle(value: i64) -> f64 {
    let uniform = value.wrapping_shr(24).rem_euclid(1024) as f64 / 1024.0;
    (uniform - 0.5) * 0.9
}

fn fuzzed_biome_at_block(
    zoom_seed: i64,
    block_x: i32,
    block_y: i32,
    block_z: i32,
    sampler: &mut ChunkBiomeSampler<'_>,
) -> u16 {
    let abs_x = block_x - 2;
    let abs_y = block_y - 2;
    let abs_z = block_z - 2;
    let parent_x = abs_x >> 2;
    let parent_y = abs_y >> 2;
    let parent_z = abs_z >> 2;
    let fract_x = f64::from(abs_x & 3) / 4.0;
    let fract_y = f64::from(abs_y & 3) / 4.0;
    let fract_z = f64::from(abs_z & 3) / 4.0;
    let mut min_index = 0usize;
    let mut min_distance = f64::INFINITY;

    for i in 0..8usize {
        let x_even = (i & 4) == 0;
        let y_even = (i & 2) == 0;
        let z_even = (i & 1) == 0;
        let cx = if x_even { parent_x } else { parent_x + 1 };
        let cy = if y_even { parent_y } else { parent_y + 1 };
        let cz = if z_even { parent_z } else { parent_z + 1 };
        let dx = if x_even { fract_x } else { fract_x - 1.0 };
        let dy = if y_even { fract_y } else { fract_y - 1.0 };
        let dz = if z_even { fract_z } else { fract_z - 1.0 };
        let mut random = lcg_next(zoom_seed, i64::from(cx));
        random = lcg_next(random, i64::from(cy));
        random = lcg_next(random, i64::from(cz));
        random = lcg_next(random, i64::from(cx));
        random = lcg_next(random, i64::from(cy));
        random = lcg_next(random, i64::from(cz));
        let fx = get_fiddle(random);
        random = lcg_next(random, zoom_seed);
        let fy = get_fiddle(random);
        random = lcg_next(random, zoom_seed);
        let fz = get_fiddle(random);
        let distance = (dx + fx).powi(2) + (dy + fy).powi(2) + (dz + fz).powi(2);
        if min_distance > distance {
            min_index = i;
            min_distance = distance;
        }
    }

    let biome_x = if (min_index & 4) == 0 {
        parent_x
    } else {
        parent_x + 1
    };
    let biome_y = if (min_index & 2) == 0 {
        parent_y
    } else {
        parent_y + 1
    };
    let biome_z = if (min_index & 1) == 0 {
        parent_z
    } else {
        parent_z + 1
    };
    sampler.sample(biome_x, biome_y, biome_z).id() as u16
}

struct BiomeColumn<'a> {
    sampler: ChunkBiomeSampler<'a>,
    zoom_seed: i64,
    x: i32,
    z: i32,
}

impl SurfaceBiomeProvider for BiomeColumn<'_> {
    fn biome_id(&mut self, y: i32) -> u16 {
        fuzzed_biome_at_block(self.zoom_seed, self.x, y, self.z, &mut self.sampler)
    }
}

struct SurfaceRules {
    surface_noise: NormalNoise,
    surface_secondary_noise: NormalNoise,
    clay_bands_offset_noise: NormalNoise,
    clay_bands: [BlockStateId; CLAY_BAND_LENGTH],
    condition_noises: Vec<NormalNoise>,
    vertical_gradient_randoms: Vec<RandomSplitter>,
    noise_random: RandomSplitter,
    temperature_noise: PerlinSimplexNoise,
    frozen_temperature_noise: PerlinSimplexNoise,
    biome_info_noise: PerlinSimplexNoise,
    sea_level: i32,
}

impl SurfaceRules {
    fn new(
        splitter: &RandomSplitter,
        params: &FxHashMap<String, steel_worldgen::density::NoiseParameters>,
    ) -> Self {
        let mut clay_random = splitter.with_hash_of(&NameHash::new("minecraft:clay_bands"));
        let clay_bands = generate_clay_bands(&mut clay_random);
        let condition_noises = <OverworldNoises as DimensionNoises>::surface_noise_ids()
            .iter()
            .map(|&id| create_noise(splitter, id, params))
            .collect();
        let vertical_gradient_randoms =
            <OverworldNoises as DimensionNoises>::surface_gradient_ids()
                .iter()
                .map(|&id| splitter.with_hash_of(&NameHash::new(id)).next_positional())
                .collect();
        let temperature_noise = simplex_noise(1234, &[0]);
        let frozen_temperature_noise = simplex_noise(3456, &[-2, -1, 0]);
        let biome_info_noise = simplex_noise(2345, &[0]);
        Self {
            surface_noise: create_noise(splitter, "minecraft:surface", params),
            surface_secondary_noise: create_noise(splitter, "minecraft:surface_secondary", params),
            clay_bands_offset_noise: create_noise(splitter, "minecraft:clay_bands_offset", params),
            clay_bands,
            condition_noises,
            vertical_gradient_randoms,
            noise_random: splitter.clone(),
            temperature_noise,
            frozen_temperature_noise,
            biome_info_noise,
            sea_level: SEA_LEVEL,
        }
    }

    fn surface_depth(&self, x: i32, z: i32) -> i32 {
        let noise = self
            .surface_noise
            .get_value(f64::from(x), 0.0, f64::from(z));
        let jitter = self.noise_random.at(x, 0, z).next_f64() * 0.25;
        (noise * 2.75 + 3.0 + jitter) as i32
    }

    fn surface_secondary(&self, x: i32, z: i32) -> f64 {
        self.surface_secondary_noise
            .get_value(f64::from(x), 0.0, f64::from(z))
    }
}

impl SurfaceNoiseProvider for SurfaceRules {
    fn condition_noise(&self, noise_index: usize, x: i32, z: i32) -> f64 {
        self.condition_noises[noise_index].get_value(f64::from(x), 0.0, f64::from(z))
    }

    fn get_band(&self, x: i32, y: i32, z: i32) -> BlockStateId {
        let offset = (self
            .clay_bands_offset_noise
            .get_value(f64::from(x), 0.0, f64::from(z))
            * 4.0
            + 0.5)
            .floor() as i32;
        let index = ((y + offset) % CLAY_BAND_LENGTH as i32 + CLAY_BAND_LENGTH as i32) as usize
            % CLAY_BAND_LENGTH;
        self.clay_bands[index]
    }

    fn cold_enough_to_snow(&self, biome_id: u16, block_x: i32, block_y: i32, block_z: i32) -> bool {
        let biome = REGISTRY
            .biomes
            .by_id(biome_id as usize)
            .expect("invalid biome id");
        let base_temperature = biome.temperature;
        let temperature = match biome.temperature_modifier {
            TemperatureModifier::None => base_temperature,
            TemperatureModifier::Frozen => {
                let large = self
                    .frozen_temperature_noise
                    .get_value(f64::from(block_x) * 0.05, f64::from(block_z) * 0.05)
                    * 7.0;
                let edge = self
                    .biome_info_noise
                    .get_value(f64::from(block_x) * 0.2, f64::from(block_z) * 0.2);
                let combined = large + edge;
                if combined < 0.3 {
                    let small = self
                        .biome_info_noise
                        .get_value(f64::from(block_x) * 0.09, f64::from(block_z) * 0.09);
                    if small < 0.8 { 0.2 } else { base_temperature }
                } else {
                    base_temperature
                }
            }
        };
        if block_y > self.sea_level + 17 {
            let value = self
                .temperature_noise
                .get_value(f64::from(block_x) / 8.0, f64::from(block_z) / 8.0)
                as f32
                * 8.0;
            temperature - (value + block_y as f32 - (self.sea_level + 17) as f32) * 0.05 / 40.0
                < 0.15
        } else {
            temperature < 0.15
        }
    }

    fn vertical_gradient(
        &self,
        gradient_index: usize,
        block_x: i32,
        block_y: i32,
        block_z: i32,
        true_at_and_below: i32,
        false_at_and_above: i32,
    ) -> bool {
        if block_y <= true_at_and_below {
            return true;
        }
        if block_y >= false_at_and_above {
            return false;
        }
        let probability = f64::from(false_at_and_above - block_y)
            / f64::from(false_at_and_above - true_at_and_below);
        let random = self.vertical_gradient_randoms[gradient_index]
            .at(block_x, block_y, block_z)
            .next_f32();
        f64::from(random) < probability
    }
}

fn create_noise(
    splitter: &RandomSplitter,
    id: &str,
    params: &FxHashMap<String, steel_worldgen::density::NoiseParameters>,
) -> NormalNoise {
    let parameters = params
        .get(id)
        .unwrap_or_else(|| panic!("missing noise parameters for {id}"));
    NormalNoise::create(
        splitter,
        id,
        parameters.first_octave,
        &parameters.amplitudes,
    )
}

fn simplex_noise(seed: u64, octaves: &[i32]) -> PerlinSimplexNoise {
    let mut random = RandomSource::Legacy(LegacyRandom::from_seed(seed));
    PerlinSimplexNoise::new(&mut random, octaves)
}

fn generate_clay_bands(random: &mut RandomSource) -> [BlockStateId; CLAY_BAND_LENGTH] {
    let terracotta = vanilla_blocks::TERRACOTTA.default_state();
    let orange = vanilla_blocks::ORANGE_TERRACOTTA.default_state();
    let yellow = vanilla_blocks::YELLOW_TERRACOTTA.default_state();
    let brown = vanilla_blocks::BROWN_TERRACOTTA.default_state();
    let red = vanilla_blocks::RED_TERRACOTTA.default_state();
    let white = vanilla_blocks::WHITE_TERRACOTTA.default_state();
    let light_gray = vanilla_blocks::LIGHT_GRAY_TERRACOTTA.default_state();
    let mut bands = [terracotta; CLAY_BAND_LENGTH];

    let mut i = 0usize;
    while i < CLAY_BAND_LENGTH {
        i += random.next_i32_bounded(5) as usize + 1;
        if i < CLAY_BAND_LENGTH {
            bands[i] = orange;
        }
        i += 1;
    }
    make_clay_bands(random, &mut bands, 1, yellow);
    make_clay_bands(random, &mut bands, 2, brown);
    make_clay_bands(random, &mut bands, 1, red);

    let white_count = random.next_i32_between(9, 15);
    let mut placed = 0;
    let mut start = 0usize;
    while placed < white_count && start < CLAY_BAND_LENGTH {
        bands[start] = white;
        if start > 1 && random.next_bool() {
            bands[start - 1] = light_gray;
        }
        if start + 1 < CLAY_BAND_LENGTH && random.next_bool() {
            bands[start + 1] = light_gray;
        }
        placed += 1;
        start += random.next_i32_bounded(16) as usize + 4;
    }
    bands
}

fn make_clay_bands(
    random: &mut RandomSource,
    bands: &mut [BlockStateId; CLAY_BAND_LENGTH],
    base_width: i32,
    state: BlockStateId,
) {
    let count = random.next_i32_between(6, 15);
    for _ in 0..count {
        let width = (base_width + random.next_i32_bounded(3)) as usize;
        let start = random.next_i32_bounded(CLAY_BAND_LENGTH as i32) as usize;
        for offset in 0..width {
            if start + offset >= CLAY_BAND_LENGTH {
                break;
            }
            bands[start + offset] = state;
        }
    }
}
