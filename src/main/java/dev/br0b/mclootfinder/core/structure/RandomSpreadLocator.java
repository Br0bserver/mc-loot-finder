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

        int minChunkX = Math.floorDiv(centerBlockX - radiusBlocks, 16);
        int maxChunkX = Math.floorDiv(centerBlockX + radiusBlocks, 16);
        int minChunkZ = Math.floorDiv(centerBlockZ - radiusBlocks, 16);
        int maxChunkZ = Math.floorDiv(centerBlockZ + radiusBlocks, 16);
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
                int blockX = chunkX * 16 + 8;
                int blockZ = chunkZ * 16 + 8;
                long dx = (long) blockX - centerBlockX;
                long dz = (long) blockZ - centerBlockZ;
                long distanceSquared = dx * dx + dz * dz;
                if (distanceSquared <= (long) radiusBlocks * radiusBlocks) {
                    candidates.add(new StructureCandidate(
                            chunkX, chunkZ, blockX, blockZ, distanceSquared
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
        return locate(worldSeed, centerBlockX, centerBlockZ, radiusBlocks, spec.placement());
    }
}
