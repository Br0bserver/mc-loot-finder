package dev.br0b.mclootfinder.vanilla;

import dev.br0b.mclootfinder.core.StructureSpec;
import net.minecraft.world.level.ChunkPos;
import net.minecraft.world.level.StructureManager;
import net.minecraft.world.level.levelgen.WorldOptions;
import net.minecraft.world.level.levelgen.WorldgenRandom;
import net.minecraft.world.level.levelgen.XoroshiroRandomSource;
import net.minecraft.world.level.levelgen.structure.BoundingBox;
import net.minecraft.world.level.levelgen.structure.StructureStart;

import java.util.ArrayList;
import java.util.Comparator;
import java.util.List;

/**
 * Generic correctness backend: executes vanilla StructureStart placement in a
 * recording world and reads the container block entities it creates.
 */
public final class VanillaPlacementChestScanner {
    private VanillaPlacementChestScanner() {
    }

    public static List<ChestPrediction> scan(
            long worldSeed,
            StructureSpec spec,
            StructureStart start,
            VanillaRuntime26_1_2 runtime
    ) {
        if (!start.isValid()) {
            return List.of();
        }

        ChunkPos startChunk = start.getChunkPos();
        VanillaRuntime26_1_2.DecorationCoordinates decoration =
                runtime.structureDecorationCoordinates(runtime.structureId(start));
        List<ChunkPos> chunks = start.getBoundingBox().intersectingChunks()
                .sorted(Comparator
                        .comparingInt((ChunkPos chunk) -> chunk.equals(startChunk) ? 0 : 1)
                        .thenComparingInt(ChunkPos::x)
                        .thenComparingInt(ChunkPos::z))
                .toList();
        List<ChestPrediction> result = new ArrayList<>();
        for (ChunkPos chunk : chunks) {
            RecordingWorldGenLevel recording = new RecordingWorldGenLevel(
                    runtime, spec, worldSeed
            );
            StructureManager structureManager = new StructureManager(
                    recording.level(),
                    new WorldOptions(worldSeed, true, false),
                    null
            );
            WorldgenRandom random = new WorldgenRandom(new XoroshiroRandomSource(0L));
            long decorationSeed = random.setDecorationSeed(
                    worldSeed, chunk.getMinBlockX(), chunk.getMinBlockZ()
            );
            random.setFeatureSeed(
                    decorationSeed, decoration.indexWithinStep(), decoration.step()
            );
            BoundingBox bounds = new BoundingBox(
                    chunk.getMinBlockX(), runtime.heightAccessor(spec).getMinY(), chunk.getMinBlockZ(),
                    chunk.getMaxBlockX(), runtime.heightAccessor(spec).getMaxY(), chunk.getMaxBlockZ()
            );
            try {
                start.placeInChunk(
                        recording.level(),
                        structureManager,
                        runtime.chunkGenerator(spec),
                        random,
                        bounds,
                        chunk
                );
            } catch (RuntimeException exception) {
                throw new IllegalStateException(
                        "Vanilla placement recording failed for " + spec.name()
                                + " start=" + startChunk + " decoration=" + chunk,
                        exception
                );
            }

            int ordinal = 0;
            for (RecordingWorldGenLevel.RecordedContainer container : recording.containers()) {
                if (Math.floorDiv(container.pos().getX(), 16) != chunk.x()
                        || Math.floorDiv(container.pos().getZ(), 16) != chunk.z()) {
                    continue;
                }
                int containerOrdinal = ordinal++;
                if (container.lootTable().isEmpty()) {
                    continue;
                }
                result.add(new ChestPrediction(
                        startChunk.x(),
                        startChunk.z(),
                        container.pos().getX(),
                        container.pos().getY(),
                        container.pos().getZ(),
                        container.lootTable(),
                        containerOrdinal,
                        container.lootSeed()
                ));
            }
        }
        return List.copyOf(result);
    }
}
