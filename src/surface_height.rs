//! Column height sampling for surface-anchored structures.
//!
//! Replicates `pumpkin_world::generation::structure::height_sampler::NoiseHeightSampler`
//! (which is `pub(crate)` inside the Pumpkin fork) so the scanner can resolve
//! vanilla `ChunkGenerator` height queries for structures such as the desert
//! pyramid without building any chunks.
//!
//! The fork's sampler output (top non-air Y + 1) equals vanilla
//! `getBaseHeight`; vanilla `getFirstOccupiedHeight` is `getBaseHeight - 1`.
//! Both accessors are exposed so callers can mirror the exact vanilla query
//! (desert pyramid: `getFirstOccupiedHeight` for the corner sea-level check and
//! the biome position, the heightmap value for the piece base height).

use std::collections::HashMap;

use pumpkin_data::Block;
use pumpkin_util::math::floor_div;
use pumpkin_world::generation::{
    biome_coords,
    generator::VanillaGenerator,
    noise::{
        ChunkNoiseGenerator,
        aquifer_sampler::FluidLevel,
        router::surface_height_sampler::{
            SurfaceHeightEstimateSampler, SurfaceHeightSamplerBuilderOptions,
        },
    },
    proto_chunk::StandardChunkFluidLevelSampler,
};

pub struct ColumnHeightSampler<'a> {
    generator: &'a VanillaGenerator,
    preliminary: SurfaceHeightEstimateSampler<'a>,
    heights: HashMap<(i32, i32), i32>,
}

impl<'a> ColumnHeightSampler<'a> {
    pub fn new(generator: &'a VanillaGenerator, start_x: i32, start_z: i32) -> Self {
        let shape = &generator.settings.shape;
        let horizontal_biome_end = biome_coords::from_block(16) as usize;
        let preliminary = SurfaceHeightEstimateSampler::generate(
            &generator.base_router.surface_estimator,
            &SurfaceHeightSamplerBuilderOptions::new(
                biome_coords::from_block(start_x),
                biome_coords::from_block(start_z),
                horizontal_biome_end,
                i32::from(shape.min_y),
                i32::from(shape.max_y()),
                shape.vertical_cell_block_count() as usize,
            ),
        );
        Self {
            generator,
            preliminary,
            heights: HashMap::new(),
        }
    }

    /// Vanilla `ChunkGenerator.getBaseHeight`: the height stored in heightmaps
    /// (one above the top block matching the heightmap predicate).
    ///
    /// The fork's exclusive sampler output (top non-air Y + 1) equals this
    /// value exactly; locked by the 26.1.2 desert pyramid chest vectors.
    pub fn base_height(&mut self, x: i32, z: i32) -> i32 {
        self.estimate_height(x, z)
    }

    /// Vanilla `ChunkGenerator.getFirstOccupiedHeight`: `getBaseHeight - 1`,
    /// i.e. the Y of the top block matching the heightmap predicate.
    pub fn first_occupied_height(&mut self, x: i32, z: i32) -> i32 {
        self.estimate_height(x, z) - 1
    }

    fn estimate_height(&mut self, x: i32, z: i32) -> i32 {
        let key = (x, z);
        if let Some(height) = self.heights.get(&key) {
            return *height;
        }
        let height = self.sample_column(x, z);
        self.heights.insert(key, height);
        height
    }

    fn sample_column(&mut self, x: i32, z: i32) -> i32 {
        let settings = self.generator.settings;
        let shape = &settings.shape;
        let horizontal = i32::from(shape.horizontal_cell_block_count());
        let vertical = i32::from(shape.vertical_cell_block_count());
        let start_x = floor_div(x, horizontal) * horizontal;
        let start_z = floor_div(z, horizontal) * horizontal;
        let local_x = x.rem_euclid(horizontal);
        let local_z = z.rem_euclid(horizontal);
        let fluid_sampler = StandardChunkFluidLevelSampler::new(
            FluidLevel::new(
                settings.sea_level,
                Block::from_state_id(settings.default_fluid.id),
            ),
            FluidLevel::new(-54, &Block::LAVA),
        );
        let mut noise = ChunkNoiseGenerator::new(
            &self.generator.base_router.noise,
            &self.generator.random_config,
            1,
            start_x,
            start_z,
            shape,
            fluid_sampler,
            settings.aquifers_enabled,
            false,
            Vec::new(),
            Vec::new(),
            None,
        );

        noise.sample_start_density();
        noise.sample_end_density(0);
        let minimum_cell_y = floor_div(i32::from(noise.min_y()), vertical);
        let cell_count = i32::from(noise.height()) / vertical;
        for cell_y in (0..cell_count).rev() {
            noise.on_sampled_cell_corners(0, cell_y, 0);
            let sample_start_y = (minimum_cell_y + cell_y) * vertical;
            for local_y in (0..vertical).rev() {
                let y = sample_start_y + local_y;
                noise.interpolate_y(f64::from(local_y) / f64::from(vertical));
                noise.interpolate_x(f64::from(local_x) / f64::from(horizontal));
                noise.interpolate_z(f64::from(local_z) / f64::from(horizontal));
                let state = noise
                    .sample_block_state(
                        &self.generator.random_config.ore_random_deriver,
                        start_x,
                        sample_start_y,
                        start_z,
                        local_x,
                        local_y,
                        local_z,
                        &mut self.preliminary,
                    )
                    .unwrap_or(self.generator.default_block);
                if !state.is_air() {
                    return y + 1;
                }
            }
        }

        i32::from(shape.min_y)
    }
}
