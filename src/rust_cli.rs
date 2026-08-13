use std::collections::HashMap;

use crate::rust_core::ancient_city;
use crate::rust_core::candidate_structure;
use crate::rust_core::candidates::locate;
use crate::rust_core::decoration_random::container_loot_seed;
use crate::rust_core::loot;
use crate::rust_core::{CANDIDATE_STRUCTURES, ContainerSeedShortcut, SpreadType};

pub fn run(arguments: impl IntoIterator<Item = String>) -> Result<u8, String> {
    let arguments = arguments.into_iter().collect::<Vec<_>>();
    let Some(command) = arguments.first().map(String::as_str) else {
        print_help();
        return Ok(0);
    };

    if command == "help" || command == "--help" {
        print_help();
        return Ok(0);
    }

    let options = Options::parse(&arguments[1..])?;
    match command {
        "candidates" => candidates(&options),
        "chests" => chests(&options),
        "find" => find(&options),
        "container-seed" => container_seed(&options),
        "explain" => explain(&options),
        "loot" => loot_command(&options),
        _ => Err(format!(
            "command '{command}' has not been migrated to the Rust CLI yet"
        )),
    }
}

fn find(options: &Options) -> Result<u8, String> {
    require_version(options)?;
    let structure = candidate_structure(options.text("structure", "ancient_city"))?;
    if structure.name != "ancient_city" {
        return Err(format!(
            "Rust find currently supports only ancient_city; '{}' is not available yet",
            structure.name
        ));
    }
    let world_seed = options.required_i64("seed")?;
    let item = options.text("item", structure.default_item);
    require_identifier(item, "--item")?;
    let center_x = options.i32("center-x", 0)?;
    let center_z = options.i32("center-z", 0)?;
    let radius = options.i32("radius", 5_000)?;
    let limit = options.i32("limit", 20)?;
    if limit < 0 {
        return Err("--limit must be non-negative".to_owned());
    }

    let candidates = locate(world_seed, center_x, center_z, radius, structure.placement)?;
    let scanner = ancient_city::Scanner::new(world_seed);
    let mut valid_structures = 0;
    let mut checked_chests = 0;
    let mut unpredictable_zero_seeds = 0;
    let mut matches = Vec::new();
    for candidate in &candidates {
        let scan = scanner.scan(candidate.chunk_x, candidate.chunk_z)?;
        if !scan.valid_structure {
            continue;
        }
        valid_structures += 1;
        for chest in scan.chests {
            if !structure.loot_tables.contains(&chest.loot_table.as_str()) {
                continue;
            }
            checked_chests += 1;
            if chest.loot_seed == 0 {
                unpredictable_zero_seeds += 1;
                continue;
            }
            let item_count = loot::roll(&chest.loot_table, chest.loot_seed)?
                .into_iter()
                .filter(|stack| stack.item == item)
                .map(|stack| stack.count)
                .sum::<i32>();
            if item_count > 0 {
                matches.push((chest, item_count));
            }
        }
    }
    let exit_code = if matches.is_empty() { 1 } else { 0 };

    if options.flag("json") {
        print!(
            "{{\"version\":\"26.1.2\",\"structure\":\"ancient_city\",\"seed\":{},\"item\":\"{}\",\"placement_candidates\":{},\"valid_structures\":{},\"checked_chests\":{},\"hits\":{},\"unpredictable_zero_seeds\":{},\"matches\":[",
            world_seed,
            item,
            candidates.len(),
            valid_structures,
            checked_chests,
            matches.len(),
            unpredictable_zero_seeds
        );
        for (index, (chest, item_count)) in matches.iter().take(limit as usize).enumerate() {
            if index != 0 {
                print!(",");
            }
            print!(
                "{{\"x\":{},\"y\":{},\"z\":{},\"item_count\":{},\"loot_table\":\"{}\",\"loot_seed\":{},\"start_chunk_x\":{},\"start_chunk_z\":{}}}",
                chest.x,
                chest.y,
                chest.z,
                item_count,
                chest.loot_table,
                chest.loot_seed,
                chest.structure_chunk_x,
                chest.structure_chunk_z
            );
        }
        println!("]}}");
        return Ok(exit_code);
    }

    println!("Minecraft Java 26.1.2");
    println!("World seed: {world_seed}");
    println!("Structure: ancient_city");
    println!("Item: {item}");
    println!(
        "Search area: {} blocks around ({center_x}, {center_z})\n",
        grouped(i64::from(radius))
    );
    println!("Found {}\n", quantity(matches.len() as i64, "chest"));
    for (index, (chest, item_count)) in matches.iter().take(limit as usize).enumerate() {
        println!("[{}]", index + 1);
        println!("  Position: ({}, {}, {})", chest.x, chest.y, chest.z);
        println!("  Item count: {item_count}");
        println!("  Loot table: {}", chest.loot_table);
        println!("  Loot seed: {}", chest.loot_seed);
        println!(
            "  Start chunk: ({}, {})\n",
            chest.structure_chunk_x, chest.structure_chunk_z
        );
    }
    let shown = matches.len().min(limit as usize);
    println!(
        "Checked: {}, {}, {}",
        quantity(candidates.len() as i64, "candidate"),
        quantity(valid_structures, "valid structure"),
        quantity(checked_chests, "container")
    );
    println!(
        "Shown: {} of {}",
        grouped(shown as i64),
        quantity(matches.len() as i64, "match")
    );
    if unpredictable_zero_seeds != 0 {
        println!(
            "Skipped: {} with LootTableSeed 0",
            quantity(unpredictable_zero_seeds, "container")
        );
    }
    Ok(exit_code)
}

fn chests(options: &Options) -> Result<u8, String> {
    require_version(options)?;
    let structure = candidate_structure(options.text("structure", "ancient_city"))?;
    if structure.name != "ancient_city" {
        return Err(format!(
            "Rust chests currently supports only ancient_city; '{}' still uses the Java CLI",
            structure.name
        ));
    }
    let world_seed = options.required_i64("seed")?;
    let center_x = options.i32("center-x", 0)?;
    let center_z = options.i32("center-z", 0)?;
    let radius = options.i32("radius", 2_000)?;
    let limit = options.i32("limit", 100)?;
    if limit < 0 {
        return Err("--limit must be non-negative".to_owned());
    }
    let candidates = locate(world_seed, center_x, center_z, radius, structure.placement)?;
    let scanner = ancient_city::Scanner::new(world_seed);
    let mut valid_structures = 0;
    let mut containers = Vec::new();
    for candidate in &candidates {
        let scan = scanner.scan(candidate.chunk_x, candidate.chunk_z)?;
        if scan.valid_structure {
            valid_structures += 1;
            containers.extend(scan.chests);
        }
    }
    containers.retain(|chest| !chest.loot_table.is_empty());

    if options.flag("json") {
        print!(
            "{{\"version\":\"26.1.2\",\"structure\":\"ancient_city\",\"seed\":{},\"placement_candidates\":{},\"valid_structures\":{},\"chest_count\":{},\"chests\":[",
            world_seed,
            candidates.len(),
            valid_structures,
            containers.len()
        );
        for (index, chest) in containers.iter().take(limit as usize).enumerate() {
            if index != 0 {
                print!(",");
            }
            print!(
                "{{\"x\":{},\"y\":{},\"z\":{},\"loot_table\":\"{}\",\"loot_seed\":{},\"start_chunk_x\":{},\"start_chunk_z\":{},\"ordinal\":{}}}",
                chest.x,
                chest.y,
                chest.z,
                chest.loot_table,
                chest.loot_seed,
                chest.structure_chunk_x,
                chest.structure_chunk_z,
                chest.ordinal
            );
        }
        println!("]}}\n");
        return Ok(0);
    }

    println!("Minecraft Java 26.1.2");
    println!("World seed: {world_seed}");
    println!(
        "Structure: ancient_city\nSearch area: {} blocks around ({center_x}, {center_z})\n",
        grouped(i64::from(radius))
    );
    println!("Found {}\n", quantity(containers.len() as i64, "container"));
    for (index, chest) in containers.iter().take(limit as usize).enumerate() {
        println!("[{}]", index + 1);
        println!("  Position: ({}, {}, {})", chest.x, chest.y, chest.z);
        println!(
            "  Start chunk: ({}, {})",
            chest.structure_chunk_x, chest.structure_chunk_z
        );
        println!("  Loot table: {}", chest.loot_table);
        println!("  Loot seed: {}", chest.loot_seed);
        println!("  Ordinal: {}\n", chest.ordinal);
    }
    let shown = containers.len().min(limit as usize);
    println!(
        "Checked: {}, {}",
        quantity(candidates.len() as i64, "candidate"),
        quantity(valid_structures, "valid structure")
    );
    println!(
        "Shown: {} of {}",
        grouped(shown as i64),
        quantity(containers.len() as i64, "container")
    );
    Ok(0)
}

fn loot_command(options: &Options) -> Result<u8, String> {
    require_version(options)?;
    let table = options.text("table", "minecraft:chests/ancient_city");
    require_identifier(table, "--table")?;
    let supported = CANDIDATE_STRUCTURES
        .iter()
        .flat_map(|structure| structure.loot_tables.iter())
        .any(|supported| *supported == table);
    if !supported {
        return Err(format!(
            "unsupported loot table: {table}; use one listed by 'explain'"
        ));
    }
    let seed = options.required_i64("loot-seed")?;
    if !options.flag("json") {
        println!("Minecraft Java 26.1.2");
        println!("Loot table: {table}");
        println!("Loot seed: {seed}\n");
        println!("Generating...\n");
    }
    let stacks = loot::roll(table, seed)?;
    if options.flag("json") {
        print!(
            "{{\"version\":\"26.1.2\",\"loot_table\":\"{table}\",\"loot_seed\":{seed},\"items\":["
        );
        for (index, stack) in stacks.iter().enumerate() {
            if index != 0 {
                print!(",");
            }
            print!("{{\"item\":\"{}\",\"count\":{}}}", stack.item, stack.count);
        }
        println!("]}}");
    } else {
        println!("Generated {}\n", quantity(stacks.len() as i64, "stack"));
        for (index, stack) in stacks.iter().enumerate() {
            println!("[{}] {} x{}", index + 1, stack.item, stack.count);
        }
    }
    Ok(0)
}

fn require_identifier(value: &str, option: &str) -> Result<(), String> {
    let Some((namespace, path)) = value.split_once(':') else {
        return Err(format!("{option} must be a namespaced Minecraft id"));
    };
    let valid_namespace = !namespace.is_empty()
        && namespace.bytes().all(|byte| {
            byte.is_ascii_lowercase() || byte.is_ascii_digit() || b"_.-".contains(&byte)
        });
    let valid_path = !path.is_empty()
        && path.bytes().all(|byte| {
            byte.is_ascii_lowercase() || byte.is_ascii_digit() || b"_./-".contains(&byte)
        });
    if valid_namespace && valid_path {
        Ok(())
    } else {
        Err(format!("{option} must be a namespaced Minecraft id"))
    }
}

fn container_seed(options: &Options) -> Result<u8, String> {
    require_version(options)?;
    let structure = candidate_structure(options.text("structure", "ancient_city"))?;
    if structure.container_seed == ContainerSeedShortcut::None {
        return Err(format!(
            "container-seed is not available for {}; use 'chests' to execute vanilla placement",
            structure.name
        ));
    }
    let world_seed = options.required_i64("seed")?;
    let chunk_x = options.i32("chunk-x", 0)?;
    let chunk_z = options.i32("chunk-z", 0)?;
    let structure_index = options.i32("structure-index", structure.structure_index)?;
    let step = options.i32("step", structure.decoration_step)?;
    let ordinal = options.i32("ordinal", 0)?;
    let loot_table_seed = container_loot_seed(
        world_seed,
        chunk_x,
        chunk_z,
        structure_index,
        step,
        ordinal,
        structure.container_seed,
    )?;

    if options.flag("json") {
        println!(
            "{{\"version\":\"26.1.2\",\"structure\":\"{}\",\"world_seed\":{},\"chunk_x\":{},\"chunk_z\":{},\"structure_index\":{},\"step\":{},\"ordinal\":{},\"loot_table_seed\":{}}}",
            structure.name,
            world_seed,
            chunk_x,
            chunk_z,
            structure_index,
            step,
            ordinal,
            loot_table_seed
        );
    } else {
        println!("Minecraft Java 26.1.2");
        println!("Structure: {}", structure.name);
        println!("World seed: {world_seed}");
        println!("Decoration chunk: ({chunk_x}, {chunk_z})");
        println!("Container ordinal: {ordinal}\n");
        println!("LootTableSeed: {loot_table_seed}");
    }
    Ok(0)
}

fn print_help() {
    println!("mc-loot-finder");
    println!("Minecraft Java 26.1.2 structure container and loot finder");
    println!();
    println!("Commands:");
    println!();
    println!("  candidates --seed N [search options]");
    println!("    List possible structure chunks without full verification.");
    println!();
    println!("  chests --seed N [search options]");
    println!("    Verify structures and list their block containers.");
    println!();
    println!("  find --seed N [--item ID] [search options]");
    println!("    Find containers that generate the requested item.");
    println!();
    println!("  loot --loot-seed N [--table ID]");
    println!("    Replay one supported loot table.");
    println!();
    println!("  container-seed --seed N --chunk-x X --chunk-z Z [options]");
    println!("    Calculate a seed for supported shortcut structures.");
    println!();
    println!("  explain [--structure NAME]");
    println!("    Show defaults, supported structures, and loot tables.");
    println!();
    println!("Search options:");
    println!("  --structure NAME  --center-x X  --center-z Z");
    println!("  --radius N  --limit N");
    println!();
    println!("Common options:");
    println!("  --version 26.1.2  --json");
    println!();
    println!("Use 'explain' to list structure capabilities and defaults.");
}

fn explain(options: &Options) -> Result<u8, String> {
    require_version(options)?;
    let structure_name = options.text("structure", "");
    if structure_name.is_empty() {
        if options.flag("json") {
            print!("{{\"version\":\"26.1.2\",\"structures\":[");
            for (index, structure) in CANDIDATE_STRUCTURES.iter().enumerate() {
                if index != 0 {
                    print!(",");
                }
                print!(
                    "{{\"name\":\"{}\",\"dimension\":\"{}\",\"full_scan\":{},\"default_item\":\"{}\",\"loot_tables\":{}}}",
                    structure.name,
                    structure.dimension,
                    structure.name == "ancient_city",
                    structure.default_item,
                    structure.loot_tables.len()
                );
            }
            println!("]}}");
            return Ok(0);
        }
        println!("Minecraft Java 26.1.2\n");
        println!("Command defaults:");
        println!("  candidates: ancient_city, center (0, 0), radius 5,000, limit 100");
        println!("  chests: ancient_city, center (0, 0), radius 2,000, limit 100");
        println!("  find: ancient_city, center (0, 0), radius 5,000, limit 20");
        println!("  loot: minecraft:chests/ancient_city\n");
        println!("Structure capabilities:");
        println!("  Only ancient_city currently supports chests and find.");
        println!("  Other entries support candidate calculation only.");
        for (index, structure) in CANDIDATE_STRUCTURES.iter().enumerate() {
            println!("\n[{}] {}", index + 1, structure.name);
            println!("  Dimension: {}", structure.dimension);
            println!(
                "  Commands: {}",
                if structure.name == "ancient_city" {
                    "candidates, chests, find, loot"
                } else {
                    "candidates"
                }
            );
            println!("  Default item: {}", structure.default_item);
            println!(
                "  Loot tables: {}",
                grouped(structure.loot_tables.len() as i64)
            );
        }
        println!("\nUse 'explain --structure NAME' for details.");
        return Ok(0);
    }

    let structure = candidate_structure(structure_name)?;
    if options.flag("json") {
        let spread = match structure.placement.spread {
            SpreadType::Linear => "LINEAR",
            SpreadType::Triangular => "TRIANGULAR",
        };
        let shortcut = match structure.container_seed {
            ContainerSeedShortcut::Direct => "DIRECT",
            ContainerSeedShortcut::DesertPyramid => "DESERT_PYRAMID",
            ContainerSeedShortcut::None => "NONE",
        };
        print!(
            "{{\"version\":\"26.1.2\",\"name\":\"{}\",\"structure_id\":\"{}\",\"dimension\":\"{}\",\"full_scan\":{},\"default_item\":\"{}\",\"placement\":{{\"spacing\":{},\"separation\":{},\"salt\":{},\"spread\":\"{}\"}},\"decoration_step\":{},\"decoration_index\":{},\"scanner\":\"{}\",\"container_seed_shortcut\":\"{}\",\"loot_tables\":[",
            structure.name,
            structure.structure_id,
            structure.dimension,
            structure.name == "ancient_city",
            structure.default_item,
            structure.placement.spacing,
            structure.placement.separation,
            structure.placement.salt,
            spread,
            structure.decoration_step,
            structure.structure_index,
            structure.scanner,
            shortcut
        );
        for (index, table) in structure.loot_tables.iter().enumerate() {
            if index != 0 {
                print!(",");
            }
            print!("\"{table}\"");
        }
        println!("]}}");
        return Ok(0);
    }

    let spread = match structure.placement.spread {
        SpreadType::Linear => "LINEAR",
        SpreadType::Triangular => "TRIANGULAR",
    };
    let shortcut = match structure.container_seed {
        ContainerSeedShortcut::Direct => "DIRECT",
        ContainerSeedShortcut::DesertPyramid => "DESERT_PYRAMID",
        ContainerSeedShortcut::None => "NONE",
    };
    println!("Minecraft Java 26.1.2");
    println!("Structure: {}", structure.name);
    println!("Structure ID: {}", structure.structure_id);
    println!("Dimension: {}", structure.dimension);
    println!(
        "Commands: {}",
        if structure.name == "ancient_city" {
            "candidates, chests, find, loot"
        } else {
            "candidates only"
        }
    );
    println!("Default item: {}\n", structure.default_item);
    println!("Placement:");
    println!("  Spacing: {}", structure.placement.spacing);
    println!("  Separation: {}", structure.placement.separation);
    println!("  Salt: {}", structure.placement.salt);
    println!("  Spread: {spread}\n");
    println!("Container calculation:");
    println!("  Decoration step: {}", structure.decoration_step);
    println!("  Structure index: {}", structure.structure_index);
    println!("  Scanner: {}", structure.scanner);
    println!("  Seed shortcut: {shortcut}\n");
    println!("Loot tables:");
    for table in structure.loot_tables {
        println!("  {table}");
    }
    Ok(0)
}

fn candidates(options: &Options) -> Result<u8, String> {
    require_version(options)?;
    let structure = candidate_structure(options.text("structure", "ancient_city"))?;
    let seed = options.required_i64("seed")?;
    let center_x = options.i32("center-x", 0)?;
    let center_z = options.i32("center-z", 0)?;
    let radius = options.i32("radius", 5_000)?;
    let limit = options.i32("limit", 100)?;
    if limit < 0 {
        return Err("--limit must be non-negative".to_owned());
    }
    let candidates = locate(seed, center_x, center_z, radius, structure.placement)?;

    if options.flag("json") {
        print!(
            "{{\"version\":\"26.1.2\",\"structure\":\"{}\",\"seed\":{},\"status\":\"candidate_only\",\"candidates\":[",
            structure.name, seed
        );
        for (index, candidate) in candidates.iter().take(limit as usize).enumerate() {
            if index != 0 {
                print!(",");
            }
            print!(
                "{{\"chunk_x\":{},\"chunk_z\":{},\"block_x\":{},\"block_z\":{},\"distance\":{:.3}}}",
                candidate.chunk_x,
                candidate.chunk_z,
                candidate.block_x,
                candidate.block_z,
                (candidate.squared_distance as f64).sqrt()
            );
        }
        println!("]}}");
        return Ok(0);
    }

    println!("Minecraft Java 26.1.2");
    println!("World seed: {seed}");
    println!("Structure: {}", structure.name);
    println!(
        "Search area: {} blocks around ({center_x}, {center_z})\n",
        grouped(i64::from(radius))
    );
    println!(
        "Found {}\n",
        quantity(candidates.len() as i64, "placement candidate")
    );
    let shown = candidates.len().min(limit as usize);
    for (index, candidate) in candidates.iter().take(shown).enumerate() {
        println!("[{}]", index + 1);
        println!("  Chunk: ({}, {})", candidate.chunk_x, candidate.chunk_z);
        println!("  Center: ({}, {})", candidate.block_x, candidate.block_z);
        println!(
            "  Distance: {:.1} blocks\n",
            (candidate.squared_distance as f64).sqrt()
        );
    }
    println!("Candidates are not verified structures.");
    println!("Use 'chests' or 'find' to verify them.");
    println!(
        "Shown: {} of {}",
        grouped(shown as i64),
        quantity(candidates.len() as i64, "placement candidate")
    );
    Ok(0)
}

fn require_version(options: &Options) -> Result<(), String> {
    let version = options.text("version", "26.1.2");
    if version == "26.1.2" {
        Ok(())
    } else {
        Err(format!(
            "unsupported Minecraft version: {version}; supported: 26.1.2"
        ))
    }
}

fn grouped(value: i64) -> String {
    let negative = value < 0;
    let digits = value.unsigned_abs().to_string();
    let mut result = String::with_capacity(digits.len() + digits.len() / 3 + usize::from(negative));
    if negative {
        result.push('-');
    }
    for (index, character) in digits.chars().enumerate() {
        if index != 0 && (digits.len() - index).is_multiple_of(3) {
            result.push(',');
        }
        result.push(character);
    }
    result
}

fn quantity(count: i64, singular: &str) -> String {
    format!(
        "{} {singular}{}",
        grouped(count),
        if count == 1 { "" } else { "s" }
    )
}

#[derive(Debug, Default, Eq, PartialEq)]
struct Options {
    values: HashMap<String, String>,
}

impl Options {
    fn parse(arguments: &[String]) -> Result<Self, String> {
        let mut values = HashMap::new();
        let mut index = 0;
        while index < arguments.len() {
            let key = &arguments[index];
            let Some(name) = key.strip_prefix("--") else {
                return Err(format!("expected an option, got: {key}"));
            };
            if name.is_empty() {
                return Err("option name must not be empty".to_owned());
            }
            if index + 1 == arguments.len() || arguments[index + 1].starts_with("--") {
                values.insert(name.to_owned(), "true".to_owned());
                index += 1;
            } else {
                values.insert(name.to_owned(), arguments[index + 1].clone());
                index += 2;
            }
        }
        Ok(Self { values })
    }

    fn text<'a>(&'a self, key: &str, fallback: &'a str) -> &'a str {
        self.values.get(key).map_or(fallback, String::as_str)
    }

    fn required_i64(&self, key: &str) -> Result<i64, String> {
        let value = self
            .values
            .get(key)
            .ok_or_else(|| format!("missing required option --{key}"))?;
        value
            .parse()
            .map_err(|_| format!("--{key} must be a 64-bit integer"))
    }

    fn i32(&self, key: &str, fallback: i32) -> Result<i32, String> {
        let Some(value) = self.values.get(key) else {
            return Ok(fallback);
        };
        value
            .parse()
            .map_err(|_| format!("--{key} must be a 32-bit integer"))
    }

    fn flag(&self, key: &str) -> bool {
        self.values.get(key).is_some_and(|value| value == "true")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_values_and_flags() {
        let arguments = ["--seed".to_owned(), "-1".to_owned(), "--json".to_owned()];
        let options = Options::parse(&arguments).unwrap();

        assert_eq!(options.values.get("seed").map(String::as_str), Some("-1"));
        assert_eq!(options.values.get("json").map(String::as_str), Some("true"));
    }

    #[test]
    fn rejects_positional_arguments() {
        let error = Options::parse(&["seed".to_owned()]).unwrap_err();
        assert_eq!(error, "expected an option, got: seed");
    }

    #[test]
    fn formats_grouped_numbers() {
        assert_eq!(grouped(0), "0");
        assert_eq!(grouped(5_000), "5,000");
        assert_eq!(grouped(-1_234_567), "-1,234,567");
    }
}
