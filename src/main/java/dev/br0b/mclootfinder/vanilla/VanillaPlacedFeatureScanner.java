package dev.br0b.mclootfinder.vanilla;

import dev.br0b.mclootfinder.core.StructureSpec;
import dev.br0b.mclootfinder.core.VersionProfile;
import dev.br0b.mclootfinder.core.structure.StructureCandidate;
import dev.br0b.mclootfinder.engine.StructureScan;
import net.minecraft.core.BlockPos;
import net.minecraft.world.level.ChunkPos;
import net.minecraft.world.level.levelgen.WorldgenRandom;
import net.minecraft.world.level.levelgen.XoroshiroRandomSource;

import java.util.ArrayList;
import java.util.Comparator;
import java.util.List;

/** Executes version-pinned per-chunk placed features such as desert wells. */
final class VanillaPlacedFeatureScanner {
    private VanillaPlacedFeatureScanner() {
    }

    static List<StructureCandidate> locateCandidates(
            long worldSeed,
            StructureSpec spec,
            VanillaRuntime26_1_2 runtime,
            int centerBlockX,
            int centerBlockZ,
            int radiusBlocks
    ) {
        if (radiusBlocks < 0) {
            throw new IllegalArgumentException("radius must be non-negative");
        }
        if (!(spec.placement() instanceof VersionProfile.FeatureProfile profile)) {
            throw new IllegalArgumentException(spec.name() + " is not a placed feature");
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
        long radiusSquared = (long) radiusBlocks * radiusBlocks;
        var decoration = runtime.placedFeatureDecorationCoordinates(spec);
        List<StructureCandidate> candidates = new ArrayList<>();
        for (int chunkX = minChunkX; chunkX <= maxChunkX; chunkX++) {
            for (int chunkZ = minChunkZ; chunkZ <= maxChunkZ; chunkZ++) {
                WorldgenRandom random = featureRandom(
                        worldSeed, chunkX, chunkZ, decoration
                );
                if (random.nextFloat() >= 1.0F / profile.rarityChance()) {
                    continue;
                }
                int blockX = chunkX * 16 + random.nextInt(16);
                int blockZ = chunkZ * 16 + random.nextInt(16);
                long dx = (long) blockX - centerBlockX;
                long dz = (long) blockZ - centerBlockZ;
                if (Math.abs(dx) > radiusBlocks || Math.abs(dz) > radiusBlocks
                        || dx * dx > radiusSquared - dz * dz) {
                    continue;
                }
                candidates.add(new StructureCandidate(
                        chunkX, chunkZ, blockX, blockZ, dx * dx + dz * dz
                ));
            }
        }
        candidates.sort(Comparator.comparingLong(StructureCandidate::squaredDistanceFromCenter));
        return List.copyOf(candidates);
    }

    static StructureScan scan(
            long worldSeed,
            StructureSpec spec,
            VanillaRuntime26_1_2 runtime,
            int chunkX,
            int chunkZ
    ) {
        var decoration = runtime.placedFeatureDecorationCoordinates(spec);
        WorldgenRandom random = featureRandom(worldSeed, chunkX, chunkZ, decoration);
        RecordingWorldGenLevel recording = RecordingWorldGenLevel.forDesertSurface(
                runtime, spec, worldSeed
        );
        boolean placed;
        try {
            placed = runtime.placedFeature(spec).placeWithBiomeCheck(
                    recording.level(),
                    runtime.chunkGenerator(spec),
                    random,
                    new BlockPos(
                            chunkX * 16,
                            runtime.heightAccessor(spec).getMinY(),
                            chunkZ * 16
                    )
            );
        } catch (RuntimeException exception) {
            throw new IllegalStateException(
                    "Vanilla feature recording failed for " + spec.name()
                            + " chunk=(" + chunkX + "," + chunkZ + ")",
                    exception
            );
        }
        if (!placed) {
            return StructureScan.absent();
        }
        List<ChestPrediction> sources = recording.lootSources().stream()
                .map(source -> new ChestPrediction(
                        chunkX,
                        chunkZ,
                        source.pos().getX(),
                        source.pos().getY(),
                        source.pos().getZ(),
                        source.lootTable(),
                        -1,
                        source.lootSeed(),
                        source.kind(),
                        source.blockId()
                ))
                .toList();
        return new StructureScan(true, sources);
    }

    private static WorldgenRandom featureRandom(
            long worldSeed,
            int chunkX,
            int chunkZ,
            VanillaRuntime26_1_2.DecorationCoordinates decoration
    ) {
        WorldgenRandom random = new WorldgenRandom(new XoroshiroRandomSource(0L));
        long decorationSeed = random.setDecorationSeed(
                worldSeed, chunkX * 16, chunkZ * 16
        );
        random.setFeatureSeed(
                decorationSeed, decoration.indexWithinStep(), decoration.step()
        );
        return random;
    }
}
