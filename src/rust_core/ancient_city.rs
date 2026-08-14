use std::collections::HashMap;

use pumpkin_data::{
    Rotation,
    dimension::Dimension,
    structures::{Structure, StructureKeys},
    tag::{RegistryKey, get_tag_ids},
};
use pumpkin_util::{
    math::{block_box::BlockBox, vector3::Vector3},
    world_seed::Seed,
};
use pumpkin_world::generation::structure::structures::jigsaw::PoolElementStructurePiece;
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

use super::ContainerSeedShortcut;
use super::decoration_random::container_loot_seed;

const WORLD_MIN_Y: i32 = -64;
const SEA_LEVEL: i32 = 63;
const STRUCTURE_INDEX: i32 = 0;
const DECORATION_STEP: i32 = 7;

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
    generator: VanillaGenerator,
    valid_biomes: &'static [u16],
}

impl Scanner {
    #[must_use]
    pub fn new(world_seed: i64) -> Self {
        let biome_tag = Structure::ANCIENT_CITY
            .biomes
            .strip_prefix('#')
            .unwrap_or(Structure::ANCIENT_CITY.biomes);
        Self {
            world_seed,
            generator: VanillaGenerator::new(Seed(world_seed as u64), Dimension::OVERWORLD),
            valid_biomes: get_tag_ids(RegistryKey::WorldgenBiome, biome_tag)
                .expect("ancient city biome tag must exist"),
        }
    }

    pub fn scan(&self, chunk_x: i32, chunk_z: i32) -> Result<Scan, String> {
        let mut sampler = MultiNoiseSampler::generate(
            &self.generator.base_router.multi_noise,
            &MultiNoiseSamplerBuilderOptions::new(0, 0, 0),
        );
        self.scan_with_sampler(chunk_x, chunk_z, &mut sampler)
    }

    pub fn scan_many(
        &self,
        chunks: impl IntoIterator<Item = (i32, i32)>,
    ) -> Result<Vec<Scan>, String> {
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
    ) -> Result<Scan, String> {
        let probe_structure = Structure {
            size: Some(0),
            ..Structure::ANCIENT_CITY
        };
        let probe_context = StructureGeneratorContext {
            seed: self.world_seed,
            chunk_x,
            chunk_z,
            random: create_chunk_random(self.world_seed, chunk_x, chunk_z),
            sea_level: SEA_LEVEL,
            min_y: WORLD_MIN_Y,
            height_sampler: None,
            structure_key: Some(StructureKeys::AncientCity),
        };
        let Some(probe) = generate_structure_position(
            &StructureKeys::AncientCity,
            &probe_structure,
            probe_context,
        ) else {
            return Ok(Scan {
                valid_structure: false,
                chests: Vec::new(),
            });
        };
        let start = probe.start_pos.0;
        let biome = MultiNoiseBiomeSupplier::OVERWORLD.biome(
            biome_coords::from_block(start.x),
            biome_coords::from_block(start.y),
            biome_coords::from_block(start.z),
            sampler,
        );
        if !self.valid_biomes.contains(&(biome.id as u16)) {
            return Ok(Scan {
                valid_structure: false,
                chests: Vec::new(),
            });
        }

        let context = StructureGeneratorContext {
            seed: self.world_seed,
            chunk_x,
            chunk_z,
            random: create_chunk_random(self.world_seed, chunk_x, chunk_z),
            sea_level: SEA_LEVEL,
            min_y: WORLD_MIN_Y,
            height_sampler: None,
            structure_key: Some(StructureKeys::AncientCity),
        };
        let position = generate_structure_position(
            &StructureKeys::AncientCity,
            &Structure::ANCIENT_CITY,
            context,
        )
        .ok_or_else(|| "validated ancient city failed full placement".to_owned())?;

        let collector = position
            .collector
            .lock()
            .map_err(|_| "ancient city piece collector was poisoned".to_owned())?;
        let mut raw = Vec::new();
        for piece in &collector.pieces {
            let Some(piece) = piece.as_any().downcast_ref::<PoolElementStructurePiece>() else {
                continue;
            };
            collect_piece_chests(piece, chunk_x, chunk_z, &mut raw);
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
                STRUCTURE_INDEX,
                DECORATION_STEP,
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
}

struct RawChest {
    x: i32,
    y: i32,
    z: i32,
    loot_table: String,
}

fn collect_piece_chests(
    piece: &PoolElementStructurePiece,
    _structure_chunk_x: i32,
    _structure_chunk_z: i32,
    output: &mut Vec<RawChest>,
) {
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

#[allow(dead_code)]
fn _keep_block_box_in_public_api(_: BlockBox, _: Rotation) {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn scans_known_26_1_2_cities() {
        let scanner = Scanner::new(114514);
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
}
