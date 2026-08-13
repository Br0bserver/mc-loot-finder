package dev.br0b.mclootfinder.vanilla;

import dev.br0b.mclootfinder.core.StructureSpec;
import dev.br0b.mclootfinder.core.random.DecorationRandom;
import net.minecraft.core.BlockPos;
import net.minecraft.nbt.CompoundTag;
import net.minecraft.world.level.ChunkPos;
import net.minecraft.world.level.block.Blocks;
import net.minecraft.world.level.levelgen.structure.PoolElementStructurePiece;
import net.minecraft.world.level.levelgen.structure.StructurePiece;
import net.minecraft.world.level.levelgen.structure.StructureStart;
import net.minecraft.world.level.levelgen.structure.pools.ListPoolElement;
import net.minecraft.world.level.levelgen.structure.pools.SinglePoolElement;
import net.minecraft.world.level.levelgen.structure.pools.StructurePoolElement;
import net.minecraft.world.level.levelgen.structure.templatesystem.StructurePlaceSettings;
import net.minecraft.world.level.levelgen.structure.templatesystem.StructureTemplate;
import net.minecraft.world.level.levelgen.structure.templatesystem.StructureTemplateManager;

import java.util.ArrayList;
import java.util.HashMap;
import java.util.List;
import java.util.Map;

/** Extracts Jigsaw chest positions in vanilla piece/container placement order. */
public final class JigsawChestScanner {
    private JigsawChestScanner() {
    }

    public static List<ChestPrediction> scan(
            long worldSeed,
            StructureSpec spec,
            StructureStart start,
            StructureTemplateManager templates
    ) {
        return scan(
                worldSeed, spec, start, templates,
                spec.indexWithinStep(), spec.decorationStep()
        );
    }

    public static List<ChestPrediction> scan(
            long worldSeed,
            StructureSpec spec,
            StructureStart start,
            StructureTemplateManager templates,
            int structureIndex,
            int decorationStep
    ) {
        if (!start.isValid()) {
            return List.of();
        }
        List<RawChest> raw = new ArrayList<>();
        for (StructurePiece piece : start.getPieces()) {
            if (piece instanceof PoolElementStructurePiece poolPiece) {
                collectFromElement(
                        poolPiece.getElement(),
                        poolPiece.getPosition(),
                        poolPiece.getRotation(),
                        templates,
                        raw
                );
            }
        }

        Map<Long, Integer> nextOrdinalByChunk = new HashMap<>();
        List<ChestPrediction> predictions = new ArrayList<>(raw.size());
        for (RawChest chest : raw) {
            int chunkX = Math.floorDiv(chest.pos().getX(), 16);
            int chunkZ = Math.floorDiv(chest.pos().getZ(), 16);
            long chunkKey = ChunkPos.pack(chunkX, chunkZ);
            int ordinal = nextOrdinalByChunk.getOrDefault(chunkKey, 0);
            nextOrdinalByChunk.put(chunkKey, ordinal + 1);
            long lootSeed = DecorationRandom.containerLootSeed(
                    worldSeed,
                    chunkX,
                    chunkZ,
                    structureIndex,
                    decorationStep,
                    ordinal
            );
            predictions.add(new ChestPrediction(
                    start.getChunkPos().x(),
                    start.getChunkPos().z(),
                    chest.pos().getX(),
                    chest.pos().getY(),
                    chest.pos().getZ(),
                    chest.lootTable(),
                    ordinal,
                    lootSeed
            ));
        }
        return List.copyOf(predictions);
    }

    private static void collectFromElement(
            StructurePoolElement element,
            BlockPos position,
            net.minecraft.world.level.block.Rotation rotation,
            StructureTemplateManager templates,
            List<RawChest> output
    ) {
        if (element instanceof SinglePoolElement single) {
            StructureTemplate template = templates.getOrCreate(single.getTemplateLocation());
            StructurePlaceSettings settings = new StructurePlaceSettings().setRotation(rotation);
            for (StructureTemplate.StructureBlockInfo info :
                    template.filterBlocks(position, settings, Blocks.CHEST, true)) {
                CompoundTag nbt = info.nbt();
                String lootTable = nbt == null ? "" : nbt.getStringOr("LootTable", "");
                output.add(new RawChest(info.pos(), lootTable));
            }
        } else if (element instanceof ListPoolElement list) {
            for (StructurePoolElement child : list.getElements()) {
                collectFromElement(child, position, rotation, templates, output);
            }
        }
    }

    private record RawChest(BlockPos pos, String lootTable) {
    }
}
