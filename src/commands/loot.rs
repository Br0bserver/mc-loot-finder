use crate::catalog::CANDIDATE_STRUCTURES;
use crate::cli::{LootArgs, require_identifier, require_version};
use crate::loot;
use crate::output::quantity;

pub fn run(args: LootArgs) -> Result<u8, String> {
    require_version(&args.common.version)?;
    let table = args.table;
    require_identifier(&table, "--table")?;
    let supported = CANDIDATE_STRUCTURES
        .iter()
        .flat_map(|structure| structure.loot_tables.iter())
        .any(|supported| *supported == table);
    if !supported {
        return Err(format!(
            "unsupported loot table: {table}; use one listed by 'explain'"
        ));
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
