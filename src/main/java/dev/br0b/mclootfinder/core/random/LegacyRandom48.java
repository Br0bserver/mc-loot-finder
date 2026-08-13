package dev.br0b.mclootfinder.core.random;

/** Minimal java.util.Random-compatible 48-bit LCG used by structure placement. */
public final class LegacyRandom48 {
    private static final long MULTIPLIER = 0x5DEECE66DL;
    private static final long ADDEND = 0xBL;
    private static final long MASK = (1L << 48) - 1;

    private long seed;

    public LegacyRandom48(long seed) {
        setSeed(seed);
    }

    public void setSeed(long seed) {
        this.seed = (seed ^ MULTIPLIER) & MASK;
    }

    public int next(int bits) {
        seed = (seed * MULTIPLIER + ADDEND) & MASK;
        return (int) (seed >>> (48 - bits));
    }

    public int nextInt(int bound) {
        if (bound <= 0) {
            throw new IllegalArgumentException("bound must be positive");
        }
        if ((bound & -bound) == bound) {
            return (int) ((bound * (long) next(31)) >> 31);
        }

        int bits;
        int value;
        do {
            bits = next(31);
            value = bits % bound;
        } while (bits - value + (bound - 1) < 0);
        return value;
    }

    public long nextLong() {
        return ((long) next(32) << 32) + next(32);
    }

    public float nextFloat() {
        return next(24) * 0x1.0p-24f;
    }
}
