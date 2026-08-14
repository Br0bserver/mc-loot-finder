package dev.br0b.mclootfinder.vanilla;

import dev.br0b.mclootfinder.engine.SearchEngine;
import dev.br0b.mclootfinder.engine.SearchEngineProvider;

/** Full vanilla runtime provider retained as the correctness oracle. */
public final class VanillaSearchEngineProvider implements SearchEngineProvider {
    @Override
    public String minecraftVersion() {
        return "26.1.2";
    }

    @Override
    public SearchEngine open(long worldSeed) {
        return VanillaSearchEngine.load(worldSeed);
    }
}
