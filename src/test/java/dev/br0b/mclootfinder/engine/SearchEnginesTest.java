package dev.br0b.mclootfinder.engine;

import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertEquals;

class SearchEnginesTest {
    @Test
    void discoversTheVanillaOracleProvider() {
        assertEquals(java.util.List.of("26.1.2"), SearchEngines.availableVersions());
    }
}
