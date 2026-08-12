package dev.br0b.mclootfinder.vanilla;

import dev.br0b.mclootfinder.core.StructureSpec;
import dev.br0b.mclootfinder.core.Versions;
import net.minecraft.world.level.ChunkPos;
import org.junit.jupiter.api.Test;

import java.util.List;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertTrue;

class AdditionalStructureIntegrationTest {
    @Test
    void junglePyramidUsesGenericVanillaPlacement() {
        assertVector(
                Versions.V26_1_2.junglePyramid(),
                8,
                69,
                List.of(
                        "131,69,1117,minecraft:chests/jungle_temple_dispenser,5521895672757389794,0",
                        "137,69,1115,minecraft:chests/jungle_temple_dispenser,-3938261896625959214,1",
                        "136,68,1115,minecraft:chests/jungle_temple,-9141150294435319860,2",
                        "137,68,1108,minecraft:chests/jungle_temple,-6664571285535009793,3"
                )
        );
    }

    @Test
    void iglooBasementUsesGenericVanillaPlacement() {
        assertVector(
                Versions.V26_1_2.igloo(),
                98,
                192,
                List.of(
                        "1569,122,3076,minecraft:chests/igloo_chest,-7862992963971781551,0"
                )
        );
    }

    private static void assertVector(
            StructureSpec spec,
            int startChunkX,
            int startChunkZ,
            List<String> expected
    ) {
        try (VanillaRuntime26_1_2 runtime = VanillaRuntime26_1_2.load(0L)) {
            runtime.verifyStructureProfile(spec);
            var start = runtime.generateSelectedStructure(
                    spec, new ChunkPos(startChunkX, startChunkZ)
            );
            assertTrue(start.isValid());
            assertEquals(expected, StructureChestScanner.scan(0L, spec, start, runtime).stream()
                    .map(chest -> "%d,%d,%d,%s,%d,%d".formatted(
                            chest.x(), chest.y(), chest.z(), chest.lootTable(),
                            chest.lootTableSeed(), chest.containerOrdinalInDecorationChunk()
                    ))
                    .toList());
        }
    }
}
