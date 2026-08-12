package dev.br0b.mclootfinder.vanilla;

import dev.br0b.mclootfinder.core.VersionProfile;
import net.minecraft.world.level.levelgen.structure.StructureStart;
import net.minecraft.world.level.levelgen.structure.templatesystem.StructureTemplateManager;

import java.util.List;

/** Backward-compatible ancient-city facade over the generic Jigsaw scanner. */
public final class AncientCityChestScanner {
    private AncientCityChestScanner() {
    }

    public static List<ChestPrediction> scan(
            long worldSeed,
            VersionProfile version,
            StructureStart start,
            StructureTemplateManager templates
    ) {
        return JigsawChestScanner.scan(worldSeed, version.ancientCity(), start, templates);
    }
}
