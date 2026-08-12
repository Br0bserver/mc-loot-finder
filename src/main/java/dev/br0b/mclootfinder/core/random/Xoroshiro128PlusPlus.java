package dev.br0b.mclootfinder.core.random;

/** Minecraft's xoroshiro128++ source, including its legacy long-seed expansion. */
public final class Xoroshiro128PlusPlus {
    private static final long GOLDEN_RATIO_64 = -7_046_029_254_386_353_131L;
    private static final long SILVER_RATIO_64 = 7_640_891_576_956_012_809L;
    private static final long STAFFORD_MIX_1 = -4_658_895_280_553_007_687L;
    private static final long STAFFORD_MIX_2 = -7_723_592_293_110_705_685L;

    private long seedLo;
    private long seedHi;

    public Xoroshiro128PlusPlus(long seed) {
        setSeed(seed);
    }

    public void setSeed(long seed) {
        long lowBits = seed ^ SILVER_RATIO_64;
        long highBits = lowBits + GOLDEN_RATIO_64;
        this.seedLo = mixStafford13(lowBits);
        this.seedHi = mixStafford13(highBits);
        if ((seedLo | seedHi) == 0L) {
            seedLo = GOLDEN_RATIO_64;
            seedHi = SILVER_RATIO_64;
        }
    }

    public long nextLong() {
        long s0 = seedLo;
        long s1 = seedHi;
        long result = Long.rotateLeft(s0 + s1, 17) + s0;
        s1 ^= s0;
        seedLo = Long.rotateLeft(s0, 49) ^ s1 ^ (s1 << 21);
        seedHi = Long.rotateLeft(s1, 28);
        return result;
    }

    static long mixStafford13(long value) {
        value = (value ^ (value >>> 30)) * STAFFORD_MIX_1;
        value = (value ^ (value >>> 27)) * STAFFORD_MIX_2;
        return value ^ (value >>> 31);
    }
}

