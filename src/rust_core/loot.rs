use std::collections::{HashMap, HashSet};
use std::sync::LazyLock;

use serde_json::Value;

use super::legacy_random::LegacyRandom48;

const LOOT_RUNTIME: &str = include_str!("../main/resources/mclootfinder/26.1.2/loot-runtime.json");
const LOOT_TABLES: &str = include_str!("../main/resources/mclootfinder/26.1.2/loot-tables.json");

static DATA: LazyLock<Result<LootData, String>> = LazyLock::new(load_data);
static TABLES: LazyLock<Result<HashMap<String, Table>, String>> = LazyLock::new(load_tables);

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LootStack {
    pub item: String,
    pub count: i32,
}

pub fn roll(loot_table: &str, seed: i64) -> Result<Vec<LootStack>, String> {
    if seed == 0 {
        return Err("LootTableSeed 0 is vanilla's unseeded sentinel".to_owned());
    }
    let data = DATA.as_ref().map_err(Clone::clone)?;
    let tables = TABLES.as_ref().map_err(Clone::clone)?;
    let mut random = LegacyRandom48::new(seed);
    let mut result = Vec::new();
    roll_into(loot_table, tables, data, &mut random, &mut result)?;
    Ok(result)
}

fn roll_into(
    table_id: &str,
    tables: &HashMap<String, Table>,
    data: &LootData,
    random: &mut LegacyRandom48,
    result: &mut Vec<LootStack>,
) -> Result<(), String> {
    let table = tables
        .get(table_id)
        .ok_or_else(|| format!("vanilla loot table not found: {table_id}"))?;
    for pool in &table.pools {
        if pool
            .random_chance
            .is_some_and(|chance| f64::from(random.next_float()) >= chance)
        {
            continue;
        }
        let rolls = pool.rolls.next_int(random);
        for _ in 0..rolls {
            let entry = select(&pool.entries, random)?;
            if entry
                .random_chance
                .is_some_and(|chance| f64::from(random.next_float()) >= chance)
            {
                continue;
            }
            if let Some(nested) = &entry.nested_table {
                roll_into(nested, tables, data, random, result)?;
                continue;
            }
            let Some(item) = &entry.item else {
                continue;
            };
            let mut stack = MutableStack {
                item: item.clone(),
                count: 1,
            };
            for function in &entry.functions {
                apply(function, &mut stack, data, random)?;
            }
            for function in &pool.functions {
                apply(function, &mut stack, data, random)?;
            }
            result.push(LootStack {
                item: stack.item,
                count: stack.count,
            });
        }
    }
    Ok(())
}

fn apply(
    function: &Function,
    stack: &mut MutableStack,
    data: &LootData,
    random: &mut LegacyRandom48,
) -> Result<(), String> {
    match function.kind.as_str() {
        "minecraft:set_count" => {
            stack.count = function.number()?.next_int(random);
        }
        "minecraft:set_damage" => {
            if data.item(&stack.item)?.damageable {
                function.number()?.consume_float(random);
            }
        }
        "minecraft:enchant_randomly" => enchant_randomly(
            stack,
            random,
            data,
            function.options.as_deref().unwrap_or_default(),
            function.only_compatible,
        )?,
        "minecraft:enchant_with_levels" => {
            let level = function.number()?.next_int(random);
            enchant_with_levels(
                stack,
                random,
                data,
                level,
                function.options.as_deref().unwrap_or_default(),
            )?;
        }
        "minecraft:set_potion" | "minecraft:exploration_map" | "minecraft:set_name" => {}
        "minecraft:set_ominous_bottle_amplifier" => {
            function.number()?.next_int(random);
        }
        "minecraft:set_instrument" => {
            if data.goat_horns != 0 {
                random.next_int(data.goat_horns);
            }
        }
        "minecraft:set_stew_effect" => {
            let selected = random.next_int(function.alternatives.len() as i32) as usize;
            function.alternatives[selected].consume_float(random);
        }
        kind => return Err(format!("unsupported 26.1.2 loot function: {kind}")),
    }
    Ok(())
}

fn enchant_randomly(
    stack: &mut MutableStack,
    random: &mut LegacyRandom48,
    data: &LootData,
    options: &[String],
    only_compatible: bool,
) -> Result<(), String> {
    let choices = enchantments(data, options)?
        .into_iter()
        .filter(|enchantment| {
            stack.item == "minecraft:book"
                || !only_compatible
                || enchantment.compatible_items.contains(&stack.item)
        })
        .collect::<Vec<_>>();
    if choices.is_empty() {
        return Ok(());
    }
    let selected = choices[random.next_int(choices.len() as i32) as usize];
    next_inclusive(random, 1, selected.max_level);
    if stack.item == "minecraft:book" {
        stack.item = "minecraft:enchanted_book".to_owned();
    }
    Ok(())
}

fn enchant_with_levels(
    stack: &mut MutableStack,
    random: &mut LegacyRandom48,
    data: &LootData,
    mut level: i32,
    options: &[String],
) -> Result<(), String> {
    let item = data.item(&stack.item)?;
    if item.enchantability == 0 {
        return Ok(());
    }

    let spread = item.enchantability / 4 + 1;
    level += 1 + random.next_int(spread) + random.next_int(spread);
    let variance = (random.next_float() + random.next_float() - 1.0) * 0.15;
    level = java_round(level as f32 + level as f32 * variance).max(1);

    let mut available = available_enchantments(&stack.item, level, enchantments(data, options)?);
    if available.is_empty() {
        return Ok(());
    }

    let mut selected = Vec::new();
    choose_weighted(&available, random, &mut selected)?;
    while random.next_int(50) <= level {
        if let Some(last) = selected.last() {
            available.retain(|candidate| {
                compatible(data, &candidate.enchantment.id, &last.enchantment.id)
            });
        }
        if available.is_empty() {
            break;
        }
        choose_weighted(&available, random, &mut selected)?;
        level /= 2;
    }

    if stack.item == "minecraft:book" {
        stack.item = "minecraft:enchanted_book".to_owned();
    }
    Ok(())
}

fn available_enchantments<'a>(
    item: &str,
    level: i32,
    enchantments: Vec<&'a EnchantmentSpec>,
) -> Vec<AvailableEnchantment<'a>> {
    let book = item == "minecraft:book";
    let mut available = Vec::new();
    for enchantment in enchantments {
        if !book && !enchantment.primary_items.contains(item) {
            continue;
        }
        for enchantment_level in (1..=enchantment.max_level).rev() {
            if level >= enchantment.min_cost(enchantment_level)
                && level <= enchantment.max_cost(enchantment_level)
            {
                available.push(AvailableEnchantment {
                    enchantment,
                    _level: enchantment_level,
                });
                break;
            }
        }
    }
    available
}

fn choose_weighted<'a>(
    available: &[AvailableEnchantment<'a>],
    random: &mut LegacyRandom48,
    selected: &mut Vec<AvailableEnchantment<'a>>,
) -> Result<(), String> {
    let total_weight = available.iter().map(|value| value.enchantment.weight).sum();
    let mut choice = random.next_int(total_weight);
    for value in available {
        choice -= value.enchantment.weight;
        if choice < 0 {
            selected.push(*value);
            return Ok(());
        }
    }
    Err("enchantment weights are inconsistent".to_owned())
}

fn compatible(data: &LootData, first: &str, second: &str) -> bool {
    if first == second {
        return false;
    }
    let Some(first) = data.enchantments.get(first) else {
        return false;
    };
    let Some(second_spec) = data.enchantments.get(second) else {
        return false;
    };
    !first.exclusive.contains(second) && !second_spec.exclusive.contains(&first.id)
}

fn enchantments<'a>(
    data: &'a LootData,
    options: &[String],
) -> Result<Vec<&'a EnchantmentSpec>, String> {
    let ids = if options == ["#minecraft:on_random_loot"] {
        &data.on_random_loot
    } else {
        options
    };
    ids.iter()
        .map(|id| {
            data.enchantments
                .get(id)
                .ok_or_else(|| format!("unknown 26.1.2 enchantment: {id}"))
        })
        .collect()
}

fn select<'a>(entries: &'a [Entry], random: &mut LegacyRandom48) -> Result<&'a Entry, String> {
    if entries.len() == 1 {
        return Ok(&entries[0]);
    }
    let total_weight = entries.iter().map(|entry| entry.weight).sum();
    let mut choice = random.next_int(total_weight);
    for entry in entries {
        choice -= entry.weight;
        if choice < 0 {
            return Ok(entry);
        }
    }
    Err("loot entry weights are inconsistent".to_owned())
}

fn load_tables() -> Result<HashMap<String, Table>, String> {
    let roots: HashMap<String, Value> =
        serde_json::from_str(LOOT_TABLES).map_err(|error| error.to_string())?;
    roots
        .into_iter()
        .map(|(id, value)| parse_table(&id, &value).map(|table| (id, table)))
        .collect()
}

fn parse_table(id: &str, root: &Value) -> Result<Table, String> {
    if root.get("functions").is_some() {
        return Err(format!("table-level functions are unsupported: {id}"));
    }
    let pools = array(root, "pools")?
        .iter()
        .map(|pool| {
            let entries = array(pool, "entries")?
                .iter()
                .map(|entry| parse_entry(id, entry))
                .collect::<Result<Vec<_>, _>>()?;
            Ok(Pool {
                rolls: parse_number(required(pool, "rolls")?)?,
                entries,
                functions: parse_functions(pool)?,
                random_chance: parse_random_chance(id, pool)?,
            })
        })
        .collect::<Result<Vec<_>, String>>()?;
    Ok(Table { pools })
}

fn parse_entry(table: &str, entry: &Value) -> Result<Entry, String> {
    let kind = string(entry, "type")?;
    if !matches!(
        kind,
        "minecraft:item" | "minecraft:empty" | "minecraft:loot_table"
    ) {
        return Err(format!("unsupported loot entry in {table}: {kind}"));
    }
    if entry.get("quality").is_some() {
        return Err(format!("quality entries are unsupported: {table}"));
    }
    let functions = parse_functions(entry)?;
    if kind == "minecraft:loot_table" && !functions.is_empty() {
        return Err(format!(
            "functions on nested loot tables are unsupported: {table}"
        ));
    }
    Ok(Entry {
        item: (kind == "minecraft:item")
            .then(|| string(entry, "name").map(str::to_owned))
            .transpose()?,
        nested_table: (kind == "minecraft:loot_table")
            .then(|| string(entry, "value").map(str::to_owned))
            .transpose()?,
        weight: entry.get("weight").and_then(Value::as_i64).unwrap_or(1) as i32,
        functions,
        random_chance: parse_random_chance(table, entry)?,
    })
}

fn parse_functions(owner: &Value) -> Result<Vec<Function>, String> {
    let Some(functions) = owner.get("functions") else {
        return Ok(Vec::new());
    };
    functions
        .as_array()
        .ok_or_else(|| "loot functions must be an array".to_owned())?
        .iter()
        .map(parse_function)
        .collect()
}

fn parse_function(value: &Value) -> Result<Function, String> {
    let kind = string(value, "function")?.to_owned();
    let mut function = Function {
        kind: kind.clone(),
        number: None,
        options: None,
        only_compatible: true,
        alternatives: Vec::new(),
    };
    match kind.as_str() {
        "minecraft:set_count" => function.number = Some(parse_number(required(value, "count")?)?),
        "minecraft:set_damage" => function.number = Some(parse_number(required(value, "damage")?)?),
        "minecraft:enchant_randomly" => {
            function.options = Some(parse_options(required(value, "options")?)?);
            function.only_compatible = value
                .get("only_compatible")
                .and_then(Value::as_bool)
                .unwrap_or(true);
        }
        "minecraft:enchant_with_levels" => {
            function.number = Some(parse_number(required(value, "levels")?)?);
            function.options = Some(parse_options(required(value, "options")?)?);
        }
        "minecraft:set_potion" | "minecraft:exploration_map" | "minecraft:set_name" => {}
        "minecraft:set_ominous_bottle_amplifier" => {
            function.number = Some(parse_number(required(value, "amplifier")?)?);
        }
        "minecraft:set_instrument" => {
            function.options = Some(vec![string(value, "options")?.to_owned()]);
        }
        "minecraft:set_stew_effect" => {
            function.alternatives = array(value, "effects")?
                .iter()
                .map(|effect| parse_number(required(effect, "duration")?))
                .collect::<Result<Vec<_>, _>>()?;
        }
        _ => return Err(format!("unsupported loot function: {kind}")),
    }
    Ok(function)
}

fn parse_number(value: &Value) -> Result<NumberSpec, String> {
    if let Some(number) = value.as_f64() {
        let number = number as f32;
        return Ok(NumberSpec {
            min: number,
            max: number,
        });
    }
    if string(value, "type")? != "minecraft:uniform" {
        return Err(format!("unsupported number provider: {value}"));
    }
    Ok(NumberSpec {
        min: number(value, "min")? as f32,
        max: number(value, "max")? as f32,
    })
}

fn parse_options(value: &Value) -> Result<Vec<String>, String> {
    if let Some(option) = value.as_str() {
        return Ok(vec![option.to_owned()]);
    }
    let options = value
        .as_array()
        .ok_or_else(|| "enchantment options must be a string or array".to_owned())?
        .iter()
        .map(|option| {
            option
                .as_str()
                .map(str::to_owned)
                .ok_or_else(|| "enchantment option must be a string".to_owned())
        })
        .collect::<Result<Vec<_>, _>>()?;
    if options.is_empty() {
        return Err("enchantment options must not be empty".to_owned());
    }
    Ok(options)
}

fn parse_random_chance(table: &str, value: &Value) -> Result<Option<f64>, String> {
    let Some(conditions) = value.get("conditions") else {
        return Ok(None);
    };
    let conditions = conditions
        .as_array()
        .ok_or_else(|| format!("unsupported loot condition: {table}"))?;
    if conditions.len() != 1 || string(&conditions[0], "condition")? != "minecraft:random_chance" {
        return Err(format!("unsupported loot condition: {table}"));
    }
    Ok(Some(number(&conditions[0], "chance")?))
}

fn load_data() -> Result<LootData, String> {
    let root: Value = serde_json::from_str(LOOT_RUNTIME).map_err(|error| error.to_string())?;
    let enchantments = array(&root, "enchantments")?
        .iter()
        .map(|value| {
            let enchantment = EnchantmentSpec {
                id: string(value, "id")?.to_owned(),
                weight: integer(value, "weight")?,
                max_level: integer(value, "max_level")?,
                min_cost_base: integer(value, "min_cost_base")?,
                min_cost_per_level: integer(value, "min_cost_per_level")?,
                max_cost_base: integer(value, "max_cost_base")?,
                max_cost_per_level: integer(value, "max_cost_per_level")?,
                primary_items: string_set(required(value, "primary_items")?)?,
                compatible_items: string_set(required(value, "compatible_items")?)?,
                exclusive: string_set(required(value, "exclusive")?)?,
            };
            Ok((enchantment.id.clone(), enchantment))
        })
        .collect::<Result<HashMap<_, _>, String>>()?;
    let items = required(&root, "items")?
        .as_object()
        .ok_or_else(|| "items must be an object".to_owned())?
        .iter()
        .map(|(id, value)| {
            Ok((
                id.clone(),
                ItemSpec {
                    enchantability: integer(value, "enchantability")?,
                    damageable: boolean(value, "damageable")?,
                },
            ))
        })
        .collect::<Result<HashMap<_, _>, String>>()?;
    Ok(LootData {
        enchantments,
        on_random_loot: string_vec(required(&root, "on_random_loot")?)?,
        items,
        goat_horns: integer(&root, "goat_horns")?,
    })
}

fn required<'a>(value: &'a Value, key: &str) -> Result<&'a Value, String> {
    value.get(key).ok_or_else(|| format!("missing {key}"))
}

fn array<'a>(value: &'a Value, key: &str) -> Result<&'a [Value], String> {
    required(value, key)?
        .as_array()
        .map(Vec::as_slice)
        .ok_or_else(|| format!("{key} must be an array"))
}

fn string<'a>(value: &'a Value, key: &str) -> Result<&'a str, String> {
    required(value, key)?
        .as_str()
        .ok_or_else(|| format!("{key} must be a string"))
}

fn number(value: &Value, key: &str) -> Result<f64, String> {
    required(value, key)?
        .as_f64()
        .ok_or_else(|| format!("{key} must be a number"))
}

fn integer(value: &Value, key: &str) -> Result<i32, String> {
    required(value, key)?
        .as_i64()
        .map(|number| number as i32)
        .ok_or_else(|| format!("{key} must be an integer"))
}

fn boolean(value: &Value, key: &str) -> Result<bool, String> {
    required(value, key)?
        .as_bool()
        .ok_or_else(|| format!("{key} must be a boolean"))
}

fn string_vec(value: &Value) -> Result<Vec<String>, String> {
    value
        .as_array()
        .ok_or_else(|| "value must be an array".to_owned())?
        .iter()
        .map(|value| {
            value
                .as_str()
                .map(str::to_owned)
                .ok_or_else(|| "array value must be a string".to_owned())
        })
        .collect()
}

fn string_set(value: &Value) -> Result<HashSet<String>, String> {
    Ok(string_vec(value)?.into_iter().collect())
}

fn next_inclusive(random: &mut LegacyRandom48, min: i32, max: i32) -> i32 {
    if min >= max {
        min
    } else {
        random.next_int(max - min + 1) + min
    }
}

fn java_round(value: f32) -> i32 {
    (value + 0.5).floor() as i32
}

struct Table {
    pools: Vec<Pool>,
}

struct Pool {
    rolls: NumberSpec,
    entries: Vec<Entry>,
    functions: Vec<Function>,
    random_chance: Option<f64>,
}

struct Entry {
    item: Option<String>,
    nested_table: Option<String>,
    weight: i32,
    functions: Vec<Function>,
    random_chance: Option<f64>,
}

struct Function {
    kind: String,
    number: Option<NumberSpec>,
    options: Option<Vec<String>>,
    only_compatible: bool,
    alternatives: Vec<NumberSpec>,
}

impl Function {
    fn number(&self) -> Result<NumberSpec, String> {
        self.number
            .ok_or_else(|| format!("{} requires a number", self.kind))
    }
}

#[derive(Clone, Copy)]
struct NumberSpec {
    min: f32,
    max: f32,
}

impl NumberSpec {
    fn next_int(self, random: &mut LegacyRandom48) -> i32 {
        next_inclusive(random, self.min as i32, self.max as i32)
    }

    fn consume_float(self, random: &mut LegacyRandom48) {
        if self.min < self.max {
            random.next_float();
        }
    }
}

struct EnchantmentSpec {
    id: String,
    weight: i32,
    max_level: i32,
    min_cost_base: i32,
    min_cost_per_level: i32,
    max_cost_base: i32,
    max_cost_per_level: i32,
    primary_items: HashSet<String>,
    compatible_items: HashSet<String>,
    exclusive: HashSet<String>,
}

impl EnchantmentSpec {
    fn min_cost(&self, level: i32) -> i32 {
        self.min_cost_base + self.min_cost_per_level * (level - 1)
    }

    fn max_cost(&self, level: i32) -> i32 {
        self.max_cost_base + self.max_cost_per_level * (level - 1)
    }
}

struct ItemSpec {
    enchantability: i32,
    damageable: bool,
}

#[derive(Clone, Copy)]
struct AvailableEnchantment<'a> {
    enchantment: &'a EnchantmentSpec,
    _level: i32,
}

struct LootData {
    enchantments: HashMap<String, EnchantmentSpec>,
    on_random_loot: Vec<String>,
    items: HashMap<String, ItemSpec>,
    goat_horns: i32,
}

impl LootData {
    fn item(&self, id: &str) -> Result<&ItemSpec, String> {
        self.items
            .get(id)
            .ok_or_else(|| format!("unknown 26.1.2 item: {id}"))
    }
}

struct MutableStack {
    item: String,
    count: i32,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rolls_known_ancient_city_seed() {
        let stacks = roll("minecraft:chests/ancient_city", 1).unwrap();
        assert_eq!(stacks[0].item, "minecraft:sculk_catalyst");
        assert!(
            stacks
                .iter()
                .any(|stack| stack.item == "minecraft:enchanted_golden_apple")
        );
    }

    #[test]
    fn rejects_zero_seed() {
        assert!(roll("minecraft:chests/ancient_city", 0).is_err());
    }
}
