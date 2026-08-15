use crate::catalog::candidate_structure;
use crate::cli::{SearchArgs, require_version};
use crate::output::{
    CandidateJson, CandidatesOutput, grouped, print_json, quantity, rounded_distance,
};
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
        let output = CandidatesOutput {
            version: "26.1.2",
            structure: structure.name,
            seed,
            status: "candidate_only",
            candidates: candidates
                .iter()
                .take(limit as usize)
                .map(|candidate| CandidateJson {
                    chunk_x: candidate.chunk_x,
                    chunk_z: candidate.chunk_z,
                    block_x: candidate.block_x,
                    block_z: candidate.block_z,
                    distance: rounded_distance(candidate.squared_distance),
                })
                .collect(),
        };
        print_json(&output);
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
