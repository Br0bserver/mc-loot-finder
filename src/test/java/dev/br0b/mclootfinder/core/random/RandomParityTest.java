package dev.br0b.mclootfinder.core.random;

import net.minecraft.util.RandomSource;
import net.minecraft.world.level.levelgen.LegacyRandomSource;
import net.minecraft.world.level.levelgen.WorldgenRandom;
import net.minecraft.world.level.levelgen.XoroshiroRandomSource;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertEquals;

class RandomParityTest {
    @Test
    void legacyRandomMatchesMinecraftForStructureOffsets() {
        long[] seeds = {0L, 1L, -1L, Long.MIN_VALUE, Long.MAX_VALUE, 9_876_543_210L};
        for (long seed : seeds) {
            LegacyRandom48 ours = new LegacyRandom48(seed);
            RandomSource minecraft = new LegacyRandomSource(seed);
            for (int index = 0; index < 100; index++) {
                assertEquals(minecraft.nextInt(16), ours.nextInt(16));
            }
        }
    }

    @Test
    void xoroshiroMatchesMinecraft() {
        long[] seeds = {0L, 1L, -1L, Long.MIN_VALUE, Long.MAX_VALUE, 9_876_543_210L};
        for (long seed : seeds) {
            Xoroshiro128PlusPlus ours = new Xoroshiro128PlusPlus(seed);
            RandomSource minecraft = new XoroshiroRandomSource(seed);
            for (int index = 0; index < 100; index++) {
                assertEquals(minecraft.nextLong(), ours.nextLong());
            }
        }
    }

    @Test
    void decorationAndContainerSeedsMatchMinecraft() {
        long worldSeed = -7_712_345_678_901_234L;
        int chunkX = -137;
        int chunkZ = 89;
        int structureIndex = 0;
        int step = 7;

        WorldgenRandom minecraft = new WorldgenRandom(new XoroshiroRandomSource(0L));
        long decorationSeed = minecraft.setDecorationSeed(worldSeed, chunkX * 16, chunkZ * 16);
        minecraft.setFeatureSeed(decorationSeed, structureIndex, step);

        for (int ordinal = 0; ordinal < 20; ordinal++) {
            assertEquals(
                    minecraft.nextLong(),
                    DecorationRandom.containerLootSeed(
                            worldSeed, chunkX, chunkZ, structureIndex, step, ordinal
                    )
            );
        }
    }

    @Test
    void decorationNextIntAndFollowingLongsMatchMinecraft() {
        long[] seeds = {0L, 1L, -1L, Long.MIN_VALUE, Long.MAX_VALUE};
        int[][] chunks = {{0, 0}, {-188, 0}, {77, -213}, {-137, 89}};
        for (long worldSeed : seeds) {
            for (int[] chunk : chunks) {
                WorldgenRandom minecraft = new WorldgenRandom(new XoroshiroRandomSource(0L));
                long vanillaDecorationSeed = minecraft.setDecorationSeed(
                        worldSeed, chunk[0] * 16, chunk[1] * 16
                );
                minecraft.setFeatureSeed(vanillaDecorationSeed, 1, 4);

                DecorationRandom ours = new DecorationRandom();
                long ourDecorationSeed = ours.setDecorationSeed(
                        worldSeed, chunk[0] * 16, chunk[1] * 16
                );
                ours.setFeatureSeed(ourDecorationSeed, 1, 4);

                assertEquals(minecraft.nextInt(3), ours.nextInt(3));
                for (int index = 0; index < 8; index++) {
                    assertEquals(minecraft.nextLong(), ours.nextLong());
                }
            }
        }
    }
}
