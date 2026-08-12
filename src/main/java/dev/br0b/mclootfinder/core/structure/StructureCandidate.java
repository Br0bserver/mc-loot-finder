package dev.br0b.mclootfinder.core.structure;

public record StructureCandidate(
        int chunkX,
        int chunkZ,
        int blockX,
        int blockZ,
        long squaredDistanceFromCenter
) {
}

