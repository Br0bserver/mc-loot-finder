use crate::catalog::candidate_structure;
use crate::cli::{SearchArgs, require_version};
use crate::commands::locate_and_scan;
use crate::output::{grouped, quantity};

pub fn run(args: SearchArgs) -> Result<u8, String> {
    require_version(&args.common.version)?;
    let structure = candidate_structure(&args.structure)?;
    let world_seed = args.seed;
    let center_x = args.center_x;
    let center_z = args.center_z;
    let radius = args.radius.unwrap_or(2_000);
    let limit = args.limit.unwrap_or(100);

    let (candidates, scans) = locate_and_scan(world_seed, structure, center_x, center_z, radius)?;
    let mut valid_structures = 0;
    let mut containers = Vec::new();
    for scan in scans {
        if scan.valid_structure {
            valid_structures += 1;
            containers.extend(scan.chests);
        }
    }
    containers.retain(|chest| !chest.loot_table.is_empty());

    if args.common.json {
        print!(
            "{{\"version\":\"26.1.2\",\"structure\":\"{}\",\"seed\":{},\"placement_candidates\":{},\"valid_structures\":{},\"chest_count\":{},\"chests\":[",
            structure.name,
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
        "Structure: {}\nSearch area: {} blocks around ({center_x}, {center_z})\n",
        structure.name,
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
