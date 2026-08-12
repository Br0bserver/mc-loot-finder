package dev.br0b.mclootfinder.vanilla;

import dev.br0b.mclootfinder.core.Versions;
import net.minecraft.core.BlockPos;
import net.minecraft.core.registries.BuiltInRegistries;
import net.minecraft.core.registries.Registries;
import net.minecraft.resources.Identifier;
import net.minecraft.resources.ResourceKey;
import net.minecraft.server.MinecraftServer;
import net.minecraft.server.level.ServerLevel;
import net.minecraft.world.level.storage.loot.LootContext;
import net.minecraft.world.level.storage.loot.LootParams;
import net.minecraft.world.level.storage.loot.LootTable;
import net.minecraft.world.level.storage.loot.parameters.LootContextParamSets;
import net.minecraft.world.level.storage.loot.parameters.LootContextParams;
import net.minecraft.world.phys.Vec3;
import org.junit.jupiter.api.Test;

import java.util.ArrayList;
import java.util.List;
import java.util.Optional;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.mockito.Mockito.mock;
import static org.mockito.Mockito.when;

class AncientCityLootOracleParityTest {
    private static final ResourceKey<LootTable> ANCIENT_CITY = ResourceKey.create(
            Registries.LOOT_TABLE,
            Identifier.withDefaultNamespace("chests/ancient_city")
    );

    @Test
    void compactOracleMatchesVanillaLootTable() {
        try (VanillaRuntime26_1_2 runtime = VanillaRuntime26_1_2.load(0L)) {
            runtime.verifyAncientCityProfile(Versions.V26_1_2);
            ServerLevel level = mock(ServerLevel.class);
            MinecraftServer server = mock(MinecraftServer.class);
            when(level.getServer()).thenReturn(server);
            when(level.registryAccess()).thenReturn(runtime.registries());
            when(server.reloadableRegistries()).thenReturn(runtime.reloadableResources().fullRegistries());

            LootParams params = new LootParams.Builder(level)
                    .withParameter(LootContextParams.ORIGIN, Vec3.atCenterOf(BlockPos.ZERO))
                    .create(LootContextParamSets.CHEST);
            LootTable vanilla = runtime.reloadableResources().fullRegistries().getLootTable(ANCIENT_CITY);
            AncientCityLootOracle oracle = new AncientCityLootOracle(runtime.registries());

            List<Long> seeds = new ArrayList<>();
            seeds.add(2_858_756_560_459_657_823L);
            seeds.add(8_771_329_210_713_860_093L);
            for (long seed = 1; seed <= 4_096; seed++) {
                seeds.add(seed);
            }

            for (long seed : seeds) {
                assertEquals(vanillaRoll(vanilla, params, seed), oracle.roll(seed), "seed=" + seed);
            }
        }
    }

    private static List<LootStack> vanillaRoll(LootTable table, LootParams params, long seed) {
        LootContext context = new LootContext.Builder(params)
                .withOptionalRandomSeed(seed)
                .create(Optional.of(Identifier.withDefaultNamespace("chests/ancient_city")));
        List<LootStack> result = new ArrayList<>();
        table.getRandomItemsRaw(context, stack -> result.add(new LootStack(
                BuiltInRegistries.ITEM.getKey(stack.getItem()).toString(),
                stack.getCount()
        )));
        return List.copyOf(result);
    }
}
