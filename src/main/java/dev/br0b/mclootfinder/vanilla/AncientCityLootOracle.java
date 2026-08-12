package dev.br0b.mclootfinder.vanilla;

import net.minecraft.core.RegistryAccess;

import java.util.List;

/** Compatibility facade over the data-driven 26.1.2 loot interpreter. */
public final class AncientCityLootOracle {
    public static final String TABLE = "minecraft:chests/ancient_city";
    public static final String SILENCE = "minecraft:silence_armor_trim_smithing_template";

    private final LootOracle delegate;

    public AncientCityLootOracle(RegistryAccess registries) {
        this.delegate = new JsonLootTableOracle26_1_2(registries);
    }

    public List<LootStack> roll(long lootTableSeed) {
        return delegate.roll(TABLE, lootTableSeed);
    }

    public boolean contains(long lootTableSeed, String item) {
        return delegate.contains(TABLE, lootTableSeed, item);
    }
}
