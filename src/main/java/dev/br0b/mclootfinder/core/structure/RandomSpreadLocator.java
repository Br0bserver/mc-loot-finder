package dev.br0b.mclootfinder.core.structure;

import dev.br0b.mclootfinder.core.VersionProfile;
import dev.br0b.mclootfinder.core.StructureSpec;
import dev.br0b.mclootfinder.core.random.LegacyRandom48;

import java.util.ArrayList;
import java.util.Comparator;
import java.util.List;

/** Exact first-stage locator for vanilla random-spread structure sets. */
public final class RandomSpreadLocator {
    private static final long REGION_X_MULTIPLIER = 341_873_128_712L;
    private static final long REGION_Z_MULTIPLIER = 132_897_987_541L;

    private RandomSpreadLocator() {
    }

    public static List<StructureCandidate> locate(
            long worldSeed,
            int centerBlockX,
            int centerBlockZ,
            int radiusBlocks,
            VersionProfile.StructureProfile profile
    ) {
        if (radiusBlocks < 0) {
            throw new IllegalArgumentException("radius must be non-negative");
        }

        long minBlockX = (long) centerBlockX - radiusBlocks;
        long maxBlockX = (long) centerBlockX + radiusBlocks;
        long minBlockZ = (long) centerBlockZ - radiusBlocks;
        long maxBlockZ = (long) centerBlockZ + radiusBlocks;
        if (minBlockX < Integer.MIN_VALUE || maxBlockX > Integer.MAX_VALUE
                || minBlockZ < Integer.MIN_VALUE || maxBlockZ > Integer.MAX_VALUE) {
            throw new IllegalArgumentException(
                    "search area exceeds the supported block coordinate range"
            );
        }

        int minChunkX = (int) Math.floorDiv(minBlockX, 16L);
        int maxChunkX = (int) Math.floorDiv(maxBlockX, 16L);
        int minChunkZ = (int) Math.floorDiv(minBlockZ, 16L);
        int maxChunkZ = (int) Math.floorDiv(maxBlockZ, 16L);
        int minRegionX = Math.floorDiv(minChunkX, profile.spacing());
        int maxRegionX = Math.floorDiv(maxChunkX, profile.spacing());
        int minRegionZ = Math.floorDiv(minChunkZ, profile.spacing());
        int maxRegionZ = Math.floorDiv(maxChunkZ, profile.spacing());

        List<StructureCandidate> candidates = new ArrayList<>();
        for (int regionX = minRegionX; regionX <= maxRegionX; regionX++) {
            for (int regionZ = minRegionZ; regionZ <= maxRegionZ; regionZ++) {
                int limit = profile.spacing() - profile.separation();
                long placementSeed = worldSeed
                        + (long) regionX * REGION_X_MULTIPLIER
                        + (long) regionZ * REGION_Z_MULTIPLIER
                        + profile.salt();
                LegacyRandom48 random = new LegacyRandom48(placementSeed);
                int offsetX = random.nextInt(limit);
                int offsetZ;
                if (profile.spreadType() == VersionProfile.SpreadType.TRIANGULAR) {
                    offsetX = (offsetX + random.nextInt(limit)) / 2;
                    offsetZ = (random.nextInt(limit) + random.nextInt(limit)) / 2;
                } else {
                    offsetZ = random.nextInt(limit);
                }
                int chunkX = regionX * profile.spacing() + offsetX;
                int chunkZ = regionZ * profile.spacing() + offsetZ;
                long candidateBlockX = (long) chunkX * 16 + 8;
                long candidateBlockZ = (long) chunkZ * 16 + 8;
                long dx = candidateBlockX - centerBlockX;
                long dz = candidateBlockZ - centerBlockZ;
                long radiusSquared = (long) radiusBlocks * radiusBlocks;
                if (Math.abs(dx) <= radiusBlocks && Math.abs(dz) <= radiusBlocks
                        && dx * dx <= radiusSquared - dz * dz) {
                    long distanceSquared = dx * dx + dz * dz;
                    candidates.add(new StructureCandidate(
                            chunkX, chunkZ, (int) candidateBlockX, (int) candidateBlockZ,
                            distanceSquared
                    ));
                }
            }
        }
        candidates.sort(Comparator.comparingLong(StructureCandidate::squaredDistanceFromCenter));
        return List.copyOf(candidates);
    }

    public static List<StructureCandidate> locate(
            long worldSeed,
            int centerBlockX,
            int centerBlockZ,
            int radiusBlocks,
            StructureSpec spec
    ) {
        return locate(
                worldSeed,
                centerBlockX,
                centerBlockZ,
                radiusBlocks,
                spec.randomSpreadPlacement()
        );
    }
}
