package dev.br0b.mclootfinder.vanilla;

import dev.br0b.mclootfinder.core.Versions;
import net.minecraft.world.level.ChunkPos;
import org.junit.jupiter.api.Test;

import java.util.List;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertTrue;

class StrongholdIntegrationTest {
    @Test
    void concentricRingPlacementAndContainersMatchVanilla() {
        var spec = Versions.V26_1_2.stronghold();
        try (VanillaRuntime26_1_2 runtime = VanillaRuntime26_1_2.load(0L)) {
            runtime.verifyStructureProfile(spec);
            var candidates = runtime.locateConcentricRingCandidates(spec, 0, 0, 5_000);
            assertEquals(List.of(
                    "-13,-106", "-87,71", "125,57", "-218,190",
                    "278,96", "223,-203", "-298,-95"
            ), candidates.stream()
                    .map(candidate -> candidate.chunkX() + "," + candidate.chunkZ())
                    .toList());

            var first = candidates.getFirst();
            var start = runtime.generateSelectedStructure(
                    spec, new ChunkPos(first.chunkX(), first.chunkZ())
            );
            assertTrue(start.isValid());
            assertEquals(List.of(
                    "-211,-5,-1735,minecraft:chests/stronghold_library,-11939494983489694,0",
                    "-202,0,-1731,minecraft:chests/stronghold_library,3409615373764762369,0",
                    "-194,-6,-1724,minecraft:chests/stronghold_corridor,-2519506147567311482,0",
                    "-180,-6,-1738,minecraft:chests/stronghold_corridor,1748814651822569355,0",
                    "-185,-6,-1721,minecraft:chests/stronghold_corridor,-9121233803224987497,0",
                    "-187,-8,-1672,minecraft:chests/stronghold_corridor,-6572614333168395987,0",
                    "-171,-5,-1741,minecraft:chests/stronghold_library,4146347882151658670,0",
                    "-175,0,-1732,minecraft:chests/stronghold_library,-6674441815014145477,1",
                    "-164,-6,-1667,minecraft:chests/stronghold_crossing,-7261058392992991024,0"
            ), StructureChestScanner.scan(0L, spec, start, runtime).stream()
                    .map(chest -> "%d,%d,%d,%s,%d,%d".formatted(
                            chest.x(), chest.y(), chest.z(), chest.lootTable(),
                            chest.lootTableSeed(), chest.containerOrdinalInDecorationChunk()
                    ))
                    .toList());
        }
    }
}
