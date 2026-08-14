package dev.br0b.mclootfinder.engine;

import dev.br0b.mclootfinder.core.StructureSpec;
import dev.br0b.mclootfinder.core.structure.StructureCandidate;
import dev.br0b.mclootfinder.vanilla.LootOracle;

import java.util.List;

public interface SearchEngine extends AutoCloseable {
    void verifyProfile(StructureSpec spec);

    List<StructureCandidate> locateCandidates(
            StructureSpec spec,
            int centerBlockX,
            int centerBlockZ,
            int radiusBlocks
    );

    StructureScan scan(StructureSpec spec, int chunkX, int chunkZ);

    LootOracle lootOracle();

    @Override
    void close();
}
