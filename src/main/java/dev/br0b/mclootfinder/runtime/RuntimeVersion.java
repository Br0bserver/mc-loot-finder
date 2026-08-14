package dev.br0b.mclootfinder.runtime;

import java.net.URI;

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
        int recipeVersion
) {
    public RuntimeVersion {
        if (minecraftVersion.isBlank() || javaMajorVersion <= 0 || serverSize <= 0
                || bundledServerSize <= 0 || recipeVersion <= 0) {
            throw new IllegalArgumentException("Invalid runtime version metadata");
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
            1
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
