package dev.br0b.mclootfinder.cli;

import dev.br0b.mclootfinder.core.StructureSpec;
import dev.br0b.mclootfinder.core.VersionProfile;
import dev.br0b.mclootfinder.core.Versions;
import dev.br0b.mclootfinder.core.structure.RandomSpreadLocator;
import dev.br0b.mclootfinder.core.structure.StructureCandidate;
import dev.br0b.mclootfinder.engine.SearchEngine;
import dev.br0b.mclootfinder.loot.StandaloneLootOracle26_1_2;
import dev.br0b.mclootfinder.vanilla.ChestPrediction;
import dev.br0b.mclootfinder.vanilla.StructureChestScanner;
import dev.br0b.mclootfinder.vanilla.VanillaSearchEngine;

import java.io.PrintStream;
import java.util.ArrayList;
import java.util.HashMap;
import java.util.List;
import java.util.Locale;
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
            case "archaeology" -> archaeology(new Arguments(args, 1), out);
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
        List<StructureCandidate> candidates = locateCandidates(
                worldSeed, centerX, centerZ, radius, spec
        );

        if (arguments.flag("json")) {
            printCandidatesJson(out, version, spec, worldSeed, candidates, limit);
            return 0;
        }
        printSearchHeader(out, version, spec, worldSeed, centerX, centerZ, radius);
        out.printf("Found %s%n%n", quantity(candidates.size(), "placement candidate"));
        int shown = Math.min(limit, candidates.size());
        for (int index = 0; index < shown; index++) {
            StructureCandidate candidate = candidates.get(index);
            out.printf("[%d]%n", index + 1);
            out.printf("  Chunk: (%d, %d)%n", candidate.chunkX(), candidate.chunkZ());
            out.printf("  Center: (%d, %d)%n", candidate.blockX(), candidate.blockZ());
            out.printf("  Distance: %s blocks%n%n",
                    decimal(Math.sqrt(candidate.squaredDistanceFromCenter())));
        }
        out.println("Candidates are not verified structures.");
        out.println("Use 'chests' or 'find' to verify them.");
        out.printf("Shown: %s of %s%n",
                number(shown), quantity(candidates.size(), "placement candidate"));
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
            out.printf("Minecraft Java %s%n", version.minecraftVersion());
            out.printf("Structure: %s%n", spec.name());
            out.printf("World seed: %d%n", worldSeed);
            out.printf("Decoration chunk: (%d, %d)%n", chunkX, chunkZ);
            out.printf("Container ordinal: %d%n%n", ordinal);
            out.printf("LootTableSeed: %d%n", lootTableSeed);
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
        boolean json = arguments.flag("json");
        if (!json) {
            printSearchHeader(out, version, spec, worldSeed, centerX, centerZ, radius);
            out.println("Searching...");
            out.println();
        }
        int validStructures = 0;
        List<StructureCandidate> candidates;
        List<ChestPrediction> predictions = new ArrayList<>();
        try (SearchEngine engine = VanillaSearchEngine.load(worldSeed)) {
            engine.verifyProfile(spec);
            candidates = engine.locateCandidates(spec, centerX, centerZ, radius);
            for (StructureCandidate candidate : candidates) {
                var scan = engine.scan(spec, candidate.chunkX(), candidate.chunkZ());
                if (!scan.validStructure()) {
                    continue;
                }
                validStructures++;
                predictions.addAll(scan.containers());
            }
        }
        requireUnambiguousContainerStreams(spec.name(), predictions);
        predictions = visibleContainers(predictions);

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
            out.printf("Found %s%n%n", quantity(predictions.size(), "container"));
            int shown = Math.min(limit, predictions.size());
            printContainers(out, predictions, shown, true);
            out.printf("Checked: %s, %s%n",
                    quantity(candidates.size(), "candidate"),
                    quantity(validStructures, "valid structure"));
            out.printf("Shown: %s of %s%n",
                    number(shown), quantity(predictions.size(), "container"));
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
        boolean json = arguments.flag("json");
        if (!json) {
            printSearchHeader(out, version, spec, worldSeed, centerX, centerZ, radius);
            out.printf("Item: %s%n%n", target);
            out.println("Searching...");
            out.println();
        }
        int validStructures = 0;
        int checkedChests = 0;
        int checkedArchaeology = 0;
        int unpredictableZeroSeeds = 0;
        List<StructureCandidate> candidates;
        List<ChestPrediction> allChests = new ArrayList<>();
        List<ChestPrediction> matches = new ArrayList<>();
        try (SearchEngine engine = VanillaSearchEngine.load(worldSeed)) {
            engine.verifyProfile(spec);
            candidates = engine.locateCandidates(spec, centerX, centerZ, radius);
            for (StructureCandidate candidate : candidates) {
                var scan = engine.scan(spec, candidate.chunkX(), candidate.chunkZ());
                if (!scan.validStructure()) {
                    continue;
                }
                validStructures++;
                allChests.addAll(scan.containers());
            }
            requireUnambiguousContainerStreams(spec.name(), allChests);
            allChests = visibleLootSources(allChests);
            for (ChestPrediction chest : allChests) {
                if (!spec.lootTables().contains(chest.lootTable())) {
                    continue;
                }
                if (chest.sourceKind() == ChestPrediction.LootSourceKind.ARCHAEOLOGY) {
                    checkedArchaeology++;
                } else {
                    checkedChests++;
                }
                if (chest.lootTableSeed() == 0L) {
                    unpredictableZeroSeeds++;
                    continue;
                }
                if (engine.lootOracle().contains(
                        chest.lootTable(), chest.lootTableSeed(), target
                )) {
                    matches.add(chest);
                }
            }
        }

        if (json) {
            out.printf("{\"version\":\"%s\",\"structure\":\"%s\",\"seed\":%d,"
                            + "\"item\":\"%s\",\"placement_candidates\":%d,"
                            + "\"valid_structures\":%d,\"checked_chests\":%d,"
                            + "\"checked_archaeology\":%d,\"checked_sources\":%d,\"hits\":%d,"
                            + "\"unpredictable_zero_seeds\":%d,\"matches\":[",
                    version.minecraftVersion(), spec.name(), worldSeed, target, candidates.size(),
                    validStructures, checkedChests, checkedArchaeology,
                    checkedChests + checkedArchaeology, matches.size(), unpredictableZeroSeeds);
            for (int index = 0; index < Math.min(limit, matches.size()); index++) {
                ChestPrediction chest = matches.get(index);
                if (index != 0) {
                    out.print(',');
                }
                out.printf("{\"x\":%d,\"y\":%d,\"z\":%d,\"loot_table\":\"%s\","
                                + "\"loot_seed\":%d,\"start_chunk_x\":%d,"
                                + "\"start_chunk_z\":%d",
                        chest.x(), chest.y(), chest.z(), chest.lootTable(), chest.lootTableSeed(),
                        chest.structureChunkX(), chest.structureChunkZ());
                if (chest.sourceKind() == ChestPrediction.LootSourceKind.ARCHAEOLOGY) {
                    out.printf(",\"source_kind\":\"archaeology\",\"block\":\"%s\"",
                            chest.sourceBlock());
                }
                out.print('}');
            }
            out.println("]}");
        } else {
            out.printf("Found %s%n%n", quantity(matches.size(), "match"));
            int shown = Math.min(limit, matches.size());
            printContainers(out, matches, shown, false);
            out.printf("Checked: %s, %s, %s, %s%n",
                    quantity(candidates.size(), "candidate"),
                    quantity(validStructures, "valid structure"),
                    quantity(checkedChests, "container"),
                    quantity(checkedArchaeology, "suspicious block"));
            out.printf("Shown: %s of %s%n", number(shown), quantity(matches.size(), "match"));
            if (unpredictableZeroSeeds != 0) {
                out.printf("Skipped: %s with LootTableSeed 0%n",
                        quantity(unpredictableZeroSeeds, "loot source"));
            }
        }
        return matches.isEmpty() ? 1 : 0;
    }

    private static int archaeology(Arguments arguments, PrintStream out) {
        VersionProfile version = Versions.require(arguments.text("version", "26.1.2"));
        StructureSpec spec = version.structure(
                arguments.text("structure", "desert_pyramid")
        );
        if (spec.lootTables().stream().noneMatch(table -> table.startsWith(
                "minecraft:archaeology/"
        ))) {
            throw new IllegalArgumentException(
                    "Archaeology is not available for " + spec.name()
            );
        }
        long worldSeed = arguments.longValue("seed");
        int centerX = arguments.intValue("center-x", 0);
        int centerZ = arguments.intValue("center-z", 0);
        int radius = arguments.intValue("radius", 2_000);
        int limit = arguments.intValue("limit", 100);
        requireNonNegativeLimit(limit);
        boolean json = arguments.flag("json");
        if (!json) {
            printSearchHeader(out, version, spec, worldSeed, centerX, centerZ, radius);
            out.println("Searching...");
            out.println();
        }

        int validStructures = 0;
        List<StructureCandidate> candidates;
        List<ChestPrediction> predictions = new ArrayList<>();
        try (SearchEngine engine = VanillaSearchEngine.load(worldSeed)) {
            engine.verifyProfile(spec);
            candidates = engine.locateCandidates(spec, centerX, centerZ, radius);
            for (StructureCandidate candidate : candidates) {
                var scan = engine.scan(spec, candidate.chunkX(), candidate.chunkZ());
                if (!scan.validStructure()) {
                    continue;
                }
                validStructures++;
                predictions.addAll(scan.containers());
            }
        }
        requireUnambiguousContainerStreams(spec.name(), predictions);
        predictions = visibleArchaeology(predictions);

        if (json) {
            out.printf("{\"version\":\"%s\",\"structure\":\"%s\",\"seed\":%d,"
                            + "\"placement_candidates\":%d,\"valid_structures\":%d,"
                            + "\"archaeology_count\":%d,\"blocks\":[",
                    version.minecraftVersion(), spec.name(), worldSeed, candidates.size(),
                    validStructures, predictions.size());
            for (int index = 0; index < Math.min(limit, predictions.size()); index++) {
                ChestPrediction source = predictions.get(index);
                if (index != 0) {
                    out.print(',');
                }
                out.printf("{\"x\":%d,\"y\":%d,\"z\":%d,\"block\":\"%s\","
                                + "\"loot_table\":\"%s\",\"loot_seed\":%d,"
                                + "\"start_chunk_x\":%d,\"start_chunk_z\":%d}",
                        source.x(), source.y(), source.z(), source.sourceBlock(),
                        source.lootTable(), source.lootTableSeed(),
                        source.structureChunkX(), source.structureChunkZ());
            }
            out.println("]}");
        } else {
            out.printf("Found %s%n%n", quantity(predictions.size(), "suspicious block"));
            int shown = Math.min(limit, predictions.size());
            printContainers(out, predictions, shown, false);
            out.printf("Checked: %s, %s%n",
                    quantity(candidates.size(), "candidate"),
                    quantity(validStructures, "valid structure"));
            out.printf("Shown: %s of %s%n",
                    number(shown), quantity(predictions.size(), "suspicious block"));
        }
        return 0;
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
        boolean json = arguments.flag("json");
        if (!json) {
            out.printf("Minecraft Java %s%n", version.minecraftVersion());
            out.printf("Loot table: %s%n", table);
            out.printf("Loot seed: %d%n%n", lootSeed);
            out.println("Generating...");
            out.println();
        }
        var stacks = new StandaloneLootOracle26_1_2().roll(table, lootSeed);
        if (json) {
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
            out.printf("Generated %s%n%n", quantity(stacks.size(), "stack"));
            for (int index = 0; index < stacks.size(); index++) {
                var stack = stacks.get(index);
                out.printf("[%d] %s x%d%n", index + 1, stack.item(), stack.count());
            }
        }
        return 0;
    }

    private static int explain(Arguments arguments, PrintStream out) {
        VersionProfile version = Versions.require(arguments.text("version", "26.1.2"));
        String structureName = arguments.text("structure", "");
        if (structureName.isEmpty()) {
            if (arguments.flag("json")) {
                out.printf("{\"version\":\"%s\",\"structures\":[",
                        version.minecraftVersion());
                for (int index = 0; index < version.structures().size(); index++) {
                    if (index != 0) {
                        out.print(',');
                    }
                    StructureSpec spec = version.structures().get(index);
                    out.printf("{\"name\":\"%s\",\"dimension\":\"%s\","
                                    + "\"default_item\":\"%s\",\"loot_tables\":%d}",
                            spec.name(), spec.dimensionId(), spec.defaultTargetItem(),
                            spec.lootTables().size());
                }
                out.println("]}");
                return 0;
            }
            out.printf("Minecraft Java %s%n", version.minecraftVersion());
            out.println();
            out.println("Command defaults:");
            out.println("  candidates: ancient_city, center (0, 0), radius 5,000, limit 100");
            out.println("  chests: ancient_city, center (0, 0), radius 2,000, limit 100");
            out.println("  archaeology: desert_pyramid, center (0, 0), radius 2,000, limit 100");
            out.println("  find: ancient_city, center (0, 0), radius 5,000, limit 20");
            out.println("  loot: minecraft:chests/ancient_city");
            out.println();
            out.println("Supported structures:");
            for (int index = 0; index < version.structures().size(); index++) {
                StructureSpec spec = version.structures().get(index);
                out.printf("%n[%d] %s%n", index + 1, spec.name());
                out.printf("  Dimension: %s%n", spec.dimensionId());
                out.printf("  Default item: %s%n", spec.defaultTargetItem());
                out.printf("  Loot tables: %s%n", number(spec.lootTables().size()));
            }
            out.println();
            out.println("Use 'explain --structure NAME' for details.");
            return 0;
        }

        StructureSpec spec = version.structure(structureName);
        if (arguments.flag("json")) {
            out.printf("{\"version\":\"%s\",\"name\":\"%s\","
                            + "\"structure_id\":\"%s\",\"dimension\":\"%s\","
                            + "\"default_item\":\"%s\",\"placement\":",
                    version.minecraftVersion(), spec.name(), spec.structureId(),
                    spec.dimensionId(), spec.defaultTargetItem());
            printPlacementJson(out, spec);
            out.printf(",\"decoration_step\":%d,\"decoration_index\":%d,"
                            + "\"scanner\":\"%s\",\"container_seed_shortcut\":\"%s\","
                            + "\"loot_tables\":[",
                    spec.decorationStep(), spec.indexWithinStep(), spec.scannerKind(),
                    spec.containerSeedShortcut());
            for (int index = 0; index < spec.lootTables().size(); index++) {
                if (index != 0) {
                    out.print(',');
                }
                out.printf("\"%s\"", spec.lootTables().get(index));
            }
            out.println("]}");
            return 0;
        }
        out.printf("Minecraft Java %s%n", version.minecraftVersion());
        out.printf("Structure: %s%n", spec.name());
        out.printf("Structure ID: %s%n", spec.structureId());
        out.printf("Dimension: %s%n", spec.dimensionId());
        out.printf("Default item: %s%n", spec.defaultTargetItem());
        out.println();
        out.println("Placement:");
        printPlacementText(out, spec);
        out.println();
        out.println("Container calculation:");
        out.printf("  Decoration step: %d%n", spec.decorationStep());
        out.printf("  Structure index: %d%n", spec.indexWithinStep());
        out.printf("  Scanner: %s%n", spec.scannerKind());
        out.printf("  Seed shortcut: %s%n", spec.containerSeedShortcut());
        out.println();
        out.println("Loot tables:");
        spec.lootTables().forEach(table -> out.println("  " + table));
        return 0;
    }

    private static StructureSpec structure(Arguments arguments, VersionProfile version) {
        return version.structure(arguments.text("structure", "ancient_city"));
    }

    private static List<StructureCandidate> locateCandidates(
            long worldSeed,
            int centerX,
            int centerZ,
            int radius,
            StructureSpec spec
    ) {
        if (spec.placement() instanceof VersionProfile.StructureProfile) {
            return RandomSpreadLocator.locate(worldSeed, centerX, centerZ, radius, spec);
        }
        try (SearchEngine engine = VanillaSearchEngine.load(worldSeed)) {
            engine.verifyProfile(spec);
            return engine.locateCandidates(spec, centerX, centerZ, radius);
        }
    }

    private static void printPlacementJson(PrintStream out, StructureSpec spec) {
        if (spec.placement() instanceof VersionProfile.StructureProfile placement) {
            out.printf("{\"type\":\"random_spread\",\"spacing\":%d,"
                            + "\"separation\":%d,\"salt\":%d,\"spread\":\"%s\"}",
                    placement.spacing(), placement.separation(), placement.salt(),
                    placement.spreadType());
            return;
        }
        if (spec.placement() instanceof VersionProfile.FeatureProfile placement) {
            out.printf("{\"type\":\"placed_feature\",\"rarity_chance\":%d}",
                    placement.rarityChance());
            return;
        }
        VersionProfile.ConcentricRingsProfile placement =
                (VersionProfile.ConcentricRingsProfile) spec.placement();
        out.printf("{\"type\":\"concentric_rings\",\"distance\":%d,"
                        + "\"spread\":%d,\"count\":%d,\"salt\":%d}",
                placement.distance(), placement.spread(), placement.count(), placement.salt());
    }

    private static void printPlacementText(PrintStream out, StructureSpec spec) {
        if (spec.placement() instanceof VersionProfile.StructureProfile placement) {
            out.println("  Type: RANDOM_SPREAD");
            out.printf("  Spacing: %d%n", placement.spacing());
            out.printf("  Separation: %d%n", placement.separation());
            out.printf("  Salt: %d%n", placement.salt());
            out.printf("  Spread: %s%n", placement.spreadType());
            return;
        }
        if (spec.placement() instanceof VersionProfile.FeatureProfile placement) {
            out.println("  Type: PLACED_FEATURE");
            out.printf("  Attempts: about once every %d chunks%n", placement.rarityChance());
            return;
        }
        VersionProfile.ConcentricRingsProfile placement =
                (VersionProfile.ConcentricRingsProfile) spec.placement();
        out.println("  Type: CONCENTRIC_RINGS");
        out.printf("  Distance: %d chunks%n", placement.distance());
        out.printf("  Spread: %d%n", placement.spread());
        out.printf("  Count: %d%n", placement.count());
        out.printf("  Salt: %d%n", placement.salt());
    }

    private static void printSearchHeader(
            PrintStream out,
            VersionProfile version,
            StructureSpec spec,
            long worldSeed,
            int centerX,
            int centerZ,
            int radius
    ) {
        out.printf("Minecraft Java %s%n", version.minecraftVersion());
        out.printf("World seed: %d%n", worldSeed);
        out.printf("Structure: %s%n", spec.name());
        out.printf("Search area: %s blocks around (%d, %d)%n%n",
                number(radius), centerX, centerZ);
    }

    private static void printContainers(
            PrintStream out,
            List<ChestPrediction> containers,
            int shown,
            boolean includeOrdinal
    ) {
        for (int index = 0; index < shown; index++) {
            ChestPrediction chest = containers.get(index);
            out.printf("[%d]%n", index + 1);
            out.printf("  Position: (%d, %d, %d)%n", chest.x(), chest.y(), chest.z());
            out.printf("  Start chunk: (%d, %d)%n",
                    chest.structureChunkX(), chest.structureChunkZ());
            out.printf("  Loot table: %s%n", chest.lootTable());
            out.printf("  Loot seed: %d%n", chest.lootTableSeed());
            if (chest.sourceKind() == ChestPrediction.LootSourceKind.ARCHAEOLOGY) {
                out.printf("  Source: archaeology%n");
                out.printf("  Block: %s%n", chest.sourceBlock());
            }
            if (includeOrdinal) {
                out.printf("  Ordinal: %d%n", chest.containerOrdinalInDecorationChunk());
            }
            out.println();
        }
    }

    private static String number(long value) {
        return String.format(Locale.ROOT, "%,d", value);
    }

    private static String quantity(long count, String singular) {
        return number(count) + " " + singular + (count == 1 ? "" : "s");
    }

    private static String decimal(double value) {
        return String.format(Locale.ROOT, "%,.1f", value);
    }

    private static void printHelp(PrintStream out) {
        out.println("mc-loot-finder");
        out.println("Minecraft Java 26.1.2 structure container and loot finder");
        out.println();
        out.println("Commands:");
        out.println();
        out.println("  candidates --seed N [search options]");
        out.println("    List possible structure chunks without full verification.");
        out.println();
        out.println("  chests --seed N [search options]");
        out.println("    Verify structures and list their block containers.");
        out.println();
        out.println("  archaeology --seed N [search options]");
        out.println("    List suspicious sand and gravel generated by supported structures.");
        out.println();
        out.println("  find --seed N [--item ID] [search options]");
        out.println("    Find containers that generate the requested item.");
        out.println();
        out.println("  loot --loot-seed N [--table ID]");
        out.println("    Replay one supported loot table.");
        out.println();
        out.println("  container-seed --seed N --chunk-x X --chunk-z Z [options]");
        out.println("    Calculate a seed for supported shortcut structures.");
        out.println();
        out.println("  explain [--structure NAME]");
        out.println("    Show defaults, supported structures, and loot tables.");
        out.println();
        out.println("Search options:");
        out.println("  --structure NAME  --center-x X  --center-z Z");
        out.println("  --radius N  --limit N");
        out.println();
        out.println("Common options:");
        out.println("  --version 26.1.2  --json");
        out.println();
        out.println("Use 'explain' to list supported structures and defaults.");
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

    private static List<ChestPrediction> visibleContainers(List<ChestPrediction> chests) {
        return chests.stream()
                .filter(chest -> chest.sourceKind() == ChestPrediction.LootSourceKind.CONTAINER)
                .filter(chest -> !chest.lootTable().isEmpty())
                .toList();
    }

    private static List<ChestPrediction> visibleArchaeology(List<ChestPrediction> sources) {
        return sources.stream()
                .filter(source -> source.sourceKind()
                        == ChestPrediction.LootSourceKind.ARCHAEOLOGY)
                .filter(source -> !source.lootTable().isEmpty())
                .toList();
    }

    private static List<ChestPrediction> visibleLootSources(List<ChestPrediction> sources) {
        return sources.stream()
                .filter(source -> !source.lootTable().isEmpty())
                .toList();
    }

    static void requireUnambiguousContainerStreams(
            String structureName,
            List<ChestPrediction> chests
    ) {
        Map<Long, Long> ownerByDecorationChunk = new HashMap<>();
        for (ChestPrediction chest : chests) {
            long decorationChunk = packChunk(
                    Math.floorDiv(chest.x(), 16), Math.floorDiv(chest.z(), 16)
            );
            long startChunk = packChunk(chest.structureChunkX(), chest.structureChunkZ());
            Long previous = ownerByDecorationChunk.putIfAbsent(decorationChunk, startChunk);
            if (previous != null && previous.longValue() != startChunk) {
                throw new IllegalArgumentException(
                        "Two " + structureName + " starts consume containers in decoration chunk ("
                                + (int) decorationChunk + ","
                                + (int) (decorationChunk >> 32)
                                + "); cross-start stream merging is not implemented"
                );
            }
        }
    }

    private static long packChunk(int x, int z) {
        return (x & 0xffffffffL) | ((z & 0xffffffffL) << 32);
    }
}
