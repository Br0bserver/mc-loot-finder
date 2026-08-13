package dev.br0b.mclootfinder.vanilla;

import net.minecraft.world.level.ChunkPos;
import net.minecraft.world.level.levelgen.structure.PoolElementStructurePiece;
import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertEquals;

final class AncientCityLayoutProbeTest {
    @Test
    void printsFirstPieceAndLayoutSummary() {
        try (var runtime = VanillaRuntime26_1_2.load(114514L)) {
            var start = runtime.generateAncientCity(new ChunkPos(244, 171));
            var first = (PoolElementStructurePiece) start.getPieces().getFirst();

            System.out.printf(
                    "VANILLA position=%s rotation=%s element=%s first_box=%s pieces=%d box=%s%n",
                    first.getPosition(),
                    first.getRotation(),
                    first.getElement(),
                    first.getBoundingBox(),
                    start.getPieces().size(),
                    start.getBoundingBox()
            );

            assertEquals(90, start.getPieces().size());
            assertEquals(3787, start.getBoundingBox().minX());
            assertEquals(-64, start.getBoundingBox().minY());
            assertEquals(2609, start.getBoundingBox().minZ());
            assertEquals(4025, start.getBoundingBox().maxX());
            assertEquals(-10, start.getBoundingBox().maxY());
            assertEquals(2857, start.getBoundingBox().maxZ());
        }
    }
}
