package dev.br0b.mclootfinder.vanilla;

import dev.br0b.mclootfinder.core.StructureSpec;
import dev.br0b.mclootfinder.core.Versions;
import net.minecraft.world.level.ChunkPos;
import org.junit.jupiter.api.Test;

import java.util.List;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertTrue;

class ArchaeologyIntegrationTest {
    @Test
    void desertPyramidSuspiciousSandMatchesVanillaPlacement() {
        assertVector(
                Versions.V26_1_2.desertPyramid(),
                0,
                -188,
                List.of(
                        "14,70,-3000,minecraft:suspicious_sand,minecraft:archaeology/desert_pyramid,4123156316230",
                        "15,67,-3001,minecraft:suspicious_sand,minecraft:archaeology/desert_pyramid,4398034219075",
                        "13,67,-3001,minecraft:suspicious_sand,minecraft:archaeology/desert_pyramid,3848278405187",
                        "15,68,-3002,minecraft:suspicious_sand,minecraft:archaeology/desert_pyramid,4398034214980",
                        "17,67,-3002,minecraft:suspicious_sand,minecraft:archaeology/desert_pyramid,4947790028867",
                        "17,67,-3003,minecraft:suspicious_sand,minecraft:archaeology/desert_pyramid,4947790024771"
                )
        );
    }

    @Test
    void oceanRuinSuspiciousGravelMatchesVanillaProcessorOutput() {
        assertVector(
                Versions.V26_1_2.structure("ocean_ruin"),
                -16,
                -31,
                List.of(
                        "-260,63,-502,minecraft:suspicious_gravel,minecraft:archaeology/ocean_ruin_cold,5956601882311804918",
                        "-257,63,-499,minecraft:suspicious_gravel,minecraft:archaeology/ocean_ruin_cold,5348657930804334674",
                        "-259,63,-500,minecraft:suspicious_gravel,minecraft:archaeology/ocean_ruin_cold,709691931556411661",
                        "-257,63,-496,minecraft:suspicious_gravel,minecraft:archaeology/ocean_ruin_cold,5424688650095754921",
                        "-260,63,-496,minecraft:suspicious_gravel,minecraft:archaeology/ocean_ruin_cold,158201479976614800",
                        "-256,63,-499,minecraft:suspicious_gravel,minecraft:archaeology/ocean_ruin_cold,8952214779475525344"
                )
        );
    }

    @Test
    void warmOceanRuinUsesSuspiciousSandAndWarmLoot() {
        var spec = Versions.V26_1_2.structure("ocean_ruin");
        try (VanillaRuntime26_1_2 runtime = VanillaRuntime26_1_2.load(0L)) {
            var start = runtime.generateSelectedStructure(spec, new ChunkPos(28, 21));
            assertTrue(start.isValid());
            var sources = StructureChestScanner.scanAll(0L, spec, start, runtime).stream()
                    .filter(source -> source.sourceKind()
                            == ChestPrediction.LootSourceKind.ARCHAEOLOGY)
                    .toList();
            assertEquals(40, sources.size());
            assertTrue(sources.stream().allMatch(source ->
                    source.sourceBlock().equals("minecraft:suspicious_sand")
                            && source.lootTable().equals(
                                    "minecraft:archaeology/ocean_ruin_warm"
                            )
            ));
            assertEquals(
                    "451,63,337,563775844101600967",
                    "%d,%d,%d,%d".formatted(
                            sources.getFirst().x(), sources.getFirst().y(),
                            sources.getFirst().z(), sources.getFirst().lootTableSeed()
                    )
            );
        }
    }

    @Test
    void trailRuinsSuspiciousGravelMatchesVanillaProcessorOutput() {
        var spec = Versions.V26_1_2.trailRuins();
        try (VanillaRuntime26_1_2 runtime = VanillaRuntime26_1_2.load(0L)) {
            var start = runtime.generateSelectedStructure(spec, new ChunkPos(38, -27));
            assertTrue(start.isValid());
            var sources = StructureChestScanner.scanAll(0L, spec, start, runtime).stream()
                    .filter(source -> source.sourceKind()
                            == ChestPrediction.LootSourceKind.ARCHAEOLOGY)
                    .toList();
            assertEquals(99, sources.size());
            assertTrue(sources.stream().allMatch(source ->
                    source.sourceBlock().equals("minecraft:suspicious_gravel")
            ));
            assertEquals(
                    java.util.Set.of(
                            "minecraft:archaeology/trail_ruins_common",
                            "minecraft:archaeology/trail_ruins_rare"
                    ),
                    sources.stream().map(ChestPrediction::lootTable)
                            .collect(java.util.stream.Collectors.toSet())
            );
            assertEquals(
                    List.of(
                            "608,51,-422,minecraft:archaeology/trail_ruins_common,-554345753514548585",
                            "610,51,-424,minecraft:archaeology/trail_ruins_common,-6543980276395535317",
                            "609,52,-425,minecraft:archaeology/trail_ruins_rare,7169779876079930898"
                    ),
                    sources.subList(0, 3).stream().map(source -> "%d,%d,%d,%s,%d".formatted(
                            source.x(), source.y(), source.z(), source.lootTable(),
                            source.lootTableSeed()
                    )).toList()
            );
            assertEquals(
                    List.of(
                            "640,50,-434,minecraft:archaeology/trail_ruins_common,9193263046726126964",
                            "644,47,-435,minecraft:archaeology/trail_ruins_common,-2236894430893853836",
                            "646,47,-433,minecraft:archaeology/trail_ruins_common,-4428564151409048691"
                    ),
                    sources.subList(sources.size() - 3, sources.size()).stream()
                            .map(source -> "%d,%d,%d,%s,%d".formatted(
                                    source.x(), source.y(), source.z(), source.lootTable(),
                                    source.lootTableSeed()
                            )).toList()
            );
        }
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
            assertEquals(expected, StructureChestScanner.scanAll(0L, spec, start, runtime).stream()
                    .filter(source -> source.sourceKind()
                            == ChestPrediction.LootSourceKind.ARCHAEOLOGY)
                    .map(source -> "%d,%d,%d,%s,%s,%d".formatted(
                            source.x(), source.y(), source.z(), source.sourceBlock(),
                            source.lootTable(), source.lootTableSeed()
                    ))
                    .toList());
        }
    }
}
