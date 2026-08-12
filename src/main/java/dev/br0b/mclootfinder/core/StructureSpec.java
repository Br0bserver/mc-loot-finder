package dev.br0b.mclootfinder.core;

import java.util.List;

/** Version-pinned structure pipeline configuration. */
public record StructureSpec(
        String name,
        String structureId,
        String dimensionId,
        VersionProfile.StructureProfile placement,
        int decorationStep,
        int indexWithinStep,
        ScannerKind scannerKind,
        ContainerSeedShortcut containerSeedShortcut,
        List<SelectionEntry> structureSetEntries,
        List<String> lootTables,
        String defaultTargetItem
) {
    public StructureSpec {
        structureSetEntries = List.copyOf(structureSetEntries);
        lootTables = List.copyOf(lootTables);
        if (structureSetEntries.stream().noneMatch(entry -> entry.structureId().equals(structureId))) {
            throw new IllegalArgumentException("structure-set entries must contain " + structureId);
        }
    }

    public record SelectionEntry(String structureId, int weight) {
        public SelectionEntry {
            if (weight <= 0) {
                throw new IllegalArgumentException("structure selection weight must be positive");
            }
        }
    }

    public enum ScannerKind {
        JIGSAW_FAST,
        VANILLA_PLACEMENT
    }

    /** Optional shortcut for deriving a seed without placing the structure. */
    public enum ContainerSeedShortcut {
        DIRECT,
        DESERT_PYRAMID,
        NONE
    }
}
