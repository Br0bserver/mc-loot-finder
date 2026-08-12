package dev.br0b.mclootfinder.vanilla;

import java.util.List;

public interface LootOracle {
    List<LootStack> roll(String lootTable, long lootTableSeed);

    default boolean contains(String lootTable, long lootTableSeed, String item) {
        return roll(lootTable, lootTableSeed).stream()
                .anyMatch(stack -> stack.item().equals(item));
    }
}
