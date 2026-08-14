package dev.br0b.mclootfinder.vanilla;

import dev.br0b.mclootfinder.core.Versions;
import net.minecraft.world.level.ChunkPos;
import org.junit.jupiter.api.Test;

import java.util.List;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertTrue;

class BuriedTreasureIntegrationTest {
    @Test
    void genericScannerFindsSeedZeroBuriedTreasure() {
        var spec = Versions.V26_1_2.structure("buried_treasure");
        var chunk = new ChunkPos(0, -22);
        try (VanillaRuntime26_1_2 runtime = VanillaRuntime26_1_2.load(0L)) {
            assertTrue(runtime.isStructurePlacementChunk(spec, chunk));
            var start = runtime.generateSelectedStructure(spec, chunk);
            assertTrue(start.isValid());
            assertEquals(List.of(
                    "9,63,-343,minecraft:chests/buried_treasure,-2156648588641602659,0"
            ), StructureChestScanner.scan(0L, spec, start, runtime).stream()
                    .map(chest -> "%d,%d,%d,%s,%d,%d".formatted(
                            chest.x(), chest.y(), chest.z(), chest.lootTable(),
                            chest.lootTableSeed(), chest.containerOrdinalInDecorationChunk()
                    ))
                    .toList());
        }
    }
}
