use std::collections::HashMap;

use pumpkin_data::{
    dimension::Dimension,
    structures::{Structure, StructureKeys},
    tag::{RegistryKey, get_tag_ids},
};
use pumpkin_util::{math::vector3::Vector3, random::RandomImpl, world_seed::Seed};
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

use crate::catalog::ContainerSeedShortcut;
use crate::decoration_seed::container_loot_seed;

const OVERWORLD_MIN_Y: i32 = -64;
const NETHER_MIN_Y: i32 = 0;
const OVERWORLD_SEA_LEVEL: i32 = 63;
const NETHER_SEA_LEVEL: i32 = 32;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Kind {
    AncientCity,
    BastionRemnant,
}

impl Kind {
    fn structure(self) -> Structure {
        match self {
            Self::AncientCity => Structure::ANCIENT_CITY,
            Self::BastionRemnant => Structure::BASTION_REMNANT,
        }
    }

    const fn structure_key(self) -> StructureKeys {
        match self {
            Self::AncientCity => StructureKeys::AncientCity,
            Self::BastionRemnant => StructureKeys::BastionRemnant,
        }
    }

    const fn dimension(self) -> Dimension {
        match self {
            Self::AncientCity => Dimension::OVERWORLD,
            Self::BastionRemnant => Dimension::THE_NETHER,
        }
    }

    const fn min_y(self) -> i32 {
        match self {
            Self::AncientCity => OVERWORLD_MIN_Y,
            Self::BastionRemnant => NETHER_MIN_Y,
        }
    }

    const fn sea_level(self) -> i32 {
        match self {
            Self::AncientCity => OVERWORLD_SEA_LEVEL,
            Self::BastionRemnant => NETHER_SEA_LEVEL,
        }
    }

    const fn decoration_coordinates(self) -> (i32, i32) {
        match self {
            Self::AncientCity => (7, 0),
            Self::BastionRemnant => (4, 0),
        }
    }

    const fn biome_supplier(self) -> MultiNoiseBiomeSupplier {
        match self {
            Self::AncientCity => MultiNoiseBiomeSupplier::OVERWORLD,
            Self::BastionRemnant => MultiNoiseBiomeSupplier::NETHER,
        }
    }
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
    kind: Kind,
    generator: VanillaGenerator,
    valid_biomes: &'static [u16],
    fortress_biomes: &'static [u16],
}

impl Scanner {
    /// Build a scanner for a structure name that supports full chest scanning.
    pub fn for_structure(structure_name: &str, world_seed: i64) -> Result<Self, String> {
        let kind = match structure_name {
            "ancient_city" => Kind::AncientCity,
            "bastion_remnant" => Kind::BastionRemnant,
            _ => {
                return Err(format!(
                    "Rust chests and find do not support {structure_name} yet"
                ));
            }
        };
        Ok(Self::new(world_seed, kind))
    }

    #[must_use]
    pub fn new(world_seed: i64, kind: Kind) -> Self {
        let structure = kind.structure();
        Self {
            world_seed,
            kind,
            generator: VanillaGenerator::new(Seed(world_seed as u64), kind.dimension()),
            valid_biomes: structure_biomes(&structure),
            fortress_biomes: if kind == Kind::BastionRemnant {
                structure_biomes(&Structure::FORTRESS)
            } else {
                &[]
            },
        }
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
        if self.kind == Kind::BastionRemnant
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
        .ok_or_else(|| "validated jigsaw structure failed full placement".to_owned())?;

        let collector = position
            .collector
            .lock()
            .map_err(|_| "jigsaw piece collector was poisoned".to_owned())?;
        let mut raw = Vec::new();
        for piece in &collector.pieces {
            let Some(piece) = piece.as_any().downcast_ref::<PoolElementStructurePiece>() else {
                continue;
            };
            collect_piece_chests(piece, &mut raw);
        }

        let (decoration_step, structure_index) = self.kind.decoration_coordinates();
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
                structure_index,
                decoration_step,
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

    fn bastion_reached_in_weighted_selection(
        &self,
        chunk_x: i32,
        chunk_z: i32,
        sampler: &mut MultiNoiseSampler<'_>,
    ) -> Result<bool, String> {
        let mut selection_random = create_chunk_random(self.world_seed, chunk_x, chunk_z);
        let bastion_selected_first = selection_random.next_bounded_i32(5) >= 2;
        if bastion_selected_first {
            return Ok(true);
        }

        let block_x = chunk_x
            .checked_mul(16)
            .ok_or_else(|| "fortress biome probe x coordinate overflowed".to_owned())?;
        let block_z = chunk_z
            .checked_mul(16)
            .ok_or_else(|| "fortress biome probe z coordinate overflowed".to_owned())?;
        let fortress_start = Vector3::new(block_x, 64, block_z);
        Ok(!self.biome_is_valid(fortress_start, self.fortress_biomes, sampler))
    }

    fn biome_is_valid(
        &self,
        position: Vector3<i32>,
        valid_biomes: &[u16],
        sampler: &mut MultiNoiseSampler<'_>,
    ) -> bool {
        let biome = self.kind.biome_supplier().biome(
            biome_coords::from_block(position.x),
            biome_coords::from_block(position.y),
            biome_coords::from_block(position.z),
            sampler,
        );
        valid_biomes.contains(&(biome.id as u16))
    }
}

fn structure_biomes(structure: &Structure) -> &'static [u16] {
    let biome_tag = structure
        .biomes
        .strip_prefix('#')
        .unwrap_or(structure.biomes);
    get_tag_ids(RegistryKey::WorldgenBiome, biome_tag)
        .expect("vanilla structure biome tag must exist")
}

const fn invalid_scan() -> Scan {
    Scan {
        valid_structure: false,
        chests: Vec::new(),
    }
}

struct RawChest {
    x: i32,
    y: i32,
    z: i32,
    loot_table: String,
}

fn collect_piece_chests(piece: &PoolElementStructurePiece, output: &mut Vec<RawChest>) {
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    #[test]
    fn scans_known_26_1_2_cities() {
        let scanner = Scanner::new(114514, Kind::AncientCity);
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

    #[test]
    fn scans_known_26_1_2_bastions() {
        let scanner = Scanner::new(0, Kind::BastionRemnant);
        let scans = scanner
            .scan_many([(11, -14), (-27, -10), (62, 32)])
            .expect("scan known bastions");
        assert!(scans.iter().all(|scan| scan.valid_structure));
        assert_eq!(
            scans
                .iter()
                .map(|scan| scan.chests.len())
                .collect::<Vec<_>>(),
            [3, 11, 6]
        );
        assert_eq!(
            scans[0].chests.first(),
            Some(&Chest {
                structure_chunk_x: 11,
                structure_chunk_z: -14,
                x: 180,
                y: 80,
                z: -233,
                loot_table: "minecraft:chests/bastion_bridge".to_owned(),
                ordinal: 0,
                loot_seed: 1_335_123_538_721_756_194,
            })
        );
        assert_eq!(
            scans[1].chests.first(),
            Some(&Chest {
                structure_chunk_x: -27,
                structure_chunk_z: -10,
                x: -428,
                y: 35,
                z: -189,
                loot_table: "minecraft:chests/bastion_other".to_owned(),
                ordinal: 0,
                loot_seed: -5_513_880_696_554_537_352,
            })
        );
        assert_eq!(
            scans[2].chests.first(),
            Some(&Chest {
                structure_chunk_x: 62,
                structure_chunk_z: 32,
                x: 1011,
                y: 35,
                z: 496,
                loot_table: "minecraft:chests/bastion_treasure".to_owned(),
                ordinal: 0,
                loot_seed: -6_403_023_197_147_397_919,
            })
        );

        let tables = scans
            .iter()
            .flat_map(|scan| scan.chests.iter())
            .map(|chest| chest.loot_table.as_str())
            .collect::<HashSet<_>>();
        assert_eq!(
            tables,
            HashSet::from([
                "minecraft:chests/bastion_bridge",
                "minecraft:chests/bastion_hoglin_stable",
                "minecraft:chests/bastion_other",
                "minecraft:chests/bastion_treasure",
            ])
        );
    }
}
