package dev.br0b.mclootfinder.cli;

import dev.br0b.mclootfinder.core.StructureSpec;
import dev.br0b.mclootfinder.core.VersionProfile;
import dev.br0b.mclootfinder.core.Versions;
import dev.br0b.mclootfinder.core.structure.RandomSpreadLocator;
import dev.br0b.mclootfinder.core.structure.StructureCandidate;
import dev.br0b.mclootfinder.vanilla.ChestPrediction;
import dev.br0b.mclootfinder.vanilla.JsonLootTableOracle26_1_2;
import dev.br0b.mclootfinder.vanilla.StructureChestScanner;
import dev.br0b.mclootfinder.vanilla.VanillaRuntime26_1_2;
import net.minecraft.world.level.ChunkPos;

import java.io.PrintStream;
import java.util.ArrayList;
import java.util.HashMap;
import java.util.List;
import java.util.Map;

public final class Main {
    private Main() {
    }

    public static void main(String[] args) {
        int exitCode;
        try {
            exitCode = run(args, System.out, System.err);
        } catch (IllegalArgumentException exception) {
            System.err.println("error: " + exception.getMessage());
            System.err.println("Run 'mc-loot-finder help' for usage.");
            exitCode = 2;
        } catch (IllegalStateException exception) {
            System.err.println("error: " + exception.getMessage());
            exitCode = 1;
        }
        if (exitCode != 0) {
            System.exit(exitCode);
        }
    }

    static int run(String[] args, PrintStream out, PrintStream err) {
        if (args.length == 0 || "help".equals(args[0]) || "--help".equals(args[0])) {
            printHelp(out);
            return 0;
        }
        return switch (args[0]) {
            case "candidates" -> candidates(new Arguments(args, 1), out);
            case "container-seed" -> containerSeed(new Arguments(args, 1), out);
            case "chests" -> chests(new Arguments(args, 1), out);
            case "find" -> findLoot(new Arguments(args, 1), out);
            case "loot" -> rollLoot(new Arguments(args, 1), out);
            case "explain" -> explain(new Arguments(args, 1), out);
            default -> throw new IllegalArgumentException("Unknown command: " + args[0]);
        };
    }

    private static int candidates(Arguments arguments, PrintStream out) {
        VersionProfile version = Versions.require(arguments.text("version", "26.1.2"));
        StructureSpec spec = structure(arguments, version);
        long worldSeed = arguments.longValue("seed");
        int centerX = arguments.intValue("center-x", 0);
        int centerZ = arguments.intValue("center-z", 0);
        int radius = arguments.intValue("radius", 5_000);
        int limit = arguments.intValue("limit", 100);
        requireNonNegativeLimit(limit);
        List<StructureCandidate> candidates = RandomSpreadLocator.locate(
                worldSeed, centerX, centerZ, radius, spec
        );

        if (arguments.flag("json")) {
            printCandidatesJson(out, version, spec, worldSeed, candidates, limit);
            return 0;
        }
        out.printf("Minecraft Java %s %s placement candidates%n",
                version.minecraftVersion(), spec.name());
        out.printf("seed=%d center=(%d,%d) radius=%d blocks candidates=%d%n",
                worldSeed, centerX, centerZ, radius, candidates.size());
        out.println("status=CANDIDATE_ONLY (biome/structure and weighted-set selection not validated)");
        out.println("chunk_x chunk_z block_x block_z distance");
        candidates.stream().limit(limit).forEach(candidate -> out.printf(
                "%7d %7d %7d %7d %.1f%n",
                candidate.chunkX(), candidate.chunkZ(), candidate.blockX(), candidate.blockZ(),
                Math.sqrt(candidate.squaredDistanceFromCenter())
        ));
        return 0;
    }

    private static void printCandidatesJson(
            PrintStream out,
            VersionProfile version,
            StructureSpec spec,
            long worldSeed,
            List<StructureCandidate> candidates,
            int limit
    ) {
        out.printf("{\"version\":\"%s\",\"structure\":\"%s\",\"seed\":%d,"
                        + "\"status\":\"candidate_only\",\"candidates\":[",
                version.minecraftVersion(), spec.name(), worldSeed);
        int count = Math.min(limit, candidates.size());
        for (int index = 0; index < count; index++) {
            StructureCandidate candidate = candidates.get(index);
            if (index > 0) {
                out.print(',');
            }
            out.printf("{\"chunk_x\":%d,\"chunk_z\":%d,\"block_x\":%d,"
                            + "\"block_z\":%d,\"distance\":%.3f}",
                    candidate.chunkX(), candidate.chunkZ(), candidate.blockX(), candidate.blockZ(),
                    Math.sqrt(candidate.squaredDistanceFromCenter()));
        }
        out.println("]}");
    }

    private static int containerSeed(Arguments arguments, PrintStream out) {
        VersionProfile version = Versions.require(arguments.text("version", "26.1.2"));
        StructureSpec spec = structure(arguments, version);
        long worldSeed = arguments.longValue("seed");
        int chunkX = arguments.intValue("chunk-x", 0);
        int chunkZ = arguments.intValue("chunk-z", 0);
        int structureIndex = arguments.intValue("structure-index", spec.indexWithinStep());
        int step = arguments.intValue("step", spec.decorationStep());
        int ordinal = arguments.intValue("ordinal", 0);
        long lootTableSeed = StructureChestScanner.containerLootSeed(
                worldSeed, spec, chunkX, chunkZ, structureIndex, step, ordinal
        );
        if (arguments.flag("json")) {
            out.printf("{\"version\":\"%s\",\"structure\":\"%s\",\"world_seed\":%d,"
                            + "\"chunk_x\":%d,\"chunk_z\":%d,\"structure_index\":%d,"
                            + "\"step\":%d,\"ordinal\":%d,\"loot_table_seed\":%d}%n",
                    version.minecraftVersion(), spec.name(), worldSeed, chunkX, chunkZ,
                    structureIndex, step, ordinal, lootTableSeed);
        } else {
            out.printf("LootTableSeed=%d%n", lootTableSeed);
            out.println("Ordinal is zero-based among containers in this decoration RNG stream.");
        }
        return 0;
    }

    private static int chests(Arguments arguments, PrintStream out) {
        VersionProfile version = Versions.require(arguments.text("version", "26.1.2"));
        StructureSpec spec = structure(arguments, version);
        long worldSeed = arguments.longValue("seed");
        int centerX = arguments.intValue("center-x", 0);
        int centerZ = arguments.intValue("center-z", 0);
        int radius = arguments.intValue("radius", 2_000);
        int limit = arguments.intValue("limit", 100);
        requireNonNegativeLimit(limit);
        List<StructureCandidate> candidates = RandomSpreadLocator.locate(
                worldSeed, centerX, centerZ, radius, spec
        );

        boolean json = arguments.flag("json");
        if (!json) {
            out.printf("Loading vanilla Minecraft Java %s worldgen data...%n",
                    version.minecraftVersion());
        }
        int validStructures = 0;
        List<ChestPrediction> predictions = new ArrayList<>();
        try (VanillaRuntime26_1_2 runtime = VanillaRuntime26_1_2.load(worldSeed)) {
            runtime.verifyStructureProfile(spec);
            for (StructureCandidate candidate : candidates) {
                ChunkPos candidateChunk = new ChunkPos(candidate.chunkX(), candidate.chunkZ());
                if (!runtime.isStructurePlacementChunk(spec, candidateChunk)) {
                    continue;
                }
                var start = runtime.generateSelectedStructure(
                        spec, candidateChunk
                );
                if (!start.isValid()) {
                    continue;
                }
                validStructures++;
                predictions.addAll(StructureChestScanner.scan(
                        worldSeed, spec, start, runtime
                ));
            }
        }
        requireUnambiguousContainerStreams(spec, predictions);

        if (json) {
            out.printf("{\"version\":\"%s\",\"structure\":\"%s\",\"seed\":%d,"
                            + "\"placement_candidates\":%d,\"valid_structures\":%d,"
                            + "\"chest_count\":%d,\"chests\":[",
                    version.minecraftVersion(), spec.name(), worldSeed, candidates.size(),
                    validStructures, predictions.size());
            for (int index = 0; index < Math.min(limit, predictions.size()); index++) {
                ChestPrediction chest = predictions.get(index);
                if (index != 0) {
                    out.print(',');
                }
                out.printf("{\"x\":%d,\"y\":%d,\"z\":%d,\"loot_table\":\"%s\","
                                + "\"loot_seed\":%d,\"start_chunk_x\":%d,"
                                + "\"start_chunk_z\":%d,\"ordinal\":%d}",
                        chest.x(), chest.y(), chest.z(), chest.lootTable(), chest.lootTableSeed(),
                        chest.structureChunkX(), chest.structureChunkZ(),
                        chest.containerOrdinalInDecorationChunk());
            }
            out.println("]}");
        } else {
            out.println("x y z loot_table loot_seed start_chunk ordinal");
            predictions.stream().limit(limit).forEach(chest -> out.printf(
                    "%d %d %d %s %d (%d,%d) %d%n",
                    chest.x(), chest.y(), chest.z(), chest.lootTable(), chest.lootTableSeed(),
                    chest.structureChunkX(), chest.structureChunkZ(),
                    chest.containerOrdinalInDecorationChunk()
            ));
            out.printf("placement_candidates=%d valid_structures=%d chest_rows=%d shown=%d%n",
                    candidates.size(), validStructures, predictions.size(),
                    Math.min(predictions.size(), limit));
            out.println("No target chunks were generated or loaded; starts were built in memory.");
        }
        return 0;
    }

    private static int findLoot(Arguments arguments, PrintStream out) {
        VersionProfile version = Versions.require(arguments.text("version", "26.1.2"));
        StructureSpec spec = structure(arguments, version);
        long worldSeed = arguments.longValue("seed");
        String target = arguments.text("item", spec.defaultTargetItem());
        requireIdentifier(target, "--item");
        int centerX = arguments.intValue("center-x", 0);
        int centerZ = arguments.intValue("center-z", 0);
        int radius = arguments.intValue("radius", 5_000);
        int limit = arguments.intValue("limit", 20);
        requireNonNegativeLimit(limit);
        List<StructureCandidate> candidates = RandomSpreadLocator.locate(
                worldSeed, centerX, centerZ, radius, spec
        );

        boolean json = arguments.flag("json");
        if (!json) {
            out.printf("Searching Minecraft Java %s seed %d %s for %s...%n",
                    version.minecraftVersion(), worldSeed, spec.name(), target);
        }
        int validStructures = 0;
        int checkedChests = 0;
        int unpredictableZeroSeeds = 0;
        List<ChestPrediction> allChests = new ArrayList<>();
        List<ChestPrediction> matches = new ArrayList<>();
        try (VanillaRuntime26_1_2 runtime = VanillaRuntime26_1_2.load(worldSeed)) {
            runtime.verifyStructureProfile(spec);
            JsonLootTableOracle26_1_2 oracle = new JsonLootTableOracle26_1_2(runtime.registries());
            for (StructureCandidate candidate : candidates) {
                ChunkPos candidateChunk = new ChunkPos(candidate.chunkX(), candidate.chunkZ());
                if (!runtime.isStructurePlacementChunk(spec, candidateChunk)) {
                    continue;
                }
                var start = runtime.generateSelectedStructure(
                        spec, candidateChunk
                );
                if (!start.isValid()) {
                    continue;
                }
                validStructures++;
                allChests.addAll(StructureChestScanner.scan(
                        worldSeed, spec, start, runtime
                ));
            }
            requireUnambiguousContainerStreams(spec, allChests);
            for (ChestPrediction chest : allChests) {
                if (!spec.lootTables().contains(chest.lootTable())) {
                    continue;
                }
                checkedChests++;
                if (chest.lootTableSeed() == 0L) {
                    unpredictableZeroSeeds++;
                    continue;
                }
                if (oracle.contains(chest.lootTable(), chest.lootTableSeed(), target)) {
                    matches.add(chest);
                }
            }
        }

        if (json) {
            out.printf("{\"version\":\"%s\",\"structure\":\"%s\",\"seed\":%d,"
                            + "\"item\":\"%s\",\"placement_candidates\":%d,"
                            + "\"valid_structures\":%d,\"checked_chests\":%d,\"hits\":%d,"
                            + "\"unpredictable_zero_seeds\":%d,\"matches\":[",
                    version.minecraftVersion(), spec.name(), worldSeed, target, candidates.size(),
                    validStructures, checkedChests, matches.size(), unpredictableZeroSeeds);
            for (int index = 0; index < Math.min(limit, matches.size()); index++) {
                ChestPrediction chest = matches.get(index);
                if (index != 0) {
                    out.print(',');
                }
                out.printf("{\"x\":%d,\"y\":%d,\"z\":%d,\"loot_table\":\"%s\","
                                + "\"loot_seed\":%d,\"start_chunk_x\":%d,"
                                + "\"start_chunk_z\":%d}",
                        chest.x(), chest.y(), chest.z(), chest.lootTable(), chest.lootTableSeed(),
                        chest.structureChunkX(), chest.structureChunkZ());
            }
            out.println("]}");
        } else {
            out.println("x y z loot_table loot_seed start_chunk");
            matches.stream().limit(limit).forEach(chest -> out.printf(
                    "%d %d %d %s %d (%d,%d)%n",
                    chest.x(), chest.y(), chest.z(), chest.lootTable(), chest.lootTableSeed(),
                    chest.structureChunkX(), chest.structureChunkZ()
            ));
            out.printf("placement_candidates=%d valid_structures=%d checked_chests=%d "
                            + "hits=%d shown=%d%n",
                    candidates.size(), validStructures, checkedChests, matches.size(),
                    Math.min(matches.size(), limit));
            if (unpredictableZeroSeeds != 0) {
                out.printf("warning: skipped %d zero-sentinel LootTableSeed chest(s)%n",
                        unpredictableZeroSeeds);
            }
        }
        return matches.isEmpty() ? 1 : 0;
    }

    private static int rollLoot(Arguments arguments, PrintStream out) {
        VersionProfile version = Versions.require(arguments.text("version", "26.1.2"));
        String table = arguments.text("table", "minecraft:chests/ancient_city");
        requireIdentifier(table, "--table");
        List<String> supportedTables = version.structures().stream()
                .flatMap(spec -> spec.lootTables().stream())
                .distinct()
                .toList();
        if (!supportedTables.contains(table)) {
            throw new IllegalArgumentException(
                    "Unsupported loot table: " + table + "; use one listed by 'explain'"
            );
        }
        long lootSeed = arguments.longValue("loot-seed");
        try (VanillaRuntime26_1_2 runtime = VanillaRuntime26_1_2.load(0L)) {
            var stacks = new JsonLootTableOracle26_1_2(runtime.registries()).roll(table, lootSeed);
            if (arguments.flag("json")) {
                out.printf("{\"version\":\"26.1.2\",\"loot_table\":\"%s\","
                        + "\"loot_seed\":%d,\"items\":[", table, lootSeed);
                for (int index = 0; index < stacks.size(); index++) {
                    var stack = stacks.get(index);
                    if (index != 0) {
                        out.print(',');
                    }
                    out.printf("{\"item\":\"%s\",\"count\":%d}",
                            stack.item(), stack.count());
                }
                out.println("]}");
            } else {
                stacks.forEach(stack -> out.printf("%s x%d%n", stack.item(), stack.count()));
            }
        }
        return 0;
    }

    private static int explain(Arguments arguments, PrintStream out) {
        VersionProfile version = Versions.require(arguments.text("version", "26.1.2"));
        out.printf("Minecraft Java %s support matrix:%n", version.minecraftVersion());
        for (StructureSpec spec : version.structures()) {
            out.printf("  exact: %s (%s, %d loot table(s))%n",
                    spec.name(), spec.dimensionId(), spec.lootTables().size());
        }
        out.println("  exact: random-spread candidates and weighted structure-set selection");
        out.println("  exact: vanilla biome/structure generation -> chest coordinates and LootTableSeed");
        out.println("  exact: bundled vanilla JSON loot tables -> aggregate items / target match");
        return 0;
    }

    private static StructureSpec structure(Arguments arguments, VersionProfile version) {
        return version.structure(arguments.text("structure", "ancient_city"));
    }

    private static void printHelp(PrintStream out) {
        out.println("mc-loot-finder — deterministic Minecraft Java loot research CLI");
        out.println();
        out.println("Structures: " + Versions.V26_1_2.structures().stream()
                .map(StructureSpec::name)
                .collect(java.util.stream.Collectors.joining(", "))
                + " (default: ancient_city)");
        out.println("Commands:");
        out.println("  candidates --seed N [--structure NAME --center-x X --center-z Z --radius BLOCKS --limit N --json]");
        out.println("  container-seed --seed N --chunk-x X --chunk-z Z [--structure NAME --ordinal N --json]");
        out.println("  chests --seed N [--structure NAME --center-x X --center-z Z --radius BLOCKS --limit N --json]");
        out.println("  find --seed N [--structure NAME --item ITEM_ID --center-x X --center-z Z --radius BLOCKS --limit N --json]");
        out.println("  loot --loot-seed N [--table LOOT_TABLE_ID --json]");
        out.println("  explain [--version 26.1.2]");
        out.println();
        out.println("Only Minecraft Java 26.1.2 is currently supported.");
    }

    private static void requireNonNegativeLimit(int limit) {
        if (limit < 0) {
            throw new IllegalArgumentException("--limit must be non-negative");
        }
    }

    private static void requireIdentifier(String value, String option) {
        if (!value.matches("[a-z0-9_.-]+:[a-z0-9_./-]+")) {
            throw new IllegalArgumentException(option + " must be a namespaced Minecraft id");
        }
    }

    private static void requireUnambiguousContainerStreams(
            StructureSpec spec,
            List<ChestPrediction> chests
    ) {
        Map<Long, Long> ownerByDecorationChunk = new HashMap<>();
        for (ChestPrediction chest : chests) {
            long decorationChunk = ChunkPos.pack(
                    Math.floorDiv(chest.x(), 16), Math.floorDiv(chest.z(), 16)
            );
            long startChunk = ChunkPos.pack(chest.structureChunkX(), chest.structureChunkZ());
            Long previous = ownerByDecorationChunk.putIfAbsent(decorationChunk, startChunk);
            if (previous != null && previous.longValue() != startChunk) {
                throw new IllegalArgumentException(
                        "Two " + spec.name() + " starts consume containers in decoration chunk ("
                                + ChunkPos.getX(decorationChunk) + ","
                                + ChunkPos.getZ(decorationChunk)
                                + "); cross-start stream merging is not implemented"
                );
            }
        }
    }
}
