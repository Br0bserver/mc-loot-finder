package dev.br0b.mclootfinder.vanilla;

import dev.br0b.mclootfinder.core.Versions;
import dev.br0b.mclootfinder.core.structure.RandomSpreadLocator;
import net.minecraft.core.BlockPos;
import net.minecraft.core.registries.Registries;
import net.minecraft.resources.ResourceKey;
import net.minecraft.server.level.ServerLevel;
import net.minecraft.util.RandomSource;
import net.minecraft.world.RandomizableContainer;
import net.minecraft.world.level.ChunkPos;
import net.minecraft.world.level.StructureManager;
import net.minecraft.world.level.WorldGenLevel;
import net.minecraft.world.level.block.Blocks;
import net.minecraft.world.level.block.EntityBlock;
import net.minecraft.world.level.block.entity.BlockEntity;
import net.minecraft.world.level.block.state.BlockState;
import net.minecraft.world.level.levelgen.WorldgenRandom;
import net.minecraft.world.level.levelgen.XoroshiroRandomSource;
import net.minecraft.world.level.levelgen.structure.BoundingBox;
import net.minecraft.world.level.material.Fluids;
import org.junit.jupiter.api.Test;

import java.util.HashMap;
import java.util.LinkedHashMap;
import java.util.List;
import java.util.Map;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertTrue;
import static org.mockito.ArgumentMatchers.any;
import static org.mockito.ArgumentMatchers.anyInt;
import static org.mockito.Mockito.mock;
import static org.mockito.Mockito.when;

/**
 * Executes vanilla StructureStart.placeInChunk and compares its loaded chest
 * block entities with the fast template scanner. This closes the gap between
 * inferred palette order and actual vanilla placement order.
 */
class AncientCityChestPlacementParityTest {
    @Test
    void scannerMatchesVanillaPlacementIncludingLootSeeds() {
        long worldSeed = 0L;
        var version = Versions.V26_1_2;
        try (VanillaRuntime26_1_2 runtime = VanillaRuntime26_1_2.load(worldSeed)) {
            int validCities = 0;
            int chestCount = 0;
            Map<Long, Long> ownerByDecorationChunk = new HashMap<>();
            for (var candidate : RandomSpreadLocator.locate(
                    worldSeed, 0, 0, 5_000, version.ancientCity())) {
                var start = runtime.generateAncientCity(new ChunkPos(candidate.chunkX(), candidate.chunkZ()));
                if (!start.isValid()) {
                    continue;
                }
                validCities++;
                List<ChestPrediction> predictions = AncientCityChestScanner.scan(
                        worldSeed, version, start, runtime.templateManager()
                );
                chestCount += predictions.size();
                for (ChestPrediction prediction : predictions) {
                    long decorationChunk = ChunkPos.pack(
                            Math.floorDiv(prediction.x(), 16), Math.floorDiv(prediction.z(), 16)
                    );
                    long startChunk = ChunkPos.pack(
                            prediction.structureChunkX(), prediction.structureChunkZ()
                    );
                    Long previous = ownerByDecorationChunk.putIfAbsent(decorationChunk, startChunk);
                    assertTrue(previous == null || previous == startChunk,
                            "test vector has a cross-start container stream overlap");
                }
                assertScannerMatchesPlacement(
                        runtime, worldSeed, version.ancientCity(), start, predictions
                );
            }
            assertEquals(9, validCities);
            assertEquals(204, chestCount);
        }
    }

    static void assertScannerMatchesPlacement(
            VanillaRuntime26_1_2 runtime,
            long worldSeed,
            dev.br0b.mclootfinder.core.StructureSpec spec,
            net.minecraft.world.level.levelgen.structure.StructureStart start,
            List<ChestPrediction> predictions
    ) {
        Map<Long, List<ChestPrediction>> byChunk = predictions.stream().collect(
                java.util.stream.Collectors.groupingBy(
                        chest -> ChunkPos.pack(
                                Math.floorDiv(chest.x(), 16),
                                Math.floorDiv(chest.z(), 16)
                        ),
                        LinkedHashMap::new,
                        java.util.stream.Collectors.toList()
                )
        );

        for (var chunkEntry : byChunk.entrySet()) {
            ChunkPos chunk = ChunkPos.unpack(chunkEntry.getKey());
            CapturedWorld captured = new CapturedWorld(runtime, worldSeed);
            WorldgenRandom random = new WorldgenRandom(new XoroshiroRandomSource(0L));
            long decorationSeed = random.setDecorationSeed(
                    worldSeed, chunk.getMinBlockX(), chunk.getMinBlockZ()
            );
            random.setFeatureSeed(
                    decorationSeed,
                    spec.indexWithinStep(),
                    spec.decorationStep()
            );
            BoundingBox bounds = new BoundingBox(
                    chunk.getMinBlockX(), runtime.heightAccessor(spec).getMinY(), chunk.getMinBlockZ(),
                    chunk.getMaxBlockX(), runtime.heightAccessor(spec).getMaxY(), chunk.getMaxBlockZ()
            );
            start.placeInChunk(
                    captured.level,
                    mock(StructureManager.class),
                    runtime.chunkGenerator(spec),
                    random,
                    bounds,
                    chunk
            );

            Map<BlockPos, ActualChest> actual = captured.chests();
            assertEquals(actual.size(), captured.randomizableContainerCount(),
                    "non-chest randomizable container in chunk=" + chunk);
            assertEquals(chunkEntry.getValue().size(), actual.size(), "chunk=" + chunk);
            for (ChestPrediction expected : chunkEntry.getValue()) {
                ActualChest placed = actual.get(new BlockPos(expected.x(), expected.y(), expected.z()));
                assertTrue(placed != null, "missing placed chest at " + expected);
                assertEquals(expected.lootTable(), placed.lootTable(), "at " + placed.pos());
                assertEquals(expected.lootTableSeed(), placed.lootSeed(), "at " + placed.pos());
            }
        }
    }

    private record ActualChest(BlockPos pos, String lootTable, long lootSeed) {
    }

    private static final class CapturedWorld {
        private final WorldGenLevel level = mock(WorldGenLevel.class);
        private final Map<BlockPos, BlockState> states = new HashMap<>();
        private final Map<BlockPos, BlockEntity> blockEntities = new HashMap<>();

        private CapturedWorld(VanillaRuntime26_1_2 runtime, long seed) {
            ServerLevel serverLevel = mock(ServerLevel.class);
            when(level.getLevel()).thenReturn(serverLevel);
            when(level.registryAccess()).thenReturn(runtime.registries());
            when(level.holderLookup(Registries.BLOCK))
                    .thenReturn(runtime.registries().lookupOrThrow(Registries.BLOCK));
            when(level.getSeed()).thenReturn(seed);
            when(level.getMinY()).thenReturn(runtime.heightAccessor().getMinY());
            when(level.getHeight()).thenReturn(runtime.heightAccessor().getHeight());
            when(level.getRandom()).thenReturn(RandomSource.create(seed));
            when(level.getFluidState(any(BlockPos.class)))
                    .thenReturn(Fluids.EMPTY.defaultFluidState());
            when(level.getBlockState(any(BlockPos.class))).thenAnswer(invocation -> {
                BlockPos pos = invocation.getArgument(0);
                return states.getOrDefault(pos, Blocks.DEEPSLATE.defaultBlockState());
            });
            when(level.getBlockEntity(any(BlockPos.class))).thenAnswer(invocation ->
                    blockEntities.get(invocation.<BlockPos>getArgument(0))
            );
            when(level.setBlock(any(BlockPos.class), any(BlockState.class), anyInt()))
                    .thenAnswer(invocation -> {
                        BlockPos pos = invocation.<BlockPos>getArgument(0).immutable();
                        BlockState state = invocation.getArgument(1);
                        states.put(pos, state);
                        if (state.getBlock() instanceof EntityBlock entityBlock) {
                            BlockEntity blockEntity = entityBlock.newBlockEntity(pos, state);
                            if (blockEntity != null) {
                                blockEntities.put(pos, blockEntity);
                            }
                        } else {
                            blockEntities.remove(pos);
                        }
                        return true;
                    });
        }

        private Map<BlockPos, ActualChest> chests() {
            Map<BlockPos, ActualChest> result = new HashMap<>();
            for (var entry : blockEntities.entrySet()) {
                if (!(entry.getValue() instanceof RandomizableContainer container)
                        || !states.get(entry.getKey()).is(Blocks.CHEST)) {
                    continue;
                }
                ResourceKey<net.minecraft.world.level.storage.loot.LootTable> table = container.getLootTable();
                result.put(entry.getKey(), new ActualChest(
                        entry.getKey(),
                        table == null ? "" : table.identifier().toString(),
                        container.getLootTableSeed()
                ));
            }
            return result;
        }

        private long randomizableContainerCount() {
            return blockEntities.values().stream()
                    .filter(RandomizableContainer.class::isInstance)
                    .count();
        }
    }
}
