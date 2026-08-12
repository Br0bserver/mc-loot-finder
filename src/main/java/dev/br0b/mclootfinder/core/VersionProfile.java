package dev.br0b.mclootfinder.core;

import java.util.List;

/** Version-specific constants that are data-driven in vanilla Minecraft. */
public record VersionProfile(
        String minecraftVersion,
        List<StructureSpec> structures
) {
    public VersionProfile {
        structures = List.copyOf(structures);
        if (structures.isEmpty()) {
            throw new IllegalArgumentException("version must support at least one structure");
        }
        long distinctNames = structures.stream().map(StructureSpec::name).distinct().count();
        if (distinctNames != structures.size()) {
            throw new IllegalArgumentException("structure names must be unique");
        }
    }

    public record StructureProfile(
            int spacing,
            int separation,
            int salt,
            SpreadType spreadType
    ) {
        public StructureProfile(int spacing, int separation, int salt) {
            this(spacing, separation, salt, SpreadType.LINEAR);
        }

        public StructureProfile {
            if (spacing <= 0) {
                throw new IllegalArgumentException("spacing must be positive");
            }
            if (separation < 0 || separation >= spacing) {
                throw new IllegalArgumentException("separation must be in [0, spacing)");
            }
        }
    }

    public enum SpreadType {
        LINEAR,
        TRIANGULAR
    }

    public StructureSpec structure(String name) {
        String normalized = name.startsWith("minecraft:")
                ? name.substring("minecraft:".length())
                : name;
        return structures.stream()
                .filter(spec -> spec.name().equals(normalized)
                        || structurePath(spec.structureId()).equals(normalized))
                .findFirst()
                .orElseThrow(() -> new IllegalArgumentException(
                        "Unsupported structure: " + name + "; supported: "
                                + structures.stream()
                                .map(StructureSpec::name)
                                .collect(java.util.stream.Collectors.joining(", "))
                ));
    }

    public StructureSpec ancientCity() {
        return structure("ancient_city");
    }

    public StructureSpec bastionRemnant() {
        return structure("bastion_remnant");
    }

    public StructureSpec desertPyramid() {
        return structure("desert_pyramid");
    }

    public StructureSpec woodlandMansion() {
        return structure("woodland_mansion");
    }

    public StructureSpec junglePyramid() {
        return structure("jungle_pyramid");
    }

    public StructureSpec igloo() {
        return structure("igloo");
    }

    public StructureSpec endCity() {
        return structure("end_city");
    }

    private static String structurePath(String id) {
        int separator = id.indexOf(':');
        return separator < 0 ? id : id.substring(separator + 1);
    }
}
