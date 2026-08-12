package dev.br0b.mclootfinder.core.random;

/** Reproduces the random stream supplied to structures during biome decoration. */
public final class DecorationRandom {
    private final Xoroshiro128PlusPlus random = new Xoroshiro128PlusPlus(0L);

    public long setDecorationSeed(long worldSeed, int blockX, int blockZ) {
        random.setSeed(worldSeed);
        long xMultiplier = nextLong() | 1L;
        long zMultiplier = nextLong() | 1L;
        long decorationSeed = ((long) blockX * xMultiplier + (long) blockZ * zMultiplier) ^ worldSeed;
        random.setSeed(decorationSeed);
        return decorationSeed;
    }

    public void setFeatureSeed(long decorationSeed, int featureIndex, int decorationStep) {
        random.setSeed(decorationSeed + featureIndex + 10_000L * decorationStep);
    }

    public long nextLong() {
        // WorldgenRandom implements BitRandomSource. When its delegate is not a
        // LegacyRandomSource, each next(32) consumes one xoroshiro long and keeps
        // its high 32 bits; BitRandomSource.nextLong() then combines two calls.
        int high = (int) (random.nextLong() >>> 32);
        int low = (int) (random.nextLong() >>> 32);
        return ((long) high << 32) + low;
    }

    /** Matches BitRandomSource.nextInt(bound) over WorldgenRandom's xoroshiro delegate. */
    public int nextInt(int bound) {
        if (bound <= 0) {
            throw new IllegalArgumentException("bound must be positive");
        }
        if ((bound & (bound - 1)) == 0) {
            return (int) ((bound * (long) nextBits(31)) >> 31);
        }
        int bits;
        int value;
        do {
            bits = nextBits(31);
            value = bits % bound;
        } while (bits - value + (bound - 1) < 0);
        return value;
    }

    private int nextBits(int bits) {
        return (int) (random.nextLong() >>> (64 - bits));
    }

    public static long containerLootSeed(
            long worldSeed,
            int chunkX,
            int chunkZ,
            int structureIndex,
            int decorationStep,
            int containerOrdinal
    ) {
        if (containerOrdinal < 0) {
            throw new IllegalArgumentException("container ordinal must be non-negative");
        }
        DecorationRandom random = new DecorationRandom();
        long decorationSeed = random.setDecorationSeed(worldSeed, chunkX * 16, chunkZ * 16);
        random.setFeatureSeed(decorationSeed, structureIndex, decorationStep);
        long result = 0L;
        for (int index = 0; index <= containerOrdinal; index++) {
            result = random.nextLong();
        }
        return result;
    }
}
