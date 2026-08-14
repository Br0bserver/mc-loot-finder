package dev.br0b.mclootfinder.runtime;

import java.io.IOException;
import java.io.InputStream;
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
import java.security.MessageDigest;
import java.security.NoSuchAlgorithmException;
import java.time.Duration;
import java.util.HexFormat;
import java.util.Properties;
import java.util.zip.ZipEntry;
import java.util.zip.ZipFile;

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
                Files.isRegularFile(versionRoot.resolve("runtime/runtime.jar"))
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
                if (sourceStateMatches()) {
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
        ) && verifiedFile(
                sourceRoot().resolve("server-inner.jar"),
                version.bundledServerSize(),
                "SHA-256",
                version.bundledServerSha256()
        );
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
                    && Files.size(sourceRoot().resolve("server.jar")) == version.serverSize()
                    && Files.size(sourceRoot().resolve("server-inner.jar"))
                    == version.bundledServerSize();
        } catch (IOException | NumberFormatException exception) {
            return false;
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
        String actual = HexFormat.of().formatHex(digest.digest());
        if (!actual.equals(expectedHash)) {
            throw new IllegalStateException(
                    algorithm + " mismatch for " + path + ": expected "
                            + expectedHash + ", got " + actual
            );
        }
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
}
