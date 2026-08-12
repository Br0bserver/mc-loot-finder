package dev.br0b.mclootfinder.vanilla;

import dev.br0b.mclootfinder.core.StructureSpec;
import dev.br0b.mclootfinder.core.random.DecorationRandom;
import net.minecraft.core.BlockPos;
import net.minecraft.core.Direction;
import net.minecraft.world.level.ChunkPos;
import net.minecraft.world.level.levelgen.structure.BoundingBox;
import net.minecraft.world.level.levelgen.structure.StructurePiece;
import net.minecraft.world.level.levelgen.structure.StructureStart;
import net.minecraft.world.level.levelgen.structure.structures.DesertPyramidPiece;

import java.util.ArrayList;
import java.util.HashMap;
import java.util.List;
import java.util.Map;

/** Reproduces DesertPyramidPiece's four procedural chest placements. */
public final class DesertPyramidChestScanner {
    private static final String LOOT_TABLE = "minecraft:chests/desert_pyramid";
    private static final List<Direction> CHEST_ORDER = List.of(
            Direction.NORTH, Direction.EAST, Direction.SOUTH, Direction.WEST
    );

    private DesertPyramidChestScanner() {
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
        if (start.getPieces().size() != 1
                || !(start.getPieces().getFirst() instanceof DesertPyramidPiece piece)) {
            throw new IllegalStateException("Unexpected desert pyramid piece layout");
        }

        BoundingBox box = piece.getBoundingBox();
        int lowestGround = lowestGround(runtime, spec, box);
        DecorationRandom startChunkRandom = featureRandom(
                worldSeed, spec, start.getChunkPos().x(), start.getChunkPos().z()
        );
        int chestY = lowestGround - startChunkRandom.nextInt(3) - 11;

        List<BlockPos> positions = new ArrayList<>(4);
        for (Direction direction : CHEST_ORDER) {
            int localX = 10 + direction.getStepX() * 2;
            int localZ = 10 + direction.getStepZ() * 2;
            positions.add(new BlockPos(
                    worldX(piece, localX, localZ),
                    chestY,
                    worldZ(piece, localX, localZ)
            ));
        }

        Map<Long, Integer> nextOrdinalByChunk = new HashMap<>();
        List<ChestPrediction> predictions = new ArrayList<>(4);
        for (BlockPos position : positions) {
            int chunkX = Math.floorDiv(position.getX(), 16);
            int chunkZ = Math.floorDiv(position.getZ(), 16);
            long chunkKey = ChunkPos.pack(chunkX, chunkZ);
            int ordinal = nextOrdinalByChunk.getOrDefault(chunkKey, 0);
            nextOrdinalByChunk.put(chunkKey, ordinal + 1);

            long lootSeed = StructureChestScanner.containerLootSeed(
                    worldSeed, spec, chunkX, chunkZ, ordinal
            );
            predictions.add(new ChestPrediction(
                    start.getChunkPos().x(),
                    start.getChunkPos().z(),
                    position.getX(),
                    position.getY(),
                    position.getZ(),
                    LOOT_TABLE,
                    ordinal,
                    lootSeed
            ));
        }
        return List.copyOf(predictions);
    }

    private static int lowestGround(
            VanillaRuntime26_1_2 runtime,
            StructureSpec spec,
            BoundingBox box
    ) {
        int result = Integer.MAX_VALUE;
        for (int z = box.minZ(); z <= box.maxZ(); z++) {
            for (int x = box.minX(); x <= box.maxX(); x++) {
                result = Math.min(result, runtime.motionBlockingHeight(spec, x, z));
            }
        }
        return result;
    }

    private static DecorationRandom featureRandom(
            long worldSeed,
            StructureSpec spec,
            int chunkX,
            int chunkZ
    ) {
        DecorationRandom random = new DecorationRandom();
        long decorationSeed = random.setDecorationSeed(worldSeed, chunkX * 16, chunkZ * 16);
        random.setFeatureSeed(
                decorationSeed, spec.indexWithinStep(), spec.decorationStep()
        );
        return random;
    }

    private static int worldX(StructurePiece piece, int localX, int localZ) {
        BoundingBox box = piece.getBoundingBox();
        return switch (piece.getOrientation()) {
            case NORTH, SOUTH -> box.minX() + localX;
            case WEST -> box.maxX() - localZ;
            case EAST -> box.minX() + localZ;
            default -> throw new IllegalStateException(
                    "Unexpected desert pyramid orientation: " + piece.getOrientation()
            );
        };
    }

    private static int worldZ(StructurePiece piece, int localX, int localZ) {
        BoundingBox box = piece.getBoundingBox();
        return switch (piece.getOrientation()) {
            case NORTH -> box.maxZ() - localZ;
            case SOUTH -> box.minZ() + localZ;
            case WEST, EAST -> box.minZ() + localX;
            default -> throw new IllegalStateException(
                    "Unexpected desert pyramid orientation: " + piece.getOrientation()
            );
        };
    }
}
