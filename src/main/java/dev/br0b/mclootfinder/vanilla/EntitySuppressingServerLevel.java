package dev.br0b.mclootfinder.vanilla;

import net.minecraft.resources.ResourceKey;
import net.minecraft.server.MinecraftServer;
import net.minecraft.server.level.ServerLevel;
import net.minecraft.world.flag.FeatureFlagSet;
import net.minecraft.world.level.CustomSpawner;
import net.minecraft.world.level.Level;
import net.minecraft.world.level.dimension.LevelStem;
import net.minecraft.world.level.storage.LevelStorageSource;
import net.minecraft.world.level.storage.ServerLevelData;
import org.objenesis.ObjenesisStd;

import java.util.List;
import java.util.concurrent.Executor;

/**
 * A constructor-free ServerLevel whose only purpose is to make entity data
 * markers no-ops while structure templates are being inspected for chests.
 *
 * <p>Vanilla checks {@link #enabledFeatures()} before invoking an entity's
 * factory. The empty feature set disables vanilla entity types, so none of the
 * uninitialised ServerLevel state is observed.</p>
 */
final class EntitySuppressingServerLevel extends ServerLevel {
    private static final ObjenesisStd OBJENESIS = new ObjenesisStd();

    private EntitySuppressingServerLevel() {
        // Never invoked: Objenesis allocates this class without running either
        // this constructor or ServerLevel's heavyweight constructor.
        super(
                (MinecraftServer) null,
                (Executor) null,
                (LevelStorageSource.LevelStorageAccess) null,
                (ServerLevelData) null,
                (ResourceKey<Level>) null,
                (LevelStem) null,
                false,
                0L,
                List.<CustomSpawner>of(),
                false
        );
    }

    static ServerLevel create() {
        return OBJENESIS.newInstance(EntitySuppressingServerLevel.class);
    }

    @Override
    public FeatureFlagSet enabledFeatures() {
        return FeatureFlagSet.of();
    }
}
