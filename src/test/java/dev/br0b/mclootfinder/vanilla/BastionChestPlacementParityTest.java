package dev.br0b.mclootfinder.vanilla;

import dev.br0b.mclootfinder.core.Versions;
import net.minecraft.world.level.ChunkPos;
import org.junit.jupiter.api.Test;

import java.util.HashSet;
import java.util.List;
import java.util.Set;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertTrue;

class BastionChestPlacementParityTest {
    @Test
    void scannerMatchesVanillaPlacementAcrossAllBastionLootTableFamilies() {
        long worldSeed = 0L;
        var spec = Versions.V26_1_2.bastionRemnant();
        Set<String> seenTables = new HashSet<>();
        int chestCount = 0;

        try (VanillaRuntime26_1_2 runtime = VanillaRuntime26_1_2.load(worldSeed)) {
            runtime.verifyStructureProfile(spec);
            List<ChunkPos> starts = List.of(
                    new ChunkPos(11, -14),   // bridge
                    new ChunkPos(-27, -10), // hoglin stable
                    new ChunkPos(62, 32)    // treasure
            );
            for (ChunkPos startChunk : starts) {
                var start = runtime.generateSelectedStructure(spec, startChunk);
                assertTrue(start.isValid(), "bastion test vector drifted: " + startChunk);
                List<ChestPrediction> predictions = JigsawChestScanner.scan(
                        worldSeed, spec, start, runtime.templateManager()
                );
                chestCount += predictions.size();
                predictions.stream().map(ChestPrediction::lootTable).forEach(seenTables::add);
                AncientCityChestPlacementParityTest.assertScannerMatchesPlacement(
                        runtime, worldSeed, spec, start, predictions
                );
            }
        }

        assertEquals(20, chestCount);
        assertEquals(Set.copyOf(spec.lootTables()), seenTables);
    }
}
