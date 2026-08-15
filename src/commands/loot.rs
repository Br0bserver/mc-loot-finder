use crate::catalog::CANDIDATE_STRUCTURES;
use crate::cli::{LootArgs, require_identifier, require_version};
use crate::error::Error;
use crate::loot;
use crate::output::{LootOutput, LootStackJson, print_json, quantity};

pub fn run(args: LootArgs) -> Result<u8, Error> {
    require_version(&args.common.version)?;
    let table = args.table;
    require_identifier(&table, "--table")?;
    let supported = CANDIDATE_STRUCTURES
        .iter()
        .flat_map(|structure| structure.loot_tables.iter())
        .any(|supported| *supported == table);
    if !supported {
        return Err(Error::Usage(format!(
            "unsupported loot table: {table}; use one listed by 'explain'"
        )));
    }
    let seed = args.loot_seed;
    if !args.common.json {
        println!("Minecraft Java 26.1.2");
        println!("Loot table: {table}");
        println!("Loot seed: {seed}\n");
        println!("Generating...\n");
    }
    let stacks = loot::roll(&table, seed)?;
    if args.common.json {
        let output = LootOutput {
            version: "26.1.2",
            loot_table: table.clone(),
            loot_seed: seed,
            items: stacks
                .iter()
                .map(|stack| LootStackJson {
                    item: stack.item.clone(),
                    count: stack.count,
                })
                .collect(),
        };
        print_json(&output);
    } else {
        println!("Generated {}\n", quantity(stacks.len() as i64, "stack"));
        for (index, stack) in stacks.iter().enumerate() {
            println!("[{}] {} x{}", index + 1, stack.item, stack.count);
        }
    }
    Ok(0)
}
