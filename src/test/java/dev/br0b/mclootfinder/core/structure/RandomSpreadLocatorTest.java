package dev.br0b.mclootfinder.core.structure;

import dev.br0b.mclootfinder.core.Versions;
import net.minecraft.SharedConstants;
import net.minecraft.server.Bootstrap;
import net.minecraft.world.level.ChunkPos;
import net.minecraft.world.level.levelgen.structure.placement.RandomSpreadStructurePlacement;
import net.minecraft.world.level.levelgen.structure.placement.RandomSpreadType;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertEquals;

class RandomSpreadLocatorTest {
    @Test
    void candidateChunksMatchMinecraftPlacementImplementation() {
        SharedConstants.tryDetectVersion();
        Bootstrap.bootStrap();
        for (var profile : java.util.List.of(
                Versions.V26_1_2.ancientCity(),
                Versions.V26_1_2.bastionRemnant(),
                Versions.V26_1_2.desertPyramid(),
                Versions.V26_1_2.woodlandMansion())) {
            var minecraft = new RandomSpreadStructurePlacement(
                    profile.placement().spacing(), profile.placement().separation(),
                    profile.placement().spreadType() == dev.br0b.mclootfinder.core.VersionProfile.SpreadType.TRIANGULAR
                            ? RandomSpreadType.TRIANGULAR
                            : RandomSpreadType.LINEAR,
                    profile.placement().salt()
            );
            long[] seeds = {0L, 1L, -1L, 12_345_678_901_234L};
            for (long seed : seeds) {
                for (int sourceX = -1_000; sourceX <= 1_000; sourceX += 73) {
                    for (int sourceZ = -1_000; sourceZ <= 1_000; sourceZ += 91) {
                        ChunkPos expected = minecraft.getPotentialStructureChunk(seed, sourceX, sourceZ);
                        int centerX = expected.getMiddleBlockX();
                        int centerZ = expected.getMiddleBlockZ();
                        var result = RandomSpreadLocator.locate(
                                seed, centerX, centerZ, 0, profile
                        );
                        assertEquals(1, result.size());
                        assertEquals(expected.x(), result.getFirst().chunkX());
                        assertEquals(expected.z(), result.getFirst().chunkZ());
                    }
                }
            }
        }
    }
}
