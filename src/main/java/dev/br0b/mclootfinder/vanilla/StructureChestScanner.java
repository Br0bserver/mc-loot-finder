package dev.br0b.mclootfinder.vanilla;

import dev.br0b.mclootfinder.core.StructureSpec;
import dev.br0b.mclootfinder.core.random.DecorationRandom;
import net.minecraft.world.level.levelgen.structure.StructureStart;

import java.util.List;

/** Dispatches to the version-pinned container layout implementation. */
public final class StructureChestScanner {
    private StructureChestScanner() {
    }

    public static List<ChestPrediction> scan(
            long worldSeed,
            StructureSpec spec,
            StructureStart start,
            VanillaRuntime26_1_2 runtime
    ) {
        List<ChestPrediction> placed = switch (spec.scannerKind()) {
            case JIGSAW_FAST -> JigsawChestScanner.scan(
                    worldSeed, spec, start, runtime.templateManager()
            );
            case VANILLA_PLACEMENT -> VanillaPlacementChestScanner.scan(
                    worldSeed, spec, start, runtime
            );
        };
        return placed.stream()
                .filter(chest -> !chest.lootTable().isEmpty())
                .toList();
    }

    public static long containerLootSeed(
            long worldSeed,
            StructureSpec spec,
            int chunkX,
            int chunkZ,
            int ordinal
    ) {
        return containerLootSeed(
                worldSeed,
                spec,
                chunkX,
                chunkZ,
                spec.indexWithinStep(),
                spec.decorationStep(),
                ordinal
        );
    }

    public static long containerLootSeed(
            long worldSeed,
            StructureSpec spec,
            int chunkX,
            int chunkZ,
            int structureIndex,
            int decorationStep,
            int ordinal
    ) {
        if (ordinal < 0) {
            throw new IllegalArgumentException("container ordinal must be non-negative");
        }
        if (spec.containerSeedShortcut() == StructureSpec.ContainerSeedShortcut.NONE) {
            throw new IllegalArgumentException(
                    "container-seed is not available for " + spec.name()
                            + "; use 'chests' to execute vanilla placement"
            );
        }

        DecorationRandom random = new DecorationRandom();
        long decorationSeed = random.setDecorationSeed(worldSeed, chunkX * 16, chunkZ * 16);
        random.setFeatureSeed(decorationSeed, structureIndex, decorationStep);
        if (spec.containerSeedShortcut()
                == StructureSpec.ContainerSeedShortcut.DESERT_PYRAMID) {
            random.nextInt(3);
        }
        long result = 0L;
        for (int index = 0; index <= ordinal; index++) {
            result = random.nextLong();
        }
        return result;
    }
}
