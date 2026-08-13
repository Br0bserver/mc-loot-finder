package dev.br0b.mclootfinder.vanilla;

import dev.br0b.mclootfinder.core.Versions;
import dev.br0b.mclootfinder.core.structure.RandomSpreadLocator;
import net.minecraft.world.level.ChunkPos;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertFalse;
import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.fail;

class WoodlandMansionIntegrationTest {
    @Test
    void profileMatchesVanillaRegistry() {
        try (VanillaRuntime26_1_2 runtime = VanillaRuntime26_1_2.load(0L)) {
            runtime.verifyStructureProfile(Versions.V26_1_2.woodlandMansion());
        }
    }

    @Test
    void genericScannerFindsASeedZeroMansion() {
        long worldSeed = 0L;
        var spec = Versions.V26_1_2.woodlandMansion();
        try (VanillaRuntime26_1_2 runtime = VanillaRuntime26_1_2.load(worldSeed)) {
            runtime.verifyStructureProfile(spec);
            for (var candidate : RandomSpreadLocator.locate(
                    worldSeed, 0, 0, 4_000, spec
            )) {
                var start = runtime.generateSelectedStructure(
                        spec, new ChunkPos(candidate.chunkX(), candidate.chunkZ())
                );
                if (!start.isValid()) {
                    continue;
                }
                var chests = StructureChestScanner.scan(
                        worldSeed, spec, start, runtime
                );
                assertFalse(chests.isEmpty(), "valid mansion contained no captured chests");
                assertEquals(new ChunkPos(-221, -52), start.getChunkPos());
                assertEquals(java.util.List.of(
                        "-3534,69,-817,901766045902888527,0",
                        "-3507,69,-820,-8848498207950452855,0",
                        "-3510,91,-814,-4018319632420834225,8",
                        "-3510,91,-802,-6821191583928121953,9",
                        "-3510,91,-798,-5138943431233530681,0",
                        "-3510,91,-786,-109245569350193768,1",
                        "-3509,66,-774,3860057794113135919,1"
                ), chests.stream().map(chest -> "%d,%d,%d,%d,%d".formatted(
                        chest.x(), chest.y(), chest.z(), chest.lootTableSeed(),
                        chest.containerOrdinalInDecorationChunk()
                )).toList());
                assertEquals(java.util.List.of("minecraft:chests/woodland_mansion"),
                        chests.stream().map(ChestPrediction::lootTable).distinct().toList());
                return;
            }
        }
        fail("no valid seed-zero mansion within 4000 blocks");
    }
}
