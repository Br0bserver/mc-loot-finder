use crate::catalog::{CANDIDATE_STRUCTURES, SpreadType, candidate_structure};
use crate::cli::{ExplainArgs, require_version};
use crate::error::Error;
use crate::output::{
    ExplainDetailOutput, ExplainListingOutput, ExplainStructureJson, PlacementJson, grouped,
    print_json,
};

pub fn run(args: ExplainArgs) -> Result<u8, Error> {
    require_version(&args.common.version)?;
    let structure_name = args.structure.as_deref().unwrap_or("");
    if structure_name.is_empty() {
        if args.common.json {
            let output = ExplainListingOutput {
                version: "26.1.2",
                structures: CANDIDATE_STRUCTURES
                    .iter()
                    .map(|structure| ExplainStructureJson {
                        name: structure.name,
                        dimension: structure.dimension,
                        full_scan: structure.supports_full_scan(),
                        default_item: structure.default_item,
                        loot_tables: structure.loot_tables.len(),
                    })
                    .collect(),
            };
            print_json(&output);
            return Ok(0);
        }
        println!("Minecraft Java 26.1.2\n");
        println!("Command defaults:");
        println!("  candidates: ancient_city, center (0, 0), radius 5,000, limit 100");
        println!("  chests: ancient_city, center (0, 0), radius 2,000, limit 100");
        println!("  find: ancient_city, center (0, 0), radius 5,000, limit 20");
        println!("  loot: minecraft:chests/ancient_city\n");
        let full_scan_names = CANDIDATE_STRUCTURES
            .iter()
            .filter(|structure| structure.supports_full_scan())
            .map(|structure| structure.name)
            .collect::<Vec<_>>()
            .join(", ");
        println!("Structure capabilities:");
        println!("  Full scans: {full_scan_names}.");
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
        let output = ExplainDetailOutput {
            version: "26.1.2",
            name: structure.name,
            structure_id: structure.structure_id,
            dimension: structure.dimension,
            full_scan: structure.supports_full_scan(),
            default_item: structure.default_item,
            placement: PlacementJson {
                spacing: structure.placement.spacing,
                separation: structure.placement.separation,
                salt: structure.placement.salt,
                spread: match structure.placement.spread {
                    SpreadType::Linear => "LINEAR",
                    SpreadType::Triangular => "TRIANGULAR",
                },
            },
            decoration_step: structure.decoration.map_or(-1, |spec| spec.step),
            decoration_index: structure.decoration.map_or(-1, |spec| spec.structure_index),
            scanner: structure.reference_scanner.as_str(),
            container_seed_shortcut: structure
                .decoration
                .map_or("NONE", |spec| spec.shortcut.as_str()),
            loot_tables: structure.loot_tables.to_vec(),
        };
        print_json(&output);
        return Ok(0);
    }

    let spread = match structure.placement.spread {
        SpreadType::Linear => "LINEAR",
        SpreadType::Triangular => "TRIANGULAR",
    };
    let decoration_step = structure.decoration.map_or(-1, |spec| spec.step);
    let structure_index = structure.decoration.map_or(-1, |spec| spec.structure_index);
    let shortcut = structure
        .decoration
        .map_or("NONE", |spec| spec.shortcut.as_str());
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
    println!("  Decoration step: {decoration_step}");
    println!("  Structure index: {structure_index}");
    println!("  Scanner: {}", structure.reference_scanner.as_str());
    println!("  Seed shortcut: {shortcut}\n");
    println!("Loot tables:");
    for table in structure.loot_tables {
        println!("  {table}");
    }
    Ok(0)
}
