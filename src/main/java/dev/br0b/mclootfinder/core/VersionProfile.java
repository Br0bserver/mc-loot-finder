package dev.br0b.mclootfinder.core;

import java.util.List;

/** Version-specific constants that are data-driven in vanilla Minecraft. */
public record VersionProfile(
        String minecraftVersion,
        StructureSpec ancientCity,
        StructureSpec bastionRemnant,
        StructureSpec desertPyramid,
        StructureSpec woodlandMansion
) {
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
        return switch (normalized) {
            case "ancient_city" -> ancientCity;
            case "bastion_remnant" -> bastionRemnant;
            case "desert_pyramid" -> desertPyramid;
            case "woodland_mansion", "mansion" -> woodlandMansion;
            default -> throw new IllegalArgumentException(
                    "Unsupported structure: " + name
                            + "; supported: ancient_city, bastion_remnant, desert_pyramid, "
                            + "woodland_mansion"
            );
        };
    }

    public List<StructureSpec> structures() {
        return List.of(ancientCity, bastionRemnant, desertPyramid, woodlandMansion);
    }
}
