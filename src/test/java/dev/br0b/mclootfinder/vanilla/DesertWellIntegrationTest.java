package dev.br0b.mclootfinder.vanilla;

import dev.br0b.mclootfinder.core.Versions;
import org.junit.jupiter.api.Test;

import java.util.List;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertTrue;

class DesertWellIntegrationTest {
    @Test
    void desertWellSuspiciousSandMatchesVanillaPlacedFeature() {
        var spec = Versions.V26_1_2.desertWell();
        try (VanillaRuntime26_1_2 runtime = VanillaRuntime26_1_2.load(0L)) {
            runtime.verifyPlacedFeatureProfile(spec);
            var candidates = VanillaPlacedFeatureScanner.locateCandidates(
                    0L, spec, runtime, -620, -2460, 32
            );
            assertEquals(1, candidates.size());
            assertEquals(
                    "-39,-154,-621,-2460",
                    "%d,%d,%d,%d".formatted(
                            candidates.getFirst().chunkX(), candidates.getFirst().chunkZ(),
                            candidates.getFirst().blockX(), candidates.getFirst().blockZ()
                    )
            );

            var scan = VanillaPlacedFeatureScanner.scan(0L, spec, runtime, -39, -154);
            assertTrue(scan.validStructure());
            assertEquals(
                    List.of(
                            "-622,64,-2460,minecraft:suspicious_sand,minecraft:archaeology/desert_well,-170699190288320",
                            "-621,63,-2460,minecraft:suspicious_sand,minecraft:archaeology/desert_well,-170424312381377"
                    ),
                    scan.containers().stream().map(source -> "%d,%d,%d,%s,%s,%d".formatted(
                            source.x(), source.y(), source.z(), source.sourceBlock(),
                            source.lootTable(), source.lootTableSeed()
                    )).toList()
            );
        }
    }
}
