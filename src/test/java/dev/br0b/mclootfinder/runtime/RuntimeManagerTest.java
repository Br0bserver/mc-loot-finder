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
import java.util.zip.ZipOutputStream;

import static org.junit.jupiter.api.Assertions.assertFalse;
import static org.junit.jupiter.api.Assertions.assertThrows;
import static org.junit.jupiter.api.Assertions.assertTrue;

class RuntimeManagerTest {
    @TempDir
    Path temporaryDirectory;

    @Test
    void installsAndVerifiesASuppliedBundledServerJar() throws Exception {
        byte[] inner = "named minecraft server classes".getBytes(StandardCharsets.UTF_8);
        Path outer = temporaryDirectory.resolve("server.jar");
        try (ZipOutputStream zip = new ZipOutputStream(Files.newOutputStream(outer))) {
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
                1
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
        assertTrue(manager.verifySource());
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
}
