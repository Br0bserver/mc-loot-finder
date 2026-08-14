package dev.br0b.mclootfinder.runtime;

import java.io.IOException;
import java.io.InputStream;
import java.io.InputStreamReader;
import java.io.OutputStream;
import java.io.PrintStream;
import java.io.UncheckedIOException;
import java.net.http.HttpClient;
import java.net.http.HttpRequest;
import java.net.http.HttpResponse;
import java.nio.channels.FileChannel;
import java.nio.channels.FileLock;
import java.nio.file.AtomicMoveNotSupportedException;
import java.nio.file.Files;
import java.nio.file.Path;
import java.nio.file.StandardCopyOption;
import java.nio.file.StandardOpenOption;
import java.nio.charset.StandardCharsets;
import java.security.MessageDigest;
import java.security.NoSuchAlgorithmException;
import java.time.Duration;
import java.util.ArrayList;
import java.util.Comparator;
import java.util.HexFormat;
import java.util.LinkedHashSet;
import java.util.List;
import java.util.Properties;
import java.util.Set;
import java.util.zip.ZipEntry;
import java.util.zip.ZipFile;
import java.util.zip.ZipOutputStream;

/** Downloads, verifies, and caches the official input for local runtime extraction. */
public final class RuntimeManager {
    private static final String STATE_FILE = "source.properties";

    private final RuntimeVersion version;
    private final Path cacheRoot;
    private final HttpClient httpClient;

    public RuntimeManager(RuntimeVersion version, Path cacheRoot) {
        this(
                version,
                cacheRoot,
                HttpClient.newBuilder()
                        .connectTimeout(Duration.ofSeconds(20))
                        .followRedirects(HttpClient.Redirect.NORMAL)
                        .build()
        );
    }

    RuntimeManager(RuntimeVersion version, Path cacheRoot, HttpClient httpClient) {
        this.version = version;
        this.cacheRoot = cacheRoot.toAbsolutePath().normalize();
        this.httpClient = httpClient;
    }

    public static RuntimeManager createDefault(String minecraftVersion) {
        return new RuntimeManager(RuntimeVersion.require(minecraftVersion), defaultCacheRoot());
    }

    public RuntimeStatus status() {
        Path versionRoot = versionRoot();
        return new RuntimeStatus(
                version.minecraftVersion(),
                version.recipeVersion(),
                versionRoot,
                sourceStateMatches(),
                runtimeStateMatches()
        );
    }

    public RuntimeStatus installSource(
            Path suppliedServerJar,
            boolean offline,
            PrintStream progress
    ) {
        Path versionRoot = versionRoot();
        Path sourceRoot = sourceRoot();
        try {
            Files.createDirectories(sourceRoot);
            try (FileChannel channel = FileChannel.open(
                    versionRoot.resolve("install.lock"),
                    StandardOpenOption.CREATE,
                    StandardOpenOption.WRITE
            ); FileLock ignored = channel.lock()) {
                if (sourceStateMatches() && runtimeStateMatches()) {
                    Files.deleteIfExists(sourceRoot.resolve("server-inner.jar"));
                    progress.println("Minecraft Java " + version.minecraftVersion()
                            + " source is already installed.");
                    return status();
                }

                Path outerJar = sourceRoot.resolve("server.jar");
                if (!verifiedFile(
                        outerJar, version.serverSize(), "SHA-1", version.serverSha1()
                )) {
                    acquireOuterJar(outerJar, suppliedServerJar, offline, progress);
                }

                Path innerJar = sourceRoot.resolve("server-inner.jar");
                extractBundledServer(outerJar, innerJar);
                try {
                    generateRuntime(outerJar, innerJar);
                } finally {
                    Files.deleteIfExists(innerJar);
                }
                writeState();
                progress.println("Prepared verified Minecraft Java "
                        + version.minecraftVersion() + " source in " + versionRoot + ".");
                return status();
            }
        } catch (IOException exception) {
            throw new UncheckedIOException("Could not install the local runtime source", exception);
        } catch (InterruptedException exception) {
            Thread.currentThread().interrupt();
            throw new IllegalStateException("Runtime source download was interrupted", exception);
        }
    }

    public boolean verifySource() {
        return verifiedFile(
                sourceRoot().resolve("server.jar"),
                version.serverSize(),
                "SHA-1",
                version.serverSha1()
        );
    }

    public boolean verifyRuntime() {
        try {
            Path runtimeRoot = versionRoot().resolve("runtime");
            Properties state = loadProperties(runtimeRoot.resolve("runtime.properties"));
            if (!runtimeStateMetadataMatches(state)) {
                return false;
            }
            if (!verifiedFile(
                    runtimeRoot.resolve("server.jar"),
                    Long.parseLong(state.getProperty("serverSize", "0")),
                    "SHA-256",
                    state.getProperty("serverSha256", "")
            )) {
                return false;
            }
            int libraryCount = Integer.parseInt(state.getProperty("library.count", "0"));
            for (int index = 0; index < libraryCount; index++) {
                Path relative = safeRelativePath(
                        state.getProperty("library." + index + ".path")
                );
                long size = Long.parseLong(state.getProperty("library." + index + ".size"));
                String sha256 = state.getProperty("library." + index + ".sha256", "");
                if (!verifiedFile(runtimeRoot.resolve("libraries").resolve(relative),
                        size, "SHA-256", sha256)) {
                    return false;
                }
            }
            return true;
        } catch (IllegalArgumentException exception) {
            return false;
        }
    }

    public List<Path> runtimeClasspath() {
        Path runtimeRoot = versionRoot().resolve("runtime");
        Properties state = loadProperties(runtimeRoot.resolve("runtime.properties"));
        if (!runtimeStateMetadataMatches(state)) {
            throw new IllegalStateException(
                    "Generated runtime is not installed for Minecraft Java "
                            + version.minecraftVersion()
            );
        }
        List<Path> result = new ArrayList<>();
        result.add(runtimeRoot.resolve("server.jar"));
        int libraryCount = Integer.parseInt(state.getProperty("library.count", "0"));
        for (int index = 0; index < libraryCount; index++) {
            Path relative = safeRelativePath(state.getProperty("library." + index + ".path"));
            result.add(runtimeRoot.resolve("libraries").resolve(relative));
        }
        return List.copyOf(result);
    }

    private void acquireOuterJar(
            Path destination,
            Path suppliedServerJar,
            boolean offline,
            PrintStream progress
    ) throws IOException, InterruptedException {
        Path temporary = Files.createTempFile(sourceRoot(), "server-", ".jar.part");
        try {
            if (suppliedServerJar != null) {
                Path source = suppliedServerJar.toAbsolutePath().normalize();
                progress.println("Verifying supplied Minecraft server jar...");
                Files.copy(source, temporary, StandardCopyOption.REPLACE_EXISTING);
            } else {
                if (offline) {
                    throw new IllegalStateException(
                            "Runtime source is not installed; provide --minecraft-jar while offline"
                    );
                }
                progress.println("Downloading official Minecraft Java "
                        + version.minecraftVersion() + " server jar...");
                HttpRequest request = HttpRequest.newBuilder(version.serverUri())
                        .timeout(Duration.ofMinutes(5))
                        .GET()
                        .build();
                HttpResponse<InputStream> response = httpClient.send(
                        request, HttpResponse.BodyHandlers.ofInputStream()
                );
                if (response.statusCode() != 200) {
                    throw new IOException(
                            "Minecraft download returned HTTP " + response.statusCode()
                    );
                }
                try (InputStream input = response.body();
                     OutputStream output = Files.newOutputStream(temporary)) {
                    input.transferTo(output);
                }
            }
            requireVerified(
                    temporary, version.serverSize(), "SHA-1", version.serverSha1()
            );
            atomicReplace(temporary, destination);
        } finally {
            Files.deleteIfExists(temporary);
        }
    }

    private void extractBundledServer(Path outerJar, Path destination) throws IOException {
        Path temporary = Files.createTempFile(sourceRoot(), "server-inner-", ".jar.part");
        try (ZipFile zip = new ZipFile(outerJar.toFile())) {
            ZipEntry entry = zip.getEntry(version.bundledServerPath());
            if (entry == null) {
                throw new IOException(
                        "Official server jar is missing " + version.bundledServerPath()
                );
            }
            try (InputStream input = zip.getInputStream(entry);
                 OutputStream output = Files.newOutputStream(temporary)) {
                input.transferTo(output);
            }
            requireVerified(
                    temporary,
                    version.bundledServerSize(),
                    "SHA-256",
                    version.bundledServerSha256()
            );
            atomicReplace(temporary, destination);
        } finally {
            Files.deleteIfExists(temporary);
        }
    }

    private void generateRuntime(Path outerJar, Path innerJar) throws IOException {
        Path versionRoot = versionRoot();
        Path destination = versionRoot.resolve("runtime");
        Path temporary = Files.createTempDirectory(versionRoot, "runtime-");
        boolean installed = false;
        try {
            Path runtimeServer = temporary.resolve("server.jar");
            createServerRuntime(
                    innerJar, runtimeServer, version.serverClassListResource()
            );
            Properties state = new Properties();
            state.setProperty("minecraftVersion", version.minecraftVersion());
            state.setProperty("recipeVersion", Integer.toString(version.recipeVersion()));
            state.setProperty("sourceServerSha256", version.bundledServerSha256());
            state.setProperty("serverSha256", digest(runtimeServer, "SHA-256"));
            state.setProperty("serverSize", Long.toString(Files.size(runtimeServer)));

            List<RuntimeLibrary> libraries = new ArrayList<>();
            Set<String> remainingLibraries = new LinkedHashSet<>(
                    version.runtimeLibraryPaths()
            );
            try (ZipFile zip = new ZipFile(outerJar.toFile())) {
                ZipEntry listEntry = zip.getEntry("META-INF/libraries.list");
                if (listEntry == null) {
                    throw new IOException("Official server jar is missing META-INF/libraries.list");
                }
                try (var reader = new java.io.BufferedReader(new InputStreamReader(
                        zip.getInputStream(listEntry), StandardCharsets.UTF_8
                ))) {
                    String line;
                    while ((line = reader.readLine()) != null) {
                        if (line.isBlank()) {
                            continue;
                        }
                        String[] fields = line.split("\\t", 3);
                        if (fields.length != 3) {
                            throw new IOException("Invalid official library entry: " + line);
                        }
                        if (!remainingLibraries.remove(fields[2])) {
                            continue;
                        }
                        Path relative = safeRelativePath(fields[2]);
                        String entryName = "META-INF/libraries/" + fields[2];
                        ZipEntry libraryEntry = zip.getEntry(entryName);
                        if (libraryEntry == null || libraryEntry.getSize() < 0) {
                            throw new IOException("Official server jar is missing " + entryName);
                        }
                        Path target = temporary.resolve("libraries").resolve(relative);
                        Files.createDirectories(target.getParent());
                        try (InputStream input = zip.getInputStream(libraryEntry);
                             OutputStream output = Files.newOutputStream(target)) {
                            input.transferTo(output);
                        }
                        requireVerified(
                                target, libraryEntry.getSize(), "SHA-256", fields[0]
                        );
                        String compactClassList = version.compactLibraryClassLists()
                                .get(fields[2]);
                        if (compactClassList != null) {
                            compactLibrary(target, compactClassList);
                        }
                        libraries.add(new RuntimeLibrary(
                                relative,
                                Files.size(target),
                                digest(target, "SHA-256")
                        ));
                    }
                }
            }
            if (!remainingLibraries.isEmpty()) {
                throw new IOException(
                        "Official server jar is missing required runtime libraries: "
                                + String.join(", ", remainingLibraries)
                );
            }

            state.setProperty("library.count", Integer.toString(libraries.size()));
            for (int index = 0; index < libraries.size(); index++) {
                RuntimeLibrary library = libraries.get(index);
                state.setProperty("library." + index + ".path",
                        library.path().toString().replace('\\', '/'));
                state.setProperty("library." + index + ".size",
                        Long.toString(library.size()));
                state.setProperty("library." + index + ".sha256", library.sha256());
            }
            try (OutputStream output = Files.newOutputStream(
                    temporary.resolve("runtime.properties")
            )) {
                state.store(output, "mc-loot-finder generated runtime");
            }

            deleteRecursively(destination);
            atomicReplace(temporary, destination);
            installed = true;
        } finally {
            if (!installed) {
                deleteRecursively(temporary);
            }
        }
    }

    private static void createServerRuntime(
            Path source,
            Path destination,
            String classListResource
    ) throws IOException {
        Set<String> retainedClasses = classListResource.isBlank()
                ? Set.of()
                : readRuntimeClassList(classListResource);
        Set<String> missing = new LinkedHashSet<>(retainedClasses);
        try (ZipFile input = new ZipFile(source.toFile());
             ZipOutputStream output = new ZipOutputStream(Files.newOutputStream(destination))) {
            output.setLevel(9);
            var entries = input.entries();
            while (entries.hasMoreElements()) {
                ZipEntry entry = entries.nextElement();
                String name = entry.getName();
                if (entry.isDirectory() || isSignatureMetadata(name)) {
                    continue;
                }
                if (!retainedClasses.isEmpty() && name.endsWith(".class")
                        && !retainedClasses.contains(name)) {
                    continue;
                }
                missing.remove(name);
                ZipEntry copied = new ZipEntry(name);
                copied.setTime(0L);
                output.putNextEntry(copied);
                try (InputStream content = input.getInputStream(entry)) {
                    content.transferTo(output);
                }
                output.closeEntry();
            }
        }
        if (!missing.isEmpty()) {
            throw new IOException(
                    "Runtime class list entries are absent from " + source + ": "
                            + String.join(", ", missing)
            );
        }
    }

    private static void compactLibrary(Path library, String classListResource)
            throws IOException {
        Set<String> retainedClasses = readRuntimeClassList(classListResource);
        Path temporary = Files.createTempFile(
                library.getParent(), library.getFileName().toString(), ".compact"
        );
        Set<String> missing = new LinkedHashSet<>(retainedClasses);
        try {
            try (ZipFile input = new ZipFile(library.toFile());
                 ZipOutputStream output = new ZipOutputStream(
                         Files.newOutputStream(temporary)
                 )) {
                output.setLevel(9);
                var entries = input.entries();
                while (entries.hasMoreElements()) {
                    ZipEntry entry = entries.nextElement();
                    String name = entry.getName();
                    if (entry.isDirectory() || isSignatureMetadata(name)) {
                        continue;
                    }
                    if (name.endsWith(".class") && !retainedClasses.contains(name)) {
                        continue;
                    }
                    missing.remove(name);
                    ZipEntry copied = new ZipEntry(name);
                    copied.setTime(0L);
                    output.putNextEntry(copied);
                    try (InputStream content = input.getInputStream(entry)) {
                        content.transferTo(output);
                    }
                    output.closeEntry();
                }
            }
            if (!missing.isEmpty()) {
                throw new IOException(
                        "Runtime class list entries are absent from " + library + ": "
                                + String.join(", ", missing)
                );
            }
            atomicReplace(temporary, library);
        } finally {
            Files.deleteIfExists(temporary);
        }
    }

    private static Set<String> readRuntimeClassList(String classListResource)
            throws IOException {
        Set<String> retainedClasses = new LinkedHashSet<>();
        try (InputStream input = RuntimeManager.class.getResourceAsStream(classListResource)) {
            if (input == null) {
                throw new IOException("Missing runtime class list " + classListResource);
            }
            try (var reader = new java.io.BufferedReader(new InputStreamReader(
                    input, StandardCharsets.UTF_8
            ))) {
                String line;
                while ((line = reader.readLine()) != null) {
                    String entry = line.strip();
                    if (!entry.isEmpty() && !entry.startsWith("#")) {
                        retainedClasses.add(entry);
                    }
                }
            }
        }
        if (retainedClasses.isEmpty()) {
            throw new IOException("Runtime class list is empty: " + classListResource);
        }
        return retainedClasses;
    }

    private static boolean isSignatureMetadata(String name) {
        String upper = name.toUpperCase(java.util.Locale.ROOT);
        return upper.equals("META-INF/MANIFEST.MF")
                || upper.startsWith("META-INF/")
                && (upper.endsWith(".SF")
                || upper.endsWith(".RSA")
                || upper.endsWith(".DSA"));
    }

    private boolean sourceStateMatches() {
        Path stateFile = sourceRoot().resolve(STATE_FILE);
        if (!Files.isRegularFile(stateFile)) {
            return false;
        }
        Properties state = new Properties();
        try (InputStream input = Files.newInputStream(stateFile)) {
            state.load(input);
            return state.getProperty("minecraftVersion", "")
                    .equals(version.minecraftVersion())
                    && state.getProperty("serverSha1", "").equals(version.serverSha1())
                    && state.getProperty("bundledServerSha256", "")
                    .equals(version.bundledServerSha256())
                    && Integer.parseInt(state.getProperty("recipeVersion", "0"))
                    == version.recipeVersion()
                    && Files.size(sourceRoot().resolve("server.jar")) == version.serverSize();
        } catch (IOException | NumberFormatException exception) {
            return false;
        }
    }

    private boolean runtimeStateMatches() {
        Path runtimeRoot = versionRoot().resolve("runtime");
        Properties state = loadProperties(runtimeRoot.resolve("runtime.properties"));
        if (!runtimeStateMetadataMatches(state)) {
            return false;
        }
        try {
            if (Files.size(runtimeRoot.resolve("server.jar"))
                    != Long.parseLong(state.getProperty("serverSize", "0"))) {
                return false;
            }
            int libraryCount = Integer.parseInt(state.getProperty("library.count", "0"));
            for (int index = 0; index < libraryCount; index++) {
                Path relative = safeRelativePath(
                        state.getProperty("library." + index + ".path")
                );
                long size = Long.parseLong(state.getProperty("library." + index + ".size"));
                if (Files.size(runtimeRoot.resolve("libraries").resolve(relative)) != size) {
                    return false;
                }
            }
            return true;
        } catch (IOException | IllegalArgumentException exception) {
            return false;
        }
    }

    private boolean runtimeStateMetadataMatches(Properties state) {
        try {
            return state.getProperty("minecraftVersion", "").equals(version.minecraftVersion())
                    && Integer.parseInt(state.getProperty("recipeVersion", "0"))
                    == version.recipeVersion()
                    && state.getProperty("sourceServerSha256", "")
                    .equals(version.bundledServerSha256())
                    && state.getProperty("serverSha256", "").matches("[0-9a-f]{64}")
                    && Long.parseLong(state.getProperty("serverSize", "0")) > 0
                    && Long.parseLong(state.getProperty("serverSize", "0"))
                    < version.bundledServerSize()
                    && runtimeLibraryPathsMatch(state);
        } catch (NumberFormatException exception) {
            return false;
        }
    }

    private boolean runtimeLibraryPathsMatch(Properties state) {
        try {
            int count = Integer.parseInt(state.getProperty("library.count", "0"));
            if (count != version.runtimeLibraryPaths().size()) {
                return false;
            }
            Set<String> paths = new LinkedHashSet<>();
            for (int index = 0; index < count; index++) {
                paths.add(safeRelativePath(
                        state.getProperty("library." + index + ".path")
                ).toString().replace('\\', '/'));
            }
            return paths.size() == count
                    && paths.equals(new LinkedHashSet<>(version.runtimeLibraryPaths()));
        } catch (IllegalArgumentException exception) {
            return false;
        }
    }

    private static Properties loadProperties(Path path) {
        Properties properties = new Properties();
        if (!Files.isRegularFile(path)) {
            return properties;
        }
        try (InputStream input = Files.newInputStream(path)) {
            properties.load(input);
            return properties;
        } catch (IOException exception) {
            return new Properties();
        }
    }

    private void writeState() throws IOException {
        Properties state = new Properties();
        state.setProperty("minecraftVersion", version.minecraftVersion());
        state.setProperty("serverSha1", version.serverSha1());
        state.setProperty("bundledServerSha256", version.bundledServerSha256());
        state.setProperty("recipeVersion", Integer.toString(version.recipeVersion()));
        Path temporary = Files.createTempFile(sourceRoot(), "source-", ".properties.part");
        try (OutputStream output = Files.newOutputStream(temporary)) {
            state.store(output, "mc-loot-finder verified runtime source");
        }
        atomicReplace(temporary, sourceRoot().resolve(STATE_FILE));
    }

    private static boolean verifiedFile(
            Path path,
            long expectedSize,
            String algorithm,
            String expectedHash
    ) {
        try {
            requireVerified(path, expectedSize, algorithm, expectedHash);
            return true;
        } catch (IOException | IllegalStateException exception) {
            return false;
        }
    }

    private static void requireVerified(
            Path path,
            long expectedSize,
            String algorithm,
            String expectedHash
    ) throws IOException {
        if (!Files.isRegularFile(path) || Files.size(path) != expectedSize) {
            throw new IllegalStateException("Unexpected file size for " + path);
        }
        String actual = digest(path, algorithm);
        if (!actual.equals(expectedHash)) {
            throw new IllegalStateException(
                    algorithm + " mismatch for " + path + ": expected "
                            + expectedHash + ", got " + actual
            );
        }
    }

    private static String digest(Path path, String algorithm) throws IOException {
        MessageDigest digest;
        try {
            digest = MessageDigest.getInstance(algorithm);
        } catch (NoSuchAlgorithmException exception) {
            throw new IllegalStateException("Missing digest algorithm " + algorithm, exception);
        }
        try (InputStream input = Files.newInputStream(path)) {
            byte[] buffer = new byte[64 * 1024];
            int read;
            while ((read = input.read(buffer)) >= 0) {
                digest.update(buffer, 0, read);
            }
        }
        return HexFormat.of().formatHex(digest.digest());
    }

    private static void atomicReplace(Path source, Path destination) throws IOException {
        try {
            Files.move(
                    source,
                    destination,
                    StandardCopyOption.ATOMIC_MOVE,
                    StandardCopyOption.REPLACE_EXISTING
            );
        } catch (AtomicMoveNotSupportedException exception) {
            Files.move(source, destination, StandardCopyOption.REPLACE_EXISTING);
        }
    }

    private static Path safeRelativePath(String value) {
        if (value == null || value.isBlank()) {
            throw new IllegalArgumentException("Runtime library path is missing");
        }
        Path relative = Path.of(value).normalize();
        if (relative.isAbsolute() || relative.startsWith("..")) {
            throw new IllegalArgumentException("Unsafe runtime library path: " + value);
        }
        return relative;
    }

    private static void deleteRecursively(Path path) throws IOException {
        if (!Files.exists(path)) {
            return;
        }
        try (var paths = Files.walk(path)) {
            for (Path entry : paths.sorted(Comparator.reverseOrder()).toList()) {
                Files.deleteIfExists(entry);
            }
        }
    }

    private Path versionRoot() {
        return cacheRoot.resolve(version.minecraftVersion());
    }

    private Path sourceRoot() {
        return versionRoot().resolve("source");
    }

    private static Path defaultCacheRoot() {
        String override = System.getenv("MC_LOOT_FINDER_CACHE");
        if (override != null && !override.isBlank()) {
            return Path.of(override);
        }
        String os = System.getProperty("os.name", "").toLowerCase(java.util.Locale.ROOT);
        if (os.contains("win")) {
            String localAppData = System.getenv("LOCALAPPDATA");
            if (localAppData != null && !localAppData.isBlank()) {
                return Path.of(localAppData, "mc-loot-finder");
            }
        } else {
            String xdg = System.getenv("XDG_CACHE_HOME");
            if (xdg != null && !xdg.isBlank()) {
                return Path.of(xdg, "mc-loot-finder");
            }
        }
        return Path.of(System.getProperty("user.home"), ".cache", "mc-loot-finder");
    }

    private record RuntimeLibrary(Path path, long size, String sha256) {
    }
}
