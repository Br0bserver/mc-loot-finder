package dev.br0b.mclootfinder.core;

import java.util.List;

/** Version-pinned structure pipeline configuration. */
public record StructureSpec(
        String name,
        String structureId,
        String dimensionId,
        VersionProfile.PlacementProfile placement,
        int decorationStep,
        int indexWithinStep,
        ScannerKind scannerKind,
        ContainerSeedShortcut containerSeedShortcut,
        List<SelectionEntry> structureSetEntries,
        List<String> lootTables,
        String defaultTargetItem
) {
    public StructureSpec {
        if (placement == null) {
            throw new IllegalArgumentException("structure placement must be specified");
        }
        structureSetEntries = List.copyOf(structureSetEntries);
        lootTables = List.copyOf(lootTables);
        if (structureSetEntries.stream().noneMatch(entry -> entry.structureId().equals(structureId))) {
            throw new IllegalArgumentException("structure-set entries must contain " + structureId);
        }
    }

    public VersionProfile.StructureProfile randomSpreadPlacement() {
        if (placement instanceof VersionProfile.StructureProfile profile) {
            return profile;
        }
        throw new IllegalArgumentException(name + " does not use random-spread placement");
    }

    public record SelectionEntry(String structureId, int weight, boolean accepted) {
        public SelectionEntry(String structureId, int weight) {
            this(structureId, weight, true);
        }

        public SelectionEntry {
            if (weight <= 0) {
                throw new IllegalArgumentException("structure selection weight must be positive");
            }
        }
    }

    public enum ScannerKind {
        JIGSAW_FAST,
        VANILLA_PLACEMENT,
        PLACED_FEATURE
    }

    /** Optional shortcut for deriving a seed without placing the structure. */
    public enum ContainerSeedShortcut {
        DIRECT,
        DESERT_PYRAMID,
        NONE
    }
}
