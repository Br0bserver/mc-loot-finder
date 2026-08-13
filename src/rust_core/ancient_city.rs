use std::collections::HashMap;

use pumpkin_data::{
    Rotation,
    dimension::Dimension,
    structures::{Structure, StructureKeys},
};
use pumpkin_util::{
    math::{block_box::BlockBox, vector3::Vector3},
    world_seed::Seed,
};
use pumpkin_world::generation::structure::structures::jigsaw::PoolElementStructurePiece;
use pumpkin_world::{
    biome::MultiNoiseBiomeSupplier,
    generation::{
        generator::{GeneratorInit, VanillaGenerator},
        noise::router::multi_noise_sampler::{MultiNoiseSampler, MultiNoiseSamplerBuilderOptions},
        structure::{
            lazily_generate_structure,
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
}

impl Scanner {
    #[must_use]
    pub fn new(world_seed: i64) -> Self {
        Self {
            world_seed,
            generator: VanillaGenerator::new(Seed(world_seed as u64), Dimension::OVERWORLD),
        }
    }

    pub fn scan(&self, chunk_x: i32, chunk_z: i32) -> Result<Scan, String> {
        let mut sampler = MultiNoiseSampler::generate(
            &self.generator.base_router.multi_noise,
            &MultiNoiseSamplerBuilderOptions::new(0, 0, 0),
        );
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
        let Some(position) = lazily_generate_structure(
            &StructureKeys::AncientCity,
            &Structure::ANCIENT_CITY,
            context,
            &MultiNoiseBiomeSupplier::OVERWORLD,
            &mut sampler,
        ) else {
            return Ok(Scan {
                valid_structure: false,
                chests: Vec::new(),
            });
        };

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
    use pumpkin_world::{
        biome::BiomeSupplier,
        generation::{
            biome_coords,
            structure::{generate_structure_position, structures::StructureGeneratorContext},
        },
    };

    use super::*;

    #[test]
    fn diagnose_26_1_2_biome_mismatch() {
        let scanner = Scanner::new(114514);
        for (chunk_x, chunk_z) in [(96, 5), (197, 222)] {
            let context = StructureGeneratorContext {
                seed: scanner.world_seed,
                chunk_x,
                chunk_z,
                random: create_chunk_random(scanner.world_seed, chunk_x, chunk_z),
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
            .expect("ancient city candidate should produce a jigsaw position");
            let start = position.start_pos.0;
            let mut sampler = MultiNoiseSampler::generate(
                &scanner.generator.base_router.multi_noise,
                &MultiNoiseSamplerBuilderOptions::new(0, 0, 0),
            );
            let biome_x = biome_coords::from_block(start.x);
            let biome_y = biome_coords::from_block(start.y);
            let biome_z = biome_coords::from_block(start.z);
            let point = sampler.sample(biome_x, biome_y, biome_z);
            let biome =
                MultiNoiseBiomeSupplier::OVERWORLD.biome(biome_x, biome_y, biome_z, &mut sampler);
            println!(
                "candidate=({chunk_x},{chunk_z}) start=({},{},{}) biome=minecraft:{} id={} climate=({},{},{},{},{},{})",
                start.x,
                start.y,
                start.z,
                biome.registry_id,
                biome.id,
                point.temperature,
                point.humidity,
                point.continentalness,
                point.erosion,
                point.depth,
                point.weirdness
            );
        }
    }

    #[test]
    fn diagnose_26_1_2_layout_mismatch() {
        let scanner = Scanner::new(114514);
        let chunk_x = 96;
        let chunk_z = 5;
        let context = StructureGeneratorContext {
            seed: scanner.world_seed,
            chunk_x,
            chunk_z,
            random: create_chunk_random(scanner.world_seed, chunk_x, chunk_z),
            sea_level: SEA_LEVEL,
            min_y: WORLD_MIN_Y,
            height_sampler: None,
            structure_key: Some(StructureKeys::AncientCity),
        };
        let mut sampler = MultiNoiseSampler::generate(
            &scanner.generator.base_router.multi_noise,
            &MultiNoiseSamplerBuilderOptions::new(0, 0, 0),
        );
        let position = lazily_generate_structure(
            &StructureKeys::AncientCity,
            &Structure::ANCIENT_CITY,
            context,
            &MultiNoiseBiomeSupplier::OVERWORLD,
            &mut sampler,
        )
        .expect("ancient city should generate");
        let collector = position.collector.lock().expect("collector");

        println!("PUMPKIN_LAYOUT pieces={}", collector.pieces.len());
        for (piece_index, piece) in collector.pieces.iter().enumerate() {
            let piece = piece
                .as_any()
                .downcast_ref::<PoolElementStructurePiece>()
                .expect("ancient city jigsaw piece");
            let origin = piece.pos.0;
            let mut templates = Vec::new();
            piece.element.for_each_template(|name, _, _, _| {
                templates.push(name.to_owned());
            });
            println!(
                "PUMPKIN_PIECE index={piece_index:03} position={},{},{} rotation={:?} templates={:?} box={},{},{}..{},{},{}",
                origin.x,
                origin.y,
                origin.z,
                piece.rotation,
                templates,
                piece.piece.bounding_box.min.x,
                piece.piece.bounding_box.min.y,
                piece.piece.bounding_box.min.z,
                piece.piece.bounding_box.max.x,
                piece.piece.bounding_box.max.y,
                piece.piece.bounding_box.max.z,
            );
            for (junction_index, junction) in piece.junctions.iter().enumerate() {
                println!(
                    "PUMPKIN_JUNCTION piece={piece_index:03} junction={junction_index:02} source={},{},{} delta_y={} projection={:?}",
                    junction.source_x,
                    junction.source_ground_y,
                    junction.source_z,
                    junction.delta_y,
                    junction.projection,
                );
            }
            piece.element.for_each_template(|name, _, _, template| {
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
                    println!(
                        "PUMPKIN_CHEST piece={piece_index} template={name} piece_position=({},{},{}) rotation={:?} template_size=({},{},{}) template_local=({},{},{}) transformed_local=({},{},{}) world=({},{},{}) box=({},{},{})..({},{},{})",
                        origin.x,
                        origin.y,
                        origin.z,
                        piece.rotation,
                        template.size.x,
                        template.size.y,
                        template.size.z,
                        block.pos.x,
                        block.pos.y,
                        block.pos.z,
                        local.x,
                        local.y,
                        local.z,
                        world.x,
                        world.y,
                        world.z,
                        piece.piece.bounding_box.min.x,
                        piece.piece.bounding_box.min.y,
                        piece.piece.bounding_box.min.z,
                        piece.piece.bounding_box.max.x,
                        piece.piece.bounding_box.max.y,
                        piece.piece.bounding_box.max.z,
                    );
                }
            });
        }
    }
}
