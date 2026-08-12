package dev.br0b.mclootfinder.core;

public final class Versions {
    /**
     * Values extracted from the vanilla 26.1.2 data pack and runtime registry
     * for supported structures, their structure sets, and loot tables.
     */
    public static final VersionProfile V26_1_2 = new VersionProfile(
            "26.1.2",
            java.util.List.of(
                new StructureSpec(
                    "ancient_city",
                    "minecraft:ancient_city",
                    "minecraft:overworld",
                    new VersionProfile.StructureProfile(24, 8, 20_083_232),
                    7,
                    0,
                    StructureSpec.ScannerKind.JIGSAW_FAST,
                    StructureSpec.ContainerSeedShortcut.DIRECT,
                    java.util.List.of(new StructureSpec.SelectionEntry("minecraft:ancient_city", 1)),
                    java.util.List.of("minecraft:chests/ancient_city"),
                    "minecraft:silence_armor_trim_smithing_template"
                ),
                new StructureSpec(
                    "bastion_remnant",
                    "minecraft:bastion_remnant",
                    "minecraft:the_nether",
                    new VersionProfile.StructureProfile(27, 4, 30_084_232),
                    4,
                    0,
                    StructureSpec.ScannerKind.JIGSAW_FAST,
                    StructureSpec.ContainerSeedShortcut.DIRECT,
                    java.util.List.of(
                            new StructureSpec.SelectionEntry("minecraft:fortress", 2, false),
                            new StructureSpec.SelectionEntry("minecraft:bastion_remnant", 3)
                    ),
                    java.util.List.of(
                            "minecraft:chests/bastion_bridge",
                            "minecraft:chests/bastion_hoglin_stable",
                            "minecraft:chests/bastion_other",
                            "minecraft:chests/bastion_treasure"
                    ),
                    "minecraft:netherite_upgrade_smithing_template"
                ),
                new StructureSpec(
                    "desert_pyramid",
                    "minecraft:desert_pyramid",
                    "minecraft:overworld",
                    new VersionProfile.StructureProfile(32, 8, 14_357_617),
                    4,
                    1,
                    StructureSpec.ScannerKind.VANILLA_PLACEMENT,
                    StructureSpec.ContainerSeedShortcut.DESERT_PYRAMID,
                    java.util.List.of(new StructureSpec.SelectionEntry("minecraft:desert_pyramid", 1)),
                    java.util.List.of("minecraft:chests/desert_pyramid"),
                    "minecraft:dune_armor_trim_smithing_template"
                ),
                new StructureSpec(
                    "jungle_pyramid",
                    "minecraft:jungle_pyramid",
                    "minecraft:overworld",
                    new VersionProfile.StructureProfile(32, 8, 14_357_619),
                    4,
                    4,
                    StructureSpec.ScannerKind.VANILLA_PLACEMENT,
                    StructureSpec.ContainerSeedShortcut.NONE,
                    java.util.List.of(new StructureSpec.SelectionEntry("minecraft:jungle_pyramid", 1)),
                    java.util.List.of(
                            "minecraft:chests/jungle_temple",
                            "minecraft:chests/jungle_temple_dispenser"
                    ),
                    "minecraft:wild_armor_trim_smithing_template"
                ),
                new StructureSpec(
                    "igloo",
                    "minecraft:igloo",
                    "minecraft:overworld",
                    new VersionProfile.StructureProfile(32, 8, 14_357_618),
                    4,
                    3,
                    StructureSpec.ScannerKind.VANILLA_PLACEMENT,
                    StructureSpec.ContainerSeedShortcut.NONE,
                    java.util.List.of(new StructureSpec.SelectionEntry("minecraft:igloo", 1)),
                    java.util.List.of("minecraft:chests/igloo_chest"),
                    "minecraft:golden_apple"
                ),
                new StructureSpec(
                    "end_city",
                    "minecraft:end_city",
                    "minecraft:the_end",
                    new VersionProfile.StructureProfile(
                            20,
                            11,
                            10_387_313,
                            VersionProfile.SpreadType.TRIANGULAR
                    ),
                    4,
                    2,
                    StructureSpec.ScannerKind.VANILLA_PLACEMENT,
                    StructureSpec.ContainerSeedShortcut.NONE,
                    java.util.List.of(new StructureSpec.SelectionEntry("minecraft:end_city", 1)),
                    java.util.List.of("minecraft:chests/end_city_treasure"),
                    "minecraft:spire_armor_trim_smithing_template"
                ),
                new StructureSpec(
                    "ruined_portal",
                    "minecraft:ruined_portal",
                    "minecraft:overworld",
                    new VersionProfile.StructureProfile(40, 15, 34_222_645),
                    -1,
                    -1,
                    StructureSpec.ScannerKind.VANILLA_PLACEMENT,
                    StructureSpec.ContainerSeedShortcut.NONE,
                    ruinedPortalEntries(),
                    java.util.List.of("minecraft:chests/ruined_portal"),
                    "minecraft:enchanted_golden_apple"
                ),
                new StructureSpec(
                    "ruined_portal_nether",
                    "minecraft:ruined_portal_nether",
                    "minecraft:the_nether",
                    new VersionProfile.StructureProfile(40, 15, 34_222_645),
                    -1,
                    -1,
                    StructureSpec.ScannerKind.VANILLA_PLACEMENT,
                    StructureSpec.ContainerSeedShortcut.NONE,
                    ruinedPortalEntries(),
                    java.util.List.of("minecraft:chests/ruined_portal"),
                    "minecraft:enchanted_golden_apple"
                ),
                new StructureSpec(
                    "trial_chambers",
                    "minecraft:trial_chambers",
                    "minecraft:overworld",
                    new VersionProfile.StructureProfile(34, 12, 94_251_327),
                    -1,
                    -1,
                    StructureSpec.ScannerKind.VANILLA_PLACEMENT,
                    StructureSpec.ContainerSeedShortcut.NONE,
                    java.util.List.of(new StructureSpec.SelectionEntry("minecraft:trial_chambers", 1)),
                    java.util.List.of(
                            "minecraft:chests/trial_chambers/corridor",
                            "minecraft:chests/trial_chambers/entrance",
                            "minecraft:chests/trial_chambers/intersection",
                            "minecraft:chests/trial_chambers/intersection_barrel",
                            "minecraft:chests/trial_chambers/supply"
                    ),
                    "minecraft:trial_key"
                ),
                new StructureSpec(
                    "woodland_mansion",
                    "minecraft:mansion",
                    "minecraft:overworld",
                    new VersionProfile.StructureProfile(
                            80,
                            20,
                            10_387_319,
                            VersionProfile.SpreadType.TRIANGULAR
                    ),
                    4,
                    5,
                    StructureSpec.ScannerKind.VANILLA_PLACEMENT,
                    StructureSpec.ContainerSeedShortcut.NONE,
                    java.util.List.of(new StructureSpec.SelectionEntry("minecraft:mansion", 1)),
                    java.util.List.of("minecraft:chests/woodland_mansion"),
                    "minecraft:vex_armor_trim_smithing_template"
                )
            )
    );

    private Versions() {
    }

    private static java.util.List<StructureSpec.SelectionEntry> ruinedPortalEntries() {
        return java.util.List.of(
                new StructureSpec.SelectionEntry("minecraft:ruined_portal", 1),
                new StructureSpec.SelectionEntry("minecraft:ruined_portal_desert", 1),
                new StructureSpec.SelectionEntry("minecraft:ruined_portal_jungle", 1),
                new StructureSpec.SelectionEntry("minecraft:ruined_portal_swamp", 1),
                new StructureSpec.SelectionEntry("minecraft:ruined_portal_mountain", 1),
                new StructureSpec.SelectionEntry("minecraft:ruined_portal_ocean", 1),
                new StructureSpec.SelectionEntry("minecraft:ruined_portal_nether", 1)
        );
    }

    public static VersionProfile require(String version) {
        if (V26_1_2.minecraftVersion().equals(version)) {
            return V26_1_2;
        }
        throw new IllegalArgumentException(
                "Unsupported Minecraft version: " + version + "; supported: 26.1.2"
        );
    }
}
