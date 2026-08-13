package dev.br0b.mclootfinder.vanilla;

import dev.br0b.mclootfinder.core.StructureSpec;
import dev.br0b.mclootfinder.core.Versions;
import net.minecraft.world.level.ChunkPos;
import org.junit.jupiter.api.Test;

import java.util.Set;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertTrue;

class VillageAndFortressIntegrationTest {
    @Test
    void netherFortressUsesTheOtherNetherComplexVariant() {
        assertStructure(
                Versions.V26_1_2.structure("nether_fortress"),
                15,
                2,
                6,
                Set.of("minecraft:chests/nether_bridge")
        );
    }

    @Test
    void villageFamilyUsesFastJigsawScannerForSelectedBiomeVariant() {
        assertStructure(
                Versions.V26_1_2.structure("village"),
                38,
                45,
                5,
                Set.of(
                        "minecraft:chests/village/village_cartographer",
                        "minecraft:chests/village/village_tannery",
                        "minecraft:chests/village/village_savanna_house"
                )
        );
    }

    @Test
    void overlappingOutpostTemplateWritesResolveToOnePhysicalChest() {
        assertStructure(
                Versions.V26_1_2.structure("pillager_outpost"),
                36,
                103,
                1,
                Set.of("minecraft:chests/pillager_outpost")
        );
    }

    private static void assertStructure(
            StructureSpec spec,
            int chunkX,
            int chunkZ,
            int expectedContainers,
            Set<String> expectedTables
    ) {
        try (VanillaRuntime26_1_2 runtime = VanillaRuntime26_1_2.load(0L)) {
            var start = runtime.generateSelectedStructure(spec, new ChunkPos(chunkX, chunkZ));
            assertTrue(start.isValid());
            var chests = StructureChestScanner.scan(0L, spec, start, runtime);
            assertEquals(expectedContainers, chests.size());
            assertEquals(expectedTables,
                    chests.stream().map(ChestPrediction::lootTable).collect(
                            java.util.stream.Collectors.toSet()
                    ));
        }
    }
}
