use crate::catalog::ScanKind;

use pumpkin_data::{
    dimension::Dimension,
    structures::{Structure, StructureKeys},
};
use pumpkin_world::biome::MultiNoiseBiomeSupplier;

use super::{NETHER_MIN_Y, NETHER_SEA_LEVEL, OVERWORLD_MIN_Y, OVERWORLD_SEA_LEVEL};

impl ScanKind {
    fn profile(self) -> KindProfile {
        match self {
            Self::AncientCity => KindProfile {
                structure: Structure::ANCIENT_CITY,
                key: StructureKeys::AncientCity,
                dimension: Dimension::OVERWORLD,
                min_y: OVERWORLD_MIN_Y,
                sea_level: OVERWORLD_SEA_LEVEL,
                biome: MultiNoiseBiomeSupplier::OVERWORLD,
            },
            Self::BastionRemnant => KindProfile {
                structure: Structure::BASTION_REMNANT,
                key: StructureKeys::BastionRemnant,
                dimension: Dimension::THE_NETHER,
                min_y: NETHER_MIN_Y,
                sea_level: NETHER_SEA_LEVEL,
                biome: MultiNoiseBiomeSupplier::NETHER,
            },
            Self::DesertPyramid => KindProfile {
                structure: Structure::DESERT_PYRAMID,
                key: StructureKeys::DesertPyramid,
                dimension: Dimension::OVERWORLD,
                min_y: OVERWORLD_MIN_Y,
                sea_level: OVERWORLD_SEA_LEVEL,
                biome: MultiNoiseBiomeSupplier::OVERWORLD,
            },
            Self::Igloo => KindProfile {
                structure: Structure::IGLOO,
                key: StructureKeys::Igloo,
                dimension: Dimension::OVERWORLD,
                min_y: OVERWORLD_MIN_Y,
                sea_level: OVERWORLD_SEA_LEVEL,
                biome: MultiNoiseBiomeSupplier::OVERWORLD,
            },
            Self::Village => KindProfile {
                structure: Structure::VILLAGE_PLAINS,
                key: StructureKeys::VillagePlains,
                dimension: Dimension::OVERWORLD,
                min_y: OVERWORLD_MIN_Y,
                sea_level: OVERWORLD_SEA_LEVEL,
                biome: MultiNoiseBiomeSupplier::OVERWORLD,
            },
            Self::PillagerOutpost => KindProfile {
                structure: Structure::PILLAGER_OUTPOST,
                key: StructureKeys::PillagerOutpost,
                dimension: Dimension::OVERWORLD,
                min_y: OVERWORLD_MIN_Y,
                sea_level: OVERWORLD_SEA_LEVEL,
                biome: MultiNoiseBiomeSupplier::OVERWORLD,
            },
            Self::BuriedTreasure => KindProfile {
                structure: Structure::BURIED_TREASURE,
                key: StructureKeys::BuriedTreasure,
                dimension: Dimension::OVERWORLD,
                min_y: OVERWORLD_MIN_Y,
                sea_level: OVERWORLD_SEA_LEVEL,
                biome: MultiNoiseBiomeSupplier::OVERWORLD,
            },
        }
    }
    pub(crate) fn structure(self) -> Structure {
        self.profile().structure
    }
    pub(crate) fn structure_key(self) -> StructureKeys {
        self.profile().key
    }
    pub(crate) fn dimension(self) -> Dimension {
        self.profile().dimension
    }
    pub(crate) fn min_y(self) -> i32 {
        self.profile().min_y
    }
    pub(crate) fn sea_level(self) -> i32 {
        self.profile().sea_level
    }
    pub(crate) fn biome_supplier(self) -> MultiNoiseBiomeSupplier {
        self.profile().biome
    }
}

struct KindProfile {
    structure: Structure,
    key: StructureKeys,
    dimension: Dimension,
    min_y: i32,
    sea_level: i32,
    biome: MultiNoiseBiomeSupplier,
}
