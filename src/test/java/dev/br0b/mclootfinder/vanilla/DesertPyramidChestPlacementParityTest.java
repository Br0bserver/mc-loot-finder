package dev.br0b.mclootfinder.vanilla;

import dev.br0b.mclootfinder.core.StructureSpec;
import dev.br0b.mclootfinder.core.Versions;
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
import net.minecraft.world.level.chunk.ChunkAccess;
import net.minecraft.world.level.levelgen.Heightmap;
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
import static org.junit.jupiter.api.Assertions.assertNotNull;
import static org.junit.jupiter.api.Assertions.assertTrue;
import static org.mockito.ArgumentMatchers.any;
import static org.mockito.ArgumentMatchers.anyInt;
import static org.mockito.Mockito.mock;
import static org.mockito.Mockito.when;

class DesertPyramidChestPlacementParityTest {
    @Test
    void proceduralScannerMatchesVanillaPlacementAcrossOrientations() {
        long worldSeed = 0L;
        StructureSpec spec = Versions.V26_1_2.desertPyramid();

        try (VanillaRuntime26_1_2 runtime = VanillaRuntime26_1_2.load(worldSeed)) {
            runtime.verifyStructureProfile(spec);
            List<ChunkPos> starts = List.of(
                    new ChunkPos(0, -188),
                    new ChunkPos(77, -213),
                    new ChunkPos(81, -254)
            );
            for (ChunkPos startChunk : starts) {
                var start = runtime.generateSelectedStructure(spec, startChunk);
                assertTrue(start.isValid(), "desert pyramid test vector drifted: " + startChunk);
                List<ChestPrediction> predictions = StructureChestScanner.scan(
                        worldSeed, spec, start, runtime
                );
                assertEquals(4, predictions.size());
                assertPlacement(runtime, worldSeed, spec, startChunk, predictions);
            }
        }
    }

    private static void assertPlacement(
            VanillaRuntime26_1_2 runtime,
            long worldSeed,
            StructureSpec spec,
            ChunkPos startChunk,
            List<ChestPrediction> predictions
    ) {
        CapturedWorld captured = new CapturedWorld(runtime, worldSeed, spec);

        // The structure start chunk initializes ScatteredFeaturePiece.HPos.
        var start = runtime.generateSelectedStructure(spec, startChunk);
        placeChunk(runtime, worldSeed, spec, start, startChunk, captured);

        Map<Long, List<ChestPrediction>> byChunk = predictions.stream().collect(
                java.util.stream.Collectors.groupingBy(
                        chest -> ChunkPos.pack(
                                Math.floorDiv(chest.x(), 16), Math.floorDiv(chest.z(), 16)
                        ),
                        LinkedHashMap::new,
                        java.util.stream.Collectors.toList()
                )
        );
        for (long packedChunk : byChunk.keySet()) {
            ChunkPos chunk = ChunkPos.unpack(packedChunk);
            if (!chunk.equals(startChunk)) {
                placeChunk(runtime, worldSeed, spec, start, chunk, captured);
            }
        }

        Map<BlockPos, ActualChest> actual = captured.chests();
        assertEquals(4, actual.size());
        assertEquals(4, captured.randomizableContainerCount());
        for (ChestPrediction expected : predictions) {
            ActualChest chest = actual.get(new BlockPos(expected.x(), expected.y(), expected.z()));
            assertNotNull(chest, "missing placed chest at " + expected);
            assertEquals(expected.lootTable(), chest.lootTable(), "at " + chest.pos());
            assertEquals(expected.lootTableSeed(), chest.lootSeed(), "at " + chest.pos());
        }
    }

    private static void placeChunk(
            VanillaRuntime26_1_2 runtime,
            long worldSeed,
            StructureSpec spec,
            net.minecraft.world.level.levelgen.structure.StructureStart start,
            ChunkPos chunk,
            CapturedWorld captured
    ) {
        WorldgenRandom random = new WorldgenRandom(new XoroshiroRandomSource(0L));
        long decorationSeed = random.setDecorationSeed(
                worldSeed, chunk.getMinBlockX(), chunk.getMinBlockZ()
        );
        random.setFeatureSeed(decorationSeed, spec.indexWithinStep(), spec.decorationStep());
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
    }

    private record ActualChest(BlockPos pos, String lootTable, long lootSeed) {
    }

    private static final class CapturedWorld {
        private final WorldGenLevel level = mock(WorldGenLevel.class);
        private final Map<BlockPos, BlockState> states = new HashMap<>();
        private final Map<BlockPos, BlockEntity> blockEntities = new HashMap<>();

        private CapturedWorld(
                VanillaRuntime26_1_2 runtime,
                long seed,
                StructureSpec spec
        ) {
            ServerLevel serverLevel = mock(ServerLevel.class);
            ChunkAccess chunk = mock(ChunkAccess.class);
            when(level.getLevel()).thenReturn(serverLevel);
            when(level.registryAccess()).thenReturn(runtime.registries());
            when(level.holderLookup(Registries.BLOCK))
                    .thenReturn(runtime.registries().lookupOrThrow(Registries.BLOCK));
            when(level.getSeed()).thenReturn(seed);
            when(level.getMinY()).thenReturn(runtime.heightAccessor(spec).getMinY());
            when(level.getMaxY()).thenReturn(runtime.heightAccessor(spec).getMaxY());
            when(level.getHeight()).thenReturn(runtime.heightAccessor(spec).getHeight());
            when(level.getRandom()).thenReturn(RandomSource.create(seed));
            when(level.getChunk(any(BlockPos.class))).thenReturn(chunk);
            when(level.getHeightmapPos(any(Heightmap.Types.class), any(BlockPos.class)))
                    .thenAnswer(invocation -> {
                        BlockPos pos = invocation.getArgument(1);
                        return new BlockPos(
                                pos.getX(),
                                runtime.motionBlockingHeight(spec, pos.getX(), pos.getZ()),
                                pos.getZ()
                        );
                    });
            when(level.getFluidState(any(BlockPos.class)))
                    .thenReturn(Fluids.EMPTY.defaultFluidState());
            when(level.getBlockState(any(BlockPos.class))).thenAnswer(invocation -> {
                BlockPos pos = invocation.getArgument(0);
                return states.getOrDefault(pos, Blocks.SANDSTONE.defaultBlockState());
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
                ResourceKey<net.minecraft.world.level.storage.loot.LootTable> table =
                        container.getLootTable();
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
