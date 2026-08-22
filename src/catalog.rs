use crate::error::Error;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SpreadType {
    Linear,
    Triangular,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Placement {
    pub spacing: i32,
    pub separation: i32,
    pub salt: i64,
    pub spread: SpreadType,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ScanKind {
    AncientCity,
    BastionRemnant,
    DesertPyramid,
    Igloo,
    Village,
    PillagerOutpost,
    BuriedTreasure,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ContainerSeedShortcut {
    Direct,
    DesertPyramid,
}

impl ContainerSeedShortcut {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Direct => "DIRECT",
            Self::DesertPyramid => "DESERT_PYRAMID",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct DecorationSeedSpec {
    pub structure_index: i32,
    pub step: i32,
    pub ordinal_offset: i32,
    pub shortcut: ContainerSeedShortcut,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ScanSupport {
    CandidatesOnly,
    Full(ScanKind),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ScannerBackend {
    JigsawFast,
    VanillaPlacement,
}

impl ScannerBackend {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::JigsawFast => "JIGSAW_FAST",
            Self::VanillaPlacement => "VANILLA_PLACEMENT",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CandidateStructure {
    pub name: &'static str,
    pub structure_id: &'static str,
    pub structure_path: &'static str,
    pub dimension: &'static str,
    pub placement: Placement,
    pub support: ScanSupport,
    pub decoration: Option<DecorationSeedSpec>,
    pub reference_scanner: ScannerBackend,
    pub loot_tables: &'static [&'static str],
    pub default_item: &'static str,
}

const fn linear(spacing: i32, separation: i32, salt: i64) -> Placement {
    Placement {
        spacing,
        separation,
        salt,
        spread: SpreadType::Linear,
    }
}

pub const VILLAGE_PLACEMENT: Placement = linear(34, 8, 10_387_312);

const fn triangular(spacing: i32, separation: i32, salt: i64) -> Placement {
    Placement {
        spacing,
        separation,
        salt,
        spread: SpreadType::Triangular,
    }
}

pub const CANDIDATE_STRUCTURES: &[CandidateStructure] = &[
    CandidateStructure {
        name: "ancient_city",
        support: ScanSupport::Full(ScanKind::AncientCity),
        structure_id: "minecraft:ancient_city",
        structure_path: "ancient_city",
        dimension: "minecraft:overworld",
        placement: linear(24, 8, 20_083_232),
        decoration: Some(DecorationSeedSpec {
            structure_index: 0,
            step: 7,
            ordinal_offset: 0,
            shortcut: ContainerSeedShortcut::Direct,
        }),
        reference_scanner: ScannerBackend::JigsawFast,
        loot_tables: &["minecraft:chests/ancient_city"],
        default_item: "minecraft:silence_armor_trim_smithing_template",
    },
    CandidateStructure {
        name: "bastion_remnant",
        support: ScanSupport::Full(ScanKind::BastionRemnant),
        structure_id: "minecraft:bastion_remnant",
        structure_path: "bastion_remnant",
        dimension: "minecraft:the_nether",
        placement: linear(27, 4, 30_084_232),
        decoration: Some(DecorationSeedSpec {
            structure_index: 0,
            step: 4,
            ordinal_offset: 0,
            shortcut: ContainerSeedShortcut::Direct,
        }),
        reference_scanner: ScannerBackend::JigsawFast,
        loot_tables: &[
            "minecraft:chests/bastion_bridge",
            "minecraft:chests/bastion_hoglin_stable",
            "minecraft:chests/bastion_other",
            "minecraft:chests/bastion_treasure",
        ],
        default_item: "minecraft:netherite_upgrade_smithing_template",
    },
    CandidateStructure {
        name: "desert_pyramid",
        support: ScanSupport::Full(ScanKind::DesertPyramid),
        structure_id: "minecraft:desert_pyramid",
        structure_path: "desert_pyramid",
        dimension: "minecraft:overworld",
        placement: linear(32, 8, 14_357_617),
        decoration: Some(DecorationSeedSpec {
            structure_index: 1,
            step: 4,
            ordinal_offset: 0,
            shortcut: ContainerSeedShortcut::DesertPyramid,
        }),
        reference_scanner: ScannerBackend::VanillaPlacement,
        loot_tables: &["minecraft:chests/desert_pyramid"],
        default_item: "minecraft:dune_armor_trim_smithing_template",
    },
    CandidateStructure {
        name: "jungle_pyramid",
        support: ScanSupport::CandidatesOnly,
        structure_id: "minecraft:jungle_pyramid",
        structure_path: "jungle_pyramid",
        dimension: "minecraft:overworld",
        placement: linear(32, 8, 14_357_619),
        decoration: None,
        reference_scanner: ScannerBackend::VanillaPlacement,
        loot_tables: &[
            "minecraft:chests/jungle_temple",
            "minecraft:chests/jungle_temple_dispenser",
        ],
        default_item: "minecraft:wild_armor_trim_smithing_template",
    },
    CandidateStructure {
        name: "igloo",
        support: ScanSupport::Full(ScanKind::Igloo),
        structure_id: "minecraft:igloo",
        structure_path: "igloo",
        dimension: "minecraft:overworld",
        placement: linear(32, 8, 14_357_618),
        decoration: Some(DecorationSeedSpec {
            structure_index: 3,
            step: 4,
            ordinal_offset: 1,
            shortcut: ContainerSeedShortcut::Direct,
        }),
        reference_scanner: ScannerBackend::VanillaPlacement,
        loot_tables: &["minecraft:chests/igloo_chest"],
        default_item: "minecraft:golden_apple",
    },
    CandidateStructure {
        name: "end_city",
        support: ScanSupport::CandidatesOnly,
        structure_id: "minecraft:end_city",
        structure_path: "end_city",
        dimension: "minecraft:the_end",
        placement: triangular(20, 11, 10_387_313),
        decoration: None,
        reference_scanner: ScannerBackend::VanillaPlacement,
        loot_tables: &["minecraft:chests/end_city_treasure"],
        default_item: "minecraft:spire_armor_trim_smithing_template",
    },
    CandidateStructure {
        name: "ruined_portal",
        support: ScanSupport::CandidatesOnly,
        structure_id: "minecraft:ruined_portal",
        structure_path: "ruined_portal",
        dimension: "minecraft:overworld",
        placement: linear(40, 15, 34_222_645),
        decoration: None,
        reference_scanner: ScannerBackend::VanillaPlacement,
        loot_tables: &["minecraft:chests/ruined_portal"],
        default_item: "minecraft:enchanted_golden_apple",
    },
    CandidateStructure {
        name: "ruined_portal_nether",
        support: ScanSupport::CandidatesOnly,
        structure_id: "minecraft:ruined_portal_nether",
        structure_path: "ruined_portal_nether",
        dimension: "minecraft:the_nether",
        placement: linear(40, 15, 34_222_645),
        decoration: None,
        reference_scanner: ScannerBackend::VanillaPlacement,
        loot_tables: &["minecraft:chests/ruined_portal"],
        default_item: "minecraft:enchanted_golden_apple",
    },
    CandidateStructure {
        name: "trial_chambers",
        support: ScanSupport::CandidatesOnly,
        structure_id: "minecraft:trial_chambers",
        structure_path: "trial_chambers",
        dimension: "minecraft:overworld",
        placement: linear(34, 12, 94_251_327),
        decoration: None,
        reference_scanner: ScannerBackend::VanillaPlacement,
        loot_tables: &[
            "minecraft:chests/trial_chambers/corridor",
            "minecraft:chests/trial_chambers/entrance",
            "minecraft:chests/trial_chambers/intersection",
            "minecraft:chests/trial_chambers/intersection_barrel",
            "minecraft:chests/trial_chambers/supply",
            "minecraft:chests/trial_chambers/reward",
            "minecraft:dispensers/trial_chambers/chamber",
            "minecraft:dispensers/trial_chambers/corridor",
            "minecraft:dispensers/trial_chambers/water",
            "minecraft:pots/trial_chambers/corridor",
        ],
        default_item: "minecraft:trial_key",
    },
    CandidateStructure {
        name: "shipwreck",
        support: ScanSupport::CandidatesOnly,
        structure_id: "minecraft:shipwreck",
        structure_path: "shipwreck",
        dimension: "minecraft:overworld",
        placement: linear(24, 4, 165_745_295),
        decoration: None,
        reference_scanner: ScannerBackend::VanillaPlacement,
        loot_tables: &[
            "minecraft:chests/shipwreck_map",
            "minecraft:chests/shipwreck_supply",
            "minecraft:chests/shipwreck_treasure",
        ],
        default_item: "minecraft:coast_armor_trim_smithing_template",
    },
    CandidateStructure {
        name: "ocean_ruin",
        support: ScanSupport::CandidatesOnly,
        structure_id: "minecraft:ocean_ruin_cold",
        structure_path: "ocean_ruin_cold",
        dimension: "minecraft:overworld",
        placement: linear(20, 8, 14_357_621),
        decoration: None,
        reference_scanner: ScannerBackend::VanillaPlacement,
        loot_tables: &[
            "minecraft:chests/underwater_ruin_big",
            "minecraft:chests/underwater_ruin_small",
        ],
        default_item: "minecraft:golden_apple",
    },
    CandidateStructure {
        name: "nether_fortress",
        support: ScanSupport::CandidatesOnly,
        structure_id: "minecraft:fortress",
        structure_path: "fortress",
        dimension: "minecraft:the_nether",
        placement: linear(27, 4, 30_084_232),
        decoration: None,
        reference_scanner: ScannerBackend::VanillaPlacement,
        loot_tables: &["minecraft:chests/nether_bridge"],
        default_item: "minecraft:diamond",
    },
    CandidateStructure {
        name: "village",
        support: ScanSupport::Full(ScanKind::Village),
        structure_id: "minecraft:village_plains",
        structure_path: "village_plains",
        dimension: "minecraft:overworld",
        placement: VILLAGE_PLACEMENT,
        decoration: None,
        reference_scanner: ScannerBackend::JigsawFast,
        loot_tables: &[
            "minecraft:chests/village/village_armorer",
            "minecraft:chests/village/village_butcher",
            "minecraft:chests/village/village_cartographer",
            "minecraft:chests/village/village_desert_house",
            "minecraft:chests/village/village_fisher",
            "minecraft:chests/village/village_fletcher",
            "minecraft:chests/village/village_mason",
            "minecraft:chests/village/village_plains_house",
            "minecraft:chests/village/village_savanna_house",
            "minecraft:chests/village/village_shepherd",
            "minecraft:chests/village/village_snowy_house",
            "minecraft:chests/village/village_taiga_house",
            "minecraft:chests/village/village_tannery",
            "minecraft:chests/village/village_temple",
            "minecraft:chests/village/village_toolsmith",
            "minecraft:chests/village/village_weaponsmith",
        ],
        default_item: "minecraft:diamond",
    },
    CandidateStructure {
        name: "buried_treasure",
        support: ScanSupport::Full(ScanKind::BuriedTreasure),
        structure_id: "minecraft:buried_treasure",
        structure_path: "buried_treasure",
        dimension: "minecraft:overworld",
        placement: linear(1, 0, 0),
        decoration: None,
        reference_scanner: ScannerBackend::VanillaPlacement,
        loot_tables: &["minecraft:chests/buried_treasure"],
        default_item: "minecraft:heart_of_the_sea",
    },
    CandidateStructure {
        name: "pillager_outpost",
        support: ScanSupport::Full(ScanKind::PillagerOutpost),
        structure_id: "minecraft:pillager_outpost",
        structure_path: "pillager_outpost",
        dimension: "minecraft:overworld",
        placement: linear(32, 8, 165_745_296),
        decoration: Some(DecorationSeedSpec {
            structure_index: 4,
            step: 9,
            ordinal_offset: 0,
            shortcut: ContainerSeedShortcut::Direct,
        }),
        reference_scanner: ScannerBackend::JigsawFast,
        loot_tables: &["minecraft:chests/pillager_outpost"],
        default_item: "minecraft:sentry_armor_trim_smithing_template",
    },
    CandidateStructure {
        name: "woodland_mansion",
        support: ScanSupport::CandidatesOnly,
        structure_id: "minecraft:mansion",
        structure_path: "mansion",
        dimension: "minecraft:overworld",
        placement: triangular(80, 20, 10_387_319),
        decoration: None,
        reference_scanner: ScannerBackend::VanillaPlacement,
        loot_tables: &["minecraft:chests/woodland_mansion"],
        default_item: "minecraft:vex_armor_trim_smithing_template",
    },
];

impl CandidateStructure {
    /// Whether `chests` and `find` have an exact world-generation scanner.
    pub const fn supports_full_scan(&self) -> bool {
        matches!(self.support, ScanSupport::Full(_))
    }
}

pub fn candidate_structure(name: &str) -> Result<&'static CandidateStructure, Error> {
    let normalized = name.strip_prefix("minecraft:").unwrap_or(name);
    CANDIDATE_STRUCTURES
        .iter()
        .find(|structure| structure.name == normalized || structure.structure_path == normalized)
        .ok_or_else(|| {
            Error::Structure(format!(
                "unsupported structure: {name}; supported: {}",
                CANDIDATE_STRUCTURES
                    .iter()
                    .map(|structure| structure.name)
                    .collect::<Vec<_>>()
                    .join(", ")
            ))
        })
}
