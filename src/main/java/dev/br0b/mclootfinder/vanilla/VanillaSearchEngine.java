package dev.br0b.mclootfinder.vanilla;

import dev.br0b.mclootfinder.core.StructureSpec;
import dev.br0b.mclootfinder.core.VersionProfile;
import dev.br0b.mclootfinder.core.structure.RandomSpreadLocator;
import dev.br0b.mclootfinder.core.structure.StructureCandidate;
import dev.br0b.mclootfinder.engine.SearchEngine;
import dev.br0b.mclootfinder.engine.StructureScan;
import net.minecraft.world.level.ChunkPos;

import java.util.List;

public final class VanillaSearchEngine implements SearchEngine {
    private final long worldSeed;
    private final VanillaRuntime26_1_2 runtime;
    private final LootOracle lootOracle;

    private VanillaSearchEngine(long worldSeed, VanillaRuntime26_1_2 runtime) {
        this.worldSeed = worldSeed;
        this.runtime = runtime;
        this.lootOracle = new JsonLootTableOracle26_1_2(runtime.registries());
    }

    public static VanillaSearchEngine load(long worldSeed) {
        return new VanillaSearchEngine(worldSeed, VanillaRuntime26_1_2.load(worldSeed));
    }

    @Override
    public void verifyProfile(StructureSpec spec) {
        runtime.verifyStructureProfile(spec);
    }

    @Override
    public List<StructureCandidate> locateCandidates(
            StructureSpec spec,
            int centerBlockX,
            int centerBlockZ,
            int radiusBlocks
    ) {
        if (spec.placement() instanceof VersionProfile.FeatureProfile) {
            return VanillaPlacedFeatureScanner.locateCandidates(
                    worldSeed, spec, runtime, centerBlockX, centerBlockZ, radiusBlocks
            );
        }
        if (spec.placement() instanceof VersionProfile.StructureProfile) {
            return RandomSpreadLocator.locate(
                    worldSeed, centerBlockX, centerBlockZ, radiusBlocks, spec
            );
        }
        return runtime.locateConcentricRingCandidates(
                spec, centerBlockX, centerBlockZ, radiusBlocks
        );
    }

    @Override
    public StructureScan scan(StructureSpec spec, int chunkX, int chunkZ) {
        if (spec.scannerKind() == StructureSpec.ScannerKind.PLACED_FEATURE) {
            return VanillaPlacedFeatureScanner.scan(
                    worldSeed, spec, runtime, chunkX, chunkZ
            );
        }
        ChunkPos chunk = new ChunkPos(chunkX, chunkZ);
        if (!runtime.isStructurePlacementChunk(spec, chunk)) {
            return StructureScan.absent();
        }
        var start = runtime.generateSelectedStructure(spec, chunk);
        if (!start.isValid()) {
            return StructureScan.absent();
        }
        return new StructureScan(
                true,
                StructureChestScanner.scanAll(worldSeed, spec, start, runtime)
        );
    }

    @Override
    public LootOracle lootOracle() {
        return lootOracle;
    }

    @Override
    public void close() {
        runtime.close();
    }
}
