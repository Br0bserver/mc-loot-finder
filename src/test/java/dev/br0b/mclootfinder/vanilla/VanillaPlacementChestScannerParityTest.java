package dev.br0b.mclootfinder.vanilla;

import dev.br0b.mclootfinder.core.StructureSpec;
import dev.br0b.mclootfinder.core.Versions;
import net.minecraft.world.level.ChunkPos;
import org.junit.jupiter.api.Test;

import java.util.Comparator;
import java.util.List;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertTrue;

class VanillaPlacementChestScannerParityTest {
    private static final Comparator<ChestPrediction> BY_POSITION = Comparator
            .comparingInt(ChestPrediction::x)
            .thenComparingInt(ChestPrediction::y)
            .thenComparingInt(ChestPrediction::z);

    @Test
    void genericPlacementBackendMatchesAncientCityFastScanner() {
        assertMatchesJigsawFastScanner(
                Versions.V26_1_2.ancientCity(), 50, 79
        );
    }

    @Test
    void genericPlacementBackendMatchesBastionFastScanner() {
        assertMatchesJigsawFastScanner(
                Versions.V26_1_2.bastionRemnant(), 11, -14
        );
    }

    @Test
    void genericPlacementBackendMatchesDesertPyramidRecipeScanner() {
        long worldSeed = 0L;
        StructureSpec spec = Versions.V26_1_2.desertPyramid();
        try (VanillaRuntime26_1_2 runtime = VanillaRuntime26_1_2.load(worldSeed)) {
            ChunkPos startChunk = new ChunkPos(0, -188);
            var recipeStart = runtime.generateSelectedStructure(spec, startChunk);
            var genericStart = runtime.generateSelectedStructure(spec, startChunk);
            assertTrue(recipeStart.isValid());
            assertTrue(genericStart.isValid());
            assertPredictionsEqual(
                    DesertPyramidChestScanner.scan(worldSeed, spec, recipeStart, runtime),
                    VanillaPlacementChestScanner.scan(worldSeed, spec, genericStart, runtime)
            );
        }
    }

    private static void assertMatchesJigsawFastScanner(
            StructureSpec spec,
            int startChunkX,
            int startChunkZ
    ) {
        long worldSeed = 0L;
        try (VanillaRuntime26_1_2 runtime = VanillaRuntime26_1_2.load(worldSeed)) {
            ChunkPos startChunk = new ChunkPos(startChunkX, startChunkZ);
            var fastStart = runtime.generateSelectedStructure(spec, startChunk);
            var genericStart = runtime.generateSelectedStructure(spec, startChunk);
            assertTrue(fastStart.isValid());
            assertTrue(genericStart.isValid());
            assertPredictionsEqual(
                    JigsawChestScanner.scan(
                            worldSeed, spec, fastStart, runtime.templateManager()
                    ),
                    VanillaPlacementChestScanner.scan(
                            worldSeed, spec, genericStart, runtime
                    )
            );
        }
    }

    private static void assertPredictionsEqual(
            List<ChestPrediction> expected,
            List<ChestPrediction> actual
    ) {
        assertEquals(
                expected.stream()
                        .filter(chest -> !chest.lootTable().isEmpty())
                        .sorted(BY_POSITION)
                        .toList(),
                actual.stream().sorted(BY_POSITION).toList()
        );
    }
}
