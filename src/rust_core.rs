pub mod candidates;
pub mod decoration_random;
pub mod legacy_random;
pub mod xoroshiro;

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
    pub structure_path: &'static str,
    pub placement: Placement,
    pub decoration_step: i32,
    pub structure_index: i32,
    pub container_seed: ContainerSeedShortcut,
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
        structure_path: "ancient_city",
        placement: linear(24, 8, 20_083_232),
        decoration_step: 7,
        structure_index: 0,
        container_seed: ContainerSeedShortcut::Direct,
    },
    CandidateStructure {
        name: "bastion_remnant",
        structure_path: "bastion_remnant",
        placement: linear(27, 4, 30_084_232),
        decoration_step: 4,
        structure_index: 0,
        container_seed: ContainerSeedShortcut::Direct,
    },
    CandidateStructure {
        name: "desert_pyramid",
        structure_path: "desert_pyramid",
        placement: linear(32, 8, 14_357_617),
        decoration_step: 4,
        structure_index: 1,
        container_seed: ContainerSeedShortcut::DesertPyramid,
    },
    CandidateStructure {
        name: "jungle_pyramid",
        structure_path: "jungle_pyramid",
        placement: linear(32, 8, 14_357_619),
        decoration_step: 4,
        structure_index: 4,
        container_seed: ContainerSeedShortcut::None,
    },
    CandidateStructure {
        name: "igloo",
        structure_path: "igloo",
        placement: linear(32, 8, 14_357_618),
        decoration_step: 4,
        structure_index: 3,
        container_seed: ContainerSeedShortcut::None,
    },
    CandidateStructure {
        name: "end_city",
        structure_path: "end_city",
        placement: triangular(20, 11, 10_387_313),
        decoration_step: 4,
        structure_index: 2,
        container_seed: ContainerSeedShortcut::None,
    },
    CandidateStructure {
        name: "ruined_portal",
        structure_path: "ruined_portal",
        placement: linear(40, 15, 34_222_645),
        decoration_step: -1,
        structure_index: -1,
        container_seed: ContainerSeedShortcut::None,
    },
    CandidateStructure {
        name: "ruined_portal_nether",
        structure_path: "ruined_portal_nether",
        placement: linear(40, 15, 34_222_645),
        decoration_step: -1,
        structure_index: -1,
        container_seed: ContainerSeedShortcut::None,
    },
    CandidateStructure {
        name: "trial_chambers",
        structure_path: "trial_chambers",
        placement: linear(34, 12, 94_251_327),
        decoration_step: -1,
        structure_index: -1,
        container_seed: ContainerSeedShortcut::None,
    },
    CandidateStructure {
        name: "shipwreck",
        structure_path: "shipwreck",
        placement: linear(24, 4, 165_745_295),
        decoration_step: -1,
        structure_index: -1,
        container_seed: ContainerSeedShortcut::None,
    },
    CandidateStructure {
        name: "ocean_ruin",
        structure_path: "ocean_ruin_cold",
        placement: linear(20, 8, 14_357_621),
        decoration_step: -1,
        structure_index: -1,
        container_seed: ContainerSeedShortcut::None,
    },
    CandidateStructure {
        name: "nether_fortress",
        structure_path: "fortress",
        placement: linear(27, 4, 30_084_232),
        decoration_step: -1,
        structure_index: -1,
        container_seed: ContainerSeedShortcut::None,
    },
    CandidateStructure {
        name: "village",
        structure_path: "village_plains",
        placement: linear(34, 8, 10_387_312),
        decoration_step: -1,
        structure_index: -1,
        container_seed: ContainerSeedShortcut::None,
    },
    CandidateStructure {
        name: "buried_treasure",
        structure_path: "buried_treasure",
        placement: linear(1, 0, 0),
        decoration_step: -1,
        structure_index: -1,
        container_seed: ContainerSeedShortcut::None,
    },
    CandidateStructure {
        name: "pillager_outpost",
        structure_path: "pillager_outpost",
        placement: linear(32, 8, 165_745_296),
        decoration_step: -1,
        structure_index: -1,
        container_seed: ContainerSeedShortcut::None,
    },
    CandidateStructure {
        name: "woodland_mansion",
        structure_path: "mansion",
        placement: triangular(80, 20, 10_387_319),
        decoration_step: 4,
        structure_index: 5,
        container_seed: ContainerSeedShortcut::None,
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
