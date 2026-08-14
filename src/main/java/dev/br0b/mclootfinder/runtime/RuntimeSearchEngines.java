package dev.br0b.mclootfinder.runtime;

import dev.br0b.mclootfinder.core.StructureSpec;
import dev.br0b.mclootfinder.engine.SearchEngine;
import dev.br0b.mclootfinder.engine.SearchEngines;
import dev.br0b.mclootfinder.engine.StructureScan;
import dev.br0b.mclootfinder.vanilla.LootOracle;

import java.io.IOException;
import java.io.UncheckedIOException;
import java.net.URL;
import java.net.URLClassLoader;
import java.nio.file.Path;
import java.util.ArrayList;
import java.util.Collections;
import java.util.LinkedHashSet;
import java.util.List;
import java.util.Set;

/** Opens the vanilla provider against the locally generated official runtime. */
public final class RuntimeSearchEngines {
    private static final List<String> PARENT_FIRST_CLASSES = List.of(
            "dev.br0b.mclootfinder.core.StructureSpec",
            "dev.br0b.mclootfinder.core.VersionProfile",
            "dev.br0b.mclootfinder.engine.",
            "dev.br0b.mclootfinder.vanilla.ChestPrediction",
            "dev.br0b.mclootfinder.vanilla.LootOracle",
            "dev.br0b.mclootfinder.vanilla.LootStack"
    );

    private RuntimeSearchEngines() {
    }

    public static SearchEngine open(String minecraftVersion, long worldSeed) {
        RuntimeManager manager = RuntimeManager.createDefault(minecraftVersion);
        if (!manager.status().generatedRuntimeReady()) {
            throw new IllegalStateException(
                    "The local Minecraft Java " + minecraftVersion
                            + " runtime is not installed; run 'mc-loot-finder runtime install'"
            );
        }

        List<URL> urls = new ArrayList<>();
        urls.add(applicationLocation());
        manager.runtimeClasspath().stream()
                .map(RuntimeSearchEngines::toUrl)
                .forEach(urls::add);

        ChildFirstClassLoader loader = new ChildFirstClassLoader(
                urls.toArray(URL[]::new), RuntimeSearchEngines.class.getClassLoader()
        );
        try {
            SearchEngine delegate = withContext(
                    loader,
                    () -> SearchEngines.open(minecraftVersion, worldSeed, loader)
            );
            return new IsolatedSearchEngine(delegate, loader);
        } catch (RuntimeException | Error exception) {
            closeLoader(loader, exception);
            throw exception;
        }
    }

    private static URL applicationLocation() {
        var source = RuntimeSearchEngines.class.getProtectionDomain().getCodeSource();
        if (source == null) {
            throw new IllegalStateException("Could not locate the mc-loot-finder application jar");
        }
        return source.getLocation();
    }

    private static URL toUrl(Path path) {
        try {
            return path.toUri().toURL();
        } catch (IOException exception) {
            throw new UncheckedIOException("Could not add runtime path " + path, exception);
        }
    }

    private static <T> T withContext(
            ClassLoader loader,
            ContextOperation<T> operation
    ) {
        Thread thread = Thread.currentThread();
        ClassLoader previous = thread.getContextClassLoader();
        thread.setContextClassLoader(loader);
        try {
            return operation.run();
        } finally {
            thread.setContextClassLoader(previous);
        }
    }

    private static void closeLoader(URLClassLoader loader, Throwable pending) {
        try {
            loader.close();
        } catch (IOException closeException) {
            pending.addSuppressed(closeException);
        }
    }

    @FunctionalInterface
    private interface ContextOperation<T> {
        T run();
    }

    private static final class IsolatedSearchEngine implements SearchEngine {
        private final SearchEngine delegate;
        private final URLClassLoader loader;
        private final LootOracle lootOracle;
        private boolean closed;

        private IsolatedSearchEngine(SearchEngine delegate, URLClassLoader loader) {
            this.delegate = delegate;
            this.loader = loader;
            LootOracle delegateOracle = withContext(loader, delegate::lootOracle);
            this.lootOracle = (lootTable, lootTableSeed) -> withContext(
                    loader, () -> delegateOracle.roll(lootTable, lootTableSeed)
            );
        }

        @Override
        public void verifyProfile(StructureSpec spec) {
            withContext(loader, () -> {
                delegate.verifyProfile(spec);
                return null;
            });
        }

        @Override
        public StructureScan scan(StructureSpec spec, int chunkX, int chunkZ) {
            return withContext(loader, () -> delegate.scan(spec, chunkX, chunkZ));
        }

        @Override
        public LootOracle lootOracle() {
            return lootOracle;
        }

        @Override
        public void close() {
            if (closed) {
                return;
            }
            closed = true;
            RuntimeException failure = null;
            try {
                withContext(loader, () -> {
                    delegate.close();
                    return null;
                });
            } catch (RuntimeException exception) {
                failure = exception;
            }
            try {
                loader.close();
            } catch (IOException exception) {
                if (failure != null) {
                    failure.addSuppressed(exception);
                } else {
                    failure = new UncheckedIOException(
                            "Could not close the generated runtime", exception
                    );
                }
            }
            if (failure != null) {
                throw failure;
            }
        }
    }

    private static final class ChildFirstClassLoader extends URLClassLoader {
        private ChildFirstClassLoader(URL[] urls, ClassLoader parent) {
            super(urls, parent);
        }

        @Override
        protected Class<?> loadClass(String name, boolean resolve)
                throws ClassNotFoundException {
            synchronized (getClassLoadingLock(name)) {
                Class<?> loaded = findLoadedClass(name);
                if (loaded == null) {
                    if (parentFirst(name)) {
                        loaded = super.loadClass(name, false);
                    } else {
                        try {
                            loaded = findClass(name);
                        } catch (ClassNotFoundException ignored) {
                            loaded = super.loadClass(name, false);
                        }
                    }
                }
                if (resolve) {
                    resolveClass(loaded);
                }
                return loaded;
            }
        }

        @Override
        public URL getResource(String name) {
            URL resource = findResource(name);
            return resource != null ? resource : getParent().getResource(name);
        }

        @Override
        public java.util.Enumeration<URL> getResources(String name) throws IOException {
            Set<URL> resources = new LinkedHashSet<>();
            resources.addAll(Collections.list(findResources(name)));
            resources.addAll(Collections.list(getParent().getResources(name)));
            return Collections.enumeration(resources);
        }

        private static boolean parentFirst(String name) {
            if (name.startsWith("java.") || name.startsWith("javax.")
                    || name.startsWith("jdk.") || name.startsWith("sun.")) {
                return true;
            }
            return PARENT_FIRST_CLASSES.stream().anyMatch(name::startsWith);
        }
    }
}
