package dev.br0b.mclootfinder.vanilla;

import com.google.gson.JsonElement;
import com.google.gson.JsonObject;
import com.google.gson.JsonParser;
import net.minecraft.core.Holder;
import net.minecraft.core.HolderSet;
import net.minecraft.core.RegistryAccess;
import net.minecraft.core.registries.BuiltInRegistries;
import net.minecraft.core.registries.Registries;
import net.minecraft.resources.Identifier;
import net.minecraft.resources.ResourceKey;
import net.minecraft.tags.TagKey;
import net.minecraft.util.RandomSource;
import net.minecraft.world.item.Item;
import net.minecraft.world.item.ItemStack;
import net.minecraft.world.item.Items;
import net.minecraft.world.item.enchantment.Enchantment;
import net.minecraft.world.item.enchantment.EnchantmentHelper;

import java.io.IOException;
import java.io.InputStreamReader;
import java.io.UncheckedIOException;
import java.nio.charset.StandardCharsets;
import java.util.ArrayList;
import java.util.HashMap;
import java.util.List;
import java.util.Map;
import java.util.Optional;

/**
 * Small, fail-closed interpreter for the function subset used by the 26.1.2
 * ancient-city, bastion, and desert-pyramid chest tables. Definitions come from the
 * bundled vanilla data pack rather than duplicated Java entry lists.
 */
public final class JsonLootTableOracle26_1_2 implements LootOracle {
    private final RegistryAccess registries;
    private final Map<String, Table> tables = new HashMap<>();

    public JsonLootTableOracle26_1_2(RegistryAccess registries) {
        this.registries = registries;
    }

    @Override
    public List<LootStack> roll(String lootTable, long lootTableSeed) {
        if (lootTableSeed == 0L) {
            throw new IllegalArgumentException(
                    "LootTableSeed 0 is vanilla's unseeded sentinel"
            );
        }
        Table table = tables.computeIfAbsent(lootTable, this::load);
        RandomSource random = RandomSource.create(lootTableSeed);
        List<LootStack> result = new ArrayList<>();
        rollInto(table, random, result);
        return List.copyOf(result);
    }

    private void rollInto(Table table, RandomSource random, List<LootStack> result) {
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
                    rollInto(tables.computeIfAbsent(entry.nestedTable(), this::load), random, result);
                    continue;
                }
                if (entry.itemId() == null) {
                    continue;
                }
                Item item = BuiltInRegistries.ITEM.getValue(Identifier.parse(entry.itemId()));
                if (item == null) {
                    throw new IllegalStateException("Unknown vanilla item: " + entry.itemId());
                }
                ItemStack stack = new ItemStack(item);
                for (Function function : entry.functions()) {
                    stack = apply(function, stack, random);
                }
                for (Function function : pool.functions()) {
                    stack = apply(function, stack, random);
                }
                result.add(new LootStack(
                        BuiltInRegistries.ITEM.getKey(stack.getItem()).toString(),
                        stack.getCount()
                ));
            }
        }
    }

    private ItemStack apply(Function function, ItemStack stack, RandomSource random) {
        return switch (function.type()) {
            case "minecraft:set_count" -> {
                stack.setCount(function.number().nextInt(random));
                yield stack;
            }
            case "minecraft:set_damage" -> {
                if (stack.isDamageableItem()) {
                    function.number().consumeFloat(random);
                }
                yield stack;
            }
            case "minecraft:enchant_randomly" -> enchantRandomly(
                    stack, random, function.options(), function.onlyCompatible()
            );
            case "minecraft:enchant_with_levels" -> EnchantmentHelper.enchantItem(
                    random,
                    stack,
                    function.number().nextInt(random),
                    registries,
                    Optional.of(resolveOptions(function.options()))
            );
            case "minecraft:set_potion" -> stack;
            case "minecraft:set_ominous_bottle_amplifier" -> {
                function.number().nextInt(random);
                yield stack;
            }
            case "minecraft:exploration_map", "minecraft:set_name" -> stack;
            case "minecraft:set_instrument" -> {
                var instruments = registries.lookupOrThrow(Registries.INSTRUMENT);
                TagKey<net.minecraft.world.item.Instrument> tag = TagKey.create(
                        Registries.INSTRUMENT,
                        Identifier.parse(function.options().getFirst().substring(1))
                );
                List<?> choices = instruments.getOrThrow(tag).stream().toList();
                if (!choices.isEmpty()) {
                    random.nextInt(choices.size());
                }
                yield stack;
            }
            case "minecraft:set_stew_effect" -> {
                int selected = random.nextInt(function.alternatives().size());
                function.alternatives().get(selected).consumeFloat(random);
                yield stack;
            }
            default -> throw new IllegalStateException(
                    "Unsupported 26.1.2 loot function: " + function.type()
            );
        };
    }

    private ItemStack enchantRandomly(
            ItemStack stack,
            RandomSource random,
            List<String> options,
            boolean onlyCompatible
    ) {
        ItemStack inputStack = stack;
        List<Holder<Enchantment>> choices = resolveOptions(options).stream()
                .filter(holder -> inputStack.is(Items.BOOK)
                        || !onlyCompatible
                        || holder.value().canEnchant(inputStack))
                .toList();
        if (choices.isEmpty()) {
            return stack;
        }
        Holder<Enchantment> selected = choices.get(random.nextInt(choices.size()));
        Enchantment enchantment = selected.value();
        int level = nextInclusive(
                random, enchantment.getMinLevel(), enchantment.getMaxLevel()
        );
        if (stack.is(Items.BOOK)) {
            stack = new ItemStack(Items.ENCHANTED_BOOK);
        }
        stack.enchant(selected, level);
        return stack;
    }

    private HolderSet<Enchantment> resolveOptions(List<String> values) {
        var enchantments = registries.lookupOrThrow(Registries.ENCHANTMENT);
        if (values.size() != 1 || !values.getFirst().startsWith("#")) {
            return HolderSet.direct(values.stream()
                    .map(value -> ResourceKey.create(
                            Registries.ENCHANTMENT, Identifier.parse(value)
                    ))
                    .map(enchantments::getOrThrow)
                    .toList());
        }
        String value = values.getFirst();
        if (value.startsWith("#")) {
            TagKey<Enchantment> tag = TagKey.create(
                    Registries.ENCHANTMENT, Identifier.parse(value.substring(1))
            );
            return enchantments.getOrThrow(tag);
        }
        throw new IllegalStateException("Unreachable enchantment option: " + value);
    }

    private static Entry select(List<Entry> entries, RandomSource random) {
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

    private Table load(String lootTable) {
        Identifier id = Identifier.parse(lootTable);
        String path = "/data/" + id.getNamespace() + "/loot_table/" + id.getPath() + ".json";
        try (var stream = JsonLootTableOracle26_1_2.class.getResourceAsStream(path)) {
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
                        throw new IllegalStateException(
                                "Quality entries are unsupported: " + lootTable
                        );
                    }
                    Double entryChance = parseRandomChance(entry, lootTable);
                    List<Function> functions = new ArrayList<>();
                    if (entry.has("functions")) {
                        for (JsonElement functionElement : entry.getAsJsonArray("functions")) {
                            functions.add(parseFunction(functionElement.getAsJsonObject()));
                        }
                    }
                    if (type.equals("minecraft:loot_table") && !functions.isEmpty()) {
                        throw new IllegalStateException(
                                "Functions on nested loot tables are unsupported: " + lootTable
                        );
                    }
                    entries.add(new Entry(
                            type.equals("minecraft:item") ? entry.get("name").getAsString() : null,
                            type.equals("minecraft:loot_table") ? entry.get("value").getAsString() : null,
                            entry.has("weight") ? entry.get("weight").getAsInt() : 1,
                            List.copyOf(functions),
                            entryChance
                    ));
                }
                List<Function> poolFunctions = new ArrayList<>();
                if (pool.has("functions")) {
                    for (JsonElement functionElement : pool.getAsJsonArray("functions")) {
                        poolFunctions.add(parseFunction(functionElement.getAsJsonObject()));
                    }
                }
                pools.add(new Pool(
                        parseNumber(pool.get("rolls")),
                        List.copyOf(entries),
                        List.copyOf(poolFunctions),
                        parseRandomChance(pool, lootTable)
                ));
            }
            return new Table(List.copyOf(pools));
        } catch (IOException exception) {
            throw new UncheckedIOException("Could not read " + lootTable, exception);
        }
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
                    List.of(json.get("options").getAsString()),
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

    private static int nextInclusive(RandomSource random, int min, int max) {
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
        private int nextInt(RandomSource random) {
            int integerMin = (int) min;
            int integerMax = (int) max;
            return nextInclusive(random, integerMin, integerMax);
        }

        private void consumeFloat(RandomSource random) {
            if (min < max) {
                random.nextFloat();
            }
        }
    }
}
