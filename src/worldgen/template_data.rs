//! Precompiled structure template container definitions for Minecraft 26.1.2.
//!
//! Generated from extracted vanilla 26.1.2 structure NBT templates.

use rustc_hash::FxHashMap;
use std::sync::LazyLock;

#[derive(Clone, Copy, Debug)]
pub struct TemplateChest {
    pub x: i32,
    pub y: i32,
    pub z: i32,
    pub loot_table: &'static str,
}

#[derive(Clone, Copy, Debug)]
pub struct TemplateMarker {
    pub x: i32,
    pub y: i32,
    pub z: i32,
    pub metadata: &'static str,
}

#[derive(Clone, Copy, Debug)]
pub struct TemplateBlockPos {
    pub x: i32,
    pub y: i32,
    pub z: i32,
}

#[derive(Clone, Debug)]
pub struct TemplateContainerData {
    pub size: [i32; 3],
    pub chests: &'static [TemplateChest],
    pub markers: &'static [TemplateMarker],
    pub randomizable_containers: &'static [TemplateBlockPos],
}

pub static TEMPLATE_CONTAINERS: LazyLock<FxHashMap<&'static str, TemplateContainerData>> =
    LazyLock::new(|| {
        let mut map = FxHashMap::default();
        map.insert(
            "ancient_city/city_center/city_center_2",
            TemplateContainerData {
                size: [18, 31, 41],
                chests: &[],
                markers: &[],
                randomizable_containers: &[TemplateBlockPos { x: 9, y: 8, z: 20 }],
            },
        );
        map.insert(
            "ancient_city/structures/barracks",
            TemplateContainerData {
                size: [21, 12, 17],
                chests: &[
                    TemplateChest {
                        x: 4,
                        y: 4,
                        z: 15,
                        loot_table: "minecraft:chests/ancient_city",
                    },
                    TemplateChest {
                        x: 12,
                        y: 4,
                        z: 15,
                        loot_table: "minecraft:chests/ancient_city",
                    },
                ],
                markers: &[],
                randomizable_containers: &[
                    TemplateBlockPos { x: 4, y: 4, z: 15 },
                    TemplateBlockPos { x: 12, y: 4, z: 15 },
                ],
            },
        );
        map.insert(
            "ancient_city/structures/chamber_1",
            TemplateContainerData {
                size: [19, 10, 15],
                chests: &[TemplateChest {
                    x: 14,
                    y: 3,
                    z: 11,
                    loot_table: "minecraft:chests/ancient_city",
                }],
                markers: &[],
                randomizable_containers: &[TemplateBlockPos { x: 14, y: 3, z: 11 }],
            },
        );
        map.insert(
            "ancient_city/structures/chamber_2",
            TemplateContainerData {
                size: [12, 6, 11],
                chests: &[TemplateChest {
                    x: 10,
                    y: 2,
                    z: 5,
                    loot_table: "minecraft:chests/ancient_city",
                }],
                markers: &[],
                randomizable_containers: &[TemplateBlockPos { x: 10, y: 2, z: 5 }],
            },
        );
        map.insert(
            "ancient_city/structures/chamber_3",
            TemplateContainerData {
                size: [10, 6, 11],
                chests: &[TemplateChest {
                    x: 7,
                    y: 1,
                    z: 5,
                    loot_table: "minecraft:chests/ancient_city",
                }],
                markers: &[],
                randomizable_containers: &[TemplateBlockPos { x: 7, y: 1, z: 5 }],
            },
        );
        map.insert(
            "ancient_city/structures/ice_box_1",
            TemplateContainerData {
                size: [19, 10, 15],
                chests: &[TemplateChest {
                    x: 12,
                    y: 1,
                    z: 7,
                    loot_table: "minecraft:chests/ancient_city_ice_box",
                }],
                markers: &[],
                randomizable_containers: &[TemplateBlockPos { x: 12, y: 1, z: 7 }],
            },
        );
        map.insert(
            "ancient_city/structures/sauna_1",
            TemplateContainerData {
                size: [29, 10, 37],
                chests: &[
                    TemplateChest {
                        x: 12,
                        y: 1,
                        z: 19,
                        loot_table: "minecraft:chests/ancient_city",
                    },
                    TemplateChest {
                        x: 21,
                        y: 1,
                        z: 18,
                        loot_table: "minecraft:chests/ancient_city",
                    },
                    TemplateChest {
                        x: 25,
                        y: 1,
                        z: 10,
                        loot_table: "minecraft:chests/ancient_city",
                    },
                ],
                markers: &[],
                randomizable_containers: &[
                    TemplateBlockPos { x: 12, y: 1, z: 19 },
                    TemplateBlockPos { x: 21, y: 1, z: 18 },
                    TemplateBlockPos { x: 25, y: 1, z: 10 },
                ],
            },
        );
        map.insert(
            "ancient_city/structures/tall_ruin_1",
            TemplateContainerData {
                size: [17, 23, 17],
                chests: &[TemplateChest {
                    x: 8,
                    y: 16,
                    z: 8,
                    loot_table: "minecraft:chests/ancient_city",
                }],
                markers: &[],
                randomizable_containers: &[TemplateBlockPos { x: 8, y: 16, z: 8 }],
            },
        );
        map.insert(
            "ancient_city/structures/tall_ruin_2",
            TemplateContainerData {
                size: [17, 23, 17],
                chests: &[
                    TemplateChest {
                        x: 7,
                        y: 1,
                        z: 5,
                        loot_table: "minecraft:chests/ancient_city",
                    },
                    TemplateChest {
                        x: 13,
                        y: 7,
                        z: 9,
                        loot_table: "minecraft:chests/ancient_city",
                    },
                ],
                markers: &[],
                randomizable_containers: &[
                    TemplateBlockPos { x: 7, y: 1, z: 5 },
                    TemplateBlockPos { x: 13, y: 7, z: 9 },
                ],
            },
        );
        map.insert(
            "ancient_city/structures/tall_ruin_3",
            TemplateContainerData {
                size: [17, 23, 17],
                chests: &[TemplateChest {
                    x: 10,
                    y: 14,
                    z: 7,
                    loot_table: "minecraft:chests/ancient_city",
                }],
                markers: &[],
                randomizable_containers: &[TemplateBlockPos { x: 10, y: 14, z: 7 }],
            },
        );
        map.insert(
            "ancient_city/structures/tall_ruin_4",
            TemplateContainerData {
                size: [17, 23, 17],
                chests: &[TemplateChest {
                    x: 8,
                    y: 7,
                    z: 13,
                    loot_table: "minecraft:chests/ancient_city",
                }],
                markers: &[],
                randomizable_containers: &[TemplateBlockPos { x: 8, y: 7, z: 13 }],
            },
        );
        map.insert(
            "bastion/bridge/ramparts/rampart_0",
            TemplateContainerData {
                size: [16, 22, 16],
                chests: &[TemplateChest {
                    x: 3,
                    y: 13,
                    z: 7,
                    loot_table: "minecraft:chests/bastion_other",
                }],
                markers: &[],
                randomizable_containers: &[TemplateBlockPos { x: 3, y: 13, z: 7 }],
            },
        );
        map.insert(
            "bastion/bridge/ramparts/rampart_1",
            TemplateContainerData {
                size: [16, 32, 16],
                chests: &[
                    TemplateChest {
                        x: 7,
                        y: 17,
                        z: 7,
                        loot_table: "minecraft:chests/bastion_other",
                    },
                    TemplateChest {
                        x: 7,
                        y: 17,
                        z: 8,
                        loot_table: "minecraft:chests/bastion_other",
                    },
                    TemplateChest {
                        x: 7,
                        y: 17,
                        z: 12,
                        loot_table: "minecraft:chests/bastion_other",
                    },
                ],
                markers: &[],
                randomizable_containers: &[
                    TemplateBlockPos { x: 7, y: 17, z: 7 },
                    TemplateBlockPos { x: 7, y: 17, z: 8 },
                    TemplateBlockPos { x: 7, y: 17, z: 12 },
                ],
            },
        );
        map.insert(
            "bastion/bridge/starting_pieces/entrance",
            TemplateContainerData {
                size: [17, 32, 32],
                chests: &[TemplateChest {
                    x: 9,
                    y: 16,
                    z: 4,
                    loot_table: "minecraft:chests/bastion_bridge",
                }],
                markers: &[],
                randomizable_containers: &[TemplateBlockPos { x: 9, y: 16, z: 4 }],
            },
        );
        map.insert(
            "bastion/hoglin_stable/large_stables/inner_3",
            TemplateContainerData {
                size: [14, 6, 8],
                chests: &[TemplateChest {
                    x: 1,
                    y: 1,
                    z: 5,
                    loot_table: "minecraft:chests/bastion_hoglin_stable",
                }],
                markers: &[],
                randomizable_containers: &[TemplateBlockPos { x: 1, y: 1, z: 5 }],
            },
        );
        map.insert(
            "bastion/hoglin_stable/ramparts/ramparts_1",
            TemplateContainerData {
                size: [16, 32, 16],
                chests: &[
                    TemplateChest {
                        x: 7,
                        y: 17,
                        z: 7,
                        loot_table: "minecraft:chests/bastion_other",
                    },
                    TemplateChest {
                        x: 7,
                        y: 17,
                        z: 8,
                        loot_table: "minecraft:chests/bastion_other",
                    },
                    TemplateChest {
                        x: 7,
                        y: 17,
                        z: 11,
                        loot_table: "minecraft:chests/bastion_other",
                    },
                ],
                markers: &[],
                randomizable_containers: &[
                    TemplateBlockPos { x: 7, y: 17, z: 7 },
                    TemplateBlockPos { x: 7, y: 17, z: 8 },
                    TemplateBlockPos { x: 7, y: 17, z: 11 },
                ],
            },
        );
        map.insert(
            "bastion/hoglin_stable/ramparts/ramparts_2",
            TemplateContainerData {
                size: [16, 21, 16],
                chests: &[
                    TemplateChest {
                        x: 12,
                        y: 3,
                        z: 13,
                        loot_table: "minecraft:chests/bastion_other",
                    },
                    TemplateChest {
                        x: 3,
                        y: 13,
                        z: 9,
                        loot_table: "minecraft:chests/bastion_other",
                    },
                ],
                markers: &[],
                randomizable_containers: &[
                    TemplateBlockPos { x: 12, y: 3, z: 13 },
                    TemplateBlockPos { x: 3, y: 13, z: 9 },
                ],
            },
        );
        map.insert(
            "bastion/hoglin_stable/ramparts/ramparts_3",
            TemplateContainerData {
                size: [16, 12, 16],
                chests: &[TemplateChest {
                    x: 6,
                    y: 3,
                    z: 4,
                    loot_table: "minecraft:chests/bastion_other",
                }],
                markers: &[],
                randomizable_containers: &[TemplateBlockPos { x: 6, y: 3, z: 4 }],
            },
        );
        map.insert(
            "bastion/hoglin_stable/small_stables/inner_2",
            TemplateContainerData {
                size: [12, 6, 8],
                chests: &[TemplateChest {
                    x: 3,
                    y: 1,
                    z: 6,
                    loot_table: "minecraft:chests/bastion_hoglin_stable",
                }],
                markers: &[],
                randomizable_containers: &[TemplateBlockPos { x: 3, y: 1, z: 6 }],
            },
        );
        map.insert(
            "bastion/hoglin_stable/walls/side_wall_0",
            TemplateContainerData {
                size: [16, 24, 16],
                chests: &[TemplateChest {
                    x: 9,
                    y: 7,
                    z: 9,
                    loot_table: "minecraft:chests/bastion_other",
                }],
                markers: &[],
                randomizable_containers: &[TemplateBlockPos { x: 9, y: 7, z: 9 }],
            },
        );
        map.insert(
            "bastion/hoglin_stable/walls/wall_base",
            TemplateContainerData {
                size: [16, 24, 16],
                chests: &[TemplateChest {
                    x: 12,
                    y: 4,
                    z: 13,
                    loot_table: "minecraft:chests/bastion_other",
                }],
                markers: &[],
                randomizable_containers: &[TemplateBlockPos { x: 12, y: 4, z: 13 }],
            },
        );
        map.insert(
            "bastion/treasure/bases/centers/center_0",
            TemplateContainerData {
                size: [7, 6, 8],
                chests: &[TemplateChest {
                    x: 4,
                    y: 3,
                    z: 6,
                    loot_table: "minecraft:chests/bastion_treasure",
                }],
                markers: &[],
                randomizable_containers: &[TemplateBlockPos { x: 4, y: 3, z: 6 }],
            },
        );
        map.insert(
            "bastion/treasure/bases/centers/center_1",
            TemplateContainerData {
                size: [7, 6, 8],
                chests: &[
                    TemplateChest {
                        x: 2,
                        y: 3,
                        z: 6,
                        loot_table: "minecraft:chests/bastion_treasure",
                    },
                    TemplateChest {
                        x: 4,
                        y: 3,
                        z: 6,
                        loot_table: "minecraft:chests/bastion_treasure",
                    },
                ],
                markers: &[],
                randomizable_containers: &[
                    TemplateBlockPos { x: 2, y: 3, z: 6 },
                    TemplateBlockPos { x: 4, y: 3, z: 6 },
                ],
            },
        );
        map.insert(
            "bastion/treasure/bases/centers/center_2",
            TemplateContainerData {
                size: [7, 6, 8],
                chests: &[TemplateChest {
                    x: 1,
                    y: 2,
                    z: 4,
                    loot_table: "minecraft:chests/bastion_treasure",
                }],
                markers: &[],
                randomizable_containers: &[TemplateBlockPos { x: 1, y: 2, z: 4 }],
            },
        );
        map.insert(
            "bastion/treasure/bases/centers/center_3",
            TemplateContainerData {
                size: [7, 6, 8],
                chests: &[TemplateChest {
                    x: 3,
                    y: 3,
                    z: 5,
                    loot_table: "minecraft:chests/bastion_treasure",
                }],
                markers: &[],
                randomizable_containers: &[TemplateBlockPos { x: 3, y: 3, z: 5 }],
            },
        );
        map.insert(
            "bastion/treasure/ramparts/mid_wall_main",
            TemplateContainerData {
                size: [17, 32, 14],
                chests: &[
                    TemplateChest {
                        x: 11,
                        y: 29,
                        z: 6,
                        loot_table: "minecraft:chests/bastion_other",
                    },
                    TemplateChest {
                        x: 13,
                        y: 29,
                        z: 8,
                        loot_table: "minecraft:chests/bastion_other",
                    },
                ],
                markers: &[],
                randomizable_containers: &[
                    TemplateBlockPos { x: 11, y: 29, z: 6 },
                    TemplateBlockPos { x: 13, y: 29, z: 8 },
                ],
            },
        );
        map.insert(
            "bastion/treasure/ramparts/mid_wall_side",
            TemplateContainerData {
                size: [17, 31, 14],
                chests: &[
                    TemplateChest {
                        x: 11,
                        y: 29,
                        z: 6,
                        loot_table: "minecraft:chests/bastion_other",
                    },
                    TemplateChest {
                        x: 11,
                        y: 29,
                        z: 7,
                        loot_table: "minecraft:chests/bastion_other",
                    },
                ],
                markers: &[],
                randomizable_containers: &[
                    TemplateBlockPos { x: 11, y: 29, z: 6 },
                    TemplateBlockPos { x: 11, y: 29, z: 7 },
                ],
            },
        );
        map.insert(
            "bastion/treasure/walls/bottom/wall_0",
            TemplateContainerData {
                size: [5, 16, 24],
                chests: &[TemplateChest {
                    x: 3,
                    y: 6,
                    z: 2,
                    loot_table: "minecraft:chests/bastion_other",
                }],
                markers: &[],
                randomizable_containers: &[TemplateBlockPos { x: 3, y: 6, z: 2 }],
            },
        );
        map.insert(
            "bastion/treasure/walls/mid/wall_0",
            TemplateContainerData {
                size: [5, 15, 24],
                chests: &[TemplateChest {
                    x: 3,
                    y: 11,
                    z: 22,
                    loot_table: "minecraft:chests/bastion_other",
                }],
                markers: &[],
                randomizable_containers: &[TemplateBlockPos { x: 3, y: 11, z: 22 }],
            },
        );
        map.insert(
            "bastion/units/center_pieces/center_0",
            TemplateContainerData {
                size: [11, 7, 11],
                chests: &[TemplateChest {
                    x: 8,
                    y: 1,
                    z: 3,
                    loot_table: "minecraft:chests/bastion_other",
                }],
                markers: &[],
                randomizable_containers: &[TemplateBlockPos { x: 8, y: 1, z: 3 }],
            },
        );
        map.insert(
            "bastion/units/center_pieces/center_1",
            TemplateContainerData {
                size: [11, 8, 11],
                chests: &[TemplateChest {
                    x: 6,
                    y: 1,
                    z: 9,
                    loot_table: "minecraft:chests/bastion_other",
                }],
                markers: &[],
                randomizable_containers: &[TemplateBlockPos { x: 6, y: 1, z: 9 }],
            },
        );
        map.insert(
            "bastion/units/center_pieces/center_2",
            TemplateContainerData {
                size: [11, 8, 11],
                chests: &[TemplateChest {
                    x: 1,
                    y: 1,
                    z: 7,
                    loot_table: "minecraft:chests/bastion_other",
                }],
                markers: &[],
                randomizable_containers: &[TemplateBlockPos { x: 1, y: 1, z: 7 }],
            },
        );
        map.insert(
            "bastion/units/ramparts/ramparts_0",
            TemplateContainerData {
                size: [16, 32, 16],
                chests: &[
                    TemplateChest {
                        x: 7,
                        y: 17,
                        z: 7,
                        loot_table: "minecraft:chests/bastion_other",
                    },
                    TemplateChest {
                        x: 7,
                        y: 17,
                        z: 8,
                        loot_table: "minecraft:chests/bastion_other",
                    },
                    TemplateChest {
                        x: 7,
                        y: 17,
                        z: 12,
                        loot_table: "minecraft:chests/bastion_other",
                    },
                ],
                markers: &[],
                randomizable_containers: &[
                    TemplateBlockPos { x: 7, y: 17, z: 7 },
                    TemplateBlockPos { x: 7, y: 17, z: 8 },
                    TemplateBlockPos { x: 7, y: 17, z: 12 },
                ],
            },
        );
        map.insert(
            "bastion/units/ramparts/ramparts_1",
            TemplateContainerData {
                size: [16, 22, 16],
                chests: &[TemplateChest {
                    x: 3,
                    y: 13,
                    z: 6,
                    loot_table: "minecraft:chests/bastion_other",
                }],
                markers: &[],
                randomizable_containers: &[TemplateBlockPos { x: 3, y: 13, z: 6 }],
            },
        );
        map.insert(
            "bastion/units/stages/stage_0_2",
            TemplateContainerData {
                size: [12, 7, 8],
                chests: &[TemplateChest {
                    x: 1,
                    y: 4,
                    z: 6,
                    loot_table: "minecraft:chests/bastion_other",
                }],
                markers: &[],
                randomizable_containers: &[TemplateBlockPos { x: 1, y: 4, z: 6 }],
            },
        );
        map.insert(
            "bastion/units/stages/stage_1_2",
            TemplateContainerData {
                size: [12, 7, 8],
                chests: &[TemplateChest {
                    x: 7,
                    y: 1,
                    z: 6,
                    loot_table: "minecraft:chests/bastion_other",
                }],
                markers: &[],
                randomizable_containers: &[TemplateBlockPos { x: 7, y: 1, z: 6 }],
            },
        );
        map.insert(
            "bastion/units/walls/wall_base",
            TemplateContainerData {
                size: [16, 24, 16],
                chests: &[
                    TemplateChest {
                        x: 2,
                        y: 4,
                        z: 5,
                        loot_table: "minecraft:chests/bastion_other",
                    },
                    TemplateChest {
                        x: 3,
                        y: 4,
                        z: 5,
                        loot_table: "minecraft:chests/bastion_other",
                    },
                ],
                markers: &[],
                randomizable_containers: &[
                    TemplateBlockPos { x: 2, y: 4, z: 5 },
                    TemplateBlockPos { x: 3, y: 4, z: 5 },
                ],
            },
        );
        map.insert(
            "end_city/base_floor",
            TemplateContainerData {
                size: [10, 4, 10],
                chests: &[],
                markers: &[
                    TemplateMarker {
                        x: 3,
                        y: 2,
                        z: 9,
                        metadata: "Sentry",
                    },
                    TemplateMarker {
                        x: 6,
                        y: 2,
                        z: 9,
                        metadata: "Sentry",
                    },
                ],
                randomizable_containers: &[],
            },
        );
        map.insert(
            "end_city/fat_tower_middle",
            TemplateContainerData {
                size: [13, 8, 13],
                chests: &[],
                markers: &[
                    TemplateMarker {
                        x: 2,
                        y: 2,
                        z: 6,
                        metadata: "Sentry",
                    },
                    TemplateMarker {
                        x: 10,
                        y: 2,
                        z: 6,
                        metadata: "Sentry",
                    },
                    TemplateMarker {
                        x: 6,
                        y: 6,
                        z: 2,
                        metadata: "Sentry",
                    },
                    TemplateMarker {
                        x: 6,
                        y: 6,
                        z: 10,
                        metadata: "Sentry",
                    },
                ],
                randomizable_containers: &[],
            },
        );
        map.insert(
            "end_city/fat_tower_top",
            TemplateContainerData {
                size: [17, 6, 17],
                chests: &[],
                markers: &[
                    TemplateMarker {
                        x: 3,
                        y: 2,
                        z: 11,
                        metadata: "Chest",
                    },
                    TemplateMarker {
                        x: 5,
                        y: 2,
                        z: 13,
                        metadata: "Chest",
                    },
                ],
                randomizable_containers: &[
                    TemplateBlockPos { x: 3, y: 1, z: 11 },
                    TemplateBlockPos { x: 5, y: 1, z: 13 },
                ],
            },
        );
        map.insert(
            "end_city/second_floor_2",
            TemplateContainerData {
                size: [12, 8, 12],
                chests: &[],
                markers: &[TemplateMarker {
                    x: 8,
                    y: 5,
                    z: 6,
                    metadata: "Sentry",
                }],
                randomizable_containers: &[],
            },
        );
        map.insert(
            "end_city/ship",
            TemplateContainerData {
                size: [13, 24, 29],
                chests: &[],
                markers: &[
                    TemplateMarker {
                        x: 6,
                        y: 4,
                        z: 8,
                        metadata: "Sentry",
                    },
                    TemplateMarker {
                        x: 5,
                        y: 5,
                        z: 7,
                        metadata: "Chest",
                    },
                    TemplateMarker {
                        x: 6,
                        y: 5,
                        z: 7,
                        metadata: "Elytra",
                    },
                    TemplateMarker {
                        x: 7,
                        y: 5,
                        z: 7,
                        metadata: "Chest",
                    },
                    TemplateMarker {
                        x: 8,
                        y: 6,
                        z: 27,
                        metadata: "Sentry",
                    },
                    TemplateMarker {
                        x: 4,
                        y: 11,
                        z: 27,
                        metadata: "Sentry",
                    },
                ],
                randomizable_containers: &[
                    TemplateBlockPos { x: 5, y: 4, z: 7 },
                    TemplateBlockPos { x: 7, y: 4, z: 7 },
                ],
            },
        );
        map.insert(
            "end_city/third_floor_2",
            TemplateContainerData {
                size: [14, 8, 14],
                chests: &[],
                markers: &[
                    TemplateMarker {
                        x: 2,
                        y: 5,
                        z: 2,
                        metadata: "Sentry",
                    },
                    TemplateMarker {
                        x: 11,
                        y: 5,
                        z: 2,
                        metadata: "Sentry",
                    },
                    TemplateMarker {
                        x: 6,
                        y: 6,
                        z: 2,
                        metadata: "Chest",
                    },
                ],
                randomizable_containers: &[TemplateBlockPos { x: 6, y: 5, z: 2 }],
            },
        );
        map.insert(
            "end_city/tower_top",
            TemplateContainerData {
                size: [9, 5, 9],
                chests: &[],
                markers: &[TemplateMarker {
                    x: 4,
                    y: 3,
                    z: 4,
                    metadata: "Sentry",
                }],
                randomizable_containers: &[],
            },
        );
        map.insert(
            "igloo/bottom",
            TemplateContainerData {
                size: [7, 6, 9],
                chests: &[],
                markers: &[TemplateMarker {
                    x: 1,
                    y: 2,
                    z: 6,
                    metadata: "chest",
                }],
                randomizable_containers: &[TemplateBlockPos { x: 1, y: 1, z: 6 }],
            },
        );
        map.insert(
            "pillager_outpost/watchtower",
            TemplateContainerData {
                size: [15, 21, 15],
                chests: &[TemplateChest {
                    x: 9,
                    y: 14,
                    z: 10,
                    loot_table: "minecraft:chests/pillager_outpost",
                }],
                markers: &[],
                randomizable_containers: &[TemplateBlockPos { x: 9, y: 14, z: 10 }],
            },
        );
        map.insert(
            "pillager_outpost/watchtower_overgrown",
            TemplateContainerData {
                size: [15, 23, 15],
                chests: &[TemplateChest {
                    x: 9,
                    y: 14,
                    z: 10,
                    loot_table: "minecraft:chests/pillager_outpost",
                }],
                markers: &[],
                randomizable_containers: &[TemplateBlockPos { x: 9, y: 14, z: 10 }],
            },
        );
        map.insert(
            "ruined_portal/giant_portal_1",
            TemplateContainerData {
                size: [11, 17, 16],
                chests: &[TemplateChest {
                    x: 4,
                    y: 3,
                    z: 3,
                    loot_table: "minecraft:chests/ruined_portal",
                }],
                markers: &[],
                randomizable_containers: &[TemplateBlockPos { x: 4, y: 3, z: 3 }],
            },
        );
        map.insert(
            "ruined_portal/giant_portal_2",
            TemplateContainerData {
                size: [11, 16, 16],
                chests: &[TemplateChest {
                    x: 9,
                    y: 1,
                    z: 9,
                    loot_table: "minecraft:chests/ruined_portal",
                }],
                markers: &[],
                randomizable_containers: &[TemplateBlockPos { x: 9, y: 1, z: 9 }],
            },
        );
        map.insert(
            "ruined_portal/giant_portal_3",
            TemplateContainerData {
                size: [16, 16, 16],
                chests: &[TemplateChest {
                    x: 9,
                    y: 2,
                    z: 3,
                    loot_table: "minecraft:chests/ruined_portal",
                }],
                markers: &[],
                randomizable_containers: &[TemplateBlockPos { x: 9, y: 2, z: 3 }],
            },
        );
        map.insert(
            "ruined_portal/portal_1",
            TemplateContainerData {
                size: [6, 10, 6],
                chests: &[TemplateChest {
                    x: 2,
                    y: 2,
                    z: 0,
                    loot_table: "minecraft:chests/ruined_portal",
                }],
                markers: &[],
                randomizable_containers: &[TemplateBlockPos { x: 2, y: 2, z: 0 }],
            },
        );
        map.insert(
            "ruined_portal/portal_10",
            TemplateContainerData {
                size: [12, 8, 10],
                chests: &[TemplateChest {
                    x: 2,
                    y: 1,
                    z: 7,
                    loot_table: "minecraft:chests/ruined_portal",
                }],
                markers: &[],
                randomizable_containers: &[TemplateBlockPos { x: 2, y: 1, z: 7 }],
            },
        );
        map.insert(
            "ruined_portal/portal_2",
            TemplateContainerData {
                size: [9, 12, 9],
                chests: &[TemplateChest {
                    x: 8,
                    y: 2,
                    z: 6,
                    loot_table: "minecraft:chests/ruined_portal",
                }],
                markers: &[],
                randomizable_containers: &[TemplateBlockPos { x: 8, y: 2, z: 6 }],
            },
        );
        map.insert(
            "ruined_portal/portal_3",
            TemplateContainerData {
                size: [8, 9, 9],
                chests: &[TemplateChest {
                    x: 3,
                    y: 3,
                    z: 6,
                    loot_table: "minecraft:chests/ruined_portal",
                }],
                markers: &[],
                randomizable_containers: &[TemplateBlockPos { x: 3, y: 3, z: 6 }],
            },
        );
        map.insert(
            "ruined_portal/portal_4",
            TemplateContainerData {
                size: [8, 9, 9],
                chests: &[TemplateChest {
                    x: 3,
                    y: 3,
                    z: 2,
                    loot_table: "minecraft:chests/ruined_portal",
                }],
                markers: &[],
                randomizable_containers: &[TemplateBlockPos { x: 3, y: 3, z: 2 }],
            },
        );
        map.insert(
            "ruined_portal/portal_5",
            TemplateContainerData {
                size: [10, 10, 7],
                chests: &[TemplateChest {
                    x: 4,
                    y: 3,
                    z: 2,
                    loot_table: "minecraft:chests/ruined_portal",
                }],
                markers: &[],
                randomizable_containers: &[TemplateBlockPos { x: 4, y: 3, z: 2 }],
            },
        );
        map.insert(
            "ruined_portal/portal_6",
            TemplateContainerData {
                size: [5, 7, 7],
                chests: &[TemplateChest {
                    x: 1,
                    y: 1,
                    z: 4,
                    loot_table: "minecraft:chests/ruined_portal",
                }],
                markers: &[],
                randomizable_containers: &[TemplateBlockPos { x: 1, y: 1, z: 4 }],
            },
        );
        map.insert(
            "ruined_portal/portal_7",
            TemplateContainerData {
                size: [9, 7, 9],
                chests: &[TemplateChest {
                    x: 0,
                    y: 1,
                    z: 2,
                    loot_table: "minecraft:chests/ruined_portal",
                }],
                markers: &[],
                randomizable_containers: &[TemplateBlockPos { x: 0, y: 1, z: 2 }],
            },
        );
        map.insert(
            "ruined_portal/portal_8",
            TemplateContainerData {
                size: [14, 9, 9],
                chests: &[TemplateChest {
                    x: 4,
                    y: 4,
                    z: 2,
                    loot_table: "minecraft:chests/ruined_portal",
                }],
                markers: &[],
                randomizable_containers: &[TemplateBlockPos { x: 4, y: 4, z: 2 }],
            },
        );
        map.insert(
            "ruined_portal/portal_9",
            TemplateContainerData {
                size: [10, 8, 9],
                chests: &[TemplateChest {
                    x: 4,
                    y: 1,
                    z: 0,
                    loot_table: "minecraft:chests/ruined_portal",
                }],
                markers: &[],
                randomizable_containers: &[TemplateBlockPos { x: 4, y: 1, z: 0 }],
            },
        );
        map.insert(
            "shipwreck/rightsideup_backhalf",
            TemplateContainerData {
                size: [9, 9, 16],
                chests: &[],
                markers: &[
                    TemplateMarker {
                        x: 5,
                        y: 3,
                        z: 6,
                        metadata: "map_chest",
                    },
                    TemplateMarker {
                        x: 6,
                        y: 5,
                        z: 12,
                        metadata: "treasure_chest",
                    },
                ],
                randomizable_containers: &[
                    TemplateBlockPos { x: 5, y: 2, z: 6 },
                    TemplateBlockPos { x: 6, y: 4, z: 12 },
                ],
            },
        );
        map.insert(
            "shipwreck/rightsideup_backhalf_degraded",
            TemplateContainerData {
                size: [9, 9, 16],
                chests: &[],
                markers: &[
                    TemplateMarker {
                        x: 5,
                        y: 3,
                        z: 6,
                        metadata: "map_chest",
                    },
                    TemplateMarker {
                        x: 6,
                        y: 5,
                        z: 12,
                        metadata: "treasure_chest",
                    },
                ],
                randomizable_containers: &[
                    TemplateBlockPos { x: 5, y: 2, z: 6 },
                    TemplateBlockPos { x: 6, y: 4, z: 12 },
                ],
            },
        );
        map.insert(
            "shipwreck/rightsideup_fronthalf",
            TemplateContainerData {
                size: [9, 9, 24],
                chests: &[],
                markers: &[TemplateMarker {
                    x: 4,
                    y: 3,
                    z: 8,
                    metadata: "supply_chest",
                }],
                randomizable_containers: &[TemplateBlockPos { x: 4, y: 2, z: 8 }],
            },
        );
        map.insert(
            "shipwreck/rightsideup_fronthalf_degraded",
            TemplateContainerData {
                size: [9, 9, 24],
                chests: &[],
                markers: &[TemplateMarker {
                    x: 4,
                    y: 3,
                    z: 8,
                    metadata: "supply_chest",
                }],
                randomizable_containers: &[TemplateBlockPos { x: 4, y: 2, z: 8 }],
            },
        );
        map.insert(
            "shipwreck/rightsideup_full",
            TemplateContainerData {
                size: [9, 9, 28],
                chests: &[],
                markers: &[
                    TemplateMarker {
                        x: 4,
                        y: 3,
                        z: 8,
                        metadata: "supply_chest",
                    },
                    TemplateMarker {
                        x: 5,
                        y: 3,
                        z: 18,
                        metadata: "map_chest",
                    },
                    TemplateMarker {
                        x: 6,
                        y: 5,
                        z: 24,
                        metadata: "treasure_chest",
                    },
                ],
                randomizable_containers: &[
                    TemplateBlockPos { x: 4, y: 2, z: 8 },
                    TemplateBlockPos { x: 5, y: 2, z: 18 },
                    TemplateBlockPos { x: 6, y: 4, z: 24 },
                ],
            },
        );
        map.insert(
            "shipwreck/rightsideup_full_degraded",
            TemplateContainerData {
                size: [9, 9, 28],
                chests: &[],
                markers: &[
                    TemplateMarker {
                        x: 4,
                        y: 3,
                        z: 8,
                        metadata: "supply_chest",
                    },
                    TemplateMarker {
                        x: 5,
                        y: 3,
                        z: 18,
                        metadata: "map_chest",
                    },
                    TemplateMarker {
                        x: 6,
                        y: 5,
                        z: 24,
                        metadata: "treasure_chest",
                    },
                ],
                randomizable_containers: &[
                    TemplateBlockPos { x: 4, y: 2, z: 8 },
                    TemplateBlockPos { x: 5, y: 2, z: 18 },
                    TemplateBlockPos { x: 6, y: 4, z: 24 },
                ],
            },
        );
        map.insert(
            "shipwreck/sideways_backhalf",
            TemplateContainerData {
                size: [9, 9, 17],
                chests: &[],
                markers: &[
                    TemplateMarker {
                        x: 3,
                        y: 3,
                        z: 13,
                        metadata: "treasure_chest",
                    },
                    TemplateMarker {
                        x: 6,
                        y: 4,
                        z: 8,
                        metadata: "map_chest",
                    },
                ],
                randomizable_containers: &[
                    TemplateBlockPos { x: 3, y: 2, z: 13 },
                    TemplateBlockPos { x: 6, y: 3, z: 8 },
                ],
            },
        );
        map.insert(
            "shipwreck/sideways_backhalf_degraded",
            TemplateContainerData {
                size: [9, 9, 17],
                chests: &[],
                markers: &[
                    TemplateMarker {
                        x: 3,
                        y: 3,
                        z: 13,
                        metadata: "treasure_chest",
                    },
                    TemplateMarker {
                        x: 6,
                        y: 4,
                        z: 8,
                        metadata: "map_chest",
                    },
                ],
                randomizable_containers: &[
                    TemplateBlockPos { x: 3, y: 2, z: 13 },
                    TemplateBlockPos { x: 6, y: 3, z: 8 },
                ],
            },
        );
        map.insert(
            "shipwreck/sideways_fronthalf",
            TemplateContainerData {
                size: [9, 9, 24],
                chests: &[],
                markers: &[TemplateMarker {
                    x: 5,
                    y: 4,
                    z: 8,
                    metadata: "supply_chest",
                }],
                randomizable_containers: &[TemplateBlockPos { x: 5, y: 3, z: 8 }],
            },
        );
        map.insert(
            "shipwreck/sideways_fronthalf_degraded",
            TemplateContainerData {
                size: [9, 9, 24],
                chests: &[],
                markers: &[TemplateMarker {
                    x: 5,
                    y: 4,
                    z: 8,
                    metadata: "supply_chest",
                }],
                randomizable_containers: &[TemplateBlockPos { x: 5, y: 3, z: 8 }],
            },
        );
        map.insert(
            "shipwreck/sideways_full",
            TemplateContainerData {
                size: [9, 9, 28],
                chests: &[],
                markers: &[
                    TemplateMarker {
                        x: 3,
                        y: 3,
                        z: 24,
                        metadata: "treasure_chest",
                    },
                    TemplateMarker {
                        x: 5,
                        y: 4,
                        z: 8,
                        metadata: "supply_chest",
                    },
                    TemplateMarker {
                        x: 6,
                        y: 4,
                        z: 19,
                        metadata: "map_chest",
                    },
                ],
                randomizable_containers: &[
                    TemplateBlockPos { x: 3, y: 2, z: 24 },
                    TemplateBlockPos { x: 5, y: 3, z: 8 },
                    TemplateBlockPos { x: 6, y: 3, z: 19 },
                ],
            },
        );
        map.insert(
            "shipwreck/sideways_full_degraded",
            TemplateContainerData {
                size: [9, 9, 28],
                chests: &[],
                markers: &[
                    TemplateMarker {
                        x: 3,
                        y: 3,
                        z: 24,
                        metadata: "treasure_chest",
                    },
                    TemplateMarker {
                        x: 5,
                        y: 4,
                        z: 8,
                        metadata: "supply_chest",
                    },
                    TemplateMarker {
                        x: 6,
                        y: 4,
                        z: 19,
                        metadata: "map_chest",
                    },
                ],
                randomizable_containers: &[
                    TemplateBlockPos { x: 3, y: 2, z: 24 },
                    TemplateBlockPos { x: 5, y: 3, z: 8 },
                    TemplateBlockPos { x: 6, y: 3, z: 19 },
                ],
            },
        );
        map.insert(
            "shipwreck/upsidedown_backhalf",
            TemplateContainerData {
                size: [9, 9, 16],
                chests: &[],
                markers: &[
                    TemplateMarker {
                        x: 2,
                        y: 3,
                        z: 12,
                        metadata: "treasure_chest",
                    },
                    TemplateMarker {
                        x: 3,
                        y: 6,
                        z: 5,
                        metadata: "map_chest",
                    },
                ],
                randomizable_containers: &[
                    TemplateBlockPos { x: 2, y: 2, z: 12 },
                    TemplateBlockPos { x: 3, y: 5, z: 5 },
                ],
            },
        );
        map.insert(
            "shipwreck/upsidedown_backhalf_degraded",
            TemplateContainerData {
                size: [9, 9, 16],
                chests: &[],
                markers: &[
                    TemplateMarker {
                        x: 2,
                        y: 3,
                        z: 12,
                        metadata: "treasure_chest",
                    },
                    TemplateMarker {
                        x: 3,
                        y: 6,
                        z: 5,
                        metadata: "map_chest",
                    },
                ],
                randomizable_containers: &[
                    TemplateBlockPos { x: 2, y: 2, z: 12 },
                    TemplateBlockPos { x: 3, y: 5, z: 5 },
                ],
            },
        );
        map.insert(
            "shipwreck/upsidedown_fronthalf",
            TemplateContainerData {
                size: [9, 9, 22],
                chests: &[],
                markers: &[
                    TemplateMarker {
                        x: 3,
                        y: 6,
                        z: 17,
                        metadata: "map_chest",
                    },
                    TemplateMarker {
                        x: 4,
                        y: 6,
                        z: 8,
                        metadata: "supply_chest",
                    },
                ],
                randomizable_containers: &[
                    TemplateBlockPos { x: 3, y: 5, z: 17 },
                    TemplateBlockPos { x: 4, y: 5, z: 8 },
                ],
            },
        );
        map.insert(
            "shipwreck/upsidedown_fronthalf_degraded",
            TemplateContainerData {
                size: [9, 9, 22],
                chests: &[],
                markers: &[
                    TemplateMarker {
                        x: 3,
                        y: 6,
                        z: 17,
                        metadata: "map_chest",
                    },
                    TemplateMarker {
                        x: 4,
                        y: 6,
                        z: 8,
                        metadata: "supply_chest",
                    },
                ],
                randomizable_containers: &[
                    TemplateBlockPos { x: 3, y: 5, z: 17 },
                    TemplateBlockPos { x: 4, y: 5, z: 8 },
                ],
            },
        );
        map.insert(
            "shipwreck/upsidedown_full",
            TemplateContainerData {
                size: [9, 9, 28],
                chests: &[],
                markers: &[
                    TemplateMarker {
                        x: 2,
                        y: 3,
                        z: 24,
                        metadata: "treasure_chest",
                    },
                    TemplateMarker {
                        x: 3,
                        y: 6,
                        z: 17,
                        metadata: "map_chest",
                    },
                    TemplateMarker {
                        x: 4,
                        y: 6,
                        z: 8,
                        metadata: "supply_chest",
                    },
                ],
                randomizable_containers: &[
                    TemplateBlockPos { x: 2, y: 2, z: 24 },
                    TemplateBlockPos { x: 3, y: 5, z: 17 },
                    TemplateBlockPos { x: 4, y: 5, z: 8 },
                ],
            },
        );
        map.insert(
            "shipwreck/upsidedown_full_degraded",
            TemplateContainerData {
                size: [9, 9, 28],
                chests: &[],
                markers: &[
                    TemplateMarker {
                        x: 2,
                        y: 3,
                        z: 24,
                        metadata: "treasure_chest",
                    },
                    TemplateMarker {
                        x: 3,
                        y: 6,
                        z: 17,
                        metadata: "map_chest",
                    },
                    TemplateMarker {
                        x: 4,
                        y: 6,
                        z: 8,
                        metadata: "supply_chest",
                    },
                ],
                randomizable_containers: &[
                    TemplateBlockPos { x: 2, y: 2, z: 24 },
                    TemplateBlockPos { x: 3, y: 5, z: 17 },
                    TemplateBlockPos { x: 4, y: 5, z: 8 },
                ],
            },
        );
        map.insert(
            "shipwreck/with_mast",
            TemplateContainerData {
                size: [9, 21, 28],
                chests: &[],
                markers: &[
                    TemplateMarker {
                        x: 4,
                        y: 3,
                        z: 9,
                        metadata: "supply_chest",
                    },
                    TemplateMarker {
                        x: 5,
                        y: 3,
                        z: 18,
                        metadata: "map_chest",
                    },
                    TemplateMarker {
                        x: 6,
                        y: 5,
                        z: 24,
                        metadata: "treasure_chest",
                    },
                ],
                randomizable_containers: &[
                    TemplateBlockPos { x: 4, y: 2, z: 9 },
                    TemplateBlockPos { x: 5, y: 2, z: 18 },
                    TemplateBlockPos { x: 6, y: 4, z: 24 },
                ],
            },
        );
        map.insert(
            "shipwreck/with_mast_degraded",
            TemplateContainerData {
                size: [9, 21, 28],
                chests: &[],
                markers: &[
                    TemplateMarker {
                        x: 4,
                        y: 3,
                        z: 9,
                        metadata: "supply_chest",
                    },
                    TemplateMarker {
                        x: 5,
                        y: 3,
                        z: 18,
                        metadata: "map_chest",
                    },
                    TemplateMarker {
                        x: 6,
                        y: 5,
                        z: 24,
                        metadata: "treasure_chest",
                    },
                ],
                randomizable_containers: &[
                    TemplateBlockPos { x: 4, y: 2, z: 9 },
                    TemplateBlockPos { x: 5, y: 2, z: 18 },
                    TemplateBlockPos { x: 6, y: 4, z: 24 },
                ],
            },
        );
        map.insert(
            "trial_chambers/chamber/eruption/center_1",
            TemplateContainerData {
                size: [5, 12, 5],
                chests: &[],
                markers: &[],
                randomizable_containers: &[TemplateBlockPos { x: 2, y: 6, z: 2 }],
            },
        );
        map.insert(
            "trial_chambers/chests/supply",
            TemplateContainerData {
                size: [3, 2, 3],
                chests: &[TemplateChest {
                    x: 1,
                    y: 1,
                    z: 1,
                    loot_table: "minecraft:chests/trial_chambers/supply",
                }],
                markers: &[],
                randomizable_containers: &[TemplateBlockPos { x: 1, y: 1, z: 1 }],
            },
        );
        map.insert(
            "trial_chambers/corridor/addon/arrow_dispenser",
            TemplateContainerData {
                size: [9, 2, 5],
                chests: &[],
                markers: &[],
                randomizable_containers: &[TemplateBlockPos { x: 1, y: 0, z: 2 }],
            },
        );
        map.insert(
            "trial_chambers/corridor/end_2",
            TemplateContainerData {
                size: [19, 20, 19],
                chests: &[TemplateChest {
                    x: 9,
                    y: 3,
                    z: 6,
                    loot_table: "minecraft:chests/trial_chambers/intersection",
                }],
                markers: &[],
                randomizable_containers: &[TemplateBlockPos { x: 9, y: 3, z: 6 }],
            },
        );
        map.insert(
            "trial_chambers/corridor/entrance_1",
            TemplateContainerData {
                size: [19, 20, 19],
                chests: &[
                    TemplateChest {
                        x: 6,
                        y: 3,
                        z: 7,
                        loot_table: "minecraft:chests/trial_chambers/reward",
                    },
                    TemplateChest {
                        x: 4,
                        y: 9,
                        z: 14,
                        loot_table: "minecraft:chests/trial_chambers/entrance",
                    },
                    TemplateChest {
                        x: 9,
                        y: 9,
                        z: 14,
                        loot_table: "minecraft:chests/trial_chambers/entrance",
                    },
                ],
                markers: &[],
                randomizable_containers: &[
                    TemplateBlockPos { x: 6, y: 3, z: 7 },
                    TemplateBlockPos { x: 4, y: 9, z: 14 },
                    TemplateBlockPos { x: 9, y: 9, z: 14 },
                ],
            },
        );
        map.insert(
            "trial_chambers/corridor/entrance_2",
            TemplateContainerData {
                size: [19, 20, 19],
                chests: &[
                    TemplateChest {
                        x: 4,
                        y: 9,
                        z: 14,
                        loot_table: "minecraft:chests/trial_chambers/entrance",
                    },
                    TemplateChest {
                        x: 9,
                        y: 9,
                        z: 14,
                        loot_table: "minecraft:chests/trial_chambers/entrance",
                    },
                ],
                markers: &[],
                randomizable_containers: &[
                    TemplateBlockPos { x: 4, y: 9, z: 14 },
                    TemplateBlockPos { x: 9, y: 9, z: 14 },
                ],
            },
        );
        map.insert(
            "trial_chambers/corridor/entrance_3",
            TemplateContainerData {
                size: [19, 22, 19],
                chests: &[
                    TemplateChest {
                        x: 4,
                        y: 2,
                        z: 4,
                        loot_table: "minecraft:chests/trial_chambers/entrance",
                    },
                    TemplateChest {
                        x: 4,
                        y: 2,
                        z: 14,
                        loot_table: "minecraft:chests/trial_chambers/entrance",
                    },
                    TemplateChest {
                        x: 4,
                        y: 7,
                        z: 14,
                        loot_table: "minecraft:chests/trial_chambers/entrance",
                    },
                ],
                markers: &[],
                randomizable_containers: &[
                    TemplateBlockPos { x: 4, y: 2, z: 4 },
                    TemplateBlockPos { x: 4, y: 2, z: 14 },
                    TemplateBlockPos { x: 4, y: 7, z: 14 },
                ],
            },
        );
        map.insert(
            "trial_chambers/decor/barrel",
            TemplateContainerData {
                size: [1, 2, 1],
                chests: &[TemplateChest {
                    x: 0,
                    y: 1,
                    z: 0,
                    loot_table: "minecraft:chests/trial_chambers/corridor",
                }],
                markers: &[],
                randomizable_containers: &[TemplateBlockPos { x: 0, y: 1, z: 0 }],
            },
        );
        map.insert(
            "trial_chambers/decor/disposal",
            TemplateContainerData {
                size: [1, 4, 3],
                chests: &[],
                markers: &[],
                randomizable_containers: &[
                    TemplateBlockPos { x: 0, y: 0, z: 1 },
                    TemplateBlockPos { x: 0, y: 1, z: 1 },
                ],
            },
        );
        map.insert(
            "trial_chambers/dispensers/chamber",
            TemplateContainerData {
                size: [2, 1, 3],
                chests: &[],
                markers: &[],
                randomizable_containers: &[TemplateBlockPos { x: 1, y: 0, z: 1 }],
            },
        );
        map.insert(
            "trial_chambers/dispensers/floor_dispenser",
            TemplateContainerData {
                size: [2, 2, 1],
                chests: &[],
                markers: &[],
                randomizable_containers: &[TemplateBlockPos { x: 1, y: 0, z: 0 }],
            },
        );
        map.insert(
            "trial_chambers/dispensers/wall_dispenser",
            TemplateContainerData {
                size: [2, 2, 3],
                chests: &[],
                markers: &[],
                randomizable_containers: &[TemplateBlockPos { x: 0, y: 0, z: 1 }],
            },
        );
        map.insert(
            "trial_chambers/hallway/encounter_4",
            TemplateContainerData {
                size: [14, 22, 19],
                chests: &[],
                markers: &[],
                randomizable_containers: &[
                    TemplateBlockPos { x: 1, y: 15, z: 13 },
                    TemplateBlockPos { x: 1, y: 15, z: 14 },
                ],
            },
        );
        map.insert(
            "trial_chambers/hallway/rubble",
            TemplateContainerData {
                size: [5, 7, 4],
                chests: &[TemplateChest {
                    x: 2,
                    y: 2,
                    z: 0,
                    loot_table: "minecraft:chests/trial_chambers/reward",
                }],
                markers: &[],
                randomizable_containers: &[TemplateBlockPos { x: 2, y: 2, z: 0 }],
            },
        );
        map.insert(
            "trial_chambers/hallway/rubble_chamber",
            TemplateContainerData {
                size: [5, 7, 4],
                chests: &[TemplateChest {
                    x: 2,
                    y: 2,
                    z: 0,
                    loot_table: "minecraft:chests/trial_chambers/reward",
                }],
                markers: &[],
                randomizable_containers: &[TemplateBlockPos { x: 2, y: 2, z: 0 }],
            },
        );
        map.insert(
            "trial_chambers/intersection/intersection_2",
            TemplateContainerData {
                size: [23, 20, 22],
                chests: &[TemplateChest {
                    x: 13,
                    y: 8,
                    z: 3,
                    loot_table: "minecraft:chests/trial_chambers/intersection_barrel",
                }],
                markers: &[],
                randomizable_containers: &[TemplateBlockPos { x: 13, y: 8, z: 3 }],
            },
        );
        map.insert(
            "trial_chambers/intersection/intersection_3",
            TemplateContainerData {
                size: [21, 37, 22],
                chests: &[TemplateChest {
                    x: 13,
                    y: 15,
                    z: 10,
                    loot_table: "minecraft:chests/trial_chambers/intersection_barrel",
                }],
                markers: &[],
                randomizable_containers: &[TemplateBlockPos {
                    x: 13,
                    y: 15,
                    z: 10,
                }],
            },
        );
        map.insert(
            "underwater_ruin/big_brick_1",
            TemplateContainerData {
                size: [16, 16, 16],
                chests: &[],
                markers: &[
                    TemplateMarker {
                        x: 5,
                        y: 0,
                        z: 4,
                        metadata: "chest",
                    },
                    TemplateMarker {
                        x: 6,
                        y: 1,
                        z: 4,
                        metadata: "drowned",
                    },
                    TemplateMarker {
                        x: 8,
                        y: 1,
                        z: 11,
                        metadata: "drowned",
                    },
                ],
                randomizable_containers: &[],
            },
        );
        map.insert(
            "underwater_ruin/big_brick_2",
            TemplateContainerData {
                size: [16, 16, 16],
                chests: &[],
                markers: &[
                    TemplateMarker {
                        x: 9,
                        y: 1,
                        z: 10,
                        metadata: "chest",
                    },
                    TemplateMarker {
                        x: 10,
                        y: 1,
                        z: 8,
                        metadata: "drowned",
                    },
                ],
                randomizable_containers: &[],
            },
        );
        map.insert(
            "underwater_ruin/big_brick_3",
            TemplateContainerData {
                size: [16, 16, 16],
                chests: &[],
                markers: &[
                    TemplateMarker {
                        x: 2,
                        y: 1,
                        z: 7,
                        metadata: "drowned",
                    },
                    TemplateMarker {
                        x: 6,
                        y: 1,
                        z: 9,
                        metadata: "drowned",
                    },
                    TemplateMarker {
                        x: 12,
                        y: 2,
                        z: 2,
                        metadata: "chest",
                    },
                ],
                randomizable_containers: &[],
            },
        );
        map.insert(
            "underwater_ruin/big_brick_8",
            TemplateContainerData {
                size: [16, 16, 16],
                chests: &[],
                markers: &[
                    TemplateMarker {
                        x: 5,
                        y: 0,
                        z: 4,
                        metadata: "chest",
                    },
                    TemplateMarker {
                        x: 3,
                        y: 1,
                        z: 8,
                        metadata: "drowned",
                    },
                    TemplateMarker {
                        x: 10,
                        y: 1,
                        z: 9,
                        metadata: "drowned",
                    },
                ],
                randomizable_containers: &[],
            },
        );
        map.insert(
            "underwater_ruin/big_cracked_1",
            TemplateContainerData {
                size: [16, 16, 16],
                chests: &[],
                markers: &[
                    TemplateMarker {
                        x: 5,
                        y: 0,
                        z: 4,
                        metadata: "chest",
                    },
                    TemplateMarker {
                        x: 12,
                        y: 1,
                        z: 11,
                        metadata: "drowned",
                    },
                ],
                randomizable_containers: &[],
            },
        );
        map.insert(
            "underwater_ruin/big_cracked_2",
            TemplateContainerData {
                size: [16, 16, 16],
                chests: &[],
                markers: &[
                    TemplateMarker {
                        x: 9,
                        y: 1,
                        z: 7,
                        metadata: "drowned",
                    },
                    TemplateMarker {
                        x: 9,
                        y: 1,
                        z: 10,
                        metadata: "chest",
                    },
                    TemplateMarker {
                        x: 12,
                        y: 1,
                        z: 9,
                        metadata: "drowned",
                    },
                ],
                randomizable_containers: &[],
            },
        );
        map.insert(
            "underwater_ruin/big_cracked_3",
            TemplateContainerData {
                size: [16, 16, 16],
                chests: &[],
                markers: &[
                    TemplateMarker {
                        x: 9,
                        y: 2,
                        z: 2,
                        metadata: "drowned",
                    },
                    TemplateMarker {
                        x: 9,
                        y: 2,
                        z: 7,
                        metadata: "drowned",
                    },
                    TemplateMarker {
                        x: 12,
                        y: 2,
                        z: 2,
                        metadata: "chest",
                    },
                    TemplateMarker {
                        x: 13,
                        y: 2,
                        z: 6,
                        metadata: "drowned",
                    },
                ],
                randomizable_containers: &[],
            },
        );
        map.insert(
            "underwater_ruin/big_cracked_8",
            TemplateContainerData {
                size: [16, 16, 16],
                chests: &[],
                markers: &[
                    TemplateMarker {
                        x: 5,
                        y: 0,
                        z: 4,
                        metadata: "chest",
                    },
                    TemplateMarker {
                        x: 2,
                        y: 1,
                        z: 4,
                        metadata: "drowned",
                    },
                ],
                randomizable_containers: &[],
            },
        );
        map.insert(
            "underwater_ruin/big_mossy_1",
            TemplateContainerData {
                size: [16, 16, 16],
                chests: &[],
                markers: &[
                    TemplateMarker {
                        x: 5,
                        y: 0,
                        z: 4,
                        metadata: "chest",
                    },
                    TemplateMarker {
                        x: 6,
                        y: 1,
                        z: 8,
                        metadata: "drowned",
                    },
                ],
                randomizable_containers: &[],
            },
        );
        map.insert(
            "underwater_ruin/big_mossy_2",
            TemplateContainerData {
                size: [16, 16, 16],
                chests: &[],
                markers: &[
                    TemplateMarker {
                        x: 4,
                        y: 1,
                        z: 5,
                        metadata: "drowned",
                    },
                    TemplateMarker {
                        x: 9,
                        y: 1,
                        z: 6,
                        metadata: "drowned",
                    },
                    TemplateMarker {
                        x: 9,
                        y: 1,
                        z: 10,
                        metadata: "chest",
                    },
                ],
                randomizable_containers: &[],
            },
        );
        map.insert(
            "underwater_ruin/big_mossy_3",
            TemplateContainerData {
                size: [16, 16, 16],
                chests: &[],
                markers: &[
                    TemplateMarker {
                        x: 5,
                        y: 1,
                        z: 9,
                        metadata: "drowned",
                    },
                    TemplateMarker {
                        x: 12,
                        y: 2,
                        z: 2,
                        metadata: "chest",
                    },
                ],
                randomizable_containers: &[],
            },
        );
        map.insert(
            "underwater_ruin/big_mossy_8",
            TemplateContainerData {
                size: [16, 16, 16],
                chests: &[],
                markers: &[
                    TemplateMarker {
                        x: 5,
                        y: 0,
                        z: 4,
                        metadata: "chest",
                    },
                    TemplateMarker {
                        x: 7,
                        y: 1,
                        z: 8,
                        metadata: "drowned",
                    },
                    TemplateMarker {
                        x: 11,
                        y: 1,
                        z: 10,
                        metadata: "drowned",
                    },
                ],
                randomizable_containers: &[],
            },
        );
        map.insert(
            "underwater_ruin/big_warm_4",
            TemplateContainerData {
                size: [16, 16, 16],
                chests: &[],
                markers: &[
                    TemplateMarker {
                        x: 11,
                        y: 0,
                        z: 8,
                        metadata: "chest",
                    },
                    TemplateMarker {
                        x: 3,
                        y: 1,
                        z: 5,
                        metadata: "drowned",
                    },
                    TemplateMarker {
                        x: 4,
                        y: 2,
                        z: 6,
                        metadata: "drowned",
                    },
                ],
                randomizable_containers: &[],
            },
        );
        map.insert(
            "underwater_ruin/big_warm_5",
            TemplateContainerData {
                size: [16, 16, 16],
                chests: &[],
                markers: &[
                    TemplateMarker {
                        x: 7,
                        y: 0,
                        z: 7,
                        metadata: "chest",
                    },
                    TemplateMarker {
                        x: 5,
                        y: 1,
                        z: 5,
                        metadata: "drowned",
                    },
                    TemplateMarker {
                        x: 7,
                        y: 1,
                        z: 9,
                        metadata: "drowned",
                    },
                    TemplateMarker {
                        x: 10,
                        y: 1,
                        z: 12,
                        metadata: "drowned",
                    },
                    TemplateMarker {
                        x: 11,
                        y: 1,
                        z: 4,
                        metadata: "drowned",
                    },
                ],
                randomizable_containers: &[],
            },
        );
        map.insert(
            "underwater_ruin/big_warm_6",
            TemplateContainerData {
                size: [16, 16, 16],
                chests: &[],
                markers: &[
                    TemplateMarker {
                        x: 10,
                        y: 0,
                        z: 9,
                        metadata: "chest",
                    },
                    TemplateMarker {
                        x: 5,
                        y: 1,
                        z: 4,
                        metadata: "drowned",
                    },
                    TemplateMarker {
                        x: 9,
                        y: 1,
                        z: 9,
                        metadata: "drowned",
                    },
                ],
                randomizable_containers: &[],
            },
        );
        map.insert(
            "underwater_ruin/big_warm_7",
            TemplateContainerData {
                size: [16, 16, 16],
                chests: &[],
                markers: &[
                    TemplateMarker {
                        x: 11,
                        y: 0,
                        z: 7,
                        metadata: "chest",
                    },
                    TemplateMarker {
                        x: 4,
                        y: 1,
                        z: 5,
                        metadata: "drowned",
                    },
                    TemplateMarker {
                        x: 10,
                        y: 1,
                        z: 8,
                        metadata: "drowned",
                    },
                ],
                randomizable_containers: &[],
            },
        );
        map.insert(
            "underwater_ruin/brick_1",
            TemplateContainerData {
                size: [6, 7, 7],
                chests: &[],
                markers: &[TemplateMarker {
                    x: 3,
                    y: 1,
                    z: 5,
                    metadata: "chest",
                }],
                randomizable_containers: &[],
            },
        );
        map.insert(
            "underwater_ruin/brick_2",
            TemplateContainerData {
                size: [6, 7, 7],
                chests: &[],
                markers: &[
                    TemplateMarker {
                        x: 2,
                        y: 0,
                        z: 1,
                        metadata: "chest",
                    },
                    TemplateMarker {
                        x: 2,
                        y: 1,
                        z: 5,
                        metadata: "drowned",
                    },
                    TemplateMarker {
                        x: 3,
                        y: 1,
                        z: 2,
                        metadata: "drowned",
                    },
                ],
                randomizable_containers: &[],
            },
        );
        map.insert(
            "underwater_ruin/brick_3",
            TemplateContainerData {
                size: [6, 7, 7],
                chests: &[],
                markers: &[TemplateMarker {
                    x: 1,
                    y: 0,
                    z: 5,
                    metadata: "chest",
                }],
                randomizable_containers: &[],
            },
        );
        map.insert(
            "underwater_ruin/brick_4",
            TemplateContainerData {
                size: [6, 7, 7],
                chests: &[],
                markers: &[
                    TemplateMarker {
                        x: 1,
                        y: 1,
                        z: 4,
                        metadata: "chest",
                    },
                    TemplateMarker {
                        x: 2,
                        y: 1,
                        z: 5,
                        metadata: "drowned",
                    },
                    TemplateMarker {
                        x: 4,
                        y: 1,
                        z: 5,
                        metadata: "drowned",
                    },
                ],
                randomizable_containers: &[],
            },
        );
        map.insert(
            "underwater_ruin/brick_5",
            TemplateContainerData {
                size: [6, 7, 7],
                chests: &[],
                markers: &[TemplateMarker {
                    x: 4,
                    y: 0,
                    z: 4,
                    metadata: "chest",
                }],
                randomizable_containers: &[],
            },
        );
        map.insert(
            "underwater_ruin/brick_6",
            TemplateContainerData {
                size: [6, 7, 7],
                chests: &[],
                markers: &[
                    TemplateMarker {
                        x: 2,
                        y: 0,
                        z: 2,
                        metadata: "chest",
                    },
                    TemplateMarker {
                        x: 2,
                        y: 1,
                        z: 3,
                        metadata: "drowned",
                    },
                    TemplateMarker {
                        x: 1,
                        y: 6,
                        z: 4,
                        metadata: "drowned",
                    },
                    TemplateMarker {
                        x: 3,
                        y: 6,
                        z: 1,
                        metadata: "drowned",
                    },
                    TemplateMarker {
                        x: 3,
                        y: 6,
                        z: 4,
                        metadata: "drowned",
                    },
                ],
                randomizable_containers: &[],
            },
        );
        map.insert(
            "underwater_ruin/brick_7",
            TemplateContainerData {
                size: [6, 7, 7],
                chests: &[],
                markers: &[TemplateMarker {
                    x: 1,
                    y: 0,
                    z: 3,
                    metadata: "chest",
                }],
                randomizable_containers: &[],
            },
        );
        map.insert(
            "underwater_ruin/brick_8",
            TemplateContainerData {
                size: [6, 7, 7],
                chests: &[],
                markers: &[
                    TemplateMarker {
                        x: 2,
                        y: 1,
                        z: 4,
                        metadata: "drowned",
                    },
                    TemplateMarker {
                        x: 3,
                        y: 1,
                        z: 4,
                        metadata: "chest",
                    },
                ],
                randomizable_containers: &[],
            },
        );
        map.insert(
            "underwater_ruin/cracked_1",
            TemplateContainerData {
                size: [6, 7, 7],
                chests: &[],
                markers: &[TemplateMarker {
                    x: 3,
                    y: 1,
                    z: 5,
                    metadata: "chest",
                }],
                randomizable_containers: &[],
            },
        );
        map.insert(
            "underwater_ruin/cracked_2",
            TemplateContainerData {
                size: [6, 7, 7],
                chests: &[],
                markers: &[TemplateMarker {
                    x: 2,
                    y: 0,
                    z: 1,
                    metadata: "chest",
                }],
                randomizable_containers: &[],
            },
        );
        map.insert(
            "underwater_ruin/cracked_3",
            TemplateContainerData {
                size: [6, 7, 7],
                chests: &[],
                markers: &[TemplateMarker {
                    x: 1,
                    y: 0,
                    z: 5,
                    metadata: "chest",
                }],
                randomizable_containers: &[],
            },
        );
        map.insert(
            "underwater_ruin/cracked_4",
            TemplateContainerData {
                size: [6, 7, 7],
                chests: &[],
                markers: &[TemplateMarker {
                    x: 1,
                    y: 1,
                    z: 4,
                    metadata: "chest",
                }],
                randomizable_containers: &[],
            },
        );
        map.insert(
            "underwater_ruin/cracked_5",
            TemplateContainerData {
                size: [6, 7, 7],
                chests: &[],
                markers: &[TemplateMarker {
                    x: 4,
                    y: 0,
                    z: 4,
                    metadata: "chest",
                }],
                randomizable_containers: &[],
            },
        );
        map.insert(
            "underwater_ruin/cracked_6",
            TemplateContainerData {
                size: [6, 7, 7],
                chests: &[],
                markers: &[TemplateMarker {
                    x: 2,
                    y: 0,
                    z: 2,
                    metadata: "chest",
                }],
                randomizable_containers: &[],
            },
        );
        map.insert(
            "underwater_ruin/cracked_7",
            TemplateContainerData {
                size: [6, 7, 7],
                chests: &[],
                markers: &[
                    TemplateMarker {
                        x: 1,
                        y: 0,
                        z: 3,
                        metadata: "chest",
                    },
                    TemplateMarker {
                        x: 2,
                        y: 1,
                        z: 3,
                        metadata: "drowned",
                    },
                ],
                randomizable_containers: &[],
            },
        );
        map.insert(
            "underwater_ruin/cracked_8",
            TemplateContainerData {
                size: [6, 7, 7],
                chests: &[],
                markers: &[TemplateMarker {
                    x: 3,
                    y: 1,
                    z: 4,
                    metadata: "chest",
                }],
                randomizable_containers: &[],
            },
        );
        map.insert(
            "underwater_ruin/mossy_1",
            TemplateContainerData {
                size: [6, 7, 7],
                chests: &[],
                markers: &[TemplateMarker {
                    x: 1,
                    y: 1,
                    z: 2,
                    metadata: "drowned",
                }],
                randomizable_containers: &[],
            },
        );
        map.insert(
            "underwater_ruin/mossy_2",
            TemplateContainerData {
                size: [6, 7, 7],
                chests: &[],
                markers: &[TemplateMarker {
                    x: 2,
                    y: 0,
                    z: 1,
                    metadata: "chest",
                }],
                randomizable_containers: &[],
            },
        );
        map.insert(
            "underwater_ruin/mossy_3",
            TemplateContainerData {
                size: [6, 7, 7],
                chests: &[],
                markers: &[TemplateMarker {
                    x: 1,
                    y: 0,
                    z: 5,
                    metadata: "chest",
                }],
                randomizable_containers: &[],
            },
        );
        map.insert(
            "underwater_ruin/mossy_4",
            TemplateContainerData {
                size: [6, 7, 7],
                chests: &[],
                markers: &[TemplateMarker {
                    x: 1,
                    y: 1,
                    z: 4,
                    metadata: "chest",
                }],
                randomizable_containers: &[],
            },
        );
        map.insert(
            "underwater_ruin/mossy_5",
            TemplateContainerData {
                size: [6, 7, 7],
                chests: &[],
                markers: &[
                    TemplateMarker {
                        x: 4,
                        y: 0,
                        z: 4,
                        metadata: "chest",
                    },
                    TemplateMarker {
                        x: 3,
                        y: 1,
                        z: 3,
                        metadata: "drowned",
                    },
                ],
                randomizable_containers: &[],
            },
        );
        map.insert(
            "underwater_ruin/mossy_6",
            TemplateContainerData {
                size: [6, 7, 7],
                chests: &[],
                markers: &[
                    TemplateMarker {
                        x: 2,
                        y: 0,
                        z: 2,
                        metadata: "chest",
                    },
                    TemplateMarker {
                        x: 2,
                        y: 6,
                        z: 2,
                        metadata: "drowned",
                    },
                ],
                randomizable_containers: &[],
            },
        );
        map.insert(
            "underwater_ruin/mossy_7",
            TemplateContainerData {
                size: [6, 7, 7],
                chests: &[],
                markers: &[
                    TemplateMarker {
                        x: 1,
                        y: 0,
                        z: 3,
                        metadata: "chest",
                    },
                    TemplateMarker {
                        x: 2,
                        y: 1,
                        z: 2,
                        metadata: "drowned",
                    },
                ],
                randomizable_containers: &[],
            },
        );
        map.insert(
            "underwater_ruin/mossy_8",
            TemplateContainerData {
                size: [6, 7, 7],
                chests: &[],
                markers: &[
                    TemplateMarker {
                        x: 3,
                        y: 1,
                        z: 4,
                        metadata: "chest",
                    },
                    TemplateMarker {
                        x: 4,
                        y: 1,
                        z: 4,
                        metadata: "drowned",
                    },
                ],
                randomizable_containers: &[],
            },
        );
        map.insert(
            "underwater_ruin/warm_1",
            TemplateContainerData {
                size: [6, 7, 7],
                chests: &[],
                markers: &[TemplateMarker {
                    x: 3,
                    y: 1,
                    z: 1,
                    metadata: "chest",
                }],
                randomizable_containers: &[],
            },
        );
        map.insert(
            "underwater_ruin/warm_2",
            TemplateContainerData {
                size: [6, 7, 7],
                chests: &[],
                markers: &[
                    TemplateMarker {
                        x: 3,
                        y: 1,
                        z: 4,
                        metadata: "chest",
                    },
                    TemplateMarker {
                        x: 2,
                        y: 2,
                        z: 2,
                        metadata: "drowned",
                    },
                    TemplateMarker {
                        x: 3,
                        y: 2,
                        z: 2,
                        metadata: "drowned",
                    },
                ],
                randomizable_containers: &[],
            },
        );
        map.insert(
            "underwater_ruin/warm_3",
            TemplateContainerData {
                size: [6, 7, 7],
                chests: &[],
                markers: &[
                    TemplateMarker {
                        x: 3,
                        y: 0,
                        z: 4,
                        metadata: "chest",
                    },
                    TemplateMarker {
                        x: 3,
                        y: 1,
                        z: 2,
                        metadata: "drowned",
                    },
                ],
                randomizable_containers: &[],
            },
        );
        map.insert(
            "underwater_ruin/warm_4",
            TemplateContainerData {
                size: [6, 7, 7],
                chests: &[],
                markers: &[
                    TemplateMarker {
                        x: 1,
                        y: 0,
                        z: 2,
                        metadata: "chest",
                    },
                    TemplateMarker {
                        x: 2,
                        y: 1,
                        z: 2,
                        metadata: "drowned",
                    },
                ],
                randomizable_containers: &[],
            },
        );
        map.insert(
            "underwater_ruin/warm_5",
            TemplateContainerData {
                size: [6, 7, 7],
                chests: &[],
                markers: &[
                    TemplateMarker {
                        x: 3,
                        y: 0,
                        z: 4,
                        metadata: "chest",
                    },
                    TemplateMarker {
                        x: 2,
                        y: 1,
                        z: 2,
                        metadata: "drowned",
                    },
                ],
                randomizable_containers: &[],
            },
        );
        map.insert(
            "underwater_ruin/warm_6",
            TemplateContainerData {
                size: [6, 7, 7],
                chests: &[],
                markers: &[TemplateMarker {
                    x: 4,
                    y: 1,
                    z: 4,
                    metadata: "chest",
                }],
                randomizable_containers: &[],
            },
        );
        map.insert(
            "underwater_ruin/warm_7",
            TemplateContainerData {
                size: [6, 7, 7],
                chests: &[],
                markers: &[TemplateMarker {
                    x: 3,
                    y: 0,
                    z: 3,
                    metadata: "chest",
                }],
                randomizable_containers: &[],
            },
        );
        map.insert(
            "underwater_ruin/warm_8",
            TemplateContainerData {
                size: [6, 7, 7],
                chests: &[],
                markers: &[
                    TemplateMarker {
                        x: 3,
                        y: 0,
                        z: 3,
                        metadata: "chest",
                    },
                    TemplateMarker {
                        x: 1,
                        y: 2,
                        z: 3,
                        metadata: "drowned",
                    },
                ],
                randomizable_containers: &[],
            },
        );
        map.insert(
            "village/desert/houses/desert_fisher_1",
            TemplateContainerData {
                size: [8, 6, 11],
                chests: &[],
                markers: &[],
                randomizable_containers: &[
                    TemplateBlockPos { x: 2, y: 1, z: 5 },
                    TemplateBlockPos { x: 5, y: 1, z: 8 },
                    TemplateBlockPos { x: 6, y: 1, z: 4 },
                    TemplateBlockPos { x: 6, y: 1, z: 8 },
                    TemplateBlockPos { x: 2, y: 2, z: 5 },
                    TemplateBlockPos { x: 6, y: 2, z: 8 },
                ],
            },
        );
        map.insert(
            "village/desert/houses/desert_medium_house_1",
            TemplateContainerData {
                size: [6, 6, 7],
                chests: &[TemplateChest {
                    x: 2,
                    y: 1,
                    z: 1,
                    loot_table: "minecraft:chests/village/village_desert_house",
                }],
                markers: &[],
                randomizable_containers: &[TemplateBlockPos { x: 2, y: 1, z: 1 }],
            },
        );
        map.insert(
            "village/desert/houses/desert_medium_house_2",
            TemplateContainerData {
                size: [11, 9, 7],
                chests: &[TemplateChest {
                    x: 8,
                    y: 3,
                    z: 3,
                    loot_table: "minecraft:chests/village/village_desert_house",
                }],
                markers: &[],
                randomizable_containers: &[TemplateBlockPos { x: 8, y: 3, z: 3 }],
            },
        );
        map.insert(
            "village/desert/houses/desert_small_house_4",
            TemplateContainerData {
                size: [5, 5, 5],
                chests: &[TemplateChest {
                    x: 3,
                    y: 1,
                    z: 1,
                    loot_table: "minecraft:chests/village/village_desert_house",
                }],
                markers: &[],
                randomizable_containers: &[TemplateBlockPos { x: 3, y: 1, z: 1 }],
            },
        );
        map.insert(
            "village/desert/houses/desert_small_house_6",
            TemplateContainerData {
                size: [6, 18, 5],
                chests: &[TemplateChest {
                    x: 3,
                    y: 13,
                    z: 2,
                    loot_table: "minecraft:chests/village/village_desert_house",
                }],
                markers: &[],
                randomizable_containers: &[TemplateBlockPos { x: 3, y: 13, z: 2 }],
            },
        );
        map.insert(
            "village/desert/houses/desert_temple_1",
            TemplateContainerData {
                size: [11, 7, 10],
                chests: &[TemplateChest {
                    x: 1,
                    y: 1,
                    z: 8,
                    loot_table: "minecraft:chests/village/village_temple",
                }],
                markers: &[],
                randomizable_containers: &[TemplateBlockPos { x: 1, y: 1, z: 8 }],
            },
        );
        map.insert(
            "village/desert/houses/desert_tool_smith_1",
            TemplateContainerData {
                size: [9, 9, 9],
                chests: &[TemplateChest {
                    x: 7,
                    y: 5,
                    z: 1,
                    loot_table: "minecraft:chests/village/village_toolsmith",
                }],
                markers: &[],
                randomizable_containers: &[TemplateBlockPos { x: 7, y: 5, z: 1 }],
            },
        );
        map.insert(
            "village/desert/houses/desert_weaponsmith_1",
            TemplateContainerData {
                size: [10, 6, 7],
                chests: &[TemplateChest {
                    x: 1,
                    y: 1,
                    z: 2,
                    loot_table: "minecraft:chests/village/village_weaponsmith",
                }],
                markers: &[],
                randomizable_containers: &[TemplateBlockPos { x: 1, y: 1, z: 2 }],
            },
        );
        map.insert(
            "village/desert/zombie/houses/desert_medium_house_1",
            TemplateContainerData {
                size: [6, 6, 7],
                chests: &[TemplateChest {
                    x: 2,
                    y: 1,
                    z: 1,
                    loot_table: "minecraft:chests/village/village_desert_house",
                }],
                markers: &[],
                randomizable_containers: &[TemplateBlockPos { x: 2, y: 1, z: 1 }],
            },
        );
        map.insert(
            "village/desert/zombie/houses/desert_medium_house_2",
            TemplateContainerData {
                size: [11, 9, 7],
                chests: &[TemplateChest {
                    x: 8,
                    y: 3,
                    z: 3,
                    loot_table: "minecraft:chests/village/village_desert_house",
                }],
                markers: &[],
                randomizable_containers: &[TemplateBlockPos { x: 8, y: 3, z: 3 }],
            },
        );
        map.insert(
            "village/desert/zombie/houses/desert_small_house_4",
            TemplateContainerData {
                size: [5, 5, 5],
                chests: &[TemplateChest {
                    x: 3,
                    y: 1,
                    z: 1,
                    loot_table: "minecraft:chests/village/village_desert_house",
                }],
                markers: &[],
                randomizable_containers: &[TemplateBlockPos { x: 3, y: 1, z: 1 }],
            },
        );
        map.insert(
            "village/desert/zombie/houses/desert_small_house_6",
            TemplateContainerData {
                size: [5, 17, 5],
                chests: &[TemplateChest {
                    x: 2,
                    y: 13,
                    z: 2,
                    loot_table: "minecraft:chests/village/village_desert_house",
                }],
                markers: &[],
                randomizable_containers: &[TemplateBlockPos { x: 2, y: 13, z: 2 }],
            },
        );
        map.insert(
            "village/plains/houses/plains_big_house_1",
            TemplateContainerData {
                size: [7, 11, 11],
                chests: &[TemplateChest {
                    x: 4,
                    y: 5,
                    z: 8,
                    loot_table: "minecraft:chests/village/village_plains_house",
                }],
                markers: &[],
                randomizable_containers: &[TemplateBlockPos { x: 4, y: 5, z: 8 }],
            },
        );
        map.insert(
            "village/plains/houses/plains_cartographer_1",
            TemplateContainerData {
                size: [10, 8, 7],
                chests: &[TemplateChest {
                    x: 3,
                    y: 1,
                    z: 4,
                    loot_table: "minecraft:chests/village/village_cartographer",
                }],
                markers: &[],
                randomizable_containers: &[TemplateBlockPos { x: 3, y: 1, z: 4 }],
            },
        );
        map.insert(
            "village/plains/houses/plains_fisher_cottage_1",
            TemplateContainerData {
                size: [11, 9, 10],
                chests: &[TemplateChest {
                    x: 4,
                    y: 3,
                    z: 6,
                    loot_table: "minecraft:chests/village/village_fisher",
                }],
                markers: &[],
                randomizable_containers: &[
                    TemplateBlockPos { x: 2, y: 2, z: 2 },
                    TemplateBlockPos { x: 9, y: 2, z: 7 },
                    TemplateBlockPos { x: 4, y: 3, z: 6 },
                ],
            },
        );
        map.insert(
            "village/plains/houses/plains_medium_house_2",
            TemplateContainerData {
                size: [7, 6, 13],
                chests: &[TemplateChest {
                    x: 2,
                    y: 1,
                    z: 10,
                    loot_table: "minecraft:chests/village/village_plains_house",
                }],
                markers: &[],
                randomizable_containers: &[TemplateBlockPos { x: 2, y: 1, z: 10 }],
            },
        );
        map.insert(
            "village/plains/houses/plains_small_house_7",
            TemplateContainerData {
                size: [7, 7, 8],
                chests: &[TemplateChest {
                    x: 4,
                    y: 1,
                    z: 5,
                    loot_table: "minecraft:chests/village/village_plains_house",
                }],
                markers: &[],
                randomizable_containers: &[TemplateBlockPos { x: 4, y: 1, z: 5 }],
            },
        );
        map.insert(
            "village/plains/houses/plains_tannery_1",
            TemplateContainerData {
                size: [8, 7, 10],
                chests: &[TemplateChest {
                    x: 5,
                    y: 1,
                    z: 7,
                    loot_table: "minecraft:chests/village/village_tannery",
                }],
                markers: &[],
                randomizable_containers: &[TemplateBlockPos { x: 5, y: 1, z: 7 }],
            },
        );
        map.insert(
            "village/plains/houses/plains_weaponsmith_1",
            TemplateContainerData {
                size: [9, 8, 11],
                chests: &[TemplateChest {
                    x: 6,
                    y: 1,
                    z: 4,
                    loot_table: "minecraft:chests/village/village_weaponsmith",
                }],
                markers: &[],
                randomizable_containers: &[TemplateBlockPos { x: 6, y: 1, z: 4 }],
            },
        );
        map.insert(
            "village/plains/zombie/houses/plains_big_house_1",
            TemplateContainerData {
                size: [7, 11, 11],
                chests: &[TemplateChest {
                    x: 4,
                    y: 5,
                    z: 8,
                    loot_table: "minecraft:chests/village/village_plains_house",
                }],
                markers: &[],
                randomizable_containers: &[TemplateBlockPos { x: 4, y: 5, z: 8 }],
            },
        );
        map.insert(
            "village/plains/zombie/houses/plains_medium_house_2",
            TemplateContainerData {
                size: [7, 6, 13],
                chests: &[TemplateChest {
                    x: 2,
                    y: 1,
                    z: 10,
                    loot_table: "minecraft:chests/village/village_plains_house",
                }],
                markers: &[],
                randomizable_containers: &[TemplateBlockPos { x: 2, y: 1, z: 10 }],
            },
        );
        map.insert(
            "village/plains/zombie/houses/plains_small_house_7",
            TemplateContainerData {
                size: [7, 7, 8],
                chests: &[TemplateChest {
                    x: 4,
                    y: 1,
                    z: 5,
                    loot_table: "minecraft:chests/village/village_plains_house",
                }],
                markers: &[],
                randomizable_containers: &[TemplateBlockPos { x: 4, y: 1, z: 5 }],
            },
        );
        map.insert(
            "village/savanna/houses/savanna_butchers_shop_2",
            TemplateContainerData {
                size: [13, 10, 9],
                chests: &[TemplateChest {
                    x: 6,
                    y: 4,
                    z: 5,
                    loot_table: "minecraft:chests/village/village_butcher",
                }],
                markers: &[],
                randomizable_containers: &[TemplateBlockPos { x: 6, y: 4, z: 5 }],
            },
        );
        map.insert(
            "village/savanna/houses/savanna_cartographer_1",
            TemplateContainerData {
                size: [8, 8, 9],
                chests: &[TemplateChest {
                    x: 4,
                    y: 3,
                    z: 6,
                    loot_table: "minecraft:chests/village/village_cartographer",
                }],
                markers: &[],
                randomizable_containers: &[TemplateBlockPos { x: 4, y: 3, z: 6 }],
            },
        );
        map.insert(
            "village/savanna/houses/savanna_fisher_cottage_1",
            TemplateContainerData {
                size: [8, 11, 9],
                chests: &[],
                markers: &[],
                randomizable_containers: &[
                    TemplateBlockPos { x: 2, y: 2, z: 4 },
                    TemplateBlockPos { x: 6, y: 2, z: 2 },
                    TemplateBlockPos { x: 2, y: 3, z: 4 },
                ],
            },
        );
        map.insert(
            "village/savanna/houses/savanna_mason_1",
            TemplateContainerData {
                size: [8, 7, 10],
                chests: &[TemplateChest {
                    x: 5,
                    y: 1,
                    z: 5,
                    loot_table: "minecraft:chests/village/village_mason",
                }],
                markers: &[],
                randomizable_containers: &[TemplateBlockPos { x: 5, y: 1, z: 5 }],
            },
        );
        map.insert(
            "village/savanna/houses/savanna_medium_house_1",
            TemplateContainerData {
                size: [8, 7, 15],
                chests: &[TemplateChest {
                    x: 2,
                    y: 1,
                    z: 2,
                    loot_table: "minecraft:chests/village/village_savanna_house",
                }],
                markers: &[],
                randomizable_containers: &[TemplateBlockPos { x: 2, y: 1, z: 2 }],
            },
        );
        map.insert(
            "village/savanna/houses/savanna_medium_house_2",
            TemplateContainerData {
                size: [10, 8, 11],
                chests: &[TemplateChest {
                    x: 7,
                    y: 1,
                    z: 6,
                    loot_table: "minecraft:chests/village/village_savanna_house",
                }],
                markers: &[],
                randomizable_containers: &[TemplateBlockPos { x: 7, y: 1, z: 6 }],
            },
        );
        map.insert(
            "village/savanna/houses/savanna_small_house_2",
            TemplateContainerData {
                size: [7, 7, 7],
                chests: &[TemplateChest {
                    x: 2,
                    y: 1,
                    z: 2,
                    loot_table: "minecraft:chests/village/village_savanna_house",
                }],
                markers: &[],
                randomizable_containers: &[TemplateBlockPos { x: 2, y: 1, z: 2 }],
            },
        );
        map.insert(
            "village/savanna/houses/savanna_small_house_4",
            TemplateContainerData {
                size: [10, 8, 7],
                chests: &[TemplateChest {
                    x: 7,
                    y: 1,
                    z: 4,
                    loot_table: "minecraft:chests/village/village_savanna_house",
                }],
                markers: &[],
                randomizable_containers: &[TemplateBlockPos { x: 7, y: 1, z: 4 }],
            },
        );
        map.insert(
            "village/savanna/houses/savanna_small_house_7",
            TemplateContainerData {
                size: [7, 7, 7],
                chests: &[TemplateChest {
                    x: 2,
                    y: 1,
                    z: 4,
                    loot_table: "minecraft:chests/village/village_savanna_house",
                }],
                markers: &[],
                randomizable_containers: &[TemplateBlockPos { x: 2, y: 1, z: 4 }],
            },
        );
        map.insert(
            "village/savanna/houses/savanna_tannery_1",
            TemplateContainerData {
                size: [8, 6, 9],
                chests: &[TemplateChest {
                    x: 2,
                    y: 1,
                    z: 6,
                    loot_table: "minecraft:chests/village/village_tannery",
                }],
                markers: &[],
                randomizable_containers: &[TemplateBlockPos { x: 2, y: 1, z: 6 }],
            },
        );
        map.insert(
            "village/savanna/houses/savanna_weaponsmith_2",
            TemplateContainerData {
                size: [9, 7, 13],
                chests: &[TemplateChest {
                    x: 2,
                    y: 2,
                    z: 9,
                    loot_table: "minecraft:chests/village/village_weaponsmith",
                }],
                markers: &[],
                randomizable_containers: &[TemplateBlockPos { x: 2, y: 2, z: 9 }],
            },
        );
        map.insert(
            "village/savanna/zombie/houses/savanna_medium_house_1",
            TemplateContainerData {
                size: [8, 7, 15],
                chests: &[TemplateChest {
                    x: 2,
                    y: 1,
                    z: 2,
                    loot_table: "minecraft:chests/village/village_savanna_house",
                }],
                markers: &[],
                randomizable_containers: &[TemplateBlockPos { x: 2, y: 1, z: 2 }],
            },
        );
        map.insert(
            "village/savanna/zombie/houses/savanna_medium_house_2",
            TemplateContainerData {
                size: [10, 8, 11],
                chests: &[TemplateChest {
                    x: 7,
                    y: 1,
                    z: 6,
                    loot_table: "minecraft:chests/village/village_savanna_house",
                }],
                markers: &[],
                randomizable_containers: &[TemplateBlockPos { x: 7, y: 1, z: 6 }],
            },
        );
        map.insert(
            "village/savanna/zombie/houses/savanna_small_house_2",
            TemplateContainerData {
                size: [7, 7, 7],
                chests: &[TemplateChest {
                    x: 2,
                    y: 1,
                    z: 2,
                    loot_table: "minecraft:chests/village/village_savanna_house",
                }],
                markers: &[],
                randomizable_containers: &[TemplateBlockPos { x: 2, y: 1, z: 2 }],
            },
        );
        map.insert(
            "village/savanna/zombie/houses/savanna_small_house_4",
            TemplateContainerData {
                size: [10, 8, 7],
                chests: &[TemplateChest {
                    x: 7,
                    y: 1,
                    z: 4,
                    loot_table: "minecraft:chests/village/village_savanna_house",
                }],
                markers: &[],
                randomizable_containers: &[TemplateBlockPos { x: 7, y: 1, z: 4 }],
            },
        );
        map.insert(
            "village/savanna/zombie/houses/savanna_small_house_7",
            TemplateContainerData {
                size: [7, 7, 7],
                chests: &[TemplateChest {
                    x: 2,
                    y: 1,
                    z: 4,
                    loot_table: "minecraft:chests/village/village_savanna_house",
                }],
                markers: &[],
                randomizable_containers: &[TemplateBlockPos { x: 2, y: 1, z: 4 }],
            },
        );
        map.insert(
            "village/snowy/houses/snowy_armorer_house_1",
            TemplateContainerData {
                size: [8, 8, 7],
                chests: &[TemplateChest {
                    x: 2,
                    y: 1,
                    z: 5,
                    loot_table: "minecraft:chests/village/village_armorer",
                }],
                markers: &[],
                randomizable_containers: &[TemplateBlockPos { x: 2, y: 1, z: 5 }],
            },
        );
        map.insert(
            "village/snowy/houses/snowy_cartographer_house_1",
            TemplateContainerData {
                size: [7, 7, 11],
                chests: &[TemplateChest {
                    x: 2,
                    y: 1,
                    z: 3,
                    loot_table: "minecraft:chests/village/village_cartographer",
                }],
                markers: &[],
                randomizable_containers: &[TemplateBlockPos { x: 2, y: 1, z: 3 }],
            },
        );
        map.insert(
            "village/snowy/houses/snowy_fisher_cottage",
            TemplateContainerData {
                size: [9, 8, 7],
                chests: &[],
                markers: &[],
                randomizable_containers: &[
                    TemplateBlockPos { x: 5, y: 2, z: 2 },
                    TemplateBlockPos { x: 6, y: 2, z: 2 },
                    TemplateBlockPos { x: 6, y: 3, z: 2 },
                ],
            },
        );
        map.insert(
            "village/snowy/houses/snowy_shepherds_house_1",
            TemplateContainerData {
                size: [9, 5, 10],
                chests: &[TemplateChest {
                    x: 6,
                    y: 1,
                    z: 7,
                    loot_table: "minecraft:chests/village/village_shepherd",
                }],
                markers: &[],
                randomizable_containers: &[TemplateBlockPos { x: 6, y: 1, z: 7 }],
            },
        );
        map.insert(
            "village/snowy/houses/snowy_small_house_5",
            TemplateContainerData {
                size: [7, 5, 5],
                chests: &[TemplateChest {
                    x: 4,
                    y: 1,
                    z: 3,
                    loot_table: "minecraft:chests/village/village_snowy_house",
                }],
                markers: &[],
                randomizable_containers: &[TemplateBlockPos { x: 4, y: 1, z: 3 }],
            },
        );
        map.insert(
            "village/snowy/houses/snowy_small_house_6",
            TemplateContainerData {
                size: [7, 9, 7],
                chests: &[TemplateChest {
                    x: 4,
                    y: 1,
                    z: 2,
                    loot_table: "minecraft:chests/village/village_snowy_house",
                }],
                markers: &[],
                randomizable_containers: &[TemplateBlockPos { x: 4, y: 1, z: 2 }],
            },
        );
        map.insert(
            "village/snowy/houses/snowy_tannery_1",
            TemplateContainerData {
                size: [8, 9, 9],
                chests: &[TemplateChest {
                    x: 5,
                    y: 1,
                    z: 6,
                    loot_table: "minecraft:chests/village/village_tannery",
                }],
                markers: &[],
                randomizable_containers: &[TemplateBlockPos { x: 5, y: 1, z: 6 }],
            },
        );
        map.insert(
            "village/snowy/houses/snowy_weapon_smith_1",
            TemplateContainerData {
                size: [9, 7, 10],
                chests: &[TemplateChest {
                    x: 2,
                    y: 1,
                    z: 2,
                    loot_table: "minecraft:chests/village/village_weaponsmith",
                }],
                markers: &[],
                randomizable_containers: &[TemplateBlockPos { x: 2, y: 1, z: 2 }],
            },
        );
        map.insert(
            "village/snowy/zombie/houses/snowy_small_house_5",
            TemplateContainerData {
                size: [7, 5, 5],
                chests: &[TemplateChest {
                    x: 4,
                    y: 1,
                    z: 3,
                    loot_table: "minecraft:chests/village/village_snowy_house",
                }],
                markers: &[],
                randomizable_containers: &[TemplateBlockPos { x: 4, y: 1, z: 3 }],
            },
        );
        map.insert(
            "village/snowy/zombie/houses/snowy_small_house_6",
            TemplateContainerData {
                size: [7, 9, 7],
                chests: &[TemplateChest {
                    x: 4,
                    y: 1,
                    z: 2,
                    loot_table: "minecraft:chests/village/village_snowy_house",
                }],
                markers: &[],
                randomizable_containers: &[TemplateBlockPos { x: 4, y: 1, z: 2 }],
            },
        );
        map.insert(
            "village/taiga/houses/taiga_cartographer_house_1",
            TemplateContainerData {
                size: [7, 10, 8],
                chests: &[TemplateChest {
                    x: 2,
                    y: 4,
                    z: 3,
                    loot_table: "minecraft:chests/village/village_cartographer",
                }],
                markers: &[],
                randomizable_containers: &[TemplateBlockPos { x: 2, y: 4, z: 3 }],
            },
        );
        map.insert(
            "village/taiga/houses/taiga_fisher_cottage_1",
            TemplateContainerData {
                size: [10, 8, 12],
                chests: &[],
                markers: &[],
                randomizable_containers: &[
                    TemplateBlockPos { x: 8, y: 2, z: 1 },
                    TemplateBlockPos { x: 9, y: 2, z: 4 },
                ],
            },
        );
        map.insert(
            "village/taiga/houses/taiga_fletcher_house_1",
            TemplateContainerData {
                size: [10, 6, 11],
                chests: &[TemplateChest {
                    x: 5,
                    y: 1,
                    z: 4,
                    loot_table: "minecraft:chests/village/village_fletcher",
                }],
                markers: &[],
                randomizable_containers: &[TemplateBlockPos { x: 5, y: 1, z: 4 }],
            },
        );
        map.insert(
            "village/taiga/houses/taiga_medium_house_1",
            TemplateContainerData {
                size: [8, 11, 7],
                chests: &[TemplateChest {
                    x: 2,
                    y: 0,
                    z: 2,
                    loot_table: "minecraft:chests/village/village_taiga_house",
                }],
                markers: &[],
                randomizable_containers: &[TemplateBlockPos { x: 2, y: 0, z: 2 }],
            },
        );
        map.insert(
            "village/taiga/houses/taiga_medium_house_2",
            TemplateContainerData {
                size: [7, 11, 8],
                chests: &[TemplateChest {
                    x: 4,
                    y: 5,
                    z: 4,
                    loot_table: "minecraft:chests/village/village_taiga_house",
                }],
                markers: &[],
                randomizable_containers: &[TemplateBlockPos { x: 4, y: 5, z: 4 }],
            },
        );
        map.insert(
            "village/taiga/houses/taiga_medium_house_3",
            TemplateContainerData {
                size: [8, 7, 13],
                chests: &[TemplateChest {
                    x: 3,
                    y: 1,
                    z: 2,
                    loot_table: "minecraft:chests/village/village_taiga_house",
                }],
                markers: &[],
                randomizable_containers: &[TemplateBlockPos { x: 3, y: 1, z: 2 }],
            },
        );
        map.insert(
            "village/taiga/houses/taiga_small_house_3",
            TemplateContainerData {
                size: [7, 7, 7],
                chests: &[TemplateChest {
                    x: 2,
                    y: 1,
                    z: 2,
                    loot_table: "minecraft:chests/village/village_taiga_house",
                }],
                markers: &[],
                randomizable_containers: &[TemplateBlockPos { x: 2, y: 1, z: 2 }],
            },
        );
        map.insert(
            "village/taiga/houses/taiga_small_house_5",
            TemplateContainerData {
                size: [9, 7, 7],
                chests: &[TemplateChest {
                    x: 3,
                    y: 1,
                    z: 2,
                    loot_table: "minecraft:chests/village/village_taiga_house",
                }],
                markers: &[],
                randomizable_containers: &[TemplateBlockPos { x: 3, y: 1, z: 2 }],
            },
        );
        map.insert(
            "village/taiga/houses/taiga_tannery_1",
            TemplateContainerData {
                size: [9, 6, 9],
                chests: &[TemplateChest {
                    x: 6,
                    y: 1,
                    z: 2,
                    loot_table: "minecraft:chests/village/village_tannery",
                }],
                markers: &[],
                randomizable_containers: &[TemplateBlockPos { x: 6, y: 1, z: 2 }],
            },
        );
        map.insert(
            "village/taiga/houses/taiga_tool_smith_1",
            TemplateContainerData {
                size: [11, 6, 8],
                chests: &[TemplateChest {
                    x: 6,
                    y: 1,
                    z: 3,
                    loot_table: "minecraft:chests/village/village_toolsmith",
                }],
                markers: &[],
                randomizable_containers: &[TemplateBlockPos { x: 6, y: 1, z: 3 }],
            },
        );
        map.insert(
            "village/taiga/houses/taiga_weaponsmith_1",
            TemplateContainerData {
                size: [7, 9, 7],
                chests: &[TemplateChest {
                    x: 2,
                    y: 1,
                    z: 3,
                    loot_table: "minecraft:chests/village/village_weaponsmith",
                }],
                markers: &[],
                randomizable_containers: &[TemplateBlockPos { x: 2, y: 1, z: 3 }],
            },
        );
        map.insert(
            "village/taiga/zombie/houses/taiga_cartographer_house_1",
            TemplateContainerData {
                size: [7, 10, 8],
                chests: &[TemplateChest {
                    x: 2,
                    y: 4,
                    z: 3,
                    loot_table: "minecraft:chests/village/village_cartographer",
                }],
                markers: &[],
                randomizable_containers: &[TemplateBlockPos { x: 2, y: 4, z: 3 }],
            },
        );
        map.insert(
            "village/taiga/zombie/houses/taiga_fisher_cottage_1",
            TemplateContainerData {
                size: [10, 8, 12],
                chests: &[],
                markers: &[],
                randomizable_containers: &[
                    TemplateBlockPos { x: 8, y: 2, z: 1 },
                    TemplateBlockPos { x: 9, y: 2, z: 4 },
                ],
            },
        );
        map.insert(
            "village/taiga/zombie/houses/taiga_medium_house_1",
            TemplateContainerData {
                size: [8, 11, 7],
                chests: &[TemplateChest {
                    x: 2,
                    y: 0,
                    z: 2,
                    loot_table: "minecraft:chests/village/village_taiga_house",
                }],
                markers: &[],
                randomizable_containers: &[TemplateBlockPos { x: 2, y: 0, z: 2 }],
            },
        );
        map.insert(
            "village/taiga/zombie/houses/taiga_medium_house_2",
            TemplateContainerData {
                size: [7, 11, 8],
                chests: &[TemplateChest {
                    x: 4,
                    y: 5,
                    z: 4,
                    loot_table: "minecraft:chests/village/village_taiga_house",
                }],
                markers: &[],
                randomizable_containers: &[TemplateBlockPos { x: 4, y: 5, z: 4 }],
            },
        );
        map.insert(
            "village/taiga/zombie/houses/taiga_medium_house_3",
            TemplateContainerData {
                size: [8, 7, 13],
                chests: &[TemplateChest {
                    x: 3,
                    y: 1,
                    z: 2,
                    loot_table: "minecraft:chests/village/village_taiga_house",
                }],
                markers: &[],
                randomizable_containers: &[TemplateBlockPos { x: 3, y: 1, z: 2 }],
            },
        );
        map.insert(
            "village/taiga/zombie/houses/taiga_small_house_3",
            TemplateContainerData {
                size: [7, 7, 7],
                chests: &[TemplateChest {
                    x: 2,
                    y: 1,
                    z: 2,
                    loot_table: "minecraft:chests/village/village_taiga_house",
                }],
                markers: &[],
                randomizable_containers: &[TemplateBlockPos { x: 2, y: 1, z: 2 }],
            },
        );
        map.insert(
            "village/taiga/zombie/houses/taiga_small_house_5",
            TemplateContainerData {
                size: [9, 7, 7],
                chests: &[TemplateChest {
                    x: 3,
                    y: 1,
                    z: 2,
                    loot_table: "minecraft:chests/village/village_taiga_house",
                }],
                markers: &[],
                randomizable_containers: &[TemplateBlockPos { x: 3, y: 1, z: 2 }],
            },
        );
        map.insert(
            "village/taiga/zombie/houses/taiga_tool_smith_1",
            TemplateContainerData {
                size: [11, 6, 8],
                chests: &[TemplateChest {
                    x: 6,
                    y: 1,
                    z: 3,
                    loot_table: "minecraft:chests/village/village_toolsmith",
                }],
                markers: &[],
                randomizable_containers: &[TemplateBlockPos { x: 6, y: 1, z: 3 }],
            },
        );
        map.insert(
            "woodland_mansion/1x1_a4",
            TemplateContainerData {
                size: [7, 8, 7],
                chests: &[],
                markers: &[TemplateMarker {
                    x: 6,
                    y: 6,
                    z: 3,
                    metadata: "ChestWest",
                }],
                randomizable_containers: &[],
            },
        );
        map.insert(
            "woodland_mansion/1x1_as1",
            TemplateContainerData {
                size: [7, 8, 7],
                chests: &[],
                markers: &[TemplateMarker {
                    x: 5,
                    y: 1,
                    z: 1,
                    metadata: "ChestWest",
                }],
                randomizable_containers: &[],
            },
        );
        map.insert(
            "woodland_mansion/1x1_b5",
            TemplateContainerData {
                size: [7, 11, 7],
                chests: &[],
                markers: &[],
                randomizable_containers: &[TemplateBlockPos { x: 6, y: 1, z: 6 }],
            },
        );
        map.insert(
            "woodland_mansion/1x2_a1",
            TemplateContainerData {
                size: [7, 8, 15],
                chests: &[],
                markers: &[
                    TemplateMarker {
                        x: 3,
                        y: 1,
                        z: 3,
                        metadata: "Warrior",
                    },
                    TemplateMarker {
                        x: 3,
                        y: 1,
                        z: 12,
                        metadata: "ChestSouth",
                    },
                ],
                randomizable_containers: &[],
            },
        );
        map.insert(
            "woodland_mansion/1x2_a3",
            TemplateContainerData {
                size: [7, 8, 15],
                chests: &[],
                markers: &[TemplateMarker {
                    x: 3,
                    y: 2,
                    z: 8,
                    metadata: "Warrior",
                }],
                randomizable_containers: &[],
            },
        );
        map.insert(
            "woodland_mansion/1x2_a4",
            TemplateContainerData {
                size: [7, 8, 15],
                chests: &[],
                markers: &[],
                randomizable_containers: &[TemplateBlockPos { x: 6, y: 1, z: 5 }],
            },
        );
        map.insert(
            "woodland_mansion/1x2_a6",
            TemplateContainerData {
                size: [7, 8, 15],
                chests: &[],
                markers: &[],
                randomizable_containers: &[TemplateBlockPos { x: 2, y: 1, z: 8 }],
            },
        );
        map.insert(
            "woodland_mansion/1x2_a7",
            TemplateContainerData {
                size: [7, 8, 15],
                chests: &[],
                markers: &[],
                randomizable_containers: &[TemplateBlockPos { x: 3, y: 1, z: 0 }],
            },
        );
        map.insert(
            "woodland_mansion/1x2_a8",
            TemplateContainerData {
                size: [7, 8, 15],
                chests: &[],
                markers: &[TemplateMarker {
                    x: 2,
                    y: 1,
                    z: 2,
                    metadata: "Warrior",
                }],
                randomizable_containers: &[],
            },
        );
        map.insert(
            "woodland_mansion/1x2_a9",
            TemplateContainerData {
                size: [7, 8, 15],
                chests: &[],
                markers: &[TemplateMarker {
                    x: 5,
                    y: 1,
                    z: 1,
                    metadata: "Warrior",
                }],
                randomizable_containers: &[
                    TemplateBlockPos { x: 0, y: 2, z: 3 },
                    TemplateBlockPos { x: 0, y: 2, z: 5 },
                    TemplateBlockPos { x: 0, y: 2, z: 7 },
                    TemplateBlockPos { x: 0, y: 2, z: 9 },
                    TemplateBlockPos { x: 0, y: 2, z: 11 },
                    TemplateBlockPos { x: 3, y: 2, z: 3 },
                    TemplateBlockPos { x: 3, y: 2, z: 5 },
                    TemplateBlockPos { x: 3, y: 2, z: 7 },
                    TemplateBlockPos { x: 3, y: 2, z: 9 },
                    TemplateBlockPos { x: 3, y: 2, z: 11 },
                    TemplateBlockPos { x: 5, y: 2, z: 6 },
                    TemplateBlockPos { x: 5, y: 2, z: 8 },
                    TemplateBlockPos { x: 5, y: 2, z: 10 },
                    TemplateBlockPos { x: 5, y: 2, z: 11 },
                    TemplateBlockPos { x: 0, y: 4, z: 3 },
                    TemplateBlockPos { x: 0, y: 4, z: 5 },
                    TemplateBlockPos { x: 0, y: 4, z: 7 },
                    TemplateBlockPos { x: 0, y: 4, z: 9 },
                    TemplateBlockPos { x: 0, y: 4, z: 11 },
                    TemplateBlockPos { x: 3, y: 4, z: 3 },
                    TemplateBlockPos { x: 3, y: 4, z: 5 },
                    TemplateBlockPos { x: 3, y: 4, z: 7 },
                    TemplateBlockPos { x: 3, y: 4, z: 9 },
                    TemplateBlockPos { x: 3, y: 4, z: 11 },
                    TemplateBlockPos { x: 5, y: 4, z: 6 },
                    TemplateBlockPos { x: 5, y: 4, z: 7 },
                    TemplateBlockPos { x: 5, y: 4, z: 9 },
                    TemplateBlockPos { x: 5, y: 4, z: 11 },
                    TemplateBlockPos { x: 0, y: 6, z: 3 },
                    TemplateBlockPos { x: 0, y: 6, z: 5 },
                    TemplateBlockPos { x: 0, y: 6, z: 7 },
                    TemplateBlockPos { x: 0, y: 6, z: 9 },
                    TemplateBlockPos { x: 0, y: 6, z: 11 },
                    TemplateBlockPos { x: 3, y: 6, z: 3 },
                    TemplateBlockPos { x: 3, y: 6, z: 5 },
                    TemplateBlockPos { x: 3, y: 6, z: 7 },
                    TemplateBlockPos { x: 3, y: 6, z: 9 },
                    TemplateBlockPos { x: 3, y: 6, z: 11 },
                    TemplateBlockPos { x: 5, y: 6, z: 6 },
                    TemplateBlockPos { x: 5, y: 6, z: 8 },
                    TemplateBlockPos { x: 5, y: 6, z: 10 },
                    TemplateBlockPos { x: 5, y: 6, z: 11 },
                ],
            },
        );
        map.insert(
            "woodland_mansion/1x2_b1",
            TemplateContainerData {
                size: [7, 8, 15],
                chests: &[],
                markers: &[TemplateMarker {
                    x: 3,
                    y: 1,
                    z: 13,
                    metadata: "Warrior",
                }],
                randomizable_containers: &[],
            },
        );
        map.insert(
            "woodland_mansion/1x2_b2",
            TemplateContainerData {
                size: [7, 8, 15],
                chests: &[],
                markers: &[TemplateMarker {
                    x: 5,
                    y: 1,
                    z: 11,
                    metadata: "Warrior",
                }],
                randomizable_containers: &[],
            },
        );
        map.insert(
            "woodland_mansion/1x2_b3",
            TemplateContainerData {
                size: [7, 8, 15],
                chests: &[],
                markers: &[
                    TemplateMarker {
                        x: 3,
                        y: 1,
                        z: 0,
                        metadata: "ChestSouth",
                    },
                    TemplateMarker {
                        x: 3,
                        y: 1,
                        z: 4,
                        metadata: "Warrior",
                    },
                ],
                randomizable_containers: &[],
            },
        );
        map.insert(
            "woodland_mansion/1x2_b4",
            TemplateContainerData {
                size: [7, 8, 15],
                chests: &[],
                markers: &[TemplateMarker {
                    x: 4,
                    y: 3,
                    z: 9,
                    metadata: "ChestNorth",
                }],
                randomizable_containers: &[],
            },
        );
        map.insert(
            "woodland_mansion/1x2_c3",
            TemplateContainerData {
                size: [7, 11, 15],
                chests: &[],
                markers: &[
                    TemplateMarker {
                        x: 4,
                        y: 1,
                        z: 6,
                        metadata: "Warrior",
                    },
                    TemplateMarker {
                        x: 4,
                        y: 1,
                        z: 9,
                        metadata: "Warrior",
                    },
                    TemplateMarker {
                        x: 4,
                        y: 1,
                        z: 12,
                        metadata: "Warrior",
                    },
                ],
                randomizable_containers: &[],
            },
        );
        map.insert(
            "woodland_mansion/1x2_d2",
            TemplateContainerData {
                size: [7, 11, 15],
                chests: &[],
                markers: &[TemplateMarker {
                    x: 6,
                    y: 7,
                    z: 12,
                    metadata: "ChestWest",
                }],
                randomizable_containers: &[],
            },
        );
        map.insert(
            "woodland_mansion/1x2_d3",
            TemplateContainerData {
                size: [7, 11, 15],
                chests: &[],
                markers: &[
                    TemplateMarker {
                        x: 2,
                        y: 2,
                        z: 10,
                        metadata: "Warrior",
                    },
                    TemplateMarker {
                        x: 4,
                        y: 2,
                        z: 10,
                        metadata: "Warrior",
                    },
                    TemplateMarker {
                        x: 3,
                        y: 3,
                        z: 3,
                        metadata: "Mage",
                    },
                ],
                randomizable_containers: &[],
            },
        );
        map.insert(
            "woodland_mansion/1x2_s1",
            TemplateContainerData {
                size: [7, 8, 15],
                chests: &[],
                markers: &[TemplateMarker {
                    x: 3,
                    y: 1,
                    z: 4,
                    metadata: "ChestSouth",
                }],
                randomizable_containers: &[],
            },
        );
        map.insert(
            "woodland_mansion/1x2_s2",
            TemplateContainerData {
                size: [7, 8, 15],
                chests: &[],
                markers: &[],
                randomizable_containers: &[TemplateBlockPos { x: 3, y: 4, z: 5 }],
            },
        );
        map.insert(
            "woodland_mansion/1x2_se1",
            TemplateContainerData {
                size: [7, 11, 15],
                chests: &[],
                markers: &[
                    TemplateMarker {
                        x: 3,
                        y: 9,
                        z: 1,
                        metadata: "ChestSouth",
                    },
                    TemplateMarker {
                        x: 3,
                        y: 9,
                        z: 13,
                        metadata: "ChestNorth",
                    },
                ],
                randomizable_containers: &[],
            },
        );
        map.insert(
            "woodland_mansion/2x2_a1",
            TemplateContainerData {
                size: [15, 8, 15],
                chests: &[],
                markers: &[
                    TemplateMarker {
                        x: 12,
                        y: 1,
                        z: 3,
                        metadata: "Warrior",
                    },
                    TemplateMarker {
                        x: 2,
                        y: 2,
                        z: 8,
                        metadata: "Group of Allays",
                    },
                    TemplateMarker {
                        x: 4,
                        y: 2,
                        z: 2,
                        metadata: "Group of Allays",
                    },
                    TemplateMarker {
                        x: 5,
                        y: 2,
                        z: 12,
                        metadata: "Group of Allays",
                    },
                    TemplateMarker {
                        x: 12,
                        y: 2,
                        z: 12,
                        metadata: "Group of Allays",
                    },
                ],
                randomizable_containers: &[],
            },
        );
        map.insert(
            "woodland_mansion/2x2_a2",
            TemplateContainerData {
                size: [15, 8, 15],
                chests: &[],
                markers: &[TemplateMarker {
                    x: 9,
                    y: 1,
                    z: 2,
                    metadata: "Warrior",
                }],
                randomizable_containers: &[
                    TemplateBlockPos { x: 0, y: 2, z: 1 },
                    TemplateBlockPos { x: 0, y: 2, z: 2 },
                    TemplateBlockPos { x: 0, y: 2, z: 4 },
                    TemplateBlockPos { x: 0, y: 2, z: 5 },
                    TemplateBlockPos { x: 0, y: 2, z: 7 },
                    TemplateBlockPos { x: 0, y: 2, z: 8 },
                    TemplateBlockPos { x: 0, y: 2, z: 10 },
                    TemplateBlockPos { x: 0, y: 2, z: 11 },
                    TemplateBlockPos { x: 6, y: 2, z: 4 },
                    TemplateBlockPos { x: 6, y: 2, z: 5 },
                    TemplateBlockPos { x: 6, y: 2, z: 7 },
                    TemplateBlockPos { x: 6, y: 2, z: 8 },
                    TemplateBlockPos { x: 6, y: 2, z: 10 },
                    TemplateBlockPos { x: 6, y: 2, z: 11 },
                    TemplateBlockPos { x: 8, y: 2, z: 6 },
                    TemplateBlockPos { x: 8, y: 2, z: 7 },
                    TemplateBlockPos { x: 8, y: 2, z: 9 },
                    TemplateBlockPos { x: 8, y: 2, z: 10 },
                    TemplateBlockPos { x: 8, y: 2, z: 12 },
                    TemplateBlockPos { x: 8, y: 2, z: 13 },
                    TemplateBlockPos { x: 14, y: 2, z: 6 },
                    TemplateBlockPos { x: 14, y: 2, z: 7 },
                    TemplateBlockPos { x: 14, y: 2, z: 9 },
                    TemplateBlockPos { x: 14, y: 2, z: 10 },
                    TemplateBlockPos { x: 14, y: 2, z: 12 },
                    TemplateBlockPos { x: 14, y: 2, z: 13 },
                ],
            },
        );
        map.insert(
            "woodland_mansion/2x2_b1",
            TemplateContainerData {
                size: [15, 11, 15],
                chests: &[],
                markers: &[
                    TemplateMarker {
                        x: 2,
                        y: 1,
                        z: 8,
                        metadata: "Mage",
                    },
                    TemplateMarker {
                        x: 4,
                        y: 1,
                        z: 4,
                        metadata: "Warrior",
                    },
                    TemplateMarker {
                        x: 8,
                        y: 1,
                        z: 9,
                        metadata: "Warrior",
                    },
                ],
                randomizable_containers: &[],
            },
        );
        map.insert(
            "woodland_mansion/2x2_b2",
            TemplateContainerData {
                size: [15, 11, 15],
                chests: &[],
                markers: &[
                    TemplateMarker {
                        x: 2,
                        y: 1,
                        z: 7,
                        metadata: "Mage",
                    },
                    TemplateMarker {
                        x: 9,
                        y: 1,
                        z: 8,
                        metadata: "Warrior",
                    },
                    TemplateMarker {
                        x: 13,
                        y: 1,
                        z: 2,
                        metadata: "Warrior",
                    },
                ],
                randomizable_containers: &[],
            },
        );
        map.insert(
            "woodland_mansion/2x2_b4",
            TemplateContainerData {
                size: [15, 11, 15],
                chests: &[],
                markers: &[
                    TemplateMarker {
                        x: 3,
                        y: 1,
                        z: 3,
                        metadata: "Warrior",
                    },
                    TemplateMarker {
                        x: 3,
                        y: 1,
                        z: 8,
                        metadata: "Mage",
                    },
                    TemplateMarker {
                        x: 4,
                        y: 1,
                        z: 11,
                        metadata: "Warrior",
                    },
                ],
                randomizable_containers: &[],
            },
        );
        map.insert(
            "woodland_mansion/2x2_b5",
            TemplateContainerData {
                size: [15, 11, 15],
                chests: &[],
                markers: &[TemplateMarker {
                    x: 7,
                    y: 7,
                    z: 0,
                    metadata: "ChestSouth",
                }],
                randomizable_containers: &[],
            },
        );
        map
    });

pub fn get_template_container_data(template_name: &str) -> Option<&'static TemplateContainerData> {
    let normalized = template_name
        .strip_prefix("minecraft:")
        .unwrap_or(template_name);
    TEMPLATE_CONTAINERS.get(normalized)
}
