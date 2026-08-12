package dev.br0b.mclootfinder.vanilla;

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

class JsonLootTableOracleParityTest {
    private static final List<String> TABLES = List.of(
            "minecraft:chests/ancient_city",
            "minecraft:chests/bastion_bridge",
            "minecraft:chests/bastion_hoglin_stable",
            "minecraft:chests/bastion_other",
            "minecraft:chests/bastion_treasure",
            "minecraft:chests/desert_pyramid",
            "minecraft:chests/jungle_temple",
            "minecraft:chests/jungle_temple_dispenser",
            "minecraft:chests/igloo_chest",
            "minecraft:chests/end_city_treasure",
            "minecraft:chests/ruined_portal",
            "minecraft:chests/trial_chambers/corridor",
            "minecraft:chests/trial_chambers/entrance",
            "minecraft:chests/trial_chambers/intersection",
            "minecraft:chests/trial_chambers/intersection_barrel",
            "minecraft:chests/trial_chambers/supply",
            "minecraft:chests/woodland_mansion"
    );

    @Test
    void jsonInterpreterMatchesAllSupportedVanillaTables() {
        try (VanillaRuntime26_1_2 runtime = VanillaRuntime26_1_2.load(0L)) {
            ServerLevel level = mock(ServerLevel.class);
            MinecraftServer server = mock(MinecraftServer.class);
            when(level.getServer()).thenReturn(server);
            when(level.registryAccess()).thenReturn(runtime.registries());
            when(server.reloadableRegistries()).thenReturn(runtime.reloadableResources().fullRegistries());
            LootParams params = new LootParams.Builder(level)
                    .withParameter(LootContextParams.ORIGIN, Vec3.atCenterOf(BlockPos.ZERO))
                    .create(LootContextParamSets.CHEST);
            JsonLootTableOracle26_1_2 oracle = new JsonLootTableOracle26_1_2(runtime.registries());

            for (String tableId : TABLES) {
                Identifier id = Identifier.parse(tableId);
                ResourceKey<LootTable> key = ResourceKey.create(Registries.LOOT_TABLE, id);
                LootTable vanilla = runtime.reloadableResources().fullRegistries().getLootTable(key);
                for (long seed = 1; seed <= 2_048; seed++) {
                    assertEquals(
                            vanillaRoll(vanilla, params, id, seed),
                            oracle.roll(tableId, seed),
                            tableId + " seed=" + seed
                    );
                }
            }
        }
    }

    private static List<LootStack> vanillaRoll(
            LootTable table,
            LootParams params,
            Identifier sequence,
            long seed
    ) {
        LootContext context = new LootContext.Builder(params)
                .withOptionalRandomSeed(seed)
                .create(Optional.of(sequence));
        List<LootStack> result = new ArrayList<>();
        table.getRandomItemsRaw(context, stack -> result.add(new LootStack(
                BuiltInRegistries.ITEM.getKey(stack.getItem()).toString(),
                stack.getCount()
        )));
        return List.copyOf(result);
    }
}
