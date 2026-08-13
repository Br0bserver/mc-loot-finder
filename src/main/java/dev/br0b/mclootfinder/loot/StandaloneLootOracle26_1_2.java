package dev.br0b.mclootfinder.loot;

import com.google.gson.JsonElement;
import com.google.gson.JsonObject;
import com.google.gson.JsonParser;
import dev.br0b.mclootfinder.core.random.LegacyRandom48;
import dev.br0b.mclootfinder.vanilla.LootOracle;
import dev.br0b.mclootfinder.vanilla.LootStack;

import java.io.IOException;
import java.io.InputStreamReader;
import java.io.UncheckedIOException;
import java.nio.charset.StandardCharsets;
import java.util.ArrayList;
import java.util.HashMap;
import java.util.HashSet;
import java.util.List;
import java.util.Map;
import java.util.Set;

/**
 * Version-pinned loot interpreter that does not initialize Minecraft's server runtime.
 * The bundled snapshot contains only the item and enchantment metadata that affects RNG.
 */
public final class StandaloneLootOracle26_1_2 implements LootOracle {
    private static final String SNAPSHOT = "/mclootfinder/26.1.2/loot-runtime.json";

    private final LootData data = loadData();
    private final Map<String, Table> tables = new HashMap<>();

    @Override
    public List<LootStack> roll(String lootTable, long lootTableSeed) {
        if (lootTableSeed == 0L) {
            throw new IllegalArgumentException(
                    "LootTableSeed 0 is vanilla's unseeded sentinel"
            );
        }
        LegacyRandom48 random = new LegacyRandom48(lootTableSeed);
        List<LootStack> result = new ArrayList<>();
        rollInto(table(lootTable), random, result);
        return List.copyOf(result);
    }

    private void rollInto(Table table, LegacyRandom48 random, List<LootStack> result) {
        for (Pool pool : table.pools()) {
            if (pool.randomChance() != null && random.nextFloat() >= pool.randomChance()) {
                continue;
            }
            int rolls = pool.rolls().nextInt(random);
            for (int roll = 0; roll < rolls; roll++) {
                Entry entry = select(pool.entries(), random);
                if (entry.randomChance() != null && random.nextFloat() >= entry.randomChance()) {
                    continue;
                }
                if (entry.nestedTable() != null) {
                    rollInto(table(entry.nestedTable()), random, result);
                    continue;
                }
                if (entry.itemId() == null) {
                    continue;
                }
                MutableStack stack = new MutableStack(entry.itemId(), 1);
                for (Function function : entry.functions()) {
                    apply(function, stack, random);
                }
                for (Function function : pool.functions()) {
                    apply(function, stack, random);
                }
                result.add(new LootStack(stack.item(), stack.count()));
            }
        }
    }

    private void apply(Function function, MutableStack stack, LegacyRandom48 random) {
        switch (function.type()) {
            case "minecraft:set_count" -> stack.setCount(function.number().nextInt(random));
            case "minecraft:set_damage" -> {
                if (data.item(stack.item()).damageable()) {
                    function.number().consumeFloat(random);
                }
            }
            case "minecraft:enchant_randomly" -> enchantRandomly(
                    stack, random, function.options(), function.onlyCompatible()
            );
            case "minecraft:enchant_with_levels" -> enchantWithLevels(
                    stack, random, function.number().nextInt(random), function.options()
            );
            case "minecraft:set_potion", "minecraft:exploration_map", "minecraft:set_name" -> {
            }
            case "minecraft:set_ominous_bottle_amplifier" ->
                    function.number().nextInt(random);
            case "minecraft:set_instrument" -> {
                if (data.goatHorns() != 0) {
                    random.nextInt(data.goatHorns());
                }
            }
            case "minecraft:set_stew_effect" -> {
                int selected = random.nextInt(function.alternatives().size());
                function.alternatives().get(selected).consumeFloat(random);
            }
            default -> throw new IllegalStateException(
                    "Unsupported 26.1.2 loot function: " + function.type()
            );
        }
    }

    private void enchantRandomly(
            MutableStack stack,
            LegacyRandom48 random,
            List<String> options,
            boolean onlyCompatible
    ) {
        List<EnchantmentSpec> choices = enchantments(options).stream()
                .filter(enchantment -> stack.item().equals("minecraft:book")
                        || !onlyCompatible
                        || enchantment.compatibleItems().contains(stack.item()))
                .toList();
        if (choices.isEmpty()) {
            return;
        }
        EnchantmentSpec selected = choices.get(random.nextInt(choices.size()));
        nextInclusive(random, 1, selected.maxLevel());
        if (stack.item().equals("minecraft:book")) {
            stack.setItem("minecraft:enchanted_book");
        }
    }

    private void enchantWithLevels(
            MutableStack stack,
            LegacyRandom48 random,
            int level,
            List<String> options
    ) {
        ItemSpec item = data.item(stack.item());
        if (item.enchantability() == 0) {
            return;
        }

        int spread = item.enchantability() / 4 + 1;
        level += 1 + random.nextInt(spread) + random.nextInt(spread);
        float variance = (random.nextFloat() + random.nextFloat() - 1.0f) * 0.15f;
        level = Math.max(1, Math.round(level + level * variance));

        List<AvailableEnchantment> available = availableEnchantments(
                stack.item(), level, enchantments(options)
        );
        if (available.isEmpty()) {
            return;
        }

        List<AvailableEnchantment> selected = new ArrayList<>();
        chooseWeighted(available, random, selected);
        while (random.nextInt(50) <= level) {
            if (!selected.isEmpty()) {
                String last = selected.getLast().enchantment().id();
                available.removeIf(candidate -> !compatible(
                        candidate.enchantment().id(), last
                ));
            }
            if (available.isEmpty()) {
                break;
            }
            chooseWeighted(available, random, selected);
            level /= 2;
        }

        if (stack.item().equals("minecraft:book")) {
            stack.setItem("minecraft:enchanted_book");
        }
    }

    private List<AvailableEnchantment> availableEnchantments(
            String item,
            int level,
            List<EnchantmentSpec> enchantments
    ) {
        boolean book = item.equals("minecraft:book");
        List<AvailableEnchantment> available = new ArrayList<>();
        for (EnchantmentSpec enchantment : enchantments) {
            if (!book && !enchantment.primaryItems().contains(item)) {
                continue;
            }
            for (int enchantmentLevel = enchantment.maxLevel(); enchantmentLevel >= 1;
                 enchantmentLevel--) {
                if (level >= enchantment.minCost(enchantmentLevel)
                        && level <= enchantment.maxCost(enchantmentLevel)) {
                    available.add(new AvailableEnchantment(enchantment, enchantmentLevel));
                    break;
                }
            }
        }
        return available;
    }

    private static void chooseWeighted(
            List<AvailableEnchantment> available,
            LegacyRandom48 random,
            List<AvailableEnchantment> selected
    ) {
        int totalWeight = available.stream()
                .mapToInt(value -> value.enchantment().weight())
                .sum();
        int choice = random.nextInt(totalWeight);
        for (AvailableEnchantment value : available) {
            choice -= value.enchantment().weight();
            if (choice < 0) {
                selected.add(value);
                return;
            }
        }
        throw new IllegalStateException("Enchantment weights are inconsistent");
    }

    private boolean compatible(String first, String second) {
        if (first.equals(second)) {
            return false;
        }
        return !data.enchantment(first).exclusive().contains(second)
                && !data.enchantment(second).exclusive().contains(first);
    }

    private List<EnchantmentSpec> enchantments(List<String> options) {
        if (options.size() == 1 && options.getFirst().equals("#minecraft:on_random_loot")) {
            return data.onRandomLoot().stream().map(data::enchantment).toList();
        }
        return options.stream().map(data::enchantment).toList();
    }

    private static Entry select(List<Entry> entries, LegacyRandom48 random) {
        if (entries.size() == 1) {
            return entries.getFirst();
        }
        int totalWeight = entries.stream().mapToInt(Entry::weight).sum();
        int choice = random.nextInt(totalWeight);
        for (Entry entry : entries) {
            choice -= entry.weight();
            if (choice < 0) {
                return entry;
            }
        }
        throw new IllegalStateException("Loot entry weights are inconsistent");
    }

    private Table table(String lootTable) {
        return tables.computeIfAbsent(lootTable, StandaloneLootOracle26_1_2::loadTable);
    }

    private static Table loadTable(String lootTable) {
        String[] id = lootTable.split(":", 2);
        String path = "/data/" + id[0] + "/loot_table/" + id[1] + ".json";
        try (var stream = StandaloneLootOracle26_1_2.class.getResourceAsStream(path)) {
            if (stream == null) {
                throw new IllegalArgumentException("Vanilla loot table not found: " + lootTable);
            }
            JsonObject root = JsonParser.parseReader(new InputStreamReader(
                    stream, StandardCharsets.UTF_8
            )).getAsJsonObject();
            if (root.has("functions")) {
                throw new IllegalStateException("Table-level functions are unsupported: " + lootTable);
            }
            List<Pool> pools = new ArrayList<>();
            for (JsonElement poolElement : root.getAsJsonArray("pools")) {
                JsonObject pool = poolElement.getAsJsonObject();
                List<Entry> entries = new ArrayList<>();
                for (JsonElement entryElement : pool.getAsJsonArray("entries")) {
                    JsonObject entry = entryElement.getAsJsonObject();
                    String type = entry.get("type").getAsString();
                    if (!type.equals("minecraft:item") && !type.equals("minecraft:empty")
                            && !type.equals("minecraft:loot_table")) {
                        throw new IllegalStateException(
                                "Unsupported loot entry in " + lootTable + ": " + type
                        );
                    }
                    if (entry.has("quality")) {
                        throw new IllegalStateException("Quality entries are unsupported: " + lootTable);
                    }
                    List<Function> functions = parseFunctions(entry);
                    if (type.equals("minecraft:loot_table") && !functions.isEmpty()) {
                        throw new IllegalStateException(
                                "Functions on nested loot tables are unsupported: " + lootTable
                        );
                    }
                    entries.add(new Entry(
                            type.equals("minecraft:item") ? entry.get("name").getAsString() : null,
                            type.equals("minecraft:loot_table") ? entry.get("value").getAsString() : null,
                            entry.has("weight") ? entry.get("weight").getAsInt() : 1,
                            functions,
                            parseRandomChance(entry, lootTable)
                    ));
                }
                pools.add(new Pool(
                        parseNumber(pool.get("rolls")),
                        List.copyOf(entries),
                        parseFunctions(pool),
                        parseRandomChance(pool, lootTable)
                ));
            }
            return new Table(List.copyOf(pools));
        } catch (IOException exception) {
            throw new UncheckedIOException("Could not read " + lootTable, exception);
        }
    }

    private static List<Function> parseFunctions(JsonObject owner) {
        if (!owner.has("functions")) {
            return List.of();
        }
        List<Function> functions = new ArrayList<>();
        for (JsonElement element : owner.getAsJsonArray("functions")) {
            functions.add(parseFunction(element.getAsJsonObject()));
        }
        return List.copyOf(functions);
    }

    private static Function parseFunction(JsonObject json) {
        String type = json.get("function").getAsString();
        return switch (type) {
            case "minecraft:set_count" -> new Function(
                    type, parseNumber(json.get("count")), null, true, List.of()
            );
            case "minecraft:set_damage" -> new Function(
                    type, parseNumber(json.get("damage")), null, true, List.of()
            );
            case "minecraft:enchant_randomly" -> new Function(
                    type,
                    null,
                    parseOptions(json.get("options")),
                    !json.has("only_compatible") || json.get("only_compatible").getAsBoolean(),
                    List.of()
            );
            case "minecraft:enchant_with_levels" -> new Function(
                    type,
                    parseNumber(json.get("levels")),
                    parseOptions(json.get("options")),
                    true,
                    List.of()
            );
            case "minecraft:set_potion", "minecraft:exploration_map", "minecraft:set_name" ->
                    new Function(type, null, null, true, List.of());
            case "minecraft:set_ominous_bottle_amplifier" -> new Function(
                    type, parseNumber(json.get("amplifier")), null, true, List.of()
            );
            case "minecraft:set_instrument" -> new Function(
                    type, null, List.of(json.get("options").getAsString()), true, List.of()
            );
            case "minecraft:set_stew_effect" -> {
                List<NumberSpec> effects = new ArrayList<>();
                for (JsonElement effect : json.getAsJsonArray("effects")) {
                    effects.add(parseNumber(effect.getAsJsonObject().get("duration")));
                }
                yield new Function(type, null, null, true, List.copyOf(effects));
            }
            default -> throw new IllegalStateException("Unsupported loot function: " + type);
        };
    }

    private static NumberSpec parseNumber(JsonElement element) {
        if (element.isJsonPrimitive()) {
            float value = element.getAsFloat();
            return new NumberSpec(value, value);
        }
        JsonObject object = element.getAsJsonObject();
        if (!"minecraft:uniform".equals(object.get("type").getAsString())) {
            throw new IllegalStateException("Unsupported number provider: " + object);
        }
        return new NumberSpec(object.get("min").getAsFloat(), object.get("max").getAsFloat());
    }

    private static List<String> parseOptions(JsonElement element) {
        if (element.isJsonPrimitive()) {
            return List.of(element.getAsString());
        }
        List<String> options = new ArrayList<>();
        for (JsonElement option : element.getAsJsonArray()) {
            options.add(option.getAsString());
        }
        if (options.isEmpty()) {
            throw new IllegalStateException("Enchantment options must not be empty");
        }
        return List.copyOf(options);
    }

    private static Double parseRandomChance(JsonObject object, String lootTable) {
        if (!object.has("conditions")) {
            return null;
        }
        var conditions = object.getAsJsonArray("conditions");
        if (conditions.size() != 1
                || !"minecraft:random_chance".equals(
                conditions.get(0).getAsJsonObject().get("condition").getAsString())) {
            throw new IllegalStateException("Unsupported loot condition: " + lootTable);
        }
        return conditions.get(0).getAsJsonObject().get("chance").getAsDouble();
    }

    private static LootData loadData() {
        try (var stream = StandaloneLootOracle26_1_2.class.getResourceAsStream(SNAPSHOT)) {
            if (stream == null) {
                throw new IllegalStateException("Missing loot runtime snapshot: " + SNAPSHOT);
            }
            JsonObject root = JsonParser.parseReader(new InputStreamReader(
                    stream, StandardCharsets.UTF_8
            )).getAsJsonObject();
            Map<String, EnchantmentSpec> enchantments = new HashMap<>();
            for (JsonElement element : root.getAsJsonArray("enchantments")) {
                JsonObject json = element.getAsJsonObject();
                EnchantmentSpec enchantment = new EnchantmentSpec(
                        json.get("id").getAsString(),
                        json.get("weight").getAsInt(),
                        json.get("max_level").getAsInt(),
                        json.get("min_cost_base").getAsInt(),
                        json.get("min_cost_per_level").getAsInt(),
                        json.get("max_cost_base").getAsInt(),
                        json.get("max_cost_per_level").getAsInt(),
                        strings(json.getAsJsonArray("primary_items")),
                        strings(json.getAsJsonArray("compatible_items")),
                        strings(json.getAsJsonArray("exclusive"))
                );
                enchantments.put(enchantment.id(), enchantment);
            }
            Map<String, ItemSpec> items = new HashMap<>();
            for (var entry : root.getAsJsonObject("items").entrySet()) {
                JsonObject json = entry.getValue().getAsJsonObject();
                items.put(entry.getKey(), new ItemSpec(
                        json.get("enchantability").getAsInt(),
                        json.get("damageable").getAsBoolean()
                ));
            }
            return new LootData(
                    Map.copyOf(enchantments),
                    stringsList(root.getAsJsonArray("on_random_loot")),
                    Map.copyOf(items),
                    root.get("goat_horns").getAsInt()
            );
        } catch (IOException exception) {
            throw new UncheckedIOException("Could not load " + SNAPSHOT, exception);
        }
    }

    private static Set<String> strings(com.google.gson.JsonArray array) {
        Set<String> values = new java.util.LinkedHashSet<>();
        array.forEach(value -> values.add(value.getAsString()));
        return Set.copyOf(values);
    }

    private static List<String> stringsList(com.google.gson.JsonArray array) {
        List<String> values = new ArrayList<>();
        array.forEach(value -> values.add(value.getAsString()));
        return List.copyOf(values);
    }

    private static int nextInclusive(LegacyRandom48 random, int min, int max) {
        return min >= max ? min : random.nextInt(max - min + 1) + min;
    }

    private record Table(List<Pool> pools) {
    }

    private record Pool(
            NumberSpec rolls,
            List<Entry> entries,
            List<Function> functions,
            Double randomChance
    ) {
    }

    private record Entry(
            String itemId,
            String nestedTable,
            int weight,
            List<Function> functions,
            Double randomChance
    ) {
    }

    private record Function(
            String type,
            NumberSpec number,
            List<String> options,
            boolean onlyCompatible,
            List<NumberSpec> alternatives
    ) {
    }

    private record NumberSpec(float min, float max) {
        private int nextInt(LegacyRandom48 random) {
            return nextInclusive(random, (int) min, (int) max);
        }

        private void consumeFloat(LegacyRandom48 random) {
            if (min < max) {
                random.nextFloat();
            }
        }
    }

    private record EnchantmentSpec(
            String id,
            int weight,
            int maxLevel,
            int minCostBase,
            int minCostPerLevel,
            int maxCostBase,
            int maxCostPerLevel,
            Set<String> primaryItems,
            Set<String> compatibleItems,
            Set<String> exclusive
    ) {
        private int minCost(int level) {
            return minCostBase + minCostPerLevel * (level - 1);
        }

        private int maxCost(int level) {
            return maxCostBase + maxCostPerLevel * (level - 1);
        }
    }

    private record ItemSpec(int enchantability, boolean damageable) {
    }

    private record AvailableEnchantment(EnchantmentSpec enchantment, int level) {
    }

    private record LootData(
            Map<String, EnchantmentSpec> enchantments,
            List<String> onRandomLoot,
            Map<String, ItemSpec> items,
            int goatHorns
    ) {
        private EnchantmentSpec enchantment(String id) {
            EnchantmentSpec result = enchantments.get(id);
            if (result == null) {
                throw new IllegalStateException("Unknown 26.1.2 enchantment: " + id);
            }
            return result;
        }

        private ItemSpec item(String id) {
            ItemSpec result = items.get(id);
            if (result == null) {
                throw new IllegalStateException("Unknown 26.1.2 item: " + id);
            }
            return result;
        }
    }

    private static final class MutableStack {
        private String item;
        private int count;

        private MutableStack(String item, int count) {
            this.item = item;
            this.count = count;
        }

        private String item() {
            return item;
        }

        private void setItem(String item) {
            this.item = item;
        }

        private int count() {
            return count;
        }

        private void setCount(int count) {
            this.count = count;
        }
    }
}
