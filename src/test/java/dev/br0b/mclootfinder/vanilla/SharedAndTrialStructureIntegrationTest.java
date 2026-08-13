package dev.br0b.mclootfinder.vanilla;

import dev.br0b.mclootfinder.core.Versions;
import net.minecraft.world.level.ChunkPos;
import org.junit.jupiter.api.Test;

import java.util.Set;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertTrue;

class SharedAndTrialStructureIntegrationTest {
    @Test
    void ruinedPortalFamilySelectsAndPlacesTheActualVariant() {
        var spec = Versions.V26_1_2.structure("ruined_portal");
        try (VanillaRuntime26_1_2 runtime = VanillaRuntime26_1_2.load(0L)) {
            var start = runtime.generateSelectedStructure(spec, new ChunkPos(-22, 6));
            assertTrue(start.isValid());
            var chest = StructureChestScanner.scan(0L, spec, start, runtime).getFirst();
            assertEquals("-352,55,100,minecraft:chests/ruined_portal,-6371263386669125558",
                    "%d,%d,%d,%s,%d".formatted(
                            chest.x(), chest.y(), chest.z(), chest.lootTable(),
                            chest.lootTableSeed()
                    ));
        }
    }

    @Test
    void trialChambersExposeEveryRandomizableCarrier() {
        var spec = Versions.V26_1_2.structure("trial_chambers");
        try (VanillaRuntime26_1_2 runtime = VanillaRuntime26_1_2.load(0L)) {
            var start = runtime.generateSelectedStructure(spec, new ChunkPos(14, 5));
            assertTrue(start.isValid());
            var containers = StructureChestScanner.scan(0L, spec, start, runtime);
            assertEquals(95, containers.size());
            assertTrue(containers.stream().anyMatch(chest ->
                    chest.lootTable().equals("minecraft:chests/trial_chambers/reward")));
            assertTrue(containers.stream().anyMatch(chest ->
                    chest.lootTable().startsWith("minecraft:dispensers/trial_chambers/")));
            assertTrue(containers.stream().anyMatch(chest ->
                    chest.lootTable().startsWith("minecraft:pots/trial_chambers/")));
            assertTrue(containers.stream()
                    .map(ChestPrediction::lootTable)
                    .allMatch(spec.lootTables()::contains));
        }
    }
}
