package dev.br0b.mclootfinder.vanilla;

public record ChestPrediction(
        int structureChunkX,
        int structureChunkZ,
        int x,
        int y,
        int z,
        String lootTable,
        int containerOrdinalInDecorationChunk,
        long lootTableSeed
) {
}
