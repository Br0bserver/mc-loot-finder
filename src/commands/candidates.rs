use crate::catalog::candidate_structure;
use crate::cli::{SearchArgs, require_version};
use crate::output::{grouped, quantity};
use crate::placement;

pub fn run(args: SearchArgs) -> Result<u8, String> {
    require_version(&args.common.version)?;
    let structure = candidate_structure(&args.structure)?;
    let seed = args.seed;
    let center_x = args.center_x;
    let center_z = args.center_z;
    let radius = args.radius.unwrap_or(5_000);
    let limit = args.limit.unwrap_or(100);
    let candidates = placement::locate(seed, center_x, center_z, radius, structure.placement)?;

    if args.common.json {
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
