package dev.br0b.mclootfinder.vanilla;

import dev.br0b.mclootfinder.core.StructureSpec;
import dev.br0b.mclootfinder.core.Versions;
import net.minecraft.world.level.ChunkPos;
import org.junit.jupiter.api.Test;

import java.util.List;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertTrue;

class MarineStructureIntegrationTest {
    @Test
    void shipwreckFamilyUsesSelectedVariant() {
        assertVector(
                Versions.V26_1_2.structure("shipwreck"),
                14,
                8,
                List.of(
                        "219,60,142,minecraft:chests/shipwreck_treasure,8114931824729312727",
                        "235,61,144,minecraft:chests/shipwreck_supply,-3774492170699737302",
                        "224,61,145,minecraft:chests/shipwreck_map,-2986182992758690057"
                )
        );
    }

    @Test
    void oceanRuinFamilyUsesSelectedVariant() {
        assertVector(
                Versions.V26_1_2.structure("ocean_ruin"),
                -16,
                -31,
                List.of(
                        "-258,63,-497,minecraft:chests/underwater_ruin_small,5505147133889129545"
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
            var start = runtime.generateSelectedStructure(
                    spec, new ChunkPos(startChunkX, startChunkZ)
            );
            assertTrue(start.isValid());
            assertEquals(expected, StructureChestScanner.scan(0L, spec, start, runtime).stream()
                    .map(chest -> "%d,%d,%d,%s,%d".formatted(
                            chest.x(), chest.y(), chest.z(), chest.lootTable(), chest.lootTableSeed()
                    )).toList());
        }
    }
}
