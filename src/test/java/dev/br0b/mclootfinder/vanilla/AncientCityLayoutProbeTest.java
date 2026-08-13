package dev.br0b.mclootfinder.vanilla;

import net.minecraft.world.level.ChunkPos;
import net.minecraft.world.level.levelgen.structure.PoolElementStructurePiece;
import org.junit.jupiter.api.Test;

import java.io.IOException;
import java.nio.file.Files;
import java.nio.file.Path;

import static org.junit.jupiter.api.Assertions.assertEquals;

final class AncientCityLayoutProbeTest {
    @Test
    void printsFirstPieceAndLayoutSummary() throws IOException {
        try (var runtime = VanillaRuntime26_1_2.load(114514L)) {
            var start = runtime.generateAncientCity(new ChunkPos(244, 171));
            var first = (PoolElementStructurePiece) start.getPieces().getFirst();
            var geometry = new StringBuilder();

            System.out.printf(
                    "VANILLA position=%s rotation=%s element=%s first_box=%s pieces=%d box=%s%n",
                    first.getPosition(),
                    first.getRotation(),
                    first.getElement(),
                    first.getBoundingBox(),
                    start.getPieces().size(),
                    start.getBoundingBox()
            );
            for (int index = 0; index < start.getPieces().size(); index++) {
                var piece = (PoolElementStructurePiece) start.getPieces().get(index);
                System.out.printf(
                        "VANILLA_PIECE index=%03d position=%s rotation=%s element=%s box=%s%n",
                        index,
                        piece.getPosition(),
                        piece.getRotation(),
                        piece.getElement(),
                        piece.getBoundingBox()
                );
                var box = piece.getBoundingBox();
                geometry.append(String.format(
                        "%03d\t%d\t%d\t%d\t%s\t%d\t%d\t%d\t%d\t%d\t%d%n",
                        index,
                        piece.getPosition().getX(),
                        piece.getPosition().getY(),
                        piece.getPosition().getZ(),
                        rotationName(piece.getRotation().toString()),
                        box.minX(),
                        box.minY(),
                        box.minZ(),
                        box.maxX(),
                        box.maxY(),
                        box.maxZ()
                ));
            }

            assertEquals(
                    Files.readString(Path.of("tools/pumpkin-jigsaw-probe/ancient-city-114514.tsv")),
                    geometry.toString()
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

    private static String rotationName(String rotation) {
        return switch (rotation) {
            case "NONE" -> "None";
            case "CLOCKWISE_90" -> "Clockwise90";
            case "CLOCKWISE_180" -> "Rotate180";
            case "COUNTERCLOCKWISE_90" -> "CounterClockwise90";
            default -> throw new IllegalArgumentException("Unknown rotation: " + rotation);
        };
    }
}
