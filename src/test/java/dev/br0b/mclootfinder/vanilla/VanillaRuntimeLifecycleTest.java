package dev.br0b.mclootfinder.vanilla;

import org.junit.jupiter.api.Test;

import java.nio.file.Files;
import java.nio.file.Path;

import static org.junit.jupiter.api.Assertions.assertFalse;
import static org.junit.jupiter.api.Assertions.assertTrue;

class VanillaRuntimeLifecycleTest {
    @Test
    void closeDeletesItsTemplateScratchDirectory() {
        Path scratch;
        try (VanillaRuntime26_1_2 runtime = VanillaRuntime26_1_2.load(0L)) {
            scratch = runtime.scratchPath();
            assertTrue(Files.isDirectory(scratch));
        }

        assertFalse(Files.exists(scratch));
    }
}
