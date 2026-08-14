package dev.br0b.mclootfinder.runtime;

import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.io.TempDir;

import java.io.ByteArrayOutputStream;
import java.io.PrintStream;
import java.net.URI;
import java.nio.charset.StandardCharsets;
import java.nio.file.Files;
import java.nio.file.Path;
import java.security.MessageDigest;
import java.util.HexFormat;
import java.util.zip.ZipEntry;
import java.util.zip.ZipFile;
import java.util.zip.ZipOutputStream;

import static org.junit.jupiter.api.Assertions.assertFalse;
import static org.junit.jupiter.api.Assertions.assertNotNull;
import static org.junit.jupiter.api.Assertions.assertNull;
import static org.junit.jupiter.api.Assertions.assertThrows;
import static org.junit.jupiter.api.Assertions.assertTrue;

class RuntimeManagerTest {
    @TempDir
    Path temporaryDirectory;

    @Test
    void installsAndVerifiesASuppliedBundledServerJar() throws Exception {
        byte[] inner = testInnerJar();
        byte[] library = "official runtime library".getBytes(StandardCharsets.UTF_8);
        byte[] unusedLibrary = "unused official library".getBytes(StandardCharsets.UTF_8);
        String libraryPath = "com/example/runtime/test-runtime.jar";
        String unusedLibraryPath = "com/example/unused/test-unused.jar";
        Path outer = temporaryDirectory.resolve("server.jar");
        try (ZipOutputStream zip = new ZipOutputStream(Files.newOutputStream(outer))) {
            zip.putNextEntry(new ZipEntry("META-INF/libraries.list"));
            zip.write((digest(library, "SHA-256") + "\tcom.example:runtime:test\t"
                    + libraryPath + "\n"
                    + digest(unusedLibrary, "SHA-256") + "\tcom.example:unused:test\t"
                    + unusedLibraryPath + "\n").getBytes(StandardCharsets.UTF_8));
            zip.closeEntry();
            zip.putNextEntry(new ZipEntry("META-INF/libraries/" + libraryPath));
            zip.write(library);
            zip.closeEntry();
            zip.putNextEntry(new ZipEntry("META-INF/libraries/" + unusedLibraryPath));
            zip.write(unusedLibrary);
            zip.closeEntry();
            zip.putNextEntry(new ZipEntry("META-INF/versions/test/server-test.jar"));
            zip.write(inner);
            zip.closeEntry();
        }
        RuntimeVersion version = new RuntimeVersion(
                "test",
                25,
                URI.create("https://invalid.example/server.jar"),
                Files.size(outer),
                digest(outer, "SHA-1"),
                "META-INF/versions/test/server-test.jar",
                inner.length,
                digest(inner, "SHA-256"),
                1,
                java.util.List.of(libraryPath)
        );
        RuntimeManager manager = new RuntimeManager(
                version, temporaryDirectory.resolve("cache")
        );

        assertFalse(manager.status().sourceReady());
        manager.installSource(
                outer,
                true,
                new PrintStream(new ByteArrayOutputStream())
        );

        assertTrue(manager.status().sourceReady());
        assertTrue(manager.status().generatedRuntimeReady());
        assertTrue(manager.verifySource());
        assertTrue(manager.verifyRuntime());
        assertTrue(manager.runtimeClasspath().stream().allMatch(Files::isRegularFile));
        assertTrue(manager.runtimeClasspath().size() == 2);
        assertTrue(manager.runtimeClasspath().get(1).endsWith(libraryPath));
        assertTrue(Files.mismatch(
                manager.runtimeClasspath().get(1),
                writeExpectedLibrary(library)
        ) == -1);
        assertFalse(Files.exists(
                manager.runtimeClasspath().getFirst().getParent()
                        .resolve("libraries").resolve(unusedLibraryPath)
        ));
        assertFalse(Files.exists(
                manager.status().cacheDirectory().resolve("source/server-inner.jar")
        ));
        try (ZipFile runtimeServer = new ZipFile(
                manager.runtimeClasspath().getFirst().toFile()
        )) {
            assertNotNull(runtimeServer.getEntry("net/minecraft/Test.class"));
            assertNull(runtimeServer.getEntry("META-INF/MANIFEST.MF"));
            assertNull(runtimeServer.getEntry("META-INF/MOJANGCS.SF"));
        }
    }

    @Test
    void offlineInstallFailsWithoutASuppliedJar() {
        RuntimeManager manager = new RuntimeManager(
                RuntimeVersion.V26_1_2, temporaryDirectory.resolve("cache")
        );

        assertThrows(IllegalStateException.class, () -> manager.installSource(
                null,
                true,
                new PrintStream(new ByteArrayOutputStream())
        ));
    }

    private static String digest(Path path, String algorithm) throws Exception {
        return digest(Files.readAllBytes(path), algorithm);
    }

    private static String digest(byte[] value, String algorithm) throws Exception {
        return HexFormat.of().formatHex(
                MessageDigest.getInstance(algorithm).digest(value)
        );
    }

    private static byte[] testInnerJar() throws Exception {
        ByteArrayOutputStream bytes = new ByteArrayOutputStream();
        try (ZipOutputStream zip = new ZipOutputStream(bytes)) {
            zip.putNextEntry(new ZipEntry("META-INF/MANIFEST.MF"));
            zip.write("Manifest-Version: 1.0\n\n".getBytes(StandardCharsets.UTF_8));
            zip.closeEntry();
            zip.putNextEntry(new ZipEntry("META-INF/MOJANGCS.SF"));
            zip.write("signature metadata".getBytes(StandardCharsets.UTF_8));
            zip.closeEntry();
            zip.putNextEntry(new ZipEntry("net/minecraft/Test.class"));
            zip.write("named minecraft server classes".getBytes(StandardCharsets.UTF_8));
            zip.closeEntry();
        }
        return bytes.toByteArray();
    }

    private Path writeExpectedLibrary(byte[] bytes) throws Exception {
        Path expected = temporaryDirectory.resolve("expected-library.jar");
        Files.write(expected, bytes);
        return expected;
    }
}
