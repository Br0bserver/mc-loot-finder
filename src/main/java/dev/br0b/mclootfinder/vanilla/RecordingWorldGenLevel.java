package dev.br0b.mclootfinder.vanilla;

import dev.br0b.mclootfinder.core.StructureSpec;
import net.minecraft.core.BlockPos;
import net.minecraft.core.Registry;
import net.minecraft.core.registries.Registries;
import net.minecraft.resources.ResourceKey;
import net.minecraft.server.level.ServerLevel;
import net.minecraft.util.RandomSource;
import net.minecraft.world.RandomizableContainer;
import net.minecraft.world.attribute.EnvironmentAttributeReader;
import net.minecraft.world.flag.FeatureFlags;
import net.minecraft.world.level.ChunkPos;
import net.minecraft.world.level.WorldGenLevel;
import net.minecraft.world.level.biome.BiomeManager;
import net.minecraft.world.level.block.Blocks;
import net.minecraft.world.level.block.EntityBlock;
import net.minecraft.world.level.block.entity.BlockEntity;
import net.minecraft.world.level.block.entity.BlockEntityType;
import net.minecraft.world.level.block.state.BlockState;
import net.minecraft.world.level.chunk.ChunkAccess;
import net.minecraft.world.level.chunk.PalettedContainerFactory;
import net.minecraft.world.level.chunk.ProtoChunk;
import net.minecraft.world.level.chunk.UpgradeData;
import net.minecraft.world.level.levelgen.Heightmap;
import net.minecraft.world.level.material.Fluids;

import java.lang.reflect.InvocationHandler;
import java.lang.reflect.Method;
import java.lang.reflect.Proxy;
import java.util.ArrayList;
import java.util.HashMap;
import java.util.LinkedHashMap;
import java.util.List;
import java.util.Map;
import java.util.Optional;
import java.util.function.Predicate;

/**
 * Minimal in-memory WorldGenLevel that records vanilla structure placement.
 * Unsupported world interactions fail closed instead of silently approximating.
 */
final class RecordingWorldGenLevel implements InvocationHandler {
    private final VanillaRuntime26_1_2 runtime;
    private final StructureSpec spec;
    private final long worldSeed;
    private final RandomSource levelRandom;
    private final Map<BlockPos, BlockState> states = new HashMap<>();
    private final Map<BlockPos, BlockEntity> blockEntities = new LinkedHashMap<>();
    private final Map<Long, Integer> heightCache = new HashMap<>();
    private final Map<Long, ProtoChunk> chunks = new HashMap<>();
    private final WorldGenLevel level;
    private final ServerLevel entitySuppressingLevel;
    private final BiomeManager biomeManager;
    private long subTick;

    RecordingWorldGenLevel(
            VanillaRuntime26_1_2 runtime,
            StructureSpec spec,
            long worldSeed
    ) {
        this.runtime = runtime;
        this.spec = spec;
        this.worldSeed = worldSeed;
        this.levelRandom = RandomSource.create(worldSeed);
        this.entitySuppressingLevel = EntitySuppressingServerLevel.create(worldSeed);
        this.level = (WorldGenLevel) Proxy.newProxyInstance(
                WorldGenLevel.class.getClassLoader(),
                new Class<?>[]{WorldGenLevel.class},
                this
        );
        this.biomeManager = new BiomeManager(
                (quartX, quartY, quartZ) -> runtime.noiseBiome(
                        spec, quartX, quartY, quartZ
                ),
                BiomeManager.obfuscateSeed(worldSeed)
        );
    }

    WorldGenLevel level() {
        return level;
    }

    List<RecordedContainer> containers() {
        List<RecordedContainer> result = new ArrayList<>();
        for (var entry : blockEntities.entrySet()) {
            if (!(entry.getValue() instanceof RandomizableContainer container)) {
                continue;
            }
            ResourceKey<net.minecraft.world.level.storage.loot.LootTable> table =
                    container.getLootTable();
            result.add(new RecordedContainer(
                    entry.getKey(),
                    table == null ? "" : table.identifier().toString(),
                    container.getLootTableSeed()
            ));
        }
        return List.copyOf(result);
    }

    @Override
    public Object invoke(Object proxy, Method method, Object[] nullableArgs) throws Throwable {
        Object[] args = nullableArgs == null ? new Object[0] : nullableArgs;
        String name = method.getName();

        if (method.getDeclaringClass() == Object.class) {
            return switch (name) {
                case "toString" -> "RecordingWorldGenLevel[" + spec.name() + "]";
                case "hashCode" -> System.identityHashCode(proxy);
                case "equals" -> proxy == args[0];
                default -> throw unsupported(method);
            };
        }

        switch (name) {
            case "registryAccess" -> { return runtime.registries(); }
            case "holderLookup" -> {
                @SuppressWarnings("unchecked")
                ResourceKey<? extends Registry<Object>> key =
                        (ResourceKey<? extends Registry<Object>>) args[0];
                return runtime.registries().lookupOrThrow(key);
            }
            case "getSeed" -> { return worldSeed; }
            case "getRandom" -> { return levelRandom; }
            case "getMinY" -> { return runtime.heightAccessor(spec).getMinY(); }
            case "getMaxY" -> { return runtime.heightAccessor(spec).getMaxY(); }
            case "getHeight" -> {
                if (args.length == 0) {
                    return runtime.heightAccessor(spec).getHeight();
                }
                return terrainHeight((int) args[1], (int) args[2]);
            }
            case "getHeightmapPos" -> {
                BlockPos pos = (BlockPos) args[1];
                return new BlockPos(pos.getX(), terrainHeight(pos.getX(), pos.getZ()), pos.getZ());
            }
            case "getSeaLevel" -> { return runtime.chunkGenerator(spec).getSeaLevel(); }
            case "dimensionType" -> { return runtime.dimensionType(spec); }
            case "getBiomeManager" -> { return biomeManager; }
            case "getUncachedNoiseBiome" -> {
                return runtime.noiseBiome(spec, (int) args[0], (int) args[1], (int) args[2]);
            }
            case "enabledFeatures" -> { return FeatureFlags.DEFAULT_FLAGS; }
            case "isClientSide" -> { return false; }
            case "hasChunk", "hasChunkAt", "ensureCanWrite" -> { return true; }
            case "getChunk" -> { return chunk(args); }
            case "getChunkForCollisions" -> { return level; }
            case "getBlockState" -> { return blockState((BlockPos) args[0]); }
            case "getFluidState" -> { return Fluids.EMPTY.defaultFluidState(); }
            case "getBlockEntity" -> { return blockEntity(method, args); }
            case "setBlock" -> { return setBlock((BlockPos) args[0], (BlockState) args[1]); }
            case "removeBlock", "destroyBlock" -> {
                BlockPos pos = ((BlockPos) args[0]).immutable();
                states.remove(pos);
                blockEntities.remove(pos);
                return true;
            }
            case "isStateAtPosition" -> {
                @SuppressWarnings("unchecked")
                Predicate<BlockState> predicate = (Predicate<BlockState>) args[1];
                return predicate.test(blockState((BlockPos) args[0]));
            }
            case "isFluidAtPosition" -> {
                @SuppressWarnings("unchecked")
                Predicate<net.minecraft.world.level.material.FluidState> predicate =
                        (Predicate<net.minecraft.world.level.material.FluidState>) args[1];
                return predicate.test(Fluids.EMPTY.defaultFluidState());
            }
            case "getEntities", "getEntityCollisions", "players" -> { return List.of(); }
            case "addFreshEntity", "addFreshEntityWithPassengers" -> { return true; }
            case "nextSubTickCount" -> { return subTick++; }
            case "getLevel" -> { return entitySuppressingLevel; }
            case "environmentAttributes" -> { return EnvironmentAttributeReader.EMPTY; }
            case "getServer", "getChunkSource", "getLevelData",
                    "getWorldBorder", "getLightEngine" -> {
                return null;
            }
            case "getSkyDarken" -> { return 0; }
            case "getBrightness", "getRawBrightness", "getMaxLocalRawBrightness",
                    "getEffectiveSkyBrightness" -> { return 15; }
            case "playSound", "addParticle", "levelEvent", "gameEvent",
                    "sendBlockUpdated", "updateNeighborsAt", "neighborShapeChanged",
                    "scheduleTick", "setCurrentlyGenerating", "blockUpdated" -> {
                return null;
            }
            default -> {
                if (method.isDefault()) {
                    return InvocationHandler.invokeDefault(proxy, method, args);
                }
                throw unsupported(method);
            }
        }
    }

    private Object chunk(Object[] args) {
        int chunkX;
        int chunkZ;
        if (args[0] instanceof BlockPos pos) {
            chunkX = Math.floorDiv(pos.getX(), 16);
            chunkZ = Math.floorDiv(pos.getZ(), 16);
        } else {
            chunkX = (int) args[0];
            chunkZ = (int) args[1];
        }
        long key = ChunkPos.pack(chunkX, chunkZ);
        return chunks.computeIfAbsent(key, ignored -> new ProtoChunk(
                new ChunkPos(chunkX, chunkZ),
                UpgradeData.EMPTY,
                runtime.heightAccessor(spec),
                PalettedContainerFactory.create(runtime.registries()),
                null
        ));
    }

    private Object blockEntity(Method method, Object[] args) {
        BlockEntity blockEntity = blockEntities.get((BlockPos) args[0]);
        if (method.getReturnType() == Optional.class) {
            if (blockEntity == null) {
                return Optional.empty();
            }
            if (args.length > 1 && args[1] instanceof BlockEntityType<?> type
                    && !type.isValid(blockState((BlockPos) args[0]))) {
                return Optional.empty();
            }
            return Optional.of(blockEntity);
        }
        return blockEntity;
    }

    private boolean setBlock(BlockPos mutablePos, BlockState state) {
        BlockPos pos = mutablePos.immutable();
        states.put(pos, state);
        if (state.getBlock() instanceof EntityBlock entityBlock) {
            BlockEntity current = blockEntities.get(pos);
            if (current == null || current.getType() == null
                    || !current.getBlockState().is(state.getBlock())) {
                BlockEntity created = entityBlock.newBlockEntity(pos, state);
                if (created != null) {
                    blockEntities.put(pos, created);
                }
            }
        } else {
            blockEntities.remove(pos);
        }
        return true;
    }

    private BlockState blockState(BlockPos pos) {
        BlockState placed = states.get(pos);
        if (placed != null) {
            return placed;
        }
        // Structure placement tests surrounding solidity much more often than
        // it needs a real column. Exact heightmap requests are handled above;
        // an untouched position otherwise uses the dimension's solid substrate.
        return switch (spec.dimensionId()) {
            case "minecraft:the_nether" -> Blocks.NETHERRACK.defaultBlockState();
            case "minecraft:the_end" -> Blocks.END_STONE.defaultBlockState();
            default -> Blocks.STONE.defaultBlockState();
        };
    }

    private int terrainHeight(int blockX, int blockZ) {
        long key = ChunkPos.pack(blockX, blockZ);
        return heightCache.computeIfAbsent(
                key,
                ignored -> runtime.motionBlockingHeight(spec, blockX, blockZ)
        );
    }

    private static UnsupportedOperationException unsupported(Method method) {
        return new UnsupportedOperationException(
                "Unsupported structure world interaction: " + method.toGenericString()
        );
    }

    record RecordedContainer(BlockPos pos, String lootTable, long lootSeed) {
    }
}
