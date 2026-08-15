use crate::catalog::{ContainerSeedShortcut, candidate_structure};
use crate::cli::{ContainerSeedArgs, require_version};
use crate::decoration_seed::container_loot_seed;

pub fn run(args: ContainerSeedArgs) -> Result<u8, String> {
    require_version(&args.common.version)?;
    let structure = candidate_structure(&args.structure)?;
    if structure.container_seed == ContainerSeedShortcut::None {
        return Err(format!(
            "container-seed is not available for {}; use 'chests' to execute vanilla placement",
            structure.name
        ));
    }
    let world_seed = args.seed;
    let chunk_x = args.chunk_x;
    let chunk_z = args.chunk_z;
    let structure_index = args.structure_index.unwrap_or(structure.structure_index);
    let step = args.step.unwrap_or(structure.decoration_step);
    let ordinal = args.ordinal;
    let loot_table_seed = container_loot_seed(
        world_seed,
        chunk_x,
        chunk_z,
        structure_index,
        step,
        ordinal,
        structure.container_seed,
    )?;

    if args.common.json {
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
