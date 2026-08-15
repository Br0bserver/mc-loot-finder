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
pub struct CandidateStructure {
    pub name: &'static str,
    pub structure_id: &'static str,
    pub structure_path: &'static str,
    pub dimension: &'static str,
    pub placement: Placement,
    pub decoration_step: i32,
    pub structure_index: i32,
    pub container_seed: ContainerSeedShortcut,
    pub scanner: &'static str,
    pub loot_tables: &'static [&'static str],
    pub default_item: &'static str,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ContainerSeedShortcut {
    Direct,
    DesertPyramid,
    None,
}

const fn linear(spacing: i32, separation: i32, salt: i64) -> Placement {
    Placement {
        spacing,
        separation,
        salt,
        spread: SpreadType::Linear,
    }
}

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
        structure_id: "minecraft:ancient_city",
        structure_path: "ancient_city",
        dimension: "minecraft:overworld",
        placement: linear(24, 8, 20_083_232),
        decoration_step: 7,
        structure_index: 0,
        container_seed: ContainerSeedShortcut::Direct,
        scanner: "JIGSAW_FAST",
        loot_tables: &["minecraft:chests/ancient_city"],
        default_item: "minecraft:silence_armor_trim_smithing_template",
    },
    CandidateStructure {
        name: "bastion_remnant",
        structure_id: "minecraft:bastion_remnant",
        structure_path: "bastion_remnant",
        dimension: "minecraft:the_nether",
        placement: linear(27, 4, 30_084_232),
        decoration_step: 4,
        structure_index: 0,
        container_seed: ContainerSeedShortcut::Direct,
        scanner: "JIGSAW_FAST",
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
        structure_id: "minecraft:desert_pyramid",
        structure_path: "desert_pyramid",
        dimension: "minecraft:overworld",
        placement: linear(32, 8, 14_357_617),
        decoration_step: 4,
        structure_index: 1,
        container_seed: ContainerSeedShortcut::DesertPyramid,
        scanner: "VANILLA_PLACEMENT",
        loot_tables: &["minecraft:chests/desert_pyramid"],
        default_item: "minecraft:dune_armor_trim_smithing_template",
    },
    CandidateStructure {
        name: "jungle_pyramid",
        structure_id: "minecraft:jungle_pyramid",
        structure_path: "jungle_pyramid",
        dimension: "minecraft:overworld",
        placement: linear(32, 8, 14_357_619),
        decoration_step: 4,
        structure_index: 4,
        container_seed: ContainerSeedShortcut::None,
        scanner: "VANILLA_PLACEMENT",
        loot_tables: &[
            "minecraft:chests/jungle_temple",
            "minecraft:chests/jungle_temple_dispenser",
        ],
        default_item: "minecraft:wild_armor_trim_smithing_template",
    },
    CandidateStructure {
        name: "igloo",
        structure_id: "minecraft:igloo",
        structure_path: "igloo",
        dimension: "minecraft:overworld",
        placement: linear(32, 8, 14_357_618),
        decoration_step: 4,
        structure_index: 3,
        container_seed: ContainerSeedShortcut::None,
        scanner: "VANILLA_PLACEMENT",
        loot_tables: &["minecraft:chests/igloo_chest"],
        default_item: "minecraft:golden_apple",
    },
    CandidateStructure {
        name: "end_city",
        structure_id: "minecraft:end_city",
        structure_path: "end_city",
        dimension: "minecraft:the_end",
        placement: triangular(20, 11, 10_387_313),
        decoration_step: 4,
        structure_index: 2,
        container_seed: ContainerSeedShortcut::None,
        scanner: "VANILLA_PLACEMENT",
        loot_tables: &["minecraft:chests/end_city_treasure"],
        default_item: "minecraft:spire_armor_trim_smithing_template",
    },
    CandidateStructure {
        name: "ruined_portal",
        structure_id: "minecraft:ruined_portal",
        structure_path: "ruined_portal",
        dimension: "minecraft:overworld",
        placement: linear(40, 15, 34_222_645),
        decoration_step: -1,
        structure_index: -1,
        container_seed: ContainerSeedShortcut::None,
        scanner: "VANILLA_PLACEMENT",
        loot_tables: &["minecraft:chests/ruined_portal"],
        default_item: "minecraft:enchanted_golden_apple",
    },
    CandidateStructure {
        name: "ruined_portal_nether",
        structure_id: "minecraft:ruined_portal_nether",
        structure_path: "ruined_portal_nether",
        dimension: "minecraft:the_nether",
        placement: linear(40, 15, 34_222_645),
        decoration_step: -1,
        structure_index: -1,
        container_seed: ContainerSeedShortcut::None,
        scanner: "VANILLA_PLACEMENT",
        loot_tables: &["minecraft:chests/ruined_portal"],
        default_item: "minecraft:enchanted_golden_apple",
    },
    CandidateStructure {
        name: "trial_chambers",
        structure_id: "minecraft:trial_chambers",
        structure_path: "trial_chambers",
        dimension: "minecraft:overworld",
        placement: linear(34, 12, 94_251_327),
        decoration_step: -1,
        structure_index: -1,
        container_seed: ContainerSeedShortcut::None,
        scanner: "VANILLA_PLACEMENT",
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
        structure_id: "minecraft:shipwreck",
        structure_path: "shipwreck",
        dimension: "minecraft:overworld",
        placement: linear(24, 4, 165_745_295),
        decoration_step: -1,
        structure_index: -1,
        container_seed: ContainerSeedShortcut::None,
        scanner: "VANILLA_PLACEMENT",
        loot_tables: &[
            "minecraft:chests/shipwreck_map",
            "minecraft:chests/shipwreck_supply",
            "minecraft:chests/shipwreck_treasure",
        ],
        default_item: "minecraft:coast_armor_trim_smithing_template",
    },
    CandidateStructure {
        name: "ocean_ruin",
        structure_id: "minecraft:ocean_ruin_cold",
        structure_path: "ocean_ruin_cold",
        dimension: "minecraft:overworld",
        placement: linear(20, 8, 14_357_621),
        decoration_step: -1,
        structure_index: -1,
        container_seed: ContainerSeedShortcut::None,
        scanner: "VANILLA_PLACEMENT",
        loot_tables: &[
            "minecraft:chests/underwater_ruin_big",
            "minecraft:chests/underwater_ruin_small",
        ],
        default_item: "minecraft:golden_apple",
    },
    CandidateStructure {
        name: "nether_fortress",
        structure_id: "minecraft:fortress",
        structure_path: "fortress",
        dimension: "minecraft:the_nether",
        placement: linear(27, 4, 30_084_232),
        decoration_step: -1,
        structure_index: -1,
        container_seed: ContainerSeedShortcut::None,
        scanner: "VANILLA_PLACEMENT",
        loot_tables: &["minecraft:chests/nether_bridge"],
        default_item: "minecraft:diamond",
    },
    CandidateStructure {
        name: "village",
        structure_id: "minecraft:village_plains",
        structure_path: "village_plains",
        dimension: "minecraft:overworld",
        placement: linear(34, 8, 10_387_312),
        decoration_step: -1,
        structure_index: -1,
        container_seed: ContainerSeedShortcut::None,
        scanner: "JIGSAW_FAST",
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
        structure_id: "minecraft:buried_treasure",
        structure_path: "buried_treasure",
        dimension: "minecraft:overworld",
        placement: linear(1, 0, 0),
        decoration_step: -1,
        structure_index: -1,
        container_seed: ContainerSeedShortcut::None,
        scanner: "VANILLA_PLACEMENT",
        loot_tables: &["minecraft:chests/buried_treasure"],
        default_item: "minecraft:heart_of_the_sea",
    },
    CandidateStructure {
        name: "pillager_outpost",
        structure_id: "minecraft:pillager_outpost",
        structure_path: "pillager_outpost",
        dimension: "minecraft:overworld",
        placement: linear(32, 8, 165_745_296),
        decoration_step: -1,
        structure_index: -1,
        container_seed: ContainerSeedShortcut::None,
        scanner: "JIGSAW_FAST",
        loot_tables: &["minecraft:chests/pillager_outpost"],
        default_item: "minecraft:sentry_armor_trim_smithing_template",
    },
    CandidateStructure {
        name: "woodland_mansion",
        structure_id: "minecraft:mansion",
        structure_path: "mansion",
        dimension: "minecraft:overworld",
        placement: triangular(80, 20, 10_387_319),
        decoration_step: 4,
        structure_index: 5,
        container_seed: ContainerSeedShortcut::None,
        scanner: "VANILLA_PLACEMENT",
        loot_tables: &["minecraft:chests/woodland_mansion"],
        default_item: "minecraft:vex_armor_trim_smithing_template",
    },
];

pub fn candidate_structure(name: &str) -> Result<&'static CandidateStructure, String> {
    let normalized = name.strip_prefix("minecraft:").unwrap_or(name);
    CANDIDATE_STRUCTURES
        .iter()
        .find(|structure| structure.name == normalized || structure.structure_path == normalized)
        .ok_or_else(|| {
            format!(
                "unsupported structure: {name}; supported: {}",
                CANDIDATE_STRUCTURES
                    .iter()
                    .map(|structure| structure.name)
                    .collect::<Vec<_>>()
                    .join(", ")
            )
        })
}
