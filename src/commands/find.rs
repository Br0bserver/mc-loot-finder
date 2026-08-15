use crate::catalog::candidate_structure;
use crate::cli::{FindArgs, require_identifier, require_version};
use crate::commands::locate_and_scan;
use crate::loot;
use crate::output::{FindMatch, FindOutput, grouped, print_json, quantity};

pub fn run(args: FindArgs) -> Result<u8, String> {
    require_version(&args.search.common.version)?;
    let structure = candidate_structure(&args.search.structure)?;
    let world_seed = args.search.seed;
    let item = args.item.as_deref().unwrap_or(structure.default_item);
    require_identifier(item, "--item")?;
    let center_x = args.search.center_x;
    let center_z = args.search.center_z;
    let radius = args.search.radius.unwrap_or(5_000);
    let limit = args.search.limit.unwrap_or(20);

    let (candidates, scans) = locate_and_scan(world_seed, structure, center_x, center_z, radius)?;
    let mut valid_structures = 0;
    let mut checked_chests = 0;
    let mut unpredictable_zero_seeds = 0;
    let mut matches = Vec::new();
    for scan in scans {
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

    if args.search.common.json {
        let output = FindOutput {
            version: "26.1.2",
            structure: structure.name,
            seed: world_seed,
            item: item.to_owned(),
            placement_candidates: candidates.len(),
            valid_structures,
            checked_chests,
            hits: matches.len(),
            unpredictable_zero_seeds,
            matches: matches
                .iter()
                .take(limit as usize)
                .map(|(chest, item_count)| FindMatch {
                    x: chest.x,
                    y: chest.y,
                    z: chest.z,
                    item_count: *item_count,
                    loot_table: chest.loot_table.clone(),
                    loot_seed: chest.loot_seed,
                    start_chunk_x: chest.structure_chunk_x,
                    start_chunk_z: chest.structure_chunk_z,
                })
                .collect(),
        };
        print_json(&output);
        return Ok(exit_code);
    }

    println!("Minecraft Java 26.1.2");
    println!("World seed: {world_seed}");
    println!("Structure: {}", structure.name);
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
        quantity(valid_structures as i64, "valid structure"),
        quantity(checked_chests as i64, "container")
    );
    println!(
        "Shown: {} of {}",
        grouped(shown as i64),
        quantity(matches.len() as i64, "match")
    );
    if unpredictable_zero_seeds != 0 {
        println!(
            "Skipped: {} with LootTableSeed 0",
            quantity(unpredictable_zero_seeds as i64, "container")
        );
    }
    Ok(exit_code)
}
