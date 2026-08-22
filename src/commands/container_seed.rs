use crate::catalog::{ContainerSeedSpec, DecorationSeedSpec, candidate_structure};
use crate::cli::{ContainerSeedArgs, require_version};
use crate::decoration_seed::container_loot_seed;
use crate::error::Error;
use crate::output::{ContainerSeedOutput, print_json};
use crate::worldgen::Scanner;

pub fn run(args: ContainerSeedArgs) -> Result<u8, Error> {
    require_version(&args.common.version)?;
    let structure = candidate_structure(&args.structure)?;
    let strategy = structure.container_seed.ok_or_else(|| {
        Error::Usage(format!(
            "container-seed is not available for {}; use 'chests' to execute vanilla placement",
            structure.name
        ))
    })?;
    let world_seed = args.seed;
    let chunk_x = args.chunk_x;
    let chunk_z = args.chunk_z;
    let ordinal = args.ordinal;

    let (structure_index, step, loot_table_seed, chunk_label) = match strategy {
        ContainerSeedSpec::Decoration(default) => {
            let decoration = DecorationSeedSpec {
                structure_index: args.structure_index.unwrap_or(default.structure_index),
                step: args.step.unwrap_or(default.step),
                ..default
            };
            (
                Some(decoration.structure_index),
                Some(decoration.step),
                container_loot_seed(world_seed, chunk_x, chunk_z, decoration, ordinal)?,
                "Decoration",
            )
        }
        ContainerSeedSpec::StructureScan => {
            if args.structure_index.is_some() || args.step.is_some() {
                return Err(Error::Usage(format!(
                    "{} derives container seeds from exact structure placement; \
                     --structure-index and --step do not apply",
                    structure.name
                )));
            }
            let scanner = Scanner::for_structure(structure, world_seed)?;
            let scan = scanner
                .scan_many([(chunk_x, chunk_z)])?
                .into_iter()
                .next()
                .expect("one requested chunk must produce one scan");
            if !scan.valid_structure {
                return Err(Error::Usage(format!(
                    "chunk ({chunk_x}, {chunk_z}) is not a valid {} for seed {world_seed}",
                    structure.name
                )));
            }
            let chest = scan
                .chests
                .into_iter()
                .find(|chest| chest.ordinal == ordinal)
                .ok_or_else(|| {
                    Error::Usage(format!(
                        "{} at chunk ({chunk_x}, {chunk_z}) has no container ordinal {ordinal}",
                        structure.name
                    ))
                })?;
            (None, None, chest.loot_seed, "Structure")
        }
    };

    if args.common.json {
        let output = ContainerSeedOutput {
            version: "26.1.2",
            structure: structure.name,
            world_seed,
            chunk_x,
            chunk_z,
            structure_index,
            step,
            ordinal,
            loot_table_seed,
        };
        print_json(&output);
    } else {
        println!("Minecraft Java 26.1.2");
        println!("Structure: {}", structure.name);
        println!("World seed: {world_seed}");
        println!("{chunk_label} chunk: ({chunk_x}, {chunk_z})");
        println!("Container ordinal: {ordinal}\n");
        println!("LootTableSeed: {loot_table_seed}");
    }
    Ok(0)
}
