package dev.br0b.mclootfinder.runtime;

import java.nio.file.Path;

public record RuntimeStatus(
        String minecraftVersion,
        int recipeVersion,
        Path cacheDirectory,
        boolean sourceReady,
        boolean generatedRuntimeReady
) {
}
