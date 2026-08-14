package dev.br0b.mclootfinder.engine;

import java.util.List;
import java.util.ServiceLoader;

/** Resolves search runtimes without coupling the CLI to their implementation. */
public final class SearchEngines {
    private SearchEngines() {
    }

    public static SearchEngine open(String minecraftVersion, long worldSeed) {
        return open(minecraftVersion, worldSeed, SearchEngines.class.getClassLoader());
    }

    public static SearchEngine open(
            String minecraftVersion,
            long worldSeed,
            ClassLoader runtimeLoader
    ) {
        List<SearchEngineProvider> matches = ServiceLoader
                .load(SearchEngineProvider.class, runtimeLoader)
                .stream()
                .map(ServiceLoader.Provider::get)
                .filter(provider -> provider.minecraftVersion().equals(minecraftVersion))
                .toList();
        if (matches.isEmpty()) {
            throw new IllegalStateException(
                    "No search runtime is installed for Minecraft Java " + minecraftVersion
            );
        }
        if (matches.size() != 1) {
            throw new IllegalStateException(
                    "Multiple search runtimes are installed for Minecraft Java "
                            + minecraftVersion
            );
        }
        return matches.getFirst().open(worldSeed);
    }

    public static List<String> availableVersions() {
        return ServiceLoader.load(SearchEngineProvider.class).stream()
                .map(ServiceLoader.Provider::get)
                .map(SearchEngineProvider::minecraftVersion)
                .distinct()
                .sorted()
                .toList();
    }
}
