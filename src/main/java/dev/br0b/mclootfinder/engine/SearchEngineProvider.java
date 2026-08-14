package dev.br0b.mclootfinder.engine;

/** Provides a version-pinned structure search runtime. */
public interface SearchEngineProvider {
    String minecraftVersion();

    SearchEngine open(long worldSeed);
}
