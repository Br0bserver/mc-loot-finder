package dev.br0b.mclootfinder.cli;

import org.junit.jupiter.api.Test;

import java.io.ByteArrayOutputStream;
import java.io.PrintStream;
import java.nio.charset.StandardCharsets;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertThrows;
import static org.junit.jupiter.api.Assertions.assertTrue;

class MainTest {
    @Test
    void candidatesJsonIsMachineReadableShape() {
        ByteArrayOutputStream bytes = new ByteArrayOutputStream();
        int result = Main.run(new String[]{
                "candidates", "--seed", "0", "--radius", "5000", "--limit", "2", "--json"
        }, new PrintStream(bytes, true, StandardCharsets.UTF_8), System.err);

        String output = bytes.toString(StandardCharsets.UTF_8);
        assertEquals(0, result);
        assertTrue(output.startsWith("{\"version\":\"26.1.2\""));
        assertTrue(output.contains("\"structure\":\"ancient_city\""));
        assertTrue(output.contains("\"status\":\"candidate_only\""));
        assertTrue(output.endsWith("}\n"));
    }

    @Test
    void bastionCandidatePipelineUsesItsOwnProfile() {
        ByteArrayOutputStream bytes = new ByteArrayOutputStream();
        int result = Main.run(new String[]{
                "candidates", "--seed", "0", "--structure", "bastion_remnant",
                "--radius", "2000", "--limit", "1", "--json"
        }, new PrintStream(bytes, true, StandardCharsets.UTF_8), System.err);

        String output = bytes.toString(StandardCharsets.UTF_8);
        assertEquals(0, result);
        assertTrue(output.contains("\"structure\":\"bastion_remnant\""));
        assertTrue(output.contains("\"chunk_x\":"));
    }

    @Test
    void desertPyramidCandidatePipelineUsesItsOwnProfile() {
        ByteArrayOutputStream bytes = new ByteArrayOutputStream();
        int result = Main.run(new String[]{
                "candidates", "--seed", "0", "--structure", "desert_pyramid",
                "--radius", "2000", "--limit", "1", "--json"
        }, new PrintStream(bytes, true, StandardCharsets.UTF_8), System.err);

        String output = bytes.toString(StandardCharsets.UTF_8);
        assertEquals(0, result);
        assertTrue(output.contains("\"structure\":\"desert_pyramid\""));
        assertTrue(output.contains("\"chunk_x\":"));
    }

    @Test
    void woodlandMansionCandidatePipelineUsesTriangularProfile() {
        ByteArrayOutputStream bytes = new ByteArrayOutputStream();
        int result = Main.run(new String[]{
                "candidates", "--seed", "0", "--structure", "woodland_mansion",
                "--radius", "5000", "--limit", "1", "--json"
        }, new PrintStream(bytes, true, StandardCharsets.UTF_8), System.err);

        String output = bytes.toString(StandardCharsets.UTF_8);
        assertEquals(0, result);
        assertTrue(output.contains("\"structure\":\"woodland_mansion\""));
        assertTrue(output.contains("\"chunk_x\":"));
    }

    @Test
    void mansionContainerSeedShortcutFailsClosed() {
        IllegalArgumentException error = assertThrows(IllegalArgumentException.class, () ->
                Main.run(new String[]{
                        "container-seed", "--seed", "0", "--structure", "woodland_mansion",
                        "--chunk-x", "0", "--chunk-z", "0"
                }, System.out, System.err));
        assertTrue(error.getMessage().contains("use 'chests'"));
    }

    @Test
    void helpListsCatalogStructures() {
        ByteArrayOutputStream bytes = new ByteArrayOutputStream();
        assertEquals(0, Main.run(new String[]{"help"},
                new PrintStream(bytes, true, StandardCharsets.UTF_8), System.err));
        String output = bytes.toString(StandardCharsets.UTF_8);
        assertTrue(output.contains("jungle_pyramid"));
        assertTrue(output.contains("igloo"));
    }
}
