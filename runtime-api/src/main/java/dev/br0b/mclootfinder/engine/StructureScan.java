package dev.br0b.mclootfinder.engine;

import dev.br0b.mclootfinder.vanilla.ChestPrediction;

import java.util.List;

public record StructureScan(boolean validStructure, List<ChestPrediction> containers) {
    public StructureScan {
        containers = List.copyOf(containers);
    }

    public static StructureScan absent() {
        return new StructureScan(false, List.of());
    }
}
