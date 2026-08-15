use crate::catalog::{
    CANDIDATE_STRUCTURES, ContainerSeedShortcut, SpreadType, candidate_structure,
};
use crate::cli::{ExplainArgs, require_version};
use crate::output::grouped;

pub fn run(args: ExplainArgs) -> Result<u8, String> {
    require_version(&args.common.version)?;
    let structure_name = args.structure.as_deref().unwrap_or("");
    if structure_name.is_empty() {
        if args.common.json {
            print!("{{\"version\":\"26.1.2\",\"structures\":[");
            for (index, structure) in CANDIDATE_STRUCTURES.iter().enumerate() {
                if index != 0 {
                    print!(",");
                }
                print!(
                    "{{\"name\":\"{}\",\"dimension\":\"{}\",\"full_scan\":{},\"default_item\":\"{}\",\"loot_tables\":{}}}",
                    structure.name,
                    structure.dimension,
                    structure.supports_full_scan(),
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
        println!("  ancient_city and bastion_remnant support chests and find.");
        println!("  Other entries support candidate calculation only.");
        for (index, structure) in CANDIDATE_STRUCTURES.iter().enumerate() {
            println!("\n[{}] {}", index + 1, structure.name);
            println!("  Dimension: {}", structure.dimension);
            println!(
                "  Commands: {}",
                if structure.supports_full_scan() {
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
    if args.common.json {
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
            structure.supports_full_scan(),
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
        if structure.supports_full_scan() {
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
