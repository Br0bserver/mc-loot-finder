package dev.br0b.mclootfinder.vanilla;

public record ChestPrediction(
        int structureChunkX,
        int structureChunkZ,
        int x,
        int y,
        int z,
        String lootTable,
        int containerOrdinalInDecorationChunk,
        long lootTableSeed,
        LootSourceKind sourceKind,
        String sourceBlock
) {
    public ChestPrediction(
            int structureChunkX,
            int structureChunkZ,
            int x,
            int y,
            int z,
            String lootTable,
            int containerOrdinalInDecorationChunk,
            long lootTableSeed
    ) {
        this(
                structureChunkX,
                structureChunkZ,
                x,
                y,
                z,
                lootTable,
                containerOrdinalInDecorationChunk,
                lootTableSeed,
                LootSourceKind.CONTAINER,
                ""
        );
    }

    public enum LootSourceKind {
        CONTAINER,
        ARCHAEOLOGY
    }
}
