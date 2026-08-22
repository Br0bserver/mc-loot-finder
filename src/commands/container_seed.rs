use crate::catalog::{DecorationSeedSpec, candidate_structure};
use crate::cli::{ContainerSeedArgs, require_version};
use crate::decoration_seed::container_loot_seed;
use crate::error::Error;
use crate::output::{ContainerSeedOutput, print_json};

pub fn run(args: ContainerSeedArgs) -> Result<u8, Error> {
    require_version(&args.common.version)?;
    let structure = candidate_structure(&args.structure)?;
    let mut decoration = structure.decoration.ok_or_else(|| {
        Error::Usage(format!(
            "container-seed is not available for {}; use 'chests' to execute vanilla placement",
            structure.name
        ))
    })?;
    let world_seed = args.seed;
    let chunk_x = args.chunk_x;
    let chunk_z = args.chunk_z;
    decoration = DecorationSeedSpec {
        structure_index: args.structure_index.unwrap_or(decoration.structure_index),
        step: args.step.unwrap_or(decoration.step),
        ..decoration
    };
    let ordinal = args.ordinal;
    let loot_table_seed = container_loot_seed(world_seed, chunk_x, chunk_z, decoration, ordinal)?;

    if args.common.json {
        let output = ContainerSeedOutput {
            version: "26.1.2",
            structure: structure.name,
            world_seed,
            chunk_x,
            chunk_z,
            structure_index: decoration.structure_index,
            step: decoration.step,
            ordinal,
            loot_table_seed,
        };
        print_json(&output);
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
