package dev.br0b.mclootfinder.vanilla;

import dev.br0b.mclootfinder.core.StructureSpec;
import dev.br0b.mclootfinder.core.Versions;
import net.minecraft.SharedConstants;
import net.minecraft.commands.Commands;
import net.minecraft.core.Holder;
import net.minecraft.core.LayeredRegistryAccess;
import net.minecraft.core.Registry;
import net.minecraft.core.RegistryAccess;
import net.minecraft.core.registries.Registries;
import net.minecraft.resources.Identifier;
import net.minecraft.resources.ResourceKey;
import net.minecraft.server.Bootstrap;
import net.minecraft.server.RegistryLayer;
import net.minecraft.server.ReloadableServerResources;
import net.minecraft.server.WorldLoader;
import net.minecraft.server.packs.repository.PackRepository;
import net.minecraft.server.packs.repository.ServerPacksSource;
import net.minecraft.server.packs.resources.CloseableResourceManager;
import net.minecraft.server.permissions.PermissionSet;
import net.minecraft.util.Util;
import net.minecraft.util.datafix.DataFixers;
import net.minecraft.world.level.ChunkPos;
import net.minecraft.world.level.Level;
import net.minecraft.world.level.LevelHeightAccessor;
import net.minecraft.world.level.WorldDataConfiguration;
import net.minecraft.world.level.biome.Biome;
import net.minecraft.world.level.chunk.ChunkGenerator;
import net.minecraft.world.level.chunk.ChunkGeneratorStructureState;
import net.minecraft.world.level.dimension.DimensionType;
import net.minecraft.world.level.dimension.LevelStem;
import net.minecraft.world.level.levelgen.LegacyRandomSource;
import net.minecraft.world.level.levelgen.Heightmap;
import net.minecraft.world.level.levelgen.NoiseBasedChunkGenerator;
import net.minecraft.world.level.levelgen.RandomState;
import net.minecraft.world.level.levelgen.WorldDimensions;
import net.minecraft.world.level.levelgen.WorldgenRandom;
import net.minecraft.world.level.levelgen.presets.WorldPresets;
import net.minecraft.world.level.levelgen.structure.Structure;
import net.minecraft.world.level.levelgen.structure.StructureStart;
import net.minecraft.world.level.levelgen.structure.StructureSet;
import net.minecraft.world.level.levelgen.structure.templatesystem.StructureTemplateManager;
import net.minecraft.world.level.storage.LevelStorageSource;

import java.io.IOException;
import java.io.UncheckedIOException;
import java.nio.file.Files;
import java.nio.file.Path;
import java.util.ArrayList;
import java.util.Comparator;
import java.util.List;
import java.util.Map;
import java.util.concurrent.CompletionException;

/** Loads vanilla 26.1.2 registries and dimension worldgen without loading chunks. */
public final class VanillaRuntime26_1_2 implements AutoCloseable {
    private final long worldSeed;
    private final CloseableResourceManager resourceManager;
    private final ReloadableServerResources reloadableResources;
    private final RegistryAccess.Frozen registries;
    private final WorldDimensions.Complete dimensions;
    private final LevelStorageSource.LevelStorageAccess storageAccess;
    private final Path scratchPath;
    private final StructureTemplateManager templateManager;
    private final Map<String, DimensionContext> dimensionContexts;

    private VanillaRuntime26_1_2(
            long worldSeed,
            CloseableResourceManager resourceManager,
            ReloadableServerResources reloadableResources,
            LayeredRegistryAccess<RegistryLayer> layeredRegistries,
            WorldDimensions.Complete dimensions
    ) {
        this.worldSeed = worldSeed;
        this.resourceManager = resourceManager;
        this.reloadableResources = reloadableResources;
        this.registries = layeredRegistries.compositeAccess();
        this.dimensions = dimensions;
        this.dimensionContexts = Map.of(
                "minecraft:overworld", createDimensionContext(Level.OVERWORLD, LevelStem.OVERWORLD),
                "minecraft:the_nether", createDimensionContext(Level.NETHER, LevelStem.NETHER),
                "minecraft:the_end", createDimensionContext(Level.END, LevelStem.END)
        );

        Path createdScratch = null;
        try {
            createdScratch = Files.createTempDirectory("mc-loot-finder-templates-");
            LevelStorageSource storageSource = LevelStorageSource.createDefault(createdScratch);
            this.storageAccess = storageSource.createAccess("scratch");
            this.scratchPath = createdScratch;
        } catch (IOException exception) {
            if (createdScratch != null) {
                try {
                    deleteRecursively(createdScratch);
                } catch (IOException cleanupException) {
                    exception.addSuppressed(cleanupException);
                }
            }
            throw new UncheckedIOException("Could not create the temporary template path", exception);
        }
        this.templateManager = new StructureTemplateManager(
                resourceManager,
                storageAccess,
                DataFixers.getDataFixer(),
                registries.lookupOrThrow(Registries.BLOCK)
        );
    }

    private DimensionContext createDimensionContext(
            ResourceKey<Level> levelKey,
            ResourceKey<LevelStem> stemKey
    ) {
        LevelStem stem = dimensions.dimensions().getValueOrThrow(stemKey);
        ChunkGenerator generator = stem.generator();
        if (!(generator instanceof NoiseBasedChunkGenerator noiseGenerator)) {
            throw new IllegalStateException(levelKey.identifier() + " is not noise-based");
        }
        ResourceKey<?> settingsKey = noiseGenerator.generatorSettings().unwrapKey().orElseThrow();
        @SuppressWarnings("unchecked")
        ResourceKey<net.minecraft.world.level.levelgen.NoiseGeneratorSettings> typedSettingsKey =
                (ResourceKey<net.minecraft.world.level.levelgen.NoiseGeneratorSettings>) settingsKey;
        RandomState state = RandomState.create(registries, typedSettingsKey, worldSeed);
        DimensionType type = stem.type().value();
        ChunkGeneratorStructureState structureState =
                ChunkGeneratorStructureState.createForNormal(
                        state,
                        worldSeed,
                        generator.getBiomeSource(),
                        registries.lookupOrThrow(Registries.STRUCTURE_SET)
                );
        return new DimensionContext(
                levelKey,
                generator,
                state,
                LevelHeightAccessor.create(type.minY(), type.height()),
                type,
                structureState
        );
    }

    public static VanillaRuntime26_1_2 load(long worldSeed) {
        SharedConstants.tryDetectVersion();
        Bootstrap.bootStrap();

        PackRepository packs = ServerPacksSource.createVanillaTrustedRepository();
        WorldLoader.PackConfig packConfig = new WorldLoader.PackConfig(
                packs, WorldDataConfiguration.DEFAULT, true, true
        );
        WorldLoader.InitConfig initConfig = new WorldLoader.InitConfig(
                packConfig, Commands.CommandSelection.DEDICATED, PermissionSet.NO_PERMISSIONS
        );

        try {
            return WorldLoader.load(
                    initConfig,
                    context -> {
                        Registry<LevelStem> datapackDimensions = context.datapackDimensions()
                                .lookupOrThrow(Registries.LEVEL_STEM);
                        WorldDimensions normal = WorldPresets.createNormalWorldDimensions(
                                context.datapackWorldgen()
                        );
                        WorldDimensions.Complete complete = normal.bake(datapackDimensions);
                        return new WorldLoader.DataLoadOutput<>(
                                complete, complete.dimensionsRegistryAccess()
                        );
                    },
                    (resources, reloadable, registries, complete) -> new VanillaRuntime26_1_2(
                            worldSeed, resources, reloadable, registries, complete
                    ),
                    Util.backgroundExecutor(),
                    Runnable::run
            ).join();
        } catch (CompletionException exception) {
            throw new IllegalStateException(
                    "Could not load vanilla 26.1.2 worldgen data", exception.getCause()
            );
        }
    }

    public StructureStart generateAncientCity(ChunkPos startChunk) {
        return generateSelectedStructure(Versions.V26_1_2.ancientCity(), startChunk);
    }

    /** Reproduces weighted structure-set selection, including biome fallback. */
    public StructureStart generateSelectedStructure(StructureSpec spec, ChunkPos startChunk) {
        List<StructureSpec.SelectionEntry> remaining = new ArrayList<>(spec.structureSetEntries());
        if (remaining.size() == 1) {
            return generateStructure(spec, remaining.getFirst().structureId(), startChunk);
        }

        WorldgenRandom selectionRandom = new WorldgenRandom(new LegacyRandomSource(0L));
        selectionRandom.setLargeFeatureSeed(worldSeed, startChunk.x(), startChunk.z());
        int totalWeight = remaining.stream().mapToInt(StructureSpec.SelectionEntry::weight).sum();
        while (!remaining.isEmpty()) {
            int choice = selectionRandom.nextInt(totalWeight);
            int selectedIndex = 0;
            for (; selectedIndex < remaining.size(); selectedIndex++) {
                choice -= remaining.get(selectedIndex).weight();
                if (choice < 0) {
                    break;
                }
            }
            StructureSpec.SelectionEntry selected = remaining.get(selectedIndex);
            StructureStart start = generateStructure(spec, selected.structureId(), startChunk);
            if (start.isValid()) {
                return selected.accepted() ? start : StructureStart.INVALID_START;
            }
            remaining.remove(selectedIndex);
            totalWeight -= selected.weight();
        }
        return StructureStart.INVALID_START;
    }

    private StructureStart generateStructure(
            StructureSpec spec,
            String structureId,
            ChunkPos startChunk
    ) {
        DimensionContext dimension = dimension(spec);
        Registry<Structure> structureRegistry = registries.lookupOrThrow(Registries.STRUCTURE);
        ResourceKey<Structure> key = ResourceKey.create(
                Registries.STRUCTURE, Identifier.parse(structureId)
        );
        Holder.Reference<Structure> holder = structureRegistry.get(key).orElseThrow();
        Structure structure = holder.value();
        return structure.generate(
                holder,
                dimension.levelKey(),
                registries,
                dimension.chunkGenerator(),
                dimension.chunkGenerator().getBiomeSource(),
                dimension.randomState(),
                templateManager,
                worldSeed,
                startChunk,
                0,
                dimension.heightAccessor(),
                structure.biomes()::contains
        );
    }

    /** Validates the version-pinned decoration RNG stream coordinates. */
    public void verifyStructureProfile(StructureSpec spec) {
        DecorationCoordinates actual = structureDecorationCoordinates(spec.structureId());
        int step = actual.step();
        int index = actual.indexWithinStep();
        if (spec.decorationStep() >= 0 && spec.indexWithinStep() >= 0
                && (step != spec.decorationStep() || index != spec.indexWithinStep())) {
            throw new IllegalStateException(
                    "26.1.2 profile drift: " + spec.name() + " step/index expected "
                            + spec.decorationStep() + "/" + spec.indexWithinStep()
                            + " but vanilla loaded " + step + "/" + index
            );
        }
    }

    public DecorationCoordinates structureDecorationCoordinates(String structureId) {
        Registry<Structure> structureRegistry = registries.lookupOrThrow(Registries.STRUCTURE);
        ResourceKey<Structure> key = ResourceKey.create(
                Registries.STRUCTURE, Identifier.parse(structureId)
        );
        Structure target = structureRegistry.getValueOrThrow(key);
        int step = target.step().ordinal();
        List<Structure> structuresInStep = structureRegistry.stream()
                .filter(structure -> structure.step() == target.step())
                .toList();
        int index = structuresInStep.indexOf(target);
        return new DecorationCoordinates(step, index);
    }

    public String structureId(StructureStart start) {
        Registry<Structure> structureRegistry = registries.lookupOrThrow(Registries.STRUCTURE);
        Identifier id = structureRegistry.getKey(start.getStructure());
        if (id == null) {
            throw new IllegalStateException("Generated structure is absent from the registry");
        }
        return id.toString();
    }

    /** Applies vanilla frequency reduction and cross-structure exclusion rules. */
    public boolean isStructurePlacementChunk(StructureSpec spec, ChunkPos chunk) {
        Registry<StructureSet> sets = registries.lookupOrThrow(Registries.STRUCTURE_SET);
        java.util.Set<String> expected = spec.structureSetEntries().stream()
                .map(StructureSpec.SelectionEntry::structureId)
                .collect(java.util.stream.Collectors.toSet());
        StructureSet set = sets.stream()
                .filter(candidate -> candidate.structures().stream()
                        .map(entry -> entry.structure().unwrapKey().orElseThrow()
                                .identifier().toString())
                        .collect(java.util.stream.Collectors.toSet())
                        .equals(expected))
                .findFirst()
                .orElseThrow(() -> new IllegalStateException(
                        "Vanilla structure set not found for " + spec.name()
                ));
        return set.placement().isStructureChunk(
                dimension(spec).structureState(), chunk.x(), chunk.z()
        );
    }

    public record DecorationCoordinates(int step, int indexWithinStep) {
    }

    public void verifyAncientCityProfile(dev.br0b.mclootfinder.core.VersionProfile profile) {
        verifyStructureProfile(profile.ancientCity());
    }

    public StructureTemplateManager templateManager() {
        return templateManager;
    }

    public ChunkGenerator chunkGenerator() {
        return chunkGenerator(Versions.V26_1_2.ancientCity());
    }

    public LevelHeightAccessor heightAccessor() {
        return heightAccessor(Versions.V26_1_2.ancientCity());
    }

    public ChunkGenerator chunkGenerator(StructureSpec spec) {
        return dimension(spec).chunkGenerator();
    }

    public LevelHeightAccessor heightAccessor(StructureSpec spec) {
        return dimension(spec).heightAccessor();
    }

    public int motionBlockingHeight(StructureSpec spec, int blockX, int blockZ) {
        DimensionContext dimension = dimension(spec);
        return dimension.chunkGenerator().getBaseHeight(
                blockX,
                blockZ,
                Heightmap.Types.MOTION_BLOCKING_NO_LEAVES,
                dimension.heightAccessor(),
                dimension.randomState()
        );
    }

    public Holder<Biome> noiseBiome(StructureSpec spec, int quartX, int quartY, int quartZ) {
        DimensionContext dimension = dimension(spec);
        return dimension.chunkGenerator().getBiomeSource().getNoiseBiome(
                quartX, quartY, quartZ, dimension.randomState().sampler()
        );
    }

    public DimensionType dimensionType(StructureSpec spec) {
        return dimension(spec).dimensionType();
    }

    public ReloadableServerResources reloadableResources() {
        return reloadableResources;
    }

    public RegistryAccess.Frozen registries() {
        return registries;
    }

    Path scratchPath() {
        return scratchPath;
    }

    private DimensionContext dimension(StructureSpec spec) {
        DimensionContext context = dimensionContexts.get(spec.dimensionId());
        if (context == null) {
            throw new IllegalArgumentException("Unsupported dimension: " + spec.dimensionId());
        }
        return context;
    }

    @Override
    public void close() {
        RuntimeException failure = null;
        try {
            storageAccess.close();
        } catch (IOException exception) {
            failure = new UncheckedIOException(exception);
        }
        try {
            resourceManager.close();
        } catch (RuntimeException exception) {
            failure = appendFailure(failure, exception);
        }
        try {
            deleteRecursively(scratchPath);
        } catch (IOException exception) {
            failure = appendFailure(failure, new UncheckedIOException(exception));
        }
        if (failure != null) {
            throw failure;
        }
    }

    private static RuntimeException appendFailure(
            RuntimeException failure,
            RuntimeException next
    ) {
        if (failure == null) {
            return next;
        }
        failure.addSuppressed(next);
        return failure;
    }

    private static void deleteRecursively(Path root) throws IOException {
        if (!Files.exists(root)) {
            return;
        }
        List<Path> paths;
        try (var walk = Files.walk(root)) {
            paths = walk.sorted(Comparator.reverseOrder()).toList();
        }
        for (Path path : paths) {
            Files.deleteIfExists(path);
        }
    }

    private record DimensionContext(
            ResourceKey<Level> levelKey,
            ChunkGenerator chunkGenerator,
            RandomState randomState,
            LevelHeightAccessor heightAccessor,
            DimensionType dimensionType,
            ChunkGeneratorStructureState structureState
    ) {
    }
}
