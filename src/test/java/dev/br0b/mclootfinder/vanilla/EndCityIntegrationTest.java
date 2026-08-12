package dev.br0b.mclootfinder.vanilla;

import dev.br0b.mclootfinder.core.Versions;
import net.minecraft.world.level.ChunkPos;
import org.junit.jupiter.api.Test;

import java.util.List;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertTrue;

class EndCityIntegrationTest {
    @Test
    void genericScannerFindsEndCityChests() {
        var spec = Versions.V26_1_2.endCity();
        try (VanillaRuntime26_1_2 runtime = VanillaRuntime26_1_2.load(0L)) {
            runtime.verifyStructureProfile(spec);
            var start = runtime.generateSelectedStructure(spec, new ChunkPos(86, 64));
            assertTrue(start.isValid());
            assertEquals(List.of(
                    "1390,106,1033,-8159403464680465500,0",
                    "1392,106,1031,7731847916610423894,0"
            ), StructureChestScanner.scan(0L, spec, start, runtime).stream()
                    .map(chest -> "%d,%d,%d,%d,%d".formatted(
                            chest.x(), chest.y(), chest.z(), chest.lootTableSeed(),
                            chest.containerOrdinalInDecorationChunk()
                    ))
                    .toList());
        }
    }
}
