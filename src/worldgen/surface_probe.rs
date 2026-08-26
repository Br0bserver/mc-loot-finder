use std::cell::RefCell;
use std::sync::Weak;

use glam::IVec2;
use rustc_hash::FxHashMap;
use steel_core::chunk::chunk_access::ChunkAccess;
use steel_core::chunk::heightmap::HeightmapType;
use steel_core::chunk::proto_chunk::ProtoChunk;
use steel_core::chunk::section::{ChunkSection, Sections};
use steel_core::worldgen::{ChunkGenerator, VanillaGenerator};
use steel_registry::RegistryEntry;
use steel_registry::blocks::block_state_ext::BlockStateExt;
use steel_registry::vanilla_blocks;
use steel_utils::{BlockPos, ChunkPos};
use steel_worldgen::biomes::BiomeSourceKind;
use steel_worldgen::density_functions::overworld::OverworldNoises;

const MIN_Y: i32 = -64;
const HEIGHT: i32 = 384;

pub(super) struct SurfaceTerrainSampler {
    generator: VanillaGenerator<OverworldNoises>,
    biome_source: BiomeSourceKind,
    chunks: FxHashMap<(i32, i32), ChunkAccess>,
}

impl SurfaceTerrainSampler {
    pub(super) fn new(world_seed: i64) -> Self {
        let seed = world_seed as u64;
        Self {
            generator: VanillaGenerator::new(BiomeSourceKind::overworld(seed), seed),
            biome_source: BiomeSourceKind::overworld(seed),
            chunks: FxHashMap::default(),
        }
    }

    pub(super) fn height(&mut self, x: i32, z: i32, ocean_floor: bool) -> i32 {
        let heightmap = if ocean_floor {
            HeightmapType::OceanFloorWg
        } else {
            HeightmapType::WorldSurfaceWg
        };
        let chunk = self.chunk(x.div_euclid(16), z.div_euclid(16));
        chunk.height_at(
            heightmap,
            x.rem_euclid(16) as usize,
            z.rem_euclid(16) as usize,
        )
    }

    pub(super) fn is_buried_treasure_support(&mut self, x: i32, y: i32, z: i32) -> bool {
        if y < MIN_Y {
            return false;
        }
        let state = self
            .chunk(x.div_euclid(16), z.div_euclid(16))
            .get_block_state(BlockPos::new(x, y, z));
        let block = state.get_block();
        block == &vanilla_blocks::SANDSTONE
            || block == &vanilla_blocks::STONE
            || block == &vanilla_blocks::ANDESITE
            || block == &vanilla_blocks::GRANITE
            || block == &vanilla_blocks::DIORITE
    }

    fn chunk(&mut self, chunk_x: i32, chunk_z: i32) -> &ChunkAccess {
        let key = (chunk_x, chunk_z);
        if !self.chunks.contains_key(&key) {
            let chunk = self.generate_chunk(chunk_x, chunk_z);
            self.chunks.insert(key, chunk);
        }
        self.chunks
            .get(&key)
            .expect("generated terrain chunk must be cached")
    }

    fn generate_chunk(&self, chunk_x: i32, chunk_z: i32) -> ChunkAccess {
        let sections = (0..(HEIGHT / 16))
            .map(|_| ChunkSection::new_empty())
            .collect::<Vec<_>>()
            .into_boxed_slice();
        let chunk = ChunkAccess::Proto(ProtoChunk::new(
            Sections::from_owned(sections),
            ChunkPos(IVec2::new(chunk_x, chunk_z)),
            MIN_Y,
            HEIGHT,
            Weak::<steel_core::world::World>::new(),
        ));

        self.generator.create_biomes(&chunk);
        self.generator.fill_from_noise(&chunk, None);

        let biome_sampler = RefCell::new(self.biome_source.chunk_sampler());
        self.generator
            .build_surface(&chunk, &|quart_x, quart_y, quart_z| {
                biome_sampler
                    .borrow_mut()
                    .sample(quart_x, quart_y, quart_z)
                    .id() as u16
            });
        chunk
    }
}
