use std::collections::{HashMap, hash_map::Entry};

use pumpkin_data::BlockStateId;
use pumpkin_util::{HeightMap, math::vector3::Vector3};
use pumpkin_world::{
    ProtoChunk,
    generation::generator::{VanillaGenerator, WorldGenerator},
};

pub(super) struct TerrainSampler<'a> {
    world_generator: &'a WorldGenerator,
    generator: &'a VanillaGenerator,
    chunks: HashMap<(i32, i32), ProtoChunk>,
}

impl<'a> TerrainSampler<'a> {
    pub(super) fn new(world_generator: &'a WorldGenerator) -> Self {
        let WorldGenerator::Noise(generator) = world_generator else {
            unreachable!("scanner only constructs noise generators")
        };
        Self {
            world_generator,
            generator,
            chunks: HashMap::with_capacity(4),
        }
    }

    pub(super) fn height(&mut self, heightmap: HeightMap, x: i32, z: i32) -> i32 {
        self.chunk(x.div_euclid(16), z.div_euclid(16))
            .get_top_y(&heightmap, x, z)
    }

    pub(super) fn block_state(&mut self, x: i32, y: i32, z: i32) -> BlockStateId {
        self.chunk(x.div_euclid(16), z.div_euclid(16))
            .get_block_state(&Vector3::new(x, y, z))
    }

    fn chunk(&mut self, chunk_x: i32, chunk_z: i32) -> &ProtoChunk {
        let world_generator = self.world_generator;
        let generator = self.generator;
        match self.chunks.entry((chunk_x, chunk_z)) {
            Entry::Occupied(entry) => entry.into_mut(),
            Entry::Vacant(entry) => entry.insert(Self::generate_chunk(
                world_generator,
                generator,
                chunk_x,
                chunk_z,
            )),
        }
    }

    fn generate_chunk(
        world_generator: &WorldGenerator,
        generator: &VanillaGenerator,
        chunk_x: i32,
        chunk_z: i32,
    ) -> ProtoChunk {
        let mut chunk = ProtoChunk::new(chunk_x, chunk_z, world_generator);
        chunk.step_to_biomes(generator);
        chunk.set_structure_starts(generator);
        chunk.set_structure_references(generator);
        chunk.step_to_noise(generator);
        chunk.step_to_surface(generator);
        chunk.step_to_carvers(generator);
        chunk
    }
}
