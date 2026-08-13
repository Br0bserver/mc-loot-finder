package dev.br0b.mclootfinder.cli;

import dev.br0b.mclootfinder.vanilla.ChestPrediction;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.Timeout;

import java.io.ByteArrayOutputStream;
import java.io.PrintStream;
import java.nio.charset.StandardCharsets;
import java.util.List;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertThrows;
import static org.junit.jupiter.api.Assertions.assertTrue;

class MainTest {
    @Test
    @Timeout(3)
    void lootCommandUsesStandaloneRuntime() {
        ByteArrayOutputStream bytes = new ByteArrayOutputStream();
        assertEquals(0, Main.run(new String[]{
                "loot", "--loot-seed", "1",
                "--table", "minecraft:chests/ancient_city", "--json"
        }, new PrintStream(bytes, true, StandardCharsets.UTF_8), System.err));

        String output = bytes.toString(StandardCharsets.UTF_8);
        assertTrue(output.contains("\"loot_seed\":1"));
        assertTrue(output.contains("minecraft:enchanted_golden_apple"));
    }

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
    void candidatesTextUsesBlocksInsteadOfAWidthDependentTable() {
        ByteArrayOutputStream bytes = new ByteArrayOutputStream();
        assertEquals(0, Main.run(new String[]{
                "candidates", "--seed", "0", "--radius", "5000", "--limit", "1"
        }, new PrintStream(bytes, true, StandardCharsets.UTF_8), System.err));
        String output = bytes.toString(StandardCharsets.UTF_8);
        assertTrue(output.contains("Search area: 5,000 blocks around (0, 0)"));
        assertTrue(output.contains("[1]\n  Chunk: ("));
        assertTrue(output.contains("Candidates are not verified structures."));
        assertTrue(!output.contains("chunk_x chunk_z"));
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
    void helpPointsToCommandsAndStructureCatalog() {
        ByteArrayOutputStream bytes = new ByteArrayOutputStream();
        assertEquals(0, Main.run(new String[]{"help"},
                new PrintStream(bytes, true, StandardCharsets.UTF_8), System.err));
        String output = bytes.toString(StandardCharsets.UTF_8);
        assertTrue(output.contains("find --seed N"));
        assertTrue(output.contains("Use 'explain' to list supported structures"));
    }

    @Test
    void explainCanQueryOneStructureAsJson() {
        ByteArrayOutputStream bytes = new ByteArrayOutputStream();
        assertEquals(0, Main.run(new String[]{
                "explain", "--structure", "trial_chambers", "--json"
        }, new PrintStream(bytes, true, StandardCharsets.UTF_8), System.err));
        String output = bytes.toString(StandardCharsets.UTF_8);
        assertTrue(output.contains("\"name\":\"trial_chambers\""));
        assertTrue(output.contains("minecraft:chests/trial_chambers/reward"));
        assertTrue(output.contains("\"default_item\":\"minecraft:trial_key\""));
    }

    @Test
    void explainListsCommandDefaults() {
        ByteArrayOutputStream bytes = new ByteArrayOutputStream();
        assertEquals(0, Main.run(new String[]{"explain"},
                new PrintStream(bytes, true, StandardCharsets.UTF_8), System.err));
        String output = bytes.toString(StandardCharsets.UTF_8);
        assertTrue(output.contains("candidates: ancient_city"));
        assertTrue(output.contains("find: ancient_city"));
    }

    @Test
    void emptyLootTableContainersStillDetectSharedRandomStreams() {
        var first = new ChestPrediction(1, 1, 32, 64, 32, "", 0, 1L);
        var second = new ChestPrediction(
                2, 2, 33, 64, 33, "minecraft:chests/trial_chambers/supply", 1, 2L
        );

        IllegalArgumentException error = assertThrows(IllegalArgumentException.class, () ->
                Main.requireUnambiguousContainerStreams(
                        "trial_chambers",
                        List.of(first, second)
                ));
        assertTrue(error.getMessage().contains("cross-start stream merging"));
    }
}
