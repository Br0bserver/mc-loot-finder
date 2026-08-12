package dev.br0b.mclootfinder.core;

public final class Versions {
    /**
     * Values extracted from the vanilla 26.1.2 data pack and runtime registry
     * for ancient cities, Bastion Remnants, desert pyramids, their structure
     * sets, and loot tables.
     */
    public static final VersionProfile V26_1_2 = new VersionProfile(
            "26.1.2",
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
                            new StructureSpec.SelectionEntry("minecraft:fortress", 2),
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
    );

    private Versions() {
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
