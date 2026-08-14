package dev.br0b.mclootfinder.runtime;

import java.net.URI;
import java.util.List;
import java.util.Map;

/** Pinned official inputs used to construct a local search runtime. */
public record RuntimeVersion(
        String minecraftVersion,
        int javaMajorVersion,
        URI serverUri,
        long serverSize,
        String serverSha1,
        String bundledServerPath,
        long bundledServerSize,
        String bundledServerSha256,
        int recipeVersion,
        List<String> runtimeLibraryPaths,
        Map<String, String> compactLibraryClassLists
) {
    public RuntimeVersion {
        if (minecraftVersion.isBlank() || javaMajorVersion <= 0 || serverSize <= 0
                || bundledServerSize <= 0 || recipeVersion <= 0) {
            throw new IllegalArgumentException("Invalid runtime version metadata");
        }
        runtimeLibraryPaths = List.copyOf(runtimeLibraryPaths);
        if (runtimeLibraryPaths.isEmpty()
                || runtimeLibraryPaths.stream().anyMatch(String::isBlank)
                || runtimeLibraryPaths.stream().distinct().count() != runtimeLibraryPaths.size()) {
            throw new IllegalArgumentException("Invalid runtime library list");
        }
        compactLibraryClassLists = Map.copyOf(compactLibraryClassLists);
        if (!runtimeLibraryPaths.containsAll(compactLibraryClassLists.keySet())
                || compactLibraryClassLists.values().stream().anyMatch(String::isBlank)) {
            throw new IllegalArgumentException("Invalid compact runtime library recipes");
        }
    }

    public static final RuntimeVersion V26_1_2 = new RuntimeVersion(
            "26.1.2",
            25,
            URI.create(
                    "https://piston-data.mojang.com/v1/objects/"
                            + "97ccd4c0ed3f81bbb7bfacddd1090b0c56f9bc51/server.jar"
            ),
            60_417_480L,
            "97ccd4c0ed3f81bbb7bfacddd1090b0c56f9bc51",
            "META-INF/versions/26.1.2/server-26.1.2.jar",
            24_555_215L,
            "4723380bd2a0a0206719b50f2e390383afdaf82b0a76a0d573baf788e6aa3e86",
            4,
            List.of(
                    "com/google/guava/failureaccess/1.0.3/failureaccess-1.0.3.jar",
                    "com/google/guava/guava/33.5.0-jre/guava-33.5.0-jre.jar",
                    "com/mojang/authlib/7.0.63/authlib-7.0.63.jar",
                    "com/mojang/brigadier/1.3.10/brigadier-1.3.10.jar",
                    "com/mojang/datafixerupper/9.0.19/datafixerupper-9.0.19.jar",
                    "com/mojang/jtracy/1.0.37/jtracy-1.0.37.jar",
                    "com/mojang/logging/1.6.11/logging-1.6.11.jar",
                    "io/netty/netty-buffer/4.2.7.Final/netty-buffer-4.2.7.Final.jar",
                    "io/netty/netty-codec-base/4.2.7.Final/netty-codec-base-4.2.7.Final.jar",
                    "io/netty/netty-common/4.2.7.Final/netty-common-4.2.7.Final.jar",
                    "it/unimi/dsi/fastutil/8.5.18/fastutil-8.5.18.jar",
                    "org/apache/commons/commons-lang3/3.19.0/commons-lang3-3.19.0.jar",
                    "org/apache/logging/log4j/log4j-api/2.25.2/log4j-api-2.25.2.jar",
                    "org/apache/logging/log4j/log4j-core/2.25.2/log4j-core-2.25.2.jar",
                    "org/apache/logging/log4j/log4j-slf4j2-impl/2.25.2/"
                            + "log4j-slf4j2-impl-2.25.2.jar",
                    "org/joml/joml/1.10.8/joml-1.10.8.jar",
                    "org/slf4j/slf4j-api/2.0.17/slf4j-api-2.0.17.jar"
            ),
            Map.of(
                    "it/unimi/dsi/fastutil/8.5.18/fastutil-8.5.18.jar",
                    "/mclootfinder/26.1.2/fastutil-runtime-classes.txt"
            )
    );

    public static RuntimeVersion require(String minecraftVersion) {
        if (V26_1_2.minecraftVersion().equals(minecraftVersion)) {
            return V26_1_2;
        }
        throw new IllegalArgumentException(
                "Runtime installation is not available for Minecraft Java " + minecraftVersion
        );
    }
}
