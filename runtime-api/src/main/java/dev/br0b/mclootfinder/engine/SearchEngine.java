package dev.br0b.mclootfinder.engine;

import dev.br0b.mclootfinder.core.StructureSpec;
import dev.br0b.mclootfinder.vanilla.LootOracle;

public interface SearchEngine extends AutoCloseable {
    void verifyProfile(StructureSpec spec);

    StructureScan scan(StructureSpec spec, int chunkX, int chunkZ);

    LootOracle lootOracle();

    @Override
    void close();
}
